// SPDX-License-Identifier: LGPL-3.0-or-later
//! Sea level from the ocean's water: the level at which the planet's surface water, less what its
//! land ice holds, fills the open sea of the planet's ground (Ben, decisions log 2026-10-21: sea
//! level follows the ocean basins' volume, ridges, continents, ice and sediment; the fixed curve
//! is gone). docs/science/sea-level.md has the why and the checks.
//!
//! # The rule
//! The planet holds `planet.surface_water_m` of water, as a layer over its whole area (Earth's
//! ocean and land ice, ledger ISO-085). Each step the land ice takes its share as water
//! (ρ_i/ρ_w of its volume, the climate's ice sheets), and the rest stands in the sea: the level S
//! with
//!
//!   Σ_{c in the sea at S} (S − h_c) · a_c = V_water,
//!
//! h the ground the surface stage starts the step from and a the cells' true areas. The sea at S
//! is what the surface itself calls sea (`surface::pflood::open_sea`): ground at or below S in a
//! connected body of at least `min_cells` cells, so a closed hollow below S away from the ocean
//! takes none of its water (ledger ISO-086).
//!
//! # How it is solved
//! The plain hypsometric level S₀ (every cell below it counted, connected or not) is the lowest S
//! can be: the sea at any level is a subset of the ground below it. The open sea at S₀ is the
//! ocean's seed; a priority flood from it gives every other cell the level c at which the sea
//! reaches it (its spill height: the lowest pass between it and the ocean, or its own ground).
//! Over the cells sorted by c the sea's water is piecewise linear in S, V(S) = S·A − B with A the
//! area and B the area-weighted ground of the cells with c ≤ S, and it jumps where the sea tops a
//! pass and floods the hollow behind it. The walk up that list stops where V reaches the water:
//! between two passes S = (V + B)/A; at a pass whose hollow takes the rest, S is the pass.
//! Exact for the ground given, and one flood and one sort of the cells a step. The open sea at
//! that level is taken once more: where a body grew to `min_cells` on the way up without
//! reaching the ocean (the seeds at S₀ missed it), the level is bisected to 1 mm on the open
//! sea's own water instead (ledger ISO-086).

use rayon::prelude::*;

use crate::surface::grid::{Dims, SurfaceGrid};
use crate::surface::params::FloodMethod;
use crate::surface::pflood;

/// The water a cubic metre of land ice holds, m³: ρ_i/ρ_w with the climate's densities
/// (`climate::ice`). ledger: CLM-099, CLM-210
fn ice_to_water() -> f64 {
    crate::climate::ice::RHO_ICE / crate::climate::ice::RHO_SEA
}

/// The water layer (m over the whole planet) that `ice_km3` of land ice takes out of the sea, the
/// planet's area `area_km2` (both in the same normalisation).
pub fn ice_layer_m(ice_km3: f64, area_km2: f64) -> f64 {
    if area_km2 <= 0.0 || !ice_km3.is_finite() {
        return 0.0;
    }
    ice_to_water() * ice_km3.max(0.0) / area_km2 * 1000.0
}

/// The sea level, m, at which `layer_m` metres of water over the whole planet fill the open sea of
/// ground `h` (m, the grid `d`), each row's cells `row_area` in area (any unit: only shares
/// count). `min_cells` and `method` are the surface's open-sea rule and flood (see the module
/// docs). Ground that holds no sea at all gives its lowest cell.
pub fn fill_level(
    d: &Dims,
    h: &[f64],
    row_area: &[f64],
    layer_m: f64,
    min_cells: usize,
    method: FloodMethod,
) -> f64 {
    debug_assert_eq!(row_area.len(), d.h);
    let w = d.w;
    let total: f64 = row_area.iter().sum::<f64>() * w as f64;
    fill_level_with(d, h, |c| row_area[c / w], total, layer_m, min_cells, method)
}

/// [`fill_level`] on any grid with its own cell areas (`SurfaceGrid::area`; the cube's are true
/// areas), the planet's area their sum in cell order.
pub fn fill_level_on<G: SurfaceGrid>(
    d: &G,
    h: &[f64],
    layer_m: f64,
    min_cells: usize,
    method: FloodMethod,
) -> f64 {
    // ledger: SCL-150 (game-choice: true areas, summed in cell order)
    let total: f64 = (0..d.cells()).map(|c| d.area(c)).sum();
    fill_level_with(d, h, |c| d.area(c), total, layer_m, min_cells, method)
}

/// The level with cell areas `a` that sum to `total` (the module notes, "How it is solved").
fn fill_level_with<G: SurfaceGrid, A: Fn(usize) -> f64 + Sync>(
    d: &G,
    h: &[f64],
    a: A,
    total: f64,
    layer_m: f64,
    min_cells: usize,
    method: FloodMethod,
) -> f64 {
    let n = d.cells();
    debug_assert_eq!(h.len(), n);
    let water = layer_m.max(0.0) * total;
    // 1. The plain hypsometric level.
    let mut order = ascending(n, |c| h[c]);
    let s0 = walk(
        order
            .iter()
            .map(|&c| (h[c as usize], h[c as usize], a(c as usize))),
        water,
    )
    .unwrap_or(h[order[0] as usize]);
    // 2. The open sea at S₀ and every other cell's spill height.
    let mut kind = vec![pflood::LAND; n];
    pflood::open_sea_with(method, d, h, s0, min_cells, &mut kind);
    if !kind.contains(&pflood::BASE) {
        return s0;
    }
    let mut spill = vec![0.0f64; n];
    pflood::flood_with(method, d, h, h, &mut kind, &mut spill);
    // 3. The walk up the spill heights.
    order = ascending(n, |c| spill[c]);
    let s1 = walk(
        order.iter().map(|&c| {
            let c = c as usize;
            (spill[c].max(h[c]), h[c], a(c))
        }),
        water,
    )
    .unwrap_or(s0);
    // 4. The open sea at S₁ itself (ledger: ISO-086). The seeds at S₀ miss a body of water that
    // grows to `min_cells` between S₀ and S₁ without reaching the ocean: the surface calls it
    // sea at S₁, and the walk counted it as a lake. Then the level is where the open sea's own
    // water, V(S) = Σ over the open sea at S of (S − h)·a, reaches the planet's: V only grows
    // with S (the bodies only grow), V(S₀) ≤ the water ≤ V(S₁), and where a body's turn to be
    // sea makes V jump past the water, the level stays at that turn, as at a pass.
    let mut at = vec![pflood::LAND; n];
    pflood::open_sea_with(method, d, h, s1, min_cells, &mut at);
    let missed = (0..n).any(|c| at[c] == pflood::BASE && spill[c].max(h[c]) > s1);
    if !missed {
        return s1;
    }
    let volume = |s: f64, at: &mut [u8]| -> f64 {
        at.fill(pflood::LAND);
        pflood::open_sea_with(method, d, h, s, min_cells, at);
        // In cell order: the same sum whatever the thread count.
        (0..n)
            .filter(|&c| at[c] == pflood::BASE)
            .map(|c| (s - h[c]).max(0.0) * a(c))
            .sum::<f64>()
    };
    let (mut lo, mut hi) = (s0, s1);
    // To a millimetre (FILL_TOLERANCE_M): 2⁻ᵏ of S₁ − S₀, the closed hollows' water over the
    // sea's area.
    while hi - lo > FILL_TOLERANCE_M {
        let mid = 0.5 * (lo + hi);
        if volume(mid, &mut at) >= water {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    hi
}

/// The cells `0..n` in ascending order of `key` (`f64::total_cmp`), ties in cell order: the
/// permutation sorting the indices with that comparison gives (the keys and indices together are
/// distinct, so every correct sort gives it), sorted on packed keys, the key's total-order bits and
/// the index in one `u128`, so the sort reads contiguous memory instead of looking each key up
/// (the 2026-10-27 profile: the two indirect sorts were most of the sea level's time).
fn ascending(n: usize, key: impl Fn(usize) -> f64 + Sync) -> Vec<u32> {
    let mut packed: Vec<u128> = (0..n)
        .into_par_iter()
        .map(|c| (u128::from(total_order_bits(key(c))) << 32) | c as u128)
        .collect();
    packed.par_sort_unstable();
    packed.into_par_iter().map(|x| x as u32).collect()
}

/// An unsigned integer whose order is `f64::total_cmp`'s (IEEE 754 totalOrder): negative values
/// with their bits flipped, the rest with the sign bit set.
fn total_order_bits(x: f64) -> u64 {
    let b = x.to_bits();
    if b >> 63 == 1 { !b } else { b | (1 << 63) }
}

/// How close the level found by halving comes to the open sea's own (`fill_level`, step 4), m
/// (game-choice: numerical, agent 2026-10-21).
const FILL_TOLERANCE_M: f64 = 1.0e-3;

/// The level at which `water` fills the cells `(reached at, ground, area)`, sorted by the level
/// each is reached at; None for no cells.
fn walk(cells: impl Iterator<Item = (f64, f64, f64)>, water: f64) -> Option<f64> {
    let (mut area, mut ground) = (0.0f64, 0.0f64);
    let mut any = false;
    for (reach, z, a) in cells {
        // Between the last pass and this one: the cells already in.
        if area > 0.0 && reach * area - ground >= water {
            return Some((water + ground) / area);
        }
        area += a;
        ground += z * a;
        any = true;
        // At this pass, the hollow behind it flooded.
        if reach * area - ground >= water {
            return Some(reach);
        }
    }
    any.then(|| (water + ground) / area.max(f64::MIN_POSITIVE))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The packed sort gives the permutation the indirect `total_cmp` sort gave, on heights with
    /// ties, both zeros, infinities and NaNs.
    #[test]
    fn the_packed_sort_equals_the_indirect_one() {
        let mut s = 12345u64;
        let mut next = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let special = [
            0.0,
            -0.0,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            -f64::NAN,
            1.5,
            -1.5,
        ];
        let h: Vec<f64> = (0..50_000)
            .map(|_| {
                let r = next();
                match r % 10 {
                    0 => special[(r / 10 % 8) as usize],
                    1 => ((r / 10) % 7) as f64 - 3.0,
                    _ => (r as f64 / u64::MAX as f64 - 0.5) * 2.0e4,
                }
            })
            .collect();
        let mut old: Vec<u32> = (0..h.len() as u32).collect();
        old.par_sort_unstable_by(|&x, &y| {
            h[x as usize]
                .total_cmp(&h[y as usize])
                .then_with(|| x.cmp(&y))
        });
        assert_eq!(ascending(h.len(), |c| h[c]), old);
    }

    fn flat(w: usize, h: usize) -> (Dims, Vec<f64>) {
        (Dims::new(w, h, 1.0, true), vec![1.0; h])
    }

    #[test]
    fn a_bathtub_fills_by_its_area() {
        // Half the planet 1,000 m deep, half at +500 m: 400 m of water over the planet fills the
        // deep half 800 m deep, so the sea stands at −200 m.
        let (d, rows) = flat(16, 8);
        let h: Vec<f64> = (0..d.cells())
            .map(|c| if c % 16 < 8 { -1000.0 } else { 500.0 })
            .collect();
        let s = fill_level(&d, &h, &rows, 400.0, 1, FloodMethod::Serial);
        assert!((s - -200.0).abs() < 1e-9, "{s}");
        // Above the land it spreads over everything: 1,000 m fills the deep half to +500 m (750 m
        // of it) and the other 250 m stand over the whole planet, at +750 m.
        let s = fill_level(&d, &h, &rows, 1000.0, 1, FloodMethod::Serial);
        assert!((s - 750.0).abs() < 1e-9, "{s}");
    }

    #[test]
    fn rows_count_by_their_area() {
        // Row 0 deep and four times the area of the rest's cells: its share of the water.
        let (d, mut rows) = flat(4, 2);
        rows[0] = 4.0;
        let h = vec![-100.0, -100.0, -100.0, -100.0, 50.0, 50.0, 50.0, 50.0];
        // 16 area units deep (4 cells × 4); water 0.5 m × 20 = 10 units·m: S = −100 + 10/16.
        let s = fill_level(&d, &h, &rows, 0.5, 1, FloodMethod::Serial);
        assert!((s - (-100.0 + 10.0 / 16.0)).abs() < 1e-9, "{s}");
    }

    #[test]
    fn a_closed_hollow_waits_for_the_sea_to_top_its_rim() {
        // An ocean (columns 0 to 5, 48 cells at −2,000 m), land at +100 m, and a hollow (columns
        // 10 to 13, rows 2 to 5, 16 cells at −3,000 m) behind a pass at +40 m (column 9, row 3)
        // that a channel at +30 m (columns 6 to 8, row 3) joins to the ocean.
        let (d, rows) = flat(16, 8);
        let mut h = vec![100.0; d.cells()];
        for j in 0..8 {
            for i in 0..6 {
                h[j * 16 + i] = -2000.0;
            }
        }
        for j in 2..6 {
            for i in 10..14 {
                h[j * 16 + i] = -3000.0;
            }
        }
        for i in 6..9 {
            h[3 * 16 + i] = 30.0;
        }
        h[3 * 16 + 9] = 40.0;
        // 128 cells of unit area: the water, in area·m, as a layer.
        let level = |water: f64| fill_level(&d, &h, &rows, water / 128.0, 20, FloodMethod::Serial);
        // A little water: the ocean alone takes it (the hollow is deeper but closed).
        let s = level(48_000.0);
        assert!((s - -1000.0).abs() < 1e-9, "{s}");
        // Plain hypsometry would have put a third of it in the hollow (−1,500 m).
        assert!(s > -1500.0 + 1e-6);
        // The ocean and the channel up to the pass hold 97,950; the hollow takes 48,640 more.
        let (to_pass, with_hollow) = (48.0 * 2040.0 + 3.0 * 10.0, 146_590.0);
        // Anything between the two leaves the sea at the pass (the water pours into the hollow):
        // at the pass's 40 m plus the flood's ε (`next_up` a cell, pflood's module docs).
        let s = level(0.5 * (to_pass + with_hollow));
        assert!((40.0..40.0 + 1e-9).contains(&s), "{s}");
        // Past the hollow's room, the 68 cells rise together.
        let s = level(with_hollow + 10.0 * 68.0);
        assert!((s - 50.0).abs() < 1e-6, "{s}");
    }

    #[test]
    fn a_hollow_that_turns_sea_on_the_way_up_is_sea_at_the_level() {
        // An ocean (columns 0 to 5, 48 cells at −2,000 m) and, behind land at +100 m, a closed
        // hollow of 24 cells: 8 at −2,500 m (columns 9 to 12, rows 2 and 3) and 16 at −1,600 m
        // (the rest of columns 9 to 14, rows 2 to 5). The open sea's rule takes 20 cells.
        let (d, rows) = flat(16, 8);
        let mut h = vec![100.0; d.cells()];
        for j in 0..8 {
            for i in 0..6 {
                h[j * 16 + i] = -2000.0;
            }
        }
        for j in 2..6 {
            for i in 9..15 {
                h[j * 16 + i] = if j < 4 && i < 13 { -2500.0 } else { -1600.0 };
            }
        }
        // 20,800 area·m of water. Plain hypsometry puts it at −1,700 m (the 8 deep cells are
        // too few to be sea there); the ocean alone would hold it at −1,566.7 m, where the whole
        // hollow is under the level and 24 cells: sea by the rule. With the hollow as sea, the
        // water stands below −1,600 m, where the hollow is not sea: the level stays at the turn.
        let s = fill_level(&d, &h, &rows, 20_800.0 / 128.0, 20, FloodMethod::Serial);
        assert!((-1600.0..-1600.0 + 2.0e-3).contains(&s), "{s}");
        // With water enough for both, the hollow fills with the ocean: 26,400 at −1,600 m, and
        // 72 cells rise together after.
        let s = fill_level(
            &d,
            &h,
            &rows,
            (26_400.0 + 72.0 * 50.0) / 128.0,
            20,
            FloodMethod::Serial,
        );
        assert!((s - -1550.0).abs() < 2.0e-3, "{s}");
    }

    #[test]
    fn the_parallel_flood_gives_the_same_level() {
        let (d, rows) = flat(64, 32);
        let h: Vec<f64> = (0..d.cells())
            .map(|c| {
                let (i, j) = ((c % 64) as f64, (c / 64) as f64);
                3000.0 * (i * 0.21).sin() * (j * 0.17).cos() - 800.0 + 40.0 * (i * j * 0.013).sin()
            })
            .collect();
        for layer in [10.0, 300.0, 1500.0] {
            let a = fill_level(&d, &h, &rows, layer, 20, FloodMethod::Serial);
            let b = fill_level(&d, &h, &rows, layer, 20, FloodMethod::Parallel);
            assert!((a - b).abs() < 1e-6, "{layer}: {a} vs {b}");
        }
    }

    #[test]
    fn ice_takes_its_water_from_the_layer() {
        // 3.6·10⁶ km³ of ice on a planet of 5.1·10⁸ km²: ρ_i/ρ_w · 7.06 m.
        let x = ice_layer_m(3.6e6, 5.1e8);
        assert!((x - 917.0 / 1028.0 * 3.6e6 / 5.1e8 * 1000.0).abs() < 1e-9);
        assert_eq!(ice_layer_m(-1.0, 5.1e8), 0.0);
    }
}

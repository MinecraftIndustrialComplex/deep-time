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
//! Exact for the ground given, and one flood and one sort of the cells a step.

use rayon::prelude::*;

use crate::surface::grid::Dims;
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
    let n = d.cells();
    debug_assert_eq!(h.len(), n);
    debug_assert_eq!(row_area.len(), d.h);
    let w = d.w;
    let a = |c: usize| row_area[c / w];
    let total: f64 = row_area.iter().sum::<f64>() * w as f64;
    let water = layer_m.max(0.0) * total;
    // 1. The plain hypsometric level.
    let mut order: Vec<u32> = (0..n as u32).collect();
    order.par_sort_unstable_by(|&x, &y| {
        h[x as usize]
            .total_cmp(&h[y as usize])
            .then_with(|| x.cmp(&y))
    });
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
    order.par_sort_unstable_by(|&x, &y| {
        spill[x as usize]
            .total_cmp(&spill[y as usize])
            .then_with(|| x.cmp(&y))
    });
    walk(
        order.iter().map(|&c| {
            let c = c as usize;
            (spill[c].max(h[c]), h[c], a(c))
        }),
        water,
    )
    .unwrap_or(s0)
}

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
        // Above the land it spreads over everything.
        let s = fill_level(&d, &h, &rows, 1000.0, 1, FloodMethod::Serial);
        assert!((s - 1250.0).abs() < 1e-9, "{s}");
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
        // Anything between the two leaves the sea at the pass (the water pours into the hollow).
        assert_eq!(level(0.5 * (to_pass + with_hollow)), 40.0);
        // Past the hollow's room, the 68 cells rise together.
        let s = level(with_hollow + 10.0 * 68.0);
        assert!((s - 50.0).abs() < 1e-6, "{s}");
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

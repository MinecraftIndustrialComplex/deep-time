// SPDX-License-Identifier: LGPL-3.0-or-later
//! The ice cover: `climate/ice_cover` (L2, u8, blocks).
//!
//! Deep Time's own surface pass lays snow and packed ice on glaciated ground whatever biome the
//! pack's biome source drew there (Ben, 2026-09-30, "Force snow/ice cover"): Terralith has no flat
//! inland ice biome, and its gravel desert would otherwise show through an ice cap. This module is
//! the bake's half: the **thickness of the cover** per L2 cell, `1 + ⌊H / step_m⌋` blocks up to the
//! table's `max_blocks` where the land carries ice and 0 elsewhere and on the sea, `H` the climate
//! stage's land-ice thickness (`climate/ice_thickness_m`, L1, m: the deep-time climate's ice sheets
//! and glaciers). A run without that field has no cover.
//!
//! **From L1 to L2** ([`downscale`]). The climate decides where ice stands on its L1 cells, each
//! cell at its mean height; an L2 cell sees its own ground, up to a few hundred metres lower or
//! higher. Three steps carry the ice down a level without moving it onto ground the climate left
//! bare or the local climate cannot hold:
//!
//! 1. **The extent is the climate's** (ledger BIO-082): an L2 land cell is a candidate where its
//!    nearest L1 cell carries ice (≥ 1 m, the glacier flag's threshold), so the cover covers the
//!    same area as the climate's ice. A bilinear resample of the thickness, read as "ice where it
//!    is at least a metre", spread every ice cell's ice half an L1 cell onto each bare neighbour.
//! 2. **Thickness, line and precipitation at the L2 cell**: the bilinear blend of the four L1 cells
//!    around it, over the iced ones for the thickness (a margin keeps its ice's thickness rather than
//!    thinning toward the bare neighbour) and over the cells with a value for the equilibrium line
//!    (`climate/ela_m`, NaN at sea) and the precipitation.
//! 3. **The body's balance closes** (ledger BIO-157): a glacier holds ice below its equilibrium line
//!    only as far as the ice flowing down from its accumulation area pays for the melt there; in a
//!    steady state the balance summed over the body is zero, which is what fixes the terminus (the
//!    accumulation-area ratio and area-altitude balance ratio methods: Meier & Post 1962; Porter
//!    1975; Osmaston 2005; Cuffey & Paterson 2010 §4.6; from-memory). Each 4-connected body of
//!    candidates is scored twice with the climate's own law
//!    ([`crate::climate::ice::surface_balance`]): on each cell's own ground (the ice surface at the
//!    ground plus the thickness), and at its L1 cell's mean candidate ground, as the climate saw it.
//!    The law is concave (the cap above the line, a steeper melt below it), so a body on rough
//!    ground balances worse than its L1 cells did. Where the first sum falls short of
//!    `min(0, second)` the body's most negative cells are given up one by one until it does not:
//!    a body the climate grew holds what it can feed, and a body the climate keeps shrinking (lagged
//!    ice under a warmer climate, a sheet's outlet) keeps its deficit, not more. An outlet glacier
//!    fed by a large accumulation area stays, on warm ground too (Franz Josef Glacier's tongue
//!    reaches the rain forest); a valley cell below the line of an L1 cell whose ice barely balances
//!    does not.
//!
//! The game reads the layer and the table's `ice_cover` record (`notes.bake.climate.ice_cover`)
//! and lays the blocks (`worldgen/IcePass`): the cover replaces the top blocks of the ground, so
//! the terrain's height is unchanged. Every reader of the layer (the game, the projection onto the
//! world grid, the workbench) reads it by one rule, `deeptime_format::raster::cover_of` (BIO-159).
//! The bake's L2 biome inputs take the cover as glaciated and EF ([`super::biomes`], BIO-158).

use deeptime_format::GridDesc;
use deeptime_format::raster::bilinear_weights;
use rayon::prelude::*;

use super::placement::IceCoverSpec;
use crate::climate::ICE_MIN_M;
use crate::climate::ice::surface_balance;
use crate::world::Field;

/// The climate's surface mass balance law (`climate::ice::surface_balance`): gradients above and
/// below the line, (m of ice/yr)/m, and the cap per m/yr of precipitation (GLA-069, GLA-070).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Law {
    pub beta_acc: f64,
    pub beta_abl: f64,
    pub acc_max: f64,
}

impl Law {
    fn at(&self, above_m: f64, precip_m_yr: f64) -> f64 {
        surface_balance(
            above_m,
            precip_m_yr,
            self.beta_acc,
            self.beta_abl,
            self.acc_max,
        )
    }
}

/// Precipitation where the run has none, m/yr (the climate's own default for the balance field).
const PRECIP_DEFAULT_M_YR: f64 = 0.5;

/// What the cover is made from.
pub struct Inputs<'a> {
    /// The climate's land-ice thickness, m (L1).
    pub thickness: &'a Field,
    /// The equilibrium-line altitude, m above the sea (L1, NaN at sea); without it no body is
    /// trimmed.
    pub ela: Option<&'a Field>,
    /// Precipitation, mm/yr (L1).
    pub precip: Option<&'a Field>,
    /// The L2 ground, m, on `l2`.
    pub elevation_m: &'a [f32],
    pub l2: &'a GridDesc,
    /// The sea level in the elevation's units.
    pub sea_m: f64,
    pub law: Law,
}

/// The cover and what the downscaling did.
#[derive(Clone, Debug, PartialEq)]
pub struct Cover {
    /// Blocks per L2 cell.
    pub blocks: Vec<u8>,
    /// L2 land cells whose nearest L1 cell carries ice: the climate's extent.
    pub climate_cells: usize,
    /// Of those, the cells their body's balance could not hold.
    pub trimmed_cells: usize,
    /// 4-connected bodies of the climate's extent, and how many lost cells.
    pub bodies: usize,
    pub trimmed_bodies: usize,
}

/// The L1 cells of the four-cell blend at block (bx, bz) of field `f`, their weights, and the
/// nearest cell (its index in `f.data`).
fn corners(f: &Field, bx: f64, bz: f64) -> ([usize; 4], [f64; 4], usize) {
    let g = &f.grid;
    let u = (bx - g.origin_block_x as f64) / f64::from(g.r);
    let v = (bz - g.origin_block_z as f64) / f64::from(g.r);
    let (fi, fj) = (u.floor(), v.floor());
    let (i0, j0) = (fi as i64, fj as i64);
    let at = |i: i64, j: i64| g.clamp_j(j) as usize * g.w as usize + g.wrap_i(i) as usize;
    let k = [
        at(i0, j0),
        at(i0 + 1, j0),
        at(i0, j0 + 1),
        at(i0 + 1, j0 + 1),
    ];
    let near = at((u + 0.5).floor() as i64, (v + 0.5).floor() as i64);
    (k, bilinear_weights(u - fi, v - fj), near)
}

/// The blend of `f` over the corners whose value `keep` accepts, the weights renormalised; NaN
/// when none is accepted.
fn blend_over(f: &Field, k: &[usize; 4], w: &[f64; 4], keep: impl Fn(f32) -> bool) -> f64 {
    let (mut sum, mut wt) = (0.0, 0.0);
    for (&c, &wc) in k.iter().zip(w) {
        let v = f.data[c];
        if keep(v) && wc > 0.0 {
            sum += wc * f64::from(v);
            wt += wc;
        }
    }
    if wt > 0.0 { sum / wt } else { f64::NAN }
}

/// The ice cover of the L2 grid from the climate's L1 ice (the module header's three steps).
pub fn downscale(inp: &Inputs<'_>, spec: &IceCoverSpec) -> Cover {
    let l2 = inp.l2;
    let (w, h) = (l2.w as usize, l2.h as usize);
    let n = w * h;
    assert_eq!(inp.elevation_m.len(), n, "the ground is on the L2 grid");
    let th = inp.thickness;
    let iced = |v: f32| v >= ICE_MIN_M;
    // Step 1 and 2: the candidates (only they are stored: an L2 grid has up to 33 M cells), their
    // thickness, line and precipitation, and their L1 cell.
    let rows: Vec<Vec<Cand>> = (0..h)
        .into_par_iter()
        .map(|j| {
            let bz = l2.block_z(j as i64) as f64;
            let mut row = Vec::new();
            for i in 0..w {
                let k = j * w + i;
                let z = f64::from(inp.elevation_m[k]);
                if z.is_nan() || z - inp.sea_m <= 0.0 {
                    continue;
                }
                let bx = l2.block_x(i as i64) as f64;
                let (ks, ws, near) = corners(th, bx, bz);
                if !iced(th.data[near]) {
                    continue;
                }
                let ela = inp.ela.map_or(f64::NAN, |f| {
                    let (ke, we, _) = corners(f, bx, bz);
                    blend_over(f, &ke, &we, f32::is_finite)
                });
                let p = inp
                    .precip
                    .map(|f| {
                        let (kp, wp, _) = corners(f, bx, bz);
                        blend_over(f, &kp, &wp, f32::is_finite) / 1000.0
                    })
                    .filter(|p| p.is_finite())
                    .unwrap_or(PRECIP_DEFAULT_M_YR);
                row.push(Cand {
                    k,
                    l1: near,
                    z,
                    h: blend_over(th, &ks, &ws, iced),
                    ela,
                    p,
                    body: u32::MAX,
                    kept: false,
                });
            }
            row
        })
        .collect();
    let mut cand: Vec<Cand> = rows.into_iter().flatten().collect();
    // Each L2 cell's candidate, by index into `cand`.
    let mut slot = vec![u32::MAX; n];
    for (s, c) in cand.iter().enumerate() {
        slot[c.k] = s as u32;
    }
    // Each L1 cell's mean candidate ground: the height its ice stood at in the climate.
    let mut mean: std::collections::HashMap<usize, (f64, u32)> = Default::default();
    for c in &cand {
        let e = mean.entry(c.l1).or_insert((0.0, 0));
        e.0 += c.z;
        e.1 += 1;
    }
    let balance = |c: &Cand, ground: f64| {
        if c.ela.is_nan() {
            0.0
        } else {
            inp.law.at(ground - inp.sea_m + c.h - c.ela, c.p)
        }
    };
    // Step 3: the bodies (4-connected; x wraps, z does not), each closed on its balance.
    let (mut bodies, mut trimmed_bodies, mut trimmed_cells) = (0, 0, 0);
    let mut stack = Vec::new();
    let mut cells: Vec<usize> = Vec::new();
    for start in 0..cand.len() {
        if cand[start].body != u32::MAX {
            continue;
        }
        let id = bodies as u32;
        bodies += 1;
        cells.clear();
        cand[start].body = id;
        stack.push(start);
        while let Some(s) = stack.pop() {
            cells.push(s);
            let k = cand[s].k;
            let (i, j) = (k % w, k / w);
            let mut next = [usize::MAX; 4];
            next[0] = j * w + (i + w - 1) % w;
            next[1] = j * w + (i + 1) % w;
            if j > 0 {
                next[2] = k - w;
            }
            if j + 1 < h {
                next[3] = k + w;
            }
            for m in next {
                let t = if m == usize::MAX { u32::MAX } else { slot[m] };
                if t != u32::MAX && cand[t as usize].body == u32::MAX {
                    cand[t as usize].body = id;
                    stack.push(t as usize);
                }
            }
        }
        cells.sort_unstable();
        let mut own = Vec::with_capacity(cells.len());
        let (mut sum_own, mut sum_l1) = (0.0, 0.0);
        for &s in &cells {
            let c = &cand[s];
            let (z, m) = mean[&c.l1];
            let b = balance(c, c.z);
            sum_own += b;
            sum_l1 += balance(c, z / f64::from(m));
            own.push((b, s));
        }
        for &s in &cells {
            cand[s].kept = true;
        }
        // The rounding of two sums of the same law over the same cells, not a shortfall.
        let floor = sum_l1.min(0.0) - 1e-9 * (1.0 + sum_l1.abs());
        if sum_own < floor {
            trimmed_bodies += 1;
            own.retain(|&(b, _)| b < 0.0);
            own.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            for &(b, s) in &own {
                if sum_own >= floor {
                    break;
                }
                cand[s].kept = false;
                sum_own -= b;
                trimmed_cells += 1;
            }
        }
    }
    let mut blocks = vec![0u8; n];
    for c in cand.iter().filter(|c| c.kept) {
        blocks[c.k] = spec.blocks(c.h);
    }
    Cover {
        blocks,
        climate_cells: cand.len(),
        trimmed_cells,
        bodies,
        trimmed_bodies,
    }
}

/// An L2 cell under the climate's ice (`downscale`'s steps 1 and 2).
struct Cand {
    /// Its L2 index, and its nearest L1 cell's.
    k: usize,
    l1: usize,
    /// Ground, ice thickness and equilibrium line, m; precipitation, m/yr.
    z: f64,
    h: f64,
    ela: f64,
    p: f64,
    /// Its body, and whether the body's balance holds it.
    body: u32,
    kept: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 256-block planet: L1 16 × 8 cells of 16 blocks, L2 32 × 16 of 8, the same origin.
    fn grids() -> (GridDesc, GridDesc) {
        (
            GridDesc::for_planet("L1", 256, 16).unwrap(),
            GridDesc::for_planet("L2", 256, 8).unwrap(),
        )
    }

    fn l1_field(g: &GridDesc, f: impl Fn(usize, usize) -> f32) -> Field {
        let w = g.w as usize;
        Field {
            grid: g.clone(),
            data: (0..g.cells()).map(|c| f(c % w, c / w)).collect(),
        }
    }

    const LAW: Law = Law {
        beta_acc: 0.005,
        beta_abl: 0.01,
        acc_max: 0.5,
    };

    fn cover(
        th: &Field,
        ela: Option<&Field>,
        precip: Option<&Field>,
        z: &[f32],
        l2: &GridDesc,
    ) -> Cover {
        downscale(
            &Inputs {
                thickness: th,
                ela,
                precip,
                elevation_m: z,
                l2,
                sea_m: 0.0,
                law: LAW,
            },
            &IceCoverSpec::default(),
        )
    }

    fn covered(c: &Cover, l2: &GridDesc) -> Vec<(usize, usize)> {
        let w = l2.w as usize;
        (0..c.blocks.len())
            .filter(|&k| c.blocks[k] > 0)
            .map(|k| (k % w, k / w))
            .collect()
    }

    #[test]
    fn the_cover_keeps_the_climates_extent_and_its_thickness() {
        let (l1, l2) = grids();
        // One iced L1 cell, (5, 3), 500 m thick, on high ground above its line.
        let th = l1_field(&l1, |i, j| if (i, j) == (5, 3) { 500.0 } else { 0.0 });
        let ela = l1_field(&l1, |_, _| 1000.0);
        let z = vec![2000.0; l2.cells()];
        let c = cover(&th, Some(&ela), None, &z, &l2);
        // The L2 cells nearest to it: columns 9 and 10, rows 5 and 6 (an odd L2 cell's nearest L1
        // cell is the higher one, `floor(u + 0.5)`), each with the cell's full 500 m.
        assert_eq!(covered(&c, &l2), [(9, 5), (10, 5), (9, 6), (10, 6)]);
        let full = IceCoverSpec::default().blocks(500.0);
        assert!(c.blocks.iter().all(|&b| b == 0 || b == full));
        assert_eq!((c.climate_cells, c.trimmed_cells, c.bodies), (4, 0, 1));
        // The bilinear resample read as ice where at least a metre covered 9 cells, the outer ones
        // at half the thickness.
        let spread = crate::bake::resample(&th, &l2, false);
        assert_eq!(spread.iter().filter(|&&t| t >= 1.0).count(), 9);
    }

    #[test]
    fn a_low_valley_cell_the_body_cannot_feed_is_given_up() {
        let (l1, l2) = grids();
        // L1 cell (5, 3), 50 m of ice, its four L2 cells at 1,300 m but one valley at 100 m: the
        // cell's mean ground is the line's 1,000 m, so the climate saw a body in balance
        // (+0.25 m/yr a cell), but on its own ground the valley melts 8.5 m/yr and the three
        // others gain at most 0.5.
        let th = l1_field(&l1, |i, j| if (i, j) == (5, 3) { 50.0 } else { 0.0 });
        let ela = l1_field(&l1, |_, _| 1000.0);
        let precip = l1_field(&l1, |_, _| 1000.0);
        let w = l2.w as usize;
        let mut z = vec![1300.0; l2.cells()];
        z[5 * w + 9] = 100.0;
        let c = cover(&th, Some(&ela), Some(&precip), &z, &l2);
        assert_eq!(covered(&c, &l2), [(10, 5), (9, 6), (10, 6)]);
        assert_eq!(
            (c.climate_cells, c.trimmed_cells, c.trimmed_bodies),
            (4, 1, 1)
        );
    }

    #[test]
    fn an_outlet_fed_by_its_accumulation_area_stays() {
        let (l1, l2) = grids();
        // Three iced L1 cells in a row, 50 m thick, on ground at 3,000 m against a line at
        // 1,000 m, and one tongue cell at 700 m: the eleven high cells gain 5.5 m/yr, the tongue
        // melts 2.5, so the body feeds it and it keeps its ice below the line.
        let th = l1_field(&l1, |i, j| {
            if j == 3 && (4..=6).contains(&i) {
                50.0
            } else {
                0.0
            }
        });
        let ela = l1_field(&l1, |_, _| 1000.0);
        let precip = l1_field(&l1, |_, _| 1000.0);
        let w = l2.w as usize;
        let mut z = vec![3000.0; l2.cells()];
        z[6 * w + 12] = 700.0;
        let c = cover(&th, Some(&ela), Some(&precip), &z, &l2);
        assert_eq!(c.climate_cells, 12);
        assert_eq!(c.trimmed_cells, 0);
        assert!(
            c.blocks[6 * w + 12] > 0,
            "the tongue below the line keeps its ice"
        );
    }

    #[test]
    fn lagged_ice_below_the_line_keeps_the_deficit_the_climate_gave_it() {
        let (l1, l2) = grids();
        // 200 m of ice under a line at 3,000 m: the climate's ice is shrinking everywhere. On flat
        // ground and on ground rough wholly below the line (the law is linear there) the body
        // balances exactly as the climate's cells did, so nothing goes.
        let th = l1_field(&l1, |i, j| {
            if (i, j) == (5, 3) || (i, j) == (6, 3) {
                200.0
            } else {
                0.0
            }
        });
        let ela = l1_field(&l1, |_, _| 3000.0);
        let flat = vec![1000.0; l2.cells()];
        let c = cover(&th, Some(&ela), None, &flat, &l2);
        assert_eq!((c.climate_cells, c.trimmed_cells), (8, 0));
        let w = l2.w as usize;
        let mut rough = flat.clone();
        rough[5 * w + 9] = 200.0;
        rough[6 * w + 10] = 1800.0;
        let c = cover(&th, Some(&ela), None, &rough, &l2);
        assert_eq!((c.climate_cells, c.trimmed_cells), (8, 0));
    }

    #[test]
    fn no_line_no_trim_and_the_sea_has_none() {
        let (l1, l2) = grids();
        let th = l1_field(&l1, |i, j| if (i, j) == (5, 3) { 3000.0 } else { 0.0 });
        let w = l2.w as usize;
        let mut z = vec![50.0; l2.cells()];
        z[5 * w + 9] = -20.0;
        z[6 * w + 10] = 0.0;
        let c = cover(&th, None, None, &z, &l2);
        // The two cells at or below the sea have none; the others the thickest cover.
        assert_eq!(covered(&c, &l2), [(10, 5), (9, 6)]);
        assert!(c.blocks.iter().all(|&b| b == 0 || b == 12));
        // Under 1 m and NaN are no ice.
        let thin = l1_field(&l1, |i, j| match (i, j) {
            (5, 3) => 0.5,
            (6, 3) => f32::NAN,
            _ => 0.0,
        });
        assert_eq!(cover(&thin, None, None, &z, &l2).climate_cells, 0);
    }

    #[test]
    fn a_body_across_the_seam_is_one_body() {
        let (l1, l2) = grids();
        // L1 columns 15 and 0 of row 3: their L2 cells are columns 29, 30 and 31, 0 (the seam).
        let th = l1_field(&l1, |i, j| {
            if j == 3 && (i == 15 || i == 0) {
                100.0
            } else {
                0.0
            }
        });
        let z = vec![500.0; l2.cells()];
        let c = cover(&th, None, None, &z, &l2);
        assert_eq!((c.climate_cells, c.bodies), (8, 1));
        assert_eq!(
            covered(&c, &l2),
            [
                (0, 5),
                (29, 5),
                (30, 5),
                (31, 5),
                (0, 6),
                (29, 6),
                (30, 6),
                (31, 6)
            ]
        );
    }
}

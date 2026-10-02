// SPDX-License-Identifier: LGPL-3.0-or-later
//! Inherited banded iron formation as a field of basins (Ben, 2026-10-02: "Earth-like patch
//! field"). When a run starts after the 2.6–1.85 Ga iron-formation window (`clocks`), some of the
//! stable continental interiors hold Paleoproterozoic iron formation in their basement. On Earth
//! they are a few basins of about 10⁵ km² each — Hamersley (about 100,000 km²), Transvaal (190,000
//! preserved), Lake Superior (220,000, the region enclosing the iron ranges), the Kursk anomaly
//! (120,000), Nabberu (60,000), the Labrador Trough (about 50,000), the Quadrilátero Ferrífero
//! (7,000) — about 0.75 million km² together, 1.4 % of Earth's 53.5 million km² of shields and
//! cratons (ledger ORE-263, ORE-264). Before this, every stable interior was permissive (412 BIF
//! sites on a quick 16k planet).
//!
//! [`deal`] marks that share of the stable continental interiors at the run's first surface step
//! as basins: coherent patches of about the preset's basin area (a lognormal around it), ellipses
//! of random elongation (1–3) and direction with edges roughened by smooth noise, never smaller
//! than a quarter of the basin area, so a planet gets a few whole basins rather than speckle; the
//! last basin takes what is left of the share, the share is never exceeded by more than one basin.
//! The mask rides with its crust (`surface`, the strata log's nearest-cell carry; crust born
//! later holds none), and the frames' `deeptime:inherited_bif` is the mask where the crust is a
//! stable interior (`surface::fields`).

use super::random::Rng;
use crate::surface::grid::{Dims, box_blur};

/// Spread (σ of ln area) of the basin areas around the preset's basin area: Earth's basins of
/// the window run from 7,000 to 220,000 km², a factor ~2 either side of 10⁵ km² for most.
/// game-choice (shape of the patch field; Earth's list in the module docs). ledger: ORE-264
pub const AREA_SIGMA: f64 = 0.5;

/// The smallest basin, as a share of the basin area: a leftover of the share below it is not
/// dealt (no speckle). game-choice. ledger: ORE-264
pub const MIN_BASIN_SHARE: f64 = 0.25;

/// How far the smooth noise moves a basin's edge, as a share of its radius. game-choice.
/// ledger: ORE-264
pub const EDGE_WARP: f64 = 0.25;

/// What [`deal`] made.
#[derive(Clone, Debug, PartialEq)]
pub struct Basins {
    /// 1 in a basin, 0 elsewhere, per cell of the grid.
    pub mask: Vec<f32>,
    /// Basins dealt.
    pub count: usize,
    /// The share of the eligible area they cover.
    pub share: f64,
}

/// Deals basins over the `eligible` cells (stable continental interior) of a grid of `d`: about
/// `share` of their area, basins of about `basin_km2` (block-metric km², as the deposit tracts
/// measure area). `seed` fixes the draw.
pub fn deal(d: &Dims, eligible: &[bool], share: f64, basin_km2: f64, seed: u64) -> Basins {
    let n = d.cells();
    let mut mask = vec![0.0f32; n];
    let cell_km2 = d.cell_area() / 1.0e6;
    let cells: Vec<usize> = (0..n).filter(|&c| eligible[c]).collect();
    let total_km2 = cells.len() as f64 * cell_km2;
    let target = share.clamp(0.0, 1.0) * total_km2;
    if cells.is_empty() || target <= 0.0 || basin_km2.is_nan() || basin_km2 <= 0.0 {
        return Basins {
            mask,
            count: 0,
            share: 0.0,
        };
    }
    let mut rng = Rng::stream(seed, "deposits/bif_basins");
    // Smooth noise for the edges, correlated over about a quarter of a basin's width.
    let dx_km = d.dx / 1000.0;
    let r = ((basin_km2.sqrt() / 4.0 / dx_km).round() as usize).max(1);
    let mut noise: Vec<f64> = (0..n).map(|_| rng.normal()).collect();
    box_blur(d, &mut noise, r, 2);
    let sd = (noise.iter().map(|v| v * v).sum::<f64>() / n as f64)
        .sqrt()
        .max(1e-12);
    let min_km2 = MIN_BASIN_SHARE * basin_km2;
    let (w, h) = (d.w as i64, d.h as i64);
    let mut placed = 0.0;
    let mut count = 0;
    for _ in 0..256 {
        let left = target - placed;
        if left <= 0.0 || (count > 0 && left < 0.5 * min_km2) {
            break;
        }
        let want = (basin_km2 * (AREA_SIGMA * rng.normal()).exp())
            .clamp(min_km2.min(left), left.max(min_km2.min(target)));
        let q = 1.0 + 2.0 * rng.uniform();
        let yaw = std::f64::consts::PI * rng.uniform();
        let centre = cells[rng.below(cells.len() as u64) as usize];
        if mask[centre] > 0.0 {
            continue;
        }
        // Semi-axes in cells: a·b·π = the area, a = q·b.
        let b_km = (want / (std::f64::consts::PI * q)).sqrt();
        let (a, b) = (q * b_km / dx_km, b_km / dx_km);
        let (ci, cj) = ((centre as i64) % w, (centre as i64) / w);
        let reach = (a * (1.0 + EDGE_WARP * 3.0)).ceil() as i64 + 1;
        let (s, c) = yaw.sin_cos();
        let mut got = 0.0;
        for dj in -reach..=reach {
            let j = cj + dj;
            if j < 0 || j >= h {
                continue;
            }
            for di in -reach..=reach {
                let i = (ci + di).rem_euclid(w);
                let k = (j * w + i) as usize;
                if !eligible[k] || mask[k] > 0.0 {
                    continue;
                }
                let (x, z) = (
                    di as f64 * c + dj as f64 * s,
                    -(di as f64) * s + dj as f64 * c,
                );
                let dist = ((x / a).powi(2) + (z / b).powi(2)).sqrt();
                if dist <= 1.0 + EDGE_WARP * noise[k] / sd {
                    mask[k] = 1.0;
                    got += cell_km2;
                }
            }
        }
        if got > 0.0 {
            placed += got;
            count += 1;
        }
    }
    Basins {
        mask,
        count,
        share: placed / total_km2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Connected patches (4-neighbours, x wrapping) of a mask, their sizes in cells.
    fn patches(d: &Dims, mask: &[f32]) -> Vec<usize> {
        let (w, h) = (d.w, d.h);
        let mut seen = vec![false; mask.len()];
        let mut out = Vec::new();
        for s in 0..mask.len() {
            if mask[s] <= 0.0 || seen[s] {
                continue;
            }
            let mut stack = vec![s];
            seen[s] = true;
            let mut size = 0;
            while let Some(c) = stack.pop() {
                size += 1;
                let (i, j) = (c % w, c / w);
                let mut next = vec![j * w + (i + 1) % w, j * w + (i + w - 1) % w];
                if j > 0 {
                    next.push((j - 1) * w + i);
                }
                if j + 1 < h {
                    next.push((j + 1) * w + i);
                }
                for k in next {
                    if mask[k] > 0.0 && !seen[k] {
                        seen[k] = true;
                        stack.push(k);
                    }
                }
            }
            out.push(size);
        }
        out
    }

    /// A 16k planet's L1 grid (4.8 km cells) with a band of stable interior across the middle.
    fn planet() -> (Dims, Vec<bool>) {
        let d = Dims::new(1024, 512, 4800.0, true);
        let eligible = (0..d.cells())
            .map(|c| (150..330).contains(&(c / d.w)) && (100..700).contains(&(c % d.w)))
            .collect();
        (d, eligible)
    }

    /// Before the patch field every stable interior was permissive. Now about the share of it is
    /// basin, in a few whole basins of about the basin area, none of them speckle.
    #[test]
    fn a_share_of_the_interiors_in_whole_basins() {
        let (d, eligible) = planet();
        let interior_km2 = eligible.iter().filter(|&&e| e).count() as f64 * 23.04;
        for seed in 1..6 {
            let b = deal(&d, &eligible, 0.05, 100_000.0, seed);
            // The share, never beyond one basin's worth.
            assert!(
                b.share > 0.035 && b.share < 0.05 + 2.0 * 100_000.0 / interior_km2,
                "{b:?}"
            );
            assert!(b.mask.iter().zip(&eligible).all(|(&m, &e)| m == 0.0 || e));
            let sizes = patches(&d, &b.mask);
            let km2: Vec<f64> = sizes.iter().map(|&s| s as f64 * 23.04).collect();
            assert!((1..=12).contains(&b.count), "{} basins", b.count);
            // Coherent: nearly all the area in patches of at least a tenth of a basin.
            let big: f64 = km2.iter().filter(|&&a| a >= 10_000.0).sum();
            assert!(big >= 0.9 * km2.iter().sum::<f64>(), "{km2:?}");
        }
    }

    /// Share 0 deals nothing; a small share on a small interior gives one basin, not speckle.
    #[test]
    fn no_share_no_basin_and_a_small_share_is_one_basin() {
        let (d, eligible) = planet();
        let none = deal(&d, &eligible, 0.0, 100_000.0, 7);
        assert_eq!(none.count, 0);
        assert!(none.mask.iter().all(|&m| m == 0.0));
        let one = deal(&d, &eligible, 0.014, 100_000.0, 7);
        assert!(one.count <= 2, "{one:?}");
        let sizes = patches(&d, &one.mask);
        assert!(*sizes.iter().max().unwrap() as f64 * 23.04 > 0.5 * 0.014 * 2_488_320.0);
    }

    /// The draw is fixed by the seed.
    #[test]
    fn the_seed_fixes_the_basins() {
        let (d, eligible) = planet();
        assert_eq!(
            deal(&d, &eligible, 0.03, 80_000.0, 3),
            deal(&d, &eligible, 0.03, 80_000.0, 3)
        );
        assert_ne!(
            deal(&d, &eligible, 0.03, 80_000.0, 3).mask,
            deal(&d, &eligible, 0.03, 80_000.0, 4).mask
        );
    }
}

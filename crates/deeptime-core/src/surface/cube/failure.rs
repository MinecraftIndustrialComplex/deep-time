// SPDX-License-Identifier: LGPL-3.0-or-later
//! The submarine slope failure (`marine::slope_failure`, ledger ERO-169) on the cube grid.
//!
//! The rule is the raster's: a pair of open-sea cells (a, b) with h_a − h_b > S_c·d moves
//! rate·(h_a − h_b − S_c·d) from a to b each iteration, scaled down for a giver that would give
//! more than its loose sediment `avail`. On the cube the cells differ in area, so the pair moves
//! a volume: the excess times the pair's mean area ½(a_a + a_b), which each end turns into metres
//! over its own area. On equal cells that is the raster's rule exactly; on the cube it keeps
//! Σ a·h to round-off. Every iteration visits every cell (the raster's neighbourhood skipping is
//! a speed-up of the same result and is not ported).

use rayon::prelude::*;

use super::ordered_sum;
use crate::sphere::cube::{CubeGrid, NO_CELL};
use crate::surface::marine::FailureParams;

/// Moves sediment down sea-floor slopes steeper than `p.critical_slope` on the open-sea cells
/// (`sea[c]`, below `sea_level`). Returns the metres moved over cells of the mean area
/// (Σ |Δh|·a/ā / 2).
pub fn slope_failure(
    g: &CubeGrid,
    sea: &[bool],
    sea_level: f64,
    p: &FailureParams,
    h: &mut [f64],
    avail: &mut [f64],
) -> f64 {
    let t = g.topology();
    let n = t.cells();
    if n == 0 {
        return 0.0;
    }
    let (a, mean) = (g.areas(), g.mean_area());
    let mut wet: Vec<bool> = (0..n)
        .into_par_iter()
        .map(|c| sea[c] && h[c] < sea_level)
        .collect();
    let mut scale = vec![0.0f64; n];
    let mut dz = vec![0.0f64; n];
    let mut moved = 0.0;
    for _ in 0..p.iterations {
        {
            let (hh, wt, av): (&[f64], &[bool], &[f64]) = (h, &wet, avail);
            scale.par_iter_mut().enumerate().for_each(|(c, s)| {
                *s = 0.0;
                if !wt[c] || av[c] <= 0.0 {
                    return;
                }
                let mut give = 0.0;
                for (k, &m) in t.slots(c).iter().enumerate() {
                    if m == NO_CELL || !wt[m as usize] {
                        continue;
                    }
                    let m = m as usize;
                    let ck = p.critical_slope * g.dists(c)[k];
                    // ledger: SCL-148 (game-choice: the pair's volume on its mean area)
                    give += p.rate * (hh[c] - hh[m] - ck).max(0.0) * (0.5 * (a[c] + a[m])) / a[c];
                }
                if give > 0.0 {
                    *s = (av[c] / give).min(1.0);
                }
            });
        }
        {
            let (hh, wt, sc): (&[f64], &[bool], &[f64]) = (h, &wet, &scale);
            dz.par_iter_mut().enumerate().for_each(|(c, out)| {
                *out = 0.0;
                if !wt[c] {
                    return;
                }
                let mut v = 0.0;
                for (k, &m) in t.slots(c).iter().enumerate() {
                    if m == NO_CELL || !wt[m as usize] {
                        continue;
                    }
                    let m = m as usize;
                    let ck = p.critical_slope * g.dists(c)[k];
                    let give = (hh[c] - hh[m] - ck).max(0.0);
                    let take = (hh[m] - hh[c] - ck).max(0.0);
                    v += p.rate * (take * sc[m] - give * sc[c]) * (0.5 * (a[c] + a[m]));
                }
                *out = v / a[c];
            });
        }
        let movers = dz.par_iter().filter(|&&v| v != 0.0).count();
        if movers == 0 {
            break;
        }
        let d: &[f64] = &dz;
        moved += ordered_sum(n, |r| r.map(|c| d[c].abs() * a[c] / mean * 0.5).sum());
        h.par_iter_mut()
            .zip(avail.par_iter_mut())
            .zip(wet.par_iter_mut())
            .enumerate()
            .for_each(|(c, ((hv, av), w))| {
                *hv += d[c];
                *av = (*av + d[c]).max(0.0);
                *w = sea[c] && *hv < sea_level;
            });
    }
    moved
}

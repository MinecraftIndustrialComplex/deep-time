// SPDX-License-Identifier: LGPL-3.0-or-later
//! The submarine slope failure (`marine::slope_failure`, ledger ERO-169) on the cube grid.
//!
//! The rule is the raster's: a pair of open-sea cells (a, b) with h_a − h_b > S_c·d moves
//! rate·(h_a − h_b − S_c·d) from a to b each iteration, scaled down for a giver that would give
//! more than its loose sediment `avail`. On the cube the cells differ in area, so the pair moves
//! a volume: the excess times the pair's mean area ½(a_a + a_b), which each end turns into metres
//! over its own area. On equal cells that is the raster's rule exactly; on the cube it keeps
//! Σ a·h to round-off. As on the raster, an iteration with few movers is followed by one that
//! visits only their neighbourhoods: a cell whose inputs did not change gives the change it gave
//! before, which was none.

use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

use super::CHUNK;
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
    // Above this many movers every cell is recomputed (the raster's bound).
    // game-choice: a speed bound only, the same bits either way (tested; Claude 2026-10-22)
    let dense_above = g.topology().cells() / 4;
    slope_failure_with(g, sea, sea_level, p, h, avail, dense_above)
}

/// [`slope_failure`] recomputing every cell after an iteration with more than `dense_above`
/// movers (0: every iteration recomputes every cell).
pub(super) fn slope_failure_with(
    g: &CubeGrid,
    sea: &[bool],
    sea_level: f64,
    p: &FailureParams,
    h: &mut [f64],
    avail: &mut [f64],
    dense_above: usize,
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
    // The share of what cell c would give that its loose sediment allows.
    let share = |c: usize, hh: &[f64], wt: &[bool], av: &[f64]| -> f64 {
        if !wt[c] || av[c] <= 0.0 {
            return 0.0;
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
            (av[c] / give).min(1.0)
        } else {
            0.0
        }
    };
    // Cell c's change in metres.
    let change = |c: usize, hh: &[f64], wt: &[bool], sc: &[f64]| -> f64 {
        if !wt[c] {
            return 0.0;
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
        v / a[c]
    };
    let chunks = n.div_ceil(CHUNK);
    let span = |q: usize| q * CHUNK..((q + 1) * CHUNK).min(n);
    // Neighbourhood marks: `near1[c] == stamp` for a mover and its neighbours (whose shares can
    // change), `near2` for those and their neighbours (whose changes can). Every store of an
    // iteration writes the same stamp, so the marks do not depend on the order of the stores.
    let near1: Vec<AtomicU32> = (0..n).map(|_| AtomicU32::new(0)).collect();
    let near2: Vec<AtomicU32> = (0..n).map(|_| AtomicU32::new(0)).collect();
    let mut stamp = 0u32;
    let shares = |scale: &mut [f64], all: bool, s: u32, hh: &[f64], wt: &[bool], av: &[f64]| {
        scale
            .par_chunks_mut(CHUNK)
            .enumerate()
            .for_each(|(q, part)| {
                for (d, x) in part.iter_mut().enumerate() {
                    let c = q * CHUNK + d;
                    if all || near1[c].load(Relaxed) == s {
                        *x = share(c, hh, wt, av);
                    }
                }
            });
    };
    let mut scale = vec![0.0f64; n];
    shares(&mut scale, true, stamp, &*h, &wet, &*avail);
    // Each chunk's movers and their changes, ascending.
    let mut rows: Vec<Vec<(u32, f64)>> = vec![Vec::new(); chunks];
    let mut sparse = false;
    let mut moved = 0.0;
    for it in 0..p.iterations {
        let all = !sparse;
        {
            let (hh, wt, sc, s): (&[f64], &[bool], &[f64], u32) = (h, &wet, &scale, stamp);
            rows.par_iter_mut().enumerate().for_each(|(q, out)| {
                out.clear();
                for c in span(q) {
                    if all || near2[c].load(Relaxed) == s {
                        let dz = change(c, hh, wt, sc);
                        if dz != 0.0 {
                            out.push((c as u32, dz));
                        }
                    }
                }
            });
        }
        let count: usize = rows.iter().map(Vec::len).sum();
        if count == 0 {
            break;
        }
        sparse = count <= dense_above;
        let last: &[Vec<(u32, f64)>] = &rows;
        // The metres moved: each chunk's movers in ascending order, the chunks in order (the
        // sum over every cell of a chunk, whose other cells add +0).
        let parts: Vec<f64> = last
            .par_iter()
            .map(|row| {
                row.iter()
                    .map(|&(c, dz)| dz.abs() * a[c as usize] / mean * 0.5)
                    .sum()
            })
            .collect();
        moved += parts.iter().sum::<f64>();
        h.par_chunks_mut(CHUNK)
            .zip(avail.par_chunks_mut(CHUNK))
            .zip(wet.par_chunks_mut(CHUNK))
            .zip(last.par_iter())
            .enumerate()
            .for_each(|(q, (((hr, ar), wr), row))| {
                for &(c, dz) in row {
                    let d = c as usize - q * CHUNK;
                    hr[d] += dz;
                    ar[d] = (ar[d] + dz).max(0.0);
                    wr[d] = sea[c as usize] && hr[d] < sea_level;
                }
            });
        if it + 1 < p.iterations {
            if sparse {
                stamp += 1;
                let s = stamp;
                last.par_iter().for_each(|row| {
                    for &(c, _) in row {
                        let c = c as usize;
                        near1[c].store(s, Relaxed);
                        for &m in t.slots(c) {
                            if m != NO_CELL {
                                near1[m as usize].store(s, Relaxed);
                            }
                        }
                    }
                });
                (0..chunks).into_par_iter().for_each(|q| {
                    for c in span(q) {
                        if near1[c].load(Relaxed) == s {
                            near2[c].store(s, Relaxed);
                            for &m in t.slots(c) {
                                if m != NO_CELL {
                                    near2[m as usize].store(s, Relaxed);
                                }
                            }
                        }
                    }
                });
            }
            shares(&mut scale, !sparse, stamp, &*h, &wet, &*avail);
        }
    }
    moved
}

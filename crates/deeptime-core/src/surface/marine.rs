// SPDX-License-Identifier: LGPL-3.0-or-later
//! Marine sediment: delivery at the river mouths and depth-dependent diffusion.
//!
//! # Diffusivity (Landlab `SimpleSubmarineDiffuser`, MIT)
//! Ported from Landlab's `SimpleSubmarineDiffuser` (https://github.com/landlab/landlab,
//! `src/landlab/components/marine_sediment_transport/simple_submarine_diffuser.py`, MIT; the
//! equations, not the code): D(z) = D₀·f₁·f₂ with water depth z = sea level − h,
//! f₁ = (tanh(z / tidal range) + 1) / 2 (a smooth shoreline; a step without tidal range) and
//! f₂ = exp(−(z − wave base) / wave base) below the wave base (1 above it). Land takes the
//! hillslope diffusivity instead of Landlab's tiny value, since `diffusion.rs` solves land and sea
//! together. Landlab's defaults are D₀ = 100 m²/yr, wave base 60 m, tidal range 2 m.
//!
//! # Delivery at the mouths (Deep Time's rule)
//! Rivers deliver their load (`spl.rs`) to the sea cell they drain into. Dumping it there would
//! make towers on steep (active) margins, where the diffusivity below the wave base is nil, so each
//! mouth first **fills accommodation** outwards: a priority queue over sea cells from the mouth,
//! shallowest first, raises each cell to a clinoform fill level — `fill_depth_m` below sea level
//! at the mouth, deepening by `shelf_slope` with distance — until the load is spent. Deltas and
//! shelves prograde from the mouths this way; the diffusion then smooths them. Mass is exact: a
//! load that finds no more room (a full closed sea) spreads evenly over the cells it reached.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::grid::Dims;

/// Landlab's `SimpleSubmarineDiffuser` diffusivity at water depth `z` (m, positive below sea
/// level), m²/yr; `land` is returned on land.
pub fn submarine_kd(z: f64, d0: f64, wave_base: f64, tidal_range: f64, land: f64) -> f64 {
    let f1 = if tidal_range > 0.0 {
        ((z / tidal_range).tanh() + 1.0) / 2.0
    } else if z >= 0.0 {
        1.0
    } else {
        0.0
    };
    let f2 = if z > wave_base && wave_base > 0.0 {
        (-(z - wave_base) / wave_base).exp()
    } else {
        1.0
    };
    // Across the shoreline blend towards the land value as f₁ falls.
    d0 * f1 * f2 + land * (1.0 - f1)
}

#[derive(Clone, Copy, PartialEq)]
struct Item {
    z: f64,
    c: u32,
    dist: f32,
}

impl Eq for Item {}

impl Ord for Item {
    fn cmp(&self, o: &Self) -> Ordering {
        o.z.total_cmp(&self.z).then_with(|| o.c.cmp(&self.c))
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// Scratch for [`fill`], reused between mouths and steps.
#[derive(Default)]
pub struct FillScratch {
    stamp: Vec<u32>,
    generation: u32,
    heap: BinaryHeap<Item>,
    /// The cells the last [`fill`] raised.
    pub touched: Vec<u32>,
    visited: Vec<u32>,
}

/// Parameters of the mouth fill.
#[derive(Clone, Copy, Debug)]
pub struct FillParams {
    pub sea_level: f64,
    pub fill_depth_m: f64,
    pub shelf_slope: f64,
    /// Cells a mouth may visit before it dumps the rest where it stands.
    pub max_cells: usize,
}

/// Spreads `volume` m³ from the mouth `mouth` over sea cells (`sea[c]`), raising `h` and adding
/// the thickness to `dep`. Returns the number of cells that received sediment.
#[allow(clippy::too_many_arguments)]
pub fn fill(
    d: &Dims,
    sea: &[bool],
    mouth: usize,
    volume: f64,
    p: &FillParams,
    h: &mut [f64],
    dep: &mut [f32],
    s: &mut FillScratch,
) -> usize {
    let a = d.cell_area();
    if s.stamp.len() != d.cells() {
        s.stamp = vec![0; d.cells()];
        s.generation = 0;
    }
    s.generation = s.generation.wrapping_add(1);
    if s.generation == 0 {
        s.stamp.iter_mut().for_each(|x| *x = 0);
        s.generation = 1;
    }
    let stamp_id = s.generation;
    s.heap.clear();
    s.touched.clear();
    s.visited.clear();
    s.heap.push(Item {
        z: h[mouth],
        c: mouth as u32,
        dist: 0.0,
    });
    s.stamp[mouth] = stamp_id;
    let mut left = volume;
    let mut touched = 0;
    let mut visited = 0;
    while left > 0.0 {
        let Some(it) = s.heap.pop() else { break };
        let c = it.c as usize;
        visited += 1;
        s.visited.push(c as u32);
        let level = p.sea_level - p.fill_depth_m - p.shelf_slope * f64::from(it.dist);
        if h[c] < level {
            let take = ((level - h[c]) * a).min(left);
            let dz = take / a;
            h[c] += dz;
            dep[c] += dz as f32;
            left -= take;
            touched += 1;
            s.touched.push(c as u32);
        }
        if left <= 0.0 || visited >= p.max_cells {
            break;
        }
        for (m, dl) in d.neighbours(c) {
            if sea[m] && s.stamp[m] != stamp_id {
                s.stamp[m] = stamp_id;
                s.heap.push(Item {
                    z: h[m],
                    c: m as u32,
                    dist: it.dist + dl as f32,
                });
            }
        }
    }
    if left > 0.0 {
        // A closed sea is full (or the mouth ran out of cells): the rest spreads evenly over
        // what it reached, so a load never makes a tower.
        let cells = s.visited.len().max(1) as f64;
        let dz = left / (cells * a);
        for &c in &s.visited {
            let c = c as usize;
            h[c] += dz;
            dep[c] += dz as f32;
        }
        // (A cell may now appear twice in `touched`; its users are idempotent.)
        touched += s.visited.len();
        s.touched.extend_from_slice(&s.visited);
    }
    touched
}

/// Share of a pair's excess the slope failure moves per iteration, and its iterations per LEM
/// step: numerical, not physical (a slope relaxes over a few steps; the result is the critical
/// slope either way).
pub const FAILURE_RATE: f64 = 0.05; // ledger: ERO-169
pub const FAILURE_ITERATIONS: u32 = 40; // ledger: ERO-169

/// Counters of the slope failure's work since the process started (P1-17 diagnostics, printed by
/// `deeptime bench`; not part of any world): full sweeps, neighbourhood passes, the movers they
/// started from, and the cells within one cell of those movers.
static FAILURE_COUNTS: [std::sync::atomic::AtomicU64; 4] = [
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
];

/// The slope failure's counters: (full sweeps, neighbourhood passes, movers summed over the
/// neighbourhood passes, cells within one cell of them).
pub fn failure_counts() -> [u64; 4] {
    std::array::from_fn(|k| FAILURE_COUNTS[k].load(std::sync::atomic::Ordering::Relaxed))
}

fn count_failure(k: usize, v: u64) {
    FAILURE_COUNTS[k].fetch_add(v, std::sync::atomic::Ordering::Relaxed);
}

/// Parameters of the submarine slope failure ([`slope_failure`]).
#[derive(Clone, Copy, Debug)]
pub struct FailureParams {
    /// The steepest gradient sediment holds under the sea (tan of `surface.submarine_slope_deg`).
    pub critical_slope: f64,
    /// Share of a pair's excess over the critical drop moved per iteration (≤ 1/16 is stable).
    pub rate: f64,
    pub iterations: u32,
}

/// Submarine slope failure (margins and spires task, 2026-10-03): sediment on the sea floor
/// steeper than `critical_slope` slides down it (slumps, debris flows, turbidity currents), so a
/// prograding shelf ends in a foreset at that angle and the material builds the slope's toe and
/// the continental rise, instead of standing as the cliff the mouth fill leaves (it raises every
/// cell it reaches to the fill level) and the marine diffusion, which vanishes below the wave
/// base, never relaxes.
///
/// Only pairs of open-sea cells (`sea[c]`, below `sea_level`) take part, and a cell gives at most
/// `avail[c]` metres in all (its loose sediment and sedimentary log: the crystalline margin the
/// plate model built does not slump). The pass is a Jacobi gather like the amplifier's thermal
/// pass (`thermal.rs`): a pair (a, b) with h_a − h_b > S_c·d moves rate·(h_a − h_b − S_c·d) from a
/// to b, scaled down for a giver that would exceed what it has; both ends compute the same
/// amount, so **matter is conserved** to round-off. Returns the metres moved (Σ|Δh|/2 · cells).
///
/// Cost (P1-17): only the first iteration visits every cell. After it, a cell's share can change
/// only within one cell of a cell that moved, and its Δh only within two, so each later
/// iteration recomputes just that neighbourhood of the last iteration's movers (or every cell
/// again when the movers are many). The neighbourhoods are marked on the raster and the passes
/// scan it row by row in parallel, which keeps the cells in ascending order without sorting;
/// cells away from the raster's edges reach their neighbours by index steps, and whether a cell
/// takes part is kept per cell. Each value is computed by the same expression from the same
/// inputs and the movers are applied in ascending cell order, so the result is bit for bit what
/// the full sweep gives (`slope_failure_reference` in the tests keeps the full sweep).
pub fn slope_failure(
    d: &Dims,
    sea: &[bool],
    sea_level: f64,
    p: &FailureParams,
    h: &mut [f64],
    avail: &mut [f64],
) -> f64 {
    use super::grid::OFFSETS;
    use rayon::prelude::*;
    let (w, hgt) = (d.w, d.h);
    let n = d.cells();
    if n == 0 {
        return 0.0;
    }
    // Whether a cell takes part, kept up to date as the cells move (P1-17: the test was taken
    // nine times per cell and pass before).
    let mut wet: Vec<bool> = (0..n)
        .into_par_iter()
        .map(|c| sea[c] && h[c] < sea_level)
        .collect();
    // S_c·d for each neighbour offset, and the offset as an index step for cells away from the
    // raster's edges (the same neighbours `Dims::neighbour` gives them).
    let crit: [f64; 8] = std::array::from_fn(|k| p.critical_slope * d.step_len(k));
    let step: [isize; 8] =
        std::array::from_fn(|k| OFFSETS[k].1 as isize * w as isize + OFFSETS[k].0 as isize);
    let inner = |i: usize, j: usize| i >= 1 && i + 1 < w && j >= 1 && j + 1 < hgt;
    let nbr = |c: usize, inner: bool, k: usize| -> Option<usize> {
        if inner {
            Some((c as isize + step[k]) as usize)
        } else {
            d.neighbour(c, k)
        }
    };
    // What each cell would give, and the share of it its sediment allows.
    let share = |c: usize, inner: bool, hh: &[f64], wet: &[bool], avail: &[f64]| -> f64 {
        if !wet[c] || avail[c] <= 0.0 {
            return 0.0;
        }
        let mut give = 0.0;
        for (k, &ck) in crit.iter().enumerate() {
            if let Some(m) = nbr(c, inner, k)
                && wet[m]
            {
                give += p.rate * (hh[c] - hh[m] - ck).max(0.0);
            }
        }
        if give > 0.0 {
            (avail[c] / give).min(1.0)
        } else {
            0.0
        }
    };
    let change = |c: usize, inner: bool, hh: &[f64], wet: &[bool], sc: &[f64]| -> f64 {
        let mut dz = 0.0;
        if wet[c] {
            for (k, &ck) in crit.iter().enumerate() {
                let Some(m) = nbr(c, inner, k) else {
                    continue;
                };
                if !wet[m] {
                    continue;
                }
                let out = (hh[c] - hh[m] - ck).max(0.0);
                let inn = (hh[m] - hh[c] - ck).max(0.0);
                dz += p.rate * (inn * sc[m] - out * sc[c]);
            }
        }
        dz
    };
    // Above this many movers every cell is recomputed; below it only their neighbourhoods are
    // (either gives the same values: cells farther away have the inputs they had).
    let dense_above = n / 8;
    let mut scale = vec![0.0f64; n];
    let mut moved = 0.0;
    // The cells that moved in the last iteration (ascending), or None for "all of them".
    let mut movers: Option<Vec<u32>> = None;
    // Neighbourhood marks: `near1[c] == stamp` within one cell of a mover, `near2` within two.
    let mut near1 = vec![0u32; n];
    let mut near2 = vec![0u32; n];
    let mut stamp = 0u32;
    let mut ring = Vec::new();
    // Each row's moved cells and their changes, ascending.
    let mut rows: Vec<Vec<(u32, f64)>> = vec![Vec::new(); hgt];
    for _ in 0..p.iterations {
        let sparse = movers.as_deref().filter(|m| m.len() <= dense_above);
        if let Some(from) = sparse {
            stamp += 1;
            ring.clear();
            for &c in from {
                let c = c as usize;
                let i = inner(c % w, c / w);
                for m in std::iter::once(Some(c))
                    .chain((0..8).map(|k| nbr(c, i, k)))
                    .flatten()
                {
                    if near1[m] != stamp {
                        near1[m] = stamp;
                        ring.push(m);
                    }
                }
            }
            for &c in &ring {
                let i = inner(c % w, c / w);
                for m in std::iter::once(Some(c))
                    .chain((0..8).map(|k| nbr(c, i, k)))
                    .flatten()
                {
                    near2[m] = stamp;
                }
            }
            count_failure(1, 1);
            count_failure(2, from.len() as u64);
            count_failure(3, ring.len() as u64);
        } else {
            count_failure(0, 1);
        }
        let all = sparse.is_none();
        {
            let hh: &[f64] = h;
            let av: &[f64] = avail;
            let wt: &[bool] = &wet;
            let (n1, s) = (&near1, stamp);
            scale.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
                for (i, x) in row.iter_mut().enumerate() {
                    let c = j * w + i;
                    if all || n1[c] == s {
                        *x = share(c, inner(i, j), hh, wt, av);
                    }
                }
            });
            let sc: &[f64] = &scale;
            let n2: &[u32] = &near2;
            rows.par_iter_mut().enumerate().for_each(|(j, out)| {
                out.clear();
                for i in 0..w {
                    let c = j * w + i;
                    if all || n2[c] == s {
                        let dz = change(c, inner(i, j), hh, wt, sc);
                        if dz != 0.0 {
                            out.push((c as u32, dz));
                        }
                    }
                }
            });
        }
        let mut moved_now = Vec::new();
        for row in &rows {
            for &(c, dz) in row {
                let c = c as usize;
                h[c] += dz;
                avail[c] = (avail[c] + dz).max(0.0);
                wet[c] = sea[c] && h[c] < sea_level;
                moved += dz.abs() * 0.5;
                moved_now.push(c as u32);
            }
        }
        if moved_now.is_empty() {
            break;
        }
        movers = Some(moved_now);
    }
    moved
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The slope failure as it was before P1-17's active set: every cell, every iteration.
    fn slope_failure_reference(
        d: &Dims,
        sea: &[bool],
        sea_level: f64,
        p: &FailureParams,
        h: &mut [f64],
        avail: &mut [f64],
    ) -> f64 {
        use rayon::prelude::*;
        let n = d.cells();
        let wet = |c: usize, h: &[f64]| sea[c] && h[c] < sea_level;
        let mut scale = vec![0.0f64; n];
        let mut next = vec![0.0f64; n];
        let mut moved = 0.0;
        for _ in 0..p.iterations {
            let hh: &[f64] = h;
            scale.par_iter_mut().enumerate().for_each(|(c, s)| {
                *s = 0.0;
                if !wet(c, hh) || avail[c] <= 0.0 {
                    return;
                }
                let mut give = 0.0;
                for (m, dl) in d.neighbours(c) {
                    if wet(m, hh) {
                        give += p.rate * (hh[c] - hh[m] - p.critical_slope * dl).max(0.0);
                    }
                }
                if give > 0.0 {
                    *s = (avail[c] / give).min(1.0);
                }
            });
            let sc: &[f64] = &scale;
            next.par_iter_mut().enumerate().for_each(|(c, o)| {
                let mut dz = 0.0;
                if wet(c, hh) {
                    for (m, dl) in d.neighbours(c) {
                        if !wet(m, hh) {
                            continue;
                        }
                        let out = (hh[c] - hh[m] - p.critical_slope * dl).max(0.0);
                        let inn = (hh[m] - hh[c] - p.critical_slope * dl).max(0.0);
                        dz += p.rate * (inn * sc[m] - out * sc[c]);
                    }
                }
                *o = dz;
            });
            let mut any = false;
            for c in 0..n {
                let dz = next[c];
                if dz != 0.0 {
                    any = true;
                    h[c] += dz;
                    avail[c] = (avail[c] + dz).max(0.0);
                    moved += dz.abs() * 0.5;
                }
            }
            if !any {
                break;
            }
        }
        moved
    }

    /// A sea floor with sediment mounds (mouth fills) on a crystalline floor, a coast whose
    /// cells sit within centimetres of the sea level (so cells cross it while they slump), and
    /// rough ground; `mounds` = 0 gives sediment everywhere.
    fn failure_world(
        w: usize,
        hh: usize,
        seed: u64,
        mounds: usize,
    ) -> (Vec<f64>, Vec<f64>, Vec<bool>) {
        let mut rng = crate::sphere::SplitMix64::new(seed);
        let n = w * hh;
        let mut h = vec![0.0; n];
        let mut avail = vec![0.0; n];
        for (c, z) in h.iter_mut().enumerate() {
            let i = c % w;
            *z = if i < w / 6 {
                5.0 + 50.0 * rng.next_f64()
            } else if i < w / 6 + 2 {
                (rng.next_f64() - 0.5) * 0.02 // straddles the sea level
            } else {
                -3000.0 + 400.0 * rng.next_f64()
            };
        }
        if mounds == 0 {
            for a in avail.iter_mut() {
                *a = 2000.0 * rng.next_f64();
            }
            for z in h.iter_mut() {
                if *z < 0.0 {
                    *z += 1500.0 * rng.next_f64();
                }
            }
        } else {
            for _ in 0..mounds {
                let (ci, cj) = (rng.below(w as u64) as usize, rng.below(hh as u64) as usize);
                let r = 1 + rng.below(4) as usize;
                for dj in 0..=2 * r {
                    for di in 0..=2 * r {
                        // ci + di − r, mod w (w may be smaller than r).
                        let (i, j) = ((ci + di + r * (w - 1)) % w, cj + dj);
                        if j < r || j - r >= hh {
                            continue;
                        }
                        let c = (j - r) * w + i;
                        if h[c] < -10.0 {
                            h[c] = -100.0 - 30.0 * rng.next_f64();
                            avail[c] = 3000.0 * rng.next_f64();
                        }
                    }
                }
            }
        }
        let sea: Vec<bool> = h.iter().map(|&z| z < 0.0).collect();
        (h, avail, sea)
    }

    /// P1-17: the active-set iterations give the full sweep's heights, sediment and the metres
    /// moved bit for bit, on the dense path (sediment everywhere), the sparse one (a few mounds,
    /// so few cells move after the first iteration) and grids that do not wrap or are one or two
    /// cells wide, at 1, 2, 4 and 8 threads.
    #[test]
    fn the_active_set_failure_equals_the_full_sweep_bit_for_bit() {
        let s_c = 2.5f64.to_radians().tan();
        let mut cases = 0;
        for &(w, hh, wrap, mounds) in &[
            (96usize, 64usize, true, 3usize),
            (96, 64, true, 0),
            (160, 48, false, 6),
            (1, 40, true, 1),
            (2, 40, true, 1),
            (40, 1, true, 2),
            (33, 17, true, 40),
        ] {
            for seed in 1..=3u64 {
                for &iterations in &[FAILURE_ITERATIONS, 400] {
                    let d = Dims::new(w, hh, 4800.0, wrap);
                    let (h0, a0, sea) = failure_world(w, hh, seed, mounds);
                    let p = FailureParams {
                        critical_slope: s_c,
                        rate: FAILURE_RATE,
                        iterations,
                    };
                    let (mut hr, mut ar) = (h0.clone(), a0.clone());
                    let mr = slope_failure_reference(&d, &sea, 0.0, &p, &mut hr, &mut ar);
                    for threads in [1, 2, 4, 8] {
                        let pool = rayon::ThreadPoolBuilder::new()
                            .num_threads(threads)
                            .build()
                            .unwrap();
                        let (mut h, mut a) = (h0.clone(), a0.clone());
                        let m = pool.install(|| slope_failure(&d, &sea, 0.0, &p, &mut h, &mut a));
                        let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
                        assert_eq!(bits(&h), bits(&hr), "h {w}x{hh} seed {seed} t{threads}");
                        assert_eq!(bits(&a), bits(&ar), "avail {w}x{hh} seed {seed}");
                        assert_eq!(m.to_bits(), mr.to_bits(), "moved {w}x{hh} seed {seed}");
                    }
                    if mr > 0.0 {
                        cases += 1;
                    }
                }
            }
        }
        assert!(cases >= 20, "only {cases} cases moved any sediment");
    }

    /// The sparse path is the one that runs once the first sweep has done most of the work: on
    /// a floor with a few mounds, fewer than a sixteenth of the cells move in the later
    /// iterations, so the comparison above exercises it.
    #[test]
    fn few_cells_move_after_the_first_failure_sweep() {
        let (w, hh) = (96, 64);
        let d = Dims::new(w, hh, 4800.0, true);
        let (mut h, mut avail, sea) = failure_world(w, hh, 2, 3);
        let p = FailureParams {
            critical_slope: 2.5f64.to_radians().tan(),
            rate: FAILURE_RATE,
            iterations: 1,
        };
        let before = h.clone();
        slope_failure(&d, &sea, 0.0, &p, &mut h, &mut avail);
        let movers = h.iter().zip(&before).filter(|(a, b)| a != b).count();
        assert!(movers > 0 && movers <= w * hh / 16, "{movers} movers");
    }

    /// A sediment cliff (a mouth fill's front) under the sea relaxes to the critical slope with
    /// its mass kept; a crystalline step (no sediment) and land do not move.
    #[test]
    fn a_submarine_sediment_cliff_relaxes_to_the_critical_slope() {
        let (w, hh) = (64, 3);
        let d = Dims::new(w, hh, 1000.0, false);
        let mut h = vec![0.0; w * hh];
        let mut avail = vec![0.0; w * hh];
        for j in 0..hh {
            for i in 0..w {
                let c = j * w + i;
                h[c] = if i < 4 {
                    50.0 // land
                } else if i < 20 {
                    -100.0 // the fill's shelf, on 3 km of sediment
                } else {
                    -3000.0
                };
                avail[c] = if (4..20).contains(&i) { 3000.0 } else { 0.0 };
            }
        }
        let sea: Vec<bool> = h.iter().map(|&z| z < 0.0).collect();
        let s_c = 2.5f64.to_radians().tan();
        let p = FailureParams {
            critical_slope: s_c,
            rate: 0.05,
            iterations: 30_000,
        };
        let before: f64 = h.iter().sum();
        slope_failure(&d, &sea, 0.0, &p, &mut h, &mut avail);
        let after: f64 = h.iter().sum();
        assert!((before - after).abs() < 1e-6 * before.abs(), "mass kept");
        let row = &h[w..2 * w];
        assert_eq!(row[0], 50.0, "land does not move");
        // No pair of sea cells is left much steeper than the critical slope.
        let steepest = (4..w - 1)
            .map(|i| (row[i] - row[i + 1]).abs() / 1000.0)
            .fold(0.0, f64::max);
        assert!(steepest < 1.2 * s_c, "steepest {steepest} against {s_c}");
        // The foot is built out: the floor beyond the old cliff was raised.
        assert!(row[24] > -2900.0, "{}", row[24]);
        // A step without sediment stays a step.
        let mut h2 = vec![-100.0; w * hh];
        for (c, z) in h2.iter_mut().enumerate() {
            if c % w >= 20 {
                *z = -3000.0;
            }
        }
        let keep = h2.clone();
        let mut none = vec![0.0; w * hh];
        let sea2 = vec![true; w * hh];
        slope_failure(&d, &sea2, 0.0, &p, &mut h2, &mut none);
        assert_eq!(h2, keep);
    }

    #[test]
    fn kd_falls_below_the_wave_base_and_on_land() {
        let deep = submarine_kd(1000.0, 100.0, 60.0, 2.0, 1.0);
        let shallow = submarine_kd(30.0, 100.0, 60.0, 2.0, 1.0);
        let land = submarine_kd(-50.0, 100.0, 60.0, 2.0, 1.0);
        assert!((shallow - 100.0).abs() < 1e-6);
        assert!(deep < 1e-3 && deep > 0.0);
        assert!((land - 1.0).abs() < 1e-6);
    }

    #[test]
    fn a_mouth_fill_spends_exactly_its_volume() {
        let d = Dims::new(20, 20, 1000.0, true);
        let mut h = vec![-500.0; 400];
        let sea = vec![true; 400];
        let mut dep = vec![0.0f32; 400];
        let p = FillParams {
            sea_level: 0.0,
            fill_depth_m: 20.0,
            shelf_slope: 0.001,
            max_cells: 10_000,
        };
        let before: f64 = h.iter().sum();
        let v = 3.3e9;
        let mut s = FillScratch::default();
        let n = fill(&d, &sea, 210, v, &p, &mut h, &mut dep, &mut s);
        let after: f64 = h.iter().sum();
        assert!(n >= 7);
        assert!(
            ((after - before) * 1.0e6 - v).abs() < 1.0,
            "{} vs {v}",
            (after - before) * 1e6
        );
        assert!(h[210] <= -20.0 + 1e-9);
    }
}

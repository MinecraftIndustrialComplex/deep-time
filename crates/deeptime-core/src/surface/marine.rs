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
//! # Delivery at the mouths (ledger: ERO-050)
//! Rivers deliver their load (`spl.rs`) to the sea cell they drain into. Dumping it there would
//! make towers on steep (active) margins, where the diffusivity below the wave base is nil, so each
//! mouth first **fills accommodation** outwards: a priority queue over sea cells from the mouth
//! raises each cell to a clinoform fill level — `fill_depth_m` below sea level at the mouth,
//! deepening by `shelf_slope` with distance — until the load is spent. Mass is exact: a load that
//! finds no more room (a full closed sea) spreads evenly over the cells it reached.
//!
//! The queue's order: the load goes down from the mouth, nearest room first, and climbs over a
//! rise only when there is no more room below it. Each cell's key is the highest bed on the fill's
//! way to it (its spill, as in the Priority-Flood), lowest first; among equal spills, so all the
//! ground the load reaches going down, the nearest first, then the deepest. So a delta and its
//! shelf build out from the mouth (the foreset beside the mouth before the deep floor beyond it),
//! a hollow the mouth sits in fills from its floor, and the diffusion then smooths them. This is
//! the order of goSPL's marine deposition (its cells highest bed first, each filled to the
//! clinoform height and the rest passed to its lower receivers) and of FastScape's marine scheme
//! (the shelf filled from the shore downstream, the overflow passed on along the flow); the rule
//! is cited, no code of theirs is ported (both GPL). Until 2026-10-28 the queue popped the deepest
//! cell it had reached first, against the "shallowest first" this note gave: a mouth's load ran
//! down the slope and filled the deepest room within reach before the room beside the mouth.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::grid::{Dims, SurfaceGrid};

#[cfg(feature = "gpu")]
mod gpu;

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

/// A sea cell the fill has reached: `spill` is the highest bed on the fill's way to it from the
/// mouth (the cell's own bed or the way's), `z` its own bed, `dist` the way's length.
#[derive(Clone, Copy, PartialEq)]
struct Item {
    spill: f64,
    dist: f32,
    z: f64,
    c: u32,
}

impl Eq for Item {}

impl Ord for Item {
    /// The heap pops its greatest: the lowest spill, then the nearest, then the deepest bed, then
    /// the lower cell (ERO-050).
    fn cmp(&self, o: &Self) -> Ordering {
        o.spill
            .total_cmp(&self.spill)
            .then_with(|| o.dist.total_cmp(&self.dist))
            .then_with(|| o.z.total_cmp(&self.z))
            .then_with(|| o.c.cmp(&self.c))
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// Counters of the mouth fills since the process started (P1-17 diagnostics, printed by
/// `deeptime bench`; not part of any world): fills planned (a mouth planned again counts twice),
/// cells popped, cells raised, the most cells one fill popped, and the fills [`fill_all`] planned
/// again in turn.
static FILL_COUNTS: [std::sync::atomic::AtomicU64; 5] = [
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
];

/// The mouth fills' counters: (fills, cells popped, cells raised, the most cells one fill popped,
/// fills planned again in turn).
pub fn fill_counts() -> [u64; 5] {
    std::array::from_fn(|k| FILL_COUNTS[k].load(std::sync::atomic::Ordering::Relaxed))
}

/// The cells a fill has put on its heap: an open-addressing set as small as the fill, emptied
/// through the slots it filled (a grid-sized stamp array for each of the pool's threads would cost
/// tens of megabytes apiece).
#[derive(Default)]
struct Seen {
    slots: Vec<u32>,
    filled: Vec<u32>,
    shift: u32,
}

impl Seen {
    const EMPTY: u32 = u32::MAX;

    fn clear(&mut self) {
        for &k in &self.filled {
            self.slots[k as usize] = Self::EMPTY;
        }
        self.filled.clear();
    }

    /// Adds `c`; false when it was there.
    fn insert(&mut self, c: u32) -> bool {
        if 2 * (self.filled.len() + 1) > self.slots.len() {
            self.grow();
        }
        let mask = self.slots.len() - 1;
        let mut k = (u64::from(c).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> self.shift) as usize;
        loop {
            match self.slots[k] {
                Self::EMPTY => {
                    self.slots[k] = c;
                    self.filled.push(k as u32);
                    return true;
                }
                x if x == c => return false,
                _ => k = (k + 1) & mask,
            }
        }
    }

    fn grow(&mut self) {
        let held: Vec<u32> = self
            .filled
            .iter()
            .map(|&k| self.slots[k as usize])
            .collect();
        let size = (self.slots.len() * 2).max(256);
        self.slots = vec![Self::EMPTY; size];
        self.shift = 64 - size.trailing_zeros();
        self.filled.clear();
        for c in held {
            self.insert(c);
        }
    }
}

/// One fill's scratch and its plan: the raises it makes, (cell, metres) in order, and the cells it
/// popped (whose heights it read before raising any).
#[derive(Default)]
struct Planner {
    seen: Seen,
    heap: BinaryHeap<Item>,
    visited: Vec<u32>,
    writes: Vec<(u32, f64)>,
}

thread_local! {
    /// [`fill_all`]'s planner on each thread of the pool.
    static PLANNER: std::cell::RefCell<Planner> = std::cell::RefCell::new(Planner::default());
}

/// Scratch for [`fill`] and [`fill_all`], reused between mouths and steps.
#[derive(Default)]
pub struct FillScratch {
    planner: Planner,
    /// The cells the last [`fill`] raised (a cell twice when the rest was spread over it).
    pub touched: Vec<u32>,
    /// [`fill_all`]'s cells raised since its batch's plans read the heights (`== written_gen`).
    written: Vec<u32>,
    written_gen: u32,
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
pub fn fill<G: SurfaceGrid>(
    d: &G,
    sea: &[bool],
    mouth: usize,
    volume: f64,
    p: &FillParams,
    h: &mut [f64],
    dep: &mut [f32],
    s: &mut FillScratch,
) -> usize {
    plan(d, sea, mouth, volume, p, h, &mut s.planner);
    apply(&s.planner.writes, h, dep, &mut s.touched);
    s.touched.len()
}

/// [`fill`]'s raises on the heights `h` as they stand, into `w.writes`, without making them: a
/// cell's height is read when it goes on the heap and when it is popped, and a cell is raised only
/// when it is popped, after both reads, so the plan raises what the fill would.
fn plan<G: SurfaceGrid>(
    d: &G,
    sea: &[bool],
    mouth: usize,
    volume: f64,
    p: &FillParams,
    h: &[f64],
    w: &mut Planner,
) {
    w.seen.clear();
    w.heap.clear();
    w.visited.clear();
    w.writes.clear();
    w.heap.push(Item {
        spill: h[mouth],
        dist: 0.0,
        z: h[mouth],
        c: mouth as u32,
    });
    w.seen.insert(mouth as u32);
    let mut left = volume;
    let mut raised = 0;
    while left > 0.0 {
        let Some(it) = w.heap.pop() else { break };
        let c = it.c as usize;
        w.visited.push(c as u32);
        let level = p.sea_level - p.fill_depth_m - p.shelf_slope * f64::from(it.dist);
        if h[c] < level {
            let a = d.area(c);
            let take = ((level - h[c]) * a).min(left);
            w.writes.push((c as u32, take / a));
            left -= take;
            raised += 1;
        }
        if left <= 0.0 || w.visited.len() >= p.max_cells {
            break;
        }
        for (m, dl) in d.neighbours(c) {
            if sea[m] && w.seen.insert(m as u32) {
                w.heap.push(Item {
                    spill: h[m].max(it.spill),
                    dist: it.dist + dl as f32,
                    z: h[m],
                    c: m as u32,
                });
            }
        }
    }
    {
        use std::sync::atomic::Ordering::Relaxed;
        let popped = w.visited.len() as u64;
        FILL_COUNTS[0].fetch_add(1, Relaxed);
        FILL_COUNTS[1].fetch_add(popped, Relaxed);
        FILL_COUNTS[2].fetch_add(raised, Relaxed);
        FILL_COUNTS[3].fetch_max(popped, Relaxed);
    }
    if left > 0.0 {
        // A closed sea is full (or the mouth ran out of cells): the rest spreads evenly over
        // what it reached, so a load never makes a tower.
        let dz = match d.uniform_area() {
            Some(a) => {
                let cells = w.visited.len().max(1) as f64;
                left / (cells * a)
            }
            None => {
                // ledger: SCL-151 (game-choice: the rest over the visited area)
                let area: f64 = w.visited.iter().map(|&c| d.area(c as usize)).sum();
                left / area.max(f64::MIN_POSITIVE)
            }
        };
        w.writes.extend(w.visited.iter().map(|&c| (c, dz)));
    }
}

/// Raises `h` and `dep` by a plan's `writes` in their order, and lists the cells in `touched`.
fn apply(writes: &[(u32, f64)], h: &mut [f64], dep: &mut [f32], touched: &mut Vec<u32>) {
    touched.clear();
    for &(c, dz) in writes {
        let cu = c as usize;
        h[cu] += dz;
        dep[cu] += dz as f32;
        touched.push(c);
    }
}

/// The fills of `mouths`, (cell, m³), one after another as [`fill`] makes them, bit for bit;
/// `after(k, touched)` runs after mouth `k`'s with the cells it raised (the speed work of
/// 2026-10-28, for the shallow-first fill's longer walks).
///
/// A batch of mouths is planned in parallel on the heights as they stand, and the plans are
/// applied in order. A plan holds if no earlier mouth of its batch raised a cell it popped: the
/// cells it only put on its heap do not matter, since raising a cell only puts it later in the
/// heap's order (a higher spill, or the same spill and a higher bed), so a cell that was not
/// popped is still not popped. A plan that does not hold is made again on the heights as they are
/// then. Returns how many mouths were planned again.
#[allow(clippy::too_many_arguments)]
pub fn fill_all<G: SurfaceGrid>(
    d: &G,
    sea: &[bool],
    mouths: &[(usize, f64)],
    p: &FillParams,
    h: &mut [f64],
    dep: &mut [f32],
    s: &mut FillScratch,
    mut after: impl FnMut(usize, &[u32]),
) -> u64 {
    use rayon::prelude::*;
    let n = d.cells();
    if s.written.len() != n {
        s.written = vec![0; n];
        s.written_gen = 0;
    }
    // Enough mouths to keep the pool busy, few enough that a plan seldom meets a raise.
    let batch_size = 32 * rayon::current_num_threads().max(1);
    let mut again = 0u64;
    for (b, batch) in mouths.chunks(batch_size).enumerate() {
        let plans: Vec<_> = {
            let heights = &*h;
            batch
                .par_iter()
                .map(|&(m, vol)| {
                    PLANNER.with(|w| {
                        let mut w = w.borrow_mut();
                        plan(d, sea, m, vol, p, heights, &mut w);
                        (w.visited.clone(), w.writes.clone())
                    })
                })
                .collect()
        };
        s.written_gen = s.written_gen.wrapping_add(1);
        if s.written_gen == 0 {
            s.written.iter_mut().for_each(|x| *x = 0);
            s.written_gen = 1;
        }
        let mark = s.written_gen;
        for (k, (visited, writes)) in plans.into_iter().enumerate() {
            let holds = visited.iter().all(|&c| s.written[c as usize] != mark);
            let writes = if holds {
                writes
            } else {
                again += 1;
                let (m, vol) = batch[k];
                plan(d, sea, m, vol, p, h, &mut s.planner);
                std::mem::take(&mut s.planner.writes)
            };
            apply(&writes, h, dep, &mut s.touched);
            for &(c, _) in &writes {
                s.written[c as usize] = mark;
            }
            after(b * batch_size + k, &s.touched);
        }
    }
    FILL_COUNTS[4].fetch_add(again, std::sync::atomic::Ordering::Relaxed);
    again
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
/// again when the movers are many). The neighbourhoods are marked on the raster, each row by its
/// own task from the movers of the rows around it (P1-19), and the passes scan the raster row by
/// row in parallel, which keeps the cells in ascending order without sorting;
/// cells away from the raster's edges reach their neighbours by index steps, and whether a cell
/// takes part is kept per cell. Each value is computed by the same expression from the same
/// inputs and the movers are applied in ascending cell order, so the result is bit for bit what
/// the full sweep gives (`slope_failure_reference` in the tests keeps the full sweep).
///
/// With the `gpu` feature a session's device takes the call on a large grid (`gpu.rs` here,
/// `docs/design/gpu-compute.md` §16): every iteration over every cell in `f32`, the same rule; a
/// device that fails hands the call back to this path.
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
    #[cfg(feature = "gpu")]
    if let Some(g) = gpu::session_device(n) {
        let started = std::time::Instant::now();
        match gpu::slope_failure(&g, d, sea, sea_level, p, h, avail) {
            Ok(moved) => {
                crate::compute::lap("slope_failure_device", started);
                count_failure(0, u64::from(p.iterations));
                return moved;
            }
            Err(e) => gpu::device_failed(&e),
        }
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
    // The dense passes on a row's inner cells, i in 1..w − 1 of a row j in 1..hgt − 1, into
    // `out[i − 1]` (the speed work of 2026-10-28): each neighbour read as the row shifted by its
    // index step, and a dry neighbour's term multiplied by 0 instead of skipped. That adds ±0 to a
    // sum that is never −0, so the bits are `share`'s and `change`'s, and the row's cells go
    // through the same arithmetic side by side, which the compiler vectorizes.
    let rows_ok = w >= 3 && hgt >= 3;
    let shifted = |j: usize, k: usize| (j * w + 1).wrapping_add_signed(step[k]);
    let share_row = |j: usize, hh: &[f64], wt: &[bool], av: &[f64], out: &mut [f64]| {
        let (len, base) = (w - 2, j * w + 1);
        let hm: [&[f64]; 8] = std::array::from_fn(|k| &hh[shifted(j, k)..][..len]);
        let wm: [&[bool]; 8] = std::array::from_fn(|k| &wt[shifted(j, k)..][..len]);
        let (hc, wc, ac) = (&hh[base..][..len], &wt[base..][..len], &av[base..][..len]);
        for (i, o) in out[..len].iter_mut().enumerate() {
            let mut give = 0.0;
            for k in 0..8 {
                let wf = f64::from(u8::from(wm[k][i]));
                give += wf * (p.rate * (hc[i] - hm[k][i] - crit[k]).max(0.0));
            }
            *o = if wc[i] && ac[i] > 0.0 && give > 0.0 {
                (ac[i] / give).min(1.0)
            } else {
                0.0
            };
        }
    };
    let change_row = |j: usize, hh: &[f64], wt: &[bool], sc: &[f64], out: &mut [f64]| {
        let (len, base) = (w - 2, j * w + 1);
        let hm: [&[f64]; 8] = std::array::from_fn(|k| &hh[shifted(j, k)..][..len]);
        let wm: [&[bool]; 8] = std::array::from_fn(|k| &wt[shifted(j, k)..][..len]);
        let sm: [&[f64]; 8] = std::array::from_fn(|k| &sc[shifted(j, k)..][..len]);
        let (hc, wc, scc) = (&hh[base..][..len], &wt[base..][..len], &sc[base..][..len]);
        for (i, o) in out[..len].iter_mut().enumerate() {
            let mut dz = 0.0;
            for k in 0..8 {
                let wf = f64::from(u8::from(wm[k][i]));
                let out = (hc[i] - hm[k][i] - crit[k]).max(0.0);
                let inn = (hm[k][i] - hc[i] - crit[k]).max(0.0);
                dz += wf * (p.rate * (inn * sm[k][i] - out * scc[i]));
            }
            *o = if wc[i] { dz } else { 0.0 };
        }
    };
    // Above this many movers every cell is recomputed; below it only their neighbourhoods are
    // (either gives the same values: cells farther away have the inputs they had).
    let dense_above = n / 4;
    // Every cell's share is written by the first iteration before anything reads it.
    let mut scale = crate::alloc::filled(n, 0.0f64);
    let mut moved = 0.0;
    // Neighbourhood marks: `near1[c] == stamp` within one row and one column of a mover, `near2`
    // within two (columns wrapping as the raster does). The raster's neighbours are those eight
    // offsets, so these hold every cell within one neighbour step of a mover and every cell
    // within two (on rasters one or two cells wide, a few more; recomputing a cell whose inputs
    // did not change gives the value it has, so that changes nothing).
    let mut near1 = crate::alloc::filled(n, 0u32);
    let mut near2 = crate::alloc::filled(n, 0u32);
    let mut stamp = 0u32;
    // The column `di` cells from `i`, if the raster has it.
    let col = |i: usize, di: isize| -> Option<usize> {
        let k = i as isize + di;
        if (0..w as isize).contains(&k) {
            Some(k as usize)
        } else if d.wrap_x {
            Some(k.rem_euclid(w as isize) as usize)
        } else {
            None
        }
    };
    // Each row's moved cells and their changes, ascending.
    let mut rows: Vec<Vec<(u32, f64)>> = vec![Vec::new(); hgt];
    // The shares of every cell (`all`) or of the cells marked `near1 == s`, rows in parallel.
    let shares =
        |scale: &mut [f64], all: bool, s: u32, n1: &[u32], hh: &[f64], wt: &[bool], av: &[f64]| {
            scale.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
                if all && rows_ok && j >= 1 && j + 1 < hgt {
                    row[0] = share(j * w, false, hh, wt, av);
                    share_row(j, hh, wt, av, &mut row[1..w - 1]);
                    row[w - 1] = share(j * w + w - 1, false, hh, wt, av);
                    return;
                }
                for (i, x) in row.iter_mut().enumerate() {
                    let c = j * w + i;
                    if all || n1[c] == s {
                        *x = share(c, inner(i, j), hh, wt, av);
                    }
                }
            });
        };
    // Marks the neighbourhoods of the movers in `last` with `s`, rows in parallel (P1-19; the
    // marks were set one mover at a time before): row j takes the movers of rows j − 2 to j + 2.
    // Returns the cells within one cell of a mover.
    let mark = |near1: &mut [u32], near2: &mut [u32], s: u32, last: &[Vec<(u32, f64)>]| -> usize {
        near1
            .par_chunks_mut(w)
            .zip(near2.par_chunks_mut(w))
            .enumerate()
            .map(|(j, (n1, n2))| {
                for (jj, row) in last
                    .iter()
                    .enumerate()
                    .take((j + 3).min(hgt))
                    .skip(j.saturating_sub(2))
                {
                    let close = jj + 1 >= j && jj <= j + 1;
                    for &(c, _) in row {
                        let i = c as usize - jj * w;
                        for di in -2isize..=2 {
                            if let Some(k) = col(i, di) {
                                n2[k] = s;
                                if close && di.abs() <= 1 {
                                    n1[k] = s;
                                }
                            }
                        }
                    }
                }
                n1.iter().filter(|&&v| v == s).count()
            })
            .sum()
    };
    // The first iteration takes every cell; each later one the last one's neighbourhoods when
    // its movers were few (`sparse`), every cell again when they were many. An iteration's marks
    // and shares are made at the end of the one before it (P1-19).
    count_failure(0, 1);
    shares(&mut scale, true, stamp, &near1, &*h, &wet, &*avail);
    let mut sparse = false;
    for it in 0..p.iterations {
        let all = !sparse;
        {
            let hh: &[f64] = h;
            let wt: &[bool] = &wet;
            let sc: &[f64] = &scale;
            let (n2, s): (&[u32], u32) = (&near2, stamp);
            rows.par_iter_mut()
                .enumerate()
                .for_each_init(Vec::new, |buf, (j, out)| {
                    out.clear();
                    if all && rows_ok && j >= 1 && j + 1 < hgt {
                        buf.resize(w - 2, 0.0);
                        change_row(j, hh, wt, sc, buf);
                        let row = (0..w).map(|i| {
                            if i == 0 || i == w - 1 {
                                change(j * w + i, false, hh, wt, sc)
                            } else {
                                buf[i - 1]
                            }
                        });
                        for (i, dz) in row.enumerate() {
                            if dz != 0.0 {
                                out.push(((j * w + i) as u32, dz));
                            }
                        }
                        return;
                    }
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
        let count: usize = rows.iter().map(Vec::len).sum();
        let next = count > 0 && it + 1 < p.iterations;
        sparse = count <= dense_above;
        // The metres moved, summed in ascending cell order as before (P1-17) on this thread,
        // beside the movers' changes (rows in parallel, each row's cells its own) and the next
        // iteration's marks and shares, which the sum waited for before (P1-19). The marks need
        // the movers' list only when the next iteration takes the sparse path.
        let last: &[Vec<(u32, f64)>] = &rows;
        rayon::join(
            || {
                for row in last {
                    for &(_, dz) in row {
                        moved += dz.abs() * 0.5;
                    }
                }
            },
            || {
                h.par_chunks_mut(w)
                    .zip(avail.par_chunks_mut(w))
                    .zip(wet.par_chunks_mut(w))
                    .zip(sea.par_chunks(w))
                    .zip(last.par_iter())
                    .enumerate()
                    .for_each(|(j, ((((hr, ar), wr), sr), row))| {
                        for &(c, dz) in row {
                            let i = c as usize - j * w;
                            hr[i] += dz;
                            ar[i] = (ar[i] + dz).max(0.0);
                            wr[i] = sr[i] && hr[i] < sea_level;
                        }
                    });
                if next {
                    if sparse {
                        stamp += 1;
                        let near = mark(&mut near1, &mut near2, stamp, last);
                        count_failure(1, 1);
                        count_failure(2, count as u64);
                        count_failure(3, near as u64);
                    } else {
                        count_failure(0, 1);
                    }
                    shares(&mut scale, !sparse, stamp, &near1, &*h, &wet, &*avail);
                }
            },
        );
        if count == 0 {
            break;
        }
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

    /// The fill as it was written before the plans (2026-10-28): one heap walk that raises as it
    /// goes, a grid-sized stamp for the cells it has seen. The reference for [`fill`] and
    /// [`fill_all`].
    #[allow(clippy::too_many_arguments)]
    fn fill_reference(
        d: &Dims,
        sea: &[bool],
        mouth: usize,
        volume: f64,
        p: &FillParams,
        h: &mut [f64],
        dep: &mut [f32],
        touched: &mut Vec<u32>,
    ) {
        let mut seen = vec![false; d.cells()];
        let mut heap = BinaryHeap::new();
        let mut visited = Vec::new();
        touched.clear();
        heap.push(Item {
            spill: h[mouth],
            dist: 0.0,
            z: h[mouth],
            c: mouth as u32,
        });
        seen[mouth] = true;
        let mut left = volume;
        while left > 0.0 {
            let Some(it) = heap.pop() else { break };
            let c = it.c as usize;
            visited.push(c as u32);
            let level = p.sea_level - p.fill_depth_m - p.shelf_slope * f64::from(it.dist);
            if h[c] < level {
                let a = d.area(c);
                let take = ((level - h[c]) * a).min(left);
                let dz = take / a;
                h[c] += dz;
                dep[c] += dz as f32;
                left -= take;
                touched.push(c as u32);
            }
            if left <= 0.0 || visited.len() >= p.max_cells {
                break;
            }
            for (m, dl) in d.neighbours(c) {
                if sea[m] && !seen[m] {
                    seen[m] = true;
                    heap.push(Item {
                        spill: h[m].max(it.spill),
                        dist: it.dist + dl as f32,
                        z: h[m],
                        c: m as u32,
                    });
                }
            }
        }
        if left > 0.0 {
            let dz = left / (visited.len().max(1) as f64 * d.cell_area());
            for &c in &visited {
                h[c as usize] += dz;
                dep[c as usize] += dz as f32;
            }
            touched.extend_from_slice(&visited);
        }
    }

    /// The mouths' fills planned in parallel give the fills in turn bit for bit (heights, deposit,
    /// and each mouth's raised cells for the provenance), on pools of 1, 3 and 8 threads: mouths
    /// along a crowded coast whose plans overlap, loads from a sliver of a cell to dozens of cells,
    /// a closed sea that fills up and a mouth that runs out of cells. [`fill`] alone gives them too.
    #[test]
    fn the_mouths_planned_in_parallel_fill_as_they_did_in_turn() {
        let (w, rows) = (96, 64);
        let d = Dims::new(w, rows, 1000.0, true);
        let n = d.cells();
        let mut rng = crate::sphere::SplitMix64::new(28);
        let mut uni = || (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        // Land in the middle columns, a closed sea 3 x 3 inside it, the ocean either side.
        let closed = |c: usize| (20..23).contains(&(c / w)) && (50..53).contains(&(c % w));
        let h0: Vec<f64> = (0..n)
            .map(|c| {
                let (i, j) = (c % w, c / w);
                let coast = 40.0 + 6.0 * (j as f64 * 0.3).sin();
                let x = i as f64 - coast;
                if closed(c) {
                    -300.0 - uni() * 50.0
                } else if (0.0..24.0).contains(&x) {
                    100.0 + uni() * 50.0
                } else {
                    -40.0 * x.abs().min((x - 24.0).abs()) - uni() * 200.0
                }
            })
            .collect();
        let sea: Vec<bool> = h0.iter().map(|&z| z < 0.0).collect();
        let mouths: Vec<(usize, f64)> = (0..n)
            .filter(|&c| sea[c] && d.neighbours(c).any(|(m, _)| !sea[m]))
            .map(|c| {
                let v = 10f64.powf(5.0 + 6.5 * uni());
                (c, if closed(c) { 1.0e10 } else { v })
            })
            .collect();
        assert!(mouths.len() > 150, "{} mouths", mouths.len());
        let p = FillParams {
            sea_level: 0.0,
            fill_depth_m: 5.0,
            shelf_slope: 0.0005,
            max_cells: 400,
        };
        let (mut h_want, mut dep_want) = (h0.clone(), vec![0.0f32; n]);
        let mut touched_want = Vec::new();
        let mut t = Vec::new();
        for &(m, vol) in &mouths {
            fill_reference(&d, &sea, m, vol, &p, &mut h_want, &mut dep_want, &mut t);
            touched_want.push(t.clone());
        }
        let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        let fbits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        // One fill after another.
        let (mut h, mut dep) = (h0.clone(), vec![0.0f32; n]);
        let mut s = FillScratch::default();
        for &(m, vol) in &mouths {
            fill(&d, &sea, m, vol, &p, &mut h, &mut dep, &mut s);
        }
        assert!(
            bits(&h) == bits(&h_want) && fbits(&dep) == fbits(&dep_want),
            "fill"
        );
        for threads in [1, 3, 8] {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            let (mut h, mut dep) = (h0.clone(), vec![0.0f32; n]);
            let mut touched = vec![Vec::new(); mouths.len()];
            let mut s = FillScratch::default();
            let again = pool.install(|| {
                fill_all(&d, &sea, &mouths, &p, &mut h, &mut dep, &mut s, |k, t| {
                    touched[k] = t.to_vec();
                })
            });
            assert!(bits(&h) == bits(&h_want), "heights, {threads} threads");
            assert!(
                fbits(&dep) == fbits(&dep_want),
                "deposit, {threads} threads"
            );
            assert_eq!(touched, touched_want, "raised cells, {threads} threads");
            assert!(again > 0, "no plan popped ground an earlier mouth raised");
            // The closed sea filled up: the rest spread over what it reached.
            assert!(h[21 * w + 51] > p.sea_level - p.fill_depth_m);
        }
    }

    /// The fill's set of seen cells holds what went in, through growth and clearing.
    #[test]
    fn the_seen_set_keeps_its_cells() {
        let mut s = Seen::default();
        for round in 0..3u32 {
            let cells: Vec<u32> = (0..5000u32).map(|k| k * 7 + round).collect();
            for &c in &cells {
                assert!(s.insert(c));
            }
            for &c in &cells {
                assert!(!s.insert(c));
            }
            s.clear();
            assert!(s.slots.iter().all(|&x| x == Seen::EMPTY));
        }
    }

    /// A mouth's load fills the room beside the mouth before the deeper floor farther out
    /// (ERO-050): the shelf at one cell before the trough at two, though the trough is deeper and
    /// the deepest-first order took it first; then outward, nearest first.
    #[test]
    fn a_mouth_fills_the_near_room_before_the_deep_floor_beyond() {
        // One row: a shelf at 0..10, the mouth at 10, one shelf cell at 11, a trough at 12..21.
        let d = Dims::new(21, 1, 1000.0, false);
        let bed = |i: usize| match i {
            0..10 => -20.0,
            10 => -1.0,
            11 => -30.0,
            _ => -2000.0,
        };
        let mut h: Vec<f64> = (0..21).map(bed).collect();
        let sea = vec![true; 21];
        let mut dep = vec![0.0f32; 21];
        let p = FillParams {
            sea_level: 0.0,
            fill_depth_m: 0.0,
            shelf_slope: 0.0005,
            max_cells: 10_000,
        };
        let room = |i: usize, cells: f64| {
            let level = -p.shelf_slope * d.dx * cells;
            (level - bed(i)).max(0.0) * d.cell_area()
        };
        // The mouth's room and its two neighbours', one cell away.
        // (A hair less, so the sums' rounding cannot leave a crumb for the next cell.)
        let near = (room(10, 0.0) + room(9, 1.0) + room(11, 1.0)) * (1.0 - 1e-9);
        let mut s = FillScratch::default();
        fill(&d, &sea, 10, near, &p, &mut h, &mut dep, &mut s);
        let level1 = -p.shelf_slope * d.dx;
        assert!(
            (h[9] - level1).abs() < 1e-6 && (h[11] - level1).abs() < 1e-6,
            "{h:?}"
        );
        assert!(
            dep[..9].iter().chain(&dep[12..]).all(|&x| x == 0.0),
            "{dep:?}"
        );
        // The next load goes on to the cells two away, the trough's first before the shelf's.
        fill(
            &d,
            &sea,
            10,
            room(12, 2.0) + 0.5 * room(8, 2.0),
            &p,
            &mut h,
            &mut dep,
            &mut s,
        );
        assert!((h[12] - 2.0 * level1).abs() < 1e-6, "{h:?}");
        assert!(dep[8] > 0.0 && h[8] < 2.0 * level1, "{h:?}");
        assert!(
            dep[..8].iter().chain(&dep[13..]).all(|&x| x == 0.0),
            "{dep:?}"
        );
    }

    /// A mouth at the floor of a hollow fills the hollow from its floor: the load does not climb
    /// the hollow's side while there is room below it.
    #[test]
    fn a_mouth_in_a_hollow_fills_it_from_its_floor() {
        // A V-shaped hollow, 1,000 m deep at the mouth (10), rising 100 m a cell to -100 m.
        let d = Dims::new(21, 1, 1000.0, false);
        let bed = |i: usize| -1000.0 + 100.0 * (i as f64 - 10.0).abs();
        let mut h: Vec<f64> = (0..21).map(bed).collect();
        let sea = vec![true; 21];
        let mut dep = vec![0.0f32; 21];
        let p = FillParams {
            sea_level: 0.0,
            fill_depth_m: 0.0,
            shelf_slope: 0.0005,
            max_cells: 10_000,
        };
        let mut s = FillScratch::default();
        // Less than the mouth cell's room: it all stays at the floor.
        fill(&d, &sea, 10, 0.5e9, &p, &mut h, &mut dep, &mut s);
        assert!(dep[10] > 0.0 && dep.iter().enumerate().all(|(i, &x)| i == 10 || x == 0.0));
        // More: the mouth's cell to the level, then the cells next up the sides, not those above.
        fill(&d, &sea, 10, 1.5e9, &p, &mut h, &mut dep, &mut s);
        assert!(h[10].abs() < 1e-6, "{h:?}");
        assert!(dep[9] > 0.0 && dep[11] > 0.0, "{dep:?}");
        assert!(
            dep[..9].iter().chain(&dep[12..]).all(|&x| x == 0.0),
            "{dep:?}"
        );
    }

    /// The device's slope failure against the CPU's on the worlds of the bit-for-bit test above
    /// (dense, sparse, not wrapping, one and two cells wide, one row), at the stage's iterations
    /// and at 400: the heights and the sediment within a centimetre on floors kilometres deep
    /// (the device's `f32` steps there are a few tenths of a millimetre), the metres moved within
    /// 1e-4 of the CPU's, and matter kept to the `f32` round-off of the changes.
    #[cfg(feature = "gpu")]
    #[test]
    fn the_gpu_slope_failure_matches_the_cpu() {
        let g = match deeptime_gpu::shared(true) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("skipped: {e}");
                return;
            }
        };
        eprintln!("adapter: {}", g.describe());
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
                    let label = format!("{w}x{hh} wrap {wrap} seed {seed} iterations {iterations}");
                    let d = Dims::new(w, hh, 4800.0, wrap);
                    let (h0, a0, sea) = failure_world(w, hh, seed, mounds);
                    let p = FailureParams {
                        critical_slope: s_c,
                        rate: FAILURE_RATE,
                        iterations,
                    };
                    let (mut hc, mut ac) = (h0.clone(), a0.clone());
                    let mc = slope_failure(&d, &sea, 0.0, &p, &mut hc, &mut ac);
                    let (mut hd, mut ad) = (h0.clone(), a0.clone());
                    let md = gpu::slope_failure(&g, &d, &sea, 0.0, &p, &mut hd, &mut ad)
                        .expect("the device runs the slope failure");
                    let worst = |a: &[f64], b: &[f64]| {
                        a.iter()
                            .zip(b)
                            .fold(0.0f64, |m, (x, y)| m.max((x - y).abs()))
                    };
                    let (dh, da) = (worst(&hd, &hc), worst(&ad, &ac));
                    let mass: f64 = hd.iter().sum::<f64>() - h0.iter().sum::<f64>();
                    eprintln!(
                        "{label}: moved {mc:.4} vs {md:.4}, heights {dh:.2e}, sediment {da:.2e}, \
                         mass {mass:.2e}"
                    );
                    assert!(dh <= 1e-2, "{label}: heights {dh:e} from the CPU's");
                    assert!(da <= 1e-2, "{label}: sediment {da:e} from the CPU's");
                    assert!(
                        (md - mc).abs() <= 1e-4 * mc + 1e-6,
                        "{label}: moved {md} against {mc}"
                    );
                    assert!(
                        mass.abs() <= 1e-5 * mc.max(1.0),
                        "{label}: mass changed by {mass:e} m·cells"
                    );
                    if mc > 0.0 {
                        cases += 1;
                    }
                }
            }
        }
        assert!(cases >= 20, "only {cases} cases moved any sediment");
    }
}

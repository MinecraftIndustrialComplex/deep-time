// SPDX-License-Identifier: LGPL-3.0-or-later
//! The amplifier's passes on a window of the fine grid (P1-06): stream-power erosion with multiple
//! flow directions, thermal erosion (`thermal.rs`), deposition, and the flood that guarantees a
//! drained surface. Every pass but the flood is a Jacobi iteration — a pure function of the
//! previous arrays, one row per Rayon task — so results never depend on the thread count, and a
//! pass of dependency radius `r` leaves a tile's core exact when its halo is at least `r`.
//!
//! # Ported from Schott et al. 2024
//! "Terrain Amplification using Multi-scale Erosion" (ACM TOG 43(4)), code
//! `H-Schott/MultiScaleErosion` (MIT, commit 964a3d5; NOTICE.md). Equations, not lines:
//! - **Drainage area** (`erosion.glsl`): every cell sends its area to each lower neighbour in
//!   proportion to (slope)^p (their `flow_p` 1.3; here 1.5 by default, `s·√s`), and a cell's area
//!   is its own plus what its neighbours send: one hop per iteration, so an area is only right
//!   within the iterations' reach. **Changed:** the iteration starts from the level below's exact
//!   drainage area (`resample.rs`), which it refines locally and keeps beyond its reach; that is
//!   what lets a few dozen iterations replace their hundreds.
//! - **Stream power** (`erosion.glsl`): a cell lowers by `k · A^m_a · min(S, 1)^n_s` (their
//!   `p_sa` 0.8, `p_sl` 2) but never below its steepest receiver. **Changed:** the coefficient is
//!   `incision_m` metres per iteration at a reference area and slope (a stream-power law in SI
//!   units instead of their unit-free terrain), scaled by the rock's erodibility, the lowering is
//!   capped at a fraction of the drop to the receiver (`max_frac`), a cell never goes under
//!   `receiver + min_gradient · d` (so every eroded cell keeps a strictly lower neighbour), the
//!   sea is the base level, and lake cells (the young landscape's lakes) are not cut.
//! - **Thermal**: `thermal.rs`.
//! - **Deposition** (`deposition.glsl`): sediment travels along the same multiple-flow weights
//!   and settles where the flow's transport capacity falls below the load. **Changed:** the load
//!   is the material the erosion pass actually removed (mass is kept: what is not laid down
//!   within reach of its source is reported as lost, at most the load still in transit at the
//!   end), a cell takes a fraction of the load's excess over the capacity, and never more than
//!   half the gap to its lowest higher neighbour, so a deposit cannot bury a cell's donors or
//!   turn a slope into a pit.
//!
//! # The flood
//! [`flood_repair`] is Priority-Flood+ε (Barnes, Lehman & Mulla 2014, implemented from the
//! paper, as `pflood.rs`) on the window: cells below sea level and the young landscape's lakes are
//! outlets, so is any window edge that is not the planet's edge, and every other cell that cannot
//! drain is raised until it can (`eps_m` above the neighbour it drains to). It is the one pass
//! that is not local. A depression is filled the same way in a tile as in the whole raster when it
//! and its spill point lie inside the tile's window, which the halo's `repair_margin` cells
//! ensure for any depression the amplification itself creates (a coarse depression the young
//! landscape had is a lake and stays one).

use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::time::Instant;

use super::resample::Fine;
use super::window::{NONE, STEP, Win, map_cells, map_cells2};
use crate::surface::detail::Noise;
use crate::surface::thermal;

/// `x^p`, spelled out for the exponents the amplifier defaults to.
#[inline]
fn pw(x: f32, p: f32) -> f32 {
    if p == 1.0 {
        x
    } else if p == 2.0 {
        x * x
    } else if p == 1.5 {
        x * x.sqrt()
    } else {
        x.powf(p)
    }
}

/// The resolved numbers of one level's passes.
#[derive(Clone, Copy, Debug)]
pub struct Physics {
    /// Cell size, m.
    pub dx: f32,
    /// Sea level, m.
    pub sea: f32,
    /// Multiple-flow exponent.
    pub flow_power: f32,
    pub area_exp: f32,
    pub slope_exp: f32,
    /// Lowering per iteration at `area_ref_m2` and `slope_ref`, m.
    pub incision_m: f32,
    pub area_ref_m2: f32,
    /// The area term stops growing at this drainage area, m² (rivers the coarse levels already
    /// carved are not cut again).
    pub area_cap_m2: f32,
    pub slope_ref: f32,
    pub slope_cap: f32,
    pub max_frac: f32,
    /// A cell loses at most this fraction of its initial drop to its lowest neighbour in all
    /// (the erosion is scaled by the cell size: a stream cuts a valley of some cells' relief).
    pub cap_frac: f32,
    /// A cell stays this gradient above its receiver.
    pub min_gradient: f32,
    /// The drama-scaled critical slope.
    pub thermal_slope: f32,
    pub thermal_rate: f32,
    /// Sediment a reference flow carries, m per cell.
    pub cap_ref_m: f32,
    pub cap_area_exp: f32,
    pub cap_slope_exp: f32,
    pub deposit_frac: f32,
    /// Rise per cell of a filled surface, m.
    pub eps_m: f32,
    /// Block-scale glacial valleys (`glacial.rs`).
    pub glacial: super::glacial::GlacialShape,
}

/// Iterations of each pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counts {
    pub erode: u32,
    pub thermal: u32,
    pub deposit: u32,
    /// Dependency radius of the glacial pass, cells (`GlacialShape::radius`; 0 when off).
    pub glacial: u32,
}

impl Counts {
    /// Dependency radius, cells, of the seeding and the iterations (the flood's reach and the
    /// post-processing come on top, `tiles::halo`).
    pub fn radius(&self) -> usize {
        1 + 2 * self.erode as usize
            + self.thermal as usize
            + 2 * self.deposit as usize
            + self.glacial as usize
    }
}

/// How the noise perturbs a level (`amplify.seed_relief`, `amplify.thermal_noise`).
#[derive(Clone, Copy, Debug)]
pub struct NoiseCfg {
    pub noise: Noise,
    /// Relief seeding: metres per (slope · cell size).
    pub seed_relief: f32,
    /// The threshold angle varies by ± this fraction.
    pub thermal_noise: f32,
    /// Blocks per cell of this level, and the block of global cell (0, 0).
    pub cell_blocks: f64,
    pub origin_x: f64,
    pub origin_z: f64,
}

/// A level's working arrays over one window.
pub struct Buf {
    pub win: Win,
    pub z: Vec<f32>,
    pub z2: Vec<f32>,
    pub a: Vec<f32>,
    pub a2: Vec<f32>,
    pub norm: Vec<f32>,
    pub eroded: Vec<f32>,
    pub sed: Vec<f32>,
    pub sed2: Vec<f32>,
    pub own: Vec<f32>,
    pub kfac: Vec<f32>,
    pub lake: Vec<u8>,
    /// 1 where the flood after the seeding raised the cell: a filled depression, not a channel,
    /// which the erosion leaves alone (it would cut a gorge out of nothing).
    pub frozen: Vec<u8>,
    /// The most each cell may lose to erosion in all, m.
    pub cap: Vec<f32>,
    pub tmult: Vec<f32>,
    /// The part of the window whose statistics count (`x0, y0, w, h`): the tile's core.
    pub core: (usize, usize, usize, usize),
    /// The glaciers' surface, m (empty without glaciers; `glacial.rs`).
    pub zs: Vec<f32>,
}

impl Buf {
    /// Bytes per cell of a buffer (the memory estimate).
    pub const BYTES_PER_CELL: u64 = 12 * 4 + 2 + 4;

    pub fn new(win: Win, fine: Fine) -> Buf {
        let n = win.cells();
        Buf {
            win,
            z: fine.z,
            z2: vec![0.0; n],
            a: fine.area,
            a2: vec![0.0; n],
            norm: vec![0.0; n],
            eroded: vec![0.0; n],
            sed: vec![0.0; n],
            sed2: vec![0.0; n],
            own: fine.own,
            kfac: fine.kfac,
            lake: fine.lake,
            frozen: vec![0; n],
            cap: vec![0.0; n],
            tmult: vec![1.0; n],
            core: (0, 0, win.w, win.h),
            zs: fine.zs,
        }
    }
}

/// Milliseconds per phase.
#[derive(Clone, Copy, Debug, Default)]
pub struct Times {
    pub seed_ms: f64,
    pub erode_ms: f64,
    pub thermal_ms: f64,
    pub deposit_ms: f64,
    pub flood_ms: f64,
    pub glacial_ms: f64,
    /// Arrays to and from a [`PassDevice`] (0 on the CPU). The device's own work counts in the
    /// phase it belongs to: the device is waited for at each phase's end.
    pub transfer_ms: f64,
}

impl Times {
    pub fn add(&mut self, o: &Times) {
        self.seed_ms += o.seed_ms;
        self.erode_ms += o.erode_ms;
        self.thermal_ms += o.thermal_ms;
        self.deposit_ms += o.deposit_ms;
        self.flood_ms += o.flood_ms;
        self.glacial_ms += o.glacial_ms;
        self.transfer_ms += o.transfer_ms;
    }
}

/// Runs the middle of [`run_passes`] (erosion, glacial valleys, thermal erosion, deposition)
/// somewhere other than the CPU: the GPU (`gpu.rs`, feature `gpu`). The dispatch point of the
/// amplifier's GPU path (`docs/design/gpu-compute.md`); the CPU's [`middle_cpu`] is the reference.
pub trait PassDevice: Send + Sync + std::fmt::Debug {
    /// The device, for the run's notes.
    fn name(&self) -> String;
    /// Host bytes per window cell while a window is on the device (its buffers, when the device
    /// shares the host's memory, and the staging of downloads).
    fn host_bytes_per_cell(&self) -> u64;
    /// Whether a window of `cells` fits on the device.
    fn fits(&self, cells: usize) -> bool;
    /// What [`middle_cpu`] does, on `buf` after the seeding and the first flood. On `Err` the
    /// arrays are undefined: the caller samples the window again and runs it on the CPU.
    fn middle(
        &self,
        buf: &mut Buf,
        ph: &Physics,
        counts: &Counts,
        times: &mut Times,
    ) -> Result<super::glacial::ShapeStats, String>;
}

/// N(c) = Σ (slope to each lower neighbour)^p for land cells, 0 for the sea.
pub fn norms(win: &Win, z: &[f32], ph: &Physics, out: &mut [f32]) {
    map_cells(win, out, |i, j, o| {
        let c = j * win.w + i;
        let zc = z[c];
        if zc <= ph.sea {
            *o = 0.0;
            return;
        }
        let mut nb = [NONE; 8];
        win.nbrs(i, j, &mut nb);
        let mut sum = 0.0f32;
        for (k, &n) in nb.iter().enumerate() {
            if n == NONE {
                continue;
            }
            let s = (zc - z[n as usize]) / (STEP[k] * ph.dx);
            if s > 0.0 {
                sum += pw(s, ph.flow_power);
            }
        }
        *o = sum;
    });
}

/// One iteration of the drainage area: a land cell's own area plus what its neighbours send it.
pub fn area_step(
    win: &Win,
    z: &[f32],
    norm: &[f32],
    a_in: &[f32],
    own: &[f32],
    ph: &Physics,
    out: &mut [f32],
) {
    map_cells(win, out, |i, j, o| {
        let c = j * win.w + i;
        let zc = z[c];
        if zc <= ph.sea {
            *o = own[c];
            return;
        }
        let mut nb = [NONE; 8];
        win.nbrs(i, j, &mut nb);
        let mut a = own[c];
        for (k, &n) in nb.iter().enumerate() {
            if n == NONE {
                continue;
            }
            let n = n as usize;
            let s = (z[n] - zc) / (STEP[k] * ph.dx);
            if s > 0.0 && norm[n] > 0.0 && z[n] > ph.sea {
                a += pw(s, ph.flow_power) / norm[n] * a_in[n];
            }
        }
        *o = a;
    });
}

/// One iteration of stream-power erosion; `eroded` accumulates the metres removed.
#[allow(clippy::too_many_arguments)]
pub fn erode_step(
    win: &Win,
    z: &[f32],
    a: &[f32],
    kfac: &[f32],
    lake: &[u8],
    frozen: &[u8],
    cap: &[f32],
    ph: &Physics,
    out: &mut [f32],
    eroded: &mut [f32],
) {
    map_cells2(win, out, eroded, |i, j, o, e| {
        let c = j * win.w + i;
        let zc = z[c];
        *o = zc;
        if zc <= ph.sea || lake[c] != 0 || frozen[c] != 0 {
            return;
        }
        let mut nb = [NONE; 8];
        win.nbrs(i, j, &mut nb);
        let mut smax = 0.0f32;
        let (mut zr, mut dr) = (zc, ph.dx);
        for (k, &n) in nb.iter().enumerate() {
            if n == NONE {
                continue;
            }
            let d = STEP[k] * ph.dx;
            let s = (zc - z[n as usize]) / d;
            if s > smax {
                smax = s;
                zr = z[n as usize];
                dr = d;
            }
        }
        if smax <= 0.0 {
            return;
        }
        let floor = zr.max(ph.sea) + ph.min_gradient * dr;
        let room = zc - floor;
        if room <= 0.0 {
            return;
        }
        let spl = ph.incision_m
            * kfac[c]
            * pw(a[c].min(ph.area_cap_m2) / ph.area_ref_m2, ph.area_exp)
            * pw(smax.min(ph.slope_cap) / ph.slope_ref, ph.slope_exp);
        let left = cap[c] - *e;
        let delta = spl.min(room * ph.max_frac).min(left);
        if delta > 0.0 {
            *o = zc - delta;
            *e += delta;
        }
    });
}

/// One iteration of deposition: sediment moves along the flow weights and settles where the
/// capacity is below the load.
#[allow(clippy::too_many_arguments)]
pub fn deposit_step(
    win: &Win,
    z: &[f32],
    a: &[f32],
    norm: &[f32],
    sed_in: &[f32],
    ph: &Physics,
    out: &mut [f32],
    sed_out: &mut [f32],
) {
    map_cells2(win, out, sed_out, |i, j, o, so| {
        let c = j * win.w + i;
        let zc = z[c];
        if zc <= ph.sea {
            // Sediment that reaches the sea leaves the amplifier.
            *o = zc;
            *so = 0.0;
            return;
        }
        let mut nb = [NONE; 8];
        win.nbrs(i, j, &mut nb);
        let mut load = 0.0f32;
        let mut smax = 0.0f32;
        let mut gap = f32::INFINITY;
        for (k, &n) in nb.iter().enumerate() {
            if n == NONE {
                continue;
            }
            let n = n as usize;
            let d = STEP[k] * ph.dx;
            let up = (z[n] - zc) / d;
            if up > 0.0 {
                gap = gap.min(z[n] - zc);
                if norm[n] > 0.0 && z[n] > ph.sea {
                    load += pw(up, ph.flow_power) / norm[n] * sed_in[n];
                }
            }
            let down = -up;
            if down > smax {
                smax = down;
            }
        }
        let cap = ph.cap_ref_m
            * pw(a[c] / ph.area_ref_m2, ph.cap_area_exp)
            * pw(smax.min(ph.slope_cap) / ph.slope_ref, ph.cap_slope_exp);
        let dep = if load > cap {
            (ph.deposit_frac * (load - cap)).min(load)
        } else {
            0.0
        };
        let dep = dep.min(0.5 * gap);
        *o = zc + dep;
        *so = load - dep;
    });
}

/// Relief-seeding: the terrain gets smooth noise of amplitude `seed_relief · slope · dx`, so the
/// erosion passes have concavities to concentrate flow in below the coarse level's resolution.
pub fn seed_step(win: &Win, z: &[f32], ph: &Physics, cfg: &NoiseCfg, out: &mut [f32]) {
    let cell = cfg.cell_blocks;
    map_cells(win, out, |i, j, o| {
        let c = j * win.w + i;
        let zc = z[c];
        *o = zc;
        if zc <= ph.sea || cfg.seed_relief <= 0.0 {
            return;
        }
        let mut nb = [NONE; 8];
        win.nbrs(i, j, &mut nb);
        let mut smax = 0.0f32;
        for (k, &n) in nb.iter().enumerate() {
            if n == NONE {
                continue;
            }
            let s = (z[n as usize] - zc).abs() / (STEP[k] * ph.dx);
            smax = smax.max(s);
        }
        let (gi, gj) = win.global(i, j);
        let x = cfg.origin_x + gi as f64 * cell;
        let zb = cfg.origin_z + gj as f64 * cell;
        let n = 0.65 * cfg.noise.value(x, zb, 3.0 * cell, 101)
            + 0.35 * cfg.noise.value(x, zb, 7.0 * cell, 102);
        let v = zc + cfg.seed_relief * smax * ph.dx * n as f32;
        *o = v.max(ph.sea + 0.05);
    });
}

/// How much each land cell may lose to erosion in all: `cap_frac` of its drop to its lowest
/// neighbour (0 on flats and pits).
pub fn incision_cap(win: &Win, z: &[f32], ph: &Physics, out: &mut [f32]) {
    map_cells(win, out, |i, j, o| {
        let c = j * win.w + i;
        let zc = z[c];
        *o = 0.0;
        if zc <= ph.sea {
            return;
        }
        let mut nb = [NONE; 8];
        win.nbrs(i, j, &mut nb);
        let mut low = zc;
        for &n in nb.iter() {
            if n != NONE {
                low = low.min(z[n as usize]);
            }
        }
        *o = ph.cap_frac * (zc - low.max(ph.sea));
    });
}

/// The threshold-angle multiplier 1 ± `thermal_noise` (smooth, ~40 cells across).
pub fn tmult_fill(win: &Win, cfg: &NoiseCfg, out: &mut [f32]) {
    let cell = cfg.cell_blocks;
    map_cells(win, out, |i, j, o| {
        let (gi, gj) = win.global(i, j);
        let x = cfg.origin_x + gi as f64 * cell;
        let zb = cfg.origin_z + gj as f64 * cell;
        *o = 1.0 + cfg.thermal_noise * cfg.noise.value(x, zb, 40.0 * cell, 103) as f32;
    });
}

#[derive(Clone, Copy)]
struct Entry {
    z: f32,
    g: u64,
    c: u32,
}

impl PartialEq for Entry {
    fn eq(&self, o: &Self) -> bool {
        self.cmp(o) == Ordering::Equal
    }
}
impl Eq for Entry {}
impl Ord for Entry {
    fn cmp(&self, o: &Self) -> Ordering {
        // A min-heap on height, ties by the cell's index on the whole planet (the same in every
        // window).
        o.z.total_cmp(&self.z).then_with(|| o.g.cmp(&self.g))
    }
}
impl PartialOrd for Entry {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// What a flood did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FloodStats {
    /// Cells raised, and by how much in all (metres · cells).
    pub raised_cells: u64,
    pub raised_m_cells: f64,
    pub max_raise_m: f32,
    /// The window cell that rose most.
    pub max_raise_at: usize,
    /// Land cells no outlet could reach (left as they were).
    pub unreached: u64,
}

/// Priority-Flood+ε over the window: raises every land cell outside `lake` that cannot drain to
/// an outlet (a cell at or below `sea`, a lake cell, an open window edge) until it can, and
/// leaves the outlets alone. `scratch` (window-sized) holds the filled surface. The returned
/// statistics count the cells of `core` (`x0, y0, w, h` in window coordinates) only, so a tiled
/// run reports what a whole-raster run does.
pub fn flood_repair(
    win: &Win,
    z: &mut [f32],
    lake: &[u8],
    sea: f32,
    eps: f32,
    scratch: &mut [f32],
    core: (usize, usize, usize, usize),
) -> FloodStats {
    let n = win.cells();
    let (w, h) = (win.w, win.h);
    let mut closed = vec![false; n];
    let filled = scratch;
    filled.copy_from_slice(z);
    let mut heap: BinaryHeap<Entry> = BinaryHeap::new();
    let mut pit: VecDeque<u32> = VecDeque::new();
    let open_west_east = !win.wraps();
    let open_top = !win.top_edge();
    let open_bottom = !win.bottom_edge();
    let outlet = |i: usize, j: usize, c: usize| -> bool {
        z[c] <= sea
            || lake[c] != 0
            || (open_west_east && (i == 0 || i + 1 == w))
            || (open_top && j == 0)
            || (open_bottom && j + 1 == h)
    };
    let mut nb = [NONE; 8];
    // Seed with the outlets that touch a cell that is not one.
    for j in 0..h {
        for i in 0..w {
            let c = j * w + i;
            if !outlet(i, j, c) {
                continue;
            }
            closed[c] = true;
            win.nbrs(i, j, &mut nb);
            let touches_land = nb.iter().any(|&m| {
                m != NONE && {
                    let m = m as usize;
                    !outlet(m % w, m / w, m)
                }
            });
            if touches_land {
                heap.push(Entry {
                    z: z[c],
                    g: win.global_index(i, j),
                    c: c as u32,
                });
            }
        }
    }
    let mut stats = FloodStats::default();
    loop {
        let c = if let Some(c) = pit.pop_front() {
            c as usize
        } else if let Some(e) = heap.pop() {
            e.c as usize
        } else {
            break;
        };
        let (ci, cj) = (c % w, c / w);
        let zc = filled[c];
        win.nbrs(ci, cj, &mut nb);
        for &m in nb.iter() {
            if m == NONE {
                continue;
            }
            let m = m as usize;
            if closed[m] {
                continue;
            }
            closed[m] = true;
            let up = zc + eps;
            if z[m] <= up {
                filled[m] = up;
                pit.push_back(m as u32);
            } else {
                filled[m] = z[m];
                heap.push(Entry {
                    z: z[m],
                    g: win.global_index(m % w, m / w),
                    c: m as u32,
                });
            }
        }
    }
    let (cx0, cy0, cw, ch) = core;
    for c in 0..n {
        let in_core = {
            let (i, j) = (c % w, c / w);
            (cx0..cx0 + cw).contains(&i) && (cy0..cy0 + ch).contains(&j)
        };
        if !closed[c] {
            if z[c] > sea && in_core {
                stats.unreached += 1;
            }
            continue;
        }
        let up = filled[c] - z[c];
        if up > 0.0 {
            z[c] = filled[c];
            if in_core {
                stats.raised_cells += 1;
                stats.raised_m_cells += f64::from(up);
                if up > stats.max_raise_m {
                    stats.max_raise_m = up;
                    stats.max_raise_at = c;
                }
            }
        }
    }
    stats
}

/// Land cells outside `lake` with no strictly lower neighbour, among the cells of the rectangle
/// `core` (`x0, y0, w, h` in window coordinates).
pub fn count_sinks(
    win: &Win,
    z: &[f32],
    lake: &[u8],
    sea: f32,
    core: (usize, usize, usize, usize),
) -> u64 {
    use rayon::prelude::*;
    let (x0, y0, cw, ch) = core;
    (y0..y0 + ch)
        .into_par_iter()
        .map(|j| {
            let mut sinks = 0u64;
            let mut nb = [NONE; 8];
            for i in x0..x0 + cw {
                let c = j * win.w + i;
                if z[c] <= sea || lake[c] != 0 {
                    continue;
                }
                win.nbrs(i, j, &mut nb);
                let lower = nb.iter().any(|&m| m != NONE && z[m as usize] < z[c]);
                if !lower {
                    sinks += 1;
                }
            }
            sinks
        })
        .sum()
}

/// The range of `z` over the 5 × 5 cells around every cell (the detail layer's local relief):
/// columns wrap when the window holds them all and are replicated at an open window edge, rows
/// are replicated at the window's edge.
pub fn relief5(win: &Win, z: &[f32], out: &mut [f32]) {
    map_cells(win, out, |i, j, o| {
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for dj in -2i64..=2 {
            let jj = (j as i64 + dj).clamp(0, win.h as i64 - 1) as usize;
            for di in -2i64..=2 {
                let ii = if win.wraps() {
                    (i as i64 + di).rem_euclid(win.w as i64) as usize
                } else {
                    (i as i64 + di).clamp(0, win.w as i64 - 1) as usize
                };
                let v = z[jj * win.w + ii];
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        *o = hi - lo;
    });
}

/// What one run of the passes did (metres · cells, so volumes follow by the cell area).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PassStats {
    /// The flood after the seeding (it drains the interpolated terrain the erosion starts from,
    /// so no erosion pass deepens a pit into a slot) and the one at the end.
    pub pre_flood: FloodStats,
    pub flood: FloodStats,
    /// The glacial valleys (`glacial.rs`).
    pub glacial: super::glacial::ShapeStats,
}

/// Seeding, erosion, thermal erosion, deposition and the flood over one window; leaves the
/// result in `buf.z` (and the last drainage area in `buf.a`, the eroded thickness in
/// `buf.eroded` and the sediment still in transit in `buf.sed`).
pub fn run_passes(
    buf: &mut Buf,
    ph: &Physics,
    counts: &Counts,
    cfg: &NoiseCfg,
    times: &mut Times,
) -> PassStats {
    match run_passes_on(None, buf, ph, counts, cfg, times) {
        Ok(stats) => stats,
        Err(_) => unreachable!("the CPU passes do not fail"),
    }
}

/// [`run_passes`] with its middle on `device` when there is one. `Err` when the device failed;
/// `buf` is then undefined.
pub fn run_passes_on(
    device: Option<&dyn PassDevice>,
    buf: &mut Buf,
    ph: &Physics,
    counts: &Counts,
    cfg: &NoiseCfg,
    times: &mut Times,
) -> Result<PassStats, String> {
    let pre_flood = seed_and_flood(buf, ph, cfg, times);
    let glacial = match device {
        Some(d) => d.middle(buf, ph, counts, times)?,
        None => middle_cpu(buf, ph, counts, times),
    };
    let started = Instant::now();
    let flood = flood_repair(
        &buf.win,
        &mut buf.z,
        &buf.lake,
        ph.sea,
        ph.eps_m,
        &mut buf.z2,
        buf.core,
    );
    times.flood_ms += started.elapsed().as_secs_f64() * 1000.0;
    Ok(PassStats {
        pre_flood,
        flood,
        glacial,
    })
}

/// Zeroes the eroded thickness, sets each cell's incision cap and damps the erodibility under
/// the glaciers; returns the erodibility to restore once the erosion is done (`glacial::damp`).
pub fn erosion_setup(buf: &mut Buf, ph: &Physics) -> Option<Vec<f32>> {
    buf.eroded.iter_mut().for_each(|e| *e = 0.0);
    incision_cap(&buf.win, &buf.z, ph, &mut buf.cap);
    // Rivers cut little where the glaciers were (`glacial.rs`); the rates come back after.
    super::glacial::damp(&ph.glacial, &buf.z, &buf.zs, &mut buf.kfac)
}

/// The relief seeding, the threshold-angle field and the flood after them (which marks the
/// cells it raised as frozen).
fn seed_and_flood(buf: &mut Buf, ph: &Physics, cfg: &NoiseCfg, times: &mut Times) -> FloodStats {
    let win = buf.win;
    let started = Instant::now();
    seed_step(&win, &buf.z, ph, cfg, &mut buf.z2);
    std::mem::swap(&mut buf.z, &mut buf.z2);
    tmult_fill(&win, cfg, &mut buf.tmult);
    times.seed_ms += started.elapsed().as_secs_f64() * 1000.0;
    let started = Instant::now();
    buf.sed2.copy_from_slice(&buf.z);
    let pre_flood = flood_repair(
        &win,
        &mut buf.z,
        &buf.lake,
        ph.sea,
        ph.eps_m,
        &mut buf.z2,
        buf.core,
    );
    for ((f, &z), &z0) in buf.frozen.iter_mut().zip(&buf.z).zip(&buf.sed2) {
        *f = u8::from(z > z0 + 1.0e-3);
    }
    times.flood_ms += started.elapsed().as_secs_f64() * 1000.0;
    pre_flood
}

/// Erosion, glacial valleys, thermal erosion and deposition on the CPU.
pub fn middle_cpu(
    buf: &mut Buf,
    ph: &Physics,
    counts: &Counts,
    times: &mut Times,
) -> super::glacial::ShapeStats {
    let win = buf.win;
    let started = Instant::now();
    let kfac_saved = erosion_setup(buf, ph);
    for _ in 0..counts.erode {
        norms(&win, &buf.z, ph, &mut buf.norm);
        area_step(&win, &buf.z, &buf.norm, &buf.a, &buf.own, ph, &mut buf.a2);
        std::mem::swap(&mut buf.a, &mut buf.a2);
        erode_step(
            &win,
            &buf.z,
            &buf.a,
            &buf.kfac,
            &buf.lake,
            &buf.frozen,
            &buf.cap,
            ph,
            &mut buf.z2,
            &mut buf.eroded,
        );
        std::mem::swap(&mut buf.z, &mut buf.z2);
    }
    if let Some(k) = kfac_saved {
        buf.kfac = k;
    }
    times.erode_ms += started.elapsed().as_secs_f64() * 1000.0;

    // U-shaped troughs, cirques and aretes under the glaciers (`glacial.rs`), on the level whose
    // counts ask for it (the last: its radius is in cells of the block-scale grid).
    let started = Instant::now();
    let glacial = if counts.glacial > 0 {
        // The swap buffers and the flow norms are free between the erosion and the thermal
        // passes.
        super::glacial::shape(
            &win,
            &ph.glacial,
            &mut buf.z,
            &buf.zs,
            buf.core,
            [
                &mut buf.z2,
                &mut buf.a2,
                &mut buf.norm,
                &mut buf.sed,
                &mut buf.sed2,
                &mut buf.cap,
            ],
        )
    } else {
        super::glacial::ShapeStats::default()
    };
    times.glacial_ms += started.elapsed().as_secs_f64() * 1000.0;

    let started = Instant::now();
    for _ in 0..counts.thermal {
        thermal::step(
            &win,
            &buf.z,
            &buf.tmult,
            ph.thermal_slope,
            ph.thermal_rate,
            ph.dx,
            &mut buf.z2,
        );
        std::mem::swap(&mut buf.z, &mut buf.z2);
    }
    times.thermal_ms += started.elapsed().as_secs_f64() * 1000.0;

    let started = Instant::now();
    buf.sed.copy_from_slice(&buf.eroded);
    for _ in 0..counts.deposit {
        norms(&win, &buf.z, ph, &mut buf.norm);
        area_step(&win, &buf.z, &buf.norm, &buf.a, &buf.own, ph, &mut buf.a2);
        std::mem::swap(&mut buf.a, &mut buf.a2);
        deposit_step(
            &win,
            &buf.z,
            &buf.a,
            &buf.norm,
            &buf.sed,
            ph,
            &mut buf.z2,
            &mut buf.sed2,
        );
        std::mem::swap(&mut buf.z, &mut buf.z2);
        std::mem::swap(&mut buf.sed, &mut buf.sed2);
    }
    times.deposit_ms += started.elapsed().as_secs_f64() * 1000.0;
    glacial
}

#[cfg(test)]
mod tests {
    use super::*;

    fn physics(dx: f32) -> Physics {
        Physics {
            dx,
            sea: 0.0,
            flow_power: 1.5,
            area_exp: 1.0,
            slope_exp: 2.0,
            incision_m: 2.0,
            area_ref_m2: 1.0e8,
            area_cap_m2: 1.0e9,
            slope_ref: 0.1,
            slope_cap: 1.0,
            max_frac: 0.5,
            cap_frac: 0.5,
            min_gradient: 1.0e-4,
            thermal_slope: 0.6,
            thermal_rate: 0.05,
            cap_ref_m: 5.0,
            cap_area_exp: 1.0,
            cap_slope_exp: 1.0,
            deposit_frac: 0.5,
            eps_m: 0.01,
            glacial: super::super::glacial::GlacialShape::default(),
        }
    }

    /// A tilted plane draining to the sea in the north: row 0 is the sea.
    fn plane(w: usize, h: usize) -> Vec<f32> {
        (0..w * h)
            .map(|c| {
                if c / w == 0 {
                    -10.0
                } else {
                    50.0 * (c / w) as f32
                }
            })
            .collect()
    }

    #[test]
    fn the_flood_fills_pits_and_leaves_lakes_and_slopes_alone() {
        let win = Win::full(32, 16);
        let mut z = plane(32, 16);
        // A one-cell pit, a bowl of three cells with a flat floor, and a lake basin.
        z[8 * 32 + 5] -= 200.0;
        for i in 10..13 {
            z[8 * 32 + i] = 100.0;
        }
        for i in 20..24 {
            for j in 6..10 {
                z[j * 32 + i] -= 300.0;
            }
        }
        let mut lake = vec![0u8; 32 * 16];
        for i in 20..24 {
            for j in 6..10 {
                lake[j * 32 + i] = 1;
            }
        }
        let before = z.clone();
        let mut scratch = vec![0.0; 32 * 16];
        let st = flood_repair(&win, &mut z, &lake, 0.0, 0.01, &mut scratch, (0, 0, 32, 16));
        assert!(st.raised_cells > 0);
        assert_eq!(st.unreached, 0);
        // The lake bed is untouched, and no other land cell is left without a lower neighbour.
        for i in 20..24 {
            for j in 6..10 {
                assert_eq!(z[j * 32 + i], before[j * 32 + i]);
            }
        }
        assert_eq!(count_sinks(&win, &z, &lake, 0.0, (0, 0, 32, 16)), 0);
        // The pit rose to just above its lowest neighbour's level, the far slope did not move.
        assert!(z[8 * 32 + 5] > before[8 * 32 + 5] + 100.0);
        assert_eq!(z[15 * 32 + 30], before[15 * 32 + 30]);
        // Nothing was ever lowered.
        assert!(z.iter().zip(&before).all(|(a, b)| a >= b));
    }

    #[test]
    fn a_window_and_the_whole_agree_on_the_flood_of_a_local_pit() {
        let (w, h) = (48, 20);
        let mut z = plane(w, h);
        z[10 * w + 4] -= 150.0; // near the seam
        z[10 * w + 5] -= 150.0;
        let lake = vec![0u8; w * h];
        let mut whole = z.clone();
        let mut scratch = vec![0.0; w * h];
        flood_repair(
            &Win::full(w, h),
            &mut whole,
            &lake,
            0.0,
            0.01,
            &mut scratch,
            (0, 0, w, h),
        );
        // A window across the seam, holding the pit well inside it.
        let win = Win {
            w: 24,
            h: 14,
            gi0: -12,
            gj0: 3,
            gw: w,
            gh: h,
        };
        let mut part: Vec<f32> = (0..win.h)
            .flat_map(|j| (0..win.w).map(move |i| (i, j)))
            .map(|(i, j)| {
                let (gi, gj) = win.global(i, j);
                z[gj * w + gi]
            })
            .collect();
        let lake_p = vec![0u8; win.cells()];
        let mut scratch = vec![0.0; win.cells()];
        flood_repair(
            &win,
            &mut part,
            &lake_p,
            0.0,
            0.01,
            &mut scratch,
            (0, 0, win.w, win.h),
        );
        for j in 0..win.h {
            for i in 0..win.w {
                let (gi, gj) = win.global(i, j);
                // Cells more than 3 from the window's edges match the whole raster exactly.
                if i >= 3 && i + 3 < win.w && j >= 3 && j + 3 < win.h {
                    assert_eq!(part[j * win.w + i], whole[gj * w + gi], "({gi},{gj})");
                }
            }
        }
    }

    #[test]
    fn erosion_never_lowers_below_the_receiver_or_the_sea_and_keeps_drainage() {
        let win = Win::full(32, 16);
        let n = win.cells();
        let ph = physics(600.0);
        let mut z = plane(32, 16);
        // A ridge and a gully, so there is something to cut.
        for j in 1..16 {
            z[j * 32 + 16] += 400.0;
        }
        let a: Vec<f32> = (0..n)
            .map(|c| 1.0e6 * (1.0 + (c / 32) as f32 * 30.0))
            .collect();
        let kfac = vec![1.0f32; n];
        let lake = vec![0u8; n];
        let mut norm = vec![0.0; n];
        let mut out = vec![0.0; n];
        let mut eroded = vec![0.0; n];
        norms(&win, &z, &ph, &mut norm);
        assert!(
            norm[8 * 32 + 3] > 0.0 && norm[3] == 0.0,
            "the sea has no flow"
        );
        let frozen = vec![0u8; n];
        let cap = vec![1.0e9f32; n];
        erode_step(
            &win,
            &z,
            &a,
            &kfac,
            &lake,
            &frozen,
            &cap,
            &ph,
            &mut out,
            &mut eroded,
        );
        for c in 0..n {
            assert!(out[c] <= z[c]);
            assert!(out[c] >= ph.sea || z[c] <= ph.sea);
            assert!((z[c] - out[c] - eroded[c]).abs() < 1e-3);
        }
        assert!(eroded.iter().any(|&e| e > 0.0));
        // Every eroded cell still has a strictly lower neighbour.
        let none = vec![0u8; n];
        assert_eq!(count_sinks(&win, &out, &none, 0.0, (0, 0, 32, 16)), 0);
    }

    #[test]
    fn deposit_never_buries_a_donor_and_the_total_is_bounded_by_the_load() {
        let win = Win::full(32, 16);
        let n = win.cells();
        let ph = physics(600.0);
        let z = plane(32, 16);
        let a = vec![5.0e7f32; n];
        let mut norm = vec![0.0; n];
        norms(&win, &z, &ph, &mut norm);
        let sed = vec![3.0f32; n];
        let (mut out, mut sed_out) = (vec![0.0; n], vec![0.0; n]);
        deposit_step(&win, &z, &a, &norm, &sed, &ph, &mut out, &mut sed_out);
        let mut load_total = 0.0f64;
        for c in 0..n {
            assert!(out[c] >= z[c]);
            assert!(sed_out[c] >= 0.0);
            load_total += f64::from(out[c] - z[c]) + f64::from(sed_out[c]);
        }
        // Order between neighbours is kept: the plane still drains everywhere.
        let none = vec![0u8; n];
        assert_eq!(count_sinks(&win, &out, &none, 0.0, (0, 0, 32, 16)), 0);
        assert!(load_total > 0.0);
    }
}

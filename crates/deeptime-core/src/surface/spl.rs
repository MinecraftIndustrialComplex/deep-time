// SPDX-License-Identifier: LGPL-3.0-or-later
//! Fluvial erosion and deposition: the implicit stream-power law of Braun & Willett 2013
//! (Geomorphology 180–181:170–179) extended with the erosion–deposition term of Yuan, Braun,
//! Guerit, Rouby & Cordonnier 2019, "A new efficient method to solve the stream power law model
//! taking into account sediment deposition" (JGR Earth Surface 124:1346–1365), and a critical
//! slope in the manner of Landlab's `ThresholdEroder` (MIT; see below). Implemented from the
//! papers; fastscapelib (GPL-3.0) was not read.
//!
//! # Equations
//! For every land cell i with receiver r at distance L, effective drainage area A (runoff-weighted,
//! `routing::accumulate`) and erodibility K (the preset's K times the surface rock's multiplier):
//!
//! ∂h/∂t = U − K A^m S^n + (G / A) Q_in,
//!
//! where Q_in is the sediment flux entering i from upstream (m³/yr) and G the dimensionless
//! deposition coefficient of Yuan et al. (G = 0 is the detachment-limited law of Braun & Willett).
//! Uplift and advection are applied before the solve (the surface stage adds them to h), so each
//! step solves, from the outlets upstream (the stack order), the backward-Euler equation
//!
//! h_i = h*_i + Δt·G·Q_in,i / A_i − Δt·K A^m ((h_i − h_r) / L)^n,
//!
//! exactly for n = 1 (h_i = (h*_i + Δt·G·Q_in/A + F·h_r) / (1 + F), F = K A^m Δt / L) and by
//! Newton's method otherwise. Q_in depends on the solution upstream, so the solve is repeated
//! (Yuan et al.'s iteration) with the fluxes of the previous pass until the largest change falls
//! below `tol_m` or `max_iter` passes are done; the flux pass that ends every iteration makes h
//! and the fluxes consistent, so **mass is conserved exactly** whatever the number of iterations:
//! Σ (h* − h)·a over land equals the sediment delivered to the base level (the sea).
//!
//! # Lakes, the base level and deposition limits
//! - A cell whose receiver is a base-level cell (the sea) erodes towards the base level
//!   (`base_z`, sea level), not towards the sea floor.
//! - Lake cells (filled by the priority flood by more than [`LAKE_TOL_M`]) do not incise. A lake
//!   traps the sediment that reaches it, up to its room (the volume under its level), and passes
//!   the rest on at its exit; what it keeps settles over the whole lake, every cell raised by the
//!   same share of its depth (HYD-110: a lake is a still basin whose floor takes the load, not a
//!   channel through it; before, only the cells on the flow path filled, a causeway that left the
//!   rest of the lake empty for good). A cell that drains into a lake erodes towards the lake
//!   surface.
//! - Deposition never exceeds the supply: a cell that would take more than flows into it takes
//!   exactly its inflow (the flux leaving it is then zero).
//! - Sinks (land nothing drains) keep everything that reaches them.
//!
//! # Critical slope (Landlab `ThresholdEroder`, MIT)
//! Ported from the documented rule of Landlab's `ThresholdEroder`
//! (https://github.com/landlab/landlab, `src/landlab/components/threshold_eroder/`, MIT):
//! slopes steeper than S_c are cut back to S_c, S(S > S_c) = S_c, walking the stack from the
//! outlets. **Change:** Landlab lets the removed rock "dissolve away"; here it becomes sediment of
//! the cell (it enters the flux pass as erosion), so landslides feed the rivers and mass is kept.

use rayon::prelude::*;

use super::grid::SurfaceGrid;
use super::pflood::{BASE, LAND, SINK};
use super::routing::Routing;
use super::routing::forest::{NONE, SharedF64};

pub use super::pflood::LAKE_TOL_M;

/// Source-rock families for provenance (the strata log's `provenance`).
pub const FAMILIES: usize = 4;

/// Parameters of one solve.
#[derive(Clone, Copy, Debug)]
pub struct SplParams {
    pub m: f64,
    pub n: f64,
    /// Yuan et al.'s G (0 = detachment-limited).
    pub g: f64,
    /// Time step, years.
    pub dt: f64,
    /// Critical slope S_c (∞ = none).
    pub sc: f64,
    pub max_iter: usize,
    /// Convergence of the iteration, metres.
    pub tol_m: f64,
}

/// The inputs of one solve.
pub struct SplInput<'a> {
    pub kind: &'a [u8],
    /// Elevation after uplift and advection (h*), m.
    pub h0: &'a [f64],
    /// The filled surface (lake levels), m.
    pub filled: &'a [f64],
    /// Base level of base-level cells, m.
    pub base_z: &'a [f64],
    /// Effective drainage area, m².
    pub area: &'a [f64],
    /// Erodibility K per cell, m^(1−2m)/yr.
    pub kf: &'a [f64],
}

/// What a solve did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SplReport {
    pub iterations: usize,
    /// Largest elevation change of the last iteration, m.
    pub residual_m: f64,
}

/// [`lake_links`]: not a lake cell (or a sink).
const NOT_LAKE: u8 = 0;
/// A lake cell draining into another cell of the same lake.
const LAKE_CHAIN: u8 = 1;
/// A lake cell draining out of its lake into the sea, a sink or another lake: the lake upstream
/// of it settles here.
const LAKE_EXIT: u8 = 2;
/// A lake cell draining out of its lake over its spill, a land cell: the lake settles at the
/// spill, with every other cell of the lake that drains over the same spill.
const LAKE_SPILL: u8 = 3;

/// Every cell's place in its lake, in stack order: a lake cell (filled more than [`LAKE_TOL_M`],
/// not a sink) drains along the lake's surface to the cell where its water leaves the lake. A
/// receiver is in the same lake when it is a lake cell within [`LAKE_TOL_M`] of the same level
/// (the flood raises a lake's cells by ε steps towards its spill, `pflood.rs`, so several of them
/// can drain over the spill itself).
fn lake_links(r: &Routing, input: &SplInput<'_>) -> Vec<u8> {
    let lake = |c: usize| input.kind[c] != SINK && input.filled[c] - input.h0[c] > LAKE_TOL_M;
    r.stack
        .par_iter()
        .zip(r.forest.rpos.par_iter())
        .map(|(&c, &rp)| {
            let c = c as usize;
            if !lake(c) {
                return NOT_LAKE;
            }
            let rc = r.rcv[c] as usize;
            if rp != NONE
                && rc != c
                && input.kind[rc] != BASE
                && lake(rc)
                && (input.filled[c] - input.filled[rc]).abs() <= LAKE_TOL_M
            {
                LAKE_CHAIN
            } else if rp != NONE && rc != c && input.kind[rc] == LAND && !lake(rc) {
                LAKE_SPILL
            } else {
                LAKE_EXIT
            }
        })
        .collect()
}

pub(super) fn incise(b: f64, base: f64, f: f64, n: f64) -> f64 {
    if b <= base {
        return b;
    }
    if (n - 1.0).abs() < 1e-12 {
        return (b + f * base) / (1.0 + f);
    }
    // Newton on g(x) = x − b + F (x − base)^n, x in (base, b].
    let mut x = b;
    for _ in 0..50 {
        let dxb = (x - base).max(0.0);
        let g = x - b + f * dxb.powf(n);
        let gp = 1.0 + n * f * dxb.powf(n - 1.0);
        let next = (x - g / gp).clamp(base, b);
        if (next - x).abs() < 1e-9 {
            return next;
        }
        x = next;
    }
    x
}

/// Cells handed to another thread by the downstream pass (`Forest::down`): the solve costs a
/// Newton iteration (two `powf` calls per step) per cell, about 100 ns.
const DOWN_GRAIN: u32 = 256;
/// Leaves handed to one task by the upstream passes.
const UP_CHUNK: usize = 64;

/// Everything a cell needs from the rest of the solve, gathered once in stack order (dense, so
/// the passes walk contiguous memory). All values are the very expressions the serial loop
/// computed per iteration, hoisted out of it: `factor` (F of the module documentation) does not
/// change between iterations, so computing it once is bit-identical to computing it six times.
#[derive(Clone, Copy)]
struct Cell {
    h0: f64,
    filled: f64,
    /// Drainage area, floored at 1 % of a cell, m².
    ar: f64,
    /// The cell's area, m², and its share of the grid's reference area (exactly 1 on the
    /// raster, whose cells are all the reference): a lake's room is counted in metres over
    /// cells of the reference area.
    a: f64,
    wgt: f64,
    /// Distance to the receiver, metres (at least 1e-9).
    len: f64,
    /// F = K A^m Δt / L^n.
    factor: f64,
    /// Base level of a base-level receiver, or the filled level of a lake receiver.
    rfix: f64,
    /// 0: the receiver is a base-level cell, 1: a lake, 2: an ordinary cell.
    rmode: u8,
    /// Not a sink and not a lake: the cell incises (or aggrades).
    solved: bool,
    sink: bool,
}

/// Solves one step of fluvial erosion and deposition into `h` (which holds h* for base cells on
/// return, unchanged). `qin` returns the sediment inflow of every cell (m³/yr), so a base-level
/// cell's `qin` is what its basin delivers to it.
///
/// **Parallel and bit-identical to the serial loop.** The downstream pass (every cell after its
/// receiver) and the flux pass (every cell after its donors) run over the drainage forest with
/// `Forest::down` and `Forest::up`; the inflow of a cell is summed over its donors in ascending
/// order, which is the order the serial reverse-stack loop added them in (see `forest.rs`), so the
/// result does not depend on the thread count and equals the serial solve to the last bit.
pub fn solve<G: SurfaceGrid>(
    d: &G,
    r: &Routing,
    input: &SplInput<'_>,
    p: &SplParams,
    h: &mut [f64],
    qin: &mut [f64],
) -> SplReport {
    solve_cut(d, r, input, p, h, qin, None)
}

/// [`solve`], also returning in `cut` (when given; one value per cell, metres) how far the
/// critical slope cut each cell below the stream-power solution in the last iteration: the
/// landslide share of the step's erosion, for the surface stage's erosion diagnostics
/// (`ErosionBands`). The solution is the same bit for bit with or without it.
pub fn solve_cut<G: SurfaceGrid>(
    d: &G,
    r: &Routing,
    input: &SplInput<'_>,
    p: &SplParams,
    h: &mut [f64],
    qin: &mut [f64],
    cut: Option<&mut [f64]>,
) -> SplReport {
    // The unit of a lake's room (the raster's one cell area, the cube's mean). ledger: SCL-149
    let a = d.ref_area();
    let n = d.cells();
    let f = &r.forest;
    let l = f.len();
    h.copy_from_slice(input.h0);
    qin.iter_mut().for_each(|q| *q = 0.0);
    let kind = input.kind;
    let cells: Vec<Cell> = r
        .stack
        .par_iter()
        .map(|&c| {
            let c = c as usize;
            let h0 = input.h0[c];
            let rc = r.rcv[c] as usize;
            let (rmode, rfix) = if kind[rc] == BASE {
                (0, input.base_z[rc])
            } else if input.filled[rc] - input.h0[rc] > LAKE_TOL_M {
                (1, input.filled[rc])
            } else {
                (2, 0.0)
            };
            let ac = d.area(c);
            let ar = input.area[c].max(0.01 * ac);
            let len = f64::from(r.len[c]).max(1e-9);
            Cell {
                h0,
                filled: input.filled[c],
                ar,
                a: ac,
                wgt: ac / a,
                len,
                factor: input.kf[c] * ar.powf(p.m) * p.dt / len.powf(p.n),
                rfix,
                rmode,
                solved: kind[c] != SINK && input.filled[c] - h0 <= LAKE_TOL_M,
                sink: kind[c] == SINK,
            }
        })
        .collect();
    let links = lake_links(r, input);
    let hs = SharedF64::new(l, 0.0);
    (0..l).into_par_iter().for_each(|i| hs.set(i, cells[i].h0));
    let qs = SharedF64::new(l, 0.0);
    let out = SharedF64::new(l, 0.0);
    // A lake's room upstream of each of its cells (m × cells), and the share of it its exit fills.
    let room = SharedF64::new(l, 0.0);
    let share = SharedF64::new(l, 0.0);
    let delta = SharedF64::new(l, 0.0);
    let cuts = cut.as_ref().map(|_| SharedF64::new(l, 0.0));
    let mut report = SplReport::default();
    for it in 0..p.max_iter.max(1) {
        // Downstream first: every receiver is solved before its donors. Sinks and lakes only
        // take sediment: the flux pass fills them.
        f.down(DOWN_GRAIN, |i| {
            let c = &cells[i];
            if !c.solved {
                return;
            }
            let before = hs.get(i);
            let rp = f.rpos[i];
            debug_assert!(c.rmode == 0 || rp != NONE);
            let base = match c.rmode {
                0 => c.rfix,
                1 => hs.get(rp as usize).max(c.rfix),
                _ => hs.get(rp as usize),
            };
            let b = c.h0 + p.g * qs.get(i) / c.ar * p.dt;
            let mut x = incise(b, base, c.factor, p.n);
            if let Some(cs) = &cuts {
                cs.set(i, (x - (base + p.sc * c.len)).max(0.0));
            }
            if x > base + p.sc * c.len {
                x = base + p.sc * c.len;
            }
            hs.set(i, x);
            delta.set(i, (x - before).abs());
        });
        // The change is the largest one of a cell that was solved, as in the serial loop (cells
        // that were not keep h, whose change is 0).
        let change = delta.max().max(0.0);
        // Upstream first: the fluxes of this solution; sinks keep all they get, a lake keeps what
        // reaches it up to its room (its exit settles it over the whole lake, below), and
        // deposition is capped by the supply.
        f.up(UP_CHUNK, |i| {
            let mut inflow = 0.0;
            let mut upstream = 0.0;
            let (mut lake_in, mut lake_room) = (0.0, 0.0);
            for &dp in &f.dpos[f.dptr[i] as usize..f.dptr[i + 1] as usize] {
                let o = out.get(dp as usize);
                inflow += o;
                match links[dp as usize] {
                    LAKE_CHAIN => upstream += room.get(dp as usize),
                    LAKE_SPILL => {
                        lake_in += o;
                        lake_room += room.get(dp as usize);
                    }
                    _ => {}
                }
            }
            if lake_room > 0.0 {
                // The spill of a lake: the lake keeps what reaches it up to its room.
                // ledger: HYD-110
                let dep = (lake_in * p.dt / a).min(lake_room);
                share.set(i, dep / lake_room);
                inflow = (inflow - dep * a / p.dt).max(0.0);
            }
            qs.set(i, inflow);
            let c = &cells[i];
            // The inflow over this cell, m; and over the reference area, m (the room's unit).
            let supply = inflow * p.dt / c.a;
            let o = if c.sink {
                hs.set(i, c.h0 + supply);
                0.0
            } else if links[i] == LAKE_CHAIN || links[i] == LAKE_SPILL {
                // ledger: HYD-110
                room.set(i, upstream + (c.filled - c.h0) * c.wgt);
                inflow
            } else if links[i] == LAKE_EXIT {
                let v = upstream + (c.filled - c.h0) * c.wgt;
                let dep = (inflow * p.dt / a).min(v);
                share.set(i, if v > 0.0 { dep / v } else { 0.0 });
                (inflow - dep * a / p.dt).max(0.0)
            } else {
                let o = inflow + (c.h0 - hs.get(i)) * c.a / p.dt;
                if o < 0.0 {
                    hs.set(i, c.h0 + supply);
                    0.0
                } else {
                    o
                }
            };
            out.set(i, o);
        });
        report.iterations = it + 1;
        report.residual_m = if it == 0 { f64::INFINITY } else { change };
        if it > 0 && change < p.tol_m {
            break;
        }
        if p.g == 0.0 {
            // Without deposition the first pass is already the solution (no flux feedback).
            report.residual_m = 0.0;
            break;
        }
    }
    // Each lake's exit settles what the lake kept over all of it, every cell raised by the same
    // share of its depth (the deep parts take the most: sediment focusing). ledger: HYD-110
    f.down(DOWN_GRAIN, |i| {
        let c = &cells[i];
        match links[i] {
            LAKE_CHAIN | LAKE_SPILL => {
                let s = share.get(f.rpos[i] as usize);
                share.set(i, s);
                hs.set(i, c.h0 + s * (c.filled - c.h0));
            }
            LAKE_EXIT => hs.set(i, c.h0 + share.get(i) * (c.filled - c.h0)),
            _ => {}
        }
    });
    for (i, &c) in r.stack.iter().enumerate() {
        h[c as usize] = hs.get(i);
        qin[c as usize] = qs.get(i);
    }
    if let (Some(out), Some(cs)) = (cut, &cuts) {
        out.iter_mut().for_each(|x| *x = 0.0);
        for (i, &c) in r.stack.iter().enumerate() {
            // A cell the flux pass refilled (deposition) lost nothing to the cut.
            let lost = (input.h0[c as usize] - h[c as usize]).max(0.0);
            out[c as usize] = cs.get(i).min(lost);
        }
    }
    // What each basin delivers to its base-level cell.
    for (k, &o) in f.outlets.iter().enumerate() {
        let mut inflow = 0.0;
        for &dp in &f.opos[f.optr[k] as usize..f.optr[k + 1] as usize] {
            inflow += out.get(dp as usize);
        }
        qin[o as usize] = inflow;
    }
    debug_assert_eq!(h.len(), n);
    report
}

/// The source-rock mix of the sediment flux: for every cell, the flux entering it split by the
/// family of the rock it came from (`fam[c]` of each eroding cell), m³/yr, after a [`solve`].
/// Deposition takes from the entering flux in proportion; a lake's cells all take the mix that
/// enters the lake, and its exit takes what the whole lake kept (HYD-110). Parallel over the
/// drainage forest and bit-identical to the serial reverse-stack loop, like [`solve`].
pub fn families<G: SurfaceGrid>(
    d: &G,
    r: &Routing,
    input: &SplInput<'_>,
    fam: &[u8],
    dt: f64,
    h: &[f64],
    mix: &mut Vec<[f64; FAMILIES]>,
) {
    let f = &r.forest;
    let l = f.len();
    // The erosion rate of each cell (negative: deposition) and its family, in stack order.
    let ev: Vec<f64> = r
        .stack
        .par_iter()
        .map(|&c| (input.h0[c as usize] - h[c as usize]) * d.area(c as usize) / dt)
        .collect();
    let fv: Vec<usize> = r
        .stack
        .par_iter()
        .map(|&c| usize::from(fam[c as usize]).min(FAMILIES - 1))
        .collect();
    let links = lake_links(r, input);
    let out = SharedF64::new(l * FAMILIES, 0.0);
    let fin = SharedF64::new(l * FAMILIES, 0.0);
    // What a lake kept upstream of each of its cells, m³/yr.
    let kept = SharedF64::new(l, 0.0);
    f.up(UP_CHUNK, |i| {
        let mut inflow = [0.0f64; FAMILIES];
        let mut upstream = 0.0;
        let mut spilled = 0.0;
        for &dp in &f.dpos[f.dptr[i] as usize..f.dptr[i + 1] as usize] {
            for (k, x) in inflow.iter_mut().enumerate() {
                *x += out.get(dp as usize * FAMILIES + k);
            }
            match links[dp as usize] {
                LAKE_CHAIN => upstream += kept.get(dp as usize),
                LAKE_SPILL => spilled += kept.get(dp as usize),
                _ => {}
            }
        }
        for (k, &x) in inflow.iter().enumerate() {
            fin.set(i * FAMILIES + k, x);
        }
        let mut qin: f64 = inflow.iter().sum();
        if spilled > 0.0 && qin > 0.0 {
            // The lakes spilling here kept their share of what entered them.
            let keep = ((qin - spilled) / qin).clamp(0.0, 1.0);
            inflow.iter_mut().for_each(|x| *x *= keep);
            qin = inflow.iter().sum();
        }
        let e = match links[i] {
            LAKE_CHAIN | LAKE_SPILL => {
                kept.set(i, upstream + (-ev[i]).max(0.0));
                0.0
            }
            LAKE_EXIT => ev[i].min(0.0) - upstream,
            _ => ev[i],
        };
        let mut o = inflow;
        if e > 0.0 {
            o[fv[i]] += e;
        } else if qin > 0.0 {
            let keep = ((qin + e) / qin).clamp(0.0, 1.0);
            o.iter_mut().for_each(|x| *x *= keep);
        }
        for (k, &x) in o.iter().enumerate() {
            out.set(i * FAMILIES + k, x);
        }
    });
    // A lake's cells take the mix entering it at its exit (or its spill).
    f.down(DOWN_GRAIN, |i| {
        if links[i] == LAKE_CHAIN || links[i] == LAKE_SPILL {
            let rp = f.rpos[i] as usize;
            for k in 0..FAMILIES {
                fin.set(i * FAMILIES + k, fin.get(rp * FAMILIES + k));
            }
        }
    });
    mix.clear();
    mix.resize(d.cells(), [0.0; FAMILIES]);
    for (i, &c) in r.stack.iter().enumerate() {
        for (k, x) in mix[c as usize].iter_mut().enumerate() {
            *x = fin.get(i * FAMILIES + k);
        }
    }
    for (j, &o) in f.outlets.iter().enumerate() {
        let mut inflow = [0.0f64; FAMILIES];
        for &dp in &f.opos[f.optr[j] as usize..f.optr[j + 1] as usize] {
            for (k, x) in inflow.iter_mut().enumerate() {
                *x += out.get(dp as usize * FAMILIES + k);
            }
        }
        mix[o as usize] = inflow;
    }
}

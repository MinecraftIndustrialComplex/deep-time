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
//! - Lake cells (filled by the priority flood by more than [`LAKE_TOL_M`]) do not incise: they
//!   trap the sediment that reaches them, up to the lake level, and pass the rest on. A cell that
//!   drains into a lake erodes towards the lake surface.
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

use super::grid::Dims;
use super::pflood::{BASE, SINK};
use super::routing::Routing;

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

fn incise(b: f64, base: f64, f: f64, n: f64) -> f64 {
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

/// Solves one step of fluvial erosion and deposition into `h` (which holds h* for base cells on
/// return, unchanged). `qin` returns the sediment inflow of every cell (m³/yr), so a base-level
/// cell's `qin` is what its basin delivers to it.
pub fn solve(
    d: &Dims,
    r: &Routing,
    input: &SplInput<'_>,
    p: &SplParams,
    h: &mut [f64],
    qin: &mut [f64],
) -> SplReport {
    let a = d.cell_area();
    let n = d.cells();
    h.copy_from_slice(input.h0);
    qin.iter_mut().for_each(|q| *q = 0.0);
    let mut report = SplReport::default();
    let kind = input.kind;
    for it in 0..p.max_iter.max(1) {
        let mut change: f64 = 0.0;
        // Downstream first: every receiver is solved before its donors.
        for &c in &r.stack {
            let c = c as usize;
            let before = h[c];
            let h0 = input.h0[c];
            // Sinks and lakes only take sediment: the flux pass fills them.
            if kind[c] != SINK && input.filled[c] - h0 <= LAKE_TOL_M {
                let rc = r.rcv[c] as usize;
                let base = if kind[rc] == BASE {
                    input.base_z[rc]
                } else if input.filled[rc] - input.h0[rc] > LAKE_TOL_M {
                    h[rc].max(input.filled[rc])
                } else {
                    h[rc]
                };
                let ar = input.area[c].max(0.01 * a);
                let len = f64::from(r.len[c]).max(1e-9);
                let b = h0 + p.g * qin[c] / ar * p.dt;
                let f = input.kf[c] * ar.powf(p.m) * p.dt / len.powf(p.n);
                let mut x = incise(b, base, f, p.n);
                if x > base + p.sc * len {
                    x = base + p.sc * len;
                }
                h[c] = x;
            }
            change = change.max((h[c] - before).abs());
        }
        // Upstream first: the fluxes of this solution; sinks keep all they get, lakes fill up to
        // their level, and deposition is capped by the supply.
        qin.iter_mut().for_each(|q| *q = 0.0);
        for &c in r.stack.iter().rev() {
            let c = c as usize;
            let h0 = input.h0[c];
            let supply = qin[c] * p.dt / a;
            let out = if kind[c] == SINK {
                h[c] = h0 + supply;
                0.0
            } else if input.filled[c] - h0 > LAKE_TOL_M {
                let dep = supply.min(input.filled[c] - h0);
                h[c] = h0 + dep;
                (qin[c] - dep * a / p.dt).max(0.0)
            } else {
                let out = qin[c] + (h0 - h[c]) * a / p.dt;
                if out < 0.0 {
                    h[c] = h0 + supply;
                    0.0
                } else {
                    out
                }
            };
            let rc = r.rcv[c] as usize;
            if rc != c {
                qin[rc] += out;
            }
        }
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
    debug_assert_eq!(h.len(), n);
    report
}

/// The source-rock mix of the sediment flux: for every cell, the flux entering it split by the
/// family of the rock it came from (`fam[c]` of each eroding cell), m³/yr, after a [`solve`].
/// Deposition takes from the entering flux in proportion.
pub fn families(
    d: &Dims,
    r: &Routing,
    input: &SplInput<'_>,
    fam: &[u8],
    dt: f64,
    h: &[f64],
    mix: &mut Vec<[f64; FAMILIES]>,
) {
    let a = d.cell_area();
    mix.clear();
    mix.resize(d.cells(), [0.0; FAMILIES]);
    for &c in r.stack.iter().rev() {
        let c = c as usize;
        let fin = mix[c];
        let qin: f64 = fin.iter().sum();
        let e = (input.h0[c] - h[c]) * a / dt;
        let mut out = fin;
        if e > 0.0 {
            out[usize::from(fam[c]).min(FAMILIES - 1)] += e;
        } else if qin > 0.0 {
            let keep = ((qin + e) / qin).clamp(0.0, 1.0);
            out.iter_mut().for_each(|x| *x *= keep);
        }
        let rc = r.rcv[c] as usize;
        if rc != c {
            for k in 0..FAMILIES {
                mix[rc][k] += out[k];
            }
        }
    }
}

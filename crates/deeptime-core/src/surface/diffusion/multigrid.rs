// SPDX-License-Identifier: LGPL-3.0-or-later
//! Conjugate gradients preconditioned by one multigrid V-cycle (P1-19, item 25 of the gate
//! profile): the same system (I + Δt·L) h¹ = h⁰ as the Jacobi-preconditioned solve
//! ([`super::cg`]), the same stopping rule (RMS residual), the same return of the residual's sum.
//!
//! **Why.** The Jacobi preconditioner leaves the condition number at about 1 + 8·c_max with
//! c = D·Δt/Δx²: on the shelves (D₀ = 100 m²/yr over 2 Myr on 2.4-km cells) c reaches ~35, and the
//! solve took ~111 iterations a step. A multigrid cycle removes the smooth error that the diagonal
//! cannot (Briggs, Henson & McCormick 2000, *A Multigrid Tutorial*, 2nd ed., SIAM: ch. 2–3 for
//! the smoothing and coarse-grid principle; Tatebe 1993, "The multigrid preconditioned conjugate
//! gradient method", Proc. 6th Copper Mountain Conference on Multigrid Methods, NASA CP-3224,
//! 621–634, for the cycle as a preconditioner, which must then be symmetric and positive
//! definite).
//!
//! **The cycle.** Cell-centred levels, each cell of a coarse level the 2 × 2 block of the level
//! below (the last column or row alone when the count is odd; x keeps wrapping). Restriction sums
//! a block's residuals, prolongation hands a block's correction to each of its cells (R = Pᵀ,
//! piecewise constant). A coarse level is the same kind of operator: its "mass" is the sum of the
//! block's (the identity's share of I + Δt·L) and the coupling across a coarse face is
//! [`SCALE`] times the sum of the fine couplings across it; [`SCALE`] = 1 is the Galerkin
//! operator PᵀAP, which piecewise-constant transfer makes about twice too stiff for smooth
//! error at each level (Braess 1995, "Towards algebraic multigrid for elliptic problems of second
//! order", Computing 55:379–393, whose remedy is to over-weight the correction), and ½ is the
//! operator rediscretised on the coarse cells. Smoothing is damped Jacobi, z ← z + ω D⁻¹ (r − A z)
//! (ω = [`OMEGA`], [`NU`] sweeps before and after the coarse correction, the first from z = 0),
//! which is symmetric, so pre- and post-smoothing alike make the cycle a symmetric positive
//! definite operator for any coarse operator that is itself one; the coarsest level (≤ [`COARSEST`]
//! cells) takes [`COARSE_SWEEPS`] sweeps. Every level of the cycle runs in `f32` (the
//! preconditioner need not be exact; the conjugate gradients, the operator they apply and the
//! residual they stop on stay `f64`), which halves its memory traffic.
//!
//! The result is not bit-identical to the Jacobi solve (another sequence of iterates stopping at
//! the same residual), and since 2026-10-18 neither is required (Ben: "I am not worried about
//! determinism, performance is far more important"); its reductions are still taken per strip of
//! rows and added in order, so it is the same at any thread count.

use rayon::prelude::*;

use super::{Live, Operator, STRIP_ROWS, Workspace, sum_zero};
use crate::surface::grid::Dims;

/// Damping of the Jacobi smoother (4/5 is the best smoothing factor for the five-point Laplacian
/// in two dimensions; Briggs et al. 2000, ch. 2, give 2/3 for one dimension). The tuning sweep
/// (`tune_the_cycle`, 2026-10-18, 2048 x 1024 shelf at 8 threads) found 0.8 fastest of 0.6 to 1.0.
pub(super) const OMEGA: f32 = 0.8; // ledger: ERO-183
/// Smoothing sweeps before and after each coarse correction: two took 6 to 8 outer iterations
/// where one took 10 to 13 and were 10 to 20 % faster in the same sweep.
pub(super) const NU: usize = 2; // ledger: ERO-183
/// Coarse couplings as a share of the fine couplings summed across the coarse face (1/2: the
/// piecewise-constant aggregation's Galerkin operator is about twice too stiff for smooth error,
/// Braess 1995; the sweep found 0.5 to 0.6 fastest of 0.5 to 1.0).
pub(super) const SCALE: f32 = 0.5; // ledger: ERO-183
/// A level this small is the coarsest.
const COARSEST: usize = 64; // ledger: ERO-183
/// Sweeps on the coarsest level.
const COARSE_SWEEPS: usize = 16; // ledger: ERO-183

/// The cycle's settings (the tests vary them).
#[derive(Clone, Copy, Debug)]
pub(super) struct Cycle {
    pub omega: f32,
    pub nu: usize,
    pub scale: f32,
}

pub(super) const CYCLE: Cycle = Cycle {
    omega: OMEGA,
    nu: NU,
    scale: SCALE,
};

/// One level: the five-point operator in `f32` and the cycle's vectors.
struct Level {
    w: usize,
    h: usize,
    wrap: bool,
    east: Vec<f32>,
    south: Vec<f32>,
    diag: Vec<f32>,
    /// ω / diag.
    wd: Vec<f32>,
    /// The identity's share of the diagonal (cells of the finest level per cell).
    mass: Vec<f32>,
    r: Vec<f32>,
    z: Vec<f32>,
    t: Vec<f32>,
}

/// Rows per task on a level of width `w`: a task covers at least ~16k cells, so the small levels
/// run as one task instead of paying the pool for each row.
fn min_rows(w: usize) -> usize {
    (16_384 / w.max(1)).max(1)
}

impl Level {
    fn finest(op: &Operator, omega: f32) -> Level {
        let n = op.w * op.h;
        let to32 = |v: &[f64]| -> Vec<f32> { v.par_iter().map(|&x| x as f32).collect() };
        let diag = to32(op.diag);
        let wd = diag.par_iter().map(|&g| omega / g).collect();
        Level {
            w: op.w,
            h: op.h,
            wrap: op.wrap,
            east: to32(op.east),
            south: to32(op.south),
            diag,
            wd,
            mass: Vec::new(),
            r: vec![0.0; n],
            z: vec![0.0; n],
            t: vec![0.0; n],
        }
    }

    /// The next coarser level (module notes), or `None` when this one is the coarsest.
    fn coarsen(&self, cy: &Cycle) -> Option<Level> {
        let (w, h) = (self.w, self.h);
        let (wc, hc) = (w.div_ceil(2), h.div_ceil(2));
        if w * h <= COARSEST || (wc == w && hc == h) {
            return None;
        }
        let nc = wc * hc;
        let wrap = self.wrap && wc > 1;
        let fine_mass = |c: usize| -> f32 {
            if self.mass.is_empty() {
                1.0
            } else {
                self.mass[c]
            }
        };
        let mut east = vec![0.0f32; nc];
        let mut south = vec![0.0f32; nc];
        let mut mass = vec![0.0f32; nc];
        east.par_chunks_mut(wc)
            .zip(south.par_chunks_mut(wc))
            .zip(mass.par_chunks_mut(wc))
            .enumerate()
            .for_each(|(jc, ((er, sr), mr))| {
                let rows = (2 * jc)..(2 * jc + 2).min(h);
                let last_row = rows.end - 1;
                for ic in 0..wc {
                    let cols = (2 * ic)..(2 * ic + 2).min(w);
                    let last_col = cols.end - 1;
                    let mut m = 0.0f32;
                    for j in rows.clone() {
                        for i in cols.clone() {
                            m += fine_mass(j * w + i);
                        }
                    }
                    mr[ic] = m;
                    // The east face of the block: the fine faces leaving its last column, which
                    // reach the next block (or wrap to the first; none on a ring of one block).
                    if ic + 1 < wc || wrap {
                        let mut e = 0.0f32;
                        for j in rows.clone() {
                            e += self.east[j * w + last_col];
                        }
                        er[ic] = cy.scale * e;
                    }
                    if jc + 1 < hc {
                        let mut s = 0.0f32;
                        for i in cols.clone() {
                            s += self.south[last_row * w + i];
                        }
                        sr[ic] = cy.scale * s;
                    }
                }
            });
        let mut diag = vec![0.0f32; nc];
        diag.par_chunks_mut(wc).enumerate().for_each(|(j, dr)| {
            let c0 = j * wc;
            for i in 0..wc {
                let west = if i > 0 {
                    east[c0 + i - 1]
                } else if wrap {
                    east[c0 + wc - 1]
                } else {
                    0.0
                };
                let north = if j > 0 { south[c0 - wc + i] } else { 0.0 };
                dr[i] = mass[c0 + i] + east[c0 + i] + west + south[c0 + i] + north;
            }
        });
        let wd = diag.par_iter().map(|&g| cy.omega / g).collect();
        Some(Level {
            w: wc,
            h: hc,
            wrap,
            east,
            south,
            diag,
            wd,
            mass,
            r: vec![0.0; nc],
            z: vec![0.0; nc],
            t: vec![0.0; nc],
        })
    }

    /// Row `j` of A·x into `out`, from rows j − 1, j, j + 1 of x.
    #[inline]
    fn apply_row(
        &self,
        j: usize,
        north: Option<&[f32]>,
        cur: &[f32],
        south: Option<&[f32]>,
        out: &mut [f32],
    ) {
        let w = self.w;
        let c0 = j * w;
        let diag = &self.diag[c0..c0 + w];
        let east = &self.east[c0..c0 + w];
        for ((o, &g), &x) in out.iter_mut().zip(diag).zip(cur) {
            *o = g * x;
        }
        for ((o, &e), &x) in out.iter_mut().zip(east).zip(&cur[1..]) {
            *o -= e * x;
        }
        for ((o, &e), &x) in out[1..].iter_mut().zip(east).zip(cur) {
            *o -= e * x;
        }
        if self.wrap {
            out[w - 1] -= east[w - 1] * cur[0];
            out[0] -= east[w - 1] * cur[w - 1];
        }
        if let Some(s) = south {
            let srow = &self.south[c0..c0 + w];
            for ((o, &c), &x) in out.iter_mut().zip(srow).zip(s) {
                *o -= c * x;
            }
        }
        if let Some(nrow) = north {
            let above = &self.south[c0 - w..c0];
            for ((o, &c), &x) in out.iter_mut().zip(above).zip(nrow) {
                *o -= c * x;
            }
        }
    }

    /// z = ω D⁻¹ r (the first sweep, from z = 0).
    fn first_sweep(&mut self) {
        let w = self.w;
        let (r, wd) = (&self.r, &self.wd);
        self.z
            .par_chunks_mut(w)
            .with_min_len(min_rows(w))
            .enumerate()
            .for_each(|(j, zr)| {
                let c0 = j * w;
                for ((z, &rv), &k) in zr.iter_mut().zip(&r[c0..c0 + w]).zip(&wd[c0..c0 + w]) {
                    *z = k * rv;
                }
            });
    }

    /// One sweep z ← z' + ω D⁻¹ (r − A z') with z' = z + P·`coarse` (the coarse level's
    /// correction, when given). With `dot`, also returns Σ dot·z (the new z).
    fn sweep(&mut self, coarse: Option<(&[f32], usize)>, dot: Option<&[f64]>) -> f64 {
        let (w, h) = (self.w, self.h);
        let mut t = std::mem::take(&mut self.t);
        let sums: Vec<f64> = {
            let me = &*self;
            let z = &me.z;
            // Row j of z', into `out`.
            let corrected = |j: usize, out: &mut [f32]| {
                let zr = &z[j * w..(j + 1) * w];
                match coarse {
                    Some((zc, wc)) => {
                        let crow = &zc[(j / 2) * wc..(j / 2 + 1) * wc];
                        for (i, (o, &v)) in out.iter_mut().zip(zr).enumerate() {
                            *o = v + crow[i / 2];
                        }
                    }
                    None => out.copy_from_slice(zr),
                }
            };
            t.par_chunks_mut(STRIP_ROWS * w)
                .with_min_len(min_rows(w).div_ceil(STRIP_ROWS))
                .enumerate()
                .map_init(
                    || {
                        [
                            vec![0.0f32; w],
                            vec![0.0f32; w],
                            vec![0.0f32; w],
                            vec![0.0f32; w],
                        ]
                    },
                    |bufs, (s, ts)| {
                        let rows = ts.len() / w;
                        let j0 = s * STRIP_ROWS;
                        let [ring0, ring1, ring2, ax] = bufs;
                        if j0 > 0 {
                            corrected(j0 - 1, ring0);
                        }
                        corrected(j0, ring1);
                        let mut acc = sum_zero();
                        for jj in 0..rows {
                            let j = j0 + jj;
                            if j + 1 < h {
                                corrected(j + 1, ring2);
                            }
                            let north = (j > 0).then_some(&ring0[..]);
                            let south = (j + 1 < h).then_some(&ring2[..]);
                            me.apply_row(j, north, ring1, south, ax);
                            let c0 = j * w;
                            let tr = &mut ts[jj * w..(jj + 1) * w];
                            let (rr, wd) = (&me.r[c0..c0 + w], &me.wd[c0..c0 + w]);
                            for i in 0..w {
                                tr[i] = ring1[i] + wd[i] * (rr[i] - ax[i]);
                            }
                            if let Some(dv) = dot {
                                for (&a, &b) in dv[c0..c0 + w].iter().zip(tr.iter()) {
                                    acc += a * f64::from(b);
                                }
                            }
                            std::mem::swap(ring0, ring1);
                            std::mem::swap(ring1, ring2);
                        }
                        acc
                    },
                )
                .collect()
        };
        self.t = std::mem::replace(&mut self.z, t);
        sums.iter().sum()
    }

    /// The coarse level's right-hand side: Pᵀ (r − A z), the residual summed over each block.
    fn restrict_into(&self, coarse: &mut Level) {
        let (w, h) = (self.w, self.h);
        let wc = coarse.w;
        let z = &self.z;
        coarse
            .r
            .par_chunks_mut(wc)
            .with_min_len(min_rows(wc))
            .enumerate()
            .for_each_init(
                || vec![0.0f32; w],
                |res, (jc, rc)| {
                    rc.iter_mut().for_each(|v| *v = 0.0);
                    for j in (2 * jc)..(2 * jc + 2).min(h) {
                        let north = (j > 0).then(|| &z[(j - 1) * w..j * w]);
                        let south = (j + 1 < h).then(|| &z[(j + 1) * w..(j + 2) * w]);
                        self.apply_row(j, north, &z[j * w..(j + 1) * w], south, res);
                        let rr = &self.r[j * w..(j + 1) * w];
                        for i in 0..w {
                            rc[i / 2] += rr[i] - res[i];
                        }
                    }
                },
            );
    }
}

/// What the cycle runs on besides the CPU: a wgpu device with the `gpu` feature, nothing
/// without it.
#[cfg(feature = "gpu")]
pub(super) type Device = deeptime_gpu::Gpu;
#[cfg(not(feature = "gpu"))]
pub(super) type Device = std::convert::Infallible;

/// In `auto` mode a grid smaller than this keeps its cycle on the CPU: a cycle's fixed cost on a
/// device (its ~60 dispatches, a submission and a read-back) is about what the CPU cycle of a
/// grid this size takes. A forced `gpu` mode uses the device at any size (the tests on lavapipe).
/// game-choice: engineering threshold, not physical (gpu-phase3, 2026-10-19).
#[cfg(feature = "gpu")]
const DEVICE_MIN_CELLS: usize = 1 << 17;

/// The session's device for a solve of `n` cells, if it should take the cycle (`compute.rs`).
#[cfg(feature = "gpu")]
fn session_device(n: usize) -> Option<std::sync::Arc<Device>> {
    let a = crate::compute::ambient()?;
    if a.mode() == crate::compute::ComputeMode::Auto && n < DEVICE_MIN_CELLS {
        return None;
    }
    crate::compute::ambient_device(gpu::KERNEL)
}

/// The preconditioner of one solve: the CPU's levels, or the device's. A device that fails hands
/// the rest of the solve to a CPU cycle built then (from the same operator and residual).
enum Pre {
    Cpu(Hierarchy),
    #[cfg(feature = "gpu")]
    Device(Box<gpu::DeviceCycle>),
}

impl Pre {
    fn new(op: &Operator, cy: Cycle, device: Option<&std::sync::Arc<Device>>) -> Pre {
        #[cfg(feature = "gpu")]
        if let Some(g) = device {
            let started = std::time::Instant::now();
            match gpu::DeviceCycle::new(g, op, cy) {
                Ok(dc) => {
                    crate::compute::lap("mg_setup_device", started);
                    return Pre::Device(Box::new(dc));
                }
                Err(e) => device_failed(&e),
            }
        }
        #[cfg(not(feature = "gpu"))]
        if let Some(d) = device {
            match **d {}
        }
        let started = std::time::Instant::now();
        let h = Hierarchy::new(op, cy);
        crate::compute::lap("mg_setup", started);
        Pre::Cpu(h)
    }

    /// The finest level's `f32` residual, which the caller sets before [`Pre::precondition`].
    fn r32(&mut self) -> &mut [f32] {
        match self {
            Pre::Cpu(h) => &mut h.levels[0].r,
            #[cfg(feature = "gpu")]
            Pre::Device(d) => &mut d.r32,
        }
    }

    fn z(&self) -> &[f32] {
        match self {
            Pre::Cpu(h) => h.z(),
            #[cfg(feature = "gpu")]
            Pre::Device(d) => &d.z32,
        }
    }

    /// z = M⁻¹ r; returns Σ `r64`·z.
    fn precondition(&mut self, op: &Operator, cy: Cycle, r64: &[f64]) -> f64 {
        #[cfg(not(feature = "gpu"))]
        let _ = (op, cy);
        let started = std::time::Instant::now();
        match self {
            Pre::Cpu(h) => {
                let v = h.precondition(r64);
                crate::compute::lap("mg_cycle", started);
                v
            }
            #[cfg(feature = "gpu")]
            Pre::Device(d) => match d.precondition(r64) {
                Ok(v) => {
                    crate::compute::lap("mg_cycle_device", started);
                    v
                }
                Err(e) => {
                    device_failed(&e);
                    let mut h = Hierarchy::new(op, cy);
                    h.levels[0].r.copy_from_slice(&d.r32);
                    *self = Pre::Cpu(h);
                    self.precondition(op, cy, r64)
                }
            },
        }
    }
}

/// Says once per session that the device failed and the cycle runs on the CPU (counted).
#[cfg(feature = "gpu")]
fn device_failed(e: &str) {
    let msg = format!(
        "compute: {} failed on the GPU, the CPU runs the cycle ({e})",
        gpu::KERNEL
    );
    match crate::compute::ambient() {
        Some(a) => a.tell_once("surface:diffusion failed", crate::log::Level::Warn, msg),
        None => eprintln!("{msg}"),
    }
    crate::compute::add("mg_fallback_n", 1.0);
}

/// The level hierarchy of one solve.
struct Hierarchy {
    levels: Vec<Level>,
    cy: Cycle,
}

impl Hierarchy {
    fn new(op: &Operator, cy: Cycle) -> Hierarchy {
        let mut levels = vec![Level::finest(op, cy.omega)];
        while let Some(next) = levels.last().and_then(|l| l.coarsen(&cy)) {
            levels.push(next);
        }
        Hierarchy { levels, cy }
    }

    /// z = M⁻¹ r on the finest level, whose `r` the caller has set; returns Σ `r64`·z.
    fn precondition(&mut self, r64: &[f64]) -> f64 {
        vcycle(&mut self.levels, &self.cy, Some(r64))
    }

    fn z(&self) -> &[f32] {
        &self.levels[0].z
    }
}

/// One V-cycle from z = 0 on `levels[0]`; with `dot`, returns Σ dot·z.
fn vcycle(levels: &mut [Level], cy: &Cycle, dot: Option<&[f64]>) -> f64 {
    let (l, rest) = levels.split_first_mut().expect("a level");
    if rest.is_empty() {
        l.first_sweep();
        let mut acc = 0.0;
        for k in 1..COARSE_SWEEPS.max(2) {
            let last = k + 1 == COARSE_SWEEPS.max(2);
            acc = l.sweep(None, if last { dot } else { None });
        }
        return acc;
    }
    l.first_sweep();
    for _ in 1..cy.nu {
        l.sweep(None, None);
    }
    l.restrict_into(&mut rest[0]);
    vcycle(rest, cy, None);
    let wc = rest[0].w;
    let mut acc = l.sweep(Some((&rest[0].z, wc)), if cy.nu == 1 { dot } else { None });
    for k in 1..cy.nu {
        acc = l.sweep(None, if k + 1 == cy.nu { dot } else { None });
    }
    acc
}

/// Refines `x` to the solution of A·x = b (b: the workspace's right-hand side, which the caller
/// has set and sized; [`Workspace::fit`] has sized the rest) to an RMS residual of `tol_m`, by
/// conjugate gradients preconditioned with one V-cycle, then puts the residual's sum back so
/// Σx = Σb; returns the iterations. The `f64` vectors are the workspace's (each written before it
/// is read); the cycle's levels are built for the solve.
pub(super) fn cg(
    ws: &mut Workspace,
    d: &Dims,
    x: &mut [f64],
    kd: &[f64],
    dt: f64,
    tol_m: f64,
) -> usize {
    #[cfg(feature = "gpu")]
    let device = session_device(d.cells());
    #[cfg(not(feature = "gpu"))]
    let device: Option<std::sync::Arc<Device>> = None;
    cg_on(ws, d, x, kd, dt, tol_m, CYCLE, device.as_ref())
}

/// [`cg`] with the cycle's settings given and the cycle on the CPU (the tests vary the cycle).
#[cfg(test)]
pub(super) fn cg_with(
    ws: &mut Workspace,
    d: &Dims,
    x: &mut [f64],
    kd: &[f64],
    dt: f64,
    tol_m: f64,
    cy: Cycle,
) -> usize {
    cg_on(ws, d, x, kd, dt, tol_m, cy, None)
}

/// [`cg`] with the cycle's settings and its device given (the one dispatch point of the device
/// cycle; None: the CPU's).
#[allow(clippy::too_many_arguments)]
pub(super) fn cg_on(
    ws: &mut Workspace,
    d: &Dims,
    x: &mut [f64],
    kd: &[f64],
    dt: f64,
    tol_m: f64,
    cy: Cycle,
    device: Option<&std::sync::Arc<Device>>,
) -> usize {
    let n = d.cells();
    let (w, h) = (d.w, d.h);
    if n == 0 {
        return 0;
    }
    let Workspace {
        b,
        east,
        south,
        diag,
        ap,
        r,
        pold,
        pnew,
        ..
    } = ws;
    let b: &[f64] = b;
    let live = Live::all(d);
    let op = Operator::new(d, kd, dt, &live, east, south, diag);
    let mut mg = Pre::new(&op, cy, device);
    // r = b − A·x, its f32 copy for the cycle, and r·r.
    let rr0: f64 = {
        let r32 = mg.r32();
        let xs: &[f64] = x;
        r.par_chunks_mut(STRIP_ROWS * w)
            .zip(r32.par_chunks_mut(STRIP_ROWS * w))
            .enumerate()
            .map(|(s, (rs, r3))| {
                let j0 = s * STRIP_ROWS;
                let mut acc = sum_zero();
                for jj in 0..rs.len() / w {
                    let j = j0 + jj;
                    let north = (j > 0).then(|| &xs[(j - 1) * w..j * w]);
                    let south = (j + 1 < h).then(|| &xs[(j + 1) * w..(j + 2) * w]);
                    let rrow = &mut rs[jj * w..(jj + 1) * w];
                    op.apply_row(j, north, &xs[j * w..(j + 1) * w], south, rrow);
                    for (i, (rv, &bv)) in rrow.iter_mut().zip(&b[j * w..(j + 1) * w]).enumerate() {
                        *rv = bv - *rv;
                        r3[jj * w + i] = *rv as f32;
                        acc += *rv * *rv;
                    }
                }
                acc
            })
            .collect::<Vec<f64>>()
            .iter()
            .sum()
    };
    let tol2 = tol_m * tol_m * n as f64;
    let mut iterations = 0;
    if rr0 > tol2 {
        let mut rz = mg.precondition(&op, cy, r);
        // The direction, two buffers: an iteration reads the last and writes the next. The first
        // direction is z itself, so the old one, which the workspace still holds from another
        // solve, is not read.
        let (mut pold, mut pnew) = (pold, pnew);
        let mut beta: Option<f64> = None;
        let mut pending: Option<f64> = None;
        while iterations < super::CG_MAX_ITER {
            iterations += 1;
            // p = z + β·p_old, A·p and p·A·p; and the last iteration's x += α·p_old.
            let pap: f64 = {
                let z = mg.z();
                let pold: &[f64] = pold;
                let fill = |j: usize, dst: &mut [f64]| {
                    let c0 = j * w;
                    let zr = &z[c0..c0 + w];
                    match beta {
                        Some(beta) => {
                            for ((d, &zv), &pv) in dst.iter_mut().zip(zr).zip(&pold[c0..c0 + w]) {
                                *d = f64::from(zv) + beta * pv;
                            }
                        }
                        None => {
                            for (d, &zv) in dst.iter_mut().zip(zr) {
                                *d = f64::from(zv);
                            }
                        }
                    }
                };
                pnew.par_chunks_mut(STRIP_ROWS * w)
                    .zip(ap.par_chunks_mut(STRIP_ROWS * w))
                    .zip(x.par_chunks_mut(STRIP_ROWS * w))
                    .enumerate()
                    .map_init(
                        || [vec![0.0f64; w], vec![0.0f64; w], vec![0.0f64; w]],
                        |ring, (s, ((pn, aps), xs))| {
                            let rows = pn.len() / w;
                            let j0 = s * STRIP_ROWS;
                            if let Some(alpha) = pending {
                                let c0 = j0 * w;
                                for (xv, &pv) in xs.iter_mut().zip(&pold[c0..c0 + rows * w]) {
                                    *xv += alpha * pv;
                                }
                            }
                            let [r0, r1, r2] = ring;
                            if j0 > 0 {
                                fill(j0 - 1, r0);
                            }
                            fill(j0, r1);
                            let mut acc = sum_zero();
                            for jj in 0..rows {
                                let j = j0 + jj;
                                if j + 1 < h {
                                    fill(j + 1, r2);
                                }
                                pn[jj * w..(jj + 1) * w].copy_from_slice(r1);
                                let north = (j > 0).then_some(&r0[..]);
                                let south = (j + 1 < h).then_some(&r2[..]);
                                let arow = &mut aps[jj * w..(jj + 1) * w];
                                op.apply_row(j, north, r1, south, arow);
                                for (&p, &a) in r1.iter().zip(arow.iter()) {
                                    acc += p * a;
                                }
                                std::mem::swap(r0, r1);
                                std::mem::swap(r1, r2);
                            }
                            acc
                        },
                    )
                    .collect::<Vec<f64>>()
                    .iter()
                    .sum()
            };
            std::mem::swap(&mut pold, &mut pnew);
            if pap <= 0.0 || !pap.is_finite() {
                break;
            }
            let alpha = rz / pap;
            pending = Some(alpha);
            // r −= α·A·p, its f32 copy, r·r.
            let rr: f64 = {
                let r32 = mg.r32();
                r.par_chunks_mut(STRIP_ROWS * w)
                    .zip(r32.par_chunks_mut(STRIP_ROWS * w))
                    .zip(ap.par_chunks(STRIP_ROWS * w))
                    .map(|((rs, r3), aps)| {
                        let mut acc = sum_zero();
                        for ((rv, r3v), &av) in rs.iter_mut().zip(r3.iter_mut()).zip(aps) {
                            *rv -= alpha * av;
                            *r3v = *rv as f32;
                            acc += *rv * *rv;
                        }
                        acc
                    })
                    .collect::<Vec<f64>>()
                    .iter()
                    .sum()
            };
            if rr <= tol2 {
                break;
            }
            let rz_new = mg.precondition(&op, cy, r);
            if !(rz_new.is_finite() && rz > 0.0) {
                break;
            }
            beta = Some(rz_new / rz);
            rz = rz_new;
        }
        if let Some(alpha) = pending {
            x.par_iter_mut()
                .zip(pold.par_iter())
                .for_each(|(xv, &pv)| *xv += alpha * pv);
        }
    }
    // Σx = Σb − Σr: the residual's sum back to the cells that diffuse, in proportion to their
    // coupling (as the Jacobi solve does).
    op.apply(x, ap, &live);
    let (lost, weight): (f64, f64) = b
        .par_chunks(w)
        .zip(ap.par_chunks(w))
        .zip(op.diag.par_chunks(w))
        .map(|((br, ar), gr)| {
            let mut l = sum_zero();
            let mut g = sum_zero();
            for ((p, q), d) in br.iter().zip(ar).zip(gr) {
                l += p - q;
                g += d - 1.0;
            }
            (l, g)
        })
        .collect::<Vec<(f64, f64)>>()
        .iter()
        .fold((sum_zero(), sum_zero()), |(a, b), &(l, g)| (a + l, b + g));
    if weight > 0.0 && lost != 0.0 {
        let k = lost / weight;
        x.par_iter_mut()
            .zip(op.diag.par_iter())
            .for_each(|(xv, g)| *xv += k * (g - 1.0));
    }
    iterations
}

#[cfg(feature = "gpu")]
mod gpu;

#[cfg(test)]
mod tests;

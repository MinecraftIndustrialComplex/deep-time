// SPDX-License-Identifier: LGPL-3.0-or-later
//! Implicit diffusion ∂h/∂t = ∇·(D ∇h) on the cube grid: the Lambert solve's scheme
//! (`surface/diffusion.rs`) on the cube's cells.
//!
//! **The system.** Backward Euler with lumped mass: for each cell c of area a_c,
//! a_c (x_c − b_c) + Δt Σ_m D_cm w_cm (x_c − x_m) = 0, with w the coupling Laplacian of the grid
//! (`CubeGrid::lap`: linear finite elements on the quads of cell centres, both diagonal cuts
//! averaged, so each cell couples to its four side and four diagonal neighbours). Divided by the
//! mean cell area ā it reads (μ_c + Σ_m K_cm) x_c − Σ_m K_cm x_m = μ_c b_c, with μ_c = a_c/ā and
//! K_cm = Δt D_cm w_cm / ā. On a raster of equal cells with the five-point stencil (w = 1 to the
//! sides, 0 to the corners) that is the Lambert system (I + Δt L) x = b exactly.
//!
//! **The diffusivity of a pair.** A side pair takes the arithmetic mean of its two cells, as the
//! Lambert faces do. A diagonal pair, whose weight comes from the quad of four cells around the
//! grid vertex it crosses, takes the least of that quad's four side means: the quad's energy is
//! then the quad's own finite-element energy at that diffusivity plus non-negative side terms, so
//! the matrix stays symmetric positive definite where diffusivities jump (the land-sea edge), as
//! long as the side weights are positive (`sphere/cube/tests.rs` checks them).
//!
//! **The solve.** The first guess is the locally one-dimensional splitting, face by face: an
//! implicit sweep along each face's rows, then along its columns (each a tridiagonal system; the
//! couplings across a face's edge are taken at the old values, so the splitting needs no wrap).
//! Then conjugate gradients, preconditioned by the diagonal ([`step_in`], the crust's spreading)
//! or by one multigrid V-cycle (`multigrid.rs`), stop at an RMS residual of `tol_m` over all
//! cells, or after [`CG_MAX_ITER`] iterations.
//!
//! **Volume.** Σ_c μ_c x_c = Σ_c μ_c b_c − Σ_c r_c for the final residual r (the columns of the
//! matrix sum to μ), so the residual's sum is put back in proportion to each cell's coupling
//! (Δx_c ∝ Σ_m K_cm / μ_c): **Σ a·h is kept to round-off**, the property the tests check.
//!
//! **Cells with nothing to diffuse.** A cell whose couplings are all zero is a block of the
//! identity that nothing couples to; the solve leaves it as it was, bit for bit, and visits only
//! the cells with a coupling (the crust's spreading diffuses a few per cent of them). The first
//! guess runs only when every cell takes part (the surface stage's solves).

use rayon::prelude::*;

use super::multigrid::{self, Hierarchy};
use super::{CHUNK, ordered_sum};
use crate::sphere::cube::{CubeGrid, NO_CELL};
use crate::surface::diffusion::{CG_MAX_ITER, CG_TOL_M, thomas};
use crate::surface::params::DiffusionSolver;

/// The matrix (μ + ΣK) − K on the cells that take part, ascending.
#[derive(Default)]
pub(super) struct Op {
    /// The cells, ascending.
    pub cells: Vec<u32>,
    /// Each slot's neighbour as an index into `cells` (NO_CELL: none, or not taking part).
    pub cols: Vec<[u32; 8]>,
    /// The couplings K of each slot (0 where there is no neighbour).
    pub k: Vec<[f64; 8]>,
    /// μ = a/ā.
    pub mu: Vec<f64>,
    /// Σ K of each cell.
    pub s: Vec<f64>,
    /// μ + Σ K.
    pub diag: Vec<f64>,
    /// Every cell takes part (`cells[i] == i`).
    pub all: bool,
}

impl Op {
    pub(super) fn len(&self) -> usize {
        self.cells.len()
    }

    /// out = A·p.
    pub(super) fn apply(&self, p: &[f64], out: &mut [f64]) {
        out.par_chunks_mut(CHUNK)
            .enumerate()
            .for_each(|(q, o)| {
                let i0 = q * CHUNK;
                for (d, y) in o.iter_mut().enumerate() {
                    *y = self.row(i0 + d, p);
                }
            });
    }

    #[inline(always)]
    fn row(&self, i: usize, p: &[f64]) -> f64 {
        let (cols, k) = (&self.cols[i], &self.k[i]);
        let mut acc = self.diag[i] * p[i];
        for s in 0..8 {
            let j = cols[s];
            if j != NO_CELL {
                acc -= k[s] * p[j as usize];
            }
        }
        acc
    }
}

/// The couplings of cell `c` (`scale` = Δt/ā).
#[inline]
fn couplings(g: &CubeGrid, kd: &[f64], scale: f64, c: usize) -> [f64; 8] {
    let s = g.topology().slots(c);
    let lap = g.lap(c);
    let mean = |x: usize, y: usize| 0.5 * (kd[x] + kd[y]);
    let mut k = [0.0; 8];
    for slot in 0..8 {
        let m = s[slot];
        if m == NO_CELL {
            continue;
        }
        let m = m as usize;
        let d = if slot % 2 == 0 {
            mean(c, m)
        } else {
            // A diagonal: its two side cells exist (only diagonals go missing, at the corners).
            let a = s[slot - 1] as usize;
            let b = s[(slot + 1) % 8] as usize;
            mean(c, a).min(mean(a, m)).min(mean(m, b)).min(mean(b, c))
        };
        k[slot] = scale * d * lap[slot];
    }
    k
}

/// The kept vectors of the solves (grid-sized; reused between steps).
#[derive(Default)]
pub struct Workspace {
    pub(super) op: Op,
    full: Vec<[f64; 8]>,
    pos: Vec<u32>,
    b: Vec<f64>,
    rhs: Vec<f64>,
    x: Vec<f64>,
    r: Vec<f64>,
    po: Vec<f64>,
    pn: Vec<f64>,
    ap: Vec<f64>,
    z: Vec<f64>,
    tmp: Vec<f64>,
    pub(super) hier: Option<Hierarchy>,
}

fn fit(v: &mut Vec<f64>, n: usize) {
    if v.len() != n {
        *v = crate::alloc::filled(n, 0.0);
    }
}

impl Workspace {
    /// Builds the operator for `kd` and `dt`: every cell (`all`), or the cells with a coupling.
    fn build(&mut self, g: &CubeGrid, kd: &[f64], dt: f64, all: bool) {
        let n = g.topology().cells();
        let scale = dt / g.mean_area();
        let mean = g.mean_area();
        let areas = g.areas();
        let op = &mut self.op;
        op.all = all;
        if all {
            op.cells.clear();
            op.cells.extend(0..n as u32);
            op.k.resize(n, [0.0; 8]);
            op.k.par_iter_mut()
                .enumerate()
                .for_each(|(c, k)| *k = couplings(g, kd, scale, c));
            op.cols.resize(n, [NO_CELL; 8]);
            op.cols
                .par_iter_mut()
                .enumerate()
                .for_each(|(c, s)| *s = *g.topology().slots(c));
        } else {
            self.full.resize(n, [0.0; 8]);
            self.full
                .par_iter_mut()
                .enumerate()
                .for_each(|(c, k)| *k = couplings(g, kd, scale, c));
            let full = &self.full;
            op.cells.clear();
            op.cells.par_extend(
                (0..n as u32)
                    .into_par_iter()
                    .filter(|&c| full[c as usize].iter().any(|&k| k != 0.0)),
            );
            self.pos.clear();
            self.pos.resize(n, NO_CELL);
            for (i, &c) in op.cells.iter().enumerate() {
                self.pos[c as usize] = i as u32;
            }
            let (cells, pos) = (&op.cells, &self.pos);
            op.k.clear();
            op.k.par_extend(cells.par_iter().map(|&c| full[c as usize]));
            op.cols.clear();
            op.cols.par_extend(cells.par_iter().map(|&c| {
                let s = g.topology().slots(c as usize);
                std::array::from_fn(|k| {
                    if s[k] == NO_CELL {
                        NO_CELL
                    } else {
                        pos[s[k] as usize]
                    }
                })
            }));
        }
        let l = op.cells.len();
        op.mu.clear();
        op.mu
            .par_extend(op.cells.par_iter().map(|&c| areas[c as usize] / mean));
        op.s.clear();
        op.s.par_extend(op.k.par_iter().map(|k| k.iter().sum::<f64>()));
        op.diag.clear();
        op.diag
            .par_extend(op.mu.par_iter().zip(&op.s).map(|(m, s)| m + s));
        for v in [
            &mut self.rhs,
            &mut self.x,
            &mut self.r,
            &mut self.po,
            &mut self.pn,
            &mut self.ap,
        ] {
            fit(v, l);
        }
    }
}

/// One implicit step by the preset's solver: conjugate gradients preconditioned by a multigrid
/// V-cycle to an RMS residual of `tol_m` metres on every cell, or [`step_in`]'s Jacobi solve
/// (whose tolerance stays the Lambert solve's `CG_TOL_M`). Returns the iterations.
pub fn solve(
    ws: &mut Workspace,
    g: &CubeGrid,
    h: &mut [f64],
    kd: &[f64],
    dt: f64,
    solver: DiffusionSolver,
    tol_m: f64,
) -> usize {
    match solver {
        DiffusionSolver::Jacobi => step_in(ws, g, h, kd, dt),
        DiffusionSolver::Multigrid => {
            if dt.is_nan() || dt <= 0.0 {
                return 0;
            }
            ws.build(g, kd, dt, true);
            run(ws, g, h, tol_m, true)
        }
    }
}

/// One implicit step with the Jacobi preconditioner on the cells with a coupling.
pub fn step_in(ws: &mut Workspace, g: &CubeGrid, h: &mut [f64], kd: &[f64], dt: f64) -> usize {
    if dt.is_nan() || dt <= 0.0 {
        return 0;
    }
    ws.build(g, kd, dt, false);
    if ws.op.len() == 0 {
        return 0;
    }
    if ws.op.len() == g.topology().cells() {
        // Every cell takes part: the same operator with the first guess.
        ws.op.all = true;
    }
    run(ws, g, h, CG_TOL_M, false)
}

/// [`step_in`] with a fresh workspace.
pub fn step(g: &CubeGrid, h: &mut [f64], kd: &[f64], dt: f64) -> usize {
    step_in(&mut Workspace::default(), g, h, kd, dt)
}

/// The first guess, the conjugate gradients and the volume's return, on a built operator.
fn run(ws: &mut Workspace, g: &CubeGrid, h: &mut [f64], tol_m: f64, mg: bool) -> usize {
    let l = ws.op.len();
    {
        let (op, rhs, x) = (&ws.op, &mut ws.rhs, &mut ws.x);
        rhs.par_iter_mut()
            .zip(x.par_iter_mut())
            .enumerate()
            .for_each(|(i, (b, xv))| {
                let v = h[op.cells[i] as usize];
                *b = op.mu[i] * v;
                *xv = v;
            });
    }
    if ws.op.all {
        fit(&mut ws.b, l);
        fit(&mut ws.tmp, l);
        ws.b.copy_from_slice(&ws.x);
        lod(g, &ws.op, &ws.b, &mut ws.x, &mut ws.tmp);
    }
    if mg {
        fit(&mut ws.z, l);
        multigrid::prepare(&mut ws.hier, g, &ws.op);
    }
    let iterations = cg(ws, g.topology().cells(), tol_m, mg);
    // Σ μx = Σ μb − Σ r: the residual's sum back to the cells, in proportion to their coupling.
    let Workspace {
        op, rhs, x, ap, ..
    } = ws;
    op.apply(x, ap);
    let lost = ordered_sum(l, |r| r.map(|i| rhs[i] - ap[i]).sum());
    let weight = ordered_sum(l, |r| op.s[r].iter().sum());
    if weight > 0.0 && lost != 0.0 {
        let kk = lost / weight;
        x.par_iter_mut()
            .enumerate()
            .for_each(|(i, xv)| *xv += kk * op.s[i] / op.mu[i]);
    }
    for (i, &c) in op.cells.iter().enumerate() {
        h[c as usize] = x[i];
    }
    iterations
}

/// The locally one-dimensional first guess on every cell: rows of each face, then columns, each
/// a tridiagonal system (μ + K) y = μ b divided through by μ; couplings across a face's edge
/// take the old value of the cell beyond it.
fn lod(g: &CubeGrid, op: &Op, b: &[f64], x: &mut [f64], tmp: &mut [f64]) {
    let n = g.n();
    let nn = n * n;
    let t = g.topology();
    // Along the rows (slots E = 0 and W = 4).
    x.par_chunks_mut(n).enumerate().for_each(|(row, xr)| {
        let c0 = row * n;
        let (mut lo, mut di, mut up) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
        for i in 0..n {
            let c = c0 + i;
            let (ke, kw, mu) = (op.k[c][0], op.k[c][4], op.mu[c]);
            di[i] = 1.0 + (ke + kw) / mu;
            let mut v = b[c];
            if i > 0 {
                lo[i] = -kw / mu;
            } else {
                v += kw * b[t.slots(c)[4] as usize] / mu;
            }
            if i + 1 < n {
                up[i] = -ke / mu;
            } else {
                v += ke * b[t.slots(c)[0] as usize] / mu;
            }
            xr[i] = v;
        }
        thomas(&lo, &di, &up, xr);
    });
    // Along the columns (slots N = 2 and S = 6), into `tmp` column by column.
    let y: &[f64] = x;
    tmp.par_chunks_mut(n).enumerate().for_each(|(col, tc)| {
        let (f, i) = (col / n, col % n);
        let (mut lo, mut di, mut up) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
        for j in 0..n {
            let c = f * nn + j * n + i;
            let (kn, ks, mu) = (op.k[c][2], op.k[c][6], op.mu[c]);
            di[j] = 1.0 + (kn + ks) / mu;
            let mut v = y[c];
            if j > 0 {
                lo[j] = -kn / mu;
            } else {
                v += kn * y[t.slots(c)[2] as usize] / mu;
            }
            if j + 1 < n {
                up[j] = -ks / mu;
            } else {
                v += ks * y[t.slots(c)[6] as usize] / mu;
            }
            tc[j] = v;
        }
        thomas(&lo, &di, &up, tc);
    });
    let tmp: &[f64] = tmp;
    x.par_chunks_mut(n).enumerate().for_each(|(row, xr)| {
        let (f, j) = (row / n, row % n);
        for (i, v) in xr.iter_mut().enumerate() {
            *v = tmp[f * nn + i * n + j];
        }
    });
}

/// Conjugate gradients on the built operator from `ws.x`, preconditioned by the diagonal or by
/// the V-cycle; stops at an RMS residual of `tol_m` over `cells` cells. Two passes an iteration:
/// the direction with A·p and p·A·p, then x, r and r·r (and r·z for the diagonal).
fn cg(ws: &mut Workspace, cells: usize, tol_m: f64, mg: bool) -> usize {
    let l = ws.op.len();
    let tol2 = tol_m * tol_m * cells as f64;
    {
        let Workspace { op, rhs, x, r, .. } = &mut *ws;
        op.apply(x, r);
        r.par_iter_mut()
            .zip(rhs.par_iter())
            .for_each(|(rv, b)| *rv = b - *rv);
    }
    let rr0 = ordered_sum(l, |q| ws.r[q].iter().map(|v| v * v).sum());
    if rr0 <= tol2 {
        return 0;
    }
    let mut rz = if mg {
        multigrid::precondition(ws.hier.as_mut().expect("prepared"), &ws.r, &mut ws.z);
        let (r, z) = (&ws.r, &ws.z);
        ordered_sum(l, |q| q.map(|i| r[i] * z[i]).sum())
    } else {
        let (r, d) = (&ws.r, &ws.op.diag);
        ordered_sum(l, |q| q.map(|i| r[i] * r[i] / d[i]).sum())
    };
    let mut beta: Option<f64> = None;
    let mut iterations = 0;
    for _ in 0..CG_MAX_ITER {
        // The direction pn = z + β po, A·pn and pn·A·pn.
        let pap = {
            let Workspace {
                op, r, po, pn, ap, z, ..
            } = &mut *ws;
            let (r, po, z): (&[f64], &[f64], &[f64]) = (r, po, z);
            let dir = |j: usize| -> f64 {
                let zj = if mg { z[j] } else { r[j] / op.diag[j] };
                match beta {
                    Some(b) => zj + b * po[j],
                    None => zj,
                }
            };
            let parts: Vec<f64> = pn
                .par_chunks_mut(CHUNK)
                .zip(ap.par_chunks_mut(CHUNK))
                .enumerate()
                .map(|(q, (pc, ac))| {
                    let i0 = q * CHUNK;
                    let mut s = 0.0;
                    for d in 0..pc.len() {
                        let i = i0 + d;
                        let pi = dir(i);
                        let (cols, k) = (&op.cols[i], &op.k[i]);
                        let mut acc = op.diag[i] * pi;
                        for t in 0..8 {
                            let j = cols[t];
                            if j != NO_CELL {
                                acc -= k[t] * dir(j as usize);
                            }
                        }
                        pc[d] = pi;
                        ac[d] = acc;
                        s += pi * acc;
                    }
                    s
                })
                .collect();
            parts.iter().sum::<f64>()
        };
        std::mem::swap(&mut ws.po, &mut ws.pn);
        if pap.is_nan() || pap <= 0.0 {
            break;
        }
        let alpha = rz / pap;
        // x += α p, r −= α A·p, and the sums.
        let (rr, rz_jac) = {
            let Workspace {
                op, x, r, po, ap, ..
            } = &mut *ws;
            let (p, ap): (&[f64], &[f64]) = (po, ap);
            let parts: Vec<[f64; 2]> = x
                .par_chunks_mut(CHUNK)
                .zip(r.par_chunks_mut(CHUNK))
                .enumerate()
                .map(|(q, (xc, rc))| {
                    let i0 = q * CHUNK;
                    let (mut s0, mut s1) = (0.0, 0.0);
                    for d in 0..xc.len() {
                        let i = i0 + d;
                        xc[d] += alpha * p[i];
                        let rv = rc[d] - alpha * ap[i];
                        rc[d] = rv;
                        s0 += rv * rv;
                        s1 += rv * rv / op.diag[i];
                    }
                    [s0, s1]
                })
                .collect();
            (
                parts.iter().map(|s| s[0]).sum::<f64>(),
                parts.iter().map(|s| s[1]).sum::<f64>(),
            )
        };
        iterations += 1;
        if rr <= tol2 {
            break;
        }
        let rz_new = if mg {
            multigrid::precondition(ws.hier.as_mut().expect("prepared"), &ws.r, &mut ws.z);
            let (r, z) = (&ws.r, &ws.z);
            ordered_sum(l, |q| q.map(|i| r[i] * z[i]).sum())
        } else {
            rz_jac
        };
        beta = Some(rz_new / rz);
        rz = rz_new;
    }
    iterations
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! Implicit linear diffusion ∂h/∂t = ∇·(D ∇h) with a diffusivity per cell: hillslope creep on land
//! (the preset's κ times the surface rock's diffusivity multiplier) and depth-dependent marine
//! transport in the sea (`marine.rs`), in one solve.
//!
//! **Scheme:** fully implicit (backward Euler) in 2D: (I + Δt·L) h¹ = h⁰ with L the five-point
//! flux-form Laplacian, periodic in x and with no flux through the first and last row, solved by
//! conjugate gradients with a Jacobi preconditioner. The initial guess is the locally
//! one-dimensional splitting of the ADI family (an implicit sweep along x — a cyclic tridiagonal
//! system per row — then along z), which alone would leave streaks along the last sweep's
//! direction where D changes sharply at the time steps deep time uses (D·Δt/Δx² reaches ~30 on the
//! shelf over 2 Myr; the first 100-Myr run showed them); Peaceman–Rachford ADI would oscillate
//! there. Faces take the arithmetic mean of their two cells' diffusivities (as Landlab's
//! `LinearDiffuser` maps node diffusivities to links).
//!
//! The matrix's columns sum to one, so Σh¹ = Σh⁰ − Σr for the final residual r; the residual's
//! sum is put back in proportion to each cell's coupling, so **Σh is conserved** to round-off —
//! the mass-conservation property the tests check. Rows are processed in parallel and every
//! reduction is a row-ordered sum, so results do not depend on the thread count.
//!
//! **Fused passes (P1-17).** The conjugate-gradient loop is memory- and barrier-bound (about 111
//! iterations a step over ~20 vectors of 4 MB), so an iteration is two parallel passes over strips
//! of rows instead of seven regions: one forms the direction and A·p and reduces p·A·p, the other
//! updates x and r, forms z and reduces r·r and r·z. The arithmetic is the textbook loop's, cell
//! by cell and operation by operation (no fused multiply-add, no reciprocal in place of a
//! division, terms of A·p added in the same order); only the schedule differs. Reductions keep
//! the old scheme exactly: a sum per row in ascending column order, then the row sums added in
//! ascending row order on one thread. Nothing depends on the strip height or the thread pool, so
//! results are bit-identical to the textbook loop at any thread count; `diffusion/parity.rs` keeps
//! that loop verbatim and compares bit patterns.

use rayon::prelude::*;

use super::grid::Dims;

/// One implicit step of Δt years on `h` with diffusivities `kd` (m²/yr); returns the conjugate
/// gradient iterations it took.
pub fn step(d: &Dims, h: &mut [f64], kd: &[f64], dt: f64) -> usize {
    let b = h.to_vec();
    lod(d, h, kd, dt);
    cg(d, &b, h, kd, dt)
}

/// The locally one-dimensional splitting (the initial guess).
fn lod(d: &Dims, h: &mut [f64], kd: &[f64], dt: f64) {
    let (w, hh) = (d.w, d.h);
    let s2 = d.dx * d.dx;
    // x sweep, row by row.
    h.par_chunks_mut(w)
        .zip(kd.par_chunks(w))
        .for_each(|(row, krow)| {
            let face = |i: usize| -> f64 {
                // Face between i and i + 1 (wrapping when the grid does).
                if i + 1 < w {
                    0.5 * (krow[i] + krow[i + 1]) * dt / s2
                } else if d.wrap_x && w > 1 {
                    0.5 * (krow[w - 1] + krow[0]) * dt / s2
                } else {
                    0.0
                }
            };
            let mut lo = vec![0.0; w];
            let mut di = vec![0.0; w];
            let mut up = vec![0.0; w];
            for i in 0..w {
                let west = if i == 0 { face(w - 1) } else { face(i - 1) };
                let east = face(i);
                lo[i] = -west;
                up[i] = -east;
                di[i] = 1.0 + west + east;
            }
            if d.wrap_x && w > 2 {
                let corner_lo = lo[0];
                let corner_up = up[w - 1];
                cyclic(&lo, &di, &up, corner_up, corner_lo, row);
            } else {
                if w == 2 && d.wrap_x {
                    // Two cells share two faces; fold them into one.
                    let f = face(0) + face(1);
                    lo[1] = -f;
                    up[0] = -f;
                    di[0] = 1.0 + f;
                    di[1] = 1.0 + f;
                }
                lo[0] = 0.0;
                up[w - 1] = 0.0;
                if w == 1 {
                    di[0] = 1.0;
                }
                thomas(&lo, &di, &up, row);
            }
        });
    // z sweep, column by column (gathered, solved, scattered). A task takes a block of
    // `COL_BLOCK` neighbouring columns, so the cache line it reads for one is the line it needs
    // for the next seven, and hands the block back row-major; the rows then copy their pieces
    // back in parallel. Each column is solved as ever.
    let blocks: Vec<Vec<f64>> = (0..w.div_ceil(COL_BLOCK))
        .into_par_iter()
        .map(|blk| {
            let i0 = blk * COL_BLOCK;
            let nb = COL_BLOCK.min(w - i0);
            let mut out = vec![0.0; hh * nb];
            let mut col = vec![0.0; hh];
            let mut lo = vec![0.0; hh];
            let mut di = vec![0.0; hh];
            let mut up = vec![0.0; hh];
            for b in 0..nb {
                let i = i0 + b;
                for (j, c) in col.iter_mut().enumerate() {
                    *c = h[j * w + i];
                }
                let face = |j: usize| -> f64 {
                    if j + 1 < hh {
                        0.5 * (kd[j * w + i] + kd[(j + 1) * w + i]) * dt / s2
                    } else {
                        0.0
                    }
                };
                for j in 0..hh {
                    let north = if j == 0 { 0.0 } else { face(j - 1) };
                    let south = face(j);
                    lo[j] = -north;
                    up[j] = -south;
                    di[j] = 1.0 + north + south;
                }
                thomas(&lo, &di, &up, &mut col);
                for (j, &v) in col.iter().enumerate() {
                    out[j * nb + b] = v;
                }
            }
            out
        })
        .collect();
    h.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
        for (blk, out) in blocks.iter().enumerate() {
            let i0 = blk * COL_BLOCK;
            let nb = COL_BLOCK.min(w - i0);
            row[i0..i0 + nb].copy_from_slice(&out[j * nb..(j + 1) * nb]);
        }
    });
}

/// Columns per task in the z sweep of `lod` (a cache line of `f64`).
const COL_BLOCK: usize = 8;

/// RMS residual (m) at which the conjugate gradients stop …
const CG_TOL_M: f64 = 1.0e-6;
/// … or after this many iterations.
const CG_MAX_ITER: usize = 400;

/// Rows per task in the fused passes. It sets the grain of the work and nothing else: every sum is
/// taken per row, so no result depends on it (the parity test runs grids whose height it does not
/// divide, and grids shorter than a strip).
const STRIP_ROWS: usize = 8;

/// The value `Iterator::sum` starts an `f64` sum from (−0.0 on today's toolchain). The fused loops
/// start their row sums from it too, so that a sum of signed zeros has the sign the serial
/// `.sum()` gave it.
#[inline(always)]
fn sum_zero() -> f64 {
    std::iter::empty::<f64>().sum()
}

/// The five-point operator of (I + Δt·L): east and south face couplings per cell.
struct Operator {
    w: usize,
    h: usize,
    wrap: bool,
    /// Coupling across the east face of each cell (0 where there is no east neighbour).
    east: Vec<f64>,
    /// Coupling across the south face (0 on the last row).
    south: Vec<f64>,
    diag: Vec<f64>,
}

impl Operator {
    /// Cell by cell the same expressions as the serial loops it replaced (faces from the arithmetic
    /// mean of the two cells' diffusivities, the diagonal as ((((1 + east) + west) + south) +
    /// north)); rows are independent, so they run in parallel.
    fn new(d: &Dims, kd: &[f64], dt: f64) -> Operator {
        let (w, h) = (d.w, d.h);
        let s2 = d.dx * d.dx;
        let n = w * h;
        // On a two-cell ring both faces join the same pair; one cell wide, the only x face is the
        // cell itself and there is none.
        let wrap = d.wrap_x && w > 1;
        let mut east = vec![0.0; n];
        let mut south = vec![0.0; n];
        east.par_chunks_mut(w)
            .zip(south.par_chunks_mut(w))
            .enumerate()
            .for_each(|(j, (erow, srow))| {
                let krow = &kd[j * w..(j + 1) * w];
                for i in 0..w {
                    if i + 1 < w {
                        erow[i] = 0.5 * (krow[i] + krow[i + 1]) * dt / s2;
                    } else if wrap {
                        erow[i] = 0.5 * (krow[i] + krow[0]) * dt / s2;
                    }
                }
                if j + 1 < h {
                    let below = &kd[(j + 1) * w..(j + 2) * w];
                    for i in 0..w {
                        srow[i] = 0.5 * (krow[i] + below[i]) * dt / s2;
                    }
                }
            });
        let mut diag = vec![1.0; n];
        diag.par_chunks_mut(w).enumerate().for_each(|(j, drow)| {
            let c0 = j * w;
            let erow = &east[c0..c0 + w];
            let srow = &south[c0..c0 + w];
            for i in 0..w {
                let west = if i > 0 {
                    erow[i - 1]
                } else if wrap {
                    erow[w - 1]
                } else {
                    0.0
                };
                let north = if j > 0 { south[c0 - w + i] } else { 0.0 };
                drow[i] = 1.0 + erow[i] + west + srow[i] + north;
            }
        });
        Operator {
            w,
            h,
            wrap,
            east,
            south,
            diag,
        }
    }

    /// Row `j` of y = A·x into `out`, given rows j − 1, j and j + 1 of x (`north` is `None` on the
    /// first row, `south` on the last). Each cell gets its terms in one fixed order — diagonal,
    /// east, west, south, north, each subtracted from the running value — and the row is built one
    /// term at a time, which is the same arithmetic cell by cell but vectorises.
    fn apply_row(
        &self,
        j: usize,
        north: Option<&[f64]>,
        cur: &[f64],
        south: Option<&[f64]>,
        out: &mut [f64],
    ) {
        let w = self.w;
        let c0 = j * w;
        let diag = &self.diag[c0..c0 + w];
        let east = &self.east[c0..c0 + w];
        for ((o, &g), &x) in out.iter_mut().zip(diag).zip(cur) {
            *o = g * x;
        }
        // East neighbour i + 1, then the wrapped one for the last cell.
        for ((o, &e), &x) in out.iter_mut().zip(east).zip(&cur[1..]) {
            *o -= e * x;
        }
        if self.wrap {
            out[w - 1] -= east[w - 1] * cur[0];
        }
        // West neighbour i − 1, then the wrapped one for the first cell.
        for ((o, &e), &x) in out[1..].iter_mut().zip(east).zip(cur) {
            *o -= e * x;
        }
        if self.wrap {
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

    /// y = A·x, rows in parallel.
    fn apply(&self, x: &[f64], y: &mut [f64]) {
        let (w, h) = (self.w, self.h);
        y.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
            let north = (j > 0).then(|| &x[(j - 1) * w..j * w]);
            let south = (j + 1 < h).then(|| &x[(j + 1) * w..(j + 2) * w]);
            self.apply_row(j, north, &x[j * w..(j + 1) * w], south, row);
        });
    }
}

/// `out[k]` = Σ a·b over row k of the strip `a`, `b` (rows of `w` cells): each row's sum is taken
/// in ascending column order from [`sum_zero`], exactly as a serial sum over the row would.
/// Four rows run side by side, so the CPU overlaps their additions instead of waiting on each
/// one's previous result; that changes the schedule and not the order within any row.
fn row_dots(w: usize, a: &[f64], b: &[f64], out: &mut [f64]) {
    let rows = out.len();
    debug_assert!(a.len() == rows * w && b.len() == rows * w);
    let mut k = 0;
    while k + 4 <= rows {
        let base = k * w;
        let (a0, a1) = (&a[base..base + w], &a[base + w..base + 2 * w]);
        let (a2, a3) = (
            &a[base + 2 * w..base + 3 * w],
            &a[base + 3 * w..base + 4 * w],
        );
        let (b0, b1) = (&b[base..base + w], &b[base + w..base + 2 * w]);
        let (b2, b3) = (
            &b[base + 2 * w..base + 3 * w],
            &b[base + 3 * w..base + 4 * w],
        );
        let (mut s0, mut s1, mut s2, mut s3) = (sum_zero(), sum_zero(), sum_zero(), sum_zero());
        for i in 0..w {
            s0 += a0[i] * b0[i];
            s1 += a1[i] * b1[i];
            s2 += a2[i] * b2[i];
            s3 += a3[i] * b3[i];
        }
        out[k] = s0;
        out[k + 1] = s1;
        out[k + 2] = s2;
        out[k + 3] = s3;
        k += 4;
    }
    while k < rows {
        let base = k * w;
        let mut s = sum_zero();
        for (p, q) in a[base..base + w].iter().zip(&b[base..base + w]) {
            s += p * q;
        }
        out[k] = s;
        k += 1;
    }
}

/// Setup pass: r = b − A·x, the preconditioned residual z = r/diag, and per row the sums
/// `sums[j] = [Σ r·r, Σ r·z]`.
fn pass_init(
    op: &Operator,
    b: &[f64],
    x: &[f64],
    r: &mut [f64],
    z: &mut [f64],
    sums: &mut [[f64; 2]],
) {
    let (w, h) = (op.w, op.h);
    r.par_chunks_mut(STRIP_ROWS * w)
        .zip(z.par_chunks_mut(STRIP_ROWS * w))
        .zip(sums.par_chunks_mut(STRIP_ROWS))
        .enumerate()
        .for_each(|(s, ((rs, zs), ss))| {
            let rows = ss.len();
            let j0 = s * STRIP_ROWS;
            for jj in 0..rows {
                let j = j0 + jj;
                let north = (j > 0).then(|| &x[(j - 1) * w..j * w]);
                let south = (j + 1 < h).then(|| &x[(j + 1) * w..(j + 2) * w]);
                let rrow = &mut rs[jj * w..(jj + 1) * w];
                op.apply_row(j, north, &x[j * w..(j + 1) * w], south, rrow);
                for (rv, &bv) in rrow.iter_mut().zip(&b[j * w..(j + 1) * w]) {
                    *rv = bv - *rv;
                }
                let zrow = &mut zs[jj * w..(jj + 1) * w];
                let diag = &op.diag[j * w..(j + 1) * w];
                for ((zv, &rv), &dv) in zrow.iter_mut().zip(&*rrow).zip(diag) {
                    *zv = rv / dv;
                }
            }
            let (mut rr, mut rz) = ([0.0; STRIP_ROWS], [0.0; STRIP_ROWS]);
            row_dots(w, rs, rs, &mut rr[..rows]);
            row_dots(w, rs, zs, &mut rz[..rows]);
            for (jj, sum) in ss.iter_mut().enumerate() {
                *sum = [rr[jj], rz[jj]];
            }
        });
}

/// First pass of an iteration: the search direction p and A·p with p·A·p per row. The direction is
/// z on the first iteration and z + β·p_old afterwards; rows j − 1 and j + 1 of it are needed to
/// apply the operator to row j, so each strip forms its own rows into `pnew` (a separate buffer,
/// so no strip reads what another writes) and its neighbours' edge rows into a ring of three rows
/// that stays in cache. It is the same value cell by cell wherever it is formed.
/// `pap_rows[j]` = Σ p·(A·p) of row j.
fn pass_direction(
    op: &Operator,
    z: &[f64],
    pold: &[f64],
    beta: Option<f64>,
    pnew: &mut [f64],
    ap: &mut [f64],
    pap_rows: &mut [f64],
) {
    let (w, h) = (op.w, op.h);
    pnew.par_chunks_mut(STRIP_ROWS * w)
        .zip(ap.par_chunks_mut(STRIP_ROWS * w))
        .zip(pap_rows.par_chunks_mut(STRIP_ROWS))
        .enumerate()
        .for_each_init(
            || [vec![0.0; w], vec![0.0; w], vec![0.0; w]],
            |ring, (s, ((pn, aps), pr))| {
                let rows = pr.len();
                let j0 = s * STRIP_ROWS;
                let fill = |j: usize, dst: &mut [f64]| {
                    let zrow = &z[j * w..(j + 1) * w];
                    match beta {
                        Some(beta) => {
                            let prow = &pold[j * w..(j + 1) * w];
                            for ((d, &zv), &pv) in dst.iter_mut().zip(zrow).zip(prow) {
                                *d = zv + beta * pv;
                            }
                        }
                        None => dst.copy_from_slice(zrow),
                    }
                };
                // ring = [row j − 1, row j, row j + 1] of the direction.
                if j0 > 0 {
                    fill(j0 - 1, &mut ring[0]);
                }
                fill(j0, &mut ring[1]);
                pn[..w].copy_from_slice(&ring[1]);
                for jj in 0..rows {
                    let j = j0 + jj;
                    if j + 1 < h {
                        fill(j + 1, &mut ring[2]);
                        if jj + 1 < rows {
                            pn[(jj + 1) * w..(jj + 2) * w].copy_from_slice(&ring[2]);
                        }
                    }
                    let north = (j > 0).then(|| &ring[0][..]);
                    let south = (j + 1 < h).then(|| &ring[2][..]);
                    op.apply_row(j, north, &ring[1], south, &mut aps[jj * w..(jj + 1) * w]);
                    ring.rotate_left(1);
                }
                row_dots(w, pn, aps, pr);
            },
        );
}

/// Second pass of an iteration: x += α·p, r −= α·A·p, z = r/diag, and per row the sums
/// `sums[j] = [Σ r·r, Σ r·z]` of the new r.
#[allow(clippy::too_many_arguments)]
fn pass_update(
    op: &Operator,
    alpha: f64,
    p: &[f64],
    ap: &[f64],
    x: &mut [f64],
    r: &mut [f64],
    z: &mut [f64],
    sums: &mut [[f64; 2]],
) {
    let w = op.w;
    x.par_chunks_mut(STRIP_ROWS * w)
        .zip(r.par_chunks_mut(STRIP_ROWS * w))
        .zip(z.par_chunks_mut(STRIP_ROWS * w))
        .zip(sums.par_chunks_mut(STRIP_ROWS))
        .enumerate()
        .for_each(|(s, (((xs, rs), zs), ss))| {
            let rows = ss.len();
            let c0 = s * STRIP_ROWS * w;
            let len = rows * w;
            let (ps, aps, ds) = (&p[c0..c0 + len], &ap[c0..c0 + len], &op.diag[c0..c0 + len]);
            for (xv, &pv) in xs.iter_mut().zip(ps) {
                *xv += alpha * pv;
            }
            for (rv, &av) in rs.iter_mut().zip(aps) {
                *rv -= alpha * av;
            }
            for ((zv, &rv), &dv) in zs.iter_mut().zip(&*rs).zip(ds) {
                *zv = rv / dv;
            }
            let (mut rr, mut rz) = ([0.0; STRIP_ROWS], [0.0; STRIP_ROWS]);
            row_dots(w, rs, rs, &mut rr[..rows]);
            row_dots(w, rs, zs, &mut rz[..rows]);
            for (jj, sum) in ss.iter_mut().enumerate() {
                *sum = [rr[jj], rz[jj]];
            }
        });
}

/// Refines `x` to the solution of A·x = b by Jacobi-preconditioned conjugate gradients, then puts
/// the residual's sum back so Σx = Σb; returns the iterations.
///
/// Every iteration is two passes over the grid, each a parallel region of strips of rows: one forms
/// the direction and A·p and reduces p·A·p, the other updates x and r and reduces r·r and r·z
/// (the textbook loop takes seven regions and about twice the memory traffic). The arithmetic is
/// the textbook loop's, term for term. **Reductions:** a sum is a sum per row in ascending
/// column order, then those row sums added in ascending row order, on one thread — never a
/// tree over threads, never a function of the strip size or the pool — so the result is the same
/// on any number of threads (`parity.rs` compares bit patterns against the textbook loop).
fn cg(d: &Dims, b: &[f64], x: &mut [f64], kd: &[f64], dt: f64) -> usize {
    let n = d.cells();
    let h = d.h;
    let op = Operator::new(d, kd, dt);
    let mut ap = vec![0.0; n];
    let mut r = vec![0.0; n];
    let mut z = vec![0.0; n];
    let mut sums = vec![[0.0; 2]; h];
    pass_init(&op, b, x, &mut r, &mut z, &mut sums);
    let tol2 = CG_TOL_M * CG_TOL_M * n as f64;
    let mut iterations = 0;
    let rr0: f64 = sums.iter().map(|s| s[0]).sum();
    if rr0 > tol2 {
        let mut rz: f64 = sums.iter().map(|s| s[1]).sum();
        // The direction, two buffers: an iteration reads the last and writes the next.
        let mut pold = vec![0.0; n];
        let mut pnew = vec![0.0; n];
        let mut pap_rows = vec![0.0; h];
        // The first direction is z itself; afterwards z + β·p.
        let mut beta = None;
        while iterations < CG_MAX_ITER {
            iterations += 1;
            pass_direction(&op, &z, &pold, beta, &mut pnew, &mut ap, &mut pap_rows);
            std::mem::swap(&mut pold, &mut pnew);
            let pap: f64 = pap_rows.iter().sum();
            if pap <= 0.0 {
                break;
            }
            let alpha = rz / pap;
            pass_update(&op, alpha, &pold, &ap, x, &mut r, &mut z, &mut sums);
            let rr: f64 = sums.iter().map(|s| s[0]).sum();
            if rr <= tol2 {
                break;
            }
            let rz_new: f64 = sums.iter().map(|s| s[1]).sum();
            beta = Some(rz_new / rz);
            rz = rz_new;
        }
    }
    // Σx = Σb − Σr (the columns of A sum to one): return the residual's sum to the cells that
    // take part in the diffusion, in proportion to their coupling.
    op.apply(x, &mut ap);
    let lost: f64 = b.iter().zip(&ap).map(|(p, q)| p - q).sum();
    let weight: f64 = op.diag.iter().map(|g| g - 1.0).sum();
    if weight > 0.0 && lost != 0.0 {
        let k = lost / weight;
        x.par_iter_mut()
            .zip(op.diag.par_iter())
            .for_each(|(xv, g)| *xv += k * (g - 1.0));
    }
    iterations
}

/// Solves the tridiagonal system (lo, di, up) x = b in place of `b` (Thomas algorithm).
pub fn thomas(lo: &[f64], di: &[f64], up: &[f64], b: &mut [f64]) {
    let n = b.len();
    if n == 0 {
        return;
    }
    let mut c = vec![0.0; n];
    let mut beta = di[0];
    b[0] /= beta;
    for i in 1..n {
        c[i] = up[i - 1] / beta;
        beta = di[i] - lo[i] * c[i];
        b[i] = (b[i] - lo[i] * b[i - 1]) / beta;
    }
    for i in (0..n - 1).rev() {
        let next = b[i + 1];
        b[i] -= c[i + 1] * next;
    }
}

/// Solves the cyclic tridiagonal system with corners `alpha` (row n−1, column 0: `lo[0]`'s
/// counterpart) and `beta` (row 0, column n−1) by Sherman–Morrison, in place of `b`.
fn cyclic(lo: &[f64], di: &[f64], up: &[f64], alpha: f64, beta: f64, b: &mut [f64]) {
    // Row 0 couples to x[n−1] with `beta`; row n−1 couples to x[0] with `alpha`.
    let n = b.len();
    let gamma = -di[0];
    let mut bb = di.to_vec();
    bb[0] = di[0] - gamma;
    bb[n - 1] = di[n - 1] - alpha * beta / gamma;
    let mut l = lo.to_vec();
    let mut u = up.to_vec();
    l[0] = 0.0;
    u[n - 1] = 0.0;
    let mut x = b.to_vec();
    thomas(&l, &bb, &u, &mut x);
    let mut z = vec![0.0; n];
    z[0] = gamma;
    z[n - 1] = alpha;
    thomas(&l, &bb, &u, &mut z);
    let fact = (x[0] + beta * x[n - 1] / gamma) / (1.0 + z[0] + beta * z[n - 1] / gamma);
    for i in 0..n {
        b[i] = x[i] - fact * z[i];
    }
}

#[cfg(test)]
mod parity;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cyclic_solves_a_ring() {
        // A ring of 5 with a random right-hand side: check A·x = b.
        let n = 5;
        let k = [0.3, 1.2, 0.7, 2.0, 0.1];
        let mut lo = vec![0.0; n];
        let mut up = vec![0.0; n];
        let mut di = vec![0.0; n];
        for i in 0..n {
            let west = k[(i + n - 1) % n];
            let east = k[i];
            lo[i] = -west;
            up[i] = -east;
            di[i] = 1.0 + west + east;
        }
        let b0 = [1.0, -2.0, 3.0, 0.5, 4.0];
        let mut x = b0.to_vec();
        cyclic(&lo, &di, &up, up[n - 1], lo[0], &mut x);
        for i in 0..n {
            let ax = di[i] * x[i] + lo[i] * x[(i + n - 1) % n] + up[i] * x[(i + 1) % n];
            assert!((ax - b0[i]).abs() < 1e-10, "row {i}: {ax} vs {}", b0[i]);
        }
    }
}

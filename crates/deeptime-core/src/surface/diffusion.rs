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
    // z sweep, column by column (gathered, solved, scattered).
    let cols: Vec<Vec<f64>> = (0..w)
        .into_par_iter()
        .map(|i| {
            let mut col: Vec<f64> = (0..hh).map(|j| h[j * w + i]).collect();
            let face = |j: usize| -> f64 {
                if j + 1 < hh {
                    0.5 * (kd[j * w + i] + kd[(j + 1) * w + i]) * dt / s2
                } else {
                    0.0
                }
            };
            let mut lo = vec![0.0; hh];
            let mut di = vec![0.0; hh];
            let mut up = vec![0.0; hh];
            for j in 0..hh {
                let north = if j == 0 { 0.0 } else { face(j - 1) };
                let south = face(j);
                lo[j] = -north;
                up[j] = -south;
                di[j] = 1.0 + north + south;
            }
            thomas(&lo, &di, &up, &mut col);
            col
        })
        .collect();
    for (i, col) in cols.iter().enumerate() {
        for (j, &v) in col.iter().enumerate() {
            h[j * w + i] = v;
        }
    }
}

/// RMS residual (m) at which the conjugate gradients stop …
const CG_TOL_M: f64 = 1.0e-6;
/// … or after this many iterations.
const CG_MAX_ITER: usize = 400;

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
    fn new(d: &Dims, kd: &[f64], dt: f64) -> Operator {
        let (w, h) = (d.w, d.h);
        let s2 = d.dx * d.dx;
        let n = w * h;
        let mut east = vec![0.0; n];
        let mut south = vec![0.0; n];
        for c in 0..n {
            let (i, j) = (c % w, c / w);
            if i + 1 < w {
                east[c] = 0.5 * (kd[c] + kd[c + 1]) * dt / s2;
            } else if d.wrap_x && w > 1 {
                east[c] = 0.5 * (kd[c] + kd[c + 1 - w]) * dt / s2;
            }
            if j + 1 < h {
                south[c] = 0.5 * (kd[c] + kd[c + w]) * dt / s2;
            }
        }
        // On a two-cell ring both faces join the same pair; one cell wide, the only x face is the
        // cell itself.
        if w == 1 {
            east.iter_mut().for_each(|e| *e = 0.0);
        }
        let mut diag = vec![1.0; n];
        for c in 0..n {
            let (i, j) = (c % w, c / w);
            let west = if i > 0 {
                east[c - 1]
            } else if d.wrap_x && w > 1 {
                east[c + w - 1]
            } else {
                0.0
            };
            let north = if j > 0 { south[c - w] } else { 0.0 };
            diag[c] = 1.0 + east[c] + west + south[c] + north;
        }
        Operator {
            w,
            h,
            wrap: d.wrap_x && w > 1,
            east,
            south,
            diag,
        }
    }

    /// y = A·x, rows in parallel.
    fn apply(&self, x: &[f64], y: &mut [f64]) {
        let (w, h) = (self.w, self.h);
        y.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
            for (i, yv) in row.iter_mut().enumerate() {
                let c = j * w + i;
                let mut v = self.diag[c] * x[c];
                if i + 1 < w {
                    v -= self.east[c] * x[c + 1];
                } else if self.wrap {
                    v -= self.east[c] * x[c + 1 - w];
                }
                if i > 0 {
                    v -= self.east[c - 1] * x[c - 1];
                } else if self.wrap {
                    v -= self.east[c + w - 1] * x[c + w - 1];
                }
                if j + 1 < h {
                    v -= self.south[c] * x[c + w];
                }
                if j > 0 {
                    v -= self.south[c - w] * x[c - w];
                }
                *yv = v;
            }
        });
    }
}

/// Σ a·b as a row-ordered sum of row sums (the same result on any number of threads).
fn dot(w: usize, a: &[f64], b: &[f64]) -> f64 {
    let rows: Vec<f64> = a
        .par_chunks(w)
        .zip(b.par_chunks(w))
        .map(|(x, y)| x.iter().zip(y).map(|(p, q)| p * q).sum::<f64>())
        .collect();
    rows.iter().sum()
}

/// Refines `x` to the solution of A·x = b by Jacobi-preconditioned conjugate gradients, then puts
/// the residual's sum back so Σx = Σb; returns the iterations.
fn cg(d: &Dims, b: &[f64], x: &mut [f64], kd: &[f64], dt: f64) -> usize {
    let n = d.cells();
    let w = d.w;
    let op = Operator::new(d, kd, dt);
    let mut ax = vec![0.0; n];
    op.apply(x, &mut ax);
    let mut r: Vec<f64> = b.iter().zip(&ax).map(|(p, q)| p - q).collect();
    let tol2 = CG_TOL_M * CG_TOL_M * n as f64;
    let mut iterations = 0;
    if dot(w, &r, &r) > tol2 {
        let mut z: Vec<f64> = r.iter().zip(&op.diag).map(|(p, q)| p / q).collect();
        let mut p = z.clone();
        let mut rz = dot(w, &r, &z);
        let mut ap = vec![0.0; n];
        while iterations < CG_MAX_ITER {
            iterations += 1;
            op.apply(&p, &mut ap);
            let pap = dot(w, &p, &ap);
            if pap <= 0.0 {
                break;
            }
            let alpha = rz / pap;
            x.par_iter_mut()
                .zip(p.par_iter())
                .for_each(|(xv, pv)| *xv += alpha * pv);
            r.par_iter_mut()
                .zip(ap.par_iter())
                .for_each(|(rv, av)| *rv -= alpha * av);
            if dot(w, &r, &r) <= tol2 {
                break;
            }
            z.par_iter_mut()
                .zip(r.par_iter().zip(op.diag.par_iter()))
                .for_each(|(zv, (rv, dv))| *zv = rv / dv);
            let rz_new = dot(w, &r, &z);
            let beta = rz_new / rz;
            rz = rz_new;
            p.par_iter_mut()
                .zip(z.par_iter())
                .for_each(|(pv, zv)| *pv = zv + beta * *pv);
        }
    }
    // Σx = Σb − Σr (the columns of A sum to one): return the residual's sum to the cells that
    // take part in the diffusion, in proportion to their coupling.
    op.apply(x, &mut ax);
    let lost: f64 = b.iter().zip(&ax).map(|(p, q)| p - q).sum();
    let weight: f64 = op.diag.iter().map(|g| g - 1.0).sum();
    if weight > 0.0 && lost != 0.0 {
        let k = lost / weight;
        for (xv, g) in x.iter_mut().zip(&op.diag) {
            *xv += k * (g - 1.0);
        }
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

// SPDX-License-Identifier: LGPL-3.0-or-later
//! Implicit linear diffusion ∂h/∂t = ∇·(D ∇h) with a diffusivity per cell: hillslope creep on land
//! (the preset's κ times the surface rock's diffusivity multiplier) and depth-dependent marine
//! transport in the sea (`marine.rs`), in one solve.
//!
//! **Scheme:** locally one-dimensional backward Euler (the fractional-step splitting of the ADI
//! family): a fully implicit sweep along x — one cyclic tridiagonal system per row, since the grid
//! wraps — then one along z — a tridiagonal system per column with no flux through the first and
//! last row. Peaceman–Rachford ADI would be second order but oscillates at the time steps deep
//! time uses (D·Δt/Δx² reaches ~10³ on the shelf over 2 Myr); backward Euler per direction is
//! unconditionally stable and monotone. Faces take the arithmetic mean of their two cells'
//! diffusivities (as Landlab's `LinearDiffuser` maps node diffusivities to links).
//!
//! Every sweep is in flux form with zero flux through the closed rows, so the matrix columns sum
//! to one and **Σh is conserved** (to round-off) — the mass-conservation property the tests
//! check. Rows and columns are solved in parallel.

use rayon::prelude::*;

use super::grid::Dims;

/// One implicit step of Δt years on `h` with diffusivities `kd` (m²/yr).
pub fn step(d: &Dims, h: &mut [f64], kd: &[f64], dt: f64) {
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

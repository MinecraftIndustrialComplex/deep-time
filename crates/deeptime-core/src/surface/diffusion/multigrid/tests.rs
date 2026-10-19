// SPDX-License-Identifier: LGPL-3.0-or-later
//! The multigrid-preconditioned solve against a plain conjugate-gradient solve written from the
//! definition of the system, and against the Jacobi-preconditioned solve it replaces: within the
//! tolerance both stop at, with the mass kept, on small, odd, wrapping and non-wrapping grids; and
//! the cycle is the symmetric positive definite operator the conjugate gradients need.

use std::time::Instant;

use super::super::{Live, Operator, Workspace, lod, step};
use super::*;
use crate::sphere::SplitMix64;
use crate::surface::marine::submarine_kd;

/// (I + Δt·L)·x from the definition: five points, faces by the arithmetic mean of the two cells'
/// diffusivities, wrapping in x when the grid does (two faces between the cells of a two-cell
/// ring, none on a one-cell one), no flux through the first and last rows.
fn apply_reference(d: &Dims, kd: &[f64], dt: f64, x: &[f64], y: &mut [f64]) {
    let (w, h) = (d.w, d.h);
    let s2 = d.dx * d.dx;
    let wrap = d.wrap_x && w > 1;
    for j in 0..h {
        for i in 0..w {
            let c = j * w + i;
            let east = if i + 1 < w {
                Some(c + 1)
            } else {
                wrap.then_some(j * w)
            };
            let west = if i > 0 {
                Some(c - 1)
            } else {
                wrap.then_some(j * w + w - 1)
            };
            let south = (j + 1 < h).then_some(c + w);
            let north = (j > 0).then_some(c.wrapping_sub(w));
            let mut v = x[c];
            for m in [east, west, south, north].into_iter().flatten() {
                v += 0.5 * (kd[c] + kd[m]) * dt / s2 * (x[c] - x[m]);
            }
            y[c] = v;
        }
    }
}

/// The solution to a relative residual of 1e-13, by unpreconditioned conjugate gradients.
fn reference(d: &Dims, b: &[f64], kd: &[f64], dt: f64) -> Vec<f64> {
    let n = d.cells();
    let mut x = b.to_vec();
    let mut ax = vec![0.0; n];
    apply_reference(d, kd, dt, &x, &mut ax);
    let mut r: Vec<f64> = b.iter().zip(&ax).map(|(p, q)| p - q).collect();
    let mut p = r.clone();
    let mut rr: f64 = r.iter().map(|v| v * v).sum();
    let bb: f64 = b.iter().map(|v| v * v).sum::<f64>().max(1.0);
    for _ in 0..20 * n + 100 {
        if rr <= 1e-26 * bb {
            break;
        }
        apply_reference(d, kd, dt, &p, &mut ax);
        let pap: f64 = p.iter().zip(&ax).map(|(a, b)| a * b).sum();
        let alpha = rr / pap;
        for k in 0..n {
            x[k] += alpha * p[k];
            r[k] -= alpha * ax[k];
        }
        let rr_new: f64 = r.iter().map(|v| v * v).sum();
        let beta = rr_new / rr;
        rr = rr_new;
        for k in 0..n {
            p[k] = r[k] + beta * p[k];
        }
    }
    x
}

/// A terrain of a few octaves of value noise, metres (a coast through the middle of its range).
fn terrain(w: usize, h: usize, seed: u64) -> Vec<f64> {
    let mut rng = SplitMix64::new(seed);
    let mut z = vec![0.0; w * h];
    for oct in 0..6 {
        let g = 2usize << oct;
        let grid: Vec<f64> = (0..g * g).map(|_| rng.next_f64() - 0.5).collect();
        let amp = 3000.0 / f64::from(1 << oct);
        for j in 0..h {
            let fy = j as f64 / h.max(1) as f64 * (g - 1) as f64;
            let (j0, ty) = (fy.floor() as usize, fy.fract());
            for i in 0..w {
                let fx = i as f64 / w as f64 * g as f64;
                let (i0, tx) = (fx.floor() as usize, fx.fract());
                let at = |a: usize, b: usize| grid[b.min(g - 1) * g + a % g];
                let top = at(i0, j0) * (1.0 - tx) + at(i0 + 1, j0) * tx;
                let bot = at(i0, j0 + 1) * (1.0 - tx) + at(i0 + 1, j0 + 1) * tx;
                z[j * w + i] += amp * (top * (1.0 - ty) + bot * ty);
            }
        }
    }
    z
}

/// The surface stage's diffusivities on `z` (sea level 0): the preset defaults' marine transport
/// and a land value with a per-cell spread like the rock multipliers'.
fn shelf_kd(z: &[f64], seed: u64) -> Vec<f64> {
    let mut rng = SplitMix64::new(seed ^ 0x5eed);
    z.iter()
        .map(|&h| {
            let land = 2.0 * (0.25 + 4.0 * rng.next_f64());
            submarine_kd(-h, 100.0, 60.0, 2.0, land)
        })
        .collect()
}

/// The surface stage's solve with the multigrid preconditioner: the splitting, then the cycle's
/// conjugate gradients.
fn solve_mg(
    d: &Dims,
    b: &[f64],
    kd: &[f64],
    dt: f64,
    tol: f64,
    cy: Cycle,
    guess: bool,
) -> (Vec<f64>, usize) {
    let mut x = b.to_vec();
    if guess {
        lod(d, &mut x, kd, dt, &Live::all(d));
    }
    let mut ws = Workspace::default();
    ws.fit_rhs(d);
    ws.b.copy_from_slice(b);
    ws.fit(d);
    let its = cg_with(&mut ws, d, &mut x, kd, dt, tol, cy);
    (x, its)
}

/// The Jacobi-preconditioned solve, `diffusion::step` (every row visited: no diffusivity here is
/// zero).
fn solve_jacobi(d: &Dims, b: &[f64], kd: &[f64], dt: f64) -> (Vec<f64>, usize) {
    let mut x = b.to_vec();
    let its = step(d, &mut x, kd, dt);
    (x, its)
}

/// The operator's three vectors for `d`, for [`Operator::new`] to fill.
fn op_vectors(d: &Dims) -> [Vec<f64>; 3] {
    let n = d.cells();
    [vec![0.0; n], vec![0.0; n], vec![0.0; n]]
}

fn rms(a: &[f64], b: &[f64]) -> f64 {
    let s: f64 = a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum();
    (s / a.len().max(1) as f64).sqrt()
}

/// The RMS error a solve stopped at an RMS residual of `tol` may have: at most `tol` before the
/// residual's sum is put back (the eigenvalues of I + Δt·L are at least one), plus that put-back,
/// which is at most `tol` times the RMS of the coupling over its mean.
fn bound(d: &Dims, kd: &[f64], dt: f64, tol: f64) -> f64 {
    let [mut east, mut south, mut diag] = op_vectors(d);
    let op = Operator::new(d, kd, dt, &Live::all(d), &mut east, &mut south, &mut diag);
    let n = op.diag.len() as f64;
    let mean = op.diag.iter().map(|g| g - 1.0).sum::<f64>() / n;
    let rms = (op.diag.iter().map(|g| (g - 1.0) * (g - 1.0)).sum::<f64>() / n).sqrt();
    tol * (1.0 + if mean > 0.0 { rms / mean } else { 0.0 }) * 1.01
}

const SIZES: [(usize, usize); 12] = [
    (1, 1),
    (1, 7),
    (7, 1),
    (2, 2),
    (2, 5),
    (3, 5),
    (5, 3),
    (17, 9),
    (33, 33),
    (64, 40),
    (129, 67),
    (200, 96),
];

#[test]
fn the_multigrid_solve_meets_its_tolerance_and_keeps_the_mass() {
    let mut solved = 0;
    for &(w, h) in &SIZES {
        for wrap in [true, false] {
            for seed in 1..3u64 {
                let d = Dims::new(w, h, 2400.0, wrap);
                let b = terrain(w, h, seed * 7 + w as u64);
                let kd = shelf_kd(&b, seed);
                for &(dt, tol) in &[(2.0e6, 1.0e-6), (2.0e6, 1.0e-3), (2.0e4, 1.0e-6)] {
                    let label = format!("{w}x{h} wrap {wrap} seed {seed} dt {dt} tol {tol}");
                    let exact = reference(&d, &b, &kd, dt);
                    let (x, its) = solve_mg(&d, &b, &kd, dt, tol, CYCLE, true);
                    assert!(its < super::super::CG_MAX_ITER, "{label}: {its} iterations");
                    let err = rms(&x, &exact);
                    let lim = bound(&d, &kd, dt, tol);
                    assert!(err <= lim, "{label}: RMS error {err:.3e} above {lim:.3e}");
                    let (sb, sx): (f64, f64) = (b.iter().sum(), x.iter().sum());
                    let scale: f64 = b.iter().map(|v| v.abs()).sum::<f64>().max(1.0);
                    assert!(
                        (sb - sx).abs() <= 1e-12 * scale,
                        "{label}: mass {sb} vs {sx}"
                    );
                    // Without the splitting's first guess, too.
                    let (x0, _) = solve_mg(&d, &b, &kd, dt, tol, CYCLE, false);
                    let err0 = rms(&x0, &exact);
                    assert!(
                        err0 <= lim,
                        "{label}, no guess: RMS error {err0:.3e} above {lim:.3e}"
                    );
                    solved += 1;
                }
            }
        }
    }
    assert!(solved >= 100);
}

#[test]
fn the_multigrid_and_jacobi_solves_agree_within_their_tolerance() {
    for &(w, h) in &[(64usize, 40usize), (129, 67), (5, 3)] {
        for wrap in [true, false] {
            let d = Dims::new(w, h, 2400.0, wrap);
            let b = terrain(w, h, 11 + w as u64);
            let kd = shelf_kd(&b, 3);
            let dt = 2.0e6;
            let tol = super::super::CG_TOL_M;
            let (xm, _) = solve_mg(&d, &b, &kd, dt, tol, CYCLE, true);
            let (xj, _) = solve_jacobi(&d, &b, &kd, dt);
            let lim = 2.0 * bound(&d, &kd, dt, tol);
            let diff = rms(&xm, &xj);
            assert!(
                diff <= lim,
                "{w}x{h} wrap {wrap}: RMS {diff:.3e} above {lim:.3e}"
            );
        }
    }
}

/// The conjugate gradients need M⁻¹ symmetric and positive definite: ⟨M⁻¹r₁, r₂⟩ = ⟨r₁, M⁻¹r₂⟩
/// (to single precision, the cycle's) and ⟨M⁻¹r, r⟩ > 0.
#[test]
fn the_cycle_is_a_symmetric_positive_definite_operator() {
    for &(w, h, wrap) in &[
        (64usize, 40usize, true),
        (37, 23, false),
        (5, 3, true),
        (2, 9, true),
        (1, 1, false),
    ] {
        let d = Dims::new(w, h, 2400.0, wrap);
        let z = terrain(w, h, 5);
        let kd = shelf_kd(&z, 5);
        let [mut east, mut south, mut diag] = op_vectors(&d);
        let live = Live::all(&d);
        let op = Operator::new(&d, &kd, 2.0e6, &live, &mut east, &mut south, &mut diag);
        for cy in [
            CYCLE,
            Cycle {
                omega: 0.7,
                nu: 2,
                scale: 1.0,
            },
        ] {
            let mut mg = Hierarchy::new(&op, cy);
            let mut rng = SplitMix64::new(9);
            let r1: Vec<f64> = (0..w * h).map(|_| rng.next_f64() - 0.5).collect();
            let r2: Vec<f64> = (0..w * h).map(|_| rng.next_f64() - 0.5).collect();
            let mut z_of = |r: &[f64]| -> (Vec<f64>, f64) {
                for (a, &b) in mg.levels[0].r.iter_mut().zip(r) {
                    *a = b as f32;
                }
                let rz = mg.precondition(r);
                (mg.z().iter().map(|&v| f64::from(v)).collect(), rz)
            };
            let (z1, rz1) = z_of(&r1);
            let (z2, rz2) = z_of(&r2);
            let dot = |a: &[f64], b: &[f64]| a.iter().zip(b).map(|(p, q)| p * q).sum::<f64>();
            assert!(
                rz1 > 0.0 && rz2 > 0.0,
                "{w}x{h}: not positive ({rz1}, {rz2})"
            );
            assert!((rz1 - dot(&z1, &r1)).abs() <= 1e-9 * rz1.abs().max(1e-30));
            let (a, b) = (dot(&z1, &r2), dot(&r1, &z2));
            let lim = 1e-4 * (rz1 * rz2).sqrt();
            assert!((a - b).abs() <= lim, "{w}x{h} {cy:?}: {a} vs {b}");
        }
    }
}

#[test]
fn the_cycle_takes_far_fewer_iterations_than_jacobi_on_a_shelf() {
    let (w, h) = (256, 128);
    let d = Dims::new(w, h, 2400.0, true);
    let b = terrain(w, h, 21);
    let kd = shelf_kd(&b, 21);
    let dt = 2.0e6;
    let (_, mg) = solve_mg(&d, &b, &kd, dt, super::super::CG_TOL_M, CYCLE, true);
    let (_, jac) = solve_jacobi(&d, &b, &kd, dt);
    assert!(3 * mg <= jac, "multigrid {mg} iterations, Jacobi {jac}");
}

/// The settings sweep behind [`OMEGA`], [`NU`] and [`SCALE`] (P1-19): iterations and time per
/// solve on a shelf-and-continent grid the size of a full 16k surface band, for each cycle, with
/// and without the splitting's first guess, beside the Jacobi solve. Run by hand:
/// `cargo test --release -p deeptime-core tune_the_cycle -- --ignored --nocapture`.
#[test]
#[ignore]
fn tune_the_cycle() {
    let (w, h) = (
        std::env::var("MG_W")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2048usize),
        std::env::var("MG_H")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1024usize),
    );
    let d = Dims::new(w, h, 2400.0, true);
    let b = terrain(w, h, 3);
    let kd = shelf_kd(&b, 3);
    let dt = 2.0e6;
    let reps = 3;
    let time = |f: &dyn Fn() -> (Vec<f64>, usize)| -> (f64, usize, Vec<f64>) {
        let mut best = f64::MAX;
        let mut last = (Vec::new(), 0);
        for _ in 0..reps {
            let t = Instant::now();
            last = f();
            best = best.min(t.elapsed().as_secs_f64());
        }
        (best, last.1, last.0)
    };
    let (tj, ij, xj) = time(&|| solve_jacobi(&d, &b, &kd, dt));
    let (_, _, exact) = time(&|| solve_mg(&d, &b, &kd, dt, 1e-10, CYCLE, true));
    println!(
        "tune {w}x{h} threads {} jacobi: {ij} its {tj:.3} s err {:.2e}",
        rayon::current_num_threads(),
        rms(&xj, &exact)
    );
    for tol in [1e-6, 1e-5, 1e-4, 1e-3] {
        for nu in [1usize, 2] {
            for omega in [0.6f32, 0.7, 0.8, 0.9, 1.0] {
                for scale in [0.5f32, 0.6, 0.75, 1.0] {
                    for guess in [true, false] {
                        let cy = Cycle { omega, nu, scale };
                        let (t, its, x) = time(&|| solve_mg(&d, &b, &kd, dt, tol, cy, guess));
                        println!(
                            "tune tol {tol:e} nu {nu} omega {omega} scale {scale} guess {guess}: \
                             {its} its {t:.3} s err {:.2e} vs_jacobi {:.2e}",
                            rms(&x, &exact),
                            rms(&x, &xj)
                        );
                    }
                }
            }
        }
    }
}

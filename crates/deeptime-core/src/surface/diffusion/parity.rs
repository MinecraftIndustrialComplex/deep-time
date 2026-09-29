// SPDX-License-Identifier: LGPL-3.0-or-later
//! The fused diffusion solve against the textbook loop it replaced (P1-17). `reference` keeps
//! P1-04's `step`, `lod`, `Operator`, `dot` and `cg` verbatim (seven parallel regions per
//! iteration, `dot` as row sums collected and added in row order); every comparison is on the bits
//! of the floats, not on a tolerance, because a resumed run has to reproduce a run at another
//! thread count exactly. The grids are random, wrap in x (and, in some cases, do not), carry the
//! diffusivity pattern of a real surface (land κ, marine transport falling with depth below the
//! wave base, the open sea beside the coast) or the stiff extremes (zero, 1e-20, 1e5), have widths
//! and heights that the strip of rows does not divide, and are solved on pools of 1, 2, 4 and 8
//! threads.

use super::super::grid::Dims;
use super::super::marine::submarine_kd;
use crate::sphere::SplitMix64;

/// P1-04's solver, as it was (only the visibility of the three timing wrappers at the end is new).
#[allow(dead_code)]
mod reference {
    use super::Dims;
    use rayon::prelude::*;

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

    pub fn time_lod(d: &Dims, h: &mut [f64], kd: &[f64], dt: f64) {
        lod(d, h, kd, dt);
    }

    pub fn time_operator(d: &Dims, kd: &[f64], dt: f64) {
        std::hint::black_box(Operator::new(d, kd, dt));
    }

    pub fn time_cg(d: &Dims, b: &[f64], x: &mut [f64], kd: &[f64], dt: f64) -> usize {
        cg(d, b, x, kd, dt)
    }
}

fn in_pool<R: Send>(threads: usize, f: impl FnOnce() -> R + Send) -> R {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap()
        .install(f)
}

/// A field on a wrapping raster made of value noise, so that neighbouring cells are related the
/// way terrain is (eight octaves, as the routing tests' `fractal`).
fn fractal(w: usize, hgt: usize, seed: u64) -> Vec<f64> {
    let mut rng = SplitMix64::new(seed);
    let mut z = vec![0.0; w * hgt];
    for oct in 0..8 {
        let (gw, gh) = (4usize << oct, 2usize << oct);
        let g: Vec<f64> = (0..gw * gh).map(|_| rng.next_f64() - 0.5).collect();
        let amp = 4000.0 / f64::from(1 << oct).powf(0.9);
        for j in 0..hgt {
            let fy = j as f64 / hgt as f64 * gh as f64;
            let (j0, ty) = (fy.floor() as usize, fy.fract());
            for i in 0..w {
                let fx = i as f64 / w as f64 * gw as f64;
                let (i0, tx) = (fx.floor() as usize, fx.fract());
                let at = |a: usize, b: usize| g[b.min(gh - 1) * gw + a % gw];
                let top = at(i0, j0) * (1.0 - tx) + at(i0 + 1, j0) * tx;
                let bot = at(i0, j0 + 1) * (1.0 - tx) + at(i0 + 1, j0 + 1) * tx;
                z[j * w + i] += amp * (top * (1.0 - ty) + bot * ty);
            }
        }
    }
    z
}

struct Case {
    d: Dims,
    h: Vec<f64>,
    kd: Vec<f64>,
    dt: f64,
    /// Sea level (m) the diffusivities were made for.
    sea: f64,
}

/// Elevations of ±2000 m of white noise and the stiff diffusivity pattern of the mass test
/// (0, 1e-20, 5, 300, 1e5 cyclically).
fn rough(w: usize, hgt: usize, seed: u64, wrap: bool) -> Case {
    let d = Dims::new(w, hgt, 4800.0, wrap);
    let mut rng = SplitMix64::new(seed);
    let n = d.cells();
    let h = (0..n).map(|_| rng.next_f64() * 4000.0 - 2000.0).collect();
    let kd = (0..n)
        .map(|c| match (c + seed as usize) % 5 {
            0 => 0.0,
            1 => 1.0e-20,
            2 => 5.0,
            3 => 300.0,
            _ => 1.0e5,
        })
        .collect();
    Case {
        d,
        h,
        kd,
        dt: 2.0e6,
        sea: 0.0,
    }
}

/// A fractal terrain with `sea_pct` per cent of it below sea level and the surface stage's
/// diffusivity: κ × a rock multiplier on land, `submarine_kd` in the sea (so the open sea is a
/// large connected region of 300 m²/yr next to a 2 m²/yr coast).
fn shore(w: usize, hgt: usize, seed: u64, wrap: bool, sea_pct: usize) -> Case {
    let d = Dims::new(w, hgt, 4800.0, wrap);
    let n = d.cells();
    let h = fractal(w, hgt, seed);
    let mut sorted = h.clone();
    sorted.sort_by(f64::total_cmp);
    let sea = sorted[(n * sea_pct / 100).min(n - 1)];
    let mut rng = SplitMix64::new(seed ^ 0x5eed);
    let kd = h
        .iter()
        .map(|&z| {
            let land = 2.0 * (0.3 + 2.7 * rng.next_f64());
            submarine_kd(sea - z, 300.0, 100.0, 2.0, land)
        })
        .collect();
    Case {
        d,
        h,
        kd,
        dt: 2.0e6,
        sea,
    }
}

fn same(what: &str, a: &[f64], b: &[f64]) {
    assert_eq!(a.len(), b.len(), "{what}: length");
    for (i, (x, y)) in a.iter().zip(b).enumerate() {
        assert_eq!(x.to_bits(), y.to_bits(), "{what}[{i}]: {x} vs {y}");
    }
}

/// `steps` steps in a row with the textbook solver: the field and the iteration count of each.
fn old(c: &Case, steps: usize, threads: usize) -> (Vec<f64>, Vec<usize>) {
    in_pool(threads, || {
        let mut h = c.h.clone();
        let its = (0..steps)
            .map(|_| reference::step(&c.d, &mut h, &c.kd, c.dt))
            .collect();
        (h, its)
    })
}

/// The same with the fused solver.
fn new(c: &Case, steps: usize, threads: usize) -> (Vec<f64>, Vec<usize>) {
    in_pool(threads, || {
        let mut h = c.h.clone();
        let its = (0..steps)
            .map(|_| super::step(&c.d, &mut h, &c.kd, c.dt))
            .collect();
        (h, its)
    })
}

/// Compares the two on pools of 1, 2, 4 and 8 threads (and the textbook solver with itself: the
/// reference is thread-count independent too, which is what the row-ordered sums are for);
/// returns the largest iteration count seen.
fn check(c: &Case, steps: usize, what: &str) -> usize {
    let (want, want_its) = old(c, steps, 1);
    let mut most = 0;
    for threads in [1, 2, 4, 8] {
        let tag = format!("{what}, {threads} threads");
        let (got, its) = new(c, steps, threads);
        same(&format!("{tag}: h"), &got, &want);
        assert_eq!(its, want_its, "{tag}: iterations");
        if threads > 1 {
            let (again, its) = old(c, steps, threads);
            same(&format!("{tag}: reference h"), &again, &want);
            assert_eq!(its, want_its, "{tag}: reference iterations");
        }
        most = most.max(its.into_iter().max().unwrap_or(0));
    }
    most
}

#[test]
fn the_empty_sum_starts_where_the_serial_sums_did() {
    // The fused row sums start from `sum_zero()`: the value `.sum()` folds from, so that a row of
    // negative zeros sums to the same signed zero either way.
    let empty = std::iter::empty::<f64>().sum::<f64>();
    assert_eq!(super::sum_zero().to_bits(), empty.to_bits());
    let negz = [-0.0f64, -0.0, -0.0];
    let serial: f64 = negz.iter().sum();
    let mut fused = super::sum_zero();
    for v in negz {
        fused += v;
    }
    assert_eq!(fused.to_bits(), serial.to_bits());
}

#[test]
fn row_sums_are_the_serial_sums_of_each_row() {
    let mut rng = SplitMix64::new(3);
    for (w, rows) in [(1, 1), (3, 2), (17, 4), (64, 5), (33, 8), (100, 11)] {
        let a: Vec<f64> = (0..w * rows).map(|_| rng.next_f64() * 2.0 - 1.0).collect();
        let b: Vec<f64> = (0..w * rows).map(|_| rng.next_f64() * 2.0 - 1.0).collect();
        let mut got = vec![0.0; rows];
        super::row_dots(w, &a, &b, &mut got);
        for k in 0..rows {
            let want: f64 = a[k * w..(k + 1) * w]
                .iter()
                .zip(&b[k * w..(k + 1) * w])
                .map(|(p, q)| p * q)
                .sum();
            assert_eq!(got[k].to_bits(), want.to_bits(), "{w}x{rows} row {k}");
        }
    }
}

#[test]
fn the_fused_solve_is_the_textbook_solve_on_rough_stiff_grids() {
    // Wrapping, with the zero and 1e5 diffusivities of the mass test.
    let mut most = 0;
    for seed in 1..5 {
        most = most.max(check(&rough(160, 96, seed, true), 1, "rough wrapping"));
    }
    assert!(most > 5, "the CG loop must run ({most} iterations)");
    // Three steps in a row (the second and third start from a smoothed field).
    check(&rough(96, 40, 9, true), 3, "rough, three steps");
}

#[test]
fn the_fused_solve_is_the_textbook_solve_at_the_shore_and_in_the_open_sea() {
    // The surface stage's diffusivities: a coast at 2 m²/yr against a sea of 300 m²/yr falling with
    // depth, 5 %, 40 % and 78 % of the cells in the sea (the earthlike share), wrapping and not.
    let mut most = 0;
    for (seed, sea, wrap) in [
        (1, 78, true),
        (2, 78, true),
        (3, 40, true),
        (4, 5, true),
        (5, 78, false),
        (6, 100, true),
    ] {
        most = most.max(check(
            &shore(192, 96, seed, wrap, sea),
            1,
            &format!("shore, sea {sea} %, seed {seed}"),
        ));
    }
    assert!(most > 20, "a real coast needs many iterations ({most})");
    check(&shore(128, 64, 7, true, 78), 4, "shore, four steps");
}

#[test]
fn the_fused_solve_is_the_textbook_solve_on_awkward_shapes() {
    // Widths and heights around the strip of rows, one row, one column, two, three.
    for (w, hgt, wrap) in [
        (1, 1, true),
        (1, 9, true),
        (9, 1, true),
        (2, 2, true),
        (2, 5, true),
        (3, 7, true),
        (5, 3, false),
        (7, 8, true),
        (7, 9, true),
        (16, 7, true),
        (33, 17, false),
        (40, 15, true),
        (50, 17, true),
        (64, 16, true),
        (64, 24, true),
    ] {
        check(
            &rough(w, hgt, 11, wrap),
            1,
            &format!("rough {w}x{hgt} wrap {wrap}"),
        );
        check(
            &shore(w, hgt, 12, wrap, 60),
            2,
            &format!("shore {w}x{hgt} wrap {wrap}"),
        );
    }
}

#[test]
fn the_fused_solve_is_the_textbook_solve_when_there_is_nothing_to_solve_or_too_much() {
    let mut c = rough(64, 32, 5, true);
    // No diffusion at all: the residual is zero, the loop never starts.
    c.kd.iter_mut().for_each(|k| *k = 0.0);
    assert_eq!(check(&c, 1, "kd = 0"), 0);
    // One constant diffusivity, and the field constant too.
    c.kd.iter_mut().for_each(|k| *k = 50.0);
    check(&c, 2, "kd constant");
    c.h.iter_mut().for_each(|v| *v = 100.0);
    check(&c, 1, "flat field");
    // A tolerance that a long Δt with 1e5 against 1e-20 keeps out of reach: the iteration cap.
    let mut c = rough(128, 64, 8, true);
    c.dt = 2.0e8;
    let most = check(&c, 1, "capped");
    assert!(most >= 50, "stiff grid took {most} iterations");
}

/// Times each phase of one diffusion step on an L1-sized terrain (`cargo test --release -p
/// deeptime-core diffusion_timing -- --ignored --nocapture`, `DEEPTIME_TIMING_REPS` repetitions,
/// 12 by default): the textbook solver beside the fused one on the earthlike diffusivities, at
/// each thread count. The two are timed in interleaved pairs (old, new, old, new, …), so that a
/// machine whose load moves affects both alike; minimum and median of each are printed, and the
/// calibration loop shows what a perfectly parallel job gets from the machine at that moment.
#[test]
#[ignore = "a timing, not a check"]
fn diffusion_timing() {
    let c = shore(1024, 512, 7, true, 78);
    let (d, n) = (c.d, c.d.cells());
    let reps = std::env::var("DEEPTIME_TIMING_REPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(12);
    let ms = |f: &mut dyn FnMut()| {
        let t = std::time::Instant::now();
        f();
        t.elapsed().as_secs_f64() * 1e3
    };
    let best = |f: &mut dyn FnMut()| (0..reps).map(|_| ms(f)).fold(f64::INFINITY, f64::min);
    // (min, median) of `reps` interleaved pairs.
    let pairs = |a: &mut dyn FnMut(), b: &mut dyn FnMut()| {
        let (mut ta, mut tb) = (Vec::new(), Vec::new());
        for _ in 0..reps {
            ta.push(ms(a));
            tb.push(ms(b));
        }
        let stat = |mut v: Vec<f64>| {
            v.sort_by(f64::total_cmp);
            (v[0], v[v.len() / 2])
        };
        (stat(ta), stat(tb))
    };
    println!(
        "{n} cells; ms per step, min/median of {reps} interleaved pairs (old = textbook loop, new = fused passes)"
    );
    // What the stage itself does around the solve (surface/mod.rs, serial): the diffusivity fill.
    let mut kd = vec![0.0; n];
    let fill = best(&mut || {
        for (k, &z) in kd.iter_mut().zip(&c.h) {
            *k = submarine_kd(c.sea - z, 300.0, 100.0, 2.0, 2.0);
        }
        std::hint::black_box(&kd);
    });
    println!("kd fill in surface/mod.rs (serial): {fill:.1} ms");
    for threads in [1, 2, 4, 8, 12] {
        in_pool(threads, || {
            let cal = best(&mut || {
                use rayon::prelude::*;
                let s: f64 = (0..4_000_000u32)
                    .into_par_iter()
                    .map(|i| f64::from(i).sqrt().sin())
                    .sum();
                std::hint::black_box(s);
            });
            // The state after the locally one-dimensional guess: what the solves start from.
            let b = c.h.clone();
            let mut x0 = c.h.clone();
            super::lod(&d, &mut x0, &c.kd, c.dt);
            let (lod_old, lod_new) = pairs(
                &mut || {
                    let mut x = c.h.clone();
                    reference::time_lod(&d, &mut x, &c.kd, c.dt);
                    std::hint::black_box(&x);
                },
                &mut || {
                    let mut x = c.h.clone();
                    super::lod(&d, &mut x, &c.kd, c.dt);
                    std::hint::black_box(&x);
                },
            );
            let (op_old, op_new) = pairs(
                &mut || reference::time_operator(&d, &c.kd, c.dt),
                &mut || {
                    std::hint::black_box(super::Operator::new(&d, &c.kd, c.dt));
                },
            );
            let mut its = (0, 0);
            let (cg_old, cg_new) = pairs(
                &mut || {
                    let mut x = x0.clone();
                    its.0 = reference::time_cg(&d, &b, &mut x, &c.kd, c.dt);
                },
                &mut || {
                    let mut x = x0.clone();
                    its.1 = super::cg(&d, &b, &mut x, &c.kd, c.dt);
                },
            );
            let (step_old, step_new) = pairs(
                &mut || {
                    let mut h = c.h.clone();
                    reference::step(&d, &mut h, &c.kd, c.dt);
                },
                &mut || {
                    let mut h = c.h.clone();
                    super::step(&d, &mut h, &c.kd, c.dt);
                },
            );
            println!(
                "{threads:>2} threads (calibration {cal:.1} ms): lod old {:.1}/{:.1} new {:.1}/{:.1} | operator old {:.1}/{:.1} new {:.1}/{:.1} | cg ({} / {} it) old {:.1}/{:.1} new {:.1}/{:.1} = {:.2} → {:.2} ms/it (min) | step old {:.1}/{:.1} new {:.1}/{:.1}",
                lod_old.0,
                lod_old.1,
                lod_new.0,
                lod_new.1,
                op_old.0,
                op_old.1,
                op_new.0,
                op_new.1,
                its.0,
                its.1,
                cg_old.0,
                cg_old.1,
                cg_new.0,
                cg_new.1,
                cg_old.0 / its.0 as f64,
                cg_new.0 / its.1 as f64,
                step_old.0,
                step_old.1,
                step_new.0,
                step_new.1,
            );
        });
    }
}

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
//! updates r and reduces r·r and r·z. Since P1-19 the preconditioned residual z = r/diag is formed
//! where it is used instead of stored, and the update x += α·p, which nothing reads until the loop
//! ends, is taken by the next iteration's first pass (which reads p anyway) or once after the
//! loop: three of the eighteen vector streams an iteration moved. The arithmetic is the textbook loop's, cell
//! by cell and operation by operation (no fused multiply-add, no reciprocal in place of a
//! division, terms of A·p added in the same order); only the schedule differs. Reductions keep
//! the old scheme exactly: a sum per row in ascending column order, then the row sums added in
//! ascending row order on one thread. Nothing depends on the strip height or the thread pool, so
//! results are bit-identical to the textbook loop at any thread count; `diffusion/parity.rs` keeps
//! that loop verbatim and compares bit patterns.
//!
//! **Rows with nothing to diffuse (P1-19).** The crust's spreading (`spread.rs`) diffuses only the
//! cells above the strength height, a few per cent of the grid, through this same solve. A row
//! whose diffusivity is +0 everywhere, with its two neighbours' +0 too, is a block of the identity
//! in (I + Δt·L) that nothing couples to, and the textbook loop leaves its cells as they were:
//! every term it adds to them is a signed zero (a +0 coupling times a finite value), and h ± 0 = h
//! when h is not itself zero; its residual, direction and A·p stay +0, and so do its row sums. A
//! step therefore visits only the strips of [`STRIP_ROWS`] rows that hold a row with something
//! to diffuse, leaves the others as they are with the row sums (+0) the loop forms for them, and
//! solves the tridiagonal sweeps along z one run of visited rows at a time (a skipped row cuts a
//! column's system in two: the elimination passes only signed zeros across it). The reasoning
//! needs normal elevations (not zero, not subnormal, not within a factor of two of subnormal) in
//! every row of zero diffusivity, a positive finite Δt and cell size, and finite results; a step
//! that has another elevation there visits every row, and one that ends with a value that is not
//! finite is solved again on
//! every row, so the result is the textbook loop's in every case. The parity tests compare both
//! with the textbook loop bit for bit on grids that diffuse in a few blobs.

use rayon::prelude::*;

use super::grid::Dims;

/// One implicit step of Δt years on `h` with diffusivities `kd` (m²/yr); returns the conjugate
/// gradient iterations it took.
pub fn step(d: &Dims, h: &mut [f64], kd: &[f64], dt: f64) -> usize {
    let live = Live::of(d, h, kd, dt);
    let b = crate::alloc::copied(h);
    lod(d, h, kd, dt, &live);
    let its = cg(d, &b, h, kd, dt, &live);
    if live.strips.is_some() && !live.finite(d, h) {
        // A value that is not finite would reach the skipped rows in the full loop (0 · ∞):
        // solve on every row instead.
        h.copy_from_slice(&b);
        let all = Live::all(d);
        lod(d, h, kd, dt, &all);
        return cg(d, &b, h, kd, dt, &all);
    }
    its
}

/// The strips of [`STRIP_ROWS`] rows a step visits: every one, or those that hold a row with
/// something to diffuse (the module's "Rows with nothing to diffuse").
struct Live {
    /// Per strip, whether it is visited; `None`: every strip is.
    strips: Option<Vec<bool>>,
    /// The runs of visited rows, `[start, end)`, ascending and apart.
    runs: Vec<(usize, usize)>,
}

impl Live {
    /// Every row.
    fn all(d: &Dims) -> Live {
        Live {
            strips: None,
            runs: vec![(0, d.h)],
        }
    }

    /// The strips a step on `h` with `kd` over `dt` has to visit.
    fn of(d: &Dims, h: &[f64], kd: &[f64], dt: f64) -> Live {
        let (w, hh) = (d.w, d.h);
        let s2 = d.dx * d.dx;
        if w == 0 || hh == 0 || !(dt.is_finite() && dt > 0.0 && s2.is_finite() && s2 > 0.0) {
            return Live::all(d);
        }
        // Rows whose diffusivity is +0 throughout (not −0, NaN or anything else), so every face
        // between two of their cells is 0.5 · (+0 + +0) · Δt / Δx² = +0.
        let zero: Vec<bool> = kd
            .par_chunks(w)
            .map(|row| row.iter().all(|k| k.to_bits() == 0))
            .collect();
        if !zero.iter().any(|&z| z) {
            return Live::all(d);
        }
        // A row is skipped when it and its neighbours (where it has them) are zero rows.
        let skip = |j: usize| zero[j] && (j == 0 || zero[j - 1]) && (j + 1 == hh || zero[j + 1]);
        let strips: Vec<bool> = (0..hh.div_ceil(STRIP_ROWS))
            .map(|s| (s * STRIP_ROWS..((s + 1) * STRIP_ROWS).min(hh)).any(|j| !skip(j)))
            .collect();
        if strips.iter().all(|&v| v) {
            return Live::all(d);
        }
        // The skipped rows and the first and last rows of a run beside them are zero rows; their
        // elevations must be normal, and so must half of them (the cyclic sweep along x halves
        // the first cell of a row and doubles it back, exactly only then).
        let normal = h
            .par_chunks(w)
            .zip(zero.par_iter())
            .all(|(row, &z)| !z || row.iter().all(|v| (0.5 * v).is_normal()));
        if !normal {
            return Live::all(d);
        }
        let mut runs: Vec<(usize, usize)> = Vec::new();
        for (s, _) in strips.iter().enumerate().filter(|(_, v)| **v) {
            let (a, b) = (s * STRIP_ROWS, ((s + 1) * STRIP_ROWS).min(hh));
            match runs.last_mut() {
                Some(last) if last.1 == a => last.1 = b,
                _ => runs.push((a, b)),
            }
        }
        Live {
            strips: Some(strips),
            runs,
        }
    }

    /// Whether strip `s` is visited.
    #[inline]
    fn strip(&self, s: usize) -> bool {
        self.strips.as_ref().is_none_or(|v| v[s])
    }

    /// Whether row `j` is visited.
    #[inline]
    fn row(&self, j: usize) -> bool {
        self.strip(j / STRIP_ROWS)
    }

    /// Whether every visited cell of `x` is finite.
    fn finite(&self, d: &Dims, x: &[f64]) -> bool {
        x.par_chunks(d.w)
            .enumerate()
            .all(|(j, row)| !self.row(j) || row.iter().all(|v| v.is_finite()))
    }
}

/// The locally one-dimensional splitting (the initial guess).
fn lod(d: &Dims, h: &mut [f64], kd: &[f64], dt: f64, live: &Live) {
    let (w, hh) = (d.w, d.h);
    let s2 = d.dx * d.dx;
    // x sweep, row by row (on a skipped row it is the identity).
    h.par_chunks_mut(w)
        .zip(kd.par_chunks(w))
        .enumerate()
        .for_each(|(j, (row, krow))| {
            if !live.row(j) {
                return;
            }
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
    // back in parallel. Each column is solved as ever, one run of visited rows at a time (the
    // whole column when every row is visited): the face between a run and the skipped row beside
    // it is +0, so the column's own elimination carries only signed zeros across it (`Live`).
    let mut slot = vec![usize::MAX; hh];
    let mut visited = 0;
    for &(a, e) in &live.runs {
        for s in &mut slot[a..e] {
            *s = visited;
            visited += 1;
        }
    }
    let blocks: Vec<Vec<f64>> = (0..w.div_ceil(COL_BLOCK))
        .into_par_iter()
        .map(|blk| {
            let i0 = blk * COL_BLOCK;
            let nb = COL_BLOCK.min(w - i0);
            let mut out = vec![0.0; visited * nb];
            let mut col = vec![0.0; hh];
            let mut lo = vec![0.0; hh];
            let mut di = vec![0.0; hh];
            let mut up = vec![0.0; hh];
            for b in 0..nb {
                let i = i0 + b;
                let face = |j: usize| -> f64 {
                    if j + 1 < hh {
                        0.5 * (kd[j * w + i] + kd[(j + 1) * w + i]) * dt / s2
                    } else {
                        0.0
                    }
                };
                for &(a, e) in &live.runs {
                    let len = e - a;
                    for (jj, c) in col[..len].iter_mut().enumerate() {
                        *c = h[(a + jj) * w + i];
                    }
                    for jj in 0..len {
                        let j = a + jj;
                        let north = if j == 0 { 0.0 } else { face(j - 1) };
                        let south = face(j);
                        lo[jj] = -north;
                        up[jj] = -south;
                        di[jj] = 1.0 + north + south;
                    }
                    thomas(&lo[..len], &di[..len], &up[..len], &mut col[..len]);
                    let k0 = slot[a];
                    for (jj, &v) in col[..len].iter().enumerate() {
                        out[(k0 + jj) * nb + b] = v;
                    }
                }
            }
            out
        })
        .collect();
    h.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
        let k = slot[j];
        if k == usize::MAX {
            return;
        }
        for (blk, out) in blocks.iter().enumerate() {
            let i0 = blk * COL_BLOCK;
            let nb = COL_BLOCK.min(w - i0);
            row[i0..i0 + nb].copy_from_slice(&out[k * nb..(k + 1) * nb]);
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
    /// north)); rows are independent, so they run in parallel. A skipped row keeps the +0 faces
    /// and the diagonal of one that those expressions give it (`Live`).
    fn new(d: &Dims, kd: &[f64], dt: f64, live: &Live) -> Operator {
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
                if !live.row(j) {
                    return;
                }
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
            if !live.row(j) {
                return;
            }
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

    /// y = A·x on the visited rows, rows in parallel.
    fn apply(&self, x: &[f64], y: &mut [f64], live: &Live) {
        let (w, h) = (self.w, self.h);
        y.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
            if !live.row(j) {
                return;
            }
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

/// Per row k of the strip `r` (rows of `w` cells) with diagonal `g`: `out[k]` = [Σ r·r, Σ r·z]
/// with the preconditioned residual z = r/g formed cell by cell as it is added (the division and
/// the product the textbook loop takes, without storing z). Each sum is taken in ascending column
/// order from [`sum_zero`], as a serial sum over the row would; two rows run side by side, so the
/// CPU overlaps four chains of additions, which changes the schedule and not the order within any
/// sum.
fn row_sums(w: usize, r: &[f64], g: &[f64], out: &mut [[f64; 2]]) {
    let rows = out.len();
    debug_assert!(r.len() == rows * w && g.len() == rows * w);
    let mut k = 0;
    while k + 2 <= rows {
        let base = k * w;
        let (r0, r1) = (&r[base..base + w], &r[base + w..base + 2 * w]);
        let (g0, g1) = (&g[base..base + w], &g[base + w..base + 2 * w]);
        let (mut a0, mut b0, mut a1, mut b1) = (sum_zero(), sum_zero(), sum_zero(), sum_zero());
        for i in 0..w {
            let (x0, x1) = (r0[i], r1[i]);
            a0 += x0 * x0;
            b0 += x0 * (x0 / g0[i]);
            a1 += x1 * x1;
            b1 += x1 * (x1 / g1[i]);
        }
        out[k] = [a0, b0];
        out[k + 1] = [a1, b1];
        k += 2;
    }
    if k < rows {
        let base = k * w;
        let (mut a, mut b) = (sum_zero(), sum_zero());
        for (&x, &gv) in r[base..base + w].iter().zip(&g[base..base + w]) {
            a += x * x;
            b += x * (x / gv);
        }
        out[k] = [a, b];
    }
}

/// Setup pass: r = b − A·x and per row the sums `sums[j] = [Σ r·r, Σ r·z]` (z = r/diag, the
/// preconditioned residual). Every pass leaves a skipped strip as it is (+0, `Live`).
fn pass_init(
    op: &Operator,
    b: &[f64],
    x: &[f64],
    r: &mut [f64],
    sums: &mut [[f64; 2]],
    live: &Live,
) {
    let (w, h) = (op.w, op.h);
    r.par_chunks_mut(STRIP_ROWS * w)
        .zip(sums.par_chunks_mut(STRIP_ROWS))
        .enumerate()
        .for_each(|(s, (rs, ss))| {
            if !live.strip(s) {
                return;
            }
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
            }
            let c0 = j0 * w;
            row_sums(w, rs, &op.diag[c0..c0 + rows * w], ss);
        });
}

/// First pass of an iteration: the search direction p and A·p with p·A·p per row, and the last
/// iteration's x += α·p_old (`pending`), which nothing reads before the loop ends, so it rides
/// on the pass that reads p_old anyway. The direction is z on the first iteration and
/// z + β·p_old afterwards, with z = r/diag formed where it is used; rows j − 1 and j + 1 of it are
/// needed to apply the operator to row j, so each strip forms its own rows into `pnew` (a
/// separate buffer, so no strip reads what another writes) and its neighbours' edge rows into a
/// ring of three rows that stays in cache. It is the same value cell by cell wherever it is
/// formed. `pap_rows[j]` = Σ p·(A·p) of row j.
#[allow(clippy::too_many_arguments)]
fn pass_direction(
    op: &Operator,
    r: &[f64],
    pold: &[f64],
    beta: Option<f64>,
    pending: Option<f64>,
    x: &mut [f64],
    pnew: &mut [f64],
    ap: &mut [f64],
    pap_rows: &mut [f64],
    live: &Live,
) {
    let (w, h) = (op.w, op.h);
    pnew.par_chunks_mut(STRIP_ROWS * w)
        .zip(ap.par_chunks_mut(STRIP_ROWS * w))
        .zip(x.par_chunks_mut(STRIP_ROWS * w))
        .zip(pap_rows.par_chunks_mut(STRIP_ROWS))
        .enumerate()
        .for_each_init(
            || [vec![0.0; w], vec![0.0; w], vec![0.0; w]],
            |ring, (s, (((pn, aps), xs), pr))| {
                if !live.strip(s) {
                    return;
                }
                let rows = pr.len();
                let j0 = s * STRIP_ROWS;
                if let Some(alpha) = pending {
                    let c0 = j0 * w;
                    for (xv, &pv) in xs.iter_mut().zip(&pold[c0..c0 + rows * w]) {
                        *xv += alpha * pv;
                    }
                }
                let fill = |j: usize, dst: &mut [f64]| {
                    let rrow = &r[j * w..(j + 1) * w];
                    let grow = &op.diag[j * w..(j + 1) * w];
                    match beta {
                        Some(beta) => {
                            let prow = &pold[j * w..(j + 1) * w];
                            for (((d, &rv), &gv), &pv) in
                                dst.iter_mut().zip(rrow).zip(grow).zip(prow)
                            {
                                *d = rv / gv + beta * pv;
                            }
                        }
                        None => {
                            for ((d, &rv), &gv) in dst.iter_mut().zip(rrow).zip(grow) {
                                *d = rv / gv;
                            }
                        }
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

/// Second pass of an iteration: r −= α·A·p and per row the sums `sums[j] = [Σ r·r, Σ r·z]` of
/// the new r (x += α·p waits for the next direction pass, or the end of the loop).
fn pass_update(
    op: &Operator,
    alpha: f64,
    ap: &[f64],
    r: &mut [f64],
    sums: &mut [[f64; 2]],
    live: &Live,
) {
    let w = op.w;
    r.par_chunks_mut(STRIP_ROWS * w)
        .zip(sums.par_chunks_mut(STRIP_ROWS))
        .enumerate()
        .for_each(|(s, (rs, ss))| {
            if !live.strip(s) {
                return;
            }
            let rows = ss.len();
            let c0 = s * STRIP_ROWS * w;
            let len = rows * w;
            for (rv, &av) in rs.iter_mut().zip(&ap[c0..c0 + len]) {
                *rv -= alpha * av;
            }
            row_sums(w, rs, &op.diag[c0..c0 + len], ss);
        });
}

/// x += α·p on the visited strips (the last iteration's update, which no direction pass took).
fn pass_x(w: usize, alpha: f64, p: &[f64], x: &mut [f64], live: &Live) {
    x.par_chunks_mut(STRIP_ROWS * w)
        .zip(p.par_chunks(STRIP_ROWS * w))
        .enumerate()
        .for_each(|(s, (xs, ps))| {
            if live.strip(s) {
                for (xv, &pv) in xs.iter_mut().zip(ps) {
                    *xv += alpha * pv;
                }
            }
        });
}

/// Refines `x` to the solution of A·x = b by Jacobi-preconditioned conjugate gradients, then puts
/// the residual's sum back so Σx = Σb; returns the iterations.
///
/// Every iteration is two passes over the grid, each a parallel region of strips of rows: one forms
/// the direction and A·p, reduces p·A·p and takes the last iteration's x += α·p, the other
/// updates r and reduces r·r and r·z (the textbook loop takes seven regions and more than twice
/// the memory traffic). The arithmetic is the textbook loop's, term for term. **Reductions:** a
/// sum is a sum per row in ascending column order, then those row sums added in ascending row
/// order, on one thread (never a tree over threads, never a function of the strip size or the
/// pool), so the result is the same on any number of threads (`parity.rs` compares bit patterns
/// against the textbook loop).
fn cg(d: &Dims, b: &[f64], x: &mut [f64], kd: &[f64], dt: f64, live: &Live) -> usize {
    let n = d.cells();
    let (w, h) = (d.w, d.h);
    let op = Operator::new(d, kd, dt, live);
    let mut ap = vec![0.0; n];
    let mut r = vec![0.0; n];
    let mut sums = vec![[0.0; 2]; h];
    pass_init(&op, b, x, &mut r, &mut sums, live);
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
        // The step x += α·p of the last update, not yet taken.
        let mut pending = None;
        while iterations < CG_MAX_ITER {
            iterations += 1;
            pass_direction(
                &op,
                &r,
                &pold,
                beta,
                pending.take(),
                x,
                &mut pnew,
                &mut ap,
                &mut pap_rows,
                live,
            );
            std::mem::swap(&mut pold, &mut pnew);
            let pap: f64 = pap_rows.iter().sum();
            if pap <= 0.0 {
                break;
            }
            let alpha = rz / pap;
            pass_update(&op, alpha, &ap, &mut r, &mut sums, live);
            pending = Some(alpha);
            let rr: f64 = sums.iter().map(|s| s[0]).sum();
            if rr <= tol2 {
                break;
            }
            let rz_new: f64 = sums.iter().map(|s| s[1]).sum();
            beta = Some(rz_new / rz);
            rz = rz_new;
        }
        if let Some(alpha) = pending {
            pass_x(w, alpha, &pold, x, live);
        }
    }
    // Σx = Σb − Σr (the columns of A sum to one): return the residual's sum to the cells that
    // take part in the diffusion, in proportion to their coupling. Both sums run over the cells
    // in order, as `.sum()` does; each cell of a skipped row adds +0 to them (b − A·b and
    // 1 − 1), which changes nothing but a −0 into +0, so the row adds one +0.
    op.apply(x, &mut ap, live);
    let (mut lost, mut weight) = (sum_zero(), sum_zero());
    for j in 0..h {
        let cells = j * w..(j + 1) * w;
        if live.row(j) {
            for (p, q) in b[cells.clone()].iter().zip(&ap[cells.clone()]) {
                lost += p - q;
            }
            for g in &op.diag[cells] {
                weight += g - 1.0;
            }
        } else {
            lost += 0.0;
            weight += 0.0;
        }
    }
    if weight > 0.0 && lost != 0.0 {
        let k = lost / weight;
        x.par_chunks_mut(w)
            .zip(op.diag.par_chunks(w))
            .enumerate()
            .for_each(|(j, (xr, gr))| {
                if live.row(j) {
                    for (xv, g) in xr.iter_mut().zip(gr) {
                        *xv += k * (g - 1.0);
                    }
                }
            });
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

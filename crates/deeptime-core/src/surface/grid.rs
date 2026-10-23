// SPDX-License-Identifier: LGPL-3.0-or-later
//! The raster the surface kernels run on (plan §5.2): W × H square cells of side `dx` metres in
//! **block metric** (every L1 cell is r × 300 m wide in x and z whatever its latitude), wrapping in
//! x and closed (no flow across) at the first and last row. The planet's L1 grid is one such
//! raster; the oracle and the unit tests use small ones.
//!
//! Neighbours are the eight of D8, in a fixed order (E, NE, N, NW, W, SW, S, SE — "north" is row
//! j − 1), so every kernel visits them identically and ties break the same way on every run.

/// W × H cells of `dx` metres, wrapping in x when `wrap_x`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dims {
    pub w: usize,
    pub h: usize,
    /// Cell side, metres (block metric).
    pub dx: f64,
    pub wrap_x: bool,
}

/// Column and row offsets of the eight neighbours, and whether each is diagonal.
pub const OFFSETS: [(i64, i64, bool); 8] = [
    (1, 0, false),
    (1, -1, true),
    (0, -1, false),
    (-1, -1, true),
    (-1, 0, false),
    (-1, 1, true),
    (0, 1, false),
    (1, 1, true),
];

impl Dims {
    pub fn new(w: usize, h: usize, dx: f64, wrap_x: bool) -> Dims {
        Dims { w, h, dx, wrap_x }
    }

    pub fn cells(&self) -> usize {
        self.w * self.h
    }

    /// Area of one cell, m² (block metric).
    pub fn cell_area(&self) -> f64 {
        self.dx * self.dx
    }

    /// The neighbour of cell `c` at offset `k` of [`OFFSETS`], if it exists (x wraps, rows clamp;
    /// on a grid one cell wide the wrapped neighbour would be the cell itself and is skipped).
    #[inline]
    pub fn neighbour(&self, c: usize, k: usize) -> Option<usize> {
        // The raster's topology is the projection's neighbour rule.
        let (di, dj, _) = OFFSETS[k];
        let (ni, nj) = deeptime_format::projection::neighbour(
            self.w,
            self.h,
            self.wrap_x,
            (c % self.w, c / self.w),
            (di, dj),
        )?;
        let n = nj * self.w + ni;
        (n != c).then_some(n)
    }

    /// Distance to the neighbour at offset `k`, metres.
    #[inline]
    pub fn step_len(&self, k: usize) -> f64 {
        if OFFSETS[k].2 {
            self.dx * std::f64::consts::SQRT_2
        } else {
            self.dx
        }
    }

    /// The eight neighbours of `c` with their distances: [`Dims::neighbour`] and
    /// [`Dims::step_len`] for k = 0..8, with the cell's column and row found once instead of
    /// once per neighbour (P1-17: the two integer divisions per neighbour were most of the cost
    /// of the kernels that walk them).
    #[inline]
    pub fn neighbours(&self, c: usize) -> impl Iterator<Item = (usize, f64)> + '_ {
        let ij = (c % self.w, c / self.w);
        (0..8).filter_map(move |k| {
            let (di, dj, _) = OFFSETS[k];
            let (ni, nj) =
                deeptime_format::projection::neighbour(self.w, self.h, self.wrap_x, ij, (di, dj))?;
            let n = nj * self.w + ni;
            (n != c).then(|| (n, self.step_len(k)))
        })
    }

    /// Distance between two neighbouring cells, metres (dx or dx·√2).
    #[inline]
    pub fn dist(&self, a: usize, b: usize) -> f64 {
        if a % self.w == b % self.w || a / self.w == b / self.w {
            self.dx
        } else {
            self.dx * std::f64::consts::SQRT_2
        }
    }
}

/// What the surface's graph solvers need of a grid (cube world plan §3 S2): its cells, each cell's
/// D8 neighbours with their distances in the fixed order of [`OFFSETS`] (a missing neighbour is
/// skipped), each cell's area, and how to cut the cells into strips for the parallel flood.
/// [`Dims`] (the x-wrapped raster, block metric) and `sphere::cube::CubeGrid` (the cube, true
/// metric) implement it. On `Dims` every method is the raster's own arithmetic, so a solver
/// written on this trait gives the raster's results bit for bit.
pub trait SurfaceGrid: Sync {
    /// Number of cells.
    fn cells(&self) -> usize;

    /// The neighbours of `c` with their distances, metres, in [`OFFSETS`] order.
    fn neighbours(&self, c: usize) -> impl Iterator<Item = (usize, f64)> + '_;

    /// Area of cell `c`, m².
    fn area(&self, c: usize) -> f64;

    /// The area a lake's room is counted in (`spl.rs`: metres times cells of this area), m²: the
    /// raster's one cell area, the cube's mean.
    fn ref_area(&self) -> f64;

    /// `Some(a)` when every cell has area `a` (the raster), so a sum of areas keeps its one
    /// product (`marine::fill`).
    fn uniform_area(&self) -> Option<f64>;

    /// The parallel flood's strips: contiguous cell ranges, ascending, covering every cell, about
    /// `rows` grid rows each. A grain of work only: the flood's result does not depend on it.
    fn strips(&self, rows: usize) -> Vec<std::ops::Range<usize>>;

    /// The cells of the strip `s` (one of [`SurfaceGrid::strips`]) that may have a neighbour
    /// outside it, ascending (a superset is allowed), into `out`.
    fn strip_border(&self, s: std::ops::Range<usize>, out: &mut Vec<usize>);

    /// The neighbours of `c` (the cells of [`SurfaceGrid::neighbours`], in its order) whose index
    /// lies in `[lo, hi)`, into `out`; returns how many.
    #[inline]
    fn neighbours_in(&self, c: usize, lo: usize, hi: usize, out: &mut [usize; 8]) -> usize {
        let mut k = 0;
        for (m, _) in self.neighbours(c) {
            if m >= lo && m < hi {
                out[k] = m;
                k += 1;
            }
        }
        k
    }
}

impl SurfaceGrid for Dims {
    #[inline]
    fn cells(&self) -> usize {
        Dims::cells(self)
    }

    #[inline]
    fn neighbours(&self, c: usize) -> impl Iterator<Item = (usize, f64)> + '_ {
        Dims::neighbours(self, c)
    }

    #[inline]
    fn area(&self, _c: usize) -> f64 {
        self.cell_area()
    }

    #[inline]
    fn ref_area(&self) -> f64 {
        self.cell_area()
    }

    #[inline]
    fn uniform_area(&self) -> Option<f64> {
        Some(self.cell_area())
    }

    fn strips(&self, rows: usize) -> Vec<std::ops::Range<usize>> {
        let rows = rows.max(1);
        (0..self.h)
            .step_by(rows)
            .map(|j0| j0 * self.w..(j0 + rows).min(self.h) * self.w)
            .collect()
    }

    fn strip_border(&self, s: std::ops::Range<usize>, out: &mut Vec<usize>) {
        // The first and the last row of the strip: the others reach only rows inside it.
        let w = self.w.max(1);
        let (j0, j1) = (s.start / w, s.end / w);
        out.clear();
        out.extend(j0 * w..(j0 + 1).min(j1) * w);
        if j1 > j0 + 1 {
            out.extend((j1 - 1) * w..j1 * w);
        }
    }

    /// The row-based form of the default: the strip `[lo, hi)` is whole rows.
    #[inline]
    fn neighbours_in(&self, c: usize, lo: usize, hi: usize, out: &mut [usize; 8]) -> usize {
        let w = self.w as i64;
        let (r0, r1) = ((lo / self.w) as i64, (hi / self.w) as i64);
        let (i, j) = ((c % self.w) as i64, (c / self.w) as i64);
        let mut k = 0;
        for &(di, dj, _) in &OFFSETS {
            let nj = j + dj;
            if nj < r0 || nj >= r1 {
                continue;
            }
            let mut ni = i + di;
            if ni < 0 || ni >= w {
                if !self.wrap_x {
                    continue;
                }
                ni = ni.rem_euclid(w);
            }
            let m = (nj * w + ni) as usize;
            if m != c {
                out[k] = m;
                k += 1;
            }
        }
        k
    }
}

/// Box blur of `v` (in place) with half-width `r` cells, repeated `passes` times (three passes
/// approximate a Gaussian of σ ≈ r). x wraps when the grid does; rows clamp. Mass is kept on a
/// wrapping row, and the edge rows reflect.
pub fn box_blur(d: &Dims, v: &mut [f64], r: usize, passes: usize) {
    use rayon::prelude::*;
    if r == 0 {
        return;
    }
    let (w, h) = (d.w, d.h);
    let norm = 1.0 / (2 * r + 1) as f64;
    for _ in 0..passes {
        v.par_chunks_mut(w).for_each(|row| {
            let src = row.to_vec();
            let at = |i: i64| -> f64 {
                if d.wrap_x {
                    src[i.rem_euclid(w as i64) as usize]
                } else {
                    src[i.clamp(0, w as i64 - 1) as usize]
                }
            };
            let mut acc: f64 = (-(r as i64)..=r as i64).map(at).sum();
            for (i, x) in row.iter_mut().enumerate() {
                *x = acc * norm;
                acc += at(i as i64 + r as i64 + 1) - at(i as i64 - r as i64);
            }
        });
        let cols: Vec<Vec<f64>> = (0..w)
            .into_par_iter()
            .map(|i| {
                let src: Vec<f64> = (0..h).map(|j| v[j * w + i]).collect();
                let at = |j: i64| -> f64 {
                    let j = if j < 0 {
                        -j - 1
                    } else if j >= h as i64 {
                        2 * h as i64 - j - 1
                    } else {
                        j
                    };
                    src[j.clamp(0, h as i64 - 1) as usize]
                };
                let mut out = vec![0.0; h];
                let mut acc: f64 = (-(r as i64)..=r as i64).map(at).sum();
                for (j, o) in out.iter_mut().enumerate() {
                    *o = acc * norm;
                    acc += at(j as i64 + r as i64 + 1) - at(j as i64 - r as i64);
                }
                out
            })
            .collect();
        // Back into the rows, rows in parallel (P1-17: the strided serial copy was a tenth of the
        // blur's time on 8 threads).
        v.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
            for (x, col) in row.iter_mut().zip(&cols) {
                *x = col[j];
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// P1-17: the box blur gives the bits of the blur that copied its columns back serially.
    #[test]
    fn the_box_blur_equals_the_serial_copy_back() {
        fn reference(d: &Dims, v: &mut [f64], r: usize, passes: usize) {
            let (w, h) = (d.w, d.h);
            let norm = 1.0 / (2 * r + 1) as f64;
            for _ in 0..passes {
                for row in v.chunks_mut(w) {
                    let src = row.to_vec();
                    let at = |i: i64| -> f64 {
                        if d.wrap_x {
                            src[i.rem_euclid(w as i64) as usize]
                        } else {
                            src[i.clamp(0, w as i64 - 1) as usize]
                        }
                    };
                    let mut acc: f64 = (-(r as i64)..=r as i64).map(at).sum();
                    for (i, x) in row.iter_mut().enumerate() {
                        *x = acc * norm;
                        acc += at(i as i64 + r as i64 + 1) - at(i as i64 - r as i64);
                    }
                }
                let cols: Vec<Vec<f64>> = (0..w)
                    .map(|i| {
                        let src: Vec<f64> = (0..h).map(|j| v[j * w + i]).collect();
                        let at = |j: i64| -> f64 {
                            let j = if j < 0 {
                                -j - 1
                            } else if j >= h as i64 {
                                2 * h as i64 - j - 1
                            } else {
                                j
                            };
                            src[j.clamp(0, h as i64 - 1) as usize]
                        };
                        let mut out = vec![0.0; h];
                        let mut acc: f64 = (-(r as i64)..=r as i64).map(at).sum();
                        for (j, o) in out.iter_mut().enumerate() {
                            *o = acc * norm;
                            acc += at(j as i64 + r as i64 + 1) - at(j as i64 - r as i64);
                        }
                        out
                    })
                    .collect();
                for (i, col) in cols.iter().enumerate() {
                    for (j, &x) in col.iter().enumerate() {
                        v[j * w + i] = x;
                    }
                }
            }
        }
        let mut rng = crate::sphere::SplitMix64::new(5);
        for &(w, h, wrap, r) in &[
            (50usize, 30usize, true, 3usize),
            (7, 40, false, 5),
            (1, 5, true, 2),
        ] {
            let d = Dims::new(w, h, 1000.0, wrap);
            let v0: Vec<f64> = (0..w * h).map(|_| rng.next_f64() * 100.0 - 20.0).collect();
            let (mut a, mut b) = (v0.clone(), v0);
            box_blur(&d, &mut a, r, 2);
            reference(&d, &mut b, r, 2);
            let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
            assert_eq!(bits(&a), bits(&b), "{w}x{h}");
        }
    }

    #[test]
    fn neighbours_wrap_in_x_and_stop_at_the_rows() {
        let d = Dims::new(4, 3, 10.0, true);
        let n: Vec<usize> = d.neighbours(0).map(|(n, _)| n).collect();
        assert_eq!(n, vec![1, 3, 7, 4, 5]);
        let top_right: Vec<usize> = d.neighbours(3).map(|(n, _)| n).collect();
        assert_eq!(top_right, vec![0, 2, 6, 7, 4]);
        assert!((d.dist(0, 5) - 10.0 * std::f64::consts::SQRT_2).abs() < 1e-12);
        let closed = Dims::new(4, 3, 10.0, false);
        assert_eq!(closed.neighbours(0).count(), 3);
    }

    /// `neighbours` is `neighbour` and `step_len` for k = 0..8, cell by cell, on grids that wrap
    /// and do not, including the one- and two-cell-wide and one-row ones.
    #[test]
    fn neighbours_match_the_one_at_a_time_rule() {
        for &(w, h) in &[(1, 1), (1, 5), (2, 5), (5, 1), (3, 3), (17, 9), (64, 2)] {
            for wrap in [true, false] {
                let d = Dims::new(w, h, 7.5, wrap);
                for c in 0..d.cells() {
                    let fast: Vec<(usize, u64)> =
                        d.neighbours(c).map(|(n, l)| (n, l.to_bits())).collect();
                    let slow: Vec<(usize, u64)> = (0..8)
                        .filter_map(|k| d.neighbour(c, k).map(|n| (n, d.step_len(k).to_bits())))
                        .collect();
                    assert_eq!(fast, slow, "{w}x{h} wrap {wrap} cell {c}");
                }
            }
        }
    }

    #[test]
    fn blur_keeps_the_mean_on_a_wrapping_band() {
        let d = Dims::new(32, 16, 1.0, true);
        let mut v: Vec<f64> = (0..d.cells()).map(|c| ((c * 7919) % 13) as f64).collect();
        let before: f64 = v.iter().sum();
        box_blur(&d, &mut v, 3, 3);
        let after: f64 = v.iter().sum();
        assert!((before - after).abs() < 1e-9 * before, "{before} {after}");
    }
}

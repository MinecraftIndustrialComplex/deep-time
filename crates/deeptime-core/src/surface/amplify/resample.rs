// SPDX-License-Identifier: LGPL-3.0-or-later
//! One level of the amplifier and its ×2 upsampling (P1-06).
//!
//! A [`Level`] is the arrays of a `w × h` planet grid at one cell size: the terrain, the effective
//! drainage area the level below computed exactly, the runoff weight and rock erodibility the
//! erosion passes use, and the footprint of the lakes the young landscape already had. Fine cell
//! (I, J) of the ×2 grid sits on the coarse point (I/2, J/2): even cells coincide with a coarse
//! cell and odd ones lie half-way (a grid cell is a point sample, `GridDesc`), so every fine
//! value is a function of the coarse array and the cell's own global coordinates. That is what
//! makes a tile's upsampling equal to the whole raster's: a tile samples exactly the cells its
//! window covers.
//!
//! - **Terrain**: Catmull–Rom half-way values (−1, 9, 9, −1)/16 clamped between the two nearest
//!   samples, separably. The clamp is what keeps a coast (a 500 m step from a shelf to a beach)
//!   from growing a raised rim or a ditch, at the price of flat-topped peaks (the erosion passes
//!   give them back their shape). x wraps, rows clamp.
//! - **Drainage area**: interpolated in log space, minus the duplication of the coarse cell's own
//!   area (a headwater cell of the coarse level is four fine cells that each own a quarter of it):
//!   the erosion passes' prior, which they refine locally and keep beyond their reach.
//! - **Runoff weight and erodibility**: linear.
//! - **Lakes**: each fine cell is a lake cell when any of the fine 3 × 3 around it lies in a coarse
//!   lake cell, so the footprint is inherited and grows by one fine cell.

use super::window::Win;

/// The arrays of one grid of the amplifier's ladder.
#[derive(Clone, Debug)]
pub struct Level {
    pub w: usize,
    pub h: usize,
    /// Cell size, m (block metric).
    pub dx: f64,
    /// Terrain, m.
    pub z: Vec<f32>,
    /// Effective (runoff-weighted) drainage area, m².
    pub area: Vec<f32>,
    /// Runoff weight (1 = the reference runoff).
    pub wgt: Vec<f32>,
    /// Rock erodibility multiplier.
    pub kfac: Vec<f32>,
    /// 1 inside the lakes (and closed basins) the young landscape had, else 0.
    pub lake: Vec<u8>,
    /// The glaciers' surface, m, where they were, and a metre under the ground elsewhere (so it
    /// interpolates smoothly across their margins); empty without glaciers (`glacial.rs`).
    pub zs: Vec<f32>,
}

impl Level {
    /// Bytes the arrays hold.
    pub fn bytes(&self) -> u64 {
        4 * (self.z.len() + self.area.len() + self.wgt.len() + self.kfac.len() + self.zs.len())
            as u64
            + self.lake.len() as u64
    }
}

/// The fine values of a window (see [`Level::sample`]).
#[derive(Clone, Debug, Default)]
pub struct Fine {
    pub z: Vec<f32>,
    pub area: Vec<f32>,
    pub own: Vec<f32>,
    pub kfac: Vec<f32>,
    pub lake: Vec<u8>,
    pub zs: Vec<f32>,
}

/// The Catmull–Rom value half-way between `p1` and `p2`, clamped between them.
#[inline]
fn mid(p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    let v = (9.0 * (p1 + p2) - (p0 + p3)) * 0.0625;
    v.clamp(p1.min(p2), p1.max(p2))
}

/// Index of coarse cell (i, j) of a `w × h` grid: x wraps, rows clamp.
#[inline]
fn idx(w: usize, h: usize, i: i64, j: i64) -> usize {
    i.rem_euclid(w as i64) as usize + j.clamp(0, h as i64 - 1) as usize * w
}

/// Terrain of the fine cell whose coarse cell is (ci, cj), parities (fi, fj).
fn z_at(z: &[f32], w: usize, h: usize, ci: i64, cj: i64, fi: bool, fj: bool) -> f32 {
    let g = |i: i64, j: i64| z[idx(w, h, i, j)];
    match (fi, fj) {
        (false, false) => g(ci, cj),
        (true, false) => mid(g(ci - 1, cj), g(ci, cj), g(ci + 1, cj), g(ci + 2, cj)),
        (false, true) => mid(g(ci, cj - 1), g(ci, cj), g(ci, cj + 1), g(ci, cj + 2)),
        (true, true) => {
            let row = |r: i64| {
                mid(
                    g(ci - 1, cj + r),
                    g(ci, cj + r),
                    g(ci + 1, cj + r),
                    g(ci + 2, cj + r),
                )
            };
            mid(row(-1), row(0), row(1), row(2))
        }
    }
}

/// Average of the coarse samples around the fine cell (1 on a node, 2 half-way, 4 in the middle
/// of a coarse quad) of `f`.
fn lin_at(
    f: impl Fn(usize) -> f64,
    w: usize,
    h: usize,
    ci: i64,
    cj: i64,
    fi: bool,
    fj: bool,
) -> f64 {
    let g = |i: i64, j: i64| f(idx(w, h, i, j));
    match (fi, fj) {
        (false, false) => g(ci, cj),
        (true, false) => 0.5 * (g(ci, cj) + g(ci + 1, cj)),
        (false, true) => 0.5 * (g(ci, cj) + g(ci, cj + 1)),
        (true, true) => 0.25 * (g(ci, cj) + g(ci + 1, cj) + g(ci, cj + 1) + g(ci + 1, cj + 1)),
    }
}

/// The ×2 terrain of a plain `w × h` array (clamped Catmull–Rom; x wraps, rows clamp).
pub fn upsample2_z(z: &[f32], w: usize, h: usize) -> Vec<f32> {
    use rayon::prelude::*;
    let mut out = vec![0.0f32; 4 * w * h];
    out.par_chunks_mut(2 * w).enumerate().for_each(|(j, row)| {
        for (i, o) in row.iter_mut().enumerate() {
            *o = z_at(
                z,
                w,
                h,
                (i >> 1) as i64,
                (j >> 1) as i64,
                i & 1 == 1,
                j & 1 == 1,
            );
        }
    });
    out
}

/// The ×2 values of a plain array by bilinear interpolation.
pub fn upsample2_linear(v: &[f32], w: usize, h: usize) -> Vec<f32> {
    use rayon::prelude::*;
    let mut out = vec![0.0f32; 4 * w * h];
    out.par_chunks_mut(2 * w).enumerate().for_each(|(j, row)| {
        for (i, o) in row.iter_mut().enumerate() {
            *o = lin_at(
                |c| f64::from(v[c]),
                w,
                h,
                (i >> 1) as i64,
                (j >> 1) as i64,
                i & 1 == 1,
                j & 1 == 1,
            ) as f32;
        }
    });
    out
}

/// The ×2 values of a plain array by replicating each cell 2 × 2.
pub fn upsample2_nearest<T: Copy + Send + Sync>(v: &[T], w: usize, h: usize) -> Vec<T> {
    use rayon::prelude::*;
    (0..2 * h)
        .into_par_iter()
        .flat_map_iter(|j| (0..2 * w).map(move |i| v[(j >> 1) * w + (i >> 1)]))
        .collect()
}

impl Level {
    pub fn cells(&self) -> usize {
        self.w * self.h
    }

    /// The fine grid's cell size, m.
    pub fn fine_dx(&self) -> f64 {
        self.dx / 2.0
    }

    /// The ×2 values of the fine cells a window covers (`win.gw == 2 · self.w`,
    /// `win.gh == 2 · self.h`). Cells beyond the window's rows are never asked for.
    pub fn sample(&self, win: &Win) -> Fine {
        debug_assert_eq!(win.gw, 2 * self.w);
        debug_assert_eq!(win.gh, 2 * self.h);
        let n = win.cells();
        let mut f = Fine {
            z: vec![0.0; n],
            area: vec![0.0; n],
            own: vec![0.0; n],
            kfac: vec![0.0; n],
            lake: vec![0; n],
            zs: Vec::new(),
        };
        let dxf = self.fine_dx();
        let a_fine = (dxf * dxf) as f32;
        let a_coarse = (self.dx * self.dx) as f32;
        let ln_area = |c: usize| f64::from(self.area[c].max(1.0)).ln();
        let wgt = |c: usize| f64::from(self.wgt[c]);
        let kfac = |c: usize| f64::from(self.kfac[c]);
        use rayon::prelude::*;
        let w = win.w;
        f.z.par_chunks_mut(w)
            .zip(f.area.par_chunks_mut(w))
            .zip(f.own.par_chunks_mut(w))
            .zip(f.kfac.par_chunks_mut(w))
            .zip(f.lake.par_chunks_mut(w))
            .enumerate()
            .for_each(|(j, ((((zr, ar), or), kr), lr))| {
                for i in 0..w {
                    let (gi, gj) = win.global(i, j);
                    let (gi, gj) = (gi as i64, gj as i64);
                    let (ci, cj) = (gi >> 1, gj >> 1);
                    let (fi, fj) = (gi & 1 == 1, gj & 1 == 1);
                    zr[i] = z_at(&self.z, self.w, self.h, ci, cj, fi, fj);
                    let wg = lin_at(wgt, self.w, self.h, ci, cj, fi, fj) as f32;
                    or[i] = wg * a_fine;
                    kr[i] = lin_at(kfac, self.w, self.h, ci, cj, fi, fj) as f32;
                    // Area: log-space interpolation, less the coarse cell's own area shared out.
                    let interp = lin_at(ln_area, self.w, self.h, ci, cj, fi, fj).exp() as f32;
                    ar[i] = (interp - wg * (a_coarse - a_fine)).max(or[i]);
                    let mut lake = 0u8;
                    'scan: for b in -1..=1i64 {
                        for a in -1..=1i64 {
                            let c = idx(self.w, self.h, (gi + a) >> 1, (gj + b) >> 1);
                            if self.lake[c] != 0 {
                                lake = 1;
                                break 'scan;
                            }
                        }
                    }
                    lr[i] = lake;
                }
            });
        if self.zs.len() == self.cells() {
            // Clamped Catmull-Rom, like the terrain: a glacier's surface is smooth.
            f.zs = vec![0.0; n];
            f.zs.par_chunks_mut(w).enumerate().for_each(|(j, zr)| {
                for (i, o) in zr.iter_mut().enumerate() {
                    let (gi, gj) = win.global(i, j);
                    let (gi, gj) = (gi as i64, gj as i64);
                    *o = z_at(
                        &self.zs,
                        self.w,
                        self.h,
                        gi >> 1,
                        gj >> 1,
                        gi & 1 == 1,
                        gj & 1 == 1,
                    );
                }
            });
        }
        f
    }

    /// The whole ×2 level's terrain (the fine grid's `z`), for callers that need no other array.
    pub fn upsampled_z(&self) -> Vec<f32> {
        self.sample(&Win::full(2 * self.w, 2 * self.h)).z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level(w: usize, h: usize, f: impl Fn(usize, usize) -> f32) -> Level {
        let n = w * h;
        Level {
            w,
            h,
            dx: 1000.0,
            z: (0..n).map(|c| f(c % w, c / w)).collect(),
            area: vec![1.0e6; n],
            wgt: vec![1.0; n],
            kfac: vec![1.0; n],
            lake: vec![0; n],
            zs: Vec::new(),
        }
    }

    #[test]
    fn even_cells_are_the_coarse_cells_and_the_seam_is_smooth() {
        let l = level(16, 8, |i, j| {
            let x = i as f32 / 16.0 * std::f32::consts::TAU;
            100.0 * x.sin() + 30.0 * j as f32
        });
        let up = l.upsampled_z();
        let fw = 32;
        for j in 0..8 {
            for i in 0..16 {
                assert_eq!(up[2 * j * fw + 2 * i], l.z[j * 16 + i]);
            }
        }
        // Across the seam (fine columns 31 and 0) the values continue the sine like any others.
        let step = |a: usize, b: usize| (up[4 * fw + a] - up[4 * fw + b]).abs();
        assert!(
            step(31, 0) < 25.0 && step(30, 31) < 25.0,
            "{} {}",
            step(31, 0),
            step(30, 31)
        );
        assert!(
            (step(31, 0) - step(15, 16)).abs() < 10.0,
            "{} {}",
            step(31, 0),
            step(15, 16)
        );
    }

    #[test]
    fn a_step_gets_no_overshoot_and_a_ramp_stays_a_ramp() {
        // A step: 500 m of shelf against a 50 m beach; nothing may leave [−500, 50].
        let l = level(16, 8, |i, _| if i < 8 { 50.0 } else { -500.0 });
        let up = l.upsampled_z();
        assert!(up.iter().all(|&z| (-500.0..=50.0).contains(&z)));
        // A plane is reproduced exactly: Catmull–Rom is exact on linear data away from the clamp.
        let l = level(16, 8, |i, j| 10.0 * i as f32 + 3.0 * j as f32);
        let up = l.upsampled_z();
        let fw = 32;
        for j in 2..12 {
            for i in 2..26 {
                let want = 5.0 * i as f32 + 1.5 * j as f32;
                assert!(
                    (up[j * fw + i] - want).abs() < 1e-3,
                    "({i},{j}): {}",
                    up[j * fw + i]
                );
            }
        }
    }

    #[test]
    fn a_window_equals_the_same_cells_of_the_whole() {
        let mut l = level(32, 16, |i, j| ((i * 7 + j * 13) % 29) as f32 * 3.0);
        l.area = (0..32 * 16).map(|c| 1.0e6 * (1 + c % 17) as f32).collect();
        l.lake = (0..32 * 16).map(|c| u8::from(c % 41 == 0)).collect();
        l.wgt = (0..32 * 16).map(|c| 0.5 + (c % 5) as f32 * 0.1).collect();
        let whole = l.sample(&Win::full(64, 32));
        // A window across the seam and touching the north pole.
        let win = Win {
            w: 20,
            h: 9,
            gi0: -7,
            gj0: 0,
            gw: 64,
            gh: 32,
        };
        let part = l.sample(&win);
        for j in 0..9 {
            for i in 0..20 {
                let (gi, gj) = win.global(i, j);
                let g = gj * 64 + gi;
                let c = j * 20 + i;
                assert_eq!(part.z[c], whole.z[g]);
                assert_eq!(part.area[c], whole.area[g]);
                assert_eq!(part.own[c], whole.own[g]);
                assert_eq!(part.lake[c], whole.lake[g]);
            }
        }
    }

    #[test]
    fn area_prior_of_a_headwater_is_its_own_fine_area_and_lakes_grow() {
        let mut l = level(16, 8, |_, _| 0.0);
        // Every coarse cell is a headwater (its area is its own): so is every fine cell.
        l.area = vec![1.0e6; 16 * 8];
        let f = l.sample(&Win::full(32, 16));
        let own = (500.0f64 * 500.0) as f32;
        assert!(
            f.area.iter().all(|&a| (a - own).abs() < 1.0),
            "{}",
            f.area[0]
        );
        // A river cell: its neighbours' log-mean lifts the half-way cells.
        l.area[4 * 16 + 4] = 1.0e9;
        let f = l.sample(&Win::full(32, 16));
        assert!(
            (f.area[8 * 32 + 8] - 1.0e9 + 0.75e6).abs() < 1.0e3,
            "{}",
            f.area[8 * 32 + 8]
        );
        assert!(f.area[8 * 32 + 9] > 1.0e7 && f.area[8 * 32 + 9] < 1.0e9);
        // One coarse lake cell: a 2 × 2 fine block plus one fine cell all round is lake.
        l.lake[4 * 16 + 4] = 1;
        let f = l.sample(&Win::full(32, 16));
        let count: usize = f.lake.iter().map(|&x| usize::from(x)).sum();
        assert_eq!(count, 16, "4 × 4 fine cells");
        assert_eq!(f.lake[8 * 32 + 7], 1);
        assert_eq!(f.lake[8 * 32 + 6], 0);
    }
}

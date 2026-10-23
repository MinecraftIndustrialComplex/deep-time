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
//! - **Lakes**: a lake is its water surface: the level the bake's water balance holds it at on
//!   the first level (`balance.rs`), or with the hydrology off the level the young landscape's
//!   flood filled the closed basin to. A fine cell takes the highest surface among the coarse cells of its own
//!   interpolation quad (the coarse cell it sits on, or the two or four it lies between) and is a
//!   lake cell when its terrain lies under that surface. The shore is therefore the contour of the
//!   fine terrain at the lake's level: smooth, and the same in every direction. A footprint copied
//!   from the coarse cells (each fine cell a lake when a coarse lake cell lay within a cell of it)
//!   drew every lake's edge as a staircase of coarse cells, and the passes, which leave lakes
//!   alone, cut that staircase into the walls of every closed basin (ledger AMP-090).
//! - **Closed basins** ([`basin`]): ground the open sea does not reach is never sea, however far
//!   below sea level it lies, and the lowest point of a basin the water balance leaves dry is
//!   where its floor drains to (`balance.rs`). A fine cell is closed ground when every coarse
//!   cell of its quad is, and a sink when it sits on a coarse sink. A level keeps the class a
//!   byte a cell ([`Level::closed`]); a window keeps it in bits 1 and 2 of its lake byte
//!   ([`cell`]), so the class costs a window nothing (ledger AMP-096).

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
    /// The water surface, m, of the lake at the cell, [`NO_LAKE`] elsewhere: the lakes the water
    /// balance leaves on the young landscape (`balance.rs`), or with the hydrology off the level
    /// the young landscape's flood filled each closed basin to.
    pub lake_z: Vec<f32>,
    /// The glaciers' surface, m, where they were, and a metre under the ground elsewhere (so it
    /// interpolates smoothly across their margins); empty without glaciers (`glacial.rs`).
    pub zs: Vec<f32>,
    /// What the ground of each cell is ([`basin`]); empty when every cell is [`basin::OPEN`].
    pub closed: Vec<u8>,
}

/// [`Level::lake_z`] of a cell outside every lake.
pub const NO_LAKE: f32 = f32::NEG_INFINITY;

/// What a cell of [`Level::closed`] is (and a fine cell, [`cell::class`]).
pub mod basin {
    /// Ground the open sea reaches, or above sea level: under sea level it is sea.
    pub const OPEN: u8 = 0;
    /// The ground of a closed basin (the hydrology's `closed_basin` layer): never sea, however
    /// far below sea level it lies.
    pub const CLOSED: u8 = 1;
    /// The lowest point of a basin the water balance leaves dry: its floor drains here, so the
    /// flood keeps the basin instead of filling it (an outlet, like a lake).
    pub const SINK: u8 = 2;
}

/// A window's byte for a fine cell ([`Fine::lake`], `Buf::lake`): bit 0 is set when the cell lies
/// under a lake's surface, bits 1 and 2 hold its [`basin`] class. The class rides in the lake
/// flag's byte so that a window carries no byte more for it (the default world's amplifier is
/// within 38 MiB of the 4 GiB cap, 2026-10-23).
pub mod cell {
    /// The lake bit.
    pub const LAKE: u8 = 1;

    /// Whether the cell lies under a lake's surface.
    #[inline]
    pub fn lake(m: u8) -> bool {
        m & LAKE != 0
    }

    /// The cell's [`super::basin`] class.
    #[inline]
    pub fn class(m: u8) -> u8 {
        m >> 1
    }

    /// The byte of a cell with lake flag `lake` and basin class `class`.
    #[inline]
    pub fn of(lake: bool, class: u8) -> u8 {
        u8::from(lake) | (class << 1)
    }
}

/// Whether ground at `z` with basin class `closed` ([`basin`]) is under the sea: below sea level
/// and open to it (ledger: AMP-096).
#[inline]
pub fn wet(z: f32, sea: f32, closed: u8) -> bool {
    z <= sea && closed == basin::OPEN
}

impl Level {
    /// Bytes the arrays hold.
    pub fn bytes(&self) -> u64 {
        4 * (self.z.len()
            + self.area.len()
            + self.wgt.len()
            + self.kfac.len()
            + self.lake_z.len()
            + self.zs.len()) as u64
            + self.closed.len() as u64
    }

    /// Whether cell `c` lies in a lake (or closed basin).
    pub fn is_lake(&self, c: usize) -> bool {
        self.lake_z.get(c).is_some_and(|s| s.is_finite())
    }
}

/// The fine values of a window (see [`Level::sample`]).
#[derive(Clone, Debug, Default)]
pub struct Fine {
    pub z: Vec<f32>,
    pub area: Vec<f32>,
    pub own: Vec<f32>,
    pub kfac: Vec<f32>,
    /// The cell's [`cell`] byte: its lake bit, set where it lies under a lake's surface
    /// ([`Level::lake_surface`]), and its basin class ([`Level::basin_class`]). The surface
    /// itself is not kept per fine cell: the next level's [`Level::lake_z`] takes it from the
    /// coarse level when a tile hands its core on (`tiles::LevelSink`), which keeps 4 B a cell out
    /// of every window.
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
                    // The highest lake surface among the corners of the cell's quad (1, 2 or 4
                    // coarse cells); a lake cell when the terrain lies under it. ledger: AMP-090
                    let lake = zr[i] < self.lake_surface(gi, gj);
                    lr[i] = cell::of(lake, self.basin_class(gi, gj));
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

    /// The lake surface over fine cell (gi, gj) of the ×2 grid, m: the highest lake surface among
    /// the coarse cells at the corners of its interpolation quad (the coarse cell it sits on, or
    /// the two or four it lies between), [`NO_LAKE`] when none of them is in a lake. The fine
    /// cell is a lake cell when its terrain lies under it ([`Level::sample`]), and then this is
    /// its [`Level::lake_z`] on the next level (`tiles::LevelSink`). ledger: AMP-090
    #[inline]
    pub fn lake_surface(&self, gi: i64, gj: i64) -> f32 {
        let (ci, cj) = (gi >> 1, gj >> 1);
        let (fi, fj) = (gi & 1 == 1, gj & 1 == 1);
        let mut s = NO_LAKE;
        for cj2 in cj..=cj + i64::from(fj) {
            for ci2 in ci..=ci + i64::from(fi) {
                s = s.max(self.lake_z[idx(self.w, self.h, ci2, cj2)]);
            }
        }
        s
    }

    /// The [`basin`] class of fine cell (gi, gj) of the ×2 grid: closed ground when every coarse
    /// cell at the corners of its interpolation quad is (ledger: AMP-096), and a sink on a coarse
    /// sink's own node (ledger: AMP-097); open when the level has no classes. It is the cell's
    /// class in a window ([`Level::sample`]) and on the next level (`tiles::LevelSink`).
    #[inline]
    pub fn basin_class(&self, gi: i64, gj: i64) -> u8 {
        if self.closed.len() != self.cells() {
            return basin::OPEN;
        }
        let (ci, cj) = (gi >> 1, gj >> 1);
        let (fi, fj) = (gi & 1 == 1, gj & 1 == 1);
        for cj2 in cj..=cj + i64::from(fj) {
            for ci2 in ci..=ci + i64::from(fi) {
                if self.closed[idx(self.w, self.h, ci2, cj2)] == basin::OPEN {
                    return basin::OPEN;
                }
            }
        }
        if !fi && !fj && self.closed[idx(self.w, self.h, ci, cj)] == basin::SINK {
            basin::SINK
        } else {
            basin::CLOSED
        }
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
            lake_z: vec![NO_LAKE; n],
            zs: Vec::new(),
            closed: Vec::new(),
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
        l.lake_z = (0..32 * 16)
            .map(|c| if c % 41 == 0 { l.z[c] + 40.0 } else { NO_LAKE })
            .collect();
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
    fn area_prior_of_a_headwater_is_its_own_fine_area() {
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
    }

    #[test]
    fn a_lake_on_flat_ground_covers_the_quads_round_its_cell() {
        // One coarse lake cell on flat ground: every fine cell whose quad has it as a corner (the
        // cell's own point and the half-way cells round it, 3 × 3) lies under its surface.
        let mut l = level(16, 8, |_, _| 0.0);
        l.lake_z[4 * 16 + 4] = 1.0;
        let f = l.sample(&Win::full(32, 16));
        let cells: Vec<usize> = (0..32 * 16).filter(|&c| f.lake[c] == 1).collect();
        let want: Vec<usize> = (7..=9)
            .flat_map(|j| (7..=9).map(move |i| j * 32 + i))
            .collect();
        assert_eq!(cells, want);
        let surface = |c: usize| l.lake_surface((c % 32) as i64, (c / 32) as i64);
        assert!(cells.iter().all(|&c| surface(c) == 1.0));
        assert!(
            (0..32 * 16)
                .filter(|c| !want.contains(c))
                .all(|c| surface(c) == NO_LAKE)
        );
    }

    #[test]
    fn a_lake_shore_is_the_terrain_contour_at_its_surface_and_round() {
        // A round bowl, its centre off the grid, filled to 600 m: the coarse lake is a disc of
        // coarse cells (a staircase of them). On the fine grid every cell under the surface is
        // lake and no other (the shore is the contour), and the shore lies at the same distance
        // from the centre in every direction, to within the fine cells' own step.
        let (cx, cz) = (16.3f32, 8.6f32);
        let bowl = |x: f32, z: f32| 2.0 * ((x - cx).powi(2) + (z - cz).powi(2));
        let mut l = level(32, 16, |i, j| bowl(i as f32, j as f32));
        let surface = 30.0;
        l.lake_z = (0..32 * 16)
            .map(|c| if l.z[c] < surface { surface } else { NO_LAKE })
            .collect();
        let f = l.sample(&Win::full(64, 32));
        let mut shore = Vec::new();
        for j in 1..31 {
            for i in 1..63 {
                let c = j * 64 + i;
                assert_eq!(f.lake[c] == 1, f.z[c] < surface, "({i},{j})");
                let edge = [c - 1, c + 1, c - 64, c + 64]
                    .iter()
                    .any(|&n| f.lake[n] != f.lake[c]);
                if f.lake[c] == 1 && edge {
                    shore.push(
                        ((i as f32 / 2.0 - cx).powi(2) + (j as f32 / 2.0 - cz).powi(2)).sqrt(),
                    );
                }
            }
        }
        let (lo, hi) = shore
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &r| (a.min(r), b.max(r)));
        // The contour's radius is √15 ≈ 3.87 coarse cells; a coarse staircase spreads by a
        // coarse cell or more.
        assert!(shore.len() > 20, "{}", shore.len());
        assert!(hi - lo < 0.75, "shore radius {lo}..{hi} coarse cells");
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The climate's coarse grids (P1-26 T1, `docs/design/p1-26-per-cell-climate.md` §2.1, §4.6): a
//! block of k × k L1 cells is one coarse cell. The flowing ice sheet (`sheet.rs`, about 256
//! columns) and the per-cell energy balance and ocean (`ebm2d.rs`, `ocean2d.rs`, 128 columns) both
//! live on one.
//!
//! **Geometry on the sphere.** Every latitude comes from the map projection
//! (`deeptime_format::projection`): a coarse row's edges are the latitudes of its first and last
//! fine rows' outer edges, so a projection whose rows are not evenly spaced in latitude (Lambert's
//! equal-area cylinder) gives rows of unequal height and nothing here assumes otherwise. Areas,
//! face lengths and centre distances are then exact on the unit sphere:
//!
//! - cell area A = Δλ·(sin φ_N − sin φ_S) (Σ A = 4π);
//! - a row's mean cos φ is (sin φ_N − sin φ_S)/Δφ, so A = Δλ·Δφ·cos̄ φ;
//! - an east–west face is Δφ long and its two centres cos̄ φ·Δλ apart;
//! - a north–south face is cos φ_face·Δλ long and its centres Δφ_c apart (the centres being the
//!   rows' mid-latitudes).
//!
//! Diffusion in flux form, C·A·∂T/∂t = Σ_faces D·G·(T_nb − T), with G = length / distance, then
//! conserves Σ A·T exactly, and on a zonally uniform field reduces to the 1-D band model's
//! finite-volume form (`ebm.rs`), which is a test of the energy balance (T2).
//!
//! **Lengths in metres** (`dx`, `dy`) are the sheet's: true and Earth-normalized, a row never
//! narrower than a quarter of its height (so the poles do not stall the sheet's explicit step).

use deeptime_format::projection::Projection;
use rayon::prelude::*;

/// A coarse grid over an L1-like fine grid (`fw` × `fh`, rows from the north pole, x wrapping).
#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    /// Coarse cells across and down.
    pub w: usize,
    pub h: usize,
    /// Fine cells per coarse cell along each side.
    pub k: usize,
    /// Fine grid.
    pub fw: usize,
    pub fh: usize,
    /// Width of every coarse row and the height of a row, m (true, Earth-normalized; a row is
    /// never narrower than a quarter of its height).
    pub dx: Vec<f64>,
    pub dy: f64,
    /// Latitude of every coarse row's edges, degrees, north first (h + 1 values, from the
    /// projection).
    pub lat_edge: Vec<f64>,
    /// Longitude span of every coarse column, radians (Σ = 2π; the last column may be narrower).
    pub lon_w: Vec<f64>,
}

impl Grid {
    /// The coarse grid of a `fw` × `fh` fine grid whose rows are `sx[j]` m wide and `sy` m high,
    /// about `cells` coarse columns across. The fine rows span the planet pole to pole (a map
    /// picture of the projection, `Projection::picture_lat_deg`).
    pub fn new(fw: usize, fh: usize, sx: &[f64], sy: f64, cells: usize) -> Grid {
        let fh32 = fh as u32;
        Grid::with_edges(fw, fh, sx, sy, cells, &|y| {
            Projection::picture_lat_deg(y as f64, fh32)
        })
    }

    /// [`Grid::new`] with the latitude of every fine row edge `y` (0 north … `fh` south) given
    /// (a test's equal-angle rows).
    pub fn with_edges(
        fw: usize,
        fh: usize,
        sx: &[f64],
        sy: f64,
        cells: usize,
        lat_of_edge: &dyn Fn(usize) -> f64,
    ) -> Grid {
        let k = (fw / cells.max(1)).max(1);
        let (w, h) = (fw.div_ceil(k), fh.div_ceil(k));
        let dy = sy * k as f64;
        let dx = (0..h)
            .map(|j| {
                let rows = (j * k..((j + 1) * k).min(fh))
                    .map(|r| sx[r])
                    .collect::<Vec<_>>();
                let mean = rows.iter().sum::<f64>() / rows.len().max(1) as f64;
                (mean * k as f64).max(0.25 * dy)
            })
            .collect();
        let lat_edge = (0..=h).map(|j| lat_of_edge((j * k).min(fh))).collect();
        let lon_w = (0..w)
            .map(|i| {
                let cols = ((i + 1) * k).min(fw) - i * k;
                std::f64::consts::TAU * cols as f64 / fw as f64
            })
            .collect();
        Grid {
            w,
            h,
            k,
            fw,
            fh,
            dx,
            dy,
            lat_edge,
            lon_w,
        }
    }

    /// The coarse cell holding fine cell `c`.
    pub fn coarse_of(&self, c: usize) -> usize {
        let (i, j) = (c % self.fw, c / self.fw);
        (j / self.k) * self.w + i / self.k
    }

    /// Latitude of row `j`'s middle (between its edges), degrees.
    pub fn lat_c(&self, j: usize) -> f64 {
        0.5 * (self.lat_edge[j] + self.lat_edge[j + 1])
    }

    /// Row `j`'s height in latitude, radians.
    pub fn dphi(&self, j: usize) -> f64 {
        (self.lat_edge[j] - self.lat_edge[j + 1]).to_radians()
    }

    /// sin φ_N − sin φ_S of row `j` (its area per radian of longitude on the unit sphere).
    pub fn sin_span(&self, j: usize) -> f64 {
        self.lat_edge[j].to_radians().sin() - self.lat_edge[j + 1].to_radians().sin()
    }

    /// Row `j`'s mean cos φ: (sin φ_N − sin φ_S)/Δφ.
    pub fn cos_mean(&self, j: usize) -> f64 {
        let d = self.dphi(j);
        if d > 0.0 { self.sin_span(j) / d } else { 0.0 }
    }

    /// Area of cell (i, j) on the unit sphere.
    pub fn area(&self, i: usize, j: usize) -> f64 {
        self.lon_w[i] * self.sin_span(j)
    }

    /// Area of every cell on the unit sphere, rows north first.
    pub fn areas(&self) -> Vec<f64> {
        (0..self.w * self.h)
            .map(|c| self.area(c % self.w, c / self.w))
            .collect()
    }

    /// Conductance factor G (face length / centre distance, unit sphere) of the east–west face
    /// between cell (i, j) and cell (i + 1, j) (x wraps).
    pub fn g_ew(&self, i: usize, j: usize) -> f64 {
        let i1 = (i + 1) % self.w;
        let dist = self.cos_mean(j).max(1e-12) * 0.5 * (self.lon_w[i] + self.lon_w[i1]);
        self.dphi(j) / dist
    }

    /// Conductance factor G of the north–south face between cell (i, j) and cell (i, j + 1) (the
    /// next row south); 0 for the last row.
    pub fn g_ns(&self, i: usize, j: usize) -> f64 {
        if j + 1 >= self.h {
            return 0.0;
        }
        let len = self.lat_edge[j + 1].to_radians().cos().max(0.0) * self.lon_w[i];
        let dist = (self.lat_c(j) - self.lat_c(j + 1)).to_radians();
        if dist > 0.0 { len / dist } else { 0.0 }
    }

    /// Block means of a fine field over the cells `pick` accepts (all cells where it accepts none).
    pub fn coarsen(&self, v: &[f32], pick: &(dyn Fn(usize) -> bool + Sync)) -> Vec<f32> {
        self.coarsen_weighted(v, pick, &|_| 1.0)
    }

    /// [`Grid::coarsen`] weighted by each fine cell's true area, for shares and means over the
    /// sphere (the climate's land fraction, heights). The map picture's rows are equal-area
    /// (uniform in sin φ, Lambert's cylinder), so every fine cell weighs the same.
    pub fn coarsen_area(&self, v: &[f32], pick: &(dyn Fn(usize) -> bool + Sync)) -> Vec<f32> {
        self.coarsen_weighted(v, pick, &|_| 1.0)
    }

    fn coarsen_weighted(
        &self,
        v: &[f32],
        pick: &(dyn Fn(usize) -> bool + Sync),
        wt: &(dyn Fn(usize) -> f64 + Sync),
    ) -> Vec<f32> {
        (0..self.w * self.h)
            .into_par_iter()
            .map(|cc| {
                let (ci, cj) = (cc % self.w, cc / self.w);
                let (mut s, mut n, mut s_all, mut n_all) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
                for j in cj * self.k..((cj + 1) * self.k).min(self.fh) {
                    for i in ci * self.k..((ci + 1) * self.k).min(self.fw) {
                        let c = j * self.fw + i;
                        let x = f64::from(v[c]);
                        if !x.is_finite() {
                            continue;
                        }
                        let a = wt(c);
                        s_all += a * x;
                        n_all += a;
                        if pick(c) {
                            s += a * x;
                            n += a;
                        }
                    }
                }
                if n > 0.0 {
                    (s / n) as f32
                } else if n_all > 0.0 {
                    (s_all / n_all) as f32
                } else {
                    0.0
                }
            })
            .collect()
    }

    /// The four coarse cells around fine cell `c` and their bilinear weights (x wraps; rows clamp).
    pub fn corners(&self, c: usize) -> [(usize, f64); 4] {
        let k = self.k as f64;
        let (i, j) = (c % self.fw, c / self.fw);
        let x = (i as f64 + 0.5) / k - 0.5;
        let y = ((j as f64 + 0.5) / k - 0.5).clamp(0.0, (self.h - 1) as f64);
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let i0 = (x0 as i64).rem_euclid(self.w as i64) as usize;
        let i1 = (i0 + 1) % self.w;
        let j0 = y0 as usize;
        let j1 = (j0 + 1).min(self.h - 1);
        [
            (j0 * self.w + i0, (1.0 - fx) * (1.0 - fy)),
            (j0 * self.w + i1, fx * (1.0 - fy)),
            (j1 * self.w + i0, (1.0 - fx) * fy),
            (j1 * self.w + i1, fx * fy),
        ]
    }

    /// A coarse field at every fine cell, bilinearly (x wraps; rows clamp).
    pub fn refine(&self, v: &[f32]) -> Vec<f32> {
        (0..self.fw * self.fh)
            .into_par_iter()
            .map(|c| {
                self.corners(c)
                    .iter()
                    .map(|&(cc, wt)| wt * f64::from(v[cc]))
                    .sum::<f64>() as f32
            })
            .collect()
    }

    /// A coarse field at fine cell `c`, bilinearly over only the surrounding coarse cells `has`
    /// accepts (weights renormalized); `None` when none of the four has it (the caller then uses
    /// the other medium's value: a tiny island takes the sea's).
    pub fn masked_at(&self, v: &[f64], has: &dyn Fn(usize) -> bool, c: usize) -> Option<f64> {
        let (mut s, mut wsum) = (0.0, 0.0);
        for (cc, wt) in self.corners(c) {
            if wt > 0.0 && has(cc) {
                s += wt * v[cc];
                wsum += wt;
            }
        }
        // A corner with weight 0 (exactly on a coarse centre) still counts as present: the
        // value there is the one coarse cell's.
        if wsum > 0.0 {
            return Some(s / wsum);
        }
        self.corners(c)
            .iter()
            .find(|(cc, _)| has(*cc))
            .map(|&(cc, _)| v[cc])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn earthish(fw: usize, cells: usize) -> Grid {
        let fh = Projection::picture_height(fw as u32) as usize;
        Grid::new(fw, fh, &vec![1.0e4; fh], 1.0e4, cells)
    }

    #[test]
    fn the_cells_tile_the_sphere() {
        for (fw, cells) in [(1024, 128), (2048, 128), (1000, 128), (64, 16)] {
            let g = earthish(fw, cells);
            let total: f64 = g.areas().iter().sum();
            assert!(
                (total - 4.0 * std::f64::consts::PI).abs() < 1e-11,
                "{fw}/{cells}: Σ A = {total}"
            );
            assert!((g.lat_edge[0] - 90.0).abs() < 1e-12);
            assert!((g.lat_edge[g.h] + 90.0).abs() < 1e-12);
            assert!(g.lat_edge.windows(2).all(|e| e[0] > e[1]), "north first");
        }
        let g = earthish(1024, 128);
        assert_eq!((g.w, g.h, g.k), (128, 41, 8));
    }

    #[test]
    fn faces_are_the_sphere_s_lengths_over_distances() {
        let g = earthish(1024, 128);
        // A row's area is its mean cos φ × Δφ × Δλ; the equatorial rows are almost squares.
        for j in 0..g.h {
            let a = g.area(0, j);
            assert!((a - g.cos_mean(j) * g.dphi(j) * g.lon_w[0]).abs() < 1e-15);
        }
        let eq = g.h / 2;
        assert!((g.g_ew(0, eq) - 1.0).abs() < 1e-2, "{}", g.g_ew(0, eq));
        assert!(
            (g.g_ns(0, eq - 1) - 1.0).abs() < 1e-2,
            "{}",
            g.g_ns(0, eq - 1)
        );
        // No face beyond the last row; the polar rows' east–west faces are long against their
        // centres' distance.
        assert_eq!(g.g_ns(5, g.h - 1), 0.0);
        assert!(g.g_ew(0, 0) > 20.0);
    }

    #[test]
    fn coarsen_and_refine_round_trip() {
        let g = Grid::new(64, 32, &[1.0; 32], 1.0, 16);
        assert_eq!((g.w, g.h, g.k), (16, 8, 4));
        let v: Vec<f32> = (0..64 * 32).map(|c| (c / 64) as f32).collect();
        let cv = g.coarsen(&v, &|_| true);
        assert!((cv[0] - 1.5).abs() < 1e-6);
        let back = g.refine(&cv);
        assert!((back[10 * 64 + 5] - 10.0).abs() < 0.6);
        // A constant survives coarsening (either way) and refining exactly.
        let c7 = vec![7.0f32; 64 * 32];
        for cv in [
            g.coarsen(&c7, &|_| true),
            g.coarsen_area(&c7, &|c| c % 3 == 0),
        ] {
            assert!(cv.iter().all(|&x| x == 7.0));
            assert!(g.refine(&cv).iter().all(|&x| (x - 7.0).abs() < 1e-6));
        }
    }

    #[test]
    fn the_masked_refine_reads_only_its_medium() {
        let g = Grid::new(64, 32, &[1.0; 32], 1.0, 16);
        // Land (value 10) in the western half of the coarse grid, sea (value −5) in the east.
        let land = |cc: usize| (cc % g.w) < g.w / 2;
        let v: Vec<f64> = (0..g.w * g.h)
            .map(|cc| if land(cc) { 10.0 } else { -5.0 })
            .collect();
        for c in 0..64 * 32 {
            if let Some(x) = g.masked_at(&v, &land, c) {
                assert_eq!(x, 10.0, "cell {c} read the sea");
            }
            let sea = |cc: usize| !land(cc);
            if let Some(x) = g.masked_at(&v, &sea, c) {
                assert_eq!(x, -5.0, "cell {c} read the land");
            }
        }
        // Far inside the sea, no land corner: None.
        let deep = 16 * 64 + 48;
        assert!(g.masked_at(&v, &land, deep).is_none());
        assert!(g.masked_at(&v, &|_| false, deep).is_none());
    }
}

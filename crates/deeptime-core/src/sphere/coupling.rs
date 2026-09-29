// SPDX-License-Identifier: LGPL-3.0-or-later
//! Block space ↔ sphere, and the L0 → raster coupling (plan §5.2).
//!
//! # Conventions
//! - **Longitude** λ = 360°·(x + C/2)/C, in [0°, 360°): block column x = −C/2 (the seam) is λ = 0,
//!   x = 0 is λ = 180°, and x wraps with period C, so x = C/2 is the seam again.
//! - **Latitude** φ = −360°·z/C, north positive; north is −z, as in Minecraft. z ∈ [−C/4, C/4]
//!   spans pole to pole (0.02197° per block at C = 16,384), and φ clamps to ±90° beyond.
//! - **Unit vector** v = (cos φ cos λ, cos φ sin λ, sin φ): +Z is the north pole and +X the seam
//!   meridian.
//! - **Raster cells** follow `deeptime_format::GridDesc`: cell (i, j) is the point sample at block
//!   (origin_block_x + i·r, origin_block_z + j·r), so column 0 sits on the seam and row 0 on the
//!   north pole; i wraps modulo W = C/r.
//!
//! # Coupling
//! Every raster cell keeps a [`CellLink`]: the L0 triangle whose cone holds the cell's direction,
//! its barycentric weights there, and the nearest sample. [`Coupling::build`] fills all of them
//! after a resampling (a k-d-tree query and a short walk per cell, rows in parallel);
//! [`Coupling::refresh`] redoes only the cells a predicate marks stale (plates, P1-03, mark the
//! cells whose triangle left its plate). Scalar fields interpolate barycentrically
//! ([`Coupling::interpolate`]); categorical ones (plate id, crust type, orogeny type) take the
//! nearest sample ([`Coupling::categorical`]).

use glam::DVec3;
use rayon::prelude::*;

use deeptime_format::GridDesc;

use super::{SampleTree, SphereMesh};

/// The unit vector at latitude `lat_deg` and longitude `lon_deg` (degrees).
pub fn unit_from_lat_lon_deg(lat_deg: f64, lon_deg: f64) -> DVec3 {
    let (sp, cp) = lat_deg.to_radians().sin_cos();
    let (sl, cl) = lon_deg.to_radians().sin_cos();
    DVec3::new(cp * cl, cp * sl, sp)
}

/// Latitude in [−90°, 90°] and longitude in [0°, 360°) of the direction `v` (any length > 0).
/// At the poles the longitude is 0.
pub fn lat_lon_deg(v: DVec3) -> (f64, f64) {
    let lat = v.z.atan2(v.x.hypot(v.y)).to_degrees();
    (lat, wrap_deg(v.y.atan2(v.x).to_degrees()))
}

/// Wraps an angle into [0°, 360°).
fn wrap_deg(a: f64) -> f64 {
    let w = a.rem_euclid(360.0);
    if w >= 360.0 { 0.0 } else { w }
}

/// The equirectangular mapping between a planet's block space and its sphere, for circumference
/// C blocks (plan §5.1–§5.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockFrame {
    c: f64,
}

impl BlockFrame {
    /// The frame of a planet `circumference` blocks around.
    pub fn new(circumference: u32) -> BlockFrame {
        BlockFrame {
            c: f64::from(circumference),
        }
    }

    /// C, in blocks.
    pub fn circumference(&self) -> f64 {
        self.c
    }

    /// Wraps block x into [−C/2, C/2).
    pub fn wrap_x(&self, x: f64) -> f64 {
        let t = (x + 0.5 * self.c).rem_euclid(self.c);
        (if t >= self.c { 0.0 } else { t }) - 0.5 * self.c
    }

    /// Longitude of block x, in [0°, 360°).
    pub fn lon_deg(&self, x: f64) -> f64 {
        wrap_deg(360.0 * (self.wrap_x(x) + 0.5 * self.c) / self.c)
    }

    /// Latitude of block z, in [−90°, 90°] (north is −z).
    pub fn lat_deg(&self, z: f64) -> f64 {
        (-360.0 * z / self.c).clamp(-90.0, 90.0)
    }

    /// Block x of a longitude, in [−C/2, C/2).
    pub fn x_of_lon(&self, lon_deg: f64) -> f64 {
        self.wrap_x(wrap_deg(lon_deg) / 360.0 * self.c - 0.5 * self.c)
    }

    /// Block z of a latitude, in [−C/4, C/4].
    pub fn z_of_lat(&self, lat_deg: f64) -> f64 {
        -lat_deg.clamp(-90.0, 90.0) / 360.0 * self.c
    }

    /// The unit vector of block (x, z).
    pub fn unit(&self, x: f64, z: f64) -> DVec3 {
        unit_from_lat_lon_deg(self.lat_deg(z), self.lon_deg(x))
    }

    /// Block (x, z) of the direction `v`, x in [−C/2, C/2).
    pub fn block(&self, v: DVec3) -> (f64, f64) {
        let (lat, lon) = lat_lon_deg(v);
        (self.x_of_lon(lon), self.z_of_lat(lat))
    }
}

/// How one raster cell samples L0.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CellLink {
    /// The triangle whose cone holds the cell's direction.
    pub tri: u32,
    /// The sample nearest the cell (for categorical fields).
    pub nearest: u32,
    /// Barycentric weights, in the order of the triangle's vertices; non-negative, sum 1.
    pub w: [f32; 3],
}

/// The cell → (triangle, weights, nearest sample) lookup for one raster grid.
#[derive(Clone, Debug)]
pub struct Coupling {
    grid: GridDesc,
    frame: BlockFrame,
    links: Vec<CellLink>,
}

impl Coupling {
    /// Links every cell of `grid` to `mesh` (the triangulation) and `tree` (a k-d tree over the
    /// same points). Rows run in parallel on the current Rayon pool; within a row each walk starts
    /// from the previous cell's triangle.
    pub fn build(grid: &GridDesc, mesh: &SphereMesh, tree: &SampleTree) -> Coupling {
        let frame = BlockFrame::new(grid.circumference() as u32);
        let mut links = vec![CellLink::default(); grid.cells()];
        links
            .par_chunks_mut(grid.w as usize)
            .enumerate()
            .for_each(|(j, row)| {
                let z = grid.block_z(j as i64) as f64;
                let mut hint = None;
                for (i, link) in row.iter_mut().enumerate() {
                    let q = frame.unit(grid.block_x(i as i64) as f64, z);
                    *link = link_cell(mesh, tree, q, hint);
                    hint = Some(link.tri);
                }
            });
        Coupling {
            grid: grid.clone(),
            frame,
            links,
        }
    }

    /// Re-links the cells for which `stale(cell index, current link)` holds, each from a fresh
    /// k-d-tree query, and returns how many it re-linked. The cell index is j·W + i.
    pub fn refresh(
        &mut self,
        mesh: &SphereMesh,
        tree: &SampleTree,
        stale: impl Fn(usize, &CellLink) -> bool + Sync,
    ) -> usize {
        let (grid, frame) = (&self.grid, self.frame);
        let w = grid.w as usize;
        self.links
            .par_chunks_mut(w)
            .enumerate()
            .map(|(j, row)| {
                let z = grid.block_z(j as i64) as f64;
                let mut count = 0;
                for (i, link) in row.iter_mut().enumerate() {
                    if stale(j * w + i, link) {
                        let q = frame.unit(grid.block_x(i as i64) as f64, z);
                        *link = link_cell(mesh, tree, q, None);
                        count += 1;
                    }
                }
                count
            })
            .sum()
    }

    /// Re-links every cell through `f(cell index, cell direction, link)`, in parallel on the
    /// current Rayon pool, and returns what `f` returns per cell (row-major, index j·W + i). For
    /// samples that move between rebuilds (the plates, P1-03): the caller locates each cell in its
    /// own frames and writes the triangle, weights and nearest sample it found, so
    /// [`Coupling::interpolate`] and [`Coupling::categorical`] keep working on the moved crust.
    pub fn relink<T: Send>(
        &mut self,
        f: impl Fn(usize, DVec3, &mut CellLink) -> T + Sync,
    ) -> Vec<T> {
        let (grid, frame) = (&self.grid, self.frame);
        let w = grid.w as usize;
        let rows: Vec<Vec<T>> = self
            .links
            .par_chunks_mut(w)
            .enumerate()
            .map(|(j, row)| {
                let z = grid.block_z(j as i64) as f64;
                row.iter_mut()
                    .enumerate()
                    .map(|(i, link)| {
                        let q = frame.unit(grid.block_x(i as i64) as f64, z);
                        f(j * w + i, q, link)
                    })
                    .collect()
            })
            .collect();
        rows.into_iter().flatten().collect()
    }

    /// The grid this coupling serves.
    pub fn grid(&self) -> &GridDesc {
        &self.grid
    }

    /// The block frame of the grid's planet.
    pub fn frame(&self) -> BlockFrame {
        self.frame
    }

    /// Every cell's link, row-major (index j·W + i).
    pub fn links(&self) -> &[CellLink] {
        &self.links
    }

    /// The link of cell (i, j), i wrapped and j clamped as the grid does.
    pub fn link(&self, i: i64, j: i64) -> CellLink {
        let (i, j) = (self.grid.wrap_i(i) as usize, self.grid.clamp_j(j) as usize);
        self.links[j * self.grid.w as usize + i]
    }

    /// The unit vector of cell (i, j).
    pub fn cell_unit(&self, i: i64, j: i64) -> DVec3 {
        let (i, j) = (
            i64::from(self.grid.wrap_i(i)),
            i64::from(self.grid.clamp_j(j)),
        );
        self.frame
            .unit(self.grid.block_x(i) as f64, self.grid.block_z(j) as f64)
    }

    /// A scalar field sampled onto the grid: Σ wₖ·value(vertex k) per cell. `value` reads the
    /// field at a sample index of `mesh` (the mesh the coupling was built with).
    pub fn interpolate(&self, mesh: &SphereMesh, value: impl Fn(u32) -> f64 + Sync) -> Vec<f64> {
        let tris = mesh.triangles();
        self.links
            .par_iter()
            .map(|l| {
                let t = tris[l.tri as usize];
                (0..3).map(|k| f64::from(l.w[k]) * value(t[k])).sum()
            })
            .collect()
    }

    /// A categorical field sampled onto the grid: the value at each cell's nearest sample.
    pub fn categorical<T: Send>(&self, value: impl Fn(u32) -> T + Sync) -> Vec<T> {
        self.links.par_iter().map(|l| value(l.nearest)).collect()
    }
}

/// Links one direction: nearest sample from the tree, then a walk from `hint` (or from a
/// triangle of the nearest sample) to the containing triangle.
fn link_cell(mesh: &SphereMesh, tree: &SampleTree, q: DVec3, hint: Option<u32>) -> CellLink {
    let (nearest, _) = tree.nearest(q);
    let start = hint.or_else(|| mesh.vertex_triangle(nearest)).unwrap_or(0);
    let tri = mesh.locate(q, start);
    let w = mesh.barycentric(tri, q);
    CellLink {
        tri,
        nearest,
        w: w.map(|x| x as f32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sphere::{SplitMix64, fibonacci_sphere};

    /// The acceptance check of P1-02 on the 16k-quick ladder (L0 100k samples, L1 1024×512 at
    /// r 16): every L1 cell has weights summing to 1 within 1e-6, none below −1e-9, and a triangle
    /// that contains the cell's direction (exact test). The raw weights are recomputed here in
    /// f64 without the clamp, so the containment is checked independently of the locator.
    #[test]
    fn every_l1_cell_has_valid_barycentric_weights() {
        let pts = fibonacci_sphere(100_000);
        let mesh = SphereMesh::build(&pts).unwrap();
        let tree = SampleTree::new(&pts);
        let grid = GridDesc::for_planet("L1", 16_384, 16).unwrap();
        assert_eq!((grid.w, grid.h), (1024, 512));
        let coupling = Coupling::build(&grid, &mesh, &tree);
        assert_eq!(coupling.links().len(), 1024 * 512);
        let bad: Vec<String> = (0..grid.cells())
            .into_par_iter()
            .filter_map(|c| {
                let (i, j) = ((c % 1024) as i64, (c / 1024) as i64);
                let l = coupling.link(i, j);
                let q = coupling.cell_unit(i, j);
                let sum: f64 = l.w.iter().map(|&x| f64::from(x)).sum();
                let [a, b, cc] = mesh.triangles()[l.tri as usize].map(|v| pts[v as usize]);
                let raw = [q.dot(b.cross(cc)), q.dot(cc.cross(a)), q.dot(a.cross(b))];
                let s = raw[0] + raw[1] + raw[2];
                let ok = (sum - 1.0).abs() <= 1e-6
                    && l.w.iter().all(|&x| f64::from(x) >= -1e-9)
                    && raw.iter().all(|&x| x / s >= -1e-9)
                    && mesh.contains(l.tri, q);
                (!ok).then(|| format!("cell ({i}, {j}): w {:?} sum {sum} raw {raw:?}", l.w))
            })
            .collect();
        assert!(bad.is_empty(), "{} bad cells, first {}", bad.len(), bad[0]);
        // The nearest sample agrees with the tree, and row 0 (the north pole) is one direction.
        for c in (0..grid.cells()).step_by(997) {
            let q = coupling.cell_unit((c % 1024) as i64, (c / 1024) as i64);
            let l = coupling.links()[c];
            assert!(
                (pts[l.nearest as usize].distance_squared(q) - tree.nearest(q).1).abs() < 1e-15
            );
        }
        assert!((coupling.cell_unit(0, 0) - DVec3::Z).length() < 1e-15);
        assert!((coupling.cell_unit(517, 0) - DVec3::Z).length() < 1e-15);
    }

    #[test]
    fn interpolation_is_exact_for_constants_and_close_for_smooth_fields() {
        let pts = fibonacci_sphere(20_000);
        let mesh = SphereMesh::build(&pts).unwrap();
        let tree = SampleTree::new(&pts);
        let grid = GridDesc::for_planet("L1", 16_384, 32).unwrap();
        let coupling = Coupling::build(&grid, &mesh, &tree);
        let ones = coupling.interpolate(&mesh, |_| 1.0);
        assert!(ones.iter().all(|&x| (x - 1.0).abs() < 1e-6));
        // sin(latitude) is smooth: the error is second order in the spacing (0.025 rad here).
        let z = coupling.interpolate(&mesh, |v| pts[v as usize].z);
        let mut worst = 0.0f64;
        for (c, &zi) in z.iter().enumerate() {
            let q = coupling.cell_unit((c % grid.w as usize) as i64, (c / grid.w as usize) as i64);
            worst = worst.max((zi - q.z).abs());
        }
        assert!(worst < 1e-3, "worst interpolation error {worst}");
        let ids = coupling.categorical(|v| v);
        assert!(
            ids.iter()
                .zip(coupling.links())
                .all(|(&id, l)| id == l.nearest)
        );
    }

    #[test]
    fn refresh_relinks_only_stale_cells() {
        let pts = fibonacci_sphere(5_000);
        let mesh = SphereMesh::build(&pts).unwrap();
        let tree = SampleTree::new(&pts);
        let grid = GridDesc::for_planet("L1", 16_384, 64).unwrap();
        let fresh = Coupling::build(&grid, &mesh, &tree);
        let mut c = fresh.clone();
        for l in c.links.iter_mut().step_by(3) {
            *l = CellLink::default();
        }
        let n = c.refresh(&mesh, &tree, |k, _| k % 3 == 0);
        assert_eq!(n, grid.cells().div_ceil(3));
        for (k, (a, b)) in c.links().iter().zip(fresh.links()).enumerate() {
            // A cell on a shared edge may land in either triangle; the weights then agree on the
            // shared vertices, and the direction is contained either way.
            if a.tri != b.tri {
                let q = c.cell_unit((k % grid.w as usize) as i64, (k / grid.w as usize) as i64);
                assert!(mesh.contains(a.tri, q), "cell {k}");
            }
            assert_eq!(a.nearest, b.nearest);
        }
        assert_eq!(c.refresh(&mesh, &tree, |_, _| false), 0);
    }

    // ---- The block ↔ lat/lon ↔ unit-vector mapping (property tests over random inputs). ----

    const CS: [u32; 2] = [16_384, 32_768];

    /// Wrap-aware difference of two block x values.
    fn dx(frame: &BlockFrame, a: f64, b: f64) -> f64 {
        frame.wrap_x(a - b).abs()
    }

    #[test]
    fn fixed_points_of_the_mapping() {
        for c in CS {
            let f = BlockFrame::new(c);
            let (h, q) = (f64::from(c) / 2.0, f64::from(c) / 4.0);
            assert_eq!(f.lon_deg(-h), 0.0, "the seam is longitude 0");
            assert_eq!(f.lon_deg(h), 0.0, "x = C/2 is the seam again");
            assert_eq!(f.lon_deg(0.0), 180.0);
            assert_eq!(f.lat_deg(-q), 90.0, "north is -z");
            assert_eq!(f.lat_deg(q), -90.0);
            assert_eq!(f.lat_deg(0.0), 0.0);
            assert_eq!(f.lat_deg(-2.0 * q), 90.0, "latitude clamps beyond the pole");
            assert!((f.unit(0.0, -q) - DVec3::Z).length() < 1e-15);
            assert!((f.unit(-h, 0.0) - DVec3::X).length() < 1e-15);
            assert!((f.unit(0.0, 0.0) + DVec3::X).length() < 1e-15);
            // Grid rows: row 0 is the north pole, row H/2 the equator.
            let g = GridDesc::for_planet("L1", c, 16).unwrap();
            assert_eq!(f.lat_deg(g.block_z(0) as f64), 90.0);
            assert_eq!(f.lat_deg(g.block_z(i64::from(g.h) / 2) as f64), 0.0);
            assert_eq!(f.lon_deg(g.block_x(0) as f64), 0.0);
            // One block of x is 360°/C of longitude.
            assert!((f.lon_deg(-h + 1.0) - 360.0 / f64::from(c)).abs() < 1e-12);
        }
    }

    #[test]
    fn prop_block_lat_lon_round_trip() {
        let mut rng = SplitMix64::new(0xB10C);
        for c in CS {
            let f = BlockFrame::new(c);
            let (h, q) = (f64::from(c) / 2.0, f64::from(c) / 4.0);
            for _ in 0..20_000 {
                let x = -h + rng.next_f64() * f64::from(c);
                let z = -q + rng.next_f64() * 2.0 * q;
                // Through latitude/longitude.
                let (lat, lon) = (f.lat_deg(z), f.lon_deg(x));
                assert!((0.0..360.0).contains(&lon) && (-90.0..=90.0).contains(&lat));
                assert!(dx(&f, f.x_of_lon(lon), x) < 1e-9, "x {x}");
                assert!((f.z_of_lat(lat) - z).abs() < 1e-9, "z {z}");
                // Through the unit vector (longitude is ill-conditioned by 1/cos φ at the poles).
                let v = f.unit(x, z);
                assert!((v.length() - 1.0).abs() < 1e-15);
                let (x2, z2) = f.block(v);
                assert!((-h..h).contains(&x2));
                assert!((z2 - z).abs() < 1e-8, "z {z} -> {z2}");
                if lat.abs() < 89.0 {
                    assert!(dx(&f, x2, x) < 1e-8, "x {x} -> {x2}");
                }
                // Lat/lon through the unit vector.
                let (lat2, lon2) = lat_lon_deg(v);
                assert!((lat2 - lat).abs() < 1e-10);
                if lat.abs() < 89.0 {
                    let d = (lon2 - lon).rem_euclid(360.0);
                    assert!(d.min(360.0 - d) < 1e-10);
                }
            }
        }
    }

    #[test]
    fn prop_x_wraps_at_the_seam() {
        let mut rng = SplitMix64::new(0x5EA1);
        for c in CS {
            let f = BlockFrame::new(c);
            let (cc, h, q) = (f64::from(c), f64::from(c) / 2.0, f64::from(c) / 4.0);
            for _ in 0..20_000 {
                let x = (rng.next_f64() - 0.5) * 8.0 * cc; // four circumferences either way
                let z = -q + rng.next_f64() * 2.0 * q;
                let w = f.wrap_x(x);
                assert!((-h..h).contains(&w), "wrap_x({x}) = {w}");
                assert!(((x - w) / cc - ((x - w) / cc).round()).abs() < 1e-9);
                let v = f.unit(x, z);
                for k in [-2.0, -1.0, 1.0, 3.0] {
                    assert!((f.unit(x + k * cc, z) - v).length() < 1e-9);
                }
                assert!(dx(&f, f.x_of_lon(f.lon_deg(x)), x) < 1e-7);
                let lon = (rng.next_f64() - 0.5) * 2000.0;
                let xl = f.x_of_lon(lon);
                assert!((-h..h).contains(&xl));
                let d = (f.lon_deg(xl) - lon).rem_euclid(360.0);
                assert!(d.min(360.0 - d) < 1e-9);
            }
            // The two columns either side of the seam are one block apart on the sphere.
            for lat in [0.0, 30.0, -60.0] {
                let z = f.z_of_lat(lat);
                let (a, b) = (f.unit(h - 1.0, z), f.unit(-h, z));
                let one_block = std::f64::consts::TAU / cc * f64::cos(f64::to_radians(lat));
                assert!((a.angle_between(b) - one_block).abs() < 1e-9 * one_block.max(1e-3));
                assert!((f.unit(h, z) - b).length() < 1e-12);
            }
        }
    }
}

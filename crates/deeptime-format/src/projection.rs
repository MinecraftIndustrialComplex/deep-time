// SPDX-License-Identifier: LGPL-3.0-or-later
//! The map projection between block space and the sphere: the one place every conversion between
//! block coordinates, raster rows and latitude/longitude goes through
//! (`docs/design/equal-area-projection.md` §2; the Java twin is `worlddata/Projection.java`, the
//! pages' is `window.DTProj` in `deeptime-viewer/assets/projection.js`).
//!
//! The projection is **equirectangular** (plan §5.2): block x is linear in longitude and block z
//! in latitude, one block being 360°/C both ways. Ben reopened the choice on 2026-10-02 (Lambert's
//! equal-area cylinder, or a square tiling map); whichever he picks replaces the bodies of these
//! functions, and the family-specific work of the design note §5 follows. Until then every
//! function reproduces, operation for operation, the arithmetic its callers did by hand, so the
//! refactor changes no output bit (`tools/review/projection-identity.sh`).
//!
//! # Conventions
//! - **Longitude** λ = 360°·(x + C/2)/C in [0°, 360°): x = −C/2 (the seam) is 0°, x wraps with
//!   period C.
//! - **Latitude** φ = −360°·z/C, north positive; north is −z, as in Minecraft. The poles are at
//!   z = ∓C/4 ([`Projection::pole_z`]) and φ clamps to ±90° beyond them.
//! - **Unit vector** (cos φ cos λ, cos φ sin λ, sin φ): +Z the north pole, +X the seam meridian.
//! - **Grids** (`GridDesc`) are square lattices in block space: W = C/r, H = W/2, origin
//!   (−C/2, −C/4) ([`Projection::grid_origin_z`], [`Projection::grid_rows`]).
//! - **Map pictures** of the whole planet span the grid: x linear in longitude from the seam, y
//!   linear in z from the north pole; aspect 2 : 1 ([`Projection::picture_height`]).
//!
//! - **Topology**: a raster wraps in x and stops at its first and last rows
//!   ([`wrap_col`], [`clamp_row`], [`neighbour`]); block x wraps with period C and the planet
//!   ends at the poles ([`Projection::on_planet`]).
//! - **Metric**: a block is cos φ blocks of planet east–west and one north–south
//!   ([`Projection::scale_ew`], [`Projection::scale_ns`], [`Projection::metric`]), so a block
//!   square stands for cos φ of its area ([`Projection::area_scale`]); north is −z everywhere
//!   ([`Projection::north`]).
//!
//! The spatial indexes of the world data (the river tiles, the structure buckets, the site index,
//! the playability regions) are frames for lookups, not geography: lattices of their own cells
//! over the planet's extent, from [`Projection::index_origin`] over [`Projection::index_span`]
//! blocks.
//!
//! The projection is a game choice (ledger SCL-049).

/// Block space ↔ sphere for one planet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projection {
    c: f64,
}

impl Projection {
    /// The projection of a planet `circumference` blocks round.
    pub fn new(circumference: u32) -> Projection {
        Projection::of(f64::from(circumference))
    }

    /// [`Projection::new`] for a circumference held as a float.
    pub fn of(circumference: f64) -> Projection {
        Projection { c: circumference }
    }

    /// C, blocks.
    pub fn circumference(&self) -> f64 {
        self.c
    }

    /// The planet's radius R = C / 2π, blocks (the sphere the blocks stand for, at true scale on
    /// the equator).
    pub fn radius(&self) -> f64 {
        self.c / std::f64::consts::TAU
    }

    /// |z| of the poles, blocks: C/4. The world's playable z range is [−pole_z, pole_z].
    pub fn pole_z(&self) -> f64 {
        self.c / 4.0
    }

    /// The planet's extent in block space, blocks: (east–west, north–south) = (C, C/2).
    pub fn extent(&self) -> (f64, f64) {
        (self.c, 2.0 * self.pole_z())
    }

    /// Whether block (x, z) is on the planet: between the poles (x wraps, so every x is).
    pub fn on_planet(&self, _x: f64, z: f64) -> bool {
        z.abs() <= self.pole_z()
    }

    /// Whether block z lies strictly between the poles.
    pub fn inside_poles(&self, z: f64) -> bool {
        z.abs() < self.pole_z()
    }

    /// Clamps block z to the planet: [−pole_z, pole_z].
    pub fn clamp_z(&self, z: f64) -> f64 {
        z.clamp(-self.pole_z(), self.pole_z())
    }

    /// Wraps block x into [−C/2, C/2).
    pub fn wrap_x(&self, x: f64) -> f64 {
        let t = (x + 0.5 * self.c).rem_euclid(self.c);
        (if t >= self.c { 0.0 } else { t }) - 0.5 * self.c
    }

    /// Longitude of block x, degrees in [0, 360).
    pub fn lon_deg(&self, x: f64) -> f64 {
        wrap_deg(360.0 * (self.wrap_x(x) + 0.5 * self.c) / self.c)
    }

    /// Block x of a longitude, in [−C/2, C/2).
    pub fn x_of_lon(&self, lon_deg: f64) -> f64 {
        self.wrap_x(wrap_deg(lon_deg) / 360.0 * self.c - 0.5 * self.c)
    }

    /// Latitude of block z, degrees in [−90, 90] (north is −z; clamped beyond the poles).
    pub fn lat_deg(&self, z: f64) -> f64 {
        (-360.0 * z / self.c).clamp(-90.0, 90.0)
    }

    /// Block z of a latitude (clamped to ±90°), in [−C/4, C/4].
    pub fn z_of_lat(&self, lat_deg: f64) -> f64 {
        -lat_deg.clamp(-90.0, 90.0) / 360.0 * self.c
    }

    /// Block (x, z) of a latitude and longitude, x in [−C/2, C/2).
    pub fn block_of(&self, lat_deg: f64, lon_deg: f64) -> (f64, f64) {
        (self.x_of_lon(lon_deg), self.z_of_lat(lat_deg))
    }

    /// (latitude, longitude) of block (x, z), degrees.
    pub fn lat_lon(&self, x: f64, z: f64) -> (f64, f64) {
        (self.lat_deg(z), self.lon_deg(x))
    }

    /// The unit vector of block (x, z): (cos φ cos λ, cos φ sin λ, sin φ).
    pub fn unit(&self, x: f64, z: f64) -> [f64; 3] {
        unit_of_lat_lon(self.lat_deg(z), self.lon_deg(x))
    }

    /// cos φ of block z (0 at the poles).
    pub fn cos_lat(&self, z: f64) -> f64 {
        self.lat_deg(z).to_radians().cos().max(0.0)
    }

    /// Blocks of planet per block east–west at block z: cos φ (the metric's x scale).
    pub fn scale_ew(&self, z: f64) -> f64 {
        self.cos_lat(z)
    }

    /// Blocks of planet per block north–south at block z: 1 everywhere.
    pub fn scale_ns(&self, _z: f64) -> f64 {
        1.0
    }

    /// The metric of block space at block (x, z): (g_xx, g_xz, g_zz), blocks of planet per block,
    /// squared. Diagonal for a cylindrical projection: (cos² φ, 0, 1).
    pub fn metric(&self, _x: f64, z: f64) -> (f64, f64, f64) {
        let (e, n) = (self.scale_ew(z), self.scale_ns(z));
        (e * e, 0.0, n * n)
    }

    /// Area of planet per block of block space at block z (√det g): cos φ.
    pub fn area_scale(&self, z: f64) -> f64 {
        self.scale_ew(z) * self.scale_ns(z)
    }

    /// The unit vector of north in block space at block (x, z), as (dx, dz): (0, −1) everywhere.
    pub fn north(&self, _x: f64, _z: f64) -> (f64, f64) {
        (0.0, -1.0)
    }

    /// The great-circle distance between blocks a and b, blocks of planet.
    pub fn great_circle_blocks(&self, a: (f64, f64), b: (f64, f64)) -> f64 {
        let (p1, p2) = (
            self.lat_deg(a.1).to_radians(),
            self.lat_deg(b.1).to_radians(),
        );
        let dl = (self.lon_deg(b.0) - self.lon_deg(a.0)).to_radians();
        // Haversine: well conditioned for short distances.
        let s = ((p2 - p1) / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
        2.0 * self.radius() * s.sqrt().clamp(0.0, 1.0).asin()
    }

    /// Block z of the first row of every grid of this planet: −C/4 (the north pole).
    pub fn grid_origin_z(&self) -> i64 {
        -((self.c as i64) / 4)
    }

    /// Rows of a grid `w` cells wide: W/2 (pole to pole is half the circumference).
    pub fn grid_rows(&self, w: u32) -> u32 {
        w / 2
    }

    /// Block (x, z) of the first cell of the world data's spatial indexes (river tiles, structure
    /// buckets, site index, playability regions): (−C/2, −C/4), the seam and the north pole.
    pub fn index_origin(&self) -> (f64, f64) {
        (-self.c / 2.0, -self.c / 4.0)
    }

    /// Blocks the spatial indexes span: (east–west, north–south) = (C, C/2).
    pub fn index_span(&self) -> (f64, f64) {
        (self.c, self.c / 2.0)
    }

    /// [`Projection::index_origin`] in whole blocks.
    pub fn index_origin_blocks(&self) -> (i64, i64) {
        let c = self.c as i64;
        (-(c / 2), -(c / 4))
    }

    /// [`Projection::index_span`] in whole blocks.
    pub fn index_span_blocks(&self) -> (i64, i64) {
        let c = self.c as i64;
        (c, c / 2)
    }

    /// The latitude, degrees, in the middle of the band from row `j` to row `j + 1` of a grid `h`
    /// rows high (rows from the north pole).
    pub fn band_lat_deg(j: u32, h: u32) -> f64 {
        90.0 - 180.0 * (f64::from(j) + 0.5) / f64::from(h)
    }

    /// The area weight of the band from row `j` to row `j + 1` of a grid `h` rows high, at its
    /// middle: cos φ there.
    pub fn band_weight(j: u32, h: u32) -> f64 {
        (std::f64::consts::PI * (0.5 - (f64::from(j) + 0.5) / f64::from(h))).cos()
    }

    /// The longitude, degrees (not wrapped), at column coordinate `x` of a map picture or grid
    /// `w` columns wide (x = 0 the seam).
    pub fn picture_lon_deg(x: f64, w: u32) -> f64 {
        360.0 * x / f64::from(w.max(1))
    }

    /// The column coordinate, in [0, w), of longitude `lon_deg` in a map picture `w` columns wide.
    pub fn picture_x_of_lon(lon_deg: f64, w: u32) -> f64 {
        wrap_deg(lon_deg) / 360.0 * f64::from(w)
    }

    /// Block z at row coordinate `y` of a map picture `h` rows high (y = 0 the north pole).
    pub fn picture_z(&self, y: f64, h: u32) -> f64 {
        -self.c / 4.0 + y * (self.c / 2.0) / f64::from(h.max(1))
    }

    /// The row coordinate of block z in a map picture `h` rows high.
    pub fn picture_y(&self, z: f64, h: u32) -> f64 {
        (z + self.c / 4.0) * f64::from(h) / (self.c / 2.0)
    }

    /// The latitude, degrees, at row coordinate `y` of a map picture `h` rows high.
    pub fn picture_lat_deg(y: f64, h: u32) -> f64 {
        90.0 - 180.0 * y / f64::from(h.max(1))
    }

    /// The row coordinate of latitude `lat_deg` in a map picture `h` rows high.
    pub fn picture_y_of_lat(lat_deg: f64, h: u32) -> f64 {
        (90.0 - lat_deg) / 180.0 * f64::from(h)
    }

    /// The height of a map picture `w` pixels wide at the projection's aspect (2 : 1).
    pub fn picture_height(w: u32) -> u32 {
        (w / 2).max(1)
    }
}

/// The raster's topology in x: wraps a column index into [0, w).
#[inline]
pub fn wrap_col(i: i64, w: i64) -> i64 {
    i.rem_euclid(w)
}

/// The raster's topology in z: clamps a row index into [0, h − 1] (the rows stop at the poles).
#[inline]
pub fn clamp_row(j: i64, h: i64) -> i64 {
    j.clamp(0, h - 1)
}

/// The neighbour rule of a `w × h` raster: the cell at offset (di, dj) from cell (i, j), or
/// `None` when there is none. Columns wrap when `wraps_x` (the planet's grids do; small test
/// rasters may not), rows stop at the first and the last.
#[inline]
pub fn neighbour(
    w: usize,
    h: usize,
    wraps_x: bool,
    (i, j): (usize, usize),
    (di, dj): (i64, i64),
) -> Option<(usize, usize)> {
    let nj = j as i64 + dj;
    if nj < 0 || nj >= h as i64 {
        return None;
    }
    let mut ni = i as i64 + di;
    if ni < 0 || ni >= w as i64 {
        if !wraps_x {
            return None;
        }
        ni = wrap_col(ni, w as i64);
    }
    Some((ni as usize, nj as usize))
}

/// The unit vector at latitude `lat_deg` and longitude `lon_deg` (degrees).
pub fn unit_of_lat_lon(lat_deg: f64, lon_deg: f64) -> [f64; 3] {
    let (sp, cp) = lat_deg.to_radians().sin_cos();
    let (sl, cl) = lon_deg.to_radians().sin_cos();
    [cp * cl, cp * sl, sp]
}

/// Wraps an angle into [0°, 360°).
pub fn wrap_deg(a: f64) -> f64 {
    let w = a.rem_euclid(360.0);
    if w >= 360.0 { 0.0 } else { w }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GridDesc;

    /// SplitMix64, for property tests without a dependency.
    struct Rng(u64);
    impl Rng {
        fn f(&mut self) -> f64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    const CS: [u32; 3] = [4_096, 16_384, 32_768];

    #[test]
    fn fixed_points() {
        for c in CS {
            let p = Projection::new(c);
            let q = f64::from(c) / 4.0;
            assert_eq!(p.pole_z(), q);
            assert_eq!(p.lat_deg(0.0), 0.0);
            assert_eq!(p.lat_deg(-q), 90.0, "north is -z");
            assert_eq!(p.lat_deg(q), -90.0);
            assert_eq!(p.lat_deg(-2.0 * q), 90.0, "latitude clamps past the pole");
            assert_eq!(p.z_of_lat(45.0), -q / 2.0);
            assert_eq!(
                p.lon_deg(-f64::from(c) / 2.0),
                0.0,
                "the seam is longitude 0"
            );
            assert_eq!(p.lon_deg(0.0), 180.0);
            assert_eq!(p.grid_origin_z(), -(i64::from(c) / 4));
            // True scale on the equator both ways.
            let ns = p.great_circle_blocks((0.0, 0.0), (0.0, 1.0));
            let ew = p.great_circle_blocks((0.0, 0.0), (1.0, 0.0));
            assert!(
                (ns - 1.0).abs() < 1e-6 && (ew - 1.0).abs() < 1e-6,
                "{ns} {ew}"
            );
        }
    }

    #[test]
    fn prop_block_lat_lon_round_trip() {
        let mut g = Rng(0xB10C);
        for c in CS {
            let p = Projection::new(c);
            let (h, q) = (f64::from(c) / 2.0, p.pole_z());
            for _ in 0..20_000 {
                let x = -h + g.f() * f64::from(c);
                let z = -q + g.f() * 2.0 * q;
                let (lat, lon) = p.lat_lon(x, z);
                let (x2, z2) = p.block_of(lat, lon);
                assert!(p.wrap_x(x2 - x).abs() < 1e-9, "x {x} -> {x2}");
                assert!((z2 - z).abs() < 1e-9, "z {z} -> {z2}");
                let v = p.unit(x, z);
                let want = unit_of_lat_lon(lat, lon);
                assert_eq!(v, want);
                assert!(((v[0] * v[0] + v[1] * v[1] + v[2] * v[2]) - 1.0).abs() < 1e-15);
            }
        }
    }

    #[test]
    fn prop_raster_row_round_trip() {
        // Block z ↔ grid row ↔ latitude.
        let mut g = Rng(0x2A57);
        for c in CS {
            for r in [2u32, 8, 16] {
                let grid = GridDesc::for_planet("L", c, r).unwrap();
                let p = grid.projection();
                for _ in 0..5_000 {
                    let z = -p.pole_z() + g.f() * 2.0 * p.pole_z();
                    let v = grid.row_of_z(z);
                    assert!((grid.z_of_row(v) - z).abs() < 1e-9);
                    let j = v.floor() as u32;
                    let zj = grid.block_z(i64::from(j)) as f64;
                    assert_eq!(grid.lat_of_row(j), p.lat_deg(zj));
                }
            }
        }
    }

    #[test]
    fn the_cells_sum_to_the_sphere() {
        for c in CS {
            for r in [2u32, 8, 16] {
                let grid = GridDesc::for_planet("L", c, r).unwrap();
                let total: f64 = (0..grid.h)
                    .map(|j| grid.cell_area_km2(j, 300.0) * f64::from(grid.w))
                    .sum();
                let r_km = f64::from(c) * 0.3 / std::f64::consts::TAU;
                let sphere = 2.0 * std::f64::consts::TAU * r_km * r_km;
                assert!((total / sphere - 1.0).abs() < 0.01, "{total} vs {sphere}");
                // The band weights integrate cos φ by the midpoint rule: to 2/π of the rows.
                let w: f64 = grid.row_weights().iter().sum();
                let want = f64::from(grid.h) * 2.0 / std::f64::consts::PI;
                assert!((w / want - 1.0).abs() < 1e-3, "{w} vs {want}");
            }
        }
    }

    #[test]
    fn the_rasters_metric_is_the_spheres() {
        // A short step on the raster, measured with the projection's local metric (east–west
        // scale_ew, north–south scale_ns blocks of planet per block), against the great circle.
        let mut g = Rng(0x6EA7);
        for c in CS {
            let p = Projection::new(c);
            for _ in 0..20_000 {
                let lat = -75.0 + 150.0 * g.f();
                let (x, z) = p.block_of(lat, 360.0 * g.f());
                let (dx, dz) = (8.0 * (g.f() - 0.5), 8.0 * (g.f() - 0.5));
                let zm = z + dz / 2.0;
                let metric = (dx * p.scale_ew(zm)).hypot(dz * p.scale_ns(zm));
                let gc = p.great_circle_blocks((x, z), (x + dx, z + dz));
                assert!(
                    (metric - gc).abs() <= 3e-3 * gc + 1e-6,
                    "C {c} lat {lat:.1}: metric {metric} gc {gc}"
                );
            }
        }
    }

    #[test]
    fn the_seam_is_continuous() {
        for c in CS {
            let p = Projection::new(c);
            let h = f64::from(c) / 2.0;
            for lat in [0.0, 30.0, -60.0, 79.0] {
                let z = p.z_of_lat(lat);
                let one = p.great_circle_blocks((h - 1.0, z), (-h, z));
                let inside = p.great_circle_blocks((-1.0, z), (0.0, z));
                assert!((one - inside).abs() < 1e-9, "lat {lat}: {one} {inside}");
                assert!(p.great_circle_blocks((h, z), (-h, z)) < 1e-6);
            }
        }
    }

    #[test]
    fn the_neighbour_rule_wraps_columns_and_stops_at_the_rows() {
        assert_eq!(neighbour(8, 4, true, (7, 1), (1, 0)), Some((0, 1)));
        assert_eq!(neighbour(8, 4, true, (0, 1), (-1, -1)), Some((7, 0)));
        assert_eq!(neighbour(8, 4, false, (7, 1), (1, 0)), None);
        assert_eq!(neighbour(8, 4, true, (3, 0), (0, -1)), None);
        assert_eq!(neighbour(8, 4, true, (3, 3), (1, 1)), None);
        assert_eq!(wrap_col(-1, 8), 7);
        assert_eq!(clamp_row(9, 4), 3);
        let grid = GridDesc::for_planet("L", 4_096, 16).unwrap();
        assert_eq!(grid.wrap_i(-1), grid.w - 1);
        assert_eq!(grid.clamp_j(i64::from(grid.h) + 5), grid.h - 1);
    }

    #[test]
    fn the_extent_metric_and_frames_agree() {
        for c in CS {
            let p = Projection::new(c);
            let (ew, ns) = p.extent();
            assert_eq!(ew, f64::from(c));
            assert_eq!(ns, 2.0 * p.pole_z());
            assert!(p.on_planet(0.0, p.pole_z()) && !p.on_planet(0.0, p.pole_z() + 1.0));
            assert!(!p.inside_poles(-p.pole_z()) && p.inside_poles(0.0));
            assert_eq!(p.clamp_z(1e9), p.pole_z());
            // The grids and the spatial indexes share the planet's extent.
            let grid = GridDesc::for_planet("L", c, 16).unwrap();
            assert_eq!(
                p.index_origin_blocks(),
                (grid.origin_block_x, grid.origin_block_z)
            );
            assert_eq!(p.index_origin(), (-ew / 2.0, -p.pole_z()));
            assert_eq!(p.index_span(), (ew, ns));
            assert_eq!(
                p.index_span_blocks(),
                (
                    i64::from(grid.w) * i64::from(grid.r),
                    i64::from(grid.h) * i64::from(grid.r)
                )
            );
            // The metric: area scale is √det g, cos φ here; north is −z.
            for lat in [0.0, 30.0, 60.0, 89.0] {
                let z = p.z_of_lat(lat);
                let (gxx, gxz, gzz) = p.metric(0.0, z);
                assert!((p.area_scale(z) - (gxx * gzz - gxz * gxz).sqrt()).abs() < 1e-15);
                assert!((p.area_scale(z) - lat.to_radians().cos()).abs() < 1e-12);
            }
            assert_eq!(p.north(0.0, 0.0), (0.0, -1.0));
            // Bands: the middle of band j is half a row past row j.
            assert_eq!(Projection::band_lat_deg(0, grid.h), Projection::picture_lat_deg(0.5, grid.h));
            assert_eq!(grid.band_lat_deg(3), Projection::band_lat_deg(3, grid.h));
            assert!(
                (grid.band_weight(3) - grid.band_lat_deg(3).to_radians().cos()).abs() < 1e-12
            );
        }
        assert_eq!(Projection::picture_lon_deg(256.0, 1024), 90.0);
        assert_eq!(Projection::picture_x_of_lon(450.0, 1024), 256.0);
    }

    #[test]
    fn pictures_span_the_grid() {
        let p = Projection::new(16_384);
        assert_eq!(Projection::picture_height(1024), 512);
        assert_eq!(p.picture_z(0.0, 512), -4096.0);
        assert_eq!(p.picture_z(256.0, 512), 0.0);
        for y in [0.0, 17.5, 200.0] {
            assert!((p.picture_y(p.picture_z(y, 512), 512) - y).abs() < 1e-9);
            let lat = p.lat_deg(p.picture_z(y, 512));
            assert!((Projection::picture_lat_deg(y, 512) - lat).abs() < 1e-12);
        }
        let grid = GridDesc::for_planet("L1", 16_384, 16).unwrap();
        assert_eq!(grid.picture_cell(512.0, 256.0, 1024, 512), (512, 256));
        assert_eq!(grid.picture_cell(3.0, 7.0, 512, 256), (6, 14));
    }
}

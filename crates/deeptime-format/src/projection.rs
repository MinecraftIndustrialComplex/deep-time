// SPDX-License-Identifier: LGPL-3.0-or-later
//! The map projection between block space and the sphere: the one place every conversion between
//! block coordinates, raster rows and latitude/longitude goes through
//! (`docs/design/equal-area-projection.md` §2; the Java twin is `worlddata/Projection.java`, the
//! pages' is `window.DTProj` in `deeptime-viewer/assets/projection.js`).
//!
//! The projection is **Lambert's cylindrical equal-area projection**, true on the equator (Ben,
//! decisions log 2026-10-02: "Lambert for now"): block x is linear in longitude and block z in
//! sin φ, so every block covers the same area of the planet. It replaced the equirectangular
//! layout (z linear in φ) the module was written with.
//!
//! # Conventions
//! - A planet of circumference C blocks has radius **R = C / 2π** blocks ([`Projection::radius`]).
//! - **Longitude** λ = 360°·(x + C/2)/C in [0°, 360°): x = −C/2 (the seam) is 0°, x wraps with
//!   period C.
//! - **Latitude** φ = asin(−z / R), north positive; north is −z, as in Minecraft. The poles are at
//!   z = ∓R ([`Projection::pole_z`]: 2,607.6 blocks at C = 16,384) and φ clamps to ±90° beyond.
//! - **Unit vector** (cos φ cos λ, cos φ sin λ, sin φ): +Z the north pole, +X the seam meridian.
//! - **Metric**: east–west a block is cos φ blocks of planet, north–south 1 / cos φ
//!   ([`Projection::scale_ew`], [`Projection::scale_ns`], [`Projection::metric`]); their product,
//!   the area a block stands for, is 1 everywhere ([`Projection::area_scale`]). North is −z
//!   everywhere ([`Projection::north`]). Shapes are true only on the equator.
//! - **Grids** (`GridDesc`) are square lattices in block space: W = C/r columns from the seam, and
//!   rows from z = −Zo to Zo, Zo the smallest multiple of the planet's quantum (64 blocks on every
//!   real planet) at or past the pole ([`Projection::grid_half_extent`]); so the grids of a planet
//!   share their origin and nest (L3 row 8j is L1 row j), and their rows are uniform in sin φ:
//!   every cell inside the sphere has the same area. The rows past a pole hold no planet.
//! - **Map pictures** of the whole planet run pole to pole: x linear in longitude from the seam,
//!   y linear in z (so in sin φ) from the north pole; aspect π : 1 ([`Projection::picture_height`]).
//! - **Topology**: a raster wraps in x and stops at its first and last rows
//!   ([`wrap_col`], [`clamp_row`], [`neighbour`]); block x wraps with period C and the planet
//!   ends at the poles ([`Projection::on_planet`]).
//!
//! The spatial indexes of the world data (the river tiles, the structure buckets, the site index,
//! the playability regions) are frames for lookups, not geography: lattices of their own cells
//! over the grids' extent, from [`Projection::index_origin`] over [`Projection::index_span`]
//! blocks.
//!
//! The projection is a game choice (ledger: SCL-049); that it keeps areas is a tested property of
//! the formulas (ledger: SCL-052).

use std::f64::consts::{PI, TAU};

/// The largest quantum of the grids' z extent, blocks ([`Projection::quantum`]).
pub const QUANTUM_MAX: u32 = 64;

/// Block space ↔ sphere for one planet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projection {
    c: f64,
    /// R = C / 2π, blocks.
    r: f64,
}

impl Projection {
    /// The projection of a planet `circumference` blocks round.
    pub fn new(circumference: u32) -> Projection {
        Projection::of(f64::from(circumference))
    }

    /// [`Projection::new`] for a circumference held as a float.
    pub fn of(circumference: f64) -> Projection {
        Projection {
            c: circumference,
            r: circumference / TAU,
        }
    }

    /// C, blocks.
    pub fn circumference(&self) -> f64 {
        self.c
    }

    /// The planet's radius R = C / 2π, blocks (the sphere the blocks stand for, at true scale on
    /// the equator).
    pub fn radius(&self) -> f64 {
        self.r
    }

    /// |z| of the poles, blocks: R. The planet's z range is [−pole_z, pole_z].
    pub fn pole_z(&self) -> f64 {
        self.r
    }

    /// The planet's extent in block space, blocks: (east–west, north–south) = (C, 2R).
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

    /// sin φ of block z, clamped to [−1, 1] (north is −z).
    pub fn sin_lat(&self, z: f64) -> f64 {
        (-z / self.r).clamp(-1.0, 1.0)
    }

    /// Latitude of block z, degrees in [−90, 90] (north is −z; clamped beyond the poles).
    pub fn lat_deg(&self, z: f64) -> f64 {
        self.sin_lat(z).asin().to_degrees()
    }

    /// Block z of a latitude (clamped to ±90°), in [−R, R].
    pub fn z_of_lat(&self, lat_deg: f64) -> f64 {
        -self.r * lat_deg.clamp(-90.0, 90.0).to_radians().sin()
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
        let (sp, cp) = (self.sin_lat(z), self.cos_lat(z));
        let (sl, cl) = self.lon_deg(x).to_radians().sin_cos();
        [cp * cl, cp * sl, sp]
    }

    /// cos φ of block z (0 at and beyond the poles).
    pub fn cos_lat(&self, z: f64) -> f64 {
        let s = self.sin_lat(z);
        (1.0 - s * s).max(0.0).sqrt()
    }

    /// Blocks of planet per block east–west at block z: cos φ (the metric's x scale; 0 at the
    /// poles).
    pub fn scale_ew(&self, z: f64) -> f64 {
        self.cos_lat(z)
    }

    /// Blocks of planet per block north–south at block z: 1 / cos φ (infinite at the poles).
    pub fn scale_ns(&self, z: f64) -> f64 {
        1.0 / self.cos_lat(z)
    }

    /// The metric of block space at block (x, z): (g_xx, g_xz, g_zz), blocks of planet per block,
    /// squared. Diagonal for a cylindrical projection: (cos² φ, 0, 1 / cos² φ).
    pub fn metric(&self, _x: f64, z: f64) -> (f64, f64, f64) {
        let c = self.cos_lat(z);
        (c * c, 0.0, 1.0 / (c * c))
    }

    /// Area of planet per block of block space (√det g): 1 everywhere on the planet (the
    /// projection keeps areas), 0 past the poles.
    pub fn area_scale(&self, z: f64) -> f64 {
        if self.on_planet(0.0, z) { 1.0 } else { 0.0 }
    }

    /// The share of the z band [z_lo, z_hi] that lies on the planet ([−R, R]): 1 inside, less on a
    /// band reaching past a pole. With [`Projection::area_scale`] 1, it is the band's area weight.
    pub fn band_share(&self, z_lo: f64, z_hi: f64) -> f64 {
        let len = z_hi - z_lo;
        if len <= 0.0 {
            return 0.0;
        }
        let on = (z_hi.min(self.r) - z_lo.max(-self.r)).max(0.0);
        (on / len).clamp(0.0, 1.0)
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

    /// The quantum the grids' z extent is a multiple of: the largest power of two up to
    /// [`QUANTUM_MAX`] that divides C/2 and is at most R/2 (64 for every real planet).
    pub fn quantum(&self) -> u32 {
        let half = (self.c / 2.0).round() as u64;
        let mut q = QUANTUM_MAX;
        while q > 1 && (!half.is_multiple_of(u64::from(q)) || f64::from(q) > self.r / 2.0) {
            q /= 2;
        }
        q
    }

    /// The half-extent Zo of the planet's grids, blocks: the smallest multiple of the quantum at
    /// or past the pole. Every grid of a planet shares it (whatever its `r`), so the grids' rows
    /// nest (L3 row 8j is L1 row j).
    pub fn grid_half_extent(&self, _r: u32) -> i64 {
        let q = f64::from(self.quantum());
        ((self.r / q).ceil() * q) as i64
    }

    /// Block z of the first row of a grid of `r` blocks per cell: −Zo, at or just past the north
    /// pole.
    pub fn grid_origin_z(&self, r: u32) -> i64 {
        -self.grid_half_extent(r)
    }

    /// Rows of a grid of `r` blocks per cell: 2·Zo / r (rounded up when r does not divide it,
    /// which takes an r coarser than twice the quantum, 128 blocks on a real planet).
    pub fn grid_rows(&self, r: u32) -> u32 {
        let r = i64::from(r.max(1));
        ((2 * self.grid_half_extent(r as u32) + r - 1) / r) as u32
    }

    /// Block (x, z) of the first cell of the world data's spatial indexes (river tiles, structure
    /// buckets, site index, playability regions): (−C/2, −Zo), the seam and the grids' first row.
    pub fn index_origin(&self) -> (f64, f64) {
        let (x, z) = self.index_origin_blocks();
        (x as f64, z as f64)
    }

    /// Blocks the spatial indexes span: (east–west, north–south) = (C, 2·Zo).
    pub fn index_span(&self) -> (f64, f64) {
        let (x, z) = self.index_span_blocks();
        (x as f64, z as f64)
    }

    /// [`Projection::index_origin`] in whole blocks.
    pub fn index_origin_blocks(&self) -> (i64, i64) {
        let c = self.c as i64;
        (-(c / 2), -self.grid_half_extent(1))
    }

    /// [`Projection::index_span`] in whole blocks.
    pub fn index_span_blocks(&self) -> (i64, i64) {
        (self.c as i64, 2 * self.grid_half_extent(1))
    }

    /// Block z of the north edge of the workbench's tile pyramid: −C/4. The pyramid is a 2 : 1
    /// frame of power-of-two tiles centred on the equator, wider than the planet (pole to pole is
    /// C/π); its pixels past the poles hold no planet.
    pub fn tile_frame_z(&self) -> f64 {
        -self.c / 4.0
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

    /// Block z at row coordinate `y` of a map picture `h` rows high (y = 0 the north pole, y = h
    /// the south pole; pixel `py` is centred on y = py, as keyframes are).
    pub fn picture_z(&self, y: f64, h: u32) -> f64 {
        -self.r + 2.0 * self.r * y / f64::from(h.max(1))
    }

    /// The row coordinate of block z in a map picture `h` rows high.
    pub fn picture_y(&self, z: f64, h: u32) -> f64 {
        (z + self.r) / (2.0 * self.r) * f64::from(h.max(1))
    }

    /// The latitude, degrees, at row coordinate `y` of a map picture `h` rows high: rows are
    /// uniform in sin φ.
    pub fn picture_lat_deg(y: f64, h: u32) -> f64 {
        (1.0 - 2.0 * y / f64::from(h.max(1)))
            .clamp(-1.0, 1.0)
            .asin()
            .to_degrees()
    }

    /// The row coordinate of latitude `lat_deg` in a map picture `h` rows high.
    pub fn picture_y_of_lat(lat_deg: f64, h: u32) -> f64 {
        (1.0 - lat_deg.clamp(-90.0, 90.0).to_radians().sin()) / 2.0 * f64::from(h)
    }

    /// The true size, metres (east–west, north–south), of a pixel at row coordinate `y` of a
    /// `w × h` map picture of a sphere of radius `radius_m`: 2πR·cos φ / w wide and
    /// 2R / (h·cos φ) high, cos φ held at `min_cos` or more near the poles.
    pub fn picture_pixel_m(y: f64, w: u32, h: u32, radius_m: f64, min_cos: f64) -> (f64, f64) {
        let s = (1.0 - 2.0 * y / f64::from(h.max(1))).clamp(-1.0, 1.0);
        let cos = (1.0 - s * s).sqrt().max(min_cos);
        (
            TAU * radius_m * cos / f64::from(w.max(1)),
            2.0 * radius_m / (f64::from(h.max(1)) * cos),
        )
    }

    /// The height of a map picture `w` pixels wide at the projection's aspect (π : 1).
    pub fn picture_height(w: u32) -> u32 {
        ((f64::from(w) / PI).round() as u32).max(1)
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
            let r = f64::from(c) / TAU;
            assert_eq!(p.radius(), r);
            assert_eq!(p.pole_z(), r);
            assert_eq!(p.lat_deg(0.0), 0.0);
            assert_eq!(p.lat_deg(-r), 90.0, "north is -z");
            assert_eq!(p.lat_deg(r), -90.0);
            assert_eq!(p.lat_deg(-2.0 * r), 90.0, "latitude clamps past the pole");
            assert!((p.z_of_lat(90.0) + r).abs() < 1e-9);
            assert!((p.z_of_lat(30.0) + r / 2.0).abs() < 1e-9, "sin 30° = 1/2");
            assert_eq!(
                p.lon_deg(-f64::from(c) / 2.0),
                0.0,
                "the seam is longitude 0"
            );
            assert_eq!(p.lon_deg(0.0), 180.0);
            // True scale on the equator: one block of x and of z is one block of planet there.
            let ns = p.great_circle_blocks((0.0, 0.0), (0.0, 1.0));
            let ew = p.great_circle_blocks((0.0, 0.0), (1.0, 0.0));
            assert!(
                (ns - 1.0).abs() < 1e-6 && (ew - 1.0).abs() < 1e-6,
                "{ns} {ew}"
            );
        }
        // The decision's 16k numbers: 5,215 blocks pole to pole, 60° at z = −2,258, the grids to
        // ±2,624.
        let p = Projection::new(16_384);
        assert!((2.0 * p.radius() - 5_215.19).abs() < 0.01);
        assert!((p.z_of_lat(60.0) + 2_258.2).abs() < 0.1);
        assert_eq!(p.grid_half_extent(16), 2_624);
        assert_eq!(p.grid_half_extent(2), 2_624);
        assert_eq!(p.grid_rows(16), 328);
        assert_eq!(Projection::new(32_768).grid_half_extent(32), 5_248);
    }

    #[test]
    fn prop_block_lat_lon_round_trip() {
        let mut g = Rng(0xB10C);
        for c in CS {
            let p = Projection::new(c);
            let (h, r) = (f64::from(c) / 2.0, p.radius());
            for _ in 0..20_000 {
                let x = -h + g.f() * f64::from(c);
                let z = -r + g.f() * 2.0 * r;
                let (lat, lon) = p.lat_lon(x, z);
                let (x2, z2) = p.block_of(lat, lon);
                assert!(p.wrap_x(x2 - x).abs() < 1e-9, "x {x} -> {x2}");
                assert!((z2 - z).abs() < 1e-8, "z {z} -> {z2}");
                // And back from (lat, lon) through the blocks.
                let lat = -90.0 + 180.0 * g.f();
                let lon = 360.0 * g.f();
                let (x, z) = p.block_of(lat, lon);
                let (lat2, lon2) = p.lat_lon(x, z);
                assert!((lat2 - lat).abs() < 1e-7, "lat {lat} -> {lat2}");
                let d = (lon2 - lon).rem_euclid(360.0);
                assert!(d.min(360.0 - d) < 1e-9);
                // The unit vector is the one of (lat, lon).
                let v = p.unit(x, z);
                let want = unit_of_lat_lon(lat, lon);
                assert!(
                    (0..3).all(|k| (v[k] - want[k]).abs() < 1e-7),
                    "{v:?} {want:?}"
                );
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
                    let z = -p.radius() + g.f() * 2.0 * p.radius();
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
    fn every_cell_has_the_same_area_and_they_sum_to_the_sphere() {
        for c in CS {
            for r in [2u32, 4, 8, 16, 32] {
                let grid = GridDesc::for_planet("L", c, r).unwrap();
                let p = grid.projection();
                let m = 300.0;
                // The true area of each row's cells on a sphere of R·m metres: 2πR²·Δ(sin φ)
                // over the row's band, divided among its W cells (exact, not the helper's).
                let rm = p.radius() * m;
                let mut total = 0.0;
                let mut interior = Vec::new();
                for j in 0..grid.h {
                    let z = grid.block_z(i64::from(j)) as f64;
                    let lo = if j == 0 {
                        f64::NEG_INFINITY
                    } else {
                        z - f64::from(r) / 2.0
                    };
                    let hi = if j + 1 == grid.h {
                        f64::INFINITY
                    } else {
                        z + f64::from(r) / 2.0
                    };
                    let exact = TAU * rm * rm * (p.sin_lat(lo) - p.sin_lat(hi)) / f64::from(grid.w);
                    let helper = grid.cell_area_km2(j, m) * 1e6;
                    assert!(
                        (helper - exact).abs() <= 1e-9 * exact.max(1.0),
                        "row {j}: {helper} {exact}"
                    );
                    total += helper * f64::from(grid.w);
                    if grid.area_share(j) == 1.0 {
                        interior.push(exact);
                    }
                }
                let sphere = 2.0 * TAU * rm * rm;
                assert!(
                    (total / sphere - 1.0).abs() < 1e-12,
                    "C {c} r {r}: {total} vs {sphere}"
                );
                // Any two cells inside the sphere are equal (within rounding), (r·m)² each.
                let (lo, hi) = interior
                    .iter()
                    .fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
                assert!(hi / lo - 1.0 < 1e-9, "C {c} r {r}: {lo} .. {hi}");
                assert!((lo / (f64::from(r) * m).powi(2) - 1.0).abs() < 1e-9);
                // The row weights are the same shares.
                let w: f64 = grid.row_weights().iter().sum();
                assert!((w - 2.0 * p.radius() / f64::from(r)).abs() < 1e-9);
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
                let (gxx, gxz, gzz) = p.metric(x, zm);
                assert!(((gxx * gzz - gxz * gxz).sqrt() - p.area_scale(zm)).abs() < 1e-9);
            }
        }
        // The equirectangular metric (north–south unscaled) is not the sphere's here.
        let p = Projection::new(16_384);
        let z = p.z_of_lat(60.0);
        let gc = p.great_circle_blocks((0.0, z), (0.0, z + 4.0));
        assert!(
            (gc - 8.0).abs() < 0.05,
            "4 blocks of z at 60° are 8 of planet: {gc}"
        );
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
    fn pictures_span_pole_to_pole() {
        let p = Projection::new(16_384);
        assert_eq!(Projection::picture_height(1024), 326);
        assert_eq!(p.lat_deg(p.picture_z(0.0, 326)), 90.0);
        assert_eq!(p.lat_deg(p.picture_z(326.0, 326)), -90.0);
        assert!(p.lat_deg(p.picture_z(163.0, 326)).abs() < 1e-12);
        for y in [0.0, 17.5, 200.0] {
            assert!((p.picture_y(p.picture_z(y, 326), 326) - y).abs() < 1e-9);
            let lat = p.lat_deg(p.picture_z(y, 326));
            assert!((Projection::picture_lat_deg(y, 326) - lat).abs() < 1e-9);
            assert!((Projection::picture_y_of_lat(lat, 326) - y).abs() < 1e-9);
        }
        // Equal rows of a picture are equal bands of sin φ.
        let s = |y: f64| p.sin_lat(p.picture_z(y, 326));
        assert!(((s(10.0) - s(11.0)) - (s(300.0) - s(301.0))).abs() < 1e-12);
        assert_eq!(Projection::picture_lon_deg(256.0, 1024), 90.0);
        assert_eq!(Projection::picture_x_of_lon(450.0, 1024), 256.0);
    }

    #[test]
    fn quantum_and_extent() {
        assert_eq!(Projection::new(16_384).quantum(), 64);
        assert_eq!(Projection::new(32_768).quantum(), 64);
        // A tiny test planet: R = 10.2, so the quantum shrinks to 4.
        assert_eq!(Projection::new(64).quantum(), 4);
        let p = Projection::new(64);
        assert_eq!(p.grid_half_extent(4), 12);
        assert_eq!(p.grid_half_extent(2), 12);
        for c in CS {
            let p = Projection::new(c);
            for r in [1u32, 2, 4, 8, 16, 32, 64] {
                let zo = p.grid_half_extent(r);
                assert!(zo as f64 >= p.radius() && zo % i64::from(r) == 0);
                assert!((zo as f64) < p.radius() + 64.0f64.max(f64::from(r)));
            }
            let (ew, ns) = p.extent();
            assert_eq!(ew, f64::from(c));
            assert_eq!(ns, 2.0 * p.radius());
            assert!(p.on_planet(0.0, p.pole_z()) && !p.on_planet(0.0, p.pole_z() + 1.0));
            assert!(!p.inside_poles(-p.pole_z()) && p.inside_poles(0.0));
            assert_eq!(p.clamp_z(1e9), p.pole_z());
            // The grids and the spatial indexes share their origin, which covers the planet.
            let grid = GridDesc::for_planet("L", c, 16).unwrap();
            assert_eq!(
                p.index_origin_blocks(),
                (grid.origin_block_x, grid.origin_block_z)
            );
            assert_eq!(
                p.index_span_blocks(),
                (
                    i64::from(grid.w) * i64::from(grid.r),
                    i64::from(grid.h) * i64::from(grid.r)
                )
            );
            assert!(p.index_span().1 >= ns);
            assert_eq!(p.north(0.0, 0.0), (0.0, -1.0));
            assert_eq!(p.band_share(-1e9, 1e9) * 2e9, 2.0 * p.radius());
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
}

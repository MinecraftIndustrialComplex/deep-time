// SPDX-License-Identifier: LGPL-3.0-or-later
//! The planet's own metric about a point of the Lambert block frame: what draws a feature round
//! on the planet (docs/science/round-features.md; ledger SCL-230 to SCL-236).
//!
//! The simulation's grids are Lambert's cylindrical equal-area projection ([`BlockFrame`]): a
//! block east–west is cos φ blocks of the planet and a block north–south 1/cos φ of them. A shape
//! drawn round in Lambert blocks (its offsets in blocks times `m_per_block_h`) is therefore an
//! ellipse on the planet, long north–south by 1/cos² φ (1.33 at 30°, 3.04 at 55°, 8.55 at 70°),
//! which the conformal cube world draws faithfully (the equal-area Petroff–Guyou world adds its own
//! shape stretch to it).
//!
//! [`Around`] measures on the sphere instead. A block's offset from the centre is its position in
//! the azimuthal equidistant frame at the centre (Snyder 1987, *Map Projections: A Working
//! Manual*, USGS PP 1395, ch. 25), metres east and south: its length is the great-circle distance
//! and its direction the bearing (as `surface::cube::footprint::offset` measures on the cube). A
//! shape drawn as a function of that offset is round on the planet, so round in a conformal world
//! (the cube) too. Lambert is equal-area, so a shape's area, its volume and the rock it moves are the same in
//! either metric; only its outline changes.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use super::BlockFrame;

/// The cosine of the latitude below which the frame's east–west scale is held (about 0.2 arc
/// seconds from a pole), so that an ellipse's width in Lambert blocks stays finite at a pole;
/// [`Around::ellipse_blocks`] also holds it where the ellipse would be wider than half the
/// circumference. game-choice: a numerical guard (no physics), Claude 2026-10-23.
// ledger: SCL-234
const POLE_COS: f64 = 1e-6;

/// The planet's metric about block (x, z) of a Lambert frame.
#[derive(Clone, Copy, Debug)]
pub struct Around {
    frame: BlockFrame,
    /// The planet's radius R = C / 2π, blocks, and metres a radian, R · `m_per_block_h`.
    r: f64,
    m_rad: f64,
    mh: f64,
    x: f64,
    z: f64,
    sin_lat: f64,
    cos_lat: f64,
}

impl Around {
    /// The metric about block (x, z) of `frame`, whose blocks are `m_per_block_h` metres.
    pub fn new(frame: BlockFrame, m_per_block_h: f64, x: f64, z: f64) -> Around {
        let p = frame.projection();
        let r = p.radius();
        Around {
            frame,
            r,
            m_rad: r * m_per_block_h,
            mh: m_per_block_h,
            x,
            z,
            sin_lat: p.sin_lat(z),
            cos_lat: p.cos_lat(z),
        }
    }

    /// [`Around::new`] on the frame of a planet `circumference` blocks round.
    pub fn of(circumference: u32, m_per_block_h: f64, x: f64, z: f64) -> Around {
        Around::new(BlockFrame::new(circumference), m_per_block_h, x, z)
    }

    /// The centre, block (x, z).
    pub fn centre(&self) -> (f64, f64) {
        (self.x, self.z)
    }

    /// The planet's radius, m.
    pub fn radius_m(&self) -> f64 {
        self.m_rad
    }

    /// sin φ and cos φ of block row z (Lambert: sin φ = −z / R, clamped at the poles).
    #[inline]
    pub fn row(&self, z: f64) -> (f64, f64) {
        let s = (-z / self.r).clamp(-1.0, 1.0);
        (s, (1.0 - s * s).max(0.0).sqrt())
    }

    /// sin Δλ and cos Δλ of block column x: its longitude less the centre's (wrapped).
    #[inline]
    pub fn column(&self, x: f64) -> (f64, f64) {
        (self.frame.wrap_x(x - self.x) / self.r).sin_cos()
    }

    /// The great-circle angle (radians) to the point of a [`row`](Self::row) and a
    /// [`column`](Self::column), with its offset's east and north parts before scaling (0, 0 at
    /// the centre and at its antipode).
    #[inline]
    fn arc(&self, row: (f64, f64), col: (f64, f64)) -> (f64, f64, f64, f64) {
        let ((s, c), (sd, cd)) = (row, col);
        let e = c * sd;
        let n = self.cos_lat * s - self.sin_lat * c * cd;
        let dot = self.sin_lat * s + self.cos_lat * c * cd;
        let h = e.hypot(n);
        (h.atan2(dot), e, n, h)
    }

    /// The great-circle distance (m) to the point of a [`row`](Self::row) and a
    /// [`column`](Self::column).
    #[inline]
    pub fn distance_of(&self, row: (f64, f64), col: (f64, f64)) -> f64 {
        self.m_rad * self.arc(row, col).0
    }

    /// The offset (m east, m south) of the point of a [`row`](Self::row) and a
    /// [`column`](Self::column): the great-circle distance along the bearing.
    #[inline]
    pub fn offset_of(&self, row: (f64, f64), col: (f64, f64)) -> (f64, f64) {
        self.offset_within(row, col, f64::INFINITY)
            .unwrap_or((0.0, 0.0))
    }

    /// [`offset_of`](Self::offset_of) when the point lies within `reach_m` of the centre.
    #[inline]
    pub fn offset_within(
        &self,
        row: (f64, f64),
        col: (f64, f64),
        reach_m: f64,
    ) -> Option<(f64, f64)> {
        let (theta, e, n, h) = self.arc(row, col);
        let d = self.m_rad * theta;
        if d > reach_m {
            return None;
        }
        if h == 0.0 {
            return Some((0.0, 0.0));
        }
        Some((d * e / h, -d * n / h))
    }

    /// The offset (m east, m south) of block (x, z) from the centre.
    #[inline]
    pub fn offset(&self, x: f64, z: f64) -> (f64, f64) {
        self.offset_of(self.row(z), self.column(x))
    }

    /// The great-circle distance from the centre to block (x, z), m.
    #[inline]
    pub fn distance_m(&self, x: f64, z: f64) -> f64 {
        self.distance_of(self.row(z), self.column(x))
    }

    /// The block `east_m` east and `south_m` south of the centre (the inverse of
    /// [`offset`](Self::offset)): x within half a circumference of the centre's x (not wrapped),
    /// z between the poles.
    pub fn block_at(&self, east_m: f64, south_m: f64) -> (f64, f64) {
        let d = east_m.hypot(south_m);
        if d == 0.0 {
            return (self.x, self.z);
        }
        let (st, ct) = (d / self.m_rad).sin_cos();
        let (ue, un) = (east_m / d, -south_m / d);
        // In the frame where the centre is (cos φ, 0, sin φ), east (0, 1, 0), north
        // (−sin φ, 0, cos φ).
        let px = ct * self.cos_lat - st * un * self.sin_lat;
        let py = st * ue;
        let pz = (ct * self.sin_lat + st * un * self.cos_lat).clamp(-1.0, 1.0);
        (self.x + self.r * py.atan2(px), -self.r * pz)
    }

    /// The Lambert box that holds every point within `reach_m` of the centre: its half-width in x
    /// (blocks; half the circumference when the cap holds a pole) and its z range (unbounded on
    /// the side of a pole the cap holds, so the rows past the poles come too).
    pub fn bounds(&self, reach_m: f64) -> (f64, f64, f64) {
        let alpha = (reach_m / self.m_rad).max(0.0);
        let lat = self.sin_lat.atan2(self.cos_lat);
        let (north, south) = (lat + alpha, lat - alpha);
        let half = 0.5 * self.frame.circumference();
        let z0 = if north >= FRAC_PI_2 {
            f64::NEG_INFINITY
        } else {
            -self.r * north.sin()
        };
        let z1 = if south <= -FRAC_PI_2 {
            f64::INFINITY
        } else {
            -self.r * south.sin()
        };
        let hw = if alpha >= PI || north >= FRAC_PI_2 || south <= -FRAC_PI_2 {
            half
        } else {
            (self.r * (alpha.sin() / self.cos_lat).min(1.0).asin()).min(half)
        };
        (hw, z0, z1)
    }

    /// An ellipse on the planet at the centre, semi-axes `a_m` and `c_m` (m), its first axis
    /// `yaw` radians from east toward south, as the ellipse the frame draws of it in Lambert
    /// blocks (the frame's linear map at the centre: a metre east is 1 / (m cos φ) blocks of x, a
    /// metre south cos φ / m of z): (first semi-axis, second semi-axis, yaw), blocks and radians
    /// from +x toward +z, the first axis the one nearest the image of the planet's first axis.
    /// Its area is the planet ellipse's (Lambert is equal-area). A body drawn with these axes is
    /// the planet's ellipse wherever the world's projection carries it. Within about its own size
    /// of a pole, where the ellipse would reach more than half the circumference east–west, the
    /// scale is held there (SCL-234): it keeps its area and spans half the circumference.
    pub fn ellipse_blocks(&self, a_m: f64, c_m: f64, yaw: f64) -> (f64, f64, f64) {
        let reach = a_m.abs().max(c_m.abs()) / (self.mh * 0.5 * self.frame.circumference());
        let cos = self.cos_lat.max(reach).max(POLE_COS); // ledger: SCL-234
        let (kx, kz) = (1.0 / (self.mh * cos), cos / self.mh);
        let (sy, cy) = yaw.sin_cos();
        conjugate_axes(
            (kx * cy * a_m, kz * sy * a_m),
            (-kx * sy * c_m, kz * cy * c_m),
        )
    }
}

/// The axes of the ellipse with conjugate semi-diameters `p` and `q` (the images of an ellipse's
/// two semi-axes under a linear map): (first semi-axis, second semi-axis, the first's yaw in
/// radians from +x toward +z, in [−π, π)), the first the one nearest `p`'s direction. The
/// eigenvalues of M Mᵀ for M = [p q] are the squared semi-axes and its eigenvectors their
/// directions, as `bake::project_cube::project_body_exact` finds a body's image.
pub fn conjugate_axes(p: (f64, f64), q: (f64, f64)) -> (f64, f64, f64) {
    let (s00, s01, s11) = (
        p.0 * p.0 + q.0 * q.0,
        p.0 * p.1 + q.0 * q.1,
        p.1 * p.1 + q.1 * q.1,
    );
    let half = 0.5 * (s00 + s11);
    let root = (0.25 * (s00 - s11).powi(2) + s01 * s01).sqrt();
    let major = (half + root).sqrt();
    // The product of the semi-axes is |det M|: the minor from it keeps its digits where the
    // ellipse is very long (half − root cancels to nothing near a pole).
    let det = (p.0 * q.1 - p.1 * q.0).abs();
    let minor = if major > 0.0 { det / major } else { 0.0 };
    let theta = 0.5 * (2.0 * s01).atan2(s00 - s11);
    let toward = p.1.atan2(p.0);
    let k = (0..4)
        .max_by(|&i, &j| {
            let f = |k: i32| (theta + f64::from(k) * FRAC_PI_2 - toward).cos();
            f(i).total_cmp(&f(j)).then(j.cmp(&i))
        })
        .unwrap_or(0);
    let (first, second) = if k % 2 == 0 {
        (major, minor)
    } else {
        (minor, major)
    };
    let yaw = (theta + f64::from(k) * FRAC_PI_2 + PI).rem_euclid(TAU) - PI;
    (first, second, yaw)
}

#[cfg(test)]
mod tests {
    use super::*;

    const C: u32 = 16_384;
    const MH: f64 = 300.0;

    fn at_lat(lat_deg: f64) -> Around {
        let f = BlockFrame::new(C);
        Around::new(f, MH, 1234.5, f.z_of_lat(lat_deg))
    }

    #[test]
    fn the_offset_is_the_great_circle_distance_along_the_bearing() {
        let f = BlockFrame::new(C);
        for lat in [0.0, 30.0, -55.0, 70.0, 85.0] {
            let a = at_lat(lat);
            let (cx, cz) = a.centre();
            let c = f.unit(cx, cz);
            for (x, z) in [(cx + 40.0, cz), (cx, cz - 25.0), (cx - 300.0, cz + 90.0)] {
                let (e, s) = a.offset(x, z);
                let want = MH * f.projection().radius() * c.angle_between(f.unit(x, z));
                assert!((e.hypot(s) - want).abs() < 1e-6 * want, "{lat}: {e} {s} {want}");
                assert!((a.distance_m(x, z) - want).abs() < 1e-6 * want);
            }
            // A step east is east, bent poleward by the parallel's curvature: the parallel leaves
            // the great circle east by e² tan φ / 2R. A step north (−z) is north.
            let (e, s) = a.offset(cx + 1.0, cz);
            let bend = -e * e * f64::to_radians(lat).tan() / (2.0 * a.radius_m());
            assert!(e > 0.0 && (s - bend).abs() < 1e-3 * bend.abs() + 1e-9, "{lat}: {e} {s} {bend}");
            if lat < 85.0 {
                let (e, s) = a.offset(cx, cz - 1.0);
                assert!(s < 0.0 && e.abs() < 1e-9, "{lat}: {e} {s}");
            }
        }
    }

    #[test]
    fn block_at_inverts_the_offset() {
        for lat in [0.0, 30.0, -55.0, 70.0] {
            let a = at_lat(lat);
            for (e, s) in [(5_000.0, 0.0), (0.0, -12_000.0), (-30_000.0, 40_000.0), (2.0, 3.0)] {
                let (x, z) = a.block_at(e, s);
                let (e2, s2) = a.offset(x, z);
                assert!((e2 - e).abs() < 1e-6 * e.hypot(s) + 1e-9, "{lat}: {e} → {e2}");
                assert!((s2 - s).abs() < 1e-6 * e.hypot(s) + 1e-9, "{lat}: {s} → {s2}");
            }
        }
    }

    /// The Lambert block extents of a circle of `r_m` drawn with the offset: (east–west,
    /// north–south) blocks.
    fn extents(a: &Around, r_m: f64) -> (f64, f64) {
        let (cx, cz) = a.centre();
        let mut ew = 0.0f64;
        let mut ns = 0.0f64;
        for k in 0..360 {
            let th = f64::from(k).to_radians();
            let (x, z) = a.block_at(r_m * th.cos(), r_m * th.sin());
            ew = ew.max((x - cx).abs());
            ns = ns.max((z - cz).abs());
        }
        (ew, ns)
    }

    #[test]
    fn a_circle_on_the_planet_is_wide_in_lambert_blocks_by_one_over_cos_squared() {
        // A Lambert circle is long north–south on the planet by 1 / cos² φ; the planet's circle is
        // the reverse in Lambert blocks: wider in x than in z by 1 / cos² φ.
        for (lat, want) in [(30.0f64, 1.0 / 0.75), (55.0, 3.04), (70.0, 8.55)] {
            let a = at_lat(lat);
            let (ew, ns) = extents(&a, 3_000.0);
            let ratio = ew / ns;
            assert!((ratio / want - 1.0).abs() < 0.01, "{lat}°: {ratio} vs {want}");
        }
    }

    #[test]
    fn the_bounds_hold_the_cap() {
        for lat in [0.0, 30.0, -55.0, 70.0, 88.0, -89.5] {
            let a = at_lat(lat);
            for r_m in [3_000.0, 60_000.0, 400_000.0] {
                let (hw, z0, z1) = a.bounds(r_m);
                let (cx, _) = a.centre();
                for k in 0..720 {
                    let th = f64::from(k) * 0.5f64.to_radians();
                    let (x, z) = a.block_at(r_m * th.cos(), r_m * th.sin());
                    let dx = BlockFrame::new(C).wrap_x(x - cx).abs();
                    assert!(dx <= hw + 1e-6, "{lat} {r_m}: {dx} > {hw}");
                    assert!(z >= z0 - 1e-6 && z <= z1 + 1e-6, "{lat} {r_m}: {z} not in {z0}..{z1}");
                }
            }
        }
        // A cap round a pole covers every column and runs past the pole's rows.
        let (hw, z0, _) = at_lat(88.0).bounds(400_000.0);
        assert_eq!(hw, 0.5 * f64::from(C));
        assert_eq!(z0, f64::NEG_INFINITY);
    }

    #[test]
    fn an_ellipse_at_a_pole_is_finite_and_keeps_its_area() {
        // SCL-234: at and next to a pole the ellipse spans at most half the circumference
        // east–west, keeps its area and its minor axis (no cancellation to zero).
        let f = BlockFrame::new(C);
        for z in [f.z_of_lat(89.99), f.z_of_lat(90.0), f.z_of_lat(-90.0) + 3.0, -f.projection().radius() - 40.0] {
            let a = Around::new(f, MH, 10.0, z);
            for (am, cm, yaw) in [(6_000.0, 4_000.0, 0.6), (1_500.0, 1_500.0, 0.0)] {
                let (p, q, y) = a.ellipse_blocks(am, cm, yaw);
                assert!(p.is_finite() && q.is_finite() && y.is_finite(), "{z}: {p} {q} {y}");
                assert!(p > 0.0 && q > 0.0, "{z}: {p} {q}");
                assert!(p.max(q) <= 0.5 * f64::from(C) * (1.0 + 1e-9), "{z}: {p} {q}");
                assert!((p * q * MH * MH / (am * cm) - 1.0).abs() < 1e-9, "{z}: area {p} {q}");
            }
        }
    }

    #[test]
    fn a_planet_ellipse_in_blocks_comes_back_round_and_keeps_its_area() {
        for lat in [0.0f64, 30.0, -55.0, 70.0] {
            let a = at_lat(lat);
            let cos = lat.to_radians().cos();
            // A circle: wide in x by 1 / cos, narrow in z by cos, not turned.
            let (p, q, yaw) = a.ellipse_blocks(6_000.0, 6_000.0, 0.0);
            assert!((p - 20.0 / cos).abs() < 1e-9 * p, "{lat}: {p}");
            assert!((q - 20.0 * cos).abs() < 1e-9, "{lat}: {q}");
            assert!(yaw.abs() < 1e-12, "{lat}: {yaw}");
            // A yawed ellipse: its Lambert image mapped back to metres on the planet is it again
            // (the same semi-axes and area).
            let (am, cm, ym) = (9_000.0, 4_000.0, 0.6);
            let (p, q, yaw) = a.ellipse_blocks(am, cm, ym);
            assert!((p * q * MH * MH - am * cm).abs() < 1e-6 * am * cm, "{lat}: area");
            let back = |t: f64| {
                let (u, v) = (p * t.cos(), q * t.sin());
                let (sy, cy) = yaw.sin_cos();
                let (bx, bz) = (cy * u - sy * v, sy * u + cy * v);
                (bx * MH * cos, bz * MH / cos)
            };
            let (mut lo, mut hi) = (f64::MAX, 0.0f64);
            for k in 0..3600 {
                let (e, s) = back(f64::from(k) * 0.1f64.to_radians());
                lo = lo.min(e.hypot(s));
                hi = hi.max(e.hypot(s));
            }
            assert!((hi - am).abs() < 1e-3 * am && (lo - cm).abs() < 1e-3 * cm, "{lat}: {lo} {hi}");
        }
    }
}

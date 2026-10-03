// SPDX-License-Identifier: LGPL-3.0-or-later
//! The Peirce quincuncial projection as a Minecraft world square (PROTOTYPE).
//!
//! **Layout.** The planet is the square |x| ≤ a/2, |z| ≤ a/2 of side `a` blocks: the north pole at the
//! centre (0, 0), the south pole at the four corners, the equator the diamond through the four edge
//! midpoints. Block x is east on the map, block z south (Minecraft's axes), so the map is drawn as
//! seen from above the north pole, not mirrored.
//!
//! **Construction (Peirce 1879, here as in Lee 1976).** With m = ½ (K = K(½) ≈ 1.8541) the block maps
//! to w = K + (2K/a)·(x − i z), a point of the square [0, 2K] × [−K, K]. The stereographic coordinate
//! of the planet point (from the south pole, ζ = tan(θ/2)·e^{iλ}, θ the colatitude) is
//! ζ = cn(w | ½)·e^{iλ₀}. λ₀ turns the planet about its axis: it is the free choice that decides where
//! the four singular points fall.
//!
//! **Seamless by construction.** cn(· | ½) has the periods 4K and 2K ± 2iK and is even, so it is
//! unchanged by the half-turn about each edge midpoint (w ↦ −w, 4K − w, 2K ± 2iK − w). The same
//! formula therefore covers the whole plane with the planet repeated in the p2 tiling, and each edge
//! of the square is glued to itself by the half-turn about its midpoint (`glue`). The four edge
//! midpoints are the projection's singular points: cn′ = −sn·dn vanishes there (sn(0) = sn(2K) = 0,
//! dn(K ± iK) = 0), so the map is conformal everywhere else and doubles angles at those four points.

use crate::complex::C;
use crate::elliptic::{M_HALF, arccn, complete_k, pole_distance_proxy, sncndn_c};

/// The four edges of the square, in the order used everywhere in this crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    /// x = +a/2.
    East,
    /// z = +a/2.
    South,
    /// x = −a/2.
    West,
    /// z = −a/2.
    North,
}

impl Edge {
    pub const ALL: [Edge; 4] = [Edge::East, Edge::South, Edge::West, Edge::North];

    pub fn name(self) -> &'static str {
        match self {
            Edge::East => "east",
            Edge::South => "south",
            Edge::West => "west",
            Edge::North => "north",
        }
    }
}

/// The Peirce square of one world.
#[derive(Clone, Copy, Debug)]
pub struct Peirce {
    side: f64,
    lon0_deg: f64,
    /// K(½).
    k: f64,
    /// w units per block: 2K / a.
    s: f64,
}

impl Peirce {
    /// A square of `side` blocks whose edge midpoints sit at longitudes λ₀ + {0°, 90°, 180°, 270°}
    /// (west, south, east, north edge in that order; see [`Peirce::singular_points`]).
    pub fn new(side: f64, lon0_deg: f64) -> Peirce {
        assert!(side > 0.0 && side.is_finite(), "side {side}");
        let k = complete_k(M_HALF);
        Peirce {
            side,
            lon0_deg,
            k,
            s: 2.0 * k / side,
        }
    }

    pub fn side(&self) -> f64 {
        self.side
    }

    pub fn half(&self) -> f64 {
        0.5 * self.side
    }

    pub fn lon0_deg(&self) -> f64 {
        self.lon0_deg
    }

    pub fn w_of_block(&self, x: f64, z: f64) -> C {
        C::new(self.k + self.s * x, -self.s * z)
    }

    pub fn block_of_w(&self, w: C) -> (f64, f64) {
        ((w.re - self.k) / self.s, -w.im / self.s)
    }

    /// The stereographic coordinate of block (x, z) (any point of the plane: the p2 tiling), or
    /// `None` at a south-pole image (a pole of cn).
    pub fn zeta(&self, x: f64, z: f64) -> Option<C> {
        let w = self.w_of_block(x, z);
        if pole_distance_proxy(w, M_HALF) < 1e-24 {
            return None;
        }
        let cn = sncndn_c(w, M_HALF).1;
        if !cn.is_finite() || cn.abs() > 1e150 {
            return None;
        }
        Some(cn * C::cis(self.lon0_deg.to_radians()))
    }

    /// (latitude, longitude) in degrees of block (x, z); longitude in (−180°, 180°]. Defined on the
    /// whole plane (outside the square it is the glued neighbour's planet point).
    pub fn lat_lon(&self, x: f64, z: f64) -> (f64, f64) {
        match self.zeta(x, z) {
            None => (-90.0, 0.0),
            Some(zeta) => {
                let theta = 2.0 * zeta.abs().atan();
                (90.0 - theta.to_degrees(), zeta.arg().to_degrees())
            }
        }
    }

    /// The unit vector (cos φ cos λ, cos φ sin λ, sin φ) of block (x, z), straight from ζ (inverse
    /// stereographic projection), without the round trip through angles.
    pub fn unit(&self, x: f64, z: f64) -> [f64; 3] {
        match self.zeta(x, z) {
            None => [0.0, 0.0, -1.0],
            Some(zeta) => {
                let r2 = zeta.norm_sqr();
                let d = 1.0 + r2;
                [2.0 * zeta.re / d, 2.0 * zeta.im / d, (1.0 - r2) / d]
            }
        }
    }

    /// The block of a planet point in the closed square (the inverse). On a glued edge the point has
    /// two blocks; either is returned (they are each other's half-turn image).
    pub fn block_of(&self, lat_deg: f64, lon_deg: f64) -> (f64, f64) {
        if lat_deg <= -90.0 + 1e-12 {
            return (self.half(), self.half());
        }
        let theta = (90.0 - lat_deg.min(90.0)).to_radians();
        let zeta = C::cis((lon_deg - self.lon0_deg).to_radians()).scale((0.5 * theta).tan());
        let w = self.w_of_zeta(zeta);
        let (x, z, _) = self.canonical_w(w);
        self.block_of_w(C::new(x, z))
    }

    /// w with cn(w | ½) = ζ, inside the closed square [0, 2K] × [−K, K].
    fn w_of_zeta(&self, zeta: C) -> C {
        // The 90° symmetry cn(K + i(w − K)) = i·cn(w) reduces ζ to the sector |arg ζ| ≤ 45°, the
        // triangle of the square that holds the west edge.
        let q = (zeta.arg() / std::f64::consts::FRAC_PI_2).round() as i64;
        let zeta0 = zeta * C::i_pow(-q);
        let mut w0 = arccn(zeta0, M_HALF);
        if w0.re < 0.0 {
            w0 = -w0; // cn is even: the half-turn about the west edge's midpoint
        }
        let kk = C::real(self.k);
        kk + C::i_pow(q) * (w0 - kk)
    }

    /// Folds w into the closed square by the half-turns about the edge midpoints. Returns the folded
    /// (Re, Im) and whether an odd number of half-turns was used (the block frame is turned 180°).
    fn canonical_w(&self, w: C) -> (f64, f64, bool) {
        let (k, eps) = (self.k, 1e-12 * self.k);
        let mut w = w;
        let mut turned = false;
        for _ in 0..256 {
            if w.re < -eps {
                w = -w;
            } else if w.re > 2.0 * k + eps {
                w = C::real(4.0 * k) - w;
            } else if w.im > k + eps {
                w = C::new(2.0 * k, 2.0 * k) - w;
            } else if w.im < -k - eps {
                w = C::new(2.0 * k, -2.0 * k) - w;
            } else {
                return (w.re, w.im, turned);
            }
            turned = !turned;
        }
        panic!("canonical_w did not converge for {w:?}");
    }

    /// Folds any block of the plane into the closed square: the canonical block that holds the same
    /// planet point, and whether the local frame is turned 180° (an odd number of seam crossings).
    pub fn canonical(&self, x: f64, z: f64) -> (f64, f64, bool) {
        let (re, im, turned) = self.canonical_w(self.w_of_block(x, z));
        let (cx, cz) = self.block_of_w(C::new(re, im));
        (cx, cz, turned)
    }

    /// The linear scale in radians of the unit sphere per block (the map is conformal, so it is the
    /// same in every direction): L = (2K/a)·2|sn·dn| / (1 + |cn|²). At the south pole (a corner) the
    /// limit is √2·(2K/a), the same as at the north pole.
    pub fn scale_rad_per_block(&self, x: f64, z: f64) -> f64 {
        let w = self.w_of_block(x, z);
        if pole_distance_proxy(w, M_HALF) < 1e-24 {
            return std::f64::consts::SQRT_2 * self.s;
        }
        let (sn, cn, dn) = sncndn_c(w, M_HALF);
        if !cn.is_finite() {
            return std::f64::consts::SQRT_2 * self.s;
        }
        let deriv = (sn * dn).abs();
        let r2 = cn.norm_sqr();
        if r2 > 1e16 {
            // |d(1/cn)/dw| = |sn dn| / |cn|²; the stereographic factor uses 1/cn near the pole.
            let inv = deriv / r2;
            return self.s * 2.0 * inv / (1.0 + 1.0 / r2);
        }
        self.s * 2.0 * deriv / (1.0 + r2)
    }

    /// Blocks per degree of great circle at block (x, z), on a planet whose surface is the square:
    /// (π/180) / L.
    pub fn blocks_per_degree(&self, x: f64, z: f64) -> f64 {
        std::f64::consts::PI / 180.0 / self.scale_rad_per_block(x, z)
    }

    /// Planet area per block, relative to the planet's mean over the square (4π / a²): 1 means a
    /// block holds an average share of the planet; below 1 the land is drawn larger than average.
    pub fn area_scale_rel(&self, x: f64, z: f64) -> f64 {
        let l = self.scale_rad_per_block(x, z);
        l * l * self.side * self.side / (4.0 * std::f64::consts::PI)
    }

    /// The four singular points (the edge midpoints) in block coordinates, in [`Edge::ALL`] order.
    pub fn singular_points(&self) -> [(f64, f64); 4] {
        let h = self.half();
        [(h, 0.0), (0.0, h), (-h, 0.0), (0.0, -h)]
    }

    /// The midpoint of an edge.
    pub fn midpoint(&self, e: Edge) -> (f64, f64) {
        self.singular_points()[Edge::ALL.iter().position(|&f| f == e).unwrap()]
    }

    /// Block (x, z) of the edge `e` at parameter t ∈ [−a/2, a/2] (t runs along +z on east/west edges,
    /// along +x on south/north edges).
    pub fn on_edge(&self, e: Edge, t: f64) -> (f64, f64) {
        let h = self.half();
        match e {
            Edge::East => (h, t),
            Edge::West => (-h, t),
            Edge::South => (t, h),
            Edge::North => (t, -h),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// splitmix64, so the tests need no crate.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> f64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
        }
        /// A uniform point on the sphere as (lat, lon) degrees.
        fn sphere(&mut self) -> (f64, f64) {
            let u = 2.0 * self.next() - 1.0;
            (u.asin().to_degrees(), 360.0 * self.next() - 180.0)
        }
    }

    fn unit_of(lat: f64, lon: f64) -> [f64; 3] {
        let (p, l) = (lat.to_radians(), lon.to_radians());
        [p.cos() * l.cos(), p.cos() * l.sin(), p.sin()]
    }

    fn angle(a: [f64; 3], b: [f64; 3]) -> f64 {
        let c = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ];
        let s = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        s.atan2(a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
    }

    const SIDE: f64 = 8192.0;

    #[test]
    fn every_point_of_the_sphere_maps_into_the_square_and_back() {
        for lon0 in [0.0, 17.5, -133.0] {
            let p = Peirce::new(SIDE, lon0);
            let mut rng = Rng(0x5eed + lon0 as u64);
            let mut worst = 0.0f64;
            let mut pts: Vec<(f64, f64)> = (0..40_000).map(|_| rng.sphere()).collect();
            // The special places: both poles, the singular points, the edge meridians.
            pts.extend([(90.0, 0.0), (-90.0, 0.0), (-89.999, 3.0), (89.9999, -77.0)]);
            for k in 0..4 {
                let l = lon0 + 90.0 * k as f64;
                pts.extend([(0.0, l), (1e-7, l), (-1e-7, l), (-45.0, l), (-89.0, l), (0.0, l + 45.0)]);
            }
            for (lat, lon) in pts {
                let (x, z) = p.block_of(lat, lon);
                let h = p.half() * (1.0 + 1e-12);
                assert!(x.abs() <= h && z.abs() <= h, "({lat}, {lon}) -> ({x}, {z}) outside");
                let back = p.unit(x, z);
                let err = angle(back, unit_of(lat, lon));
                worst = worst.max(err);
                assert!(err < 1e-9, "({lat}, {lon}) -> ({x}, {z}): {err} rad");
            }
            eprintln!("lon0 {lon0}: worst round trip {worst:e} rad");
        }
    }

    #[test]
    fn every_block_of_the_square_maps_to_the_sphere_and_back() {
        let p = Peirce::new(SIDE, 30.0);
        let mut rng = Rng(42);
        for _ in 0..40_000 {
            let x = (rng.next() - 0.5) * SIDE;
            let z = (rng.next() - 0.5) * SIDE;
            let (lat, lon) = p.lat_lon(x, z);
            let (bx, bz) = p.block_of(lat, lon);
            // Inside the open square the inverse is unique.
            let tol = 1e-6 * (1.0 + p.blocks_per_degree(x, z) / 100.0);
            assert!((bx - x).abs() < tol && (bz - z).abs() < tol, "({x}, {z}) -> ({lat}, {lon}) -> ({bx}, {bz})");
        }
    }

    #[test]
    fn north_pole_is_the_centre_and_south_pole_every_corner() {
        let p = Peirce::new(SIDE, 0.0);
        assert!((p.lat_lon(0.0, 0.0).0 - 90.0).abs() < 1e-12);
        let h = p.half();
        for (x, z) in [(h, h), (h, -h), (-h, h), (-h, -h)] {
            assert_eq!(p.lat_lon(x, z).0, -90.0);
            // Just inside a corner is just north of the south pole.
            let (lat, _) = p.lat_lon(x * 0.9999, z * 0.9999);
            assert!(lat < -89.9, "{lat}");
        }
    }

    #[test]
    fn edge_midpoints_are_equatorial_at_quarter_turns_of_lon0() {
        for lon0 in [0.0, 25.0] {
            let p = Peirce::new(SIDE, lon0);
            // East, south, west, north edge midpoints: cn(K + δ) ≈ −δ/√2 near the centre, so the
            // west edge midpoint is λ₀ and longitude grows counter-clockwise seen from above.
            let want = [180.0, 90.0, 0.0, -90.0];
            for ((x, z), w) in p.singular_points().into_iter().zip(want) {
                let (lat, lon) = p.lat_lon(x, z);
                assert!(lat.abs() < 1e-9, "({x}, {z}) lat {lat}");
                let d = (lon - (w + lon0) + 540.0).rem_euclid(360.0) - 180.0;
                assert!(d.abs() < 1e-9, "({x}, {z}) lon {lon}, want {}", w + lon0);
            }
            // The equator is the diamond through them: |x| + |z| = a/2.
            let mut rng = Rng(7);
            for _ in 0..2000 {
                let lon = 360.0 * rng.next();
                let (x, z) = p.block_of(0.0, lon);
                assert!((x.abs() + z.abs() - p.half()).abs() < 1e-6, "lon {lon}: ({x}, {z})");
            }
        }
    }

    /// The gluing: each edge is joined to itself by the half-turn about its midpoint. The two blocks
    /// it pairs on the edge are the same planet point, and the map continued past the edge equals the
    /// glued interior rotated, so walking out through the seam (turned 180°) continues the map
    /// smoothly: C∞, not just continuous.
    #[test]
    fn glued_edges_map_the_same_sphere_points_and_continue_smoothly() {
        let p = Peirce::new(SIDE, 12.0);
        let mut rng = Rng(99);
        for e in Edge::ALL {
            let (mx, mz) = p.midpoint(e);
            let turn = |x: f64, z: f64| (2.0 * mx - x, 2.0 * mz - z);
            for _ in 0..4000 {
                let t = (rng.next() - 0.5) * SIDE;
                let (x, z) = p.on_edge(e, t);
                let (gx, gz) = turn(x, z);
                let a = p.unit(x, z);
                let b = p.unit(gx, gz);
                assert!(angle(a, b) < 1e-10, "{e:?} t {t}");
                // Off the edge, both sides: the outside point is the turned inside point.
                let (dx, dz) = ((rng.next() - 0.5) * 300.0, (rng.next() - 0.5) * 300.0);
                let (ox, oz) = (x + dx, z + dz);
                let (ix, iz) = turn(ox, oz);
                assert!(angle(p.unit(ox, oz), p.unit(ix, iz)) < 1e-10, "{e:?} at ({ox}, {oz})");
            }
        }
    }

    /// A mirror across the edge line would also pair each edge point with itself, but it is not the
    /// gluing: the map off the edge does not match. This guards the portals' turn against a flip.
    #[test]
    fn a_mirror_seam_would_not_match() {
        let p = Peirce::new(SIDE, 0.0);
        let h = p.half();
        let mut bad = 0;
        for i in 0..200 {
            let t = -h + SIDE * (i as f64 + 0.5) / 200.0;
            let (x, z) = (h + 20.0, t); // 20 blocks past the east seam
            let mirror = (2.0 * h - x, z);
            if angle(p.unit(x, z), p.unit(mirror.0, mirror.1)) > 1e-4 {
                bad += 1;
            }
        }
        assert!(bad > 190, "{bad}");
    }

    #[test]
    fn canonical_folds_far_points_back_with_the_same_planet_point() {
        let p = Peirce::new(SIDE, 5.0);
        let mut rng = Rng(3);
        for _ in 0..5000 {
            let x = (rng.next() - 0.5) * 6.0 * SIDE;
            let z = (rng.next() - 0.5) * 6.0 * SIDE;
            let (cx, cz, _) = p.canonical(x, z);
            assert!(cx.abs() <= p.half() + 1e-6 && cz.abs() <= p.half() + 1e-6);
            if p.zeta(x, z).is_some() {
                assert!(angle(p.unit(x, z), p.unit(cx, cz)) < 1e-9);
            }
        }
    }

    /// Conformal: the Jacobian of block → unit sphere, measured by central differences, has equal
    /// singular values and positive orientation (east = +x, north = −z, not mirrored), away from the
    /// four singular points. Its scale agrees with the analytic one.
    #[test]
    fn the_map_is_conformal_orientation_preserving_and_its_scale_is_the_analytic_one() {
        let p = Peirce::new(SIDE, 40.0);
        let mut rng = Rng(11);
        let mut checked = 0;
        for _ in 0..20_000 {
            let x = (rng.next() - 0.5) * SIDE * 0.999;
            let z = (rng.next() - 0.5) * SIDE * 0.999;
            let near_singular = p
                .singular_points()
                .iter()
                .any(|&(sx, sz)| (x - sx).hypot(z - sz) < 20.0);
            if near_singular {
                continue;
            }
            let h = 0.01;
            let d = |ax: f64, az: f64| {
                let a = p.unit(x + ax, z + az);
                let b = p.unit(x - ax, z - az);
                [(a[0] - b[0]) / (2.0 * h), (a[1] - b[1]) / (2.0 * h), (a[2] - b[2]) / (2.0 * h)]
            };
            let ex = d(h, 0.0);
            let en = d(0.0, -h); // north is −z
            let n = p.unit(x, z);
            let len = |v: [f64; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            let dot = ex[0] * en[0] + ex[1] * en[1] + ex[2] * en[2];
            let (lx, ln) = (len(ex), len(en));
            assert!((lx / ln - 1.0).abs() < 1e-5, "({x}, {z}): {lx} vs {ln}");
            assert!(dot.abs() < 1e-5 * lx * ln, "({x}, {z}): not orthogonal");
            let cross = [
                ex[1] * en[2] - ex[2] * en[1],
                ex[2] * en[0] - ex[0] * en[2],
                ex[0] * en[1] - ex[1] * en[0],
            ];
            assert!(cross[0] * n[0] + cross[1] * n[1] + cross[2] * n[2] > 0.0, "mirrored at ({x}, {z})");
            let want = p.scale_rad_per_block(x, z);
            assert!((lx / want - 1.0).abs() < 1e-5, "({x}, {z}): {lx} vs {want}");
            checked += 1;
        }
        assert!(checked > 19_000);
    }

    /// The singular points: the scale (radians per block) falls to zero at the four edge midpoints,
    /// and nowhere else; near each it goes like the distance r (angles doubled: planet distance ∝ r²).
    #[test]
    fn the_four_singular_points_are_the_edge_midpoints() {
        let p = Peirce::new(SIDE, 0.0);
        let n = 256;
        let mut minima = Vec::new();
        let step = SIDE / n as f64;
        let l = |i: i64, j: i64| {
            let (x, z) = (-p.half() + i as f64 * step, -p.half() + j as f64 * step);
            p.scale_rad_per_block(x, z)
        };
        for i in 0..=n as i64 {
            for j in 0..=n as i64 {
                let v = l(i, j);
                let mut is_min = true;
                for (di, dj) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let (a, b) = (i + di, j + dj);
                    if (0..=n as i64).contains(&a) && (0..=n as i64).contains(&b) && l(a, b) < v {
                        is_min = false;
                    }
                }
                if is_min {
                    minima.push((-p.half() + i as f64 * step, -p.half() + j as f64 * step, v));
                }
            }
        }
        let zeros: Vec<_> = minima.iter().filter(|m| m.2 < 1e-10).collect();
        assert_eq!(zeros.len(), 4, "{minima:?}");
        for (sx, sz) in p.singular_points() {
            assert!(zeros.iter().any(|m| (m.0 - sx).abs() < 1e-9 && (m.1 - sz).abs() < 1e-9));
            // L ∝ r: halving the distance halves the scale (sampled on a ray into the square,
            // slanted off the normal).
            let inward = (-sx.signum(), -sz.signum());
            let slant = (if sx == 0.0 { 0.3 } else { 0.0 }, if sz == 0.0 { 0.3 } else { 0.0 });
            let at = |r: f64| p.scale_rad_per_block(sx + r * (inward.0 + slant.0), sz + r * (inward.1 + slant.1));
            let ratio = at(8.0) / at(4.0);
            assert!((ratio - 2.0).abs() < 1e-3, "({sx}, {sz}): ratio {ratio}");
        }
        // Every other local minimum of the scale is well above zero.
        let others = minima.iter().filter(|m| m.2 >= 1e-10).map(|m| m.2).fold(f64::INFINITY, f64::min);
        assert!(others > 1e-5, "{others}");
    }

    /// Area distortion, measured: the planet area per block relative to the mean, weighted by planet
    /// area. The poles hold the least planet per block (2K²/π ≈ 2.19× the mean: the land there is
    /// drawn largest), and the share of planet area falls to zero at the singular points.
    #[test]
    fn area_distortion_is_measured() {
        let p = Peirce::new(SIDE, 0.0);
        let kk = complete_k(M_HALF);
        let pole = 2.0 * kk * kk / std::f64::consts::PI;
        assert!((p.area_scale_rel(0.0, 0.0) - pole).abs() < 1e-9);
        assert!((p.area_scale_rel(p.half(), p.half()) - pole).abs() < 1e-9);
        // Integrate over a fine grid of the square: the mean is 1 by construction (the square holds
        // the sphere exactly once).
        let n = 1200;
        let step = SIDE / n as f64;
        let mut sum = 0.0;
        let mut max = 0.0f64;
        for i in 0..n {
            for j in 0..n {
                let (x, z) = (-p.half() + (i as f64 + 0.5) * step, -p.half() + (j as f64 + 0.5) * step);
                let a = p.area_scale_rel(x, z);
                sum += a;
                max = max.max(a);
            }
        }
        let mean = sum / (n * n) as f64;
        assert!((mean - 1.0).abs() < 2e-3, "mean {mean}");
        assert!((max - pole).abs() < 1e-3, "max {max} vs pole {pole}");
    }
}

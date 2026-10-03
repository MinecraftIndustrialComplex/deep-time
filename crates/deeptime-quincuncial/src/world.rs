// SPDX-License-Identifier: LGPL-3.0-or-later
//! One quincuncial world, in either aspect (PROTOTYPE).
//!
//! The two layouts are the same conformal map ([`Peirce`], the elliptic function cn(· | ½)) under a
//! different rotation of the sphere and of the block plane:
//!
//! - **Peirce** (`Layout::Peirce`): the projection's pole is the geographic north pole. The world is
//!   one square of side `a`; the north pole is its centre, the south pole its four corners, the
//!   equator the diamond through the edge midpoints (the four singular points, on the equator 90°
//!   of longitude apart). Every edge is glued to itself by a half-turn about its midpoint.
//! - **Guyou** (`Layout::Guyou`, Guyou 1887; "an oblique aspect of the Peirce quincuncial
//!   projection", Wikipedia): the projection's pole is a point of the equator, the centre of the
//!   western hemisphere square; the block plane is turned 45° so that each hemisphere is an
//!   axis-aligned square of side `b` = a/√2 and the world is the 2b × b rectangle |x| ≤ b,
//!   |z| ≤ b/2 with the equator the straight line z = 0. The singular points are the corners of the
//!   hemisphere squares: the long edges' midpoints (0, ±b/2) and the rectangle's corners. The short
//!   edges are glued by a slide of 2b (a plain translation, as Toroidal World's x wrap), each long
//!   edge to itself by a half-turn about its midpoint.
//!
//! Both are exactly the same p2 tiling of the plane; only the cut of one tile and the geography it
//! carries differ. `unit` is defined on the whole plane, so a reader that samples outside the
//! canonical domain gets the glued neighbour's planet point, the "folded outside".

use crate::glue::{Seam, SeamKind};
use crate::peirce::Peirce;

/// Which aspect and cut of the quincuncial map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Peirce,
    Guyou,
}

impl Layout {
    pub fn parse(s: &str) -> Option<Layout> {
        match s {
            "peirce" => Some(Layout::Peirce),
            "guyou" => Some(Layout::Guyou),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Layout::Peirce => "peirce",
            Layout::Guyou => "guyou",
        }
    }
}

type Mat3 = [[f64; 3]; 3];

fn mat_vec(m: &Mat3, v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn mat_t_vec(m: &Mat3, v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2],
        m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2],
        m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2],
    ]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalise(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / l, v[1] / l, v[2] / l]
}

pub fn unit_of_lat_lon(lat_deg: f64, lon_deg: f64) -> [f64; 3] {
    let (p, l) = (lat_deg.to_radians(), lon_deg.to_radians());
    [p.cos() * l.cos(), p.cos() * l.sin(), p.sin()]
}

pub fn lat_lon_of_unit(v: [f64; 3]) -> (f64, f64) {
    let lat = v[2].clamp(-1.0, 1.0).asin().to_degrees();
    let lon = if v[0] == 0.0 && v[1] == 0.0 { 0.0 } else { v[1].atan2(v[0]).to_degrees() };
    (lat, lon)
}

/// A quincuncial world in block coordinates.
#[derive(Clone, Debug)]
pub struct QWorld {
    layout: Layout,
    /// The Peirce square's side a, blocks (Guyou: a = b√2).
    a: f64,
    /// Guyou's hemisphere-square side b = a/√2; for Peirce the square's side.
    size: f64,
    lon_deg: f64,
    peirce: Peirce,
    /// The rotation from the projection's frame (its pole on +z) to geographic coordinates.
    r: Mat3,
}

impl QWorld {
    /// The Peirce square of side `a` (north pole in the centre); its singular points are on the
    /// equator at longitudes `lon0` + {0°, 90°, 180°, 270°} (west, south, east, north edge).
    pub fn peirce(a: f64, lon0_deg: f64) -> QWorld {
        let (s, c) = lon0_deg.to_radians().sin_cos();
        QWorld {
            layout: Layout::Peirce,
            a,
            size: a,
            lon_deg: lon0_deg,
            peirce: Peirce::new(a, 0.0),
            r: [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// The Guyou rectangle of two hemisphere squares of side `b` (the world 2b × b), the western
    /// one centred on the equator at longitude `centre_lon` (the eastern one at `centre_lon` + 180°).
    /// North is −z everywhere on the line z = 0 (the equator), east is +x.
    pub fn guyou(b: f64, centre_lon_deg: f64) -> QWorld {
        let a = b * std::f64::consts::SQRT_2;
        let peirce = Peirce::new(a, 0.0);
        let mut w = QWorld {
            layout: Layout::Guyou,
            a,
            size: b,
            lon_deg: centre_lon_deg,
            peirce,
            r: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        };
        // The projection-frame direction of world north (−z) at the west square's centre, which
        // is the projection's pole: the meridian of the frame a short step goes down.
        let (fx, fz) = w.to_frame(-0.5 * b, -1e-3 * b);
        let step = w.peirce.unit(fx, fz);
        let a_dir = normalise([step[0], step[1], 0.0]);
        let e3 = [0.0, 0.0, 1.0];
        let b_dir = cross(e3, a_dir);
        let big_e3 = unit_of_lat_lon(0.0, centre_lon_deg);
        let big_a = [0.0, 0.0, 1.0]; // north at an equator point
        let big_b = cross(big_e3, big_a);
        let mut r = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                r[i][j] = big_e3[i] * e3[j] + big_a[i] * a_dir[j] + big_b[i] * b_dir[j];
            }
        }
        w.r = r;
        w
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Peirce: the square's side; Guyou: the hemisphere square's side b.
    pub fn size(&self) -> f64 {
        self.size
    }

    /// The Peirce square side a of the underlying map (Guyou: b√2).
    pub fn peirce_side(&self) -> f64 {
        self.a
    }

    /// Peirce: λ₀; Guyou: the western hemisphere's centre longitude.
    pub fn lon_deg(&self) -> f64 {
        self.lon_deg
    }

    /// The canonical domain (x0, x1, z0, z1).
    pub fn extent(&self) -> (f64, f64, f64, f64) {
        match self.layout {
            Layout::Peirce => {
                let h = 0.5 * self.a;
                (-h, h, -h, h)
            }
            Layout::Guyou => {
                let b = self.size;
                (-b, b, -0.5 * b, 0.5 * b)
            }
        }
    }

    /// Area of the canonical domain, blocks² (a² for both: the sphere once).
    pub fn domain_area(&self) -> f64 {
        self.a * self.a
    }

    /// World block → the Peirce frame's block (Guyou: shift the west square's centre to the
    /// origin, then turn the plane 45°).
    pub fn to_frame(&self, x: f64, z: f64) -> (f64, f64) {
        match self.layout {
            Layout::Peirce => (x, z),
            Layout::Guyou => {
                let (u, v) = (x + 0.5 * self.size, z);
                let c = std::f64::consts::FRAC_1_SQRT_2;
                (c * (u - v), c * (u + v))
            }
        }
    }

    pub fn from_frame(&self, fx: f64, fz: f64) -> (f64, f64) {
        match self.layout {
            Layout::Peirce => (fx, fz),
            Layout::Guyou => {
                let c = std::f64::consts::FRAC_1_SQRT_2;
                let (u, v) = (c * (fx + fz), c * (fz - fx));
                (u - 0.5 * self.size, v)
            }
        }
    }

    /// The geographic unit vector of world block (x, z), anywhere on the plane.
    pub fn unit(&self, x: f64, z: f64) -> [f64; 3] {
        let (fx, fz) = self.to_frame(x, z);
        mat_vec(&self.r, self.peirce.unit(fx, fz))
    }

    /// (latitude, longitude) degrees of world block (x, z), longitude in (−180°, 180°].
    pub fn lat_lon(&self, x: f64, z: f64) -> (f64, f64) {
        lat_lon_of_unit(self.unit(x, z))
    }

    /// The world block of a planet point, in the closed canonical domain.
    pub fn block_of(&self, lat_deg: f64, lon_deg: f64) -> (f64, f64) {
        let p = mat_t_vec(&self.r, unit_of_lat_lon(lat_deg, lon_deg));
        let (plat, plon) = lat_lon_of_unit(p);
        let (fx, fz) = self.peirce.block_of(plat, plon);
        let (x, z) = self.from_frame(fx, fz);
        let (cx, cz, _) = self.canonical(x, z);
        (cx, cz)
    }

    /// Folds a world block of the plane into the closed canonical domain by the seams' transforms:
    /// the block holding the same planet point, and whether the local frame is turned 180°.
    pub fn canonical(&self, x: f64, z: f64) -> (f64, f64, bool) {
        match self.layout {
            Layout::Peirce => self.peirce.canonical(x, z),
            Layout::Guyou => {
                let b = self.size;
                let eps = 1e-9 * b;
                let (mut x, mut z, mut turned) = (x, z, false);
                for _ in 0..256 {
                    if x < -b - eps || x > b + eps {
                        x -= 2.0 * b * ((x + b) / (2.0 * b)).floor();
                    }
                    if z > 0.5 * b + eps {
                        (x, z) = (-x, b - z);
                        turned = !turned;
                    } else if z < -0.5 * b - eps {
                        (x, z) = (-x, -b - z);
                        turned = !turned;
                    } else if x >= -b - eps && x <= b + eps {
                        return (x, z, turned);
                    }
                }
                panic!("canonical did not converge at ({x}, {z})");
            }
        }
    }

    /// Radians of the unit sphere per block (isotropic: the map is conformal).
    pub fn scale_rad_per_block(&self, x: f64, z: f64) -> f64 {
        let (fx, fz) = self.to_frame(x, z);
        self.peirce.scale_rad_per_block(fx, fz)
    }

    pub fn blocks_per_degree(&self, x: f64, z: f64) -> f64 {
        std::f64::consts::PI / 180.0 / self.scale_rad_per_block(x, z)
    }

    /// Planet area per block relative to the planet's mean over the domain.
    pub fn area_scale_rel(&self, x: f64, z: f64) -> f64 {
        let l = self.scale_rad_per_block(x, z);
        l * l * self.domain_area() / (4.0 * std::f64::consts::PI)
    }

    /// The four distinct singular points, as world blocks inside the closed domain.
    pub fn singular_points(&self) -> Vec<(f64, f64)> {
        match self.layout {
            Layout::Peirce => self.peirce.singular_points().to_vec(),
            Layout::Guyou => {
                let b = self.size;
                vec![(0.0, -0.5 * b), (0.0, 0.5 * b), (-b, -0.5 * b), (-b, 0.5 * b)]
            }
        }
    }

    /// The seams: every edge of the domain with its gluing transform.
    pub fn seams(&self) -> Vec<Seam> {
        match self.layout {
            Layout::Peirce => {
                let h = 0.5 * self.a;
                vec![
                    Seam::new("east", 'x', h, (-1.0, 0.0), self.a, SeamKind::HalfTurn { cx: h, cz: 0.0 }),
                    Seam::new("south", 'z', h, (0.0, -1.0), self.a, SeamKind::HalfTurn { cx: 0.0, cz: h }),
                    Seam::new("west", 'x', -h, (1.0, 0.0), self.a, SeamKind::HalfTurn { cx: -h, cz: 0.0 }),
                    Seam::new("north", 'z', -h, (0.0, 1.0), self.a, SeamKind::HalfTurn { cx: 0.0, cz: -h }),
                ]
            }
            Layout::Guyou => {
                let b = self.size;
                vec![
                    Seam::new("east", 'x', b, (-1.0, 0.0), b, SeamKind::Slide { dx: -2.0 * b, dz: 0.0 }),
                    Seam::new("west", 'x', -b, (1.0, 0.0), b, SeamKind::Slide { dx: 2.0 * b, dz: 0.0 }),
                    Seam::new("south", 'z', 0.5 * b, (0.0, -1.0), 2.0 * b, SeamKind::HalfTurn { cx: 0.0, cz: 0.5 * b }),
                    Seam::new("north", 'z', -0.5 * b, (0.0, 1.0), 2.0 * b, SeamKind::HalfTurn { cx: 0.0, cz: -0.5 * b }),
                ]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> f64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    fn angle(a: [f64; 3], b: [f64; 3]) -> f64 {
        let c = cross(a, b);
        (c[0] * c[0] + c[1] * c[1] + c[2] * c[2])
            .sqrt()
            .atan2(a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
    }

    fn worlds() -> Vec<QWorld> {
        vec![QWorld::peirce(11_585.0, 20.0), QWorld::guyou(8192.0, -70.0), QWorld::guyou(8192.0, 33.0)]
    }

    #[test]
    fn both_aspects_round_trip_the_sphere_into_their_domain() {
        for w in worlds() {
            let (x0, x1, z0, z1) = w.extent();
            let mut rng = Rng(5);
            for _ in 0..30_000 {
                let lat = (2.0 * rng.next() - 1.0).asin().to_degrees();
                let lon = 360.0 * rng.next() - 180.0;
                let (x, z) = w.block_of(lat, lon);
                let t = 1e-6 * w.size();
                assert!(x >= x0 - t && x <= x1 + t && z >= z0 - t && z <= z1 + t, "{:?} ({lat}, {lon}) -> ({x}, {z})", w.layout());
                let err = angle(w.unit(x, z), unit_of_lat_lon(lat, lon));
                assert!(err < 1e-9, "{:?} ({lat}, {lon}): {err}", w.layout());
            }
        }
    }

    #[test]
    fn every_seam_glues_the_same_planet_points_and_continues_the_map() {
        for w in worlds() {
            let mut rng = Rng(17);
            for seam in w.seams() {
                for _ in 0..3000 {
                    let (x, z) = seam.point((rng.next() - 0.5) * seam.length);
                    let (gx, gz) = seam.apply(x, z);
                    assert!(angle(w.unit(x, z), w.unit(gx, gz)) < 1e-10, "{} at ({x}, {z})", seam.name);
                    let (ox, oz) = (x + (rng.next() - 0.5) * 200.0, z + (rng.next() - 0.5) * 200.0);
                    let (ix, iz) = seam.apply(ox, oz);
                    assert!(angle(w.unit(ox, oz), w.unit(ix, iz)) < 1e-10, "{} off-seam", seam.name);
                }
            }
        }
    }

    #[test]
    fn guyou_has_a_straight_equator_north_up_and_singular_points_at_45_degrees() {
        let w = QWorld::guyou(8192.0, -70.0);
        let mut rng = Rng(2);
        for _ in 0..2000 {
            let x = (rng.next() - 0.5) * 2.0 * 8192.0;
            let (lat, _) = w.lat_lon(x, 0.0);
            assert!(lat.abs() < 1e-9, "x {x}: lat {lat}");
        }
        // Centres of the hemisphere squares.
        let (lat, lon) = w.lat_lon(-4096.0, 0.0);
        assert!(lat.abs() < 1e-9 && (lon + 70.0).abs() < 1e-9, "{lat} {lon}");
        let (_, lon) = w.lat_lon(4096.0, 0.0);
        assert!((lon - 110.0).abs() < 1e-9, "{lon}");
        // North is −z and east +x along the equator.
        for x in [-6000.0, -4096.0, -100.0, 2500.0, 7000.0] {
            let (la, lo) = w.lat_lon(x, 0.0);
            let (la_n, _) = w.lat_lon(x, -1.0);
            let (_, lo_e) = w.lat_lon(x + 1.0, 0.0);
            assert!(la_n > la, "north at x {x}");
            let d = (lo_e - lo + 540.0).rem_euclid(360.0) - 180.0;
            assert!(d > 0.0, "east at x {x}");
        }
        // The poles sit on the long edges, a quarter of the way along.
        let (lat, _) = w.lat_lon(-4096.0, -4096.0);
        assert!((lat - 90.0).abs() < 1e-7, "{lat}");
        let (lat, _) = w.lat_lon(4096.0, 4096.0);
        assert!((lat + 90.0).abs() < 1e-7, "{lat}");
        // Singular points: hemisphere-square corners, at ±45° latitude on the meridians 90° from
        // the hemisphere centres.
        for (x, z) in w.singular_points() {
            let (lat, lon) = w.lat_lon(x, z);
            assert!((lat.abs() - 45.0).abs() < 1e-7, "({x}, {z}): {lat}");
            let d = ((lon + 70.0) + 540.0).rem_euclid(360.0) - 180.0;
            assert!((d.abs() - 90.0).abs() < 1e-7, "({x}, {z}): lon {lon}");
            assert!(w.scale_rad_per_block(x, z) < 1e-12);
        }
    }

    #[test]
    fn canonical_lands_in_the_domain_with_the_same_planet_point() {
        for w in worlds() {
            let (x0, x1, z0, z1) = w.extent();
            let mut rng = Rng(23);
            for _ in 0..5000 {
                let x = (rng.next() - 0.5) * 8.0 * w.size();
                let z = (rng.next() - 0.5) * 8.0 * w.size();
                let (cx, cz, _) = w.canonical(x, z);
                let t = 1e-6 * w.size();
                assert!(cx >= x0 - t && cx <= x1 + t && cz >= z0 - t && cz <= z1 + t);
                let u = w.unit(x, z);
                if u[2] > -1.0 + 1e-9 {
                    assert!(angle(u, w.unit(cx, cz)) < 1e-8, "({x}, {z}) -> ({cx}, {cz})");
                }
            }
        }
    }
}

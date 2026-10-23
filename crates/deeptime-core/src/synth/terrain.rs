// SPDX-License-Identifier: LGPL-3.0-or-later
//! The synthetic terrains. `WrapTest` is the Java `AnalyticTestTerrain` (plan §7.1) transcribed
//! operation for operation, so world data written from it matches what the generator produces from
//! the closed form (P0-10 compares the two). Keep the two in step: a change to one is a change to
//! both.

use std::f64::consts::PI;

use deeptime_format::rivers::{Chain, Mouth, RiverVertex, Rivers};

/// Water surface of the lake test, in blocks.
pub const LAKE_WATER_Y: f64 = 120.0;

/// A terrain the synthesiser can sample at block coordinates.
pub trait Terrain: Sync {
    fn circumference(&self) -> u32;
    /// Terrain surface height H(x, z) in blocks.
    fn height(&self, x: i64, z: i64) -> f64;
    /// Water surface above the terrain, if any, in blocks.
    fn water_level(&self, _x: i64, _z: i64) -> Option<f64> {
        None
    }

    /// Whether no sea stands on block (x, z) though its ground may lie below sea level (the
    /// `closed_basin` layer, HYD-100); `None` where the terrain writes no such layer (it is then
    /// asked at (0, 0) only).
    fn closed_basin(&self, _x: i64, _z: i64) -> Option<bool> {
        None
    }

    /// Latitude φ = asin(z/R), R = C/2π (the Lambert projection, `deeptime_format::projection`;
    /// south positive here), clamped to ±π/2 beyond the poles (`AnalyticTestTerrain.latitude`;
    /// its own formula, twinned in Java).
    fn latitude(&self, z: i64) -> f64 {
        let c = f64::from(self.circumference());
        (z as f64 * std::f64::consts::TAU / c)
            .clamp(-1.0, 1.0)
            .asin()
    }

    /// A 2D biome input in vanilla parameter units (`AnalyticTestTerrain.raster`).
    fn raster(&self, layer: &str, x: i64, z: i64) -> f64 {
        match layer {
            "climate/temperature" => 0.9 - 1.8 * self.latitude(z).abs() / (PI / 2.0),
            "climate/continents" => {
                (-1.0f64).max(1.0f64.min(-0.15 + (self.height(x, z) - 63.0) / 80.0))
            }
            "climate/erosion" => 0.2,
            "climate/ridges" => 0.35,
            _ => 0.0,
        }
    }
}

/// `AnalyticTestTerrain`: two sine continents (one straddling the seam) and a Gaussian range at
/// x_r = C/8, z_r = C/32.
pub struct WrapTest {
    c: u32,
    two_pi_over_c: f64,
    xr: f64,
    zr: f64,
}

impl WrapTest {
    pub fn new(c: u32) -> WrapTest {
        WrapTest {
            c,
            two_pi_over_c: 2.0 * PI / f64::from(c),
            xr: f64::from(c) / 8.0,
            zr: f64::from(c) / 32.0,
        }
    }
}

impl Terrain for WrapTest {
    fn circumference(&self) -> u32 {
        self.c
    }

    fn height(&self, x: i64, z: i64) -> f64 {
        let c = f64::from(self.c);
        let theta = x as f64 * self.two_pi_over_c;
        let phi = self.latitude(z);
        let mut h = 63.0
            + 60.0 * (theta - PI).cos() * phi.cos()
            + 25.0 * (3.0 * theta + 0.5).sin() * (2.0 * phi).cos();
        let mut dx = x as f64 - self.xr;
        dx -= c * (dx / c).round_ties_even(); // Java Math.rint
        let dz = z as f64 - self.zr;
        h += 450.0 * (-(dx * dx) / (2.0 * 420.0 * 420.0) - (dz * dz) / (2.0 * 260.0 * 260.0)).exp();
        h
    }
}

/// The lake test (P0-13): a plain at y 100 holding a round basin and a closed channel. The water
/// footprint is the union of a disc and a capsule; `d` is the signed distance to it (negative
/// inside). Profile by `d`: floor y 110 for d ≤ 0, a wall rising to y 125 over 4 blocks, a rim
/// crest at y 125 for 4 < d ≤ 12, an outer slope down to the plain at d = 36.
///
/// West of it, two closed bowls below sea level (HYD-100, [`LakeTest::BOWLS`]): round floors of
/// radius [`LakeTest::BOWL_FLOOR_RADIUS`] rising straight to the plain over
/// [`LakeTest::BOWL_SLOPE`] blocks, the whole bowl `closed_basin`. One is dry, the other holds a
/// lake whose surface stands below sea level.
pub struct LakeTest {
    pub c: u32,
}

/// One of the lake test's closed bowls below sea level.
pub struct ClosedBowl {
    pub centre: (f64, f64),
    pub floor_y: f64,
    /// The lake's surface, blocks; `None`: dry.
    pub lake_y: Option<f64>,
}

impl LakeTest {
    pub const CENTRE: (f64, f64) = (0.0, 0.0);
    pub const LAKE_RADIUS: f64 = 40.0;
    /// The channel runs east from the lake centre along z = 0 to this x, and is closed there.
    pub const CHANNEL_END_X: f64 = 160.0;
    pub const CHANNEL_HALF_WIDTH: f64 = 4.0;
    pub const FLOOR_Y: f64 = 110.0;
    pub const RIM_Y: f64 = 125.0;
    pub const PLAIN_Y: f64 = 100.0;
    pub const WALL: f64 = 4.0;
    pub const CREST: f64 = 8.0;
    pub const SLOPE: f64 = 24.0;
    /// The closed bowls below sea level (HYD-100): a dry one and one with a lake at y 50, under
    /// the sea's y 63 (ledger: BAK-056, game-choice: agent, 2026-10-20, a test fixture).
    pub const BOWLS: [ClosedBowl; 2] = [
        ClosedBowl {
            centre: (-200.0, -70.0),
            floor_y: 40.0,
            lake_y: None,
        },
        ClosedBowl {
            centre: (-200.0, 70.0),
            floor_y: 30.0,
            lake_y: Some(50.0),
        },
    ];
    pub const BOWL_FLOOR_RADIUS: f64 = 12.0;
    /// The bowl's wall, floor to plain, blocks.
    pub const BOWL_SLOPE: f64 = 30.0;
    /// A lake bowl's `water_level` cells lie within this distance of its centre (its water's
    /// edge is at about 20.6, where the wall reaches y 50).
    pub const BOWL_WATER_RADIUS: f64 = 24.0;

    /// Distance (blocks) from block (x, z) to bowl `b`'s centre, periodic in x.
    pub fn bowl_distance(&self, b: &ClosedBowl, x: i64, z: i64) -> f64 {
        let c = f64::from(self.c);
        let mut px = x as f64 - b.centre.0;
        px -= c * (px / c).round_ties_even();
        let pz = z as f64 - b.centre.1;
        (px * px + pz * pz).sqrt()
    }

    fn bowl_height(b: &ClosedBowl, r: f64) -> f64 {
        let r = (r - Self::BOWL_FLOOR_RADIUS).max(0.0);
        if r >= Self::BOWL_SLOPE {
            Self::PLAIN_Y
        } else {
            b.floor_y + (Self::PLAIN_Y - b.floor_y) * r / Self::BOWL_SLOPE
        }
    }

    /// Signed distance (blocks) to the water footprint.
    pub fn footprint_distance(&self, x: i64, z: i64) -> f64 {
        let c = f64::from(self.c);
        // Periodic in x, like everything else.
        let mut px = x as f64 - Self::CENTRE.0;
        px -= c * (px / c).round_ties_even();
        let pz = z as f64 - Self::CENTRE.1;
        let disc = (px * px + pz * pz).sqrt() - Self::LAKE_RADIUS;
        let t = px.clamp(0.0, Self::CHANNEL_END_X);
        let seg = ((px - t) * (px - t) + pz * pz).sqrt() - Self::CHANNEL_HALF_WIDTH;
        disc.min(seg)
    }

    /// The geometry, for `manifest.json` `notes.lake`.
    pub fn notes() -> serde_json::Value {
        serde_json::json!({
            "centre_block": [Self::CENTRE.0, Self::CENTRE.1],
            "lake_floor_radius": Self::LAKE_RADIUS,
            "channel": {
                "from_block": [Self::CENTRE.0, Self::CENTRE.1],
                "to_block": [Self::CHANNEL_END_X, Self::CENTRE.1],
                "floor_half_width": Self::CHANNEL_HALF_WIDTH,
                "closed_end": true
            },
            "floor_y": Self::FLOOR_Y,
            "rim_crest_y": Self::RIM_Y,
            "water_level_y": LAKE_WATER_Y,
            "plain_y": Self::PLAIN_Y,
            "profile": "d = signed distance to the footprint (disc ∪ channel capsule); y = 110 for d ≤ 0, rises linearly to 125 at d = 4, 125 until d = 12, falls linearly to 100 at d = 36, 100 beyond",
            "water": "water_level = 120 on every cell with d ≤ 4 (inside the rim crest), nodata elsewhere; water belongs only where it is above the terrain (d < 2.67)",
            "closed_bowls": Self::BOWLS.iter().map(|b| serde_json::json!({
                "centre_block": [b.centre.0, b.centre.1],
                "floor_y": b.floor_y,
                "floor_radius": Self::BOWL_FLOOR_RADIUS,
                "slope_blocks": Self::BOWL_SLOPE,
                "lake_level_y": b.lake_y,
                "water_radius": b.lake_y.map(|_| Self::BOWL_WATER_RADIUS),
            })).collect::<Vec<_>>(),
            "closed_basin": "1 within floor_radius + slope_blocks of a closed bowl's centre (HYD-100: no sea stands there), 0 elsewhere; a bowl's profile is floor_y out to floor_radius, then straight up to plain_y over slope_blocks"
        })
    }
}

impl Terrain for LakeTest {
    fn circumference(&self) -> u32 {
        self.c
    }

    fn height(&self, x: i64, z: i64) -> f64 {
        let bowl = Self::BOWLS
            .iter()
            .map(|b| Self::bowl_height(b, self.bowl_distance(b, x, z)))
            .fold(Self::PLAIN_Y, f64::min);
        if bowl < Self::PLAIN_Y {
            return bowl;
        }
        let d = self.footprint_distance(x, z);
        let (w, cr, s) = (Self::WALL, Self::CREST, Self::SLOPE);
        if d <= 0.0 {
            Self::FLOOR_Y
        } else if d <= w {
            Self::FLOOR_Y + (Self::RIM_Y - Self::FLOOR_Y) * d / w
        } else if d <= w + cr {
            Self::RIM_Y
        } else if d <= w + cr + s {
            Self::RIM_Y - (Self::RIM_Y - Self::PLAIN_Y) * (d - w - cr) / s
        } else {
            Self::PLAIN_Y
        }
    }

    fn water_level(&self, x: i64, z: i64) -> Option<f64> {
        for b in &Self::BOWLS {
            if let Some(y) = b.lake_y
                && self.bowl_distance(b, x, z) <= Self::BOWL_WATER_RADIUS
            {
                return Some(y);
            }
        }
        (self.footprint_distance(x, z) <= Self::WALL).then_some(LAKE_WATER_Y)
    }

    fn closed_basin(&self, x: i64, z: i64) -> Option<bool> {
        Some(
            Self::BOWLS
                .iter()
                .any(|b| self.bowl_distance(b, x, z) <= Self::BOWL_FLOOR_RADIUS + Self::BOWL_SLOPE),
        )
    }
}

/// The river test (HYD-120 to HYD-124): a big river into the sea and a flush stream, the shapes
/// Ben's play world showed a block deep and curbed (2026-10-22). The ground is a function of x
/// alone, periodic in x:
///
/// - a plain at y 70.5 (ground block 70) for x ≤ −250 and x ≥ 400, through the seam;
/// - a slope down to flats at y 63.5 (ground 63, a block over the sea's top block 62), x −250 to −50;
/// - the flats to x 40, then shallows at y 61.5 (ground 61: sea, one block of it) to x 100;
/// - a shore down to a sea floor at y 45.5 over x 100 to 130, the sea to x 300, and a rise back to
///   the plain by x 400.
///
/// The river (chain 0) runs along z = 0 from x −400 to x 160 (in the sea), a vertex every 4
/// blocks, half-width 10 and centre depth 8 throughout, so every column of it lies in several
/// overlapping segment capsules, as on a baked planet; its surface falls from 68.9 to the sea's
/// 63.0 at x −50 and stays there across the flats and the shallows. The stream (chain 1) runs south
/// along x −332 on the plain from z −110 to z −14 and then into the river's vertex at x −332:
/// half-width 1, depth 1, its surface 70.95 falling to 70.55, so its top block is the plain's ground
/// block (flush). Every number here is a test fixture's (game-choice: agent, 2026-10-22).
pub struct RiverTest {
    pub c: u32,
}

impl RiverTest {
    pub const PLAIN_Y: f64 = 70.5;
    pub const FLATS_Y: f64 = 63.5;
    pub const SHALLOWS_Y: f64 = 61.5;
    pub const SEA_FLOOR_Y: f64 = 45.5;
    /// Where the plain ends and the slope begins, and where the flats begin.
    pub const SLOPE_FROM_X: f64 = -250.0;
    pub const FLATS_FROM_X: f64 = -50.0;
    pub const SHALLOWS_FROM_X: f64 = 40.0;
    pub const SHORE_FROM_X: f64 = 100.0;
    pub const SEA_FROM_X: f64 = 130.0;
    pub const RISE_FROM_X: f64 = 300.0;
    pub const PLAIN_FROM_X: f64 = 400.0;

    /// The river's centre line z, its first and last vertex x and the spacing of its vertices.
    pub const RIVER_Z: f64 = 0.0;
    pub const RIVER_FROM_X: f64 = -400.0;
    pub const RIVER_TO_X: f64 = 160.0;
    pub const RIVER_SPACING: f64 = 4.0;
    pub const RIVER_HALF_WIDTH: f64 = 10.0;
    pub const RIVER_DEPTH: f64 = 8.0;
    pub const RIVER_Q: f64 = 400.0;
    /// The river's surface at its first vertex; it falls evenly to [`Self::SEA_W`] at
    /// [`Self::FLATS_FROM_X`] and is that from there on.
    pub const RIVER_SOURCE_W: f64 = 68.9;
    /// The sea's level, the synthetic worlds' (`Scale::default_for`).
    pub const SEA_W: f64 = 63.0;

    /// The stream: its x, its first and last vertex z before the confluence, and its spacing.
    pub const STREAM_X: f64 = -332.0;
    pub const STREAM_FROM_Z: f64 = -110.0;
    pub const STREAM_TO_Z: f64 = -14.0;
    pub const STREAM_SPACING: f64 = 4.0;
    pub const STREAM_HALF_WIDTH: f64 = 1.0;
    pub const STREAM_DEPTH: f64 = 1.0;
    pub const STREAM_Q: f64 = 2.0;
    pub const STREAM_W_FROM: f64 = 70.95;
    pub const STREAM_W_TO: f64 = 70.55;

    /// x brought into [−C/2, C/2).
    fn wrap_x(&self, x: f64) -> f64 {
        let c = f64::from(self.c);
        x - c * (x / c).round_ties_even()
    }

    /// The river's surface at x (its vertices carry it).
    pub fn river_w(x: f64) -> f64 {
        if x >= Self::FLATS_FROM_X {
            Self::SEA_W
        } else {
            Self::SEA_W
                + (Self::RIVER_SOURCE_W - Self::SEA_W) * (Self::FLATS_FROM_X - x)
                    / (Self::FLATS_FROM_X - Self::RIVER_FROM_X)
        }
    }

    /// The river network: the river (chain 0) and the stream (chain 1), indexed.
    pub fn rivers(&self) -> Rivers {
        let mut r = Rivers::empty(self.c);
        let vertex = |x: f64, z: f64, w: f64, hw: f64, depth: f64, q: f64| RiverVertex {
            x: x as f32,
            z: z as f32,
            w: w as f32,
            hw: hw as f32,
            depth: depth as f32,
            q: q as f32,
        };
        let n = ((Self::RIVER_TO_X - Self::RIVER_FROM_X) / Self::RIVER_SPACING).round() as u32;
        for k in 0..=n {
            let x = Self::RIVER_FROM_X + f64::from(k) * Self::RIVER_SPACING;
            r.vertices.push(vertex(
                x,
                Self::RIVER_Z,
                Self::river_w(x),
                Self::RIVER_HALF_WIDTH,
                Self::RIVER_DEPTH,
                Self::RIVER_Q,
            ));
        }
        r.chains.push(Chain {
            first: 0,
            count: n + 1,
            mouth: Mouth::Sea,
            into: u32::MAX,
            from: u32::MAX,
        });
        let first = r.vertices.len() as u32;
        let m = ((Self::STREAM_TO_Z - Self::STREAM_FROM_Z) / Self::STREAM_SPACING).round() as u32;
        for k in 0..=m {
            let f = f64::from(k) / f64::from(m);
            r.vertices.push(vertex(
                Self::STREAM_X,
                Self::STREAM_FROM_Z + f * (Self::STREAM_TO_Z - Self::STREAM_FROM_Z),
                Self::STREAM_W_FROM + f * (Self::STREAM_W_TO - Self::STREAM_W_FROM),
                Self::STREAM_HALF_WIDTH,
                Self::STREAM_DEPTH,
                Self::STREAM_Q,
            ));
        }
        // The confluence: the river's own vertex at the stream's x.
        r.vertices.push(vertex(
            Self::STREAM_X,
            Self::RIVER_Z,
            Self::river_w(Self::STREAM_X),
            Self::STREAM_HALF_WIDTH,
            Self::STREAM_DEPTH,
            Self::STREAM_Q,
        ));
        r.chains.push(Chain {
            first,
            count: m + 2,
            mouth: Mouth::River,
            into: 0,
            from: u32::MAX,
        });
        r.build_index();
        r
    }

    /// The geometry, for `manifest.json` `notes.river`.
    pub fn notes() -> serde_json::Value {
        serde_json::json!({
            "profile": "ground by x alone (periodic): plain y 70.5 for x <= -250 and x >= 400; linear to the flats' y 63.5 at x -50; flats to x 40; shallows y 61.5 (sea, one block deep) to x 100; linear to the sea floor y 45.5 at x 130; sea floor to x 300; linear back to the plain at x 400",
            "plain_y": Self::PLAIN_Y,
            "flats": {"from_x": Self::FLATS_FROM_X, "to_x": Self::SHALLOWS_FROM_X, "y": Self::FLATS_Y},
            "shallows": {"from_x": Self::SHALLOWS_FROM_X, "to_x": Self::SHORE_FROM_X, "y": Self::SHALLOWS_Y},
            "sea_w": Self::SEA_W,
            "river": {
                "chain": 0,
                "z": Self::RIVER_Z,
                "from_x": Self::RIVER_FROM_X,
                "to_x": Self::RIVER_TO_X,
                "spacing": Self::RIVER_SPACING,
                "half_width": Self::RIVER_HALF_WIDTH,
                "depth": Self::RIVER_DEPTH,
                "source_w": Self::RIVER_SOURCE_W,
                "sea_w_from_x": Self::FLATS_FROM_X,
                "lower_reach": {"from_x": -150.0, "to_x": Self::SHORE_FROM_X}
            },
            "stream": {
                "chain": 1,
                "x": Self::STREAM_X,
                "from_z": Self::STREAM_FROM_Z,
                "to_z": Self::STREAM_TO_Z,
                "half_width": Self::STREAM_HALF_WIDTH,
                "depth": Self::STREAM_DEPTH,
                "w_from": Self::STREAM_W_FROM,
                "w_to": Self::STREAM_W_TO,
                "into": "the river's vertex at x -332, z 0"
            }
        })
    }
}

impl Terrain for RiverTest {
    fn circumference(&self) -> u32 {
        self.c
    }

    fn height(&self, x: i64, _z: i64) -> f64 {
        let x = self.wrap_x(x as f64);
        let lerp = |a: f64, b: f64, x0: f64, x1: f64| a + (b - a) * (x - x0) / (x1 - x0);
        if x < Self::SLOPE_FROM_X {
            Self::PLAIN_Y
        } else if x < Self::FLATS_FROM_X {
            lerp(
                Self::PLAIN_Y,
                Self::FLATS_Y,
                Self::SLOPE_FROM_X,
                Self::FLATS_FROM_X,
            )
        } else if x < Self::SHALLOWS_FROM_X {
            Self::FLATS_Y
        } else if x < Self::SHORE_FROM_X {
            Self::SHALLOWS_Y
        } else if x < Self::SEA_FROM_X {
            lerp(
                Self::SHALLOWS_Y,
                Self::SEA_FLOOR_Y,
                Self::SHORE_FROM_X,
                Self::SEA_FROM_X,
            )
        } else if x < Self::RISE_FROM_X {
            Self::SEA_FLOOR_Y
        } else if x < Self::PLAIN_FROM_X {
            lerp(
                Self::SEA_FLOOR_Y,
                Self::PLAIN_Y,
                Self::RISE_FROM_X,
                Self::PLAIN_FROM_X,
            )
        } else {
            Self::PLAIN_Y
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn river_test_profile_and_network() {
        let t = RiverTest { c: 1024 };
        assert_eq!(t.height(-400, 0), 70.5);
        assert_eq!(t.height(-150, 9), 67.0);
        assert_eq!(t.height(0, -100), 63.5);
        assert_eq!(t.height(60, 0), 61.5);
        assert_eq!(t.height(200, 0), 45.5);
        assert_eq!(t.height(500, 0), 70.5);
        assert_eq!(t.height(-400 + 1024, 0), 70.5);
        let r = t.rivers();
        r.validate().unwrap();
        assert_eq!(r.chains.len(), 2);
        assert_eq!(r.chains[0].count, 141);
        // The river's surface: 68.9 at its source, the sea's from x -50 on.
        assert!((f64::from(r.vertices[0].w) - 68.9).abs() < 1e-5);
        assert_eq!(r.vertices[87].x, -52.0);
        assert!(r.vertices[87].w > 63.0);
        assert_eq!(r.vertices[88].w, 63.0);
        // Its columns lie in several capsules; the deepest is the one through the column.
        let s = r.sample(0, 3);
        let deep = s.deep.unwrap();
        assert!((deep.d - 3.0).abs() < 1e-9);
        assert!((deep.floor() - (63.0 - 8.0 * 0.91)).abs() < 1e-6);
        // The stream's last vertex is the river's at its x.
        let last = &r.vertices[r.vertices.len() - 1];
        assert_eq!((last.x, last.z), (-332.0, 0.0));
        assert_eq!(last.w, r.vertices[17].w);
        let s = r.sample(-332, -60);
        assert!(s.hit.is_some_and(|h| h.seg >= 141 && h.w > 70.5));
    }

    #[test]
    fn wrap_test_is_periodic_and_in_range() {
        let t = WrapTest::new(1024);
        for z in [-300i64, -256, -100, 0, 31, 255, 300] {
            for x in [-512i64, -1, 0, 128, 511] {
                let h = t.height(x, z);
                assert!((t.height(x + 1024, z) - h).abs() < 1e-9);
                assert!((-35.0..598.0).contains(&h), "H({x},{z}) = {h}");
            }
        }
        // Beyond the pole latitude holds (the range's Gaussian still uses the raw z, as in Java).
        assert_eq!(t.latitude(-256), t.latitude(-400));
        assert_eq!(
            t.raster("climate/temperature", 7, -256),
            t.raster("climate/temperature", 7, -400)
        );
        // The range peak at (C/8, C/32).
        assert!(t.height(128, 32) > 450.0);
    }

    #[test]
    fn wrap_test_matches_hand_values() {
        // H(0, 0) at C = 16384: 63 − 60 + 25·sin(0.5) + 450·g(−2048, 420)·g(−512, 260) — the
        // range term is e^(−11.89 − 1.94) ≈ 1e−6.
        let t = WrapTest::new(16384);
        let expect = 63.0 - 60.0 + 25.0 * 0.5f64.sin();
        assert!((t.height(0, 0) - expect).abs() < 1e-3);
        assert!((t.raster("climate/temperature", 0, 0) - 0.9).abs() < 1e-12);
        assert!((t.raster("climate/temperature", 0, -4096) + 0.9).abs() < 1e-12);
        assert_eq!(t.raster("climate/vegetation", 3, 3), 0.0);
    }

    #[test]
    fn lake_profile() {
        let t = LakeTest { c: 1024 };
        assert_eq!(t.height(0, 0), 110.0);
        assert_eq!(t.height(100, 0), 110.0); // in the channel
        assert_eq!(t.height(100, 10), 125.0); // channel rim crest (d = 6)
        assert_eq!(t.height(0, 50), 125.0); // lake rim crest (d = 10)
        assert_eq!(t.height(0, 200), 100.0);
        assert_eq!(t.height(170, 0), 125.0); // closed end of the channel (d = 6)
        assert_eq!(t.water_level(0, 0), Some(120.0));
        assert_eq!(t.water_level(0, 50), None);
        // The closed bowls below sea level (HYD-100): floors 40 and 30, a lake at 50 in the second.
        assert_eq!(t.height(-200, -70), 40.0);
        assert_eq!(t.height(-200, 70), 30.0);
        assert_eq!(t.height(-200, -70 + 27), 40.0 + 60.0 * 15.0 / 30.0);
        assert_eq!(t.height(-200, -20), 100.0);
        assert_eq!(t.water_level(-200, -70), None);
        assert_eq!(t.water_level(-200, 70), Some(50.0));
        assert_eq!(t.closed_basin(-200, -70), Some(true));
        assert_eq!(t.closed_basin(-200, 70 + 41), Some(true));
        assert_eq!(t.closed_basin(0, 0), Some(false));
        assert_eq!(t.height(-200 + 1024, 70), 30.0);
        // Periodic in x.
        assert_eq!(t.height(1024, 0), 110.0);
    }
}

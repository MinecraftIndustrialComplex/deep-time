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

    /// Whether the terrain writes a `floodplain` layer (HYD-152).
    fn writes_floodplain(&self) -> bool {
        false
    }

    /// The `floodplain` layer at block (x, z): the ground's move there, blocks, or `None` off the
    /// plain. Asked only where [`Terrain::writes_floodplain`] holds.
    fn floodplain(&self, _x: i64, _z: i64) -> Option<f64> {
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

/// The delta-test world (V1 gap 5, stage 1; `deeptime synth --kind delta-test`, the Java
/// `DeltaGameTests`): ground by x alone, as the bake's delta stage leaves it. A plain at y 70.5 for
/// x <= -250 and x >= 400, a slope down to the delta plain at y 63.5 (its top block 63, a block
/// over the sea's top water block 62) from x -50 to x 120, a drop to the sea floor at y 45.5 by
/// x 150, the sea floor to x 300 and the rise back. The river (chain 0) runs east along z 0 from
/// x -400 to x 160, its surface 68.9 falling to the sea's 63.0 at x -50, half-width 10 and depth 8
/// down to the apex at x 0 and its share's half-width 7 and depth 5.5 below it. Two distributaries
/// leave the river's vertex at the apex (`from` 0): chain 1 runs south-east to (64, 64) and east
/// along z 64 to x 160, chain 2 the same to the north along z -64, half-width 7 and depth 5.5, their
/// surface the sea's. A floodplain (the `floodplain` layer, HYD-152, a move of 0) covers the river's
/// reach on the upstream plain, x -400 to -270 and |z| up to 40, for the levee texture (HYD-153).
/// Every number here is a test fixture's (game-choice: agent, 2026-10-23).
pub struct DeltaTest {
    pub c: u32,
}

impl DeltaTest {
    pub const PLAIN_Y: f64 = 70.5;
    pub const DELTA_Y: f64 = 63.5;
    pub const SEA_FLOOR_Y: f64 = 45.5;
    pub const SLOPE_FROM_X: f64 = -250.0;
    pub const DELTA_FROM_X: f64 = -50.0;
    pub const FRONT_X: f64 = 120.0;
    pub const SEA_FROM_X: f64 = 150.0;
    pub const RISE_FROM_X: f64 = 300.0;
    pub const PLAIN_FROM_X: f64 = 400.0;
    pub const SEA_W: f64 = 63.0;

    pub const RIVER_FROM_X: f64 = -400.0;
    pub const RIVER_TO_X: f64 = 160.0;
    pub const SPACING: f64 = 4.0;
    pub const SOURCE_W: f64 = 68.9;
    pub const APEX_X: f64 = 0.0;
    pub const RIVER_HALF_WIDTH: f64 = 10.0;
    pub const RIVER_DEPTH: f64 = 8.0;
    pub const RIVER_Q: f64 = 600.0;
    pub const BRANCH_HALF_WIDTH: f64 = 7.0;
    pub const BRANCH_DEPTH: f64 = 5.5;
    pub const BRANCH_Q: f64 = 200.0;
    /// Where the distributaries turn east, |z| (and x).
    pub const BRANCH_Z: f64 = 64.0;
    /// The floodplain's east end (it starts at the river's source, x -400) and half-width in z.
    pub const FLOODPLAIN_TO_X: f64 = -270.0;
    pub const FLOODPLAIN_HALF_WIDTH: f64 = 40.0;

    fn wrap_x(&self, x: f64) -> f64 {
        let c = f64::from(self.c);
        x - c * (x / c).round_ties_even()
    }

    /// The river's surface at x.
    pub fn river_w(x: f64) -> f64 {
        if x >= Self::DELTA_FROM_X {
            Self::SEA_W
        } else {
            Self::SEA_W
                + (Self::SOURCE_W - Self::SEA_W) * (Self::DELTA_FROM_X - x)
                    / (Self::DELTA_FROM_X - Self::RIVER_FROM_X)
        }
    }

    /// The network: the river (chain 0) and its two distributaries (chains 1 and 2), indexed.
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
        let n = ((Self::RIVER_TO_X - Self::RIVER_FROM_X) / Self::SPACING).round() as u32;
        for k in 0..=n {
            let x = Self::RIVER_FROM_X + f64::from(k) * Self::SPACING;
            let (hw, depth, q) = if x <= Self::APEX_X {
                (Self::RIVER_HALF_WIDTH, Self::RIVER_DEPTH, Self::RIVER_Q)
            } else {
                (Self::BRANCH_HALF_WIDTH, Self::BRANCH_DEPTH, Self::BRANCH_Q)
            };
            r.vertices
                .push(vertex(x, 0.0, Self::river_w(x), hw, depth, q));
        }
        r.chains.push(Chain::new(0, n + 1, Mouth::Sea, u32::MAX));
        for side in [1.0, -1.0] {
            let first = r.vertices.len() as u32;
            // The apex vertex itself, as the parent holds it.
            r.vertices.push(vertex(
                Self::APEX_X,
                0.0,
                Self::SEA_W,
                Self::BRANCH_HALF_WIDTH,
                Self::BRANCH_DEPTH,
                Self::BRANCH_Q,
            ));
            let diag = (Self::BRANCH_Z / Self::SPACING).round() as u32;
            for k in 1..=diag {
                let d = f64::from(k) * Self::SPACING;
                r.vertices.push(vertex(
                    Self::APEX_X + d,
                    side * d,
                    Self::SEA_W,
                    Self::BRANCH_HALF_WIDTH,
                    Self::BRANCH_DEPTH,
                    Self::BRANCH_Q,
                ));
            }
            let east =
                ((Self::RIVER_TO_X - Self::APEX_X - Self::BRANCH_Z) / Self::SPACING).round() as u32;
            for k in 1..=east {
                r.vertices.push(vertex(
                    Self::APEX_X + Self::BRANCH_Z + f64::from(k) * Self::SPACING,
                    side * Self::BRANCH_Z,
                    Self::SEA_W,
                    Self::BRANCH_HALF_WIDTH,
                    Self::BRANCH_DEPTH,
                    Self::BRANCH_Q,
                ));
            }
            let mut ch = Chain::new(first, 1 + diag + east, Mouth::Sea, u32::MAX);
            ch.from = 0;
            r.chains.push(ch);
        }
        r.build_index();
        r
    }

    /// The geometry, for `manifest.json` `notes.delta`.
    pub fn notes() -> serde_json::Value {
        serde_json::json!({
            "profile": "ground by x alone (periodic): plain y 70.5 for x <= -250 and x >= 400; linear to the delta plain's y 63.5 at x -50; delta plain to x 120; linear to the sea floor y 45.5 at x 150; sea floor to x 300; linear back to the plain at x 400",
            "plain_y": Self::PLAIN_Y,
            "delta": {"from_x": Self::DELTA_FROM_X, "to_x": Self::FRONT_X, "y": Self::DELTA_Y},
            "sea_w": Self::SEA_W,
            "apex": {"x": Self::APEX_X, "z": 0.0, "vertex_of_chain_0": ((Self::APEX_X - Self::RIVER_FROM_X) / Self::SPACING).round()},
            "river": {
                "chain": 0,
                "z": 0.0,
                "from_x": Self::RIVER_FROM_X,
                "to_x": Self::RIVER_TO_X,
                "half_width": Self::RIVER_HALF_WIDTH,
                "depth": Self::RIVER_DEPTH,
                "below_apex": {"half_width": Self::BRANCH_HALF_WIDTH, "depth": Self::BRANCH_DEPTH}
            },
            "distributaries": [
                {"chain": 1, "from": 0, "turn": [Self::BRANCH_Z, Self::BRANCH_Z], "z": Self::BRANCH_Z, "to_x": Self::RIVER_TO_X},
                {"chain": 2, "from": 0, "turn": [Self::BRANCH_Z, -Self::BRANCH_Z], "z": -Self::BRANCH_Z, "to_x": Self::RIVER_TO_X}
            ],
            "branch": {"half_width": Self::BRANCH_HALF_WIDTH, "depth": Self::BRANCH_DEPTH},
            "floodplain": {"from_x": Self::RIVER_FROM_X, "to_x": Self::FLOODPLAIN_TO_X, "half_width": Self::FLOODPLAIN_HALF_WIDTH}
        })
    }
}

impl Terrain for DeltaTest {
    fn circumference(&self) -> u32 {
        self.c
    }

    fn height(&self, x: i64, _z: i64) -> f64 {
        let x = self.wrap_x(x as f64);
        let lerp = |a: f64, b: f64, x0: f64, x1: f64| a + (b - a) * (x - x0) / (x1 - x0);
        if x < Self::SLOPE_FROM_X {
            Self::PLAIN_Y
        } else if x < Self::DELTA_FROM_X {
            lerp(
                Self::PLAIN_Y,
                Self::DELTA_Y,
                Self::SLOPE_FROM_X,
                Self::DELTA_FROM_X,
            )
        } else if x < Self::FRONT_X {
            Self::DELTA_Y
        } else if x < Self::SEA_FROM_X {
            lerp(
                Self::DELTA_Y,
                Self::SEA_FLOOR_Y,
                Self::FRONT_X,
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

    fn writes_floodplain(&self) -> bool {
        true
    }

    fn floodplain(&self, x: i64, z: i64) -> Option<f64> {
        let x = self.wrap_x(x as f64);
        let on = (Self::RIVER_FROM_X..=Self::FLOODPLAIN_TO_X).contains(&x)
            && (z as f64).abs() <= Self::FLOODPLAIN_HALF_WIDTH;
        on.then_some(0.0)
    }
}

/// The meander-test world (V1 gap 5, stage 3; `deeptime synth --kind meander-test`, the Java
/// `MeanderGameTests`): ground by |x| alone, a plain at y 70.5 for |x| <= 400 falling to a sea
/// floor at y 45.5 by |x| 440. A straight river (chain 0) runs east along z 0 from x -380 into the
/// sea at x 430, half-width 3, depth 2.5 and 100 m³/s, its surface 69.9 falling to 69.1 at x 380
/// and to the sea's 63.0 at x 412, where the ground passes the sea level. A floodplain (the
/// `floodplain` layer, HYD-152, a move of 0, laid at y 70.5: its top block 70 a block over the
/// river's top water block 69) covers x -380 to 390 and |z| up to 60. The file holds the river as
/// the bake leaves it after the meanders (HYD-155 to HYD-159, [`crate::hydro::meander`]) on that
/// plain. Every number here is a test fixture's (game-choice: agent, 2026-10-24).
pub struct MeanderTest {
    pub c: u32,
}

impl MeanderTest {
    pub const PLAIN_Y: f64 = 70.5;
    pub const SEA_FLOOR_Y: f64 = 45.5;
    /// |x| where the plain starts falling and where the sea floor starts.
    pub const SLOPE_FROM_X: f64 = 400.0;
    pub const SEA_FROM_X: f64 = 440.0;
    pub const SEA_W: f64 = 63.0;

    pub const RIVER_FROM_X: f64 = -380.0;
    pub const RIVER_TO_X: f64 = 430.0;
    pub const SPACING: f64 = 2.0;
    pub const SOURCE_W: f64 = 69.9;
    /// The surface where the plain's reach ends, and that x.
    pub const PLAIN_END_W: f64 = 69.1;
    pub const PLAIN_END_X: f64 = 380.0;
    /// Where the surface reaches the sea's (the ground passes y 63 there).
    pub const SEA_W_FROM_X: f64 = 412.0;
    pub const HALF_WIDTH: f64 = 3.0;
    pub const DEPTH: f64 = 2.5;
    pub const Q: f64 = 100.0;
    pub const FLOODPLAIN_TO_X: f64 = 390.0;
    pub const FLOODPLAIN_HALF_WIDTH: f64 = 60.0;

    fn wrap_x(&self, x: f64) -> f64 {
        let c = f64::from(self.c);
        x - c * (x / c).round_ties_even()
    }

    /// The river's surface at x.
    pub fn river_w(x: f64) -> f64 {
        if x <= Self::PLAIN_END_X {
            Self::SOURCE_W
                - (Self::SOURCE_W - Self::PLAIN_END_W) * (x - Self::RIVER_FROM_X)
                    / (Self::PLAIN_END_X - Self::RIVER_FROM_X)
        } else if x < Self::SEA_W_FROM_X {
            Self::PLAIN_END_W
                - (Self::PLAIN_END_W - Self::SEA_W) * (x - Self::PLAIN_END_X)
                    / (Self::SEA_W_FROM_X - Self::PLAIN_END_X)
        } else {
            Self::SEA_W
        }
    }

    /// Whether block (x, z) is on the floodplain.
    fn on_plain(&self, x: f64, z: f64) -> bool {
        let x = self.wrap_x(x);
        (Self::RIVER_FROM_X..=Self::FLOODPLAIN_TO_X).contains(&x)
            && z.abs() <= Self::FLOODPLAIN_HALF_WIDTH
    }

    /// The river as drawn, straight along z 0, indexed.
    pub fn straight(&self) -> Rivers {
        let mut r = Rivers::empty(self.c);
        let n = ((Self::RIVER_TO_X - Self::RIVER_FROM_X) / Self::SPACING).round() as u32;
        for k in 0..=n {
            let x = Self::RIVER_FROM_X + f64::from(k) * Self::SPACING;
            r.vertices.push(RiverVertex {
                x: x as f32,
                z: 0.0,
                w: Self::river_w(x) as f32,
                hw: Self::HALF_WIDTH as f32,
                depth: Self::DEPTH as f32,
                q: Self::Q as f32,
            });
        }
        r.chains.push(Chain::new(0, n + 1, Mouth::Sea, u32::MAX));
        r.build_index();
        r
    }

    /// The river meandered on the plain, and `stats.meanders` of that pass.
    pub fn rivers(&self) -> (Rivers, serde_json::Value) {
        let mut r = self.straight();
        let level = (Self::PLAIN_Y * 16.0).round() as i32;
        let plain = |x: f64, z: f64| self.on_plain(x, z).then_some(level);
        let scale = deeptime_format::Scale::default_for(self.c);
        let stats = crate::hydro::meander::apply(
            &mut r,
            &crate::hydro::meander::Input {
                plain: &plain,
                lambert: Some(deeptime_format::Projection::of(f64::from(self.c))),
                m_per_block_h: scale.m_per_block_h,
                m_per_block_v: scale.m_per_block_v,
            },
        );
        (r, stats)
    }

    /// The geometry, for `manifest.json` `notes.meander`, with the meander pass's record.
    pub fn notes(&self, stats: &serde_json::Value) -> serde_json::Value {
        let straight = self.straight();
        serde_json::json!({
            "profile": "ground by |x| alone (periodic): plain y 70.5 for |x| <= 400; linear to the sea floor y 45.5 at |x| 440; sea floor beyond",
            "plain_y": Self::PLAIN_Y,
            "sea_w": Self::SEA_W,
            "river": {
                "chain": 0,
                "z": 0.0,
                "from_x": Self::RIVER_FROM_X,
                "to_x": Self::RIVER_TO_X,
                "half_width": Self::HALF_WIDTH,
                "depth": Self::DEPTH,
                "q": Self::Q,
                "w": {"from": Self::SOURCE_W, "plain_end": Self::PLAIN_END_W, "plain_end_x": Self::PLAIN_END_X, "sea_from_x": Self::SEA_W_FROM_X}
            },
            "floodplain": {"from_x": Self::RIVER_FROM_X, "to_x": Self::FLOODPLAIN_TO_X, "half_width": Self::FLOODPLAIN_HALF_WIDTH},
            "straight_vertices": straight.vertices.len(),
            "stats": stats
        })
    }
}

impl Terrain for MeanderTest {
    fn circumference(&self) -> u32 {
        self.c
    }

    fn height(&self, x: i64, _z: i64) -> f64 {
        let ax = self.wrap_x(x as f64).abs();
        if ax <= Self::SLOPE_FROM_X {
            Self::PLAIN_Y
        } else if ax < Self::SEA_FROM_X {
            Self::PLAIN_Y
                + (Self::SEA_FLOOR_Y - Self::PLAIN_Y) * (ax - Self::SLOPE_FROM_X)
                    / (Self::SEA_FROM_X - Self::SLOPE_FROM_X)
        } else {
            Self::SEA_FLOOR_Y
        }
    }

    fn writes_floodplain(&self) -> bool {
        true
    }

    fn floodplain(&self, x: i64, z: i64) -> Option<f64> {
        self.on_plain(x as f64, z as f64).then_some(0.0)
    }
}

/// The fan-test world (V1 gap 5, stage 4; `deeptime synth --kind fan-test`, the Java
/// `FanGameTests`): a range running north and south along x -265 (its crest y 119, its flanks
/// falling 0.5 a block) and at its foot an alluvial fan, a cone about the apex (-200, 0) falling
/// 0.1 a block of planet from y 86.5 to the plain's y 70.5 at 160 blocks of planet (the planet's
/// distance on the Lambert frame, as [`crate::hydro::fan`] measures it, so the cone is round on
/// the planet). The plain at y 70.5 for |x| <= 400 falls to a sea floor at y 45.5 by |x| 440, as
/// on the meander-test world. A river (chain 0) leaves the range at the apex and runs east along
/// z 0 down the cone, across the plain and into the sea at x 430: half-width 4, depth 2 and
/// 50 m³/s, its surface 1.25 blocks under the cone's, then 69.25 at the fan's toe (x -40) falling
/// to 69.1 at x 380 and to the sea's 63.0 at x 412. The file holds the river as the bake leaves it
/// after the braids (HYD-171 to HYD-175, [`crate::hydro::braid`]) on the fan: the course and a
/// thread around each island. Every number here is a test fixture's (game-choice: agent,
/// 2026-10-24).
pub struct FanTest {
    pub c: u32,
}

impl FanTest {
    pub const PLAIN_Y: f64 = 70.5;
    pub const SEA_FLOOR_Y: f64 = 45.5;
    /// |x| where the plain starts falling and where the sea floor starts.
    pub const SLOPE_FROM_X: f64 = 400.0;
    pub const SEA_FROM_X: f64 = 440.0;
    pub const SEA_W: f64 = 63.0;

    /// The range's crest line, its height and its flanks' fall per block.
    pub const RIDGE_X: f64 = -265.0;
    pub const RIDGE_TOP_Y: f64 = 119.0;
    pub const RIDGE_FALL: f64 = 0.5;

    /// The fan's apex (at the range's foot), its height, the cone's fall per block of planet and
    /// its radius (blocks of planet; the cone meets the plain there).
    pub const APEX_X: f64 = -200.0;
    pub const APEX_Z: f64 = 0.0;
    pub const APEX_Y: f64 = 86.5;
    pub const FAN_FALL: f64 = 0.1;
    pub const FAN_RADIUS: f64 = 160.0;

    pub const RIVER_TO_X: f64 = 430.0;
    pub const SPACING: f64 = 2.0;
    /// The river's surface under the cone's, blocks.
    pub const UNDER_CONE: f64 = 1.25;
    /// The surface at the fan's toe, where the plain's reach ends and that x.
    pub const TOE_W: f64 = 69.25;
    pub const PLAIN_END_W: f64 = 69.1;
    pub const PLAIN_END_X: f64 = 380.0;
    /// Where the surface reaches the sea's (the ground passes y 63 there).
    pub const SEA_W_FROM_X: f64 = 412.0;
    pub const HALF_WIDTH: f64 = 4.0;
    pub const DEPTH: f64 = 2.0;
    pub const Q: f64 = 50.0;

    /// The x of the fan's toe on the river's line.
    pub fn toe_x() -> f64 {
        Self::APEX_X + Self::FAN_RADIUS
    }

    fn wrap_x(&self, x: f64) -> f64 {
        let c = f64::from(self.c);
        x - c * (x / c).round_ties_even()
    }

    fn projection(&self) -> deeptime_format::Projection {
        deeptime_format::Projection::of(f64::from(self.c))
    }

    /// The planet's distance from the apex to block (x, z), blocks of planet: the Lambert metric
    /// at the middle latitude, as [`crate::hydro::fan`] takes it.
    pub fn apex_dist(&self, x: f64, z: f64) -> f64 {
        let cs = self.projection().cos_lat(0.5 * (Self::APEX_Z + z)).max(0.05);
        (self.wrap_x(x - Self::APEX_X) * cs).hypot((z - Self::APEX_Z) / cs)
    }

    /// The cone's height at block (x, z) (below the plain past its radius).
    pub fn cone(&self, x: f64, z: f64) -> f64 {
        Self::APEX_Y - Self::FAN_FALL * self.apex_dist(x, z)
    }

    fn ridge(&self, x: f64) -> f64 {
        Self::RIDGE_TOP_Y - Self::RIDGE_FALL * (self.wrap_x(x) - Self::RIDGE_X).abs()
    }

    fn base(&self, x: f64) -> f64 {
        let ax = self.wrap_x(x).abs();
        if ax <= Self::SLOPE_FROM_X {
            Self::PLAIN_Y
        } else if ax < Self::SEA_FROM_X {
            Self::PLAIN_Y
                + (Self::SEA_FLOOR_Y - Self::PLAIN_Y) * (ax - Self::SLOPE_FROM_X)
                    / (Self::SEA_FROM_X - Self::SLOPE_FROM_X)
        } else {
            Self::SEA_FLOOR_Y
        }
    }

    /// Whether block (x, z) is on the fan: the cone stands over the plain and the range there.
    pub fn on_fan(&self, x: f64, z: f64) -> bool {
        let h = self.cone(x, z);
        h > Self::PLAIN_Y && h > self.ridge(x)
    }

    /// The river's surface at x.
    pub fn river_w(x: f64) -> f64 {
        let toe = Self::toe_x();
        if x <= toe {
            Self::APEX_Y - Self::FAN_FALL * (x - Self::APEX_X) - Self::UNDER_CONE
        } else if x <= Self::PLAIN_END_X {
            Self::TOE_W - (Self::TOE_W - Self::PLAIN_END_W) * (x - toe) / (Self::PLAIN_END_X - toe)
        } else if x < Self::SEA_W_FROM_X {
            Self::PLAIN_END_W
                - (Self::PLAIN_END_W - Self::SEA_W) * (x - Self::PLAIN_END_X)
                    / (Self::SEA_W_FROM_X - Self::PLAIN_END_X)
        } else {
            Self::SEA_W
        }
    }

    /// The river as drawn, straight along z 0 from the apex, indexed.
    pub fn straight(&self) -> Rivers {
        let mut r = Rivers::empty(self.c);
        let n = ((Self::RIVER_TO_X - Self::APEX_X) / Self::SPACING).round() as u32;
        for k in 0..=n {
            let x = Self::APEX_X + f64::from(k) * Self::SPACING;
            r.vertices.push(RiverVertex {
                x: x as f32,
                z: 0.0,
                w: Self::river_w(x) as f32,
                hw: Self::HALF_WIDTH as f32,
                depth: Self::DEPTH as f32,
                q: Self::Q as f32,
            });
        }
        r.chains.push(Chain::new(0, n + 1, Mouth::Sea, u32::MAX));
        r.build_index();
        r
    }

    /// The river braided on the fan, and `stats.braids` of that pass.
    pub fn rivers(&self) -> (Rivers, serde_json::Value) {
        let mut r = self.straight();
        let plain = |_: f64, _: f64| None;
        let fan = |x: f64, z: f64| self.on_fan(x, z);
        let scale = deeptime_format::Scale::default_for(self.c);
        let stats = crate::hydro::braid::apply(
            &mut r,
            &crate::hydro::braid::Input {
                plain: &plain,
                fan: &fan,
                lambert: Some(self.projection()),
                m_per_block_h: scale.m_per_block_h,
                m_per_block_v: scale.m_per_block_v,
            },
        );
        (r, stats)
    }

    /// Columns on the fan every 10 blocks, with the cone's height there: `[x, z, height]`.
    pub fn cone_samples(&self) -> Vec<[f64; 3]> {
        let mut out = Vec::new();
        for i in 0..=16 {
            let x = Self::APEX_X + 10.0 + 10.0 * f64::from(i);
            for j in -16..=16 {
                let z = 10.0 * f64::from(j);
                if self.on_fan(x, z) && self.cone(x, z) > Self::PLAIN_Y + 0.5 {
                    out.push([x, z, self.height(x as i64, z as i64)]);
                }
            }
        }
        out
    }

    /// The geometry, for `manifest.json` `notes.fan`, with the braid pass's record.
    pub fn notes(&self, stats: &serde_json::Value) -> serde_json::Value {
        let straight = self.straight();
        serde_json::json!({
            "profile": "plain y 70.5 for |x| <= 400 (periodic), linear to the sea floor y 45.5 at |x| 440; a range along x -265 (crest y 119, flanks 0.5 a block); a cone about (-200, 0) from y 86.5 falling 0.1 a block of planet to the plain at 160 blocks of planet; the ground is the highest of the three",
            "plain_y": Self::PLAIN_Y,
            "sea_w": Self::SEA_W,
            "ridge": {"x": Self::RIDGE_X, "top_y": Self::RIDGE_TOP_Y, "fall": Self::RIDGE_FALL},
            "fan": {
                "apex_x": Self::APEX_X,
                "apex_z": Self::APEX_Z,
                "apex_y": Self::APEX_Y,
                "fall": Self::FAN_FALL,
                "radius": Self::FAN_RADIUS,
                "toe_x": Self::toe_x()
            },
            "river": {
                "chain": 0,
                "z": 0.0,
                "from_x": Self::APEX_X,
                "to_x": Self::RIVER_TO_X,
                "half_width": Self::HALF_WIDTH,
                "depth": Self::DEPTH,
                "q": Self::Q,
                "w": {"under_cone": Self::UNDER_CONE, "toe": Self::TOE_W, "plain_end": Self::PLAIN_END_W, "plain_end_x": Self::PLAIN_END_X, "sea_from_x": Self::SEA_W_FROM_X}
            },
            "cone_samples": self.cone_samples(),
            "straight_vertices": straight.vertices.len(),
            "stats": stats
        })
    }
}

impl Terrain for FanTest {
    fn circumference(&self) -> u32 {
        self.c
    }

    fn height(&self, x: i64, z: i64) -> f64 {
        let (x, z) = (x as f64, z as f64);
        self.base(x).max(self.ridge(x)).max(self.cone(x, z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_test_profile_and_network() {
        let t = DeltaTest { c: 1024 };
        assert_eq!(t.height(-400, 0), 70.5);
        assert_eq!(t.height(0, 50), 63.5);
        assert_eq!(t.height(119, -90), 63.5);
        assert_eq!(t.height(200, 0), 45.5);
        let r = t.rivers();
        r.validate().unwrap();
        assert_eq!(r.chains.len(), 3);
        assert_eq!(r.chains[0].from, u32::MAX);
        for k in [1, 2] {
            let ch = r.chains[k];
            assert_eq!(ch.from, 0);
            let v = r.vertices[ch.first as usize];
            assert_eq!((v.x, v.z), (0.0, 0.0));
            assert_eq!(r.vertices[100].x, 0.0);
            let last = r.vertices[(ch.first + ch.count - 1) as usize];
            assert_eq!(last.x, 160.0);
            assert_eq!(last.z.abs(), 64.0);
        }
    }

    #[test]
    fn meander_test_profile_and_network() {
        let t = MeanderTest { c: 1024 };
        assert_eq!(t.height(0, 50), 70.5);
        assert_eq!(t.height(-400, 0), 70.5);
        assert_eq!(t.height(420, 0), 58.0);
        assert_eq!(t.height(500, 0), 45.5);
        assert_eq!(t.floodplain(0, 60), Some(0.0));
        assert_eq!(t.floodplain(395, 0), None);
        assert!((MeanderTest::river_w(412.0) - 63.0).abs() < 1e-12);
        let straight = t.straight();
        let (r, stats) = t.rivers();
        r.validate().unwrap();
        assert_eq!(r.chains.len(), 1);
        let v = &r.vertices;
        assert!(v.len() > straight.vertices.len(), "{stats}");
        assert_eq!(v[0], straight.vertices[0]);
        assert_eq!(
            v[v.len() - 1],
            straight.vertices[straight.vertices.len() - 1]
        );
        assert!(v.windows(2).all(|p| p[1].w <= p[0].w));
        let off = v.iter().fold(0.0f32, |m, p| m.max(p.z.abs()));
        assert!(
            (15.0..=MeanderTest::FLOODPLAIN_HALF_WIDTH as f32 - 3.0).contains(&off),
            "{off}: {stats}"
        );
        let k = stats["reach_sinuosity"].as_f64().unwrap();
        assert!(k >= 1.5, "{stats}");
        // Off the plain, down the slope into the sea, the course is the straight one.
        assert!(v.iter().filter(|p| p.x > 392.0).all(|p| p.z == 0.0));
    }

    #[test]
    fn fan_test_profile_and_network() {
        let t = FanTest { c: 2048 };
        assert_eq!(t.height(-265, 0), 119.0);
        assert_eq!(t.height(-200, 0), 86.5);
        assert!((t.height(-100, 0) - 76.5).abs() < 1e-9);
        assert_eq!(t.height(0, 0), 70.5);
        assert_eq!(t.height(500, 0), 45.5);
        assert!(t.on_fan(-150.0, 30.0) && !t.on_fan(-210.0, 0.0) && !t.on_fan(-30.0, 0.0));
        // The cone is round on the planet: 100 blocks of planet east and south stand alike.
        let p = deeptime_format::Projection::of(2048.0);
        let mut dz = 100.0;
        for _ in 0..20 {
            dz = 100.0 * p.cos_lat(0.5 * dz);
        }
        assert!((t.cone(-100.0, 0.0) - t.cone(-200.0, dz)).abs() < 1e-9);
        assert!((FanTest::river_w(FanTest::toe_x()) - FanTest::TOE_W).abs() < 1e-12);
        assert!((FanTest::river_w(412.0) - 63.0).abs() < 1e-12);
        let straight = t.straight();
        let (r, stats) = t.rivers();
        r.validate().unwrap();
        let threads: Vec<usize> = (1..r.chains.len()).collect();
        assert!(threads.len() >= 2, "{stats}");
        assert_eq!(r.chains[0].count, straight.chains[0].count);
        let mut sides = Vec::new();
        for &k in &threads {
            let ch = r.chains[k];
            assert_eq!((ch.from, ch.into, ch.mouth), (0, 0, Mouth::River));
            let v = &r.vertices[ch.first as usize..(ch.first + ch.count) as usize];
            assert!(v.windows(2).all(|p| p[1].w <= p[0].w));
            // On the fan, around an island, and the ground over the thread's water.
            assert!(v.iter().all(|q| t.on_fan(f64::from(q.x), f64::from(q.z))));
            let far = v.iter().fold(0.0f32, |m, q| if q.z.abs() > m.abs() { q.z } else { m });
            assert!((10.0..=25.0).contains(&far.abs()), "{far}: {stats}");
            for q in v {
                let ground = t.height(q.x.round() as i64, q.z.round() as i64);
                assert!(ground - f64::from(q.w) > 0.75, "{q:?}: ground {ground}");
            }
            sides.push(far.signum());
        }
        assert!(sides.windows(2).all(|p| p[0] != p[1]));
        assert!(!t.cone_samples().is_empty());
    }

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

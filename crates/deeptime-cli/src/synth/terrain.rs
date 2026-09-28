// SPDX-License-Identifier: LGPL-3.0-or-later
//! The synthetic terrains. `WrapTest` is the Java `AnalyticTestTerrain` (plan §7.1) transcribed
//! operation for operation, so world data written from it matches what the generator produces from
//! the closed form (P0-10 compares the two). Keep the two in step: a change to one is a change to
//! both.

use std::f64::consts::PI;

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

    /// Latitude φ = πz/(C/2), clamped to ±π/2 beyond the poles (`AnalyticTestTerrain.latitude`).
    fn latitude(&self, z: i64) -> f64 {
        let c = f64::from(self.circumference());
        let phi = PI * z as f64 / (c / 2.0);
        (-PI / 2.0).max((PI / 2.0).min(phi))
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
pub struct LakeTest {
    pub c: u32,
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
            "water": "water_level = 120 on every cell with d ≤ 4 (inside the rim crest), nodata elsewhere; water belongs only where it is above the terrain (d < 2.67)"
        })
    }
}

impl Terrain for LakeTest {
    fn circumference(&self) -> u32 {
        self.c
    }

    fn height(&self, x: i64, z: i64) -> f64 {
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
        (self.footprint_distance(x, z) <= Self::WALL).then_some(LAKE_WATER_Y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Periodic in x.
        assert_eq!(t.height(1024, 0), 110.0);
    }
}

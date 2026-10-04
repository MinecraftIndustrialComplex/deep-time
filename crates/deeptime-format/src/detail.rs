// SPDX-License-Identifier: LGPL-3.0-or-later
//! The sub-cell detail term of the terrain (FORMAT.md, "The detail layer" and "The terrain
//! height"): the game adds **A(x, z) · N(x, z)** to the L3 `elevation`, where A is the baked layer
//! `detail/amplitude` and N the value noise here. Both sides of the contract evaluate N:
//! `deeptime_core::surface::detail` bakes A with it (P1-06) and re-exports it, the Java reader has
//! its twin (`worldgen/DetailNoise`, P1-08), and the column evaluator (`column.rs`) sums the two.
//!
//! N is exactly specified so the twins agree bit for bit: IEEE-754 double arithmetic with
//! `+ − × /`, `floor` and `round` only (no `sin`, no library `pow`), integers hashed with
//! splitmix64's finaliser. Value noise of wavelength λ (blocks): n = max(1, round(C/λ)) lattice
//! cells around the planet, spacing s = C/n; u = (x + C/2)/s, v = z/s; the four lattice corners
//! (⌊u⌋ mod n, ⌊v⌋) … (+1, +1) (the column index mod n, so the lattice is periodic in x) get
//! [`Noise::lattice`] values in [−1, 1]; the result blends them bilinearly with the fade
//! t = f²(3 − 2f) on each axis. [`Noise::fbm`] sums `octaves` of them at λ, λ/2, λ/4, … with
//! weights 1, ½, ¼, … normalised to [−1, 1].
//!
//! Which noise a world uses (seed, salt, octaves, wavelength) is `manifest.json`
//! `notes.bake.detail` ([`DetailSpec`]), written by the bake.

use serde::{Deserialize, Serialize};

use crate::error::{FormatError, Result};
use crate::manifest::Manifest;

/// splitmix64's finaliser.
#[inline]
fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// The width of the band inside a Petroff–Guyou world's north and south edges where the detail
/// noise is blended towards its half-turn-symmetric mean, blocks ([`DetailSpec::noise`]).
pub const EDGE_BLEND_BLOCKS: f64 = 32.0; // ledger: SCL-089 (game-choice)

/// Periodic value noise (see the module documentation for the exact definition).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Noise {
    pub seed: u64,
    /// C, blocks: the period in x.
    pub circumference: u32,
    /// b of a Petroff–Guyou world (format 0.5): the north and south edges z = ∓b/2 are glued onto
    /// themselves by half-turns, and [`DetailSpec::noise`] folds and blends there. `None`: a
    /// Lambert world or a synthetic one.
    pub half_turn_b: Option<i64>,
}

impl Noise {
    pub fn new(seed: u64, circumference: u32) -> Noise {
        Noise {
            seed,
            circumference,
            half_turn_b: None,
        }
    }

    /// The noise of a world laid out by `wp`: a Petroff–Guyou world's has its half-turn edges.
    pub fn for_world(seed: u64, circumference: u32, wp: &crate::projection::WorldProjection) -> Noise {
        let mut n = Noise::new(seed, circumference);
        if let crate::projection::WorldProjection::PetroffGuyou(g) = wp {
            n.half_turn_b = Some(g.hemisphere());
        }
        n
    }

    /// The lattice value at (ix, jz) for `salt`, in [−1, 1].
    #[inline]
    pub fn lattice(&self, salt: u32, ix: i64, jz: i64) -> f64 {
        let a = mix(self.seed ^ u64::from(salt).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let b = mix(a ^ (ix as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93));
        let h = mix(b ^ (jz as u64).wrapping_mul(0xA076_1D64_78BD_642F));
        ((h >> 11) as f64) * (2.0 / 9_007_199_254_740_992.0) - 1.0
    }

    /// Value noise of wavelength `wavelength` blocks at block (x, z), in [−1, 1], periodic in x
    /// with period C.
    pub fn value(&self, x: f64, z: f64, wavelength: f64, salt: u32) -> f64 {
        let c = f64::from(self.circumference);
        let n = (c / wavelength).round().max(1.0);
        let s = c / n;
        let u = (x + c / 2.0) / s;
        let v = z / s;
        let (fu, fv) = (u.floor(), v.floor());
        let (fx, fz) = (u - fu, v - fv);
        let n_i = n as i64;
        let ix0 = (fu as i64).rem_euclid(n_i);
        let ix1 = (ix0 + 1) % n_i;
        let jz0 = fv as i64;
        let jz1 = jz0 + 1;
        let tx = fx * fx * (3.0 - 2.0 * fx);
        let tz = fz * fz * (3.0 - 2.0 * fz);
        let h00 = self.lattice(salt, ix0, jz0);
        let h10 = self.lattice(salt, ix1, jz0);
        let h01 = self.lattice(salt, ix0, jz1);
        let h11 = self.lattice(salt, ix1, jz1);
        let a = h00 + (h10 - h00) * tx;
        let b = h01 + (h11 - h01) * tx;
        a + (b - a) * tz
    }

    /// `octaves` of [`Noise::value`] at λ, λ/2, λ/4, … with weights 1, ½, ¼, … (salt + octave),
    /// normalised to [−1, 1].
    pub fn fbm(&self, x: f64, z: f64, wavelength: f64, octaves: u32, salt: u32) -> f64 {
        let (mut sum, mut norm, mut weight, mut wl) = (0.0, 0.0, 1.0, wavelength);
        for o in 0..octaves {
            sum += weight * self.value(x, z, wl, salt.wrapping_add(o));
            norm += weight;
            weight *= 0.5;
            wl *= 0.5;
        }
        sum / norm
    }
}

/// The `notes` key the bake writes its record under (`manifest.json` `notes.bake`).
pub const BAKE_NOTES: &str = "bake";

/// The game's detail term of a baked world (`notes.bake.detail`): N = [`Noise::fbm`] of
/// `Noise::new(seed, C)` at `wavelength_blocks`, `octaves`, `salt`, times the `layer` sampled
/// bilinearly. The seed is written as a decimal string: JSON readers that hold numbers as doubles
/// (and Java's `long`) cannot carry every u64.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DetailSpec {
    /// The amplitude layer (`detail/amplitude`).
    pub layer: String,
    /// The noise seed, a decimal u64.
    pub seed: String,
    pub salt: u32,
    pub octaves: u32,
    pub wavelength_blocks: f64,
}

impl DetailSpec {
    pub fn new(seed: u64, salt: u32, octaves: u32, wavelength_blocks: f64) -> DetailSpec {
        DetailSpec {
            layer: "detail/amplitude".into(),
            seed: seed.to_string(),
            salt,
            octaves,
            wavelength_blocks,
        }
    }

    /// The seed as a number.
    pub fn seed(&self) -> Result<u64> {
        self.seed
            .parse()
            .map_err(|_| FormatError::Invalid(format!("notes.bake.detail.seed {:?}", self.seed)))
    }

    /// Checks the values a reader relies on.
    pub fn validate(&self) -> Result<()> {
        self.seed()?;
        if !(1..=16).contains(&self.octaves)
            || !(self.wavelength_blocks.is_finite() && self.wavelength_blocks > 0.0)
        {
            return Err(FormatError::Invalid(format!(
                "notes.bake.detail: octaves {} (1–16), wavelength {} (> 0)",
                self.octaves, self.wavelength_blocks
            )));
        }
        Ok(())
    }

    /// The spec a manifest carries, if any (a synthetic world has none: its terrain is the
    /// `elevation` layer alone).
    pub fn of(m: &Manifest) -> Result<Option<DetailSpec>> {
        let Some(v) = m.notes.get(BAKE_NOTES).and_then(|b| b.get("detail")) else {
            return Ok(None);
        };
        let spec: DetailSpec =
            serde_json::from_value(v.clone()).map_err(|source| FormatError::Json {
                what: "manifest.json notes.bake.detail".into(),
                source,
            })?;
        spec.validate()?;
        Ok(Some(spec))
    }

    /// N at block (x, z) for a planet of circumference `c`.
    ///
    /// On a Petroff–Guyou world (`noise.half_turn_b` = b, FORMAT.md "The detail layer") a point past
    /// the north or south edge is first folded through the edge's half-turn ((x, z) → (−x, −b − z)
    /// north, (−x, b − z) south, repeated), so the terrain past an edge is the glued inside. Within
    /// [`EDGE_BLEND_BLOCKS`] of an edge (d = z + b/2 north of the equator, b/2 − z south) the noise
    /// is blended towards its half-turn-symmetric mean S = ½(N(x, z) + N(m)) with m the point's
    /// mirror past the edge, (−x, −b − z) or (−x, b − z): t = d / 32, w = t²(3 − 2t), result
    /// S + (N − S)·w. On the edge (d = 0) the result is S, which the half-turn maps to itself, so the
    /// detail term is continuous through the seam.
    pub fn noise(&self, noise: &Noise, x: f64, z: f64) -> f64 {
        let Some(b) = noise.half_turn_b else {
            return noise.fbm(x, z, self.wavelength_blocks, self.octaves, self.salt);
        };
        let b = b as f64;
        let h = 0.5 * b;
        let (mut x, mut z) = (x, z);
        for _ in 0..64 {
            if z < -h {
                x = -x;
                z = -b - z;
            } else if z > h {
                x = -x;
                z = b - z;
            } else {
                break;
            }
        }
        let n = noise.fbm(x, z, self.wavelength_blocks, self.octaves, self.salt);
        let (d, mz) = if z < 0.0 { (z + h, -b - z) } else { (h - z, b - z) };
        if d >= EDGE_BLEND_BLOCKS {
            return n;
        }
        let m = noise.fbm(-x, mz, self.wavelength_blocks, self.octaves, self.salt);
        let s = 0.5 * (n + m);
        let t = d / EDGE_BLEND_BLOCKS;
        let w = t * t * (3.0 - 2.0 * t);
        s + (n - s) * w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_noise_is_periodic_in_x_and_bounded() {
        for c in [16_384u32, 32_768, 4096, 1000] {
            let n = Noise::new(7, c);
            let cf = f64::from(c);
            for k in 0..500 {
                let x = -cf / 2.0 + (k as f64) * 13.37 % cf;
                let z = -cf / 4.0 + (k as f64) * 7.91 % (cf / 2.0);
                for wl in [4.0, 8.0, 32.0, 100.0] {
                    let a = n.value(x, z, wl, 3);
                    assert!((-1.0..=1.0).contains(&a));
                    // A lap changes the input's low bits, so equal to rounding, not to the bit.
                    let east = n.value(x + cf, z, wl, 3);
                    let west = n.value(x - 2.0 * cf, z, wl, 3);
                    assert!((a - east).abs() < 1e-9, "one lap east, wl {wl}: {a} {east}");
                    assert!((a - west).abs() < 1e-9, "two laps west: {a} {west}");
                }
                let f = n.fbm(x, z, 32.0, 3, 1);
                assert!((-1.0..=1.0).contains(&f));
                assert!((f - n.fbm(x + cf, z, 32.0, 3, 1)).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn the_seam_is_continuous() {
        let c = 16_384.0;
        let n = Noise::new(1, 16_384);
        // The seam is at x = ±C/2: a step of a tenth of a block across it changes the value like
        // a step anywhere else does.
        let mut worst_seam = 0.0f64;
        let mut worst_else = 0.0f64;
        for k in 0..200 {
            let z = -3000.0 + f64::from(k) * 29.3;
            worst_seam = worst_seam.max(
                (n.fbm(c / 2.0 - 0.05, z, 32.0, 3, 1) - n.fbm(-c / 2.0 + 0.05, z, 32.0, 3, 1))
                    .abs(),
            );
            worst_else = worst_else.max(
                (n.fbm(1000.0 - 0.05, z, 32.0, 3, 1) - n.fbm(1000.0 + 0.05, z, 32.0, 3, 1)).abs(),
            );
        }
        assert!(
            worst_seam < 0.05 && worst_else < 0.05,
            "{worst_seam} {worst_else}"
        );
    }

    #[test]
    fn noise_is_deterministic_and_seed_dependent() {
        let a = Noise::new(1, 1024);
        let b = Noise::new(2, 1024);
        let v = a.value(10.5, 20.25, 16.0, 0);
        assert_eq!(v, Noise::new(1, 1024).value(10.5, 20.25, 16.0, 0));
        assert_ne!(v, b.value(10.5, 20.25, 16.0, 0));
        assert_ne!(v, a.value(10.5, 20.25, 16.0, 1), "salt");
        // A lattice point returns the lattice value exactly.
        let s = 1024.0 / (1024.0f64 / 16.0).round();
        assert_eq!(
            a.value(-512.0 + 3.0 * s, 5.0 * s, 16.0, 0),
            a.lattice(0, 3, 5)
        );
    }

    /// Fixed values the Java twin (`DetailNoiseTest`) checks too: a change here is a change of the
    /// world-data contract.
    #[test]
    fn known_values() {
        let n = Noise::new(0xDEAD_BEEF_0123_4567, 16_384);
        assert_eq!(n.lattice(7, -3, 11), -0.9207152765116788);
        assert_eq!(n.fbm(-8191.0, 4095.0, 32.0, 3, 1729), -0.06020044448424485);
        assert_eq!(n.fbm(1234.0, -567.0, 32.0, 3, 1729), 0.48318066841072793);
        let seed = u64::MAX - 5;
        let spec = DetailSpec::new(seed, 1729, 3, 32.0);
        assert_eq!(spec.seed().unwrap(), seed);
        let json = serde_json::to_value(&spec).unwrap();
        assert_eq!(json["seed"], "18446744073709551610");
    }

    #[test]
    fn the_noise_is_continuous_through_the_half_turn_edges() {
        let b = 6_656i64;
        let n = Noise {
            half_turn_b: Some(b),
            ..Noise::new(3, 13_312)
        };
        let spec = DetailSpec::new(3, 1729, 3, 32.0);
        let h = b as f64 / 2.0;
        for k in 0..400 {
            let x = -6_656.0 + 33.27 * f64::from(k);
            // On each edge the two images (x, ∓h) and (−x, ∓h) of one planet point agree.
            for e in [-h, h] {
                let (a, c) = (spec.noise(&n, x, e), spec.noise(&n, -x, e));
                assert!((a - c).abs() < 1e-12, "edge {e} at x {x}: {a} vs {c}");
            }
            // Past the edge is the glued inside: (x, −h − d) is (−x, −h + d).
            for d in [0.25, 3.0, 17.0, 40.0] {
                let out = spec.noise(&n, x, -h - d);
                let inside = spec.noise(&n, -x, -h + d);
                assert!((out - inside).abs() < 1e-12);
                let out = spec.noise(&n, x, h + d);
                let inside = spec.noise(&n, -x, h - d);
                assert!((out - inside).abs() < 1e-12);
            }
            // Away from the edges the plain noise.
            assert_eq!(spec.noise(&n, x, 0.0), n.fbm(x, 0.0, 32.0, 3, 1729));
        }
    }
}

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

/// Periodic value noise (see the module documentation for the exact definition).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Noise {
    pub seed: u64,
    /// C, blocks: the period in x.
    pub circumference: u32,
}

impl Noise {
    pub fn new(seed: u64, circumference: u32) -> Noise {
        Noise {
            seed,
            circumference,
        }
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
    pub fn noise(&self, noise: &Noise, x: f64, z: f64) -> f64 {
        noise.fbm(x, z, self.wavelength_blocks, self.octaves, self.salt)
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
}

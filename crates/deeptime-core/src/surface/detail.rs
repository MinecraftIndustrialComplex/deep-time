// SPDX-License-Identifier: LGPL-3.0-or-later
//! The sub-cell detail layer (plan §8.2, P1-06) and the noise the amplifier shares with it.
//!
//! # What the game adds below L3
//! `deeptime:terrain` (plan §7.1) adds to the L3 height H(x, z) a term **A(x, z) · N(x, z)**:
//! N is [`Noise::fbm`], a value noise **periodic in x with period C** (the world wraps, so a
//! detail term with any other period would show a seam at x = ±C/2), and A the baked layer
//! `detail/amplitude` (u8, 1/16 block per step, L3 grid, bilinear at chunk time): the local relief
//! times the drama slider, faded to zero along channels and on valley floors, so rivers and plains
//! stay smooth and mountains get rougher — and drama 0 removes the term altogether.
//!
//! **The layer** (this module, `amplitude_blocks`): with `relief` the range of the L3 heights over
//! the 5 × 5 cells around a cell (in blocks) and `area` its effective drainage area,
//!
//! A = min(drama · relief_fraction · relief, max_blocks · drama) · (1 − smoothstep(a₀, a₁, area_km²))
//!
//! where smoothstep is the cubic 3t² − 2t³ on the clamped t = (area − a₀)/(a₁ − a₀).
//!
//! **The noise** ([`Noise`]) is exactly specified so the Java twin (`worldgen/DetailNoise`, P1-08)
//! reproduces it: everything is IEEE-754 double arithmetic with `+ − × /`, `floor` and `round`
//! only (no `sin`, no library `pow`), and integers hash with splitmix64's finaliser. Value noise
//! of wavelength λ (blocks): n = max(1, round(C/λ)) lattice cells around the planet, spacing
//! s = C/n; u = (x + C/2)/s, v = z/s; the four lattice corners (⌊u⌋ mod n, ⌊v⌋) … (+1, +1)
//! (the column index mod n, so the lattice is periodic in x) get `lattice` values in [−1, 1]; the
//! result blends them bilinearly with the fade t = f²(3 − 2f) on each axis. `fbm` sums `octaves`
//! of them at λ, λ/2, λ/4, … with weights 1, ½, ¼, … normalised to [−1, 1].

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// Blocks per step of the `detail/amplitude` u8 layer.
pub const AMPLITUDE_STEP_BLOCKS: f64 = 1.0 / 16.0;
/// The largest amplitude the u8 layer holds, blocks.
pub const AMPLITUDE_MAX_BLOCKS: f64 = 255.0 * AMPLITUDE_STEP_BLOCKS;

/// The `amplify.detail` section.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct DetailParams {
    /// Amplitude as a fraction of the 5 × 5-cell relief (at drama 1).
    pub relief_fraction: f64,
    /// Largest amplitude at drama 1, blocks (drama 2 doubles it; the u8 layer holds 15.9).
    pub max_blocks: f64,
    /// The noise fades out between these effective drainage areas, km² (rivers stay smooth).
    pub channel_area_km2: [f64; 2],
    /// Octaves of the noise the game adds (stored for the game's `deeptime:terrain`).
    pub octaves: u32,
    /// Longest wavelength of the noise, blocks. Minecraft's noise cell is 4 blocks, so waves
    /// under 8 blocks cannot be drawn; each octave halves it.
    pub wavelength_blocks: f64,
}

impl Default for DetailParams {
    fn default() -> Self {
        DetailParams {
            relief_fraction: 0.12,
            max_blocks: 6.0,
            channel_area_km2: [25.0, 250.0],
            octaves: 3,
            wavelength_blocks: 32.0,
        }
    }
}

impl DetailParams {
    pub fn validate(&self, path: &str) -> Result<(), CoreError> {
        let bad = |k: &str, why: &str| CoreError::preset(format!("{path}.{k}"), why);
        if !(self.relief_fraction.is_finite() && (0.0..=1.0).contains(&self.relief_fraction)) {
            return Err(bad("relief_fraction", "must be in [0, 1]"));
        }
        if !(self.max_blocks.is_finite()
            && self.max_blocks > 0.0
            && 2.0 * self.max_blocks <= AMPLITUDE_MAX_BLOCKS)
        {
            return Err(bad(
                "max_blocks",
                "must be in (0, 7.9] (drama 2 doubles it and the u8 layer holds 15.9 blocks)",
            ));
        }
        let [a0, a1] = self.channel_area_km2;
        if !(a0.is_finite() && a1.is_finite() && a0 > 0.0 && a1 > a0) {
            return Err(bad("channel_area_km2", "[a0, a1] with 0 < a0 < a1 (km²)"));
        }
        if !(1..=6).contains(&self.octaves) {
            return Err(bad("octaves", "must be in [1, 6]"));
        }
        if !(self.wavelength_blocks.is_finite() && self.wavelength_blocks >= 4.0) {
            return Err(bad("wavelength_blocks", "must be ≥ 4"));
        }
        Ok(())
    }

    /// The amplitude, blocks, of a cell with `relief_blocks` of local relief and `area_m2` of
    /// effective drainage area, at slider `drama` (0..=2).
    pub fn amplitude_blocks(&self, relief_blocks: f64, area_m2: f64, drama: f64) -> f64 {
        let a = (drama * self.relief_fraction * relief_blocks).min(self.max_blocks * drama);
        let [a0, a1] = self.channel_area_km2;
        let t = ((area_m2 * 1.0e-6 - a0) / (a1 - a0)).clamp(0.0, 1.0);
        let fade = t * t * (3.0 - 2.0 * t);
        (a * (1.0 - fade)).clamp(0.0, AMPLITUDE_MAX_BLOCKS)
    }

    /// The layer's raw u8 for an amplitude in blocks.
    pub fn amplitude_raw(blocks: f64) -> u8 {
        (blocks / AMPLITUDE_STEP_BLOCKS).round().clamp(0.0, 255.0) as u8
    }
}

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

    #[test]
    fn amplitude_follows_relief_drama_and_channels() {
        let d = DetailParams::default();
        d.validate("$.amplify.detail").unwrap();
        // Rough ground, small catchment: fraction of the relief.
        assert!((d.amplitude_blocks(20.0, 1.0e6, 1.0) - 2.4).abs() < 1e-12);
        // Capped, and drama scales both the fraction and the cap.
        assert_eq!(d.amplitude_blocks(500.0, 1.0e6, 1.0), 6.0);
        assert_eq!(d.amplitude_blocks(500.0, 1.0e6, 2.0), 12.0);
        assert_eq!(d.amplitude_blocks(500.0, 1.0e6, 0.0), 0.0);
        // Rivers fade it out: fully gone above the second area, none lost below the first.
        assert_eq!(d.amplitude_blocks(500.0, 300.0e6, 1.0), 0.0);
        assert_eq!(d.amplitude_blocks(500.0, 20.0e6, 1.0), 6.0);
        let mid = d.amplitude_blocks(500.0, 137.5e6, 1.0);
        assert!((mid - 3.0).abs() < 1e-9, "{mid}");
        assert_eq!(DetailParams::amplitude_raw(2.4), 38);
        assert_eq!(DetailParams::amplitude_raw(1000.0), 255);
        assert!(
            DetailParams {
                max_blocks: 9.0,
                ..DetailParams::default()
            }
            .validate("$.x")
            .is_err()
        );
    }
}

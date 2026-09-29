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
//! **The noise** ([`Noise`], defined in `deeptime_format::detail` since P1-08: it is part of the
//! world-data contract) is exactly specified so the Java twin (`worldgen/DetailNoise`, P1-08)
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

/// The value noise the game's detail term uses: the world-data contract's own
/// (`deeptime_format::detail`, FORMAT.md), which the Java reader mirrors.
pub use deeptime_format::detail::Noise;

#[cfg(test)]
mod tests {
    use super::*;

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

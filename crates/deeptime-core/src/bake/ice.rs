// SPDX-License-Identifier: LGPL-3.0-or-later
//! The ice cover: `climate/ice_cover` (L2, u8, blocks).
//!
//! Deep Time's own surface pass lays snow and packed ice on glaciated ground whatever biome the
//! pack's biome source drew there (Ben, 2026-09-30, "Force snow/ice cover"): Terralith has no flat
//! inland ice biome, and its gravel desert would otherwise show through an ice cap. This module is
//! the bake's half: the **thickness of the cover** per L2 cell, from the climate stage's
//! equilibrium-line altitude (`climate/anomaly/ela_m`, L1, every cell) and the ground the biome
//! inputs were placed on. Where the ground lies above its ELA (the glacier rule of
//! [`crate::climate::glaciers`], here on the finer grid) the cover is one block at the foot and one
//! more per `step_m` of height above the line, up to the table's `max_blocks`; it is 0 elsewhere
//! and on the sea.
//!
//! The game reads the layer and the table's `ice_cover` record (`notes.bake.climate.ice_cover`)
//! and lays the blocks (`worldgen/IcePass`): the cover replaces the top blocks of the ground, so
//! the terrain's height is unchanged. The layer is the small interface between the climate and the
//! pass: a climate stage that carries a real ice thickness only has to write it here.

use super::placement::IceCoverSpec;

/// Blocks of cover per cell of `elevation_m` (ground, m) whose equilibrium line is `ela_m` (m above
/// the sea, the same grid); `sea_m` is the sea level in the same elevation units.
pub fn cover_blocks(
    elevation_m: &[f32],
    ela_m: &[f32],
    sea_m: f64,
    spec: &IceCoverSpec,
) -> Vec<u8> {
    elevation_m
        .iter()
        .zip(ela_m)
        .map(|(&z, &ela)| {
            let above_sea = f64::from(z) - sea_m;
            if above_sea <= 0.0 || ela.is_nan() {
                0
            } else {
                spec.blocks(above_sea - f64::from(ela))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cover_grows_with_height_above_the_line() {
        let spec = IceCoverSpec::default();
        assert_eq!((spec.step_m, spec.max_blocks), (160.0, 12));
        // Ground at 100, 700, 900, 1500, 4000 m over a line at 700 m; the last two sea-level cells.
        let z = [100.0, 700.0, 900.0, 1500.0, 4000.0, 0.0, -50.0];
        let ela = [700.0; 7];
        let c = cover_blocks(&z, &ela, 0.0, &spec);
        assert_eq!(c, [0, 0, 2, 6, 12, 0, 0]);
    }

    #[test]
    fn no_line_no_ice_and_the_sea_has_none() {
        let spec = IceCoverSpec::default();
        let c = cover_blocks(
            &[500.0, 500.0, -100.0],
            &[f32::NAN, -1.0e6, -1.0e6],
            0.0,
            &spec,
        );
        assert_eq!(
            c,
            [0, 12, 0],
            "a NaN line gives none; a line far below gives the thickest; the sea none"
        );
        // Above-sea heights count from the sea level, not from zero.
        let c = cover_blocks(&[150.0], &[100.0], 100.0, &spec);
        assert_eq!(c, [0], "50 m over the sea is under a 100 m line");
    }

    #[test]
    fn a_table_can_change_the_step_and_the_ceiling() {
        let spec = IceCoverSpec {
            step_m: 50.0,
            max_blocks: 4,
            ..IceCoverSpec::default()
        };
        assert_eq!(
            cover_blocks(&[10.0, 60.0, 400.0], &[0.0; 3], 0.0, &spec),
            [1, 2, 4]
        );
    }
}

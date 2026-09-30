// SPDX-License-Identifier: LGPL-3.0-or-later
//! The ice cover: `climate/ice_cover` (L2, u8, blocks).
//!
//! Deep Time's own surface pass lays snow and packed ice on glaciated ground whatever biome the
//! pack's biome source drew there (Ben, 2026-09-30, "Force snow/ice cover"): Terralith has no flat
//! inland ice biome, and its gravel desert would otherwise show through an ice cap. This module is
//! the bake's half: the **thickness of the cover** per L2 cell, `1 + ⌊t / step_m⌋` blocks up to the
//! table's `max_blocks` where the land carries ice and 0 elsewhere and on the sea. `t` is, in
//! order of preference, the climate stage's land-ice thickness (`climate/ice_thickness_m`, L1, m:
//! the deep-time climate's ice sheets and glaciers, [`cover_from_thickness`]); else the height of the
//! ground above its equilibrium line (`climate/anomaly/ela_m`, L1, every cell: the Ohmura rule of
//! [`crate::climate::glaciers`] on the finer grid, [`cover_blocks`]); `notes.bake.climate.ice_cover.
//! source` says which.
//!
//! The game reads the layer and the table's `ice_cover` record (`notes.bake.climate.ice_cover`)
//! and lays the blocks (`worldgen/IcePass`): the cover replaces the top blocks of the ground, so
//! the terrain's height is unchanged. The layer is the small interface between the climate and the
//! pass: a climate that changes how it carries ice only has to keep this layer's meaning.

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

/// Blocks of cover per cell from the climate's land-ice thickness (`climate/ice_thickness_m`, m,
/// on the same grid as `elevation_m`): the cell's ice where the ice is at least a metre thick (the
/// glacier flag's own threshold) and the ground is above the sea, `1 + ⌊thickness / step_m⌋` blocks
/// up to the ceiling. The thickness is the climate's own: ice sheets and glaciers as the ice model
/// grew them, not a height above a line.
pub fn cover_from_thickness(
    elevation_m: &[f32],
    thickness_m: &[f32],
    sea_m: f64,
    spec: &IceCoverSpec,
) -> Vec<u8> {
    elevation_m
        .iter()
        .zip(thickness_m)
        .map(|(&z, &t)| {
            if f64::from(z) - sea_m <= 0.0 || t.is_nan() || t < 1.0 {
                0
            } else {
                spec.blocks(f64::from(t))
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
    fn the_climate_s_ice_thickness_sets_the_cover() {
        let spec = IceCoverSpec::default();
        // Ice 0, 0.5 m (under the glacier threshold), 1 m, 159 m, 160 m, 3 km, NaN; the last is sea.
        let z = [100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, -20.0];
        let t = [0.0, 0.5, 1.0, 159.0, 160.0, 3000.0, f32::NAN, 500.0];
        assert_eq!(
            cover_from_thickness(&z, &t, 0.0, &spec),
            [0, 0, 1, 1, 2, 12, 0, 0]
        );
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

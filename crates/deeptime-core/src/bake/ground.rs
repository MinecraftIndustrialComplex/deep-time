// SPDX-License-Identifier: LGPL-3.0-or-later
//! Stone for tools on a planet made of many rocks (P1-28, Ben's decision log 2026-10-01).
//!
//! Deep Time paints the ground with rock-specific blocks, and vanilla's stone tools and furnaces
//! want cobblestone-class items, so the world has to put tool-grade stone within reach. Two
//! settings do that, both under the preset's `ground` section:
//!
//! - **Spawn.** The spawn ranking ([`super::terrain::spawn`], [`crate::deposits::spawn`]) wants a
//!   tool-grade rock class (`stone_use` tool, [`crate::surface::lithology`]) exposed or within
//!   `spawn_toolstone_depth_blocks` of the surface somewhere within `spawn_toolstone_blocks` of the
//!   spawn ([`Toolstone`]). 0 blocks switches it off.
//! - **Cobbles.** River beds, beaches and glacial till get patches and lenses of cobblestone, the
//!   natural source of the blocks vanilla's recipes ask for by name (piston, dispenser, lever). That
//!   is a placement on top of the baked column made by the game (`CobblePass`, `WaterBodies`), from
//!   (x, z) and these densities alone; the bake only records the densities in `notes.bake.ground`,
//!   so nothing about it is parity-relevant between Rust and Java.

use deeptime_format::strata::ColumnHorizon;
use deeptime_format::{LazyLayer, RasterRead};
use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::surface::lithology;

/// The `ground` section of a preset (every key optional).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct GroundParams {
    /// The share of river-bed columns whose bed is cobblestone instead of gravel or sand, 0–1.
    /// Patches and lenses, not a carpet: the cobbles sit in blotches [`Self::cobble_patch_blocks`]
    /// across.
    pub river_cobbles: f64,
    /// The share of beach surface (sand and gravel in a beach biome) that is cobblestone, 0–1.
    pub beach_cobbles: f64,
    /// The share of glacial till (ground whose top rock class is till) that is cobblestone, 0–1.
    pub till_cobbles: f64,
    /// How big a patch of cobbles is, blocks across.
    pub cobble_patch_blocks: f64,
    /// A tool-grade rock must be within this many blocks of the spawn; 0 asks for none.
    pub spawn_toolstone_blocks: u32,
    /// … exposed or at most this many blocks below the surface (the soil on top is a few blocks).
    pub spawn_toolstone_depth_blocks: u32,
}

impl Default for GroundParams {
    fn default() -> Self {
        GroundParams {
            river_cobbles: 0.30,
            beach_cobbles: 0.10,
            till_cobbles: 0.12,
            cobble_patch_blocks: 9.0,
            spawn_toolstone_blocks: 48,
            spawn_toolstone_depth_blocks: 8,
        }
    }
}

impl GroundParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let within = |v: f64, lo: f64, hi: f64, key: &str| {
            if v.is_finite() && (lo..=hi).contains(&v) {
                Ok(())
            } else {
                Err(CoreError::preset(
                    format!("$.ground.{key}"),
                    format!("must be in [{lo}, {hi}]"),
                ))
            }
        };
        within(self.river_cobbles, 0.0, 0.6, "river_cobbles")?;
        within(self.beach_cobbles, 0.0, 0.6, "beach_cobbles")?;
        within(self.till_cobbles, 0.0, 0.6, "till_cobbles")?;
        within(self.cobble_patch_blocks, 2.0, 64.0, "cobble_patch_blocks")?;
        within(
            f64::from(self.spawn_toolstone_blocks),
            0.0,
            512.0,
            "spawn_toolstone_blocks",
        )?;
        within(
            f64::from(self.spawn_toolstone_depth_blocks),
            0.0,
            32.0,
            "spawn_toolstone_depth_blocks",
        )
    }
}

/// Tool-grade rock near a point: the question the spawn ranking asks of every candidate.
pub struct Toolstone<'a> {
    pub horizons: &'a LazyLayer,
    /// The terrain's surface height at a block, the baked `elevation` layer.
    pub elevation: &'a (dyn RasterRead + Sync),
    pub min_y: i32,
    pub radius: u32,
    pub depth: u32,
}

/// Sample spacing of [`Toolstone::near`], blocks (a tool-grade bed is wider than this).
const STEP: i64 = 6;

/// What [`Toolstone::near`] found.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    /// Blocks from the query point to the sample that found it.
    pub distance: f64,
    /// The rock class index (of `strata/classes.json`).
    pub class: u8,
}

impl Toolstone<'_> {
    /// The nearest sample within the radius whose column holds a tool-grade horizon from the
    /// surface down to `depth` blocks below it; samples on rings of growing radius, `STEP` apart.
    pub fn near(&self, x: i64, z: i64) -> Option<Hit> {
        let mut stack: Vec<ColumnHorizon> = Vec::new();
        let r = i64::from(self.radius);
        let mut ring = 0i64;
        while ring <= r {
            let n = if ring == 0 {
                1
            } else {
                ((std::f64::consts::TAU * ring as f64) / STEP as f64).ceil() as i64
            };
            for k in 0..n {
                let a = std::f64::consts::TAU * k as f64 / n as f64;
                let (px, pz) = (
                    x + (ring as f64 * a.cos()).round() as i64,
                    z + (ring as f64 * a.sin()).round() as i64,
                );
                if let Some(class) = self.toolstone_at(px, pz, &mut stack) {
                    return Some(Hit {
                        distance: ring as f64,
                        class,
                    });
                }
            }
            ring += STEP;
        }
        None
    }

    /// The tool-grade class in column (x, z) between the surface and `depth` blocks under it.
    fn toolstone_at(&self, x: i64, z: i64, stack: &mut Vec<ColumnHorizon>) -> Option<u8> {
        let surface = self.elevation.bilinear(x as f64, z as f64);
        if !surface.is_finite() {
            return None;
        }
        let hi = surface.ceil() as i32 - 1;
        let lo = hi - self.depth as i32;
        self.horizons.stack_at(x, z, self.min_y, stack);
        stack.iter().enumerate().find_map(|(h, &(top, class, _))| {
            // Horizon h holds the blocks above the next horizon's top, up to its own.
            let bottom = stack.get(h + 1).map_or(i32::MIN, |n| n.0 + 1);
            (top >= lo && bottom <= hi && lithology::is_toolstone(u16::from(class))).then_some(class)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_are_valid_and_unknown_keys_are_refused() {
        GroundParams::default().validate().unwrap();
        let p: GroundParams = serde_json::from_str(r#"{"river_cobbles": 0.5}"#).unwrap();
        assert_eq!(p.river_cobbles, 0.5);
        assert_eq!(p.spawn_toolstone_blocks, 48);
        assert!(serde_json::from_str::<GroundParams>(r#"{"pebbles": 1}"#).is_err());
        let bad = GroundParams {
            river_cobbles: 0.9,
            ..GroundParams::default()
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn the_toolstone_classes_are_the_hard_ones() {
        for hard in ["granite", "basalt", "quartzite", "chert", "gneiss"] {
            assert!(lithology::is_toolstone(lithology::index(hard)), "{hard}");
        }
        for soft in ["shale", "chalk", "halite", "tuff", "sand", "till", "limestone", "sandstone"] {
            assert!(!lithology::is_toolstone(lithology::index(soft)), "{soft}");
        }
        let n = lithology::table()
            .iter()
            .filter(|c| c.stone_use == lithology::StoneUse::Tool)
            .count();
        assert_eq!(n, 21);
    }
}

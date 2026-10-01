// SPDX-License-Identifier: LGPL-3.0-or-later
//! The `underground` section of a preset: where the game has lava. The bake records the section in
//! `notes.bake.underground.params` and writes the active volcanoes to `volcanoes/active.json`; the
//! game reads both when it generates chunks (Java `worldgen/LavaRules`, `worldgen/MagmaPass`).
//!
//! **The lava floor and lava lakes.** Vanilla fills every open block below y −54 with lava and
//! lets a cave lake at y −10 or lower be lava. Both heights are hard-coded for a world whose floor
//! is y −64: they are floor + 10 and floor + 54. In a Deep Time world (floor y −352 by default)
//! they would flood the lowest 298 blocks of every cave and of every stretch of sea deeper than
//! 117 blocks, so the two distances are settings here and the game measures them from the world's
//! floor.
//!
//! **Lava where the simulation has it** (Ben, decisions log 2026-10-01, "Volcanoes is a go
//! ahead"). Every volcano the volcano stage leaves active at the present gets a magma chamber
//! under it, a conduit up to its crater and, at the most active ones standing on land, a crater
//! lava lake. A dead volcano gets nothing: its chamber froze into the plutons the structure stage
//! already drew. The game builds the bodies from [`active_volcanoes`]'s list and these settings;
//! FORMAT.md ("Lava") has the geometry. The random lava lakes and lava springs of vanilla's and
//! the pack's biomes are removed from Deep Time worlds (`remove_lava_features`).

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::CoreError;
use crate::surface::volcanoes::VolcanoState;

/// The largest distance accepted, blocks (the default build range is 992 blocks tall).
pub const MAX_BLOCKS: u32 = 992;

/// The world-data file of the active volcanoes.
pub const VOLCANOES_PATH: &str = "volcanoes/active.json";
/// The version of that file.
pub const VOLCANOES_VERSION: u32 = 1;

/// The `underground` section of a preset (every key optional).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct UndergroundParams {
    /// Caves and any other open space less than this many blocks above the world's floor are
    /// filled with lava (vanilla: 10). 0: no lava floor. Under a sea bed that deep the lava stays
    /// eight blocks below the bed, so the sea is water all the way down.
    pub lava_floor_blocks: u32,
    /// Cave lakes may be lava where their 40-block level cell is centred no more than this many
    /// blocks above the world's floor (vanilla: 54); higher lakes are always water. 0: no lava
    /// lakes.
    pub lava_lake_blocks: u32,
    /// Depth of the top of an active volcano's magma chamber below its crater floor, km. Real
    /// shallow chambers sit 2–6 km under the summit (Kīlauea's about 2–4 km, Mount St Helens'
    /// from 5 km, Vesuvius' shallow one about 4–5 km). At 20 m per block 3 km is 150 blocks. The
    /// game raises a chamber that would reach the lava floor.
    pub magma_chamber_depth_km: f64,
    /// Multiplies every magma chamber's size (its radius is a tenth of the volcano's basal
    /// radius, between 1.2 and 4.5 km, and its half-height a fifth of that radius). 0: no magma
    /// under volcanoes at all (no chambers, conduits or crater lakes).
    pub magma_chamber_size: f64,
    /// The share of the active volcanoes standing on land that hold a crater lava lake, the most
    /// active first. Persistent lava lakes are rare on Earth (about eight of some 1,400 active
    /// volcanoes); the default is far above that, because a player should be able to find one.
    pub crater_lake_share: f64,
    /// Whether the random lava lakes and lava springs of vanilla's and the pack's biomes are taken
    /// out of a Deep Time world, so lava is found at the volcanoes and at the world's floor only.
    pub remove_lava_features: bool,
}

impl Default for UndergroundParams {
    fn default() -> Self {
        UndergroundParams {
            lava_floor_blocks: 10,
            lava_lake_blocks: 54,
            magma_chamber_depth_km: 3.0,
            magma_chamber_size: 1.0,
            crater_lake_share: 0.25,
            remove_lava_features: true,
        }
    }
}

impl UndergroundParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        for (v, key) in [
            (self.lava_floor_blocks, "lava_floor_blocks"),
            (self.lava_lake_blocks, "lava_lake_blocks"),
        ] {
            if v > MAX_BLOCKS {
                return Err(CoreError::preset(
                    format!("$.underground.{key}"),
                    format!("must be in [0, {MAX_BLOCKS}]"),
                ));
            }
        }
        for (v, lo, hi, key) in [
            (
                self.magma_chamber_depth_km,
                0.5,
                15.0,
                "magma_chamber_depth_km",
            ),
            (self.magma_chamber_size, 0.0, 4.0, "magma_chamber_size"),
            (self.crater_lake_share, 0.0, 1.0, "crater_lake_share"),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                return Err(CoreError::preset(
                    format!("$.underground.{key}"),
                    format!("must be in [{lo}, {hi}]"),
                ));
            }
        }
        Ok(())
    }
}

/// A volcano's activity, 0–1: the arc magma flux of an arc volcano's last step, the mean plume
/// proximity of a hotspot's.
fn activity(v: &crate::surface::volcanoes::Volcano) -> f64 {
    let a = if v.hotspot.is_some() {
        if v.active_myr > 0.0 {
            v.flux_myr / v.active_myr
        } else {
            0.5
        }
    } else {
        v.flux_last
    };
    if a.is_finite() {
        a.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// `volcanoes/active.json`: the volcanoes active at the present, in id order (FORMAT.md, "Lava").
/// Dead volcanoes and those that left the catalogue are not listed.
pub fn active_volcanoes(st: &VolcanoState) -> Value {
    let r3 = |v: f64| (v * 1000.0).round() / 1000.0;
    let mut list: Vec<&crate::surface::volcanoes::Volcano> = st
        .volcanoes
        .iter()
        .filter(|v| v.active() && v.gone.is_none() && v.radius_m > 0.0)
        .collect();
    list.sort_by_key(|v| v.id);
    let volcanoes: Vec<Value> = list
        .iter()
        .map(|v| {
            let crater = v.depression.map(|d| {
                json!({
                    "radius_m": (d.rho * v.radius_m).round(),
                    "depth_m": (d.depth * v.height_m).round(),
                    "caldera": d.caldera,
                })
            });
            json!({
                "id": v.id,
                "kind": v.kind.name(),
                "x": r3(v.x),
                "z": r3(v.z),
                "radius_m": v.radius_m.round(),
                "height_m": v.height_m.round(),
                "activity": r3(activity(v)),
                "oceanic": v.oceanic,
                "silicic": v.silicic,
                "age_myr": r3((st.t_myr - v.born_myr).max(0.0)),
                "crater": crater,
            })
        })
        .collect();
    json!({
        "version": VOLCANOES_VERSION,
        "t_myr": st.t_myr,
        "volcanoes": volcanoes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_vanillas_heights_above_the_floor() {
        // Vanilla: floor −64, lava below −54, lava lakes at −10 and lower.
        let d = UndergroundParams::default();
        assert_eq!(-64 + d.lava_floor_blocks as i32, -54);
        assert_eq!(-64 + d.lava_lake_blocks as i32, -10);
        d.validate().unwrap();
    }

    #[test]
    fn a_section_may_set_one_key_and_rejects_unknown_ones() {
        let p: UndergroundParams = serde_json::from_str(r#"{"lava_lake_blocks": 0}"#).unwrap();
        assert_eq!(p.lava_floor_blocks, 10);
        assert_eq!(p.lava_lake_blocks, 0);
        assert!(p.remove_lava_features);
        assert!(serde_json::from_str::<UndergroundParams>(r#"{"lava": 1}"#).is_err());
        assert!(serde_json::from_str::<UndergroundParams>(r#"{"lava_floor_blocks": -1}"#).is_err());
        let e = UndergroundParams {
            lava_floor_blocks: 993,
            ..UndergroundParams::default()
        }
        .validate()
        .unwrap_err();
        assert!(
            e.to_string().contains("$.underground.lava_floor_blocks"),
            "{e}"
        );
        let e = UndergroundParams {
            crater_lake_share: 1.5,
            ..UndergroundParams::default()
        }
        .validate()
        .unwrap_err();
        assert!(
            e.to_string().contains("$.underground.crater_lake_share"),
            "{e}"
        );
    }
}

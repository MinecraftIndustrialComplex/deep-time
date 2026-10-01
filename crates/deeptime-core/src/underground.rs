// SPDX-License-Identifier: LGPL-3.0-or-later
//! The `underground` section of a preset: what the game puts in the open space below ground.
//! Nothing here is simulated; the bake records the section in `notes.bake.underground.params` and
//! the game reads it back when it generates chunks (Java `worldgen/LavaRules`).
//!
//! **Lava.** Vanilla fills every open block below y −54 with lava and lets a cave lake at y −10
//! or lower be lava. Both heights are hard-coded for a world whose floor is y −64: they are
//! floor + 10 and floor + 54. In a Deep Time world (floor y −352 by default) they would flood the
//! lowest 298 blocks of every cave and of every stretch of sea deeper than 117 blocks, so the two
//! distances are settings here and the game measures them from the world's floor.

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// The largest distance accepted, blocks (the default build range is 992 blocks tall).
pub const MAX_BLOCKS: u32 = 992;

/// The `underground` section of a preset (every key optional).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
}

impl Default for UndergroundParams {
    fn default() -> Self {
        UndergroundParams {
            lava_floor_blocks: 10,
            lava_lake_blocks: 54,
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
        Ok(())
    }
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
        assert!(serde_json::from_str::<UndergroundParams>(r#"{"lava": 1}"#).is_err());
        assert!(serde_json::from_str::<UndergroundParams>(r#"{"lava_floor_blocks": -1}"#).is_err());
        let e = UndergroundParams {
            lava_floor_blocks: 993,
            lava_lake_blocks: 54,
        }
        .validate()
        .unwrap_err();
        assert!(
            e.to_string().contains("$.underground.lava_floor_blocks"),
            "{e}"
        );
    }
}

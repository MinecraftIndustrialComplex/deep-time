// SPDX-License-Identifier: LGPL-3.0-or-later
//! Rivers and lakes (plan §8.4, work item P1-22, moved from P2-03 by Ben's decision of 2026-09-29,
//! "Rivers & lakes in v1"): the drainage of the finished terrain, the lakes it fills and the river
//! network it carries, in block space, for the bake to write and the game to carve.
//!
//! The bake ([`crate::bake::water`]) runs [`network::run`] on the L3 terrain (the amplifier's
//! `elevation`, 2 blocks a cell on every preset but 32k full, which is averaged to 4 blocks so the
//! working set stays under ~600 MB) with the climate's runoff:
//!
//! 1. **Drainage.** The open sea is every cell at least a block below sea level in a connected body of at least
//!    [`OPEN_SEA_KM2`]; a Priority-Flood (Barnes, Lehman & Mulla 2014, Algorithm 3 with the pit
//!    queue of Algorithm 4, implemented from the paper as `surface::pflood` is) from its shores
//!    (and the first and last rows, the poles past the hard cap, as outlets at their own height)
//!    gives every land cell its spill level F and the cell it was reached from. Each cell drains to
//!    its steepest strictly lower neighbour on F, else (flats and lakes) to the cell it was
//!    reached from, so every path descends to the sea, and the receivers form a forest.
//! 2. **Lakes: fill and spill.** A connected set of cells below their F is a depression; it becomes
//!    a lake at its spill level F when it is at least `lake_min_area_km2` and
//!    `lake_min_depth_blocks` deep. Every lake is full to its lowest rim and spills through it (the
//!    plan's P2 exit test, "every lake's surface equals its lowest rim"); the outflow is its inflow
//!    less the evaporation from its surface (`lake_evaporation`), so a lake in a desert may have no
//!    outlet river. Evaporation-limited levels below the rim (Fill–Spill–Merge's partial fills)
//!    stay in P2-03.
//! 3. **Discharge.** Runoff P − ET (Fu's Budyko curve, as the surface stage, from the climate
//!    stage's temperature and precipitation) accumulated down the receivers: Q in m³/s.
//! 4. **Rivers.** Cells carrying at least `river_min_discharge_m3s` from at least
//!    `river_min_area_km2` are river cells. They are cut into chains at confluences (a chain
//!    follows its largest donor), each ending in the sea, a lake or a larger river; the vertices
//!    are smoothed (a Gaussian of half-width `smoothing_blocks`) to take the grid's 45° steps out. The water
//!    surface at each vertex is F − `freeboard_blocks`, never below the level of the sea or lake
//!    downstream: it never rises downstream. Width and depth follow Leopold & Maddock's hydraulic
//!    geometry, W = a·Q^0.5 and D = c·Q^0.4, scaled for the block world (`width_coef`,
//!    `depth_coef`; × `scale.feature_scale.rivers`), at least one block each.
//!
//! What the game does with it is `deeptime_format::water` (and FORMAT.md, "Water columns").

pub mod network;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// The smallest body of below-sea-level cells that counts as open sea, km² (smaller ones are
/// closed basins: lakes if they are deep enough).
pub const OPEN_SEA_KM2: f64 = 5_000.0;

/// The largest hydrology grid, cells; a larger L3 is averaged down by 2 in each direction.
pub const MAX_CELLS: usize = 40_000_000;

/// Hydraulic geometry exponents (Leopold & Maddock 1953: width ∝ Q^0.5, depth ∝ Q^0.4).
pub const WIDTH_EXPONENT: f64 = 0.5;
pub const DEPTH_EXPONENT: f64 = 0.4;

/// The `hydro` section of a preset (every key optional).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct HydroParams {
    /// Rivers and lakes at all (false: `water_level` all nodata and no river network, as before
    /// P1-22).
    pub enabled: bool,
    /// Discharge from which a stream is drawn, m³/s.
    pub river_min_discharge_m3s: f64,
    /// Drainage area from which a stream is drawn, km².
    pub river_min_area_km2: f64,
    /// Channel width in blocks per (m³/s)^0.5, before `scale.feature_scale.rivers`.
    pub width_coef: f64,
    /// Channel depth in blocks per (m³/s)^0.4.
    pub depth_coef: f64,
    /// The widest channel, blocks.
    pub max_width_blocks: f64,
    /// The deepest channel, blocks.
    pub max_depth_blocks: f64,
    /// How far the water surface sits below the valley floor it follows, blocks.
    pub freeboard_blocks: f64,
    /// The half-width of the kernel that smooths river courses, blocks.
    pub smoothing_blocks: f64,
    /// The smallest lake, km².
    pub lake_min_area_km2: f64,
    /// The shallowest lake at its deepest point, blocks.
    pub lake_min_depth_blocks: f64,
    /// Whether evaporation from a lake's surface reduces what flows out of it.
    pub lake_evaporation: bool,
    /// Channels at least this wide are river biomes, blocks.
    pub biome_min_width_blocks: f64,
    /// Banks slope up 1:1 from a channel's edge over this many blocks …
    pub bank_zone_blocks: f64,
    /// … plus this share of its half-width.
    pub bank_zone_ratio: f64,
    /// Whether the game's carvers (caves, canyons) skip the water blocks Deep Time places in lakes
    /// and rivers. The ground under the water is still carved, so caves run beneath it and may
    /// open into a lake or river bed; only the water itself is never replaced (no air pockets in
    /// it, no holes in a lake's surface). Read by the game from `notes.bake.water.params`.
    pub protect_water_from_carvers: bool,
}

impl Default for HydroParams {
    fn default() -> Self {
        HydroParams {
            enabled: true,
            river_min_discharge_m3s: 2.0,
            river_min_area_km2: 50.0,
            width_coef: 1.0,
            depth_coef: 0.5,
            max_width_blocks: 64.0,
            max_depth_blocks: 12.0,
            freeboard_blocks: 1.0,
            smoothing_blocks: 8.0,
            lake_min_area_km2: 1.0,
            lake_min_depth_blocks: 1.0,
            lake_evaporation: true,
            biome_min_width_blocks: 6.0,
            bank_zone_blocks: 1.0,
            bank_zone_ratio: 0.5,
            protect_water_from_carvers: true,
        }
    }
}

impl HydroParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let within = |v: f64, lo: f64, hi: f64, key: &str| {
            if v.is_finite() && (lo..=hi).contains(&v) {
                Ok(())
            } else {
                Err(CoreError::preset(
                    format!("$.hydro.{key}"),
                    format!("must be in [{lo}, {hi}]"),
                ))
            }
        };
        within(
            self.river_min_discharge_m3s,
            0.01,
            10_000.0,
            "river_min_discharge_m3s",
        )?;
        within(
            self.river_min_area_km2,
            0.0,
            100_000.0,
            "river_min_area_km2",
        )?;
        within(self.width_coef, 0.05, 20.0, "width_coef")?;
        within(self.depth_coef, 0.05, 20.0, "depth_coef")?;
        within(self.max_width_blocks, 1.0, 256.0, "max_width_blocks")?;
        within(self.max_depth_blocks, 1.0, 64.0, "max_depth_blocks")?;
        within(self.freeboard_blocks, 0.0, 8.0, "freeboard_blocks")?;
        within(self.smoothing_blocks, 0.0, 64.0, "smoothing_blocks")?;
        within(self.lake_min_area_km2, 0.0, 100_000.0, "lake_min_area_km2")?;
        within(
            self.lake_min_depth_blocks,
            0.0,
            64.0,
            "lake_min_depth_blocks",
        )?;
        within(
            self.biome_min_width_blocks,
            1.0,
            1000.0,
            "biome_min_width_blocks",
        )?;
        within(self.bank_zone_blocks, 0.0, 32.0, "bank_zone_blocks")?;
        within(self.bank_zone_ratio, 0.0, 4.0, "bank_zone_ratio")
    }

    /// Channel width in blocks at discharge `q` (m³/s), with the rivers feature scale.
    pub fn width(&self, q: f64, scale: f64) -> f64 {
        (scale * self.width_coef * q.max(0.0).powf(WIDTH_EXPONENT))
            .clamp(1.0, self.max_width_blocks)
    }

    /// Channel depth in blocks at discharge `q`.
    pub fn depth(&self, q: f64) -> f64 {
        (self.depth_coef * q.max(0.0).powf(DEPTH_EXPONENT)).clamp(1.0, self.max_depth_blocks)
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The named-field interface between the history stages and the deposit engine (plan §5.4.2,
//! §5.7.5 "Deposit fields"; `ore_genesis_modeling.md` Q1's ~20 per-cell fields).
//!
//! The history stages (plates P1-03, surface P1-04, climate P1-05, strata P1-07) publish **frames**:
//! snapshots of named L1 fields and global clocks at a time `t`, each standing for `dt` years of
//! history. The deposit engine reads them oldest first through [`FieldHistory`], twice: pass 1
//! over every cell (favourability, maturity, trap timing), pass 2 only at the chosen sites
//! (burial and erosion since formation, water table, climate). A history must therefore be
//! replayable: the synthetic one regenerates a frame from its seed, the real pipeline records its
//! frames (in memory or on disk) as it runs.
//!
//! [`FIELDS`] and [`CLOCKS`] list every name the built-in deposit types read, with units, range
//! and the task that provides it; `STATUS.md` repeats the table for P1-03…P1-07. Host-rock
//! fractions are the open family `deeptime:host/<rock class>` (P1-07). A history may leave any
//! field out: a type whose fields are missing is skipped with a warning, and a history with no
//! host fields at all means "no lithology opinion" (every cell a full host). The regional fields
//! (`deeptime:basin_brine`, `deeptime:regional_head_drop_m_km`) are not recorded: the engine
//! derives them from each frame it reads (`regional.rs`) when their inputs are there.

use std::collections::BTreeMap;

use deeptime_format::GridDesc;
use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// Prefix of the host-rock fraction fields (`deeptime:host/dolostone`, 0–1).
pub const HOST_PREFIX: &str = "deeptime:host/";
/// Prefix of the global clocks.
pub const CLOCK_PREFIX: &str = "deeptime:clock/";

/// Names of the crust-tracking fields a moving history's frames carry: for each L1 cell, where its
/// crust was at the start of the frame, as a column and a row offset in cells (small integers,
/// exact in the 16-bit packing; NaN for crust born in the frame). The engine carries its
/// accumulators with the plates through them ([`FieldHistory::origins`]).
pub const ORIGIN_FIELDS: [&str; 2] = ["deeptime:origin_di", "deeptime:origin_dj"];
/// A cell whose crust did not exist at the start of the frame (or of the history).
pub const NO_ORIGIN: u32 = u32::MAX;

// The fields the engine itself reads (beyond each type's rules).
pub const ELEVATION: &str = "deeptime:elevation_m";
pub const EROSION: &str = "deeptime:erosion_rate_m_myr";
pub const DEPOSITION: &str = "deeptime:deposition_rate_m_myr";
pub const WATER_TABLE: &str = "deeptime:water_table_depth_m";
pub const PRECIP: &str = "deeptime:precip_mm";
pub const GRADIENT: &str = "deeptime:topographic_gradient_m_km";
pub const BURIAL_TEMP: &str = "deeptime:burial_temp_c";

/// What one name is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldSpec {
    pub name: &'static str,
    pub units: &'static str,
    /// Typical physical range (for the viewer's ramps and the synthetic generator).
    pub range: (f32, f32),
    /// The task that publishes it.
    pub provider: &'static str,
    pub meaning: &'static str,
}

/// Every per-cell field the built-in types and the engine read.
pub const FIELDS: &[FieldSpec] = &[
    FieldSpec {
        name: "deeptime:arc_magma_flux",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P1-03 plates",
        meaning: "calc-alkaline arc magmatism, relative to a vigorous continental arc",
    },
    FieldSpec {
        name: "deeptime:magma_oxidation",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P1-03 plates",
        meaning: "magnetite-series (oxidised, hydrous) fraction of the arc magmas",
    },
    FieldSpec {
        name: "deeptime:convergence_rate_km_myr",
        units: "km/Myr",
        range: (0.0, 200.0),
        provider: "P1-03 plates",
        meaning: "plate convergence rate at the nearest trench, carried onto the upper plate's arc",
    },
    FieldSpec {
        name: "deeptime:obliquity_deg",
        units: "degrees",
        range: (0.0, 90.0),
        provider: "P1-03 plates",
        meaning: "convergence obliquity to the trench normal",
    },
    FieldSpec {
        name: "deeptime:slab_age_myr",
        units: "Myr",
        range: (0.0, 200.0),
        provider: "P1-03 plates",
        meaning: "age of the subducting slab at the trench",
    },
    FieldSpec {
        name: "deeptime:crust_thickness_km",
        units: "km",
        range: (0.0, 80.0),
        provider: "P1-03 plates",
        meaning: "crustal thickness",
    },
    FieldSpec {
        name: "deeptime:fault_density",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P1-03 plates",
        meaning: "density of active and reactivated faults (strain-rate proxy)",
    },
    FieldSpec {
        name: "deeptime:orogenic_uplift_m_myr",
        units: "m/Myr",
        range: (0.0, 1000.0),
        provider: "P1-03 plates",
        meaning: "rock uplift rate from collision and crustal thickening",
    },
    FieldSpec {
        name: "deeptime:rift_subsidence_m_myr",
        units: "m/Myr",
        range: (0.0, 500.0),
        provider: "P1-03 plates",
        meaning: "tectonic subsidence rate of rifts and sag basins",
    },
    FieldSpec {
        name: "deeptime:heat_flow_mw_m2",
        units: "mW/m²",
        range: (30.0, 150.0),
        provider: "P1-03 plates",
        meaning: "surface heat flow",
    },
    FieldSpec {
        name: "deeptime:dist_litho_step_km",
        units: "km",
        range: (0.0, 3000.0),
        provider: "P1-03 plates",
        meaning: "distance to the nearest thick–thin lithosphere transition (Hoggard et al. 2020)",
    },
    FieldSpec {
        name: "deeptime:inherited_bif",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P1-03 plates (craton regime) × the timeline (deposits::clocks)",
        meaning: "craton basement that inherited the BIF acme (the acme's share before the run)",
    },
    FieldSpec {
        name: "deeptime:shelf",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P1-04 surface",
        meaning: "shallow marine shelf fraction (passive margin or intracratonic sea)",
    },
    FieldSpec {
        name: ELEVATION,
        units: "m",
        range: (-6000.0, 6000.0),
        provider: "P1-04 surface",
        meaning: "surface elevation relative to sea level",
    },
    FieldSpec {
        name: EROSION,
        units: "m/Myr",
        range: (0.0, 1000.0),
        provider: "P1-04 surface",
        meaning: "erosion (exhumation) rate of the column",
    },
    FieldSpec {
        name: DEPOSITION,
        units: "m/Myr",
        range: (0.0, 500.0),
        provider: "P1-04 surface",
        meaning: "deposition (burial) rate of the column",
    },
    FieldSpec {
        name: GRADIENT,
        units: "m/km",
        range: (0.0, 50.0),
        provider: "P1-04 surface",
        meaning: "regional topographic gradient (drives basin brines)",
    },
    FieldSpec {
        name: "deeptime:carbonate_platform",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P1-04 surface (carbonate producer)",
        meaning: "carbonate-platform facies fraction of the column",
    },
    FieldSpec {
        name: "deeptime:evaporite",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P1-04 surface",
        meaning: "evaporite facies fraction (brine source)",
    },
    FieldSpec {
        name: "deeptime:euxinic_basin",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P1-04 surface",
        meaning: "euxinic (organic-rich, reduced) black-shale facies",
    },
    FieldSpec {
        name: "deeptime:coal_facies",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P1-04 surface (strata log)",
        meaning: "peat-mire facies: accommodation matched to peat production (Bohacs & Suter 1997)",
    },
    FieldSpec {
        name: "deeptime:source_rock_toc",
        units: "wt% TOC",
        range: (0.0, 10.0),
        provider: "P1-04 surface (strata log)",
        meaning: "organic carbon of the best source-rock layer in the column",
    },
    FieldSpec {
        name: BURIAL_TEMP,
        units: "°C",
        range: (0.0, 250.0),
        provider: "P1-04 surface + P1-07 strata (burial × geotherm)",
        meaning: "temperature of the organic-rich layer (source rock or coal) at this time (Lopatin TTI)",
    },
    FieldSpec {
        name: "deeptime:reservoir_quality",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P1-04 surface",
        meaning: "porous reservoir (sandstone, carbonate) above the source",
    },
    FieldSpec {
        name: "deeptime:seal",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P1-04 surface",
        meaning: "sealing layer (shale, evaporite) above the reservoir",
    },
    FieldSpec {
        name: "deeptime:trap_structure",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P1-07 strata (folds, salt domes); P2-08 impacts (crater rims and uplifts)",
        meaning: "structural closure (anticline, salt dome or impact structure) at reservoir level",
    },
    FieldSpec {
        name: "deeptime:impact_melt_m",
        units: "m",
        range: (0.0, 3000.0),
        provider: "P2-08 impacts",
        meaning: "thickness of the impact melt sheet of a crater formed in the frame",
    },
    FieldSpec {
        name: "deeptime:impact_hydrothermal",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P2-08 impacts",
        meaning: "impact-generated hydrothermal system of a crater formed in the frame",
    },
    FieldSpec {
        name: "deeptime:impact_shock",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P2-08 impacts",
        meaning: "shock metamorphism of the target of a crater formed in the frame",
    },
    FieldSpec {
        name: "deeptime:impact_basin",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "P2-08 impacts",
        meaning: "inside the rim of an impact structure at least 5 km across",
    },
    FieldSpec {
        name: WATER_TABLE,
        units: "m",
        range: (0.0, 1000.0),
        provider: "P1-04 surface + P1-05 climate",
        meaning: "depth of the water table below the surface",
    },
    FieldSpec {
        name: PRECIP,
        units: "mm/yr",
        range: (0.0, 3000.0),
        provider: "P1-05 climate",
        meaning: "mean annual precipitation",
    },
    FieldSpec {
        name: "deeptime:t_mean_c",
        units: "°C",
        range: (-30.0, 35.0),
        provider: "P1-05 climate",
        meaning: "mean annual temperature",
    },
];

/// Global clocks: one value per frame for the whole planet (plan §5.1 `AtmosphereClock`).
pub const CLOCKS: &[FieldSpec] = &[
    FieldSpec {
        name: "deeptime:clock/oxygenated_oceans",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "the timeline and climate.present_age_gyr (deposits::clocks)",
        meaning: "oxidised hydrosphere; sediment-hosted Zn–Pb only after 2.02 Ga (Leach et al. 2010)",
    },
    FieldSpec {
        name: "deeptime:clock/land_plants",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "the timeline and climate.present_age_gyr (deposits::clocks)",
        meaning: "land plants exist (coal; Nelsen et al. 2016)",
    },
    FieldSpec {
        name: "deeptime:clock/bif_acme",
        units: "0–1",
        range: (0.0, 1.0),
        provider: "the timeline and climate.present_age_gyr (deposits::clocks)",
        meaning: "the 2.65–1.85 Ga banded-iron-formation acme (Bekker et al. 2010)",
    },
];

/// The spec of a field or clock name.
pub fn spec(name: &str) -> Option<&'static FieldSpec> {
    FIELDS.iter().chain(CLOCKS).find(|s| s.name == name)
}

/// One snapshot of the history.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FieldFrame {
    /// Years relative to world creation (negative = past).
    pub t_years: f64,
    /// The span of history this frame stands for.
    pub dt_years: f64,
    /// L1 fields, row-major (`data[j · w + i]`), physical units.
    pub fields: BTreeMap<String, Vec<f32>>,
    /// Global clocks at `t_years`.
    pub clocks: BTreeMap<String, f32>,
}

/// A name resolved in a frame.
#[derive(Clone, Copy, Debug)]
pub enum Values<'a> {
    Cells(&'a [f32]),
    Uniform(f32),
}

impl Values<'_> {
    pub fn at(&self, cell: usize) -> f32 {
        match self {
            Values::Cells(v) => v[cell],
            Values::Uniform(v) => *v,
        }
    }
}

impl FieldFrame {
    /// The field or clock `name`.
    pub fn values(&self, name: &str) -> Option<Values<'_>> {
        if let Some(v) = self.fields.get(name) {
            return Some(Values::Cells(v));
        }
        self.clocks.get(name).map(|v| Values::Uniform(*v))
    }

    pub fn dt_myr(&self) -> f64 {
        self.dt_years / 1.0e6
    }
}

/// A replayable history of frames on one L1 grid, oldest first.
pub trait FieldHistory: Send + Sync {
    fn grid(&self) -> &GridDesc;

    fn frame_count(&self) -> usize;

    /// Frame `k` (0 = oldest). Every frame has the same field and clock names.
    fn frame(&self, k: usize) -> Result<FieldFrame, CoreError>;

    /// The names every frame carries (fields and clocks).
    fn names(&self) -> Vec<String>;

    /// Present-day drainage: the L1 cell each cell drains to (itself for a sink or the sea), from
    /// P1-04's routing. `None` when unknown (placers then stay at their source).
    fn receivers(&self) -> Option<&[u32]>;

    /// Crust motion into frame `k`: for each L1 cell, the cell that held its crust at the end of
    /// frame `k − 1` ([`NO_ORIGIN`] for crust born in frame `k`). `None` when the crust does not
    /// move in this history (the synthetic one, frame 0, a history recorded without the origin
    /// fields): cells are then fixed crust parcels.
    fn origins(&self, _k: usize) -> Result<Option<Vec<u32>>, CoreError> {
        Ok(None)
    }
}

/// Origin cells from the offsets of [`ORIGIN_FIELDS`] (`w` columns): NaN offsets are new crust.
pub fn origins_from_offsets(w: usize, di: &[f32], dj: &[f32]) -> Vec<u32> {
    let h = di.len() / w.max(1);
    di.iter()
        .zip(dj)
        .enumerate()
        .map(|(c, (&a, &b))| {
            if !a.is_finite() || !b.is_finite() {
                return NO_ORIGIN;
            }
            let i = (c % w) as i64 + a.round() as i64;
            let j = ((c / w) as i64 + b.round() as i64).clamp(0, h as i64 - 1);
            (j as usize * w + i.rem_euclid(w as i64) as usize) as u32
        })
        .collect()
}

/// A history held in memory (tests; and the shape P1-03…P1-07 can record into).
#[derive(Clone, Debug, Default)]
pub struct MemoryHistory {
    pub grid: Option<GridDesc>,
    pub frames: Vec<FieldFrame>,
    pub receivers: Option<Vec<u32>>,
}

impl FieldHistory for MemoryHistory {
    fn grid(&self) -> &GridDesc {
        self.grid.as_ref().expect("a MemoryHistory needs a grid")
    }

    fn frame_count(&self) -> usize {
        self.frames.len()
    }

    fn frame(&self, k: usize) -> Result<FieldFrame, CoreError> {
        self.frames
            .get(k)
            .cloned()
            .ok_or_else(|| CoreError::InvalidArgument(format!("no frame {k}")))
    }

    fn names(&self) -> Vec<String> {
        self.frames.first().map_or_else(Vec::new, |f| {
            f.fields.keys().chain(f.clocks.keys()).cloned().collect()
        })
    }

    fn receivers(&self) -> Option<&[u32]> {
        self.receivers.as_deref()
    }
}

/// True area of L1 cell row `j` in km² on a planet whose blocks are `m_per_block_h` wide:
/// (r · m / 1000)² · cos φ (plan §5.2: block-metric cells, true area shrinks with latitude).
pub fn cell_area_km2(grid: &GridDesc, j: u32, m_per_block_h: f64) -> f64 {
    let side_km = f64::from(grid.r) * m_per_block_h / 1000.0;
    side_km * side_km * lat_of_row(grid, j).to_radians().cos().max(0.0)
}

/// Latitude (degrees, north positive) of L1 row `j`: z = origin + j·r, lat = −z / (C/4) · 90.
pub fn lat_of_row(grid: &GridDesc, j: u32) -> f64 {
    let z = grid.block_z(i64::from(j)) as f64;
    -z / (grid.circumference() as f64 / 4.0) * 90.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs_are_unique_and_namespaced() {
        let mut names: Vec<&str> = FIELDS.iter().chain(CLOCKS).map(|s| s.name).collect();
        let n = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), n);
        for s in FIELDS.iter().chain(CLOCKS) {
            assert!(deeptime_format::events::is_namespaced(s.name), "{}", s.name);
            assert!(s.range.0 < s.range.1);
        }
        assert!(spec("deeptime:clock/land_plants").is_some());
    }

    #[test]
    fn areas_sum_to_the_sphere() {
        let grid = GridDesc::for_planet("L1", 16_384, 16).unwrap();
        let total: f64 = (0..grid.h)
            .map(|j| cell_area_km2(&grid, j, 300.0) * f64::from(grid.w))
            .sum();
        let r_km = 16_384.0 * 0.3 / (2.0 * std::f64::consts::PI);
        let sphere = 4.0 * std::f64::consts::PI * r_km * r_km;
        assert!((total / sphere - 1.0).abs() < 0.01, "{total} vs {sphere}");
        assert!((lat_of_row(&grid, 0) - 90.0).abs() < 1e-9);
    }
}

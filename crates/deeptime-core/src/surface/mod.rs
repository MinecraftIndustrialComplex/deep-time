// SPDX-License-Identifier: LGPL-3.0-or-later
//! Deep-time surface processes on L1 (plan §8.2, work item P1-04): erosion, deposition,
//! diffusion, marine sedimentation, carbonate production, weathering and the strata log, coupled
//! to the plate model step by step.
//!
//! # Coupling with the plates (plan §8 intro)
//! The stage is interleaved with the plates stage (`Stage::coupled_to`): after every plate step it
//! takes one step of the same length. L1 is fixed on the planet and the crust moves over it, so the
//! surface's elevation, regolith and strata log move with the plate under each cell (`advect.rs`),
//! each with its own crust: a cell takes them from the cells that held the same crust before the
//! step, and where another plate's crust has arrived it starts from the plate model's elevation.
//! Uplift is what the plate model did to the crust under a cell (subduction and collision uplift,
//! thermal subsidence, trench accretion), exactly P1-03's `plates/uplift_mm_yr`: the surface
//! starts a step at its last elevation, carried, plus that uplift. The **offset** is what stands
//! between it and the plate model's own elevation (`plates/elevation_m`, sampled onto L1 every
//! step; the pipeline presets set `tectonics.l1_every_steps` = 1): valleys, basins, eroded ranges,
//! what the coarse L0 samples do not carry. It is an output, not the state (2026-10-01: carried as
//! the state and added to the plate model's elevation, it was added to whatever crust the cell
//! read next, and built 13–20 km "mountains").
//!
//! The surface flows back: the whole offset of every cell (`surface/feedback_m`, and the material
//! moved this step, `surface/material_m`) is spread onto the L0 samples at the start of the next
//! plate step (`plates::PlatesStage`), so a sample's elevation is the surface's at its scale and
//! the plate model's uplift laws see the real height. The plate model's own continental erosion
//! (Cortial's ε_c) is off in the pipeline presets — this stage does erosion.
//!
//! # One step (Δt = the plate step, `substeps` LEM steps inside it)
//! 1. Carry the elevation, regolith and strata log; clear them on crust born this step.
//! 2. h = carried elevation + the plate model's uplift; sea level from the preset (`params::SeaLevel`), or, with a
//!    climate stage, its eustatic sea level (the curve plus glacio-eustasy); climate
//!    (`climate.rs`: the climate stage's fields, which it wrote just before this step on the same
//!    ground, when the preset has a `climate` section (P1-05), else the stand-in).
//! 3. Per LEM step: the sea is the base level; priority-flood (`pflood.rs`), D8 routing and the
//!    stack (`routing.rs`); implicit stream power with erosion–deposition and a critical slope
//!    (`spl.rs`), runoff-weighted areas, K from the surface rock; river loads fill the sea from
//!    their mouths (`marine.rs`); carbonate production (`carbonate.rs`) and evaporites in
//!    restricted arid seas and closed arid lakes; one implicit diffusion over land (hillslope κ
//!    from the surface rock) and sea (Landlab's depth-dependent D).
//! 4. The net change of every cell goes into the strata log with its facies (`facies.rs`,
//!    `strata_log.rs`) or erodes regolith, then the log; weathering makes regolith
//!    (`weathering.rs`); buried regolith becomes a log layer (a palaeosol).
//! 5. Local isostasy: the load change, smoothed over `isostasy_radius_km`, is compensated by
//!    `isostasy` (0.7 of it under water).
//!    Then crust that stands higher than its strength holds spreads sideways (`spread.rs`:
//!    lower-crustal flow; the height of plateaus, and its dependence on gravity, come from here).
//!    With `flexure.strength` > 0 the plate has stiffness (`flexure.rs`): every load of the step
//!    (the plate model's thickening, erosion and deposition, the spreading, volcanic edifices,
//!    water, ice) bends it, and the solve takes the place of that share of the local isostasy.
//! 6. Keyframe fields, the plate feedback, the deposit engine's frame (`fields.rs`) and, every
//!    frame, the events (`events.rs`).
//!
//! Mass: routing, stream power, the mouth fill and diffusion conserve material exactly (to
//! round-off); carbonate and evaporite are added from the sea. `notes.surface.mass_residual_km3`
//! reports Σ(Δh)·a − (carbonate + evaporite) for the whole run.

pub mod advect;
pub mod amplify;
pub mod carbonate;
pub mod climate;
pub mod detail;
pub mod diffusion;
pub mod erosion_bands;
pub mod events;
pub mod facies;
pub mod fields;
pub mod flexure;
pub mod glacial;
pub mod grid;
pub mod impacts;
pub mod lithology;
pub mod marine;
pub mod oracle;
pub mod params;
pub mod pflood;
pub mod routing;
pub mod spl;
pub mod spread;
pub mod strata_log;
pub mod thermal;
pub mod volcanoes;
pub mod weathering;
pub mod young;

#[cfg(test)]
mod plateau_check;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::time::Instant;

use deeptime_format::GridDesc;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use params::SurfaceParams;

use crate::deposits::recorded::RecordedHistory;
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::plates::state::{OCEANIC, OROGENY_ANDEAN, OROGENY_HIMALAYAN, pool, regime};
use crate::stage::{Stage, StageContext, StageEstimate, from_state, to_state};
use crate::world::{Field, WorldState};
use facies::{Kind, Setting};
use grid::{Dims, box_blur};
use lithology::Classes;
use pflood::{BASE, LAND};
use strata_log::{ColumnMut, IN_SITU, StrataLog};

/// The stage's name in a pipeline (`deeptime stage surface`).
pub const STAGE_NAME: &str = "surface";

/// Fields the stage writes every step.
pub mod field {
    /// Surface elevation, m (datum of the plate model).
    pub const ELEVATION_M: &str = "surface/elevation_m";
    /// Net erosion of the step, m/Myr.
    pub const EROSION: &str = "surface/erosion_m_myr";
    /// Net deposition of the step, m/Myr.
    pub const DEPOSITION: &str = "surface/deposition_m_myr";
    /// Thickness of the strata log, m.
    pub const SEDIMENT: &str = "surface/sediment_m";
    /// The plate and the rigid group under each cell after the last step (`plates/plate_id`,
    /// `plates/group_id` as the stage saw them): the next step's advection carries the offset only
    /// from cells that held the same crust (`advect::bilinear_same`).
    pub const CRUST_PLATE: &str = "surface/crust_plate";
    pub const CRUST_GROUP: &str = "surface/crust_group";
    /// The surface rock class (index into `synth::EVERY_CLASS`).
    pub const CLASS: &str = "surface/class";
    /// log₁₀ of the drainage area in km² (land; NaN at sea).
    pub const DRAINAGE: &str = "surface/drainage";
    /// Regolith thickness, m.
    pub const REGOLITH: &str = "surface/regolith_m";
    /// Elevation change the plate model applies to L0 at its next step, m.
    pub const FEEDBACK: &str = "surface/feedback_m";
    /// Material added (+) or removed (−) by surface processes, m (the plate model's rock pools).
    pub const MATERIAL: &str = "surface/material_m";
    /// Written by the plate model: the feedback it applied, sampled back onto L1, m.
    pub const FEEDBACK_APPLIED: &str = "surface/feedback_applied_m";
    /// Centre-line thickness of the glaciers of the step's last ice solve, m (`glacial.rs`).
    pub const ICE: &str = "surface/ice_thickness_m";
    /// Glacial erosion of the step, m/Myr.
    pub const GLACIAL_EROSION: &str = "surface/glacial_erosion_m_myr";
}

/// Keyframe layers (the group adds the plate model's `elevation`, `plates`, `crust_age`,
/// `regime`; `elevation` shows the surface's).
pub const KEYFRAME_LAYERS: [&str; 7] = [
    field::EROSION,
    field::DEPOSITION,
    field::SEDIMENT,
    field::CLASS,
    field::DRAINAGE,
    field::REGOLITH,
    field::ICE,
];

/// Cumulative volumes of the run, km³ (block metric: cell area (r·300 m)²).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Totals {
    pub eroded_km3: f64,
    pub fluvial_km3: f64,
    pub lake_km3: f64,
    pub marine_km3: f64,
    pub carbonate_km3: f64,
    pub evaporite_km3: f64,
    pub to_sea_km3: f64,
    pub weathered_km3: f64,
    pub isostasy_km3: f64,
    /// Sediment moved down submarine slopes by slope failure (`marine::slope_failure`).
    #[serde(default)]
    pub slope_failure_km3: f64,
    /// Σ(Δh)·a − (carbonate + evaporite): what the processes created or lost, km³.
    pub mass_residual_km3: f64,
    pub steps: u64,
    pub spl_iterations: u64,
    /// Conjugate-gradient iterations of the diffusion solves.
    pub diffusion_iterations: u64,
    pub sinks: u64,
    pub frames: u64,
    /// Land erosion by process and elevation band (`erosion_bands`).
    #[serde(default)]
    pub by_band: erosion_bands::ErosionBands,
}

/// The surface's state, kept in `WorldState::surface` (checkpoints carry it; later stages read
/// the strata log and the regolith).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfaceState {
    pub grid: GridDesc,
    /// Plate steps processed.
    pub step: u64,
    pub t_myr: f64,
    /// Surface elevation minus the plate model's after the last step, m.
    pub offset_m: Vec<f32>,
    /// Surface elevation after the last step, m.
    pub elevation_m: Vec<f32>,
    pub regolith_m: Vec<f32>,
    /// `weathering::{SOIL, SAPROLITE, LATERITE, BAUXITE}`.
    pub regolith_kind: Vec<u8>,
    pub log: StrataLog,
    /// Every plate's Ω at the end of the last step (what the next plate step turns it by).
    pub omega: Vec<[f64; 3]>,
    pub acc: fields::FrameAcc,
    pub events: events::EventState,
    pub totals: Totals,
    /// Sea level of the last step, m.
    pub sea_level_m: f64,
    /// The plate's load and deflection (`flexure.rs`); None while `flexure.strength` is 0.
    pub flexure: Option<flexure::FlexState>,
    /// Inherited iron-formation basins (`deposits::bif_basins`): 1 in a basin, per cell, carried
    /// with the crust; empty until dealt at the first step of a run that inherits them.
    #[serde(default)]
    pub bif_basin: Vec<f32>,
}

/// Working arrays (rebuilt after a resume, never saved).
struct Scratch {
    d: Dims,
    h: Vec<f64>,
    h_new: Vec<f64>,
    filled: Vec<f64>,
    base_z: Vec<f64>,
    kind: Vec<u8>,
    routing: routing::Routing,
    area: Vec<f64>,
    qin: Vec<f64>,
    /// The critical slope's cut of the last solve (`spl::solve_cut`), m.
    cut: Vec<f64>,
    kf: Vec<f64>,
    kd: Vec<f64>,
    weight: Vec<f32>,
    mix: Vec<[f64; spl::FAMILIES]>,
    fill: marine::FillScratch,
    /// K with the glaciers' share of each cell taken out (ice, not rivers, erodes under it).
    kf_eff: Vec<f64>,
    ice: glacial::IceState,
    glacial: glacial::GlacialOut,
}

impl Scratch {
    fn new(d: Dims) -> Scratch {
        let n = d.cells();
        Scratch {
            d,
            h: vec![0.0; n],
            h_new: vec![0.0; n],
            filled: vec![0.0; n],
            base_z: vec![0.0; n],
            kind: vec![LAND; n],
            routing: routing::Routing::default(),
            area: vec![0.0; n],
            qin: vec![0.0; n],
            cut: vec![0.0; n],
            kf: vec![0.0; n],
            kd: vec![0.0; n],
            weight: vec![1.0; n],
            mix: Vec::new(),
            fill: marine::FillScratch::default(),
            kf_eff: vec![0.0; n],
            ice: glacial::IceState::default(),
            glacial: glacial::GlacialOut::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Saved {
    done: u64,
}

/// The surface stage.
pub struct SurfaceStage {
    preset: PlanetPreset,
    p: SurfaceParams,
    total: u64,
    done: u64,
    classes: Classes,
    erod: Vec<f64>,
    diffu: Vec<f64>,
    fam: Vec<u8>,
    scratch: Option<Scratch>,
    /// The flexure solver's transforms and working arrays (rebuilt after a resume).
    flex_solver: Option<flexure::Solver>,
    /// Milliseconds per process in this session.
    timings: BTreeMap<&'static str, f64>,
}

/// Per-cell deposit components of one step, m.
#[derive(Clone, Default)]
struct Parts {
    eroded: Vec<f32>,
    fluvial: Vec<f32>,
    lake: Vec<f32>,
    marine: Vec<f32>,
    carbonate: Vec<f32>,
    evaporite: Vec<f32>,
    /// Till and glaciomarine diamict (`glacial.rs`).
    till: Vec<f32>,
    /// Glacial erosion (also in `eroded`).
    glacial_eroded: Vec<f32>,
    /// Stream-power erosion, the critical slope's cut included (also in `eroded`).
    spl_eroded: Vec<f32>,
    /// The critical slope's share of `spl_eroded`.
    threshold_cut: Vec<f32>,
    /// Hillslope (and shoreline) diffusion's lowering (also in `eroded`).
    diffusion_eroded: Vec<f32>,
    prov_marine: Vec<u16>,
}

impl Parts {
    /// Every part empty, written in parallel (P1-19).
    fn new(n: usize) -> Parts {
        use crate::alloc::filled;
        Parts {
            eroded: filled(n, 0.0),
            fluvial: filled(n, 0.0),
            lake: filled(n, 0.0),
            marine: filled(n, 0.0),
            carbonate: filled(n, 0.0),
            evaporite: filled(n, 0.0),
            till: filled(n, 0.0),
            glacial_eroded: filled(n, 0.0),
            spl_eroded: filled(n, 0.0),
            threshold_cut: filled(n, 0.0),
            diffusion_eroded: filled(n, 0.0),
            prov_marine: filled(n, IN_SITU),
        }
    }
}

/// `WorldState::globals`: cell-steps so far where another plate's crust arrived under a cell and
/// the surface's offset started again from the plate model's elevation (`advect::bilinear_same`).
pub const FOREIGN_CRUST_GLOBAL: &str = "surface/foreign_crust_cell_steps";
/// The inherited iron-formation basins dealt at the first step (`deposits::bif_basins`), and the
/// share of the stable continental interiors they cover.
pub const BIF_BASINS_GLOBAL: &str = "surface/bif_basins";
pub const BIF_BASIN_SHARE_GLOBAL: &str = "surface/bif_basin_share";

/// The plate fields the stage reads each step.
struct PlateFields {
    elevation: Vec<f32>,
    uplift: Vec<f32>,
    plate_id: Vec<f32>,
    /// `plates/group_id`; absent in worlds built without the plate model's L1 sampling (tests).
    group_id: Option<Vec<f32>>,
    crust_type: Vec<f32>,
    crust_age: Vec<f32>,
    regime: Vec<f32>,
    orogeny_type: Vec<f32>,
    orogeny_age: Vec<f32>,
    top_pool: Vec<f32>,
    flags: Vec<f32>,
    oxidation: Vec<f32>,
    convergence: Vec<f32>,
    obliquity: Vec<f32>,
    slab_age: Vec<f32>,
    heat_flow: Vec<f32>,
    thickness: Vec<f32>,
}

impl PlateFields {
    fn read(world: &WorldState, n: usize) -> Result<PlateFields, CoreError> {
        let get = |name: &str| -> Result<Vec<f32>, CoreError> {
            world
                .fields
                .get(name)
                .filter(|f| f.data.len() == n)
                .map(|f| f.data.clone())
                .ok_or_else(|| {
                    CoreError::InvalidState(format!(
                        "the surface stage needs the plates' L1 field {name} every step \
                         (tectonics.l1_every_steps = 1)"
                    ))
                })
        };
        Ok(PlateFields {
            elevation: get("plates/elevation_m")?,
            uplift: get("plates/uplift_mm_yr")?,
            plate_id: get("plates/plate_id")?,
            group_id: get("plates/group_id").ok(),
            crust_type: get("plates/crust_type")?,
            crust_age: get("crust_age")?,
            regime: get("regime")?,
            orogeny_type: get("plates/orogeny_type")?,
            orogeny_age: get("plates/orogeny_age_myr")?,
            top_pool: get("plates/top_pool")?,
            flags: get("plates/flags")?,
            oxidation: get("plates/oxidation")?,
            convergence: get("plates/convergence_mm_yr")?,
            obliquity: get("plates/obliquity_deg")?,
            slab_age: get("plates/slab_age_myr")?,
            heat_flow: get("plates/heat_flow_mw_m2")?,
            thickness: get("plates/thickness_m")?,
        })
    }
}

/// The basement's rock class where no sediment or regolith covers it: from the crust type, the
/// regime, the orogeny and the top rock pool of the plate model's column. Arcs are andesite; a
/// young Andean orogeny (the plate model's subduction uplift, which reaches far behind the arc)
/// makes only its volcanic pool andesite (P1-07: at first every basement under a young Andean
/// orogeny was andesite, a third of the land at 16k).
pub fn basement_class(
    cls: &Classes,
    crust_type: u8,
    reg: u8,
    orogeny: u8,
    orogeny_age: f32,
    top_pool: u8,
) -> u16 {
    if crust_type == OCEANIC {
        return if usize::from(top_pool) == pool::MAFIC_PLUTONIC {
            cls.gabbro
        } else {
            cls.basalt
        };
    }
    let young = |myr: f32| orogeny_age.is_finite() && orogeny_age < myr;
    if reg == regime::HOTSPOT {
        return cls.basalt;
    }
    if reg == regime::ARC {
        return cls.andesite;
    }
    match usize::from(top_pool) {
        pool::SEDIMENT => cls.mudstone,
        pool::SEDIMENTARY => cls.sandstone,
        pool::FELSIC_VOLCANIC => {
            if orogeny == OROGENY_ANDEAN && young(30.0) {
                cls.andesite
            } else {
                cls.rhyolite
            }
        }
        pool::MAFIC_VOLCANIC => cls.basalt,
        pool::METAMORPHIC => {
            if orogeny == OROGENY_HIMALAYAN && young(100.0) {
                cls.schist
            } else {
                cls.gneiss
            }
        }
        pool::FELSIC_PLUTONIC => {
            if orogeny == OROGENY_ANDEAN {
                cls.granodiorite
            } else {
                cls.granite
            }
        }
        pool::MAFIC_PLUTONIC => cls.gabbro,
        _ => cls.granite,
    }
}

/// The provenance code of a sediment flux split by source family (none for an empty flux).
fn dominant(mix: &[f64; spl::FAMILIES]) -> Option<u16> {
    let tot: f64 = mix.iter().sum();
    if tot <= 0.0 {
        return None;
    }
    let (k, v) = mix
        .iter()
        .enumerate()
        .max_by(|x, y| x.1.total_cmp(y.1).then(y.0.cmp(&x.0)))?;
    Some(strata_log::provenance(k, v / tot))
}

/// The climate's drawdown carbonate settles on sea floor this shallow, m: above the carbonate
/// compensation depth (~4–5 km today), where calcite is preserved.
pub const CARBONATE_FLOOR_M: f64 = 4000.0;

/// Regolith at least this thick is the surface rock (for erodibility, diffusivity and the
/// `surface/class` layer), m.
pub const REGOLITH_SHOWS_M: f32 = 10.0;

fn regolith_class(cls: &Classes, kind: u8) -> u16 {
    match kind {
        weathering::SAPROLITE => cls.saprolite,
        weathering::LATERITE => cls.laterite,
        weathering::BAUXITE => cls.bauxite,
        _ => cls.soil,
    }
}

/// Estimate for a preset with a surface section (plan §5.7.5: LEM 40 B/cell, strata log 12·K
/// B/cell; the deposit frames and fields on top; cost ≈ 1 µs per cell per LEM step, §8.2).
pub fn estimate(p: &PlanetPreset) -> StageEstimate {
    let s = p.surface.clone().unwrap_or_default();
    let c = u64::from(p.scale.circumference_blocks);
    let r = u64::from(p.resolution.l1_r);
    let cells = (c / r) * (c / r / 2);
    let log = StrataLog::bytes_per_cell(s.log_layers);
    let per_cell = retained_per_cell(&s)
        + 13 * 8 // f64 working arrays
        + 3 * 4 // the step's erosion by process (`erosion_bands`)
        + log // the log's advected copy
        + 21 * 8; // the glaciers' ice state, loads and passes (`glacial.rs`)
    let frames = (p.run_myr() / s.frame_every_myr).ceil().max(1.0) as u64;
    let history = frames * cells * 12; // packed frames, ~12 B/cell after LZ4 (measured in P1-04)
    let steps = p.plate_steps() as f64 * f64::from(s.substeps);
    StageEstimate {
        bytes: cells * per_cell + history + crate::plates::estimate(p).bytes,
        seconds: cells as f64 * steps * 1.0e-6,
        history_bytes: history,
    }
}

/// What of the surface stage's per-cell estimate stays in the world after its last step: the
/// strata log, the fields, the frame accumulator, mix, weights and kinds, and the plate's state
/// (`flexure.rs`) and its solver's transforms. The solve's own vectors (some 30 more per cell)
/// live only inside a surface step, far under the later stages' peak.
fn retained_per_cell(s: &SurfaceParams) -> u64 {
    StrataLog::bytes_per_cell(s.log_layers) // the log
        + 30 * 4 // fields
        + (fields::STEP_FIELDS.len() as u64) * 4 // the frame accumulator
        + 24 // mix, weights, kinds
        + if s.flexure.active() { 6 * 4 + 12 * 8 } else { 0 }
}

/// The world the coupled stages leave for the stages after them, without the recorded history:
/// the plates' estimate, the surface's retained part per L1 cell ([`estimate`] less the step's
/// working arrays, the log's advected copy and the glaciers' passes) and the climate's 24 monthly
/// maps of its last snapshot (P1-25; 96 B per L1 cell). P1-17 (2026-10-05, 500 Myr, scattered
/// continents, history excluded): 651 MiB on quick 16k against the 576 MiB the run held before
/// the young stage, 2,127 MiB on full 16k against 2,010 MiB.
pub fn retained_bytes(p: &PlanetPreset) -> u64 {
    let s = p.surface.clone().unwrap_or_default();
    let c = u64::from(p.scale.circumference_blocks);
    let r = u64::from(p.resolution.l1_r);
    let cells = (c / r) * (c / r / 2);
    let monthly = if p.climate.is_some() { 24 * 4 } else { 0 };
    cells * (retained_per_cell(&s) + monthly) + crate::plates::estimate(p).bytes
}

impl SurfaceStage {
    pub fn new(preset: PlanetPreset) -> SurfaceStage {
        let p = preset.surface.clone().unwrap_or_default();
        let total = preset.plate_steps();
        let table = lithology::table();
        SurfaceStage {
            preset,
            p,
            total,
            done: 0,
            classes: Classes::get(),
            erod: lithology::erodibility(),
            diffu: lithology::diffusivity(),
            fam: table.iter().map(|c| c.source_family).collect(),
            scratch: None,
            flex_solver: None,
            timings: BTreeMap::new(),
        }
    }

    fn grid(&self) -> Result<GridDesc, CoreError> {
        Ok(GridDesc::for_planet(
            "L1",
            self.preset.scale.circumference_blocks,
            self.preset.resolution.l1_r,
        )?)
    }

    /// Where the run sits in Earth's history (the secular clocks of the deposit types).
    fn epoch(&self) -> crate::deposits::clocks::Epoch {
        crate::deposits::clocks::Epoch::of_preset(&self.preset)
    }

    fn time(&mut self, what: &'static str, since: Instant) {
        *self.timings.entry(what).or_insert(0.0) += since.elapsed().as_secs_f64() * 1000.0;
    }

    fn advance(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        let grid = self.grid()?;
        let n = grid.cells();
        let (w, hgt) = (grid.w as usize, grid.h as usize);
        let dx = f64::from(grid.r) * self.preset.scale.m_per_block_h;
        let d = Dims::new(w, hgt, dx, true);
        let a = d.cell_area();
        let plates = world.plates.as_ref().ok_or_else(|| {
            CoreError::InvalidState("the surface stage runs after a plate step".into())
        })?;
        let t_end_myr = plates.t_myr;
        let dt_myr = self.preset.dt_myr();
        let dt = dt_myr * 1.0e6;
        let omega_now: Vec<[f64; 3]> = plates
            .plates
            .iter()
            .map(|p| if p.alive { p.omega } else { [0.0; 3] })
            .collect();
        let parents: Vec<Option<u32>> = plates.plates.iter().map(|p| p.parent).collect();
        let pstep = plates.step;
        let pf = PlateFields::read(world, n)?;
        let cls = self.classes;
        let p = self.p.clone();

        // 1. The state, carried with the plates.
        let started = Instant::now();
        // Cells where another plate's crust arrived this step (the offset starts again there).
        let mut foreign_cells = 0u64;
        let mut st = match world.surface.take() {
            Some(mut st) => {
                ctx.reserve(StrataLog::bytes_per_cell(st.log.k) * n as u64 + 16 * n as u64)?;
                // Plates born this step moved with their parent's Ω.
                let omega: Vec<[f64; 3]> = (0..parents.len())
                    .map(|mut k| {
                        for _ in 0..64 {
                            if k < st.omega.len() {
                                return st.omega[k];
                            }
                            match parents[k] {
                                Some(pk) => k = pk as usize,
                                None => break,
                            }
                        }
                        [0.0; 3]
                    })
                    .collect();
                let src = advect::sources(&grid, &pf.plate_id, &omega, dt);
                // The elevation moves with its own crust and takes the plate model's uplift
                // (`advect::carried_elevation`); the offset is what stands over the plate model's
                // elevation here.
                let prev_ids = |name: &str| {
                    world
                        .fields
                        .get(name)
                        .filter(|f| f.data.len() == n)
                        .map(|f| f.data.as_slice())
                };
                let carry = advect::Carry {
                    w,
                    h: hgt,
                    src: &src,
                    prev_elevation: &st.elevation_m,
                    prev_plate: prev_ids(field::CRUST_PLATE),
                    prev_group: prev_ids(field::CRUST_GROUP),
                    plate: &pf.plate_id,
                    group: pf.group_id.as_deref(),
                    plate_elevation: &pf.elevation,
                    uplift_mm_yr: &pf.uplift,
                    parents: &parents,
                    known_plates: st.omega.len(),
                    dt_yr: dt,
                };
                let (carried, foreign) = advect::carried_elevation(&carry);
                // The plate's load and deflection move with the same crust.
                if p.flexure.active() {
                    st.flexure
                        .get_or_insert_default()
                        .carry(&carry, &st.elevation_m);
                } else {
                    st.flexure = None;
                }
                foreign_cells = foreign;
                st.offset_m = carried
                    .par_iter()
                    .zip(pf.elevation.par_iter())
                    .map(|(h, z)| h - z)
                    .collect();
                let reg = std::mem::take(&mut st.regolith_m);
                st.regolith_m = src
                    .par_iter()
                    .map(|&[fi, fj]| advect::bilinear(w, hgt, &reg, fi, fj).max(0.0))
                    .collect();
                let nearest: Vec<usize> = src
                    .par_iter()
                    .map(|&[fi, fj]| advect::nearest(w, hgt, fi, fj))
                    .collect();
                let kinds = std::mem::take(&mut st.regolith_kind);
                st.regolith_kind = nearest.par_iter().map(|&s| kinds[s]).collect();
                if st.bif_basin.len() == n {
                    let basin = std::mem::take(&mut st.bif_basin);
                    st.bif_basin = nearest.par_iter().map(|&s| basin[s]).collect();
                }
                // Each layer slot is written once, in parallel: the source column's used layers,
                // then empty ones (P1-17: `StrataLog::new`'s serial zero fill of the whole log,
                // 129 MB a step at 16k quick and 515 MB at full, then the copy over it).
                let k = st.log.k as usize;
                let old = std::mem::take(&mut st.log);
                let layers: Vec<strata_log::Layer> = (0..n * k)
                    .into_par_iter()
                    .map(|x| {
                        let (c, i) = (x / k, x % k);
                        let s = nearest[c];
                        if i < usize::from(old.count[s]) {
                            old.layers[s * k + i]
                        } else {
                            strata_log::Layer::default()
                        }
                    })
                    .collect();
                let count: Vec<u8> = nearest.par_iter().map(|&s| old.count[s]).collect();
                st.log = StrataLog {
                    k: old.k,
                    count,
                    layers,
                };
                // Crust born this step starts bare.
                let mut newborn = vec![false; n];
                for (c, born) in newborn.iter_mut().enumerate() {
                    let age = pf.crust_age[c];
                    if age >= 0.0 && age < dt_myr as f32 {
                        st.offset_m[c] = 0.0;
                        st.regolith_m[c] = 0.0;
                        st.regolith_kind[c] = weathering::SOIL;
                        st.log.clear(c);
                        if let Some(b) = st.bif_basin.get_mut(c) {
                            *b = 0.0;
                        }
                        *born = true;
                    }
                }
                // The frame keeps where each cell's crust was when the frame began (P1-14).
                st.acc.compose_back(&nearest, &newborn);
                st
            }
            None => {
                // The state and scratch; not the recorded history, which grows a frame at a
                // time and goes to disk when the cap needs the room (P1-17).
                let e = estimate(&self.preset);
                ctx.reserve((e.bytes - e.history_bytes) / 2)?;
                SurfaceState {
                    grid: grid.clone(),
                    step: 0,
                    t_myr: t_end_myr - dt_myr,
                    offset_m: vec![0.0; n],
                    elevation_m: pf.elevation.clone(),
                    regolith_m: vec![0.0; n],
                    regolith_kind: vec![weathering::SOIL; n],
                    log: StrataLog::new(n, p.log_layers),
                    omega: omega_now.clone(),
                    acc: fields::FrameAcc {
                        t_start_years: (t_end_myr - dt_myr) * 1.0e6,
                        ..fields::FrameAcc::default()
                    },
                    events: events::EventState::default(),
                    totals: Totals::default(),
                    sea_level_m: p.sea_level.at(t_end_myr - dt_myr),
                    flexure: None,
                    bif_basin: Vec::new(),
                }
            }
        };
        self.time("advect", started);

        // 2. Elevation, sea, climate, surface rock.
        let started = Instant::now();
        if self.scratch.as_ref().is_none_or(|s| s.d != d) {
            self.scratch = Some(Scratch::new(d));
        }
        let mut sc = self.scratch.take().expect("built above");
        sc.h.par_iter_mut()
            .enumerate()
            .for_each(|(c, h)| *h = f64::from(pf.elevation[c]) + f64::from(st.offset_m[c]));
        let h_start = crate::alloc::copied(&sc.h);
        // Sea level: the climate stage's when it ran this step (the curve plus glacio-eustasy:
        // ice sheets lower the sea and expose the shelves, `climate::ice`), else the curve.
        let sea = world
            .globals
            .get(crate::climate::SEA_LEVEL_GLOBAL)
            .copied()
            .filter(|z| z.is_finite())
            .unwrap_or_else(|| p.sea_level.at(t_end_myr));
        let cap_m = world
            .globals
            .get(crate::climate::CAP_CARBONATE_GLOBAL)
            .copied()
            .filter(|x| x.is_finite() && *x > 0.0)
            .unwrap_or(0.0);
        let lat_of = |j: usize| fields::row_lat(&grid, j);
        let clim = climate::snapshot(world, &d, &lat_of, &sc.h, sea);
        // The glaciers' climate (`glacial.rs`): the ELA of the climate stage; none without it.
        // The deep-time surface is physics: glaciers as wide as they are (the valley width is the
        // young landscape's readability setting).
        let gp = glacial::GlacialParams {
            valley_width: 1.0,
            ..p.glacial.clone()
        };
        let gravity = self.preset.planet.gravity_m_s2;
        let ela_z = if gp.active() {
            glacial::ela_z_from(
                world
                    .fields
                    .get(glacial::source::ELA)
                    .map(|f| f.data.as_slice()),
                n,
                sea,
                0.0,
            )
        } else {
            None
        };
        // The climate's own ice (FORMAT.md, "The climate's ice fields"), when it has it: glaciers
        // only where it had ice during the step, eroding for the share of the step it stood, and
        // no rivers under thick ice.
        let field_n = |name: &str| -> Option<Vec<f32>> {
            world
                .fields
                .get(name)
                .filter(|f| f.data.len() == n)
                .map(|f| f.data.clone())
        };
        let ice_allowed = field_n(glacial::source::ICE_MAX_THICKNESS)
            .or_else(|| field_n(glacial::source::ICE_THICKNESS));
        let ice_share = field_n(glacial::source::ICE_TIME_SHARE);
        let climate_ice = field_n(glacial::source::ICE_THICKNESS);
        let precip_m_yr: Vec<f32> = clim.precip_mm.par_iter().map(|&p| p / 1000.0).collect();
        let mut gtot = glacial::Totals::read(&world.notes);
        let mut ice_cells = 0u64;
        let mut open: Vec<f64> =
            sc.h.iter()
                .map(|&z| if z <= sea { 1.0 } else { 0.0 })
                .collect();
        box_blur(&d, &mut open, ((150_000.0 / dx).round() as usize).max(1), 2);
        let restricted: Vec<bool> = (0..n)
            .into_par_iter()
            .map(|c| sc.h[c] <= sea && open[c] < 0.4)
            .collect();
        let basement: Vec<u16> = (0..n)
            .into_par_iter()
            .map(|c| {
                basement_class(
                    &cls,
                    pf.crust_type[c] as u8,
                    pf.regime[c] as u8,
                    pf.orogeny_type[c] as u8,
                    pf.orogeny_age[c],
                    if pf.top_pool[c].is_finite() {
                        pf.top_pool[c] as u8
                    } else {
                        255
                    },
                )
            })
            .collect();
        // The rock a cell shows and erodes: regolith only where it is thick (a few metres of soil
        // are gone in the first thousand years of a 2-Myr step), else the log's top, else the
        // basement.
        let surface_class = |st: &SurfaceState, c: usize| -> u16 {
            if st.regolith_m[c] >= REGOLITH_SHOWS_M {
                regolith_class(&cls, st.regolith_kind[c])
            } else if st.log.total(c) >= 0.5 {
                st.log.top(c).map_or(basement[c], |l| l.class)
            } else {
                basement[c]
            }
        };
        let class0: Vec<u16> = (0..n)
            .into_par_iter()
            .map(|c| surface_class(&st, c))
            .collect();
        // Cell by cell, so in parallel (P1-19).
        let fam: Vec<u8> = class0
            .par_iter()
            .map(|&k| self.fam[usize::from(k)])
            .collect();
        let erod = &self.erod;
        sc.kf
            .par_iter_mut()
            .zip(sc.weight.par_iter_mut())
            .enumerate()
            .for_each(|(c, (kf, weight))| {
                *kf = p.k_spl * erod[usize::from(class0[c])];
                *weight = (clim.runoff_m_yr[c] / p.runoff_ref_m_yr as f32).max(0.05);
            });
        self.time("setup", started);

        // 3. The LEM steps.
        let mut parts = Parts::new(n);
        let subs = p.substeps.max(1) as usize;
        let dts = dt / subs as f64;
        let spl_p = spl::SplParams {
            m: p.m_spl,
            n: p.n_spl,
            g: p.g_deposition,
            dt: dts,
            sc: p.threshold_slope,
            max_iter: p.spl_max_iter as usize,
            tol_m: 0.01,
        };
        let carb_p = carbonate::CarbonateParams {
            max_rate_m_yr: p.carbonate_max_mm_yr * 1.0e-3,
            extinction_per_m: p.carbonate_extinction_per_m,
            surface_light: p.carbonate_surface_light,
            saturating_light: p.carbonate_saturating_light,
            tidal_range_m: p.tidal_range_m,
            min_temp_c: p.carbonate_min_temp_c,
            clastic_m: p.carbonate_clastic_m,
        };
        let fill_p = marine::FillParams {
            sea_level: sea,
            fill_depth_m: p.marine_fill_depth_m,
            shelf_slope: p.shelf_slope,
            max_cells: 200_000,
        };
        let mut to_sea = 0.0;
        let mut iterations = 0u64;
        let mut cg_iterations = 0u64;
        let mut sinks = 0u64;
        // Submarine slope failure (`marine::slope_failure`): what each cell may lose, its loose and
        // logged sediment at the start of the step plus what the step lays on it.
        let failure = (p.submarine_slope_deg > 0.0).then(|| marine::FailureParams {
            critical_slope: p.submarine_slope_deg.to_radians().tan(),
            rate: marine::FAILURE_RATE,
            iterations: marine::FAILURE_ITERATIONS,
        });
        let h_step0 = crate::alloc::copied(&sc.h);
        let avail0: Vec<f64> = (0..n)
            .into_par_iter()
            .map(|c| st.log.total(c) + f64::from(st.regolith_m[c]))
            .collect();
        let mut failed_m_cells = 0.0f64;
        for sub in 0..subs {
            let started = Instant::now();
            pflood::open_sea(&d, &sc.h, sea, (n / 500).max(1), &mut sc.kind);
            sc.base_z.iter_mut().for_each(|b| *b = sea);
            sinks += pflood::flood(&d, &sc.h, &sc.base_z, &mut sc.kind, &mut sc.filled) as u64;
            self.time("pflood", started);
            let started = Instant::now();
            routing::receivers(&d, &sc.filled, &sc.kind, &mut sc.routing);
            routing::stack(&d, &sc.kind, &mut sc.routing);
            routing::accumulate(&d, &sc.routing, &sc.weight, &mut sc.area);
            self.time("routing", started);
            // Under a glacier the ice erodes, not a river: K keeps the ice-free share of a cell.
            if let Some(ela) = &ela_z {
                let started = Instant::now();
                let ic = glacial::IceClimate {
                    ela_z: ela,
                    precip_m_yr: &precip_m_yr,
                    t_c: &clim.t_c,
                    allowed: ice_allowed.as_deref(),
                    time_share: ice_share.as_deref(),
                };
                sc.ice = std::mem::take(&mut sc.glacial.ice);
                glacial::ice(
                    &d,
                    &sc.routing,
                    &sc.kind,
                    &sc.h,
                    &sc.filled,
                    sea,
                    &ic,
                    &gp,
                    gravity,
                    &mut sc.ice,
                );
                let (kf, cover) = (&sc.kf, &sc.ice.cover);
                sc.kf_eff
                    .par_iter_mut()
                    .enumerate()
                    .for_each(|(c, kf_eff)| {
                        // The glacier's share of the cell, or the climate's thick ice.
                        let thick = climate_ice.as_ref().map_or(0.0, |v| {
                            crate::climate::ebm::smoothstep(10.0, 100.0, f64::from(v[c]))
                        });
                        *kf_eff = kf[c] * (1.0 - cover[c].max(thick)).max(0.0);
                    });
                self.time("glacial", started);
            } else {
                sc.kf_eff.copy_from_slice(&sc.kf);
            }
            let started = Instant::now();
            let input = spl::SplInput {
                kind: &sc.kind,
                h0: &sc.h,
                filled: &sc.filled,
                base_z: &sc.base_z,
                area: &sc.area,
                kf: &sc.kf_eff,
            };
            let rep = spl::solve_cut(
                &d,
                &sc.routing,
                &input,
                &spl_p,
                &mut sc.h_new,
                &mut sc.qin,
                Some(&mut sc.cut),
            );
            iterations += rep.iterations as u64;
            if sub + 1 == subs {
                spl::families(&d, &sc.routing, &input, &fam, dts, &sc.h_new, &mut sc.mix);
            }
            {
                // Cell by cell, so in parallel (P1-17).
                let (kind, h, h_new, filled, cut) =
                    (&sc.kind, &sc.h, &sc.h_new, &sc.filled, &sc.cut);
                parts
                    .eroded
                    .par_iter_mut()
                    .zip(parts.spl_eroded.par_iter_mut())
                    .zip(parts.threshold_cut.par_iter_mut())
                    .zip(parts.lake.par_iter_mut())
                    .zip(parts.fluvial.par_iter_mut())
                    .enumerate()
                    .for_each(
                        |(c, ((((eroded, spl_eroded), threshold_cut), lake), fluvial))| {
                            if kind[c] == BASE {
                                return;
                            }
                            let dh = h_new[c] - h[c];
                            *threshold_cut += cut[c] as f32;
                            if dh < 0.0 {
                                *eroded += (-dh) as f32;
                                *spl_eroded += (-dh) as f32;
                            } else if filled[c] - h[c] > spl::LAKE_TOL_M {
                                *lake += dh as f32;
                            } else {
                                *fluvial += dh as f32;
                            }
                        },
                    );
            }
            std::mem::swap(&mut sc.h, &mut sc.h_new);
            self.time("spl", started);

            // River loads into the sea.
            let started = Instant::now();
            let sea_mask: Vec<bool> = sc.kind.par_iter().map(|&k| k == BASE).collect();
            // The mouths, ascending (found in parallel; the fills change neither test).
            let (kind, qin) = (&sc.kind, &sc.qin);
            let mouths: Vec<usize> = (0..n)
                .into_par_iter()
                .filter(|&m| kind[m] == BASE && qin[m] > 0.0)
                .collect();
            for m in mouths {
                let vol = sc.qin[m] * dts;
                to_sea += vol;
                marine::fill(
                    &d,
                    &sea_mask,
                    m,
                    vol,
                    &fill_p,
                    &mut sc.h,
                    &mut parts.marine,
                    &mut sc.fill,
                );
                if sub + 1 == subs
                    && let Some(code) = sc.mix.get(m).and_then(dominant)
                {
                    // Cells this mouth reached first take its provenance.
                    for &c in &sc.fill.touched {
                        let c = c as usize;
                        if parts.prov_marine[c] == IN_SITU {
                            parts.prov_marine[c] = code;
                        }
                    }
                }
            }
            self.time("marine_fill", started);

            // Glaciers: erosion, till, and their loads into lakes and the sea (`glacial.rs`).
            if let Some(ela) = &ela_z {
                let started = Instant::now();
                let ic = glacial::IceClimate {
                    ela_z: ela,
                    precip_m_yr: &precip_m_yr,
                    t_c: &clim.t_c,
                    allowed: ice_allowed.as_deref(),
                    time_share: ice_share.as_deref(),
                };
                let mut g = std::mem::take(&mut sc.glacial);
                // One ice state, not two: the cover's solve hands its arrays on.
                g.ice = std::mem::take(&mut sc.ice);
                let above: Vec<bool> = sc.h.par_iter().map(|&z| z >= sea).collect();
                glacial::step(
                    &d,
                    &sc.routing,
                    &sc.kind,
                    &sc.filled,
                    sea,
                    &ic,
                    &gp,
                    gravity,
                    dts,
                    &mut sc.h,
                    &mut g,
                );
                // Cell by cell; the count is an integer sum, so in parallel (P1-17).
                let h = &sc.h;
                gtot.cut_below_sea_cells += parts
                    .eroded
                    .par_iter_mut()
                    .zip(parts.glacial_eroded.par_iter_mut())
                    .zip(parts.till.par_iter_mut())
                    .zip(parts.lake.par_iter_mut())
                    .enumerate()
                    .map(|(c, (((eroded, glacial_eroded), till), lake))| {
                        let e = g.eroded[c] as f32;
                        *eroded += e;
                        *glacial_eroded += e;
                        *till += g.till[c] as f32;
                        *lake += g.lake[c] as f32;
                        u64::from(above[c] && h[c] < sea && g.eroded[c] > 0.0)
                    })
                    .sum::<u64>();
                let sea_mask: Vec<bool> = sc.kind.par_iter().map(|&k| k == BASE).collect();
                // (the base cells with a load, ascending: the others have none)
                for &m in &g.sea_cells {
                    let m = m as usize;
                    let (ice_load, melt_load) = (g.sea_ice_load[m], g.sea_melt_load[m]);
                    if ice_load > 0.0 {
                        to_sea += ice_load;
                        marine::fill(
                            &d,
                            &sea_mask,
                            m,
                            ice_load,
                            &fill_p,
                            &mut sc.h,
                            &mut parts.till,
                            &mut sc.fill,
                        );
                    }
                    if melt_load > 0.0 {
                        to_sea += melt_load;
                        marine::fill(
                            &d,
                            &sea_mask,
                            m,
                            melt_load,
                            &fill_p,
                            &mut sc.h,
                            &mut parts.marine,
                            &mut sc.fill,
                        );
                    }
                }
                let km3 = 1.0e-9;
                gtot.eroded_km3 += g.eroded_m3 * km3;
                gtot.till_km3 += g.till_m3 * km3;
                gtot.lake_km3 += g.lake_m3 * km3;
                gtot.sink_km3 += g.sink_m3 * km3;
                gtot.sea_km3 += g.sea_m3 * km3;
                ice_cells = ice_cells.max(g.ice.cells as u64);
                sc.glacial = g;
                self.time("glacial", started);
            }

            // Carbonates and evaporites.
            let started = Instant::now();
            let evap_rate = p.evaporite_m_myr * 1.0e-6;
            // Cell by cell, so in parallel (P1-17).
            let (kind, filled, clastic) = (&sc.kind, &sc.filled, &parts.marine);
            sc.h.par_iter_mut()
                .zip(parts.carbonate.par_iter_mut())
                .zip(parts.evaporite.par_iter_mut())
                .enumerate()
                .for_each(|(c, ((h, carb), evaporite))| {
                    if kind[c] == BASE {
                        let z = sea - *h;
                        let add = carbonate::produce(
                            z,
                            f64::from(clim.sst_c[c]),
                            f64::from(clastic[c]),
                            dts,
                            &carb_p,
                        );
                        *h += add;
                        *carb += add as f32;
                        // The climate's drawdown carbonate (after a snowball: its cap carbonate)
                        // on the sea floor above the compensation depth, whatever the water's
                        // temperature, once per plate step.
                        let z = sea - *h;
                        if sub == 0 && cap_m > 0.0 && z > 0.0 && z <= CARBONATE_FLOOR_M {
                            let extra = cap_m.min(z);
                            *h += extra;
                            *carb += extra as f32;
                        }
                        let z = sea - *h;
                        if restricted[c]
                            && z > 0.0
                            && z < 200.0
                            && f64::from(clim.aridity[c]) < facies::limits::EVAPORITE_ARIDITY
                        {
                            let e = (evap_rate * dts).min(z);
                            *h += e;
                            *evaporite += e as f32;
                        }
                    } else if f64::from(clim.aridity[c]) < facies::limits::EVAPORITE_ARIDITY {
                        let room = filled[c] - *h;
                        if room > spl::LAKE_TOL_M {
                            let e = (evap_rate * dts).min(room);
                            *h += e;
                            *evaporite += e as f32;
                        }
                    }
                });
            self.time("carbonate", started);

            // Hillslope and marine diffusion.
            // Every loop here is cell by cell with no carried sum, so running them in parallel
            // gives the serial loops' values (P1-17; the tanh and exp of `submarine_kd` alone
            // were ~50 ms a step on the full preset's L1).
            let started = Instant::now();
            {
                let (diffu, h) = (&self.diffu, &sc.h);
                sc.kd.par_iter_mut().enumerate().for_each(|(c, kd)| {
                    let land = p.kd_land_m2_yr * diffu[usize::from(class0[c])];
                    *kd = marine::submarine_kd(
                        sea - h[c],
                        p.kd_marine_m2_yr,
                        p.wave_base_m,
                        p.tidal_range_m,
                        land,
                    );
                });
            }
            let before = crate::alloc::copied(&sc.h);
            let solve = Instant::now();
            cg_iterations += diffusion::step(&d, &mut sc.h, &sc.kd, dts) as u64;
            // The solve's own share of "diffusion" (P1-17's breakdown).
            self.time("diffusion_solve", solve);
            let failing = Instant::now();
            if let Some(fp) = &failure {
                let sea_mask: Vec<bool> = sc.kind.par_iter().map(|&k| k == BASE).collect();
                let h = &sc.h;
                let mut avail: Vec<f64> = (0..n)
                    .into_par_iter()
                    .map(|c| (avail0[c] + h[c] - h_step0[c]).max(0.0))
                    .collect();
                failed_m_cells +=
                    marine::slope_failure(&d, &sea_mask, sea, fp, &mut sc.h, &mut avail);
            }
            self.time("slope_failure", failing);
            {
                let (h, kind) = (&sc.h, &sc.kind);
                parts
                    .eroded
                    .par_iter_mut()
                    .zip(parts.diffusion_eroded.par_iter_mut())
                    .zip(parts.marine.par_iter_mut())
                    .zip(parts.fluvial.par_iter_mut())
                    .enumerate()
                    .for_each(|(c, (((eroded, diffusion_eroded), marine), fluvial))| {
                        let dd = (h[c] - before[c]) as f32;
                        if dd < 0.0 {
                            *eroded -= dd;
                            *diffusion_eroded -= dd;
                        } else if kind[c] == BASE {
                            *marine += dd;
                        } else {
                            *fluvial += dd;
                        }
                    });
            }
            self.time("diffusion", started);
        }

        // 4. Strata log, regolith and weathering.
        let started = Instant::now();
        let t_mid = (t_end_myr - 0.5 * dt_myr) as f32;
        // The coal facies needs land plants (the atmosphere clock, `deposits::clocks`).
        let land_plants = self.epoch().land_plants_at(f64::from(t_mid) * 1.0e6); // ledger: ORE-018
        let slope: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|c| {
                let r = sc.routing.rcv[c] as usize;
                if r == c || sc.kind[c] == BASE {
                    0.0
                } else {
                    ((sc.filled[c] - sc.filled[r]) / f64::from(sc.routing.len[c]).max(1.0)) as f32
                }
            })
            .collect();
        let net: Vec<f64> = (0..n)
            .into_par_iter()
            .map(|c| sc.h[c] - h_start[c])
            .collect();
        st.totals.by_band.add(
            &d,
            &erosion_bands::StepErosion {
                h0: &h_start,
                sea_m: sea,
                spl: &parts.spl_eroded,
                threshold: &parts.threshold_cut,
                glacial: &parts.glacial_eroded,
                diffusion: &parts.diffusion_eroded,
                net: &net,
                dt_myr,
            },
        );
        let mut kind_step = crate::alloc::filled(n, fields::KIND_NONE);
        let mut coal = crate::alloc::filled(n, false);
        let w0_ref = p.weathering_max_m_myr * 1.0e-6;
        let mut weathered = 0.0f64;
        {
            let k = st.log.k as usize;
            let log = &mut st.log;
            let rows: Vec<(u8, bool, f64)> = log
                .layers
                .par_chunks_mut(k)
                .zip(log.count.par_iter_mut())
                .zip(st.regolith_m.par_iter_mut())
                .zip(st.regolith_kind.par_iter_mut())
                .enumerate()
                .map(|(c, (((layers, count), reg), rkind))| {
                    let mut one = ColumnMut { layers, count };
                    let mut kind_c = fields::KIND_NONE;
                    let mut coal_c = false;
                    let z_end = sc.h[c];
                    let marine = sc.kind[c] == BASE && z_end <= sea;
                    let dm = net[c];
                    let setting = Setting {
                        marine,
                        depth_m: (sea - z_end).max(0.0),
                        slope: f64::from(slope[c]),
                        t_c: f64::from(clim.t_c[c]),
                        aridity: f64::from(clim.aridity[c]),
                        restricted: restricted[c],
                        fluvial: f64::from(parts.fluvial[c]),
                        lake: f64::from(parts.lake[c]),
                        marine_clastic: f64::from(parts.marine[c]),
                        carbonate: f64::from(parts.carbonate[c]),
                        evaporite: f64::from(parts.evaporite[c]),
                        glacial: f64::from(parts.till[c]),
                        rate_m_myr: dm / dt_myr,
                        land_plants,
                    };
                    if dm > 0.0 {
                        if *reg >= 0.5 {
                            one.deposit(t_mid, *reg, regolith_class(&cls, *rkind), IN_SITU);
                            *reg = 0.0;
                            *rkind = weathering::SOIL;
                        }
                        let (class, kind) = facies::classify(&setting, &cls);
                        let prov = match kind {
                            _ if class == cls.till => IN_SITU,
                            Kind::Clastic | Kind::Euxinic => {
                                if f64::from(parts.marine[c])
                                    >= f64::from(parts.fluvial[c] + parts.lake[c])
                                {
                                    parts.prov_marine[c]
                                } else {
                                    sc.mix.get(c).and_then(dominant).unwrap_or(IN_SITU)
                                }
                            }
                            _ => IN_SITU,
                        };
                        one.deposit(t_mid, dm as f32, class, prov);
                        kind_c = match kind {
                            Kind::Carbonate => fields::KIND_CARBONATE,
                            Kind::Evaporite => fields::KIND_EVAPORITE,
                            Kind::Euxinic => fields::KIND_EUXINIC,
                            _ => fields::KIND_NONE,
                        };
                        coal_c = facies::peat_mire(&setting);
                    } else if dm < 0.0 {
                        let mut left = -dm;
                        let take = f64::from(*reg).min(left);
                        *reg -= take as f32;
                        left -= take;
                        one.erode(left);
                    }
                    let mut produced = 0.0;
                    if !marine {
                        let w0 = w0_ref
                            * weathering::climate_factor(
                                f64::from(clim.precip_mm[c]),
                                f64::from(clim.t_c[c]),
                            );
                        let h_old = f64::from(*reg);
                        let h_new =
                            weathering::integrate(h_old, w0, p.weathering_decay_depth_m, dt);
                        produced = h_new - h_old;
                        one.erode(produced);
                        *reg = h_new as f32;
                        let parent = one.top().map_or(basement[c], |l| l.class);
                        let felsic = self.fam[usize::from(parent)] == 0;
                        *rkind = weathering::kind(
                            h_new,
                            f64::from(clim.t_c[c]),
                            f64::from(clim.precip_mm[c]),
                            f64::from(slope[c]),
                            felsic,
                        );
                    }
                    (kind_c, coal_c, produced)
                })
                .collect();
            for (c, (kc, cc, wp)) in rows.into_iter().enumerate() {
                kind_step[c] = kc;
                coal[c] = cc && net[c] > 0.0;
                weathered += wp;
            }
        }
        self.time("strata", started);

        // 5. Isostasy: local, for the share the flexure solve (5c) does not take.
        let started = Instant::now();
        let flex_on = p.flexure.active();
        let local_share = 1.0 - p.flexure.strength;
        let h_eroded = flex_on.then(|| crate::alloc::copied(&sc.h));
        let mut iso = 0.0;
        if local_share > 0.0 {
            let mut load: Vec<f64> = (0..n)
                .into_par_iter()
                .map(|c| {
                    let r = if sc.h[c] <= sea { 0.7 } else { 1.0 };
                    net[c] * r * p.isostasy * local_share
                })
                .collect();
            let radius = (p.isostasy_radius_km * 1000.0 / dx).round() as usize;
            box_blur(&d, &mut load, radius, 3);
            for (z, l) in sc.h.iter_mut().zip(&load) {
                *z -= l;
                iso -= l;
            }
        }
        let h_before_spread = flex_on.then(|| crate::alloc::copied(&sc.h));
        self.time("isostasy", started);

        // 5b. Crust over its strength spreads (`spread.rs`): it moves crust, not surface rock, so
        // it comes after the step's net change was logged; the plate model learns of it through
        // the feedback.
        let started = Instant::now();
        let flow = spread::step(
            &d,
            &mut sc.h,
            sea,
            self.preset.tectonics.strength_height_m(gravity),
            self.preset.tectonics.crust_flow_km2_myr,
            dt,
        );
        if flow.cells > 0 {
            *world
                .globals
                .entry(spread::CELLS_GLOBAL.to_owned())
                .or_insert(0.0) += flow.cells as f64;
            *world
                .globals
                .entry(spread::MOVED_GLOBAL.to_owned())
                .or_insert(0.0) += flow.moved_m * a / 1.0e9;
        }
        self.time("spread", started);

        // 5c. Flexure (`flexure.rs`): the plate bends under the step's loads.
        let mut flexed = None;
        // The land's gap budget (`plates::gap_budget`, measurement only): the surface's parts of
        // the feedback, per cell.
        let budget = world
            .plates
            .as_ref()
            .is_some_and(|st| st.gap_budget.is_some());
        let h_spread = (budget && flex_on).then(|| crate::alloc::copied(&sc.h));
        let mut answers: Option<Vec<(f64, f64)>> = None;
        if let (Some(h_eroded), Some(h_before_spread)) = (&h_eroded, &h_before_spread) {
            let started = Instant::now();
            let ice: Option<Vec<f32>> = world
                .fields
                .get(crate::climate::field::ICE_THICKNESS)
                .filter(|f| f.data.len() == n)
                .map(|f| f.data.clone());
            let x_in = flexure::StepIn {
                dims: &d,
                norm_dx_m: std::f64::consts::TAU * self.preset.tectonics.model_radius_km * 1000.0
                    / w as f64,
                // The plate model's mean sample spacing on its sphere.
                plate_spacing_m: (4.0
                    * std::f64::consts::PI
                    * (self.preset.tectonics.model_radius_km * 1000.0).powi(2)
                    / f64::from(self.preset.resolution.l0_samples))
                .sqrt(),
                gravity_m_s2: gravity,
                sea_level_m: sea,
                dt_myr,
                h_start: &h_start,
                h_eroded,
                h_before_spread,
                uplift_mm_yr: &pf.uplift,
                crust_type: &pf.crust_type,
                crust_age_myr: &pf.crust_age,
                orogeny_age_myr: &pf.orogeny_age,
                regime: &pf.regime,
                plate_id: &pf.plate_id,
                ice_m: ice.as_deref(),
                rock_share: p.isostasy,
                parts: budget,
            };
            if budget {
                answers = Some(flexure::local_answers(
                    &p.flexure,
                    st.flexure.as_ref(),
                    &x_in,
                    &sc.h,
                ));
            }
            let out = flexure::step(
                &p.flexure,
                &mut self.flex_solver,
                &mut st.flexure,
                &x_in,
                &mut sc.h,
            )?;
            iso += out.sum_change_m;
            if let Some(rep) = out.solve {
                let mut count = |name: &str, by: f64| {
                    *world.globals.entry(name.to_owned()).or_insert(0.0) += by;
                };
                count(flexure::SOLVES_GLOBAL, 1.0);
                count(flexure::ITERATIONS_GLOBAL, f64::from(rep.iterations));
                if !rep.converged {
                    count(flexure::UNCONVERGED_GLOBAL, 1.0);
                }
            }
            flexed = Some(out);
            self.time("flexure", started);
        }
        if let (Some(h_eroded), Some(h_before_spread), Some(h_spread), Some(local)) =
            (&h_eroded, &h_before_spread, &h_spread, &answers)
        {
            use crate::plates::gap_budget::field as gap;
            // The flexure's change split by what it answers: the step's loads (their local
            // answers apart), the roots already in the elevation, the moment released, and the
            // rest (other stages' loads, the solves' unfinished parts). A step without a solve
            // moved nothing.
            let split = flexed.as_ref().filter(|o| {
                o.loads_m.len() == n && o.roots_m.len() == n && o.released_m.len() == n
            });
            let loads = |c: usize| split.map_or(0.0, |o| f64::from(o.loads_m[c]));
            let roots = |c: usize| split.map_or(0.0, |o| f64::from(o.roots_m[c]));
            let released = |c: usize| split.map_or(0.0, |o| f64::from(o.released_m[c]));
            let rebound = (0..n).map(|c| (h_before_spread[c] - h_eroded[c] + local[c].0) as f32);
            let water = (0..n).map(|c| local[c].1 as f32);
            let flex = (0..n).map(|c| (loads(c) - local[c].0 - local[c].1) as f32);
            let rest =
                (0..n).map(|c| (sc.h[c] - h_spread[c] - loads(c) - roots(c) - released(c)) as f32);
            let spread = (0..n).map(|c| (h_spread[c] - h_before_spread[c]) as f32);
            for (name, data) in [
                (gap::REBOUND, rebound.collect::<Vec<f32>>()),
                (gap::WATER, water.collect()),
                (gap::FLEXURE, flex.collect()),
                (gap::ROOTS, (0..n).map(|c| roots(c) as f32).collect()),
                (gap::RELEASED, (0..n).map(|c| released(c) as f32).collect()),
                (gap::FLEX_REST, rest.collect()),
                (gap::SPREAD, spread.collect()),
                // The volcano stage and the impacts add their changes of the surface to it.
                (gap::LATER, vec![0.0; n]),
            ] {
                world.fields.insert(
                    name.to_owned(),
                    Field {
                        grid: grid.clone(),
                        data,
                    },
                );
            }
        }

        // 6. Outputs.
        let started = Instant::now();
        let sum_net: f64 = net.iter().sum();
        let carb: f64 = parts.carbonate.iter().map(|&x| f64::from(x)).sum();
        let evap: f64 = parts.evaporite.iter().map(|&x| f64::from(x)).sum();
        let km3 = a / 1.0e9;
        let t = &mut st.totals;
        t.eroded_km3 += parts.eroded.iter().map(|&x| f64::from(x)).sum::<f64>() * km3;
        t.fluvial_km3 += parts.fluvial.iter().map(|&x| f64::from(x)).sum::<f64>() * km3;
        t.lake_km3 += parts.lake.iter().map(|&x| f64::from(x)).sum::<f64>() * km3;
        t.marine_km3 += parts.marine.iter().map(|&x| f64::from(x)).sum::<f64>() * km3;
        t.carbonate_km3 += carb * km3;
        t.evaporite_km3 += evap * km3;
        t.to_sea_km3 += to_sea / 1.0e9;
        t.weathered_km3 += weathered * km3;
        t.isostasy_km3 += iso * km3;
        t.slope_failure_km3 += failed_m_cells * km3;
        t.mass_residual_km3 += (sum_net - carb - evap) * km3;
        t.steps += 1;
        t.spl_iterations += iterations;
        t.diffusion_iterations += cg_iterations;
        t.sinks += sinks;

        let m_per_block_v = self.preset.scale.m_per_block_v;
        let sea_y = f64::from(self.preset.scale.sea_level_y);
        let erosion: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|c| ((-net[c]).max(0.0) / dt_myr) as f32)
            .collect();
        let deposition: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|c| (net[c].max(0.0) / dt_myr) as f32)
            .collect();
        let class_now: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|c| f32::from(surface_class(&st, c)))
            .collect();
        let sediment: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|c| st.log.total(c) as f32)
            .collect();
        let drainage: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|c| {
                if sc.kind[c] == BASE {
                    f32::NAN
                } else {
                    (sc.area[c] / 1.0e6).max(1e-3).log10() as f32
                }
            })
            .collect();
        let elevation_y: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|c| (sea_y + (sc.h[c] - sea) / m_per_block_v) as f32)
            .collect();
        // What the plate model is told: where the surface stands over the plate model's own
        // elevation, all of it, not only this step's change. Each sample takes the mean over its
        // cells, so its elevation is the surface's at its scale and the uplift laws that look at
        // it (the ceiling on collision and subduction uplift, the cap) see the real height. With
        // only the step's change handed over, whatever a sample had not taken (the spread onto
        // coarser samples, its clamps) stayed in the surface for good and the two drifted apart:
        // crust the plate model held at 2.8 km stood at 14 km and still took a full collision
        // surge.
        let feedback: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|c| (sc.h[c] - f64::from(pf.elevation[c])) as f32)
            .collect();
        let material: Vec<f32> = net.par_iter().map(|&x| x as f32).collect();
        st.offset_m = (0..n)
            .into_par_iter()
            .map(|c| (sc.h[c] - f64::from(pf.elevation[c])) as f32)
            .collect();
        st.elevation_m = sc.h.par_iter().map(|&z| z as f32).collect();
        let put = |world: &mut WorldState, name: &str, data: Vec<f32>| {
            world.fields.insert(
                name.to_owned(),
                Field {
                    grid: grid.clone(),
                    data,
                },
            );
        };
        put(world, "elevation", elevation_y);
        put(
            world,
            field::ELEVATION_M,
            crate::alloc::copied(&st.elevation_m),
        );
        put(world, field::EROSION, crate::alloc::copied(&erosion));
        put(world, field::DEPOSITION, crate::alloc::copied(&deposition));
        put(world, field::SEDIMENT, sediment);
        put(world, field::CLASS, class_now);
        put(world, field::DRAINAGE, drainage);
        put(world, field::REGOLITH, crate::alloc::copied(&st.regolith_m));
        put(world, field::FEEDBACK, feedback);
        put(world, field::MATERIAL, material);
        let ice_now: Vec<f32> = if ela_z.is_some() && sc.glacial.ice.footprint.len() == n {
            // Over the glaciers' whole width, on the ground as it now is.
            (0..n)
                .into_par_iter()
                .map(|c| {
                    let s = sc.glacial.ice.footprint[c];
                    if s.is_finite() {
                        (s - sc.h[c]).max(0.0) as f32
                    } else {
                        0.0
                    }
                })
                .collect()
        } else {
            crate::alloc::filled(n, 0.0)
        };
        let g_rate: Vec<f32> = parts
            .glacial_eroded
            .par_iter()
            .map(|&e| (f64::from(e) / dt_myr) as f32)
            .collect();
        if ela_z.is_some() {
            if ice_cells > 0 {
                gtot.steps_with_ice += 1;
            }
            gtot.ice_cells_max = gtot.ice_cells_max.max(ice_cells);
            gtot.ice_cells_last = ice_cells;
            let fastest = g_rate.iter().copied().fold(0.0f32, f32::max);
            gtot.max_rate_m_myr = gtot.max_rate_m_myr.max(f64::from(fastest));
            gtot.write(&mut world.notes);
        }
        put(world, field::ICE, ice_now);
        put(
            world,
            field::CRUST_PLATE,
            crate::alloc::copied(&pf.plate_id),
        );
        match &pf.group_id {
            Some(g) => put(world, field::CRUST_GROUP, crate::alloc::copied(g)),
            None => {
                world.fields.remove(field::CRUST_GROUP);
            }
        }
        put(world, field::GLACIAL_EROSION, g_rate);
        match flexed {
            Some(out) => {
                put(world, flexure::field::FLEXURE, out.flexure_m);
                put(world, flexure::field::DEFLECTION, out.deflection_m);
                put(
                    world,
                    flexure::field::ELASTIC_THICKNESS,
                    out.elastic_thickness_km,
                );
            }
            None => {
                for name in [
                    flexure::field::FLEXURE,
                    flexure::field::DEFLECTION,
                    flexure::field::ELASTIC_THICKNESS,
                ] {
                    world.fields.remove(name);
                }
            }
        }
        *world
            .globals
            .entry(FOREIGN_CRUST_GLOBAL.to_owned())
            .or_insert(0.0) += foreign_cells as f64;
        world.fields.remove(field::FEEDBACK_APPLIED);
        self.time("outputs", started);

        // The deposit engine's fields for this step.
        let started = Instant::now();
        let rel: Vec<f64> = sc.h.par_iter().map(|&z| z - sea).collect();
        let step_fields = fields::step_fields(
            &fields::StepInputs {
                flags: &pf.flags,
                crust_type: &pf.crust_type,
                orogeny_type: &pf.orogeny_type,
                orogeny_age: &pf.orogeny_age,
                regime: &pf.regime,
                uplift_mm_yr: &pf.uplift,
                oxidation: &pf.oxidation,
                convergence: &pf.convergence,
                obliquity: &pf.obliquity,
                slab_age: &pf.slab_age,
                heat_flow: &pf.heat_flow,
                thickness_m: &pf.thickness,
                rel_elev: &rel,
                erosion_m_myr: &erosion,
                deposition_m_myr: &deposition,
                kind: &kind_step,
                coal: &coal,
                slope: &slope,
                t_c: &clim.t_c,
                precip_mm: &clim.precip_mm,
                aridity: &clim.aridity,
                log: &st.log,
                regolith_m: &st.regolith_m,
                dx,
            },
            &cls,
        );
        for (name, v) in fields::STEP_FIELDS.iter().zip(&step_fields) {
            st.acc.add(name, v, dt);
        }
        // Inherited iron-formation basins (`deposits::bif_basins`; Ben, 2026-10-02): dealt once,
        // over the stable continental interiors of the first step, then carried with the crust;
        // the frames see them where the crust is a stable interior.
        let epoch = self.epoch();
        if st.bif_basin.len() != n && epoch.inherits_bif() {
            let dp = self.preset.deposits.clone().unwrap_or_default();
            let eligible: Vec<bool> = (0..n)
                .map(|c| {
                    pf.crust_type[c] as u8 == crate::plates::state::CONTINENTAL
                        && pf.regime[c] as u8 == crate::plates::state::regime::CRATON
                })
                .collect();
            let b = crate::deposits::bif_basins::deal(
                &d,
                &eligible,
                dp.inherited_bif_share,
                dp.bif_basin_km2,
                self.preset.seed,
            );
            world
                .globals
                .insert(BIF_BASINS_GLOBAL.to_owned(), b.count as f64);
            world
                .globals
                .insert(BIF_BASIN_SHARE_GLOBAL.to_owned(), b.share);
            st.bif_basin = b.mask;
        }
        if st.bif_basin.len() == n {
            let v: Vec<f32> = st
                .bif_basin
                .par_iter()
                .zip(pf.regime.par_iter())
                .map(|(&b, &r)| {
                    if r as u8 == crate::plates::state::regime::CRATON {
                        b
                    } else {
                        0.0
                    }
                })
                .collect();
            st.acc.add(fields::BIF_BASIN_STEP, &v, dt);
        }
        // The climate's ice as frame means (FORMAT.md "The climate's ice fields").
        for (name, from) in [
            (
                crate::climate::history::ICE_THICKNESS,
                crate::climate::field::ICE_THICKNESS,
            ),
            (
                crate::climate::history::ICE_COVER,
                crate::climate::field::ICE_TIME_SHARE,
            ),
        ] {
            if let Some(f) = world.fields.get(from).filter(|f| f.data.len() == n) {
                st.acc.add(name, &f.data, dt);
            }
        }
        st.acc.span_years += dt;
        let last = self.done + 1 >= self.total;
        let t_end_years = t_end_myr * 1.0e6;
        if last || t_end_years - st.acc.t_start_years >= p.frame_every_myr * 1.0e6 - 1.0 {
            let t_start = st.acc.t_start_years;
            let span_myr = st.acc.span_years / 1.0e6;
            let get = |acc: &fields::FrameAcc, name: &str| -> Vec<f32> {
                acc.sums
                    .get(name)
                    .map(|v| {
                        v.iter()
                            .map(|&s| (f64::from(s) / acc.span_years.max(1e-9)) as f32)
                            .collect()
                    })
                    .unwrap_or_else(|| crate::alloc::filled(n, 0.0))
            };
            let ero = get(&st.acc, "deeptime:erosion_rate_m_myr");
            let dep = get(&st.acc, "deeptime:deposition_rate_m_myr");
            let elev = get(&st.acc, "deeptime:elevation_m");
            let frame = fields::flush(&mut st.acc, &d, t_end_years, &self.epoch());
            let history = world
                .history
                .get_or_insert_with(|| RecordedHistory::new(grid.clone()));
            history.push(&frame);
            st.totals.frames += 1;
            if last {
                history.receivers = Some(sc.routing.rcv.clone());
            }
            let view = events::FrameView {
                d: &d,
                grid: &grid,
                m_per_block: self.preset.scale.m_per_block_h,
                r_km: self.preset.scale.radius_m() / 1000.0,
                t_start,
                t_end: t_end_years,
                span_myr,
                erosion: &ero,
                deposition: &dep,
                elevation: &elev,
                regime: &pf.regime,
                crust_type: &pf.crust_type,
                sea_level_m: sea,
                erosion_pulse_m_myr: p.erosion_pulse_m_myr,
                basin_m_myr: p.basin_m_myr,
                min_km2: p.event_min_area_km2,
                transgression_frac: p.transgression_frac,
            };
            let mut ev = std::mem::take(&mut st.events);
            events::frame(world, &mut ev, &view, last)?;
            st.events = ev;
        }
        self.time("fields_events", started);

        st.omega = omega_now;
        st.step = pstep;
        st.t_myr = t_end_myr;
        st.sea_level_m = sea;
        if last {
            world.notes.insert(
                STAGE_NAME.into(),
                summary(&st, &self.timings, world).to_string(),
            );
        }
        world.surface = Some(st);
        self.scratch = Some(sc);
        Ok(())
    }
}

/// The run summary (`manifest.json` `notes.surface`, and the CLI's `surface_final` line).
pub fn summary(
    st: &SurfaceState,
    timings: &BTreeMap<&'static str, f64>,
    world: &WorldState,
) -> serde_json::Value {
    let n = st.log.cells().max(1);
    let layers: u64 = st.log.count.iter().map(|&c| u64::from(c)).sum();
    let sediment: f64 = (0..st.log.cells()).map(|c| st.log.total(c)).sum();
    let dx = f64::from(st.grid.r) * 300.0;
    let land = st
        .elevation_m
        .iter()
        .filter(|&&z| f64::from(z) > st.sea_level_m)
        .count();
    let mut peak = 0.0f32;
    for &z in &st.elevation_m {
        peak = peak.max(z);
    }
    let history = world.history.as_ref();
    json!({
        "steps": st.totals.steps,
        "totals": st.totals,
        "land_erosion_by_band_m_myr": st.totals.by_band.json(),
        "land_frac": land as f64 / n as f64,
        "max_elevation_m": peak,
        "sediment_km3": sediment * dx * dx / 1.0e9,
        "mean_log_layers": layers as f64 / n as f64,
        "sea_level_m": st.sea_level_m,
        "events": {
            "erosion_pulse": st.events.counts[0],
            "basin_subsidence": st.events.counts[1],
            "transgression_regression": st.events.counts[2],
        },
        "history_frames": history.map_or(0, |h| h.frame_count()),
        "history_bytes": history.map_or(0, |h| h.bytes()),
        "millis": timings,
    })
}

impl Stage for SurfaceStage {
    fn name(&self) -> &str {
        STAGE_NAME
    }

    fn estimate(&self) -> StageEstimate {
        estimate(&self.preset)
    }

    fn total_steps(&self) -> u64 {
        self.total
    }

    fn steps_done(&self) -> u64 {
        self.done
    }

    fn step(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        if self.done >= self.total {
            return Ok(());
        }
        ctx.pool.install(|| self.advance(world, ctx))?;
        self.done += 1;
        Ok(())
    }

    fn keyframe_layers(&self) -> &[&'static str] {
        &KEYFRAME_LAYERS
    }

    fn coupled_to(&self) -> Option<&'static str> {
        Some(crate::plates::STAGE_NAME)
    }

    fn save_state(&self) -> Result<Vec<u8>, CoreError> {
        to_state(&Saved { done: self.done })
    }

    fn load_state(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        let s: Saved = from_state(bytes)?;
        self.done = s.done;
        self.scratch = None;
        self.flex_solver = None;
        Ok(())
    }
}

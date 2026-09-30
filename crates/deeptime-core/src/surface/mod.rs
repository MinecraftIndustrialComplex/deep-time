// SPDX-License-Identifier: LGPL-3.0-or-later
//! Deep-time surface processes on L1 (plan §8.2, work item P1-04): erosion, deposition,
//! diffusion, marine sedimentation, carbonate production, weathering and the strata log, coupled
//! to the plate model step by step.
//!
//! # Coupling with the plates (plan §8 intro)
//! The stage is interleaved with the plates stage (`Stage::coupled_to`): after every plate step it
//! takes one step of the same length. L1 is fixed on the planet and the crust moves over it, so the
//! surface keeps its elevation as the plate model's (`plates/elevation_m`, sampled onto L1 every
//! step; the pipeline presets set `tectonics.l1_every_steps` = 1) plus an **offset** — valleys,
//! basins, eroded ranges, everything the plate model's coarse L0 elevation does not carry — which
//! moves with the plates (`advect.rs`) together with the regolith and the strata log. Uplift is
//! therefore whatever the plate model did to the crust under a cell (subduction and collision
//! uplift, thermal subsidence, trench accretion), exactly P1-03's `plates/uplift_mm_yr`.
//!
//! The surface's changes flow back: the net elevation change of every cell (`surface/feedback_m`,
//! and the material moved, `surface/material_m`) is spread onto the L0 samples at the start of the
//! next plate step (`plates::PlatesStage`), which reports what it applied as seen from L1
//! (`surface/feedback_applied_m`); the offset is corrected by it, so the two elevations never
//! drift apart. The plate model's own continental erosion (Cortial's ε_c) is off in the pipeline
//! presets — this stage does erosion.
//!
//! # One step (Δt = the plate step, `substeps` LEM steps inside it)
//! 1. Advect the offset, regolith and strata log; clear them on crust born this step.
//! 2. h = plate elevation + offset; sea level from the preset (`params::SeaLevel`), or, with a
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
pub mod events;
pub mod facies;
pub mod fields;
pub mod grid;
pub mod lithology;
pub mod marine;
pub mod oracle;
pub mod params;
pub mod pflood;
pub mod routing;
pub mod spl;
pub mod strata_log;
pub mod thermal;
pub mod weathering;
pub mod young;

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
}

/// Keyframe layers (the group adds the plate model's `elevation`, `plates`, `crust_age`,
/// `regime`; `elevation` shows the surface's).
pub const KEYFRAME_LAYERS: [&str; 6] = [
    field::EROSION,
    field::DEPOSITION,
    field::SEDIMENT,
    field::CLASS,
    field::DRAINAGE,
    field::REGOLITH,
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
    /// Σ(Δh)·a − (carbonate + evaporite): what the processes created or lost, km³.
    pub mass_residual_km3: f64,
    pub steps: u64,
    pub spl_iterations: u64,
    /// Conjugate-gradient iterations of the diffusion solves.
    pub diffusion_iterations: u64,
    pub sinks: u64,
    pub frames: u64,
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
    kf: Vec<f64>,
    kd: Vec<f64>,
    weight: Vec<f32>,
    mix: Vec<[f64; spl::FAMILIES]>,
    fill: marine::FillScratch,
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
            kf: vec![0.0; n],
            kd: vec![0.0; n],
            weight: vec![1.0; n],
            mix: Vec::new(),
            fill: marine::FillScratch::default(),
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
    prov_marine: Vec<u16>,
}

impl Parts {
    fn new(n: usize) -> Parts {
        Parts {
            eroded: vec![0.0; n],
            fluvial: vec![0.0; n],
            lake: vec![0.0; n],
            marine: vec![0.0; n],
            carbonate: vec![0.0; n],
            evaporite: vec![0.0; n],
            prov_marine: vec![IN_SITU; n],
        }
    }
}

/// The plate fields the stage reads each step.
struct PlateFields {
    elevation: Vec<f32>,
    uplift: Vec<f32>,
    plate_id: Vec<f32>,
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
    let per_cell = 12 * 8 // f64 working arrays
        + 2 * log // the log and its advected copy
        + 30 * 4 // fields
        + (fields::STEP_FIELDS.len() as u64) * 4 // the frame accumulator
        + 24; // mix, weights, kinds
    let frames = (p.length_myr() / s.frame_every_myr).ceil().max(1.0) as u64;
    let history = frames * cells * 12; // packed frames, ~12 B/cell after LZ4 (measured in P1-04)
    let steps = p.plate_steps() as f64 * f64::from(s.substeps);
    StageEstimate {
        bytes: cells * per_cell + history + crate::plates::estimate(p).bytes,
        seconds: cells as f64 * steps * 1.0e-6,
        history_bytes: history,
    }
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
        let dt_myr = self.preset.length_myr() / self.total.max(1) as f64;
        let dt = dt_myr * 1.0e6;
        let omega_now: Vec<[f64; 3]> = plates
            .plates
            .iter()
            .map(|p| if p.alive { p.omega } else { [0.0; 3] })
            .collect();
        let parents: Vec<Option<u32>> = plates.plates.iter().map(|p| p.parent).collect();
        let pstep = plates.step;
        let pf = PlateFields::read(world, n)?;
        let applied = world
            .fields
            .get(field::FEEDBACK_APPLIED)
            .filter(|f| f.data.len() == n)
            .map(|f| f.data.clone());
        let cls = self.classes;
        let p = self.p.clone();

        // 1. The state, carried with the plates.
        let started = Instant::now();
        let mut st = match world.surface.take() {
            Some(mut st) => {
                ctx.reserve(StrataLog::bytes_per_cell(st.log.k) * n as u64 + 16 * n as u64)?;
                let mut off = st.offset_m.clone();
                if let Some(ap) = &applied {
                    off.par_iter_mut()
                        .zip(ap.par_iter())
                        .for_each(|(o, a)| *o -= a);
                }
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
                st.offset_m = src
                    .par_iter()
                    .map(|&[fi, fj]| advect::bilinear(w, hgt, &off, fi, fj))
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
                let k = st.log.k as usize;
                let old = std::mem::take(&mut st.log);
                let mut log = StrataLog::new(n, old.k);
                log.layers
                    .par_chunks_mut(k)
                    .zip(log.count.par_iter_mut())
                    .zip(nearest.par_iter())
                    .for_each(|((col, cnt), &s)| {
                        let m = usize::from(old.count[s]);
                        col[..m].copy_from_slice(&old.layers[s * k..s * k + m]);
                        *cnt = old.count[s];
                    });
                st.log = log;
                // Crust born this step starts bare.
                let mut newborn = vec![false; n];
                for (c, born) in newborn.iter_mut().enumerate() {
                    let age = pf.crust_age[c];
                    if age >= 0.0 && age < dt_myr as f32 {
                        st.offset_m[c] = 0.0;
                        st.regolith_m[c] = 0.0;
                        st.regolith_kind[c] = weathering::SOIL;
                        st.log.clear(c);
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
        let h_start = sc.h.clone();
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
        let mut open: Vec<f64> =
            sc.h.iter()
                .map(|&z| if z <= sea { 1.0 } else { 0.0 })
                .collect();
        box_blur(&d, &mut open, ((150_000.0 / dx).round() as usize).max(1), 2);
        let restricted: Vec<bool> = (0..n).map(|c| sc.h[c] <= sea && open[c] < 0.4).collect();
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
        let fam: Vec<u8> = class0.iter().map(|&k| self.fam[usize::from(k)]).collect();
        #[allow(clippy::needless_range_loop)]
        for c in 0..n {
            let k = usize::from(class0[c]);
            sc.kf[c] = p.k_spl * self.erod[k];
            sc.weight[c] = (clim.runoff_m_yr[c] / p.runoff_ref_m_yr as f32).max(0.05);
        }
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
            let started = Instant::now();
            let input = spl::SplInput {
                kind: &sc.kind,
                h0: &sc.h,
                filled: &sc.filled,
                base_z: &sc.base_z,
                area: &sc.area,
                kf: &sc.kf,
            };
            let rep = spl::solve(&d, &sc.routing, &input, &spl_p, &mut sc.h_new, &mut sc.qin);
            iterations += rep.iterations as u64;
            if sub + 1 == subs {
                spl::families(&d, &sc.routing, &input, &fam, dts, &sc.h_new, &mut sc.mix);
            }
            for c in 0..n {
                if sc.kind[c] == BASE {
                    continue;
                }
                let dh = sc.h_new[c] - sc.h[c];
                if dh < 0.0 {
                    parts.eroded[c] += (-dh) as f32;
                } else if sc.filled[c] - sc.h[c] > spl::LAKE_TOL_M {
                    parts.lake[c] += dh as f32;
                } else {
                    parts.fluvial[c] += dh as f32;
                }
            }
            std::mem::swap(&mut sc.h, &mut sc.h_new);
            self.time("spl", started);

            // River loads into the sea.
            let started = Instant::now();
            let sea_mask: Vec<bool> = sc.kind.iter().map(|&k| k == BASE).collect();
            for m in 0..n {
                if sc.kind[m] != BASE || sc.qin[m] <= 0.0 {
                    continue;
                }
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

            // Carbonates and evaporites.
            let started = Instant::now();
            let evap_rate = p.evaporite_m_myr * 1.0e-6;
            #[allow(clippy::needless_range_loop)]
            for c in 0..n {
                if sc.kind[c] == BASE {
                    let z = sea - sc.h[c];
                    let add = carbonate::produce(
                        z,
                        f64::from(clim.sst_c[c]),
                        f64::from(parts.marine[c]),
                        dts,
                        &carb_p,
                    );
                    sc.h[c] += add;
                    parts.carbonate[c] += add as f32;
                    // The climate's drawdown carbonate (after a snowball: its cap carbonate) on the
                    // sea floor above the compensation depth, whatever the water's temperature,
                    // once per plate step.
                    let z = sea - sc.h[c];
                    if sub == 0 && cap_m > 0.0 && z > 0.0 && z <= CARBONATE_FLOOR_M {
                        let extra = cap_m.min(z);
                        sc.h[c] += extra;
                        parts.carbonate[c] += extra as f32;
                    }
                    let z = sea - sc.h[c];
                    if restricted[c]
                        && z > 0.0
                        && z < 200.0
                        && f64::from(clim.aridity[c]) < facies::limits::EVAPORITE_ARIDITY
                    {
                        let e = (evap_rate * dts).min(z);
                        sc.h[c] += e;
                        parts.evaporite[c] += e as f32;
                    }
                } else if f64::from(clim.aridity[c]) < facies::limits::EVAPORITE_ARIDITY {
                    let room = sc.filled[c] - sc.h[c];
                    if room > spl::LAKE_TOL_M {
                        let e = (evap_rate * dts).min(room);
                        sc.h[c] += e;
                        parts.evaporite[c] += e as f32;
                    }
                }
            }
            self.time("carbonate", started);

            // Hillslope and marine diffusion.
            let started = Instant::now();
            #[allow(clippy::needless_range_loop)]
            for c in 0..n {
                let land = p.kd_land_m2_yr * self.diffu[usize::from(class0[c])];
                sc.kd[c] = marine::submarine_kd(
                    sea - sc.h[c],
                    p.kd_marine_m2_yr,
                    p.wave_base_m,
                    p.tidal_range_m,
                    land,
                );
            }
            let before = sc.h.clone();
            cg_iterations += diffusion::step(&d, &mut sc.h, &sc.kd, dts) as u64;
            #[allow(clippy::needless_range_loop)]
            for c in 0..n {
                let dd = (sc.h[c] - before[c]) as f32;
                if dd < 0.0 {
                    parts.eroded[c] -= dd;
                } else if sc.kind[c] == BASE {
                    parts.marine[c] += dd;
                } else {
                    parts.fluvial[c] += dd;
                }
            }
            self.time("diffusion", started);
        }

        // 4. Strata log, regolith and weathering.
        let started = Instant::now();
        let t_mid = (t_end_myr - 0.5 * dt_myr) as f32;
        let slope: Vec<f32> = (0..n)
            .map(|c| {
                let r = sc.routing.rcv[c] as usize;
                if r == c || sc.kind[c] == BASE {
                    0.0
                } else {
                    ((sc.filled[c] - sc.filled[r]) / f64::from(sc.routing.len[c]).max(1.0)) as f32
                }
            })
            .collect();
        let net: Vec<f64> = (0..n).map(|c| sc.h[c] - h_start[c]).collect();
        let mut kind_step = vec![fields::KIND_NONE; n];
        let mut coal = vec![false; n];
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
                        rate_m_myr: dm / dt_myr,
                        land_plants: true,
                    };
                    if dm > 0.0 {
                        if *reg >= 0.5 {
                            one.deposit(t_mid, *reg, regolith_class(&cls, *rkind), IN_SITU);
                            *reg = 0.0;
                            *rkind = weathering::SOIL;
                        }
                        let (class, kind) = facies::classify(&setting, &cls);
                        let prov = match kind {
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

        // 5. Isostasy.
        let started = Instant::now();
        let mut load: Vec<f64> = (0..n)
            .map(|c| {
                let r = if sc.h[c] <= sea { 0.7 } else { 1.0 };
                net[c] * r * p.isostasy
            })
            .collect();
        let radius = (p.isostasy_radius_km * 1000.0 / dx).round() as usize;
        box_blur(&d, &mut load, radius, 3);
        let mut iso = 0.0;
        for (z, l) in sc.h.iter_mut().zip(&load) {
            *z -= l;
            iso -= l;
        }
        self.time("isostasy", started);

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
        t.mass_residual_km3 += (sum_net - carb - evap) * km3;
        t.steps += 1;
        t.spl_iterations += iterations;
        t.diffusion_iterations += cg_iterations;
        t.sinks += sinks;

        let m_per_block_v = self.preset.scale.m_per_block_v;
        let sea_y = f64::from(self.preset.scale.sea_level_y);
        let erosion: Vec<f32> = (0..n)
            .map(|c| ((-net[c]).max(0.0) / dt_myr) as f32)
            .collect();
        let deposition: Vec<f32> = (0..n).map(|c| (net[c].max(0.0) / dt_myr) as f32).collect();
        let class_now: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|c| f32::from(surface_class(&st, c)))
            .collect();
        let sediment: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|c| st.log.total(c) as f32)
            .collect();
        let drainage: Vec<f32> = (0..n)
            .map(|c| {
                if sc.kind[c] == BASE {
                    f32::NAN
                } else {
                    (sc.area[c] / 1.0e6).max(1e-3).log10() as f32
                }
            })
            .collect();
        let elevation_y: Vec<f32> = (0..n)
            .map(|c| (sea_y + (sc.h[c] - sea) / m_per_block_v) as f32)
            .collect();
        let feedback: Vec<f32> = (0..n).map(|c| (sc.h[c] - h_start[c]) as f32).collect();
        let material: Vec<f32> = net.iter().map(|&x| x as f32).collect();
        st.offset_m = (0..n)
            .map(|c| (sc.h[c] - f64::from(pf.elevation[c])) as f32)
            .collect();
        st.elevation_m = sc.h.iter().map(|&z| z as f32).collect();
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
        put(world, field::ELEVATION_M, st.elevation_m.clone());
        put(world, field::EROSION, erosion.clone());
        put(world, field::DEPOSITION, deposition.clone());
        put(world, field::SEDIMENT, sediment);
        put(world, field::CLASS, class_now);
        put(world, field::DRAINAGE, drainage);
        put(world, field::REGOLITH, st.regolith_m.clone());
        put(world, field::FEEDBACK, feedback);
        put(world, field::MATERIAL, material);
        world.fields.remove(field::FEEDBACK_APPLIED);
        self.time("outputs", started);

        // The deposit engine's fields for this step.
        let started = Instant::now();
        let rel: Vec<f64> = sc.h.iter().map(|&z| z - sea).collect();
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
                    .unwrap_or_else(|| vec![0.0; n])
            };
            let ero = get(&st.acc, "deeptime:erosion_rate_m_myr");
            let dep = get(&st.acc, "deeptime:deposition_rate_m_myr");
            let elev = get(&st.acc, "deeptime:elevation_m");
            let frame = fields::flush(&mut st.acc, &d, t_end_years);
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
        Ok(())
    }
}

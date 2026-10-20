// SPDX-License-Identifier: LGPL-3.0-or-later
//! 3D structure, Phase 1 (plan §8.5, work item P1-07): column strata from the strata log,
//! metamorphism by regime history, plutons as simple bodies under arcs, and the horizon bake.
//!
//! The stage is coupled to the plates like the surface (`Stage::coupled_to`): a planet preset with
//! a `surface` section runs `[plates, surface, structure]`, one step each per plate step, and the
//! structure step follows the surface's. Each step:
//! 1. **Metamorphism** (`metamorphism.rs`) of the surface's strata log in place — the log is the
//!    surface's state (`WorldState::surface`), so metamorphosed layers erode as what they became
//!    and move with the plates like every other layer.
//! 2. **Plutons** (`plutons.rs`): the existing ones ride their plates and are unroofed; arc and
//!    collision magmatism emplaces new ones.
//!    2b. **Folds and faults** (`deform/`, P2-01): the existing ones ride their plates; the step's
//!    shortening, rifting and oblique convergence start new ones.
//! 3. **The deposit engine's fields** (P1-14's interface): whenever the surface has closed a frame
//!    of the recorded history this step, the stage adds `deeptime:trap_structure` (`traps.rs`) and
//!    the host-rock fractions `deeptime:host/<class>` (`hosts.rs`) to it, from the columns at the
//!    frame's end.
//! 4. **Keyframe layers** at the keyframe times: `structure/rock` (the rock at the surface: the
//!    top horizon of the baked column, where no pluton crops out), `structure/log_grade` (the
//!    highest metamorphic grade in the strata log), `structure/trap_structure`.
//! 5. At the end: the plutons become bodies (`WorldState::bodies`), the folds and faults the
//!    world's structural events (`WorldState::structure`), and the run's summary
//!    (`notes.structure`) records them, the metamorphosed volume and the baked stacks' shape.
//!
//! The horizon stacks themselves are baked on demand from the final world
//! ([`bake_horizons::bake`]): the CLI's viewer does it now, P1-08's bake writes them into the world
//! data.

pub mod bake_horizons;
pub mod columns;
pub mod deform;
pub mod horizon_cache;
pub mod hosts;
pub mod metamorphism;
pub mod params;
pub mod plutons;
pub mod realism;
pub mod traps;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::time::Instant;

use deeptime_format::GridDesc;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use params::StructureParams;

use crate::deposits::random::Rng;
use crate::error::CoreError;
use crate::params::{PlanetPreset, ScaleParams};
use crate::plates::l1::POOL_FIELDS;
use crate::plates::state::{CONTINENTAL, OROGENY_HIMALAYAN, pool, regime};
use crate::stage::{Stage, StageContext, StageEstimate, from_state, to_state};
use crate::surface::SurfaceState;
use crate::surface::grid::Dims;
use crate::synth::EVERY_CLASS;
use crate::world::{Field, WorldState};
use columns::{Cell, Tables};
use metamorphism::{Paths, Thermal};
use plutons::{ArcInputs, Counts, Pluton};

/// The sedimentary cover as the world will draw it: the smoothed horizon stacks the bake writes
/// (`bake_horizons::bake`, before the top raise, which only lifts the first top) read by the
/// column rule (`deeptime_format::strata`), and per class whether it is sedimentary or loose
/// (the chemical and clastic sedimentary and unconsolidated families, regolith included). The
/// coal seams of a site are held inside it (`deposits`), so they sit in the rock the strata pass
/// paints around them.
pub struct Cover {
    hz: bake_horizons::Horizons,
    desc: deeptime_format::LayerDesc,
    sedimentary: Vec<bool>,
    /// The stacks were the ones the structure stage kept (`horizon_cache`), not baked again.
    pub kept: bool,
    /// The world's folds and faults (`WorldState::structure`, the events the bake writes), which
    /// the strata pass reads every stack through ([`Cover::drawn_horizons_at`]); `None` without
    /// events.
    deform: Option<deeptime_format::structure::Deformer>,
}

impl Cover {
    /// Bakes `world`'s horizon stacks the way the bake will (errors where the world lacks the
    /// fields the columns need).
    pub fn new(world: &WorldState, preset: &PlanetPreset) -> Result<Cover, CoreError> {
        let (hz, kept) = bake_horizons::bake_noting(world, preset)?;
        let desc = hz.desc(deeptime_format::Codec::Raw);
        let sedimentary = crate::surface::lithology::table()
            .iter()
            .map(|c| {
                matches!(
                    c.family.as_str(),
                    "chemical_sedimentary" | "clastic_sedimentary" | "unconsolidated"
                )
            })
            .collect();
        let deform = world
            .structure
            .as_ref()
            .filter(|s| !s.events.is_empty())
            .map(|s| {
                deeptime_format::structure::Deformer::new(
                    s.clone(),
                    preset.scale.circumference_blocks,
                )
            });
        Ok(Cover {
            hz,
            desc,
            sedimentary,
            kept,
            deform,
        })
    }

    /// Bytes the stacks hold, with the folds and faults (their records and about as much again
    /// for the index; for the stage's memory reservation).
    pub fn bytes(&self) -> u64 {
        let events = self.deform.as_ref().map_or(0, |d| d.structure.events.len());
        (self.hz.tops.len() * 7 + events * 2 * deeptime_format::structure::EVENT_BYTES) as u64
    }

    /// The column's horizons at block (x, z) by the column rule, top-down: (top, the next
    /// horizon's top or the floor, class). A horizon fills the blocks from its top down to that
    /// lower bound + 1 (the last one down to the floor).
    pub fn horizons_at(&self, x: f64, z: f64) -> Vec<(i32, i32, u8)> {
        let mut col = Vec::new();
        self.plain_stack(x.floor() as i64, z.floor() as i64, &mut col);
        Self::spans(&col, self.hz.min_y)
    }

    /// [`Cover::horizons_at`] as the strata pass paints the column: the stack read through the
    /// world's folds and faults (`deeptime_format::column::ColumnWorld::stack`, Java's
    /// `StrataPass.column`), so a horizon is where the player finds it.
    pub fn drawn_horizons_at(&self, x: f64, z: f64) -> Vec<(i32, i32, u8)> {
        let (bx, bz) = (x.floor() as i64, z.floor() as i64);
        let mut col = Vec::new();
        match &self.deform {
            Some(d) => d.stack(
                bx,
                bz,
                self.hz.min_y,
                &mut |sx, sz, o| self.plain_stack(sx, sz, o),
                &mut deeptime_format::structure::DeformScratch::default(),
                &mut col,
            ),
            None => self.plain_stack(bx, bz, &mut col),
        }
        Self::spans(&col, self.hz.min_y)
    }

    /// (top, the next horizon's top or the floor, class) of a stack, top-down.
    fn spans(col: &[deeptime_format::strata::ColumnHorizon], floor: i32) -> Vec<(i32, i32, u8)> {
        (0..col.len())
            .map(|h| (col[h].0, col.get(h + 1).map_or(floor, |n| n.0), col[h].1))
            .collect()
    }

    /// The stack of block column (x, z) before any fold or fault (`deeptime_format::strata`).
    fn plain_stack(&self, x: i64, z: i64, out: &mut Vec<deeptime_format::strata::ColumnHorizon>) {
        let k = self.hz.k as usize;
        let w = self.hz.grid.w as usize;
        let horizon = |i: u32, j: u32, h: u32| {
            let at = (j as usize * w + i as usize) * k + h as usize;
            deeptime_format::strata::StoredHorizon {
                top: self.hz.tops[at],
                class: self.hz.classes[at],
                key: self.hz.keys[at],
            }
        };
        deeptime_format::strata::stack_at(&self.desc, self.hz.min_y, horizon, x, z, out);
    }

    /// The world's floor (block y).
    pub fn min_y(&self) -> i32 {
        self.hz.min_y
    }

    /// The stacks' cell size (blocks): the spacing of the stored stacks the column rule blends.
    pub fn cell_blocks(&self) -> f64 {
        f64::from(self.hz.grid.r)
    }

    /// Blocks of sedimentary rock under `surface_y` at block (x, z) by the column rule: down to
    /// the top of the column's first horizon of another rock at least [`COVER_MIN_BLOCKS`] thick
    /// (a thinner tongue does not end it), or to the floor.
    pub fn blocks_at(&self, x: f64, z: f64, surface_y: f64) -> f64 {
        let floor = self.hz.min_y;
        for (top, bottom, class) in self.horizons_at(x, z) {
            let sed = self
                .sedimentary
                .get(usize::from(class))
                .copied()
                .unwrap_or(false);
            if !sed && top - bottom >= COVER_MIN_BLOCKS {
                return (surface_y - f64::from(top)).max(0.0);
            }
        }
        (surface_y - f64::from(floor)).max(0.0)
    }
}

/// The thinnest horizon of another rock that ends the sedimentary cover ([`Cover::blocks_at`]).
// ledger: STR-009 (game choice)
pub const COVER_MIN_BLOCKS: i32 = 3;

/// The stage's name in a pipeline (`deeptime stage structure`).
pub const STAGE_NAME: &str = "structure";

/// Fields the stage writes.
pub mod field {
    /// The rock class at the surface (index into `synth::EVERY_CLASS`).
    pub const ROCK: &str = "structure/rock";
    /// The highest metamorphic grade in the strata log (0–4, `metamorphism::grade`).
    pub const LOG_GRADE: &str = "structure/log_grade";
    /// Structural closure, 0–1 (`traps.rs`).
    pub const TRAP: &str = "structure/trap_structure";
}

/// Keyframe layers.
pub const KEYFRAME_LAYERS: [&str; 3] = [field::ROCK, field::LOG_GRADE, field::TRAP];

/// The trap field's name in the deposit engine's history.
pub const TRAP_FIELD: &str = "deeptime:trap_structure";

/// Everything one cell's column is made of, for every cell of the L1 grid.
pub struct CellSource<'a> {
    pub grid: GridDesc,
    /// The surface's time, Myr.
    pub now_myr: f64,
    pub sea_level_m: f64,
    pub surface: &'a SurfaceState,
    pools: [&'a [f32]; pool::COUNT],
    crust: &'a [f32],
    regime: &'a [f32],
    orogeny: &'a [f32],
    orogeny_age: &'a [f32],
    heat: &'a [f32],
    t_mean: Option<&'a [f32]>,
    /// The last surface step's deposition and erosion rates (m/Myr), where the surface wrote them.
    rates: Option<(&'a [f32], &'a [f32])>,
    /// The run's start, Myr (the strata log's oldest possible age).
    t_start_myr: f64,
    p: StructureParams,
}

fn field_of<'a>(
    fields: &'a BTreeMap<String, Field>,
    name: &str,
    n: usize,
) -> Result<&'a [f32], CoreError> {
    fields
        .get(name)
        .filter(|f| f.data.len() == n)
        .map(|f| f.data.as_slice())
        .ok_or_else(|| {
            CoreError::InvalidState(format!(
                "the structure stage needs the L1 field {name} (a plate step with l1_every_steps = 1)"
            ))
        })
}

impl<'a> CellSource<'a> {
    pub fn new(world: &'a WorldState, preset: &PlanetPreset) -> Result<CellSource<'a>, CoreError> {
        let surface = world.surface.as_ref().ok_or_else(|| {
            CoreError::InvalidState("the structure stage runs after the surface stage".into())
        })?;
        Self::with(&world.fields, surface, preset)
    }

    fn with(
        fields: &'a BTreeMap<String, Field>,
        surface: &'a SurfaceState,
        preset: &PlanetPreset,
    ) -> Result<CellSource<'a>, CoreError> {
        let grid = l1_grid(preset)?;
        let n = grid.cells();
        let mut pools: [&[f32]; pool::COUNT] = [&[]; pool::COUNT];
        for (k, name) in POOL_FIELDS.iter().enumerate() {
            pools[k] = field_of(fields, name, n)?;
        }
        Ok(CellSource {
            grid,
            now_myr: surface.t_myr,
            sea_level_m: surface.sea_level_m,
            surface,
            pools,
            crust: field_of(fields, "plates/crust_type", n)?,
            regime: field_of(fields, "regime", n)?,
            orogeny: field_of(fields, "plates/orogeny_type", n)?,
            orogeny_age: field_of(fields, "plates/orogeny_age_myr", n)?,
            heat: field_of(fields, "plates/heat_flow_mw_m2", n)?,
            t_mean: fields
                .get("climate/t_mean_c")
                .filter(|f| f.data.len() == n)
                .map(|f| f.data.as_slice()),
            rates: field_of(fields, crate::surface::field::DEPOSITION, n)
                .ok()
                .zip(field_of(fields, crate::surface::field::EROSION, n).ok()),
            t_start_myr: preset.t_start_years() / 1.0e6,
            p: preset.structure.clone().unwrap_or_default(),
        })
    }

    /// Whether cell `c` is still accumulating: the last step's deposition minus erosion, summed
    /// over the cell and its eight neighbours (x wraps, z clamps), is positive. True where the
    /// surface wrote no rates.
    pub fn top_active(&self, c: usize) -> bool {
        let Some((dep, ero)) = self.rates else {
            return true;
        };
        let (w, h) = (self.grid.w as i64, self.grid.h as i64);
        let (i, j) = ((c as i64) % w, (c as i64) / w);
        let mut net = 0.0f64;
        for dj in -1..=1 {
            let jj = (j + dj).clamp(0, h - 1);
            for di in -1..=1 {
                let k = (jj * w + (i + di).rem_euclid(w)) as usize;
                let (d, e) = (f64::from(dep[k]), f64::from(ero[k]));
                if d.is_finite() && e.is_finite() {
                    net += d - e;
                }
            }
        }
        net > 0.0
    }

    /// Cell `c`'s inputs.
    pub fn cell(&self, c: usize) -> Cell<'a> {
        let s = self.surface;
        Cell {
            regolith_m: s.regolith_m[c],
            regolith_kind: s.regolith_kind[c],
            log: s.log.column(c),
            pools: std::array::from_fn(|k| self.pools[k][c]),
            crust_type: self.crust[c] as u8,
            regime: self.regime[c] as u8,
            orogeny: self.orogeny[c] as u8,
            orogeny_age: self.orogeny_age[c],
            depth_below_sea_m: self.sea_level_m - f64::from(s.elevation_m[c]),
            surface_c: self.surface_c(c),
            gradient_c_m: self.gradient_c_m(c),
            log_span_myr: (self.now_myr - self.t_start_myr).max(1.0),
            top_active: self.top_active(c),
        }
    }

    /// The surface temperature of the geotherm at cell `c`, °C.
    pub fn surface_c(&self, c: usize) -> f64 {
        self.t_mean
            .map_or(self.p.surface_temp_c, |t| f64::from(t[c]))
            .max(0.0)
    }

    /// The present geotherm at cell `c`, °C/m (`metamorphism.rs`).
    pub fn gradient_c_m(&self, c: usize) -> f64 {
        f64::from(self.heat[c].clamp(0.0, self.p.max_heat_flow_mw_m2 as f32))
            / 1000.0
            / self.p.conductivity_w_mk
    }

    /// The surface of cell `c` in blocks.
    pub fn surface_y(&self, c: usize, s: &ScaleParams) -> f64 {
        f64::from(s.sea_level_y)
            + (f64::from(self.surface.elevation_m[c]) - self.sea_level_m) / s.m_per_block_v
    }

    pub fn continental(&self, c: usize) -> bool {
        self.crust[c] as u8 == CONTINENTAL
    }

    pub fn regime(&self, c: usize) -> u8 {
        self.regime[c] as u8
    }
}

fn l1_grid(preset: &PlanetPreset) -> Result<GridDesc, CoreError> {
    Ok(GridDesc::for_planet(
        "L1",
        preset.scale.circumference_blocks,
        preset.resolution.l1_r,
    )?)
}

/// The stage's state (saved in checkpoints).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StructureState {
    pub done: u64,
    pub plutons: Vec<Pluton>,
    pub next_id: u32,
    /// Every plate's Ω at the end of the last step (what the next plate step turned it by).
    pub omega: Vec<[f64; 3]>,
    /// Frames of the recorded history the stage has published into.
    pub frames_seen: u64,
    /// The next keyframe time, years.
    pub next_kf_years: f64,
    pub counts: Counts,
    /// Strata-log volume that changed class, km³ (block metric).
    pub metamorphosed_km3: f64,
    /// Folds and faults (P2-01), riding their plates.
    pub features: Vec<deform::Feature>,
    pub next_feature: u32,
    pub deform_counts: deform::Counts,
}

/// The structure stage.
pub struct StructureStage {
    preset: PlanetPreset,
    p: StructureParams,
    total: u64,
    st: StructureState,
    tables: Tables,
    paths: Paths,
    host_classes: Vec<u16>,
    /// Class → index into `host_classes` (usize::MAX: not published).
    slot: Vec<usize>,
    timings: BTreeMap<&'static str, f64>,
    /// The world's events that can start a fold or a fault (not saved: rebuilt from the world).
    causes: deform::Causes,
    /// A step taken out of the world to run beside the next plate step (`Stage::detach`).
    detached: Option<Box<Parts>>,
}

/// What a step reads of the plate model: its time and every plate's rotation and parent, as the
/// plate step before it left them.
struct PlatesNow {
    t_myr: f64,
    omega: Vec<[f64; 3]>,
    parents: Vec<Option<u32>>,
}

impl PlatesNow {
    fn of(world: &WorldState) -> Result<PlatesNow, CoreError> {
        let plates = world.plates.as_ref().ok_or_else(|| {
            CoreError::InvalidState("the structure stage runs after a plate step".into())
        })?;
        Ok(PlatesNow {
            t_myr: plates.t_myr,
            omega: plates
                .plates
                .iter()
                .map(|p| if p.alive { p.omega } else { [0.0; 3] })
                .collect(),
            parents: plates.plates.iter().map(|p| p.parent).collect(),
        })
    }
}

/// The world fields a step reads ([`StructureStage::step_on`]) that the plate step writes anew
/// (the L1 fields) or does not touch: moved out of the world while the two run side by side, and
/// put back after, where the plate step wrote none of the name.
const MOVED: [&str; 18] = [
    "plates/heat_flow_mw_m2",
    "regime",
    "plates/thickness_m",
    "plates/orogeny_type",
    "plates/orogeny_age_myr",
    "plates/crust_type",
    "plates/plate_id",
    "plates/flags",
    "plates/convergence_mm_yr",
    "plates/obliquity_deg",
    "plates/fold_trend_deg",
    "plates/uplift_mm_yr",
    "plates/collision_mm_yr",
    "plates/shortening",
    "climate/t_mean_c",
    crate::surface::field::DEPOSITION,
    crate::surface::field::EROSION,
    crate::surface::impacts::field::TRAP,
];

/// The world fields a step reads that the plate step reads or takes itself (the surface's
/// material, which it takes into its pools, and the flexure it samples): copied.
const COPIED: [&str; 2] = [
    crate::surface::field::MATERIAL,
    crate::surface::flexure::field::FLEXURE,
];

/// A step taken out of the world to run beside the next plate step (P1-19, `Stage::detach`):
/// what [`StructureStage::step_on`] reads and the strata log it changes, and, once it ran, the
/// fields it made for the history.
struct Parts {
    now: PlatesNow,
    fields: BTreeMap<String, Field>,
    /// The names in `fields` moved out of the world ([`MOVED`]); the others are copies.
    moved: Vec<String>,
    surface: SurfaceState,
    /// The history's frame count when the step was taken out.
    frames: u64,
    /// Whether the step ran.
    stepped: bool,
    /// The deposit engine's fields for the frames it publishes into, when there are any.
    out: Option<BTreeMap<String, Vec<f32>>>,
}

/// The deposit engine's fields for the history's frames, by name, and the trap field among them
/// ([`StructureStage::frame_fields`]).
type FrameFields = (BTreeMap<String, Vec<f32>>, Vec<f32>);

/// The last step's folds and faults: the world's structure, the report, each feature's relief
/// left and the features.
type Finished = (
    deeptime_format::structure::Structure,
    serde_json::Value,
    Vec<f32>,
    Vec<deform::Feature>,
);

/// Estimate: the per-frame host fields and columns (plan §5.7.5 has no line for it yet).
pub fn estimate(p: &PlanetPreset) -> StageEstimate {
    let c = u64::from(p.scale.circumference_blocks);
    let r = u64::from(p.resolution.l1_r);
    let cells = (c / r) * (c / r / 2);
    let hosts = 32u64;
    StageEstimate {
        bytes: cells * (hosts * 4 + 3 * 4 + 64),
        seconds: cells as f64 * p.plate_steps() as f64 * 0.05e-6,
        history_bytes: 0,
    }
}

impl StructureStage {
    pub fn new(preset: PlanetPreset) -> StructureStage {
        let p = preset.structure.clone().unwrap_or_default();
        let total = preset.plate_steps();
        let host_classes = crate::deposits::types::builtin()
            .map(|t| hosts::host_classes(&t))
            .unwrap_or_default();
        let mut slot = vec![usize::MAX; EVERY_CLASS.len()];
        for (s, &k) in host_classes.iter().enumerate() {
            slot[usize::from(k)] = s;
        }
        let st = StructureState {
            next_kf_years: preset.t_start_years(),
            ..StructureState::default()
        };
        StructureStage {
            preset,
            p,
            total,
            st,
            tables: Tables::get(),
            paths: Paths::get(),
            host_classes,
            slot,
            timings: BTreeMap::new(),
            causes: deform::Causes::default(),
            detached: None,
        }
    }

    fn advance(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        let grid = l1_grid(&self.preset)?;
        let s = self.preset.scale.clone();
        let dx = f64::from(grid.r) * s.m_per_block_h;
        let d = Dims::new(grid.w as usize, grid.h as usize, dx, true);
        let now = PlatesNow::of(world)?;
        let t_end_myr = now.t_myr;
        let last = self.st.done + 1 >= self.total;
        self.causes.update(&world.events, s.circumference_blocks);
        let frames = world.history.as_ref().map_or(0, |h| h.frame_count() as u64);
        let WorldState {
            fields,
            surface,
            history,
            volcanoes,
            horizons: kept,
            ..
        } = world;
        let surface = surface.as_mut().ok_or_else(|| {
            CoreError::InvalidState("the structure stage runs after the surface stage".into())
        })?;
        let published = self.step_on(&now, fields, surface, frames, ctx)?;
        let trap = match published {
            Some((fields_out, trap)) => {
                self.publish(history, frames, &fields_out)?;
                Some(trap)
            }
            None => None,
        };
        let src = CellSource::with(fields, surface, &self.preset)?;
        let p = &self.p;
        let tables = &self.tables;

        // 4. Keyframe layers.
        let started = Instant::now();
        let t_end_years = t_end_myr * 1.0e6;
        let every = self.preset.keyframes.every_myr * 1.0e6;
        let keyframe = last || self.keyframe_at(t_end_myr);
        let mut outputs: Vec<(&str, Vec<f32>)> = Vec::new();
        if keyframe {
            let trap = match trap {
                Some(t) => t,
                None => self.trap_field(&src, &d, fields),
            };
            outputs.push((field::ROCK, self.rock_map(&src, &s)));
            outputs.push((field::LOG_GRADE, self.log_grade(&src.surface.log)));
            outputs.push((field::TRAP, trap));
            if every > 0.0 {
                let t0 = self.preset.t_start_years();
                self.st.next_kf_years = t0 + (((t_end_years - t0) / every).floor() + 1.0) * every;
            }
        }
        let summary = if last {
            let horizons = bake_horizons::bake_with(&src, &self.preset, p, tables);
            // Kept for the deposit stage's cover and the bake, which read the same inputs: in a
            // file of their own in the run's folder, so they hold no memory meanwhile.
            kept.keep(
                src.fingerprint(&self.preset, p),
                &horizons,
                Some(ctx.out_dir),
            );
            ctx.log.push(
                crate::log::Level::Info,
                format!(
                    "structure: the horizons kept for the deposit stage and the bake, {:.1} MiB packed, {}",
                    kept.bytes() as f64 / 1_048_576.0,
                    if kept.on_disk() { "in a file of their own" } else { "in memory" }
                ),
            );
            Some((horizons.mean_horizons(), horizons.check()))
        } else {
            None
        };
        let trap_shares = last.then(|| self.trap_shares(&src, &d, fields));
        add_time(&mut self.timings, "outputs", started);
        let surface_y: Vec<f64> = if last {
            self.st
                .plutons
                .iter()
                .map(|pl| src.surface_y(plutons::cell_of(&grid, pl.x, pl.z), &s))
                .collect()
        } else {
            Vec::new()
        };
        // The folds and faults as the world's structural events, with a lattice evaluation of
        // the whole planet (what it finds, and what it costs).
        let structure = if last {
            let started = Instant::now();
            // Everything the run left, with the plugs of the volcanoes that have died (made
            // here, from the catalogue as it stands: they are not kept in the stage's state).
            let mut all = self.st.features.clone();
            let plugs = deform::intrude::plugs(
                volcanoes
                    .as_ref()
                    .map_or(&[][..], |v| v.volcanoes.as_slice()),
                &p.deformation,
                s.m_per_block_h,
                self.st.next_feature,
            );
            self.st.deform_counts.plugs = plugs.len() as u64;
            all.extend(plugs);
            deform::undo_order(&mut all);
            let span = (src.now_myr - self.preset.t_start_years() / 1.0e6).max(1.0);
            let now = src.now_myr;
            let mut st = deform::finish(
                &all,
                |x, z| src.surface_y(plutons::cell_of(&grid, x, z), &s),
                |f| columns::key::of_age(0, now - f64::from(f.t_myr), span),
                f64::from(s.min_y),
            );
            let relief: Vec<f32> = all
                .iter()
                .map(|f| deform::relief_left(f, now, &p.deformation, s.m_per_block_v))
                .collect();
            let report = deform::report(
                &all,
                &mut st,
                &self.preset,
                &self.st.deform_counts,
                src.regime,
                src.crust,
            )?;
            add_time(&mut self.timings, "deformation_lattice", started);
            let mut report = report;
            if let Some(serde_json::Value::Object(timing)) =
                report.as_object_mut().and_then(|o| o.remove("timing"))
            {
                for (key, name) in [
                    ("lattice_s", "deformation_columns_s"),
                    ("threads", "deformation_threads"),
                    ("noddy_ns_point", "deformation_noddy_ns_point"),
                    ("column_ns_point", "deformation_column_ns_point"),
                    ("noddy_planet_s", "deformation_noddy_planet_s"),
                ] {
                    if let Some(v) = timing.get(key).and_then(serde_json::Value::as_f64) {
                        self.timings.insert(name, v);
                    }
                }
            }
            Some((st, report, relief, all))
        } else {
            None
        };
        drop(src);
        for (name, data) in outputs {
            fields.insert(
                name.to_owned(),
                Field {
                    grid: grid.clone(),
                    data,
                },
            );
        }

        // 5. The end: bodies and the summary.
        if last {
            self.finish(world, ctx, &surface_y, summary, structure, trap_shares)?;
        }
        Ok(())
    }

    /// Whether the step that ends at `t_myr` writes the keyframe layers (besides the last).
    fn keyframe_at(&self, t_myr: f64) -> bool {
        let every = self.preset.keyframes.every_myr * 1.0e6;
        every > 0.0 && t_myr * 1.0e6 >= self.st.next_kf_years - 1.0
    }

    /// Steps 1 to 3 of a step, the part that writes nothing of the world but the strata log
    /// (P1-19: so it can run beside the next plate step, `Stage::detach`): the metamorphism of the
    /// strata log, the plutons and the folds and faults, and, when the history has frames the
    /// stage has not published into (`frames`, the history's count), the deposit engine's fields
    /// for them with the trap field. `now` is the plate model after the plate step this one
    /// follows. The stage's own state moves on, except `frames_seen` ([`StructureStage::publish`]).
    /// The reservation comes first: a step refused the memory has changed nothing.
    fn step_on(
        &mut self,
        now: &PlatesNow,
        fields: &BTreeMap<String, Field>,
        surface: &mut SurfaceState,
        frames: u64,
        ctx: &StageContext<'_>,
    ) -> Result<Option<FrameFields>, CoreError> {
        let grid = l1_grid(&self.preset)?;
        let n = grid.cells();
        let s = self.preset.scale.clone();
        let dx = f64::from(grid.r) * s.m_per_block_h;
        let d = Dims::new(grid.w as usize, grid.h as usize, dx, true);
        let t_end_myr = now.t_myr;
        let dt_myr = self.preset.dt_myr();
        let parents = &now.parents;
        if surface.log.cells() != n {
            return Err(CoreError::InvalidState(format!(
                "the strata log has {} cells, L1 has {n}",
                surface.log.cells()
            )));
        }
        let publishes = frames > self.st.frames_seen;
        if publishes {
            ctx.reserve((n * (self.host_classes.len() + 2) * 4) as u64)?;
        }

        // 1. Metamorphism of the strata log.
        let started = Instant::now();
        let heat = field_of(fields, "plates/heat_flow_mw_m2", n)?;
        let reg = field_of(fields, "regime", n)?;
        let thick = field_of(fields, "plates/thickness_m", n)?;
        let orogeny = field_of(fields, "plates/orogeny_type", n)?;
        let orogeny_age = field_of(fields, "plates/orogeny_age_myr", n)?;
        let crust_m = field_of(fields, "plates/crust_type", n)?;
        let t_mean = fields
            .get("climate/t_mean_c")
            .filter(|f| f.data.len() == n)
            .map(|f| f.data.as_slice());
        let p = &self.p;
        let paths = &self.paths;
        let k = surface.log.k as usize;
        // Each column's metamorphosed thickness in parallel, then their sum over the cells in
        // order: a parallel `sum` of floats groups its terms as the threads split the work, so
        // its last bits changed from run to run (the history_cap test caught the note differing
        // between two runs of one planet; P1-17).
        let changed: f64 = {
            let log = &mut surface.log;
            let regolith = &surface.regolith_m;
            let per_cell: Vec<f64> = log
                .layers
                .par_chunks_mut(k)
                .zip(log.count.par_iter_mut())
                .enumerate()
                .map(|(c, (layers, count))| {
                    // Regional metamorphism is a continental process here: sediment on oceanic
                    // crust is subducted or accreted (and then continental) before it is buried
                    // deep enough.
                    if *count == 0 || crust_m[c] as u8 != CONTINENTAL {
                        return 0.0;
                    }
                    let collision = reg[c] as u8 == regime::COLLISION
                        || (orogeny[c] as u8 == OROGENY_HIMALAYAN
                            && orogeny_age[c].is_finite()
                            && orogeny_age[c] < 30.0);
                    let burial = if collision {
                        (f64::from(thick[c]) / p.reference_crust_m).clamp(1.0, p.max_burial_factor)
                    } else {
                        1.0
                    };
                    let th = Thermal {
                        surface_c: t_mean
                            .map_or(p.surface_temp_c, |t| f64::from(t[c]))
                            .max(0.0),
                        gradient_c_m: f64::from(heat[c].clamp(0.0, p.max_heat_flow_mw_m2 as f32))
                            / 1000.0
                            / p.conductivity_w_mk,
                        burial,
                    };
                    let mut col = crate::surface::strata_log::ColumnMut { layers, count };
                    metamorphism::metamorphose(&mut col, f64::from(regolith[c]), &th, paths)
                })
                .collect();
            per_cell.iter().sum()
        };
        let cell_km2 = dx * dx / 1.0e6;
        self.st.metamorphosed_km3 += changed * cell_km2 / 1000.0;
        add_time(&mut self.timings, "metamorphism", started);

        // 2. Plutons: carried, unroofed, emplaced.
        let started = Instant::now();
        let plate_id = field_of(fields, "plates/plate_id", n)?;
        let crust = field_of(fields, "plates/crust_type", n)?;
        let material = fields
            .get(crate::surface::field::MATERIAL)
            .filter(|f| f.data.len() == n)
            .map(|f| f.data.as_slice());
        let old_omega = self.st.omega.clone();
        let omega_of = |mut k: u32| -> [f64; 3] {
            for _ in 0..64 {
                if let Some(o) = old_omega.get(k as usize) {
                    return *o;
                }
                match parents.get(k as usize).copied().flatten() {
                    Some(pk) => k = pk,
                    None => break,
                }
            }
            [0.0; 3]
        };
        plutons::advance(
            &mut self.st.plutons,
            &grid,
            &omega_of,
            dt_myr * 1.0e6,
            plate_id,
            crust,
            material,
            &mut self.st.counts,
        );
        let src = CellSource::with(fields, surface, &self.preset)?;
        let arcs = ArcInputs {
            flags: field_of(fields, "plates/flags", n)?,
            convergence: field_of(fields, "plates/convergence_mm_yr", n)?,
            crust_type: crust,
            regime: reg,
            plate_id,
        };
        let mut rng = Rng::stream(
            step_seed(self.preset.seed, self.st.done),
            "structure/plutons",
        );
        let tables = &self.tables;
        let host_at = |c: usize, depth: f64| {
            let mut units = Vec::new();
            columns::build(
                &src.cell(c),
                tables,
                p,
                src.now_myr,
                depth + 1.0,
                &mut units,
            );
            plutons::host_class(&units, depth)
        };
        let born = plutons::emplace(
            &grid,
            s.m_per_block_h,
            &arcs,
            p,
            tables,
            dt_myr,
            t_end_myr,
            &mut rng,
            &mut self.st.next_id,
            host_at,
        );
        self.st.counts.emplaced += born.len() as u64;
        self.st.plutons.extend(born);
        add_time(&mut self.timings, "plutons", started);

        // 2b. Folds and faults: carried, then started where the crust is strained.
        let started = Instant::now();
        if p.deformation.enabled() {
            deform::advance(
                &mut self.st.features,
                &grid,
                &omega_of,
                dt_myr * 1.0e6,
                plate_id,
                crust,
                &mut self.st.deform_counts,
            );
            if let Some(m) = material {
                deform::wear(&mut self.st.features, &grid, m);
            }
            let flags = field_of(fields, "plates/flags", n)?;
            // The two distance transforms are serial sweeps: side by side (P1-17).
            let (orogen_km, arc_km) = rayon::join(
                || traps::orogen_distance_km(&d, reg, crust, flags),
                || traps::arc_distance_km(&d, reg, crust, flags),
            );
            let inputs = deform::Inputs {
                grid: &grid,
                m_per_block_h: s.m_per_block_h,
                m_per_block_v: s.m_per_block_v,
                regime: reg,
                crust_type: crust,
                flags,
                convergence: field_of(fields, "plates/convergence_mm_yr", n)?,
                obliquity: field_of(fields, "plates/obliquity_deg", n)?,
                thickness: thick,
                fold_trend: field_of(fields, "plates/fold_trend_deg", n)?,
                plate_id,
                uplift: field_of(fields, "plates/uplift_mm_yr", n)?,
                elevation: &src.surface.elevation_m,
                orogen_km: &orogen_km,
                arc_km: &arc_km,
                collision_mm_yr: field_of(fields, "plates/collision_mm_yr", n)?,
                shortening_km: field_of(fields, "plates/shortening", n)?,
                flexure_m: fields
                    .get(crate::surface::flexure::field::FLEXURE)
                    .filter(|f| f.data.len() == n)
                    .map(|f| f.data.as_slice()),
                collision_belt_km: self.preset.tectonics.collision_belt_km,
                blocks_per_model_km: f64::from(s.circumference_blocks)
                    / (std::f64::consts::TAU * self.preset.tectonics.model_radius_km),
            };
            let mut rng = Rng::stream(
                step_seed(self.preset.seed, self.st.done),
                "structure/deform",
            );
            let causes = &self.causes;
            let cause = |kind: deform::Kind, x: f64, z: f64| {
                deform::cause_of(
                    causes,
                    kind,
                    x,
                    z,
                    s.circumference_blocks,
                    t_end_myr * 1.0e6,
                )
            };
            deform::emplace(
                &inputs,
                p,
                &mut self.st.features,
                dt_myr,
                t_end_myr,
                &mut rng,
                &mut self.st.next_feature,
                &mut self.st.deform_counts,
                &cause,
            );
            // Stocks and dykes, on a random stream of their own (the folds and faults keep
            // theirs whatever the intrusion settings).
            let mut rng = Rng::stream(
                step_seed(self.preset.seed, self.st.done),
                "structure/intrude",
            );
            deform::intrude::emplace(
                &inputs,
                p,
                &mut self.st.features,
                dt_myr,
                t_end_myr,
                &mut rng,
                &mut self.st.next_feature,
                &mut self.st.deform_counts,
                &cause,
            );
        }
        add_time(&mut self.timings, "deformation", started);

        // 3. The deposit engine's fields for frames the surface closed this step.
        let started = Instant::now();
        let out = publishes.then(|| self.frame_fields(&src, &d, fields, &s));
        add_time(&mut self.timings, "frame_fields", started);
        self.st.omega = now.omega.clone();
        Ok(out)
    }

    /// Publishes the deposit engine's fields (`out`, from [`StructureStage::step_on`]) into the
    /// history's frames the stage has not published into, up to `frames`.
    fn publish(
        &mut self,
        history: &mut Option<crate::deposits::recorded::RecordedHistory>,
        frames: u64,
        out: &BTreeMap<String, Vec<f32>>,
    ) -> Result<(), CoreError> {
        let started = Instant::now();
        let h = history.as_mut().ok_or_else(|| {
            CoreError::InvalidState("the structure stage's frames have no history".into())
        })?;
        for fk in self.st.frames_seen..frames {
            h.add_fields(fk as usize, out)?;
        }
        self.st.frames_seen = frames;
        add_time(&mut self.timings, "frame_fields", started);
        Ok(())
    }

    /// The last step's end: the plutons as bodies, the folds and faults as the world's structure
    /// and its events, and the stage's note.
    fn finish(
        &mut self,
        world: &mut WorldState,
        ctx: &StageContext<'_>,
        surface_y: &[f64],
        summary: Option<(f64, Result<(), String>)>,
        structure: Option<Finished>,
        trap_shares: Option<serde_json::Value>,
    ) -> Result<(), CoreError> {
        let s = self.preset.scale.clone();
        let p = &self.p;
        let mut bodies = 0u64;
        let plutons = std::mem::take(&mut self.st.plutons);
        for (pl, &y) in plutons.iter().zip(surface_y) {
            let id = world.next_body_id();
            if let Some(b) = plutons::to_body(
                pl,
                id,
                y,
                s.m_per_block_h,
                s.m_per_block_v,
                s.min_y,
                p.aureole_shell,
            ) {
                world.add_body(b)?;
                bodies += 1;
            }
        }
        self.st.plutons = plutons;
        let (mean_horizons, check) = summary.expect("baked on the last step");
        let (structure, mut deformation, relief, all) =
            structure.expect("finished on the last step");
        // The event log's entries: one per fold belt and fault system.
        let groups = deform::groups(
            &all,
            f64::from(s.circumference_blocks),
            s.m_per_block_h,
            s.m_per_block_v,
        );
        deformation["events"] = json!(groups.len());
        for g in groups {
            let mut e = world
                .event(g.t_start, g.kind, deform::SOURCE)
                .until(g.t_end)
                .at(deeptime_format::Place::Point {
                    lat: g.lat,
                    lon: g.lon,
                })
                .magnitude(g.magnitude)
                .param("setting", g.setting);
            for (k, v) in g.params {
                e = e.param(k, v);
            }
            if g.cause != 0 && world.events.iter().any(|x| x.id.0 == g.cause) {
                e = e.caused_by(deeptime_format::EventId(g.cause));
            }
            world.emit(e)?;
        }
        deformation["with_relief"] = json!(relief.iter().filter(|r| **r > 0.01).count());
        deformation["traps"] = trap_shares.unwrap_or_default();
        world.structure = Some(structure);
        world.structure_relief = relief;
        ctx.log.push(
            crate::log::Level::Info,
            format!("structure deformation: {deformation}"),
        );
        let note = json!({
            "steps": self.st.done + 1,
            "plutons": {
                "emplaced": self.st.counts.emplaced,
                "alive": self.st.plutons.len(),
                "eroded": self.st.counts.eroded,
                "subducted": self.st.counts.subducted,
                "bodies": bodies,
            },
            "metamorphosed_km3": self.st.metamorphosed_km3,
            "deformation": deformation,
            "frames_published": self.st.frames_seen,
            "host_classes": self.host_classes.iter().map(|&k| EVERY_CLASS[usize::from(k)]).collect::<Vec<_>>(),
            "horizons": {
                "k": self.p.horizons,
                "mean_per_cell": mean_horizons,
                "check": match &check { Ok(()) => "ok".to_owned(), Err(e) => e.clone() },
            },
            "millis": self.timings,
        });
        world.notes.insert(STAGE_NAME.into(), note.to_string());
        if let Err(e) = check {
            return Err(CoreError::InvalidState(format!("horizon bake: {e}")));
        }
        Ok(())
    }

    /// Trap structure and host fractions from the columns (and the plutons) as they are now.
    fn frame_fields(
        &self,
        src: &CellSource<'_>,
        d: &Dims,
        fields: &BTreeMap<String, Field>,
        s: &ScaleParams,
    ) -> (BTreeMap<String, Vec<f32>>, Vec<f32>) {
        let n = src.grid.cells();
        let hc = self.host_classes.len();
        let window = self.p.host_window_m;
        let mut fr = vec![0.0f32; n * hc.max(1)];
        let p = &self.p;
        let t = &self.tables;
        // BIF inherited in the basement of the iron-formation basins under a stable interior
        // (`deposits::bif_basins`, `deposits::clocks`): the basement is its host there.
        let bif_window =
            crate::deposits::clocks::Epoch::of_preset(&self.preset).inherited_bif_window() as f32;
        let basin = &src.surface.bif_basin;
        let bif_slot =
            self.slot[usize::from(crate::surface::lithology::index("banded_iron_formation"))];
        fr.par_chunks_mut(hc.max(1))
            .enumerate()
            .for_each_init(Vec::new, |units, (c, out)| {
                columns::build(&src.cell(c), t, p, src.now_myr, window, units);
                let share = basin.get(c).map_or(0.0, |&b| b * bif_window);
                let inherited =
                    (share > 0.0 && bif_slot != usize::MAX && src.regime(c) == regime::CRATON)
                        .then_some((bif_slot, share));
                hosts::fractions_inherited(units, window, &self.slot, inherited, out);
            });
        // Plutons take their share of the window from what was there.
        for pl in &self.st.plutons {
            let s_pl = self.slot[usize::from(pl.class)];
            plutons::for_each_cell(pl, &src.grid, s.m_per_block_h, |c, r2| {
                let (top, bottom) = plutons::span_at(pl, r2);
                let part = (bottom.min(window) - top.max(0.0)).max(0.0) / window;
                if part <= 0.0 || hc == 0 {
                    return;
                }
                let row = &mut fr[c * hc..(c + 1) * hc];
                for v in row.iter_mut() {
                    *v *= (1.0 - part) as f32;
                }
                if s_pl != usize::MAX {
                    row[s_pl] += part as f32;
                }
            });
        }
        let mut out = BTreeMap::new();
        for (si, &k) in self.host_classes.iter().enumerate() {
            out.insert(
                hosts::field_name(k),
                (0..n).into_par_iter().map(|c| fr[c * hc + si]).collect(),
            );
        }
        let trap = self.trap_field(src, d, fields);
        out.insert(TRAP_FIELD.to_owned(), trap.clone());
        (out, trap)
    }

    fn trap_field(
        &self,
        src: &CellSource<'_>,
        d: &Dims,
        fields: &BTreeMap<String, Field>,
    ) -> Vec<f32> {
        self.trap_field_as(src, d, fields, TrapSource::Configured)
    }

    /// The share of continental cells whose structural closure is at least a half, three ways:
    /// as the world has it, from the regime rule alone, and from the regime rule with the belt
    /// width on the block metric (what Phase 1 did; `fold_belt_km` was read as block-km).
    fn trap_shares(
        &self,
        src: &CellSource<'_>,
        d: &Dims,
        fields: &BTreeMap<String, Field>,
    ) -> serde_json::Value {
        let n = src.grid.cells();
        let land = (0..n).filter(|&c| src.continental(c)).count().max(1) as f64;
        let share = |from: TrapSource| {
            let t = self.trap_field_as(src, d, fields, from);
            let (mut half, mut sum) = (0u64, 0.0f64);
            for (c, &v) in t.iter().enumerate() {
                if src.continental(c) {
                    half += u64::from(v >= 0.5);
                    sum += f64::from(v);
                }
            }
            json!({"closure_at_least_half": half as f64 / land, "mean": sum / land})
        };
        json!({
            "world": share(TrapSource::Configured),
            "regime_rule": share(TrapSource::Regime),
            "regime_rule_block_km": share(TrapSource::RegimeBlockKm),
        })
    }

    fn trap_field_as(
        &self,
        src: &CellSource<'_>,
        d: &Dims,
        fields: &BTreeMap<String, Field>,
        from: TrapSource,
    ) -> Vec<f32> {
        let n = src.grid.cells();
        let flags = fields
            .get("plates/flags")
            .map(|f| f.data.as_slice())
            .unwrap_or(&[]);
        let zero = vec![0.0f32; n];
        let flags = if flags.len() == n { flags } else { &zero };
        let dist = traps::orogen_distance_km(d, src.regime, src.crust, flags);
        let halite = self.tables.halite;
        let p = &self.p;
        // Impact structures (P2-08): the closure the impact stage found at its craters.
        let craters = fields
            .get(crate::surface::impacts::field::TRAP)
            .map(|f| f.data.as_slice())
            .filter(|v| v.len() == n);
        // The world's own folds and faults, where it has any (P2-01).
        let structural = (from == TrapSource::Configured && p.deformation.enabled()).then(|| {
            let s = &self.preset.scale;
            let x = p
                .deformation
                .block_stretch(s.m_per_block_h, s.m_per_block_v);
            traps::structural(&self.st.features, &src.grid, x)
        });
        // Distances on the plate model's Earth-sized sphere, as `fold_belt_km` is given: the
        // orogen distance is km on the block metric (300 m a block), and a planet 16,384 blocks
        // round is about an eighth of the Earth's size on it.
        let s = &self.preset.scale;
        let model_km_per_block_km = if from == TrapSource::RegimeBlockKm {
            1.0
        } else {
            1000.0
                / s.m_per_block_h
                / (f64::from(s.circumference_blocks)
                    / (std::f64::consts::TAU * self.preset.tectonics.model_radius_km))
        };
        (0..n)
            .into_par_iter()
            .map(|c| {
                traps::closure(
                    src.surface.log.column(c),
                    f64::from(src.surface.regolith_m[c]),
                    halite,
                    src.continental(c),
                    src.regime(c),
                    f64::from(dist[c]) * model_km_per_block_km,
                    structural.as_ref().map(|v| v[c]),
                    p,
                )
                .max(craters.map_or(0.0, |v| v[c]))
            })
            .collect()
    }

    /// The rock at the surface: each cell's top baked horizon, and plutons where they crop out.
    fn rock_map(&self, src: &CellSource<'_>, s: &ScaleParams) -> Vec<f32> {
        let n = src.grid.cells();
        let p = &self.p;
        let t = &self.tables;
        let mut rock: Vec<f32> = (0..n)
            .into_par_iter()
            .map_init(
                || {
                    (
                        Vec::new(),
                        vec![0i16; p.horizons as usize],
                        vec![0u8; p.horizons as usize],
                        vec![[0u16; 2]; p.horizons as usize],
                    )
                },
                |(units, tops, classes, keys), c| {
                    let surface_y = src.surface_y(c, s);
                    // Enough of the column that the first horizon is settled.
                    columns::build(&src.cell(c), t, p, src.now_myr, 2000.0, units);
                    bake_horizons::bake_cell(
                        units,
                        surface_y,
                        s.min_y,
                        s.max_y,
                        s.m_per_block_v,
                        &t.min_blocks,
                        tops,
                        classes,
                        keys,
                    );
                    f32::from(classes[0])
                },
            )
            .collect();
        for pl in &self.st.plutons {
            plutons::for_each_cell(pl, &src.grid, s.m_per_block_h, |c, r2| {
                if let Some(k) = plutons::outcrop_at(pl, r2, p.aureole_shell) {
                    rock[c] = f32::from(k);
                }
            });
        }
        rock
    }

    fn log_grade(&self, log: &crate::surface::strata_log::StrataLog) -> Vec<f32> {
        (0..log.cells())
            .into_par_iter()
            .map(|c| {
                log.column(c)
                    .iter()
                    .map(|l| self.paths.grades[usize::from(l.class)])
                    .max()
                    .map_or(0.0, f32::from)
            })
            .collect()
    }
}

/// Where a trap field's fold and fault closure comes from.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TrapSource {
    /// The world's own structures when it has any, else the regime rule.
    Configured,
    /// The regime rule (belts within `fold_belt_km` of an orogen, rifts).
    Regime,
    /// The regime rule with the belt width in km on the block metric (Phase 1's reading).
    RegimeBlockKm,
}

fn add_time(timings: &mut BTreeMap<&'static str, f64>, what: &'static str, since: Instant) {
    *timings.entry(what).or_insert(0.0) += since.elapsed().as_secs_f64() * 1000.0;
}

/// The seed of step `step`'s draws (a resumed run draws the same plutons).
fn step_seed(seed: u64, step: u64) -> u64 {
    seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ step.wrapping_add(0x5157_7275_6374_7572)
}

/// The run summary line (the CLI's `structure_final=…`), from `notes.structure`.
pub fn summary_line(note: &serde_json::Value) -> String {
    let pl = &note["plutons"];
    let hz = &note["horizons"];
    let df = &note["deformation"];
    let kinds = &df["structures"];
    let deformation = format!(
        " fold_trains={} thrusts={} normal_faults={} strike_slip_faults={} \
         stocks={} dykes={} plugs={} \
         deformed_frac={:.4} faulted_frac={:.4} max_candidates={} max_shift_blocks={:.1} \
         lattice_s={:.2} noddy_worst={:.1e} noddy_ns_point={:.0} column_ns_point={:.1} \
         covered_craton={:.3} covered_collision={:.3} covered_rift={:.3} covered_passive_margin={:.3} \
         deform_steps_s={:.2} deform_report_s={:.2} \
         formed_by_arc={}/{} formed_by_collision={}/{} formed_by_foreland={}/{} \
         arc_cell_steps={} arc_mean_lift={:.3} arc_lifted_share={:.3} arc_mean_strain={:.4}",
        kinds["fold_train"].as_u64().unwrap_or(0),
        kinds["thrust"].as_u64().unwrap_or(0),
        kinds["normal_fault"].as_u64().unwrap_or(0),
        kinds["strike_slip_fault"].as_u64().unwrap_or(0),
        kinds["stock"].as_u64().unwrap_or(0),
        kinds["dyke"].as_u64().unwrap_or(0),
        kinds["plug"].as_u64().unwrap_or(0),
        df["lattice"]["deformed_frac"].as_f64().unwrap_or(0.0),
        df["lattice"]["faulted_frac"].as_f64().unwrap_or(0.0),
        df["lattice"]["max_candidates"].as_u64().unwrap_or(0),
        df["lattice"]["max_shift_blocks"].as_f64().unwrap_or(0.0),
        note["millis"]["deformation_columns_s"]
            .as_f64()
            .unwrap_or(0.0),
        df["noddy_check"]["worst"].as_f64().unwrap_or(0.0),
        note["millis"]["deformation_noddy_ns_point"]
            .as_f64()
            .unwrap_or(0.0),
        note["millis"]["deformation_column_ns_point"]
            .as_f64()
            .unwrap_or(0.0),
        df["coverage"]["craton"]["covered"].as_f64().unwrap_or(0.0),
        df["coverage"]["collision"]["covered"]
            .as_f64()
            .unwrap_or(0.0),
        df["coverage"]["rift"]["covered"].as_f64().unwrap_or(0.0),
        df["coverage"]["passive_margin"]["covered"]
            .as_f64()
            .unwrap_or(0.0),
        note["millis"]["deformation"].as_f64().unwrap_or(0.0) / 1000.0,
        note["millis"]["deformation_lattice"]
            .as_f64()
            .unwrap_or(0.0)
            / 1000.0,
        df["formed_by"]["arc"]["fold_train"],
        df["formed_by"]["arc"]["thrust"],
        df["formed_by"]["collision"]["fold_train"],
        df["formed_by"]["collision"]["thrust"],
        df["formed_by"]["foreland"]["fold_train"],
        df["formed_by"]["foreland"]["thrust"],
        df["arc_rule"]["cell_steps"],
        df["arc_rule"]["mean_lift"].as_f64().unwrap_or(f64::NAN),
        df["arc_rule"]["lifted_share"].as_f64().unwrap_or(f64::NAN),
        df["arc_rule"]["mean_strain"].as_f64().unwrap_or(f64::NAN),
    );
    format!(
        "structure_final={} plutons_emplaced={} plutons_alive={} plutons_eroded={} \
         plutons_subducted={} pluton_bodies={} metamorphosed_km3={:.3e} frames_published={} \
         horizons_k={} mean_horizons={:.2} horizon_check={}{deformation}",
        note["steps"],
        pl["emplaced"],
        pl["alive"],
        pl["eroded"],
        pl["subducted"],
        pl["bodies"],
        note["metamorphosed_km3"].as_f64().unwrap_or(0.0),
        note["frames_published"],
        hz["k"],
        hz["mean_per_cell"].as_f64().unwrap_or(0.0),
        hz["check"].as_str().unwrap_or("?"),
    )
}

#[derive(Serialize, Deserialize)]
struct Saved {
    st: StructureState,
}

impl Stage for StructureStage {
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
        self.st.done
    }

    fn step(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        if self.st.done >= self.total {
            return Ok(());
        }
        if self.detached.is_some() {
            return Err(CoreError::InvalidState(
                "the structure step is out of the world".into(),
            ));
        }
        ctx.pool.install(|| self.advance(world, ctx))?;
        self.st.done += 1;
        Ok(())
    }

    fn keyframe_layers(&self) -> &[&'static str] {
        &KEYFRAME_LAYERS
    }

    fn coupled_to(&self) -> Option<&'static str> {
        Some(crate::plates::STAGE_NAME)
    }

    fn steps_beside_lead(&self) -> bool {
        true
    }

    /// The steps between keyframes write nothing of the world but the strata log
    /// ([`StructureStage::step_on`]), and the plate step reads neither the surface nor any field
    /// the step reads but the two it copies ([`COPIED`]). The last step (bodies, events, the
    /// note) and the keyframe steps (their layers) run on the world.
    fn detach(&mut self, world: &mut WorldState) -> Result<bool, CoreError> {
        if self.detached.is_some() {
            return Err(CoreError::InvalidState(
                "the structure step is out of the world already".into(),
            ));
        }
        if self.st.done + 1 >= self.total || world.surface.is_none() {
            return Ok(false);
        }
        let Ok(now) = PlatesNow::of(world) else {
            return Ok(false);
        };
        if self.keyframe_at(now.t_myr) {
            return Ok(false);
        }
        self.causes
            .update(&world.events, self.preset.scale.circumference_blocks);
        let frames = world.history.as_ref().map_or(0, |h| h.frame_count() as u64);
        let mut fields = BTreeMap::new();
        let mut moved = Vec::new();
        for &name in MOVED.iter().chain(POOL_FIELDS.iter()) {
            if let Some(f) = world.fields.remove(name) {
                fields.insert(name.to_owned(), f);
                moved.push(name.to_owned());
            }
        }
        for name in COPIED {
            if let Some(f) = world.fields.get(name) {
                let copy = Field {
                    grid: f.grid.clone(),
                    data: crate::alloc::copied(&f.data),
                };
                fields.insert(name.to_owned(), copy);
            }
        }
        let surface = world.surface.take().expect("checked above");
        self.detached = Some(Box::new(Parts {
            now,
            fields,
            moved,
            surface,
            frames,
            stepped: false,
            out: None,
        }));
        Ok(true)
    }

    fn step_detached(&mut self, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        let mut parts = self.detached.take().ok_or_else(|| {
            CoreError::InvalidState("no structure step is out of the world".into())
        })?;
        let p = &mut *parts;
        let result = if p.stepped {
            Ok(())
        } else {
            match ctx
                .pool
                .install(|| self.step_on(&p.now, &p.fields, &mut p.surface, p.frames, ctx))
            {
                Ok(out) => {
                    p.out = out.map(|(fields_out, _)| fields_out);
                    p.stepped = true;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        };
        self.detached = Some(parts);
        result
    }

    fn attach(&mut self, world: &mut WorldState) -> Result<(), CoreError> {
        let Some(parts) = self.detached.take() else {
            return Ok(());
        };
        let Parts {
            fields,
            moved,
            surface,
            frames,
            stepped,
            out,
            ..
        } = *parts;
        world.surface = Some(surface);
        let mut copies = Vec::new();
        for (name, f) in fields {
            if moved.contains(&name) {
                world.fields.entry(name).or_insert(f);
            } else {
                copies.push(f);
            }
        }
        if !stepped {
            return Err(CoreError::InvalidState(
                "the structure step taken out of the world did not run".into(),
            ));
        }
        if let Some(out) = &out {
            self.publish(&mut world.history, frames, out)?;
        }
        // The copies and the published fields (grid-sized arrays, and the kept blocks they push
        // out) are unmapped beside the next step, not before it (P1-19: 5.6 s of a full-16k run).
        crate::alloc::free_beside((copies, out));
        self.st.done += 1;
        Ok(())
    }

    fn save_state(&self) -> Result<Vec<u8>, CoreError> {
        to_state(&Saved {
            st: self.st.clone(),
        })
    }

    fn load_state(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        let s: Saved = from_state(bytes)?;
        self.st = s.st;
        self.causes = deform::Causes::default();
        self.detached = None;
        Ok(())
    }
}

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
//! 3. **The deposit engine's fields** (P1-14's interface): whenever the surface has closed a frame
//!    of the recorded history this step, the stage adds `deeptime:trap_structure` (`traps.rs`) and
//!    the host-rock fractions `deeptime:host/<class>` (`hosts.rs`) to it, from the columns at the
//!    frame's end.
//! 4. **Keyframe layers** at the keyframe times: `structure/rock` (the rock at the surface: the
//!    top horizon of the baked column, where no pluton crops out), `structure/log_grade` (the
//!    highest metamorphic grade in the strata log), `structure/trap_structure`.
//! 5. At the end: the plutons become bodies (`WorldState::bodies`), and the run's summary
//!    (`notes.structure`) records them, the metamorphosed volume and the baked stacks' shape.
//!
//! The horizon stacks themselves are baked on demand from the final world
//! ([`bake_horizons::bake`]): the CLI's viewer does it now, P1-08's bake writes them into the world
//! data.

pub mod bake_horizons;
pub mod columns;
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
}

impl Cover {
    /// Bakes `world`'s horizon stacks the way the bake will (errors where the world lacks the
    /// fields the columns need).
    pub fn new(world: &WorldState, preset: &PlanetPreset) -> Result<Cover, CoreError> {
        let hz = bake_horizons::bake(world, preset)?;
        let mut desc = deeptime_format::LayerDesc::scalar(
            "strata/horizons",
            deeptime_format::DType::HorizonStack,
            hz.grid.clone(),
            deeptime_format::Codec::Raw,
        );
        desc.scale = 1.0 / 16.0;
        desc.stack_depth = Some(hz.k);
        desc.tongue_band_keys = Some(hz.tongues.0);
        desc.tongue_minor_share = Some(hz.tongues.1);
        let sedimentary = crate::surface::lithology::table()
            .iter()
            .map(|c| {
                matches!(
                    c.family.as_str(),
                    "chemical_sedimentary" | "clastic_sedimentary" | "unconsolidated"
                )
            })
            .collect();
        Ok(Cover {
            hz,
            desc,
            sedimentary,
        })
    }

    /// Bytes the stacks hold (for the stage's memory reservation).
    pub fn bytes(&self) -> u64 {
        (self.hz.tops.len() * 7) as u64
    }

    /// Blocks of sedimentary rock under `surface_y` at block (x, z) by the column rule: down to
    /// the top of the column's first horizon of another rock at least [`COVER_MIN_BLOCKS`] thick
    /// (a thinner tongue does not end it), or to the floor.
    pub fn blocks_at(&self, x: f64, z: f64, surface_y: f64) -> f64 {
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
        let mut col = Vec::new();
        deeptime_format::strata::stack_at(
            &self.desc,
            self.hz.min_y,
            horizon,
            x.floor() as i64,
            z.floor() as i64,
            &mut col,
        );
        let floor = self.hz.min_y;
        for (h, &(top, class, _)) in col.iter().enumerate() {
            let bottom = col.get(h + 1).map_or(floor, |n| n.0);
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
}

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
        }
    }

    fn advance(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        let grid = l1_grid(&self.preset)?;
        let n = grid.cells();
        let s = self.preset.scale.clone();
        let dx = f64::from(grid.r) * s.m_per_block_h;
        let d = Dims::new(grid.w as usize, grid.h as usize, dx, true);
        let plates = world.plates.as_ref().ok_or_else(|| {
            CoreError::InvalidState("the structure stage runs after a plate step".into())
        })?;
        let t_end_myr = plates.t_myr;
        let dt_myr = self.preset.length_myr() / self.total.max(1) as f64;
        let omega_now: Vec<[f64; 3]> = plates
            .plates
            .iter()
            .map(|p| if p.alive { p.omega } else { [0.0; 3] })
            .collect();
        let parents: Vec<Option<u32>> = plates.plates.iter().map(|p| p.parent).collect();
        let last = self.st.done + 1 >= self.total;
        let WorldState {
            fields,
            surface,
            history,
            ..
        } = world;
        let surface = surface.as_mut().ok_or_else(|| {
            CoreError::InvalidState("the structure stage runs after the surface stage".into())
        })?;
        if surface.log.cells() != n {
            return Err(CoreError::InvalidState(format!(
                "the strata log has {} cells, L1 has {n}",
                surface.log.cells()
            )));
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
        let changed: f64 = {
            let log = &mut surface.log;
            let regolith = &surface.regolith_m;
            log.layers
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
                .sum()
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

        // 3. The deposit engine's fields for frames the surface closed this step.
        let started = Instant::now();
        let frames = history.as_ref().map_or(0, |h| h.frame_count() as u64);
        let mut trap: Option<Vec<f32>> = None;
        if frames > self.st.frames_seen {
            ctx.reserve((n * (self.host_classes.len() + 2) * 4) as u64)?;
            let (fields_out, tr) = self.frame_fields(&src, &d, fields, &s);
            let h = history.as_mut().expect("frames exist");
            for fk in self.st.frames_seen..frames {
                h.add_fields(fk as usize, &fields_out)?;
            }
            self.st.frames_seen = frames;
            trap = Some(tr);
        }
        add_time(&mut self.timings, "frame_fields", started);

        // 4. Keyframe layers.
        let started = Instant::now();
        let t_end_years = t_end_myr * 1.0e6;
        let every = self.preset.keyframes.every_myr * 1.0e6;
        let keyframe = last || (every > 0.0 && t_end_years >= self.st.next_kf_years - 1.0);
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
            Some((horizons.mean_horizons(), horizons.check()))
        } else {
            None
        };
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
            let mut bodies = 0u64;
            let plutons = std::mem::take(&mut self.st.plutons);
            for (pl, &y) in plutons.iter().zip(&surface_y) {
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
        }
        self.st.omega = omega_now;
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
        fr.par_chunks_mut(hc.max(1))
            .enumerate()
            .for_each_init(Vec::new, |units, (c, out)| {
                columns::build(&src.cell(c), t, p, src.now_myr, window, units);
                hosts::fractions(units, window, &self.slot, out);
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
                (0..n).map(|c| fr[c * hc + si]).collect(),
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
        (0..n)
            .into_par_iter()
            .map(|c| {
                traps::closure(
                    src.surface.log.column(c),
                    f64::from(src.surface.regolith_m[c]),
                    halite,
                    src.continental(c),
                    src.regime(c),
                    f64::from(dist[c]),
                    p,
                )
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
    format!(
        "structure_final={} plutons_emplaced={} plutons_alive={} plutons_eroded={} \
         plutons_subducted={} pluton_bodies={} metamorphosed_km3={:.3e} frames_published={} \
         horizons_k={} mean_horizons={:.2} horizon_check={}",
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

    fn save_state(&self) -> Result<Vec<u8>, CoreError> {
        to_state(&Saved {
            st: self.st.clone(),
        })
    }

    fn load_state(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        let s: Saved = from_state(bytes)?;
        self.st = s.st;
        Ok(())
    }
}

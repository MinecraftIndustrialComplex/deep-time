// SPDX-License-Identifier: LGPL-3.0-or-later
//! Amplification from L2 to L3 (plan §8.2, work item P1-06): the young landscape's terrain is
//! doubled level by level, each level eroded, slumped and silted with multiple-flow passes, until
//! it has the cells the world reads; the last level is written tile by tile (tile streaming,
//! plan §5.7.5) as the L3 layers `elevation` and `detail/amplitude`.
//!
//! # Method
//! Schott et al. 2024, "Terrain Amplification using Multi-scale Erosion" (ACM TOG 43(4)); code
//! `H-Schott/MultiScaleErosion` (MIT, commit 964a3d5, ported with attribution in NOTICE.md and
//! the module headers). Their workflow alternates ×2 upsampling with an erosion pass, a thermal
//! pass and a deposition pass; `kernels.rs` and `thermal.rs` hold the ports and what changed.
//! Here, per level:
//! 1. **Upsample** ×2 (`resample.rs`): clamped Catmull–Rom terrain, the drainage area the level
//!    below computed (the erosion pass's prior), the runoff weight and rock erodibility, and the
//!    footprint of the lakes the young landscape had.
//! 2. **Seed** smooth noise of amplitude `seed_relief · slope · cell size` on the land, so the
//!    passes have concavities to concentrate flow in below the coarse level's resolution.
//! 3. **Erode** `erosion_iterations` times: multiple-flow drainage area refined from its prior,
//!    stream-power incision scaled by the rock's erodibility, sea level the base level, lakes
//!    left alone.
//! 4. **Slump** `thermal_iterations` times: slopes over the critical slope × (0.25 + 0.75 ·
//!    drama) (`surface.drama`, the editor's slider) shed material downhill.
//! 5. **Silt** `deposition_iterations` times: the eroded material travels along the flow and
//!    settles where the capacity falls below the load (valley floors, fans).
//! 6. **Flood** (`kernels::flood_repair`): every land cell outside the lakes is made to drain.
//!
//! Levels but the last run on the whole raster with `coarse_iteration_factor ×` the iterations
//! (they are a quarter or less of the cells); the last runs on tiles with a halo so that its peak
//! memory does not depend on the planet, and its result is the same either way (`tiles.rs`).
//!
//! # No new sinks
//! A cell outside the lake footprint the young landscape had (grown by a cell per level) that
//! has no strictly lower neighbour is a *new internal sink*; the flood leaves none, and the stage
//! counts them over the finished tiles (`notes.amplify.sinks`, asserted 0 by the tests).
//!
//! # Detail layer
//! `detail.rs`: the local relief × drama, damped along channels, as the u8 layer
//! `detail/amplitude` that `deeptime:terrain` multiplies a C-periodic noise by (plan §8.2).

pub mod kernels;
pub mod resample;
pub mod tiles;
pub mod window;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::time::Instant;

use deeptime_format::{Codec, DType, GridDesc, LayerEntry, Manifest};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use self::kernels::{Buf, Counts, FloodStats, NoiseCfg, Physics, Times, run_passes};
use self::resample::Level;
use self::tiles::{
    LastCfg, LastStats, PackSink, Preview, TilePlan, TileSink, halo as plan_halo, run_last,
};
use self::window::Win;
use super::detail::{DetailParams, Noise};
use super::thermal;
use super::young;
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::stage::{Stage, StageContext, StageEstimate, from_state, to_state};
use crate::world::{Field, WorldState};

/// The stage's name in a pipeline (`deeptime stage amplify`).
pub const STAGE_NAME: &str = "amplify";

/// Fields the stage publishes (thinned copies of the L3 result on the preview grid `L3p`, for the
/// keyframes and the workbench; the real layers are the tile packs).
pub mod field {
    /// Block y of the amplified terrain.
    pub const ELEVATION: &str = "amplify/elevation";
    /// The detail amplitude, blocks.
    pub const DETAIL: &str = "amplify/detail";
    /// log₁₀ of the effective drainage area, km² (NaN at sea).
    pub const DRAINAGE: &str = "amplify/drainage";
}

/// Keyframe layers of the stage.
pub const KEYFRAME_LAYERS: [&str; 3] = [field::ELEVATION, field::DETAIL, field::DRAINAGE];

/// Whether the last level streams.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamMode {
    /// In memory when the memory cap allows it, else tile by tile (plan §5.7.5).
    Auto,
    Always,
    Never,
}

/// The `amplify` section (plan §5.1 `SurfaceParams`, amplification): every key optional.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct AmplifyParams {
    /// Erosion, thermal and deposition iterations of the last level.
    pub erosion_iterations: u32,
    pub thermal_iterations: u32,
    pub deposition_iterations: u32,
    /// The levels before the last run this many times as many iterations.
    pub coarse_iteration_factor: f64,
    /// Multiple-flow exponent p (Schott: 1.3): a cell sends its area to each lower neighbour in
    /// proportion to slope^p.
    pub flow_power: f64,
    /// Stream-power exponents m (area; Schott 0.8) and n (slope; Schott 2).
    pub area_exponent: f64,
    pub slope_exponent: f64,
    /// Lowering per iteration, m, of a cell draining 100 km² (`area_ref_km2`) at a slope of 0.1
    /// (`slope_ref`) on rock of erodibility 1.
    pub incision_m: f64,
    pub area_ref_km2: f64,
    /// The area term stops growing here, km²: the rivers the coarse levels carved are not cut
    /// again, only their tributaries are made.
    pub area_cap_km2: f64,
    pub slope_ref: f64,
    /// The slope term stops growing here.
    pub slope_cap: f64,
    /// A cell lowers by at most this fraction of its drop to its receiver per iteration.
    pub max_incision_frac: f64,
    /// A cell stays at least this gradient above its receiver.
    pub min_gradient: f64,
    /// Critical slope at drama 1 (Schott: tan 30° ≈ 0.57), and the share of an excess a pair of
    /// cells shifts per iteration and the ± fraction its noise varies the slope by.
    pub thermal_slope: f64,
    pub thermal_rate: f64,
    pub thermal_noise: f64,
    /// Sediment, m per cell, a flow of 100 km² at slope 0.1 carries; the share of a load's excess
    /// over the capacity a cell drops per iteration.
    pub deposit_capacity_m: f64,
    pub deposit_fraction: f64,
    /// Amplitude of the relief seeding: metres per (slope · cell size).
    pub seed_relief: f64,
    /// The flood raises cells this much above the neighbour they drain to, m.
    pub sink_eps_m: f64,
    /// How far, cells, a depression the amplification makes may reach; the halo covers it.
    pub repair_margin_cells: u32,
    pub stream: StreamMode,
    /// Tile edge of a streamed last level, cells (a multiple of the format's 256).
    pub tile_cells: u32,
    /// Width of the thinned copy kept for keyframes, cells.
    pub preview_width: u32,
    pub detail: DetailParams,
}

impl Default for AmplifyParams {
    fn default() -> Self {
        AmplifyParams {
            erosion_iterations: 40,
            thermal_iterations: 24,
            deposition_iterations: 12,
            coarse_iteration_factor: 2.0,
            flow_power: 2.0,
            area_exponent: 0.8,
            slope_exponent: 2.0,
            incision_m: 8.0,
            area_ref_km2: 100.0,
            area_cap_km2: 300.0,
            slope_ref: 0.1,
            slope_cap: 0.5,
            max_incision_frac: 0.3,
            min_gradient: 1.0e-4,
            thermal_slope: 0.6,
            thermal_rate: 0.05,
            thermal_noise: 0.25,
            deposit_capacity_m: 4.0,
            deposit_fraction: 0.5,
            seed_relief: 0.25,
            sink_eps_m: 0.01,
            repair_margin_cells: 32,
            stream: StreamMode::Auto,
            tile_cells: 2048,
            preview_width: 2048,
            detail: DetailParams::default(),
        }
    }
}

impl AmplifyParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let bad = |k: &str, why: &str| CoreError::preset(format!("$.amplify.{k}"), why);
        let pos = |v: f64, k: &str| {
            if v.is_finite() && v > 0.0 {
                Ok(())
            } else {
                Err(bad(k, "must be > 0"))
            }
        };
        for (n, k) in [
            (self.erosion_iterations, "erosion_iterations"),
            (self.thermal_iterations, "thermal_iterations"),
            (self.deposition_iterations, "deposition_iterations"),
        ] {
            if n > 500 {
                return Err(bad(k, "must be at most 500"));
            }
        }
        if !(self.coarse_iteration_factor.is_finite()
            && (0.0..=20.0).contains(&self.coarse_iteration_factor))
        {
            return Err(bad("coarse_iteration_factor", "must be in [0, 20]"));
        }
        if !(1.0..=4.0).contains(&self.flow_power) {
            return Err(bad("flow_power", "must be in [1, 4]"));
        }
        if !(0.1..=2.0).contains(&self.area_exponent) {
            return Err(bad("area_exponent", "must be in [0.1, 2]"));
        }
        if !(0.5..=4.0).contains(&self.slope_exponent) {
            return Err(bad("slope_exponent", "must be in [0.5, 4]"));
        }
        pos(self.incision_m, "incision_m")?;
        pos(self.area_ref_km2, "area_ref_km2")?;
        pos(self.area_cap_km2, "area_cap_km2")?;
        pos(self.slope_ref, "slope_ref")?;
        pos(self.slope_cap, "slope_cap")?;
        if !(self.max_incision_frac > 0.0 && self.max_incision_frac <= 1.0) {
            return Err(bad("max_incision_frac", "must be in (0, 1]"));
        }
        if !(0.0..0.1).contains(&self.min_gradient) {
            return Err(bad("min_gradient", "must be in [0, 0.1)"));
        }
        pos(self.thermal_slope, "thermal_slope")?;
        if !(self.thermal_rate > 0.0 && self.thermal_rate <= 0.06) {
            return Err(bad(
                "thermal_rate",
                "must be in (0, 0.06] (larger rates overshoot)",
            ));
        }
        if !(0.0..=0.9).contains(&self.thermal_noise) {
            return Err(bad("thermal_noise", "must be in [0, 0.9]"));
        }
        pos(self.deposit_capacity_m, "deposit_capacity_m")?;
        if !(self.deposit_fraction > 0.0 && self.deposit_fraction <= 1.0) {
            return Err(bad("deposit_fraction", "must be in (0, 1]"));
        }
        if !(self.seed_relief.is_finite() && (0.0..=1.0).contains(&self.seed_relief)) {
            return Err(bad("seed_relief", "must be in [0, 1]"));
        }
        if !(self.sink_eps_m >= 0.001 && self.sink_eps_m <= 1.0) {
            return Err(bad("sink_eps_m", "must be in [0.001, 1]"));
        }
        if self.repair_margin_cells > 512 {
            return Err(bad("repair_margin_cells", "must be at most 512"));
        }
        if self.tile_cells < 256 || !self.tile_cells.is_multiple_of(256) {
            return Err(bad("tile_cells", "must be a multiple of 256, at least 256"));
        }
        if !(16..=16384).contains(&self.preview_width) {
            return Err(bad("preview_width", "must be in [16, 16384]"));
        }
        self.detail.validate("$.amplify.detail")
    }
}

/// n with `coarse_r = 2ⁿ · fine_r`, when there is one.
pub fn doublings(coarse_r: u32, fine_r: u32) -> Option<u32> {
    if fine_r == 0 || !coarse_r.is_multiple_of(fine_r) {
        return None;
    }
    let ratio = coarse_r / fine_r;
    ratio.is_power_of_two().then(|| ratio.trailing_zeros())
}

/// Everything a run of the ladder needs, resolved from the preset.
#[derive(Clone, Debug)]
pub struct Config {
    pub p: AmplifyParams,
    pub drama: f64,
    pub seed: u64,
    pub circumference: u32,
    /// Sea level, m (the plate model's datum), block y of the sea, metres per block, vertically.
    pub sea: f64,
    pub sea_y: f64,
    pub mv: f64,
    pub mh: f64,
}

impl Config {
    fn counts(&self, coarse: bool) -> Counts {
        let f = if coarse {
            self.p.coarse_iteration_factor
        } else {
            1.0
        };
        let n = |x: u32| (f64::from(x) * f).round() as u32;
        Counts {
            erode: n(self.p.erosion_iterations),
            thermal: n(self.p.thermal_iterations),
            deposit: n(self.p.deposition_iterations),
        }
    }

    fn physics(&self, dx: f64) -> Physics {
        let p = &self.p;
        Physics {
            dx: dx as f32,
            sea: self.sea as f32,
            flow_power: p.flow_power as f32,
            area_exp: p.area_exponent as f32,
            slope_exp: p.slope_exponent as f32,
            incision_m: p.incision_m as f32,
            area_ref_m2: (p.area_ref_km2 * 1.0e6) as f32,
            area_cap_m2: (p.area_cap_km2 * 1.0e6) as f32,
            slope_ref: p.slope_ref as f32,
            slope_cap: p.slope_cap as f32,
            max_frac: p.max_incision_frac as f32,
            min_gradient: p.min_gradient as f32,
            thermal_slope: thermal::critical_slope(p.thermal_slope, self.drama) as f32,
            thermal_rate: p.thermal_rate as f32,
            cap_ref_m: p.deposit_capacity_m as f32,
            cap_area_exp: p.area_exponent as f32,
            cap_slope_exp: 1.0,
            deposit_frac: p.deposit_fraction as f32,
            eps_m: p.sink_eps_m as f32,
        }
    }

    fn noise_cfg(&self, gw_fine: usize) -> NoiseCfg {
        let c = f64::from(self.circumference);
        NoiseCfg {
            noise: Noise::new(self.seed, self.circumference),
            seed_relief: self.p.seed_relief as f32,
            thermal_noise: self.p.thermal_noise as f32,
            cell_blocks: c / gw_fine as f64,
            origin_x: -c / 2.0,
            origin_z: -c / 4.0,
        }
    }

    /// The halo the last level needs.
    pub fn halo(&self) -> usize {
        plan_halo(&self.counts(false), self.p.repair_margin_cells as usize)
    }

    /// The plan of a last level `gw × gh` cells: one tile, or `tile_cells` tiles with the halo.
    pub fn plan(&self, gw: usize, gh: usize, streamed: bool) -> TilePlan {
        if streamed {
            TilePlan {
                gw,
                gh,
                tile: self.p.tile_cells as usize,
                halo: self.halo(),
            }
        } else {
            TilePlan::whole(gw, gh)
        }
    }
}

/// What a level before the last did.
#[derive(Clone, Debug)]
pub struct LevelReport {
    pub w: usize,
    pub h: usize,
    pub times: Times,
    pub pre_flood: FloodStats,
    pub flood: FloodStats,
    pub millis: f64,
}

/// What a whole amplification did.
pub struct Outcome {
    pub preview: Preview,
    pub last: LastStats,
    pub levels: Vec<LevelReport>,
    pub plan: TilePlan,
}

/// Amplifies `l2` by `doublings` (≥ 1) levels; the last one goes tile by tile through `sink`.
/// `reserve(bytes)` is asked before each allocation of buffers that size.
pub fn run(
    l2: Level,
    doublings: u32,
    cfg: &Config,
    plan: TilePlan,
    sink: &mut dyn TileSink,
    cancelled: &dyn Fn() -> bool,
    reserve: &dyn Fn(u64) -> Result<(), CoreError>,
) -> Result<Outcome, CoreError> {
    if doublings == 0 {
        return Err(CoreError::InvalidArgument(
            "amplification needs at least one ×2 level".into(),
        ));
    }
    let mut level = l2;
    let mut reports = Vec::new();
    for _ in 0..doublings - 1 {
        if cancelled() {
            return Err(CoreError::Cancelled);
        }
        let started = Instant::now();
        let (gw, gh) = (2 * level.w, 2 * level.h);
        let win = Win::full(gw, gh);
        reserve(win.cells() as u64 * Buf::BYTES_PER_CELL)?;
        let dxf = level.fine_dx();
        let fine = level.sample(&win);
        let mut buf = Buf::new(win, fine);
        let ph = cfg.physics(dxf);
        let mut times = Times::default();
        let pass = run_passes(
            &mut buf,
            &ph,
            &cfg.counts(true),
            &cfg.noise_cfg(gw),
            &mut times,
        );
        let a_fine = (dxf * dxf) as f32;
        let wgt: Vec<f32> = buf.own.par_iter().map(|&o| o / a_fine).collect();
        level = Level {
            w: gw,
            h: gh,
            dx: dxf,
            z: std::mem::take(&mut buf.z),
            area: std::mem::take(&mut buf.a),
            wgt,
            kfac: std::mem::take(&mut buf.kfac),
            lake: std::mem::take(&mut buf.lake),
        };
        reports.push(LevelReport {
            w: gw,
            h: gh,
            times,
            pre_flood: pass.pre_flood,
            flood: pass.flood,
            millis: started.elapsed().as_secs_f64() * 1000.0,
        });
    }
    let (gw, gh) = (2 * level.w, 2 * level.h);
    debug_assert_eq!((plan.gw, plan.gh), (gw, gh));
    let dxf = level.fine_dx();
    let last = LastCfg {
        ph: cfg.physics(dxf),
        counts: cfg.counts(false),
        noise: cfg.noise_cfg(gw),
        detail: &cfg.p.detail,
        drama: cfg.drama,
        mv: cfg.mv,
        plan,
        preview_stride: preview_stride(gw, cfg.p.preview_width as usize),
    };
    let (preview, stats) = run_last(&level, &last, sink, cancelled, reserve)?;
    Ok(Outcome {
        preview,
        last: stats,
        levels: reports,
        plan,
    })
}

/// The smallest stride ≥ `gw / width` that divides `gw` and leaves an even number of columns.
pub fn preview_stride(gw: usize, width: usize) -> usize {
    let mut s = (gw / width.max(1)).max(1);
    while s < gw {
        if gw.is_multiple_of(s) && (gw / s).is_multiple_of(2) {
            return s;
        }
        s += 1;
    }
    gw
}

/// The layers a finished stage wrote, as the manifest lists them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerNote {
    pub name: String,
    pub path: String,
    pub dtype: String,
    pub bytes: u64,
    pub crc32: u32,
}

/// Adds the L3 layers the amplify stage wrote (`world.notes["amplify"]`) to `manifest`, so a
/// finished world-data folder lists them (called by `Session::finalize_write`; a world without
/// the stage changes nothing).
pub fn register(world: &WorldState, manifest: &mut Manifest) -> Result<(), CoreError> {
    let Some(text) = world.notes.get(STAGE_NAME) else {
        return Ok(());
    };
    let v: Value = serde_json::from_str(text)
        .map_err(|e| CoreError::InvalidState(format!("notes.amplify: {e}")))?;
    let grid: GridDesc = serde_json::from_value(v["grid"].clone())
        .map_err(|e| CoreError::InvalidState(format!("notes.amplify.grid: {e}")))?;
    let layers: Vec<LayerNote> = serde_json::from_value(v["layers"].clone())
        .map_err(|e| CoreError::InvalidState(format!("notes.amplify.layers: {e}")))?;
    manifest.grids.insert(grid.name.clone(), grid.clone());
    for l in layers {
        let dtype = match l.dtype.as_str() {
            "i16" => DType::I16,
            "u8" => DType::U8,
            other => {
                return Err(CoreError::InvalidState(format!(
                    "notes.amplify.layers: dtype {other}"
                )));
            }
        };
        manifest.layers.push(LayerEntry {
            name: l.name,
            path: l.path,
            grid: grid.name.clone(),
            dtype,
            codec: Codec::Lz4Block,
            bytes: l.bytes,
            crc32: l.crc32,
        });
    }
    Ok(())
}

/// The young landscape's fields as the ladder's first level.
pub fn level_from_world(world: &WorldState, p: &PlanetPreset) -> Result<Level, CoreError> {
    let grid = GridDesc::for_planet("L2", p.scale.circumference_blocks, p.resolution.l2_r)?;
    let n = grid.cells();
    let get = |name: &str| -> Result<&Vec<f32>, CoreError> {
        world
            .fields
            .get(name)
            .filter(|f| f.data.len() == n)
            .map(|f| &f.data)
            .ok_or_else(|| {
                CoreError::InvalidState(format!(
                    "the amplify stage needs the young landscape's field {name} on L2 \
                     ({} × {} cells): run the young stage first",
                    grid.w, grid.h
                ))
            })
    };
    let lake = get(young::field::LAKE_M)?;
    Ok(Level {
        w: grid.w as usize,
        h: grid.h as usize,
        dx: f64::from(grid.r) * p.scale.m_per_block_h,
        z: get(young::field::ELEVATION_M)?.clone(),
        area: get(young::field::AREA_M2)?.clone(),
        wgt: get(young::field::WEIGHT)?.clone(),
        kfac: get(young::field::ERODIBILITY)?.clone(),
        lake: lake.iter().map(|&m| u8::from(m > 0.0)).collect(),
    })
}

/// Estimated bytes and seconds of the stage (plan §5.7.5; P1-17 replaces the second).
pub fn estimate(p: &PlanetPreset) -> StageEstimate {
    let a = p.amplify.clone().unwrap_or_default();
    let c = u64::from(p.scale.circumference_blocks);
    let cells = |r: u32| (c / u64::from(r)) * (c / u64::from(r) / 2);
    let l3 = cells(p.resolution.l3_r);
    let l2 = cells(p.resolution.l2_r);
    let levels = doublings(p.resolution.l2_r, p.resolution.l3_r).unwrap_or(1);
    let world = super::estimate(p).bytes;
    // The last level streams when its buffers would not fit next to the rest of the world under
    // the default cap (plan §5.7.5: "any preset whose estimate exceeds its cap streams L3
    // automatically"); a tile's window is the tile and a halo of about 200 cells on each side.
    let window = (u64::from(a.tile_cells) + 2 * 200).min(c);
    let streamed = window * window.min(c / 2) * Buf::BYTES_PER_CELL;
    let whole = l3 * Buf::BYTES_PER_CELL;
    let last_bytes = match a.stream {
        StreamMode::Always => streamed,
        StreamMode::Never => whole,
        StreamMode::Auto if world + l2 * 17 + whole > crate::session::DEFAULT_MEM_CAP => streamed,
        StreamMode::Auto => whole,
    };
    // The levels before the last hold their buffers whole.
    let mid = if levels > 1 {
        cells(p.resolution.l3_r << 1) * Buf::BYTES_PER_CELL
    } else {
        0
    };
    let mid_cells: u64 = (1..levels)
        .map(|k| cells(p.resolution.l3_r << (levels - k)))
        .sum();
    StageEstimate {
        bytes: world + l2 * 17 + last_bytes.max(mid),
        seconds: l3 as f64 * 1.2e-6 + mid_cells as f64 * 3.0e-6,
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Saved {
    done: bool,
}

/// The amplify stage.
pub struct AmplifyStage {
    preset: PlanetPreset,
    p: AmplifyParams,
    drama: f64,
    done: bool,
}

impl AmplifyStage {
    pub fn new(preset: PlanetPreset) -> AmplifyStage {
        let p = preset.amplify.clone().unwrap_or_default();
        let drama = preset.surface.clone().unwrap_or_default().drama;
        AmplifyStage {
            preset,
            p,
            drama,
            done: false,
        }
    }

    fn config(&self, sea: f64) -> Config {
        let s = &self.preset.scale;
        Config {
            p: self.p.clone(),
            drama: self.drama,
            seed: self.preset.seed,
            circumference: s.circumference_blocks,
            sea,
            sea_y: f64::from(s.sea_level_y),
            mv: s.m_per_block_v,
            mh: s.m_per_block_h,
        }
    }

    fn advance(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        let started = Instant::now();
        let r = &self.preset.resolution;
        let n_dbl = doublings(r.l2_r, r.l3_r)
            .filter(|&n| n >= 1)
            .ok_or_else(|| {
                CoreError::preset(
                    "$.resolution.l3_r",
                    format!(
                        "l2_r {} must be a power of two times l3_r {}, and larger than it",
                        r.l2_r, r.l3_r
                    ),
                )
            })?;
        let sea = world
            .surface
            .as_ref()
            .map(|s| s.sea_level_m)
            .ok_or_else(|| {
                CoreError::InvalidState("the amplify stage runs after the surface".into())
            })?;
        let c = self.preset.scale.circumference_blocks;
        let l3 = GridDesc::for_planet("L3", c, r.l3_r)?;
        let cfg = self.config(sea);
        let level = level_from_world(world, &self.preset)?;
        let (gw, gh) = (l3.w as usize, l3.h as usize);

        // Whole raster in memory, or tile by tile (plan §5.7.5: streams automatically when the
        // buffers would not fit under the memory cap).
        let whole_bytes = (gw * gh) as u64 * Buf::BYTES_PER_CELL;
        let streamed = match self.p.stream {
            StreamMode::Always => true,
            StreamMode::Never => false,
            StreamMode::Auto => ctx.reserve(whole_bytes).is_err(),
        };
        let plan = cfg.plan(gw, gh, streamed && self.p.tile_cells < (gw.max(gh)) as u32);
        let reserve = |bytes: u64| ctx.reserve(bytes);
        let mut sink = PackSink::create(ctx.out_dir, &l3, 256, sea, cfg.sea_y, cfg.mv)?;
        let setup_ms = started.elapsed().as_secs_f64() * 1000.0;
        let cancelled = || ctx.cancelled();
        let out = run(level, n_dbl, &cfg, plan, &mut sink, &cancelled, &reserve)?;
        let files = sink.finish()?;

        // The thinned copies (keyframes, the workbench) …
        let s = out.preview.stride;
        let grid_p = GridDesc::for_planet("L3p", c, r.l3_r * s as u32)?;
        debug_assert_eq!(
            (grid_p.w as usize, grid_p.h as usize),
            (out.preview.w, out.preview.h)
        );
        let put = |world: &mut WorldState, name: &str, data: Vec<f32>| {
            world.fields.insert(
                name.to_owned(),
                Field {
                    grid: grid_p.clone(),
                    data,
                },
            );
        };
        let y = |z: f32| (cfg.sea_y + (f64::from(z) - sea) / cfg.mv) as f32;
        put(
            world,
            field::ELEVATION,
            out.preview.z_m.iter().map(|&z| y(z)).collect(),
        );
        put(world, field::DETAIL, out.preview.amp_blocks.clone());
        put(
            world,
            field::DRAINAGE,
            out.preview
                .z_m
                .iter()
                .zip(&out.preview.drainage)
                .map(|(&z, &d)| if f64::from(z) <= sea { f32::NAN } else { d })
                .collect(),
        );

        // … and the notes.
        let layer =
            |x: &(deeptime_format::LayerDesc, String, deeptime_format::Written)| LayerNote {
                name: x.0.name.clone(),
                path: x.1.clone(),
                dtype: match x.0.dtype {
                    DType::I16 => "i16",
                    _ => "u8",
                }
                .into(),
                bytes: x.2.bytes,
                crc32: x.2.crc32,
            };
        let a_l3 = (f64::from(l3.r) * cfg.mh).powi(2);
        let km3 = a_l3 / 1.0e9;
        let t = &out.last;
        let total_ms = started.elapsed().as_secs_f64() * 1000.0;
        let levels: Vec<Value> = out
            .levels
            .iter()
            .map(|l| {
                json!({
                    "w": l.w, "h": l.h, "millis": l.millis,
                    "passes": timing_json(&l.times),
                    "pre_flood_raised_cells": l.pre_flood.raised_cells,
                    "flood_raised_cells": l.flood.raised_cells,
                    "flood_max_raise_m": l.flood.max_raise_m,
                })
            })
            .collect();
        let note = json!({
            "grid": l3,
            "l2": {"r": r.l2_r},
            "doublings": n_dbl,
            "mode": if plan.tile >= gw.max(gh) { "in_memory" } else { "streamed" },
            "tile_cells": plan.tile,
            "halo": plan.halo,
            "tiles": t.tiles,
            "window_cells_max": t.window_cells_max,
            "cells": t.cells,
            "drama": self.drama,
            "erosion_iterations": self.p.erosion_iterations,
            "thermal_iterations": self.p.thermal_iterations,
            "deposition_iterations": self.p.deposition_iterations,
            "sinks": t.sinks,
            "pre_flood_raised_cells": t.pre_flood_raised_cells,
            "pre_flood_max_raise_m": t.pre_flood_max_raise_m,
            "flood_raised_cells": t.flood_raised_cells,
            "flood_raised_km3": t.flood_raised_m_cells * km3,
            "flood_max_raise_m": t.flood_max_raise_m,
            "flood_max_raise_at": [t.flood_max_raise_at.0, t.flood_max_raise_at.1],
            "flood_unreached": t.flood_unreached,
            "mean_abs_change_m": t.change_m_cells / (t.land_cells.max(1) as f64),
            "eroded_km3": t.eroded_m_cells * km3,
            "lost_in_transit_km3": t.transit_m_cells * km3,
            "levels": levels,
            "last": {
                "sample_ms": t.sample_ms, "detail_ms": t.detail_ms, "sink_ms": t.sink_ms,
                "passes": timing_json(&t.times),
            },
            "millis": {"setup": setup_ms, "total": total_ms},
            "preview": {"w": out.preview.w, "h": out.preview.h, "stride": s},
            "layers": [layer(&files.elevation), layer(&files.amplitude)],
        });
        world.notes.insert(STAGE_NAME.into(), note.to_string());
        ctx.log.push(crate::log::Level::Info, summary_line(&note));
        self.done = true;
        Ok(())
    }
}

fn timing_json(t: &Times) -> Value {
    json!({
        "seed": t.seed_ms, "erode": t.erode_ms, "thermal": t.thermal_ms,
        "deposit": t.deposit_ms, "flood": t.flood_ms,
    })
}

/// `amplify_final=<cells> …` (the CLI's line).
pub fn summary_line(v: &Value) -> String {
    let f = |k: &str| v[k].as_f64().unwrap_or(0.0);
    let mut per_pass = BTreeMap::new();
    let mut add = |p: &Value| {
        for k in ["seed", "erode", "thermal", "deposit", "flood"] {
            *per_pass.entry(k).or_insert(0.0) += p[k].as_f64().unwrap_or(0.0);
        }
    };
    for l in v["levels"].as_array().into_iter().flatten() {
        add(&l["passes"]);
    }
    add(&v["last"]["passes"]);
    let pass = per_pass
        .iter()
        .map(|(k, ms)| format!("{k}={:.1}s", ms / 1000.0))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "amplify_final={}x{} mode={} tiles={} halo={} sinks={} pre_flood_raised_cells={} flood_raised_cells={} \
         flood_max_raise_m={:.2}@{} mean_abs_change_m={:.1} eroded_km3={:.3e} lost_in_transit_km3={:.3e} drama={} \
         passes[{}] sample={:.1}s detail={:.1}s sink={:.1}s total={:.1}s",
        v["grid"]["w"],
        v["grid"]["h"],
        v["mode"].as_str().unwrap_or("?"),
        v["tiles"],
        v["halo"],
        v["sinks"],
        v["pre_flood_raised_cells"],
        v["flood_raised_cells"],
        f("flood_max_raise_m"),
        v["flood_max_raise_at"],
        f("mean_abs_change_m"),
        f("eroded_km3"),
        f("lost_in_transit_km3"),
        v["drama"],
        pass,
        v["last"]["sample_ms"].as_f64().unwrap_or(0.0) / 1000.0,
        v["last"]["detail_ms"].as_f64().unwrap_or(0.0) / 1000.0,
        v["last"]["sink_ms"].as_f64().unwrap_or(0.0) / 1000.0,
        v["millis"]["total"].as_f64().unwrap_or(0.0) / 1000.0,
    )
}

impl Stage for AmplifyStage {
    fn name(&self) -> &str {
        STAGE_NAME
    }

    fn estimate(&self) -> StageEstimate {
        estimate(&self.preset)
    }

    fn total_steps(&self) -> u64 {
        1
    }

    fn steps_done(&self) -> u64 {
        u64::from(self.done)
    }

    fn step(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        if self.done {
            return Ok(());
        }
        ctx.pool.install(|| self.advance(world, ctx))
    }

    fn keyframe_layers(&self) -> &[&'static str] {
        &KEYFRAME_LAYERS
    }

    fn save_state(&self) -> Result<Vec<u8>, CoreError> {
        to_state(&Saved { done: self.done })
    }

    fn load_state(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        let s: Saved = from_state(bytes)?;
        self.done = s.done;
        Ok(())
    }
}

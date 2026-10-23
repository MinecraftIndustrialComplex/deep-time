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

pub mod character;
pub mod glacial;
#[cfg(feature = "gpu")]
pub mod gpu;
pub mod impact;
pub mod kernels;
pub mod resample;
pub mod structure;
pub mod tiles;
pub mod volcano;
pub mod window;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use deeptime_format::{Codec, DType, GridDesc, LayerEntry, Manifest};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use self::character::{CharacterCtx, CharacterFields, CharacterParams, RUGGEDNESS_MAX};
use self::kernels::{Counts, NoiseCfg, PassDevice, Physics, Times};
use self::resample::Level;
use self::tiles::{
    LEVEL_BYTES_PER_CELL, LastStats, LevelSink, PackSink, Preview, TILE_BYTES_PER_CELL, TileCfg,
    TilePlan, TileSink, halo as plan_halo, run_tiles,
};
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
    /// The youth the mountain character read (0 old … 1 active), L1.
    pub const YOUTH: &str = "amplify/youth";
    /// The fold strength the mountain character read (0 … 1), L1.
    pub const FOLD: &str = "amplify/fold";
}

/// Keyframe layers of the stage.
pub const KEYFRAME_LAYERS: [&str; 5] = [
    field::ELEVATION,
    field::DETAIL,
    field::DRAINAGE,
    field::YOUTH,
    field::FOLD,
];

/// Whether the last level streams.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamMode {
    /// In memory when the memory cap allows the whole path's full peak ([`whole_last_level_peak`]),
    /// else tile by tile (plan §5.7.5).
    Auto,
    Always,
    Never,
}

/// The sea floor's critical slope at drama 1 (`AmplifyParams::submarine_slope`): tan 15° (see the
/// field). ledger: AMP-074
pub const SUBMARINE_SLOPE: f64 = 0.268;

/// Bytes per cell of the thinned copy the last level keeps for the keyframes (three f32 arrays:
/// terrain, detail amplitude, drainage), allocated before the first tile runs.
pub const PREVIEW_BYTES_PER_CELL: u64 = 12;

/// The most the last level adds to what the run holds when the amplifier starts, if it runs on the
/// whole raster: every reservation it will make, with what it holds by then.
///
/// - the whole raster's buffers (`gw × gh` cells at [`TILE_BYTES_PER_CELL`]), reserved by
///   `run_tiles` when the last level starts;
/// - the level before it (`gw/2 × gh/2` cells), which the ladder keeps until the last level is done,
///   in place of the L2 level (`l2_bytes`, held now and dropped after the first ×2 level), when
///   there are levels before the last (`doublings` ≥ 2);
/// - the preview, allocated before that reservation.
///
/// The decision between memory and tiles is made against this, not against the buffers alone
/// (2026-09-30: with the buffers alone, a 1000 Myr `earthlike_quick_16k` run chose memory with
/// 1.64 GB held and then went over 4 GiB at the last level's reservation, where the ladder
/// holds 0.16 GB more). The levels before the last decide for themselves, with the gauge as it is
/// when they start, so they are not part of this.
pub fn whole_last_level_peak(
    l2_bytes: u64,
    doublings: u32,
    gw: usize,
    gh: usize,
    preview_width: u32,
) -> u64 {
    let buffers = (gw * gh) as u64 * TILE_BYTES_PER_CELL;
    let kept = if doublings >= 2 {
        ((gw / 2) * (gh / 2)) as u64 * LEVEL_BYTES_PER_CELL
    } else {
        l2_bytes
    }
    .saturating_sub(l2_bytes);
    let s = preview_stride(gw, preview_width as usize);
    let preview = (gw.div_ceil(s) * gh.div_ceil(s)) as u64 * PREVIEW_BYTES_PER_CELL;
    buffers + kept + preview
}

/// Whether a level runs in tiles: `always` and `never` as they say; `auto` when `reserve` refuses
/// `peak_bytes`, the most the whole-raster path would add to what the run holds (so that once it
/// runs whole, none of its later reservations fails; `reserve` may spill the recorded history to
/// make the room, as it does for any stage).
pub fn streams(
    mode: StreamMode,
    peak_bytes: u64,
    reserve: &dyn Fn(u64) -> Result<(), CoreError>,
) -> bool {
    match mode {
        StreamMode::Always => true,
        StreamMode::Never => false,
        StreamMode::Auto => reserve(peak_bytes).is_err(),
    }
}

/// [`streams`] for a level of `cells` whose middle passes run on `accel`: `auto` also tiles when
/// the whole level does not fit on the device, or leaves no room under the cap for the device's
/// share of the host's memory (on an integrated GPU, all of its buffers). Tiles give the terrain
/// the whole raster gives (`the_levels_before_the_last_agree_tiled_and_whole`), so the device
/// keeps the level instead of handing it to the CPU (2026-10-19: at the 4 GiB default cap a full
/// 16k last level ran whole on the CPU with the GPU forced on, `docs/design/gpu-compute.md` §7).
pub fn streams_on(
    mode: StreamMode,
    peak_bytes: u64,
    cells: usize,
    accel: Option<&dyn PassDevice>,
    reserve: &dyn Fn(u64) -> Result<(), CoreError>,
) -> bool {
    match accel {
        Some(d) if mode == StreamMode::Auto => {
            let share = cells as u64 * d.host_bytes_per_cell();
            !d.fits(cells) || streams(mode, peak_bytes.saturating_add(share), reserve)
        }
        _ => streams(mode, peak_bytes, reserve),
    }
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
    /// A cell lowers by at most this fraction of its drop to its receiver per iteration …
    pub max_incision_frac: f64,
    /// … and by at most this fraction of its initial drop to its lowest neighbour in all: the
    /// erosion is scaled by the cell size, a stream cuts a valley of a few cells' relief.
    ///
    /// The editor's "Terrain carving" (settings audit; Ben, 2026-09-29: configurable, default medium):
    /// subtle 0.5, **medium 3 (the default)**, dramatic 10. On the 100 Myr `pipeline_quick_16k`
    /// planet the eroded volume of the amplification is 0.39 / 0.56 / 0.69 / 0.84 / 0.92 / 0.97 /
    /// 0.98 / 1.00 of the uncapped run's at 0.25 / 0.5 / 1 / 3 / 5 / 8 / 12 / ∞, so about 10 and
    /// above is "no cap"; the flood then repairs the deeper slots (its largest raise 3 m at 3,
    /// 13 m at 12, 82 m uncapped, on a small steep island).
    pub incision_cap: f64,
    /// A cell stays at least this gradient above its receiver.
    pub min_gradient: f64,
    /// Critical slope at drama 1 (Schott: tan 30° ≈ 0.57), and the share of an excess a pair of
    /// cells shifts per iteration and the ± fraction its noise varies the slope by.
    pub thermal_slope: f64,
    pub thermal_rate: f64,
    pub thermal_noise: f64,
    /// Critical slope of the sea floor (every cell below sea level, except under a glacier's
    /// ice: a fjord keeps the land's) at drama 1, in place of `thermal_slope` (sea-floor spikes
    /// task, 2026-10-20): the L3 counterpart of the young stage's `sea_floor_slope_deg`, which
    /// relaxes the plate model's sea-floor steps on L2. The land's talus angle let what the
    /// interpolation steepened stand under the sea.
    pub submarine_slope: f64, // ledger: AMP-074
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
    /// Block-scale glacial valleys under the young landscape's glaciers (`glacial.rs`).
    pub glacial: glacial::GlacialShape,
    /// The editor's "Ruggedness" (0..2, default 1; decisions log 2026-09-29): how much block-scale
    /// mountain character the last level gets (`character.rs`); 0 leaves the passes' terrain.
    pub ruggedness: f64,
    pub character: CharacterParams,
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
            incision_cap: 3.0,
            min_gradient: 1.0e-4,
            thermal_slope: 0.6,
            thermal_rate: 0.05,
            thermal_noise: 0.25,
            submarine_slope: SUBMARINE_SLOPE,
            deposit_capacity_m: 4.0,
            deposit_fraction: 0.5,
            seed_relief: 0.25,
            sink_eps_m: 0.01,
            repair_margin_cells: 32,
            stream: StreamMode::Auto,
            tile_cells: 2048,
            preview_width: 2048,
            detail: DetailParams::default(),
            glacial: glacial::GlacialShape::default(),
            ruggedness: 1.0,
            character: CharacterParams::default(),
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
        if !(self.incision_cap > 0.0 && self.incision_cap <= 100.0) {
            return Err(bad("incision_cap", "must be in (0, 100]"));
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
        if !(self.submarine_slope.is_finite() && self.submarine_slope > 0.0) {
            return Err(bad("submarine_slope", "must be > 0"));
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
        self.glacial.validate("$.amplify.glacial")?;
        if !(self.ruggedness.is_finite() && (0.0..=RUGGEDNESS_MAX).contains(&self.ruggedness)) {
            return Err(bad("ruggedness", "must be in [0, 2]"));
        }
        self.character.validate("$.amplify.character")?;
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
    /// Sea level, m (the step's sea, measured from the plate model's datum), block y of the sea,
    /// metres per block, vertically.
    pub sea: f64,
    pub sea_y: f64,
    pub mv: f64,
    pub mh: f64,
    /// Blocks per cell of the last level (L3).
    pub l3_r: u32,
    /// Where every level's middle passes run (None: the CPU; `compute.rs`).
    pub accel: Option<Arc<dyn PassDevice>>,
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
            // The glacial trough pass runs on the last level only (`glacial.rs`).
            glacial: if coarse {
                0
            } else {
                self.p.glacial.radius() as u32
            },
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
            cap_frac: p.incision_cap as f32,
            min_gradient: p.min_gradient as f32,
            thermal_slope: thermal::critical_slope(p.thermal_slope, self.drama) as f32,
            thermal_rate: p.thermal_rate as f32,
            cap_ref_m: p.deposit_capacity_m as f32,
            cap_area_exp: p.area_exponent as f32,
            cap_slope_exp: 1.0,
            deposit_frac: p.deposit_fraction as f32,
            eps_m: p.sink_eps_m as f32,
            glacial: p.glacial,
        }
    }

    fn noise_cfg(&self, gw_fine: usize) -> NoiseCfg {
        let c = f64::from(self.circumference);
        NoiseCfg {
            noise: Noise::new(self.seed, self.circumference),
            seed_relief: self.p.seed_relief as f32,
            thermal_noise: self.p.thermal_noise as f32,
            submarine: (self.p.submarine_slope / self.p.thermal_slope) as f32,
            cell_blocks: c / gw_fine as f64,
            origin_x: -c / 2.0,
            origin_z: self.origin_z((c / gw_fine as f64).round() as u32),
        }
    }

    /// Block z of row 0 of a level of `r` blocks per cell (`GridDesc::for_planet`: −Zo).
    fn origin_z(&self, r: u32) -> f64 {
        deeptime_format::Projection::new(self.circumference).grid_origin_z(r) as f64
    }

    /// The halo the last level needs: the passes', and with ruggedness the character's radius and
    /// the reach of the flood that follows it.
    pub fn halo(&self) -> usize {
        let base = plan_halo(&self.counts(false), self.p.repair_margin_cells as usize);
        if self.p.ruggedness > 0.0 {
            base + character::halo_cells(
                &self.p.character,
                self.cell_blocks_l3(),
                self.p.repair_margin_cells as usize,
            )
        } else {
            base
        }
    }

    /// Blocks per cell of the last level.
    pub fn cell_blocks_l3(&self) -> f64 {
        f64::from(self.l3_r)
    }

    /// The character's context over the last level (None at ruggedness 0).
    pub fn character<'a>(&'a self, fields: &'a CharacterFields) -> Option<CharacterCtx<'a>> {
        (self.p.ruggedness > 0.0).then(|| {
            let c = f64::from(self.circumference);
            CharacterCtx {
                p: &self.p.character,
                ruggedness: self.p.ruggedness,
                fields,
                seed: self.seed,
                circumference: self.circumference,
                cell_blocks: self.cell_blocks_l3(),
                origin_x: -c / 2.0,
                origin_z: self.origin_z(self.l3_r),
                sea: self.sea as f32,
                sea_y: self.sea_y,
                mv: self.mv,
            }
        })
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
    /// One tile: the whole raster.
    pub tiles: usize,
    pub stats: LastStats,
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
/// The levels before it run on the whole raster, or in tiles of `p.tile_cells` when `p.stream` says
/// so (or, for `auto`, when `reserve(bytes)`, which is asked before every allocation of buffers of
/// that size, refuses the whole raster's).
#[allow(clippy::too_many_arguments)]
pub fn run(
    l2: Level,
    doublings: u32,
    cfg: &Config,
    character: Option<CharacterCtx<'_>>,
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
        let dxf = level.fine_dx();
        let counts = cfg.counts(true);
        // The whole raster's buffers, and the next level's arrays which are kept either way.
        let whole_bytes = (gw * gh) as u64 * (TILE_BYTES_PER_CELL + LEVEL_BYTES_PER_CELL);
        let tiled = streams_on(
            cfg.p.stream,
            whole_bytes,
            gw * gh,
            cfg.accel.as_deref(),
            reserve,
        ) && (cfg.p.tile_cells as usize) < gw.max(gh);
        let level_plan = if tiled {
            TilePlan {
                gw,
                gh,
                tile: cfg.p.tile_cells as usize,
                halo: plan_halo(&counts, cfg.p.repair_margin_cells as usize),
            }
        } else {
            TilePlan::whole(gw, gh)
        };
        let tcfg = TileCfg {
            ph: cfg.physics(dxf),
            counts,
            noise: cfg.noise_cfg(gw),
            detail: &cfg.p.detail,
            drama: cfg.drama,
            mv: cfg.mv,
            plan: level_plan,
            preview_stride: 1,
            last: false,
            character: None,
            accel: cfg.accel.as_deref(),
        };
        reserve((gw * gh) as u64 * LEVEL_BYTES_PER_CELL)?;
        let mut next = LevelSink::new(gw, gh, dxf, level.zs.len() == level.z.len());
        amp_probe("mid_sink_ready");
        let (_, stats) = run_tiles(&level, &tcfg, &mut next, cancelled, reserve)?;
        amp_probe("mid_done");
        level = next.level;
        amp_probe("mid_old_dropped");
        reports.push(LevelReport {
            w: gw,
            h: gh,
            tiles: stats.tiles,
            stats,
            millis: started.elapsed().as_secs_f64() * 1000.0,
        });
    }
    let (gw, gh) = (2 * level.w, 2 * level.h);
    debug_assert_eq!((plan.gw, plan.gh), (gw, gh));
    let dxf = level.fine_dx();
    let last = TileCfg {
        ph: cfg.physics(dxf),
        counts: cfg.counts(false),
        noise: cfg.noise_cfg(gw),
        detail: &cfg.p.detail,
        drama: cfg.drama,
        mv: cfg.mv,
        plan,
        preview_stride: preview_stride(gw, cfg.p.preview_width as usize),
        last: true,
        character,
        accel: cfg.accel.as_deref(),
    };
    amp_probe("last_start");
    let (preview, stats) = run_tiles(&level, &last, sink, cancelled, reserve)?;
    amp_probe("last_done");
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
    let z = get(young::field::ELEVATION_M)?;
    // The glaciers' surface where they were (a metre under the ground elsewhere), when the young
    // landscape had glaciers.
    let zs: Vec<f32> = match world
        .fields
        .get(young::field::ICE_SURFACE)
        .filter(|f| f.data.len() == n && f.data.iter().any(|x| x.is_finite()))
    {
        Some(f) => f
            .data
            .iter()
            .zip(z)
            .map(|(&s, &g)| if s.is_finite() && s > g { s } else { g - 1.0 })
            .collect(),
        None => Vec::new(),
    };
    Ok(Level {
        w: grid.w as usize,
        h: grid.h as usize,
        dx: f64::from(grid.r) * p.scale.m_per_block_h,
        z: z.clone(),
        area: get(young::field::AREA_M2)?.clone(),
        wgt: get(young::field::WEIGHT)?.clone(),
        kfac: get(young::field::ERODIBILITY)?.clone(),
        // A lake's surface: its floor plus the depth the young landscape's flood filled it to.
        lake_z: lake
            .iter()
            .zip(z)
            .map(|(&m, &g)| if m > 0.0 { g + m } else { resample::NO_LAKE })
            .collect(),
        zs,
    })
}

/// Bytes per L1 cell the stage holds beside the world from its start to its end: the mountain
/// character's fields (youth, the fold's two direction terms, the volcanoes' calm), the youth and
/// fold maps it puts in the world, and the volcanoes' flood-basalt outcrop: seven f32 arrays, and
/// the shapers' tables (amp-est, 2026-10-23: 25, 31 and 29 B a cell held on quick 16k at 50 and
/// 1000 Myr and on full 32k at 100 Myr).
const CHARACTER_BYTES_PER_L1_CELL: u64 = 32;

/// The cells of a preset's grid of `r` blocks per cell, `(w, h)` (`GridDesc::for_planet`; a grid
/// the format refuses counts as `C/r × C/2r`).
fn grid_dims(p: &PlanetPreset, r: u32) -> (u64, u64) {
    let c = p.scale.circumference_blocks;
    GridDesc::for_planet("L", c, r).map_or(
        (u64::from(c / r), u64::from(c / r / 2)),
        |g| (u64::from(g.w), u64::from(g.h)),
    )
}

/// The world the young landscape leaves for this stage, `(without the history, the history)`
/// (bytes): what the stages before it keep ([`super::retained_bytes`]) and the young landscape's
/// fields on the L2 grid. (amp-est, 2026-10-23: it was the surface stage's whole estimate, its
/// step's working arrays too, on a grid of `C/2r` rows where the Petroff-Guyou grids have `2·Zo/r`:
/// 1,099 MB listed against 665 MB held on quick 16k at 50 Myr.)
fn young_world(p: &PlanetPreset) -> (u64, u64) {
    let (w2, h2) = grid_dims(p, p.resolution.l2_r);
    (
        super::retained_bytes(p) + w2 * h2 * young::FIELD_BYTES_PER_L2_CELL,
        super::estimate(p).history_bytes,
    )
}

/// The world the run leaves for the bake, `(without the history, the history)` (bytes): the young
/// landscape's world and what this stage leaves in it (the youth and fold maps, the preview's
/// three layers), which is the finished run's memory once its levels and buffers are gone.
pub fn world_bytes(p: &PlanetPreset) -> (u64, u64) {
    let a = p.amplify.clone().unwrap_or_default();
    let (w1, h1) = grid_dims(p, p.resolution.l1_r);
    let (w3, h3) = grid_dims(p, p.resolution.l3_r);
    let s = preview_stride(w3 as usize, a.preview_width as usize) as u64;
    let (world, history) = young_world(p);
    (
        world + w1 * h1 * 8 + w3.div_ceil(s) * h3.div_ceil(s) * PREVIEW_BYTES_PER_CELL,
        history,
    )
}

/// TEMPORARY PROBE (amp-est lane, never committed).
pub fn amp_probe(tag: &str) {
    if std::env::var_os("DEEPTIME_AMP_PROBE").is_some() {
        eprintln!(
            "AMP_PROBE {tag} in_use={} kept={} wpeak={}",
            crate::alloc::in_use(),
            crate::alloc::kept_bytes(),
            crate::alloc::window_peak()
        );
    }
}

/// TEMPORARY PROBE (amp-est lane, never committed): prints, then starts a new window, so `wpeak` is
/// the segment's peak since the previous segment probe.
pub fn amp_probe_seg(tag: &str) {
    if std::env::var_os("DEEPTIME_AMP_PROBE").is_some() {
        amp_probe(tag);
        crate::alloc::start_window();
    }
}

/// Estimated bytes and seconds of the stage under the default memory cap
/// ([`crate::session::DEFAULT_MEM_CAP`]; [`estimate_under`]).
pub fn estimate(p: &PlanetPreset) -> StageEstimate {
    estimate_under(p, crate::session::DEFAULT_MEM_CAP)
}

/// The most the windows of a level of `w × h` cells cut up by `plan` hold at once (cells).
fn window_cells(plan: TilePlan) -> u64 {
    plan.cores()
        .into_iter()
        .map(|core| plan.window(core).0.cells() as u64)
        .max()
        .unwrap_or(0)
}

/// Estimated bytes and seconds of the stage under a memory cap of `mem_cap` bytes (plan §5.7.5;
/// P1-17 replaces the seconds): the most the run holds at any level, the world it starts with
/// included, on the preset's real grids (`GridDesc::for_planet`).
///
/// When it starts the run holds the young landscape's world ([`young_world`]), the character's
/// fields ([`CHARACTER_BYTES_PER_L1_CELL`]) and the L2 level. Each level before the last adds the
/// next level's arrays and the buffers of its windows, and hands on the next level in place of its
/// own; the last adds [`whole_last_level_peak`] whole, or its largest window and the preview in
/// tiles. Every level decides as the run does (`streams_on`, without a device): `auto` keeps it
/// whole when that fits under the cap with the world as estimated less the recorded history (which
/// `StageContext::reserve` spills first).
///
/// The world as estimated is at or above the world the run holds (amp-est, 2026-10-23: 140 to
/// 265 MB above it on quick 16k and full 32k), so a cap that much above the whole path's need is
/// one at which this lists a level in tiles that the run, seeing its real gauge, keeps whole. The
/// run then stays under its cap (its reservation is the whole path's), above this figure.
pub fn estimate_under(p: &PlanetPreset, mem_cap: u64) -> StageEstimate {
    let a = p.amplify.clone().unwrap_or_default();
    let res = &p.resolution;
    let levels = doublings(res.l2_r, res.l3_r)
        .filter(|&n| n >= 1)
        .unwrap_or(1);
    let cells = |(w, h): (u64, u64)| w * h;
    let (world, history) = young_world(p);
    let (w1, h1) = grid_dims(p, res.l1_r);
    let l2_bytes = cells(grid_dims(p, res.l2_r)) * LEVEL_BYTES_PER_CELL;
    let start = world + w1 * h1 * CHARACTER_BYTES_PER_L1_CELL + l2_bytes;
    let cfg = AmplifyStage::new(p.clone()).config(0.0);
    let tile = a.tile_cells as usize;
    // Whole when `auto` finds the room for `whole` more beside `held` (bytes without the history,
    // which gives way first).
    let whole_fits = |held: u64, whole: u64| match a.stream {
        StreamMode::Always => false,
        StreamMode::Never => true,
        StreamMode::Auto => held + whole <= mem_cap,
    };
    let (mut held, mut level, mut peak, mut mid_cells) = (start, l2_bytes, start, 0u64);
    for k in 1..levels {
        let (w, h) = grid_dims(p, res.l3_r << (levels - k));
        let next = w * h * LEVEL_BYTES_PER_CELL;
        let whole = w * h * TILE_BYTES_PER_CELL;
        let bufs = if whole_fits(held, whole + next) || tile >= w.max(h) as usize {
            whole
        } else {
            window_cells(TilePlan {
                gw: w as usize,
                gh: h as usize,
                tile,
                halo: plan_halo(&cfg.counts(true), a.repair_margin_cells as usize),
            }) * TILE_BYTES_PER_CELL
        };
        peak = peak.max(held + next + bufs);
        held = held - level + next;
        level = next;
        mid_cells += w * h;
    }
    let (w3, h3) = grid_dims(p, res.l3_r);
    let (gw, gh) = (w3 as usize, h3 as usize);
    // The last level decides when the stage starts, with the L2 level held.
    let whole = whole_last_level_peak(l2_bytes, levels, gw, gh, a.preview_width);
    let last = if whole_fits(start, whole) || tile >= gw.max(gh) {
        start + whole
    } else {
        let s = preview_stride(gw, a.preview_width as usize);
        let preview = (gw.div_ceil(s) * gh.div_ceil(s)) as u64 * PREVIEW_BYTES_PER_CELL;
        held + preview + window_cells(cfg.plan(gw, gh, true)) * TILE_BYTES_PER_CELL
    };
    StageEstimate {
        bytes: peak.max(last) + history,
        seconds: (w3 * h3) as f64 * 1.2e-6 + mid_cells as f64 * 3.0e-6,
        history_bytes: history,
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
            l3_r: self.preset.resolution.l3_r,
            accel: None,
        }
    }

    fn advance(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        let started = Instant::now();
        amp_probe("advance_start");
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
        let mut cfg = self.config(sea);
        cfg.accel = accel(ctx);
        let mut fields = CharacterFields::from_world(world, r.l1_r, r.l3_r, &self.p.character);
        // Volcanoes (P1-21) keep their cones: no fold ridges over them.
        let shaper = volcano::Shaper::from_world(world, &self.preset);
        if let Some(sh) = &shaper {
            let l1 = GridDesc::for_planet("L1", c, r.l1_r)?;
            fields.suppress_folds(&sh.fold_keep(&l1));
            fields.calm_volcanoes(&sh.youth_keep(&l1));
        }
        // The youth and fold strength the character read, for the keyframes and the workbench.
        if !fields.neutral {
            let l1 = GridDesc::for_planet("L1", c, r.l1_r)?;
            let (youth, fold) = fields.maps();
            for (name, data) in [(field::YOUTH, youth), (field::FOLD, fold)] {
                world.fields.insert(
                    name.to_owned(),
                    Field {
                        grid: l1.clone(),
                        data,
                    },
                );
            }
        }
        let mut level = level_from_world(world, &self.preset)?;
        amp_probe("after_level_from_world");
        // Volcanoes (P1-21): the cone put in place at L2 before the erosion passes, the rest at
        // L3 after the character pass (the sink below).
        let mut volcano_cells = 0;
        if let Some(sh) = &shaper {
            let g2 = GridDesc::for_planet("L2", c, r.l2_r)?;
            volcano_cells = sh.sharpen_level(&mut level, &g2);
        }
        // Impact craters (P2-08): what L1 does not hold of them, on L3 after the last flood (the
        // sink below).
        let craters = impact::Shaper::from_world(world, &self.preset);
        // Fault scarps, rift valleys and fold ridges (P2-01): the young structures' relief, put on
        // the young landscape before the erosion passes work on it.
        let relief_stats = match structure::Relief::from_world(world, &self.preset) {
            Some(relief) => {
                let g2 = GridDesc::for_planet("L2", c, r.l2_r)?;
                relief.apply(&mut level, &g2, sea)
            }
            None => structure::ReliefStats::default(),
        };
        let (gw, gh) = (l3.w as usize, l3.h as usize);
        amp_probe("after_relief");

        // Whole raster in memory, or tile by tile (plan §5.7.5: streams automatically when the
        // buffers would not fit under the memory cap).
        // `auto` is asked about the whole path's full peak (the last level's buffers, the level
        // before it which the ladder keeps, the preview), not the buffers alone: those are the
        // reservations the last level will make, with what the run holds by then.
        let whole_bytes = (gw * gh) as u64 * TILE_BYTES_PER_CELL;
        let peak_bytes = whole_last_level_peak(level.bytes(), n_dbl, gw, gh, self.p.preview_width);
        let streamed = streams_on(
            self.p.stream,
            peak_bytes,
            gw * gh,
            cfg.accel.as_deref(),
            &|b| ctx.reserve(b),
        );
        let plan = cfg.plan(gw, gh, streamed && self.p.tile_cells < (gw.max(gh)) as u32);
        ctx.log.push(
            crate::log::Level::Info,
            format!(
                "amplify: {} MiB in use of the {} MiB cap; the last level of {gw} x {gh} cells needs {} MiB whole ({} MiB at its fullest, with the level the ladder keeps and the preview) and runs {}",
                ctx.gauge.in_use() >> 20,
                ctx.mem_cap >> 20,
                whole_bytes >> 20,
                peak_bytes >> 20,
                if plan.tile >= gw.max(gh) { "in memory" } else { "in tiles" }
            ),
        );
        let reserve = |bytes: u64| ctx.reserve(bytes);
        let mut sink = PackSink::create(ctx.out_dir, &l3, 256, sea, cfg.sea_y, cfg.mv)?;
        // Nothing is stored above the build limit: the top band is compressed (`params::BuildFit`).
        sink.fit = Some(self.preset.scale.build_fit());
        amp_probe("after_packsink");
        let setup_ms = started.elapsed().as_secs_f64() * 1000.0;
        let cancelled = || ctx.cancelled();
        let character = cfg.character(&fields);
        // A tile passes through the craters' sink, then the volcanoes', then into the pack.
        let mut out = {
            let (mut shaped_v, mut shaped_c);
            let mut target: &mut dyn TileSink = &mut sink;
            if let Some(sh) = &shaper {
                shaped_v = volcano::Sink {
                    inner: target,
                    shaper: sh,
                    grid: l3.clone(),
                };
                target = &mut shaped_v;
            }
            if let Some(sh) = &craters {
                shaped_c = impact::Sink {
                    inner: target,
                    shaper: sh,
                    grid: l3.clone(),
                };
                target = &mut shaped_c;
            }
            run(
                level, n_dbl, &cfg, character, plan, target, &cancelled, &reserve,
            )?
        };
        amp_probe("after_run");
        let files = sink.finish()?;
        amp_probe("after_finish");
        let (pw, ps) = (out.preview.w, out.preview.stride);
        if let Some(sh) = &craters {
            sh.shape_block(&l3, 0, 0, pw, ps, &mut out.preview.z_m);
        }
        if let Some(sh) = &shaper {
            sh.shape_block(&l3, 0, 0, pw, ps, &mut out.preview.z_m);
        }

        // The thinned copies (keyframes, the workbench) …
        let s = out.preview.stride;
        let grid_p = l3.thinned("L3p", s as u32);
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
                    "w": l.w, "h": l.h, "millis": l.millis, "tiles": l.tiles,
                    "passes": timing_json(&l.stats.times),
                    "pre_flood_raised_cells": l.stats.pre_flood_raised_cells,
                    "flood_raised_cells": l.stats.flood_raised_cells,
                    "flood_max_raise_m": l.stats.flood_max_raise_m,
                    "sinks": l.stats.sinks,
                    "device_tiles": l.stats.device_tiles,
                })
            })
            .collect();
        // Where the middle passes ran (`compute.rs`), over every level's windows.
        let all = || out.levels.iter().map(|l| &l.stats).chain([t]);
        let compute = json!({
            "mode": ctx.compute.as_str(),
            "device": cfg.accel.as_ref().map_or("cpu".to_string(), |d| d.name()),
            "device_tiles": all().map(|s| s.device_tiles).sum::<u64>(),
            "tiles": all().map(|s| s.tiles as u64).sum::<u64>(),
            "error": all().find_map(|s| s.device_error.clone()),
        });
        let note = json!({
            "compute": compute,
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
            "ruggedness": self.p.ruggedness,
            "character": {
                "plate_fields": !fields.neutral,
                "land_cells": t.character.land_cells,
                "mean_abs_change_blocks": t.character.abs_change_blocks_cells / (t.character.land_cells.max(1) as f64),
                "max_raise_blocks": t.character.max_raise_blocks,
                "max_cut_blocks": t.character.max_cut_blocks,
                "flood_raised_cells": t.character_flood_raised_cells,
                "softened_cells": t.character_softened_cells,
                "flood_max_raise_m": t.character_flood_max_raise_m,
                "millis": t.character_ms,
            },
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
            "mean_detail_blocks": t.detail_blocks_cells / (t.land_cells.max(1) as f64),
            "eroded_km3": t.eroded_m_cells * km3,
            "glacial": {
                "carved_km3": t.glacial_carved_m_cells * km3,
                "raised_km3": t.glacial_raised_m_cells * km3,
                "cells": t.glacial_cells,
            },
            "lost_in_transit_km3": t.transit_m_cells * km3,
            "levels": levels,
            "last": {
                "sample_ms": t.sample_ms, "detail_ms": t.detail_ms, "sink_ms": t.sink_ms,
                "passes": timing_json(&t.times),
            },
            "millis": {"setup": setup_ms, "total": total_ms},
            "preview": {"w": out.preview.w, "h": out.preview.h, "stride": s},
            "layers": [layer(&files.elevation), layer(&files.amplitude)],
            "structure_relief": relief_stats,
            "volcanoes": {
                "shaped": shaper.as_ref().map_or(0, volcano::Shaper::len),
                "l2_cells": volcano_cells,
            },
            "craters": {
                "shaped": craters.as_ref().map_or(0, impact::Shaper::len),
            },
        });
        world.notes.insert(STAGE_NAME.into(), note.to_string());
        ctx.log.push(crate::log::Level::Info, summary_line(&note));
        self.done = true;
        Ok(())
    }
}

/// The device the middle passes run on under the session's compute mode (`compute.rs`).
#[cfg(feature = "gpu")]
fn accel(ctx: &StageContext<'_>) -> Option<Arc<dyn PassDevice>> {
    let gpu = crate::compute::device(ctx.compute, ctx.log, "amplify:middle")?;
    match gpu::AmplifyGpu::new(gpu) {
        Ok(a) => Some(Arc::new(a)),
        Err(e) => {
            ctx.log.push(
                crate::log::Level::Warn,
                format!("amplify: the GPU kernels did not build, running on the CPU ({e})"),
            );
            None
        }
    }
}

#[cfg(not(feature = "gpu"))]
fn accel(ctx: &StageContext<'_>) -> Option<Arc<dyn PassDevice>> {
    crate::compute::warn_without_gpu(ctx.compute, ctx.log);
    None
}

fn timing_json(t: &Times) -> Value {
    json!({
        "seed": t.seed_ms, "erode": t.erode_ms, "thermal": t.thermal_ms,
        "deposit": t.deposit_ms, "flood": t.flood_ms, "glacial": t.glacial_ms,
        "transfer": t.transfer_ms,
    })
}

/// `amplify_final=<cells> …` (the CLI's line).
pub fn summary_line(v: &Value) -> String {
    let f = |k: &str| v[k].as_f64().unwrap_or(0.0);
    let mut per_pass = BTreeMap::new();
    let mut add = |p: &Value| {
        for k in [
            "seed", "erode", "thermal", "deposit", "flood", "glacial", "transfer",
        ] {
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
    let level_tiles = v["levels"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|l| format!("{}x{}:{}", l["w"], l["h"], l["tiles"]))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "amplify_final={}x{} mode={} tiles={} halo={} mid_tiles=[{level_tiles}] sinks={} pre_flood_raised_cells={} flood_raised_cells={} \
         flood_max_raise_m={:.2}@{} mean_abs_change_m={:.1} mean_detail_blocks={:.2} eroded_km3={:.3e} lost_in_transit_km3={:.3e} \
         glacial_carved_km3={:.3e} glacial_raised_km3={:.3e} glacial_cells={} drama={} \
         ruggedness={} character[mean_abs_blocks={:.2},flood_raised_cells={},softened_cells={},{:.1}s] \
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
        f("mean_detail_blocks"),
        f("eroded_km3"),
        f("lost_in_transit_km3"),
        v["glacial"]["carved_km3"].as_f64().unwrap_or(0.0),
        v["glacial"]["raised_km3"].as_f64().unwrap_or(0.0),
        v["glacial"]["cells"],
        v["drama"],
        v["ruggedness"],
        v["character"]["mean_abs_change_blocks"]
            .as_f64()
            .unwrap_or(0.0),
        v["character"]["flood_raised_cells"],
        v["character"]["softened_cells"],
        v["character"]["millis"].as_f64().unwrap_or(0.0) / 1000.0,
        pass,
        v["last"]["sample_ms"].as_f64().unwrap_or(0.0) / 1000.0,
        v["last"]["detail_ms"].as_f64().unwrap_or(0.0) / 1000.0,
        v["last"]["sink_ms"].as_f64().unwrap_or(0.0) / 1000.0,
        v["millis"]["total"].as_f64().unwrap_or(0.0) / 1000.0,
    ) + &compute_term(&v["compute"])
}

/// ` compute=<device>:<windows on it>/<windows>` when a device ran any window, ` compute=cpu`
/// otherwise; the device's error, if it stopped using it, follows in brackets.
fn compute_term(c: &Value) -> String {
    let device = c["device"].as_str().unwrap_or("cpu");
    let mut s = if device == "cpu" {
        " compute=cpu".to_string()
    } else {
        format!(
            " compute={}:{}/{}",
            device
                .split(" (")
                .next()
                .unwrap_or(device)
                .replace(' ', "_"),
            c["device_tiles"],
            c["tiles"]
        )
    };
    if let Some(e) = c["error"].as_str() {
        s += &format!("[fell_back: {e}]");
    }
    s
}

impl Stage for AmplifyStage {
    fn name(&self) -> &str {
        STAGE_NAME
    }

    fn estimate(&self) -> StageEstimate {
        estimate(&self.preset)
    }

    fn estimate_under(&self, mem_cap: u64) -> StageEstimate {
        estimate_under(&self.preset, mem_cap)
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

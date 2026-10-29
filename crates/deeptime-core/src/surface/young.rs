// SPDX-License-Identifier: LGPL-3.0-or-later
//! The young landscape on L2 (plan §8.2, work item P1-06): the last few Myr of erosion at twice
//! the resolution of the deep-time surface, starting from L1 upsampled.
//!
//! The deep-time run ends on L1 (4.8 km cells at 16k quick). This stage upsamples that surface
//! to L2 (`resolution.l2_r`; clamped Catmull–Rom, `amplify/resample.rs`), fills the depressions
//! the interpolation made (those shallower than `min_lake_depth_m`; deeper ones are L1's lakes)
//! and runs `steps` steps of the same physics as the deep-time surface (`surface/mod.rs`):
//! priority-flood, D8 routing with runoff-weighted areas, implicit stream power with deposition
//! and a critical slope (`spl.rs`), then hillslope and marine diffusion. The drainage network
//! that comes out — exact on L2, with the lakes the flood found — is what the amplifier refines.
//!
//! **Time.** The stage adds `myr` million years of erosion at the end of the history without
//! advancing the world clock: it is the L2 refinement of the last steps, not more history
//! (STATUS.md, P1-06). **Uplift** keeps the relief the deep-time run left from simply eroding
//! away during it: `balance` (default) uplifts each cell by the erosion rate of the last
//! deep-time step (`surface/erosion_m_myr`), i.e. it assumes the landscape was in balance;
//! `plates` uses the plate model's uplift (`plates/uplift_mm_yr`); `none` is pure decay.
//! **Steps** are as long as the deep-time surface's (`YoungParams::steps_for`; one on the presets,
//! whose surface steps 2 Myr): the balance uplift is the rate that step gave and K was
//! calibrated with it, and 0.5 Myr steps on a quick 16k planet left land eroding 200 m/Myr or
//! more 15 blocks under the history's surface on average, 8 with the history's step (YNG-002).
//!
//! Outputs (fields on L2): `young/elevation_m`, `young/area_m2` (effective drainage area),
//! `young/lake_m` (depth of a lake or closed basin, else 0), `young/runoff_weight`,
//! `young/erodibility`, plus `young/class`, `young/uplift_m_myr`, and for the keyframes
//! `young/elevation` (block y) and `young/drainage` (log₁₀ km²).

use std::collections::BTreeMap;
use std::time::Instant;

use deeptime_format::GridDesc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::amplify::{doublings, resample};
use super::grid::Dims;
use super::pflood::{BASE, LAKE_TOL_M, LAND, SINK};
use super::{SurfaceParams, climate, diffusion, glacial, lithology, marine, pflood, routing, spl};
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::stage::{Stage, StageContext, StageEstimate, from_state, to_state};
use crate::world::{Field, WorldState};

/// The stage's name in a pipeline (`deeptime stage young`).
pub const STAGE_NAME: &str = "young";

/// Fields the stage writes.
pub mod field {
    pub const ELEVATION_M: &str = "young/elevation_m";
    pub const AREA_M2: &str = "young/area_m2";
    pub const LAKE_M: &str = "young/lake_m";
    pub const WEIGHT: &str = "young/runoff_weight";
    pub const ERODIBILITY: &str = "young/erodibility";
    pub const CLASS: &str = "young/class";
    pub const UPLIFT: &str = "young/uplift_m_myr";
    /// Block y (keyframes).
    pub const ELEVATION: &str = "young/elevation";
    /// log₁₀ of the drainage area, km² (NaN at sea; keyframes).
    pub const DRAINAGE: &str = "young/drainage";
    /// The glaciers' climate on L2 (`glacial.rs`): the glacial state's ELA above sea level, the
    /// sea-level temperature, precipitation, and where the climate allows ice (when it says).
    pub const GLACIAL_ELA: &str = "young/glacial_ela_m";
    pub const T_SEA_LEVEL: &str = "young/t_sea_level_c";
    pub const PRECIP: &str = "young/precip_mm";
    pub const ICE_ALLOWED: &str = "young/ice_allowed";
    /// The glaciers of the last step: centre-line thickness (m) and surface (m; NaN without
    /// ice), what the amplifier shapes valleys under (`amplify/glacial.rs`).
    pub const ICE: &str = "young/ice_thickness_m";
    pub const ICE_SURFACE: &str = "young/ice_surface_m";
    /// Glacial erosion and till over the stage, m.
    pub const GLACIAL_EROSION: &str = "young/glacial_erosion_m";
    pub const TILL: &str = "young/till_m";
}

/// Bytes per L2 cell of the fields the stage leaves in the world: the 17 of [`field`], f32 each
/// (what the amplifier's estimate counts the world it starts with by, `amplify::estimate_under`).
pub const FIELD_BYTES_PER_L2_CELL: u64 = 17 * 4;

/// Keyframe layers of the stage.
pub const KEYFRAME_LAYERS: [&str; 3] = [field::ELEVATION, field::DRAINAGE, field::ICE];

/// Till at least this thick makes a cell's surface rock till, m.
const TILL_SHOWS_M: f32 = 5.0;

/// The sea floor's critical slope, degrees ([`YoungParams::sea_floor_slope_deg`]). ledger: YNG-022
pub const SEA_FLOOR_SLOPE_DEG: f64 = 15.0;

/// Iterations of the sea floor's failure (`marine::slope_failure` at its rate 0.05): numerical.
/// A 3 km step one L2 cell wide, a 5 km one-cell trench and a maze of 2.8 km blocks two cells
/// across relax to within a degree of the critical slope in 100 to 200
/// (tools/review/seafloor/relax.py, the rule in numpy on 2.4 km cells). ledger: YNG-023
pub const SEA_FLOOR_FAILURE_ITERATIONS: u32 = 300;

/// How the young landscape is uplifted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpliftMode {
    /// The erosion rate of the last deep-time step (a landscape in balance).
    Balance,
    /// The plate model's uplift rate.
    Plates,
    /// None: the relief decays.
    None,
}

/// The `young` section (every key optional).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct YoungParams {
    /// Erosion time added on L2, Myr (plan §8.2: the last 2–10).
    pub myr: f64,
    /// LEM steps; unset (the default), steps as long as the deep-time surface's
    /// ([`YoungParams::steps_for`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u32>,
    pub uplift: UpliftMode,
    /// Multiplier on the uplift rate.
    pub uplift_factor: f64,
    /// Depressions of the upsampled surface shallower than this are filled, m.
    pub min_lake_depth_m: f64,
    /// Multiplier on the surface section's K (finer cells may want a different erodibility).
    pub k_scale: f64,
    /// The steepest slope the sea floor holds, degrees: before the steps, open-sea cells
    /// steeper than this fail (`marine::slope_failure` with nothing limiting what a cell gives:
    /// rock as well as sediment), the matter kept and the failed rock building the slope's toe.
    /// 0 turns it off. Sea-floor spikes task (2026-10-20): the plate model draws the sea floor's
    /// steps one sample or one L1 cell wide (a trench pulled toward 10 km, a strip where plates
    /// and crusts interleave), 2 to 5 km tall over 5 to 9 km on this small planet, and nothing
    /// under the wave base relaxed them (the marine diffusion vanishes there, the sediment
    /// failure moves sediment only, the amplifier's thermal pass held them at the land's talus
    /// angle), so they stood as 30 to 45 degree walls, 80 degrees and more in the game.
    pub sea_floor_slope_deg: f64, // ledger: YNG-022
}

impl Default for YoungParams {
    fn default() -> Self {
        YoungParams {
            myr: 2.0,
            steps: None,
            uplift: UpliftMode::Balance,
            uplift_factor: 1.0,
            min_lake_depth_m: 5.0,
            k_scale: 1.0,
            sea_floor_slope_deg: SEA_FLOOR_SLOPE_DEG,
        }
    }
}

impl YoungParams {
    pub fn validate(&self) -> Result<(), CoreError> {
        let bad = |k: &str, why: &str| CoreError::preset(format!("$.young.{k}"), why);
        if !(self.myr.is_finite() && self.myr > 0.0 && self.myr <= 50.0) {
            return Err(bad("myr", "must be in (0, 50]"));
        }
        if self.steps.is_some_and(|s| !(1..=100).contains(&s)) {
            return Err(bad("steps", "must be in [1, 100]"));
        }
        if !(self.uplift_factor.is_finite() && (0.0..=10.0).contains(&self.uplift_factor)) {
            return Err(bad("uplift_factor", "must be in [0, 10]"));
        }
        if !(self.min_lake_depth_m.is_finite() && self.min_lake_depth_m >= 0.0) {
            return Err(bad("min_lake_depth_m", "must be ≥ 0"));
        }
        if !(self.k_scale.is_finite() && self.k_scale > 0.0) {
            return Err(bad("k_scale", "must be > 0"));
        }
        if !(self.sea_floor_slope_deg.is_finite()
            && (0.0..60.0).contains(&self.sea_floor_slope_deg))
        {
            return Err(bad("sea_floor_slope_deg", "must be in [0, 60)"));
        }
        Ok(())
    }

    /// The LEM steps the stage takes: `steps` when set, else `myr` in steps as long as the
    /// deep-time surface's (the plate step over `surface.substeps`), at least one. The `balance`
    /// uplift is the erosion rate the surface's implicit stream power gave over its own step, and
    /// K was calibrated with that step (ERO-004, ERO-179, ERO-180); on the same ground a shorter
    /// step erodes fast-eroding land faster (n = 2), so shorter steps lowered it under the uplift
    /// meant to hold it (YNG-002).
    pub fn steps_for(&self, preset: &PlanetPreset) -> u32 {
        if let Some(s) = self.steps {
            return s;
        }
        let plate = match preset.dt_myr() {
            dt if dt.is_finite() && dt > 0.0 => dt,
            _ => preset.tectonics.time_step_myr,
        };
        let substeps = preset.surface.as_ref().map_or(1, |s| s.substeps.max(1));
        let lem = plate / f64::from(substeps);
        (self.myr / lem).round().clamp(1.0, 100.0) as u32
    }
}

/// The young stage's own peak per L2 cell, bytes: its fields, the LEM's working arrays (routing,
/// stream power, diffusion and the slope failure), and the glaciers' ice state and passes
/// (`glacial.rs`). Measured on the real L2 grids (P1-17, 2026-10-05: the counted peak less what
/// the run held when the stage started, 413 B a cell on quick 16k and 416 B on full 16k,
/// scattered continents, 500 Myr, the kept-block cache off), with about 10 % to spare. P1-19
/// (2026-10-19): the multigrid diffusion solve's hierarchy beside the conjugate gradients' arrays
/// (`diffusion/multigrid.rs`: the finest level's seven f32 arrays, 28 B, and the coarser levels'
/// eight on a third as many cells, 11 B) raised the stage's counted peak on quick 16k from
/// 1,322.8 to 1,376.5 MiB (master against the branch, the same run and trace otherwise), 42 B a
/// cell, so its own arrays are 448 B a cell (445 B on full 16k); 460 left
/// `tests/memory_estimate.rs` 0.1 MiB short (74.4 MiB held against 74.3). 500 kept about 10 %
/// to spare again. 2026-10-19, the step's arrays freed as soon as it stops reading them
/// (`YoungStage::lem`): 275 B a cell on full 16k (the stage alone from a 100 Myr checkpoint,
/// 2,293 → 1,408 MiB above the world it started with) and about 285 B on the test's small planet
/// (74.4 → 60.3 MiB held), so 320, 12 % above the larger. 2026-10-20: the stage's arrays follow
/// the terrain (the lakes and the routing's queues), and on the test's small planet it held
/// 62.3 MiB on master d9bd2b65 and 63.1 with PLT-294 on (the presets' transition of ocean crust),
/// a hair over the 63.1 estimated, while the world it started with was the same (structure
/// 47.9 → 47.7 MiB); 340 lists 64.7 MiB there, and is about the most the quick 16k bound of
/// `the_estimate_covers_the_measured_hold_closely` allows (1,086 MiB listed, at most 15 % over
/// the 947 held on 2026-10-19). A per-cell constant cannot follow both closely: the small
/// planet's own arrays take about 320 B a cell, full 16k's 275.
pub const OWN_BYTES_PER_L2_CELL: u64 = 340;

/// Estimate: the world the surface and the stages after it leave ([`super::retained_bytes`]),
/// the recorded history, and the stage's own arrays on the L2 grid; ≈ 1 µs per cell per step
/// (`spl`, flood, routing and diffusion, plan §8.2's cost model).
///
/// P1-17 (2026-10-05): it was the surface stage's whole estimate (the step's working arrays, the
/// advected copy of the strata log, the glaciers' passes, all freed by the time this stage runs)
/// plus 352 B for each cell of a 2:1 grid (the Lambert L2 grid has 1/π as many rows as columns,
/// not 1/2): 6,049 MiB on full 16k against the 4,143 MiB the stage held (the world 2,010 MiB, its
/// own arrays 2,133 MiB, the history besides), so the editor refused full at a 5 GiB cap it fits
/// under. Now 4,485 MiB without the history (quick: 1,240 MiB against 1,105 MiB held); 3,769 MiB
/// since 2026-10-19 (quick 1,060), against 3,462 and 947 MiB held; 3,872 and 1,086 MiB since
/// 2026-10-20 (20 B a cell more, on 4096 × 1312 and 2048 × 656 cells).
pub fn estimate(p: &PlanetPreset) -> StageEstimate {
    let y = p.young.clone().unwrap_or_default();
    let c = u64::from(p.scale.circumference_blocks);
    let r = u64::from(p.resolution.l2_r);
    let cells = GridDesc::for_planet("L2", p.scale.circumference_blocks, p.resolution.l2_r)
        .map(|g| g.cells() as u64)
        .unwrap_or((c / r) * (c / r / 2));
    let surface = super::estimate(p);
    StageEstimate {
        bytes: super::retained_bytes(p) + surface.history_bytes + cells * OWN_BYTES_PER_L2_CELL,
        seconds: cells as f64 * f64::from(y.steps_for(p)) * 1.0e-6,
        history_bytes: surface.history_bytes,
    }
}

#[derive(Serialize, Deserialize)]
struct Saved {
    done: u64,
}

/// The young-landscape stage.
pub struct YoungStage {
    preset: PlanetPreset,
    p: YoungParams,
    sp: SurfaceParams,
    /// [`YoungParams::steps_for`].
    steps: u32,
    done: u64,
    timings: BTreeMap<&'static str, f64>,
}

/// A ×2 upsampling of a plain array (`amplify::resample`).
type Upsample = dyn Fn(&[f32], usize, usize) -> Vec<f32>;

/// A rock property (`table`, by class: erodibility, diffusivity) on the `w × h` grid `k`
/// doublings finer than the grid of `class1`: the L1 cells' values interpolated linearly, as
/// every other field the stage takes from L1 is.
///
/// P1-23. The stage read the property through `young/class`, which is L1's rock class copied
/// into each of the finer cells (a class cannot be interpolated). The rivers' K, the hillslopes'
/// κ and the erodibility the amplifier and the mountain character read were therefore constant
/// over 16 × 16 blocks and stepped at the L1 cells' edges, and the terrain came out with square
/// pits, benches and cliff bands the size of an L1 cell wherever the rock changes from cell to
/// cell. The class stays what it was; only what the erosion reads of it is interpolated.
pub fn rock_property(class1: &[f32], w1: usize, h1: usize, k: u32, table: &[f64]) -> Vec<f32> {
    let of = |c: f32| table[(c.max(0.0) as usize).min(table.len() - 1)] as f32;
    let (mut cur, mut w, mut h): (Vec<f32>, usize, usize) =
        (class1.iter().map(|&c| of(c)).collect(), w1, h1);
    for _ in 0..k {
        cur = resample::upsample2_linear(&cur, w, h);
        w *= 2;
        h *= 2;
    }
    cur
}

/// The sea floor's failure ([`YoungParams::sea_floor_slope_deg`]): open-sea cells (`open`, below
/// `sea`) of `z` steeper than `slope_deg` to an open-sea neighbour give the excess down the slope
/// (`marine::slope_failure`, every cell free to give any amount: rock fails as well as sediment),
/// so a wall the plate model drew one cell wide becomes a ramp at that slope with its toe built of
/// what fell. Matter is kept. Returns the volume moved, km³.
pub fn fail_sea_floor(d: &Dims, open: &[bool], sea: f64, slope_deg: f64, z: &mut [f32]) -> f64 {
    let mut h: Vec<f64> = z.iter().map(|&v| f64::from(v)).collect();
    let mut avail = vec![1.0e9; z.len()];
    let fp = marine::FailureParams {
        critical_slope: slope_deg.to_radians().tan(),
        rate: marine::FAILURE_RATE,
        iterations: SEA_FLOOR_FAILURE_ITERATIONS,
    };
    let moved = marine::slope_failure(d, open, sea, &fp, &mut h, &mut avail);
    for (v, &x) in z.iter_mut().zip(&h) {
        *v = x as f32;
    }
    moved * d.cell_area() / 1.0e9
}

/// Land elevations above sea: (median, 90th percentile, maximum), m.
fn land_stats(z: &[f32], sea: f64) -> (f64, f64, f64) {
    let mut v: Vec<f32> = z.iter().copied().filter(|&x| f64::from(x) > sea).collect();
    if v.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    v.sort_by(f32::total_cmp);
    let q = |f: f64| f64::from(v[((v.len() - 1) as f64 * f).round() as usize]) - sea;
    (q(0.5), q(0.9), q(1.0))
}

impl YoungStage {
    pub fn new(preset: PlanetPreset) -> YoungStage {
        let p = preset.young.clone().unwrap_or_default();
        let sp = preset.surface.clone().unwrap_or_default();
        let steps = p.steps_for(&preset);
        YoungStage {
            preset,
            p,
            sp,
            steps,
            done: 0,
            timings: BTreeMap::new(),
        }
    }

    fn total(&self) -> u64 {
        1 + u64::from(self.steps)
    }

    fn l2(&self) -> Result<GridDesc, CoreError> {
        Ok(GridDesc::for_planet(
            "L2",
            self.preset.scale.circumference_blocks,
            self.preset.resolution.l2_r,
        )?)
    }

    fn time(&mut self, what: &'static str, since: Instant) {
        *self.timings.entry(what).or_insert(0.0) += since.elapsed().as_secs_f64() * 1000.0;
    }

    /// Step 0: L1 → L2 and the fields the LEM steps read.
    fn setup(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        let started = Instant::now();
        let st = world.surface.as_ref().ok_or_else(|| {
            CoreError::InvalidState("the young stage runs after the surface stage".into())
        })?;
        let g1 = st.grid.clone();
        let g2 = self.l2()?;
        let k = doublings(g1.r, g2.r).ok_or_else(|| {
            CoreError::preset(
                "$.resolution.l2_r",
                format!("l1_r {} must be a power of two times l2_r {}", g1.r, g2.r),
            )
        })?;
        let n2 = g2.cells();
        ctx.reserve(n2 as u64 * 12 * 8)?;
        let (w1, h1) = (g1.w as usize, g1.h as usize);
        let sea = st.sea_level_m;
        let up = |v: &[f32], f: &Upsample| -> Vec<f32> {
            let (mut cur, mut w, mut h) = (v.to_vec(), w1, h1);
            for _ in 0..k {
                cur = f(&cur, w, h);
                w *= 2;
                h *= 2;
            }
            cur
        };
        let field1 = |name: &str| -> Option<&Vec<f32>> {
            world
                .fields
                .get(name)
                .filter(|f| f.data.len() == g1.cells())
                .map(|f| &f.data)
        };
        let mut z2 = up(&st.elevation_m, &resample::upsample2_z);
        // Class, weight and uplift at L1, then up.
        let class1: Vec<f32> = field1(super::field::CLASS)
            .cloned()
            .unwrap_or_else(|| vec![0.0; g1.cells()]);
        let class2 = up(&class1, &|v, w, h| resample::upsample2_nearest(v, w, h));
        let weight1: Vec<f32> = match (field1(climate::T_FIELD), field1(climate::P_FIELD)) {
            (Some(t), Some(p)) => (0..g1.cells())
                .map(|c| {
                    let (tc, pc) = (f64::from(t[c]), f64::from(p[c]));
                    let e = (60.0 * tc).max(0.0);
                    let runoff = (pc - climate::fu_et(pc, e)).max(0.0) / 1000.0;
                    (runoff / self.sp.runoff_ref_m_yr).max(0.05) as f32
                })
                .collect(),
            _ => vec![1.0; g1.cells()],
        };
        let weight2 = up(&weight1, &resample::upsample2_linear);
        let uplift1: Vec<f32> = match self.p.uplift {
            UpliftMode::Balance => field1(super::field::EROSION)
                .cloned()
                .unwrap_or_else(|| vec![0.0; g1.cells()]),
            UpliftMode::Plates => field1("plates/uplift_mm_yr")
                .map(|v| v.iter().map(|&x| x * 1000.0).collect())
                .unwrap_or_else(|| vec![0.0; g1.cells()]),
            UpliftMode::None => vec![0.0; g1.cells()],
        };
        let factor = self.p.uplift_factor as f32;
        let uplift2: Vec<f32> = up(&uplift1, &resample::upsample2_linear)
            .into_iter()
            .map(|u| u.max(0.0) * factor)
            .collect();
        // The glaciers' climate (`glacial.rs`): the glacial state's ELA (the climate's, else the
        // present one lowered by `young_ela_drop_m`), the temperature at sea level (so the lapse
        // rate applies on L2's own ground) and the precipitation.
        let gp = &self.sp.glacial;
        let glacial_fields: Option<[Vec<f32>; 3]> = if gp.active() {
            let ela1: Option<Vec<f32>> = match field1(glacial::source::ELA_GLACIAL) {
                Some(e) => Some(e.clone()),
                None => field1(glacial::source::ELA)
                    .map(|e| e.iter().map(|&x| x - gp.young_ela_drop_m as f32).collect()),
            };
            match (ela1, field1(climate::T_FIELD), field1(climate::P_FIELD)) {
                (Some(ela1), Some(t1), Some(p1)) => {
                    let t_sl: Vec<f32> = (0..g1.cells())
                        .map(|c| {
                            let z = (f64::from(st.elevation_m[c]) - sea).max(0.0);
                            (f64::from(t1[c]) + climate::LAPSE_C_PER_M * z) as f32
                        })
                        .collect();
                    Some([
                        up(&ela1, &resample::upsample2_linear),
                        up(&t_sl, &resample::upsample2_linear),
                        up(p1, &resample::upsample2_linear),
                    ])
                }
                _ => None,
            }
        } else {
            None
        };
        let allowed2: Option<Vec<f32>> = field1(glacial::source::ICE_MAX_EXTENT)
            .map(|m| up(m, &|v, w, h| resample::upsample2_nearest(v, w, h)));
        // Depressions the interpolation made (not L1's lakes) are filled.
        let d = Dims::new(
            g2.w as usize,
            g2.h as usize,
            f64::from(g2.r) * self.preset.scale.m_per_block_h,
            true,
        );
        let h: Vec<f64> = z2.iter().map(|&z| f64::from(z)).collect();
        let mut kind = vec![LAND; n2];
        let fm = self.sp.numerics.flood;
        pflood::open_sea_with(fm, &d, &h, sea, (n2 / 500).max(1), &mut kind);
        let base = vec![sea; n2];
        let mut filled = vec![0.0; n2];
        pflood::flood_with(fm, &d, &h, &base, &mut kind, &mut filled);
        let mut small = 0usize;
        for c in 0..n2 {
            let depth = filled[c] - h[c];
            if kind[c] != BASE && depth > 0.0 && depth < self.p.min_lake_depth_m {
                z2[c] = filled[c] as f32;
                small += 1;
            }
        }
        drop(filled);
        drop(base);
        // The sea floor fails where the plate model left it steeper than it holds.
        let sea_floor_km3 = if self.p.sea_floor_slope_deg > 0.0 {
            ctx.reserve(n2 as u64 * 26)?;
            let open: Vec<bool> = kind.iter().map(|&k| k == BASE).collect();
            fail_sea_floor(&d, &open, sea, self.p.sea_floor_slope_deg, &mut z2)
        } else {
            0.0
        };
        drop(h);
        drop(kind);
        let (p50, p90, max) = land_stats(&z2, sea);
        let put = |world: &mut WorldState, name: &str, data: Vec<f32>| {
            world.fields.insert(
                name.to_owned(),
                Field {
                    grid: g2.clone(),
                    data,
                },
            );
        };
        put(world, field::ELEVATION_M, z2);
        put(world, field::CLASS, class2);
        put(world, field::WEIGHT, weight2);
        put(world, field::UPLIFT, uplift2);
        for name in [
            field::GLACIAL_ELA,
            field::T_SEA_LEVEL,
            field::PRECIP,
            field::ICE_ALLOWED,
            field::GLACIAL_EROSION,
            field::TILL,
            field::ICE_SURFACE,
        ] {
            world.fields.remove(name);
        }
        let glaciated = glacial_fields.is_some();
        if let Some([ela, t_sl, pr]) = glacial_fields {
            put(world, field::GLACIAL_ELA, ela);
            put(world, field::T_SEA_LEVEL, t_sl);
            put(world, field::PRECIP, pr);
            if let Some(a) = allowed2 {
                put(world, field::ICE_ALLOWED, a);
            }
            put(world, field::GLACIAL_EROSION, vec![0.0; n2]);
            put(world, field::TILL, vec![0.0; n2]);
        }
        put(world, field::ICE, vec![0.0; n2]);
        world.notes.insert(
            STAGE_NAME.into(),
            json!({
                "l2": {"w": g2.w, "h": g2.h, "r": g2.r},
                "steps": self.steps, "myr": self.p.myr,
                "uplift": self.p.uplift,
                "filled_small_depressions": small,
                "sea_floor_failed_km3": sea_floor_km3,
                "glaciers": glaciated,
                "before": {"land_z_p50_m": p50, "land_z_p90_m": p90, "land_z_max_m": max},
            })
            .to_string(),
        );
        self.time("setup", started);
        Ok(())
    }

    /// One LEM step on L2 (the same physics as `SurfaceStage::advance` step 3, without the strata
    /// log, the plates or the climate stage's cadence).
    ///
    /// **Memory** (2026-10-19). A step was the run's counted peak on full 16k (4,096 × 1,312 L2
    /// cells: a byte a cell is 5.1 MiB), so each working array lives only while the step still
    /// reads it. The step reads its inputs in the world instead of copying them; drops at its start
    /// the outputs it writes again; lets the stream power's arrays go after its solve, and the
    /// routing, the fill, the glaciers' climate and their ice state after the glaciers' step (the
    /// last step routes the surface again at its end, as before); folds the glaciers' erosion and
    /// till into their fields before the diffusion solve; and writes the drainage area, the lakes
    /// and the erodibility, which only the stages after it read, on its last step only. The
    /// arithmetic is unchanged: from one full 16k checkpoint the stage's 17 fields come out the
    /// same to the bit. Its own peak there fell from 2,293 to 1,408 MiB (447 to 275 B a cell),
    /// and moved from the diffusion solve to the glaciers' step.
    fn lem(
        &mut self,
        world: &mut WorldState,
        ctx: &StageContext<'_>,
        last: bool,
    ) -> Result<(), CoreError> {
        /// A field of the step's grid (`n` values), else nothing.
        fn read<'a>(world: &'a WorldState, name: &str, n: usize) -> &'a [f32] {
            world
                .fields
                .get(name)
                .filter(|f| f.data.len() == n)
                .map_or(&[], |f| f.data.as_slice())
        }
        let g2 = self.l2()?;
        let n = g2.cells();
        ctx.reserve(n as u64 * 16 * 8)?;
        let (w, hgt) = (g2.w as usize, g2.h as usize);
        let dx = f64::from(g2.r) * self.preset.scale.m_per_block_h;
        let d = Dims::new(w, hgt, dx, true);
        let sea = world
            .surface
            .as_ref()
            .map(|s| s.sea_level_m)
            .ok_or_else(|| CoreError::InvalidState("the young stage needs the surface".into()))?;
        let has = |world: &WorldState, name: &str| {
            world.fields.get(name).is_some_and(|f| f.data.len() == n)
        };
        for name in [
            field::ELEVATION_M,
            field::CLASS,
            field::WEIGHT,
            field::UPLIFT,
        ] {
            if !has(world, name) {
                return Err(CoreError::InvalidState(format!(
                    "the young stage lost {name}"
                )));
            }
        }
        // The glaciers of the glacial state (`glacial.rs`), when the setup found their climate.
        let glaciated = [field::GLACIAL_ELA, field::T_SEA_LEVEL, field::PRECIP]
            .iter()
            .all(|name| has(world, name));
        // What the step writes again at its end goes now: the step does not read it.
        for name in [
            field::ELEVATION,
            field::DRAINAGE,
            field::AREA_M2,
            field::LAKE_M,
            field::ERODIBILITY,
            field::ICE,
        ] {
            world.fields.remove(name);
        }
        if glaciated {
            world.fields.remove(field::ICE_SURFACE);
        }
        // The glaciers' erosion and till so far, which the step adds to (back as they were
        // without glaciers).
        let mut take = |name: &str| -> Option<Vec<f32>> {
            if has(world, name) {
                world.fields.remove(name).map(|f| f.data)
            } else {
                None
            }
        };
        let mut ero_prev = take(field::GLACIAL_EROSION);
        let mut till = take(field::TILL);

        let sp = self.sp.clone();
        let k_scale = self.p.k_scale;
        let dt_myr = self.p.myr / f64::from(self.steps);
        let dt = dt_myr * 1.0e6;
        let gp = sp.glacial.clone();
        let gravity = self.preset.planet.gravity_m_s2;
        let mv = self.preset.scale.m_per_block_v;
        let sea_y = f64::from(self.preset.scale.sea_level_y);
        let fm = sp.numerics.flood;
        // L1's doublings to L2.
        let k1 = doublings(self.preset.resolution.l1_r, g2.r).unwrap_or(0);

        // The step reads the world and writes it only at its end (`started`: the routing again and
        // the outputs, timed together as before).
        let (out, started) = {
            let world: &WorldState = world;
            let z0 = read(world, field::ELEVATION_M, n);
            let class = read(world, field::CLASS, n);
            let weight = read(world, field::WEIGHT, n);
            let uplift = read(world, field::UPLIFT, n);
            // The rock's erodibility and diffusivity: L1's, interpolated ([`rock_property`]), and
            // the till's own where this stage's glaciers left enough of it (before this step's
            // glaciers add theirs) to be the surface rock.
            let till_class = usize::from(lithology::index("till"));
            let rock = |table: Vec<f64>, till_at: Option<&[f32]>| -> Vec<f64> {
                let (w1, h1) = (w >> k1, hgt >> k1);
                let smooth: Vec<f32> = match world
                    .fields
                    .get(super::field::CLASS)
                    .filter(|f| f.data.len() == w1 * h1)
                {
                    Some(c1) => rock_property(&c1.data, w1, h1, k1, &table),
                    // No L1 class (a world built by hand): the cell's own class.
                    None => rock_property(class, w, hgt, 0, &table),
                };
                (0..n)
                    .map(|c| match till_at {
                        Some(t) if t[c] >= TILL_SHOWS_M => table[till_class.min(table.len() - 1)],
                        _ => f64::from(smooth[c]),
                    })
                    .collect()
            };
            let (kf, erodibility) = {
                let erod = rock(lithology::erodibility(), till.as_deref());
                let kf: Vec<f64> = (0..n).map(|c| sp.k_spl * k_scale * erod[c]).collect();
                let erodibility = last.then(|| erod.iter().map(|&e| e as f32).collect::<Vec<_>>());
                (kf, erodibility)
            };

            let started = Instant::now();
            let mut h: Vec<f64> = (0..n)
                .map(|c| {
                    let z = f64::from(z0[c]);
                    if z > sea {
                        z + f64::from(uplift[c]) * dt_myr
                    } else {
                        z
                    }
                })
                .collect();
            let route = |h: &[f64],
                         base_z: &[f64],
                         kind: &mut Vec<u8>,
                         filled: &mut Vec<f64>,
                         rt: &mut routing::Routing,
                         area: &mut Vec<f64>| {
                pflood::open_sea_with(fm, &d, h, sea, (n / 500).max(1), kind);
                pflood::flood_with(fm, &d, h, base_z, kind, filled);
                routing::receivers(&d, filled, kind, rt);
                routing::stack(&d, kind, rt);
                routing::accumulate(&d, rt, weight, area);
            };
            let mut kind = vec![LAND; n];
            let mut filled = vec![0.0; n];
            let base_z = vec![sea; n];
            let mut rt = routing::Routing::default();
            let mut area = Vec::new();
            route(&h, &base_z, &mut kind, &mut filled, &mut rt, &mut area);
            self.time("route", started);

            let started = Instant::now();
            let glacial_clim = glaciated.then(|| {
                let (ela, t_sl, pr) = (
                    read(world, field::GLACIAL_ELA, n),
                    read(world, field::T_SEA_LEVEL, n),
                    read(world, field::PRECIP, n),
                );
                let ela_z: Vec<f64> = ela
                    .iter()
                    .map(|&e| {
                        if e.is_finite() {
                            sea + f64::from(e)
                        } else {
                            f64::NAN
                        }
                    })
                    .collect();
                let t_c: Vec<f32> = (0..n)
                    .map(|c| {
                        (f64::from(t_sl[c]) - climate::LAPSE_C_PER_M * (h[c] - sea).max(0.0)) as f32
                    })
                    .collect();
                let p_m: Vec<f32> = pr.iter().map(|&p| p / 1000.0).collect();
                (ela_z, t_c, p_m)
            });
            // Unlike the deep-time surface, rivers keep their K under the glacial state's ice:
            // that ice stood only through part of each ice-age cycle, subglacial meltwater cuts
            // too, and the `balance` uplift assumes the rivers' erosion goes on (taking it away
            // under thin, cold ice lifted whole lowlands by the uplift of 2 Myr in the first runs).
            self.time("glacial", started);

            let started = Instant::now();
            {
                let mut h_new = vec![0.0; n];
                let mut qin = vec![0.0; n];
                let spl_p = spl::SplParams {
                    m: sp.m_spl,
                    n: sp.n_spl,
                    g: sp.g_deposition,
                    dt,
                    sc: sp.threshold_slope,
                    max_iter: sp.spl_max_iter as usize,
                    tol_m: 0.01,
                };
                let input = spl::SplInput {
                    kind: &kind,
                    h0: &h,
                    filled: &filled,
                    base_z: &base_z,
                    area: &area,
                    kf: &kf,
                };
                spl::solve(&d, &rt, &input, &spl_p, &mut h_new, &mut qin);
                h = h_new;
            }
            drop(kf);
            drop(base_z);
            let drainage = |kind: &[u8], area: &[f64]| -> Vec<f32> {
                (0..n)
                    .map(|c| {
                        if kind[c] == BASE {
                            f32::NAN
                        } else {
                            (area[c] / 1.0e6).max(1e-3).log10() as f32
                        }
                    })
                    .collect()
            };
            // A step before the last keeps the drainage of the routing it started from (the last
            // routes the surface again).
            let mut drained = (!last).then(|| drainage(&kind, &area));
            drop(area);
            self.time("spl", started);

            // Glacial erosion and till (their loads to the sea leave the grid, as the rivers' do
            // here).
            let started = Instant::now();
            let mut gout = glacial::GlacialOut::default();
            if let Some((ela_z, t_c, p_m)) = &glacial_clim {
                let ic = glacial::IceClimate {
                    ela_z,
                    precip_m_yr: p_m,
                    t_c,
                    allowed: world
                        .fields
                        .get(field::ICE_ALLOWED)
                        .filter(|f| f.data.len() == n)
                        .map(|f| f.data.as_slice()),
                    time_share: None,
                };
                glacial::step(
                    &d, &rt, &kind, &filled, sea, &ic, &gp, gravity, dt, &mut h, &mut gout,
                );
                gout.release_work();
            }
            drop(glacial_clim);
            drop(rt);
            drop(filled);
            // The ice over the glaciers' whole width (`IceState::footprint`), what the outputs
            // read of the glaciers' state.
            let footprint = std::mem::take(&mut gout.ice.footprint);
            let ice_cells = if footprint.len() == n {
                gout.ice.cells
            } else {
                0
            };
            gout.ice = glacial::IceState::default();
            self.time("glacial", started);

            let started = Instant::now();
            let diffu = rock(lithology::diffusivity(), till.as_deref());
            // What the glaciers cut and left over the stage.
            let glacial_sums = if glaciated {
                let mut ero = ero_prev.take().unwrap_or_else(|| vec![0.0; n]);
                let mut t = till.take().unwrap_or_else(|| vec![0.0; n]);
                if gout.eroded.len() == n {
                    for c in 0..n {
                        ero[c] += gout.eroded[c] as f32;
                        t[c] += (gout.till[c] + gout.lake[c]) as f32;
                    }
                }
                Some((ero, t))
            } else {
                None
            };
            let glacial_m3 = (
                gout.eroded_m3,
                gout.till_m3 + gout.sink_m3 + gout.lake_m3,
                gout.sea_m3,
            );
            drop(gout);
            let kd: Vec<f64> = (0..n)
                .map(|c| {
                    let land = sp.kd_land_m2_yr * diffu[c];
                    marine::submarine_kd(
                        sea - h[c],
                        sp.kd_marine_m2_yr,
                        sp.wave_base_m,
                        sp.tidal_range_m,
                        land,
                    )
                })
                .collect();
            drop(diffu);
            let nm = &sp.numerics;
            diffusion::solve(
                &mut diffusion::Workspace::default(),
                &d,
                &mut h,
                &kd,
                dt,
                nm.diffusion_solver,
                nm.diffusion_tol_m,
            );
            drop(kd);
            // The sediment each L2 cell may lose to submarine slope failure: L1's log and loose
            // sediment over the L1 cell it lies in (`marine::slope_failure`), and what the step
            // laid down on it.
            if let Some(st) = world
                .surface
                .as_ref()
                .filter(|_| sp.submarine_slope_deg > 0.0 && n > 0)
            {
                let w1 = st.grid.w as usize;
                let sea_mask: Vec<bool> = kind.iter().map(|&k| k == BASE).collect();
                let mut avail: Vec<f64> = (0..n)
                    .map(|c| {
                        let c1 = ((c / w) >> k1) * w1 + ((c % w) >> k1);
                        let avail0 = if c1 < st.log.cells() {
                            st.log.total(c1) + f64::from(st.regolith_m[c1])
                        } else {
                            0.0
                        };
                        (avail0 + h[c] - f64::from(z0[c])).max(0.0)
                    })
                    .collect();
                let fp = marine::FailureParams {
                    critical_slope: sp.submarine_slope_deg.to_radians().tan(),
                    rate: marine::FAILURE_RATE,
                    iterations: marine::FAILURE_ITERATIONS,
                };
                marine::slope_failure(&d, &sea_mask, sea, &fp, &mut h, &mut avail);
            }
            self.time("diffusion", started);

            // The routing of the surface as it now is: areas, lakes.
            let started = Instant::now();
            let mut last_out = None;
            if last {
                let mut filled = vec![0.0; n];
                let mut rt = routing::Routing::default();
                let mut area = Vec::new();
                let base_z = vec![sea; n];
                route(&h, &base_z, &mut kind, &mut filled, &mut rt, &mut area);
                drop(base_z);
                drop(rt);
                drained = Some(drainage(&kind, &area));
                let area32: Vec<f32> = area.iter().map(|&a| a as f32).collect();
                drop(area);
                let lake: Vec<f32> = (0..n)
                    .map(|c| {
                        let depth = filled[c] - h[c];
                        if kind[c] == SINK {
                            (2.0 * LAKE_TOL_M) as f32
                        } else if kind[c] == LAND && depth > LAKE_TOL_M {
                            depth as f32
                        } else {
                            0.0
                        }
                    })
                    .collect();
                let lakes = (0..n)
                    .filter(|&c| {
                        kind[c] == SINK || (kind[c] == LAND && filled[c] - h[c] > LAKE_TOL_M)
                    })
                    .count();
                let sinks = kind.iter().filter(|&&k| k == SINK).count();
                last_out = Some(LastOut {
                    area: area32,
                    lake,
                    lakes,
                    sinks,
                });
            }
            let z_out: Vec<f32> = h.iter().map(|&z| z as f32).collect();
            let block_y: Vec<f32> = h.iter().map(|&z| (sea_y + (z - sea) / mv) as f32).collect();
            // The glaciers: where they are now (the amplifier shapes valleys under them), what they
            // cut and left over the stage; thick till becomes the surface rock.
            let surface: Vec<f32> = if footprint.len() == n {
                (0..n)
                    .map(|c| {
                        let s = footprint[c];
                        if s.is_finite() && s > h[c] {
                            s as f32
                        } else {
                            f32::NAN
                        }
                    })
                    .collect()
            } else {
                vec![f32::NAN; n]
            };
            drop(footprint);
            let ice_now: Vec<f32> = (0..n)
                .map(|c| {
                    if surface[c].is_finite() {
                        (f64::from(surface[c]) - h[c]) as f32
                    } else {
                        0.0
                    }
                })
                .collect();
            let glacial = glacial_sums.map(|(ero, t)| {
                let till_class = f32::from(lithology::index("till"));
                let class_now: Vec<f32> = (0..n)
                    .map(|c| {
                        if t[c] >= TILL_SHOWS_M {
                            till_class
                        } else {
                            class[c]
                        }
                    })
                    .collect();
                let a_km2 = dx * dx / 1.0e9;
                let note = json!({
                    "ice_cells": ice_cells,
                    "eroded_km3": glacial_m3.0 / 1.0e9,
                    "till_km3": glacial_m3.1 / 1.0e9,
                    "sea_km3": glacial_m3.2 / 1.0e9,
                    "total_eroded_km3": ero.iter().map(|&e| f64::from(e)).sum::<f64>() * a_km2,
                    "till_cells": t.iter().filter(|&&x| x >= TILL_SHOWS_M).count(),
                });
                GlacialFields {
                    surface,
                    erosion: ero,
                    till: t,
                    class: class_now,
                    note,
                }
            });
            let out = StepOut {
                land: last.then(|| land_stats(&z_out, sea)),
                z_out,
                block_y,
                drainage: drained.unwrap_or_default(),
                erodibility,
                ice_now,
                glacial,
                last: last_out,
            };
            (out, started)
        };

        let put = |world: &mut WorldState, name: &str, data: Vec<f32>| {
            world.fields.insert(
                name.to_owned(),
                Field {
                    grid: g2.clone(),
                    data,
                },
            );
        };
        put(world, field::ELEVATION, out.block_y);
        put(world, field::DRAINAGE, out.drainage);
        if let Some(e) = out.erodibility {
            put(world, field::ERODIBILITY, e);
        }
        let mut glacial_note = None;
        match out.glacial {
            Some(g) => {
                put(world, field::ICE_SURFACE, g.surface);
                put(world, field::GLACIAL_EROSION, g.erosion);
                put(world, field::TILL, g.till);
                put(world, field::CLASS, g.class);
                glacial_note = Some(g.note);
            }
            None => {
                if let Some(e) = ero_prev {
                    put(world, field::GLACIAL_EROSION, e);
                }
                if let Some(t) = till {
                    put(world, field::TILL, t);
                }
            }
        }
        put(world, field::ICE, out.ice_now);
        if let (Some(lo), Some((p50, p90, max))) = (out.last, out.land) {
            put(world, field::AREA_M2, lo.area);
            put(world, field::LAKE_M, lo.lake);
            let mut note: Value = world
                .notes
                .get(STAGE_NAME)
                .and_then(|t| serde_json::from_str(t).ok())
                .unwrap_or_else(|| json!({}));
            note["after"] = json!({"land_z_p50_m": p50, "land_z_p90_m": p90, "land_z_max_m": max});
            note["lake_cells"] = json!(lo.lakes);
            if let Some(g) = glacial_note {
                note["glacial"] = g;
            }
            note["sinks"] = json!(lo.sinks);
            note["millis"] = json!(self.timings);
            world.notes.insert(STAGE_NAME.into(), note.to_string());
        }
        put(world, field::ELEVATION_M, out.z_out);
        self.time("outputs", started);
        Ok(())
    }
}

/// What a step of [`YoungStage::lem`] writes into the world.
struct StepOut {
    z_out: Vec<f32>,
    block_y: Vec<f32>,
    drainage: Vec<f32>,
    /// The last step's.
    erodibility: Option<Vec<f32>>,
    ice_now: Vec<f32>,
    glacial: Option<GlacialFields>,
    /// The last step's land elevations above sea: median, 90th percentile, maximum.
    land: Option<(f64, f64, f64)>,
    last: Option<LastOut>,
}

/// The glaciers' fields after a step: the ice surface, the glacial erosion and till over the
/// stage, the surface rock class and the note.
struct GlacialFields {
    surface: Vec<f32>,
    erosion: Vec<f32>,
    till: Vec<f32>,
    class: Vec<f32>,
    note: Value,
}

/// What the last step's routing gives: the drainage area, the lakes and the count of lake and
/// sink cells.
struct LastOut {
    area: Vec<f32>,
    lake: Vec<f32>,
    lakes: usize,
    sinks: usize,
}

/// `young_final=…` (the CLI's line).
pub fn summary_line(v: &Value) -> String {
    let f = |p: &Value| p.as_f64().unwrap_or(f64::NAN);
    let g = &v["glacial"];
    let glacial = if g.is_object() {
        format!(
            " ice_cells={} glacial_eroded_km3={:.3e} till_cells={}",
            g["ice_cells"],
            f(&g["total_eroded_km3"]),
            g["till_cells"]
        )
    } else {
        String::new()
    };
    format!(
        "young_final={}x{} steps={} myr={} uplift={} land_z_p50_m={:.0}->{:.0} land_z_p90_m={:.0}->{:.0} \
         land_z_max_m={:.0}->{:.0} lake_cells={} sinks={} filled_small_depressions={} \
         sea_floor_failed_km3={:.3e}{glacial}",
        v["l2"]["w"],
        v["l2"]["h"],
        v["steps"],
        v["myr"],
        v["uplift"].as_str().unwrap_or("?"),
        f(&v["before"]["land_z_p50_m"]),
        f(&v["after"]["land_z_p50_m"]),
        f(&v["before"]["land_z_p90_m"]),
        f(&v["after"]["land_z_p90_m"]),
        f(&v["before"]["land_z_max_m"]),
        f(&v["after"]["land_z_max_m"]),
        v["lake_cells"],
        v["sinks"],
        v["filled_small_depressions"],
        f(&v["sea_floor_failed_km3"]),
    )
}

impl Stage for YoungStage {
    fn name(&self) -> &str {
        STAGE_NAME
    }

    fn estimate(&self) -> StageEstimate {
        estimate(&self.preset)
    }

    fn total_steps(&self) -> u64 {
        self.total()
    }

    fn steps_done(&self) -> u64 {
        self.done
    }

    fn step(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        if self.done >= self.total() {
            return Ok(());
        }
        let s = self.done;
        let last = s + 1 == self.total();
        ctx.pool.install(|| {
            if s == 0 {
                self.setup(world, ctx)
            } else {
                self.lem(world, ctx, last)
            }
        })?;
        self.done += 1;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::SplitMix;

    /// P1-17: the young stage's estimate on the built-in 16k presets covers what the stage was
    /// measured to hold (the world before it plus its own arrays, the history aside; 500 Myr,
    /// scattered continents) by at most 15 %, and full 16k's whole estimate fits a 5 GiB cap
    /// (it listed 6,049 MiB against the 4,143 MiB held, so the editor refused full at 5 GiB).
    /// The holds were measured again for P1-19's multigrid diffusion solve (2026-10-19,
    /// `DEEPTIME_MEM_TRACE`, 8 threads, the 4 GiB cap, the kept-block cache off; the stage's
    /// counted peak less the history it kept in memory): quick 1,105 → 1,162 MiB (1,376.5 less
    /// 214.5; master's own run gave 1,322.8 less 214.2 = 1,108.6, so the method matches P1-17's),
    /// full 4,143 → 4,311 MiB (the history all on disk). Again after the merge with master
    /// fb38204f (2026-10-19, the same method, CPU kernels): quick 1,162 → 1,171 MiB (1,390.1 less
    /// 219.5; master's own run gave 1,344.3 less 222.6 = 1,121.7, 13 MiB above P1-19's 1,108.6),
    /// full 4,311 → 4,349 MiB. Again once the step freed its arrays as soon as it stopped reading
    /// them (2026-10-19, the same method): quick 1,171 → 947 MiB (1,166.9 less 219.5), full
    /// 4,349 → 3,462 MiB (the history all on disk); they list 1,060 and 3,769 MiB with 320 B a
    /// cell, 1,086 and 3,872 MiB with 340 (2026-10-20, `OWN_BYTES_PER_L2_CELL`). Gap 4
    /// (2026-10-23) gave every plate `Sample` two more `f32`s (`lithosphere_km`, `keel_km`), 8 B a
    /// sample in the three copies `plates::estimate` lists: 2.3 MiB on quick (100,000 samples) and
    /// 5.7 MiB on full (250,000). The holds below are the 2026-10-19 measures plus that, not
    /// measured again (quick 947 → 949, full 3,462 → 3,468 MiB); quick lists 1,107 MiB. Gap 6
    /// (2026-10-24) gave it a `u32` and an `f32` more (`carrier`, `carrier_lithosphere_km`), the
    /// same 8 B in the same three copies, added the same way (quick 949 → 951, full 3,468 → 3,474
    /// MiB); quick lists 1,109 MiB.
    /// YNG-002: unset, the stage steps as long as the deep-time surface (the plate step over
    /// `surface.substeps`); a preset's `steps` still wins.
    #[test]
    fn the_young_steps_are_as_long_as_the_surfaces() {
        let planet = |young: serde_json::Value, substeps: u32| {
            let text = crate::preset::with_timeline(
                crate::preset::builtin("earthlike_quick_16k").unwrap(),
                Some("scattered_continents"),
                Some(500.0),
            )
            .unwrap();
            let mut v: serde_json::Value = serde_json::from_str(&text).unwrap();
            v["young"] = young;
            v["surface"]["substeps"] = serde_json::json!(substeps);
            let crate::preset::PresetBody::Planet(p) =
                crate::preset::Preset::from_json(&v.to_string())
                    .unwrap()
                    .body
            else {
                panic!("a planet preset");
            };
            p
        };
        let steps = |p: &PlanetPreset| p.young.clone().unwrap_or_default().steps_for(p);
        let p = planet(serde_json::json!({}), 1);
        assert_eq!(p.dt_myr(), 2.0);
        assert_eq!(steps(&p), 1, "2 Myr in the surface's 2 Myr steps");
        assert_eq!(steps(&planet(serde_json::json!({}), 4)), 4);
        assert_eq!(
            steps(&planet(serde_json::json!({"myr": 0.4}), 1)),
            1,
            "at least one"
        );
        assert_eq!(steps(&planet(serde_json::json!({"myr": 10.0}), 1)), 5);
        assert_eq!(
            steps(&planet(serde_json::json!({"steps": 4}), 1)),
            4,
            "set, it wins"
        );
        assert_eq!(YoungStage::new(*p).total(), 2, "the set-up and one step");
    }

    #[test]
    fn the_estimate_covers_the_measured_hold_closely() {
        const MIB: u64 = 1 << 20;
        let json = |name: &str| {
            crate::preset::with_timeline(
                crate::preset::builtin(name).unwrap(),
                Some("scattered_continents"),
                Some(500.0),
            )
            .unwrap()
        };
        for (name, held_mib) in [
            ("earthlike_quick_16k", 951u64),
            ("earthlike_full_16k", 3474),
        ] {
            let crate::preset::PresetBody::Planet(p) =
                crate::preset::Preset::from_json(&json(name)).unwrap().body
            else {
                panic!("{name} is a planet preset");
            };
            let e = estimate(&p);
            let base = (e.bytes - e.history_bytes) / MIB;
            // The 15 % is the per-cell figures'; what is on no grid is a constant beside them
            // (amp-est, 2026-10-23: `surface::UNGRIDDED_BYTES`, which these grids' slack covered).
            let ungridded = crate::surface::UNGRIDDED_BYTES / MIB;
            assert!(
                base >= held_mib && base <= held_mib * 115 / 100 + ungridded,
                "{name}: {base} MiB listed, {held_mib} MiB held"
            );
        }
        let v: serde_json::Value = serde_json::from_str(
            &crate::session::estimate_for(&json("earthlike_full_16k"), "", 5 << 30).unwrap(),
        )
        .unwrap();
        assert!(
            v["peak_bytes"].as_u64().unwrap() <= 5 << 30,
            "full 16k at a 5 GiB cap: {v}"
        );
    }

    /// The sea floor the plate model leaves, on 2.4 km cells (quick 16k's L2): an abyssal plain at
    /// 4.5 km, a trench one cell wide pulled to 10 km, a strip where two crusts interleave cell
    /// by cell (3 km steps), and a continent with its shelf ending in a one-cell wall. The
    /// steepest pair of open-sea neighbours, as a gradient (drop over distance).
    fn sea_floor_world() -> (Dims, Vec<bool>, Vec<f32>) {
        let (w, h) = (96usize, 48usize);
        let d = Dims::new(w, h, 2400.0, true);
        let mut z = vec![-4500.0f32; w * h];
        for j in 0..h {
            for i in 0..w {
                let c = j * w + i;
                if i < 12 {
                    z[c] = 300.0;
                } else if i < 16 {
                    z[c] = -100.0;
                } else if i == 40 {
                    z[c] = -10_000.0;
                } else if (60..72).contains(&i) && ((i / 2) + (j / 2)) % 2 == 0 {
                    z[c] = -1500.0;
                }
            }
        }
        let open = z.iter().map(|&v| v < 0.0).collect();
        (d, open, z)
    }

    fn steepest_open_pair(d: &Dims, open: &[bool], z: &[f32]) -> f64 {
        let mut top = 0.0f64;
        for (c, (&zc, &o)) in z.iter().zip(open).enumerate() {
            if !o {
                continue;
            }
            for (m, dist) in d.neighbours(c) {
                if open[m] {
                    top = top.max(f64::from(zc - z[m]) / dist);
                }
            }
        }
        top
    }

    /// Sea-floor spikes task (2026-10-20): the walls the plate model draws one cell wide under the
    /// sea fail to the sea floor's critical slope, rock and all. Before the task nothing below
    /// the wave base relaxed them (the marine diffusion vanishes there and the slope failure
    /// moved sediment only), and they stood at 51 to 66 degrees here; the planet's were 30 to 45
    /// degrees on the ground and 80 and more in the game. Matter is kept, land is not touched,
    /// and the trench stays a trench.
    #[test]
    fn the_plate_model_sea_floor_fails_to_its_critical_slope() {
        let (d, open, z0) = sea_floor_world();
        let crit = SEA_FLOOR_SLOPE_DEG.to_radians().tan();
        let before = steepest_open_pair(&d, &open, &z0);
        assert!(before > 1.2, "the walls start steep: {before}");
        let mut z = z0.clone();
        let km3 = fail_sea_floor(&d, &open, 0.0, SEA_FLOOR_SLOPE_DEG, &mut z);
        assert!(km3 > 0.0);
        let after = steepest_open_pair(&d, &open, &z);
        assert!(
            after <= crit * 1.03,
            "steepest sea-floor pair {:.1} degrees, critical {SEA_FLOOR_SLOPE_DEG}",
            after.atan().to_degrees()
        );
        let sum = |v: &[f32]| v.iter().map(|&x| f64::from(x)).sum::<f64>();
        let total = z0.iter().map(|&x| f64::from(x.abs())).sum::<f64>();
        assert!(
            (sum(&z) - sum(&z0)).abs() < 1e-6 * total,
            "matter kept: {} vs {}",
            sum(&z),
            sum(&z0)
        );
        for (c, ((&a, &b), &o)) in z.iter().zip(&z0).zip(&open).enumerate() {
            if !o {
                assert_eq!(a.to_bits(), b.to_bits(), "land cell {c} moved");
            }
        }
        let w = d.w;
        let trench = (0..d.h).map(|j| z[j * w + 40]).fold(f32::MAX, f32::min);
        assert!(trench < -5500.0, "the trench is still deep: {trench}");
        // 0 turns the failure off; the key stays under 60 degrees.
        let mut y = YoungParams {
            sea_floor_slope_deg: 0.0,
            ..YoungParams::default()
        };
        assert!(y.validate().is_ok());
        y.sea_floor_slope_deg = 75.0;
        assert!(y.validate().is_err());
    }

    /// P1-23: what the erosion reads of the rock has no L1 cell in it. On a random map of two
    /// rocks the property changes as much between two fine cells of one L1 cell as between two
    /// of neighbouring L1 cells (copied per cell, as it was, it does not change inside one at
    /// all), no step is larger than the contrast shared over the cell, and the mean is kept.
    #[test]
    fn rock_properties_are_interpolated_between_l1_cells() {
        let (w1, h1) = (64usize, 32usize);
        let table = [1.0f64, 3.0, 0.5];
        let mut rng = SplitMix(5);
        let class1: Vec<f32> = (0..w1 * h1).map(|_| rng.below(2) as f32).collect();
        for k in [1u32, 2] {
            let f = 1usize << k;
            let (w, h) = (w1 * f, h1 * f);
            let v = rock_property(&class1, w1, h1, k, &table);
            assert_eq!(v.len(), w * h);
            // The L1 cells' own points keep their values.
            for j in 0..h1 {
                for i in 0..w1 {
                    assert_eq!(
                        v[j * f * w + i * f],
                        table[class1[j * w1 + i] as usize] as f32
                    );
                }
            }
            // Steps along x by the place inside the L1 cell.
            let mut by_phase = vec![0.0f64; f];
            let mut largest = 0.0f64;
            for j in 0..h {
                for i in 0..w {
                    let step = f64::from((v[j * w + (i + 1) % w] - v[j * w + i]).abs());
                    by_phase[i % f] += step;
                    largest = largest.max(step);
                }
            }
            let (lo, hi) = by_phase
                .iter()
                .fold((f64::MAX, f64::MIN), |(lo, hi), &x| (lo.min(x), hi.max(x)));
            assert!(lo > 0.0, "k {k}: no change inside an L1 cell: {by_phase:?}");
            assert!(
                hi / lo < 1.25,
                "k {k}: the steps sit at the L1 cells' edges: {by_phase:?}"
            );
            assert!(
                largest <= 2.0 / f as f64 + 1e-6,
                "k {k}: a step of {largest} between neighbours (contrast 2)"
            );
            let mean1: f64 =
                class1.iter().map(|&c| table[c as usize]).sum::<f64>() / (w1 * h1) as f64;
            let mean: f64 = v.iter().map(|&x| f64::from(x)).sum::<f64>() / v.len() as f64;
            assert!((mean - mean1).abs() < 0.02, "k {k}: mean {mean} vs {mean1}");
        }
        // One rock everywhere: one value; a class beyond the table reads its last entry.
        let v = rock_property(&vec![2.0; w1 * h1], w1, h1, 1, &table);
        assert!(v.iter().all(|&x| x == 0.5));
        assert_eq!(
            rock_property(&[9.0, f32::NAN], 2, 1, 0, &table),
            vec![0.5, 1.0]
        );
    }
}

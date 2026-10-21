// SPDX-License-Identifier: LGPL-3.0-or-later
//! The parametric climate on L1 (plan §8.3, work item P1-05): a seasonal zonal energy balance
//! with the ice-albedo feedback (`ebm.rs`, climlab's equations), insolation from the tilt and the
//! star (`insolation.rs`), three-cell wind belts with a migrating ITCZ and the ocean gyres
//! (`belts.rs`), continentality, rain shadows and the lapse rate on the DEM (`lapse.rs`), Smith &
//! Barstad's orographic precipitation along each belt's wind (`orographic.rs`, ported from the MIT
//! `fastscape-lem/orographic-precipitation`, rustfft), the Köppen classes (`koppen.rs`) and
//! glaciers from the equilibrium line (`glaciers.rs`); `snapshot.rs` puts them together.
//!
//! **Deep time** (decisions log 2026-09-29): a long-term carbon cycle and a brightening star
//! (`carbon.rs`) set the EBM's greenhouse and sunlight; a fine clock (`clock.rs`) advances the
//! carbon, the EBM and the ice every 200 kyr inside each plate step, so snowball Earths can start
//! and end on their own (events `deeptime:snowball`, `deeptime:hothouse`); the last 3 Myr run
//! Milankovitch forcing (`orbit.rs`) with lagged ice sheets; the land ice has a thickness and a
//! volume that lowers the sea (`ice.rs`), which the surface stage takes as its sea level.
//!
//! # In the pipeline
//! The stage is interleaved with the plates like the surface (`Stage::coupled_to`) and sits
//! between them, so a planet preset with a `climate` section runs `[plates, climate, surface]`
//! each plate step. Every step it publishes `climate/{t_mean_c, precip_mm}` (what the surface's
//! runoff, weathering, facies and the deposit engine's frames read, `surface/climate.rs`),
//! `climate/{koppen, t_warm_c, t_cold_c, aridity, glacier, ela_m}` and the ice fields of FORMAT.md
//! "The climate's ice fields" (`climate/ice_thickness_m`, `ice_max_thickness_m`,
//! `ice_max_extent`, `ice_time_share`, `sea_ice`), on the topography the surface step is about to
//! erode: the plate model's elevation after this plate step plus the surface's offset carried with
//! the plates exactly as the surface carries it. The step's eustatic sea level goes to
//! `WorldState::globals["climate/sea_level_m"]` (`SEA_LEVEL_GLOBAL`), which the surface uses.
//!
//! **Snapshots** (plan §8.3: every 10 Myr, plus the present; 20 Myr in the pipeline presets since
//! P1-19 #27): a full climate is computed at the
//! first step, every `snapshot_every_myr` of simulated time, and at the last step. Between them
//! the climate's geographic part — each cell's sea-level departure from its row's zonal reference
//! (the EBM's ocean temperature; ln of the belts' precipitation), its aridity, its ELA and its
//! Köppen class — moves with the plates (the surface's semi-Lagrangian advection, `surface::
//! advect`), and the temperatures take the lapse rate of the current ground: a rain shadow stays
//! behind its range as the range drifts, and a continent drifting toward the pole cools with its
//! latitude's reference. The zonal references follow the fine clock's EBM every step (the
//! greenhouse and the sun change between snapshots), and the equilibrium line moves with each
//! band's summer temperature. Glaciation events are decided every step.
//!
//! The EBM's state is kept from step to step, and the ice of one step brightens the EBM's land at
//! the next (`ice_sheet_feedback`, on by default since P1-17's calibration made the continents
//! Earth-low).

pub mod belts;
pub mod carbon;
pub mod clock;
pub mod co2;
pub mod coarse;
pub mod ebm;
pub mod ebm2d;
pub mod glaciers;
#[cfg(feature = "gpu")]
pub mod gpu;
pub mod ice;
pub mod insolation;
pub mod koppen;
pub mod lapse;
pub mod moisture;
pub mod ocean2d;
pub mod orbit;
pub mod orographic;
pub mod params;
pub mod percell;
pub mod sheet;
pub mod snapshot;
pub mod upwind;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use deeptime_format::{GridDesc, Place};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use params::{ClimateModel, ClimateParams, PresentClimate, RainModel};

use crate::deposits::fields::{cell_area_km2, lat_of_row};
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::stage::{Stage, StageContext, StageEstimate, from_state, to_state};
use crate::surface::advect;
use crate::surface::grid::Dims;
use crate::world::{Field, WorldState};
use clock::{ClockState, Happening};
use ebm::{EbmResult, EbmState};
use snapshot::{Planet, Refs};

/// The stage's name in a pipeline (`deeptime stage climate`).
pub const STAGE_NAME: &str = "climate";

/// Carbonate thickness the step's weathering drawdown lays on the sea floor above the carbonate
/// compensation depth (≤ 4 km deep, `surface::CARBONATE_FLOOR_M`), m, in
/// `WorldState::globals`: the surface stage deposits it (after a snowball: the cap carbonate).
pub const CAP_CARBONATE_GLOBAL: &str = "climate/cap_carbonate_m";

/// The step's eustatic sea level (the surface section's curve plus glacio-eustasy), m, in
/// `WorldState::globals`: the surface stage takes it instead of its curve.
pub const SEA_LEVEL_GLOBAL: &str = "climate/sea_level_m";

/// Fields the stage writes every step.
pub mod field {
    /// Mean annual surface temperature, °C (the surface stage reads it).
    pub const T_MEAN: &str = crate::surface::climate::T_FIELD;
    /// Annual precipitation, mm/yr (the surface stage reads it).
    pub const PRECIP: &str = crate::surface::climate::P_FIELD;
    /// Köppen class index (`koppen::CODES`; 31 = sea).
    pub const KOPPEN: &str = "climate/koppen";
    /// Mean temperature of the warmest and the coldest month, °C.
    pub const T_WARM: &str = "climate/t_warm_c";
    pub const T_COLD: &str = "climate/t_cold_c";
    /// P / PET (Thornthwaite), capped at 10.
    pub const ARIDITY: &str = "climate/aridity";
    /// 1 where land ice stands at the end of the step (`ICE_THICKNESS` > 0).
    pub const GLACIER: &str = "climate/glacier";
    /// Equilibrium-line altitude above sea level, m (NaN at sea).
    pub const ELA: &str = "climate/ela_m";
    /// Carried with the plates between snapshots (sea-level departures from the row references).
    pub const D_MEAN: &str = "climate/anomaly/t_mean_c";
    pub const D_WARM: &str = "climate/anomaly/t_warm_c";
    pub const D_COLD: &str = "climate/anomaly/t_cold_c";
    pub const D_LN_PRECIP: &str = "climate/anomaly/ln_precip";
    /// The ELA of every cell at the last snapshot, sea included (so it can be interpolated);
    /// the published ELA is this plus its band's shift since.
    pub const ELA_ALL: &str = "climate/anomaly/ela_m";

    /// Month `m`'s (0–11) sea-level temperature, °C (the lapse rate not applied; on land the
    /// ground's temperature brought to sea level, the plateau's warmth included), written at the
    /// last step only (the bake stores it, P1-25). Month 0 starts at solar longitude 280°
    /// (`insolation::YEAR_START_RAD`, about 1 January on Earth).
    pub fn t_sl_month(m: usize) -> String {
        format!("climate/month/t_sl_c_{:02}", m + 1)
    }

    /// Month `m`'s (0–11) precipitation rate, mm/yr (the month's share is the rate / 12), written
    /// at the last step only.
    pub fn p_month(m: usize) -> String {
        format!("climate/month/p_mm_{:02}", m + 1)
    }
    /// Land ice at the end of the step, m (0 at sea and where there is none). The ice-field
    /// contract with the surface stage (glacial erosion) is FORMAT.md "The climate's ice fields".
    pub const ICE_THICKNESS: &str = "climate/ice_thickness_m";
    /// The thickest the ice stood during the step's fine clock; inside the recent ice-age window
    /// the maximum since the window began (at the present: the last glacial maximum), m.
    pub const ICE_MAX_THICKNESS: &str = "climate/ice_max_thickness_m";
    /// 1 where `ICE_MAX_THICKNESS` > 0.
    pub const ICE_MAX_EXTENT: &str = "climate/ice_max_extent";
    /// Share of the step's time the cell was under ice, 0–1.
    pub const ICE_TIME_SHARE: &str = "climate/ice_time_share";
    /// Annual-mean sea-ice cover of the sea, 0–1 (0 on land): per cell under the per-cell
    /// climate (P1-26), the band's under the banded one.
    pub const SEA_ICE: &str = "climate/sea_ice";
    /// The winter maximum and the summer minimum of the monthly sea-ice covers, 0–1 (pipeline
    /// and keyframe fields, not baked; P1-26).
    pub const SEA_ICE_MAX: &str = "climate/sea_ice_max";
    pub const SEA_ICE_MIN: &str = "climate/sea_ice_min";
    /// Surface mass balance of the land ice at the end of the step, m of ice per year: positive
    /// where snow builds it (above the equilibrium line, at most what the precipitation allows),
    /// negative where it melts (below the line: the meltwater's source); 0 off the ice.
    pub const ICE_BALANCE: &str = "climate/ice_balance_m_yr";
}

/// The ice fields the surface stage's field history records as frame means.
pub mod history {
    /// Time-mean land-ice thickness over the frame, m.
    pub const ICE_THICKNESS: &str = "deeptime:ice_thickness_m";
    /// Share of the frame's time under ice, 0–1.
    pub const ICE_COVER: &str = "deeptime:ice_cover";
}

/// Bytes the stage holds per L1 cell (`estimate`).
const BYTES_PER_CELL: u64 = 154 + 96;

/// Keyframe layers (with the group's plates and surface layers).
pub const KEYFRAME_LAYERS: [&str; 7] = [
    field::T_MEAN,
    field::PRECIP,
    field::KOPPEN,
    field::ICE_THICKNESS,
    field::SEA_ICE,
    field::SEA_ICE_MAX,
    field::SEA_ICE_MIN,
];

/// Event kinds of the deep-time climate (EVENTS.md "The climate's events").
pub const SNOWBALL: &str = "deeptime:snowball";
pub const HOTHOUSE: &str = "deeptime:hothouse";
/// The overturning conveyor's sinking moved to another basin (P1-26).
pub const OCEAN_OVERTURNING_SHIFT: &str = "deeptime:ocean_overturning_shift";

/// Ice thinner than this is no ice, m.
pub const ICE_MIN_M: f32 = 1.0;
/// The smallest ice sheet, Earth-normalized km²: "an ice body that covers an area of continental
/// size, generally defined as covering 50 000 km² or more" (Cogley et al. 2011, glossary, "Ice
/// sheet"). The smaller bodies (glaciers, ice caps, icefields) are the mountain ice, which the
/// present-day setting's land-ice share leaves out (decisions log 2026-10-18).
pub const ICE_SHEET_MIN_KM2: f64 = 50_000.0; // ledger: CLM-203
/// How far an interglacial's warm summers lift the equilibrium line above its mean, m (about
/// 2 °C of summer warmth; Earth's early Holocene), in the deep-time sheet's relaxation.
pub const INTERGLACIAL_RISE_M: f32 = 300.0; // ledger: GLA-074

/// What the stage keeps between steps (and in a checkpoint taken inside the group).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Saved {
    done: u64,
    /// Every plate's Ω after the last step (the motion of the next plate step).
    omega: Option<Vec<[f64; 3]>>,
    ebm: EbmState,
    refs: Refs,
    tracker: glaciers::Tracker,
    snapshots: u64,
    /// The last snapshot's statistics, JSON text.
    last: String,
    clock: ClockState,
    /// Sea level and grounded ice of the last step (m; Earth-normalized km³).
    sea_level_m: Option<f64>,
    ice_volume_km3: f64,
    /// The land's share under ice, all of it and the ice sheets' (`ICE_SHEET_MIN_KM2`).
    land_ice_frac: f64,
    land_sheet_frac: f64,
    /// Oceanic crust area of the last step and the smoothed consumption rate (degassing).
    ocean_km2: Option<f64>,
    subduction_km2_yr: Option<f64>,
    /// Drawdown carbonate laid on the sea floor over the run, m.
    cap_carbonate_m: f64,
    /// The ice sheet's coarse state (thickness, bed depression) inside the ice-age window.
    sheet: sheet::State,
    /// What `present_climate` did at the end of the run.
    present: Option<Present>,
    /// The per-cell climate's state between snapshots (P1-26).
    cell: percell::PerCell,
    /// Each band's annual sea ice in the zonal model at the last snapshot (between snapshots a
    /// cell's sea ice moves with its band's change since).
    snap_band_ice: Vec<f64>,
    /// The zonal model's correction fitted at the last snapshot (P1-26 design §4.1).
    correction: percell::Correction,
    /// The clock's sea-ice cover at the last snapshot (a triggered snapshot's reference).
    snap_sea_ice: f64,
    /// Snapshots the clock's departure triggered.
    triggered: u64,
    /// The CO₂ of every snowball's escape, ppm (the run's notes report them beside Earth's
    /// expected 10⁴–10⁵ ppm, design §4.2).
    escape_ppm: Vec<f64>,
    /// The end-weighted nudges' controllers (`steering`; nothing while it is off or before its tail).
    steer: crate::steering::SteerState,
    /// Where the snapshots' cells ran over the run (`CellsAccel`).
    cells: CellsCount,
}

/// What `present_climate` did: the CO₂ factor it settled on and the state with and without it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Present {
    ln_factor: f64,
    /// Whether the state asked for was reached (an ice age only reachable as a snowball, or a
    /// state beyond a factor of 10⁴, is not).
    reached: bool,
    trials: u32,
    /// The land's share under ice, all of it and the ice sheets' (`ICE_SHEET_MIN_KM2`; the
    /// target counts the sheets, decisions log 2026-10-18), without and with the factor.
    natural_land_ice: f64,
    natural_land_sheet: f64,
    natural_sea_ice: f64,
    natural_snowball: bool,
    land_ice: f64,
    land_sheet: f64,
    sea_ice: f64,
    /// Every trial in the order run: (ln factor, land under ice sheets, land under ice, sea ice,
    /// global mean °C); the first is the natural end.
    rows: Vec<[f64; 5]>,
    /// Where the ice sheets lie, without and with the factor.
    natural_profile: SheetProfile,
    profile: SheetProfile,
}

/// Where the land's ice sheets lie: for each class of height above the sea (`HEIGHT_CLASSES_M`)
/// and of latitude (`LAT_CLASSES_DEG`), the class's share of the land and the share of the class's
/// land under the sheets (true-area weighted).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct SheetProfile {
    height_land: Vec<f64>,
    height_sheet: Vec<f64>,
    lat_land: Vec<f64>,
    lat_sheet: Vec<f64>,
}

/// The lower edges of the present's height classes above the sea, m (Earth: 0.102 of the land
/// above 2 km, docs/science/plates.md).
const HEIGHT_CLASSES_M: [f64; 4] = [0.0, 1000.0, 2000.0, 3000.0];
/// The lower edges of the present's latitude classes, degrees from the equator.
const LAT_CLASSES_DEG: [f64; 3] = [0.0, 30.0, 60.0];

/// The ice a clock run carries from step to step inside one plate step.
#[derive(Clone)]
struct IceRun {
    /// Thickness now, the step's maximum, and the time each cell spent under ice, kyr.
    hs: Vec<f32>,
    hmax: Vec<f32>,
    share_acc: Vec<f32>,
    /// The equilibrium glaciers of the fine ground and the kyr since they were computed.
    heq: Vec<f32>,
    since_eq: f64,
    sheet: sheet::State,
    /// (land ice share, volume) of every window step.
    rows: Vec<(f64, f64)>,
    /// Whether the last clock step was in the window (the ice is `hs`), and outside it the last
    /// step's shifts of the line (the ice is the land above the shifted line).
    in_window: bool,
    delta: Vec<f64>,
}

/// One run of the last stretch under a CO₂ factor (`present_climate`).
struct Trial {
    ln_factor: f64,
    land_ice: f64,
    land_sheet: f64,
    sea_ice: f64,
    snowball: bool,
    profile: SheetProfile,
    clock: ClockState,
    ebm: EbmState,
    ir: IceRun,
    out: clock::Output,
}

/// The present's search (`present_climate`, CLM-151): from the natural end (`natural`, ln factor
/// 0), ln(factor) is bracketed at 0.35, 0.7, 1.4, 2.8, 5.6 and 9.2 times `dir` (at most about
/// ×10⁴), the bracket stopping at `cap` and trying the cap itself (CLM-206), then the bracket is
/// halved four times. The bracket's trials are independent (each starts from the same state), so
/// they run side by side and are read in order, as a serial search reads them: the first that
/// reaches the state ends the bracket, and only the trials up to it count (P1-17). Returns the
/// trial chosen, whether it reached the state (and `accept` took it; otherwise the chosen trial is
/// the last one short of the state), and the trials counted (the natural end's included);
/// `seen` gets every counted trial in order.
#[allow(clippy::too_many_arguments)]
fn present_search<T: Send>(
    natural: T,
    dir: f64,
    cap: f64,
    trial: &(dyn Fn(f64) -> T + Sync),
    ln_of: &dyn Fn(&T) -> f64,
    reach: &dyn Fn(&T) -> bool,
    accept: &dyn Fn(&T) -> bool,
    seen: &mut dyn FnMut(&T),
) -> (T, bool, u32) {
    let mut trials = 1u32;
    if reach(&natural) {
        return (natural, true, trials);
    }
    let mut chosen = natural;
    let mut steps: Vec<f64> = [0.35, 0.7, 1.4, 2.8, 5.6, 9.2] // ledger: CLM-151
        .into_iter()
        .filter(|&m| m < cap)
        .collect();
    if cap > 0.0 && cap < 9.2 {
        steps.push(cap);
    }
    let mut lo = 0.0;
    let mut best: Option<T> = None;
    let bracket: Vec<T> = steps.into_par_iter().map(|m| trial(dir * m)).collect();
    for t in bracket {
        trials += 1;
        seen(&t);
        if reach(&t) {
            best = Some(t);
            break;
        }
        lo = ln_of(&t).abs();
        chosen = t;
    }
    let mut reached = false;
    if let Some(mut b) = best {
        for _ in 0..4 {
            let mid = 0.5 * (lo + ln_of(&b).abs());
            let t = trial(dir * mid);
            trials += 1;
            seen(&t);
            if reach(&t) {
                b = t;
            } else {
                lo = mid;
                chosen = t;
            }
        }
        if accept(&b) {
            reached = true;
            chosen = b;
        }
    }
    (chosen, reached, trials)
}

/// The climate stage.
pub struct ClimateStage {
    preset: PlanetPreset,
    p: ClimateParams,
    total: u64,
    s: Saved,
    timings: BTreeMap<&'static str, f64>,
    cells: CellsAccel,
}

/// Where the snapshots' cells loop runs (`gpu.rs`; `docs/design/gpu-compute.md`): the device is
/// found at the first snapshot under the session's compute mode and dropped after a failure; the
/// counts go to the run's notes (`compute`). The device is not saved (a resumed run looks again);
/// the counts are (`CellsCount`), so a resumed run's notes are a whole run's.
#[derive(Default)]
struct CellsAccel {
    probed: bool,
    dev: Option<Arc<dyn snapshot::CellsDevice>>,
    device: Option<String>,
    kernel_ms: f64,
    failed: Option<String>,
}

/// The snapshots whose cells ran on the device and on the CPU, and the device's chunks,
/// dispatches, bytes each way and largest memory, over the run.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct CellsCount {
    on_device: u64,
    on_cpu: u64,
    chunks: u64,
    dispatches: u64,
    bytes_up: u64,
    bytes_down: u64,
    device_bytes: u64,
}

impl CellsAccel {
    /// The device for a snapshot on a `w × h` grid, or None for the CPU: only the per-cell
    /// climate with the belts' rain runs on one, and only when its host arrays fit the cap.
    fn get(
        &mut self,
        ctx: &StageContext<'_>,
        p: &ClimateParams,
        w: usize,
        h: usize,
        monthly: bool,
    ) -> Option<Arc<dyn snapshot::CellsDevice>> {
        if !self.probed {
            self.probed = true;
            if p.model == ClimateModel::PerCell && p.rain_model == RainModel::Belts {
                self.dev = cells_accel(ctx);
                self.device = self.dev.as_ref().map(|d| d.name());
            }
        }
        let dev = self.dev.clone()?;
        ctx.reserve(dev.host_bytes(w, h, monthly)).ok()?;
        Some(dev)
    }

    /// Counts where a snapshot's cells ran; after a device error the rest of the run is on the
    /// CPU.
    fn ran(&mut self, ctx: &StageContext<'_>, n: &mut CellsCount, run: &snapshot::CellsRun) {
        match run {
            snapshot::CellsRun::Cpu => n.on_cpu += 1,
            snapshot::CellsRun::Device(st) => {
                n.on_device += 1;
                n.chunks += u64::from(st.chunks);
                n.dispatches += u64::from(st.dispatches);
                n.bytes_up += st.bytes_up;
                n.bytes_down += st.bytes_down;
                n.device_bytes = n.device_bytes.max(st.device_bytes);
                self.kernel_ms += st.kernel_ms;
            }
            snapshot::CellsRun::Failed(e) => {
                n.on_cpu += 1;
                ctx.log.push(
                    crate::log::Level::Warn,
                    format!("climate: the cells failed on the GPU, running on the CPU ({e})"),
                );
                self.failed = Some(e.clone());
                self.dev = None;
            }
        }
    }

    fn note(&self, n: &CellsCount) -> serde_json::Value {
        json!({
            "cells_device": self.device,
            "cells_on_device": n.on_device,
            "cells_on_cpu": n.on_cpu,
            "cells_chunks": n.chunks,
            "cells_dispatches": n.dispatches,
            "cells_bytes_up": n.bytes_up,
            "cells_bytes_down": n.bytes_down,
            "cells_device_bytes": n.device_bytes,
            "cells_kernel_ms": self.kernel_ms,
            "cells_failed": self.failed,
        })
    }
}

/// The cells loop's device under the session's compute mode (`compute.rs`).
#[cfg(feature = "gpu")]
fn cells_accel(ctx: &StageContext<'_>) -> Option<Arc<dyn snapshot::CellsDevice>> {
    let g = crate::compute::device(ctx.compute, ctx.log, "climate:cells")?;
    match gpu::CellsGpu::new(g) {
        Ok(c) => Some(Arc::new(c)),
        Err(e) => {
            ctx.log.push(
                crate::log::Level::Warn,
                format!("climate: the GPU kernel did not build, the cells run on the CPU ({e})"),
            );
            None
        }
    }
}

#[cfg(not(feature = "gpu"))]
fn cells_accel(ctx: &StageContext<'_>) -> Option<Arc<dyn snapshot::CellsDevice>> {
    crate::compute::warn_without_gpu(ctx.compute, ctx.log);
    None
}

/// Memory and time estimate: the group's (surface or plates) plus ~150 B per L1 cell (18 fields,
/// eight orographic rasters, an FFT band), ~96 B more for the last snapshot's 24 monthly fields
/// (P1-25), and ~2 µs per cell per snapshot plus 0.3 µs per cell per step (the advection between
/// snapshots and the ice), plus ~5 ms per fine-clock step and, in the ice-age window, ~0.05 µs per
/// cell per clock step.
pub fn estimate(p: &PlanetPreset) -> StageEstimate {
    let c = p.climate.clone().unwrap_or_default();
    let circ = u64::from(p.scale.circumference_blocks);
    let r = u64::from(p.resolution.l1_r);
    let cells = (circ / r) * (circ / r / 2);
    let group = if p.surface.is_some() {
        crate::surface::estimate(p)
    } else {
        crate::plates::estimate(p)
    };
    let steps = p.plate_steps() as f64;
    let snaps = (p.run_myr() / c.snapshot_every_myr).ceil() + 2.0;
    let clock = p.run_myr() * 1000.0 / c.carbon_step_kyr;
    let window = c.orbital_window_myr.min(p.run_myr()) * 1000.0 / c.orbital_step_kyr;
    StageEstimate {
        bytes: group.bytes + cells * BYTES_PER_CELL,
        seconds: cells as f64 * (snaps * 2.0e-6 + steps * 0.3e-6 + window * 0.05e-6)
            + (clock + window) * 5.0e-3,
        history_bytes: group.history_bytes,
    }
}

/// Every plate's Ω for the motion after the stored step; plates born since move with their
/// parent's (the surface does the same, `surface/mod.rs`).
fn omega_for(prev: &[[f64; 3]], parents: &[Option<u32>]) -> Vec<[f64; 3]> {
    (0..parents.len())
        .map(|mut k| {
            for _ in 0..64 {
                if k < prev.len() {
                    return prev[k];
                }
                match parents[k] {
                    Some(pk) => k = pk as usize,
                    None => break,
                }
            }
            [0.0; 3]
        })
        .collect()
}

/// A Köppen class for a land cell whose carried class is the sea's (it rose from the sea between
/// snapshots): a sinusoidal year between its coldest and warmest month and even precipitation.
fn fallback_koppen(t_warm: f64, t_cold: f64, precip: f64) -> u8 {
    let t: [f64; 12] = std::array::from_fn(|m| {
        let x = (std::f64::consts::TAU * (m as f64 - 0.5) / 12.0).cos();
        0.5 * (t_warm + t_cold) - 0.5 * (t_warm - t_cold) * x
    });
    koppen::classify(&t, &[precip / 12.0; 12])
}

/// The grid's geometry in the climate's Earth-normalized lengths.
struct Geometry {
    w: usize,
    h: usize,
    /// True Earth-normalized cell area of every row, km².
    area: Vec<f64>,
    /// True Earth-normalized cell width of every row and height, m.
    sx: Vec<f64>,
    sy: Vec<f64>,
    /// EBM band of every row.
    band: Vec<usize>,
    bands: usize,
}

impl Geometry {
    fn cell_band(&self, c: usize) -> usize {
        self.band[c / self.w]
    }

    /// Ice volume of a thickness field, Earth-normalized km³.
    fn volume_km3(&self, ice: &[f32]) -> f64 {
        // The rows' sums side by side (P1-19), added in row order.
        let rows: Vec<f64> = (0..self.h)
            .into_par_iter()
            .map(|j| {
                let row: f64 = ice[j * self.w..(j + 1) * self.w]
                    .iter()
                    .map(|&x| f64::from(x))
                    .sum();
                row * self.area[j] / 1000.0
            })
            .collect();
        rows.iter().sum()
    }

    /// Area of the cells `pick` selects, Earth-normalized km².
    fn area_where(&self, pick: &(dyn Fn(usize) -> bool + Sync)) -> f64 {
        // The rows' counts side by side (P1-19), added in row order.
        let rows: Vec<f64> = (0..self.h)
            .into_par_iter()
            .map(|j| (0..self.w).filter(|&i| pick(j * self.w + i)).count() as f64 * self.area[j])
            .collect();
        rows.iter().sum()
    }

    /// Of the land cells: each band's share under ice (thickness ≥ `ICE_MIN_M`), the ice-covered
    /// area (km²) and the ice volume (km³), rows in parallel and summed in row order.
    fn ice_stats(&self, ice: &[f32], land: &[bool]) -> (Vec<f64>, f64, f64) {
        let rows: Vec<(usize, usize, f64, f64)> = (0..self.h)
            .into_par_iter()
            .map(|j| {
                let (mut on, mut all, mut area, mut vol) = (0usize, 0usize, 0.0, 0.0);
                for i in 0..self.w {
                    let c = j * self.w + i;
                    if !land[c] {
                        continue;
                    }
                    all += 1;
                    vol += f64::from(ice[c]);
                    if ice[c] >= ICE_MIN_M {
                        on += 1;
                        area += 1.0;
                    }
                }
                (on, all, area * self.area[j], vol * self.area[j] / 1000.0)
            })
            .collect();
        let mut num = vec![0usize; self.bands];
        let mut den = vec![0usize; self.bands];
        let (mut area, mut vol) = (0.0, 0.0);
        for (j, (on, all, a, v)) in rows.into_iter().enumerate() {
            num[self.band[j]] += on;
            den[self.band[j]] += all;
            area += a;
            vol += v;
        }
        let share = num
            .iter()
            .zip(&den)
            .map(|(&a, &d)| if d > 0 { a as f64 / d as f64 } else { 0.0 })
            .collect();
        (share, area, vol)
    }

    /// The ice-sheet cells among the cells `ice` selects: every continuous ice body (cells sharing
    /// an edge, the rows wrapping east to west) that covers at least `ICE_SHEET_MIN_KM2`. The rest
    /// of the ice is mountain ice.
    fn sheet_cells(&self, ice: &(dyn Fn(usize) -> bool + Sync)) -> Vec<bool> {
        let (w, h) = (self.w, self.h);
        let on: Vec<bool> = (0..w * h).into_par_iter().map(ice).collect();
        let mut seen = vec![false; w * h];
        let mut sheet = vec![false; w * h];
        let (mut stack, mut body) = (Vec::new(), Vec::new());
        for (c0, &o) in on.iter().enumerate() {
            if !o || seen[c0] {
                continue;
            }
            seen[c0] = true;
            stack.push(c0);
            body.clear();
            let mut area = 0.0;
            while let Some(c) = stack.pop() {
                body.push(c);
                let (i, j) = (c % w, c / w);
                area += self.area[j];
                let next = [
                    Some(j * w + (i + 1) % w),
                    Some(j * w + (i + w - 1) % w),
                    (j > 0).then(|| c - w),
                    (j + 1 < h).then_some(c + w),
                ];
                for d in next.into_iter().flatten() {
                    if on[d] && !seen[d] {
                        seen[d] = true;
                        stack.push(d);
                    }
                }
            }
            if area >= ICE_SHEET_MIN_KM2 {
                for &c in &body {
                    sheet[c] = true;
                }
            }
        }
        sheet
    }

    /// Where the sheets lie (`SheetProfile`): the land's area in each height and latitude class
    /// and the sheets' in it, rows side by side and added in row order.
    fn sheet_profile(
        &self,
        sheet: &[bool],
        land: &[bool],
        above_sea: &(dyn Fn(usize) -> f64 + Sync),
        lat: &[f64],
    ) -> SheetProfile {
        let (nh, nl) = (HEIGHT_CLASSES_M.len(), LAT_CLASSES_DEG.len());
        let class = |edges: &[f64], x: f64| edges.iter().rposition(|&e| x >= e).unwrap_or(0);
        let rows: Vec<Vec<f64>> = (0..self.h)
            .into_par_iter()
            .map(|j| {
                // [height land, height sheet, latitude land, latitude sheet], cell counts.
                let mut r = vec![0.0; 2 * (nh + nl)];
                let lc = class(&LAT_CLASSES_DEG, lat[j].abs());
                for c in j * self.w..(j + 1) * self.w {
                    if !land[c] {
                        continue;
                    }
                    let hc = class(&HEIGHT_CLASSES_M, above_sea(c));
                    r[hc] += 1.0;
                    r[2 * nh + lc] += 1.0;
                    if sheet[c] {
                        r[nh + hc] += 1.0;
                        r[2 * nh + nl + lc] += 1.0;
                    }
                }
                r.iter_mut().for_each(|x| *x *= self.area[j]);
                r
            })
            .collect();
        let mut t = vec![0.0; 2 * (nh + nl)];
        for r in &rows {
            t.iter_mut().zip(r).for_each(|(a, b)| *a += b);
        }
        let all: f64 = t[..nh].iter().sum();
        let share = |n: f64, d: f64| if d > 0.0 { n / d } else { 0.0 };
        SheetProfile {
            height_land: (0..nh).map(|i| share(t[i], all)).collect(),
            height_sheet: (0..nh).map(|i| share(t[nh + i], t[i])).collect(),
            lat_land: (0..nl).map(|i| share(t[2 * nh + i], all)).collect(),
            lat_sheet: (0..nl)
                .map(|i| share(t[2 * nh + nl + i], t[2 * nh + i]))
                .collect(),
        }
    }

    /// Each band's share of `pick` among the cells `of` selects (0 where there are none).
    fn band_share(
        &self,
        pick: &(dyn Fn(usize) -> bool + Sync),
        of: &(dyn Fn(usize) -> bool + Sync),
    ) -> Vec<f64> {
        let mut num = vec![0usize; self.bands];
        let mut den = vec![0usize; self.bands];
        // Each row's counts side by side (P1-19); whole numbers, so any order adds alike.
        let rows: Vec<(usize, usize)> = (0..self.h)
            .into_par_iter()
            .map(|j| {
                let (mut d, mut n) = (0usize, 0usize);
                for c in j * self.w..(j + 1) * self.w {
                    if of(c) {
                        d += 1;
                        if pick(c) {
                            n += 1;
                        }
                    }
                }
                (d, n)
            })
            .collect();
        for (j, &(d, n)) in rows.iter().enumerate() {
            den[self.band[j]] += d;
            num[self.band[j]] += n;
        }
        num.iter()
            .zip(&den)
            .map(|(&a, &d)| if d > 0 { a as f64 / d as f64 } else { 0.0 })
            .collect()
    }
}

/// Experiment toggle (the 2026-10-18 CO₂ trace; off unless set): the weathering kernel's kinetics
/// at each cell's sea-level temperature, as if high ground were not cooled by the lapse rate.
pub const KERNEL_SEA_T_ENV: &str = "DEEPTIME_EXP_KERNEL_SEA_T";

/// What the land's cells add up to at one step's end climate: the weathering kernel per band, and
/// the area-weighted sums the ice stage's notes read (land area, runoff, temperature, the kernel
/// at the cell's temperature and at its sea-level temperature, and the high ground's share).
struct LandTerms {
    kernel: carbon::Kernel,
    /// Land area, km².
    ra: f64,
    /// Area × runoff, m·km²/yr.
    rs: f64,
    /// Area × temperature.
    ta: f64,
    /// The kernel at the cell's temperature, in all and over ground above 2,000 m.
    kt: f64,
    kh: f64,
    /// Land area above 2,000 m, km².
    ah: f64,
    /// The kernel at the cell's sea-level temperature (its lapse cooling put back).
    ks: f64,
}

/// One land cell's terms (P1-19: found once, side by side, and added in cell order after).
#[derive(Clone, Copy, Default)]
struct CellTerms {
    /// The runoff, m/yr.
    runoff: f64,
    /// The kernel at the cell's temperature and at its sea-level temperature.
    kc: f64,
    ks: f64,
    /// The supply the cell adds, when the weathering is supply-limited.
    supply: f64,
}

/// The weathering kernel of a step's climate: each band's Σ area·runoff·e^(k_T(T − 15)) over its
/// land, and each band's EBM land temperature for it; and with it the ice stage's sums (P1-19,
/// #16: the cells' terms are found side by side, each once where the kernel and the notes' loop
/// each found them before, and added in cell order, so the sums are the serial loops' to the bit).
///
/// The runoff is P − ET (Fu's Budyko curve, the surface stage's), with the potential evaporation
/// of the cell's temperature *less the planet's global warming against `hydro_ref_c`*: the
/// stand-in PET (60 mm/°C) grows ~6 %/K at 15 °C while the model's precipitation grows 2 %/K
/// (Held & Soden 2006), so evaluated at the warmed temperature it made a warmer planet's land
/// drier and weathering weaker — a positive feedback inside the thermostat that ran a land-poor
/// planet (the GameTest planet, 9 % land) to 8,500 ppm and 27 °C. Held at the reference climate's
/// evaporation, the runoff keeps its spatial pattern and rises with the precipitation, as global
/// runoff does with warming (GEOCARB's ~3.8 %/K, Berner & Kothavala 2001).
///
/// `h`, `sea`: the elevation and the sea level, read by the experiment toggle
/// [`KERNEL_SEA_T_ENV`] (the kinetics at the cell's sea-level temperature, its lapse cooling put
/// back; the 2026-10-18 CO₂ trace) and by the high-ground and sea-level sums.
#[allow(clippy::too_many_arguments)]
fn land_terms(
    g: &Geometry,
    land: &[bool],
    t_mean: &[f32],
    precip: &[f32],
    er: &EbmResult,
    p: &ClimateParams,
    erosion: Option<&[f32]>,
    (h, sea): (&[f64], f64),
) -> LandTerms {
    let k_t = p.weathering_temp_per_k;
    let warming = er.t_global - p.hydro_ref_c;
    let sea_t = std::env::var_os(KERNEL_SEA_T_ENV).is_some();
    let gamma = p.lapse_c_per_km / 1000.0;
    let limited = p.weathering_supply && erosion.is_some();
    // (A row's terms at a time, rows side by side; the cells nobody reads stay zero.)
    let cells: Vec<Vec<CellTerms>> = (0..g.h)
        .into_par_iter()
        .map(|j| {
            let a = g.area[j];
            (j * g.w..(j + 1) * g.w)
                .map(|c| {
                    if !land[c] {
                        return CellTerms::default();
                    }
                    let t = f64::from(t_mean[c]);
                    let pr = f64::from(precip[c]);
                    let pet = (60.0 * (t - warming)).max(0.0);
                    let runoff = (pr - crate::surface::climate::fu_et(pr, pet)).max(0.0) / 1000.0;
                    let kc = carbon::cell_kernel(a, runoff, t, k_t);
                    // The cell's sea-level temperature: the same value where it stands at the sea.
                    let up = (h[c] - sea).max(0.0);
                    let ks = if up > 0.0 {
                        carbon::cell_kernel(a, runoff, t + gamma * up, k_t)
                    } else {
                        kc
                    };
                    let supply = match (limited, erosion) {
                        (true, Some(e)) => {
                            carbon::cell_supply(
                                a,
                                f64::from(e[c]).max(0.0),
                                p.weathering_min_denudation_m_myr,
                                p.weathering_rock_mol_kg,
                            ) * p.weathering_supply_scale
                        }
                        _ => 0.0,
                    };
                    CellTerms {
                        runoff,
                        kc,
                        ks,
                        supply,
                    }
                })
                .collect()
        })
        .collect();
    let mut band = vec![0.0; g.bands];
    let mut supply = vec![0.0; g.bands];
    let (mut ra, mut rs, mut ta) = (0.0, 0.0, 0.0);
    let (mut kt, mut kh, mut ah, mut ks) = (0.0, 0.0, 0.0, 0.0);
    for (j, row) in cells.iter().enumerate() {
        let (b, a) = (g.band[j], g.area[j]);
        for (i, x) in row.iter().enumerate() {
            let c = j * g.w + i;
            if !land[c] {
                continue;
            }
            ra += a;
            rs += a * x.runoff;
            ta += a * f64::from(t_mean[c]);
            kt += x.kc;
            ks += x.ks;
            if h[c] - sea > 2000.0 {
                kh += x.kc;
                ah += a;
            }
            band[b] += if sea_t { x.ks } else { x.kc };
            if limited {
                supply[b] += x.supply;
            }
        }
    }
    LandTerms {
        kernel: carbon::Kernel {
            band,
            ref_t_land: clock::land_temps(er).0,
            supply: if limited { supply } else { Vec::new() },
        },
        ra,
        rs,
        ta,
        kt,
        kh,
        ah,
        ks,
    }
}

/// The weathering kernel alone (see [`land_terms`]).
#[allow(clippy::too_many_arguments)]
fn kernel_of(
    g: &Geometry,
    land: &[bool],
    t_mean: &[f32],
    precip: &[f32],
    er: &EbmResult,
    p: &ClimateParams,
    erosion: Option<&[f32]>,
    hs: (&[f64], f64),
) -> carbon::Kernel {
    land_terms(g, land, t_mean, precip, er, p, erosion, hs).kernel
}

impl ClimateStage {
    pub fn new(preset: PlanetPreset) -> ClimateStage {
        let mut p = preset.climate.clone().unwrap_or_default();
        // The ice-age window and the orbit are measured from the run's end (its stop).
        p.run_end_years = preset.t_end_years();
        // The realistic ("free") choice is no steering at all (Ben, decisions log 2026-10-06,
        // "One switch for both"): with the nudges switched off the present-day adjustment is off
        // too, and the history ends as simulated. A preset without a `steering` section keeps its
        // `present_climate`.
        if preset.steering.is_some_and(|s| !s.enabled) {
            p.present_climate = PresentClimate::Natural;
        }
        let total = preset.plate_steps();
        ClimateStage {
            preset,
            p,
            total,
            s: Saved::default(),
            timings: BTreeMap::new(),
            cells: CellsAccel::default(),
        }
    }

    fn time(&mut self, what: &'static str, since: Instant) {
        *self.timings.entry(what).or_insert(0.0) += since.elapsed().as_secs_f64() * 1000.0;
    }

    /// The end-weighted nudges, when the preset has them on (`steering`).
    fn steering(&self) -> Option<crate::steering::SteeringParams> {
        self.preset.steering.filter(|s| s.enabled)
    }

    /// The surface section's sea-level curve at `t_myr`.
    fn curve_sea_level(&self, t_myr: f64) -> f64 {
        self.preset
            .surface
            .as_ref()
            .map_or(0.0, |s| s.sea_level.at(t_myr))
    }

    fn earth_scale(&self) -> f64 {
        self.preset.scale.radius_m() / (self.preset.tectonics.model_radius_km * 1000.0)
    }

    fn planet(&self, ppm: f64, t_years: f64) -> Planet {
        Planet {
            axial_tilt_deg: self.preset.planet.axial_tilt_deg,
            star_flux_rel: self.preset.planet.star_flux_rel,
            day_length_h: self.preset.planet.day_length_h,
            earth_scale: self.earth_scale(),
            m_per_block_h: self.preset.scale.m_per_block_h,
            co2_ppm: ppm,
            sun_rel: clock::sun_at(&self.p, t_years),
            orbit: clock::orbit_at(&self.p, self.preset.planet.axial_tilt_deg, t_years),
        }
    }

    /// CO₂ now, ppm.
    fn ppm(&self) -> f64 {
        if self.s.clock.started {
            self.s.clock.ppm()
        } else if self.p.initial_co2_ppm > 0.0 {
            self.p.initial_co2_ppm
        } else {
            carbon::CO2_REF_PPM
        }
    }

    /// The plates' activity this step: sea-floor production (young oceanic crust) and the ocean
    /// floor consumed at trenches (production minus the ocean floor's growth since the last step,
    /// smoothed), both Earth-normalized km²/yr.
    fn activity(
        &mut self,
        world: &WorldState,
        g: &Geometry,
        n: usize,
        dt_yr: f64,
    ) -> carbon::Activity {
        let Some(age) = world.fields.get("crust_age").filter(|f| f.data.len() == n) else {
            return carbon::Activity {
                ridge_km2_yr: carbon::RIDGE_EARTH_KM2_YR,
                subduction_km2_yr: carbon::SUBDUCTION_EARTH_KM2_YR,
            };
        };
        let (mut ridge, mut ocean) = (0.0, 0.0);
        for j in 0..g.h {
            for i in 0..g.w {
                let a = age.data[j * g.w + i];
                if a >= 0.0 {
                    ocean += g.area[j];
                    if a < carbon::RIDGE_WINDOW_MYR as f32 {
                        ridge += g.area[j];
                    }
                }
            }
        }
        let ridge = ridge / (carbon::RIDGE_WINDOW_MYR * 1.0e6);
        let raw = match self.s.ocean_km2 {
            Some(prev) => ridge - (ocean - prev) / dt_yr.max(1.0),
            None => ridge,
        };
        let k = 1.0 - (-dt_yr / carbon::SUBDUCTION_SMOOTH_YEARS).exp();
        let sub = match self.s.subduction_km2_yr {
            Some(prev) => prev + (raw - prev) * k,
            None => ridge,
        }
        .max(0.0);
        self.s.ocean_km2 = Some(ocean);
        self.s.subduction_km2_yr = Some(sub);
        carbon::Activity {
            ridge_km2_yr: ridge,
            subduction_km2_yr: sub,
        }
    }

    /// Large igneous provinces the plates erupted since the last look (their CO₂, spread over
    /// `carbon::LIP_YEARS`).
    fn new_provinces(&mut self, world: &WorldState) {
        let seen = (self.s.clock.events_seen as usize).min(world.events.len());
        let f2 = self.earth_scale().powi(2).max(1e-12);
        for e in &world.events[seen..] {
            if e.kind != crate::plates::events::LARGE_IGNEOUS_PROVINCE {
                continue;
            }
            if let Some(deeptime_format::ParamValue::Float(v)) = e.params.get("volume_km3") {
                let mol = v / f2 * carbon::LIP_MOL_PER_KM3;
                self.s.clock.lips.push((e.t_start, carbon::LIP_YEARS, mol));
            }
        }
        self.s.clock.events_seen = world.events.len() as u64;
    }

    fn advance(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        let grid = GridDesc::for_planet(
            "L1",
            self.preset.scale.circumference_blocks,
            self.preset.resolution.l1_r,
        )?;
        let n = grid.cells();
        let (w, hgt) = (grid.w as usize, grid.h as usize);
        let dx = f64::from(grid.r) * self.preset.scale.m_per_block_h;
        let d = Dims::new(w, hgt, dx, true);
        let plates = world.plates.as_ref().ok_or_else(|| {
            CoreError::InvalidState("the climate stage runs after a plate step".into())
        })?;
        let t_end_myr = plates.t_myr;
        let dt_myr = self.preset.dt_myr();
        let t1 = t_end_myr * 1.0e6;
        let t0 = t1 - dt_myr * 1.0e6;
        let omega_now: Vec<[f64; 3]> = plates
            .plates
            .iter()
            .map(|p| if p.alive { p.omega } else { [0.0; 3] })
            .collect();
        let parents: Vec<Option<u32>> = plates.plates.iter().map(|p| p.parent).collect();
        let need = |name: &str| -> Result<&Vec<f32>, CoreError> {
            world
                .fields
                .get(name)
                .filter(|f| f.data.len() == n)
                .map(|f| &f.data)
                .ok_or_else(|| {
                    CoreError::InvalidState(format!(
                        "the climate stage needs the plates' L1 field {name} every step \
                         (tectonics.l1_every_steps = 1)"
                    ))
                })
        };
        if self.s.done == 0 {
            ctx.reserve(n as u64 * BYTES_PER_CELL)?;
        }
        let p = self.p.clone();
        let es = self.earth_scale();
        let bands = p.ebm_bands as usize;
        let lat: Vec<f64> = (0..hgt).map(|j| lat_of_row(&grid, j as u32)).collect();
        let geo = Geometry {
            w,
            h: hgt,
            area: (0..hgt)
                .map(|j| {
                    cell_area_km2(&grid, j as u32, self.preset.scale.m_per_block_h) / (es * es)
                })
                .collect(),
            sx: (0..hgt)
                .map(|j| {
                    grid.true_cell_m(j as u32, self.preset.scale.m_per_block_h)
                        .0
                        / es
                })
                .collect(),
            // On the Lambert grid a row's cells are r·m / cos φ high on the sphere: the rows are
            // not evenly spaced along a column (docs/design/equal-area-projection.md §4).
            sy: (0..hgt)
                .map(|j| {
                    grid.true_cell_m(j as u32, self.preset.scale.m_per_block_h)
                        .1
                        / es
                })
                .collect(),
            band: lat.iter().map(|&l| snapshot::band_of(l, bands)).collect(),
            bands,
        };
        let gamma = p.lapse_c_per_km / 1000.0;
        let tau = p.ice_yield_stress_kpa * 1000.0;
        let cap = p.ice_max_thickness_m;
        // The ice takes the planet's gravity (Nye's profile and the sheet's flow).
        let grav = self.preset.planet.gravity_m_s2;

        // Where the crust went this plate step, and the ground the surface will erode.
        let started = Instant::now();
        let pid = need("plates/plate_id")?;
        let src =
            self.s.omega.as_ref().map(|prev| {
                advect::sources(&grid, pid, &omega_for(prev, &parents), dt_myr * 1.0e6)
            });
        // The surface's carry blur, kept per Myr at its reference step (`advect::blur_owed`).
        let blur_ref = self.preset.surface.as_ref().map_or_else(
            || crate::surface::params::Numerics::default().carry_blur_ref_myr,
            |s| s.numerics.carry_blur_ref_myr,
        ) * 1.0e6;
        let owed = src
            .as_ref()
            .and_then(|src| advect::blur_owed(w, src, dt_myr * 1.0e6, blur_ref));
        let mut h: Vec<f64> = need("plates/elevation_m")?
            .iter()
            .map(|&z| f64::from(z))
            .collect();
        if let (Some(st), Some(src), Some(prev)) =
            (world.surface.as_ref(), src.as_ref(), self.s.omega.as_ref())
        {
            // The ground the surface stage will start from: its own rule (`advect::carried_elevation`).
            let ids = |name: &str| {
                world
                    .fields
                    .get(name)
                    .filter(|f| f.data.len() == n)
                    .map(|f| f.data.as_slice())
            };
            let (carried, _) = advect::carried_elevation(&advect::Carry {
                w,
                h: hgt,
                src,
                prev_elevation: &st.elevation_m,
                prev_plate: ids(crate::surface::field::CRUST_PLATE),
                prev_group: ids(crate::surface::field::CRUST_GROUP),
                plate: pid,
                group: ids("plates/group_id"),
                plate_elevation: need("plates/elevation_m")?,
                uplift_mm_yr: need("plates/uplift_mm_yr")?,
                parents: &parents,
                known_plates: prev.len(),
                dt_yr: dt_myr * 1.0e6,
                blur_ref_yr: blur_ref,
            });
            h.par_iter_mut()
                .zip(carried.par_iter())
                .for_each(|(z, &c)| *z = f64::from(c));
        }
        // The fields carried with the plates to where their crust is now.
        let carry = |world: &WorldState, name: &str| -> Option<Vec<f32>> {
            let v = &world.fields.get(name).filter(|f| f.data.len() == n)?.data;
            Some(match src.as_ref() {
                Some(src) => {
                    let mut out: Vec<f32> = src
                        .par_iter()
                        .map(|&[fi, fj]| advect::bilinear(w, hgt, v, fi, fj))
                        .collect();
                    if let Some(var) = &owed {
                        advect::blur(w, hgt, var, &mut out, |_, _| true);
                    }
                    out
                }
                None => v.clone(),
            })
        };
        let mut ela_all = carry(world, field::ELA_ALL);
        let ice_prev = carry(world, field::ICE_THICKNESS);
        let ice_max_prev = carry(world, field::ICE_MAX_THICKNESS);
        let sea_prev = self
            .s
            .sea_level_m
            .unwrap_or_else(|| self.curve_sea_level(t0 / 1.0e6));
        self.time("topography", started);

        let k = self.s.done + 1;
        let last = k >= self.total;
        let every = ((p.snapshot_every_myr / dt_myr).round() as u64).max(1);
        let have_anomalies = [
            field::D_MEAN,
            field::D_WARM,
            field::D_COLD,
            field::D_LN_PRECIP,
            field::ELA_ALL,
            field::KOPPEN,
            field::ARIDITY,
        ]
        .iter()
        .all(|f| world.fields.get(*f).is_some_and(|x| x.data.len() == n));
        let mut snap = k == 1
            || last
            || k.is_multiple_of(every)
            || src.is_none()
            || !have_anomalies
            || self.s.refs.t_mean.len() != hgt;
        let put = |world: &mut WorldState, name: &str, data: Vec<f32>| {
            world.fields.insert(
                name.to_owned(),
                Field {
                    grid: grid.clone(),
                    data,
                },
            );
        };
        let land_at = |sea: f64| -> Vec<bool> { h.iter().map(|&z| z > sea).collect() };
        let band_land = |land: &[bool]| -> Vec<f64> {
            snapshot::band_means(&lat, w, bands, &|c| if land[c] { 1.0 } else { 0.0 })
        };

        // 0. The first step: a snapshot of the starting world for the weathering and the
        //    equilibrium line, then the carbon cycle in balance (or at the preset's CO₂).
        let land0 = land_at(sea_prev);
        let land_frac0 = band_land(&land0);
        let activity = self.activity(world, &geo, n, dt_myr * 1.0e6);
        let mut degassing = carbon::degassing(&activity, p.degassing_scale);
        if self.steering().is_some() && self.s.steer.ln_outgas != 0.0 {
            // The nudge on volcanic outgassing (`steering`): e^x of the degassing; x is 0 before
            // the tail.
            degassing *= self.s.steer.ln_outgas.exp();
        }
        self.new_provinces(world);
        if !self.s.clock.started {
            let started = Instant::now();
            let ppm0 = if p.initial_co2_ppm > 0.0 {
                p.initial_co2_ppm
            } else {
                carbon::CO2_REF_PPM
            };
            let inp = snapshot::Input {
                grid: &grid,
                d,
                lat: &lat,
                h: &h,
                sea: sea_prev,
                p: &p,
                planet: self.planet(ppm0, t0),
                // The starting snapshot only seeds the weathering and the equilibrium line; the
                // bake's months come from the last snapshot.
                monthly: false,
                ice_prev: None,
                koppen_prev: None,
                correction: &percell::Correction::default(),
            };
            let mut ebm0 = EbmState::default();
            let dev = self.cells.get(ctx, &p, w, hgt, false);
            let s0 = snapshot::compute_on(&inp, &mut ebm0, &[], &mut self.s.cell, dev.as_deref());
            self.cells.ran(ctx, &mut self.s.cells, &s0.cells_on);
            self.s.correction = s0.correction.clone();
            let er0 = s0.ebm.clone().expect("a snapshot keeps its EBM year");
            let (mean0, summer0) = clock::land_temps(&er0);
            // No erosion yet: the first step weathers without a supply limit.
            self.s.clock.kernel = kernel_of(
                &geo,
                &land0,
                &s0.t_mean,
                &s0.precip,
                &er0,
                &p,
                None,
                (&h, sea_prev),
            );
            self.s.clock.snap_summer = summer0;
            self.s.clock.snap_t_global = er0.t_global;
            self.s.clock.sheet = s0.band_sheet.clone();
            self.s.clock.t_land = mean0;
            if ela_all.is_none() {
                ela_all = Some(s0.ela.clone());
            }
            let ppm = if p.carbon_cycle && p.initial_co2_ppm <= 0.0 {
                let (ppm, state) = clock::balance(
                    &p,
                    self.preset.planet.axial_tilt_deg,
                    self.preset.planet.star_flux_rel,
                    t0,
                    &land_frac0,
                    &self.s.clock.sheet,
                    &self.s.clock.kernel,
                    degassing,
                    activity.ridge_km2_yr / carbon::RIDGE_EARTH_KM2_YR,
                    &self.s.correction,
                );
                self.s.ebm = state;
                ppm
            } else {
                self.s.ebm = ebm0;
                ppm0
            };
            self.s.clock.carbon = carbon::Carbon::at_ppm(ppm);
            self.s.clock.started = true;
            self.time("carbon_start", started);
        }

        // 1. The fine clock over the plate step.
        let started = Instant::now();
        let ela0 = ela_all.clone().unwrap_or_else(|| vec![1.0e6; n]);
        let mu0: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|c| {
                if land0[c] {
                    (h[c] - sea_prev - f64::from(ela0[c])) as f32
                } else {
                    f32::NAN
                }
            })
            .collect();
        let margins = ice::BandMargins::new(&mu0, &|c| geo.cell_band(c), bands);
        let window_step = clock::in_window(&p, t1);
        // The lagged ice sheets of the window: their thickness, the window's maximum, and the time
        // each cell spent under ice this step.
        let hs: Vec<f32> = ice_prev.clone().unwrap_or_else(|| vec![0.0; n]);
        let hmax: Vec<f32> = if clock::in_window(&p, t0) {
            ice_max_prev.clone().unwrap_or_else(|| hs.clone())
        } else {
            hs.clone()
        };
        // The ice-sheet dynamics' coarse grid and what drives it: the bed (without the ice),
        // each coarse cell's snapshot equilibrium line and its precipitation.
        let dyn_ice = p.ice_dynamics;
        let sg = sheet::Grid::new(w, hgt, &geo.sx, &geo.sy, p.ice_grid_cells as usize);
        let sp = sheet::Params::from(&p, grav);
        let coarse_band: Vec<usize> = (0..sg.h)
            .map(|cj| geo.band[(cj * sg.k + sg.k / 2).min(hgt - 1)])
            .collect();
        let bed_fine = |sea: f64| -> Vec<f32> { h.par_iter().map(|&z| (z - sea) as f32).collect() };
        // The sheet's inputs, only where the dynamic ice reads them (P1-19, #18).
        let (bed0_c, ela0_c, precip0_c) = if dyn_ice {
            let precip_prev: Vec<f32> = world
                .fields
                .get(field::PRECIP)
                .filter(|f| f.data.len() == n)
                .map_or_else(
                    // (500 mm/yr, in m.)
                    || vec![0.5; n],
                    |f| f.data.par_iter().map(|&x| x / 1000.0).collect(),
                );
            (
                sg.coarsen(&bed_fine(sea_prev), &|_| true),
                sg.coarsen(&ela0, &|c| land0[c]),
                sg.coarsen(&precip_prev, &|c| land0[c]),
            )
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };
        if dyn_ice
            && window_step
            && (!clock::in_window(&p, t0) || self.s.sheet.thick.len() != sg.w * sg.h)
        {
            // The window begins: the sheet starts from the ice the plates carried here.
            let start = sg.coarsen(&hs, &|_| true);
            self.s.sheet = sheet::State::settled(start);
        }
        let mut ir = IceRun {
            hs,
            hmax,
            share_acc: vec![0.0f32; n],
            heq: vec![0.0; n],
            since_eq: f64::MAX,
            sheet: std::mem::take(&mut self.s.sheet),
            rows: Vec::new(),
            in_window: false,
            delta: Vec::new(),
        };
        let land0_km2 = geo.area_where(&|c| land0[c]);
        let dynamic = p.carbon_cycle || p.solar_brightening || window_step;
        let out = if dynamic {
            let band_lat: Vec<f64> = (0..bands)
                .map(|b| -90.0 + (b as f64 + 0.5) * 180.0 / bands as f64)
                .collect();
            let (tilt_deg, star_flux_rel) = (
                self.preset.planet.axial_tilt_deg,
                self.preset.planet.star_flux_rel,
            );
            let ridge_rel = activity.ridge_km2_yr / carbon::RIDGE_EARTH_KM2_YR;
            let correction = self.s.correction.clone();
            let cin = |a: f64, b: f64, nudge: Option<clock::Nudge>| clock::Input {
                p: &p,
                tilt_deg,
                star_flux_rel,
                t0: a,
                t1: b,
                land_frac: &land_frac0,
                band_lat: &band_lat,
                degassing,
                ridge_rel,
                nudge,
                correction: &correction,
            };
            let ice_step = |ir: &mut IceRun, s: &clock::IceStep<'_>| -> Vec<f64> {
                ir.in_window = s.in_window;
                if !s.in_window {
                    ir.delta.clear();
                    ir.delta.extend_from_slice(s.delta);
                    return margins.shares(s.delta);
                }
                // Equilibrium ice for this orbit (refreshed every ~5 kyr), then Imbrie's lag.
                ir.since_eq += s.dt_kyr;
                if ir.since_eq >= 5.0 - 1e-9 {
                    let mask: Vec<bool> = (0..n)
                        .into_par_iter()
                        .map(|c| {
                            mu0[c].is_finite() && f64::from(mu0[c]) > s.delta[geo.cell_band(c)]
                        })
                        .collect();
                    ir.heq = ice::thickness(w, hgt, &mask, &geo.sx, &geo.sy, tau, cap, grav);
                    ir.since_eq = 0.0;
                }
                let dtk = s.dt_kyr as f32;
                if dyn_ice {
                    // The sheet flows under this step's line; the ice is the sheet's, and the
                    // equilibrium glaciers of the fine ground wherever they are thicker (valley and
                    // plateau ice the coarse grid cannot see).
                    let ela_c: Vec<f32> = (0..sg.w * sg.h)
                        .map(|cc| ela0_c[cc] + s.delta[coarse_band[cc / sg.w]] as f32)
                        .collect();
                    let forcing = sheet::Forcing {
                        bed: &bed0_c,
                        ela: &ela_c,
                        precip: &precip0_c,
                    };
                    ir.sheet.step(&sg, &forcing, &sp, s.dt_kyr * 1000.0);
                    let fine = sg.refine(&ir.sheet.thick);
                    ir.hs
                        .par_iter_mut()
                        .zip(ir.hmax.par_iter_mut())
                        .zip(ir.share_acc.par_iter_mut())
                        .zip(ir.heq.par_iter().zip(fine.par_iter()))
                        .enumerate()
                        .for_each(|(c, (((hh, hm), sa), (&e, &sh)))| {
                            let x = if land0[c] {
                                let x = e.max(sh);
                                if x >= ICE_MIN_M { x } else { 0.0 }
                            } else {
                                0.0
                            };
                            *hh = x;
                            *hm = hm.max(x);
                            if x >= ICE_MIN_M {
                                *sa += dtk;
                            }
                        });
                } else {
                    ir.hs
                        .par_iter_mut()
                        .zip(ir.hmax.par_iter_mut())
                        .zip(ir.share_acc.par_iter_mut())
                        .zip(ir.heq.par_iter())
                        .enumerate()
                        .for_each(|(c, (((hh, hm), sa), &e))| {
                            let e = if land0[c] { e } else { 0.0 };
                            let mut x = ice::relax(
                                f64::from(*hh),
                                f64::from(e),
                                s.dt_kyr,
                                p.ice_response_kyr,
                            ) as f32;
                            if x < ICE_MIN_M && e < ICE_MIN_M {
                                x = 0.0;
                            }
                            *hh = x;
                            *hm = hm.max(x);
                            if x >= ICE_MIN_M {
                                *sa += dtk;
                            }
                        });
                }
                let (sheet, ice_a, vol) = geo.ice_stats(&ir.hs, &land0);
                ir.rows.push((ice_a / land0_km2.max(1e-9), vol));
                sheet
            };
            // The present (`present_climate`): the run's last stretch again under the smallest
            // CO₂ factor that ends in the state asked for. Everything before it is untouched.
            let mode = p.present_climate;
            let t_adj = (t1 - p.present_adjust_kyr * 1.0e3).max(t0);
            if last && mode != PresentClimate::Natural {
                let started = Instant::now();
                let first = (t_adj > t0 + 1.0).then(|| {
                    clock::run(
                        &cin(t0, t_adj, None),
                        &mut self.s.clock,
                        &mut self.s.ebm,
                        &mut |s| ice_step(&mut ir, s),
                    )
                });
                let saved = (self.s.clock.clone(), self.s.ebm.clone(), ir.clone());
                // One trial: the stretch under ln(factor) `x`; its land-ice share and snowball.
                let trial = |x: f64| -> Trial {
                    let (mut c, mut e, mut i) = saved.clone();
                    let nudge = (x != 0.0).then_some(clock::Nudge {
                        t_start: t_adj,
                        ramp_years: p.present_ramp_kyr * 1.0e3,
                        ln_factor: x,
                    });
                    let out = clock::run(&cin(t_adj, t1, nudge), &mut c, &mut e, &mut |s| {
                        ice_step(&mut i, s)
                    });
                    let land_share = |share: &[f64]| -> f64 {
                        let (mut ice, mut all) = (0.0, 0.0);
                        for (b, lat) in band_lat.iter().enumerate() {
                            let a = land_frac0[b] * lat.to_radians().cos();
                            ice += a * share.get(b).copied().unwrap_or(0.0);
                            all += a;
                        }
                        if all > 0.0 { ice / all } else { 0.0 }
                    };
                    // The ice the clock's share counted, cell by cell, and of it the sheets'.
                    let iced = |k: usize| -> bool {
                        land0[k]
                            && if i.in_window {
                                i.hs[k] >= ICE_MIN_M
                            } else {
                                i.delta
                                    .get(geo.cell_band(k))
                                    .is_some_and(|&d| f64::from(mu0[k]) > d)
                            }
                    };
                    let sheet = geo.sheet_cells(&iced);
                    Trial {
                        ln_factor: x,
                        land_ice: land_share(&c.sheet),
                        land_sheet: land_share(&geo.band_share(&|k| sheet[k], &|k| land0[k])),
                        sea_ice: c.sea_ice,
                        snowball: c.snowball.is_some(),
                        profile: geo.sheet_profile(&sheet, &land0, &|k| h[k] - sea_prev, &lat),
                        clock: c,
                        ebm: e,
                        ir: i,
                        out,
                    }
                };
                let reach = |t: &Trial| -> bool {
                    match mode {
                        PresentClimate::Temperate => {
                            !t.snowball && t.land_sheet <= p.present_interglacial_ice_frac
                        }
                        PresentClimate::IceAge => {
                            t.snowball || t.land_sheet >= p.present_glacial_ice_frac
                        }
                        PresentClimate::Snowball => t.snowball,
                        PresentClimate::Natural => true,
                    }
                };
                let dir = if mode == PresentClimate::Temperate {
                    1.0
                } else {
                    -1.0
                };
                let natural = trial(0.0);
                let (nat_ice, nat_sheet, nat_sea, nat_snow) = (
                    natural.land_ice,
                    natural.land_sheet,
                    natural.sea_ice,
                    natural.snowball,
                );
                let nat_profile = natural.profile.clone();
                let row = |t: &Trial| {
                    [
                        t.ln_factor,
                        t.land_sheet,
                        t.land_ice,
                        t.sea_ice,
                        t.clock.t_global,
                    ]
                };
                let mut rows = vec![row(&natural)];
                // A temperate present warms by at most `present_co2_factor_max`; the forced
                // states are not limited.
                let cap = if mode == PresentClimate::Temperate {
                    p.present_co2_factor_max.max(1.0).ln() // ledger: CLM-206
                } else {
                    f64::INFINITY
                };
                let (chosen, reached, trials) = present_search(
                    natural,
                    dir,
                    cap,
                    &|x| trial(x),
                    &|t| t.ln_factor,
                    &reach,
                    // An ice age that could only be had as a snowball stays short of it.
                    &|t| !(mode == PresentClimate::IceAge && t.snowball),
                    &mut |t| rows.push(row(t)),
                );
                self.s.present = Some(Present {
                    ln_factor: chosen.ln_factor,
                    reached,
                    trials,
                    natural_land_ice: nat_ice,
                    natural_land_sheet: nat_sheet,
                    natural_sea_ice: nat_sea,
                    natural_snowball: nat_snow,
                    land_ice: chosen.land_ice,
                    land_sheet: chosen.land_sheet,
                    sea_ice: chosen.sea_ice,
                    rows,
                    natural_profile: nat_profile,
                    profile: chosen.profile.clone(),
                });
                self.s.clock = chosen.clock;
                self.s.ebm = chosen.ebm;
                ir = chosen.ir;
                self.time("present", started);
                match first {
                    Some(f) => f.then(chosen.out),
                    None => chosen.out,
                }
            } else {
                clock::run(
                    &cin(t0, t1, None),
                    &mut self.s.clock,
                    &mut self.s.ebm,
                    &mut |s| ice_step(&mut ir, s),
                )
            }
        } else {
            clock::Output {
                last: None,
                deltas: vec![(t1 - t0, vec![0.0; bands])],
                happenings: Vec::new(),
                co2_min: self.ppm(),
                co2_max: self.ppm(),
                t_min: f64::NAN,
                t_max: f64::NAN,
                sea_ice_max: 0.0,
                steps: 0,
                carbonate_excess_mol: 0.0,
            }
        };
        let IceRun {
            hs,
            hmax,
            share_acc,
            sheet: sheet_state,
            rows: window_rows,
            ..
        } = ir;
        self.s.sheet = sheet_state;
        // The window's curve gets the ice the callback measured.
        if window_step {
            let rows = self.s.clock.window.len();
            let k0 = rows.saturating_sub(window_rows.len());
            for (row, (share, vol)) in self.s.clock.window[k0..].iter_mut().zip(&window_rows) {
                row[2] = *share as f32;
                row[3] = *vol as f32;
            }
        }
        let delta_end = out
            .deltas
            .last()
            .map_or_else(|| vec![0.0; bands], |x| x.1.clone());
        self.time("clock", started);

        // 2. The end of the step's ice and the sea level it takes.
        let started = Instant::now();
        let mut mask_pre: Vec<bool> = Vec::new();
        let ice_end_pre: Vec<f32> = if window_step {
            hs.clone()
        } else if dyn_ice {
            // The sheet's ice of the last step (carried with the plates): the sea level lags a
            // plate step in deep time.
            hs.iter()
                .enumerate()
                .map(|(c, &x)| if land0[c] { x } else { 0.0 })
                .collect()
        } else {
            mask_pre = (0..n)
                .into_par_iter()
                .map(|c| mu0[c].is_finite() && f64::from(mu0[c]) > delta_end[geo.cell_band(c)])
                .collect();
            ice::thickness(w, hgt, &mask_pre, &geo.sx, &geo.sy, tau, cap, grav)
        };
        let curve = self.curve_sea_level(t_end_myr);
        let ocean_km2 = geo.area_where(&|c| h[c] <= curve);
        let v_pre = geo.volume_km3(&ice_end_pre);
        let mut sea =
            curve + p.glacio_eustasy * ice::eustatic_m(v_pre - p.ice_reference_km3, ocean_km2);
        if self.steering().is_some() && self.s.steer.sea_m != 0.0 {
            // The nudge on the ocean's water volume (`steering`), as the sea level it moves.
            sea += self.s.steer.sea_m;
        }
        world.globals.insert(SEA_LEVEL_GLOBAL.into(), sea);
        let land = land_at(sea);
        // The weathering's drawdown beyond the degassing precipitates as carbonate (after a
        // snowball: the cap carbonate), spread evenly over the sea floor above the compensation
        // depth.
        let shelf_km2 =
            geo.area_where(&|c| h[c] <= sea && sea - h[c] <= crate::surface::CARBONATE_FLOOR_M);
        let cap_m = if shelf_km2 > 0.0 {
            out.carbonate_excess_mol * carbon::CALCITE_M3_PER_MOL / (shelf_km2 * 1.0e6)
        } else {
            0.0
        };
        world.globals.insert(CAP_CARBONATE_GLOBAL.into(), cap_m);
        self.s.cap_carbonate_m += cap_m;
        self.time("ice", started);

        let ppm = self.ppm();
        let er_end: Option<EbmResult>;
        let mut deltas = out.deltas.clone();
        // A triggered snapshot (P1-26 design §4.2): the clock's climate moved far from the last
        // snapshot's (a snowball's onset or escape, a hothouse), so the per-cell climate, its
        // flux correction and its pattern are refreshed now instead of at the next due one.
        if !snap
            && p.model == ClimateModel::PerCell
            && out.steps > 0
            && ((self.s.clock.t_global - self.s.clock.snap_t_global).abs() > percell::TRIGGER_K
                || (self.s.clock.sea_ice - self.s.snap_sea_ice).abs() > percell::TRIGGER_ICE)
        {
            snap = true;
            self.s.triggered += 1;
        }
        if snap {
            let started = Instant::now();
            let inp = snapshot::Input {
                grid: &grid,
                d,
                lat: &lat,
                h: &h,
                sea,
                p: &p,
                planet: self.planet(ppm, t1),
                monthly: last,
                ice_prev: ice_prev.as_deref(),
                koppen_prev: world
                    .fields
                    .get(field::KOPPEN)
                    .filter(|f| f.data.len() == n)
                    .map(|f| f.data.as_slice()),
                correction: &self.s.correction,
            };
            let mut ebm = std::mem::take(&mut self.s.ebm);
            let mut cell = std::mem::take(&mut self.s.cell);
            let sinking_before = cell.sinking;
            let dev = self.cells.get(ctx, &p, w, hgt, last);
            let s = snapshot::compute_on(
                &inp,
                &mut ebm,
                &self.s.clock.sheet,
                &mut cell,
                dev.as_deref(),
            );
            self.cells.ran(ctx, &mut self.s.cells, &s.cells_on);
            self.s.ebm = ebm;
            // The conveyor moved to another basin: an event (EVENTS.md).
            if let (Some(a), Some(b)) = (sinking_before, cell.sinking) {
                let moved = a.2 != b.2
                    || deeptime_format::projection::unit_of_lat_lon(a.0, a.1)
                        .iter()
                        .zip(deeptime_format::projection::unit_of_lat_lon(b.0, b.1))
                        .map(|(x, y)| x * y)
                        .sum::<f64>()
                        < 40f64.to_radians().cos();
                if moved {
                    let sv = s
                        .cell
                        .as_ref()
                        .and_then(|c| c["sinking"]["sv"].as_f64())
                        .unwrap_or(p.overturning_sv);
                    let e = world
                        .event(t1, OCEAN_OVERTURNING_SHIFT, glaciers::SOURCE)
                        .at(Place::Point {
                            lat: b.0 as f32,
                            lon: b.1 as f32,
                        })
                        .magnitude((sv / 30.0).clamp(0.0, 1.0) as f32)
                        .param("from_lat", a.0)
                        .param("from_lon", a.1)
                        .param("transport_sv", sv);
                    world.emit(e)?;
                }
            }
            self.s.cell = cell;
            self.s.correction = s.correction.clone();
            if let Some(er) = &s.ebm {
                self.s.snap_band_ice = er.sea_ice.clone();
                self.s.snap_sea_ice = clock::sea_ice_frac(er, &band_land(&land_at(sea)));
            }
            put(world, field::SEA_ICE, s.sea_ice.clone());
            put(world, field::SEA_ICE_MAX, s.sea_ice_max.clone());
            put(world, field::SEA_ICE_MIN, s.sea_ice_min.clone());
            for (k, v) in &s.millis {
                *self.timings.entry(k).or_insert(0.0) += v;
            }
            let er = s.ebm.clone().expect("a snapshot keeps its EBM year");
            // The new equilibrium line is the reference: the clock's shifts become relative to
            // the step's end.
            let (_, summer) = clock::land_temps(&er);
            self.s.clock.snap_summer = summer;
            self.s.clock.snap_t_global = er.t_global;
            for (_, dl) in &mut deltas {
                for (x, e) in dl.iter_mut().zip(&delta_end) {
                    *x -= e;
                }
            }
            self.s.refs = s.refs.clone();
            self.s.snapshots += 1;
            self.s.last = stats(&grid, &self.preset, &s, t_end_myr, sea, &h).to_string();
            ela_all = Some(s.ela.clone());
            put(world, field::T_MEAN, s.t_mean);
            put(world, field::PRECIP, s.precip);
            put(world, field::T_WARM, s.t_warm);
            put(world, field::T_COLD, s.t_cold);
            put(world, field::ARIDITY, s.aridity);
            put(
                world,
                field::KOPPEN,
                s.koppen.iter().map(|&x| f32::from(x)).collect(),
            );
            put(world, field::D_MEAN, s.d_mean);
            put(world, field::D_WARM, s.d_warm);
            put(world, field::D_COLD, s.d_cold);
            put(world, field::D_LN_PRECIP, s.d_ln_precip);
            let monthly = s.t_sl_month.len() == 12 && s.p_month.len() == 12;
            for (m, (t, p)) in s.t_sl_month.into_iter().zip(s.p_month).enumerate() {
                if monthly && t.len() == n && p.len() == n {
                    put(world, &field::t_sl_month(m), t);
                    put(world, &field::p_month(m), p);
                }
            }
            er_end = Some(er);
            self.time("snapshot", started);
        } else {
            // Carry the geographic part with the plates; the zonal part follows the clock's EBM.
            let started = Instant::now();
            let src = src
                .as_ref()
                .expect("checked: no snapshot without a previous step");
            let carry_now = |world: &WorldState, name: &str| -> Vec<f32> {
                let v = &world.fields[name].data;
                src.par_iter()
                    .map(|&[fi, fj]| advect::bilinear(w, hgt, v, fi, fj))
                    .collect()
            };
            let d_mean = carry_now(world, field::D_MEAN);
            let d_warm = carry_now(world, field::D_WARM);
            let d_cold = carry_now(world, field::D_COLD);
            let d_lnp = carry_now(world, field::D_LN_PRECIP);
            let aridity = carry_now(world, field::ARIDITY);
            // Sea ice: carried, then moved by its band's change since the snapshot (none on land).
            let band_now = out.last.as_ref().map(|er| er.sea_ice.clone());
            for name in [field::SEA_ICE, field::SEA_ICE_MAX, field::SEA_ICE_MIN] {
                if !world.fields.get(name).is_some_and(|f| f.data.len() == n) {
                    continue;
                }
                let carried = carry_now(world, name);
                let v: Vec<f32> = (0..n)
                    .map(|c| {
                        if land[c] {
                            return 0.0;
                        }
                        let b = geo.cell_band(c);
                        let shift = match &band_now {
                            Some(now) if b < now.len() && b < self.s.snap_band_ice.len() => {
                                now[b] - self.s.snap_band_ice[b]
                            }
                            _ => 0.0,
                        };
                        (f64::from(carried[c]) + shift).clamp(0.0, 1.0) as f32
                    })
                    .collect();
                put(world, name, v);
            }
            let old_k = &world.fields[field::KOPPEN].data;
            let kop_carried: Vec<f32> = src
                .par_iter()
                .map(|&[fi, fj]| old_k[advect::nearest(w, hgt, fi, fj)])
                .collect();
            if let Some(er) = &out.last {
                self.s.refs = snapshot::zonal_refs(&lat, er, snapshot::hydro_ln(&p, er.t_global));
            }
            let refs = &self.s.refs;
            #[allow(clippy::needless_range_loop)]
            let rows: Vec<[Vec<f32>; 5]> = (0..hgt)
                .into_par_iter()
                .map(|j| {
                    let mut out: [Vec<f32>; 5] = std::array::from_fn(|_| vec![0.0; w]);
                    for i in 0..w {
                        let c = j * w + i;
                        let z = (h[c] - sea).max(0.0);
                        let tm = refs.t_mean[j] + f64::from(d_mean[c]) - gamma * z;
                        let tw = refs.t_warm[j] + f64::from(d_warm[c]) - gamma * z;
                        let tc = refs.t_cold[j] + f64::from(d_cold[c]) - gamma * z;
                        let pr = (refs.ln_precip[j] + f64::from(d_lnp[c])).exp();
                        let kop = if !land[c] {
                            f32::from(koppen::SEA)
                        } else if kop_carried[c] as u8 == koppen::SEA {
                            f32::from(fallback_koppen(tw, tc, pr))
                        } else {
                            kop_carried[c]
                        };
                        out[0][i] = tm as f32;
                        out[1][i] = pr as f32;
                        out[2][i] = tw as f32;
                        out[3][i] = tc as f32;
                        out[4][i] = kop;
                    }
                    out
                })
                .collect();
            let mut cols: [Vec<f32>; 5] = std::array::from_fn(|_| Vec::with_capacity(n));
            for r in rows {
                for (col, v) in cols.iter_mut().zip(r) {
                    col.extend(v);
                }
            }
            let [tm, pr, tw, tc, kop] = cols;
            put(world, field::T_MEAN, tm);
            put(world, field::PRECIP, pr);
            put(world, field::T_WARM, tw);
            put(world, field::T_COLD, tc);
            put(world, field::KOPPEN, kop);
            put(world, field::ARIDITY, aridity);
            put(world, field::D_MEAN, d_mean);
            put(world, field::D_WARM, d_warm);
            put(world, field::D_COLD, d_cold);
            put(world, field::D_LN_PRECIP, d_lnp);
            er_end = out.last.clone();
            self.time("advect", started);
        }

        // 3. The ice fields on the final ground and line.
        let started = Instant::now();
        let ela_all = ela_all.unwrap_or_else(|| vec![1.0e6; n]);
        let d_end: Vec<f64> = deltas
            .last()
            .map_or_else(|| vec![0.0; bands], |x| x.1.clone());
        let mu: Vec<f64> = (0..n)
            .into_par_iter()
            .map(|c| {
                if land[c] {
                    h[c] - sea - f64::from(ela_all[c])
                } else {
                    f64::NEG_INFINITY
                }
            })
            .collect();
        let (thick, thick_max, share): (Vec<f32>, Vec<f32>, Vec<f32>) = if window_step {
            let span = ((t1 - t0) / 1.0e3) as f32;
            let keep = |v: &[f32]| -> Vec<f32> {
                v.iter()
                    .enumerate()
                    .map(|(c, &x)| if land[c] { x } else { 0.0 })
                    .collect()
            };
            (
                keep(&hs),
                keep(&hmax),
                share_acc
                    .iter()
                    .enumerate()
                    .map(|(c, &s)| {
                        if land[c] {
                            (s / span.max(1e-9)).min(1.0)
                        } else {
                            0.0
                        }
                    })
                    .collect(),
            )
        } else {
            let per_band: Vec<ice::ShiftTimes> = (0..bands)
                .map(|b| {
                    let v: Vec<(f64, f64)> = deltas.iter().map(|(dt, dl)| (dl[b], *dt)).collect();
                    ice::ShiftTimes::new(&v)
                })
                .collect();
            let end_mask: Vec<bool> = (0..n)
                .into_par_iter()
                .map(|c| mu[c] > d_end[geo.cell_band(c)])
                .collect();
            let max_mask: Vec<bool> = (0..n)
                .into_par_iter()
                .map(|c| {
                    mu[c]
                        > per_band[geo.cell_band(c)]
                            .min()
                            .min(d_end[geo.cell_band(c)])
                })
                .collect();
            // Without a snapshot the line and the ground moved little since the sea level's
            // estimate: its ice is the end's when the masks agree.
            let end = if end_mask == mask_pre {
                ice_end_pre.clone()
            } else {
                ice::thickness(w, hgt, &end_mask, &geo.sx, &geo.sy, tau, cap, grav)
            };
            // The step's largest ice is most often its end's (a steady climate): no second
            // distance transform then.
            let max = if max_mask == end_mask {
                end.clone()
            } else {
                ice::thickness(w, hgt, &max_mask, &geo.sx, &geo.sy, tau, cap, grav)
            };
            let mut share: Vec<f32> = (0..n)
                .into_par_iter()
                .map(|c| {
                    if land[c] {
                        per_band[geo.cell_band(c)].share(mu[c]) as f32
                    } else {
                        0.0
                    }
                })
                .collect();
            let (mut end, mut max) = (end, max);
            if dyn_ice && p.ice_relax_kyr > 0.0 {
                // The sheet runs `ice_relax_kyr` under the step's end line from the ice the
                // plates carried, on the final ground and sea: its flow and elevation feedback
                // spread the ice beyond the equilibrium glaciers.
                let bed_c = sg.coarsen(&bed_fine(sea), &|_| true);
                let ela_c: Vec<f32> = sg
                    .coarsen(&ela_all, &|c| land[c])
                    .iter()
                    .enumerate()
                    .map(|(cc, &e)| e + d_end[coarse_band[cc / sg.w]] as f32)
                    .collect();
                let mut st = sheet::State::settled(sg.coarsen(&hs, &|_| true));
                // A sheet outlives the line that grew it (on Earth's relief 16 % of the land
                // stays under ice at today's line after a glacial, against 11 % from bare
                // ground), and in deep time nothing would ever melt it back: the ice would only
                // ratchet up from plate step to plate step. So the first third of the run stands
                // for an interglacial's warm summers, the line `INTERGLACIAL_RISE_M` higher (as
                // the orbit does every cycle), and the rest runs at the step's line.
                let warm: Vec<f32> = ela_c.iter().map(|e| e + INTERGLACIAL_RISE_M).collect();
                st.step(
                    &sg,
                    &sheet::Forcing {
                        bed: &bed_c,
                        ela: &warm,
                        precip: &precip0_c,
                    },
                    &sp,
                    p.ice_relax_kyr * 1000.0 / 3.0,
                );
                let forcing = sheet::Forcing {
                    bed: &bed_c,
                    ela: &ela_c,
                    precip: &precip0_c,
                };
                st.step(&sg, &forcing, &sp, p.ice_relax_kyr * 1000.0 * 2.0 / 3.0);
                let fine = sg.refine(&st.thick);
                for c in 0..n {
                    if land[c] && fine[c] >= ICE_MIN_M {
                        end[c] = end[c].max(fine[c]);
                        max[c] = max[c].max(fine[c]);
                        share[c] = 1.0;
                    }
                }
            }
            (end, max, share)
        };
        let glacier: Vec<bool> = thick.par_iter().map(|&x| x >= ICE_MIN_M).collect();
        let ela_pub: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|c| {
                if land[c] {
                    (f64::from(ela_all[c]) + d_end[geo.cell_band(c)]) as f32
                } else {
                    f32::NAN
                }
            })
            .collect();
        // Sea ice is the snapshot's per cell, or carried and moved by its band since (above);
        // a step without either (no clock, no snapshot) keeps the band's.
        if !world
            .fields
            .get(field::SEA_ICE)
            .is_some_and(|f| f.data.len() == n)
        {
            let sea_ice: Vec<f32> = match &er_end {
                Some(er) => (0..n)
                    .into_par_iter()
                    .map(|c| {
                        if land[c] {
                            0.0
                        } else {
                            let b = geo.cell_band(c);
                            er.sea_ice[b.min(er.sea_ice.len() - 1)] as f32
                        }
                    })
                    .collect(),
                None => vec![0.0; n],
            };
            put(world, field::SEA_ICE, sea_ice);
        }
        let ice_cover = snapshot::ice_cover(
            &grid,
            self.preset.scale.m_per_block_h,
            &land,
            &glacier.par_iter().map(|&g| u8::from(g)).collect::<Vec<_>>(),
        );
        self.s.clock.sheet = geo.band_share(&|c| glacier[c], &|c| land[c]);
        let volume = geo.volume_km3(&thick);
        self.s.ice_volume_km3 = volume;
        self.s.land_ice_frac = ice_cover.frac();
        let ice_sheet = geo.sheet_cells(&|c| land[c] && glacier[c]);
        let land_km2 = geo.area_where(&|c| land[c]);
        self.s.land_sheet_frac = if land_km2 > 0.0 {
            geo.area_where(&|c| ice_sheet[c]) / land_km2
        } else {
            0.0
        };
        self.s.sea_level_m = Some(sea);
        // The ice's surface mass balance (for the hydrology: no runoff under the ice, meltwater
        // where the balance is negative): the sheet's law on the ice surface against the line.
        let precip_m: Option<Vec<f32>> = world
            .fields
            .get(field::PRECIP)
            .filter(|f| f.data.len() == n)
            .map(|f| f.data.clone());
        let balance: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|c| {
                if !glacier[c] {
                    return 0.0;
                }
                let above = h[c] - sea + f64::from(thick[c]) - f64::from(ela_pub[c]);
                let pr = precip_m.as_ref().map_or(0.5, |v| f64::from(v[c]) / 1000.0);
                ice::surface_balance(
                    above,
                    pr,
                    p.ice_accumulation_gradient,
                    p.ice_ablation_gradient,
                    p.ice_accumulation_max_m_yr,
                ) as f32
            })
            .collect();
        put(world, field::ICE_BALANCE, balance);
        put(world, field::ELA_ALL, ela_all);
        put(world, field::ELA, ela_pub);
        put(
            world,
            field::GLACIER,
            glacier
                .par_iter()
                .map(|&g| f32::from(u8::from(g)))
                .collect(),
        );
        put(
            world,
            field::ICE_MAX_EXTENT,
            thick_max
                .par_iter()
                .map(|&x| f32::from(u8::from(x >= ICE_MIN_M)))
                .collect(),
        );
        put(world, field::ICE_THICKNESS, thick);
        put(world, field::ICE_MAX_THICKNESS, thick_max);
        put(world, field::ICE_TIME_SHARE, share);
        let mut tracker = std::mem::take(&mut self.s.tracker);
        tracker.update(
            world,
            t1,
            &ice_cover,
            p.glaciation_on_frac,
            p.glaciation_off_frac,
            last,
        )?;
        self.s.tracker = tracker;

        // The next step's weathering reads this step's climate.
        let mut runoff_mean = f64::NAN;
        let (mut kernel_high, mut land_high, mut t_land_mean) = (f64::NAN, f64::NAN, f64::NAN);
        let mut kernel_sea_t = f64::NAN;
        if let Some(er) = &er_end {
            let t_mean = &world.fields[field::T_MEAN].data;
            let precip = &world.fields[field::PRECIP].data;
            // The surface's erosion of its last step (the rock it exposed) limits the supply.
            let erosion = world
                .fields
                .get(crate::surface::field::EROSION)
                .filter(|f| f.data.len() == n)
                .map(|f| f.data.as_slice());
            // The kernel and the notes' sums in one pass (the cell's runoff found once).
            let lt = land_terms(&geo, &land, t_mean, precip, er, &p, erosion, (&h, sea));
            self.s.clock.kernel = lt.kernel;
            runoff_mean = lt.rs / lt.ra.max(1e-9);
            kernel_high = lt.kh / lt.kt.max(1e-30);
            land_high = lt.ah / lt.ra.max(1e-9);
            t_land_mean = lt.ta / lt.ra.max(1e-9);
            kernel_sea_t = lt.ks / carbon::KERNEL_EARTH;
        }
        self.time("ice", started);

        // 4. Events and the curve.
        self.emit(world, &out.happenings, last, t1, cap_m)?;
        let t_global = er_end.as_ref().map_or(f64::NAN, |e| e.t_global);
        let sea_ice_frac = er_end
            .as_ref()
            .map_or(0.0, |e| clock::sea_ice_frac(e, &band_land(&land)));
        let land_frac =
            geo.area_where(&|c| land[c]) / geo.area.iter().map(|a| a * w as f64).sum::<f64>();
        // The nudges (`steering`): measured at this step's end, acting from the next step.
        if let Some(sp) = self.steering() {
            let wgt = sp.weight(t1, p.run_end_years, self.preset.run_years());
            crate::steering::step(
                &sp,
                &mut self.s.steer,
                crate::steering::Measured {
                    t_years: t1,
                    w: wgt,
                    dt_myr,
                    land: land_frac,
                    temp_c: t_global,
                    co2: p.carbon_cycle,
                },
            );
            self.s.steer.ocean_km2 = geo.area_where(&|c| !land[c]);
        }
        // The land's weathering at this step's end climate, with and without the supply limit.
        let (w_land, w_kin) = {
            let wp = carbon::Weathering::from(&p);
            let ck = &self.s.clock;
            let kin = carbon::Kernel {
                supply: Vec::new(),
                ..ck.kernel.clone()
            };
            (
                wp.flux(&ck.kernel, &ck.t_land, &ck.sheet, ppm),
                wp.flux(&kin, &ck.t_land, &ck.sheet, ppm),
            )
        };
        self.s.clock.series.push(vec![
            t_end_myr as f32,
            ppm as f32,
            out.co2_min as f32,
            out.co2_max as f32,
            t_global as f32,
            out.t_min.min(t_global) as f32,
            out.t_max.max(t_global) as f32,
            sea_ice_frac as f32,
            out.sea_ice_max.max(sea_ice_frac) as f32,
            ice_cover.frac() as f32,
            volume as f32,
            sea as f32,
            self.s.clock.degassing as f32,
            self.s.clock.weathering as f32,
            clock::sun_at(&p, t1) as f32,
            f32::from(u8::from(self.s.clock.snowball.is_some())),
            activity.ridge_km2_yr as f32,
            activity.subduction_km2_yr as f32,
            carbon::Weathering::supply(&self.s.clock.kernel, &self.s.clock.sheet) as f32,
            land_frac as f32,
            (self.s.clock.kernel.band.iter().sum::<f64>() / carbon::KERNEL_EARTH) as f32,
            runoff_mean as f32,
            w_land as f32,
            w_kin as f32,
            kernel_high as f32,
            land_high as f32,
            t_land_mean as f32,
            kernel_sea_t as f32,
        ]);
        self.s.omega = Some(omega_now);
        if last {
            world
                .notes
                .insert(STAGE_NAME.into(), self.summary().to_string());
        }
        Ok(())
    }

    /// Writes the clock's snowballs and hothouses as events; at the end of the run the open ones
    /// with `ongoing` = 1.
    fn emit(
        &mut self,
        world: &mut WorldState,
        happenings: &[(f64, Happening)],
        last: bool,
        t_now: f64,
        cap_m: f64,
    ) -> Result<(), CoreError> {
        let mut all: Vec<Happening> = happenings.iter().map(|(_, h)| h.clone()).collect();
        if last {
            if let Some(open) = self.s.clock.snowball.clone() {
                all.push(Happening::SnowballEscape {
                    co2_escape_ppm: self.ppm(),
                    degassing_rel: self.s.clock.degassing / carbon::DEGASSING_EARTH_MOL_YR,
                    open,
                    t_end: f64::NAN,
                });
            }
            if let Some(open) = self.s.clock.hothouse.take() {
                all.push(Happening::HothouseEnd {
                    open,
                    t_end: t_now,
                    ongoing: true,
                });
            }
        }
        // Where `present_climate` moved the CO₂, the start of its stretch (a snowball it set off or
        // ended says so).
        let nudged = self
            .s
            .present
            .as_ref()
            .filter(|x| last && x.ln_factor != 0.0)
            .map(|_| t_now - self.p.present_adjust_kyr * 1.0e3);
        for hp in all {
            match hp {
                Happening::SnowballOnset => {}
                Happening::SnowballEscape {
                    open,
                    t_end,
                    co2_escape_ppm,
                    degassing_rel,
                } => {
                    let ongoing = t_end.is_nan();
                    if !ongoing {
                        self.s.escape_ppm.push(co2_escape_ppm);
                    }
                    let t_end = if ongoing { t_now } else { t_end };
                    let dur = (t_end - open.t_start) / 1.0e6;
                    let drawdown = open.co2_before_ppm / open.co2_low_ppm.max(1e-9);
                    let rise = co2_escape_ppm / open.co2_onset_ppm.max(1e-9);
                    let e = world
                        .event(open.t_start, SNOWBALL, glaciers::SOURCE)
                        .until(t_end.max(open.t_start))
                        .at(Place::Global)
                        .magnitude((dur / 50.0).clamp(0.0, 1.0) as f32)
                        .param("duration_myr", dur)
                        .param("co2_before_ppm", open.co2_before_ppm)
                        .param("co2_onset_ppm", open.co2_onset_ppm)
                        .param("co2_escape_ppm", co2_escape_ppm)
                        .param("co2_max_ppm", open.co2_max_ppm)
                        .param("co2_drawdown_factor", drawdown)
                        .param("co2_rise_factor", rise)
                        .param("tropical_land_share", open.tropical_land)
                        .param("tropical_weathering_share", open.tropical_weathering)
                        .param("degassing_onset_rel", open.degassing_rel)
                        .param("degassing_escape_rel", degassing_rel)
                        .param("sun_rel", open.sun_rel)
                        .param("t_global_min_c", open.t_min_c)
                        .param("cap_carbonate_m", if ongoing { 0.0 } else { cap_m })
                        .param(
                            "onset_cause",
                            if nudged.is_some_and(|t| open.t_start >= t) {
                                "present_setting"
                            } else if drawdown >= 1.5 {
                                "weathering_drawdown"
                            } else {
                                "slow_cooling"
                            },
                        )
                        .param(
                            "escape_cause",
                            if ongoing {
                                "none"
                            } else if nudged.is_some_and(|t| t_end >= t) {
                                "present_setting"
                            } else if rise >= 2.0 {
                                "degassing_buildup"
                            } else {
                                "warming"
                            },
                        )
                        .param("ongoing", i64::from(ongoing));
                    let id = world.emit(e)?;
                    if !ongoing {
                        self.s.clock.last_escape = Some((t_end, Some(id)));
                        self.s
                            .clock
                            .spans
                            .push([open.t_start / 1.0e6, t_end / 1.0e6]);
                    }
                    if ongoing {
                        self.s.clock.snowball = None;
                    }
                }
                Happening::HothouseEnd {
                    open,
                    t_end,
                    ongoing,
                } => {
                    let mut e =
                        world
                            .event(open.t_start, HOTHOUSE, glaciers::SOURCE)
                            .until(t_end.max(open.t_start))
                            .at(Place::Global)
                            .magnitude(
                                ((open.peak_c - self.p.hothouse_c) / 25.0).clamp(0.0, 1.0) as f32
                            )
                            .param("peak_t_global_c", open.peak_c)
                            .param("peak_t_years", open.peak_t)
                            .param("co2_peak_ppm", open.co2_peak_ppm)
                            .param("after_snowball", i64::from(open.after_escape.is_some()))
                            .param("ongoing", i64::from(ongoing));
                    let cause = match (open.after_escape, self.s.clock.last_escape) {
                        (Some(te), Some((t_esc, Some(id)))) if (te - t_esc).abs() < 1.0 => Some(id),
                        _ => None,
                    };
                    if let Some(id) = cause {
                        e = e.caused_by(id);
                    }
                    world.emit(e)?;
                }
            }
        }
        Ok(())
    }

    /// The run summary (`manifest.json` `notes.climate`, the CLI's `climate_final` line).
    fn summary(&self) -> serde_json::Value {
        let mut last: serde_json::Value = serde_json::from_str(&self.s.last).unwrap_or(json!({}));
        if let Some(o) = last.as_object_mut() {
            o.insert("co2_ppm".into(), json!(self.ppm()));
            o.insert(
                "sun_rel".into(),
                json!(self.s.clock.series.last().map_or(1.0, |r| r[14])),
            );
            o.insert("ice_volume_km3".into(), json!(self.s.ice_volume_km3));
            o.insert("sea_level_m".into(), json!(self.s.sea_level_m));
            o.insert("glaciated_land_frac".into(), json!(self.s.land_ice_frac));
            // Of it, the ice sheets' and the mountain ice's (`ICE_SHEET_MIN_KM2`, CLM-203).
            o.insert("ice_sheet_land_frac".into(), json!(self.s.land_sheet_frac));
            o.insert(
                "mountain_ice_land_frac".into(),
                json!((self.s.land_ice_frac - self.s.land_sheet_frac).max(0.0)),
            );
            o.insert("degassing_mol_yr".into(), json!(self.s.clock.degassing));
            o.insert("weathering_mol_yr".into(), json!(self.s.clock.weathering));
            o.insert(
                "cap_carbonate_total_m".into(),
                json!(self.s.cap_carbonate_m),
            );
        }
        let round = |v: &[Vec<f32>]| -> Vec<Vec<f64>> {
            v.iter()
                .map(|r| {
                    r.iter()
                        .map(|&x| {
                            let x = f64::from(x);
                            if x == 0.0 || !x.is_finite() {
                                0.0
                            } else {
                                let m = 10f64.powi(4 - x.abs().log10().ceil() as i32);
                                (x * m).round() / m
                            }
                        })
                        .collect()
                })
                .collect()
        };
        let mut out = json!({
            "snapshots": self.s.snapshots,
            "triggered_snapshots": self.s.triggered,
            // Hoffman et al. 2017 expect terminations at 10⁴–10⁵ ppm (design §4.2, WP-14a).
            "snowball_escape_co2_ppm": self.s.escape_ppm,
            "earth_escape_range_ppm": [1.0e4, 1.0e5], // ledger: CLM-166
            "glaciations": self.s.tracker.count,
            "snowballs": self.s.clock.snowballs,
            "snowball_spans_myr": self.s.clock.spans,
            "hothouses": self.s.clock.hothouses,
            "present": self.s.present.as_ref().map_or_else(
                || json!({"state": PresentClimate::Natural.name(), "co2_factor": 1.0}),
                |x| json!({
                    "state": self.p.present_climate.name(),
                    "co2_factor": x.ln_factor.exp(),
                    "reached": x.reached,
                    "trials": x.trials,
                    "land_ice_frac": x.land_ice,
                    "land_ice_sheet_frac": x.land_sheet,
                    "land_ice_mountain_frac": (x.land_ice - x.land_sheet).max(0.0),
                    "sea_ice_frac": x.sea_ice,
                    "natural_land_ice_frac": x.natural_land_ice,
                    "natural_land_ice_sheet_frac": x.natural_land_sheet,
                    "natural_sea_ice_frac": x.natural_sea_ice,
                    "natural_snowball": x.natural_snowball,
                    "co2_factor_max": (self.p.present_climate == PresentClimate::Temperate)
                        .then_some(self.p.present_co2_factor_max),
                    "trial_rows": {
                        "columns": ["co2_factor", "land_ice_sheet_frac", "land_ice_frac",
                            "sea_ice_frac", "t_global_c"],
                        "rows": x.rows.iter().map(|r| {
                            let mut r = *r;
                            r[0] = r[0].exp();
                            r.map(|v| (v * 1.0e4).round() / 1.0e4)
                        }).collect::<Vec<_>>(),
                    },
                    "sheets_by_height": {
                        "lower_edges_m": HEIGHT_CLASSES_M,
                        "natural_land_share": x.natural_profile.height_land,
                        "natural_sheet_share": x.natural_profile.height_sheet,
                        "land_share": x.profile.height_land,
                        "sheet_share": x.profile.height_sheet,
                    },
                    "sheets_by_latitude": {
                        "lower_edges_deg": LAT_CLASSES_DEG,
                        "natural_land_share": x.natural_profile.lat_land,
                        "natural_sheet_share": x.natural_profile.lat_sheet,
                        "land_share": x.profile.lat_land,
                        "sheet_share": x.profile.lat_sheet,
                    },
                }),
            ),
            "last": last,
            "series": {"columns": clock::SERIES_COLUMNS, "rows": round(&self.s.clock.series)},
            "window": {"columns": clock::WINDOW_COLUMNS, "rows": round(&self.s.clock.window)},
            "millis": self.timings,
            "compute": self.cells.note(&self.s.cells),
        });
        // The nudges a steered run had (absent when steering is off: the note is the unsteered
        // run's).
        if let Some(sp) = self.steering() {
            out["steering"] = crate::steering::report(&sp, &self.s.steer);
        }
        out
    }
}
/// Statistics of a snapshot (true-area weighted over land unless stated).
fn stats(
    grid: &GridDesc,
    preset: &PlanetPreset,
    s: &snapshot::Snapshot,
    t_myr: f64,
    sea: f64,
    h: &[f64],
) -> serde_json::Value {
    let w = grid.w as usize;
    let mut groups: BTreeMap<char, f64> = BTreeMap::new();
    let (mut land_a, mut all_a, mut t_land, mut p_land, mut t_all) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for j in 0..grid.h as usize {
        let a = cell_area_km2(grid, j as u32, preset.scale.m_per_block_h);
        for i in 0..w {
            let c = j * w + i;
            all_a += a;
            t_all += a * f64::from(s.t_mean[c]);
            if h[c] > sea {
                land_a += a;
                t_land += a * f64::from(s.t_mean[c]);
                p_land += a * f64::from(s.precip[c]);
                *groups.entry(koppen::group(s.koppen[c])).or_insert(0.0) += a;
            }
        }
    }
    let share: BTreeMap<String, f64> = ['A', 'B', 'C', 'D', 'E']
        .iter()
        .map(|g| {
            (
                g.to_string(),
                groups.get(g).copied().unwrap_or(0.0) / land_a.max(1e-9),
            )
        })
        .collect();
    json!({
        "t_myr": t_myr,
        "t_surface_c": t_all / all_a.max(1e-9),
        "t_land_c": t_land / land_a.max(1e-9),
        "precip_land_mm": p_land / land_a.max(1e-9),
        "koppen_land_share": share,
        "glaciated_land_frac": s.ice.frac(),
        "ebm_t_global_c": s.ebm_t_global,
        "ebm_net_toa_w_m2": s.ebm_net_toa,
        "ebm_years": s.ebm_years,
        "itcz_range_deg": [s.itcz_range.0, s.itcz_range.1],
        "land_z_p50_m": s.land_z.0,
        "land_z_p90_m": s.land_z.1,
        "land_above_2km": s.land_z.2,
        "per_cell": s.cell,
    })
}

impl Stage for ClimateStage {
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
        self.s.done
    }

    fn step(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        if self.s.done >= self.total {
            return Ok(());
        }
        ctx.pool.install(|| self.advance(world, ctx))?;
        self.s.done += 1;
        Ok(())
    }

    fn keyframe_layers(&self) -> &[&'static str] {
        &KEYFRAME_LAYERS
    }

    fn coupled_to(&self) -> Option<&'static str> {
        Some(crate::plates::STAGE_NAME)
    }

    fn save_state(&self) -> Result<Vec<u8>, CoreError> {
        to_state(&self.s)
    }

    fn load_state(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        self.s = from_state(bytes)?;
        Ok(())
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The parametric climate on L1 (plan §8.3, work item P1-05): a seasonal zonal energy balance
//! with the ice-albedo feedback (`ebm.rs`, climlab's equations), insolation from the tilt and the
//! star (`insolation.rs`), three-cell wind belts with a migrating ITCZ and the ocean gyres
//! (`belts.rs`), continentality, rain shadows and the lapse rate on the DEM (`lapse.rs`), Smith &
//! Barstad's orographic precipitation along each belt's wind (`orographic.rs`, ported from the MIT
//! `fastscape-lem/orographic-precipitation`, rustfft), the Köppen classes (`koppen.rs`) and
//! glaciers from the equilibrium line (`glaciers.rs`); `snapshot.rs` puts them together.
//!
//! # In the pipeline
//! The stage is interleaved with the plates like the surface (`Stage::coupled_to`) and sits
//! between them, so a planet preset with a `climate` section runs `[plates, climate, surface]`
//! each plate step. Every step it publishes `climate/{t_mean_c, precip_mm}` (what the surface's
//! runoff, weathering, facies and the deposit engine's frames read, `surface/climate.rs`) and
//! `climate/{koppen, t_warm_c, t_cold_c, aridity, glacier, ela_m}`, on the topography the surface
//! step is about to erode: the plate model's elevation after this plate step plus the surface's
//! offset carried with the plates exactly as the surface carries it.
//!
//! **Snapshots** (plan §8.3: every 10 Myr, plus the present): a full climate is computed at the
//! first step, every `snapshot_every_myr` of simulated time, and at the last step. Between them
//! the climate's geographic part — each cell's sea-level departure from its row's zonal reference
//! (the EBM's ocean temperature; ln of the belts' precipitation), its aridity, its ELA and its
//! Köppen class — moves with the plates (the surface's semi-Lagrangian advection, `surface::
//! advect`), and the temperatures take the lapse rate of the current ground: a rain shadow stays
//! behind its range as the range drifts, and a continent drifting toward the pole cools with its
//! latitude's reference. Glaciation events are decided at snapshots.
//!
//! The EBM's state is kept from snapshot to snapshot (a warm start of `ebm_warm_years`), and the
//! ice sheets of one snapshot can brighten the EBM's land at the next (`ice_sheet_feedback`, off
//! by default for now).

pub mod belts;
pub mod ebm;
pub mod glaciers;
pub mod insolation;
pub mod koppen;
pub mod lapse;
pub mod orographic;
pub mod params;
pub mod snapshot;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::time::Instant;

use deeptime_format::GridDesc;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use params::ClimateParams;

use crate::deposits::fields::{cell_area_km2, lat_of_row};
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::stage::{Stage, StageContext, StageEstimate, from_state, to_state};
use crate::surface::advect;
use crate::surface::grid::Dims;
use crate::world::{Field, WorldState};
use ebm::EbmState;
use snapshot::{Planet, Refs};

/// The stage's name in a pipeline (`deeptime stage climate`).
pub const STAGE_NAME: &str = "climate";

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
    /// 1 where land lies above its equilibrium line.
    pub const GLACIER: &str = "climate/glacier";
    /// Equilibrium-line altitude above sea level, m (NaN at sea).
    pub const ELA: &str = "climate/ela_m";
    /// Carried with the plates between snapshots (sea-level departures from the row references).
    pub const D_MEAN: &str = "climate/anomaly/t_mean_c";
    pub const D_WARM: &str = "climate/anomaly/t_warm_c";
    pub const D_COLD: &str = "climate/anomaly/t_cold_c";
    pub const D_LN_PRECIP: &str = "climate/anomaly/ln_precip";
    /// The ELA of every cell, sea included (so it can be interpolated).
    pub const ELA_ALL: &str = "climate/anomaly/ela_m";
}

/// Keyframe layers (with the group's plates and surface layers).
pub const KEYFRAME_LAYERS: [&str; 3] = [field::T_MEAN, field::PRECIP, field::KOPPEN];

/// What the stage keeps between steps (and in a checkpoint taken inside the group).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Saved {
    done: u64,
    /// Every plate's Ω after the last step (the motion of the next plate step).
    omega: Option<Vec<[f64; 3]>>,
    ebm: EbmState,
    sheet: Vec<f64>,
    refs: Refs,
    tracker: glaciers::Tracker,
    snapshots: u64,
    /// The last snapshot's statistics, JSON text.
    last: String,
}

/// The climate stage.
pub struct ClimateStage {
    preset: PlanetPreset,
    p: ClimateParams,
    total: u64,
    s: Saved,
    timings: BTreeMap<&'static str, f64>,
}

/// Memory and time estimate: the group's (surface or plates) plus ~110 B per L1 cell (13 fields,
/// eight orographic rasters, an FFT band) and ~2 µs per cell per snapshot plus 0.2 µs per cell per
/// step (the advection between snapshots).
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
    let snaps = (p.length_myr() / c.snapshot_every_myr).ceil() + 2.0;
    StageEstimate {
        bytes: group.bytes + cells * 110,
        seconds: cells as f64 * (snaps * 2.0e-6 + steps * 0.2e-6),
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

impl ClimateStage {
    pub fn new(preset: PlanetPreset) -> ClimateStage {
        let p = preset.climate.clone().unwrap_or_default();
        let total = preset.plate_steps();
        ClimateStage {
            preset,
            p,
            total,
            s: Saved::default(),
            timings: BTreeMap::new(),
        }
    }

    fn time(&mut self, what: &'static str, since: Instant) {
        *self.timings.entry(what).or_insert(0.0) += since.elapsed().as_secs_f64() * 1000.0;
    }

    fn sea_level(&self, t_myr: f64) -> f64 {
        self.preset
            .surface
            .as_ref()
            .map_or(0.0, |s| s.sea_level.at(t_myr))
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
        let dt_myr = self.preset.length_myr() / self.total.max(1) as f64;
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
            ctx.reserve(n as u64 * 110)?;
        }

        // Where the crust went this plate step, and the ground the surface will erode.
        let started = Instant::now();
        let pid = need("plates/plate_id")?;
        let src = self
            .s
            .omega
            .as_ref()
            .map(|prev| advect::sources(&grid, pid, &omega_for(prev, &parents), dt_myr * 1.0e6));
        let mut h: Vec<f64> = need("plates/elevation_m")?
            .iter()
            .map(|&z| f64::from(z))
            .collect();
        if let (Some(st), Some(src)) = (world.surface.as_ref(), src.as_ref()) {
            let mut off = st.offset_m.clone();
            if let Some(ap) = world
                .fields
                .get(crate::surface::field::FEEDBACK_APPLIED)
                .filter(|f| f.data.len() == n)
            {
                off.par_iter_mut()
                    .zip(ap.data.par_iter())
                    .for_each(|(o, a)| *o -= a);
            }
            h.par_iter_mut().enumerate().for_each(|(c, z)| {
                let [fi, fj] = src[c];
                *z += f64::from(advect::bilinear(w, hgt, &off, fi, fj));
            });
        }
        let sea = self.sea_level(t_end_myr);
        self.time("topography", started);

        let k = self.s.done + 1;
        let last = k >= self.total;
        let every = ((self.p.snapshot_every_myr / dt_myr).round() as u64).max(1);
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
        let snap = k == 1
            || last
            || k.is_multiple_of(every)
            || src.is_none()
            || !have_anomalies
            || self.s.refs.t_mean.len() != hgt;
        let lat: Vec<f64> = (0..hgt).map(|j| lat_of_row(&grid, j as u32)).collect();
        let gamma = self.p.lapse_c_per_km / 1000.0;
        let put = |world: &mut WorldState, name: &str, data: Vec<f32>| {
            world.fields.insert(
                name.to_owned(),
                Field {
                    grid: grid.clone(),
                    data,
                },
            );
        };

        if snap {
            let started = Instant::now();
            let planet = Planet {
                axial_tilt_deg: self.preset.planet.axial_tilt_deg,
                star_flux_rel: self.preset.planet.star_flux_rel,
                day_length_h: self.preset.planet.day_length_h,
                earth_scale: self.preset.scale.radius_m()
                    / (self.preset.tectonics.model_radius_km * 1000.0),
                m_per_block_h: self.preset.scale.m_per_block_h,
            };
            let inp = snapshot::Input {
                grid: &grid,
                d,
                lat: &lat,
                h: &h,
                sea,
                p: &self.p,
                planet,
            };
            let sheet = std::mem::take(&mut self.s.sheet);
            let mut ebm = std::mem::take(&mut self.s.ebm);
            let s = snapshot::compute(&inp, &mut ebm, &sheet);
            self.s.ebm = ebm;
            self.s.sheet = s.band_sheet.clone();
            for (k, v) in &s.millis {
                *self.timings.entry(k).or_insert(0.0) += v;
            }
            let t_end_years = t_end_myr * 1.0e6;
            let mut tracker = std::mem::take(&mut self.s.tracker);
            tracker.update(
                world,
                t_end_years,
                &s.ice,
                self.p.glaciation_on_frac,
                self.p.glaciation_off_frac,
                last,
            )?;
            self.s.tracker = tracker;
            self.s.refs = s.refs.clone();
            self.s.snapshots += 1;
            self.s.last = stats(&grid, &self.preset, &s, t_end_myr, sea, &h).to_string();
            let ela: Vec<f32> = (0..n)
                .map(|c| if h[c] > sea { s.ela[c] } else { f32::NAN })
                .collect();
            put(world, field::T_MEAN, s.t_mean);
            put(world, field::PRECIP, s.precip);
            put(world, field::T_WARM, s.t_warm);
            put(world, field::T_COLD, s.t_cold);
            put(world, field::ARIDITY, s.aridity);
            put(world, field::ELA, ela);
            put(world, field::ELA_ALL, s.ela);
            put(world, field::KOPPEN, s.koppen.iter().map(|&x| f32::from(x)).collect());
            put(world, field::GLACIER, s.glacier.iter().map(|&x| f32::from(x)).collect());
            put(world, field::D_MEAN, s.d_mean);
            put(world, field::D_WARM, s.d_warm);
            put(world, field::D_COLD, s.d_cold);
            put(world, field::D_LN_PRECIP, s.d_ln_precip);
            self.time("snapshot", started);
        } else {
            // Carry the geographic part with the plates; the zonal part stays with the latitude.
            let started = Instant::now();
            let src = src.expect("checked: no snapshot without a previous step");
            let carry = |world: &WorldState, name: &str| -> Vec<f32> {
                let v = &world.fields[name].data;
                src.par_iter()
                    .map(|&[fi, fj]| advect::bilinear(w, hgt, v, fi, fj))
                    .collect()
            };
            let d_mean = carry(world, field::D_MEAN);
            let d_warm = carry(world, field::D_WARM);
            let d_cold = carry(world, field::D_COLD);
            let d_lnp = carry(world, field::D_LN_PRECIP);
            let ela_all = carry(world, field::ELA_ALL);
            let aridity = carry(world, field::ARIDITY);
            let old_k = &world.fields[field::KOPPEN].data;
            let kop_carried: Vec<f32> = src
                .par_iter()
                .map(|&[fi, fj]| old_k[advect::nearest(w, hgt, fi, fj)])
                .collect();
            let refs = &self.s.refs;
            let rows: Vec<[Vec<f32>; 7]> = (0..hgt)
                .into_par_iter()
                .map(|j| {
                    let mut out: [Vec<f32>; 7] = std::array::from_fn(|_| vec![0.0; w]);
                    for i in 0..w {
                        let c = j * w + i;
                        let land = h[c] > sea;
                        let z = (h[c] - sea).max(0.0);
                        let tm = refs.t_mean[j] + f64::from(d_mean[c]) - gamma * z;
                        let tw = refs.t_warm[j] + f64::from(d_warm[c]) - gamma * z;
                        let tc = refs.t_cold[j] + f64::from(d_cold[c]) - gamma * z;
                        let pr = (refs.ln_precip[j] + f64::from(d_lnp[c])).exp();
                        let ela = f64::from(ela_all[c]);
                        let kop = if !land {
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
                        out[4][i] = if land { ela as f32 } else { f32::NAN };
                        out[5][i] = kop;
                        out[6][i] = if land && z > ela { 1.0 } else { 0.0 };
                    }
                    out
                })
                .collect();
            let mut cols: [Vec<f32>; 7] = std::array::from_fn(|_| Vec::with_capacity(n));
            for r in rows {
                for (col, v) in cols.iter_mut().zip(r) {
                    col.extend(v);
                }
            }
            let [tm, pr, tw, tc, ela, kop, glac] = cols;
            put(world, field::T_MEAN, tm);
            put(world, field::PRECIP, pr);
            put(world, field::T_WARM, tw);
            put(world, field::T_COLD, tc);
            put(world, field::ELA, ela);
            put(world, field::KOPPEN, kop);
            put(world, field::GLACIER, glac);
            put(world, field::ARIDITY, aridity);
            put(world, field::ELA_ALL, ela_all);
            put(world, field::D_MEAN, d_mean);
            put(world, field::D_WARM, d_warm);
            put(world, field::D_COLD, d_cold);
            put(world, field::D_LN_PRECIP, d_lnp);
            self.time("advect", started);
        }
        self.s.omega = Some(omega_now);
        if last {
            world
                .notes
                .insert(STAGE_NAME.into(), self.summary().to_string());
        }
        Ok(())
    }

    /// The run summary (`manifest.json` `notes.climate`, the CLI's `climate_final` line).
    fn summary(&self) -> serde_json::Value {
        let last: serde_json::Value = serde_json::from_str(&self.s.last).unwrap_or(json!({}));
        json!({
            "snapshots": self.s.snapshots,
            "glaciations": self.s.tracker.count,
            "last": last,
            "millis": self.timings,
        })
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

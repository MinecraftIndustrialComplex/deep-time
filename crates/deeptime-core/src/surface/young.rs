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

/// Keyframe layers of the stage.
pub const KEYFRAME_LAYERS: [&str; 3] = [field::ELEVATION, field::DRAINAGE, field::ICE];

/// Till at least this thick makes a cell's surface rock till, m.
const TILL_SHOWS_M: f32 = 5.0;

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
    /// LEM steps.
    pub steps: u32,
    pub uplift: UpliftMode,
    /// Multiplier on the uplift rate.
    pub uplift_factor: f64,
    /// Depressions of the upsampled surface shallower than this are filled, m.
    pub min_lake_depth_m: f64,
    /// Multiplier on the surface section's K (finer cells may want a different erodibility).
    pub k_scale: f64,
}

impl Default for YoungParams {
    fn default() -> Self {
        YoungParams {
            myr: 2.0,
            steps: 4,
            uplift: UpliftMode::Balance,
            uplift_factor: 1.0,
            min_lake_depth_m: 5.0,
            k_scale: 1.0,
        }
    }
}

impl YoungParams {
    pub fn validate(&self) -> Result<(), CoreError> {
        let bad = |k: &str, why: &str| CoreError::preset(format!("$.young.{k}"), why);
        if !(self.myr.is_finite() && self.myr > 0.0 && self.myr <= 50.0) {
            return Err(bad("myr", "must be in (0, 50]"));
        }
        if !(1..=100).contains(&self.steps) {
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
        Ok(())
    }
}

/// Estimate: the L2 fields and the LEM's working arrays; ≈ 1 µs per cell per step (`spl`, flood,
/// routing and diffusion, plan §8.2's cost model).
pub fn estimate(p: &PlanetPreset) -> StageEstimate {
    let y = p.young.clone().unwrap_or_default();
    let c = u64::from(p.scale.circumference_blocks);
    let r = u64::from(p.resolution.l2_r);
    let cells = (c / r) * (c / r / 2);
    let surface = super::estimate(p);
    StageEstimate {
        // The LEM's arrays, the fields, and the glaciers' ice state and passes (`glacial.rs`).
        bytes: surface.bytes + cells * (14 * 8 + 12 * 4 + 24 * 8),
        seconds: cells as f64 * f64::from(y.steps) * 1.0e-6,
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
    done: u64,
    timings: BTreeMap<&'static str, f64>,
}

/// A ×2 upsampling of a plain array (`amplify::resample`).
type Upsample = dyn Fn(&[f32], usize, usize) -> Vec<f32>;

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
        YoungStage {
            preset,
            p,
            sp,
            done: 0,
            timings: BTreeMap::new(),
        }
    }

    fn total(&self) -> u64 {
        1 + u64::from(self.p.steps)
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
        pflood::open_sea(&d, &h, sea, (n2 / 500).max(1), &mut kind);
        let base = vec![sea; n2];
        let mut filled = vec![0.0; n2];
        pflood::flood(&d, &h, &base, &mut kind, &mut filled);
        let mut small = 0usize;
        for c in 0..n2 {
            let depth = filled[c] - h[c];
            if kind[c] != BASE && depth > 0.0 && depth < self.p.min_lake_depth_m {
                z2[c] = filled[c] as f32;
                small += 1;
            }
        }
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
                "steps": self.p.steps, "myr": self.p.myr,
                "uplift": self.p.uplift,
                "filled_small_depressions": small,
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
    fn lem(
        &mut self,
        world: &mut WorldState,
        ctx: &StageContext<'_>,
        last: bool,
    ) -> Result<(), CoreError> {
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
        let get = |name: &str| -> Result<Vec<f32>, CoreError> {
            world
                .fields
                .get(name)
                .filter(|f| f.data.len() == n)
                .map(|f| f.data.clone())
                .ok_or_else(|| CoreError::InvalidState(format!("the young stage lost {name}")))
        };
        let z0 = get(field::ELEVATION_M)?;
        let class = get(field::CLASS)?;
        let weight = get(field::WEIGHT)?;
        let uplift = get(field::UPLIFT)?;
        let sp = self.sp.clone();
        let k_scale = self.p.k_scale;
        let dt_myr = self.p.myr / f64::from(self.p.steps);
        let dt = dt_myr * 1.0e6;
        let erod = lithology::erodibility();
        let diffu = lithology::diffusivity();
        let cls = |c: usize| (class[c].max(0.0) as usize).min(erod.len() - 1);

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
        let mut kind = vec![LAND; n];
        let mut filled = vec![0.0; n];
        let base_z = vec![sea; n];
        let mut rt = routing::Routing::default();
        let mut area = Vec::new();
        let wt: Vec<f32> = weight.clone();
        let route = |h: &[f64],
                     kind: &mut Vec<u8>,
                     filled: &mut Vec<f64>,
                     rt: &mut routing::Routing,
                     area: &mut Vec<f64>| {
            pflood::open_sea(&d, h, sea, (n / 500).max(1), kind);
            pflood::flood(&d, h, &base_z, kind, filled);
            routing::receivers(&d, filled, kind, rt);
            routing::stack(&d, kind, rt);
            routing::accumulate(&d, rt, &wt, area);
        };
        route(&h, &mut kind, &mut filled, &mut rt, &mut area);
        self.time("route", started);

        // The glaciers of the glacial state (`glacial.rs`), when the setup found their climate.
        let glacial_in = match (
            get(field::GLACIAL_ELA),
            get(field::T_SEA_LEVEL),
            get(field::PRECIP),
        ) {
            (Ok(ela), Ok(t_sl), Ok(pr)) => Some((ela, t_sl, pr, get(field::ICE_ALLOWED).ok())),
            _ => None,
        };
        let ero_prev = get(field::GLACIAL_EROSION).unwrap_or_else(|_| vec![0.0; n]);
        let till_prev = get(field::TILL).unwrap_or_else(|_| vec![0.0; n]);
        let gp = sp.glacial.clone();
        let gravity = self.preset.planet.gravity_m_s2;
        let started = Instant::now();
        let glacial_clim = glacial_in.as_ref().map(|(ela, t_sl, pr, _)| {
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
        // Unlike the deep-time surface, rivers keep their K under the glacial state's ice: that ice
        // stood only through part of each ice-age cycle, subglacial meltwater cuts too, and the
        // `balance` uplift assumes the rivers' erosion goes on (taking it away under thin, cold
        // ice lifted whole lowlands by the uplift of 2 Myr in the first runs).
        let ice_st = glacial::IceState::default();
        self.time("glacial", started);

        let started = Instant::now();
        let kf: Vec<f64> = (0..n).map(|c| sp.k_spl * k_scale * erod[cls(c)]).collect();
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
        self.time("spl", started);

        // Glacial erosion and till (their loads to the sea leave the grid, as the rivers' do
        // here).
        let started = Instant::now();
        let mut gout = glacial::GlacialOut::default();
        if let (Some((ela_z, t_c, p_m)), Some((_, _, _, allowed))) = (&glacial_clim, &glacial_in) {
            let ic = glacial::IceClimate {
                ela_z,
                precip_m_yr: p_m,
                t_c,
                allowed: allowed.as_deref(),
                time_share: None,
            };
            glacial::step(
                &d, &rt, &kind, &filled, sea, &ic, &gp, gravity, dt, &mut h, &mut gout,
            );
        }
        self.time("glacial", started);

        let started = Instant::now();
        let kd: Vec<f64> = (0..n)
            .map(|c| {
                let land = sp.kd_land_m2_yr * diffu[cls(c)];
                marine::submarine_kd(
                    sea - h[c],
                    sp.kd_marine_m2_yr,
                    sp.wave_base_m,
                    sp.tidal_range_m,
                    land,
                )
            })
            .collect();
        diffusion::step(&d, &mut h, &kd, dt);
        self.time("diffusion", started);

        // The routing of the surface as it now is: areas, lakes.
        let started = Instant::now();
        if last {
            route(&h, &mut kind, &mut filled, &mut rt, &mut area);
        }
        let mv = self.preset.scale.m_per_block_v;
        let sea_y = f64::from(self.preset.scale.sea_level_y);
        let put = |world: &mut WorldState, name: &str, data: Vec<f32>| {
            world.fields.insert(
                name.to_owned(),
                Field {
                    grid: g2.clone(),
                    data,
                },
            );
        };
        let z_out: Vec<f32> = h.iter().map(|&z| z as f32).collect();
        put(
            world,
            field::ELEVATION,
            h.iter().map(|&z| (sea_y + (z - sea) / mv) as f32).collect(),
        );
        put(
            world,
            field::DRAINAGE,
            (0..n)
                .map(|c| {
                    if kind[c] == BASE {
                        f32::NAN
                    } else {
                        (area[c] / 1.0e6).max(1e-3).log10() as f32
                    }
                })
                .collect(),
        );
        put(
            world,
            field::AREA_M2,
            area.iter().map(|&a| a as f32).collect(),
        );
        put(
            world,
            field::LAKE_M,
            (0..n)
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
                .collect(),
        );
        put(
            world,
            field::ERODIBILITY,
            (0..n).map(|c| erod[cls(c)] as f32).collect(),
        );
        // The glaciers: where they are now (the amplifier shapes valleys under them), what they
        // cut and left over the stage; thick till becomes the surface rock.
        // The ice over the glaciers' whole width (`IceState::footprint`), on the ground as it now
        // is.
        let th = if gout.ice.footprint.len() == n {
            &gout.ice
        } else {
            &ice_st
        };
        let surface: Vec<f32> = if th.footprint.len() == n {
            (0..n)
                .map(|c| {
                    let s = th.footprint[c];
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
        let ice_now: Vec<f32> = (0..n)
            .map(|c| {
                if surface[c].is_finite() {
                    (f64::from(surface[c]) - h[c]) as f32
                } else {
                    0.0
                }
            })
            .collect();
        let mut glacial_note = None;
        if glacial_in.is_some() {
            let (mut ero, mut till) = (ero_prev, till_prev);
            if gout.eroded.len() == n {
                for c in 0..n {
                    ero[c] += gout.eroded[c] as f32;
                    till[c] += (gout.till[c] + gout.lake[c]) as f32;
                }
            }
            let till_class = f32::from(lithology::index("till"));
            let class_now: Vec<f32> = (0..n)
                .map(|c| {
                    if till[c] >= TILL_SHOWS_M {
                        till_class
                    } else {
                        class[c]
                    }
                })
                .collect();
            let a_km2 = dx * dx / 1.0e9;
            glacial_note = Some(json!({
                "ice_cells": th.cells,
                "eroded_km3": gout.eroded_m3 / 1.0e9,
                "till_km3": (gout.till_m3 + gout.sink_m3 + gout.lake_m3) / 1.0e9,
                "sea_km3": gout.sea_m3 / 1.0e9,
                "total_eroded_km3": ero.iter().map(|&e| f64::from(e)).sum::<f64>() * a_km2,
                "till_cells": till.iter().filter(|&&t| t >= TILL_SHOWS_M).count(),
            }));
            put(world, field::ICE_SURFACE, surface);
            put(world, field::GLACIAL_EROSION, ero);
            put(world, field::TILL, till);
            put(world, field::CLASS, class_now);
        }
        put(world, field::ICE, ice_now);
        if last {
            let (p50, p90, max) = land_stats(&z_out, sea);
            let lakes = (0..n)
                .filter(|&c| kind[c] == SINK || (kind[c] == LAND && filled[c] - h[c] > LAKE_TOL_M))
                .count();
            let mut note: Value = world
                .notes
                .get(STAGE_NAME)
                .and_then(|t| serde_json::from_str(t).ok())
                .unwrap_or_else(|| json!({}));
            note["after"] = json!({"land_z_p50_m": p50, "land_z_p90_m": p90, "land_z_max_m": max});
            note["lake_cells"] = json!(lakes);
            if let Some(g) = glacial_note {
                note["glacial"] = g;
            }
            note["sinks"] = json!(kind.iter().filter(|&&k| k == SINK).count());
            note["millis"] = json!(self.timings);
            world.notes.insert(STAGE_NAME.into(), note.to_string());
        }
        put(world, field::ELEVATION_M, z_out);
        self.time("outputs", started);
        Ok(())
    }
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
         land_z_max_m={:.0}->{:.0} lake_cells={} sinks={} filled_small_depressions={}{glacial}",
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

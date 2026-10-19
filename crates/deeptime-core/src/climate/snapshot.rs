// SPDX-License-Identifier: LGPL-3.0-or-later
//! One climate snapshot on L1 (plan §8.3): the EBM by latitude, the belts, the moisture and heat
//! the winds carry, orographic precipitation, then per cell twelve monthly temperatures and
//! precipitations, reduced to the annual fields, the Köppen class and the glaciers.
//!
//! Per cell and month m (φ its latitude, φ′ its belt latitude, `belts.rs`):
//!
//! - **Temperature at sea level.** Sea: T_ocean(φ, m) + the gyre anomaly of its basin position.
//!   Land: T_ocean + c·(T_land − T_ocean) + m_air·ΔSST_carried, where the continentality
//!   c = 1 − max(m_air, sea share within `coast_km`) and m_air is the maritime influence the wind
//!   brought (`lapse.rs`). Then the lapse rate on the ground's height.
//! - **Precipitation**, mm/yr: max(`precip_min_mm`, base + orographic) where the orographic rate
//!   is positive, base × max(`oro_lee_floor`, 1 + orographic/base) in the lee, with
//!   base = P_belt(φ′) × subsidence(basin, φ′) × q × e^(0.07·ΔSST) × plateau(z) (0.07/K: the
//!   Clausius–Clapeyron scaling of the air's water) and orographic = Σ_regimes w·P_SB × 365 ×
//!   `oro_hours_per_day` × q × e^(0.067·(T_sl − `oro_ref_temp_c`)) (Smith & Barstad's rate for each
//!   wind regime, weighted by the month's regime weights; c_w scales with the saturation vapour
//!   density). q is the moisture after the rain shadow, blended over the month's regimes by whether
//!   each blows from the west or the east.
//!
//! The air's march (`lapse.rs`) runs along each row both ways and is smoothed over
//! `air_mixing_km` before use. Months are equal; a month's precipitation is the rate / 12. Everything runs row by row (rows in
//! parallel, each row serial), so the result does not depend on the thread count.

use std::collections::BTreeMap;
use std::time::Instant;

use deeptime_format::GridDesc;
use rayon::prelude::*;

use super::belts::{self, EASTWARD, REGIME_LAT_DEG, REGIMES};
use super::ebm::{EbmParams, EbmState};
use super::glaciers::{self, IceCover};
use super::koppen;
use super::lapse::{self, MarchParams};
use super::orographic::{self, OroParams};
use super::params::ClimateParams;
use crate::deposits::fields::cell_area_km2;
use crate::surface::grid::{Dims, box_blur};

/// The planet as the climate sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Planet {
    pub axial_tilt_deg: f64,
    pub star_flux_rel: f64,
    pub day_length_h: f64,
    /// R / R⊕ of the Earth-normalized mapping (planetary-scale lengths shrink by it).
    pub earth_scale: f64,
    pub m_per_block_h: f64,
    /// The atmosphere's CO₂, ppm (its forcing lowers each tile's longwave, `co2.rs`).
    pub co2_ppm: f64,
    /// The star's luminosity against today's (Gough 1981; `carbon.rs`), × `star_flux_rel`.
    pub sun_rel: f64,
    /// The orbit inside the recent ice-age window (`orbit.rs`); `None`: circular at the tilt.
    pub orbit: Option<super::orbit::OrbitAt>,
}

impl Planet {
    /// The EBM of climate section `p` on this planet.
    pub fn ebm(&self, p: &ClimateParams) -> EbmParams {
        let mut ep =
            EbmParams::from_climate(p, self.axial_tilt_deg, self.star_flux_rel * self.sun_rel);
        ep.co2 = Some(super::co2::Forcing::new(self.co2_ppm));
        if let Some(o) = self.orbit {
            ep.obliquity_rad = o.obliquity_deg.to_radians();
            ep.orbit = o.orbit();
        }
        ep
    }
}

/// What a snapshot needs.
pub struct Input<'a> {
    pub grid: &'a GridDesc,
    /// Block metric (cell side in map metres).
    pub d: Dims,
    /// Latitude of every row, degrees.
    pub lat: &'a [f64],
    /// Elevation of every cell, m (the plate model's datum).
    pub h: &'a [f64],
    pub sea: f64,
    pub p: &'a ClimateParams,
    pub planet: Planet,
    /// Keep every cell's twelve months ([`Snapshot::t_sl_month`], [`Snapshot::p_month`]); the
    /// stage asks for them at its last snapshot, which the bake stores (P1-25).
    pub monthly: bool,
    /// The previous step's land ice (thickness, m) and Köppen classes, for the per-cell
    /// climate's ice-sheet and desert albedo (`None`: the band shares, no desert).
    pub ice_prev: Option<&'a [f32]>,
    pub koppen_prev: Option<&'a [f32]>,
    /// The zonal model's correction from the last snapshot (the per-cell climate's; empty:
    /// none), the first guess of this snapshot's fit.
    pub correction: &'a super::percell::Correction,
}

/// Per-row zonal references the geographic anomalies are measured from (sea-level °C; ln mm/yr).
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Refs {
    pub t_mean: Vec<f64>,
    pub t_warm: Vec<f64>,
    pub t_cold: Vec<f64>,
    pub ln_precip: Vec<f64>,
}

/// A snapshot's results, per cell unless stated.
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub t_mean: Vec<f32>,
    pub t_warm: Vec<f32>,
    pub t_cold: Vec<f32>,
    /// Annual precipitation, mm/yr.
    pub precip: Vec<f32>,
    /// P / PET (Thornthwaite), capped at 10.
    pub aridity: Vec<f32>,
    /// ELA above sea level, m (every cell).
    pub ela: Vec<f32>,
    pub koppen: Vec<u8>,
    pub glacier: Vec<u8>,
    /// Each month's sea-level temperature, °C (the lapse rate not applied), and precipitation
    /// rate, mm/yr (a month's share is the rate / 12): twelve per-cell fields, only when
    /// `Input::monthly` (empty otherwise).
    pub t_sl_month: Vec<Vec<f32>>,
    pub p_month: Vec<Vec<f32>>,
    /// Sea-level anomalies from `refs` (moved with the plates between snapshots).
    pub d_mean: Vec<f32>,
    pub d_warm: Vec<f32>,
    pub d_cold: Vec<f32>,
    pub d_ln_precip: Vec<f32>,
    pub refs: Refs,
    pub ice: IceCover,
    /// Each EBM band's glaciated share of its land (the next snapshot's ice sheets).
    pub band_sheet: Vec<f64>,
    pub ebm_t_global: f64,
    pub ebm_net_toa: f64,
    pub ebm_years: u32,
    /// The EBM's last year (the stage's ELA and weathering references).
    pub ebm: Option<super::ebm::EbmResult>,
    /// Mean ITCZ latitude in the months of greatest northward and southward reach, degrees.
    pub itcz_range: (f64, f64),
    /// Land elevation above the sea: median, 90th percentile (m) and share above 2 km (by cell).
    pub land_z: (f64, f64, f64),
    pub millis: BTreeMap<&'static str, f64>,
    /// Sea-ice cover of every sea cell, 0–1 (0 on land): the annual mean, the winter maximum and
    /// the summer minimum of the monthly covers.
    pub sea_ice: Vec<f32>,
    pub sea_ice_max: Vec<f32>,
    pub sea_ice_min: Vec<f32>,
    /// The per-cell climate's figures (`climate.model = per_cell`), JSON.
    pub cell: Option<serde_json::Value>,
    /// The zonal model's correction fitted to the per-cell climate (design §4.1; empty under the
    /// banded climate): the clock runs with it until the next snapshot.
    pub correction: super::percell::Correction,
    /// Where the cells loop ran ([`CellsDevice`]).
    pub cells_on: CellsRun,
}

/// What the cells loop of [`compute_on`] reads in the case a [`CellsDevice`] runs: the per-cell
/// climate with the belts' rain (`rain_model = belts`). Every array is the CPU loop's own.
pub struct CellsInput<'a> {
    pub p: &'a ClimateParams,
    pub w: usize,
    pub h: usize,
    /// Latitude of every row, degrees.
    pub lat: &'a [f64],
    pub land: &'a [bool],
    /// Height above the sea, m (0 at sea).
    pub z: &'a [f64],
    /// The sea's share within `coast_km`.
    pub sea_near: &'a [f64],
    /// The land's share within `percell::INTERIOR_KM` (the winter inversion's).
    pub interior: &'a [f64],
    pub basin: &'a [f64],
    /// The sea's departure from its row, °C.
    pub dsst: &'a [f64],
    /// The air's march, smoothed: moisture, maritime share, basin and ΔSST, each east then west.
    pub air: &'a [Vec<f64>; 8],
    /// Each wind regime's orographic rate (empty where the regime blows on no row).
    pub oro: &'a [Vec<f32>],
    /// The ITCZ of every column and month, degrees.
    pub itcz: &'a [[f64; 12]],
    /// Each month's hemispheric shift of the subtropical highs, degrees.
    pub hemi: [f64; 12],
    /// Each row's monthly insolation, W/m².
    pub q_month: &'a [[f64; 12]],
    /// Each row's onshore share in each month.
    pub onshore: Vec<[f64; 12]>,
    pub pc: &'a super::percell::Coarse,
    /// The lapse rate, °C/m.
    pub gamma: f64,
    pub oro_scale: f64,
    /// e^ln_hydro, the hydrological cycle's scaling.
    pub hydro: f64,
    pub sharp: super::percell::Sharpen,
    pub monthly: bool,
}

/// The cells loop's results, per cell: [`Snapshot`]'s fields of the same names.
#[derive(Clone, Debug, Default)]
pub struct Cells {
    pub t_mean: Vec<f32>,
    pub t_warm: Vec<f32>,
    pub t_cold: Vec<f32>,
    pub precip: Vec<f32>,
    pub aridity: Vec<f32>,
    pub ela: Vec<f32>,
    pub koppen: Vec<u8>,
    pub glacier: Vec<u8>,
    pub sea_ice: Vec<f32>,
    pub sea_ice_max: Vec<f32>,
    pub sea_ice_min: Vec<f32>,
    pub t_sl_month: Vec<Vec<f32>>,
    pub p_month: Vec<Vec<f32>>,
}

/// What one call of a [`CellsDevice`] moved and dispatched.
#[derive(Clone, Copy, Debug, Default)]
pub struct CellsStats {
    /// Row chunks (each one upload, one dispatch per band, one download).
    pub chunks: u32,
    pub dispatches: u32,
    pub bytes_up: u64,
    pub bytes_down: u64,
    /// The largest device memory held at once, bytes.
    pub device_bytes: u64,
    /// Packing and queueing the arrays; running the kernel; reading the results back and
    /// unpacking them.
    pub upload_ms: f64,
    pub kernel_ms: f64,
    pub download_ms: f64,
}

/// Where a snapshot's cells loop ran.
#[derive(Clone, Debug, Default)]
pub enum CellsRun {
    #[default]
    Cpu,
    Device(CellsStats),
    /// The device failed (the message); the CPU loop ran.
    Failed(String),
}

/// Runs the cells loop of [`compute_on`] somewhere other than the CPU: the GPU (`gpu.rs`,
/// feature `gpu`). The climate's dispatch point of the GPU path (`docs/design/gpu-compute.md`);
/// the CPU loop is the fallback and the reference.
pub trait CellsDevice: Send + Sync + std::fmt::Debug {
    /// The device, for the run's notes.
    fn name(&self) -> String;
    /// Host bytes a call on a `w × h` grid holds beyond the CPU loop's (its packed arrays and
    /// read-back copies, and on a device sharing the host's memory its buffers).
    fn host_bytes(&self, w: usize, h: usize, monthly: bool) -> u64;
    /// The cells loop's results for `ci`; on `Err` the caller runs the CPU loop.
    fn cells(&self, ci: &CellsInput<'_>, stats: &mut CellsStats) -> Result<Cells, String>;
}

/// Thornthwaite's (1948) potential evapotranspiration from monthly mean temperatures, mm/yr
/// (uncorrected for day length; Willmott et al. 1985's form above 26.5 °C).
pub fn thornthwaite(t: &[f64; 12]) -> f64 {
    let heat: f64 = t
        .iter()
        .filter(|&&x| x > 0.0)
        .map(|&x| (x / 5.0).powf(1.514))
        .sum();
    if heat <= 0.0 {
        return 0.0;
    }
    let a = 6.75e-7 * heat.powi(3) - 7.71e-5 * heat.powi(2) + 1.792e-2 * heat + 0.49239;
    t.iter()
        .filter(|&&x| x > 0.0)
        .map(|&x| {
            if x > 26.5 {
                -415.85 + 32.24 * x - 0.43 * x * x
            } else {
                16.0 * (10.0 * x / heat).powf(a)
            }
        })
        .sum()
}

/// Band of the EBM holding latitude `lat` (degrees).
pub fn band_of(lat: f64, bands: usize) -> usize {
    (((lat + 90.0) / 180.0 * bands as f64).floor() as isize).clamp(0, bands as isize - 1) as usize
}

/// Per-band means of a per-cell quantity over rows (`None` where a band has no rows: filled from
/// the nearest band that has some).
pub fn band_means(lat: &[f64], w: usize, bands: usize, v: &dyn Fn(usize) -> f64) -> Vec<f64> {
    let mut sum = vec![0.0; bands];
    let mut cnt = vec![0usize; bands];
    for (j, &l) in lat.iter().enumerate() {
        let b = band_of(l, bands);
        for i in 0..w {
            sum[b] += v(j * w + i);
        }
        cnt[b] += w;
    }
    let have: Vec<usize> = (0..bands).filter(|&b| cnt[b] > 0).collect();
    (0..bands)
        .map(|b| {
            if cnt[b] > 0 {
                sum[b] / cnt[b] as f64
            } else {
                let k = have
                    .iter()
                    .min_by_key(|&&k| k.abs_diff(b))
                    .copied()
                    .unwrap_or(b);
                if cnt[k] > 0 {
                    sum[k] / cnt[k] as f64
                } else {
                    0.0
                }
            }
        })
        .collect()
}

/// Interpolates band values to latitude `lat`.
pub fn at_lat(lat_deg: &[f64], v: &[[f64; 12]], lat: f64) -> [f64; 12] {
    let n = lat_deg.len();
    let x = (lat + 90.0) / 180.0 * n as f64 - 0.5;
    let b0 = x.floor().clamp(0.0, (n - 1) as f64) as usize;
    let b1 = (b0 + 1).min(n - 1);
    let f = (x - b0 as f64).clamp(0.0, 1.0);
    std::array::from_fn(|m| v[b0][m] + f * (v[b1][m] - v[b0][m]))
}

fn time(millis: &mut BTreeMap<&'static str, f64>, what: &'static str, since: Instant) {
    *millis.entry(what).or_insert(0.0) += since.elapsed().as_secs_f64() * 1000.0;
}

/// Computes a snapshot on the CPU. `ebm` carries the EBM's state between snapshots (empty: spin
/// up); `sheet` is the previous snapshot's `band_sheet` (empty at the first).
pub fn compute(
    inp: &Input<'_>,
    ebm: &mut EbmState,
    sheet: &[f64],
    cell: &mut super::percell::PerCell,
) -> Snapshot {
    compute_on(inp, ebm, sheet, cell, None)
}

/// [`compute`], with the cells loop on `dev` where it can run it (the per-cell climate with the
/// belts' rain; the banded climate, the moisture budget and the upwind rain stay on the CPU).
pub fn compute_on(
    inp: &Input<'_>,
    ebm: &mut EbmState,
    sheet: &[f64],
    cell: &mut super::percell::PerCell,
    dev: Option<&dyn CellsDevice>,
) -> Snapshot {
    let mut millis = BTreeMap::new();
    let p = inp.p;
    let (w, hgt) = (inp.d.w, inp.d.h);
    let n = w * hgt;
    let dx = inp.d.dx;
    let scale = inp.planet.earth_scale;
    let km = |earth_km: f64| earth_km * 1000.0 * scale; // Earth km → map metres
    let cells = |earth_km: f64| ((km(earth_km) / dx).round() as usize).max(1);
    let gamma = p.lapse_c_per_km / 1000.0;

    let started = Instant::now();
    let land: Vec<bool> = inp.h.iter().map(|&z| z > inp.sea).collect();
    let z: Vec<f64> = inp.h.iter().map(|&z| (z - inp.sea).max(0.0)).collect();
    time(&mut millis, "setup", started);

    // 1. The EBM on the current land fractions.
    let started = Instant::now();
    let mut ep = inp.planet.ebm(p);
    let bands = ep.bands;
    let per_cell = p.model == super::ClimateModel::PerCell;
    if per_cell {
        inp.correction.apply(&mut ep);
    }
    let land_frac = band_means(inp.lat, w, bands, &|c| if land[c] { 1.0 } else { 0.0 });
    let years = if ebm.t_ocean.len() == bands {
        p.ebm_warm_years
    } else {
        p.ebm_spinup_years
    };
    let sheet: Vec<f64> = (0..bands)
        .map(|b| sheet.get(b).copied().unwrap_or(0.0) * p.ice_sheet_feedback)
        .collect();
    let mut er = ep.run(ebm, &land_frac, &sheet, years);
    time(&mut millis, "ebm", started);
    // The per-cell climate (P1-26) on the coarse grid, and the zonal model's flux correction to
    // its zonal means (design §4.1): the clock's model, its references and its state from here
    // are the corrected ones. P1-19 (#14): beside them run the fields only the land mask decides
    // (the land and sea nearby, the basins, the winter inversion's continental share and each
    // row's monthly sunlight), so that their blurs fill the cores the 2-D model's serial parts
    // leave idle. (The timings of the two sides overlap.)
    let hemi: [f64; 12] =
        std::array::from_fn(|m| belts::hemispheric_shift(p, inp.planet.axial_tilt_deg, m));
    let land_fields = || {
        let started = Instant::now();
        let mut land_near: Vec<f64> = land.iter().map(|&l| if l { 1.0 } else { 0.0 }).collect();
        box_blur(&inp.d, &mut land_near, cells(500.0), 2);
        let mut sea_near: Vec<f64> = land.iter().map(|&l| if l { 0.0 } else { 1.0 }).collect();
        box_blur(&inp.d, &mut sea_near, cells(p.coast_km), 2);
        let true_km = |j: usize| dx * inp.lat[j].to_radians().cos().max(0.0) / (1000.0 * scale);
        let basin = belts::basin_position(&inp.d, &land, &true_km, cells(300.0));
        let interior: Vec<f64> = if per_cell {
            let mut v: Vec<f64> = land.iter().map(|&l| if l { 1.0 } else { 0.0 }).collect();
            box_blur(&inp.d, &mut v, cells(super::percell::INTERIOR_KM), 2);
            v
        } else {
            Vec::new()
        };
        let q_month: Vec<[f64; 12]> = inp
            .lat
            .iter()
            .map(|&l| {
                std::array::from_fn(|m| {
                    super::insolation::monthly(ep.s0, l.to_radians(), m, ep.obliquity_rad, 4)
                })
            })
            .collect();
        (
            land_near,
            sea_near,
            basin,
            interior,
            q_month,
            started.elapsed().as_secs_f64() * 1000.0,
        )
    };
    let per_cell_climate = || {
        let mut ms: Vec<(&'static str, f64)> = Vec::new();
        if !per_cell {
            return (None, None, None, ms);
        }
        let started = Instant::now();
        let c = super::percell::run(
            &super::percell::Inputs {
                p,
                w,
                h: hgt,
                land: &land,
                z: &z,
                ice_prev: inp.ice_prev,
                koppen_prev: inp.koppen_prev,
                band_sheet: &sheet,
                ebm: &*ebm,
                ep: EbmParams {
                    flux_corr: Vec::new(),
                    d_face: Vec::new(),
                    ..ep.clone()
                },
                tilt_deg: inp.planet.axial_tilt_deg,
                day_length_h: inp.planet.day_length_h,
            },
            cell,
        );
        ms.push(("ebm2d", started.elapsed().as_secs_f64() * 1000.0));
        for &(what, secs) in &c.timing {
            ms.push((what, secs * 1000.0));
        }
        let started = Instant::now();
        let (f, e) = super::percell::fit_flux(&c, &ep, &land_frac, &sheet, ebm);
        ms.push(("flux_fit", started.elapsed().as_secs_f64() * 1000.0));
        (Some(c), Some(f), Some(e), ms)
    };
    let ((land_near, sea_near, basin, interior, q_month, belts_ms), (mut pc, fit, fitted, pc_ms)) =
        rayon::join(land_fields, per_cell_climate);
    for (what, ms) in pc_ms {
        *millis.entry(what).or_insert(0.0) += ms;
    }
    *millis.entry("belts").or_insert(0.0) += belts_ms;
    if let Some(e) = fitted {
        er = e;
    }
    let started = Instant::now();
    let t_o: Vec<[f64; 12]> = inp
        .lat
        .iter()
        .map(|&l| at_lat(&er.lat_deg, &er.t_ocean, l))
        .collect();
    let t_l: Vec<[f64; 12]> = inp
        .lat
        .iter()
        .map(|&l| at_lat(&er.lat_deg, &er.t_land, l))
        .collect();
    time(&mut millis, "ebm", started);

    // 2. The ITCZ; the sea's departure from its row.
    let started = Instant::now();
    let itcz = match &pc {
        Some(pc) => pc.itcz_l1(),
        None => belts::itcz(
            p,
            w,
            inp.lat,
            &inp.grid.row_weights(),
            &land_near,
            &t_o,
            &t_l,
            cells(p.itcz_smooth_km),
        ),
    };
    // The sea's departure from its row: the per-cell climate's own (its gyres and conveyor), or
    // the prescribed gyre offsets of the banded climate.
    let dsst: Vec<f64> = (0..n)
        .into_par_iter()
        .map(|c| {
            if land[c] {
                0.0
            } else if let Some(pc) = &pc {
                pc.dsst_at(c)
            } else {
                belts::sst_anomaly(p, basin[c], inp.lat[c / w])
            }
        })
        .collect();
    time(&mut millis, "belts", started);

    // 3. Orographic precipitation for each wind regime, over the rows it can reach (P1-19, #14:
    // run beside the winds' marches of section 4, which need none of it).
    let oro_stage = || {
        let started = Instant::now();
        let need: Vec<[bool; REGIMES]> = (0..hgt)
            .into_par_iter()
            .map(|j| {
                let phi = inp.lat[j];
                let (mut lo, mut hi) = (f64::MAX, f64::MIN);
                for s in &itcz {
                    for m in 0..12 {
                        let sh = belts::shift(phi, s[m], hemi[m]);
                        lo = lo.min(sh);
                        hi = hi.max(sh);
                    }
                }
                let mut out = [false; REGIMES];
                let steps = (((hi - lo) / 0.5).ceil() as usize).max(1);
                for k in 0..=steps {
                    let sh = lo + (hi - lo) * k as f64 / steps as f64;
                    let wt = belts::regime_weights(phi, phi - sh);
                    for r in 0..REGIMES {
                        out[r] |= wt[r] > 0.0;
                    }
                }
                out
            })
            .collect();
        let winds = belts::winds(p);
        let omega =
            std::f64::consts::TAU / (inp.planet.day_length_h * 3600.0) * (1.0 + 1.0 / 365.25);
        let oro_p = OroParams {
            conv_time_s: p.oro_conv_time_s,
            fall_time_s: p.oro_fall_time_s,
            nm: p.oro_nm,
            hw_m: p.oro_hw_m,
            cw: p.oro_cw_kg_m3,
        };
        // (The regimes' transforms side by side: P1-19, #14. Each is its own, so the fields are the
        // serial loop's.)
        let oro: Vec<Vec<f32>> = (0..REGIMES)
            .into_par_iter()
            .map(|r| {
                let rows: Vec<usize> = (0..hgt).filter(|&j| need[j][r]).collect();
                let mut field = vec![0.0f32; if rows.is_empty() { 0 } else { n }];
                if let (Some(&j0), Some(&j1)) = (rows.first(), rows.last()) {
                    let margin = 12;
                    let (a, b) = (j0.saturating_sub(margin), (j1 + margin + 1).min(hgt));
                    let band: Vec<f64> = z[a * w..b * w].to_vec();
                    let f = 2.0 * omega * REGIME_LAT_DEG[r].to_radians().sin();
                    let (u, v) = winds[r];
                    let rate =
                        orographic::precipitation(&band, w, b - a, dx, dx, u, v, f, &oro_p, 24);
                    for j in j0..=j1 {
                        for i in 0..w {
                            field[j * w + i] = rate[(j - a) * w + i] as f32;
                        }
                    }
                }
                field
            })
            .collect();
        (oro, started.elapsed().as_secs_f64() * 1000.0)
    };

    // 4. Row by row: the winds' march, twelve months per cell, the annual fields.
    let mp = MarchParams {
        recharge: km(p.ocean_recharge_km),
        moisture_decay: km(p.moisture_decay_km),
        floor: p.moisture_floor,
        maritime_decay: km(p.maritime_decay_km),
        vapour_scale_m: p.vapour_scale_m,
        barrier_recover: km(p.barrier_recover_km),
    };
    let oro_scale = 365.0 * p.oro_hours_per_day;
    // The hydrological cycle follows the global temperature (Held & Soden 2006).
    let ln_hydro = hydro_ln(p, er.t_global);
    // The air's march along every row, both ways; then smoothed over `air_mixing_km` so the
    // rows' independent marches do not leave streaks (the winds have meridional components and
    // mix across rows).
    // (Not under the upwind rain, whose budget carries the moisture and the sea's marks.)
    let upwind_mode = pc.is_some() && p.rain_model == super::RainModel::Upwind;
    let air_stage = || -> [Vec<f64>; 8] {
        if upwind_mode {
            return Default::default();
        }
        let marches: Vec<(lapse::Carried, lapse::Carried)> = (0..hgt)
            .into_par_iter()
            .map(|j| {
                let r0 = j * w;
                let ds = dx * inp.lat[j].to_radians().cos().max(1e-3);
                let (lr, zr) = (&land[r0..r0 + w], &z[r0..r0 + w]);
                let (br, dr) = (&basin[r0..r0 + w], &dsst[r0..r0 + w]);
                let wet = belts::belt_precip(inp.lat[j]) / belts::belt_precip(0.0);
                let mp = MarchParams {
                    moisture_decay: mp.moisture_decay * (1.0 + p.recycling * wet),
                    ..mp
                };
                (
                    lapse::march(lr, zr, br, dr, ds, true, &mp),
                    lapse::march(lr, zr, br, dr, ds, false, &mp),
                )
            })
            .collect();
        // The eight fields (q, maritime, basin and dsst, east then west) blur side by side
        // (P1-19, #14); each is its own, so the values are those of one after the other.
        let gather = |k: usize| -> Vec<f64> {
            let east = k.is_multiple_of(2);
            let mut v: Vec<f64> = marches
                .iter()
                .flat_map(|(e, wv)| {
                    let c = if east { e } else { wv };
                    match k / 2 {
                        0 => &c.q,
                        1 => &c.maritime,
                        2 => &c.basin,
                        _ => &c.dsst,
                    }
                    .iter()
                    .copied()
                })
                .collect();
            box_blur(&inp.d, &mut v, cells(p.air_mixing_km), 2);
            v
        };
        let fields: Vec<Vec<f64>> = (0..8).into_par_iter().map(gather).collect();
        fields.try_into().expect("eight fields")
    };
    let ((oro, oro_ms), (air, march_ms)) = rayon::join(oro_stage, || {
        let started = Instant::now();
        let air = air_stage();
        (air, started.elapsed().as_secs_f64() * 1000.0)
    });
    *millis.entry("orographic").or_insert(0.0) += oro_ms;
    *millis.entry("march").or_insert(0.0) += march_ms;
    let started = Instant::now();
    // The circulation's rain at L1 cell `c` in month `m` before the air's moisture, mm/yr (the
    // belt's rain under the subtropical highs' flanks, the cold sea's stability and the plateau
    // factor), and the moisture the march brings, 0–1.
    let plateau = |zz: f64| (-(zz - p.plateau_m).max(0.0) / p.plateau_scale_m).exp();
    // (P1-19, #14: the hemispheric shift of month `m` is `hemi[m]`, found once; its limit `k` is
    // the planet's.)
    let onshore_k = (p.subtropical_shift_deg * inp.planet.axial_tilt_deg.to_radians().sin()
        / 23.4f64.to_radians().sin())
    .max(1e-9);
    let onshore = |phi: f64, m: usize| -> f64 {
        let h = hemi[m];
        (0.5 + 0.5 * h.signum() * phi.signum() * (h.abs() / onshore_k).min(1.0)).clamp(0.0, 1.0)
    };
    // The circulation's rain at L1 cell `c` given its belt latitude `pb`, the regime weights `wt`
    // there, the onshore share `on` of its row and month and the plateau factor `plat` of its
    // height (each found once per row, month or cell by the caller, P1-19).
    let circ_at = |c: usize,
                   pb: f64,
                   wt: &[f64; REGIMES],
                   on: f64,
                   plat: f64,
                   carry: Option<(f64, f64)>|
     -> (f64, f64) {
        let fe: f64 = (0..REGIMES).filter(|&r| EASTWARD[r]).map(|r| wt[r]).sum();
        let fw = 1.0 - fe;
        // Summer onshore flow round the subtropical highs' western flanks: east coasts from 15°
        // to 45° get easterly (sea) air for moisture whatever the belt says.
        let fw_q = fw.max(on * p.onshore * belts::bump(pb.abs(), 30.0, 15.0));
        let fe_q = 1.0 - fw_q;
        let (q, a_eff, d_eff) = if let (true, Some((a, d))) = (land[c], carry) {
            (1.0, a, d)
        } else if land[c] {
            let q = fe_q * air[0][c] + fw_q * air[1][c];
            let mar = fe * air[2][c] + fw * air[3][c];
            let bas_q = fe_q * air[4][c] + fw_q * air[5][c];
            let mar_q = fe_q * air[2][c] + fw_q * air[3][c];
            let dcar = fe * air[6][c] + fw * air[7][c];
            (q, bas_q * mar_q, mar * dcar)
        } else {
            let q = if air[0].is_empty() {
                1.0
            } else {
                fe_q * air[0][c] + fw_q * air[1][c]
            };
            (q, basin[c], dsst[c])
        };
        let base = belts::belt_precip(pb)
            * belts::subsidence_factor(p, a_eff, pb)
            * (0.07 * d_eff).exp() // ledger: MOI-063
            * plat;
        (base, q)
    };
    // The same for one cell and month, finding the rest (the upwind rain's coarse averages).
    let circ = |c: usize, m: usize, carry: Option<(f64, f64)>| -> (f64, f64) {
        let (i, j) = (c % w, c / w);
        let phi = inp.lat[j];
        let pb = phi - belts::shift(phi, itcz[i][m], hemi[m]);
        let wt = belts::regime_weights(phi, pb);
        circ_at(c, pb, &wt, onshore(phi, m), plateau(z[c]), carry)
    };
    // The upwind rain (`rain_model = upwind`, `upwind.rs`): the circulation's rain at the
    // reference water, averaged over each coarse cell (weighted by the L1 rows' areas), kg/m²/s,
    // its land under the sea marks the budget's vapour carries (`carry`: none on the first pass);
    // the budget dries it along the wind. The first pass sets the mountains' sink and the marks
    // the second uses.
    let mut up_src: (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());
    let mut carry: Option<Carry> = None;
    let carry_at =
        |g: &super::coarse::Grid, cy: &Option<Carry>, c: usize, m: usize| -> Option<(f64, f64)> {
            let (ba, da) = cy.as_ref()?;
            let (mut a, mut d) = (0.0, 0.0);
            for (cc, wt) in g.corners(c) {
                a += wt * ba[cc][m];
                d += wt * da[cc][m];
            }
            Some((a, d))
        };
    let pref_of = |g: &super::coarse::Grid, cy: &Option<Carry>| -> Vec<[f64; 12]> {
        let nc = g.w * g.h;
        let per: Vec<[f64; 12]> = (0..n)
            .into_par_iter()
            .map(|c| {
                std::array::from_fn(|m| {
                    let cy = if land[c] {
                        Some(carry_at(g, cy, c, m).unwrap_or((0.0, 0.0)))
                    } else {
                        None
                    };
                    circ(c, m, cy).0
                })
            })
            .collect();
        // Weighted by the L1 rows' areas; a coarse cell made only of rows past a pole (no area)
        // takes its cells' plain mean, so no coarse cell is left without a circulation rain.
        let rw = inp.grid.row_weights();
        let (mut s, mut a) = (vec![[0.0f64; 12]; nc], vec![0.0f64; nc]);
        let (mut s0, mut a0) = (vec![[0.0f64; 12]; nc], vec![0.0f64; nc]);
        for c in 0..n {
            let cc = g.coarse_of(c);
            let wt = rw[c / w];
            a[cc] += wt;
            a0[cc] += 1.0;
            for m in 0..12 {
                s[cc][m] += wt * per[c][m];
                s0[cc][m] += per[c][m];
            }
        }
        let hydro = ln_hydro.exp();
        (0..nc)
            .map(|cc| {
                let (s, a) = if a[cc] > 0.0 {
                    (s[cc], a[cc])
                } else {
                    (s0[cc], a0[cc])
                };
                s.map(|v| v / a.max(1e-300) * hydro / YEAR_S)
            })
            .collect()
    };
    if let Some(pcm) = pc.as_mut()
        && upwind_mode
    {
        let started = Instant::now();
        let g = pcm.grid.clone();
        let nc = g.w * g.h;
        let (mut s, mut a) = (vec![0.0f64; nc], vec![0.0f64; nc]);
        for c in 0..n {
            if !land[c] {
                let cc = g.coarse_of(c);
                s[cc] += basin[c];
                a[cc] += 1.0;
            }
        }
        up_src.0 = s
            .iter()
            .zip(&a)
            .map(|(s, a)| if *a > 0.0 { s / a } else { 0.0 })
            .collect();
        up_src.1 = pcm.dsst.clone();
        let r = upwind_rain(pcm, p, &pref_of(&g, &None), &up_src, None, &cell.rain_mm);
        carry = Some((r.basin_air.clone(), r.dsst_air.clone()));
        pcm.upwind = Some(r);
        time(&mut millis, "upwind", started);
    }
    // The budget's second pass: the mountain rain the L1 terrain takes out of the column (S_oro,
    // design §3.6), the windward excess over each coarse cell, so less water passes the range.
    if let Some(pcm) = pc.as_mut()
        && (pcm.moisture.is_some() || pcm.upwind.is_some())
    {
        let started = Instant::now();
        let g = pcm.grid.clone();
        // The sink at the coarse rain's moisture, and at the reference moisture (the upwind
        // rain's sink is linear in its vapour).
        let mut sink = vec![[0.0f64; 12]; g.w * g.h];
        let mut sink_ref = vec![[0.0f64; 12]; g.w * g.h];
        let per = (g.k * g.k) as f64;
        for c in 0..n {
            if !land[c] {
                continue;
            }
            let (i, j) = (c % w, c / w);
            let phi = inp.lat[j];
            let cc = g.coarse_of(c);
            for m in 0..12 {
                let wt = belts::regime_weights(phi, phi - belts::shift(phi, itcz[i][m], hemi[m]));
                let o: f64 = (0..REGIMES)
                    .filter(|&r| wt[r] > 0.0 && !oro[r].is_empty())
                    .map(|r| wt[r] * f64::from(oro[r][c]))
                    .sum();
                if o <= 0.0 {
                    continue;
                }
                let (tl, to) = pcm.tiles(c, m);
                let tsl = tl + sea_near[c] * (to - tl);
                let ccf = (0.067 * (tsl - p.oro_ref_temp_c)).exp().clamp(0.05, 3.0);
                let q = pcm.rain_at(c, m).map_or(1.0, |x| x.1);
                let s_ref = o * oro_scale * ccf / YEAR_S / per;
                sink[cc][m] += s_ref * q;
                sink_ref[cc][m] += s_ref;
            }
        }
        if pcm.upwind.is_some() {
            let r = upwind_rain(
                pcm,
                p,
                &pref_of(&g, &carry),
                &up_src,
                Some(&sink_ref),
                &cell.rain_mm,
            );
            cell.rain_mm = r.annual_mm.clone();
            pcm.upwind = Some(r);
        } else {
            let mo = super::percell::budget(pcm, &pcm.mp, Some(&sink), &cell.bucket);
            cell.bucket = mo.bucket.clone();
            pcm.moisture = Some(mo);
        }
        time(&mut millis, "budget", started);
    }
    let refs = zonal_refs(inp.lat, &er, ln_hydro);
    let hydro = ln_hydro.exp();
    struct Row {
        t_mean: Vec<f32>,
        t_warm: Vec<f32>,
        t_cold: Vec<f32>,
        precip: Vec<f32>,
        aridity: Vec<f32>,
        ela: Vec<f32>,
        koppen: Vec<u8>,
        glacier: Vec<u8>,
        t_sl_month: Vec<[f32; 12]>,
        p_month: Vec<[f32; 12]>,
        ice: Vec<[f32; 3]>,
    }
    let sharp = super::percell::Sharpen::new(p.sea_ice_edge_sharpness);
    // The loop below on the device (`CellsDevice`), for the per-cell climate with the belts' rain;
    // the banded climate, the moisture budget and the upwind rain stay on the CPU. After a device
    // error the CPU loop runs, the reference.
    let mut cells_on = CellsRun::Cpu;
    let mut on_device: Option<Cells> = None;
    if let (Some(dev), Some(pcr)) = (dev, pc.as_ref())
        && !upwind_mode
        && pcr.upwind.is_none()
        && pcr.moisture.is_none()
        && air.iter().all(|a| a.len() == n)
    {
        let ci = CellsInput {
            p,
            w,
            h: hgt,
            lat: inp.lat,
            land: &land,
            z: &z,
            sea_near: &sea_near,
            interior: &interior,
            basin: &basin,
            dsst: &dsst,
            air: &air,
            oro: &oro,
            itcz: &itcz,
            hemi,
            q_month: &q_month,
            onshore: inp
                .lat
                .iter()
                .map(|&phi| std::array::from_fn(|m| onshore(phi, m)))
                .collect(),
            pc: pcr,
            gamma,
            oro_scale,
            hydro,
            sharp,
            monthly: inp.monthly,
        };
        let mut st = CellsStats::default();
        match dev.cells(&ci, &mut st) {
            Ok(c) => {
                *millis.entry("cells_upload").or_insert(0.0) += st.upload_ms;
                *millis.entry("cells_download").or_insert(0.0) += st.download_ms;
                on_device = Some(c);
                cells_on = CellsRun::Device(st);
            }
            Err(e) => cells_on = CellsRun::Failed(e),
        }
    }
    // (No rows when the device ran them.)
    let cpu_rows = if on_device.is_some() { 0 } else { hgt };
    let rows: Vec<Row> = (0..cpu_rows)
        .into_par_iter()
        .map(|j| {
            let phi = inp.lat[j];
            let r0 = j * w;
            let zr = &z[r0..r0 + w];
            let mut row = Row {
                t_mean: vec![0.0; w],
                t_warm: vec![0.0; w],
                t_cold: vec![0.0; w],
                precip: vec![0.0; w],
                aridity: vec![0.0; w],
                ela: vec![0.0; w],
                koppen: vec![koppen::SEA; w],
                glacier: vec![0; w],
                t_sl_month: Vec::with_capacity(if inp.monthly { w } else { 0 }),
                p_month: Vec::with_capacity(if inp.monthly { w } else { 0 }),
                ice: vec![[0.0; 3]; w],
            };
            // The row's onshore share each month (P1-19, #14: found once per row, not per cell).
            let on_row: [f64; 12] = std::array::from_fn(|m| onshore(phi, m));
            for i in 0..w {
                let c = r0 + i;
                let mut t = [0.0f64; 12];
                let mut t_sl = [0.0f64; 12];
                let mut pm = [0.0f64; 12];
                let mut ice = [0.0f64; 12];
                // The cell's coarse neighbours and weights, and its height's plateau factor, found
                // once for its twelve months.
                let taps = pc.as_ref().map(|pc| pc.taps(c));
                let plat = plateau(zr[i]);
                for m in 0..12 {
                    let pb = phi - belts::shift(phi, itcz[i][m], hemi[m]);
                    let wt = belts::regime_weights(phi, pb);
                    let tsl = if land[c] {
                        match (&pc, &taps) {
                            // Per cell: the land tile, toward the sea's by the coast's sea share,
                            // less the winter inversion.
                            (Some(pc), Some(tp)) => {
                                let (tl, to) = pc.tiles_at(tp, m);
                                let inv =
                                    super::percell::inversion(p, tl, q_month[j][m], interior[c]);
                                tl + sea_near[c] * (to - tl) - inv
                            }
                            _ => {
                                let fe: f64 =
                                    (0..REGIMES).filter(|&r| EASTWARD[r]).map(|r| wt[r]).sum();
                                let fw = 1.0 - fe;
                                let mar = fe * air[2][c] + fw * air[3][c];
                                let dcar = fe * air[6][c] + fw * air[7][c];
                                let cont = 1.0 - mar.max(sea_near[c]).clamp(0.0, 1.0);
                                t_o[j][m] + cont * (t_l[j][m] - t_o[j][m]) + mar * dcar
                            }
                        }
                    } else {
                        match (&pc, &taps) {
                            (Some(pc), Some(tp)) => {
                                ice[m] = sharp.apply(pc.ice_at_taps(tp, m));
                                pc.sea_tile_at(tp, m)
                            }
                            _ => t_o[j][m] + dsst[c],
                        }
                    };
                    t_sl[m] = tsl;
                    t[m] = tsl - gamma * zr[i];
                    let cy = match &pc {
                        Some(pc) if upwind_mode && land[c] => {
                            Some(carry_at(&pc.grid, &carry, c, m).unwrap_or((0.0, 0.0)))
                        }
                        _ => None,
                    };
                    let (base0, q) = circ_at(c, pb, &wt, on_row[m], plat, cy);
                    let base = base0 * q;
                    let mut o = 0.0;
                    for r in 0..REGIMES {
                        if wt[r] > 0.0 && !oro[r].is_empty() {
                            o += wt[r] * f64::from(oro[r][c]);
                        }
                    }
                    let cc = (0.067 * (tsl - p.oro_ref_temp_c)).exp().clamp(0.05, 3.0);
                    // The coarse rain and its moisture (`rain_model = budget` or `upwind`; the
                    // upwind rain's circulation already holds the hydrological scaling) or the
                    // belts'.
                    let (base, q, k_h) = match pc.as_ref() {
                        Some(pc) if pc.upwind.is_some() => {
                            let (dry, qq) = pc.upwind_at(c, m);
                            (base0 * dry, qq, hydro)
                        }
                        Some(pc) => match pc.rain_at(c, m) {
                            Some((pbg, qq)) => (pbg, qq, 1.0),
                            None => (base * p.belts_rain_scale, q, hydro),
                        },
                        None => (base * p.belts_rain_scale, q, hydro),
                    };
                    let oro_mm = o * oro_scale * q * cc;
                    // Windward rain adds; the lee's negative rate dries the background
                    // proportionally, down to `oro_lee_floor` of it (linear theory's lee drying
                    // over-shoots on high terrain).
                    let rate = if oro_mm >= 0.0 {
                        base + oro_mm
                    } else {
                        base * (1.0 + oro_mm / base.max(1e-9)).max(p.oro_lee_floor)
                    }
                    .max(p.precip_min_mm);
                    pm[m] = rate * k_h / 12.0;
                }
                if !land[c] {
                    row.ice[i] = if pc.is_some() {
                        [
                            (ice.iter().sum::<f64>() / 12.0) as f32,
                            ice.iter().cloned().fold(0.0, f64::max) as f32,
                            ice.iter().cloned().fold(1.0, f64::min) as f32,
                        ]
                    } else {
                        let b = band_of(phi, er.sea_ice.len());
                        [er.sea_ice[b] as f32; 3]
                    };
                }
                if inp.monthly {
                    row.t_sl_month.push(t_sl.map(|v| v as f32));
                    row.p_month.push(pm.map(|v| (v * 12.0) as f32));
                }
                let annual: f64 = pm.iter().sum();
                let mean = t.iter().sum::<f64>() / 12.0;
                row.t_mean[i] = mean as f32;
                row.t_warm[i] = t.iter().cloned().fold(f64::MIN, f64::max) as f32;
                row.t_cold[i] = t.iter().cloned().fold(f64::MAX, f64::min) as f32;
                row.precip[i] = annual as f32;
                let pet = thornthwaite(&t);
                row.aridity[i] = if pet > 0.0 {
                    (annual / pet).min(10.0)
                } else {
                    10.0
                } as f32;
                let t_ela = glaciers::ela_temperature(annual) + p.ela_offset_c;
                let ela = if gamma > 1e-9 {
                    (glaciers::summer(&t_sl) - t_ela) / gamma
                } else if glaciers::summer(&t_sl) < t_ela {
                    -1.0e6
                } else {
                    1.0e6
                };
                row.ela[i] = ela as f32;
                if land[c] {
                    row.koppen[i] = koppen::classify(&t, &pm);
                    row.glacier[i] = u8::from(zr[i] > ela);
                }
            }
            row
        })
        .collect();
    time(&mut millis, "cells", started);

    // 5. Assemble; anomalies; ice.
    let started = Instant::now();
    let mut s = Snapshot {
        refs,
        ebm_t_global: er.t_global,
        ebm_net_toa: er.net_toa(),
        ebm_years: er.years,
        ebm: Some(er.clone()),
        ..Snapshot::default()
    };
    if let Some(c) = on_device {
        s.t_mean = c.t_mean;
        s.t_warm = c.t_warm;
        s.t_cold = c.t_cold;
        s.precip = c.precip;
        s.aridity = c.aridity;
        s.ela = c.ela;
        s.koppen = c.koppen;
        s.glacier = c.glacier;
        s.sea_ice = c.sea_ice;
        s.sea_ice_max = c.sea_ice_max;
        s.sea_ice_min = c.sea_ice_min;
        s.t_sl_month = c.t_sl_month;
        s.p_month = c.p_month;
    }
    s.cells_on = cells_on;
    for r in rows {
        s.t_mean.extend(r.t_mean);
        s.t_warm.extend(r.t_warm);
        s.t_cold.extend(r.t_cold);
        s.precip.extend(r.precip);
        s.aridity.extend(r.aridity);
        s.ela.extend(r.ela);
        s.koppen.extend(r.koppen);
        s.glacier.extend(r.glacier);
        s.sea_ice.extend(r.ice.iter().map(|x| x[0]));
        s.sea_ice_max.extend(r.ice.iter().map(|x| x[1]));
        s.sea_ice_min.extend(r.ice.iter().map(|x| x[2]));
        if inp.monthly {
            if s.t_sl_month.is_empty() {
                s.t_sl_month = (0..12).map(|_| Vec::with_capacity(n)).collect();
                s.p_month = (0..12).map(|_| Vec::with_capacity(n)).collect();
            }
            for m in 0..12 {
                s.t_sl_month[m].extend(r.t_sl_month.iter().map(|v| v[m]));
                s.p_month[m].extend(r.p_month.iter().map(|v| v[m]));
            }
        }
    }
    let lift = |c: usize| gamma * z[c];
    // (The four anomaly fields side by side, cells in parallel: P1-19, #14.)
    {
        let (t_mean, t_warm, t_cold, precip) = (&s.t_mean, &s.t_warm, &s.t_cold, &s.precip);
        let refs = &s.refs;
        let ((d_mean, d_warm), (d_cold, d_ln_precip)) = rayon::join(
            || {
                rayon::join(
                    || {
                        (0..n)
                            .into_par_iter()
                            .map(|c| (f64::from(t_mean[c]) + lift(c) - refs.t_mean[c / w]) as f32)
                            .collect::<Vec<f32>>()
                    },
                    || {
                        (0..n)
                            .into_par_iter()
                            .map(|c| (f64::from(t_warm[c]) + lift(c) - refs.t_warm[c / w]) as f32)
                            .collect::<Vec<f32>>()
                    },
                )
            },
            || {
                rayon::join(
                    || {
                        (0..n)
                            .into_par_iter()
                            .map(|c| (f64::from(t_cold[c]) + lift(c) - refs.t_cold[c / w]) as f32)
                            .collect::<Vec<f32>>()
                    },
                    || {
                        (0..n)
                            .into_par_iter()
                            .map(|c| {
                                (f64::from(precip[c]).max(1e-3).ln() - refs.ln_precip[c / w]) as f32
                            })
                            .collect::<Vec<f32>>()
                    },
                )
            },
        );
        s.d_mean = d_mean;
        s.d_warm = d_warm;
        s.d_cold = d_cold;
        s.d_ln_precip = d_ln_precip;
    }
    s.band_sheet = {
        let glac = band_means(inp.lat, w, bands, &|c| f64::from(s.glacier[c]));
        glac.iter()
            .zip(&land_frac)
            .map(|(g, l)| if *l > 1e-9 { (g / l).min(1.0) } else { 0.0 })
            .collect()
    };
    s.ice = ice_cover(inp.grid, inp.planet.m_per_block_h, &land, &s.glacier);
    let mean_itcz = |m: usize| itcz.iter().map(|s| s[m]).sum::<f64>() / w.max(1) as f64;
    let reach: Vec<f64> = (0..12).map(mean_itcz).collect();
    s.itcz_range = (
        reach.iter().cloned().fold(f64::MAX, f64::min),
        reach.iter().cloned().fold(f64::MIN, f64::max),
    );
    let mut lz: Vec<f64> = (0..n).filter(|&c| land[c]).map(|c| z[c]).collect();
    lz.par_sort_unstable_by(f64::total_cmp);
    if !lz.is_empty() {
        let q = |f: f64| lz[((lz.len() - 1) as f64 * f).round() as usize];
        let high = lz.iter().filter(|&&x| x > 2000.0).count() as f64 / lz.len() as f64;
        s.land_z = (q(0.5), q(0.9), high);
    }
    if let Some(f) = fit {
        let mut st = cell_stats(pc.as_ref().expect("a fit has its 2-D climate"), &er);
        let all: Vec<f64> = f.corr.flux.iter().flatten().copied().collect();
        let d = &f.corr.d_face;
        st["flux_corr"] = serde_json::json!({
            "min_w_m2": all.iter().cloned().fold(f64::MAX, f64::min),
            "max_w_m2": all.iter().cloned().fold(f64::MIN, f64::max),
            "d_min_w_m2_k": d.iter().cloned().fold(f64::MAX, f64::min),
            "d_max_w_m2_k": d.iter().cloned().fold(f64::MIN, f64::max),
            "iterations": f.iterations,
            "worst_k": f.worst_k,
            "rms_k": f.rms_k,
            "first_rms_k": f.first_rms_k,
            "capped": f.capped,
        });
        s.cell = Some(st);
        s.correction = f.corr;
    }
    if let Some(pc) = &pc {
        s.ebm_t_global = pc.r.t_global;
        s.ebm_net_toa = pc.r.net_toa();
        s.ebm_years = pc.r.years;
    }
    time(&mut millis, "assemble", started);
    s.millis = millis;
    s
}

const YEAR_S: f64 = 365.25 * 86_400.0;

/// The upwind rain's monthly sea marks per coarse cell: the highs' side and the sea-surface
/// departure the vapour carries.
type Carry = (Vec<[f64; 12]>, Vec<[f64; 12]>);

/// The upwind rain on the snapshot's coarse climate `pcm` (warm-started from its last solution,
/// whose annual rain then sets the land's recycling; else from `annual`, the last snapshot's).
fn upwind_rain(
    pcm: &super::percell::Coarse,
    p: &ClimateParams,
    p_ref: &[[f64; 12]],
    src: &(Vec<f64>, Vec<f64>),
    sink_ref: Option<&[[f64; 12]]>,
    annual: &[f64],
) -> super::upwind::Rain {
    let prev = pcm.upwind.as_ref();
    let annual_prev: Vec<f64> = prev.map_or_else(|| annual.to_vec(), |r| r.annual_mm.clone());
    super::upwind::rain(
        p,
        &pcm.mp,
        &super::upwind::ClimateIn {
            grid: &pcm.grid,
            land: &pcm.land,
            z_low: &pcm.z_low,
            t_land: &pcm.r.t_land,
            t_ocean: &pcm.r.t_ocean,
            sea_ice: &pcm.r.sea_ice,
            p_ref,
            basin: &src.0,
            dsst: &src.1,
            lapse: p.lapse_c_per_km / 1000.0,
            sink_ref,
            start: prev,
            annual_prev: &annual_prev,
        },
    )
}

/// The per-cell climate's figures for the run's notes: its global mean and balance, the zonal
/// model's against it, the heat transports' northern peaks (PW, and where), the ocean's gyres and
/// conveyor, the moisture budget's closure.
fn cell_stats(pc: &super::percell::Coarse, er: &super::ebm::EbmResult) -> serde_json::Value {
    let g = &pc.grid;
    let edges: Vec<f64> = (1..g.h).map(|e| g.lat_edge[e]).collect();
    let peak = |v: &[f64]| -> (f64, f64) {
        v.iter()
            .zip(&edges)
            .filter(|(_, l)| **l > 0.0)
            .fold(
                (f64::MIN, 0.0),
                |a, (x, l)| if *x > a.0 { (*x, *l) } else { a },
            )
    };
    let ocean: Vec<f64> =
        pc.r.heat_ocean_pw
            .iter()
            .zip(&pc.r.heat_ocean_diff_pw)
            .map(|(a, b)| a + b)
            .collect();
    let (atm, atm_lat) = peak(&pc.r.heat_atm_pw);
    let (oc, oc_lat) = peak(&ocean);
    serde_json::json!({
        "grid": [g.w, g.h],
        "t_global_c": pc.r.t_global,
        "net_toa_w_m2": pc.r.net_toa(),
        "years": pc.r.years,
        "zonal_t_global_c": er.t_global,
        "atm_heat_peak_pw": atm,
        "atm_heat_peak_lat": atm_lat,
        "ocean_heat_peak_pw": oc,
        "ocean_heat_peak_lat": oc_lat,
        "gyre_psi_max_sv": pc.psi_max_sv,
        "gyre_sweeps": pc.gyre_sweeps,
        "sinking": pc.sinking.as_ref().map(|s| serde_json::json!({
            "lat": s.lat, "lon": s.lon, "width_km": s.width_km, "hemisphere": s.hemisphere,
            "cells": s.cells, "sea_km2": s.sea_km2, "sv": s.sv,
        })),
        "moisture_closure_max": pc.moisture.as_ref().map(|m| m.closure.iter().cloned().fold(0.0, f64::max)),
        "upwind": pc.upwind.as_ref().map(|r| serde_json::json!({
            "closure_max": r.closure,
            "sweeps_max": r.sweeps,
        })),
    })
}

/// ln of the hydrological factor at global mean `t_global` °C.
pub fn hydro_ln(p: &ClimateParams, t_global: f64) -> f64 {
    p.hydro_sensitivity_per_k * (t_global - p.hydro_ref_c)
}

/// The zonal references of every row from the EBM's ocean temperatures, with the hydrological
/// factor `ln_hydro` in the precipitation's.
pub fn zonal_refs(lat: &[f64], er: &super::ebm::EbmResult, ln_hydro: f64) -> Refs {
    let t_o: Vec<[f64; 12]> = lat
        .iter()
        .map(|&l| at_lat(&er.lat_deg, &er.t_ocean, l))
        .collect();
    Refs {
        t_mean: t_o.iter().map(|t| t.iter().sum::<f64>() / 12.0).collect(),
        t_warm: t_o
            .iter()
            .map(|t| t.iter().cloned().fold(f64::MIN, f64::max))
            .collect(),
        t_cold: t_o
            .iter()
            .map(|t| t.iter().cloned().fold(f64::MAX, f64::min))
            .collect(),
        ln_precip: lat
            .iter()
            .map(|&l| belts::belt_precip(l).ln() + ln_hydro)
            .collect(),
    }
}

/// Glaciated and land area (true km²) and the ice's centroid.
pub fn ice_cover(grid: &GridDesc, m_per_block_h: f64, land: &[bool], glacier: &[u8]) -> IceCover {
    let w = grid.w as usize;
    let mut out = IceCover::default();
    let (mut vx, mut vy, mut vz) = (0.0, 0.0, 0.0);
    for j in 0..grid.h as usize {
        let a = cell_area_km2(grid, j as u32, m_per_block_h);
        let lat = crate::deposits::fields::lat_of_row(grid, j as u32).to_radians();
        for i in 0..w {
            let c = j * w + i;
            if !land[c] {
                continue;
            }
            out.land_km2 += a;
            if glacier[c] != 0 {
                out.glaciated_km2 += a;
                let lon = i as f64 / w as f64 * std::f64::consts::TAU;
                vx += a * lat.cos() * lon.cos();
                vy += a * lat.cos() * lon.sin();
                vz += a * lat.sin();
            }
        }
    }
    if out.glaciated_km2 > 0.0 {
        out.lat = vz.atan2(vx.hypot(vy)).to_degrees();
        out.lon = vy.atan2(vx).to_degrees().rem_euclid(360.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thornthwaite_is_zero_when_frozen_and_grows_with_heat() {
        assert_eq!(thornthwaite(&[-5.0; 12]), 0.0);
        let temperate = thornthwaite(&[
            2.0, 3.0, 6.0, 9.0, 13.0, 16.0, 18.0, 18.0, 15.0, 11.0, 6.0, 3.0,
        ]);
        let tropical = thornthwaite(&[27.0; 12]);
        assert!((400.0..900.0).contains(&temperate), "{temperate}");
        assert!(tropical > 1400.0, "{tropical}");
    }
}

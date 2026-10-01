// SPDX-License-Identifier: LGPL-3.0-or-later
//! The climate's fine clock (deep-time climate, decisions log 2026-09-29): inside every plate step
//! the carbon cycle, the EBM and the ice advance in steps of `carbon_step_kyr` (200 kyr), or of
//! `orbital_step_kyr` (2 kyr) inside the recent ice-age window, so a snowball's onset and escape
//! and the glacial cycles are resolved while the full geographic snapshot stays every 10 Myr.
//!
//! One step of the clock (Δt):
//! 1. degassing D (the plate step's activity plus active large igneous provinces);
//! 2. the reservoir, implicitly against weathering (`carbon::step_reservoir`);
//! 3. the EBM from its carried state, up to `ebm_clock_years` model years (it stops early once
//!    settled), with the longwave A lowered by the CO₂ forcing, the star at its age and, in the
//!    window, the orbit — nothing is clamped: the ice-albedo feedback may run away to full cover
//!    and back;
//! 4. each band's equilibrium-line shift δ = ΔT_summer / Γ against the last snapshot, and the
//!    ice (a callback: equilibrium ice outside the window, Imbrie-lagged ice sheets inside it)
//!    whose share of each band's land brightens the next EBM step and stops its weathering;
//! 5. weathering on the new climate; the snowball and hothouse trackers.
//!
//! Everything is serial over the steps and bands (the callback's per-cell work is parallel but
//! order-independent), so the result does not depend on the thread count.

use serde::{Deserialize, Serialize};

use super::carbon::{self, Carbon, Kernel, Weathering};
use super::ebm::{EbmParams, EbmResult, EbmState};
use super::glaciers;
use super::orbit::{self, OrbitAt};
use super::params::ClimateParams;
use deeptime_format::EventId;

/// The shortest fine-clock step the carbon cycle halves down to, years.
pub const MIN_STEP_YEARS: f64 = 1000.0;

/// One row of the run's climate curve (one per plate step).
pub const SERIES_COLUMNS: [&str; 22] = [
    "t_myr",
    "co2_ppm",
    "co2_min_ppm",
    "co2_max_ppm",
    "t_global_c",
    "t_global_min_c",
    "t_global_max_c",
    "sea_ice_frac",
    "sea_ice_max_frac",
    "land_ice_frac",
    "ice_volume_km3",
    "sea_level_m",
    "degassing_mol_yr",
    "weathering_mol_yr",
    "sun_rel",
    "snowball",
    "ridge_km2_yr",
    "subduction_km2_yr",
    "weathering_supply_mol_yr",
    "land_frac",
    "kernel_rel",
    "runoff_land_m_yr",
];

/// One row of the ice-age window's curve (one per fine-clock step inside the window).
pub const WINDOW_COLUMNS: [&str; 7] = [
    "t_kyr",
    "t_global_c",
    "land_ice_frac",
    "ice_volume_km3",
    "obliquity_deg",
    "ecc",
    "summer_65n_c",
];

/// A snowball in progress.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpenSnowball {
    pub t_start: f64,
    pub co2_onset_ppm: f64,
    /// The most and the least CO₂ of the plate steps in the 10 Myr before the onset, ppm (their
    /// ratio is the drawdown that led into it).
    pub co2_before_ppm: f64,
    pub co2_low_ppm: f64,
    /// Share of the weathering from |φ| < 30° just before the onset.
    pub tropical_weathering: f64,
    /// Share of the land within 30° of the equator.
    pub tropical_land: f64,
    pub degassing_rel: f64,
    pub sun_rel: f64,
    pub t_min_c: f64,
    pub co2_max_ppm: f64,
}

/// A hothouse in progress.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpenHothouse {
    pub t_start: f64,
    pub peak_c: f64,
    pub peak_t: f64,
    pub co2_peak_ppm: f64,
    /// The escape time of the snowball it began within 5 Myr of (its event, once written, is
    /// the hothouse's cause).
    pub after_escape: Option<f64>,
}

/// What the clock carries from step to step (in the stage's saved state).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ClockState {
    /// The exogenic carbon: atmosphere and ocean.
    pub carbon: Carbon,
    pub started: bool,
    /// Annual-mean sea-ice cover of the ocean after the last clock step (seals the air–sea
    /// exchange).
    pub sea_ice: f64,
    /// Each EBM band's land temperature (annual mean) and ice-covered share of its land after
    /// the last clock step.
    pub t_land: Vec<f64>,
    pub sheet: Vec<f64>,
    /// Summer land temperature of each band at the last snapshot (the ELA reference), °C.
    pub snap_summer: Vec<f64>,
    /// The EBM's global mean at the last snapshot (precipitation reference), °C.
    pub snap_t_global: f64,
    /// The weathering kernel of the last plate step's climate.
    pub kernel: Kernel,
    /// Large igneous provinces still degassing: (start, years, mol).
    pub lips: Vec<(f64, f64, f64)>,
    /// Events of `world.events` already looked at for provinces.
    pub events_seen: u64,
    pub snowball: Option<OpenSnowball>,
    pub snowballs: u64,
    pub hothouse: Option<OpenHothouse>,
    pub hothouses: u64,
    /// The last snowball's escape time and, once written, its event (a hothouse right after it
    /// is its aftermath).
    pub last_escape: Option<(f64, Option<EventId>)>,
    pub series: Vec<Vec<f32>>,
    pub window: Vec<Vec<f32>>,
    /// Every finished snowball's onset and escape, Myr (world time).
    pub spans: Vec<[f64; 2]>,
    /// The EBM's global mean after the last clock step, °C (seafloor weathering).
    pub t_global: f64,
    /// Degassing and weathering of the last clock step, mol/yr.
    pub degassing: f64,
    pub weathering: f64,
}

/// A decision the clock made that becomes an event (emitted by the stage).
#[derive(Clone, Debug, PartialEq)]
pub enum Happening {
    SnowballOnset,
    SnowballEscape {
        open: OpenSnowball,
        t_end: f64,
        co2_escape_ppm: f64,
        degassing_rel: f64,
    },
    HothouseEnd {
        open: OpenHothouse,
        t_end: f64,
        ongoing: bool,
    },
}

/// What the ice callback is told at every clock step.
pub struct IceStep<'a> {
    pub t_years: f64,
    pub dt_kyr: f64,
    /// ELA shift of every band, m (positive: the line rose).
    pub delta: &'a [f64],
    pub in_window: bool,
}

/// What a plate step's clock needs.
pub struct Input<'a> {
    pub p: &'a ClimateParams,
    pub tilt_deg: f64,
    pub star_flux_rel: f64,
    /// The plate step: (t0, t1], years (world time, 0 at the end of the run).
    pub t0: f64,
    pub t1: f64,
    /// Each EBM band's land fraction.
    pub land_frac: &'a [f64],
    /// Each band's latitude, degrees.
    pub band_lat: &'a [f64],
    /// Degassing of the plate step before provinces, mol/yr.
    pub degassing: f64,
    /// Sea-floor production against Earth's today (seafloor weathering).
    pub ridge_rel: f64,
}

/// What a plate step's clock returns.
pub struct Output {
    pub last: Option<EbmResult>,
    /// (Δt years, δ of every band) of every clock step, in order.
    pub deltas: Vec<(f64, Vec<f64>)>,
    pub happenings: Vec<(f64, Happening)>,
    pub co2_min: f64,
    pub co2_max: f64,
    pub t_min: f64,
    pub t_max: f64,
    pub sea_ice_max: f64,
    pub steps: u32,
    /// Carbon the weathering took out beyond the degassing over the plate step (the drawdown's
    /// alkalinity, which precipitates as carbonate: after a snowball, its cap carbonate), mol.
    pub carbonate_excess_mol: f64,
}

/// The star's luminosity at world time `t` (years) against today's.
pub fn sun_at(p: &ClimateParams, t: f64) -> f64 {
    if p.solar_brightening {
        carbon::gough(p.present_age_gyr + t / 1.0e9, p.present_age_gyr)
    } else {
        1.0
    }
}

/// Whether world time `t` lies inside the recent ice-age window.
pub fn in_window(p: &ClimateParams, t: f64) -> bool {
    p.orbital_window_myr > 0.0 && t > -p.orbital_window_myr * 1.0e6 - 1.0
}

/// The orbit at world time `t`: the series inside the window; the present orbit is circular at
/// the preset's tilt outside it.
pub fn orbit_at(p: &ClimateParams, tilt: f64, t: f64) -> Option<OrbitAt> {
    in_window(p, t).then(|| orbit::at(t, tilt, p.orbital_strength))
}

/// EBM parameters at CO₂ `ppm`, world time `t`.
pub fn ebm_params(inp_p: &ClimateParams, tilt: f64, flux: f64, ppm: f64, t: f64) -> EbmParams {
    let mut ep = EbmParams::from_climate(inp_p, tilt, flux * sun_at(inp_p, t));
    ep.a -= carbon::forcing(ppm);
    if let Some(o) = orbit_at(inp_p, tilt, t) {
        ep.obliquity_rad = o.obliquity_deg.to_radians();
        ep.orbit = o.orbit();
    }
    ep
}

/// Area-weighted annual-mean sea-ice cover of the whole ocean, 0–1.
pub fn sea_ice_frac(er: &EbmResult, land: &[f64]) -> f64 {
    let (mut s, mut a) = (0.0, 0.0);
    for (j, &lat) in er.lat_deg.iter().enumerate() {
        let w = lat.to_radians().cos() * (1.0 - land.get(j).copied().unwrap_or(0.0));
        s += w * er.sea_ice[j];
        a += w;
    }
    if a > 0.0 { s / a } else { 0.0 }
}

/// Each band's annual-mean and summer land temperature, °C.
pub fn land_temps(er: &EbmResult) -> (Vec<f64>, Vec<f64>) {
    let mean = er
        .t_land
        .iter()
        .map(|m| m.iter().sum::<f64>() / 12.0)
        .collect();
    let summer = er.t_land.iter().map(glaciers::summer).collect();
    (mean, summer)
}

/// Runs the clock over one plate step. `ice` is told every step's ELA shifts and returns each
/// band's ice-covered share of its land.
pub fn run(
    inp: &Input<'_>,
    st: &mut ClockState,
    ebm: &mut EbmState,
    ice: &mut dyn FnMut(&IceStep<'_>) -> Vec<f64>,
) -> Output {
    let p = inp.p;
    let bands = inp.land_frac.len();
    let wp = Weathering::from(p);
    // The temperature response of weathering used by the implicit carbon step: the EBM's no-feedback
    // sensitivity 5.35/B K per e-fold of CO₂, CO₂ ∝ M².
    let lambda = 5.35 / p.olr_b_w_m2_k;
    let mut out = Output {
        last: None,
        deltas: Vec::new(),
        happenings: Vec::new(),
        co2_min: f64::MAX,
        co2_max: f64::MIN,
        t_min: f64::MAX,
        t_max: f64::MIN,
        sea_ice_max: 0.0,
        steps: 0,
        carbonate_excess_mol: 0.0,
    };
    if st.t_land.len() != bands {
        st.t_land = vec![0.0; bands];
    }
    if st.sheet.len() != bands {
        st.sheet = vec![0.0; bands];
    }
    if st.snap_summer.len() != bands {
        st.snap_summer = vec![0.0; bands];
    }
    let mut t = inp.t0;
    while t < inp.t1 - 1.0 {
        let win = in_window(p, t + 1.0);
        let step = if win {
            p.orbital_step_kyr * 1.0e3
        } else {
            p.carbon_step_kyr * 1.0e3
        };
        // Do not step over the window's start: the orbit and the lagged ice begin exactly there.
        let mut t_next = (t + step).min(inp.t1);
        let w_start = -p.orbital_window_myr * 1.0e6;
        if p.orbital_window_myr > 0.0 && t < w_start - 1.0 && t_next > w_start {
            t_next = w_start;
        }
        // Avoid a sliver at the end of the plate step.
        if inp.t1 - t_next < 0.25 * step {
            t_next = inp.t1;
        }
        // 1–2. Degassing, and the reservoirs implicitly: weathering at the step's end CO₂, the
        //    land temperatures moved by the EBM's no-feedback response to the CO₂ change (the EBM
        //    corrects them in 3). Where CO₂ would move by more than half (a snowball's end, the
        //    hothouse's drawdown) the step is halved, down to `MIN_STEP_YEARS`, so the EBM and its
        //    ice feedback follow the change instead of a linear guess.
        let ppm_before = st.carbon.ppm();
        let (d, carbon_next) = loop {
            let dt = t_next - t;
            st.lips.retain(|&(t0, years, _)| t0 + years > t);
            let lip: f64 = st
                .lips
                .iter()
                .map(|&(t0, years, mol)| {
                    let overlap = (t_next.min(t0 + years) - t.max(t0)).max(0.0);
                    mol * overlap / years.max(1.0)
                })
                .sum::<f64>()
                / dt.max(1.0);
            let d = inp.degassing + lip;
            if !p.carbon_cycle {
                break (d, st.carbon);
            }
            let (kernel, t_land, sheet) = (&st.kernel, &st.t_land, &st.sheet);
            let t_global = st.t_global;
            let w_of = |ppm: f64| -> f64 {
                let shift = lambda * (ppm.max(1e-9) / ppm_before.max(1e-9)).ln();
                let t: Vec<f64> = t_land.iter().map(|x| x + shift).collect();
                wp.flux(kernel, &t, sheet, ppm)
                    + wp.seafloor_flux(inp.ridge_rel, t_global + shift, ppm)
            };
            let next = st.carbon.step(d, &w_of, 1.0 - st.sea_ice, dt);
            let change = (next.ppm().max(1e-9) / ppm_before.max(1e-9)).ln().abs();
            if change > 1.5f64.ln() && dt > 2.0 * MIN_STEP_YEARS {
                t_next = t + 0.5 * dt;
                continue;
            }
            break (d, next);
        };
        let dt = t_next - t;
        out.carbonate_excess_mol += (st.carbon.total() - carbon_next.total()).max(0.0);
        st.carbon = carbon_next;
        let ppm = st.carbon.ppm();
        // 3. The EBM.
        let ep = ebm_params(p, inp.tilt_deg, inp.star_flux_rel, ppm, t_next);
        let sheet: Vec<f64> = st.sheet.iter().map(|s| s * p.ice_sheet_feedback).collect();
        let er = ep.run(ebm, inp.land_frac, &sheet, p.ebm_clock_years);
        // 4. ELA shifts and the ice.
        let (mean, summer) = land_temps(&er);
        let gamma = (p.lapse_c_per_km / 1000.0).max(1e-9);
        let delta: Vec<f64> = (0..bands)
            .map(|b| (summer[b] - st.snap_summer[b]) / gamma)
            .collect();
        st.sheet = ice(&IceStep {
            t_years: t_next,
            dt_kyr: dt / 1.0e3,
            delta: &delta,
            in_window: win,
        });
        st.t_land = mean;
        // 5. Weathering and the trackers.
        let w = wp.flux(&st.kernel, &st.t_land, &st.sheet, ppm)
            + wp.seafloor_flux(inp.ridge_rel, er.t_global, ppm);
        st.t_global = er.t_global;
        st.degassing = d;
        st.weathering = w;
        let ice_frac = sea_ice_frac(&er, inp.land_frac);
        st.sea_ice = ice_frac;
        out.co2_min = out.co2_min.min(ppm);
        out.co2_max = out.co2_max.max(ppm);
        out.t_min = out.t_min.min(er.t_global);
        out.t_max = out.t_max.max(er.t_global);
        out.sea_ice_max = out.sea_ice_max.max(ice_frac);
        track(inp, st, t_next, ppm, &er, ice_frac, d, &mut out.happenings);
        if win {
            let row = window_row(inp, t_next, &er);
            st.window.push(row);
        }
        out.deltas.push((dt, delta));
        out.last = Some(er);
        out.steps += 1;
        t = t_next;
    }
    out
}

/// Summer (JJA-like: the three warmest months) land temperature at 65° N, °C, for the window's
/// curve.
fn window_row(inp: &Input<'_>, t: f64, er: &EbmResult) -> Vec<f32> {
    let o = orbit_at(inp.p, inp.tilt_deg, t);
    let j = inp
        .band_lat
        .iter()
        .enumerate()
        .min_by(|a, b| (a.1 - 65.0).abs().total_cmp(&(b.1 - 65.0).abs()))
        .map_or(0, |x| x.0);
    vec![
        (t / 1.0e3) as f32,
        er.t_global as f32,
        0.0, // land ice share: filled by the stage
        0.0, // ice volume: filled by the stage
        o.map_or(inp.tilt_deg, |o| o.obliquity_deg) as f32,
        o.map_or(0.0, |o| o.ecc) as f32,
        glaciers::summer(&er.t_land[j]) as f32,
    ]
}

/// The snowball and hothouse trackers at time `t`.
#[allow(clippy::too_many_arguments)]
fn track(
    inp: &Input<'_>,
    st: &mut ClockState,
    t: f64,
    ppm: f64,
    er: &EbmResult,
    ice_frac: f64,
    d: f64,
    happenings: &mut Vec<(f64, Happening)>,
) {
    let p = inp.p;
    let d_rel = d / carbon::DEGASSING_EARTH_MOL_YR;
    match &mut st.snowball {
        None if ice_frac >= p.snowball_on_frac => {
            // Where the weathering came from just before (the drawdown) and how far CO₂ fell.
            let (mut trop_w, mut all_w, mut trop_l, mut all_l) = (0.0, 0.0, 0.0, 0.0);
            for (b, &lat) in inp.band_lat.iter().enumerate() {
                let kb = st.kernel.band.get(b).copied().unwrap_or(0.0);
                let lb = inp.land_frac[b] * lat.to_radians().cos();
                all_w += kb;
                all_l += lb;
                if lat.abs() < 30.0 {
                    trop_w += kb;
                    trop_l += lb;
                }
            }
            // The most and the least CO₂ of the 10 Myr before (the plate steps' curve).
            let recent: Vec<f64> = st
                .series
                .iter()
                .rev()
                .take_while(|r| f64::from(r[0]) * 1.0e6 >= t - 10.0e6)
                .map(|r| f64::from(r[1]))
                .collect();
            let before = recent.iter().copied().fold(ppm, f64::max);
            let low = recent.iter().copied().fold(ppm, f64::min);
            st.snowball = Some(OpenSnowball {
                t_start: t,
                co2_onset_ppm: ppm,
                co2_before_ppm: before,
                co2_low_ppm: low,
                tropical_weathering: if all_w > 0.0 { trop_w / all_w } else { 0.0 },
                tropical_land: if all_l > 0.0 { trop_l / all_l } else { 0.0 },
                degassing_rel: d_rel,
                sun_rel: sun_at(p, t),
                t_min_c: er.t_global,
                co2_max_ppm: ppm,
            });
            happenings.push((t, Happening::SnowballOnset));
        }
        Some(o) => {
            o.t_min_c = o.t_min_c.min(er.t_global);
            o.co2_max_ppm = o.co2_max_ppm.max(ppm);
            if ice_frac < p.snowball_off_frac {
                let open = st.snowball.take().expect("open");
                st.snowballs += 1;
                st.last_escape = Some((t, None));
                happenings.push((
                    t,
                    Happening::SnowballEscape {
                        open,
                        t_end: t,
                        co2_escape_ppm: ppm,
                        degassing_rel: d_rel,
                    },
                ));
            }
        }
        None => {}
    }
    match &mut st.hothouse {
        None if er.t_global > p.hothouse_c => {
            let after = st
                .last_escape
                .filter(|(te, _)| t - te < 5.0e6)
                .map(|(te, _)| te);
            st.hothouse = Some(OpenHothouse {
                t_start: t,
                peak_c: er.t_global,
                peak_t: t,
                co2_peak_ppm: ppm,
                after_escape: after,
            });
        }
        Some(h) => {
            if er.t_global > h.peak_c {
                h.peak_c = er.t_global;
                h.peak_t = t;
            }
            h.co2_peak_ppm = h.co2_peak_ppm.max(ppm);
            if er.t_global < p.hothouse_c - 1.0 {
                let open = st.hothouse.take().expect("open");
                st.hothouses += 1;
                happenings.push((
                    t,
                    Happening::HothouseEnd {
                        open,
                        t_end: t,
                        ongoing: false,
                    },
                ));
            }
        }
        None => {}
    }
}

/// Weathering of kernel `k` for the EBM result `er` at `ppm` with no ice change (the balance
/// search's objective).
pub fn weathering_at(
    p: &ClimateParams,
    k: &Kernel,
    er: &EbmResult,
    sheet: &[f64],
    ppm: f64,
    ridge_rel: f64,
) -> f64 {
    let (mean, _) = land_temps(er);
    let wp = Weathering::from(p);
    wp.flux(k, &mean, sheet, ppm) + wp.seafloor_flux(ridge_rel, er.t_global, ppm)
}

/// The CO₂ at which weathering balances degassing `d` on the warm branch: bisection on ln CO₂,
/// each trial an EBM run from its warm initial state. Returns (ppm, the EBM state of the answer).
/// Where no warm balance exists (the planet freezes before weathering falls to `d`) the answer is
/// the coldest warm state, at the edge of the snowball.
#[allow(clippy::too_many_arguments)]
pub fn balance(
    p: &ClimateParams,
    tilt: f64,
    flux: f64,
    t: f64,
    land: &[f64],
    sheet: &[f64],
    k: &Kernel,
    d: f64,
    ridge_rel: f64,
) -> (f64, EbmState) {
    let (mut lo, mut hi) = (1.0f64.ln(), 1.0e6f64.ln());
    let sheet_fb: Vec<f64> = sheet.iter().map(|s| s * p.ice_sheet_feedback).collect();
    let trial = |x: f64| -> (f64, EbmState) {
        let ppm = x.exp();
        let ep = ebm_params(p, tilt, flux, ppm, t);
        let mut s = ep.initial_state();
        let er = ep.run(&mut s, land, &sheet_fb, p.ebm_spinup_years.max(40));
        (weathering_at(p, k, &er, sheet, ppm, ridge_rel) - d, s)
    };
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if trial(mid).0 > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
        if hi - lo < 1e-4 {
            break;
        }
    }
    let (_, s) = trial(hi);
    (hi.exp(), s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::climate::ebm::EbmState;

    /// (t, ppm, global mean °C, sea-ice cover) of one plate step.
    type Row = (f64, f64, f64, f64);

    /// A zonal planet for the clock: land fraction per band, a weathering kernel from the EBM's
    /// present climate (runoff 0.3 m/yr on all land), and an ice callback that puts ice on the
    /// share of each band's land (ground uniform between 0 and 1 km) above its equilibrium line
    /// (summer temperature at the ELA 1 °C, lapse 6.5 °C/km).
    struct Planet {
        p: ClimateParams,
        land: Vec<f64>,
        lat: Vec<f64>,
    }

    impl Planet {
        fn new(p: ClimateParams, land_of: impl Fn(f64) -> f64) -> Planet {
            let n = p.ebm_bands as usize;
            let lat: Vec<f64> = (0..n)
                .map(|b| -90.0 + (b as f64 + 0.5) * 180.0 / n as f64)
                .collect();
            Planet {
                land: lat.iter().map(|&l| land_of(l)).collect(),
                lat,
                p,
            }
        }

        /// The clock's state in weathering balance at the start, and the EBM with it.
        fn start(&self, t: f64, d: f64) -> (ClockState, EbmState) {
            let ep = ebm_params(&self.p, 23.4, 1.0, 280.0, t);
            let mut s = ep.initial_state();
            let er = ep.run(&mut s, &self.land, &[], 60);
            let (mean, summer) = land_temps(&er);
            let km2 = 5.1e8 / self.lat.len() as f64;
            let kernel = Kernel {
                band: (0..self.lat.len())
                    .map(|b| {
                        let a = km2 * self.lat[b].to_radians().cos() * self.land[b] * 1.57;
                        carbon::cell_kernel(a, 0.3, mean[b], self.p.weathering_temp_per_k)
                    })
                    .collect(),
                ref_t_land: mean.clone(),
                supply: Vec::new(),
            };
            let mut st = ClockState {
                kernel,
                snap_summer: summer,
                t_land: mean,
                sheet: vec![0.0; self.lat.len()],
                started: true,
                ..ClockState::default()
            };
            let (ppm, ebm) = balance(&self.p, 23.4, 1.0, t, &self.land, &st.sheet, &st.kernel, d, 1.0);
            st.carbon = Carbon::at_ppm(ppm);
            (st, ebm)
        }

        /// Runs the clock from `t0` for `myr` in 2-Myr plate steps with degassing `d`, the
        /// weathering strength of step k `strength(k)`; returns the happenings and the (t, ppm,
        /// T, sea ice) of every plate step.
        fn run(
            &self,
            st: &mut ClockState,
            ebm: &mut EbmState,
            t0: f64,
            myr: f64,
            d: f64,
            strength: &dyn Fn(usize) -> f64,
        ) -> (Vec<(f64, Happening)>, Vec<Row>) {
            let gamma = self.p.lapse_c_per_km / 1000.0;
            let mut all = Vec::new();
            let mut rows = Vec::new();
            let mut t = t0;
            let mut k = 0;
            while t < t0 + myr * 1.0e6 - 1.0 {
                let mut p = self.p.clone();
                p.weathering_strength = strength(k);
                k += 1;
                let snap = st.snap_summer.clone();
                let land = self.land.clone();
                let mut ice = |s: &IceStep<'_>| -> Vec<f64> {
                    (0..land.len())
                        .map(|b| {
                            let summer = snap[b] + gamma * s.delta[b];
                            let ela = (summer - 1.0) / gamma;
                            if land[b] > 0.0 {
                                (1.0 - ela / 1000.0).clamp(0.0, 1.0)
                            } else {
                                0.0
                            }
                        })
                        .collect()
                };
                let inp = Input {
                    p: &p,
                    tilt_deg: 23.4,
                    star_flux_rel: 1.0,
                    t0: t,
                    t1: t + 2.0e6,
                    land_frac: &self.land,
                    band_lat: &self.lat,
                    degassing: d,
                    ridge_rel: 1.0,
                };
                let out = run(&inp, st, ebm, &mut ice);
                let er = out.last.expect("the clock ran");
                rows.push((t + 2.0e6, st.carbon.ppm(), er.t_global, st.sea_ice));
                st.series
                    .push(vec![((t + 2.0e6) / 1.0e6) as f32, st.carbon.ppm() as f32]);
                all.extend(out.happenings);
                t += 2.0e6;
            }
            (all, rows)
        }
    }

    fn params() -> ClimateParams {
        ClimateParams {
            orbital_window_myr: 0.0,
            ..ClimateParams::default()
        }
    }

    #[test]
    fn earthlike_conditions_stay_warm() {
        // 30 % land everywhere, Earth's degassing, today's sun: a steady 10–20 °C, no snowball.
        let pl = Planet::new(params(), |_| 0.3);
        let d = carbon::DEGASSING_EARTH_MOL_YR;
        let (mut st, mut ebm) = pl.start(-40.0e6, d);
        let (hap, rows) = pl.run(&mut st, &mut ebm, -40.0e6, 40.0, d, &|_| 1.0);
        assert!(hap.is_empty(), "{hap:?}");
        let (_, ppm, t, ice) = *rows.last().unwrap();
        assert!((5.0..25.0).contains(&t), "T {t}");
        assert!((50.0..3000.0).contains(&ppm), "CO₂ {ppm}");
        assert!(ice < 0.5, "sea ice {ice}");
    }

    #[test]
    fn a_tropical_supercontinent_snowballs_and_degassing_frees_it() {
        // A supercontinent across the tropics under a fainter sun (700 Ma), in balance at first;
        // its weathering then strengthens over 20 Myr (fresh basalt, ×1 → ×3): CO₂ falls, the ice
        // line runs away to the equator, weathering stops under the ice and the degassing builds
        // CO₂ up until the ice melts — into a hothouse.
        let pl = Planet::new(params(), |l| if l.abs() < 25.0 { 0.8 } else { 0.03 });
        let d = carbon::DEGASSING_EARTH_MOL_YR;
        let (mut st, mut ebm) = pl.start(-700.0e6, d);
        // ×1 → ×3 over the first 20 Myr, back to ×1 by 40 Myr (the fresh basalt weathers away).
        let ramp = |k: usize| {
            let k = k as f64;
            1.0 + 2.0 * (k / 10.0).min(1.0) - 2.0 * ((k - 10.0) / 10.0).clamp(0.0, 1.0)
        };
        let (hap, rows) = pl.run(&mut st, &mut ebm, -700.0e6, 80.0, d, &ramp);
        let onset = hap
            .iter()
            .position(|(_, h)| matches!(h, Happening::SnowballOnset))
            .unwrap_or_else(|| panic!("no snowball: {rows:?}"));
        let esc = hap
            .iter()
            .find_map(|(t, h)| match h {
                Happening::SnowballEscape {
                    open,
                    co2_escape_ppm,
                    ..
                } => Some((*t, open.clone(), *co2_escape_ppm)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no escape: {rows:?}"));
        assert!(onset < hap.len());
        let (t_esc, open, co2_esc) = esc;
        assert!(
            open.co2_before_ppm > 1.4 * open.co2_low_ppm,
            "a drawdown: {open:?}"
        );
        assert!(
            co2_esc > 20.0 * open.co2_onset_ppm,
            "CO₂ built up: {open:?} → {co2_esc}"
        );
        assert!(open.tropical_land > 0.7, "{open:?}");
        let dur = (t_esc - open.t_start) / 1.0e6;
        assert!((0.3..30.0).contains(&dur), "{dur} Myr");
        assert!(open.t_min_c < -20.0, "{open:?}");
        // Right after the escape: a hothouse, caused by the snowball.
        let hot = hap.iter().find_map(|(_, h)| match h {
            Happening::HothouseEnd { open, .. } if open.t_start >= t_esc - 1.0 => {
                Some(open.clone())
            }
            _ => None,
        });
        let hot = hot.unwrap_or_else(|| panic!("no hothouse after {t_esc}: {hap:?}"));
        assert!(
            hot.t_start - t_esc < 1.0e6 && hot.peak_c > pl.p.hothouse_c,
            "{hot:?}"
        );
    }

    /// The tropical supercontinent under a weathering ramp (×1 → ×3 over 20 Myr, then `hold`)
    /// with a supply of fresh rock `supply` × today's degassing spread over its land (0: none).
    /// Returns every snowball's (onset, escape), Myr.
    fn scenario(step_kyr: f64, hold: f64, supply: f64, myr: f64) -> Vec<(f64, f64)> {
        let mut p = params();
        p.carbon_step_kyr = step_kyr;
        let pl = Planet::new(p, |l| if l.abs() < 25.0 { 0.8 } else { 0.03 });
        let d = carbon::DEGASSING_EARTH_MOL_YR;
        let (mut st, mut ebm) = pl.start(-700.0e6, d);
        if supply > 0.0 {
            let total: f64 = st.kernel.band.iter().sum();
            st.kernel.supply = st
                .kernel
                .band
                .iter()
                .map(|&k| supply * d * k / total.max(1e-12))
                .collect();
        }
        let ramp = move |k: usize| {
            let k = k as f64;
            if k < 10.0 { 1.0 + 2.0 * k / 10.0 } else { hold }
        };
        let (hap, _) = pl.run(&mut st, &mut ebm, -700.0e6, myr, d, &ramp);
        hap.iter()
            .filter_map(|(t, h)| match h {
                Happening::SnowballEscape { open, .. } => Some((open.t_start / 1.0e6, *t / 1.0e6)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_200_kyr_clock_still_resolves_onset_and_escape() {
        // The same snowball on the 100- and the 200-kyr clock: the adaptive halving keeps the
        // escape's fast drawdown resolved, so onset and escape move by about a million years at most
        // (the slow drawdown crosses the runaway threshold a step or so apart).
        let fine = scenario(100.0, 1.0, 0.0, 60.0);
        let coarse = scenario(200.0, 1.0, 0.0, 60.0);
        assert!(!fine.is_empty() && !coarse.is_empty(), "{fine:?} {coarse:?}");
        let (f, c) = (fine[0], coarse[0]);
        assert!((f.0 - c.0).abs() <= 1.5, "onset {f:?} vs {c:?}");
        assert!((f.1 - c.1).abs() <= 2.0, "escape {f:?} vs {c:?}");
    }

    #[test]
    fn a_supply_limit_lengthens_the_gap_between_snowballs() {
        // Weathering held at ×3: the planet cannot stay warm and refreezes after every escape.
        // Unlimited, the hothouse's weathering draws the CO₂ down within about a million years;
        // limited by the rock its erosion exposes (3× today's degassing, the Earth-like
        // intensity), the drawdown and the gap take far longer.
        let gap = |v: &[(f64, f64)]| -> f64 {
            v.windows(2).map(|p| p[1].0 - p[0].1).fold(f64::MAX, f64::min)
        };
        let free = scenario(200.0, 3.0, 0.0, 120.0);
        let limited = scenario(200.0, 3.0, 3.0, 120.0);
        assert!(free.len() >= 2 && limited.len() >= 2, "{free:?} {limited:?}");
        let (gf, gl) = (gap(&free), gap(&limited));
        eprintln!("GAPS free {gf} limited {gl}: {free:?} {limited:?}");
        assert!(gl > 2.0 * gf && gl > 3.0, "gap {gf} → {gl} Myr");
    }

    /// The recent ice-age window over a northern continent (60 % land from 45° to 80° N, ground
    /// 0–1 km) at a fixed CO₂: each band's ice relaxes (Imbrie & Imbrie) toward the share of its
    /// land above the equilibrium line of the moment. Returns the northern ice share of the land
    /// every clock step.
    fn ice_age_window(ppm: f64, strength: f64) -> Vec<f64> {
        let p = ClimateParams {
            carbon_cycle: false,
            orbital_strength: strength,
            ..ClimateParams::default()
        };
        let pl = Planet::new(
            p.clone(),
            |l| if (45.0..80.0).contains(&l) { 0.6 } else { 0.0 },
        );
        let t0 = -p.orbital_window_myr * 1.0e6;
        let ep = ebm_params(&p, 23.4, 1.0, ppm, t0);
        let mut ebm = ep.initial_state();
        let er = ep.run(&mut ebm, &pl.land, &[], 60);
        let (mean, summer) = land_temps(&er);
        let mut st = ClockState {
            carbon: Carbon::at_ppm(ppm),
            snap_summer: summer.clone(),
            t_land: mean,
            sheet: vec![0.0; pl.lat.len()],
            started: true,
            ..ClockState::default()
        };
        let gamma = p.lapse_c_per_km / 1000.0;
        let mut share = vec![0.0f64; pl.lat.len()];
        let mut series = Vec::new();
        let land = pl.land.clone();
        let lat = pl.lat.clone();
        let mut t = t0;
        while t < -1.0 {
            let mut ice = |s: &IceStep<'_>| -> Vec<f64> {
                let (mut on, mut all) = (0.0, 0.0);
                for b in 0..land.len() {
                    if land[b] <= 0.0 {
                        continue;
                    }
                    let ela = (summer[b] + gamma * s.delta[b] - 1.0) / gamma;
                    let eq = (1.0 - ela / 1000.0).clamp(0.0, 1.0);
                    share[b] =
                        crate::climate::ice::relax(share[b], eq, s.dt_kyr, p.ice_response_kyr);
                    let w = land[b] * lat[b].to_radians().cos();
                    on += w * share[b];
                    all += w;
                }
                series.push(on / all);
                share.clone()
            };
            let inp = Input {
                p: &p,
                tilt_deg: 23.4,
                star_flux_rel: 1.0,
                t0: t,
                t1: (t + 2.0e6).min(0.0),
                land_frac: &pl.land,
                band_lat: &pl.lat,
                degassing: 0.0,
                ridge_rel: 1.0,
            };
            run(&inp, &mut st, &mut ebm, &mut ice);
            t = (t + 2.0e6).min(0.0);
        }
        series
    }

    #[test]
    fn orbital_forcing_makes_glacial_cycles_on_a_cold_planet() {
        let stats = |v: &[f64]| {
            let mut s = v.to_vec();
            s.sort_by(f64::total_cmp);
            let (lo, hi) = (s[s.len() / 20], s[s.len() * 19 / 20]);
            // Swings: local maxima standing 0.005 above the lows on either side.
            let mut peaks = 0;
            let mut rising = true;
            let mut low = v[0];
            let mut high = v[0];
            for &x in v {
                if rising {
                    if x > high {
                        high = x;
                    } else if high - x > 0.005 && high - low > 0.005 {
                        peaks += 1;
                        rising = false;
                        low = x;
                    }
                } else if x < low {
                    low = x;
                } else if x - low > 0.005 {
                    rising = true;
                    high = x;
                }
            }
            (lo, hi, peaks)
        };
        // Cold enough (60 ppm on this zonal planet, whose lowland summers are mild): the northern
        // ice waxes and wanes with the orbit, many cycles in 3 Myr (41-kyr obliquity, 23-kyr
        // precession, paced by eccentricity).
        let cold = ice_age_window(60.0, 1.0);
        let (lo, hi, peaks) = stats(&cold);
        assert_eq!(cold.len(), 1500, "2-kyr steps over 3 Myr");
        assert!(
            hi - lo > 0.015 && peaks >= 20,
            "cold: {lo:.3}..{hi:.3}, {peaks} cycles"
        );
        // No orbital swing: no cycles.
        let (lo0, hi0, _) = stats(&ice_age_window(60.0, 0.0));
        assert!(hi0 - lo0 < 0.2 * (hi - lo), "still: {lo0:.3}..{hi0:.3}");
        // A warm planet (1,500 ppm): hardly any ice to cycle.
        let (_, hi_w, _) = stats(&ice_age_window(1500.0, 1.0));
        assert!(hi_w < 0.5 * hi, "warm: up to {hi_w:.3} against {hi:.3}");
    }
}

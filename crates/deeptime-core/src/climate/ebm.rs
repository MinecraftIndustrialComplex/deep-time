// SPDX-License-Identifier: LGPL-3.0-or-later
//! Zonal seasonal energy balance with the ice-albedo feedback (plan §8.3 item 1).
//!
//! The equations are climlab's diffusive EBM (`climlab.model.ebm`, MIT; its `EBM_seasonal` with
//! `StepFunctionAlbedo`, `AplusBT` and `MeridionalHeatDiffusion`), ported as equations, not code:
//!
//! C ∂T/∂t = (1 − α) Q(φ, t) − (A + B·T) + (1/cos φ) ∂/∂φ (cos φ · D · ∂T̄/∂φ),
//!
//! α = a₀ + a₂·P₂(sin φ) where ice-free and `albedo_ice` where frozen (a step smoothed over ±3 °C
//! about `ice_temp_c`), Q the daily-mean insolation of `insolation.rs`. The albedo defaults are
//! climlab's (`EBM_seasonal`'s a₀ 0.33 and a₂ 0.25, a_i 0.62, T_f −10 °C, 90 bands); A, B and D
//! are North, Cahalan & Coakley 1981's Earth-calibrated 203.3 W/m², 2.09 W/m²/K and
//! 0.649 W/m²/K (with climlab's 210, 2 and 0.555 this model's Earth ran 3 °C cold: global mean
//! 12.6 °C against 15.6 °C, zonal means 27.7 °C at the equator, 18.4 at 31°N, 6.3 at 51°N, 0.6 at
//! 61°N, −10.5 at 81°N — Earth's are about 27, 20, 6, −1, −18).
//!
//! **Extension (Deep Time's, documented):** each band holds land (heat capacity `land_depth_m` of
//! water: the ground and the air above it) and sea, weighted by the band's land fraction f from
//! the current topography, so continents get their large seasonal cycle and the sea its small one.
//! The sea is a surface layer (the same small heat capacity as land) over a slab mixed layer
//! (`ocean_depth_m`), coupled at 30 W/m²/K when open — the surface follows the slab — and at the
//! conductance of the sea ice when ice covers it, so
//! polar seas have cold winters. The slab cannot cool below sea water's freezing point, −1.8 °C:
//! the heat it loses there grows ice (stored as latent heat, released first when it warms again);
//! the ice conducts k/h (k = 2.2 W/m/K), melts from the top when the air above it thaws, and
//! brightens the sea's surface in proportion to its cover (full at 0.5 m). The slabs also carry
//! heat poleward by diffusion through the sea's share of each band (`ocean_diffusivity_w_m2_k`,
//! the ocean's part of the poleward transport). Land and the sea's surface share the band's radiation law and the
//! diffusion of the band-mean surface temperature T̄ = f·T_land + (1 − f)·T_sea (the atmosphere
//! mixes zonally), and exchange heat at `land_ocean_exchange_w_m2_k` in proportion to the other's
//! share. Ice sheets (the band's glaciated share of land, from the previous snapshot) keep the ice
//! albedo all year.
//!
//! **Numerics.** Backward Euler for the local terms (a 3 × 3 solve per band) and for the diffusion
//! (a tridiagonal solve of T̄, whose change is shared out by heat capacity), so the step can be
//! days long. Every exchange is symmetric and the diffusion is in flux form, so energy is conserved
//! exactly: over a year in equilibrium the absorbed sunlight equals the outgoing longwave.
//! Deterministic: serial over bands.

use serde::{Deserialize, Serialize};

use super::insolation;

/// Heat capacity of one metre of water, J/m²/K (ρ c_p = 1000 × 4181).
pub const WATER_J_M3_K: f64 = 4.181e6;
const YEAR_S: f64 = 365.25 * 86_400.0;

/// The EBM's parameters (from `ClimateParams` and the planet).
#[derive(Clone, Debug, PartialEq)]
pub struct EbmParams {
    pub s0: f64,
    pub obliquity_rad: f64,
    pub a: f64,
    pub b: f64,
    pub d: f64,
    pub a0: f64,
    pub a2: f64,
    pub ai: f64,
    pub ice_temp_c: f64,
    pub c_ocean: f64,
    pub c_land: f64,
    pub exchange: f64,
    /// Meridional heat diffusivity of the slab ocean (ocean heat transport), W/m²/K.
    pub d_ocean: f64,
    pub bands: usize,
    pub steps_per_year: usize,
}

/// The model's state at the start of a year (what a warm start continues from).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EbmState {
    /// The sea's surface (and the air above it), °C.
    pub t_ocean: Vec<f64>,
    pub t_land: Vec<f64>,
    /// The sea's mixed layer, °C.
    pub t_mixed: Vec<f64>,
    /// Sea ice, as the latent heat it holds, J/m².
    pub ice_j_m2: Vec<f64>,
}

/// Freezing point of sea water, °C.
pub const SEA_FREEZE_C: f64 = -1.8;
/// Latent heat of a cubic metre of sea ice (ρ 917 kg/m³ × L 334 kJ/kg), J/m³.
pub const ICE_J_M3: f64 = 917.0 * 3.34e5;
/// Ice thickness that covers the sea completely, m.
pub const FULL_ICE_M: f64 = 0.5;
/// Thermal conductivity of sea ice, W/m/K (the surface–slab coupling is k/h under ice: thicker
/// ice grows more slowly, Stefan's law).
pub const ICE_W_M_K: f64 = 2.2;

/// Coupling of the sea's surface to its open mixed layer, W/m²/K.
pub const OPEN_WATER_W_M2_K: f64 = 30.0;

/// Monthly means of the last year run.
#[derive(Clone, Debug, PartialEq)]
pub struct EbmResult {
    /// Band centres, degrees, south to north.
    pub lat_deg: Vec<f64>,
    pub t_ocean: Vec<[f64; 12]>,
    pub t_land: Vec<[f64; 12]>,
    /// Area-weighted annual means of the last year, W/m²: absorbed shortwave, outgoing longwave.
    pub asr: f64,
    pub olr: f64,
    /// Area-weighted annual mean surface temperature, °C.
    pub t_global: f64,
    pub years: u32,
}

impl EbmResult {
    /// ASR − OLR, W/m² (0 in energy balance).
    pub fn net_toa(&self) -> f64 {
        self.asr - self.olr
    }
}

/// Smooth step from 0 (at `a`) to 1 (at `b`).
pub fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl EbmParams {
    /// The EBM of a climate section on a planet with this tilt and star flux (× Earth's).
    pub fn from_climate(
        c: &super::ClimateParams,
        axial_tilt_deg: f64,
        star_flux_rel: f64,
    ) -> EbmParams {
        EbmParams {
            s0: c.solar_constant_w_m2 * star_flux_rel,
            obliquity_rad: axial_tilt_deg.to_radians(),
            a: c.olr_a_w_m2,
            b: c.olr_b_w_m2_k,
            d: c.diffusivity_w_m2_k,
            a0: c.albedo_a0,
            a2: c.albedo_a2,
            ai: c.albedo_ice,
            ice_temp_c: c.ice_temp_c,
            c_ocean: c.ocean_depth_m * WATER_J_M3_K,
            c_land: c.land_depth_m * WATER_J_M3_K,
            exchange: c.land_ocean_exchange_w_m2_k,
            d_ocean: c.ocean_diffusivity_w_m2_k,
            bands: c.ebm_bands as usize,
            steps_per_year: c.ebm_steps_per_year as usize,
        }
    }

    fn band_lat(&self, j: usize) -> f64 {
        (-90.0 + (j as f64 + 0.5) * 180.0 / self.bands as f64).to_radians()
    }

    /// Ice share of a surface at temperature `t` (1 frozen, 0 open).
    fn frozen(&self, t: f64) -> f64 {
        1.0 - smoothstep(self.ice_temp_c - 3.0, self.ice_temp_c + 3.0, t)
    }

    fn albedo(&self, free: f64, t: f64, sheet: f64) -> f64 {
        let ice = self.frozen(t).max(sheet);
        free + (self.ai - free) * ice
    }

    /// An initial state: T = 28 − 42 sin²φ °C on both surfaces.
    pub fn initial_state(&self) -> EbmState {
        let t: Vec<f64> = (0..self.bands)
            .map(|j| {
                let s = self.band_lat(j).sin();
                28.0 - 42.0 * s * s
            })
            .collect();
        EbmState {
            t_ocean: t.clone(),
            t_land: t.clone(),
            t_mixed: t,
            ice_j_m2: vec![0.0; self.bands],
        }
    }

    /// Runs up to `years` years from `state` (left at the start of the next year) with each band's
    /// land fraction `land` and ice-sheet share of its land `sheet`; stops early once no band's
    /// annual mean moves by 0.005 K from one year to the next.
    pub fn run(&self, state: &mut EbmState, land: &[f64], sheet: &[f64], years: u32) -> EbmResult {
        let n = self.bands;
        if state.t_ocean.len() != n
            || state.t_land.len() != n
            || state.t_mixed.len() != n
            || state.ice_j_m2.len() != n
        {
            *state = self.initial_state();
        }
        let spy = self.steps_per_year.max(12);
        let per_month = spy / 12;
        let dt = YEAR_S / spy as f64;
        let lat: Vec<f64> = (0..n).map(|j| self.band_lat(j)).collect();
        let cosl: Vec<f64> = lat.iter().map(|l| l.cos()).collect();
        let dphi = std::f64::consts::PI / n as f64;
        // Faces between bands (j − ½, j + ½); the poles' faces have cos = 0 (no flux).
        let face: Vec<f64> = (0..=n)
            .map(|k| {
                if k == 0 || k == n {
                    0.0
                } else {
                    (-90.0 + k as f64 * 180.0 / n as f64).to_radians().cos()
                }
            })
            .collect();
        let free: Vec<f64> = lat
            .iter()
            .map(|l| {
                let x = l.sin();
                self.a0 + self.a2 * 0.5 * (3.0 * x * x - 1.0)
            })
            .collect();
        // Insolation of every step and band (the step's midpoint).
        let q: Vec<Vec<f64>> = (0..spy)
            .map(|s| {
                let lam = insolation::lambda_at((s as f64 + 0.5) / spy as f64);
                lat.iter()
                    .map(|&l| insolation::daily(self.s0, l, lam, self.obliquity_rad))
                    .collect()
            })
            .collect();
        let wsum: f64 = cosl.iter().sum();
        let f: Vec<f64> = (0..n).map(|j| land.get(j).copied().unwrap_or(0.0).clamp(0.0, 1.0)).collect();
        let g_sheet: Vec<f64> = (0..n).map(|j| sheet.get(j).copied().unwrap_or(0.0).clamp(0.0, 1.0)).collect();
        let mut out = EbmResult {
            lat_deg: lat.iter().map(|l| l.to_degrees()).collect(),
            t_ocean: vec![[0.0; 12]; n],
            t_land: vec![[0.0; 12]; n],
            asr: 0.0,
            olr: 0.0,
            t_global: 0.0,
            years: 0,
        };
        let mut prev_mean: Option<Vec<f64>> = None;
        // Tridiagonal scratch.
        let (mut lo, mut di, mut up, mut rhs) =
            (vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![0.0; n]);
        for year in 0..years.max(1) {
            let mut mo = vec![[0.0; 12]; n];
            let mut ml = vec![[0.0; 12]; n];
            let (mut asr, mut olr, mut tg) = (0.0, 0.0, 0.0);
            for (s, qs) in q.iter().enumerate() {
                let month = s / per_month;
                // 1. Local terms, implicit: radiation, OLR, the land–sea exchange and the sea's
                //    surface–slab coupling (through the ice where there is ice).
                for j in 0..n {
                    let (to, tl, tm) = (state.t_ocean[j], state.t_land[j], state.t_mixed[j]);
                    let h_ice = state.ice_j_m2[j] / ICE_J_M3;
                    let cover = (h_ice / FULL_ICE_M).min(1.0);
                    let g = OPEN_WATER_W_M2_K * (1.0 - cover)
                        + (ICE_W_M_K / h_ice.max(0.05)).min(OPEN_WATER_W_M2_K) * cover;
                    let ao = self.albedo(free[j], to, cover);
                    let al = self.albedo(free[j], tl, g_sheet[j]);
                    let so = qs[j] * (1.0 - ao);
                    let sl = qs[j] * (1.0 - al);
                    let (cl, cs, cm) = (self.c_land / dt, self.c_land / dt, self.c_ocean / dt);
                    let (xl, xo) = (self.exchange * (1.0 - f[j]), self.exchange * f[j]);
                    // land:  (cl + B + xl)·Tl − xl·To                 = cl·Tl + Sl − A
                    // sea:   −xo·Tl + (cs + B + xo + g)·To − g·Tm     = cs·To + So − A
                    // slab:  −g·To + (cm + g)·Tm                      = cm·Tm
                    let (r1, r2, r3) = (cl * tl + sl - self.a, cs * to + so - self.a, cm * tm);
                    let a33 = cm + g;
                    let a11 = cl + self.b + xl;
                    let a22 = cs + self.b + xo + g - g * g / a33;
                    let r2 = r2 + g * r3 / a33;
                    let det = a11 * a22 - xl * xo;
                    let tl2 = (r1 * a22 + xl * r2) / det;
                    let mut to2 = (a11 * r2 + xo * r1) / det;
                    let tm2 = (r3 + g * to2) / a33;
                    // The budget of the implicit step: what it absorbed and emitted.
                    let w = cosl[j];
                    asr += w * (f[j] * sl + (1.0 - f[j]) * so);
                    olr += w * (self.a + self.b * (f[j] * tl2 + (1.0 - f[j]) * to2));
                    // Ice melts from the top when the surface above it thaws.
                    if state.ice_j_m2[j] > 0.0 && to2 > 0.0 {
                        let melt = (to2 * self.c_land).min(state.ice_j_m2[j]);
                        state.ice_j_m2[j] -= melt;
                        to2 -= melt / self.c_land;
                    }
                    state.t_ocean[j] = to2;
                    state.t_land[j] = tl2;
                    state.t_mixed[j] = tm2;
                }
                // 2. Diffusion of the band mean, implicit. Land and the sea's surface have the
                //    same heat capacity, so both take the band mean's change.
                if self.d > 0.0 {
                    let mean: Vec<f64> = (0..n)
                        .map(|j| f[j] * state.t_land[j] + (1.0 - f[j]) * state.t_ocean[j])
                        .collect();
                    for j in 0..n {
                        let k = self.d / (cosl[j] * dphi * dphi) * dt / self.c_land;
                        let (kl, ku) = (k * face[j], k * face[j + 1]);
                        lo[j] = -kl;
                        up[j] = -ku;
                        di[j] = 1.0 + kl + ku;
                        rhs[j] = mean[j];
                    }
                    let new = thomas(&lo, &di, &up, &rhs);
                    for j in 0..n {
                        let dtm = new[j] - mean[j];
                        state.t_ocean[j] += dtm;
                        state.t_land[j] += dtm;
                    }
                }
                // 3. Ocean heat transport: diffusion of the slab through the sea's share of each
                //    face between bands (none across land).
                if self.d_ocean > 0.0 {
                    for j in 0..n {
                        let sea = (1.0 - f[j]).max(1e-6);
                        let fl = if j > 0 { (1.0 - f[j]).min(1.0 - f[j - 1]) } else { 0.0 };
                        let fu = if j + 1 < n { (1.0 - f[j]).min(1.0 - f[j + 1]) } else { 0.0 };
                        let k = self.d_ocean / (cosl[j] * dphi * dphi) * dt / (self.c_ocean * sea);
                        let (kl, ku) = (k * face[j] * fl, k * face[j + 1] * fu);
                        lo[j] = -kl;
                        up[j] = -ku;
                        di[j] = 1.0 + kl + ku;
                        rhs[j] = state.t_mixed[j];
                    }
                    state.t_mixed = thomas(&lo, &di, &up, &rhs);
                }
                // 4. Freezing and melting at the slab: it holds at the freezing point while ice
                //    grows, and melts its ice before it warms (the heat moves, none is lost).
                for j in 0..n {
                    let tm = state.t_mixed[j];
                    if tm < SEA_FREEZE_C {
                        state.ice_j_m2[j] += (SEA_FREEZE_C - tm) * self.c_ocean;
                        state.t_mixed[j] = SEA_FREEZE_C;
                    } else if state.ice_j_m2[j] > 0.0 {
                        let melt = ((tm - SEA_FREEZE_C) * self.c_ocean).min(state.ice_j_m2[j]);
                        state.ice_j_m2[j] -= melt;
                        state.t_mixed[j] -= melt / self.c_ocean;
                    }
                }
                for j in 0..n {
                    let (to, tl) = (state.t_ocean[j], state.t_land[j]);
                    mo[j][month] += to / per_month as f64;
                    ml[j][month] += tl / per_month as f64;
                    tg += cosl[j] * (f[j] * tl + (1.0 - f[j]) * to);
                }
            }
            let steps = spy as f64 * wsum;
            out.asr = asr / steps;
            out.olr = olr / steps;
            out.t_global = tg / steps;
            out.t_ocean = mo;
            out.t_land = ml;
            out.years = year + 1;
            let annual: Vec<f64> = (0..n)
                .map(|j| {
                    (0..12)
                        .map(|m| f[j] * out.t_land[j][m] + (1.0 - f[j]) * out.t_ocean[j][m])
                        .sum::<f64>()
                        / 12.0
                })
                .collect();
            if let Some(p) = &prev_mean
                && p.iter().zip(&annual).all(|(a, b)| (a - b).abs() < 0.005)
            {
                break;
            }
            prev_mean = Some(annual);
        }
        out
    }
}

/// Solves a tridiagonal system (Thomas algorithm): lo[j]·x[j−1] + di[j]·x[j] + up[j]·x[j+1] = r[j].
pub fn thomas(lo: &[f64], di: &[f64], up: &[f64], r: &[f64]) -> Vec<f64> {
    let n = di.len();
    let mut c = vec![0.0; n];
    let mut d = vec![0.0; n];
    c[0] = up[0] / di[0];
    d[0] = r[0] / di[0];
    for j in 1..n {
        let m = di[j] - lo[j] * c[j - 1];
        c[j] = up[j] / m;
        d[j] = (r[j] - lo[j] * d[j - 1]) / m;
    }
    let mut x = vec![0.0; n];
    x[n - 1] = d[n - 1];
    for j in (0..n - 1).rev() {
        x[j] = d[j] - c[j] * x[j + 1];
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::climate::ClimateParams;

    pub fn earth() -> EbmParams {
        EbmParams::from_climate(&ClimateParams::default(), 23.4, 1.0)
    }

    #[test]
    fn the_ebm_converges_to_energy_balance() {
        let p = earth();
        let n = p.bands;
        // Earth-like land: 30 % everywhere, more in the north, Antarctica at the south pole.
        let land: Vec<f64> = (0..n)
            .map(|j| {
                let lat = -90.0 + (j as f64 + 0.5) * 2.0;
                if lat < -65.0 { 0.95 } else if lat > 30.0 { 0.5 } else { 0.25 }
            })
            .collect();
        let sheet: Vec<f64> = (0..n)
            .map(|j| if (j as f64 + 0.5) * 2.0 - 90.0 < -65.0 { 1.0 } else { 0.0 })
            .collect();
        let mut st = p.initial_state();
        let r = p.run(&mut st, &land, &sheet, 200);
        assert!(r.years < 200, "converged ({} years)", r.years);
        assert!(r.net_toa().abs() < 0.2, "ASR − OLR = {} W/m²", r.net_toa());
        assert!((5.0..=20.0).contains(&r.t_global), "global mean {} °C", r.t_global);
        // Equator warm, poles cold, the continental interior's seasons larger than the sea's.
        let eq = n / 2;
        let north = n - 8; // ~74°N
        let mean = |a: &[f64; 12]| a.iter().sum::<f64>() / 12.0;
        let range = |a: &[f64; 12]| {
            a.iter().cloned().fold(f64::MIN, f64::max) - a.iter().cloned().fold(f64::MAX, f64::min)
        };
        assert!(mean(&r.t_ocean[eq]) > 20.0, "{}", mean(&r.t_ocean[eq]));
        assert!(mean(&r.t_land[north]) < -5.0, "{}", mean(&r.t_land[north]));
        let mid = n / 2 + 27; // ~55°N
        assert!(
            range(&r.t_land[mid]) > 2.0 * range(&r.t_ocean[mid]),
            "land {} ocean {}",
            range(&r.t_land[mid]),
            range(&r.t_ocean[mid])
        );
        assert!(r.t_land[mid][6] > r.t_land[mid][0] + 15.0, "July warmer than January");
        // A warm restart of the balanced state stays balanced.
        let r2 = p.run(&mut st, &land, &sheet, 3);
        assert!((r2.t_global - r.t_global).abs() < 0.05);
    }

    #[test]
    fn diffusion_conserves_energy_and_a_brighter_star_warms() {
        // No radiation (A = B = 0, dark, black): only the diffusions, the exchanges and the sea
        // ice act, so the heat Σ cos φ·(C·T − latent heat of the ice) is constant.
        let mut p = earth();
        p.s0 = 0.0;
        p.a = 0.0;
        p.b = 1e-12;
        p.exchange = 0.0;
        p.ice_temp_c = -1000.0;
        let n = p.bands;
        let land = vec![0.4; n];
        let mut st = p.initial_state();
        let heat = |st: &EbmState| -> f64 {
            (0..n)
                .map(|j| {
                    let c = (-90.0 + (j as f64 + 0.5) * 2.0f64).to_radians().cos();
                    c * (0.4 * p.c_land * st.t_land[j]
                        + 0.6
                            * (p.c_land * st.t_ocean[j] + p.c_ocean * st.t_mixed[j]
                                - st.ice_j_m2[j]))
                })
                .sum()
        };
        let h0 = heat(&st);
        p.run(&mut st, &land, &[], 2);
        assert!(((heat(&st) - h0) / h0.abs()).abs() < 1e-9);
        // Star flux: +5 % warms the planet.
        let base = earth();
        let mut hot = earth();
        hot.s0 *= 1.05;
        let land = vec![0.3; base.bands];
        let t0 = base.run(&mut base.initial_state(), &land, &[], 60).t_global;
        let t1 = hot.run(&mut hot.initial_state(), &land, &[], 60).t_global;
        assert!(t1 > t0 + 3.0, "{t0} → {t1}");
    }
}


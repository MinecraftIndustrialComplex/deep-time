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
    /// The orbit (circular unless the recent ice-age window sets it, `orbit.rs`).
    pub orbit: insolation::Orbit,
    /// The CO₂ forcing of the atmosphere's CO₂ against 280 ppm, by each tile's temperature
    /// (`co2.rs`); `None`: 280 ppm (none).
    pub co2: Option<super::co2::Forcing>,
    /// The snowball cloud greenhouse on frozen surfaces, W/m² (`snowball_cloud_w_m2`).
    pub snow_cloud: f64,
    /// The per-cell climate's flux correction of every band, [land, sea] tile, W/m², added to
    /// the absorbed sunlight (P1-26 design §4.1: fitted at each 2-D snapshot so this model's band
    /// means match the 2-D model's zonal means; empty: none).
    pub flux_corr: Vec<[f64; 2]>,
    /// The atmosphere's diffusivity of every face between bands (n + 1, the poles' first and
    /// last), W/m²/K: the per-cell climate's transport (P1-26 T6, fitted with `flux_corr`);
    /// empty: `d` everywhere.
    pub d_face: Vec<f64>,
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

/// Sea ice thicker than this stops conducting (the conductance k/h tapers to zero between half
/// this and this, m). It stands for the thick sea glacier of a very cold ocean, whose conduction
/// balances only the geothermal flux (~0.09 W/m²): the ice stops growing there on its own, so no
/// heat is invented or lost, and the latent heat it stores stays small enough to melt within a
/// few model years once the surface above it thaws. The EBM runs years per Myr of geological time
/// (the climate's fine clock), so a kilometre of stored ice would take thousands of model years to
/// melt that the real ice — metres per year under a 50 W/m² surplus — melts in centuries.
/// Present-day perennial sea ice in this model is ~4.5 m thick, below the taper.
pub const SEA_ICE_CAP_M: f64 = 10.0;

/// Monthly means of the last year run.
#[derive(Clone, Debug, PartialEq)]
pub struct EbmResult {
    /// Band centres, degrees, south to north.
    pub lat_deg: Vec<f64>,
    pub t_ocean: Vec<[f64; 12]>,
    pub t_land: Vec<[f64; 12]>,
    /// Annual-mean sea-ice cover of each band's sea, 0–1.
    pub sea_ice: Vec<f64>,
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

/// Insolation shapes by (bands, steps a year, obliquity, eccentricity, perihelion): each step's
/// (shape, distance factor) per band.
type Shapes = Option<([u64; 5], Vec<Vec<(f64, f64)>>)>;

thread_local! {
    /// The last insolation shapes [`EbmParams::run`] made.
    static SHAPES: std::cell::RefCell<Shapes> = const { std::cell::RefCell::new(None) };
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
            orbit: insolation::Orbit::default(),
            co2: None,
            snow_cloud: c.snowball_cloud_w_m2,
            flux_corr: Vec::new(),
            d_face: Vec::new(),
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
        // Each band's area per radian of longitude, sin φ_N − sin φ_S (= 2 sin(Δφ/2) cos φ, so the
        // area weights stay ∝ cos φ): the diffusion is the exact finite-volume form on the sphere,
        // which the per-cell model (`ebm2d.rs`) shares, face length over centre distance.
        let span: Vec<f64> = lat
            .iter()
            .map(|l| (l + 0.5 * dphi).sin() - (l - 0.5 * dphi).sin())
            .collect();
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
        // Insolation of every step and band (the step's midpoint): s₀ times a shape that the
        // bands, the steps, the obliquity and the orbit fix, which stay the same from call to
        // call outside the orbital window, so the shape and the orbit's distance factor are kept
        // per thread and only scaled (the speed work of 2026-10-28; `insolation::daily_orbit`'s
        // operations in its order, the same bits).
        let key = [
            n as u64,
            spy as u64,
            self.obliquity_rad.to_bits(),
            self.orbit.ecc.to_bits(),
            self.orbit.long_peri_rad.to_bits(),
        ];
        let q: Vec<Vec<f64>> = SHAPES.with(|cache| {
            let mut cache = cache.borrow_mut();
            if cache.as_ref().is_none_or(|(k, _)| *k != key) {
                let shapes: Vec<Vec<(f64, f64)>> = (0..spy)
                    .map(|s| {
                        let lam = self.orbit.lambda_at((s as f64 + 0.5) / spy as f64);
                        let df = self.orbit.distance_factor(lam);
                        lat.iter()
                            .map(|&l| (insolation::daily_shape(l, lam, self.obliquity_rad), df))
                            .collect()
                    })
                    .collect();
                *cache = Some((key, shapes));
            }
            let (_, shapes) = cache.as_ref().expect("filled above");
            shapes
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|&(x, df)| insolation::from_shape(self.s0, x) * df)
                        .collect()
                })
                .collect()
        });
        let wsum: f64 = cosl.iter().sum();
        let f: Vec<f64> = (0..n)
            .map(|j| land.get(j).copied().unwrap_or(0.0).clamp(0.0, 1.0))
            .collect();
        let g_sheet: Vec<f64> = (0..n)
            .map(|j| sheet.get(j).copied().unwrap_or(0.0).clamp(0.0, 1.0))
            .collect();
        let mut out = EbmResult {
            lat_deg: lat.iter().map(|l| l.to_degrees()).collect(),
            t_ocean: vec![[0.0; 12]; n],
            t_land: vec![[0.0; 12]; n],
            sea_ice: vec![0.0; n],
            asr: 0.0,
            olr: 0.0,
            t_global: 0.0,
            years: 0,
        };
        let mut prev_mean: Option<(Vec<f64>, Vec<f64>)> = None;
        // The two implicit diffusions' systems depend on the bands only: factored once for the
        // run (P1-17; [`Tri`] solves them as `thomas` did, bit for bit).
        let per_face = self.d_face.len() == n + 1;
        let atm = (self.d > 0.0 || per_face).then(|| {
            let (mut lo, mut di, mut up) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
            for j in 0..n {
                let k = 1.0 / (span[j] * dphi) * dt / self.c_land;
                let (dl, du) = if per_face {
                    (self.d_face[j], self.d_face[j + 1])
                } else {
                    (self.d, self.d)
                };
                let (kl, ku) = (k * dl * face[j], k * du * face[j + 1]);
                lo[j] = -kl;
                up[j] = -ku;
                di[j] = 1.0 + kl + ku;
            }
            Tri::new(&lo, &di, &up)
        });
        let slab = (self.d_ocean > 0.0).then(|| {
            let (mut lo, mut di, mut up) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
            for j in 0..n {
                let sea = (1.0 - f[j]).max(1e-6);
                let fl = if j > 0 {
                    (1.0 - f[j]).min(1.0 - f[j - 1])
                } else {
                    0.0
                };
                let fu = if j + 1 < n {
                    (1.0 - f[j]).min(1.0 - f[j + 1])
                } else {
                    0.0
                };
                let k = self.d_ocean / (span[j] * dphi) * dt / (self.c_ocean * sea);
                let (kl, ku) = (k * face[j] * fl, k * face[j + 1] * fu);
                lo[j] = -kl;
                up[j] = -ku;
                di[j] = 1.0 + kl + ku;
            }
            Tri::new(&lo, &di, &up)
        });
        let (mut mean, mut new, mut new_slab) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
        for year in 0..years.max(1) {
            let mut mo = vec![[0.0; 12]; n];
            let mut ml = vec![[0.0; 12]; n];
            let mut si = vec![0.0; n];
            let (mut asr, mut olr, mut tg) = (0.0, 0.0, 0.0);
            for (s, qs) in q.iter().enumerate() {
                let month = s / per_month;
                // 1. Local terms, implicit: radiation, OLR, the land–sea exchange and the sea's
                //    surface–slab coupling (through the ice where there is ice).
                for j in 0..n {
                    let (to, tl, tm) = (state.t_ocean[j], state.t_land[j], state.t_mixed[j]);
                    let h_ice = state.ice_j_m2[j] / ICE_J_M3;
                    let cover = (h_ice / FULL_ICE_M).min(1.0);
                    let insulated = smoothstep(0.5 * SEA_ICE_CAP_M, SEA_ICE_CAP_M, h_ice);
                    let g = OPEN_WATER_W_M2_K * (1.0 - cover)
                        + (ICE_W_M_K / h_ice.max(0.05)).min(OPEN_WATER_W_M2_K)
                            * cover
                            * (1.0 - insulated);
                    let ao = self.albedo(free[j], to, cover);
                    let al = self.albedo(free[j], tl, g_sheet[j]);
                    let [fl_corr, fo_corr] = self.flux_corr.get(j).copied().unwrap_or([0.0; 2]);
                    let so = qs[j] * (1.0 - ao) + fo_corr;
                    let sl = qs[j] * (1.0 - al) + fl_corr;
                    let (cl, cs, cm) = (self.c_land / dt, self.c_land / dt, self.c_ocean / dt);
                    let (xl, xo) = (self.exchange * (1.0 - f[j]), self.exchange * f[j]);
                    // Each tile's longwave A + B·T, less the CO₂ forcing at its temperature and
                    // the snowball cloud greenhouse on its frozen share (`co2.rs`).
                    let (a_l, b_l) = super::co2::longwave(
                        self.a,
                        self.b,
                        self.co2.as_ref(),
                        tl,
                        self.snow_cloud,
                        self.frozen(tl).max(g_sheet[j]),
                    );
                    let (a_o, b_o) = super::co2::longwave(
                        self.a,
                        self.b,
                        self.co2.as_ref(),
                        to,
                        self.snow_cloud,
                        cover,
                    );
                    // land:  (cl + B_l + xl)·Tl − xl·To                 = cl·Tl + Sl − A_l
                    // sea:   −xo·Tl + (cs + B_o + xo + g)·To − g·Tm     = cs·To + So − A_o
                    // slab:  −g·To + (cm + g)·Tm                        = cm·Tm
                    let (r1, r2, r3) = (cl * tl + sl - a_l, cs * to + so - a_o, cm * tm);
                    let a33 = cm + g;
                    let a11 = cl + b_l + xl;
                    let a22 = cs + b_o + xo + g - g * g / a33;
                    let r2 = r2 + g * r3 / a33;
                    let det = a11 * a22 - xl * xo;
                    let tl2 = (r1 * a22 + xl * r2) / det;
                    let mut to2 = (a11 * r2 + xo * r1) / det;
                    let tm2 = (r3 + g * to2) / a33;
                    // The budget of the implicit step: what it absorbed and emitted.
                    let w = cosl[j];
                    asr += w * (f[j] * sl + (1.0 - f[j]) * so);
                    olr += w * (f[j] * (a_l + b_l * tl2) + (1.0 - f[j]) * (a_o + b_o * to2));
                    // Ice melts from the top when the surface above it thaws.
                    if state.ice_j_m2[j] > 0.0 && to2 > 0.0 {
                        let melt = (to2 * self.c_land).min(state.ice_j_m2[j]);
                        state.ice_j_m2[j] -= melt;
                        to2 -= melt / self.c_land;
                    }
                    state.t_ocean[j] = to2;
                    state.t_land[j] = tl2;
                    state.t_mixed[j] = tm2;
                    // The slab freezes (or melts) at once, so the ocean's diffusion below never
                    // sees a slab under the freezing point: otherwise a slab cooled through thin
                    // ice drew heat from its neighbours' insulated slabs, whose ice then grew
                    // without end.
                    freeze_melt(&mut state.t_mixed[j], &mut state.ice_j_m2[j], self.c_ocean);
                }
                // 2. Diffusion of the band mean, implicit. Land and the sea's surface have the
                //    same heat capacity, so both take the band mean's change.
                // 3. Ocean heat transport: diffusion of the slab through the sea's share of each
                //    face between bands (none across land).
                // The slab's solve reads only the slab, which step 2 leaves alone: with both
                // diffusions on, the two solves run interleaved (P1-17; the same values).
                if let (Some(atm), Some(slab)) = (&atm, &slab) {
                    for j in 0..n {
                        mean[j] = f[j] * state.t_land[j] + (1.0 - f[j]) * state.t_ocean[j];
                    }
                    Tri::solve_pair(atm, &mean, &mut new, slab, &state.t_mixed, &mut new_slab);
                    for j in 0..n {
                        let dtm = new[j] - mean[j];
                        state.t_ocean[j] += dtm;
                        state.t_land[j] += dtm;
                    }
                    state.t_mixed.copy_from_slice(&new_slab);
                } else if let Some(atm) = &atm {
                    for j in 0..n {
                        mean[j] = f[j] * state.t_land[j] + (1.0 - f[j]) * state.t_ocean[j];
                    }
                    atm.solve(&mean, &mut new);
                    for j in 0..n {
                        let dtm = new[j] - mean[j];
                        state.t_ocean[j] += dtm;
                        state.t_land[j] += dtm;
                    }
                } else if let Some(slab) = &slab {
                    slab.solve(&state.t_mixed, &mut new);
                    state.t_mixed.copy_from_slice(&new);
                }
                // 4. Freezing and melting at the slab: it holds at the freezing point while ice
                //    grows, and melts its ice before it warms (the heat moves, none is lost).
                for j in 0..n {
                    freeze_melt(&mut state.t_mixed[j], &mut state.ice_j_m2[j], self.c_ocean);
                }
                for j in 0..n {
                    let (to, tl) = (state.t_ocean[j], state.t_land[j]);
                    mo[j][month] += to / per_month as f64;
                    ml[j][month] += tl / per_month as f64;
                    si[j] += (state.ice_j_m2[j] / ICE_J_M3 / FULL_ICE_M).min(1.0) / spy as f64;
                    tg += cosl[j] * (f[j] * tl + (1.0 - f[j]) * to);
                }
            }
            let steps = spy as f64 * wsum;
            out.asr = asr / steps;
            out.olr = olr / steps;
            out.t_global = tg / steps;
            out.t_ocean = mo;
            out.t_land = ml;
            out.sea_ice = si;
            out.years = year + 1;
            let annual: Vec<f64> = (0..n)
                .map(|j| {
                    (0..12)
                        .map(|m| f[j] * out.t_land[j][m] + (1.0 - f[j]) * out.t_ocean[j][m])
                        .sum::<f64>()
                        / 12.0
                })
                .collect();
            // Settled when no band's annual mean moved by 0.005 K and no band's sea ice by 2 cm
            // (a slowly melting or growing ice store can hold the temperature still).
            let ice = state.ice_j_m2.clone();
            if let Some((p, pi)) = &prev_mean
                && p.iter().zip(&annual).all(|(a, b)| (a - b).abs() < 0.005)
                && pi
                    .iter()
                    .zip(&ice)
                    .all(|(a, b)| (a - b).abs() < 0.02 * ICE_J_M3)
            {
                break;
            }
            prev_mean = Some((annual, ice));
        }
        out
    }
}

/// Freezing and melting at the slab `tm` (°C) with ice `ice` (J/m²) and heat capacity `c_ocean`
/// (J/m²/K): below sea water's freezing point the slab holds at it and the heat it lost grows
/// ice; above it, the ice melts first. The heat moves; none is lost.
pub fn freeze_melt(tm: &mut f64, ice: &mut f64, c_ocean: f64) {
    if *tm < SEA_FREEZE_C {
        *ice += (SEA_FREEZE_C - *tm) * c_ocean;
        *tm = SEA_FREEZE_C;
    } else if *ice > 0.0 {
        let melt = ((*tm - SEA_FREEZE_C) * c_ocean).min(*ice);
        *ice -= melt;
        *tm -= melt / c_ocean;
    }
}

/// A tridiagonal system lo[j]·x[j−1] + di[j]·x[j] + up[j]·x[j+1] = r[j] with its forward
/// elimination done (`ebm::thomas`'s multipliers c and pivots m, which depend on the coefficients
/// only): solving is the substitutions, each the same expression `thomas` evaluates.
pub(crate) struct Tri {
    pub(crate) lo: Vec<f64>,
    pub(crate) c: Vec<f64>,
    pub(crate) m: Vec<f64>,
}

impl Tri {
    pub(crate) fn new(lo: &[f64], di: &[f64], up: &[f64]) -> Tri {
        let n = di.len();
        let mut c = vec![0.0; n];
        let mut m = vec![0.0; n];
        c[0] = up[0] / di[0];
        m[0] = di[0];
        for j in 1..n {
            let mj = di[j] - lo[j] * c[j - 1];
            c[j] = up[j] / mj;
            m[j] = mj;
        }
        Tri {
            lo: lo.to_vec(),
            c,
            m,
        }
    }

    /// x = the solution for right-hand side `r` (`ebm::thomas(lo, di, up, r)`, bit for bit).
    pub(crate) fn solve(&self, r: &[f64], x: &mut [f64]) {
        let n = r.len();
        x[0] = r[0] / self.m[0];
        for j in 1..n {
            x[j] = (r[j] - self.lo[j] * x[j - 1]) / self.m[j];
        }
        for j in (0..n - 1).rev() {
            x[j] -= self.c[j] * x[j + 1];
        }
    }

    /// `a.solve(ra, xa)` and `b.solve(rb, xb)` in one pass over the bands (P1-17): the two
    /// substitutions are chains of divisions, and interleaved each waits on its own chain only.
    /// Every value is the one its own [`Tri::solve`] gives; the systems have the same size.
    pub(crate) fn solve_pair(
        a: &Tri,
        ra: &[f64],
        xa: &mut [f64],
        b: &Tri,
        rb: &[f64],
        xb: &mut [f64],
    ) {
        let n = ra.len();
        assert_eq!(rb.len(), n, "two systems of one size");
        xa[0] = ra[0] / a.m[0];
        xb[0] = rb[0] / b.m[0];
        for j in 1..n {
            xa[j] = (ra[j] - a.lo[j] * xa[j - 1]) / a.m[j];
            xb[j] = (rb[j] - b.lo[j] * xb[j - 1]) / b.m[j];
        }
        for j in (0..n - 1).rev() {
            xa[j] -= a.c[j] * xa[j + 1];
            xb[j] -= b.c[j] * xb[j + 1];
        }
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

    /// P1-17: the zonal model's diffusions are factored once per run; the factored solve gives
    /// `thomas`'s bits on band systems like the model's (diagonally dominant, zero end faces).
    #[test]
    fn the_kept_insolation_shapes_give_the_same_bits() {
        // A run with a cold cache (a fresh thread) against the same run after others filled it:
        // another orbit (a miss) and the same orbit under another s₀ (a hit, scaled).
        let land: Vec<f64> = (0..90).map(|j| 0.1 + 0.4 * (j as f64 / 90.0)).collect();
        let base = earth();
        let mut other = earth();
        other.orbit = insolation::Orbit {
            ecc: 0.05,
            long_peri_rad: 1.3,
        };
        let mut brighter = earth();
        brighter.s0 *= 1.0137;
        let run = |p: &EbmParams| {
            let mut st = p.initial_state();
            let r = p.run(&mut st, &land, &[], 3);
            let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
            (
                r.t_global.to_bits(),
                r.asr.to_bits(),
                bits(&st.t_ocean),
                bits(&st.t_land),
            )
        };
        let cold = |p: &EbmParams| std::thread::scope(|sc| sc.spawn(|| run(p)).join().unwrap());
        let (base0, brighter0) = (cold(&base), cold(&brighter));
        std::thread::scope(|sc| {
            sc.spawn(|| {
                run(&other);
                assert_eq!(run(&base), base0, "after another orbit");
                assert_eq!(
                    run(&brighter),
                    brighter0,
                    "the kept shapes under another s0"
                );
                assert_eq!(run(&base), base0, "again");
            })
            .join()
            .unwrap();
        });
    }

    #[test]
    fn the_factored_solve_equals_thomas_bit_for_bit() {
        let mut rng = crate::sphere::SplitMix64::new(3);
        for n in [1usize, 2, 3, 90, 180] {
            let mut lo = vec![0.0; n];
            let mut up = vec![0.0; n];
            let mut di = vec![0.0; n];
            for j in 0..n {
                let (a, b) = (50.0 * rng.next_f64(), 50.0 * rng.next_f64());
                let kl = if j > 0 { a } else { 0.0 };
                let ku = if j + 1 < n { b } else { 0.0 };
                lo[j] = -kl;
                up[j] = -ku;
                di[j] = 1.0 + kl + ku;
            }
            let tri = Tri::new(&lo, &di, &up);
            // A second system of the same size for the interleaved pair.
            let di2: Vec<f64> = di.iter().map(|d| d + 0.5 + rng.next_f64()).collect();
            let tri2 = Tri::new(&lo, &di2, &up);
            for _ in 0..5 {
                let r: Vec<f64> = (0..n).map(|_| 60.0 * rng.next_f64() - 30.0).collect();
                let mut x = vec![0.0; n];
                tri.solve(&r, &mut x);
                let want = thomas(&lo, &di, &up, &r);
                let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
                assert_eq!(bits(&x), bits(&want), "n {n}");
                // P1-17: the interleaved pair gives each system's own solve, bit for bit.
                let r2: Vec<f64> = (0..n).map(|_| 60.0 * rng.next_f64() - 30.0).collect();
                let mut x2 = vec![0.0; n];
                tri2.solve(&r2, &mut x2);
                let (mut pa, mut pb) = (vec![0.0; n], vec![0.0; n]);
                Tri::solve_pair(&tri, &r, &mut pa, &tri2, &r2, &mut pb);
                assert_eq!(bits(&pa), bits(&x), "pair, first, n {n}");
                assert_eq!(bits(&pb), bits(&x2), "pair, second, n {n}");
            }
        }
    }

    #[test]
    fn the_ebm_converges_to_energy_balance() {
        let p = earth();
        let n = p.bands;
        // Earth-like land: 30 % everywhere, more in the north, Antarctica at the south pole.
        let land: Vec<f64> = (0..n)
            .map(|j| {
                let lat = -90.0 + (j as f64 + 0.5) * 2.0;
                if lat < -65.0 {
                    0.95
                } else if lat > 30.0 {
                    0.5
                } else {
                    0.25
                }
            })
            .collect();
        let sheet: Vec<f64> = (0..n)
            .map(|j| {
                if (j as f64 + 0.5) * 2.0 - 90.0 < -65.0 {
                    1.0
                } else {
                    0.0
                }
            })
            .collect();
        let mut st = p.initial_state();
        let r = p.run(&mut st, &land, &sheet, 200);
        assert!(r.years < 200, "converged ({} years)", r.years);
        assert!(r.net_toa().abs() < 0.2, "ASR − OLR = {} W/m²", r.net_toa());
        assert!(
            (5.0..=20.0).contains(&r.t_global),
            "global mean {} °C",
            r.t_global
        );
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
        assert!(
            r.t_land[mid][6] > r.t_land[mid][0] + 15.0,
            "July warmer than January"
        );
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

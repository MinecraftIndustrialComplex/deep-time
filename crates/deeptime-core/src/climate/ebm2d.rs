// SPDX-License-Identifier: LGPL-3.0-or-later
//! The seasonal energy balance per coarse cell (P1-26 T2; `docs/design/p1-26-per-cell-climate.md`
//! §2): the zonal model of `ebm.rs` on the climate's coarse grid (`coarse.rs`, 128 × 64 cells
//! by default), with its heat moved between cells as well as between latitudes.
//!
//! Every cell holds what a band of `ebm.rs` holds — a land tile T_l and a sea tile T_o (each with
//! the air above it, heat capacity C_l), a slab mixed layer T_m (C_m) and sea ice stored as latent
//! heat E_i — with the same physics and constants:
//!
//! ```text
//! land:  C_l ∂T_l/∂t = (1 − α_l) Q − OLR(T_l) − x (1 − f)(T_l − T_o)                + 𝒜
//! sea:   C_l ∂T_o/∂t = (1 − α_o) Q − OLR(T_o) − x f (T_o − T_l) − g(h)(T_o − T_m)   + 𝒜
//! slab:  C_m ∂T_m/∂t = g(h)(T_o − T_m) + 𝒪 − (freezing and melting at −1.8 °C, into E_i)
//! ```
//!
//! - **Albedo per cell** (§2.4): a₀ + a₂P₂(sin φ), plus `desert_albedo` × the cell's desert share
//!   of its land (the previous snapshot's Köppen B land: a lagged, Charney-type feedback), going
//!   to the ice albedo by the larger of the snow factor of the tile's temperature and the cell's
//!   ice-sheet share (land) or by the sea-ice cover (sea).
//! - **𝒜, the atmosphere's heat transport** (§2.5), acts on the cell mean T̄ = f T_l + (1 − f) T_o
//!   and both tiles take its change: diffusion in flux form on the sphere, D on a face being
//!   (1 − f̄) D_sea(φ) + f̄ D_land(φ), each falling from its equatorial to its polar value as cos⁵ φ
//!   (Ziegler & Rehfeld 2021 eq. 2 and Table 1, the NMS83 lineage's surface-dependent D), times
//!   `atmosphere_diffusivity_scale`, the east–west faces further × `zonal_diffusivity_ratio`;
//!   then the prevailing low-level wind's zonal advection, an exact shift of each row by
//!   β_T u_a Δt/Δx cells (integer part whole cells, the fraction donor-cell), u_a being the
//!   design's 850-hPa profile in belt latitude.
//! - **𝒪, the ocean's** (§2.6): the slab's eddy diffusion through the sea's share of each face
//!   (none across land), and, when given, the wind-driven gyres and the overturning conveyor of
//!   `ocean2d.rs`.
//! - **Sea ice per cell** (§2.7): exactly `ebm.rs`'s thermodynamics, each cell from its own heat
//!   budget.
//!
//! **Numerics** (§2.8). One step: (1) the local terms implicit, a 3 × 3 solve per cell; (2) the
//! atmosphere's diffusion of T̄ implicit by ADI (Lie splitting), one cyclic tridiagonal per row
//! (Sherman–Morrison) and one tridiagonal per column; (3) the zonal shift; (4) the slab's
//! diffusion (ADI) and the ocean's transport; (5) freezing and melting. Conductances are
//! symmetric and every flux is in flux form, so Σ A·(heat) is conserved exactly; the implicit
//! solves need no time-step limit at the 8-km polar rows. Lines and cells are independent, so
//! the result does not depend on the thread count.
//!
//! On a zonally uniform planet (one land fraction per row, uniform D, no wind) every row is
//! uniform and the model is `ebm.rs` band by band (a test).

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::belts;
use super::coarse::Grid;
use super::ebm::{
    EbmParams, EbmState, FULL_ICE_M, ICE_J_M3, ICE_W_M_K, OPEN_WATER_W_M2_K, SEA_ICE_CAP_M, Tri,
    freeze_melt, smoothstep,
};
use super::insolation;

const YEAR_S: f64 = 365.25 * 86_400.0;

/// The surface-dependent diffusivity of the NMS83 lineage (Zhuang, North & Stevens 2017's column,
/// as tabulated by Ziegler & Rehfeld 2021, Table 1): equatorial and polar values, W/m²/K. These
/// are the sources' total (atmosphere + ocean) transport; `atmosphere_diffusivity_scale`
/// calibrates them down to the atmosphere's share (design §2.5, T9).
pub const D_SEA_EQ: f64 = 0.65; // ledger: CLM-120
pub const D_SEA_POLE: f64 = 0.40; // ledger: CLM-120
pub const D_LAND_EQ: f64 = 0.65; // ledger: CLM-120
pub const D_LAND_NORTH_POLE: f64 = 0.28; // ledger: CLM-120
pub const D_LAND_SOUTH_POLE: f64 = 0.20; // ledger: CLM-120

/// The low-level (about 850 hPa) zonal wind that carries heat along a row, m/s, at belt latitude
/// φ′ (degrees): trades to about 25°, westerlies peaking at 45° and still blowing at 65°, weak
/// polar easterlies (design §2.5; compared with NCEP/NCAR's 850-hPa zonal means in WP-14a).
pub fn low_level_wind(phi_b: f64) -> f64 {
    let a = phi_b.abs();
    // ledger: CLM-124
    -6.0 * belts::bump(a, 12.0, 18.0) + 9.0 * belts::bump(a, 45.0, 28.0)
        - 2.0 * belts::bump(a, 82.0, 10.0)
}

/// The per-cell model's parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct Params2d {
    /// Radiation, albedo, heat capacities, exchanges, steps, orbit (`ebm.rs`'s; `d` and `bands`
    /// are not used).
    pub ebm: EbmParams,
    /// Atmospheric diffusivity: sea and land at the equator and the poles, W/m²/K (already × the
    /// scale), and the east–west faces' factor.
    pub d_sea_eq: f64,
    pub d_sea_pole: f64,
    pub d_land_eq: f64,
    pub d_land_pole_n: f64,
    pub d_land_pole_s: f64,
    pub zonal_ratio: f64,
    /// β_T: the share of the low-level wind's speed at which the boundary layer's heat moves.
    pub wind_heat: f64,
    /// Planetary albedo added over desert, × the cell's desert share of its land.
    pub desert_albedo: f64,
    /// The seasonal shift of the subtropical belts in each month, degrees (`belts::
    /// hemispheric_shift`), and the ITCZ's largest shift.
    pub hemi_shift: [f64; 12],
    pub itcz_max_deg: f64,
    /// The planet's radius as the climate sees it (Earth-normalized), m.
    pub radius_m: f64,
}

impl Params2d {
    /// The per-cell model of climate section `c` on a planet whose zonal model is `ebm`.
    pub fn from_climate(c: &super::ClimateParams, ebm: EbmParams, tilt_deg: f64) -> Params2d {
        let s = c.atmosphere_diffusivity_scale;
        Params2d {
            ebm,
            d_sea_eq: D_SEA_EQ * s,
            d_sea_pole: D_SEA_POLE * s,
            d_land_eq: D_LAND_EQ * s,
            d_land_pole_n: D_LAND_NORTH_POLE * s,
            d_land_pole_s: D_LAND_SOUTH_POLE * s,
            zonal_ratio: c.zonal_diffusivity_ratio,
            wind_heat: c.wind_heat_transport,
            desert_albedo: c.desert_albedo,
            hemi_shift: std::array::from_fn(|m| belts::hemispheric_shift(c, tilt_deg, m)),
            itcz_max_deg: c.itcz_max_shift_deg,
            radius_m: EARTH_RADIUS_M,
        }
    }

    /// D of a face at latitude `lat` (degrees) whose two cells hold `f` land on average.
    pub fn d_face(&self, lat: f64, f: f64) -> f64 {
        let c5 = lat.to_radians().cos().max(0.0).powi(5);
        let sea = self.d_sea_pole + (self.d_sea_eq - self.d_sea_pole) * c5;
        let pole = if lat >= 0.0 {
            self.d_land_pole_n
        } else {
            self.d_land_pole_s
        };
        let land = pole + (self.d_land_eq - pole) * c5;
        (1.0 - f) * sea + f * land
    }
}

/// The climate's planet radius, m: the grid is Earth-normalized like the rest of the planetary
/// climate (params.rs, "Two length scales").
pub const EARTH_RADIUS_M: f64 = 6.371e6;

/// What the surface is in every coarse cell (rows north first).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Surface {
    /// Land fraction f.
    pub land: Vec<f64>,
    /// Share of the land under ice sheets and glaciers (× `ice_sheet_feedback`).
    pub sheet: Vec<f64>,
    /// Share of the land that is desert (Köppen B at the previous snapshot).
    pub desert: Vec<f64>,
}

/// The model's state at the start of a year, per coarse cell.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct State2d {
    pub t_land: Vec<f64>,
    pub t_ocean: Vec<f64>,
    pub t_mixed: Vec<f64>,
    pub ice_j_m2: Vec<f64>,
}

impl State2d {
    /// Every cell from the zonal model's band state at its latitude (the first snapshot's start:
    /// the 1-D model is already balanced by the carbon search). `bands` south to north.
    pub fn from_bands(g: &Grid, z: &EbmState) -> State2d {
        let n = z.t_ocean.len();
        let at = |v: &[f64], lat: f64| -> f64 {
            if n == 0 {
                return 28.0 - 42.0 * lat.to_radians().sin().powi(2);
            }
            let x = (lat + 90.0) / 180.0 * n as f64 - 0.5;
            let b0 = x.floor().clamp(0.0, (n - 1) as f64) as usize;
            let b1 = (b0 + 1).min(n - 1);
            let f = (x - b0 as f64).clamp(0.0, 1.0);
            v[b0] + f * (v[b1] - v[b0])
        };
        let mut s = State2d::default();
        for j in 0..g.h {
            let lat = g.lat_c(j);
            let (tl, to, tm, ice) = if n == 0 {
                let t = at(&[], lat);
                (t, t, t, 0.0)
            } else {
                (
                    at(&z.t_land, lat),
                    at(&z.t_ocean, lat),
                    at(&z.t_mixed, lat),
                    at(&z.ice_j_m2, lat),
                )
            };
            for _ in 0..g.w {
                s.t_land.push(tl);
                s.t_ocean.push(to);
                s.t_mixed.push(tm);
                s.ice_j_m2.push(ice);
            }
        }
        s
    }

    fn fits(&self, n: usize) -> bool {
        self.t_land.len() == n
            && self.t_ocean.len() == n
            && self.t_mixed.len() == n
            && self.ice_j_m2.len() == n
    }
}

/// The ocean's own heat transport (gyres and conveyor, `ocean2d.rs`), applied to the slab every
/// step after its diffusion.
pub trait OceanTransport: Sync {
    /// Moves heat in the slab temperatures `tm` (°C, per cell) over `dt` seconds, conservatively;
    /// adds the northward heat it moved across every interior row edge (edge e between rows e
    /// and e + 1), W, × `dt`, to `north_j`.
    fn step(&self, tm: &mut [f64], dt: f64, north_j: &mut [f64]);
}

/// Monthly means of the last year run, per cell, and the year's global budget.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Result2d {
    pub t_land: Vec<[f64; 12]>,
    pub t_ocean: Vec<[f64; 12]>,
    /// The slab's temperature, °C.
    pub t_mixed: Vec<[f64; 12]>,
    /// Sea-ice cover of the cell's sea, 0–1.
    pub sea_ice: Vec<[f64; 12]>,
    /// Area-weighted annual means, W/m² and °C.
    pub asr: f64,
    pub olr: f64,
    pub t_global: f64,
    pub years: u32,
    /// Annual-mean northward heat transport across every interior row edge (edge e between rows
    /// e and e + 1), PW: the atmosphere's diffusion, the slab's diffusion, the ocean's transport.
    pub heat_atm_pw: Vec<f64>,
    pub heat_ocean_diff_pw: Vec<f64>,
    pub heat_ocean_pw: Vec<f64>,
    /// The ITCZ's zonal-mean latitude in each month (the thermal equator of T̄), degrees.
    pub itcz: [f64; 12],
}

impl Result2d {
    /// ASR − OLR, W/m².
    pub fn net_toa(&self) -> f64 {
        self.asr - self.olr
    }
}

/// Solves a cyclic tridiagonal system lo[i]·x[i−1] + di[i]·x[i] + up[i]·x[i+1] = r[i], indices
/// mod n (Sherman–Morrison on the Thomas algorithm). n ≥ 3.
pub fn cyclic_thomas(lo: &[f64], di: &[f64], up: &[f64], r: &[f64]) -> Vec<f64> {
    let n = di.len();
    if n < 3 {
        // Two or one cells: solve directly (the wrap faces coincide).
        if n == 1 {
            return vec![r[0] / (di[0] + lo[0] + up[0])];
        }
        let (a, b, c, d) = (di[0], lo[0] + up[0], lo[1] + up[1], di[1]);
        let det = a * d - b * c;
        return vec![(r[0] * d - b * r[1]) / det, (a * r[1] - c * r[0]) / det];
    }
    let alpha = up[n - 1]; // x[0]'s coefficient in the last row
    let beta = lo[0]; // x[n − 1]'s coefficient in the first row
    let gamma = -di[0];
    let mut bb = di.to_vec();
    bb[0] = di[0] - gamma;
    bb[n - 1] = di[n - 1] - alpha * beta / gamma;
    let x = super::ebm::thomas(lo, &bb, up, r);
    let mut u = vec![0.0; n];
    u[0] = gamma;
    u[n - 1] = alpha;
    let z = super::ebm::thomas(lo, &bb, up, &u);
    let fact = (x[0] + beta * x[n - 1] / gamma) / (1.0 + z[0] + beta * z[n - 1] / gamma);
    x.iter().zip(&z).map(|(a, b)| a - fact * b).collect()
}

/// One row's zonal shift by `s` cells (east positive): the integer part moves whole cells, the
/// fraction is donor-cell (upwind). Conserves the row's sum and keeps a uniform row uniform.
pub fn shift_row(v: &mut [f64], s: f64) {
    shift_row_with(v, s, &mut Vec::new());
}

/// [`shift_row`] with a reusable buffer for the row's old values.
fn shift_row_with(v: &mut [f64], s: f64, src: &mut Vec<f64>) {
    let n = v.len();
    if n == 0 || s == 0.0 || !s.is_finite() {
        return;
    }
    let whole = s.floor();
    let frac = s - whole; // in [0, 1): a further shift east by frac
    let k = (whole as i64).rem_euclid(n as i64) as usize;
    src.clear();
    src.extend_from_slice(v);
    for i in 0..n {
        // East shift by k + frac: cell i receives (1 − frac) of cell i − k and frac of i − k − 1.
        let a = src[(i + n - k) % n];
        let b = src[(i + 2 * n - k - 1) % n];
        v[i] = (1.0 - frac) * a + frac * b;
    }
}

/// The thermal equator of a month's zonal-mean T̄ by row: the soft maximum over |φ| ≤ 35°
/// (`belts::itcz`'s rule on the zonal mean), clamped to ±`max_deg`.
fn thermal_equator(g: &Grid, row_t: &[f64], max_deg: f64) -> f64 {
    let rows: Vec<usize> = (0..g.h).filter(|&j| g.lat_c(j).abs() <= 35.0).collect();
    if rows.is_empty() {
        return 0.0;
    }
    let tmax = rows.iter().map(|&j| row_t[j]).fold(f64::MIN, f64::max);
    let (mut num, mut den) = (0.0, 0.0);
    for &j in &rows {
        let w = ((row_t[j] - tmax) / 1.5).exp() * g.sin_span(j);
        num += w * g.lat_c(j);
        den += w;
    }
    (num / den.max(1e-300)).clamp(-max_deg, max_deg)
}

/// Runs up to `years` years from `state` (left at the start of the next year) on surface `s`,
/// with the ocean's transport `ocean` if given; stops early once no cell's annual mean T̄ moves
/// by 0.005 K and no ice store by 2 cm from one year to the next. `itcz0` is the ITCZ to start
/// from (the previous run's, or zeros).
///
/// The implicit solves' coefficients do not change during a run, so each row's and column's
/// system is factored once ([`Adi`]) and a step only substitutes; the steps reuse their buffers.
/// Every value is the same expression on the same inputs as solving afresh each step (P1-17; the
/// tests keep the old loop and compare bit for bit).
pub fn run(
    p: &Params2d,
    g: &Grid,
    s: &Surface,
    state: &mut State2d,
    ocean: Option<&dyn OceanTransport>,
    years: u32,
    itcz0: [f64; 12],
) -> Result2d {
    let ep = &p.ebm;
    let (w, h) = (g.w, g.h);
    let n = w * h;
    if !state.fits(n) {
        *state = State2d::from_bands(g, &EbmState::default());
    }
    let spy = ep.steps_per_year.max(12);
    let per_month = spy / 12;
    let dt = YEAR_S / spy as f64;
    let lat: Vec<f64> = (0..h).map(|j| g.lat_c(j)).collect();
    let area: Vec<f64> = g.areas();
    let area_tot: f64 = area.iter().sum();
    let r2 = p.radius_m * p.radius_m;
    let f: Vec<f64> = (0..n)
        .map(|c| s.land.get(c).copied().unwrap_or(0.0).clamp(0.0, 1.0))
        .collect();
    let sheet: Vec<f64> = (0..n)
        .map(|c| s.sheet.get(c).copied().unwrap_or(0.0).clamp(0.0, 1.0))
        .collect();
    let desert: Vec<f64> = (0..n)
        .map(|c| s.desert.get(c).copied().unwrap_or(0.0).clamp(0.0, 1.0))
        .collect();
    let p2 = |l: f64| {
        let x = l.to_radians().sin();
        0.5 * (3.0 * x * x - 1.0)
    };
    let free_sea: Vec<f64> = lat.iter().map(|&l| ep.a0 + ep.a2 * p2(l)).collect();
    let free_land: Vec<f64> = (0..n)
        .map(|c| free_sea[c / w] + p.desert_albedo * desert[c])
        .collect();
    let frozen = |t: f64| 1.0 - smoothstep(ep.ice_temp_c - 3.0, ep.ice_temp_c + 3.0, t);
    // Insolation of every step and row (the step's midpoint).
    let q: Vec<Vec<f64>> = (0..spy)
        .map(|st| {
            let lam = ep.orbit.lambda_at((st as f64 + 0.5) / spy as f64);
            lat.iter()
                .map(|&l| {
                    insolation::daily_orbit(ep.s0, l.to_radians(), lam, ep.obliquity_rad, &ep.orbit)
                })
                .collect()
        })
        .collect();
    // Conductances (W/K per unit sphere area per R², i.e. D·G) of every face, the atmosphere's
    // and the slab's. East–west face (i, j)–(i + 1, j); north–south face (i, j)–(i, j + 1).
    let k_ew: Vec<f64> = (0..n)
        .map(|c| {
            let (i, j) = (c % w, c / w);
            let c1 = j * w + (i + 1) % w;
            p.zonal_ratio * p.d_face(lat[j], 0.5 * (f[c] + f[c1])) * g.g_ew(i, j)
        })
        .collect();
    let k_ns: Vec<f64> = (0..n)
        .map(|c| {
            let (i, j) = (c % w, c / w);
            if j + 1 >= h {
                return 0.0;
            }
            let c1 = c + w;
            p.d_face(g.lat_edge[j + 1], 0.5 * (f[c] + f[c1])) * g.g_ns(i, j)
        })
        .collect();
    let sea_min = |a: usize, b: usize| (1.0 - f[a]).min(1.0 - f[b]);
    let ko_ew: Vec<f64> = (0..n)
        .map(|c| {
            let (i, j) = (c % w, c / w);
            ep.d_ocean * sea_min(c, j * w + (i + 1) % w) * g.g_ew(i, j)
        })
        .collect();
    let ko_ns: Vec<f64> = (0..n)
        .map(|c| {
            let (i, j) = (c % w, c / w);
            if j + 1 >= h {
                0.0
            } else {
                ep.d_ocean * sea_min(c, c + w) * g.g_ns(i, j)
            }
        })
        .collect();
    // Capacities per cell, J/K per unit area (× A): the atmosphere's mean T̄ has C_l; the slab's
    // sea share C_m (1 − f).
    let cap_atm: Vec<f64> = area.iter().map(|a| ep.c_land * a).collect();
    let cap_sea: Vec<f64> = (0..n)
        .map(|c| ep.c_ocean * area[c] * (1.0 - f[c]).max(1e-6))
        .collect();
    // The zonal shift's cell width of every row, m.
    let dx_row: Vec<f64> = (0..h)
        .map(|j| p.radius_m * g.cos_mean(j).max(1e-6) * g.lon_w[0])
        .collect();
    // The implicit diffusions, factored once for the run.
    let atm = Adi::new(w, h, &k_ew, &k_ns, &cap_atm, dt);
    let slab = (ep.d_ocean > 0.0).then(|| Adi::new(w, h, &ko_ew, &ko_ns, &cap_sea, dt));

    let mut out = Result2d {
        heat_atm_pw: vec![0.0; h.saturating_sub(1)],
        heat_ocean_diff_pw: vec![0.0; h.saturating_sub(1)],
        heat_ocean_pw: vec![0.0; h.saturating_sub(1)],
        itcz: itcz0,
        ..Result2d::default()
    };
    let mut itcz = itcz0;
    let mut prev: Option<(Vec<f64>, Vec<f64>)> = None;
    // The steps' buffers: each cell's absorbed and emitted radiation, T̄ before and after the
    // atmosphere's transport, and the solves' intermediate.
    let mut a_sr = vec![0.0; n];
    let mut a_lr = vec![0.0; n];
    let mut mean = vec![0.0; n];
    let mut tb = vec![0.0; n];
    let mut half = vec![0.0; n];
    // Each cell's T̄ at the step's end, for the global mean.
    let mut tbar = vec![0.0; n];
    for year in 0..years.max(1) {
        let mut ml = vec![[0.0; 12]; n];
        let mut mo = vec![[0.0; 12]; n];
        let mut mm = vec![[0.0; 12]; n];
        let mut mi = vec![[0.0; 12]; n];
        let mut row_t = vec![[0.0f64; 12]; h];
        let (mut asr, mut olr, mut tg) = (0.0, 0.0, 0.0);
        let mut north_atm = vec![0.0; h.saturating_sub(1)];
        let mut north_od = vec![0.0; h.saturating_sub(1)];
        let mut north_o = vec![0.0; h.saturating_sub(1)];
        for (st, qs) in q.iter().enumerate() {
            let month = st / per_month;
            // 1. Local terms, implicit (`ebm.rs`'s 3 × 3 solve), cells in parallel, each writing
            //    its own tiles, its budget and its new T̄ (`mean`, the atmosphere's diffusion's
            //    input).
            state
                .t_land
                .par_iter_mut()
                .zip(state.t_ocean.par_iter_mut())
                .zip(state.t_mixed.par_iter_mut())
                .zip(state.ice_j_m2.par_iter_mut())
                .zip(a_sr.par_iter_mut().zip(a_lr.par_iter_mut()))
                .zip(mean.par_iter_mut())
                .enumerate()
                .for_each(
                    |(c, (((((tl_c, to_c), tm_c), ice_c), (sr_c, lr_c)), mean_c))| {
                        let j = c / w;
                        let (to, tl, tm) = (*to_c, *tl_c, *tm_c);
                        let mut ice = *ice_c;
                        let h_ice = ice / ICE_J_M3;
                        let cover = (h_ice / FULL_ICE_M).min(1.0);
                        let insulated = smoothstep(0.5 * SEA_ICE_CAP_M, SEA_ICE_CAP_M, h_ice);
                        let gc = OPEN_WATER_W_M2_K * (1.0 - cover)
                            + (ICE_W_M_K / h_ice.max(0.05)).min(OPEN_WATER_W_M2_K)
                                * cover
                                * (1.0 - insulated);
                        let ao = free_sea[j] + (ep.ai - free_sea[j]) * frozen(to).max(cover);
                        let al = free_land[c] + (ep.ai - free_land[c]) * frozen(tl).max(sheet[c]);
                        let so = qs[j] * (1.0 - ao);
                        let sl = qs[j] * (1.0 - al);
                        let (cl, cs, cm) = (ep.c_land / dt, ep.c_land / dt, ep.c_ocean / dt);
                        let (xl, xo) = (ep.exchange * (1.0 - f[c]), ep.exchange * f[c]);
                        let (a_l, b_l) = super::co2::longwave(
                            ep.a,
                            ep.b,
                            ep.co2.as_ref(),
                            tl,
                            ep.snow_cloud,
                            frozen(tl).max(sheet[c]),
                        );
                        let (a_o, b_o) = super::co2::longwave(
                            ep.a,
                            ep.b,
                            ep.co2.as_ref(),
                            to,
                            ep.snow_cloud,
                            cover,
                        );
                        let (r1, r2, r3) = (cl * tl + sl - a_l, cs * to + so - a_o, cm * tm);
                        let a33 = cm + gc;
                        let a11 = cl + b_l + xl;
                        let a22 = cs + b_o + xo + gc - gc * gc / a33;
                        let r2 = r2 + gc * r3 / a33;
                        let det = a11 * a22 - xl * xo;
                        let tl2 = (r1 * a22 + xl * r2) / det;
                        let mut to2 = (a11 * r2 + xo * r1) / det;
                        let mut tm2 = (r3 + gc * to2) / a33;
                        *sr_c = f[c] * sl + (1.0 - f[c]) * so;
                        *lr_c = f[c] * (a_l + b_l * tl2) + (1.0 - f[c]) * (a_o + b_o * to2);
                        if ice > 0.0 && to2 > 0.0 {
                            let melt = (to2 * ep.c_land).min(ice);
                            ice -= melt;
                            to2 -= melt / ep.c_land;
                        }
                        // At once, as `ebm.rs` does: the slab's diffusion then never sees a slab
                        // below the freezing point.
                        freeze_melt(&mut tm2, &mut ice, ep.c_ocean);
                        (*tl_c, *to_c, *tm_c, *ice_c) = (tl2, to2, tm2, ice);
                        *mean_c = f[c] * tl2 + (1.0 - f[c]) * to2;
                    },
                );
            // 2. The atmosphere's diffusion of T̄, implicit ADI: rows (cyclic), then columns.
            atm.solve(&mean, &mut half, &mut tb);
            northward(w, &k_ns, &tb, r2 * dt, &mut north_atm);
            // 3. The prevailing wind's zonal shift of T̄, then both tiles take T̄'s change; rows
            //    in parallel.
            let rows = tb
                .par_chunks_mut(w)
                .zip(mean.par_chunks(w))
                .zip(state.t_land.par_chunks_mut(w))
                .zip(state.t_ocean.par_chunks_mut(w))
                .enumerate();
            if p.wind_heat > 0.0 {
                let hemi = p.hemi_shift[month];
                rows.for_each_init(Vec::new, |src, (j, (((row, m), tl), to))| {
                    let phi = lat[j];
                    let pb = phi - belts::shift(phi, itcz[month], hemi);
                    let u = p.wind_heat * low_level_wind(pb);
                    shift_row_with(row, u * dt / dx_row[j], src);
                    add_change(row, m, tl, to);
                });
            } else {
                rows.for_each(|(_, (((row, m), tl), to))| add_change(row, m, tl, to));
            }
            // 4. The ocean: the slab's diffusion through the sea, then its transport.
            if let Some(slab) = &slab {
                // `mean` is free again: it takes the slab's new temperatures.
                slab.solve(&state.t_mixed, &mut half, &mut mean);
                northward(w, &ko_ns, &mean, r2 * dt, &mut north_od);
                std::mem::swap(&mut state.t_mixed, &mut mean);
            }
            if let Some(o) = ocean {
                o.step(&mut state.t_mixed, dt, &mut north_o);
            }
            // 5. Freezing and melting at the slab, then the month's means (each cell's own),
            //    rows in parallel (a row's zonal sum in cell order, as one loop over the cells
            //    made it); then the step's global sums, in cell order.
            let pm = per_month as f64;
            state
                .t_mixed
                .par_chunks_mut(w)
                .zip(state.ice_j_m2.par_chunks_mut(w))
                .zip(state.t_land.par_chunks(w).zip(state.t_ocean.par_chunks(w)))
                .zip(ml.par_chunks_mut(w).zip(mo.par_chunks_mut(w)))
                .zip(mm.par_chunks_mut(w).zip(mi.par_chunks_mut(w)))
                .zip(tbar.par_chunks_mut(w).zip(row_t.par_iter_mut()))
                .enumerate()
                .for_each(
                    |(j, (((((tm, ice), (tl, to)), (ml, mo)), (mm, mi)), (tbar, rt)))| {
                        for i in 0..w {
                            let c = j * w + i;
                            freeze_melt(&mut tm[i], &mut ice[i], ep.c_ocean);
                            ml[i][month] += tl[i] / pm;
                            mo[i][month] += to[i] / pm;
                            mm[i][month] += tm[i] / pm;
                            mi[i][month] += (ice[i] / ICE_J_M3 / FULL_ICE_M).min(1.0) / pm;
                            let t = f[c] * tl[i] + (1.0 - f[c]) * to[i];
                            tbar[i] = t;
                            rt[month] += t / pm;
                        }
                    },
                );
            for c in 0..n {
                asr += area[c] * a_sr[c];
                olr += area[c] * a_lr[c];
                tg += area[c] * tbar[c];
            }
        }
        let steps = spy as f64 * area_tot;
        out.asr = asr / steps;
        out.olr = olr / steps;
        out.t_global = tg / steps;
        out.t_land = ml;
        out.t_ocean = mo;
        out.t_mixed = mm;
        out.sea_ice = mi;
        out.years = year + 1;
        let pw = 1.0e15 * YEAR_S;
        out.heat_atm_pw = north_atm.iter().map(|x| x / pw).collect();
        out.heat_ocean_diff_pw = north_od.iter().map(|x| x / pw).collect();
        out.heat_ocean_pw = north_o.iter().map(|x| x / pw).collect();
        // The next year's ITCZ: the thermal equator of each month's zonal-mean T̄.
        for (m, it) in itcz.iter_mut().enumerate() {
            let rt: Vec<f64> = (0..h).map(|j| row_t[j][m] / w as f64).collect();
            *it = thermal_equator(g, &rt, p.itcz_max_deg);
        }
        out.itcz = itcz;
        let annual: Vec<f64> = (0..n)
            .map(|c| {
                (0..12)
                    .map(|m| f[c] * out.t_land[c][m] + (1.0 - f[c]) * out.t_ocean[c][m])
                    .sum::<f64>()
                    / 12.0
            })
            .collect();
        let ice = state.ice_j_m2.clone();
        if let Some((pa, pi)) = &prev
            && pa.iter().zip(&annual).all(|(a, b)| (a - b).abs() < 0.005)
            && pi
                .iter()
                .zip(&ice)
                .all(|(a, b)| (a - b).abs() < 0.02 * ICE_J_M3)
        {
            break;
        }
        prev = Some((annual, ice));
    }
    out
}

/// One row's tiles take the change of T̄ the atmosphere's transport made (`tb`, after; `mean`,
/// before).
fn add_change(tb: &[f64], mean: &[f64], tl: &mut [f64], to: &mut [f64]) {
    for (((t, m), l), o) in tb.iter().zip(mean).zip(tl.iter_mut()).zip(to.iter_mut()) {
        let d = t - m;
        *l += d;
        *o += d;
    }
}

/// Adds the northward heat a diffusion moved across every interior row edge (edge e between rows
/// e and e + 1), its south faces' conductances `k` over the new field `t`, × `scale`, to `acc`.
fn northward(w: usize, k: &[f64], t: &[f64], scale: f64, acc: &mut [f64]) {
    for (e, a) in acc.iter_mut().enumerate() {
        let s: f64 = (0..w)
            .map(|i| {
                let c = e * w + i;
                k[c] * (t[c + w] - t[c])
            })
            .sum();
        *a += s * scale;
    }
}

/// A row's implicit diffusion along x (it wraps): nothing to do where the row has no conductance,
/// else [`cyclic_thomas`] with the parts that do not depend on the right-hand side done once.
enum RowSystem {
    Still,
    /// Sherman–Morrison on a factored tridiagonal: x = y − fact·z with y the tridiagonal's
    /// solution for r, z its (fixed) solution for the correction vector, fact = (y₀ + β y_{n−1}/γ)
    /// / denom.
    Cyclic {
        tri: Tri,
        z: Vec<f64>,
        beta: f64,
        gamma: f64,
        denom: f64,
    },
    /// Fewer than three cells: solved directly each time.
    Small {
        lo: Vec<f64>,
        di: Vec<f64>,
        up: Vec<f64>,
    },
}

impl RowSystem {
    fn new(lo: Vec<f64>, di: Vec<f64>, up: Vec<f64>) -> RowSystem {
        if lo.iter().all(|&x| x == 0.0) {
            return RowSystem::Still;
        }
        let n = di.len();
        if n < 3 {
            return RowSystem::Small { lo, di, up };
        }
        // `cyclic_thomas`'s set-up, verbatim.
        let alpha = up[n - 1];
        let beta = lo[0];
        let gamma = -di[0];
        let mut bb = di.clone();
        bb[0] = di[0] - gamma;
        bb[n - 1] = di[n - 1] - alpha * beta / gamma;
        let tri = Tri::new(&lo, &bb, &up);
        let mut u = vec![0.0; n];
        u[0] = gamma;
        u[n - 1] = alpha;
        let mut z = vec![0.0; n];
        tri.solve(&u, &mut z);
        let denom = 1.0 + z[0] + beta * z[n - 1] / gamma;
        RowSystem::Cyclic {
            tri,
            z,
            beta,
            gamma,
            denom,
        }
    }

    /// `out` = the row's new values from `v`.
    fn solve(&self, v: &[f64], out: &mut [f64]) {
        match self {
            RowSystem::Still => out.copy_from_slice(v),
            RowSystem::Cyclic {
                tri,
                z,
                beta,
                gamma,
                denom,
            } => {
                let n = v.len();
                tri.solve(v, out);
                let fact = (out[0] + beta * out[n - 1] / gamma) / denom;
                for (o, b) in out.iter_mut().zip(z) {
                    *o -= fact * b;
                }
            }
            RowSystem::Small { lo, di, up } => out.copy_from_slice(&cyclic_thomas(lo, di, up, v)),
        }
    }
}

/// Implicit diffusion by ADI with a run's fixed conductances and capacities: C A (x − v)/Δt =
/// Σ K (x_nb − x) along every row (x wraps; `k_ew[c]` the conductance of cell c's east face),
/// then along every column (rows stop at the poles; `k_ns[c]` that of its south face). Each
/// row's and column's system is factored once.
struct Adi {
    w: usize,
    h: usize,
    rows: Vec<RowSystem>,
    /// The columns' systems, row-major (element j·w + i is column i's row j), so the column
    /// substitutions run along the rows.
    col_lo: Vec<f64>,
    col_c: Vec<f64>,
    col_m: Vec<f64>,
}

impl Adi {
    fn new(w: usize, h: usize, k_ew: &[f64], k_ns: &[f64], cap: &[f64], dt: f64) -> Adi {
        let rows = (0..h)
            .map(|j| {
                let r0 = j * w;
                let mut lo = vec![0.0; w];
                let mut di = vec![0.0; w];
                let mut up = vec![0.0; w];
                for i in 0..w {
                    let c = r0 + i;
                    let ke = k_ew[c] * dt / cap[c];
                    let kw = k_ew[r0 + (i + w - 1) % w] * dt / cap[c];
                    lo[i] = -kw;
                    up[i] = -ke;
                    di[i] = 1.0 + ke + kw;
                }
                RowSystem::new(lo, di, up)
            })
            .collect();
        let n = w * h;
        let (mut col_lo, mut col_c, mut col_m) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
        let (mut lo, mut di, mut up) = (vec![0.0; h], vec![0.0; h], vec![0.0; h]);
        for i in 0..w {
            for j in 0..h {
                let c = j * w + i;
                let ks = k_ns[c] * dt / cap[c];
                let kn = if j > 0 {
                    k_ns[c - w] * dt / cap[c]
                } else {
                    0.0
                };
                lo[j] = -kn;
                up[j] = -ks;
                di[j] = 1.0 + ks + kn;
            }
            let t = Tri::new(&lo, &di, &up);
            for j in 0..h {
                let c = j * w + i;
                col_lo[c] = t.lo[j];
                col_c[c] = t.c[j];
                col_m[c] = t.m[j];
            }
        }
        Adi {
            w,
            h,
            rows,
            col_lo,
            col_c,
            col_m,
        }
    }

    /// `out` = v after the row sweep and then the column sweep; `half` holds the row sweep's
    /// result. Exactly `diffuse_y(diffuse_x(v))` of the loop before P1-17.
    fn solve(&self, v: &[f64], half: &mut [f64], out: &mut [f64]) {
        let (w, h) = (self.w, self.h);
        half.par_chunks_mut(w)
            .zip(v.par_chunks(w))
            .zip(self.rows.par_iter())
            .with_min_len(8)
            .for_each(|((o, row), sys)| sys.solve(row, o));
        // Every column's substitutions at once, a row at a time: each column sees the same
        // operations in the same order as on its own.
        for i in 0..w {
            out[i] = half[i] / self.col_m[i];
        }
        for j in 1..h {
            let (done, rest) = out.split_at_mut(j * w);
            let above = &done[(j - 1) * w..];
            let c0 = j * w;
            for i in 0..w {
                let c = c0 + i;
                rest[i] = (half[c] - self.col_lo[c] * above[i]) / self.col_m[c];
            }
        }
        for j in (0..h.saturating_sub(1)).rev() {
            let (cur, below) = out.split_at_mut((j + 1) * w);
            let c0 = j * w;
            for i in 0..w {
                cur[c0 + i] -= self.col_c[c0 + i] * below[i];
            }
        }
    }
}

#[cfg(test)]
#[path = "ebm2d_reference.rs"]
mod reference;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::climate::ClimateParams;

    /// A grid of equal-angle rows (the zonal model's bands), whatever the map's projection.
    fn grid(cols: usize) -> Grid {
        let fw = cols * 4;
        let fh = fw / 2;
        Grid::with_edges(fw, fh, &vec![1.0e4; fh], 1.0e4, cols, &|y| {
            90.0 - 180.0 * y as f64 / fh as f64
        })
    }

    fn params(g: &Grid) -> Params2d {
        let c = ClimateParams::default();
        let mut ep = EbmParams::from_climate(&c, 23.4, 1.0);
        ep.bands = g.h;
        Params2d::from_climate(&c, ep, 23.4)
    }

    #[test]
    fn cyclic_solver_and_shift() {
        let n = 7;
        let lo: Vec<f64> = (0..n).map(|i| -0.3 - 0.01 * i as f64).collect();
        let up: Vec<f64> = (0..n).map(|i| -0.2 - 0.02 * i as f64).collect();
        let di: Vec<f64> = (0..n).map(|i| 2.0 + 0.1 * i as f64).collect();
        let r: Vec<f64> = (0..n).map(|i| (i as f64).sin()).collect();
        let x = cyclic_thomas(&lo, &di, &up, &r);
        for i in 0..n {
            let lhs = lo[i] * x[(i + n - 1) % n] + di[i] * x[i] + up[i] * x[(i + 1) % n];
            assert!((lhs - r[i]).abs() < 1e-12, "row {i}: {lhs} vs {}", r[i]);
        }
        let mut v: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let sum: f64 = v.iter().sum();
        shift_row(&mut v, 2.25);
        assert!((v.iter().sum::<f64>() - sum).abs() < 1e-12);
        // Cell 5 now holds 3/4 of old 3 and 1/4 of old 2.
        assert!((v[5] - (0.75 * 3.0 + 0.25 * 2.0)).abs() < 1e-12);
        let mut u = vec![4.0; 9];
        shift_row(&mut u, -13.7);
        assert!(u.iter().all(|&x| (x - 4.0).abs() < 1e-12));
    }

    #[test]
    fn a_zonally_uniform_planet_is_the_zonal_model_band_by_band() {
        let g = grid(32);
        let mut p = params(&g);
        // Uniform D (the zonal model's), no wind, no desert.
        let d = p.ebm.d;
        p.d_sea_eq = d;
        p.d_sea_pole = d;
        p.d_land_eq = d;
        p.d_land_pole_n = d;
        p.d_land_pole_s = d;
        p.wind_heat = 0.0;
        let h = g.h;
        // Land by latitude (south to north for the zonal model), an ice sheet on the southern
        // polar rows.
        let land_b: Vec<f64> = (0..h)
            .map(|b| {
                let lat = -90.0 + (b as f64 + 0.5) * 180.0 / h as f64;
                if lat < -65.0 {
                    0.95
                } else if lat > 30.0 {
                    0.5
                } else {
                    0.25 + 0.1 * (lat / 30.0).sin()
                }
            })
            .collect();
        let sheet_b: Vec<f64> = (0..h).map(|b| if b < 3 { 1.0 } else { 0.0 }).collect();
        let mut z = p.ebm.initial_state();
        let r1 = p.ebm.run(&mut z, &land_b, &sheet_b, 4);
        let s = Surface {
            land: (0..g.w * h).map(|c| land_b[h - 1 - c / g.w]).collect(),
            sheet: (0..g.w * h).map(|c| sheet_b[h - 1 - c / g.w]).collect(),
            desert: vec![0.0; g.w * h],
        };
        let mut st = State2d::from_bands(&g, &p.ebm.initial_state());
        let r2 = run(&p, &g, &s, &mut st, None, 4, [0.0; 12]);
        assert_eq!(r1.years, r2.years);
        let mut worst = 0.0f64;
        for c in 0..g.w * h {
            let b = h - 1 - c / g.w;
            for m in 0..12 {
                worst = worst
                    .max((r1.t_land[b][m] - r2.t_land[c][m]).abs())
                    .max((r1.t_ocean[b][m] - r2.t_ocean[c][m]).abs());
            }
        }
        assert!(worst < 1e-6, "largest difference {worst} K");
        assert!((r1.t_global - r2.t_global).abs() < 1e-6);
        assert!((r1.asr - r2.asr).abs() < 1e-6 && (r1.olr - r2.olr).abs() < 1e-6);
    }

    #[test]
    fn without_radiation_heat_is_conserved() {
        let g = grid(32);
        let mut p = params(&g);
        p.ebm.s0 = 0.0;
        p.ebm.a = 0.0;
        p.ebm.b = 1e-12;
        p.ebm.ice_temp_c = -1000.0;
        p.ebm.d_ocean = 0.8;
        p.wind_heat = 0.5;
        let n = g.w * g.h;
        // A ragged continent.
        let land: Vec<f64> = (0..n)
            .map(|c| {
                let (i, j) = (c % g.w, c / g.w);
                if (i * 7 + j * 3) % 11 < 4 {
                    1.0
                } else if (i + j) % 5 == 0 {
                    0.4
                } else {
                    0.0
                }
            })
            .collect();
        let s = Surface {
            land: land.clone(),
            sheet: vec![0.0; n],
            desert: vec![0.0; n],
        };
        let mut st = State2d::from_bands(&g, &p.ebm.initial_state());
        // Some ice to melt and some to grow.
        for c in 0..n {
            st.t_mixed[c] += ((c * 13) % 7) as f64 - 3.0;
            if c % 9 == 0 {
                st.ice_j_m2[c] = 0.3 * ICE_J_M3;
            }
        }
        let area = g.areas();
        let heat = |st: &State2d| -> f64 {
            (0..n)
                .map(|c| {
                    let fl = land[c];
                    area[c]
                        * (fl * p.ebm.c_land * st.t_land[c]
                            + (1.0 - fl)
                                * (p.ebm.c_land * st.t_ocean[c] + p.ebm.c_ocean * st.t_mixed[c]
                                    - st.ice_j_m2[c]))
                })
                .sum()
        };
        let h0 = heat(&st);
        run(&p, &g, &s, &mut st, None, 1, [0.0; 12]);
        let h1 = heat(&st);
        assert!(((h1 - h0) / h0.abs()).abs() < 1e-9, "{h0} → {h1}");
    }

    #[test]
    fn an_earthlike_planet_with_its_ocean_settles_into_energy_balance() {
        // 128 × 64, two continents (an Old World and a narrower New World), Antarctica.
        let fw = 512;
        let g = Grid::new(fw, fw / 2, &vec![1.0e4; fw / 2], 1.0e4, 128);
        let c = ClimateParams::default();
        let mut ep = EbmParams::from_climate(&c, 23.4, 1.0);
        ep.d_ocean = 0.8;
        let p = Params2d::from_climate(&c, ep, 23.4);
        let n = g.w * g.h;
        let land: Vec<f64> = (0..n)
            .map(|cc| {
                let lat = g.lat_c(cc / g.w);
                let lon = (cc % g.w) as f64 * 360.0 / g.w as f64;
                let old = (0.0..140.0).contains(&lon) && (-35.0..72.0).contains(&lat);
                let new = (230.0..300.0).contains(&lon) && (-55.0..70.0).contains(&lat);
                if old || new || lat < -70.0 { 1.0 } else { 0.0 }
            })
            .collect();
        let s = Surface {
            land: land.clone(),
            sheet: (0..n)
                .map(|cc| if g.lat_c(cc / g.w) < -70.0 { 1.0 } else { 0.0 })
                .collect(),
            desert: vec![0.0; n],
        };
        let u = super::super::ocean2d::belt_wind_u(&c, &g, 23.4);
        let oc = super::super::ocean2d::Ocean::build(&super::super::ocean2d::Setup {
            grid: &g,
            land: &land,
            wind_u: &u,
            omega: 7.292e-5,
            radius_m: EARTH_RADIUS_M,
            slab_m: c.ocean_depth_m,
            gyre_efficiency: 1.0,
            overturning_sv: 15.0,
            late_winter_ice: None,
        });
        assert!(oc.sinking.is_some(), "the conveyor found its basin");
        let mut st = State2d::from_bands(&g, &p.ebm.initial_state());
        let t0 = std::time::Instant::now();
        let r = run(&p, &g, &s, &mut st, Some(&oc), 40, [0.0; 12]);
        let per_year = t0.elapsed().as_secs_f64() / f64::from(r.years);
        eprintln!(
            "2-D Earth-like: {} years, {per_year:.3} s/yr, T {:.2} °C, net TOA {:.4} W/m², ψ {:.1} Sv, sweeps {}, atm peak {:.2} PW, ocean peak {:.2} PW",
            r.years,
            r.t_global,
            r.net_toa(),
            oc.psi_max_sv(),
            oc.sweeps,
            r.heat_atm_pw.iter().cloned().fold(f64::MIN, f64::max),
            r.heat_ocean_pw
                .iter()
                .zip(&r.heat_ocean_diff_pw)
                .map(|(a, b)| a + b)
                .fold(f64::MIN, f64::max),
        );
        assert!(r.years < 40, "converged ({} years)", r.years);
        assert!(r.net_toa().abs() < 0.05, "ASR − OLR = {}", r.net_toa());
        assert!((0.0..30.0).contains(&r.t_global), "{}", r.t_global);
    }

    /// Every f64 of a result and a state, as bit patterns.
    fn bits(r: &Result2d, st: &State2d) -> Vec<u64> {
        let mut v: Vec<u64> = Vec::new();
        for a in [&r.t_land, &r.t_ocean, &r.t_mixed, &r.sea_ice] {
            v.extend(a.iter().flatten().map(|x| x.to_bits()));
        }
        for x in [r.asr, r.olr, r.t_global, f64::from(r.years)] {
            v.push(x.to_bits());
        }
        for a in [&r.heat_atm_pw, &r.heat_ocean_diff_pw, &r.heat_ocean_pw] {
            v.extend(a.iter().map(|x| x.to_bits()));
        }
        v.extend(r.itcz.iter().map(|x| x.to_bits()));
        for a in [&st.t_land, &st.t_ocean, &st.t_mixed, &st.ice_j_m2] {
            v.extend(a.iter().map(|x| x.to_bits()));
        }
        v
    }

    #[test]
    fn the_factored_solves_give_the_old_loop_bit_for_bit() {
        // A ragged planet with sea ice, ice sheets, deserts, wind, the slab's diffusion and the
        // ocean's transport; one row without conductance (`RowSystem::Still`) on a second grid.
        let fw = 256;
        let g = Grid::new(fw, fw / 2, &vec![1.0e4; fw / 2], 1.0e4, 64);
        let c = ClimateParams::default();
        let mut ep = EbmParams::from_climate(&c, 23.4, 1.0);
        ep.d_ocean = 0.8;
        let p = Params2d::from_climate(&c, ep, 23.4);
        let n = g.w * g.h;
        let land: Vec<f64> = (0..n)
            .map(|cc| {
                let (i, j) = (cc % g.w, cc / g.w);
                let lat = g.lat_c(j);
                if lat < -66.0 || (i * 7 + j * 3) % 13 < 4 {
                    1.0
                } else if (i + j) % 5 == 0 {
                    0.3
                } else {
                    0.0
                }
            })
            .collect();
        let s = Surface {
            land: land.clone(),
            sheet: (0..n)
                .map(|cc| if g.lat_c(cc / g.w) < -66.0 { 1.0 } else { 0.0 })
                .collect(),
            desert: (0..n).map(|cc| ((cc * 11) % 7) as f64 / 7.0).collect(),
        };
        let u = super::super::ocean2d::belt_wind_u(&c, &g, 23.4);
        let oc = super::super::ocean2d::Ocean::build(&super::super::ocean2d::Setup {
            grid: &g,
            land: &land,
            wind_u: &u,
            omega: 7.292e-5,
            radius_m: EARTH_RADIUS_M,
            slab_m: c.ocean_depth_m,
            gyre_efficiency: 1.0,
            overturning_sv: 15.0,
            late_winter_ice: None,
        });
        let start = || {
            let mut st = State2d::from_bands(&g, &p.ebm.initial_state());
            for cc in 0..n {
                st.t_mixed[cc] += ((cc * 13) % 7) as f64 - 4.0;
                if cc % 9 == 0 {
                    st.ice_j_m2[cc] = 0.4 * ICE_J_M3;
                }
            }
            st
        };
        let go = |threads: usize, new: bool| {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            pool.install(|| {
                let mut st = start();
                let r = if new {
                    run(&p, &g, &s, &mut st, Some(&oc), 3, [0.0; 12])
                } else {
                    super::reference::run(&p, &g, &s, &mut st, Some(&oc), 3, [0.0; 12])
                };
                bits(&r, &st)
            })
        };
        let old = go(1, false);
        for threads in [1, 3, 8] {
            assert!(go(threads, true) == old, "{threads} threads");
        }
        // Without the slab's diffusion and the wind, on a grid with a row of no conductance.
        let mut q = p.clone();
        q.ebm.d_ocean = 0.0;
        q.wind_heat = 0.0;
        q.zonal_ratio = 0.0;
        let g2 = grid(16);
        let n2 = g2.w * g2.h;
        let s2 = Surface {
            land: (0..n2)
                .map(|cc| if cc % 5 < 2 { 1.0 } else { 0.0 })
                .collect(),
            sheet: vec![0.0; n2],
            desert: vec![0.0; n2],
        };
        let mut a = State2d::from_bands(&g2, &q.ebm.initial_state());
        let mut b = a.clone();
        let ra = run(&q, &g2, &s2, &mut a, None, 2, [0.0; 12]);
        let rb = super::reference::run(&q, &g2, &s2, &mut b, None, 2, [0.0; 12]);
        assert!(bits(&ra, &a) == bits(&rb, &b));
    }

    /// The year loop's speed against the old one on the default 128 × 64 grid (P1-17; run with
    /// `--ignored --nocapture`, in release).
    #[test]
    #[ignore]
    fn bench_the_year_loop() {
        let fw = 512;
        let g = Grid::new(fw, fw / 2, &vec![1.0e4; fw / 2], 1.0e4, 128);
        let c = ClimateParams::default();
        let mut ep = EbmParams::from_climate(&c, 23.4, 1.0);
        ep.d_ocean = 0.8;
        let p = Params2d::from_climate(&c, ep, 23.4);
        let n = g.w * g.h;
        let land: Vec<f64> = (0..n)
            .map(|cc| {
                let lat = g.lat_c(cc / g.w);
                let lon = (cc % g.w) as f64 * 360.0 / g.w as f64;
                let old = (0.0..140.0).contains(&lon) && (-35.0..72.0).contains(&lat);
                let new = (230.0..300.0).contains(&lon) && (-55.0..70.0).contains(&lat);
                if old || new || lat < -70.0 { 1.0 } else { 0.0 }
            })
            .collect();
        let s = Surface {
            land: land.clone(),
            sheet: vec![0.0; n],
            desert: vec![0.0; n],
        };
        let u = super::super::ocean2d::belt_wind_u(&c, &g, 23.4);
        let setup = super::super::ocean2d::Setup {
            grid: &g,
            land: &land,
            wind_u: &u,
            omega: 7.292e-5,
            radius_m: EARTH_RADIUS_M,
            slab_m: c.ocean_depth_m,
            gyre_efficiency: 1.0,
            overturning_sv: 15.0,
            late_winter_ice: None,
        };
        let t0 = std::time::Instant::now();
        let oc = super::super::ocean2d::Ocean::build(&setup);
        let build = t0.elapsed().as_secs_f64();
        let years = 12;
        let t0 = std::time::Instant::now();
        let mut st = State2d::from_bands(&g, &p.ebm.initial_state());
        super::reference::run(&p, &g, &s, &mut st, Some(&oc), years, [0.0; 12]);
        let old = t0.elapsed().as_secs_f64();
        let t0 = std::time::Instant::now();
        let mut st = State2d::from_bands(&g, &p.ebm.initial_state());
        let r = run(&p, &g, &s, &mut st, Some(&oc), years, [0.0; 12]);
        let new = t0.elapsed().as_secs_f64();
        eprintln!(
            "ebm2d bench: ocean build {build:.3} s ({} sweeps); {} years: old {old:.3} s, new {new:.3} s",
            oc.sweeps, r.years
        );
    }

    #[test]
    fn the_result_does_not_depend_on_the_thread_count() {
        let g = grid(32);
        let p = params(&g);
        let n = g.w * g.h;
        let s = Surface {
            land: (0..n)
                .map(|c| if (c % g.w) < 10 { 0.9 } else { 0.0 })
                .collect(),
            sheet: vec![0.0; n],
            desert: (0..n)
                .map(|c| if (c % g.w) < 4 { 0.5 } else { 0.0 })
                .collect(),
        };
        let go = |threads: usize| {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            pool.install(|| {
                let mut st = State2d::from_bands(&g, &p.ebm.initial_state());
                run(&p, &g, &s, &mut st, None, 3, [0.0; 12])
            })
        };
        let (a, b) = (go(1), go(3));
        assert_eq!(a, b);
    }
}

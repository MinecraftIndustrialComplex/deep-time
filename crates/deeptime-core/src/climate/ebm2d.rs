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

    /// How the year loop can run [`step`](Self::step): `Whole` (the default) as a call of its
    /// own, or `Rows` as `n` sub-steps of `ds` seconds that [`substep_rows`](Self::substep_rows)
    /// does for blocks of rows inside the loop's parallel regions, or `Nothing` where the ocean
    /// moves no heat (P1-19: fewer, larger parallel regions).
    fn sub_steps(&self, _dt: f64) -> SubSteps {
        SubSteps::Whole
    }

    /// One sub-step for a block of rows: the same values and sums as the matching part of
    /// `step`, bit for bit. Only called when `sub_steps` said `Rows`.
    fn substep_rows(&self, _a: SubstepRows<'_>) {
        unreachable!("sub_steps said Whole")
    }
}

/// How the year loop runs an ocean's step (see [`OceanTransport::sub_steps`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SubSteps {
    Nothing,
    Whole,
    Rows { n: usize, ds: f64, off: f64 },
}

/// A block of rows' scratch for [`OceanTransport::substep_rows`]: the heat the faces carry.
#[derive(Clone, Debug, Default)]
pub struct OceanScratch {
    pub ce: Vec<f64>,
    pub cn: Vec<f64>,
}

/// One sub-step of an ocean's step for rows `j0..` (`out.len() / w` of them).
pub struct SubstepRows<'a> {
    /// The sub-step's length, s.
    pub ds: f64,
    /// The rows of the field the sub-step reads, from row `src_j0` on (it needs the rows of `out`
    /// and the one above and the one below): the temperatures are `src[c] - off`.
    pub src: &'a [f64],
    pub src_j0: usize,
    pub off: f64,
    /// The first row of `out`.
    pub j0: usize,
    /// The new temperatures of those rows (the shifted ones; with `last`, °C).
    pub out: &'a mut [f64],
    /// The rows `own.0..own.1` (inside those of `out`) are the block's own: the heat moved across
    /// the north face of each of them (the field's row 0 has none) is accumulated into `north`,
    /// one per own row.
    pub own: (usize, usize),
    pub north: &'a mut [f64],
    pub scratch: &'a mut OceanScratch,
    /// On a step's last sub-step: the slab's °C before the step, which cells without water keep;
    /// `out` is then in °C.
    pub last: Option<&'a [f64]>,
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
    // The parallel regions work on blocks of whole rows (P1-19): a block is `bs` rows. A step is
    // one region (and the columns' sweeps beside it), not one per phase: for each block of rows
    // the zonal shift, the ocean's sub-steps (each block redoing the few rows of halo they need
    // from their neighbours), the freezing and the month's means, and then the next step's local
    // terms and row sweeps. Every cell's arithmetic and every sum keep their order, so the
    // values are those of the phases run one after the other (the tests keep the old loop and
    // compare bit for bit).
    let bs = h.div_ceil(2 * rayon::current_num_threads()).max(1);
    let nb = h.div_ceil(bs);
    let ctx = Ctx {
        p,
        w,
        h,
        bs,
        dt,
        pm: per_month as f64,
        lat: &lat,
        f: &f,
        free_sea: &free_sea,
        free_land: &free_land,
        sheet: &sheet,
        dx_row: &dx_row,
        ocean,
        atm: &atm,
        slab: slab.as_ref(),
    };
    let oc = ocean.map_or(SubSteps::Nothing, |o| o.sub_steps(dt));
    // The ocean's sub-steps in groups of at most `MAX_GROUP` (each group is one region).
    let groups: Vec<usize> = match oc {
        SubSteps::Rows { n: subs, .. } => {
            let g = subs.div_ceil(MAX_GROUP);
            (0..g)
                .map(|i| subs / g + usize::from(i < subs % g))
                .collect()
        }
        _ => vec![],
    };
    // The steps' buffers: each cell's absorbed and emitted radiation (this step's and the next
    // one's), T̄ before and after the atmosphere's transport, and the solves' intermediates (the
    // slab's own, so that both diffusions can run at once).
    let mut a_sr = [vec![0.0; n], vec![0.0; n]];
    let mut a_lr = [vec![0.0; n], vec![0.0; n]];
    let mut mean = vec![0.0; n];
    let mut tb = vec![0.0; n];
    let mut half_a = vec![0.0; n];
    let slab_n = if slab.is_some() { n } else { 0 };
    let mut half_s = vec![0.0; slab_n];
    let mut mean_s = vec![0.0; slab_n];
    // The ocean's groups of sub-steps pass the field on in one of two buffers and end in a third,
    // which then becomes the slab (`state.t_mixed` goes back to being the spare).
    let mut th_a = vec![0.0; if groups.len() > 1 { n } else { 0 }];
    let mut th_b = vec![0.0; if groups.len() > 1 { n } else { 0 }];
    let mut tm_new = vec![0.0; if groups.is_empty() { 0 } else { n }];
    let mut scratch: Vec<BlockScratch> = (0..nb).map(|_| BlockScratch::default()).collect();
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
        // The ocean's, one per row (row j's north face; row 0 has none), so that the blocks
        // each hold their own.
        let mut north_o = vec![0.0; h];
        // The year's first step's local terms and row sweeps (the later steps' run at the end
        // of the step before).
        stage(
            &ctx,
            &Plan {
                month: 0,
                itcz_m: 0.0,
                shift: false,
                sub: None,
                means: false,
                next: Some(&q[0]),
            },
            StageArrays {
                tb: &mut tb,
                mean: &mut mean,
                tl: &mut state.t_land,
                to: &mut state.t_ocean,
                ice: &mut state.ice_j_m2,
                out: &mut state.t_mixed,
                north: &mut north_o,
                scratch: &mut scratch,
                ml: &mut ml,
                mo: &mut mo,
                mm: &mut mm,
                mi: &mut mi,
                tbar: &mut tbar,
                rt: &mut row_t,
                sr: &mut a_sr[0],
                lr: &mut a_lr[0],
                half_a: &mut half_a,
                half_s: &mut half_s,
            },
        );
        for st in 0..q.len() {
            let month = st / per_month;
            let itcz_m = itcz[month];
            let qn = q.get(st + 1).map(|v| &v[..]);
            // 2. The columns' sweeps of the atmosphere's diffusion of T̄ (implicit ADI: the rows'
            //    were done with the local terms) and the slab's, with the heat each moved north,
            //    side by side (they share nothing).
            rayon::join(
                || {
                    atm.columns(&half_a, &mut tb);
                    northward(w, &k_ns, &tb, r2 * dt, &mut north_atm);
                },
                || {
                    if let Some(slab) = &slab {
                        slab.columns(&half_s, &mut mean_s);
                        northward(w, &ko_ns, &mean_s, r2 * dt, &mut north_od);
                    }
                },
            );
            if slab.is_some() {
                // The slab's new temperatures.
                std::mem::swap(&mut state.t_mixed, &mut mean_s);
            }
            // 3.–5. The prevailing wind's zonal shift of T̄ (both tiles then take T̄'s change),
            //    the ocean's transport of the slab's heat in its sub-steps, freezing and melting
            //    at the slab and the month's means (each cell's own, a row's zonal sum in cell
            //    order); then, for the next step, 1. and 2.'s rows; and the step's global sums,
            //    in cell order.
            let (cur_sr, nxt_sr) = pick(&mut a_sr, st % 2);
            let (cur_lr, nxt_lr) = pick(&mut a_lr, st % 2);
            match oc {
                SubSteps::Nothing => stage(
                    &ctx,
                    &Plan {
                        month,
                        itcz_m,
                        shift: true,
                        sub: None,
                        means: true,
                        next: qn,
                    },
                    StageArrays {
                        tb: &mut tb,
                        mean: &mut mean,
                        tl: &mut state.t_land,
                        to: &mut state.t_ocean,
                        ice: &mut state.ice_j_m2,
                        out: &mut state.t_mixed,
                        north: &mut north_o,
                        scratch: &mut scratch,
                        ml: &mut ml,
                        mo: &mut mo,
                        mm: &mut mm,
                        mi: &mut mi,
                        tbar: &mut tbar,
                        rt: &mut row_t,
                        sr: &mut *nxt_sr,
                        lr: &mut *nxt_lr,
                        half_a: &mut half_a,
                        half_s: &mut half_s,
                    },
                ),
                SubSteps::Whole => {
                    stage(
                        &ctx,
                        &Plan {
                            month,
                            itcz_m,
                            shift: true,
                            sub: None,
                            means: false,
                            next: None,
                        },
                        StageArrays {
                            tb: &mut tb,
                            mean: &mut mean,
                            tl: &mut state.t_land,
                            to: &mut state.t_ocean,
                            ice: &mut state.ice_j_m2,
                            out: &mut [],
                            north: &mut north_o,
                            scratch: &mut scratch,
                            ml: &mut ml,
                            mo: &mut mo,
                            mm: &mut mm,
                            mi: &mut mi,
                            tbar: &mut tbar,
                            rt: &mut row_t,
                            sr: &mut *nxt_sr,
                            lr: &mut *nxt_lr,
                            half_a: &mut half_a,
                            half_s: &mut half_s,
                        },
                    );
                    if let Some(o) = ocean {
                        o.step(
                            &mut state.t_mixed,
                            dt,
                            north_o.get_mut(1..).unwrap_or_default(),
                        );
                    }
                    stage(
                        &ctx,
                        &Plan {
                            month,
                            itcz_m,
                            shift: false,
                            sub: None,
                            means: true,
                            next: qn,
                        },
                        StageArrays {
                            tb: &mut tb,
                            mean: &mut mean,
                            tl: &mut state.t_land,
                            to: &mut state.t_ocean,
                            ice: &mut state.ice_j_m2,
                            out: &mut state.t_mixed,
                            north: &mut north_o,
                            scratch: &mut scratch,
                            ml: &mut ml,
                            mo: &mut mo,
                            mm: &mut mm,
                            mi: &mut mi,
                            tbar: &mut tbar,
                            rt: &mut row_t,
                            sr: &mut *nxt_sr,
                            lr: &mut *nxt_lr,
                            half_a: &mut half_a,
                            half_s: &mut half_s,
                        },
                    );
                }
                SubSteps::Rows { ds, off: t_off, .. } => {
                    for (gi, &subs) in groups.iter().enumerate() {
                        let first = gi == 0;
                        let last = gi + 1 == groups.len();
                        // Group g reads the slab (the first) or the field group g − 1 wrote, and
                        // writes the other field, or, the last, the new slab.
                        let (src, off) = if first {
                            (&state.t_mixed, t_off)
                        } else {
                            (&th_a, 0.0)
                        };
                        let to_out: &mut Vec<f64> = if last { &mut tm_new } else { &mut th_b };
                        stage(
                            &ctx,
                            &Plan {
                                month,
                                itcz_m,
                                shift: first,
                                sub: Some(Sub {
                                    src,
                                    off,
                                    ds,
                                    n: subs,
                                    last: last.then_some(&state.t_mixed[..]),
                                }),
                                means: last,
                                next: if last { qn } else { None },
                            },
                            StageArrays {
                                tb: &mut tb,
                                mean: &mut mean,
                                tl: &mut state.t_land,
                                to: &mut state.t_ocean,
                                ice: &mut state.ice_j_m2,
                                out: to_out,
                                north: &mut north_o,
                                scratch: &mut scratch,
                                ml: &mut ml,
                                mo: &mut mo,
                                mm: &mut mm,
                                mi: &mut mi,
                                tbar: &mut tbar,
                                rt: &mut row_t,
                                sr: &mut *nxt_sr,
                                lr: &mut *nxt_lr,
                                half_a: &mut half_a,
                                half_s: &mut half_s,
                            },
                        );
                        if !last {
                            std::mem::swap(&mut th_a, &mut th_b);
                        }
                    }
                    std::mem::swap(&mut state.t_mixed, &mut tm_new);
                }
            }
            for c in 0..n {
                asr += area[c] * cur_sr[c];
                olr += area[c] * cur_lr[c];
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
        out.heat_ocean_pw = north_o.iter().skip(1).map(|x| x / pw).collect();
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

/// The most ocean sub-steps one region does (each block of rows redoes `n − 1` rows of halo on
/// each side for `n` of them); more go in further regions.
const MAX_GROUP: usize = 4;

/// What the year loop's regions share.
struct Ctx<'a> {
    p: &'a Params2d,
    w: usize,
    h: usize,
    /// Rows per block.
    bs: usize,
    dt: f64,
    /// Steps per month.
    pm: f64,
    lat: &'a [f64],
    /// Land share of every cell.
    f: &'a [f64],
    free_sea: &'a [f64],
    free_land: &'a [f64],
    sheet: &'a [f64],
    /// The zonal shift's cell width of every row, m.
    dx_row: &'a [f64],
    ocean: Option<&'a dyn OceanTransport>,
    atm: &'a Adi,
    slab: Option<&'a Adi>,
}

/// Ocean sub-steps inside a region: the field they read, [`SubstepRows::src`] and `off`, their
/// length and number, and, on a step's last, the slab before the step.
struct Sub<'a> {
    src: &'a [f64],
    off: f64,
    ds: f64,
    n: usize,
    last: Option<&'a [f64]>,
}

/// What one region does for every block of rows, in this order: the zonal shift of T̄ (with both
/// tiles taking the change), ocean sub-steps, the freezing and melting at the slab and the
/// month's means, and the next step's local terms and row sweeps (given its insolation).
struct Plan<'a> {
    month: usize,
    /// The month's ITCZ, degrees (it steers the wind's belts).
    itcz_m: f64,
    shift: bool,
    sub: Option<Sub<'a>>,
    means: bool,
    next: Option<&'a [f64]>,
}

/// The fields a region works on, whole; it takes the ones its plan needs, a block's rows each.
struct StageArrays<'a> {
    tb: &'a mut [f64],
    mean: &'a mut [f64],
    tl: &'a mut [f64],
    to: &'a mut [f64],
    ice: &'a mut [f64],
    /// The slab: the sub-steps' result, or the slab itself where there are none.
    out: &'a mut [f64],
    /// The ocean's northward heat, one per row.
    north: &'a mut [f64],
    scratch: &'a mut [BlockScratch],
    ml: &'a mut [[f64; 12]],
    mo: &'a mut [[f64; 12]],
    mm: &'a mut [[f64; 12]],
    mi: &'a mut [[f64; 12]],
    tbar: &'a mut [f64],
    rt: &'a mut [[f64; 12]],
    /// The next step's absorbed and emitted radiation, and the two diffusions' row sweeps.
    sr: &'a mut [f64],
    lr: &'a mut [f64],
    half_a: &'a mut [f64],
    half_s: &'a mut [f64],
}

/// A block's scratch: the zonal shift's source row, the ocean's face fluxes and its halo rows.
#[derive(Default)]
struct BlockScratch {
    src: Vec<f64>,
    oc: OceanScratch,
    th: [Vec<f64>; 2],
}

/// A block of rows of a region.
struct Block<'a> {
    j0: usize,
    tb: &'a mut [f64],
    mean: &'a mut [f64],
    tl: &'a mut [f64],
    to: &'a mut [f64],
    ice: &'a mut [f64],
    out: &'a mut [f64],
    north: &'a mut [f64],
    scr: &'a mut BlockScratch,
    means: Option<MeansBlock<'a>>,
    next: Option<NextBlock<'a>>,
}

struct MeansBlock<'a> {
    ml: &'a mut [[f64; 12]],
    mo: &'a mut [[f64; 12]],
    mm: &'a mut [[f64; 12]],
    mi: &'a mut [[f64; 12]],
    tbar: &'a mut [f64],
    rt: &'a mut [[f64; 12]],
}

struct NextBlock<'a> {
    sr: &'a mut [f64],
    lr: &'a mut [f64],
    half_a: &'a mut [f64],
    half_s: &'a mut [f64],
}

/// The next chunk of a chunked field, or an empty one where the field is not being used.
fn take<'a, T>(it: &mut Option<std::slice::ChunksMut<'a, T>>) -> &'a mut [T] {
    it.as_mut().and_then(Iterator::next).unwrap_or_default()
}

/// The snow factor of a tile at `t` °C.
fn frozen(ep: &EbmParams, t: f64) -> f64 {
    1.0 - smoothstep(ep.ice_temp_c - 3.0, ep.ice_temp_c + 3.0, t)
}

/// Splits a pair of buffers into this step's (`cur`, read) and the next step's (written).
fn pick(pair: &mut [Vec<f64>; 2], cur: usize) -> (&Vec<f64>, &mut Vec<f64>) {
    let (x, y) = pair.split_at_mut(1);
    if cur == 0 {
        (&x[0], &mut y[0])
    } else {
        (&y[0], &mut x[0])
    }
}

/// One row's local terms (`ebm.rs`'s implicit 3 × 3 solve per cell): each cell writes its own
/// tiles, its budget (`sr`, `lr`) and its new T̄ (`mean`, the atmosphere's diffusion's input).
#[allow(clippy::too_many_arguments)]
fn local_row(
    c: &Ctx,
    j: usize,
    qj: f64,
    tl: &mut [f64],
    to: &mut [f64],
    tm: &mut [f64],
    ice: &mut [f64],
    sr: &mut [f64],
    lr: &mut [f64],
    mean: &mut [f64],
) {
    let ep = &c.p.ebm;
    let dt = c.dt;
    let free_sea = c.free_sea[j];
    for i in 0..c.w {
        let cc = j * c.w + i;
        let f = c.f[cc];
        let (to_v, tl_v, tm_v) = (to[i], tl[i], tm[i]);
        let mut ice_v = ice[i];
        let h_ice = ice_v / ICE_J_M3;
        let cover = (h_ice / FULL_ICE_M).min(1.0);
        let insulated = smoothstep(0.5 * SEA_ICE_CAP_M, SEA_ICE_CAP_M, h_ice);
        let gc = OPEN_WATER_W_M2_K * (1.0 - cover)
            + (ICE_W_M_K / h_ice.max(0.05)).min(OPEN_WATER_W_M2_K) * cover * (1.0 - insulated);
        let ao = free_sea + (ep.ai - free_sea) * frozen(ep, to_v).max(cover);
        let al = c.free_land[cc] + (ep.ai - c.free_land[cc]) * frozen(ep, tl_v).max(c.sheet[cc]);
        let so = qj * (1.0 - ao);
        let sl = qj * (1.0 - al);
        let (cl, cs, cm) = (ep.c_land / dt, ep.c_land / dt, ep.c_ocean / dt);
        let (xl, xo) = (ep.exchange * (1.0 - f), ep.exchange * f);
        let (a_l, b_l) = super::co2::longwave(
            ep.a,
            ep.b,
            ep.co2.as_ref(),
            tl_v,
            ep.snow_cloud,
            frozen(ep, tl_v).max(c.sheet[cc]),
        );
        let (a_o, b_o) =
            super::co2::longwave(ep.a, ep.b, ep.co2.as_ref(), to_v, ep.snow_cloud, cover);
        let (r1, r2, r3) = (cl * tl_v + sl - a_l, cs * to_v + so - a_o, cm * tm_v);
        let a33 = cm + gc;
        let a11 = cl + b_l + xl;
        let a22 = cs + b_o + xo + gc - gc * gc / a33;
        let r2 = r2 + gc * r3 / a33;
        let det = a11 * a22 - xl * xo;
        let tl2 = (r1 * a22 + xl * r2) / det;
        let mut to2 = (a11 * r2 + xo * r1) / det;
        let mut tm2 = (r3 + gc * to2) / a33;
        sr[i] = f * sl + (1.0 - f) * so;
        lr[i] = f * (a_l + b_l * tl2) + (1.0 - f) * (a_o + b_o * to2);
        if ice_v > 0.0 && to2 > 0.0 {
            let melt = (to2 * ep.c_land).min(ice_v);
            ice_v -= melt;
            to2 -= melt / ep.c_land;
        }
        // At once, as `ebm.rs` does: the slab's diffusion then never sees a slab below the
        // freezing point.
        freeze_melt(&mut tm2, &mut ice_v, ep.c_ocean);
        (tl[i], to[i], tm[i], ice[i]) = (tl2, to2, tm2, ice_v);
        mean[i] = f * tl2 + (1.0 - f) * to2;
    }
}

/// One parallel region of the year loop: `plan`'s parts for every block of rows (see [`Plan`]),
/// each part the same operations in the same order as it was as a region of its own.
fn stage(c: &Ctx, plan: &Plan, a: StageArrays) {
    let (p, w, h, bs, dt, pm) = (c.p, c.w, c.h, c.bs, c.dt, c.pm);
    let bw = bs * w;
    let month = plan.month;
    let StageArrays {
        tb,
        mean,
        tl,
        to,
        ice,
        out,
        north,
        scratch,
        ml,
        mo,
        mm,
        mi,
        tbar,
        rt,
        sr,
        lr,
        half_a,
        half_s,
    } = a;
    let needs_ice = plan.means || plan.next.is_some();
    let mut tb = plan.shift.then(|| tb.chunks_mut(bw));
    let mut mean = (plan.shift || plan.next.is_some()).then(|| mean.chunks_mut(bw));
    let mut tl = tl.chunks_mut(bw);
    let mut to = to.chunks_mut(bw);
    let mut ice = needs_ice.then(|| ice.chunks_mut(bw));
    let mut out = (!out.is_empty()).then(|| out.chunks_mut(bw));
    let mut north = plan.sub.is_some().then(|| north.chunks_mut(bs));
    let mut ml = plan.means.then(|| ml.chunks_mut(bw));
    let mut mo = plan.means.then(|| mo.chunks_mut(bw));
    let mut mm = plan.means.then(|| mm.chunks_mut(bw));
    let mut mi = plan.means.then(|| mi.chunks_mut(bw));
    let mut tbar = plan.means.then(|| tbar.chunks_mut(bw));
    let mut rt = plan.means.then(|| rt.chunks_mut(bs));
    let nxt = plan.next.is_some();
    let mut sr = nxt.then(|| sr.chunks_mut(bw));
    let mut lr = nxt.then(|| lr.chunks_mut(bw));
    let mut half_a = nxt.then(|| half_a.chunks_mut(bw));
    let mut half_s = nxt.then(|| half_s.chunks_mut(bw));
    let mut blocks: Vec<Block> = scratch
        .iter_mut()
        .enumerate()
        .map(|(b, scr)| Block {
            j0: b * bs,
            tb: take(&mut tb),
            mean: take(&mut mean),
            tl: tl.next().unwrap_or_default(),
            to: to.next().unwrap_or_default(),
            ice: take(&mut ice),
            out: take(&mut out),
            north: take(&mut north),
            scr,
            means: plan.means.then(|| MeansBlock {
                ml: take(&mut ml),
                mo: take(&mut mo),
                mm: take(&mut mm),
                mi: take(&mut mi),
                tbar: take(&mut tbar),
                rt: take(&mut rt),
            }),
            next: nxt.then(|| NextBlock {
                sr: take(&mut sr),
                lr: take(&mut lr),
                half_a: take(&mut half_a),
                half_s: take(&mut half_s),
            }),
        })
        .collect();
    blocks.par_iter_mut().for_each(|b| {
        let rows = b.tl.len() / w;
        if plan.shift {
            let hemi = p.hemi_shift[month];
            for k in 0..rows {
                let j = b.j0 + k;
                let r = k * w..(k + 1) * w;
                let row = &mut b.tb[r.clone()];
                if p.wind_heat > 0.0 {
                    let phi = c.lat[j];
                    let pb = phi - belts::shift(phi, plan.itcz_m, hemi);
                    let u = p.wind_heat * low_level_wind(pb);
                    shift_row_with(row, u * dt / c.dx_row[j], &mut b.scr.src);
                }
                add_change(row, &b.mean[r.clone()], &mut b.tl[r.clone()], &mut b.to[r]);
            }
        }
        if let (Some(s), Some(o)) = (&plan.sub, c.ocean) {
            // Sub-step k finds the block's rows (and the halo it still needs: `n − 1 − k` rows
            // on each side) from the rows of sub-step k − 1; the last ends on the block's own.
            let halo = s.n - 1;
            let own = (b.j0, b.j0 + rows);
            let [t0, t1] = &mut b.scr.th;
            let mut src_j0 = 0;
            for k in 0..s.n {
                let a0 = own.0.saturating_sub(halo - k);
                let a1 = (own.1 + halo - k).min(h);
                let last = k + 1 == s.n;
                let (rd, wr) = if k % 2 == 1 {
                    (&*t0, &mut *t1)
                } else {
                    (&*t1, &mut *t0)
                };
                let (src, off) = if k == 0 {
                    (s.src, s.off)
                } else {
                    (&rd[..], 0.0)
                };
                let dst: &mut [f64] = if last {
                    &mut *b.out
                } else {
                    wr.resize((a1 - a0) * w, 0.0);
                    &mut wr[..]
                };
                o.substep_rows(SubstepRows {
                    ds: s.ds,
                    src,
                    src_j0: if k == 0 { 0 } else { src_j0 },
                    off,
                    j0: a0,
                    out: dst,
                    north: &mut *b.north,
                    own,
                    scratch: &mut b.scr.oc,
                    last: if last { s.last } else { None },
                });
                src_j0 = a0;
            }
        }
        if let Some(m) = &mut b.means {
            for k in 0..rows {
                let j = b.j0 + k;
                for i in 0..w {
                    let x = k * w + i;
                    let cc = j * w + i;
                    freeze_melt(&mut b.out[x], &mut b.ice[x], p.ebm.c_ocean);
                    m.ml[x][month] += b.tl[x] / pm;
                    m.mo[x][month] += b.to[x] / pm;
                    m.mm[x][month] += b.out[x] / pm;
                    m.mi[x][month] += (b.ice[x] / ICE_J_M3 / FULL_ICE_M).min(1.0) / pm;
                    let t = c.f[cc] * b.tl[x] + (1.0 - c.f[cc]) * b.to[x];
                    m.tbar[x] = t;
                    m.rt[k][month] += t / pm;
                }
            }
        }
        if let (Some(qn), Some(nx)) = (plan.next, &mut b.next) {
            for k in 0..rows {
                let j = b.j0 + k;
                let r = k * w..(k + 1) * w;
                local_row(
                    c,
                    j,
                    qn[j],
                    &mut b.tl[r.clone()],
                    &mut b.to[r.clone()],
                    &mut b.out[r.clone()],
                    &mut b.ice[r.clone()],
                    &mut nx.sr[r.clone()],
                    &mut nx.lr[r.clone()],
                    &mut b.mean[r.clone()],
                );
                // 2. The atmosphere's diffusion of T̄, implicit ADI: this row's sweep (cyclic);
                //    the columns follow. And the slab's, from the slab as the local terms left
                //    it (the zonal shift never touches it).
                c.atm.row(j, &b.mean[r.clone()], &mut nx.half_a[r.clone()]);
                if let Some(slab) = c.slab {
                    slab.row(j, &b.out[r.clone()], &mut nx.half_s[r]);
                }
            }
        }
    });
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
    // Four edges at a time: each edge's sum is still one chain of adds in column order, but the
    // four chains overlap instead of waiting on each other's adds (P1-19).
    const LANES: usize = 4;
    let first = acc.len() - acc.len() % LANES;
    let (quads, rest) = acc.as_chunks_mut::<LANES>();
    for (q, a) in quads.iter_mut().enumerate() {
        let c0 = q * LANES * w;
        let mut s = [0.0f64; LANES];
        for i in 0..w {
            for (l, sl) in s.iter_mut().enumerate() {
                let c = c0 + l * w + i;
                *sl += k[c] * (t[c + w] - t[c]);
            }
        }
        for (x, sl) in a.iter_mut().zip(s) {
            *x += sl * scale;
        }
    }
    for (e, a) in rest.iter_mut().enumerate() {
        let e = first + e;
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

    /// Row `j`'s sweep: `out` = the row's new values from `v`'s.
    fn row(&self, j: usize, v: &[f64], out: &mut [f64]) {
        self.rows[j].solve(v, out);
    }

    /// The column sweeps over the field the row sweeps made (`half`): `out` = v after the row
    /// sweep and then the column sweep. Exactly `diffuse_y(diffuse_x(v))` of the loop before
    /// P1-17.
    fn columns(&self, half: &[f64], out: &mut [f64]) {
        let (w, h) = (self.w, self.h);
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
        let go = |p: &Params2d, threads: usize, new: bool| {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            pool.install(|| {
                let mut st = start();
                let r = if new {
                    run(p, &g, &s, &mut st, Some(&oc), 3, [0.0; 12])
                } else {
                    super::reference::run(p, &g, &s, &mut st, Some(&oc), 3, [0.0; 12])
                };
                bits(&r, &st)
            })
        };
        let old = go(&p, 1, false);
        for threads in [1, 3, 8] {
            assert!(go(&p, threads, true) == old, "{threads} threads");
        }
        // Monthly steps: the ocean takes many sub-steps, in more than one region (P1-19).
        let mut pm = p.clone();
        pm.ebm.steps_per_year = 12;
        let dt = YEAR_S / 12.0;
        assert!(
            matches!(oc.sub_steps(dt), SubSteps::Rows { n, .. } if n > MAX_GROUP),
            "{:?}",
            oc.sub_steps(dt)
        );
        let old = go(&pm, 1, false);
        for threads in [1, 3, 8] {
            assert!(
                go(&pm, threads, true) == old,
                "monthly steps, {threads} threads"
            );
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
        let dt = YEAR_S / p.ebm.steps_per_year.max(12) as f64;
        let time = |which: u8| {
            let t0 = std::time::Instant::now();
            let mut st = State2d::from_bands(&g, &p.ebm.initial_state());
            let r = match which {
                0 => super::reference::run(&p, &g, &s, &mut st, Some(&oc), years, [0.0; 12]),
                _ => run(&p, &g, &s, &mut st, Some(&oc), years, [0.0; 12]),
            };
            (t0.elapsed().as_secs_f64(), r.years)
        };
        let (mut ms, mut ns) = (vec![], vec![]);
        for _ in 0..5 {
            ms.push(time(0).0);
            ns.push(time(1).0);
        }
        let med = |v: &mut Vec<f64>| {
            v.sort_by(f64::total_cmp);
            (v[0], v[v.len() / 2])
        };
        let (m, n_) = (med(&mut ms), med(&mut ns));
        eprintln!(
            "ebm2d bench: ocean build {build:.3} s ({} sweeps), {:?}; {years} years, min/median of 5: old {:.3}/{:.3} s, new {:.3}/{:.3} s",
            oc.sweeps,
            oc.sub_steps(dt),
            m.0,
            m.1,
            n_.0,
            n_.1
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

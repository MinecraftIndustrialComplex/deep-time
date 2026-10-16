// SPDX-License-Identifier: LGPL-3.0-or-later
//! Rain from a monthly moisture budget on the climate's coarse grid (P1-26 T4;
//! `docs/design/p1-26-per-cell-climate.md` §3). Built behind `climate.rain_model = budget`; the
//! belt rain of `snapshot.rs` stays the default until the budget beats it on Earth's relief (§3.8).
//!
//! The column water vapour W (kg/m²) lives a week or so, much less than a month, so each month's
//! W is the steady state of that month's evaporation, transport and precipitation:
//!
//! ```text
//! ∇·(V W) − ∇·(K_q ∇W) = E(W) − P(W) − S_oro
//! ```
//!
//! - **Winds** V (§3.2): the belt winds' zonal component (`belts::winds` by the regimes at the
//!   belt latitude φ′ of the cell's column and month, its ITCZ being the thermal equator of the
//!   column's T̄), the three-cell meridional flow in φ′ (convergent at the ITCZ and at 60°,
//!   divergent at 30° and the poles), and the thermal wind of the departure of T̄ from its row mean:
//!   p′ = −κ_p T̄′ turned into a surface wind by the linear friction–Coriolis balance
//!   (r u − f v = −p′_x/ρ, r v + f u = −p′_y/ρ; Lindzen & Nigam 1987, read through Back &
//!   Bretherton 2009): a summer continent draws sea air in, a winter one pushes it out.
//! - **Evaporation** per tile (§3.3, GREB's bulk form, Stassen et al. 2019; Dommenget & Flöter
//!   2011's turbulent wind added to the mean wind): E = ρ_a C_E (|V| + u_t) q_s(T)(1 − W/W_sat),
//!   the sea's × the open water, the land's × the soil's β(S) = min(1, S/(0.75 S_max)) (Manabe
//!   1969's bucket, S stepped month by month by P − E − runoff and cycled over the year).
//! - **Saturation** W_sat = 70 e^(0.065 (T̄ − 27)) (1 − f + f e^(−z/2,500 m)) kg/m², z the land's
//!   lowland height (high land holds less water).
//! - **Precipitation** (§3.4): P = stab (W/τ_P) e^(a (W/W_sat − r_ref)), Bretherton, Peters & Back
//!   2004's monthly fit (a = 11.4) written relative to W, τ_P 6.6 days, r_ref 0.7; stab =
//!   e^(0.3 min(0, T_o − ⟨T_o⟩_row)) over the sea share (cold-current deserts).
//!
//! **Numerics** (§3.5): upwind flux form with a two-point diffusive flux, so the linearized system
//! is an M-matrix; zonal line relaxation (one Newton step of a cyclic tridiagonal per row, the
//! rows' neighbours held, sweeping north to south and back) until every cell's residual is below
//! 10⁻¹⁰ kg/m²/s. The fluxes telescope, so at convergence Σ A E = Σ A P + Σ A S_oro.

use super::belts;
use super::coarse::Grid;

/// Seconds in a month (a twelfth of the year).
pub const MONTH_S: f64 = 365.25 * 86_400.0 / 12.0;

/// The budget's parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct MoistureParams {
    /// Eddy diffusivity of the column water, m²/s.
    pub k_q: f64,
    /// Precipitation: time scale, s; e-folding rate in column relative humidity; reference RH.
    pub tau_p_s: f64,
    pub rh_slope: f64,
    pub r_ref: f64,
    /// Bulk transfer coefficient of evaporation and the turbulent wind over sea and land, m/s.
    pub c_e: f64,
    pub u_t_sea: f64,
    pub u_t_land: f64,
    /// Soil bucket capacity, kg/m² (mm).
    pub bucket_mm: f64,
    /// Thermal wind: pressure per kelvin of T̄ departure, Pa/K; linear friction, 1/s.
    pub kappa_p: f64,
    pub friction: f64,
    /// Rain suppression over cold sea, 1/K.
    pub stab_k: f64,
    /// The ITCZ's largest shift and its smoothing along a row (coarse cells, box radius).
    pub itcz_max_deg: f64,
    pub itcz_smooth_cells: usize,
    /// The subtropical belts' seasonal shift of each month, degrees.
    pub hemi_shift: [f64; 12],
    /// The belt winds (`belts::winds`).
    pub winds: [(f64, f64); belts::REGIMES],
    /// Rotation rate, rad/s; radius, m.
    pub omega: f64,
    pub radius_m: f64,
}

/// Air density at the surface, kg/m³ (as the ocean's wind stress, from memory).
pub const RHO_AIR: f64 = 1.2; // ledger: CLM-177
/// Saturation column water at 27 °C, kg/m², its rate per kelvin, and the vapour scale height
/// over land, m (design §3.3; WP-14a: each consistent with a source, none taken from one).
pub const W_SAT_27: f64 = 70.0; // ledger: CLM-144
pub const W_SAT_PER_K: f64 = 0.065; // ledger: CLM-144
pub const W_SAT_SCALE_M: f64 = 2500.0; // ledger: CLM-144
/// The three-cell surface flow's peak meridional speeds, m/s: trades, Ferrel, polar (design §3.2).
pub const V_TRADES: f64 = 2.5; // ledger: CLM-140
pub const V_FERREL: f64 = 1.0; // ledger: CLM-140
pub const V_POLAR: f64 = 0.5; // ledger: CLM-140

impl MoistureParams {
    /// The budget of climate section `c` on a planet with this tilt, rotation and radius, on a
    /// grid of `cell_km` Earth kilometres per cell at the equator.
    pub fn from_climate(
        c: &super::ClimateParams,
        tilt_deg: f64,
        omega: f64,
        radius_m: f64,
        cell_km: f64,
    ) -> MoistureParams {
        MoistureParams {
            k_q: 1.5e6,                   // ledger: CLM-139
            tau_p_s: 6.6 * 86_400.0,      // ledger: CLM-145
            rh_slope: c.precip_rh_slope,  // ledger: CLM-145
            r_ref: 0.7,                   // ledger: CLM-145
            c_e: 1.3e-3,                  // ledger: CLM-142
            u_t_sea: 3.0,                 // ledger: CLM-142
            u_t_land: 2.0,                // ledger: CLM-142
            bucket_mm: c.bucket_mm,       // ledger: CLM-143
            kappa_p: c.thermal_wind_pa_k, // ledger: CLM-141
            friction: 1.0e-5,             // ledger: CLM-141
            stab_k: 0.3,                  // ledger: CLM-146
            itcz_max_deg: c.itcz_max_shift_deg,
            itcz_smooth_cells: ((c.itcz_smooth_km / cell_km.max(1.0)).round() as usize).max(1),
            hemi_shift: std::array::from_fn(|m| belts::hemispheric_shift(c, tilt_deg, m)),
            winds: belts::winds(c),
            omega,
            radius_m,
        }
    }
}

/// What the budget needs, per coarse cell (rows north first) and month.
pub struct Input<'a> {
    pub grid: &'a Grid,
    /// Land fraction and the land's lowland height above the sea, m.
    pub land: &'a [f64],
    pub z_land: &'a [f64],
    /// Monthly land and sea tile temperatures, °C, and the sea's ice cover, 0–1 (the energy
    /// balance's last year).
    pub t_land: &'a [[f64; 12]],
    pub t_ocean: &'a [[f64; 12]],
    pub sea_ice: &'a [[f64; 12]],
    /// Mountain rain taken out on L1 (the sink S_oro), kg/m²/s per cell and month; `None`: none.
    pub oro_sink: Option<&'a [[f64; 12]]>,
    /// The bucket to start from, kg/m² per cell (empty: half full).
    pub bucket: &'a [f64],
}

/// The budget's monthly results per cell.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Moisture {
    /// Precipitation, evaporation, kg/m²/s (= mm/s); column water, kg/m²; its relative humidity.
    pub p: Vec<[f64; 12]>,
    pub e: Vec<[f64; 12]>,
    pub w: Vec<[f64; 12]>,
    pub rh: Vec<[f64; 12]>,
    /// The surface wind, m/s (east, north).
    pub u: Vec<[f64; 12]>,
    pub v: Vec<[f64; 12]>,
    /// The ITCZ of every column and month, degrees.
    pub itcz: Vec<[f64; 12]>,
    /// The soil bucket at the start of the next year, kg/m².
    pub bucket: Vec<f64>,
    /// |Σ A E − Σ A P − Σ A S_oro| / Σ A E of every month, and the sweeps each month's last solve
    /// took.
    pub closure: [f64; 12],
    pub sweeps: [u32; 12],
}

/// Saturation specific humidity at `t` °C over water (Tetens, in Murray 1967's form, at 1,013 hPa).
pub fn q_sat(t: f64) -> f64 {
    let es = 6.1078 * (17.27 * t / (t + 237.3)).exp(); // hPa; ledger: CLM-142
    0.622 * es / 1013.25
}

/// The three-cell surface flow at belt latitude φ′ (degrees), m/s north.
pub fn meridional(phi_b: f64) -> f64 {
    let a = phi_b.abs();
    let s = phi_b.signum();
    let pi = std::f64::consts::PI;
    if a < 30.0 {
        -V_TRADES * (pi * a / 30.0).sin() * s
    } else if a < 60.0 {
        V_FERREL * (pi * (a - 30.0) / 30.0).sin() * s
    } else {
        -V_POLAR * (pi * (a - 60.0) / 30.0).sin() * s
    }
}

/// The surface winds of every cell and month, m/s (east, north), and the ITCZ of every column
/// and month, degrees: §3.2's belt, three-cell and thermal winds (shared with `upwind.rs`).
pub type Winds = (Vec<[f64; 12]>, Vec<[f64; 12]>, Vec<[f64; 12]>);

/// [`Winds`] on grid `g` with land fraction `land` and the monthly tiles.
#[allow(clippy::needless_range_loop)]
pub fn winds(
    prm: &MoistureParams,
    g: &Grid,
    land: &[f64],
    t_land: &[[f64; 12]],
    t_ocean: &[[f64; 12]],
) -> Winds {
    let (w, h) = (g.w, g.h);
    let n = w * h;
    let r = prm.radius_m;
    let lat: Vec<f64> = (0..h).map(|j| g.lat_c(j)).collect();
    let f: Vec<f64> = land.iter().map(|x| x.clamp(0.0, 1.0)).collect();
    let tbar = |c: usize, m: usize| f[c] * t_land[c][m] + (1.0 - f[c]) * t_ocean[c][m];
    // ITCZ of every column and month: the soft maximum of T̄ over |φ| ≤ 35°, smoothed along the row.
    let rows35: Vec<usize> = (0..h).filter(|&j| lat[j].abs() <= 35.0).collect();
    let mut itcz = vec![[0.0f64; 12]; w];
    for i in 0..w {
        for m in 0..12 {
            if rows35.is_empty() {
                continue;
            }
            let tmax = rows35
                .iter()
                .map(|&j| tbar(j * w + i, m))
                .fold(f64::MIN, f64::max);
            let (mut num, mut den) = (0.0, 0.0);
            for &j in &rows35 {
                let wt = ((tbar(j * w + i, m) - tmax) / 1.5).exp() * g.sin_span(j);
                num += wt * lat[j];
                den += wt;
            }
            itcz[i][m] = (num / den.max(1e-300)).clamp(-prm.itcz_max_deg, prm.itcz_max_deg);
        }
    }
    let rad = prm.itcz_smooth_cells.min(w / 2);
    for _ in 0..2 {
        let src = itcz.clone();
        for i in 0..w {
            for m in 0..12 {
                let s: f64 = (0..=2 * rad).map(|k| src[(i + w + k - rad) % w][m]).sum();
                itcz[i][m] = s / (2 * rad + 1) as f64;
            }
        }
    }
    // Winds of every cell and month.
    let mut u = vec![[0.0f64; 12]; n];
    let mut v = vec![[0.0f64; 12]; n];
    for m in 0..12 {
        // T̄ departures from the row mean, smoothed (1-2-1, twice, both ways).
        let mut dep: Vec<f64> = (0..n)
            .map(|c| {
                let j = c / w;
                let mean = (0..w).map(|i| tbar(j * w + i, m)).sum::<f64>() / w as f64;
                tbar(c, m) - mean
            })
            .collect();
        for _ in 0..2 {
            let s = dep.clone();
            for c in 0..n {
                let (i, j) = (c % w, c / w);
                let e = j * w + (i + 1) % w;
                let ww = j * w + (i + w - 1) % w;
                dep[c] = 0.25 * s[ww] + 0.5 * s[c] + 0.25 * s[e];
            }
            let s = dep.clone();
            for c in 0..n {
                let j = c / w;
                let nn = if j > 0 { c - w } else { c };
                let ss = if j + 1 < h { c + w } else { c };
                dep[c] = 0.25 * s[nn] + 0.5 * s[c] + 0.25 * s[ss];
            }
        }
        for c in 0..n {
            let (i, j) = (c % w, c / w);
            let phi = lat[j];
            let pb = phi - belts::shift(phi, itcz[i][m], prm.hemi_shift[m]);
            let wt = belts::regime_weights(phi, pb);
            let ub: f64 = (0..belts::REGIMES).map(|k| wt[k] * prm.winds[k].0).sum();
            let vb = meridional(pb);
            // Thermal wind from p′ = −κ_p T̄′.
            let dx = r * phi.to_radians().cos().max(0.05) * g.lon_w[0];
            let e = j * w + (i + 1) % w;
            let ww = j * w + (i + w - 1) % w;
            let px = -prm.kappa_p * (dep[e] - dep[ww]) / (2.0 * dx);
            let (nn, ss) = (
                if j > 0 { c - w } else { c },
                if j + 1 < h { c + w } else { c },
            );
            let dy = r * (lat[nn / w] - lat[ss / w]).to_radians().max(1e-6);
            let py = -prm.kappa_p * (dep[nn] - dep[ss]) / dy;
            let fc = 2.0 * prm.omega * phi.to_radians().sin();
            let rr = prm.friction;
            let den = RHO_AIR * (rr * rr + fc * fc);
            let ut = -(rr * px + fc * py) / den;
            let vt = -(rr * py - fc * px) / den;
            u[c][m] = ub + ut;
            v[c][m] = vb + vt;
        }
    }
    (u, v, itcz)
}

/// Solves the year's twelve monthly budgets, the bucket cycled three times over the year.
#[allow(clippy::needless_range_loop)]
pub fn solve(prm: &MoistureParams, inp: &Input<'_>) -> Moisture {
    let g = inp.grid;
    let (w, h) = (g.w, g.h);
    let n = w * h;
    let r = prm.radius_m;
    let area: Vec<f64> = g.areas().iter().map(|a| a * r * r).collect();
    let f: Vec<f64> = inp.land.iter().map(|x| x.clamp(0.0, 1.0)).collect();
    let tbar = |c: usize, m: usize| f[c] * inp.t_land[c][m] + (1.0 - f[c]) * inp.t_ocean[c][m];
    let (u, v, itcz) = winds(prm, g, inp.land, inp.t_land, inp.t_ocean);
    // Face geometry: east faces (length R Δφ) and north faces (R cos φ_edge Δλ), conductances.
    let len_e: Vec<f64> = (0..h).map(|j| r * g.dphi(j)).collect();
    let len_n: Vec<f64> = (0..=h)
        .map(|e| r * g.lat_edge[e].to_radians().cos().max(0.0) * g.lon_w[0])
        .collect();
    let kd_e: Vec<f64> = (0..n).map(|c| prm.k_q * g.g_ew(c % w, c / w)).collect();
    let kd_s: Vec<f64> = (0..n).map(|c| prm.k_q * g.g_ns(c % w, c / w)).collect();
    let wsat: Vec<[f64; 12]> = (0..n)
        .map(|c| {
            let hf = 1.0 - f[c] + f[c] * (-inp.z_land[c].max(0.0) / W_SAT_SCALE_M).exp();
            std::array::from_fn(|m| W_SAT_27 * (W_SAT_PER_K * (tbar(c, m) - 27.0)).exp() * hf)
        })
        .collect();
    let stab: Vec<[f64; 12]> = (0..n)
        .map(|c| {
            let j = c / w;
            std::array::from_fn(|m| {
                let (mut s, mut a) = (0.0, 0.0);
                for i in 0..w {
                    let cc = j * w + i;
                    s += (1.0 - f[cc]) * inp.t_ocean[cc][m];
                    a += 1.0 - f[cc];
                }
                let mean = if a > 0.0 { s / a } else { inp.t_ocean[c][m] };
                let cold = (prm.stab_k * (inp.t_ocean[c][m] - mean).min(0.0)).exp();
                1.0 - (1.0 - f[c]) * (1.0 - cold)
            })
        })
        .collect();
    let s_max = prm.bucket_mm.max(1e-6);
    let mut bucket: Vec<f64> = if inp.bucket.len() == n {
        inp.bucket.to_vec()
    } else {
        vec![0.5 * s_max; n]
    };
    let mut out = Moisture {
        p: vec![[0.0; 12]; n],
        e: vec![[0.0; 12]; n],
        w: vec![[0.0; 12]; n],
        rh: vec![[0.0; 12]; n],
        u: u.clone(),
        v: v.clone(),
        itcz,
        ..Moisture::default()
    };
    let mut wv: Vec<Vec<f64>> = (0..12)
        .map(|m| (0..n).map(|c| prm.r_ref * wsat[c][m]).collect())
        .collect();
    for pass in 0..3 {
        for m in 0..12 {
            // Per-cell evaporation coefficients: E(W) = e0 (1 − W/W_sat).
            let e0: Vec<f64> = (0..n)
                .map(|c| {
                    let speed = u[c][m].hypot(v[c][m]);
                    let sea = (1.0 - f[c])
                        * (1.0 - inp.sea_ice[c][m].clamp(0.0, 1.0))
                        * RHO_AIR
                        * prm.c_e
                        * (speed + prm.u_t_sea)
                        * q_sat(inp.t_ocean[c][m]);
                    let beta = (bucket[c] / (0.75 * s_max)).min(1.0);
                    let land = f[c]
                        * beta
                        * RHO_AIR
                        * prm.c_e
                        * (speed + prm.u_t_land)
                        * q_sat(inp.t_land[c][m]);
                    sea + land
                })
                .collect();
            let sink: Vec<f64> = (0..n)
                .map(|c| inp.oro_sink.map_or(0.0, |s| s[c][m]))
                .collect();
            // Face velocities (normal components).
            let ue: Vec<f64> = (0..n)
                .map(|c| {
                    let (i, j) = (c % w, c / w);
                    0.5 * (u[c][m] + u[j * w + (i + 1) % w][m])
                })
                .collect();
            // North face of cell c (between c and c − w), northward.
            let vn: Vec<f64> = (0..n)
                .map(|c| {
                    if c >= w {
                        0.5 * (v[c][m] + v[c - w][m])
                    } else {
                        0.0
                    }
                })
                .collect();
            let wm = &mut wv[m];
            let src = |c: usize, x: f64| -> (f64, f64) {
                // (E − P − S)(x) and its derivative in x.
                let ws = wsat[c][m].max(1e-6);
                let rh = x / ws;
                let ex = (prm.rh_slope * (rh - prm.r_ref)).min(60.0).exp();
                let p = stab[c][m] * x / prm.tau_p_s * ex;
                let dp = stab[c][m] / prm.tau_p_s * ex * (1.0 + prm.rh_slope * rh);
                let e = e0[c] * (1.0 - rh);
                let de = -e0[c] / ws;
                (e - p - sink[c], de - dp)
            };
            let mut sweeps = 0u32;
            for _ in 0..4000 {
                sweeps += 1;
                let mut worst = 0.0f64;
                let order: Vec<usize> = if sweeps % 2 == 1 {
                    (0..h).collect()
                } else {
                    (0..h).rev().collect()
                };
                for &j in &order {
                    let r0 = j * w;
                    let mut lo = vec![0.0; w];
                    let mut di = vec![0.0; w];
                    let mut up = vec![0.0; w];
                    let mut res = vec![0.0; w];
                    for i in 0..w {
                        let c = r0 + i;
                        let x = wm[c];
                        // East face (c → e).
                        let e = r0 + (i + 1) % w;
                        let ie = (i + 1) % w;
                        let fu = ue[c] * len_e[j];
                        let (fp, fm) = (fu.max(0.0), fu.min(0.0));
                        let flux = fp * x + fm * wm[e] + kd_e[c] * (x - wm[e]);
                        res[i] += flux;
                        res[ie] -= flux;
                        di[i] += fp + kd_e[c];
                        up[i] += fm - kd_e[c];
                        lo[ie] -= fp + kd_e[c];
                        di[ie] -= fm - kd_e[c];
                        // North face (c → c − w), the north row held.
                        if j > 0 {
                            let nn = c - w;
                            let fv = vn[c] * len_n[j];
                            let (fp, fm) = (fv.max(0.0), fv.min(0.0));
                            let flux = fp * x + fm * wm[nn] + kd_s[nn] * (x - wm[nn]);
                            res[i] += flux;
                            di[i] += fp + kd_s[nn];
                        }
                        // South face (c → c + w), the south row held: the south cell's north face.
                        if j + 1 < h {
                            let ss = c + w;
                            let fv = vn[ss] * len_n[j + 1];
                            // Northward flux into c from ss: positive fv brings wm[ss] in.
                            let (fp, fm) = (fv.max(0.0), fv.min(0.0));
                            let inflow = fp * wm[ss] + fm * x + kd_s[c] * (wm[ss] - x);
                            res[i] -= inflow;
                            di[i] += -fm + kd_s[c];
                        }
                        let (s, ds) = src(c, x);
                        res[i] -= area[c] * s;
                        di[i] -= area[c] * ds;
                    }
                    for i in 0..w {
                        worst = worst.max(res[i].abs() / area[r0 + i]);
                    }
                    let rhs: Vec<f64> = res.iter().map(|x| -x).collect();
                    let dx = super::ebm2d::cyclic_thomas(&lo, &di, &up, &rhs);
                    for i in 0..w {
                        let c = r0 + i;
                        wm[c] = (wm[c] + dx[i]).max(0.0);
                    }
                }
                if worst < 1e-10 {
                    break;
                }
            }
            // The month's results and the bucket.
            let (mut se, mut sp, mut ss) = (0.0, 0.0, 0.0);
            for c in 0..n {
                let x = wm[c];
                let ws = wsat[c][m].max(1e-6);
                let rh = x / ws;
                let p = stab[c][m] * x / prm.tau_p_s
                    * (prm.rh_slope * (rh - prm.r_ref)).min(60.0).exp();
                let e = e0[c] * (1.0 - rh);
                out.p[c][m] = p;
                out.e[c][m] = e;
                out.w[c][m] = x;
                out.rh[c][m] = rh;
                se += area[c] * e;
                sp += area[c] * p;
                ss += area[c] * sink[c];
                if f[c] > 0.0 {
                    let speed = u[c][m].hypot(v[c][m]);
                    let beta = (bucket[c] / (0.75 * s_max)).min(1.0);
                    let el = beta
                        * RHO_AIR
                        * prm.c_e
                        * (speed + prm.u_t_land)
                        * q_sat(inp.t_land[c][m])
                        * (1.0 - rh);
                    bucket[c] = (bucket[c] + (p - el) * MONTH_S).clamp(0.0, s_max);
                }
            }
            out.closure[m] = (se - sp - ss).abs() / se.abs().max(1e-300);
            out.sweeps[m] = sweeps;
            let _ = pass;
        }
    }
    out.bucket = bucket;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::climate::ClimateParams;

    fn grid() -> Grid {
        Grid::new(512, 256, &[1.0e4; 256], 1.0e4, 128)
    }

    fn params() -> MoistureParams {
        MoistureParams::from_climate(
            &ClimateParams::default(),
            23.4,
            7.292e-5,
            crate::climate::ebm2d::EARTH_RADIUS_M,
            313.0,
        )
    }

    /// A zonal seasonal climate: T = 28 − 42 sin² of the latitude less the sun's (± 15° over the
    /// year), on land amplified by `amp`.
    fn temps(g: &Grid, land: &[f64], amp: f64) -> (Vec<[f64; 12]>, Vec<[f64; 12]>) {
        let n = g.w * g.h;
        let sun = |m: usize| 15.0 * ((m as f64 + 0.5) / 12.0 * std::f64::consts::TAU - 1.9).sin();
        let t =
            |lat: f64, m: usize| 28.0 - 42.0 * ((lat - 0.3 * sun(m)).to_radians().sin()).powi(2);
        let tl = (0..n)
            .map(|c| {
                let lat = g.lat_c(c / g.w);
                std::array::from_fn(|m| {
                    let base = t(lat, m);
                    if land[c] > 0.0 {
                        // Summer in the north is months 5–7: land departs ± amp from the sea.
                        base + amp * (sun(m) / 15.0) * lat.signum()
                    } else {
                        base
                    }
                })
            })
            .collect();
        let to = (0..n)
            .map(|c| {
                let lat = g.lat_c(c / g.w);
                std::array::from_fn(|m| t(lat, m))
            })
            .collect();
        (tl, to)
    }

    #[test]
    fn an_aquaplanet_rains_at_the_itcz_dries_the_subtropics_and_closes_its_water() {
        let g = grid();
        let n = g.w * g.h;
        let land = vec![0.0; n];
        let (tl, to) = temps(&g, &land, 0.0);
        let ice = vec![[0.0; 12]; n];
        let z = vec![0.0; n];
        let t0 = std::time::Instant::now();
        let m = solve(
            &params(),
            &Input {
                grid: &g,
                land: &land,
                z_land: &z,
                t_land: &tl,
                t_ocean: &to,
                sea_ice: &ice,
                oro_sink: None,
                bucket: &[],
            },
        );
        let secs = t0.elapsed().as_secs_f64();
        eprintln!(
            "aquaplanet budget: {secs:.2} s, sweeps {:?}, closure {:?}",
            m.sweeps, m.closure
        );
        for mo in 0..12 {
            assert!(
                m.closure[mo] < 1e-4,
                "month {mo}: closure {}",
                m.closure[mo]
            );
        }
        assert!(m.w.iter().all(|x| x.iter().all(|&v| v >= 0.0)));
        // Zonal-annual precipitation by row.
        let zp: Vec<f64> = (0..g.h)
            .map(|j| {
                (0..g.w)
                    .map(|i| m.p[j * g.w + i].iter().sum::<f64>())
                    .sum::<f64>()
                    / (12.0 * g.w as f64)
                    * 86_400.0
            })
            .collect();
        let lat: Vec<f64> = (0..g.h).map(|j| g.lat_c(j)).collect();
        let near = |a: f64, b: f64| {
            (0..g.h)
                .filter(|&j| (a..=b).contains(&lat[j].abs()))
                .map(|j| zp[j])
                .fold(0.0f64, f64::max)
        };
        let low = |a: f64, b: f64| {
            (0..g.h)
                .filter(|&j| (a..=b).contains(&lat[j].abs()))
                .map(|j| zp[j])
                .fold(f64::MAX, f64::min)
        };
        let (eq, sub, mid) = (near(0.0, 10.0), low(20.0, 32.0), near(45.0, 65.0));
        eprintln!("zonal P mm/day: equator {eq:.2}, subtropics {sub:.2}, mid-latitudes {mid:.2}");
        let jmax = (0..g.h).max_by(|a, b| zp[*a].total_cmp(&zp[*b])).unwrap();
        assert!(lat[jmax].abs() < 12.0, "rain peaks at {}", lat[jmax]);
        assert!(eq > 3.0 * sub && mid > 1.5 * sub, "{eq} {sub} {mid}");
        let jmin = (0..g.h)
            .filter(|&j| lat[j].abs() < 45.0)
            .min_by(|a, b| zp[*a].total_cmp(&zp[*b]))
            .unwrap();
        assert!(
            (18.0..=35.0).contains(&lat[jmin].abs()),
            "driest at {}",
            lat[jmin]
        );
        assert!(secs < 5.0, "{secs} s");
    }

    #[test]
    fn a_hot_summer_continent_draws_in_sea_air_and_rains_in_summer() {
        let g = grid();
        let n = g.w * g.h;
        // A continent 60° wide from 10°N to 35°N (monsoon latitudes), 10 K warmer than the sea
        // in summer and colder in winter.
        let land: Vec<f64> = (0..n)
            .map(|c| {
                let lat = g.lat_c(c / g.w);
                let lon = (c % g.w) as f64 * 360.0 / g.w as f64;
                if (10.0..35.0).contains(&lat) && (60.0..120.0).contains(&lon) {
                    1.0
                } else {
                    0.0
                }
            })
            .collect();
        let (tl, to) = temps(&g, &land, 10.0);
        let ice = vec![[0.0; 12]; n];
        let z = vec![0.0; n];
        let m = solve(
            &params(),
            &Input {
                grid: &g,
                land: &land,
                z_land: &z,
                t_land: &tl,
                t_ocean: &to,
                sea_ice: &ice,
                oro_sink: None,
                bucket: &[],
            },
        );
        let on_land: Vec<usize> = (0..n).filter(|&c| land[c] > 0.0).collect();
        let mean =
            |mo: usize| on_land.iter().map(|&c| m.p[c][mo]).sum::<f64>() / on_land.len() as f64;
        let (summer, winter) = (
            (5..8).map(mean).sum::<f64>(),
            [11, 0, 1].iter().map(|&x| mean(x)).sum::<f64>(),
        );
        eprintln!(
            "continent: summer {:.2} winter {:.2} mm/day",
            summer / 3.0 * 86_400.0,
            winter / 3.0 * 86_400.0
        );
        assert!(summer >= 2.0 * winter, "summer {summer} winter {winter}");
        // Onshore at the continent's southern coast in July: the wind blows north into it.
        let j = (0..g.h)
            .min_by(|a, b| {
                (g.lat_c(*a) - 8.0)
                    .abs()
                    .total_cmp(&(g.lat_c(*b) - 8.0).abs())
            })
            .unwrap();
        let cols: Vec<usize> = (0..g.w)
            .filter(|&i| land[(j - 2) * g.w + i] > 0.0)
            .collect();
        let vj: f64 = cols.iter().map(|&i| m.v[j * g.w + i][6]).sum::<f64>() / cols.len() as f64;
        let vj_jan: f64 =
            cols.iter().map(|&i| m.v[j * g.w + i][0]).sum::<f64>() / cols.len() as f64;
        assert!(
            vj > vj_jan + 1.0,
            "July v {vj} vs January {vj_jan} south of the continent"
        );
        for mo in 0..12 {
            assert!(
                m.closure[mo] < 1e-4,
                "month {mo}: closure {}",
                m.closure[mo]
            );
        }
    }
}

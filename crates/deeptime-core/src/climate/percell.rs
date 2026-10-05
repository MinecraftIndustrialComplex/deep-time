// SPDX-License-Identifier: LGPL-3.0-or-later
//! The per-cell climate of a snapshot (P1-26 T5; `docs/design/p1-26-per-cell-climate.md` §2.9,
//! §3.6–3.8): the coarse grid's surface built from L1, the 2-D energy balance (`ebm2d.rs`) with
//! the ocean's transport (`ocean2d.rs`), the moisture budget when `rain_model = budget`
//! (`moisture.rs`), and what `snapshot.rs` needs to bring them down to L1.
//!
//! **Downscaling** (§2.9). A land cell's month is the masked bilinear of the coarse land tiles
//! (only coarse cells that hold land, renormalized), moved toward the masked bilinear of the sea
//! tiles by its sea share within `coast_km`, less the winter inversion of snow-covered, dark,
//! continental interiors, I_max·dark(m)·snow(T_l*)·interior (dark = clamp(1 − Q_m/150 W/m², 0, 1),
//! snow the albedo's snow factor, interior = clamp((f₁₀₀₀ − 0.5)/0.5, 0, 1) of the land share
//! within 1,000 km); a sea cell's is the sea tiles'. A sea cell's ice cover is the masked bilinear
//! of the coarse monthly covers, sharpened by a logistic of slope `sea_ice_edge_sharpness` at
//! half cover. Everything after that — the lapse rate, Köppen, aridity, the equilibrium line —
//! is the snapshot's as before.

use serde::{Deserialize, Serialize};

use super::coarse::Grid;
use super::ebm::{EbmState, smoothstep};
use super::ebm2d::{self, EARTH_RADIUS_M, Params2d, Result2d, State2d, Surface};
use super::moisture::{self, Moisture, MoistureParams};
use super::ocean2d::{self, Ocean, Setup, Sinking};
use super::params::{ClimateParams, RainModel};

/// The insolation below which a month is dark for the winter inversion, W/m² (design §2.9).
pub const DARK_W_M2: f64 = 150.0; // ledger: CLM-134
/// The radius of the continental share that makes an interior, Earth km (design §2.9).
pub const INTERIOR_KM: f64 = 1000.0; // ledger: CLM-134

/// What the per-cell climate carries from snapshot to snapshot (in the stage's saved state).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PerCell {
    /// The 2-D model's state (warm start).
    pub state: State2d,
    /// The ITCZ of each month (the zonal-mean thermal equator) the next run starts from.
    pub itcz: Vec<f64>,
    /// Late-winter sea-ice cover of every coarse cell (March north, September south): where the
    /// conveyor may sink next time.
    pub late_ice: Vec<f64>,
    /// The moisture budget's soil bucket.
    pub bucket: Vec<f64>,
    /// The upwind rain's last annual rain per coarse cell, mm/yr (the land's recycling).
    #[serde(default)]
    pub rain_mm: Vec<f64>,
    /// The last sinking region: (latitude, longitude, hemisphere).
    pub sinking: Option<(f64, f64, i8)>,
}

/// The climate grid of an L1 grid `fw` × `fh`: about `cols` columns, a coarse cell being k × k L1
/// cells with k a divisor of `fw` (so every column is equally wide and the zonal shift conserves).
pub fn climate_grid(fw: usize, fh: usize, cols: usize) -> Grid {
    let mut k = (fw / cols.max(1)).max(1);
    while k > 1 && !fw.is_multiple_of(k) {
        k -= 1;
    }
    let sx = vec![1.0; fh];
    Grid::new(fw, fh, &sx, 1.0, fw / k)
}

/// What the per-cell climate needs from the snapshot.
pub struct Inputs<'a> {
    pub p: &'a ClimateParams,
    /// L1 size, land mask and height above the sea, m.
    pub w: usize,
    pub h: usize,
    pub land: &'a [bool],
    pub z: &'a [f64],
    /// The previous land ice (thickness, m, ≥ 1 is ice) and Köppen classes on L1, if any; else
    /// the zonal model's band shares of ice, south to north.
    pub ice_prev: Option<&'a [f32]>,
    pub koppen_prev: Option<&'a [f32]>,
    pub band_sheet: &'a [f64],
    /// The zonal model's state at this snapshot (the 2-D model's first start).
    pub ebm: &'a EbmState,
    /// The zonal model's parameters at this snapshot (radiation, orbit, capacities).
    pub ep: super::ebm::EbmParams,
    pub tilt_deg: f64,
    pub day_length_h: f64,
}

/// The coarse climate of a snapshot.
pub struct Coarse {
    pub grid: Grid,
    /// Land fraction and lowland height of every coarse cell.
    pub land: Vec<f64>,
    pub z_low: Vec<f64>,
    pub r: Result2d,
    pub sinking: Option<Sinking>,
    pub psi_max_sv: f64,
    pub gyre_sweeps: u32,
    pub moisture: Option<Moisture>,
    /// The upwind rain (`rain_model = upwind`; solved by the snapshot, which knows the basins).
    pub upwind: Option<super::upwind::Rain>,
    /// Monthly land and sea tiles and sea ice, [month][coarse cell].
    tl: Vec<Vec<f64>>,
    to: Vec<Vec<f64>>,
    ice: Vec<Vec<f64>>,
    /// The sea tile's annual mean less its row's mean over the sea, °C.
    pub dsst: Vec<f64>,
    /// The ITCZ of every coarse column and month (the column's thermal equator), degrees.
    pub itcz_col: Vec<[f64; 12]>,
    /// The moisture budget's parameters.
    pub mp: MoistureParams,
    /// The 2-D model's state after the run (the zonal model's start when it is fitted).
    pub state: State2d,
}

impl Coarse {
    fn has_land(&self, cc: usize) -> bool {
        self.land[cc] > 0.0
    }

    fn has_sea(&self, cc: usize) -> bool {
        self.land[cc] < 1.0
    }

    /// The land and sea tiles' temperature of month `m` at L1 cell `c` (masked bilinear; a cell
    /// with no coarse land around it takes the sea's, and the other way round).
    pub fn tiles(&self, c: usize, m: usize) -> (f64, f64) {
        let g = &self.grid;
        let l = g.masked_at(&self.tl[m], &|cc| self.has_land(cc), c);
        let s = g.masked_at(&self.to[m], &|cc| self.has_sea(cc), c);
        match (l, s) {
            (Some(l), Some(s)) => (l, s),
            (Some(l), None) => (l, l),
            (None, Some(s)) => (s, s),
            (None, None) => {
                let cc = g.coarse_of(c);
                (self.tl[m][cc], self.to[m][cc])
            }
        }
    }

    /// The sea ice of month `m` at L1 cell `c` (masked bilinear over the coarse cells with sea),
    /// before sharpening.
    pub fn ice_at(&self, c: usize, m: usize) -> f64 {
        self.grid
            .masked_at(&self.ice[m], &|cc| self.has_sea(cc), c)
            .unwrap_or_else(|| self.ice[m][self.grid.coarse_of(c)])
            .clamp(0.0, 1.0)
    }

    /// The sea's departure from its row's mean at L1 cell `c`, °C.
    pub fn dsst_at(&self, c: usize) -> f64 {
        self.grid
            .masked_at(&self.dsst, &|cc| self.has_sea(cc), c)
            .unwrap_or(0.0)
    }

    /// The ITCZ of every L1 column and month: the coarse columns' linearly between their centres.
    pub fn itcz_l1(&self) -> Vec<[f64; 12]> {
        let g = &self.grid;
        let k = g.k as f64;
        (0..g.fw)
            .map(|i| {
                let x = (i as f64 + 0.5) / k - 0.5;
                let x0 = x.floor();
                let fx = x - x0;
                let i0 = (x0 as i64).rem_euclid(g.w as i64) as usize;
                let i1 = (i0 + 1) % g.w;
                std::array::from_fn(|m| {
                    (1.0 - fx) * self.itcz_col[i0][m] + fx * self.itcz_col[i1][m]
                })
            })
            .collect()
    }

    /// The upwind rain's share of the circulation's rain and its moisture W_v/W_ref in month `m`
    /// at L1 cell `c` (bilinear; (1, 1) without the upwind rain).
    pub fn upwind_at(&self, c: usize, m: usize) -> (f64, f64) {
        let Some(up) = &self.upwind else {
            return (1.0, 1.0);
        };
        let (mut d, mut q) = (0.0, 0.0);
        for (cc, wt) in self.grid.corners(c) {
            let (dd, qq) = up.dry(cc, m);
            d += wt * dd;
            q += wt * qq;
        }
        (d, q)
    }

    /// The coarse rain's precipitation rate (mm/yr) and moisture of month `m` at L1 cell `c`
    /// (bilinear), from the moisture budget (moisture: its relative humidity over `r_ref`) or
    /// the upwind rain (moisture: the vapour over its reference water); `None` under the belts.
    pub fn rain_at(&self, c: usize, m: usize) -> Option<(f64, f64)> {
        let (mut p, mut q) = (0.0, 0.0);
        if let Some(up) = &self.upwind {
            for (cc, wt) in self.grid.corners(c) {
                let (pr, qq) = up.at(cc, m);
                p += wt * pr;
                q += wt * qq;
            }
            return Some((p * YEAR_S, q));
        }
        let mo = self.moisture.as_ref()?;
        for (cc, wt) in self.grid.corners(c) {
            p += wt * mo.p[cc][m];
            q += wt * mo.rh[cc][m];
        }
        Some((p * YEAR_S, q / self.mp.r_ref))
    }
}

const YEAR_S: f64 = 365.25 * 86_400.0;

/// Sharpens a cover by a logistic of slope `k` at one half, keeping 0 and 1 (k ≤ 1: unchanged).
pub fn sharpen(x: f64, k: f64) -> f64 {
    if k <= 1.0 {
        return x.clamp(0.0, 1.0);
    }
    let a = 4.0 * k;
    let l = |v: f64| 1.0 / (1.0 + (-a * (v - 0.5)).exp());
    let (l0, l1) = (l(0.0), l(1.0));
    ((l(x.clamp(0.0, 1.0)) - l0) / (l1 - l0)).clamp(0.0, 1.0)
}

/// The surface of the coarse cells from L1.
fn surface(inp: &Inputs<'_>, g: &Grid) -> (Surface, Vec<f64>) {
    let (w, h) = (inp.w, inp.h);
    let n = g.w * g.h;
    let lf: Vec<f32> = inp.land.iter().map(|&l| f32::from(u8::from(l))).collect();
    let land: Vec<f64> = g
        .coarsen_area(&lf, &|_| true)
        .iter()
        .map(|&x| f64::from(x))
        .collect();
    // Per coarse cell: the land cells' heights (for the lowland height), ice and desert shares.
    let mut zs: Vec<Vec<f64>> = vec![Vec::new(); n];
    let mut ice = vec![(0.0f64, 0.0f64); n];
    let mut des = vec![(0.0f64, 0.0f64); n];
    for c in 0..w * h {
        if !inp.land[c] {
            continue;
        }
        let cc = g.coarse_of(c);
        zs[cc].push(inp.z[c]);
        if let Some(v) = inp.ice_prev {
            ice[cc].0 += f64::from(u8::from(v[c] >= 1.0));
            ice[cc].1 += 1.0;
        }
        if let Some(k) = inp.koppen_prev {
            let code = k[c] as u8;
            if code != super::koppen::SEA {
                des[cc].0 += f64::from(u8::from(super::koppen::group(code) == 'B'));
                des[cc].1 += 1.0;
            }
        }
    }
    let z_low: Vec<f64> = zs
        .into_iter()
        .map(|mut v| {
            if v.is_empty() {
                return 0.0;
            }
            v.sort_by(f64::total_cmp);
            let half = v.len().div_ceil(2);
            v[..half].iter().sum::<f64>() / half as f64
        })
        .collect();
    let bands = inp.band_sheet.len();
    let sheet: Vec<f64> = (0..n)
        .map(|cc| {
            if inp.ice_prev.is_some() {
                if ice[cc].1 > 0.0 {
                    ice[cc].0 / ice[cc].1
                } else {
                    0.0
                }
            } else if bands > 0 {
                let lat = g.lat_c(cc / g.w);
                inp.band_sheet[super::snapshot::band_of(lat, bands)]
            } else {
                0.0
            }
        })
        .map(|s| s * inp.p.ice_sheet_feedback)
        .collect();
    let desert: Vec<f64> = (0..n)
        .map(|cc| {
            if des[cc].1 > 0.0 {
                des[cc].0 / des[cc].1
            } else {
                0.0
            }
        })
        .collect();
    (
        Surface {
            land,
            sheet,
            desert,
        },
        z_low,
    )
}

/// Runs the per-cell climate of a snapshot; `cell` carries its state to the next.
pub fn run(inp: &Inputs<'_>, cell: &mut PerCell) -> Coarse {
    let p = inp.p;
    let g = climate_grid(inp.w, inp.h, p.grid_columns as usize);
    let n = g.w * g.h;
    let (s, z_low) = surface(inp, &g);
    let omega = std::f64::consts::TAU / (inp.day_length_h * 3600.0) * (1.0 + 1.0 / 365.25);
    let wind_u = ocean2d::belt_wind_u(p, &g, inp.tilt_deg);
    let late = (cell.late_ice.len() == n).then_some(cell.late_ice.as_slice());
    let ocean = Ocean::build(&Setup {
        grid: &g,
        land: &s.land,
        wind_u: &wind_u,
        omega,
        radius_m: EARTH_RADIUS_M,
        slab_m: p.ocean_depth_m,
        gyre_efficiency: p.gyre_heat_efficiency,
        overturning_sv: p.overturning_sv,
        late_winter_ice: late,
    });
    let prm = Params2d::from_climate(p, inp.ep.clone(), inp.tilt_deg);
    // A warm start continues the last snapshot's state, unless the zonal model's climate has
    // since moved far from it (a snowball's onset or escape, a hothouse): then it starts again
    // from the zonal state, as at the first snapshot.
    let warm = cell.state.t_land.len() == n && !departed(&g, &s.land, &cell.state, inp.ebm);
    if !warm {
        cell.state = State2d::from_bands(&g, inp.ebm);
    }
    let itcz0: [f64; 12] = if cell.itcz.len() == 12 {
        std::array::from_fn(|m| cell.itcz[m])
    } else {
        [0.0; 12]
    };
    let years = if warm {
        p.ebm_warm_years
    } else {
        p.ebm_spinup_years
    };
    let r = ebm2d::run(&prm, &g, &s, &mut cell.state, Some(&ocean), years, itcz0);
    cell.itcz = r.itcz.to_vec();
    cell.late_ice = (0..n)
        .map(|cc| {
            let m = if g.lat_c(cc / g.w) >= 0.0 { 2 } else { 8 };
            r.sea_ice[cc][m]
        })
        .collect();
    let tl: Vec<Vec<f64>> = (0..12)
        .map(|m| (0..n).map(|cc| r.t_land[cc][m]).collect())
        .collect();
    let to: Vec<Vec<f64>> = (0..12)
        .map(|m| (0..n).map(|cc| r.t_ocean[cc][m]).collect())
        .collect();
    let ice: Vec<Vec<f64>> = (0..12)
        .map(|m| (0..n).map(|cc| r.sea_ice[cc][m]).collect())
        .collect();
    // The sea tile's annual departure from its row's sea mean.
    let annual_o: Vec<f64> = (0..n)
        .map(|cc| r.t_ocean[cc].iter().sum::<f64>() / 12.0)
        .collect();
    let dsst: Vec<f64> = (0..n)
        .map(|cc| {
            let j = cc / g.w;
            let (mut s_, mut a) = (0.0, 0.0);
            for i in 0..g.w {
                let k = j * g.w + i;
                let sea = 1.0 - s.land[k];
                s_ += sea * annual_o[k];
                a += sea;
            }
            if a > 1e-9 && s.land[cc] < 1.0 {
                annual_o[cc] - s_ / a
            } else {
                0.0
            }
        })
        .collect();
    // The ITCZ of every column (the moisture budget's rule; its smoothing in coarse cells).
    let mp = MoistureParams::from_climate(
        p,
        inp.tilt_deg,
        omega,
        EARTH_RADIUS_M,
        EARTH_RADIUS_M * g.lon_w[0] / 1000.0,
    );
    let itcz_col = column_itcz(&g, &s.land, &r, &mp);
    let mut out = Coarse {
        grid: g,
        land: s.land,
        z_low,
        psi_max_sv: ocean.psi_max_sv(),
        gyre_sweeps: ocean.sweeps,
        sinking: ocean.sinking.clone(),
        r,
        moisture: None,
        upwind: None,
        tl,
        to,
        ice,
        dsst,
        itcz_col,
        mp,
        state: cell.state.clone(),
    };
    if p.rain_model == RainModel::Budget {
        let m = budget(&out, &out.mp, None, &cell.bucket);
        out.moisture = Some(m);
    }
    cell.sinking = out.sinking.as_ref().map(|s| (s.lat, s.lon, s.hemisphere));
    out
}

/// Whether the zonal model's state `z` has moved away from the 2-D state `st`: their sea-surface
/// temperature over the sea differs by more than 3 K, or their sea-ice cover by more than 0.15
/// of the sea (the design's thresholds for a triggered snapshot, §4.2).
#[allow(clippy::needless_range_loop)]
fn departed(g: &Grid, land: &[f64], st: &State2d, z: &EbmState) -> bool {
    let nb = z.t_ocean.len();
    if nb == 0 {
        return false;
    }
    let (mut dt, mut di, mut a) = (0.0, 0.0, 0.0);
    for cc in 0..g.w * g.h {
        let j = cc / g.w;
        let wgt = g.area(cc % g.w, j) * (1.0 - land[cc]);
        if wgt <= 0.0 {
            continue;
        }
        let b = super::snapshot::band_of(g.lat_c(j), nb);
        let cover = |e: f64| (e / super::ebm::ICE_J_M3 / super::ebm::FULL_ICE_M).min(1.0);
        dt += wgt * (st.t_ocean[cc] - z.t_ocean[b]);
        di += wgt * (cover(st.ice_j_m2[cc]) - cover(z.ice_j_m2[b]));
        a += wgt;
    }
    a > 0.0 && ((dt / a).abs() > TRIGGER_K || (di / a).abs() > TRIGGER_ICE)
}

/// The departure of the zonal climate from the 2-D one that restarts the 2-D model, K and share
/// of the sea under ice (design §4.2's triggered-snapshot thresholds).
pub const TRIGGER_K: f64 = 3.0; // ledger: CLM-137
pub const TRIGGER_ICE: f64 = 0.15; // ledger: CLM-137

/// The zonal model's correction to the per-cell climate (design §4.1, P1-26 T6): the
/// atmosphere's diffusivity of every face between its bands, fitted to the 2-D model's poleward
/// heat transport, and a flux of every band's land and sea tile for what is left. The clock runs
/// with it until the next snapshot.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Correction {
    /// W/m²/K, bands + 1 faces (empty: the zonal model's own `d`).
    pub d_face: Vec<f64>,
    /// [land, sea] W/m² of every band (empty: none).
    pub flux: Vec<[f64; 2]>,
}

impl Correction {
    /// The zonal model `ep` with this correction (a correction for another band count is none).
    pub fn apply(&self, ep: &mut super::ebm::EbmParams) {
        ep.flux_corr = if self.flux.len() == ep.bands {
            self.flux.clone()
        } else {
            Vec::new()
        };
        ep.d_face = if self.d_face.len() == ep.bands + 1 {
            self.d_face.clone()
        } else {
            Vec::new()
        };
    }
}

/// The 2-D model's zonal annual mean of `value` weighted by `weight` at the latitudes `lat` of
/// the zonal model's bands: each coarse row's weighted mean, linearly between the row centres
/// that have weight (the coarse rows are wider than the bands; a stepwise remap would hand the
/// smooth zonal model steps it cannot follow). Beyond the last row with weight, that row's
/// value inside its span, `None` outside.
pub fn zonal_mean(
    c: &Coarse,
    lat: &[f64],
    value: &dyn Fn(usize) -> f64,
    weight: &dyn Fn(usize) -> f64,
) -> Vec<Option<f64>> {
    let g = &c.grid;
    // (centre, south edge, north edge, mean) of the rows with weight, south to north.
    let mut rows: Vec<(f64, f64, f64, f64)> = (0..g.h)
        .filter_map(|j| {
            let (mut s, mut a) = (0.0, 0.0);
            for i in 0..g.w {
                let cc = j * g.w + i;
                let wt = weight(cc) * g.area(i, j);
                s += wt * value(cc);
                a += wt;
            }
            (a > 1e-12).then(|| (g.lat_c(j), g.lat_edge[j + 1], g.lat_edge[j], s / a))
        })
        .collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    lat.iter()
        .map(|&l| {
            let k = rows.partition_point(|r| r.0 <= l);
            match (k.checked_sub(1).map(|i| rows[i]), rows.get(k)) {
                (Some(a), Some(b)) => {
                    let f = (l - a.0) / (b.0 - a.0).max(1e-12);
                    Some(a.3 + f * (b.3 - a.3))
                }
                (Some(a), None) => (l <= a.2).then_some(a.3),
                (None, Some(b)) => (l >= b.1).then_some(b.3),
                (None, None) => None,
            }
        })
        .collect()
}

/// The 2-D model's zonal annual means of the land and the sea tiles over every band (land
/// weighted by f, sea by 1 − f; `zonal_mean`).
pub fn zonal_tiles(c: &Coarse, lat: &[f64]) -> Vec<[Option<f64>; 2]> {
    let ann = |v: &[f64; 12]| v.iter().sum::<f64>() / 12.0;
    let l = zonal_mean(c, lat, &|cc| ann(&c.r.t_land[cc]), &|cc| c.land[cc]);
    let o = zonal_mean(c, lat, &|cc| ann(&c.r.t_ocean[cc]), &|cc| 1.0 - c.land[cc]);
    l.into_iter().zip(o).map(|(a, b)| [a, b]).collect()
}

/// The fitted diffusivity's bounds, W/m²/K, and the smallest meridional gradient it is fitted
/// across, K per radian (flatter: from the neighbouring faces).
pub const D_FIT_MIN: f64 = 0.05; // ledger: CLM-179
pub const D_FIT_MAX: f64 = 5.0; // ledger: CLM-179
const D_FIT_GRADIENT_K_RAD: f64 = 3.0; // ledger: CLM-179

/// The diffusivity of every face of the zonal model `ep` (bands + 1, south to north) that
/// carries the 2-D climate's annual poleward heat transport (atmosphere and ocean) down the 2-D
/// zonal-mean gradient of T̄, less what the zonal model's own slab diffusion carries at the 2-D
/// slab temperatures: D = (H_2D − H_slab) / (2π R² cos φ · ΔT̄/Δφ), bounded, smoothed 1-2-1.
pub fn fit_transport(c: &Coarse, ep: &super::ebm::EbmParams, land: &[f64]) -> Vec<f64> {
    let nb = ep.bands;
    let g = &c.grid;
    let lat: Vec<f64> = (0..nb)
        .map(|b| -90.0 + (b as f64 + 0.5) * 180.0 / nb as f64)
        .collect();
    let ann = |v: &[f64; 12]| v.iter().sum::<f64>() / 12.0;
    let tbar = zonal_mean(
        c,
        &lat,
        &|cc| c.land[cc] * ann(&c.r.t_land[cc]) + (1.0 - c.land[cc]) * ann(&c.r.t_ocean[cc]),
        &|_| 1.0,
    );
    let tm = zonal_mean(c, &lat, &|cc| ann(&c.r.t_mixed[cc]), &|cc| 1.0 - c.land[cc]);
    // The 2-D transport across its row edges (north first), W, by latitude.
    let edges: Vec<(f64, f64)> = (0..g.h.saturating_sub(1))
        .map(|e| {
            let pw = c.r.heat_atm_pw[e] + c.r.heat_ocean_diff_pw[e] + c.r.heat_ocean_pw[e];
            (g.lat_edge[e + 1], pw * 1.0e15)
        })
        .collect();
    let h_at = |phi: f64| -> f64 {
        // Linear between edges (descending latitude); zero at the poles.
        let mut prev = (90.0, 0.0);
        for &(l, h) in edges.iter().chain(std::iter::once(&(-90.0, 0.0))) {
            if phi >= l {
                let f = (phi - l) / (prev.0 - l).max(1e-12);
                return h + f * (prev.1 - h);
            }
            prev = (l, h);
        }
        0.0
    };
    let dphi = std::f64::consts::PI / nb as f64;
    let r2pi = EARTH_RADIUS_M * EARTH_RADIUS_M * std::f64::consts::TAU;
    let mut d: Vec<Option<f64>> = vec![None; nb + 1];
    for k in 1..nb {
        let phi = -90.0 + k as f64 * 180.0 / nb as f64;
        let (Some(ts), Some(tn)) = (tbar[k - 1], tbar[k]) else {
            continue;
        };
        let grad = (ts - tn) / dphi;
        if grad.abs() < D_FIT_GRADIENT_K_RAD {
            continue;
        }
        let cosf = phi.to_radians().cos();
        let sea = (1.0 - land[k - 1]).min(1.0 - land[k]).max(0.0);
        let slab = match (tm[k - 1], tm[k]) {
            (Some(a), Some(b)) => r2pi * cosf * ep.d_ocean * sea * (a - b) / dphi,
            _ => 0.0,
        };
        let x = (h_at(phi) - slab) / (r2pi * cosf * grad);
        if x.is_finite() {
            d[k] = Some(x.clamp(D_FIT_MIN, D_FIT_MAX));
        }
    }
    // Faces without a fit: linearly between the fitted ones (the zonal model's `d` if none).
    let known: Vec<usize> = (0..=nb).filter(|&k| d[k].is_some()).collect();
    let filled: Vec<f64> = (0..=nb)
        .map(|k| {
            if let Some(x) = d[k] {
                return x;
            }
            let lo = known.iter().rev().find(|&&q| q < k);
            let hi = known.iter().find(|&&q| q > k);
            match (lo, hi) {
                (Some(&a), Some(&b)) => {
                    let f = (k - a) as f64 / (b - a) as f64;
                    d[a].unwrap() + f * (d[b].unwrap() - d[a].unwrap())
                }
                (Some(&a), None) => d[a].unwrap(),
                (None, Some(&b)) => d[b].unwrap(),
                (None, None) => ep.d,
            }
        })
        .collect();
    smooth121(&smooth121(&filled))
}

/// The flux correction's fit (design §4.1) and how well it matched.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fit {
    pub corr: Correction,
    pub iterations: u32,
    /// After the fit: the largest |T_2D − T_1D| over the band tiles holding at least a tenth of
    /// their band, and the RMS over all tiles (weighted by area), K.
    pub worst_k: f64,
    pub rms_k: f64,
    /// The RMS miss of the first trial (the correction it started from), K.
    pub first_rms_k: f64,
    /// Band tiles held at the cap.
    pub capped: usize,
}

/// The largest flux correction, W/m² (design §4.1's guard, raised from its 15 W/m²: the zonal
/// model's land and sea tiles are tied by a 10 W/m²/K exchange, so the 2-D model's land–sea
/// contrast in a band that is mostly sea takes 30–60 W/m² on the minor tile; build log).
pub const FLUX_CORR_CAP_W_M2: f64 = 60.0; // ledger: CLM-179
/// The fit's gain against the radiation's B (below 1: the ice-albedo feedback can make the
/// zonal model more sensitive than 1/B), its most iterations and the years each runs, and when
/// it is done (RMS and largest miss, K).
const FIT_GAIN: f64 = 0.8; // ledger: CLM-179
const FIT_ITERATIONS: u32 = 30; // ledger: CLM-179
const FIT_YEARS: u32 = 3; // ledger: CLM-179
const FIT_RMS_K: f64 = 0.1; // ledger: CLM-179
const FIT_WORST_K: f64 = 0.3; // ledger: CLM-179

/// Smooths a band profile 1-2-1 (the ends keep their weight).
fn smooth121(v: &[f64]) -> Vec<f64> {
    let n = v.len();
    (0..n)
        .map(|b| {
            let a = v[b.saturating_sub(1)];
            let c = v[(b + 1).min(n - 1)];
            0.25 * a + 0.5 * v[b] + 0.25 * c
        })
        .collect()
}

/// Fits the zonal model's correction to the 2-D climate `c`: its faces' diffusivity
/// (`fit_transport`), then F ← F + gain·B·(T_2D − T_1D) for each band's land and sea tile
/// (annual means; the misses smoothed 1-2-1 over the bands, a tile under a tenth of its band
/// taking the other tile's), each trial `FIT_YEARS` warm-started years of the zonal model `ep`
/// (its `flux_corr` the first guess) on `land` and `sheet` from `ebm`, which is left at the
/// fitted state. Returns the fit and the zonal model's last year.
pub fn fit_flux(
    c: &Coarse,
    ep: &super::ebm::EbmParams,
    land: &[f64],
    sheet: &[f64],
    ebm: &mut EbmState,
) -> (Fit, super::ebm::EbmResult) {
    let nb = ep.bands;
    let lat: Vec<f64> = (0..nb)
        .map(|b| -90.0 + (b as f64 + 0.5) * 180.0 / nb as f64)
        .collect();
    let target = zonal_tiles(c, &lat);
    // The zonal model starts from the 2-D model's zonal state, so it is in the same regime (a
    // snowball the 2-D model froze into, or thawed out of) whatever the clock did since.
    let n = c.grid.w * c.grid.h;
    if c.state.t_land.len() == n {
        let st = &c.state;
        let sea = |cc: usize| 1.0 - c.land[cc];
        let l = zonal_mean(c, &lat, &|cc| st.t_land[cc], &|cc| c.land[cc]);
        let o = zonal_mean(c, &lat, &|cc| st.t_ocean[cc], &sea);
        let m = zonal_mean(c, &lat, &|cc| st.t_mixed[cc], &sea);
        let i = zonal_mean(c, &lat, &|cc| st.ice_j_m2[cc], &sea);
        let all = zonal_mean(c, &lat, &|cc| st.t_ocean[cc], &|_| 1.0);
        *ebm = EbmState {
            t_land: (0..nb)
                .map(|b| l[b].or(o[b]).or(all[b]).unwrap_or(0.0))
                .collect(),
            t_ocean: (0..nb)
                .map(|b| o[b].or(l[b]).or(all[b]).unwrap_or(0.0))
                .collect(),
            t_mixed: (0..nb)
                .map(|b| {
                    m[b].or(o[b])
                        .or(all[b])
                        .unwrap_or(0.0)
                        .max(super::ebm::SEA_FREEZE_C)
                })
                .collect(),
            ice_j_m2: (0..nb).map(|b| i[b].unwrap_or(0.0).max(0.0)).collect(),
        };
    }
    let mut ep = ep.clone();
    ep.d_face = fit_transport(c, &ep, land);
    if ep.flux_corr.len() != nb {
        ep.flux_corr = vec![[0.0; 2]; nb];
    }
    let cap = FLUX_CORR_CAP_W_M2;
    let gain = FIT_GAIN * ep.b;
    let share = |b: usize| -> [f64; 2] {
        let f = land.get(b).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        [f, 1.0 - f]
    };
    let mut fit = Fit::default();
    let mut it = 0;
    loop {
        let er = ep.run(ebm, land, sheet, FIT_YEARS);
        it += 1;
        let (mut worst, mut ss, mut wsum) = (0.0f64, 0.0, 0.0);
        let mut miss = vec![[f64::NAN; 2]; nb];
        for b in 0..nb {
            let w = lat[b].to_radians().cos();
            let t1 = [
                er.t_land[b].iter().sum::<f64>() / 12.0,
                er.t_ocean[b].iter().sum::<f64>() / 12.0,
            ];
            let sh = share(b);
            for k in 0..2 {
                if let Some(t2) = target[b][k]
                    && sh[k] > 0.0
                {
                    let m = t2 - t1[k];
                    miss[b][k] = m;
                    ss += w * sh[k] * m * m;
                    wsum += w * sh[k];
                    if sh[k] >= 0.1 {
                        worst = worst.max(m.abs());
                    }
                }
            }
        }
        fit.worst_k = worst;
        fit.rms_k = if wsum > 0.0 { (ss / wsum).sqrt() } else { 0.0 };
        if it == 1 {
            fit.first_rms_k = fit.rms_k;
        }
        if (fit.rms_k < FIT_RMS_K && worst < FIT_WORST_K) || it > FIT_ITERATIONS {
            fit.iterations = it;
            fit.capped = ep
                .flux_corr
                .iter()
                .flatten()
                .filter(|x| x.abs() >= cap - 1e-9)
                .count();
            fit.corr = Correction {
                d_face: ep.d_face.clone(),
                flux: ep.flux_corr.clone(),
            };
            return (fit, er);
        }
        // A minor tile (under a tenth of its band) or a missing one takes the other's miss; a
        // band with neither takes its neighbours' (through the smoothing, from zero).
        for (b, m) in miss.iter_mut().enumerate() {
            let sh = share(b);
            for k in 0..2 {
                if m[k].is_nan() || sh[k] < 0.1 {
                    m[k] = m[1 - k];
                }
            }
            for x in m.iter_mut() {
                if x.is_nan() {
                    *x = 0.0;
                }
            }
        }
        let mut ml: Vec<f64> = miss.iter().map(|m| m[0]).collect();
        let mut mo: Vec<f64> = miss.iter().map(|m| m[1]).collect();
        ml = smooth121(&ml);
        mo = smooth121(&mo);
        for (b, fc) in ep.flux_corr.iter_mut().enumerate() {
            fc[0] = (fc[0] + gain * ml[b]).clamp(-cap, cap);
            fc[1] = (fc[1] + gain * mo[b]).clamp(-cap, cap);
        }
    }
}

/// The moisture budget on the snapshot's coarse climate, with mountain rain taken out (`sink`,
/// kg/m²/s per coarse cell and month) when given.
pub fn budget(
    c: &Coarse,
    mp: &MoistureParams,
    sink: Option<&[[f64; 12]]>,
    bucket: &[f64],
) -> Moisture {
    moisture::solve(
        mp,
        &moisture::Input {
            grid: &c.grid,
            land: &c.land,
            z_land: &c.z_low,
            t_land: &c.r.t_land,
            t_ocean: &c.r.t_ocean,
            sea_ice: &c.r.sea_ice,
            oro_sink: sink,
            bucket,
        },
    )
}

/// The ITCZ of every coarse column and month: the soft maximum of the column's T̄ over |φ| ≤ 35°,
/// smoothed along the row (the moisture budget's rule).
fn column_itcz(g: &Grid, land: &[f64], r: &Result2d, mp: &MoistureParams) -> Vec<[f64; 12]> {
    let w = g.w;
    let rows: Vec<usize> = (0..g.h).filter(|&j| g.lat_c(j).abs() <= 35.0).collect();
    let tbar =
        |cc: usize, m: usize| land[cc] * r.t_land[cc][m] + (1.0 - land[cc]) * r.t_ocean[cc][m];
    let mut it: Vec<[f64; 12]> = (0..w)
        .map(|i| {
            std::array::from_fn(|m| {
                if rows.is_empty() {
                    return 0.0;
                }
                let tmax = rows
                    .iter()
                    .map(|&j| tbar(j * w + i, m))
                    .fold(f64::MIN, f64::max);
                let (mut num, mut den) = (0.0, 0.0);
                for &j in &rows {
                    let wt = ((tbar(j * w + i, m) - tmax) / 1.5).exp() * g.sin_span(j);
                    num += wt * g.lat_c(j);
                    den += wt;
                }
                (num / den.max(1e-300)).clamp(-mp.itcz_max_deg, mp.itcz_max_deg)
            })
        })
        .collect();
    let rad = mp.itcz_smooth_cells.min(w / 2);
    for _ in 0..2 {
        let src = it.clone();
        for (i, o) in it.iter_mut().enumerate() {
            for m in 0..12 {
                let s: f64 = (0..=2 * rad).map(|k| src[(i + w + k - rad) % w][m]).sum();
                o[m] = s / (2 * rad + 1) as f64;
            }
        }
    }
    it
}

/// The winter inversion of month `m`, °C, for a land cell with land tile `t_land` (°C), the
/// month's insolation `q` (W/m²) and land share `f1000` within 1,000 km.
pub fn inversion(p: &ClimateParams, t_land: f64, q: f64, f1000: f64) -> f64 {
    let dark = (1.0 - q / DARK_W_M2).clamp(0.0, 1.0);
    let snow = 1.0 - smoothstep(p.ice_temp_c - 3.0, p.ice_temp_c + 3.0, t_land);
    let interior = ((f1000 - 0.5) / 0.5).clamp(0.0, 1.0);
    p.winter_inversion_c * dark * snow * interior
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::climate::ebm::EbmParams;

    /// A synthetic L1 planet `w` × w/2 (rows from the north) with land where `is_land(lat, lon)`,
    /// 100 m high: its land and heights, the zonal model's band land fractions, and the zonal
    /// model of climate section `p` at CO₂ `ppm` and star flux `sun`.
    fn planet(
        p: &ClimateParams,
        w: usize,
        is_land: &dyn Fn(f64, f64) -> bool,
        ppm: f64,
        sun: f64,
    ) -> (Vec<bool>, Vec<f64>, Vec<f64>, EbmParams) {
        let h = w / 2;
        // The map's rows (Lambert's: uniform in sin φ), as the climate grid reads them.
        let lat = |j: usize| deeptime_format::Projection::picture_lat_deg(j as f64 + 0.5, h as u32);
        let land: Vec<bool> = (0..w * h)
            .map(|c| is_land(lat(c / w), (c % w) as f64 * 360.0 / w as f64))
            .collect();
        let z: Vec<f64> = land.iter().map(|&l| if l { 100.0 } else { 0.0 }).collect();
        let mut ep = EbmParams::from_climate(p, 23.4, sun);
        ep.co2 = Some(super::super::co2::Forcing::new(ppm));
        let lats: Vec<f64> = (0..h).map(lat).collect();
        let bands = super::super::snapshot::band_means(&lats, w, ep.bands, &|c| {
            f64::from(u8::from(land[c]))
        });
        (land, z, bands, ep)
    }

    fn inputs<'a>(
        p: &'a ClimateParams,
        w: usize,
        land: &'a [bool],
        z: &'a [f64],
        ebm: &'a EbmState,
        ep: &EbmParams,
    ) -> Inputs<'a> {
        Inputs {
            p,
            w,
            h: w / 2,
            land,
            z,
            ice_prev: None,
            koppen_prev: None,
            band_sheet: &[],
            ebm,
            ep: ep.clone(),
            tilt_deg: 23.4,
            day_length_h: 24.0,
        }
    }

    /// Two continents and a polar one: an Earth-like distribution.
    fn earthlike(lat: f64, lon: f64) -> bool {
        ((0.0..140.0).contains(&lon) && (-35.0..72.0).contains(&lat))
            || ((230.0..300.0).contains(&lon) && (-55.0..70.0).contains(&lat))
            || lat < -70.0
    }

    #[test]
    fn the_flux_correction_brings_the_zonal_model_onto_the_2d_zonal_means() {
        let p = ClimateParams {
            grid_columns: 64,
            ..ClimateParams::default()
        };
        let (land, z, bands, ep) = planet(&p, 256, &earthlike, 280.0, 1.0);
        let mut ebm = ep.initial_state();
        let e0 = ep.run(&mut ebm, &bands, &[], p.ebm_spinup_years);
        let mut cell = PerCell::default();
        let c = run(&inputs(&p, 256, &land, &z, &ebm, &ep), &mut cell);
        // Uncorrected, the zonal model sits away from the 2-D model's zonal means.
        let lat: Vec<f64> = (0..ep.bands)
            .map(|b| -90.0 + (b as f64 + 0.5) * 180.0 / ep.bands as f64)
            .collect();
        let tgt = zonal_tiles(&c, &lat);
        let before = (0..ep.bands)
            .filter_map(|b| tgt[b][1].map(|t| (t - e0.t_ocean[b].iter().sum::<f64>() / 12.0).abs()))
            .fold(0.0f64, f64::max);
        let (fit, er) = fit_flux(&c, &ep, &bands, &[], &mut ebm);
        eprintln!(
            "flux fit: {} iterations, worst {:.3} K (before {:.2} K), rms {:.3} K, F {:.1}..{:.1} W/m², capped {}, T 1-D {:.2} 2-D {:.2}",
            fit.iterations,
            fit.worst_k,
            before,
            fit.rms_k,
            fit.corr
                .flux
                .iter()
                .flatten()
                .cloned()
                .fold(f64::MAX, f64::min),
            fit.corr
                .flux
                .iter()
                .flatten()
                .cloned()
                .fold(f64::MIN, f64::max),
            fit.capped,
            er.t_global,
            c.r.t_global
        );
        assert!(
            before > 0.3,
            "the uncorrected models already agree ({before} K)"
        );
        // Every band within 3 K and 0.35 K RMS: the design's 0.3 K in every band is met except at
        // the ice edge, where a band's snowy tile has two states, and under the equal-area grid's
        // tall polar rows, which give the zonal bands a coarse target (build log).
        assert!(fit.worst_k < 3.0 && fit.rms_k < 0.35, "{fit:?}");
        assert!(
            fit.corr
                .flux
                .iter()
                .flatten()
                .all(|x| x.abs() <= FLUX_CORR_CAP_W_M2)
        );
        assert!(
            (er.t_global - c.r.t_global).abs() < 0.2,
            "global means {} and {}",
            er.t_global,
            c.r.t_global
        );
        // A second fit from the first one's correction lands on it again.
        let mut ep2 = ep.clone();
        fit.corr.apply(&mut ep2);
        let (again, _) = fit_flux(&c, &ep2, &bands, &[], &mut ebm);
        assert!(again.worst_k < 3.0 && again.rms_k < 0.35, "{again:?}");
    }

    /// The snowball's onset and escape on a tropical supercontinent under a faint sun (design
    /// §4.2, T6): CO₂ lowered and then raised in steps of × 1.25, each step continuing the last
    /// one's state, in the 2-D model and in the zonal model corrected to it. Each step is a
    /// snapshot: the zonal model steps on its correction (its own runaway decides its snowball,
    /// as the clock's tracker does), then is refitted to the 2-D model, from the 2-D model's
    /// zonal state. The corrected zonal model, which decides the clock's snowballs, freezes and
    /// thaws within a doubling of CO₂ (three steps) of where the 2-D model does.
    #[test]
    fn the_corrected_zonal_model_freezes_and_thaws_where_the_2d_model_does() {
        let p = ClimateParams {
            grid_columns: 32,
            ..ClimateParams::default()
        };
        let sun = 0.94;
        let tropical = |lat: f64, lon: f64| {
            let share = if lat.abs() < 25.0 { 0.8 } else { 0.03 };
            (lon / 360.0 * 32.0).fract() < share
        };
        let w = 128;
        let ice_of = |c: &Coarse| -> f64 {
            let (mut s, mut a) = (0.0, 0.0);
            for cc in 0..c.grid.w * c.grid.h {
                let wt = c.grid.area(cc % c.grid.w, cc / c.grid.w) * (1.0 - c.land[cc]);
                s += wt * c.r.sea_ice[cc].iter().sum::<f64>() / 12.0;
                a += wt;
            }
            s / a
        };
        let warm_ppm = 2000.0;
        let (land, z, bands, ep) = planet(&p, w, &tropical, warm_ppm, sun);
        let mut e1 = ep.initial_state();
        ep.run(&mut e1, &bands, &[], p.ebm_spinup_years);
        // Each step runs both models to their equilibrium (a snowball deepens over centuries).
        let q = ClimateParams {
            ebm_warm_years: 400,
            ..p.clone()
        };
        let at = |ppm: f64| {
            let mut e = ep.clone();
            e.co2 = Some(super::super::co2::Forcing::new(ppm));
            e
        };
        let none = EbmState::default();
        let mut cell = PerCell::default();
        let c = run(&inputs(&p, w, &land, &z, &e1, &ep), &mut cell);
        let (mut fit, _) = fit_flux(&c, &ep, &bands, &[], &mut e1);
        let warm_cell = cell.clone();
        // One step of both models at `ppm`: the 2-D model's sea ice, the zonal model's (its own
        // step decides its snowball, as the clock's tracker does) and its state then; a refit
        // afterwards moves the zonal state to the 2-D model's.
        let step = |ppm: f64,
                    cell: &mut PerCell,
                    e1: &mut EbmState,
                    fit: &mut Fit|
         -> (f64, f64, EbmState) {
            let c2 = run(&inputs(&q, w, &land, &z, &none, &at(ppm)), cell);
            let mut e = at(ppm);
            fit.corr.apply(&mut e);
            let r1 = e.run(e1, &bands, &[], 400);
            let i1 = super::super::clock::sea_ice_frac(&r1, &bands);
            let out = (ice_of(&c2), i1, e1.clone());
            let (f, _) = fit_flux(&c2, &e, &bands, &[], e1);
            *fit = f;
            out
        };
        let (mut on2, mut on1, mut ppm) = (None, None, warm_ppm);
        for k in 0..40 {
            ppm = warm_ppm / 1.25f64.powi(k);
            let (i2, i1, st) = step(ppm, &mut cell, &mut e1, &mut fit);
            if on2.is_none() && i2 >= p.snowball_on_frac {
                on2 = Some(ppm);
            }
            if on1.is_none() && i1 >= p.snowball_on_frac {
                on1 = Some((ppm, st));
            }
            // Six more steps down once both froze: a hard snowball before the CO₂ rises.
            if on1.is_some() && on2.is_some() {
                for _ in 0..6 {
                    ppm /= 1.25;
                    step(ppm, &mut cell, &mut e1, &mut fit);
                }
                break;
            }
        }
        let cold_cell = cell.clone();
        let (mut esc1, mut esc2) = (None, None);
        for k in 1..80 {
            let x = ppm * 1.25f64.powi(k);
            let (i2, i1, st) = step(x, &mut cell, &mut e1, &mut fit);
            if esc2.is_none() && i2 < p.snowball_off_frac {
                esc2 = Some(x);
            }
            if esc1.is_none() && i1 < p.snowball_off_frac {
                esc1 = Some((x, st));
            }
            if esc1.is_some() && esc2.is_some() {
                break;
            }
        }
        let esc2 = esc2.expect("2-D thawed");
        let on2 = on2.expect("2-D froze");
        let (on1, on_state) = on1.expect("zonal froze");
        let (esc1, esc_state) = esc1.expect("zonal thawed");
        // The triggered snapshots at the zonal model's onset and escape: the 2-D model restarts
        // from the zonal state (it departed from the 2-D one) and stays frozen, then thawed.
        let mut c_on_cell = warm_cell.clone();
        let c_on = run(
            &inputs(&p, w, &land, &z, &on_state, &at(on1)),
            &mut c_on_cell,
        );
        let mut c_esc_cell = cold_cell.clone();
        let c_esc = run(
            &inputs(&p, w, &land, &z, &esc_state, &at(esc1)),
            &mut c_esc_cell,
        );
        eprintln!(
            "snowball onset: 2-D {on2:.0} ppm, corrected zonal {on1:.0} ppm (2-D there from the zonal state: sea ice {:.3}); escape: 2-D {esc2:.0} ppm, corrected zonal {esc1:.0} ppm (2-D there from the zonal state: {:.3})",
            ice_of(&c_on),
            ice_of(&c_esc)
        );
        let steps = |a: f64, b: f64| ((a / b).ln() / 1.25f64.ln()).abs().round();
        assert!(steps(on1, on2) <= 3.0, "onsets {on1} and {on2} ppm");
        assert!(steps(esc1, esc2) <= 3.0, "escapes {esc1} and {esc2} ppm");
        assert!(ice_of(&c_on) >= p.snowball_on_frac, "{}", ice_of(&c_on));
        assert!(ice_of(&c_esc) < p.snowball_off_frac, "{}", ice_of(&c_esc));
    }

    #[test]
    fn the_climate_grid_divides_l1_and_the_sharpening_keeps_its_ends() {
        for (fw, cols, k) in [
            (1024, 128, 8),
            (2048, 128, 16),
            (256, 128, 2),
            (1000, 128, 5),
        ] {
            let g = climate_grid(fw, fw / 2, cols);
            assert_eq!(g.k, k, "{fw}");
            assert_eq!(g.w * g.k, fw);
        }
        assert_eq!(sharpen(0.0, 4.0), 0.0);
        assert!((sharpen(1.0, 4.0) - 1.0).abs() < 1e-12);
        assert!((sharpen(0.5, 4.0) - 0.5).abs() < 1e-12);
        assert!(sharpen(0.6, 4.0) > 0.8 && sharpen(0.4, 4.0) < 0.2);
        assert_eq!(sharpen(0.3, 1.0), 0.3);
    }
}

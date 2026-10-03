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

    /// The budget's precipitation rate (mm/yr) and relative humidity of month `m` at L1 cell `c`
    /// (bilinear).
    pub fn budget_at(&self, c: usize, m: usize) -> Option<(f64, f64)> {
        let mo = self.moisture.as_ref()?;
        let (mut p, mut rh) = (0.0, 0.0);
        for (cc, wt) in self.grid.corners(c) {
            p += wt * mo.p[cc][m];
            rh += wt * mo.rh[cc][m];
        }
        Some((p * YEAR_S, rh))
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
        tl,
        to,
        ice,
        dsst,
        itcz_col,
        mp,
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

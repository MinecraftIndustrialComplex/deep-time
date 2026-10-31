// SPDX-License-Identifier: LGPL-3.0-or-later
//! The climate the surface reads — runoff, temperature, precipitation (plan §8.2: "runoff from the
//! latest climate snapshot").
//!
//! **The interface:** [`snapshot`] reads the fields `climate/t_mean_c` (°C) and `climate/precip_mm`
//! (mm/yr) from the world when the climate stage has written them (P1-05: a planet preset with a
//! `climate` section runs it before the surface every plate step, `crate::climate`), and otherwise
//! builds the stand-in below. Everything the surface needs is derived from those two the same way
//! in both cases: potential evapotranspiration PET = 60 mm/°C · T (0 below 0 °C, a
//! Thornthwaite-like linear stand-in), actual evapotranspiration from Fu's form of the Budyko curve
//! with ω = 2.6 (Fu 1981; Zhang et al. 2004), runoff = P − ET, and the aridity index P / PET. With
//! the climate stage, the water temperature of carbonate factories (`sst_c`) is the stage's
//! temperature at the cell (at sea it has no lapse rate); with the stand-in, its zonal sea-level
//! temperature.
//!
//! **The stand-in (presets without a `climate` section; documented, not tuned):** temperature
//! T = 28 − 42·sin²φ °C at sea level (≈ Earth's zonal annual mean: 17 °C at 30°, 7 °C at 45°,
//! −3 °C at 60°) minus a 6.5 °C/km lapse rate; precipitation from a zonal profile of |φ| (mm/yr: 2,000 at the equator, a dry belt
//! of 350 at 27°, a wet belt of 1,000 at 45–55°, 150 at the poles — the three-cell belts of
//! `surface_processes_climate.md` §5), times an orographic factor (up to +40 % by 1.5 km, drying
//! above 3 km), times continentality 0.35 + 0.65·√(ocean fraction within ~400 km) so continental
//! interiors dry out. There is no rain shadow and no season; the climate stage replaces all of it.

use rayon::prelude::*;

use super::grid::{Dims, box_blur};
use super::lakes::CellWater;
use crate::hydro::{Climate, HydroParams};
use crate::world::WorldState;

/// Field names the climate stage writes (P1-05, `crate::climate::field`) and this module reads.
pub const T_FIELD: &str = "climate/t_mean_c";
pub const P_FIELD: &str = "climate/precip_mm";

/// Lapse rate, °C per metre.
pub const LAPSE_C_PER_M: f64 = 0.0065;

/// The climate of one step, per cell.
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    /// Mean annual temperature, °C.
    pub t_c: Vec<f32>,
    /// Sea-surface temperature (no lapse), °C.
    pub sst_c: Vec<f32>,
    pub precip_mm: Vec<f32>,
    pub pet_mm: Vec<f32>,
    pub runoff_m_yr: Vec<f32>,
    /// P / PET, capped at 10.
    pub aridity: Vec<f32>,
    /// True when it came from a climate stage, false for the stand-in.
    pub from_stage: bool,
}

/// Zonal precipitation of the stand-in, mm/yr.
pub fn zonal_precip(lat_deg: f64) -> f64 {
    const P: [(f64, f64); 10] = [
        (0.0, 2000.0),
        (10.0, 1800.0),
        (20.0, 700.0),
        (27.0, 350.0),
        (35.0, 550.0),
        (45.0, 1000.0),
        (55.0, 1000.0),
        (65.0, 600.0),
        (80.0, 250.0),
        (90.0, 150.0),
    ];
    let a = lat_deg.abs().min(90.0);
    for w in P.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if a <= x1 {
            return y0 + (y1 - y0) * (a - x0) / (x1 - x0);
        }
    }
    P[P.len() - 1].1
}

/// Sea-level temperature of the stand-in, °C.
pub fn zonal_temperature(lat_deg: f64) -> f64 {
    let s = lat_deg.to_radians().sin();
    28.0 - 42.0 * s * s
}

/// Orographic factor of the stand-in at `z` metres above sea level.
pub fn orographic(z: f64) -> f64 {
    if z <= 0.0 {
        1.0
    } else if z <= 1500.0 {
        1.0 + 0.4 * z / 1500.0
    } else if z <= 3000.0 {
        1.4
    } else {
        (1.4 - 0.4 * (z - 3000.0) / 1000.0).max(0.3)
    }
}

/// Fu's Budyko curve: actual evapotranspiration for P and PET (mm), ω = 2.6.
pub fn fu_et(p: f64, pet: f64) -> f64 {
    if p <= 0.0 || pet <= 0.0 {
        return 0.0;
    }
    const W: f64 = 2.6;
    let r = pet / p;
    p * (1.0 + r - (1.0 + r.powf(W)).powf(1.0 / W))
}

/// The climate for elevations `h` (m, relative to `sea_level`), from the world's climate fields if
/// a climate stage wrote them, else the stand-in. `lat_of_row(j)` is row j's latitude.
pub fn snapshot(
    world: &WorldState,
    d: &Dims,
    lat_of_row: &(dyn Fn(usize) -> f64 + Sync),
    h: &[f64],
    sea_level: f64,
) -> Snapshot {
    let n = d.cells();
    let stage_t = world.fields.get(T_FIELD).filter(|f| f.data.len() == n);
    let stage_p = world.fields.get(P_FIELD).filter(|f| f.data.len() == n);
    let mut out = Snapshot {
        t_c: vec![0.0; n],
        sst_c: vec![0.0; n],
        precip_mm: vec![0.0; n],
        pet_mm: vec![0.0; n],
        runoff_m_yr: vec![0.0; n],
        aridity: vec![0.0; n],
        from_stage: stage_t.is_some() && stage_p.is_some(),
    };
    // Continentality: ocean fraction within ~400 km.
    let radius = ((400_000.0 / d.dx).round() as usize / 2).max(1);
    let mut ocean: Vec<f64> = h
        .iter()
        .map(|&z| if z <= sea_level { 1.0 } else { 0.0 })
        .collect();
    if !out.from_stage {
        box_blur(d, &mut ocean, radius, 2);
    }
    let w = d.w;
    out.t_c
        .par_chunks_mut(w)
        .zip(out.sst_c.par_chunks_mut(w))
        .zip(out.precip_mm.par_chunks_mut(w))
        .zip(out.pet_mm.par_chunks_mut(w))
        .zip(out.runoff_m_yr.par_chunks_mut(w))
        .zip(out.aridity.par_chunks_mut(w))
        .enumerate()
        .for_each(|(j, (((((t, sst), p), pet), ro), ar))| {
            let lat = lat_of_row(j);
            let t0 = zonal_temperature(lat);
            let p0 = zonal_precip(lat);
            for i in 0..w {
                let c = j * w + i;
                let z = (h[c] - sea_level).max(0.0);
                let (tc, pc) = match (stage_t, stage_p) {
                    (Some(ft), Some(fp)) => (f64::from(ft.data[c]), f64::from(fp.data[c])),
                    _ => {
                        let cont = 0.35 + 0.65 * ocean[c].clamp(0.0, 1.0).sqrt();
                        (t0 - LAPSE_C_PER_M * z, p0 * orographic(z) * cont)
                    }
                };
                let e = (60.0 * tc).max(0.0);
                t[i] = tc as f32;
                sst[i] = if out.from_stage { tc } else { t0 } as f32;
                p[i] = pc as f32;
                pet[i] = e as f32;
                ro[i] = ((pc - fu_et(pc, e)).max(0.0) / 1000.0) as f32;
                ar[i] = if e > 0.0 {
                    (pc / e).min(10.0) as f32
                } else {
                    10.0
                };
            }
        });
    out
}

/// One cell's water a year under the bake's budget (`HydroParams::budget`, HYD-035 to HYD-038):
/// precipitation `precip_mm` (mm/yr), the mean annual temperature `t_c` (°C) and half the range
/// between the warmest and the coldest month `half_c` (°C; not finite: the ±8 °C of HYD-032).
/// Its runoff is what the lakes are fed with (ISO-126, ERO-217).
pub fn cell_water(hydro: &HydroParams, precip_mm: f64, t_c: f64, half_c: f64) -> CellWater {
    let cl = Climate::of(precip_mm, t_c, t_c + half_c, t_c - half_c);
    let b = hydro.budget(&cl);
    CellWater {
        runoff_m_yr: b.runoff_mm / 1000.0,
        loss_m_yr: (b.evaporation_mm - b.rain_mm) / 1000.0,
    }
}

/// [`cell_water`] for every cell of the step's climate `clim`: the climate stage's seasons
/// (`climate/t_warm_c`, `climate/t_cold_c`) as departures about the surface's own mean
/// temperature (HYD-033), none without the stage.
pub fn water_budget(world: &WorldState, clim: &Snapshot, hydro: &HydroParams) -> Vec<CellWater> {
    let n = clim.t_c.len();
    let season = |name: &str| {
        world
            .fields
            .get(name)
            .filter(|f| f.data.len() == n)
            .map(|f| &f.data[..])
    };
    let warm = season(crate::climate::field::T_WARM);
    let cold = season(crate::climate::field::T_COLD);
    (0..n)
        .into_par_iter()
        .map(|c| {
            let half = match (warm, cold) {
                (Some(w), Some(k)) => 0.5 * f64::from(w[c] - k[c]),
                _ => f64::NAN,
            };
            cell_water(
                hydro,
                f64::from(clim.precip_mm[c]),
                f64::from(clim.t_c[c]),
                half,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stand_in_has_belts_and_budyko_limits() {
        assert!(zonal_precip(0.0) > zonal_precip(27.0));
        assert!(zonal_precip(50.0) > zonal_precip(27.0));
        assert!((zonal_temperature(0.0) - 28.0).abs() < 1e-12);
        assert!(zonal_temperature(60.0) < 0.0);
        // ET never exceeds P or PET, and tends to the smaller in the limits.
        for (p, e) in [(500.0, 1500.0), (2000.0, 800.0), (1000.0, 1000.0)] {
            let et = fu_et(p, e);
            assert!(et <= p.min(e) + 1e-9 && et > 0.0);
        }
        assert!((fu_et(100.0, 1e6) - 100.0).abs() < 1.0);
    }
}

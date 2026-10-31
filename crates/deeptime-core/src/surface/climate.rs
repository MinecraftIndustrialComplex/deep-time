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
                let e = (60.0 * tc).max(0.0); // ledger: MOI-067
                t[i] = tc as f32;
                sst[i] = if out.from_stage { tc } else { t0 } as f32;
                p[i] = pc as f32;
                pet[i] = e as f32;
                ro[i] = mean_t_runoff_m_yr(pc, tc) as f32;
                ar[i] = if e > 0.0 {
                    (pc / e).min(10.0) as f32
                } else {
                    10.0
                };
            }
        });
    out
}

/// The snapshot's own runoff, m/yr, for precipitation `precip_mm` (mm/yr) and the mean annual
/// temperature `t_c` (°C): P less Fu's evapotranspiration with PET = 60 mm/°C x T, 0 below 0 °C
/// (MOI-067, HYD-034), so a land below freezing sends all of P to its rivers. The stream power's
/// runoff unless `surface.runoff_budget` (ERO-010).
pub fn mean_t_runoff_m_yr(precip_mm: f64, t_c: f64) -> f64 {
    let e = (60.0 * t_c).max(0.0); // ledger: MOI-067
    (precip_mm - fu_et(precip_mm, e)).max(0.0) / 1000.0
}

/// One cell's water a year under the bake's budget (`HydroParams::budget`, HYD-035 to HYD-038):
/// precipitation `precip_mm` (mm/yr), the mean annual temperature `t_c` (°C) and the mean
/// temperatures of the warmest and the coldest month `warm_c`, `cold_c` (°C; not finite: the
/// mean ± 8 °C of HYD-032). Its runoff is what the lakes are fed with (ISO-126, ERO-217) and,
/// with `surface.runoff_budget`, what the stream power's area counts (ERO-221).
///
/// The months are taken as given, not centred on the mean: the climate stage's year is twelve
/// months of its energy balance, and their mean, warmest and coldest need not be symmetric
/// (a polar desert at a mean of −17 °C has its warmest month at −5 and its coldest at −40). Until
/// 2026-10-31 the half-range about the mean was passed instead, which put that warmest month at
/// +0.5 °C, so its snow melted and ran off where the bake, keeping the months' departures
/// (HYD-033), runs off nothing (the runoff law's review; `a_year_s_months_are_taken_as_given`).
/// The year's warmth above freezing is drawn about the mean from the warmest month
/// (`Climate::thaw`; the coldest month does not enter), or by Holdridge's own method from the
/// twelve months where the stage wrote them ([`cell_water_of`] with [`Months::climate`]).
// ledger: ERO-221
pub fn cell_water(
    hydro: &HydroParams,
    precip_mm: f64,
    t_c: f64,
    warm_c: f64,
    cold_c: f64,
) -> CellWater {
    cell_water_of(hydro, &Climate::of(precip_mm, t_c, warm_c, cold_c))
}

/// [`cell_water`] for a cell's climate `cl` (with its twelve months where the stage wrote them:
/// [`Months::climate`]).
// ledger: ERO-221
pub fn cell_water_of(hydro: &HydroParams, cl: &Climate) -> CellWater {
    let b = hydro.budget(cl);
    CellWater {
        runoff_m_yr: b.runoff_mm / 1000.0,
        loss_m_yr: (b.evaporation_mm - b.rain_mm) / 1000.0,
    }
}

/// The climate stage's year on a grid of its cells: its warmest and coldest month
/// (`climate/t_warm_c`, `climate/t_cold_c`) and, where it wrote them (its last step, for the
/// bake: `climate/month/t_sl_c_*`), its twelve months' sea-level temperatures.
pub struct Months<'a> {
    pub warm: &'a [f32],
    pub cold: &'a [f32],
    pub twelve: Option<Vec<&'a [f32]>>,
}

impl Months<'_> {
    /// Cell `c`'s climate at precipitation `precip_mm` (mm/yr) and mean temperature `t_c` (°C):
    /// the twelve months' departures from their mean added to it where the stage wrote them
    /// (HYD-033: a month's departure is the same at any height; Holdridge's own method,
    /// `Climate::thaw`), else its warmest and coldest month as written.
    // ledger: HYD-030, HYD-033
    pub fn climate(&self, precip_mm: f64, t_c: f64, c: usize) -> Climate {
        if let Some(twelve) = &self.twelve {
            let sl: [f64; 12] = std::array::from_fn(|m| f64::from(twelve[m][c]));
            let mean = sl.iter().sum::<f64>() / 12.0;
            if mean.is_finite() {
                return Climate::of_months(precip_mm, t_c, &std::array::from_fn(|m| sl[m] - mean));
            }
        }
        Climate::of(
            precip_mm,
            t_c,
            f64::from(self.warm[c]),
            f64::from(self.cold[c]),
        )
    }
}

/// The climate stage's year ([`Months`]) on a grid of `n` cells: None unless the world holds
/// both its warmest and its coldest month there, so every reader takes both or neither; the
/// twelve months only when all twelve are there.
pub fn months(world: &WorldState, n: usize) -> Option<Months<'_>> {
    let get = |name: &str| {
        world
            .fields
            .get(name)
            .filter(|f| f.data.len() == n)
            .map(|f| &f.data[..])
    };
    let warm = get(crate::climate::field::T_WARM)?;
    let cold = get(crate::climate::field::T_COLD)?;
    let twelve: Option<Vec<&[f32]>> = (0..12)
        .map(|m| get(&crate::climate::field::t_sl_month(m)))
        .collect();
    Some(Months { warm, cold, twelve })
}

/// A cell's climate under the stage's year `months` (None: the mean ± 8 °C of HYD-032).
pub fn cell_climate(months: Option<&Months>, precip_mm: f64, t_c: f64, c: usize) -> Climate {
    match months {
        Some(m) => m.climate(precip_mm, t_c, c),
        None => Climate::of(precip_mm, t_c, f64::NAN, f64::NAN),
    }
}

/// [`cell_water`] for every cell of the step's climate `clim`. From the climate stage, its own
/// year ([`months`]): the snapshot's temperature is the stage's mean (`T_FIELD`), and the stage
/// writes its warmest and coldest month (and at its last step its twelve months) from the same
/// months and lapse rate, so they are HYD-033's departures about the surface's mean exactly.
/// Without the stage, none (HYD-032).
pub fn water_budget(world: &WorldState, clim: &Snapshot, hydro: &HydroParams) -> Vec<CellWater> {
    let n = clim.t_c.len();
    let year = if clim.from_stage {
        months(world, n)
    } else {
        None
    };
    (0..n)
        .into_par_iter()
        .map(|c| {
            cell_water_of(
                hydro,
                &cell_climate(
                    year.as_ref(),
                    f64::from(clim.precip_mm[c]),
                    f64::from(clim.t_c[c]),
                    c,
                ),
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

    // ledger: ERO-221
    /// Where no month freezes, the bake's budget is the surface's own runoff: biotemperature is
    /// the mean temperature, all of P is rain, and PET is the same 60 mm/°C (HYD-029).
    #[test]
    fn without_frost_the_budget_is_the_surface_s_runoff() {
        let hydro = HydroParams::default();
        for (p, t, half) in [
            (2000.0, 26.0, 2.0),
            (800.0, 15.0, 10.0),
            (300.0, 21.0, 14.0),
            (1200.0, 8.0, 8.0),
            (50.0, 30.0, 5.0),
        ] {
            let budget = cell_water(&hydro, p, t, t + half, t - half).runoff_m_yr;
            let own = mean_t_runoff_m_yr(p, t);
            assert!(
                (budget - own).abs() < 1.0e-12,
                "P {p} T {t} ±{half}: budget {budget}, own {own}"
            );
        }
    }

    // ledger: ERO-221, HYD-036, HYD-038
    /// Ground whose warmest month is below freezing sends nothing to its rivers under the
    /// budget (its snow stays, the glaciers' to move), and all of P under the surface's own rule.
    #[test]
    fn ground_that_never_thaws_runs_off_nothing() {
        let hydro = HydroParams::default();
        for (p, t, half) in [
            (300.0, -20.0, 10.0),
            (100.0, -30.0, 15.0),
            (800.0, -8.0, 6.0),
        ] {
            assert_eq!(
                cell_water(&hydro, p, t, t + half, t - half).runoff_m_yr,
                0.0
            );
            assert!((mean_t_runoff_m_yr(p, t) - p / 1000.0).abs() < 1.0e-12);
        }
    }

    /// The bake's budget's runoff, mm/yr, for a climate given by its months.
    fn budget_runoff_mm(p: f64, t_mean: f64, warm: f64, cold: f64) -> f64 {
        HydroParams::default()
            .budget(&Climate {
                precip_mm: p,
                t_mean_c: t_mean,
                t_warm_c: warm,
                t_cold_c: cold,
                months_c: None,
            })
            .runoff_mm
    }

    // ledger: ERO-222
    /// The Mackenzie basin, a permafrost basin of 1.8 million km² (MAGS Water and Energy Budget
    /// Synthesis, Table 2.1: "Mean annual precipitation: 410mm", "Mean annual
    /// evapotranspiration: 237mm", "Average monthly temperature: 15 C in the summer and -25 C in
    /// the winter"; Woo & Thorne 2003, Table 2: mean runoff 169 mm at Arctic Red River). The
    /// budget's evapotranspiration and runoff are within the 54 mm mean absolute error of Fu's
    /// curve over 470 catchments (Zhang et al. 2004, abstract); the snapshot's own rule (a mean
    /// below 0 °C) evaporates nothing and runs off all 410 mm.
    #[test]
    fn the_mackenzie_basin_evaporates_as_measured() {
        let (p, warm, cold) = (410.0, 15.0, -25.0);
        // The source gives no annual mean: the sinusoid's, the summer's and the winter's midpoint.
        let t = 0.5 * (warm + cold);
        let q = budget_runoff_mm(p, t, warm, cold);
        let et = p - q;
        eprintln!("Mackenzie: budget runoff {q:.0} mm, ET {et:.0} mm (measured 169 and 237)");
        assert!((et - 237.0).abs() <= 54.0, "budget ET {et:.0} against 237");
        assert!(
            (q - 169.0).abs() <= 54.0,
            "budget runoff {q:.0} against 169"
        );
        assert!((1000.0 * mean_t_runoff_m_yr(p, t) - p).abs() < 1.0e-9);
    }

    // ledger: ERO-222
    /// The McMurdo Dry Valleys, a polar desert (Fountain et al., "Snow in the McMurdo Dry
    /// Valleys, Antarctica": "Air temperatures average about –17oC with the winter minimum about
    /// -40oC and the summer maximum a few degrees above freezing"; "In Wright Valley, annual values
    /// of snowfall have ranged 7 – 11 mm water equivalent ..., most of which is lost to
    /// sublimation"). A warmest month below freezing (its mean under the summer's maximum): the
    /// budget sends none of the snowfall to the rivers. A warmest month of 2 or 3 °C: under half.
    /// The snapshot's own rule sends all of it. The budget's gap, printed: a warmest month just
    /// above freezing melts all of the snow by its degree-days and evaporates little (PET from a
    /// biotemperature of a few hundredths of a degree), so at 1 °C most of the snowfall runs off;
    /// the budget has no sublimation of the cold season's snow.
    #[test]
    fn the_dry_valleys_lose_their_snowfall() {
        for p in [7.0, 11.0] {
            assert!((1000.0 * mean_t_runoff_m_yr(p, -17.0) - p).abs() < 1.0e-9);
            for warm in [-5.0, -1.0, 0.0] {
                let q = budget_runoff_mm(p, -17.0, warm, -40.0);
                assert_eq!(q, 0.0, "warm month {warm} °C, P {p} mm");
            }
            for warm in [2.0, 3.0] {
                let q = budget_runoff_mm(p, -17.0, warm, -40.0);
                assert!(
                    q < 0.5 * p,
                    "warm month {warm} °C, P {p} mm: runoff {q:.2} mm"
                );
            }
            for warm in [0.5, 1.0, 1.5] {
                let q = budget_runoff_mm(p, -17.0, warm, -40.0);
                eprintln!("Dry Valleys P {p} mm, warmest month {warm} °C: runoff {q:.1} mm");
            }
        }
    }

    // ledger: ERO-222, HYD-032
    /// The Tibetan Plateau's lakes (Biskop et al. 2016, Table 4, the J2000g model on the HAR
    /// reanalysis, land components, mm/yr: Mapam Yumco P 230, AET 170, runoff 60; Paiku Co 250,
    /// 180, 70; Tangra Yumco 300, 210, 90; Nam Co 420, 290, 130; §2.1: "The mean annual air
    /// temperature (MAAT) lies between 0 and −3 ◦C"). The source gives no months, so they are the
    /// mean ± 8 °C of HYD-032. The budget evaporates in the plateau's summer, where the snapshot's
    /// own rule (MAAT below 0 °C) evaporates nothing, and no more than the precipitation. The
    /// budget's shortfall against Biskop's AET is printed, not asserted: Holdridge's PET has no
    /// term for the plateau's sun and wind, and a better PET would close it (ERO-222's gap).
    #[test]
    fn the_tibetan_lakes_evaporate_under_the_budget() {
        for (p, aet) in [
            (230.0, 170.0),
            (250.0, 180.0),
            (300.0, 210.0),
            (420.0, 290.0),
        ] {
            for t in [0.0, -1.5, -3.0] {
                let et = p - budget_runoff_mm(p, t, t + 8.0, t - 8.0);
                eprintln!(
                    "Tibet P {p} MAAT {t}: budget ET {et:.0} mm (Biskop {aet}, shortfall {:.0})",
                    aet - et
                );
                assert!(et > 0.0 && et <= p, "P {p}, MAAT {t}: budget ET {et:.0}");
                if t < 0.0 {
                    assert!((1000.0 * mean_t_runoff_m_yr(p, t) - p).abs() < 1.0e-9);
                }
            }
        }
    }

    // ledger: ERO-222, HYD-030, HYD-031
    /// Holdridge's worked example (Life Zone Ecology, p. 24 of the scan: Verkhoyansk's monthly
    /// means "-50.0, -44.4, -32.2, -15.5, 0, + 12.2, + 13.3, +9.4, + 1.7, -15.5, -37.2 and -47.8",
    /// "3.050 as a rough mean annual biotemperature", "mean annual precipitation 135 mm"). With
    /// the twelve months, Holdridge's own method gives his 3.05 °C (36.6 / 12). With only the mean
    /// and the warmest month, the sinusoidal year about the mean gives 2.70 °C, 0.35 under it (the
    /// method's gap on a year this lopsided: its summer is short and its winter long and flat).
    #[test]
    fn verkhoyansk_s_biotemperature_is_holdridge_s() {
        let months = [
            -50.0, -44.4, -32.2, -15.5, 0.0, 12.2, 13.3, 9.4, 1.7, -15.5, -37.2, -47.8,
        ];
        let mean = months.iter().sum::<f64>() / 12.0;
        let departures: [f64; 12] = std::array::from_fn(|m| months[m] - mean);
        let twelve = Climate::of_months(135.0, mean, &departures);
        assert!((twelve.t_warm_c - 13.3).abs() < 1.0e-9 && (twelve.t_cold_c + 50.0).abs() < 1.0e-9);
        let (t_plus, share) = twelve.thaw();
        let sinusoid = Climate::of(135.0, mean, 13.3, -50.0).thaw().0;
        eprintln!(
            "Verkhoyansk: biotemperature {t_plus:.3} °C by the twelve months, {sinusoid:.3} by the sinusoid (Holdridge 3.05); thawed {share:.2} of the year"
        );
        assert!((t_plus - 3.05).abs() < 1.0e-9, "biotemperature {t_plus}");
        assert!((sinusoid - 2.70).abs() < 0.005, "sinusoid {sinusoid}");
        // Five months at or above 0 °C, May's at 0: the year drawn straight between the months'
        // middles thaws from May's middle into September's.
        assert!(share > 3.0 / 12.0 && share < 5.0 / 12.0, "{share}");
    }

    // ledger: HYD-030, HYD-031
    /// The biotemperature is continuous in the months and never below max(mean, 0), on lopsided
    /// years too: with the months as given, a sinusoid centred between the warmest and the coldest
    /// month dropped the mean (the runoff law's re-review: P 600, mean 4 °C, warmest month 10, a
    /// coldest month of 0 °C gave 4.00 °C and of −0.01 °C 5.00, −12 % runoff for 0.01 °C; months
    /// 10 and −6 gave 3.63, under the mean).
    #[test]
    fn the_biotemperature_is_continuous_and_never_under_the_mean() {
        let hydro = HydroParams::default();
        // Both sides of a coldest month of 0 °C, a lopsided year: nothing jumps.
        let at = |cold: f64| {
            let cl = Climate::of(600.0, 4.0, 10.0, cold);
            (cl.thaw().0, cell_water_of(&hydro, &cl).runoff_m_yr * 1000.0)
        };
        let (b0, q0) = at(0.0);
        for cold in [0.01, -0.01, -6.0] {
            let (b, q) = at(cold);
            assert!(
                (b - b0).abs() < 1.0e-12 && (q - q0).abs() < 1.0e-9,
                "{cold}: {b} {q}"
            );
        }
        assert!(b0 >= 4.0, "{b0}");
        eprintln!(
            "P 600, mean 4, warmest month 10 °C: biotemperature {b0:.3} °C, runoff {q0:.0} mm, whatever the coldest month"
        );
        // The same both sides of 0 °C with twelve lopsided months: a long mild winter whose
        // coldest month crosses freezing, and a short warm summer.
        let year = |coldest: f64| {
            let mut m = [
                6.0, 3.0, coldest, 3.0, 5.0, 7.0, 9.0, 11.0, 14.0, 22.0, 14.0, 9.0,
            ];
            let mean = m.iter().sum::<f64>() / 12.0;
            m.iter_mut().for_each(|x| *x -= mean);
            let cl = Climate::of_months(600.0, mean, &m);
            let (b, share) = cl.thaw();
            (
                mean,
                b,
                share,
                cell_water_of(&hydro, &cl).runoff_m_yr * 1000.0,
            )
        };
        let (_, b0, s0, q0) = year(0.0);
        for coldest in [0.01, -0.01] {
            let (mean, b, s, q) = year(coldest);
            assert!(b >= mean.max(0.0), "{coldest}: {b} under {mean}");
            assert!(
                (b - b0).abs() < 2.0e-3 && (s - s0).abs() < 2.0e-3 && (q - q0).abs() < 0.5,
                "{coldest}: biotemperature {b} {b0}, share {s} {s0}, runoff {q} {q0}"
            );
        }
        // Never under max(mean, 0) and never over the warmest month, over a sweep of years, the
        // coldest month far under the mean, at it, and at the sinusoid's own minimum.
        for i in 0..=100 {
            let mean = -25.0 + 0.5 * f64::from(i);
            for d in 0..=30 {
                let warm = mean + f64::from(d);
                for cold in [mean - 30.0, mean - 10.0, mean, 2.0 * mean - warm] {
                    let (b, s) = Climate::of(500.0, mean, warm, cold).thaw();
                    assert!(
                        b >= mean.max(0.0) && b <= warm.max(0.0) + 1.0e-12,
                        "{mean} {warm} {cold}: {b}"
                    );
                    assert!((0.0..=1.0).contains(&s), "{mean} {warm} {cold}: share {s}");
                }
            }
        }
    }

    // ledger: ERO-221, HYD-035
    /// A land below freezing on the year with a thawed summer evaporates in it: it runs off
    /// less than all of P, and more than nothing.
    #[test]
    fn a_cold_land_with_a_summer_evaporates_in_it() {
        let hydro = HydroParams::default();
        for (p, t, half) in [(400.0, -2.0, 12.0), (250.0, -8.0, 18.0), (600.0, -1.0, 8.0)] {
            let budget = cell_water(&hydro, p, t, t + half, t - half).runoff_m_yr;
            let own = mean_t_runoff_m_yr(p, t);
            assert!((own - p / 1000.0).abs() < 1.0e-12);
            assert!(
                budget > 0.0 && budget < 0.9 * own,
                "P {p} T {t} ±{half}: budget {budget}, own {own}"
            );
        }
    }

    // ledger: ERO-221, HYD-030
    /// Land with a mean above freezing and a month below it is not frost-free: its
    /// biotemperature counts the frozen months as 0 °C, so it is above the mean (Holdridge), the
    /// budget's PET is higher than the snapshot's 60 T, and it runs off less (the runoff law's
    /// review: P 600 mm at 5 ± 15 °C, 336 against 245 mm). With no month below freezing the two
    /// are the same (`without_frost_the_budget_is_the_surface_s_runoff`). The years here are
    /// symmetric about the mean; on a lopsided one the year about the mean takes its frost from
    /// the warmest month (2 mean − warm < 0), and its biotemperature is still never under the mean
    /// (`the_biotemperature_is_continuous_and_never_under_the_mean`).
    #[test]
    fn a_warm_mean_with_a_frosty_month_runs_off_less() {
        let hydro = HydroParams::default();
        for (p, t, half) in [(600.0, 5.0, 15.0), (900.0, 8.0, 12.0), (400.0, 2.0, 10.0)] {
            let budget = 1000.0 * cell_water(&hydro, p, t, t + half, t - half).runoff_m_yr;
            let own = 1000.0 * mean_t_runoff_m_yr(p, t);
            eprintln!(
                "P {p} at {t} ± {half} °C: runoff {own:.0} mm by the snapshot's rule, {budget:.0} by the budget ({:+.0} %)",
                100.0 * (budget / own - 1.0)
            );
            assert!(
                budget > 0.0 && budget < own,
                "P {p} at {t} ± {half}: budget {budget}, own {own}"
            );
        }
    }

    // ledger: ERO-221, HYD-033
    /// The months are taken as the climate stage gives them, not centred on the mean (the runoff
    /// law's review): the Dry Valleys' year (Fountain et al.: a mean of about −17 °C, a winter
    /// minimum of about −40), with its warmest month at −5 °C, never thaws and runs off nothing.
    /// Centred on the mean, the same range put the warmest month at +0.5 °C: what that ran off is
    /// printed. (The year about the mean takes its amplitude from the warmest month as given;
    /// its coldest does not enter, `the_biotemperature_is_continuous_and_never_under_the_mean`.)
    #[test]
    fn a_year_s_months_are_taken_as_given() {
        let hydro = HydroParams::default();
        let (t, warm, cold) = (-17.0, -5.0, -40.0);
        let half = 0.5 * (warm - cold);
        for p in [7.0, 11.0, 300.0] {
            assert_eq!(cell_water(&hydro, p, t, warm, cold).runoff_m_yr, 0.0);
            let centred = 1000.0 * cell_water(&hydro, p, t, t + half, t - half).runoff_m_yr;
            eprintln!(
                "P {p} mm, mean {t}, months {warm} and {cold} °C: runoff 0; centred (+{:.1} and {:.1}): {centred:.1} mm, {:.2} of P",
                t + half,
                t - half,
                centred / p
            );
            assert!(centred > 0.0);
        }
        // Without the months, the mean ± 8 °C (HYD-032).
        let none = cell_water(&hydro, 300.0, -1.0, f64::NAN, f64::NAN).runoff_m_yr;
        let eight = cell_water(&hydro, 300.0, -1.0, 7.0, -9.0).runoff_m_yr;
        assert_eq!(none, eight);
    }
}

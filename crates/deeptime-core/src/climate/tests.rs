// SPDX-License-Identifier: LGPL-3.0-or-later
//! The climate on a synthetic planet and in the coupled pipeline.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use deeptime_format::GridDesc;

use super::snapshot::{self, Planet};
use super::*;
use crate::checkpoint::Checkpoint;
use crate::params::planet_builtin;
use crate::preset::with_timeline;
use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};

/// An idealised planet on 256 × 128 cells (16k, r 64): ocean at −4 km, a continent from 60° to
/// 150° E between 60° S and 70° N at 300 m with a 3-km north–south range along 70–74° E from 25°
/// to 60° N, and a 2-km polar continent south of 70° S.
#[allow(clippy::needless_range_loop)]
fn idealised() -> (GridDesc, Vec<f64>, Vec<f64>) {
    let grid = GridDesc::for_planet("L1", 16384, 64).unwrap();
    let (w, h) = (grid.w as usize, grid.h as usize);
    let lat: Vec<f64> = (0..h).map(|j| lat_of_row(&grid, j as u32)).collect();
    let mut z = vec![-4000.0; w * h];
    for j in 0..h {
        for i in 0..w {
            let lon = i as f64 / w as f64 * 360.0;
            let c = j * w + i;
            if lat[j] < -70.0 {
                z[c] = 2000.0;
            } else if (60.0..150.0).contains(&lon) && (-60.0..70.0).contains(&lat[j]) {
                z[c] = 300.0;
                if (70.0..74.0).contains(&lon) && (25.0..60.0).contains(&lat[j]) {
                    z[c] = 3000.0;
                }
            }
        }
    }
    (grid, lat, z)
}

fn planet() -> Planet {
    Planet {
        axial_tilt_deg: 23.4,
        star_flux_rel: 1.0,
        day_length_h: 24.0,
        earth_scale: 782.278 / 6370.0,
        m_per_block_h: 300.0,
        co2_ppm: 280.0,
        sun_rel: 1.0,
        orbit: None,
    }
}

#[test]
fn an_idealised_planet_has_earthlike_zones() {
    idealised_zones(ClimateParams::default());
}

/// The same on the banded climate (`climate.model = banded`, kept for comparison).
#[test]
fn an_idealised_planet_has_earthlike_zones_on_the_banded_climate() {
    idealised_zones(ClimateParams {
        model: crate::climate::ClimateModel::Banded,
        ..ClimateParams::default()
    });
}

/// The same with the per-cell climate's moisture budget (`climate.rain_model = budget`).
#[test]
fn an_idealised_planet_has_earthlike_zones_with_the_moisture_budget() {
    idealised_zones(ClimateParams {
        rain_model: crate::climate::RainModel::Budget,
        ..ClimateParams::default()
    });
}

/// The same with the upwind rain (`climate.rain_model = upwind`: the circulation's rain dried by
/// the budget of vapour and cloud water the wind carries).
#[test]
fn an_idealised_planet_has_earthlike_zones_with_the_upwind_rain() {
    idealised_zones(ClimateParams {
        rain_model: crate::climate::RainModel::Upwind,
        ..ClimateParams::default()
    });
}

fn idealised_zones(p: ClimateParams) {
    let (grid, lat, z) = idealised();
    let (w, h) = (grid.w as usize, grid.h as usize);
    let d = Dims::new(w, h, 64.0 * 300.0, true);
    let inp = snapshot::Input {
        grid: &grid,
        d,
        lat: &lat,
        h: &z,
        sea: 0.0,
        p: &p,
        planet: planet(),
        monthly: true,
        ice_prev: None,
        koppen_prev: None,
        correction: &super::percell::Correction::default(),
    };
    let mut ebm = EbmState::default();
    let s = snapshot::compute(&inp, &mut ebm, &[], &mut Default::default());
    assert!(s.ebm_net_toa.abs() < 0.3, "EBM balance {}", s.ebm_net_toa);
    let row = |deg: f64| {
        (0..h)
            .min_by(|&a, &b| (lat[a] - deg).abs().total_cmp(&(lat[b] - deg).abs()))
            .unwrap()
    };
    let col = |deg: f64| ((deg / 360.0 * w as f64).round() as usize) % w;
    let at = |v: &[f32], la: f64, lo: f64| f64::from(v[row(la) * w + col(lo)]);
    // Temperature: warm equator, frozen polar continent, the lapse rate on the range.
    assert!(
        at(&s.t_mean, 0.0, 250.0) > 22.0,
        "{}",
        at(&s.t_mean, 0.0, 250.0)
    );
    assert!(
        at(&s.t_mean, -80.0, 100.0) < -15.0,
        "{}",
        at(&s.t_mean, -80.0, 100.0)
    );
    assert!(at(&s.t_mean, 40.0, 72.0) < at(&s.t_mean, 40.0, 85.0) - 10.0);
    // Seasons: the continental interior's range is larger than the ocean's at 50° N.
    let range = |la, lo| at(&s.t_warm, la, lo) - at(&s.t_cold, la, lo);
    assert!(range(50.0, 120.0) > range(50.0, 250.0) + 10.0);
    // Precipitation: the ITCZ over the ocean is wetter than the subtropical high.
    assert!(at(&s.precip, 2.0, 250.0) > 2.0 * at(&s.precip, 25.0, 250.0));
    // Nowhere wetter than Earth's wettest places (about 12 m a year), the rows at and past the
    // poles included (the upwind rain once divided by a polar cell's zero circulation rain). The
    // budget rain (not a default) still has one cell of about 30 m on this planet: finite only.
    let wettest = s.precip.iter().cloned().fold(0.0f32, f32::max);
    let cap = if p.rain_model == crate::climate::RainModel::Budget {
        f32::MAX
    } else {
        15_000.0
    };
    assert!(wettest.is_finite() && wettest < cap, "wettest {wettest} mm");
    // The westerlies' rain shadow: the range's windward (west) side is wetter than its lee.
    let windward = at(&s.precip, 45.0, 68.0);
    let lee = at(&s.precip, 45.0, 80.0);
    // (The upwind rain's linear condensation and full recycling carry more water past the
    // range: 1.46 on this planet, 2026-10-05.)
    let shadow = if p.rain_model == crate::climate::RainModel::Upwind {
        1.3
    } else {
        1.5
    };
    assert!(windward > shadow * lee, "windward {windward} lee {lee}");
    // The interior is drier than the coasts the wind comes from.
    assert!(at(&s.precip, 45.0, 140.0) < windward);
    // Köppen: every main group on land; the polar continent is EF or ET and glaciated.
    let groups: std::collections::BTreeSet<char> = (0..w * h)
        .filter(|&c| z[c] > 0.0)
        .map(|c| koppen::group(s.koppen[c]))
        .collect();
    for g in ['A', 'B', 'C', 'D', 'E'] {
        assert!(groups.contains(&g), "no {g} climate: {groups:?}");
    }
    assert_eq!(koppen::group(s.koppen[row(-80.0) * w + col(100.0)]), 'E');
    assert_eq!(s.glacier[row(-80.0) * w + col(100.0)], 1);
    assert_eq!(s.koppen[row(0.0) * w + col(250.0)], koppen::SEA);
    assert!(s.ice.frac() > 0.05 && s.ice.lat < -60.0);
}

/// The wettest place on Earth in the long run: Mawsynram, India, 11.872 m a year over 38 years
/// (WMO's Records of Weather and Climate Extremes, as of 31 July 2025; verified 2026-10-18), and
/// no basin's runoff exceeds its rain. A cell's annual rain and runoff stay under this with
/// room for the model's coarser cells, m/yr.
const WETTEST_M_YR: f64 = 15.0; // ledger: CLM-205
/// No annual mean on Earth is hotter than about 35 °C (Dallol, Ethiopia; from memory), nor any
/// sea surface's than about 33 °C; a hot planet's cell stays under this, °C.
const HOTTEST_MEAN_C: f32 = 45.0; // ledger: CLM-205
/// The weathering kernel stays within this many times Earth's (`KERNEL_EARTH`): a test bound,
/// game-choice (the coordinator's call, 2026-10-18), far above a warm, wet planet's few times
/// Earth's.
const KERNEL_CEILING_REL: f64 = 100.0; // ledger: CLM-205

/// Every cell's annual rain, runoff (rain less Fu's evapotranspiration, as the weathering kernel
/// takes it) and mean temperature under the ceilings, with the cell named when one is not.
fn assert_physical(s: &snapshot::Snapshot, lat: &[f64], w: usize, what: &str) {
    for (c, (&t, &p)) in s.t_mean.iter().zip(&s.precip).enumerate() {
        let (pr, tc) = (f64::from(p), f64::from(t));
        let pet = (60.0 * tc).max(0.0);
        let runoff = (pr - crate::surface::climate::fu_et(pr, pet)).max(0.0) / 1000.0;
        let at = format!("{what}: cell ({}, {}) at {:.1}°", c % w, c / w, lat[c / w]);
        assert!(p.is_finite() && t.is_finite(), "{at}: {p} mm, {t} °C");
        assert!(
            pr / 1000.0 < WETTEST_M_YR && runoff < WETTEST_M_YR,
            "{at}: rain {pr:.0} mm/yr, runoff {runoff:.1} m/yr"
        );
        assert!(t < HOTTEST_MEAN_C, "{at}: {t:.1} °C");
    }
}

/// 2026-10-18: a narrow inlet whose sea reached just past 50° N was the narrowest basin, and the
/// overturning conveyor sank all its 30 Sv there, in one or two coarse cells: their slab took
/// the heat of cooling that water (about a petawatt), its sea went to hundreds of degrees, the
/// coasts round it got 10⁵ to 10¹¹ mm of rain a year, and the weathering kernel reached 10¹⁴ times
/// Earth's (`supercontinent`, seed 1, quick 16k). The idealised planet with such an inlet cut
/// into its continent (101–107° E, from the equator to 56° N, joined to the open ocean by a strait
/// at 0–12° N): every cell's rain, runoff and temperature stay physical, on every rain model.
/// The conveyor before the fix sank its 30 Sv in the inlet's two coarse cells at 51° N here, and
/// this planet's climate reached 258 °C and 3.5·10¹⁰ mm of rain a year.
#[test]
fn a_narrow_inlet_past_the_sinking_latitude_keeps_the_climate_physical() {
    let (grid, lat, mut z) = idealised();
    let (w, h) = (grid.w as usize, grid.h as usize);
    for j in 0..h {
        for i in 0..w {
            let lon = i as f64 / w as f64 * 360.0;
            let inlet = (101.0..107.0).contains(&lon) && (0.0..56.0).contains(&lat[j]);
            let strait = (101.0..152.0).contains(&lon) && (0.0..12.0).contains(&lat[j]);
            if inlet || strait {
                z[j * w + i] = -4000.0;
            }
        }
    }
    let d = Dims::new(w, h, 64.0 * 300.0, true);
    for rain_model in [
        crate::climate::RainModel::Belts,
        crate::climate::RainModel::Upwind,
    ] {
        let p = ClimateParams {
            rain_model,
            ..ClimateParams::default()
        };
        let inp = snapshot::Input {
            grid: &grid,
            d,
            lat: &lat,
            h: &z,
            sea: 0.0,
            p: &p,
            planet: planet(),
            monthly: true,
            ice_prev: None,
            koppen_prev: None,
            correction: &super::percell::Correction::default(),
        };
        let mut ebm = EbmState::default();
        let s = snapshot::compute(&inp, &mut ebm, &[], &mut Default::default());
        assert_physical(&s, &lat, w, &format!("{rain_model:?}"));
        // The conveyor sinks its full transport in the open ocean, not in the inlet.
        let sink = &s.cell.as_ref().unwrap()["sinking"];
        let lon = sink["lon"].as_f64().unwrap();
        assert!(!(95.0..115.0).contains(&lon), "sinks in the inlet: {sink}");
        assert_eq!(sink["sv"].as_f64().unwrap(), p.overturning_sv, "{sink}");
    }
}

/// A small coupled planet: `n` L0 samples, L1 r = 64 at 16k (256 × 128), `myr` of history.
fn pipeline_json(n: u32, myr: f64, seed: u64) -> String {
    let json = with_timeline(
        planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(myr),
    )
    .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = serde_json::json!(n);
    v["resolution"]["l1_r"] = serde_json::json!(64);
    v["keyframes"]["size"] = serde_json::json!([128, 64]);
    v["keyframes"]["every_myr"] = serde_json::json!(20.0);
    v["seed"] = serde_json::json!(seed);
    // The young landscape and L3 amplification (P1-06) have their own tests
    // (`surface::amplify::tests`); their L2 and L3 are far finer than this L1.
    v.as_object_mut().unwrap().remove("young");
    v.as_object_mut().unwrap().remove("amplify");
    v.to_string()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-climate-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn session(json: &str, dir: &Path, threads: usize) -> Session {
    Session::create(
        json,
        "",
        SessionOptions {
            out_dir: dir.to_path_buf(),
            threads,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap()
}

fn finish(s: &Session) {
    loop {
        match s.advance(Duration::from_secs(60)).unwrap() {
            AdvanceStatus::Done => break,
            AdvanceStatus::Cancelled => panic!("cancelled"),
            _ => {}
        }
    }
}

/// The present-day target counts ice sheets only (decisions log 2026-10-18): a continuous ice body
/// of 50 000 km² or more is a sheet (Cogley et al. 2011), whatever its shape and across the
/// east-west seam; a smaller body is mountain ice, and cells touching only at a corner are two
/// bodies.
#[test]
fn ice_bodies_of_fifty_thousand_km2_are_sheets_and_the_rest_mountain_ice() {
    let (w, h) = (20usize, 10usize);
    let geo = Geometry {
        w,
        h,
        area: vec![10_000.0; h],
        sx: vec![100_000.0; h],
        sy: vec![100_000.0; h],
        band: vec![0; h],
        bands: 1,
    };
    let mut ice = vec![false; w * h];
    let mut set = |i: usize, j: usize| ice[j * w + i] = true;
    // Five cells (50 000 km²) across the seam: a sheet.
    for i in [18, 19, 0, 1, 2] {
        set(i, 0);
    }
    // Four cells in a column (40 000 km²): mountain ice.
    for j in 3..7 {
        set(10, j);
    }
    // Two blocks of three meeting at a corner only: two bodies of 30 000 km².
    for (i, j) in [(5, 5), (6, 5), (6, 6), (7, 7), (8, 7), (7, 8)] {
        set(i, j);
    }
    let sheet = geo.sheet_cells(&|c| ice[c]);
    for i in [18, 19, 0, 1, 2] {
        assert!(sheet[i], "cell {i} of the seam's sheet");
    }
    let n_sheet = sheet.iter().filter(|&&s| s).count();
    assert_eq!(n_sheet, 5, "only the seam's body is a sheet");
    // One cell more on the column makes it a sheet.
    ice[7 * w + 10] = true;
    let sheet = geo.sheet_cells(&|c| ice[c]);
    assert!((3..8).all(|j| sheet[j * w + 10]));
    assert!(!sheet[5 * w + 5] && !sheet[7 * w + 7]);
}

/// Ben 2026-10-02, "Merge with ice sheets off": the flowing sheets stay off in the code's default
/// and in every built-in preset (game and pipeline) until the per-cell climate (P1-26) fixes the
/// banded climate's cold bias.
#[test]
fn flowing_ice_sheets_are_off_by_default_and_in_every_built_in_preset() {
    assert!(!ClimateParams::default().ice_dynamics);
    for (name, json) in crate::preset::BUILTIN
        .iter()
        .chain(crate::params::PIPELINE_BUILTIN.iter())
    {
        let v: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(
            v["climate"]["ice_dynamics"],
            serde_json::json!(false),
            "{name} turns the flowing ice sheets on"
        );
    }
}

/// With the sheets off nothing of the sheet reaches the world: the land ice (and so the flexure's
/// ice load and the ground it bends) is the equilibrium glaciers' of the model before the sheets,
/// whatever the sheet's own settings are. With the sheets on the same settings change the ice
/// (so the comparison can see the sheet at all).
#[test]
fn with_the_sheets_off_the_sheet_settings_change_nothing() {
    let run = |label: &str, dynamics: bool, odd_sheet: bool| -> crate::world::WorldState {
        let mut v: serde_json::Value = serde_json::from_str(&pipeline_json(3000, 20.0, 5)).unwrap();
        // A cool star, so the small planet has ice to compare; no present-day nudge.
        v["planet"]["star_flux_rel"] = serde_json::json!(0.97);
        v["climate"]["present_climate"] = serde_json::json!("natural");
        v["climate"]["ice_dynamics"] = serde_json::json!(dynamics);
        if odd_sheet {
            // Everything the sheet alone reads (the mass-balance gradients and cap also shape
            // `climate/ice_balance_m_yr`, which is published either way, so they stay).
            v["climate"]["ice_relax_kyr"] = serde_json::json!(300.0);
            v["climate"]["ice_calving_m_yr"] = serde_json::json!(0.0);
            v["climate"]["ice_grid_cells"] = serde_json::json!(64);
            v["climate"]["ice_flow_a"] = serde_json::json!(1.0e-15);
            v["climate"]["ice_isostasy_kyr"] = serde_json::json!(30.0);
        }
        let dir = temp_dir(&format!("sheet-{label}"));
        let s = session(&v.to_string(), &dir, 2);
        finish(&s);
        let world = s.checkpoint().unwrap().world;
        let _ = std::fs::remove_dir_all(&dir);
        world
    };
    let base = run("off", false, false);
    let odd = run("off-odd", false, true);
    let on = run("on-odd", true, true);
    for name in [
        field::ICE_THICKNESS,
        field::ICE_MAX_THICKNESS,
        field::GLACIER,
        field::ICE_BALANCE,
        crate::surface::field::ELEVATION_M,
    ] {
        assert_eq!(
            base.fields[name].data, odd.fields[name].data,
            "{name}: the sheet's settings reached the world with the sheets off"
        );
    }
    assert!(
        base.fields[field::ICE_THICKNESS]
            .data
            .iter()
            .any(|&x| x >= ICE_MIN_M),
        "the planet has no land ice to compare"
    );
    assert_ne!(
        base.fields[field::ICE_THICKNESS].data,
        on.fields[field::ICE_THICKNESS].data,
        "the sheets on changed nothing: the comparison cannot see them"
    );
}

/// The present's search with a cap (CLM-151, CLM-206), on a stand-in for the trials (the state is
/// reached from ln factor 2.0, ×7.4): with no cap the bracket stops at 2.8 and the halvings close in
/// on 2.0; with a cap of 1.75 no trial goes past it, the cap itself is the last tried, and the
/// search ends there short of the state. The forced states (`dir` −1) take no cap.
#[test]
fn the_present_search_stops_at_its_cap() {
    let run = |dir: f64, cap: f64, need: f64| {
        let mut seen = Vec::new();
        let (chosen, reached, trials) = present_search(
            0.0f64,
            dir,
            cap,
            &|x| x,
            &|t| *t,
            &|t| t.abs() >= need,
            &|_| true,
            &mut |t| seen.push(*t),
        );
        (chosen, reached, trials, seen)
    };
    // No cap: 0.35, 0.7, 1.4, 2.8 bracketed (2.8 reaches), then four halvings toward 2.0.
    let (x, reached, trials, seen) = run(1.0, f64::INFINITY, 2.0);
    assert!(reached);
    assert_eq!(trials, 1 + 4 + 4);
    assert_eq!(&seen[..4], &[0.35, 0.7, 1.4, 2.8]);
    assert!((2.0..2.1).contains(&x), "{x}");
    // A cap of 1.75: 0.35 and the cap, both short; the search ends at the cap.
    let cap = 1.75f64.ln();
    let (x, reached, trials, seen) = run(1.0, cap, 2.0);
    assert!(!reached);
    assert_eq!(seen, vec![0.35, cap]);
    assert_eq!(trials, 3);
    assert_eq!(x, cap);
    assert!(seen.iter().all(|&t| t <= cap));
    // Reached under the cap: the halvings stay between the last miss and the first reach.
    let (x, reached, _, seen) = run(1.0, cap, 0.5);
    assert!(reached && (0.5..=cap).contains(&x), "{x}");
    assert!(seen.iter().all(|&t| t <= cap));
    // A cap of 1 never warms: the natural end is all there is.
    let (x, reached, trials, seen) = run(1.0, 0.0, 2.0);
    assert!(!reached && x == 0.0 && trials == 1 && seen.is_empty());
    // The forced states go the other way, uncapped, down to ×10⁻⁴.
    let (x, reached, _, _) = run(-1.0, f64::INFINITY, 9.0);
    assert!(reached && (-9.2..=-9.0).contains(&x), "{x}");
    // Already there: nothing tried.
    let (x, reached, trials, seen) = run(1.0, cap, 0.0);
    assert!(reached && x == 0.0 && trials == 1 && seen.is_empty());
}

#[test]
fn the_present_climate_setting_governs_only_the_present() {
    let run = |state: &str| -> crate::world::WorldState {
        let mut v: serde_json::Value = serde_json::from_str(&pipeline_json(3000, 20.0, 7)).unwrap();
        v["climate"]["present_climate"] = serde_json::json!(state);
        let dir = temp_dir(&format!("present-{state}"));
        let s = session(&v.to_string(), &dir, 2);
        finish(&s);
        let world = s.checkpoint().unwrap().world;
        let _ = std::fs::remove_dir_all(&dir);
        world
    };
    let note = |w: &crate::world::WorldState| -> serde_json::Value {
        serde_json::from_str(&w.notes[STAGE_NAME]).unwrap()
    };
    let ocean_ice = |w: &crate::world::WorldState| -> f64 {
        let k = &w.fields[field::KOPPEN].data;
        let ice = &w.fields[field::SEA_ICE].data;
        let (mut s, mut n) = (0.0, 0.0);
        for c in 0..k.len() {
            if k[c] as u8 == koppen::SEA {
                s += f64::from(ice[c]);
                n += 1.0;
            }
        }
        s / n
    };
    let natural = run("natural");
    let snow = run("snowball");
    let temperate = run("temperate");
    let (nn, ns, nt) = (note(&natural), note(&snow), note(&temperate));
    // As simulated: no factor; this planet ends without a snowball.
    assert_eq!(nn["present"]["state"], "natural");
    assert_eq!(nn["present"]["co2_factor"], 1.0);
    assert!(ocean_ice(&natural) < 0.5);
    // A snowball: CO₂ taken down until the ocean froze over; the event is open and says why.
    assert_eq!(ns["present"]["state"], "snowball");
    assert_eq!(ns["present"]["reached"], true, "{}", ns["present"]);
    assert!(ns["present"]["co2_factor"].as_f64().unwrap() < 0.5);
    assert!(
        ocean_ice(&snow) > 0.9,
        "sea ice over {} of the ocean",
        ocean_ice(&snow)
    );
    let e = snow
        .events
        .iter()
        .find(|e| e.kind == SNOWBALL)
        .expect("the snowball's event");
    assert!(
        format!("{e:?}").contains("present_setting"),
        "the event names the setting: {e:?}"
    );
    assert!(!natural.events.iter().any(|e| e.kind == SNOWBALL));
    // Only the last 100 kyr differ: every plate step's row but the last, and the window's rows
    // before the stretch (2-kyr steps: the last 50), are the same run.
    let rows = |n: &serde_json::Value, key: &str| n[key]["rows"].as_array().unwrap().clone();
    let (sn, ss) = (rows(&nn, "series"), rows(&ns, "series"));
    assert_eq!(sn[..sn.len() - 1], ss[..ss.len() - 1]);
    let (wn, ws) = (rows(&nn, "window"), rows(&ns, "window"));
    assert_eq!(wn.len(), ws.len());
    assert_eq!(wn[..wn.len() - 50], ws[..ws.len() - 50]);
    assert_ne!(wn[wn.len() - 1], ws[ws.len() - 1]);
    // The default: no snowball, and no more land under ice than an interglacial's.
    assert_eq!(nt["present"]["state"], "temperate");
    assert_eq!(nt["present"]["reached"], true, "{}", nt["present"]);
    assert!(nt["present"]["land_ice_frac"].as_f64().unwrap() <= 0.15);
    assert!(ocean_ice(&temperate) < 0.5);
    // The present's report (2026-10-19): every trial's row, the natural end first, none past the
    // temperate cap, and where the sheets lie, each class's land share adding up to 1.
    let f = |x: &serde_json::Value| x.as_f64().unwrap();
    let pt = &nt["present"];
    let cap = f(&pt["co2_factor_max"]);
    assert_eq!(cap, ClimateParams::default().present_co2_factor_max);
    let rows = pt["trial_rows"]["rows"].as_array().unwrap();
    assert_eq!(rows.len() as u64, pt["trials"].as_u64().unwrap());
    assert_eq!(f(&rows[0][0]), 1.0);
    assert!(rows.iter().all(|r| f(&r[0]) <= cap + 1e-4), "{pt}");
    assert!(ns["present"]["co2_factor_max"].is_null());
    for key in ["sheets_by_height", "sheets_by_latitude"] {
        for part in ["natural_land_share", "land_share"] {
            let sum: f64 = pt[key][part].as_array().unwrap().iter().map(f).sum();
            assert!((sum - 1.0).abs() < 1e-9, "{key}.{part} adds to {sum}");
        }
        for part in ["natural_sheet_share", "sheet_share"] {
            let all = pt[key][part].as_array().unwrap();
            assert!(
                all.iter().all(|x| (0.0..=1.0).contains(&f(x))),
                "{key}.{part}"
            );
        }
    }
}

#[test]
fn the_climate_runs_in_the_coupled_pipeline_and_resumes() {
    let json = pipeline_json(3000, 60.0, 7);
    let dir = temp_dir("straight");
    let s = session(&json, &dir, 2);
    assert_eq!(
        s.stage_names(),
        [
            "plates",
            "climate",
            "surface",
            "volcanoes",
            "impacts",
            "structure",
            "deposits"
        ]
    );
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    // C = 16,384 at 64 blocks a cell: 256 × 82, the rows from z = −2,624, past the Lambert pole
    // at 2,607.6.
    let n = 256 * 82;
    for name in [
        field::T_MEAN,
        field::PRECIP,
        field::KOPPEN,
        field::T_WARM,
        field::T_COLD,
        field::ARIDITY,
        field::GLACIER,
        field::ELA,
        field::ICE_THICKNESS,
        field::ICE_MAX_THICKNESS,
        field::ICE_MAX_EXTENT,
        field::ICE_TIME_SHARE,
        field::SEA_ICE,
        field::ICE_BALANCE,
    ] {
        let f = world.fields.get(name).unwrap_or_else(|| panic!("{name}"));
        assert_eq!(f.data.len(), n, "{name}");
    }
    // The mass balance is the ice's alone, and finite.
    let balance = &world.fields[field::ICE_BALANCE].data;
    let on_ice = &world.fields[field::GLACIER].data;
    assert!(
        balance
            .iter()
            .zip(on_ice)
            .all(|(b, g)| b.is_finite() && (*g == 1.0 || *b == 0.0))
    );
    // The ice fields agree with each other: glaciers where the ice is, the maximum at least the
    // present, shares and cover in [0, 1].
    let ice = &world.fields[field::ICE_THICKNESS].data;
    let ice_max = &world.fields[field::ICE_MAX_THICKNESS].data;
    let glacier = &world.fields[field::GLACIER].data;
    let share = &world.fields[field::ICE_TIME_SHARE].data;
    let sea_ice = &world.fields[field::SEA_ICE].data;
    for c in 0..n {
        assert_eq!(glacier[c] == 1.0, ice[c] >= ICE_MIN_M, "cell {c}");
        assert!(ice_max[c] >= ice[c] - 1e-3 && ice[c] >= 0.0 && ice[c] <= 4000.0);
        assert!((0.0..=1.0).contains(&share[c]) && (0.0..=1.0).contains(&sea_ice[c]));
    }
    // The sea level the surface used is the climate's: the water, less the ice, over the ground.
    let sea = world.globals[SEA_LEVEL_GLOBAL];
    assert_eq!(world.surface.as_ref().unwrap().sea_level_m, sea);
    let t = &world.fields[field::T_MEAN].data;
    let p = &world.fields[field::PRECIP].data;
    assert!(
        t.iter().all(|x| (-90.0..60.0).contains(x)),
        "temperatures in range"
    );
    assert!(
        p.iter().all(|&x| (10.0..20_000.0).contains(&x)),
        "precipitation in range"
    );
    let k = &world.fields[field::KOPPEN].data;
    assert!(
        k.iter()
            .all(|&x| (0.0..=31.0).contains(&x) && x.fract() == 0.0)
    );
    assert!(k.iter().any(|&x| x < 31.0), "some land is classified");
    // The surface read the stage's climate, and the deposit frames carry it.
    let st = world.surface.as_ref().unwrap();
    let grid = st.grid.clone();
    let d = Dims::new(grid.w as usize, grid.h as usize, 64.0 * 300.0, true);
    let hh: Vec<f64> = st.elevation_m.iter().map(|&z| f64::from(z)).collect();
    let snap = crate::surface::climate::snapshot(
        &world,
        &d,
        &|j| lat_of_row(&grid, j as u32),
        &hh,
        st.sea_level_m,
    );
    assert!(snap.from_stage);
    // The last frame's precipitation (8-bit packed, averaged over the frame's steps) follows the
    // climate's, not the stand-in's.
    let hist = world.history.as_ref().unwrap();
    use crate::deposits::fields::FieldHistory;
    let frame = hist.frame(hist.frame_count() - 1).unwrap();
    let fp = &frame.fields["deeptime:precip_mm"];
    let mean = |v: &[f32]| v.iter().map(|&x| f64::from(x)).sum::<f64>() / v.len() as f64;
    let (mf, mc) = (mean(fp), mean(p));
    assert!(
        (mf - mc).abs() < 0.2 * mc,
        "frame {mf} vs climate {mc} mm/yr"
    );
    let note: serde_json::Value = serde_json::from_str(&world.notes[STAGE_NAME]).unwrap();
    // The due ones (the pipeline presets snapshot every 20 Myr since P1-19 #27, SCL-038), plus
    // those the clock's departure triggered (P1-26 design §4.2), which depend on the planet.
    assert_eq!(
        note["snapshots"].as_u64().unwrap(),
        4 + note["triggered_snapshots"].as_u64().unwrap(),
        "t = 2, 20, 40 and 60 Myr, and the triggered ones"
    );
    // The curve: one row per plate step; the ice-age window's rows every 2 kyr of the last 3 Myr;
    // the carbon cycle balanced (weathering follows degassing) at the end.
    let rows = note["series"]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 30);
    let window = note["window"]["rows"].as_array().unwrap();
    assert_eq!(window.len(), 1500);
    let last = rows.last().unwrap().as_array().unwrap();
    let col = |name: &str| {
        clock::SERIES_COLUMNS
            .iter()
            .position(|c| *c == name)
            .unwrap()
    };
    let (d, w) = (
        last[col("degassing_mol_yr")].as_f64().unwrap(),
        last[col("weathering_mol_yr")].as_f64().unwrap(),
    );
    assert!(
        (w / d - 1.0).abs() < 0.2,
        "degassing {d:e}, weathering {w:e}"
    );
    // The note's series keeps four significant figures (the notes' `round`): half a unit of the
    // fourth is at most 5·10⁻⁴ of the value (0.07 m at the −143 m this planet ends at).
    let noted = last[col("sea_level_m")].as_f64().unwrap();
    assert!(
        (noted - sea).abs() <= 5e-4 * sea.abs().max(1.0),
        "{noted} vs {sea}"
    );
    // Every step's land runoff and weathering kernel (snapshots and the steps carried between
    // them) stay physical: the mean under the wettest place's rain, the kernel within a hundred
    // times Earth's (2026-10-18: a conveyor sinking in one coarse cell took them to 3,710 m/yr and
    // 3.8·10¹⁴).
    for row in rows {
        let r = row.as_array().unwrap();
        let (runoff, kernel) = (
            r[col("runoff_land_m_yr")].as_f64().unwrap(),
            r[col("kernel_rel")].as_f64().unwrap(),
        );
        assert!(
            (0.0..WETTEST_M_YR).contains(&runoff) && (0.0..KERNEL_CEILING_REL).contains(&kernel),
            "step {:?}: runoff {runoff} m/yr, kernel {kernel} × Earth's",
            r[0]
        );
    }
    // The ice goes into the deposit frames.
    assert!(frame.fields.contains_key(history::ICE_THICKNESS));
    assert!(frame.fields.contains_key(history::ICE_COVER));

    // Resume from a checkpoint inside the group on another thread count: the same world.
    let dir2 = temp_dir("resumed");
    let s2 = session(&json, &dir2, 1);
    for _ in 0..7 {
        assert_eq!(s2.advance(Duration::ZERO).unwrap(), AdvanceStatus::Running);
    }
    let ck = s2.checkpoint().unwrap();
    let bytes = ck.to_bytes().unwrap();
    let ck = Checkpoint::from_bytes(Path::new("ck"), &bytes).unwrap();
    let s3 = Session::resume(
        ck,
        SessionOptions {
            out_dir: dir2.clone(),
            threads: 3,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap();
    finish(&s3);
    let got = s3.checkpoint().unwrap().world;
    for name in [
        field::T_MEAN,
        field::PRECIP,
        field::KOPPEN,
        field::D_LN_PRECIP,
        field::ICE_THICKNESS,
        field::ICE_MAX_THICKNESS,
        field::ICE_TIME_SHARE,
        field::SEA_ICE,
    ] {
        assert_eq!(
            got.fields[name], world.fields[name],
            "{name} differs after resuming"
        );
    }
    let series = |w: &crate::world::WorldState| -> serde_json::Value {
        let v: serde_json::Value = serde_json::from_str(&w.notes[STAGE_NAME]).unwrap();
        serde_json::json!([
            v["series"],
            v["window"],
            v["snowballs"],
            v["last"]["co2_ppm"]
        ])
    };
    assert_eq!(
        series(&got),
        series(&world),
        "the climate curve differs after resuming"
    );
    assert_eq!(got.globals, world.globals);
    assert_eq!(
        got.surface.as_ref().unwrap().elevation_m,
        world.surface.as_ref().unwrap().elevation_m
    );
    assert_eq!(got.history, world.history);
    assert_eq!(got.events.len(), world.events.len());
    for d in [dir, dir2] {
        let _ = std::fs::remove_dir_all(d);
    }
}

/// A [`snapshot::CellsDevice`] that notes, before handing the cells to the GPU, where the
/// model has a step a last-bit difference can cross: the belt latitude at 0 in some month (the
/// trades change sides, and with them the orographic field).
#[cfg(feature = "gpu")]
#[derive(Debug)]
struct Watched<'a> {
    dev: &'a dyn snapshot::CellsDevice,
    /// Per cell: the smallest |belt latitude| of its months and the height; the lapse rate.
    seen: std::sync::Mutex<(Vec<f64>, Vec<f64>, f64)>,
}

#[cfg(feature = "gpu")]
impl snapshot::CellsDevice for Watched<'_> {
    fn name(&self) -> String {
        self.dev.name()
    }

    fn host_bytes(&self, w: usize, h: usize, monthly: bool) -> u64 {
        self.dev.host_bytes(w, h, monthly)
    }

    fn cells(
        &self,
        ci: &snapshot::CellsInput<'_>,
        st: &mut snapshot::CellsStats,
    ) -> Result<snapshot::Cells, String> {
        let pb: Vec<f64> = (0..ci.w * ci.h)
            .map(|c| {
                let (i, phi) = (c % ci.w, ci.lat[c / ci.w]);
                (0..12)
                    .map(|m| (phi - belts::shift(phi, ci.itcz[i][m], ci.hemi[m])).abs())
                    .fold(f64::MAX, f64::min)
            })
            .collect();
        *self.seen.lock().unwrap() = (pb, ci.z.to_vec(), ci.gamma);
        self.dev.cells(ci, st)
    }
}

/// The largest and mean |GPU − CPU| of a field over the cells `keep` admits, and the largest
/// relative one (to |CPU|, at least `floor`).
#[cfg(feature = "gpu")]
fn gap(cpu: &[f32], dev: &[f32], floor: f64, keep: &dyn Fn(usize) -> bool) -> (f64, f64, f64) {
    assert_eq!(cpu.len(), dev.len());
    let (mut max, mut sum, mut rel, mut n) = (0.0f64, 0.0, 0.0f64, 0usize);
    for (c, (&a, &b)) in cpu.iter().zip(dev).enumerate() {
        if !keep(c) {
            continue;
        }
        assert!(b.is_finite(), "cell {c}: the GPU's value {b} (CPU {a})");
        let d = (f64::from(a) - f64::from(b)).abs();
        max = max.max(d);
        rel = rel.max(d / f64::from(a).abs().max(floor));
        sum += d;
        n += 1;
    }
    (max, sum / n.max(1) as f64, rel)
}

/// The snapshot's cells on the GPU (`gpu.rs`) against the CPU loop on a real planet's terrain
/// (20 Myr of the coupled pipeline on L1 at 16k, r 64), in today's climate and a cold one with
/// sea ice and glaciers: temperatures within 0.01 K everywhere; precipitation (annual and
/// monthly), aridity and the ELA within 1e-4 relative (the ELA, or 1 cm); the sea ice within
/// 1e-4; Köppen and the glacier flag equal; each but where a month of the cell sits on one of the
/// model's steps (`gpu.rs`, "Precision"). Several row chunks give the one chunk's results bit for
/// bit. Any adapter, lavapipe included; skipped without one.
#[cfg(feature = "gpu")]
#[test]
fn the_gpu_cells_stay_within_their_error_budget_of_the_cpu() {
    let g = match deeptime_gpu::shared(true) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("skipped: {e}");
            return;
        }
    };
    let whole = super::gpu::CellsGpu::new(g.clone()).unwrap();
    let chunked = super::gpu::CellsGpu::new(g).unwrap().with_chunk_cells(4096);
    let dir = temp_dir("gpu-cells");
    let s = session(&pipeline_json(3000, 20.0, 5), &dir, 2);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let _ = std::fs::remove_dir_all(&dir);
    let grid = GridDesc::for_planet("L1", 16384, 64).unwrap();
    let (w, h) = (grid.w as usize, grid.h as usize);
    let n = w * h;
    let lat: Vec<f64> = (0..h).map(|j| lat_of_row(&grid, j as u32)).collect();
    let z: Vec<f64> = world.fields[crate::surface::field::ELEVATION_M]
        .data
        .iter()
        .map(|&x| f64::from(x))
        .collect();
    assert_eq!(z.len(), n);
    let sea = world.surface.as_ref().unwrap().sea_level_m;
    let p = ClimateParams::default();
    for (what, star, co2) in [("today", 1.0, 280.0), ("cold", 0.97, 180.0)] {
        let inp = snapshot::Input {
            grid: &grid,
            d: Dims::new(w, h, 64.0 * 300.0, true),
            lat: &lat,
            h: &z,
            sea,
            p: &p,
            planet: Planet {
                star_flux_rel: star,
                co2_ppm: co2,
                ..planet()
            },
            monthly: true,
            ice_prev: None,
            koppen_prev: None,
            correction: &super::percell::Correction::default(),
        };
        let run = |dev: Option<&dyn snapshot::CellsDevice>| {
            snapshot::compute_on(
                &inp,
                &mut EbmState::default(),
                &[],
                &mut Default::default(),
                dev,
            )
        };
        let cpu = run(None);
        assert!(matches!(cpu.cells_on, snapshot::CellsRun::Cpu));
        let watched = Watched {
            dev: &whole,
            seen: Default::default(),
        };
        let dev = run(Some(&watched));
        let st = match &dev.cells_on {
            snapshot::CellsRun::Device(st) => *st,
            other => panic!("{what}: the cells did not run on the GPU: {other:?}"),
        };
        let many = run(Some(&chunked));
        let st_many = match &many.cells_on {
            snapshot::CellsRun::Device(st) => *st,
            other => panic!("{what}: the chunked cells did not run on the GPU: {other:?}"),
        };
        assert!(st_many.chunks > st.chunks, "{st_many:?} against {st:?}");
        assert_eq!(dev.t_mean, many.t_mean, "{what}: the chunks changed t_mean");
        assert_eq!(dev.precip, many.precip, "{what}: the chunks changed precip");
        assert_eq!(
            dev.aridity, many.aridity,
            "{what}: the chunks changed aridity"
        );
        assert_eq!(dev.ela, many.ela, "{what}: the chunks changed the ELA");
        assert_eq!(dev.koppen, many.koppen, "{what}: the chunks changed Köppen");
        assert_eq!(
            dev.glacier, many.glacier,
            "{what}: the chunks changed the glaciers"
        );
        assert_eq!(
            dev.sea_ice, many.sea_ice,
            "{what}: the chunks changed the sea ice"
        );
        assert_eq!(
            dev.t_sl_month, many.t_sl_month,
            "{what}: the chunks changed t_sl_month"
        );
        assert_eq!(
            dev.p_month, many.p_month,
            "{what}: the chunks changed p_month"
        );

        let (pb, zc, gamma) = watched.seen.into_inner().unwrap();
        let land: Vec<bool> = cpu.koppen.iter().map(|&k| k != koppen::SEA).collect();
        let land_n = land.iter().filter(|&&l| l).count();
        // The steps: the belt latitude at 0 in a month (the regimes swap), a month at 26.5 °C
        // (Thornthwaite hands over to Willmott) or at 0 °C (Thornthwaite's heat index of nearly
        // nothing).
        let on_belt_step = |c: usize| pb[c] < 1e-3;
        let on_pet_step = |c: usize| {
            (0..12).any(|m| {
                let t = f64::from(cpu.t_sl_month[m][c]) - gamma * zc[c];
                (t - 26.5).abs() < 0.01 || t.abs() < 0.01
            })
        };
        let all = |_: usize| true;
        let off_belt = |c: usize| !on_belt_step(c);
        let off_steps = |c: usize| !on_belt_step(c) && !on_pet_step(c);
        let sea_cell = |c: usize| !land[c];
        println!(
            "{what}: {w}x{h}, {land_n} land cells, {} on a belt step, {} on a PET step; \
             device {}: {st:?}; chunked: {st_many:?}",
            (0..n).filter(|&c| on_belt_step(c)).count(),
            (0..n).filter(|&c| on_pet_step(c)).count(),
            snapshot::CellsDevice::name(&whole),
        );
        // A field's gaps over the cells `keep` admits, within `abs` or `rel` of the CPU's (at
        // least `floor`).
        let check = |name: &str,
                     a: &[f32],
                     b: &[f32],
                     floor: f64,
                     keep: &dyn Fn(usize) -> bool,
                     abs: f64,
                     rel: f64| {
            let (max, mean, r) = gap(a, b, floor, keep);
            println!("{what} {name}: max {max:.3e}, mean {mean:.3e}, max relative {r:.3e}");
            assert!(
                max <= abs || r <= rel,
                "{what} {name}: max {max:e} (budget {abs:e}), relative {r:e} (budget {rel:e})"
            );
        };
        check("t_mean", &cpu.t_mean, &dev.t_mean, 1.0, &all, 0.01, 0.0);
        check("t_warm", &cpu.t_warm, &dev.t_warm, 1.0, &all, 0.01, 0.0);
        check("t_cold", &cpu.t_cold, &dev.t_cold, 1.0, &all, 0.01, 0.0);
        check(
            "precip",
            &cpu.precip,
            &dev.precip,
            1e-3,
            &off_belt,
            0.0,
            1e-4,
        );
        check(
            "aridity",
            &cpu.aridity,
            &dev.aridity,
            1e-3,
            &off_steps,
            0.0,
            1e-4,
        );
        check("ela", &cpu.ela, &dev.ela, 100.0, &off_belt, 0.01, 1e-4);
        check(
            "sea_ice",
            &cpu.sea_ice,
            &dev.sea_ice,
            1.0,
            &sea_cell,
            1e-4,
            0.0,
        );
        let (a, b) = (&cpu.sea_ice_max, &dev.sea_ice_max);
        check("sea_ice_max", a, b, 1.0, &sea_cell, 1e-4, 0.0);
        let (a, b) = (&cpu.sea_ice_min, &dev.sea_ice_min);
        check("sea_ice_min", a, b, 1.0, &sea_cell, 1e-4, 0.0);
        assert_eq!(cpu.t_sl_month.len(), 12);
        for (a, b) in cpu.t_sl_month.iter().zip(&dev.t_sl_month) {
            check("t_sl_month", a, b, 1.0, &all, 0.01, 0.0);
        }
        for (a, b) in cpu.p_month.iter().zip(&dev.p_month) {
            check("p_month", a, b, 1e-3, &off_belt, 0.0, 1e-4);
        }
        // Köppen and the glacier flag: equal, but where a criterion sits on its threshold within
        // the temperatures' and the rain's f32 noise.
        let koppen_off: Vec<usize> = (0..n)
            .filter(|&c| off_belt(c) && cpu.koppen[c] != dev.koppen[c])
            .collect();
        let glacier_off: Vec<usize> = (0..n)
            .filter(|&c| off_belt(c) && cpu.glacier[c] != dev.glacier[c])
            .collect();
        println!(
            "{what}: Köppen differs in {} cells, the glacier flag in {} (of {} glacier cells)",
            koppen_off.len(),
            glacier_off.len(),
            cpu.glacier.iter().filter(|&&g| g == 1).count()
        );
        assert!(
            koppen_off.len() <= (land_n / 1000).max(3),
            "{what}: Köppen differs in {koppen_off:?}"
        );
        for &c in &glacier_off {
            assert!(
                (zc[c] - f64::from(cpu.ela[c])).abs() < 0.05,
                "{what}: cell {c}'s glacier flag differs off its threshold: z {} ELA {} / {}",
                zc[c],
                cpu.ela[c],
                dev.ela[c]
            );
        }
        if what == "cold" {
            assert!(
                cpu.glacier.contains(&1) && cpu.sea_ice_max.iter().any(|&x| x > 0.5),
                "the cold climate has no ice to compare"
            );
        }
    }
}

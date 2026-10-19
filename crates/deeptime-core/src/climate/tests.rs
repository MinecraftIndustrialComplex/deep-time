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
    // The sea level the surface used is the climate's: the curve plus the ice's glacio-eustasy.
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
    assert!((last[col("sea_level_m")].as_f64().unwrap() - sea).abs() < 0.01);
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

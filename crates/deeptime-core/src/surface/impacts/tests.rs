// SPDX-License-Identifier: LGPL-3.0-or-later
//! The P2-08 checks: the number and sizes of impacts follow the rate; the sea stops small
//! impactors; craters ride their plates, fade with age and are counted when they go; large ones
//! are cut into the surface and leave breccia in the strata log, small ones leave bodies; the
//! events say what became of each; no flux, no impacts; the catalogue is the same on any thread
//! count and after a resume.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::*;

/// The visible relief at the default vertical scale, m.
const VM: f64 = VISIBLE_RELIEF_BLOCKS * deeptime_format::DEFAULT_M_PER_BLOCK_V;
use crate::checkpoint::Checkpoint;
use crate::params::planet_builtin;
use crate::preset::with_timeline;
use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};
use deeptime_format::ParamValue;

fn pipeline_json(n: u32, myr: f64, seed: u64, impacts: Option<serde_json::Value>) -> String {
    let json = with_timeline(
        planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(myr),
    )
    .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = json!(n);
    v["resolution"]["l1_r"] = json!(32);
    v["keyframes"]["size"] = json!([128, 64]);
    v["keyframes"]["every_myr"] = json!(20.0);
    v["seed"] = json!(seed);
    for k in ["deposits", "young", "amplify", "impacts"] {
        v.as_object_mut().unwrap().remove(k);
    }
    if let Some(i) = impacts {
        v["impacts"] = i;
    }
    v.to_string()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-impacts-{name}-{}-{:?}",
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

fn crater(d_km: f64, age_myr: f64) -> Crater {
    let set = scaling::Setting {
        gravity: scaling::G_EARTH,
        speed: 20_000.0,
        impactor_density: 3000.0,
        target_density: TARGET_DENSITY_KG_M3,
        sea_cutoff: scaling::SEA_CUTOFF_RATIO,
    };
    let l = scaling::impactor_of_earth_crater_m(&set, d_km * 1000.0);
    let made = scaling::crater(&set, l, 0.0).unwrap();
    let (profile, breccia_m, melt_m) = build(&made, &ImpactParams::default());
    Crater {
        id: 1,
        x: 0.0,
        z: 0.0,
        plate: 0,
        oceanic: false,
        t_myr: -age_myr,
        lat0: 0.0,
        lon0: 180.0,
        water_m: 0.0,
        impactor_m: l,
        energy_j: made.energy_j,
        diameter_m: made.diameter_m,
        transient_m: made.transient_m,
        profile,
        breccia_m,
        melt_m,
        melt_real_m: made.melt_m,
        melt_km3: made.melt_m3 / 1.0e9,
        imprinted: false,
        cover_m: 0.0,
        worn_m: 0.0,
        kt_m2: 0.0,
        relief_m: profile.depth_m,
        fate: Fate::Fresh,
        event: None,
        gone: None,
    }
}

/// Ben, 2026-10-02: "Go for a 3x default". The default is a game choice (Earth's rate is 1), and
/// every built-in preset that runs the stage states the same rate.
#[test]
fn the_default_rate_is_three_times_earths_in_every_builtin_preset() {
    assert_eq!(ImpactParams::default().flux, 3.0);
    let mut seen = 0;
    for (name, json) in crate::preset::BUILTIN
        .iter()
        .chain(crate::params::PIPELINE_BUILTIN.iter())
    {
        let v: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(v["impacts"]["flux"].as_f64(), Some(3.0), "{name}");
        seen += 1;
    }
    assert_eq!(seen, 8);
}

#[test]
fn size_classes_and_the_expected_rate() {
    assert_eq!(size_class(2.0), 0);
    assert_eq!(size_class(5.0), 1);
    assert_eq!(size_class(19.9), 2);
    assert_eq!(size_class(20.0), 3);
    assert_eq!(size_class(99.0), 4);
    assert_eq!(size_class(250.0), 5);
    let json = pipeline_json(12_000, 40.0, 1, Some(json!({})));
    let preset = match crate::preset::Preset::from_json(&json).unwrap().body {
        crate::preset::PresetBody::Planet(p) => (*p).clone(),
        _ => unreachable!(),
    };
    // A 16,384-block planet at 300 m per block: 7.7 million km², 1.5 % of Earth's surface.
    let area = planet_area_km2(&preset);
    assert!((area / 7.69e6 - 1.0).abs() < 0.01, "{area}");
    // Earth's own rate (flux 1); the default is 3 times it (the test above).
    let p = ImpactParams {
        flux: 1.0,
        ..ImpactParams::default()
    };
    // Craters of 2 km and more: 3.7 times the rate at 20 km (the record's shape below 20 km,
    // Hughes 2000), 0.158 per Myr on this planet; of 20 km and more, 5.6e-15 per km² per year, one
    // every 23 Myr. (The first coded rate carried the 1.8 slope down to 2 km: 2.7 per Myr.)
    let per_myr = impactors_per_myr(&preset, &p);
    assert!((per_myr - 0.158).abs() < 0.003, "{per_myr}");
    let big = ImpactParams {
        min_crater_km: 20.0,
        ..p.clone()
    };
    let one_in = 1.0 / impactors_per_myr(&preset, &big);
    assert!((one_in - 23.2).abs() < 0.5, "{one_in} Myr");
    let twice = ImpactParams {
        flux: 2.0,
        ..p.clone()
    };
    assert!((impactors_per_myr(&preset, &twice) / per_myr - 2.0).abs() < 1e-9);
    let none = ImpactParams { flux: 0.0, ..p };
    assert_eq!(impactors_per_myr(&preset, &none), 0.0);
}

#[test]
fn a_crater_fades_by_cover_wear_and_age() {
    let fresh = crater(3.0, 0.0);
    let (relief, fate) = fresh.state(VM);
    assert_eq!(fate, Fate::Fresh);
    assert!(
        (relief / fresh.profile.depth_m - 1.0).abs() < 0.06,
        "{relief}"
    );
    // Half filled: degraded, half the relief.
    let mut filled = fresh.clone();
    filled.cover_m = 0.5 * fresh.profile.depth_m;
    let (r, f) = filled.state(VM);
    assert_eq!(f, Fate::Degraded);
    assert!((r / relief - 0.5).abs() < 0.01);
    assert!((filled.lens_top_below_m() - filled.cover_m).abs() < 1e-9);
    // Filled to the rim and beyond: buried, its lens under the cover.
    filled.cover_m = 3.0 * fresh.profile.depth_m;
    assert_eq!(filled.state(VM), (0.0, Fate::Buried));
    assert!(!filled.worn_out());
    // Worn past its depth: its roots, the lens cut into; worn past the lens: gone.
    let mut worn = fresh.clone();
    worn.worn_m = fresh.profile.depth_m + 0.5 * fresh.breccia_m;
    assert_eq!(worn.state(VM), (0.0, Fate::Roots));
    assert!((worn.lens_top_below_m() + 0.5 * fresh.breccia_m).abs() < 1e-9);
    assert!(!worn.worn_out());
    worn.worn_m = fresh.profile.depth_m + 1.01 * fresh.breccia_m;
    assert!(worn.worn_out());
    // Diffusion alone: a 3-km bowl is subdued after 30 Myr and gone from view after 300; a 60-km
    // crater still stands after 300.
    let mut old = fresh.clone();
    old.kt_m2 = 0.01 * 30.0e6;
    let (r30, _) = old.state(VM);
    assert!(r30 < 0.7 * relief && r30 > 0.0, "{r30} of {relief}");
    old.kt_m2 = 0.01 * 300.0e6;
    let (r300, f300) = old.state(VM);
    assert!(r300 < r30 && f300 != Fate::Fresh, "{r300}");
    let mut big = crater(60.0, 0.0);
    let (b0, _) = big.state(VM);
    big.kt_m2 = 0.01 * 300.0e6;
    let (b300, _) = big.state(VM);
    assert!(b300 > 0.5 * b0, "{b300} of {b0}");
}

#[test]
fn fields_map_and_bodies_come_from_the_catalogue() {
    let grid = GridDesc::for_planet("L1", 16_384, 16).unwrap();
    let mh = 300.0;
    // A fresh 120-km crater at (0, 0), a 3-km one at (2000, 0), and an old 30-km one worn to its
    // roots at (-3000, 500).
    let mut big = crater(120.0, 1.0);
    big.imprinted = true;
    let mut small = crater(3.0, 1.0);
    small.x = 2000.0;
    small.id = 2;
    let mut roots = crater(30.0, 200.0);
    roots.x = -3000.0;
    roots.z = 500.0;
    roots.id = 3;
    roots.worn_m = roots.profile.depth_m + 1.0;
    let mut craters = vec![big.clone(), small.clone(), roots.clone()];
    for c in &mut craters {
        let (r, f) = c.state(VM);
        c.relief_m = r;
        c.fate = f;
    }
    let at = |v: &[f32], x: f64, z: f64| v[cell_of(&grid, x, z)];
    // Everything formed after −5 Myr is new to this frame: the two young craters.
    let f = frame_fields(&grid, mh, &craters, -5.0);
    let melt = &f[FIELD_MELT];
    assert!(
        at(melt, 0.0, 0.0) > 200.0,
        "a melt sheet: {}",
        at(melt, 0.0, 0.0)
    );
    assert_eq!(
        at(melt, 150.0, 0.0),
        0.0,
        "none outside the floor (45 km out)"
    );
    assert_eq!(at(melt, 2000.0, 0.0), 0.0, "a small bowl has no sheet");
    let hydro = &f[FIELD_HYDROTHERMAL];
    assert_eq!(at(hydro, 0.0, 0.0), 1.0);
    assert_eq!(at(hydro, 2000.0, 0.0), 0.0, "too small to circulate");
    assert_eq!(at(hydro, -3000.0, 500.0), 0.0, "long cold");
    let shock = &f[FIELD_SHOCK];
    assert_eq!(at(shock, 0.0, 0.0), 1.0);
    assert!(at(shock, 2000.0, 0.0) > 0.0 && at(shock, 2000.0, 0.0) < 0.1);
    assert_eq!(at(shock, 195.0, 0.0), 0.0, "outside 0.7 R");
    let basin = &f[FIELD_BASIN];
    assert_eq!(at(basin, 100.0, 0.0), 1.0);
    assert_eq!(at(basin, 2000.0, 0.0), 0.0, "under 5 km");
    assert_eq!(at(basin, -3000.0, 500.0), 0.0, "worn to its roots");
    assert_eq!(at(basin, 400.0, 0.0), 0.0);
    // The trap: complex craters whose structure remains.
    let trap = trap_field(&grid, mh, &craters);
    assert_eq!(at(&trap, 0.0, 0.0), 0.7);
    assert_eq!(at(&trap, 2000.0, 0.0), 0.0);
    assert_eq!(at(&trap, -3000.0, 500.0), 0.0);
    // The map.
    let m = map(&grid, mh, &craters);
    assert_eq!(at(&m, 0.0, 0.0), code::FRESH);
    assert_eq!(at(&m, 2000.0, 0.0), code::FRESH);
    assert_eq!(at(&m, -3000.0, 500.0), code::ROOTS);
    assert!(at(&m, 1000.0, 1000.0).is_nan());
    // The small crater's lens as a body: an ellipsoid of breccia about its floor.
    let b = to_body(&small, 7, 100.0, mh, 20.0, -352).expect("a lens five blocks thick");
    assert_eq!(b.role, "impact_breccia");
    assert_eq!(b.rock_class.as_deref(), Some("impact_breccia"));
    assert_eq!(b.center, [2000.0, 100.0, 0.0]);
    assert!((f64::from(b.axes[0]) - 0.95 * 1500.0 / 300.0).abs() < 1e-3);
    assert!((f64::from(b.axes[1]) - small.breccia_m / 20.0).abs() < 1e-3);
    assert!(b.zones.is_empty(), "no melt sheet in a bowl");
    assert!(b.validate().is_ok());
    // A complex crater's body carries its melt sheet on the floor.
    let mid = crater(30.0, 1.0);
    let b = to_body(&mid, 8, 60.0, mh, 20.0, -352).unwrap();
    assert_eq!(b.zones.len(), 1);
    assert_eq!(b.zones[0].rock_class.as_deref(), Some("impact_melt"));
    assert!(b.zones[0].v[0] == 0.5 && b.zones[0].v[1] > 0.5 && b.zones[0].v[1] <= 1.0);
    assert!(b.validate().is_ok());
    // Below the world: nothing.
    assert!(to_body(&small, 9, -5000.0, mh, 20.0, -352).is_none());
    // The present-day list.
    let st = ImpactState {
        craters,
        t_myr: 0.0,
        ..ImpactState::default()
    };
    let v = present_craters(&st);
    assert_eq!(v["version"], CRATERS_VERSION);
    let list = v["craters"].as_array().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0]["morphology"], "peak_ring");
    assert_eq!(list[1]["morphology"], "simple");
    assert_eq!(list[2]["fate"], "eroded_to_roots");
    assert_eq!(list[2]["age_myr"], 200.0);
}

#[test]
fn the_pipeline_makes_craters_at_the_rate_and_ages_them() {
    // Five hundred times Earth's rate, so that 120 Myr on this small planet hold thousands of
    // craters (Earth's rate gives it a few dozen in 500 Myr).
    let json = pipeline_json(20_000, 120.0, 11, Some(json!({"flux": 500.0})));
    let dir = temp_dir("rate");
    let s = session(&json, &dir, 2);
    assert_eq!(
        s.stage_names(),
        [
            "plates",
            "climate",
            "surface",
            "volcanoes",
            "impacts",
            "structure"
        ]
    );
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let st = world.impacts.as_ref().expect("the catalogue");
    let c = &st.counts;
    let formed: u64 = c.formed.iter().sum();
    let stopped: u64 = c.stopped_by_sea.iter().sum();
    assert_eq!(c.impactors, formed + stopped);
    // The count is a Poisson draw about rate × area × time.
    let expected = 500.0 * 0.158 * 120.0;
    assert!(
        (c.impactors as f64 - expected).abs() < 6.0 * expected.sqrt(),
        "{} impactors, {expected} expected",
        c.impactors
    );
    // More craters under 20 km than over it (the production function: 2.7 to 1 before the sea
    // shrinks any), and fewer of each size class above 20 km than the one below.
    let all = |k: usize| c.formed[k] + c.stopped_by_sea[k];
    assert!(
        all(0) + all(1) + all(2) > 2 * (all(3) + all(4) + all(5)),
        "{c:?}"
    );
    assert!(all(3) > 3 * all(4), "{c:?}");
    // The sea stops small impactors far more often than large ones.
    assert!(c.stopped_by_sea[0] > 0, "{c:?}");
    let share = |k: usize| c.stopped_by_sea[k] as f64 / all(k).max(1) as f64;
    assert!(share(0) > share(3), "{c:?}");
    assert!(share(0) > 0.3, "most of the planet is sea: {}", share(0));
    assert!(c.largest_km <= 1.3 * 300.0);

    // The catalogue: every crater's accounting is sane, and it is where its event says.
    let frame = BlockFrame::new(16_384);
    let grid = GridDesc::for_planet("L1", 16_384, 32).unwrap();
    assert!(!st.craters.is_empty());
    for cr in &st.craters {
        assert!(cr.cover_m >= 0.0 && cr.worn_m >= 0.0 && cr.kt_m2 >= 0.0);
        assert!((0.0..=1.0).contains(&cr.kept()));
        assert!(cr.relief_m <= 1.01 * cr.profile.depth_m, "{cr:?}");
        assert!(cr.gone.is_none() && !cr.worn_out());
        assert!(cr.t_myr <= st.t_myr);
        assert_eq!(cr.imprinted, cr.diameter_m >= 2.0 * 32.0 * 300.0);
    }
    // Craters fade: of the small ones on land, the young keep more of their relief than the old.
    let ratio = |lo: f64, hi: f64| {
        let v: Vec<f64> = st
            .craters
            .iter()
            .filter(|cr| {
                let age = st.t_myr - cr.t_myr;
                cr.diameter_m < 5000.0 && cr.water_m == 0.0 && age >= lo && age < hi
            })
            .map(|cr| cr.relief_m / cr.profile.depth_m)
            .collect();
        (v.iter().sum::<f64>() / v.len().max(1) as f64, v.len())
    };
    let (young, n_young) = ratio(0.0, 6.0);
    let (old, n_old) = ratio(60.0, 1.0e9);
    assert!(n_young > 5, "{n_young} young craters");
    assert!(
        young > old + 0.2,
        "young craters keep {young} of their relief, old ones {old} ({n_old})"
    );
    // … and many of those formed are gone or no longer show.
    let visible = st.craters.iter().filter(|cr| cr.relief_m >= VM).count() as u64;
    assert!(visible > 0 && visible < formed, "{visible} of {formed}");
    let gone: u64 = c
        .eroded
        .iter()
        .chain(&c.subducted)
        .chain(&c.overprinted)
        .sum();
    assert_eq!(formed, gone + st.craters.len() as u64);

    // Large craters are cut into the surface and lay breccia in the strata log.
    assert!(c.imprinted > 0, "{c:?}");
    let surf = world.surface.as_ref().unwrap();
    let breccia = index("impact_breccia");
    let cells_with = (0..grid.cells())
        .filter(|&cell| surf.log.column(cell).iter().any(|l| l.class == breccia))
        .count();
    assert!(cells_with > 0, "no impact breccia in the strata log");
    for cell in 0..grid.cells() {
        surf.log
            .check(cell, crate::synth::EVERY_CLASS.len())
            .unwrap();
    }
    // Small ones leave bodies.
    let lenses: Vec<&Body> = world
        .bodies
        .iter()
        .filter(|b| b.role == "impact_breccia")
        .collect();
    assert_eq!(lenses.len() as u64, st.bodies);
    assert!(!lenses.is_empty());
    for b in &lenses {
        assert_eq!(b.rock_class.as_deref(), Some("impact_breccia"));
        assert!(b.site.is_none() && b.validate().is_ok());
    }

    // Events: one per crater of 5 km or more and per smaller one that still shows; each says
    // what became of its crater, and a standing crater's event is where the crater is now.
    let events: Vec<&deeptime_format::Event> =
        world.events.iter().filter(|e| e.source == SOURCE).collect();
    assert_eq!(events.len() as u64, st.events_emitted);
    let craters_events = events.iter().filter(|e| e.kind == kind::CRATER).count() as u64;
    let big_formed: u64 = c.formed[1..].iter().sum();
    assert!(
        craters_events >= big_formed,
        "{craters_events} vs {big_formed}"
    );
    let t0 = -120.0e6;
    for e in &events {
        assert!(e.t_start >= t0 - 1.0 && e.t_start <= 1.0, "{}", e.t_start);
        assert!(matches!(e.place, Place::Point { .. }));
        if e.kind != kind::CRATER {
            assert_eq!(e.kind, kind::OCEAN);
            continue;
        }
        let Some(ParamValue::Str(fate)) = e.params.get("fate") else {
            panic!("no fate on {e:?}");
        };
        assert!(
            [
                "fresh",
                "degraded",
                "buried",
                "eroded_to_roots",
                "eroded",
                "subducted",
                "overprinted"
            ]
            .contains(&fate.as_str()),
            "{fate}"
        );
        let Some(ParamValue::Float(d)) = e.params.get("diameter_km") else {
            panic!("no diameter on {e:?}");
        };
        assert!(*d > 0.9 && *d < 400.0);
    }
    for cr in st.craters.iter().filter(|cr| cr.diameter_m >= 5000.0) {
        let id = cr.event.expect("an event for a crater of 5 km or more");
        let e = world.events.iter().find(|e| e.id == id).unwrap();
        let Place::Point { lat, lon } = e.place else {
            unreachable!()
        };
        let (x, z) = (
            frame.x_of_lon(f64::from(lon)),
            frame.z_of_lat(f64::from(lat)),
        );
        assert!(
            frame.wrap_x(x - cr.x).abs() < 2.0 && (z - cr.z).abs() < 2.0,
            "event at ({x}, {z}), crater at ({}, {})",
            cr.x,
            cr.z
        );
        assert_eq!(e.params.get("standing"), Some(&ParamValue::Int(1)), "{e:?}");
    }

    // The layers and the summary.
    let m = &world.fields[field::MAP].data;
    assert!(m.iter().any(|v| v.is_finite()));
    assert!(m.iter().all(|v| v.is_nan() || (1.0..=4.0).contains(v)));
    let trap = &world.fields[field::TRAP].data;
    assert!(trap.iter().all(|&v| v == 0.0 || v == 0.7));
    let note: Value = serde_json::from_str(&world.notes[STAGE_NAME]).unwrap();
    assert_eq!(note["formed"], formed);
    assert_eq!(note["visible"], visible);
    assert_eq!(note["catalogue"], st.craters.len());
    let line = summary_line(&note);
    assert!(line.starts_with("impacts_final="), "{line}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn no_flux_no_impacts() {
    let dir = temp_dir("none");
    let s = session(
        &pipeline_json(12_000, 30.0, 3, Some(json!({"flux": 0.0}))),
        &dir,
        2,
    );
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let st = world.impacts.as_ref().unwrap();
    assert_eq!(st.counts, Counts::default());
    assert!(st.craters.is_empty() && st.gone.is_empty());
    assert!(world.events.iter().all(|e| e.source != SOURCE));
    assert!(world.bodies.iter().all(|b| b.role != "impact_breccia"));
    assert!(world.fields[field::MAP].data.iter().all(|v| v.is_nan()));
    // The same world as one without the stage.
    let dir2 = temp_dir("without");
    let s2 = session(&pipeline_json(12_000, 30.0, 3, None), &dir2, 2);
    assert_eq!(
        s2.stage_names(),
        ["plates", "climate", "surface", "volcanoes", "structure"]
    );
    finish(&s2);
    let bare = s2.checkpoint().unwrap().world;
    assert!(bare.impacts.is_none());
    assert_eq!(
        bare.surface.as_ref().unwrap().elevation_m,
        world.surface.as_ref().unwrap().elevation_m
    );
    for d in [dir, dir2] {
        let _ = std::fs::remove_dir_all(d);
    }
}

#[test]
fn threads_and_a_resume_leave_the_craters_unchanged() {
    let json = pipeline_json(12_000, 40.0, 4, Some(json!({"flux": 680.0})));
    let dir = temp_dir("one");
    let s = session(&json, &dir, 1);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let dir2 = temp_dir("resumed");
    let s2 = session(&json, &dir2, 3);
    for _ in 0..9 {
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
    assert_eq!(got.impacts, world.impacts);
    assert!(!world.impacts.as_ref().unwrap().craters.is_empty());
    let ours = |w: &WorldState| -> Vec<deeptime_format::Event> {
        w.events
            .iter()
            .filter(|e| e.source == SOURCE)
            .cloned()
            .collect()
    };
    assert_eq!(ours(&got), ours(&world));
    assert_eq!(
        got.surface.as_ref().unwrap().elevation_m,
        world.surface.as_ref().unwrap().elevation_m
    );
    let bits = |w: &WorldState| -> Vec<u32> {
        w.fields[field::MAP]
            .data
            .iter()
            .map(|x| x.to_bits())
            .collect()
    };
    assert_eq!(bits(&got), bits(&world));
    for d in [dir, dir2] {
        let _ = std::fs::remove_dir_all(d);
    }
}

/// A crater shows while its relief is two blocks at the world's vertical scale (2026-10-02: a
/// fixed 40 m was two blocks only at 20 m a block): a bowl with 55 m of relief left shows on
/// 20 m blocks and not on 35 m ones.
#[test]
fn a_crater_shows_while_two_blocks_deep_at_any_vertical_scale() {
    assert_eq!(visible_relief_m(20.0), 40.0);
    assert_eq!(visible_relief_m(35.0), 70.0);
    let mut c = crater(3.0, 0.0);
    let (full, _) = c.state(visible_relief_m(35.0));
    // Fill it until 55 m of relief is left.
    c.cover_m = c.profile.depth_m * (1.0 - 55.0 / full);
    let (r20, f20) = c.state(visible_relief_m(20.0));
    let (r35, f35) = c.state(visible_relief_m(35.0));
    assert!((r20 - 55.0).abs() < 1.0 && r20 == r35, "{r20} {r35}");
    assert_eq!(f20, Fate::Degraded);
    assert_eq!(f35, Fate::Buried);
}

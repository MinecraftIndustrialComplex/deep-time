// SPDX-License-Identifier: LGPL-3.0-or-later
//! The P1-03 checks (plan §8.1 "Checks", §10.3 P1-03): area conserved across rifting, no sample
//! in two plates, crust ages non-negative, hull invariants after every resampling, a resumed run
//! ending in the identical world, and the statistical targets over 500 Myr.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::*;
use crate::checkpoint::Checkpoint;
use crate::params::plates_builtin;
use crate::preset::{Preset, PresetBody, StartConfig, with_timeline};
use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};

/// A small planet preset: `n` samples, `myr` of history, the given start.
fn preset_json(n: u32, myr: f64, start: &str, seed: u64) -> String {
    let json = with_timeline(
        plates_builtin("plates_quick_16k").unwrap(),
        Some(start),
        Some(myr),
    )
    .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = serde_json::json!(n);
    v["resolution"]["l1_r"] = serde_json::json!(64);
    v["keyframes"]["size"] = serde_json::json!([128, 64]);
    v["keyframes"]["every_myr"] = serde_json::json!(20.0);
    v["seed"] = serde_json::json!(seed);
    v.to_string()
}

fn preset(n: u32, myr: f64, start: &str, seed: u64) -> PlanetPreset {
    match Preset::from_json(&preset_json(n, myr, start, seed))
        .unwrap()
        .body
    {
        PresetBody::Planet(p) => *p,
        other => panic!("not a planet preset: {other:?}"),
    }
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-plates-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Steps a stage directly (no session), calling `check` after every step.
fn run_stage(
    p: &PlanetPreset,
    check: impl FnMut(&PlatesStage, &WorldState, &PlateState),
) -> WorldState {
    run_stage_with(PlatesStage::new(p.clone()), p, check)
}

/// [`run_stage`] on a stage the caller made (to switch the dynamics record off).
fn run_stage_with(
    mut stage: PlatesStage,
    p: &PlanetPreset,
    mut check: impl FnMut(&PlatesStage, &WorldState, &PlateState),
) -> WorldState {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();
    let mut world = WorldState::new(p.seed, p.scale.circumference_blocks, p.t_start_years());
    let log = crate::log::LogRing::default();
    let state = std::sync::atomic::AtomicU8::new(0);
    let gauge = NoGauge;
    while !stage.is_done() {
        let ctx = StageContext {
            pool: &pool,
            log: &log,
            out_dir: std::path::Path::new("."),
            until: std::time::Instant::now(),
            gauge: &gauge,
            mem_cap: u64::MAX,
            state: &state,
            history: None,
            needed_peak: None,
            compute: crate::compute::ComputeMode::Cpu,
        };
        stage.step(&mut world, &ctx).unwrap();
        let st = world.plates.as_ref().unwrap();
        check(&stage, &world, st);
    }
    world
}

/// The hull invariants of P1-02 on a lattice: Euler's formula, reciprocal neighbours, every
/// point a vertex.
fn check_hull(geo: &Geo) {
    let mesh = &geo.mesh;
    let (v, f) = (mesh.vertex_count() as i64, mesh.triangles().len() as i64);
    let e = mesh.edge_count() as i64;
    assert_eq!(v - e + f, 2, "V − E + F");
    assert!(mesh.skipped().is_empty());
    for (t, nb) in mesh.neighbors().iter().enumerate() {
        for &o in nb {
            assert!(
                mesh.neighbors()[o as usize].contains(&(t as u32)),
                "triangle {t} and {o} are not mutual neighbours"
            );
        }
    }
    // Every triangle is counter-clockwise seen from outside.
    for tri in mesh.triangles() {
        let [a, b, c] = tri.map(|i| mesh.points()[i as usize]);
        assert!((b - a).cross(c - a).dot(a + b + c) > 0.0);
    }
}

#[test]
fn rifting_conserves_area_exactly() {
    let p = preset(6000, 20.0, "supercontinent", 3);
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let (mut st, geo) = init::initial_state(&p, &env).unwrap();
    let pos = geometry::positions(&st, &geo);
    st.recount(&pos);
    let area = env.norm.sample_area_km2();
    let total_before: f64 = st.alive().map(|q| f64::from(q.samples) * area).sum();
    let sphere = 4.0 * std::f64::consts::PI * env.norm.model_km.powi(2);
    assert!((total_before - sphere).abs() / sphere < 1e-9);
    let big = st.alive().max_by_key(|q| q.samples).unwrap().id;
    let before = f64::from(st.plates[big as usize].samples) * area;
    let plates_before = st.plates.len();
    let mut world = WorldState::new(1, 16_384, -2.0e7);
    rifting::rift(&mut st, &mut world, &geo, &pos, &env, big, 3).unwrap();
    st.recount(&pos);
    assert!(st.plates.len() > plates_before, "the rift made plates");
    let mut after = f64::from(st.plates[big as usize].samples) * area;
    for q in &st.plates[plates_before..] {
        assert_eq!(q.parent, Some(big));
        after += f64::from(q.samples) * area;
    }
    assert!(
        (after - before).abs() / before < 1e-9,
        "rift changed the area: {before} → {after}"
    );
    let total_after: f64 = st.alive().map(|q| f64::from(q.samples) * area).sum();
    assert!((total_after - total_before).abs() / total_before < 1e-9);
    // Events: one rift, one birth per new plate, each birth caused by the rift.
    let rift = world
        .events
        .iter()
        .find(|e| e.kind == events::RIFT_ONSET)
        .unwrap();
    let births: Vec<_> = world
        .events
        .iter()
        .filter(|e| e.kind == events::PLATE_BIRTH)
        .collect();
    assert_eq!(births.len(), st.plates.len() - plates_before);
    assert!(births.iter().all(|b| b.causes == vec![rift.id]));
    assert!(stats::summary(&st).partition_ok);
}

#[test]
fn runs_keep_their_invariants() {
    for (start, seed) in [
        ("supercontinent", 1),
        ("scattered_continents", 2),
        ("random", 3),
    ] {
        let p = preset(5000, 160.0, start, seed);
        let mut resamplings = 0;
        let norm = PlatesStage::new(p.clone()).norm();
        let world = run_stage(&p, |stage, _, st| {
            let s = stats::summary(st);
            assert!(s.partition_ok, "{start}: a sample outside the alive plates");
            assert!(
                s.ocean_age_min_myr >= 0.0,
                "{start}: negative crust age {}",
                s.ocean_age_min_myr
            );
            for x in &st.samples {
                assert!(x.elevation_m.is_finite() && x.thickness_m.is_finite());
                assert!((-10_000.0..=10_000.0).contains(&x.elevation_m));
                assert!(x.pools.iter().all(|&v| v >= -1e-3 && v.is_finite()));
                if x.kind == OCEANIC {
                    assert!(f64::from(x.formed_myr) <= st.t_myr + 1e-3);
                }
                assert!((x.group as usize) < st.groups.len());
            }
            if st.counters.resamplings > resamplings {
                resamplings = st.counters.resamplings;
                let geo = &stage.cache.as_ref().unwrap().geo;
                assert_eq!(geo.lattice, st.lattice);
                check_hull(geo);
                // Right after a resampling every group is unrotated and every sample visible.
                assert!(st.samples.iter().all(|x| !x.is_hidden()));
            }
        });
        assert!(resamplings >= 2, "{start}: {resamplings} resamplings");
        let st = world.plates.as_ref().unwrap();
        let s = stats::summary(st);
        eprintln!("{start}: {} {:?}", s.line(), st.counters);
        // The L1 fields are there, on the L1 grid, and finite where they must be.
        for name in KEYFRAME_LAYERS {
            let f = &world.fields[name];
            // C = 16,384 at 64 blocks a cell: 256 × 82, the rows from z = −2,624, past the
            // Lambert pole at 2,607.6.
            assert_eq!((f.grid.w, f.grid.h), (256, 82), "{name}");
            assert_eq!(f.data.len(), 256 * 82, "{name}");
        }
        assert!(world.fields["elevation"].data.iter().all(|v| v.is_finite()));
        let ocean = world.fields["plates/crust_type"]
            .data
            .iter()
            .filter(|&&v| v == 0.0)
            .count() as f64
            / (256.0 * 128.0);
        assert!(
            (0.4..0.9).contains(&ocean),
            "{start}: L1 ocean fraction {ocean}"
        );
        // Every event is valid, and causes point backwards.
        for e in &world.events {
            e.validate().unwrap();
            for c in &e.causes {
                assert!(c.seq() < e.id.seq(), "{} caused by a later event", e.kind);
            }
        }
        let _ = norm;
    }
}

/// Bitwise equality of two worlds (NaN-safe: compared as postcard bytes), naming the first part
/// that differs instead of printing both worlds.
fn same_world(got: &WorldState, expect: &WorldState) {
    let bytes = |v: &dyn erased::Ser| v.bytes();
    let (a, b) = (
        got.plates.as_ref().unwrap(),
        expect.plates.as_ref().unwrap(),
    );
    for (i, (x, y)) in a.samples.iter().zip(&b.samples).enumerate() {
        assert!(
            bytes(x) == bytes(y),
            "sample {i} differs (step {}):\n{x:?}\n{y:?}",
            a.step
        );
    }
    assert!(bytes(&a.groups) == bytes(&b.groups), "groups differ");
    assert!(
        bytes(&a.plates) == bytes(&b.plates),
        "plates differ: {:?} vs {:?}",
        a.plates,
        b.plates
    );
    assert!(bytes(&a.contacts) == bytes(&b.contacts), "contacts differ");
    assert!(bytes(&a.hotspots) == bytes(&b.hotspots), "hotspots differ");
    assert!(bytes(&a.rng) == bytes(&b.rng), "rng differs");
    assert!(bytes(a) == bytes(b), "the plate state differs");
    assert_eq!(got.events.len(), expect.events.len(), "event counts differ");
    for (x, y) in got.events.iter().zip(&expect.events) {
        assert!(bytes(x) == bytes(y), "event differs:\n{x:?}\n{y:?}");
    }
    for (name, f) in &got.fields {
        assert!(
            bytes(f) == bytes(&expect.fields[name]),
            "field {name} differs"
        );
    }
    assert!(bytes(got) == bytes(expect), "the worlds differ");
}

mod erased {
    pub trait Ser {
        fn bytes(&self) -> Vec<u8>;
    }
    impl<T: serde::Serialize> Ser for T {
        fn bytes(&self) -> Vec<u8> {
            postcard::to_stdvec(self).unwrap()
        }
    }
}

fn session(json: &str, dir: &Path) -> Session {
    Session::create(
        json,
        "",
        SessionOptions {
            out_dir: dir.to_path_buf(),
            threads: 3,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap()
}

fn finish(s: &Session) {
    for _ in 0..100_000 {
        if s.advance(Duration::from_millis(50)).unwrap() == AdvanceStatus::Done {
            return;
        }
    }
    panic!("the run never finished");
}

/// A run checkpointed mid-stage through bytes and resumed ends in exactly the straight run's world.
#[test]
fn resumed_run_is_identical() {
    let json = preset_json(3000, 60.0, "scattered_continents", 9);
    let straight = session(&json, &temp_dir("straight"));
    finish(&straight);
    let expect = straight.checkpoint().unwrap().world;

    let dir = temp_dir("resumed");
    let s = session(&json, &dir);
    for _ in 0..13 {
        assert_eq!(s.advance(Duration::ZERO).unwrap(), AdvanceStatus::Running);
    }
    let ck = s.checkpoint().unwrap();
    assert_eq!(ck.next_stage_name(), Some(STAGE_NAME));
    assert!(!ck.stage_state.is_empty());
    let bytes = ck.to_bytes().unwrap();
    let ck = Checkpoint::from_bytes(Path::new("ck"), &bytes).unwrap();
    let s = Session::resume(
        ck,
        SessionOptions {
            out_dir: dir.clone(),
            threads: 2,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap();
    finish(&s);
    let got = s.checkpoint().unwrap().world;
    assert_eq!(got.t_years, 0.0);
    same_world(&got, &expect);
    let manifest: serde_json::Value = serde_json::from_str(&s.finalize_write().unwrap()).unwrap();
    assert_eq!(manifest["source"], "run:plates_quick_16k");
    assert!(dir.join("keyframes/index.json").is_file());
    for d in [dir, temp_dir("straight")] {
        let _ = std::fs::remove_dir_all(d);
    }
}

#[test]
fn a_preset_without_a_timeline_is_refused() {
    let err = Session::create(
        plates_builtin("plates_quick_16k").unwrap(),
        "",
        SessionOptions {
            out_dir: temp_dir("refused"),
            threads: 1,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    );
    match err {
        Err(CoreError::PresetInvalid { path, .. }) => assert_eq!(path, "$.timeline.start"),
        Err(e) => panic!("{e}"),
        Ok(_) => panic!("a preset without a start was accepted"),
    }
    let est: serde_json::Value = serde_json::from_str(
        &crate::session::estimate(&preset_json(100_000, 500.0, "random", 1), "").unwrap(),
    )
    .unwrap();
    assert_eq!(est["stages"][0]["name"], "plates");
    let secs = est["stages"][0]["seconds"].as_f64().unwrap();
    // The plan's §8.1 model: 250 × 0.28 s + 13 × (1.22 + 0.21) s ≈ 88.6 s.
    assert!((secs - 88.59).abs() < 0.1, "{secs}");
    assert_eq!(StartConfig::from_name("random"), Some(StartConfig::Random));
}

/// The statistical targets of plan §8.1 over 500 Myr: oceanic crust mostly younger than about
/// 250 Myr, and a few large plates plus small ones. At 20k samples; the 100k run is the CLI's.
#[test]
#[ignore = "500 Myr at 20k samples; run with --release -- --ignored"]
fn statistics_over_500_myr() {
    for (start, seed) in [
        ("supercontinent", 11),
        ("scattered_continents", 12),
        ("random", 13),
    ] {
        let p = preset(20_000, 500.0, start, seed);
        let world = run_stage(&p, |_, _, _| {});
        let st = world.plates.as_ref().unwrap();
        let s = stats::summary(st);
        eprintln!("{start}: {} {:?}", s.line(), st.counters);
        assert!(s.partition_ok);
        assert!(
            s.ocean_younger_250 >= 0.8,
            "{start}: only {:.2} of the ocean floor is younger than 250 Myr",
            s.ocean_younger_250
        );
        assert!(s.plates >= 6, "{start}: {} plates", s.plates);
        assert!(
            s.plates_at_least(0.05) >= 3,
            "{start}: too few large plates"
        );
        assert!(
            s.areas.iter().filter(|a| a.1 < 0.03).count() >= 2,
            "{start}: too few small plates"
        );
        assert!(
            s.areas[0].1 < 0.6,
            "{start}: one plate covers {}",
            s.areas[0].1
        );
        assert!(st.counters.rifts >= 1 && st.counters.subduction_onsets >= 3);
    }
}

/// P1-17: with `continental_shelf_share` s the continental crust is (1 − ocean_fraction)/(1 − s)
/// of the planet, its dry part still (1 − ocean_fraction), the rest a shelf from 0 m at the coast
/// to the shelf break at the ocean's edge; s = 0 keeps every continent dry (P1-03).
#[test]
fn the_shelf_share_floods_the_continents_margins() {
    for start in ["supercontinent", "scattered_continents", "random"] {
        let mut p = preset(6000, 20.0, start, 5);
        let env0 = PlatesStage::new(p.clone()).env(&p.tectonics, 1);
        let (st0, _) = init::initial_state(&p, &env0).unwrap();
        let n = st0.samples.len() as f64;
        let dry = (1.0 - p.planet.ocean_fraction) * n;
        let cont0 = st0.samples.iter().filter(|s| s.is_continental()).count() as f64;
        assert!((cont0 - dry).abs() <= 1.0, "{start}: {cont0} vs {dry}");
        assert!(
            st0.samples
                .iter()
                .filter(|s| s.is_continental())
                .all(|s| s.elevation_m > 0.0)
        );

        p.tectonics.continental_shelf_share = 0.29;
        let env = PlatesStage::new(p.clone()).env(&p.tectonics, 1);
        let (st, _) = init::initial_state(&p, &env).unwrap();
        let cont: Vec<&state::Sample> = st.samples.iter().filter(|s| s.is_continental()).collect();
        let want = dry / 0.71;
        assert!(
            (cont.len() as f64 - want).abs() <= 1.0,
            "{start}: {} continental samples, want {want}",
            cont.len()
        );
        let wet = cont.iter().filter(|s| s.elevation_m <= 0.0).count() as f64;
        assert!(
            (cont.len() as f64 - wet - dry).abs() <= 1.0,
            "{start}: dry continental samples"
        );
        assert!(
            cont.iter()
                .all(|s| s.elevation_m > 0.0 || (-130.0..=0.0).contains(&s.elevation_m)),
            "{start}: the shelf lies between 0 and −130 m"
        );
    }
}

/// P1-27: the margins are measured every 10 Myr into `notes.plates.margins`; at the start, with no
/// subduction zone yet, every coast is passive or another plate boundary, and the classes add up.
#[test]
fn margins_are_measured_and_add_up() {
    let p = preset(6000, 40.0, "scattered_continents", 7);
    let env = PlatesStage::new(p.clone()).env(&p.tectonics, 1);
    let (st, geo) = init::initial_state(&p, &env).unwrap();
    let pos = geometry::positions(&st, &geo);
    let (m, class) = stats::margin_classes(&st, &pos, &env.norm, &|_| false);
    assert!(m.coast > 0 && m.coast_km > 0.0);
    assert_eq!(
        m.subduction + m.collision,
        0,
        "no subduction zone at the start"
    );
    assert_eq!(m.passive + m.boundary + m.subduction + m.collision, m.coast);
    assert_eq!(class.len(), st.samples.len());
    assert_eq!(
        class
            .iter()
            .filter(|&&c| c & !stats::margin_class::FRONT == stats::margin_class::PASSIVE)
            .count() as u32,
        m.passive
    );
    let world = run_stage(&p, |_, _, _| {});
    let note: serde_json::Value = serde_json::from_str(&world.notes["plates"]).unwrap();
    let list = note["margins"].as_array().unwrap();
    assert_eq!(list.len(), 4, "one record every 10 Myr of 40");
    for r in list {
        let r: stats::Margins = serde_json::from_value(r.clone()).unwrap();
        assert_eq!(r.passive + r.boundary + r.subduction + r.collision, r.coast);
        assert!(r.andean <= r.subduction && r.initiated <= r.subduction);
    }
}

/// P1-27: a continent too large to be carried under sutures its whole plate to the receiving one
/// (the plate dies "sutured", owns nothing, and its margins keep their ocean); subduction
/// initiations never tear a continent (only slivers stranded in the old floor ride along); a
/// suture leaves no coast behind.
#[test]
fn sutures_and_initiations_keep_continents_whole() {
    let mut sutures = 0;
    let mut initiations = 0;
    // Seeds where the force balance welds a collision within 300 Myr on this small planet (a
    // sweep of 32 runs, 2026-10-16, after the eclogite root and whole-column isostasy: 8 welded;
    // with PLT-293, ocean crust keeping its own rock, 2026-10-20: supercontinent 1, 4, 9 and 12 of
    // seeds 1 to 12, scattered continents 12 of seeds 1 to 24, seed 19 three times, and neither
    // supercontinent 6 nor scattered 9 any more; with PLT-330 and PLT-331, the arcs' fronts on
    // the converging boundary and the arc's basement down with the slab, the same day: one weld
    // each on supercontinent 3, 4, 9, 12, 14, 16, 19 and 23 of seeds 1 to 24, scattered as in
    // `collisions_weld_on_stall_and_transfer_on_shortening`, and neither supercontinent 1 nor
    // scattered 19 any more; every run of the 48 initiated subduction; with gap 4's thermal state,
    // strength, β and root removal on by default, 2026-10-23, neither supercontinent 4 nor
    // scattered 1 welded, so scattered 12, which welds twice in that test's sweep; with V1 gap
    // 2's stages on by default, 2026-10-23, supercontinent 5, 8, 9, 14 and 19 welded of seeds 1
    // to 24 and supercontinent 4 not, so supercontinent 5; scattered 12 still welds once).
    for (start, seed) in [("supercontinent", 5), ("scattered_continents", 12)] {
        let p = preset(8000, 300.0, start, seed);
        let mut seen = 0usize;
        let world = run_stage(&p, |stage, world, st| {
            assert!(stats::summary(st).partition_ok);
            // A plate born of a subduction initiation this step holds no continental crust that
            // touches its parent's: a continent is never torn (a sliver stranded in the old
            // floor may ride along whole).
            let geo = &stage.cache.as_ref().unwrap().geo;
            for e in &world.events[seen..] {
                if e.params.get("origin")
                    != Some(&deeptime_format::ParamValue::Str(
                        "subduction_initiation".into(),
                    ))
                {
                    continue;
                }
                let (
                    Some(deeptime_format::ParamValue::Int(id)),
                    Some(deeptime_format::ParamValue::Int(parent)),
                ) = (e.params.get("plate"), e.params.get("parent"))
                else {
                    panic!("an initiation without its plates");
                };
                if geo.lattice != st.lattice || st.resample_step == st.step {
                    continue;
                }
                for i in 0..st.samples.len() {
                    if st.plate_of(i) != *id as u32 || !st.samples[i].is_continental() {
                        continue;
                    }
                    for (u, _) in geo.edges(i as u32) {
                        let o = &st.samples[u as usize];
                        assert!(
                            !(o.is_continental() && st.plate_of(u as usize) == *parent as u32),
                            "{start}: an initiation tore a continent"
                        );
                    }
                }
            }
            seen = world.events.len();
            for e in world
                .events
                .iter()
                .filter(|e| e.kind == events::PLATE_DEATH)
            {
                if e.params.get("fate") == Some(&deeptime_format::ParamValue::Str("sutured".into()))
                {
                    let Some(deeptime_format::ParamValue::Int(id)) = e.params.get("plate") else {
                        panic!("a sutured plate without its id");
                    };
                    let id = *id as u32;
                    assert!(!st.plates[id as usize].alive);
                    assert!(
                        (0..st.samples.len()).all(|i| st.plate_of(i) != id),
                        "a sutured plate still owns crust"
                    );
                }
            }
        });
        let st = world.plates.as_ref().unwrap();
        let sutured = world
            .events
            .iter()
            .filter(|e| {
                e.params.get("fate") == Some(&deeptime_format::ParamValue::Str("sutured".into()))
            })
            .count() as u32;
        assert_eq!(sutured, st.counters.sutures, "{start}");
        sutures += st.counters.sutures;
        for e in world.events.iter().filter(|e| {
            e.params.get("origin")
                == Some(&deeptime_format::ParamValue::Str(
                    "subduction_initiation".into(),
                ))
        }) {
            initiations += 1;
            let Some(deeptime_format::ParamValue::Float(f)) = e.params.get("continental_frac")
            else {
                panic!("an initiation without its continental fraction");
            };
            assert!(
                *f < 0.5,
                "{start}: an initiation's plate is {f} continental"
            );
        }
        for e in world.events.iter().filter(|e| {
            e.kind == events::COLLISION
                && e.params.get("sutured") == Some(&deeptime_format::ParamValue::Int(1))
        }) {
            assert_eq!(
                e.params.get("coast_split_km"),
                Some(&deeptime_format::ParamValue::Float(0.0)),
                "a suture leaves no coast behind"
            );
        }
    }
    assert!(sutures > 0, "no collision sutured in two 300-Myr runs");
    assert!(
        initiations > 0,
        "no subduction initiation in two 300-Myr runs"
    );
}

/// P1-27: with `cordillera_shortening_mm_yr` the continental crust over an arc stands higher than
/// without it, over the same history (since plate task 8 by the crust the shortening thickens it
/// with, taken from the plate behind the arc).
#[test]
fn arcs_build_a_cordillera() {
    let mean_arc = |shortening: f64| {
        let mut p = preset(8000, 60.0, "scattered_continents", 31);
        p.tectonics.cordillera_shortening_mm_yr = shortening;
        let mut sum = (0.0, 0usize);
        run_stage(&p, |_, _, st| {
            for s in &st.samples {
                if s.is_continental() && s.flags & state::ARC != 0 {
                    sum.0 += f64::from(s.elevation_m);
                    sum.1 += 1;
                }
            }
        });
        assert!(sum.1 > 0, "no continental arc in 60 Myr");
        sum.0 / sum.1 as f64
    };
    let (flat, range) = (mean_arc(0.0), mean_arc(6.0));
    // Plates start at rest since the switch to the force balance, so 60 Myr hold less convergence
    // than they did under the old speed rules (which asserted 300 m).
    assert!(
        range > flat + 150.0,
        "arc crust {range:.0} m with the cordillera vs {flat:.0} m without"
    );
}

/// The life cycle of the collision states over a run (`shortening.rs`): checked after every step
/// and against the dynamics record at the end.
#[derive(Default)]
struct CollisionLives {
    /// By (subducting plate, overriding plate, onset step): the shortening last seen.
    states: BTreeMap<(u32, u32, u64), f64>,
    /// The most steps a state has lived.
    longest: u64,
}

impl CollisionLives {
    fn step(&mut self, st: &PlateState) {
        for c in &st.contacts {
            let Some(col) = &c.collision else { continue };
            assert!(
                col.onset_step >= c.onset_step,
                "a collision before its contact"
            );
            assert!(col.onset_step <= col.last_step && col.last_step <= st.step);
            assert!((col.onset_myr - col.last_myr) <= 1e-9 && col.last_myr <= st.t_myr + 1e-9);
            assert!(col.shortening_km >= 0.0 && col.area_km2 >= 0.0);
            assert!(
                col.onset_convergence_mm_yr >= shortening::BEGIN_MM_YR,
                "a collision begins converging"
            );
            assert!(col.peak_convergence_mm_yr >= col.convergence_mm_yr);
            assert!(col.front_km > 0.0 && col.peak_front_km >= col.front_km);
            assert!((0.0..=1.0).contains(&col.continental_over));
            assert_eq!(
                col.later_convergence_mm_yr.is_some(),
                col.last_myr - col.onset_myr >= shortening::SLOWDOWN_AFTER_MYR - 1e-9
            );
            let (at, normal) = (
                DVec3::from_array(col.centroid),
                DVec3::from_array(col.normal),
            );
            assert!((at.length() - 1.0).abs() < 1e-9);
            assert!(normal.length() < 1.0 + 1e-9 && normal.dot(at).abs() < 1e-9);
            // Shortening never falls while the state lives, and a state is one life: its onset
            // never moves.
            let seen = self
                .states
                .entry((c.under, c.over, col.onset_step))
                .or_insert(0.0);
            assert!(col.shortening_km >= *seen - 1e-12, "shortening fell");
            *seen = col.shortening_km;
            self.longest = self.longest.max(st.step - col.onset_step + 1);
        }
    }

    /// Every collision state ended exactly once: in a transfer or weld (a collision record that
    /// carries the state's onset) or without one (a parting record; the run's end closes those
    /// still under way).
    fn finish(&self, st: &PlateState, d: &dynamics::Dynamics) {
        assert!(st.contacts.is_empty(), "the run's end closes every contact");
        assert!(st.counters.collision_onsets > 0);
        let with_state = d
            .collisions
            .iter()
            .filter(|c| c.onset_convergence_mm_yr.is_some())
            .count();
        assert_eq!(
            (with_state + d.partings.len()) as u32,
            st.counters.collision_onsets
        );
        assert!(self.states.len() as u32 <= st.counters.collision_onsets);
        assert!(
            self.longest >= 2,
            "no collision state outlived its first step"
        );
        for r in &d.partings {
            assert!(r.end_myr >= r.start_myr && r.start_myr >= r.contact_onset_myr - 1e-9);
            assert!(r.shortening_km >= 0.0 && r.area_km2 >= 0.0 && r.front_km > 0.0);
            assert!(r.peak_convergence_mm_yr > 0.0);
            assert!(matches!(
                r.reason.as_str(),
                dynamics::PARTED
                    | dynamics::REVERSED
                    | dynamics::CLOSED
                    | dynamics::EMPTIED
                    | dynamics::OPEN
            ));
        }
        for c in &d.collisions {
            assert!(c.end_myr >= c.start_myr && c.start_myr >= c.contact_onset_myr - 1e-9);
            match c.onset_convergence_mm_yr {
                // A collision of a state: it began converging, and what crossed its front is
                // its shortening times a front.
                Some(v) => {
                    assert!(v > 0.0);
                    assert_eq!(c.front_shortening_km > 0.0, c.area_km2 > 0.0);
                }
                // No state (only while the pre-task-2 count fires): a collision of one step.
                None => {
                    assert_eq!(c.start_myr, c.end_myr);
                    assert_eq!((c.front_shortening_km, c.area_km2), (0.0, 0.0));
                }
            }
        }
        // A belt record per collision that had a state, with its convergence at the onset.
        let belts = d
            .collisions
            .iter()
            .filter(|c| c.collision && c.onset_convergence_mm_yr.is_some())
            .count();
        assert_eq!(d.belts.len(), belts);
        for b in &d.belts {
            assert!(b.convergence_onset_mm_yr.is_some_and(|v| v > 0.0));
            assert!(b.end_myr.is_some_and(|t| t >= b.start_myr));
            assert!(b.wedge_width_km.is_some_and(|w| w >= 0.0));
            assert!(b.peak_height_m.is_some_and(|h| h >= 0.0));
            assert!(b.belt_width_km.is_none_or(|w| w > 0.0));
            // The plate-only presets have no surface stage, so no flexure to read a basin from.
            assert!(b.foredeep_width_km.is_none() && b.foredeep_depth_m.is_none());
        }
    }
}

/// Tasks 3 and 4: a collision builds its belt over many steps and ends in a weld only once it
/// has stalled (the larger plate keeping its id), or, for a small terrane, in a transfer once it
/// has shortened by the threshold; the states' life cycle holds; and collisions still happen.
#[test]
fn collisions_weld_on_stall_and_transfer_on_shortening() {
    // A seed whose collisions weld in 300 Myr (under the force balance a held collision welds less
    // often than under the old speed law: 8 of 32 welded in the sweep of 2026-10-16, and 12 of
    // seeds 1 to 24 under the thickening limit, `rate.rs`, 2026-10-18; 7 of seeds 1 to 24 once
    // crossed crust stays under, PLT-289 to PLT-292, the same day: 3, 7, 8, 12, 15, 20 and 21;
    // 14 of seeds 3 to 24 with the lower-crustal flow and the settling, PLT-300 and ISO-078,
    // 2026-10-20: 5 to 9, 11 to 14, 18 to 21 and 24, seed 14 three times, seed 3 no more; 12 of
    // seeds 1 to 24 with PLT-293, ocean crust keeping its own rock, the same day: 1, 4, 6, 8, 10
    // to 13, 15, 16, 18 and 19, seed 19 three times, seed 14 no more; 7 of seeds 1 to 24 with the
    // arcs' fronts on the converging boundary and the arc's basement down with the slab, PLT-330
    // and PLT-331, the same day: 1, 2, 8, 14, 21, 22 and 24, seed 1 three times, seed 19 no more:
    // fewer arcs reach a continent; 12 of seeds 1 to 24 with the high ground's sheet and the
    // trench budget on by default, PLT-412 to PLT-415 and PLT-440 to PLT-443, 2026-10-23: 2, 5,
    // 6, 7, 9, 10, 13, 15, 16, 18, 19 and 24, seed 1 no more; 13 of seeds 1 to 24 with gap 4's
    // thermal state, strength, β and root removal on by default, 2026-10-23: 2, 5, 6, 11 to 13,
    // 15, 16, 18 to 20, 22 and 23, seeds 12, 18 and 19 twice, seeds 7, 9, 10 and 24 no more; 7
    // of seeds 1 to 24 with V1 gap 2's ridge segments, initiation off the passive margin, stress
    // weights, rifts along thin lithosphere and the reversal by the arc's crust on by default,
    // 2026-10-23: 4, 12, 14, 17, 18, 19 and 21, seed 17 twice, seed 2 no more; 11 of seeds 1 to
    // 24 with the fronts by their straight share on by default, PLT-530, 2026-10-24, the sweep
    // `weld_seed_sweep`: 2, 6, 9, 11 to 13, 15, 16 and 18 to 20, seeds 12, 15 and 19 twice, seeds
    // 4, 14, 17 and 21 no more; this test and the recording test pass on all 11; 9 of seeds 1 to
    // 24 with fission and the split inheritance on by default, PLT-532 and PLT-534, the same day:
    // 1, 2, 3, 6, 11, 14, 15, 20 and 21, seeds 12 and 19 no more; this test passes on all 9, seed
    // 11 since; 11 of seeds 1 to 24 with a split's latent slab copies, PLT-534, the same day: 1,
    // 2, 8, 11 to 14, 16, 20, 22 and 23, seed 12 again since; 12 of seeds 1 to 24 with the level
    // set on by default, PLT-536, the same day: 1, 2, 4, 5, 7, 9, 12, 15, 17 to 19 and 21; 12 of
    // seeds 1 to 24 with the contact carry on by default, PLT-550, the same day: 1, 4, 5, 7, 8,
    // 10 to 12, 18, 20, 23 and 24, seed 1 three times, seeds 2, 9, 15, 17, 19 and 21 no more).
    let p = preset(8000, 300.0, "scattered_continents", 12);
    let mut life = CollisionLives::default();
    let world = run_stage(&p, |_, _, st| {
        assert!(stats::summary(st).partition_ok);
        life.step(st);
    });
    let st = world.plates.as_ref().unwrap();
    let d = st.dynamics.as_ref().unwrap();
    life.finish(st, d);
    assert!(!d.collisions.is_empty(), "no transfer or weld in 300 Myr");
    assert_eq!(d.collisions.len() as u32, st.counters.accretions);
    for c in &d.collisions {
        assert!(c.onset_convergence_mm_yr.is_some(), "fired with no state");
        if c.sutured {
            assert!(c.stalled, "a weld before the collision stalled");
            let died = c.died.expect("a weld names the plate that died");
            let (smaller, larger) = if c.under_samples > c.over_samples {
                (c.over, c.under)
            } else {
                (c.under, c.over)
            };
            assert!(
                died == smaller || c.under_samples == c.over_samples,
                "plate {died} died, {larger} was larger"
            );
            assert!(c.end_myr - c.start_myr >= p.tectonics.weld_after_myr - 1e-9);
        } else {
            assert!(
                c.stalled || c.front_shortening_km >= p.tectonics.collision_threshold_km,
                "a transfer at {} km of shortening",
                c.front_shortening_km
            );
        }
    }
    // Collisions take many steps to get there, and the welds' belts were built over them.
    assert!(d.collisions.iter().any(|c| c.end_myr > c.start_myr));
    assert!(d.collisions.iter().any(|c| c.sutured), "no weld in 300 Myr");
    assert!(
        d.collisions
            .iter()
            .filter(|c| c.sutured)
            .all(|c| c.end_myr - c.start_myr >= 4.0)
    );
}

/// The sweep behind the seeds of the two tests that need a weld (this one and
/// `recording_does_not_change_the_world_and_matches_the_run`): seeds 1 to 24 from scattered
/// continents at 8000 samples over 300 Myr and each one's welds. Run it when a default changes
/// the planet: `cargo test -p deeptime-core --release -- --ignored --nocapture weld_seed_sweep`.
#[test]
#[ignore]
fn weld_seed_sweep() {
    use rayon::prelude::*;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();
    let welds: Vec<(u64, usize)> = pool.install(|| {
        (1..=24u64)
            .into_par_iter()
            .map(|seed| {
                let p = preset(8000, 300.0, "scattered_continents", seed);
                let world = run_stage(&p, |_, _, _| {});
                let d = world.plates.as_ref().unwrap().dynamics.as_ref().unwrap();
                (seed, d.collisions.iter().filter(|c| c.sutured).count())
            })
            .collect()
    });
    let welding: Vec<u64> = welds.iter().filter(|w| w.1 > 0).map(|w| w.0).collect();
    println!("welds per seed {welds:?}");
    println!("seeds that weld {welding:?}");
}

/// Plate task 6: a collision's footprint (`Sample::collision_mm_yr`, `shortening_km` and the
/// FORELAND flag, the L1 fields `plates/collision_mm_yr` and `plates/shortening`) is there while
/// a collision has a front, carries both numbers together, and is gone the step after it ends:
/// the structure stage reads it as the strength and tightness of the folds, so a footprint that
/// outlived its collision would keep folding a belt nothing shortens.
#[test]
fn the_collision_footprint_lives_with_its_collision() {
    let p = preset(8000, 300.0, "scattered_continents", 23);
    let (mut marked_steps, mut foreland_steps, mut prev_alive, mut onsets) = (0, 0, false, 0);
    let world = run_stage(&p, |_, _, st| {
        let active = st
            .contacts
            .iter()
            .any(|c| c.collision.as_ref().is_some_and(|k| k.last_step == st.step));
        let alive = st.contacts.iter().any(|c| c.collision.is_some());
        let marked = st.samples.iter().any(|s| s.collision_mm_yr > 0.0);
        // (A collision that fires or ends in a step is gone from the contacts by its end; it was
        // alive at the end of the step before, with its front or in its grace steps without one,
        // since one in its grace steps can find its front again and fire in the same step.)
        // (A collision that fires in the step it began never shows as active.)
        let onset = st.counters.collision_onsets > onsets;
        onsets = st.counters.collision_onsets;
        assert!(
            !marked || active || prev_alive || onset,
            "step {}: a footprint with no collision behind it",
            st.step
        );
        for s in &st.samples {
            assert!(s.collision_mm_yr >= 0.0 && s.shortening_km >= 0.0);
            // The two numbers come together, and the foreland flag belongs to a footprint.
            if s.collision_mm_yr > 0.0 || s.flags & state::FORELAND != 0 {
                assert!(s.shortening_km > 0.0, "a footprint with no shortening");
            }
        }
        marked_steps += i32::from(marked);
        foreland_steps += i32::from(st.samples.iter().any(|s| s.flags & state::FORELAND != 0));
        prev_alive = alive;
    });
    assert!(
        marked_steps > 10,
        "{marked_steps} steps with a footprint in 300 Myr"
    );
    assert!(
        foreland_steps > 10,
        "{foreland_steps} steps with a foreland"
    );
    for name in ["plates/collision_mm_yr", "plates/shortening"] {
        let f = world
            .fields
            .get(name)
            .unwrap_or_else(|| panic!("no {name}"));
        assert!(f.data.iter().all(|v| v.is_finite() && *v >= 0.0), "{name}");
    }
}

/// The dynamics record (`dynamics.rs`) is measurement only: a run with it and a run without it
/// end in the same world (crust, elevation, plate ids, events, fields), and the record agrees with
/// the run's own counters and events.
#[test]
fn recording_does_not_change_the_world_and_matches_the_run() {
    // A seed with welds, so the collision record is tested (the sweep in
    // `collisions_weld_on_stall_and_transfer_on_shortening` says which seeds weld; seed 2 until
    // V1 gap 2's stages went on by default, 2026-10-23, then seed 4: of the welding seeds 4, 12,
    // 14, 19 and 21 pass, 17 puts the mix's fronts 32 % over the plates' in one step and 18 fails
    // another check; seed 19 since the fronts by their straight share went on by default,
    // PLT-530, 2026-10-24, when seed 4 no longer welds: all 11 welding seeds pass; seed 20 since
    // fission and the split inheritance went on by default, PLT-532 and PLT-534, the same day,
    // when seed 19 no longer welds: of the 9 welding seeds 1, 6, 11, 20 and 21 pass, and on 2, 3,
    // 14 and 15 the speed guard binds for 1 to 3 plate-steps in 300 Myr: build log 2026-10-24,
    // "the front lane"; seed 19 still does not weld with a split's latent slab copies, PLT-534,
    // the same day, so seed 20, the next seed that welds, stays: the seed is chosen by its weld,
    // and the guard asserted below binds on welding seed 16; seed 19 again since the level set
    // went on by default, PLT-536, the same day, when seed 19 welds again and seed 20 no more;
    // seed 20 again since the contact carry went on by default, PLT-550, the same day, when seed
    // 19 no longer welds: all 12 welding seeds pass).
    let p = preset(8000, 300.0, "scattered_continents", 20);
    let total = PlatesStage::new(p.clone()).total_steps();
    let mut checked = 0u64;
    let mut life = CollisionLives::default();
    let (mut arc_cells_seen, mut mix_seen) = (0u64, 0.0f64);
    let (mut mix_fronts, mut recorded_fronts) = (0.0f64, 0.0f64);
    let (mut worst_excess, mut worst_step) = (0.0f64, 0u64);
    let (mut balance_steps, mut longest_slab) = (0u64, 0.0f64);
    let mut deaths_before = 0;
    let mut prev_omega: BTreeMap<u32, glam::DVec3> = BTreeMap::new();
    let (mut pending, mut escaped, mut consumed) = (0i64, 0u64, 0u64);
    let with = run_stage(&p, |stage, _, st| {
        life.step(st);
        let d = st.dynamics.as_ref().expect("recording is on by default");
        let step = d.steps.last().expect("a record per step");
        assert_eq!(d.steps.len() as u64, st.step);
        assert!((step.t - st.t_myr).abs() < 1e-9);
        // The record's plates are the plates alive, and their samples the visible ones.
        assert_eq!(step.plates.len(), st.alive().count());
        assert_eq!(
            step.plates.iter().map(|q| q.visible).sum::<u32>(),
            step.visible
        );
        assert!(step.visible <= st.n && step.land <= step.visible);
        assert!(step.continental <= step.visible);
        for q in &step.plates {
            assert!(q.continental <= q.visible);
            assert!((0.0..=1.0).contains(&q.slab_share()), "{q:?}");
            assert!(q.speed_mm_yr >= 0.0 && q.boundary_km >= 0.0 && q.front_km >= 0.0);
        }
        // Task 1b. Rotation change: the vector change since the previous step, for the plates that
        // were there; the guard: a plate the guard held has its fastest point at the guard; the
        // pole: on some plate.
        let cap = stage
            .norm()
            .omega_of_speed(p.tectonics.max_plate_speed_mm_yr);
        for q in &step.plates {
            let w = glam::DVec3::from_array(st.plates[q.id as usize].omega);
            assert_eq!(q.rot_known, prev_omega.contains_key(&q.id), "{q:?}");
            match prev_omega.get(&q.id) {
                Some(w0) if w0.length() > 0.0 => {
                    assert!(
                        (q.rot_change - (w - *w0).length() / w0.length()).abs() < 1e-4,
                        "{q:?}"
                    );
                }
                _ => assert_eq!(q.rot_change, 0.0),
            }
            if q.guard {
                assert!(w.length() >= cap * (1.0 - 1e-9), "{q:?}");
            }
            let c = glam::DVec3::from_array(st.plates[q.id as usize].centroid);
            assert_eq!(
                q.spinning,
                w.length() > 0.0
                    && w.normalize().dot(c).abs() >= dynamics::SPIN_ANGLE_DEG.to_radians().cos(),
                "{q:?}"
            );
        }
        prev_omega = st
            .alive()
            .map(|q| (q.id, glam::DVec3::from_array(q.omega)))
            .collect();
        // The sea floor: its count is the visible oceanic samples', the percentiles ordered.
        let ocean = st
            .samples
            .iter()
            .filter(|s| !s.is_hidden() && s.kind == state::OCEANIC)
            .count();
        assert_eq!(step.ocean as usize, ocean);
        assert!(step.age_p50 <= step.age_p90 && step.old250 <= step.old180);
        assert!(step.old180 <= step.ocean);
        // The pieces of continental crust: largest first, within the crust, and no larger than
        // the pieces joined along every lattice edge (the record also needs the ends to be close).
        assert!(step.pieces.windows(2).all(|w| w[0] >= w[1]));
        assert!(step.pieces.len() <= dynamics::PIECES_KEPT);
        assert!(step.pieces.iter().sum::<u32>() <= step.continental);
        assert_eq!(step.continental > 0, !step.pieces.is_empty());
        let geo = &stage.cache.as_ref().expect("built").geo;
        let loose = dynamics::piece_sizes(
            st.samples.len(),
            |i| !st.samples[i].is_hidden() && st.samples[i].kind == state::CONTINENTAL,
            |i, out| out.extend(geo.edges(i as u32).map(|(u, _)| u)),
        );
        assert!(step.pieces.first().copied().unwrap_or(0) <= loose.first().copied().unwrap_or(0));
        assert!(step.pieces.len() >= loose.len().min(dynamics::PIECES_KEPT));
        // The boundary mix: every kind is a length; the subducting fronts (the first two kinds,
        // oceanic slabs) are nearly the fronts whose lengths the plates record (a collision front
        // pulls nothing since plate tasks 3 and 4). Both judge the motion across the boundary's
        // normal and call it a transform within Bird's 20 degrees (PLT-310, PLT-311; the plates'
        // fronts since PLT-330, 2026-10-20), and the plates (`subduction::fronts_begin`) also
        // need the front's lattice edge to converge. So the plates' fronts lie within the mix's,
        // and the mix holds a little more: the samples whose boundary converges while each of
        // their lattice edges parts. Measured (2026-10-20, PLT-330 and PLT-331, 48 runs of seeds
        // 1 to 24 from both starts at this size): no step in any run where the plates' fronts
        // exceed the mix's beyond the tolerance; the mix above the plates' by 3.8 to 6.6 % over a
        // run (this seed 5.4 %) and by at most 8 to 32 % in a step (this seed 10.3 %). Until
        // PLT-330 the plates also counted their contacts' transform stretches and held more.
        assert!(step.mix_km.iter().all(|&x| x.is_finite() && x >= 0.0));
        let fronts: f64 = step.mix_km[..2].iter().sum();
        let recorded: f64 = step.plates.iter().map(|q| q.front_km).sum();
        // (A plate that died in the step, welded or absorbed, has no plate record: its fronts are
        // in the mix only.)
        let tol = 0.5 * step.plates.len() as f64 + 1.0;
        if st.counters.plate_deaths == deaths_before {
            // (0.0000 of the mix to four places in every run measured; a test bound of 0.001.)
            assert!(
                recorded <= fronts * 1.001 + tol,
                "step {}: the plates' fronts {recorded} km exceed the mix's {fronts} km",
                st.step
            );
            let excess = (fronts - recorded - tol).max(0.0) / recorded.max(1.0);
            if excess > worst_excess {
                worst_excess = excess;
                worst_step = st.step;
            }
            mix_fronts += fronts;
            recorded_fronts += recorded;
        }
        deaths_before = st.counters.plate_deaths;
        // The arcs: emergent cells are cells, the percentiles ordered, nothing above the cap.
        for arc in [&step.arc_oceanic, &step.arc_continental] {
            assert!(arc.emergent <= arc.cells && arc.cells <= step.visible);
            if arc.cells > 0 {
                assert!(arc.p50_m <= arc.p90_m && arc.p90_m <= arc.max_m, "{arc:?}");
            } else {
                assert_eq!(*arc, dynamics::ArcStat::default());
            }
        }
        arc_cells_seen += u64::from(step.arc_oceanic.cells + step.arc_continental.cells);
        mix_seen += step.mix_km.iter().sum::<f64>();
        // The force balance (task 5): one answer per plate in it, finite; slabs within their
        // depth. (A plate that died later in the step may have an answer and no plate record.)
        let mut ids: Vec<u32> = step.forces.iter().map(|f| f.id).collect();
        ids.dedup();
        assert_eq!(ids.len(), step.forces.len());
        for f in &step.forces {
            let torques = [
                f.slab_pull,
                f.slab_suction,
                f.edge_energy,
                f.mantle_flow,
                f.basal_drag,
                f.slab_resistance,
                f.boundary_resistance,
                f.collision_resistance,
            ];
            assert!(torques.iter().all(|t| t.is_finite() && *t >= 0.0), "{f:?}");
            assert!(f.stress_mpa.is_finite() && f.speed_mm_yr >= 0.0 && f.rms_mm_yr >= 0.0);
        }
        balance_steps += u64::from(!step.forces.is_empty());
        for &km in &step.slabs_km {
            assert!((0.0..=p.tectonics.slab_depth_km).contains(&km), "{km}");
            longest_slab = longest_slab.max(km);
        }
        // The crust shown again (`shown.rs`, measurement only): what went under less what showed
        // again since the last resampling is what lies hidden, and what the resampling consumes.
        let sh = &step.shown;
        pending += i64::from(sh.under) - sh.shown.iter().map(|&x| i64::from(x)).sum::<i64>();
        assert!(sh.shown_cont.iter().zip(&sh.shown).all(|(c, n)| c <= n));
        assert!(sh.escaped_cont.iter().zip(&sh.escaped).all(|(c, n)| c <= n));
        if st.resample_step == st.step {
            assert_eq!(pending, i64::from(sh.consumed), "step {}", st.step);
            assert!(sh.escaped_ocean <= sh.ocean && sh.consumed_cont <= sh.consumed);
            pending = 0;
            escaped += u64::from(sh.escaped.iter().sum::<u32>());
            consumed += u64::from(sh.consumed);
        } else {
            let hidden = st.samples.iter().filter(|s| s.is_hidden()).count() as i64;
            assert_eq!(pending, hidden, "step {}", st.step);
            assert_eq!(sh.consumed + sh.escaped.iter().sum::<u32>(), 0);
        }
        checked += 1;
    });
    eprintln!("shown again: {escaped} samples escaped the resamplings, {consumed} consumed");
    assert!(consumed > 0 && escaped > 0);
    assert_eq!(checked, total);
    assert_eq!(balance_steps, total, "a balance every step");
    // The switch (force-balance design, task 7): the balance was solved every step and the speed
    // guard never bound in 300 Myr; plates started at rest and the balance set them moving.
    assert_eq!(
        with.globals.get(UNSOLVED_GLOBAL),
        None,
        "an unsolved balance"
    );
    assert_eq!(
        with.globals.get(GUARD_BOUND_GLOBAL),
        None,
        "the speed guard bound"
    );
    {
        let d = with.plates.as_ref().unwrap().dynamics.as_ref().unwrap();
        let median = |mut v: Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        // After one step from rest a plate has made 1 − e^(−δt/τ) of the way to its answer.
        let first = median(d.steps[0].plates.iter().map(|q| q.speed_mm_yr).collect());
        let late = median(
            d.steps[d.steps.len() / 2..]
                .iter()
                .flat_map(|s| s.plates.iter().map(|q| q.speed_mm_yr))
                .collect(),
        );
        assert!(late > 10.0, "the plates hardly move: median {late} mm/yr");
        assert!(
            first < late,
            "plates did not start at rest: {first} against {late}"
        );
    }
    assert!(longest_slab > 0.0, "no slab grew in 300 Myr");
    assert!(mix_seen > 0.0, "no plate boundary recorded in 300 Myr");
    // Test bounds, three times the measured excess over the run and in the worst step (above), on
    // seed 2; seed 4 has 4.6 % over the run and 11.2 % in its worst step (2026-10-23).
    assert!(
        mix_fronts >= recorded_fronts && mix_fronts <= 1.165 * recorded_fronts,
        "the mix's fronts {mix_fronts} km against the plates' {recorded_fronts} km"
    );
    assert!(
        worst_excess <= 0.31,
        "step {worst_step}: the mix's fronts exceed the plates' by {worst_excess} of them"
    );
    assert!(arc_cells_seen > 0, "no arc cell recorded in 300 Myr");
    let st = with.plates.as_ref().unwrap();
    let d = st.dynamics.as_ref().unwrap();
    assert_eq!(d.steps.len() as u64, total);
    assert!(d.steps.windows(2).all(|w| w[1].t > w[0].t));

    // Births and deaths are the events', collisions the counters'.
    let births = with
        .events
        .iter()
        .filter(|e| e.kind == events::PLATE_BIRTH)
        .count();
    let deaths = with
        .events
        .iter()
        .filter(|e| e.kind == events::PLATE_DEATH)
        .count();
    let initial = d.plates.iter().filter(|l| l.origin == "initial").count();
    assert_eq!(d.plates.len(), initial + births);
    assert_eq!(
        d.plates.iter().filter(|l| l.died_myr.is_some()).count(),
        deaths
    );
    assert_eq!(deaths as u32, st.counters.plate_deaths);
    for l in &d.plates {
        assert_eq!(l.died_myr.is_some(), l.fate.is_some());
        assert_eq!(l.died_myr.is_some(), !st.plates[l.id as usize].alive);
        if let Some(died) = l.died_myr {
            assert!(died >= l.born_myr);
            assert!(matches!(
                l.fate.as_deref(),
                Some("sutured" | "absorbed" | "consumed" | "healed")
            ));
        }
        if l.origin != "initial" {
            assert!(l.parent.is_some());
        }
    }
    let welds = d
        .plates
        .iter()
        .filter(|l| l.fate.as_deref() == Some("sutured"))
        .count() as u32;
    assert_eq!(welds, st.counters.sutures);
    assert_eq!(d.collisions.len() as u32, st.counters.accretions);
    assert_eq!(
        d.collisions.iter().filter(|c| c.collision).count() as u32,
        st.counters.collisions
    );
    assert!(
        d.collisions.iter().any(|c| c.sutured),
        "no suture in 300 Myr: the collision record is untested"
    );
    life.finish(st, d);
    for c in &d.collisions {
        assert!(c.front_shortening_km >= 0.0 && c.area_km2 >= 0.0);
        assert!(c.end_myr >= c.start_myr && c.start_myr >= c.contact_onset_myr - 1e-9);
        assert!(c.lifted100.samples <= c.lifted10.samples);
        assert!(c.peak_uplift_m >= 0.0 && c.shortening_km >= 0.0);
        assert!(c.frac > 0.0 && c.terrane_samples > 0);
        assert_eq!(c.collision, c.frac >= p.tectonics.collision_min_frac);
        for f in [c.lifted10, c.lifted100, c.emerged] {
            assert!(f.width_km <= f.length_km + 1e-6);
        }
    }
    // The sea floor: histograms add up to the samples, the last record is the last step's.
    assert!(!d.ages.is_empty());
    for a in &d.ages {
        assert_eq!(a.hist.len(), dynamics::AGE_BINS);
        assert_eq!(a.hist.iter().sum::<u32>(), a.samples);
        assert!(a.p50_myr <= a.p90_myr && a.p90_myr <= a.max_myr);
    }
    assert!((d.ages.last().unwrap().t - st.t_myr).abs() < 1e-9);
    // The note carries the whole record, beside the margins.
    let note: serde_json::Value = serde_json::from_str(&with.notes["plates"]).unwrap();
    let back: dynamics::Dynamics = serde_json::from_value(note["dynamics"].clone()).unwrap();
    assert_eq!(back.steps.len(), d.steps.len());
    assert_eq!(back.collisions.len(), d.collisions.len());
    assert_eq!(back.partings.len(), d.partings.len());
    assert!(note["margins"].is_array());

    // The same run without the record ends in the same world.
    let without = run_stage_with(
        PlatesStage::new(p.clone()).without_dynamics(),
        &p,
        |_, _, st| assert!(st.dynamics.is_none()),
    );
    let strip = |mut w: WorldState| {
        let st = w.plates.as_mut().unwrap();
        st.dynamics = None;
        st.shown = None;
        for c in &mut st.contacts {
            c.cont_in = 0;
        }
        let mut note: serde_json::Value = serde_json::from_str(&w.notes["plates"]).unwrap();
        note.as_object_mut().unwrap().remove("dynamics");
        w.notes.insert("plates".into(), note.to_string());
        w
    };
    same_world(&strip(with), &strip(without));
}

/// The land's gap budget (`gap_budget.rs`) changes nothing; its parts add up to every sample's
/// gap through the run, resamplings included; it records every 50 Myr.
#[test]
fn the_gap_budget_changes_nothing_and_adds_up() {
    let p = preset(8000, 200.0, "scattered_continents", 6);
    let with = run_stage_with(
        PlatesStage::new(p.clone()).with_gap_budget(),
        &p,
        |_, _, st| {
            let gb = st.gap_budget.as_ref().expect("on");
            let worst = gb.worst_residual_m();
            assert!(
                worst < 5.0,
                "step {}: parts off the gap by {worst} m",
                st.step
            );
        },
    );
    let st = with.plates.as_ref().unwrap();
    let gb = st.gap_budget.as_ref().unwrap();
    assert_eq!(gb.restarts, 0);
    assert!(st.counters.resamplings > 0);
    assert_eq!(gb.snapshots.len(), 4);
    let last = gb.snapshots.last().unwrap();
    assert!(last.bands.iter().map(|b| b.n).sum::<u64>() > 0);
    for b in &last.bands {
        let sum: f64 = b.parts.iter().sum();
        assert!(b.n == 0 || (sum - b.gap).abs() < 5.0, "{sum} vs {}", b.gap);
    }
    let without = run_stage_with(PlatesStage::new(p.clone()), &p, |_, _, st| {
        assert!(st.gap_budget.is_none())
    });
    let strip = |mut w: WorldState| {
        w.plates.as_mut().unwrap().gap_budget = None;
        w
    };
    same_world(&strip(with), &strip(without));
}

/// The continental crust budget (`crust_budget.rs`) changes nothing but its note, and snapshots
/// the start and every 50 Myr.
#[test]
fn the_crust_budget_changes_nothing() {
    let p = preset(8000, 200.0, "scattered_continents", 6);
    let mut with = run_stage_with(
        PlatesStage::new(p.clone()).with_crust_budget(),
        &p,
        |_, _, _| {},
    );
    let note = with
        .notes
        .remove(super::CRUST_BUDGET_NOTE)
        .expect("the budget's note");
    assert_eq!(note.matches("\ncb_t ").count(), 5, "{note}");
    assert!(note.contains("resample_blend:"), "{note}");
    let without = run_stage_with(PlatesStage::new(p.clone()), &p, |_, _, _| {});
    assert!(!without.notes.contains_key(super::CRUST_BUDGET_NOTE));
    same_world(&with, &without);
}

/// The fronts' lengths as the slab pull and the arcs count them (measurement only, PLT-554; the
/// front lane, step 8, 2026-10-24), over a 100-Myr run: the slab pull's one spacing per
/// overriding sample that pulls, the part of each along the boundary's normal, and the same
/// samples' shares of the straight front (PLT-530); and the arcs' bands by their length against
/// the arcs' straight front. Measured (2026-10-24, 8,000 samples): the pull's spacings are 1.106
/// times the same samples' straight front, 0.820 of them along the normal; the arcs' band 1.119
/// times the arcs' straight front, which is 0.868 of the pull's.
#[test]
fn the_fronts_by_the_slab_pull_and_the_arcs() {
    let p = preset(8000, 100.0, "scattered_continents", 6);
    let w = run_stage_with(PlatesStage::new(p.clone()), &p, |_, _, _| {});
    let g = |key: &str| w.globals.get(key).copied().unwrap_or(0.0);
    let lattice = g(forces::SLAB_FRONT_LATTICE_GLOBAL);
    let normal = g(forces::SLAB_FRONT_NORMAL_GLOBAL);
    let straight = g(forces::SLAB_FRONT_STRAIGHT_GLOBAL);
    let band = g(subduction::ARC_BAND_GLOBAL);
    let arcs = g(subduction::ARC_FRONT_STRAIGHT_GLOBAL);
    eprintln!(
        "slab pull: {lattice:.0} km Myr by the lattice, {:.3} of it along the normal, {:.3} by \
         the straight front; arcs: band {band:.0} km Myr, {:.3} of their straight front \
         {arcs:.0}, which is {:.3} of the slab pull's straight front",
        normal / lattice,
        straight / lattice,
        band / arcs,
        arcs / straight
    );
    assert!(lattice > 0.0 && arcs > 0.0 && band > 0.0);
    assert!(normal > 0.3 * lattice && normal <= lattice);
    assert!(straight > 0.3 * lattice && straight < 1.1 * lattice);
}

/// Two plates on the preset's planet: the northern hemisphere continental (plate 0), the southern
/// oceanic (plate 1), the southern turned by `turn` radians about the x axis since the last
/// resampling, so it runs under the continent where y > 0 and opens a gap where y < 0. Overlaps
/// are resolved (the ocean subducts, its crust hidden) as a step would.
fn two_plates(p: &PlanetPreset, env: &Env<'_>, turn: f64) -> (PlateState, Geo, Vec<DVec3>) {
    plates_split(p, env, turn, |q| q.z > 0.0)
}

/// The planet of `two_plates` with plate 0, the continent, where `north` holds.
fn plates_split(
    p: &PlanetPreset,
    env: &Env<'_>,
    turn: f64,
    north: impl Fn(DVec3) -> bool,
) -> (PlateState, Geo, Vec<DVec3>) {
    let (mut st, geo) = plates_unresolved(p, env, glam::DQuat::from_rotation_x(turn), north);
    let pos = geometry::positions(&st, &geo);
    st.recount(&pos);
    let tree = Tree::over(&pos, |_| true);
    let overlaps = subduction::overlaps(&st, &geo, &pos, &tree, env);
    let mut world = WorldState::new(1, 16_384, p.t_start_years());
    subduction::resolve(&mut st, &mut world, &geo, &overlaps, &pos, env).unwrap();
    (st, geo, pos)
}

/// The planet of `plates_split` before its overlaps are found: plate 1's group turned by `q`.
fn plates_unresolved(
    p: &PlanetPreset,
    env: &Env<'_>,
    q: glam::DQuat,
    north: impl Fn(DVec3) -> bool,
) -> (PlateState, Geo) {
    let (mut st, geo) = init::initial_state(p, env).unwrap();
    let pts = geo.points();
    for (i, s) in st.samples.iter_mut().enumerate() {
        let north = north(pts[i]);
        *s = state::Sample::blank(u32::from(!north));
        if north {
            s.kind = CONTINENTAL;
            s.pools = pools::continental_column();
            s.elevation_m = 400.0;
        } else {
            s.pools = pools::oceanic_column();
            s.elevation_m = -4000.0;
            s.formed_myr = (env.t_myr - 60.0) as f32;
        }
        s.thickness_m = pools::thickness(&s.pools);
    }
    st.groups = vec![
        state::Group {
            plate: 0,
            rot: [0.0, 0.0, 0.0, 1.0],
        },
        state::Group {
            plate: 1,
            rot: [q.x, q.y, q.z, q.w],
        },
    ];
    for (k, pl) in st.plates.iter_mut().enumerate() {
        pl.alive = k < 2;
        pl.omega = [0.0; 3];
    }
    st.contacts.clear();
    st.ridges.clear();
    st.rift_links.clear();
    (st, geo)
}

/// The outline of `plates_unresolved`'s planet at its last resampling, the equator, held exactly
/// by the level set: every sample's sine distance from it (a great circle, PLT-536).
fn equator_level_set(st: &mut PlateState, geo: &Geo) {
    st.level_set = Some(level_set::LevelSet {
        phi: geo.points().iter().map(|p| p.z.abs() as f32).collect(),
        member: st.samples.iter().map(|s| s.group).collect(),
    });
}

/// The planet's overlaps with its groups turned as they stand, by the rule `env` sets.
fn overlaps_now(
    st: &mut PlateState,
    geo: &Geo,
    env: &Env<'_>,
) -> (Vec<DVec3>, Vec<subduction::Overlap>) {
    let pos = geometry::positions(st, geo);
    st.recount(&pos);
    let tree = Tree::over(&pos, |_| true);
    let overlaps = subduction::overlaps(st, geo, &pos, &tree, env);
    (pos, overlaps)
}

/// A sample goes under another plate where it crosses that plate's outline
/// (`tectonics.outline_cover`, PLT-553; the front lane, step 8, 2026-10-24). The two-plate planet
/// with its outline, the equator, held exactly by the level set, and the ocean turned under the
/// continent by a third of a spacing where y > 0 (away from it where y < 0). Each sample's depth
/// past the other plate's outline is its signed distance from the equator in that plate's frame.
/// The outline rule finds only samples past the outline and every one past it by more than a
/// fiftieth of a spacing; the hull rule (off: a whole lattice triangle of the other plate must
/// hold the sample, `geometry::cover`) finds a subset of them, few of the shallow ones. Turned by
/// nothing, neither finds any. Measured (2026-10-24): 63 samples past the outline, the outline
/// rule finds all 63, the hull rule none (19 of them lie within 0.1 spacings, 30 within 0.1 to
/// 0.2, 14 within 0.2 to 0.35): a third of a spacing of convergence is not yet a subduction
/// zone by the hull rule.
#[test]
fn a_sample_goes_under_where_it_crosses_the_outline() {
    let p = preset(20_000, 20.0, "supercontinent", 5);
    let stage = PlatesStage::new(p.clone());
    let mut hull = p.tectonics.clone();
    hull.outline_cover = false;
    let spacing = stage.norm().spacing_rad();
    for turn in [0.0, 0.35] {
        let q = glam::DQuat::from_rotation_x(turn * spacing);
        // (rule, overlapping samples, every sample's depth past the other plate's outline).
        let mut seen: Vec<(bool, std::collections::BTreeSet<u32>, Vec<f64>)> = Vec::new();
        for (outline, k) in [(true, &p.tectonics), (false, &hull)] {
            let env = stage.env(k, 1);
            let (mut st, geo) = plates_unresolved(&p, &env, q, |x| x.z > 0.0);
            equator_level_set(&mut st, &geo);
            let (pos, overlaps) = overlaps_now(&mut st, &geo, &env);
            let depth: Vec<f64> = (0..pos.len())
                .map(|i| {
                    let z = if st.samples[i].group == 1 {
                        pos[i].z
                    } else {
                        -(q.inverse() * pos[i]).z
                    };
                    z.clamp(-1.0, 1.0).asin() / spacing
                })
                .collect();
            for o in &overlaps {
                let i = o.i as usize;
                assert_ne!(st.samples[i].group, o.group);
                assert!(
                    depth[i] > -1e-4,
                    "outline {outline}, turn {turn}: sample {i} overlaps outside the outline \
                     ({:.4} spacings)",
                    depth[i]
                );
            }
            let found = overlaps.iter().map(|o| o.i).collect();
            seen.push((outline, found, depth));
        }
        let (_, by_outline, depth) = &seen[0];
        let (_, by_hull, _) = &seen[1];
        let past: Vec<usize> = (0..depth.len()).filter(|&i| depth[i] > 0.0).collect();
        let missed: Vec<usize> = past
            .iter()
            .copied()
            .filter(|&i| depth[i] > 0.02 && !by_outline.contains(&(i as u32)))
            .collect();
        let bins = [0.0, 0.1, 0.2, 0.35];
        let share: Vec<String> = bins
            .windows(2)
            .map(|b| {
                let inb: Vec<usize> = past
                    .iter()
                    .copied()
                    .filter(|&i| depth[i] > b[0] && depth[i] <= b[1])
                    .collect();
                let hull = inb
                    .iter()
                    .filter(|&&i| by_hull.contains(&(i as u32)))
                    .count();
                format!("{}-{}: {hull} of {}", b[0], b[1], inb.len())
            })
            .collect();
        eprintln!(
            "turn {turn} spacings: {} samples past the other plate's outline; the outline rule \
             finds {} (missed {} deeper than 0.02), the hull rule {} (by depth, spacings: {})",
            past.len(),
            by_outline.len(),
            missed.len(),
            by_hull.len(),
            share.join(", ")
        );
        assert!(missed.is_empty(), "missed {missed:?}");
        assert!(by_hull.is_subset(by_outline));
        if turn == 0.0 {
            assert!(by_outline.is_empty() && by_hull.is_empty());
        } else {
            assert!(by_outline.len() > 2 * by_hull.len().max(1));
        }
    }
}

/// Bounds of `what_the_outline_cover_finds_at_a_transform` (numerical; the front lane, step 8,
/// 2026-10-24): overlaps per spacing of a transform, and how far from the boundary, spacings
/// (measured at most 0.27 and 0.48).
const TRANSFORM_OVERLAPS: [f64; 2] = [0.35, 0.6];

/// What the outline rule finds at a transform (PLT-553): the two plates of
/// `a_sample_goes_under_where_it_crosses_the_outline` sliding past each other along the equator
/// (plate 1 turned about the z axis), which no sample crosses. Three outlines: the equator held
/// exactly; the one a lattice draws with no level set (the midpoints of the edges between the
/// plates: a cut's outline until a resampling, `level_set::STALE_SPACINGS`); and that outline
/// carried by the level set through one and five resamplings. Per outline and slide, the
/// overlaps per spacing of boundary length by each rule and the farthest from the equator.
/// Measured (2026-10-24): none on the exact outline; on the lattice's, the outline rule finds up
/// to 0.22 per spacing (0.27 after one resampling, 0.16 after five) once the plates have slid a
/// spacing, up to 0.48 spacings from the equator, where the hull rule finds up to 0.04: one
/// plate's zigzag outline no longer nests in the other's. Bounded by [`TRANSFORM_OVERLAPS`].
#[test]
fn what_the_outline_cover_finds_at_a_transform() {
    let p = preset(20_000, 20.0, "supercontinent", 5);
    let stage = PlatesStage::new(p.clone());
    let mut hull = p.tectonics.clone();
    hull.outline_cover = false;
    let spacing = stage.norm().spacing_rad();
    let length = std::f64::consts::TAU / spacing;
    for (label, resamplings) in [
        ("exact", None),
        ("stale", Some(0)),
        ("resampled 1", Some(1)),
        ("resampled 5", Some(5)),
    ] {
        let env = stage.env(&p.tectonics, 1);
        let (mut st, mut geo) = plates_unresolved(&p, &env, glam::DQuat::IDENTITY, |x| x.z > 0.0);
        match resamplings {
            None => equator_level_set(&mut st, &geo),
            Some(n) => {
                st.level_set = None;
                for _ in 0..n {
                    let pos = geo.points().to_vec();
                    let mut world = WorldState::new(1, 16_384, p.t_start_years());
                    let tree = Tree::over(&pos, |_| true);
                    geo = ridges::resample(&mut st, &mut world, &pos, &tree, &geo, &env).unwrap();
                    level_set::reinit(&mut st, &geo, spacing);
                }
            }
        }
        for slide in [0.25, 0.5, 1.0, 2.37] {
            let q = glam::DQuat::from_rotation_z(slide * spacing);
            let mut line = Vec::new();
            for (outline, k) in [(true, &p.tectonics), (false, &hull)] {
                let env = stage.env(k, 1);
                let mut s2 = st.clone();
                for g in &mut s2.groups {
                    g.rot = if g.plate == 1 {
                        [q.x, q.y, q.z, q.w]
                    } else {
                        [0.0, 0.0, 0.0, 1.0]
                    };
                }
                let (pos, overlaps) = overlaps_now(&mut s2, &geo, &env);
                let far = overlaps
                    .iter()
                    .map(|o| pos[o.i as usize].z.abs().asin() / spacing)
                    .fold(0.0, f64::max);
                line.push(format!(
                    "{} {:.3} per spacing (farthest {far:.2} spacings)",
                    if outline { "outline" } else { "hull" },
                    overlaps.len() as f64 / length
                ));
                if resamplings.is_none() {
                    assert!(
                        overlaps.is_empty(),
                        "{label}, slide {slide}: {}",
                        overlaps.len()
                    );
                }
                assert!(
                    overlaps.len() as f64 / length < TRANSFORM_OVERLAPS[0]
                        && far < TRANSFORM_OVERLAPS[1],
                    "{label}, slide {slide}, outline {outline}: {} overlaps, farthest {far}",
                    overlaps.len()
                );
            }
            eprintln!(
                "{label} outline, slide {slide} spacings: {}",
                line.join(", ")
            );
        }
    }
}

/// Continental area (visible continental samples × sample area) of the two-plate planet before a
/// resampling, and after it on `tries` lattices turned at random.
fn continental_area_through_resampling(turn: f64, tries: u64) -> (PlateState, f64, Vec<f64>) {
    let p = preset(20_000, 20.0, "supercontinent", 5);
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let (st, geo, pos) = two_plates(&p, &env, turn);
    let area = env.norm.sample_area_km2();
    let continental = |st: &PlateState| {
        st.samples
            .iter()
            .filter(|s| !s.is_hidden() && s.kind == CONTINENTAL)
            .count() as f64
            * area
    };
    let before = continental(&st);
    let after = (0..tries)
        .map(|k| {
            let mut s2 = st.clone();
            s2.rng = crate::sphere::SplitMix64::new(1000 + k);
            let mut world = WorldState::new(1, 16_384, p.t_start_years());
            ridges::resample(
                &mut s2,
                &mut world,
                &pos,
                &Tree::over(&pos, |_| true),
                &geo,
                &env,
            )
            .unwrap();
            continental(&s2)
        })
        .collect();
    (st, before, after)
}

/// Bounds of the level-set tests below (numerical; the front lane, 2026-10-24). With the level set
/// the boundary's spread after 10 to 40 resamplings stayed at or below its first and its edges
/// per spacing within 4 % of their first (without it, by the 40th resampling, they rose by 0.92
/// to 1.29 spacings and by 58 to 107 %). The converging trench's offset moved 0.2 spacings
/// landward in 40 resamplings (0.9 seaward without the level set; 2.1 landward before a group
/// with no corner in the triangle took its distance outside the others' outline,
/// `level_set::margin`).
const SPREAD_SLACK: f64 = 0.15;
const EDGE_SLACK: f64 = 1.15;
const OFFSET_SLACK: f64 = 0.5;
/// The round plate's loss with the level set, samples per resampling: a closed convex outline
/// loses the same area at every radius (the linear interpolation's error is a drift of the
/// outline towards its centre of curvature in proportion to its curvature, and the curvature of a
/// closed outline integrates to 2π), measured at 0.75 to 0.85.
const ROUND_LOSS: [f64; 2] = [0.4, 1.4];

/// Plate outlines below the lattice spacing (`tectonics.level_set_membership`, PLT-536; the front
/// lane, 2026-10-24): resamplings alone must not roughen a plate boundary. The two-plate planet
/// resampled 40 times with nothing else in the step: static, and with the ocean turned under the
/// continent by `turn` between resamplings (a trench where y > 0, a ridge where y < 0). After
/// each resampling, per stretch of boundary: its samples' offset from the equator and their
/// spread about it, in lattice spacings, and the boundary's lattice edges per spacing of the
/// stretch's length (a straight boundary's count). Without the level set the static boundary's
/// samples spread from 0.44 to 1.73 spacings and its edges per spacing rise from 2.25 to 4.65 in
/// 40 resamplings (a random walk: every turned lattice moves the outline up to half a spacing);
/// with it, a great circle's outline lands where it was on every lattice (Osher & Sethian 1988;
/// Sussman, Smereka & Osher 1994).
#[test]
fn how_resamplings_roughen_a_boundary() {
    let p0 = preset(40_000, 20.0, "supercontinent", 5);
    // (level set, planet, stretch, resamplings, spread, edges per spacing, offset).
    let mut seen: Vec<(bool, &str, &str, usize, f64, f64, f64)> = Vec::new();
    for on in [false, true] {
        let mut p = p0.clone();
        p.tectonics.level_set_membership = on;
        let stage = PlatesStage::new(p.clone());
        let env = stage.env(&p.tectonics, 1);
        let spacing = env.norm.spacing_rad();
        for (turn, label) in [(0.0, "static"), (0.02, "converging")] {
            let (mut st, mut geo, mut pos) = two_plates(&p, &env, turn);
            for k in 1..=40 {
                let mut world = WorldState::new(1, 16_384, p.t_start_years());
                let tree = Tree::over(&pos, |_| true);
                geo = ridges::resample(&mut st, &mut world, &pos, &tree, &geo, &env).unwrap();
                if on {
                    level_set::reinit(&mut st, &geo, spacing);
                }
                let pts = geo.points();
                // Plate 0's visible samples with a visible plate-1 neighbour, by stretch.
                let mut stretch: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
                let mut edges = [0usize; 2];
                for (i, q) in pts.iter().enumerate() {
                    if st.samples[i].is_hidden() || st.plate_of(i) != 0 {
                        continue;
                    }
                    let n_other = geo
                        .edges(i as u32)
                        .filter(|&(j, _)| {
                            !st.samples[j as usize].is_hidden() && st.plate_of(j as usize) == 1
                        })
                        .count();
                    if n_other == 0 {
                        continue;
                    }
                    let w = if turn == 0.0 || q.y > 0.3 {
                        0
                    } else if q.y < -0.3 {
                        1
                    } else {
                        continue;
                    };
                    stretch[w].push(q.z.asin() / spacing);
                    edges[w] += n_other;
                }
                if [1, 2, 5, 10, 20, 30, 40].contains(&k) {
                    for (w, name) in [(0, "trench"), (1, "ridge")] {
                        if turn == 0.0 && w == 1 {
                            continue;
                        }
                        let v = &stretch[w];
                        let len = v.len().max(1) as f64;
                        let mean = v.iter().sum::<f64>() / len;
                        let rms = (v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / len).sqrt();
                        // The stretch's length: the whole equator, or each arc with |y| > 0.3.
                        let arc = if turn == 0.0 {
                            std::f64::consts::TAU
                        } else {
                            std::f64::consts::PI - 2.0 * 0.3f64.asin()
                        };
                        let per = edges[w] as f64 / (arc / spacing);
                        eprintln!(
                            "level set {on}: {label} {name} after {k} resamplings: {} samples, \
                             offset {mean:+.2} spacings, spread {rms:.2}, edges per spacing \
                             {per:.2}",
                            v.len(),
                        );
                        seen.push((on, label, name, k, rms, per, mean));
                    }
                }
                // The step's motion: the ocean turned under the continent, overlaps resolved.
                let q = glam::DQuat::from_rotation_x(turn);
                for g in &mut st.groups {
                    if g.plate == 1 {
                        g.rot = [q.x, q.y, q.z, q.w];
                    }
                }
                pos = geometry::positions(&st, &geo);
                st.recount(&pos);
                let tree = Tree::over(&pos, |_| true);
                let overlaps = subduction::overlaps(&st, &geo, &pos, &tree, &env);
                subduction::resolve(&mut st, &mut world, &geo, &overlaps, &pos, &env).unwrap();
            }
        }
    }
    let at = |on: bool, label: &str, name: &str, k: usize| {
        seen.iter()
            .find(|s| s.0 == on && s.1 == label && s.2 == name && s.3 == k)
            .map(|s| (s.4, s.5, s.6))
            .unwrap()
    };
    // Without the level set the static boundary walks: its spread grows and so does its length.
    let (s1, e1, _) = at(false, "static", "trench", 1);
    let (s40, e40, _) = at(false, "static", "trench", 40);
    assert!(
        s40 > 2.0 * s1 && e40 > 1.5 * e1,
        "{s1} → {s40}, {e1} → {e40}"
    );
    // With it, the static boundary and the trench (the overriding plate's outline) stay as
    // straight as the lattice draws them after one resampling, and where they were.
    for (label, name) in [("static", "trench"), ("converging", "trench")] {
        let (s1, e1, o1) = at(true, label, name, 1);
        for k in [10, 20, 30, 40] {
            let (s, e, o) = at(true, label, name, k);
            assert!(
                s < s1 + SPREAD_SLACK && e < e1 * EDGE_SLACK && (o - o1).abs() < OFFSET_SLACK,
                "{label} {name} after {k}: spread {s1} → {s}, edges per spacing {e1} → {e}, \
                 offset {o1} → {o}"
            );
        }
    }
}

/// The level set's cost (PLT-536; the front lane, 2026-10-24): the interpolated distance is exact
/// for a great circle only. A curved outline's distance is not linear in the position, and its
/// linear interpolation (and the reinitialisation's chords) moves the outline towards its centre
/// of curvature by about the spacing squared times the curvature at each resampling: a numerical
/// curvature flow. Its area loss integrates the curvature along the outline, 2π for a closed
/// convex one, so a round plate loses the same number of samples per resampling whatever its
/// radius. A round plate of 3, 6 and 12 spacings' radius on a static planet, resampled 40 times
/// with and without the level set: its samples after each, against its samples at the start.
/// Without the level set the count wanders (by +5, -7 and -12 samples after 20 resamplings);
/// with it, it falls by 0.75 to 0.85 samples per resampling at each radius.
#[test]
fn a_small_round_plate_through_resamplings() {
    let p0 = preset(40_000, 20.0, "supercontinent", 5);
    let mut seen: Vec<(bool, f64, f64)> = Vec::new();
    for on in [false, true] {
        let mut p = p0.clone();
        p.tectonics.level_set_membership = on;
        let stage = PlatesStage::new(p.clone());
        let env = stage.env(&p.tectonics, 1);
        let spacing = env.norm.spacing_rad();
        for radius in [3.0, 6.0, 12.0] {
            let centre = DVec3::new(0.2, -0.4, 0.7).normalize();
            let cos_r = (radius * spacing).cos();
            let (mut st, mut geo, pos) = plates_split(&p, &env, 0.0, |q| q.dot(centre) > cos_r);
            let count = |st: &PlateState| {
                (0..st.samples.len())
                    .filter(|&i| st.plate_of(i) == 0)
                    .count()
            };
            let start = count(&st);
            let mut pos = pos;
            let mut counts = Vec::new();
            for _ in 1..=40 {
                let mut world = WorldState::new(1, 16_384, p.t_start_years());
                let tree = Tree::over(&pos, |_| true);
                geo = ridges::resample(&mut st, &mut world, &pos, &tree, &geo, &env).unwrap();
                if on {
                    level_set::reinit(&mut st, &geo, spacing);
                }
                pos = geo.points().to_vec();
                counts.push(count(&st));
            }
            let mean_last: f64 = counts[30..].iter().map(|&c| c as f64).sum::<f64>() / 10.0;
            let change = mean_last / start as f64 - 1.0;
            eprintln!(
                "level set {on}: radius {radius} spacings, {start} samples at the start, after \
                 1, 10, 20, 40 resamplings {} {} {} {} (last ten's mean {:+.1} %)",
                counts[0],
                counts[9],
                counts[19],
                counts[39],
                change * 100.0
            );
            let lost = (start as f64 - counts[19] as f64) / 20.0;
            eprintln!(
                "level set {on}: radius {radius} spacings, lost {lost:.2} samples per resampling"
            );
            seen.push((on, radius, lost));
        }
    }
    let on: Vec<f64> = seen.iter().filter(|s| s.0).map(|s| s.2).collect();
    for &(_, radius, lost) in seen.iter().filter(|s| s.0) {
        assert!(
            lost > ROUND_LOSS[0] && lost < ROUND_LOSS[1],
            "radius {radius}: {lost} samples per resampling"
        );
    }
    let (lo, hi) = on
        .iter()
        .fold((f64::MAX, 0.0f64), |(a, b), &x| (a.min(x), b.max(x)));
    assert!(
        hi < 2.0 * lo,
        "the loss should not depend on the radius: {on:?}"
    );
}

/// A resampling moves the crust onto a new lattice and must neither make nor destroy continental
/// area beyond the lattice's own noise: on a static planet, and where an ocean runs under a
/// continent (before 2026-10-02 the continent's outer half-spacing went to the subducting plate at
/// every resampling; build log "resampling crust loss"). Tolerance: the mean change over six
/// lattices within 0.2 % of the continental area (on this 20k planet one resampling's noise is
/// about ±0.15 %, and the old loss at the converging margin about 0.6 %).
#[test]
fn resampling_conserves_continental_area() {
    for (turn, label) in [(0.0, "static"), (0.1, "converging")] {
        let (st, before, after) = continental_area_through_resampling(turn, 6);
        if turn > 0.0 {
            assert_eq!(
                st.contacts.len(),
                1,
                "the ocean subducts under the continent"
            );
            assert_eq!((st.contacts[0].under, st.contacts[0].over), (1, 0));
            assert!(st.samples.iter().filter(|s| s.is_hidden()).count() > 100);
        }
        let mean = after.iter().sum::<f64>() / after.len() as f64;
        let change = (mean - before) / before;
        eprintln!(
            "{label}: continental area {before:.0} km², after {after:?} ({:+.3} %)",
            change * 100.0
        );
        assert!(
            change.abs() < 0.002,
            "{label}: continental area {before:.0} km² → mean {mean:.0} km² ({:+.3} %)",
            change * 100.0
        );
    }
}

/// Continental crust hidden under the overriding plate accretes to the column on top at a
/// resampling (forced-subducted continental crust is not lost), unless a collision hid it: then
/// the belt already stacked its volume (`belts.rs`) and the resampling consumes it instead of
/// adding it a second time (ledger PLT-276). Each is counted in its global.
#[test]
fn a_resampling_accretes_hidden_continent_once() {
    let p = preset(20_000, 20.0, "supercontinent", 5);
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let (mut st, geo, pos) = two_plates(&p, &env, 0.1);
    let mut hidden = 0.0;
    for s in st.samples.iter_mut().filter(|s| s.is_hidden()) {
        s.kind = CONTINENTAL;
        s.pools = pools::continental_column();
        s.thickness_m = pools::thickness(&s.pools);
        hidden += f64::from(s.thickness_m);
    }
    assert!(hidden > 0.0);
    let conserved = |st: &PlateState| -> f64 {
        st.samples
            .iter()
            .map(|s| {
                s.pools[..state::pool::CONSERVED]
                    .iter()
                    .map(|&x| f64::from(x))
                    .sum::<f64>()
            })
            .sum()
    };
    let mut out = Vec::new();
    for stacked in [false, true] {
        let mut s2 = st.clone();
        for s in s2.samples.iter_mut().filter(|s| s.is_hidden()) {
            s.stacked = stacked;
        }
        let mut world = WorldState::new(1, 16_384, p.t_start_years());
        ridges::resample(
            &mut s2,
            &mut world,
            &pos,
            &Tree::over(&pos, |_| true),
            &geo,
            &env,
        )
        .unwrap();
        let g = |k: &str| world.globals.get(k).copied().unwrap_or(0.0);
        out.push((
            conserved(&s2),
            g(ridges::ACCRETED_GLOBAL),
            g(ridges::STACKED_CONSUMED_GLOBAL),
        ));
    }
    let (accreted, consumed) = (out[0], out[1]);
    let area = env.norm.sample_area_km2();
    // What the hidden continent adds when it accretes: most of its conserved column.
    let gap = (accreted.0 - consumed.0) * area / 1000.0;
    assert!(gap > 0.0, "the consumed run keeps less crust: {gap} km³");
    assert!(consumed.2 > 0.0 && accreted.2 == 0.0);
    assert!(
        (consumed.2 - gap).abs() < 0.05 * gap,
        "the consumed global {} km³ is what was not accreted, {gap} km³",
        consumed.2
    );
    assert!((accreted.1 - consumed.1 - gap).abs() < 0.05 * gap);
}

/// A collision state with nothing measured yet: what `ridges::resample` reads of it is only that
/// it is there.
fn a_collision() -> state::Collision {
    state::Collision {
        onset_myr: 0.0,
        onset_step: 0,
        last_step: 0,
        last_myr: 0.0,
        shortening_km: 0.0,
        area_km2: 0.0,
        convergence_mm_yr: 0.0,
        onset_convergence_mm_yr: 0.0,
        peak_convergence_mm_yr: 0.0,
        later_convergence_mm_yr: None,
        front_km: 0.0,
        peak_front_km: 0.0,
        centroid: [0.0, 0.0, 1.0],
        normal: [0.0, 1.0, 0.0],
        continental_over: 1.0,
        stalled_since_myr: None,
        jam_km: None,
        belt: Default::default(),
    }
}

/// The visible continental crust of a plate colliding under another, where the overriding
/// plate's footprint covers it at a resampling, crossed the front and was counted by the belt:
/// the resampling consumes it as it consumes the hidden part (ledger PLT-292). Without the
/// collision (forced subduction of a continent, PLT-228) it accretes as before.
#[test]
fn a_resampling_consumes_the_visible_crust_a_collision_took() {
    let p = preset(20_000, 20.0, "supercontinent", 5);
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let (mut st, geo, pos) = two_plates(&p, &env, 0.1);
    // Both plates continental; the hidden part is not `stacked`, so it accretes in both runs and
    // only the visible part inside the overriding plate's footprint differs.
    for s in st.samples.iter_mut() {
        s.kind = CONTINENTAL;
        s.pools = pools::continental_column();
        s.thickness_m = pools::thickness(&s.pools);
        s.stacked = false;
    }
    let k = st
        .contacts
        .iter()
        .position(|c| c.under == 1 && c.over == 0)
        .expect("plate 1 subducts under plate 0");
    let mut out = Vec::new();
    for collides in [false, true] {
        let mut s2 = st.clone();
        s2.contacts[k].collision = collides.then(a_collision);
        let mut world = WorldState::new(1, 16_384, p.t_start_years());
        ridges::resample(
            &mut s2,
            &mut world,
            &pos,
            &Tree::over(&pos, |_| true),
            &geo,
            &env,
        )
        .unwrap();
        let g = |k: &str| world.globals.get(k).copied().unwrap_or(0.0);
        out.push((
            g(ridges::ACCRETED_VISIBLE_GLOBAL),
            g(ridges::STACKED_CONSUMED_GLOBAL),
            g(ridges::ACCRETED_HIDDEN_CONTINENT_GLOBAL),
        ));
    }
    let (forced, collided) = (out[0], out[1]);
    assert!(
        forced.0 > 0.0,
        "no visible crust of plate 1 in plate 0's footprint"
    );
    assert_eq!(forced.1, 0.0);
    assert!(
        collided.1 > 0.0 && collided.0 < 0.05 * forced.0,
        "{collided:?} {forced:?}"
    );
    assert!(
        (collided.1 - forced.0).abs() < 0.05 * forced.0,
        "consumed {} km³ against {} km³ accreted without the collision",
        collided.1,
        forced.0
    );
    // The hidden, unstacked crust accretes either way.
    assert!(forced.2 > 0.0 && (collided.2 - forced.2).abs() < 1e-6 * forced.2);
}

/// One-way subduction (`tectonics.one_way_subduction`, ledger PLT-542): ocean floor that went
/// down a trench, under the overriding plate's forearc and arc, stays down when that plate moves
/// off it, and the resampling gives the ground it leaves new sea floor and counts the crust the
/// slab took; off, the old ocean floor shows again and the resampling keeps it. With the switch
/// on, ocean floor under crust that no front reaches (no fronts found, as where plates only slide
/// past each other) and continental crust (too buoyant to stay down) are judged each step as
/// before. The ocean of `converging_plates` runs under the continent; a step later, with the
/// plates where they were, the hiding is found again; then the ocean turns back where it was
/// before the turn, which leaves no overlap, and the planet is resampled.
#[test]
fn one_way_subduction_keeps_the_slab_down() {
    const SEDIMENT_M: f64 = 200.0;
    let p0 = preset(20_000, 20.0, "supercontinent", 5);
    // (samples under, held, of those under shown again after the turn back, young sea floor
    // after the resampling, the slab's global).
    let run = |on: bool, fronts: bool, continental: bool| {
        let mut p = p0.clone();
        p.tectonics.one_way_subduction = on;
        let stage = PlatesStage::new(p.clone());
        let env = stage.env(&p.tectonics, 1);
        let (mut st, geo, pos) = converging_plates(&p, &env);
        // The ocean floor's sediment: what the slab takes counts by its conserved pools.
        for s in st.samples.iter_mut().filter(|s| s.kind == OCEANIC) {
            s.pools[state::pool::SEDIMENT] = SEDIMENT_M as f32;
            s.thickness_m = pools::thickness(&s.pools);
        }
        let under: Vec<usize> = (0..st.samples.len())
            .filter(|&i| st.samples[i].is_hidden())
            .collect();
        if continental {
            for &i in &under {
                let s = &mut st.samples[i];
                s.kind = CONTINENTAL;
                s.pools = pools::continental_column();
                s.thickness_m = pools::thickness(&s.pools);
            }
        }
        if fronts {
            let mut cones = vec![[0.0; 2]; st.samples.len()];
            subduction::fronts_and_uplift(&mut st, &geo, &pos, &env, &mut cones, 0.0);
        }
        let mut world = WorldState::new(1, 16_384, p.t_start_years());
        let env2 = stage.env(&p.tectonics, 2);
        let tree = Tree::over(&pos, |_| true);
        let overlaps = subduction::overlaps(&st, &geo, &pos, &tree, &env2);
        subduction::resolve(&mut st, &mut world, &geo, &overlaps, &pos, &env2).unwrap();
        let held = st
            .one_way
            .as_ref()
            .map_or(0, |ow| ow.held.iter().filter(|&&h| h).count());
        st.groups[1].rot = [0.0, 0.0, 0.0, 1.0];
        let pos = geometry::positions(&st, &geo);
        let env3 = stage.env(&p.tectonics, 3);
        let tree = Tree::over(&pos, |_| true);
        let overlaps = subduction::overlaps(&st, &geo, &pos, &tree, &env3);
        subduction::resolve(&mut st, &mut world, &geo, &overlaps, &pos, &env3).unwrap();
        let shown = under
            .iter()
            .filter(|&&i| !st.samples[i].is_hidden())
            .count();
        ridges::resample(&mut st, &mut world, &pos, &tree, &geo, &env3).unwrap();
        let young = st
            .samples
            .iter()
            .filter(|s| s.kind == OCEANIC && !s.is_hidden() && s.age_myr(env3.t_myr) < 30.0)
            .count();
        let slab = world.globals.get(ridges::SLAB_GLOBAL).copied();
        (under.len(), held, shown, young, slab)
    };
    let area = PlatesStage::new(p0.clone())
        .env(&p0.tectonics, 1)
        .norm
        .sample_area_km2();
    let (under, held, shown, young, slab) = run(true, true, false);
    eprintln!("one way: {under} under, {held} held, {shown} shown again, {young} young, {slab:?}");
    assert!(under > 20, "{under} samples under");
    // Held: the ocean floor under the forearc and arc (the turn put some of it deeper).
    assert!(held > 0 && held <= under, "{held} of {under} held");
    assert_eq!(shown, under - held, "held crust showed again");
    assert!(
        2 * young >= held && young <= 2 * held,
        "{young} new samples of young sea floor where {held} held samples were"
    );
    // The new sea floor's points took the held floor's sediment down with the slab.
    let expected = young as f64 * SEDIMENT_M * area / 1000.0;
    assert!(
        slab.is_some_and(|v| (v - expected).abs() < 0.05 * expected),
        "{slab:?} km³ against {expected} km³"
    );
    for (on, fronts, continental, label) in [
        (false, true, false, "off"),
        (true, false, false, "no front"),
        (true, true, true, "continental"),
    ] {
        let (under, held, shown, young, slab) = run(on, fronts, continental);
        assert_eq!(held, 0, "{label}");
        assert_eq!(shown, under, "{label}: crust stayed under");
        assert_eq!(young, 0, "{label}: young sea floor");
        assert_eq!(slab.is_some(), on, "{label}");
        assert!(slab.unwrap_or(0.0) == 0.0, "{label}: {slab:?}");
    }
}

/// Ocean floor with a normal continental column's crystalline crust (T_n) turns continental
/// where it meets another plate (ledger PLT-289), whatever stacked its crust. Crust as thick as
/// the transition's ocean side (20 km) stays ocean floor at the contact, and T_n of it away from
/// any contact stays ocean floor too.
#[test]
fn ocean_floor_as_thick_as_a_continent_turns_continental_at_a_contact() {
    let p = preset(20_000, 20.0, "supercontinent", 5);
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let (mut st, geo, pos) = two_plates(&p, &env, 0.1);
    assert_eq!(st.counters.buoyant, 0);
    let buoyant = super::transition::continental_m();
    let at_contact: Vec<usize> = (0..st.samples.len())
        .filter(|&i| st.samples[i].is_hidden() && st.samples[i].kind == OCEANIC)
        .take(2)
        .collect();
    let far = (0..st.samples.len())
        .find(|&i| {
            let s = &st.samples[i];
            !s.is_hidden() && s.kind == OCEANIC && pos[i].z < -0.5
        })
        .unwrap();
    // (sample, crystalline crust in m, turns continental)
    let cases = [
        (at_contact[0], buoyant + 500.0, true),
        (at_contact[1], 20_000.0, false),
        (far, buoyant + 5000.0, false),
    ];
    for &(i, crust, _) in &cases {
        let s = &mut st.samples[i];
        s.pools = pools::oceanic_column();
        s.pools[state::pool::METAMORPHIC] +=
            (crust - super::transition::crystalline_m(&s.pools)) as f32;
        s.thickness_m = pools::thickness(&s.pools);
    }
    let tree = Tree::over(&pos, |_| true);
    let overlaps = subduction::overlaps(&st, &geo, &pos, &tree, &env);
    let mut world = WorldState::new(1, 16_384, p.t_start_years());
    subduction::resolve(&mut st, &mut world, &geo, &overlaps, &pos, &env).unwrap();
    for &(i, crust, turns) in &cases {
        let place = if i == far {
            "away from any contact"
        } else {
            "at the contact"
        };
        assert_eq!(
            st.samples[i].kind == CONTINENTAL,
            turns,
            "{crust} m of crystalline crust {place}"
        );
    }
    assert_eq!(st.counters.buoyant, 1);
}

/// Island-arc crust turns continental by the arc crust it has built (`arc_mature_crust_km` of
/// crust with the ocean's own), not by its height: a mature arc below the sea becomes continent, a young arc whose
/// cones stand above it does not, and neither does a mature arc's crust hidden in a trench.
/// (Until 2026-10-02 only arcs above sea level converted, so flexure, which bends arcs down,
/// cut the continents' growth.)
#[test]
fn arcs_turn_continental_by_their_crust_not_their_height() {
    let p = preset(6000, 20.0, "supercontinent", 3);
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let (mut st, geo) = init::initial_state(&p, &env).unwrap();
    let k = &p.tectonics;
    let mature = (k.arc_mature_crust_km * 1000.0 - super::transition::oceanic_m()) as f32;
    assert!(mature > 0.0);
    let oceanic: Vec<usize> = (0..st.samples.len())
        .filter(|&i| st.samples[i].kind == OCEANIC)
        .take(4)
        .collect();
    // (arc magma, elevation, hidden) and whether it should turn continental.
    let cases = [
        (mature * 1.05, -1800.0, false, true),
        (mature * 0.5, 600.0, false, false),
        (mature * 2.0, -3000.0, true, false),
        (0.0, 300.0, false, false),
    ];
    for (&i, &(arc, z, hidden, _)) in oceanic.iter().zip(&cases) {
        let s = &mut st.samples[i];
        s.arc_oceanic_m = arc;
        s.elevation_m = z;
        s.flags = if hidden { state::HIDDEN } else { ARC };
    }
    let z0: Vec<f32> = st.samples.iter().map(|s| s.elevation_m).collect();
    finish_samples(&mut st, &geo, &z0, &env);
    let ocean = super::transition::oceanic_m() as f32;
    for (&i, &(arc, z, hidden, turns)) in oceanic.iter().zip(&cases) {
        let s = &st.samples[i];
        assert_eq!(
            s.kind == CONTINENTAL,
            turns,
            "arc magma {arc} m (mature at {mature} m), elevation {z} m, hidden {hidden}"
        );
        // The ocean crust under the arc stays in the column, as its basement, and is no
        // continental crust (PLT-331).
        if turns {
            assert!((s.basement() - ocean).abs() < 1.0, "{s:?}");
            assert!((s.own_thickness_m() - (s.thickness_m - ocean)).abs() < 1.0);
            let own: f32 = s.own_pools().iter().sum();
            assert!((own - s.own_thickness_m()).abs() < 1.0, "{own} {s:?}");
        } else {
            assert_eq!(s.basement(), 0.0);
            assert_eq!(s.own_thickness_m(), s.thickness_m);
        }
    }
}

/// A rift that did not open heals (force-balance design §3.4): two plates a rift separated that
/// move together for `weld_after_myr` become one again, the larger keeping its id; two that move
/// past each other faster than the weld speed stay two.
#[test]
fn a_rift_that_does_not_open_heals_and_one_that_opens_does_not() {
    let p = preset(6000, 20.0, "supercontinent", 3);
    let stage = PlatesStage::new(p.clone());
    let k = &p.tectonics;
    for apart_mm_yr in [0.0, 20.0] {
        let env0 = stage.env(&p.tectonics, 1);
        let (mut st, geo, pos) = two_plates(&p, &env0, 0.0);
        let mut world = WorldState::new(1, 16_384, p.t_start_years());
        let e = events::event(&mut world, env0.t_myr, events::RIFT_ONSET, DVec3::Z);
        let rift = world.emit(e).unwrap();
        st.rift_links.push(state::RiftLink {
            a: 0,
            b: 1,
            event: rift,
            quiet_since_myr: None,
        });
        // Plate 1 turns about the z axis relative to plate 0: along the equator a strike-slip
        // of `apart_mm_yr`.
        st.plates[1].omega = (DVec3::Z * env0.norm.omega_of_speed(apart_mm_yr)).to_array();
        let steps = (k.weld_after_myr / env0.dt_myr).ceil() as u64 + 2;
        let mut healed_at = None;
        for step in 1..=steps {
            let env = stage.env(&p.tectonics, step);
            let healed = rifting::heal(&mut st, &mut world, &geo, &pos, &env).unwrap();
            if !healed.is_empty() {
                healed_at = Some(step);
                break;
            }
        }
        if apart_mm_yr == 0.0 {
            let at = healed_at.expect("a rift that never opened did not heal");
            assert!(
                at as f64 * env0.dt_myr >= k.weld_after_myr - 1e-9,
                "healed at step {at}"
            );
            assert_eq!(st.alive().count(), 1);
            assert!(st.rift_links.is_empty());
            assert_eq!(st.counters.healed, 1);
            let died = world
                .events
                .iter()
                .filter(|e| e.kind == events::PLATE_DEATH)
                .count();
            assert_eq!(died, 1);
        } else {
            assert_eq!(healed_at, None, "a rift that opened healed");
            assert_eq!(st.alive().count(), 2);
        }
    }
}

/// The two-plate planet of [`two_plates`] with the ocean converging on the continent at 50 mm/yr
/// (plate task 8's tests of the arcs and the cordillera).
fn converging_plates(p: &PlanetPreset, env: &Env<'_>) -> (PlateState, Geo, Vec<DVec3>) {
    let (mut st, geo, pos) = two_plates(p, env, 0.1);
    st.plates[1].omega = (DVec3::X * env.norm.omega_of_speed(50.0)).to_array();
    (st, geo, pos)
}

/// An arc's front is where the plates converge across their boundary (PLT-330): a contact whose
/// plates slide past each other has no front, however its lattice edges zigzag, and the same
/// contact converging head-on has one along its length.
#[test]
fn a_contact_whose_plates_slide_past_has_no_arc_front() {
    let p = preset(20_000, 20.0, "supercontinent", 5);
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let fronts = |axis: DVec3| {
        let (mut st, geo, pos) = two_plates(&p, &env, 0.1);
        assert!(!st.contacts.is_empty());
        st.plates[1].omega = (axis * env.norm.omega_of_speed(50.0)).to_array();
        subduction::fronts_begin(&st, &geo, &pos, &env).front_count()
    };
    // About x: the ocean moves north across the equator, towards the continent where y > 0.
    let head_on = fronts(DVec3::X);
    // About z: along the equator, east-west.
    let sliding = fronts(DVec3::Z);
    assert!(head_on > 20, "{head_on}");
    assert_eq!(sliding, 0);
}

/// Three plates for a front of known length (the front lane, step 9), in the planet's frame
/// turned by `frame`: plate 0 the northern hemisphere, continental; plate 1 oceanic, the box of longitudes 0 to `lon_deg` and latitudes
/// `-box_deg` to 0; plate 2 the rest of the south, oceanic. Plate 1's group is turned `turn`
/// radians north about the equator's axis square to the box's middle, so its edge on the equator
/// runs under the continent along its whole length (its southern edge parts from plate 2, its
/// sides slide past it), and it moves on north at `mm_yr`. Overlaps resolved as in
/// [`plates_split`]. The front is the equator from longitude 0 to `lon_deg`, straight and normal
/// to the convergence, which falls off as the cosine of the distance from the box's middle.
#[allow(clippy::too_many_arguments)]
fn a_front_of_known_length(
    p: &PlanetPreset,
    env: &Env<'_>,
    frame: glam::DQuat,
    lon_deg: f64,
    box_deg: f64,
    turn: f64,
    mm_yr: f64,
) -> (PlateState, Geo, Vec<DVec3>) {
    let (mut st, geo) = init::initial_state(p, env).unwrap();
    let pts = geo.points();
    let half = 0.5 * lon_deg.to_radians();
    // Rotation about `axis` carries the box's middle (cos half, sin half, 0) north.
    let axis = frame * DVec3::new(half.sin(), -half.cos(), 0.0);
    let q = glam::DQuat::from_axis_angle(axis, turn);
    let to_frame = frame.inverse();
    for (i, s) in st.samples.iter_mut().enumerate() {
        let x = to_frame * pts[i];
        let lon = x.y.atan2(x.x).to_degrees();
        let lat = x.z.clamp(-1.0, 1.0).asin().to_degrees();
        let plate = if x.z > 0.0 {
            0
        } else if (0.0..lon_deg).contains(&lon) && lat > -box_deg {
            1
        } else {
            2
        };
        *s = state::Sample::blank(plate);
        if plate == 0 {
            s.kind = CONTINENTAL;
            s.pools = pools::continental_column();
            s.elevation_m = 400.0;
        } else {
            s.pools = pools::oceanic_column();
            s.elevation_m = -4000.0;
            s.formed_myr = (env.t_myr - 60.0) as f32;
        }
        s.thickness_m = pools::thickness(&s.pools);
    }
    let still = [0.0, 0.0, 0.0, 1.0];
    st.groups = (0..3)
        .map(|g| state::Group {
            plate: g,
            rot: if g == 1 { [q.x, q.y, q.z, q.w] } else { still },
        })
        .collect();
    for (k, pl) in st.plates.iter_mut().enumerate() {
        pl.alive = k < 3;
        pl.omega = [0.0; 3];
    }
    st.plates[1].omega = (axis * env.norm.omega_of_speed(mm_yr)).to_array();
    st.contacts.clear();
    st.ridges.clear();
    st.rift_links.clear();
    let pos = geometry::positions(&st, &geo);
    st.recount(&pos);
    let tree = Tree::over(&pos, |_| true);
    let overlaps = subduction::overlaps(&st, &geo, &pos, &tree, env);
    let mut world = WorldState::new(1, 16_384, p.t_start_years());
    subduction::resolve(&mut st, &mut world, &geo, &overlaps, &pos, env).unwrap();
    (st, geo, pos)
}

/// The arcs' band on a straight front of known length (the front lane, step 9, before any law
/// change; plates.md, "Step 9"). The arcs make their crust over a band `arc_distance_km` behind
/// the front (`w_arc_of`, PLT-045, a Gaussian of `arc_width_km` whose integral across the arc is
/// √π · `arc_width_km`), on the lattice distance from the nearest front sample (`fronts_reach`).
/// Past a front's end that distance runs round the end, so a front of length L has a band of
/// L + π · `arc_distance_km` (a quarter circle at each end, on the overriding side only), where
/// Earth's arcs stop near the slab's edge. Fronts of 10 to 60 degrees of a great circle, on the
/// equator and on a great circle tilted 60 degrees from it: the band split at the meridians of
/// the front's ends into the part along the front and the part past its ends. The ends' part is
/// about the two quarter circles whatever the length; the part along the front is short of its
/// length by the lattice's distance, which runs longer than the straight line across the band (up
/// to 2/√3 on a hexagonal lattice, by the edges' direction, which turns along the Fibonacci
/// lattice's spiral), so the band is narrower than its width. Measured (2026-10-24, 40,000
/// samples): see plates.md, "Step 9".
#[test]
fn the_arcs_band_runs_round_a_fronts_ends() {
    let p = preset(40_000, 20.0, "supercontinent", 5);
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let k = &p.tectonics;
    let wrap = std::f64::consts::PI * k.arc_distance_km;
    let per_sample = env.norm.sample_area_km2() / (std::f64::consts::PI.sqrt() * k.arc_width_km);
    for (name, frame) in [
        ("equator", glam::DQuat::IDENTITY),
        ("tilted", glam::DQuat::from_rotation_y(60f64.to_radians())),
    ] {
        let to_frame = frame.inverse();
        for lon_deg in [10.0_f64, 20.0, 40.0, 60.0] {
            let (mut st, geo, pos) =
                a_front_of_known_length(&p, &env, frame, lon_deg, 15.0, 0.02, 50.0);
            let length = env.norm.km(lon_deg.to_radians());
            // The band by where it lies: between the meridians of the front's ends, or past them.
            let begun = subduction::fronts_begin(&st, &geo, &pos, &env);
            let (dist, _, reached) = subduction::fronts_reach(&geo, &begun, &env);
            let (mut along, mut ends) = (0.0, 0.0);
            for &i in &reached {
                let i = i as usize;
                if !dist[i].is_finite() || st.samples[i].is_hidden() {
                    continue;
                }
                let w = subduction::w_arc_of(env.norm.km(dist[i]), k) * per_sample;
                let x = to_frame * pos[i];
                let lon = x.y.atan2(x.x).to_degrees();
                if (0.0..=lon_deg).contains(&lon) {
                    along += w;
                } else {
                    ends += w;
                }
            }
            let mut cones = vec![[0.0; 2]; st.samples.len()];
            let pull = subduction::fronts_and_uplift(&mut st, &geo, &pos, &env, &mut cones, 0.0);
            let straight = pull.front_straight_km_myr / env.dt_myr;
            let band = pull.arc_band_km_myr / env.dt_myr;
            eprintln!(
                "{name}, front of {lon_deg} degrees, {length:.0} km: straight {straight:.0} km \
                 ({:.3}), band {band:.0} km ({:.3} of the straight front): along the front \
                 {along:.0} km ({:.3} of the straight front), past its ends {ends:.0} km ({:.2} \
                 of pi times the arc's distance)",
                straight / length,
                band / straight,
                along / straight,
                ends / wrap,
            );
            assert!(
                (straight / length - 1.0).abs() < 0.05,
                "{name}: the straight front {straight:.0} km of {length:.0}"
            );
            // The split is the measure's band.
            assert!(
                ((along + ends) / band - 1.0).abs() < 0.01,
                "{name}, {lon_deg} degrees: {along:.0} + {ends:.0} km against {band:.0}"
            );
            // Past the ends: about the two quarter circles, at any length. The lattice's front
            // ends within a spacing of the meridians the split is made at, which moves up to
            // about a spacing of band at each end (0.15 of the wrap at 40,000 samples) from one
            // part to the other.
            assert!(
                (0.45..1.2).contains(&(ends / wrap)),
                "{name}, {lon_deg} degrees: past the ends {ends:.0} km"
            );
            // Along the front: no more than its length, and no less than a hexagonal lattice's
            // longest path across allows, less that spacing at each end.
            assert!(
                (0.75..1.03).contains(&(along / straight)),
                "{name}, {lon_deg} degrees: along the front {along:.0} km"
            );
        }
    }
}

/// The slab pull at the arcs' fronts (`tectonics.front_pull`, PLT-560; plates.md, "Step 9"), on
/// [`a_front_of_known_length`]'s front of 40 degrees, plate 1 converging at 50 mm/yr at an angle
/// θ to the front's normal. On, a full-length slab's pull has the torque of the reference pull
/// per metre (FRC-002, by the slab's age, FRC-004) over the front's straight length along its
/// normal, whatever θ while the plates converge beyond Bird's 20 degrees of a transform (PLT-311),
/// and a slab grows from nothing by the convergence along the normal; at θ = 80 and 90 degrees the
/// box's edge has no front (a corner sample or two may). Off (the old law), the pull follows each
/// sample's fastest-converging lattice edge,
/// one spacing per sample, the slab grows by the convergence along that edge, and a transform's
/// zigzag still feeds a slab. Measured (2026-10-24, 40,000 samples): on, the pull 0.988 and 0.996
/// of the front's (equator, tilted) at 0 and 45 degrees and 0.956 and 0.964 at 60, within a degree
/// of the normal, the slab fed 0.97 to 0.99 of the convergence across the front's middle; off,
/// 1.095 and 1.052 at 0 degrees, 18 to 24 degrees off the normal at 45 and 60, the slab fed up to
/// 1.30 times the convergence at 60, and on the equator at 90 degrees (a transform) 0.46 of the
/// front's pull and a slab fed at 43.5 km/Myr.
#[test]
fn the_slab_pull_takes_the_arcs_fronts_along_their_normal() {
    let lon_deg = 40.0_f64;
    let half = 0.5 * lon_deg.to_radians();
    let thetas = [0.0_f64, 45.0, 60.0, 80.0, 90.0];
    for (name, frame) in [
        ("equator", glam::DQuat::IDENTITY),
        ("tilted", glam::DQuat::from_rotation_y(60f64.to_radians())),
    ] {
        // (the pull's torque on plate 1, N m; the slab fed from nothing, km/Myr) by θ, on and off.
        let mut got: Vec<Vec<(DVec3, f64)>> = Vec::new();
        let mut want = DVec3::ZERO;
        for on in [true, false] {
            let mut p = preset(40_000, 20.0, "supercontinent", 5);
            p.tectonics.front_pull = on;
            let stage = PlatesStage::new(p.clone());
            let env = stage.env(&p.tectonics, 1);
            let k = &p.tectonics;
            let mut row = Vec::new();
            for &theta in &thetas {
                let (mut st, geo, pos) =
                    a_front_of_known_length(&p, &env, frame, lon_deg, 15.0, 0.02, 50.0);
                let t = theta.to_radians();
                let axis = frame
                    * (DVec3::new(half.sin(), -half.cos(), 0.0) * t.cos() + DVec3::Z * t.sin());
                st.plates[1].omega = (axis * env.norm.omega_of_speed(50.0)).to_array();
                st.contacts.retain(|c| (c.under, c.over) == (1, 0));
                assert_eq!(st.contacts.len(), 1, "{name}: the contact");
                // The slab as it grows from nothing in one step …
                let mut grown = st.clone();
                grown.slabs.clear();
                forces::step(&mut grown, &geo, &pos, &env, DVec3::Z, 0.0);
                let fed = grown
                    .slabs
                    .iter()
                    .find(|s| (s.under, s.over) == (1, 0))
                    .map_or(0.0, |s| s.km)
                    / env.dt_myr;
                // … and the pull of a full-length one.
                st.slabs = vec![state::SlabTrack {
                    under: 1,
                    over: 0,
                    km: k.slab_depth_km,
                    ..state::SlabTrack::default()
                }];
                let torque = forces::slab_torques(&st, &geo, &pos, &env)[1].0;
                if on && theta == 0.0 {
                    // The reference pull per metre over the front along its normal (north, in
                    // the frame): R x × ẑ over the equator from 0 to `lon_deg`.
                    let c =
                        forces::Coeffs::new(k, &env.norm, env.gravity, st.a0_km2, DVec3::Z, 0.0);
                    let age = (0..st.samples.len())
                        .find(|&i| st.plate_of(i) == 1)
                        .map(|i| st.samples[i].age_myr(env.t_myr))
                        .unwrap();
                    let per_m = c.pull_n_m * forces::slab_weight(age);
                    let d = lon_deg.to_radians();
                    want = frame
                        * DVec3::new(1.0 - d.cos(), -d.sin(), 0.0)
                        * (per_m * c.radius_m * c.radius_m);
                }
                row.push((torque, fed));
            }
            got.push(row);
        }
        for (j, &theta) in thetas.iter().enumerate() {
            let [(t_on, fed_on), (t_off, fed_off)] = [got[0][j], got[1][j]];
            let angle = |t: DVec3| t.angle_between(want).to_degrees();
            eprintln!(
                "{name}, {theta} degrees from the normal: on, the pull {:.3} of the front's along \
                 its normal ({:.1} degrees off), the slab fed {fed_on:.1} km/Myr ({:.3} of the \
                 convergence across the front's middle); off, {:.3} ({:.1} degrees off), \
                 {fed_off:.1} km/Myr ({:.3})",
                t_on.length() / want.length(),
                angle(t_on),
                fed_on / (50.0 * theta.to_radians().cos()),
                t_off.length() / want.length(),
                angle(t_off),
                fed_off / (50.0 * theta.to_radians().cos()),
            );
            if theta < 70.0 {
                // The front's straight length along its normal, whatever the motion's angle.
                assert!(
                    (0.95..1.03).contains(&(t_on.length() / want.length())),
                    "{name}, {theta} degrees: {t_on} against {want}"
                );
                assert!(angle(t_on) < 2.0, "{name}, {theta} degrees: {t_on}");
                // The same front at any angle while every sample of it converges beyond the
                // transform's 20 degrees; at 60 degrees the motion is 10 degrees from that at the
                // front's middle and less at its ends, where the boundary's normal (the chord of
                // about five spacings) turns, so a sample or two of each end drops out.
                let tol = if theta <= 45.0 { 1e-9 } else { 0.06 };
                assert!(
                    (t_on - got[0][0].0).length() <= tol * want.length(),
                    "{name}, {theta} degrees: {t_on} against the normal front's {}",
                    got[0][0].0
                );
                // Fed by the convergence along the normal (cos 20° at the front's ends).
                let across = fed_on / (50.0 * theta.to_radians().cos());
                assert!(
                    (0.93..1.0).contains(&across),
                    "{name}, {theta} degrees: {across}"
                );
            } else {
                // Within Bird's 20 degrees of a transform: no front along the box's edge. At its
                // corners the boundary turns towards plate 2's side, and the normal there (the
                // chord of the corner's crossings to plate 1) can face the motion: a corner sample
                // or two may still converge (tilted, 80 degrees: 0.068 of the front's pull), so
                // the slab fed is theirs, not the front's.
                assert!(
                    t_on.length() < 0.1 * want.length(),
                    "{name}, {theta} degrees: {t_on} against {want}"
                );
            }
        }
        // The old law: a slab at a transform, and an oblique front's slab fed faster than the
        // plates converge across it.
        assert!(
            got[1][4].1 > 0.0,
            "{name}: the old law feeds no slab at a transform"
        );
        assert!(got[1][1].1 > got[0][1].1, "{name}: {:?}", got);
    }
}

/// FRC-016's reference plate on the lattice: plate 1, oceanic floor of 80 Myr (a slab of weight 1,
/// FRC-004), a cap of the average starting plate's area about +X; plate 0 beyond the stretch of
/// its rim `trench_km` long about its northern point, plate 2 the rest, both continental. Plate
/// 1's group is turned `turn` rad north (about −Y), so the rim overlaps plate 0 there (the active
/// contact (1, 0)), and it moves north at `mm_yr`; the other contacts are cleared and a
/// full-length slab hangs from plate 1 under plate 0. Returns the trench's angle about the cap's
/// centre, rad, with the cap's angular radius.
fn a_reference_plate(
    p: &PlanetPreset,
    env: &Env<'_>,
    trench_km: f64,
    turn: f64,
    mm_yr: f64,
) -> (PlateState, Geo, Vec<DVec3>, f64, f64) {
    let (mut st, geo) = init::initial_state(p, env).unwrap();
    let pts = geo.points();
    let r_km = env.norm.model_km;
    let cos_r = 1.0 - st.a0_km2 / (2.0 * std::f64::consts::PI * r_km * r_km);
    let r = cos_r.acos();
    let alpha = trench_km / (r_km * r.sin());
    // The rim's points at an angle β about the centre from its northern point lie at
    // z = sin r cos β: the trench is the rim above its ends.
    let z_end = r.sin() * (0.5 * alpha).cos();
    for (i, s) in st.samples.iter_mut().enumerate() {
        let x = pts[i];
        let plate = if x.x > cos_r {
            1
        } else if x.z > z_end {
            0
        } else {
            2
        };
        *s = state::Sample::blank(plate);
        if plate == 1 {
            s.pools = pools::oceanic_column();
            s.elevation_m = -4000.0;
            s.formed_myr = (env.t_myr - forces::SLAB_AGE_REF_MYR) as f32;
        } else {
            s.kind = CONTINENTAL;
            s.pools = pools::continental_column();
            s.elevation_m = 400.0;
        }
        s.thickness_m = pools::thickness(&s.pools);
    }
    let q = glam::DQuat::from_axis_angle(-DVec3::Y, turn);
    let still = [0.0, 0.0, 0.0, 1.0];
    st.groups = (0..3)
        .map(|g| state::Group {
            plate: g,
            rot: if g == 1 { [q.x, q.y, q.z, q.w] } else { still },
        })
        .collect();
    for (k, pl) in st.plates.iter_mut().enumerate() {
        pl.alive = k < 3;
        pl.omega = [0.0; 3];
    }
    st.plates[1].omega = (-DVec3::Y * env.norm.omega_of_speed(mm_yr)).to_array();
    st.contacts.clear();
    st.ridges.clear();
    st.rift_links.clear();
    st.slabs.clear();
    let pos = geometry::positions(&st, &geo);
    st.recount(&pos);
    let tree = Tree::over(&pos, |_| true);
    let overlaps = subduction::overlaps(&st, &geo, &pos, &tree, env);
    let mut world = WorldState::new(1, 16_384, p.t_start_years());
    subduction::resolve(&mut st, &mut world, &geo, &overlaps, &pos, env).unwrap();
    st.contacts.retain(|c| (c.under, c.over) == (1, 0));
    assert_eq!(st.contacts.len(), 1, "the reference trench's contact");
    st.slabs = vec![state::SlabTrack {
        under: 1,
        over: 0,
        km: p.tectonics.slab_depth_km,
        ..state::SlabTrack::default()
    }];
    (st, geo, pos, alpha, r)
}

/// FRC-016's drag calibration on the slab pull's own count (the front lane, step 9; plates.md,
/// "Step 9"): the reference plate ([`a_reference_plate`]: the average starting plate's area, a
/// 3,000 km trench of a full-length 80-Myr slab, its neighbours held still by their keels, no
/// mantle flow) on the lattice, through the force balance. With `front_pull` (PLT-560) its pull's
/// torque is the reference pull per metre over the trench's shares along their normals, the
/// definition's own count (a straight trench along its normal), so the drag the definition gives
/// moves the plate, per the definition's trench, at the speed of the analytic ring's plate
/// (`forces.rs`, `the_reference_plate_moves_at_the_speed_scale`), and c_b needs no change.
/// Measured (2026-10-24, 40,000 samples): the lattice's trench 32 samples, shares 3,193 km for
/// the rim's 3,000 (crossing sums 3,358 km); on, the pull 1.060 of the rim's along its normal (0.5
/// degrees off), the plate at 53.2 mm/yr, 50.2 per the rim's trench (the ring 49.5 for the scale's
/// 45); off, the pull 1.235 of the rim's (11.0 degrees off), the plate at 55.4 mm/yr.
#[test]
fn the_reference_plate_on_the_lattice_moves_at_the_speed_scale() {
    let mut speeds = Vec::new();
    let mut scale = 0.0;
    for on in [true, false] {
        let mut p = preset(40_000, 20.0, "supercontinent", 5);
        scale = p.tectonics.plate_speed_scale_mm_yr;
        p.tectonics.front_pull = on;
        p.tectonics.mantle_flow_mm_yr = 0.0;
        p.tectonics.keel_drag = 1.0e4;
        let stage = PlatesStage::new(p.clone());
        let env = stage.env(&p.tectonics, 1);
        let k = &p.tectonics;
        let (mut st, geo, pos, alpha, r) = a_reference_plate(
            &p,
            &env,
            forces::REF_TRENCH_KM,
            0.02,
            k.plate_speed_scale_mm_yr,
        );
        let torque = forces::slab_torques(&st, &geo, &pos, &env)[1].0;
        let ans = forces::step(&mut st, &geo, &pos, &env, DVec3::Z, 0.0);
        assert!(ans.solved);
        let w = ans.omega[1].expect("plate 1 in the balance");
        let centroid = DVec3::from_array(st.plates[1].centroid);
        let speed = env.norm.velocity_mm_yr(w, centroid).length();
        let held = ans.omega[0].map_or(0.0, |w0| env.norm.velocity_mm_yr(w0, centroid).length());
        // The reference pull per metre over the rim's trench along its outward normal.
        let c = forces::Coeffs::new(k, &env.norm, env.gravity, st.a0_km2, DVec3::Z, 0.0);
        let steps = 2000;
        let mut want = DVec3::ZERO;
        for j in 0..steps {
            let beta = alpha * ((j as f64 + 0.5) / steps as f64 - 0.5);
            let x = DVec3::X * r.cos() + (DVec3::Z * beta.cos() + DVec3::Y * beta.sin()) * r.sin();
            let out = (x * r.cos() - DVec3::X).normalize();
            want += x.cross(out) * (alpha / steps as f64);
        }
        want *= c.pull_n_m * c.radius_m * c.radius_m * r.sin();
        // The lattice's trench: plate 0's samples with an edge to plate 1, their shares on their
        // normals, and their crossing sums (the boundary's normal times its straight length,
        // with no normal of the neighbourhood in it), as torques per unit pull.
        let n = st.samples.len();
        let plate_of: Vec<u32> = (0..n).map(|i| st.plate_of(i)).collect();
        let rots: Vec<glam::DQuat> = st.groups.iter().map(|g| kinematics::quat(g.rot)).collect();
        let (mut rows, mut share_km, mut cross_km) = (0usize, 0.0, 0.0);
        let (mut by_share, mut by_cross) = (DVec3::ZERO, DVec3::ZERO);
        for v in 0..n {
            if plate_of[v] != 0
                || st.samples[v].is_hidden()
                || !geo.edges(v as u32).any(|(u, _)| plate_of[u as usize] == 1)
            {
                continue;
            }
            let normal =
                shortening::boundary_normal(&geo, &pos, &rots, &st.samples, &plate_of, v, 1);
            let share = shortening::front_share_rad(
                &geo,
                &pos,
                &rots,
                &st.samples,
                &plate_of,
                v,
                1,
                normal,
            );
            let cross = shortening::crossing_sum(
                &geo,
                pos[v],
                rots[st.samples[v].group as usize],
                &plate_of,
                v as u32,
                1,
            );
            rows += 1;
            share_km += env.norm.km(share);
            cross_km += env.norm.km(cross.length());
            by_share += pos[v].cross(-normal) * share;
            by_cross += pos[v].cross(-cross);
        }
        let rim = want / (c.pull_n_m * c.radius_m * c.radius_m);
        eprintln!(
            "front_pull {on}: the reference plate moves at {speed:.1} mm/yr ({:.1} per the rim's \
             trench; the speed scale \
             {:.0}; its overriding neighbour {held:.2}); the pull {:.3} of the trench's along its \
             normal ({:.1} degrees off); the lattice's trench: {rows} samples, shares {share_km:.0} \
             km, crossing sums {cross_km:.0} km (the rim's {:.0}), the shares' torque {:.3} of the \
             rim's, the crossing sums' {:.3}",
            speed * rim.length() / by_share.length(),
            k.plate_speed_scale_mm_yr,
            torque.length() / want.length(),
            torque.angle_between(want).to_degrees(),
            forces::REF_TRENCH_KM,
            by_share.length() / rim.length(),
            by_cross.length() / rim.length(),
        );
        assert!(held < 0.01 * speed, "the neighbours move: {held}");
        if on {
            // The pull is the trench's shares along their normals, exactly …
            let shares = by_share * (c.pull_n_m * c.radius_m * c.radius_m);
            assert!(
                (torque - shares).length() <= 1e-4 * torque.length(),
                "{torque} against the shares' {shares}"
            );
            // … and the shares are the rim's trench, but for where the lattice's trench ends:
            // plate 0's samples end at z = sin r cos(α/2), a cut at α/2 to the rim (a shallow
            // angle), so each end lies within about a spacing over sin(α/2) of the rim's.
            assert!(
                (0.94..1.08).contains(&(by_share.length() / rim.length())),
                "the shares' torque {by_share} against the rim's {rim}"
            );
            assert!(torque.angle_between(want).to_degrees() < 2.0);
        }
        // The speed per the definition's trench. With the pull counted as the definition counts
        // it (on), the drag the definition gives moves the plate at the speed scale but for the
        // sphere's cap (the ring's 49.5 mm/yr for its 45, its own test's reading).
        let per_trench = speed * rim.length() / by_share.length();
        speeds.push((speed, per_trench));
    }
    let (on, off) = (speeds[0], speeds[1]);
    assert!(
        (0.95 * scale..1.25 * scale).contains(&on.1),
        "the reference plate per its trench {:.1} mm/yr, the scale {scale}",
        on.1
    );
    // The old count's pull is larger and moves the plate faster.
    assert!(off.0 > on.0, "{speeds:?}");
}

/// A class's row of [`the_margin_as_the_mix_sees_it`]: samples, normals over 70 degrees off, their
/// mean angle (degrees), samples beside hidden crust.
type MarginRow = (usize, usize, f64, usize);

/// The continent's margin towards plate 1 as the boundary mix sees it (`dynamics::boundary_mix`),
/// on the converging half (y > 0.2, away from the rotation's poles at ±x): per class (the mix's
/// index, `None` last), the samples, those whose boundary normal lies more than 70 degrees from
/// the margin's (−z, the lattice's equator), the mean of that angle, degrees, and those with a
/// hidden neighbour across; and the mix's ocean-continent and other km.
fn the_margin_as_the_mix_sees_it(
    st: &PlateState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env<'_>,
) -> (Vec<MarginRow>, f64, f64) {
    let n = st.samples.len();
    let classes: Vec<std::sync::atomic::AtomicU8> = (0..n)
        .map(|_| std::sync::atomic::AtomicU8::new(0))
        .collect();
    let mix = dynamics::boundary_mix(st, geo, pos, env, Some(&classes));
    let plate_of: Vec<u32> = (0..n).map(|i| st.plate_of(i)).collect();
    let rots: Vec<glam::DQuat> = st.groups.iter().map(|g| kinematics::quat(g.rot)).collect();
    let mut rows = vec![(0usize, 0usize, 0.0f64, 0usize); dynamics::mix::NAMES.len() + 1];
    for v in 0..n {
        let s = &st.samples[v];
        if s.is_hidden() || plate_of[v] != 0 || pos[v].y <= 0.2 {
            continue;
        }
        let across: Vec<u32> = geo
            .edges(v as u32)
            .map(|(u, _)| u)
            .filter(|&u| plate_of[u as usize] == 1)
            .collect();
        if across.is_empty() {
            continue;
        }
        let class = match classes[v].load(std::sync::atomic::Ordering::Relaxed) {
            0 => dynamics::mix::NAMES.len(),
            c => usize::from(c) - 1,
        };
        let normal = shortening::boundary_normal(geo, pos, &rots, &st.samples, &plate_of, v, 1);
        let angle = normal.angle_between(-DVec3::Z).to_degrees();
        let row = &mut rows[class];
        row.0 += 1;
        row.1 += usize::from(angle > 70.0);
        row.2 += angle;
        row.3 += usize::from(across.iter().any(|&u| st.samples[u as usize].is_hidden()));
    }
    for row in &mut rows {
        if row.0 > 0 {
            row.2 /= row.0 as f64;
        }
    }
    (
        rows,
        mix.km[dynamics::mix::OCEAN_CONTINENT],
        mix.km[dynamics::mix::OTHER],
    )
}

/// The ocean-continent dip after a rebuild, probed (the front lane, step 9; plates.md, "Step
/// 9"): the records lose 16,500 km of ocean-continent front within 2 Myr of a resampling, to the
/// boundary judged a transform, on the mature, fast margins. Here the two-plate planet of
/// [`converging_plates`] (an ocean moving north under a continent, head-on, at up to 50 mm/yr on
/// the y > 0 half of the equator) goes through a resampling with its contact carried (PLT-550),
/// as the plates step does it; the boundary mix before and after, and the continent's margin
/// sample by sample ([`the_margin_as_the_mix_sees_it`]). It shows no dip: the margin stays
/// ocean-continent through the resampling and a step on, its normals within a few degrees of the
/// convergence, the contact active. So the records' dip is not the resampling of a head-on margin
/// itself; what this fixture lacks (obliquity, plates of several groups, a real margin's shape,
/// other plates nearby) is where to look next.
#[test]
fn the_ocean_continent_front_through_a_resampling() {
    let p = preset(20_000, 20.0, "supercontinent", 5);
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let (mut st, geo, pos) = converging_plates(&p, &env);
    st.dynamics = Some(dynamics::Dynamics::default());
    let before = the_margin_as_the_mix_sees_it(&st, &geo, &pos, &env);
    let mut world = WorldState::new(1, 16_384, p.t_start_years());
    subduction::carry(&mut st, env.step, p.tectonics.contact_carry);
    let tree = Tree::over(&pos, |_| true);
    let geo = ridges::resample(&mut st, &mut world, &pos, &tree, &geo, &env).unwrap();
    let pos = geometry::positions(&st, &geo);
    let env2 = stage.env(&p.tectonics, 2);
    let tree = Tree::over(&pos, |_| true);
    let overlaps = subduction::overlaps(&st, &geo, &pos, &tree, &env2);
    subduction::resolve(&mut st, &mut world, &geo, &overlaps, &pos, &env2).unwrap();
    let after = the_margin_as_the_mix_sees_it(&st, &geo, &pos, &env2);
    let c = &st.contacts[0];
    eprintln!(
        "the contact after: active {}, carried {}, last step {}",
        c.active(env2.step),
        c.carried,
        c.last_step
    );
    // A step on, as the records' "1 to 2 Myr" row: the plates move on the new lattice and the
    // overlaps form again.
    kinematics::advance(&mut st, env2.dt_yr);
    let pos = geometry::positions(&st, &geo);
    st.recount(&pos);
    let env3 = stage.env(&p.tectonics, 3);
    let tree = Tree::over(&pos, |_| true);
    let overlaps = subduction::overlaps(&st, &geo, &pos, &tree, &env3);
    subduction::resolve(&mut st, &mut world, &geo, &overlaps, &pos, &env3).unwrap();
    let on = the_margin_as_the_mix_sees_it(&st, &geo, &pos, &env3);
    let c = &st.contacts[0];
    eprintln!(
        "the contact a step on: active {}, carried {}, last step {}",
        c.active(env3.step),
        c.carried,
        c.last_step
    );
    for (when, (rows, oc, other)) in [("before", &before), ("after", &after), ("a step on", &on)] {
        eprintln!("{when}: the mix's ocean-continent {oc:.0} km, other {other:.0} km");
        for (class, row) in rows.iter().enumerate() {
            if row.0 == 0 {
                continue;
            }
            let name = dynamics::mix::NAMES.get(class).copied().unwrap_or("none");
            eprintln!(
                "  {when}, {name}: {} margin samples, {} with the normal over 70 degrees off \
                 (mean {:.1}), {} beside hidden crust",
                row.0, row.1, row.2, row.3
            );
        }
    }
    // Measured 2026-10-24: 21,715 km before, 22,993 km after and a step on; no margin sample's
    // normal over 70 degrees off at any stage. The head-on margin holds.
    let oc = dynamics::mix::OCEAN_CONTINENT;
    assert!(before.0[oc].0 > 20, "{:?}", before.0);
    for (when, (rows, km, _)) in [("after", &after), ("a step on", &on)] {
        assert!(
            *km >= 0.9 * before.1,
            "{when}: {km:.0} km, {:.0} before",
            before.1
        );
        assert!(rows[oc].0 > 20, "{when}: {:?}", rows);
    }
    for (when, (rows, _, _)) in [("before", &before), ("after", &after), ("a step on", &on)] {
        let off: usize = rows.iter().map(|r| r.1).sum();
        assert_eq!(
            off, 0,
            "{when}: margin normals over 70 degrees off: {:?}",
            rows
        );
    }
    assert!(st.contacts[0].active(env3.step));
}

/// A subduction zone goes on through a resampling (`tectonics.contact_carry`, PLT-550; plates.md,
/// "Step 7"). The resampling gives both plates of [`converging_plates`] fresh crust with no
/// overlap, so at the next step, before the plates move, the contact sees none. Carried, it keeps
/// its front as it had it before the resampling; off, it has none until the overlap forms again
/// (the record's subducting length fell from 99,000 to 63,000 km in the first 2 Myr after a
/// resampling, base6 seeds 1 to 4). A carried contact whose plates slide past has no front and is
/// carried no more; one whose overlap is seen again is active by it, not carried.
#[test]
fn a_subduction_zone_goes_on_through_a_resampling() {
    let p0 = preset(20_000, 20.0, "supercontinent", 5);
    let run = |on: bool| {
        let mut p = p0.clone();
        p.tectonics.contact_carry = on;
        let stage = PlatesStage::new(p.clone());
        let env = stage.env(&p.tectonics, 1);
        let (mut st, geo, pos) = converging_plates(&p, &env);
        assert_eq!(st.contacts.len(), 1);
        assert!(!st.contacts[0].carried);
        let before = subduction::fronts_begin(&st, &geo, &pos, &env).front_count();
        let mut world = WorldState::new(1, 16_384, p.t_start_years());
        subduction::carry(&mut st, env.step, on);
        assert_eq!(st.contacts[0].carried, on);
        let tree = Tree::over(&pos, |_| true);
        let geo = ridges::resample(&mut st, &mut world, &pos, &tree, &geo, &env).unwrap();
        let pos = geometry::positions(&st, &geo);
        let env2 = stage.env(&p.tectonics, 2);
        let tree = Tree::over(&pos, |_| true);
        let overlaps = subduction::overlaps(&st, &geo, &pos, &tree, &env2);
        subduction::resolve(&mut st, &mut world, &geo, &overlaps, &pos, &env2).unwrap();
        let after = subduction::fronts_begin(&st, &geo, &pos, &env2).front_count();
        (st, geo, pos, world, before, after)
    };
    let (_, _, _, _, before_off, after_off) = run(false);
    let (st, geo, pos, world, before, after) = run(true);
    eprintln!("fronts: before {before} / {before_off}, after on {after}, off {after_off}");
    assert_eq!(st.contacts.len(), 1);
    assert_eq!(
        st.contacts[0].last_step, 1,
        "overlap seen after the resampling"
    );
    assert!(before > 20 && before == before_off, "{before} {before_off}");
    assert_eq!(after_off, 0, "a front without its overlap, off");
    assert!(
        after > before / 2 && after <= 2 * before,
        "{after} fronts carried, {before} before"
    );
    let p = p0.clone();
    let stage = PlatesStage::new(p.clone());
    let env2 = stage.env(&p.tectonics, 2);
    // The plates slide past each other (about z, along the equator): no front, and the carry ends.
    let mut sliding = st.clone();
    sliding.plates[1].omega = (DVec3::Z * env2.norm.omega_of_speed(50.0)).to_array();
    let mut cones = vec![[0.0; 2]; sliding.samples.len()];
    subduction::fronts_and_uplift(&mut sliding, &geo, &pos, &env2, &mut cones, 0.0);
    assert!(!sliding.contacts[0].carried, "carried with no front");
    // Converging on: the carry holds while the fronts stand.
    let mut held = st.clone();
    subduction::fronts_and_uplift(&mut held, &geo, &pos, &env2, &mut cones, 0.0);
    assert!(
        held.contacts[0].carried,
        "the carry ended with fronts standing"
    );
    // The ocean moves under the continent: the overlap is seen, and the contact is active by it.
    let mut world = world;
    let env3 = stage.env(&p.tectonics, 3);
    for g in held.groups.iter_mut().filter(|g| g.plate == 1) {
        let q = glam::DQuat::from_rotation_x(0.05);
        g.rot = [q.x, q.y, q.z, q.w];
    }
    let pos3 = geometry::positions(&held, &geo);
    let tree = Tree::over(&pos3, |_| true);
    let overlaps = subduction::overlaps(&held, &geo, &pos3, &tree, &env3);
    subduction::resolve(&mut held, &mut world, &geo, &overlaps, &pos3, &env3).unwrap();
    let c = &held.contacts[0];
    assert_eq!((c.last_step, c.carried), (3, false));
    assert!(c.active(3));
}

/// Plate task 8: the volcano stage's felsic cones over an arc (already in the pools as volcanic
/// rock) count inside the arc's own rate, so the arc adds that much less crust and its magma is
/// counted once; the cones it counted are cleared for the hotspots.
#[test]
fn arc_cones_are_counted_inside_the_arc_rate() {
    let p = preset(20_000, 20.0, "supercontinent", 5);
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let (st, geo, pos) = converging_plates(&p, &env);
    let n = st.samples.len();
    let run = |cone_m: f64| {
        let mut s2 = st.clone();
        let mut cones = vec![[cone_m, 0.0]; n];
        let pull = subduction::fronts_and_uplift(&mut s2, &geo, &pos, &env, &mut cones, 0.0);
        (pull, cones)
    };
    let (bare, _) = run(0.0);
    assert!(
        bare.front_continental_km_myr > 0.0,
        "no continental arc front"
    );
    assert!(bare.arc_continental_km3 > 0.0);
    let (with, left) = run(0.5);
    let counted = left.iter().filter(|c| c[0] == 0.0).count();
    assert!(counted > 0 && counted < n, "{counted} of {n} cones counted");
    let cones_km3 = counted as f64 * 0.5 * env.norm.sample_area_km2() / 1000.0;
    let less = bare.arc_continental_km3 - with.arc_continental_km3;
    assert!(
        (less - cones_km3).abs() < 1e-3 * cones_km3,
        "the arc added {less} km³ less for {cones_km3} km³ of cones"
    );
}

/// Plate task 8: a hotspot's basalt is what the volcano stage's cones over it leave of its rate.
#[test]
fn hotspot_cones_are_counted_inside_the_hotspot_rate() {
    let mut p = preset(6000, 20.0, "supercontinent", 3);
    p.tectonics.hotspot_births_per_100myr = 0.0;
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let (mut st, geo) = init::initial_state(&p, &env).unwrap();
    st.hotspots.clear();
    let pos = geometry::positions(&st, &geo);
    let tree = Tree::over(&pos, |_| true);
    let at = pos[0];
    st.hotspots.push(state::Hotspot {
        id: 0,
        pos: at.to_array(),
        born_myr: env.t_myr - 1.0,
        dies_myr: env.t_myr + 100.0,
        uplift_mm_yr: 0.3,
        lip_event: None,
        plates: Vec::new(),
        track_km: 0.0,
        closed: false,
    });
    let n = st.samples.len();
    let run = |cone_m: f64| {
        let mut s2 = st.clone();
        let mut world = WorldState::new(1, 16_384, p.t_start_years());
        let mut cones = vec![[0.0, cone_m]; n];
        hotspots::step(&mut s2, &mut world, &pos, &tree, &env, &mut cones).unwrap();
        let counted = cones.iter().filter(|c| c[1] == 0.0).count();
        (world.globals[hotspots::MAGMA_GLOBAL], counted)
    };
    let (bare, _) = run(0.0);
    assert!(bare > 0.0);
    let (with, counted) = run(1.0);
    assert!(counted > 0 && counted < n, "{counted}");
    let cones_km3 = counted as f64 * env.norm.sample_area_km2() / 1000.0;
    assert!(cones_km3 < bare, "{cones_km3} km³ of cones for {bare} km³");
    assert!(
        ((bare - with) - cones_km3).abs() < 1e-3 * cones_km3,
        "{bare} − {with} vs {cones_km3} km³ of cones"
    );
}

/// Plate task 8: the cordillera thickens by crust shortened from the continent behind it, which
/// thins by as much (volume kept): the step's crust changes by its arc magma alone, and the range
/// stands higher than without the shortening, the crust behind it no higher.
#[test]
fn a_cordillera_takes_its_crust_from_behind_the_range() {
    let mut p = preset(20_000, 20.0, "supercontinent", 5);
    let stage = PlatesStage::new(p.clone());
    let env0 = stage.env(&p.tectonics, 1);
    let (st, geo, pos) = converging_plates(&p, &env0);
    let mut flat = st.clone();
    let mut cones = vec![[0.0; 2]; st.samples.len()];
    subduction::fronts_and_uplift(&mut flat, &geo, &pos, &env0, &mut cones, 0.0);
    p.tectonics.cordillera_shortening_mm_yr = 6.0;
    let env = stage.env(&p.tectonics, 1);
    let total = |st: &PlateState| -> f64 {
        st.samples
            .iter()
            .map(|s| s.pools.iter().map(|&x| f64::from(x)).sum::<f64>())
            .sum()
    };
    let mut s2 = st.clone();
    let pull = subduction::fronts_and_uplift(&mut s2, &geo, &pos, &env, &mut cones, 0.0);
    assert!(pull.cordillera_km3 > 0.0, "no cordillera");
    let area = env.norm.sample_area_km2();
    let added = (total(&s2) - total(&st)) * area / 1000.0;
    let magma = pull.arc_continental_km3 + pull.arc_oceanic_km3;
    assert!(
        (added - magma).abs() < 1e-3 * pull.cordillera_km3.max(magma),
        "crust changed {added} km³ for {magma} km³ of arc magma ({} km³ shortened)",
        pull.cordillera_km3
    );
    // Against the same step without the shortening: thickened samples stand higher, thinned
    // ones, behind the range, no higher (a column standing below its Airy height keeps it).
    let (mut up, mut higher, mut down) = (0, 0, 0);
    for (a, b) in flat.samples.iter().zip(&s2.samples) {
        if a.is_hidden() || a.kind != CONTINENTAL {
            continue;
        }
        if b.thickness_m > a.thickness_m + 1.0 {
            assert!(b.elevation_m >= a.elevation_m, "a thickened sample sank");
            up += 1;
            higher += usize::from(b.elevation_m > a.elevation_m + 1.0);
        } else if b.thickness_m < a.thickness_m - 1.0 {
            assert!(b.elevation_m <= a.elevation_m, "a thinned sample rose");
            down += 1;
        }
    }
    assert!(
        up > 0 && higher > 0 && down > 0,
        "{up} thickened ({higher} higher), {down} thinned"
    );
}

/// Subduction erosion (PLT-441, PLT-442): an oceanic slab coming in faster than the threshold
/// under little sediment takes the rate's crust per km of erosive front from the overriding
/// plate's forearc, every km³ of it booked, no column below an ocean's and none raised; a slower
/// slab, or one under more than the cover threshold, takes none.
#[test]
fn an_erosive_front_takes_its_forearc_to_the_mantle() {
    let mut p = preset(20_000, 20.0, "supercontinent", 5);
    p.tectonics.subduction_erosion_km3_per_km_myr = 90.0;
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let total = |st: &PlateState| -> f64 {
        st.samples
            .iter()
            .map(|s| s.pools.iter().map(|&x| f64::from(x)).sum::<f64>())
            .sum()
    };
    let run = |speed: f64, cover_m: f32| {
        let (mut st, geo, pos) = two_plates(&p, &env, 0.1);
        st.plates[1].omega = (DVec3::X * env.norm.omega_of_speed(speed)).to_array();
        for s in st.samples.iter_mut().filter(|s| s.kind != CONTINENTAL) {
            s.pools[state::pool::SEDIMENTARY] = cover_m;
            s.thickness_m = pools::thickness(&s.pools);
        }
        let mut cones = vec![[0.0; 2]; st.samples.len()];
        let pull = subduction::fronts_and_uplift(&mut st, &geo, &pos, &env, &mut cones, 0.0);
        let before = st.clone();
        let forearcs = pull.forearcs.expect("the rule is on");
        let e = subduction::erode_forearcs(&mut st, &env, &forearcs);
        (before, st, e)
    };
    let (before, after, e) = run(80.0, 200.0);
    assert!(e.front_continental_km_myr > 0.0, "{e:?}");
    let asked = 90.0 * (e.front_continental_km_myr + e.front_oceanic_km_myr);
    let booked = e.continental_km3 + e.other_km3;
    assert!(
        (booked + e.short_km3 - asked).abs() < 1e-3 * asked,
        "{booked} + {} km³ booked for {asked}",
        e.short_km3
    );
    let lost = (total(&before) - total(&after)) * env.norm.sample_area_km2() / 1000.0;
    assert!(
        (lost - booked).abs() < 1e-3 * booked,
        "{lost} lost, {booked} booked"
    );
    let mut thinned = 0;
    for (a, b) in before.samples.iter().zip(&after.samples) {
        if b.thickness_m < a.thickness_m - 1.0 {
            thinned += 1;
            assert!(b.thickness_m >= 6999.0, "a forearc below an ocean's column");
            assert!(b.elevation_m <= a.elevation_m, "an eroded forearc rose");
        }
    }
    assert!(thinned > 0);
    for (speed, cover) in [(50.0, 200.0), (80.0, 1500.0)] {
        let (_, _, e) = run(speed, cover);
        assert_eq!(e, subduction::Eroded::default(), "{speed} mm/yr, {cover} m");
    }
}

/// The trench's retreat (PLT-522, PLT-523; Scholl & von Huene 2010): with `erosion_retreat` on,
/// an erosive margin consumes its front samples whole, at the chance that moves the trench
/// inboard by 2.5 km/Myr. Over many draws of one step the area consumed per km of erosive front
/// and Myr is that retreat; every km³ of the consumed columns is booked, and the volume per km
/// and Myr is the retreat times the column; each consumed sample leaves the overriding plate
/// for the slab's, as the slab's sample across the front with no sediment, where it stands;
/// nothing else changes. A slower slab, or one under more than the cover threshold, takes none.
#[test]
fn an_eroding_margin_retreats_at_scholl_and_von_huenes_rate() {
    let mut p = preset(20_000, 20.0, "supercontinent", 5);
    p.tectonics.subduction_erosion_km3_per_km_myr = 90.0;
    p.tectonics.erosion_retreat = true;
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let area = env.norm.sample_area_km2();
    let setup = |speed: f64, cover_m: f32| {
        let (mut st, geo, pos) = two_plates(&p, &env, 0.1);
        st.plates[1].omega = (DVec3::X * env.norm.omega_of_speed(speed)).to_array();
        for s in st.samples.iter_mut().filter(|s| s.kind != CONTINENTAL) {
            s.pools[state::pool::SEDIMENTARY] = cover_m;
            s.thickness_m = pools::thickness(&s.pools);
        }
        let mut cones = vec![[0.0; 2]; st.samples.len()];
        let pull = subduction::fronts_and_uplift(&mut st, &geo, &pos, &env, &mut cones, 0.0);
        (st, geo, pull.forearcs.expect("the rule is on"))
    };
    let (st, _geo, forearcs) = setup(80.0, 200.0);
    let same = |a: &state::Sample, b: &state::Sample| {
        a.group == b.group
            && a.thickness_m.to_bits() == b.thickness_m.to_bits()
            && a.elevation_m.to_bits() == b.elevation_m.to_bits()
            && a.kind == b.kind
    };
    let across: std::collections::BTreeMap<u32, u32> =
        forearcs.front.iter().map(|&[v, u]| (v, u)).collect();
    let (mut swept_km2, mut front_km_myr, mut booked_km3, mut column_km3) = (0.0, 0.0, 0.0, 0.0);
    for draw in 0..400_u64 {
        let mut s2 = st.clone();
        s2.rng = crate::sphere::SplitMix64::new(draw);
        let e = subduction::erode_forearcs(&mut s2, &env, &forearcs);
        assert!(
            e.front_continental_km_myr > 0.0 && e.short_km3 == 0.0,
            "{e:?}"
        );
        front_km_myr += e.front_continental_km_myr + e.front_oceanic_km_myr;
        swept_km2 += e.retreat_continental_km2 + e.retreat_other_km2;
        booked_km3 += e.continental_km3 + e.other_km3;
        let mut gone = 0.0;
        for (i, (a, b)) in st.samples.iter().zip(&s2.samples).enumerate() {
            if same(a, b) {
                continue;
            }
            let u = *across.get(&(i as u32)).expect("only front samples change");
            assert_eq!(
                s2.plate_of(i),
                st.plate_of(u as usize),
                "{i} left for the slab"
            );
            assert_ne!(s2.plate_of(i), st.plate_of(i));
            let slab = st.samples[u as usize];
            assert_eq!(b.kind, slab.kind);
            assert_eq!(b.elevation_m, slab.elevation_m);
            assert_eq!(b.pools[state::pool::SEDIMENTARY], 0.0);
            assert_eq!(
                s2.groups[b.group as usize].rot, st.groups[a.group as usize].rot,
                "{i} moved"
            );
            gone += f64::from(a.thickness_m) * area / 1000.0;
        }
        assert!(
            ((e.continental_km3 + e.other_km3) - gone).abs() <= 1e-6 * gone.max(1.0),
            "{gone} km³ consumed, {e:?}"
        );
        column_km3 += gone;
    }
    let retreat = swept_km2 / front_km_myr;
    assert!(
        (retreat - subduction::RETREAT_KM_PER_MYR).abs() < 0.15 * subduction::RETREAT_KM_PER_MYR,
        "{retreat} km/Myr over {front_km_myr} km × Myr"
    );
    // The volume per km and Myr is the retreat times the consumed columns: here the continental
    // margin's (with the arc's crust on it).
    assert!((booked_km3 - column_km3).abs() < 1e-6 * column_km3);
    let column_km = column_km3 / swept_km2;
    let normal_km = f64::from(pools::thickness(&pools::continental_column())) / 1000.0;
    assert!(
        column_km >= 0.99 * normal_km && column_km < 1.3 * normal_km,
        "{column_km} km consumed, the normal column {normal_km} km"
    );
    for (speed, cover) in [(50.0, 200.0), (80.0, 1500.0)] {
        let (st, _, forearcs) = setup(speed, cover);
        let mut s2 = st.clone();
        let e = subduction::erode_forearcs(&mut s2, &env, &forearcs);
        assert_eq!(e, subduction::Eroded::default(), "{speed} mm/yr, {cover} m");
        assert_eq!(s2.groups.len(), st.groups.len());
        assert!(st.samples.iter().zip(&s2.samples).all(|(a, b)| same(a, b)));
    }
}

/// Sediment subduction (PLT-440): of the sediment and sedimentary rock on oceanic crust hidden
/// under the overriding plate, a resampling sends its share to the mantle and accretes the rest;
/// the subducted global books what the accretion lost, and a run without the rule has no global.
#[test]
fn a_resampling_subducts_its_share_of_the_slabs_sediment() {
    let mut p = preset(20_000, 20.0, "supercontinent", 5);
    let stage = PlatesStage::new(p.clone());
    let (mut st, geo, pos) = two_plates(&p, &stage.env(&p.tectonics, 1), 0.1);
    for s in st.samples.iter_mut().filter(|s| s.is_hidden()) {
        s.pools[state::pool::SEDIMENT] = 100.0;
        s.pools[state::pool::SEDIMENTARY] = 400.0;
        s.thickness_m = pools::thickness(&s.pools);
    }
    let conserved = |st: &PlateState| -> f64 {
        st.samples
            .iter()
            .map(|s| {
                s.pools[..state::pool::CONSERVED]
                    .iter()
                    .map(|&x| f64::from(x))
                    .sum::<f64>()
            })
            .sum()
    };
    let mut out = Vec::new();
    for share in [0.0, 0.83] {
        p.tectonics.sediment_subduction_share = share;
        let env = stage.env(&p.tectonics, 1);
        let mut s2 = st.clone();
        let mut world = WorldState::new(1, 16_384, p.t_start_years());
        ridges::resample(
            &mut s2,
            &mut world,
            &pos,
            &Tree::over(&pos, |_| true),
            &geo,
            &env,
        )
        .unwrap();
        out.push((
            conserved(&s2) * env.norm.sample_area_km2() / 1000.0,
            world
                .globals
                .get(ridges::ACCRETED_GLOBAL)
                .copied()
                .unwrap_or(0.0),
            world
                .globals
                .get(ridges::SEDIMENT_SUBDUCTED_GLOBAL)
                .copied(),
        ));
    }
    let (kept, sent) = (&out[0], &out[1]);
    assert_eq!(kept.2, None);
    let down = sent.2.expect("the subducted global");
    assert!(down > 0.0);
    assert!(
        ((kept.1 - sent.1) - down).abs() < 1e-3 * down,
        "accreted {} → {} km³ for {down} km³ subducted",
        kept.1,
        sent.1
    );
    assert!(((kept.0 - sent.0) - down).abs() < 1e-3 * down);
}

/// Plate task 8 (the gap budget's recommendation 1): continental crust a resampling accretes
/// raises the surface by its Airy height, never past the column's, and the surface gets the rise
/// as uplift; crust a collision already stacked is consumed and raises far less.
#[test]
fn accreted_crust_stands_up_by_its_airy_height() {
    let p = preset(20_000, 20.0, "supercontinent", 5);
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let (mut st, geo, pos) = two_plates(&p, &env, 0.1);
    for s in st.samples.iter_mut().filter(|s| s.is_hidden()) {
        s.kind = CONTINENTAL;
        s.pools = pools::continental_column();
        s.thickness_m = pools::thickness(&s.pools);
    }
    let k = &p.tectonics;
    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
    let mut rises = Vec::new();
    for stacked in [false, true] {
        let mut s2 = st.clone();
        for s in s2.samples.iter_mut().filter(|s| s.is_hidden()) {
            s.stacked = stacked;
        }
        let mut world = WorldState::new(1, 16_384, p.t_start_years());
        ridges::resample(
            &mut s2,
            &mut world,
            &pos,
            &Tree::over(&pos, |_| true),
            &geo,
            &env,
        )
        .unwrap();
        let rise = world
            .globals
            .get(ridges::ACCRETED_RISE_GLOBAL)
            .copied()
            .unwrap_or(0.0);
        rises.push(rise);
        if stacked {
            continue;
        }
        assert!(rise > 0.0, "accreted crust raised nothing");
        let mut raised = 0;
        for s in s2
            .samples
            .iter()
            .filter(|s| s.is_continental() && !s.is_hidden())
        {
            // The blend's surface stands at most 400 m (the continent's), so anything higher
            // is the accretion's.
            if s.elevation_m > 500.0 {
                raised += 1;
                let iso = collapse::iso_height(&s.pools, z_0, env.gravity);
                assert!(
                    f64::from(s.elevation_m) <= iso.min(k.max_elevation_m) + 1.0,
                    "raised to {} m over its Airy height {iso:.0} m",
                    s.elevation_m
                );
                assert!(s.uplift_mm_yr > 0.0, "a raised sample without uplift");
            }
        }
        assert!(raised > 0);
    }
    // What still rises with the hidden crust consumed is the visible crust the lattice folds
    // onto itself at the margin.
    assert!(rises[1] < 0.5 * rises[0], "{rises:?} km³");
}

/// Plate task 8 (the gap budget's recommendation 3): ocean crust thicker than the ocean's own
/// cools towards the floor its crust holds up on the transition's height line, no higher than
/// the half-way column's, outside a margin too; the ocean's own 7 km cools to the bare floor.
#[test]
fn thick_ocean_crust_cools_towards_the_floor_it_holds_up() {
    let mut p = preset(6000, 20.0, "supercontinent", 3);
    // The pipeline presets' floor (GDH1; the plate-only presets keep Cortial's trench depth).
    p.tectonics.oceanic_floor_elevation_m = -5600.0;
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let (mut st, geo) = init::initial_state(&p, &env).unwrap();
    let k = &p.tectonics;
    let oceanic: Vec<usize> = (0..st.samples.len())
        .filter(|&i| st.samples[i].kind == OCEANIC && !st.samples[i].is_hidden())
        .take(3)
        .collect();
    // The ocean's own crust, an arc at maturity (8 km more) and crust stacked at a resampling.
    let extra = [0.0f32, 8000.0, 60_000.0];
    for (&i, &x) in oceanic.iter().zip(&extra) {
        let s = &mut st.samples[i];
        s.pools = pools::oceanic_column();
        s.pools[pool::FELSIC_PLUTONIC] += x;
        s.thickness_m = pools::thickness(&s.pools);
        s.elevation_m = -3000.0;
        s.flags = 0;
    }
    for _ in 0..800 {
        surface(&mut st, &geo, &env, &[], 0.0);
    }
    let per_m = transition::crust_per_metre(&env);
    let cap = 0.5 * (transition::continental_m() + transition::oceanic_m());
    let z_f = k.oceanic_floor_elevation_m;
    let mut last = f64::NEG_INFINITY;
    for (&i, &x) in oceanic.iter().zip(&extra) {
        let want =
            transition::held_floor_m(transition::oceanic_m() + f64::from(x), cap, z_f, per_m);
        let z = f64::from(st.samples[i].elevation_m);
        assert!(
            (z - want).abs() < 50.0,
            "{x} m more crust: {z:.0} m vs {want:.0} m"
        );
        assert!(z >= last - 1.0);
        last = z;
    }
    assert!((f64::from(st.samples[oceanic[0]].elevation_m) - z_f).abs() < 50.0);
    assert!(last < 0.0 && last > z_f + 2000.0, "{last}");
}

/// The heat flow's final pass (ledger: PLT-469), on by default since 2026-10-23: continental crust
/// that kept the sea floor's heat flow (33.7 mW/m² was baked on 229 Myr floor) is lifted to the
/// proxy's floor (55 + 35 e^(−t/60), at least 55); continental crust already above its law and
/// oceanic crust are left alone; with the switch off nothing changes. The proxy's floor alone: the
/// lithosphere's thermal state (on by default since 2026-10-23, whose own conductive value is the
/// pass's floor with it) is off here.
#[test]
fn the_final_heat_flow_pass_lifts_only_low_continental_samples() {
    let mut p = preset(2_000, 20.0, "supercontinent", 5);
    p.tectonics.thermal_state = false;
    let stage = PlatesStage::new(p.clone());
    let env = stage.env(&p.tectonics, 1);
    let (mut st, _geo) = init::initial_state(&p, &env).unwrap();
    assert!(TectonicParams::default().heat_flow_refresh, "on by default");
    assert!(p.tectonics.heat_flow_refresh, "and in the built-in preset");
    let cont: Vec<usize> = (0..st.samples.len())
        .filter(|&i| st.samples[i].kind == CONTINENTAL)
        .take(3)
        .collect();
    let sea = (0..st.samples.len())
        .find(|&i| st.samples[i].kind == OCEANIC)
        .unwrap();
    assert_eq!(cont.len(), 3);
    // Orogeny long ago (the floor is 55.0016 at 600 Myr), 20 Myr ago (55 + 35 e^(-1/3) = 80.1)
    // and never (55).
    let t = st.t_myr;
    st.samples[cont[0]].orogeny_myr = (t - 600.0) as f32;
    st.samples[cont[1]].orogeny_myr = (t - 20.0) as f32;
    st.samples[cont[2]].orogeny_myr = f32::NEG_INFINITY;
    for &i in &cont {
        st.samples[i].heat_flow_mw_m2 = 33.7;
    }
    st.samples[sea].heat_flow_mw_m2 = 33.7;
    let before = st.clone();
    let mut off = p.tectonics.clone();
    off.heat_flow_refresh = false;
    assert!(!off.thermal_state);
    thermal::refresh_heat_flow(&mut st, &off, t);
    assert_eq!(st, before, "off changes nothing");
    thermal::refresh_heat_flow(&mut st, &p.tectonics, t);
    let q = |i: usize| f64::from(st.samples[i].heat_flow_mw_m2);
    assert!((q(cont[0]) - 55.0016).abs() < 0.001, "{}", q(cont[0]));
    assert!(
        (q(cont[1]) - (55.0 + 35.0 * (-20.0f64 / 60.0).exp())).abs() < 0.001,
        "{}",
        q(cont[1])
    );
    assert!((q(cont[2]) - 55.0).abs() < 0.001, "{}", q(cont[2]));
    assert_eq!(q(sea), f64::from(33.7f32), "the sea floor keeps its own");
    // Crust above its law keeps its value, and only heat flow moves.
    let hot = cont[0];
    st.samples[hot].heat_flow_mw_m2 = 120.0;
    let mut after = st.clone();
    thermal::refresh_heat_flow(&mut after, &p.tectonics, t);
    assert_eq!(after, st, "a second pass changes nothing");
    let mut flat = st.clone();
    for s in &mut flat.samples {
        s.heat_flow_mw_m2 = 0.0;
    }
    let mut was = before.clone();
    for s in &mut was.samples {
        s.heat_flow_mw_m2 = 0.0;
    }
    assert_eq!(flat, was, "no field but heat flow moved");
}

/// The front lane's cut trace (2026-10-24, plates.md "The front's length: why the plates are
/// ragged"): which part of the step cuts the plates into pieces, over the quick preset's
/// defaults. A diagnosis, not a check: it prints each part's samples cut off their plate and
/// joined back, and the share cut off at the end of every 50 Myr; then the same with
/// `plate_fission` (PLT-532) on, and its globals; then with the level set (PLT-536) on as well.
#[test]
#[ignore = "diagnosis: 300 Myr at 40k samples; run with --release -- --ignored --nocapture"]
fn where_the_plates_are_cut() {
    cut_trace::enable(true);
    for (start, seed, fission, level_set) in [
        ("supercontinent", 1, false, false),
        ("scattered_continents", 2, false, false),
        ("supercontinent", 1, true, false),
        ("scattered_continents", 2, true, false),
        ("supercontinent", 1, true, true),
        ("scattered_continents", 2, true, true),
    ] {
        let mut p = preset(40_000, 300.0, start, seed);
        p.tectonics.plate_fission = fission;
        p.tectonics.level_set_membership = level_set;
        let _ = cut_trace::take();
        let mut shares = Vec::new();
        let mut next = 50.0;
        let world = run_stage(&p, |stage, _, st| {
            if st.t_myr - p.t_start_years() / 1.0e6 >= next {
                let geo = &stage.cache.as_ref().unwrap().geo;
                let visible = st.samples.iter().filter(|s| !s.is_hidden()).count();
                let cut = cut_trace::count(st, geo);
                shares.push((next, cut as f64 / visible as f64, st.alive().count()));
                next += 50.0;
            }
        });
        let parts = cut_trace::take();
        eprintln!(
            "{start} seed {seed} fission {fission} level set {level_set}: cut share, plates every \
             50 Myr {shares:.3?}"
        );
        let g = |k: &str| world.globals.get(k).copied().unwrap_or(0.0);
        eprintln!(
            "  fission births {} samples {} captured {}",
            g(ridges::FISSION_GLOBAL),
            g(ridges::FISSION_SAMPLES_GLOBAL),
            g(ridges::CAPTURED_SAMPLES_GLOBAL)
        );
        for part in &parts {
            eprintln!(
                "  {:<15} cut {:>8} joined {:>8} net {:>+8} marks {}",
                part.what,
                part.cut,
                part.joined,
                part.cut as i64 - part.joined as i64,
                part.marks
            );
        }
    }
    cut_trace::enable(false);
}

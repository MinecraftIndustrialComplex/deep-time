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
    mut check: impl FnMut(&PlatesStage, &WorldState, &PlateState),
) -> WorldState {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();
    let mut world = WorldState::new(p.seed, p.scale.circumference_blocks, p.t_start_years());
    let mut stage = PlatesStage::new(p.clone());
    let log = crate::log::LogRing::default();
    let state = std::sync::atomic::AtomicU8::new(0);
    let gauge = NoGauge;
    while !stage.is_done() {
        let ctx = StageContext {
            pool: &pool,
            log: &log,
            until: std::time::Instant::now(),
            gauge: &gauge,
            mem_cap: u64::MAX,
            state: &state,
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
            assert_eq!(f.data.len(), 256 * 128, "{name}");
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

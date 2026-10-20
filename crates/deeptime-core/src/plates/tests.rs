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
    // sweep of 32 runs, 2026-10-16, after the eclogite root and whole-column isostasy: 8 welded).
    for (start, seed) in [("supercontinent", 6), ("scattered_continents", 9)] {
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
    // 2026-10-20: 5 to 9, 11 to 14, 18 to 21 and 24, seed 14 three times, seed 3 no more).
    let p = preset(8000, 300.0, "scattered_continents", 14);
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
    // A seed with welds, so the collision record is tested (as in
    // `collisions_weld_on_stall_and_transfer_on_shortening`).
    let p = preset(8000, 300.0, "scattered_continents", 14);
    let total = PlatesStage::new(p.clone()).total_steps();
    let mut checked = 0u64;
    let mut life = CollisionLives::default();
    let (mut arc_cells_seen, mut mix_seen) = (0u64, 0.0f64);
    let (mut mix_fronts, mut recorded_fronts) = (0.0f64, 0.0f64);
    let (mut balance_steps, mut longest_slab) = (0u64, 0.0f64);
    let mut deaths_before = 0;
    let mut prev_omega: BTreeMap<u32, glam::DVec3> = BTreeMap::new();
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
        // oceanic slabs) are the fronts whose lengths the plates record (a collision front pulls
        // nothing since plate tasks 3 and 4), less their strike-slip stretches, which the mix
        // counts as transform since 2026-10-20 (PLT-311): never more than the plates' fronts, to
        // the rounding of a km a plate, and most of them over the run.
        assert!(step.mix_km.iter().all(|&x| x.is_finite() && x >= 0.0));
        let fronts: f64 = step.mix_km[..2].iter().sum();
        let recorded: f64 = step.plates.iter().map(|q| q.front_km).sum();
        // (A plate that died in the step, welded or absorbed, has no plate record: its fronts are
        // in the mix only.)
        let tol = 0.5 * step.plates.len() as f64 + 1.0;
        if st.counters.plate_deaths == deaths_before {
            assert!(
                fronts <= recorded + tol,
                "step {}: mix fronts {fronts} km, plates' fronts {recorded} km",
                st.step
            );
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
        checked += 1;
    });
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
    assert!(
        mix_fronts > 0.5 * recorded_fronts,
        "the mix's fronts {mix_fronts} km against the plates' {recorded_fronts} km"
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

/// Two plates on the preset's planet: the northern hemisphere continental (plate 0), the southern
/// oceanic (plate 1), the southern turned by `turn` radians about the x axis since the last
/// resampling, so it runs under the continent where y > 0 and opens a gap where y < 0. Overlaps
/// are resolved (the ocean subducts, its crust hidden) as a step would.
fn two_plates(p: &PlanetPreset, env: &Env<'_>, turn: f64) -> (PlateState, Geo, Vec<DVec3>) {
    let (mut st, geo) = init::initial_state(p, env).unwrap();
    let pts = geo.points();
    for (i, s) in st.samples.iter_mut().enumerate() {
        let north = pts[i].z > 0.0;
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
    let q = glam::DQuat::from_rotation_x(turn);
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
    let pos = geometry::positions(&st, &geo);
    st.recount(&pos);
    let tree = Tree::over(&pos, |_| true);
    let overlaps = subduction::detect(&st, &geo, &pos, &tree);
    let mut world = WorldState::new(1, 16_384, p.t_start_years());
    subduction::resolve(&mut st, &mut world, &overlaps, &pos, env).unwrap();
    (st, geo, pos)
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
    let overlaps = subduction::detect(&st, &geo, &pos, &tree);
    let mut world = WorldState::new(1, 16_384, p.t_start_years());
    subduction::resolve(&mut st, &mut world, &overlaps, &pos, &env).unwrap();
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
    for (&i, &(arc, z, hidden, turns)) in oceanic.iter().zip(&cases) {
        assert_eq!(
            st.samples[i].kind == CONTINENTAL,
            turns,
            "arc magma {arc} m (mature at {mature} m), elevation {z} m, hidden {hidden}"
        );
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
        let pull = subduction::fronts_and_uplift(&mut s2, &geo, &pos, &env, &mut cones);
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
    subduction::fronts_and_uplift(&mut flat, &geo, &pos, &env0, &mut cones);
    p.tectonics.cordillera_shortening_mm_yr = 6.0;
    let env = stage.env(&p.tectonics, 1);
    let total = |st: &PlateState| -> f64 {
        st.samples
            .iter()
            .map(|s| s.pools.iter().map(|&x| f64::from(x)).sum::<f64>())
            .sum()
    };
    let mut s2 = st.clone();
    let pull = subduction::fronts_and_uplift(&mut s2, &geo, &pos, &env, &mut cones);
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
        surface(&mut st, &geo, &env, &[]);
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

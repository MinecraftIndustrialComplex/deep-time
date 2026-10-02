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
    for (start, seed) in [("supercontinent", 21), ("scattered_continents", 22)] {
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

/// P1-27: with `cordillera_uplift_mm_yr` the continental crust over an arc stands higher than
/// without it, over the same history.
#[test]
fn arcs_build_a_cordillera() {
    let mean_arc = |uplift: f64| {
        let mut p = preset(8000, 60.0, "scattered_continents", 31);
        p.tectonics.cordillera_uplift_mm_yr = uplift;
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
    let (flat, range) = (mean_arc(0.0), mean_arc(0.5));
    assert!(
        range > flat + 300.0,
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
            assert!(c.speed_count_km >= 0.0);
            let Some(col) = &c.collision else { continue };
            assert!(
                col.onset_step >= c.onset_step,
                "a collision before its contact"
            );
            assert!(col.onset_step <= col.last_step && col.last_step <= st.step);
            assert!((col.onset_myr - col.last_myr) <= 1e-9 && col.last_myr <= st.t_myr + 1e-9);
            assert!(col.shortening_km >= 0.0 && col.area_km2 >= 0.0);
            assert!(
                col.onset_convergence_mm_yr > 0.0,
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
            assert!(b.wedge_width_km.is_none() && b.foredeep_width_km.is_none());
        }
    }
}

/// With `tectonics.collision_on_shortening` on, a transfer or weld fires only on real shortening:
/// every collision record ends a collision state that shortened by the threshold, however fast
/// the plates slid; the states' life cycle holds; and collisions still happen.
#[test]
fn collisions_on_real_shortening_fire_at_the_threshold() {
    let mut p = preset(8000, 300.0, "supercontinent", 21);
    p.tectonics.collision_on_shortening = true;
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
        assert!(
            c.front_shortening_km >= p.tectonics.collision_threshold_km,
            "fired at {} km of shortening",
            c.front_shortening_km
        );
    }
    // Some collision took longer than a step to get there.
    assert!(d.collisions.iter().any(|c| c.end_myr > c.start_myr));
}

/// The dynamics record (`dynamics.rs`) is measurement only: a run with it and a run without it
/// end in the same world (crust, elevation, plate ids, events, fields), and the record agrees with
/// the run's own counters and events.
#[test]
fn recording_does_not_change_the_world_and_matches_the_run() {
    let p = preset(8000, 300.0, "supercontinent", 21);
    let total = PlatesStage::new(p.clone()).total_steps();
    let mut checked = 0u64;
    let mut life = CollisionLives::default();
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
        // were there; the guard: rotation at the cap; the pole: on some plate.
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
            assert_eq!(q.guard, w.length() >= cap * (1.0 - 1e-9), "{q:?}");
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
        checked += 1;
    });
    assert_eq!(checked, total);
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
                Some("sutured" | "absorbed" | "consumed")
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
        // The built-in presets still fire on the count used before task 2 (relative speed ×
        // time): `tectonics.collision_on_shortening` is off.
        assert!(!p.tectonics.collision_on_shortening);
        assert!(
            c.counted_km >= p.tectonics.collision_threshold_km,
            "fired at {} km",
            c.counted_km
        );
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

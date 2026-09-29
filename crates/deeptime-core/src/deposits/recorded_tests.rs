// SPDX-License-Identifier: LGPL-3.0-or-later
//! The deposit stage on the history the plates and surface stages record (P1-14): a small coupled
//! planet (plates → surface → structure → deposits) whose sites come from the real recorded
//! frames, the same run on different thread counts, a checkpoint taken inside the deposit stage,
//! deposits riding rigidly drifting crust, and histories with fields missing.

use std::collections::BTreeMap as Map;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use super::fields::{FieldHistory, HOST_PREFIX, ORIGIN_FIELDS};
use super::recorded::RecordedHistory;
use super::synthetic::{SyntheticHistory, SyntheticParams};
use super::*;
use crate::checkpoint::Checkpoint;
use crate::params::{PlanetPreset, planet_builtin};
use crate::preset::{Preset, PresetBody, with_timeline};
use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};

/// A small coupled planet: `n` L0 samples, L1 r = 64 at 16k (256 × 128), `myr` of history, the
/// deposit stage last, with `count_mult` (the small planet has little area, so tests raise it).
fn pipeline_json(n: u32, myr: f64, seed: u64, count_mult: f64) -> String {
    let json = with_timeline(
        planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(myr),
    )
    .unwrap();
    let mut v: Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = serde_json::json!(n);
    v["resolution"]["l1_r"] = serde_json::json!(64);
    v["keyframes"]["size"] = serde_json::json!([128, 64]);
    v["keyframes"]["every_myr"] = serde_json::json!(20.0);
    v["seed"] = serde_json::json!(seed);
    v["deposits"]["count_mult"] = serde_json::json!(count_mult);
    v.to_string()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-deposits-real-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn options(dir: &PathBuf, threads: usize) -> SessionOptions {
    SessionOptions {
        out_dir: dir.clone(),
        threads,
        mem_cap: 0,
    }
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

/// The whole pipeline on `threads` threads: the world at the end and its census.
fn run(json: &str, threads: usize, name: &str) -> (WorldState, Value) {
    let dir = temp_dir(name);
    let s = Session::create(json, "", options(&dir, threads), Arc::new(NoGauge)).unwrap();
    assert_eq!(s.stage_names(), ["plates", "surface", "structure", "deposits"]);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let census: Value = serde_json::from_str(&world.notes[STAGE_NAME]).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    (world, census)
}

fn sites_by_type(w: &WorldState) -> Map<&str, usize> {
    let mut out = Map::new();
    for s in &w.sites {
        *out.entry(s.deposit_type.as_str()).or_insert(0) += 1;
    }
    out
}

/// A run of the coupled pipeline on 1 thread and on 3: the same sites, bodies, events and census;
/// and the sites hold every family whose fields the history has.
#[test]
fn coupled_pipeline_places_every_family_and_is_deterministic_across_threads() {
    let json = pipeline_json(3000, 100.0, 5, 40.0);
    let (a, census) = run(&json, 1, "one");
    let (b, census_b) = run(&json, 3, "three");
    for t in census["types"].as_array().unwrap() {
        eprintln!(
            "TYPE {} enabled={} tracts={} permissive_km2={:.0} drawn={} sites={}",
            t["id"], t["enabled"], t["tracts"], t["permissive_km2"].as_f64().unwrap(), t["drawn"], t["sites"]
        );
    }
    eprintln!("sites by type: {:?}", sites_by_type(&a));
    // Same result on any thread count.
    assert_eq!(a.sites, b.sites, "sites differ between 1 and 3 threads");
    assert_eq!(a.bodies, b.bodies, "bodies differ");
    assert_eq!(a.events, b.events, "events differ");
    assert_eq!(census, census_b, "census differs");
    let bits = |w: &WorldState| -> Vec<u32> {
        w.fields["deposits/sites"].data.iter().map(|v| v.to_bits()).collect()
    };
    assert_eq!(bits(&a), bits(&b), "sites layer differs");
    // The history is still in the world after the stage (a checkpoint carries it).
    assert!(a.history.as_ref().is_some_and(|h| h.frame_count() == 10));
    // The census names its history.
    assert_eq!(census["history"]["source"], "recorded");
    assert_eq!(census["history"]["frames"], 10);
    // P1-07 publishes the trap structure and the host fractions, so no type is skipped and no
    // fallback is needed; the census names the history the stage read.
    assert_eq!(census["skipped"], serde_json::json!([]));
    assert_eq!(census["fallbacks"], serde_json::json!([]));
    let names = census["history"]["names"].as_array().unwrap();
    for wanted in ["deeptime:trap_structure", "deeptime:host/granodiorite"] {
        assert!(names.iter().any(|n| n == wanted), "{wanted} not in the history");
    }
    // A 100 Myr run on 3,000 samples is young: the petroleum system, which needs source, seal,
    // reservoir and structure in one basin, is the family that develops (the synthetic-history
    // test below has every family; the 500 Myr acceptance run is in STATUS.md). Its sites are
    // real, with kitchens in the log.
    let by = sites_by_type(&a);
    assert!(by.get("deeptime:petroleum_trap").copied().unwrap_or(0) > 0, "{by:?}");
    assert!(
        a.events
            .iter()
            .any(|e| e.kind == "deeptime:petroleum_generation" && e.source == SOURCE)
    );
    // Every site has its forming event with causes in the log (the real events of the plates and
    // the surface, not the synthetic ones).
    for s in &a.sites {
        let e = a
            .events
            .iter()
            .find(|e| e.id == s.forming_event)
            .expect("the forming event is in the log");
        assert!(!e.causes.is_empty(), "site {} ({})", s.id, s.deposit_type);
        for c in &e.causes {
            let cause = a.events.iter().find(|x| x.id == *c).expect("cause in the log");
            assert_ne!(cause.source, synthetic::SOURCE);
        }
    }
    assert!(census["types"].as_array().unwrap().len() >= 7);
}

// ---- a checkpoint inside the deposit stage ----------------------------------------------------

/// A checkpoint taken in the middle of the deposit stage (the history rides in the world) resumes,
/// on another thread count, to the same sites.
#[test]
fn a_checkpoint_inside_the_deposit_stage_resumes_to_the_same_sites() {
    let json = pipeline_json(2000, 40.0, 3, 40.0);
    let (straight, _) = run(&json, 2, "straight");
    let dir = temp_dir("resume");
    let s = Session::create(&json, "", options(&dir, 2), Arc::new(NoGauge)).unwrap();
    // Into the deposit stage (index 3), a few frames in.
    while s.progress().stage_index < 3 {
        s.advance(Duration::from_millis(1)).unwrap();
    }
    for _ in 0..3 {
        s.advance(Duration::ZERO).unwrap();
    }
    let ck = s.checkpoint().unwrap();
    assert_eq!(ck.next_stage_name(), Some("deposits"));
    assert!(ck.world.history.is_some(), "the checkpoint carries the history");
    let bytes = ck.to_bytes().unwrap();
    drop(s);
    let ck = Checkpoint::from_bytes(std::path::Path::new("ck"), &bytes).unwrap();
    let s = Session::resume(ck, options(&dir, 3), Arc::new(NoGauge)).unwrap();
    finish(&s);
    let resumed = s.checkpoint().unwrap().world;
    assert_eq!(resumed.sites, straight.sites);
    assert_eq!(resumed.bodies, straight.bodies);
    assert_eq!(resumed.events, straight.events);
    let _ = std::fs::remove_dir_all(&dir);
}

// ---- the engine on recorded frames made from the synthetic history -----------------------------

/// The synthetic history of `deposits_synthetic_4k.json`: 256 × 128 cells, 500 Myr in 25 Myr
/// frames (twenty), every family's geology in it.
fn synthetic_params() -> SyntheticParams {
    SyntheticParams {
        seed: 11,
        circumference: 4096,
        grid_r: 16,
        t_start_years: -5.0e8,
        t_end_years: 0.0,
        snapshot_every_years: 2.5e7,
    }
}

/// The pipeline preset on the synthetic history's planet (its circumference and grid, so its
/// scales and seed), the deposit stage's parameters from it. Its timeline is left at the
/// surface's 10 Myr frames: the stage's frame count (50 expected, 20 recorded) is corrected from
/// the history in set-up.
fn preset_for(sp: &SyntheticParams, count_mult: f64) -> PlanetPreset {
    let json = pipeline_json(2000, -sp.t_start_years / 1.0e6, sp.seed, count_mult);
    let mut v: Value = serde_json::from_str(&json).unwrap();
    v["scale"]["circumference_blocks"] = serde_json::json!(sp.circumference);
    v["resolution"]["l1_r"] = serde_json::json!(sp.grid_r);
    match Preset::from_json(&v.to_string()).unwrap().body {
        PresetBody::Planet(p) => *p,
        other => panic!("not a planet preset: {other:?}"),
    }
}

/// Recorded frames of the synthetic history whose crust drifts `shift` cells east per frame (the
/// field values of frame `k` are the fixed-crust ones moved by `shift · k`; frames carry the
/// origin offsets that say so), keeping the fields `keep` allows.
fn drifting(sp: &SyntheticParams, shift: i64, keep: &dyn Fn(&str) -> bool) -> RecordedHistory {
    let synth = SyntheticHistory::new(sp.clone()).unwrap();
    let grid = synth.grid().clone();
    let (w, n) = (grid.w as usize, grid.cells());
    let mut h = RecordedHistory::new(grid);
    for k in 0..synth.frame_count() {
        let mut frame = synth.frame(k).unwrap();
        frame.fields.retain(|name, _| keep(name));
        let by = (shift * k as i64).rem_euclid(w as i64) as usize;
        if by != 0 {
            for v in frame.fields.values_mut() {
                let old = v.clone();
                for (c, x) in v.iter_mut().enumerate() {
                    let (i, j) = (c % w, c / w);
                    *x = old[j * w + (i + w - by) % w];
                }
            }
        }
        frame
            .fields
            .insert(ORIGIN_FIELDS[0].into(), vec![(-shift) as f32; n]);
        frame.fields.insert(ORIGIN_FIELDS[1].into(), vec![0.0; n]);
        h.push(&frame);
    }
    h
}

/// Steps a deposit stage over `world` with two threads; the log lines come back.
fn drive(stage: &mut DepositStage, world: &mut WorldState) -> Vec<String> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();
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
        stage.step(world, &ctx).unwrap();
    }
    log.drain()
}

/// The world a deposit stage starts from: the synthetic history's tectonic events in the log (what
/// the plates stage emits for real) and `history` recorded.
fn world_with(sp: &SyntheticParams, history: RecordedHistory) -> WorldState {
    let mut world = WorldState::new(sp.seed, sp.circumference, 0.0);
    let synth = SyntheticHistory::new(sp.clone()).unwrap();
    for e in synth.events() {
        let ev = world
            .event(e.t_start, e.kind, synthetic::SOURCE)
            .until(e.t_end)
            .at(Place::Point {
                lat: e.lat,
                lon: e.lon,
            })
            .magnitude(e.magnitude)
            .param("feature", e.feature);
        world.emit(ev).unwrap();
    }
    world.history = Some(history);
    world
}

fn run_recorded(sp: &SyntheticParams, history: RecordedHistory, count_mult: f64) -> (WorldState, Vec<String>) {
    let p = preset_for(sp, count_mult);
    let mut world = world_with(sp, history);
    let mut stage = DepositStage::recorded(&p).unwrap();
    let log = drive(&mut stage, &mut world);
    (world, log)
}

/// The recorded path reads the same frames as the synthetic one and places sites of every family
/// the history has fields for, with the log naming nothing skipped.
#[test]
fn the_recorded_stage_places_every_family() {
    let sp = synthetic_params();
    let (w, log) = run_recorded(&sp, drifting(&sp, 0, &|_| true), 25.0);
    let by = sites_by_type(&w);
    eprintln!("sites by type: {by:?}");
    for family in [
        "deeptime:porphyry_cu_mo_au",
        "deeptime:coal_seam",
        "deeptime:petroleum_trap",
    ] {
        assert!(by.get(family).copied().unwrap_or(0) > 0, "{family}: {by:?}");
    }
    assert!(
        by.get("deeptime:mvt_zn_pb").copied().unwrap_or(0)
            + by.get("deeptime:sedex_zn_pb").copied().unwrap_or(0)
            > 0,
        "{by:?}"
    );
    let census: Value = serde_json::from_str(&w.notes[STAGE_NAME]).unwrap();
    assert_eq!(census["skipped"], serde_json::json!([]));
    assert!(!log.iter().any(|l| l.contains("skipped")), "{log:?}");
    // The forming events cite the events in the log.
    for s in &w.sites {
        let e = w.events.iter().find(|e| e.id == s.forming_event).unwrap();
        assert!(!e.causes.is_empty(), "site {} ({})", s.id, s.deposit_type);
    }
}

/// Deposits ride the crust: when the whole crust drifts east a few cells per frame (each frame's
/// fields moved with it, the frames saying where each cell's crust came from), every favourability
/// map at the end is the fixed-crust map moved by the total drift, cell for cell.
#[test]
fn deposits_ride_a_rigidly_drifting_crust() {
    let sp = synthetic_params();
    let shift = 3i64;
    let (fixed, _) = run_recorded(&sp, drifting(&sp, 0, &|_| true), 1.0);
    let (moved, _) = run_recorded(&sp, drifting(&sp, shift, &|_| true), 1.0);
    let frames = 20i64;
    let total = shift * (frames - 1);
    let mut compared = 0;
    for (name, a) in &fixed.fields {
        if !name.starts_with("deposits/favourability/") {
            continue;
        }
        let b = &moved.fields[name];
        let w = a.grid.w as usize;
        let mut wrong = 0;
        for c in 0..a.data.len() {
            let (i, j) = (c % w, c / w);
            let from = j * w + (i as i64 - total).rem_euclid(w as i64) as usize;
            if a.data[from].to_bits() != b.data[c].to_bits() {
                wrong += 1;
            }
        }
        assert_eq!(wrong, 0, "{name}: {wrong} of {} cells differ", a.data.len());
        if a.data.iter().any(|v| *v > 0.0) {
            compared += 1;
        }
    }
    assert!(compared >= 4, "{compared} non-empty layers compared");
}

/// A type whose fields the history lacks is skipped, by name, in the log and the census, and
/// nothing else changes: the other types' favourability is exactly what a full history gives.
#[test]
fn missing_fields_skip_the_types_that_need_them_and_nothing_else() {
    let sp = synthetic_params();
    let (full, _) = run_recorded(&sp, drifting(&sp, 0, &|_| true), 25.0);
    let lacking = |name: &str| {
        name != "deeptime:trap_structure" && name != "deeptime:carbonate_platform"
    };
    let (part, log) = run_recorded(&sp, drifting(&sp, 0, &lacking), 25.0);
    let census: Value = serde_json::from_str(&part.notes[STAGE_NAME]).unwrap();
    let skipped: Map<String, Vec<String>> = census["skipped"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            (
                s["type"].as_str().unwrap().to_owned(),
                s["missing"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|m| m.as_str().unwrap().to_owned())
                    .collect(),
            )
        })
        .collect();
    let types = types::builtin().unwrap();
    for t in &types {
        let reads_trap = t.fields().iter().any(|f| f == "deeptime:trap_structure");
        let reads_carbonate = t
            .fields()
            .iter()
            .any(|f| f == "deeptime:carbonate_platform");
        assert_eq!(
            skipped.contains_key(&t.id),
            reads_trap || reads_carbonate,
            "{}: {skipped:?}",
            t.id
        );
    }
    assert!(skipped.contains_key("deeptime:petroleum_trap"));
    assert_eq!(
        skipped["deeptime:petroleum_trap"],
        ["deeptime:trap_structure"]
    );
    for id in skipped.keys() {
        assert!(
            log.iter().any(|l| l.contains("skipped") && l.contains(id.as_str())),
            "{id} is not in the log: {log:?}"
        );
        assert!(
            !part.sites.iter().any(|s| &s.deposit_type == id),
            "{id} placed sites without its fields"
        );
    }
    // The types that kept their fields are untouched.
    for t in types.iter().filter(|t| !skipped.contains_key(&t.id) && !t.child_only) {
        let name = format!("deposits/favourability/{}", t.short());
        let (a, b) = (&full.fields[&name], &part.fields[&name]);
        let same = a
            .data
            .iter()
            .zip(&b.data)
            .all(|(x, y)| x.to_bits() == y.to_bits());
        assert!(same, "{} changed when unrelated fields went missing", t.id);
    }
    assert_eq!(census["fallbacks"], serde_json::json!([]), "the hosts are there");
    // Without any host-rock field the fallback is documented, not a skip: every type still runs,
    // the log and the census say the lithology opinion is gone.
    let no_hosts = |name: &str| !name.starts_with(HOST_PREFIX);
    let (bare, log) = run_recorded(&sp, drifting(&sp, 0, &no_hosts), 25.0);
    let census: Value = serde_json::from_str(&bare.notes[STAGE_NAME]).unwrap();
    assert_eq!(census["skipped"], serde_json::json!([]));
    let fallbacks = census["fallbacks"].as_array().unwrap();
    assert!(
        fallbacks
            .iter()
            .any(|f| f.as_str().unwrap().contains(HOST_PREFIX)),
        "{fallbacks:?}"
    );
    assert!(log.iter().any(|l| l.contains("no lithology opinion")), "{log:?}");
    assert!(!bare.sites.is_empty());
}

/// A world without a recorded history says what to run first.
#[test]
fn a_world_without_history_is_a_clear_error() {
    let sp = synthetic_params();
    let p = preset_for(&sp, 1.0);
    let mut world = WorldState::new(1, sp.circumference, 0.0);
    let mut stage = DepositStage::recorded(&p).unwrap();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    let log = crate::log::LogRing::default();
    let state = std::sync::atomic::AtomicU8::new(0);
    let ctx = StageContext {
        pool: &pool,
        log: &log,
        until: std::time::Instant::now(),
        gauge: &NoGauge,
        mem_cap: u64::MAX,
        state: &state,
    };
    let e = stage.step(&mut world, &ctx).unwrap_err().to_string();
    assert!(e.contains("field history") && e.contains("surface"), "{e}");
}

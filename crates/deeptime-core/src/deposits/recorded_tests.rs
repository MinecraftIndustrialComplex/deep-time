// SPDX-License-Identifier: LGPL-3.0-or-later
//! The deposit stage on the history the plates and surface stages record (P1-14): a small coupled
//! planet (plates → surface → structure → deposits) whose sites come from the real recorded
//! frames, the same run on different thread counts, a checkpoint taken inside the deposit stage,
//! deposits riding rigidly drifting crust, and histories with fields missing.

use std::collections::BTreeMap as Map;
use std::path::{Path, PathBuf};
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
pub(super) fn pipeline_json(n: u32, myr: f64, seed: u64, count_mult: f64) -> String {
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
    // The young landscape and L3 amplification (P1-06) have their own tests.
    v.as_object_mut().unwrap().remove("young");
    v.as_object_mut().unwrap().remove("amplify");
    v.to_string()
}

pub(super) fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-deposits-real-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn options(dir: &Path, threads: usize) -> SessionOptions {
    SessionOptions {
        out_dir: dir.to_path_buf(),
        threads,
        mem_cap: 0,
    }
}

pub(super) fn finish(s: &Session) {
    loop {
        match s.advance(Duration::from_secs(60)).unwrap() {
            AdvanceStatus::Done => break,
            AdvanceStatus::Cancelled => panic!("cancelled"),
            _ => {}
        }
    }
}

/// The deposit stage's census in `w`'s notes, without its wall-clock laps (`millis`, a run's
/// timing, not the world's).
pub(super) fn census_of(w: &WorldState) -> Value {
    let mut census: Value = serde_json::from_str(&w.notes[STAGE_NAME]).unwrap();
    if let Some(o) = census.as_object_mut() {
        o.remove("millis");
    }
    census
}

/// The whole pipeline on `threads` threads: the world at the end and its census.
fn run(json: &str, threads: usize, name: &str) -> (WorldState, Value) {
    let dir = temp_dir(name);
    let s = Session::create(json, "", options(&dir, threads), Arc::new(NoGauge)).unwrap();
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
    let census = census_of(&world);
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

/// A run of the coupled pipeline on 1 thread and on 3: the same sites, bodies, events and census,
/// from the history the plates, surface and structure stages recorded.
#[test]
fn coupled_pipeline_reads_its_own_history_and_is_deterministic_across_threads() {
    let json = pipeline_json(3000, 100.0, 5, 40.0);
    let (a, census) = run(&json, 1, "one");
    let (b, census_b) = run(&json, 3, "three");
    for t in census["types"].as_array().unwrap() {
        eprintln!(
            "TYPE {} enabled={} tracts={} permissive_km2={:.0} drawn={} sites={}",
            t["id"],
            t["enabled"],
            t["tracts"],
            t["permissive_km2"].as_f64().unwrap(),
            t["drawn"],
            t["sites"]
        );
    }
    eprintln!("sites by type: {:?}", sites_by_type(&a));
    // Same result on any thread count.
    assert_eq!(a.sites, b.sites, "sites differ between 1 and 3 threads");
    assert_eq!(a.bodies, b.bodies, "bodies differ");
    assert_eq!(a.events, b.events, "events differ");
    assert_eq!(census, census_b, "census differs");
    let bits = |w: &WorldState| -> Vec<u32> {
        w.fields["deposits/sites"]
            .data
            .iter()
            .map(|v| v.to_bits())
            .collect()
    };
    assert_eq!(bits(&a), bits(&b), "sites layer differs");
    // Everything else the stages leave is the same bits on any thread count too (P1-17 made
    // the per-cell loops of the surface, climate, flexure, glacial, slope failure, volcano,
    // structure and deposit steps parallel and must not change a bit): every field by name,
    // then the whole world (plates, surface, catalogues, structure). The notes carry wall
    // times and are left out; the census above is compared on its own.
    for (name, f) in &a.fields {
        let g = &b.fields[name];
        assert!(
            f.data.len() == g.data.len()
                && f.data
                    .iter()
                    .zip(&g.data)
                    .all(|(x, y)| x.to_bits() == y.to_bits()),
            "field {name} differs between 1 and 3 threads"
        );
    }
    assert_eq!(
        a.fields.keys().collect::<Vec<_>>(),
        b.fields.keys().collect::<Vec<_>>()
    );
    let body = |w: &WorldState| {
        let mut w = w.clone();
        w.notes.clear();
        w.history = None;
        postcard::to_allocvec(&w).unwrap()
    };
    assert!(
        body(&a) == body(&b),
        "the world differs between 1 and 3 threads"
    );
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
        assert!(
            names.iter().any(|n| n == wanted),
            "{wanted} not in the history"
        );
    }
    // A 100 Myr run on 3,000 samples is young: only the families whose setting forms fast develop.
    // Which ones depends on the climate (with P1-04's stand-in it was the petroleum system; with
    // P1-05's real climate it is coal on the humid coastal plains). The synthetic-history test below
    // has every family; the 500 Myr acceptance run is in STATUS.md. Here: some sites, all real.
    let by = sites_by_type(&a);
    assert!(by.values().sum::<usize>() > 0, "{by:?}");
    if by.contains_key("deeptime:petroleum_trap") {
        assert!(
            a.events
                .iter()
                .any(|e| e.kind == "deeptime:petroleum_generation" && e.source == SOURCE)
        );
    }
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
            let cause = a
                .events
                .iter()
                .find(|x| x.id == *c)
                .expect("cause in the log");
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
    assert!(
        ck.world.history.is_some(),
        "the checkpoint carries the history"
    );
    let bytes = ck.to_bytes().unwrap();
    drop(s);
    let ck = Checkpoint::from_bytes(Path::new("ck"), &bytes).unwrap();
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
            out_dir: std::path::Path::new("."),
            until: std::time::Instant::now(),
            gauge: &gauge,
            mem_cap: u64::MAX,
            state: &state,
            history: None,
            needed_peak: None,
            compute: crate::compute::ComputeMode::Cpu,
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

fn run_recorded(
    sp: &SyntheticParams,
    history: RecordedHistory,
    count_mult: f64,
) -> (WorldState, Vec<String>) {
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
    let lacking =
        |name: &str| name != "deeptime:trap_structure" && name != "deeptime:carbonate_platform";
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
            log.iter()
                .any(|l| l.contains("skipped") && l.contains(id.as_str())),
            "{id} is not in the log: {log:?}"
        );
        assert!(
            !part.sites.iter().any(|s| &s.deposit_type == id),
            "{id} placed sites without its fields"
        );
    }
    // The types that kept their fields are untouched.
    for t in types
        .iter()
        .filter(|t| !skipped.contains_key(&t.id) && !t.child_only)
    {
        let name = format!("deposits/favourability/{}", t.short());
        let (a, b) = (&full.fields[&name], &part.fields[&name]);
        let same = a
            .data
            .iter()
            .zip(&b.data)
            .all(|(x, y)| x.to_bits() == y.to_bits());
        assert!(same, "{} changed when unrelated fields went missing", t.id);
    }
    assert_eq!(
        census["fallbacks"],
        serde_json::json!([]),
        "the hosts are there"
    );
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
    assert!(
        log.iter().any(|l| l.contains("no lithology opinion")),
        "{log:?}"
    );
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
        out_dir: std::path::Path::new("."),
        until: std::time::Instant::now(),
        gauge: &NoGauge,
        mem_cap: u64::MAX,
        state: &state,
        history: None,
        needed_peak: None,
        compute: crate::compute::ComputeMode::Cpu,
    };
    let e = stage.step(&mut world, &ctx).unwrap_err().to_string();
    assert!(e.contains("field history") && e.contains("surface"), "{e}");
}

// ---- the survivor formula and the regional MVT driver (2026-09-29) -----------------------------

/// A recorded history on a planet of `circumference` blocks with L1 cells of `r` blocks: `frames`
/// frames of 10 Myr ending now, each named field's value at (frame, name, column, row) from
/// `value`.
fn history_of(
    circumference: u32,
    r: u32,
    frames: usize,
    names: &[&str],
    clocks: &[(&str, f32)],
    value: &dyn Fn(usize, &str, usize, usize) -> f32,
) -> RecordedHistory {
    let grid = deeptime_format::GridDesc::for_planet("L1", circumference, r).unwrap();
    let (w, n) = (grid.w as usize, grid.cells());
    let mut h = RecordedHistory::new(grid);
    for k in 0..frames {
        let mut frame = fields::FieldFrame {
            t_years: -1.0e7 * (frames - 1 - k) as f64,
            dt_years: 1.0e7,
            ..Default::default()
        };
        for name in names {
            frame.fields.insert(
                (*name).to_owned(),
                (0..n).map(|c| value(k, name, c % w, c / w)).collect(),
            );
        }
        for (name, v) in clocks {
            frame.clocks.insert((*name).to_owned(), *v);
        }
        h.push(&frame);
    }
    h
}

/// The deposit stage of a pipeline preset on `circumference`/`r`, seeded `seed`, with `count_mult`
/// and provinces off, over `history` (no events in the log), running `types` when given.
fn run_on(
    circumference: u32,
    r: u32,
    seed: u64,
    count_mult: f64,
    history: RecordedHistory,
    types: Option<Vec<types::DepositType>>,
) -> (WorldState, Value) {
    let sp = SyntheticParams {
        seed,
        circumference,
        grid_r: r,
        t_start_years: -1.0e8,
        t_end_years: 0.0,
        snapshot_every_years: 1.0e7,
    };
    let mut p = preset_for(&sp, count_mult);
    p.deposits.as_mut().unwrap().province = Province::Off;
    let mut world = WorldState::new(seed, circumference, 0.0);
    world.history = Some(history);
    let mut stage = DepositStage::recorded(&p).unwrap();
    if let Some(t) = types {
        stage = stage.with_types(t);
    }
    drive(&mut stage, &mut world);
    let census: Value = serde_json::from_str(&world.notes[STAGE_NAME]).unwrap();
    (world, census)
}

/// L1 (column, row) of a site's centre (the engine's own mapping; a centre jittered to the edge of
/// its cell and rounded to whole blocks may land in the next one).
fn site_cell(s: &deeptime_format::Site, circumference: u32, r: u32) -> (usize, usize) {
    let grid = deeptime_format::GridDesc::for_planet("L1", circumference, r).unwrap();
    let cell = cluster::cell_of(&grid, f64::from(s.center[0]), f64::from(s.center[2]));
    ((cell % grid.w) as usize, (cell / grid.w) as usize)
}

/// The survivor formula (plan §8.6 as built 2026-09-29) on a history whose preservation is known:
/// a block of crust fully favourable for 100 Myr that erodes 40 m/Myr everywhere. Deposits form
/// uniformly in time, so a deposit of half-height h formed at depth z survives when its age is
/// below (z + h)/(40 m/Myr) and counts toward the density (top within the 1 km assessment depth)
/// when its age is also above (z − h − 1 km)/(40 m/Myr): P̄₁ = E_T[(age window)/100 Myr]. The
/// engine's preservation is that closed form; over 300 planets the mean number of counted survivors
/// is the density's target ρ·A within sampling error, all sites (the deeper ones too) average
/// target · P̄/P̄₁, and about target / P̄₁ form.
#[test]
fn the_survivor_formula_hits_its_target_in_expectation() {
    let (circumference, r) = (2048u32, 16u32);
    let block = |i: usize, j: usize| (40..60).contains(&i) && (28..36).contains(&j);
    let near_block = |i: usize, j: usize| (39..61).contains(&i) && (27..37).contains(&j);
    let rate = 40.0f32;
    let history = || {
        history_of(
            circumference,
            r,
            10,
            &[
                "test:fertile",
                fields::ELEVATION,
                fields::EROSION,
                fields::DEPOSITION,
            ],
            &[],
            &|_, name, i, j| match name {
                "test:fertile" if block(i, j) => 1.0,
                fields::ELEVATION => 100.0,
                fields::EROSION => rate,
                _ => 0.0,
            },
        )
    };
    // The block's true area, and a density that asks for three survivors.
    let grid = deeptime_format::GridDesc::for_planet("L1", circumference, r).unwrap();
    let area: f64 = (0..grid.h)
        .map(|j| {
            (0..grid.w)
                .filter(|&i| block(i as usize, j as usize))
                .count() as f64
                * fields::cell_area_km2(&grid, j, 300.0)
        })
        .sum();
    let rho = 3.0 * 1.0e5 / area;
    let mut t = types::builtin().unwrap().remove(0); // porphyry's sizes and formation depth
    t.id = "test:uniform".into();
    t.components = types::Components {
        source: vec![types::Term {
            field: Some("test:fertile".into()),
            curve: vec![[0.0, 0.0], [1.0, 1.0]],
            weight: 1.0,
            any_of: vec![],
            note: None,
        }],
        ..types::Components::default()
    };
    t.clock_gates.clear();
    t.hosts = types::Hosts::default();
    t.geometry.children.clear();
    t.preservation.erosion_behavior = types::ErosionBehavior::Truncate;
    t.sizing.density_per_1e5km2 = types::Percentiles {
        p90: rho / 4.0,
        p50: rho,
        p10: rho * 2.0,
    };
    // A single estimate: the target is ρ itself (a measured distribution would take its mean).
    t.sizing.density_basis = types::DensityBasis::Estimate;
    t.validate().unwrap();
    // The closed form, on a fine tonnage quadrature.
    let tonnage = sizing::SizeModel::new(&t.sizing).tonnage;
    let nodes = 20_000;
    let span_m = f64::from(rate) * 100.0;
    let (z, a) = (
        t.preservation.formation_depth_m,
        t.preservation.assessment_depth_m,
    );
    let (mut want_p, mut want_p1) = (0.0, 0.0);
    for m in 0..nodes {
        let q = (f64::from(m) + 0.5) / f64::from(nodes);
        let h = geometry::half_height_m(&t, tonnage.quantile(q) * 1.0e6);
        let old = ((z + h) / span_m).min(1.0);
        let young = ((z - h - a) / span_m).max(0.0);
        want_p += old;
        want_p1 += (old - young).max(0.0);
    }
    want_p /= f64::from(nodes);
    want_p1 /= f64::from(nodes);
    assert!(
        want_p1 > 0.2 && want_p1 < want_p - 0.02 && want_p < 0.8,
        "a test with something to say: {want_p1} of {want_p}"
    );
    let runs = 300u64;
    let (mut survivors, mut counted, mut formed) = (0usize, 0u64, 0u64);
    for seed in 0..runs {
        let (w, census) = run_on(
            circumference,
            r,
            seed,
            1.0,
            history(),
            Some(vec![t.clone()]),
        );
        let c = &census["types"][0];
        assert_eq!(c["id"], "test:uniform");
        let target = c["target"].as_f64().unwrap();
        assert!((target - 3.0).abs() < 1e-6, "target {target}");
        let p = c["preservation"].as_f64().unwrap();
        assert!((p - want_p1).abs() < 0.01, "preservation {p} vs {want_p1}");
        let sites = c["expected_sites"].as_f64().unwrap();
        assert!(
            (sites - 3.0 * want_p / want_p1).abs() < 0.1,
            "expected sites {sites}"
        );
        assert!((c["expected"].as_f64().unwrap() - 3.0).abs() < 1e-6);
        assert_eq!(c["capped"], false);
        survivors += w.sites.len();
        counted += c["counted"].as_u64().unwrap();
        formed += c["drawn"].as_u64().unwrap();
        for s in &w.sites {
            let (i, j) = site_cell(s, circumference, r);
            assert!(
                near_block(i, j),
                "a site outside the fertile block at {i},{j}"
            );
        }
    }
    let n = runs as f64;
    let mean = counted as f64 / n;
    let se = (3.0 / n).sqrt();
    assert!(
        (mean - 3.0).abs() < 4.0 * se + 0.03,
        "mean counted survivors {mean} vs target 3 (±{se:.3})"
    );
    let want_sites = 3.0 * want_p / want_p1;
    let mean_sites = survivors as f64 / n;
    assert!(
        (mean_sites - want_sites).abs() < 4.0 * (want_sites / n).sqrt() + 0.05,
        "mean sites {mean_sites} vs {want_sites}"
    );
    let mean_formed = formed as f64 / n;
    let want_formed = 3.0 / want_p1;
    assert!(
        (mean_formed - want_formed).abs() < 4.0 * (want_formed / n).sqrt() + 0.05,
        "mean formed {mean_formed} vs {want_formed}"
    );
    eprintln!(
        "survivor formula: P̄₁ {want_p1:.4}, P̄ {want_p:.4}, mean counted {mean:.3} (target 3), \
         mean sites {mean_sites:.2} ({want_sites:.2}), mean formed {mean_formed:.2} \
         ({want_formed:.2} expected) over {runs} planets"
    );
}

/// The regional MVT driver: a carbonate platform beside a hot foreland basin in front of a rising
/// orogen gets MVT ore, though neither the basin nor the orogen reaches the platform; the same
/// platform far from both, or with the orogen no longer rising, or beside a cold basin, gets none.
#[test]
fn mvt_forms_where_a_basin_lies_beside_carbonate() {
    // 256 × 128 cells of 9.6 km: an orogen in columns 10–20, a foreland basin in 21–40, platform A
    // in 41–55 (beside the basin), platform B in 150–165 (about 1,000 km from either).
    let (circumference, r) = (8192u32, 32u32);
    let orogen = |i: usize| (10..=20).contains(&i);
    let basin = |i: usize| (21..=40).contains(&i);
    let platform = |i: usize| (41..=55).contains(&i) || (150..=165).contains(&i);
    let names = [
        fields::ELEVATION,
        fields::EROSION,
        fields::DEPOSITION,
        fields::BURIAL_TEMP,
        "deeptime:orogenic_uplift_m_myr",
        "deeptime:crust_thickness_km",
        "deeptime:fault_density",
        "deeptime:carbonate_platform",
        "deeptime:host/dolostone",
    ];
    let clocks = [("deeptime:clock/oxygenated_oceans", 1.0f32)];
    let run = |uplift: f32, basin_t: f32| {
        let h = history_of(
            circumference,
            r,
            5,
            &names,
            &clocks,
            &|_, name, i, _| match name {
                fields::ELEVATION if orogen(i) => 2500.0,
                fields::ELEVATION => 150.0,
                fields::DEPOSITION if basin(i) => 60.0,
                fields::BURIAL_TEMP if basin(i) => basin_t,
                fields::BURIAL_TEMP => 20.0,
                "deeptime:orogenic_uplift_m_myr" if orogen(i) => uplift,
                "deeptime:crust_thickness_km" => 38.0,
                "deeptime:fault_density" => 0.3,
                "deeptime:carbonate_platform" if platform(i) => 0.4,
                "deeptime:host/dolostone" if platform(i) => 0.1,
                _ => 0.0,
            },
        );
        let mvt: Vec<types::DepositType> = types::builtin()
            .unwrap()
            .into_iter()
            .filter(|t| t.id == "deeptime:mvt_zn_pb")
            .collect();
        run_on(circumference, r, 7, 5.0, h, Some(mvt))
    };
    let (w, census) = run(300.0, 110.0);
    let fav = &w.fields["deposits/favourability/mvt_zn_pb"];
    let mean_in = |cols: std::ops::RangeInclusive<usize>| {
        let v: Vec<f32> = (0..fav.data.len())
            .filter(|c| cols.contains(&(c % 256)))
            .map(|c| fav.data[c])
            .collect();
        v.iter().sum::<f32>() / v.len() as f32
    };
    let (a, b) = (mean_in(41..=55), mean_in(150..=165));
    assert!(a > 0.5, "platform A favourability {a}");
    assert_eq!(b, 0.0, "platform B favourability");
    assert_eq!(mean_in(21..=40), 0.0, "the basin itself has no carbonate");
    assert!(!w.sites.is_empty(), "{}", census["types"][0]);
    for s in &w.sites {
        let (i, _) = site_cell(s, circumference, r);
        assert!((40..=56).contains(&i), "MVT site in column {i}");
    }
    // No rising orogen: no drive. A cold basin: no brine.
    for (uplift, t) in [(0.0, 110.0), (300.0, 20.0)] {
        let (w, _) = run(uplift, t);
        let fav = &w.fields["deposits/favourability/mvt_zn_pb"];
        assert!(
            fav.data.iter().all(|v| *v == 0.0),
            "uplift {uplift}, basin {t} °C"
        );
        assert!(w.sites.is_empty());
    }
}

/// Coal seam fields (2026-09-29): a deposit is a stack of seams, one `ore` body per seam with
/// consecutive ids, the same footprint and yaw, running top to bottom without touching, and the
/// fields of the whole planet adjoin without their plan footprints overlapping.
#[test]
fn coal_fields_adjoin_without_overlap_and_hold_stacked_seams() {
    // The synthetic planet is 1,200 km around, so fields of a thousand km² are made small (ore
    // amount 0.003×: footprints about a fifth as wide) to fit a dozen of them.
    let sp = synthetic_params();
    let mut p = preset_for(&sp, 400.0);
    p.deposits.as_mut().unwrap().size_mult = 0.003;
    let mut w = world_with(&sp, drifting(&sp, 0, &|_| true));
    let mut stage = DepositStage::recorded(&p).unwrap();
    drive(&mut stage, &mut w);
    let coal: Vec<&deeptime_format::Site> = w
        .sites
        .iter()
        .filter(|s| s.deposit_type == "deeptime:coal_seam")
        .collect();
    assert!(coal.len() >= 3, "coal sites: {}", coal.len());
    let mut stacks = 0;
    let mut footprints = Vec::new();
    for s in &coal {
        let bodies: Vec<&deeptime_format::bodies::Body> =
            w.bodies.iter().filter(|b| b.site == Some(s.id)).collect();
        assert!(!bodies.is_empty(), "site {} has no body", s.id);
        assert_eq!(s.body, Some(bodies[0].id));
        for (i, b) in bodies.iter().enumerate() {
            assert_eq!(b.role, "ore");
            assert_eq!(b.id, bodies[0].id + i as u32, "consecutive ids");
            assert_eq!(
                (b.center[0], b.center[2]),
                (bodies[0].center[0], bodies[0].center[2])
            );
            assert_eq!(
                (b.axes[0], b.axes[2]),
                (bodies[0].axes[0], bodies[0].axes[2])
            );
            assert_eq!(b.yaw, bodies[0].yaw);
            assert_eq!(b.dip, bodies[0].dip);
            if let Some(next) = bodies.get(i + 1) {
                // Parallel slabs (the dip tilts them alike): apart by more than their thickness.
                let gap = (b.center[1] - next.center[1]) * f64::from(b.dip).cos();
                assert!(
                    gap >= f64::from(b.axes[1] + next.axes[1]) - 1e-3,
                    "site {} seams {i}/{}: gap {gap} vs {} + {}",
                    s.id,
                    i + 1,
                    b.axes[1],
                    next.axes[1]
                );
            }
        }
        stacks += usize::from(bodies.len() > 1);
        footprints.push((
            (bodies[0].center[0], bodies[0].center[2]),
            [
                f64::from(bodies[0].axes[0]),
                f64::from(bodies[0].axes[2]),
                f64::from(bodies[0].yaw),
            ],
        ));
    }
    // The shrunken fields are thin (seams under 0.7 m are not minable, so few fit one field), yet
    // a good part of the sites still stack.
    assert!(stacks >= 5, "{stacks} of {} sites stack", coal.len());
    let c = f64::from(sp.circumference);
    for (i, (pa, ea)) in footprints.iter().enumerate() {
        for (pb, eb) in &footprints[i + 1..] {
            assert!(
                !cluster::ellipses_overlap(c, *pa, *ea, *pb, *eb),
                "coal fields at {pa:?} and {pb:?} overlap"
            );
        }
    }
}

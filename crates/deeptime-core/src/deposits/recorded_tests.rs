// SPDX-License-Identifier: LGPL-3.0-or-later
//! The deposit stage on the history the plates and surface stages record (P1-14): a small coupled
//! planet (plates → surface → deposits) whose sites come from the real recorded frames, the same
//! run on different thread counts, a checkpoint taken inside the deposit stage, and a history
//! with fields missing.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use super::fields::{FieldHistory, HOST_PREFIX};
use super::recorded::RecordedHistory;
use super::*;
use crate::checkpoint::Checkpoint;
use crate::params::planet_builtin;
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
    assert_eq!(s.stage_names(), ["plates", "surface", "deposits"]);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let census: Value = serde_json::from_str(&world.notes[STAGE_NAME]).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    (world, census)
}

fn sites_by_type(w: &WorldState) -> BTreeMap<&str, usize> {
    let mut out = BTreeMap::new();
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
    eprintln!("census: {census}");
    eprintln!("sites by type: {:?}", sites_by_type(&a));
    // Same result on any thread count.
    assert_eq!(a.sites, b.sites, "sites differ between 1 and 3 threads");
    assert_eq!(a.bodies, b.bodies, "bodies differ");
    assert_eq!(a.events, b.events, "events differ");
    assert_eq!(census, census_b, "census differs");
    assert_eq!(
        a.fields["deposits/sites"].data, b.fields["deposits/sites"].data,
        "sites layer differs"
    );
    // The history is still in the world after the stage (a checkpoint carries it).
    assert!(a.history.as_ref().is_some_and(|h| h.frame_count() == 10));
    // The census names its history.
    assert_eq!(census["history"]["source"], "recorded");
    assert_eq!(census["history"]["frames"], 10);
    // Every family that has its fields has sites: porphyry (with its children), coal (iron's
    // sedimentary member), Zn–Pb (MVT and SEDEX). BIF needs the P1-07 basement and the BIF acme
    // clock; petroleum needs P1-07's trap structure.
    let by = sites_by_type(&a);
    assert!(by.get("deeptime:porphyry_cu_mo_au").copied().unwrap_or(0) > 0, "{by:?}");
    assert!(by.get("deeptime:coal_seam").copied().unwrap_or(0) > 0, "{by:?}");
    assert!(
        by.get("deeptime:mvt_zn_pb").copied().unwrap_or(0)
            + by.get("deeptime:sedex_zn_pb").copied().unwrap_or(0)
            > 0,
        "{by:?}"
    );
    // Petroleum is skipped, by name, with the field it lacks.
    let skipped = census["skipped"].as_array().unwrap();
    let names: Vec<&str> = skipped.iter().map(|s| s["type"].as_str().unwrap()).collect();
    assert_eq!(names, ["deeptime:petroleum_trap"], "{skipped:?}");
    assert_eq!(
        skipped[0]["missing"],
        serde_json::json!(["deeptime:trap_structure"])
    );
    assert!(!by.contains_key("deeptime:petroleum_trap"));
    // No host fields yet: the documented fallback is named.
    let fallbacks = census["fallbacks"].as_array().unwrap();
    assert!(
        fallbacks
            .iter()
            .any(|f| f.as_str().unwrap().contains(HOST_PREFIX)),
        "{fallbacks:?}"
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

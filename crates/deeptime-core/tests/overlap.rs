// SPDX-License-Identifier: LGPL-3.0-or-later
//! A coupled group's follower stepping beside the lead's next step (P1-19: the structure stage's
//! step k beside the plate step k + 1, `Stage::steps_beside_lead`) leaves the world it leaves on
//! its own, on any number of threads: one run with the overlap off on one thread and one with it
//! on, on three, end the same byte for byte. The second run reads the world between `advance`
//! calls now and then (`Session::with_world`), which runs a step put off on the world first.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use deeptime_core::session::{AdvanceStatus, NoGauge, Session, SessionOptions};
use deeptime_core::world::WorldState;
use serde_json::{Value, json};

/// The history test's small coupled planet (L1 512 × 164) over 60 Myr, with a frame every third
/// plate step: the structure stage publishes into the history on some of the steps it takes
/// beside the plates and not on others.
fn preset() -> String {
    let json = deeptime_core::preset::with_timeline(
        deeptime_core::params::planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(60.0),
    )
    .unwrap();
    let mut v: Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = json!(3000);
    v["resolution"]["l1_r"] = json!(32);
    v["keyframes"]["size"] = json!([32, 16]);
    v["keyframes"]["every_myr"] = json!(20.0);
    v["seed"] = json!(7);
    v["surface"]["frame_every_myr"] = json!(6.0);
    v.as_object_mut().unwrap().remove("young");
    v.as_object_mut().unwrap().remove("amplify");
    v.to_string()
}

/// A note without its wall-clock times.
fn without_times(text: &str) -> Value {
    fn strip(v: &mut Value) {
        match v {
            Value::Object(m) => {
                m.retain(|k, _| k != "millis" && k != "timings" && !k.ends_with("_ms"));
                m.values_mut().for_each(strip);
            }
            Value::Array(a) => a.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut v: Value = serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.into()));
    strip(&mut v);
    v
}

/// Runs the planet; returns the world and the time the structure stage stepped beside the
/// plates (ms).
fn run(json: &str, threads: usize, overlap: bool, name: &str) -> (WorldState, f64) {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("deeptime-overlap-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let s = Session::create(
        json,
        "",
        SessionOptions {
            out_dir: dir.clone(),
            threads,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap();
    s.set_overlap(overlap);
    let mut calls = 0u64;
    loop {
        // One group step a call: a step put off waits for the next call.
        match s.advance(Duration::from_millis(1)).unwrap() {
            AdvanceStatus::Done => break,
            AdvanceStatus::Cancelled => panic!("cancelled"),
            _ => {}
        }
        calls += 1;
        if calls % 5 == 0 {
            s.with_world(|w| w.t_years).unwrap();
        }
    }
    let beside = s
        .stage_timings()
        .unwrap()
        .iter()
        .find(|t| t.name == deeptime_core::structure::STAGE_NAME)
        .map_or(0.0, |t| t.beside_millis);
    let world = s.with_world(Clone::clone).unwrap();
    drop(s);
    let _ = std::fs::remove_dir_all(&dir);
    (world, beside)
}

#[test]
fn stepping_beside_the_plates_changes_nothing() {
    let json = preset();
    let (a, alone) = run(&json, 1, false, "alone");
    let (b, beside) = run(&json, 3, true, "beside");
    assert_eq!(alone, 0.0, "the overlap off steps nothing beside the plates");
    assert!(beside > 0.0, "the structure stage stepped beside the plates");
    assert!(
        a.history.as_ref().is_some_and(|h| h.frame_count() >= 8),
        "the planet records frames"
    );
    assert_eq!(a.sites, b.sites);
    assert_eq!(a.bodies, b.bodies);
    assert_eq!(a.events, b.events);
    assert_eq!(
        a.notes.keys().collect::<Vec<_>>(),
        b.notes.keys().collect::<Vec<_>>()
    );
    for (k, text) in &a.notes {
        assert_eq!(without_times(text), without_times(&b.notes[k]), "note {k}");
    }
    assert!(a.history == b.history, "the recorded history");
    let body = |w: &WorldState| {
        let mut w = w.clone();
        w.notes.clear();
        postcard::to_stdvec(&w).unwrap()
    };
    assert!(body(&a) == body(&b), "the world's bytes");
}

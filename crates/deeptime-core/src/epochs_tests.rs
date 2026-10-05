// SPDX-License-Identifier: LGPL-3.0-or-later
//! Pick the epoch, stop early, continue and the nudges, on a small planet through the whole
//! pipeline and `finalize_write` (docs/design/playability-steering.md §9).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use deeptime_format::Manifest;
use serde_json::{Value, json};

use super::{EPOCHS_DIR, EpochIndex};
use crate::checkpoint::Checkpoint;
use crate::params::planet_builtin;
use crate::preset::{with_stop, with_timeline};
use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};
use crate::world::WorldState;

/// A small planet with every stage (the bake tests' planet): C 16,384 on coarse grids, a frame
/// every 10 Myr, `myr` long, with or without steering.
fn small_planet(myr: f64, steering: Option<Value>) -> String {
    let json = with_timeline(
        planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(myr),
    )
    .unwrap();
    let mut v: Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = json!(3000);
    v["resolution"]["l1_r"] = json!(64);
    v["resolution"]["l2_r"] = json!(32);
    v["resolution"]["l3_r"] = json!(16);
    v["keyframes"]["size"] = json!([128, 64]);
    v["keyframes"]["every_myr"] = json!(10.0);
    v["seed"] = json!(5);
    v["young"] = json!({"myr": 1.0, "steps": 2});
    v["amplify"] = json!({
        "erosion_iterations": 4, "thermal_iterations": 3, "deposition_iterations": 2,
        "coarse_iteration_factor": 1.0, "repair_margin_cells": 8,
        "tile_cells": 256, "preview_width": 128
    });
    match steering {
        Some(s) => v["steering"] = s,
        None => {
            v.as_object_mut().unwrap().remove("steering");
        }
    }
    v.to_string()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-epochs-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn options(dir: &Path) -> SessionOptions {
    SessionOptions {
        out_dir: dir.to_path_buf(),
        threads: 3,
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
    s.finalize_write().unwrap();
}

/// A run into `dir` (epochs on when asked); returns its final world.
fn run(json: &str, dir: &Path, epochs: bool) -> WorldState {
    let mut s = Session::create(json, "", options(dir), Arc::new(NoGauge)).unwrap();
    s.set_epochs(epochs).unwrap();
    finish(&s);
    s.checkpoint().unwrap().world
}

/// The run in `dir` again, to the stop of `json`; returns the session after it finished.
fn resume(json: &str, dir: &Path) -> Session {
    let mut s = Session::resume_epoch(json, "", options(dir), Arc::new(NoGauge)).unwrap();
    s.set_epochs(true).unwrap();
    finish(&s);
    s
}

/// Every world-data file the manifest lists, with its CRC (what the game reads).
fn files(dir: &Path) -> Vec<(String, u32)> {
    let m = Manifest::read(dir).unwrap();
    m.verify(dir).unwrap();
    let mut v: Vec<(String, u32)> = m.files.iter().map(|f| (f.path.clone(), f.crc32)).collect();
    v.sort();
    v
}

fn notes(dir: &Path) -> Value {
    serde_json::to_value(Manifest::read(dir).unwrap().notes).unwrap()
}

fn steering(extra: Value) -> Value {
    let mut v = json!({"enabled": true, "tail_fraction": 0.2});
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    v
}

/// Ben: "at the 500M year mark they can do a continue button and choose when to stop". A run
/// continued from 40 to 60 Myr is the run planned with `stop_myr` 60 from the start, file for
/// file, with and without steering; it reruns only the old tail.
#[test]
fn a_continued_run_is_the_run_planned_to_its_new_stop() {
    for (name, st) in [
        ("plain", None),
        ("steered", Some(steering(json!({"target": "custom",
            "land_min": 0.9, "land_max": 1.0, "temp_min_c": 40.0, "temp_max_c": 50.0,
            "sea_rate_m_myr": 10.0, "co2_rate_per_myr": 0.5})))),
    ] {
        let base = small_planet(40.0, st.clone());
        let a = temp_dir(&format!("continue-{name}"));
        run(&base, &a, true);
        let idx = EpochIndex::read(&a).unwrap().expect("an epochs index");
        assert!(!idx.entries.is_empty());
        let longer = with_stop(&base, Some(60.0)).unwrap();
        let s = resume(&longer, &a);
        let r = s.resumed().unwrap().clone();
        // The old run's last free step: 40 Myr in 2-Myr steps, the 3-Myr window (and 20 % of
        // steering, 8 Myr) before the end.
        let want_from = if st.is_some() { 16 } else { 18 };
        assert_eq!((r.from_step, r.steps), (want_from, 30), "{name}");
        let b = temp_dir(&format!("planned-{name}"));
        let planned = run(&longer, &b, false);
        assert_eq!(files(&a), files(&b), "{name}: the folders differ");
        assert_eq!(s.checkpoint().unwrap().world, planned, "{name}: the worlds differ");
        assert_eq!(notes(&a)["run"]["resumed"]["from_step"], json!(want_from));
        let _ = std::fs::remove_dir_all(&a);
        let _ = std::fs::remove_dir_all(&b);
    }
}

/// Ben: "Pick the epoch" and "if the player reaches a state after like 120M years that they think
/// is perfect for their run can we let them stop the sim early and get playing?". Baking a
/// keyframe of a finished run, and stopping a running run at a keyframe it drew, give the world
/// of a run planned to that keyframe's time; the earlier frames are kept.
#[test]
fn an_epoch_pick_and_a_stop_bake_the_run_planned_to_that_time() {
    let base = small_planet(60.0, Some(steering(json!({}))));
    let a = temp_dir("pick");
    run(&base, &a, true);
    let frames = deeptime_format::keyframes::KeyframeIndex::read(&a).unwrap();
    // The keyframe at 30 Myr after the start (t = −30 Myr).
    let pick = frames
        .frames
        .iter()
        .find(|f| (f.t + 30.0e6).abs() < 1.0)
        .expect("a frame at 30 Myr");
    let at = with_stop(&base, Some(30.0)).unwrap();
    let s = resume(&at, &a);
    let r = s.resumed().unwrap().clone();
    assert!(r.from_step > 0 && r.from_step < 15, "{r:?}");
    assert!(r.keyframes_kept > 0 && r.keyframes_kept <= pick.n + 1);
    let b = temp_dir("pick-planned");
    let planned = run(&at, &b, false);
    assert_eq!(files(&a), files(&b));
    assert_eq!(s.checkpoint().unwrap().world, planned);
    drop(s);

    // Stop early: a run cancelled after its 50-Myr frame, then stopped at 40 Myr.
    let c = temp_dir("stop");
    let mut live = Session::create(&base, "", options(&c), Arc::new(NoGauge)).unwrap();
    live.set_epochs(true).unwrap();
    while live.progress().sim_time > 9.0e6 as i64 {
        live.advance(Duration::from_millis(1)).unwrap();
    }
    live.cancel();
    drop(live);
    let early = with_stop(&base, Some(40.0)).unwrap();
    let s = resume(&early, &c);
    assert!(s.resumed().unwrap().from_step > 0);
    let d = temp_dir("stop-planned");
    let planned = run(&early, &d, false);
    assert_eq!(files(&c), files(&d));
    assert_eq!(s.checkpoint().unwrap().world, planned);
    for dir in [a, b, c, d] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// Steering off changes nothing: a preset without the section and one with `enabled: false` give
/// the same files; a steered run's world before its tail is the unsteered run's (its epoch
/// checkpoints match the unsteered ones step for step).
#[test]
fn steering_off_changes_nothing_and_on_changes_only_the_tail() {
    let plain = small_planet(60.0, None);
    let off = small_planet(60.0, Some(json!({"enabled": false})));
    let on = small_planet(60.0, Some(steering(json!({}))));
    let (a, b, c) = (temp_dir("off-a"), temp_dir("off-b"), temp_dir("off-c"));
    let wa = run(&plain, &a, true);
    let wb = run(&off, &b, false);
    assert_eq!(files(&a), files(&b));
    assert_eq!(wa, wb);
    assert!(notes(&b)["climate"].get("steering").is_none());
    run(&on, &c, true);
    let ia = EpochIndex::read(&a).unwrap().unwrap();
    let ic = EpochIndex::read(&c).unwrap().unwrap();
    assert_eq!(ia.key, ic.key, "steering is outside the key");
    let mut shared = 0;
    for e in &ic.entries {
        if let Some(u) = ia.entries.iter().find(|u| u.step == e.step) {
            let x = Checkpoint::read(&a.join(EPOCHS_DIR).join(&u.file)).unwrap();
            let y = Checkpoint::read(&c.join(EPOCHS_DIR).join(&e.file)).unwrap();
            assert_eq!(x.world, y.world, "step {}", e.step);
            shared += 1;
        }
    }
    assert!(shared >= 3, "{shared} shared steps");
    for dir in [a, b, c] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// The nudges reach targets set just outside the unsteered planet's end state, without jumps or
/// flip-flopping: the sea level moves one way by at most its rate a step, outgassing too.
#[test]
fn the_nudges_reach_their_targets_gently() {
    let plain = small_planet(60.0, None);
    let a = temp_dir("nudge-plain");
    run(&plain, &a, false);
    let rows = notes(&a)["climate"]["series"].clone();
    let cols: Vec<String> = serde_json::from_value(rows["columns"].clone()).unwrap();
    let last = rows["rows"].as_array().unwrap().last().unwrap().clone();
    let col = |n: &str| last[cols.iter().position(|c| c == n).unwrap()].as_f64().unwrap();
    let (land0, temp0) = (col("land_frac"), col("t_global_c"));
    // Targets above both: more land, warmer.
    let (sea_rate, co2_rate) = (10.0, 0.5);
    let on = small_planet(
        60.0,
        Some(steering(json!({
            "target": "custom", "tail_fraction": 0.4,
            "land_min": land0 + 0.03, "land_max": land0 + 0.10,
            "temp_min_c": temp0 + 2.0, "temp_max_c": temp0 + 6.0,
            "sea_rate_m_myr": sea_rate, "sea_max_m": 400.0,
            "co2_rate_per_myr": co2_rate, "co2_factor_max": 4.0
        }))),
    );
    let b = temp_dir("nudge-on");
    run(&on, &b, false);
    let st = notes(&b)["climate"]["steering"].clone();
    let end_land = st["end"]["land_frac"].as_f64().unwrap();
    let end_temp = st["end"]["t_global_c"].as_f64().unwrap();
    assert!(
        end_land >= land0 + 0.02,
        "land {land0} → {end_land}: {st}"
    );
    assert!(end_temp >= temp0 + 1.5, "temperature {temp0} → {end_temp}: {st}");
    assert!(st["sea_offset_m"].as_f64().unwrap() < 0.0, "the sea fell");
    assert!(st["outgassing_factor"].as_f64().unwrap() > 1.0);
    // No jump: at most the rate times the 2-Myr step; no reversal of either control.
    assert!(st["max_sea_step_m"].as_f64().unwrap() <= sea_rate * 2.0 + 1e-9);
    assert!(st["max_ln_outgassing_step"].as_f64().unwrap() <= co2_rate * 2.0 + 1e-9);
    assert_eq!(st["sea_reversals"], json!(0), "{st}");
    assert_eq!(st["co2_reversals"], json!(0), "{st}");
    for dir in [a, b] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

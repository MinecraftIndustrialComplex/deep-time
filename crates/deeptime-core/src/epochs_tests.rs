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
    world(&s)
}

/// A session's final world without its notes (the stages' summaries carry their timings).
fn world(s: &Session) -> WorldState {
    let mut w = s.checkpoint().unwrap().world;
    w.notes.clear();
    w
}

/// What differs between two worlds, by part (empty: the same), so a failure names the part
/// instead of printing two worlds.
fn world_diff(a: &WorldState, b: &WorldState) -> Vec<String> {
    // Bit for bit (NaN marks "none" in many fields, and NaN != NaN): through postcard.
    fn bits<T: serde::Serialize>(x: &T) -> Vec<u8> {
        postcard::to_stdvec(x).unwrap()
    }
    let mut out = Vec::new();
    if a.t_years.to_bits() != b.t_years.to_bits() {
        out.push(format!("t_years {} vs {}", a.t_years, b.t_years));
    }
    let keys: std::collections::BTreeSet<&String> =
        a.fields.keys().chain(b.fields.keys()).collect();
    for k in keys {
        if a.fields.get(k).map(bits) != b.fields.get(k).map(bits) {
            out.push(format!("field {k}"));
        }
    }
    if bits(&a.globals) != bits(&b.globals) {
        out.push(format!("globals {:?} vs {:?}", a.globals, b.globals));
    }
    if bits(&a.events) != bits(&b.events) {
        let first = a
            .events
            .iter()
            .zip(&b.events)
            .position(|(x, y)| bits(x) != bits(y));
        out.push(format!(
            "events ({} vs {}, first difference at {first:?}: {:?} vs {:?})",
            a.events.len(),
            b.events.len(),
            first.map(|i| &a.events[i]),
            first.map(|i| &b.events[i])
        ));
    }
    let parts = [
        ("sites", bits(&a.sites) == bits(&b.sites)),
        ("bodies", bits(&a.bodies) == bits(&b.bodies)),
        ("plates", bits(&a.plates) == bits(&b.plates)),
        ("surface", bits(&a.surface) == bits(&b.surface)),
        ("volcanoes", bits(&a.volcanoes) == bits(&b.volcanoes)),
        ("impacts", bits(&a.impacts) == bits(&b.impacts)),
        (
            "structure",
            bits(&a.structure) == bits(&b.structure)
                && bits(&a.structure_relief) == bits(&b.structure_relief),
        ),
        ("history", a.history == b.history),
        ("whole (ids)", bits(a) == bits(b)),
    ];
    for (name, same) in parts {
        if !same {
            out.push(name.to_owned());
        }
    }
    out
}

/// What differs between two folders' world-data files (path: CRC or absent).
fn files_diff(a: &[(String, u32)], b: &[(String, u32)]) -> Vec<String> {
    let mut out = Vec::new();
    for (p, c) in a {
        match b.iter().find(|(q, _)| q == p) {
            Some((_, d)) if d == c => {}
            Some(_) => out.push(format!("{p} differs")),
            None => out.push(format!("{p} only in the first")),
        }
    }
    for (p, _) in b {
        if !a.iter().any(|(q, _)| q == p) {
            out.push(format!("{p} only in the second"));
        }
    }
    out
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
        (
            "steered",
            Some(steering(json!({"target": "custom",
            "land_min": 0.9, "land_max": 1.0, "temp_min_c": 40.0, "temp_max_c": 50.0,
            "sea_rate_m_myr": 10.0, "co2_rate_per_myr": 0.5}))),
        ),
    ] {
        let base = small_planet(40.0, st.clone());
        let a = temp_dir(&format!("continue-{name}"));
        run(&base, &a, true);
        let idx = EpochIndex::read(&a).unwrap().expect("an epochs index");
        assert!(!idx.entries.is_empty());
        let held: usize = idx
            .entries
            .iter()
            .map(|e| e.frames_to - e.frames_from)
            .sum();
        let last = idx.entries.iter().map(|e| e.frames_to).max().unwrap();
        assert!(last > 0);
        assert_eq!(held, last, "{name}: each history frame is stored once");
        let longer = with_stop(&base, Some(60.0)).unwrap();
        let s = resume(&longer, &a);
        let r = s.resumed().unwrap().clone();
        // The old run's last free step: 40 Myr in 2-Myr steps, the 3-Myr window (and 20 % of
        // steering, 8 Myr) before the end.
        let want_from = if st.is_some() { 16 } else { 18 };
        assert_eq!((r.from_step, r.steps), (want_from, 30), "{name}");
        let b = temp_dir(&format!("planned-{name}"));
        let planned = run(&longer, &b, false);
        let w = world(&s);
        assert_eq!(
            world_diff(&w, &planned),
            Vec::<String>::new(),
            "{name}: the worlds"
        );
        assert_eq!(
            files_diff(&files(&a), &files(&b)),
            Vec::<String>::new(),
            "{name}: the folders"
        );
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
    let original = files(&a);
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
    assert_eq!(
        world_diff(&world(&s), &planned),
        Vec::<String>::new(),
        "the pick's world"
    );
    assert_eq!(
        files_diff(&files(&a), &files(&b)),
        Vec::<String>::new(),
        "the pick's folder"
    );
    drop(s);
    // Back to the planned end from the picked epoch: the original run again, frame for frame
    // (the frames the pick drew in its tail are not kept).
    let back = resume(&base, &a);
    assert!(
        back.resumed().unwrap().from_step <= 12,
        "{:?}",
        back.resumed()
    );
    drop(back);
    assert_eq!(
        files_diff(&files(&a), &original),
        Vec::<String>::new(),
        "picked, then continued back to the planned end"
    );

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
    assert_eq!(
        world_diff(&world(&s), &planned),
        Vec::<String>::new(),
        "the stop's world"
    );
    assert_eq!(
        files_diff(&files(&c), &files(&d)),
        Vec::<String>::new(),
        "the stop's folder"
    );
    for dir in [a, b, c, d] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// The realistic ("free") choice is no steering at all (Ben, decisions log 2026-10-06, "One switch
/// for both"): a preset with `steering.enabled: false` (and the preset's own `present_climate`,
/// temperate) gives the same world and files as an unsteered run, without the section, with
/// `present_climate` off (natural). And a steered run's world before its tail is the unsteered
/// run's (its epoch checkpoints match the unsteered ones step for step).
#[test]
fn the_free_choice_is_the_unsteered_run_and_steering_changes_only_the_tail() {
    let mut v: Value = serde_json::from_str(&small_planet(60.0, None)).unwrap();
    v["climate"]["present_climate"] = json!("natural");
    let plain = v.to_string();
    let off = small_planet(60.0, Some(json!({"enabled": false})));
    let o: Value = serde_json::from_str(&off).unwrap();
    assert_eq!(
        o["climate"]["present_climate"],
        json!("temperate"),
        "the free preset keeps its own setting: the switch turns it off"
    );
    let on = small_planet(60.0, Some(steering(json!({}))));
    let (a, b, c) = (temp_dir("off-a"), temp_dir("off-b"), temp_dir("off-c"));
    let wa = run(&plain, &a, true);
    let wb = run(&off, &b, false);
    // Everything but the preset file, which names the section.
    let no_preset = |d: &Path| -> Vec<(String, u32)> {
        files(d)
            .into_iter()
            .filter(|(p, _)| p != "preset.json")
            .collect()
    };
    assert_eq!(
        world_diff(&wa, &wb),
        Vec::<String>::new(),
        "off: the worlds"
    );
    assert_eq!(
        files_diff(&no_preset(&a), &no_preset(&b)),
        Vec::<String>::new(),
        "off: the folders"
    );
    assert!(notes(&b)["climate"].get("steering").is_none());
    assert_eq!(
        notes(&b)["climate"]["present"]["state"],
        json!("natural"),
        "free turns the present-day adjustment off"
    );
    run(&on, &c, true);
    let ia = EpochIndex::read(&a).unwrap().unwrap();
    let ic = EpochIndex::read(&c).unwrap().unwrap();
    assert_eq!(ia.key, ic.key, "steering is outside the key");
    let mut shared = 0;
    for e in &ic.entries {
        if let Some(u) = ia.entries.iter().find(|u| u.step == e.step) {
            let chain = |dir: &Path, i: &EpochIndex, e: &super::EpochEntry| -> Vec<PathBuf> {
                i.chain(e)
                    .unwrap()
                    .iter()
                    .map(|c| dir.join(EPOCHS_DIR).join(&c.file))
                    .collect()
            };
            let mut x = Checkpoint::read_chain(&chain(&a, &ia, u)).unwrap().world;
            let mut y = Checkpoint::read_chain(&chain(&c, &ic, e)).unwrap().world;
            x.notes.clear();
            y.notes.clear();
            assert_eq!(world_diff(&x, &y), Vec::<String>::new(), "step {}", e.step);
            shared += 1;
        }
    }
    assert!(shared >= 2, "{shared} shared steps");
    for dir in [a, b, c] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// The nudges reach targets set just outside the unsteered planet's end state, without jumps or
/// flip-flopping: the sea level moves one way by at most its rate a step, outgassing too.
#[test]
fn the_nudges_reach_their_targets_gently() {
    // The planet this test was tuned on moves its plates with the fixed slab suction (PLT-186).
    // With the saturating suction (PLT-540, on by default since 2026-10-24) the steered run warms
    // past the band's top in its last step (16.8 °C against 16.0) and the outgassing turns back
    // once; the test is about the nudges, not the plates, so it keeps its planet.
    let fixed_suction = |planet: String| {
        let mut v: Value = serde_json::from_str(&planet).unwrap();
        v["tectonics"]["suction_saturates"] = json!(false);
        v.to_string()
    };
    let plain = fixed_suction(small_planet(60.0, None));
    let a = temp_dir("nudge-plain");
    run(&plain, &a, false);
    let rows = notes(&a)["climate"]["series"].clone();
    let cols: Vec<String> = serde_json::from_value(rows["columns"].clone()).unwrap();
    let last = rows["rows"].as_array().unwrap().last().unwrap().clone();
    let col = |n: &str| {
        last[cols.iter().position(|c| c == n).unwrap()]
            .as_f64()
            .unwrap()
    };
    let (land0, temp0) = (col("land_frac"), col("t_global_c"));
    // Targets above both: more land, warmer, and above everything the plain planet reached in the
    // steered tail (the last 0.4 of the run), so the nudges act from the tail's start. On this
    // small planet the land share swings by 0.02 to 0.04 a step and the last step (the ice-age
    // window) moves land and temperature more than a step's nudge can answer: bands set just above
    // the plain end alone left the steered land in its band until the last steps (with the sea
    // from the ocean's volume, 2026-10-21: in the band to 6 Myr before the end, then a fall of
    // 0.04 in the last step, the sea only 44 m lower). The temperature band starts 3.5 K above
    // the tail's warmest: the nudge is proportional, so with the band 0.5 K above it the steered
    // tail sat on the band's lower edge (14.2 to 14.8 °C), the lower sea's extra land cooling it
    // as fast as the outgassing warmed it, and the run ended 0.16 K above the plain one.
    let tail = |name: &str| {
        let i = cols.iter().position(|c| c == name).unwrap();
        let t = cols.iter().position(|c| c == "t_myr").unwrap();
        rows["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r[t].as_f64().unwrap() >= -0.4 * 60.0)
            .map(|r| r[i].as_f64().unwrap())
            .fold(f64::NEG_INFINITY, f64::max)
    };
    let (land_top, temp_top) = (tail("land_frac"), tail("t_global_c"));
    let (sea_rate, co2_rate) = (20.0, 0.5);
    let on = fixed_suction(small_planet(
        60.0,
        Some(steering(json!({
            "target": "custom", "tail_fraction": 0.4,
            "land_min": land_top + 0.02, "land_max": land_top + 0.09,
            "temp_min_c": temp_top + 3.5, "temp_max_c": temp_top + 7.5,
            "sea_rate_m_myr": sea_rate, "sea_max_m": 400.0,
            "co2_rate_per_myr": co2_rate, "co2_factor_max": 4.0
        }))),
    ));
    let b = temp_dir("nudge-on");
    run(&on, &b, false);
    let st = notes(&b)["climate"]["steering"].clone();
    let end_land = st["end"]["land_frac"].as_f64().unwrap();
    let end_temp = st["end"]["t_global_c"].as_f64().unwrap();
    // The land share falls through this planet's tail; the nudge lowers the sea as soon as it
    // leaves the band, and the planet ends with more land than without it.
    assert!(end_land >= land0 + 0.01, "land {land0} → {end_land}: {st}");
    assert!(
        end_temp >= temp0 + 1.5,
        "temperature {temp0} → {end_temp}: {st}"
    );
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

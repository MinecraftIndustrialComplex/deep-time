// SPDX-License-Identifier: LGPL-3.0-or-later
//! The recorded history spilled to disk (P1-17; decisions log 2026-09-29): the coupled pipeline
//! with every frame sent to disk as soon as its step is done, or only some of them, on other
//! thread counts, gives the same world byte for byte as with every frame in memory; so does a run
//! resumed from checkpoints whose frames were on disk.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use super::STAGE_NAME;
use super::recorded_tests::{finish, pipeline_json, temp_dir};
use crate::checkpoint::Checkpoint;
use crate::session::{NoGauge, Session, SessionOptions};
use crate::world::WorldState;

fn options(dir: &Path, threads: usize) -> SessionOptions {
    SessionOptions {
        out_dir: dir.to_path_buf(),
        threads,
        mem_cap: 0,
    }
}

/// What a run ends with: the world, its census, and how much of the history went to disk.
struct Outcome {
    world: WorldState,
    census: Value,
    on_disk: usize,
    frames: usize,
    spilled_frames: u64,
}

fn run(json: &str, threads: usize, budget: Option<u64>, name: &str) -> Outcome {
    let dir = temp_dir(name);
    let mut s = Session::create(json, "", options(&dir, threads), Arc::new(NoGauge)).unwrap();
    s.set_history_budget(budget);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let census: Value = serde_json::from_str(&world.notes[STAGE_NAME]).unwrap();
    let h = world.history.as_ref().unwrap();
    let out = Outcome {
        on_disk: h.frames_on_disk(),
        frames: h.frame_count(),
        spilled_frames: s.spill().written_frames(),
        world,
        census,
    };
    drop(s);
    // The spill file never had a name on Unix, and is gone with the session anyway.
    #[cfg(unix)]
    assert!(
        std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .all(|e| !e.file_name().to_string_lossy().ends_with(".spill")),
        "no spill file left in the run's folder"
    );
    let _ = std::fs::remove_dir_all(&dir);
    out
}

/// A stage note without its wall-clock times (`millis`, `timings`, `*_ms`), which differ between
/// any two runs.
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

/// Byte-for-byte equality of two worlds: the postcard body (every field, the plates, the surface,
/// events, sites and bodies, NaNs included), the stage notes but for their timings, and the
/// recorded history.
fn assert_same_world(a: &WorldState, b: &WorldState, what: &str) {
    assert_eq!(a.sites, b.sites, "{what}: sites");
    assert_eq!(a.bodies, b.bodies, "{what}: bodies");
    assert_eq!(a.events, b.events, "{what}: events");
    assert_eq!(
        a.notes.keys().collect::<Vec<_>>(),
        b.notes.keys().collect::<Vec<_>>(),
        "{what}: notes"
    );
    for (k, text) in &a.notes {
        assert_eq!(
            without_times(text),
            without_times(&b.notes[k]),
            "{what}: note {k}"
        );
    }
    assert!(a.history == b.history, "{what}: the recorded history");
    let body = |w: &WorldState| {
        let mut w = w.clone();
        w.notes.clear();
        postcard::to_stdvec(&w).unwrap()
    };
    assert!(body(a) == body(b), "{what}: the world's bytes differ");
}

/// Every frame to disk after each step on 1 thread, a budget of about two frames on 2 threads,
/// and everything in memory on 3 threads: the same world, sites and census.
#[test]
fn a_history_spilled_to_disk_gives_the_same_world_and_census() {
    let json = pipeline_json(3000, 100.0, 5, 40.0);
    let memory = run(&json, 3, None, "spill-memory");
    assert_eq!(memory.on_disk, 0);
    assert_eq!(memory.spilled_frames, 0);
    assert_eq!(memory.frames, 10);
    assert!(!memory.world.sites.is_empty(), "the test planet has sites");

    let disk = run(&json, 1, Some(0), "spill-all");
    assert_eq!(
        disk.on_disk, disk.frames,
        "a budget of 0 leaves no frame in memory"
    );
    // The structure stage adds to each frame after the surface closed it: a frame on disk comes
    // back and is written again, so more frames are written than recorded.
    assert!(disk.spilled_frames >= disk.frames as u64);
    assert_same_world(&memory.world, &disk.world, "all on disk, 1 thread");
    assert_eq!(memory.census, disk.census);

    let frame = memory.world.history.as_ref().unwrap().bytes() / memory.frames as u64;
    let some = run(&json, 2, Some(2 * frame), "spill-some");
    assert!(
        some.on_disk > 0 && some.on_disk < some.frames,
        "a budget of two frames keeps some: {} of {} on disk",
        some.on_disk,
        some.frames
    );
    assert_same_world(&memory.world, &some.world, "partly on disk, 2 threads");
    assert_eq!(memory.census, some.census);
}

/// Checkpoints of a run whose frames are on disk — one inside the plates–surface group, one inside
/// the deposit stage — written to files (the frames streamed from the spill file), read back (the
/// frames left in the checkpoint) and resumed on other thread counts and budgets: the same world
/// as a straight run with everything in memory.
#[test]
fn a_checkpoint_with_a_spilled_history_resumes_to_the_same_result() {
    let json = pipeline_json(2000, 60.0, 3, 40.0);
    let straight = run(&json, 2, None, "ck-straight");
    let dir = temp_dir("ck-spilled");
    std::fs::create_dir_all(&dir).unwrap();

    // Inside the group: 17 plate steps of 30 (3 frames recorded, all on disk).
    let mut s = Session::create(&json, "", options(&dir.join("a"), 1), Arc::new(NoGauge)).unwrap();
    s.set_history_budget(Some(0));
    for _ in 0..17 {
        s.advance(Duration::ZERO).unwrap();
    }
    let mid_group = dir.join("group.dtcp");
    s.write_checkpoint(&mid_group).unwrap();
    {
        let h = s.checkpoint().unwrap().world.history.unwrap();
        assert_eq!(h.frame_count(), 3);
        assert_eq!(
            h.frames_on_disk(),
            3,
            "spilled when the checkpoint was written"
        );
    }
    // The checkpoint written from the spill file = one written from memory.
    let from_memory = s.checkpoint().unwrap();
    assert_eq!(
        std::fs::read(&mid_group).unwrap(),
        from_memory.to_bytes().unwrap()
    );
    drop((s, from_memory));

    // Resume it on 3 threads with a budget of one frame, into the deposit stage and a few frames
    // of its pass 1; checkpoint again there.
    let ck = Checkpoint::read(&mid_group).unwrap();
    assert_eq!(ck.next_stage_name(), Some("plates"));
    assert_eq!(ck.world.history.as_ref().unwrap().frames_on_disk(), 3);
    let frame = straight.world.history.as_ref().unwrap().bytes() / straight.frames as u64;
    let mut s = Session::resume(ck, options(&dir.join("b"), 3), Arc::new(NoGauge)).unwrap();
    s.set_history_budget(Some(frame));
    while s.progress().stage_index < 4 {
        s.advance(Duration::from_millis(1)).unwrap();
    }
    for _ in 0..3 {
        s.advance(Duration::ZERO).unwrap();
    }
    let in_deposits = dir.join("deposits.dtcp");
    s.write_checkpoint(&in_deposits).unwrap();
    drop(s);

    // The last leg on 2 threads, every frame on disk (read from the checkpoint and the new
    // spill file alike).
    let ck = Checkpoint::read(&in_deposits).unwrap();
    assert_eq!(ck.next_stage_name(), Some("deposits"));
    assert!(!ck.stage_state.is_empty(), "inside the deposit stage");
    let mut s = Session::resume(ck, options(&dir.join("c"), 2), Arc::new(NoGauge)).unwrap();
    s.set_history_budget(Some(0));
    finish(&s);
    let resumed = s.checkpoint().unwrap().world;
    drop(s);
    assert_same_world(
        &straight.world,
        &resumed,
        "resumed twice from spilled checkpoints",
    );
    let census: Value = serde_json::from_str(&resumed.notes[STAGE_NAME]).unwrap();
    assert_eq!(census, straight.census);
    let _ = std::fs::remove_dir_all(&dir);
}

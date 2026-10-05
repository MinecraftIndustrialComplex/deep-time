// SPDX-License-Identifier: LGPL-3.0-or-later
//! The memory cap spills the recorded history instead of failing (P1-17; decisions log
//! 2026-09-29, "Spill history to the SSD"), measured by the game's and the CLI's counting
//! allocator: a run whose cap is below its all-in-memory peak by half its history finishes, with
//! frames on disk, never over its cap, to the same world byte for byte; a cap the rest of the run
//! cannot fit still fails cleanly. One test in its own binary, so the allocator counts nothing
//! else.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use deeptime_core::CoreError;
use deeptime_core::alloc::{self, CountingAlloc};
use deeptime_core::session::{AdvanceStatus, MemoryGauge, Session, SessionOptions};
use deeptime_core::world::WorldState;
use serde_json::{Value, json};

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

struct Gauge;

/// Each phase's highest count in the current run (name, bytes), for the report of where a run
/// went over its cap.
static PHASES: Mutex<Vec<(String, u64)>> = Mutex::new(Vec::new());

impl MemoryGauge for Gauge {
    fn in_use(&self) -> u64 {
        alloc::in_use()
    }
    fn peak(&self) -> u64 {
        alloc::peak()
    }
    fn phase(&self, name: &str, begin: bool) {
        if begin {
            alloc::start_window();
            return;
        }
        let peak = alloc::window_peak();
        let mut p = PHASES.lock().unwrap();
        match p.iter_mut().find(|(n, _)| n == name) {
            Some(e) => e.1 = e.1.max(peak),
            None => p.push((name.to_owned(), peak)),
        }
    }
    fn phase_peak(&self) -> u64 {
        alloc::window_peak()
    }
    fn release(&self) -> u64 {
        alloc::release_cache()
    }
    fn keep_below(&self, bytes: u64) {
        alloc::keep_below(bytes);
    }
}

/// `deposits::recorded_tests`' small coupled planet (here 512 × 164 L1 cells) over 100 Myr, recording a
/// frame every plate step and keeping 8 layers of strata, so the history (50 frames) outweighs
/// the rest and half of it is more than the stages' reservations keep in hand.
fn preset() -> String {
    let json = deeptime_core::preset::with_timeline(
        deeptime_core::params::planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(100.0),
    )
    .unwrap();
    let mut v: Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = json!(3000);
    // L1 512 × 164: at 64 blocks a cell the Lambert grid (256 × 82) left too small a history to
    // outweigh the rest (it was tuned on the equirectangular 256 × 128).
    v["resolution"]["l1_r"] = json!(32);
    v["keyframes"]["size"] = json!([32, 16]);
    v["keyframes"]["every_myr"] = json!(20.0);
    v["seed"] = json!(5);
    v["deposits"]["count_mult"] = json!(40.0);
    v["surface"]["frame_every_myr"] = json!(2.0);
    v["surface"]["log_layers"] = json!(8);
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

/// A run's figures are relative to what the process held when it started (the gauge counts the
/// whole process, the worlds of earlier runs included).
struct Outcome {
    world: WorldState,
    peak: u64,
    /// Each phase's peak (above the run's start).
    phases: Vec<(String, u64)>,
    /// The most a reservation asked for, without the history's frames then in memory.
    needed: u64,
    spilled_frames: u64,
}

/// Runs `json` with `cap` bytes on top of what the process holds now.
fn run(json: &str, cap: u64, name: &str) -> Result<Outcome, CoreError> {
    let start = alloc::in_use();
    let dir: PathBuf = std::env::temp_dir().join(format!(
        "deeptime-history-cap-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    alloc::reset_peak();
    PHASES.lock().unwrap().clear();
    let s = Session::create(
        json,
        "",
        SessionOptions {
            out_dir: dir.clone(),
            threads: 2,
            mem_cap: start.saturating_add(cap),
        },
        Arc::new(Gauge),
    )?;
    let mut spills = 0usize;
    let mut show = |s: &Session| {
        for line in s.log().drain() {
            if line.contains("spilled to disk") {
                spills += 1;
                if spills > 3 {
                    continue;
                }
            }
            if (line.contains("history") || line.contains("failed"))
                && !line.contains("session created")
            {
                eprintln!("[{name}] {}", line.replace('\t', " "));
            }
        }
    };
    let result = loop {
        let status = s.advance(Duration::from_millis(100));
        show(&s);
        match status {
            Ok(AdvanceStatus::Done) => break Ok(()),
            Ok(AdvanceStatus::Cancelled) => panic!("cancelled"),
            Ok(_) => {}
            Err(e) => break Err(e),
        }
    };
    let peak = alloc::peak() - start;
    let phases = PHASES
        .lock()
        .unwrap()
        .iter()
        .map(|(n, b)| (n.clone(), b.saturating_sub(start)))
        .collect();
    eprintln!("[{name}] {spills} spill lines");
    result?;
    let world = s.with_world(Clone::clone)?;
    let spilled_frames = s.spill().written_frames();
    let needed = s.needed_base_peak().saturating_sub(start);
    drop(s);
    let _ = std::fs::remove_dir_all(&dir);
    Ok(Outcome {
        world,
        peak,
        phases,
        needed,
        spilled_frames,
    })
}

#[test]
fn a_run_over_its_cap_spills_its_history_and_ends_the_same() {
    // The history's spill alone: no freed blocks kept for reuse (they would be released first).
    alloc::set_cache_limit(0);
    let json = preset();
    let free = run(&json, u64::MAX, "unlimited").unwrap();
    assert_eq!(free.spilled_frames, 0);
    let h = free.world.history.as_ref().unwrap();
    let history = h.bytes();
    assert!(h.frame_count() >= 45, "{} frames", h.frame_count());
    eprintln!(
        "unlimited: peak {:.1} MiB, reservations up to {:.1} MiB, history {:.1} MiB in {} frames",
        free.peak as f64 / 1048576.0,
        free.needed as f64 / 1048576.0,
        history as f64 / 1048576.0,
        h.frame_count()
    );

    // Half the history cannot stay in memory; every reservation still fits (the stages ask for
    // their working sets before the history grows).
    let cap = (free.peak - history / 2).max(free.needed + (2 << 20));
    assert!(
        cap + history / 4 < free.peak,
        "the planet must force a spill: cap {cap}, peak {}, history {history}",
        free.peak
    );
    let capped = run(&json, cap, "capped").expect("the run fits once its history spills");
    eprintln!(
        "cap {:.1} MiB: peak {:.1} MiB, {} frames spilled",
        cap as f64 / 1048576.0,
        capped.peak as f64 / 1048576.0,
        capped.spilled_frames
    );
    assert!(capped.spilled_frames > 0, "frames went to disk");
    assert!(capped.peak < free.peak, "the spill kept the peak down");
    // The cap holds: the run never held more than it (memacct, 2026-10-04).
    let over: Vec<String> = capped
        .phases
        .iter()
        .filter(|(_, b)| *b > cap)
        .map(|(n, b)| format!("{n} {:.1} MiB", *b as f64 / 1048576.0))
        .collect();
    assert!(
        capped.peak <= cap,
        "the run held {:.1} MiB under a {:.1} MiB cap (over it in: {over:?})",
        capped.peak as f64 / 1048576.0,
        cap as f64 / 1048576.0
    );
    let (a, b) = (&free.world, &capped.world);
    assert_eq!(a.sites, b.sites);
    assert_eq!(a.bodies, b.bodies);
    assert_eq!(a.events, b.events);
    for (k, text) in &a.notes {
        assert_eq!(without_times(text), without_times(&b.notes[k]), "note {k}");
    }
    assert!(a.history == b.history, "the recorded history");
    let body = |w: &WorldState| {
        let mut w = w.clone();
        w.notes.clear();
        postcard::to_stdvec(&w).unwrap()
    };
    assert!(body(a) == body(b), "the world's bytes");

    // A cap the base cannot fit fails cleanly, naming the cap.
    match run(&json, 1 << 20, "tiny") {
        Err(CoreError::MemoryCap { cap, needed }) => assert!(needed > cap),
        Err(e) => panic!("expected the memory cap, got {e}"),
        Ok(_) => panic!("a 1 MiB cap cannot fit the planet"),
    }
}

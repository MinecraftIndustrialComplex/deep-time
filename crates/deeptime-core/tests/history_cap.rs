// SPDX-License-Identifier: LGPL-3.0-or-later
//! The memory cap spills the recorded history instead of failing (P1-17; decisions log
//! 2026-09-29, "Spill history to the SSD"), measured by a counting allocator as in the game and
//! the CLI: a run whose cap is below its all-in-memory peak by half its history finishes, with
//! frames on disk, to the same world byte for byte; a cap the rest of the run cannot fit still
//! fails cleanly. One test in its own binary, so the allocator counts nothing else.

use std::alloc::{GlobalAlloc, Layout, System};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use deeptime_core::CoreError;
use deeptime_core::session::{AdvanceStatus, MemoryGauge, Session, SessionOptions};
use deeptime_core::world::WorldState;
use serde_json::{Value, json};

static IN_USE: AtomicU64 = AtomicU64::new(0);
static PEAK: AtomicU64 = AtomicU64::new(0);

struct Counting;

// SAFETY: every method forwards to `System` with the caller's layout and only updates counters.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwarded unchanged.
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            let now =
                IN_USE.fetch_add(layout.size() as u64, Ordering::Relaxed) + layout.size() as u64;
            PEAK.fetch_max(now, Ordering::Relaxed);
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` came from `alloc` with `layout`.
        unsafe { System.dealloc(ptr, layout) };
        IN_USE.fetch_sub(layout.size() as u64, Ordering::Relaxed);
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

struct Gauge;

impl MemoryGauge for Gauge {
    fn in_use(&self) -> u64 {
        IN_USE.load(Ordering::Relaxed)
    }
    fn peak(&self) -> u64 {
        PEAK.load(Ordering::Relaxed)
    }
}

/// `deposits::recorded_tests`' small coupled planet (256 × 128 L1 cells) over 100 Myr, recording a
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
    v["resolution"]["l1_r"] = json!(64);
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
    /// The most a reservation asked for, without the history's frames then in memory.
    needed: u64,
    spilled_frames: u64,
}

/// Runs `json` with `cap` bytes on top of what the process holds now.
fn run(json: &str, cap: u64, name: &str) -> Result<Outcome, CoreError> {
    let start = IN_USE.load(Ordering::Relaxed);
    let dir: PathBuf = std::env::temp_dir().join(format!(
        "deeptime-history-cap-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    PEAK.store(IN_USE.load(Ordering::Relaxed), Ordering::Relaxed);
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
    let peak = PEAK.load(Ordering::Relaxed) - start;
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
        needed,
        spilled_frames,
    })
}

#[test]
fn a_run_over_its_cap_spills_its_history_and_ends_the_same() {
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

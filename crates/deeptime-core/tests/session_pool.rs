// SPDX-License-Identifier: LGPL-3.0-or-later
//! A session works on its own Rayon pool and nowhere else (P1-17). Its thread count is the
//! player's choice (the game's setting, `--threads` on the command line), so no part of a run may
//! fall through to Rayon's global pool, which has every core of the machine. Two parts did: the
//! deposits stage's horizon bake (its coal seams' cover) and the water view drawn from the bake,
//! both called outside the session's `install`. On the test VM's 32 cores they ran on 32 threads
//! beside a 2-thread session, and the freed blocks of those threads' allocator arenas put the
//! resident peak of `tests/memory_estimate.rs` over its bound.
//!
//! Here the global pool has one thread, and that thread is kept busy for the whole run: work sent
//! to the global pool waits for it forever, so a run that sends any would never finish. One test
//! in its own binary, so the global pool is this test's to set up.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use deeptime_core::session::{AdvanceStatus, NoGauge, Session, SessionOptions};
use serde_json::{Value, json};

/// The memory test's small planet with every stage and the bake (`tests/memory_estimate.rs`).
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
    v["resolution"]["l2_r"] = json!(32);
    v["resolution"]["l3_r"] = json!(16);
    v["keyframes"]["size"] = json!([128, 64]);
    v["keyframes"]["every_myr"] = json!(20.0);
    v["seed"] = json!(5);
    v["young"] = json!({"myr": 1.0, "steps": 2});
    v["amplify"] = json!({
        "erosion_iterations": 4, "thermal_iterations": 3, "deposition_iterations": 2,
        "coarse_iteration_factor": 1.0, "repair_margin_cells": 8,
        "tile_cells": 256, "preview_width": 128
    });
    v.to_string()
}

#[test]
fn a_run_never_uses_the_global_pool() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build_global()
        .expect("the global pool is set up before anything uses it");
    // The global pool's one thread waits until the run is over.
    let (release, held) = mpsc::channel::<()>();
    rayon::spawn(move || {
        let _ = held.recv();
    });

    let dir: PathBuf =
        std::env::temp_dir().join(format!("deeptime-session-pool-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (done, finished) = mpsc::channel();
    let out = dir.clone();
    std::thread::spawn(move || {
        let s = Session::create(
            &preset(),
            "",
            SessionOptions {
                out_dir: out,
                threads: 2,
                mem_cap: 0,
            },
            Arc::new(NoGauge),
        )
        .unwrap();
        loop {
            match s.advance(Duration::from_secs(60)).unwrap() {
                AdvanceStatus::Done => break,
                AdvanceStatus::Cancelled => panic!("cancelled"),
                _ => {}
            }
        }
        s.finalize_write().unwrap();
        let _ = done.send(());
    });
    // The run takes about half a minute on two threads; a run that sent work to the global pool
    // would wait for ever.
    let result = finished.recv_timeout(Duration::from_secs(900));
    let _ = release.send(());
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        result.is_ok(),
        "the run did not finish: some of its work went to Rayon's global pool (or it panicked)"
    );
}

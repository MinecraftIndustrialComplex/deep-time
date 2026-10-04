// SPDX-License-Identifier: LGPL-3.0-or-later
//! The estimate is honest (memacct, 2026-10-04): on a small planet through every stage and the
//! bake, measured by the game's own counting allocator, the memory the run holds while a stage
//! steps is never more than that stage's estimate, and the run's peak never more than the
//! estimate's peak. One test in its own binary, so the allocator counts nothing else.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use deeptime_core::alloc::{self, CountingAlloc};
use deeptime_core::session::{AdvanceStatus, MemoryGauge, Session, SessionOptions};
use serde_json::{Value, json};

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// The allocator's counters, and each phase's peak (bytes above what the process held at the
/// start).
struct Gauge {
    start: u64,
    peaks: Mutex<BTreeMap<String, u64>>,
}

impl MemoryGauge for Gauge {
    fn in_use(&self) -> u64 {
        alloc::in_use()
    }
    fn peak(&self) -> u64 {
        alloc::peak()
    }
    fn phase_peak(&self) -> u64 {
        alloc::window_peak()
    }
    fn release(&self) -> u64 {
        alloc::release_cache()
    }
    fn phase(&self, name: &str, begin: bool) {
        if begin {
            alloc::start_window();
            return;
        }
        let peak = self.phase_peak().saturating_sub(self.start);
        let mut p = self.peaks.lock().unwrap();
        let e = p.entry(name.to_owned()).or_default();
        *e = (*e).max(peak);
    }
}

/// The bake tests' small planet with every stage (L1 256 × 82, L2 512 × 164, L3 1024 × 328),
/// 100 Myr.
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
fn no_stage_holds_more_than_its_estimate() {
    let json = preset();
    let estimate: Value =
        serde_json::from_str(&deeptime_core::session::estimate_for(&json, "", 0).unwrap())
            .unwrap();
    let dir: PathBuf =
        std::env::temp_dir().join(format!("deeptime-memory-estimate-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    // The model's figures, without the freed blocks the allocator keeps for reuse (at most
    // `alloc::CACHE_BYTES`, which the estimate adds on its own).
    alloc::set_cache_limit(0);
    let start = alloc::in_use();
    alloc::reset_peak();
    let gauge = Arc::new(Gauge {
        start,
        peaks: Mutex::new(BTreeMap::new()),
    });
    let s = Session::create(
        &json,
        "",
        SessionOptions {
            out_dir: dir.clone(),
            threads: 2,
            mem_cap: 0,
        },
        gauge.clone(),
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
    let run_peak = alloc::peak().saturating_sub(start);
    drop(s);
    let _ = std::fs::remove_dir_all(&dir);

    let mib = |b: u64| b as f64 / 1048576.0;
    let peaks = gauge.peaks.lock().unwrap().clone();
    let mut over = Vec::new();
    for st in estimate["stages"].as_array().unwrap() {
        let name = st["name"].as_str().unwrap();
        let est = st["bytes"].as_u64().unwrap();
        let held = peaks.get(name).copied().unwrap_or(0);
        eprintln!(
            "{name:10} held {:7.1} MiB  estimate {:7.1} MiB",
            mib(held),
            mib(est)
        );
        if held > est {
            over.push(format!("{name}: {:.1} > {:.1} MiB", mib(held), mib(est)));
        }
    }
    let est_peak = estimate["peak_bytes"].as_u64().unwrap();
    eprintln!(
        "run       peak {:7.1} MiB  estimate {:7.1} MiB (keyframes {:.1}, bake {:.1})",
        mib(run_peak),
        mib(est_peak),
        mib(peaks.get("keyframes").copied().unwrap_or(0)),
        mib(peaks.get("bake").copied().unwrap_or(0)),
    );
    if run_peak > est_peak {
        over.push(format!("the run: {:.1} > {:.1} MiB", mib(run_peak), mib(est_peak)));
    }
    assert!(over.is_empty(), "held more than estimated: {over:?}");
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The estimate is honest (memacct, 2026-10-04): on a small planet through every stage and the
//! bake, measured by the game's own counting allocator, the memory the run holds while a stage
//! steps is never more than that stage's estimate, and the run's peak never more than the
//! estimate's peak; and what the allocator counts is what the process holds (on Linux, the
//! resident peak of the run is within 10 % of the counted peak). One test in its own binary, so
//! the allocator counts nothing else.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use deeptime_core::alloc::{self, CountingAlloc};
use deeptime_core::compute::ComputeMode;
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
    fn kept(&self) -> u64 {
        alloc::kept_bytes()
    }
    fn keep_below(&self, bytes: u64) {
        alloc::keep_below(bytes);
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
        serde_json::from_str(&deeptime_core::session::estimate_for(&json, "", 0).unwrap()).unwrap();
    let dir: PathBuf =
        std::env::temp_dir().join(format!("deeptime-memory-estimate-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    // With the freed blocks the allocator keeps for reuse, as in the game: the session keeps
    // them inside each stage's estimate (`MemoryGauge::keep_below`).
    alloc::release_cache();
    let rss_start = rss_reset();
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
    // On the CPU: a GPU's driver (its code and its own heaps) is memory the counting allocator
    // never sees, so the resident bound below does not hold for it. With `auto` the amplify stage
    // probes for a device, and on the test VM that loads Mesa's lavapipe (rejected, a software
    // adapter): 2026-10-19, resident peak 137.5 and 138.8 MiB against 99.9 and 98.3 on the CPU,
    // the counted peak 95.3 MiB in all four (docs/design/gpu-compute.md, section 4).
    s.set_compute(ComputeMode::Cpu);
    loop {
        match s.advance(Duration::from_secs(60)).unwrap() {
            AdvanceStatus::Done => break,
            AdvanceStatus::Cancelled => panic!("cancelled"),
            _ => {}
        }
    }
    s.finalize_write().unwrap();
    let run_peak = alloc::peak().saturating_sub(start);
    let rss_peak = rss_hwm().map(|h| h.saturating_sub(rss_start.unwrap_or(0)));
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
    let est_bake = estimate["bake_bytes"]
        .as_u64()
        .expect("the bake is in the estimate");
    eprintln!(
        "bake      held {:7.1} MiB  estimate {:7.1} MiB (projection pass {:.1})",
        mib(peaks.get("bake").copied().unwrap_or(0)),
        mib(est_bake),
        mib(estimate["bake_projection_bytes"].as_u64().unwrap()),
    );
    // The bake's own figure holds too (amp-est, 2026-10-23: it listed 452.6 MB for a projection
    // pass that held 964.8 MB on quick 16k, and the session keeps the allocator's blocks under it).
    let bake_held = peaks.get("bake").copied().unwrap_or(0);
    if bake_held > est_bake {
        over.push(format!(
            "bake: {:.1} > {:.1} MiB",
            mib(bake_held),
            mib(est_bake)
        ));
    }
    if run_peak > est_peak {
        over.push(format!(
            "the run: {:.1} > {:.1} MiB",
            mib(run_peak),
            mib(est_peak)
        ));
    }
    assert!(over.is_empty(), "held more than estimated: {over:?}");
    // The count is the process's memory: the resident peak above the start is within 10 % of
    // the counted peak (plus 16 MiB for thread stacks, the code paged in and the C library's own
    // buffers). Before memacct the resident peak was a third over the count (glibc kept freed
    // grids). On 2026-10-17 it was over the bound again: parts of the run used Rayon's global
    // pool, every core (`tests/session_pool.rs`), and the session's threads' glibc arenas kept
    // the small blocks they freed, more on a busy machine (`alloc::release_cache` now trims
    // them). The bound is unchanged.
    if let (Some(rss), true) = (rss_peak, rss_start.is_some()) {
        let slack = (16u64 << 20) + run_peak / 10;
        eprintln!(
            "resident peak {:.1} MiB, counted peak {:.1} MiB, bound {:.1} MiB",
            mib(rss),
            mib(run_peak),
            mib(run_peak + slack)
        );
        assert!(
            rss <= run_peak + slack,
            "the process held {:.1} MiB, the count said {:.1} MiB",
            mib(rss),
            mib(run_peak)
        );
        assert!(
            run_peak <= rss + slack,
            "the count said {:.1} MiB, the process held {:.1} MiB",
            mib(run_peak),
            mib(rss)
        );
    }
}

/// `/proc/self/status` field `key` in bytes (Linux).
fn status_bytes(key: &str) -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/status").ok()?;
    let kb: u64 = s
        .lines()
        .find_map(|l| l.strip_prefix(key))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    Some(kb * 1024)
}

/// Resets the resident peak (`VmHWM`) to the resident size now and returns that size; `None`
/// where the kernel does not allow it (not Linux).
fn rss_reset() -> Option<u64> {
    std::fs::write("/proc/self/clear_refs", "5").ok()?;
    status_bytes("VmRSS:")
}

/// The resident peak since [`rss_reset`].
fn rss_hwm() -> Option<u64> {
    status_bytes("VmHWM:")
}

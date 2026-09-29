// SPDX-License-Identifier: LGPL-3.0-or-later
//! The P1-04 checks (plan §10.3 P1-04, §9.2): receivers acyclic, depressions resolved, mass
//! conserved by diffusion and by a whole erosion–deposition–marine step, the stream-power steady
//! state on a ramp against its analytic profile, the strata log's invariants, and the coupled
//! plates + surface pipeline (fields, frames, events, resume).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::grid::Dims;
use super::pflood::{BASE, LAND, SINK};
use super::strata_log::StrataLog;
use super::*;
use crate::checkpoint::Checkpoint;
use crate::deposits::fields::FieldHistory;
use crate::params::planet_builtin;
use crate::preset::with_timeline;
use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};
use crate::sphere::SplitMix64;

/// A noisy DEM with pits and a sea band along the first rows: (dims, h, kind, base_z).
fn noisy(w: usize, h: usize, seed: u64) -> (Dims, Vec<f64>, Vec<u8>, Vec<f64>) {
    let d = Dims::new(w, h, 1000.0, true);
    let mut rng = SplitMix64::new(seed);
    let mut z = vec![0.0; w * h];
    let mut kind = vec![LAND; w * h];
    for c in 0..w * h {
        let j = c / w;
        if j < 3 {
            z[c] = -100.0 - rng.next_f64() * 50.0;
            kind[c] = BASE;
        } else {
            z[c] = j as f64 * 20.0 + rng.next_f64() * 400.0;
        }
    }
    (d, z, kind, vec![0.0; w * h])
}

#[test]
fn receivers_are_acyclic_and_depressions_resolved() {
    for seed in 1..6 {
        let (d, h, mut kind, base_z) = noisy(48, 40, seed);
        let mut filled = vec![0.0; d.cells()];
        let sinks = pflood::flood(&d, &h, &base_z, &mut kind, &mut filled);
        assert_eq!(sinks, 0, "the sea reaches every cell");
        let mut lakes = 0;
        for c in 0..d.cells() {
            assert!(filled[c] >= h[c] || kind[c] == BASE, "cell {c}");
            if kind[c] == LAND {
                // Depressions resolved: a strictly lower neighbour on the filled surface.
                assert!(
                    d.neighbours(c).any(|(m, _)| filled[m] < filled[c]),
                    "seed {seed} cell {c} is a pit"
                );
                if filled[c] - h[c] > pflood::LAKE_TOL_M {
                    lakes += 1;
                }
            }
        }
        assert!(lakes > 0, "the noisy DEM has pits to fill");
        let mut r = routing::Routing::default();
        routing::receivers(&d, &filled, &kind, &mut r);
        routing::stack(&d, &kind, &mut r);
        assert!(routing::check_acyclic(&r), "seed {seed}");
        // Every land cell is in the stack once, after its receiver, and drains to the sea.
        let land = kind.iter().filter(|&&k| k == LAND).count();
        assert_eq!(r.stack.len(), land);
        let mut pos = vec![usize::MAX; d.cells()];
        for (i, &c) in r.stack.iter().enumerate() {
            pos[c as usize] = i;
        }
        for &c in &r.stack {
            let c = c as usize;
            let rc = r.rcv[c] as usize;
            assert!(kind[rc] == BASE || pos[rc] < pos[c]);
            let mut x = c;
            while r.rcv[x] as usize != x {
                x = r.rcv[x] as usize;
            }
            assert_eq!(kind[x], BASE);
        }
        let mut area = Vec::new();
        routing::accumulate(&d, &r, &vec![1.0; d.cells()], &mut area);
        let into_sea: f64 = (0..d.cells())
            .filter(|&c| kind[c] == BASE)
            .map(|c| area[c] - d.cell_area())
            .sum();
        assert!((into_sea / d.cell_area() - land as f64).abs() < 1e-6);
    }
}

#[test]
fn land_without_a_sea_drains_to_a_sink() {
    let d = Dims::new(8, 6, 100.0, true);
    let h: Vec<f64> = (0..48).map(|c| 10.0 + (c % 7) as f64).collect();
    let mut kind = vec![LAND; 48];
    let mut filled = vec![0.0; 48];
    let sinks = pflood::flood(&d, &h, &vec![0.0; 48], &mut kind, &mut filled);
    assert_eq!(sinks, 1);
    assert_eq!(kind.iter().filter(|&&k| k == SINK).count(), 1);
    let mut r = routing::Routing::default();
    routing::receivers(&d, &filled, &kind, &mut r);
    routing::stack(&d, &kind, &mut r);
    assert!(routing::check_acyclic(&r));
    assert_eq!(r.stack.len(), 48);
}

#[test]
fn diffusion_conserves_mass() {
    let mut rng = SplitMix64::new(7);
    for (w, h, wrap) in [(64, 32, true), (33, 17, false), (2, 5, true), (1, 9, true)] {
        let d = Dims::new(w, h, 4800.0, wrap);
        let n = d.cells();
        let mut z: Vec<f64> = (0..n).map(|_| rng.next_f64() * 4000.0 - 2000.0).collect();
        let kd: Vec<f64> = (0..n)
            .map(|c| match c % 5 {
                0 => 0.0,
                1 => 1.0e-20,
                2 => 5.0,
                3 => 300.0,
                _ => 1.0e5,
            })
            .collect();
        let before: f64 = z.iter().sum();
        for _ in 0..10 {
            diffusion::step(&d, &mut z, &kd, 2.0e6);
        }
        let after: f64 = z.iter().sum();
        assert!(
            (before - after).abs() <= 1e-9 * (n as f64) * 2000.0,
            "{w}x{h}: {before} → {after}"
        );
        // Diffusion is monotone: no new extremes.
        assert!(
            z.iter()
                .all(|v| v.is_finite() && *v > -2000.1 && *v < 2000.1)
        );
    }
}

/// Runs the stream power on a ramp of identical columns (row 0 the base level) to steady state.
fn ramp(n: f64, g: f64) -> (Dims, Vec<f64>, f64, f64, f64) {
    let d = Dims::new(4, 40, 500.0, true);
    let cells = d.cells();
    let (k, m, u) = (2.0e-5, 0.5, 1.0e-3);
    let dt = 5.0e5;
    let mut z: Vec<f64> = (0..cells).map(|c| (c / d.w) as f64 * 0.01).collect();
    let mut kind = vec![LAND; cells];
    for (c, kk) in kind.iter_mut().enumerate().take(d.w) {
        *kk = BASE;
        z[c] = 0.0;
    }
    let base_z = vec![0.0; cells];
    let kf = vec![k; cells];
    let p = spl::SplParams {
        m,
        n,
        g,
        dt,
        sc: f64::INFINITY,
        max_iter: 20,
        tol_m: 1e-6,
    };
    let mut filled = vec![0.0; cells];
    let mut r = routing::Routing::default();
    let mut area = Vec::new();
    let mut out = vec![0.0; cells];
    let mut qin = vec![0.0; cells];
    for _ in 0..4000 {
        let h0: Vec<f64> = (0..cells)
            .map(|c| if kind[c] == BASE { z[c] } else { z[c] + u * dt })
            .collect();
        let mut kd = kind.clone();
        pflood::flood(&d, &h0, &base_z, &mut kd, &mut filled);
        routing::receivers(&d, &filled, &kd, &mut r);
        routing::stack(&d, &kd, &mut r);
        routing::accumulate(&d, &r, &vec![1.0; cells], &mut area);
        let input = spl::SplInput {
            kind: &kd,
            h0: &h0,
            filled: &filled,
            base_z: &base_z,
            area: &area,
            kf: &kf,
        };
        spl::solve(&d, &r, &input, &p, &mut out, &mut qin);
        z.copy_from_slice(&out);
    }
    (d, z, k, m, u)
}

#[test]
fn stream_power_matches_the_analytic_ramp() {
    for n in [1.0, 2.0] {
        let (d, z, k, m, u) = ramp(n, 0.0);
        // Steady state of U = K A^m S^n on the discrete ramp: row j drains to row j − 1 with the
        // area of the rows above it, A_j = (H − j)·dx², so S_j = (U / (K A_j^m))^(1/n).
        let mut expect = 0.0;
        let mut worst: f64 = 0.0;
        for j in 1..d.h {
            let a = (d.h - j) as f64 * d.dx * d.dx;
            expect += d.dx * (u / (k * a.powf(m))).powf(1.0 / n);
            for i in 0..d.w {
                let got = z[j * d.w + i];
                worst = worst.max(((got - expect) / expect).abs());
            }
        }
        // Tolerance: the implicit scheme's fixed point is the discrete analytic profile, so the
        // error is round-off plus what 4000 steps of 0.5 Myr leave of the approach (n = 2 converges
        // more slowly near the divide).
        let tol = if n == 1.0 { 1e-6 } else { 1e-3 };
        assert!(worst < tol, "n = {n}: worst relative error {worst:e}");
    }
}

#[test]
fn erosion_and_deposition_conserve_mass() {
    // A noisy landscape draining to a sea: stream power with deposition (G = 1), the mouth fill
    // and diffusion move material, never make or lose it.
    let (d, h, mut kind, base_z) = noisy(40, 30, 3);
    let n = d.cells();
    let a = d.cell_area();
    let mut filled = vec![0.0; n];
    pflood::flood(&d, &h, &base_z, &mut kind, &mut filled);
    let mut r = routing::Routing::default();
    routing::receivers(&d, &filled, &kind, &mut r);
    routing::stack(&d, &kind, &mut r);
    let mut area = Vec::new();
    routing::accumulate(&d, &r, &vec![1.0; n], &mut area);
    let kf = vec![1.0e-5; n];
    let p = spl::SplParams {
        m: 0.5,
        n: 1.0,
        g: 1.0,
        dt: 2.0e6,
        sc: 0.3,
        max_iter: 8,
        tol_m: 0.01,
    };
    let input = spl::SplInput {
        kind: &kind,
        h0: &h,
        filled: &filled,
        base_z: &base_z,
        area: &area,
        kf: &kf,
    };
    let mut z = vec![0.0; n];
    let mut qin = vec![0.0; n];
    let rep = spl::solve(&d, &r, &input, &p, &mut z, &mut qin);
    assert!(rep.iterations >= 2);
    let land_loss: f64 = (0..n)
        .filter(|&c| kind[c] != BASE)
        .map(|c| (h[c] - z[c]) * a)
        .sum();
    let delivered: f64 = (0..n)
        .filter(|&c| kind[c] == BASE)
        .map(|c| qin[c] * p.dt)
        .sum();
    assert!(land_loss > 0.0);
    assert!(
        (land_loss - delivered).abs() < 1e-9 * land_loss,
        "{land_loss} vs {delivered}"
    );
    // Deposition happened somewhere (lakes fill, G deposits on low slopes).
    assert!((0..n).any(|c| kind[c] == LAND && z[c] > h[c]));
    // The mouth fill spends every mouth's load on the sea.
    let sea: Vec<bool> = kind.iter().map(|&k| k == BASE).collect();
    let before: f64 = z.iter().sum();
    let mut dep = vec![0.0f32; n];
    let mut s = marine::FillScratch::default();
    let fp = marine::FillParams {
        sea_level: 0.0,
        fill_depth_m: 20.0,
        shelf_slope: 0.0005,
        max_cells: 100_000,
    };
    for m in 0..n {
        if sea[m] && qin[m] > 0.0 {
            marine::fill(&d, &sea, m, qin[m] * p.dt, &fp, &mut z, &mut dep, &mut s);
        }
    }
    let after: f64 = z.iter().sum();
    assert!(((after - before) * a - delivered).abs() < 1e-9 * delivered);
    // The whole planet's material is unchanged.
    let start: f64 = h.iter().sum();
    let mut kd = vec![5.0; n];
    for c in 0..n {
        if sea[c] {
            kd[c] = marine::submarine_kd(-z[c], 300.0, 100.0, 2.0, 5.0);
        }
    }
    diffusion::step(&d, &mut z, &kd, p.dt);
    let end: f64 = z.iter().sum();
    assert!((end - start).abs() < 1e-9 * start.abs().max(1.0) * n as f64);
}

#[test]
fn strata_log_keeps_its_invariants() {
    let classes = crate::synth::EVERY_CLASS.len();
    let mut rng = SplitMix64::new(11);
    let mut log = StrataLog::new(5, 6);
    let mut totals = [0.0f64; 5];
    let mut t = -500.0f32;
    for step in 0..4000 {
        let c = (rng.next_u64() % 5) as usize;
        t += 0.1;
        if rng.next_f64() < 0.65 {
            let th = (rng.next_f64() * 30.0) as f32 + 0.01;
            // Few classes so same-class merges happen often.
            let class = (rng.next_u64() % 4) as u16 * 7;
            log.deposit(c, t, th, class, strata_log::provenance(1, 0.5));
            totals[c] += f64::from(th);
        } else {
            let amount = rng.next_f64() * 40.0;
            let removed = log.erode(c, amount);
            assert!(removed <= amount + 1e-9);
            totals[c] -= removed;
        }
        #[allow(clippy::needless_range_loop)]
        for k in 0..5 {
            log.check(k, classes)
                .unwrap_or_else(|e| panic!("step {step}: {e}"));
            assert!(
                (log.total(k) - totals[k]).abs() < 1e-2 * totals[k].max(1.0),
                "step {step}: cell {k} holds {} of {}",
                log.total(k),
                totals[k]
            );
        }
    }
    assert!(log.count.contains(&6), "the cap was reached");
}

/// A small coupled planet: `n` L0 samples, L1 r = 64 at 16k (256 × 128), `myr` of history.
fn pipeline_json(n: u32, myr: f64, seed: u64) -> String {
    let json = with_timeline(
        planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(myr),
    )
    .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = serde_json::json!(n);
    v["resolution"]["l1_r"] = serde_json::json!(64);
    v["keyframes"]["size"] = serde_json::json!([128, 64]);
    v["keyframes"]["every_myr"] = serde_json::json!(20.0);
    v["seed"] = serde_json::json!(seed);
    v.to_string()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-surface-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn session(json: &str, dir: &Path) -> Session {
    Session::create(
        json,
        "",
        SessionOptions {
            out_dir: dir.to_path_buf(),
            threads: 2,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap()
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

#[test]
fn the_coupled_pipeline_runs_and_resumes() {
    let json = pipeline_json(3000, 60.0, 5);
    let dir = temp_dir("straight");
    let s = session(&json, &dir);
    assert_eq!(s.stage_names(), ["plates", "surface", "structure"]);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let st = world.surface.as_ref().expect("the surface ran");
    assert_eq!(st.totals.steps, 30);
    let classes = crate::synth::EVERY_CLASS.len();
    for c in 0..st.log.cells() {
        st.log.check(c, classes).unwrap();
    }
    assert!(
        st.log.count.iter().any(|&n| n > 0),
        "something was deposited"
    );
    assert!(st.totals.eroded_km3 > 0.0 && st.totals.marine_km3 > 0.0);
    // Mass: the processes only add carbonate and evaporite (round-off aside).
    let moved = st.totals.eroded_km3.max(1.0);
    assert!(
        st.totals.mass_residual_km3.abs() < 1e-6 * moved,
        "residual {} of {moved} km³",
        st.totals.mass_residual_km3
    );
    for name in [
        field::ELEVATION_M,
        field::EROSION,
        field::CLASS,
        field::DRAINAGE,
        "plates/top_pool",
        "plates/flags",
    ] {
        assert!(world.fields.contains_key(name), "{name}");
    }
    // The deposit engine's history: 6 frames of 10 Myr with every published field.
    let h = world.history.as_ref().expect("frames recorded");
    assert_eq!(h.frame_count(), 6);
    let names = h.names();
    for f in crate::deposits::fields::FIELDS {
        assert!(
            names.iter().any(|n| n == f.name),
            "{} not published",
            f.name
        );
    }
    let frame = h.frame(5).unwrap();
    assert_eq!(frame.t_years, 0.0);
    assert!(h.receivers().is_some_and(|r| r.len() == h.grid().cells()));
    let r = h.receivers().unwrap();
    let routing = routing::Routing {
        rcv: r.to_vec(),
        ..Default::default()
    };
    assert!(routing::check_acyclic(&routing));
    // Every built-in deposit type finds its fields (the trap structure and the host fractions
    // come from the structure stage, P1-07).
    for t in crate::deposits::types::builtin().unwrap() {
        let missing: Vec<String> = t
            .fields()
            .into_iter()
            .filter(|f| !names.contains(f))
            .collect();
        assert!(missing.is_empty(), "{} lacks {missing:?}", t.id);
    }
    // Resume from a checkpoint in the middle of the group: the same world.
    let dir2 = temp_dir("resumed");
    let s2 = session(&json, &dir2);
    for _ in 0..11 {
        assert_eq!(s2.advance(Duration::ZERO).unwrap(), AdvanceStatus::Running);
    }
    let ck = s2.checkpoint().unwrap();
    assert_eq!(ck.next_stage_name(), Some("plates"));
    let bytes = ck.to_bytes().unwrap();
    let ck = Checkpoint::from_bytes(Path::new("ck"), &bytes).unwrap();
    let s3 = Session::resume(
        ck,
        SessionOptions {
            out_dir: dir2.clone(),
            threads: 3,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap();
    finish(&s3);
    let got = s3.checkpoint().unwrap().world;
    let a = got.surface.as_ref().unwrap();
    assert_eq!(a.elevation_m, st.elevation_m, "resumed surface differs");
    assert_eq!(a.log, st.log);
    assert_eq!(got.events.len(), world.events.len());
    assert_eq!(got.history, world.history);
    assert_eq!(got.bodies, world.bodies);
    for d in [dir, dir2] {
        let _ = std::fs::remove_dir_all(d);
    }
}

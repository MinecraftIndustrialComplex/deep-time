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

/// HYD-110: a lake keeps what its rivers bring up to its room and settles it over all of its
/// floor, every cell raised by the same share of its depth (not a causeway along the flow path);
/// a small supply fills part of the room and passes nothing on, a large one fills it all.
#[test]
fn a_lake_settles_its_inflow_over_its_whole_floor() {
    let (w, rows) = (20, 30);
    let d = Dims::new(w, rows, 1000.0, true);
    let n = d.cells();
    let a = d.cell_area();
    // A ramp up from a sea along the first rows, with a bowl 200 m deep across rows 10 to 15 of
    // columns 5 to 14: its downstream rim (row 9) is the spill, so rows 10 to 12 hold a lake.
    let mut kind = vec![LAND; n];
    let mut h = vec![0.0; n];
    for c in 0..n {
        let (i, j) = (c % w, c / w);
        if j < 3 {
            kind[c] = BASE;
            h[c] = -100.0;
        } else {
            h[c] = j as f64 * 50.0 + (i as f64 * 0.37).sin();
            if (10..16).contains(&j) && (5..15).contains(&i) {
                h[c] -= 200.0;
            }
        }
    }
    let base_z = vec![0.0; n];
    let mut filled = vec![0.0; n];
    pflood::flood(&d, &h, &base_z, &mut kind, &mut filled);
    let lake: Vec<usize> = (0..n)
        .filter(|&c| kind[c] == LAND && filled[c] - h[c] > pflood::LAKE_TOL_M)
        .collect();
    assert!(lake.len() >= 20, "{} lake cells", lake.len());
    let room: f64 = lake.iter().map(|&c| filled[c] - h[c]).sum();
    let mut r = routing::Routing::default();
    routing::receivers(&d, &filled, &kind, &mut r);
    routing::stack(&d, &kind, &mut r);
    let mut area = Vec::new();
    routing::accumulate(&d, &r, &vec![1.0; n], &mut area);
    let mut shares = Vec::new();
    for kf in [2.0e-9, 1.0e-6] {
        let p = spl::SplParams {
            m: 0.5,
            n: 1.0,
            g: 0.0,
            dt: 1.0e6,
            sc: f64::INFINITY,
            max_iter: 4,
            tol_m: 0.01,
        };
        let kfv = vec![kf; n];
        let input = spl::SplInput {
            kind: &kind,
            h0: &h,
            filled: &filled,
            base_z: &base_z,
            area: &area,
            kf: &kfv,
        };
        let mut z = vec![0.0; n];
        let mut qin = vec![0.0; n];
        spl::solve(&d, &r, &input, &p, &mut z, &mut qin);
        // One share of its depth over the whole lake, off the flow path too.
        let share = (z[lake[0]] - h[lake[0]]) / (filled[lake[0]] - h[lake[0]]);
        for &c in &lake {
            let s = (z[c] - h[c]) / (filled[c] - h[c]);
            assert!(
                (s - share).abs() < 1e-9,
                "kf {kf}: cell {c} share {s} vs {share}"
            );
        }
        assert!(
            share > 0.0 && share <= 1.0 + 1e-12,
            "kf {kf}: share {share}"
        );
        let kept: f64 = lake.iter().map(|&c| z[c] - h[c]).sum();
        assert!((kept - share * room).abs() < 1e-9 * room);
        // Mass: the land's loss is what the sea got.
        let land_loss: f64 = (0..n)
            .filter(|&c| kind[c] != BASE)
            .map(|c| (h[c] - z[c]) * a)
            .sum();
        let delivered: f64 = (0..n)
            .filter(|&c| kind[c] == BASE)
            .map(|c| qin[c] * p.dt)
            .sum();
        assert!(
            (land_loss - delivered).abs() < 1e-9 * land_loss.abs().max(a),
            "kf {kf}: {land_loss} vs {delivered}"
        );
        // The provenance mix: every lake cell takes the mix entering the lake.
        let mut mix = Vec::new();
        spl::families(&d, &r, &input, &vec![0u8; n], p.dt, &z, &mut mix);
        for &c in &lake {
            assert!(mix[c][0] > 0.0, "kf {kf}: cell {c} has no mix");
        }
        shares.push(share);
    }
    assert!(
        shares[0] < 1.0 - 1e-6,
        "a small supply fills part of the lake: {shares:?}"
    );
    assert!(
        (shares[1] - 1.0).abs() < 1e-12,
        "a large one fills it: {shares:?}"
    );
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
    // These tests are about the plates and the surface; the deposit stage has its own
    // (`deposits::tests`).
    v.as_object_mut().unwrap().remove("deposits");
    // The young landscape and L3 amplification (P1-06) have their own tests
    // (`amplify::tests`).
    v.as_object_mut().unwrap().remove("young");
    v.as_object_mut().unwrap().remove("amplify");
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
    assert_eq!(
        s.stage_names(),
        [
            "plates",
            "climate",
            "surface",
            "volcanoes",
            "impacts",
            "structure"
        ]
    );
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
    // The periplatform tracer lays no more carbonate than its diffusion took (ERO-195).
    assert!(
        st.totals.periplatform_km3 <= st.totals.periplatform_source_km3 * (1.0 + 1.0e-4) + 1.0e-9
    );
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
    // come from the structure stage, P1-07; the regional fields the deposit engine derives on
    // reading, `deposits::regional`).
    let mut names = names;
    names.extend(crate::deposits::regional::derivable(&names));
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

/// Chemical denudation in the coupled pipeline (`climate.chemical_denudation`, ERO-205). Off: no
/// field, no globals, nothing dissolved. On: the climate places its weathered Ca + Mg on land as a
/// thickness of rock every step, the surface takes it off (its dissolved total grows and the mass
/// residual still closes with it), and the planet differs.
#[test]
fn chemical_denudation_takes_the_weathered_rock_away() {
    let run = |on: bool, name: &str| {
        let mut v: serde_json::Value = serde_json::from_str(&pipeline_json(3000, 30.0, 7)).unwrap();
        v["climate"]["chemical_denudation"] = serde_json::json!(on);
        let dir = temp_dir(name);
        let s = session(&v.to_string(), &dir);
        finish(&s);
        let world = s.checkpoint().unwrap().world;
        let _ = std::fs::remove_dir_all(dir);
        world
    };
    let off = run(false, "chem-off");
    let on = run(true, "chem-on");
    let (a, c) = (off.surface.as_ref().unwrap(), on.surface.as_ref().unwrap());
    assert!(!off.fields.contains_key(crate::climate::field::DISSOLVED));
    assert!(
        !off.globals
            .contains_key(crate::climate::DISSOLVED_MOL_GLOBAL)
    );
    assert_eq!(a.totals.dissolved_km3, 0.0);
    assert!(c.totals.dissolved_km3 > 0.0);
    assert!(on.globals[crate::climate::DISSOLVED_MOL_GLOBAL] > 0.0);
    let moved = c.totals.eroded_km3.max(1.0);
    assert!(
        c.totals.mass_residual_km3.abs() < 1e-6 * moved,
        "residual {} of {moved} km³",
        c.totals.mass_residual_km3
    );
    // The last step's dissolved rock: on part of the planet (its land), m per 2-Myr step at
    // the pace of weathering (m/Myr), not of a landslide.
    let dz = &on.fields[crate::climate::field::DISSOLVED].data;
    assert!(dz.iter().all(|x| x.is_finite() && *x >= 0.0));
    let wet: Vec<f32> = dz.iter().copied().filter(|&x| x > 0.0).collect();
    assert!(!wet.is_empty() && wet.len() < dz.len());
    let mean_m_myr = wet.iter().map(|&x| f64::from(x)).sum::<f64>() / wet.len() as f64 / 2.0;
    assert!((0.01..20.0).contains(&mean_m_myr), "{mean_m_myr} m/Myr");
    assert_ne!(a.elevation_m, c.elevation_m);
    // The pelagic rain (`pelagic.rs`) returns the dissolved load: off, no opal and no returned
    // carbonate; on, its silica comes back as opal mol for mol and its Ca + Mg as carbonate, all
    // of the supply laid (what a full cell cannot hold settles where there is room).
    assert_eq!(
        (a.totals.opal_km3, a.totals.returned_carbonate_km3),
        (0.0, 0.0)
    );
    let t = &c.totals;
    assert!(t.returned_carbonate_km3 > 0.0 && t.opal_km3 > 0.0, "{t:?}");
    let opal_km3 = t.dissolved_km3 * crate::climate::carbon::ROCK_KG_M3 * pelagic::SILICA_SHARE
        / pelagic::SIO2_KG_PER_MOL
        * pelagic::opal_m3_per_mol();
    assert!(
        (t.opal_km3 - opal_km3).abs() <= 1e-5 * opal_km3,
        "opal {} km³ against the dissolved silica's {opal_km3}",
        t.opal_km3
    );
    assert!(
        t.rain_unlaid_km3.abs() <= 1e-6 * (opal_km3 + t.returned_carbonate_km3),
        "unlaid {} km³",
        t.rain_unlaid_km3
    );
    eprintln!(
        "dissolved {:.4} km³: returned carbonate {:.4}, opal {:.4}, dust {:.4}, unlaid {:.2e}",
        t.dissolved_km3, t.returned_carbonate_km3, t.opal_km3, t.dust_km3, t.rain_unlaid_km3
    );
}

/// Flexure in the coupled pipeline (`flexure.rs`). With `flexure.strength` 0 the stage is the one
/// it was: no state, no fields, and the same planet as a preset without the section. With it on,
/// every step is solved and converges, the plate carries loads its cells did not make (basins and
/// bulges), the buoyancy of the sinking balances the load, and surface processes still conserve
/// mass.
#[test]
fn flexure_off_changes_nothing_and_on_bends_the_plate() {
    let run = |edit: &dyn Fn(&mut serde_json::Value), name: &str| {
        let mut v: serde_json::Value = serde_json::from_str(&pipeline_json(3000, 40.0, 7)).unwrap();
        edit(&mut v);
        let dir = temp_dir(name);
        let s = session(&v.to_string(), &dir);
        finish(&s);
        let world = s.checkpoint().unwrap().world;
        let _ = std::fs::remove_dir_all(dir);
        world
    };
    let off = run(
        &|v| v["surface"]["flexure"]["strength"] = serde_json::json!(0.0),
        "flex-off",
    );
    let absent = run(
        &|v| {
            v["surface"].as_object_mut().unwrap().remove("flexure");
        },
        "flex-absent",
    );
    let on = run(
        &|v| v["surface"]["flexure"]["strength"] = serde_json::json!(1.0),
        "flex-on",
    );
    let (a, b, c) = (
        off.surface.as_ref().unwrap(),
        absent.surface.as_ref().unwrap(),
        on.surface.as_ref().unwrap(),
    );
    assert!(a.flexure.is_none() && b.flexure.is_none());
    assert_eq!(
        a.elevation_m, b.elevation_m,
        "strength 0 is not the old stage"
    );
    assert_eq!(a.log, b.log);
    for name in [
        flexure::field::FLEXURE,
        flexure::field::DEFLECTION,
        flexure::field::ELASTIC_THICKNESS,
    ] {
        assert!(!off.fields.contains_key(name), "{name} without flexure");
        assert!(on.fields.contains_key(name), "{name}");
    }
    assert!(!off.globals.contains_key(flexure::SOLVES_GLOBAL));
    assert_eq!(on.globals[flexure::SOLVES_GLOBAL], 20.0);
    assert!(!on.globals.contains_key(flexure::UNCONVERGED_GLOBAL));
    let fs = c.flexure.as_ref().expect("the plate's state");
    assert!(c.elevation_m.iter().all(|z| z.is_finite()));
    assert_ne!(a.elevation_m, c.elevation_m);
    let moved = c.totals.eroded_km3.max(1.0);
    assert!(c.totals.mass_residual_km3.abs() < 1e-6 * moved);
    // Stiff and soft crust both exist, and the stiff crust carries load regionally: some cells
    // stand well below local isostasy (basins, moats) and some above (loads held up, bulges).
    let te = &on.fields[flexure::field::ELASTIC_THICKNESS].data;
    let (te_min, te_max) = te
        .iter()
        .fold((f32::MAX, f32::MIN), |m, &t| (m.0.min(t), m.1.max(t)));
    assert!(te_min < 10.0 && te_max > 40.0, "Te {te_min}–{te_max} km");
    let flex = &on.fields[flexure::field::FLEXURE].data;
    let below = flex.iter().filter(|&&f| f < -50.0).count();
    let above = flex.iter().filter(|&&f| f > 50.0).count();
    assert!(
        below > 20 && above > 20,
        "{below} cells below, {above} above"
    );
    // Compensation: the mantle displaced (ρ_m·w) weighs what the load does (rock and the sea
    // over it, ρ_m·Q, plus the ice of the last step), to the solve's tolerance.
    let ice = on
        .fields
        .get(crate::climate::field::ICE_THICKNESS)
        .map(|f| f.data.clone())
        .unwrap_or_else(|| vec![0.0; flex.len()]);
    let load: f64 = fs
        .load_m
        .iter()
        .zip(&ice)
        .map(|(&q, &i)| f64::from(q) + flexure::RHO_ICE / flexure::RHO_MANTLE * f64::from(i))
        .sum();
    let sunk: f64 = fs.deflection_m.iter().map(|&w| f64::from(w)).sum();
    let scale: f64 = fs.load_m.iter().map(|&q| f64::from(q).abs()).sum();
    assert!(scale > 1.0e4, "there are loads: {scale}");
    assert!(
        (sunk - load).abs() < 0.02 * scale,
        "sinking {sunk} against load {load} (of {scale})"
    );
}

/// The held offsets (ISO-125) in the coupled pipeline: with `surface.held_offsets` the surface
/// marks the cells whose offset it holds (a gap between plates, a trench row) and hands the plate
/// model none of their offset (NaN in the feedback; the plate model counts such a cell as an
/// offset of 0 at its weight), every other cell's whole offset as before; without it neither the
/// mask nor a NaN.
#[test]
fn the_surface_hands_the_plate_model_nothing_of_a_held_offset() {
    let run = |on: bool, name: &str| {
        let mut v: serde_json::Value = serde_json::from_str(&pipeline_json(3000, 40.0, 7)).unwrap();
        v["surface"]["held_offsets"] = serde_json::json!(on);
        let dir = temp_dir(name);
        let s = session(&v.to_string(), &dir);
        finish(&s);
        let world = s.checkpoint().unwrap().world;
        let _ = std::fs::remove_dir_all(dir);
        world
    };
    let off = run(false, "held-off");
    assert!(!off.fields.contains_key(field::OFFSET_HELD));
    assert!(
        off.fields[field::FEEDBACK]
            .data
            .iter()
            .all(|x| x.is_finite())
    );
    let on = run(true, "held-on");
    let held = &on.fields[field::OFFSET_HELD].data;
    let fb = &on.fields[field::FEEDBACK].data;
    let n_held = held.iter().filter(|&&k| k > 0.5).count();
    assert!(n_held > 0, "a trench row or a gap somewhere");
    assert!(n_held < held.len() / 4, "{n_held} of {}", held.len());
    for (c, (&k, &f)) in held.iter().zip(fb).enumerate() {
        assert_eq!(k > 0.5, f.is_nan(), "cell {c}: held {k}, feedback {f}");
    }
    let st = on.surface.as_ref().unwrap();
    assert!(st.elevation_m.iter().all(|z| z.is_finite()));
    let samples = &on.plates.as_ref().unwrap().samples;
    assert!(samples.iter().all(|s| s.elevation_m.is_finite()));
}

/// The surface on large cells (build log 2026-10-18, the surface divergence): the small coupled
/// planet at 1,221 m a block, the Earth-proportioned scale, whose cells are 78 km wide. While the
/// plate held the plateau spreading's change up as crust (before ISO-070) this planet grew a
/// checkerboard of peaks and pits from about 80 Myr in and ended 100 Myr at 29.9 km; now it ends
/// at 4.0 km. Every cell stays within twice Earth's relief of the sea (Everest's summit and the
/// Challenger Deep, ISO-071).
#[test]
fn a_planet_of_large_cells_keeps_its_surface_physical() {
    let mut v: serde_json::Value = serde_json::from_str(&pipeline_json(3000, 100.0, 11)).unwrap();
    v["scale"]["m_per_block_h"] = serde_json::json!(1221.0);
    let dir = temp_dir("large-cells");
    let s = session(&v.to_string(), &dir);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let _ = std::fs::remove_dir_all(dir);
    let sea = world
        .globals
        .get(crate::climate::SEA_LEVEL_GLOBAL)
        .copied()
        .unwrap_or(0.0);
    let st = world.surface.as_ref().expect("the surface ran");
    assert_eq!(st.totals.steps, 50);
    let (lo, hi) = st.elevation_m.iter().fold((f64::MAX, f64::MIN), |m, &z| {
        (m.0.min(f64::from(z)), m.1.max(f64::from(z)))
    });
    assert!(
        hi - sea < 2.0 * guard::EVEREST_M && sea - lo < 2.0 * guard::CHALLENGER_DEEP_M,
        "the surface spans {:.0} to {:.0} m from the sea",
        lo - sea,
        hi - sea
    );
}

/// A plate step of two substeps (the 4-Myr step, P1-19 #27): the periplatform tracer reads the
/// column as each substep leaves it (`carbonate::StepTop`), and the carbonate it lays is never
/// more than the carbonate its diffusion took off sea cells (ERO-195). Mass still balances.
#[test]
fn two_substeps_keep_the_periplatform_tracer_within_its_source() {
    let mut v: serde_json::Value = serde_json::from_str(&pipeline_json(3000, 80.0, 5)).unwrap();
    v["tectonics"]["time_step_myr"] = serde_json::json!(4.0);
    v["surface"]["substeps"] = serde_json::json!(2);
    v["surface"]["frame_every_myr"] = serde_json::json!(20.0);
    let dir = temp_dir("two-substeps");
    let s = session(&v.to_string(), &dir);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let _ = std::fs::remove_dir_all(dir);
    let st = world.surface.as_ref().expect("the surface ran");
    assert_eq!(st.totals.steps, 20);
    let classes = crate::synth::EVERY_CLASS.len();
    for c in 0..st.log.cells() {
        st.log.check(c, classes).unwrap();
    }
    let t = &st.totals;
    assert!(
        t.periplatform_km3 <= t.periplatform_source_km3 * (1.0 + 1.0e-4) + 1.0e-9,
        "periplatform {} km³ over its source {} km³",
        t.periplatform_km3,
        t.periplatform_source_km3
    );
    let moved = t.eroded_km3.max(1.0);
    assert!(
        t.mass_residual_km3.abs() < 1e-6 * moved,
        "residual {} of {moved} km³",
        t.mass_residual_km3
    );
}

// ledger: ERO-221, ERO-011, HYD-033
/// The step's wiring of `surface.runoff_budget` (the runoff law's review): with the switch, the
/// step takes the bake's budget per cell (`step_water`) on the climate stage's own months and
/// the stream power's weight (`runoff_weight`) is its runoff; without it, the snapshot's own,
/// whatever the lakes take. Four climates at 1 km on the stage's fields: ground that never
/// thaws (mean −12, months −2 and −25 °C), a polar desert's year (−17, −5 and −40: centred on its
/// mean, the range would put its warmest month above freezing), a cold mean with a summer (−1.5,
/// 6.5 and −9.5) and frost-free land (15, 22 and 8).
#[test]
fn the_runoff_budget_reaches_the_stream_power_s_weight() {
    let grid = deeptime_format::GridDesc::for_planet("L1", 16384, 64).unwrap();
    let (w, h) = (grid.w as usize, grid.h as usize);
    let n = w * h;
    let d = Dims::new(w, h, 4800.0, true);
    let climates = [
        (300.0f32, -12.0f32, -2.0f32, -25.0f32),
        (300.0, -17.0, -5.0, -40.0),
        (300.0, -1.5, 6.5, -9.5),
        (800.0, 15.0, 22.0, 8.0),
    ];
    let of = |c: usize| climates[(c % w) % climates.len()];
    let mut world = WorldState::new(1, 16384, 0.0);
    for (name, pick) in [
        (climate::P_FIELD, 0usize),
        (climate::T_FIELD, 1),
        (crate::climate::field::T_WARM, 2),
        (crate::climate::field::T_COLD, 3),
    ] {
        let data = (0..n)
            .map(|c| {
                let (p, t, wm, cd) = of(c);
                [p, t, wm, cd][pick]
            })
            .collect();
        world.fields.insert(
            name.into(),
            Field {
                grid: grid.clone(),
                data,
            },
        );
    }
    let elevation = vec![1000.0; n];
    let clim = climate::snapshot(&world, &d, &|_| 0.0, &elevation, 0.0);
    assert!(clim.from_stage);
    let hydro = crate::hydro::HydroParams::default();
    let params = |budget: bool, closed: bool| {
        let mut p = SurfaceParams {
            runoff_budget: budget,
            closed_basins: closed,
            ..SurfaceParams::default()
        };
        p.flexure.water_by_basin = false;
        p
    };
    let weights = |p: &SurfaceParams| -> Vec<f32> {
        let water = step_water(&world, &clim, &hydro, p);
        assert_eq!(
            water.is_some(),
            p.runoff_budget || p.closed_basins,
            "the budget is taken when the stream power or the lakes read it"
        );
        (0..4)
            .map(|c| runoff_weight(p, &clim, water.as_deref(), c))
            .collect()
    };
    let off = weights(&params(false, false));
    let lakes_only = weights(&params(false, true));
    let on = weights(&params(true, false));
    eprintln!("weights off {off:?}, the lakes' budget only {lakes_only:?}, on {on:?}");
    // Off: a mean below freezing runs off all of P (0.3 m over the reference 0.5 m).
    for (c, w) in off.iter().enumerate().take(3) {
        assert!((w - 0.6).abs() < 1.0e-6, "cell {c}: {w}");
    }
    assert_eq!(lakes_only, off, "the lakes' budget alone leaves the weight");
    // On: ground that never thaws, and the polar desert's year as the stage gave it, at the floor.
    assert_eq!(on[0], 0.05);
    assert_eq!(on[1], 0.05, "the polar desert's months taken as given");
    assert!(
        on[2] > 0.05 && on[2] < 0.9 * off[2],
        "the cold land's summer: {}",
        on[2]
    );
    assert!(
        (on[3] - off[3]).abs() < 1.0e-6,
        "frost-free land is the same: {} against {}",
        on[3],
        off[3]
    );
}

// ledger: ERO-221
/// The switch reaches the stream power through the surface's step itself (the runoff law's
/// re-review, finding 5): one step of the coupled pipeline (the plates, the climate stage, the
/// surface) with `surface.runoff_budget` on. The step's measurement
/// (`Totals::runoff_weight_km2_myr`) sums over its land the weight its stream power used and the
/// snapshot's own; the planet's frozen and frosty ground makes the used one smaller (a step that
/// read the snapshot's runoff again, or the budget's water not at all, gives the two the same).
#[test]
fn one_surface_step_takes_the_budget_s_weight() {
    let mut v: serde_json::Value =
        serde_json::from_str(&pipeline_json(3000, 2.0, 5)).expect("the pipeline's preset");
    v["surface"]["runoff_budget"] = serde_json::json!(true);
    let dir = temp_dir("runoff-budget-step");
    let s = session(&v.to_string(), &dir);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let t = world
        .surface
        .as_ref()
        .expect("the surface ran")
        .totals
        .clone();
    let [land, used, own, high, high_used, high_own] = t.runoff_weight_km2_myr;
    eprintln!(
        "one step, {} steps: land {land:.4e} km2 Myr, used weight {:.3}, own {:.3}; above 2 km {high:.4e}, {:.3} and {:.3}",
        t.steps,
        used / land,
        own / land,
        high_used / high.max(1.0e-30),
        high_own / high.max(1.0e-30)
    );
    assert!(t.steps >= 1 && land > 0.0, "{t:?}");
    assert!(used > 0.0 && used < own, "used {used}, own {own}");
    let _ = std::fs::remove_dir_all(&dir);
}

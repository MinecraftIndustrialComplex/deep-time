// SPDX-License-Identifier: LGPL-3.0-or-later
//! The P1-07 checks (plan §10.3 P1-07): on the coupled plates + surface + structure pipeline, the
//! baked horizon stacks are ordered top-down, every horizon is at least one block thick after
//! scaling and every class is valid; the deposit engine's history carries the trap structure and
//! the host fractions; plutons become bodies; a resumed run is the same run.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::*;
use crate::checkpoint::Checkpoint;
use crate::deposits::fields::FieldHistory;
use crate::params::planet_builtin;
use crate::preset::{Preset, with_timeline};
use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};

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
    // Plenty of plutons on the small test planet.
    v["structure"] = serde_json::json!({"pluton_rate_km2_myr": 2e-3});
    // These tests are about the structure stage; the deposit stage after it has its own
    // (`deposits::recorded_tests`).
    v.as_object_mut().unwrap().remove("deposits");
    // The young landscape and L3 amplification (P1-06) have their own tests.
    v.as_object_mut().unwrap().remove("young");
    v.as_object_mut().unwrap().remove("amplify");
    v.to_string()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-structure-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn session(json: &str, dir: &Path, threads: usize) -> Session {
    Session::create(
        json,
        "",
        SessionOptions {
            out_dir: dir.to_path_buf(),
            threads,
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
fn the_pipeline_bakes_ordered_block_thick_valid_stacks() {
    let json = pipeline_json(3000, 60.0, 7);
    let dir = temp_dir("bake");
    let s = session(&json, &dir, 2);
    assert_eq!(
        s.stage_names(),
        ["plates", "climate", "surface", "structure"]
    );
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let preset = match &Preset::from_json(&json).unwrap().body {
        crate::preset::PresetBody::Planet(p) => (**p).clone(),
        _ => unreachable!(),
    };

    // The acceptance: ordered top-down, ≥ one block thick after scaling, classes valid.
    let h = bake_horizons::bake(&world, &preset).unwrap();
    h.check().unwrap();
    assert_eq!(h.k, 32);
    assert!(h.mean_horizons() > 2.0, "{}", h.mean_horizons());
    // The stack's top is the surface, and the layer round-trips through the format.
    let src = CellSource::new(&world, &preset).unwrap();
    for c in (0..h.cells()).step_by(97) {
        let want = (src.surface_y(c, &preset.scale).clamp(-351.0, 639.0) * 16.0).round();
        assert!((f64::from(h.cell(c).0[0]) - want).abs() <= 1.0, "cell {c}");
    }
    let layer = h.to_layer(deeptime_format::Codec::Raw).unwrap();
    let got = layer.horizon(5, 7, 1);
    let c = 7 * h.grid.w as usize + 5;
    assert_eq!(
        (got.top, got.class, got.key),
        (
            h.tops[c * 32 + 1],
            h.classes[c * 32 + 1],
            h.keys[c * 32 + 1]
        )
    );

    // Many classes appear: sediments, basement, metamorphic and mantle rocks.
    let mut seen = [false; 59];
    for &k in &h.classes {
        if k != bake_horizons::NO_CLASS {
            seen[usize::from(k)] = true;
        }
    }
    let used = seen.iter().filter(|&&x| x).count();
    assert!(used >= 12, "{used} classes in the stacks");

    // The deposit engine's history: every built-in type finds every field it reads.
    let hist = world.history.as_ref().expect("frames");
    let mut names = hist.names();
    // The regional fields the deposit engine derives on reading (`deposits::regional`).
    names.extend(crate::deposits::regional::derivable(&names));
    for t in crate::deposits::types::builtin().unwrap() {
        let missing: Vec<String> = t
            .fields()
            .into_iter()
            .filter(|f| !names.contains(f))
            .collect();
        assert!(missing.is_empty(), "{} lacks {missing:?}", t.id);
        for class in t.hosts.weights().unwrap().keys() {
            let f = format!("deeptime:host/{class}");
            assert!(names.contains(&f), "{f}");
        }
    }
    let frame = hist.frame(hist.frame_count() - 1).unwrap();
    let trap = &frame.fields[TRAP_FIELD];
    assert!(trap.iter().all(|&x| (0.0..=1.0 + 1e-6).contains(&x)));
    let hosts: Vec<&Vec<f32>> = frame
        .fields
        .iter()
        .filter(|(k, _)| k.starts_with(crate::deposits::fields::HOST_PREFIX))
        .map(|(_, v)| v)
        .collect();
    for c in 0..h.cells() {
        let sum: f32 = hosts.iter().map(|v| v[c]).sum();
        assert!(sum <= 1.0 + 0.02, "cell {c}: host fractions sum to {sum}");
    }

    // Plutons: emplaced, and the survivors are bodies of their class.
    let note: serde_json::Value = serde_json::from_str(&world.notes[STAGE_NAME]).unwrap();
    assert!(note["plutons"]["emplaced"].as_u64().unwrap() > 0, "{note}");
    let bodies = note["plutons"]["bodies"].as_u64().unwrap();
    assert_eq!(world.bodies.len() as u64, bodies);
    for b in &world.bodies {
        assert_eq!(b.role, "pluton");
        let class = b.rock_class.as_deref().unwrap();
        assert!(
            ["granodiorite", "granite", "diorite"].contains(&class),
            "{class}"
        );
        b.validate().unwrap();
    }
    assert_eq!(note["horizons"]["check"], "ok");
    // Keyframe layers exist with valid values.
    let rock = &world.fields[field::ROCK].data;
    assert!(rock.iter().all(|&k| k >= 0.0 && (k as usize) < 59));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_resumed_run_is_the_same_run() {
    let json = pipeline_json(2000, 30.0, 3);
    let dir = temp_dir("straight");
    let s = session(&json, &dir, 2);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let dir2 = temp_dir("resumed");
    let s2 = session(&json, &dir2, 2);
    for _ in 0..7 {
        assert_eq!(s2.advance(Duration::ZERO).unwrap(), AdvanceStatus::Running);
    }
    let ck = s2.checkpoint().unwrap();
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
    assert_eq!(
        got.surface.as_ref().unwrap().log,
        world.surface.as_ref().unwrap().log
    );
    assert_eq!(got.bodies, world.bodies);
    assert_eq!(got.history, world.history);
    assert_eq!(got.fields[field::ROCK].data, world.fields[field::ROCK].data);
    for d in [dir, dir2] {
        let _ = std::fs::remove_dir_all(d);
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The live map (`deeptime_core::live`) changes nothing in the simulation: the same planet run
//! once with nobody watching and once with the satellite view drawn live after every step ends
//! with the same world, byte for byte, and the same keyframes (the same grid, the same pixels).
//! The live frames fall between the keyframes, and none of them is kept or written.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use deeptime_core::live::LivePacing;
use deeptime_core::session::{AdvanceStatus, NoGauge, Session, SessionOptions};
use deeptime_core::world::WorldState;
use serde_json::{Value, json};

/// The overlap test's small coupled planet (L1 256 × 82, 60 Myr, a keyframe every 20 Myr).
fn preset() -> String {
    let json = deeptime_core::preset::with_timeline(
        deeptime_core::params::planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(60.0),
    )
    .unwrap();
    let mut v: Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = json!(3000);
    v["resolution"]["l1_r"] = json!(64);
    v["keyframes"]["size"] = json!([32, 16]);
    v["keyframes"]["every_myr"] = json!(20.0);
    v["seed"] = json!(7);
    v["surface"]["frame_every_myr"] = json!(6.0);
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

/// Every keyframe: its info and the pixels of each layer.
type Keyframes = Vec<(String, Vec<Vec<u8>>)>;

struct Run {
    world: WorldState,
    keyframes: Keyframes,
    /// Files in `keyframes/`.
    files: usize,
    /// The live frames seen between `advance` calls: (seq, t, layer).
    live: Vec<(u64, f64, String)>,
    live_drawn: u64,
}

fn files(dir: &Path) -> usize {
    std::fs::read_dir(dir).map_or(0, |d| d.count())
}

fn run(json: &str, watch: bool, name: &str) -> Run {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("deeptime-live-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let s = Session::create(
        json,
        "",
        SessionOptions {
            out_dir: dir.clone(),
            threads: 3,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap();
    s.set_live_pacing(LivePacing::EVERY_STEP);
    let mut live: Vec<(u64, f64, String)> = Vec::new();
    loop {
        if watch {
            s.watch_live(Some("satellite"));
        }
        // One group step a call.
        match s.advance(Duration::from_millis(1)).unwrap() {
            AdvanceStatus::Done => break,
            AdvanceStatus::Cancelled => panic!("cancelled"),
            _ => {}
        }
        if let Some(f) = s.live_frame()
            && live.last().is_none_or(|l| l.0 != f.seq)
        {
            assert_eq!(f.rgba.len(), (f.w * f.h * 4) as usize);
            live.push((f.seq, f.t, f.layer.clone()));
        }
    }
    let mut keyframes = Keyframes::new();
    for n in 0..s.keyframe_count() {
        let info = s.keyframe_info(n).unwrap();
        let v: Value = serde_json::from_str(&info).unwrap();
        let (w, h) = (v["w"].as_u64().unwrap(), v["h"].as_u64().unwrap());
        let layers = v["layers"].as_array().unwrap().len() as u32;
        let pixels = (0..layers)
            .map(|k| {
                let mut px = vec![0u8; (w * h * 4) as usize];
                s.copy_keyframe(n, k, &mut px).unwrap();
                px
            })
            .collect();
        keyframes.push((info, pixels));
    }
    let world = s.with_world(Clone::clone).unwrap();
    let live_drawn = s.live_stats().frames;
    let files = files(&dir.join("keyframes"));
    drop(s);
    let _ = std::fs::remove_dir_all(&dir);
    Run {
        world,
        keyframes,
        files,
        live,
        live_drawn,
    }
}

#[test]
fn the_live_map_changes_nothing() {
    let json = preset();
    let a = run(&json, false, "off");
    let b = run(&json, true, "on");

    // Nobody watched: nothing drawn.
    assert_eq!(a.live_drawn, 0);
    assert!(a.live.is_empty());

    // Watched: frames of the satellite view between the keyframes, none of them kept.
    let kf_times: Vec<f64> = a
        .keyframes
        .iter()
        .map(|(info, _)| {
            serde_json::from_str::<Value>(info).unwrap()["t"]
                .as_f64()
                .unwrap()
        })
        .collect();
    assert!(kf_times.len() >= 3, "keyframes {kf_times:?}");
    assert!(b.live.len() >= 10, "live frames seen: {:?}", b.live);
    assert!(b.live.iter().all(|l| l.2 == "satellite"));
    let between = b.live.iter().filter(|l| !kf_times.contains(&l.1)).count();
    assert!(
        between >= 10,
        "live frames at times no keyframe has: {between} of {:?} (keyframes {kf_times:?})",
        b.live
    );
    assert!(
        b.live
            .windows(2)
            .all(|w| w[0].0 < w[1].0 && w[0].1 < w[1].1),
        "newer frames, later times"
    );
    assert_eq!(a.files, b.files, "no live frame written");

    // The keyframes: the same grid, the same pixels.
    assert_eq!(a.keyframes.len(), b.keyframes.len());
    for ((ia, pa), (ib, pb)) in a.keyframes.iter().zip(&b.keyframes) {
        assert_eq!(ia, ib);
        assert!(pa == pb, "keyframe {ia}: its pixels");
    }

    // The world, byte for byte.
    let (a, b) = (a.world, b.world);
    assert_eq!(a.t_years, b.t_years);
    assert_eq!(a.sites, b.sites);
    assert_eq!(a.bodies, b.bodies);
    assert_eq!(a.events, b.events);
    assert_eq!(
        a.notes.keys().collect::<Vec<_>>(),
        b.notes.keys().collect::<Vec<_>>()
    );
    for (k, text) in &a.notes {
        assert_eq!(without_times(text), without_times(&b.notes[k]), "note {k}");
    }
    assert!(a.history == b.history, "the recorded history");
    let body = |w: &WorldState| {
        let mut w = w.clone();
        w.notes.clear();
        postcard::to_stdvec(&w).unwrap()
    };
    assert!(body(&a) == body(&b), "the world's bytes");
}

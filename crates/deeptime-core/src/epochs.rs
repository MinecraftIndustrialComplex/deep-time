// SPDX-License-Identifier: LGPL-3.0-or-later
//! Epoch checkpoints (docs/design/playability-steering.md §3): the states of a run's
//! end-independent trajectory, kept so a player can bake the world at any keyframe, stop a run
//! early or continue it past its planned length without running it again from the start.
//!
//! A run's world after plate step k is the same for every stop whose own tail starts after k
//! (`PlanetPreset::last_free_step`). While it runs, a session with epochs on writes a checkpoint
//! (`checkpoint.rs`'s format) after the coupled group's step k for every k that is the last free
//! step of one of its keyframes, or of its own stop, into `<out>/epochs/step-<k>.dtcp`, and lists
//! them in `<out>/epochs/index.json`. A resumed run ([`crate::session::Session::resume_epoch`])
//! takes the latest one at or before its own last free step whose preset key matches.
//!
//! The key is the CRC-32 of the canonical preset with the keys that only shape a run's tail taken
//! out (`timeline.stop_myr`, the `steering` section, `climate.present_*`): a checkpoint serves
//! every preset that differs from its run only in them.
//!
//! The folder is not world data: no manifest lists it, and the game deletes it when a staging
//! folder becomes a world.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use deeptime_format::fsutil::write_atomic;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::preset::Preset;

/// The epochs folder inside a run's folder.
pub const EPOCHS_DIR: &str = "epochs";
/// Its index.
pub const INDEX_FILE: &str = "index.json";
/// The index's schema (bump when its shape changes; an index of another schema is ignored).
pub const INDEX_SCHEMA: u32 = 1;

/// One checkpoint of the store.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpochEntry {
    /// The plate step after which it was taken.
    pub step: u64,
    /// World time, years.
    pub t_years: f64,
    /// The file, relative to the epochs folder.
    pub file: String,
    pub bytes: u64,
    /// Seconds it took to write.
    pub write_seconds: f64,
}

/// `epochs/index.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpochIndex {
    pub schema: u32,
    /// The preset key every entry was taken under.
    pub key: String,
    /// The canonical preset of the run that last wrote here (the CLI's `deeptime resume` starts
    /// from it).
    pub preset: Value,
    /// The content bundle it ran with.
    pub content: String,
    /// In step order.
    pub entries: Vec<EpochEntry>,
}

impl EpochIndex {
    /// The epochs folder of a run's folder.
    pub fn dir(out: &Path) -> PathBuf {
        out.join(EPOCHS_DIR)
    }

    /// Reads a run's index; `None` when there is none (or of another schema).
    pub fn read(out: &Path) -> Result<Option<EpochIndex>, CoreError> {
        let path = Self::dir(out).join(INDEX_FILE);
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(CoreError::io(path, e)),
        };
        let v: Value = serde_json::from_slice(&bytes)
            .map_err(|e| CoreError::InvalidState(format!("{}: {e}", path.display())))?;
        if v["schema"].as_u64() != Some(u64::from(INDEX_SCHEMA)) {
            return Ok(None);
        }
        serde_json::from_value(v)
            .map(Some)
            .map_err(|e| CoreError::InvalidState(format!("{}: {e}", path.display())))
    }

    /// Writes the index atomically.
    pub fn write(&self, out: &Path) -> Result<(), CoreError> {
        let dir = Self::dir(out);
        std::fs::create_dir_all(&dir).map_err(|e| CoreError::io(&dir, e))?;
        let mut json = serde_json::to_vec_pretty(self)
            .map_err(|e| CoreError::InvalidState(format!("epoch index: {e}")))?;
        json.push(b'\n');
        write_atomic(&dir.join(INDEX_FILE), &json)?;
        Ok(())
    }

    /// The latest entry at or before step `need` (`None`: none, start over).
    pub fn choose(&self, need: u64) -> Option<&EpochEntry> {
        self.entries.iter().filter(|e| e.step <= need).max_by_key(|e| e.step)
    }

    /// Adds (or replaces) an entry, keeping step order.
    pub fn put(&mut self, e: EpochEntry) {
        self.entries.retain(|x| x.step != e.step);
        self.entries.push(e);
        self.entries.sort_by_key(|x| x.step);
    }

    /// The bytes the store takes.
    pub fn bytes(&self) -> u64 {
        self.entries.iter().map(|e| e.bytes).sum()
    }
}

/// The file name of step `k`'s checkpoint.
pub fn file_name(step: u64) -> String {
    format!("step-{step}.dtcp")
}

/// The preset key: CRC-32 of the canonical preset without the keys that only shape the tail.
pub fn key(preset: &Preset) -> String {
    let mut v = preset.to_json();
    if let Some(t) = v.get_mut("timeline").and_then(Value::as_object_mut) {
        t.remove("stop_myr");
    }
    if let Some(o) = v.as_object_mut() {
        o.remove("steering");
    }
    if let Some(c) = v.get_mut("climate").and_then(Value::as_object_mut) {
        c.retain(|k, _| !k.starts_with("present_"));
    }
    format!("crc32:{:08x}", crc32fast::hash(v.to_string().as_bytes()))
}

/// The plate steps after which a run writes epoch checkpoints: the last free step of every point
/// of the grid `every_years` (the keyframe interval in the game: the step whose end first reaches
/// the grid, as the session draws keyframes) and of the run's own stop. Step 0 is left out.
pub fn plan(p: &PlanetPreset, every_years: Option<f64>) -> BTreeSet<u64> {
    let total = p.plate_steps();
    let mut stops: Vec<u64> = vec![total];
    if let Some(every) = every_years.filter(|e| e.is_finite() && *e > 0.0) {
        // The session's keyframe grid: from the run's start, a frame after the step whose end
        // first reaches the next multiple of the interval.
        let origin = p.t_start_years();
        let mut next = origin;
        for k in 1..=total {
            let t = p.t_at_step(k);
            if t >= next {
                stops.push(k);
                next = origin + (((t - origin) / every).floor() + 1.0) * every;
            }
        }
    }
    let dt = p.dt_myr();
    stops
        .into_iter()
        .map(|s| {
            if s == total {
                p.last_free_step()
            } else {
                p.with_stop(Some(s as f64 * dt)).last_free_step()
            }
        })
        .filter(|&k| k > 0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::{PresetBody, with_timeline};

    fn planet(json: &str) -> PlanetPreset {
        match Preset::from_json(json).unwrap().body {
            PresetBody::Planet(p) => *p,
            _ => panic!("planet"),
        }
    }

    fn quick(length: f64, steering: bool) -> String {
        let json = with_timeline(
            crate::params::planet_builtin("pipeline_quick_16k").unwrap(),
            Some("scattered_continents"),
            Some(length),
        )
        .unwrap();
        let mut v: Value = serde_json::from_str(&json).unwrap();
        if steering {
            v["steering"] = serde_json::json!({"enabled": true, "tail_fraction": 0.1});
        } else {
            v.as_object_mut().unwrap().remove("steering");
        }
        v.to_string()
    }

    #[test]
    fn the_tail_is_the_window_or_the_steering_and_the_last_step() {
        // 500 Myr in 2-Myr steps; the 3-Myr window covers steps 249 (498 Myr, t = −2 Myr) and 250.
        let p = planet(&quick(500.0, false));
        assert_eq!(p.plate_steps(), 250);
        assert_eq!(p.last_free_step(), 248);
        // Steering over the last 10 %: 50 Myr, steps 226 … 250 (t > −50 Myr).
        let s = planet(&quick(500.0, true));
        assert_eq!(s.last_free_step(), 225);
        // Stopped at 120 Myr: the tail is 12 Myr (steps 55 … 60).
        let early = s.with_stop(Some(120.0));
        assert_eq!(early.plate_steps(), 60);
        assert_eq!(early.t_end_years(), -380.0e6);
        assert_eq!(early.last_free_step(), 54);
        // Continued to 700: the end is 200 Myr after the present; 70 Myr of tail.
        let cont = s.with_stop(Some(700.0));
        assert_eq!(cont.plate_steps(), 350);
        assert_eq!(cont.t_end_years(), 200.0e6);
        assert_eq!(cont.last_free_step(), 315);
        // Without a stop the end is the present, exactly.
        assert_eq!(s.t_end_years(), 0.0);
        assert_eq!(s.with_stop(Some(500.0)).t_end_years(), 0.0);
    }

    #[test]
    fn the_plan_has_one_checkpoint_per_keyframe_and_one_for_the_stop() {
        let p = planet(&quick(500.0, true));
        let plan = plan(&p, Some(50.0e6));
        assert_eq!(super::plan(&p, None).len(), 1, "without a grid: the stop's alone");
        // Keyframes at 0 (after step 1: none earlier is free), 50, 100, … 500 Myr; each one's
        // checkpoint is 10 % of its length earlier: 45 Myr = step 22, 90 = 45, …; the stop's is
        // step 225 (450 Myr).
        assert!(plan.contains(&22) && plan.contains(&45) && plan.contains(&225));
        assert!(plan.iter().all(|&k| k <= p.last_free_step()));
        assert_eq!(plan.len(), 10);
        // Every planned step is free for the stop it serves.
        for &k in &plan {
            assert!(!p.with_stop(Some(500.0)).step_depends_on_end(k));
        }
    }

    #[test]
    fn the_key_ignores_what_shapes_only_the_tail() {
        let a = Preset::from_json(&quick(500.0, true)).unwrap();
        let mut v: Value = serde_json::from_str(&quick(500.0, false)).unwrap();
        v["timeline"]["stop_myr"] = serde_json::json!(700.0);
        v["climate"]["present_climate"] = serde_json::json!("natural");
        let b = Preset::from_json(&v.to_string()).unwrap();
        assert_eq!(key(&a), key(&b));
        v["seed"] = serde_json::json!(99);
        let c = Preset::from_json(&v.to_string()).unwrap();
        assert_ne!(key(&a), key(&c));
        let idx = EpochIndex {
            schema: INDEX_SCHEMA,
            key: key(&a),
            preset: Value::Null,
            content: String::new(),
            entries: [5, 22, 45]
                .iter()
                .map(|&s| EpochEntry {
                    step: s,
                    t_years: 0.0,
                    file: file_name(s),
                    bytes: 1,
                    write_seconds: 0.0,
                })
                .collect(),
        };
        assert_eq!(idx.choose(44).map(|e| e.step), Some(22));
        assert_eq!(idx.choose(45).map(|e| e.step), Some(45));
        assert_eq!(idx.choose(4), None);
    }
}

#[cfg(test)]
#[path = "epochs_tests.rs"]
mod session_tests;

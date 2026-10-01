// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime hydro --world-data <dir> [--set hydro.KEY=V]…` (P1-22): re-runs the bake's water part
//! (`deeptime_core::bake::water`: lakes, `water_level`, the river network) on a folder a planet run
//! already baked, with the folder's own preset (`preset.json`) and any `--set` overrides, and
//! rewrites `water_level`, `rivers/rivers.dtrv`, `notes.bake.water` and the manifest. The water
//! part reads only what the bake wrote before it, so this gives the bytes a run with the same
//! preset writes; it is how the parameters are tuned without re-running the history.
//!
//! The spawn candidates stay as they were (they need the run's sites; a lake may now cover one).
//! The placers settle again into the new channels (`deposits::placer_channels::settle_folder`).

use std::path::Path;

use deeptime_core::bake::{Out, water};
use deeptime_core::preset::{Preset, PresetBody};
use deeptime_format::Manifest;

use crate::run::{CliResult, apply_overrides, read_text};

/// Runs it; returns the summary line.
pub fn run(world_data: &Path, set: &[String]) -> CliResult<String> {
    let json = read_text(&world_data.join("preset.json"), "preset")?;
    let json = apply_overrides(&json, set)?;
    let preset = Preset::from_json(&json)?;
    let PresetBody::Planet(p) = &preset.body else {
        return Err("the folder's preset is not a planet preset: nothing to drain".into());
    };
    let mut m = Manifest::read(world_data)?;
    let mut out = Out {
        dir: world_data,
        m: &mut m,
        bytes: 0,
        layers: Vec::new(),
    };
    let w = water::bake(p, &mut out)?;
    let line = water::summary_line(&w.note);
    if let Some(bake) = m.notes.get_mut("bake")
        && bake.is_object()
    {
        bake["water"] = w.note;
    }
    if !set.is_empty() {
        std::fs::write(world_data.join("preset.json"), preset.to_json().to_string())
            .map_err(|e| format!("preset.json: {e}"))?;
        let entry = deeptime_format::FileEntry::of(world_data, "preset.json")?;
        m.files.retain(|f| f.path != "preset.json");
        m.files.push(entry);
    }
    m.write(world_data)?;
    // The placers settle into the new channels; the pass reads the folder as the game does, so
    // the manifest is written before it and again after it.
    let placers = deeptime_core::deposits::placer_channels::settle_folder(world_data, p, &mut m)?;
    m.write(world_data)?;
    let line = format!(
        "{line}; placers: {} in channels, {} benches of {}",
        placers["in_channels"].as_u64().unwrap_or(0),
        placers["benches"].as_u64().unwrap_or(0),
        placers["placers"].as_u64().unwrap_or(0)
    );
    Ok(line)
}

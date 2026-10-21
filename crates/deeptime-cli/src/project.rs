// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime project --world-data <lambert dir> --out <dir> [--set scale.KEY=V]…` (SW-1): copies a
//! baked Lambert world folder and runs the bake's projection pass on the copy
//! (`deeptime_core::bake::project::project_folder`), then the guaranteed deposits' anchor and its
//! top-up (`deeptime_core::deposits::anchor::anchor_folder`), as `Session::finalize_write` does at
//! the end of a run whose preset says `scale.projection` `petroff_guyou`. It is how the pass's
//! settings (λc, the footprint) are tuned without re-running the history. The copy is not the
//! folder such a run writes: the Lambert run's deposit stage counted the essentials' regions, and
//! measured its inland margin, on the Lambert frame (DEP-220, DEP-222), so its guaranteed
//! deposits lie where a Lambert world wants them; the anchor moves them under the projected
//! world's ground, and the top-up adds a source to each land region of the projected world that
//! lacks one.

use std::path::Path;

use deeptime_core::bake::project;
use deeptime_core::preset::{Preset, PresetBody};
use deeptime_format::Manifest;
use deeptime_format::projection::{ProjectionKind, WorldProjection};
use serde_json::Value;

use crate::run::{CliResult, apply_overrides, read_text};

fn copy_tree(from: &Path, to: &Path) -> CliResult<()> {
    std::fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    for entry in std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let entry = entry?;
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if entry.file_type()?.is_dir() {
            copy_tree(&src, &dst)?;
        } else {
            std::fs::copy(&src, &dst).map_err(|e| format!("{}: {e}", src.display()))?;
        }
    }
    Ok(())
}

/// Runs it; returns the pass's record (`notes.bake.projection`).
pub fn run(world_data: &Path, out: &Path, set: &[String]) -> CliResult<Value> {
    let m = Manifest::read(world_data)?;
    if !matches!(m.world_projection(), WorldProjection::Lambert(_)) {
        return Err("the folder is not a Lambert world (the pass rewrites a Lambert bake)".into());
    }
    let json = read_text(&world_data.join("preset.json"), "preset")?;
    let mut sets = vec!["scale.projection=petroff_guyou".to_owned()];
    sets.extend(set.iter().cloned());
    let json = apply_overrides(&json, &sets)?;
    let preset = Preset::from_json(&json)?;
    let PresetBody::Planet(p) = &preset.body else {
        return Err("the folder's preset is not a planet preset".into());
    };
    if p.scale.projection != ProjectionKind::PetroffGuyou {
        return Err("--set scale.projection must stay petroff_guyou".into());
    }
    if out.exists() {
        std::fs::remove_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    copy_tree(world_data, out)?;
    let mut m = Manifest::read(out)?;
    let note = project::project_folder(out, p, &mut m)?;
    if let Some(b) = m.notes.get_mut(deeptime_core::bake::NOTES_KEY) {
        b["projection"] = note.clone();
    }
    // The copy's preset is the one it was projected with.
    std::fs::write(out.join("preset.json"), preset.to_json().to_string())
        .map_err(|e| format!("preset.json: {e}"))?;
    let entry = deeptime_format::FileEntry::of(out, "preset.json")?;
    m.files.retain(|f| f.path != "preset.json");
    m.files.push(entry);
    m.write(out)?;
    // The guaranteed deposits under the projected world's ground, as a run's finish does
    // (`Session::finalize_write`): the pass reads the written folder.
    let anchored = deeptime_core::deposits::anchor::anchor_folder(out, p, &mut m, None)?;
    if p.deposits.as_ref().is_some_and(|d| !d.essentials.is_empty()) {
        eprintln!(
            "deeptime project: the Lambert run's deposit stage placed the essentials on the Lambert \
             world's regions and coasts (DEP-220, DEP-222), not this world's; the top-up added {} \
             sources where this world's land regions lacked one",
            anchored["topped_up"]
        );
    }
    if let Some(b) = m.notes.get_mut(deeptime_core::bake::NOTES_KEY) {
        b["guaranteed"] = anchored;
    }
    m.write(out)?;
    Ok(note)
}

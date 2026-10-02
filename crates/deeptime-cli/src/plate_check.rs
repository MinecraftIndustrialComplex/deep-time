// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime plate-check`: the plate dynamics of finished runs against the target ranges of
//! `docs/design/plate-collisions-and-welding.md` §6 (`deeptime_core::plates::check`).
//!
//! A run is read from whichever holds its record (`notes.plates.dynamics`, written by the plates
//! stage, `deeptime_core::plates::dynamics`):
//! - a checkpoint file (`stage --out`, `run --checkpoints`): the record in the plate state, and the
//!   margins in the world's plates note;
//! - a world-data folder (`run --out`) or its `manifest.json`: the plates note of the manifest;
//! - a JSON file holding a plates note itself (`{"margins": […], "dynamics": {…}}`).
//!
//! Several `--run`s give the mean of the measures, as the design asserts its targets on seeds 1-4.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use deeptime_core::checkpoint::Checkpoint;
use deeptime_core::plates::check::{self, RunData, Targets};
use deeptime_core::plates::dynamics::Dynamics;
use deeptime_core::plates::stats::Margins;
use serde_json::Value;

use crate::run::CliResult;

pub struct Options<'a> {
    pub runs: &'a [PathBuf],
    pub json: Option<&'a Path>,
    pub targets: Option<&'a Path>,
}

/// The plates note's dynamics and margins out of its JSON.
fn from_note(note: &Value, what: &Path) -> CliResult<(Option<Dynamics>, Vec<Margins>)> {
    let dynamics = match note.get("dynamics") {
        Some(v) => Some(serde_json::from_value(v.clone()).map_err(|e| {
            format!(
                "{}: notes.plates.dynamics does not read: {e}",
                what.display()
            )
        })?),
        None => None,
    };
    let margins = note["margins"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default();
    Ok((dynamics, margins))
}

/// Reads one run.
pub fn load(path: &Path) -> CliResult<RunData> {
    let name = path.display().to_string();
    let missing = || {
        format!(
            "{name}: no plate dynamics record (notes.plates.dynamics): the run predates it or was \
             made with it switched off"
        )
    };
    let (dynamics, margins) = if path.is_dir() || path.extension().is_some_and(|e| e == "json") {
        let file = if path.is_dir() {
            path.join("manifest.json")
        } else {
            path.to_owned()
        };
        let text =
            std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let v: Value =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", file.display()))?;
        let note = if v.get("notes").is_some() {
            &v["notes"]["plates"]
        } else {
            &v
        };
        from_note(note, &file)?
    } else {
        let ck = Checkpoint::read(path)?;
        let note: Value = ck
            .world
            .notes
            .get(deeptime_core::plates::STAGE_NAME)
            .and_then(|t| serde_json::from_str(t).ok())
            .unwrap_or(Value::Null);
        let (from_note_dyn, margins) = from_note(&note, path)?;
        // The state's record is always current; the note's is as of the last margins record.
        let live = ck.world.plates.as_ref().and_then(|p| p.dynamics.clone());
        (live.or(from_note_dyn), margins)
    };
    let dynamics = dynamics.ok_or_else(missing)?;
    Ok(RunData {
        name,
        dynamics,
        margins,
    })
}

pub fn run(o: &Options<'_>) -> CliResult<ExitCode> {
    if o.runs.is_empty() {
        return Err(
            "give at least one --run <checkpoint, world-data folder or manifest.json>".into(),
        );
    }
    let mut targets = Targets::builtin();
    if let Some(file) = o.targets {
        let text = std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
        targets.apply_json(&text)?;
    }
    let runs = o
        .runs
        .iter()
        .map(|p| load(p))
        .collect::<CliResult<Vec<_>>>()?;
    let rows = check::evaluate(&runs, &targets);
    let names: Vec<String> = runs.iter().map(|r| r.name.clone()).collect();
    println!(
        "deeptime plate-check: {} run(s); the first has {} steps of {} Myr",
        runs.len(),
        runs[0].dynamics.steps.len(),
        runs[0].dynamics.dt_myr
    );
    print!("{}", check::render(&rows));
    if let Some(out) = o.json {
        let v = check::to_json(&rows, &names);
        std::fs::write(
            out,
            serde_json::to_string_pretty(&v).expect("a table serialises"),
        )
        .map_err(|e| format!("{}: {e}", out.display()))?;
    }
    let ranged = rows
        .iter()
        .filter(|r| r.lo.is_some() || r.hi.is_some())
        .count();
    let bad = rows.iter().filter(|r| r.status.failed()).count();
    println!(
        "deeptime plate-check: {} of {ranged} targets met{}",
        ranged - bad,
        if bad == 0 { "" } else { "; FAILED" }
    );
    Ok(if bad == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

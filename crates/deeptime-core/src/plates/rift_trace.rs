// SPDX-License-Identifier: LGPL-3.0-or-later
//! The rift trace (the runoff lane, 2026-10-31; measurement only, off by default): what each rift
//! does, for the question continental extension's second arm left open
//! (docs/design/continental-extension.md, "The second arm (ext-cum2)": the stretching drowned the
//! continents; is it the wide mode's reach, and does a rift narrow as its crust thins?).
//!
//! [`ENV`] names a folder. Each plate step with a rift appends one row per pair of plates to
//! `steps.tsv` ([`STEP_FIELDS`]: the boundary, the opening's area and what the columns took of
//! it, and per side the mode, the edge columns the mode test reads, how far the search reached
//! and how many columns thinned how far out), and each resampling one row per fill to `lays.tsv`
//! ([`LAY_FIELDS`]: the gap between the pair, how many of its samples took a column, break-up
//! when fewer than all, and what waited). The records are `extension::RiftRecord` and
//! `extension::LayRecord`, which the stretching and the laying fill whether the trace is on or
//! not; no rule reads them, so the world a run leaves is the same with the trace on or off (on
//! the pool, the trace's runs against the arm's own runs of the same seeds).
//! `tools/review/extension/rift_trace.py` reads the two files per pair: the opening before
//! break-up, the mode, the area thinned.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::extension::{LayRecord, RiftRecord, SideRecord};

/// The folder the trace writes into; unset or empty, the trace is off.
pub const ENV: &str = "DEEPTIME_RIFT_TRACE";

/// The folder [`ENV`] names, read once.
pub fn env_dir() -> Option<PathBuf> {
    static DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    DIR.get_or_init(|| {
        std::env::var_os(ENV)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    })
    .clone()
}

/// A side's columns in a `steps.tsv` row, after the prefix `a_` or `b_`.
const SIDE_FIELDS: [&str; 13] = [
    "mode", "edges", "crust_km", "lith_km", "moho_c", "reach_km", "found", "weighted", "thinned",
    "at_limit", "far_km", "mean_km", "given_m",
];

/// `steps.tsv`'s columns before the two sides'.
pub const STEP_FIELDS: [&str; 9] = [
    "t_myr",
    "step",
    "a",
    "b",
    "length_km",
    "speed_mm_yr",
    "asked_km2",
    "taken_km2",
    "stretched_m",
];

/// `lays.tsv`'s columns.
pub const LAY_FIELDS: [&str; 15] = [
    "t_myr",
    "a",
    "b",
    "total_m",
    "source_m",
    "gap",
    "columns",
    "column_m",
    "even_m",
    "crystalline_m",
    "floor_m",
    "waits_m",
    "fallback_m",
    "waited",
    "breakup",
];

/// `steps.tsv`'s header line.
pub fn step_header() -> String {
    let mut f: Vec<String> = STEP_FIELDS.iter().map(|s| (*s).to_owned()).collect();
    for side in ["a", "b"] {
        f.extend(SIDE_FIELDS.iter().map(|s| format!("{side}_{s}")));
    }
    f.join("\t")
}

fn side_cells(s: &SideRecord) -> [String; 13] {
    [
        s.mode.to_string(),
        s.edges.to_string(),
        format!("{:.3}", s.crust_km),
        format!("{:.3}", s.lith_km),
        format!("{:.1}", s.moho_c),
        format!("{:.1}", s.reach_km),
        s.found.to_string(),
        s.weighted.to_string(),
        s.thinned.to_string(),
        s.at_limit.to_string(),
        format!("{:.1}", s.far_km),
        format!("{:.1}", s.mean_km),
        format!("{:.1}", s.given_m),
    ]
}

/// A step's row for `r`.
pub fn step_row(t_myr: f64, step: u64, r: &RiftRecord) -> String {
    let mut f: Vec<String> = vec![
        format!("{t_myr:.4}"),
        step.to_string(),
        r.a.to_string(),
        r.b.to_string(),
        format!("{:.2}", r.length_km),
        format!("{:.4}", r.speed_mm_yr),
        format!("{:.3}", r.asked_km2),
        format!("{:.3}", r.taken_km2),
        format!("{:.1}", r.stretched_m),
    ];
    for s in &r.sides {
        f.extend(side_cells(s));
    }
    f.join("\t")
}

/// A resampling's row for `r` (`breakup` 1 when fewer gap samples took a column than the gap
/// holds).
pub fn lay_row(t_myr: f64, r: &LayRecord) -> String {
    [
        format!("{t_myr:.4}"),
        r.a.to_string(),
        r.b.to_string(),
        format!("{:.1}", r.total_m),
        format!("{:.1}", r.source_m),
        r.gap.to_string(),
        r.columns.to_string(),
        format!("{:.1}", r.column_m),
        format!("{:.1}", r.even_m),
        format!("{:.1}", r.crystalline_m),
        format!("{:.1}", r.floor_m),
        format!("{:.1}", r.waits_m),
        format!("{:.1}", r.fallback_m),
        r.waited.to_string(),
        u8::from(r.gap > 0 && r.columns < r.gap).to_string(),
    ]
    .join("\t")
}

/// Appends a step's rows to `dir/steps.tsv`.
pub fn write_steps(dir: &Path, t_myr: f64, step: u64, records: &[RiftRecord]) {
    let lines: Vec<String> = records.iter().map(|r| step_row(t_myr, step, r)).collect();
    append(dir, "steps.tsv", &step_header(), &lines);
}

/// Appends a resampling's rows to `dir/lays.tsv`.
pub fn write_lays(dir: &Path, t_myr: f64, records: &[LayRecord]) {
    let lines: Vec<String> = records.iter().map(|r| lay_row(t_myr, r)).collect();
    append(dir, "lays.tsv", &LAY_FIELDS.join("\t"), &lines);
}

/// Appends `lines` to `dir/name`, the header first if the file is new (a failure is reported and
/// ignored).
fn append(dir: &Path, name: &str, header: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    let path = dir.join(name);
    let r = std::fs::create_dir_all(dir).and_then(|()| {
        let new = !path.exists();
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        let mut buf = String::new();
        if new {
            buf.push_str(header);
            buf.push('\n');
        }
        for l in lines {
            buf.push_str(l);
            buf.push('\n');
        }
        f.write_all(buf.as_bytes())
    });
    if let Err(e) = r {
        eprintln!("rift_trace: cannot write {}: {e}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The rows have as many cells as their headers.
    #[test]
    fn the_rift_trace_rows_match_their_headers() {
        let cells = |s: &str| s.split('\t').count();
        let step = step_row(2.0, 1, &RiftRecord::default());
        assert_eq!(cells(&step), cells(&step_header()));
        assert_eq!(cells(&step_header()), STEP_FIELDS.len() + 2 * SIDE_FIELDS.len());
        let lay = lay_row(2.0, &LayRecord::default());
        assert_eq!(cells(&lay), LAY_FIELDS.len());
        let broke = LayRecord {
            gap: 4,
            columns: 1,
            ..LayRecord::default()
        };
        assert!(lay_row(2.0, &broke).ends_with("\t1"));
    }
}

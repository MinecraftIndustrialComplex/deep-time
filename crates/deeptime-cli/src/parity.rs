// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime parity`: the Java↔Rust column parity fixture (plan §9.2, P1-08). Picks columns of a
//! world-data folder deterministically, evaluates each with the column evaluator
//! (`deeptime_format::ColumnWorld::sample`, what `sampleColumn` and `inspect-column` answer) and
//! writes them as `parity/columns.json`; `ColumnParityTest` evaluates the same columns through the
//! Java reader and asserts every one is equal, and the real-planet GameTest compares placed blocks
//! with them.
//!
//! The columns: 60 % anywhere on the planet with x over two laps ([−C, C), so half of them need
//! the wrap), 10 % within 40 blocks of the seam, 10 % near the poles and past them (z clamps), and
//! 20 % inside the boxes of random bodies (so the bodies and their zones are exercised); a world
//! without bodies gets random columns instead.

use std::path::Path;

use deeptime_core::synth::SplitMix;
use deeptime_format::{ColumnWorld, FormatError, fsutil::write_atomic};
use serde_json::{Value, json};

/// The fixture's default size (plan §9.2: 500 columns).
pub const DEFAULT_COLUMNS: usize = 500;
/// The default seed of the column choice.
pub const DEFAULT_SEED: u64 = 0x5EED_0C01_0B0E;

/// The columns to sample.
pub fn pick(w: &ColumnWorld, n: usize, seed: u64) -> Vec<(i64, i64)> {
    let c = w.c;
    let mut rng = SplitMix(seed);
    let mut out = Vec::with_capacity(n);
    let (seam, poles, bodies) = (n / 10, n / 10, if w.bodies.is_empty() { 0 } else { n / 5 });
    for _ in 0..seam {
        let dx = rng.below(80) as i64 - 40;
        let z = rng.below((c / 2) as u64) as i64 - c / 4;
        out.push((c / 2 + dx, z));
    }
    for k in 0..poles {
        let x = rng.below(c as u64) as i64 - c / 2;
        let d = rng.below(96) as i64 - 32;
        let z = if k % 2 == 0 {
            -c / 4 + d
        } else {
            c / 4 - 1 - d
        };
        out.push((x, z));
    }
    for _ in 0..bodies {
        let b = &w.bodies[rng.below(w.bodies.len() as u64) as usize];
        let [lo, hi] = b.aabb;
        let x =
            i64::from(lo[0]) + rng.below((i64::from(hi[0]) - i64::from(lo[0]) + 1) as u64) as i64;
        let z =
            i64::from(lo[2]) + rng.below((i64::from(hi[2]) - i64::from(lo[2]) + 1) as u64) as i64;
        out.push((x, z));
    }
    while out.len() < n {
        let x = rng.below(2 * c as u64) as i64 - c;
        let z = rng.below((c / 2) as u64) as i64 - c / 4;
        out.push((x, z));
    }
    out
}

/// Writes the fixture; returns (columns, columns through a body).
pub fn run(world: &Path, out: &Path, n: usize, seed: u64) -> Result<(usize, usize), FormatError> {
    let w = ColumnWorld::open(world)?;
    let columns: Vec<Value> = pick(&w, n, seed)
        .into_iter()
        .map(|(x, z)| w.sample(x, z).to_json())
        .collect();
    let through = columns
        .iter()
        .filter(|c| c["bodies"].as_array().is_some_and(|b| !b.is_empty()))
        .count();
    let doc = json!({
        "format_version": w.manifest.format_version,
        "world": w.manifest.source,
        "circumference": w.c,
        "min_y": w.min_y,
        "max_y": w.max_y,
        "seed": seed.to_string(),
        "classes": w.classes,
        "columns": columns,
    });
    let bytes = serde_json::to_vec(&doc).map_err(|e| FormatError::Invalid(e.to_string()))?;
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| FormatError::Io {
            path: parent.to_owned(),
            source: e,
        })?;
    }
    write_atomic(out, &bytes)?;
    Ok((n, through))
}

/// Writes the climate parity fixture (`parity/climate.json`, P1-25) when the world has the twelve
/// months: the climate evaluator (`deeptime_format::ClimateWorld::sample`, what Deep Time's
/// climate API answers) at `n` blocks — the columns [`pick`] chooses, a third each at the terrain's
/// top, at sea level and at a random height in the build range. Returns the blocks written, or
/// None for a world without the months (nothing written).
pub fn run_climate(
    world: &Path,
    out: &Path,
    n: usize,
    seed: u64,
) -> Result<Option<usize>, FormatError> {
    let Some(cw) = deeptime_format::ClimateWorld::open(world)? else {
        return Ok(None);
    };
    let w = ColumnWorld::open(world)?;
    let mut rng = SplitMix(seed ^ 0xC11A_7E00);
    let sea = w.manifest.scale.sea_level_y;
    let span = (w.max_y - w.min_y + 1) as u64;
    let samples: Vec<Value> = pick(&w, n, seed)
        .into_iter()
        .enumerate()
        .map(|(k, (x, z))| {
            let y = match k % 3 {
                0 => w.sample(x, z).top_y + 1,
                1 => sea,
                _ => w.min_y + rng.below(span) as i32,
            };
            cw.sample(x, y, z).to_json()
        })
        .collect();
    let doc = json!({
        "format_version": w.manifest.format_version,
        "world": w.manifest.source,
        "circumference": w.c,
        "lapse_c_per_km": cw.lapse_c_per_km(),
        "seed": seed.to_string(),
        "samples": samples,
    });
    let bytes = serde_json::to_vec(&doc).map_err(|e| FormatError::Invalid(e.to_string()))?;
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| FormatError::Io {
            path: parent.to_owned(),
            source: e,
        })?;
    }
    write_atomic(out, &bytes)?;
    Ok(Some(n))
}

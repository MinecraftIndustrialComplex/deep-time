// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime strata-check`: how many rock columns a geologist would object to
//! (`deeptime_core::structure::realism`: metamorphic grade inversions, thin metamorphic slivers,
//! plutonic rock lying in the pile like a bed), measured where the question is asked:
//!
//! - `--world-data <folder>`: the columns the game builds (the column rule over the baked stacks,
//!   under the terrain), at `--columns` random blocks, and the stored stacks themselves, cell by
//!   cell (what the bake wrote, before the rule blends sixteen of them);
//! - `--checkpoint <file>`: the stacks the bake would write from a checkpoint's world, before the
//!   lateral smoothing and after it (which stage makes a finding).
//!
//! A world with folds and faults (P2-01) also gets `structure`: how many of the sampled columns
//! the structures change, how far they move a bed, how many columns meet a bed twice (a thrust's
//! repetition), how many hold an intrusion, that no column is malformed (tops descend, neighbours
//! differ, the first top is the column's own), and the findings of the same columns read without
//! the structures (`plain_columns`), so what the structures add to the census is on the page.
//!
//! `--column X,Z` (repeatable) prints those columns' stacks instead of the census.

use std::path::Path;

use deeptime_core::checkpoint::Checkpoint;
use deeptime_core::preset::{Preset, PresetBody};
use deeptime_core::structure::bake_horizons::{self, Horizons};
use deeptime_core::structure::realism::{self, Census, Findings, Limits, Rules};
use deeptime_core::structure::{CellSource, columns};
use deeptime_core::synth::{EVERY_CLASS, SplitMix};
use deeptime_format::{ColumnWorld, RasterRead};
use serde_json::{Value, json};

use crate::run::CliResult;

pub struct Options<'a> {
    pub world_data: Option<&'a Path>,
    pub checkpoint: Option<&'a Path>,
    pub columns: usize,
    pub seed: u64,
    pub limits: Limits,
    pub at: &'a [(i64, i64)],
}

pub fn run(o: &Options<'_>) -> CliResult<Value> {
    match (o.world_data, o.checkpoint) {
        (Some(dir), None) => world_data(dir, o),
        (None, Some(file)) => checkpoint(file, o),
        _ => Err("give exactly one of --world-data <folder> and --checkpoint <file>".into()),
    }
}

fn findings_json(col: &[realism::Visible], f: &Findings, rules: &Rules) -> Value {
    let name = |k: u8| rules.names.get(usize::from(k)).map_or("?", |s| s.as_str());
    json!({
        "stack": col.iter().map(|&(top, blocks, k)| json!([name(k), top, blocks])).collect::<Vec<_>>(),
        "inversions": f.inversions.iter().map(|&(up, low, top)| json!([name(up), name(low), top])).collect::<Vec<_>>(),
        "slivers": f.slivers.iter().map(|&(top, blocks, k)| json!([name(k), top, blocks])).collect::<Vec<_>>(),
        "plutonic_sheets": f.sheets.iter().map(|&(top, blocks, k)| json!([name(k), top, blocks])).collect::<Vec<_>>(),
        "plutonic_with_a_floor": f.floored.iter().map(|&(top, blocks, k)| json!([name(k), top, blocks])).collect::<Vec<_>>(),
    })
}

fn world_data(dir: &Path, o: &Options<'_>) -> CliResult<Value> {
    let w = ColumnWorld::open(dir)?;
    let layer = w
        .strata()
        .ok_or_else(|| format!("{}: no strata/horizons layer", dir.display()))?;
    let mut deform = deeptime_format::structure::DeformScratch::default();
    let rules = Rules::for_classes(&w.classes);
    let sea = w.manifest.scale.sea_level_y;
    let (mut stack, mut raw, mut col, mut f) =
        (Vec::new(), Vec::new(), Vec::new(), Findings::default());
    let mut column = |x: i64, z: i64, col: &mut Vec<realism::Visible>, f: &mut Findings| -> i32 {
        w.stack(x, z, &mut deform, &mut stack);
        raw.clear();
        raw.extend(stack.iter().map(|&(top, class, _)| (top, class)));
        let ground = w.top_y(x, z);
        realism::visible(&raw, ground, w.min_y, col);
        realism::check(col, &rules, &o.limits, f);
        ground
    };
    if !o.at.is_empty() {
        let mut out = Vec::new();
        for &(x, z) in o.at {
            let ground = column(x, z, &mut col, &mut f);
            let mut v = findings_json(&col, &f, &rules);
            v["x"] = json!(x);
            v["z"] = json!(z);
            v["ground_y"] = json!(ground);
            out.push(v);
        }
        return Ok(json!({ "columns": out }));
    }
    let (mut all, mut land) = (Census::default(), Census::default());
    let mut rng = SplitMix(o.seed);
    let mut picks = Vec::with_capacity(o.columns);
    let zp = w.manifest.world_projection().z_range().1 as i64;
    for _ in 0..o.columns {
        let x = rng.below(w.c as u64) as i64 - w.c / 2;
        let z = rng.below(2 * zp as u64) as i64 - zp;
        picks.push((x, z));
        let ground = column(x, z, &mut col, &mut f);
        all.add(&col, &f, &rules);
        if ground >= sea {
            land.add(&col, &f, &rules);
        }
    }
    let structure = structure_census(&w, &picks, &rules, &o.limits);
    // The stored stacks, each under its own first top (which the bake raised over the terrain).
    let d = layer.desc().clone();
    let k = d.stack_depth.unwrap_or(0);
    let mut cells = Census::default();
    for j in 0..d.grid.h {
        for i in 0..d.grid.w {
            raw.clear();
            raw.extend(
                (0..k)
                    .map(|h| layer.horizon(i, j, h))
                    .take_while(|h| h.is_used())
                    .map(|h| {
                        (
                            (f64::from(h.top) * d.scale + d.offset).floor() as i32,
                            h.class,
                        )
                    }),
            );
            let Some(&(ground, _)) = raw.first() else {
                continue;
            };
            realism::visible(&raw, ground, w.min_y, &mut col);
            realism::check(&col, &rules, &o.limits, &mut f);
            cells.add(&col, &f, &rules);
        }
    }
    Ok(json!({
        "world_data": dir.display().to_string(),
        "limits": {"sliver_blocks": o.limits.sliver_blocks, "sheet_blocks": o.limits.sheet_blocks},
        "seed": o.seed,
        "columns": all.to_json(),
        "land_columns": land.to_json(),
        "stored_cells": cells.to_json(),
        "structure": structure,
    }))
}

/// What the world's folds, faults and intrusions do to the columns at `picks` (`Null` for a
/// world without any).
fn structure_census(
    w: &ColumnWorld,
    picks: &[(i64, i64)],
    rules: &Rules,
    limits: &realism::Limits,
) -> Value {
    use deeptime_format::structure::Shape;
    let Some(d) = w.structure() else {
        return Value::Null;
    };
    let intrusive: std::collections::BTreeSet<u8> = d
        .structure
        .events
        .iter()
        .filter_map(|e| match e.shape {
            Shape::Intrusion { class, .. } => Some(class),
            _ => None,
        })
        .collect();
    let mut scratch = deeptime_format::structure::DeformScratch::default();
    let (mut plain, mut bent) = (Vec::new(), Vec::new());
    let (mut raw, mut col, mut f) = (Vec::new(), Vec::new(), Findings::default());
    let mut plain_census = Census::default();
    let (mut changed, mut land, mut land_changed) = (0u64, 0u64, 0u64);
    let (mut repeated, mut intruded, mut malformed, mut cover_kept) = (0u64, 0u64, 0u64, 0u64);
    let mut max_offset = 0i32;
    let mut offsets: Vec<i32> = Vec::new();
    for &(x, z) in picks {
        w.plain_stack(x, z, &mut plain);
        w.stack(x, z, &mut scratch, &mut bent);
        let ground = w.top_y(x, z);
        raw.clear();
        raw.extend(plain.iter().map(|&(top, class, _)| (top, class)));
        realism::visible(&raw, ground, w.min_y, &mut col);
        realism::check(&col, rules, limits, &mut f);
        plain_census.add(&col, &f, rules);
        let is_land = ground >= w.manifest.scale.sea_level_y;
        land += u64::from(is_land);
        // Well-formed whatever the events: tops descend, neighbours differ in class, and the
        // first top is the column's own.
        let ok = bent.windows(2).all(|p| p[0].0 > p[1].0 && p[0].1 != p[1].1)
            && bent.first().map(|h| h.0) == plain.first().map(|h| h.0);
        malformed += u64::from(!ok);
        if bent == plain {
            continue;
        }
        changed += 1;
        land_changed += u64::from(is_land);
        // The youngest rock is where it was (structures are older than what covers them).
        cover_kept += u64::from(bent.first() == plain.first());
        // A bed met twice going down where the plain column meets it once (a thrust carried
        // it over itself). (Intertonguing facies repeat a bed in the plain column too.)
        let times = |col: &[(i32, u8, u16)], h: &(i32, u8, u16)| {
            col.iter().filter(|g| g.1 == h.1 && g.2 == h.2).count()
        };
        let twice = bent
            .iter()
            .any(|h| !intrusive.contains(&h.1) && times(&bent, h) > 1 && times(&plain, h) == 1);
        repeated += u64::from(twice);
        intruded +=
            u64::from(bent.iter().any(|h| {
                intrusive.contains(&h.1) && !plain.iter().any(|g| g.1 == h.1 && g.2 == h.2)
            }));
        // How far the beds both columns hold once have moved.
        let mut worst = 0i32;
        for h in plain.iter().skip(1) {
            if times(&plain, h) == 1
                && times(&bent, h) == 1
                && let Some(g) = bent.iter().find(|g| g.2 == h.2 && g.1 == h.1)
            {
                worst = worst.max((g.0 - h.0).abs());
            }
        }
        max_offset = max_offset.max(worst);
        offsets.push(worst);
    }
    offsets.sort_unstable();
    let n = picks.len().max(1) as f64;
    let pct = |q: f64| {
        offsets
            .get(((offsets.len() as f64 - 1.0) * q) as usize)
            .copied()
            .unwrap_or(0)
    };
    json!({
        "events": d.structure.events.len(),
        "margin_blocks": d.structure.margin,
        "changed_share": changed as f64 / n,
        "land_changed_share": land_changed as f64 / land.max(1) as f64,
        "malformed_columns": malformed,
        "cover_kept_share": cover_kept as f64 / changed.max(1) as f64,
        "bed_repeated_columns": repeated,
        "intruded_columns": intruded,
        "bed_offset_blocks": {"median": pct(0.5), "p90": pct(0.9), "max": max_offset},
        "plain_columns": plain_census.to_json(),
    })
}

fn checkpoint(file: &Path, o: &Options<'_>) -> CliResult<Value> {
    let ck = Checkpoint::read(file)?;
    let preset = match Preset::from_json(&ck.preset_json)?.body {
        PresetBody::Planet(p) => *p,
        _ => return Err(format!("{}: not a planet run", file.display()).into()),
    };
    let rules = Rules::builtin();
    let smoothed = bake_horizons::bake(&ck.world, &preset)?;
    let mut unsmoothed = preset.clone();
    let mut sp = unsmoothed.structure.clone().unwrap_or_default();
    sp.strata_smoothing_cells = 0.0;
    unsmoothed.structure = Some(sp.clone());
    let raw = bake_horizons::bake(&ck.world, &unsmoothed)?;
    if !o.at.is_empty() {
        let src = CellSource::new(&ck.world, &preset)?;
        let tables = columns::Tables::get();
        let g = &raw.grid;
        let mut out = Vec::new();
        for &(x, z) in o.at {
            let c0 = i64::from(g.w) * i64::from(g.r);
            let wx = (x + c0 / 2).rem_euclid(c0) - c0 / 2;
            let i = ((wx - g.origin_block_x) / i64::from(g.r)).clamp(0, i64::from(g.w) - 1);
            let j = ((z - g.origin_block_z) / i64::from(g.r)).clamp(0, i64::from(g.h) - 1);
            let c = (j * i64::from(g.w) + i) as usize;
            let cell = src.cell(c);
            let mut units = Vec::new();
            columns::build(&cell, &tables, &sp, src.now_myr, 12_000.0, &mut units);
            let stack = |hz: &Horizons| -> Value {
                let (tops, classes, keys) = hz.cell(c);
                (0..tops.len())
                    .map(|h| {
                        json!([
                            EVERY_CLASS[usize::from(classes[h])],
                            f64::from(tops[h]) / 16.0,
                            keys[h][0],
                            keys[h][1]
                        ])
                    })
                    .collect()
            };
            out.push(json!({
                "x": x, "z": z, "cell": [i, j],
                "surface_y": src.surface_y(c, &preset.scale),
                "crust_type": cell.crust_type, "regime": cell.regime,
                "orogeny": cell.orogeny, "orogeny_age_myr": if cell.orogeny_age.is_finite() { json!(cell.orogeny_age) } else { Value::Null },
                "gradient_c_km": cell.gradient_c_m * 1000.0,
                "regolith_m": cell.regolith_m,
                "top_active": cell.top_active,
                "pools_m": cell.pools,
                "log_bottom_first": cell.log.iter().map(|l| json!([EVERY_CLASS[usize::from(l.class)], l.age_myr, l.thickness_m])).collect::<Vec<_>>(),
                "units": units.iter().map(|u| json!([EVERY_CLASS[usize::from(u.class)], u.thickness_m, format!("{:?}", u.source), u.key[0], u.key[1]])).collect::<Vec<_>>(),
                "stack_before_smoothing": stack(&raw),
                "stack_smoothed": stack(&smoothed),
            }));
        }
        return Ok(json!({ "cells": out }));
    }
    Ok(json!({
        "checkpoint": file.display().to_string(),
        "limits": {"sliver_blocks": o.limits.sliver_blocks, "sheet_blocks": o.limits.sheet_blocks},
        "smoothing_sigma_cells": preset.structure.clone().unwrap_or_default().strata_smoothing_cells,
        "cells_before_smoothing": realism::census_of_cells(&raw, &rules, &o.limits, 1).to_json(),
        "cells_smoothed": realism::census_of_cells(&smoothed, &rules, &o.limits, 1).to_json(),
    }))
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! `rock-census`: how much of a baked planet's land is which rock class (P1-28).
//!
//! Samples the world data's column evaluator ([`deeptime_format::ColumnWorld::sample`], the twin of
//! the game's strata pass, bodies included) on a square lattice over the dry land and counts, per
//! rock class, the blocks of
//!
//! - **the top rock**: the class at the column's top block, and the class four blocks under it
//!   (what a player digging through a few blocks of soil meets first);
//! - **the top N blocks** from the terrain's top block down (default 64): the rock a player and the
//!   ore in it actually see;
//! - **the whole column** from the top block to the floor of the world.
//!
//! A tool for the class → block mapping: the shares of a class table's blocks follow by adding up
//! the classes that resolve to one block (`stone`, `deepslate`), so a mapping is judged on the
//! planet's real rock before the game generates a chunk. It counts classes, not placed blocks:
//! soil, ore, caves and water are the generator's, not the world data's.

use std::collections::BTreeMap;
use std::path::Path;

use deeptime_format::ColumnWorld;
use serde_json::{Value, json};

use crate::{Error, Result};

/// Columns below this depth under the top block count as "under the soil" for the second top-rock
/// histogram.
pub const UNDER_SOIL: i32 = 4;

#[derive(Default)]
struct Tally {
    top: BTreeMap<String, u64>,
    under_soil: BTreeMap<String, u64>,
    near_surface: BTreeMap<String, u64>,
    whole: BTreeMap<String, u64>,
    /// A few columns (x, z) of each top-rock class, spread over the sample, for picking review boxes.
    examples: BTreeMap<String, Vec<(i64, i64)>>,
}

fn class_name(c: &Option<String>) -> String {
    c.clone().unwrap_or_else(|| "unclassified_crust".into())
}

impl Tally {
    /// Adds one column: its rock runs `(top, bottom, class)` from the top block `top_y` down.
    fn add(&mut self, rock: &[(i32, i32, Option<String>)], top_y: i32, top_n: i32) {
        for (i, (hi, lo, class)) in rock.iter().enumerate() {
            let name = class_name(class);
            let len = u64::from((hi - lo + 1).max(0) as u32);
            if i == 0 {
                *self.top.entry(name.clone()).or_default() += 1;
            }
            if (*lo..=*hi).contains(&(top_y - UNDER_SOIL)) {
                *self.under_soil.entry(name.clone()).or_default() += 1;
            }
            *self.whole.entry(name.clone()).or_default() += len;
            let near = (*hi).min(top_y) - (*lo).max(top_y - top_n + 1) + 1;
            if near > 0 {
                *self.near_surface.entry(name).or_default() += near as u64;
            }
        }
    }
}

/// How far down a player digs when looking for stone (blocks under the top block).
pub const DIG_BLOCKS: i32 = 32;

/// The census of the dry land of `dir` on a lattice `step` blocks apart, `top_n` blocks deep. With
/// `stone_classes` (the classes a pack gives plain stone or deepslate) it also reports how much of
/// the land has none of them within `reach` blocks, at the surface and within `DIG_BLOCKS` below it.
pub fn run(
    dir: &Path,
    step: i64,
    top_n: i32,
    stone_classes: &[String],
    reach: i64,
) -> Result<Value> {
    let w = ColumnWorld::open(dir).map_err(|e| Error(e.to_string()))?;
    let c = w.c;
    let sea = w.manifest.scale.sea_level_y;
    let step = step.max(1);
    let mut t = Tally::default();
    let (mut land, mut sampled) = (0u64, 0u64);
    let (cols, rows) = (
        ((c / step) as usize).max(1),
        (((c / 2) / step) as usize).max(1),
    );
    // Per lattice cell: None = not dry land, else (stone class at the top block, within DIG_BLOCKS).
    let mut cells: Vec<Option<(bool, bool)>> = vec![None; cols * rows];
    let is_stone =
        |class: &Option<String>| class.as_ref().is_some_and(|c| stone_classes.contains(c));
    let mut z = -c / 4 + step / 2;
    while z < c / 4 {
        let mut x = -c / 2 + step / 2;
        while x < c / 2 {
            sampled += 1;
            let col = w.sample(x, z);
            // Dry land: the terrain above the sea and no lake, river or sea over it.
            if col.top_y > sea && col.hydro.kind.name() == "none" {
                land += 1;
                let (i, j) = (((x + c / 2) / step) as usize, ((z + c / 4) / step) as usize);
                if i < cols && j < rows {
                    let top = col.rock.first().is_some_and(|r| is_stone(&r.2));
                    let dig = col.rock.iter().any(|(hi, lo, class)| {
                        is_stone(class) && *lo <= col.top_y && *hi >= col.top_y - DIG_BLOCKS + 1
                    });
                    cells[j * cols + i] = Some((top, dig));
                }
                t.add(&col.rock, col.top_y, top_n);
                // Every 4,000th dry-land column away from the poles, for review boxes that are
                // not picked by class (the per-class examples are biased towards rare rock).
                if land % 4000 == 1 && z.abs() < c / 4 * 4 / 5 {
                    t.examples.entry("*land*".into()).or_default().push((x, z));
                }
                if let Some((_, _, class)) = col.rock.first() {
                    let n = t.top[&class_name(class)];
                    let e = t.examples.entry(class_name(class)).or_default();
                    if e.len() < 4 && n % 211 == 1 {
                        e.push((x, z));
                    }
                }
            }
            x += step;
        }
        z += step;
    }
    let shares = |m: &BTreeMap<String, u64>| -> Value {
        let total: u64 = m.values().sum();
        let mut rows: Vec<(&String, &u64)> = m.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        json!({
            "total": total,
            "classes": rows.iter().map(|(k, v)| json!({
                "class": k,
                "count": v,
                "share": if total == 0 { 0.0 } else { **v as f64 / total as f64 },
            })).collect::<Vec<_>>(),
        })
    };
    let isolation = if stone_classes.is_empty() {
        Value::Null
    } else {
        // Share of dry land with no stone-class cell within `reach` blocks (wrapping in x).
        let r = (reach / step).max(0) as i64;
        let (mut none_top, mut none_dig, mut n) = (0u64, 0u64, 0u64);
        for j in 0..rows as i64 {
            for i in 0..cols as i64 {
                if cells[j as usize * cols + i as usize].is_none() {
                    continue;
                }
                n += 1;
                let (mut top, mut dig) = (false, false);
                for dj in -r..=r {
                    let jj = j + dj;
                    if jj < 0 || jj >= rows as i64 {
                        continue;
                    }
                    for di in -r..=r {
                        if di * di + dj * dj > r * r {
                            continue;
                        }
                        let ii = (i + di).rem_euclid(cols as i64);
                        if let Some((t, d)) = cells[jj as usize * cols + ii as usize] {
                            top |= t;
                            dig |= d;
                        }
                    }
                }
                none_top += u64::from(!top);
                none_dig += u64::from(!dig);
            }
        }
        json!({
            "stone_classes": stone_classes,
            "reach_blocks": reach,
            "dig_blocks": DIG_BLOCKS,
            "land_cells": n,
            "no_stone_at_the_surface": if n == 0 { 0.0 } else { none_top as f64 / n as f64 },
            "no_stone_within_dig_depth": if n == 0 { 0.0 } else { none_dig as f64 / n as f64 },
            "definition": "share of dry land cells that have no cell of a stone class (top block) within reach blocks, and none whose rock within dig_blocks of the top block is a stone class; a start there has to walk for stone",
        })
    };
    Ok(json!({
        "isolation": isolation,
        "world_data": dir.display().to_string(),
        "step_blocks": step,
        "top_blocks": top_n,
        "sea_level_y": sea,
        "columns_sampled": sampled,
        "land_columns": land,
        "land_share": if sampled == 0 { 0.0 } else { land as f64 / sampled as f64 },
        "top_rock": shares(&t.top),
        "under_soil_rock": shares(&t.under_soil),
        "top_n_blocks": shares(&t.near_surface),
        "whole_column": shares(&t.whole),
        "examples": t.examples,
        "definition": "dry land only (terrain top above sea level, no lake, river or sea); classes from the column evaluator (horizons by the column rule, bodies over them); top_rock = the class at the top block (columns), under_soil_rock = the class UNDER_SOIL (4) blocks below it (columns), top_n_blocks = blocks within top_blocks of the top block, whole_column = every block from the top to the floor",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_column_is_counted_by_its_top_rock_its_rock_under_the_soil_the_top_blocks_and_all() {
        let rock = vec![
            (100, 98, Some("shale".to_string())),
            (97, 90, Some("sandstone".to_string())),
            (89, -10, Some("gneiss".to_string())),
        ];
        let mut t = Tally::default();
        t.add(&rock, 100, 8);
        assert_eq!(t.top["shale"], 1);
        assert_eq!(t.top.len(), 1);
        // Four blocks under the top block (y 96) is sandstone.
        assert_eq!(t.under_soil.len(), 1);
        assert_eq!(t.under_soil["sandstone"], 1);
        // The top 8 blocks are y 93..=100: 3 of shale, 5 of sandstone.
        assert_eq!(t.near_surface["shale"], 3);
        assert_eq!(t.near_surface["sandstone"], 5);
        assert!(!t.near_surface.contains_key("gneiss"));
        // Every block of the column.
        assert_eq!(t.whole["shale"], 3);
        assert_eq!(t.whole["sandstone"], 8);
        assert_eq!(t.whole["gneiss"], 100);
    }

    #[test]
    fn a_column_without_a_class_is_the_unclassified_crust() {
        let mut t = Tally::default();
        t.add(&[(50, 0, None)], 50, 64);
        assert_eq!(t.top["unclassified_crust"], 1);
        assert_eq!(t.near_surface["unclassified_crust"], 51);
    }
}

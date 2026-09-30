// SPDX-License-Identifier: LGPL-3.0-or-later
//! `water-check` (P1-22): the water blocks of a generated area, checked the way the game would
//! move them. Every water block (`minecraft:water`, source or flowing) is
//!
//! - **floating** when the block under it is air (it would fall);
//! - **leaking** when a horizontal neighbour is air and the block under that neighbour is not water
//!   (it would spread over ground or pour into air). Flowing water resting on water does not spread
//!   sideways (vanilla's "water hole" rule), so only sources and flowing water on ground are asked;
//!   a neighbour of air over water is a **step** (a river's one-block step; at rest once the game
//!   has put its flowing ring there), counted apart.
//!
//! **Lakes**: every 4-connected body of columns whose highest water block is a source at the same
//! y, of at least `min_lake` columns and not touching the area's edge, is a lake; its rim is every
//! neighbouring column that holds no water, and the report gives each lake's surface y (the top
//! face of its highest water block) and its lowest rim's top block + 1, so "the surface equals its
//! lowest rim ±1 block" (plan P2 exit test) reads off directly. Water whose column's top is ice
//! (frozen lakes) is found under the ice.
//!
//! With the world data (`--world-data`), every finding is split by what Deep Time decided for its
//! column (`deeptime_format::water`): **ours** in a river or lake column (the only water the pass
//! places), **other** elsewhere (vanilla's springs, the aquifer's sea-level water in cave mouths,
//! features), and a lake must be a lake column there. A neighbour's ice counts as the water under it.

use std::collections::{HashMap, VecDeque};

use serde_json::{Value, json};

use deeptime_format::ColumnWorld;
use deeptime_format::water::WaterKind;

use crate::blocks;
use crate::world::{Area, World};

fn level_of(key: &str) -> Option<u32> {
    let i = key.find("level=")?;
    key[i + 6..]
        .split(|c: char| !c.is_ascii_digit())
        .next()?
        .parse()
        .ok()
}

/// Checks `area` (x1, z1 exclusive; the outermost ring of columns is only read as neighbours).
pub fn check(w: &World, area: Area, min_lake: usize, wd: Option<&ColumnWorld>) -> Value {
    let mut kinds: HashMap<(i32, i32), WaterKind> = HashMap::new();
    let mut kind_of = |x: i32, z: i32| -> Option<WaterKind> {
        let wd = wd?;
        Some(*kinds.entry((x, z)).or_insert_with(|| wd.hydro(i64::from(x), i64::from(z)).kind))
    };
    // Water the pass placed: in a river or lake column (it never puts water in a bank).
    let ours = |k: Option<WaterKind>| matches!(k, Some(WaterKind::River | WaterKind::Lake));
    // Ice over water is a frozen pool: water beside air over ice rests on it like on water.
    let is_ice = |id: u32| w.reg.name(id).ends_with("ice");
    let (mut floating_ours, mut leaks_ours) = (0u64, 0u64);
    let mut leaks_by_kind: std::collections::BTreeMap<&'static str, u64> = std::collections::BTreeMap::new();
    let is_water_block = |id: u32| {
        let s = &w.reg.states[id as usize];
        s.name == "minecraft:water"
    };
    let is_air = |id: u32| w.reg.flags(id) & blocks::AIR != 0;
    let holds_water = |id: u32| w.reg.flags(id) & blocks::WATER != 0;
    let (mut water, mut sources, mut floating, mut leaks, mut steps) =
        (0u64, 0u64, 0u64, 0u64, 0u64);
    let mut samples: Vec<Value> = Vec::new();
    // Per column: the top water block y (source), for the lake census.
    let mut top_water: HashMap<(i32, i32), i32> = HashMap::new();
    let mut ground_top: HashMap<(i32, i32), i32> = HashMap::new();
    let mut wet: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    for x in area.x0 + 1..area.x1 - 1 {
        for z in area.z0 + 1..area.z1 - 1 {
            let Some(c) = w.chunk(x, z) else { continue };
            let Some(top) = w.top(x, z) else { continue };
            if let Some(g) = w.terrain_top(x, z) {
                ground_top.insert((x, z), g);
            }
            let low = (top - 64).max(c.min_y());
            let mut first_water: Option<(i32, bool)> = None;
            for y in (low..=top).rev() {
                let id = w.block(x, y, z);
                if !is_water_block(id) {
                    // Air, plants and ice (a frozen surface) are looked through; ground ends the column.
                    let f = w.reg.flags(id);
                    if holds_water(id) && first_water.is_none() {
                        // Seagrass, kelp, anything waterlogged: the water's surface all the same.
                        first_water = Some((y, true));
                    }
                    if is_air(id)
                        || f & (blocks::FEATURE | blocks::WATER) != 0
                        || w.reg.name(id).ends_with("ice")
                    {
                        continue;
                    }
                    break;
                }
                let key = &w.reg.states[id as usize].key;
                let lvl = level_of(key).unwrap_or(0);
                let source = lvl == 0;
                water += 1;
                sources += u64::from(source);
                if first_water.is_none() {
                    first_water = Some((y, source));
                }
                let below = w.block(x, y - 1, z);
                if is_air(below) {
                    floating += 1;
                    let k = kind_of(x, z);
                    floating_ours += u64::from(ours(k));
                    if samples.len() < 40 {
                        samples.push(json!({"what": "floating", "at": [x, y, z], "state": key,
                                            "column": k.map(WaterKind::name)}));
                    }
                }
                if !source && holds_water(below) {
                    continue;
                }
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let n = w.block(x + dx, y, z + dz);
                    if !is_air(n) {
                        continue;
                    }
                    if w.chunk(x + dx, z + dz).is_none() {
                        continue;
                    }
                    let under = w.block(x + dx, y - 1, z + dz);
                    if holds_water(under) || is_ice(under) {
                        steps += 1;
                    } else {
                        leaks += 1;
                        let k = kind_of(x, z);
                        leaks_ours += u64::from(ours(k));
                        *leaks_by_kind.entry(k.map_or("?", WaterKind::name)).or_default() += 1;
                        if samples.len() < 40 {
                            samples.push(json!({"what": "leak", "at": [x, y, z], "state": key,
                                                "into": [x + dx, y, z + dz], "column": k.map(WaterKind::name)}));
                        }
                    }
                }
            }
            if first_water.is_some() {
                wet.insert((x, z));
            }
            if let Some((y, true)) = first_water
                && (wd.is_none() || kind_of(x, z) == Some(WaterKind::Lake))
            {
                top_water.insert((x, z), y);
            }
        }
    }
    // Lakes: bodies of equal top-water y.
    let mut seen: HashMap<(i32, i32), bool> = HashMap::new();
    let mut lakes: Vec<Value> = Vec::new();
    let mut keys: Vec<(i32, i32)> = top_water.keys().copied().collect();
    keys.sort();
    for start in keys {
        if seen.contains_key(&start) {
            continue;
        }
        let y0 = top_water[&start];
        let mut q = VecDeque::from([start]);
        seen.insert(start, true);
        let mut cols = Vec::new();
        let mut rim_min = i32::MAX;
        let mut rim_at = (0, 0);
        let mut edge = false;
        while let Some((x, z)) = q.pop_front() {
            cols.push((x, z));
            if x <= area.x0 + 1 || x >= area.x1 - 2 || z <= area.z0 + 1 || z >= area.z1 - 2 {
                edge = true;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (x + dx, z + dz);
                match top_water.get(&n) {
                    Some(&y) if y == y0 => {
                        if let std::collections::hash_map::Entry::Vacant(e) = seen.entry(n) {
                            e.insert(true);
                            q.push_back(n);
                        }
                    }
                    Some(_) => {}
                    // The rim: dry neighbours (a river leaving the lake is water, not rim).
                    None if !wet.contains(&n) => {
                        if let Some(&g) = ground_top.get(&n)
                            && g < rim_min
                        {
                            rim_min = g;
                            rim_at = n;
                        }
                    }
                    None => {}
                }
            }
        }
        if cols.len() >= min_lake && !edge {
            lakes.push(json!({
                "columns": cols.len(),
                "surface_y": y0 + 1,
                "lowest_rim_top": rim_min,
                "rim_minus_surface": i64::from(rim_min) + 1 - i64::from(y0 + 1),
                "lowest_rim_at": [rim_at.0, rim_at.1],
                "lowest_rim_blocks": (rim_min - 1..=y0 + 1)
                    .rev()
                    .map(|y| w.reg.name(w.block(rim_at.0, y, rim_at.1)).to_owned())
                    .collect::<Vec<_>>(),
                "at": cols[0],
            }));
        }
    }
    let within = lakes
        .iter()
        .filter(|l| {
            l["rim_minus_surface"]
                .as_i64()
                .is_some_and(|d| d.abs() <= 1)
        })
        .count();
    json!({
        "area": area.to_json(),
        "water_blocks": water,
        "sources": sources,
        "floating": floating,
        "leaks": leaks,
        "floating_ours": wd.map(|_| floating_ours),
        "leaks_ours": wd.map(|_| leaks_ours),
        "leaks_by_column": leaks_by_kind,
        "steps": steps,
        "lakes": lakes.len(),
        "lakes_within_1": within,
        "lake_list": lakes,
        "samples": samples,
    })
}

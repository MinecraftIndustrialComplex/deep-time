// SPDX-License-Identifier: LGPL-3.0-or-later
//! Which rock each ore block sits in (plan Q10: ore blocks per host rock, with and without
//! Excavated Variants).
//!
//! **The host** of an ore block is estimated from its neighbours that are terrain rock: not air,
//! fluid, a feature ([`crate::blocks::is_feature`]) or another ore. First the 8 neighbours in the
//! same layer (y): Deep Time's strata are horizontal, so the rock beside an ore block is the rock it
//! replaced, where the rock above and below may already be the next horizon (a 3-block coal seam's
//! middle layer has the seam's rock beside it and other rock above and below). If none of those is
//! rock, the 5 × 5 square of the layer, then the 26 neighbours in 3D. Within a ring the most common
//! block wins; ties go to the alphabetically first name, so the answer does not depend on the order
//! the neighbours are visited in. An ore block with no rock anywhere near has no host: `(none)`.
//!
//! **Pairs** (`--other`): two worlds generated from the same data (Q10: without and with EV) are
//! compared position by position: every position where either holds an ore block is counted as
//! (block in this world, block in the other, host estimated in this world). An EV block in the
//! other world names its stone exactly, which checks the estimate.

use std::collections::{BTreeMap, HashMap};

use crate::blocks::{self, Registry};
use crate::world::{Area, World};

/// The name counted for an ore block without a rock neighbour.
pub const NO_HOST: &str = "(none)";

/// The most common rock among `neighbours` (block state ids), ties to the first name; None when
/// none of them is rock.
pub fn host_of(reg: &Registry, neighbours: impl IntoIterator<Item = u32>) -> Option<String> {
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for id in neighbours {
        let f = reg.flags(id);
        if f & (blocks::AIR | blocks::FLUID | blocks::FEATURE | blocks::ORE) == 0 {
            *counts.entry(reg.name(id)).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(a.0)))
        .map(|(name, _)| name.to_owned())
}

/// The host of the block at (x, y, z) by the rings in the module docs.
pub fn host_at(w: &World, x: i32, y: i32, z: i32) -> Option<String> {
    let ring = |r: i32| {
        let mut v = Vec::new();
        for dx in -r..=r {
            for dz in -r..=r {
                if dx.abs().max(dz.abs()) == r {
                    v.push(w.block(x + dx, y, z + dz));
                }
            }
        }
        v
    };
    host_of(&w.reg, ring(1))
        .or_else(|| host_of(&w.reg, ring(2)))
        .or_else(|| {
            let mut v = Vec::with_capacity(26);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if (dx, dy, dz) != (0, 0, 0) {
                            v.push(w.block(x + dx, y + dy, z + dz));
                        }
                    }
                }
            }
            host_of(&w.reg, v)
        })
}

/// Excavated Variants' stone for one of its generated blocks: `excavated_variants:<ore ns>__<ore
/// path>__<stone ns>__<stone path>` → `<stone ns>:<stone path>` (the stone config's id, `/` written
/// as `_`), e.g. `excavated_variants:minecraft_granite`. None for any other block.
pub fn ev_stone(name: &str) -> Option<String> {
    let path = name.strip_prefix("excavated_variants:")?;
    let parts: Vec<&str> = path.split("__").collect();
    (parts.len() == 4).then(|| format!("{}:{}", parts[2], parts[3]))
}

/// One counted position: the block here, the block at the same position in the other world
/// (None without `--other`), the host estimated here.
pub type Key = (String, Option<String>, String);

/// What [`ore_hosts`] counts: every [`Key`], and per y the ore positions of this world and the
/// positions whose block differs in the other world (where a cut through the pair shows most).
#[derive(Default)]
pub struct Counts {
    pub pairs: BTreeMap<Key, u64>,
    pub by_y: BTreeMap<i32, [u64; 2]>,
}

/// Ore positions in `area` (every height, or `y_min..=y_max`) of `w`, and with `other` also the
/// positions where only the other world holds ore, counted by [`Key`].
pub fn ore_hosts(
    w: &World,
    other: Option<&World>,
    area: Area,
    y_range: Option<(i32, i32)>,
) -> Counts {
    let mut out = Counts::default();
    for (&(cx, cz), chunk) in &w.chunks {
        if !area.contains_chunk(cx, cz) {
            continue;
        }
        let (y0, y1) = y_range.unwrap_or((chunk.min_y(), chunk.max_y() - 1));
        for lx in 0..16 {
            for lz in 0..16 {
                let (x, z) = (cx * 16 + lx as i32, cz * 16 + lz as i32);
                if x < area.x0 || x >= area.x1 || z < area.z0 || z >= area.z1 {
                    continue;
                }
                for y in y0.max(chunk.min_y())..=y1.min(chunk.max_y() - 1) {
                    let id = chunk.get(lx, y, lz);
                    let here_ore = w.reg.flags(id) & blocks::ORE != 0;
                    let there = other.map(|o| {
                        let t = o.block(x, y, z);
                        (o.reg.name(t).to_owned(), o.reg.flags(t) & blocks::ORE != 0)
                    });
                    if !here_ore && !there.as_ref().is_some_and(|t| t.1) {
                        continue;
                    }
                    let host = host_at(w, x, y, z).unwrap_or_else(|| NO_HOST.to_owned());
                    let here = w.reg.name(id).to_owned();
                    let y_row = out.by_y.entry(y).or_default();
                    y_row[0] += u64::from(here_ore);
                    y_row[1] += u64::from(there.as_ref().is_some_and(|t| t.0 != here));
                    *out.pairs
                        .entry((here, there.map(|t| t.0), host))
                        .or_default() += 1;
                }
            }
        }
    }
    out
}

/// The JSON `deeptime-inspect ore-hosts` prints: every counted key, the ore blocks of this world
/// grouped by host, and (with `--other`) how many positions hold ore in only one of the two.
pub fn to_json(area: Area, all: &Counts) -> serde_json::Value {
    let counts = &all.pairs;
    let is_ore = |n: &str| blocks::classify(n) & blocks::ORE != 0;
    let mut by_host: BTreeMap<&str, BTreeMap<&str, u64>> = BTreeMap::new();
    let (mut ores, mut only_here, mut only_there) = (0u64, 0u64, 0u64);
    for ((here, there, host), n) in counts {
        if is_ore(here) {
            ores += n;
            *by_host.entry(host).or_default().entry(here).or_default() += n;
        }
        if let Some(t) = there {
            match (is_ore(here), is_ore(t)) {
                (true, false) => only_here += n,
                (false, true) => only_there += n,
                _ => {}
            }
        }
    }
    let list: Vec<serde_json::Value> = counts
        .iter()
        .map(|((here, there, host), n)| {
            serde_json::json!({"ore": here, "other": there, "host": host, "count": n,
                "material": blocks::ore_material(here).or_else(|| there.as_deref().and_then(blocks::ore_material)),
                "ev_stone": ev_stone(here).or_else(|| there.as_deref().and_then(ev_stone))})
        })
        .collect();
    serde_json::json!({
        "area": area.to_json(),
        "definition": "host = the most common rock (not air, fluid, feature or ore) among the 8 neighbours in the same layer, else the 5x5 square of the layer, else the 26 neighbours in 3D (ties: first by name)",
        "ore_blocks": ores,
        "ore_only_here": only_here,
        "ore_only_in_other": only_there,
        "pairs": list,
        "by_host": by_host,
        "by_y": all.by_y.iter().map(|(y, [o, c])| (y.to_string(), serde_json::json!({"ore": o, "differs": c})))
            .collect::<serde_json::Map<String, serde_json::Value>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_is_the_most_common_rock_neighbour() {
        let mut r = Registry::new();
        let granite = r.intern("minecraft:granite", &[]);
        let sandstone = r.intern("minecraft:sandstone", &[]);
        let ore = r.intern("minecraft:copper_ore", &[]);
        let water = r.intern("minecraft:water", &[]);
        let grass = r.intern("minecraft:short_grass", &[]);
        // Ore, water, grass and air never count, however many there are.
        let n = [
            ore, ore, ore, ore, water, water, water, grass, grass, 0, 0, 0, granite, granite,
            sandstone,
        ];
        assert_eq!(host_of(&r, n).as_deref(), Some("minecraft:granite"));
        // A tie goes to the first name.
        assert_eq!(
            host_of(&r, [sandstone, granite, sandstone, granite]).as_deref(),
            Some("minecraft:granite")
        );
        assert_eq!(host_of(&r, [ore, water, 0]), None);
    }

    #[test]
    fn ev_stone_of_a_variant() {
        assert_eq!(
            ev_stone("excavated_variants:excavated_variants__minecraft_copper_ore__excavated_variants__minecraft_granite")
                .as_deref(),
            Some("excavated_variants:minecraft_granite")
        );
        assert_eq!(
            ev_stone(
                "excavated_variants:excavated_variants__create_zinc_ore__deeptime__packed_mud"
            )
            .as_deref(),
            Some("deeptime:packed_mud")
        );
        assert_eq!(ev_stone("minecraft:copper_ore"), None);
    }
}

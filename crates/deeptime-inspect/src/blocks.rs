// SPDX-License-Identifier: LGPL-3.0-or-later
//! Block states met in a world, interned, and what the renders and checks need to know about
//! them. Everything here is decided from the block's id (and its `waterlogged` property): the tool
//! has no game registry, so the classes are name rules, listed below and in the crate docs.

use std::collections::HashMap;

/// No block (air, cave air, void air).
pub const AIR: u16 = 1;
/// A fluid block: water, lava, bubble column.
pub const FLUID: u16 = 1 << 1;
/// Holds water: water, bubble columns, underwater plants, anything `waterlogged=true`.
pub const WATER: u16 = 1 << 2;
/// Not terrain: vegetation, tree parts, snow layers, sheet ice, coral (see [`is_feature`]).
pub const FEATURE: u16 = 1 << 3;
/// An ore block (see [`ore_material`]).
pub const ORE: u16 = 1 << 4;
/// Material a surface rule lays over the rock (see [`is_surface_material`]).
pub const SURFACE: u16 = 1 << 5;
/// Lava (also [`FLUID`]).
pub const LAVA: u16 = 1 << 6;

/// Skipped when looking for the terrain surface.
pub const NOT_TERRAIN: u16 = AIR | FLUID | FEATURE;

/// One interned block state.
#[derive(Clone, Debug)]
pub struct State {
    /// `namespace:path`.
    pub name: String,
    /// `name[k=v,…]` with properties in file order, or just the name.
    pub key: String,
    pub flags: u16,
}

/// The block states of one world, interned; ids are indices.
#[derive(Default)]
pub struct Registry {
    pub states: Vec<State>,
    by_key: HashMap<String, u32>,
}

impl Registry {
    pub fn new() -> Registry {
        let mut r = Registry::default();
        r.intern("minecraft:air", &[]);
        r
    }

    /// The id of block `name` with `props`, adding it on first sight.
    pub fn intern(&mut self, name: &str, props: &[(String, String)]) -> u32 {
        let name = if name.contains(':') {
            name.to_owned()
        } else {
            format!("minecraft:{name}")
        };
        let key = if props.is_empty() {
            name.clone()
        } else {
            let p: Vec<String> = props.iter().map(|(k, v)| format!("{k}={v}")).collect();
            format!("{name}[{}]", p.join(","))
        };
        if let Some(&id) = self.by_key.get(&key) {
            return id;
        }
        let waterlogged = props.iter().any(|(k, v)| k == "waterlogged" && v == "true");
        let mut flags = classify(&name);
        if waterlogged {
            flags |= WATER;
        }
        let id = self.states.len() as u32;
        self.states.push(State {
            name,
            key: key.clone(),
            flags,
        });
        self.by_key.insert(key, id);
        id
    }

    pub fn flags(&self, id: u32) -> u16 {
        self.states[id as usize].flags
    }

    pub fn name(&self, id: u32) -> &str {
        &self.states[id as usize].name
    }
}

/// The path part of `namespace:path`.
pub fn path_of(name: &str) -> &str {
    name.split_once(':').map_or(name, |(_, p)| p)
}

/// The class flags of block `name` (`namespace:path`).
pub fn classify(name: &str) -> u16 {
    let path = path_of(name);
    let mut f = 0;
    match path {
        "air" | "cave_air" | "void_air" => f |= AIR,
        "water" | "bubble_column" => f |= FLUID | WATER,
        "lava" => f |= FLUID | LAVA,
        _ => {}
    }
    if matches!(path, "kelp" | "kelp_plant" | "seagrass" | "tall_seagrass") {
        f |= WATER;
    }
    if f & (AIR | FLUID) == 0 && is_feature(path) {
        f |= FEATURE;
    }
    if ore_material(name).is_some() {
        f |= ORE;
    }
    if is_surface_material(path) {
        f |= SURFACE;
    }
    f
}

/// Blocks that sit on the terrain rather than being it: vegetation of every kind, tree parts
/// (logs, wood, stems, leaves, roots, huge mushrooms), coral and sea pickles, snow layers, sheet
/// ice, bee nests, pumpkins and melons, pointed dripstone, moss carpets. Rules on the path, so
/// modded trees and flowers that follow vanilla naming (`*_log`, `*_leaves`, `*_sapling`, …) are
/// covered too.
pub fn is_feature(path: &str) -> bool {
    const SUFFIXES: &[&str] = &[
        "_log",
        "_wood",
        "_stem",
        "_hyphae",
        "_leaves",
        "_sapling",
        "_propagule",
        "_mushroom",
        "_mushroom_block",
        "_fungus",
        "_roots",
        "_coral",
        "_coral_block",
        "_coral_fan",
        "_coral_wall_fan",
        "_tulip",
        "_orchid",
        "_bush",
        "_petals",
        "_carpet",
        "_vines",
        "_vines_plant",
        "_flower",
    ];
    const EXACT: &[&str] = &[
        "short_grass",
        "grass",
        "tall_grass",
        "fern",
        "large_fern",
        "dandelion",
        "poppy",
        "allium",
        "azure_bluet",
        "oxeye_daisy",
        "cornflower",
        "lily_of_the_valley",
        "sunflower",
        "lilac",
        "peony",
        "wither_rose",
        "torchflower",
        "pitcher_plant",
        "spore_blossom",
        "seagrass",
        "tall_seagrass",
        "kelp",
        "kelp_plant",
        "sea_pickle",
        "sugar_cane",
        "cactus",
        "bamboo",
        "vine",
        "glow_lichen",
        "sculk_vein",
        "lily_pad",
        "azalea",
        "flowering_azalea",
        "big_dripleaf",
        "small_dripleaf",
        "cocoa",
        "pumpkin",
        "melon",
        "bee_nest",
        "beehive",
        "snow",
        "ice",
        "nether_sprouts",
        "pointed_dripstone",
        "frogspawn",
        "hanging_roots",
    ];
    path.starts_with("stripped_")
        || EXACT.contains(&path)
        || SUFFIXES.iter().any(|s| path.ends_with(s))
}

/// Soil and loose cover that surface rules lay over the rock: grass, dirt, podzol, mycelium,
/// mud, moss, sand, red sand, gravel, snow blocks, powder snow, packed and blue ice, clay and
/// the terracottas. The `rock` render looks below these.
pub fn is_surface_material(path: &str) -> bool {
    matches!(
        path,
        "grass_block"
            | "dirt"
            | "coarse_dirt"
            | "rooted_dirt"
            | "podzol"
            | "mycelium"
            | "dirt_path"
            | "farmland"
            | "mud"
            | "moss_block"
            | "sand"
            | "red_sand"
            | "suspicious_sand"
            | "gravel"
            | "suspicious_gravel"
            | "snow_block"
            | "powder_snow"
            | "packed_ice"
            | "blue_ice"
            | "clay"
            | "terracotta"
    ) || path.ends_with("_terracotta")
}

/// The ore material of block `name`: `*_ore` with its `deepslate_`/`nether_`/`end_` prefix
/// dropped (`minecraft:deepslate_iron_ore` → `iron`), `ancient_debris` → `netherite`,
/// `raw_*_block` → the metal. None for anything else.
pub fn ore_material(name: &str) -> Option<String> {
    // Excavated Variants' generated blocks (P1-15): `excavated_variants:<ore ns>__<ore path>__
    // <stone ns>__<stone path>` with `/` as `_`, e.g. `excavated_variants__minecraft_copper_ore__
    // excavated_variants__minecraft_granite`: the material is the ore path's last word before `_ore`.
    if name.starts_with("excavated_variants:") {
        let parts: Vec<&str> = path_of(name).split("__").collect();
        if parts.len() == 4 {
            let ore = parts[1].strip_suffix("_ore").unwrap_or(parts[1]);
            let m = ore.rsplit('_').next().unwrap_or(ore);
            return (!m.is_empty()).then(|| m.to_owned());
        }
    }
    let path = path_of(name);
    if path == "ancient_debris" {
        return Some("netherite".into());
    }
    if let Some(m) = path
        .strip_prefix("raw_")
        .and_then(|p| p.strip_suffix("_block"))
    {
        return Some(m.to_owned());
    }
    let base = path.strip_suffix("_ore")?;
    let base = ["deepslate_", "nether_", "end_", "blackstone_", "stone_"]
        .iter()
        .find_map(|p| base.strip_prefix(p))
        .unwrap_or(base);
    (!base.is_empty()).then(|| base.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes() {
        assert_eq!(classify("minecraft:air") & AIR, AIR);
        assert_eq!(classify("minecraft:water"), FLUID | WATER);
        assert_eq!(classify("minecraft:lava"), FLUID | LAVA);
        for f in [
            "minecraft:oak_log",
            "minecraft:stripped_birch_wood",
            "minecraft:jungle_leaves",
            "minecraft:short_grass",
            "minecraft:tall_seagrass",
            "minecraft:brain_coral_block",
            "minecraft:fire_coral_wall_fan",
            "minecraft:sea_pickle",
            "minecraft:snow",
            "minecraft:ice",
            "minecraft:mushroom_stem",
            "minecraft:red_mushroom_block",
            "minecraft:dead_bush",
            "minecraft:cactus",
            "minecraft:moss_carpet",
            "minecraft:mangrove_roots",
            "biomesoplenty:fir_log",
        ] {
            assert!(classify(f) & FEATURE != 0, "{f} is a feature");
        }
        for t in [
            "minecraft:stone",
            "minecraft:grass_block",
            "minecraft:sand",
            "minecraft:sandstone",
            "minecraft:snow_block",
            "minecraft:packed_ice",
            "minecraft:granite",
            "create:asurine",
        ] {
            assert!(classify(t) & NOT_TERRAIN == 0, "{t} is terrain");
        }
        assert!(classify("minecraft:kelp_plant") & WATER != 0);
        assert!(classify("minecraft:grass_block") & SURFACE != 0);
        assert!(classify("minecraft:red_terracotta") & SURFACE != 0);
        assert!(classify("minecraft:stone") & SURFACE == 0);
    }

    #[test]
    fn waterlogged_and_interning() {
        let mut r = Registry::new();
        let a = r.intern(
            "minecraft:oak_stairs",
            &[("waterlogged".into(), "true".into())],
        );
        let b = r.intern(
            "minecraft:oak_stairs",
            &[("waterlogged".into(), "false".into())],
        );
        let c = r.intern("stone", &[]);
        assert_ne!(a, b);
        assert!(r.flags(a) & WATER != 0);
        assert!(r.flags(b) & WATER == 0);
        assert_eq!(r.name(c), "minecraft:stone");
        assert_eq!(r.intern("minecraft:stone", &[]), c);
        assert_eq!(
            r.states[a as usize].key,
            "minecraft:oak_stairs[waterlogged=true]"
        );
    }

    #[test]
    fn ores() {
        assert_eq!(
            ore_material("minecraft:deepslate_iron_ore").as_deref(),
            Some("iron")
        );
        assert_eq!(
            ore_material("minecraft:nether_gold_ore").as_deref(),
            Some("gold")
        );
        assert_eq!(ore_material("minecraft:coal_ore").as_deref(), Some("coal"));
        assert_eq!(
            ore_material("create:deepslate_zinc_ore").as_deref(),
            Some("zinc")
        );
        assert_eq!(
            ore_material("minecraft:raw_copper_block").as_deref(),
            Some("copper")
        );
        assert_eq!(
            ore_material(
                "excavated_variants:excavated_variants__minecraft_copper_ore__excavated_variants__minecraft_granite"
            )
            .as_deref(),
            Some("copper")
        );
        assert_eq!(
            ore_material("excavated_variants:excavated_variants__create_zinc_ore__excavated_variants__create_asurine")
                .as_deref(),
            Some("zinc")
        );
        assert_eq!(
            ore_material("minecraft:ancient_debris").as_deref(),
            Some("netherite")
        );
        assert_eq!(ore_material("minecraft:stone"), None);
        assert_eq!(ore_material("minecraft:_ore"), None);
    }
}

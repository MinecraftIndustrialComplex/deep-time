// SPDX-License-Identifier: LGPL-3.0-or-later
//! The rock classes the surface reads (plan §5.4.1): Deep Time's own `rock_class` data
//! (`neoforge/src/main/resources/data/deeptime/deeptime/rock_class/*.json`, P1-12), compiled in, in
//! the order of `synth::EVERY_CLASS` (a class's index is its position there; the strata log
//! stores it as a u16). The surface uses `erodibility` (a multiplier on the stream-power K),
//! `diffusivity` (on the hillslope κ), `family` and `groups` (provenance), and `tags`.
//!
//! Datapack overrides of the classes reach the core with the content bundle (plan §5.4); until the
//! bundle is wired to the stages the built-in table is used (the same numbers the jar ships).

use std::sync::OnceLock;

use serde::Deserialize;

use crate::synth::EVERY_CLASS;

macro_rules! class_files {
    ($($name:literal),* $(,)?) => {
        [$(($name, include_str!(concat!(
            "../../../../neoforge/src/main/resources/data/deeptime/deeptime/rock_class/",
            $name,
            ".json"
        )))),*]
    };
}

/// The class files, in `EVERY_CLASS` order.
const FILES: [(&str, &str); 59] = class_files!(
    "granite",
    "granodiorite",
    "diorite",
    "gabbro",
    "anorthosite",
    "peridotite",
    "rhyolite",
    "andesite",
    "basalt",
    "tuff",
    "obsidian",
    "scoria",
    "pumice",
    "conglomerate",
    "sandstone",
    "red_bed",
    "mudstone",
    "shale",
    "black_shale",
    "limestone",
    "dolostone",
    "chalk",
    "chert",
    "travertine",
    "halite",
    "gypsum",
    "anhydrite",
    "coal",
    "ironstone",
    "banded_iron_formation",
    "phosphorite",
    "slate",
    "phyllite",
    "schist",
    "blueschist",
    "greenstone",
    "gneiss",
    "quartzite",
    "marble",
    "amphibolite",
    "eclogite",
    "hornfels",
    "skarn",
    "serpentinite",
    "kimberlite",
    "carbonatite",
    "pegmatite",
    "greisen",
    "soil",
    "saprolite",
    "laterite",
    "bauxite",
    "gravel",
    "sand",
    "clay",
    "mud",
    "till",
    "loess",
    "volcanic_ash",
);

#[derive(Deserialize)]
struct File {
    class: String,
    family: String,
    #[serde(default)]
    groups: Vec<String>,
    density_kg_m3: f64,
    erodibility: f64,
    diffusivity: f64,
    #[serde(default)]
    tags: Vec<String>,
}

/// What the surface needs of one class.
#[derive(Clone, Debug, PartialEq)]
pub struct RockClass {
    pub name: &'static str,
    pub family: String,
    pub density_kg_m3: f64,
    pub erodibility: f64,
    pub diffusivity: f64,
    /// Provenance family of material eroded from it: 0 felsic crystalline, 1 mafic and volcanic,
    /// 2 sedimentary (recycled), 3 metamorphic.
    pub source_family: u8,
    pub tags: Vec<String>,
}

/// The table, in `EVERY_CLASS` order.
pub fn table() -> &'static [RockClass] {
    static TABLE: OnceLock<Vec<RockClass>> = OnceLock::new();
    TABLE.get_or_init(|| {
        FILES
            .iter()
            .zip(EVERY_CLASS.iter())
            .map(|(&(name, json), &want)| {
                assert_eq!(name, want, "class order");
                let f: File = serde_json::from_str(json)
                    .unwrap_or_else(|e| panic!("rock_class/{name}.json: {e}"));
                assert_eq!(f.class, name);
                let felsic = f
                    .groups
                    .iter()
                    .any(|g| g.ends_with("/felsic") || g.ends_with("/intermediate"));
                let source_family = match f.family.as_str() {
                    "metamorphic" => 3,
                    "clastic_sedimentary" | "chemical_sedimentary" | "unconsolidated" => 2,
                    "volcanic" if felsic => 0,
                    "volcanic" => 1,
                    _ if felsic => 0,
                    _ => 1,
                };
                RockClass {
                    name,
                    family: f.family,
                    density_kg_m3: f.density_kg_m3,
                    erodibility: f.erodibility,
                    diffusivity: f.diffusivity,
                    source_family,
                    tags: f.tags,
                }
            })
            .collect()
    })
}

/// The index of class `name` (panics for a name outside the vocabulary: callers use literals the
/// tests check).
pub fn index(name: &str) -> u16 {
    EVERY_CLASS
        .iter()
        .position(|&c| c == name)
        .unwrap_or_else(|| panic!("no rock class {name}")) as u16
}

/// Erodibility multipliers, indexed by class (a copy for the kernels).
pub fn erodibility() -> Vec<f64> {
    table().iter().map(|c| c.erodibility).collect()
}

/// Diffusivity multipliers, indexed by class.
pub fn diffusivity() -> Vec<f64> {
    table().iter().map(|c| c.diffusivity).collect()
}

/// Whether class `k` carries `tag`.
pub fn has_tag(k: u16, tag: &str) -> bool {
    table()
        .get(usize::from(k))
        .is_some_and(|c| c.tags.iter().any(|t| t == tag))
}

/// The classes the surface names, resolved once.
#[derive(Clone, Copy, Debug)]
pub struct Classes {
    pub granite: u16,
    pub granodiorite: u16,
    pub gabbro: u16,
    pub rhyolite: u16,
    pub andesite: u16,
    pub basalt: u16,
    pub conglomerate: u16,
    pub sandstone: u16,
    pub red_bed: u16,
    pub mudstone: u16,
    pub shale: u16,
    pub black_shale: u16,
    pub limestone: u16,
    pub dolostone: u16,
    pub halite: u16,
    pub gypsum: u16,
    pub coal: u16,
    pub schist: u16,
    pub gneiss: u16,
    pub soil: u16,
    pub saprolite: u16,
    pub laterite: u16,
    pub bauxite: u16,
}

impl Classes {
    pub fn get() -> Classes {
        Classes {
            granite: index("granite"),
            granodiorite: index("granodiorite"),
            gabbro: index("gabbro"),
            rhyolite: index("rhyolite"),
            andesite: index("andesite"),
            basalt: index("basalt"),
            conglomerate: index("conglomerate"),
            sandstone: index("sandstone"),
            red_bed: index("red_bed"),
            mudstone: index("mudstone"),
            shale: index("shale"),
            black_shale: index("black_shale"),
            limestone: index("limestone"),
            dolostone: index("dolostone"),
            halite: index("halite"),
            gypsum: index("gypsum"),
            coal: index("coal"),
            schist: index("schist"),
            gneiss: index("gneiss"),
            soil: index("soil"),
            saprolite: index("saprolite"),
            laterite: index("laterite"),
            bauxite: index("bauxite"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_the_rock_class_data() {
        let t = table();
        assert_eq!(t.len(), 59);
        let g = &t[usize::from(index("granite"))];
        assert_eq!((g.erodibility, g.diffusivity), (1.0, 1.0));
        assert_eq!(g.source_family, 0);
        assert_eq!(t[usize::from(index("basalt"))].source_family, 1);
        assert_eq!(t[usize::from(index("shale"))].source_family, 2);
        assert_eq!(t[usize::from(index("gneiss"))].source_family, 3);
        assert!(has_tag(index("sandstone"), "reservoir"));
        assert!(has_tag(index("black_shale"), "source_rock"));
        assert!(t.iter().all(|c| c.erodibility > 0.0 && c.diffusivity > 0.0));
        let _ = Classes::get();
        let _ = t[0].density_kg_m3;
    }
}

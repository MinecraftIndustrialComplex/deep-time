// SPDX-License-Identifier: LGPL-3.0-or-later
//! The subcommands (plan §9.7): each writes PNGs and a JSON summary into the output folder and
//! returns the summary.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::blocks;
use crate::colors::ColorTable;
use crate::error::{Error, Result};
use crate::image::{Image, Rgba, TRANSPARENT, hex, mix, shade, text_width};
use crate::palette::Palette;
use crate::region::{self, RegionFile};
use crate::surface::{ExcludeOpts, Excluded, Surface, percentile};
use crate::world::{Area, World};
use crate::worlddata::WorldData;

/// The definition every height check uses, written into its JSON.
pub const HEIGHT_DEFINITION: &str = "measured = y of the highest block in the column that is not air (air, cave_air, void_air), not a fluid (water, lava, bubble_column) and not a feature (vegetation, *_log/*_wood/*_stem/*_hyphae/stripped_*, *_leaves, *_roots, huge mushrooms, coral and coral blocks, sea pickles, snow layers, sheet ice, cactus, bamboo, sugar cane, pumpkins, melons, bee nests, pointed dripstone, moss carpets): OCEAN_FLOOR minus logs, leaves and features. expected = ceil(H) - 1 with H the world data's elevation, bilinear, x wrapped into [-C/2, C/2) (the GameTests' expected top). diff = measured - expected. Fully generated chunks only; structure and carved columns excluded (see \"excluded\").";

/// The exclusion rules, written into the JSON (details in `surface`).
pub const EXCLUSION_RULES: &str = "not_full: chunk status is not full (not counted in columns_total); no_data: H is nodata; no_terrain: no terrain block; structure: within structure_margin blocks (x/z) of a structure piece whose box reaches to within structure_margin blocks of the surface; carved: within pit_margin columns of a pit, a column at least pit_depth blocks below the morphological closing (square max then min, half-width rho = the largest of 2, 4, ..., pit_radius whose square, widened by 2 * bump_radius, holds only chunks at status carvers or later) of the measured surface after an opening (square min then max, half-width bump_radius) that removes narrow bumps; columns with no such square are not tested. Uses the measured surface only, never the world data.";

// Diverging ramp for height differences: blue (measured below expected) ↔ red (above), gray
// midpoint. Blue steps from the reference palette's sequential ramp; red steps matched to them
// in lightness.
const DIFF_ZERO: Rgba = [0xf0, 0xef, 0xec, 255];
const DIFF_NEG: [Rgba; 3] = [
    [0x86, 0xb6, 0xef, 255],
    [0x39, 0x87, 0xe5, 255],
    [0x18, 0x4f, 0x95, 255],
];
const DIFF_POS: [Rgba; 3] = [
    [0xf2, 0xa5, 0xa4, 255],
    [0xe3, 0x49, 0x48, 255],
    [0x9e, 0x25, 0x24, 255],
];
const EXCLUDED_INK: Rgba = [0x4a, 0x48, 0x45, 255];
const SURFACE_BG: Rgba = [0xfc, 0xfc, 0xfb, 255];
const INK: Rgba = [0x0b, 0x0b, 0x0b, 255];
const INK_MUTED: Rgba = [0x6b, 0x6a, 0x66, 255];
const BAR: Rgba = [0x2a, 0x78, 0xd6, 255];

fn diff_color(d: i32) -> Rgba {
    match d {
        0 => DIFF_ZERO,
        d if d < 0 => DIFF_NEG[((-d) as usize - 1).min(2)],
        d => DIFF_POS[(d as usize - 1).min(2)],
    }
}

/// Excluded columns in the difference maps: carved solid dark, structures checkered.
fn excluded_color(e: Excluded, x: i32, z: i32) -> Rgba {
    match e {
        Excluded::NotFull => TRANSPARENT,
        Excluded::Structure if (x + z) & 1 == 0 => DIFF_ZERO,
        _ => EXCLUDED_INK,
    }
}

pub struct Out {
    pub dir: PathBuf,
    pub scale: usize,
}

impl Out {
    pub fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    fn png(&self, img: &Image, name: &str) -> Result<String> {
        let p = self.path(name);
        img.scaled(self.scale).save_png(&p)?;
        Ok(p.display().to_string())
    }

    pub fn json(&self, v: &Value, name: &str) -> Result<String> {
        let p = self.path(name);
        std::fs::create_dir_all(&self.dir).map_err(|e| Error::io(&self.dir, e))?;
        std::fs::write(&p, serde_json::to_string_pretty(v)?).map_err(|e| Error::io(&p, e))?;
        Ok(p.display().to_string())
    }
}

/// `base.ext`, or `base-<i>.ext` when there are several areas.
fn numbered(base: &str, i: usize, n: usize, ext: &str) -> String {
    if n == 1 {
        format!("{base}.{ext}")
    } else {
        format!("{base}-{i}.{ext}")
    }
}

/// The areas to render: `--area` if given, else one per group of generated chunks.
fn areas(world: &World, area: Option<Area>) -> Result<Vec<Area>> {
    let v = match area {
        Some(a) => vec![a],
        None => world.full_areas(),
    };
    if v.is_empty() {
        return Err(Error(format!(
            "{}: no fully generated chunks{}",
            world.dir.display(),
            if area.is_some() { " in the area" } else { "" }
        )));
    }
    Ok(v)
}

fn grow(a: Area, m: i32) -> Area {
    Area {
        x0: a.x0 - m,
        z0: a.z0 - m,
        x1: a.x1 + m,
        z1: a.z1 + m,
    }
}

fn world_summary(world: &World) -> Value {
    let full = world.chunks.values().filter(|c| c.is_full()).count();
    let mut statuses: BTreeMap<&str, usize> = BTreeMap::new();
    for c in world.chunks.values() {
        *statuses.entry(c.status.as_str()).or_default() += 1;
    }
    json!({
        "statuses": statuses,
        "dir": world.dir.display().to_string(),
        "regions_read": world.regions,
        "chunks": world.chunks.len(),
        "chunks_full": full,
        "chunk_errors": world.errors.iter().take(20).collect::<Vec<_>>(),
        "chunk_error_count": world.errors.len(),
    })
}

/// The top-down colour of column (x, z): the highest non-air block, water blended over its
/// floor by depth, hill-shaded against the north-west neighbour.
fn top_color(world: &World, colors: &ColorTable, x: i32, z: i32) -> Option<Rgba> {
    let top = world.top(x, z)?;
    let id = world.block(x, top, z);
    let flags = world.reg.flags(id);
    let mut c = colors.color(world.reg.name(id));
    if flags & blocks::WATER != 0 && flags & blocks::FLUID != 0 {
        let floor = world.terrain_top(x, z);
        let depth = floor.map_or(64, |f| top - f);
        let under = floor.map_or([0x20, 0x30, 0x60, 255], |f| {
            colors.color(world.reg.name(world.block(x, f, z)))
        });
        c = mix(c, under, (0.4 + 0.008 * f64::from(depth)).min(0.8));
    }
    let nw = world.top(x - 1, z - 1).unwrap_or(top);
    let f = 1.0 + (f64::from(top - nw) * 0.08).clamp(-0.3, 0.3);
    Some(shade(c, f))
}

pub fn topdown(
    world_dir: &Path,
    area: Option<Area>,
    colors: &ColorTable,
    out: &Out,
) -> Result<Value> {
    let world = World::load_area(world_dir, area.map(|a| grow(a, 16)))?;
    let list = areas(&world, area)?;
    let mut files = Vec::new();
    let mut top_blocks: HashMap<String, usize> = HashMap::new();
    for (i, a) in list.iter().enumerate() {
        let mut img = Image::new(a.width(), a.height(), TRANSPARENT);
        for z in a.z0..a.z1 {
            for x in a.x0..a.x1 {
                if world.full_chunk(x, z).is_none() {
                    continue;
                }
                if let Some(c) = top_color(&world, colors, x, z) {
                    img.set((x - a.x0) as usize, (z - a.z0) as usize, c);
                }
                if let Some(t) = world.top(x, z) {
                    *top_blocks
                        .entry(world.reg.name(world.block(x, t, z)).to_owned())
                        .or_default() += 1;
                }
            }
        }
        let png = out.png(&img, &numbered("topdown", i, list.len(), "png"))?;
        files.push(json!({"area": a.to_json(), "png": png}));
    }
    let mut tops: Vec<_> = top_blocks.into_iter().collect();
    tops.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let v = json!({
        "command": "topdown",
        "definition": "each pixel is one column seen from above: the highest non-air block's colour (water blended over the floor by depth), hill-shaded from the north-west; north is up",
        "world": world_summary(&world),
        "images": files,
        "top_blocks": tops.iter().take(40).map(|(k, n)| json!([k, n])).collect::<Vec<_>>(),
    });
    out.json(&v, "topdown.json")?;
    Ok(v)
}

/// Köppen class colours by index (Beck et al. 2018's palette, as `climate::koppen::COLOURS`; 31 =
/// sea) and codes.
pub const KOPPEN_COLOURS: [&str; 32] = [
    "#0000ff", "#0078ff", "#46aafa", "#7fbfff", "#ff0000", "#ff9696", "#f5a500", "#ffdc64",
    "#ffff00", "#c8c800", "#969600", "#96ff96", "#64c864", "#329632", "#c8ff50", "#64ff50",
    "#32c800", "#ff00ff", "#c800c8", "#963296", "#966496", "#aaafff", "#5a78dc", "#4b50b4",
    "#320087", "#00ffff", "#37c8ff", "#007d7d", "#00465f", "#b2b2b2", "#666666", "#0b1d3a",
];
pub const KOPPEN_CODES: [&str; 32] = [
    "Af", "Am", "Aw", "As", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb", "Cwc",
    "Cfa", "Cfb", "Cfc", "Dsa", "Dsb", "Dsc", "Dsd", "Dwa", "Dwb", "Dwc", "Dwd", "Dfa", "Dfb",
    "Dfc", "Dfd", "ET", "EF", "sea",
];

/// Map colours of vanilla biomes (the conventional biome-map palette); others get a colour from a
/// hash of their id.
const BIOME_COLOURS: [(&str, &str); 52] = [
    ("ocean", "#000070"),
    ("deep_ocean", "#000030"),
    ("warm_ocean", "#0000ac"),
    ("lukewarm_ocean", "#000090"),
    ("deep_lukewarm_ocean", "#000040"),
    ("cold_ocean", "#202070"),
    ("deep_cold_ocean", "#202038"),
    ("frozen_ocean", "#7070d6"),
    ("deep_frozen_ocean", "#404090"),
    ("plains", "#8db360"),
    ("sunflower_plains", "#b5db88"),
    ("snowy_plains", "#ffffff"),
    ("ice_spikes", "#b4dcdc"),
    ("desert", "#fa9418"),
    ("swamp", "#07f9b2"),
    ("mangrove_swamp", "#67352b"),
    ("forest", "#056621"),
    ("flower_forest", "#2d8e49"),
    ("birch_forest", "#307444"),
    ("old_growth_birch_forest", "#589c6c"),
    ("dark_forest", "#40511a"),
    ("taiga", "#0b6659"),
    ("old_growth_pine_taiga", "#596651"),
    ("old_growth_spruce_taiga", "#818e79"),
    ("snowy_taiga", "#31554a"),
    ("savanna", "#bdb25f"),
    ("savanna_plateau", "#a79d64"),
    ("windswept_savanna", "#e5da87"),
    ("windswept_hills", "#606060"),
    ("windswept_gravelly_hills", "#888888"),
    ("windswept_forest", "#507050"),
    ("jungle", "#537b09"),
    ("sparse_jungle", "#628b17"),
    ("bamboo_jungle", "#768e14"),
    ("badlands", "#d94515"),
    ("eroded_badlands", "#ff6d3d"),
    ("wooded_badlands", "#b09765"),
    ("meadow", "#60a445"),
    ("cherry_grove", "#ffa0d0"),
    ("grove", "#47875a"),
    ("snowy_slopes", "#c4c4c4"),
    ("frozen_peaks", "#a0a0ff"),
    ("jagged_peaks", "#dcdcdc"),
    ("stony_peaks", "#7b8f74"),
    ("river", "#0000ff"),
    ("frozen_river", "#a0a0ff"),
    ("beach", "#fade55"),
    ("snowy_beach", "#faf0c0"),
    ("stony_shore", "#a2a284"),
    ("mushroom_fields", "#ff00ff"),
    ("dripstone_caves", "#7b6d52"),
    ("lush_caves", "#2c6d17"),
];

/// A biome's map colour.
pub fn biome_colour(id: &str) -> Rgba {
    if id == "terralith:deep_warm_ocean" {
        return [0x00, 0x00, 0x60, 255];
    }
    let path = id.strip_prefix("minecraft:");
    if let Some(p) = path
        && let Some((_, c)) = BIOME_COLOURS.iter().find(|(n, _)| *n == p)
    {
        return crate::image::parse_hex(c).unwrap_or([128, 128, 128, 255]);
    }
    // FNV-1a of the id → a mid-light colour.
    let mut h: u32 = 0x811c_9dc5;
    for b in id.bytes() {
        h = (h ^ u32::from(b)).wrapping_mul(0x0100_0193);
    }
    let ch = |s: u32| (64 + ((h >> s) & 0xff) * 160 / 255) as u8;
    [ch(0), ch(8), ch(16), 255]
}

/// Surface biomes and the Köppen map of the same area (P1-09).
pub fn biomes(
    world_dir: &Path,
    area: Option<Area>,
    wd: Option<&WorldData>,
    name: &str,
    out: &Out,
) -> Result<Value> {
    let world = World::load_area(world_dir, area)?;
    let list = areas(&world, area)?;
    let mut files = Vec::new();
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut by_class: BTreeMap<u8, BTreeMap<String, u64>> = BTreeMap::new();
    for (i, a) in list.iter().enumerate() {
        let mut img = Image::new(a.width(), a.height(), TRANSPARENT);
        let mut kimg = Image::new(a.width(), a.height(), TRANSPARENT);
        for z in a.z0..a.z1 {
            for x in a.x0..a.x1 {
                let Some(c) = world.full_chunk(x, z) else {
                    continue;
                };
                let Some(top) = world.terrain_top(x, z).or_else(|| world.top(x, z)) else {
                    continue;
                };
                let Some(b) = c.biome((x & 15) as usize, top, (z & 15) as usize) else {
                    continue;
                };
                let (px, pz) = ((x - a.x0) as usize, (z - a.z0) as usize);
                img.set(px, pz, biome_colour(b));
                *counts.entry(b.to_owned()).or_default() += 1;
                if let Some(k) = wd.and_then(|w| w.koppen(x, z)) {
                    let kc = crate::image::parse_hex(KOPPEN_COLOURS[usize::from(k)])
                        .unwrap_or([0, 0, 0, 255]);
                    kimg.set(px, pz, kc);
                    *by_class
                        .entry(k)
                        .or_default()
                        .entry(b.to_owned())
                        .or_default() += 1;
                }
            }
        }
        let png = out.png(&img, &numbered(name, i, list.len(), "png"))?;
        let mut f = json!({"area": a.to_json(), "png": png});
        if wd.is_some() {
            f["koppen_png"] = json!(out.png(
                &kimg,
                &numbered(&format!("{name}-koppen"), i, list.len(), "png")
            )?);
        }
        files.push(f);
    }
    let mut legend: Vec<(String, u64)> = counts.into_iter().collect();
    legend.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let classes: serde_json::Map<String, Value> = by_class
        .iter()
        .map(|(k, m)| {
            let total: u64 = m.values().sum();
            let mut v: Vec<(&String, &u64)> = m.iter().collect();
            v.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            (
                KOPPEN_CODES[usize::from(*k)].to_owned(),
                json!({
                    "columns": total,
                    "biomes": v.iter().take(8).map(|(b, n)| json!([b, n])).collect::<Vec<_>>(),
                }),
            )
        })
        .collect();
    let v = json!({
        "command": "biomes",
        "definition": "each pixel is one column: the biome the chunk stores at the column's terrain top (the highest block that is not air, fluid or a feature), coloured with the conventional biome-map palette (vanilla) or a hash of the id (mods); the -koppen image is the world data's climate/koppen at the same columns (nearest cell), Beck et al. 2018's palette; north is up",
        "world": world_summary(&world),
        "images": files,
        "legend": legend.iter().map(|(b, n)| json!({"id": b, "columns": n, "colour": hex(biome_colour(b))})).collect::<Vec<_>>(),
        "koppen_colours": KOPPEN_COLOURS.iter().zip(KOPPEN_CODES).map(|(c, k)| json!([k, c])).collect::<Vec<_>>(),
        "by_koppen": classes,
    });
    out.json(&v, &format!("{name}.json"))?;
    Ok(v)
}

/// Rock sources when neither `--palette` nor `--tags` is given: the world's server mods and
/// datapacks, and this repository's resources if run from its root.
pub fn default_tag_sources(world_dir: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(parent) = world_dir.parent() {
        let mods = parent.join("mods");
        if let Ok(rd) = std::fs::read_dir(&mods) {
            let mut jars: Vec<PathBuf> = rd
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "jar"))
                .collect();
            jars.sort();
            v.extend(jars);
        }
    }
    let dp = world_dir.join("datapacks");
    if dp.is_dir() {
        v.push(dp);
    }
    let repo = PathBuf::from("neoforge/src/main/resources");
    if repo.is_dir() {
        v.push(repo);
    }
    v
}

pub fn rock(
    world_dir: &Path,
    area: Option<Area>,
    palette: &Palette,
    colors: &ColorTable,
    out: &Out,
) -> Result<Value> {
    const MAX_DEPTH: i32 = 96;
    let world = World::load_area(world_dir, area)?;
    let list = areas(&world, area)?;
    let mut classes: BTreeMap<String, usize> = BTreeMap::new();
    let mut unmapped: BTreeMap<String, usize> = BTreeMap::new();
    let mut depth_sum: i64 = 0;
    let mut found = 0usize;
    let mut files = Vec::new();
    for (i, a) in list.iter().enumerate() {
        let mut img = Image::new(a.width(), a.height(), TRANSPARENT);
        for z in a.z0..a.z1 {
            for x in a.x0..a.x1 {
                if world.full_chunk(x, z).is_none() {
                    continue;
                }
                let Some(t) = world.terrain_top(x, z) else {
                    continue;
                };
                let hit = (t - MAX_DEPTH..=t)
                    .rev()
                    .find(|&y| world.flags(x, y, z) & (blocks::NOT_TERRAIN | blocks::SURFACE) == 0);
                let Some(y) = hit else { continue };
                let name = world.reg.name(world.block(x, y, z));
                found += 1;
                depth_sum += i64::from(t - y);
                let (px, pz) = ((x - a.x0) as usize, (z - a.z0) as usize);
                match palette.class_of(name) {
                    Some(class) => {
                        *classes.entry(class.to_owned()).or_default() += 1;
                        let rep = palette.first.get(class).map_or(name, String::as_str);
                        img.set(px, pz, colors.color(rep));
                    }
                    None => {
                        *unmapped.entry(name.to_owned()).or_default() += 1;
                        // Unmapped blocks: their own colour, checkered with dark ink.
                        let c = if (x + z) & 1 == 0 {
                            colors.color(name)
                        } else {
                            EXCLUDED_INK
                        };
                        img.set(px, pz, c);
                    }
                }
            }
        }
        let png = out.png(&img, &numbered("rock", i, list.len(), "png"))?;
        files.push(json!({"area": a.to_json(), "png": png}));
    }
    let v = json!({
        "command": "rock",
        "definition": format!("per column the first block below the terrain surface (down to {MAX_DEPTH} blocks) that is not air, fluid, a feature or surface material (grass, dirt, sand, gravel, snow, ice, clay, terracotta, ...), coloured by its rock class's first block; blocks with no class are checkered"),
        "world": world_summary(&world),
        "palette_sources": palette.sources,
        "palette_blocks": palette.blocks.len(),
        "images": files,
        "classes": classes,
        "unmapped_blocks": unmapped,
        "columns": found,
        "mean_depth_below_surface": if found > 0 { depth_sum as f64 / found as f64 } else { 0.0 },
    });
    out.json(&v, "rock.json")?;
    Ok(v)
}

/// A fixed colour per common ore material; others hashed.
fn ore_color(material: &str) -> Rgba {
    match material {
        "coal" => [0x3a, 0x3a, 0x3a, 255],
        "iron" => [0xd8, 0xaf, 0x93, 255],
        "copper" => [0xe0, 0x77, 0x3e, 255],
        "gold" => [0xfc, 0xee, 0x4b, 255],
        "redstone" => [0xff, 0x1a, 0x1a, 255],
        "lapis" => [0x1f, 0x4f, 0xbf, 255],
        "diamond" => [0x5d, 0xec, 0xf5, 255],
        "emerald" => [0x17, 0xdd, 0x62, 255],
        "zinc" => [0xa6, 0xc5, 0xae, 255],
        "quartz" => [0xe7, 0xe1, 0xd8, 255],
        "netherite" => [0x6d, 0x3a, 0x33, 255],
        m => crate::colors::fallback(m),
    }
}

/// A site's footprint for the ore render's overlay (P1-15): id, deposit type, and the inclusive
/// block AABB x and z bounds.
#[derive(Clone, Debug)]
pub struct SiteBox {
    pub id: u32,
    pub deposit_type: String,
    pub x0: i32,
    pub z0: i32,
    pub x1: i32,
    pub z1: i32,
}

impl SiteBox {
    fn contains(&self, x: i32, z: i32) -> bool {
        x >= self.x0 && x <= self.x1 && z >= self.z0 && z <= self.z1
    }
}

pub fn ore(world_dir: &Path, area: Option<Area>, out: &Out) -> Result<Value> {
    ore_with_sites(world_dir, area, &[], out)
}

/// The ore render with site footprints outlined (white) and counted: how many ore columns lie
/// inside a site's footprint and how many outside (background scatter and unclaimed ores).
pub fn ore_with_sites(
    world_dir: &Path,
    area: Option<Area>,
    sites: &[SiteBox],
    out: &Out,
) -> Result<Value> {
    let world = World::load_area(world_dir, area)?;
    let list = areas(&world, area)?;
    struct Stat {
        count: usize,
        min_y: i32,
        max_y: i32,
        sum_y: i64,
    }
    let mut by_block: BTreeMap<String, Stat> = BTreeMap::new();
    let mut by_material: BTreeMap<String, Stat> = BTreeMap::new();
    let mut files = Vec::new();
    let mut in_sites: BTreeMap<String, usize> = BTreeMap::new();
    let mut outside: BTreeMap<String, usize> = BTreeMap::new();
    let mut shown: Vec<Value> = Vec::new();
    for (i, a) in list.iter().enumerate() {
        // Highest ore per column: (y, material).
        let mut best: HashMap<(i32, i32), (i32, String)> = HashMap::new();
        for c in world.chunks.values() {
            if !c.is_full() || !a.contains_chunk(c.cx, c.cz) {
                continue;
            }
            for (si, s) in c.sections.iter().enumerate() {
                let Some(s) = s else { continue };
                if s.flags & blocks::ORE == 0 {
                    continue;
                }
                let y0 = (c.min_section + si as i32) * 16;
                for ly in 0..16 {
                    for lz in 0..16 {
                        for lx in 0..16 {
                            let id = s.get(lx, ly, lz);
                            if world.reg.flags(id) & blocks::ORE == 0 {
                                continue;
                            }
                            let (x, y, z) =
                                (c.cx * 16 + lx as i32, y0 + ly as i32, c.cz * 16 + lz as i32);
                            if x < a.x0 || x >= a.x1 || z < a.z0 || z >= a.z1 {
                                continue;
                            }
                            let name = world.reg.name(id);
                            let mat = blocks::ore_material(name).unwrap_or_default();
                            for (map, key) in [
                                (&mut by_block, name.to_owned()),
                                (&mut by_material, mat.clone()),
                            ] {
                                let st = map.entry(key).or_insert(Stat {
                                    count: 0,
                                    min_y: i32::MAX,
                                    max_y: i32::MIN,
                                    sum_y: 0,
                                });
                                st.count += 1;
                                st.min_y = st.min_y.min(y);
                                st.max_y = st.max_y.max(y);
                                st.sum_y += i64::from(y);
                            }
                            let e = best.entry((x, z)).or_insert((i32::MIN, String::new()));
                            if y > e.0 {
                                *e = (y, mat);
                            }
                        }
                    }
                }
            }
        }
        let mut img = Image::new(a.width(), a.height(), TRANSPARENT);
        for z in a.z0..a.z1 {
            for x in a.x0..a.x1 {
                if world.full_chunk(x, z).is_none() {
                    continue;
                }
                let (px, pz) = ((x - a.x0) as usize, (z - a.z0) as usize);
                match best.get(&(x, z)) {
                    Some((y, mat)) => {
                        let depth = world.terrain_top(x, z).map_or(0, |t| (t - y).max(0));
                        let f = 1.0 - (f64::from(depth) / 400.0).min(0.7);
                        img.set(px, pz, shade(ore_color(mat), f));
                        let bucket = if sites.iter().any(|s| s.contains(x, z)) {
                            &mut in_sites
                        } else {
                            &mut outside
                        };
                        *bucket.entry(mat.clone()).or_insert(0) += 1;
                    }
                    None => img.set(px, pz, [0x1a, 0x1a, 0x19, 255]),
                }
            }
        }
        // Site footprints: a white outline, drawn over the ore.
        for sb in sites {
            if sb.x1 < a.x0 || sb.x0 >= a.x1 || sb.z1 < a.z0 || sb.z0 >= a.z1 {
                continue;
            }
            shown.push(json!({"id": sb.id, "type": sb.deposit_type, "x": [sb.x0, sb.x1], "z": [sb.z0, sb.z1]}));
            for z in sb.z0.max(a.z0)..=sb.z1.min(a.z1 - 1) {
                for x in sb.x0.max(a.x0)..=sb.x1.min(a.x1 - 1) {
                    if x == sb.x0 || x == sb.x1 || z == sb.z0 || z == sb.z1 {
                        img.set(
                            (x - a.x0) as usize,
                            (z - a.z0) as usize,
                            [0xff, 0xff, 0xff, 255],
                        );
                    }
                }
            }
        }
        let png = out.png(&img, &numbered("ore", i, list.len(), "png"))?;
        files.push(json!({"area": a.to_json(), "png": png}));
    }
    let stat = |m: &BTreeMap<String, Stat>| -> Value {
        m.iter()
            .map(|(k, s)| {
                (
                    k.clone(),
                    json!({"count": s.count, "min_y": s.min_y, "max_y": s.max_y,
                           "mean_y": s.sum_y as f64 / s.count.max(1) as f64}),
                )
            })
            .collect::<serde_json::Map<_, _>>()
            .into()
    };
    let legend: serde_json::Map<String, Value> = by_material
        .keys()
        .map(|m| (m.clone(), Value::from(hex(ore_color(m)))))
        .collect();
    let v = json!({
        "command": "ore",
        "definition": "ore blocks (*_ore, Excavated Variants variants, ancient_debris, raw_*_block) by material (deepslate_/nether_ prefixes dropped); each pixel is the column's highest ore, darker the deeper below the terrain surface (full dark at 280 blocks); columns with no ore are near-black",
        "world": world_summary(&world),
        "images": files,
        "legend": legend,
        "materials": stat(&by_material),
        "blocks": stat(&by_block),
        "sites": shown,
        "ore_columns_in_site_footprints": in_sites,
        "ore_columns_outside_sites": outside,
    });
    out.json(&v, "ore.json")?;
    Ok(v)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// The plane x = at; the image runs along z.
    X,
    /// The plane z = at; the image runs along x.
    Z,
}

pub struct SectionArgs {
    pub axis: Axis,
    pub at: i32,
    pub from: i32,
    pub to: i32,
    pub y_from: Option<i32>,
    pub y_to: Option<i32>,
}

/// Overlay colours for the section (drawn on top of the blocks).
const OVERLAY_EXPECTED: Rgba = [0xd5, 0x51, 0x81, 255];
const OVERLAY_WATER: Rgba = [0x1b, 0xaf, 0x7a, 255];
const OVERLAY_HORIZONS: [Rgba; 4] = [
    [0xed, 0xa1, 0x00, 255],
    [0x4a, 0x3a, 0xa7, 255],
    [0x00, 0x83, 0x00, 255],
    [0xeb, 0x68, 0x34, 255],
];

pub fn section(
    world_dir: &Path,
    wd: Option<&WorldData>,
    args: &SectionArgs,
    colors: &ColorTable,
    out: &Out,
    stdout: &mut Vec<String>,
) -> Result<Value> {
    let (lo, hi) = (args.from.min(args.to), args.from.max(args.to));
    let pos = |t: i32| match args.axis {
        Axis::X => (args.at, t),
        Axis::Z => (t, args.at),
    };
    let world = World::load(world_dir, |cx, cz| match args.axis {
        Axis::X => cx == args.at >> 4 && cz >= lo >> 4 && cz <= hi >> 4,
        Axis::Z => cz == args.at >> 4 && cx >= lo >> 4 && cx <= hi >> 4,
    })?;
    if world.chunks.is_empty() {
        return Err(Error(format!(
            "{}: no chunks on the section line",
            world_dir.display()
        )));
    }
    struct Col {
        t: i32,
        top: Option<i32>,
        terrain: Option<i32>,
        water: Option<i32>,
        expected: Option<i32>,
        horizons: Vec<(f64, String)>,
        level: Option<f64>,
    }
    let cols: Vec<Col> = (lo..=hi)
        .map(|t| {
            let (x, z) = pos(t);
            Col {
                t,
                top: world.top(x, z),
                terrain: world.terrain_top(x, z),
                water: world.water_top(x, z),
                expected: wd.and_then(|w| w.expected_top(x, z)),
                horizons: wd.map(|w| w.horizons(x, z)).unwrap_or_default(),
                level: wd.and_then(|w| w.water_level(x, z)),
            }
        })
        .collect();
    let min_y = world
        .chunks
        .values()
        .map(|c| c.min_y())
        .min()
        .unwrap_or(-64);
    let max_y = world
        .chunks
        .values()
        .map(|c| c.max_y())
        .max()
        .unwrap_or(320)
        - 1;
    let lowest = cols.iter().filter_map(|c| c.terrain).min().unwrap_or(min_y);
    let highest = cols
        .iter()
        .flat_map(|c| [c.top, c.water, c.expected])
        .flatten()
        .max()
        .unwrap_or(max_y);
    let y0 = args.y_from.unwrap_or(lowest - 48).max(min_y);
    let y1 = args.y_to.unwrap_or(highest + 12).min(max_y);
    if y1 < y0 {
        return Err(Error(format!("empty y range {y0}..{y1}")));
    }
    let (w, h) = (cols.len(), (y1 - y0 + 1) as usize);
    let mut img = Image::new(w, h, TRANSPARENT);
    let row = |y: i32| (y1 - y) as usize;
    for (i, c) in cols.iter().enumerate() {
        let (x, z) = pos(c.t);
        let loaded = world.chunk(x, z).is_some();
        for y in y0..=y1 {
            let id = world.block(x, y, z);
            let f = world.reg.flags(id);
            let px = if !loaded {
                TRANSPARENT
            } else if f & blocks::AIR != 0 {
                if c.top.is_some_and(|t| y < t) {
                    [0x1f, 0x1f, 0x1e, 255]
                } else {
                    [0xdf, 0xe9, 0xf3, 255]
                }
            } else {
                colors.color(world.reg.name(id))
            };
            img.set(i, row(y), px);
        }
    }
    // Overlays: horizon tops (k ≥ 1; horizon 0 starts at the surface), the world data's water
    // level, and the expected surface (the first block above the expected top).
    for (i, c) in cols.iter().enumerate() {
        for (k, (top, _)) in c.horizons.iter().enumerate().skip(1) {
            let y = top.floor() as i32;
            // Only where there is ground: tops above the column's top block paint nothing.
            if (y0..=y1).contains(&y) && c.top.is_some_and(|t| y <= t) {
                let under = img.get(i, row(y));
                img.set(i, row(y), mix(OVERLAY_HORIZONS[(k - 1) % 4], under, 0.75));
            }
        }
        if let Some(l) = c.level {
            let y = l.floor() as i32;
            if (y0..=y1).contains(&y) {
                img.set(i, row(y), OVERLAY_WATER);
            }
        }
        if let Some(e) = c.expected
            && (y0..=y1).contains(&(e + 1))
        {
            img.set(i, row(e + 1), OVERLAY_EXPECTED);
        }
    }
    // Scale, then add axis labels in a margin.
    let s = out.scale.max(1);
    let body = img.scaled(s);
    let (ml, mb) = (36usize, 18usize);
    let mut fin = Image::new(body.w + ml, body.h + mb, SURFACE_BG);
    for y in 0..body.h {
        for x in 0..body.w {
            fin.set(x + ml, y, body.get(x, y));
        }
    }
    let first_tick = y0 + (16 - y0.rem_euclid(16)) % 16;
    for y in (first_tick..=y1).step_by(16) {
        let py = row(y) * s;
        fin.fill_rect(ml - 4, py, 4, 1, INK_MUTED);
        let label = y.to_string();
        let tw = text_width(&label, 1);
        fin.text(
            ml.saturating_sub(6 + tw),
            py.saturating_sub(2),
            &label,
            1,
            INK,
        );
    }
    let first_t = lo + (32 - lo.rem_euclid(32)) % 32;
    for t in (first_t..=hi).step_by(32) {
        let px = ml + (t - lo) as usize * s;
        fin.fill_rect(px, body.h, 1, 4, INK_MUTED);
        fin.text(px + 2, body.h + 6, &t.to_string(), 1, INK);
    }
    let axis_name = match args.axis {
        Axis::X => "x",
        Axis::Z => "z",
    };
    let base = format!("section-{axis_name}{}", args.at);
    let p = out.path(&format!("{base}.png"));
    fin.save_png(&p)?;
    // Water surfaces, for scripts: one line per distinct water top.
    let mut water: BTreeMap<i32, usize> = BTreeMap::new();
    for c in &cols {
        if let Some(y) = c.water {
            *water.entry(y).or_default() += 1;
        }
    }
    stdout.push(format!(
        "columns={} water_columns={}",
        cols.len(),
        water.values().sum::<usize>()
    ));
    for (y, n) in &water {
        stdout.push(format!(
            "water_top_y={} water_block_y={y} columns={n}",
            y + 1
        ));
    }
    let along = match args.axis {
        Axis::X => "z",
        Axis::Z => "x",
    };
    let v = json!({
        "command": "section",
        "plane": format!("{axis_name} = {}", args.at),
        "along": along,
        "from": lo, "to": hi, "y_from": y0, "y_to": y1,
        "definition": "one pixel per block (then scaled): blocks by colour, sky light blue, air under the column's top block near-black; overlays: magenta = the first block above the world data's expected top (ceil(H)), green = the world data's water_level, yellow/violet/green/orange = the tops of strata horizons 1.. at or below the column's top block (horizon 0 starts at the surface). water_top_y = the top face of the highest block holding water above the terrain surface (water_block_y + 1; P0-13's lakeHolds uses the same; sea level 63 gives water_top_y 63, water_block_y 62).",
        "legend": {"expected_surface": hex(OVERLAY_EXPECTED), "water_level": hex(OVERLAY_WATER),
                   "horizons": OVERLAY_HORIZONS.iter().map(|c| hex(*c)).collect::<Vec<_>>()},
        "bodies": "none: world-data format 0.0 has no site bodies",
        "world_data": wd.map(|w| w.dir.display().to_string()),
        "png": p.display().to_string(),
        "water_tops": water.iter().map(|(y, n)| json!({"water_top_y": y + 1, "water_block_y": y, "columns": n})).collect::<Vec<_>>(),
        "columns": cols.iter().map(|c| json!({
            along: c.t, "top": c.top, "terrain": c.terrain, "water_block_y": c.water,
            "expected": c.expected,
            "horizons": c.horizons.iter().map(|(t, n)| json!([t, n])).collect::<Vec<_>>(),
            "water_level": c.level,
        })).collect::<Vec<_>>(),
    });
    out.json(&v, &format!("{base}.json"))?;
    Ok(v)
}

/// Height-check statistics over compared columns.
#[derive(Default)]
struct DiffStats {
    diffs: Vec<i32>,
    hist: BTreeMap<i32, usize>,
    excluded: BTreeMap<&'static str, usize>,
    total: usize,
    worst: Vec<(i32, i32, i32, i32, i32, f64)>,
}

impl DiffStats {
    fn add(&mut self, x: i32, z: i32, measured: i32, expected: i32, h: f64) {
        let d = measured - expected;
        self.diffs.push(d);
        *self.hist.entry(d).or_default() += 1;
        self.worst.push((d.abs(), x, z, measured, expected, h));
        if self.worst.len() > 64 {
            self.worst.sort_by_key(|a| std::cmp::Reverse(a.0));
            self.worst.truncate(16);
        }
    }

    fn summary(&mut self) -> Value {
        let mut abs: Vec<i32> = self.diffs.iter().map(|d| d.abs()).collect();
        abs.sort_unstable();
        self.worst
            .sort_by(|a, b| b.0.cmp(&a.0).then((a.1, a.2).cmp(&(b.1, b.2))));
        self.worst.truncate(10);
        let n = self.diffs.len();
        let mean = if n > 0 {
            self.diffs.iter().map(|&d| f64::from(d)).sum::<f64>() / n as f64
        } else {
            0.0
        };
        let within1 = abs.iter().filter(|&&a| a <= 1).count();
        json!({
            "columns_total": self.total,
            "columns_compared": n,
            "excluded": self.excluded,
            "p50_abs_diff_blocks": percentile(&abs, 50.0),
            "p99_abs_diff_blocks": percentile(&abs, 99.0),
            "max_abs_diff_blocks": abs.last(),
            "mean_diff_blocks": mean,
            "share_within_1_block": if n > 0 { within1 as f64 / n as f64 } else { 0.0 },
            "histogram": self.hist.iter().map(|(d, c)| (d.to_string(), Value::from(*c))).collect::<serde_json::Map<_, _>>(),
            "worst": self.worst.iter().map(|&(_, x, z, m, e, h)| json!({"x": x, "z": z, "measured": m, "expected": e, "H": h})).collect::<Vec<_>>(),
        })
    }
}

/// Draws the difference histogram: bins −5 … +5 plus both tails, log-scaled bar heights.
fn histogram_png(hist: &BTreeMap<i32, usize>) -> Image {
    let mut bins: Vec<(String, usize)> = Vec::new();
    let tail_lo: usize = hist.iter().filter(|(d, _)| **d < -5).map(|(_, c)| c).sum();
    let tail_hi: usize = hist.iter().filter(|(d, _)| **d > 5).map(|(_, c)| c).sum();
    bins.push(("<-5".into(), tail_lo));
    for d in -5..=5 {
        let label = if d > 0 {
            format!("+{d}")
        } else {
            d.to_string()
        };
        bins.push((label, hist.get(&d).copied().unwrap_or(0)));
    }
    bins.push((">5".into(), tail_hi));
    let (bar_w, gap, left, top, plot_h, bottom) =
        (28usize, 8usize, 16usize, 44usize, 220usize, 40usize);
    let w = left * 2 + bins.len() * (bar_w + gap);
    let mut img = Image::new(w, top + plot_h + bottom, SURFACE_BG);
    img.text(left, 10, "MEASURED - EXPECTED (BLOCKS)", 2, INK);
    img.text(left, 26, "COLUMNS PER BIN, LOG SCALE", 1, INK_MUTED);
    let max = bins.iter().map(|b| b.1).max().unwrap_or(1).max(1);
    let scale = |c: usize| {
        if c == 0 {
            0
        } else {
            ((((c as f64) + 1.0).log10() / ((max as f64) + 1.0).log10()) * plot_h as f64)
                .round()
                .max(2.0) as usize
        }
    };
    let base = top + plot_h;
    img.fill_rect(left, base, w - 2 * left, 1, INK_MUTED);
    for (i, (label, c)) in bins.iter().enumerate() {
        let x = left + i * (bar_w + gap) + gap / 2;
        let bh = scale(*c);
        img.fill_rect(x, base - bh, bar_w, bh, BAR);
        let count = c.to_string();
        let cw = text_width(&count, 1);
        img.text(
            x + (bar_w.saturating_sub(cw)) / 2,
            (base - bh).saturating_sub(8),
            &count,
            1,
            INK_MUTED,
        );
        let lw = text_width(label, 2);
        img.text(x + (bar_w.saturating_sub(lw)) / 2, base + 8, label, 2, INK);
    }
    img
}

pub fn compare(
    world_dir: &Path,
    wd: &WorldData,
    area: Option<Area>,
    opts: ExcludeOpts,
    out: &Out,
) -> Result<Value> {
    let world = World::load_area(world_dir, area.map(|a| grow(a, 256)))?;
    let list = areas(&world, area)?;
    let mut stats = DiffStats::default();
    let mut maps = Vec::new();
    for (i, a) in list.iter().enumerate() {
        let xs: Vec<i32> = (a.x0..a.x1).collect();
        let fold = |x: i32| wd.wrap_x(i64::from(x)) as i32;
        let surf = Surface::compute(&world, &xs, fold, a.z0, a.z1, opts);
        let mut img = Image::new(a.width(), a.height(), TRANSPARENT);
        let mut local = DiffStats::default();
        for z in a.z0..a.z1 {
            for x in a.x0..a.x1 {
                let expected = wd.expected_top(x, z);
                let (px, pz) = ((x - a.x0) as usize, (z - a.z0) as usize);
                match surf.exclusion(&world, x, z, expected) {
                    Some(Excluded::NotFull) => {}
                    Some(e) => {
                        stats.total += 1;
                        local.total += 1;
                        *stats.excluded.entry(e.name()).or_default() += 1;
                        *local.excluded.entry(e.name()).or_default() += 1;
                        img.set(px, pz, excluded_color(e, x, z));
                    }
                    None => {
                        let (m, e) = (surf.top(x, z).expect("checked"), expected.expect("checked"));
                        stats.total += 1;
                        local.total += 1;
                        let h = wd.height(x, z);
                        stats.add(x, z, m, e, h);
                        local.add(x, z, m, e, h);
                        img.set(px, pz, diff_color(m - e));
                    }
                }
            }
        }
        let png = out.png(&img, &numbered("compare-map", i, list.len(), "png"))?;
        let mut s = local.summary();
        s["area"] = a.to_json();
        s["png"] = Value::from(png);
        maps.push(s);
    }
    let hist_png = out.path("compare-hist.png");
    histogram_png(&stats.hist).save_png(&hist_png)?;
    let mut v = stats.summary();
    let summary = json!({
        "command": "compare",
        "definition": HEIGHT_DEFINITION,
        "exclusions": {
            "rules": EXCLUSION_RULES,
            "params": opts.to_json(),
        },
        "world": world_summary(&world),
        "world_data": wd.dir.display().to_string(),
        "circumference": wd.circumference,
        "maps": maps,
        "histogram_png": hist_png.display().to_string(),
        "map_legend": {
            "0": hex(DIFF_ZERO), "-1": hex(DIFF_NEG[0]), "-2": hex(DIFF_NEG[1]), "<=-3": hex(DIFF_NEG[2]),
            "+1": hex(DIFF_POS[0]), "+2": hex(DIFF_POS[1]), ">=+3": hex(DIFF_POS[2]),
            "carved": hex(EXCLUDED_INK), "structure": "checker of the carved ink and the zero gray",
            "not generated": "transparent"
        },
    });
    if let (Value::Object(dst), Value::Object(src)) = (&mut v, summary) {
        for (k, val) in src {
            dst.insert(k, val);
        }
    }
    out.json(&v, "compare.json")?;
    Ok(v)
}

pub struct SeamArgs {
    /// The circumference C.
    pub span: i64,
    /// Columns on each side of the seam.
    pub width: i32,
    /// z range (inclusive); default: where both sides are generated.
    pub z_range: Option<(i32, i32)>,
    pub opts: ExcludeOpts,
}

pub fn seam(
    world_dir: &Path,
    wd: Option<&WorldData>,
    args: &SeamArgs,
    colors: &ColorTable,
    out: &Out,
) -> Result<Value> {
    let (span, width, z_range, opts) = (args.span, args.width, args.z_range, args.opts);
    let half = (span / 2) as i32;
    let reach = width + 256;
    let world = World::load(world_dir, |cx, _| {
        let (x0, x1) = (cx * 16, cx * 16 + 16);
        x1 > half - reach && x0 < half || x0 < -half + reach && x1 > -half
    })?;
    // Strip column i → block x: the last `width` columns before the seam, then the first
    // `width` after it.
    let strip_x = |i: i32| {
        if i < width {
            half - width + i
        } else {
            -half + (i - width)
        }
    };
    let (z0, z1) = match z_range {
        Some(r) => r,
        None => {
            let east: Vec<i32> = world
                .full_chunks()
                .into_iter()
                .filter(|&(cx, _)| cx == (half - 1) >> 4)
                .map(|(_, cz)| cz)
                .collect();
            let west: Vec<i32> = world
                .full_chunks()
                .into_iter()
                .filter(|&(cx, _)| cx == (-half) >> 4)
                .map(|(_, cz)| cz)
                .collect();
            let both: Vec<i32> = east.iter().copied().filter(|c| west.contains(c)).collect();
            if both.is_empty() {
                return Err(Error(format!(
                    "{}: no fully generated chunks on both sides of the seam x = ±{half}",
                    world_dir.display()
                )));
            }
            (
                both.iter().min().unwrap() * 16,
                both.iter().max().unwrap() * 16 + 15,
            )
        }
    };
    // One surface across the seam: the strip is contiguous in the wrapped world.
    let fold = |x: i32| (x + half).rem_euclid(2 * half) - half;
    let xs: Vec<i32> = (0..2 * width).map(strip_x).collect();
    let surf = Surface::compute(&world, &xs, fold, z0, z1 + 1, opts);
    let (w, h) = ((2 * width) as usize, (z1 - z0 + 1) as usize);
    let mut strip = Image::new(w, h, TRANSPARENT);
    let mut diff = Image::new(w, h, TRANSPARENT);
    let mut stats = DiffStats::default();
    let mut per_column = Vec::new();
    for i in 0..2 * width {
        let x = strip_x(i);
        let mut col = DiffStats::default();
        for z in z0..=z1 {
            let pz = (z - z0) as usize;
            if world.full_chunk(x, z).is_none() {
                continue;
            }
            if let Some(c) = top_color(&world, colors, x, z) {
                strip.set(i as usize, pz, c);
            }
            let Some(wd) = wd else { continue };
            let expected = wd.expected_top(x, z);
            let s = &surf;
            match s.exclusion(&world, x, z, expected) {
                Some(Excluded::NotFull) => {}
                Some(e) => {
                    stats.total += 1;
                    col.total += 1;
                    *stats.excluded.entry(e.name()).or_default() += 1;
                    diff.set(i as usize, pz, excluded_color(e, x, z));
                }
                None => {
                    let (m, e) = (s.top(x, z).expect("checked"), expected.expect("checked"));
                    stats.total += 1;
                    col.total += 1;
                    stats.add(x, z, m, e, wd.height(x, z));
                    col.add(x, z, m, e, wd.height(x, z));
                    diff.set(i as usize, pz, diff_color(m - e));
                }
            }
        }
        if wd.is_some() {
            let mut abs: Vec<i32> = col.diffs.iter().map(|d| d.abs()).collect();
            abs.sort_unstable();
            let n = col.diffs.len();
            per_column.push(json!({
                "x": x, "compared": n,
                "mean_diff": if n > 0 { col.diffs.iter().map(|&d| f64::from(d)).sum::<f64>() / n as f64 } else { 0.0 },
                "max_abs_diff": abs.last(),
            }));
        }
    }
    // Seam ticks at the top and bottom edges, between the two halves.
    for img in [&mut strip, &mut diff] {
        for dy in 0..3.min(h) {
            img.set(width as usize - 1, dy, INK);
            img.set(width as usize, dy, INK);
            img.set(width as usize - 1, h - 1 - dy, INK);
            img.set(width as usize, h - 1 - dy, INK);
        }
    }
    // The seam step per row, as GeneratorGameTests.periodicAcrossSeam judges it. A row with a
    // seam column the diff excludes (carved, a structure, ...) is skipped and counted, as the diff
    // skips it: on 2026-10-28 three rows at z 71 to 73 failed where a cave crossed the seam under a
    // sand surface and the sand above it had fallen in on one side.
    let excluded = |x: i32, z: i32| {
        wd.is_some_and(|wd| surf.exclusion(&world, x, z, wd.expected_top(x, z)).is_some())
    };
    let mut mismatches = Vec::new();
    let (mut rows, mut rows_excluded) = (0, 0);
    for z in z0..=z1 {
        let t = |x: i32| world.full_chunk(x, z).and(world.terrain_top(x, z));
        if let (Some(e), Some(e2), Some(wv), Some(w2)) =
            (t(half - 1), t(half - 2), t(-half), t(-half + 1))
        {
            if [half - 1, half - 2, -half, -half + 1]
                .into_iter()
                .any(|x| excluded(x, z))
            {
                rows_excluded += 1;
                continue;
            }
            rows += 1;
            let step = (e - wv).abs();
            if step > (e - e2).abs().max((wv - w2).abs()).max(1) {
                mismatches.push(json!({"z": z, "east": e, "west": wv}));
            }
        }
    }
    let strip_png = out.png(&strip, "seam.png")?;
    let diff_png = if wd.is_some() {
        Some(out.png(&diff, "seam-diff.png")?)
    } else {
        None
    };
    let mut v = if wd.is_some() {
        stats.summary()
    } else {
        json!({})
    };
    let summary = json!({
        "command": "seam",
        "definition": format!("the {width} columns west of x = C/2 (x = {}..{}) then the {width} east of it (x = {}..{}), z = {z0}..{z1}, north up; seam.png is the top-down view, seam-diff.png the height difference (colours as compare's map). {HEIGHT_DEFINITION}", half - width, half - 1, -half, -half + width - 1),
        "circumference": span,
        "z_from": z0, "z_to": z1,
        "world": world_summary(&world),
        "png": strip_png,
        "diff_png": diff_png,
        "seam_rows": rows,
        "seam_rows_excluded": rows_excluded,
        "seam_mismatches": mismatches.len(),
        "seam_mismatch_rows": mismatches.iter().take(20).collect::<Vec<_>>(),
        "seam_rule": "a row mismatches when |h(C/2-1) - h(-C/2)| > max(|h(C/2-1) - h(C/2-2)|, |h(-C/2) - h(-C/2+1)|, 1), h = the terrain surface; a row with one of the four columns excluded (the exclusion rules) is skipped and counted in seam_rows_excluded",
        "per_column": per_column,
        "exclusions": {"rules": EXCLUSION_RULES, "params": opts.to_json()},
    });
    if let (Value::Object(dst), Value::Object(src)) = (&mut v, summary) {
        for (k, val) in src {
            dst.insert(k, val);
        }
    }
    out.json(&v, "seam.json")?;
    Ok(v)
}

/// Lists region files and counts those outside the canonical span −C/1024 ≤ rx < C/1024.
pub fn regions(world_dir: &Path, span: i64, stdout: &mut Vec<String>) -> Result<Value> {
    let dir = world_dir.join("region");
    let list = region::list_regions(&dir)?;
    let half = span / 1024;
    let (mut outside, mut outside_empty, mut chunks_total) = (Vec::new(), Vec::new(), 0usize);
    let mut rows = Vec::new();
    for (rx, rz, path) in &list {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let chunks = RegionFile::open(path)
            .map(|r| r.present().len())
            .unwrap_or(0);
        chunks_total += chunks;
        let inside = (-half..half).contains(&i64::from(*rx));
        if !inside {
            if chunks == 0 {
                outside_empty.push(name.clone());
            } else {
                outside.push(name.clone());
            }
        }
        stdout.push(format!(
            "{name} chunks={chunks} bytes={bytes}{}",
            if inside {
                ""
            } else if chunks == 0 {
                " OUTSIDE (empty)"
            } else {
                " OUTSIDE"
            }
        ));
        rows.push(json!({"file": name, "rx": rx, "rz": rz, "chunks": chunks, "bytes": bytes, "inside": inside}));
    }
    stdout.push(format!(
        "regions={} chunks={chunks_total} outside={} outside_empty={}",
        list.len(),
        outside.len(),
        outside_empty.len()
    ));
    stdout.push(format!("outside={}", outside.len()));
    Ok(json!({
        "command": "regions",
        "circumference": span,
        "rule": "a region file r.<rx>.<rz>.mca is inside the canonical span when -C/1024 <= rx < C/1024; outside counts files outside it that hold at least one chunk, outside_empty those that hold none (the game creates the file on a mere lookup)",
        "regions": rows,
        "outside": outside,
        "outside_empty": outside_empty,
    }))
}

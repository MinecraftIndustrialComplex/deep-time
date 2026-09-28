// SPDX-License-Identifier: LGPL-3.0-or-later
//! Block colours: `colors` builds a table from the jars' own assets (the average colour of each
//! block's top texture), and every render looks colours up in it.
//!
//! For each `assets/<ns>/blockstates/<block>.json`: pick one model (variant `""`, else one with
//! `axis=y`, else the first; for multipart, the first part without a condition), follow its
//! `parent` chain collecting `textures`, take the `up` face of the highest element (else the
//! first face, else `particle`), resolve `#` references, and average that texture's opaque
//! pixels weighted by alpha. Faces with a `tintindex` get the default biome tint (grass,
//! foliage; spruce, birch and mangrove leaves their fixed colours; water and lily pads theirs).
//! Mods' assets override the vanilla jar's, as resource packs do.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::blocks;
use crate::error::{Error, Result};
use crate::image::{self, Rgba, decode_png, hex};
use crate::zip::Zip;

/// Block id → colour, with a fallback for blocks the table does not know.
#[derive(Default)]
pub struct ColorTable {
    pub map: HashMap<String, Rgba>,
}

impl ColorTable {
    /// Reads a table written by `colors` (`{"blocks": {"ns:id": "#rrggbb[aa]"}}`).
    pub fn load(path: &Path) -> Result<ColorTable> {
        let text = std::fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
        let v: Value = serde_json::from_str(&text)?;
        let blocks = v
            .get("blocks")
            .and_then(Value::as_object)
            .ok_or_else(|| Error(format!("{}: no \"blocks\" object", path.display())))?;
        let map = blocks
            .iter()
            .filter_map(|(k, v)| Some((k.clone(), image::parse_hex(v.as_str()?)?)))
            .collect();
        Ok(ColorTable { map })
    }

    /// `--colors <file>` if given, else the built-in fallback only.
    pub fn from_arg(path: Option<&Path>) -> Result<ColorTable> {
        path.map_or_else(|| Ok(ColorTable::default()), ColorTable::load)
    }

    /// The colour of block `name`, opaque: the table's, else [`fallback`].
    pub fn color(&self, name: &str) -> Rgba {
        let c = self
            .map
            .get(name)
            .copied()
            .unwrap_or_else(|| fallback(name));
        [c[0], c[1], c[2], 255]
    }
}

/// A plausible colour for blocks the table lacks: a few common ones by hand, the rest from the
/// name's class, else a muted colour hashed from the name.
pub fn fallback(name: &str) -> Rgba {
    let path = blocks::path_of(name);
    let known: &[(&str, u32)] = &[
        ("stone", 0x7d7d7d),
        ("deepslate", 0x505053),
        ("dirt", 0x866043),
        ("coarse_dirt", 0x77553b),
        ("grass_block", 0x7fb238),
        ("sand", 0xdbcfa3),
        ("red_sand", 0xbe6621),
        ("sandstone", 0xd8cb9b),
        ("red_sandstone", 0xb5621f),
        ("gravel", 0x837f7e),
        ("granite", 0x956755),
        ("diorite", 0xbcbcbc),
        ("andesite", 0x888888),
        ("tuff", 0x6c6d66),
        ("calcite", 0xdfe0dc),
        ("clay", 0xa0a7b3),
        ("water", 0x3f76e4),
        ("lava", 0xcf5b13),
        ("snow", 0xf9fefe),
        ("snow_block", 0xf9fefe),
        ("ice", 0x91b7fd),
        ("packed_ice", 0x8db4fa),
        ("bedrock", 0x555555),
        ("obsidian", 0x0f0b19),
        ("smooth_basalt", 0x484850),
        ("mud", 0x3c393d),
        ("terracotta", 0x985e43),
        ("cactus", 0x5a8a2f),
        ("short_grass", 0x6f9e3c),
        ("seagrass", 0x2e7d32),
    ];
    if let Some((_, v)) = known.iter().find(|(k, _)| *k == path) {
        return [(v >> 16) as u8, (v >> 8) as u8, *v as u8, 255];
    }
    if path.ends_with("_leaves") {
        return [0x4a, 0x7a, 0x21, 255];
    }
    if path.ends_with("_log") || path.ends_with("_wood") {
        return [0x6b, 0x54, 0x33, 255];
    }
    if path.contains("coral") {
        return [0xd8, 0x5a, 0x8f, 255];
    }
    // FNV-1a of the name → a muted hue.
    let mut h: u32 = 0x811c_9dc5;
    for b in name.bytes() {
        h = (h ^ u32::from(b)).wrapping_mul(0x0100_0193);
    }
    let c = |s: u32| 90 + ((h >> s) & 0x7f) as u8;
    [c(0), c(8), c(16), 255]
}

/// Default tints (plains grass and foliage; the fixed leaf colours).
const GRASS: u32 = 0x91bd59;
const FOLIAGE: u32 = 0x77ab2f;
const WATER: u32 = 0x3f76e4;

fn tint_for(path: &str) -> u32 {
    match path {
        "spruce_leaves" => 0x619961,
        "birch_leaves" => 0x80a755,
        "mangrove_leaves" => 0x8db127,
        "water" | "bubble_column" => WATER,
        "lily_pad" => 0x208030,
        p if p.ends_with("_leaves") || p == "vine" => FOLIAGE,
        _ => GRASS,
    }
}

fn apply_tint(c: Rgba, tint: u32) -> Rgba {
    let t = [(tint >> 16) as u8, (tint >> 8) as u8, tint as u8];
    let m = |a: u8, b: u8| ((u16::from(a) * u16::from(b)) / 255) as u8;
    [m(c[0], t[0]), m(c[1], t[1]), m(c[2], t[2]), c[3]]
}

/// The assets of several jars, later sources overriding earlier ones.
struct Assets {
    zips: Vec<Zip>,
    index: HashMap<String, usize>,
}

impl Assets {
    fn new() -> Assets {
        Assets {
            zips: Vec::new(),
            index: HashMap::new(),
        }
    }

    fn add(&mut self, zip: Zip) {
        let i = self.zips.len();
        let mut nested = Vec::new();
        for name in zip.names() {
            if name.starts_with("assets/") {
                self.index.insert(name.to_owned(), i);
            } else if name.starts_with("META-INF/jarjar/") && name.ends_with(".jar") {
                nested.push(name.to_owned());
            }
        }
        let label = zip.name.clone();
        let mut inner = Vec::new();
        for n in nested {
            if let Ok(bytes) = zip.read(&n)
                && let Ok(z) = Zip::from_vec(format!("{label}!{n}"), bytes)
            {
                inner.push(z);
            }
        }
        self.zips.push(zip);
        for z in inner {
            if z.names().any(|n| n.starts_with("assets/")) {
                self.add(z);
            }
        }
    }

    fn read(&self, path: &str) -> Option<Vec<u8>> {
        let i = *self.index.get(path)?;
        self.zips[i].read(path).ok()
    }

    fn json(&self, path: &str) -> Option<Value> {
        serde_json::from_slice(&self.read(path)?).ok()
    }
}

/// `ns:path` or `path` (namespace minecraft) → (ns, path).
fn split_id(id: &str) -> (&str, &str) {
    id.split_once(':').unwrap_or(("minecraft", id))
}

/// The model a blockstate file picks for the top view.
fn pick_model(state: &Value) -> Option<String> {
    let model_of = |v: &Value| -> Option<String> {
        let v = if let Some(a) = v.as_array() {
            a.first()?
        } else {
            v
        };
        v.get("model")?.as_str().map(str::to_owned)
    };
    if let Some(vars) = state.get("variants").and_then(Value::as_object) {
        let pick = vars
            .get("")
            .or_else(|| vars.get("normal"))
            .or_else(|| {
                vars.iter()
                    .find(|(k, _)| k.split(',').any(|kv| kv == "axis=y"))
                    .map(|(_, v)| v)
            })
            .or_else(|| vars.values().next())?;
        return model_of(pick);
    }
    let parts = state.get("multipart").and_then(Value::as_array)?;
    parts
        .iter()
        .find(|p| p.get("when").is_none())
        .or_else(|| parts.first())
        .and_then(|p| model_of(p.get("apply")?))
}

/// The top texture of `model` and whether its face is tinted.
fn top_texture(assets: &Assets, model: &str) -> Option<(String, bool)> {
    let mut textures: HashMap<String, String> = HashMap::new();
    let mut elements: Option<Value> = None;
    let mut current = model.to_owned();
    for _ in 0..32 {
        let (ns, path) = split_id(&current);
        let Some(m) = assets.json(&format!("assets/{ns}/models/{path}.json")) else {
            break;
        };
        if let Some(t) = m.get("textures").and_then(Value::as_object) {
            for (k, v) in t {
                if let Some(s) = v.as_str() {
                    textures.entry(k.clone()).or_insert_with(|| s.to_owned());
                }
            }
        }
        if elements.is_none() {
            elements = m.get("elements").cloned();
        }
        match m.get("parent").and_then(Value::as_str) {
            Some(p) if !p.starts_with("builtin/") => current = p.to_owned(),
            _ => break,
        }
    }
    let (reference, tinted) = elements
        .as_ref()
        .and_then(Value::as_array)
        .and_then(|els| {
            let top_face = |e: &Value| {
                let f = e.get("faces")?.get("up")?;
                let to_y = e.get("to")?.get(1)?.as_f64()?;
                let from = e.get("from")?;
                let to = e.get("to")?;
                let area = (to.get(0)?.as_f64()? - from.get(0)?.as_f64()?).abs()
                    * (to.get(2)?.as_f64()? - from.get(2)?.as_f64()?).abs();
                Some((to_y, area, f.clone()))
            };
            let best = els
                .iter()
                .filter_map(top_face)
                .max_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
            let face = match best {
                Some((_, _, f)) => f,
                None => els
                    .iter()
                    .find_map(|e| e.get("faces")?.as_object()?.values().next().cloned())?,
            };
            Some((
                face.get("texture")?.as_str()?.to_owned(),
                face.get("tintindex").is_some(),
            ))
        })
        .or_else(|| {
            textures
                .get("particle")
                .map(|p| (p.clone(), false))
                .or_else(|| textures.values().next().map(|p| (p.clone(), false)))
        })?;
    // Resolve #references.
    let mut r = reference;
    for _ in 0..16 {
        match r.strip_prefix('#') {
            Some(key) => r = textures.get(key)?.clone(),
            None => return Some((r, tinted)),
        }
    }
    None
}

/// Alpha-weighted mean of the pixels with alpha > 0; the alpha is the mean coverage.
fn average(img: &image::Image) -> Option<Rgba> {
    let (mut r, mut g, mut b, mut a, mut n) = (0f64, 0f64, 0f64, 0f64, 0usize);
    for p in &img.px {
        if p[3] == 0 {
            continue;
        }
        let w = f64::from(p[3]);
        r += f64::from(p[0]) * w;
        g += f64::from(p[1]) * w;
        b += f64::from(p[2]) * w;
        a += w;
        n += 1;
    }
    if a == 0.0 {
        return None;
    }
    let cov = (n as f64 / img.px.len() as f64 * 255.0).round() as u8;
    Some([
        (r / a).round() as u8,
        (g / a).round() as u8,
        (b / a).round() as u8,
        cov.max(1),
    ])
}

/// Builds the colour table from `vanilla` (Minecraft's client resources jar) and every `.jar`
/// in `mods_dir`, and writes it as JSON to `out`. Returns (resolved, unresolved) counts.
pub fn build(mods_dir: Option<&Path>, vanilla: &Path, out: &Path) -> Result<(usize, usize)> {
    let mut assets = Assets::new();
    let mut sources = vec![vanilla.display().to_string()];
    assets.add(Zip::open(vanilla)?);
    if let Some(dir) = mods_dir {
        let mut jars: Vec<PathBuf> = std::fs::read_dir(dir)
            .map_err(|e| Error::io(dir, e))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "jar"))
            .collect();
        jars.sort();
        for j in jars {
            match Zip::open(&j) {
                Ok(z) => {
                    sources.push(j.display().to_string());
                    assets.add(z);
                }
                Err(e) => eprintln!("colors: skipping {}: {e}", j.display()),
            }
        }
    }
    let mut states: Vec<(String, String)> = assets
        .index
        .keys()
        .filter_map(|k| {
            let rest = k.strip_prefix("assets/")?;
            let (ns, file) = rest.split_once("/blockstates/")?;
            let block = file.strip_suffix(".json")?;
            Some((format!("{ns}:{block}"), k.clone()))
        })
        .collect();
    states.sort();
    let mut table = BTreeMap::new();
    let mut unresolved = Vec::new();
    let mut texture_cache: HashMap<String, Option<Rgba>> = HashMap::new();
    for (block, file) in &states {
        let color = assets
            .json(file)
            .as_ref()
            .and_then(pick_model)
            .and_then(|m| top_texture(&assets, &m))
            .and_then(|(tex, tinted)| {
                let (ns, path) = split_id(&tex);
                let key = format!("assets/{ns}/textures/{path}.png");
                let avg = *texture_cache.entry(key.clone()).or_insert_with(|| {
                    assets
                        .read(&key)
                        .and_then(|b| decode_png(&b).ok())
                        .and_then(|img| average(&img))
                });
                let p = blocks::path_of(block);
                avg.map(|c| {
                    if tinted || matches!(p, "water" | "bubble_column") {
                        apply_tint(c, tint_for(p))
                    } else {
                        c
                    }
                })
            });
        match color {
            Some(c) => {
                table.insert(block.clone(), hex(c));
            }
            None => unresolved.push(block.clone()),
        }
    }
    let doc = json!({
        "generator": "deeptime-inspect colors",
        "method": "average of the top face texture (alpha-weighted), default biome tints",
        "sources": sources,
        "tints": {"grass": format!("#{GRASS:06x}"), "foliage": format!("#{FOLIAGE:06x}"), "water": format!("#{WATER:06x}")},
        "blocks": table,
        "unresolved": unresolved,
    });
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
    }
    std::fs::write(out, serde_json::to_string_pretty(&doc)?).map_err(|e| Error::io(out, e))?;
    Ok((table.len(), unresolved.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zip::write_stored;

    fn e<'a>(path: &'a str, bytes: &'a [u8]) -> (&'a str, &'a [u8]) {
        (path, bytes)
    }

    fn png(w: u32, h: u32, px: &[[u8; 4]]) -> Vec<u8> {
        let mut out = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut out, w, h);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            let mut wr = enc.write_header().unwrap();
            let flat: Vec<u8> = px.iter().flatten().copied().collect();
            wr.write_image_data(&flat).unwrap();
        }
        out
    }

    #[test]
    fn builds_a_table_from_jars() {
        let dir = std::env::temp_dir().join(format!("dt-inspect-colors-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("mods")).unwrap();
        // "vanilla": a cube_all parent chain, a grass-like tinted top, a cross plant, water.
        let cube = br##"{"elements":[{"from":[0,0,0],"to":[16,16,16],"faces":{"up":{"texture":"#up"},"north":{"texture":"#north"}}}]}"##;
        let cube_all = br##"{"parent":"block/cube","textures":{"up":"#all","north":"#all","particle":"#all"}}"##;
        let stone =
            br#"{"parent":"minecraft:block/cube_all","textures":{"all":"minecraft:block/stone"}}"#;
        let grass = br#"{"elements":[{"from":[0,0,0],"to":[16,16,16],"faces":{"up":{"texture":"block/grass_top","tintindex":0}}}]}"#;
        let red = png(2, 1, &[[200, 0, 0, 255], [100, 0, 0, 255]]);
        let white = png(1, 2, &[[255, 255, 255, 255], [0, 0, 0, 0]]);
        let vanilla = write_stored(&[
            e("assets/minecraft/blockstates/stone.json", br#"{"variants":{"":{"model":"minecraft:block/stone"}}}"#),
            e("assets/minecraft/blockstates/grass_block.json", br#"{"variants":{"snowy=false":[{"model":"block/grass"}],"snowy=true":{"model":"block/stone"}}}"#),
            e("assets/minecraft/blockstates/water.json", br#"{"variants":{"level=0":{"model":"block/water"}}}"#),
            e("assets/minecraft/models/block/cube.json", cube),
            e("assets/minecraft/models/block/cube_all.json", cube_all),
            e("assets/minecraft/models/block/stone.json", stone),
            e("assets/minecraft/models/block/grass.json", grass),
            e("assets/minecraft/models/block/water.json", br#"{"textures":{"particle":"block/water_still"}}"#),
            e("assets/minecraft/textures/block/stone.png", &red),
            e("assets/minecraft/textures/block/grass_top.png", &white),
            e("assets/minecraft/textures/block/water_still.png", &white),
        ]);
        let vpath = dir.join("client-extra.jar");
        std::fs::write(&vpath, vanilla).unwrap();
        // A mod overriding stone's texture and adding its own block on a vanilla parent.
        let blue = png(1, 1, &[[0, 0, 200, 255]]);
        let modjar = write_stored(&[
            e("assets/minecraft/textures/block/stone.png", &blue),
            e("assets/testmod/blockstates/rock.json", br#"{"multipart":[{"when":{"x":"1"},"apply":{"model":"block/stone"}},{"apply":{"model":"testmod:block/rock"}}]}"#),
            e("assets/testmod/models/block/rock.json", br#"{"parent":"minecraft:block/cube_all","textures":{"all":"testmod:block/rock"}}"#),
            e("assets/testmod/textures/block/rock.png", &red),
        ]);
        std::fs::write(dir.join("mods/testmod.jar"), modjar).unwrap();

        let out = dir.join("colors.json");
        let (ok, missing) = build(Some(&dir.join("mods")), &vpath, &out).unwrap();
        assert_eq!((ok, missing), (4, 0));
        let t = ColorTable::load(&out).unwrap();
        assert_eq!(
            t.map["minecraft:stone"],
            [0, 0, 200, 255],
            "mods override vanilla"
        );
        assert_eq!(
            t.map["testmod:rock"],
            [150, 0, 0, 255],
            "parent chain and average"
        );
        // White top, half transparent, tinted with the grass colour.
        assert_eq!(t.map["minecraft:grass_block"], [0x91, 0xbd, 0x59, 128]);
        assert_eq!(t.map["minecraft:water"], [0x3f, 0x76, 0xe4, 128]);
        assert_eq!(t.color("minecraft:grass_block")[3], 255);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn fallback_is_stable() {
        assert_eq!(fallback("minecraft:stone"), [0x7d, 0x7d, 0x7d, 255]);
        assert_eq!(fallback("x:unknown"), fallback("x:unknown"));
    }
}

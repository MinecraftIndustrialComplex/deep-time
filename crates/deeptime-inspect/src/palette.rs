// SPDX-License-Identifier: LGPL-3.0-or-later
//! Block → rock class, for the `rock` render. From a palette dump (`--palette`, the format
//! `/deeptime probe palette` will write: `{"classes": {"granite": ["minecraft:granite", …]}}`, or
//! a flat `{"blocks": {"minecraft:granite": "granite"}}`), else from the `#deeptime:rock/<class>`
//! block tags found in jars, datapack zips and folders (`data/<ns>/tags/block/…`), with `#tag`
//! references resolved.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::error::{Error, Result};
use crate::zip::Zip;

pub struct Palette {
    /// Block id → rock class.
    pub blocks: HashMap<String, String>,
    /// Class → its first block (for the class colour).
    pub first: BTreeMap<String, String>,
    /// Where the mapping came from.
    pub sources: Vec<String>,
}

impl Palette {
    pub fn class_of(&self, block: &str) -> Option<&str> {
        self.blocks.get(block).map(String::as_str)
    }

    /// Reads a palette dump.
    pub fn from_dump(path: &Path) -> Result<Palette> {
        let text = std::fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
        let v: Value = serde_json::from_str(&text)?;
        let mut p = Palette {
            blocks: HashMap::new(),
            first: BTreeMap::new(),
            sources: vec![path.display().to_string()],
        };
        if let Some(classes) = v.get("classes").and_then(Value::as_object) {
            for (class, members) in classes {
                for m in members
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    p.add(class, m);
                }
            }
        } else {
            let map = v.get("blocks").unwrap_or(&v);
            for (block, class) in map.as_object().into_iter().flatten() {
                if let Some(c) = class.as_str() {
                    p.add(c, block);
                }
            }
        }
        Ok(p)
    }

    fn add(&mut self, class: &str, block: &str) {
        self.blocks
            .entry(block.to_owned())
            .or_insert_with(|| class.to_owned());
        self.first
            .entry(class.to_owned())
            .or_insert_with(|| block.to_owned());
    }

    /// Builds the palette from the rock tags in `sources` (jars, zips or folders).
    pub fn from_tags(sources: &[PathBuf]) -> Result<Palette> {
        // tag id → raw values, later sources appending (tags merge unless "replace").
        let mut tags: HashMap<String, Vec<String>> = HashMap::new();
        let mut used = Vec::new();
        for src in sources {
            let mut found = Vec::new();
            if src.is_dir() {
                collect_dir(src, src, &mut found)?;
            } else if let Ok(zip) = Zip::open(src) {
                let names: Vec<String> = zip
                    .names()
                    .filter(|n| is_block_tag(n))
                    .map(str::to_owned)
                    .collect();
                for n in names {
                    if let Ok(b) = zip.read(&n) {
                        found.push((n, b));
                    }
                }
            }
            if found.is_empty() {
                continue;
            }
            used.push(src.display().to_string());
            for (name, bytes) in found {
                let Some(id) = tag_id(&name) else { continue };
                let Ok(v) = serde_json::from_slice::<Value>(&bytes) else {
                    continue;
                };
                let values: Vec<String> = v
                    .get("values")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|e| {
                        e.as_str()
                            .or_else(|| e.get("id").and_then(Value::as_str))
                            .map(str::to_owned)
                    })
                    .collect();
                let entry = tags.entry(id).or_default();
                if v.get("replace").and_then(Value::as_bool) == Some(true) {
                    entry.clear();
                }
                entry.extend(values);
            }
        }
        let mut p = Palette {
            blocks: HashMap::new(),
            first: BTreeMap::new(),
            sources: used,
        };
        let mut rock: Vec<&String> = tags
            .keys()
            .filter(|k| k.starts_with("deeptime:rock/"))
            .collect();
        rock.sort();
        for id in rock {
            let class = id.trim_start_matches("deeptime:rock/");
            for b in resolve(&tags, id, 0) {
                p.add(class, &b);
            }
        }
        Ok(p)
    }
}

/// `data/<ns>/tags/block/<path>.json` (1.21 folder name; `blocks` before 1.21).
fn is_block_tag(name: &str) -> bool {
    tag_id(name).is_some()
}

/// The tag id of a tag file path; the path may sit below a datapack folder (`<pack>/data/…`).
fn tag_id(name: &str) -> Option<String> {
    let at = if name.starts_with("data/") {
        0
    } else {
        name.find("/data/")? + 1
    };
    let rest = &name[at + "data/".len()..];
    let (ns, rest) = rest.split_once('/')?;
    let path = rest
        .strip_prefix("tags/block/")
        .or_else(|| rest.strip_prefix("tags/blocks/"))?
        .strip_suffix(".json")?;
    Some(format!("{ns}:{path}"))
}

fn collect_dir(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Ok(());
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_dir(root, &p, out)?;
        } else if let Ok(rel) = p.strip_prefix(root) {
            let rel = rel.to_string_lossy().replace('\\', "/");
            if is_block_tag(&rel) {
                out.push((rel, std::fs::read(&p).map_err(|e| Error::io(&p, e))?));
            } else if rel.ends_with(".zip") || rel.ends_with(".jar") {
                // Datapack zips inside a datapacks/ folder.
                if let Ok(zip) = Zip::open(&p) {
                    let names: Vec<String> = zip
                        .names()
                        .filter(|n| is_block_tag(n))
                        .map(str::to_owned)
                        .collect();
                    for n in names {
                        if let Ok(b) = zip.read(&n) {
                            out.push((n, b));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn resolve(tags: &HashMap<String, Vec<String>>, id: &str, depth: usize) -> Vec<String> {
    if depth > 16 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for v in tags.get(id).into_iter().flatten() {
        if let Some(inner) = v.strip_prefix('#') {
            out.extend(resolve(tags, inner, depth + 1));
        } else if v.contains(':') {
            out.push(v.clone());
        } else {
            out.push(format!("minecraft:{v}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_and_dumps() {
        let dir = std::env::temp_dir().join(format!("dt-inspect-palette-{}", std::process::id()));
        let rock = dir.join("data/deeptime/tags/block/rock");
        let c = dir.join("data/c/tags/block");
        std::fs::create_dir_all(&rock).unwrap();
        std::fs::create_dir_all(&c).unwrap();
        std::fs::write(
            rock.join("granite.json"),
            r#"{"values":["minecraft:granite",{"id":"x:pink","required":false}]}"#,
        )
        .unwrap();
        std::fs::write(rock.join("basalt.json"), r##"{"values":["#c:basalts"]}"##).unwrap();
        std::fs::write(c.join("basalts.json"), r#"{"values":["smooth_basalt"]}"#).unwrap();
        let p = Palette::from_tags(std::slice::from_ref(&dir)).unwrap();
        assert_eq!(p.class_of("minecraft:granite"), Some("granite"));
        assert_eq!(p.class_of("x:pink"), Some("granite"));
        assert_eq!(p.class_of("minecraft:smooth_basalt"), Some("basalt"));
        assert_eq!(p.first["granite"], "minecraft:granite");

        let dump = dir.join("palette.json");
        std::fs::write(&dump, r#"{"classes":{"tuff":["minecraft:tuff"]}}"#).unwrap();
        assert_eq!(
            Palette::from_dump(&dump)
                .unwrap()
                .class_of("minecraft:tuff"),
            Some("tuff")
        );
        std::fs::write(&dump, r#"{"blocks":{"minecraft:mud":"mud"}}"#).unwrap();
        assert_eq!(
            Palette::from_dump(&dump).unwrap().class_of("minecraft:mud"),
            Some("mud")
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}

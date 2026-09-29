// SPDX-License-Identifier: LGPL-3.0-or-later
//! The world data a render is checked against (`crates/deeptime-format/FORMAT.md`), sampled the
//! way the mod samples it: `elevation` bilinear with x wrapped into the canonical span
//! (`WorldData.height`), `water_level` nearest, and the strata stack of the nearest L1 cell
//! (`StrataPass`).

use std::path::{Path, PathBuf};

use deeptime_format::{LoadedLayer, Manifest, RasterRead, TilePackReader};

use crate::error::{Error, Result};

pub struct WorldData {
    pub dir: PathBuf,
    pub manifest: Manifest,
    pub circumference: i64,
    elevation: LoadedLayer,
    water: Option<LoadedLayer>,
    strata: Option<LoadedLayer>,
    pub classes: Vec<String>,
}

fn load_layer(dir: &Path, m: &Manifest, name: &str) -> Result<Option<LoadedLayer>> {
    let Some(entry) = m.layer(name) else {
        return Ok(None);
    };
    let path = dir.join(&entry.path);
    Ok(Some(TilePackReader::open(&path)?.load()?))
}

impl WorldData {
    pub fn open(dir: &Path) -> Result<WorldData> {
        let manifest = Manifest::read(dir)?;
        let elevation = load_layer(dir, &manifest, "elevation")?
            .ok_or_else(|| Error(format!("{}: no elevation layer", dir.display())))?;
        let water = load_layer(dir, &manifest, "water_level")?;
        let strata = load_layer(dir, &manifest, "strata/horizons")?;
        let classes = std::fs::read_to_string(dir.join("strata/classes.json"))
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| {
                v.get("classes")?.as_array().map(|a| {
                    a.iter()
                        .map(|c| c.as_str().unwrap_or("?").to_owned())
                        .collect()
                })
            })
            .unwrap_or_default();
        let circumference = i64::from(manifest.scale.circumference_blocks);
        Ok(WorldData {
            dir: dir.to_owned(),
            manifest,
            circumference,
            elevation,
            water,
            strata,
            classes,
        })
    }

    /// Block x folded into the canonical span [−C/2, C/2).
    pub fn wrap_x(&self, x: i64) -> i64 {
        let c = self.circumference;
        (x + c / 2).rem_euclid(c) - c / 2
    }

    /// The terrain height H at column (x, z): bilinear `elevation`, NaN where it has no data.
    pub fn height(&self, x: i32, z: i32) -> f64 {
        self.elevation
            .bilinear(self.wrap_x(i64::from(x)) as f64, f64::from(z))
    }

    /// The top solid block the generator must produce for H: `⌈H⌉ − 1` (the density is positive
    /// below H), the value the GameTests compare `OCEAN_FLOOR_WG` with.
    pub fn expected_top(&self, x: i32, z: i32) -> Option<i32> {
        let h = self.height(x, z);
        h.is_finite().then(|| h.ceil() as i32 - 1)
    }

    /// The water surface above the terrain (nearest cell), if the world data has one there.
    pub fn water_level(&self, x: i32, z: i32) -> Option<f64> {
        let w = self.water.as_ref()?;
        let v = w.nearest(self.wrap_x(i64::from(x)) as f64, f64::from(z));
        v.is_finite().then_some(v)
    }

    /// The horizon stack at (x, z) as `StrataPass` reads it: the nearest L1 cell, horizons
    /// top-down as (top in blocks, class name), stopping at the first unused one.
    pub fn horizons(&self, x: i32, z: i32) -> Vec<(f64, String)> {
        let Some(s) = &self.strata else {
            return Vec::new();
        };
        let g = &s.desc.grid;
        let wx = self.wrap_x(i64::from(x)) as f64;
        // Java's Math.round: ⌊v + 0.5⌋.
        let i = ((wx - g.origin_block_x as f64) / f64::from(g.r) + 0.5).floor() as i64;
        let j = ((f64::from(z) - g.origin_block_z as f64) / f64::from(g.r) + 0.5).floor() as i64;
        let (i, j) = (g.wrap_i(i), g.clamp_j(j));
        let k = s.desc.stack_depth.unwrap_or(0);
        let mut out = Vec::new();
        for h in 0..k {
            let (top, class) = s.horizon(i, j, h);
            if class == 255 || top == i16::MIN {
                break;
            }
            let name = self
                .classes
                .get(usize::from(class))
                .cloned()
                .unwrap_or_else(|| format!("class{class}"));
            out.push((f64::from(top) * s.desc.scale + s.desc.offset, name));
        }
        out
    }
}

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

    /// Horizons per column in the strata layer (its `stack_depth`), 0 without one.
    pub fn stack_depth(&self) -> usize {
        self.strata
            .as_ref()
            .map_or(0, |s| s.desc.stack_depth.unwrap_or(0) as usize)
    }

    /// Horizon `k`'s top at cell (i, j) in blocks, `RasterSampler.horizonTop`: i wraps, j clamps,
    /// NaN where the horizon is unused.
    fn horizon_top(s: &deeptime_format::LoadedLayer, i: i64, j: i64, k: u32) -> f64 {
        let g = &s.desc.grid;
        let (top, _) = s.horizon(g.wrap_i(i), g.clamp_j(j), k);
        if top == i16::MIN {
            f64::NAN
        } else {
            f64::from(top) * s.desc.scale + s.desc.offset
        }
    }

    /// One column's horizons as `StrataPass.column` (v1, P1-12) builds them: tops bilinear
    /// between the four surrounding cells (the nearest cell's top where a corner lacks the
    /// horizon), classes from the nearest cell, integer tops ⌊top⌋ that never rise with depth.
    /// Top-down (top y, class index); horizon h fills tops[h+1] < y ≤ tops[h], the last one to
    /// the bottom of the world. The operations follow the Java line by line (same rounding).
    pub fn strata_column(&self, x: i32, z: i32) -> Vec<(i32, u8)> {
        let Some(s) = &self.strata else {
            return Vec::new();
        };
        let g = &s.desc.grid;
        let c = self.circumference;
        let k = s.desc.stack_depth.unwrap_or(0);
        // Java: Math.floorMod(x + c / 2, c) - c / 2 in int arithmetic, then widened.
        let wx = ((i64::from(x) + c / 2).rem_euclid(c) - c / 2) as f64;
        let u = (wx - g.origin_block_x as f64) / f64::from(g.r);
        let v = (f64::from(z) - g.origin_block_z as f64) / f64::from(g.r);
        let (i_n, j_n) = ((u + 0.5).floor() as i64, (v + 0.5).floor() as i64);
        let (i0, j0) = (u.floor() as i64, v.floor() as i64);
        let (fu, fv) = (u - i0 as f64, v - j0 as f64);
        let mut out = Vec::new();
        let mut prev = i32::MAX;
        for h in 0..k {
            let (_, cls) = s.horizon(g.wrap_i(i_n), g.clamp_j(j_n), h);
            let nearest = Self::horizon_top(s, i_n, j_n, h);
            if cls == 255 || nearest.is_nan() {
                break;
            }
            let t00 = Self::horizon_top(s, i0, j0, h);
            let t10 = Self::horizon_top(s, i0 + 1, j0, h);
            let t01 = Self::horizon_top(s, i0, j0 + 1, h);
            let t11 = Self::horizon_top(s, i0 + 1, j0 + 1, h);
            let top = if t00.is_nan() || t10.is_nan() || t01.is_nan() || t11.is_nan() {
                nearest
            } else {
                (t00 + (t10 - t00) * fu) * (1.0 - fv) + (t01 + (t11 - t01) * fu) * fv
            };
            // Java's (int) Math.floor saturates like Rust's `as`.
            let t = (top.floor() as i32).min(prev);
            out.push((t, cls));
            prev = t;
        }
        out
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

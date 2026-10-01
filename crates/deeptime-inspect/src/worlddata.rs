// SPDX-License-Identifier: LGPL-3.0-or-later
//! The world data a render is checked against (`crates/deeptime-format/FORMAT.md`), sampled the
//! way the mod samples it: the terrain height with x wrapped into the canonical span
//! (`WorldData.height`: `elevation` bilinear plus, in a baked world, the detail term),
//! `water_level` nearest, and the strata stack of the nearest L1 cell (`StrataPass`).

use std::path::{Path, PathBuf};

use deeptime_format::{DetailSpec, LoadedLayer, Manifest, Noise, RasterRead, TilePackReader};

use crate::error::{Error, Result};

pub struct WorldData {
    pub dir: PathBuf,
    pub manifest: Manifest,
    pub circumference: i64,
    elevation: LoadedLayer,
    /// The detail term of a baked world (format 0.1): its spec, noise and amplitude layer.
    detail: Option<(DetailSpec, Noise, LoadedLayer)>,
    water: Option<LoadedLayer>,
    strata: Option<LoadedLayer>,
    /// The world's folds and faults (P2-01), which the stacks are read through.
    structure: Option<deeptime_format::structure::Deformer>,
    pub classes: Vec<String>,
    koppen: std::sync::OnceLock<Option<LoadedLayer>>,
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
        let detail = match DetailSpec::of(&manifest)? {
            Some(spec) => {
                let amp = load_layer(dir, &manifest, &spec.layer)?
                    .ok_or_else(|| Error(format!("{}: no {} layer", dir.display(), spec.layer)))?;
                let noise = Noise::new(spec.seed()?, manifest.scale.circumference_blocks);
                Some((spec, noise, amp))
            }
            None => None,
        };
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
        let structure =
            deeptime_format::structure::Deformer::read(dir, manifest.scale.circumference_blocks)?;
        Ok(WorldData {
            structure,
            dir: dir.to_owned(),
            manifest,
            circumference,
            elevation,
            detail,
            water,
            strata,
            classes,
            koppen: std::sync::OnceLock::new(),
        })
    }

    /// Block x folded into the canonical span [−C/2, C/2).
    pub fn wrap_x(&self, x: i64) -> i64 {
        let c = self.circumference;
        (x + c / 2).rem_euclid(c) - c / 2
    }

    /// The terrain height H at column (x, z): bilinear `elevation`, plus the detail term A·N in a
    /// baked world (`deeptime_format::ColumnWorld::height`); NaN where it has no data.
    pub fn height(&self, x: i32, z: i32) -> f64 {
        let (bx, bz) = (self.wrap_x(i64::from(x)) as f64, f64::from(z));
        let e = self.elevation.bilinear(bx, bz);
        match &self.detail {
            Some((spec, noise, amp)) => e + amp.bilinear(bx, bz) * spec.noise(noise, bx, bz),
            None => e,
        }
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

    /// The Köppen class index at (x, z) (the baked `climate/koppen`, nearest cell; 31 = sea), if
    /// the world data has the layer. The layer is read on first use of any column.
    pub fn koppen(&self, x: i32, z: i32) -> Option<u8> {
        let k = self
            .koppen
            .get_or_init(|| {
                load_layer(&self.dir, &self.manifest, "climate/koppen")
                    .ok()
                    .flatten()
            })
            .as_ref()?;
        let v = k.nearest(self.wrap_x(i64::from(x)) as f64, f64::from(z));
        v.is_finite().then(|| v.round().clamp(0.0, 31.0) as u8)
    }

    /// Horizons per column in the strata layer (its `stack_depth`), 0 without one.
    pub fn stack_depth(&self) -> usize {
        self.strata
            .as_ref()
            .map_or(0, |s| s.desc.stack_depth.unwrap_or(0) as usize)
    }

    /// One column's horizons as the strata pass builds them (`StrataPass.column`, through
    /// `LoadedLayer::stack_at`, the column rule of format 0.2): top-down (top block y, class
    /// index); horizon h fills tops[h+1] < y ≤ tops[h], the last one to the bottom of the world.
    /// Empty without strata.
    pub fn strata_column(&self, x: i32, z: i32) -> Vec<(i32, u8)> {
        let mut out = Vec::new();
        if let Some(s) = &self.strata {
            let min_y = self.manifest.scale.min_y;
            match &self.structure {
                Some(d) => d.stack(
                    i64::from(x),
                    i64::from(z),
                    min_y,
                    &mut |sx, sz, o| s.stack_at(sx, sz, min_y, o),
                    &mut deeptime_format::structure::DeformScratch::default(),
                    &mut out,
                ),
                None => s.stack_at(i64::from(x), i64::from(z), min_y, &mut out),
            }
        }
        out.into_iter().map(|(t, c, _)| (t, c)).collect()
    }

    /// The horizon stack at (x, z) as `StrataPass` reads it (the column rule): horizons top-down
    /// as (top in blocks, class name).
    pub fn horizons(&self, x: i32, z: i32) -> Vec<(f64, String)> {
        self.strata_column(x, z)
            .into_iter()
            .map(|(top, class)| {
                let name = self
                    .classes
                    .get(usize::from(class))
                    .cloned()
                    .unwrap_or_else(|| format!("class{class}"));
                (f64::from(top), name)
            })
            .collect()
    }
}

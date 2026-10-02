// SPDX-License-Identifier: LGPL-3.0-or-later
//! The column evaluator (plan §6.1, §8.8, §9.2; FORMAT.md, "Columns"): what Deep Time decides for
//! one block column of a world — the terrain height, the water surface, the horizon stack, the
//! bodies the column passes through and the rock from the terrain's top block down to the floor.
//! Java's `worlddata/Columns` is its twin, operation for operation; `deeptime parity` writes 500
//! columns of a world as `parity/columns.json`, which `ColumnParityTest` compares with Java's
//! answers, and `deeptime inspect-column` and the JNI `sampleColumn` call [`ColumnWorld::sample`].
//!
//! Reads go through [`LazyLayer`]s, so a column of a 32k planet decodes only the tiles it touches.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::bodies::{Body, read_bodies};
use crate::detail::{DetailSpec, Noise};
use crate::error::{FormatError, Result};
use crate::lazy::LazyLayer;
use crate::manifest::Manifest;
use crate::raster::{DType, RasterRead};
use crate::rivers::{Rivers, read_rivers};
use crate::sites::{Site, SiteIndex, read_sites};
use crate::strata::ColumnHorizon;
use crate::structure::{DeformScratch, Deformer};
use crate::water::{self, WaterColumn, WaterSource, WaterSpec};

/// The terrain layer.
pub const ELEVATION: &str = "elevation";
/// The water surface above the terrain (nodata = none).
pub const WATER_LEVEL: &str = "water_level";
/// The horizon stacks.
pub const HORIZONS: &str = "strata/horizons";
/// The class index table of the horizon stacks.
pub const CLASSES_PATH: &str = "strata/classes.json";

/// One world-data folder, opened for column queries.
pub struct ColumnWorld {
    pub dir: PathBuf,
    pub manifest: Manifest,
    /// C, blocks.
    pub c: i64,
    pub min_y: i32,
    pub max_y: i32,
    elevation: LazyLayer,
    detail: Option<(DetailSpec, Noise, LazyLayer)>,
    water: Option<LazyLayer>,
    horizons: Option<LazyLayer>,
    /// `structure/events.dtst` (P2-01): the folds and faults the stacks are read through.
    structure: Option<Deformer>,
    /// `strata/classes.json`, index → rock class.
    pub classes: Vec<String>,
    /// Every body, ascending id.
    pub bodies: Vec<Body>,
    sites: Option<(Vec<Site>, SiteIndex)>,
    others: Vec<(String, LazyLayer)>,
    /// `rivers/rivers.dtrv` (P1-22), when the world has one.
    pub rivers: Option<Rivers>,
}

/// What the evaluator decides for one column (FORMAT.md, "Columns").
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    /// The column as asked; `wx` folded into [−C/2, C/2).
    pub x: i64,
    pub z: i64,
    pub wx: i64,
    /// E: the `elevation` layer, bilinear at (wx, z), blocks.
    pub elevation: f64,
    /// A·N: the detail term (0 without one), blocks.
    pub detail: f64,
    /// H = E + A·N, the height the terrain density is zero at.
    pub surface: f64,
    /// ⌈H⌉ − 1: the terrain's top block at a noise-cell corner column (clamped to the build range).
    pub top_y: i32,
    /// The `water_level` layer, nearest, where it has data.
    pub water: Option<f64>,
    /// (top y, class index, unit key) per horizon, top-down, as the strata pass reads the stack
    /// ([`crate::strata::stack_at`]).
    pub horizons: Vec<ColumnHorizon>,
    /// (body id, top y, bottom y) per maximal run of the column's blocks inside a body, ascending id.
    pub bodies: Vec<(u32, i32, i32)>,
    /// (top y, bottom y, rock class) from `top_y` down to `min_y`: the body's class (a zone's
    /// winning) over the horizon's; None where neither names one (unclassified crust).
    pub rock: Vec<(i32, i32, Option<String>)>,
    /// Lakes, rivers, the sea and banks at the column (P1-22, [`crate::water`]).
    pub hydro: WaterColumn,
}

fn io(path: &Path, e: std::io::Error) -> FormatError {
    FormatError::Io {
        path: path.to_owned(),
        source: e,
    }
}

impl ColumnWorld {
    /// Opens `dir`: the manifest (refusing another format version), the terrain, detail, water and
    /// strata layers, the class table and the bodies.
    pub fn open(dir: &Path) -> Result<ColumnWorld> {
        let manifest = Manifest::read(dir)?;
        let layer = |name: &str| -> Result<Option<LazyLayer>> {
            match manifest.layer(name) {
                Some(e) => LazyLayer::open(&dir.join(&e.path)).map(Some),
                None => Ok(None),
            }
        };
        let elevation = layer(ELEVATION)?.ok_or_else(|| {
            FormatError::Invalid(format!("{}: no {ELEVATION} layer", dir.display()))
        })?;
        let c = i64::from(manifest.scale.circumference_blocks);
        let detail = match DetailSpec::of(&manifest)? {
            Some(spec) => match layer(&spec.layer)? {
                Some(l) => {
                    let noise = Noise::new(spec.seed()?, manifest.scale.circumference_blocks);
                    Some((spec, noise, l))
                }
                None => {
                    return Err(FormatError::Invalid(format!(
                        "{}: notes.bake.detail names the layer {}, which the manifest lacks",
                        dir.display(),
                        spec.layer
                    )));
                }
            },
            None => None,
        };
        let horizons = layer(HORIZONS)?;
        let classes = if horizons.is_some() {
            let path = dir.join(CLASSES_PATH);
            let text = std::fs::read(&path).map_err(|e| io(&path, e))?;
            let v: Value = serde_json::from_slice(&text).map_err(|source| FormatError::Json {
                what: path.display().to_string(),
                source,
            })?;
            v["classes"]
                .as_array()
                .ok_or_else(|| FormatError::corrupt(&path, "no classes array"))?
                .iter()
                .map(|c| c.as_str().unwrap_or("?").to_owned())
                .collect()
        } else {
            Vec::new()
        };
        let mut bodies = read_bodies(dir)?.unwrap_or_default();
        bodies.sort_by_key(|b| b.id);
        let water = layer(WATER_LEVEL)?;
        let skip = [ELEVATION, WATER_LEVEL, HORIZONS];
        let mut others = Vec::new();
        for e in &manifest.layers {
            let is_detail = detail.as_ref().is_some_and(|(s, _, _)| s.layer == e.name);
            if !skip.contains(&e.name.as_str()) && !is_detail && e.dtype != DType::HorizonStack {
                others.push((e.name.clone(), LazyLayer::open(&dir.join(&e.path))?));
            }
        }
        let structure = Deformer::read(dir, manifest.scale.circumference_blocks)?;
        Ok(ColumnWorld {
            dir: dir.to_owned(),
            structure,
            sites: read_sites(dir)?,
            rivers: read_rivers(dir)?,
            min_y: manifest.scale.min_y,
            max_y: manifest.scale.max_y,
            manifest,
            c,
            elevation,
            detail,
            water,
            horizons,
            classes,
            bodies,
            others,
        })
    }

    /// Block x folded into the canonical span [−C/2, C/2).
    pub fn wrap_x(&self, x: i64) -> i64 {
        (x + self.c / 2).rem_euclid(self.c) - self.c / 2
    }

    /// (E, A·N) at block (x, z): the terrain height is their sum (FORMAT.md, "The terrain
    /// height"). Both are evaluated at the folded x.
    pub fn height_parts(&self, x: i64, z: i64) -> (f64, f64) {
        let (bx, bz) = (self.wrap_x(x) as f64, z as f64);
        let e = self.elevation.bilinear(bx, bz);
        let d = match &self.detail {
            Some((spec, noise, amp)) => amp.bilinear(bx, bz) * spec.noise(noise, bx, bz),
            None => 0.0,
        };
        (e, d)
    }

    /// The `strata/horizons` layer, when the world has one (its stored stacks, cell by cell).
    pub fn strata(&self) -> Option<&LazyLayer> {
        self.horizons.as_ref()
    }

    /// The world's folds and faults, when it has any.
    pub fn structure(&self) -> Option<&Deformer> {
        self.structure.as_ref()
    }

    /// The horizon stack of column (x, z) as the strata pass paints it: the column rule
    /// ([`crate::strata::stack_at`]) read through the world's folds and faults
    /// ([`crate::structure::Structure::deformed_stack`]). Empty without a strata layer.
    pub fn stack(&self, x: i64, z: i64, scratch: &mut DeformScratch, out: &mut Vec<ColumnHorizon>) {
        out.clear();
        let Some(h) = &self.horizons else {
            return;
        };
        match &self.structure {
            Some(d) => d.stack(
                x,
                z,
                self.min_y,
                &mut |sx, sz, o| h.stack_at(sx, sz, self.min_y, o),
                scratch,
                out,
            ),
            None => h.stack_at(x, z, self.min_y, out),
        }
    }

    /// The horizon stack of column (x, z) before any fold or fault ([`crate::strata::stack_at`]).
    pub fn plain_stack(&self, x: i64, z: i64, out: &mut Vec<ColumnHorizon>) {
        out.clear();
        if let Some(h) = &self.horizons {
            h.stack_at(x, z, self.min_y, out);
        }
    }

    /// The top block of the terrain at column (x, z) (`min_y − 1` where there is none), as
    /// [`ColumnWorld::sample`] gives it.
    pub fn top_y(&self, x: i64, z: i64) -> i32 {
        let surface = self.height(x, z);
        if surface.is_finite() {
            ((surface.ceil() as i64) - 1).clamp(i64::from(self.min_y) - 1, i64::from(self.max_y))
                as i32
        } else {
            self.min_y - 1
        }
    }

    /// The terrain height H at block (x, z).
    pub fn height(&self, x: i64, z: i64) -> f64 {
        let (e, d) = self.height_parts(x, z);
        if self.detail.is_some() { e + d } else { e }
    }

    /// The water constants of this world (FORMAT.md, "Water columns").
    pub fn water_spec(&self) -> WaterSpec {
        WaterSpec::planet(self.manifest.scale.sea_level_y, self.min_y, self.max_y)
    }

    /// The water of column (x, z) (any lap).
    pub fn hydro(&self, x: i64, z: i64) -> WaterColumn {
        water::column(self, &self.water_spec(), self.c, x, z)
    }

    /// Bodies whose AABB footprint holds column (wx, z), ascending id.
    fn bodies_at(&self, wx: i64, z: i64) -> impl Iterator<Item = &Body> {
        let c = self.c;
        self.bodies.iter().filter(move |b| {
            let [lo, hi] = b.aabb;
            if z < i64::from(lo[2]) || z > i64::from(hi[2]) {
                return false;
            }
            let span = i64::from(hi[0]) - i64::from(lo[0]);
            span >= c - 1 || (wx - i64::from(lo[0])).rem_euclid(c) <= span
        })
    }

    /// The rock class a body gives block (wx, y, z), if the block is inside and the body or one
    /// of its zones names a class there (the last zone that applies and names one wins).
    fn body_class<'b>(&self, b: &'b Body, wx: i64, y: i32, z: i64) -> Option<Option<&'b str>> {
        let at = b.locate(self.c as u32, wx as i32, y, z as i32)?;
        let mut class = b.rock_class.as_deref();
        for zone in b.zones_at(at) {
            if let Some(rc) = zone.rock_class.as_deref() {
                class = Some(rc);
            }
        }
        Some(class)
    }

    /// Evaluates column (x, z).
    pub fn sample(&self, x: i64, z: i64) -> Column {
        let wx = self.wrap_x(x);
        let (elevation, detail) = self.height_parts(x, z);
        let surface = if self.detail.is_some() {
            elevation + detail
        } else {
            elevation
        };
        let top_y = if surface.is_finite() {
            ((surface.ceil() as i64) - 1).clamp(i64::from(self.min_y) - 1, i64::from(self.max_y))
                as i32
        } else {
            self.min_y - 1
        };
        let water = self
            .water
            .as_ref()
            .map(|w| w.nearest(wx as f64, z as f64))
            .filter(|v| !v.is_nan());
        let mut horizons = Vec::new();
        self.stack(x, z, &mut DeformScratch::default(), &mut horizons);
        let near: Vec<&Body> = self.bodies_at(wx, z).collect();

        let mut bodies = Vec::new();
        for b in &near {
            let (lo, hi) = (b.aabb[0][1].max(self.min_y), b.aabb[1][1].min(self.max_y));
            let mut run: Option<(i32, i32)> = None;
            for y in (lo..=hi).rev() {
                let inside = b.locate(self.c as u32, wx as i32, y, z as i32).is_some();
                match (&mut run, inside) {
                    (Some((_, bottom)), true) => *bottom = y,
                    (None, true) => run = Some((y, y)),
                    (Some(r), false) => {
                        bodies.push((b.id, r.0, r.1));
                        run = None;
                    }
                    (None, false) => {}
                }
            }
            if let Some(r) = run {
                bodies.push((b.id, r.0, r.1));
            }
        }

        let mut rock: Vec<(i32, i32, Option<String>)> = Vec::new();
        for y in (self.min_y..=top_y).rev() {
            let mut class: Option<&str> = horizon_at(&horizons, y)
                .and_then(|h| self.classes.get(usize::from(horizons[h].1)))
                .map(String::as_str);
            for b in &near {
                if let Some(Some(bc)) = self.body_class(b, wx, y, z) {
                    class = Some(bc);
                }
            }
            match rock.last_mut() {
                Some(run) if run.2.as_deref() == class => run.1 = y,
                _ => rock.push((y, y, class.map(str::to_owned))),
            }
        }
        Column {
            x,
            z,
            wx,
            elevation,
            detail,
            surface,
            top_y,
            water,
            horizons,
            bodies,
            rock,
            hydro: self.hydro(x, z),
        }
    }

    /// Everything `deeptime inspect-column` shows about column (x, z): the [`Column`], its
    /// latitude and longitude, every other layer's value there, the horizons with their class
    /// names, the bodies with their roles, and the sites whose box holds the column.
    pub fn inspect(&self, x: i64, z: i64) -> Value {
        let col = self.sample(x, z);
        let c = self.c as f64;
        let (bx, bz) = (col.wx as f64, z as f64);
        let mut layers = serde_json::Map::new();
        for (name, l) in &self.others {
            let mut v = json!({ "nearest": finite(l.nearest(bx, bz)), "units": l.desc().units });
            if matches!(l.desc().dtype, DType::I16 | DType::I8 | DType::F32) {
                v["bilinear"] = finite(l.bilinear(bx, bz));
            }
            layers.insert(name.clone(), v);
        }
        let named = |i: u8| {
            self.classes
                .get(usize::from(i))
                .cloned()
                .unwrap_or_else(|| format!("#{i}"))
        };
        let horizons: Vec<Value> = col
            .horizons
            .iter()
            .enumerate()
            .map(|(h, &(top, class, unit))| {
                let bottom = col
                    .horizons
                    .get(h + 1)
                    .map_or(self.min_y, |&(t, _, _)| t + 1);
                json!({ "top": top, "bottom": bottom, "class": named(class), "unit": unit })
            })
            .collect();
        let bodies: Vec<Value> = col
            .bodies
            .iter()
            .map(|&(id, top, bottom)| {
                let b = self.bodies.iter().find(|b| b.id == id);
                json!({
                    "id": id, "top": top, "bottom": bottom,
                    "site": b.and_then(|b| b.site), "role": b.map(|b| b.role.clone()),
                    "shape": b.map(|b| b.shape.name()), "rock_class": b.and_then(|b| b.rock_class.clone()),
                })
            })
            .collect();
        let sites: Vec<Value> = match &self.sites {
            Some((sites, index)) => index
                .at_block(col.wx as i32, z as i32)
                .iter()
                .filter_map(|id| sites.iter().find(|s| s.id == *id))
                .filter(|s| {
                    let [lo, hi] = s.aabb;
                    let span = i64::from(hi[0]) - i64::from(lo[0]);
                    z >= i64::from(lo[2])
                        && z <= i64::from(hi[2])
                        && (span >= self.c - 1
                            || (col.wx - i64::from(lo[0])).rem_euclid(self.c) <= span)
                })
                .map(Site::to_json)
                .collect(),
            None => Vec::new(),
        };
        let mut v = col.to_json();
        v["lat"] = json!(-360.0 * bz / c);
        v["lon"] = json!(360.0 * (bx + c / 2.0) / c);
        v["world"] = json!(self.manifest.source);
        v["layers"] = Value::Object(layers);
        v["strata"] = json!(horizons);
        v["body_details"] = json!(bodies);
        v["sites"] = json!(sites);
        v
    }
}

impl WaterSource for ColumnWorld {
    fn height(&self, x: i64, z: i64) -> f64 {
        ColumnWorld::height(self, x, z)
    }

    fn level(&self, x: i64, z: i64, reach: i64) -> f64 {
        match &self.water {
            Some(w) => water_level_at(w, x, z, reach),
            None => f64::NAN,
        }
    }

    fn rivers(&self) -> Option<&Rivers> {
        self.rivers.as_ref()
    }
}

/// The shore rule (Java `WaterLevels.levelAt`): the `water_level` cell nearest block (x, z), else
/// the value of the nearest wet cell within `reach` blocks (first found at the smallest squared
/// distance, cells visited by column then row); NaN when there is none.
pub fn water_level_at(levels: &impl RasterRead, x: i64, z: i64, reach: i64) -> f64 {
    let own = levels.nearest(x as f64, z as f64);
    if !own.is_nan() || reach <= 0 {
        return own;
    }
    let g = &levels.desc().grid;
    let r = i64::from(g.r);
    let (ox, oz) = (g.origin_block_x, g.origin_block_z);
    let i0 = (x - reach - ox).div_euclid(r);
    let i1 = (x + reach - ox).div_euclid(r) + 1;
    let j0 = (z - reach - oz).div_euclid(r);
    let j1 = (z + reach - oz).div_euclid(r) + 1;
    let mut best = f64::NAN;
    let mut best_d2 = reach * reach + 1;
    for i in i0..=i1 {
        let dx = ox + i * r - x;
        for j in j0..=j1 {
            let dz = oz + j * r - z;
            let d2 = dx * dx + dz * dz;
            if d2 >= best_d2 {
                continue;
            }
            let v = levels.get(i, j);
            if !v.is_nan() {
                best = v;
                best_d2 = d2;
            }
        }
    }
    best
}

/// The horizon holding block y, or None above the first horizon's top (`Column.horizonAt`).
pub fn horizon_at(horizons: &[ColumnHorizon], y: i32) -> Option<usize> {
    if horizons.is_empty() || y > horizons[0].0 {
        return None;
    }
    let mut h = 0;
    while h + 1 < horizons.len() && y <= horizons[h + 1].0 {
        h += 1;
    }
    Some(h)
}

fn finite(v: f64) -> Value {
    if v.is_finite() { json!(v) } else { Value::Null }
}

impl Column {
    /// The column's JSON (FORMAT.md, "Columns"): what `parity/columns.json` holds per column.
    pub fn to_json(&self) -> Value {
        json!({
            "x": self.x, "z": self.z, "wx": self.wx,
            "elevation": finite(self.elevation), "detail": finite(self.detail),
            "surface": finite(self.surface), "top_y": self.top_y,
            "water": self.water,
            "horizons": self.horizons.iter().map(|&(t, c, u)| json!([t, c, u])).collect::<Vec<_>>(),
            "bodies": self.bodies.iter().map(|&(id, t, b)| json!([id, t, b])).collect::<Vec<_>>(),
            "rock": self.rock.iter().map(|(t, b, c)| json!([t, b, c])).collect::<Vec<_>>(),
            "hydro": self.hydro.to_json(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizon_at_follows_the_java_rule() {
        let h = [(100, 3u8, 0u16), (80, 4, 1), (10, 5, 2)];
        assert_eq!(horizon_at(&h, 101), None);
        assert_eq!(horizon_at(&h, 100), Some(0));
        assert_eq!(horizon_at(&h, 81), Some(0));
        assert_eq!(horizon_at(&h, 80), Some(1));
        assert_eq!(horizon_at(&h, 11), Some(1));
        assert_eq!(horizon_at(&h, 10), Some(2));
        assert_eq!(horizon_at(&h, -352), Some(2));
        assert_eq!(horizon_at(&[], 0), None);
    }
}

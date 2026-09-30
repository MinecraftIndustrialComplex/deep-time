// SPDX-License-Identifier: LGPL-3.0-or-later
//! The bake (plan §8.8, work item P1-08): a finished planet run becomes the chunk-time layers of
//! plan §5.3 in block space, next to the events, sites, bodies and keyframes the session writes.
//! `Session::finalize_write` calls [`bake`] for every planet preset, after the amplify stage's L3
//! layers are registered and before the manifest is written (last, plan §6.3).
//!
//! | Part | Writes | Module |
//! |---|---|---|
//! | terrain | `elevation` (the amplifier's, or L2/L1 upsampled without one), `water_level` (nodata: lakes are Phase 2), `spawn/candidates.json` | [`terrain`] |
//! | strata | `strata/horizons` (P1-07's stacks, the top horizon raised over the amplified terrain), `strata/classes.json` | [`strata`] |
//! | climate | `climate/{t_mean_c, p_mm, koppen}` on L1; `climate/{temperature, vegetation, continents, erosion, ridges}` on L2 (the §7.2 mapping, [`biomes`]) | [`climate`], [`biomes`] |
//! | surface | `surface/regolith_blocks`, `surface/class_top` on L2 | [`surface`] |
//! | meta | `meta/{plate_id, crust_age_myr, province}` on L1 | [`meta`] |
//!
//! The record goes to `manifest.json` `notes.bake`: the detail term the game adds
//! (`deeptime_format::DetailSpec`), what each part did and how long it took, the bytes written.
//! The Java reader and the column evaluator (`deeptime_format::column`) read what this writes;
//! FORMAT.md is the contract.

pub mod biomes;
pub mod climate;
pub mod ice;
pub mod meta;
pub mod placement;
pub mod strata;
pub mod surface;
pub mod terrain;

#[cfg(test)]
mod tests;

use std::path::Path;
use std::time::Instant;

use deeptime_format::detail::{BAKE_NOTES, DetailSpec};
use deeptime_format::fsutil::write_atomic;
use deeptime_format::{
    Cell, Codec, DType, FileEntry, GridDesc, LayerDesc, LayerEntry, Manifest, Raster, Scale,
    write_tile_pack,
};
use serde_json::{Map, Value, json};

use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::world::{Field, WorldState};

/// The version of the bake's record (`notes.bake.version`).
pub const VERSION: u32 = 1;

/// The salt of the game's detail noise (`notes.bake.detail.salt`; the amplifier's own noise uses
/// salts 101–103).
pub const DETAIL_SALT: u32 = 1729;

/// The folder and manifest the bake writes into, with a tally of what it wrote.
pub struct Out<'a> {
    pub dir: &'a Path,
    pub m: &'a mut Manifest,
    pub bytes: u64,
    pub layers: Vec<String>,
}

impl Out<'_> {
    /// Writes `data` (row-major cells of `desc.grid`) as `rasters/<name>.dtr` (or `<name>.dtr` for
    /// `strata/*`), and lists it.
    pub fn layer<T: Cell>(&mut self, desc: LayerDesc, data: &[T]) -> Result<(), CoreError> {
        let raster = Raster {
            grid: desc.grid.clone(),
            data: data.to_vec(),
        };
        self.layer_bytes(desc, &raster.to_le_bytes())
    }

    /// Writes a layer from its row-major cell bytes.
    pub fn layer_bytes(&mut self, desc: LayerDesc, bytes: &[u8]) -> Result<(), CoreError> {
        let rel = layer_path(&desc.name);
        let w = write_tile_pack(&self.dir.join(&rel), &desc, bytes)?;
        self.listed(&desc, rel, w.bytes, w.crc32);
        Ok(())
    }

    /// Lists a layer written by other means (a streamed pack).
    pub fn listed(&mut self, desc: &LayerDesc, rel: String, bytes: u64, crc32: u32) {
        self.m
            .grids
            .insert(desc.grid.name.clone(), desc.grid.clone());
        self.m.layers.retain(|l| l.name != desc.name);
        self.m.layers.push(LayerEntry {
            name: desc.name.clone(),
            path: rel,
            grid: desc.grid.name.clone(),
            dtype: desc.dtype,
            codec: desc.codec,
            bytes,
            crc32,
        });
        self.bytes += bytes;
        self.layers.push(desc.name.clone());
    }

    /// Writes a JSON side file and lists it.
    pub fn json(&mut self, rel: &str, v: &Value) -> Result<(), CoreError> {
        let mut bytes = serde_json::to_vec_pretty(v)
            .map_err(|e| CoreError::InvalidState(format!("{rel}: {e}")))?;
        bytes.push(b'\n');
        let crc32 = write_atomic(&self.dir.join(rel), &bytes)?;
        self.m.files.retain(|f| f.path != rel);
        self.m.files.push(FileEntry {
            path: rel.to_owned(),
            bytes: bytes.len() as u64,
            crc32,
        });
        self.bytes += bytes.len() as u64;
        Ok(())
    }
}

/// Where a layer's tile pack lives: `strata/…` beside `strata/classes.json`, the rest under
/// `rasters/` (FORMAT.md, "Folder layout").
pub fn layer_path(name: &str) -> String {
    if name.starts_with("strata/") {
        format!("{name}.dtr")
    } else {
        format!("rasters/{name}.dtr")
    }
}

/// A scalar layer descriptor.
pub(crate) fn desc(
    name: &str,
    dtype: DType,
    grid: &GridDesc,
    scale: f64,
    units: &str,
    nodata: Option<f64>,
) -> LayerDesc {
    let mut d = LayerDesc::scalar(name, dtype, grid.clone(), Codec::Lz4Block);
    d.scale = scale;
    d.units = units.to_owned();
    d.nodata = nodata;
    d
}

/// Bilinear sample of a field at block (bx, bz): x wraps, z clamps, NaN corners give NaN (the
/// arithmetic of `RasterRead::bilinear`).
pub(crate) fn bilinear(f: &Field, bx: f64, bz: f64) -> f32 {
    let g = &f.grid;
    let u = (bx - g.origin_block_x as f64) / f64::from(g.r);
    let v = (bz - g.origin_block_z as f64) / f64::from(g.r);
    let (fi, fj) = (u.floor(), v.floor());
    let (fu, fv) = (u - fi, v - fj);
    let (i0, j0) = (fi as i64, fj as i64);
    let v00 = f64::from(f.get(i0, j0));
    let v10 = f64::from(f.get(i0 + 1, j0));
    let v01 = f64::from(f.get(i0, j0 + 1));
    let v11 = f64::from(f.get(i0 + 1, j0 + 1));
    let a = v00 + (v10 - v00) * fu;
    let b = v01 + (v11 - v01) * fu;
    (a + (b - a) * fv) as f32
}

/// Nearest-cell sample of a field at block (bx, bz).
pub(crate) fn nearest(f: &Field, bx: f64, bz: f64) -> f32 {
    let g = &f.grid;
    let u = (bx - g.origin_block_x as f64) / f64::from(g.r);
    let v = (bz - g.origin_block_z as f64) / f64::from(g.r);
    f.get((u + 0.5).floor() as i64, (v + 0.5).floor() as i64)
}

/// A field resampled onto `grid` (bilinear, or nearest for categorical fields).
pub(crate) fn resample(f: &Field, grid: &GridDesc, categorical: bool) -> Vec<f32> {
    use rayon::prelude::*;
    if f.grid.w == grid.w && f.grid.h == grid.h && f.grid.r == grid.r {
        return f.data.clone();
    }
    let w = grid.w as usize;
    let mut out = vec![0f32; grid.cells()];
    out.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
        let bz = grid.block_z(j as i64) as f64;
        for (i, o) in row.iter_mut().enumerate() {
            let bx = grid.block_x(i as i64) as f64;
            *o = if categorical {
                nearest(f, bx, bz)
            } else {
                bilinear(f, bx, bz)
            };
        }
    });
    out
}

/// A world field by name, if it is on a grid of the planet.
pub(crate) fn field<'w>(world: &'w WorldState, name: &str) -> Option<&'w Field> {
    world
        .fields
        .get(name)
        .filter(|f| f.data.len() == f.grid.cells() && !f.data.is_empty())
}

/// The planet's L1 and L2 grids.
pub(crate) fn grids(p: &PlanetPreset) -> Result<(GridDesc, GridDesc), CoreError> {
    let c = p.scale.circumference_blocks;
    Ok((
        GridDesc::for_planet("L1", c, p.resolution.l1_r)?,
        GridDesc::for_planet("L2", c, p.resolution.l2_r)?,
    ))
}

fn ms(t: Instant) -> f64 {
    (t.elapsed().as_secs_f64() * 1000.0 * 10.0).round() / 10.0
}

/// Bakes a finished planet run into `dir` and records it in `m` (layers, grids, files, the
/// preset's scale). Returns `notes.bake`. A world without the deep-time surface (the plate-only
/// presets) has no terrain to bake: the record says so and nothing is written.
pub fn bake(
    world: &WorldState,
    p: &PlanetPreset,
    dir: &Path,
    m: &mut Manifest,
) -> Result<Value, CoreError> {
    let started = Instant::now();
    let s = &p.scale;
    m.scale = Scale {
        circumference_blocks: s.circumference_blocks,
        m_per_block_h: s.m_per_block_h,
        m_per_block_v: s.m_per_block_v,
        min_y: s.min_y,
        max_y: s.max_y,
        sea_level_y: s.sea_level_y,
    };
    m.planet.insert("radius_m".into(), json!(s.radius_m()));
    let Some(surface) = world.surface.as_ref() else {
        return Ok(json!({
            "version": VERSION,
            "baked": false,
            "why": "the preset has no surface section: no terrain to bake (a plate-only run)",
        }));
    };
    let mut out = Out {
        dir,
        m,
        bytes: 0,
        layers: Vec::new(),
    };
    let mut millis = Map::new();
    let mut note = Map::new();
    note.insert("version".into(), json!(VERSION));
    note.insert("baked".into(), json!(true));

    let t = Instant::now();
    let terrain = terrain::bake(world, p, surface, &mut out)?;
    millis.insert("terrain".into(), json!(ms(t)));
    note.insert("terrain".into(), terrain.note.clone());
    if terrain.detail {
        let a = p.amplify.clone().unwrap_or_default().detail;
        let spec = DetailSpec::new(p.seed, DETAIL_SALT, a.octaves, a.wavelength_blocks);
        note.insert(
            "detail".into(),
            serde_json::to_value(&spec).expect("a detail spec serialises"),
        );
    }

    let t = Instant::now();
    note.insert("strata".into(), strata::bake(world, p, &terrain, &mut out)?);
    millis.insert("strata".into(), json!(ms(t)));

    let t = Instant::now();
    note.insert(
        "climate".into(),
        climate::bake(world, p, surface, &mut out)?,
    );
    millis.insert("climate".into(), json!(ms(t)));

    let t = Instant::now();
    note.insert(
        "surface".into(),
        surface::bake(world, p, surface, &mut out)?,
    );
    millis.insert("surface".into(), json!(ms(t)));

    let t = Instant::now();
    note.insert("meta".into(), meta::bake(world, p, surface, &mut out)?);
    millis.insert("meta".into(), json!(ms(t)));

    millis.insert("total".into(), json!(ms(started)));
    note.insert("millis".into(), Value::Object(millis));
    note.insert("bytes".into(), json!(out.bytes));
    note.insert("layers".into(), json!(out.layers));
    Ok(Value::Object(note))
}

/// The `notes` key of the record.
pub const NOTES_KEY: &str = BAKE_NOTES;

/// `bake … ` (the CLI's and the log's line).
pub fn summary_line(v: &Value) -> String {
    if v["baked"] == false {
        return format!("bake skipped: {}", v["why"].as_str().unwrap_or("?"));
    }
    let ms = |k: &str| v["millis"][k].as_f64().unwrap_or(0.0) / 1000.0;
    format!(
        "bake layers={} bytes={:.1}MiB elevation={} horizons_mean={:.2} top_raised_cells={} provinces={} \
         spawn_candidates={} millis[terrain={:.1}s strata={:.1}s climate={:.1}s surface={:.1}s meta={:.1}s total={:.1}s]",
        v["layers"].as_array().map_or(0, Vec::len),
        v["bytes"].as_f64().unwrap_or(0.0) / 1_048_576.0,
        v["terrain"]["elevation"].as_str().unwrap_or("?"),
        v["strata"]["mean_horizons"].as_f64().unwrap_or(0.0),
        v["strata"]["top_raised_cells"],
        v["meta"]["provinces"]["count"],
        v["terrain"]["spawn_candidates"],
        ms("terrain"),
        ms("strata"),
        ms("climate"),
        ms("surface"),
        ms("meta"),
        ms("total"),
    )
}

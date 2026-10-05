// SPDX-License-Identifier: LGPL-3.0-or-later
//! The terrain part of the bake: `elevation` and `detail/amplitude` on L3, `water_level`, and the
//! spawn candidates.
//!
//! - **elevation.** The amplify stage (P1-06) already streamed `rasters/elevation.dtr` and
//!   `rasters/detail/amplitude.dtr` into the folder and `amplify::register` listed them; the bake
//!   keeps them. A preset without the amplify stage (tests, trimmed pipelines) gets an L3
//!   `elevation` upsampled bilinearly from the young landscape (L2), else from the deep-time
//!   surface (L1), and no detail layer. Metres become block y as everywhere: y = sea_level_y +
//!   (h − sea) / m_per_block_v, stored ×16 (i16).
//! - **water_level.** Every cell nodata here; the water part (`super::water`, P1-22) replaces it
//!   with the lakes after the climate is baked (and leaves it so when `hydro.enabled` is false).
//! - **spawn/candidates.json** ([`spawn`], called after the water part): the standard rule
//!   (`synth::spawn_candidates`), dry flat land nearest to (0, 0).

use std::path::PathBuf;

use deeptime_format::{DType, GridDesc, LazyLayer, TilePackWriter};
use rayon::prelude::*;
use serde_json::{Value, json};

use super::{Out, desc, field, layer_path, resample};
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::surface::SurfaceState;
use crate::surface::young;
use crate::synth::{NODATA_I16, quantise};
use crate::world::{Field, WorldState};

/// What the rest of the bake needs from the terrain.
pub struct Terrain {
    pub note: Value,
    pub l3: GridDesc,
    pub elevation: PathBuf,
    /// The detail amplitude layer, when the amplifier wrote one.
    pub amplitude: Option<PathBuf>,
    /// True when the world has the detail term (`notes.bake.detail`).
    pub detail: bool,
}

pub fn bake(
    world: &WorldState,
    p: &PlanetPreset,
    surface: &SurfaceState,
    out: &mut Out<'_>,
) -> Result<Terrain, CoreError> {
    let s = &p.scale;
    let c = s.circumference_blocks;
    let (mut source, l3) = match out.m.layer("elevation") {
        Some(e) => {
            let grid = out
                .m
                .grids
                .get(&e.grid)
                .cloned()
                .ok_or_else(|| CoreError::InvalidState(format!("no grid {}", e.grid)))?;
            out.bytes += e.bytes;
            out.layers.push("elevation".into());
            ("amplify", grid)
        }
        None => {
            let l3 = GridDesc::for_planet("L3", c, p.resolution.l3_r)?;
            let sea = surface.sea_level_m;
            let (from, name) = match field(world, young::field::ELEVATION_M) {
                Some(f) => (f.clone(), "young (L2, bilinear)"),
                None => (
                    Field {
                        grid: surface.grid.clone(),
                        data: surface.elevation_m.clone(),
                    },
                    "surface (L1, bilinear)",
                ),
            };
            let metres = resample(&from, &l3, false);
            let (sy, mv) = (f64::from(s.sea_level_y), s.m_per_block_v);
            let fit = s.build_fit();
            let raw: Vec<i16> = metres
                .par_iter()
                .map(|&h| quantise(fit.compress(sy + (f64::from(h) - sea) / mv) * 16.0))
                .collect();
            out.layer(
                desc("elevation", DType::I16, &l3, 1.0 / 16.0, "block", None),
                &raw,
            )?;
            (name, l3)
        }
    };
    let elevation = out.dir.join(
        &out.m
            .layer("elevation")
            .expect("the elevation layer is listed")
            .path,
    );
    let amplitude = match out.m.layer("detail/amplitude") {
        Some(a) => {
            out.bytes += a.bytes;
            out.layers.push(a.name.clone());
            Some(out.dir.join(&a.path))
        }
        None => None,
    };
    if amplitude.is_none() && source == "amplify" {
        source = "amplify (no detail layer)";
    }

    // water_level: nodata everywhere, written a tile at a time (no L3-sized buffer).
    let wd = desc(
        "water_level",
        DType::I16,
        &l3,
        1.0 / 16.0,
        "block",
        Some(f64::from(NODATA_I16)),
    );
    let rel = layer_path("water_level");
    let mut w = TilePackWriter::create(&out.dir.join(&rel), &wd)?;
    for t in 0..wd.tile_count() {
        let (_, _, tw, th) = wd.tile_rect(t);
        let cells: Vec<u8> = std::iter::repeat_n(NODATA_I16.to_le_bytes(), (tw * th) as usize)
            .flatten()
            .collect();
        w.put_tile(t, &cells)?;
    }
    let written = w.finish()?;
    out.listed(&wd, rel, written.bytes, written.crc32);

    Ok(Terrain {
        note: json!({
            "elevation": source,
            "grid": l3,
            "water_level": "nodata until the water part (P1-22) writes the lakes",
        }),
        l3,
        elevation,
        detail: amplitude.is_some(),
        amplitude,
    })
}

/// `spawn/candidates.json`, after the water part: the standard rule (`synth::spawn_candidates`),
/// dry flat land at least two blocks above sea level, nearest to (0, 0) first (Ben, 2026-10-05: no
/// ranking by the essentials). Returns the number of candidates.
pub fn spawn(p: &PlanetPreset, terrain: &Terrain, out: &mut Out<'_>) -> Result<usize, CoreError> {
    let s = &p.scale;
    let elev = LazyLayer::open(&terrain.elevation)?;
    let wl = out
        .m
        .layer("water_level")
        .ok_or_else(|| CoreError::InvalidState("no water_level layer".into()))?;
    let water = LazyLayer::open(&out.dir.join(&wl.path))?;
    let spawn = crate::synth::spawn_candidates(&elev, &water, s.sea_level_y);
    let n = spawn["candidates"].as_array().map_or(0, Vec::len);
    out.json("spawn/candidates.json", &spawn)?;
    Ok(n)
}

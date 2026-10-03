// SPDX-License-Identifier: LGPL-3.0-or-later
//! The water part of the bake (P1-22): `water_level` and `rivers/rivers.dtrv` from the terrain
//! and the climate already in the folder ([`crate::hydro`]). It reads only what the bake wrote
//! before it (the `elevation` layer and the L1 `climate/t_mean_c` and `climate/p_mm`), so
//! `deeptime hydro --world-data <dir>` re-runs it on any baked folder with the same result.
//!
//! - **Grid.** The hydrology runs on the `elevation` grid (L3) when it has at most
//!   [`hydro::MAX_CELLS`] cells; a larger one (32k full: 134 M cells) is averaged 2 × 2 into grid
//!   `L3h` first, and `water_level` is written on that grid (a layer names its own grid).
//! - **Climate.** Precipitation and mean temperature bilinear from L1 at each cell, and the
//!   warmest and coldest month from the twelve monthly layers (`climate/month/t_sl_c_*`: the
//!   months' departures from their mean are the same at any height, so they are added to the
//!   cell's mean; a folder without them takes ± 8 °C). [`hydro::HydroParams::budget`] turns them
//!   into runoff, rain on a lake and its evaporation. Without a climate stage the surface stage's
//!   zonal stand-in at sea level.
//! - **water_level.** Each lake's cells hold its level (1/16 block), nodata elsewhere; the game's
//!   shore rule carries it a noise cell past the last wet cell. A full lake's level is its rim's
//!   height, a terminal lake's the level of its water balance, below the rim. The layer means what
//!   it meant: nothing in the game's reading of it changes.
//! - **Ice.** No layer: the game ices a lake column's top block where `climate/t_mean_c`
//!   (bilinear at the column) is below `notes.bake.water.params.lake_freeze_mean_c`, and keeps the
//!   other lake columns open whatever the biome (Java `worlddata/LakeIce`; FORMAT.md "Lake ice").
//!   `notes.bake.water.stats.frozen_in_frozen_band` records how much of the frozen lake area lies
//!   in the biome temperature layer's frozen band, where vanilla alone would have iced it.
//! - **playa.** Blocks of salt (`hydro.playa_blocks`) on the cells of a playa, 0 elsewhere: the
//!   game replaces that many of the ground's top blocks with the `halite` rock class's block.

use std::path::Path;

use deeptime_format::rivers::{RIVERS_PATH, write_rivers};
use deeptime_format::{DType, FileEntry, GridDesc, LoadedLayer, RasterRead, TilePackReader};
use serde_json::{Value, json};

use super::{Out, desc};
use crate::error::CoreError;
use crate::hydro::{self, network};
use crate::params::PlanetPreset;
use crate::surface::climate::{zonal_precip, zonal_temperature};

/// The warmest and the coldest month's departure from the year's mean, °C, per cell of the
/// monthly layers' grid.
struct Seasons {
    grid: GridDesc,
    warm: Vec<f32>,
    cold: Vec<f32>,
}

impl Seasons {
    fn of(months: &[LoadedLayer]) -> Option<Seasons> {
        let grid = months.first()?.desc.grid.clone();
        if months.len() != 12 || months.iter().any(|m| m.desc.grid != grid) {
            return None;
        }
        let (w, h) = (grid.w, grid.h);
        let mut warm = Vec::with_capacity(grid.cells());
        let mut cold = Vec::with_capacity(grid.cells());
        for j in 0..h {
            for i in 0..w {
                let (mut lo, mut hi, mut sum) = (f64::INFINITY, f64::NEG_INFINITY, 0.0);
                for m in months {
                    let t = m.desc.physical(m.raw_cell(i, j));
                    lo = lo.min(t);
                    hi = hi.max(t);
                    sum += t;
                }
                let mean = sum / 12.0;
                warm.push((hi - mean) as f32);
                cold.push((lo - mean) as f32);
            }
        }
        Some(Seasons { grid, warm, cold })
    }

    /// (warmest, coldest) departure at block (x, z), bilinear; NaN where the layers hold none.
    fn at(&self, bx: f64, bz: f64) -> (f64, f64) {
        let g = &self.grid;
        let u = (bx - g.origin_block_x as f64) / f64::from(g.r);
        let v = (bz - g.origin_block_z as f64) / f64::from(g.r);
        let (fi, fj) = (u.floor(), v.floor());
        let (fu, fv) = (u - fi, v - fj);
        let at = |i: i64, j: i64| g.clamp_j(j) as usize * g.w as usize + g.wrap_i(i) as usize;
        let (i0, j0) = (fi as i64, fj as i64);
        let k = [
            at(i0, j0),
            at(i0 + 1, j0),
            at(i0, j0 + 1),
            at(i0 + 1, j0 + 1),
        ];
        let mix = |d: &[f32]| {
            let a = f64::from(d[k[0]]) + f64::from(d[k[1]] - d[k[0]]) * fu;
            let b = f64::from(d[k[2]]) + f64::from(d[k[3]] - d[k[2]]) * fu;
            a + (b - a) * fv
        };
        (mix(&self.warm), mix(&self.cold))
    }
}

/// The upper edge of the frozen band of the biome temperature input (`climate/temperature`,
/// vanilla's parameter: `OverworldBiomeBuilder`'s first temperature band is −1 … −0.45).
const FROZEN_BAND_TOP: f64 = -0.45;

/// The layer of the playas' salt (blocks; on `water_level`'s grid) and the rock class it is.
pub const PLAYA_LAYER: &str = "playa";
pub const PLAYA_ROCK_CLASS: &str = "halite";

/// What the water part made, for the tests and the CLI.
pub struct Water {
    pub note: Value,
    pub output: Option<network::Output>,
}

/// Loads an i16 layer whole.
pub(crate) fn load_i16(path: &Path) -> Result<(GridDesc, Vec<i16>), CoreError> {
    let l = TilePackReader::open(path)?.load()?;
    if l.desc.dtype != DType::I16 {
        return Err(CoreError::InvalidState(format!(
            "{}: expected an i16 layer",
            path.display()
        )));
    }
    let v = l
        .bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| i16::from_le_bytes(*b))
        .collect();
    Ok((l.desc.grid, v))
}

/// Averages a raster 2 × 2 (x wraps: W is even).
pub(crate) fn halve(g: &GridDesc, v: &[i16]) -> (GridDesc, Vec<i16>) {
    let (w, h) = (g.w as usize, g.h as usize);
    let (w2, h2) = (w / 2, h / 2);
    let mut out = vec![0i16; w2 * h2];
    for j in 0..h2 {
        for i in 0..w2 {
            let s: i32 = [(0, 0), (1, 0), (0, 1), (1, 1)]
                .iter()
                .map(|&(di, dj)| i32::from(v[(2 * j + dj) * w + 2 * i + di]))
                .sum();
            out[j * w2 + i] = s.div_euclid(4) as i16;
        }
    }
    let grid = GridDesc {
        name: format!("{}h", g.name),
        w: g.w / 2,
        h: g.h / 2,
        r: g.r * 2,
        wraps_x: g.wraps_x,
        origin_block_x: g.origin_block_x,
        origin_block_z: g.origin_block_z,
    };
    (grid, out)
}

/// Runs the hydrology on the folder `out.dir` and writes its layers and file.
pub fn bake(p: &PlanetPreset, out: &mut Out<'_>) -> Result<Water, CoreError> {
    let params = p.hydro.unwrap_or_default();
    let rivers_path = out.dir.join(RIVERS_PATH);
    if !params.enabled {
        if rivers_path.exists() {
            std::fs::remove_file(&rivers_path).map_err(|e| CoreError::io(&rivers_path, e))?;
        }
        out.m.files.retain(|f| f.path != RIVERS_PATH);
        out.m.layers.retain(|l| l.name != PLAYA_LAYER);
        // An all-nodata `water_level` on the elevation grid (a re-run over a folder that had
        // lakes clears them).
        if let Some(e) = out.m.layer("elevation") {
            let grid = out
                .m
                .grids
                .get(&e.grid)
                .cloned()
                .ok_or_else(|| CoreError::InvalidState(format!("no grid {}", e.grid)))?;
            let wd = desc(
                "water_level",
                DType::I16,
                &grid,
                1.0 / 16.0,
                "block",
                Some(f64::from(network::NODATA)),
            );
            out.layer(wd, &vec![network::NODATA; grid.cells()])?;
            let mut seen = false;
            out.layers
                .retain(|l| l != "water_level" || !std::mem::replace(&mut seen, true));
        }
        return Ok(Water {
            note: json!({"enabled": false, "why": "hydro.enabled is false: no lakes or rivers"}),
            output: None,
        });
    }
    let e = out
        .m
        .layer("elevation")
        .ok_or_else(|| CoreError::InvalidState("no elevation layer to drain".into()))?;
    let (mut grid, mut elev) = load_i16(&out.dir.join(&e.path))?;
    let mut halved = false;
    while grid.cells() > hydro::MAX_CELLS && grid.w % 2 == 0 && grid.h % 2 == 0 {
        (grid, elev) = halve(&grid, &elev);
        halved = true;
    }

    // The L1 climate is small: whole in memory, so the parallel sampling shares no cache lock.
    let layer = |name: &str| -> Result<Option<LoadedLayer>, CoreError> {
        match out.m.layer(name) {
            Some(l) => Ok(Some(TilePackReader::open(&out.dir.join(&l.path))?.load()?)),
            None => Ok(None),
        }
    };
    let t_layer = layer("climate/t_mean_c")?;
    let p_layer = layer("climate/p_mm")?;
    let mut months = Vec::new();
    for m in 0..12 {
        if let Some(l) = layer(&super::climate::t_sl_month_layer(m))? {
            months.push(l);
        }
    }
    let seasons = Seasons::of(&months);
    drop(months);
    let band = layer("climate/temperature")?;
    let s = &p.scale;
    let c = f64::from(s.circumference_blocks);
    let from_stage = t_layer.is_some() && p_layer.is_some();
    let seasonal = seasons.is_some();
    let climate = move |bx: i64, bz: i64| -> hydro::Climate {
        let (x, z) = (bx as f64, bz as f64);
        let (t, pr) = match (&t_layer, &p_layer) {
            (Some(t), Some(pl)) => (t.bilinear(x, z), pl.bilinear(x, z)),
            _ => {
                let lat = deeptime_format::Projection::of(c).lat_deg(z);
                (zonal_temperature(lat), zonal_precip(lat))
            }
        };
        let (warm, cold) = seasons
            .as_ref()
            .map_or((f64::NAN, f64::NAN), |s| s.at(x, z));
        hydro::Climate::of(pr, t, t + warm, t + cold)
    };
    let sea_raw = s.sea_level_y * 16;
    let result = network::run(&network::Input {
        grid: &grid,
        elev: &elev,
        sea_raw,
        m_per_block_h: s.m_per_block_h,
        climate: &climate,
        params: &params,
        width_scale: s.feature_scale.rivers,
    });
    drop(elev);
    let mut stats = result.stats.clone();
    // How much of the frozen lake area the game's frozen biomes cover (the temperature layer's
    // frozen band, below its upper edge −0.45): there vanilla's `freeze_top_layer` ices the water.
    if let Some(band) = &band {
        let (ox, oz, r) = (grid.origin_block_x, grid.origin_block_z, i64::from(grid.r));
        let w = grid.w as usize;
        let (mut frozen, mut in_band, mut open_in_band, mut open) = (0u64, 0u64, 0u64, 0u64);
        for (cell, &k) in result.kind.iter().enumerate() {
            if k != network::kind::LAKE && k != network::kind::FROZEN {
                continue;
            }
            let (x, z) = (ox + (cell % w) as i64 * r, oz + (cell / w) as i64 * r);
            let cold = band.nearest(x as f64, z as f64) < FROZEN_BAND_TOP;
            if k == network::kind::FROZEN {
                frozen += 1;
                in_band += u64::from(cold);
            } else {
                open += 1;
                open_in_band += u64::from(cold);
            }
        }
        let share = |a: u64, b: u64| {
            if b > 0 {
                json!((a as f64 / b as f64 * 1e4).round() / 1e4)
            } else {
                Value::Null
            }
        };
        stats["frozen_in_frozen_band"] = share(in_band, frozen);
        stats["open_in_frozen_band"] = share(open_in_band, open);
    }

    // water_level: replaces the nodata layer the terrain part wrote.
    let wd = desc(
        "water_level",
        DType::I16,
        &grid,
        1.0 / 16.0,
        "block",
        Some(f64::from(network::NODATA)),
    );
    out.layer(wd, &result.water_level)?;
    // `Out::layer` tallied the layer a second time.
    let mut seen = false;
    out.layers.retain(|l| {
        if l == "water_level" {
            let keep = !seen;
            seen = true;
            keep
        } else {
            true
        }
    });

    // playa: blocks of salt on the floor of basins too dry for a lake (0 elsewhere).
    let salt = params.playa_blocks.round().clamp(0.0, 255.0) as u8;
    let playa: Vec<u8> = result
        .kind
        .iter()
        .map(|&k| if k == network::kind::PLAYA { salt } else { 0 })
        .collect();
    out.layer(
        desc(PLAYA_LAYER, DType::U8, &grid, 1.0, "blocks", None),
        &playa,
    )?;
    drop(playa);

    let (bytes, crc32) = write_rivers(out.dir, &result.rivers)?;
    out.m.files.retain(|f| f.path != RIVERS_PATH);
    out.m.files.push(FileEntry {
        path: RIVERS_PATH.to_owned(),
        bytes,
        crc32,
    });
    out.bytes += bytes;

    let note = json!({
        "enabled": true,
        "method": "p1-22b",
        "params": params,
        "width_scale": s.feature_scale.rivers,
        "grid": grid.name,
        "halved": halved,
        "climate": if from_stage { "climate stage (L1, bilinear)" } else { "zonal stand-in" },
        "seasons": if seasonal { "the twelve monthly layers" } else { "mean ± 8 °C" },
        "playa": {"layer": PLAYA_LAYER, "rock_class": PLAYA_ROCK_CLASS, "blocks": salt},
        "rivers_file": RIVERS_PATH,
        "rivers_bytes": bytes,
        "stats": stats,
    });
    Ok(Water {
        note,
        output: Some(result),
    })
}

/// `water …` (the CLI's line).
pub fn summary_line(v: &Value) -> String {
    if v["enabled"] == false {
        return "water disabled".into();
    }
    let s = &v["stats"];
    format!(
        "water grid={} lakes={} lake_km2={} share_of_land={} largest_km2={} below_rim={} frozen={} playas={} dry={} endorheic={} chains={} vertices={} river_km={} max_q={} widest={} bytes={}",
        v["grid"].as_str().unwrap_or("?"),
        s["lakes"],
        s["lake_area_km2"],
        s["lake_share_of_land"],
        s["largest_lake_km2"],
        s["basins"]["below_rim"],
        s["frozen"]["lakes"],
        s["basins"]["playa"],
        s["basins"]["dry"],
        s["endorheic_lakes"],
        s["chains"],
        s["vertices"],
        s["river_length_km"],
        s["max_discharge_m3s"],
        s["widest_blocks"],
        v["rivers_bytes"],
    )
}

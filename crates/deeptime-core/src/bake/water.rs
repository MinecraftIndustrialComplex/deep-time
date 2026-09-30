// SPDX-License-Identifier: LGPL-3.0-or-later
//! The water part of the bake (P1-22): `water_level` and `rivers/rivers.dtrv` from the terrain
//! and the climate already in the folder ([`crate::hydro`]). It reads only what the bake wrote
//! before it (the `elevation` layer and the L1 `climate/t_mean_c` and `climate/p_mm`), so
//! `deeptime hydro --world-data <dir>` re-runs it on any baked folder with the same result.
//!
//! - **Grid.** The hydrology runs on the `elevation` grid (L3) when it has at most
//!   [`hydro::MAX_CELLS`] cells; a larger one (32k full: 134 M cells) is averaged 2 × 2 into grid
//!   `L3h` first, and `water_level` is written on that grid (a layer names its own grid).
//! - **Climate.** Precipitation and temperature bilinear from L1 at each cell; PET = 60 mm/°C · T
//!   (0 below 0 °C), as the surface stage. Without a climate stage the surface stage's zonal
//!   stand-in at sea level.
//! - **water_level.** Each lake's cells hold its level (1/16 block), nodata elsewhere; the game's
//!   shore rule carries it a noise cell past the last wet cell.

use std::path::Path;

use deeptime_format::rivers::{RIVERS_PATH, write_rivers};
use deeptime_format::{DType, FileEntry, GridDesc, LoadedLayer, RasterRead, TilePackReader};
use serde_json::{Value, json};

use super::{Out, desc};
use crate::error::CoreError;
use crate::hydro::{self, network};
use crate::params::PlanetPreset;
use crate::surface::climate::{zonal_precip, zonal_temperature};

/// What the water part made, for the tests and the CLI.
pub struct Water {
    pub note: Value,
    pub output: Option<network::Output>,
}

/// Loads an i16 layer whole.
fn load_i16(path: &Path) -> Result<(GridDesc, Vec<i16>), CoreError> {
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
fn halve(g: &GridDesc, v: &[i16]) -> (GridDesc, Vec<i16>) {
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
    let params = p.hydro.clone().unwrap_or_default();
    let rivers_path = out.dir.join(RIVERS_PATH);
    if !params.enabled {
        if rivers_path.exists() {
            std::fs::remove_file(&rivers_path).map_err(|e| CoreError::io(&rivers_path, e))?;
        }
        out.m.files.retain(|f| f.path != RIVERS_PATH);
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
            out.layers.retain(|l| l != "water_level" || !std::mem::replace(&mut seen, true));
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
    let s = &p.scale;
    let c = f64::from(s.circumference_blocks);
    let from_stage = t_layer.is_some() && p_layer.is_some();
    let climate = move |bx: i64, bz: i64| -> (f64, f64) {
        let (x, z) = (bx as f64, bz as f64);
        let (t, pr) = match (&t_layer, &p_layer) {
            (Some(t), Some(pl)) => (t.bilinear(x, z), pl.bilinear(x, z)),
            _ => {
                let lat = -360.0 * z / c;
                (zonal_temperature(lat), zonal_precip(lat))
            }
        };
        let (t, pr) = (
            if t.is_finite() { t } else { 0.0 },
            if pr.is_finite() { pr.max(0.0) } else { 0.0 },
        );
        (pr, (60.0 * t).max(0.0))
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
        "method": "p1-22",
        "params": params,
        "width_scale": s.feature_scale.rivers,
        "grid": grid.name,
        "halved": halved,
        "climate": if from_stage { "climate stage (L1, bilinear)" } else { "zonal stand-in" },
        "rivers_file": RIVERS_PATH,
        "rivers_bytes": bytes,
        "stats": result.stats,
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
        "water grid={} lakes={} lake_km2={} endorheic={} chains={} vertices={} river_km={} max_q={} widest={} bytes={}",
        v["grid"].as_str().unwrap_or("?"),
        s["lakes"],
        s["lake_area_km2"],
        s["endorheic_lakes"],
        s["chains"],
        s["vertices"],
        s["river_length_km"],
        s["max_discharge_m3s"],
        s["widest_blocks"],
        v["rivers_bytes"],
    )
}

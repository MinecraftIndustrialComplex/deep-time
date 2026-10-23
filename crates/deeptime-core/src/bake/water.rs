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
//! - **Deltas** (V1 gap 5, stage 1; [`hydro::delta`]). With `hydro.deltas` the network runs
//!   twice: the first run finds the rivers that reach the open sea, [`hydro::delta::grow`] lays a
//!   plain at each mouth, one block above the sea's top water block, and the second run drains the
//!   raised ground; [`hydro::delta::distributaries`] then splits each plain's main stem into
//!   chains that leave it (`from` in the river file). The raise goes into `elevation` itself (on
//!   its own grid, each L3 cell of a plain cell at least the plain's height), `detail/amplitude`
//!   is zeroed there (a plain is flat), and the layer [`DELTA_LAYER`] keeps what was added, so a
//!   re-run takes it off first and starts from the history's ground. On a Lambert world the
//!   horizon stacks' tops are raised over the plains here; a projected world's pass does that for
//!   every cell.

use std::path::Path;

use deeptime_format::column::CLOSED_BASIN;
use deeptime_format::projection::ProjectionKind;
use deeptime_format::rivers::{RIVERS_PATH, write_rivers};
use deeptime_format::{DType, FileEntry, GridDesc, LoadedLayer, RasterRead};
use rayon::prelude::*;
use serde_json::{Value, json};

use super::{Out, desc};
use crate::error::CoreError;
use crate::hydro::{self, delta, network};
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

/// What the delta plains added to `elevation`, 1/16 block, on its grid (0 off the plains).
pub const DELTA_LAYER: &str = "delta";

/// Drops every listing of `name` in `out.layers` but the first (`Out::layer` tallies a rewritten
/// layer again).
fn tally_once(out: &mut Out<'_>, name: &str) {
    let mut seen = false;
    out.layers
        .retain(|l| l != name || !std::mem::replace(&mut seen, true));
}

/// Takes a previous run's delta plains off `elevation` (a re-run over a baked folder) and drops
/// the delta layer: the hydrology starts from the ground the history left.
fn take_off_deltas(out: &mut Out<'_>) -> Result<(), CoreError> {
    let Some(d) = out.m.layer(DELTA_LAYER).cloned() else {
        return Ok(());
    };
    let path = out.dir.join(&d.path);
    let (dg, raise) = load_i16(&path)?;
    if let Some(e) = out.m.layer("elevation").cloned() {
        let mut el = crate::bake::load_layer(&out.dir.join(&e.path))?;
        if el.desc.grid == dg && el.desc.dtype == DType::I16 {
            el.bytes
                .as_chunks_mut::<2>()
                .0
                .par_iter_mut()
                .zip(raise.par_iter())
                .for_each(|(b, &r)| {
                    if r > 0 {
                        *b = i16::from_le_bytes(*b).saturating_sub(r).to_le_bytes();
                    }
                });
            out.layer_bytes(el.desc, &el.bytes)?;
            tally_once(out, "elevation");
        }
    }
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| CoreError::io(&path, e))?;
    }
    out.m.layers.retain(|l| l.name != DELTA_LAYER);
    out.layers.retain(|l| l != DELTA_LAYER);
    Ok(())
}

/// What laying the plains on the elevation's grid did.
struct Laid {
    /// Cells of the elevation's grid raised, and the largest raise, blocks.
    cells: u64,
    max_blocks: f64,
    amplitude_zeroed: u64,
    /// The horizon stacks raised over the plains (a Lambert world), and the largest raise, blocks.
    stack_cells: u64,
    stack_max_blocks: f64,
}

/// Raises `elevation` (grid `l3`) to the plains (on the hydrology's grid, `halvings` times
/// coarser): each L3 cell of a plain cell is at least the plain's height. Writes the raise as
/// [`DELTA_LAYER`], zeroes `detail/amplitude` on the raised cells, and on a Lambert world raises
/// the horizon stacks' tops over them (`project::raise_world_tops`, the bake's top rule).
fn lay_plains(
    p: &PlanetPreset,
    out: &mut Out<'_>,
    l3: &GridDesc,
    hydro_grid: &GridDesc,
    halvings: u32,
    plains: &delta::Plains,
) -> Result<Laid, CoreError> {
    let e = out
        .m
        .layer("elevation")
        .cloned()
        .ok_or_else(|| CoreError::InvalidState("no elevation layer".into()))?;
    let mut el = crate::bake::load_layer(&out.dir.join(&e.path))?;
    if el.desc.grid != *l3 || el.desc.dtype != DType::I16 {
        return Err(CoreError::InvalidState(
            "the elevation layer changed under the water part".into(),
        ));
    }
    let f = 1usize << halvings;
    let (w3, wh) = (l3.w as usize, hydro_grid.w as usize);
    let mut raise = vec![0i16; l3.cells()];
    let (mut cells, mut max_raw) = (0u64, 0i16);
    for pl in &plains.plains {
        for (&c, &h) in pl.cells.iter().zip(&pl.height) {
            let (i, j) = (c as usize % wh, c as usize / wh);
            for dj in 0..f {
                for di in 0..f {
                    let c3 = (j * f + dj) * w3 + i * f + di;
                    let at = c3 * 2;
                    let old = i16::from_le_bytes([el.bytes[at], el.bytes[at + 1]]);
                    if h > old && raise[c3] == 0 {
                        raise[c3] = h - old;
                        max_raw = max_raw.max(h - old);
                        cells += 1;
                        el.bytes[at..at + 2].copy_from_slice(&h.to_le_bytes());
                    }
                }
            }
        }
    }
    let mut laid = Laid {
        cells,
        max_blocks: f64::from(max_raw) / 16.0,
        amplitude_zeroed: 0,
        stack_cells: 0,
        stack_max_blocks: 0.0,
    };
    out.layer_bytes(el.desc.clone(), &el.bytes)?;
    tally_once(out, "elevation");
    out.layer(
        desc(DELTA_LAYER, DType::I16, l3, 1.0 / 16.0, "block", None),
        &raise,
    )?;

    // A plain is flat: no detail noise on it.
    if let Some(a) = out.m.layer("detail/amplitude").cloned() {
        let mut amp = crate::bake::load_layer(&out.dir.join(&a.path))?;
        if amp.desc.grid == *l3 && amp.desc.dtype == DType::U8 {
            for (b, &r) in amp.bytes.iter_mut().zip(&raise) {
                if r > 0 && *b != 0 {
                    *b = 0;
                    laid.amplitude_zeroed += 1;
                }
            }
            if laid.amplitude_zeroed > 0 {
                out.layer_bytes(amp.desc, &amp.bytes)?;
                tally_once(out, "detail/amplitude");
            }
        }
    }

    // The horizon stacks' tops over the plains: a projected world's pass raises every cell's;
    // a Lambert world is played from these stacks, so they are raised here, over the plain cells
    // alone (the elevation elsewhere masked as nodata).
    if p.scale.projection == ProjectionKind::Lambert
        && let Some(st) = out.m.layer("strata/horizons").cloned()
    {
        const MASK: i16 = i16::MIN;
        let mut masked = el;
        masked.desc.nodata = Some(f64::from(MASK));
        for (b, &r) in masked.bytes.as_chunks_mut::<2>().0.iter_mut().zip(&raise) {
            if r == 0 {
                *b = MASK.to_le_bytes();
            }
        }
        let mut stack = crate::bake::load_layer(&out.dir.join(&st.path))?;
        let (n, max, _) = super::project::raise_world_tops(
            &stack.desc,
            &mut stack.bytes,
            &masked,
            None,
            p.scale.min_y,
            p.scale.max_y,
            false,
        );
        laid.stack_cells = n;
        laid.stack_max_blocks = max;
        if n > 0 {
            out.layer_bytes(stack.desc, &stack.bytes)?;
            tally_once(out, "strata/horizons");
        }
    }
    Ok(laid)
}

/// What the water part made, for the tests and the CLI.
pub struct Water {
    pub note: Value,
    pub output: Option<network::Output>,
}

/// Loads an i16 layer whole (in parallel, P1-19).
pub(crate) fn load_i16(path: &Path) -> Result<(GridDesc, Vec<i16>), CoreError> {
    let l = crate::bake::load_layer(path)?;
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
        .par_iter()
        .map(|b| i16::from_le_bytes(*b))
        .collect();
    Ok((l.desc.grid, v))
}

/// Averages a raster 2 × 2 (x wraps: W is even).
pub(crate) fn halve(g: &GridDesc, v: &[i16]) -> (GridDesc, Vec<i16>) {
    let (w, h) = (g.w as usize, g.h as usize);
    let (w2, h2) = (w / 2, h / 2);
    let mut out = vec![0i16; w2 * h2];
    // A row in parallel (P1-19; whole numbers, so the same in any order).
    if w2 > 0 {
        out.par_chunks_mut(w2).enumerate().for_each(|(j, row)| {
            for (i, o) in row.iter_mut().enumerate() {
                let s: i32 = [(0, 0), (1, 0), (0, 1), (1, 1)]
                    .iter()
                    .map(|&(di, dj)| i32::from(v[(2 * j + dj) * w + 2 * i + di]))
                    .sum();
                *o = s.div_euclid(4) as i16;
            }
        });
    }
    let grid = GridDesc {
        name: format!("{}h", g.name),
        w: g.w / 2,
        h: g.h / 2,
        r: g.r * 2,
        wraps_x: g.wraps_x,
        origin_block_x: g.origin_block_x,
        origin_block_z: g.origin_block_z,
        z_edges: g.z_edges,
    };
    (grid, out)
}

/// Runs the hydrology on the folder `out.dir` and writes its layers and file.
pub fn bake(p: &PlanetPreset, out: &mut Out<'_>) -> Result<Water, CoreError> {
    let params = p.hydro.unwrap_or_default();
    let rivers_path = out.dir.join(RIVERS_PATH);
    take_off_deltas(out)?;
    if !params.enabled {
        if rivers_path.exists() {
            std::fs::remove_file(&rivers_path).map_err(|e| CoreError::io(&rivers_path, e))?;
        }
        out.m.files.retain(|f| f.path != RIVERS_PATH);
        out.m
            .layers
            .retain(|l| l.name != PLAYA_LAYER && l.name != CLOSED_BASIN);
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
            tally_once(out, "water_level");
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
    let l3 = grid.clone();
    let mut halvings = 0u32;
    while grid.cells() > hydro::MAX_CELLS && grid.w % 2 == 0 && grid.h % 2 == 0 {
        (grid, elev) = halve(&grid, &elev);
        halvings += 1;
    }
    let halved = halvings > 0;

    // The L1 climate is small: whole in memory, so the parallel sampling shares no cache lock.
    let layer = |name: &str| -> Result<Option<LoadedLayer>, CoreError> {
        match out.m.layer(name) {
            Some(l) => Ok(Some(crate::bake::load_layer(&out.dir.join(&l.path))?)),
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
    let first = network::run(&network::Input {
        grid: &grid,
        elev: &elev,
        sea_raw,
        m_per_block_h: s.m_per_block_h,
        climate: &climate,
        params: &params,
        width_scale: s.feature_scale.rivers,
        mouths: params.deltas,
    });
    // The delta plains on the first network's mouths, then the network again on the raised
    // ground and the distributaries across each plain.
    let plains = if params.deltas {
        delta::grow(
            &delta::Input {
                grid: &grid,
                elev: &elev,
                sea_raw,
                m_per_block_h: s.m_per_block_h,
                m_per_block_v: s.m_per_block_v,
                fit: s.build_fit(),
                params: &params,
                width_scale: s.feature_scale.rivers,
            },
            &first,
        )
    } else {
        delta::Plains::default()
    };
    let first_chains = first.rivers.chains.len();
    let (result, splits, laid) = if plains.plains.is_empty() {
        (first, Vec::new(), None)
    } else {
        drop(first);
        plains.apply(&mut elev);
        let mut second = network::run(&network::Input {
            grid: &grid,
            elev: &elev,
            sea_raw,
            m_per_block_h: s.m_per_block_h,
            climate: &climate,
            params: &params,
            width_scale: s.feature_scale.rivers,
            mouths: true,
        });
        let splits = delta::distributaries(
            &delta::Input {
                grid: &grid,
                elev: &elev,
                sea_raw,
                m_per_block_h: s.m_per_block_h,
                m_per_block_v: s.m_per_block_v,
                fit: s.build_fit(),
                params: &params,
                width_scale: s.feature_scale.rivers,
            },
            &elev,
            &plains,
            &mut second,
        );
        drop(elev);
        let laid = lay_plains(p, out, &l3, &grid, halvings, &plains)?;
        (second, splits, Some(laid))
    };
    let mut stats = result.stats.clone();
    stats["chains"] = json!(result.rivers.chains.len());
    stats["vertices"] = json!(result.rivers.vertices.len());
    stats["index_entries"] = json!(result.rivers.entries.len());
    if params.deltas {
        let cell_km2 = (f64::from(grid.r) * s.m_per_block_h).powi(2) / 1e6;
        let land = stats["land_cells"].as_u64().unwrap_or(0);
        let mut d = delta::stats(&plains, &splits, land, cell_km2);
        d["first_pass_chains"] = json!(first_chains);
        if let Some(l) = &laid {
            d["raised_cells"] = json!(l.cells);
            d["max_raise_blocks"] = json!(l.max_blocks);
            d["amplitude_zeroed"] = json!(l.amplitude_zeroed);
            d["stack_raised_cells"] = json!(l.stack_cells);
            d["stack_max_raise_blocks"] = json!(l.stack_max_blocks);
        }
        stats["deltas"] = d;
    }
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
    tally_once(out, "water_level");

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

    // closed_basin (ledger: BAK-055): 1 where the game puts no sea on ground below sea level
    // (HYD-100, HYD-102; FORMAT.md, "Water columns", step 3).
    out.layer(
        desc(CLOSED_BASIN, DType::U8, &grid, 1.0, "flag", None),
        &result.closed_basin,
    )?;

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
        "method": if params.deltas { "hyd-130" } else { "hyd-100" },
        "params": params,
        "width_scale": s.feature_scale.rivers,
        "grid": grid.name,
        "halved": halved,
        "climate": if from_stage { "climate stage (L1, bilinear)" } else { "zonal stand-in" },
        "seasons": if seasonal { "the twelve monthly layers" } else { "mean ± 8 °C" },
        "playa": {"layer": PLAYA_LAYER, "rock_class": PLAYA_ROCK_CLASS, "blocks": salt},
        "closed_basin": {"layer": CLOSED_BASIN, "reach_blocks": hydro::CLOSED_REACH_BLOCKS},
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
    let d = &s["deltas"];
    let deltas = if d.is_null() {
        String::new()
    } else {
        format!(
            " deltas={} delta_km2={} delta_share_of_land={} distributaries={} lagoons={}",
            d["deltas"], d["plain_km2"], d["share_of_land"], d["distributaries_added"], d["lagoons"],
        )
    };
    format!(
        "water grid={} lakes={} lake_km2={} share_of_land={} largest_km2={} below_rim={} frozen={} playas={} dry={} endorheic={} below_sea_closed_km2={} below_sea_dry_km2={} below_sea_lakes={} chains={} vertices={} river_km={} max_q={} widest={} bytes={}{deltas}",
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
        s["below_sea"]["closed_km2"],
        s["below_sea"]["dry_km2"],
        s["below_sea"]["lakes_below_sea_level"],
        s["chains"],
        s["vertices"],
        s["river_length_km"],
        s["max_discharge_m3s"],
        s["widest_blocks"],
        v["rivers_bytes"],
    )
}

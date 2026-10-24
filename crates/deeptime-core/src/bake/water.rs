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
//! - **Floodplains** (V1 gap 5, stage 2; HYD-150 to HYD-152). With `hydro.floodplains` the last
//!   network run also finds the floodplains (GFPLAIN: the cells draining to a river cell whose
//!   ground lies at most the flood depth h = 0.01 A^0.30 m over that cell's), and the bake lays each
//!   as a flat at its river's bank, one block over the river's water: `elevation` is set to that
//!   level on the plain (each L3 cell of a plain cell), `detail/amplitude` is zeroed there, and the
//!   layer [`FLOODPLAIN_LAYER`] keeps what the plain moved the ground by, so a re-run takes it off
//!   first. The delta plains are left as they were laid.
//! - **Meanders** (V1 gap 5, stage 3; HYD-155 to HYD-159; [`hydro::meander`]). With
//!   `hydro.meanders` (and the floodplains) each reach of a chain on the plain just laid that is
//!   gentler than the braided line and has room is redrawn as a sine-generated curve, measured on
//!   the planet (the Lambert metric on a Lambert folder). Only the river file changes: no layer,
//!   no ground; the plain is already one block over the water wherever the new channel runs.
//! - **Fans** (V1 gap 5, stage 4; HYD-165 to HYD-170; [`hydro::fan`]). With `hydro.fans` the
//!   first network's chains are searched for where a river leaves ground steeper than its fan for
//!   gentler, and a cone of Bull's area and slope for its drainage is laid there before the second
//!   run (which then also runs when there are fans and no delta plain): the second network drains
//!   the cones, finds no floodplain on them, and the raise goes into `elevation` (the cone at each
//!   L3 cell), `detail/amplitude` is zeroed there, the layer [`FAN_LAYER`] keeps what was added (a
//!   re-run takes it off), and a Lambert world's stacks are raised, as for the delta plains.
//! - **Braids** (V1 gap 5, stage 4; HYD-171 to HYD-175; [`hydro::braid`]). With `hydro.braids`,
//!   after the meanders, a reach on a fan, or on the floodplain and steeper than the braided line,
//!   divides around islands into two threads: the river file only (each island's second thread a
//!   chain from and into its course).

use std::collections::HashMap;
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

/// What the floodplains moved `elevation` by, 1/16 block, on its grid (nodata off the plains; 0 on a
/// plain cell whose ground was at the plain's level already).
pub const FLOODPLAIN_LAYER: &str = "floodplain";

/// What the alluvial fans added to `elevation`, 1/16 block, on its grid (0 off the fans).
pub const FAN_LAYER: &str = "fan";

/// Whether the folder being drained is in the simulation's Lambert frame, so lengths on it go
/// through the Lambert metric: a fresh bake's manifest has no layout block yet (the session writes
/// it when the bake ends), and a baked Lambert folder that `deeptime hydro` drains again has one
/// of kind `lambert`. Testing only for a missing block measured a re-run in the grid's blocks.
pub(crate) fn lambert_frame(m: &deeptime_format::Manifest) -> bool {
    m.projection
        .as_ref()
        .is_none_or(|s| s.kind == ProjectionKind::Lambert)
}

/// Drops every listing of `name` in `out.layers` but the first (`Out::layer` tallies a rewritten
/// layer again).
fn tally_once(out: &mut Out<'_>, name: &str) {
    let mut seen = false;
    out.layers
        .retain(|l| l != name || !std::mem::replace(&mut seen, true));
}

/// Takes a previous run's delta plains or fans (the raise layer `name`) off `elevation` (a re-run
/// over a baked folder) and drops the layer: the hydrology starts from the ground the history left.
fn take_off_raise(out: &mut Out<'_>, name: &str) -> Result<(), CoreError> {
    let Some(d) = out.m.layer(name).cloned() else {
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
    out.m.layers.retain(|l| l.name != name);
    out.layers.retain(|l| l != name);
    Ok(())
}

/// Takes a previous run's floodplains off `elevation` (a re-run over a baked folder) and drops the
/// floodplain layer, as [`take_off_raise`] does for the delta plains.
fn take_off_floodplains(out: &mut Out<'_>) -> Result<(), CoreError> {
    let Some(d) = out.m.layer(FLOODPLAIN_LAYER).cloned() else {
        return Ok(());
    };
    let path = out.dir.join(&d.path);
    let (dg, moved) = load_i16(&path)?;
    if let Some(e) = out.m.layer("elevation").cloned() {
        let mut el = crate::bake::load_layer(&out.dir.join(&e.path))?;
        if el.desc.grid == dg && el.desc.dtype == DType::I16 {
            el.bytes
                .as_chunks_mut::<2>()
                .0
                .par_iter_mut()
                .zip(moved.par_iter())
                .for_each(|(b, &m)| {
                    if m != network::NODATA && m != 0 {
                        *b = i16::from_le_bytes(*b).saturating_sub(m).to_le_bytes();
                    }
                });
            out.layer_bytes(el.desc, &el.bytes)?;
            tally_once(out, "elevation");
        }
    }
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| CoreError::io(&path, e))?;
    }
    out.m.layers.retain(|l| l.name != FLOODPLAIN_LAYER);
    out.layers.retain(|l| l != FLOODPLAIN_LAYER);
    Ok(())
}

/// Sets `elevation` (grid `l3`) to the floodplains' level (on the hydrology's grid, `halvings` times
/// coarser; [`network::Output::floodplain`]): each L3 cell of a plain cell takes the plain's
/// height (HYD-151). Writes what that moved as [`FLOODPLAIN_LAYER`], zeroes `detail/amplitude` on
/// the plain (it is flat), and on a Lambert world raises the horizon stacks' tops over the cells it
/// raised (a lowered cell's stack is cut by the ground, as erosion cuts it). Returns the record.
fn lay_floodplains(
    p: &PlanetPreset,
    out: &mut Out<'_>,
    l3: &GridDesc,
    hydro_grid: &GridDesc,
    halvings: u32,
    level: &[i16],
) -> Result<Value, CoreError> {
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
    let mut moved = vec![network::NODATA; l3.cells()];
    let (mut cells, mut raised, mut lowered, mut max_up, mut max_down) = (0u64, 0u64, 0u64, 0, 0);
    for (c, &lv) in level.iter().enumerate() {
        if lv == network::NODATA {
            continue;
        }
        let (i, j) = (c % wh, c / wh);
        for dj in 0..f {
            for di in 0..f {
                let c3 = (j * f + dj) * w3 + i * f + di;
                let at = c3 * 2;
                let old = i16::from_le_bytes([el.bytes[at], el.bytes[at + 1]]);
                let dz = (i32::from(lv) - i32::from(old))
                    .clamp(i32::from(i16::MIN) + 1, i32::from(i16::MAX));
                moved[c3] = dz as i16;
                cells += 1;
                if dz > 0 {
                    raised += 1;
                    max_up = max_up.max(dz);
                } else if dz < 0 {
                    lowered += 1;
                    max_down = max_down.max(-dz);
                }
                el.bytes[at..at + 2].copy_from_slice(&lv.to_le_bytes());
            }
        }
    }
    out.layer_bytes(el.desc.clone(), &el.bytes)?;
    tally_once(out, "elevation");
    out.layer(
        desc(
            FLOODPLAIN_LAYER,
            DType::I16,
            l3,
            1.0 / 16.0,
            "block",
            Some(f64::from(network::NODATA)),
        ),
        &moved,
    )?;

    // A floodplain is flat: no detail noise on it.
    let mut amplitude_zeroed = 0u64;
    if let Some(a) = out.m.layer("detail/amplitude").cloned() {
        let mut amp = crate::bake::load_layer(&out.dir.join(&a.path))?;
        if amp.desc.grid == *l3 && amp.desc.dtype == DType::U8 {
            for (b, &m) in amp.bytes.iter_mut().zip(&moved) {
                if m != network::NODATA && *b != 0 {
                    *b = 0;
                    amplitude_zeroed += 1;
                }
            }
            if amplitude_zeroed > 0 {
                out.layer_bytes(amp.desc, &amp.bytes)?;
                tally_once(out, "detail/amplitude");
            }
        }
    }

    // The horizon stacks' tops over the raised cells (a Lambert world; see `lay_plains`).
    let (mut stack_cells, mut stack_max) = (0u64, 0.0);
    if raised > 0
        && p.scale.projection == ProjectionKind::Lambert
        && let Some(st) = out.m.layer("strata/horizons").cloned()
    {
        const MASK: i16 = i16::MIN;
        let mut masked = el;
        masked.desc.nodata = Some(f64::from(MASK));
        for (b, &m) in masked.bytes.as_chunks_mut::<2>().0.iter_mut().zip(&moved) {
            if m == network::NODATA || m <= 0 {
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
        stack_cells = n;
        stack_max = max;
        if n > 0 {
            out.layer_bytes(stack.desc, &stack.bytes)?;
            tally_once(out, "strata/horizons");
        }
    }
    Ok(json!({
        "l3_cells": cells,
        "raised_cells": raised,
        "lowered_cells": lowered,
        "max_raise_blocks": f64::from(max_up) / 16.0,
        "max_lowering_blocks": f64::from(max_down) / 16.0,
        "amplitude_zeroed": amplitude_zeroed,
        "stack_raised_cells": stack_cells,
        "stack_max_raise_blocks": stack_max,
    }))
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

/// Raises `elevation` (grid `l3`) to the fans' cones (their cells on the hydrology's grid,
/// `halvings` times coarser; the cone evaluated at each L3 cell, so a fan is round on the planet
/// at L3's spacing too). Writes the raise as [`FAN_LAYER`], zeroes `detail/amplitude` on the
/// raised cells (the cone is the fan's surface), and on a Lambert world raises the horizon stacks'
/// tops over them, as [`lay_plains`] does. Returns the record.
fn lay_fans(
    p: &PlanetPreset,
    out: &mut Out<'_>,
    l3: &GridDesc,
    hydro_grid: &GridDesc,
    halvings: u32,
    fans: &hydro::fan::Fans,
) -> Result<Value, CoreError> {
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
    let (ox, oz, r3) = (
        l3.origin_block_x as f64,
        l3.origin_block_z as f64,
        f64::from(l3.r),
    );
    // Each L3 cell's highest cone.
    let mut top: HashMap<usize, i16> = HashMap::new();
    for fan in &fans.fans {
        for &c in &fan.cells {
            let (i, j) = (c as usize % wh, c as usize / wh);
            for dj in 0..f {
                for di in 0..f {
                    let (i3, j3) = (i * f + di, j * f + dj);
                    let (x, z) = (ox + i3 as f64 * r3, oz + j3 as f64 * r3);
                    let cone = fans
                        .cone_raw(fan, x, z)
                        .floor()
                        .clamp(f64::from(i16::MIN), f64::from(i16::MAX))
                        as i16;
                    let t = top.entry(j3 * w3 + i3).or_insert(i16::MIN);
                    *t = (*t).max(cone);
                }
            }
        }
    }
    let mut raise = vec![0i16; l3.cells()];
    let (mut cells, mut max_raw) = (0u64, 0i16);
    for (&c3, &h) in &top {
        let at = c3 * 2;
        let old = i16::from_le_bytes([el.bytes[at], el.bytes[at + 1]]);
        if h > old {
            raise[c3] = h - old;
            max_raw = max_raw.max(h - old);
            cells += 1;
            el.bytes[at..at + 2].copy_from_slice(&h.to_le_bytes());
        }
    }
    // The hollows the cones close, at their spill (HYD-177).
    for &(c, lvl) in &fans.ponds {
        let (i, j) = (c as usize % wh, c as usize / wh);
        for dj in 0..f {
            for di in 0..f {
                let t = top.entry((j * f + dj) * w3 + i * f + di).or_insert(i16::MIN);
                *t = (*t).max(lvl);
            }
        }
    }
    drop(top);
    out.layer_bytes(el.desc.clone(), &el.bytes)?;
    tally_once(out, "elevation");
    out.layer(
        desc(FAN_LAYER, DType::I16, l3, 1.0 / 16.0, "block", None),
        &raise,
    )?;

    let mut amplitude_zeroed = 0u64;
    if let Some(a) = out.m.layer("detail/amplitude").cloned() {
        let mut amp = crate::bake::load_layer(&out.dir.join(&a.path))?;
        if amp.desc.grid == *l3 && amp.desc.dtype == DType::U8 {
            for (b, &r) in amp.bytes.iter_mut().zip(&raise) {
                if r > 0 && *b != 0 {
                    *b = 0;
                    amplitude_zeroed += 1;
                }
            }
            if amplitude_zeroed > 0 {
                out.layer_bytes(amp.desc, &amp.bytes)?;
                tally_once(out, "detail/amplitude");
            }
        }
    }

    let (mut stack_cells, mut stack_max) = (0u64, 0.0);
    if cells > 0
        && p.scale.projection == ProjectionKind::Lambert
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
        stack_cells = n;
        stack_max = max;
        if n > 0 {
            out.layer_bytes(stack.desc, &stack.bytes)?;
            tally_once(out, "strata/horizons");
        }
    }
    Ok(json!({
        "raised_cells": cells,
        "max_raise_blocks": f64::from(max_raw) / 16.0,
        "amplitude_zeroed": amplitude_zeroed,
        "stack_raised_cells": stack_cells,
        "stack_max_raise_blocks": stack_max,
    }))
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
    // In the reverse of the order they were laid in.
    take_off_floodplains(out)?;
    take_off_raise(out, FAN_LAYER)?;
    take_off_raise(out, DELTA_LAYER)?;
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
        floodplains: params.floodplains.then_some(network::Floodplains {
            m_per_block_v: s.m_per_block_v,
            skip: &[],
        }),
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
    let mut on_plains: Vec<u32> = plains
        .plains
        .iter()
        .flat_map(|pl| pl.cells.iter().copied())
        .collect();
    on_plains.sort_unstable();
    on_plains.dedup();
    // The alluvial fans where the first network's rivers leave steep ground, off the delta plains
    // (HYD-165 to HYD-170).
    let lambert = lambert_frame(out.m).then(|| deeptime_format::Projection::of(c));
    let mut fans = if params.fans {
        let taken = |cell: u32| on_plains.binary_search(&cell).is_ok();
        hydro::fan::find(
            &hydro::fan::Input {
                grid: &grid,
                elev: &elev,
                sea_raw,
                m_per_block_h: s.m_per_block_h,
                m_per_block_v: s.m_per_block_v,
                lambert,
                taken: &taken,
            },
            &first,
        )
    } else {
        hydro::fan::Fans::default()
    };
    let (mut result, splits, laid, fan_laid) = if plains.plains.is_empty() && fans.fans.is_empty() {
        (first, Vec::new(), None, None)
    } else {
        plains.apply(&mut elev);
        if !fans.fans.is_empty() {
            // No hollow on or behind a fan: the deposit fills what the cones close (HYD-177).
            let before = elev.clone();
            fans.apply(&mut elev);
            fans.fill_ponds(
                &grid,
                &before,
                &mut elev,
                sea_raw,
                &first.closed_basin,
                &first.kind,
            );
        }
        drop(first);
        // No floodplain on a delta plain or a fan: both are laid already.
        let mut skip = on_plains.clone();
        skip.extend(fans.cells());
        skip.sort_unstable();
        skip.dedup();
        let mut second = network::run(&network::Input {
            grid: &grid,
            elev: &elev,
            sea_raw,
            m_per_block_h: s.m_per_block_h,
            climate: &climate,
            params: &params,
            width_scale: s.feature_scale.rivers,
            mouths: params.deltas,
            floodplains: params.floodplains.then_some(network::Floodplains {
                m_per_block_v: s.m_per_block_v,
                skip: &skip,
            }),
        });
        let splits = if plains.plains.is_empty() {
            Vec::new()
        } else {
            delta::distributaries(
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
            )
        };
        drop(elev);
        let laid = if plains.plains.is_empty() {
            None
        } else {
            Some(lay_plains(p, out, &l3, &grid, halvings, &plains)?)
        };
        let fan_laid = if fans.fans.is_empty() {
            None
        } else {
            Some(lay_fans(p, out, &l3, &grid, halvings, &fans)?)
        };
        (second, splits, laid, fan_laid)
    };
    let flood_laid = if params.floodplains && !result.floodplain.is_empty() {
        Some(lay_floodplains(
            p,
            out,
            &l3,
            &grid,
            halvings,
            &result.floodplain,
        )?)
    } else {
        None
    };
    // The meanders on the plain just laid (after the distributaries, which index the stem's
    // vertices by its cells; HYD-155 to HYD-159).
    let (ox, oz, gr) = (
        grid.origin_block_x as f64,
        grid.origin_block_z as f64,
        f64::from(grid.r),
    );
    let (gw, gh, wraps) = (i64::from(grid.w), i64::from(grid.h), grid.wraps_x);
    // The hydrology's cell at block (x, z), None off the grid.
    let cell_at = move |x: f64, z: f64| -> Option<usize> {
        let (i, j) = (
            ((x - ox) / gr).round() as i64,
            ((z - oz) / gr).round() as i64,
        );
        if j < 0 || j >= gh || (!wraps && (i < 0 || i >= gw)) {
            return None;
        }
        Some((j * gw + i.rem_euclid(gw)) as usize)
    };
    let fp = std::mem::take(&mut result.floodplain);
    let plain = |x: f64, z: f64| -> Option<i32> {
        let v = *fp.get(cell_at(x, z)?)?;
        (v != network::NODATA).then_some(i32::from(v))
    };
    let meandered = if params.meanders && params.floodplains && !fp.is_empty() {
        Some(hydro::meander::apply(
            &mut result.rivers,
            &hydro::meander::Input {
                plain: &plain,
                lambert,
                m_per_block_h: s.m_per_block_h,
                m_per_block_v: s.m_per_block_v,
            },
        ))
    } else {
        None
    };
    // The braids on the fans and the steep reaches of the plain, after the meanders (HYD-171 to
    // HYD-175).
    let braided = if params.braids && (!fp.is_empty() || !fans.fans.is_empty()) {
        // The fan's ground (its highest cone) on each fan cell, blocks.
        let ground = fans.ground();
        let fan = |x: f64, z: f64| -> Option<f64> {
            let cl = cell_at(x, z)? as u32;
            let i = ground.binary_search_by_key(&cl, |e| e.0).ok()?;
            Some(f64::from(ground[i].1) / 16.0)
        };
        Some(hydro::braid::apply(
            &mut result.rivers,
            &hydro::braid::Input {
                plain: &plain,
                fan: &fan,
                lambert,
                m_per_block_h: s.m_per_block_h,
                m_per_block_v: s.m_per_block_v,
            },
        ))
    } else {
        None
    };
    result.floodplain = fp;
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
    if let Some(l) = flood_laid {
        stats["floodplains"]["laid"] = l;
    }
    if let Some(m) = meandered {
        stats["meanders"] = m;
    }
    if params.fans {
        let cell_km2 = (f64::from(grid.r) * s.m_per_block_h).powi(2) / 1e6;
        let land = stats["land_cells"].as_u64().unwrap_or(0);
        let mut f = hydro::fan::stats(&fans, land, cell_km2, s.m_per_block_v);
        if let Some(l) = fan_laid {
            f["laid"] = l;
        }
        stats["fans"] = f;
    }
    if let Some(b) = braided {
        stats["braids"] = b;
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
    // No wetland on or beside a playa (ledger: BIO-178).
    let dried = dry_beside_playas(out, &grid, &result.kind, c)?;

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
        "method": if params.fans || params.braids {
            "hyd-165"
        } else if params.floodplains && params.meanders {
            "hyd-155"
        } else if params.floodplains {
            "hyd-150"
        } else if params.deltas {
            "hyd-130"
        } else {
            "hyd-100"
        },
        "params": params,
        "width_scale": s.feature_scale.rivers,
        "grid": grid.name,
        "halved": halved,
        "climate": if from_stage { "climate stage (L1, bilinear)" } else { "zonal stand-in" },
        "seasons": if seasonal { "the twelve monthly layers" } else { "mean ± 8 °C" },
        "playa": {"layer": PLAYA_LAYER, "rock_class": PLAYA_ROCK_CLASS, "blocks": salt, "wetland_cleared": dried},
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

/// The wetland and erosion layers the playas clear ([`super::biomes::dry_beside_playas`]).
const WETLAND: &str = "climate/wetland";
const EROSION: &str = "climate/erosion";

/// No wetland on or beside a playa (ledger: BIO-178): the playa cells of the hydrology's `grid`
/// (`kind`) clear the L2 wetland flag within [`super::biomes::PLAYA_WETLAND_MARGIN_BLOCKS`] of
/// ground, plus two L2 cells (the game's bilinear read between nodes, and the projection's
/// nearest-cell read of the Lambert node) and a hydrology cell (the projection's footprint read of
/// the playa), and rewrite `climate/wetland` and `climate/erosion`. In the bake the folder is the
/// Lambert one (the projection then resamples both); `deeptime hydro` on a projected world clears
/// on the world's own grids. The record for `notes.bake.water.playa.wetland_cleared`.
fn dry_beside_playas(
    out: &mut Out<'_>,
    grid: &GridDesc,
    kind: &[u8],
    circumference: f64,
) -> Result<Value, CoreError> {
    let w = grid.w as usize;
    let (ox, oz, r) = (grid.origin_block_x, grid.origin_block_z, i64::from(grid.r));
    let playas: Vec<(i64, i64)> = kind
        .iter()
        .enumerate()
        .filter(|&(_, &k)| k == network::kind::PLAYA)
        .map(|(cell, _)| (ox + (cell % w) as i64 * r, oz + (cell / w) as i64 * r))
        .collect();
    let (Some(we), Some(ee)) = (out.m.layer(WETLAND).cloned(), out.m.layer(EROSION).cloned())
    else {
        return Ok(json!({"skipped": "no climate/wetland or climate/erosion layer"}));
    };
    if playas.is_empty() {
        return Ok(json!({"playa_cells": 0, "nodes": 0}));
    }
    let mut wet = crate::bake::load_layer(&out.dir.join(&we.path))?;
    let mut ero = crate::bake::load_layer(&out.dir.join(&ee.path))?;
    if wet.desc.dtype != DType::U8 || ero.desc.dtype != DType::I16 || wet.desc.grid != ero.desc.grid
    {
        return Err(CoreError::InvalidState(format!(
            "{WETLAND} ({:?} on {}) and {EROSION} ({:?} on {}) are not u8 and i16 on one grid",
            wet.desc.dtype, wet.desc.grid.name, ero.desc.dtype, ero.desc.grid.name
        )));
    }
    let reach = super::biomes::PLAYA_WETLAND_MARGIN_BLOCKS
        + 2.0 * f64::from(wet.desc.grid.r)
        + f64::from(grid.r);
    let lambert = lambert_frame(out.m).then_some(circumference);
    let cleared = super::biomes::dry_beside_playas(
        &mut wet.bytes,
        &mut ero.bytes,
        &wet.desc.grid,
        &playas,
        reach,
        lambert,
    );
    if cleared > 0 {
        for (layer, old) in [(wet, we.bytes), (ero, ee.bytes)] {
            let name = layer.desc.name.clone();
            out.layer_bytes(layer.desc, &layer.bytes)?;
            // `Out::layer_bytes` tallied the layer and its bytes a second time.
            out.bytes = out.bytes.saturating_sub(old);
            let mut seen = false;
            out.layers
                .retain(|l| *l != name || !std::mem::replace(&mut seen, true));
        }
    }
    Ok(json!({
        "playa_cells": playas.len(),
        "reach_blocks": reach,
        "lambert": lambert.is_some(),
        "nodes": cleared,
    }))
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
            d["deltas"],
            d["plain_km2"],
            d["share_of_land"],
            d["distributaries_added"],
            d["lagoons"],
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

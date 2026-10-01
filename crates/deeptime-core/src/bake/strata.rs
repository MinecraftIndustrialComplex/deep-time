// SPDX-License-Identifier: LGPL-3.0-or-later
//! The strata part of the bake: `strata/horizons` (L1, K horizons per cell) and
//! `strata/classes.json`, from P1-07's horizon bake (`structure::bake_horizons`).
//!
//! **The top horizon reaches over the amplified terrain.** P1-07 puts each cell's first horizon
//! top at the deep-time surface (L1). The terrain the game builds is the L3 `elevation` plus the
//! detail term, which rises above the L1 surface on amplified ridges, and the strata pass paints
//! only stone below a horizon's top: without this step, ridge crests would stay unclassified
//! stone. So the first horizon's top of every cell is raised to at least the highest terrain any
//! block interpolating from that cell can have — the largest `elevation` plus the largest
//! `detail/amplitude` over the L3 cells within two L1 cells of it — plus one block. The strata
//! pass reads tops bilinearly between the four cells around a block, each of which covers that
//! block's L3 neighbourhood, so the interpolated top is never below the terrain. Only the first
//! horizon changes, and only upwards: deeper horizon tops stay where the history put them, so
//! valleys cut below them expose the rock beneath, as they should.

use deeptime_format::TilePackReader;
use rayon::prelude::*;
use serde_json::{Value, json};

use super::Out;
use super::terrain::Terrain;
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::structure::bake_horizons::{self, Horizons, NO_TOP};
use crate::world::WorldState;

/// Blocks added above the highest terrain a cell's top horizon must cover.
pub const TOP_MARGIN_BLOCKS: i32 = 1;

pub fn bake(
    world: &WorldState,
    p: &PlanetPreset,
    terrain: &Terrain,
    out: &mut Out<'_>,
) -> Result<Value, CoreError> {
    let sp = p.structure.clone().unwrap_or_default();
    let mut hz = bake_horizons::bake(world, p)?;
    hz.check()
        .map_err(|e| CoreError::InvalidState(format!("the horizon bake failed its check: {e}")))?;
    let (raised, max_raise, limit) = raise_tops(
        &mut hz,
        terrain,
        p.scale.max_y,
        p.scale.sea_level_y,
        p.scale.build_fit().knee_y,
    )?;
    hz.check().map_err(|e| {
        CoreError::InvalidState(format!(
            "the horizons failed their check after the raise: {e}"
        ))
    })?;
    let layer = hz.to_layer(deeptime_format::Codec::Lz4Block)?;
    let mean = hz.mean_horizons();
    let k = hz.k;
    out.layer_bytes(layer.desc, &layer.bytes)?;
    out.json(
        deeptime_format::column::CLASSES_PATH,
        &Horizons::classes_json(),
    )?;
    Ok(json!({
        "grid": hz.grid,
        "k": k,
        "mean_horizons": mean,
        "top_raised_cells": raised,
        "top_raise_blocks_max": max_raise,
        "build_limit": limit.note(p.scale.max_y, p.scale.build_fit()),
        "top_rule": "the first horizon's top is at least the highest elevation plus the highest detail amplitude over the L3 cells within two L1 cells, plus one block",
        "smoothing": {
            "sigma_cells": sp.strata_smoothing_cells,
            "sample_step_cells": bake_horizons::smooth_stride(sp.strata_smoothing_cells),
            "radius_samples": bake_horizons::SMOOTH_RADIUS_SAMPLES,
            "tongue_band_keys": bake_horizons::tongue_params(&sp).0,
            "tongue_minor_share": sp.strata_minor_share,
            "rule": "each cell's stack is the Gaussian-weighted mean of its neighbours' height functions, cut at every key any of them has, each interval the class of the largest weighted thickness; the cell's own surface stays its first top",
        },
    }))
}

/// How much of the baked `elevation` stands above the world's build limit (`scale.max_y`): the game
/// cannot place those blocks, so such ground ends in a flat top at the limit (the column
/// evaluator's `top_y` is clamped there). Nothing in the pipeline bounds the terrain to the build
/// range; the bake counts it so a run says so (`notes.bake.terrain.build_limit`, the `bake` line).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BuildLimit {
    /// L3 cells whose elevation is above `max_y`.
    pub cells_above: u64,
    /// L3 cells above the compression knee: their stored elevation is not the simulated one.
    pub cells_compressed: u64,
    /// L3 cells at or above sea level.
    pub land_cells: u64,
    /// The highest elevation, blocks (y); −∞ when no cell has a value.
    pub highest_y: f64,
}

impl BuildLimit {
    const NONE: BuildLimit = BuildLimit {
        cells_above: 0,
        cells_compressed: 0,
        land_cells: 0,
        highest_y: f64::NEG_INFINITY,
    };

    fn merge(self, o: BuildLimit) -> BuildLimit {
        BuildLimit {
            cells_above: self.cells_above + o.cells_above,
            cells_compressed: self.cells_compressed + o.cells_compressed,
            land_cells: self.land_cells + o.land_cells,
            highest_y: self.highest_y.max(o.highest_y),
        }
    }

    pub fn note(&self, max_y: i32, fit: crate::params::BuildFit) -> Value {
        let share = if self.land_cells == 0 {
            0.0
        } else {
            self.cells_above as f64 / self.land_cells as f64
        };
        json!({
            "max_y": max_y,
            "cells_above": self.cells_above,
            "knee_y": fit.knee_y,
            "top_y": fit.top_y,
            "cells_compressed": self.cells_compressed,
            "rule_compressed": "elevation cells above knee_y: stored compressed towards top_y (scale.build_limit_band_blocks), never above it",
            "land_cells": self.land_cells,
            "share_of_land": share,
            "highest_y": if self.highest_y.is_finite() { json!(self.highest_y) } else { Value::Null },
            "rule": "elevation cells above max_y (the detail term not counted); the game ends such ground at max_y",
        })
    }
}

/// Per L1 cell, the highest L3 terrain its interpolation reaches, and the first horizon's top
/// raised to it. Returns (cells raised, largest raise in blocks, the elevation against the build
/// limit).
fn raise_tops(
    hz: &mut Horizons,
    t: &Terrain,
    max_y: i32,
    sea_level_y: i32,
    knee_y: f64,
) -> Result<(u64, f64, BuildLimit), CoreError> {
    let g1 = hz.grid.clone();
    let (w1, h1) = (g1.w as usize, g1.h as usize);
    let l3 = &t.l3;
    if !g1.r.is_multiple_of(l3.r) {
        return Err(CoreError::InvalidState(format!(
            "L1 r {} is not a multiple of L3 r {}",
            g1.r, l3.r
        )));
    }
    let ratio = (g1.r / l3.r) as usize;
    // Highest elevation and highest amplitude of the L3 cells in each L1 cell's square
    // [x1, x1 + r1) × [z1, z1 + r1), in blocks.
    let (limit_y, sea_y) = (f64::from(max_y), f64::from(sea_level_y));
    type Tile = (usize, usize, usize, Vec<f64>, BuildLimit);
    let square = |path: &std::path::Path| -> Result<(Vec<f64>, BuildLimit), CoreError> {
        let r = TilePackReader::open(path)?;
        let d = r.desc().clone();
        let mut best = vec![f64::NEG_INFINITY; w1 * h1];
        // Each tile covers whole L1 squares (its edge, 256 cells, is a multiple of the ratio), so
        // tiles fill disjoint parts of `best`.
        let tiles: Vec<Tile> = (0..d.tile_count())
            .into_par_iter()
            .map(|tix| -> Result<Tile, CoreError> {
                let bytes = r.read_tile(tix)?;
                let (x0, z0, tw, th) = d.tile_rect(tix);
                let cb = d.cell_bytes();
                let (i0, j0) = (x0 as usize / ratio, z0 as usize / ratio);
                let lw = (tw as usize).div_ceil(ratio);
                let lh = (th as usize).div_ceil(ratio);
                let mut local = vec![f64::NEG_INFINITY; lw * lh];
                let mut limit = BuildLimit::NONE;
                for lj in 0..th as usize {
                    let sj = (z0 as usize + lj) / ratio - j0;
                    for li in 0..tw as usize {
                        let si = (x0 as usize + li) / ratio - i0;
                        let at = (lj * tw as usize + li) * cb;
                        let v = d.physical(d.dtype.decode_scalar(&bytes[at..at + cb]));
                        let c = sj * lw + si;
                        if v.is_finite() && v > local[c] {
                            local[c] = v;
                        }
                        if v.is_finite() {
                            limit.cells_above += u64::from(v > limit_y);
                            limit.cells_compressed += u64::from(v > knee_y);
                            limit.land_cells += u64::from(v >= sea_y);
                            limit.highest_y = limit.highest_y.max(v);
                        }
                    }
                }
                Ok((i0, j0, lw, local, limit))
            })
            .collect::<Result<_, _>>()?;
        let mut limit = BuildLimit::NONE;
        for (i0, j0, lw, local, l) in tiles {
            limit = limit.merge(l);
            for (k, v) in local.into_iter().enumerate() {
                let (i, j) = (i0 + k % lw, j0 + k / lw);
                if i < w1 && j < h1 {
                    let b = &mut best[j * w1 + i];
                    if v > *b {
                        *b = v;
                    }
                }
            }
        }
        Ok((best, limit))
    };
    let (elev, limit) = square(&t.elevation)?;
    let amp = match &t.amplitude {
        Some(a) => square(a)?.0,
        None => vec![0.0; w1 * h1],
    };
    // Over the 4 × 4 squares i − 2 ..= i + 1, j − 2 ..= j + 1 (x wraps, z clamps).
    let window = |v: &[f64], i: usize, j: usize| -> f64 {
        let mut m = f64::NEG_INFINITY;
        for dj in -2i64..=1 {
            let jj = (j as i64 + dj).clamp(0, h1 as i64 - 1) as usize;
            for di in -2i64..=1 {
                let ii = (i as i64 + di).rem_euclid(w1 as i64) as usize;
                m = m.max(v[jj * w1 + ii]);
            }
        }
        m
    };
    let k = hz.k as usize;
    let tops: Vec<(usize, i16, f64)> = (0..w1 * h1)
        .into_par_iter()
        .filter_map(|c| {
            let (i, j) = (c % w1, c / w1);
            let e = window(&elev, i, j);
            let a = window(&amp, i, j).max(0.0);
            if !e.is_finite() {
                return None;
            }
            let need = ((e + a).ceil() as i32 + TOP_MARGIN_BLOCKS).min(max_y) * 16;
            let top = hz.tops[c * k];
            if top == NO_TOP || i32::from(top) >= need {
                return None;
            }
            let raise = f64::from(need - i32::from(top)) / 16.0;
            Some((
                c,
                need.clamp(i32::from(i16::MIN) + 1, i32::from(i16::MAX)) as i16,
                raise,
            ))
        })
        .collect();
    let mut max_raise = 0.0f64;
    for &(c, top, raise) in &tops {
        hz.tops[c * k] = top;
        max_raise = max_raise.max(raise);
    }
    Ok((tops.len() as u64, max_raise, limit))
}

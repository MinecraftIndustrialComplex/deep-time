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
    let mut hz = bake_horizons::bake(world, p)?;
    hz.check()
        .map_err(|e| CoreError::InvalidState(format!("the horizon bake failed its check: {e}")))?;
    let (raised, max_raise) = raise_tops(&mut hz, terrain, p.scale.max_y)?;
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
        "top_rule": "the first horizon's top is at least the highest elevation plus the highest detail amplitude over the L3 cells within two L1 cells, plus one block",
        "smoothing": {
            "sigma_cells": bake_horizons::SMOOTH_SIGMA_CELLS,
            "radius_cells": bake_horizons::SMOOTH_RADIUS_CELLS,
            "rule": "each cell's stack is the Gaussian-weighted mean of its neighbours' height functions, cut at every key any of them has, each interval the class of the largest weighted thickness; the cell's own surface stays its first top",
        },
    }))
}

/// Per L1 cell, the highest L3 terrain its interpolation reaches, and the first horizon's top
/// raised to it. Returns (cells raised, largest raise in blocks).
fn raise_tops(hz: &mut Horizons, t: &Terrain, max_y: i32) -> Result<(u64, f64), CoreError> {
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
    let square = |path: &std::path::Path| -> Result<Vec<f64>, CoreError> {
        let r = TilePackReader::open(path)?;
        let d = r.desc().clone();
        let mut best = vec![f64::NEG_INFINITY; w1 * h1];
        // Each tile covers whole L1 squares (its edge, 256 cells, is a multiple of the ratio), so
        // tiles fill disjoint parts of `best`.
        let tiles: Vec<(usize, usize, usize, Vec<f64>)> = (0..d.tile_count())
            .into_par_iter()
            .map(
                |tix| -> Result<(usize, usize, usize, Vec<f64>), CoreError> {
                    let bytes = r.read_tile(tix)?;
                    let (x0, z0, tw, th) = d.tile_rect(tix);
                    let cb = d.cell_bytes();
                    let (i0, j0) = (x0 as usize / ratio, z0 as usize / ratio);
                    let lw = (tw as usize).div_ceil(ratio);
                    let lh = (th as usize).div_ceil(ratio);
                    let mut local = vec![f64::NEG_INFINITY; lw * lh];
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
                        }
                    }
                    Ok((i0, j0, lw, local))
                },
            )
            .collect::<Result<_, _>>()?;
        for (i0, j0, lw, local) in tiles {
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
        Ok(best)
    };
    let elev = square(&t.elevation)?;
    let amp = match &t.amplitude {
        Some(a) => square(a)?,
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
    Ok((tops.len() as u64, max_raise))
}

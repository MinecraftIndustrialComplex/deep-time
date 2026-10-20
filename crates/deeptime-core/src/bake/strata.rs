// SPDX-License-Identifier: LGPL-3.0-or-later
//! The strata part of the bake: `strata/horizons` (L1, K horizons per cell) and
//! `strata/classes.json`, from P1-07's horizon bake (`structure::bake_horizons`).
//!
//! **The rock rises with the terrain.** P1-07 puts each cell's first horizon top at the
//! deep-time surface (L1, the surface the history left). The terrain the game builds is the L3
//! `elevation` plus the detail term, which on mountains stands well above that surface (the
//! amplifier's relief, `surface::amplify`). That relief is rock of the region, not a pile of its
//! youngest bed, so two steps follow (ledger: STR-052, STR-053):
//!
//! 1. **The lift.** Every horizon of a cell rises by how far the mean L3 elevation round the
//!    cell's node stands above its first top, where it does (never down; capped so the first top
//!    stays at or under `max_y`). The contacts follow the ground; the loose cover and the
//!    regolith keep the thickness the history gave them (no young cover is invented: the till and
//!    the amplifier's deposits are not in the stacks). Where the terrain lies below the history's
//!    surface nothing moves: valleys cut into the stack and expose the rock beneath, as before.
//!    What the history put at a depth (bodies, sites, the structure file's faults and
//!    intrusions) rises by the same lift, read through the column rule's B-spline ([`Lift`]).
//! 2. **The crests.** The first top of every cell must still reach the highest terrain any
//!    block interpolating from it can have (the largest `elevation` plus the largest
//!    `detail/amplitude` over the L3 cells within two L1 cells) plus one block; the column rule
//!    reads tops through a B-spline of the 4 × 4 cells round a block with positive weights, so the
//!    interpolated top is then never below the terrain. The thickest rock unit beginning within
//!    16 blocks of rock under the surface takes up what is missing, the beds over it rising with
//!    the ground ([`raise_stack`], the smoothing's ridge rule).

use deeptime_format::TilePackReader;
use rayon::prelude::*;
use serde_json::{Value, json};

use super::Out;
use super::terrain::Terrain;
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::structure::bake_horizons::{self, Horizons, NO_CLASS, NO_TOP};
use crate::structure::realism;
use crate::world::WorldState;

/// Blocks added above the highest terrain a cell's top horizon must cover.
pub const TOP_MARGIN_BLOCKS: i32 = 1;

pub fn bake(
    world: &WorldState,
    p: &PlanetPreset,
    terrain: &Terrain,
    out: &mut Out<'_>,
) -> Result<(Value, Lift), CoreError> {
    let sp = p.structure.clone().unwrap_or_default();
    let started = std::time::Instant::now();
    // The structure stage's horizons when it kept them for these inputs (P1-19).
    let (mut hz, kept) = bake_horizons::bake_noting(world, p)?;
    let horizons_ms = started.elapsed().as_secs_f64() * 1000.0;
    hz.check()
        .map_err(|e| CoreError::InvalidState(format!("the horizon bake failed its check: {e}")))?;
    // What a geologist would object to in the stacks as written (before the top raise, which
    // only lifts first tops): `deeptime strata-check` measures the same on the baked columns.
    let tables = crate::structure::columns::Tables::get();
    let census = realism::census_of_cells(
        &hz,
        &realism::Rules::builtin(),
        &realism::Limits::default(),
        1,
    );
    let stack_rules = match hz.check_bake(&tables.min_blocks) {
        Ok(()) => "ok".to_owned(),
        Err(e) => e,
    };
    let Raised {
        raised,
        max_raise,
        limit,
        fill,
        lift,
        lift_note,
    } = raise_tops(
        &mut hz,
        terrain,
        p.scale.min_y,
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
    Ok((json!({
        "grid": hz.grid,
        "k": k,
        "mean_horizons": mean,
        "top_raised_cells": raised,
        "top_raise_blocks_max": max_raise,
        "lift": lift_note,
        "mountain_fill": fill,
        "build_limit": limit.note(p.scale.min_y, p.scale.max_y, p.scale.build_fit()),
        "realism": {
            "cells": census.columns,
            "grade_inversion": census.share(census.with_inversion),
            "metamorphic_sliver": census.share(census.with_sliver),
            "plutonic_sheet": census.share(census.with_sheet),
            "plutonic_with_a_floor": census.share(census.with_floored),
            "stack_rules": stack_rules,
            "rule": "share of the stored stacks with a horizon that was never as hot as one above it formed at, with a metamorphic horizon under 8 blocks inside the stack, with a plutonic horizon under 16 blocks between other rocks, with plutonic rock over rock of another family; stack_rules: every key span in one key domain and no metamorphic or plutonic sliver",
        },
        "top_rule": "after the lift, the first horizon's top is at least the highest elevation plus the highest detail amplitude over the L3 cells within two L1 cells, plus one block; the thickest rock unit beginning within 16 blocks of the rock surface takes up the difference and the horizons over it rise with the ground",
        "smoothing": {
            "sigma_cells": sp.strata_smoothing_cells,
            "sample_step_cells": bake_horizons::smooth_stride(sp.strata_smoothing_cells),
            "radius_samples": bake_horizons::SMOOTH_RADIUS_SAMPLES,
            "tongue_band_keys": bake_horizons::tongue_params(&sp).0,
            "tongue_minor_share": sp.strata_minor_share,
            "rule": "each cell's stack is the Gaussian-weighted mean of its neighbours' height functions, cut at every key any of them has, each interval the class of the largest weighted thickness; the cell's own surface stays its first top",
        },
        // How long the horizons took, and whether they were the structure stage's (a run's
        // timing, like the bake's own `millis`).
        "millis": { "horizons": horizons_ms, "horizons_kept": kept },
    }), lift))
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
    /// L3 cells whose elevation is below `min_y`: the game cuts such sea floor flat at the world's
    /// floor (2026-10-02: the floor is y −512 so the deepest trenches fit; this counts any that
    /// still do not, at extreme settings).
    pub cells_below: u64,
    /// L3 cells in the bottom [`FLOOR_SLIDE_BLOCKS`] above `min_y`, where the router's density
    /// slide flattens the terrain.
    pub cells_in_slide: u64,
    /// The lowest elevation, blocks (y); +∞ when no cell has a value.
    pub lowest_y: f64,
}

/// The height above the world's floor of the router's bottom density slide (`planet.json`).
pub const FLOOR_SLIDE_BLOCKS: f64 = 24.0;

impl BuildLimit {
    const NONE: BuildLimit = BuildLimit {
        cells_above: 0,
        cells_compressed: 0,
        land_cells: 0,
        highest_y: f64::NEG_INFINITY,
        cells_below: 0,
        cells_in_slide: 0,
        lowest_y: f64::INFINITY,
    };

    fn merge(self, o: BuildLimit) -> BuildLimit {
        BuildLimit {
            cells_above: self.cells_above + o.cells_above,
            cells_compressed: self.cells_compressed + o.cells_compressed,
            land_cells: self.land_cells + o.land_cells,
            highest_y: self.highest_y.max(o.highest_y),
            cells_below: self.cells_below + o.cells_below,
            cells_in_slide: self.cells_in_slide + o.cells_in_slide,
            lowest_y: self.lowest_y.min(o.lowest_y),
        }
    }

    pub fn note(&self, min_y: i32, max_y: i32, fit: crate::params::BuildFit) -> Value {
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
            "min_y": min_y,
            "cells_below_min_y": self.cells_below,
            "cells_in_floor_slide": self.cells_in_slide,
            "lowest_y": if self.lowest_y.is_finite() { json!(self.lowest_y) } else { Value::Null },
            "rule_floor": "elevation cells below min_y: the game cuts such sea floor flat at the world's floor; cells_in_floor_slide are within 24 blocks above min_y, where the density slide flattens the terrain",
        })
    }
}

/// Per L1 cell: the whole stack lifted where the final terrain stands above the surface the
/// history left ([`Lift`]), then the top units raised over the highest L3 terrain the cell's
/// interpolation reaches ([`raise_stack`]). Returns the counts, the elevation against the build
/// limit, the mountain-fill measure and the lift.
fn raise_tops(
    hz: &mut Horizons,
    t: &Terrain,
    min_y: i32,
    max_y: i32,
    sea_level_y: i32,
    knee_y: f64,
) -> Result<Raised, CoreError> {
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
    let k = hz.k as usize;
    // Highest elevation and highest amplitude of the L3 cells in each L1 cell's square
    // [x1, x1 + r1) × [z1, z1 + r1), in blocks.
    let (limit_y, sea_y) = (f64::from(max_y), f64::from(sea_level_y));
    let (floor_y, slide_y) = (f64::from(min_y), f64::from(min_y) + FLOOR_SLIDE_BLOCKS);
    type Tile = (usize, usize, usize, Vec<f64>, Vec<(f64, u32)>, BuildLimit);
    // Per L1 square: the highest value, the mean of its L3 cells, and the build-limit counts.
    type Squares = (Vec<f64>, Vec<f64>, BuildLimit);
    let square = |path: &std::path::Path| -> Result<Squares, CoreError> {
        let r = TilePackReader::open(path)?;
        let d = r.desc().clone();
        let mut best = vec![f64::NEG_INFINITY; w1 * h1];
        let mut sum = vec![(0.0f64, 0u32); w1 * h1];
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
                let mut local_sum = vec![(0.0f64, 0u32); lw * lh];
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
                            local_sum[c].0 += v;
                            local_sum[c].1 += 1;
                            limit.cells_above += u64::from(v > limit_y);
                            limit.cells_compressed += u64::from(v > knee_y);
                            limit.land_cells += u64::from(v >= sea_y);
                            limit.highest_y = limit.highest_y.max(v);
                            limit.cells_below += u64::from(v < floor_y);
                            limit.cells_in_slide += u64::from(v >= floor_y && v < slide_y);
                            limit.lowest_y = limit.lowest_y.min(v);
                        }
                    }
                }
                Ok((i0, j0, lw, local, local_sum, limit))
            })
            .collect::<Result<_, _>>()?;
        let mut limit = BuildLimit::NONE;
        for (i0, j0, lw, local, local_sum, l) in tiles {
            limit = limit.merge(l);
            for (q, (v, (sv, n))) in local.into_iter().zip(local_sum).enumerate() {
                let (i, j) = (i0 + q % lw, j0 + q / lw);
                if i < w1 && j < h1 {
                    let b = &mut best[j * w1 + i];
                    if v > *b {
                        *b = v;
                    }
                    let a = &mut sum[j * w1 + i];
                    a.0 += sv;
                    a.1 += n;
                }
            }
        }
        let mean = sum
            .into_iter()
            .map(|(v, n)| if n > 0 { v / f64::from(n) } else { f64::NAN })
            .collect();
        Ok((best, mean, limit))
    };
    let (elev, mean, limit) = square(&t.elevation)?;
    let amp = match &t.amplitude {
        Some(a) => square(a)?.0,
        None => vec![0.0; w1 * h1],
    };
    // The surface the history left (each cell's first top before any raise), 1/16 block.
    let coarse: Vec<i16> = (0..w1 * h1).map(|c| hz.tops[c * k]).collect();
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
    // The terrain at each cell's node (the column rule places a cell's stack at its corner
    // (x1, z1), `deeptime_format::strata::stack_at`): the mean of the four squares round it.
    let node_mean = |i: usize, j: usize| -> f64 {
        let (mut sum, mut n) = (0.0, 0u32);
        for dj in -1i64..=0 {
            let jj = (j as i64 + dj).clamp(0, h1 as i64 - 1) as usize;
            for di in -1i64..=0 {
                let ii = (i as i64 + di).rem_euclid(w1 as i64) as usize;
                let v = mean[jj * w1 + ii];
                if v.is_finite() {
                    sum += v;
                    n += 1;
                }
            }
        }
        if n > 0 { sum / f64::from(n) } else { f64::NAN }
    };
    let tables = crate::structure::columns::Tables::get();
    let max16 = max_y * 16;
    // Per cell: (the whole-stack lift, the crest raise of the top units), 1/16 block.
    let moved: Vec<(i32, i32)> = hz
        .tops
        .par_chunks_mut(k)
        .zip(hz.classes.par_chunks(k))
        .zip(hz.keys.par_chunks(k))
        .enumerate()
        .map(|(c, ((tops, classes), keys))| {
            let (i, j) = (c % w1, c / w1);
            let n = classes.iter().position(|&x| x == NO_CLASS).unwrap_or(k);
            if n == 0 || tops[0] == NO_TOP {
                return (0, 0);
            }
            let top0 = i32::from(tops[0]);
            // The whole stack rises where the terrain stands above the surface the history
            // left: the amplifier's relief is rock, so every contact follows it (below that
            // surface the valleys cut into the stack as before). ledger: STR-052
            let m = node_mean(i, j);
            let lift = if m.is_finite() {
                ((m * 16.0).round() as i32 - top0).clamp(0, (max16 - top0).max(0))
            } else {
                0
            };
            if lift > 0 {
                for t in &mut tops[..n] {
                    *t = (i32::from(*t) + lift).min(i32::from(i16::MAX)) as i16;
                }
            }
            // The crests within the window: the first top covers the highest terrain any block
            // interpolating from this cell can have.
            let e = window(&elev, i, j);
            let a = window(&amp, i, j).max(0.0);
            if !e.is_finite() {
                return (lift, 0);
            }
            let need = ((e + a).ceil() as i32 + TOP_MARGIN_BLOCKS).min(max_y) * 16;
            let raise = raise_stack(
                &mut tops[..n],
                &classes[..n],
                &keys[..n],
                need,
                min_y * 16,
                &tables.unconsolidated,
            );
            (lift, raise)
        })
        .collect();
    let (mut raised, mut max_raise) = (0u64, 0.0f64);
    let (mut lifted, mut lift_sum, mut max_lift) = (0u64, 0.0f64, 0.0f64);
    let mut lift = vec![0.0f32; w1 * h1];
    for (c, &(l, r)) in moved.iter().enumerate() {
        if r > 0 {
            raised += 1;
            max_raise = max_raise.max(f64::from(r) / 16.0);
        }
        if l > 0 {
            lifted += 1;
            lift_sum += f64::from(l) / 16.0;
            max_lift = max_lift.max(f64::from(l) / 16.0);
            lift[c] = l as f32 / 16.0;
        }
    }
    let fill = mountain_fill(hz, &coarse, &mean);
    let note = json!({
        "cells": lifted,
        "mean_blocks": if lifted > 0 { lift_sum / lifted as f64 } else { 0.0 },
        "max_blocks": max_lift,
        "rule": "every horizon of a cell rises by how far the mean L3 elevation round the cell's node stands above the surface the history left (never down; capped at max_y); bodies, sites and the folds' and faults' heights rise by the same lift, read through the column rule's B-spline",
    });
    Ok(Raised {
        raised,
        max_raise,
        limit,
        fill,
        lift: Lift {
            grid: g1,
            blocks: lift,
        },
        lift_note: note,
    })
}

/// What [`raise_tops`] did.
struct Raised {
    raised: u64,
    max_raise: f64,
    limit: BuildLimit,
    fill: Value,
    lift: Lift,
    lift_note: Value,
}

/// Raises one stack's first top to at least `need` (1/16 block) and returns the raise (0 when it
/// was high enough). The thickest rock unit (a run of horizons of one class in one key domain)
/// that begins within [`bake_horizons::RIDGE_SEARCH_BLOCKS`] of rock under the rock surface takes
/// up the difference, and the horizons over it (the regolith, loose cover and thinner beds) rise with the ground, keeping
/// their thickness: a crest is the region's top formation, not a pile of its youngest bed (the
/// smoothing's ridge rule, `Horizons::smoothed`). With no rock unit that near the surface the
/// first horizon takes it up, as before. `tops`, `classes` and `keys` are the stack's used
/// horizons, `floor` the last one's base (1/16 block); `unconsolidated` is per class
/// (`columns::Tables`).
pub fn raise_stack(
    tops: &mut [i16],
    classes: &[u8],
    keys: &[[u16; 2]],
    need: i32,
    floor: i32,
    unconsolidated: &[bool],
) -> i32 {
    use crate::structure::columns::key;
    let n = tops.len().min(classes.len()).min(keys.len());
    if n == 0 || tops[0] == NO_TOP || i32::from(tops[0]) >= need {
        return 0;
    }
    let raise = need - i32::from(tops[0]);
    let loose =
        |h: usize| key::domain(keys[h][0]) == 0 || unconsolidated.get(usize::from(classes[h])) == Some(&true);
    let base = |h: usize| if h + 1 < n { i32::from(tops[h + 1]) } else { floor };
    // The unit that fills the raise: the thickest rock unit (a run of horizons of one class in
    // one key domain) beginning within `RIDGE_SEARCH_BLOCKS` of rock under the rock surface, as
    // the smoothing's ridge rule picks it.
    let mut fill: Option<(usize, i32)> = None;
    let (mut depth, mut h) = (0i32, 0usize);
    while h < n && depth < 16 * bake_horizons::RIDGE_SEARCH_BLOCKS {
        if loose(h) {
            h += 1;
            continue;
        }
        let (class, dom) = (classes[h], key::domain(keys[h][0]));
        let mut e = h;
        while e + 1 < n && classes[e + 1] == class && key::domain(keys[e + 1][0]) == dom {
            e += 1;
        }
        let th = i32::from(tops[h]) - base(e);
        if fill.is_none_or(|f| th > f.1) {
            fill = Some((h, th));
        }
        depth += th;
        h = e + 1;
    }
    let upto = fill.map_or(0, |f| f.0);
    for t in &mut tops[..=upto] {
        *t = (i32::from(*t) + raise).min(i32::from(i16::MAX)) as i16;
    }
    raise
}

/// The whole-stack lift of the bake ([`raise_tops`]): per L1 cell of the horizon grid, how far
/// (blocks) every horizon rose. Whatever the history put at a depth under its surface (the
/// plutons and ore bodies, the deposit sites, the faults and intrusions of the structure file)
/// rises with it, by the lift the column rule reads at its place, so it stays in the rock it was
/// put in.
#[derive(Clone, Debug)]
pub struct Lift {
    pub grid: deeptime_format::GridDesc,
    pub blocks: Vec<f32>,
}

impl Lift {
    /// The lift at block (x, z), blocks: the column rule's cubic B-spline over the 4 × 4 cells
    /// round it, as `deeptime_format::strata::stack_at` reads the horizon tops (a uniform lift of
    /// every cell moves every interpolated top by this much).
    pub fn at(&self, x: f64, z: f64) -> f64 {
        let g = &self.grid;
        if self.blocks.is_empty() || g.w == 0 || g.r == 0 {
            return 0.0;
        }
        let c = f64::from(g.w) * f64::from(g.r);
        let wx = (x + c / 2.0).rem_euclid(c) - c / 2.0;
        let u = (wx - g.origin_block_x as f64) / f64::from(g.r);
        let v = (z - g.origin_block_z as f64) / f64::from(g.r);
        let (i0, j0) = (u.floor() as i64, v.floor() as i64);
        let (bu, bv) = (
            deeptime_format::strata::bspline(u - i0 as f64),
            deeptime_format::strata::bspline(v - j0 as f64),
        );
        let mut sum = 0.0;
        for (b, &wv) in bv.iter().enumerate() {
            for (a, &wu) in bu.iter().enumerate() {
                let (ci, cj) = g.cell(i0 - 1 + a as i64, j0 - 1 + b as i64);
                sum += wu * wv * f64::from(self.blocks[cj as usize * g.w as usize + ci as usize]);
            }
        }
        sum
    }

    /// `bodies` risen by the lift at each one's centre (its clip and box with it).
    pub fn bodies(&self, bodies: &[deeptime_format::Body]) -> Vec<deeptime_format::Body> {
        bodies
            .iter()
            .map(|b| {
                let mut nb = b.clone();
                let dy = self.at(b.center[0], b.center[2]);
                if dy > 0.0 {
                    nb.center[1] += dy;
                    if nb.clip_top_y != deeptime_format::bodies::NO_CLIP {
                        nb.clip_top_y += dy.round() as i32;
                    }
                    nb.aabb = nb.computed_aabb();
                }
                nb
            })
            .collect()
    }

    /// `sites` risen with their bodies (`bodies`, already risen): a site's box is its bodies'
    /// boxes, its centre rises by the lift there.
    pub fn sites(
        &self,
        sites: &[deeptime_format::Site],
        bodies: &[deeptime_format::Body],
    ) -> Vec<deeptime_format::Site> {
        sites
            .iter()
            .map(|s| {
                let mut ns = s.clone();
                let dy = self
                    .at(f64::from(s.center[0]), f64::from(s.center[2]))
                    .round() as i32;
                ns.center[1] += dy;
                let mine: Vec<_> = bodies.iter().filter(|b| b.site == Some(s.id)).collect();
                if mine.is_empty() {
                    ns.aabb[0][1] += dy;
                    ns.aabb[1][1] += dy;
                } else {
                    // The bodies keep their own x and z; only y changes.
                    ns.aabb[0][1] = mine.iter().map(|b| b.aabb[0][1]).min().unwrap_or(s.aabb[0][1]);
                    ns.aabb[1][1] = mine.iter().map(|b| b.aabb[1][1]).max().unwrap_or(s.aabb[1][1]);
                }
                ns
            })
            .collect()
    }

    /// The structure file's heights risen by the lift at each event's centre: a fault's plane
    /// and an intrusion's centre (a fold moves columns relative to where they are).
    pub fn structure(
        &self,
        st: &deeptime_format::structure::Structure,
    ) -> deeptime_format::structure::Structure {
        use deeptime_format::structure::Shape;
        let mut out = st.clone();
        for e in &mut out.events {
            let dy = self.at(e.window.centre[0], e.window.centre[1]);
            match &mut e.shape {
                Shape::Fault { y, .. } | Shape::Intrusion { y, .. } => *y += dy,
                Shape::Fold { .. } => {}
            }
        }
        out
    }
}

/// Relief (blocks) above the coarse surface a cell must have to count in [`mountain_fill`].
pub const FILL_RELIEF_BLOCKS: [f64; 2] = [8.0, 30.0]; // ledger: STR-051 (game-choice: review thresholds)

/// How much of the relief the final terrain has above the surface the history left (a cell's
/// first top before the raise, `coarse`, 1/16 block) the stack's top unit fills: per L1 cell
/// whose mean L3 elevation (`mean`, blocks) stands at least [`FILL_RELIEF_BLOCKS`] above that
/// surface, the share of the interval [coarse, mean] the first horizon spans. Returns, per
/// threshold, the cells, the share of them where the top unit fills more than half the relief,
/// and the mean share filled.
pub fn mountain_fill(hz: &Horizons, coarse: &[i16], mean: &[f64]) -> Value {
    let floor = f64::from(hz.min_y);
    let mut rows = Vec::new();
    for &min_relief in &FILL_RELIEF_BLOCKS {
        let (mut n, mut most, mut sum) = (0u64, 0u64, 0.0f64);
        for (c, (&c0, &m)) in coarse.iter().zip(mean).enumerate() {
            if c0 == NO_TOP || !m.is_finite() {
                continue;
            }
            let lo = f64::from(c0) / 16.0;
            let relief = m - lo;
            if relief < min_relief {
                continue;
            }
            let (tops, _, _) = hz.cell(c);
            if tops.is_empty() {
                continue;
            }
            let top = f64::from(tops[0]) / 16.0;
            let base = tops.get(1).map_or(floor, |&t| f64::from(t) / 16.0);
            let filled = (top.min(m) - base.max(lo)).max(0.0) / relief;
            n += 1;
            most += u64::from(filled > 0.5);
            sum += filled;
        }
        rows.push(json!({
            "min_relief_blocks": min_relief,
            "cells": n,
            "top_unit_fills_most": if n > 0 { most as f64 / n as f64 } else { 0.0 },
            "mean_share_filled": if n > 0 { sum / n as f64 } else { 0.0 },
        }));
    }
    json!({
        "by_relief": rows,
        "rule": "L1 cells whose mean L3 elevation stands at least min_relief_blocks above the surface the history left (the first top before the raise): the share of them where the first horizon spans more than half of that relief, and the mean share it spans",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::columns::Tables;
    use crate::surface::lithology::index;

    fn class(name: &str) -> u8 {
        index(name) as u8
    }

    #[test]
    fn a_crest_is_the_thickest_rock_unit_near_the_surface_not_its_top_bed() {
        let t = Tables::get();
        let floor = -512 * 16;
        // Soil (regolith keys), 4 blocks of sandstone (the log), 40 of granite, gabbro (pools).
        let classes = [
            class("soil"),
            class("sandstone"),
            class("granite"),
            class("gabbro"),
        ];
        let keys = [
            [0, 10],
            [5000, 5010],
            [32_768 + 64, 32_768 + 127],
            [33_100, 33_163],
        ];
        let mut tops = [1600i16, 1568, 1504, 864];
        let raise = raise_stack(&mut tops, &classes, &keys, 1920, floor, &t.unconsolidated);
        assert_eq!(raise, 320);
        // The granite thickens; the soil and the sandstone keep their thickness over it.
        assert_eq!(tops, [1920, 1888, 1824, 864]);

        // A thick top bed is itself the formation that fills the crest.
        let mut tops = [1600i16, 1568, 1088, 864];
        raise_stack(&mut tops, &classes, &keys, 1920, floor, &t.unconsolidated);
        assert_eq!(tops, [1920, 1888, 1088, 864]);

        // High enough already: nothing moves.
        let mut tops = [1600i16, 1568, 1504, 864];
        let raise = raise_stack(&mut tops, &classes, &keys, 1500, floor, &t.unconsolidated);
        assert_eq!(raise, 0);
        assert_eq!(tops, [1600, 1568, 1504, 864]);
    }

    #[test]
    fn a_uniform_lift_moves_every_point_by_itself() {
        let grid = deeptime_format::GridDesc::for_planet("L1", 1024, 16).unwrap();
        let n = grid.cells();
        let lift = Lift {
            grid,
            blocks: vec![12.5; n],
        };
        for (x, z) in [(0.0, 0.0), (-511.0, 3.5), (300.25, -100.0), (511.9, 150.0)] {
            let got = lift.at(x, z);
            assert!((got - 12.5).abs() < 1e-9, "({x}, {z}): {got}");
        }
        let none = Lift {
            grid: lift.grid.clone(),
            blocks: vec![0.0; n],
        };
        assert_eq!(none.at(17.0, -3.0), 0.0);
    }
}

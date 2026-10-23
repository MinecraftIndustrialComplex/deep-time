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
//! 1. **The lift.** Every horizon of a cell rises by the relief at its node: over the L3 cells of
//!    the four L1 squares round it, the mean of the elevation less the history's surface at the
//!    same point (the first tops as the column rule draws them, a B-spline), where that is
//!    positive (never down; capped so the first top stays at or under `max_y`). Measured point by
//!    point, a node in a valley of the history's surface between its ridges has no relief where
//!    the terrain only follows that surface. The contacts follow the ground; the loose cover and
//!    the regolith keep the thickness the history gave them (no young cover is invented: the till
//!    and the amplifier's deposits are not in the stacks). Where the terrain lies below the
//!    history's surface nothing moves: valleys cut into the stack and expose the rock beneath, as
//!    before. The lift is one value per node, so a valley narrower than the node window
//!    (amplifier relief, 2 × the L1 spacing) still cuts into its lifted stack
//!    ([`strata_census::surface`] counts how far). What the history put at a depth rises with the
//!    rock it was put in: bodies and sites by the column rule's height of their stratigraphic key
//!    before and after ([`key_shift`]), the structure file's faults and intrusions by the lift at
//!    their centre ([`Lift`]).
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
use super::strata_census::{self, Point, census_stride};
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
    crate::surface::amplify::amp_probe_seg(&format!(
        "strata_horizons cells={} k={}",
        hz.grid.cells(),
        hz.k
    ));
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
        mut lift,
        mut lift_note,
        node_relief,
        moved,
        points,
        stride,
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
    // The bodies and sites rise with the rock at their centre (STR-056).
    let (bodies_dy, sites_dy) = shifts(&hz, &moved, &world.bodies, &world.sites);
    lift_note["bodies"] = shift_note(
        &lift,
        &bodies_dy,
        world.bodies.iter().map(|b| (b.center[0], b.center[2])),
    );
    lift_note["sites"] = shift_note(
        &lift,
        &sites_dy,
        world
            .sites
            .iter()
            .map(|s| (f64::from(s.center[0]), f64::from(s.center[2]))),
    );
    lift.bodies_dy = bodies_dy;
    lift.sites_dy = sites_dy;
    // What the ground cuts and what surface/class_top names there (STR-057): class_top is the
    // young landscape's class, else the deep-time surface's, nearest on L2 (`super::surface`).
    let (_, l2) = super::grids(p)?;
    let ct_field = super::field(world, crate::surface::young::field::CLASS)
        .or_else(|| super::field(world, crate::surface::field::CLASS));
    let class_top = ct_field.map(|f| {
        let l2 = l2.clone();
        move |x: f64, z: f64| -> Option<u8> {
            let i = ((x - l2.origin_block_x as f64) / f64::from(l2.r)).round() as i64;
            let j = ((z - l2.origin_block_z as f64) / f64::from(l2.r)).round() as i64;
            let v = super::nearest(f, l2.block_x(i) as f64, l2.block_z(j) as f64);
            (!v.is_nan()).then(|| (v.round() as i64).clamp(0, 254) as u8)
        }
    });
    let surface_census = strata_census::surface(
        &hz,
        &node_relief,
        &points,
        stride,
        class_top
            .as_ref()
            .map(|f| f as &(dyn Fn(f64, f64) -> Option<u8> + Sync)),
        &tables.unconsolidated,
    );
    drop(points);
    crate::surface::amplify::amp_probe_seg("strata_raised");
    let layer = hz.to_layer(deeptime_format::Codec::Lz4Block)?;
    crate::surface::amplify::amp_probe_seg("strata_layer");
    let mean = hz.mean_horizons();
    let k = hz.k;
    out.layer_bytes(layer.desc, &layer.bytes)?;
    // The lift beside the stacks it raised (FORMAT.md, "The lift"): the structure file keeps the
    // history's heights, and its readers add the lift at each column they read (STR-055).
    let lift16: Vec<i16> = moved
        .iter()
        .map(|m| m.lift.clamp(0, i32::from(i16::MAX)) as i16)
        .collect();
    out.layer(
        super::desc(
            deeptime_format::structure::LIFT_LAYER,
            deeptime_format::DType::I16,
            &hz.grid,
            1.0 / 16.0,
            "blocks",
            None,
        ),
        &lift16,
    )?;
    out.json(
        deeptime_format::column::CLASSES_PATH,
        &Horizons::classes_json(),
    )?;
    Ok((
        json!({
            "grid": hz.grid,
            "k": k,
            "mean_horizons": mean,
            "top_raised_cells": raised,
            "top_raise_blocks_max": max_raise,
            "lift": lift_note,
            "mountain_fill": fill,
            "surface_census": surface_census,
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
        }),
        lift,
    ))
}

/// The bodies' or the sites' rise (`dy`, blocks, one per centre `at`): how many rose, their mean
/// and largest rise, and how many rose more than a block beyond the lift at their centre (the
/// crest raise's part, STR-056).
fn shift_note(lift: &Lift, dy: &[f64], at: impl Iterator<Item = (f64, f64)>) -> Value {
    let (mut n, mut sum, mut max, mut beyond) = (0u64, 0.0f64, 0.0f64, 0u64);
    for (&d, (x, z)) in dy.iter().zip(at) {
        if d == 0.0 {
            continue;
        }
        n += 1;
        sum += d;
        max = max.max(d);
        beyond += u64::from((d - lift.at(x, z)).abs() > 1.0);
    }
    json!({
        "count": dy.len(),
        "risen": n,
        "mean_blocks": if n > 0 { sum / n as f64 } else { 0.0 },
        "max_blocks": max,
        "off_the_lift_over_1_block": beyond,
    })
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

/// Per L1 square [x1, x1 + r1) × [z1, z1 + r1) of an L3 layer: the highest value, the sum and
/// count of (value minus the history's surface at the same point), the build-limit counts and the
/// census's land points.
struct Squares {
    best: Vec<f64>,
    relief: Vec<(f64, u32)>,
    limit: BuildLimit,
    points: Vec<Point>,
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
    let stride = census_stride(l3.cells());
    // The surface the history left (each cell's first top before any raise), 1/16 block, and in
    // blocks as the column rule draws it between the nodes (NaN where a cell has no stack).
    let coarse: Vec<i16> = (0..w1 * h1).map(|c| hz.tops[c * k]).collect();
    let coarse_blocks: Vec<f32> = coarse
        .iter()
        .map(|&t| {
            if t == NO_TOP {
                f32::NAN
            } else {
                f32::from(t) / 16.0
            }
        })
        .collect();
    let (limit_y, sea_y) = (f64::from(max_y), f64::from(sea_level_y));
    let (floor_y, slide_y) = (f64::from(min_y), f64::from(min_y) + FLOOR_SLIDE_BLOCKS);
    type Tile = (
        usize,
        usize,
        usize,
        Vec<f64>,
        Vec<(f64, u32)>,
        BuildLimit,
        Vec<Point>,
    );
    // `relief`: also sum each L3 cell's value less the history's surface at its position (the
    // column rule's B-spline of the first tops, as the stack is drawn there), count the cells
    // against the build limit and take the census's land points.
    let square = |path: &std::path::Path, relief: bool| -> Result<Squares, CoreError> {
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
                let mut points = Vec::new();
                for lj in 0..th as usize {
                    let gj = z0 as usize + lj;
                    let sj = gj / ratio - j0;
                    for li in 0..tw as usize {
                        let gi = x0 as usize + li;
                        let si = gi / ratio - i0;
                        let at = (lj * tw as usize + li) * cb;
                        let v = d.physical(d.dtype.decode_scalar(&bytes[at..at + cb]));
                        let c = sj * lw + si;
                        if !v.is_finite() {
                            continue;
                        }
                        if v > local[c] {
                            local[c] = v;
                        }
                        if !relief {
                            continue;
                        }
                        let (bx, bz) = (l3.block_x(gi as i64) as f64, l3.block_z(gj as i64) as f64);
                        let under = bspline_at(&g1, &coarse_blocks, bx, bz);
                        if under.is_finite() {
                            local_sum[c].0 += v - under;
                            local_sum[c].1 += 1;
                            if v >= sea_y && gi.is_multiple_of(stride) && gj.is_multiple_of(stride)
                            {
                                points.push(Point {
                                    x: bx,
                                    z: bz,
                                    elev: v,
                                    relief: v - under,
                                });
                            }
                        }
                        limit.cells_above += u64::from(v > limit_y);
                        limit.cells_compressed += u64::from(v > knee_y);
                        limit.land_cells += u64::from(v >= sea_y);
                        limit.highest_y = limit.highest_y.max(v);
                        limit.cells_below += u64::from(v < floor_y);
                        limit.cells_in_slide += u64::from(v >= floor_y && v < slide_y);
                        limit.lowest_y = limit.lowest_y.min(v);
                    }
                }
                Ok((i0, j0, lw, local, local_sum, limit, points))
            })
            .collect::<Result<_, _>>()?;
        let mut limit = BuildLimit::NONE;
        let mut points = Vec::new();
        for (i0, j0, lw, local, local_sum, l, p) in tiles {
            limit = limit.merge(l);
            points.extend(p);
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
        Ok(Squares {
            best,
            relief: sum,
            limit,
            points,
        })
    };
    let Squares {
        best: elev,
        relief,
        limit,
        points,
    } = square(&t.elevation, true)?;
    let amp = match &t.amplitude {
        Some(a) => square(a, false)?.best,
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
    // The relief at each cell's node (the column rule places a cell's stack at its corner
    // (x1, z1), `deeptime_format::strata::stack_at`): the mean, over the L3 cells of the four
    // squares round it, of the final terrain less the history's surface at the same point. Both
    // are read at the same points, so a node in a coarse valley between coarse ridges has no
    // relief where the terrain only follows that surface. ledger: STR-052
    let node_relief: Vec<f64> = (0..w1 * h1)
        .into_par_iter()
        .map(|c| {
            let (i, j) = (c % w1, c / w1);
            let (mut sum, mut n) = (0.0, 0u32);
            for dj in -1i64..=0 {
                let jj = (j as i64 + dj).clamp(0, h1 as i64 - 1) as usize;
                for di in -1i64..=0 {
                    let ii = (i as i64 + di).rem_euclid(w1 as i64) as usize;
                    let (s, m) = relief[jj * w1 + ii];
                    sum += s;
                    n += m;
                }
            }
            if n > 0 { sum / f64::from(n) } else { f64::NAN }
        })
        .collect();
    let tables = crate::structure::columns::Tables::get();
    let max16 = max_y * 16;
    // Per cell: the whole-stack lift and the crest raise of the top units.
    let moved: Vec<Moved> = hz
        .tops
        .par_chunks_mut(k)
        .zip(hz.classes.par_chunks(k))
        .zip(hz.keys.par_chunks(k))
        .enumerate()
        .map(|(c, ((tops, classes), keys))| {
            let (i, j) = (c % w1, c / w1);
            let n = classes.iter().position(|&x| x == NO_CLASS).unwrap_or(k);
            if n == 0 || tops[0] == NO_TOP {
                return Moved::default();
            }
            let top0 = i32::from(tops[0]);
            // The whole stack rises where the terrain stands above the surface the history
            // left: the amplifier's relief is rock, so every contact follows it (below that
            // surface the valleys cut into the stack as before). ledger: STR-052
            let m = node_relief[c];
            let lift = if m.is_finite() {
                ((m * 16.0).round() as i32).clamp(0, (max16 - top0).max(0))
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
                return Moved {
                    lift,
                    ..Moved::default()
                };
            }
            let need = ((e + a).ceil() as i32 + TOP_MARGIN_BLOCKS).min(max_y) * 16;
            let (raise, upto) = raise_stack_unit(
                &mut tops[..n],
                &classes[..n],
                &keys[..n],
                need,
                min_y * 16,
                &tables.unconsolidated,
            );
            Moved {
                lift,
                raise,
                upto: upto as u16,
            }
        })
        .collect();
    let (mut raised, mut max_raise) = (0u64, 0.0f64);
    let (mut lifted, mut lift_sum, mut max_lift) = (0u64, 0.0f64, 0.0f64);
    let mut lift = vec![0.0f32; w1 * h1];
    for (c, m) in moved.iter().enumerate() {
        if m.raise > 0 {
            raised += 1;
            max_raise = max_raise.max(f64::from(m.raise) / 16.0);
        }
        if m.lift > 0 {
            lifted += 1;
            lift_sum += f64::from(m.lift) / 16.0;
            max_lift = max_lift.max(f64::from(m.lift) / 16.0);
            lift[c] = m.lift as f32 / 16.0;
        }
    }
    let raise16: Vec<i32> = moved.iter().map(|m| m.raise).collect();
    let fill =
        strata_census::mountain_fill(hz, &coarse, &node_relief, &raise16, &tables.unconsolidated);
    let note = json!({
        "cells": lifted,
        "mean_blocks": if lifted > 0 { lift_sum / lifted as f64 } else { 0.0 },
        "max_blocks": max_lift,
        "rule": "every horizon of a cell rises by the mean, over the L3 cells of the four L1 squares round the cell's node, of the final elevation less the history's surface at the same point (the first tops through the column rule's B-spline), where that is positive (capped at max_y); bodies and sites rise as the rock at their centre does (the column rule's height of its stratigraphic key, before and after the lift and the crest raise); the structure file keeps the history's heights and its readers add the lift at each column they read (the strata/lift layer)",
    });
    Ok(Raised {
        raised,
        max_raise,
        limit,
        fill,
        lift: Lift {
            grid: g1,
            blocks: lift,
            bodies_dy: Vec::new(),
            sites_dy: Vec::new(),
        },
        lift_note: note,
        node_relief,
        moved,
        points,
        stride,
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
    /// Per L1 cell, the relief at its node (blocks; NaN where unknown) and what raised it.
    node_relief: Vec<f64>,
    moved: Vec<Moved>,
    /// The surface census's land points and their stride (L3 cells).
    points: Vec<Point>,
    stride: usize,
}

/// What the bake's raise did to one L1 cell's stack (1/16 block): the whole-stack lift, the
/// crest raise, and the last horizon the crest raise moved (horizons 0 ..= `upto` rose by it).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Moved {
    pub lift: i32,
    pub raise: i32,
    pub upto: u16,
}

impl Moved {
    /// How far horizon `h`'s top rose, 1/16 block.
    pub fn top(&self, h: usize) -> i32 {
        self.lift
            + if h <= usize::from(self.upto) {
                self.raise
            } else {
                0
            }
    }
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
    raise_stack_unit(tops, classes, keys, need, floor, unconsolidated).0
}

/// [`raise_stack`], also returning the horizon that took up the raise (horizons 0 ..= it rose;
/// 0 when nothing moved).
pub fn raise_stack_unit(
    tops: &mut [i16],
    classes: &[u8],
    keys: &[[u16; 2]],
    need: i32,
    floor: i32,
    unconsolidated: &[bool],
) -> (i32, usize) {
    use crate::structure::columns::key;
    let n = tops.len().min(classes.len()).min(keys.len());
    if n == 0 || tops[0] == NO_TOP || i32::from(tops[0]) >= need {
        return (0, 0);
    }
    let raise = need - i32::from(tops[0]);
    let loose = |h: usize| {
        key::domain(keys[h][0]) == 0 || unconsolidated.get(usize::from(classes[h])) == Some(&true)
    };
    let base = |h: usize| {
        if h + 1 < n {
            i32::from(tops[h + 1])
        } else {
            floor
        }
    };
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
    (raise, upto)
}

/// The whole-stack lift of the bake ([`raise_tops`]): per L1 cell of the horizon grid, how far
/// (blocks) every horizon rose. Whatever the history put at a depth under its surface rises with
/// the rock it was put in: the plutons and ore bodies and the deposit sites by how far the rock at
/// their centre rose (`bodies_dy`, `sites_dy`: [`key_shift`], the lift and the crest raise
/// together). The structure file keeps the history's heights: the bake writes the lift as the
/// `strata/lift` layer and the deformed column rule adds it at every column it reads
/// (`deeptime_format::structure::Structure::deformed_stack`).
#[derive(Clone, Debug)]
pub struct Lift {
    pub grid: deeptime_format::GridDesc,
    pub blocks: Vec<f32>,
    /// Per body of the world (in its order), how far the rock at its centre rose, blocks; empty:
    /// the lift at its centre ([`Lift::at`]).
    pub bodies_dy: Vec<f64>,
    /// Per site of the world, the same.
    pub sites_dy: Vec<f64>,
}

/// The column rule's cubic B-spline of a per-cell field `v` of grid `g` at block (x, z), as
/// `deeptime_format::strata::stack_at` reads the horizon tops (x wraps, z clamps); NaN where any
/// of the 16 cells is.
pub fn bspline_at(g: &deeptime_format::GridDesc, v: &[f32], x: f64, z: f64) -> f64 {
    if v.is_empty() || g.w == 0 || g.r == 0 {
        return 0.0;
    }
    let c = f64::from(g.w) * f64::from(g.r);
    let wx = (x + c / 2.0).rem_euclid(c) - c / 2.0;
    let u = (wx - g.origin_block_x as f64) / f64::from(g.r);
    let w = (z - g.origin_block_z as f64) / f64::from(g.r);
    let (i0, j0) = (u.floor() as i64, w.floor() as i64);
    let (bu, bv) = (
        deeptime_format::strata::bspline(u - i0 as f64),
        deeptime_format::strata::bspline(w - j0 as f64),
    );
    let mut sum = 0.0;
    for (b, &wv) in bv.iter().enumerate() {
        for (a, &wu) in bu.iter().enumerate() {
            let (ci, cj) = g.cell(i0 - 1 + a as i64, j0 - 1 + b as i64);
            sum += wu * wv * f64::from(v[cj as usize * g.w as usize + ci as usize]);
        }
    }
    sum
}

/// One cell's height (1/16 block) at every key of `ss` (ascending), its tops read by `top(h)`:
/// the column rule's reading of one stack (`Horizons::height_at`).
fn heights_sorted(
    top: impl Fn(usize) -> i32,
    keys: &[[u16; 2]],
    floor: f64,
    ss: &[u16],
    out: &mut [f64],
) {
    let n = keys.len();
    if n == 0 {
        out.fill(floor);
        return;
    }
    let mut h = 0;
    for (o, &s) in out.iter_mut().zip(ss) {
        if s <= keys[0][0] {
            *o = f64::from(top(0));
            continue;
        }
        while h + 1 < n && keys[h + 1][0] <= s {
            h += 1;
        }
        let t = f64::from(top(h));
        let b = if h + 1 < n {
            f64::from(top(h + 1))
        } else {
            floor
        };
        let [kt, kb] = keys[h];
        *o = if s >= kb {
            b
        } else {
            t - (t - b) * (f64::from(s - kt) / f64::from(kb - kt))
        };
    }
}

/// How far the bake's raise moved the rock at block (x, y, z), blocks (ledger: STR-056): the
/// column rule's height of the stratigraphic key the rock has there, after the raise less before.
/// The lift and the crest raise only move tops, so every cell keeps its keys, and the column's
/// height is linear in the key between the keys of the sixteen cells round it: the rock at y lies
/// at a fraction of one such interval before and at the same fraction of it after, so it keeps
/// its horizon. A point over the column's first top moves with that top; one under the last key
/// (the floor) by what that key moved. `hz` is the raised stacks, `moved` per cell what raised
/// them.
pub fn key_shift(hz: &Horizons, moved: &[Moved], x: f64, y: f64, z: f64) -> f64 {
    use deeptime_format::strata::{CELLS, bspline, spline_cell, spline_cell_at};
    let g = &hz.grid;
    if g.w == 0 || g.r == 0 || moved.len() != g.cells() {
        return 0.0;
    }
    // The column rule's sixteen cells and weights at the block column (`stack_at`; on a cube's
    // grid the block's face, across its seams).
    let (bx, bz) = (x.floor() as i64, z.floor() as i64);
    let Some((face, i0, j0, fu, fv)) = spline_cell(g, bx, bz) else {
        return 0.0;
    };
    let (bu, bv) = (bspline(fu), bspline(fv));
    let mut cells = [(0usize, 0.0f64); CELLS];
    for (b, &wv) in bv.iter().enumerate() {
        for (a, &wu) in bu.iter().enumerate() {
            let (ci, cj) = spline_cell_at(g, face, i0 - 1 + a as i64, j0 - 1 + b as i64);
            cells[b * 4 + a] = (cj as usize * g.w as usize + ci as usize, wu * wv);
        }
    }
    if cells
        .iter()
        .all(|&(c, _)| moved[c].lift == 0 && moved[c].raise == 0)
    {
        return 0.0;
    }
    let mut keys: Vec<u16> = cells
        .iter()
        .flat_map(|&(c, _)| hz.cell(c).2.iter().flatten().copied())
        .collect();
    keys.sort_unstable();
    keys.dedup();
    let m = keys.len();
    if m == 0 {
        return 0.0;
    }
    let floor = f64::from(hz.min_y) * 16.0;
    let (mut before, mut after) = (vec![0.0f64; m], vec![0.0f64; m]);
    let (mut hb, mut ha) = (vec![0.0f64; m], vec![0.0f64; m]);
    for &(c, w) in &cells {
        let (tops, _, ks) = hz.cell(c);
        let mv = moved[c];
        heights_sorted(
            |h| i32::from(tops[h]) - mv.top(h),
            ks,
            floor,
            &keys,
            &mut hb,
        );
        heights_sorted(|h| i32::from(tops[h]), ks, floor, &keys, &mut ha);
        for q in 0..m {
            before[q] += w * hb[q];
            after[q] += w * ha[q];
        }
    }
    let y16 = y * 16.0;
    if y16 >= before[0] {
        return (after[0] - before[0]) / 16.0;
    }
    for q in 0..m - 1 {
        if y16 >= before[q + 1] {
            let f = (before[q] - y16) / (before[q] - before[q + 1]);
            return (after[q] - f * (after[q] - after[q + 1]) - y16) / 16.0;
        }
    }
    (after[m - 1] - before[m - 1]) / 16.0
}

/// Per body and per site, how far the rock at its centre rose ([`key_shift`]).
pub fn shifts(
    hz: &Horizons,
    moved: &[Moved],
    bodies: &[deeptime_format::Body],
    sites: &[deeptime_format::Site],
) -> (Vec<f64>, Vec<f64>) {
    let b = bodies
        .par_iter()
        .map(|b| key_shift(hz, moved, b.center[0], b.center[1], b.center[2]))
        .collect();
    let s = sites
        .par_iter()
        .map(|s| {
            key_shift(
                hz,
                moved,
                f64::from(s.center[0]),
                f64::from(s.center[1]),
                f64::from(s.center[2]),
            )
        })
        .collect();
    (b, s)
}

impl Lift {
    /// The lift at block (x, z), blocks: the column rule's cubic B-spline over the 4 × 4 cells
    /// round it, as `deeptime_format::strata::stack_at` reads the horizon tops (a uniform lift of
    /// every cell moves every interpolated top by this much).
    pub fn at(&self, x: f64, z: f64) -> f64 {
        bspline_at(&self.grid, &self.blocks, x, z)
    }

    /// `bodies` risen with the rock at each one's centre (`bodies_dy`; the lift there when the
    /// bake gave none), their clips and boxes with them.
    pub fn bodies(&self, bodies: &[deeptime_format::Body]) -> Vec<deeptime_format::Body> {
        let given = self.bodies_dy.len() == bodies.len();
        bodies
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let mut nb = b.clone();
                let dy = if given {
                    self.bodies_dy[i]
                } else {
                    self.at(b.center[0], b.center[2])
                };
                if dy != 0.0 {
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

    /// `sites` risen with the rock at each one's centre (`sites_dy`; the lift there when the bake
    /// gave none), so a site stays in its host horizon; its box is its bodies' boxes (`bodies`,
    /// already risen), or its own risen with it.
    pub fn sites(
        &self,
        sites: &[deeptime_format::Site],
        bodies: &[deeptime_format::Body],
    ) -> Vec<deeptime_format::Site> {
        let given = self.sites_dy.len() == sites.len();
        sites
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let mut ns = s.clone();
                let dy = if given {
                    self.sites_dy[i]
                } else {
                    self.at(f64::from(s.center[0]), f64::from(s.center[2]))
                }
                .round() as i32;
                ns.center[1] += dy;
                let mine: Vec<_> = bodies.iter().filter(|b| b.site == Some(s.id)).collect();
                if mine.is_empty() {
                    ns.aabb[0][1] += dy;
                    ns.aabb[1][1] += dy;
                } else {
                    // The bodies keep their own x and z; only y changes.
                    ns.aabb[0][1] = mine
                        .iter()
                        .map(|b| b.aabb[0][1])
                        .min()
                        .unwrap_or(s.aabb[0][1]);
                    ns.aabb[1][1] = mine
                        .iter()
                        .map(|b| b.aabb[1][1])
                        .max()
                        .unwrap_or(s.aabb[1][1]);
                }
                ns
            })
            .collect()
    }
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
            bodies_dy: Vec::new(),
            sites_dy: Vec::new(),
        };
        for (x, z) in [(0.0, 0.0), (-511.0, 3.5), (300.25, -100.0), (511.9, 150.0)] {
            let got = lift.at(x, z);
            assert!((got - 12.5).abs() < 1e-9, "({x}, {z}): {got}");
        }
        let none = Lift {
            grid: lift.grid.clone(),
            blocks: vec![0.0; n],
            bodies_dy: Vec::new(),
            sites_dy: Vec::new(),
        };
        assert_eq!(none.at(17.0, -3.0), 0.0);
    }

    /// A small planet's stacks (soil, sandstone, granite and gabbro, the same keys everywhere,
    /// tops varying from cell to cell), raised as the bake raises them: a lift of 0 to 40 blocks
    /// on two cells in three, then a crest raise of up to 30 blocks on every other cell, which
    /// thickens the granite and carries the soil and sandstone up over it.
    fn raised_planet() -> (Horizons, Horizons, Vec<Moved>) {
        let t = Tables::get();
        let grid = deeptime_format::GridDesc::for_planet("L1", 1024, 16).unwrap();
        let n = grid.cells();
        let k = 4usize;
        let min_y = -64;
        let classes = [
            class("soil"),
            class("sandstone"),
            class("granite"),
            class("gabbro"),
        ];
        let keys = [
            [0u16, 10],
            [5000, 5010],
            [32_768 + 64, 32_768 + 127],
            [33_100, 33_163],
        ];
        let mut rng = crate::sphere::SplitMix64::new(7);
        let mut tops = vec![NO_TOP; n * k];
        for c in 0..n {
            let s = 64 * 16 + rng.below(32 * 16) as i32;
            let sand = 2 * 16 + rng.below(12 * 16) as i32;
            let granite = 10 * 16 + rng.below(40 * 16) as i32;
            let t0 = [s, s - 32, s - 32 - sand, s - 32 - sand - granite];
            for (h, &v) in t0.iter().enumerate() {
                tops[c * k + h] = v as i16;
            }
        }
        let before = Horizons {
            grid,
            k: k as u32,
            min_y,
            tops,
            classes: classes.repeat(n),
            keys: keys.repeat(n),
            tongues: (64, 0.15),
            min_blocks: Vec::new(),
        };
        let mut after = before.clone();
        let mut moved = vec![Moved::default(); n];
        for (c, m) in moved.iter_mut().enumerate() {
            let tops = &mut after.tops[c * k..(c + 1) * k];
            let lift = if rng.below(3) > 0 {
                rng.below(40 * 16) as i32
            } else {
                0
            };
            for t in tops.iter_mut() {
                *t = (i32::from(*t) + lift) as i16;
            }
            let need = i32::from(tops[0])
                + if rng.below(2) == 0 {
                    rng.below(30 * 16) as i32
                } else {
                    0
                };
            let (raise, upto) =
                raise_stack_unit(tops, &classes, &keys, need, min_y * 16, &t.unconsolidated);
            *m = Moved {
                lift,
                raise,
                upto: upto as u16,
            };
        }
        (before, after, moved)
    }

    /// The column rule's undeformed column of `hz` at block (x, z).
    fn column(hz: &Horizons, x: i64, z: i64) -> Vec<deeptime_format::strata::ColumnHorizon> {
        let desc = hz.desc(deeptime_format::Codec::Raw);
        let (k, w) = (hz.k as usize, hz.grid.w as usize);
        let mut out = Vec::new();
        deeptime_format::strata::stack_at(
            &desc,
            hz.min_y,
            |i, j, h| {
                let n = (j as usize * w + i as usize) * k + h as usize;
                deeptime_format::strata::StoredHorizon {
                    top: hz.tops[n],
                    class: hz.classes[n],
                    key: hz.keys[n],
                }
            },
            x,
            z,
            &mut out,
        );
        out
    }

    /// The class of a drawn column at block y (`None` over its first top).
    fn class_at(col: &[deeptime_format::strata::ColumnHorizon], y: i32) -> Option<u8> {
        if col.is_empty() || y > col[0].0 {
            return None;
        }
        let mut h = 0;
        while h + 1 < col.len() && col[h + 1].0 >= y {
            h += 1;
        }
        Some(col[h].1)
    }

    fn site(id: u32, center: [i32; 3]) -> deeptime_format::Site {
        deeptime_format::Site {
            id,
            kind: deeptime_format::SiteKind::Deposit,
            deposit_type: "deeptime:test_deposit".into(),
            center,
            lat_lon: [0.0, 0.0],
            aabb: [center, center],
            body: None,
            products: Vec::new(),
            forming_event: deeptime_format::EventId(1),
            parent: None,
            extraction: deeptime_format::Extraction::default(),
        }
    }

    #[test]
    fn a_site_keeps_its_host_stratum_across_the_lift() {
        let (before, after, moved) = raised_planet();
        let mut rng = crate::sphere::SplitMix64::new(11);
        let (mut sites, mut host) = (Vec::new(), Vec::new());
        while sites.len() < 300 {
            let (x, z) = (rng.below(1024) as i64 - 512, rng.below(400) as i64 - 200);
            let col = column(&before, x, z);
            let h = rng.below(col.len() as u64) as usize;
            let bottom = col.get(h + 1).map_or(before.min_y, |c| c.0 + 1);
            // Mid-horizon, clear of the contacts by more than the rounding of a site's centre.
            if col[h].0 - bottom < 6 {
                continue;
            }
            let y = (col[h].0 + bottom) / 2;
            sites.push(site(sites.len() as u32, [x as i32, y, z as i32]));
            host.push(col[h].1);
        }
        let (_, sites_dy) = shifts(&after, &moved, &[], &sites);
        let blocks: Vec<f32> = moved.iter().map(|m| m.lift as f32 / 16.0).collect();
        let lift = Lift {
            grid: after.grid.clone(),
            blocks,
            bodies_dy: Vec::new(),
            sites_dy,
        };
        let risen = lift.sites(&sites, &[]);
        let (mut rose, mut lift_alone_leaves) = (0, 0);
        for ((s, r), &class) in sites.iter().zip(&risen).zip(&host) {
            let [x, y, z] = r.center;
            let col = column(&after, i64::from(x), i64::from(z));
            assert_eq!(
                class_at(&col, y),
                Some(class),
                "site {} at {:?} rose to y {y}",
                s.id,
                s.center
            );
            assert_eq!(r.aabb[0][1] - s.aabb[0][1], y - s.center[1]);
            rose += usize::from(y > s.center[1]);
            // The lift at the centre alone (the rule before) misses the crest raise.
            let by_lift = s.center[1] + lift.at(f64::from(x), f64::from(z)).round() as i32;
            lift_alone_leaves += usize::from(class_at(&col, by_lift) != Some(class));
        }
        assert!(rose > sites.len() / 2, "only {rose} sites rose");
        assert!(
            lift_alone_leaves > 0,
            "the test needs sites the crest raise moves off the lift"
        );
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The strata bake's review measures: how the horizon stacks meet the final terrain after the
//! lift and the crest raise ([`super::strata`]). They only measure; nothing here changes the
//! world data.
//!
//! - [`mountain_fill`] (`notes.bake.strata.mountain_fill`, ledger STR-051): what fills the relief
//!   the terrain has over the surface the history left, at the lift's own window.
//! - [`surface`] (`notes.bake.strata.surface_census`, STR-057): on valley floors (ground well
//!   under the lifted surface), which horizon the ground cuts; on land, whether
//!   `surface/class_top` names the rock the column rule draws at the ground.
//! - [`structure_offsets`] (`notes.bake.strata.structure_lift`, STR-055): how far the lift at a
//!   fault's or an intrusion's centre is from the lift at the columns it cuts (the error of
//!   moving each event by its centre's lift, which the per-column lift of the readers replaced).

use deeptime_format::strata::{ColumnHorizon, StackScratch, StoredHorizon, stack_at_with};
use rayon::prelude::*;
use serde_json::{Value, json};

use super::strata::{Lift, bspline_at};
use crate::structure::bake_horizons::{Horizons, NO_TOP};
use crate::structure::columns::key;
use crate::synth::EVERY_CLASS;

/// Every how many L3 cells (along x and z) the surface census takes a point, at least.
pub const CENSUS_STRIDE: usize = 4; // ledger: STR-057 (game-choice: a review sample)
/// About how many L3 cells the surface census reads over the whole grid: on a larger grid it
/// takes every n-th cell, n above [`CENSUS_STRIDE`] (bounds its memory and time).
pub const CENSUS_CELLS: usize = 1 << 21; // ledger: STR-057 (game-choice: a review sample)

/// The surface census's stride over an L3 grid of `cells` cells.
pub fn census_stride(cells: usize) -> usize {
    ((cells as f64 / CENSUS_CELLS as f64).sqrt().ceil() as usize).max(CENSUS_STRIDE)
}

/// One L3 land point the surface census reads: block (x, z), the elevation and its relief over
/// the history's surface there (the column rule's B-spline of the first tops before the raise),
/// blocks.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f64,
    pub z: f64,
    pub elev: f64,
    pub relief: f64,
}

/// Relief (blocks) over the history's surface a node must have to count in [`mountain_fill`].
pub const FILL_RELIEF_BLOCKS: [f64; 2] = [8.0, 30.0]; // ledger: STR-051 (game-choice: review thresholds)

/// Whether horizon `h` of a stored stack is loose cover or regolith (a key in the regolith's
/// domain, or a class the tables call unconsolidated).
fn loose_stored(classes: &[u8], keys: &[[u16; 2]], h: usize, unconsolidated: &[bool]) -> bool {
    key::domain(keys[h][0]) == 0 || unconsolidated.get(usize::from(classes[h])) == Some(&true)
}

/// What fills the relief the final terrain has over the surface the history left, at the lift's
/// own window (STR-051). Per L1 cell whose node relief (`node_relief`, blocks: over the L3 cells
/// of the four squares round its node, the mean of the elevation less the history's surface at
/// the same point) is at least [`FILL_RELIEF_BLOCKS`]: the interval [top0, top0 + relief] over
/// the cell's first top before the raise (`coarse`, 1/16 block) and the share of it each horizon
/// of the raised stack `hz` spans. Returns, per threshold, the cells and: the share of them where
/// one horizon spans over half (`one_unit_fills_most`), where the first horizon does, where the
/// first rock horizon (the first neither regolith nor loose cover) does and its mean share; the
/// mean share of regolith and loose cover; the mean share the crest raise stretched (`raise`, per
/// cell, 1/16 block, over the relief, at most 1).
pub fn mountain_fill(
    hz: &Horizons,
    coarse: &[i16],
    node_relief: &[f64],
    raise: &[i32],
    unconsolidated: &[bool],
) -> Value {
    let floor = f64::from(hz.min_y);
    let mut rows = Vec::new();
    for &min_relief in &FILL_RELIEF_BLOCKS {
        let (mut n, mut one, mut first, mut rock) = (0u64, 0u64, 0u64, 0u64);
        let (mut rock_sum, mut loose_sum, mut stretch_sum) = (0.0f64, 0.0f64, 0.0f64);
        for (c, (&c0, &rn)) in coarse.iter().zip(node_relief).enumerate() {
            if c0 == NO_TOP || !rn.is_finite() || rn < min_relief {
                continue;
            }
            let (tops, classes, keys) = hz.cell(c);
            if tops.is_empty() {
                continue;
            }
            let lo = f64::from(c0) / 16.0;
            let hi = lo + rn;
            let share = |h: usize| {
                let top = f64::from(tops[h]) / 16.0;
                let base = tops.get(h + 1).map_or(floor, |&t| f64::from(t) / 16.0);
                (top.min(hi) - base.max(lo)).max(0.0) / rn
            };
            let (mut best, mut loose, mut first_rock) = (0.0f64, 0.0f64, None);
            for h in 0..tops.len() {
                let s = share(h);
                best = best.max(s);
                if loose_stored(classes, keys, h, unconsolidated) {
                    loose += s;
                } else if first_rock.is_none() {
                    first_rock = Some(s);
                }
            }
            let fr = first_rock.unwrap_or(0.0);
            n += 1;
            one += u64::from(best > 0.5);
            first += u64::from(share(0) > 0.5);
            rock += u64::from(fr > 0.5);
            rock_sum += fr;
            loose_sum += loose;
            stretch_sum += (f64::from(raise.get(c).copied().unwrap_or(0)) / 16.0 / rn).min(1.0);
        }
        let mean = |v: f64| if n > 0 { v / n as f64 } else { 0.0 };
        let share = |v: u64| if n > 0 { v as f64 / n as f64 } else { 0.0 };
        rows.push(json!({
            "min_relief_blocks": min_relief,
            "cells": n,
            "one_unit_fills_most": share(one),
            "first_horizon_fills_most": share(first),
            "first_rock_fills_most": share(rock),
            "first_rock_mean_share": mean(rock_sum),
            "loose_mean_share": mean(loose_sum),
            "stretched_mean_share": mean(stretch_sum),
        }));
    }
    json!({
        "by_relief": rows,
        "rule": "L1 cells whose node relief (over the L3 cells of the four squares round the node, the mean of the elevation less the history's surface at the same point, the lift's own measure) is at least min_relief_blocks: the interval from the cell's first top before the raise up by that relief, and the share of it each horizon of the raised stack spans; one_unit: any one horizon over half; first_rock: the first horizon that is neither regolith nor loose cover; stretched: the crest raise over the relief, at most 1",
    })
}

/// The surface census's thresholds (blocks): a land point is on a valley floor where the smoothed
/// node relief (the lift as the column rule reads it) is at least `[0]` and the ground stands at
/// least `[1]` under it.
pub const VALLEY_BLOCKS: [f64; 2] = [4.0, 4.0]; // ledger: STR-057 (game-choice: review thresholds)
/// A land point counts as lifted where the smoothed node relief is at least this (blocks).
pub const LIFTED_BLOCKS: f64 = 1.0; // ledger: STR-057 (game-choice: review threshold)
/// How deep under the ground the game looks for bedrock under loose cover
/// (`GeologyGround.COVER_BLOCKS`).
pub const COVER_BLOCKS: i32 = 8; // ledger: BIO-153 (game-choice)

/// What the census read at one point's ground block: over the column's first top, or the class
/// there, whether it is the first horizon and loose, the depth under the first top (blocks), and
/// against class_top (when it has one): class_top is the ground's class, class_top is
/// consolidated, it is the drawn bedrock, the game's bedrock is the drawn bedrock.
#[derive(Default)]
struct Ground {
    above_top: bool,
    ground: Option<u8>,
    in_first: bool,
    loose: bool,
    depth: f64,
    class_top: Option<[bool; 4]>,
}

/// The census's counts over a set of points.
#[derive(Clone)]
struct Tally {
    n: u64,
    in_first: u64,
    loose: u64,
    above_top: u64,
    depth_sum: f64,
    with_class_top: u64,
    class_top_is_ground: u64,
    class_top_rock: u64,
    class_top_rock_is_drawn: u64,
    bedrock_agrees: u64,
    classes: Vec<u64>,
}

impl Tally {
    fn new() -> Tally {
        Tally {
            n: 0,
            in_first: 0,
            loose: 0,
            above_top: 0,
            depth_sum: 0.0,
            with_class_top: 0,
            class_top_is_ground: 0,
            class_top_rock: 0,
            class_top_rock_is_drawn: 0,
            bedrock_agrees: 0,
            classes: vec![0; 256],
        }
    }

    fn add(&mut self, g: &Ground) {
        self.n += 1;
        self.above_top += u64::from(g.above_top);
        let Some(class) = g.ground else {
            return;
        };
        self.in_first += u64::from(g.in_first);
        self.loose += u64::from(g.loose);
        self.depth_sum += g.depth;
        self.classes[usize::from(class)] += 1;
        if let Some([is_ground, rock, drawn, agrees]) = g.class_top {
            self.with_class_top += 1;
            self.class_top_is_ground += u64::from(is_ground);
            self.class_top_rock += u64::from(rock);
            self.class_top_rock_is_drawn += u64::from(drawn);
            self.bedrock_agrees += u64::from(agrees);
        }
    }

    fn merge(mut self, o: Tally) -> Tally {
        self.n += o.n;
        self.in_first += o.in_first;
        self.loose += o.loose;
        self.above_top += o.above_top;
        self.depth_sum += o.depth_sum;
        self.with_class_top += o.with_class_top;
        self.class_top_is_ground += o.class_top_is_ground;
        self.class_top_rock += o.class_top_rock;
        self.class_top_rock_is_drawn += o.class_top_rock_is_drawn;
        self.bedrock_agrees += o.bedrock_agrees;
        for (a, b) in self.classes.iter_mut().zip(o.classes) {
            *a += b;
        }
        self
    }

    fn note(&self) -> Value {
        let share = |v: u64, of: u64| if of > 0 { v as f64 / of as f64 } else { 0.0 };
        let mut top: Vec<(usize, u64)> = self
            .classes
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, v)| v > 0)
            .collect();
        top.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let classes: Vec<Value> = top
            .iter()
            .take(6)
            .map(|&(c, v)| {
                json!({
                    "class": EVERY_CLASS.get(c).copied().unwrap_or("?"),
                    "share": share(v, self.n),
                })
            })
            .collect();
        json!({
            "points": self.n,
            "ground_in_first_horizon": share(self.in_first, self.n),
            "ground_in_loose_cover": share(self.loose, self.n),
            "ground_above_first_top": share(self.above_top, self.n),
            "mean_depth_under_first_top_blocks": if self.n > self.above_top { self.depth_sum / (self.n - self.above_top) as f64 } else { 0.0 },
            "with_class_top": self.with_class_top,
            "class_top_is_ground_class": share(self.class_top_is_ground, self.with_class_top),
            "class_top_rock": self.class_top_rock,
            "class_top_rock_is_drawn_rock": share(self.class_top_rock_is_drawn, self.class_top_rock),
            "bedrock_agrees": share(self.bedrock_agrees, self.with_class_top),
            "ground_classes": classes,
        })
    }
}

/// The horizon of a drawn column holding block y (`None` over its first top).
fn horizon_at(col: &[ColumnHorizon], y: i32) -> Option<usize> {
    if col.is_empty() || y > col[0].0 {
        return None;
    }
    let mut h = 0;
    while h + 1 < col.len() && col[h + 1].0 >= y {
        h += 1;
    }
    Some(h)
}

/// The first consolidated class of a drawn column from horizon `h` down, within
/// [`COVER_BLOCKS`] under block y (`GeologyGround.rockAt`'s search).
fn bedrock(col: &[ColumnHorizon], h: usize, y: i32, unconsolidated: &[bool]) -> Option<u8> {
    for hz in &col[h..] {
        if hz.0.min(y) < y - COVER_BLOCKS {
            break;
        }
        if unconsolidated.get(usize::from(hz.1)) != Some(&true) {
            return Some(hz.1);
        }
    }
    None
}

/// The surface census (STR-057) over the land points `points` of the raised stacks `hz`. The
/// smoothed node relief S (the column rule's B-spline of max(0, `node_relief`), the lift where it
/// is not capped) sorts them: on a valley floor (S at least [`VALLEY_BLOCKS`]`[0]`, the ground at
/// least `[1]` under it), lifted (S at least [`LIFTED_BLOCKS`]) or not. For each set, at the
/// ground block (the elevation, floored) of the column rule's undeformed column: the share in its
/// first horizon, in loose cover or regolith, over its first top, the mean depth under the first
/// top, the commonest classes; and against `class_top` (block (x, z) to `surface/class_top`, as
/// the game reads it): the share where it is the ground's class, where a consolidated class_top
/// is the column's bedrock (the ground's class, or under loose cover the first consolidated class
/// within [`COVER_BLOCKS`]), and where the game's bedrock (`GeologyGround.rockAt`: class_top, or
/// under a loose class_top that search) is the column's.
pub fn surface(
    hz: &Horizons,
    node_relief: &[f64],
    points: &[Point],
    stride: usize,
    class_top: Option<&(dyn Fn(f64, f64) -> Option<u8> + Sync)>,
    unconsolidated: &[bool],
) -> Value {
    let lifted: Vec<f32> = node_relief
        .iter()
        .map(|&r| {
            if r.is_finite() {
                r.max(0.0) as f32
            } else {
                0.0
            }
        })
        .collect();
    let desc = hz.desc(deeptime_format::Codec::Raw);
    let (k, w) = (hz.k as usize, hz.grid.w as usize);
    let horizon = |i: u32, j: u32, h: u32| {
        let n = (j as usize * w + i as usize) * k + h as usize;
        StoredHorizon {
            top: hz.tops[n],
            class: hz.classes[n],
            key: hz.keys[n],
        }
    };
    let loose = |c: u8, unit: u16| {
        key::domain(unit) == 0 || unconsolidated.get(usize::from(c)) == Some(&true)
    };
    let [valley, up, flat] = points
        .par_iter()
        .fold(
            || {
                (
                    StackScratch::default(),
                    Vec::new(),
                    [Tally::new(), Tally::new(), Tally::new()],
                )
            },
            |(mut scratch, mut col, mut sets), p| {
                let s = bspline_at(&hz.grid, &lifted, p.x, p.z);
                stack_at_with(
                    &mut scratch,
                    &desc,
                    hz.min_y,
                    horizon,
                    p.x.floor() as i64,
                    p.z.floor() as i64,
                    &mut col,
                );
                let y = p.elev.floor() as i32;
                let mut t = Ground::default();
                match horizon_at(&col, y) {
                    None => t.above_top = true,
                    Some(h) => {
                        let (_, class, unit) = col[h];
                        t.ground = Some(class);
                        t.in_first = h == 0;
                        t.loose = loose(class, unit);
                        t.depth = f64::from(col[0].0 - y);
                        if let Some(ct) = class_top.and_then(|f| f(p.x, p.z)) {
                            let rock = bedrock(&col, h, y, unconsolidated);
                            let ct_loose = unconsolidated.get(usize::from(ct)) == Some(&true);
                            let game = if ct_loose {
                                rock.or(Some(ct))
                            } else {
                                Some(ct)
                            };
                            t.class_top = Some([
                                ct == class,
                                !ct_loose,
                                !ct_loose && rock == Some(ct),
                                game == rock.or(Some(class)),
                            ]);
                        }
                    }
                }
                let on_floor = s >= VALLEY_BLOCKS[0] && p.relief <= s - VALLEY_BLOCKS[1];
                let is_up = s >= LIFTED_BLOCKS;
                if on_floor {
                    sets[0].add(&t);
                }
                sets[if is_up { 1 } else { 2 }].add(&t);
                (scratch, col, sets)
            },
        )
        .map(|(_, _, sets)| sets)
        .reduce(
            || [Tally::new(), Tally::new(), Tally::new()],
            |[a0, a1, a2], [b0, b1, b2]| [a0.merge(b0), a1.merge(b1), a2.merge(b2)],
        );

    json!({
        "stride_cells": stride,
        "land_points": points.len(),
        "valley_floor": valley.note(),
        "lifted": up.note(),
        "not_lifted": flat.note(),
        "rule": "land L3 points every stride_cells cells; S the column rule's B-spline of max(0, node relief); valley_floor: S at least 4 blocks and the elevation at least 4 under the history's surface plus S; lifted: S at least 1 block; at the ground block of the column rule's undeformed column: the horizon there, and surface/class_top (the sim's surface class, L2 nearest) against the drawn class and the drawn bedrock (the first consolidated class within 8 blocks, GeologyGround.rockAt's search)",
    })
}

/// How far the lift at a fault's or an intrusion's centre is from the lift at the columns the
/// event cuts (STR-055): what moving each event by its centre's lift would get wrong, which the
/// deformed column rule's per-column lift (`strata/lift`) does not do. Over a 9 × 9
/// lattice of each event's window (an intrusion's: inside its ellipse), |lift(centre) −
/// lift(point)|, blocks, as quantiles, for every point and for the points where either lift is
/// above zero.
pub fn structure_offsets(lift: &Lift, st: &deeptime_format::structure::Structure) -> Value {
    use deeptime_format::structure::Shape;
    const N: usize = 9;
    let (mut all, mut up) = ([Vec::new(), Vec::new()], [Vec::new(), Vec::new()]);
    for e in &st.events {
        let kind = match e.shape {
            Shape::Fault { .. } => 0,
            Shape::Intrusion { .. } => 1,
            Shape::Fold { .. } => continue,
        };
        let w = &e.window;
        let at_c = lift.at(w.centre[0], w.centre[1]);
        for i in 0..N {
            let ua = (i as f64 + 0.5) / N as f64 * 2.0 - 1.0;
            for j in 0..N {
                let ub = (j as f64 + 0.5) / N as f64 * 2.0 - 1.0;
                if kind == 1 && ua * ua + ub * ub >= 1.0 {
                    continue;
                }
                let (a, b) = (ua * w.half_along, ub * w.half_across);
                let x = w.centre[0] + a * w.along[0] + b * w.across[0];
                let z = w.centre[1] + a * w.along[1] + b * w.across[1];
                let at_p = lift.at(x, z);
                let d = (at_c - at_p).abs();
                all[kind].push(d);
                if at_c > 0.0 || at_p > 0.0 {
                    up[kind].push(d);
                }
            }
        }
    }
    let stats = |mut v: Vec<f64>| {
        v.sort_unstable_by(f64::total_cmp);
        let n = v.len();
        let q = |p: f64| {
            if n == 0 {
                0.0
            } else {
                v[((n - 1) as f64 * p).round() as usize]
            }
        };
        json!({
            "points": n,
            "p50": q(0.5),
            "p90": q(0.9),
            "p99": q(0.99),
            "max": q(1.0),
            "share_over_2_blocks": if n > 0 { v.iter().filter(|&&d| d > 2.0).count() as f64 / n as f64 } else { 0.0 },
        })
    };
    let [fa, ia] = all;
    let [fu, iu] = up;
    let (every, every_up) = ([&fa[..], &ia[..]].concat(), [&fu[..], &iu[..]].concat());
    json!({
        "faults": { "all": stats(fa), "lifted": stats(fu) },
        "intrusions": { "all": stats(ia), "lifted": stats(iu) },
        "every": { "all": stats(every), "lifted": stats(every_up) },
        "rule": "per fault and intrusion, a 9 x 9 lattice over its window (an intrusion's: inside its ellipse): |lift at the centre - lift at the point|, blocks; lifted: points where either is above zero",
    })
}

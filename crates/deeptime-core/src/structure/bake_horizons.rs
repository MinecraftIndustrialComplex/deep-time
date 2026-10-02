// SPDX-License-Identifier: LGPL-3.0-or-later
//! The horizon bake (plan §5.7.4, §8.8): each L1 cell's column (`columns.rs`) as a horizon stack
//! in block space — the `strata/horizons` layer the Java strata pass reads (`StrataPass`), K
//! horizons per cell, each an i16 top (1/16 block), a u8 class index into `strata/classes.json`
//! (`synth::EVERY_CLASS`) and its stratigraphic key span (`columns::key`), top-down; horizon h
//! fills from its top down to the next horizon's top, the last down to `min_y`. Unused horizons
//! have top `i16::MIN`, class 255 and keys `u16::MAX` (`deeptime_format::raster`). The keys let the
//! column evaluator correlate neighbouring stacks (format 0.2): every merge below keeps the
//! union of its members' spans, and the spans of a stack never overlap and grow downwards.
//!
//! Metres become blocks as the plan's bake does: an elevation of h metres above the sea of the
//! time is y = sea_level_y + h / m_per_block_v, and a unit of t metres spans t / m_per_block_v
//! blocks. The first horizon's top is the cell's surface (clamped to [min_y + 1, max_y]); units
//! wholly below `min_y` are dropped. Then **every horizon is at least one block thick after
//! scaling** (the last one down to `min_y` too) and **at most K are kept**: the thinnest horizon
//! under one block merges with its thinner neighbour, then, while more than K remain, the adjacent
//! pair with the smallest combined thickness merges (the strata log's cap rule); a merged horizon
//! takes the thicker member's class, and neighbours of one class and one key domain merge.
//!
//! **No slivers of metamorphic or plutonic rock.** Inside a stack (under its first horizon and
//! over its last, which the ground and the world's floor may cut anywhere) a horizon of
//! regional-metamorphic rock is at least `columns::METAMORPHIC_MIN_BLOCKS` thick and one of
//! plutonic rock at least `columns::PLUTONIC_MIN_BLOCKS`: a thinner one gives its blocks to the
//! horizon under it (the one over it where only that is of its key domain). Such rock is a regional body; a three-block sheet of gneiss between
//! greenstone and granodiorite is a thin leftover of the plate model's rock pools, not geology.
//!
//! **A horizon's key span stays inside one key domain** (`columns::key::domain`: the regolith,
//! each grade band's strata log, each grade band's pools, the mantle). A merge across domains
//! keeps the span of the member whose class it keeps (the other member's keys become a hiatus:
//! this cell has none of those units), not the union. The union made one horizon's height a
//! straight line over keys that neighbouring cells give to other rocks — a metre of soil merged
//! into six kilometres of basement schist gave the schist the regolith's and the whole strata
//! log's keys, and the smoothing then laid a share of it over the neighbours' shale.
//!
//! Plutons stay bodies (`plutons.rs`); P1-08 writes this layer and the bodies into the world data
//! (this task produces the stacks for the viewer and the tests).

use deeptime_format::{Codec, DType, FormatError, GridDesc, LayerDesc, LoadedLayer};
use rayon::prelude::*;
use serde_json::{Value, json};

use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::synth::EVERY_CLASS;
use crate::world::WorldState;

use super::columns::{self, Tables, Unit, key};
use super::{CellSource, StructureParams};

/// Unused horizon top.
pub const NO_TOP: i16 = i16::MIN;
/// Unused horizon class.
pub const NO_CLASS: u8 = 255;
/// Unused horizon key.
pub const NO_KEY: u16 = u16::MAX;

/// The horizon stacks of a grid.
#[derive(Clone, Debug, PartialEq)]
pub struct Horizons {
    pub grid: GridDesc,
    pub k: u32,
    pub min_y: i32,
    /// `tops[c·K + h]`, 1/16 block.
    pub tops: Vec<i16>,
    /// `classes[c·K + h]`, index into `synth::EVERY_CLASS`.
    pub classes: Vec<u8>,
    /// `keys[c·K + h]`, the horizon's stratigraphic key span [top, base] (`columns::key`).
    pub keys: Vec<[u16; 2]>,
    /// The column rule's tongue parameters written into the layer (keys per band, minor share).
    pub tongues: (u32, f64),
    /// Per class, the least thickness (blocks) of a horizon inside a column, written into the
    /// layer for the column rule (`deeptime_format::strata::drop_slivers`); empty: none.
    pub min_blocks: Vec<u8>,
}

/// A horizon while it is baked: top (1/16 block), class, key span.
#[derive(Clone, Copy, Debug)]
struct H {
    top: i32,
    class: u16,
    key: [u16; 2],
}

/// Bakes one column: `units` top-down under a surface at `surface_y` (blocks) into `tops`,
/// `classes` and `keys` (length K, padded). Returns the number of horizons.
#[allow(clippy::too_many_arguments)]
pub fn bake_cell(
    units: &[Unit],
    surface_y: f64,
    min_y: i32,
    max_y: i32,
    m_per_block_v: f64,
    min_blocks: &[i32],
    tops: &mut [i16],
    classes: &mut [u8],
    keys: &mut [[u16; 2]],
) -> usize {
    let top0 = surface_y.clamp(f64::from(min_y) + 1.0, f64::from(max_y));
    let mut hz: Vec<H> = Vec::with_capacity(units.len());
    let mut y = top0;
    for u in units {
        if y <= f64::from(min_y) {
            break;
        }
        let q = (y * 16.0).round() as i32;
        match hz.last_mut() {
            Some(last) if last.class == u.class && key::joins(last.key, u.key) => {
                last.key[1] = last.key[1].max(u.key[1])
            }
            _ => hz.push(H {
                top: q,
                class: u.class,
                key: u.key,
            }),
        }
        y -= f64::from(u.thickness_m) / m_per_block_v;
    }
    if hz.is_empty() {
        let (class, key) = units.last().map_or((0, [0, 1]), |u| (u.class, u.key));
        hz.push(H {
            top: (top0 * 16.0).round() as i32,
            class,
            key,
        });
    }
    settle(&mut hz, min_y * 16, tops.len(), min_blocks);
    write_cell(&hz, tops, classes, keys)
}

/// Makes `hz` a valid stack: every horizon at least one block thick (the last down to `floor`,
/// 1/16 block), every horizon inside the stack at least its class's `min_blocks` thick, and at
/// most `k` of them. The thinnest horizon under its least thickness goes first: one under a
/// block merges with its thinner neighbour; a thicker one (a sliver of metamorphic or plutonic
/// rock) gives its blocks to the horizon under it where that is of its key domain, else to the
/// one over it where that is, else to the one under it, and that horizon keeps its class and
/// keys. Then, while more than `k` remain, the adjacent pair with the smallest combined
/// thickness merges. A merge keeps the upper top and the thicker member's class, with both key
/// spans where they lie in one key domain and the kept class's span where they do not;
/// neighbours of one class and one domain join. Then the key spans are made to grow downwards without
/// overlapping or vanishing, whatever rounding did.
fn settle(hz: &mut Vec<H>, floor: i32, k: usize, min_blocks: &[i32]) {
    let thick = |hz: &[H], i: usize| -> i32 {
        let below = if i + 1 < hz.len() {
            hz[i + 1].top
        } else {
            floor
        };
        hz[i].top - below
    };
    // The least thickness of horizon i: a block, and inside the stack its class's.
    let need = |hz: &[H], i: usize| -> i32 {
        if i == 0 || i + 1 == hz.len() {
            16
        } else {
            16 * min_blocks
                .get(usize::from(hz[i].class))
                .copied()
                .unwrap_or(1)
                .max(1)
        }
    };
    let merge = |hz: &mut Vec<H>, i: usize| {
        let (ta, tb) = (thick(hz, i), thick(hz, i + 1));
        let (a, b) = (hz[i], hz[i + 1]);
        let upper = ta >= tb;
        let class = if upper { a.class } else { b.class };
        hz[i].class = class;
        hz[i].key = if key::joins(a.key, b.key) {
            [a.key[0], a.key[1].max(b.key[1])]
        } else if upper {
            a.key
        } else {
            b.key
        };
        hz.remove(i + 1);
        if i + 1 < hz.len() && hz[i + 1].class == class && key::joins(hz[i].key, hz[i + 1].key) {
            hz[i].key[1] = hz[i].key[1].max(hz[i + 1].key[1]);
            hz.remove(i + 1);
        }
        if i > 0 && hz[i - 1].class == class && key::joins(hz[i - 1].key, hz[i].key) {
            hz[i - 1].key[1] = hz[i - 1].key[1].max(hz[i].key[1]);
            hz.remove(i);
        }
    };
    loop {
        if hz.len() < 2 {
            break;
        }
        let thin = (0..hz.len())
            .filter(|&i| thick(hz, i) < need(hz, i))
            .min_by_key(|&i| (thick(hz, i), i));
        if let Some(i) = thin
            && thick(hz, i) >= 16
        {
            // A sliver of metamorphic or plutonic rock (inside the stack: horizons lie over and
            // under it): its blocks go to the horizon under it where that is of its key domain,
            // else to the one over it where that is, else to the one under it; that horizon
            // keeps its class and its keys (the sliver's keys become a hiatus), so the same
            // unit is the same rock in the next cell, whichever of the two is thicker there.
            let below = key::joins(hz[i].key, hz[i + 1].key);
            let above = key::joins(hz[i - 1].key, hz[i].key);
            if below || !above {
                hz[i + 1].top = hz[i].top;
            }
            hz.remove(i);
            if hz[i - 1].class == hz[i].class && key::joins(hz[i - 1].key, hz[i].key) {
                hz[i - 1].key[1] = hz[i - 1].key[1].max(hz[i].key[1]);
                hz.remove(i);
            }
            continue;
        }
        if let Some(i) = thin {
            let up = if i > 0 { Some(thick(hz, i - 1)) } else { None };
            let down = if i + 1 < hz.len() {
                Some(thick(hz, i + 1))
            } else {
                None
            };
            let with_up = match (up, down) {
                (Some(a), Some(b)) => a <= b,
                (Some(_), None) => true,
                _ => false,
            };
            merge(hz, if with_up { i - 1 } else { i });
            continue;
        }
        if hz.len() > k {
            let i = (0..hz.len() - 1)
                .min_by_key(|&i| (thick(hz, i) + thick(hz, i + 1), i))
                .expect("two or more horizons");
            merge(hz, i);
            continue;
        }
        break;
    }
    let mut below_prev = 0u16;
    for (h, x) in hz.iter_mut().enumerate() {
        if h > 0 {
            x.key[0] = x.key[0].max(below_prev);
        }
        x.key[1] = x.key[1].max(x.key[0].saturating_add(1));
        below_prev = x.key[1];
    }
}

/// Writes `hz` into one cell's K slots (padded); returns the horizons used.
fn write_cell(hz: &[H], tops: &mut [i16], classes: &mut [u8], keys: &mut [[u16; 2]]) -> usize {
    for h in 0..tops.len() {
        match hz.get(h) {
            Some(x) => {
                tops[h] = x.top.clamp(i32::from(i16::MIN) + 1, i32::from(i16::MAX)) as i16;
                classes[h] = x.class as u8;
                keys[h] = x.key;
            }
            None => {
                tops[h] = NO_TOP;
                classes[h] = NO_CLASS;
                keys[h] = [NO_KEY, NO_KEY];
            }
        }
    }
    hz.len()
}

/// The smoothing window's half-width in samples (`Horizons::smoothed`): (2·5 + 1)² samples,
/// `SMOOTH_SAMPLE_SIGMA` cells of σ each.
pub const SMOOTH_RADIUS_SAMPLES: i64 = 5;
/// σ one sample step can serve: a larger σ samples every `ceil(σ / 2.5)`-th cell.
pub const SMOOTH_SAMPLE_SIGMA: f64 = 2.5;

/// The sample step of a smoothing of σ `sigma` cells (1 up to σ 2.5).
pub fn smooth_stride(sigma: f64) -> i64 {
    (sigma / SMOOTH_SAMPLE_SIGMA).ceil().max(1.0) as i64
}

/// The Gaussian the window's weights are cut from is this many σ wide ([`smooth_axis_weight`]).
pub const SMOOTH_TAPER_WIDEN: f64 = 1.5;

/// The weight, along one axis, of a lattice sample `d` cells from the cell being smoothed
/// (`Horizons::smoothed`; a sample's weight is the product of its two axes'): a Gaussian of
/// [`SMOOTH_TAPER_WIDEN`]·σ lowered by its own value at the window's edge, so that it reaches
/// zero there, and zero beyond.
///
/// P1-23. The weights were a plain Gaussian of σ cut off at the window's edge (2σ, where it
/// still holds 0.135 of its peak). A cell's samples are the lattice points inside its window, so
/// as the cell moved by one a row of samples dropped out at full weight and another came in:
/// the weighted mean position of the samples swung ±0.23 cells round the cell with the lattice's
/// period (four L1 cells, 64 blocks, at the default σ 10; ±0.68 cells at σ 20), which printed
/// that lattice on every smoothed horizon. A weight that is zero at the edge lets samples come
/// and go unnoticed: the swing is under 0.01 cells. The effective width is 0.85σ (the cut-off
/// Gaussian's was 0.89σ).
pub fn smooth_axis_weight(d: f64, sigma: f64) -> f64 {
    let reach = (smooth_stride(sigma) * SMOOTH_RADIUS_SAMPLES) as f64;
    let s2 = 2.0 * (SMOOTH_TAPER_WIDEN * sigma) * (SMOOTH_TAPER_WIDEN * sigma);
    ((-(d * d) / s2).exp() - (-(reach * reach) / s2).exp()).max(0.0)
}

/// How far under the mean rock surface (blocks) the smoothing looks for the unit that fills the
/// height of a cell standing above the mean surface (`Horizons::smoothed`): the thickest unit
/// beginning within it. The beds over that unit (thinner than this, together) rise with the
/// ground.
pub const RIDGE_SEARCH_BLOCKS: i32 = 16;

/// The column rule's tongue parameters a preset asks for: (keys per band, minor share).
pub fn tongue_params(p: &StructureParams) -> (u32, f64) {
    let band = (f64::from(deeptime_format::strata::BAND_KEYS) * p.strata_tongue_scale)
        .round()
        .clamp(1.0, 65_535.0) as u32;
    (band, p.strata_minor_share)
}

impl Horizons {
    /// Cell `c`'s height (1/16 block) at stratigraphic key `s`: its horizon tops at their key
    /// tops, linear inside a span, the next top across a gap, `min_y` below the last span (the
    /// column rule's reading of one stack, `deeptime_format::strata`).
    pub fn height_at(&self, c: usize, s: u16) -> f64 {
        let (tops, _, keys) = self.cell(c);
        let floor = f64::from(self.min_y * 16);
        if tops.is_empty() {
            return floor;
        }
        if s <= keys[0][0] {
            return f64::from(tops[0]);
        }
        let mut h = 0;
        while h + 1 < tops.len() && keys[h + 1][0] <= s {
            h += 1;
        }
        let t = f64::from(tops[h]);
        let b = if h + 1 < tops.len() {
            f64::from(tops[h + 1])
        } else {
            floor
        };
        let [kt, kb] = keys[h];
        if s >= kb {
            return b;
        }
        t - (t - b) * (f64::from(s - kt) / f64::from(kb - kt))
    }

    /// The stacks smoothed laterally, so that neighbouring cells tell one regional story
    /// instead of each its own (the strata log is carried cell by cell and differs a lot between
    /// neighbours): for each cell, the Gaussian-weighted mean (σ `sigma` cells; x wraps, z clamps)
    /// of the neighbours' height functions (weights that reach zero at the window's edge,
    /// [`smooth_axis_weight`]), cut at every key top and base any of them has; each
    /// interval takes the class holding the largest weighted thickness there, equal neighbours
    /// join. The neighbours are the cells on a lattice of step [`smooth_stride`] anchored to the
    /// grid (every cell samples the same lattice, so neighbouring cells see the same data) within
    /// [`SMOOTH_RADIUS_SAMPLES`] steps, so the cost stays (2·5 + 1)² stacks a cell for any σ. The
    /// cell's own surface stays its first top: the regolith and the loose sediment on top
    /// (the unconsolidated classes) lie on it with their mean thickness (they follow the ground,
    /// not the mean surface), and the rock units the neighbours hold above their base pinch out
    /// there. Where the cell stands above the mean, the thickest rock unit that begins within
    /// [`RIDGE_SEARCH_BLOCKS`] of the mean rock surface fills the difference, and the thinner
    /// beds over it keep their thickness and rise with the ground: a ridge 60 blocks over the
    /// mean surface is the region's top formation, not 60 blocks of saprolite, clay or of a
    /// coal seam that happens to be the youngest bed. The stacks
    /// are then settled again (at least a block thick, metamorphic and plutonic rock inside the
    /// stack at least `t.min_blocks` of its class, at most K). An interval takes its class from
    /// horizons that span it, and a span lies in one key domain, so a new horizon's span does
    /// too: a class that wins on both sides of a domain's edge makes two horizons. σ 0 changes
    /// nothing.
    pub fn smoothed(&self, sigma: f64, t: &Tables) -> Horizons {
        if sigma <= 0.0 {
            return self.clone();
        }
        let (w, h) = (i64::from(self.grid.w), i64::from(self.grid.h));
        let k = self.k as usize;
        let step = smooth_stride(sigma);
        let reach = step * SMOOTH_RADIUS_SAMPLES;
        let lattice = |a: i64| -> std::ops::RangeInclusive<i64> {
            // Lattice positions (multiples of step) within a ± reach.
            (a - reach).div_euclid(step) + i64::from((a - reach).rem_euclid(step) != 0)
                ..=(a + reach).div_euclid(step)
        };
        let mut out = self.clone();
        out.tops
            .par_chunks_mut(k)
            .zip(out.classes.par_chunks_mut(k))
            .zip(out.keys.par_chunks_mut(k))
            .enumerate()
            .for_each_init(
                || (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                |(near, keys, ys, weight, win), (c, ((tc, cc), kc))| {
                    let own = self.cell(c);
                    if own.0.is_empty() {
                        return;
                    }
                    let (i, j) = (c as i64 % w, c as i64 / w);
                    near.clear();
                    keys.clear();
                    let mut wsum = 0.0f64;
                    for lj in lattice(j) {
                        let jl = lj * step;
                        for li in lattice(i) {
                            let il = li * step;
                            let (di, dj) = ((il - i) as f64, (jl - j) as f64);
                            let g = smooth_axis_weight(di, sigma) * smooth_axis_weight(dj, sigma);
                            if g <= 0.0 {
                                continue;
                            }
                            let n = (jl.clamp(0, h - 1) * w + il.rem_euclid(w)) as usize;
                            near.push((n, g));
                            wsum += g;
                            for key in self.cell(n).2 {
                                keys.extend_from_slice(key);
                            }
                        }
                    }
                    for x in near.iter_mut() {
                        x.1 /= wsum;
                    }
                    keys.sort_unstable();
                    keys.dedup();
                    let m = keys.len();
                    // ys[q]: the smoothed height at keys[q].
                    ys.clear();
                    ys.resize(m, 0.0f64);
                    for &(n, g) in near.iter() {
                        for (q, &s) in keys.iter().enumerate() {
                            ys[q] += g * self.height_at(n, s);
                        }
                    }
                    let top0 = f64::from(own.0[0]);
                    let mut hz: Vec<H> = Vec::with_capacity(k);
                    // The class of every interval: the most weighted thickness in [s0, s1].
                    win.clear();
                    win.resize(m.saturating_sub(1), NO_CLASS);
                    for q in 0..m.saturating_sub(1) {
                        let (s0, s1) = (keys[q], keys[q + 1]);
                        if ys[q + 1] >= ys[q] {
                            continue;
                        }
                        weight.clear();
                        for &(n, g) in near.iter() {
                            let (tops, classes, nk) = self.cell(n);
                            let Some(hh) =
                                (0..tops.len()).find(|&x| nk[x][0] <= s0 && s1 <= nk[x][1])
                            else {
                                continue;
                            };
                            let th = g * (self.height_at(n, s0) - self.height_at(n, s1));
                            match weight
                                .iter_mut()
                                .find(|e: &&mut (u8, f64)| e.0 == classes[hh])
                            {
                                Some(e) => e.1 += th,
                                None => weight.push((classes[hh], th)),
                            }
                        }
                        if let Some(&(class, _)) = weight
                            .iter()
                            .filter(|e| e.1 > 0.0)
                            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
                        {
                            win[q] = class;
                        }
                    }
                    // The regolith and loose sediment: they lie on the cell's own surface with
                    // their mean thickness, wherever the mean surface is.
                    let loose = |q: usize| {
                        key::domain(keys[q + 1]) == 0
                            || t.unconsolidated.get(usize::from(win[q])) == Some(&true)
                    };
                    // The unit that fills the height of a cell standing over the mean surface:
                    // the thickest rock unit (a run of intervals of one class in one key domain,
                    // by its mean thickness) that begins within `RIDGE_SEARCH_BLOCKS` of the
                    // mean rock surface. The thinner beds over it keep their thickness and rise
                    // with the ground.
                    let mut fill: Option<(usize, f64)> = None;
                    let mut depth = 0.0f64;
                    let mut q = 0;
                    while q + 1 < m && depth < f64::from(16 * RIDGE_SEARCH_BLOCKS) {
                        if win[q] == NO_CLASS || loose(q) {
                            q += 1;
                            continue;
                        }
                        let (class, dom) = (win[q], key::domain(keys[q]));
                        let (mut e, mut th) = (q, 0.0f64);
                        while e + 1 < m
                            && (win[e] == NO_CLASS
                                || (win[e] == class
                                    && key::domain(keys[e]) == dom
                                    && key::domain(keys[e + 1]) == dom))
                        {
                            if win[e] != NO_CLASS {
                                th += ys[e] - ys[e + 1];
                            }
                            e += 1;
                        }
                        if fill.is_none_or(|f| th > f.1) {
                            fill = Some((q, th));
                        }
                        depth += th;
                        q = e.max(q + 1);
                    }
                    let fill = fill.map(|f| f.0);
                    // The loose cover laid on the cell's own surface so far (1/16 block), how far
                    // the cell's rock surface stands over the mean one, and whether a rock
                    // horizon has begun.
                    let (mut cover, mut shift, mut rock) = (0.0f64, 0.0f64, false);
                    for q in 0..m.saturating_sub(1) {
                        let class = win[q];
                        if class == NO_CLASS {
                            continue;
                        }
                        let (s0, s1) = (keys[q], keys[q + 1]);
                        let drape = !rock && loose(q);
                        let rock_top = top0 - cover;
                        if !drape && !rock {
                            shift = (rock_top - ys[q]).max(0.0);
                        }
                        // Boundaries down to the fill unit's top rise with the ground.
                        let lift = |b: usize| {
                            if fill.is_some_and(|f| b <= f) {
                                shift
                            } else {
                                0.0
                            }
                        };
                        let (y0, y1) = if drape {
                            (rock_top, rock_top - (ys[q] - ys[q + 1]))
                        } else {
                            (
                                (ys[q] + lift(q)).min(rock_top),
                                (ys[q + 1] + lift(q + 1)).min(rock_top),
                            )
                        };
                        if y1 >= y0 {
                            continue;
                        }
                        let top = if hz.is_empty() {
                            own.0[0].into()
                        } else if !drape && !rock {
                            // The first rock unit starts at the cover's base.
                            rock_top.round() as i32
                        } else {
                            y0.round() as i32
                        };
                        if drape {
                            cover += y0 - y1;
                        } else {
                            rock = true;
                        }
                        match hz.last_mut() {
                            Some(last)
                                if last.class == u16::from(class)
                                    && key::joins(last.key, [s0, s1]) =>
                            {
                                last.key[1] = s1
                            }
                            _ => hz.push(H {
                                top,
                                class: u16::from(class),
                                key: [s0, s1],
                            }),
                        }
                    }
                    // Tops strictly falling, none at or below the floor.
                    for x in 1..hz.len() {
                        hz[x].top = hz[x].top.min(hz[x - 1].top - 1);
                    }
                    hz.retain(|x| x.top > self.min_y * 16);
                    if hz.is_empty() {
                        hz.push(H {
                            top: own.0[0].into(),
                            class: u16::from(own.1[0]),
                            key: own.2[0],
                        });
                    }
                    settle(&mut hz, self.min_y * 16, k, &t.min_blocks);
                    write_cell(&hz, tc, cc, kc);
                },
            );
        out
    }
}

impl Horizons {
    pub fn cells(&self) -> usize {
        self.grid.cells()
    }

    /// Cell `c`'s used horizons: (tops, classes, key spans).
    pub fn cell(&self, c: usize) -> (&[i16], &[u8], &[[u16; 2]]) {
        let k = self.k as usize;
        let n = self.classes[c * k..(c + 1) * k]
            .iter()
            .position(|&x| x == NO_CLASS)
            .unwrap_or(k);
        (
            &self.tops[c * k..c * k + n],
            &self.classes[c * k..c * k + n],
            &self.keys[c * k..c * k + n],
        )
    }

    /// The plan's acceptance (P1-07): every stack ordered top-down, every horizon at least one
    /// block thick (the last down to `min_y`), every class a class of the vocabulary, at most K,
    /// padding only after the used horizons; and (format 0.2) every key span non-empty, below the
    /// one over it and not overlapping it.
    pub fn check(&self) -> Result<(), String> {
        let k = self.k as usize;
        let floor = self.min_y * 16;
        for c in 0..self.cells() {
            let (tops, classes, keys) = self.cell(c);
            for h in 0..keys.len() {
                if keys[h][1] <= keys[h][0] {
                    return Err(format!("cell {c} horizon {h}: key span {:?}", keys[h]));
                }
                if h > 0 && keys[h][0] < keys[h - 1][1] {
                    return Err(format!(
                        "cell {c} horizon {h}: key span {:?} overlaps {:?} above it",
                        keys[h],
                        keys[h - 1]
                    ));
                }
            }
            if tops.is_empty() {
                return Err(format!("cell {c}: no horizons"));
            }
            for h in 0..tops.len() {
                if usize::from(classes[h]) >= EVERY_CLASS.len() {
                    return Err(format!("cell {c} horizon {h}: class {}", classes[h]));
                }
                let below = if h + 1 < tops.len() {
                    i32::from(tops[h + 1])
                } else {
                    floor
                };
                if i32::from(tops[h]) - below < 16 {
                    return Err(format!(
                        "cell {c} horizon {h}: top {} and {} below it: thinner than a block",
                        tops[h], below
                    ));
                }
            }
            for h in tops.len()..k {
                if self.tops[c * k + h] != NO_TOP
                    || self.classes[c * k + h] != NO_CLASS
                    || self.keys[c * k + h] != [NO_KEY, NO_KEY]
                {
                    return Err(format!("cell {c} horizon {h}: padding is not unused"));
                }
            }
        }
        Ok(())
    }

    /// What the bake's own stacks hold beyond [`Horizons::check`] (a synthetic world's stacks
    /// need not): every key span inside one key domain (`columns::key::domain`), and every
    /// horizon inside a stack at least its class's `min_blocks` thick.
    pub fn check_bake(&self, min_blocks: &[i32]) -> Result<(), String> {
        for c in 0..self.cells() {
            let (tops, classes, keys) = self.cell(c);
            for h in 0..tops.len() {
                if key::domain(keys[h][0]) != key::domain(keys[h][1]) {
                    return Err(format!(
                        "cell {c} horizon {h}: key span {:?} crosses key domains",
                        keys[h]
                    ));
                }
                let need = min_blocks
                    .get(usize::from(classes[h]))
                    .copied()
                    .unwrap_or(1);
                if h > 0
                    && h + 1 < tops.len()
                    && i32::from(tops[h]) - i32::from(tops[h + 1]) < 16 * need
                {
                    return Err(format!(
                        "cell {c} horizon {h}: class {} is {} sixteenths thick, under its {need} blocks",
                        classes[h],
                        i32::from(tops[h]) - i32::from(tops[h + 1])
                    ));
                }
            }
        }
        Ok(())
    }

    /// The `strata/horizons` layer's descriptor: what the column rule reads beside the cells.
    pub fn desc(&self, codec: Codec) -> LayerDesc {
        let mut d = LayerDesc::scalar(
            "strata/horizons",
            DType::HorizonStack,
            self.grid.clone(),
            codec,
        );
        d.units = "block/16 top, class index, key top, key base".into();
        d.scale = 1.0 / 16.0;
        d.stack_depth = Some(self.k);
        d.tongue_band_keys = Some(self.tongues.0);
        d.tongue_minor_share = Some(self.tongues.1);
        if !self.min_blocks.is_empty() {
            d.min_blocks = Some(self.min_blocks.clone());
        }
        d
    }

    /// The `strata/horizons` layer (per cell K i16 tops, K u8 classes, K u16 key tops, K u16 key
    /// bases).
    pub fn to_layer(&self, codec: Codec) -> Result<LoadedLayer, FormatError> {
        let k = self.k as usize;
        let mut bytes = Vec::with_capacity(self.cells() * 7 * k);
        for c in 0..self.cells() {
            for &t in &self.tops[c * k..(c + 1) * k] {
                bytes.extend_from_slice(&t.to_le_bytes());
            }
            bytes.extend_from_slice(&self.classes[c * k..(c + 1) * k]);
            for side in 0..2 {
                for key in &self.keys[c * k..(c + 1) * k] {
                    bytes.extend_from_slice(&key[side].to_le_bytes());
                }
            }
        }
        LoadedLayer::new(self.desc(codec), bytes)
    }

    /// `strata/classes.json`.
    pub fn classes_json() -> Value {
        json!({ "classes": EVERY_CLASS.as_slice() })
    }

    /// The class of each cell's first horizon (the rock at its surface), as f32 for rasters.
    pub fn outcrop(&self) -> Vec<f32> {
        let k = self.k as usize;
        (0..self.cells())
            .map(|c| f32::from(self.classes[c * k]))
            .collect()
    }

    /// Mean horizons per cell.
    pub fn mean_horizons(&self) -> f64 {
        let used = self.classes.iter().filter(|&&x| x != NO_CLASS).count();
        used as f64 / self.cells().max(1) as f64
    }
}

/// Bakes every cell of the world's L1 grid at the end of a run (or at any step after the
/// structure stage ran).
pub fn bake(world: &WorldState, preset: &PlanetPreset) -> Result<Horizons, CoreError> {
    let p = preset.structure.clone().unwrap_or_default();
    let src = CellSource::new(world, preset)?;
    Ok(bake_with(&src, preset, &p, &Tables::get()))
}

/// [`bake`] from an assembled source.
pub fn bake_with(
    src: &CellSource<'_>,
    preset: &PlanetPreset,
    p: &StructureParams,
    t: &Tables,
) -> Horizons {
    let s = &preset.scale;
    // `scale.feature_scale.strata` thickens (or thins) every layer in blocks: the metres a block
    // stands for, vertically, are divided by it for the strata alone (1 leaves them as they are).
    let mv = s.m_per_block_v / s.feature_scale.strata;
    let k = p.horizons as usize;
    let n = src.grid.cells();
    let mut tops = vec![NO_TOP; n * k];
    let mut classes = vec![NO_CLASS; n * k];
    let mut keys = vec![[NO_KEY, NO_KEY]; n * k];
    tops.par_chunks_mut(k)
        .zip(classes.par_chunks_mut(k))
        .zip(keys.par_chunks_mut(k))
        .enumerate()
        .for_each_init(Vec::new, |units, (c, ((tc, cc), kc))| {
            let surface_y = src.surface_y(c, s);
            let depth = (surface_y - f64::from(s.min_y)).max(1.0) * mv + 100.0;
            columns::build(&src.cell(c), t, p, src.now_myr, depth, units);
            bake_cell(
                units,
                surface_y,
                s.min_y,
                s.max_y,
                mv,
                &t.min_blocks,
                tc,
                cc,
                kc,
            );
        });
    Horizons {
        grid: src.grid.clone(),
        k: k as u32,
        min_y: s.min_y,
        tops,
        classes,
        keys,
        tongues: tongue_params(p),
        min_blocks: t
            .min_blocks
            .iter()
            .map(|&b| b.clamp(1, 255) as u8)
            .collect(),
    }
    .smoothed(p.strata_smoothing_cells, t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::columns::Source;
    use crate::surface::lithology::index;

    /// A unit whose key span follows the order it is made in (so a list of them is ordered).
    fn unit(name: &str, th: f32) -> Unit {
        use std::sync::atomic::{AtomicU16, Ordering};
        static NEXT: AtomicU16 = AtomicU16::new(0);
        let k = NEXT.fetch_add(10, Ordering::Relaxed);
        Unit {
            class: index(name),
            thickness_m: th,
            source: Source::Log,
            key: [k, k + 10],
        }
    }

    /// `scale.feature_scale.strata` (the settings audit): the bake gets the metres per block
    /// divided by it, so every layer is that many times thicker in blocks and the stack's tops
    /// move down by the same factor; 1 is the calibrated bake.
    #[test]
    fn the_strata_feature_scale_thickens_every_layer() {
        let units = [
            unit("sandstone", 400.0),
            unit("shale", 600.0),
            unit("limestone", 3000.0),
            unit("granite", 20_000.0),
        ];
        let bake = |mv: f64| {
            let mut tops = vec![0i16; 4];
            let mut classes = vec![0u8; 4];
            let mut keys = vec![[0u16; 2]; 4];
            bake_cell(
                &units,
                100.0,
                -352,
                639,
                mv,
                &[],
                &mut tops,
                &mut classes,
                &mut keys,
            );
            tops
        };
        let base = bake(20.0);
        // Depth below the surface (1/16 blocks) of the second top: 400 m at 20 m/block = 20 blocks.
        assert_eq!(i32::from(base[0]) - i32::from(base[1]), 20 * 16);
        assert_eq!(bake(20.0 / 1.0), base, "a scale of 1 changes nothing");
        let thick = bake(20.0 / 2.0);
        assert_eq!(i32::from(thick[0]) - i32::from(thick[1]), 40 * 16);
        let thin = bake(20.0 / 0.5);
        assert_eq!(i32::from(thin[0]) - i32::from(thin[1]), 10 * 16);
    }

    #[test]
    fn thin_units_merge_and_the_stack_fits_k() {
        let k = 4;
        let mut tops = vec![0i16; k];
        let mut classes = vec![0u8; k];
        let mut keys = vec![[0u16; 2]; k];
        // Surface at y 100 (2,000 m above y 0 at 20 m/block): a 5-m soil, 400 m sandstone, 10 m
        // coal, 600 m shale, 3 km limestone, then granite to depth.
        let units = [
            unit("soil", 5.0),
            unit("sandstone", 400.0),
            unit("coal", 10.0),
            unit("shale", 600.0),
            unit("limestone", 3000.0),
            unit("granite", 20_000.0),
        ];
        let n = bake_cell(
            &units,
            100.0,
            -352,
            639,
            20.0,
            &[],
            &mut tops,
            &mut classes,
            &mut keys,
        );
        assert_eq!(n, 4);
        let h = Horizons {
            grid: GridDesc::for_planet("L1", 64, 32).unwrap(),
            k: k as u32,
            min_y: -352,
            tops: tops.repeat(2),
            classes: classes.repeat(2),
            keys: keys.repeat(2),
            tongues: (64, 0.15),
            min_blocks: Vec::new(),
        };
        h.check().unwrap();
        // The soil and coal were thinner than a block and joined their neighbours.
        assert_eq!(
            &classes[..],
            &[
                index("sandstone") as u8,
                index("shale") as u8,
                index("limestone") as u8,
                index("granite") as u8
            ]
        );
        assert_eq!(tops[0], 1600, "the surface");
        // More horizons than K: the thinnest pairs merge.
        let many: Vec<Unit> = (0..20)
            .map(|i| {
                unit(
                    if i % 2 == 0 { "sandstone" } else { "shale" },
                    100.0 + i as f32,
                )
            })
            .collect();
        let n = bake_cell(
            &many,
            300.0,
            -352,
            639,
            20.0,
            &[],
            &mut tops,
            &mut classes,
            &mut keys,
        );
        assert!((1..=k).contains(&n), "{n}");
        let h = Horizons {
            grid: GridDesc::for_planet("L1", 64, 32).unwrap(),
            k: k as u32,
            min_y: -352,
            tops: tops.repeat(2),
            classes: classes.repeat(2),
            keys: keys.repeat(2),
            tongues: (64, 0.15),
            min_blocks: Vec::new(),
        };
        h.check().unwrap();
        // A surface below the world's floor still gets one horizon a block thick.
        let n = bake_cell(
            &units,
            -500.0,
            -352,
            639,
            20.0,
            &[],
            &mut tops,
            &mut classes,
            &mut keys,
        );
        assert_eq!(n, 1);
        assert_eq!(tops[0], (-351 * 16) as i16);
        assert_eq!(tops[1], NO_TOP);
    }

    /// Units with the keys `columns::build` would give them: a log layer or a pool slot of a band.
    fn keyed(name: &str, th: f32, key: [u16; 2]) -> Unit {
        Unit {
            class: index(name),
            thickness_m: th,
            source: Source::Log,
            key,
        }
    }

    #[test]
    fn metamorphic_and_plutonic_slivers_merge_and_spans_stay_in_their_domain() {
        let t = Tables::get();
        let k = 8;
        let mut tops = vec![0i16; k];
        let mut classes = vec![0u8; k];
        let mut keys = vec![[0u16; 2]; k];
        // A metre of soil on 1 km of schist, a 60-m (3-block) gneiss, a 200-m (10-block)
        // granodiorite, then gabbro: the soil is under a block and joins the schist, which keeps
        // its own keys (not the regolith's); the gneiss sliver and the plutonic sheet are gone.
        let schist = key::pool(4, 4, 0);
        let units = [
            keyed("soil", 1.0, key::REGOLITH),
            keyed("schist", 1000.0, schist),
            keyed("gneiss", 60.0, key::pool(6, 4, 1)),
            keyed("granodiorite", 200.0, key::pool(6, 5, 0)),
            keyed("gabbro", 20_000.0, key::pool(6, 6, 0)),
        ];
        let n = bake_cell(
            &units,
            100.0,
            -352,
            639,
            20.0,
            &t.min_blocks,
            &mut tops,
            &mut classes,
            &mut keys,
        );
        assert_eq!(n, 2, "{classes:?}");
        assert_eq!(classes[0], index("schist") as u8);
        assert_eq!(keys[0], schist, "the soil's keys are a hiatus");
        assert_eq!(classes[1], index("gabbro") as u8);
        let h = Horizons {
            grid: GridDesc::for_planet("L1", 64, 32).unwrap(),
            k: k as u32,
            min_y: -352,
            tops: tops.repeat(2),
            classes: classes.repeat(2),
            keys: keys.repeat(2),
            tongues: (64, 0.15),
            min_blocks: Vec::new(),
        };
        h.check().unwrap();
        h.check_bake(&t.min_blocks).unwrap();
        // Thick enough, they stay; at the surface and at the floor a thin one stays too.
        let units = [
            keyed("schist", 60.0, schist),
            keyed("gneiss", 400.0, key::pool(6, 4, 1)),
            keyed("granodiorite", 800.0, key::pool(6, 5, 0)),
            keyed("gabbro", 20_000.0, key::pool(6, 6, 0)),
        ];
        let n = bake_cell(
            &units,
            100.0,
            -352,
            639,
            20.0,
            &t.min_blocks,
            &mut tops,
            &mut classes,
            &mut keys,
        );
        assert_eq!(n, 4, "{classes:?}");
    }

    /// The smoothing keeps the regolith on the ground: a ridge standing over the mean surface is
    /// its top rock unit under the regolith's own two blocks and the loose sand's three, not
    /// sixty blocks of saprolite or sand.
    #[test]
    fn the_regolith_follows_the_ground_through_the_smoothing() {
        let t = Tables::get();
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let (w, n, k) = (grid.w as usize, grid.cells(), 8);
        let units = [
            keyed("saprolite", 40.0, key::REGOLITH),
            keyed("sand", 60.0, [4096, 4500]),
            keyed("sandstone", 2000.0, [4500, 6000]),
            keyed("granite", 20_000.0, key::pool(6, 5, 0)),
        ];
        let mut tops = vec![NO_TOP; n * k];
        let mut classes = vec![NO_CLASS; n * k];
        let mut keys = vec![[NO_KEY; 2]; n * k];
        // A ridge three cells wide, 60 blocks over the plain.
        let ridge = |c: usize| (30..33).contains(&(c % w));
        for c in 0..n {
            bake_cell(
                &units,
                if ridge(c) { 160.0 } else { 100.0 },
                -352,
                639,
                20.0,
                &t.min_blocks,
                &mut tops[c * k..(c + 1) * k],
                &mut classes[c * k..(c + 1) * k],
                &mut keys[c * k..(c + 1) * k],
            );
        }
        let h = Horizons {
            grid,
            k: k as u32,
            min_y: -352,
            tops,
            classes,
            keys,
            tongues: (64, 0.15),
            min_blocks: Vec::new(),
        }
        .smoothed(2.5, &t);
        h.check().unwrap();
        for c in [10 * w + 31, 10 * w + 5, 10 * w + 29] {
            let (tops, classes, _) = h.cell(c);
            assert_eq!(classes[0], index("saprolite") as u8, "cell {c}");
            assert_eq!(classes[1], index("sand") as u8, "cell {c}");
            assert_eq!(classes[2], index("sandstone") as u8, "cell {c}");
            assert_eq!(
                tops[0] - tops[1],
                2 * 16,
                "cell {c}: the regolith's own 40 m"
            );
            assert_eq!(
                tops[1] - tops[2],
                3 * 16,
                "cell {c}: the loose sand's own 60 m"
            );
            assert_eq!(
                i32::from(tops[0]),
                if ridge(c) { 160 * 16 } else { 100 * 16 },
                "cell {c}"
            );
        }
    }

    /// A ridge is made of the region's top formation: a thin youngest bed (a coal seam) rides
    /// up with the ground instead of filling the ridge.
    #[test]
    fn a_thin_top_bed_does_not_fill_a_ridge() {
        let t = Tables::get();
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let (w, n, k) = (grid.w as usize, grid.cells(), 8);
        let units = [
            keyed("coal", 40.0, [4096, 4200]),
            keyed("sandstone", 2000.0, [4200, 6000]),
            keyed("granite", 20_000.0, key::pool(6, 5, 0)),
        ];
        let mut tops = vec![NO_TOP; n * k];
        let mut classes = vec![NO_CLASS; n * k];
        let mut keys = vec![[NO_KEY; 2]; n * k];
        let ridge = |c: usize| (30..33).contains(&(c % w));
        for c in 0..n {
            bake_cell(
                &units,
                if ridge(c) { 160.0 } else { 100.0 },
                -352,
                639,
                20.0,
                &t.min_blocks,
                &mut tops[c * k..(c + 1) * k],
                &mut classes[c * k..(c + 1) * k],
                &mut keys[c * k..(c + 1) * k],
            );
        }
        let h = Horizons {
            grid,
            k: k as u32,
            min_y: -352,
            tops,
            classes,
            keys,
            tongues: (64, 0.15),
            min_blocks: Vec::new(),
        }
        .smoothed(2.5, &t);
        h.check().unwrap();
        for c in [10 * w + 31, 10 * w + 5] {
            let (tops, classes, _) = h.cell(c);
            assert_eq!(classes[0], index("coal") as u8, "cell {c}");
            assert_eq!(classes[1], index("sandstone") as u8, "cell {c}");
            assert_eq!(tops[0] - tops[1], 2 * 16, "cell {c}: the seam's own 40 m");
        }
        // The sandstone under the ridge is the plain's 100 blocks and the ridge's 60.
        let (plain, ridge_cell) = (h.cell(10 * w + 5).0, h.cell(10 * w + 31).0);
        let extra = (ridge_cell[1] - ridge_cell[2]) - (plain[1] - plain[2]);
        assert!(extra > 25 * 16, "{extra}");
    }

    /// A sliver gives its blocks to the horizon under it, which keeps its class: two thin
    /// basement units between a cover and the lower crust do not turn into more cover.
    #[test]
    fn a_sliver_joins_the_horizon_under_it() {
        let t = Tables::get();
        let k = 8;
        let mut tops = vec![0i16; k];
        let mut classes = vec![0u8; k];
        let mut keys = vec![[0u16; 2]; k];
        let units = [
            keyed("phyllite", 260.0, key::pool(3, 0, 1)),
            keyed("gneiss", 140.0, key::pool(6, 4, 0)),
            keyed("granodiorite", 280.0, key::pool(6, 5, 0)),
            keyed("gabbro", 20_000.0, key::pool(6, 6, 0)),
        ];
        let n = bake_cell(
            &units,
            100.0,
            -352,
            639,
            20.0,
            &t.min_blocks,
            &mut tops,
            &mut classes,
            &mut keys,
        );
        // Gneiss (7 blocks) joins the granodiorite (14 + 7 = 21 blocks, over its 16).
        assert_eq!(n, 3, "{classes:?}");
        assert_eq!(classes[0], index("phyllite") as u8);
        assert_eq!(classes[1], index("granodiorite") as u8);
        assert_eq!(keys[1], key::pool(6, 5, 0));
        assert_eq!(i32::from(tops[0] - tops[1]), 13 * 16);
        assert_eq!(classes[2], index("gabbro") as u8);
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The column rule of `horizon_stack` layers (format 0.2; FORMAT.md, "`horizon_stack` cells"):
//! the horizon stack at one block column, correlated between the strata cells around it.
//! Java's `worlddata/Columns.stack` is its twin, operation for operation (`ColumnParityTest`).
//!
//! Every stored horizon carries a **stratigraphic key span** [top, base]: a u16 that orders units
//! the same way in every cell (the bake gives the regolith, the strata log by age, the basement
//! pools and the mantle their own ranges). A cell's stack is then a function from key to height:
//! at its horizons' key tops the stored tops, linear inside a horizon's span, constant across a
//! gap between spans (a hiatus: that cell has none of those units). The column rule:
//!
//! 1. The 4 × 4 cells around the column, weights the uniform cubic B-spline of the column's
//!    position (smooth to the second derivative; they sum to 1 and are never negative).
//! 2. Every key top and base of the sixteen stacks, sorted: the boundaries of the column's
//!    intervals. The height of a boundary is the weighted sum of the cells' heights at its key —
//!    so a bed present in every cell has its top and base interpolated smoothly, and a bed
//!    missing in some cells thins to nothing towards them (a pinch-out).
//! 3. In each interval the cells may disagree about the rock (a facies change): each class
//!    present gets the weighted sum of the thicknesses its cells give it there. One class fills
//!    the interval; several intertongue ([`interfinger`]): the key axis is cut into bands on a
//!    grid every column shares and each band is shared among the classes in proportion (a minor
//!    class fading out), stacked by their horizons' key-span midpoints, so each tongue thins
//!    towards the cells that lack its class instead of meeting the other at a vertical wall. The
//!    answer depends only on the weights and the stacks, not on how the intervals are cut, so a
//!    cell whose weight is zero changes nothing and the column is continuous across cell edges.
//! 4. Heights are floored to blocks; a horizon left with no block is dropped and equal classes
//!    in a row join. The last horizon reaches down to the floor.

use crate::raster::LayerDesc;

/// Unused horizon class.
pub const NO_CLASS: u8 = 255;
/// Unused horizon key.
pub const NO_KEY: u16 = u16::MAX;

/// One stored horizon of a `horizon_stack` cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoredHorizon {
    /// Top, raw (1/16 block); `i16::MIN` = unused.
    pub top: i16,
    /// Class index; 255 = unused.
    pub class: u8,
    /// Stratigraphic key span [top, base].
    pub key: [u16; 2],
}

impl StoredHorizon {
    pub fn is_used(&self) -> bool {
        self.class != NO_CLASS && self.top != i16::MIN
    }
}

/// One horizon of a column as the strata pass paints it: (top block y, class index, unit key).
/// The unit key is the key top of the stored horizon it came from (the least of them where
/// several cells give it); `StrataPass` picks a class's block member by it.
pub type ColumnHorizon = (i32, u8, u16);

/// The height of `hz` (one cell's used horizons, top-down) at key `s`. `at` is the horizon the
/// previous (smaller or equal) key stopped at, so a walk up the keys of a stack reads each
/// horizon once: the horizon found for `s` is the first one whose successor starts after `s`, a
/// test that holds for every smaller key too, so starting the search from an earlier answer
/// finds the same horizon as starting from the top.
fn height_at(hz: &[StoredHorizon], at: &mut usize, s: u16, floor: f64, d: &LayerDesc) -> f64 {
    let top = |h: &StoredHorizon| f64::from(h.top) * d.scale + d.offset;
    if hz.is_empty() {
        return floor;
    }
    if s <= hz[0].key[0] {
        return top(&hz[0]);
    }
    let mut h = *at;
    while h + 1 < hz.len() && hz[h + 1].key[0] <= s {
        h += 1;
    }
    *at = h;
    let t = top(&hz[h]);
    let b = if h + 1 < hz.len() {
        top(&hz[h + 1])
    } else {
        floor
    };
    let (kt, kb) = (hz[h].key[0], hz[h].key[1]);
    if s >= kb {
        return b;
    }
    t - (t - b) * (f64::from(s - kt) / f64::from(kb - kt))
}

/// The horizon of `hz` whose span holds the key interval [s0, s1], if any. `at` is where the
/// search for an earlier (smaller or equal) `s0` ended, as for [`height_at`].
fn horizon_over(hz: &[StoredHorizon], at: &mut usize, s0: u16, s1: u16) -> Option<usize> {
    if hz.is_empty() || s0 < hz[0].key[0] {
        return None;
    }
    let mut h = *at;
    while h + 1 < hz.len() && hz[h + 1].key[0] <= s0 {
        h += 1;
    }
    *at = h;
    (s1 <= hz[h].key[1]).then_some(h)
}

/// Appends a horizon with top height `top` (blocks) to `out`: floored, never above the one over
/// it; a horizon left empty is replaced, and one of the class above continues it.
fn emit(out: &mut Vec<ColumnHorizon>, top: f64, class: u8, unit: u16) {
    let mut t = top.floor() as i32;
    if let Some(&(last, _, _)) = out.last() {
        t = t.min(last);
        if last == t {
            out.pop();
        }
    }
    if let Some(&(_, c, _)) = out.last()
        && c == class
    {
        return;
    }
    out.push((t, class, unit));
}

/// The horizon stack at block (x, z) of a `horizon_stack` layer described by `d`, whose in-range
/// cells `horizon(i, j, k)` reads; `min_y` is the world's floor. `out` gets the column's horizons
/// top-down (see the module docs); horizon h fills from its top down to the next horizon's
/// top + 1, the last down to the floor.
pub fn stack_at(
    d: &LayerDesc,
    min_y: i32,
    horizon: impl Fn(u32, u32, u32) -> StoredHorizon,
    x: i64,
    z: i64,
    out: &mut Vec<ColumnHorizon>,
) {
    stack_at_with(&mut StackScratch::default(), d, min_y, horizon, x, z, out);
}

/// Working memory of [`stack_at_with`]: the sixteen cells' horizons, the keys and the heights at
/// them. A reader that asks for many columns (a map, a section) keeps one per thread and spares
/// the allocations of every column.
#[derive(Default)]
pub struct StackScratch {
    corners: [Vec<StoredHorizon>; CELLS],
    keys: Vec<u16>,
    /// The cells' heights at the keys: cell n's at key q is `y[n * m + q]`, m the keys' count.
    y: Vec<f64>,
    entries: Vec<(u32, u8, u16, f64)>,
}

/// [`stack_at`] with the working memory given: the same answer, bit for bit.
pub fn stack_at_with(
    scratch: &mut StackScratch,
    d: &LayerDesc,
    min_y: i32,
    horizon: impl Fn(u32, u32, u32) -> StoredHorizon,
    x: i64,
    z: i64,
    out: &mut Vec<ColumnHorizon>,
) {
    out.clear();
    let StackScratch {
        corners,
        keys,
        y,
        entries,
    } = scratch;
    let g = &d.grid;
    let k = d.stack_depth.unwrap_or(0);
    let c = i64::from(g.w) * i64::from(g.r);
    let wx = (x + c / 2).rem_euclid(c) - c / 2;
    let u = (wx - g.origin_block_x) as f64 / f64::from(g.r);
    let v = (z - g.origin_block_z) as f64 / f64::from(g.r);
    let (i0, j0) = (u.floor() as i64, v.floor() as i64);
    let (bu, bv) = (bspline(u - i0 as f64), bspline(v - j0 as f64));
    // The 4 × 4 cells i0 − 1 … i0 + 2, j0 − 1 … j0 + 2, row by row; weight bu[a]·bv[b].
    let mut w = [0.0f64; CELLS];
    for (b, &wv) in bv.iter().enumerate() {
        for (a, &wu) in bu.iter().enumerate() {
            let n = b * 4 + a;
            w[n] = wu * wv;
            let (ci, cj) = (g.wrap_i(i0 - 1 + a as i64), g.clamp_j(j0 - 1 + b as i64));
            corners[n].clear();
            corners[n].extend(
                (0..k)
                    .map(|h| horizon(ci, cj, h))
                    .take_while(StoredHorizon::is_used),
            );
        }
    }
    keys.clear();
    keys.extend(corners.iter().flatten().flat_map(|h| h.key));
    keys.sort_unstable();
    keys.dedup();
    let m = keys.len();
    if m == 0 {
        return;
    }
    let floor = f64::from(min_y);
    let band_keys = d.tongue_band_keys.unwrap_or(BAND_KEYS).max(1);
    let minor = d.tongue_minor_share.unwrap_or(MINOR_SHARE);
    // Each cell's height at every key; the keys ascend, so each cell's horizons are walked once.
    y.clear();
    y.resize(CELLS * m, 0.0);
    for (hz, row) in corners.iter().zip(y.chunks_mut(m)) {
        let mut at = 0;
        for (cell, &s) in row.iter_mut().zip(keys.iter()) {
            *cell = height_at(hz, &mut at, s, floor, d);
        }
    }
    // The weighted mean as an offset from the cell (i0, j0)'s height, so that equal heights
    // give exactly that height whatever the weights' rounding.
    let height = |q: usize| {
        let base = y[REF * m + q];
        let mut sum = 0.0f64;
        for cn in 0..CELLS {
            sum += w[cn] * (y[cn * m + q] - base);
        }
        base + sum
    };
    if m == 1 {
        // A single key: every stack is one horizon with a degenerate span.
        if let Some(h) = corners.iter().find_map(|hz| hz.first()) {
            emit(out, height(0), h.class, h.key[0]);
        }
        return;
    }
    // (order, class, unit, weighted thickness) of each class an interval holds.
    entries.clear();
    let mut at = [0usize; CELLS];
    let mut top = height(0);
    for q in 0..m - 1 {
        let (s0, s1) = (keys[q], keys[q + 1]);
        let bottom = height(q + 1);
        entries.clear();
        for cn in 0..CELLS {
            if w[cn] <= 0.0 {
                continue;
            }
            let Some(h) = horizon_over(&corners[cn], &mut at[cn], s0, s1) else {
                continue;
            };
            let t = w[cn] * (y[cn * m + q] - y[cn * m + q + 1]);
            if t <= 0.0 {
                continue;
            }
            let s = corners[cn][h];
            let order = u32::from(s.key[0]) + u32::from(s.key[1]);
            match entries.iter_mut().find(|e| e.1 == s.class) {
                Some(e) => {
                    e.0 = e.0.min(order);
                    e.2 = e.2.min(s.key[0]);
                    e.3 += t;
                }
                None => entries.push((order, s.class, s.key[0], t)),
            }
        }
        if entries.len() == 1 {
            emit(out, top, entries[0].1, entries[0].2);
        } else if entries.len() > 1 {
            entries.sort_unstable_by_key(|e| (e.0, e.1));
            interfinger(out, entries, (s0, s1), (top, bottom), band_keys, minor);
        }
        top = bottom;
    }
}

/// Keys per band of an interfingering facies change ([`interfinger`]) where the layer does not
/// say (`LayerDesc::tongue_band_keys`).
pub const BAND_KEYS: u32 = 64;
/// A class's share of an interval below which it takes no part of a band ([`interfinger`]) where
/// the layer does not say (`LayerDesc::tongue_minor_share`).
pub const MINOR_SHARE: f64 = 0.15;

/// Emits the interval [s0, s1] (heights `top` to `bottom`, linear in the key between) where
/// several classes share it (`entries` in stacking order, each with its weighted thickness): the
/// key axis is cut into bands of [`BAND_KEYS`] on a grid every column shares, so bands follow the
/// bedding, and each band is split among the classes in proportion to their thicknesses, the
/// first on top. Across a facies change the classes thus intertongue, each tongue thinning
/// towards the cells that lack its class, instead of meeting at a wall; and the answer does not
/// depend on how the interval was cut (a zero-weight cell's keys change nothing).
fn interfinger(
    out: &mut Vec<ColumnHorizon>,
    entries: &[(u32, u8, u16, f64)],
    (s0, s1): (u16, u16),
    (top, bottom): (f64, f64),
    band_keys: u32,
    minor: f64,
) {
    let mut total = 0.0f64;
    for e in entries {
        total += e.3;
    }
    // Shares less the soft threshold (a minor class fades out continuously instead of leaving
    // a one-block tongue in every band), renormalised; the raw shares if none is left.
    let mut share = [0.0f64; CELLS];
    let mut kept = 0.0f64;
    for (i, e) in entries.iter().enumerate() {
        share[i] = (e.3 / total - minor).max(0.0);
        kept += share[i];
    }
    if kept <= 0.0 {
        kept = 0.0;
        for (i, e) in entries.iter().enumerate() {
            share[i] = e.3 / total;
            kept += share[i];
        }
    }
    let (k0, k1) = (f64::from(s0), f64::from(s1));
    let band = f64::from(band_keys);
    for n in u32::from(s0) / band_keys..=(u32::from(s1) - 1) / band_keys {
        let base = f64::from(n * band_keys);
        let mut sum = 0.0f64;
        for (i, e) in entries.iter().enumerate() {
            let lo = base + band * (sum / kept);
            sum += share[i];
            let hi = base + band * (sum / kept);
            let (lo, hi) = (lo.max(k0), hi.min(k1));
            if lo < hi {
                emit(out, top - (lo - k0) / (k1 - k0) * (top - bottom), e.1, e.2);
            }
        }
    }
}

/// Cells in the column rule's stencil.
pub const CELLS: usize = 16;
/// The stencil index of cell (i0, j0), the reference of the weighted mean.
const REF: usize = 5;

/// The uniform cubic B-spline weights of the four cells i0 − 1 … i0 + 2 at fraction `f` of the
/// way from cell i0 to i0 + 1 (they sum to 1; FORMAT.md gives the same formulas).
pub fn bspline(f: f64) -> [f64; 4] {
    let g = 1.0 - f;
    let f2 = f * f;
    let f3 = f2 * f;
    [
        g * g * g / 6.0,
        (3.0 * f3 - 6.0 * f2 + 4.0) / 6.0,
        (-3.0 * f3 + 3.0 * f2 + 3.0 * f + 1.0) / 6.0,
        f3 / 6.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raster::{Codec, DType, GridDesc, LoadedLayer};

    type Stack = Vec<(i16, u8, u16, u16)>;

    /// An 8 × 4 grid (r 32, C 256): cell column i holds `cell(i)` in every row: (top in blocks,
    /// class, key top, key base) per horizon, K = 4. Cell column i sits at block x = −128 + 32·i.
    fn layer(cell: impl Fn(usize) -> Stack) -> LoadedLayer {
        let grid = GridDesc::for_planet("L1", 256, 32).unwrap();
        let mut d = LayerDesc::scalar(
            "strata/horizons",
            DType::HorizonStack,
            grid.clone(),
            Codec::Raw,
        );
        d.scale = 1.0 / 16.0;
        d.stack_depth = Some(4);
        let mut bytes = Vec::new();
        for _j in 0..grid.h {
            for i in 0..grid.w as usize {
                let st = cell(i);
                let get = |h: usize| st.get(h).copied();
                for h in 0..4 {
                    let t = get(h).map_or(i16::MIN, |x| x.0 * 16);
                    bytes.extend_from_slice(&t.to_le_bytes());
                }
                for h in 0..4 {
                    bytes.push(get(h).map_or(NO_CLASS, |x| x.1));
                }
                for h in 0..4 {
                    bytes.extend_from_slice(&get(h).map_or(NO_KEY, |x| x.2).to_le_bytes());
                }
                for h in 0..4 {
                    bytes.extend_from_slice(&get(h).map_or(NO_KEY, |x| x.3).to_le_bytes());
                }
            }
        }
        LoadedLayer::new(d, bytes).unwrap()
    }

    fn at(l: &LoadedLayer, x: i64) -> Vec<(i32, u8)> {
        let mut out = Vec::new();
        l.stack_at(x, 0, -64, &mut out);
        out.into_iter().map(|(t, c, _)| (t, c)).collect()
    }

    #[test]
    fn the_same_stack_everywhere_is_that_stack() {
        let l = layer(|_| vec![(40, 1, 0, 10), (20, 2, 10, 20), (0, 3, 20, 30)]);
        for x in [-128, -100, -3, 0, 77, 127, 300] {
            assert_eq!(at(&l, x), [(40, 1), (20, 2), (0, 3)], "x {x}");
        }
    }

    #[test]
    fn b_spline_weights_sum_to_one() {
        for f in [0.0, 0.1, 0.5, 0.77, 0.999] {
            let b = bspline(f);
            assert!((b.iter().sum::<f64>() - 1.0).abs() < 1e-12);
            assert!(b.iter().all(|&x| x >= 0.0));
        }
        assert_eq!(bspline(0.0)[3], 0.0);
    }

    #[test]
    fn a_shared_bed_is_smooth_and_a_missing_one_pinches_out() {
        // Cells 0–3: sand over coal over sand over granite; cells 4–7: no coal (a hiatus there),
        // the granite 20 blocks higher.
        let l = layer(|i| {
            if i < 4 {
                vec![
                    (40, 1, 0, 10),
                    (30, 5, 10, 12),
                    (28, 1, 12, 20),
                    (0, 3, 20, 30),
                ]
            } else {
                vec![(60, 1, 0, 10), (20, 3, 20, 30)]
            }
        });
        let coal = |x: i64| {
            let s = at(&l, x);
            s.iter()
                .position(|&(_, c)| c == 5)
                .map_or(0, |h| s[h].0 - s[h + 1].0)
        };
        // Far inside the coal cells the coal is its own 2 blocks, and it is gone well inside
        // the others; in between it thins.
        assert_eq!(coal(-128 + 32), 2);
        assert_eq!(coal(-128 + 32 * 6), 0);
        // Continuity: the tops of the first and last horizons move by at most a block a column.
        let mut prev = at(&l, -128);
        for x in -127..=128 {
            let now = at(&l, x);
            assert!((now[0].0 - prev[0].0).abs() <= 1, "{x}: {prev:?} → {now:?}");
            let (a, b) = (now.last().unwrap(), prev.last().unwrap());
            assert!((a.0 - b.0).abs() <= 1, "{x}: {prev:?} → {now:?}");
            prev = now;
        }
    }

    #[test]
    fn a_facies_change_intertongues_instead_of_a_wall() {
        // One bed spanning three bands of keys, sandstone (6) in cells 0–3, shale (7) in 4–7.
        let l = layer(|i| {
            let class = if i < 4 { 6 } else { 7 };
            vec![(40, class, 0, 768), (-20, 3, 768, 800)]
        });
        // Midway between the two groups (x = −128 + 32·3.5 = −16) both classes tongue in each
        // band; deep inside a group only its own class.
        let mid = at(&l, -16);
        assert!(mid.iter().filter(|&&(_, c)| c == 6).count() >= 2, "{mid:?}");
        assert!(mid.iter().filter(|&&(_, c)| c == 7).count() >= 2, "{mid:?}");
        assert_eq!(at(&l, -128 + 32), [(40, 6), (-20, 3)]);
        assert_eq!(at(&l, -128 + 32 * 6), [(40, 7), (-20, 3)]);
    }

    #[test]
    fn a_cell_without_weight_changes_nothing() {
        // At a cell point (x = 0, cell 4) the cell two to the east (6) has weight 0: whatever
        // keys its stack has, the column is the same.
        let base = |i: usize| -> Stack {
            if i.is_multiple_of(2) {
                vec![(40, 6, 0, 100), (0, 3, 100, 200)]
            } else {
                vec![(30, 7, 0, 100), (5, 3, 100, 200)]
            }
        };
        let a = layer(base);
        let b = layer(|i| {
            if i == 6 {
                vec![(50, 9, 0, 37), (45, 6, 37, 61), (10, 8, 61, 150)]
            } else {
                base(i)
            }
        });
        assert_eq!(at(&a, 0), at(&b, 0));
    }

    // The column rule as it was written before `StackScratch`: each horizon search from the top
    // and every buffer allocated per call. The version that walks the keys (and `stack_at_with`,
    // which a reader asking for many columns reuses) must give this answer exactly.
    fn ref_height_at(hz: &[StoredHorizon], s: u16, floor: f64, d: &LayerDesc) -> f64 {
        let top = |h: &StoredHorizon| f64::from(h.top) * d.scale + d.offset;
        if hz.is_empty() {
            return floor;
        }
        if s <= hz[0].key[0] {
            return top(&hz[0]);
        }
        let mut h = 0;
        while h + 1 < hz.len() && hz[h + 1].key[0] <= s {
            h += 1;
        }
        let t = top(&hz[h]);
        let b = if h + 1 < hz.len() {
            top(&hz[h + 1])
        } else {
            floor
        };
        let (kt, kb) = (hz[h].key[0], hz[h].key[1]);
        if s >= kb {
            return b;
        }
        t - (t - b) * (f64::from(s - kt) / f64::from(kb - kt))
    }

    fn ref_horizon_over(hz: &[StoredHorizon], s0: u16, s1: u16) -> Option<usize> {
        if hz.is_empty() || s0 < hz[0].key[0] {
            return None;
        }
        let mut h = 0;
        while h + 1 < hz.len() && hz[h + 1].key[0] <= s0 {
            h += 1;
        }
        (s1 <= hz[h].key[1]).then_some(h)
    }

    fn ref_stack_at(l: &LoadedLayer, min_y: i32, x: i64, z: i64) -> Vec<ColumnHorizon> {
        let d = &l.desc;
        let mut out = Vec::new();
        let g = &d.grid;
        let k = d.stack_depth.unwrap_or(0);
        let c = i64::from(g.w) * i64::from(g.r);
        let wx = (x + c / 2).rem_euclid(c) - c / 2;
        let u = (wx - g.origin_block_x) as f64 / f64::from(g.r);
        let v = (z - g.origin_block_z) as f64 / f64::from(g.r);
        let (i0, j0) = (u.floor() as i64, v.floor() as i64);
        let (bu, bv) = (bspline(u - i0 as f64), bspline(v - j0 as f64));
        let mut w = [0.0f64; CELLS];
        let mut corners: Vec<Vec<StoredHorizon>> = Vec::with_capacity(CELLS);
        for (b, &wv) in bv.iter().enumerate() {
            for (a, &wu) in bu.iter().enumerate() {
                w[b * 4 + a] = wu * wv;
                let (ci, cj) = (g.wrap_i(i0 - 1 + a as i64), g.clamp_j(j0 - 1 + b as i64));
                corners.push(
                    (0..k)
                        .map(|h| l.horizon(ci, cj, h))
                        .take_while(StoredHorizon::is_used)
                        .collect(),
                );
            }
        }
        let mut keys: Vec<u16> = corners.iter().flatten().flat_map(|h| h.key).collect();
        keys.sort_unstable();
        keys.dedup();
        let m = keys.len();
        if m == 0 {
            return out;
        }
        let floor = f64::from(min_y);
        let band_keys = d.tongue_band_keys.unwrap_or(BAND_KEYS).max(1);
        let minor = d.tongue_minor_share.unwrap_or(MINOR_SHARE);
        let y: Vec<Vec<f64>> = corners
            .iter()
            .map(|hz| {
                keys.iter()
                    .map(|&s| ref_height_at(hz, s, floor, d))
                    .collect()
            })
            .collect();
        let height = |q: usize| {
            let base = y[REF][q];
            let mut sum = 0.0f64;
            for cn in 0..CELLS {
                sum += w[cn] * (y[cn][q] - base);
            }
            base + sum
        };
        if m == 1 {
            if let Some(h) = corners.iter().find_map(|hz| hz.first()) {
                emit(&mut out, height(0), h.class, h.key[0]);
            }
            return out;
        }
        let mut entries: Vec<(u32, u8, u16, f64)> = Vec::with_capacity(4);
        let mut top = height(0);
        for q in 0..m - 1 {
            let (s0, s1) = (keys[q], keys[q + 1]);
            let bottom = height(q + 1);
            entries.clear();
            for cn in 0..CELLS {
                if w[cn] <= 0.0 {
                    continue;
                }
                let Some(h) = ref_horizon_over(&corners[cn], s0, s1) else {
                    continue;
                };
                let t = w[cn] * (y[cn][q] - y[cn][q + 1]);
                if t <= 0.0 {
                    continue;
                }
                let s = corners[cn][h];
                let order = u32::from(s.key[0]) + u32::from(s.key[1]);
                match entries.iter_mut().find(|e| e.1 == s.class) {
                    Some(e) => {
                        e.0 = e.0.min(order);
                        e.2 = e.2.min(s.key[0]);
                        e.3 += t;
                    }
                    None => entries.push((order, s.class, s.key[0], t)),
                }
            }
            if entries.len() == 1 {
                emit(&mut out, top, entries[0].1, entries[0].2);
            } else if entries.len() > 1 {
                entries.sort_unstable_by_key(|e| (e.0, e.1));
                interfinger(
                    &mut out,
                    &entries,
                    (s0, s1),
                    (top, bottom),
                    band_keys,
                    minor,
                );
            }
            top = bottom;
        }
        out
    }

    /// A 24 × 8 grid of random stacks (K = 6): keys ascend down each stack with gaps (hiatuses),
    /// tops descend, classes differ between neighbours, some cells have fewer horizons.
    fn random_layer(seed: u64) -> LoadedLayer {
        let mut s = seed | 1;
        let mut next = move |n: u64| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s % n
        };
        let grid = GridDesc::for_planet("L1", 24 * 16, 16).unwrap();
        let mut d = LayerDesc::scalar(
            "strata/horizons",
            DType::HorizonStack,
            grid.clone(),
            Codec::Raw,
        );
        d.scale = 1.0 / 16.0;
        d.stack_depth = Some(6);
        let mut bytes = Vec::new();
        for _ in 0..grid.cells() {
            let n = 1 + next(6) as usize;
            let (mut key, mut top) = (next(40) as u16, 300 * 16 - next(100 * 16) as i32);
            let mut st = Vec::new();
            for _ in 0..n {
                let span = 1 + next(300) as u16;
                st.push((top as i16, next(7) as u8, key, key + span));
                key += span + next(50) as u16;
                top -= 16 + next(60 * 16) as i32;
            }
            for h in 0..6 {
                bytes.extend_from_slice(&st.get(h).map_or(i16::MIN, |t| t.0).to_le_bytes());
            }
            for h in 0..6 {
                bytes.push(st.get(h).map_or(NO_CLASS, |t| t.1));
            }
            for h in 0..6 {
                bytes.extend_from_slice(&st.get(h).map_or(NO_KEY, |t| t.2).to_le_bytes());
            }
            for h in 0..6 {
                bytes.extend_from_slice(&st.get(h).map_or(NO_KEY, |t| t.3).to_le_bytes());
            }
        }
        LoadedLayer::new(d, bytes).unwrap()
    }

    #[test]
    fn reusing_the_scratch_and_walking_the_keys_changes_no_answer() {
        let mut scratch = StackScratch::default();
        let (mut out, mut total, mut tongues) = (Vec::new(), 0, 0);
        for seed in [1u64, 7, 99, 12345] {
            let l = random_layer(seed);
            for z in (-64..64).step_by(3) {
                for x in (-192..192).step_by(5) {
                    let want = ref_stack_at(&l, -352, x, z);
                    l.stack_at(x, z, -352, &mut out);
                    assert_eq!(out, want, "stack_at, seed {seed} ({x}, {z})");
                    stack_at_with(
                        &mut scratch,
                        &l.desc,
                        -352,
                        |i, j, k| l.horizon(i, j, k),
                        x,
                        z,
                        &mut out,
                    );
                    assert_eq!(out, want, "stack_at_with, seed {seed} ({x}, {z})");
                    total += 1;
                    // A class that comes back lower down: a tongue, or a repeated bed.
                    tongues += usize::from(
                        want.iter()
                            .enumerate()
                            .any(|(i, h)| want[..i].iter().any(|e| e.1 == h.1)),
                    );
                }
            }
        }
        assert!(total > 2000);
        assert!(tongues > 0, "the random stacks do intertongue somewhere");
    }
}

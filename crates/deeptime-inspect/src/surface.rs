// SPDX-License-Identifier: LGPL-3.0-or-later
//! The terrain surface of a strip of columns and the columns a height check leaves out.
//!
//! **Excluded columns** (checked in this order):
//! - `not_full`: the chunk is not at status `full`. Not counted.
//! - `no_data`: the world data has no elevation there (H is nodata).
//! - `no_terrain`: the column has no terrain block at all.
//! - `structure`: the column lies within `structure_margin` blocks (x/z) of a structure piece
//!   whose bounding box reaches up to within `structure_margin` blocks of the column's surface.
//!   Structures rebuild the ground around them (terrain adaptation, foundations).
//! - `carved`: the column lies in a pit of the measured surface, or within `pit_margin` columns
//!   (Chebyshev) of one. A column is in a pit when it lies at least `pit_depth` blocks below the
//!   **morphological closing** of the **opened** measured surface. Opening (the minimum over a
//!   square of half-width `bump_radius`, then the maximum) removes narrow bumps such as aquifer
//!   barriers and geodes; closing (the maximum over a square of half-width ρ, then the minimum)
//!   fills every hollow narrower than 2ρ. Both leave a plane, a slope and any smooth shape wider
//!   than their squares unchanged, so the difference finds what carvers, ravines, noise-cave
//!   entrances and surface lava lakes cut out of the ground. ρ is the largest of 2, 4, 8, 16, …
//!   ≤ `pit_radius` whose square of half-width ρ + 2 · `bump_radius` around the column holds
//!   only terrain-complete columns (status `carvers` or later, so the proto-chunk ring around the
//!   generated area counts); a column with no such square (at the edge of the generated land) is
//!   not tested.
//!   The test reads only the measured surface, never the world data it is compared with. What
//!   it cannot tell apart from a carved pit: a real hollow narrower than 2 · `pit_radius` that
//!   the world data itself asks for (none in the synthetic worlds; rivers from Phase 2 on).

use std::collections::{HashMap, VecDeque};

use crate::chunk::Piece;
use crate::world::World;

#[derive(Clone, Copy, Debug)]
pub struct ExcludeOpts {
    pub pit_radius: i32,
    pub bump_radius: i32,
    pub pit_depth: i32,
    pub pit_margin: i32,
    pub structure_margin: i32,
}

impl Default for ExcludeOpts {
    fn default() -> ExcludeOpts {
        ExcludeOpts {
            pit_radius: 32,
            bump_radius: 4,
            pit_depth: 2,
            pit_margin: 2,
            structure_margin: 12,
        }
    }
}

impl ExcludeOpts {
    pub fn to_json(self) -> serde_json::Value {
        serde_json::json!({
            "pit_radius": self.pit_radius,
            "bump_radius": self.bump_radius,
            "pit_depth": self.pit_depth,
            "pit_margin": self.pit_margin,
            "structure_margin": self.structure_margin,
        })
    }
}

/// Why a column is left out of a height check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Excluded {
    NotFull,
    NoData,
    NoTerrain,
    Structure,
    Carved,
}

impl Excluded {
    pub fn name(self) -> &'static str {
        match self {
            Excluded::NotFull => "not_full",
            Excluded::NoData => "no_data",
            Excluded::NoTerrain => "no_terrain",
            Excluded::Structure => "structure",
            Excluded::Carved => "carved",
        }
    }
}

/// Sliding-window maximum (or minimum) over `[i − r, i + r]`, O(n) with a monotonic deque.
/// `None` entries are ignored; a window with none left gives `None`.
pub fn sliding(v: &[Option<i32>], r: usize, max: bool) -> Vec<Option<i32>> {
    let better = |a: i32, b: i32| if max { a >= b } else { a <= b };
    let n = v.len();
    let mut out = vec![None; n];
    let mut dq: VecDeque<usize> = VecDeque::new();
    let mut next = 0;
    for (i, slot) in out.iter_mut().enumerate() {
        // Admit everything up to i + r.
        while next < n && next <= i + r {
            if let Some(x) = v[next] {
                while let Some(&b) = dq.back() {
                    if better(x, v[b].unwrap()) {
                        dq.pop_back();
                    } else {
                        break;
                    }
                }
                dq.push_back(next);
            }
            next += 1;
        }
        while let Some(&f) = dq.front() {
            if f + r < i {
                dq.pop_front();
            } else {
                break;
            }
        }
        *slot = dq.front().and_then(|&f| v[f]);
    }
    out
}

/// A 2D grid of optional heights, row-major (`z` outer).
struct Grid {
    w: usize,
    h: usize,
    v: Vec<Option<i32>>,
}

impl Grid {
    /// Square max (or min) filter of half-width r, separable.
    fn filter(&self, r: usize, max: bool) -> Grid {
        let mut rows = vec![None; self.v.len()];
        for z in 0..self.h {
            let row = &self.v[z * self.w..(z + 1) * self.w];
            rows[z * self.w..(z + 1) * self.w].copy_from_slice(&sliding(row, r, max));
        }
        let mut out = vec![None; self.v.len()];
        let mut col = vec![None; self.h];
        for x in 0..self.w {
            for z in 0..self.h {
                col[z] = rows[z * self.w + x];
            }
            for (z, c) in sliding(&col, r, max).into_iter().enumerate() {
                out[z * self.w + x] = c;
            }
        }
        Grid {
            w: self.w,
            h: self.h,
            v: out,
        }
    }
}

/// The measured surface over a strip of columns (`xs`, which may wrap across the seam) and
/// z-range, with a margin, plus what the exclusion rules need.
pub struct Surface {
    xs: Vec<i32>,
    x_index: HashMap<i32, usize>,
    z0: i32,
    h: usize,
    top: Vec<Option<i32>>,
    carved: Vec<bool>,
    pieces: Vec<Piece>,
    opts: ExcludeOpts,
}

impl Surface {
    /// `xs`: the target columns' x, west to east (consecutive in the wrapped world); `wrap`
    /// folds any x into the stored range; `z0..z1` the target rows.
    pub fn compute(
        world: &World,
        xs_target: &[i32],
        wrap: impl Fn(i32) -> i32,
        z0: i32,
        z1: i32,
        opts: ExcludeOpts,
    ) -> Surface {
        let rmax = (1..)
            .map(|k| 1i32 << k)
            .take_while(|&r| r <= opts.pit_radius.max(2))
            .last()
            .unwrap_or(2);
        // Closing at a column reads heights up to 2ρ (+ 2 × the opening's radius) away; pits
        // spread by pit_margin.
        let m = 2 * rmax + 2 * opts.bump_radius.max(0) + opts.pit_margin.max(0);
        let first = xs_target.first().copied().unwrap_or(0);
        let n_target = xs_target.len() as i32;
        let xs: Vec<i32> = (-m..n_target + m).map(|u| wrap(first + u)).collect();
        let (gz0, gz1) = (z0 - m, z1 + m);
        let (w, h) = (xs.len(), (gz1 - gz0) as usize);
        let mut top = vec![None; w * h];
        for (zi, z) in (gz0..gz1).enumerate() {
            for (u, &x) in xs.iter().enumerate() {
                if world.chunk(x, z).is_some_and(|c| c.terrain_done()) {
                    top[zi * w + u] = world.terrain_top(x, z);
                }
            }
        }
        // Summed-area table of missing columns, for "is this square all terrain?".
        let mut sat = vec![0u32; (w + 1) * (h + 1)];
        for z in 0..h {
            for x in 0..w {
                let miss = u32::from(top[z * w + x].is_none());
                sat[(z + 1) * (w + 1) + x + 1] =
                    miss + sat[z * (w + 1) + x + 1] + sat[(z + 1) * (w + 1) + x]
                        - sat[z * (w + 1) + x];
            }
        }
        let square_full = |x: usize, z: usize, r: usize| -> bool {
            if x < r || z < r || x + r >= w || z + r >= h {
                return false;
            }
            let (xa, xb, za, zb) = (x - r, x + r + 1, z - r, z + r + 1);
            sat[zb * (w + 1) + xb] + sat[za * (w + 1) + xa]
                == sat[za * (w + 1) + xb] + sat[zb * (w + 1) + xa]
        };
        // Opening first (minimum, then maximum, over a small square) removes narrow bumps —
        // aquifer barriers, geodes — that would otherwise hold the closing up over flat ground.
        let ro = opts.bump_radius.max(0) as usize;
        let grid = Grid {
            w,
            h,
            v: top.clone(),
        }
        .filter(ro, false)
        .filter(ro, true);
        let scales: Vec<usize> = (1..)
            .map(|k| 1usize << k)
            .take_while(|&r| r <= rmax as usize)
            .collect();
        let closings: Vec<Grid> = scales
            .iter()
            .map(|&r| grid.filter(r, true).filter(r, false))
            .collect();
        let mut pit = vec![false; w * h];
        for z in 0..h {
            for x in 0..w {
                let Some(t) = top[z * w + x] else { continue };
                if let Some(si) = (0..scales.len())
                    .rev()
                    .find(|&si| square_full(x, z, scales[si] + 2 * ro))
                    && let Some(c) = closings[si].v[z * w + x]
                {
                    pit[z * w + x] = c - t >= opts.pit_depth;
                }
            }
        }
        let pm = opts.pit_margin.max(0) as usize;
        let near = Grid {
            w,
            h,
            v: pit.iter().map(|&p| Some(i32::from(p))).collect(),
        }
        .filter(pm, true);
        let carved = near.v.iter().map(|v| *v == Some(1)).collect();
        let pieces = world
            .chunks
            .values()
            .flat_map(|c| c.pieces.iter().cloned())
            .collect();
        let x_index = xs.iter().enumerate().map(|(u, &x)| (x, u)).collect();
        Surface {
            xs,
            x_index,
            z0: gz0,
            h,
            top,
            carved,
            pieces,
            opts,
        }
    }

    fn index(&self, x: i32, z: i32) -> Option<usize> {
        let u = *self.x_index.get(&x)?;
        let zi = z - self.z0;
        (zi >= 0 && (zi as usize) < self.h).then(|| zi as usize * self.xs.len() + u)
    }

    /// The terrain surface at (x, z) (terrain-complete chunks in the strip).
    pub fn top(&self, x: i32, z: i32) -> Option<i32> {
        self.top[self.index(x, z)?]
    }

    pub fn carved(&self, x: i32, z: i32) -> bool {
        self.index(x, z).is_some_and(|i| self.carved[i])
    }

    /// The structure piece near (x, z) that reaches the surface `top`, if any.
    pub fn structure(&self, x: i32, z: i32, top: i32) -> Option<&Piece> {
        let m = self.opts.structure_margin;
        self.pieces.iter().find(|p| {
            x >= p.min[0] - m
                && x <= p.max[0] + m
                && z >= p.min[2] - m
                && z <= p.max[2] + m
                && p.max[1] >= top - m
        })
    }

    /// Why (x, z) is excluded from a check against `expected` (None: it is compared).
    pub fn exclusion(
        &self,
        world: &World,
        x: i32,
        z: i32,
        expected: Option<i32>,
    ) -> Option<Excluded> {
        if world.full_chunk(x, z).is_none() {
            return Some(Excluded::NotFull);
        }
        if expected.is_none() {
            return Some(Excluded::NoData);
        }
        let Some(top) = self.top(x, z) else {
            return Some(Excluded::NoTerrain);
        };
        if self.structure(x, z, top).is_some() {
            return Some(Excluded::Structure);
        }
        if self.carved(x, z) {
            return Some(Excluded::Carved);
        }
        None
    }
}

/// Nearest-rank percentile of sorted values (p in 0..=100).
pub fn percentile(sorted: &[i32], p: f64) -> Option<i32> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((p / 100.0) * sorted.len() as f64).ceil().max(1.0) as usize;
    Some(sorted[rank.min(sorted.len()) - 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank() {
        let v: Vec<i32> = (1..=100).collect();
        assert_eq!(percentile(&v, 99.0), Some(99));
        assert_eq!(percentile(&v, 50.0), Some(50));
        assert_eq!(percentile(&v, 100.0), Some(100));
        assert_eq!(percentile(&[0, 0, 5], 99.0), Some(5));
        assert_eq!(percentile(&[], 99.0), None);
    }

    #[test]
    fn sliding_matches_brute_force() {
        let v: Vec<Option<i32>> = (0..60)
            .map(|i: i32| (i % 7 != 3).then_some((i * 37 % 23) - 11))
            .collect();
        for r in [0usize, 1, 2, 5, 9] {
            for max in [true, false] {
                let got = sliding(&v, r, max);
                for (i, g) in got.iter().enumerate() {
                    let lo = i.saturating_sub(r);
                    let hi = (i + r).min(v.len() - 1);
                    let w = v[lo..=hi].iter().flatten().copied();
                    let want = if max { w.max() } else { w.min() };
                    assert_eq!(*g, want, "r={r} max={max} i={i}");
                }
            }
        }
    }

    #[test]
    fn closing_keeps_slopes_and_fills_pits() {
        // A plane rising 1 block per 3 columns in x and 1 per 5 in z, with a 6 × 4 pit 9 deep.
        let (w, h) = (80usize, 70usize);
        let mut v: Vec<Option<i32>> = (0..w * h)
            .map(|i| Some(((i % w) / 3 + (i / w) / 5) as i32))
            .collect();
        for z in 30..34 {
            for x in 40..46 {
                v[z * w + x] = Some(v[z * w + x].unwrap() - 9);
            }
        }
        let g = Grid { w, h, v: v.clone() };
        let c = g.filter(8, true).filter(8, false);
        for z in 16..h - 16 {
            for x in 16..w - 16 {
                let d = c.v[z * w + x].unwrap() - v[z * w + x].unwrap();
                let in_pit = (30..34).contains(&z) && (40..46).contains(&x);
                if in_pit {
                    assert!(d >= 8, "pit at {x},{z} filled by {d}");
                } else {
                    assert_eq!(d, 0, "slope changed at {x},{z}");
                }
            }
        }
    }
}

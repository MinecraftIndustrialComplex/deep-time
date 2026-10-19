// SPDX-License-Identifier: LGPL-3.0-or-later
//! Depression handling: Priority-Flood+ε of Barnes, Lehman & Mulla 2014, "Priority-flood: An
//! optimal depression-filling and watershed-labeling algorithm for digital elevation models"
//! (Computers & Geosciences 62:117–127; arXiv 1511.04463), Algorithms 3 and 4 — implemented from
//! the paper. RichDEM (GPL-3.0) and `Barnes2013-Depressions` (no licence file) were not opened.
//!
//! The flood starts from the base-level cells (the sea, or the fixed rows of the oracle grid) and
//! raises every land cell that cannot drain to at least the next representable value above the
//! cell it was reached from (`f64::next_up`, the paper's "nextafter" ε), so that every land cell
//! has a strictly lower neighbour on the filled surface. Cells reached this way are **lake**
//! cells when the fill exceeds their elevation by more than [`LAKE_TOL_M`]; the difference is
//! the lake depth. Cells raised by a few ulps only (flats) are not lakes.
//!
//! Cells in pits are processed from a plain FIFO queue instead of the priority queue (the paper's
//! improved variant, Algorithm 4), which is what makes it fast on DEMs with many flats and pits.
//! Land that no base-level cell can reach (a closed continent with no sea) is drained through its
//! lowest cell, which becomes a sink.
//!
//! **Parallelism (P1-19, ledger ERO-185).** The surface stages flood through [`flood_with`] and
//! [`open_sea_with`], which take the preset's `surface.numerics.flood`: `parallel` (the default)
//! runs the order-independent flood of [`parallel`] (Barnes 2016's tiled Priority-Flood for the
//! ε = 0 fill, then an exact ε pass per group of undecided cells), `serial` this module's flood.
//! P1-17 kept the flood serial because its result depends on the order it runs in: a cell in a
//! flat or a pit takes `next_up` of whichever neighbour the FIFO pit queue popped first, so where
//! a flat is fed from two sides at the same elevation the serial order shows in the ulps, and
//! lakes are flats by construction (a lake's sediment fill leaves its cells within ulps of one
//! another). Measured on the 500-Myr `pipeline_quick_16k` run, the least fixed point the parallel
//! flood computes differs from this flood in 168 of 250 steps, in about 170 of the ~13 000 raised
//! cells per step, by ulps. Since 2026-10-18 the checkpoints need not match bit for bit (Ben: "I
//! am not worried about determinism, performance is far more important"), and the parallel flood
//! is itself one function of the terrain. Both give a filled surface on which every land cell has
//! a strictly lower neighbour. The routing (`routing.rs`) and the stream-power solve (`spl.rs`)
//! next to it are parallel and bit-identical to their serial loops.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

use super::grid::Dims;
use super::params::FloodMethod;

pub mod parallel;

/// A fill deeper than this makes a lake cell, metres.
pub const LAKE_TOL_M: f64 = 0.01;

/// Cell states for the flood.
pub const LAND: u8 = 0;
/// A base-level cell (sea or fixed boundary).
pub const BASE: u8 = 1;
/// A land cell that became a sink because nothing could drain it.
pub const SINK: u8 = 2;

#[derive(Clone, Copy, PartialEq)]
struct Entry {
    z: f64,
    c: u32,
}

impl Eq for Entry {}

impl Ord for Entry {
    fn cmp(&self, o: &Self) -> Ordering {
        // A min-heap on elevation, then on the cell index (deterministic ties).
        o.z.total_cmp(&self.z).then_with(|| o.c.cmp(&self.c))
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// Marks the open sea: cells at or below `sea_level` in a connected (8-neighbour, x wrapping)
/// body of at least `min_cells` cells are [`BASE`]; everything else is [`LAND`] — a small closed
/// basin below sea level (a rift floor, a Dead Sea) is a depression the flood fills as a lake, not
/// a sea that river loads could never leave.
pub fn open_sea(d: &Dims, h: &[f64], sea_level: f64, min_cells: usize, kind: &mut [u8]) {
    let n = d.cells();
    let mut label = vec![u32::MAX; n];
    let mut todo: Vec<u32> = Vec::new();
    let mut members: Vec<u32> = Vec::new();
    kind.iter_mut().for_each(|k| *k = LAND);
    for s in 0..n {
        if label[s] != u32::MAX || h[s] > sea_level {
            continue;
        }
        label[s] = s as u32;
        todo.clear();
        members.clear();
        todo.push(s as u32);
        while let Some(c) = todo.pop() {
            members.push(c);
            for (m, _) in d.neighbours(c as usize) {
                if label[m] == u32::MAX && h[m] <= sea_level {
                    label[m] = s as u32;
                    todo.push(m as u32);
                }
            }
        }
        if members.len() >= min_cells {
            for &c in &members {
                kind[c as usize] = BASE;
            }
        }
    }
}

/// Fills `filled` with the ε-filled surface of `h`. `kind[c]` is [`BASE`] for base-level cells on
/// entry (their filled level is `base_z[c]`); on return, land cells that had to become sinks are
/// marked [`SINK`]. Returns the number of sinks made.
pub fn flood(d: &Dims, h: &[f64], base_z: &[f64], kind: &mut [u8], filled: &mut [f64]) -> usize {
    let n = d.cells();
    let mut closed = vec![false; n];
    let mut open: BinaryHeap<Entry> = BinaryHeap::new();
    let mut pit: VecDeque<u32> = VecDeque::new();
    for c in 0..n {
        if kind[c] == BASE {
            closed[c] = true;
            filled[c] = base_z[c];
            // Only base cells that touch land need to seed the flood.
            if d.neighbours(c).any(|(m, _)| kind[m] != BASE) {
                open.push(Entry {
                    z: filled[c],
                    c: c as u32,
                });
            }
        }
    }
    let mut sinks = 0;
    let mut next_unclosed = 0usize;
    loop {
        let c = if let Some(c) = pit.pop_front() {
            c as usize
        } else if let Some(e) = open.pop() {
            e.c as usize
        } else {
            // Land nothing could reach: its lowest unclosed cell becomes a sink.
            while next_unclosed < n && closed[next_unclosed] {
                next_unclosed += 1;
            }
            if next_unclosed >= n {
                break;
            }
            let mut best = next_unclosed;
            for c in next_unclosed..n {
                if !closed[c] && h[c] < h[best] {
                    best = c;
                }
            }
            closed[best] = true;
            filled[best] = h[best];
            kind[best] = SINK;
            sinks += 1;
            open.push(Entry {
                z: filled[best],
                c: best as u32,
            });
            continue;
        };
        let zc = filled[c];
        for (m, _) in d.neighbours(c) {
            if closed[m] {
                continue;
            }
            closed[m] = true;
            let up = zc.next_up();
            if h[m] <= up {
                filled[m] = up;
                pit.push_back(m as u32);
            } else {
                filled[m] = h[m];
                open.push(Entry {
                    z: filled[m],
                    c: m as u32,
                });
            }
        }
    }
    sinks
}

/// [`open_sea`] or its parallel form ([`parallel::open_sea`]; the same cells either way).
pub fn open_sea_with(
    m: FloodMethod,
    d: &Dims,
    h: &[f64],
    sea_level: f64,
    min_cells: usize,
    kind: &mut [u8],
) {
    match m {
        FloodMethod::Serial => open_sea(d, h, sea_level, min_cells, kind),
        FloodMethod::Parallel => parallel::open_sea(d, h, sea_level, min_cells, kind),
    }
}

/// [`flood`] or the order-independent parallel flood ([`parallel::flood`]).
pub fn flood_with(
    m: FloodMethod,
    d: &Dims,
    h: &[f64],
    base_z: &[f64],
    kind: &mut [u8],
    filled: &mut [f64],
) -> usize {
    match m {
        FloodMethod::Serial => flood(d, h, base_z, kind, filled),
        FloodMethod::Parallel => parallel::flood(d, h, base_z, kind, filled),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pit_is_filled_to_its_spill_and_drains() {
        // A bowl (rows 2–4, columns 1–3, at 2 m) behind a rim at 10 m with a spill at 5 m in
        // the rim (column 2, row 1); row 0 is the sea.
        let d = Dims::new(5, 6, 1.0, false);
        let mut h = vec![10.0; 30];
        for j in 2..5 {
            for i in 1..4 {
                h[j * 5 + i] = 2.0;
            }
        }
        h[5 + 2] = 5.0;
        let mut kind = vec![LAND; 30];
        for (i, k) in kind.iter_mut().enumerate().take(5) {
            *k = BASE;
            h[i] = 0.0;
        }
        let base_z = vec![0.0; 30];
        let mut f = vec![0.0; 30];
        assert_eq!(flood(&d, &h, &base_z, &mut kind, &mut f), 0);
        for j in 2..5 {
            for i in 1..4 {
                let c = j * 5 + i;
                assert!(f[c] > 5.0 && f[c] < 5.0 + 1e-9, "cell {c}: {}", f[c]);
            }
        }
        assert_eq!(f[7], 5.0, "the spill keeps its height");
        // Every land cell has a strictly lower neighbour on the filled surface.
        for c in 5..30 {
            assert!(d.neighbours(c).any(|(m, _)| f[m] < f[c]), "cell {c}");
        }
    }
}

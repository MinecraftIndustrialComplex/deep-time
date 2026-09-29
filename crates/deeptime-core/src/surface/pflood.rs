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
//! **Parallelism:** the flood is serial (Barnes 2016's tiled parallel version is the optimisation
//! plan §8.2 names; on L1 the serial flood over land cells costs a few tens of milliseconds, so it
//! is not needed yet — see STATUS.md P1-04).

use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

use super::grid::Dims;

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

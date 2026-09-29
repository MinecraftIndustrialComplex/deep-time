// SPDX-License-Identifier: LGPL-3.0-or-later
//! A window of the fine planet grid held in memory, and the neighbourhood rules every amplifier
//! kernel shares (P1-06, plan §5.7.5).
//!
//! The L3 raster wraps in x and is closed at its first and last row (as `surface::grid::Dims`).
//! A window is a rectangle of it — the whole raster when it fits, else one tile plus a halo — and
//! a kernel that steps it sees a neighbour exactly when the planet has one:
//! - a neighbour column beyond the window wraps inside it when the window holds every column
//!   (`w == gw`), else it is **replicated** from the window's edge (a value the halo makes
//!   irrelevant: it can only touch cells the halo has already given up);
//! - a neighbour row beyond the window is **absent** when the window's edge is the planet's edge
//!   (no flow across the poles), else replicated the same way.
//!
//! A pass of dependency radius `r` therefore leaves the cells at least `r` from a replicated edge
//! exactly as a pass over the whole raster would; a tiled run gives every tile a halo of the sum
//! of its passes' radii, which is why a streamed result equals an in-memory one bit for bit
//! (every kernel is a pure function of the previous array, applied row by row).

use rayon::prelude::*;

/// "No neighbour" in a neighbour list.
pub const NONE: u32 = u32::MAX;

/// Column and row offsets of the eight neighbours, `surface::grid::OFFSETS`' order (E, NE, N, NW,
/// W, SW, S, SE; north is row j − 1).
pub const OFFSETS: [(i64, i64); 8] = [
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

/// A `w × h` rectangle of the `gw × gh` planet grid whose first cell is global cell
/// (`gi0`, `gj0`); `gi0` may be negative (the window then starts west of the seam and wraps).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Win {
    pub w: usize,
    pub h: usize,
    pub gi0: i64,
    pub gj0: i64,
    pub gw: usize,
    pub gh: usize,
}

impl Win {
    /// The whole raster.
    pub fn full(gw: usize, gh: usize) -> Win {
        Win {
            w: gw,
            h: gh,
            gi0: 0,
            gj0: 0,
            gw,
            gh,
        }
    }

    pub fn cells(&self) -> usize {
        self.w * self.h
    }

    /// True when the window holds every column of the planet, so a neighbour beyond its east
    /// edge is its west edge.
    #[inline]
    pub fn wraps(&self) -> bool {
        self.w == self.gw
    }

    /// The window's first row is the planet's first row.
    #[inline]
    pub fn top_edge(&self) -> bool {
        self.gj0 == 0
    }

    /// The window's last row is the planet's last row.
    #[inline]
    pub fn bottom_edge(&self) -> bool {
        self.gj0 + self.h as i64 == self.gh as i64
    }

    /// Global cell (column wrapped into [0, gw), row) of window cell (i, j).
    #[inline]
    pub fn global(&self, i: usize, j: usize) -> (usize, usize) {
        (
            (self.gi0 + i as i64).rem_euclid(self.gw as i64) as usize,
            (self.gj0 + j as i64) as usize,
        )
    }

    /// The row-major index of global cell (gi, gj) on the whole planet: a tie-break that is the
    /// same in every window.
    #[inline]
    pub fn global_index(&self, i: usize, j: usize) -> u64 {
        let (gi, gj) = self.global(i, j);
        gj as u64 * self.gw as u64 + gi as u64
    }

    /// The eight neighbours of window cell (i, j), [`NONE`] where the planet has none.
    #[inline]
    pub fn nbrs(&self, i: usize, j: usize, out: &mut [u32; 8]) {
        let (w, h) = (self.w, self.h);
        let c = j * w + i;
        if i > 0 && i + 1 < w && j > 0 && j + 1 < h {
            out[0] = (c + 1) as u32;
            out[1] = (c + 1 - w) as u32;
            out[2] = (c - w) as u32;
            out[3] = (c - 1 - w) as u32;
            out[4] = (c - 1) as u32;
            out[5] = (c + w - 1) as u32;
            out[6] = (c + w) as u32;
            out[7] = (c + w + 1) as u32;
            return;
        }
        for (k, &(di, dj)) in OFFSETS.iter().enumerate() {
            let nj = j as i64 + dj;
            let nj = if nj < 0 {
                if self.top_edge() {
                    out[k] = NONE;
                    continue;
                }
                0
            } else if nj >= h as i64 {
                if self.bottom_edge() {
                    out[k] = NONE;
                    continue;
                }
                h as i64 - 1
            } else {
                nj
            };
            let ni = i as i64 + di;
            let ni = if ni < 0 {
                if self.wraps() { w as i64 - 1 } else { 0 }
            } else if ni >= w as i64 {
                if self.wraps() { 0 } else { w as i64 - 1 }
            } else {
                ni
            };
            out[k] = (nj as usize * w + ni as usize) as u32;
        }
    }
}

/// Distance to each neighbour in cells: 1 for the four sides, √2 for the corners.
pub const STEP: [f32; 8] = [
    1.0,
    std::f32::consts::SQRT_2,
    1.0,
    std::f32::consts::SQRT_2,
    1.0,
    std::f32::consts::SQRT_2,
    1.0,
    std::f32::consts::SQRT_2,
];

/// Applies `f(i, j, out)` to every cell of a window, one row per Rayon task: `out` is the cell's
/// output slot. Every kernel is `out = f(previous arrays)`, so the order the rows run in is
/// irrelevant and the result never depends on the thread count.
pub fn map_cells<T: Send>(win: &Win, out: &mut [T], f: impl Fn(usize, usize, &mut T) + Sync) {
    debug_assert_eq!(out.len(), win.cells());
    out.par_chunks_mut(win.w).enumerate().for_each(|(j, row)| {
        for (i, o) in row.iter_mut().enumerate() {
            f(i, j, o);
        }
    });
}

/// [`map_cells`] with two output arrays.
pub fn map_cells2<T: Send, U: Send>(
    win: &Win,
    a: &mut [T],
    b: &mut [U],
    f: impl Fn(usize, usize, &mut T, &mut U) + Sync,
) {
    debug_assert_eq!(a.len(), win.cells());
    debug_assert_eq!(b.len(), win.cells());
    a.par_chunks_mut(win.w)
        .zip(b.par_chunks_mut(win.w))
        .enumerate()
        .for_each(|(j, (ra, rb))| {
            for (i, (x, y)) in ra.iter_mut().zip(rb.iter_mut()).enumerate() {
                f(i, j, x, y);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_window_wraps_and_stops_at_the_poles() {
        let w = Win::full(4, 3);
        let mut n = [0u32; 8];
        w.nbrs(0, 0, &mut n);
        // E, NE (absent), N (absent), NW (absent), W (wraps to column 3), SW (wraps), S, SE.
        assert_eq!(n, [1, NONE, NONE, NONE, 3, 7, 4, 5]);
        w.nbrs(3, 2, &mut n);
        assert_eq!(n, [8, 4, 7, 6, 10, NONE, NONE, NONE]);
        w.nbrs(1, 1, &mut n);
        assert_eq!(n, [6, 2, 1, 0, 4, 8, 9, 10]);
    }

    #[test]
    fn a_tile_window_replicates_its_open_edges_and_keeps_the_planet_edge() {
        // A tile in the middle of a 16 × 8 planet: nothing is absent.
        let mid = Win {
            w: 4,
            h: 4,
            gi0: 6,
            gj0: 2,
            gw: 16,
            gh: 8,
        };
        let mut n = [0u32; 8];
        mid.nbrs(0, 0, &mut n);
        assert!(n.iter().all(|&x| x != NONE));
        // A tile against the north pole, west of the seam: north is absent, west replicates.
        let pole = Win {
            w: 4,
            h: 4,
            gi0: -2,
            gj0: 0,
            gw: 16,
            gh: 8,
        };
        pole.nbrs(0, 0, &mut n);
        assert_eq!(n[2], NONE);
        assert_ne!(n[4], NONE);
        assert_eq!(pole.global(0, 0), (14, 0));
        assert_eq!(pole.global(2, 3), (0, 3));
        assert_eq!(pole.global_index(2, 3), 3 * 16);
    }
}

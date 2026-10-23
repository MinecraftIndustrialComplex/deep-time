// SPDX-License-Identifier: LGPL-3.0-or-later
//! The order-independent parallel flood (P1-19, item 28 of the gate profile): the least fixed
//! point of the ε relaxation, computed in parallel, so the filled surface is one function of the
//! terrain and nothing else (not the thread count, not the strip size, not a queue's order).
//!
//! **What it computes.** The filled level F of every cell is the unique solution of
//!
//! ```text
//! F[c] = base_z[c]                                     on base-level cells,
//! F[c] = h[c]                                          on sinks,
//! F[c] = max(h[c], next_up(min over neighbours m of F[m]))  on every other cell,
//! ```
//!
//! which is what the plain Priority-Flood+ε of Barnes, Lehman & Mulla 2014 (their Algorithm 3: one
//! priority queue, no pit queue) returns whatever order it breaks ties in: it closes cells in
//! non-decreasing order of F, so the first neighbour a cell is reached from has the least F of
//! its neighbours, and two neighbours tied in F give it the same value. The FIFO pit queue of the
//! serial flood (their Algorithm 4, [`super::flood`]) processes a raised cell before a tied cell
//! still in the priority queue, which is why the serial result depends on order on flats. The
//! solution is unique: if F and G both solve it and F[c] < G[c] at the cell c of least F where
//! they differ, the neighbour m that sets F[c] has F[m] < F[c], so G[m] ≤ F[m] and
//! G[c] ≤ max(h[c], next_up(G[m])) ≤ F[c], a contradiction.
//!
//! **How (two phases).**
//!
//! 1. The ε = 0 fill W (the depression-filled surface: every cell at least the lowest spill on
//!    its way to base level) by the parallel Priority-Flood of Barnes 2016, "Parallel
//!    Priority-Flood depression filling for trillion cell digital elevation models on desktops or
//!    clusters" (Computers & Geosciences 96:56–68; arXiv 1606.06204), implemented from the paper:
//!    the grid is cut into strips (the grid's own [`SurfaceGrid::strips`]: whole rows of the
//!    raster, so the wrap in x stays inside a strip, or whole rows of one cube face); each
//!    strip is flooded on its own from its base-level cells and the cells of its edges that border
//!    another strip (his Algorithm 1), labelling each cell with the watershed it drains to and
//!    noting, for each pair of watersheds that meet, the lowest level at which they meet; the
//!    strips' edges are joined (his Algorithm 2, every edge cell against its neighbours in later
//!    strips); a Priority-Flood over that graph of watersheds from the base level (label 0, at −∞)
//!    gives each watershed the lowest level at which it can drain; and each cell is raised to its
//!    watershed's level. The paper treats ε = 0 only ("Setting ε > 0 requires more complex methods
//!    than those described here"), and W does not depend on order: ties at ε = 0 give equal
//!    values.
//! 2. The ε fill from W. A cell with a neighbour m whose F is surely below its own height has
//!    F = h (it drains, and is not raised): F[m] ≤ next_upᵏ(W[m]) for the k ≤ n cells of the path
//!    that sets W[m], so `ulp_bound(W[m]) < h[c]` decides it without knowing F[m]. The cells left
//!    (lakes, flats, and cells within a few micrometres of their lowest neighbour's fill: about
//!    13,000 of the 16k quick planet's 336,000 a step) form connected groups whose every neighbour
//!    outside is known: a base cell, a sink or a cell with F = h. Each group runs the plain
//!    Priority-Flood+ε from those neighbours (one priority queue, ties by cell index), the groups
//!    in parallel, which gives F exactly (the fixed point restricted to a group, given its
//!    neighbours, is unique by the same argument).
//!
//! **Sinks.** Land that no base cell can reach (a closed continent without sea) drains through
//! its lowest cell, the lowest index among equals: the serial flood's rule, one sink per
//! 8-connected piece of such land. Here they are the watersheds the graph flood cannot reach,
//! grouped by the graph's edges; their lowest cells become sinks and the first phase runs again
//! with them as base level (only when there are any).
//!
//! The open sea ([`open_sea`]) is a connected-component labelling, done here by strips and a
//! union of the strips' edges; which cells are open sea is a property of the terrain, so it is
//! the serial [`super::open_sea`]'s result cell for cell.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::ops::Range;

use rayon::prelude::*;

use super::super::grid::SurfaceGrid;
use super::{BASE, LAND, SINK};

/// Rows per strip of the first phase. A grain of work only: the result is the same for any.
pub(crate) const STRIP_ROWS: usize = 16;

/// Cells per task of the passes that visit every cell on its own.
const CHUNK: usize = 4096;

/// The base level's watershed label.
const OCEAN: u32 = 0;
/// No label yet.
const NONE: u32 = u32::MAX;

#[derive(Clone, Copy, PartialEq)]
struct Entry {
    z: f64,
    c: u32,
}

impl Eq for Entry {}

impl Ord for Entry {
    fn cmp(&self, o: &Self) -> Ordering {
        // A min-heap on the level, then on the cell (or label) index.
        o.z.total_cmp(&self.z).then_with(|| o.c.cmp(&self.c))
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// `v` cut into the strips' slices (contiguous, ascending, covering `v`).
fn split_strips<'a, T>(v: &'a mut [T], strips: &[Range<usize>]) -> Vec<&'a mut [T]> {
    let mut out = Vec::with_capacity(strips.len());
    let mut rest = v;
    for st in strips {
        let (a, b) = std::mem::take(&mut rest).split_at_mut(st.end - st.start);
        out.push(a);
        rest = b;
    }
    debug_assert!(rest.is_empty());
    out
}

/// Whether `c` has a neighbour outside `[lo, hi)`.
#[inline]
fn reaches_out<G: SurfaceGrid>(d: &G, c: usize, lo: usize, hi: usize) -> bool {
    d.neighbours(c).any(|(m, _)| m < lo || m >= hi)
}

/// The cells that hold their level: base cells at `base_z`, sinks at their height.
struct Fixed<'a> {
    kind: &'a [u8],
    base_z: &'a [f64],
    h: &'a [f64],
    /// Sinks made by this flood (the serial flood treats a `SINK` on entry as land; so does this).
    sink: Option<&'a [bool]>,
}

impl Fixed<'_> {
    #[inline]
    fn level(&self, c: usize) -> Option<f64> {
        if self.kind[c] == BASE {
            Some(self.base_z[c])
        } else if self.sink.is_some_and(|s| s[c]) {
            Some(self.h[c])
        } else {
            None
        }
    }
}

/// One strip's flood (Barnes 2016, Algorithm 1): the strip's part of W before the watersheds'
/// levels are known, a local watershed label per cell (`OCEAN` for cells that reach a base cell
/// inside the strip first), and the lowest level at which each pair of watersheds meets.
struct Strip {
    /// Local labels used: 1..=labels.
    labels: u32,
    /// (a, b, level) with a < b, local labels; may repeat a pair.
    edges: Vec<(u32, u32, f64)>,
}

fn strip_fill<G: SurfaceGrid>(
    d: &G,
    fx: &Fixed,
    h: &[f64],
    st: Range<usize>,
    wf: &mut [f64],
    lab: &mut [u32],
) -> Strip {
    let (off, hi) = (st.start, st.end);
    let len = hi - off;
    let mut open: BinaryHeap<Entry> = BinaryHeap::new();
    let mut pit: VecDeque<u32> = VecDeque::new();
    let mut nb = [0usize; 8];
    for l in 0..len {
        let c = off + l;
        match fx.level(c) {
            Some(z) => {
                wf[l] = z;
                lab[l] = OCEAN;
            }
            None => {
                wf[l] = h[c];
                lab[l] = NONE;
            }
        }
    }
    // Seeds: base cells beside a land cell of the strip, at their level, and the land cells of
    // the edges that border another strip, at their height (they may drain across the edge).
    // The heap's order is total (level, then cell), so the order of the pushes does not matter.
    for l in 0..len {
        let c = off + l;
        if lab[l] == OCEAN {
            let k = d.neighbours_in(c, off, hi, &mut nb);
            if nb[..k].iter().any(|&m| lab[m - off] != OCEAN) {
                open.push(Entry {
                    z: wf[l],
                    c: c as u32,
                });
            }
        }
    }
    let mut border = Vec::new();
    d.strip_border(st.clone(), &mut border);
    for &c in &border {
        let l = c - off;
        if lab[l] != OCEAN && reaches_out(d, c, off, hi) {
            open.push(Entry {
                z: wf[l],
                c: c as u32,
            });
        }
    }
    let mut labels = 0u32;
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let mut scan = 0usize;
    loop {
        let c = if let Some(c) = pit.pop_front() {
            c as usize
        } else if let Some(e) = open.pop() {
            e.c as usize
        } else {
            // Land of the strip that no seed reached: a piece of land with no base cell that does
            // not reach the strip's edges. It drains nowhere; the watershed it gets has no edge
            // in the graph, so the global flood finds it and makes it a sink.
            while scan < len && lab[scan] != NONE {
                scan += 1;
            }
            if scan >= len {
                break;
            }
            labels += 1;
            lab[scan] = labels;
            off + scan
        };
        let l = c - off;
        let k = d.neighbours_in(c, off, hi, &mut nb);
        if lab[l] == NONE {
            // An edge cell: the watershed of a labelled neighbour no higher, or a new one.
            lab[l] = match nb[..k]
                .iter()
                .map(|&m| m - off)
                .find(|&ml| lab[ml] != NONE && wf[ml] <= wf[l])
            {
                Some(ml) => lab[ml],
                None => {
                    labels += 1;
                    labels
                }
            };
        }
        let (lc, zc) = (lab[l], wf[l]);
        for &m in &nb[..k] {
            let ml = m - off;
            let lm = lab[ml];
            if lm != NONE {
                if lm != lc {
                    let e = zc.max(wf[ml]);
                    edges.push((lc.min(lm), lc.max(lm), e));
                }
            } else {
                lab[ml] = lc;
                if wf[ml] <= zc {
                    wf[ml] = zc;
                    pit.push_back(m as u32);
                } else {
                    open.push(Entry {
                        z: wf[ml],
                        c: m as u32,
                    });
                }
            }
        }
    }
    Strip { labels, edges }
}

/// Union–find with path halving (the smaller root wins, so the roots do not depend on order).
fn find(p: &mut [u32], mut a: u32) -> u32 {
    while p[a as usize] != a {
        let g = p[p[a as usize] as usize];
        p[a as usize] = g;
        a = g;
    }
    a
}

fn union(p: &mut [u32], a: u32, b: u32) {
    let (ra, rb) = (find(p, a), find(p, b));
    if ra != rb {
        let (lo, hi) = (ra.min(rb), ra.max(rb));
        p[hi as usize] = lo;
    }
}

/// Phase 1: `wf` = W, `lab` = global watershed labels. Returns the cells that must become sinks
/// (empty when every watershed drains to base level).
fn fill_w<G: SurfaceGrid>(
    d: &G,
    fx: &Fixed,
    h: &[f64],
    ranges: &[Range<usize>],
    wf: &mut [f64],
    lab: &mut [u32],
) -> Vec<usize> {
    let strips: Vec<Strip> = split_strips(wf, ranges)
        .into_par_iter()
        .zip(split_strips(lab, ranges))
        .zip(ranges.par_iter())
        .map(|((ws, ls), st)| strip_fill(d, fx, h, st.clone(), ws, ls))
        .collect();
    // Global labels: strip s's local label l is offset[s] + l; 0 stays the base level.
    let mut offset = Vec::with_capacity(strips.len());
    let mut total = 0u32;
    for s in &strips {
        offset.push(total);
        total += s.labels;
    }
    split_strips(lab, ranges)
        .into_par_iter()
        .zip(offset.par_iter())
        .for_each(|(ls, &o)| {
            for v in ls.iter_mut() {
                if *v != OCEAN {
                    *v += o;
                }
            }
        });
    let glob = |o: u32, l: u32| if l == OCEAN { OCEAN } else { o + l };
    // The graph: the strips' own edges, and their joins (Barnes 2016, Algorithm 2: each cell of a
    // strip's edge against its neighbours in later strips; on the raster, a strip's last row
    // against the two or three cells across the edge).
    let joins: Vec<Vec<(u32, u32, f64)>> = ranges
        .par_iter()
        .map(|st| {
            let mut out = Vec::new();
            let mut border = Vec::new();
            d.strip_border(st.clone(), &mut border);
            let (lab, wf) = (&*lab, &*wf);
            for &a in &border {
                for (b, _) in d.neighbours(a) {
                    if b < st.end {
                        continue;
                    }
                    let (la, lb) = (lab[a], lab[b]);
                    if la != lb {
                        out.push((la.min(lb), la.max(lb), wf[a].max(wf[b])));
                    }
                }
            }
            out
        })
        .collect();
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    for (s, st) in strips.iter().enumerate() {
        let o = offset[s];
        edges.extend(st.edges.iter().map(|&(a, b, e)| {
            let (ga, gb) = (glob(o, a), glob(o, b));
            (ga.min(gb), ga.max(gb), e)
        }));
    }
    for j in joins {
        edges.extend(j);
    }
    edges.par_sort_unstable_by(|x, y| x.0.cmp(&y.0).then(x.1.cmp(&y.1)).then(x.2.total_cmp(&y.2)));
    edges.dedup_by(|x, y| x.0 == y.0 && x.1 == y.1);
    // Adjacency (CSR) of the labels 0..=total.
    let nl = total as usize + 1;
    let mut start = vec![0u32; nl + 1];
    for &(a, b, _) in &edges {
        start[a as usize + 1] += 1;
        start[b as usize + 1] += 1;
    }
    for i in 0..nl {
        start[i + 1] += start[i];
    }
    let mut fill = start.clone();
    let mut adj = vec![(0u32, 0.0f64); edges.len() * 2];
    for &(a, b, e) in &edges {
        adj[fill[a as usize] as usize] = (b, e);
        fill[a as usize] += 1;
        adj[fill[b as usize] as usize] = (a, e);
        fill[b as usize] += 1;
    }
    // The Priority-Flood over the graph: each watershed's lowest level of escape.
    let mut level = vec![f64::INFINITY; nl];
    level[OCEAN as usize] = f64::NEG_INFINITY;
    let mut q: BinaryHeap<Entry> = BinaryHeap::new();
    q.push(Entry {
        z: f64::NEG_INFINITY,
        c: OCEAN,
    });
    while let Some(Entry { z, c: a }) = q.pop() {
        if z > level[a as usize] {
            continue;
        }
        for &(b, e) in &adj[start[a as usize] as usize..start[a as usize + 1] as usize] {
            let v = z.max(e);
            if v < level[b as usize] {
                level[b as usize] = v;
                q.push(Entry { z: v, c: b });
            }
        }
    }
    if level[1..].contains(&f64::INFINITY) {
        // Watersheds that cannot drain, grouped into the pieces of land they make; each piece's
        // lowest cell (lowest index among equals) becomes a sink.
        let mut parent: Vec<u32> = (0..nl as u32).collect();
        for &(a, b, _) in &edges {
            if level[a as usize] == f64::INFINITY && level[b as usize] == f64::INFINITY {
                union(&mut parent, a, b);
            }
        }
        let mut lowest: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
        for c in 0..d.cells() {
            let l = lab[c];
            if l == OCEAN || level[l as usize] != f64::INFINITY {
                continue;
            }
            let r = find(&mut parent, l);
            let e = lowest.entry(r).or_insert(c);
            if h[c] < h[*e] {
                *e = c;
            }
        }
        return lowest.into_values().collect();
    }
    wf.par_iter_mut().zip(lab.par_iter()).for_each(|(v, &l)| {
        if l != OCEAN {
            *v = v.max(level[l as usize]);
        }
    });
    Vec::new()
}

/// An upper bound on next_upᵏ(x) for any k ≤ `k` (each step adds at most |x|·2⁻⁵² + 2⁻¹⁰⁷⁴, and
/// |x| at most doubles while k·2⁻⁵² < ½), with a factor of two for the rounding of the sum.
#[inline]
fn ulp_bound(x: f64, k: f64) -> f64 {
    x + 2.0 * k * (2.0 * x.abs() * f64::EPSILON + 2.0 * f64::from_bits(1))
}

/// Fills `filled` with the ε-filled surface of `h`, as [`super::flood`] (same arguments, same
/// sinks), but order-independent and in parallel (the module's notes). Returns the sinks made.
pub fn flood<G: SurfaceGrid>(
    d: &G,
    h: &[f64],
    base_z: &[f64],
    kind: &mut [u8],
    filled: &mut [f64],
) -> usize {
    flood_rows(d, h, base_z, kind, filled, STRIP_ROWS)
}

/// [`flood`] with strips of `rows` rows (the tests vary it; the result must not change).
pub(crate) fn flood_rows<G: SurfaceGrid>(
    d: &G,
    h: &[f64],
    base_z: &[f64],
    kind: &mut [u8],
    filled: &mut [f64],
    rows: usize,
) -> usize {
    let n = d.cells();
    if n == 0 {
        return 0;
    }
    let ranges = d.strips(rows.max(1));
    let mut lab = vec![NONE; n];
    let mut sink_mark: Option<Vec<bool>> = None;
    let mut sinks = 0;
    loop {
        let fx = Fixed {
            kind,
            base_z,
            h,
            sink: sink_mark.as_deref(),
        };
        let new = fill_w(d, &fx, h, &ranges, filled, &mut lab);
        if new.is_empty() {
            break;
        }
        let mark = sink_mark.get_or_insert_with(|| vec![false; n]);
        for c in new {
            mark[c] = true;
            sinks += 1;
        }
    }
    let fx = Fixed {
        kind,
        base_z,
        h,
        sink: sink_mark.as_deref(),
    };
    // Phase 2: the cells whose F is h for sure keep W (= h there); the rest are marked PENDING.
    const PENDING: u32 = u32::MAX;
    const KNOWN: u32 = u32::MAX - 1;
    let k = n as f64;
    {
        let wf: &[f64] = filled;
        lab.par_chunks_mut(CHUNK).enumerate().for_each(|(j, row)| {
            let mut nb = [0usize; 8];
            for (i, v) in row.iter_mut().enumerate() {
                let c = j * CHUNK + i;
                if fx.level(c).is_some() {
                    *v = KNOWN;
                    continue;
                }
                let cnt = d.neighbours_in(c, 0, n, &mut nb);
                let drains = nb[..cnt].iter().any(|&m| {
                    let up = match fx.level(m) {
                        Some(z) => z,
                        None => ulp_bound(wf[m], k),
                    };
                    up < h[c]
                });
                *v = if drains { KNOWN } else { PENDING };
            }
        });
    }
    // The pending cells' connected groups (8 neighbours), in ascending order of their first cell.
    let pending: Vec<u32> = lab
        .par_chunks(CHUNK)
        .enumerate()
        .flat_map_iter(|(j, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, v)| **v == PENDING)
                .map(move |(i, _)| (j * CHUNK + i) as u32)
                .collect::<Vec<_>>()
        })
        .collect();
    let mut groups: Vec<Vec<u32>> = Vec::new();
    let mut nb = [0usize; 8];
    let mut todo: Vec<u32> = Vec::new();
    for &s in &pending {
        if lab[s as usize] != PENDING {
            continue;
        }
        let g = groups.len() as u32;
        let mut members = Vec::new();
        lab[s as usize] = g;
        todo.push(s);
        while let Some(c) = todo.pop() {
            members.push(c);
            let cnt = d.neighbours_in(c as usize, 0, n, &mut nb);
            for &m in &nb[..cnt] {
                if lab[m] == PENDING {
                    lab[m] = g;
                    todo.push(m as u32);
                }
            }
        }
        members.sort_unstable();
        groups.push(members);
    }
    // Each group: the plain Priority-Flood+ε from its known neighbours.
    let wf: &[f64] = filled;
    let lab: &[u32] = &lab;
    let solved: Vec<Vec<f64>> = groups
        .par_iter()
        .enumerate()
        .map(|(g, members)| {
            let g = g as u32;
            let idx = |c: usize| members.binary_search(&(c as u32)).unwrap_or(usize::MAX);
            let mut f = vec![f64::NAN; members.len()];
            let mut closed = vec![false; members.len()];
            let mut q: BinaryHeap<Entry> = BinaryHeap::new();
            let mut nb = [0usize; 8];
            let mut seeds: Vec<u32> = Vec::new();
            for &c in members {
                let cnt = d.neighbours_in(c as usize, 0, n, &mut nb);
                seeds.extend(
                    nb[..cnt]
                        .iter()
                        .filter(|&&m| lab[m] != g)
                        .map(|&m| m as u32),
                );
            }
            seeds.sort_unstable();
            seeds.dedup();
            for m in seeds {
                // A known neighbour: a base cell or sink at its level, or a cell with F = h.
                q.push(Entry {
                    z: wf[m as usize],
                    c: m,
                });
            }
            while let Some(Entry { z, c }) = q.pop() {
                let cnt = d.neighbours_in(c as usize, 0, n, &mut nb);
                for &m in &nb[..cnt] {
                    if lab[m] != g {
                        continue;
                    }
                    let li = idx(m);
                    if closed[li] {
                        continue;
                    }
                    closed[li] = true;
                    let v = h[m].max(z.next_up());
                    f[li] = v;
                    q.push(Entry { z: v, c: m as u32 });
                }
            }
            f
        })
        .collect();
    for (members, f) in groups.iter().zip(&solved) {
        for (&c, &v) in members.iter().zip(f) {
            filled[c as usize] = v;
        }
    }
    if let Some(mark) = &sink_mark {
        for (k, &m) in kind.iter_mut().zip(mark) {
            if m {
                *k = SINK;
            }
        }
    }
    sinks
}

/// Marks the open sea as [`super::open_sea`] does (cells at or below `sea_level` in an
/// 8-connected body of at least `min_cells`, x wrapping), by strips: each strip labels its own
/// bodies, the bodies that touch across a strip edge are joined, and every cell of a body large
/// enough becomes [`BASE`], the rest [`LAND`]. The bodies are the terrain's, so the result is the
/// serial one's cell for cell.
pub fn open_sea<G: SurfaceGrid>(
    d: &G,
    h: &[f64],
    sea_level: f64,
    min_cells: usize,
    kind: &mut [u8],
) {
    let n = d.cells();
    if n == 0 {
        return;
    }
    let ranges = d.strips(STRIP_ROWS);
    let mut lab = vec![NONE; n];
    // Per strip: its bodies' sizes, local labels 0..len.
    let sizes: Vec<Vec<u32>> = split_strips(&mut lab, &ranges)
        .into_par_iter()
        .zip(ranges.par_iter())
        .map(|(ls, st)| {
            let (off, hi) = (st.start, st.end);
            let mut sizes = Vec::new();
            let mut todo: Vec<usize> = Vec::new();
            let mut nb = [0usize; 8];
            for l in 0..ls.len() {
                if ls[l] != NONE || h[off + l] > sea_level {
                    continue;
                }
                let g = sizes.len() as u32;
                let mut count = 0u32;
                ls[l] = g;
                todo.push(off + l);
                while let Some(c) = todo.pop() {
                    count += 1;
                    let k = d.neighbours_in(c, off, hi, &mut nb);
                    for &m in &nb[..k] {
                        if ls[m - off] == NONE && h[m] <= sea_level {
                            ls[m - off] = g;
                            todo.push(m);
                        }
                    }
                }
                sizes.push(count);
            }
            sizes
        })
        .collect();
    let mut offset = Vec::with_capacity(sizes.len());
    let mut total = 0u32;
    for s in &sizes {
        offset.push(total);
        total += s.len() as u32;
    }
    split_strips(&mut lab, &ranges)
        .into_par_iter()
        .zip(offset.par_iter())
        .for_each(|(ls, &o)| {
            for v in ls.iter_mut() {
                if *v != NONE {
                    *v += o;
                }
            }
        });
    // The bodies that touch across a strip's edge (each pair once, from the earlier strip; the
    // union's roots do not depend on the order).
    let mut parent: Vec<u32> = (0..total).collect();
    let mut border = Vec::new();
    for st in &ranges {
        d.strip_border(st.clone(), &mut border);
        for &a in &border {
            if lab[a] == NONE {
                continue;
            }
            for (b, _) in d.neighbours(a) {
                if b >= st.end && lab[b] != NONE {
                    union(&mut parent, lab[a], lab[b]);
                }
            }
        }
    }
    let mut size = vec![0u64; total as usize];
    for (s, sz) in sizes.iter().enumerate() {
        for (l, &v) in sz.iter().enumerate() {
            let r = find(&mut parent, offset[s] + l as u32);
            size[r as usize] += u64::from(v);
        }
    }
    let big: Vec<bool> = (0..total)
        .map(|l| size[find(&mut parent, l) as usize] >= min_cells as u64)
        .collect();
    kind.par_iter_mut().zip(lab.par_iter()).for_each(|(k, &l)| {
        *k = if l != NONE && big[l as usize] {
            BASE
        } else {
            LAND
        };
    });
}

#[cfg(test)]
mod tests;

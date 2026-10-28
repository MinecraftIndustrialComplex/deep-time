// SPDX-License-Identifier: LGPL-3.0-or-later
//! Flow routing: single flow direction (D8) on the ε-filled surface, the donor lists and the stack
//! order of Braun & Willett 2013 ("A very efficient O(n), implicit and parallel method to solve
//! the stream power equation governing fluvial incision and landscape evolution", Geomorphology
//! 180–181:170–179), implemented from the paper.
//!
//! **Why D8 and not MFD:** the O(n) implicit stream-power solve (`spl.rs`) and its extension with
//! deposition need one receiver per cell, as in Braun & Willett and Yuan et al. 2019; the stack
//! (every cell after its receiver) exists only for single-direction routing. At L1 (4.8 km cells)
//! the grid artefacts of D8 (rivers along the eight directions) are below what the keyframes and
//! the deposit engine resolve; multiple-flow routing belongs to the young landscape (P1-06), where
//! channels are drawn at block scale.
//!
//! Every land cell drains to the neighbour of steepest descent on the filled surface (base-level
//! neighbours count at their base level); base-level cells and sinks are their own receivers.
//! Because the filled surface strictly decreases along every receiver path (`pflood.rs`), the
//! receiver graph is a forest whose roots are the base-level cells and sinks: acyclic by
//! construction, which `check_acyclic` verifies in the tests.
//!
//! **Parallel passes (P1-17).** `stack` also builds the [`forest::Forest`] (the stack's cells by
//! position, their receivers, donors and subtree sizes); `accumulate`, `spl::solve` and
//! `spl::families` walk it with a work-stealing top-down scheduler and a bottom-up one in which the
//! last donor to finish carries on to the receiver. Their sums are gathers over the donors in
//! ascending cell index, the order of the serial reverse-stack loops, so every result equals the
//! serial one to the last bit on any thread count (`parity.rs` compares bit patterns against the
//! P1-04 loops on random wrapping grids).

use rayon::prelude::*;

use super::grid::SurfaceGrid;
use super::pflood::{BASE, LAND, SINK};

pub mod forest;
use forest::{Forest, SharedF64};

#[cfg(test)]
mod parity;

/// Cells handed to one task in the parallel sums (`Forest::up`).
const SUM_CHUNK: usize = 64;

/// Receivers, donors and the stack of one routing.
#[derive(Clone, Debug, Default)]
pub struct Routing {
    /// The cell each cell drains to (itself for base-level cells and sinks).
    pub rcv: Vec<u32>,
    /// Distance to the receiver, metres (0 for roots).
    pub len: Vec<f32>,
    /// Land cells (and sinks), every cell after its receiver, basin after basin.
    pub stack: Vec<u32>,
    /// Per basin: its root (a base-level cell or a sink) and its range in `stack`.
    pub basins: Vec<(u32, u32, u32)>,
    /// The forest by position in `stack`, for the parallel passes (`forest.rs`).
    pub forest: Forest,
}

/// D8 receivers of every cell on the filled surface `f`.
pub fn receivers<G: SurfaceGrid>(d: &G, f: &[f64], kind: &[u8], out: &mut Routing) {
    let n = d.cells();
    out.rcv.resize(n, 0);
    out.len.resize(n, 0.0);
    out.rcv
        .par_iter_mut()
        .zip(out.len.par_iter_mut())
        .enumerate()
        .for_each(|(c, (r, l))| {
            if kind[c] != LAND {
                *r = c as u32;
                *l = 0.0;
                return;
            }
            let mut best = c;
            let mut best_s = 0.0;
            let mut best_l = 0.0;
            for (m, dl) in d.neighbours(c) {
                let s = (f[c] - f[m]) / dl;
                if s > best_s {
                    best_s = s;
                    best = m;
                    best_l = dl;
                }
            }
            *r = best as u32;
            *l = best_l as f32;
        });
}

/// Donor lists (CSR) and the stack, basin by basin, rooted at every base-level cell and sink that
/// has donors. Land cells without a path to any root (impossible after the flood) are left out.
pub fn stack<G: SurfaceGrid>(d: &G, kind: &[u8], r: &mut Routing) {
    let n = d.cells();
    let (count, donors) =
        donors_by_neighbours(d, &r.rcv).unwrap_or_else(|| donors_in_order(n, &r.rcv));
    // Each root's basin in parallel, each as the serial walk took it, and the basins in root
    // order (the speed work of 2026-10-28; one walk after another before).
    let roots: Vec<u32> = (0..n as u32)
        .into_par_iter()
        .filter(|&root| {
            let root = root as usize;
            kind[root] != LAND && count[root] != count[root + 1]
        })
        .collect();
    let basins: Vec<Vec<u32>> = roots
        .par_iter()
        .map(|&root| {
            let root = root as usize;
            let mut basin: Vec<u32> = Vec::new();
            if kind[root] == SINK {
                basin.push(root as u32);
            }
            let mut todo: Vec<u32> = donors[count[root] as usize..count[root + 1] as usize].to_vec();
            while let Some(c) = todo.pop() {
                basin.push(c);
                let c = c as usize;
                todo.extend_from_slice(&donors[count[c] as usize..count[c + 1] as usize]);
            }
            basin
        })
        .collect();
    r.basins.clear();
    let mut start = 0u32;
    r.basins.extend(roots.iter().zip(&basins).map(|(&root, b)| {
        let end = start + b.len() as u32;
        let basin = (root, start, end);
        start = end;
        basin
    }));
    r.stack.clear();
    r.stack
        .par_extend(basins.par_iter().flat_map_iter(|b| b.iter().copied()));
    drop(basins);
    debug_assert!(kind.iter().all(|&k| k == BASE || k == LAND || k == SINK));
    r.forest
        .build(&r.rcv, &r.stack, kind, &count, &donors, &r.basins);
}

/// Each cell's donors (the cells that drain to it), ascending by cell index, as offsets (`count`,
/// n + 1) and the list: a counting sort over the cells in order.
fn donors_in_order(n: usize, rcv: &[u32]) -> (Vec<u32>, Vec<u32>) {
    let mut count = vec![0u32; n + 1];
    for c in 0..n {
        let rc = rcv[c] as usize;
        if rc != c {
            count[rc + 1] += 1;
        }
    }
    for c in 0..n {
        count[c + 1] += count[c];
    }
    let mut donors = vec![0u32; count[n] as usize];
    let mut fill = count.clone();
    for c in 0..n {
        let rc = rcv[c] as usize;
        if rc != c {
            donors[fill[rc] as usize] = c as u32;
            fill[rc] += 1;
        }
    }
    (count, donors)
}

/// [`donors_in_order`]'s lists, found in parallel among each cell's neighbours (a cell drains to
/// a neighbour); `None` when some cell drains to a cell that is not its neighbour.
fn donors_by_neighbours<G: SurfaceGrid>(d: &G, rcv: &[u32]) -> Option<(Vec<u32>, Vec<u32>)> {
    let n = rcv.len();
    let of = |rc: usize| -> ([u32; 8], usize) {
        let mut l = [0u32; 8];
        let mut k = 0;
        for (m, _) in d.neighbours(rc) {
            if m != rc && rcv[m] as usize == rc && !l[..k].contains(&(m as u32)) {
                l[k] = m as u32;
                k += 1;
            }
        }
        l[..k].sort_unstable();
        (l, k)
    };
    let k: Vec<u8> = (0..n).into_par_iter().map(|rc| of(rc).1 as u8).collect();
    let movers = rcv
        .par_iter()
        .enumerate()
        .filter(|&(c, &rc)| rc as usize != c)
        .count();
    // Offsets: chunk totals in parallel, the chunks' starts in order, then each chunk's own.
    const CHUNK: usize = 1 << 16;
    let totals: Vec<u32> = k
        .par_chunks(CHUNK)
        .map(|ch| ch.iter().map(|&x| u32::from(x)).sum())
        .collect();
    let mut starts = Vec::with_capacity(totals.len());
    let mut at = 0u32;
    for t in &totals {
        starts.push(at);
        at += t;
    }
    if at as usize != movers {
        return None;
    }
    let mut count = vec![0u32; n + 1];
    count[1..]
        .par_chunks_mut(CHUNK)
        .zip(k.par_chunks(CHUNK))
        .zip(starts.par_iter())
        .for_each(|((out, ch), &s0)| {
            let mut s = s0;
            for (o, &x) in out.iter_mut().zip(ch) {
                s += u32::from(x);
                *o = s;
            }
        });
    drop(k);
    let mut donors = Vec::with_capacity(movers);
    donors.par_extend((0..n).into_par_iter().flat_map_iter(|rc| {
        let (l, k) = of(rc);
        l.into_iter().take(k)
    }));
    Some((count, donors))
}

/// Drainage area of every cell, weighted by `weight` (runoff relative to the reference, so the
/// result is the effective area of plan §8.2's runoff coupling), m². Base-level roots collect the
/// area of their basins.
///
/// Parallel over the drainage forest (`Forest::up`); every cell adds its donors' areas to its own
/// weight in ascending donor order, the order of the serial reverse-stack loop, so the sums are
/// bit-identical to it.
pub fn accumulate<G: SurfaceGrid>(d: &G, r: &Routing, weight: &[f32], area: &mut Vec<f64>) {
    area.clear();
    area.par_extend(
        weight
            .par_iter()
            .enumerate()
            .map(|(c, &w)| f64::from(w) * d.area(c)),
    );
    let f = &r.forest;
    let acc = SharedF64::new(f.len(), 0.0);
    {
        let own: &[f64] = area;
        f.up(SUM_CHUNK, |i| {
            let mut s = own[r.stack[i] as usize];
            for &dp in &f.dpos[f.dptr[i] as usize..f.dptr[i + 1] as usize] {
                s += acc.get(dp as usize);
            }
            acc.set(i, s);
        });
    }
    for (i, &c) in r.stack.iter().enumerate() {
        area[c as usize] = acc.get(i);
    }
    for (k, &o) in f.outlets.iter().enumerate() {
        let mut s = area[o as usize];
        for &dp in &f.opos[f.optr[k] as usize..f.optr[k + 1] as usize] {
            s += acc.get(dp as usize);
        }
        area[o as usize] = s;
    }
}

/// Follows receivers from every cell; true when every path reaches a root within `n` steps.
pub fn check_acyclic(r: &Routing) -> bool {
    let n = r.rcv.len();
    // Depth-first with memo: 0 unknown, 1 on the current path, 2 known to reach a root.
    let mut state = vec![0u8; n];
    let mut path = Vec::new();
    for start in 0..n {
        let mut c = start;
        path.clear();
        loop {
            match state[c] {
                2 => break,
                1 => return false,
                _ => {}
            }
            state[c] = 1;
            path.push(c);
            let rc = r.rcv[c] as usize;
            if rc == c {
                break;
            }
            c = rc;
        }
        for &p in &path {
            state[p] = 2;
        }
    }
    true
}

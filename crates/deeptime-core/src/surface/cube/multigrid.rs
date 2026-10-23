// SPDX-License-Identifier: LGPL-3.0-or-later
//! One multigrid V-cycle as the preconditioner of the cube's diffusion solve: the Lambert cycle
//! (`surface/diffusion/multigrid.rs`, ledger ERO-183) on the cube's levels.
//!
//! **The levels.** Each coarse level is a cube grid of side ⌈n/2⌉: a coarse cell is the 2 × 2
//! block of a face's cells (the last row or column alone when n is odd), so the coarsening never
//! crosses a face's edge, and the couplings across the edges come from the fine ones (a coarse
//! cell couples to every coarse cell one of its fine cells couples to). A coarse level's "mass"
//! is the sum of its block's, its couplings are [`SCALE`] times the sum of the fine couplings
//! between the two blocks, and its diagonal is the mass plus its couplings, as on the raster.
//! Restriction sums a block's residuals, prolongation hands a block's correction to each of its
//! cells (piecewise constant). Smoothing is damped Jacobi ([`OMEGA`], [`NU`] sweeps before and
//! after, the first from zero), the coarsest level (at most [`COARSEST`] cells) takes
//! [`COARSE_SWEEPS`]: the same symmetric cycle, in `f32`.
//!
//! The levels' sparsity (each level's neighbour lists, which fine couplings sum into each coarse
//! one, the blocks) depends on the grid only and is built once per grid size; a solve fills in
//! the values. Every sum runs over a fixed list in a fixed order, so the cycle is the same at any
//! thread count.

use rayon::prelude::*;

use super::CHUNK;
use super::diffusion::Op;
use crate::sphere::cube::{CubeGrid, NO_CELL};
use crate::surface::diffusion::{COARSE_SWEEPS, COARSEST, NU, OMEGA, SCALE};

/// One level's sparsity and its link to the finer level.
struct Pattern {
    /// The level's cube side and cells.
    side: usize,
    cells: usize,
    /// Neighbour lists (CSR).
    ptr: Vec<u32>,
    col: Vec<u32>,
    /// Per entry, the finer level's entries summed into it (level ≥ 1).
    src_ptr: Vec<u32>,
    src: Vec<u32>,
    /// Per cell, its block of finer cells, ascending (level ≥ 1).
    kid_ptr: Vec<u32>,
    kids: Vec<u32>,
    /// Per finer cell, its block's cell here (level ≥ 1).
    parent: Vec<u32>,
}

/// One level's values and the cycle's vectors.
#[derive(Default)]
struct Level {
    val: Vec<f32>,
    mass: Vec<f32>,
    diag: Vec<f32>,
    r: Vec<f32>,
    z: Vec<f32>,
    t: Vec<f32>,
}

/// The levels of one grid size.
pub(crate) struct Hierarchy {
    side: usize,
    pats: Vec<Pattern>,
    levels: Vec<Level>,
}

/// The finest level's pattern: the grid's neighbour table in slot order.
fn finest(g: &CubeGrid) -> Pattern {
    let t = g.topology();
    let n = t.cells();
    let mut ptr = Vec::with_capacity(n + 1);
    let mut col = Vec::with_capacity(8 * n);
    ptr.push(0u32);
    for c in 0..n {
        for &m in t.slots(c) {
            if m != NO_CELL {
                col.push(m);
            }
        }
        ptr.push(col.len() as u32);
    }
    Pattern {
        side: g.n(),
        cells: n,
        ptr,
        col,
        src_ptr: Vec::new(),
        src: Vec::new(),
        kid_ptr: Vec::new(),
        kids: Vec::new(),
        parent: Vec::new(),
    }
}

/// The level above `f`: 2 × 2 blocks of each face.
fn coarsen(f: &Pattern) -> Pattern {
    let nf = f.side;
    let m = nf.div_ceil(2);
    let cells = 6 * m * m;
    let parent: Vec<u32> = (0..f.cells)
        .into_par_iter()
        .map(|c| {
            let (face, rest) = (c / (nf * nf), c % (nf * nf));
            let (i, j) = (rest % nf, rest / nf);
            (face * m * m + (j / 2) * m + i / 2) as u32
        })
        .collect();
    let mut kid_ptr = vec![0u32; cells + 1];
    for &p in &parent {
        kid_ptr[p as usize + 1] += 1;
    }
    for k in 0..cells {
        kid_ptr[k + 1] += kid_ptr[k];
    }
    let mut fill = kid_ptr.clone();
    let mut kids = vec![0u32; f.cells];
    for (c, &p) in parent.iter().enumerate() {
        kids[fill[p as usize] as usize] = c as u32;
        fill[p as usize] += 1;
    }
    // Each coarse cell's neighbours, ascending, and the fine entries of each.
    let rows: Vec<(Vec<u32>, Vec<u32>, Vec<u32>)> = (0..cells)
        .into_par_iter()
        .map(|r| {
            let mut pairs: Vec<(u32, u32)> = Vec::new();
            for &c in &kids[kid_ptr[r] as usize..kid_ptr[r + 1] as usize] {
                let c = c as usize;
                for e in f.ptr[c]..f.ptr[c + 1] {
                    let p = parent[f.col[e as usize] as usize];
                    if p as usize != r {
                        pairs.push((p, e));
                    }
                }
            }
            // Stable: within a neighbour the fine entries keep their order.
            pairs.sort_by_key(|&(p, _)| p);
            let (mut cols, mut counts, mut src) = (Vec::new(), Vec::new(), Vec::new());
            for (p, e) in pairs {
                if cols.last() != Some(&p) {
                    cols.push(p);
                    counts.push(0u32);
                }
                *counts.last_mut().expect("pushed") += 1;
                src.push(e);
            }
            (cols, counts, src)
        })
        .collect();
    let (mut ptr, mut col, mut src_ptr, mut src) = (vec![0u32], Vec::new(), vec![0u32], Vec::new());
    for (cols, counts, s) in rows {
        col.extend_from_slice(&cols);
        ptr.push(col.len() as u32);
        let mut at = *src_ptr.last().expect("seeded");
        for k in counts {
            at += k;
            src_ptr.push(at);
        }
        src.extend_from_slice(&s);
    }
    Pattern {
        side: m,
        cells,
        ptr,
        col,
        src_ptr,
        src,
        kid_ptr,
        kids,
        parent,
    }
}

impl Hierarchy {
    fn new(g: &CubeGrid) -> Hierarchy {
        let mut pats = vec![finest(g)];
        loop {
            let last = pats.last().expect("finest");
            if last.cells <= COARSEST || last.side <= 1 {
                break;
            }
            let next = coarsen(last);
            pats.push(next);
        }
        let levels = pats.iter().map(|_| Level::default()).collect();
        Hierarchy {
            side: g.n(),
            pats,
            levels,
        }
    }
}

fn fit(v: &mut Vec<f32>, n: usize) {
    if v.len() != n {
        v.clear();
        v.resize(n, 0.0);
    }
}

/// The hierarchy for `g` (built once per grid size) with the values of the operator `op`
/// (every cell taking part).
pub(super) fn prepare(hier: &mut Option<Hierarchy>, g: &CubeGrid, op: &Op) {
    debug_assert!(op.all);
    if hier.as_ref().map(|h| h.side) != Some(g.n()) {
        *hier = Some(Hierarchy::new(g));
    }
    let h = hier.as_mut().expect("built");
    // The finest level: the operator in f32.
    {
        let (p, lv) = (&h.pats[0], &mut h.levels[0]);
        fit(&mut lv.val, p.col.len());
        fit(&mut lv.mass, p.cells);
        fit(&mut lv.diag, p.cells);
        fit(&mut lv.r, p.cells);
        fit(&mut lv.z, p.cells);
        fit(&mut lv.t, p.cells);
        let t = g.topology();
        let mut at = 0usize;
        for c in 0..p.cells {
            for (s, &m) in t.slots(c).iter().enumerate() {
                if m != NO_CELL {
                    lv.val[at] = op.k[c][s] as f32;
                    at += 1;
                }
            }
        }
        lv.mass
            .par_iter_mut()
            .zip(lv.diag.par_iter_mut())
            .enumerate()
            .for_each(|(c, (m, d))| {
                *m = op.mu[c] as f32;
                *d = op.diag[c] as f32;
            });
    }
    for l in 1..h.pats.len() {
        let (fine, coarse) = h.levels.split_at_mut(l);
        let (fl, cl) = (&fine[l - 1], &mut coarse[0]);
        let p = &h.pats[l];
        fit(&mut cl.val, p.col.len());
        fit(&mut cl.mass, p.cells);
        fit(&mut cl.diag, p.cells);
        fit(&mut cl.r, p.cells);
        fit(&mut cl.z, p.cells);
        fit(&mut cl.t, p.cells);
        cl.val.par_iter_mut().enumerate().for_each(|(e, v)| {
            let mut s = 0.0f32;
            for &f in &p.src[p.src_ptr[e] as usize..p.src_ptr[e + 1] as usize] {
                s += fl.val[f as usize];
            }
            // ledger: SCL-146, ERO-183 (the raster's constants on 2 × 2 face blocks)
            *v = SCALE * s;
        });
        let val = &cl.val;
        cl.mass
            .par_iter_mut()
            .zip(cl.diag.par_iter_mut())
            .enumerate()
            .for_each(|(r, (m, d))| {
                let mut s = 0.0f32;
                for &c in &p.kids[p.kid_ptr[r] as usize..p.kid_ptr[r + 1] as usize] {
                    s += fl.mass[c as usize];
                }
                let mut k = 0.0f32;
                for &v in &val[p.ptr[r] as usize..p.ptr[r + 1] as usize] {
                    k += v;
                }
                *m = s;
                *d = s + k;
            });
    }
}

/// z = V-cycle(r).
pub(super) fn precondition(h: &mut Hierarchy, r: &[f64], z: &mut [f64]) {
    h.levels[0]
        .r
        .par_iter_mut()
        .zip(r.par_iter())
        .for_each(|(a, &b)| *a = b as f32);
    vcycle(&h.pats, &mut h.levels);
    z.par_iter_mut()
        .zip(h.levels[0].z.par_iter())
        .for_each(|(a, &b)| *a = f64::from(b));
}

/// One damped Jacobi sweep, z ← z + ω (r − A z)/diag (from z = 0 when `first`).
fn sweep(p: &Pattern, lv: &mut Level, first: bool) {
    let Level {
        val, diag, r, z, t, ..
    } = lv;
    if first {
        z.par_iter_mut()
            .zip(r.par_iter().zip(diag.par_iter()))
            .for_each(|(zv, (rv, d))| *zv = OMEGA * rv / d);
        return;
    }
    {
        let z: &[f32] = z;
        t.par_chunks_mut(CHUNK).enumerate().for_each(|(q, tc)| {
            let i0 = q * CHUNK;
            for (d, tv) in tc.iter_mut().enumerate() {
                let i = i0 + d;
                let mut az = diag[i] * z[i];
                for e in p.ptr[i] as usize..p.ptr[i + 1] as usize {
                    az -= val[e] * z[p.col[e] as usize];
                }
                *tv = z[i] + OMEGA * (r[i] - az) / diag[i];
            }
        });
    }
    std::mem::swap(z, t);
}

fn vcycle(pats: &[Pattern], levels: &mut [Level]) {
    let (lv, rest) = levels.split_first_mut().expect("a level");
    let p = &pats[0];
    if rest.is_empty() {
        for k in 0..COARSE_SWEEPS {
            sweep(p, lv, k == 0);
        }
        return;
    }
    for k in 0..NU {
        sweep(p, lv, k == 0);
    }
    // The residual r − A z, summed over each block into the coarse level's r.
    {
        let Level {
            val, diag, r, z, t, ..
        } = &mut *lv;
        let z: &[f32] = z;
        t.par_chunks_mut(CHUNK).enumerate().for_each(|(q, tc)| {
            let i0 = q * CHUNK;
            for (d, tv) in tc.iter_mut().enumerate() {
                let i = i0 + d;
                let mut az = diag[i] * z[i];
                for e in p.ptr[i] as usize..p.ptr[i + 1] as usize {
                    az -= val[e] * z[p.col[e] as usize];
                }
                *tv = r[i] - az;
            }
        });
    }
    let cp = &pats[1];
    {
        let t: &[f32] = &lv.t;
        rest[0].r.par_iter_mut().enumerate().for_each(|(rc, v)| {
            let mut s = 0.0f32;
            for &c in &cp.kids[cp.kid_ptr[rc] as usize..cp.kid_ptr[rc + 1] as usize] {
                s += t[c as usize];
            }
            *v = s;
        });
    }
    vcycle(&pats[1..], rest);
    {
        let zc: &[f32] = &rest[0].z;
        lv.z.par_iter_mut()
            .enumerate()
            .for_each(|(c, v)| *v += zc[cp.parent[c] as usize]);
    }
    for _ in 0..NU {
        sweep(p, lv, false);
    }
}

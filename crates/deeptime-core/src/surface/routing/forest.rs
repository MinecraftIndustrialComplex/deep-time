// SPDX-License-Identifier: LGPL-3.0-or-later
//! The drainage forest in stack order, and two schedulers that run a per-cell closure over it in
//! parallel: [`Forest::down`] (every cell after its receiver: the stream-power solve's first pass)
//! and [`Forest::up`] (every cell after all its donors: the flux pass, the drainage-area sum and
//! the source-mix sum).
//!
//! **Determinism.** The result of a pass must not depend on the thread count or on the order in
//! which the scheduler happens to run independent cells, because every checkpoint and every
//! resumed run is compared bit for bit (plan §6.2, P1-04's determinism tests). Both schedulers
//! keep that property by construction:
//! - the closure of a cell reads only values that are final when the cell runs (its receiver's
//!   for `down`, its donors' for `up`) and writes only its own cell;
//! - a sum over donors is written as a *gather* by the receiver, over the donors in ascending cell
//!   index, starting from the receiver's own term. That is exactly the order in which the serial
//!   reverse-stack loop `acc[rc] += x[c]` adds them (stack order lists the donors of a cell from
//!   the highest index down, so the reverse walk meets them lowest index first), so the floating
//!   point sums are identical to the serial ones, not merely close.
//!
//! Cells are addressed by their *position in the stack*, so the working arrays of a solve are dense
//! (land cells only) and a subtree is one contiguous range.
//!
//! **Sharing without `unsafe`.** Values that tasks write and other tasks read during a pass live in
//! [`SharedF64`] (relaxed atomics over the bits of an `f64`; plain loads and stores on every
//! target we build for). The ordering comes from the schedulers: `down` hands a subtree to a new
//! task only after its root has run (a spawn is a release/acquire edge), `up` counts donors down
//! with acquire-release read-modify-writes and lets the last donor carry on to the receiver.

use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::super::pflood::BASE;

/// "No such position" (a basin's top has no receiver in the stack).
pub const NONE: u32 = u32::MAX;

/// A shared array of `f64` that parallel tasks write disjointly and read after the writer has
/// finished (see the module documentation).
#[derive(Debug, Default)]
pub struct SharedF64(Vec<AtomicU64>);

impl SharedF64 {
    /// `n` values, all `v`.
    pub fn new(n: usize, v: f64) -> Self {
        let bits = v.to_bits();
        SharedF64(rayon::iter::repeat_n(bits, n).map(AtomicU64::new).collect())
    }

    /// A copy of `src`.
    pub fn from_slice(src: &[f64]) -> Self {
        SharedF64(
            src.par_iter()
                .map(|&x| AtomicU64::new(x.to_bits()))
                .collect(),
        )
    }

    #[inline]
    pub fn get(&self, i: usize) -> f64 {
        f64::from_bits(self.0[i].load(Ordering::Relaxed))
    }

    #[inline]
    pub fn set(&self, i: usize, v: f64) {
        self.0[i].store(v.to_bits(), Ordering::Relaxed);
    }

    /// The largest value (NaNs ignored, as `f64::max` does); −∞ when empty.
    pub fn max(&self) -> f64 {
        self.0
            .par_iter()
            .map(|a| f64::from_bits(a.load(Ordering::Relaxed)))
            .reduce(|| f64::NEG_INFINITY, f64::max)
    }
}

/// The drainage forest of one routing, by position in the stack.
#[derive(Debug, Default)]
pub struct Forest {
    /// Position of every cell in the stack ([`NONE`] for cells not in it).
    pub pos: Vec<u32>,
    /// Position of each stack cell's receiver ([`NONE`] for a basin top: its receiver is a
    /// base-level cell, or itself for a sink).
    pub rpos: Vec<u32>,
    /// Donors (positions) of each stack cell, CSR: `dpos[dptr[i]..dptr[i + 1]]`, in ascending
    /// order of the donors' *cell index*.
    pub dptr: Vec<u32>,
    pub dpos: Vec<u32>,
    /// Cells in the subtree of each position, itself included.
    pub size: Vec<u32>,
    /// Positions of the basin tops.
    pub tops: Vec<u32>,
    /// Positions of the cells without donors.
    pub leaves: Vec<u32>,
    /// Base-level cells that receive from the stack (river mouths), with their donors as
    /// positions in the same layout: `opos[optr[k]..optr[k + 1]]` for `outlets[k]`.
    pub outlets: Vec<u32>,
    pub optr: Vec<u32>,
    pub opos: Vec<u32>,
    /// Donors not yet done, per position, for [`Forest::up`]; equal to the donor count between passes.
    pend: Vec<AtomicU32>,
}

impl Clone for Forest {
    fn clone(&self) -> Self {
        Forest {
            pos: self.pos.clone(),
            rpos: self.rpos.clone(),
            dptr: self.dptr.clone(),
            dpos: self.dpos.clone(),
            size: self.size.clone(),
            tops: self.tops.clone(),
            leaves: self.leaves.clone(),
            outlets: self.outlets.clone(),
            optr: self.optr.clone(),
            opos: self.opos.clone(),
            pend: (0..self.dptr.len().saturating_sub(1))
                .map(|i| AtomicU32::new(self.dptr[i + 1] - self.dptr[i]))
                .collect(),
        }
    }
}

impl Forest {
    /// Cells in the stack.
    pub fn len(&self) -> usize {
        self.rpos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rpos.is_empty()
    }

    /// Builds the forest from the stack and the per-cell donor lists (`cptr`/`cdon`, CSR by cell,
    /// donors ascending by cell index) of `routing::stack`.
    pub(super) fn build(
        &mut self,
        rcv: &[u32],
        stack: &[u32],
        kind: &[u8],
        cptr: &[u32],
        cdon: &[u32],
        basins: &[(u32, u32, u32)],
    ) {
        let n = rcv.len();
        let l = stack.len();
        self.pos.clear();
        self.pos.resize(n, NONE);
        for (i, &c) in stack.iter().enumerate() {
            self.pos[c as usize] = i as u32;
        }
        let pos = &self.pos;
        self.rpos.clear();
        self.rpos.extend(stack.iter().map(|&c| {
            let rc = rcv[c as usize];
            if rc == c { NONE } else { pos[rc as usize] }
        }));
        self.dptr.clear();
        self.dptr.reserve(l + 1);
        self.dptr.push(0);
        self.dpos.clear();
        for &c in stack {
            let c = c as usize;
            for &d in &cdon[cptr[c] as usize..cptr[c + 1] as usize] {
                debug_assert_ne!(
                    pos[d as usize], NONE,
                    "a donor of a stack cell is in the stack"
                );
                self.dpos.push(pos[d as usize]);
            }
            self.dptr.push(self.dpos.len() as u32);
        }
        self.size.clear();
        self.size.resize(l, 1);
        for i in (0..l).rev() {
            let p = self.rpos[i];
            if p != NONE {
                self.size[p as usize] += self.size[i];
            }
        }
        self.tops.clear();
        self.leaves.clear();
        for i in 0..l {
            if self.rpos[i] == NONE {
                self.tops.push(i as u32);
            }
            if self.dptr[i] == self.dptr[i + 1] {
                self.leaves.push(i as u32);
            }
        }
        self.outlets.clear();
        self.optr.clear();
        self.optr.push(0);
        self.opos.clear();
        for &(root, _, _) in basins {
            let c = root as usize;
            if kind[c] != BASE {
                continue;
            }
            self.outlets.push(root);
            for &d in &cdon[cptr[c] as usize..cptr[c + 1] as usize] {
                self.opos.push(pos[d as usize]);
            }
            self.optr.push(self.opos.len() as u32);
        }
        self.pend.clear();
        let dptr = &self.dptr;
        self.pend
            .extend((0..l).map(|i| AtomicU32::new(dptr[i + 1] - dptr[i])));
    }

    /// Runs `f(i)` once for every position `i`, each after its receiver's `f`. `grain` is the
    /// number of cells worth handing to another thread (the cost of a spawn against the cost of
    /// `f`).
    ///
    /// A worker keeps a stack of ready subtrees, walks the heaviest child of each cell first and
    /// gives the oldest (highest, so largest) entries of its stack away when it holds more than
    /// two grains of work.
    pub fn down<F: Fn(usize) + Sync>(&self, grain: u32, f: F) {
        let l = self.len();
        if l == 0 {
            return;
        }
        if rayon::current_num_threads() <= 1 || l <= 2 * grain as usize {
            for i in 0..l {
                f(i);
            }
            return;
        }
        rayon::scope(|s| self.run_down(grain, &f, self.tops.clone(), s));
    }

    fn run_down<'s, F: Fn(usize) + Sync>(
        &'s self,
        grain: u32,
        f: &'s F,
        mut work: Vec<u32>,
        s: &rayon::Scope<'s>,
    ) {
        let mut pending: u64 = work.iter().map(|&i| u64::from(self.size[i as usize])).sum();
        while let Some(i) = work.pop() {
            let i = i as usize;
            f(i);
            let kids = &self.dpos[self.dptr[i] as usize..self.dptr[i + 1] as usize];
            // Heaviest child last, so it is the next one this worker runs.
            let mut heavy = usize::MAX;
            for (k, &c) in kids.iter().enumerate() {
                if heavy == usize::MAX || self.size[c as usize] > self.size[kids[heavy] as usize] {
                    heavy = k;
                }
            }
            for (k, &c) in kids.iter().enumerate() {
                if k != heavy {
                    work.push(c);
                }
            }
            if heavy != usize::MAX {
                work.push(kids[heavy]);
            }
            // The cell left the stack and its children entered: weight −1.
            pending -= 1;
            if pending >= 2 * u64::from(grain) && work.len() >= 2 {
                let mut take = 0;
                let mut w = 0u64;
                while take + 1 < work.len() && w < u64::from(grain) {
                    w += u64::from(self.size[work[take] as usize]);
                    take += 1;
                }
                let part: Vec<u32> = work.drain(..take).collect();
                pending -= w;
                s.spawn(move |s| self.run_down(grain, f, part, s));
            }
        }
    }

    /// Runs `f(i)` once for every position `i`, each after the `f` of all its donors. Leaves are
    /// handed out in chunks of `chunk`; a chain continues up from a leaf for as long as the walker
    /// is the last donor to arrive at the next cell.
    pub fn up<F: Fn(usize) + Sync>(&self, chunk: usize, f: F) {
        let l = self.len();
        if l == 0 {
            return;
        }
        if rayon::current_num_threads() <= 1 {
            for i in (0..l).rev() {
                f(i);
            }
            return;
        }
        self.leaves.par_chunks(chunk.max(1)).for_each(|leaves| {
            for &leaf in leaves {
                let mut i = leaf as usize;
                loop {
                    f(i);
                    let p = self.rpos[i];
                    if p == NONE {
                        break;
                    }
                    let p = p as usize;
                    if self.pend[p].fetch_sub(1, Ordering::AcqRel) != 1 {
                        break;
                    }
                    // Between passes the counter holds the donor count again.
                    self.pend[p].store(self.dptr[p + 1] - self.dptr[p], Ordering::Relaxed);
                    i = p;
                }
            }
        });
    }
}

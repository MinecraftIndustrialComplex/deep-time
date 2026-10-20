// SPDX-License-Identifier: LGPL-3.0-or-later
//! The planet's structure on a lattice (plan §8.5: "the evaluator runs at bake time on a lattice,
//! L2 columns × a vertical sample every few blocks").
//!
//! Two evaluations of the same thing:
//! - [`columns`]: the column rule (`deeptime_format::structure`) at every column of a grid. One
//!   call per column gives the whole column, so its cost does not depend on how finely the
//!   height is sampled.
//! - [`noddy_check`]: the Noddy port (`deeptime-noddy`) at every lattice point of a sample of
//!   columns, with the same structures as windowed Noddy events, compared with the column rule
//!   point by point. It is the reference, and its cost per point is what a lattice evaluation
//!   by the general engine would take.

use std::time::Instant;

use deeptime_format::GridDesc;
use deeptime_format::structure::{Segment, Shape, StructIndex, Structure};
use deeptime_noddy as noddy;
use rayon::prelude::*;
use serde::Serialize;

use super::feature::Feature;

/// What the column rule found over a grid of columns.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ColumnStats {
    pub columns: u64,
    /// Columns with at least one event that moves rock inside the height range.
    pub deformed: u64,
    /// Columns a fault plane crosses inside the height range.
    pub faulted: u64,
    /// Candidate events over all columns, and the most at one column.
    pub candidates: u64,
    pub max_candidates: u32,
    /// The most segments one column has inside the height range.
    pub max_segments: u32,
    /// The largest vertical and horizontal displacement of any segment, blocks.
    pub max_shift: f64,
    pub max_heave: f64,
    pub seconds: f64,
}

impl ColumnStats {
    fn merge(mut self, o: ColumnStats) -> ColumnStats {
        self.columns += o.columns;
        self.deformed += o.deformed;
        self.faulted += o.faulted;
        self.candidates += o.candidates;
        self.max_candidates = self.max_candidates.max(o.max_candidates);
        self.max_segments = self.max_segments.max(o.max_segments);
        self.max_shift = self.max_shift.max(o.max_shift);
        self.max_heave = self.max_heave.max(o.max_heave);
        self
    }
}

/// The part of `segs` inside [`min_y`, `max_y`].
fn inside(segs: &[Segment], min_y: f64, max_y: f64) -> impl Iterator<Item = &Segment> {
    segs.iter()
        .filter(move |s| s.y_hi > min_y && s.y_lo < max_y)
}

/// Runs the column rule at every column of `grid` (its cells' block positions) and gathers what
/// it finds between `min_y` and `max_y`.
pub fn columns(
    st: &Structure,
    index: &StructIndex,
    grid: &GridDesc,
    min_y: f64,
    max_y: f64,
) -> ColumnStats {
    let started = Instant::now();
    let c = index.circumference();
    let w = grid.w as usize;
    let mut stats = (0..grid.h as usize)
        .into_par_iter()
        .map_init(
            || (Vec::<u32>::new(), Vec::<Segment>::new()),
            |(cand, segs), j| {
                let mut s = ColumnStats::default();
                let z = grid.block_z(j as i64) as f64;
                for i in 0..w {
                    let x = grid.block_x(i as i64) as f64;
                    index.candidates(st, x, z, cand);
                    s.columns += 1;
                    s.candidates += cand.len() as u64;
                    s.max_candidates = s.max_candidates.max(cand.len() as u32);
                    if cand.is_empty() {
                        continue;
                    }
                    st.column(cand, x, z, c, segs);
                    let mut n = 0u32;
                    let mut moved = false;
                    for g in inside(segs, min_y, max_y) {
                        n += 1;
                        let heave = (g.x - x).hypot(g.z - z);
                        if g.dy != 0.0 || heave != 0.0 {
                            moved = true;
                        }
                        s.max_shift = s.max_shift.max(g.dy.abs());
                        s.max_heave = s.max_heave.max(heave);
                    }
                    s.max_segments = s.max_segments.max(n);
                    if moved {
                        s.deformed += 1;
                    }
                    if n > 1 {
                        s.faulted += 1;
                    }
                }
                s
            },
        )
        .reduce(ColumnStats::default, ColumnStats::merge);
    stats.seconds = started.elapsed().as_secs_f64();
    stats
}

/// The Noddy port against the column rule on a sample of columns.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct NoddyCheck {
    /// Columns sampled, and those of them with at least one candidate event.
    pub columns: u64,
    pub columns_with_events: u64,
    /// Lattice points compared.
    pub points: u64,
    /// The largest distance between the two restored positions, blocks, over the points where
    /// both put the point on the same side of every fault.
    pub worst: f64,
    /// Points the two put on different sides of a fault (within the caller's plane guard of it).
    pub side_flips: u64,
    /// The longest history (events) of a sampled column, and the columns whose history did not
    /// fit one Noddy history (127 events) and was undone as a chain of them.
    pub max_history: u64,
    pub long_histories: u64,
    /// Events undone per point by the Noddy port, on average.
    pub events_per_point: f64,
    /// Seconds in the Noddy port's `restore` and in the column rule over the same points (one
    /// thread each).
    pub seconds_noddy: f64,
    pub seconds_columns: f64,
}

impl NoddyCheck {
    pub fn ns_per_point_noddy(&self) -> f64 {
        self.seconds_noddy * 1.0e9 / self.points.max(1) as f64
    }

    pub fn ns_per_point_columns(&self) -> f64 {
        self.seconds_columns * 1.0e9 / self.points.max(1) as f64
    }
}

/// The plane guard of the stage's own check (`deform::report`, the run note the dense-structures
/// test reads): the agreement that test asks of a long history, 1e-4 of a block. ledger: CRE-034
pub const STAGE_PLANE_GUARD_BLOCKS: f64 = 1.0e-4;

/// Compares the two at the columns `sample` (block positions), at heights `min_y`, `min_y +
/// step`, … below `max_y`. `features` are the structures in the order of `st.events`. A point
/// within `plane_guard` blocks of a fault plane may fall on either side (a side flip, left out of
/// `worst`): each caller passes the agreement it asks, so the guard never excuses a disagreement
/// larger than the caller's own tolerance. ledger: CRE-034
#[allow(clippy::too_many_arguments)]
pub fn noddy_check(
    features: &[Feature],
    st: &Structure,
    index: &StructIndex,
    sample: &[(f64, f64)],
    min_y: f64,
    max_y: f64,
    step: f64,
    plane_guard: f64,
) -> NoddyCheck {
    assert_eq!(features.len(), st.events.len());
    let c = index.circumference();
    let profile = noddy::Profile::sine_table();
    let mut out = NoddyCheck::default();
    let mut cand = Vec::new();
    let mut segs = Vec::new();
    let mut undone = 0u64;
    let heights: Vec<f64> = {
        let mut v = Vec::new();
        let mut y = min_y;
        while y < max_y {
            v.push(y);
            y += step;
        }
        v
    };
    let mut restored = vec![[0.0f64; 3]; heights.len()];
    for &(x, z) in sample {
        out.columns += 1;
        index.candidates(st, x, z, &mut cand);
        if cand.is_empty() {
            continue;
        }
        out.columns_with_events += 1;
        // The history, oldest first, in a frame centred on the column. A chain, not one Noddy
        // history: a history holds 127 events (the original's sequence code) and a column of a
        // long-lived belt has more candidates than that (until 2026-10-02 such a column panicked
        // the run, and then was skipped). A chain undoes them in runs, to the same position, so
        // no sampled column is left out; `long_histories` counts the ones that needed it.
        let mut history = noddy::Chain::new();
        for &n in cand.iter().rev() {
            let y = match st.events[n as usize].shape {
                Shape::Fault { y, .. } => y,
                Shape::Fold { .. } | Shape::Intrusion { .. } => 0.0,
            };
            // (An intrusion moves nothing: it has no Noddy event here.)
            if let Some(e) = features[n as usize].to_noddy(x, z, c, y, &profile) {
                history.push(e);
            }
        }
        let t = Instant::now();
        for (k, &y) in heights.iter().enumerate() {
            restored[k] = history.restore([0.0, 0.0, y]);
        }
        out.seconds_noddy += t.elapsed().as_secs_f64();
        out.max_history = out.max_history.max(history.count() as u64);
        out.long_histories += u64::from(history.parts().len() > 1);
        undone += (cand.len() * heights.len()) as u64;

        let t = Instant::now();
        st.column(&cand, x, z, c, &mut segs);
        let mut k_seg = 0;
        for (k, &y) in heights.iter().enumerate() {
            while k_seg + 1 < segs.len() && y >= segs[k_seg].y_hi {
                k_seg += 1;
            }
            let g = &segs[k_seg];
            let mine = [g.x, y + g.dy, g.z];
            // Noddy (east, north, up) about the column → block (x, y, z).
            let theirs = [x + restored[k][0], restored[k][2], z - restored[k][1]];
            let d = ((mine[0] - theirs[0]).powi(2)
                + (mine[1] - theirs[1]).powi(2)
                + (mine[2] - theirs[2]).powi(2))
            .sqrt();
            // A point within rounding of a fault plane may fall on either side. "Within rounding"
            // is the caller's (`plane_guard`): the unit tests keep 1e-6, the stage's own check the
            // 1e-4 of a block its dense-structures test asks of a long history (on 2026-10-20 a
            // point 5.3e-6 of a block above a cut, in a column undoing 740 events, fell on the
            // other side in the Noddy port: a fault tip, 0.33 blocks of slip apart, scattered
            // continents seed 3, where 1e-6 had held).
            let near_plane = segs
                .iter()
                .any(|s| (y - s.y_lo).abs() < plane_guard || (y - s.y_hi).abs() < plane_guard);
            if d > 1e-3 && near_plane {
                out.side_flips += 1;
            } else {
                out.worst = out.worst.max(d);
            }
            out.points += 1;
        }
        out.seconds_columns += t.elapsed().as_secs_f64();
    }
    out.events_per_point = undone as f64 / out.points.max(1) as f64;
    out
}

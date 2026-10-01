// SPDX-License-Identifier: LGPL-3.0-or-later
//
// `profile_at` is Noddy's `calcProfile` (Loop3D/noddyverse, commit 6a21b23, `src/proflib.c`; MIT
// License, Copyright (c) 2021 Loop3D), as ported in `deeptime-noddy`. See NOTICE.md.
//! Folds and faults at one block column (plan §8.5, P2-01): the structural events a baked world
//! carries, and how they restore a column of the deformed world to columns of the undeformed one.
//!
//! The rock of a Deep Time world is stored undeformed, as horizon stacks ([`crate::strata`]). A
//! fold or a fault is an **event**: a rule that says where a point of the world as it is now was
//! before the event happened (inverse kinematics, after Noddy: Jessell & Valenta 1996). The rock
//! at a point is found by undoing the events that touch it, youngest first, and reading the stack
//! at the place the point came from.
//!
//! Every event here acts on a whole vertical column at once. Its strength at a place is a
//! [`Window`] weight that depends on the horizontal position only; a **fold** then shifts the
//! column up or down by its profile, and a **fault** moves the part of the column above its plane
//! by one vector. So a column of the deformed world is a few intervals of height, each a shifted
//! copy of part of an undeformed column: [`Structure::column`] returns them as [`Segment`]s.
//!
//! The arithmetic is IEEE-754 double `+ − × /`, comparisons and `floor`/`ceil` only, in the order
//! written, with every trigonometric quantity precomputed into the event and the fold profile a
//! stored table, so the Java reader can give the same answer bit for bit.
//!
//! These are `deeptime_noddy`'s windowed events (an upright sine fold and a translation fault
//! under a polynomial window) written out for a column; `deeptime-core`'s tests hold the two
//! together.

use serde::{Deserialize, Serialize};

/// 2π as Noddy writes it (seven decimals). Part of the contract: the phase of a fold is computed
/// with this value on both sides.
#[allow(clippy::approx_constant)]
pub const NODDY_TAU: f64 = 6.2831853;

/// Samples of a fold profile over one wavelength (Noddy's `X_PROFILE_RESOLUTION` + 1).
pub const PROFILE_SAMPLES: usize = 629;

/// A rectangle on the map with a smooth weight inside it, 1 at the centre and 0 at the edge.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Window {
    /// The centre, block (x, z).
    pub centre: [f64; 2],
    /// Unit vector along the structure (a fold's axis, a fault's strike), block (x, z).
    pub along: [f64; 2],
    /// Unit vector across it; for a fault, towards the hanging wall.
    pub across: [f64; 2],
    /// Half the length along the structure, blocks.
    pub half_along: f64,
    /// Half the width, blocks.
    pub half_across: f64,
}

/// `(1 − t²)²` for |t| < 1, 0 outside.
#[inline]
pub fn bump(t: f64) -> f64 {
    if t > -1.0 && t < 1.0 {
        let u = 1.0 - t * t;
        u * u
    } else {
        0.0
    }
}

impl Window {
    /// Coordinates of block position (x, z) along and across the window, x taken the short way
    /// round a planet of circumference `c` blocks.
    #[inline]
    pub fn local(&self, x: f64, z: f64, c: f64) -> (f64, f64) {
        let mut dx = x - self.centre[0];
        dx -= (dx / c + 0.5).floor() * c;
        let dz = z - self.centre[1];
        (
            dx * self.along[0] + dz * self.along[1],
            dx * self.across[0] + dz * self.across[1],
        )
    }

    /// The weight at local coordinates (`a` along, `b` across).
    #[inline]
    pub fn weight(&self, a: f64, b: f64) -> f64 {
        let wa = bump(a / self.half_along);
        if wa == 0.0 {
            return 0.0;
        }
        wa * bump(b / self.half_across)
    }

    /// Half the side of the square about the centre that holds every column of non-zero weight.
    pub fn reach(&self) -> f64 {
        (self.half_along * self.half_along + self.half_across * self.half_across).sqrt()
    }
}

/// What an event does to a column.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Shape {
    /// An upright fold train: the column moved up by
    /// `weight · amplitude · profile(2π · (b − offset) / wavelength)`, `b` the across coordinate.
    Fold {
        /// Blocks.
        wavelength: f64,
        /// Blocks (the crest's height above the undeformed level at the window's centre).
        amplitude: f64,
        /// The across coordinate of the profile's origin (its phase), blocks. The bake keeps it
        /// at or beyond the window's edge (`offset ≤ −half_across`), so the phase is never
        /// negative inside the window.
        offset: f64,
        /// Index into [`Structure::profiles`].
        profile: u8,
    },
    /// A fault: the rock on the pole's side of the plane (its hanging wall) moved by
    /// `weight · slip`.
    Fault {
        /// Height (block y) of the plane at the window's centre.
        y: f64,
        /// The plane's unit normal in block (x, y, z), pointing to the hanging wall; its y part
        /// is positive (a vertical fault is stored a hair off vertical, as Noddy does).
        pole: [f64; 3],
        /// The hanging wall's displacement at weight 1, block (x, y, z).
        slip: [f64; 3],
    },
}

/// One structural event.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StructEvent {
    pub id: u32,
    /// When it happened, years relative to world creation (negative: the past). Rock younger
    /// than this is not deformed by it.
    pub t_years: f64,
    pub window: Window,
    pub shape: Shape,
}

/// An interval of a deformed column and where its rock came from: a point at height y with
/// `y_lo ≤ y < y_hi` was, before every event, at `(x, y + dy, z)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub y_lo: f64,
    pub y_hi: f64,
    /// The source column, block coordinates (x not wrapped).
    pub x: f64,
    pub z: f64,
    /// Added to a height of the interval to get the source height.
    pub dy: f64,
    /// The youngest fault whose hanging wall holds the interval (its index in the events passed
    /// to [`Structure::column`]), or `u32::MAX`.
    pub block: u32,
}

/// `calcProfile`: the profile table read at `position` radians, linear between samples. The
/// table holds one wavelength in [`PROFILE_SAMPLES`] values of 1/100 (Noddy's units).
#[inline]
pub fn profile_at(table: &[f32], position: f64) -> f64 {
    const RES: i64 = 628;
    let (percent, mut i1, mut i2);
    if position < 0.0 {
        let d = (-position / NODDY_TAU) * 628.0;
        let f = d.floor();
        percent = d - f;
        i1 = RES - (f as i64) % RES;
        i2 = RES - (d.ceil() as i64) % RES;
    } else {
        let d = (position / NODDY_TAU) * 628.0;
        let f = d.floor();
        percent = d - f;
        i1 = (f as i64) % RES;
        i2 = (d.ceil() as i64) % RES;
    }
    i1 = i1.clamp(0, RES - 1);
    i2 = i2.clamp(0, RES - 1);
    let v1 = f64::from(table[i1 as usize]) / 100.0;
    let v2 = f64::from(table[i2 as usize]) / 100.0;
    v1 + percent * (v2 - v1)
}

/// A world's structural events.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Structure {
    /// Events, youngest first (the order they are undone in).
    pub events: Vec<StructEvent>,
    /// Fold profiles, [`PROFILE_SAMPLES`] values each.
    pub profiles: Vec<Vec<f32>>,
}

/// Blocks added to an event's reach when deciding which columns it is a candidate for: the
/// source of a segment may lie this far from its column (faults move rock sideways) and still
/// meet every event that could touch it.
pub const REACH_MARGIN: f64 = 64.0;

/// Side of the index's buckets, blocks.
const BUCKET: f64 = 128.0;

/// Which events to undo at a column. **The rule** (both sides of the contract apply it, so it is
/// part of the answer, not only a shortcut): event `e` is a candidate for the column at block
/// position (x, z) when |x − centre x|, taken the short way round the planet, and |z − centre z|
/// are both at most `reach(e) + REACH_MARGIN`; the candidates are undone in the order of
/// [`Structure::events`].
#[derive(Clone, Debug)]
pub struct StructIndex {
    c: f64,
    w: usize,
    h: usize,
    /// Per event: its reach plus the margin.
    radius: Vec<f64>,
    buckets: Vec<Vec<u32>>,
}

impl StructIndex {
    /// Indexes `st` on a planet `circumference` blocks around.
    pub fn new(st: &Structure, circumference: u32) -> StructIndex {
        let c = f64::from(circumference);
        let w = ((c / BUCKET).ceil() as usize).max(1);
        let h = ((c / 2.0 / BUCKET).ceil() as usize).max(1);
        let radius: Vec<f64> = st
            .events
            .iter()
            .map(|e| e.window.reach() + REACH_MARGIN)
            .collect();
        let mut buckets = vec![Vec::new(); w * h];
        for (n, e) in st.events.iter().enumerate() {
            let r = radius[n];
            let j0 = ((e.window.centre[1] - r + c / 4.0) / BUCKET).floor() as i64;
            let j1 = ((e.window.centre[1] + r + c / 4.0) / BUCKET).floor() as i64;
            let i0 = ((e.window.centre[0] - r + c / 2.0) / BUCKET).floor() as i64;
            let i1 = ((e.window.centre[0] + r + c / 2.0) / BUCKET).floor() as i64;
            let across = (i1 - i0 + 1).min(w as i64);
            for j in j0.max(0)..=j1.min(h as i64 - 1) {
                for di in 0..across {
                    let i = (i0 + di).rem_euclid(w as i64) as usize;
                    buckets[j as usize * w + i].push(n as u32);
                }
            }
        }
        StructIndex {
            c,
            w,
            h,
            radius,
            buckets,
        }
    }

    /// The planet's circumference, blocks.
    pub fn circumference(&self) -> f64 {
        self.c
    }

    /// The candidates of the column at block position (x, z), youngest first, into `out`.
    pub fn candidates(&self, st: &Structure, x: f64, z: f64, out: &mut Vec<u32>) {
        out.clear();
        if self.radius.is_empty() {
            return;
        }
        let i = (((x + self.c / 2.0) / BUCKET).floor() as i64).rem_euclid(self.w as i64) as usize;
        let j = (((z + self.c / 4.0) / BUCKET).floor() as i64).clamp(0, self.h as i64 - 1) as usize;
        for &n in &self.buckets[j * self.w + i] {
            let e = &st.events[n as usize];
            let mut dx = x - e.window.centre[0];
            dx -= (dx / self.c + 0.5).floor() * self.c;
            let dz = z - e.window.centre[1];
            let r = self.radius[n as usize];
            if dx >= -r && dx <= r && dz >= -r && dz <= r {
                out.push(n);
            }
        }
    }
}

impl Structure {
    /// Restores the column at block position (x, z) through `events` (indices into
    /// [`Structure::events`], ascending: youngest first) on a planet `c` blocks around: the
    /// segments are written to `out` bottom-up, covering every height once.
    pub fn column(&self, events: &[u32], x: f64, z: f64, c: f64, out: &mut Vec<Segment>) {
        out.clear();
        out.push(Segment {
            y_lo: f64::NEG_INFINITY,
            y_hi: f64::INFINITY,
            x,
            z,
            dy: 0.0,
            block: u32::MAX,
        });
        for (n, &e) in events.iter().enumerate() {
            let ev = &self.events[e as usize];
            let mut k = 0;
            while k < out.len() {
                let s = out[k];
                let (a, b) = ev.window.local(s.x, s.z, c);
                let w = ev.window.weight(a, b);
                if w == 0.0 {
                    k += 1;
                    continue;
                }
                match ev.shape {
                    Shape::Fold {
                        wavelength,
                        amplitude,
                        offset,
                        profile,
                    } => {
                        let phase = (b - offset) * NODDY_TAU / wavelength;
                        let p = profile_at(&self.profiles[usize::from(profile)], phase);
                        out[k].dy = s.dy - w * (amplitude * p);
                        k += 1;
                    }
                    Shape::Fault { y, pole, slip } => {
                        // The plane's height at the segment's source column, in source heights;
                        // the same position the window was read at.
                        let (dx, dz) = (
                            a * ev.window.along[0] + b * ev.window.across[0],
                            a * ev.window.along[1] + b * ev.window.across[1],
                        );
                        let plane = y - (dx * pole[0] + dz * pole[2]) / pole[1];
                        // In the column's own heights.
                        let cut = plane - s.dy;
                        if cut >= s.y_hi {
                            k += 1;
                            continue;
                        }
                        let moved = Segment {
                            y_lo: if cut > s.y_lo { cut } else { s.y_lo },
                            y_hi: s.y_hi,
                            x: s.x - w * slip[0],
                            z: s.z - w * slip[2],
                            dy: s.dy - w * slip[1],
                            block: if s.block == u32::MAX {
                                n as u32
                            } else {
                                s.block
                            },
                        };
                        if cut > s.y_lo {
                            out[k].y_hi = cut;
                            out.insert(k + 1, moved);
                            k += 2;
                        } else {
                            out[k] = moved;
                            k += 1;
                        }
                    }
                }
            }
        }
    }

    /// Where the point (x, y, z) was before every event of `events`: the source column and
    /// height (`column`, then the segment that holds y).
    pub fn restore(
        &self,
        events: &[u32],
        x: f64,
        y: f64,
        z: f64,
        c: f64,
        scratch: &mut Vec<Segment>,
    ) -> [f64; 3] {
        self.column(events, x, z, c, scratch);
        for s in scratch.iter() {
            if y >= s.y_lo && y < s.y_hi {
                return [s.x, y + s.dy, s.z];
            }
        }
        [x, y, z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine() -> Vec<f32> {
        (0..PROFILE_SAMPLES)
            .map(|i| (100.0 * (std::f64::consts::TAU * i as f64 / 628.0).sin()) as f32)
            .collect()
    }

    fn window() -> Window {
        // Long axis north-south (block −z is north), across pointing east.
        Window {
            centre: [100.0, 200.0],
            along: [0.0, -1.0],
            across: [1.0, 0.0],
            half_along: 400.0,
            half_across: 150.0,
        }
    }

    fn fold() -> StructEvent {
        StructEvent {
            id: 0,
            t_years: -1.0e8,
            window: window(),
            shape: Shape::Fold {
                wavelength: 100.0,
                amplitude: 12.0,
                offset: 0.0,
                profile: 0,
            },
        }
    }

    /// A normal fault through (100, 64, 200) striking north, dipping 60° east, 10 blocks of slip.
    fn fault() -> StructEvent {
        let (s, c) = (60.0f64.to_radians().sin(), 60.0f64.to_radians().cos());
        StructEvent {
            id: 1,
            t_years: -5.0e7,
            window: window(),
            shape: Shape::Fault {
                y: 64.0,
                pole: [s, c, 0.0],
                slip: [10.0 * c, -10.0 * s, 0.0],
            },
        }
    }

    #[test]
    fn a_fold_shifts_the_whole_column() {
        let st = Structure {
            events: vec![fold()],
            profiles: vec![sine()],
        };
        let mut segs = Vec::new();
        // A quarter wavelength east of the centre: the crest, the full amplitude.
        st.column(&[0], 125.0, 200.0, 4096.0, &mut segs);
        assert_eq!(segs.len(), 1);
        let w = bump(25.0 / 150.0);
        assert!((segs[0].dy + w * 12.0).abs() < 1e-3, "{}", segs[0].dy);
        assert_eq!((segs[0].x, segs[0].z), (125.0, 200.0));
        // Outside the window: untouched.
        st.column(&[0], 260.0, 200.0, 4096.0, &mut segs);
        assert_eq!(segs[0].dy, 0.0);
        st.column(&[0], 125.0, 700.0, 4096.0, &mut segs);
        assert_eq!(segs[0].dy, 0.0);
        // The trough on the other side.
        st.column(&[0], 75.0, 200.0, 4096.0, &mut segs);
        assert!((segs[0].dy - w * 12.0).abs() < 1e-3);
        // The same column one lap round the planet.
        let mut lap = Vec::new();
        st.column(&[0], 125.0 + 4096.0, 200.0, 4096.0, &mut lap);
        assert_eq!(lap[0].dy, {
            st.column(&[0], 125.0, 200.0, 4096.0, &mut segs);
            segs[0].dy
        });
    }

    #[test]
    fn a_fault_splits_the_column_at_its_plane() {
        let st = Structure {
            events: vec![fault()],
            profiles: vec![sine()],
        };
        let mut segs = Vec::new();
        // 20 blocks into the hanging wall: the plane is 20·tan 60° below the trace.
        st.column(&[0], 120.0, 200.0, 4096.0, &mut segs);
        assert_eq!(segs.len(), 2);
        let cut = 64.0 - 20.0 * 60.0f64.to_radians().tan();
        assert!((segs[0].y_hi - cut).abs() < 1e-9 && (segs[1].y_lo - cut).abs() < 1e-9);
        // Below the plane: the foot wall, in place.
        assert_eq!(
            (segs[0].x, segs[0].dy, segs[0].block),
            (120.0, 0.0, u32::MAX)
        );
        // Above: the hanging wall, restored up the dip (west and up).
        let w = bump(20.0 / 150.0);
        assert!(segs[1].x < 120.0 && segs[1].dy > 0.0 && segs[1].block == 0);
        assert!((segs[1].dy - w * 10.0 * 60.0f64.to_radians().sin()).abs() < 1e-9);
        // West of the trace the plane is above the trace's height; the hanging wall is the top.
        st.column(&[0], 90.0, 200.0, 4096.0, &mut segs);
        assert_eq!(segs.len(), 2);
        assert!((segs[0].y_hi - (64.0 + 10.0 * 60.0f64.to_radians().tan())).abs() < 1e-9);
        // Beyond the window on either side: nothing.
        st.column(&[0], 100.0 - 150.0, 200.0, 4096.0, &mut segs);
        assert_eq!(segs.len(), 1);
        st.column(&[0], 100.0 + 150.0, 200.0, 4096.0, &mut segs);
        assert_eq!(segs.len(), 1);
        // Past the fault's tip: one segment, nothing moved.
        st.column(&[0], 120.0, 700.0, 4096.0, &mut segs);
        assert_eq!(segs.len(), 1);
        // A point restores through the segment that holds it.
        let p = st.restore(&[0], 120.0, 70.0, 200.0, 4096.0, &mut segs);
        assert!(p[0] < 120.0 && p[1] > 70.0 && p[2] == 200.0);
        let p = st.restore(&[0], 120.0, -100.0, 200.0, 4096.0, &mut segs);
        assert_eq!(p, [120.0, -100.0, 200.0]);
    }

    #[test]
    fn events_are_undone_in_order() {
        // Folded, then faulted (the fault is the younger: index 0).
        let st = Structure {
            events: vec![fault(), fold()],
            profiles: vec![sine()],
        };
        let mut segs = Vec::new();
        st.column(&[0, 1], 120.0, 200.0, 4096.0, &mut segs);
        assert_eq!(segs.len(), 2);
        // The hanging wall reads the fold where it came from, the foot wall where it is.
        let mut alone = Vec::new();
        let only_fold = Structure {
            events: vec![fold()],
            profiles: vec![sine()],
        };
        only_fold.column(&[0], segs[1].x, segs[1].z, 4096.0, &mut alone);
        let fault_dy = bump(20.0 / 150.0) * 10.0 * 60.0f64.to_radians().sin();
        assert!((segs[1].dy - (fault_dy + alone[0].dy)).abs() < 1e-12);
        only_fold.column(&[0], 120.0, 200.0, 4096.0, &mut alone);
        assert_eq!(segs[0].dy, alone[0].dy);
        // The segments tile the column.
        assert_eq!(segs[0].y_lo, f64::NEG_INFINITY);
        assert_eq!(segs[0].y_hi, segs[1].y_lo);
        assert_eq!(segs[1].y_hi, f64::INFINITY);
    }

    #[test]
    fn the_profile_table_is_read_as_noddy_reads_it() {
        let t = sine();
        assert_eq!(t.len(), PROFILE_SAMPLES);
        for k in 0..2000 {
            let x = f64::from(k) * 0.0071 - 3.0;
            assert!((profile_at(&t, x) - x.sin()).abs() < 0.03, "{x}");
        }
        assert!((profile_at(&t, std::f64::consts::FRAC_PI_2) - 1.0).abs() < 1e-4);
    }
}

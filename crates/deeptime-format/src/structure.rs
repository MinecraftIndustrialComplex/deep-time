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
pub const NODDY_TAU: f64 = 6.2831853; // ledger: DEF-066

/// Samples of a fold profile over one wavelength (Noddy's `X_PROFILE_RESOLUTION` + 1).
pub const PROFILE_SAMPLES: usize = 629; // ledger: DEF-068

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
    // ledger: DEF-039
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
    /// An intrusion (a stock, a dyke, a volcanic plug): an ellipsoid of igneous rock with a
    /// vertical axis, its horizontal half-axes the window's `half_along` and `half_across`. It
    /// moves nothing: it replaces the rock older than itself inside it, so it cuts the folds and
    /// faults before it and is offset by the ones after.
    Intrusion {
        /// Height (block y) of its centre.
        y: f64,
        /// Its vertical half-axis, blocks.
        half_height: f64,
        /// The rock class it is made of (index into the world's classes).
        class: u8,
    },
}

/// One structural event.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StructEvent {
    pub id: u32,
    /// Its age as a stratigraphic key of the horizon stacks (`strata/horizons`): a horizon whose
    /// unit key is below this is rock laid down after the event, which the event did not deform.
    pub key: u16,
    /// When it happened, years relative to world creation (negative: the past).
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
    /// Blocks added to an event's reach when deciding which columns it is a candidate for
    /// ([`StructIndex`]): at least the furthest any fault chain moves rock sideways in this world
    /// (the bake measures it), so the source of every segment still meets every event that could
    /// touch it.
    pub margin: f64,
}

/// The least [`Structure::margin`] a bake writes, blocks.
pub const REACH_MARGIN: f64 = 64.0; // ledger: DEF-074

/// Side of the index's buckets, blocks.
const BUCKET: f64 = 128.0;

/// Which events to undo at a column. **The rule** (both sides of the contract apply it, so it is
/// part of the answer, not only a shortcut): event `e` is a candidate for the column at block
/// position (x, z) when |x − centre x|, taken the short way round the planet, and |z − centre z|
/// are both at most `reach(e) + margin` ([`Structure::margin`]); the candidates are undone in the order of
/// [`Structure::events`].
#[derive(Clone, Debug)]
pub struct StructIndex {
    c: f64,
    /// The buckets' origin, block (x, z).
    ox: f64,
    oz: f64,
    w: usize,
    h: usize,
    /// Per event: its reach plus the margin.
    radius: Vec<f64>,
    buckets: Vec<Vec<u32>>,
}

impl StructIndex {
    /// Indexes `st` on a Lambert planet `circumference` blocks around.
    pub fn new(st: &Structure, circumference: u32) -> StructIndex {
        StructIndex::new_in(
            st,
            circumference,
            crate::projection::IndexFrame::lambert(circumference),
        )
    }

    /// Indexes `st` on a world of x period `circumference` over `frame` (the world projection's
    /// index frame).
    pub fn new_in(
        st: &Structure,
        circumference: u32,
        frame: crate::projection::IndexFrame,
    ) -> StructIndex {
        let c = f64::from(circumference);
        let (ox, oz) = (frame.origin.0 as f64, frame.origin.1 as f64);
        let (span_x, span_z) = (frame.span.0 as f64, frame.span.1 as f64);
        let w = ((span_x / BUCKET).ceil() as usize).max(1);
        let h = ((span_z / BUCKET).ceil() as usize).max(1);
        let radius: Vec<f64> = st
            .events
            .iter()
            .map(|e| e.window.reach() + st.margin)
            .collect();
        let mut buckets = vec![Vec::new(); w * h];
        for (n, e) in st.events.iter().enumerate() {
            let r = radius[n];
            let j0 = ((e.window.centre[1] - r - oz) / BUCKET).floor() as i64;
            let j1 = ((e.window.centre[1] + r - oz) / BUCKET).floor() as i64;
            let i0 = ((e.window.centre[0] - r - ox) / BUCKET).floor() as i64;
            let i1 = ((e.window.centre[0] + r - ox) / BUCKET).floor() as i64;
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
            ox,
            oz,
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
        let (ox, oz) = (self.ox, self.oz);
        let i = (((x - ox) / BUCKET).floor() as i64).rem_euclid(self.w as i64) as usize;
        let j = (((z - oz) / BUCKET).floor() as i64).clamp(0, self.h as i64 - 1) as usize;
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
                    Shape::Intrusion { .. } => k += 1,
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

// ------------------------------------------------------------------------- the deformed stack

use crate::strata::ColumnHorizon;

/// One piece of a column while its events are being undone.
#[derive(Clone, Copy, Debug)]
struct Piece {
    /// Heights of the column the piece covers, [lo, hi).
    lo: f64,
    hi: f64,
    /// Where its rock is read: the source column (block position) and the height offset.
    sx: f64,
    sz: f64,
    dy: f64,
    /// The key of the oldest event that moved it: its rock is at least that old.
    key: u16,
    /// Rock younger than an event that would have moved it: no older event touches it.
    done: bool,
    /// Index of its source column's stack in the scratch.
    src: usize,
    /// An intrusion's rock (class, unit key): the piece is that rock, whatever its source says.
    rock: Option<(u8, u16)>,
}

/// Working memory of [`Structure::deformed_stack`], kept per thread by a reader of many columns.
#[derive(Default)]
pub struct DeformScratch {
    cands: Vec<u32>,
    pieces: Vec<Piece>,
    /// Stacks of the source columns met, by block column.
    stacks: Vec<((i64, i64), Vec<ColumnHorizon>)>,
    spare: Vec<Vec<ColumnHorizon>>,
}

impl DeformScratch {
    fn reset(&mut self) {
        for (_, v) in self.stacks.drain(..) {
            self.spare.push(v);
        }
        self.pieces.clear();
    }

    /// The scratch index of the stack of block column (x, z), read with `stack` when new.
    fn source(
        &mut self,
        x: i64,
        z: i64,
        stack: &mut dyn FnMut(i64, i64, &mut Vec<ColumnHorizon>),
    ) -> usize {
        if let Some(i) = self.stacks.iter().position(|(at, _)| *at == (x, z)) {
            return i;
        }
        let mut v = self.spare.pop().unwrap_or_default();
        stack(x, z, &mut v);
        self.stacks.push(((x, z), v));
        self.stacks.len() - 1
    }
}

impl Structure {
    /// **The deformed column rule** (FORMAT.md, "Structural events"): the horizon stack of block
    /// column (x, z) with the world's folds and faults, top-down like the undeformed rule's
    /// ([`crate::strata::stack_at`], which `stack(x, z, out)` must be).
    ///
    /// 1. The column's own stack S₀ and its candidate events ([`StructIndex`]). No candidates (or
    ///    an empty stack): S₀.
    /// 2. One piece covers every height, reading S₀ unshifted. The events are undone youngest
    ///    first; each one, for every piece not yet finished whose source column it has weight at:
    ///    - **Cover.** In the piece's source stack, the first horizon (top-down) whose unit key is
    ///      at least the event's key is the youngest rock the event deformed. None: the whole
    ///      piece is younger and is finished. Otherwise the part of the piece above that
    ///      horizon's top (source heights ≥ top + 1) is split off and finished: rock laid down
    ///      after the event is not moved by it, nor by anything older.
    ///    - **Fold:** the rest is shifted (`dy −= weight · amplitude · profile`).
    ///    - **Fault:** the rest is cut at the fault plane; the part above it now reads the column
    ///      `weight · slip` back along the slip, from the stack of the block column that holds
    ///      that position.
    ///    - **Intrusion:** the part of the rest inside the ellipsoid (read at the piece's source
    ///      position, so younger faults and folds have moved it) becomes the intrusion's rock and
    ///      is finished.
    ///
    ///    A moved piece takes the event's key.
    /// 3. The pieces are written out top-down. A piece holds the blocks y with lo ≤ y + ½ < hi
    ///    (the top piece up to S₀'s first top, the bottom one down to `min_y`); block y reads
    ///    source block y + ⌊dy + ½⌋ of the piece's source stack, whose first horizon reaches up
    ///    for ever; a horizon whose unit key is below the piece's key (rock younger than the
    ///    event that moved the piece: the shift has reached up into the cover) reads as the
    ///    lowest horizon of that cover (the last one going down whose key is still below the
    ///    piece's): the bed laid on the deformed surface fills its lows. Equal classes in a row
    ///    join.
    #[allow(clippy::too_many_arguments)]
    pub fn deformed_stack(
        &self,
        index: &StructIndex,
        x: i64,
        z: i64,
        min_y: i32,
        stack: &mut dyn FnMut(i64, i64, &mut Vec<ColumnHorizon>),
        scratch: &mut DeformScratch,
        out: &mut Vec<ColumnHorizon>,
    ) {
        out.clear();
        scratch.reset();
        let c = index.circumference();
        let (fx, fz) = (x as f64, z as f64);
        let mut cands = std::mem::take(&mut scratch.cands);
        index.candidates(self, fx, fz, &mut cands);
        let own = scratch.source(x, z, stack);
        if cands.is_empty() || scratch.stacks[own].1.is_empty() {
            out.extend_from_slice(&scratch.stacks[own].1);
            scratch.cands = cands;
            return;
        }
        scratch.pieces.push(Piece {
            lo: f64::NEG_INFINITY,
            hi: f64::INFINITY,
            sx: fx,
            sz: fz,
            dy: 0.0,
            key: 0,
            done: false,
            src: own,
            rock: None,
        });
        for &e in &cands {
            let ev = &self.events[e as usize];
            let mut k = 0;
            while k < scratch.pieces.len() {
                let p = scratch.pieces[k];
                if p.done {
                    k += 1;
                    continue;
                }
                let (a, b) = ev.window.local(p.sx, p.sz, c);
                let w = ev.window.weight(a, b);
                if w == 0.0 {
                    k += 1;
                    continue;
                }
                // Cover: what the event is too old to have moved.
                let cover = {
                    let s = &scratch.stacks[p.src].1;
                    s.iter().position(|hz| hz.2 >= ev.key).map(|h| (h, s[h].0))
                };
                let Some((h, top)) = cover else {
                    scratch.pieces[k].done = true;
                    k += 1;
                    continue;
                };
                let mut next = k + 1;
                if h > 0 {
                    let cut = f64::from(top + 1) - p.dy;
                    if cut <= p.lo {
                        scratch.pieces[k].done = true;
                        k += 1;
                        continue;
                    }
                    if cut < p.hi {
                        scratch.pieces[k].hi = cut;
                        scratch.pieces.insert(
                            k + 1,
                            Piece {
                                lo: cut,
                                done: true,
                                ..p
                            },
                        );
                        next += 1;
                    }
                }
                let p = scratch.pieces[k];
                match ev.shape {
                    Shape::Fold {
                        wavelength,
                        amplitude,
                        offset,
                        profile,
                    } => {
                        let phase = (b - offset) * NODDY_TAU / wavelength;
                        let v = profile_at(&self.profiles[usize::from(profile)], phase);
                        scratch.pieces[k].dy = p.dy - w * (amplitude * v);
                        scratch.pieces[k].key = ev.key;
                    }
                    Shape::Fault { y, pole, slip } => {
                        let (dx, dz) = (
                            a * ev.window.along[0] + b * ev.window.across[0],
                            a * ev.window.along[1] + b * ev.window.across[1],
                        );
                        let plane = y - (dx * pole[0] + dz * pole[2]) / pole[1];
                        let cut = plane - p.dy;
                        if cut < p.hi {
                            let (sx, sz) = (p.sx - w * slip[0], p.sz - w * slip[2]);
                            let src = scratch.source(sx.floor() as i64, sz.floor() as i64, stack);
                            let moved = Piece {
                                lo: if cut > p.lo { cut } else { p.lo },
                                hi: p.hi,
                                sx,
                                sz,
                                dy: p.dy - w * slip[1],
                                key: ev.key,
                                done: false,
                                src,
                                rock: None,
                            };
                            if cut > p.lo {
                                scratch.pieces[k].hi = cut;
                                scratch.pieces.insert(k + 1, moved);
                                next += 1;
                            } else {
                                scratch.pieces[k] = moved;
                            }
                        }
                    }
                    Shape::Intrusion {
                        y,
                        half_height,
                        class,
                    } => {
                        let (ua, ub) = (a / ev.window.half_along, b / ev.window.half_across);
                        let r2 = ua * ua + ub * ub;
                        if r2 < 1.0 {
                            let hh = half_height * (1.0 - r2).sqrt();
                            let lo = (y - hh) - p.dy;
                            let hi = (y + hh) - p.dy;
                            let lo = if lo > p.lo { lo } else { p.lo };
                            let hi = if hi < p.hi { hi } else { p.hi };
                            if lo < hi {
                                let body = Piece {
                                    lo,
                                    hi,
                                    done: true,
                                    rock: Some((class, ev.key)),
                                    ..p
                                };
                                let mut at = k;
                                if lo > p.lo {
                                    scratch.pieces[k].hi = lo;
                                    at = k + 1;
                                    scratch.pieces.insert(at, body);
                                    next += 1;
                                } else {
                                    scratch.pieces[k] = body;
                                }
                                if hi < p.hi {
                                    scratch.pieces.insert(at + 1, Piece { lo: hi, ..p });
                                    next += 1;
                                }
                            }
                        }
                    }
                }
                k = next;
            }
        }
        scratch.cands = cands;
        let top_limit = scratch.stacks[own].1[0].0;
        for p in scratch.pieces.iter().rev() {
            let y_max = if p.hi == f64::INFINITY {
                top_limit
            } else {
                top_limit.min((p.hi - 0.5).ceil() as i32 - 1)
            };
            let y_min = if p.lo == f64::NEG_INFINITY {
                min_y
            } else {
                min_y.max((p.lo - 0.5).ceil() as i32)
            };
            if y_max < y_min {
                continue;
            }
            if let Some((class, unit)) = p.rock {
                if out.last().is_none_or(|l| l.1 != class) {
                    out.push((y_max, class, unit));
                }
                continue;
            }
            let d = (p.dy + 0.5).floor() as i32;
            let s = &scratch.stacks[p.src].1;
            let n = s.len();
            for h in 0..n {
                let top = if h == 0 { y_max } else { y_max.min(s[h].0 - d) };
                let bottom = if h + 1 < n {
                    y_min.max(s[h + 1].0 + 1 - d)
                } else {
                    y_min
                };
                if top < bottom {
                    continue;
                }
                // Rock younger than the event that moved the piece: the shift has reached up
                // into the cover. The cover's lowest bed stands in (it fills the lows of the
                // surface it was laid on).
                let mut hh = h;
                if s[hh].2 < p.key {
                    while hh + 1 < n && s[hh + 1].2 < p.key {
                        hh += 1;
                    }
                }
                if out.last().is_none_or(|l| l.1 != s[hh].1) {
                    out.push((top, s[hh].1, s[hh].2));
                }
            }
        }
    }
}

/// A world's structure ready to deform columns: the events and their index.
pub struct Deformer {
    pub structure: Structure,
    pub index: StructIndex,
}

impl Deformer {
    pub fn new(structure: Structure, circumference: u32) -> Deformer {
        let index = StructIndex::new(&structure, circumference);
        Deformer { structure, index }
    }

    /// The deformer of a Lambert world-data folder; `None` when the world has no structural
    /// events.
    pub fn read(world: &Path, circumference: u32) -> Result<Option<Deformer>> {
        Deformer::read_in(
            world,
            circumference,
            crate::projection::IndexFrame::lambert(circumference),
        )
    }

    /// The deformer of a world-data folder whose spatial indexes use `frame` (the manifest's
    /// world projection); `None` when the world has no structural events.
    pub fn read_in(
        world: &Path,
        circumference: u32,
        frame: crate::projection::IndexFrame,
    ) -> Result<Option<Deformer>> {
        Ok(read_structure(world)?
            .filter(|s| !s.events.is_empty())
            .map(|s| {
                let index = StructIndex::new_in(&s, circumference, frame);
                Deformer {
                    structure: s,
                    index,
                }
            }))
    }

    /// [`Structure::deformed_stack`] at block column (x, z).
    pub fn stack(
        &self,
        x: i64,
        z: i64,
        min_y: i32,
        stack: &mut dyn FnMut(i64, i64, &mut Vec<ColumnHorizon>),
        scratch: &mut DeformScratch,
        out: &mut Vec<ColumnHorizon>,
    ) {
        self.structure
            .deformed_stack(&self.index, x, z, min_y, stack, scratch, out);
    }
}

// ------------------------------------------------------------------------------------- the file

use std::path::Path;

use crate::container::{self, f64_at, u16_at, u32_at};
use crate::error::{FormatError, Result};
use crate::fsutil::write_atomic;

/// The world's structural events, relative to the world-data folder.
pub const STRUCTURE_PATH: &str = "structure/events.dtst";
const MAGIC: [u8; 4] = *b"DTST";
const FIXED_BYTES: usize = 16;
/// Bytes of one event record.
pub const EVENT_BYTES: usize = 136;

/// Encodes `structure/events.dtst` (FORMAT.md, "Structural events").
pub fn encode_structure(st: &Structure) -> Result<Vec<u8>> {
    let mut records = Vec::with_capacity(st.events.len() * EVENT_BYTES);
    for e in &st.events {
        let start = records.len();
        records.extend_from_slice(&e.id.to_le_bytes());
        let (kind, profile, tail): (u8, u8, [f64; 7]) = match e.shape {
            Shape::Fold {
                wavelength,
                amplitude,
                offset,
                profile,
            } => (
                0,
                profile,
                [wavelength, amplitude, offset, 0.0, 0.0, 0.0, 0.0],
            ),
            Shape::Fault { y, pole, slip } => (
                1,
                0,
                [y, pole[0], pole[1], pole[2], slip[0], slip[1], slip[2]],
            ),
            Shape::Intrusion {
                y,
                half_height,
                class,
            } => (2, class, [y, half_height, 0.0, 0.0, 0.0, 0.0, 0.0]),
        };
        records.push(kind);
        records.push(profile);
        records.extend_from_slice(&e.key.to_le_bytes());
        let w = &e.window;
        for v in [
            e.t_years,
            w.centre[0],
            w.centre[1],
            w.along[0],
            w.along[1],
            w.across[0],
            w.across[1],
            w.half_along,
            w.half_across,
        ]
        .into_iter()
        .chain(tail)
        {
            records.extend_from_slice(&v.to_le_bytes());
        }
        debug_assert_eq!(records.len() - start, EVENT_BYTES);
    }
    let mut profiles = Vec::with_capacity(st.profiles.len() * PROFILE_SAMPLES * 4);
    for p in &st.profiles {
        if p.len() != PROFILE_SAMPLES {
            return Err(FormatError::Invalid(format!(
                "a fold profile has {} samples, not {PROFILE_SAMPLES}",
                p.len()
            )));
        }
        for v in p {
            profiles.extend_from_slice(&v.to_le_bytes());
        }
    }
    let mut fixed = Vec::with_capacity(FIXED_BYTES);
    fixed.extend_from_slice(&(st.events.len() as u32).to_le_bytes());
    fixed.extend_from_slice(&(st.profiles.len() as u32).to_le_bytes());
    fixed.extend_from_slice(&st.margin.to_le_bytes());
    container::encode(MAGIC, &fixed, &[&records, &profiles])
}

/// Decodes `structure/events.dtst`, checking what the column rule relies on.
pub fn decode_structure(path: &Path, b: &[u8]) -> Result<Structure> {
    let p = container::parse(path, b, MAGIC, FIXED_BYTES, |_| 2)?;
    let n = u32_at(p.fixed, 0) as usize;
    let np = u32_at(p.fixed, 4) as usize;
    let margin = f64_at(p.fixed, 8);
    let (records, profs) = (p.sections[0], p.sections[1]);
    let corrupt = |m: String| FormatError::corrupt(path, m);
    if records.len() != n * EVENT_BYTES || profs.len() != np * PROFILE_SAMPLES * 4 {
        return Err(corrupt(
            "event or profile section has the wrong length".into(),
        ));
    }
    if !(margin.is_finite() && margin >= 0.0) {
        return Err(corrupt(format!("margin {margin}")));
    }
    let profiles: Vec<Vec<f32>> = (0..np)
        .map(|k| {
            (0..PROFILE_SAMPLES)
                .map(|i| crate::container::f32_at(profs, (k * PROFILE_SAMPLES + i) * 4))
                .collect()
        })
        .collect();
    let mut events = Vec::with_capacity(n);
    for r in 0..n {
        let rec = &records[r * EVENT_BYTES..(r + 1) * EVENT_BYTES];
        let id = u32_at(rec, 0);
        let f = |k: usize| f64_at(rec, 8 + 8 * k);
        if (0..16).any(|k| !f(k).is_finite()) {
            return Err(corrupt(format!("event {id}: a value is not finite")));
        }
        let window = Window {
            centre: [f(1), f(2)],
            along: [f(3), f(4)],
            across: [f(5), f(6)],
            half_along: f(7),
            half_across: f(8),
        };
        if !(window.half_along > 0.0 && window.half_across > 0.0) {
            return Err(corrupt(format!("event {id}: an empty window")));
        }
        let shape = match rec[4] {
            0 => {
                let profile = rec[5];
                if usize::from(profile) >= np || f(9) == 0.0 {
                    return Err(corrupt(format!("event {id}: fold profile or wavelength")));
                }
                Shape::Fold {
                    wavelength: f(9),
                    amplitude: f(10),
                    offset: f(11),
                    profile,
                }
            }
            1 => {
                if f(11) <= 0.0 {
                    return Err(corrupt(format!("event {id}: the pole must point up")));
                }
                Shape::Fault {
                    y: f(9),
                    pole: [f(10), f(11), f(12)],
                    slip: [f(13), f(14), f(15)],
                }
            }
            2 => {
                if f(10) <= 0.0 {
                    return Err(corrupt(format!("event {id}: an intrusion of no height")));
                }
                Shape::Intrusion {
                    y: f(9),
                    half_height: f(10),
                    class: rec[5],
                }
            }
            k => return Err(corrupt(format!("event {id}: kind {k}"))),
        };
        events.push(StructEvent {
            id,
            key: u16_at(rec, 6),
            t_years: f(0),
            window,
            shape,
        });
    }
    if events.windows(2).any(|w| w[0].key > w[1].key) {
        return Err(corrupt("events are not youngest first".into()));
    }
    Ok(Structure {
        events,
        profiles,
        margin,
    })
}

/// Writes `structure/events.dtst` under `world` (atomically); returns (bytes, CRC32).
pub fn write_structure(world: &Path, st: &Structure) -> Result<(u64, u32)> {
    let bytes = encode_structure(st)?;
    let crc = write_atomic(&world.join(STRUCTURE_PATH), &bytes)?;
    Ok((bytes.len() as u64, crc))
}

/// Reads `structure/events.dtst`; `None` when the folder has none.
pub fn read_structure(world: &Path) -> Result<Option<Structure>> {
    let path = world.join(STRUCTURE_PATH);
    match std::fs::read(&path) {
        Ok(b) => Ok(Some(decode_structure(&path, &b)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(FormatError::io(&path, e)),
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
            key: 9000,
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
            key: 7000,
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
            margin: REACH_MARGIN,
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
            margin: REACH_MARGIN,
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
            margin: REACH_MARGIN,
        };
        let mut segs = Vec::new();
        st.column(&[0, 1], 120.0, 200.0, 4096.0, &mut segs);
        assert_eq!(segs.len(), 2);
        // The hanging wall reads the fold where it came from, the foot wall where it is.
        let mut alone = Vec::new();
        let only_fold = Structure {
            events: vec![fold()],
            profiles: vec![sine()],
            margin: REACH_MARGIN,
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

    /// Soil over young cover over three older beds, the same in every column; `step` blocks
    /// lower for every eight blocks eastwards when it is not zero.
    fn cake(step: i32) -> impl FnMut(i64, i64, &mut Vec<ColumnHorizon>) {
        move |x, _z, out| {
            let d = step * (x as i32).div_euclid(8);
            out.clear();
            out.extend_from_slice(&[
                (100 - d, 0, 100),
                (90 - d, 1, 5000),
                (60 - d, 2, 9000),
                (30 - d, 3, 20000),
                (-d, 4, 40000),
            ]);
        }
    }

    fn deformed(st: &Structure, x: i64, z: i64, step: i32) -> Vec<ColumnHorizon> {
        let index = StructIndex::new(st, 4096);
        let mut out = Vec::new();
        st.deformed_stack(
            &index,
            x,
            z,
            -64,
            &mut cake(step),
            &mut DeformScratch::default(),
            &mut out,
        );
        out
    }

    fn world(events: Vec<StructEvent>) -> Structure {
        Structure {
            events,
            profiles: vec![sine()],
            margin: REACH_MARGIN,
        }
    }

    #[test]
    fn a_column_no_event_touches_is_the_plain_stack() {
        let st = world(vec![fold(), fault()]);
        let mut plain = Vec::new();
        cake(0)(900, 200, &mut plain);
        assert_eq!(deformed(&st, 900, 200, 0), plain);
        assert_eq!(deformed(&world(Vec::new()), 125, 200, 0), plain);
    }

    #[test]
    fn a_fold_bends_the_rock_older_than_itself_under_flat_cover() {
        // The fold's key (9000) is the third bed's: soil and cover (keys 100, 5000) are younger.
        let st = world(vec![fold()]);
        let w = bump(25.0 / 150.0);
        let lift = (w * 12.0 + 0.5).floor() as i32;
        // A crest (a quarter wavelength east of the centre): the old beds are up by the
        // amplitude, the cover and its base where they were (an angular unconformity).
        assert_eq!(
            deformed(&st, 125, 200, 0),
            [
                (100, 0, 100),
                (90, 1, 5000),
                (60, 2, 9000),
                (30 + lift, 3, 20000),
                (lift, 4, 40000)
            ]
        );
        // A trough: down by as much; the cover's lowest bed fills what the shift left open.
        assert_eq!(
            deformed(&st, 75, 200, 0),
            [
                (100, 0, 100),
                (90, 1, 5000),
                (60 - lift, 2, 9000),
                (30 - lift, 3, 20000),
                (-lift, 4, 40000)
            ]
        );
        // A fold younger than the cover bends the cover too, but never the soil.
        let mut young = fold();
        young.key = 4500;
        let col = deformed(&world(vec![young]), 125, 200, 0);
        assert_eq!(col[0], (100, 0, 100));
        assert_eq!(col[1], (90, 1, 5000));
        assert_eq!(col[2].0, 60 + lift);
    }

    #[test]
    fn a_normal_fault_drops_its_hanging_wall_and_a_thrust_repeats_the_section() {
        // The fault of `fault()` (key 7000: younger than the three old beds, older than the
        // cover), 80 blocks into its hanging wall, where the plane is under the world's floor.
        let st = world(vec![fault()]);
        let col = deformed(&st, 180, 200, 0);
        let w = bump(80.0 / 150.0);
        let drop = (w * 10.0 * 60.0f64.to_radians().sin() + 0.5).floor() as i32;
        // The cover's beds where they were, its lowest thicker by the throw; the hanging
        // wall's beds lower by the throw than the foot wall's would be; below the plane the
        // foot wall in place.
        assert_eq!(
            &col[..3],
            &[(100, 0, 100), (90, 1, 5000), (60 - drop, 2, 9000)]
        );
        assert_eq!(col[3], (30 - drop, 3, 20000), "{col:?}");
        assert_eq!(col.last().unwrap(), &(-drop, 4, 40000));
        // Nearer the trace the plane cuts the column: hanging wall over foot wall, the bed
        // under the plane where the foot wall has it.
        let col = deformed(&st, 120, 200, 0);
        let near = (bump(20.0 / 150.0) * 10.0 * 60.0f64.to_radians().sin() + 0.5).floor() as i32;
        assert_eq!(col[2], (60 - near, 2, 9000));
        assert_eq!(col[3], (28, 3, 20000), "{col:?}");
        assert_eq!(col.last().unwrap(), &(0, 4, 40000));
        // A thrust (the slip reversed) on beds that dip: the hanging wall carries older rock
        // over younger, so a bed is met twice going down.
        let mut thrust = fault();
        if let Shape::Fault { slip, .. } = &mut thrust.shape {
            *slip = [-slip[0] * 4.0, -slip[1] * 4.0, 0.0];
        }
        thrust.key = 4200;
        let col = deformed(&world(vec![thrust]), 120, 200, 0);
        let classes: Vec<u8> = col.iter().map(|h| h.1).collect();
        let twice = (1..5u8).any(|k| classes.iter().filter(|&&c| c == k).count() > 1);
        assert!(twice, "a repeated bed in {col:?}");
        // Tops strictly descend and no two neighbours share a class, whatever the events.
        for step in [0, 1] {
            for x in [60, 90, 101, 120, 160, 240] {
                let col = deformed(&world(vec![fault(), fold()]), x, 150, step);
                assert!(
                    col.windows(2).all(|w| w[0].0 > w[1].0 && w[0].1 != w[1].1),
                    "{col:?}"
                );
                let mut plain = Vec::new();
                cake(step)(x, 150, &mut plain);
                assert_eq!(col[0], plain[0], "the first top is the column's own");
            }
        }
    }

    /// A stock about (100, 40, 200): 60 by 40 blocks across, 30 blocks of half-height.
    fn stock(key: u16) -> StructEvent {
        StructEvent {
            id: 2,
            key,
            t_years: -6.0e7,
            window: Window {
                half_along: 60.0,
                half_across: 40.0,
                ..window()
            },
            shape: Shape::Intrusion {
                y: 40.0,
                half_height: 30.0,
                class: 9,
            },
        }
    }

    #[test]
    fn an_intrusion_cuts_older_rock_and_is_cut_by_younger_structures() {
        // Younger than the third bed (key 9000), older than the cover (5000): its ellipsoid
        // holds y 10 to 70 at the centre, and it rises through the third bed to the base of
        // the cover at y 60, which was laid down on it.
        let st = world(vec![stock(8000)]);
        assert_eq!(
            deformed(&st, 100, 200, 0),
            [
                (100, 0, 100),
                (90, 1, 5000),
                (60, 9, 8000),
                (9, 3, 20000),
                (0, 4, 40000)
            ]
        );
        // Off centre the ellipsoid is thinner; outside it the plain stack.
        let off = deformed(&st, 100 + 30, 200, 0);
        let hh = 30.0 * (1.0 - 0.75f64 * 0.75).sqrt();
        assert_eq!(off[2], (60, 2, 9000));
        assert_eq!(
            off[3],
            ((40.0 + hh - 0.5).ceil() as i32 - 1, 9, 8000),
            "{off:?}"
        );
        let mut plain = Vec::new();
        cake(0)(100, 200, &mut plain);
        assert_eq!(deformed(&st, 100 + 41, 200, 0), plain);
        assert_eq!(deformed(&st, 100, 200 + 61, 0), plain);
        // Older than every bed but the last: it lies wholly inside the oldest rock.
        let deep = deformed(&world(vec![stock(30000)]), 100, 200, 0);
        assert_eq!(
            deep,
            [
                (100, 0, 100),
                (90, 1, 5000),
                (60, 2, 9000),
                (30, 3, 20000),
                (0, 4, 40000)
            ],
            "an intrusion older than the rock at its level leaves that rock alone"
        );
        // A younger fold lifts the stock with the beds (its top stays cut off at the base of
        // the cover; its floor rises); an older fold does not move it.
        let w = bump(25.0 / 150.0);
        let lift = (w * 12.0 + 0.5).floor() as i32;
        let mut young_fold = fold();
        young_fold.key = 7000;
        // (top of the stock, top of the rock under it)
        let body = |col: &[ColumnHorizon]| {
            let i = col.iter().position(|h| h.1 == 9).expect("the stock");
            (col[i].0, col[i + 1].0)
        };
        let alone = body(&deformed(&world(vec![stock(8000)]), 125, 200, 0));
        let lifted = body(&deformed(
            &world(vec![young_fold, stock(8000)]),
            125,
            200,
            0,
        ));
        assert_eq!(lifted.0, 60, "cut off under the cover");
        assert!(
            (lifted.1 - (alone.1 + lift)).abs() <= 1,
            "{lifted:?} {alone:?}"
        );
        let cut = body(&deformed(&world(vec![stock(8000), fold()]), 125, 200, 0));
        assert_eq!(cut, alone);
        // Whatever the order: tops descend, neighbours differ.
        for events in [
            vec![fault(), stock(8000), fold()],
            vec![stock(6000), fault(), fold()],
        ] {
            for x in [70, 95, 100, 120, 135] {
                let col = deformed(&world(events.clone()), x, 190, 1);
                assert!(
                    col.windows(2).all(|w| w[0].0 > w[1].0 && w[0].1 != w[1].1),
                    "{col:?}"
                );
            }
        }
    }

    /// Whatever the events and wherever the column: the stack is well formed (tops descend,
    /// neighbours differ, the first horizon is the column's own, nothing outside the world's
    /// height), every rock in it comes from a stack or an intrusion, and a column outside every
    /// window is the plain stack.
    #[test]
    fn any_history_gives_well_formed_columns() {
        let mut seed = 0x9e37_79b9_7f4a_7c15u64;
        let mut rnd = move || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        let mut total = 0usize;
        for world_n in 0..40 {
            let n = 1 + (rnd() * 12.0) as usize;
            let mut events: Vec<StructEvent> = (0..n)
                .map(|id| {
                    let az = rnd() * std::f64::consts::TAU;
                    let window = Window {
                        centre: [rnd() * 600.0 - 300.0, rnd() * 600.0 - 300.0],
                        along: [az.sin(), -az.cos()],
                        across: [az.cos(), az.sin()],
                        half_along: 40.0 + rnd() * 300.0,
                        half_across: 20.0 + rnd() * 150.0,
                    };
                    let shape = match (rnd() * 3.0) as u32 {
                        0 => Shape::Fold {
                            wavelength: 20.0 + rnd() * 100.0,
                            amplitude: rnd() * 30.0,
                            offset: -400.0,
                            profile: 0,
                        },
                        1 => {
                            let dip = (10.0 + rnd() * 79.0f64).to_radians();
                            let slip = (rnd() - 0.5) * 60.0;
                            Shape::Fault {
                                y: rnd() * 120.0 - 20.0,
                                pole: [
                                    dip.sin() * window.across[0],
                                    dip.cos(),
                                    dip.sin() * window.across[1],
                                ],
                                slip: [
                                    slip * dip.cos() * window.across[0],
                                    -slip * dip.sin(),
                                    slip * dip.cos() * window.across[1],
                                ],
                            }
                        }
                        _ => Shape::Intrusion {
                            y: rnd() * 140.0 - 40.0,
                            half_height: 5.0 + rnd() * 80.0,
                            class: 9,
                        },
                    };
                    StructEvent {
                        id: id as u32,
                        key: (rnd() * 45000.0) as u16,
                        t_years: 0.0,
                        window,
                        shape,
                    }
                })
                .collect();
            events.sort_by_key(|e| e.key);
            let st = world(events);
            let step: i32 = world_n % 3;
            let mut touched = 0;
            for _ in 0..200 {
                let (x, z) = (
                    (rnd() * 1400.0 - 700.0) as i64,
                    (rnd() * 1400.0 - 700.0) as i64,
                );
                let col = deformed(&st, x, z, step);
                let mut plain = Vec::new();
                cake(step)(x, z, &mut plain);
                assert!(
                    col.windows(2).all(|w| w[0].0 > w[1].0 && w[0].1 != w[1].1),
                    "{col:?}"
                );
                assert_eq!(col[0], plain[0], "the youngest rock is the column's own");
                assert!(col.iter().all(|h| h.1 <= 9), "{col:?}");
                // (The rule writes nothing under the world's floor.)
                plain.retain(|h| h.0 >= -64);
                let outside = st.events.iter().all(|e| {
                    let (a, b) = e.window.local(x as f64, z as f64, 4096.0);
                    e.window.weight(a, b) == 0.0
                });
                let mut col = col;
                col.retain(|h| h.0 >= -64);
                if outside {
                    assert_eq!(col, plain);
                }
                touched += usize::from(col != plain);
            }
            total += touched;
        }
        assert!(total > 200, "only {total} of 8,000 columns changed");
    }

    #[test]
    fn the_structure_file_round_trips() {
        let mut st = world(vec![fault(), stock(8000), fold()]);
        st.margin = 109.0;
        let bytes = encode_structure(&st).unwrap();
        assert_eq!(
            decode_structure(Path::new("events.dtst"), &bytes).unwrap(),
            st
        );
        // Events out of order, a pole pointing down and a truncated file are refused.
        let mut bad = st.clone();
        bad.events.swap(0, 1);
        let b = encode_structure(&bad).unwrap();
        assert!(decode_structure(Path::new("e"), &b).is_err());
        let mut bad = st.clone();
        if let Shape::Fault { pole, .. } = &mut bad.events[0].shape {
            pole[1] = -0.5;
        }
        let b = encode_structure(&bad).unwrap();
        assert!(decode_structure(Path::new("e"), &b).is_err());
        assert!(decode_structure(Path::new("e"), &bytes[..bytes.len() - 9]).is_err());
        let dir = std::env::temp_dir().join(format!("dt-structure-{}", std::process::id()));
        write_structure(&dir, &st).unwrap();
        assert_eq!(read_structure(&dir).unwrap().unwrap(), st);
        assert!(Deformer::read(&dir, 4096).unwrap().is_some());
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(read_structure(&dir).unwrap().is_none());
    }
}

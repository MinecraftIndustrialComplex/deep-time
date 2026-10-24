// SPDX-License-Identifier: LGPL-3.0-or-later
//! Braided reaches (V1 gap 5, stage 4; docs/science/rivers-and-lakes.md, "Fans and braids"): a
//! river divides around alluvial islands where it is steep or builds a fan.
//!
//! "A braided river is one which flows in two or more anastomosing channels around alluvial
//! islands. ... braided reaches taken as a whole are steeper, wider, and shallower than undivided
//! reaches carrying the same flow" (Leopold & Wolman 1957, p. 53). Here, after the meanders:
//!
//! - **Where** (HYD-171). A reach braids on an alluvial fan ([`super::fan`]; "the main channel
//!   ... divides into several braided distributary channels", Bull 1964, p. 93), and on a
//!   floodplain where its slope over a bar's length is above Leopold & Wolman's line between
//!   meandering and braided channels, s = 0.0125 Q^−0.44 (HYD-158: the reaches the meanders leave).
//! - **Threads** (HYD-172). Around each island the channel is two threads, each 0.9 of the
//!   undivided width and 0.75 of its depth: the divided channels' widths sum to 1.6 to 2.0 times
//!   the undivided one's and their depth is 0.6 to 0.9 of it (Leopold & Wolman 1957, p. 50). Each
//!   carries half the flow (HYD-175).
//! - **Islands** (HYD-173, HYD-174). An island is a lens 5 channel widths long (Horse Creek's
//!   central bar, "nearly 250 feet" in a channel of 50, LW1957 p. 41) and one width across at its
//!   middle; the islands follow one another down the reach on alternate sides of the course, so
//!   the river weaves between them. A channel narrower than 3 blocks does not braid: its threads
//!   and island would merge in the world's blocks.
//! - **Format.** The river keeps its course as one thread (narrowed and shallowed around each
//!   island) and each island's other thread is a chain of its own that leaves the course at a
//!   vertex (`from`) and rejoins it at another (`into`, mouth `River`): no format change. The
//!   water surface along a thread is the course's at the same distance down it, lowered by as much
//!   as a fan's ground under the thread lies below the fan's ground under the course (a cone falls
//!   away to either side of a course down its axis), and never rising.
//! - **Room.** A thread is drawn only where its bank lies on the floodplain (at or over its water)
//!   or the fan, and no other channel's bank zone lies within twice its reach across the course;
//!   the island goes to the other side when one side has no room. Distances are the planet's
//!   (round-features.md, SCL-230).

use std::time::Instant;

use deeptime_format::Projection;
use deeptime_format::rivers::{Chain, Mouth, NO_CHAIN, RiverVertex, Rivers};
use rayon::prelude::*;
use serde_json::{Value, json};

use super::meander::{BRAID_SLOPE_COEF, BRAID_SLOPE_EXP};

/// Each of two threads' width over the undivided channel's: the divided channels' widths sum to
/// 1.6 to 2.0 times the undivided one's in Leopold & Wolman's (1957, p. 50) natural rivers.
pub const THREAD_WIDTH_SHARE: f64 = 0.9; // ledger: HYD-172
/// A thread's depth over the undivided channel's: 0.6 to 0.9 in the same rivers (p. 50).
pub const THREAD_DEPTH_SHARE: f64 = 0.75; // ledger: HYD-172
/// An island's length in undivided channel widths: Horse Creek's central bar, "nearly 250 feet
/// down the center of the channel" of 50 feet (Leopold & Wolman 1957, p. 41).
pub const BAR_LENGTH_PER_WIDTH: f64 = 5.0; // ledger: HYD-173
/// An island's width at its middle in undivided channel widths (game-choice: agent, 2026-10-24;
/// the sources read give none, only that the island "is considerably wider than the new gravel
/// bar", LW1957 p. 44).
pub const ISLAND_WIDTH_PER_WIDTH: f64 = 1.0; // ledger: HYD-174 (game-choice)
/// The narrowest channel that braids, blocks (game-choice: agent, 2026-10-24; narrower threads and
/// islands merge in the world's blocks).
pub const MIN_WIDTH_BLOCKS: f64 = 3.0; // ledger: HYD-174 (game-choice)
/// The share of the flow each thread carries (game-choice: agent, 2026-10-24; Leopold & Wolman's
/// one measured split, the Green River's 305 and 120 of 425 ft³/s, is no rule).
pub const THREAD_Q_SHARE: f64 = 0.5; // ledger: HYD-175 (game-choice)

/// The course's sample spacing, and the room probe's across it, blocks of planet.
const STEP: f64 = 1.0;
/// The least cos φ the Lambert metric is taken at (the pole rows).
const COS_MIN: f64 = 0.05;
/// How far short of a distance down the course a vertex may lie and still count as at it, blocks
/// of planet (f32 vertex positions on a Lambert frame land a few ulps off).
const SNAP: f64 = 0.01;
/// Lower bounds of the discharge classes in the record, m³/s.
const Q_CLASSES: [f64; 4] = [0.0, 10.0, 100.0, 1000.0];

/// What the braids read.
pub struct Input<'a> {
    /// The floodplain's level at block (x, z), 1/16 block, None off the plain.
    pub plain: &'a (dyn Fn(f64, f64) -> Option<i32> + Sync),
    /// The ground of an alluvial fan at block (x, z), blocks; None off the fans.
    pub fan: &'a (dyn Fn(f64, f64) -> Option<f64> + Sync),
    /// The planet whose Lambert blocks the network is in; None for a frame drawn true to shape.
    pub lambert: Option<Projection>,
    /// Metres per block across and up.
    pub m_per_block_h: f64,
    pub m_per_block_v: f64,
}

/// The lowest plain level (1/16 block) whose top block is not under a water surface w.
fn flush_level(w: f64) -> i32 {
    ((w * 16.0).round() as i32 + 15).div_euclid(16) * 16 - 15
}

fn q_class(q: f64) -> usize {
    Q_CLASSES.iter().take_while(|&&e| q >= e).count().max(1) - 1
}

/// One point of a course.
#[derive(Clone, Copy)]
struct Pt {
    x: f64,
    z: f64,
    w: f64,
    hw: f64,
    depth: f64,
    q: f64,
}

/// One island: the chain's vertex it starts at, how far each vertex inside it is along the island
/// (0 to 1), and the new thread's vertices.
struct Island {
    a: usize,
    along: Vec<f64>,
    thread: Vec<RiverVertex>,
}

/// The island's lens across at `f` of its length: 0 at its ends, 1 at its middle.
fn lens(f: f64) -> f64 {
    (std::f64::consts::PI * f.clamp(0.0, 1.0)).sin()
}

#[derive(Default, Clone)]
struct Tally {
    length: f64,
    fan: f64,
    steep: f64,
    braided: f64,
    braided_fan: f64,
    islands: u64,
    no_room: u64,
    narrow: f64,
    by_q: [[f64; 2]; 4],
}

impl Tally {
    fn add(&mut self, o: &Tally) {
        self.length += o.length;
        self.fan += o.fan;
        self.steep += o.steep;
        self.braided += o.braided;
        self.braided_fan += o.braided_fan;
        self.islands += o.islands;
        self.no_room += o.no_room;
        self.narrow += o.narrow;
        for (a, b) in self.by_q.iter_mut().zip(o.by_q) {
            a[0] += b[0];
            a[1] += b[1];
        }
    }
}

struct Net<'a> {
    r: &'a Rivers,
    inp: &'a Input<'a>,
    c: f64,
    chain_of: Vec<u32>,
}

impl Net<'_> {
    /// Blocks of planet per block east and south at block z.
    fn scales(&self, z: f64) -> (f64, f64) {
        match &self.inp.lambert {
            Some(p) => {
                let c = p.cos_lat(z).max(COS_MIN);
                (c, 1.0 / c)
            }
            None => (1.0, 1.0),
        }
    }

    fn dx(&self, ax: f64, bx: f64) -> f64 {
        let d = bx - ax;
        d - self.c * (d / self.c).round()
    }

    fn dist(&self, ax: f64, az: f64, bx: f64, bz: f64) -> f64 {
        let (ex, ez) = self.scales(0.5 * (az + bz));
        (self.dx(ax, bx) * ex).hypot((bz - az) * ez)
    }

    /// Ground a thread may run on at (x, z) with water surface w: the floodplain at or over the
    /// water, or a fan.
    fn room(&self, x: f64, z: f64, w: f64) -> bool {
        (self.inp.fan)(x, z).is_some()
            || (self.inp.plain)(x, z).is_some_and(|lv| lv >= flush_level(w))
    }

    /// Whether (x, z), `t` blocks across the course of chain `own` (vertices from `first`, their
    /// distances down it `u`) from its point at distance `s`, lies in the channel or bank zone of
    /// another chain, or of the course itself where it comes back (a segment further along it
    /// than the probe is across it plus its reach; grid distance, as the index is built).
    #[allow(clippy::too_many_arguments)]
    fn blocked(&self, own: u32, first: usize, u: &[f64], s: f64, t: f64, x: f64, z: f64) -> bool {
        let r = self.r;
        let (xi, zi) = (x.round() as i64, z.round() as i64);
        let ci = i64::from(r.c);
        let wx = (xi + ci / 2).rem_euclid(ci) - ci / 2;
        for &seg in r.tile_entries(r.tile_of(wx, zi)) {
            let su = seg as usize;
            let k = self.chain_of[su];
            if k == u32::MAX || self.chain_of.get(su + 1) != Some(&k) {
                continue;
            }
            let (a, b) = (&r.vertices[su], &r.vertices[su + 1]);
            let reach = r.reach_of(f64::from(a.hw.max(b.hw)));
            if k == own {
                let i = su - first;
                let (u0, u1) = (u[i], u[i + 1]);
                if (0.5 * (u0 + u1) - s).abs() <= t + reach + 0.5 * (u1 - u0) {
                    continue;
                }
            }
            let (ax, az) = (f64::from(a.x), f64::from(a.z));
            let px = ax + self.dx(ax, x);
            let (ex, ez) = (f64::from(b.x) - ax, f64::from(b.z) - az);
            let l2 = ex * ex + ez * ez;
            let f = if l2 > 0.0 {
                (((px - ax) * ex + (z - az) * ez) / l2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let d = (px - ax - f * ex).hypot(z - az - f * ez);
            if d <= reach {
                return true;
            }
        }
        false
    }

    /// The course of `v` at planet distance s along it.
    fn at(v: &[RiverVertex], u: &[f64], s: f64) -> Pt {
        let m = v.len();
        let i = u.partition_point(|&x| x <= s).clamp(1, m - 1) - 1;
        let len = u[i + 1] - u[i];
        let t = if len > 0.0 {
            ((s - u[i]) / len).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (a, b) = (&v[i], &v[i + 1]);
        let l = |p: f32, q: f32| f64::from(p) + (f64::from(q) - f64::from(p)) * t;
        Pt {
            x: l(a.x, b.x),
            z: l(a.z, b.z),
            w: l(a.w, b.w),
            hw: l(a.hw, b.hw),
            depth: l(a.depth, b.depth),
            q: l(a.q, b.q),
        }
    }

    /// The islands of chain k and its record.
    fn plan(&self, k: usize) -> (Vec<Island>, Tally) {
        let ch = self.r.chains[k];
        let first = ch.first as usize;
        let v = &self.r.vertices[first..first + ch.count as usize];
        let m = v.len();
        let mut tally = Tally::default();
        let mut u = vec![0.0; m];
        for i in 1..m {
            u[i] = u[i - 1]
                + self.dist(
                    f64::from(v[i - 1].x),
                    f64::from(v[i - 1].z),
                    f64::from(v[i].x),
                    f64::from(v[i].z),
                );
        }
        let total = u[m - 1];
        tally.length = total;
        if ch.from != NO_CHAIN || m < 4 || total < 4.0 * STEP {
            return (Vec::new(), tally);
        }
        let n = (total / STEP).floor() as usize + 1;
        let pts: Vec<Pt> = (0..n)
            .map(|j| Self::at(v, &u, j as f64 * STEP))
            .collect();
        let (mv, mh) = (self.inp.m_per_block_v, self.inp.m_per_block_h);
        // Samples that braid: on a fan, or on the plain and steeper than the line over a bar.
        let mut cand = vec![false; n];
        for j in 0..n {
            let p = &pts[j];
            let on_fan = (self.inp.fan)(p.x, p.z).is_some();
            let qc = q_class(p.q);
            if on_fan {
                tally.fan += STEP;
            }
            let braids = on_fan || {
                let half = ((BAR_LENGTH_PER_WIDTH * p.hw).max(2.0) / STEP).round() as usize;
                let (a, b) = (j.saturating_sub(half), (j + half).min(n - 1));
                let slope =
                    (pts[a].w - pts[b].w).max(0.0) * mv / ((b - a).max(1) as f64 * STEP * mh);
                let line = if p.q > 0.0 {
                    BRAID_SLOPE_COEF * p.q.powf(BRAID_SLOPE_EXP)
                } else {
                    f64::INFINITY
                };
                let steep = slope > line && self.room(p.x, p.z, p.w);
                if steep {
                    tally.steep += STEP;
                }
                steep
            };
            if !braids {
                continue;
            }
            tally.by_q[qc][0] += STEP;
            if 2.0 * p.hw < MIN_WIDTH_BLOCKS {
                tally.narrow += STEP;
                continue;
            }
            cand[j] = true;
        }
        // The islands down each run of samples that braid, on alternate sides.
        let own = k as u32;
        let mut islands = Vec::new();
        let mut side = if k.is_multiple_of(2) { 1.0 } else { -1.0 };
        let mut j = 0;
        while j < n {
            if !cand[j] {
                j += 1;
                continue;
            }
            let r0 = j;
            while j < n && cand[j] {
                j += 1;
            }
            let r1 = j - 1;
            let mut s = r0 as f64 * STEP;
            let s_end = r1 as f64 * STEP;
            loop {
                let p = Self::at(v, &u, s);
                let len = BAR_LENGTH_PER_WIDTH * 2.0 * p.hw;
                if s + len > s_end {
                    break;
                }
                // The island's span, on the course's vertices.
                let a = u.partition_point(|&x| x < s - SNAP);
                let b = u.partition_point(|&x| x < u[a.min(m - 1)] + len - SNAP);
                if a < 1 || b + 1 >= m || b < a + 2 || u[b] > s_end + STEP {
                    s += 0.5 * len;
                    continue;
                }
                let (ua, ub) = (u[a], u[b]);
                let island = [side, -side]
                    .into_iter()
                    .find_map(|sg| self.thread(own, first, v, &u, a, b, ua, ub, sg).map(|t| (sg, t)));
                match island {
                    Some((sg, thread)) => {
                        tally.islands += 1;
                        tally.braided += ub - ua;
                        if (self.inp.fan)(p.x, p.z).is_some() {
                            tally.braided_fan += ub - ua;
                        }
                        tally.by_q[q_class(p.q)][1] += ub - ua;
                        let along = (a + 1..b).map(|i| (u[i] - ua) / (ub - ua)).collect();
                        islands.push(Island {
                            a,
                            along,
                            thread,
                        });
                        side = -sg;
                        s = ub;
                    }
                    None => {
                        tally.no_room += 1;
                        s += 0.5 * len;
                    }
                }
            }
        }
        (islands, tally)
    }

    /// The other thread around an island over the course's vertices [a, b] on side `sg`, None
    /// where it has no room.
    #[allow(clippy::too_many_arguments)]
    fn thread(
        &self,
        own: u32,
        first: usize,
        v: &[RiverVertex],
        u: &[f64],
        a: usize,
        b: usize,
        ua: f64,
        ub: f64,
        sg: f64,
    ) -> Option<Vec<RiverVertex>> {
        let len = ub - ua;
        let span = |s: f64| lens((s - ua) / len);
        let tangent = |s: f64| -> (f64, f64) {
            let h = (0.25 * len).max(STEP);
            let (p0, p1) = (
                Self::at(v, u, (s - h).max(ua)),
                Self::at(v, u, (s + h).min(ub)),
            );
            let (ex, ez) = self.scales(0.5 * (p0.z + p1.z));
            let (te, ts) = (self.dx(p0.x, p1.x) * ex, (p1.z - p0.z) * ez);
            let l = te.hypot(ts).max(1e-9);
            (te / l, ts / l)
        };
        let steps = (len / STEP).ceil().max(2.0) as usize;
        let mut out: Vec<RiverVertex> = Vec::with_capacity(steps + 1);
        let mut prev_dir: Option<(f64, f64)> = None;
        for i in 0..=steps {
            let s = ua + len * i as f64 / steps as f64;
            let p = Self::at(v, u, s);
            let hw0 = p.hw;
            let f = span(s);
            let hw_t = hw0 * (1.0 - (1.0 - THREAD_WIDTH_SHARE) * f);
            let island = ISLAND_WIDTH_PER_WIDTH * 2.0 * hw0;
            let off = (2.0 * THREAD_WIDTH_SHARE * hw0 + island) * f;
            let (te, ts) = tangent(s);
            let (ne, ns) = (-ts * sg, te * sg);
            let (ex, ez) = self.scales(p.z);
            let at = |t: f64| (p.x + t * ne / ex, p.z + t * ns / ez);
            // The bank beyond the thread on the ground, and nothing else within twice the reach.
            if i > 0 && i < steps {
                let reach = off + hw_t + self.r.reach_of(hw_t) - hw_t;
                let mut t = hw0;
                while t <= 2.0 * reach + STEP {
                    let (x, z) = at(t);
                    if t <= reach && !self.room(x, z, p.w) {
                        return None;
                    }
                    if self.blocked(own, first, u, s, t, x, z) {
                        return None;
                    }
                    t += STEP;
                }
            }
            let (x, z) = if i == 0 {
                (f64::from(v[a].x), f64::from(v[a].z))
            } else if i == steps {
                (f64::from(v[b].x), f64::from(v[b].z))
            } else {
                at(off)
            };
            // No turning back on itself (an island inside a tight bend).
            if let Some(&last) = out.last() {
                let (ex, ez) = self.scales(z);
                let d = (self.dx(f64::from(last.x), x) * ex, (z - f64::from(last.z)) * ez);
                if let Some(pd) = prev_dir
                    && d.0 * pd.0 + d.1 * pd.1 < 0.0
                {
                    return None;
                }
                if d.0.hypot(d.1) > 1e-6 {
                    prev_dir = Some(d);
                }
            }
            let x = if i == 0 || i == steps {
                x
            } else {
                f64::from(v[a].x) + self.dx(f64::from(v[a].x), x)
            };
            // On a fan the thread's water stands as far under its ground as the course's under
            // the course's ground: the cone falls away to either side of a course down its axis.
            let lower = match ((self.inp.fan)(p.x, p.z), (self.inp.fan)(x, z)) {
                (Some(gc), Some(gt)) => (gc - gt).max(0.0),
                _ => 0.0,
            };
            out.push(RiverVertex {
                x: x as f32,
                z: z as f32,
                w: if i == 0 {
                    v[a].w
                } else if i == steps {
                    v[b].w
                } else {
                    (p.w - lower) as f32
                },
                hw: hw_t as f32,
                depth: (p.depth * (1.0 - (1.0 - THREAD_DEPTH_SHARE) * f)) as f32,
                q: (p.q * THREAD_Q_SHARE) as f32,
            });
        }
        // The surface never rises down the thread.
        for i in 1..out.len() {
            if out[i].w > out[i - 1].w {
                out[i].w = out[i - 1].w;
            }
        }
        Some(out)
    }
}

/// Braids `rivers` in place (the chains keep their ids and order, the new threads follow them;
/// the index is rebuilt) and returns `stats.braids`.
pub fn apply(rivers: &mut Rivers, inp: &Input<'_>) -> Value {
    let t0 = Instant::now();
    let k_chains = rivers.chains.len();
    let mut chain_of = vec![u32::MAX; rivers.vertices.len()];
    for (k, ch) in rivers.chains.iter().enumerate() {
        for i in ch.first..ch.first + ch.count {
            chain_of[i as usize] = k as u32;
        }
    }
    let net = Net {
        r: &*rivers,
        inp,
        c: f64::from(rivers.c),
        chain_of,
    };
    let plans: Vec<(Vec<Island>, Tally)> = (0..k_chains)
        .into_par_iter()
        .map(|k| net.plan(k))
        .collect();
    drop(net);
    let mut total = Tally::default();
    let mut threads = 0u64;
    for (k, (islands, tally)) in plans.into_iter().enumerate() {
        total.add(&tally);
        let first = rivers.chains[k].first as usize;
        for isl in islands {
            // The course narrows and shallows around the island, and carries the other share of
            // the flow.
            for (i, &al) in isl.along.iter().enumerate() {
                let f = lens(al);
                let vtx = &mut rivers.vertices[first + isl.a + 1 + i];
                vtx.hw = (f64::from(vtx.hw) * (1.0 - (1.0 - THREAD_WIDTH_SHARE) * f)) as f32;
                vtx.depth = (f64::from(vtx.depth) * (1.0 - (1.0 - THREAD_DEPTH_SHARE) * f)) as f32;
                vtx.q = (f64::from(vtx.q) * (1.0 - THREAD_Q_SHARE)) as f32;
            }
            let start = rivers.vertices.len() as u32;
            let count = isl.thread.len() as u32;
            rivers.vertices.extend(isl.thread);
            let mut chain = Chain::new(start, count, Mouth::River, k as u32);
            chain.from = k as u32;
            rivers.chains.push(chain);
            threads += 1;
        }
    }
    rivers.build_index();
    let km = inp.m_per_block_h / 1000.0;
    let r1 = |x: f64| (x * 10.0).round() / 10.0;
    let by_q: Vec<Value> = Q_CLASSES
        .iter()
        .zip(total.by_q)
        .map(|(&q, b)| {
            json!({
                "q_min_m3s": q,
                "braiding_km": r1(b[0] * km),
                "braided_km": r1(b[1] * km),
            })
        })
        .collect();
    json!({
        "threads_added": threads,
        "islands": total.islands,
        "no_room": total.no_room,
        "network_km": r1(total.length * km),
        "on_fans_km": r1(total.fan * km),
        "steep_on_plain_km": r1(total.steep * km),
        "too_narrow_km": r1(total.narrow * km),
        "braided_km": r1(total.braided * km),
        "braided_on_fans_km": r1(total.braided_fan * km),
        "by_q": by_q,
        "millis": t0.elapsed().as_millis() as u64,
    })
}

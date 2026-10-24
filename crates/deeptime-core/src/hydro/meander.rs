// SPDX-License-Identifier: LGPL-3.0-or-later
//! Meanders (V1 gap 5, stage 3; docs/science/rivers-and-lakes.md, "Meanders"): a river on a
//! floodplain winds across it.
//!
//! The network's chains follow the drainage grid's cells, smoothed: straight down every valley.
//! Here each reach of a chain that lies on the floodplain ([`super::network::Floodplains`]), is not
//! steep enough to braid, and has room is redrawn as a sine-generated curve about its course:
//!
//! - **Shape** (Langbein & Leopold 1966, HYD-156). The channel's direction off the valley's is
//!   φ = ω sin(2π s / M) along its own length s, M = k λ for a downvalley wavelength λ; the
//!   sinuosity is k = 1 / J₀(ω), and the half amplitude across the valley a = λ H₀(ω) / (4 J₀(ω))
//!   (J₀ Bessel's and H₀ Struve's function of order 0). The curve has no overhangs while ω < π/2.
//! - **Wavelength** (Leopold & Wolman 1960, HYD-155): λ = 10.9 w, w the channel's drawn width
//!   (2 hw). The amplitude is what is free: the paper finds it "determined more by erosion
//!   characteristics of stream banks and by other local factors" than by the width, so here the
//!   plain's room decides it, up to the amplitude of sinuosity [`MAX_SINUOSITY`] (HYD-157).
//! - **Where.** A sample of the course meanders where its channel is on the plain with the water
//!   no higher than the plain's top block (the plain is laid at the bank, HYD-151), where the
//!   course's slope over a wavelength is under Leopold & Wolman's (1957) line between meandering
//!   and braided channels, s = 0.0125 Q^−0.44 (HYD-158; Q stands for the bankfull discharge), and
//!   away from both ends of its chain and from a distributary's start. Across the course the
//!   plain's room is probed to its edge (or half way to another river's bank zone); the bounds
//!   are eroded over a wavelength and averaged over half of one, and the belt is placed inside
//!   them, as close to the old course as the room allows. Reaches at least a wavelength long with
//!   at least [`MIN_AMPLITUDE_BLOCKS`] of amplitude are drawn (HYD-159), tapered to the old course
//!   over half a wavelength at each end.
//! - **The planet's metric.** Everything is measured on the planet, not in the grid's blocks: on
//!   a Lambert frame a block east is cos φ of the planet's and a block south 1 / cos φ, so a
//!   meander belt is as wide and as long at 55° as at the equator (round-features.md, SCL-230).
//! - **Joins.** A tributary whose parent was redrawn under its mouth is cut where it first meets
//!   the parent's new channel (or carried on to it, when the parent swung away), and ends on a
//!   vertex of the new channel. Distributaries are left as they are.
//!
//! The water surface along a redrawn reach is the old course's at the same downvalley distance,
//! so it never rises downstream, and the plain beside the new channel stands where it was laid:
//! the old course is plain like the rest. No format change: the chains get more vertices.

use std::f64::consts::{FRAC_PI_2, PI, TAU};
use std::sync::OnceLock;
use std::time::Instant;

use deeptime_format::Projection;
use deeptime_format::rivers::{Mouth, NO_CHAIN, RiverVertex, Rivers};
use rayon::prelude::*;
use serde_json::{Value, json};

/// Meander wavelength per channel width (Leopold & Wolman 1960, p. 774: L = 10.9 w^1.01 ft, and
/// at unit exponents the paper's own ratio 10.9).
pub const WAVELENGTH_PER_WIDTH: f64 = 10.9; // ledger: HYD-155
/// The most sinuous a reach is drawn: Langbein & Leopold (1966) read Leopold & Wolman's (1960)
/// rivers as "dominantly between 1.1 and 2.0"; past it a bend is cut off (LW1960, p. 773).
pub const MAX_SINUOSITY: f64 = 2.0; // ledger: HYD-157
/// Leopold & Wolman's (1957) line between meandering and braided channels, s = 0.06 Q^−0.44 with Q
/// in ft³/s (eq. 1, p. 60); in m³/s the coefficient is 0.06 · 35.315^−0.44 = 0.0125.
pub const BRAID_SLOPE_COEF: f64 = 0.0125; // ledger: HYD-158
/// Its exponent.
pub const BRAID_SLOPE_EXP: f64 = -0.44; // ledger: HYD-158
/// The least amplitude drawn, blocks: a block is the world's least visible bend (game-choice:
/// agent, 2026-10-24).
pub const MIN_AMPLITUDE_BLOCKS: f64 = 1.0; // ledger: HYD-159 (game-choice)

/// The course's sample spacing, blocks of planet.
const STEP: f64 = 1.0;
/// The room probe's spacing across the course, blocks of planet.
const PROBE: f64 = 1.0;
/// The least cos φ the Lambert metric is taken at (the pole rows).
const COS_MIN: f64 = 0.05;
/// The shape table: ω rows from 0 to the cap, phases per wavelength, and integration steps.
const OMEGA_STEPS: usize = 64;
const PHASE_STEPS: usize = 256;
const T_STEPS: usize = 2048;
/// Simpson intervals for J₀ and H₀.
const SIMPSON: usize = 512;
/// The redrawn channel's vertex spacing is this share of its wavelength, within [1, 6] blocks.
const PATH_STEP_PER_WAVELENGTH: f64 = 1.0 / 40.0;
/// Lower bounds of the discharge classes in the record, m³/s.
const Q_CLASSES: [f64; 4] = [0.0, 10.0, 100.0, 1000.0];
/// Upper edges of the reach sinuosity histogram (the last bin is everything past 1.5).
const SINUOSITY_EDGES: [f64; 5] = [1.05, 1.1, 1.3, 1.5, 2.0];

fn simpson(f: impl Fn(f64) -> f64, a: f64, b: f64) -> f64 {
    let n = SIMPSON;
    let h = (b - a) / n as f64;
    let mut s = f(a) + f(b);
    for i in 1..n {
        s += f(a + i as f64 * h) * if i % 2 == 1 { 4.0 } else { 2.0 };
    }
    s * h / 3.0
}

/// Bessel's J₀(x) = (1/π) ∫₀^π cos(x sin t) dt.
pub fn bessel_j0(x: f64) -> f64 {
    simpson(|t| (x * t.sin()).cos(), 0.0, PI) / PI
}

/// Struve's H₀(x) = (2/π) ∫₀^{π/2} sin(x sin t) dt.
pub fn struve_h0(x: f64) -> f64 {
    2.0 / PI * simpson(|t| (x * t.sin()).sin(), 0.0, FRAC_PI_2)
}

/// The sine-generated curve's sinuosity for maximum angle ω (radians): 1 / J₀(ω) (HYD-156).
pub fn sinuosity(omega: f64) -> f64 {
    1.0 / bessel_j0(omega)
}

/// Its half amplitude across the valley per downvalley wavelength: H₀(ω) / (4 J₀(ω)).
pub fn amplitude_ratio(omega: f64) -> f64 {
    struve_h0(omega) / (4.0 * bessel_j0(omega))
}

fn bisect(f: impl Fn(f64) -> f64, mut lo: f64, mut hi: f64) -> f64 {
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if f(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// The maximum angle of sinuosity k ≥ 1 (below J₀'s first zero, 2.405).
pub fn omega_of_sinuosity(k: f64) -> f64 {
    bisect(|w| sinuosity(w) - k, 0.0, 2.4)
}

/// The curve's cross-valley offset over its half amplitude at each downvalley phase, per ω.
struct Shape {
    omega_max: f64,
    /// [`amplitude_ratio`] per row, ascending.
    ratio: Vec<f64>,
    /// Ŷ per (row, phase): 0 at phase 0 (a crossing, the channel at its steepest to the valley),
    /// 1 at π/2 (a bend's apex).
    y: Vec<f32>,
}

impl Shape {
    fn build() -> Shape {
        let omega_max = omega_of_sinuosity(MAX_SINUOSITY);
        let mut ratio = Vec::with_capacity(OMEGA_STEPS + 1);
        let mut y = Vec::with_capacity((OMEGA_STEPS + 1) * PHASE_STEPS);
        for i in 0..=OMEGA_STEPS {
            let w = omega_max * i as f64 / OMEGA_STEPS as f64;
            ratio.push(if i == 0 { 0.0 } else { amplitude_ratio(w) });
            y.extend(Shape::row(w));
        }
        Shape {
            omega_max,
            ratio,
            y,
        }
    }

    /// One row: the curve x(t), y(t) for direction φ = ω cos t over t ∈ [0, 2π], x scaled to a
    /// phase in [0, 2π] and y to its maximum, read at even phases.
    fn row(omega: f64) -> Vec<f32> {
        let phase = |p: usize| TAU * p as f64 / PHASE_STEPS as f64;
        if omega <= 0.0 {
            return (0..PHASE_STEPS).map(|p| phase(p).sin() as f32).collect();
        }
        let dt = TAU / T_STEPS as f64;
        let (mut xs, mut ys) = (vec![0.0; T_STEPS + 1], vec![0.0; T_STEPS + 1]);
        for s in 0..T_STEPS {
            let (a, b) = (
                omega * (s as f64 * dt).cos(),
                omega * ((s + 1) as f64 * dt).cos(),
            );
            xs[s + 1] = xs[s] + 0.5 * dt * (a.cos() + b.cos());
            ys[s + 1] = ys[s] + 0.5 * dt * (a.sin() + b.sin());
        }
        let xn = TAU / xs[T_STEPS];
        let ymax = ys.iter().fold(0.0f64, |m, v| m.max(v.abs()));
        let mut out = Vec::with_capacity(PHASE_STEPS);
        let mut s = 0;
        for p in 0..PHASE_STEPS {
            let th = phase(p);
            while s + 1 < T_STEPS && xs[s + 1] * xn < th {
                s += 1;
            }
            let (x0, x1) = (xs[s] * xn, xs[s + 1] * xn);
            let f = if x1 > x0 {
                ((th - x0) / (x1 - x0)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            out.push(((ys[s] + f * (ys[s + 1] - ys[s])) / ymax) as f32);
        }
        out
    }

    /// The largest amplitude ratio drawn.
    fn ratio_max(&self) -> f64 {
        self.ratio[OMEGA_STEPS]
    }

    /// Ŷ at phase θ for maximum angle ω (bilinear in the table).
    fn y_hat(&self, theta: f64, omega: f64) -> f64 {
        let f = (omega / self.omega_max).clamp(0.0, 1.0) * OMEGA_STEPS as f64;
        let i0 = (f.floor() as usize).min(OMEGA_STEPS - 1);
        let fi = f - i0 as f64;
        let g = theta.rem_euclid(TAU) / TAU * PHASE_STEPS as f64;
        let p0 = (g.floor() as usize) % PHASE_STEPS;
        let p1 = (p0 + 1) % PHASE_STEPS;
        let fp = g - g.floor();
        let at = |i: usize, p: usize| f64::from(self.y[i * PHASE_STEPS + p]);
        let a = at(i0, p0) + fp * (at(i0, p1) - at(i0, p0));
        let b = at(i0 + 1, p0) + fp * (at(i0 + 1, p1) - at(i0 + 1, p0));
        a + fi * (b - a)
    }

    /// The maximum angle whose amplitude ratio is r (linear between rows).
    fn omega_of_ratio(&self, r: f64) -> f64 {
        if r <= 0.0 {
            return 0.0;
        }
        if r >= self.ratio_max() {
            return self.omega_max;
        }
        let i = self
            .ratio
            .partition_point(|&q| q <= r)
            .clamp(1, OMEGA_STEPS);
        let (r0, r1) = (self.ratio[i - 1], self.ratio[i]);
        let f = if r1 > r0 { (r - r0) / (r1 - r0) } else { 0.0 };
        self.omega_max * ((i - 1) as f64 + f) / OMEGA_STEPS as f64
    }
}

fn shape() -> &'static Shape {
    static SHAPE: OnceLock<Shape> = OnceLock::new();
    SHAPE.get_or_init(Shape::build)
}

/// The meander's sinuosity for half amplitude `a` per wavelength (the table's inverse).
pub fn sinuosity_of_ratio(r: f64) -> f64 {
    sinuosity(shape().omega_of_ratio(r))
}

/// The cross-valley offset over the half amplitude at phase θ for amplitude ratio r.
pub fn y_hat(theta: f64, r: f64) -> f64 {
    let s = shape();
    s.y_hat(theta, s.omega_of_ratio(r))
}

/// The amplitude ratio at [`MAX_SINUOSITY`].
pub fn ratio_cap() -> f64 {
    shape().ratio_max()
}

/// What the meanders read.
pub struct Input<'a> {
    /// The floodplain's level at block (x, z), 1/16 block (the layer's raw value), None off the
    /// plain ([`super::network::Output::floodplain`]).
    pub plain: &'a (dyn Fn(f64, f64) -> Option<i32> + Sync),
    /// The planet whose Lambert blocks the network is in; None for a frame drawn true to shape
    /// (the metric is then the blocks').
    pub lambert: Option<Projection>,
    /// Metres per block across and up.
    pub m_per_block_h: f64,
    pub m_per_block_v: f64,
}

/// The lowest plain level (1/16 block) whose top block is not under a water surface w: the
/// water's top block is ⌈w⌉ − 1, so the plain's ⌈H⌉ − 1 ≥ ⌈w⌉ − 1.
fn flush_level(w: f64) -> i32 {
    ((w * 16.0).round() as i32 + 15).div_euclid(16) * 16 - 15
}

fn q_class(q: f64) -> usize {
    Q_CLASSES.iter().take_while(|&&e| q >= e).count().max(1) - 1
}

/// A tributary's mouth: where it joins its parent and how far from there the parent's belt may
/// pass over it.
#[derive(Clone, Copy)]
struct Join {
    x: f64,
    z: f64,
    radius: f64,
}

/// The record of one chain, blocks of planet.
#[derive(Clone, Default)]
struct Tally {
    length: f64,
    plain: f64,
    steep: f64,
    narrow: f64,
    short: f64,
    reaches: u64,
    reach_axis: f64,
    reach_path: f64,
    /// Amplitude times axis length.
    amp_len: f64,
    max_offset: f64,
    hist: [u64; 6],
    /// Per discharge class: plain, steep, reach axis, reach path.
    by_q: [[f64; 4]; 4],
}

impl Tally {
    fn add(&mut self, o: &Tally) {
        self.length += o.length;
        self.plain += o.plain;
        self.steep += o.steep;
        self.narrow += o.narrow;
        self.short += o.short;
        self.reaches += o.reaches;
        self.reach_axis += o.reach_axis;
        self.reach_path += o.reach_path;
        self.amp_len += o.amp_len;
        self.max_offset = self.max_offset.max(o.max_offset);
        for (a, b) in self.hist.iter_mut().zip(o.hist) {
            *a += b;
        }
        for (a, b) in self.by_q.iter_mut().zip(o.by_q) {
            for (x, y) in a.iter_mut().zip(b) {
                *x += y;
            }
        }
    }
}

struct Plan {
    verts: Option<Vec<RiverVertex>>,
    tally: Tally,
}

/// One point of a course, with the vertex attributes there.
#[derive(Clone, Copy)]
struct Pt {
    x: f64,
    z: f64,
    w: f64,
    hw: f64,
    depth: f64,
    q: f64,
}

impl Pt {
    fn vertex(&self) -> RiverVertex {
        RiverVertex {
            x: self.x as f32,
            z: self.z as f32,
            w: self.w as f32,
            hw: self.hw as f32,
            depth: self.depth as f32,
            q: self.q as f32,
        }
    }
}

struct Net<'a> {
    r: &'a Rivers,
    inp: &'a Input<'a>,
    c: f64,
    chain_of: Vec<u32>,
    join: Vec<Option<Join>>,
    /// In-chain vertex indices that stay where they are, besides each chain's ends.
    fixed: Vec<Vec<usize>>,
    ratio_cap: f64,
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

    /// The x offset from a to b, folded to the nearest lap.
    fn dx(&self, ax: f64, bx: f64) -> f64 {
        let d = bx - ax;
        d - self.c * (d / self.c).round()
    }

    /// The planet's distance between two near points.
    fn dist(&self, ax: f64, az: f64, bx: f64, bz: f64) -> f64 {
        let (ex, ez) = self.scales(0.5 * (az + bz));
        (self.dx(ax, bx) * ex).hypot((bz - az) * ez)
    }

    /// Whether the plain at (x, z) holds a channel whose water surface is w.
    fn on_plain(&self, x: f64, z: f64, w: f64) -> bool {
        (self.inp.plain)(x, z).is_some_and(|lv| lv >= flush_level(w))
    }

    /// Whether (x, z) lies in the channel or bank zone of a chain other than `own` (grid
    /// distance, as the index is built; a direct tributary's segments near its mouth do not count:
    /// the parent's belt may pass over them, and the tributary is cut where it meets it).
    fn blocked(&self, own: u32, x: f64, z: f64) -> bool {
        let r = self.r;
        let (xi, zi) = (x.round() as i64, z.round() as i64);
        let ci = i64::from(r.c);
        let wx = (xi + ci / 2).rem_euclid(ci) - ci / 2;
        for &s in r.tile_entries(r.tile_of(wx, zi)) {
            let k = self.chain_of[s as usize];
            if k == own || k == u32::MAX || self.chain_of.get(s as usize + 1) != Some(&k) {
                continue;
            }
            let (a, b) = (&r.vertices[s as usize], &r.vertices[s as usize + 1]);
            if let Some(j) = self.join[k as usize]
                && r.chains[k as usize].into == own
                && (self.dist(f64::from(a.x), f64::from(a.z), j.x, j.z) <= j.radius
                    || self.dist(f64::from(b.x), f64::from(b.z), j.x, j.z) <= j.radius)
            {
                continue;
            }
            let (ax, az) = (f64::from(a.x), f64::from(a.z));
            let px = ax + self.dx(ax, x);
            let (ex, ez) = (f64::from(b.x) - ax, f64::from(b.z) - az);
            let l2 = ex * ex + ez * ez;
            let t = if l2 > 0.0 {
                (((px - ax) * ex + (z - az) * ez) / l2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let d = (px - ax - t * ex).hypot(z - az - t * ez);
            if d <= r.reach_of(f64::from(a.hw.max(b.hw))) {
                return true;
            }
        }
        false
    }

    /// The course of `v` at planet distance s along it (`u` the vertices' distances).
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

    /// Meanders chain k (its new vertices, None if it keeps its course) and its record.
    fn plan(&self, k: usize) -> Plan {
        let ch = self.r.chains[k];
        let v = &self.r.vertices[ch.first as usize..(ch.first + ch.count) as usize];
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
        let keep = |tally: Tally| Plan { verts: None, tally };
        if ch.from != NO_CHAIN || m < 3 || total < 4.0 * STEP {
            return keep(tally);
        }
        let n = (total / STEP).floor() as usize + 1;
        let pts: Vec<Pt> = (0..n).map(|j| Self::at(v, &u, j as f64 * STEP)).collect();
        let samples = |len: f64| (len / STEP).round().max(0.0) as usize;
        // Box means over ±h samples, by prefix sums.
        let boxed = |a: &[f64], half: &dyn Fn(usize) -> usize, lo: usize, hi: usize| -> Vec<f64> {
            let mut pre = Vec::with_capacity(hi - lo + 2);
            pre.push(0.0);
            for &x in &a[lo..=hi] {
                pre.push(pre.last().unwrap() + x);
            }
            (lo..=hi)
                .map(|j| {
                    let h = half(j);
                    let (a0, a1) = (j.saturating_sub(h).max(lo), (j + h).min(hi));
                    (pre[a1 - lo + 1] - pre[a0 - lo]) / (a1 - a0 + 1) as f64
                })
                .collect()
        };
        let hw: Vec<f64> = pts.iter().map(|p| p.hw).collect();
        let raw_half = |j: usize| samples(WAVELENGTH_PER_WIDTH * 2.0 * hw[j] / 4.0);
        let hw_mean = boxed(&hw, &raw_half, 0, n - 1);
        let lam: Vec<f64> = hw_mean
            .iter()
            .map(|&h| (WAVELENGTH_PER_WIDTH * 2.0 * h).max(4.0 * STEP))
            .collect();
        let quarter = |j: usize| samples(lam[j] / 4.0).max(2);
        let xs: Vec<f64> = pts.iter().map(|p| p.x).collect();
        let zs: Vec<f64> = pts.iter().map(|p| p.z).collect();
        let (xs, zs) = (
            boxed(&xs, &quarter, 0, n - 1),
            boxed(&zs, &quarter, 0, n - 1),
        );
        // The valley's direction (planet metric) and the normal to its left (+90° from east to
        // south), from the smoothed course.
        let mut tan = vec![(1.0, 0.0); n];
        for j in 0..n {
            let (a, b) = (j.saturating_sub(1), (j + 1).min(n - 1));
            let (ex, ez) = self.scales(pts[j].z);
            let (te, ts) = (self.dx(xs[a], xs[b]) * ex, (zs[b] - zs[a]) * ez);
            let l = te.hypot(ts);
            tan[j] = if l > 1e-9 {
                (te / l, ts / l)
            } else if j > 0 {
                tan[j - 1]
            } else {
                (1.0, 0.0)
            };
        }
        let nrm: Vec<(f64, f64)> = tan.iter().map(|&(te, ts)| (-ts, te)).collect();
        // Signed curvature (positive turning towards the normal), over ±half a quarter wavelength.
        let curv: Vec<f64> = (0..n)
            .map(|j| {
                let d = (quarter(j) / 2).max(1);
                let (a, b) = (j.saturating_sub(d), (j + d).min(n - 1));
                if b == a {
                    return 0.0;
                }
                let (t1, t2) = (tan[a], tan[b]);
                let ang = (t1.0 * t2.1 - t1.1 * t2.0).atan2(t1.0 * t2.0 + t1.1 * t2.1);
                ang / ((b - a) as f64 * STEP)
            })
            .collect();
        // Samples beside a vertex that stays (the ends, a distributary's start).
        let mut held = vec![false; n];
        let mut hold = |f: usize| {
            let (s0, s1) = (u[f.saturating_sub(1)], u[(f + 1).min(m - 1)]);
            let j0 = (s0 / STEP).floor().max(0.0) as usize;
            let j1 = ((s1 / STEP).ceil() as usize).min(n - 1);
            held[j0..=j1].iter_mut().for_each(|h| *h = true);
        };
        hold(0);
        hold(m - 1);
        for &f in &self.fixed[k] {
            hold(f);
        }
        let (mv, mh) = (self.inp.m_per_block_v, self.inp.m_per_block_h);
        let mut cand = vec![false; n];
        for j in 0..n {
            let p = &pts[j];
            if !self.on_plain(p.x, p.z, p.w) || held[j] {
                continue;
            }
            let qc = q_class(p.q);
            tally.plain += STEP;
            tally.by_q[qc][0] += STEP;
            let h = samples(lam[j] / 2.0).max(1);
            let (a, b) = (j.saturating_sub(h), (j + h).min(n - 1));
            let slope = (pts[a].w - pts[b].w).max(0.0) * mv / ((b - a).max(1) as f64 * STEP * mh);
            let line = if p.q > 0.0 {
                BRAID_SLOPE_COEF * p.q.powf(BRAID_SLOPE_EXP)
            } else {
                f64::INFINITY
            };
            if slope > line {
                tally.steep += STEP;
                tally.by_q[qc][1] += STEP;
                continue;
            }
            cand[j] = true;
        }
        // The room across the course: to the plain's edge, or half way to another chain's bank.
        let own = k as u32;
        let a_cap: Vec<f64> = lam.iter().map(|&l| l * self.ratio_cap).collect();
        let bounds: Vec<(f64, f64)> = (0..n)
            .into_par_iter()
            .map(|j| {
                if !cand[j] {
                    return (0.0, 0.0);
                }
                let p = &pts[j];
                let t_max = 2.0 * a_cap[j] + p.hw + PROBE;
                let (ex, ez) = self.scales(p.z);
                let side = |sgn: f64| -> f64 {
                    let (ne, ns) = (sgn * nrm[j].0, sgn * nrm[j].1);
                    let mut t = PROBE;
                    let mut last = 0.0;
                    while t <= t_max {
                        let (x, z) = (p.x + t * ne / ex, p.z + t * ns / ez);
                        if !self.on_plain(x, z, p.w) {
                            return last;
                        }
                        if self.blocked(own, x, z) {
                            return 0.5 * t;
                        }
                        last = t;
                        t += PROBE;
                    }
                    last
                };
                let (left, right) = (side(1.0), side(-1.0));
                let mut hi = left - p.hw;
                let mut lo = -(right - p.hw);
                let kap = curv[j];
                if kap > 1e-9 {
                    hi = hi.min(1.0 / kap - p.hw);
                } else if kap < -1e-9 {
                    lo = lo.max(-(1.0 / -kap - p.hw));
                }
                (lo, hi)
            })
            .collect();
        // Runs of candidates; within each the eroded and averaged bounds, the amplitude and the
        // belt's centre.
        let mut amp = vec![0.0; n];
        let mut cen = vec![0.0; n];
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
            let len = r1 - r0 + 1;
            let lo: Vec<f64> = (r0..=r1).map(|i| bounds[i].0).collect();
            let hi: Vec<f64> = (r0..=r1).map(|i| bounds[i].1).collect();
            let (lo_t, hi_t) = (
                SparseTable::new(&lo, f64::max),
                SparseTable::new(&hi, f64::min),
            );
            let mut lo_e = Vec::with_capacity(len);
            let mut hi_e = Vec::with_capacity(len);
            for i in 0..len {
                let h = samples(lam[r0 + i] / 2.0);
                let (a, b) = (i.saturating_sub(h), (i + h).min(len - 1));
                lo_e.push(lo_t.query(a, b));
                hi_e.push(hi_t.query(a, b));
            }
            let qh = |i: usize| samples(lam[r0 + i] / 4.0);
            let lo_s = boxed(&lo_e, &qh, 0, len - 1);
            let hi_s = boxed(&hi_e, &qh, 0, len - 1);
            for i in 0..len {
                let a = (0.5 * (hi_s[i] - lo_s[i])).min(a_cap[r0 + i]);
                if a < MIN_AMPLITUDE_BLOCKS {
                    tally.narrow += STEP;
                    continue;
                }
                amp[r0 + i] = a;
                // The belt as near the old course as it fits (when the room is just the belt, its
                // middle: the two ends can cross by a rounding).
                let (c0, c1) = (lo_s[i] + a, hi_s[i] - a);
                cen[r0 + i] = if c0 < c1 {
                    0.0f64.clamp(c0, c1)
                } else {
                    0.5 * (lo_s[i] + hi_s[i])
                };
            }
        }
        // Reaches: runs of drawable samples, snapped to the course's vertices.
        let mut reaches: Vec<(usize, usize)> = Vec::new();
        let mut j = 0;
        while j < n {
            if amp[j] < MIN_AMPLITUDE_BLOCKS {
                j += 1;
                continue;
            }
            let j0 = j;
            while j < n && amp[j] >= MIN_AMPLITUDE_BLOCKS {
                j += 1;
            }
            let j1 = j - 1;
            let (s0, s1) = (j0 as f64 * STEP, j1 as f64 * STEP);
            let a = u.partition_point(|&x| x < s0);
            let b = u.partition_point(|&x| x <= s1).saturating_sub(1);
            let lam_mean = lam[j0..=j1].iter().sum::<f64>() / (j1 - j0 + 1) as f64;
            if a >= 1 && b + 2 <= m && b > a && u[b] - u[a] >= lam_mean {
                reaches.push((a, b));
            } else {
                tally.short += (j1 - j0 + 1) as f64 * STEP;
            }
        }
        if reaches.is_empty() {
            return keep(tally);
        }
        // The cumulative phase along the course.
        let mut theta = vec![0.0; n];
        for j in 1..n {
            theta[j] = theta[j - 1] + TAU * STEP / lam[j - 1];
        }
        let omega: Vec<f64> = (0..n)
            .map(|j| shape().omega_of_ratio(amp[j] / lam[j]))
            .collect();
        let lerp = |a: &[f64], s: f64| -> f64 {
            let f = (s / STEP).clamp(0.0, (n - 1) as f64);
            let j0 = (f.floor() as usize).min(n.saturating_sub(2));
            let t = f - j0 as f64;
            a[j0] + (a[j0 + 1] - a[j0]) * t
        };
        let sh = shape();
        let nrm_e: Vec<f64> = nrm.iter().map(|p| p.0).collect();
        let nrm_s: Vec<f64> = nrm.iter().map(|p| p.1).collect();
        let q_s: Vec<f64> = pts.iter().map(|p| p.q).collect();
        let mut out: Vec<RiverVertex> = Vec::with_capacity(m * 2);
        let mut next = 0usize;
        for (ri, &(a, b)) in reaches.iter().enumerate() {
            out.extend_from_slice(&v[next..=a]);
            let (ua, ub) = (u[a], u[b]);
            let sgn = if (k + ri).is_multiple_of(2) {
                1.0
            } else {
                -1.0
            };
            let th_a = lerp(&theta, ua);
            let eval = |s: f64| -> (Pt, f64) {
                let lam_s = lerp(&lam, s);
                let d = (s - ua).min(ub - s).max(0.0);
                let tau = (FRAC_PI_2 * (d / (0.5 * lam_s)).min(1.0)).sin().powi(2);
                let yh = sh.y_hat(lerp(&theta, s) - th_a, lerp(&omega, s));
                let y = tau * (lerp(&cen, s) + sgn * lerp(&amp, s) * yh);
                let (ne, ns) = (lerp(&nrm_e, s), lerp(&nrm_s, s));
                let l = ne.hypot(ns).max(1e-9);
                let mut p = Self::at(v, &u, s);
                let (ex, ez) = self.scales(p.z);
                p.x += y * ne / l / ex;
                p.z += y * ns / l / ez;
                (p, y)
            };
            let lam_mean = {
                let (j0, j1) = (samples(ua), samples(ub).min(n - 1));
                lam[j0..=j1].iter().sum::<f64>() / (j1 - j0 + 1) as f64
            };
            let step = (lam_mean * PATH_STEP_PER_WAVELENGTH).clamp(1.0, 6.0);
            // Knots at the course's samples, subdivided where the new channel runs across them.
            let mut knots = vec![ua];
            let j_first = (ua / STEP).floor() as usize + 1;
            let j_last = ((ub / STEP).ceil() as usize).saturating_sub(1);
            for j in j_first..=j_last.min(n - 1) {
                let s = j as f64 * STEP;
                if s > ua && s < ub {
                    knots.push(s);
                }
            }
            knots.push(ub);
            let mut prev = (Self::at(v, &u, ua), 0.0);
            let mut dense: Vec<(Pt, f64)> = Vec::new();
            for w in knots.windows(2) {
                let end = eval(w[1]);
                let d = self.dist(prev.0.x, prev.0.z, end.0.x, end.0.z);
                let parts = (d / (0.5 * step)).ceil().max(1.0) as usize;
                for i in 1..parts {
                    dense.push(eval(w[0] + (w[1] - w[0]) * i as f64 / parts as f64));
                }
                dense.push(end);
                prev = end;
            }
            // Keep a point every `step` of the new channel; the last dense point is vertex b.
            let mut last = Self::at(v, &u, ua);
            let mut run = 0.0;
            let mut path = 0.0;
            let mut amp_len = 0.0;
            let vb = Self::at(v, &u, ub);
            let mut prev_pt = last;
            for (i, &(p, y)) in dense.iter().enumerate() {
                run += self.dist(prev_pt.x, prev_pt.z, p.x, p.z);
                prev_pt = p;
                tally.max_offset = tally.max_offset.max(y.abs());
                if i + 1 == dense.len() {
                    break;
                }
                if run >= step && self.dist(p.x, p.z, vb.x, vb.z) >= 0.5 * step {
                    path += self.dist(last.x, last.z, p.x, p.z);
                    out.push(p.vertex());
                    last = p;
                    run = 0.0;
                }
            }
            path += self.dist(last.x, last.z, vb.x, vb.z);
            amp_len += amp[samples(ua)..=samples(ub).min(n - 1)]
                .iter()
                .sum::<f64>()
                * STEP;
            let axis = ub - ua;
            let k_reach = path / axis.max(1e-9);
            tally.reaches += 1;
            tally.reach_axis += axis;
            tally.reach_path += path;
            tally.amp_len += amp_len;
            let qc = q_class(lerp(&q_s, 0.5 * (ua + ub)));
            tally.by_q[qc][2] += axis;
            tally.by_q[qc][3] += path;
            let bin = SINUOSITY_EDGES
                .iter()
                .take_while(|&&e| k_reach >= e)
                .count();
            tally.hist[bin] += 1;
            next = b;
        }
        out.extend_from_slice(&v[next..]);
        Plan {
            verts: Some(out),
            tally,
        }
    }
}

/// Range minimum (or maximum) queries over a fixed array.
struct SparseTable {
    levels: Vec<Vec<f64>>,
    op: fn(f64, f64) -> f64,
}

impl SparseTable {
    fn new(a: &[f64], op: fn(f64, f64) -> f64) -> SparseTable {
        let mut levels = vec![a.to_vec()];
        let mut w = 1;
        while 2 * w <= a.len() {
            let prev = levels.last().unwrap();
            let next: Vec<f64> = (0..=a.len() - 2 * w)
                .map(|i| op(prev[i], prev[i + w]))
                .collect();
            levels.push(next);
            w *= 2;
        }
        SparseTable { levels, op }
    }

    /// The op over a[i..=j].
    fn query(&self, i: usize, j: usize) -> f64 {
        let len = j - i + 1;
        let k = usize::BITS as usize - 1 - len.leading_zeros() as usize;
        (self.op)(self.levels[k][i], self.levels[k][j + 1 - (1 << k)])
    }
}

/// Meanders `rivers` in place (the chains keep their ids and order; the index is rebuilt) and
/// returns `stats.meanders`.
pub fn apply(rivers: &mut Rivers, inp: &Input<'_>) -> Value {
    let t0 = Instant::now();
    let vertices_before = rivers.vertices.len();
    let k_chains = rivers.chains.len();
    let mut chain_of = vec![u32::MAX; rivers.vertices.len()];
    for (k, ch) in rivers.chains.iter().enumerate() {
        for i in ch.first..ch.first + ch.count {
            chain_of[i as usize] = k as u32;
        }
    }
    let ratio_cap = ratio_cap();
    let mut net = Net {
        r: &*rivers,
        inp,
        c: f64::from(rivers.c),
        chain_of,
        join: vec![None; k_chains],
        fixed: vec![Vec::new(); k_chains],
        ratio_cap,
    };
    // The vertex of `p` nearest (x, z), in-chain.
    let nearest_in = |net: &Net<'_>, p: usize, x: f64, z: f64| -> usize {
        let ch = net.r.chains[p];
        let mut best = (f64::INFINITY, 0usize);
        for i in 0..ch.count as usize {
            let q = &net.r.vertices[ch.first as usize + i];
            let d = net.dist(x, z, f64::from(q.x), f64::from(q.z));
            if d < best.0 {
                best = (d, i);
            }
        }
        best.1
    };
    let mut joins = vec![None; k_chains];
    let mut fixed = vec![Vec::new(); k_chains];
    for (k, ch) in rivers.chains.iter().enumerate() {
        if ch.mouth == Mouth::River && (ch.into as usize) < k_chains && ch.count >= 2 {
            let e = rivers.vertices[(ch.first + ch.count - 1) as usize];
            let (x, z) = (f64::from(e.x), f64::from(e.z));
            let i = nearest_in(&net, ch.into as usize, x, z);
            let p = rivers.chains[ch.into as usize];
            let hw = f64::from(rivers.vertices[p.first as usize + i].hw);
            let radius = 2.0 * WAVELENGTH_PER_WIDTH * 2.0 * hw * ratio_cap + hw + 2.0;
            joins[k] = Some(Join { x, z, radius });
        }
        if ch.from != NO_CHAIN && (ch.from as usize) < k_chains {
            let s = rivers.vertices[ch.first as usize];
            let p = rivers.chains[ch.from as usize];
            if let Some(i) = (0..p.count as usize).find(|&i| {
                let q = &rivers.vertices[p.first as usize + i];
                q.x == s.x && q.z == s.z
            }) {
                fixed[ch.from as usize].push(i);
            }
        }
    }
    net.join = joins;
    net.fixed = fixed;
    let plans: Vec<Plan> = (0..k_chains).into_par_iter().map(|k| net.plan(k)).collect();
    let mut total = Tally::default();
    for p in &plans {
        total.add(&p.tally);
    }
    let changed: Vec<bool> = plans.iter().map(|p| p.verts.is_some()).collect();
    let mut new: Vec<Vec<RiverVertex>> = plans
        .into_iter()
        .enumerate()
        .map(|(k, p)| {
            p.verts.unwrap_or_else(|| {
                let ch = net.r.chains[k];
                net.r.vertices[ch.first as usize..(ch.first + ch.count) as usize].to_vec()
            })
        })
        .collect();

    // Tributaries whose parent moved under their mouth, parents first.
    let mut depth = vec![usize::MAX; k_chains];
    for k in 0..k_chains {
        let mut path = Vec::new();
        let mut cur = k;
        let base = loop {
            if depth[cur] != usize::MAX {
                break depth[cur];
            }
            let ch = net.r.chains[cur];
            if ch.mouth != Mouth::River || ch.into as usize >= k_chains || path.len() > k_chains {
                depth[cur] = 0;
                break 0;
            }
            path.push(cur);
            cur = ch.into as usize;
        };
        for (i, &q) in path.iter().rev().enumerate() {
            depth[q] = base + i + 1;
        }
    }
    let mut order: Vec<usize> = (0..k_chains)
        .filter(|&k| net.r.chains[k].mouth == Mouth::River && depth[k] > 0)
        .collect();
    order.sort_by_key(|&k| (depth[k], k));
    let mut into_now: Vec<u32> = net.r.chains.iter().map(|ch| ch.into).collect();
    // A chain cut at a join: its vertices before the cut, and the cut's index in them.
    let mut cut: Vec<Option<(Vec<RiverVertex>, usize)>> = vec![None; k_chains];
    let (mut moved, mut truncated, mut retargeted) = (0u64, 0u64, 0u64);
    let mut max_move: f64 = 0.0;
    // The same point: z bit for bit, x on any lap.
    let c_f = f64::from(rivers.c);
    let same = |a: &RiverVertex, x: f32, z: f32| {
        let d = (f64::from(a.x) - f64::from(x)).rem_euclid(c_f);
        a.z == z && (d < 1e-3 || d > c_f - 1e-3)
    };
    for &t in &order {
        let Some(&end) = new[t].last() else { continue };
        let mut p = into_now[t] as usize;
        let mut guard = 0;
        let parent = loop {
            guard += 1;
            if guard > k_chains || new[t].len() < 2 {
                break None;
            }
            if new[p].iter().any(|q| same(q, end.x, end.z)) {
                break None;
            }
            // The parent's own cut dropped the point: the confluence moved to its parent.
            if let Some((pre, at)) = &cut[p] {
                let i = (0..pre.len())
                    .min_by(|&a, &b| {
                        let da = net.dist(
                            f64::from(pre[a].x),
                            f64::from(pre[a].z),
                            f64::from(end.x),
                            f64::from(end.z),
                        );
                        let db = net.dist(
                            f64::from(pre[b].x),
                            f64::from(pre[b].z),
                            f64::from(end.x),
                            f64::from(end.z),
                        );
                        da.total_cmp(&db)
                    })
                    .unwrap_or(0);
                if i >= *at && net.r.chains[p].mouth == Mouth::River {
                    p = into_now[p] as usize;
                    retargeted += 1;
                    continue;
                }
            }
            if !changed[p] && cut[p].is_none() {
                break None;
            }
            break Some(p);
        };
        into_now[t] = p as u32;
        let Some(p) = parent else { continue };
        let (ex_, ez_) = net.scales(f64::from(end.z));
        let (exf, ezf) = (f64::from(end.x), f64::from(end.z));
        let local = |x: f64, z: f64| (net.dx(exf, x) * ex_, (z - ezf) * ez_);
        let pv = &new[p];
        let hw_near = pv
            .iter()
            .map(|q| (net.dist(exf, ezf, f64::from(q.x), f64::from(q.z)), q.hw))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map_or(1.0, |a| f64::from(a.1));
        let radius = 2.0 * WAVELENGTH_PER_WIDTH * 2.0 * hw_near * ratio_cap + hw_near + 2.0;
        let reach = radius + 2.0 * hw_near + 8.0;
        let mut segs: Vec<usize> = (0..pv.len().saturating_sub(1))
            .filter(|&i| {
                let (a, b) = (
                    local(f64::from(pv[i].x), f64::from(pv[i].z)),
                    local(f64::from(pv[i + 1].x), f64::from(pv[i + 1].z)),
                );
                a.0.hypot(a.1) <= reach || b.0.hypot(b.1) <= reach
            })
            .collect();
        if segs.is_empty() {
            segs = (0..pv.len().saturating_sub(1)).collect();
        }
        // The parent's new channel nearest a point: (distance, its half-width there).
        let near = |x: f64, z: f64| -> (f64, f64) {
            let (px, pz) = local(x, z);
            let mut best = (f64::INFINITY, 1.0);
            for &i in &segs {
                let (a, b) = (
                    local(f64::from(pv[i].x), f64::from(pv[i].z)),
                    local(f64::from(pv[i + 1].x), f64::from(pv[i + 1].z)),
                );
                let (ex, ez) = (b.0 - a.0, b.1 - a.1);
                let l2 = ex * ex + ez * ez;
                let tt = if l2 > 0.0 {
                    (((px - a.0) * ex + (pz - a.1) * ez) / l2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let d = (px - a.0 - tt * ex).hypot(pz - a.1 - tt * ez);
                if d < best.0 {
                    best = (
                        d,
                        f64::from(pv[i].hw) + (f64::from(pv[i + 1].hw) - f64::from(pv[i].hw)) * tt,
                    );
                }
            }
            best
        };
        // The parent's vertex nearest a point.
        let vertex_near = |x: f64, z: f64| -> RiverVertex {
            let (px, pz) = local(x, z);
            let mut best = (f64::INFINITY, pv[0]);
            for &i in &segs {
                for q in [pv[i], pv[i + 1]] {
                    let a = local(f64::from(q.x), f64::from(q.z));
                    let d = (a.0 - px).hypot(a.1 - pz);
                    if d < best.0 {
                        best = (d, q);
                    }
                }
            }
            best.1
        };
        let tv = new[t].clone();
        let mt = tv.len();
        let mut k0 = mt - 1;
        while k0 > 0 && {
            let q = &tv[k0 - 1];
            net.dist(f64::from(q.x), f64::from(q.z), exf, ezf) <= radius
        } {
            k0 -= 1;
        }
        let hit = (k0.max(1)..mt - 1).find(|&i| {
            let (d, hw) = near(f64::from(tv[i].x), f64::from(tv[i].z));
            d <= hw
        });
        let at = match hit {
            Some(i) => {
                truncated += 1;
                i
            }
            None => {
                moved += 1;
                mt - 1
            }
        };
        let prev = tv[at - 1];
        let mut j = vertex_near(f64::from(tv[at].x), f64::from(tv[at].z));
        if hit.is_none() {
            j = vertex_near(f64::from(prev.x), f64::from(prev.z));
        }
        let shift = (f64::from(prev.x - j.x) / net.c).round() * net.c;
        let last = RiverVertex {
            x: (f64::from(j.x) + shift) as f32,
            z: j.z,
            w: prev.w.min(j.w),
            hw: end.hw,
            depth: end.depth,
            q: end.q,
        };
        max_move = max_move.max(net.dist(exf, ezf, f64::from(last.x), f64::from(last.z)));
        let mut nt = tv[..at].to_vec();
        nt.push(last);
        cut[t] = Some((tv, at));
        new[t] = nt;
    }

    // The new arrays.
    let mut vertices = Vec::with_capacity(new.iter().map(Vec::len).sum());
    let mut chains = net.r.chains.clone();
    let mut length_after = 0.0;
    for (k, nv) in new.iter().enumerate() {
        chains[k].first = vertices.len() as u32;
        chains[k].count = nv.len() as u32;
        if chains[k].mouth == Mouth::River {
            chains[k].into = into_now[k];
        }
        for w in nv.windows(2) {
            length_after += net.dist(
                f64::from(w[0].x),
                f64::from(w[0].z),
                f64::from(w[1].x),
                f64::from(w[1].z),
            );
        }
        vertices.extend_from_slice(nv);
    }
    let chains_meandered = changed.iter().filter(|&&c| c).count();
    drop(net);
    rivers.vertices = vertices;
    rivers.chains = chains;
    rivers.build_index();

    let km = inp.m_per_block_h / 1000.0;
    let r4 = |x: f64| (x * 1e4).round() / 1e4;
    let r1 = |x: f64| (x * 10.0).round() / 10.0;
    let by_q: Vec<Value> = Q_CLASSES
        .iter()
        .zip(total.by_q)
        .map(|(&q, b)| {
            json!({
                "q_min_m3s": q,
                "plain_km": r1(b[0] * km),
                "steep_km": r1(b[1] * km),
                "meandered_axis_km": r1(b[2] * km),
                "meandered_path_km": r1(b[3] * km),
                "sinuosity": if b[2] > 0.0 { json!(r4(b[3] / b[2])) } else { Value::Null },
                "meandered_share_of_plain": if b[0] > 0.0 { json!(r4(b[2] / b[0])) } else { Value::Null },
            })
        })
        .collect();
    json!({
        "chains_meandered": chains_meandered,
        "reaches": total.reaches,
        "network_km_before": r1(total.length * km),
        "network_km_after": r1(length_after * km),
        "network_sinuosity": r4(length_after / total.length.max(1e-9)),
        "plain_km": r1(total.plain * km),
        "steep_km": r1(total.steep * km),
        "narrow_km": r1(total.narrow * km),
        "short_km": r1(total.short * km),
        "meandered_axis_km": r1(total.reach_axis * km),
        "meandered_path_km": r1(total.reach_path * km),
        "reach_sinuosity": if total.reach_axis > 0.0 { json!(r4(total.reach_path / total.reach_axis)) } else { Value::Null },
        "meandered_share_of_plain": if total.plain > 0.0 { json!(r4(total.reach_axis / total.plain)) } else { Value::Null },
        "mean_amplitude_blocks": if total.reach_axis > 0.0 { json!(r1(total.amp_len / total.reach_axis)) } else { Value::Null },
        "max_offset_blocks": r1(total.max_offset),
        "sinuosity_edges": SINUOSITY_EDGES,
        "reach_sinuosity_hist": total.hist,
        "by_q": by_q,
        "vertices_before": vertices_before,
        "vertices_after": rivers.vertices.len(),
        "joins_cut": truncated,
        "joins_carried": moved,
        "joins_moved_to_grandparent": retargeted,
        "max_join_move_blocks": r1(max_move),
        "millis": t0.elapsed().as_millis() as u64,
    })
}

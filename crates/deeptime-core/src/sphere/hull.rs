// SPDX-License-Identifier: LGPL-3.0-or-later
//! The spherical Delaunay triangulation of L0's samples, as the 3D convex hull of their unit
//! vectors (plan §5.2).
//!
//! For points on a sphere the two are the same thing: a hull face's plane cuts the sphere in the
//! face's circumcircle and leaves every other point on the inner side, which is the empty-circle
//! property (Brown 1979). Seen from the centre, the hull's faces tile the sphere.
//!
//! # Algorithm
//! A randomized incremental convex hull with conflict buckets: Clarkson & Shor 1989 ("Applications
//! of random sampling in computational geometry, II"), as presented by de Berg, Cheong, van
//! Kreveld & Overmars 2008 ("Computational Geometry", 3rd ed., ch. 11), in the bucketing form that
//! Guibas, Knuth & Sharir 1992 use for incremental Delaunay triangulation. Implemented from those
//! descriptions; no code was ported.
//!
//! 1. **Snap.** Coordinates are rounded to integers in units of 2⁻³⁸ ([`SNAP_BITS`]), which moves a
//!    unit vector by at most 3.2e-12 (0.02 mm on Earth's radius). Orientation is then decided
//!    exactly: a floating-point evaluation with Shewchuk's (1997) stage-A error bound, and an
//!    `i128` evaluation when the bound cannot certify the sign (|coordinates| ≤ 2³⁸ keeps every
//!    determinant below 2¹²⁰). Every combinatorial decision is therefore exact for the snapped
//!    points, and the output is always a closed, consistently oriented triangulation.
//! 2. **Start** from four points spanning a tetrahedron (quickhull's choice: the farthest pair of
//!    axis extremes, the point farthest from their line, the point farthest from that plane), and
//!    a centre `c`, the tetrahedron's rounded centroid, which stays strictly inside every later
//!    hull. Every other point waits in the bucket of the face whose cone from `c` contains it.
//! 3. **Insert** the waiting points in a seeded biased randomized insertion order (BRIO: random
//!    rounds of doubling size, each sorted along a Morton curve). A point p outside the hull sees
//!    the face of its bucket (the ray from `c` to p leaves the hull through that face, so p lies
//!    beyond its plane); a point that does not is inside or on the hull, is left out and is
//!    reported by [`SphereMesh::skipped`] (duplicates, points pushed inside by rounding). The
//!    faces p sees form a disc, found by a flood fill from the bucket's face; they are replaced by
//!    the fan joining p to the disc's boundary (the horizon), and the points waiting in the
//!    removed faces move to the fan face whose cone contains them. The removed disc and the fan
//!    cover the same cone of directions, so one fan face always does.
//!
//! Expected time O(n log n) for n points in random order (each waiting point changes bucket
//! O(log n) times in expectation, Clarkson & Shor; BRIO keeps the bound), O(n) memory. Measured
//! on the M6 Mac mini (release): 0.10 s for 100k points, 0.25 s for 250k. Degenerate input —
//! four or more cocircular points, such as a cube's corners — gives coplanar neighbouring faces,
//! one of the valid Delaunay triangulations. At the end the sphere's centre must lie strictly
//! inside the hull (the samples surround it), or the build fails with [`HullError::NotCovering`];
//! point location and the raster coupling look from that centre.
//!
//! The build is sequential; the snapping, the initial bucketing and the final checks run on the
//! current Rayon pool. Measured build times are in STATUS.md (P1-02).

use std::fmt;

use glam::DVec3;
use rayon::prelude::*;

use super::SplitMix64;

/// Coordinates are snapped to multiples of 2^-SNAP_BITS for the exact predicates.
pub const SNAP_BITS: u32 = 38;
const SNAP_SCALE: f64 = (1u64 << SNAP_BITS) as f64;
/// The default seed of the insertion order.
pub const DEFAULT_SEED: u64 = 0x5EED_0DEE_9714_E001;
const NONE: u32 = u32::MAX;

/// A snapped point: coordinates in units of 2^-SNAP_BITS, each in [−2^SNAP_BITS, 2^SNAP_BITS].
type Fixed = [i64; 3];
const ORIGIN: Fixed = [0; 3];

/// Shewchuk's o3derrboundA with ε = 2⁻⁵³.
const HALF_EPS: f64 = f64::EPSILON / 2.0;
const ORIENT_ERRBOUND: f64 = (7.0 + 56.0 * HALF_EPS) * HALF_EPS;

fn snap(v: DVec3) -> Fixed {
    let s = |c: f64| (c * SNAP_SCALE).round().clamp(-SNAP_SCALE, SNAP_SCALE) as i64;
    [s(v.x), s(v.y), s(v.z)]
}

/// The exact sign of ((b − a) × (c − a)) · (d − a): +1 when d lies on the side the normal of the
/// counter-clockwise triangle abc points to, −1 on the other side, 0 when the four are coplanar.
#[inline]
fn orient(a: &Fixed, b: &Fixed, c: &Fixed, d: &Fixed) -> i32 {
    // Differences are exact in i64 and in f64: |·| ≤ 2^(SNAP_BITS+1) < 2^53.
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let w = [d[0] - a[0], d[1] - a[1], d[2] - a[2]];
    let (ux, uy, uz) = (u[0] as f64, u[1] as f64, u[2] as f64);
    let (vx, vy, vz) = (v[0] as f64, v[1] as f64, v[2] as f64);
    let (wx, wy, wz) = (w[0] as f64, w[1] as f64, w[2] as f64);
    let (vywz, vzwy) = (vy * wz, vz * wy);
    let (vxwz, vzwx) = (vx * wz, vz * wx);
    let (vxwy, vywx) = (vx * wy, vy * wx);
    let det = ux * (vywz - vzwy) - uy * (vxwz - vzwx) + uz * (vxwy - vywx);
    let permanent = ux.abs() * (vywz.abs() + vzwy.abs())
        + uy.abs() * (vxwz.abs() + vzwx.abs())
        + uz.abs() * (vxwy.abs() + vywx.abs());
    let bound = ORIENT_ERRBOUND * permanent;
    if det > bound {
        1
    } else if -det > bound {
        -1
    } else {
        orient_exact(&u, &v, &w)
    }
}

/// u · (v × w) in `i128`: |u|, |v|, |w| ≤ 2^39, so the 2×2 minors stay below 2^79 and the result
/// below 3·2^118.
#[cold]
fn orient_exact(u: &[i64; 3], v: &[i64; 3], w: &[i64; 3]) -> i32 {
    let m = |a: i64, b: i64| i128::from(a) * i128::from(b);
    let m1 = m(v[1], w[2]) - m(v[2], w[1]);
    let m2 = m(v[0], w[2]) - m(v[2], w[0]);
    let m3 = m(v[0], w[1]) - m(v[1], w[0]);
    let det = i128::from(u[0]) * m1 - i128::from(u[1]) * m2 + i128::from(u[2]) * m3;
    det.signum() as i32
}

/// Why a triangulation could not be built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HullError {
    /// Fewer than four points.
    TooFewPoints(usize),
    /// More points than the `u32` indices address (the limit is 2³⁰).
    TooManyPoints(usize),
    /// The point at this index is not finite or has zero length.
    BadPoint(usize),
    /// The points are coplanar or do not surround the sphere's centre (e.g. all in one
    /// hemisphere), so their hull does not tile the sphere.
    NotCovering,
    /// An internal invariant failed (cannot happen with exact predicates; kept as an error rather
    /// than a panic so a bug surfaces as a failed stage).
    Internal(&'static str),
}

impl fmt::Display for HullError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HullError::TooFewPoints(n) => {
                write!(f, "a sphere triangulation needs 4 points, got {n}")
            }
            HullError::TooManyPoints(n) => write!(f, "{n} points exceed the 2^30 limit"),
            HullError::BadPoint(i) => write!(f, "point {i} is not a finite non-zero vector"),
            HullError::NotCovering => {
                write!(f, "the points do not surround the sphere's centre")
            }
            HullError::Internal(what) => write!(f, "sphere triangulation invariant failed: {what}"),
        }
    }
}

impl std::error::Error for HullError {}

/// A spherical Delaunay triangulation: triangles over the input points, counter-clockwise seen
/// from outside the sphere, with their edge neighbours.
#[derive(Clone, Debug)]
pub struct SphereMesh {
    points: Vec<DVec3>,
    fixed: Vec<Fixed>,
    triangles: Vec<[u32; 3]>,
    neighbors: Vec<[u32; 3]>,
    vertex_triangle: Vec<u32>,
    skipped: Vec<u32>,
}

/// Vertex adjacency in compressed rows: the neighbours of vertex v are
/// `neighbors[offsets[v]..offsets[v + 1]]`, counter-clockwise seen from outside.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adjacency {
    /// Row starts; one more entry than there are points.
    pub offsets: Vec<u32>,
    /// The concatenated rows.
    pub neighbors: Vec<u32>,
}

impl Adjacency {
    /// The neighbours of vertex `v` (empty for a skipped point).
    pub fn of(&self, v: u32) -> &[u32] {
        let v = v as usize;
        &self.neighbors[self.offsets[v] as usize..self.offsets[v + 1] as usize]
    }
}

impl SphereMesh {
    /// Triangulates `points` (normalised to unit vectors first) with the default insertion seed.
    pub fn build(points: &[DVec3]) -> Result<SphereMesh, HullError> {
        SphereMesh::build_seeded(points, DEFAULT_SEED)
    }

    /// Triangulates `points` inserting them in the order drawn from `seed`. The triangulation
    /// is unique up to cocircular ties, so the seed only matters for degenerate input and speed.
    pub fn build_seeded(points: &[DVec3], seed: u64) -> Result<SphereMesh, HullError> {
        let n = points.len();
        if n < 4 {
            return Err(HullError::TooFewPoints(n));
        }
        if n > 1 << 30 {
            return Err(HullError::TooManyPoints(n));
        }
        if let Some(i) = points
            .par_iter()
            .position_first(|p| !p.is_finite() || p.length_squared() < 1e-200)
        {
            return Err(HullError::BadPoint(i));
        }
        let unit: Vec<DVec3> = points.par_iter().map(|p| p.normalize()).collect();
        let fixed: Vec<Fixed> = unit.par_iter().map(|&p| snap(p)).collect();
        let tet = initial_simplex(&unit, &fixed)?;
        let mut builder = Builder::new(&fixed, tet)?;
        let order = brio_order(&fixed, &tet, seed);
        for (round, &p) in order.iter().enumerate() {
            builder.insert(p, round as u32 + 1)?;
        }
        let mut mesh = builder.finish()?;
        mesh.points = unit;
        mesh.fixed = fixed;
        Ok(mesh)
    }

    /// The same triangulation over `points`, this mesh's points turned by one rotation (unit
    /// vectors, one per point, in the same order). A rotation keeps every face's orientation and
    /// its empty circumcircle, so the triangles, their neighbours and the skipped points carry
    /// over; only where four points are cocircular to rounding may the diagonal differ from the
    /// one [`SphereMesh::build`] would pick for the turned points, and either is Delaunay
    /// (P1-19: the plate model's resamplings turn one lattice instead of triangulating each).
    pub fn turned(&self, points: Vec<DVec3>) -> SphereMesh {
        assert_eq!(
            points.len(),
            self.points.len(),
            "one turned point per point"
        );
        let fixed = points.par_iter().map(|&p| snap(p)).collect();
        SphereMesh {
            points,
            fixed,
            triangles: self.triangles.clone(),
            neighbors: self.neighbors.clone(),
            vertex_triangle: self.vertex_triangle.clone(),
            skipped: self.skipped.clone(),
        }
    }

    /// The points, normalised; vertex indices refer to this slice.
    pub fn points(&self) -> &[DVec3] {
        &self.points
    }

    /// The triangles, counter-clockwise seen from outside.
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }

    /// `neighbors()[t][k]` is the triangle across edge k of triangle t, the edge from vertex
    /// `triangles()[t][k]` to `triangles()[t][(k + 1) % 3]`.
    pub fn neighbors(&self) -> &[[u32; 3]] {
        &self.neighbors
    }

    /// Points that are not vertices, ascending: duplicates, and points that lie inside or on
    /// the hull of the others. Empty for any set of distinct points on the sphere.
    pub fn skipped(&self) -> &[u32] {
        &self.skipped
    }

    /// The number of vertices (points minus skipped).
    pub fn vertex_count(&self) -> usize {
        self.points.len() - self.skipped.len()
    }

    /// The number of edges, 3F/2 on a closed triangulated surface.
    pub fn edge_count(&self) -> usize {
        self.triangles.len() * 3 / 2
    }

    /// One triangle incident to vertex `v`, or `None` for a skipped point.
    pub fn vertex_triangle(&self, v: u32) -> Option<u32> {
        self.vertex_triangle
            .get(v as usize)
            .copied()
            .filter(|&t| t != NONE)
    }

    /// Whether the direction `q` lies in the closed cone from the sphere's centre over triangle
    /// `t` (exact, on snapped coordinates).
    pub fn contains(&self, t: u32, q: DVec3) -> bool {
        self.contains_fixed(t, &snap(q.normalize_or_zero()))
    }

    fn contains_fixed(&self, t: u32, q: &Fixed) -> bool {
        let [a, b, c] = self.triangles[t as usize].map(|v| &self.fixed[v as usize]);
        orient(&ORIGIN, a, b, q) >= 0
            && orient(&ORIGIN, b, c, q) >= 0
            && orient(&ORIGIN, c, a, q) >= 0
    }

    /// A triangle whose closed cone contains the direction `q`, found by a stochastic visibility
    /// walk (Devillers, Pion & Teillaud 2002) from triangle `hint` with exact predicates. Walks
    /// from a nearby hint take a step or two; a walk that has not arrived after
    /// 64 + 16·√F steps falls back to scanning every triangle.
    pub fn locate(&self, q: DVec3, hint: u32) -> u32 {
        let q = snap(q.normalize_or_zero());
        let nt = self.triangles.len() as u32;
        let mut t = if hint < nt { hint } else { 0 };
        // xorshift32 for the edge order; the high bit keeps the state non-zero.
        let mut rng = 0x9E37_79B9u32 ^ (t & 0x7FFF_FFFF);
        let limit = 64 + 16 * (nt as f64).sqrt() as usize;
        'walk: for _ in 0..limit {
            let tri = self.triangles[t as usize];
            rng ^= rng << 13;
            rng ^= rng >> 17;
            rng ^= rng << 5;
            let first = (rng % 3) as usize;
            for s in 0..3 {
                let e = (first + s) % 3;
                let (a, b) = (tri[e] as usize, tri[(e + 1) % 3] as usize);
                if orient(&ORIGIN, &self.fixed[a], &self.fixed[b], &q) < 0 {
                    t = self.neighbors[t as usize][e];
                    continue 'walk;
                }
            }
            return t;
        }
        (0..nt)
            .into_par_iter()
            .find_first(|&t| self.contains_fixed(t, &q))
            .unwrap_or(t)
    }

    /// Barycentric weights of the direction `q` in triangle `t`, in the order of the triangle's
    /// vertices: the weights of the point where the ray from the centre towards `q` meets the
    /// triangle's plane (gnomonic barycentrics, w_a ∝ det(q, b, c) and cyclically). They are
    /// non-negative and sum to 1 when `q` lies in the triangle's cone, as [`SphereMesh::locate`]
    /// guarantees; rounding below zero is clamped.
    pub fn barycentric(&self, t: u32, q: DVec3) -> [f64; 3] {
        let tri = self.triangles[t as usize];
        let [a, b, c] = tri.map(|v| self.points[v as usize]);
        let raw = [q.dot(b.cross(c)), q.dot(c.cross(a)), q.dot(a.cross(b))];
        let w = raw.map(|x| x.max(0.0));
        let s = w[0] + w[1] + w[2];
        if s > 0.0 && s.is_finite() {
            w.map(|x| x / s)
        } else {
            // q is (numerically) at the centre's antipode of the face or the face is flat:
            // give everything to the nearest corner.
            let d = [a, b, c].map(|v| v.dot(q));
            let k = (0..3).fold(0, |k, i| if d[i] > d[k] { i } else { k });
            let mut out = [0.0; 3];
            out[k] = 1.0;
            out
        }
    }

    /// Vertex adjacency (the Delaunay neighbours of every vertex), counter-clockwise seen from
    /// outside; empty rows for skipped points.
    pub fn vertex_neighbors(&self) -> Adjacency {
        let rows: Vec<Vec<u32>> = (0..self.points.len() as u32)
            .into_par_iter()
            .map(|v| self.star(v))
            .collect();
        let mut offsets = Vec::with_capacity(rows.len() + 1);
        offsets.push(0u32);
        let mut total = 0u32;
        for r in &rows {
            total += r.len() as u32;
            offsets.push(total);
        }
        Adjacency {
            offsets,
            neighbors: rows.concat(),
        }
    }

    /// The neighbours of `v` in counter-clockwise order.
    fn star(&self, v: u32) -> Vec<u32> {
        let Some(t0) = self.vertex_triangle(v) else {
            return Vec::new();
        };
        let mut out = Vec::with_capacity(8);
        let mut t = t0;
        loop {
            let tri = self.triangles[t as usize];
            let k = tri
                .iter()
                .position(|&x| x == v)
                .expect("vertex in its star");
            out.push(tri[(k + 1) % 3]);
            t = self.neighbors[t as usize][(k + 2) % 3];
            if t == t0 || out.len() > self.triangles.len() {
                return out;
            }
        }
    }
}

/// The insertion order: a biased randomized insertion order (Amenta, Choi & Rote 2003,
/// "Incremental constructions con BRIO"). The shuffled points split into rounds of doubling size
/// (the last round is the second half, the one before it the quarter before that, …) and each
/// round is sorted along a Morton curve, so consecutive insertions touch nearby faces while every
/// round stays a random sample, which keeps the expected O(n log n) bound. On 250k Fibonacci
/// points it saves about 6% over a plain shuffle (254 against 270 ms on the M6 Mac mini).
fn brio_order(fixed: &[Fixed], skip: &[u32; 4], seed: u64) -> Vec<u32> {
    let mut order: Vec<u32> = (0..fixed.len() as u32)
        .filter(|i| !skip.contains(i))
        .collect();
    SplitMix64::new(seed).shuffle(&mut order);
    let mut end = order.len();
    while end > 64 {
        let start = end / 2;
        order[start..end].par_sort_unstable_by_key(|&i| morton(&fixed[i as usize]));
        end = start;
    }
    order
}

/// The 63-bit Morton (Z-order) key of a snapped point, 21 bits per axis.
fn morton(p: &Fixed) -> u64 {
    fn spread(x: u64) -> u64 {
        let mut x = x & 0x1F_FFFF;
        x = (x | x << 32) & 0x001F_0000_0000_FFFF;
        x = (x | x << 16) & 0x001F_0000_FF00_00FF;
        x = (x | x << 8) & 0x100F_00F0_0F00_F00F;
        x = (x | x << 4) & 0x10C3_0C30_C30C_30C3;
        x = (x | x << 2) & 0x1249_2492_4924_9249;
        x
    }
    let q =
        |c: i64| ((c + (1 << SNAP_BITS)) >> (SNAP_BITS + 1 - 21)).clamp(0, (1 << 21) - 1) as u64;
    spread(q(p[0])) | spread(q(p[1])) << 1 | spread(q(p[2])) << 2
}

/// Four points spanning a tetrahedron: quickhull's farthest-pair / farthest-from-line /
/// farthest-from-plane choice (in f64; the result is checked exactly).
fn initial_simplex(unit: &[DVec3], fixed: &[Fixed]) -> Result<[u32; 4], HullError> {
    let n = unit.len();
    let argmax = |f: &(dyn Fn(usize) -> f64 + Sync)| -> usize {
        (0..n)
            .into_par_iter()
            .map(|i| (f(i), i))
            .reduce(
                || (f64::NEG_INFINITY, usize::MAX),
                |x, y| {
                    if y.0 > x.0 || (y.0 == x.0 && y.1 < x.1) {
                        y
                    } else {
                        x
                    }
                },
            )
            .1
    };
    let mut extremes = [0usize; 6];
    for axis in 0..3 {
        extremes[2 * axis] = argmax(&|i| unit[i][axis]);
        extremes[2 * axis + 1] = argmax(&|i| -unit[i][axis]);
    }
    let (mut a, mut b, mut best) = (extremes[0], extremes[1], -1.0);
    for (k, &i) in extremes.iter().enumerate() {
        for &j in &extremes[k + 1..] {
            let d = unit[i].distance_squared(unit[j]);
            if d > best {
                (a, b, best) = (i, j, d);
            }
        }
    }
    let ab = unit[b] - unit[a];
    let c = argmax(&|i| (unit[i] - unit[a]).cross(ab).length_squared());
    let normal = ab.cross(unit[c] - unit[a]);
    let d = argmax(&|i| normal.dot(unit[i] - unit[a]).abs());
    if orient(&fixed[a], &fixed[b], &fixed[c], &fixed[d]) == 0 {
        return Err(HullError::NotCovering);
    }
    Ok([a as u32, b as u32, c as u32, d as u32])
}

const PENDING: u8 = 0;
const INSERTED: u8 = 1;
const SKIPPED: u8 = 2;

/// The incremental hull under construction. Faces live in slots that are recycled through
/// `free`; each face heads a singly linked bucket (`head`, `next`) of the waiting points whose
/// cone from `center` it holds.
struct Builder<'a> {
    p: &'a [Fixed],
    center: Fixed,
    tri: Vec<[u32; 3]>,
    nbr: Vec<[u32; 3]>,
    alive: Vec<bool>,
    head: Vec<u32>,
    mark: Vec<u32>,
    free: Vec<u32>,
    next: Vec<u32>,
    state: Vec<u8>,
    bucket: Vec<u32>,
    vstamp: Vec<u32>,
    vstart: Vec<u32>,
    // Scratch, reused across insertions.
    stack: Vec<u32>,
    visible: Vec<u32>,
    horizon: Vec<[u32; 3]>,
    fan: Vec<u32>,
    orphans: Vec<u32>,
}

impl<'a> Builder<'a> {
    fn new(p: &'a [Fixed], tet: [u32; 4]) -> Result<Builder<'a>, HullError> {
        let n = p.len();
        let [a, mut b, mut c, d] = tet;
        let pt = |i: u32| &p[i as usize];
        if orient(pt(a), pt(b), pt(c), pt(d)) > 0 {
            std::mem::swap(&mut b, &mut c);
        }
        // With d below abc these four are counter-clockwise seen from outside.
        let faces = [[a, b, c], [a, d, b], [b, d, c], [c, d, a]];
        let mut nbr = [[NONE; 3]; 4];
        for (f, tf) in faces.iter().enumerate() {
            for (k, &u) in tf.iter().enumerate() {
                let w = tf[(k + 1) % 3];
                for (g, tg) in faces.iter().enumerate() {
                    if g != f && (0..3).any(|e| tg[e] == w && tg[(e + 1) % 3] == u) {
                        nbr[f][k] = g as u32;
                    }
                }
            }
        }
        let center: Fixed = std::array::from_fn(|k| {
            let s: i64 = tet.iter().map(|&v| p[v as usize][k]).sum();
            (s as f64 / 4.0).round() as i64
        });
        for (f, tf) in faces.iter().enumerate() {
            if nbr[f].contains(&NONE) {
                return Err(HullError::Internal("initial tetrahedron adjacency"));
            }
            let [x, y, z] = tf.map(pt);
            if orient(x, y, z, &center) >= 0 {
                // A tetrahedron too thin for its rounded centroid to be strictly inside.
                return Err(HullError::NotCovering);
            }
        }
        let cap = 2 * n + 8;
        let mut builder = Builder {
            p,
            center,
            tri: Vec::with_capacity(cap),
            nbr: Vec::with_capacity(cap),
            alive: Vec::with_capacity(cap),
            head: Vec::with_capacity(cap),
            mark: Vec::with_capacity(cap),
            free: Vec::new(),
            next: vec![NONE; n],
            state: vec![PENDING; n],
            bucket: vec![NONE; n],
            vstamp: vec![0; n],
            vstart: vec![NONE; n],
            stack: Vec::new(),
            visible: Vec::new(),
            horizon: Vec::new(),
            fan: Vec::new(),
            orphans: Vec::new(),
        };
        for (f, tf) in faces.iter().enumerate() {
            builder.tri.push(*tf);
            builder.nbr.push(nbr[f]);
            builder.alive.push(true);
            builder.head.push(NONE);
            builder.mark.push(0);
        }
        for &v in &tet {
            builder.state[v as usize] = INSERTED;
        }
        let assign: Vec<u32> = {
            let b = &builder;
            (0..n as u32)
                .into_par_iter()
                .map(|q| {
                    if b.state[q as usize] != PENDING {
                        NONE
                    } else {
                        (0..4u32)
                            .find(|&f| b.in_cone(f, &p[q as usize]))
                            .unwrap_or(0)
                    }
                })
                .collect()
        };
        for (q, &f) in assign.iter().enumerate() {
            if f != NONE {
                builder.push_bucket(f, q as u32);
            }
        }
        Ok(builder)
    }

    /// Whether point `q` lies strictly beyond the plane of face `f`.
    #[inline]
    fn sees(&self, f: u32, q: u32) -> bool {
        let [a, b, c] = self.tri[f as usize].map(|v| &self.p[v as usize]);
        orient(a, b, c, &self.p[q as usize]) > 0
    }

    /// Whether `q` lies in the closed cone from the centre over face `f`.
    #[inline]
    fn in_cone(&self, f: u32, q: &Fixed) -> bool {
        let [a, b, c] = self.tri[f as usize].map(|v| &self.p[v as usize]);
        let o = &self.center;
        orient(o, a, b, q) >= 0 && orient(o, b, c, q) >= 0 && orient(o, c, a, q) >= 0
    }

    fn push_bucket(&mut self, f: u32, q: u32) {
        self.next[q as usize] = self.head[f as usize];
        self.head[f as usize] = q;
        self.bucket[q as usize] = f;
    }

    fn alloc(&mut self, tri: [u32; 3], nbr: [u32; 3]) -> u32 {
        if let Some(f) = self.free.pop() {
            let i = f as usize;
            self.tri[i] = tri;
            self.nbr[i] = nbr;
            self.alive[i] = true;
            self.head[i] = NONE;
            self.mark[i] = 0;
            f
        } else {
            self.tri.push(tri);
            self.nbr.push(nbr);
            self.alive.push(true);
            self.head.push(NONE);
            self.mark.push(0);
            (self.tri.len() - 1) as u32
        }
    }

    /// Inserts point `p` in insertion round `round` (≥ 1; it stamps the faces and vertices
    /// this insertion touches).
    fn insert(&mut self, p: u32, round: u32) -> Result<(), HullError> {
        let f0 = self.bucket[p as usize];
        if !self.sees(f0, p) {
            self.state[p as usize] = SKIPPED;
            return Ok(());
        }
        self.state[p as usize] = INSERTED;
        let (vis, inv) = (2 * round, 2 * round + 1);

        // Flood-fill the faces p sees; record the horizon as (u, w, outside face) with u → w the
        // edge's direction in the visible face.
        let mut stack = std::mem::take(&mut self.stack);
        let mut visible = std::mem::take(&mut self.visible);
        let mut horizon = std::mem::take(&mut self.horizon);
        stack.clear();
        visible.clear();
        horizon.clear();
        self.mark[f0 as usize] = vis;
        stack.push(f0);
        while let Some(g) = stack.pop() {
            visible.push(g);
            let tg = self.tri[g as usize];
            for (k, &h) in self.nbr[g as usize].iter().enumerate() {
                let m = self.mark[h as usize];
                if m == vis {
                    continue;
                }
                if m != inv {
                    if self.sees(h, p) {
                        self.mark[h as usize] = vis;
                        stack.push(h);
                        continue;
                    }
                    self.mark[h as usize] = inv;
                }
                horizon.push([tg[k], tg[(k + 1) % 3], h]);
            }
        }

        // Empty the visible faces' buckets and free their slots.
        let mut orphans = std::mem::take(&mut self.orphans);
        orphans.clear();
        for &g in &visible {
            let mut q = self.head[g as usize];
            while q != NONE {
                if self.state[q as usize] == PENDING {
                    orphans.push(q);
                }
                q = self.next[q as usize];
            }
            self.head[g as usize] = NONE;
            self.alive[g as usize] = false;
            self.free.push(g);
        }

        // The fan: face (u, w, p) for every horizon edge u → w, glued to the outside face across
        // u → w and to its fan neighbours through the horizon vertices.
        let mut fan = std::mem::take(&mut self.fan);
        fan.clear();
        for &[u, w, h] in &horizon {
            let f = self.alloc([u, w, p], [h, NONE, NONE]);
            let th = self.tri[h as usize];
            let e =
                (0..3)
                    .find(|&e| th[e] == w && th[(e + 1) % 3] == u)
                    .ok_or(HullError::Internal(
                        "horizon edge missing from its outside face",
                    ))?;
            self.nbr[h as usize][e] = f;
            if self.vstamp[u as usize] == round {
                return Err(HullError::Internal("horizon is not a simple cycle"));
            }
            self.vstamp[u as usize] = round;
            self.vstart[u as usize] = f;
            fan.push(f);
        }
        for &f in &fan {
            let w = self.tri[f as usize][1];
            if self.vstamp[w as usize] != round {
                return Err(HullError::Internal("horizon is not closed"));
            }
            let g = self.vstart[w as usize];
            self.nbr[f as usize][1] = g;
            self.nbr[g as usize][2] = f;
        }

        // Rebucket: orphans of one face are neighbours, so start each search where the last hit.
        let mut last = 0usize;
        for &q in &orphans {
            let qp = self.p[q as usize];
            let m = fan.len();
            let hit = (0..m)
                .map(|s| (last + s) % m)
                .find(|&j| self.in_cone(fan[j], &qp))
                .ok_or(HullError::Internal("a waiting point left the fan's cone"))?;
            last = hit;
            self.push_bucket(fan[hit], q);
        }

        self.stack = stack;
        self.visible = visible;
        self.horizon = horizon;
        self.fan = fan;
        self.orphans = orphans;
        Ok(())
    }

    /// The live faces, compacted, as a mesh whose `points` and `fixed` the caller fills in.
    fn finish(self) -> Result<SphereMesh, HullError> {
        let n = self.p.len();
        let ids: Vec<u32> = (0..self.tri.len() as u32)
            .filter(|&f| self.alive[f as usize])
            .collect();
        let covers = ids.par_iter().all(|&f| {
            let [a, b, c] = self.tri[f as usize].map(|v| &self.p[v as usize]);
            orient(a, b, c, &ORIGIN) < 0
        });
        if !covers {
            return Err(HullError::NotCovering);
        }
        let mut new_id = vec![NONE; self.tri.len()];
        for (k, &f) in ids.iter().enumerate() {
            new_id[f as usize] = k as u32;
        }
        let triangles: Vec<[u32; 3]> = ids.iter().map(|&f| self.tri[f as usize]).collect();
        let neighbors: Vec<[u32; 3]> = ids
            .iter()
            .map(|&f| self.nbr[f as usize].map(|g| new_id[g as usize]))
            .collect();
        if neighbors.iter().flatten().any(|&g| g == NONE) {
            return Err(HullError::Internal("a neighbour points at a removed face"));
        }
        let mut vertex_triangle = vec![NONE; n];
        for (t, tri) in triangles.iter().enumerate() {
            for &v in tri {
                vertex_triangle[v as usize] = t as u32;
            }
        }
        if self.state.contains(&PENDING) {
            return Err(HullError::Internal("a point was never inserted"));
        }
        let skipped: Vec<u32> = (0..n as u32)
            .filter(|&v| self.state[v as usize] == SKIPPED)
            .collect();
        Ok(SphereMesh {
            points: Vec::new(),
            fixed: Vec::new(),
            triangles,
            neighbors,
            vertex_triangle,
            skipped,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sphere::fibonacci_sphere;

    /// V − E + F with E counted as distinct undirected edges (not from the 3F/2 identity).
    fn euler_characteristic(mesh: &SphereMesh) -> i64 {
        let mut edges: Vec<(u32, u32)> = mesh
            .triangles()
            .iter()
            .flat_map(|t| (0..3).map(move |k| (t[k].min(t[(k + 1) % 3]), t[k].max(t[(k + 1) % 3]))))
            .collect();
        edges.par_sort_unstable();
        edges.dedup();
        let mut verts: Vec<u32> = mesh.triangles().iter().flatten().copied().collect();
        verts.par_sort_unstable();
        verts.dedup();
        assert_eq!(
            verts.len(),
            mesh.vertex_count(),
            "every non-skipped point is a vertex"
        );
        assert_eq!(
            edges.len(),
            mesh.edge_count(),
            "every edge borders exactly two triangles"
        );
        verts.len() as i64 - edges.len() as i64 + mesh.triangles().len() as i64
    }

    /// Structural and Delaunay checks, all exact on the snapped coordinates.
    fn check_mesh(mesh: &SphereMesh) {
        let tris = mesh.triangles();
        let nbrs = mesh.neighbors();
        (0..tris.len()).into_par_iter().for_each(|t| {
            let tri = tris[t];
            assert!(
                tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2],
                "triangle {t} repeats a vertex"
            );
            let [a, b, c] = tri.map(|v| &mesh.fixed[v as usize]);
            assert_eq!(
                orient(a, b, c, &ORIGIN),
                -1,
                "triangle {t} does not face outward"
            );
            for (k, &g) in nbrs[t].iter().enumerate() {
                let g = g as usize;
                assert_ne!(g, t);
                let (u, w) = (tri[k], tri[(k + 1) % 3]);
                let back = (0..3).find(|&e| tris[g][e] == w && tris[g][(e + 1) % 3] == u);
                let e = back.unwrap_or_else(|| panic!("triangle {g} lacks edge {w}->{u} of {t}"));
                assert_eq!(
                    nbrs[g][e] as usize, t,
                    "neighbour links of {t} and {g} disagree"
                );
                // Locally convex = locally Delaunay: the far vertex of the neighbour is not above.
                let far = &mesh.fixed[tris[g][(e + 2) % 3] as usize];
                assert!(
                    orient(a, b, c, far) <= 0,
                    "edge {u}-{w} of triangle {t} is not Delaunay"
                );
            }
        });
    }

    #[test]
    fn fibonacci_triangulation_satisfies_euler() {
        for n in [12, 100, 1_000, 20_000, 100_000] {
            let pts = fibonacci_sphere(n);
            let mesh = SphereMesh::build(&pts).unwrap();
            assert!(
                mesh.skipped().is_empty(),
                "n={n}: skipped {:?}",
                mesh.skipped()
            );
            assert_eq!(mesh.triangles().len(), 2 * n - 4, "n={n}");
            assert_eq!(euler_characteristic(&mesh), 2, "n={n}: V - E + F");
            check_mesh(&mesh);
        }
    }

    #[test]
    fn random_points_triangulate_and_seeds_agree() {
        let mut rng = SplitMix64::new(11);
        let pts: Vec<DVec3> = (0..10_000).map(|_| rng.unit_vector()).collect();
        let a = SphereMesh::build_seeded(&pts, 1).unwrap();
        let b = SphereMesh::build_seeded(&pts, 2).unwrap();
        for mesh in [&a, &b] {
            assert!(mesh.skipped().is_empty());
            assert_eq!(euler_characteristic(mesh), 2);
            check_mesh(mesh);
        }
        // Random points are in general position: the Delaunay triangulation is unique.
        let canon = |m: &SphereMesh| {
            let mut t: Vec<[u32; 3]> = m
                .triangles()
                .iter()
                .map(|t| {
                    let k = (0..3).min_by_key(|&k| t[k]).unwrap();
                    [t[k], t[(k + 1) % 3], t[(k + 2) % 3]]
                })
                .collect();
            t.sort_unstable();
            t
        };
        assert_eq!(canon(&a), canon(&b));
    }

    /// A turned triangulation is the turned points' (P1-19): it faces outward and is locally
    /// Delaunay on the turned points' own snapped coordinates, and point location works on it.
    #[test]
    fn a_turned_triangulation_is_the_turned_points_own() {
        let mut rng = SplitMix64::new(23);
        for n in [1_000, 20_000, 100_000] {
            let mesh = SphereMesh::build(&fibonacci_sphere(n)).unwrap();
            for _ in 0..3 {
                let axis = rng.unit_vector();
                let q = glam::DQuat::from_axis_angle(axis, 0.3 + 2.0 * rng.unit_vector().x.abs());
                let pts: Vec<DVec3> = mesh.points().iter().map(|&p| (q * p).normalize()).collect();
                let turned = mesh.turned(pts.clone());
                assert_eq!(euler_characteristic(&turned), 2, "n={n}");
                check_mesh(&turned);
                for (i, &p) in pts.iter().enumerate().step_by(97) {
                    let t = turned.locate(p, 0);
                    assert!(
                        turned.triangles()[t as usize].contains(&(i as u32))
                            || turned.contains(t, p)
                    );
                }
            }
        }
    }

    /// Brute force on a small set: every point lies on or below every face's plane.
    #[test]
    fn small_hull_is_globally_convex() {
        let mut rng = SplitMix64::new(5);
        let pts: Vec<DVec3> = (0..300).map(|_| rng.unit_vector()).collect();
        let mesh = SphereMesh::build(&pts).unwrap();
        for tri in mesh.triangles() {
            let [a, b, c] = tri.map(|v| &mesh.fixed[v as usize]);
            for q in &mesh.fixed {
                assert!(orient(a, b, c, q) <= 0);
            }
        }
    }

    /// Degenerate input: cocircular points (cube corners, octahedron) and exact duplicates.
    #[test]
    fn degenerate_sets_still_triangulate() {
        let s = 1.0 / 3f64.sqrt();
        let cube: Vec<DVec3> = (0..8)
            .map(|i| {
                DVec3::new(
                    if i & 1 == 0 { s } else { -s },
                    if i & 2 == 0 { s } else { -s },
                    if i & 4 == 0 { s } else { -s },
                )
            })
            .collect();
        let octa = vec![
            DVec3::X,
            -DVec3::X,
            DVec3::Y,
            -DVec3::Y,
            DVec3::Z,
            -DVec3::Z,
        ];
        for pts in [cube, octa] {
            let mesh = SphereMesh::build(&pts).unwrap();
            assert!(mesh.skipped().is_empty());
            assert_eq!(mesh.triangles().len(), 2 * pts.len() - 4);
            assert_eq!(euler_characteristic(&mesh), 2);
            check_mesh(&mesh);
        }
        let mut pts = fibonacci_sphere(2_000);
        pts.extend_from_within(100..150);
        pts.push(pts[7] * 2.0); // same direction, normalised onto pts[7]
        let mesh = SphereMesh::build(&pts).unwrap();
        assert_eq!(mesh.skipped().len(), 51);
        assert_eq!(mesh.vertex_count(), 2_000);
        assert_eq!(euler_characteristic(&mesh), 2);
        check_mesh(&mesh);
    }

    #[test]
    fn bad_input_is_rejected() {
        assert_eq!(
            SphereMesh::build(&[DVec3::X; 3]).unwrap_err(),
            HullError::TooFewPoints(3)
        );
        let mut pts = fibonacci_sphere(10);
        pts[4] = DVec3::new(f64::NAN, 0.0, 1.0);
        assert_eq!(SphereMesh::build(&pts).unwrap_err(), HullError::BadPoint(4));
        pts[4] = DVec3::ZERO;
        assert_eq!(SphereMesh::build(&pts).unwrap_err(), HullError::BadPoint(4));
        // A cap of points (z > 0.2) does not surround the centre.
        let cap: Vec<DVec3> = fibonacci_sphere(1_000)
            .into_iter()
            .filter(|p| p.z > 0.2)
            .collect();
        assert_eq!(SphereMesh::build(&cap).unwrap_err(), HullError::NotCovering);
        // All on one great circle: coplanar.
        let ring: Vec<DVec3> = (0..20)
            .map(|i| {
                let a = i as f64 * 0.3;
                DVec3::new(a.cos(), a.sin(), 0.0)
            })
            .collect();
        assert_eq!(
            SphereMesh::build(&ring).unwrap_err(),
            HullError::NotCovering
        );
    }

    #[test]
    fn locate_finds_the_containing_triangle() {
        let pts = fibonacci_sphere(5_000);
        let mesh = SphereMesh::build(&pts).unwrap();
        let mut rng = SplitMix64::new(9);
        for k in 0..5_000 {
            let q = rng.unit_vector();
            // Far hints exercise long walks; every hint must still arrive.
            let t = mesh.locate(q, (k * 7919) as u32 % mesh.triangles().len() as u32);
            assert!(mesh.contains(t, q));
            let w = mesh.barycentric(t, q);
            assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-12);
            assert!(w.iter().all(|&x| x >= 0.0));
        }
        // Vertices locate onto a triangle that has them (weight 1 there).
        for v in [0u32, 17, 4_999] {
            let t = mesh.locate(pts[v as usize], 0);
            let k = mesh.triangles()[t as usize].iter().position(|&x| x == v);
            assert!(k.is_some());
            assert!((mesh.barycentric(t, pts[v as usize])[k.unwrap()] - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn vertex_neighbors_are_the_delaunay_edges() {
        let pts = fibonacci_sphere(3_000);
        let mesh = SphereMesh::build(&pts).unwrap();
        let adj = mesh.vertex_neighbors();
        assert_eq!(adj.offsets.len(), pts.len() + 1);
        assert_eq!(adj.neighbors.len(), 2 * mesh.edge_count());
        for v in 0..pts.len() as u32 {
            let row = adj.of(v);
            assert!(row.len() >= 3, "vertex {v} has degree {}", row.len());
            for &u in row {
                assert!(
                    adj.of(u).contains(&v),
                    "adjacency of {v} and {u} is not symmetric"
                );
            }
        }
    }

    #[test]
    fn brio_order_is_a_permutation() {
        let pts = fibonacci_sphere(10_000);
        let fixed: Vec<Fixed> = pts.iter().map(|&p| snap(p)).collect();
        let skip = [0, 1, 2, 3];
        let mut order = brio_order(&fixed, &skip, 1);
        assert_eq!(order.len(), 9_996);
        order.sort_unstable();
        assert_eq!(order, (4..10_000).collect::<Vec<u32>>());
        assert_eq!(morton(&[-(1 << 38); 3]), 0);
        assert_eq!(morton(&[1 << 38; 3]), (1 << 63) - 1);
    }

    #[test]
    fn orient_filter_matches_exact() {
        let mut rng = SplitMix64::new(21);
        for _ in 0..20_000 {
            let [a, b, c, d] = std::array::from_fn(|_| snap(rng.unit_vector()));
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let w = [d[0] - a[0], d[1] - a[1], d[2] - a[2]];
            assert_eq!(orient(&a, &b, &c, &d), orient_exact(&u, &v, &w));
        }
        // Nearly coplanar: d on the plane of abc up to one unit.
        let (a, b, c) = ([1i64 << 38, 0, 0], [0, 1i64 << 38, 0], [0, 0, 1i64 << 38]);
        let third = (1i64 << 38) / 3;
        for dz in -2..=2 {
            let d = [third, third, (1i64 << 38) - 2 * third + dz];
            assert_eq!(orient(&a, &b, &c, &d), dz.signum() as i32);
        }
    }
}

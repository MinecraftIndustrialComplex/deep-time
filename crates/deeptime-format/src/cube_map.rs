// SPDX-License-Identifier: LGPL-3.0-or-later
//! The cube world's map slot (docs/plans/cube-world-plan.md §1.6, work item M2): a face's local
//! point (u, v) ∈ [−1, 1]² to the unit sphere and back, from a table.
//!
//! Two tables ship with the crate, selected by name through the same slot ([`CubeMap::builtin`]):
//! - `conformal-n513` ([`WORLD_MAP`]): the conformal cube (τ = 0), the world's map, which its
//!   manifest names with the table's sha256;
//! - `moser-t1-n513` ([`SIMULATION_MAP`]): Moser's equal-area cube (τ = 1), the simulation's.
//!
//! Both come from `tools/cube-map/cube_map.py` (the file layout is in that script's docstring and
//! FORMAT.md "The cube map table"). The table holds one eighth of a face, the triangle
//! 0 ≤ v ≤ u ≤ 1, as the planet point in gnomonic coordinates of the face frame,
//! (a, b) = (y/x, z/x) with x along the face centre; the rest of the face is sign flips and a swap.
//!
//! # Arithmetic
//! Only +, −, ×, IEEE division and square root, in the order written here, with no fused
//! multiply-add: Java's `worlddata/CubeMap` and the pages' `DTCubeMap` (projection.js) give the same
//! bits (tests/data/projection_parity.json). Latitude and longitude go through
//! [`crate::portable_math`].
//!
//! # The evaluator
//! The forward is a Catmull-Rom bicubic over the table's nodes, padded by ghost nodes (beyond
//! u = 1 the mirror across the edge's plane, (a, b) → (1/a, b/a); beyond v = 1 its swap; past the
//! corner a linear extrapolation; beyond u = 0 and v = 0 the sign flips), with a = 1 written exactly
//! on the edge u = 1. The conformal map behaves like (z − z_c)^(4/3) at a cube corner, so its table
//! carries a corner patch (ledger SCL-104): a finer table over the K coarse cells next to the
//! corner, blended in by a smoothstep over the coarse cell next to it. The inverse starts from a
//! second table over the gnomonic triangle (bilinear) and takes [`NEWTON_STEPS`] Newton steps on
//! the forward bicubic.

use std::sync::{Arc, OnceLock};

use crate::cube::{Cube, CubeTurn, Face};
use crate::error::{FormatError, Result};
use crate::portable_math as pm;
use crate::projection::{lat_lon_of_unit, local_frame};

/// The table file's magic.
pub const MAGIC: &[u8; 4] = b"DTCM";
/// The table file's layout version (the header's second field).
pub const LAYOUT_VERSION: u32 = 1;
/// The header's size, bytes.
pub const HEADER_BYTES: usize = 112;

/// The world's map: the conformal cube (decisions log 2026-10-22, the study's pick;
/// ledger: SCL-101).
pub const WORLD_MAP: &str = "conformal-n513";
/// The simulation's map: Moser's equal-area cube on the polar axis (ledger: SCL-102).
pub const SIMULATION_MAP: &str = "moser-t1-n513";

/// Newton steps of the inverse after the inverse table's start: 4 take the round trip to 3e-12
/// block at 32k on the triangle and 8e-10 within 100 blocks of a corner (the plan's 2 leave
/// 0.005 and 0.029 block; review/out/cube-map/check-conformal-n513.json; ledger: SCL-103).
pub const NEWTON_STEPS: usize = 4;

/// The tables that ship with the crate: name, bytes, the `.sha256` file beside it.
const BUILTIN: [(&str, &[u8], &str); 2] = [
    (
        WORLD_MAP,
        include_bytes!("../data/cube-map-conformal-n513.bin"),
        include_str!("../data/cube-map-conformal-n513.bin.sha256"),
    ),
    (
        SIMULATION_MAP,
        include_bytes!("../data/cube-map-moser-t1-n513.bin"),
        include_str!("../data/cube-map-moser-t1-n513.bin.sha256"),
    ),
];

/// floor(x) clamped to [0, last] (0 for NaN), without `f64::floor`, a libm call on baseline
/// x86-64 that cost a third of the forward.
#[inline(always)]
fn floor_clamped(x: f64, last: usize) -> usize {
    let mut i = x as i64; // toward zero, saturating
    if (i as f64) > x {
        i -= 1;
    }
    i.clamp(0, last as i64) as usize
}

/// The faces' frames ([`Face::frame`]) by index, so the face picks a row and not a branch.
const FRAMES: [[[f64; 3]; 3]; 6] = [
    Face::F0.frame(),
    Face::F1.frame(),
    Face::F2.frame(),
    Face::F3.frame(),
    Face::N.frame(),
    Face::S.frame(),
];

/// A padded node grid over [−1, n + 1]², node (i, j) at index (j + 1)·side + (i + 1), so the four
/// nodes a bicubic row reads along i are adjacent.
#[derive(Clone, Debug)]
struct Grid {
    /// Cells a side.
    n: usize,
    /// n + 3.
    side: usize,
    nodes: Vec<[f64; 2]>,
}

/// The Catmull-Rom weights of four nodes p0 .. p3 at t ∈ [0, 1] (the cubic through p1 at t = 0
/// and p2 at t = 1, slopes (p2 − p0)/2 and (p3 − p1)/2): exactly (0, 1, 0, 0) at t = 0 and
/// (0, 0, 1, 0) at t = 1, so the bicubic is exact at its nodes.
#[inline(always)]
fn weights(t: f64) -> [f64; 4] {
    let t2 = t * t;
    let t3 = t2 * t;
    [
        0.5 * ((2.0 * t2 - t) - t3),
        0.5 * ((2.0 - 5.0 * t2) + 3.0 * t3),
        0.5 * ((t + 4.0 * t2) - 3.0 * t3),
        0.5 * (t3 - t2),
    ]
}

/// The weights' derivatives along t.
#[inline(always)]
fn dweights(t: f64) -> [f64; 4] {
    let t2 = t * t;
    [
        0.5 * ((4.0 * t - 1.0) - 3.0 * t2),
        0.5 * (9.0 * t2 - 10.0 * t),
        0.5 * ((1.0 + 8.0 * t) - 9.0 * t2),
        0.5 * (3.0 * t2 - 2.0 * t),
    ]
}

/// Σ w_k p_k on both components of four (a, b) nodes, summed from k = 0.
#[inline(always)]
fn mix(w: &[f64; 4], p: &[[f64; 2]]) -> [f64; 2] {
    [
        ((w[0] * p[0][0] + w[1] * p[1][0]) + w[2] * p[2][0]) + w[3] * p[3][0],
        ((w[0] * p[0][1] + w[1] * p[1][1]) + w[2] * p[2][1]) + w[3] * p[3][1],
    ]
}

impl Grid {
    fn new(n: usize) -> Grid {
        let side = n + 3;
        Grid {
            n,
            side,
            nodes: vec![[f64::NAN; 2]; side * side],
        }
    }

    #[inline]
    fn idx(&self, i: isize, j: isize) -> usize {
        (j + 1) as usize * self.side + (i + 1) as usize
    }

    #[inline]
    fn get(&self, i: isize, j: isize) -> [f64; 2] {
        self.nodes[self.idx(i, j)]
    }

    #[inline]
    fn set(&mut self, i: isize, j: isize, p: [f64; 2]) {
        let k = self.idx(i, j);
        self.nodes[k] = p;
    }

    /// The cell of grid coordinate x (node units): floor(x) clamped to [0, n − 1], and x's offset
    /// in it.
    #[inline(always)]
    fn cell(&self, x: f64) -> (usize, f64) {
        let i = floor_clamped(x, self.n - 1);
        (i, x - i as f64)
    }

    /// The four nodes of row j0 + dj − 1 from column i0 − 1.
    #[inline(always)]
    fn row(&self, i0: usize, j0: usize, dj: usize) -> &[[f64; 2]] {
        let b = (j0 + dj) * self.side + i0;
        &self.nodes[b..b + 4]
    }

    /// The bicubic at grid coordinates (x, y), node units: the cubic along x on each of the four
    /// rows, then along y.
    #[inline(always)]
    fn value(&self, x: f64, y: f64) -> [f64; 2] {
        let (i0, tu) = self.cell(x);
        let (j0, tv) = self.cell(y);
        let wu = weights(tu);
        let r = [0, 1, 2, 3].map(|dj| mix(&wu, self.row(i0, j0, dj)));
        mix(&weights(tv), &r)
    }

    /// The bicubic at (x, y) and its derivatives along x and y, per node unit.
    #[inline(always)]
    fn value_d(&self, x: f64, y: f64) -> Eval {
        let (i0, tu) = self.cell(x);
        let (j0, tv) = self.cell(y);
        let (wu, dwu) = (weights(tu), dweights(tu));
        let (wv, dwv) = (weights(tv), dweights(tv));
        let r = [0, 1, 2, 3].map(|dj| mix(&wu, self.row(i0, j0, dj)));
        let dr = [0, 1, 2, 3].map(|dj| mix(&dwu, self.row(i0, j0, dj)));
        Eval {
            val: mix(&wv, &r),
            du: mix(&wv, &dr),
            dv: mix(&dwv, &r),
        }
    }
}

/// The triangle's (a, b) at a point and its derivatives along u and v.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Eval {
    val: [f64; 2],
    du: [f64; 2],
    dv: [f64; 2],
}

/// The corner patch of a conformal table.
#[derive(Clone, Debug)]
struct Patch {
    /// K: coarse cells from the corner the patch covers.
    k: usize,
    /// F: fine cells to a coarse cell.
    f: usize,
    /// 1 − K/m: where the patch starts along u and v.
    u0: f64,
    /// m·F: fine cells per unit of u.
    mf: f64,
    grid: Grid,
}

/// A cube map table, ready to evaluate (the slot of plan §1.6).
#[derive(Clone, Debug)]
pub struct CubeMap {
    name: String,
    commit: String,
    tau: f64,
    flow_steps: u32,
    m: usize,
    mf: f64,
    q: usize,
    sha256: [u8; 32],
    coarse: Grid,
    patch: Option<Patch>,
    /// The inverse table over the whole gnomonic square [0, 1]² (the triangle and its swap):
    /// node (k, l) at k·(q + 1) + l.
    inv: Vec<[f64; 2]>,
}

/// A point on the unit sphere and its derivatives per unit of a face's u and v, in the cube
/// frame ([`CubeMap::jacobian`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FaceJacobian {
    pub p: [f64; 3],
    pub du: [f64; 3],
    pub dv: [f64; 3],
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn f64_at(b: &[u8], at: usize) -> f64 {
    let mut w = [0u8; 8];
    w.copy_from_slice(&b[at..at + 8]);
    f64::from_le_bytes(w)
}

fn ascii_field(b: &[u8], what: &str) -> Result<String> {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    let s = &b[..end];
    if !s.iter().all(|c| c.is_ascii_graphic()) {
        return Err(FormatError::Invalid(format!(
            "cube map table: the {what} is not printable ASCII"
        )));
    }
    Ok(String::from_utf8_lossy(s).into_owned())
}

/// Reads `count` (a, b) pairs of a triangle stored row by row (for i, for j ≤ i) from `at`.
fn triangle_pairs(b: &[u8], at: usize, rows: usize) -> Vec<Vec<[f64; 2]>> {
    let mut out = Vec::with_capacity(rows);
    let mut p = at;
    for i in 0..rows {
        let mut row = Vec::with_capacity(i + 1);
        for _ in 0..=i {
            row.push([f64_at(b, p), f64_at(b, p + 8)]);
            p += 16;
        }
        out.push(row);
    }
    out
}

impl CubeMap {
    /// A table from its file's bytes (layout in FORMAT.md "The cube map table").
    pub fn load(bytes: &[u8]) -> Result<CubeMap> {
        let bad = |why: String| FormatError::Invalid(format!("cube map table: {why}"));
        if bytes.len() < HEADER_BYTES || &bytes[..4] != MAGIC {
            return Err(bad("not a cube map table (magic DTCM)".into()));
        }
        let version = u32_at(bytes, 4);
        if version != LAYOUT_VERSION {
            return Err(bad(format!(
                "layout version {version}, this reader reads {LAYOUT_VERSION}"
            )));
        }
        let m = u32_at(bytes, 8) as usize;
        let q = u32_at(bytes, 12) as usize;
        let tau = f64_at(bytes, 16);
        let flow_steps = u32_at(bytes, 24);
        let k = u32_at(bytes, 28) as usize;
        let f = u32_at(bytes, 32) as usize;
        if !(2..=65_536).contains(&m) || !(1..=65_536).contains(&q) {
            return Err(bad(format!("m {m}, q {q}")));
        }
        if (k == 0) != (f == 0) || k >= m || f > 4_096 {
            return Err(bad(format!("corner patch K {k}, F {f} with m {m}")));
        }
        let name = ascii_field(&bytes[40..72], "name")?;
        let commit = ascii_field(&bytes[72..112], "generator commit")?;
        let nf = (m + 1) * (m + 2) / 2;
        let ni = (q + 1) * (q + 2) / 2;
        let ps = if k > 0 { k * f + 3 } else { 0 };
        let np = ps * (ps + 1) / 2;
        let want = HEADER_BYTES + 16 * (nf + ni + np);
        if bytes.len() != want {
            return Err(bad(format!("{} bytes, want {want}", bytes.len())));
        }
        let sha256 = crate::sha256::sha256(bytes);

        // The coarse table and its ghosts (cube_map.py `padded`).
        let tri = triangle_pairs(bytes, HEADER_BYTES, m + 1);
        let mut g = Grid::new(m);
        let mi = m as isize;
        for i in 0..=mi {
            for j in 0..=mi {
                let (hi, lo) = (i.max(j) as usize, i.min(j) as usize);
                let p = tri[hi][lo];
                g.set(i, j, if j <= i { p } else { [p[1], p[0]] });
            }
        }
        pad_edge_and_corner(&mut g, mi);
        // Beyond u = 0: (−a, b) of column 1, rows j ∈ [0, m + 1]; beyond v = 0: (a, −b) of row 1,
        // i ∈ [−1, m + 1].
        for j in 0..=mi + 1 {
            let p = g.get(1, j);
            g.set(-1, j, [-p[0], p[1]]);
        }
        for i in -1..=mi + 1 {
            let p = g.get(i, 1);
            g.set(i, -1, [p[0], -p[1]]);
        }
        if g.nodes
            .iter()
            .any(|p| !p[0].is_finite() || !p[1].is_finite())
        {
            return Err(bad("forward nodes not finite".into()));
        }

        // The inverse table, as a square.
        let itri = triangle_pairs(bytes, HEADER_BYTES + 16 * nf, q + 1);
        let mut inv = vec![[0.0; 2]; (q + 1) * (q + 1)];
        for kk in 0..=q {
            for ll in 0..=q {
                let p = itri[kk.max(ll)][kk.min(ll)];
                inv[kk * (q + 1) + ll] = if ll <= kk { p } else { [p[1], p[0]] };
            }
        }

        // The corner patch: every node of [−1, n + 1]² stored, the upper triangle by the swap.
        let patch = if k > 0 {
            let n = k * f;
            let ptri = triangle_pairs(bytes, HEADER_BYTES + 16 * (nf + ni), ps);
            let mut pg = Grid::new(n);
            let ni_ = n as isize;
            for i in -1..=ni_ + 1 {
                for j in -1..=ni_ + 1 {
                    let (hi, lo) = ((i.max(j) + 1) as usize, (i.min(j) + 1) as usize);
                    let p = ptri[hi][lo];
                    pg.set(i, j, if j <= i { p } else { [p[1], p[0]] });
                }
            }
            if pg
                .nodes
                .iter()
                .any(|p| !p[0].is_finite() || !p[1].is_finite())
            {
                return Err(bad("corner patch nodes not finite".into()));
            }
            Some(Patch {
                k,
                f,
                u0: 1.0 - k as f64 / m as f64,
                mf: (m * f) as f64,
                grid: pg,
            })
        } else {
            None
        };
        Ok(CubeMap {
            name,
            commit,
            tau,
            flow_steps,
            m,
            mf: m as f64,
            q,
            sha256,
            coarse: g,
            patch,
            inv,
        })
    }

    /// A table from its bytes, checked against the sha256 a manifest names (lowercase hex).
    pub fn load_checked(bytes: &[u8], sha256_hex: &str) -> Result<CubeMap> {
        let map = CubeMap::load(bytes)?;
        if map.sha256_hex() != sha256_hex.trim().to_ascii_lowercase() {
            return Err(FormatError::Invalid(format!(
                "cube map table {}: sha256 {}, the manifest names {}",
                map.name,
                map.sha256_hex(),
                sha256_hex.trim()
            )));
        }
        Ok(map)
    }

    /// The names of the tables that ship with the crate.
    pub fn builtin_names() -> [&'static str; 2] {
        [BUILTIN[0].0, BUILTIN[1].0]
    }

    /// A table that ships with the crate, by name ([`WORLD_MAP`], [`SIMULATION_MAP`]), loaded once
    /// and checked against its `.sha256` file. `None` for another name.
    pub fn builtin(name: &str) -> Option<Arc<CubeMap>> {
        static LOADED: [OnceLock<Arc<CubeMap>>; 2] = [OnceLock::new(), OnceLock::new()];
        let i = BUILTIN.iter().position(|b| b.0 == name)?;
        let (_, bytes, sum) = BUILTIN[i];
        Some(
            LOADED[i]
                .get_or_init(|| {
                    let hex = sum.split_whitespace().next().unwrap_or("");
                    let map = CubeMap::load_checked(bytes, hex)
                        .unwrap_or_else(|e| panic!("the crate's cube map table {name}: {e}"));
                    assert_eq!(map.name, name, "the table's header names it");
                    Arc::new(map)
                })
                .clone(),
        )
    }

    /// The world's map ([`WORLD_MAP`]).
    pub fn world() -> Arc<CubeMap> {
        CubeMap::builtin(WORLD_MAP).expect("the world's table ships with the crate")
    }

    /// The simulation's map ([`SIMULATION_MAP`]).
    pub fn simulation() -> Arc<CubeMap> {
        CubeMap::builtin(SIMULATION_MAP).expect("the simulation's table ships with the crate")
    }

    /// The table's name (`conformal-n513`, `moser-t1-n513`).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The generator's commit.
    pub fn commit(&self) -> &str {
        &self.commit
    }

    /// τ: 0 the conformal cube, 1 Moser's equal-area cube.
    pub fn tau(&self) -> f64 {
        self.tau
    }

    /// The RK4 steps of Moser's flow the generator took (0 for the conformal cube).
    pub fn flow_steps(&self) -> u32 {
        self.flow_steps
    }

    /// m: forward cells along the triangle's side (the face has 2m + 1 nodes a side).
    pub fn cells(&self) -> usize {
        self.m
    }

    /// q: the inverse table's cells along the gnomonic triangle's side.
    pub fn inverse_cells(&self) -> usize {
        self.q
    }

    /// The corner patch's (K, F), if the table has one.
    pub fn corner_patch(&self) -> Option<(usize, usize)> {
        self.patch.as_ref().map(|p| (p.k, p.f))
    }

    /// The file's sha256.
    pub fn sha256(&self) -> [u8; 32] {
        self.sha256
    }

    /// The file's sha256, lowercase hex.
    pub fn sha256_hex(&self) -> String {
        crate::sha256::hex(&self.sha256)
    }

    /// (a, b) on the triangle at (u, v), 0 ≤ v ≤ u ≤ 1, without the edge's exact a = 1 (a Newton
    /// iterate may step a hair outside the triangle).
    #[inline(always)]
    fn triangle_value_raw(&self, u: f64, v: f64) -> [f64; 2] {
        let val = |me: &CubeMap| me.coarse.value(u * me.mf, v * me.mf);
        let Some(p) = &self.patch else {
            return val(self);
        };
        // Coarse cells inward from the patch's edge (v is the smaller coordinate).
        let w = (v - p.u0) * self.mf;
        if w <= 0.0 {
            return val(self);
        }
        let fv = p.grid.value((u - p.u0) * p.mf, (v - p.u0) * p.mf);
        if w >= 1.0 {
            return fv;
        }
        let c = val(self);
        let s = w * w * (3.0 - 2.0 * w);
        [c[0] + s * (fv[0] - c[0]), c[1] + s * (fv[1] - c[1])]
    }

    /// (a, b) and its derivatives along u and v on the triangle, as [`Self::triangle_value_raw`].
    #[inline(always)]
    fn triangle_raw(&self, u: f64, v: f64) -> Eval {
        let scaled = |e: Eval, k: f64| Eval {
            val: e.val,
            du: [e.du[0] * k, e.du[1] * k],
            dv: [e.dv[0] * k, e.dv[1] * k],
        };
        let coarse = |me: &CubeMap| scaled(me.coarse.value_d(u * me.mf, v * me.mf), me.mf);
        let Some(p) = &self.patch else {
            return coarse(self);
        };
        let w = (v - p.u0) * self.mf;
        if w <= 0.0 {
            return coarse(self);
        }
        let fe = scaled(p.grid.value_d((u - p.u0) * p.mf, (v - p.u0) * p.mf), p.mf);
        if w >= 1.0 {
            return fe;
        }
        let ce = coarse(self);
        let s = w * w * (3.0 - 2.0 * w);
        let ds = 6.0 * w * (1.0 - w) * self.mf;
        let blend = |c: f64, f: f64| c + s * (f - c);
        Eval {
            val: [blend(ce.val[0], fe.val[0]), blend(ce.val[1], fe.val[1])],
            du: [blend(ce.du[0], fe.du[0]), blend(ce.du[1], fe.du[1])],
            dv: [
                blend(ce.dv[0], fe.dv[0]) + (fe.val[0] - ce.val[0]) * ds,
                blend(ce.dv[1], fe.dv[1]) + (fe.val[1] - ce.val[1]) * ds,
            ],
        }
    }

    /// (a, b) on the triangle at (u, v), with a = 1 exactly on the edge u = 1.
    #[inline(always)]
    pub fn triangle(&self, u: f64, v: f64) -> [f64; 2] {
        let mut ab = self.triangle_value_raw(u, v);
        if u == 1.0 {
            ab[0] = 1.0;
        }
        ab
    }

    /// The unit vector, cube frame, of face `face`'s local point (u, v) ∈ [−1, 1]².
    #[inline]
    pub fn forward(&self, face: Face, u: f64, v: f64) -> [f64; 3] {
        let (au, av) = (u.abs(), v.abs());
        let swap = av > au;
        // One call on selected arguments (not a call in each arm: one inlined copy, no branch).
        let (tu, tv) = if swap { (av, au) } else { (au, av) };
        let ab = self.triangle(tu, tv);
        let (a, b) = if swap { (ab[1], ab[0]) } else { (ab[0], ab[1]) };
        let ga = if u < 0.0 { -a } else { a };
        let gb = if v < 0.0 { -b } else { b };
        // One division: the reciprocal of r, then products (Java and JS do the same, bit for bit).
        let ir = 1.0 / ((1.0 + ga * ga) + gb * gb).sqrt();
        to_cube_frame(face, [ir, ga * ir, gb * ir])
    }

    /// The unit vector of (u, v) and its derivatives per unit of u and v, cube frame.
    pub fn jacobian(&self, face: Face, u: f64, v: f64) -> FaceJacobian {
        let (au, av) = (u.abs(), v.abs());
        let swap = av > au;
        let (tu, tv) = if swap { (av, au) } else { (au, av) };
        let mut e = self.triangle_raw(tu, tv);
        if tu == 1.0 {
            e.val[0] = 1.0;
        }
        let su = if u < 0.0 { -1.0 } else { 1.0 };
        let sv = if v < 0.0 { -1.0 } else { 1.0 };
        let sw = su * sv;
        // Through the fold: |u|, |v| and the swap (see the module's tests for the check).
        let (a, b, da_du, da_dv, db_du, db_dv) = if swap {
            (
                e.val[1],
                e.val[0],
                e.dv[1],
                sw * e.du[1],
                sw * e.dv[0],
                e.du[0],
            )
        } else {
            (
                e.val[0],
                e.val[1],
                e.du[0],
                sw * e.dv[0],
                sw * e.du[1],
                e.dv[1],
            )
        };
        let ga = su * a;
        let gb = sv * b;
        let ir = 1.0 / ((1.0 + ga * ga) + gb * gb).sqrt();
        let n = [ir, ga * ir, gb * ir];
        // d(g/r) = (dg − n (n·dg)) / r, with n·dg = (ga dga + gb dgb) / r.
        let dn = |dga: f64, dgb: f64| {
            let k = (ga * dga + gb * dgb) * ir;
            [-(n[0] * k) * ir, (dga - n[1] * k) * ir, (dgb - n[2] * k) * ir]
        };
        FaceJacobian {
            p: to_cube_frame(face, n),
            du: to_cube_frame(face, dn(da_du, db_du)),
            dv: to_cube_frame(face, dn(da_dv, db_dv)),
        }
    }

    /// The inverse table's bilinear start at (a, b) on the gnomonic triangle.
    #[inline]
    fn inverse_start(&self, a: f64, b: f64) -> (f64, f64) {
        let q = self.q;
        let (x, y) = (a * q as f64, b * q as f64);
        let (k, l) = (floor_clamped(x, q - 1), floor_clamped(y, q - 1));
        let (tx, ty) = (x - k as f64, y - l as f64);
        let row = q + 1;
        let n00 = self.inv[k * row + l];
        let n10 = self.inv[(k + 1) * row + l];
        let n01 = self.inv[k * row + l + 1];
        let n11 = self.inv[(k + 1) * row + l + 1];
        let u0 = n00[0] + tx * (n10[0] - n00[0]);
        let u1 = n01[0] + tx * (n11[0] - n01[0]);
        let w0 = n00[1] + tx * (n10[1] - n00[1]);
        let w1 = n01[1] + tx * (n11[1] - n01[1]);
        (u0 + ty * (u1 - u0), w0 + ty * (w1 - w0))
    }

    /// (u, v) on the triangle whose forward is (a, b), 0 ≤ b ≤ a ≤ 1: the inverse table's start,
    /// [`NEWTON_STEPS`] Newton steps, then clamped into the triangle and put back on its lines
    /// (a = 1 gives u = 1, b = 0 gives v = 0, a = b gives v = u).
    // Plain comparisons, not `clamp`: the order Java and JS follow, and no panic on a NaN bound.
    #[allow(clippy::manual_clamp)]
    pub fn triangle_inverse(&self, a: f64, b: f64) -> (f64, f64) {
        let (mut u, mut v) = self.inverse_start(a, b);
        for _ in 0..NEWTON_STEPS {
            let e = self.triangle_raw(u, v);
            let (ra, rb) = (a - e.val[0], b - e.val[1]);
            let det = e.du[0] * e.dv[1] - e.dv[0] * e.du[1];
            let nu = u + (e.dv[1] * ra - e.dv[0] * rb) / det;
            let nv = v + (e.du[0] * rb - e.du[1] * ra) / det;
            u = nu;
            v = nv;
        }
        if u > 1.0 {
            u = 1.0;
        }
        if u < 0.0 {
            u = 0.0;
        }
        if v > u {
            v = u;
        }
        if v < 0.0 {
            v = 0.0;
        }
        if a == 1.0 {
            u = 1.0;
        }
        if b == 0.0 {
            v = 0.0;
        }
        if a == b {
            v = u;
        }
        (u, v)
    }

    /// The face and local point (u, v) ∈ [−1, 1]² of the cube-frame vector `p` (any length > 0).
    /// The face is [`Face::of_vector`]'s (ties to x, then y, then z).
    pub fn inverse(&self, p: [f64; 3]) -> (Face, f64, f64) {
        let face = Face::of_vector(p);
        let [c, eu, ev] = FRAMES[face.index()];
        let x = (c[0] * p[0] + c[1] * p[1]) + c[2] * p[2];
        let y = (eu[0] * p[0] + eu[1] * p[1]) + eu[2] * p[2];
        let z = (ev[0] * p[0] + ev[1] * p[1]) + ev[2] * p[2];
        let (a, b) = (y / x, z / x);
        let (aa, bb) = (a.abs(), b.abs());
        let swap = bb > aa;
        let (ta, tb) = if swap { (bb, aa) } else { (aa, bb) };
        let (tu, tv) = self.triangle_inverse(ta, tb);
        let (u, v) = if swap { (tv, tu) } else { (tu, tv) };
        (
            face,
            if a < 0.0 { -u } else { u },
            if b < 0.0 { -v } else { v },
        )
    }
}

/// The ghosts beyond u = 1 and v = 1 and past the corner (cube_map.py `padded`, in its order).
fn pad_edge_and_corner(g: &mut Grid, m: isize) {
    // Beyond u = 1 (j < m): the mirror of node (m − 1, j) across the edge's plane.
    for j in 0..m {
        let p = g.get(m - 1, j);
        g.set(m + 1, j, [1.0 / p[0], p[1] / p[0]]);
    }
    // Beyond v = 1 (i < m): the swap of the above.
    for i in 0..m {
        let p = g.get(m + 1, i);
        g.set(i, m + 1, [p[1], p[0]]);
    }
    // Past the corner: linear along the edge, its swap, and linear along the diagonal.
    let (pmm, pm1, pd) = (g.get(m, m), g.get(m, m - 1), g.get(m - 1, m - 1));
    let e = [2.0 * pmm[0] - pm1[0], 2.0 * pmm[1] - pm1[1]];
    g.set(m, m + 1, e);
    g.set(m + 1, m, [e[1], e[0]]);
    g.set(m + 1, m + 1, [2.0 * pmm[0] - pd[0], 2.0 * pmm[1] - pd[1]]);
}

/// A face-frame vector (along the centre, e_u, e_v) in the cube frame.
#[inline(always)]
fn to_cube_frame(face: Face, w: [f64; 3]) -> [f64; 3] {
    let [c, eu, ev] = FRAMES[face.index()];
    [
        (c[0] * w[0] + eu[0] * w[1]) + ev[0] * w[2],
        (c[1] * w[0] + eu[1] * w[1]) + ev[1] * w[2],
        (c[2] * w[0] + eu[2] * w[1]) + ev[2] * w[2],
    ]
}

#[inline]
fn dot(p: [f64; 3], q: [f64; 3]) -> f64 {
    (p[0] * q[0] + p[1] * q[1]) + p[2] * q[2]
}

/// The planet's metres per block at a point of the cube world ([`CubeWorld::jacobian`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldJacobian {
    /// [[east, north] per block of net +x, [east, north] per block of net +z], planet metres
    /// (the layout of `PetroffGuyou::jacobian`). For the conformal map a scalar times a rotation
    /// and a reflection (net +z is south-ish).
    pub m: [[f64; 2]; 2],
    /// √|det m|: planet metres per block (the conformal map's one scale; for another map the
    /// geometric mean of its two).
    pub sigma: f64,
    /// The direction of net +x on the planet, radians from east toward north
    /// (atan2(m[0][1], m[0][0]), the portable atan2).
    pub angle: f64,
}

/// A cube world: the net, the map table, the turn that places the cube on the planet and the
/// planet's size. Latitude, the poles and every metres-per-block figure come from here, never from
/// the net alone (coordinator, 2026-10-22: the world's turn is free).
#[derive(Clone, Debug)]
pub struct CubeWorld {
    cube: Cube,
    map: Arc<CubeMap>,
    turn: CubeTurn,
    radius_m: f64,
}

impl CubeWorld {
    /// The world of net `cube`, map `map` and turn `turn`, at `m_per_block_h` nominal metres per
    /// block: the planet's radius in metres is m_per_block_h × [`Cube::radius`] (the six faces'
    /// area is the sphere's).
    pub fn new(cube: Cube, map: Arc<CubeMap>, turn: CubeTurn, m_per_block_h: f64) -> CubeWorld {
        let radius_m = m_per_block_h * cube.radius();
        CubeWorld {
            cube,
            map,
            turn,
            radius_m,
        }
    }

    pub fn cube(&self) -> &Cube {
        &self.cube
    }

    pub fn map(&self) -> &CubeMap {
        &self.map
    }

    pub fn turn(&self) -> CubeTurn {
        self.turn
    }

    /// The planet's radius, metres.
    pub fn radius_m(&self) -> f64 {
        self.radius_m
    }

    /// The planet unit vector of a face's local point.
    #[inline]
    pub fn unit_on_face(&self, face: Face, u: f64, v: f64) -> [f64; 3] {
        self.turn.to_planet(self.map.forward(face, u, v))
    }

    /// The planet unit vector the net point (x, z) shows (through [`Cube::fold_point`]: x wraps,
    /// a point beyond a net edge shows the point it is glued to); `None` past the net's corners.
    pub fn unit(&self, x: f64, z: f64) -> Option<[f64; 3]> {
        let p = self.cube.fold_point(x, z)?;
        Some(self.unit_on_face(p.face, p.u, p.v))
    }

    /// The face and local point of the planet unit vector `p`.
    pub fn local_of_unit(&self, p: [f64; 3]) -> (Face, f64, f64) {
        self.map.inverse(self.turn.to_cube(p))
    }

    /// The net point (in the window) of the planet unit vector `p`.
    pub fn point_of_unit(&self, p: [f64; 3]) -> (f64, f64) {
        let (face, u, v) = self.local_of_unit(p);
        self.cube.point_of_local(face, u, v)
    }

    /// (latitude, longitude) in degrees of the net point (x, z), longitude in [0, 360).
    pub fn lat_lon(&self, x: f64, z: f64) -> Option<(f64, f64)> {
        self.unit(x, z).map(lat_lon_of_unit)
    }

    /// The net point of (latitude, longitude), degrees.
    pub fn point_of_lat_lon(&self, lat_deg: f64, lon_deg: f64) -> (f64, f64) {
        self.point_of_unit(unit_of_lat_lon_pm(lat_deg, lon_deg))
    }

    /// Metres per block at a face's local point ([`WorldJacobian`]).
    pub fn jacobian_on_face(&self, face: Face, u: f64, v: f64) -> WorldJacobian {
        let j = self.map.jacobian(face, u, v);
        let p = self.turn.to_planet(j.p);
        let (du, dv) = (self.turn.to_planet(j.du), self.turn.to_planet(j.dv));
        let (east, north) = local_frame(p);
        // du/dx = dv/dz = 2/s.
        let k = self.radius_m / (self.cube.side() as f64 / 2.0);
        let m = [
            [dot(du, east) * k, dot(du, north) * k],
            [dot(dv, east) * k, dot(dv, north) * k],
        ];
        world_jacobian(m)
    }

    /// Metres per block at the net point (x, z), through the fold (beyond a net edge the motion's
    /// turn is applied, so the rows stay per block of the queried point's +x and +z); `None` past
    /// the net's corners.
    pub fn jacobian(&self, x: f64, z: f64) -> Option<WorldJacobian> {
        let p = self.cube.fold_point(x, z)?;
        let j = self.jacobian_on_face(p.face, p.u, p.v);
        if p.iso.k == 0 {
            return Some(j);
        }
        let row = |dx: f64, dz: f64| {
            let (vx, vz) = p.iso.vector(dx, dz);
            [
                j.m[0][0] * vx + j.m[1][0] * vz,
                j.m[0][1] * vx + j.m[1][1] * vz,
            ]
        };
        Some(world_jacobian([row(1.0, 0.0), row(0.0, 1.0)]))
    }

    /// The unit vector of north in block space at the net point (x, z), as (dx, dz).
    pub fn north(&self, x: f64, z: f64) -> Option<(f64, f64)> {
        let j = self.jacobian(x, z)?.m;
        // Solve J·(dx, dz) = (0, 1): dx·j[0] + dz·j[1] = (east 0, north 1).
        let det = j[0][0] * j[1][1] - j[1][0] * j[0][1];
        if det == 0.0 || !det.is_finite() {
            return Some((0.0, -1.0));
        }
        let (dx, dz) = (-j[1][0] / det, j[0][0] / det);
        let n = (dx * dx + dz * dz).sqrt();
        Some((dx / n, dz / n))
    }

    /// The north pole's face and local point, found through the turn.
    pub fn north_pole(&self) -> (Face, f64, f64) {
        self.local_of_unit([0.0, 0.0, 1.0])
    }

    /// The south pole's face and local point, found through the turn.
    pub fn south_pole(&self) -> (Face, f64, f64) {
        self.local_of_unit([0.0, 0.0, -1.0])
    }

    /// The planet unit vector of a cube corner ([`crate::cube::Corner::cube`]).
    pub fn corner_unit(&self, cube: [i8; 3]) -> [f64; 3] {
        let r = 3.0f64.sqrt();
        self.turn.to_planet([
            f64::from(cube[0]) / r,
            f64::from(cube[1]) / r,
            f64::from(cube[2]) / r,
        ])
    }

    /// (latitude, longitude) in degrees of a cube corner.
    pub fn corner_lat_lon(&self, cube: [i8; 3]) -> (f64, f64) {
        lat_lon_of_unit(self.corner_unit(cube))
    }
}

fn world_jacobian(m: [[f64; 2]; 2]) -> WorldJacobian {
    let det = m[0][0] * m[1][1] - m[0][1] * m[1][0];
    WorldJacobian {
        m,
        sigma: det.abs().sqrt(),
        angle: pm::atan2(m[0][1], m[0][0]),
    }
}

/// The unit vector of (latitude, longitude) in degrees, on the portable functions.
pub fn unit_of_lat_lon_pm(lat_deg: f64, lon_deg: f64) -> [f64; 3] {
    let (phi, lam) = (pm::to_radians(lat_deg), pm::to_radians(lon_deg));
    let (sp, cp) = (pm::sin(phi), pm::cos(phi));
    [cp * pm::cos(lam), cp * pm::sin(lam), sp]
}

#[cfg(test)]
mod tests;

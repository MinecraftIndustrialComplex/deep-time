// SPDX-License-Identifier: LGPL-3.0-or-later
//! The bake's projection pass for a cube world (format 0.7; cube plan phase 1, items B1 to B10;
//! FORMAT.md "The bake on a cube world"; science page `docs/science/cube-world-bake.md`). The
//! simulation runs on its Lambert grid as it always has; this pass rewrites the world folder on the
//! six faces of the conformal cube (`conformal-n513`), turned on the planet so its eight corners
//! lie as far from land as the turn search finds ([`crate::bake::cube_turn`]). Ben, 2026-10-22: the
//! rasters face by face with no void cells, the map table shipped in the code and named with its
//! sha256 in the manifest (Q1), the conformal cube turned freely (Q4).
//!
//! - **Rasters** ([`resample_cube`]): every world cell is a point sample at its centre. The
//!   cell's footprint on the simulation's grid comes from the composed Jacobian (world cell →
//!   planet → Lambert): where it spans more than one simulation cell along an axis, a Gaussian
//!   along that axis averages it; else the bilinear read alone (SCL-124). This replaces the
//!   Petroff–Guyou pass's polar widening (SCL-090) on the cube. Categories, horizon stacks and
//!   cover take the nearest cell as there.
//! - **Topology**: the closed basins, the biomes' fills and distances and the relief run on the
//!   cube's own neighbours, across every seam, with great-circle distances (SCL-122, SCL-123).
//! - **Vector data**: positions through the inverse map, directions and sizes through the
//!   analytic Jacobian ([`CubeMapping`]); river chains that cross a turning seam are cut on the
//!   seam into partner halves ([`Mouth::Seam`], SCL-126).

use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};
use std::f64::consts::{PI, TAU};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use rayon::prelude::*;
use serde_json::{Value, json};

use deeptime_format::bodies::{read_bodies, write_bodies};
use deeptime_format::column::CLOSED_BASIN;
use deeptime_format::cube::{Cube, Face};
use deeptime_format::cube_map::{CubeMap, CubeView, WORLD_MAP};
use deeptime_format::fsutil::Flusher;
use deeptime_format::projection::{
    Axis, IndexFrame, Projection, Side, WorldProjection, cube_border, cube_spec, face_for,
    local_frame,
};
use deeptime_format::raster::{RasterRead, cover_of};
use deeptime_format::rivers::{Chain, Mouth, RiverVertex, Rivers, read_rivers, write_rivers};
use deeptime_format::sites::{read_sites, write_sites_in};
use deeptime_format::structure::{read_structure, write_structure};
use deeptime_format::{FileEntry, GridDesc, LayerDesc, LoadedLayer, Manifest};

use crate::bake::cube_turn;
use crate::bake::load_layer;
use crate::bake::project::{
    BORDER_INSET_BLOCKS, BiomeGround, BodyY, Drawn, Held, Kind, ROW_BAND, SPAWN_EDGE_MARGIN, Src,
    VERSION, WorldMap, apply, area_share, body_y, drawn_detail, err, is_biome_layer, kind_of, ms,
    project_body, project_site, project_structure, put_raw, raise_world_tops, settle_biome_layers,
    stratabound_sites, world_ground, world_raise_shifts, write_deferred, write_json,
};
use crate::bake::relief::{Frame, SimFrame};
use crate::error::CoreError;
use crate::hydro;
use crate::params::{EdgeBorder, LAMBDA_C_AUTO, PlanetPreset};

/// Below this Gaussian width, in simulation cells, an axis of a footprint takes the bilinear read
/// alone (the Gaussian would change nothing a reader sees).
pub const FOOT_MIN_SIGMA_CELLS: f64 = 0.25; // ledger: SCL-124 (game-choice)

/// The widest Gaussian along a footprint's long axis, as a share of the simulation's
/// circumference: at the simulation's poles a Lambert row is one planet point, so the long axis
/// grows without bound; ±3 of these widths cover the whole row.
pub const FOOT_MAX_SIGMA_SHARE: f64 = 1.0 / 12.0; // ledger: SCL-124 (game-choice)

/// The most samples a footprint takes each side of its centre along its long and short axes
/// (2,049 and 65 in all; the Petroff–Guyou pass's 2,048 and 64).
const FOOT_MAX_STEPS: (i64, i64) = (1024, 32);

/// The bilinear read's response to a checkerboard at the simulation's resolution, projected on
/// cos πu · cos πv: (8/π²)², the footprint record's leak with no Gaussian.
const BILINEAR_CHECKER: f64 = 0.657; // ledger: SCL-124 ((8/π²)², derived)

/// The footprint record's bands of simulation cells per world cell along the long axis.
const D_BANDS: [f64; 6] = [1.0, 1.1, 1.2, 1.5, 2.0, 4.0];

/// No spawn candidate within this many blocks of a cube corner's image in the net (where the map
/// squeezes the planet to a point and three faces meet).
pub const SPAWN_CORNER_MARGIN: f64 = 256.0; // ledger: SCL-127 (game-choice)

/// The longest step a river chain takes in the world before it is cut as at the sea, blocks (the
/// Petroff–Guyou pass's).
const RIVER_JUMP_BLOCKS: f64 = 256.0;

#[inline]
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn normalize(a: [f64; 3]) -> [f64; 3] {
    let n = dot(a, a).sqrt();
    [a[0] / n, a[1] / n, a[2] / n]
}

/// The great-circle angle between two unit vectors, radians (by the chord: exact near 0).
#[inline]
fn angle(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    2.0 * (0.5 * dot(d, d).sqrt()).min(1.0).asin()
}

// --- the seams -------------------------------------------------------------------------------------

/// `f` at net point (x, z), blended across the cube's turning seams (ledger: SCL-125): within
/// `reach` blocks of a turning net edge, inside its face, `f` is mixed with its value at the point
/// beyond the partner edge that shows the same planet point (where the other face's net
/// coordinates go on), half and half on the seam and fading to the point's own value at `reach`
/// (a smoothstep). Both faces then read the same value on the seam, so a pattern drawn in net
/// coordinates (noise) runs on across it. Away from the turning seams, and past the net, `f` alone.
/// Near a corner the nearer edge's partner is taken.
pub fn seam_blend(cube: &Cube, x: f64, z: f64, reach: f64, f: impl Fn(f64, f64) -> f64) -> f64 {
    let here = f(x, z);
    if reach <= 0.0 {
        return here;
    }
    let xw = cube.wrap_x_point(x);
    let edges = cube.edges();
    let mut best: Option<(usize, f64)> = None;
    for (k, e) in edges.iter().enumerate().skip(2) {
        let (n, t) = match e.axis {
            Axis::X => (xw, z),
            Axis::Z => (z, xw),
        };
        if t < e.span[0] as f64 || t > e.span[1] as f64 {
            continue;
        }
        // How far inside the face (the planet's side of the line).
        let d = match e.beyond {
            Side::Min => n - e.coord as f64,
            Side::Max => e.coord as f64 - n,
        };
        if (0.0..reach).contains(&d) && best.is_none_or(|(_, bd)| d < bd) {
            best = Some((k, d));
        }
    }
    let Some((k, d)) = best else {
        return here;
    };
    let (px, pz) = edges[edges[k].partner].iso.inverse().point(xw, z);
    let t = d / reach;
    let w = 0.5 * (1.0 - t * t * (3.0 - 2.0 * t));
    (1.0 - w) * here + w * f(px, pz)
}

/// Which seam a step between two faces crosses: `direct` (neighbours in the net: F3|F0, F0|F1,
/// F1|F2, N|F0, S|F0), `slide` (F2|F3, the east-west wrap) or `turning` (glued by a quarter or a
/// half turn), or `none` on one face.
pub fn seam_class(a: Face, b: Face) -> &'static str {
    use Face::*;
    if a == b {
        return "none";
    }
    match (a.min(b), a.max(b)) {
        (F0, F1) | (F1, F2) | (F0, F3) | (F0, N) | (F0, S) => "direct",
        (F2, F3) => "slide",
        _ => "turning",
    }
}

// --- the biomes' fills and distances (SCL-122) -----------------------------------------------------

/// The nearest source of each cell, for a set of sources: (source cell or u32::MAX, chord²).
type Nearest = Arc<(Vec<u32>, Vec<f64>)>;

/// The biome land call's fills and distances on a cube grid ([`crate::bake::biomes::settle_to_ground_by`]):
/// each cell takes its nearest source by great-circle distance, found by a multi-source Dijkstra
/// over the grid's 8 neighbours across every seam ([`hydro::cube_neighbour8`]) that carries each
/// cell's source and keys it by the true angle to it (ledger: SCL-122).
pub(crate) struct CubeSettle {
    grid: GridDesc,
    units: Vec<[f64; 3]>,
    /// Cells per radian: the simulation's radius over the bake blocks a world cell spans.
    scale: f64,
    /// The last few source sets and their answers (the land call asks for the same two donor sets
    /// a dozen times).
    cache: RefCell<Vec<(Vec<bool>, Nearest)>>,
}

impl CubeSettle {
    /// `scale`: cells per radian of the planet.
    pub(crate) fn new(grid: &GridDesc, view: &CubeView<'_>, scale: f64) -> CubeSettle {
        let n = grid.w as usize * grid.h as usize;
        let w = grid.w as usize;
        let half = f64::from(grid.face_blocks()) / 2.0;
        let r = f64::from(grid.r);
        let units = (0..n)
            .into_par_iter()
            .map(|c| {
                let (face, a, b) = grid.face_of_cell((c % w) as u32, (c / w) as u32);
                let u = ((f64::from(a) + 0.5) * r - half) / half;
                let v = ((f64::from(b) + 0.5) * r - half) / half;
                view.unit_on_face(face, u, v)
            })
            .collect();
        CubeSettle {
            grid: grid.clone(),
            units,
            scale,
            cache: RefCell::new(Vec::new()),
        }
    }

    fn nearest(&self, src: &[bool]) -> Option<Nearest> {
        if !src.iter().any(|&s| s) {
            return None;
        }
        if let Some((_, r)) = self
            .cache
            .borrow()
            .iter()
            .find(|(k, _)| k.as_slice() == src)
        {
            return Some(r.clone());
        }
        let n = src.len();
        let mut from = vec![u32::MAX; n];
        let mut key = vec![f64::INFINITY; n];
        let mut heap = BinaryHeap::new();
        for (c, _) in src.iter().enumerate().filter(|(_, s)| **s) {
            from[c] = c as u32;
            key[c] = 0.0;
            heap.push(Reverse((0u64, c as u32)));
        }
        let nb = hydro::cube_neighbour8(&self.grid);
        while let Some(Reverse((kb, c))) = heap.pop() {
            let c = c as usize;
            // Keys are non-negative, so their bits order as the numbers do.
            if f64::from_bits(kb) > key[c] {
                continue;
            }
            let s = self.units[from[c] as usize];
            for k in 0..8 {
                let Some(m) = nb(c, k) else {
                    continue;
                };
                let um = self.units[m];
                let d = [um[0] - s[0], um[1] - s[1], um[2] - s[2]];
                let d2 = dot(d, d);
                if d2 < key[m] {
                    key[m] = d2;
                    from[m] = from[c];
                    heap.push(Reverse((d2.to_bits(), m as u32)));
                }
            }
        }
        let r: Nearest = Arc::new((from, key));
        let mut cache = self.cache.borrow_mut();
        if cache.len() >= 2 {
            cache.remove(0);
        }
        cache.push((src.to_vec(), r.clone()));
        Some(r)
    }

    /// Every cell takes the value of its nearest source (`src`); false with no source.
    pub(crate) fn fill(&self, v: &mut [f32], src: &[bool]) -> bool {
        let Some(r) = self.nearest(src) else {
            return false;
        };
        // A source is its own nearest, so its value stays as it was while the others copy it.
        for (c, &s) in r.0.iter().enumerate() {
            if s != u32::MAX {
                v[c] = v[s as usize];
            }
        }
        true
    }

    /// Each cell's distance, in cells, to the nearest cell whose `land` differs from `to_sea`
    /// (the land's distance to the sea when `to_sea`), on the planet; infinite with none.
    pub(crate) fn dist(&self, land: &[bool], to_sea: bool) -> Vec<f32> {
        let src: Vec<bool> = land.iter().map(|&l| l != to_sea).collect();
        let Some(r) = self.nearest(&src) else {
            return vec![f32::INFINITY; land.len()];
        };
        r.1.iter()
            .map(|&d2| {
                if d2.is_finite() {
                    (2.0 * (0.5 * d2.sqrt()).min(1.0).asin() * self.scale) as f32
                } else {
                    f32::INFINITY
                }
            })
            .collect()
    }
}

// --- the mapping -------------------------------------------------------------------------------------

/// How an image of a world point was reached ([`CubeMapping::nearest_image`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Image {
    /// The point itself.
    Direct,
    /// A lap east or west (the slide).
    Slide,
    /// Beyond turning net edge k ([`Cube::edges`]).
    Turn(usize),
}

/// Where a world cell samples the simulation, and how wide (SCL-124).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Foot {
    /// The Lambert block of the cell's centre.
    pub(crate) xs: f64,
    pub(crate) zs: f64,
    /// Simulation cells per world cell along the footprint's long and short axes.
    pub(crate) d: (f64, f64),
    /// The Gaussian's widths along them, simulation blocks (0: the bilinear read alone).
    pub(crate) sigma: (f64, f64),
    /// The long axis, a unit vector in Lambert blocks.
    pub(crate) e1: (f64, f64),
}

impl Foot {
    /// The share of a checkerboard at the simulation's resolution that the footprint passes:
    /// the bilinear read's [`BILINEAR_CHECKER`] times the Gaussian's mean response to the
    /// checkerboard's two diagonal waves, cos πu cos πv = ½ (cos π(u + v) + cos π(u − v)).
    pub(crate) fn checker_leak(&self, r_s: f64) -> f64 {
        let e2 = (-self.e1.1, self.e1.0);
        let response = |sz: f64| {
            let f1 = (self.e1.0 + sz * self.e1.1) / (2.0 * r_s);
            let f2 = (e2.0 + sz * e2.1) / (2.0 * r_s);
            let ex = 2.0 * PI * PI * ((self.sigma.0 * f1).powi(2) + (self.sigma.1 * f2).powi(2));
            (-ex).exp()
        };
        BILINEAR_CHECKER * 0.5 * (response(1.0) + response(-1.0))
    }

    /// The footprint's samples (Lambert x, z, weight; weights summing to 1) into `out`.
    pub(crate) fn samples(&self, r_s: f64, out: &mut Vec<(f64, f64, f64)>) {
        out.clear();
        let axis = |s: f64, cap: i64| -> (i64, f64) {
            if s <= 0.0 {
                return (0, 0.0);
            }
            let m = ((3.0 * s / s.min(r_s)).ceil() as i64).clamp(1, cap);
            (m, 3.0 * s / m as f64)
        };
        let (m1, h1) = axis(self.sigma.0, FOOT_MAX_STEPS.0);
        let (m2, h2) = axis(self.sigma.1, FOOT_MAX_STEPS.1);
        let e2 = (-self.e1.1, self.e1.0);
        let wt = |a: f64, s: f64| {
            if s > 0.0 {
                (-a * a / (2.0 * s * s)).exp()
            } else {
                1.0
            }
        };
        let mut total = 0.0;
        for i in -m1..=m1 {
            let a1 = i as f64 * h1;
            let w1 = wt(a1, self.sigma.0);
            for k in -m2..=m2 {
                let a2 = k as f64 * h2;
                let w = w1 * wt(a2, self.sigma.1);
                out.push((
                    self.xs + a1 * self.e1.0 + a2 * e2.0,
                    self.zs + a1 * self.e1.1 + a2 * e2.1,
                    w,
                ));
                total += w;
            }
        }
        for s in out.iter_mut() {
            s.2 /= total;
        }
    }
}

/// The cube world → simulation mapping of the pass: the net, the cube's view with Jacobians in
/// the simulation's blocks (σ: simulation blocks per world block) and the simulation's Lambert
/// projection.
#[derive(Clone, Copy)]
pub(crate) struct CubeMapping {
    pub(crate) cube: &'static Cube,
    /// The world's view, its radius the simulation's (blocks).
    pub(crate) view: CubeView<'static>,
    pub(crate) src: Projection,
    /// R of the simulation, blocks.
    pub(crate) rs: f64,
}

impl CubeMapping {
    pub(crate) fn new(cube: &'static Cube, view: CubeView<'static>, c_sim: u32) -> CubeMapping {
        let src = Projection::new(c_sim);
        let rs = src.radius();
        CubeMapping {
            cube,
            view: CubeView {
                radius_m: rs,
                ..view
            },
            src,
            rs,
        }
    }

    /// The Lambert block of a unit vector (x in [−C/2, C/2), z = −R sin φ) and cos φ.
    #[inline]
    pub(crate) fn src_block(&self, v: [f64; 3]) -> (f64, f64, f64) {
        let c = self.src.circumference();
        let lon = v[1].atan2(v[0]).to_degrees();
        let xs = self.src.wrap_x((lon / 360.0) * c - 0.5 * c);
        let zs = -self.rs * v[2].clamp(-1.0, 1.0);
        (xs, zs, v[0].hypot(v[1]))
    }

    /// The world point (in the net's window) of simulation block (xs, zs).
    pub(crate) fn world_at(&self, xs: f64, zs: f64) -> (f64, f64) {
        self.view.point_of_unit(self.src.unit(xs, zs))
    }

    /// The image of world point `q` (canonical) nearest to `p`: `q` itself, a lap east or west,
    /// or the point beyond a turning net edge that shows it (in the edge's band, a face deep, or
    /// on its line: a seam point is on both faces' edges).
    pub(crate) fn nearest_image(&self, p: (f64, f64), q: (f64, f64)) -> ((f64, f64), Image) {
        let per = self.cube.period() as f64;
        let edges = self.cube.edges();
        let mut cands: [((f64, f64), Image); 39] = [((0.0, 0.0), Image::Direct); 39];
        let mut n = 0;
        for k in [0.0, -1.0, 1.0] {
            let kind = if k == 0.0 {
                Image::Direct
            } else {
                Image::Slide
            };
            cands[n] = ((q.0 + k * per, q.1), kind);
            n += 1;
        }
        for (ei, e) in edges.iter().enumerate().skip(2) {
            let b = e.iso.inverse().point(q.0, q.1);
            for k in [0.0, -1.0, 1.0] {
                cands[n] = ((b.0 + k * per, b.1), Image::Turn(ei));
                n += 1;
            }
        }
        let d = |c: (f64, f64)| (c.0 - p.0).hypot(c.1 - p.1);
        let c = &mut cands[..n];
        c.sort_by(|a, b| d(a.0).total_cmp(&d(b.0)));
        let side = self.cube.side() as f64;
        let tol = 1e-6 * side;
        for &(pt, kind) in c.iter() {
            match kind {
                Image::Turn(ei) => {
                    let e = &edges[ei];
                    let x = self.cube.wrap_x_point(pt.0);
                    let (n, t) = match e.axis {
                        Axis::X => (x, pt.1),
                        Axis::Z => (pt.1, x),
                    };
                    let depth = match e.beyond {
                        Side::Min => e.coord as f64 - n,
                        Side::Max => n - e.coord as f64,
                    };
                    if t >= e.span[0] as f64 - tol
                        && t <= e.span[1] as f64 + tol
                        && (-tol..=side).contains(&depth)
                    {
                        return (pt, kind);
                    }
                }
                _ => return (pt, kind),
            }
        }
        (q, Image::Direct)
    }

    /// World per simulation block at (xs, zs), analytic ([`WorldMap::jacobian`]): with m the
    /// cube's simulation blocks per world block ([east, north] per net +x and per net +z) at the
    /// world point and the Lambert block's (cos φ east, −1 / cos φ north per +zs),
    /// J = [[m₁₁ cos, −m₀₁ cos], [m₁₀ / cos, −m₀₀ / cos]] / det m.
    pub(crate) fn jacobian_at(&self, xs: f64, zs: f64) -> [[f64; 2]; 2] {
        let v = self.src.unit(xs, zs);
        let (face, u, w) = self.view.local_of_unit(v);
        let m = self.view.jacobian_on_face(face, u, w).m;
        let cos = v[0].hypot(v[1]).max(1e-9);
        let det = m[0][0] * m[1][1] - m[1][0] * m[0][1];
        [
            [m[1][1] * cos / det, -m[0][1] * cos / det],
            [m[1][0] / (cos * det), -m[0][0] / (cos * det)],
        ]
    }

    /// The footprint of a world cell of `r_w` blocks at face `face`'s local point (u, v) on a
    /// source grid of `r_s` blocks, and the cell's σ (simulation blocks per world block).
    pub(crate) fn foot(&self, face: Face, u: f64, v: f64, r_w: f64, r_s: f64) -> (Foot, f64) {
        let fj = self.view.map.jacobian(face, u, v);
        let t = self.view.turn;
        let p = t.to_planet(fj.p);
        let (du, dv) = (t.to_planet(fj.du), t.to_planet(fj.dv));
        let (east, north) = local_frame(p);
        let k = self.rs / (self.cube.side() as f64 / 2.0);
        let m = [
            [dot(du, east) * k, dot(du, north) * k],
            [dot(dv, east) * k, dot(dv, north) * k],
        ];
        let sigma = (m[0][0] * m[1][1] - m[0][1] * m[1][0]).abs().sqrt();
        let (xs, zs, cos) = self.src_block(p);
        let cos = cos.max(1e-9);
        // L, Lambert blocks per world block: (dxs, dzs) = (E / cos, −N cos); S = L Lᵀ.
        let s00 = (m[0][0] * m[0][0] + m[1][0] * m[1][0]) / (cos * cos);
        let s11 = cos * cos * (m[0][1] * m[0][1] + m[1][1] * m[1][1]);
        let s01 = -(m[0][0] * m[0][1] + m[1][0] * m[1][1]);
        let half = 0.5 * (s00 + s11);
        let disc = (0.25 * (s00 - s11) * (s00 - s11) + s01 * s01).sqrt();
        let (l1, l2) = (half + disc, (half - disc).max(0.0));
        let e1 = if s01.abs() > 1e-12 * (s00 + s11) {
            let (a, b) = (s01, l1 - s00);
            let n = a.hypot(b);
            (a / n, b / n)
        } else if s00 >= s11 {
            (1.0, 0.0)
        } else {
            (0.0, 1.0)
        };
        let d = (r_w * l1.sqrt() / r_s, r_w * l2.sqrt() / r_s);
        // The bilinear read already spans a cell: the Gaussian adds the rest in quadrature.
        let width = |dk: f64| {
            let s = r_s * (dk * dk - 1.0).max(0.0).sqrt();
            if s < FOOT_MIN_SIGMA_CELLS * r_s {
                0.0
            } else {
                s
            }
        };
        let s1 = width(d.0).min(FOOT_MAX_SIGMA_SHARE * self.src.circumference());
        let s2 = width(d.1);
        (
            Foot {
                xs,
                zs,
                d,
                sigma: (s1, s2),
                e1,
            },
            sigma,
        )
    }
}

impl WorldMap for CubeMapping {
    fn to_world(&self, xs: f64, zs: f64) -> (f64, f64) {
        self.world_at(xs, zs)
    }

    fn jacobian(&self, xs: f64, zs: f64) -> [[f64; 2]; 2] {
        self.jacobian_at(xs, zs)
    }

    fn period(&self) -> i64 {
        self.cube.period()
    }
}

impl Frame for CubeMapping {
    fn unit(&self, x: f64, z: f64) -> [f64; 3] {
        self.view.unit(x, z).unwrap_or([f64::NAN; 3])
    }

    fn point(&self, v: [f64; 3]) -> (f64, f64) {
        self.view.point_of_unit(v)
    }

    fn nearest_image(&self, p: (f64, f64), q: (f64, f64)) -> (f64, f64) {
        CubeMapping::nearest_image(self, p, q).0
    }
}

impl SimFrame for CubeMapping {
    fn sim_block(&self, v: [f64; 3]) -> (f64, f64) {
        let (xs, zs, _) = self.src_block(v);
        (xs, zs)
    }

    fn sim_unit(&self, xs: f64, zs: f64) -> [f64; 3] {
        self.src.unit(xs, zs)
    }
}

// --- rasters -----------------------------------------------------------------------------------------

/// The footprint record of one grid, by band of [`Foot::d`]'s long axis.
#[derive(Clone, Debug, Default)]
struct FootStats {
    cells: [u64; 7],
    samples: [u64; 7],
    leak: [f64; 7],
    max_samples: u64,
}

impl FootStats {
    fn add(&mut self, f: &Foot, n: usize, r_s: f64) {
        let b = D_BANDS
            .iter()
            .position(|&e| f.d.0 < e)
            .unwrap_or(D_BANDS.len());
        self.cells[b] += 1;
        self.samples[b] += n as u64;
        self.leak[b] += f.checker_leak(r_s);
        self.max_samples = self.max_samples.max(n as u64);
    }

    fn merge(mut self, o: FootStats) -> FootStats {
        for b in 0..7 {
            self.cells[b] += o.cells[b];
            self.samples[b] += o.samples[b];
            self.leak[b] += o.leak[b];
        }
        self.max_samples = self.max_samples.max(o.max_samples);
        self
    }

    fn note(&self) -> Value {
        let names = ["<1", "1-1.1", "1.1-1.2", "1.2-1.5", "1.5-2", "2-4", ">=4"];
        let total: u64 = self.cells.iter().sum();
        let bands: Vec<Value> = (0..7)
            .map(|b| {
                let c = self.cells[b].max(1) as f64;
                json!({
                    "d_long": names[b],
                    "cells": self.cells[b],
                    "share": self.cells[b] as f64 / total.max(1) as f64,
                    "mean_samples": self.samples[b] as f64 / c,
                    "checker_leak": self.leak[b] / c,
                })
            })
            .collect();
        json!({"bands": bands, "max_samples": self.max_samples})
    }
}

/// Resamples the loaded layers of one source grid onto cube grid `wg` (B2): each world cell's
/// centre, through the composed Jacobian's footprint ([`CubeMapping::foot`]). Returns each
/// layer's cell bytes in the grid's stacked order and the footprint record.
fn resample_cube(cm: &CubeMapping, srcs: &[Src], wg: &GridDesc) -> (Vec<Vec<u8>>, Value) {
    let (w, h) = (wg.w as usize, wg.h as usize);
    let r_w = f64::from(wg.r);
    let r_s = f64::from(srcs[0].layer.desc.grid.r);
    let half = f64::from(wg.face_blocks()) / 2.0;
    let mut outs: Vec<Vec<u8>> = srcs.iter().map(|s| vec![0u8; w * h * s.cb]).collect();
    let mut stats = FootStats::default();
    let nodata = |s: &Src, v: f64| {
        if v.is_nan() {
            s.layer.desc.nodata.unwrap_or(0.0)
        } else {
            v
        }
    };
    for band_start in (0..h).step_by(ROW_BAND) {
        let band_end = (band_start + ROW_BAND).min(h);
        let rows: Vec<(Vec<Vec<u8>>, FootStats)> = (band_start..band_end)
            .into_par_iter()
            .map(|j| {
                let mut row: Vec<Vec<u8>> = srcs.iter().map(|s| vec![0u8; w * s.cb]).collect();
                let mut st = FootStats::default();
                let mut samp: Vec<(f64, f64, f64)> = Vec::new();
                for i in 0..w {
                    let (face, a, b) = wg.face_of_cell(i as u32, j as u32);
                    let u = ((f64::from(a) + 0.5) * r_w - half) / half;
                    let v = ((f64::from(b) + 0.5) * r_w - half) / half;
                    let (f, _) = cm.foot(face, u, v, r_w, r_s);
                    f.samples(r_s, &mut samp);
                    st.add(&f, samp.len(), r_s);
                    for (s, out) in srcs.iter().zip(row.iter_mut()) {
                        let cell = &mut out[i * s.cb..(i + 1) * s.cb];
                        let nearest = || {
                            let (ci, cj) = s.nearest_cell(f.xs, f.zs);
                            s.raw(i64::from(ci), i64::from(cj))
                        };
                        match s.kind {
                            Kind::Stack => {
                                let (ci, cj) = s.nearest_cell(f.xs, f.zs);
                                let g = &s.layer.desc.grid;
                                let at = (cj as usize * g.w as usize + ci as usize) * s.cb;
                                cell.copy_from_slice(&s.layer.bytes[at..at + s.cb]);
                            }
                            Kind::Categorical => {
                                put_raw(s.layer.desc.dtype, nodata(s, nearest()), cell);
                            }
                            Kind::Point => {
                                let v = s.bilinear(f.xs, f.zs);
                                let v = if v.is_nan() { nearest() } else { v };
                                put_raw(s.layer.desc.dtype, nodata(s, v), cell);
                            }
                            Kind::Cover => {
                                let v = cover_of(
                                    nearest(),
                                    samp.iter().map(|&(x, z, wt)| (s.cover(x, z), wt)),
                                );
                                put_raw(s.layer.desc.dtype, v, cell);
                            }
                            Kind::Continuous => {
                                let mut sum = 0.0;
                                let mut nan = false;
                                for &(x, z, wt) in &samp {
                                    let v = s.bilinear(x, z);
                                    if v.is_nan() {
                                        nan = true;
                                        break;
                                    }
                                    sum += wt * v;
                                }
                                let v = if nan { nearest() } else { sum };
                                put_raw(s.layer.desc.dtype, nodata(s, v), cell);
                            }
                        }
                    }
                }
                (row, st)
            })
            .collect();
        for (j, (row, st)) in (band_start..band_end).zip(rows) {
            stats = stats.merge(st);
            for (k, r) in row.into_iter().enumerate() {
                let cb = srcs[k].cb;
                outs[k][j * w * cb..(j + 1) * w * cb].copy_from_slice(&r);
            }
        }
    }
    (outs, stats.note())
}

/// A layer's physical value at stacked cell c (NaN for no-data).
#[inline]
fn phys(desc: &LayerDesc, bytes: &[u8], c: usize) -> f64 {
    let cb = desc.cell_bytes();
    desc.physical(desc.dtype.decode_scalar(&bytes[c * cb..(c + 1) * cb]))
}

/// The quantiles of a set of steps.
fn step_quantiles(mut v: Vec<f64>) -> Value {
    if v.is_empty() {
        return json!({"n": 0});
    }
    v.sort_by(f64::total_cmp);
    let q = |f: f64| v[((f * (v.len() - 1) as f64).round() as usize).min(v.len() - 1)];
    let r = |x: f64| (x * 1e4).round() / 1e4;
    json!({"n": v.len(), "p50": r(q(0.5)), "p99": r(q(0.99)), "max": r(q(1.0))})
}

/// The B1 record of a layer on a cube grid: the size of the steps between neighbouring cells
/// across each kind of seam ([`seam_class`], each seam once), against the steps inside the faces
/// (every fourth row and column) and the first inward step from each face edge (`near_edge`; and
/// `beside_<kind>`, the same ground's control for each kind of seam, from both of its faces).
/// Physical units of the layer; pairs with no-data skipped.
fn seam_steps(desc: &LayerDesc, bytes: &[u8]) -> Value {
    let g = &desc.grid;
    let n = i64::from(g.w);
    let w = g.w as usize;
    let at = |face: Face, a: i64, b: i64| {
        let (ci, cj) = g.face_cell(face, a, b);
        (cj as usize * w + ci as usize, g.face_of_cell(ci, cj).0)
    };
    let val = |c: usize| phys(desc, bytes, c);
    let per_face: Vec<BTreeMap<&'static str, Vec<f64>>> = Face::ALL
        .par_iter()
        .map(|&face| {
            let mut out: BTreeMap<&'static str, Vec<f64>> = BTreeMap::new();
            let mut push = |k: &'static str, a: f64, b: f64| {
                if a.is_finite() && b.is_finite() {
                    out.entry(k).or_default().push((a - b).abs());
                }
            };
            for t in 0..n {
                for (a, b, da, db) in [
                    (n - 1, t, 1, 0),
                    (0, t, -1, 0),
                    (t, n - 1, 0, 1),
                    (t, 0, 0, -1),
                ] {
                    let (c0, _) = at(face, a, b);
                    let (c1, nf) = at(face, a + da, b + db);
                    let (ci, _) = at(face, a - da, b - db);
                    let class = seam_class(face, nf);
                    if class != "none" && face < nf {
                        push(class, val(c0), val(c1));
                    }
                    push("near_edge", val(c0), val(ci));
                    // The same ground's control for each kind of seam: the first inward step
                    // beside it, on both of its faces.
                    match class {
                        "direct" => push("beside_direct", val(c0), val(ci)),
                        "slide" => push("beside_slide", val(c0), val(ci)),
                        "turning" => push("beside_turning", val(c0), val(ci)),
                        _ => {}
                    }
                }
            }
            for b in (0..n - 1).step_by(4) {
                for a in (0..n - 1).step_by(4) {
                    let (c0, _) = at(face, a, b);
                    let (cx, _) = at(face, a + 1, b);
                    let (cz, _) = at(face, a, b + 1);
                    push("within", val(c0), val(cx));
                    push("within", val(c0), val(cz));
                }
            }
            out
        })
        .collect();
    let mut all: BTreeMap<&'static str, Vec<f64>> = BTreeMap::new();
    for m in per_face {
        for (k, v) in m {
            all.entry(k).or_default().extend(v);
        }
    }
    let mut o = serde_json::Map::new();
    o.insert("grid".into(), json!(g.name));
    for k in [
        "within",
        "near_edge",
        "direct",
        "slide",
        "turning",
        "beside_direct",
        "beside_slide",
        "beside_turning",
    ] {
        o.insert(k.into(), step_quantiles(all.remove(k).unwrap_or_default()));
    }
    Value::Object(o)
}

/// The share of a cube layer's planet area where `pred` holds: each cell weighted by the area it
/// draws (|∂p/∂u × ∂p/∂v| at its centre, σ² up to a constant).
fn planet_share(layer: &LoadedLayer, map: &CubeMap, pred: impl Fn(f64) -> bool + Sync) -> f64 {
    let g = &layer.desc.grid;
    let (w, h) = (g.w as usize, g.h as usize);
    let half = f64::from(g.face_blocks()) / 2.0;
    let r = f64::from(g.r);
    let (num, den) = (0..h)
        .into_par_iter()
        .map(|j| {
            let (mut num, mut den) = (0.0, 0.0);
            for i in 0..w {
                let (face, a, b) = g.face_of_cell(i as u32, j as u32);
                let u = ((f64::from(a) + 0.5) * r - half) / half;
                let v = ((f64::from(b) + 0.5) * r - half) / half;
                let fj = map.jacobian(face, u, v);
                let x = [
                    fj.du[1] * fj.dv[2] - fj.du[2] * fj.dv[1],
                    fj.du[2] * fj.dv[0] - fj.du[0] * fj.dv[2],
                    fj.du[0] * fj.dv[1] - fj.du[1] * fj.dv[0],
                ];
                let wt = dot(x, x).sqrt();
                if pred(phys(&layer.desc, &layer.bytes, j * w + i)) {
                    num += wt;
                }
                den += wt;
            }
            (num, den)
        })
        .reduce(|| (0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));
    if den > 0.0 { num / den } else { 0.0 }
}

/// The quantiles of the planet's metres per world block (σ) over a lattice of 129 × 129 points a
/// face (uniform in world blocks): the relief's block constants checked against it (SCL-123).
fn sigma_quantiles(view: &CubeView<'_>) -> Value {
    let k = 129usize;
    let mut s: Vec<f64> = Face::ALL
        .par_iter()
        .flat_map_iter(|&face| {
            (0..k * k).map(move |c| {
                let u = -1.0 + (2.0 * (c % k) as f64 + 1.0) / k as f64;
                let v = -1.0 + (2.0 * (c / k) as f64 + 1.0) / k as f64;
                view.jacobian_on_face(face, u, v).sigma
            })
        })
        .collect();
    s.sort_by(f64::total_cmp);
    let q = |f: f64| s[((f * (s.len() - 1) as f64).round() as usize).min(s.len() - 1)];
    let r = |x: f64| (x * 100.0).round() / 100.0;
    json!({
        "min": r(q(0.0)), "p01": r(q(0.01)), "p10": r(q(0.1)), "p50": r(q(0.5)),
        "p90": r(q(0.9)), "p99": r(q(0.99)), "max": r(q(1.0)),
    })
}

// --- vector data -------------------------------------------------------------------------------------

/// The B5 record of the rivers' projection.
#[derive(Debug, Default)]
struct RiverStats {
    seam_cuts: usize,
    sea_cuts: usize,
    partner_mismatch: usize,
    /// Planet length of the simulation's chains, of the steps kept, and of the world's chains
    /// (each world segment by σ at its middle), simulation blocks.
    len_sim: f64,
    len_sim_kept: f64,
    len_world: f64,
    /// Steps between faces, by face pair: [kept whole, cut on the seam, cut as at the sea].
    crossings: BTreeMap<String, [usize; 3]>,
}

/// Where the great circle from `a` to `b` (planet unit vectors, on adjacent faces) crosses the
/// faces' common edge: the planet point and its share of the way from `a`. None for faces that
/// are not adjacent.
fn seam_crossing(view: &CubeView<'_>, a: [f64; 3], b: [f64; 3]) -> Option<([f64; 3], f64)> {
    let (ca, cb) = (view.turn.to_cube(a), view.turn.to_cube(b));
    let (fa, fb) = (Face::of_vector(ca), Face::of_vector(cb));
    let (na, nb) = (fa.frame()[0], fb.frame()[0]);
    if fa == fb || dot(na, nb).abs() > 0.5 {
        return None;
    }
    // The edge's plane: equal components along the two faces' centres.
    let n = [na[0] - nb[0], na[1] - nb[1], na[2] - nb[2]];
    let den = dot(n, [ca[0] - cb[0], ca[1] - cb[1], ca[2] - cb[2]]);
    if den.abs() < 1e-300 {
        return None;
    }
    let t = (dot(n, ca) / den).clamp(0.0, 1.0);
    let c = normalize([
        ca[0] + t * (cb[0] - ca[0]),
        ca[1] + t * (cb[1] - ca[1]),
        ca[2] + t * (cb[2] - ca[2]),
    ]);
    Some((view.turn.to_planet(c), t))
}

/// The turning net edge whose line holds point p (within `tol`), if any.
fn edge_under(cube: &Cube, p: (f64, f64), tol: f64) -> Option<usize> {
    let x = cube.wrap_x_point(p.0);
    cube.edges().iter().enumerate().skip(2).find_map(|(k, e)| {
        let (n, t) = match e.axis {
            Axis::X => (x, p.1),
            Axis::Z => (p.1, x),
        };
        ((n - e.coord as f64).abs() <= tol
            && t >= e.span[0] as f64 - tol
            && t <= e.span[1] as f64 + tol)
            .then_some(k)
    })
}

/// The river network on the cube (B5): vertices through the inverse map, x kept continuous along
/// a chain through the slide and the direct seams; a chain that crosses a turning seam is cut on
/// it (SCL-126): the part before ends at the seam point on its face's edge with [`Mouth::Seam`]
/// into the part after, which starts at the same planet point on the partner edge, the vertex's
/// attributes taken at the crossing's share of the step. A step between faces that are not
/// adjacent, or longer than [`RIVER_JUMP_BLOCKS`], ends its part as at the sea. Half-widths are in
/// world blocks (divided by σ, simulation blocks per world block); depths are kept. Confluences are
/// re-pointed at the part they join.
fn project_rivers_cube(cm: &CubeMapping, r: &Rivers, frame: IndexFrame) -> (Rivers, RiverStats) {
    #[derive(Clone, Copy, PartialEq)]
    enum End {
        Seam,
        Sea,
    }
    struct Part {
        verts: Vec<RiverVertex>,
        orig: Vec<u32>,
        end: End,
    }
    let new_part = || Part {
        verts: Vec::new(),
        orig: Vec::new(),
        end: End::Sea,
    };
    let period = cm.cube.period() as u32;
    let sigma = |p: (f64, f64)| cm.view.jacobian(p.0, p.1).map_or(1.0, |j| j.sigma);
    let world = |v: RiverVertex, p: (f64, f64)| RiverVertex {
        x: p.0 as f32,
        z: p.1 as f32,
        hw: (f64::from(v.hw) / sigma(p).max(1e-9)) as f32,
        ..v
    };
    let lerp = |a: RiverVertex, b: RiverVertex, t: f32| RiverVertex {
        x: a.x,
        z: a.z,
        w: a.w + (b.w - a.w) * t,
        hw: a.hw + (b.hw - a.hw) * t,
        depth: a.depth + (b.depth - a.depth) * t,
        q: a.q + (b.q - a.q) * t,
    };
    let face_of = |u: [f64; 3]| cm.view.local_of_unit(u).0;
    let mut st = RiverStats::default();
    let mut parts_of: Vec<Vec<Part>> = Vec::with_capacity(r.chains.len());
    for ch in &r.chains {
        let mut parts = vec![new_part()];
        let mut prev: Option<((f64, f64), [f64; 3], RiverVertex)> = None;
        for k in ch.first..ch.first + ch.count {
            let v = r.vertices[k as usize];
            let unit = cm.src.unit(f64::from(v.x), f64::from(v.z));
            let q = cm.view.point_of_unit(unit);
            let Some((pp, pu, pv)) = prev else {
                let part = parts.last_mut().expect("a part");
                part.verts.push(world(v, q));
                part.orig.push(k);
                prev = Some((q, unit, v));
                continue;
            };
            let step = angle(pu, unit) * cm.rs;
            st.len_sim += step;
            let (fa, fb) = (face_of(pu), face_of(unit));
            let pair = (fa != fb).then(|| {
                let (a, b) = (fa.min(fb), fa.max(fb));
                format!("{}|{}", a.name(), b.name())
            });
            let (im, kind) = cm.nearest_image(pp, q);
            let jump = (im.0 - pp.0).hypot(im.1 - pp.1) > RIVER_JUMP_BLOCKS;
            let cut = match kind {
                _ if jump => None,
                Image::Direct | Image::Slide => Some(im),
                Image::Turn(_) => None,
            };
            let mut here = q;
            match cut {
                Some(im) => {
                    // Whole: on along the chain.
                    here = im;
                    st.len_sim_kept += step;
                    if let Some(pair) = &pair {
                        st.crossings.entry(pair.clone()).or_default()[0] += 1;
                    }
                    let part = parts.last_mut().expect("a part");
                    part.verts.push(world(v, im));
                    part.orig.push(k);
                }
                None => {
                    let crossing = (!jump).then(|| seam_crossing(&cm.view, pu, unit)).flatten();
                    match crossing {
                        Some((c, t)) => {
                            // Cut on the seam: the same planet point on both edges.
                            let qc = cm.view.point_of_unit(c);
                            let (end1, _) = cm.nearest_image(pp, qc);
                            let (start2, _) = cm.nearest_image(q, qc);
                            let cv = lerp(pv, v, t as f32);
                            let part = parts.last_mut().expect("a part");
                            part.verts.push(world(cv, end1));
                            part.orig.push(k);
                            part.end = End::Seam;
                            let mut next = new_part();
                            next.verts.push(world(cv, start2));
                            next.orig.push(k);
                            next.verts.push(world(v, q));
                            next.orig.push(k);
                            parts.push(next);
                            st.seam_cuts += 1;
                            st.len_sim_kept += step;
                            if let Some(pair) = &pair {
                                st.crossings.entry(pair.clone()).or_default()[1] += 1;
                            }
                        }
                        None => {
                            parts.last_mut().expect("a part").end = End::Sea;
                            let mut next = new_part();
                            next.verts.push(world(v, q));
                            next.orig.push(k);
                            parts.push(next);
                            st.sea_cuts += 1;
                            if let Some(pair) = &pair {
                                st.crossings.entry(pair.clone()).or_default()[2] += 1;
                            }
                        }
                    }
                }
            }
            prev = Some((here, unit, v));
        }
        // A part a sea cut left with one vertex goes (a seam cut never leaves one).
        parts.retain(|p| p.verts.len() >= 2);
        // The world's length, and the seam halves checked.
        let tol = 1e-3;
        for (pi, p) in parts.iter().enumerate() {
            for s in p.verts.windows(2) {
                let mid = (
                    0.5 * (f64::from(s[0].x) + f64::from(s[1].x)),
                    0.5 * (f64::from(s[0].z) + f64::from(s[1].z)),
                );
                let len = f64::from(s[1].x - s[0].x).hypot(f64::from(s[1].z - s[0].z));
                st.len_world += len * sigma(mid);
            }
            if p.end == End::Seam {
                let ok = parts.get(pi + 1).is_some_and(|nx| {
                    let a = p.verts.last().expect("vertices");
                    let b = nx.verts[0];
                    let (pa, pb) = (
                        (f64::from(a.x), f64::from(a.z)),
                        (f64::from(b.x), f64::from(b.z)),
                    );
                    let ea = edge_under(cm.cube, pa, tol * 1e2);
                    let eb = edge_under(cm.cube, pb, tol * 1e2);
                    let same = match (cm.view.unit(pa.0, pa.1), cm.view.unit(pb.0, pb.1)) {
                        (Some(ua), Some(ub)) => angle(ua, ub) * cm.rs < 0.5,
                        _ => false,
                    };
                    same && matches!((ea, eb), (Some(x), Some(y)) if cm.cube.edges()[x].partner == y)
                });
                st.partner_mismatch += usize::from(!ok);
            }
        }
        parts_of.push(parts);
    }
    let mut first_id = Vec::with_capacity(parts_of.len());
    let mut next = 0u32;
    for parts in &parts_of {
        first_id.push(next);
        next += parts.len() as u32;
    }
    let part_near = |t: usize, x: f32, z: f32| -> Option<u32> {
        let c = r.c as f32;
        let mut best: Option<(f32, u32)> = None;
        for (pi, p) in parts_of[t].iter().enumerate() {
            for &o in &p.orig {
                let v = r.vertices[o as usize];
                let mut dx = (v.x - x).abs() % c;
                dx = dx.min(c - dx);
                let d = dx.hypot(v.z - z);
                if best.is_none_or(|(bd, _)| d < bd) {
                    best = Some((d, first_id[t] + pi as u32));
                }
            }
        }
        best.map(|b| b.1)
    };
    let mut out = Rivers::empty_in(period, frame);
    out.biome_min_width = r.biome_min_width;
    out.bank_zone = r.bank_zone;
    out.bank_ratio = r.bank_ratio;
    for (k, parts) in parts_of.iter().enumerate() {
        let ch = &r.chains[k];
        let n = parts.len();
        for (pi, p) in parts.iter().enumerate() {
            let id = first_id[k] + pi as u32;
            let (mouth, into) = if pi + 1 < n {
                match p.end {
                    End::Seam => (Mouth::Seam, id + 1),
                    End::Sea => (Mouth::Sea, u32::MAX),
                }
            } else {
                match ch.mouth {
                    Mouth::River => {
                        let end = r.vertices[(ch.first + ch.count - 1) as usize];
                        match part_near(ch.into as usize, end.x, end.z) {
                            Some(id) => (Mouth::River, id),
                            None => (Mouth::Sea, u32::MAX),
                        }
                    }
                    m => (m, ch.into),
                }
            };
            out.chains.push(Chain {
                first: out.vertices.len() as u32,
                count: p.verts.len() as u32,
                mouth,
                into,
            });
            out.vertices.extend_from_slice(&p.verts);
        }
    }
    out.build_index();
    (out, st)
}

impl RiverStats {
    fn note(&self, out: &Rivers) -> Value {
        let direct_cut: usize = self
            .crossings
            .iter()
            .filter(|(k, _)| {
                let mut it = k.split('|').map(|n| Face::ALL.iter().find(|f| f.name() == n));
                matches!((it.next(), it.next()), (Some(Some(a)), Some(Some(b))) if seam_class(*a, *b) != "turning")
            })
            .map(|(_, v)| v[1] + v[2])
            .sum();
        json!({
            "chains": out.chains.len(),
            "vertices": out.vertices.len(),
            "seam_cuts": self.seam_cuts,
            "sea_cuts": self.sea_cuts,
            "seam_partner_mismatch": self.partner_mismatch,
            "cuts_on_direct_seams": direct_cut,
            "length_sim_blocks": self.len_sim,
            "length_sim_kept_blocks": self.len_sim_kept,
            "length_world_planet_blocks": self.len_world,
            "length_change": if self.len_sim_kept > 0.0 { self.len_world / self.len_sim_kept - 1.0 } else { 0.0 },
            "crossings": self.crossings.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<serde_json::Map<_, _>>(),
        })
    }
}

/// A deposit body on a cube world (B4; SCL-091 made exact, ledger: SCL-130): its centre through
/// the map as [`project_body`], and its horizontal ellipse's exact image under the composed
/// Jacobian J. The image of the ellipse R(yaw) diag(a, c) has the singular values of
/// J R diag(a, c) as its semi-axes, so its area is |det J| a c even where the Lambert grid's shear
/// turns the two axes' images off the perpendicular. SCL-091 stretches each axis by the length of
/// its own image, which overstates the area there (by 25 % for a body yawed 45° at 45° latitude);
/// the Petroff-Guyou world keeps that rule. The new first axis is the singular direction nearest
/// the old first axis's image, sign included, so a dip keeps its side.
pub(crate) fn project_body_exact(
    map: &impl WorldMap,
    b: &deeptime_format::Body,
) -> deeptime_format::Body {
    let mut out = project_body(map, b);
    let j = map.jacobian(b.center[0], b.center[2]);
    let (sy, cy) = f64::from(b.yaw).sin_cos();
    let (a, c) = (f64::from(b.axes[0]), f64::from(b.axes[2]));
    let p = apply(&j, (cy * a, sy * a));
    let q = apply(&j, (-sy * c, cy * c));
    // M Mᵀ for M = [p q]: its eigenvalues are the squared semi-axes, its eigenvectors their
    // directions.
    let (s00, s01, s11) = (
        p.0 * p.0 + q.0 * q.0,
        p.0 * p.1 + q.0 * q.1,
        p.1 * p.1 + q.1 * q.1,
    );
    let half = 0.5 * (s00 + s11);
    let root = (0.25 * (s00 - s11).powi(2) + s01 * s01).sqrt();
    let (major, minor) = ((half + root).sqrt(), (half - root).max(0.0).sqrt());
    let theta = 0.5 * (2.0 * s01).atan2(s00 - s11);
    let toward = p.1.atan2(p.0);
    let (k, _) = (0..4)
        .map(|k| (k, (theta + f64::from(k) * 0.5 * PI - toward).cos()))
        .fold((0, f64::MIN), |best, x| if x.1 > best.1 { x } else { best });
    let (ax0, ax2) = if k % 2 == 0 {
        (major, minor)
    } else {
        (minor, major)
    };
    let yaw = (theta + f64::from(k) * 0.5 * PI + PI).rem_euclid(TAU) - PI;
    if ax0.is_finite() && ax2.is_finite() && ax0 > 0.0 && ax2 > 0.0 {
        out.axes[0] = ax0 as f32;
        out.axes[2] = ax2 as f32;
        out.yaw = yaw as f32;
        out.aabb = out.computed_aabb();
    }
    out
}

/// B4: a projected body's planet area against its area in the simulation (Lambert, equal-area).
struct BodyArea {
    /// The map's local scale squared averaged over the world ellipse's points on the planet (12
    /// directions on 4 rings, weighted by area) times the ellipse's area: the planet area the
    /// ellipse would cover were its points past a turning seam drawn across it (phase 2's images).
    ratio: f64,
    /// The share of the ellipse's points off the planet (past a turning seam, in the net's void).
    off_planet: f64,
}

/// [`BodyArea`] of world body `nb` projected from simulation body `b`; None with no point of it on
/// the planet.
fn body_area_ratio(
    view: &CubeView<'_>,
    b: &deeptime_format::Body,
    nb: &deeptime_format::Body,
) -> Option<BodyArea> {
    const RINGS: usize = 4;
    const DIRS: usize = 12;
    let a0 = f64::from(b.axes[0]) * f64::from(b.axes[2]);
    let (a, c) = (f64::from(nb.axes[0]), f64::from(nb.axes[2]));
    let (sy, cy) = f64::from(nb.yaw).sin_cos();
    let (mut sum, mut weight, mut all) = (0.0, 0.0, 0.0);
    for i in 0..RINGS {
        let r = (i as f64 + 0.5) / RINGS as f64;
        for k in 0..DIRS {
            let t = TAU * k as f64 / DIRS as f64;
            let (u, v) = (a * r * t.cos(), c * r * t.sin());
            let (x, z) = (
                nb.center[0] + u * cy - v * sy,
                nb.center[2] + u * sy + v * cy,
            );
            all += r;
            if let Some(j) = view.jacobian(x, z) {
                sum += r * j.sigma * j.sigma;
                weight += r;
            }
        }
    }
    let ratio = sum / weight * a * c / a0;
    (a0 > 0.0 && weight > 0.0 && ratio.is_finite()).then(|| BodyArea {
        ratio,
        off_planet: 1.0 - weight / all,
    })
}

/// Whether world body `nb`'s horizontal ellipse holds a cube corner's image (net period `period`):
/// the map's scale is singular there (a 360° neighbourhood folded into 270°), so no one ellipse
/// keeps such a body's planet area; phase 2 draws it per face.
fn body_holds_a_corner(corners: &[[i64; 2]], period: f64, nb: &deeptime_format::Body) -> bool {
    let (a, c) = (f64::from(nb.axes[0]), f64::from(nb.axes[2]));
    let (sy, cy) = f64::from(nb.yaw).sin_cos();
    corners.iter().any(|im| {
        let mut dx = im[0] as f64 - nb.center[0];
        dx -= (dx / period).round() * period;
        let dz = im[1] as f64 - nb.center[2];
        let (u, v) = (cy * dx + sy * dz, -sy * dx + cy * dz);
        (u / a).powi(2) + (v / c).powi(2) <= 1.0
    })
}

/// The distance from net point (x, z) to the nearest cube corner image and to the nearest turning
/// net edge, blocks.
fn corner_and_edge_distance(cube: &Cube, x: f64, z: f64) -> (f64, f64) {
    let x = cube.wrap_x_point(x);
    let corner = cube
        .corners()
        .iter()
        .flat_map(|c| c.images.iter())
        .map(|im| (im[0] as f64 - x).hypot(im[1] as f64 - z))
        .fold(f64::INFINITY, f64::min);
    let edge = cube
        .edges()
        .iter()
        .skip(2)
        .map(|e| {
            let (n, t) = match e.axis {
                Axis::X => (x, z),
                Axis::Z => (z, x),
            };
            let tc = t.clamp(e.span[0] as f64, e.span[1] as f64);
            (n - e.coord as f64).hypot(t - tc)
        })
        .fold(f64::INFINITY, f64::min);
    (corner, edge)
}

/// The spawn candidates on the cube (SCL-127): as the Petroff–Guyou pass's, each ranked one moved
/// to the L3 cell under its image (its surface read there), the unranked ones picked again on the
/// world's own grids (`synth::spawn_candidates`); any within [`SPAWN_CORNER_MARGIN`] of a cube
/// corner's image or [`SPAWN_EDGE_MARGIN`] of a turning net edge, or past the simulation's pole, are
/// dropped.
fn project_spawn_cube(
    cm: &CubeMapping,
    elev: &LoadedLayer,
    water: Option<&LoadedLayer>,
    sea_level_y: i32,
    v: &mut Value,
) {
    let g = &elev.desc.grid;
    let keep = |x: f64, z: f64| {
        let (c, e) = corner_and_edge_distance(cm.cube, x, z);
        c >= SPAWN_CORNER_MARGIN && e >= SPAWN_EDGE_MARGIN
    };
    if v.get("ranked_by").is_some()
        && let Some(list) = v["candidates"].as_array_mut()
    {
        list.retain_mut(|c| {
            let (Some(x), Some(z)) = (c["x"].as_f64(), c["z"].as_f64()) else {
                return false;
            };
            if z.abs() >= cm.rs {
                return false;
            }
            let (wx, wz) = cm.world_at(x, z);
            let Some(sp) = g.cube_spot(wx, wz) else {
                return false;
            };
            let (ci, cj) = g.face_cell(sp.face, sp.a.round() as i64, sp.b.round() as i64);
            let y = elev.get(i64::from(ci), i64::from(cj));
            let (px, pz) = g.cube_point(ci, cj);
            c["x"] = json!(px.floor() as i64);
            c["z"] = json!(pz.floor() as i64);
            c["cell"] = json!([ci, cj]);
            c["surface_y"] = json!(y);
            keep(px, pz)
        });
        if !list.is_empty() {
            return;
        }
    }
    let Some(water) = water else {
        return;
    };
    let mut fresh = crate::synth::spawn_candidates(elev, water, sea_level_y);
    if let Some(list) = fresh["candidates"].as_array_mut() {
        list.retain(|c| match (c["x"].as_f64(), c["z"].as_f64()) {
            (Some(x), Some(z)) => keep(x + 0.5, z + 0.5),
            _ => false,
        });
    }
    v["candidates"] = fresh["candidates"].take();
    v["rule"] = fresh["rule"].take();
    if let Some(o) = v.as_object_mut() {
        o.remove("ranked_by");
    }
    v["picked_in_the_world"] = json!(true);
}

/// A structure window's four corners leave the planet (lie past a turning net edge or the net's
/// corners): it crosses a turning seam (SCL-128).
fn window_crosses(cube: &Cube, w: &deeptime_format::structure::Window) -> bool {
    [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)]
        .iter()
        .any(|&(sa, sc)| {
            let x = w.centre[0] + sa * w.half_along * w.along[0] + sc * w.half_across * w.across[0];
            let z = w.centre[1] + sa * w.half_along * w.along[1] + sc * w.half_across * w.across[1];
            cube.local_of_point(x, z).is_none()
        })
}

// --- the pass ----------------------------------------------------------------------------------------

/// Rewrites the Lambert world folder `dir` (its manifest `m`) as a cube world (format 0.7).
/// Returns `notes.bake.projection`; its `stats` are the land under the cube
/// (`notes.bake.stats`, [`cube_turn::land_stats`]).
pub fn project_folder_cube(
    dir: &Path,
    p: &PlanetPreset,
    m: &mut Manifest,
) -> Result<Value, CoreError> {
    let started = Instant::now();
    let c_sim = p.scale.circumference_blocks;
    let sea = f64::from(m.scale.sea_level_y);
    let elev_entry = m
        .layers
        .iter()
        .position(|l| l.name == "elevation")
        .ok_or_else(|| err("projection pass: the world has no elevation layer"))?;
    let amp_entry = m.layers.iter().position(|l| l.name == "detail/amplitude");
    let (elev_path, amp_path) = (
        dir.join(&m.layers[elev_entry].path),
        amp_entry.map(|k| dir.join(&m.layers[k].path)),
    );
    let (elev_src, amp_src) = rayon::join(
        || load_layer(&elev_path),
        || amp_path.as_deref().map(load_layer).transpose(),
    );
    let elev_src = Arc::new(elev_src?);
    let amp_src = amp_src?.map(Arc::new);
    let spec = deeptime_format::DetailSpec::of(m)?;
    let spec_seed = spec.as_ref().map(|s| s.seed()).transpose()?;
    let src_proj = Projection::new(c_sim);

    // --- the turn (B3) ---------------------------------------------------------------------------
    let land = |v: [f64; 3]| -> bool {
        let (lat, lon) = deeptime_format::projection::lat_lon_of_unit(v);
        let (x, z) = src_proj.block_of(lat, lon);
        let g = &elev_src.desc.grid;
        let u = (x - g.origin_block_x as f64) / f64::from(g.r);
        let w = (z - g.origin_block_z as f64) / f64::from(g.r);
        let (ci, cj) = g.cell((u + 0.5).floor() as i64, (w + 0.5).floor() as i64);
        let cb = elev_src.desc.cell_bytes();
        let at = (cj as usize * g.w as usize + ci as usize) * cb;
        let raw = elev_src
            .desc
            .dtype
            .decode_scalar(&elev_src.bytes[at..at + cb]);
        raw * elev_src.desc.scale + elev_src.desc.offset >= sea
    };
    let t = Instant::now();
    let land_pts = cube_turn::Land::new(&land, cube_turn::BAKE.samples);
    let choice = if p.scale.lambda_c_deg != LAMBDA_C_AUTO {
        cube_turn::pinned(&land_pts, p.scale.lambda_c_deg)
    } else {
        cube_turn::search(&land_pts, p.seed, &cube_turn::BAKE)
    };
    let turn_ms = ms(t);
    let s = face_for(c_sim);
    let cube = Cube::shared(s);
    let map = CubeMap::builtin_static(WORLD_MAP)
        .ok_or_else(|| err(format!("projection pass: no built-in cube map {WORLD_MAP}")))?;
    let wp = WorldProjection::cube(s, map, choice.turn);
    let WorldProjection::Cube(wview) = wp else {
        return Err(err("projection pass: the cube projection is not a cube"));
    };
    let cm = CubeMapping::new(cube, wview, c_sim);
    let rs = cm.rs;
    let radius_m = f64::from(c_sim) * p.scale.m_per_block_h / TAU;
    let mview = CubeView { radius_m, ..wview };
    // The nominal world block, metres: the planet's area over the six faces' blocks.
    let world_m_h = radius_m * (4.0 * PI / 6.0).sqrt() / f64::from(s);
    let period = cube.period() as u32;
    let stats = cube_turn::land_stats(&wview, &land_pts, &choice);
    let turn = cube_turn::turn_note(&choice);

    // --- rasters, grid by grid ---------------------------------------------------------------------
    let t = Instant::now();
    let mut flusher = Flusher::new();
    let (mut load_ms, mut resample_ms, mut write_ms) = (0.0, 0.0, 0.0);
    let mut by_grid: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (k, l) in m.layers.iter().enumerate() {
        by_grid.entry(l.grid.clone()).or_default().push(k);
    }
    let mut new_grids = BTreeMap::new();
    let mut layer_notes = Vec::new();
    let mut world_elev: Option<LoadedLayer> = None;
    let mut world_amp: Option<LoadedLayer> = None;
    let mut world_water: Option<LoadedLayer> = None;
    let mut world_closed: Option<LoadedLayer> = None;
    let mut held_biomes: Vec<Held> = Vec::new();
    let mut world_stack: Option<(usize, LayerDesc, Vec<u8>)> = None;
    let mut shares = serde_json::Map::new();
    let mut closed_note = json!(null);
    let mut world_ice: Option<LoadedLayer> = None;
    let mut world_class: Option<LoadedLayer> = None;
    let rivers_sim = read_rivers(dir)?;
    let standing = crate::bake::relief::read_standing(dir)?;
    let relief_noise = spec_seed.map(|sd| deeptime_format::Noise::for_world(sd, period, &wp));
    let m_per_block_v = m.scale.m_per_block_v;
    let mut relief_note = json!(null);
    let mut foot_note = serde_json::Map::new();
    let mut steps_note = serde_json::Map::new();
    for (gname, entries) in &by_grid {
        let src_grid = m
            .grids
            .get(gname)
            .cloned()
            .ok_or_else(|| err(format!("projection pass: no grid {gname}")))?;
        let wg = wp.grid(gname, src_grid.r)?;
        let tl = Instant::now();
        let paths: Vec<_> = entries
            .iter()
            .map(|&k| (k, dir.join(&m.layers[k].path)))
            .collect();
        let srcs: Vec<Src> = paths
            .par_iter()
            .map(|(k, path)| -> Result<Src, CoreError> {
                let k = *k;
                let layer = if k == elev_entry {
                    elev_src.clone()
                } else if Some(k) == amp_entry {
                    amp_src.clone().expect("the amplitude layer was loaded")
                } else {
                    Arc::new(load_layer(path)?)
                };
                let kind = kind_of(&layer.desc);
                let cb = layer.desc.cell_bytes();
                Ok(Src {
                    entry: k,
                    layer,
                    kind,
                    cb,
                })
            })
            .collect::<Result<_, _>>()?;
        load_ms += ms(tl);
        let tl = Instant::now();
        let (mut outs, foot) = resample_cube(&cm, &srcs, &wg);
        foot_note.insert(gname.clone(), foot);
        resample_ms += ms(tl);
        // The closed basins on the world's own ground, across the seams (HYD-105, SCL-122).
        if let Some(ci) = srcs.iter().position(|s| s.layer.desc.name == CLOSED_BASIN) {
            let tl = Instant::now();
            let here = srcs.iter().position(|s| s.entry == elev_entry);
            if let Some(e) = world_ground(
                &wg,
                here.map(|k| (&srcs[k].layer.desc, &outs[k][..])),
                world_elev.as_ref(),
            ) {
                // The nominal cell's area (the cube's cells are of one size on the planet to
                // within the map's spread).
                let cell_km = f64::from(wg.r) * world_m_h / 1000.0;
                let min_open = ((hydro::OPEN_SEA_KM2 / (cell_km * cell_km)).ceil() as usize).max(1);
                let radius = (hydro::CLOSED_REACH_BLOCKS / f64::from(wg.r))
                    .ceil()
                    .max(1.0) as usize;
                let n = wg.w as usize * wg.h as usize;
                let before = outs[ci].iter().filter(|&&v| v == 1).count();
                let (layer, cleared) = hydro::closed_basin_on_world_by(
                    n,
                    hydro::cube_neighbour8(&wg),
                    |c| e[c] <= sea,
                    |c| e[c] <= sea - 1.0,
                    &outs[ci],
                    min_open,
                    radius,
                );
                let after = layer.iter().filter(|&&v| v == 1).count();
                outs[ci] = layer;
                closed_note = json!({
                    "grid": gname,
                    "min_open_cells": min_open,
                    "radius_cells": radius,
                    "cleared_low_cells": cleared,
                    "flagged_cells_nearest": before,
                    "flagged_cells": after,
                    "millis": ms(tl),
                });
            }
        }
        // The block-scale relief, per face across the seams (SCL-123).
        if let Some(k) = srcs.iter().position(|s| s.entry == elev_entry) {
            let water = srcs.iter().position(|s| s.layer.desc.name == "water_level");
            let mut elev = std::mem::take(&mut outs[k]);
            let on = crate::bake::relief::OnWorld {
                map: &cm,
                grid: &wg,
                water: water.map(|w| (&srcs[w].layer.desc, &outs[w][..])),
                ice: world_ice.as_ref(),
                class_top: world_class.as_ref(),
                rivers: rivers_sim.as_ref(),
                standing: standing.as_ref(),
                noise: relief_noise,
                sea_y: sea,
                planet_r_m: radius_m,
                m_per_block_v,
            };
            relief_note = on.draw(&srcs[k].layer.desc, &mut elev);
            outs[k] = elev;
        }

        let tl = Instant::now();
        for (s, bytes) in srcs.iter().zip(outs) {
            let mut desc = s.layer.desc.clone();
            desc.grid = wg.clone();
            // A cube grid's tiles never hold two faces: the face's own tile edge, not the
            // Lambert grid's (256 does not divide a 4k face's 64 L1 cells).
            desc.tile = wg.tile();
            layer_notes.push(json!([desc.name, format!("{:?}", s.kind).to_lowercase()]));
            if s.kind == Kind::Continuous
                && matches!(
                    desc.name.as_str(),
                    "elevation" | "water_level" | "detail/amplitude"
                )
            {
                steps_note.insert(desc.name.clone(), seam_steps(&desc, &bytes));
            }
            if s.kind == Kind::Stack && world_stack.is_none() {
                world_stack = Some((s.entry, desc, bytes));
                continue;
            }
            if is_biome_layer(&desc.name) {
                held_biomes.push(Held {
                    entry: s.entry,
                    desc,
                    bytes,
                });
                continue;
            }
            let entry = &mut m.layers[s.entry];
            let written = write_deferred(&mut flusher, &dir.join(&entry.path), &desc, &bytes)?;
            entry.bytes = written.bytes;
            entry.crc32 = written.crc32;
            let world = LoadedLayer::new(desc, bytes)?;
            match world.desc.name.as_str() {
                "elevation" => {
                    shares.insert(
                        "land_map".into(),
                        json!(area_share(&world, true, |v| v >= sea)),
                    );
                    shares.insert(
                        "land_world_planet".into(),
                        json!(planet_share(&world, map, |v| v >= sea)),
                    );
                    shares.insert(
                        "land_true".into(),
                        json!(area_share(&s.layer, false, |v| v >= sea)),
                    );
                    world_elev = Some(world);
                }
                "detail/amplitude" => world_amp = Some(world),
                "water_level" => world_water = Some(world),
                CLOSED_BASIN => world_closed = Some(world),
                "surface/class_top" => world_class = Some(world),
                "climate/ice_cover" => {
                    shares.insert(
                        "ice_cover_map".into(),
                        json!(area_share(&world, true, |v| v > 0.0)),
                    );
                    shares.insert(
                        "ice_cover_world_planet".into(),
                        json!(planet_share(&world, map, |v| v > 0.0)),
                    );
                    shares.insert(
                        "ice_cover_true".into(),
                        json!(area_share(&s.layer, false, |v| v > 0.0)),
                    );
                    world_ice = Some(world);
                }
                _ => {}
            }
        }
        new_grids.insert(gname.clone(), wg);
        write_ms += ms(tl);
    }
    m.grids = new_grids;
    // The biome layers' land call on the world's ground, its fills and distances on the planet
    // (BIO-170, SCL-122).
    let mut biome_land_note = json!(null);
    if !held_biomes.is_empty() {
        let tl = Instant::now();
        if let Some(we) = &world_elev {
            let ground = BiomeGround {
                sea_level_y: m.scale.sea_level_y,
                m_per_block_v: m.scale.m_per_block_v,
                world_m_per_block_h: world_m_h,
                bake_blocks_per_world_block: rs / cube.radius(),
                elevation: we,
                closed: world_closed.as_ref(),
                cube: Some((&cm.view, rs)),
            };
            let params = p.biomes.clone().unwrap_or_default();
            biome_land_note = settle_biome_layers(&mut held_biomes, &ground, &params);
        }
        for held in held_biomes {
            let entry = &mut m.layers[held.entry];
            let written = write_deferred(
                &mut flusher,
                &dir.join(&entry.path),
                &held.desc,
                &held.bytes,
            )?;
            entry.bytes = written.bytes;
            entry.crc32 = written.crc32;
        }
        write_ms += ms(tl);
    }
    let bodies = read_bodies(dir)?;
    let sites = read_sites(dir)?;
    let stratabound = match &sites {
        Some((ss, _)) => stratabound_sites(ss.iter().map(|s| (s.id, s.deposit_type.as_str())))?,
        None => Default::default(),
    };
    let strata_body = |b: &deeptime_format::Body| b.site.is_some_and(|s| stratabound.contains(&s));
    let (mut body_host_dy, mut site_host_dy) = (Vec::new(), Vec::new());
    let mut tops_note = json!(null);
    if let Some((entry, desc, mut bytes)) = world_stack {
        let tl = Instant::now();
        if let Some(we) = &world_elev {
            let record = !stratabound.is_empty();
            let (raised, max_raise, moved) = raise_world_tops(
                &desc,
                &mut bytes,
                we,
                world_amp.as_ref(),
                m.scale.min_y,
                m.scale.max_y,
                record,
            );
            tops_note = json!({"raised_cells": raised, "max_raise_blocks": max_raise});
            if !moved.is_empty() {
                let (mut points, mut of_body) = (Vec::new(), Vec::new());
                if let Some(bs) = &bodies {
                    for (i, b) in bs.iter().enumerate().filter(|(_, b)| strata_body(b)) {
                        points.push(project_body(&cm, b).center);
                        of_body.push(i);
                    }
                }
                let n_bodies = points.len();
                let mut of_site = Vec::new();
                if let Some((ss, _)) = &sites {
                    for (i, s) in ss.iter().enumerate() {
                        if stratabound.contains(&s.id) {
                            let c = project_site(&cm, s, &[], 0).center;
                            points.push([f64::from(c[0]), f64::from(c[1]), f64::from(c[2])]);
                            of_site.push(i);
                        }
                    }
                }
                let dy = world_raise_shifts(&desc, &bytes, m.scale.min_y, &moved, &points);
                if let Some(bs) = &bodies {
                    body_host_dy = vec![0.0; bs.len()];
                    for (&i, &d) in of_body.iter().zip(&dy[..n_bodies]) {
                        body_host_dy[i] = d;
                    }
                }
                if let Some((ss, _)) = &sites {
                    site_host_dy = vec![0.0; ss.len()];
                    for (&i, &d) in of_site.iter().zip(&dy[n_bodies..]) {
                        site_host_dy[i] = d;
                    }
                }
            }
        }
        let e = &mut m.layers[entry];
        let written = write_deferred(&mut flusher, &dir.join(&e.path), &desc, &bytes)?;
        e.bytes = written.bytes;
        e.crc32 = written.crc32;
        write_ms += ms(tl);
    }
    let raster_ms = ms(t);

    // --- vector data -----------------------------------------------------------------------------
    let t = Instant::now();
    let frame = wp.index_frame();
    let mut vec_note = serde_json::Map::new();
    let set_file = |m: &mut Manifest, rel: &str, bytes: u64, crc32: u32| {
        m.files.retain(|f| f.path != rel);
        m.files.push(FileEntry {
            path: rel.to_owned(),
            bytes,
            crc32,
        });
    };
    if let Some(r) = &rivers_sim {
        let (out, st) = project_rivers_cube(&cm, r, frame);
        let (bytes, crc) = write_rivers(dir, &out)?;
        set_file(m, deeptime_format::rivers::RIVERS_PATH, bytes, crc);
        vec_note.insert("rivers".into(), st.note(&out));
    }
    if let Some(st) = read_structure(dir)? {
        let out = project_structure(&cm, &st);
        let crossing = out
            .events
            .iter()
            .filter(|e| window_crosses(cube, &e.window))
            .count();
        let (bytes, crc) = write_structure(dir, &out)?;
        set_file(m, deeptime_format::structure::STRUCTURE_PATH, bytes, crc);
        vec_note.insert(
            "structure_events".into(),
            json!({"count": out.events.len(), "crossing_turning_seams_kept": crossing}),
        );
    }
    let mut new_bodies = Vec::new();
    if let (Some(bs), Some(we)) = (&bodies, &world_elev) {
        let noise = |c: u32, wpj: &WorldProjection| match (spec.as_ref(), spec_seed) {
            (Some(sp), Some(seed)) => {
                Some((sp.clone(), deeptime_format::Noise::for_world(seed, c, wpj)))
            }
            _ => None,
        };
        let old = Drawn {
            elev: &elev_src,
            detail: drawn_detail(
                amp_src.as_deref(),
                noise(c_sim, &WorldProjection::Lambert(Projection::new(c_sim))),
            ),
            c: f64::from(c_sim),
            cube: false,
        };
        let new = Drawn {
            elev: we,
            detail: drawn_detail(world_amp.as_ref(), noise(period, &wp)),
            c: f64::from(period),
            cube: true,
        };
        // The planet area each body keeps, by where it lands: mid-face, by a seam (within 64
        // blocks of a face edge) or by a corner (within 5°).
        let corner_units: Vec<[f64; 3]> = cube_turn::corner_dirs()
            .iter()
            .map(|&c| wview.turn.to_planet(c))
            .collect();
        let zone = cube_turn::CORNER_ZONE_DEG.to_radians().cos();
        let edge_u = 1.0 - 64.0 / (f64::from(s) / 2.0);
        let place = |x: f64, z: f64| -> usize {
            let Some((face, u, v)) = cube.local_of_point(x, z) else {
                return 1;
            };
            let unit = cm.view.unit_on_face(face, u, v);
            if corner_units.iter().any(|&c| dot(c, unit) >= zone) {
                2
            } else if u.abs().max(v.abs()) > edge_u {
                1
            } else {
                0
            }
        };
        // A body whose ellipse holds a corner's image is counted apart (its own record below):
        // the map is singular there, so its area is phase 2's per-face images' to keep.
        let corner_images: Vec<[i64; 2]> = cube
            .corners()
            .iter()
            .flat_map(|c| c.images.iter().copied())
            .collect();
        let mut ratios: [Vec<f64>; 3] = Default::default();
        let mut ratios_091: [Vec<f64>; 3] = Default::default();
        let (mut over_corner, mut off_planet) = (Vec::new(), (0usize, 0.0f64));
        let (mut shifts, mut clipped, mut with_host) = ((0usize, 0.0f64), 0usize, (0usize, 0.0f64));
        new_bodies = bs
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let mut nb = project_body_exact(&cm, b);
                let (x, z) = (nb.center[0], nb.center[2]);
                let scl091 = project_body(&cm, b);
                let (area, area_091) = (
                    body_area_ratio(&cm.view, b, &nb),
                    body_area_ratio(&cm.view, b, &scl091),
                );
                if body_holds_a_corner(&corner_images, f64::from(period), &nb) {
                    let (corner, _) = corner_and_edge_distance(cube, x, z);
                    over_corner.push(json!({
                        "id": b.id, "role": b.role,
                        "semi_axes_blocks": [nb.axes[0], nb.axes[2]],
                        "corner_distance_blocks": (corner * 10.0).round() / 10.0,
                        "planet_area_ratio": area.as_ref().map(|r| (r.ratio * 1e4).round() / 1e4),
                        "planet_area_ratio_scl091": area_091.as_ref().map(|r| (r.ratio * 1e4).round() / 1e4),
                        "off_planet_share": area.as_ref().map(|r| (r.off_planet * 1e3).round() / 1e3),
                    }));
                } else {
                    let k = place(x, z);
                    if let Some(r) = &area {
                        ratios[k].push(r.ratio);
                        if r.off_planet > 0.0 {
                            off_planet.0 += 1;
                            off_planet.1 = off_planet.1.max(r.off_planet);
                        }
                    }
                    if let Some(r) = &area_091 {
                        ratios_091[k].push(r.ratio);
                    }
                }
                let ground = new.height(x, z);
                let dy = ground - old.height(b.center[0], b.center[2]);
                let host = strata_body(b).then(|| body_host_dy.get(i).copied().unwrap_or(0.0));
                match body_y(&mut nb, dy, host, ground) {
                    BodyY::Moved(d) => {
                        shifts.0 += usize::from(d.abs() >= 1.0);
                        shifts.1 = shifts.1.max(d.abs());
                    }
                    BodyY::InHost { dy, clipped: c } => {
                        clipped += usize::from(c);
                        with_host.0 += usize::from(dy.abs() >= 1.0);
                        with_host.1 = with_host.1.max(dy.abs());
                    }
                    BodyY::Kept => {}
                }
                nb
            })
            .collect();
        let (bytes, crc) = write_bodies(dir, &new_bodies)?;
        set_file(m, deeptime_format::bodies::BODIES_PATH, bytes, crc);
        let median = |v: &mut Vec<f64>| -> Value {
            if v.is_empty() {
                return Value::Null;
            }
            v.sort_by(f64::total_cmp);
            json!((v[v.len() / 2] * 1e5).round() / 1e5)
        };
        let by_place = |[mut r0, mut r1, mut r2]: [Vec<f64>; 3]| -> Value {
            let worst = r0
                .iter()
                .chain(&r1)
                .chain(&r2)
                .map(|r| (r - 1.0).abs())
                .fold(0.0f64, f64::max);
            json!({
                "mid_face": median(&mut r0), "seam": median(&mut r1), "corner": median(&mut r2),
                "counts": [r0.len(), r1.len(), r2.len()],
                "largest_change": (worst * 1e4).round() / 1e4,
            })
        };
        vec_note.insert(
            "bodies".into(),
            json!({
                "count": new_bodies.len(), "moved_a_block_or_more": shifts.0,
                "largest_move_blocks": shifts.1, "stratabound_clipped_to_ground": clipped,
                "stratabound_raised_with_host": with_host.0,
                "stratabound_largest_host_raise_blocks": with_host.1,
                "planet_area_ratio_median": by_place(ratios),
                "planet_area_ratio_median_scl091": by_place(ratios_091),
                "reaching_past_a_turning_seam": {
                    "count": off_planet.0, "largest_off_planet_share": (off_planet.1 * 1e3).round() / 1e3,
                },
                "over_a_corner": over_corner,
            }),
        );
    }
    if let Some((sites, _)) = sites {
        let out: Vec<_> = sites
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let dy = site_host_dy.get(i).map_or(0, |d| d.round() as i32);
                project_site(&cm, s, &new_bodies, dy)
            })
            .collect();
        let w = write_sites_in(dir, period, frame, &out)?;
        set_file(
            m,
            deeptime_format::sites::SITES_PATH,
            w.sites_bytes,
            w.sites_crc32,
        );
        set_file(
            m,
            deeptime_format::sites::INDEX_PATH,
            w.index_bytes,
            w.index_crc32,
        );
        vec_note.insert("sites".into(), json!(out.len()));
    }
    for rel in [
        crate::underground::VOLCANOES_PATH,
        crate::bake::relief::STANDING_PATH,
        crate::surface::impacts::CRATERS_PATH,
    ] {
        let path = dir.join(rel);
        if !path.exists() {
            continue;
        }
        let mut v: Value = serde_json::from_slice(
            &std::fs::read(&path).map_err(|e| CoreError::io(path.clone(), e))?,
        )
        .map_err(|e| err(format!("{rel}: {e}")))?;
        let key = if rel == crate::surface::impacts::CRATERS_PATH {
            "craters"
        } else {
            "volcanoes"
        };
        if let Some(list) = v[key].as_array_mut() {
            for o in list {
                if let (Some(x), Some(z)) = (o["x"].as_f64(), o["z"].as_f64()) {
                    let (wx, wz) = cm.world_at(x, z);
                    o["x"] = json!((wx * 1000.0).round() / 1000.0);
                    o["z"] = json!((wz * 1000.0).round() / 1000.0);
                }
            }
        }
        let crc = write_json(dir, rel, &v)?;
        set_file(m, rel, crc.0, crc.1);
    }
    let spawn_rel = "spawn/candidates.json";
    if dir.join(spawn_rel).exists()
        && let Some(we) = &world_elev
    {
        let path = dir.join(spawn_rel);
        let mut v: Value = serde_json::from_slice(
            &std::fs::read(&path).map_err(|e| CoreError::io(path.clone(), e))?,
        )
        .map_err(|e| err(format!("{spawn_rel}: {e}")))?;
        project_spawn_cube(&cm, we, world_water.as_ref(), m.scale.sea_level_y, &mut v);
        vec_note.insert(
            "spawn_candidates".into(),
            json!(v["candidates"].as_array().map_or(0, Vec::len)),
        );
        let crc = write_json(dir, spawn_rel, &v)?;
        set_file(m, spawn_rel, crc.0, crc.1);
    }
    let vector_ms = ms(t);
    let t = Instant::now();
    flusher.commit()?;
    let commit_ms = ms(t);

    // --- the manifest ------------------------------------------------------------------------------
    let border = match p.scale.edge_border {
        EdgeBorder::Stop => Some(cube_border(cube, BORDER_INSET_BLOCKS)),
        EdgeBorder::None => None,
    };
    m.set_projection(Some(cube_spec(&wview, border, Some(choice.distances_deg))));
    m.scale.circumference_blocks = period;
    m.scale.m_per_block_h = world_m_h;
    let mut note = json!({
        "version": VERSION,
        "kind": "cube",
        "face_blocks": s,
        "world_blocks": [4 * s, 3 * s],
        "map": {"name": map.name(), "sha256": map.sha256_hex()},
        "turn": turn,
        "stats": stats,
        "m_per_block_h": world_m_h,
        "sigma_m_per_block": sigma_quantiles(&mview),
        "bake_blocks_per_world_block": rs / cube.radius(),
        "layers": layer_notes,
        "relief": relief_note,
        "shares": Value::Object(shares),
        "footprint": Value::Object(foot_note),
        "seam_steps": Value::Object(steps_note),
        "top_raise": tops_note,
        "closed_basin": closed_note,
        "biome_land": biome_land_note,
        "vector": Value::Object(vec_note),
        "millis": {
            "turn": turn_ms, "rasters": raster_ms, "load": load_ms, "resample": resample_ms,
            "write": write_ms, "vector": vector_ms, "commit": commit_ms, "total": ms(started),
        },
    });
    if p.scale.edge_border == EdgeBorder::None {
        note["border"] = json!("none");
    }
    Ok(note)
}

/// The pass's log line for a cube world (`projection cube …`).
pub fn summary_line(v: &Value) -> String {
    let f = |v: &Value| v.as_f64().unwrap_or(f64::NAN);
    let s = &v["shares"];
    let st = &v["seam_steps"]["elevation"];
    let r = &v["vector"]["rivers"];
    format!(
        "projection cube s={} corners_from_land_deg={:.2} all_at_sea_share={:.3} land_map={:.4} \
         land_world={:.4} land_true={:.4} ice_world={:.4} ice_true={:.4} \
         elev_step_p99[within={:.2} direct={:.2} turning={:.2}] river_seam_cuts={} \
         river_partner_mismatch={} m_per_block_h={:.2} millis[turn={:.0} rasters={:.0} \
         vector={:.0} total={:.0}]",
        v["face_blocks"],
        f(&v["turn"]["min_distance_to_land_deg"]),
        f(&v["turn"]["share_all_at_sea"]),
        f(&s["land_map"]),
        f(&s["land_world_planet"]),
        f(&s["land_true"]),
        f(&s["ice_cover_world_planet"]),
        f(&s["ice_cover_true"]),
        f(&st["within"]["p99"]),
        f(&st["direct"]["p99"]),
        f(&st["turning"]["p99"]),
        r["seam_cuts"],
        r["seam_partner_mismatch"],
        f(&v["m_per_block_h"]),
        f(&v["millis"]["turn"]),
        f(&v["millis"]["rasters"]),
        f(&v["millis"]["vector"]),
        f(&v["millis"]["total"]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::DType;
    use deeptime_format::bodies::{Body, NO_CLIP, Shape};
    use deeptime_format::cube::CubeTurn;

    fn mapping(s: u32, c_sim: u32, turn: CubeTurn) -> CubeMapping {
        let map = CubeMap::builtin_static(WORLD_MAP).expect("the world map");
        let WorldProjection::Cube(view) = WorldProjection::cube(s, map, turn) else {
            unreachable!("a cube projection")
        };
        CubeMapping::new(Cube::shared(s), view, c_sim)
    }

    fn turned() -> CubeTurn {
        CubeTurn::from_angles(37.0, 21.0, 11.0)
    }

    /// A Lambert layer of f32 cells, `f` of each cell's index.
    fn lambert_src(c: u32, r: u32, f: impl Fn(i64, i64) -> f64) -> Src {
        let grid = GridDesc::for_planet("L3", c, r).expect("a grid");
        let mut bytes = Vec::with_capacity(grid.w as usize * grid.h as usize * 4);
        for j in 0..i64::from(grid.h) {
            for i in 0..i64::from(grid.w) {
                bytes.extend_from_slice(&(f(i, j) as f32).to_le_bytes());
            }
        }
        let desc = crate::bake::desc("elevation", DType::F32, &grid, 1.0, "blocks", None);
        let layer = Arc::new(LoadedLayer::new(desc, bytes).expect("a layer"));
        Src {
            entry: 0,
            kind: Kind::Continuous,
            cb: 4,
            layer,
        }
    }

    /// Local (u, v) of a stacked cube cell's centre.
    fn cell_local(g: &GridDesc, c: usize) -> (Face, f64, f64) {
        let w = g.w as usize;
        let (face, a, b) = g.face_of_cell((c % w) as u32, (c / w) as u32);
        let half = f64::from(g.face_blocks()) / 2.0;
        let r = f64::from(g.r);
        (
            face,
            ((f64::from(a) + 0.5) * r - half) / half,
            ((f64::from(b) + 0.5) * r - half) / half,
        )
    }

    /// SCL-125: on every turning seam both faces read the same blended value, and a point
    /// away from the turning seams reads the field itself.
    #[test]
    fn seam_blend_agrees_across_every_turning_seam() {
        let cube = Cube::shared(256);
        let f = |x: f64, z: f64| (x * 0.031).sin() + (z * 0.017).cos() + 0.001 * x;
        let reach = 32.0;
        let mut checked = 0;
        for (k, e) in cube.edges().iter().enumerate().skip(2) {
            let partner = &cube.edges()[e.partner];
            for step in 1..8 {
                let t = e.span[0] as f64 + (e.span[1] - e.span[0]) as f64 * f64::from(step) / 8.0;
                let p = match e.axis {
                    Axis::X => (e.coord as f64, t),
                    Axis::Z => (t, e.coord as f64),
                };
                // The same planet point seen from the partner's face.
                let q = partner.iso.inverse().point(p.0, p.1);
                assert_eq!(edge_under(cube, q, 1e-9), Some(e.partner), "edge {k}");
                let a = seam_blend(cube, p.0, p.1, reach, f);
                let b = seam_blend(cube, q.0, q.1, reach, f);
                assert!((a - b).abs() < 1e-9, "edge {k} at {p:?}: {a} vs {b}");
                // Half and half on the seam.
                assert!((a - 0.5 * (f(p.0, p.1) + f(q.0, q.1))).abs() < 1e-9);
                checked += 1;
            }
        }
        assert_eq!(checked, 12 * 7);
        // F1's centre: no turning seam within reach.
        assert_eq!(seam_blend(cube, 256.0, 0.0, reach, f), f(256.0, 0.0));
        // At the reach the blend has faded out.
        let e = &cube.edges()[2];
        let mid = 0.5 * (e.span[0] + e.span[1]) as f64;
        let inward = match e.beyond {
            Side::Min => reach,
            Side::Max => -reach,
        };
        let p = match e.axis {
            Axis::X => (e.coord as f64 + inward, mid),
            Axis::Z => (mid, e.coord as f64 + inward),
        };
        assert_eq!(seam_blend(cube, p.0, p.1, reach, f), f(p.0, p.1));
    }

    /// SCL-122: the settle's distances are great-circle distances to the nearest source, found
    /// across every seam, and its fills copy the nearest source's value.
    #[test]
    fn the_settle_distances_are_great_circle_across_the_seams() {
        let (s, r) = (256u32, 8u32);
        let m = mapping(s, 1024, turned());
        let grid = GridDesc::for_cube("L2", s, r).expect("a cube grid");
        let cs = CubeSettle::new(&grid, &m.view, 1.0);
        let n = grid.w as usize * grid.h as usize;
        // Land in two patches that each cross several faces.
        let land: Vec<bool> = (0..n)
            .map(|c| {
                let u = cs.units[c];
                (u[0] > 0.55 && u[1] > -0.3) || (u[2] < -0.45 && u[1] > 0.1)
            })
            .collect();
        let sea: Vec<usize> = (0..n).filter(|&c| !land[c]).collect();
        let d = cs.dist(&land, true);
        let cell = (PI / 2.0) / f64::from(grid.w);
        let (mut worst, mut off) = (0.0f64, 0usize);
        for (c, &dc) in d.iter().enumerate() {
            let truth = sea
                .iter()
                .map(|&k| angle(cs.units[c], cs.units[k]))
                .fold(f64::INFINITY, f64::min);
            let e = (f64::from(dc) - truth).abs();
            worst = worst.max(e);
            off += usize::from(e > 1e-4 * cell);
        }
        assert!(worst < 0.5 * cell, "worst {worst} rad, a cell {cell}");
        assert!(
            (off as f64) < 0.03 * n as f64,
            "{off} of {n} cells off the true nearest"
        );
        let mut v: Vec<f32> = (0..n)
            .map(|c| if land[c] { -1.0 } else { c as f32 })
            .collect();
        assert!(cs.fill(&mut v, &land.iter().map(|l| !l).collect::<Vec<_>>()));
        assert!(v.iter().all(|&x| x >= 0.0 && !land[x as usize]));
        assert!(!cs.fill(&mut v, &vec![false; n]));
    }

    /// The analytic Jacobian of the Lambert → cube map against finite differences.
    #[test]
    fn the_analytic_jacobian_matches_finite_differences() {
        let m = mapping(1024, 4096, turned());
        let mut checked = 0;
        for k in 0..600 {
            let xs = -2048.0 + 4096.0 * ((f64::from(k) * 0.618_034) % 1.0);
            let zs = m.rs * 0.9 * (2.0 * ((f64::from(k) * 0.414_214) % 1.0) - 1.0);
            let h = 0.02;
            let p0 = m.world_at(xs, zs);
            let px = m.world_at(xs + h, zs);
            let pz = m.world_at(xs, zs + h);
            let face = |p: (f64, f64)| m.cube.local_of_point(p.0, p.1).map(|l| l.0);
            if face(p0).is_none() || face(p0) != face(px) || face(p0) != face(pz) {
                continue;
            }
            let fd = [
                [(px.0 - p0.0) / h, (px.1 - p0.1) / h],
                [(pz.0 - p0.0) / h, (pz.1 - p0.1) / h],
            ];
            let j = m.jacobian_at(xs, zs);
            let scale = fd.iter().flatten().fold(0.0f64, |a, &b| a.max(b.abs()));
            for a in 0..2 {
                for b in 0..2 {
                    assert!(
                        (fd[a][b] - j[a][b]).abs() < 1e-3 * scale,
                        "({xs}, {zs}) J[{a}][{b}]: {} vs {}",
                        j[a][b],
                        fd[a][b]
                    );
                }
            }
            checked += 1;
        }
        assert!(checked > 500, "{checked}");
    }

    /// B2 (SCL-124): a checkerboard at the simulation's resolution, read through the cells'
    /// footprints, by band of the footprint's short axis D₂ (simulation cells per world cell).
    /// Where D₂ ≥ 1.2 the checkerboard's peak stays under 1 % (analytically at most 0.86 %);
    /// below that the world cell is about as fine as the simulation's and passes it, as the
    /// bilinear read does (65.7 %).
    #[test]
    fn the_footprint_stops_the_simulations_checkerboard() {
        let (c_sim, r_s, s) = (2048u32, 8u32, 512u32);
        let src = lambert_src(c_sim, r_s, |i, j| if (i + j) % 2 == 0 { 1.0 } else { -1.0 });
        let m = mapping(s, c_sim, turned());
        let g = src.layer.desc.grid.clone();
        for r_w in [8u32, 16] {
            let wg = GridDesc::for_cube("L3", s, r_w).expect("a cube grid");
            let (outs, _) = resample_cube(&m, std::slice::from_ref(&src), &wg);
            let n = wg.w as usize * wg.h as usize;
            let (mut num, mut den, mut cnt, mut leak) = ([0.0; 7], [0.0; 7], [0usize; 7], [0.0; 7]);
            for c in 0..n {
                let (face, u, v) = cell_local(&wg, c);
                let (f, _) = m.foot(face, u, v, f64::from(r_w), f64::from(r_s));
                let su = (f.xs - g.origin_block_x as f64) / f64::from(r_s);
                let sv = (f.zs - g.origin_block_z as f64) / f64::from(r_s);
                let cc = (PI * su).cos() * (PI * sv).cos();
                let out = f64::from(f32::from_le_bytes(
                    outs[0][c * 4..c * 4 + 4].try_into().expect("4 bytes"),
                ));
                let b = D_BANDS
                    .iter()
                    .position(|&e| f.d.1 < e)
                    .unwrap_or(D_BANDS.len());
                num[b] += out * cc;
                den[b] += cc * cc;
                cnt[b] += 1;
                leak[b] += f.checker_leak(f64::from(r_s));
            }
            for b in 0..7 {
                if cnt[b] == 0 {
                    continue;
                }
                let amp = num[b] / den[b];
                eprintln!(
                    "r_w {r_w} D2 band {b}: cells {} amp {:.4} expected {:.4}",
                    cnt[b],
                    amp,
                    leak[b] / cnt[b] as f64
                );
                if b >= 3 && cnt[b] >= 200 {
                    assert!(amp.abs() < 0.01, "r_w {r_w} band {b}: {amp}");
                }
            }
            assert!(cnt[3..].iter().sum::<usize>() > 0 || r_w == 8);
        }
    }

    /// B4: a body's planet area through the projection, mid-face, on a turning seam and 2° from
    /// a corner, at several yaws: the exact ellipse image keeps the area at its centre to 1e-4
    /// and over the whole body (the scale squared averaged over it) to 1 %; stretching each axis
    /// by its own image (SCL-091) never gives less.
    #[test]
    fn a_body_keeps_its_planet_area_on_a_seam_and_by_a_corner() {
        let m = mapping(1024, 4096, turned());
        let ratio = |cube_dir: [f64; 3], yaw: f32| {
            let p = m.view.turn.to_planet(normalize(cube_dir));
            assert!(p[2].abs() < 0.95, "too near the simulation's pole: {p:?}");
            let (xs, zs, _) = m.src_block(p);
            let b = Body::new(
                1,
                None,
                Shape::Lens,
                "ore",
                [xs, 40.0, zs],
                [6.0, 3.0, 4.0],
                yaw,
                0.0,
                NO_CLIP,
            );
            let nb = project_body_exact(&m, &b);
            let j = m
                .view
                .jacobian(nb.center[0], nb.center[2])
                .expect("on the planet");
            let s2 = j.sigma * j.sigma / 24.0;
            let at_centre = f64::from(nb.axes[0]) * f64::from(nb.axes[2]) * s2;
            assert!((at_centre - 1.0).abs() < 1e-4, "yaw {yaw}: {at_centre}");
            let old = project_body(&m, &b);
            let old_ratio = f64::from(old.axes[0]) * f64::from(old.axes[2]) * s2;
            assert!(old_ratio >= at_centre - 1e-4, "yaw {yaw}: {old_ratio}");
            body_area_ratio(&m.view, &b, &nb)
                .expect("on the planet")
                .ratio
        };
        // 2° from corner (1, 1, 1) towards F0's centre.
        let c = normalize([1.0, 1.0, 1.0]);
        let x = normalize([1.0, -0.5, -0.5]);
        let a = 2f64.to_radians();
        let by_corner = [
            c[0] * a.cos() + x[0] * a.sin(),
            c[1] * a.cos() + x[1] * a.sin(),
            c[2] * a.cos() + x[2] * a.sin(),
        ];
        for yaw in [0.0f32, 0.8, 2.0] {
            let mid = ratio([1.0, 0.1, -0.05], yaw);
            let seam = ratio([0.0, 1.0, 1.0], yaw);
            let corner = ratio(by_corner, yaw);
            for (name, r) in [("mid", mid), ("seam", seam), ("corner", corner)] {
                assert!((r - 1.0).abs() < 0.01, "{name}, yaw {yaw}: {r}");
            }
        }
        // A body wider than its distance from the corner holds the corner's image (and is counted
        // apart); the small one beside it does not.
        let images: Vec<[i64; 2]> = m
            .cube
            .corners()
            .iter()
            .flat_map(|c| c.images.iter().copied())
            .collect();
        let (xs, zs, _) = m.src_block(m.view.turn.to_planet(normalize(by_corner)));
        let period = m.cube.period() as f64;
        for (axes, holds) in [([6.0, 3.0, 4.0], false), ([60.0, 30.0, 90.0], true)] {
            let b = Body::new(
                1,
                None,
                Shape::Lens,
                "ore",
                [xs, 40.0, zs],
                axes,
                0.8,
                0.0,
                NO_CLIP,
            );
            let nb = project_body_exact(&m, &b);
            assert_eq!(body_holds_a_corner(&images, period, &nb), holds, "{axes:?}");
        }
    }

    /// Simulation-block vertices along the great circle from cube-frame `a` to `b`, x kept
    /// continuous.
    fn chain_between(m: &CubeMapping, a: [f64; 3], b: [f64; 3], n: usize) -> Vec<RiverVertex> {
        let (a, b) = (normalize(a), normalize(b));
        let c = m.src.circumference();
        let mut out: Vec<RiverVertex> = Vec::with_capacity(n);
        for k in 0..n {
            let t = k as f64 / (n - 1) as f64;
            let p = m.view.turn.to_planet(normalize([
                a[0] + t * (b[0] - a[0]),
                a[1] + t * (b[1] - a[1]),
                a[2] + t * (b[2] - a[2]),
            ]));
            let (mut xs, zs, _) = m.src_block(p);
            if let Some(prev) = out.last() {
                xs += c * ((f64::from(prev.x) - xs) / c).round();
            }
            out.push(RiverVertex {
                x: xs as f32,
                z: zs as f32,
                w: 70.0 - k as f32 * 0.1,
                hw: 3.0,
                depth: 2.0,
                q: 10.0 + k as f32,
            });
        }
        out
    }

    /// B5 (SCL-126): a chain across a turning seam is cut there into two halves on partner
    /// edges at the same planet point; one across a direct seam is not cut; the planet length
    /// changes by under 0.5 %.
    #[test]
    fn rivers_are_cut_on_turning_seams_only() {
        let m = mapping(1024, 4096, turned());
        let mut r = Rivers::empty(4096);
        for (a, b) in [
            // F1 → N (turning).
            ([0.15, 1.0, 0.7], [0.15, 0.7, 1.0]),
            // F0 → F1 (direct).
            ([1.0, 0.7, 0.1], [0.7, 1.0, 0.15]),
        ] {
            let v = chain_between(&m, a, b, 60);
            r.chains.push(Chain {
                first: r.vertices.len() as u32,
                count: v.len() as u32,
                mouth: Mouth::Sea,
                into: u32::MAX,
            });
            r.vertices.extend(v);
        }
        let frame = WorldProjection::Cube(m.view).index_frame();
        let (out, st) = project_rivers_cube(&m, &r, frame);
        assert_eq!(st.seam_cuts, 1);
        assert_eq!(st.sea_cuts, 0);
        assert_eq!(st.partner_mismatch, 0);
        assert_eq!(out.chains.len(), 3);
        assert_eq!(out.chains[0].mouth, Mouth::Seam);
        assert_eq!(out.chains[0].into, 1);
        assert_eq!(out.chains[1].mouth, Mouth::Sea);
        let end = out.vertices[(out.chains[0].count - 1) as usize];
        let start = out.vertices[out.chains[1].first as usize];
        let ua = m
            .view
            .unit(f64::from(end.x), f64::from(end.z))
            .expect("on the planet");
        let ub = m
            .view
            .unit(f64::from(start.x), f64::from(start.z))
            .expect("on the planet");
        assert!(angle(ua, ub) * m.rs < 0.05, "the halves meet at one point");
        let change = st.len_world / st.len_sim_kept - 1.0;
        assert!(change.abs() < 0.005, "length change {change}");
        // The direct seam's crossing is whole.
        assert_eq!(st.crossings.get("F0|F1"), Some(&[1, 0, 0]));
        // Widths are in world blocks: the simulation's over σ.
        let v = out.vertices[out.chains[2].first as usize];
        let sig = m
            .view
            .jacobian(f64::from(v.x), f64::from(v.z))
            .expect("on the planet")
            .sigma;
        assert!((f64::from(v.hw) * sig - 3.0).abs() < 1e-3);
        out.validate().expect("a valid network");
    }

    /// B6 (SCL-122): a basin across a turning seam is one basin: closed on both faces while the
    /// ocean does not reach it, and opened on both when the ocean joins it from one side.
    #[test]
    fn a_basin_on_a_turning_seam_is_one_basin() {
        let (s, r) = (256u32, 8u32);
        let m = mapping(s, 1024, CubeTurn::IDENTITY);
        let grid = GridDesc::for_cube("L3", s, r).expect("a cube grid");
        let cs = CubeSettle::new(&grid, &m.view, 1.0);
        let n = grid.w as usize * grid.h as usize;
        // The basin round the middle of the N|F1 seam; the ocean round F3's centre.
        let seam = normalize([0.0, 1.0, 1.0]);
        let ocean = [0.0, -1.0, 0.0];
        let basin_cells = |c: usize| dot(cs.units[c], seam) > 0.97;
        let ground = |join: bool| -> Vec<f64> {
            (0..n)
                .map(|c| {
                    let u = cs.units[c];
                    if dot(u, ocean) > 0.5 {
                        -50.0
                    } else if basin_cells(c) {
                        -5.0
                    } else if join && u[2] > 0.5 && u[1] < 0.75 && u[1] > -0.9 {
                        // A channel over N from the basin to the ocean.
                        if u[0].abs() < 0.08 { -5.0 } else { 10.0 }
                    } else {
                        10.0
                    }
                })
                .collect()
        };
        let faces_of = |cells: &[usize]| -> Vec<Face> {
            let mut f: Vec<Face> = cells.iter().map(|&c| cell_local(&grid, c).0).collect();
            f.sort();
            f.dedup();
            f
        };
        let basin: Vec<usize> = (0..n).filter(|&c| basin_cells(c)).collect();
        assert_eq!(faces_of(&basin), vec![Face::F1, Face::N]);
        for join in [false, true] {
            let e = ground(join);
            let flag: Vec<u8> = (0..n).map(|c| u8::from(basin_cells(c))).collect();
            let (layer, _) = hydro::closed_basin_on_world_by(
                n,
                hydro::cube_neighbour8(&grid),
                |c| e[c] <= 0.0,
                |c| e[c] <= -1.0,
                &flag,
                20,
                1,
            );
            let closed: Vec<u8> = basin.iter().map(|&c| layer[c]).collect();
            if join {
                assert!(
                    closed.iter().all(|&v| v == 0),
                    "the ocean opens it on both faces"
                );
            } else {
                assert!(closed.iter().all(|&v| v == 1), "closed on both faces");
            }
        }
    }

    /// The nearest image of a planet point across a turning seam is the point beyond the edge.
    #[test]
    fn the_nearest_image_crosses_turning_seams() {
        let m = mapping(1024, 4096, turned());
        let near = |c: [f64; 3]| m.view.point_of_unit(m.view.turn.to_planet(normalize(c)));
        // F1 just south of its north edge (N|F1), and N just across.
        let p = near([0.2, 1.0, 0.99]);
        let q = near([0.2, 0.99, 1.0]);
        assert!((q.0 - p.0).hypot(q.1 - p.1) > 100.0, "far apart in the net");
        let (im, kind) = m.nearest_image(p, q);
        assert!(matches!(kind, Image::Turn(_)));
        assert!((im.0 - p.0).hypot(im.1 - p.1) < 20.0, "{im:?} vs {p:?}");
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The bake's projection pass (Ben, decisions log 2026-10-03: "Bake only"; 2026-10-04: "We'll
//! proceed w/ Petroff–Guyou"): the simulation runs on its own Lambert grid, the bake writes the
//! world folder in Lambert as it always has, and this pass, the last step of
//! `Session::finalize_write`, rewrites that folder in the world's projection
//! ([`deeptime_format::projection::PetroffGuyou`]). FORMAT.md "The bake's projection pass" is the
//! contract; `docs/science/seamless-world.md` the science.
//!
//! - **λc** ([`choose_lambda_c`]): the longitude of the western hemisphere square's centre, chosen
//!   so the four cone points (where a turn of planet is drawn in half a turn of world) lie as far
//!   from land as possible, or the preset's `scale.lambda_c_deg`.
//! - **Rasters**: every world cell is a point sample at its block; the block goes to the sphere
//!   (the projection's inverse) and to the simulation's Lambert block, where the layer is sampled.
//!   Continuous layers are averaged over a footprint that is **isotropic on the sphere**
//!   ([`Footprint`]): as wide as the world cell's own footprint and as the source's coarsest
//!   spacing there. Near the poles a Lambert row spans many blocks of planet north–south but its
//!   columns a fraction of a block east–west; sampling it at a point draws that anisotropy as radial
//!   streaks round the poles (the prototype's flaw, `docs/design/seamless-wrapping.md` §5). The
//!   footprint widens east–west until both directions are resolved alike, so the poles come out as
//!   smooth as the simulation resolved them, with no direction preferred. Categories, horizon stacks
//!   and anything holding no-data take the nearest cell.
//! - **Vector data** (rivers, folds and faults, sites, bodies, volcanoes, craters, spawn
//!   candidates): positions through the projection; directions and lengths through its local
//!   Jacobian; river chains that cross a turning edge are split there.
//! - Events (latitude and longitude) and keyframes (the simulation's Lambert pictures) stay as
//!   they are.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use rayon::prelude::*;
use serde_json::{Value, json};

use deeptime_format::bodies::{read_bodies, write_bodies};
use deeptime_format::projection::{
    BorderSpec, PetroffGuyou, Projection, ProjectionKind, WorldProjection, lat_lon_of_unit,
};
use deeptime_format::rivers::{Chain, Mouth, RiverVertex, Rivers, read_rivers, write_rivers};
use deeptime_format::sites::{read_sites, write_sites_in};
use deeptime_format::structure::{Shape, read_structure, write_structure};
use deeptime_format::{
    DType, FileEntry, GridDesc, LayerDesc, LoadedLayer, Manifest, TilePackReader, write_tile_pack,
};

use crate::error::CoreError;
use crate::params::{EdgeBorder, LAMBDA_C_AUTO, PlanetPreset};

/// The version of the pass's record (`notes.bake.projection.version`).
pub const VERSION: u32 = 1;

/// λc candidates, degrees apart (1,440 round the planet).
pub const LAMBDA_STEP_DEG: f64 = 0.25; // ledger: SCL-087 (game-choice: the search resolution)

/// The farthest a cone point's distance to land is measured, degrees: beyond it every λc is as
/// good (the secondary score, the sum of the four distances, then decides).
pub const LAND_SEARCH_DEG: f64 = 20.0; // ledger: SCL-087

/// The temporary border's inset inside the north and south edges, blocks.
pub const BORDER_INSET_BLOCKS: f64 = 1.0; // ledger: SCL-088 (game-choice)

/// The largest planet-per-world-block stretch the footprint follows (near a cone point the
/// stretch grows without bound; the cone points are put in open sea).
pub const MAX_STRETCH: f64 = 8.0; // ledger: SCL-090 (game-choice)

/// The most samples a footprint takes east–west and north–south.
const MAX_SAMPLES_X: usize = 2048;
const MAX_SAMPLES_Z: usize = 64;

/// The layers sampled as categories (the nearest cell): class and id codes.
const CATEGORICAL: [&str; 4] = [
    "climate/koppen",
    "surface/class_top",
    "meta/plate_id",
    "meta/province",
];

/// How a layer is resampled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    /// Footprint mean of the bilinear field (the nearest cell where a sample is no-data).
    Continuous,
    /// The nearest cell's value.
    Categorical,
    /// The nearest cell's bytes (horizon stacks).
    Stack,
}

fn kind_of(desc: &LayerDesc) -> Kind {
    if desc.dtype == DType::HorizonStack {
        Kind::Stack
    } else if CATEGORICAL.contains(&desc.name.as_str()) {
        Kind::Categorical
    } else {
        Kind::Continuous
    }
}

fn err(m: impl Into<String>) -> CoreError {
    CoreError::InvalidState(m.into())
}

fn ms(t: Instant) -> f64 {
    (t.elapsed().as_secs_f64() * 10_000.0).round() / 10.0
}

/// A loaded source layer.
struct Src {
    entry: usize,
    layer: LoadedLayer,
    kind: Kind,
    cb: usize,
}

impl Src {
    /// The raw value of cell (i, j) (columns wrap, rows clamp), NaN for no-data.
    #[inline]
    fn raw(&self, i: i64, j: i64) -> f64 {
        let g = &self.layer.desc.grid;
        let (ci, cj) = g.cell(i, j);
        let at = (cj as usize * g.w as usize + ci as usize) * self.cb;
        let v = self.layer.desc.dtype.decode_scalar(&self.layer.bytes[at..at + self.cb]);
        match self.layer.desc.nodata {
            Some(nd) if v == nd => f64::NAN,
            _ => v,
        }
    }

    /// The cell nearest block (x, z).
    #[inline]
    fn nearest_cell(&self, x: f64, z: f64) -> (u32, u32) {
        let g = &self.layer.desc.grid;
        let u = (x - g.origin_block_x as f64) / f64::from(g.r);
        let v = (z - g.origin_block_z as f64) / f64::from(g.r);
        g.cell((u + 0.5).floor() as i64, (v + 0.5).floor() as i64)
    }

    /// Bilinear raw value at block (x, z) (the arithmetic of `RasterRead::bilinear`).
    #[inline]
    fn bilinear(&self, x: f64, z: f64) -> f64 {
        let g = &self.layer.desc.grid;
        let u = (x - g.origin_block_x as f64) / f64::from(g.r);
        let v = (z - g.origin_block_z as f64) / f64::from(g.r);
        let (fi, fj) = (u.floor(), v.floor());
        let (fu, fv) = (u - fi, v - fj);
        let (i0, j0) = (fi as i64, fj as i64);
        let v00 = self.raw(i0, j0);
        let v10 = self.raw(i0 + 1, j0);
        let v01 = self.raw(i0, j0 + 1);
        let v11 = self.raw(i0 + 1, j0 + 1);
        let a = v00 + (v10 - v00) * fu;
        let b = v01 + (v11 - v01) * fu;
        a + (b - a) * fv
    }
}

/// Writes raw value `v` of `dtype` into `out` (integers rounded half away from zero and clamped
/// to the type's range).
fn put_raw(dtype: DType, v: f64, out: &mut [u8]) {
    let r = |lo: f64, hi: f64| v.round().clamp(lo, hi);
    match dtype {
        DType::U8 => out[0] = r(0.0, 255.0) as u8,
        DType::I8 => out[0] = r(-128.0, 127.0) as i8 as u8,
        DType::U16 => out.copy_from_slice(&(r(0.0, 65_535.0) as u16).to_le_bytes()),
        DType::I16 => out.copy_from_slice(&(r(-32_768.0, 32_767.0) as i16).to_le_bytes()),
        DType::U32 => out.copy_from_slice(&(r(0.0, 4_294_967_295.0) as u32).to_le_bytes()),
        DType::F32 => out.copy_from_slice(&(v as f32).to_le_bytes()),
        DType::HorizonStack => {}
    }
}

/// Where a world cell samples the simulation: the Lambert block of its sphere point and the
/// footprint's half-widths there, Lambert blocks.
#[derive(Clone, Copy, Debug)]
pub struct Footprint {
    pub xs: f64,
    pub zs: f64,
    pub hx: f64,
    pub hz: f64,
}

/// The world → simulation mapping of the pass.
pub struct Mapping {
    pub g: PetroffGuyou,
    pub src: Projection,
    /// R of the simulation, blocks (C / 2π).
    rs: f64,
    /// The stretch table: the largest planet blocks (simulation units) per world block, on a
    /// lattice of `sr`-block cells from (−b, −b/2).
    stretch: Vec<f32>,
    sw: usize,
    sh: usize,
    sr: f64,
}

impl Mapping {
    /// The mapping of world `g` onto the simulation of circumference `c_sim`, its stretch table on
    /// `sr`-block cells.
    pub fn new(g: PetroffGuyou, c_sim: u32, sr: u32) -> Mapping {
        let src = Projection::new(c_sim);
        let rs = src.radius();
        let b = g.hemisphere() as f64;
        let sr_f = f64::from(sr.max(1));
        let sw = ((2.0 * b) / sr_f).ceil() as usize;
        let sh = (b / sr_f).ceil() as usize;
        let stretch: Vec<f32> = (0..sw * sh)
            .into_par_iter()
            .map(|k| {
                let (i, j) = (k % sw, k / sw);
                // Cell centres, never exactly on a cone point.
                let x = -b + (i as f64 + 0.5) * sr_f;
                let z = -b / 2.0 + (j as f64 + 0.5) * sr_f;
                let jac = g.jacobian(x, z, 0.25);
                let s = rs / g.radius();
                let (ax, az) = (jac[0][0] * s, jac[0][1] * s);
                let (bx, bz) = (jac[1][0] * s, jac[1][1] * s);
                let a = ax * ax + az * az;
                let c = bx * bx + bz * bz;
                let d = ax * bx + az * bz;
                let l = 0.5 * (a + c) + (0.25 * (a - c) * (a - c) + d * d).sqrt();
                (l.sqrt().clamp(0.25, MAX_STRETCH)) as f32
            })
            .collect();
        Mapping {
            g,
            src,
            rs,
            stretch,
            sw,
            sh,
            sr: sr_f,
        }
    }

    fn stretch_at(&self, x: f64, z: f64) -> f64 {
        let b = self.g.hemisphere() as f64;
        let i = (((x + b) / self.sr).floor() as i64).clamp(0, self.sw as i64 - 1) as usize;
        let j = (((z + b / 2.0) / self.sr).floor() as i64).clamp(0, self.sh as i64 - 1) as usize;
        f64::from(self.stretch[j * self.sw + i])
    }

    /// The Lambert block of a unit vector (x in [−C/2, C/2), z = −R sin φ).
    #[inline]
    pub fn src_block(&self, v: [f64; 3]) -> (f64, f64, f64) {
        let c = self.src.circumference();
        let lon = v[1].atan2(v[0]).to_degrees();
        let xs = self.src.wrap_x((lon / 360.0) * c - 0.5 * c);
        let zs = -self.rs * v[2].clamp(-1.0, 1.0);
        let cos = (v[0] * v[0] + v[1] * v[1]).sqrt();
        (xs, zs, cos)
    }

    /// Where world point (x, z), a cell of `r_w` blocks, samples a source grid of `r_s` blocks.
    pub fn footprint(&self, x: f64, z: f64, r_w: f64, r_s: f64) -> Footprint {
        let v = self.g.unit(x, z);
        let (xs, zs, cos) = self.src_block(v);
        let cos = cos.max(1e-6);
        // The isotropic diameter on the planet: the world cell's footprint and the source's
        // north–south spacing, whichever is wider.
        let d = (self.stretch_at(x, z) * r_w).max(r_s / cos);
        // Lambert blocks per planet block: 1/cos east–west, cos north–south; the bilinear read
        // already covers one source cell.
        let hx = ((d / cos - r_s).max(0.0) * 0.5).min(0.5 * self.src.circumference());
        let hz = (d * cos - r_s).max(0.0) * 0.5;
        Footprint { xs, zs, hx, hz }
    }

    /// The world point of simulation block (xs, zs).
    pub fn to_world(&self, xs: f64, zs: f64) -> (f64, f64) {
        self.g.point_of_unit(self.src.unit(xs, zs))
    }

    /// The image of world point `q` (canonical) nearest to `p` through the edges' motions: the
    /// slides and the north and south half-turns. Returns the image and whether it went through a
    /// half-turn.
    pub fn nearest_image(&self, p: (f64, f64), q: (f64, f64)) -> ((f64, f64), bool) {
        let b = self.g.hemisphere() as f64;
        let mut best = (q, false);
        let mut bd = f64::INFINITY;
        for (base, turned) in [
            (q, false),
            ((-q.0, -b - q.1), true),
            ((-q.0, b - q.1), true),
        ] {
            for k in [-1.0, 0.0, 1.0] {
                let im = (base.0 + k * 2.0 * b, base.1);
                let d = (im.0 - p.0).hypot(im.1 - p.1);
                if d < bd {
                    bd = d;
                    best = (im, turned);
                }
            }
        }
        best
    }

    /// The Jacobian of simulation block → world point at (xs, zs): columns d(world)/d(xs) and
    /// d(world)/d(zs), as [[dx/dxs, dz/dxs], [dx/dzs, dz/dzs]].
    pub fn jacobian(&self, xs: f64, zs: f64) -> [[f64; 2]; 2] {
        let h = 0.5;
        let p = self.to_world(xs, zs);
        let d = |dx: f64, dz: f64| {
            let (a, _) = self.nearest_image(p, self.to_world(xs + dx, zs + dz));
            let (m, _) = self.nearest_image(p, self.to_world(xs - dx, zs - dz));
            [(a.0 - m.0) / (2.0 * h), (a.1 - m.1) / (2.0 * h)]
        };
        [d(h, 0.0), d(0.0, h)]
    }
}

/// Applies a Jacobian (as [`Mapping::jacobian`] returns it) to a simulation vector (dx, dz).
#[inline]
fn apply(j: &[[f64; 2]; 2], v: (f64, f64)) -> (f64, f64) {
    (
        j[0][0] * v.0 + j[1][0] * v.1,
        j[0][1] * v.0 + j[1][1] * v.1,
    )
}

/// The λc the bake chose and how good it is.
#[derive(Clone, Debug, PartialEq)]
pub struct LambdaChoice {
    pub lambda_c_deg: f64,
    /// The least of the four cone points' distances to the nearest land, degrees (capped at
    /// [`LAND_SEARCH_DEG`]).
    pub min_distance_deg: f64,
    /// Each cone point's distance to land, degrees (the layout's order).
    pub distances_deg: [f64; 4],
    /// The share of the candidates with all four cone points at sea, and with all four at least
    /// 3° from land.
    pub share_at_sea: f64,
    pub share_3deg: f64,
}

/// Chooses λc (`LAMBDA_STEP_DEG` steps) for a world of hemisphere `b` so the four cone points lie
/// as far from land as possible: the largest least distance to land, then the largest sum, then
/// the smallest λc. `land(v)` says whether the unit vector v is land.
pub fn choose_lambda_c(b: u32, land: &(dyn Fn([f64; 3]) -> bool + Sync)) -> LambdaChoice {
    let g0 = PetroffGuyou::new(b, 0.0);
    let cones: Vec<(f64, f64)> = g0
        .spec(None)
        .cone_points
        .iter()
        .map(|c| (c.lat_deg, c.lon_deg))
        .collect();
    let n = (360.0 / LAMBDA_STEP_DEG).round() as usize;
    // The distance to land from (lat, lon), degrees: the first ring (0.1° apart, 360 directions)
    // holding land.
    let dist = |lat: f64, lon: f64| -> f64 {
        let c = deeptime_format::projection::unit_of_lat_lon(lat, lon);
        if land(c) {
            return 0.0;
        }
        let (e, nn) = deeptime_format::projection::local_frame(c);
        let rings = (LAND_SEARCH_DEG / 0.1).round() as usize;
        for k in 1..=rings {
            let a = (k as f64 * 0.1).to_radians();
            let (sa, ca) = a.sin_cos();
            for d in 0..360 {
                let t = (d as f64).to_radians();
                let (st, ct) = t.sin_cos();
                let v = [
                    c[0] * ca + (e[0] * ct + nn[0] * st) * sa,
                    c[1] * ca + (e[1] * ct + nn[1] * st) * sa,
                    c[2] * ca + (e[2] * ct + nn[2] * st) * sa,
                ];
                if land(v) {
                    return k as f64 * 0.1;
                }
            }
        }
        LAND_SEARCH_DEG
    };
    // Every cone point sits at ±45° latitude; its distance to land as a function of longitude
    // is tabled once per latitude (the candidates' longitudes are a lattice).
    let lats: Vec<f64> = {
        let mut v: Vec<f64> = cones.iter().map(|c| c.0).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        v
    };
    let tables: Vec<Vec<f64>> = lats
        .iter()
        .map(|&lat| {
            (0..n)
                .into_par_iter()
                .map(|k| dist(lat, k as f64 * LAMBDA_STEP_DEG))
                .collect()
        })
        .collect();
    let lookup = |lat: f64, lon: f64| {
        let t = lats.iter().position(|&l| (l - lat).abs() < 1e-6).unwrap_or(0);
        let k = ((deeptime_format::projection::wrap_deg(lon) / LAMBDA_STEP_DEG).round() as usize)
            % n;
        tables[t][k]
    };
    let mut best: Option<(f64, f64, f64, [f64; 4])> = None;
    let (mut at_sea, mut at_3) = (0usize, 0usize);
    for k in 0..n {
        let lc = k as f64 * LAMBDA_STEP_DEG;
        let ds: [f64; 4] = std::array::from_fn(|i| lookup(cones[i].0, cones[i].1 + lc));
        let mn = ds.iter().cloned().fold(f64::INFINITY, f64::min);
        let sum: f64 = ds.iter().sum();
        if mn > 0.0 {
            at_sea += 1;
        }
        if mn >= 3.0 {
            at_3 += 1;
        }
        let better = match best {
            None => true,
            Some((bm, bs, _, _)) => mn > bm || (mn == bm && sum > bs),
        };
        if better {
            best = Some((mn, sum, lc, ds));
        }
    }
    let (mn, _, lc, ds) = best.expect("at least one candidate");
    LambdaChoice {
        lambda_c_deg: lc,
        min_distance_deg: mn,
        distances_deg: ds,
        share_at_sea: at_sea as f64 / n as f64,
        share_3deg: at_3 as f64 / n as f64,
    }
}

/// The world's hemisphere and λc for preset `p`, given the land test (the auto choice when the
/// preset says −1).
pub fn world_of(
    p: &PlanetPreset,
    land: &(dyn Fn([f64; 3]) -> bool + Sync),
) -> (PetroffGuyou, Option<LambdaChoice>) {
    let b = PetroffGuyou::hemisphere_for(p.scale.circumference_blocks);
    if p.scale.lambda_c_deg == LAMBDA_C_AUTO {
        let ch = choose_lambda_c(b, land);
        (PetroffGuyou::new(b, ch.lambda_c_deg), Some(ch))
    } else {
        (PetroffGuyou::new(b, p.scale.lambda_c_deg), None)
    }
}

/// Resamples the loaded layers of one source grid onto world grid `wg`; returns each layer's
/// row-major cell bytes.
fn resample_grid(map: &Mapping, srcs: &[Src], wg: &GridDesc) -> Vec<Vec<u8>> {
    let (w, h) = (wg.w as usize, wg.h as usize);
    let r_w = f64::from(wg.r);
    let r_s = f64::from(srcs[0].layer.desc.grid.r);
    let mut outs: Vec<Vec<u8>> = srcs.iter().map(|s| vec![0u8; w * h * s.cb]).collect();
    // Rows in parallel: each task fills its row of every layer.
    let rows: Vec<Vec<Vec<u8>>> = (0..h)
        .into_par_iter()
        .map(|j| {
            let z = wg.block_z(j as i64) as f64;
            let mut row: Vec<Vec<u8>> = srcs.iter().map(|s| vec![0u8; w * s.cb]).collect();
            for i in 0..w {
                let x = wg.block_x(i as i64) as f64;
                let f = map.footprint(x, z, r_w, r_s);
                let nx = (1 + (2.0 * f.hx / r_s).ceil() as usize).min(MAX_SAMPLES_X);
                let nz = (1 + (2.0 * f.hz / r_s).ceil() as usize).min(MAX_SAMPLES_Z);
                for (s, out) in srcs.iter().zip(row.iter_mut()) {
                    let cell = &mut out[i * s.cb..(i + 1) * s.cb];
                    match s.kind {
                        Kind::Stack => {
                            let (ci, cj) = s.nearest_cell(f.xs, f.zs);
                            let g = &s.layer.desc.grid;
                            let at = (cj as usize * g.w as usize + ci as usize) * s.cb;
                            cell.copy_from_slice(&s.layer.bytes[at..at + s.cb]);
                        }
                        Kind::Categorical => {
                            let (ci, cj) = s.nearest_cell(f.xs, f.zs);
                            let v = s.raw(i64::from(ci), i64::from(cj));
                            let v = if v.is_nan() {
                                s.layer.desc.nodata.unwrap_or(0.0)
                            } else {
                                v
                            };
                            put_raw(s.layer.desc.dtype, v, cell);
                        }
                        Kind::Continuous => {
                            let mut sum = 0.0;
                            let mut nan = false;
                            'outer: for kz in 0..nz {
                                let dz = if nz == 1 {
                                    0.0
                                } else {
                                    -f.hz + 2.0 * f.hz * kz as f64 / (nz - 1) as f64
                                };
                                for kx in 0..nx {
                                    let dx = if nx == 1 {
                                        0.0
                                    } else {
                                        -f.hx + 2.0 * f.hx * kx as f64 / (nx - 1) as f64
                                    };
                                    let v = s.bilinear(f.xs + dx, f.zs + dz);
                                    if v.is_nan() {
                                        nan = true;
                                        break 'outer;
                                    }
                                    sum += v;
                                }
                            }
                            let v = if nan {
                                let (ci, cj) = s.nearest_cell(f.xs, f.zs);
                                s.raw(i64::from(ci), i64::from(cj))
                            } else {
                                sum / (nx * nz) as f64
                            };
                            let v = if v.is_nan() {
                                s.layer.desc.nodata.unwrap_or(0.0)
                            } else {
                                v
                            };
                            put_raw(s.layer.desc.dtype, v, cell);
                        }
                    }
                }
            }
            row
        })
        .collect();
    for (j, row) in rows.into_iter().enumerate() {
        for (k, r) in row.into_iter().enumerate() {
            let cb = srcs[k].cb;
            outs[k][j * w * cb..(j + 1) * w * cb].copy_from_slice(&r);
        }
    }
    // The edge rows are glued onto themselves (cell i is cell w − i): make them exactly so.
    for (k, out) in outs.iter_mut().enumerate() {
        let cb = srcs[k].cb;
        for j in [0, h - 1] {
            for i in 1..w / 2 {
                let (a, b) = (j * w + i, j * w + (w - i));
                let src: Vec<u8> = out[a * cb..(a + 1) * cb].to_vec();
                out[b * cb..(b + 1) * cb].copy_from_slice(&src);
            }
        }
    }
    outs
}

/// Row weights of a world grid for area shares: the edge rows are half cells.
fn world_row_weight(j: usize, h: usize) -> f64 {
    if j == 0 || j + 1 == h { 0.5 } else { 1.0 }
}

/// The share of a layer's area where `pred` holds, on the world grid (`world` = true) or on the
/// simulation's Lambert grid (true area: rows weighted by their share of the planet).
fn area_share(layer: &LoadedLayer, world: bool, pred: impl Fn(f64) -> bool) -> f64 {
    let g = &layer.desc.grid;
    let cb = layer.desc.cell_bytes();
    let (w, h) = (g.w as usize, g.h as usize);
    let (mut num, mut den) = (0.0, 0.0);
    for j in 0..h {
        let wt = if world {
            world_row_weight(j, h)
        } else {
            g.area_share(j as u32)
        };
        if wt == 0.0 {
            continue;
        }
        for i in 0..w {
            let at = (j * w + i) * cb;
            let raw = layer.desc.dtype.decode_scalar(&layer.bytes[at..at + cb]);
            let v = raw * layer.desc.scale + layer.desc.offset;
            if pred(v) {
                num += wt;
            }
            den += wt;
        }
    }
    if den > 0.0 { num / den } else { 0.0 }
}

/// Mean squared tangential over radial elevation slope round the poles (2°–10° from each), on a
/// sampler of world points: about 1 for terrain with no preferred direction, far above 1 for
/// streaks radiating from the pole. Also returns the same ratio at mid-latitudes (30°–60°) for
/// reference, where no streaks can form.
fn pole_anisotropy(map: &Mapping, h: &(dyn Fn(f64, f64) -> f64 + Sync)) -> (f64, f64, usize) {
    let g = &map.g;
    let b = g.hemisphere() as f64;
    let ring = |lat0: f64, lat1: f64, pole: bool| -> (f64, f64, usize) {
        let n = 4_000;
        let res: Vec<(f64, f64)> = (0..n)
            .into_par_iter()
            .filter_map(|k| {
                // Points spread over the band of both hemispheres (a golden-angle spiral).
                let t = (k as f64 + 0.5) / n as f64;
                let north = k % 2 == 0;
                let lat = lat0 + (lat1 - lat0) * t;
                let lat = if north { lat } else { -lat };
                let lon = (k as f64 * 137.507_764_05) % 360.0;
                let (x, z) = g.block_of(lat, lon);
                if z.abs() > b / 2.0 - 4.0 {
                    return None;
                }
                // The world direction of north (towards the pole for a northern point).
                let (nx, nz) = g.north(x, z);
                let (rx, rz) = if north || !pole { (nx, nz) } else { (-nx, -nz) };
                let (tx, tz) = (-rz, rx);
                let d = 2.0;
                let dr = (h(x + d * rx, z + d * rz) - h(x - d * rx, z - d * rz)) / (2.0 * d);
                let dt = (h(x + d * tx, z + d * tz) - h(x - d * tx, z - d * tz)) / (2.0 * d);
                (dr.is_finite() && dt.is_finite()).then_some((dr * dr, dt * dt))
            })
            .collect();
        let (sr, st) = res.iter().fold((0.0, 0.0), |a, v| (a.0 + v.0, a.1 + v.1));
        (st, sr, res.len())
    };
    let (pt, pr, pn) = ring(80.0, 88.0, true);
    let (mt, mr, _) = ring(30.0, 60.0, false);
    let r = |t: f64, r: f64| if r > 0.0 { t / r } else { f64::NAN };
    (r(pt, pr), r(mt, mr), pn)
}

/// Rewrites the Lambert world folder `dir` (its manifest `m`) in the preset's world projection.
/// A Lambert preset gets only its layout block. Returns `notes.bake.projection`.
pub fn project_folder(dir: &Path, p: &PlanetPreset, m: &mut Manifest) -> Result<Value, CoreError> {
    let started = Instant::now();
    let c_sim = p.scale.circumference_blocks;
    if p.scale.projection == ProjectionKind::Lambert {
        m.projection = Some(WorldProjection::Lambert(Projection::new(c_sim)).spec(None));
        return Ok(json!({"version": VERSION, "kind": "lambert"}));
    }
    let sea = f64::from(m.scale.sea_level_y);

    // The land test for λc: the baked elevation (Lambert) at the nearest cell.
    let elev_entry = m
        .layers
        .iter()
        .position(|l| l.name == "elevation")
        .ok_or_else(|| err("projection pass: the world has no elevation layer"))?;
    let elev_src = TilePackReader::open(&dir.join(&m.layers[elev_entry].path))?.load()?;
    let amp_src = match m.layers.iter().find(|l| l.name == "detail/amplitude") {
        Some(l) => Some(TilePackReader::open(&dir.join(&l.path))?.load()?),
        None => None,
    };
    let spec = deeptime_format::DetailSpec::of(m)?;
    let spec_seed = spec.as_ref().map(|s| s.seed()).transpose()?;
    let src_proj = Projection::new(c_sim);
    let land = |v: [f64; 3]| -> bool {
        let (lat, lon) = lat_lon_of_unit(v);
        let (x, z) = src_proj.block_of(lat, lon);
        let g = &elev_src.desc.grid;
        let u = (x - g.origin_block_x as f64) / f64::from(g.r);
        let w = (z - g.origin_block_z as f64) / f64::from(g.r);
        let (ci, cj) = g.cell((u + 0.5).floor() as i64, (w + 0.5).floor() as i64);
        let cb = elev_src.desc.cell_bytes();
        let at = (cj as usize * g.w as usize + ci as usize) * cb;
        let raw = elev_src.desc.dtype.decode_scalar(&elev_src.bytes[at..at + cb]);
        raw * elev_src.desc.scale + elev_src.desc.offset >= sea
    };
    let t = Instant::now();
    let (g, choice) = world_of(p, &land);
    let lambda_ms = ms(t);
    let b = g.hemisphere();
    let stretch_r = m.grids.get("L1").map_or(16, |g| g.r);
    let t = Instant::now();
    let map = Mapping::new(g, c_sim, stretch_r);
    let stretch_ms = ms(t);
    let wp = WorldProjection::PetroffGuyou(g);
    let period = g.period() as u32;

    // --- rasters, grid by grid ---------------------------------------------------------------
    let t = Instant::now();
    let mut by_grid: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (k, l) in m.layers.iter().enumerate() {
        by_grid.entry(l.grid.clone()).or_default().push(k);
    }
    let mut new_grids = BTreeMap::new();
    let mut layer_notes = Vec::new();
    // The world's elevation and ice cover, kept for the spawn candidates and the evidence.
    let mut world_elev: Option<LoadedLayer> = None;
    let mut world_amp: Option<LoadedLayer> = None;
    let mut shares = serde_json::Map::new();
    for (gname, entries) in &by_grid {
        let src_grid = m
            .grids
            .get(gname)
            .cloned()
            .ok_or_else(|| err(format!("projection pass: no grid {gname}")))?;
        let wg = wp.grid(gname, src_grid.r)?;
        let mut srcs = Vec::new();
        for &k in entries {
            let layer = TilePackReader::open(&dir.join(&m.layers[k].path))?.load()?;
            let kind = kind_of(&layer.desc);
            let cb = layer.desc.cell_bytes();
            srcs.push(Src {
                entry: k,
                layer,
                kind,
                cb,
            });
        }
        let outs = resample_grid(&map, &srcs, &wg);
        for (s, bytes) in srcs.iter().zip(outs) {
            let mut desc = s.layer.desc.clone();
            desc.grid = wg.clone();
            let entry = &mut m.layers[s.entry];
            let written = write_tile_pack(&dir.join(&entry.path), &desc, &bytes)?;
            entry.bytes = written.bytes;
            entry.crc32 = written.crc32;
            layer_notes.push(json!([desc.name, format!("{:?}", s.kind).to_lowercase()]));
            let world = LoadedLayer::new(desc, bytes)?;
            match world.desc.name.as_str() {
                "elevation" => {
                    shares.insert(
                        "land_map".into(),
                        json!(area_share(&world, true, |v| v >= sea)),
                    );
                    shares.insert(
                        "land_true".into(),
                        json!(area_share(&s.layer, false, |v| v >= sea)),
                    );
                    world_elev = Some(world);
                }
                "detail/amplitude" => world_amp = Some(world),
                "climate/ice_cover" => {
                    shares.insert(
                        "ice_cover_map".into(),
                        json!(area_share(&world, true, |v| v > 0.0)),
                    );
                    shares.insert(
                        "ice_cover_true".into(),
                        json!(area_share(&s.layer, false, |v| v > 0.0)),
                    );
                }
                _ => {}
            }
        }
        new_grids.insert(gname.clone(), wg);
    }
    m.grids = new_grids;
    let raster_ms = ms(t);

    // --- the evidence: streaks round the poles -----------------------------------------------
    let t = Instant::now();
    let mut aniso = json!(null);
    if let Some(we) = &world_elev {
        let h_world = |x: f64, z: f64| {
            let g = &we.desc.grid;
            let u = (x - g.origin_block_x as f64) / f64::from(g.r);
            let v = (z - g.origin_block_z as f64) / f64::from(g.r);
            let (fi, fj) = (u.floor(), v.floor());
            let (fu, fv) = (u - fi, v - fj);
            let cb = we.desc.cell_bytes();
            let raw = |i: i64, j: i64| {
                let (ci, cj) = g.cell(i, j);
                let at = (cj as usize * g.w as usize + ci as usize) * cb;
                we.desc.dtype.decode_scalar(&we.bytes[at..at + cb]) * we.desc.scale
            };
            let (i0, j0) = (fi as i64, fj as i64);
            let a = raw(i0, j0) + (raw(i0 + 1, j0) - raw(i0, j0)) * fu;
            let bb = raw(i0, j0 + 1) + (raw(i0 + 1, j0 + 1) - raw(i0, j0 + 1)) * fu;
            a + (bb - a) * fv
        };
        // The same terrain read straight from the Lambert raster at each block's sphere point
        // (the prototype's point sampling), for comparison.
        let h_point = |x: f64, z: f64| {
            let (xs, zs, _) = map.src_block(map.g.unit(x, z));
            let g = &elev_src.desc.grid;
            let u = (xs - g.origin_block_x as f64) / f64::from(g.r);
            let v = (zs - g.origin_block_z as f64) / f64::from(g.r);
            let (fi, fj) = (u.floor(), v.floor());
            let (fu, fv) = (u - fi, v - fj);
            let cb = elev_src.desc.cell_bytes();
            let raw = |i: i64, j: i64| {
                let (ci, cj) = g.cell(i, j);
                let at = (cj as usize * g.w as usize + ci as usize) * cb;
                elev_src.desc.dtype.decode_scalar(&elev_src.bytes[at..at + cb]) * elev_src.desc.scale
            };
            let (i0, j0) = (fi as i64, fj as i64);
            let a = raw(i0, j0) + (raw(i0 + 1, j0) - raw(i0, j0)) * fu;
            let bb = raw(i0, j0 + 1) + (raw(i0 + 1, j0 + 1) - raw(i0, j0 + 1)) * fu;
            a + (bb - a) * fv
        };
        let (pw, mw, n) = pole_anisotropy(&map, &h_world);
        let (pp, mp, _) = pole_anisotropy(&map, &h_point);
        aniso = json!({
            "points": n,
            "poles_tangential_over_radial": pw,
            "mid_latitudes_tangential_over_radial": mw,
            "point_sampled_poles": pp,
            "point_sampled_mid_latitudes": mp,
        });
    }
    let aniso_ms = ms(t);

    // --- vector data --------------------------------------------------------------------------
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
    if let Some(r) = read_rivers(dir)? {
        let (out, splits) = project_rivers(&map, &r, frame);
        let (bytes, crc) = write_rivers(dir, &out)?;
        set_file(m, deeptime_format::rivers::RIVERS_PATH, bytes, crc);
        vec_note.insert(
            "rivers".into(),
            json!({"chains": out.chains.len(), "vertices": out.vertices.len(), "split_at_edges": splits}),
        );
    }
    if let Some(st) = read_structure(dir)? {
        let out = project_structure(&map, &st);
        let (bytes, crc) = write_structure(dir, &out)?;
        set_file(m, deeptime_format::structure::STRUCTURE_PATH, bytes, crc);
        vec_note.insert("structure_events".into(), json!(out.events.len()));
    }
    let bodies = read_bodies(dir)?;
    let mut new_bodies = Vec::new();
    let mut shifts = (0usize, 0.0f64);
    if let (Some(bs), Some(we)) = (&bodies, &world_elev) {
        // Each body keeps its depth under the drawn terrain: it moves up or down by the change of
        // the drawn height between its old column and its new one (the resampling smooths the
        // terrain a little, more near the poles), so a crater's lens stays under the floor and a
        // placer on the ground.
        // The detail term of a drawing: its noise (spec and seed) for x period `c` in projection
        // `wpj`; paired with the amplitude layer below (a plain fn keeps the borrow's lifetime).
        let noise = |c: u32, wpj: &WorldProjection| match (spec.as_ref(), spec_seed) {
            (Some(s), Some(seed)) => Some((s.clone(), deeptime_format::Noise::for_world(seed, c, wpj))),
            _ => None,
        };
        fn pair(
            amp: Option<&LoadedLayer>,
            n: Option<(deeptime_format::DetailSpec, deeptime_format::Noise)>,
        ) -> Option<(&LoadedLayer, deeptime_format::DetailSpec, deeptime_format::Noise)> {
            match (amp, n) {
                (Some(a), Some((s, n))) => Some((a, s, n)),
                _ => None,
            }
        }
        let old = Drawn {
            elev: &elev_src,
            detail: pair(
                amp_src.as_ref(),
                noise(c_sim, &WorldProjection::Lambert(Projection::new(c_sim))),
            ),
            c: f64::from(c_sim),
        };
        let new = Drawn {
            elev: we,
            detail: pair(world_amp.as_ref(), noise(period, &wp)),
            c: f64::from(period),
        };
        new_bodies = bs
            .iter()
            .map(|b| {
                let mut nb = project_body(&map, b);
                let dy = new.height(nb.center[0], nb.center[2]) - old.height(b.center[0], b.center[2]);
                if dy.is_finite() {
                    nb.center[1] += dy;
                    if nb.clip_top_y != deeptime_format::bodies::NO_CLIP {
                        nb.clip_top_y += dy.round() as i32;
                    }
                    nb.aabb = nb.computed_aabb();
                    shifts.0 += usize::from(dy.abs() >= 1.0);
                    shifts.1 = shifts.1.max(dy.abs());
                }
                nb
            })
            .collect();
        let (bytes, crc) = write_bodies(dir, &new_bodies)?;
        set_file(m, deeptime_format::bodies::BODIES_PATH, bytes, crc);
        vec_note.insert(
            "bodies".into(),
            json!({"count": new_bodies.len(), "moved_a_block_or_more": shifts.0, "largest_move_blocks": shifts.1}),
        );
    }
    if let Some((sites, _)) = read_sites(dir)? {
        let out: Vec<_> = sites
            .iter()
            .map(|s| project_site(&map, s, &new_bodies))
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
        let key = if rel == crate::underground::VOLCANOES_PATH {
            "volcanoes"
        } else {
            "craters"
        };
        if let Some(list) = v[key].as_array_mut() {
            for o in list {
                if let (Some(x), Some(z)) = (o["x"].as_f64(), o["z"].as_f64()) {
                    let (wx, wz) = map.to_world(x, z);
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
        project_spawn(&map, we, &mut v);
        let crc = write_json(dir, spawn_rel, &v)?;
        set_file(m, spawn_rel, crc.0, crc.1);
    }
    let vector_ms = ms(t);

    // --- the manifest --------------------------------------------------------------------------
    let border = match p.scale.edge_border {
        EdgeBorder::Stop => Some(BorderSpec {
            kind: "edge_stop".into(),
            edges: vec!["north".into(), "south".into()],
            inset_blocks: BORDER_INSET_BLOCKS,
            temporary: true,
        }),
        EdgeBorder::None => None,
    };
    m.projection = Some(wp.spec(border));
    let radius_m = f64::from(c_sim) * p.scale.m_per_block_h / std::f64::consts::TAU;
    m.scale.circumference_blocks = period;
    // The true size of a world block: the planet's area over the world's 2b² blocks.
    m.scale.m_per_block_h = radius_m * (2.0 * std::f64::consts::PI).sqrt() / b as f64;
    let mut note = json!({
        "version": VERSION,
        "kind": "petroff_guyou",
        "hemisphere_blocks": b,
        "world_blocks": [2 * b, b],
        "lambda_c_deg": g.lambda_c_deg(),
        "lambda_c": match &choice {
            Some(ch) => json!({
                "chosen": true,
                "step_deg": LAMBDA_STEP_DEG,
                "min_distance_to_land_deg": ch.min_distance_deg,
                "cone_distances_deg": ch.distances_deg,
                "share_all_at_sea": ch.share_at_sea,
                "share_all_3deg_from_land": ch.share_3deg,
            }),
            None => json!({"chosen": false, "set_by": "scale.lambda_c_deg"}),
        },
        "m_per_block_h": m.scale.m_per_block_h,
        "layers": layer_notes,
        "shares": Value::Object(shares),
        "pole_anisotropy": aniso,
        "vector": Value::Object(vec_note),
        "millis": {
            "lambda_c": lambda_ms, "stretch": stretch_ms, "rasters": raster_ms,
            "anisotropy": aniso_ms, "vector": vector_ms, "total": ms(started),
        },
    });
    if p.scale.edge_border == EdgeBorder::None {
        note["border"] = json!("none");
    }
    Ok(note)
}

/// The terrain height the game draws at block (x, z): the elevation plus the detail term
/// (`ColumnWorld::height`), from loaded layers; `c` the x period the detail noise wraps with.
struct Drawn<'a> {
    elev: &'a LoadedLayer,
    detail: Option<(&LoadedLayer, deeptime_format::DetailSpec, deeptime_format::Noise)>,
    c: f64,
}

impl Drawn<'_> {
    fn height(&self, x: f64, z: f64) -> f64 {
        use deeptime_format::RasterRead;
        let (x, z) = (x.round(), z.round());
        let wx = x - (x / self.c + 0.5).floor() * self.c;
        let e = self.elev.bilinear(wx, z);
        match &self.detail {
            Some((amp, spec, noise)) => e + amp.bilinear(wx, z) * spec.noise(noise, wx, z),
            None => e,
        }
    }
}

fn write_json(dir: &Path, rel: &str, v: &Value) -> Result<(u64, u32), CoreError> {
    let mut bytes =
        serde_json::to_vec_pretty(v).map_err(|e| CoreError::InvalidState(format!("{rel}: {e}")))?;
    bytes.push(b'\n');
    let crc = deeptime_format::fsutil::write_atomic(&dir.join(rel), &bytes)?;
    Ok((bytes.len() as u64, crc))
}

/// The river network in the world: vertices through the projection, x kept continuous along a
/// chain through the slide; a chain that crosses a turning edge is split there (each part but the
/// last ends at the edge, as at the sea; the last keeps the mouth). Chains keep their order, so a
/// split renumbers the chains after it; confluences are re-pointed at the part they join. Returns
/// the network and the number of splits.
pub fn project_rivers(
    map: &Mapping,
    r: &Rivers,
    frame: deeptime_format::projection::IndexFrame,
) -> (Rivers, usize) {
    struct Part {
        verts: Vec<RiverVertex>,
        /// Original vertex indices.
        orig: Vec<u32>,
    }
    let period = map.g.period() as u32;
    let mut splits = 0;
    // Each original chain's surviving parts (at least two vertices each).
    let mut parts_of: Vec<Vec<Part>> = Vec::with_capacity(r.chains.len());
    for ch in &r.chains {
        let mut parts = vec![Part {
            verts: Vec::new(),
            orig: Vec::new(),
        }];
        let mut prev: Option<(f64, f64)> = None;
        for k in ch.first..ch.first + ch.count {
            let v = r.vertices[k as usize];
            let q = map.to_world(f64::from(v.x), f64::from(v.z));
            let (mut pt, turned) = match prev {
                None => (q, false),
                Some(pp) => map.nearest_image(pp, q),
            };
            let jump = prev.is_some_and(|pp| (pt.0 - pp.0).hypot(pt.1 - pp.1) > 256.0);
            if turned || jump {
                // A turning edge between the two vertices: end the part, start a new one at q.
                parts.push(Part {
                    verts: Vec::new(),
                    orig: Vec::new(),
                });
                splits += 1;
                pt = q;
            }
            let part = parts.last_mut().expect("a part");
            part.verts.push(RiverVertex {
                x: pt.0 as f32,
                z: pt.1 as f32,
                ..v
            });
            part.orig.push(k);
            prev = Some(pt);
        }
        parts.retain(|p| p.verts.len() >= 2);
        parts_of.push(parts);
    }
    // New ids in order.
    let mut first_id = Vec::with_capacity(parts_of.len());
    let mut next = 0u32;
    for parts in &parts_of {
        first_id.push(next);
        next += parts.len() as u32;
    }
    // The part of chain `t` holding the original vertex nearest to (x, z) (simulation blocks).
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
            let (mouth, into) = if pi + 1 < n {
                (Mouth::Sea, u32::MAX)
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
    (out, splits)
}

/// A structural event in the world: the window's centre through the projection, its axes and
/// sizes through the local Jacobian; a fault's slip and plane turned and stretched with them.
pub fn project_structure(
    map: &Mapping,
    st: &deeptime_format::structure::Structure,
) -> deeptime_format::structure::Structure {
    let mut out = st.clone();
    for e in &mut out.events {
        let w = &mut e.window;
        let (cx, cz) = (w.centre[0], w.centre[1]);
        let j = map.jacobian(cx, cz);
        let (wx, wz) = map.to_world(cx, cz);
        let a = apply(&j, (w.along[0], w.along[1]));
        let c = apply(&j, (w.across[0], w.across[1]));
        let (la, lc) = (a.0.hypot(a.1).max(1e-9), c.0.hypot(c.1).max(1e-9));
        let along = (a.0 / la, a.1 / la);
        // Across stays perpendicular to along, on the side the Jacobian takes it.
        let mut across = (-along.1, along.0);
        if across.0 * c.0 + across.1 * c.1 < 0.0 {
            across = (-across.0, -across.1);
        }
        let old_along = w.along;
        let old_across = w.across;
        w.centre = [wx, wz];
        w.along = [along.0, along.1];
        w.across = [across.0, across.1];
        w.half_along *= la;
        w.half_across *= lc;
        match &mut e.shape {
            Shape::Fold {
                wavelength, offset, ..
            } => {
                *wavelength *= lc;
                *offset *= lc;
            }
            Shape::Fault { pole, slip, .. } => {
                // Horizontal parts in the window's own axes, stretched like them (a slip as a
                // vector, the plane's normal as a covector), then renormalised.
                let comp = |v: [f64; 3]| {
                    (
                        v[0] * old_along[0] + v[2] * old_along[1],
                        v[0] * old_across[0] + v[2] * old_across[1],
                    )
                };
                let (sa, sc) = comp(*slip);
                let s = (sa * la, sc * lc);
                slip[0] = s.0 * along.0 + s.1 * across.0;
                slip[2] = s.0 * along.1 + s.1 * across.1;
                let (pa, pc) = comp(*pole);
                let q = (pa / la, pc / lc);
                let mut p3 = [
                    q.0 * along.0 + q.1 * across.0,
                    pole[1],
                    q.0 * along.1 + q.1 * across.1,
                ];
                let n = (p3[0] * p3[0] + p3[1] * p3[1] + p3[2] * p3[2]).sqrt();
                if n > 0.0 {
                    for v in &mut p3 {
                        *v /= n;
                    }
                }
                *pole = p3;
            }
            Shape::Intrusion { .. } => {}
        }
    }
    out
}

/// A body in the world: its centre through the projection, its horizontal axes and yaw through
/// the local Jacobian, its box recomputed.
pub fn project_body(map: &Mapping, b: &deeptime_format::Body) -> deeptime_format::Body {
    let mut out = b.clone();
    let j = map.jacobian(b.center[0], b.center[2]);
    let (wx, wz) = map.to_world(b.center[0], b.center[2]);
    let (sy, cy) = f64::from(b.yaw).sin_cos();
    let ua = apply(&j, (cy, sy));
    let uc = apply(&j, (-sy, cy));
    out.center = [wx, b.center[1], wz];
    out.yaw = ua.1.atan2(ua.0) as f32;
    out.axes[0] = (f64::from(b.axes[0]) * ua.0.hypot(ua.1)) as f32;
    out.axes[2] = (f64::from(b.axes[2]) * uc.0.hypot(uc.1)) as f32;
    out.aabb = out.computed_aabb();
    out
}

/// A site in the world: its centre through the projection; its box the union of its bodies'
/// (already projected) or, without bodies, its old box moved with the centre.
pub fn project_site(
    map: &Mapping,
    s: &deeptime_format::Site,
    bodies: &[deeptime_format::Body],
) -> deeptime_format::Site {
    let mut out = s.clone();
    let (wx, wz) = map.to_world(f64::from(s.center[0]), f64::from(s.center[2]));
    let (cx, cz) = (wx.round() as i32, wz.round() as i32);
    let mut lo = [i32::MAX; 3];
    let mut hi = [i32::MIN; 3];
    let mut any = false;
    for b in bodies.iter().filter(|b| b.site == Some(s.id)) {
        // Boxes near the slide may sit a lap away from the centre: bring them next to it.
        let period = map.g.period() as i32;
        let shift = {
            let d = (b.aabb[0][0] + b.aabb[1][0]) / 2 - cx;
            -((f64::from(d) / f64::from(period)).round() as i32) * period
        };
        for k in 0..3 {
            let off = if k == 0 { shift } else { 0 };
            lo[k] = lo[k].min(b.aabb[0][k] + off);
            hi[k] = hi[k].max(b.aabb[1][k] + off);
        }
        any = true;
    }
    if !any {
        let [olo, ohi] = s.aabb;
        let (dx, dz) = (cx - s.center[0], cz - s.center[2]);
        lo = [olo[0] + dx, olo[1], olo[2] + dz];
        hi = [ohi[0] + dx, ohi[1], ohi[2] + dz];
    }
    out.center = [cx, s.center[1], cz];
    out.aabb = [lo, hi];
    out
}

/// The spawn candidates in the world: each moved to the world L3 point nearest its image, its
/// surface read from the world's elevation; reordered by distance from (0, 0) unless ranked.
fn project_spawn(map: &Mapping, elev: &LoadedLayer, v: &mut Value) {
    let g = &elev.desc.grid;
    let cb = elev.desc.cell_bytes();
    let ranked = v.get("ranked_by").is_some();
    if let Some(list) = v["candidates"].as_array_mut() {
        for c in list.iter_mut() {
            let (Some(x), Some(z)) = (c["x"].as_f64(), c["z"].as_f64()) else {
                continue;
            };
            let (wx, wz) = map.to_world(x, z);
            let i = ((wx - g.origin_block_x as f64) / f64::from(g.r)).round() as i64;
            let j = ((wz - g.origin_block_z as f64) / f64::from(g.r)).round() as i64;
            let (ci, cj) = g.cell(i, j);
            let at = (cj as usize * g.w as usize + ci as usize) * cb;
            let y = elev.desc.dtype.decode_scalar(&elev.bytes[at..at + cb]) * elev.desc.scale
                + elev.desc.offset;
            c["x"] = json!(g.block_x(i64::from(ci)));
            c["z"] = json!(g.block_z(i64::from(cj)));
            c["cell"] = json!([ci, cj]);
            c["surface_y"] = json!(y);
        }
        if !ranked {
            list.sort_by(|a, b| {
                let d = |o: &Value| {
                    let (x, z) = (o["x"].as_f64().unwrap_or(0.0), o["z"].as_f64().unwrap_or(0.0));
                    x * x + z * z
                };
                d(a).partial_cmp(&d(b)).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }
}

/// The pass's log line (`projection …`).
pub fn summary_line(v: &Value) -> String {
    if v["kind"] != "petroff_guyou" {
        return format!("projection {}", v["kind"].as_str().unwrap_or("?"));
    }
    let f = |v: &Value| v.as_f64().unwrap_or(f64::NAN);
    let s = &v["shares"];
    let a = &v["pole_anisotropy"];
    format!(
        "projection petroff_guyou b={} lambda_c={:.2} cones_from_land_deg={:.2} lambda_share_at_sea={:.3} \
         land_map={:.4} land_true={:.4} ice_map={:.4} ice_true={:.4} poles_t/r={:.2} (point-sampled {:.2}) mid_t/r={:.2} \
         m_per_block_h={:.2} millis[lambda={:.0} rasters={:.0} vector={:.0} total={:.0}]",
        v["hemisphere_blocks"],
        f(&v["lambda_c_deg"]),
        f(&v["lambda_c"]["min_distance_to_land_deg"]),
        f(&v["lambda_c"]["share_all_at_sea"]),
        f(&s["land_map"]),
        f(&s["land_true"]),
        f(&s["ice_cover_map"]),
        f(&s["ice_cover_true"]),
        f(&a["poles_tangential_over_radial"]),
        f(&a["point_sampled_poles"]),
        f(&a["mid_latitudes_tangential_over_radial"]),
        f(&v["m_per_block_h"]),
        f(&v["millis"]["lambda_c"]),
        f(&v["millis"]["rasters"]),
        f(&v["millis"]["vector"]),
        f(&v["millis"]["total"]),
    )
}

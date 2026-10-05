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

/// Where the footprint starts widening along the parallels: the cosine of the latitude (about 59°).
pub const POLAR_WIDEN_FROM_COS: f64 = 0.52; // ledger: SCL-090 (game-choice)

/// The most the footprint is widened along the parallels round the poles.
pub const POLAR_WIDEN_MAX: f64 = 8.0; // ledger: SCL-090 (game-choice)

/// How fast the widening grows poleward: the power of cos₀ / cos φ.
pub const POLAR_WIDEN_POWER: f64 = 4.0; // ledger: SCL-090 (game-choice)

/// How much wider than its isotropic diameter the footprint is along the parallel at a point
/// whose latitude has cosine `cos`: 1 up to [`POLAR_WIDEN_FROM_COS`], then (cos₀ / cos)^p
/// ([`POLAR_WIDEN_POWER`]) up to [`POLAR_WIDEN_MAX`] (reached at about 72°). The simulation's
/// solvers work in Lambert blocks, so its polar terrain has the grid's shape: a feature k cells
/// across is k cos φ planet blocks wide east–west and k / cos φ long north–south. The isotropic
/// footprint removes only the features shorter than one row; the longer ones stay as streaks
/// radiating from the pole. Averaging along the parallel over several rows' length leaves only
/// what the simulation resolved in both directions there. The constants were fitted on an
/// earthlike_quick_16k bake so the tangential-over-radial slope ratio round the poles comes down
/// to the mid-latitudes' (STATUS build log, SW-1).
pub fn polar_widening(cos: f64) -> f64 {
    let (c0, max, p) = polar_widening_params();
    (c0 / cos.max(1e-9)).powf(p).clamp(1.0, max)
}

fn polar_widening_params() -> (f64, f64, f64) {
    // Tuning only: DEEPTIME_POLAR_WIDEN="cos0,max,power" overrides the constants.
    static P: std::sync::OnceLock<(f64, f64, f64)> = std::sync::OnceLock::new();
    *P.get_or_init(|| {
        std::env::var("DEEPTIME_POLAR_WIDEN")
            .ok()
            .and_then(|v| {
                let mut it = v.split(',').map(|t| t.trim().parse::<f64>().ok());
                Some((it.next()??, it.next()??, it.next()??))
            })
            .unwrap_or((POLAR_WIDEN_FROM_COS, POLAR_WIDEN_MAX, POLAR_WIDEN_POWER))
    })
}

/// The most samples a footprint takes east–west and north–south.
const MAX_SAMPLES_X: usize = 2048;
const MAX_SAMPLES_Z: usize = 64;

/// The layers sampled as categories (the nearest cell): class and id codes, and flags (a layer whose
/// `units` is `flag` is one too, named here or not).
const CATEGORICAL: [&str; 5] = [
    "climate/koppen",
    "climate/wetland",
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
    /// The bilinear field at the point, no footprint: the climate's band coordinates
    /// (`units` `vanilla_param`), which the bake sets per Köppen class; averaged across a wide
    /// polar footprint they would drift off the class the nearest `climate/koppen` cell names
    /// (a Dfc column read Dfa's humidity and drew forest, not taiga). The Lambert world's game read
    /// them the same way, bilinear at the block.
    Point,
    /// The nearest cell's bytes (horizon stacks).
    Stack,
}

fn kind_of(desc: &LayerDesc) -> Kind {
    if desc.dtype == DType::HorizonStack {
        Kind::Stack
    } else if CATEGORICAL.contains(&desc.name.as_str()) || desc.units == "flag" {
        Kind::Categorical
    } else if desc.units == "vanilla_param" {
        Kind::Point
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
        let v = self
            .layer
            .desc
            .dtype
            .decode_scalar(&self.layer.bytes[at..at + self.cb]);
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
        self.footprint_widened(x, z, r_w, r_s, &polar_widening)
    }

    /// [`Mapping::footprint`] with the widening along the parallel given as a function of the
    /// latitude's cosine.
    fn footprint_widened(
        &self,
        x: f64,
        z: f64,
        r_w: f64,
        r_s: f64,
        widen: &dyn Fn(f64) -> f64,
    ) -> Footprint {
        let v = self.g.unit(x, z);
        let (xs, zs, cos) = self.src_block(v);
        let cos = cos.max(1e-6);
        // The isotropic diameter on the planet: the world cell's footprint and the source's
        // north–south spacing, whichever is wider.
        let d = (self.stretch_at(x, z) * r_w).max(r_s / cos);
        // Lambert blocks per planet block: 1/cos east–west, cos north–south; the bilinear read
        // already covers one source cell. Round the poles the footprint is also widened along
        // the parallel ([`polar_widening`]).
        let hx = ((d * widen(cos) / cos - r_s).max(0.0) * 0.5).min(0.5 * self.src.circumference());
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
    (j[0][0] * v.0 + j[1][0] * v.1, j[0][1] * v.0 + j[1][1] * v.1)
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
        let t = lats
            .iter()
            .position(|&l| (l - lat).abs() < 1e-6)
            .unwrap_or(0);
        let k =
            ((deeptime_format::projection::wrap_deg(lon) / LAMBDA_STEP_DEG).round() as usize) % n;
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
                        Kind::Point => {
                            let v = s.bilinear(f.xs, f.zs);
                            let v = if v.is_nan() {
                                let (ci, cj) = s.nearest_cell(f.xs, f.zs);
                                s.raw(i64::from(ci), i64::from(cj))
                            } else {
                                v
                            };
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

/// Mean squared tangential over radial elevation slope round the poles (80°–88°), on a sampler of
/// world points: about 1 for terrain with no preferred direction, far above 1 for streaks
/// radiating from the pole. Also returns the same ratio at mid-latitudes (30°–60°) for reference.
fn pole_anisotropy(map: &Mapping, h: &(dyn Fn(f64, f64) -> f64 + Sync)) -> (f64, f64, usize) {
    pole_anisotropy_n(map, h, 4_000)
}

/// [`pole_anisotropy`] on `n` points per band.
fn pole_anisotropy_n(
    map: &Mapping,
    h: &(dyn Fn(f64, f64) -> f64 + Sync),
    n: usize,
) -> (f64, f64, usize) {
    let (p, pn) = band_anisotropy(map, h, 80.0, 88.0, true, n);
    let (m, _) = band_anisotropy(map, h, 30.0, 60.0, false, n);
    (p, m, pn)
}

/// Mean squared tangential over radial elevation slope over `n` points of the latitude band
/// [lat0, lat1] of both hemispheres (radial: towards the pole when `pole`, else north), and the
/// number of points measured.
fn band_anisotropy(
    map: &Mapping,
    h: &(dyn Fn(f64, f64) -> f64 + Sync),
    lat0: f64,
    lat1: f64,
    pole: bool,
    n: usize,
) -> (f64, usize) {
    let g = &map.g;
    let b = g.hemisphere() as f64;
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
    (if sr > 0.0 { st / sr } else { f64::NAN }, res.len())
}

/// Raises the first horizon of every world stack cell to the bake's top rule: at least the highest
/// world elevation plus detail amplitude over the L3 cells of the 5 × 5 stack cells round it, plus
/// `TOP_MARGIN_BLOCKS`, capped at `max_y` (`bake::strata::raise_tops` on the simulation's grid).
/// Beyond a turning edge the window reads the glued cells (row −k is row k at column w − i).
/// Returns (cells raised, largest raise in blocks).
fn raise_world_tops(
    desc: &LayerDesc,
    bytes: &mut [u8],
    elev: &LoadedLayer,
    amp: Option<&LoadedLayer>,
    max_y: i32,
) -> (u64, f64) {
    use crate::bake::strata::TOP_MARGIN_BLOCKS;
    use crate::structure::bake_horizons::NO_TOP;
    let g1 = &desc.grid;
    let (w1, h1) = (g1.w as usize, g1.h as usize);
    let g3 = &elev.desc.grid;
    if g3.r == 0 || !g1.r.is_multiple_of(g3.r) {
        return (0, 0.0);
    }
    let ratio = (g1.r / g3.r) as usize;
    let (w3, h3) = (g3.w as usize, g3.h as usize);
    // Per stack cell, the highest value of the L3 cells it holds (cells i·ratio .. i·ratio + ratio).
    let square = |l: &LoadedLayer| -> Vec<f64> {
        let cb = l.desc.cell_bytes();
        let mut best = vec![f64::NEG_INFINITY; w1 * h1];
        for j3 in 0..h3 {
            let j = (j3 / ratio).min(h1 - 1);
            for i3 in 0..w3 {
                let i = (i3 / ratio).min(w1 - 1);
                let at = (j3 * w3 + i3) * cb;
                let raw = l.desc.dtype.decode_scalar(&l.bytes[at..at + cb]);
                if l.desc.nodata == Some(raw) {
                    continue;
                }
                let v = raw * l.desc.scale + l.desc.offset;
                let c = &mut best[j * w1 + i];
                if v > *c {
                    *c = v;
                }
            }
        }
        best
    };
    let e = square(elev);
    let a = amp.map_or_else(|| vec![0.0; w1 * h1], square);
    let at = |v: &[f64], i: i64, j: i64| -> f64 {
        // Rows past an edge are the glued rows: (i, −k) is (w − i, k); (i, h − 1 + k) is
        // (w − i, h − 1 − k).
        let (mut i, mut j) = (i, j);
        if j < 0 {
            j = -j;
            i = w1 as i64 - i;
        } else if j > h1 as i64 - 1 {
            j = 2 * (h1 as i64 - 1) - j;
            i = w1 as i64 - i;
        }
        let j = j.clamp(0, h1 as i64 - 1) as usize;
        let i = i.rem_euclid(w1 as i64) as usize;
        v[j * w1 + i]
    };
    let cb = desc.cell_bytes();
    let (mut raised, mut max_raise) = (0u64, 0.0f64);
    for j in 0..h1 {
        for i in 0..w1 {
            let (mut me, mut ma) = (f64::NEG_INFINITY, 0.0f64);
            // A window symmetric about the cell (the bake's is i − 2 ..= i + 1 on square cells;
            // these are point samples), so the self-glued edge rows stay glued.
            for dj in -2i64..=2 {
                for di in -2i64..=2 {
                    me = me.max(at(&e, i as i64 + di, j as i64 + dj));
                    ma = ma.max(at(&a, i as i64 + di, j as i64 + dj));
                }
            }
            if !me.is_finite() {
                continue;
            }
            let need = ((me + ma).ceil() as i32 + TOP_MARGIN_BLOCKS).min(max_y) * 16;
            let off = (j * w1 + i) * cb;
            let top = i16::from_le_bytes([bytes[off], bytes[off + 1]]);
            if top == NO_TOP || i32::from(top) >= need {
                continue;
            }
            let new = need.clamp(i32::from(i16::MIN) + 1, i32::from(i16::MAX)) as i16;
            bytes[off..off + 2].copy_from_slice(&new.to_le_bytes());
            raised += 1;
            max_raise = max_raise.max(f64::from(need - i32::from(top)) / 16.0);
        }
    }
    (raised, max_raise)
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
        let raw = elev_src
            .desc
            .dtype
            .decode_scalar(&elev_src.bytes[at..at + cb]);
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
    let mut world_water: Option<LoadedLayer> = None;
    // The world's horizon stacks (entry, descriptor, cells), raised over the world's terrain below.
    let mut world_stack: Option<(usize, LayerDesc, Vec<u8>)> = None;
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
            if s.kind == Kind::Stack && world_stack.is_none() {
                world_stack = Some((s.entry, desc.clone(), bytes.clone()));
            }
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
                "water_level" => world_water = Some(world),
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
    // The first horizon covers the world's terrain (the bake's top rule, `bake::strata`): the
    // stacks come from the nearest cell and the terrain from a footprint mean, so a stack can sit
    // under the drawn ground where the footprint is wide.
    let mut tops_note = json!(null);
    if let (Some((entry, desc, mut bytes)), Some(we)) = (world_stack, &world_elev) {
        let (raised, max_raise) =
            raise_world_tops(&desc, &mut bytes, we, world_amp.as_ref(), m.scale.max_y);
        if raised > 0 {
            let e = &mut m.layers[entry];
            let written = write_tile_pack(&dir.join(&e.path), &desc, &bytes)?;
            e.bytes = written.bytes;
            e.crc32 = written.crc32;
        }
        tops_note = json!({"raised_cells": raised, "max_raise_blocks": max_raise});
    }
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
                elev_src
                    .desc
                    .dtype
                    .decode_scalar(&elev_src.bytes[at..at + cb])
                    * elev_src.desc.scale
            };
            let (i0, j0) = (fi as i64, fj as i64);
            let a = raw(i0, j0) + (raw(i0 + 1, j0) - raw(i0, j0)) * fu;
            let bb = raw(i0, j0 + 1) + (raw(i0 + 1, j0 + 1) - raw(i0, j0 + 1)) * fu;
            a + (bb - a) * fv
        };
        let (pw, mw, n) = pole_anisotropy(&map, &h_world);
        let (pp, mp, _) = pole_anisotropy(&map, &h_point);
        let (sw, _) = band_anisotropy(&map, &h_world, 70.0, 80.0, true, 4_000);
        let (sp, _) = band_anisotropy(&map, &h_point, 70.0, 80.0, true, 4_000);
        aniso = json!({
            "points": n,
            "subpolar_tangential_over_radial": sw,
            "point_sampled_subpolar": sp,
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
        // `wpj`; paired with the amplitude layer by `drawn_detail`.
        let noise = |c: u32, wpj: &WorldProjection| match (spec.as_ref(), spec_seed) {
            (Some(s), Some(seed)) => {
                Some((s.clone(), deeptime_format::Noise::for_world(seed, c, wpj)))
            }
            _ => None,
        };
        let old = Drawn {
            elev: &elev_src,
            detail: drawn_detail(
                amp_src.as_ref(),
                noise(c_sim, &WorldProjection::Lambert(Projection::new(c_sim))),
            ),
            c: f64::from(c_sim),
        };
        let new = Drawn {
            elev: we,
            detail: drawn_detail(world_amp.as_ref(), noise(period, &wp)),
            c: f64::from(period),
        };
        new_bodies = bs
            .iter()
            .map(|b| {
                let mut nb = project_body(&map, b);
                let dy =
                    new.height(nb.center[0], nb.center[2]) - old.height(b.center[0], b.center[2]);
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
        project_spawn(&map, we, world_water.as_ref(), m.scale.sea_level_y, &mut v);
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
        "top_raise": tops_note,
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
    detail: DrawnDetail<'a>,
    c: f64,
}

/// A drawing's detail term: the amplitude layer, the spec and the noise.
type DrawnDetail<'a> = Option<(
    &'a LoadedLayer,
    deeptime_format::DetailSpec,
    deeptime_format::Noise,
)>;

/// The detail term of a drawing from its amplitude layer and its noise (either may be absent).
fn drawn_detail(
    amp: Option<&LoadedLayer>,
    n: Option<(deeptime_format::DetailSpec, deeptime_format::Noise)>,
) -> DrawnDetail<'_> {
    match (amp, n) {
        (Some(a), Some((s, n))) => Some((a, s, n)),
        _ => None,
    }
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
    // A body where the mapping is singular (the Jacobian vanishes: a body at or past the simulation's pole row, found on an
    // Earth-proportioned test planet, body 22342 with semi-axes [0, 34.6, 0]) keeps its source axes: the format refuses
    // a zero semi-axis and the body is a few blocks wide wherever it lands.
    let stretch = |v: f32, s: f64| {
        let n = f64::from(v) * s;
        if n.is_finite() && n > 0.0 { n as f32 } else { v }
    };
    out.axes[0] = stretch(b.axes[0], ua.0.hypot(ua.1));
    out.axes[2] = stretch(b.axes[2], uc.0.hypot(uc.1));
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

/// No spawn candidate within this many blocks of the north and south edges (the game's
/// `SpawnHandler.EDGE_MARGIN`, POL-024).
const SPAWN_EDGE_MARGIN: f64 = 64.0; // ledger: POL-024

/// The spawn candidates in the world. Ranked ones (by the essentials) are each moved to the world
/// L3 point nearest their image, their surface read from the world's elevation; any that lay past
/// the simulation's pole (the Lambert rows beyond z = ±R, one point of the planet each) or land
/// within [`SPAWN_EDGE_MARGIN`] of a turning edge are dropped. Unranked ones (dry land nearest
/// (0, 0)) are picked again on the world's own grids by the same rule
/// (`synth::spawn_candidates`), since "nearest the centre" is the world's centre now; so are ranked
/// ones when none is left.
fn project_spawn(
    map: &Mapping,
    elev: &LoadedLayer,
    water: Option<&LoadedLayer>,
    sea_level_y: i32,
    v: &mut Value,
) {
    let g = &elev.desc.grid;
    let cb = elev.desc.cell_bytes();
    let zmax = map.g.hemisphere() as f64 / 2.0 - SPAWN_EDGE_MARGIN;
    let ranked = v.get("ranked_by").is_some();
    if ranked && let Some(list) = v["candidates"].as_array_mut() {
        let rs = map.rs;
        list.retain_mut(|c| {
            let (Some(x), Some(z)) = (c["x"].as_f64(), c["z"].as_f64()) else {
                return false;
            };
            if z.abs() >= rs {
                return false;
            }
            let (wx, wz) = map.to_world(x, z);
            let i = ((wx - g.origin_block_x as f64) / f64::from(g.r)).round() as i64;
            let j = ((wz - g.origin_block_z as f64) / f64::from(g.r)).round() as i64;
            let (ci, cj) = g.cell(i, j);
            let at = (cj as usize * g.w as usize + ci as usize) * cb;
            let y = elev.desc.dtype.decode_scalar(&elev.bytes[at..at + cb]) * elev.desc.scale
                + elev.desc.offset;
            let bz = g.block_z(i64::from(cj));
            c["x"] = json!(g.block_x(i64::from(ci)));
            c["z"] = json!(bz);
            c["cell"] = json!([ci, cj]);
            c["surface_y"] = json!(y);
            (bz as f64).abs() <= zmax
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
        list.retain(|c| c["z"].as_f64().is_some_and(|z| z.abs() <= zmax));
    }
    v["candidates"] = fresh["candidates"].take();
    v["rule"] = fresh["rule"].take();
    if let Some(o) = v.as_object_mut() {
        o.remove("ranked_by");
    }
    v["picked_in_the_world"] = json!(true);
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
         land_map={:.4} land_true={:.4} ice_map={:.4} ice_true={:.4} poles_t/r={:.2} (point-sampled {:.2}) \
         subpolar_t/r={:.2} (point-sampled {:.2}) mid_t/r={:.2} \
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
        f(&a["subpolar_tangential_over_radial"]),
        f(&a["point_sampled_subpolar"]),
        f(&a["mid_latitudes_tangential_over_radial"]),
        f(&v["m_per_block_h"]),
        f(&v["millis"]["lambda_c"]),
        f(&v["millis"]["rasters"]),
        f(&v["millis"]["vector"]),
        f(&v["millis"]["total"]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The footprint mean of a continuous field over the simulation's Lambert blocks, sampled as
    /// [`resample_grid`] samples a layer (its spacing the source cell `r_s`).
    fn mean(f: &dyn Fn(f64, f64) -> f64, fp: &Footprint, r_s: f64) -> f64 {
        let nx = (1 + (2.0 * fp.hx / r_s).ceil() as usize).min(MAX_SAMPLES_X);
        let nz = (1 + (2.0 * fp.hz / r_s).ceil() as usize).min(MAX_SAMPLES_Z);
        let at = |k: usize, n: usize, h: f64| {
            if n == 1 {
                0.0
            } else {
                -h + 2.0 * h * k as f64 / (n - 1) as f64
            }
        };
        let mut sum = 0.0;
        for kz in 0..nz {
            for kx in 0..nx {
                sum += f(fp.xs + at(kx, nx, fp.hx), fp.zs + at(kz, nz, fp.hz));
            }
        }
        sum / (nx * nz) as f64
    }

    /// A body anywhere on the simulation's grid, the pole rows and the rows past them included, projects to a body the
    /// format accepts (positive finite semi-axes): an Earth-proportioned test planet's bake died on one at a singular spot.
    #[test]
    fn a_projected_body_always_has_positive_axes() {
        let c_sim = 4096;
        let g = PetroffGuyou::new(PetroffGuyou::hemisphere_for(c_sim), 30.0);
        let map = Mapping::new(g, c_sim, 16);
        let pole = deeptime_format::Projection::new(c_sim).pole_z();
        for iz in -12..=12 {
            for ix in -8..8 {
                let (x, z) = (f64::from(ix) * 256.0 + 3.0, f64::from(iz) / 12.0 * (pole + 64.0));
                let b = deeptime_format::Body::new(
                    1,
                    None,
                    deeptime_format::bodies::Shape::Blanket,
                    "ore",
                    [x, 40.0, z],
                    [6.0, 30.0, 9.0],
                    0.7,
                    0.0,
                    deeptime_format::bodies::NO_CLIP,
                );
                let p = project_body(&map, &b);
                assert!(p.axes.iter().all(|v| v.is_finite() && *v > 0.0), "{x} {z}: {:?}", p.axes);
            }
        }
    }

    /// The simulation's solvers work in Lambert blocks, so its polar terrain has the grid's shape
    /// (features a few cells across, k cos φ planet blocks wide and k / cos φ long). Resampled
    /// with a footprint isotropic on the sphere, such terrain still streaks radially round the
    /// poles (the tangential slope far above the radial one); widened along the parallels, it does
    /// not. A field of random plane waves of 6 to 24 Lambert blocks stands in for the terrain.
    #[test]
    fn lambert_shaped_polar_terrain_does_not_streak() {
        let c_sim = 4096;
        let g = PetroffGuyou::new(PetroffGuyou::hemisphere_for(c_sim), 30.0);
        let map = Mapping::new(g, c_sim, 16);
        let mut seed = 0x5EED_u64;
        let mut rnd = || {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        let waves: Vec<(f64, f64, f64)> = (0..12)
            .map(|_| {
                let len = 6.0 + 18.0 * rnd();
                let dir = std::f64::consts::TAU * rnd();
                let k = std::f64::consts::TAU / len;
                (k * dir.cos(), k * dir.sin(), std::f64::consts::TAU * rnd())
            })
            .collect();
        let field = |xs: f64, zs: f64| -> f64 {
            waves
                .iter()
                .map(|&(a, b, ph)| (a * xs + b * zs + ph).sin())
                .sum()
        };
        let (r_w, r_s) = (2.0, 2.0);
        let isotropic = |x: f64, z: f64| {
            mean(
                &field,
                &map.footprint_widened(x, z, r_w, r_s, &|_| 1.0),
                r_s,
            )
        };
        let widened = |x: f64, z: f64| mean(&field, &map.footprint(x, z, r_w, r_s), r_s);
        let (pole_iso, mid_iso, n) = pole_anisotropy_n(&map, &isotropic, 300);
        let (pole_wide, mid_wide, _) = pole_anisotropy_n(&map, &widened, 300);
        eprintln!(
            "tangential / radial slope round the poles: isotropic footprint {pole_iso:.2}, widened \
             {pole_wide:.2} (mid-latitudes {mid_iso:.2}, {mid_wide:.2}; {n} points)"
        );
        assert!(n > 200, "{n} points");
        assert!(
            pole_iso > 4.0,
            "the isotropic footprint streaks: {pole_iso}"
        );
        assert!(
            pole_wide < 2.5 && pole_wide > 0.2,
            "widened, no streaks and no rings: {pole_wide}"
        );
        // Below 60° nothing changes.
        assert!(
            (mid_iso - mid_wide).abs() < 0.2 * mid_iso,
            "{mid_iso} vs {mid_wide}"
        );
    }

    /// Flags and codes take the nearest cell: the wetland flag (0 none, 1 swamp, 2 cold) must not
    /// be averaged into swamp between cold wetland and dry land.
    #[test]
    fn flags_and_codes_are_categories() {
        let grid = deeptime_format::GridDesc::for_planet("L2", 1024, 8).unwrap();
        let desc =
            |name: &str, units: &str| crate::bake::desc(name, DType::U8, &grid, 1.0, units, None);
        for name in [
            "climate/wetland",
            "climate/koppen",
            "meta/province",
            "surface/class_top",
        ] {
            assert_eq!(kind_of(&desc(name, "")), Kind::Categorical, "{name}");
        }
        assert_eq!(kind_of(&desc("some/mask", "flag")), Kind::Categorical);
        assert_eq!(kind_of(&desc("elevation", "block")), Kind::Continuous);
        // The climate's band coordinates are read at the point, as the Köppen class beside them.
        assert_eq!(
            kind_of(&desc("climate/vegetation", "vanilla_param")),
            Kind::Point
        );
    }
}

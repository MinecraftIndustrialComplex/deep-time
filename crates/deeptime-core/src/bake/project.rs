// SPDX-License-Identifier: LGPL-3.0-or-later
//! The bake's projection pass (Ben, decisions log 2026-10-03: "Bake only"; 2026-10-24: "Cube
//! only"): the simulation runs on its own Lambert grid, the bake writes the world folder in that
//! frame, and this pass, the last step of `Session::finalize_write`, rewrites the folder as the cube
//! world ([`crate::bake::project_cube`]; FORMAT.md "The bake on a cube world", science page
//! `docs/science/cube-world-bake.md`).
//!
//! This module holds what the pass shares with its tests and the bake's other steps: how a layer
//! is resampled ([`Kind`]) and read ([`Src`]), the top raise of the world's horizon stacks
//! ([`raise_world_tops`]), the world's ground and the biome layers' land call
//! ([`settle_biome_layers`]), the drawn height ([`Drawn`]), and the vector data's projection
//! through a [`WorldMap`] (structure windows, bodies, sites). The Petroff–Guyou pass that came
//! before the cube (SCL-083 to SCL-090) left with the Lambert and Petroff–Guyou worlds (cube plan
//! §7.4).

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use rayon::prelude::*;
use serde_json::{Value, json};

use deeptime_format::fsutil::Flusher;
use deeptime_format::raster::{RasterRead, bilinear_weights, cover_of};
use deeptime_format::structure::Shape;
use deeptime_format::tilepack::write_stored_tiles;
use deeptime_format::{DType, GridDesc, LayerDesc, LoadedLayer, Manifest, Written};

use crate::bake::biomes;
use crate::bake::stored_tiles;
use crate::bake::strata::{Moved, key_shift};
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::sphere::around::conjugate_axes;
use crate::structure::bake_horizons::Horizons;

/// The version of the pass's record (`notes.bake.projection.version`).
pub const VERSION: u32 = 1;

/// The temporary border's inset inside the cube world's open edges, blocks.
pub const BORDER_INSET_BLOCKS: f64 = 1.0; // ledger: SCL-088 (game-choice)

/// The layers sampled as categories (the nearest cell): class and id codes, and flags (a layer whose
/// `units` is `flag` is one too, named here or not). `climate/koppen_l2` came a day after this list
/// and was averaged as a number until 2026-10-18: a class border's footprint mean named a class
/// neither side had (BSk beside Cfa read Csa to Csc), which the census and the biome rules' Köppen
/// conditions read (ledger SCL-124; SCL-090 then). `meta/crust_type` (2026-10-19) is a code: an average of 0 and
/// 1 at a coast would be no crust at all. `strata/lift` (2026-10-20) is a number but travels with
/// the horizon stacks, which are the nearest cell's ([`Kind::Stack`]): the deformed column rule
/// takes it for how far that cell's stack rose, so it must be the same cell's (STR-055).
/// `delta` (2026-10-23) is a number too, what a delta plain added to the ground, but it says where
/// the plain is: the world's plain is the nearest cell's ([`PLAIN_NEAREST`]), and a footprint mean
/// spread the layer over 5,747 cells beside the D5 world's plains, 4,808 of them not above the
/// sea's level (HYD-146). `floodplain` (2026-10-24) likewise says where a floodplain is, with nodata
/// off it, which a mean would spread or drop (HYD-152).
const CATEGORICAL: [&str; 10] = [
    "climate/koppen",
    "climate/koppen_l2",
    "climate/wetland",
    "surface/class_top",
    "meta/plate_id",
    "meta/crust_type",
    "meta/province",
    "strata/lift",
    crate::bake::water::DELTA_LAYER,
    crate::bake::water::FLOODPLAIN_LAYER,
];

/// The layers a world cell on a delta plain (its nearest simulation cell's `delta` above 0) takes
/// from that cell, as `delta` itself does. The plain is flat ground one block over the sea, its
/// height in (sea level, sea level + 1] and its river's course a block under it (HYD-136); the
/// bilinear read and the footprint mean blend its edge with the beach band and the sea beside it,
/// which lie a fraction of a block under the sea. On the D5 world 3,534 of the 24,313 world cells
/// on a plain fell under the sea that way (the mean 0.10 blocks under it at the median) and 1,048
/// more whose mean stood over it ended at or under it, so 79 % stood above the sea against the
/// bake's 97 %. The amplitude is 0 on the plain (`water`), and the world's is too. ledger: HYD-146
pub(crate) const PLAIN_NEAREST: [&str; 2] = ["elevation", "detail/amplitude"];

/// Per source of one grid, whether a world cell on a delta plain takes its nearest cell
/// ([`PLAIN_NEAREST`]), and the grid's `delta` source; all false and none without a plain layer.
pub(crate) fn plain_rule(srcs: &[Src]) -> (Vec<bool>, Option<usize>) {
    let delta = srcs
        .iter()
        .position(|s| s.layer.desc.name == crate::bake::water::DELTA_LAYER);
    let takes = srcs
        .iter()
        .map(|s| delta.is_some() && PLAIN_NEAREST.contains(&s.layer.desc.name.as_str()))
        .collect();
    (takes, delta)
}

/// Whether simulation block (xs, zs) lies on a delta plain: its nearest cell of `delta` is raised.
#[inline]
pub(crate) fn on_plain(delta: Option<&Src>, xs: f64, zs: f64) -> bool {
    delta.is_some_and(|d| {
        let (ci, cj) = d.nearest_cell(xs, zs);
        d.raw(i64::from(ci), i64::from(cj)) > 0.0
    })
}

/// How a layer is resampled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
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
    /// A cover layer (`climate/ice_cover`), by the cover rule (`deeptime_format::raster::cover_of`):
    /// the world cell is covered where the nearest cell is, the same cell the categories beside it
    /// read (so a covered world cell is of the cover's class, `climate/koppen_l2` EF), at the mean
    /// depth of the footprint's samples that are covered, each read by the rule at its block. The
    /// footprint mean of the blocks covered every world cell that one covered sample reached (one
    /// block from a 12-block margin's twelfth), so the cover's area grew with the footprint, most
    /// where it was widest, at the Petroff–Guyou poles (ledger BIO-159; SCL-090, retired).
    Cover,
}

/// The ice cover layer (blocks of snow and ice, `bake::ice`).
pub(crate) const ICE_COVER: &str = "climate/ice_cover";

/// The layers read by the cover rule ([`Kind::Cover`]).
const COVER: [&str; 1] = [ICE_COVER];

pub(crate) fn kind_of(desc: &LayerDesc) -> Kind {
    if desc.dtype == DType::HorizonStack {
        Kind::Stack
    } else if COVER.contains(&desc.name.as_str()) {
        Kind::Cover
    } else if CATEGORICAL.contains(&desc.name.as_str()) || desc.units == "flag" {
        Kind::Categorical
    } else if desc.units == "vanilla_param" {
        Kind::Point
    } else {
        Kind::Continuous
    }
}

pub(crate) fn err(m: impl Into<String>) -> CoreError {
    CoreError::InvalidState(m.into())
}

pub(crate) fn ms(t: Instant) -> f64 {
    (t.elapsed().as_secs_f64() * 10_000.0).round() / 10.0
}

/// A loaded source layer.
pub(crate) struct Src {
    pub(crate) entry: usize,
    /// Shared with the pass's own copy of the elevation and amplitude layers (one load of each).
    pub(crate) layer: Arc<LoadedLayer>,
    pub(crate) kind: Kind,
    pub(crate) cb: usize,
}

impl Src {
    /// The raw value of cell (i, j) (columns wrap, rows clamp), NaN for no-data.
    #[inline]
    pub(crate) fn raw(&self, i: i64, j: i64) -> f64 {
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
    pub(crate) fn nearest_cell(&self, x: f64, z: f64) -> (u32, u32) {
        let g = &self.layer.desc.grid;
        let u = (x - g.origin_block_x as f64) / f64::from(g.r);
        let v = (z - g.origin_block_z as f64) / f64::from(g.r);
        g.cell((u + 0.5).floor() as i64, (v + 0.5).floor() as i64)
    }

    /// Bilinear raw value at block (x, z) (the arithmetic of `RasterRead::bilinear`).
    #[inline]
    pub(crate) fn bilinear(&self, x: f64, z: f64) -> f64 {
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

    /// The cover rule's raw value at block (x, z) (the arithmetic of `RasterRead::cover`).
    #[inline]
    pub(crate) fn cover(&self, x: f64, z: f64) -> f64 {
        let g = &self.layer.desc.grid;
        let u = (x - g.origin_block_x as f64) / f64::from(g.r);
        let v = (z - g.origin_block_z as f64) / f64::from(g.r);
        let (fi, fj) = (u.floor(), v.floor());
        let (i0, j0) = (fi as i64, fj as i64);
        let w = bilinear_weights(u - fi, v - fj);
        let (ci, cj) = self.nearest_cell(x, z);
        cover_of(
            self.raw(i64::from(ci), i64::from(cj)),
            [
                (self.raw(i0, j0), w[0]),
                (self.raw(i0 + 1, j0), w[1]),
                (self.raw(i0, j0 + 1), w[2]),
                (self.raw(i0 + 1, j0 + 1), w[3]),
            ],
        )
    }
}

/// Writes raw value `v` of `dtype` into `out` (integers rounded half away from zero and clamped
/// to the type's range).
pub(crate) fn put_raw(dtype: DType, v: f64, out: &mut [u8]) {
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

/// Applies a Jacobian (as [`WorldMap::jacobian`] returns it) to a simulation vector (dx, dz).
#[inline]
pub(crate) fn apply(j: &[[f64; 2]; 2], v: (f64, f64)) -> (f64, f64) {
    (j[0][0] * v.0 + j[1][0] * v.1, j[0][1] * v.0 + j[1][1] * v.1)
}

/// What the vector data's projection reads of a world (structure windows, bodies, sites): where
/// a simulation block lands, the local Jacobian there and the world's x period: the cube world's
/// (`bake::project_cube::CubeMapping`).
pub trait WorldMap {
    /// The world point of simulation block (xs, zs).
    fn to_world(&self, xs: f64, zs: f64) -> (f64, f64);
    /// World per simulation block at (xs, zs): [[dx/dxs, dz/dxs], [dx/dzs, dz/dzs]].
    fn jacobian(&self, xs: f64, zs: f64) -> [[f64; 2]; 2];
    /// The world's x period, blocks.
    fn period(&self) -> i64;
}

/// Bytes per cell of the layers the bake writes on each grid (level, name, bytes), summed over the
/// layers (the pass loads all of one grid's layers at once): L1 holds the horizon stacks, L2 and L3
/// a few scalars (L3's 13 include the delta plains' raise, 2 B, `water::DELTA_LAYER`, the
/// floodplains' change, 2 B, `water::FLOODPLAIN_LAYER`, and the fans' raise, 2 B,
/// `water::FAN_LAYER`). Measured on a
/// baked planet (`the_projection_estimate_covers_the_baked_layers` fails if the bake outgrows
/// them); a figure of the memory model, not a physical one.
pub(crate) const LAYER_BYTES_PER_CELL: [(u32, &str, u64); 3] =
    [(1, "L1", 288), (2, "L2", 16), (3, "L3", 13)];

/// Of L3's bytes a cell, the water's layers (`water_level` 2 B, `playa` and `closed_basin` 1 B
/// each): they are on the hydrology's grid, which is L3 halved where L3 is over
/// `hydro::MAX_CELLS` (`water::bake`; 32k full), a grid of its own then ("L3h").
pub(crate) const WATER_LAYER_BYTES_PER_CELL: u64 = 4;

/// Bytes per world cell of the hydrology's grid the pass holds beside that grid's layers while it
/// draws the closed basins on the world's own ground (HYD-105): the ground as f64
/// ([`world_ground`]) and `hydro::closed_basin_on_world`'s flags, 1 B each, and fronts. A figure of
/// the memory model (amp-est, 2026-10-23: the ground and the working set held 25.1 B a cell on
/// quick 16k's world grid of 22.2 M cells, 37.3 B where the ocean passed 2^24 cells, the bake's
/// peak, while the closed basins' flood kept every body cell and a depth-first stack as usize;
/// with a breadth-first front of u32 and a state byte a cell, 11.0 B: 8.0 for the ground, 3.0 for
/// the flood).
pub(crate) const GROUND_BYTES_PER_WORLD_CELL: u64 = 12;

/// Bytes per world cell of the elevation's grid the block-scale relief holds beside the grid's
/// layers (`relief::OnWorld::draw`: its f32 change and the calm flag; 5.0 B held on quick 16k).
pub(crate) const RELIEF_BYTES_PER_WORLD_CELL: u64 = 6;

/// Bytes per Fibonacci point of the cube's turn search (`cube_turn::Land` and `cube_turn::search`:
/// the points, their land flags, the two k-d trees and the copies they are built from, and the
/// search's candidates). Cube plan §7.4, 2026-10-24: the memory_estimate test planet's bake peaked
/// in the search, 45.9 MiB above the loaded sources at 400,000 points (120 B a point), a figure the
/// grids' steps left out while the estimate was checked on the Petroff-Guyou world only.
pub(crate) const TURN_BYTES_PER_SAMPLE: u64 = 128;

/// The memory the projection pass holds beyond the finished world (bytes), for the run's
/// estimate (`session::estimate_for`).
///
/// The pass takes the grids in turn (L1, L2, L3, then the hydrology's halved grid if there is one),
/// holding the grid's source layers, its resampled layers and one band of rows ([`ROW_BAND`]) or,
/// after the band, what it draws on that grid: the closed basins on the hydrology's grid
/// ([`GROUND_BYTES_PER_WORLD_CELL`]), the relief on the elevation's
/// ([`RELIEF_BYTES_PER_WORLD_CELL`]). Before the grids comes the cube's turn search
/// ([`TURN_BYTES_PER_SAMPLE`]). Beside them stay the Lambert elevation and amplitude it loads
/// first (L3, shared with that grid's sources, kept to the end) and the world layers of the grids
/// before: the horizon stacks (kept for the top raise), the L2 layers the relief and the biomes'
/// land call read, and the world's elevation (the halved grid's ground). Last comes the top
/// raise on the horizon stacks and their write, beside all of it. Memory accounting, 2026-10-05:
/// before this the pass was not in the estimate and kept a second copy of every resampled layer
/// beside its output. amp-est, 2026-10-23: the closed basins' ground and working set were left
/// out (452.6 MB listed for a pass that held 969.9 MB on quick 16k), and the water's layers were
/// counted on L3 where the hydrology's grid is halved.
pub fn estimate_bytes(p: &PlanetPreset) -> u64 {
    let c = p.scale.circumference_blocks;
    // The cube world's grids: n = s / r cells a face side, six faces.
    let s = u64::from(deeptime_format::projection::face_for(c));
    let src_cells =
        |r: u32| GridDesc::for_planet("", c, r).map_or(0, |g| u64::from(g.w) * u64::from(g.h));
    let world_width = |r: u32| s / u64::from(r.max(1));
    let world_cells = |r: u32| 6 * world_width(r) * world_width(r);
    let r3 = p.resolution.l3_r;
    let hydro_r = crate::bake::hydrology_r(p);
    let l3 = LAYER_BYTES_PER_CELL[2].2;
    let terrain = l3 - WATER_LAYER_BYTES_PER_CELL;
    // (grid, cell size, layer bytes a cell), in the pass's order.
    let mut grids = vec![
        (1, p.resolution.l1_r, LAYER_BYTES_PER_CELL[0].2),
        (2, p.resolution.l2_r, LAYER_BYTES_PER_CELL[1].2),
        (3, r3, if hydro_r == r3 { l3 } else { terrain }),
    ];
    if hydro_r != r3 {
        grids.push((4, hydro_r, WATER_LAYER_BYTES_PER_CELL));
    }
    // The Lambert elevation and amplitude, loaded before the first grid (a part of the L3 sources).
    let kept_sources = src_cells(r3) * terrain;
    // The turn search on them, before the first grid (its points are dropped after the land
    // statistics).
    let turn = crate::bake::cube_turn::BAKE.samples as u64 * TURN_BYTES_PER_SAMPLE;
    let (mut worst, mut kept) = (kept_sources + turn, 0u64);
    for (level, r, per_cell) in grids {
        let (src, out) = (src_cells(r) * per_cell, world_cells(r) * per_cell);
        let mut work = ROW_BAND as u64 * world_width(r) * per_cell;
        if level >= 3 && r == hydro_r {
            work = work.max(world_cells(r) * GROUND_BYTES_PER_WORLD_CELL);
        }
        if level == 3 {
            work = work.max(world_cells(r) * RELIEF_BYTES_PER_WORLD_CELL);
        }
        let resident = if level == 3 { 0 } else { kept_sources };
        worst = worst.max(resident + kept + src + out + work);
        kept += out;
    }
    // The top raise, after every grid, beside their layers ([`raise_world_tops`]; amp-est,
    // 2026-10-23: the bake's peak on the memory_estimate test planet, which the steps above left
    // out). Its parts follow each other: the raise (the stack's two maxima as f64 and its record
    // of moved cells, 16 B a cell, three times over while the record grows), the shifts
    // (`world_raise_shifts`: a copy of the stacks and a dense record), then the stack's write
    // (`bake::stored_tiles`: 1.1 × the stack and one batch, and the writer's buffer).
    let horizons = p.structure.as_ref().map_or(0, |s| u64::from(s.horizons));
    if horizons > 0 {
        let cells = world_cells(p.resolution.l1_r);
        let stack_cell = 7 * horizons;
        let stack = cells * stack_cell;
        let tile = u64::from(deeptime_format::raster::TILE).pow(2) * stack_cell;
        let batch = (crate::bake::STORE_BATCH_BYTES / tile).max(1) * tile;
        let raise = cells * (16 + 3 * 16);
        let shifts = cells * (stack_cell + 12 + 16);
        let write = (stack + batch.min(stack)) * 11 / 10 + (1 << 20);
        worst = worst.max(kept_sources + kept + raise.max(shifts).max(write));
    }
    worst
}

/// Rows of the world grid resampled at a time (their results are copied out before the next band).
pub(crate) const ROW_BAND: usize = 64;

/// Writes a layer's tile pack (its tiles compressed in parallel) to its temporary name, its fsync
/// started; `flusher`'s commit puts it in place (P1-19).
pub(crate) fn write_deferred(
    flusher: &mut Flusher,
    path: &Path,
    desc: &LayerDesc,
    cells: &[u8],
) -> Result<Written, CoreError> {
    let tiles = stored_tiles(desc, cells)?;
    flusher.write(path, |f| -> Result<Written, CoreError> {
        use std::io::Write;
        let mut w = std::io::BufWriter::with_capacity(1 << 20, f);
        let written = write_stored_tiles(&mut w, desc, &tiles)?;
        w.flush().map_err(|e| CoreError::io(path, e))?;
        Ok(written)
    })
}

/// Row weights of a world grid for area shares: the edge rows count half (the Petroff–Guyou
/// world's glued rows were half cells; the cube's records kept the rule).
fn world_row_weight(j: usize, h: usize) -> f64 {
    if j == 0 || j + 1 == h { 0.5 } else { 1.0 }
}

/// The share of a layer's area where `pred` holds, on the world grid (`world` = true) or on the
/// simulation's Lambert grid (true area: rows weighted by their share of the planet).
pub(crate) fn area_share(layer: &LoadedLayer, world: bool, pred: impl Fn(f64) -> bool) -> f64 {
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

/// Raises the first horizon of every world stack cell to the bake's top rule: at least the highest
/// world elevation plus detail amplitude over the L3 cells of the 5 × 5 stack cells round it, plus
/// `TOP_MARGIN_BLOCKS`, capped at `max_y` (`bake::strata::raise_tops` on the simulation's grid),
/// the thickest rock unit near the surface taking up the raise (`bake::strata::raise_stack`).
/// On a cube grid the window runs across the faces' edges; on the simulation's Lambert frame
/// (the tests' grids) its columns wrap and its rows stop at the edge.
/// Returns (cells raised, largest raise in blocks, and with `record` what the raise did to each
/// raised cell: its index and `Moved`, no lift, the raise and the last horizon it moved).
pub(crate) fn raise_world_tops(
    desc: &LayerDesc,
    bytes: &mut [u8],
    elev: &LoadedLayer,
    amp: Option<&LoadedLayer>,
    min_y: i32,
    max_y: i32,
    record: bool,
) -> (u64, f64, Vec<(u32, Moved)>) {
    use crate::bake::strata::{TOP_MARGIN_BLOCKS, raise_stack_unit};
    use crate::structure::bake_horizons::{NO_CLASS, NO_TOP};
    let unconsolidated = crate::structure::columns::Tables::get().unconsolidated;
    let k = desc.stack_depth.unwrap_or(0) as usize;
    let g1 = &desc.grid;
    let (w1, h1) = (g1.w as usize, g1.h as usize);
    let g3 = &elev.desc.grid;
    if g3.r == 0 || !g1.r.is_multiple_of(g3.r) {
        return (0, 0.0, Vec::new());
    }
    let ratio = (g1.r / g3.r) as usize;
    let (w3, h3) = (g3.w as usize, g3.h as usize);
    // Per stack cell, the highest value of the L3 cells it holds (cells i·ratio .. i·ratio + ratio).
    // A stack row in parallel (P1-19): row j holds the L3 rows j·ratio .. j·ratio + ratio, the
    // last row every L3 row past it as well; a maximum is the same in any order.
    let square = |l: &LoadedLayer| -> Vec<f64> {
        let cb = l.desc.cell_bytes();
        let mut best = vec![f64::NEG_INFINITY; w1 * h1];
        best.par_chunks_mut(w1).enumerate().for_each(|(j, row)| {
            let rows = if j + 1 == h1 {
                j * ratio..h3
            } else {
                j * ratio..((j + 1) * ratio).min(h3)
            };
            for j3 in rows {
                for i3 in 0..w3 {
                    let i = (i3 / ratio).min(w1 - 1);
                    let at = (j3 * w3 + i3) * cb;
                    let raw = l.desc.dtype.decode_scalar(&l.bytes[at..at + cb]);
                    if l.desc.nodata == Some(raw) {
                        continue;
                    }
                    let v = raw * l.desc.scale + l.desc.offset;
                    let c = &mut row[i];
                    if v > *c {
                        *c = v;
                    }
                }
            }
        });
        best
    };
    let e = square(elev);
    let a = amp.map_or_else(|| vec![0.0; w1 * h1], square);
    let cube = g1.is_cube();
    let at = |v: &[f64], i: i64, j: i64, di: i64, dj: i64| -> f64 {
        // On a cube grid the window runs across the face's edges onto its neighbours, turned
        // with them ([`GridDesc::face_cell`], SCL-122).
        if cube {
            let (face, a, b) = g1.face_of_cell(i as u32, j as u32);
            let (ci, cj) = g1.face_cell(face, i64::from(a) + di, i64::from(b) + dj);
            return v[cj as usize * w1 + ci as usize];
        }
        let j = (j + dj).clamp(0, h1 as i64 - 1) as usize;
        let i = (i + di).rem_euclid(w1 as i64) as usize;
        v[j * w1 + i]
    };
    let cb = desc.cell_bytes();
    // A row in parallel (P1-19): each cell reads only `e` and `a`; the count is a sum of whole
    // numbers and the largest raise a maximum, so both are the same in any order, and the
    // record's rows join in order.
    bytes
        .par_chunks_mut(w1 * cb)
        .enumerate()
        .map(|(j, row)| {
            let (mut raised, mut max_raise, mut cells) = (0u64, 0.0f64, Vec::new());
            for i in 0..w1 {
                let (mut me, mut ma) = (f64::NEG_INFINITY, 0.0f64);
                // A window symmetric about the cell (the bake's is i − 2 ..= i + 1 on square
                // cells; these are point samples).
                for dj in -2i64..=2 {
                    for di in -2i64..=2 {
                        me = me.max(at(&e, i as i64, j as i64, di, dj));
                        ma = ma.max(at(&a, i as i64, j as i64, di, dj));
                    }
                }
                if !me.is_finite() {
                    continue;
                }
                let need = ((me + ma).ceil() as i32 + TOP_MARGIN_BLOCKS).min(max_y) * 16;
                let off = i * cb;
                let top = i16::from_le_bytes([row[off], row[off + 1]]);
                if top == NO_TOP || i32::from(top) >= need || k == 0 {
                    continue;
                }
                // The cell's stack: K tops (i16), K classes, K key tops, K key bases (u16), as
                // `Horizons::to_layer` writes it.
                let cell = &mut row[off..off + cb];
                let classes: Vec<u8> = cell[2 * k..3 * k].to_vec();
                let n = classes.iter().position(|&x| x == NO_CLASS).unwrap_or(k);
                let u16_at = |c: &[u8], at: usize| u16::from_le_bytes([c[at], c[at + 1]]);
                let keys: Vec<[u16; 2]> = (0..n)
                    .map(|h| [u16_at(cell, 3 * k + 2 * h), u16_at(cell, 5 * k + 2 * h)])
                    .collect();
                let mut tops: Vec<i16> = (0..n)
                    .map(|h| i16::from_le_bytes([cell[2 * h], cell[2 * h + 1]]))
                    .collect();
                let (raise, upto) = raise_stack_unit(
                    &mut tops,
                    &classes[..n],
                    &keys,
                    need,
                    min_y * 16,
                    &unconsolidated,
                );
                for (h, t) in tops.iter().enumerate() {
                    cell[2 * h..2 * h + 2].copy_from_slice(&t.to_le_bytes());
                }
                raised += 1;
                max_raise = max_raise.max(f64::from(raise) / 16.0);
                if record && raise > 0 {
                    let moved = Moved {
                        lift: 0,
                        raise,
                        upto: upto as u16,
                    };
                    cells.push(((j * w1 + i) as u32, moved));
                }
            }
            (raised, max_raise, cells)
        })
        .reduce(
            || (0, 0.0, Vec::new()),
            |mut x, y| {
                x.2.extend(y.2);
                (x.0 + y.0, x.1.max(y.1), x.2)
            },
        )
}

/// The world's horizon stacks as [`Horizons`], from their layer's bytes (per cell K i16 tops, K
/// u8 classes, K u16 key tops, K u16 key bases, as `Horizons::to_layer` writes them).
fn horizons_of(desc: &LayerDesc, bytes: &[u8], min_y: i32) -> Horizons {
    let k = desc.stack_depth.unwrap_or(0) as usize;
    let cb = desc.cell_bytes();
    let n = bytes.len().checked_div(cb).unwrap_or(0);
    let (mut tops, mut classes, mut keys) = (
        Vec::with_capacity(n * k),
        Vec::with_capacity(n * k),
        Vec::with_capacity(n * k),
    );
    let u16_at = |c: &[u8], at: usize| u16::from_le_bytes([c[at], c[at + 1]]);
    for cell in bytes.chunks_exact(cb.max(1)).take(n) {
        tops.extend((0..k).map(|h| i16::from_le_bytes([cell[2 * h], cell[2 * h + 1]])));
        classes.extend_from_slice(&cell[2 * k..3 * k]);
        keys.extend((0..k).map(|h| [u16_at(cell, 3 * k + 2 * h), u16_at(cell, 5 * k + 2 * h)]));
    }
    Horizons {
        grid: desc.grid.clone(),
        k: k as u32,
        min_y,
        tops,
        classes,
        keys,
        tongues: (
            desc.tongue_band_keys.unwrap_or(1),
            desc.tongue_minor_share.unwrap_or(0.0),
        ),
        min_blocks: desc.min_blocks.clone().unwrap_or_default(),
    }
}

/// How far the world's top raise ([`raise_world_tops`], its record `moved`) moved the rock at each
/// of `points` (world blocks x, y, z), blocks: the bake's own rule for what rises with the rock
/// ([`key_shift`], STR-056) on the raised world stacks (`desc`, `bytes`). The raise lifts the
/// thickest rock unit near the surface and every horizon over it, so a stratabound body in one of
/// those horizons rises with it and keeps its host (ORE-271). Zero everywhere when nothing rose.
pub(crate) fn world_raise_shifts(
    desc: &LayerDesc,
    bytes: &[u8],
    min_y: i32,
    moved: &[(u32, Moved)],
    points: &[[f64; 3]],
) -> Vec<f64> {
    if moved.is_empty() || points.is_empty() {
        return vec![0.0; points.len()];
    }
    let hz = horizons_of(desc, bytes, min_y);
    let mut dense = vec![Moved::default(); hz.cells()];
    for &(c, m) in moved {
        if let Some(d) = dense.get_mut(c as usize) {
            *d = m;
        }
    }
    points
        .par_iter()
        .map(|p| key_shift(&hz, &dense, p[0], p[1], p[2]))
        .collect()
}

/// Rewrites the Lambert-frame world folder `dir` (its manifest `m`) as the cube world
/// ([`crate::bake::project_cube::project_folder_cube`]). Returns `notes.bake.projection`.
pub fn project_folder(dir: &Path, p: &PlanetPreset, m: &mut Manifest) -> Result<Value, CoreError> {
    crate::bake::project_cube::project_folder_cube(dir, p, m)
}

/// The world's ground on grid `wg`, blocks: the elevation layer's own resampled cells when it is
/// on this grid (`here`), else the nearest cell of the world's elevation written before (the
/// water grid `L3h` of a world too large for the hydrology at L3). None without either.
pub(crate) fn world_ground(
    wg: &GridDesc,
    here: Option<(&LayerDesc, &[u8])>,
    world_elev: Option<&LoadedLayer>,
) -> Option<Vec<f64>> {
    let n = wg.w as usize * wg.h as usize;
    if let Some((desc, bytes)) = here {
        let cb = desc.cell_bytes();
        return Some(
            (0..n)
                .into_par_iter()
                .map(|c| desc.physical(desc.dtype.decode_scalar(&bytes[c * cb..(c + 1) * cb])))
                .collect(),
        );
    }
    let we = world_elev?;
    let (w, r) = (wg.w as usize, i64::from(wg.r));
    Some(
        (0..n)
            .into_par_iter()
            .map(|c| {
                // A cube cell's point is its centre (a reader takes the block under it).
                if wg.is_cube() {
                    let (px, pz) = wg.cube_point((c % w) as u32, (c / w) as u32);
                    return we.nearest(px - 0.5, pz - 0.5);
                }
                let (i, j) = ((c % w) as i64, (c / w) as i64);
                we.nearest(
                    (wg.origin_block_x + i * r) as f64,
                    (wg.origin_block_z + j * r) as f64,
                )
            })
            .collect(),
    )
}

/// A resampled layer kept back (moved, not copied) until the pass has the ground it waits for.
pub(crate) struct Held {
    pub(crate) entry: usize,
    pub(crate) desc: LayerDesc,
    pub(crate) bytes: Vec<u8>,
}

/// The L2 layers the biome bake's land call decides (BIO-170): the five biome inputs, the wetland
/// flag and the Köppen class the inputs were placed by, and the ice cover, which it takes off the
/// nodes the world's sea covers (BIO-177).
pub(crate) fn is_biome_layer(name: &str) -> bool {
    crate::synth::CLIMATE.contains(&name)
        || name == "climate/wetland"
        || name == "climate/koppen_l2"
        || name == ICE_COVER
}

/// What the biome layers' land call reads: the world's ground.
pub(crate) struct BiomeGround<'a> {
    pub(crate) sea_level_y: i32,
    pub(crate) m_per_block_v: f64,
    /// The size of a world block, metres.
    pub(crate) world_m_per_block_h: f64,
    /// How many of the bake's (Lambert) blocks one world block spans in ground.
    pub(crate) bake_blocks_per_world_block: f64,
    /// The world's elevation layer (blocks), the L3 ground the column rule reads.
    pub(crate) elevation: &'a LoadedLayer,
    /// The world's `closed_basin` layer (HYD-105's redraw), if the bake has the hydrology.
    pub(crate) closed: Option<&'a LoadedLayer>,
    /// The world's `playa` layer, if the bake has the hydrology: no swamp beside its salt (BIO-178).
    pub(crate) playa: Option<&'a LoadedLayer>,
    /// The world's `delta` layer, if the bake has delta plains: no ocean biome on them (BIO-185).
    pub(crate) delta: Option<&'a LoadedLayer>,
    /// A cube world's view and the simulation's radius (bake blocks): the land call's fills and
    /// distances run across the faces' seams, on the planet (SCL-122).
    pub(crate) cube: Option<(&'a deeptime_format::cube_map::CubeView<'static>, f64)>,
}

/// The land call of the biome layers on the world's ground (ledger: BIO-170, BIO-171, BAK-057, HYD-108):
/// the rule is the column evaluator's own ([`deeptime_format::water::sea_covers_height`]) on the
/// world's elevation and `closed_basin` layers at each L2 node's block; the nodes whose biome inputs
/// say the other medium take the other's record ([`biomes::settle_to_ground`]); the rest are not
/// touched. The ice cover, when held with them, is taken off every node the world's sea covers
/// (BIO-177). Returns the record for `notes.bake.projection.biome_land`.
pub(crate) fn settle_biome_layers(
    held: &mut [Held],
    ground: &BiomeGround<'_>,
    params: &biomes::BiomeParams,
) -> Value {
    let find = |name: &str| held.iter().position(|h| h.desc.name == name);
    let names = crate::synth::CLIMATE;
    let idx: Vec<usize> = match names.iter().map(|n| find(n)).collect::<Option<Vec<_>>>() {
        Some(v) => v,
        None => return json!({"skipped": "the bake has no climate layers"}),
    };
    let wet_idx = find("climate/wetland");
    let class_idx = find("climate/koppen_l2");
    let cover_idx = find(ICE_COVER);
    let wg = held[idx[0]].desc.grid.clone();
    let (w, h) = (wg.w as usize, wg.h as usize);
    let n = w * h;
    // The layers, as physical values (the five) and raw codes (the two).
    let physical = |hd: &Held| -> Vec<f32> {
        let cb = hd.desc.cell_bytes();
        (0..n)
            .into_par_iter()
            .map(|c| {
                hd.desc
                    .physical(hd.desc.dtype.decode_scalar(&hd.bytes[c * cb..(c + 1) * cb]))
                    as f32
            })
            .collect()
    };
    let codes = |hd: &Held| -> Vec<u8> {
        let cb = hd.desc.cell_bytes();
        (0..n)
            .map(|c| hd.desc.dtype.decode_scalar(&hd.bytes[c * cb..(c + 1) * cb]) as u8)
            .collect()
    };
    let mut cells = biomes::BiomeCells {
        temperature: physical(&held[idx[0]]),
        vegetation: physical(&held[idx[1]]),
        continents: physical(&held[idx[2]]),
        erosion: physical(&held[idx[3]]),
        ridges: physical(&held[idx[4]]),
        wetland: wet_idx.map_or_else(|| vec![biomes::WETLAND_NONE; n], |k| codes(&held[k])),
        class: class_idx.map_or_else(Vec::new, |k| codes(&held[k])),
    };
    // The world's rule at every node: the sea covers it, and how deep.
    let mut sea = vec![false; n];
    let mut depth_m = vec![0f32; n];
    sea.par_chunks_mut(w)
        .zip(depth_m.par_chunks_mut(w))
        .enumerate()
        .for_each(|(j, (srow, drow))| {
            let z = wg.block_z(j as i64) as f64;
            for (i, (s, d)) in srow.iter_mut().zip(drow.iter_mut()).enumerate() {
                let x = wg.block_x(i as i64) as f64;
                // A cube node's block is the one under its cell's centre.
                let (x, z) = if wg.is_cube() {
                    let (px, pz) = wg.cube_point(i as u32, j as u32);
                    (px - 0.5, pz - 0.5)
                } else {
                    (x, z)
                };
                let height = ground.elevation.nearest(x, z);
                let closed = || ground.closed.is_some_and(|c| c.nearest(x, z) >= 0.5);
                *s = deeptime_format::water::sea_covers_height(height, ground.sea_level_y, closed);
                *d = ((f64::from(ground.sea_level_y) - height) * ground.m_per_block_v) as f32;
            }
        });
    let land_nodes = sea.iter().filter(|&&s| !s).count();
    let blocks_per_cell = f64::from(wg.r) * ground.bake_blocks_per_world_block;
    let cube = ground.cube.map(|(view, rs)| {
        crate::bake::project_cube::CubeSettle::new(&wg, view, rs / blocks_per_cell)
    });
    let settle = |cells: &mut biomes::BiomeCells, sea: &[bool]| match &cube {
        Some(cs) => biomes::settle_to_ground_by(
            cells,
            sea,
            &depth_m,
            blocks_per_cell,
            params,
            &|v, src| cs.fill(v, src),
            &|land, to_sea| cs.dist(land, to_sea),
        ),
        None => biomes::settle_to_ground(cells, w, h, sea, &depth_m, blocks_per_cell, params),
    };
    let settled = settle(&mut cells, &sea);
    // No ocean biome on a delta plain's land, on the world's own nodes after the land call
    // (BIO-185). The land call reads the ground at a node's block, and the game reads the biome
    // inputs bilinear between a column's four nodes: a plain is a few L2 cells across, so the
    // plain's columns beside the sea read a sea node's continentalness and drew an ocean biome on
    // dry ground. A node is the plain's when a dry column of a plain lies within a cell of it
    // either way (the columns whose read it reaches, at the plain's cells); the land call runs
    // again with those nodes dry, so the ones the sea still held take the land's record by its
    // own law (the nearest land node's, the continentalness by the distance to the sea), and the
    // sea's columns beside the plain read the coast's. ledger: BIO-185
    let (mut plain_nodes, mut plain_settled) = (0usize, biomes::Settled::default());
    if let Some(dl) = ground.delta {
        let r2 = f64::from(wg.r);
        let r3 = f64::from(dl.desc.grid.r);
        let reach = ((r2 / r3).round() as i64 - 1).max(0);
        let dry_plain = |x: f64, z: f64| {
            dl.nearest(x, z) > 0.0
                && !deeptime_format::water::sea_covers_height(
                    ground.elevation.nearest(x, z),
                    ground.sea_level_y,
                    || ground.closed.is_some_and(|c| c.nearest(x, z) >= 0.5),
                )
        };
        let at: Vec<bool> = (0..n)
            .into_par_iter()
            .map(|k| {
                let (i, j) = (k % w, k / w);
                let (x, z) = if wg.is_cube() {
                    let (px, pz) = wg.cube_point(i as u32, j as u32);
                    (px - 0.5, pz - 0.5)
                } else {
                    (wg.block_x(i as i64) as f64, wg.block_z(j as i64) as f64)
                };
                (-reach..=reach).any(|a| {
                    (-reach..=reach).any(|b| dry_plain(x + a as f64 * r3, z + b as f64 * r3))
                })
            })
            .collect();
        plain_nodes = at.iter().filter(|&&a| a).count();
        if plain_nodes > 0 {
            let dry: Vec<bool> = sea.iter().zip(&at).map(|(&s, &a)| s && !a).collect();
            plain_settled = settle(&mut cells, &dry);
        }
    }
    // No swamp on or beside a playa, on the world's own nodes after the land call (BIO-178): the
    // projection's reads and the land call's donors bring a swamp's erosion and flag onto ground
    // the bake's own clearing (`water`, on the Lambert nodes) did not see. A node holds a playa
    // when a playa cell lies in its square; the reach is the margin, the game's bilinear read
    // (a block's four nodes, up to a cell and a half from the node marked for its playa cell, so
    // two cells) and a playa cell. ledger: BIO-178
    let (mut dried, mut playa_nodes, mut reach_blocks) = (Vec::new(), 0usize, 0.0);
    if let Some(pl) = ground.playa {
        let r2 = f64::from(wg.r);
        let r3 = f64::from(pl.desc.grid.r);
        let m = ((r2 / r3).round() as usize).max(1);
        let step = r2 / m as f64;
        let at: Vec<bool> = (0..n)
            .into_par_iter()
            .map(|k| {
                let (i, j) = (k % w, k / w);
                let (x, z) = if wg.is_cube() {
                    wg.cube_point(i as u32, j as u32)
                } else {
                    (wg.block_x(i as i64) as f64, wg.block_z(j as i64) as f64)
                };
                (0..m * m).any(|s| {
                    let dx = -0.5 * r2 + ((s % m) as f64 + 0.5) * step;
                    let dz = -0.5 * r2 + ((s / m) as f64 + 0.5) * step;
                    pl.nearest(x + dx, z + dz) > 0.0
                })
            })
            .collect();
        playa_nodes = at.iter().filter(|&&a| a).count();
        reach_blocks = biomes::PLAYA_WETLAND_MARGIN_BLOCKS + 2.0 * r2 + r3;
        if playa_nodes > 0 {
            let elsewhere: Vec<bool> = at.iter().map(|&a| !a).collect();
            let from_playa = match &cube {
                Some(cs) => cs.dist(&elsewhere, true),
                None => biomes::distance_cells(&elsewhere, w, h, true),
            };
            dried = biomes::dry_beside_playa_nodes(&mut cells, &from_playa, reach_blocks / r2);
        }
    }
    // Write the settled and dried nodes back, and only those.
    let changed: Vec<u32> = settled
        .to_sea
        .iter()
        .chain(&settled.to_land)
        .chain(&plain_settled.to_sea)
        .chain(&plain_settled.to_land)
        .chain(&dried)
        .copied()
        .collect();
    let five: [&Vec<f32>; 5] = [
        &cells.temperature,
        &cells.vegetation,
        &cells.continents,
        &cells.erosion,
        &cells.ridges,
    ];
    for (&li, values) in idx.iter().zip(five) {
        let hd = &mut held[li];
        let cb = hd.desc.cell_bytes();
        let (scale, offset, dtype) = (hd.desc.scale, hd.desc.offset, hd.desc.dtype);
        for &k in &changed {
            let k = k as usize;
            let raw = (f64::from(values[k]) - offset) / scale;
            put_raw(dtype, raw, &mut hd.bytes[k * cb..(k + 1) * cb]);
        }
    }
    for (li, values) in [(wet_idx, &cells.wetland), (class_idx, &cells.class)] {
        if let Some(li) = li {
            let hd = &mut held[li];
            for &k in &changed {
                hd.bytes[k as usize] = values[k as usize];
            }
        }
    }
    // The ice cover lies on the ground: where the world's sea stands the game has water, the ice
    // pass leaves a column whose top is fluid alone, and the class is the sea's (BIO-171), so the
    // cover goes too and a covered node stays of the class EF (BIO-158, BIO-159). ledger: BIO-177
    // A covered node the land call settled to land took its class from a land donor, which may
    // be bare ground beside the ice: it is of the class EF as every covered node is.
    let (mut cover_cleared, mut cover_ef) = (0usize, 0usize);
    if let Some(ci) = cover_idx.filter(|&ci| held[ci].desc.grid == wg) {
        let hd = &mut held[ci];
        let cb = hd.desc.cell_bytes();
        let (scale, offset, dtype) = (hd.desc.scale, hd.desc.offset, hd.desc.dtype);
        let mut covered = vec![false; n];
        for ((cell, &at_sea), cov) in hd
            .bytes
            .chunks_exact_mut(cb)
            .zip(&sea)
            .zip(covered.iter_mut())
        {
            if hd.desc.physical(dtype.decode_scalar(cell)) > 0.0 {
                if at_sea {
                    put_raw(dtype, (0.0 - offset) / scale, cell);
                    cover_cleared += 1;
                } else {
                    *cov = true;
                }
            }
        }
        if let Some(li) = class_idx {
            for (class, &cov) in held[li].bytes.iter_mut().zip(&covered) {
                if cov && *class != crate::climate::koppen::EF {
                    *class = crate::climate::koppen::EF;
                    cover_ef += 1;
                }
            }
        }
    }
    let cell_km = f64::from(wg.r) * ground.world_m_per_block_h / 1000.0;
    let km2 = |cells: usize| (cells as f64 * cell_km * cell_km * 10.0).round() / 10.0;
    json!({
        "grid": wg.name,
        "rule": "water::sea_covers_height on the world elevation and closed_basin at each node's block",
        "nodes": n,
        "land_nodes": land_nodes,
        "land_km2": km2(land_nodes),
        "land_biome_under_sea": {"nodes": settled.to_sea.len(), "km2": km2(settled.to_sea.len())},
        "ocean_biome_on_dry_ground": {"nodes": settled.to_land.len(), "km2": km2(settled.to_land.len())},
        "ice_cover_off_the_sea": {"nodes": cover_cleared, "km2": km2(cover_cleared)},
        "ice_cover_class_ef": {"nodes": cover_ef, "km2": km2(cover_ef)},
        "playa_dried": {
            "playa_nodes": playa_nodes,
            "reach_blocks": reach_blocks,
            "nodes": dried.len(),
            "km2": km2(dried.len()),
        },
        "delta_plain": {
            "plain_nodes": plain_nodes,
            "to_land": plain_settled.to_land.len(),
            "to_land_km2": km2(plain_settled.to_land.len()),
        },
        "bake_blocks_per_world_block": ground.bake_blocks_per_world_block,
    })
}

/// The terrain height the game draws at block (x, z): the elevation plus the detail term
/// (`ColumnWorld::height`), from loaded layers; `c` the x period the detail noise wraps with.
pub(crate) struct Drawn<'a> {
    pub(crate) elev: &'a LoadedLayer,
    pub(crate) detail: DrawnDetail<'a>,
    pub(crate) c: f64,
    /// A cube world's drawing: the block (⌊x⌋, ⌊z⌋) read as the game reads a cube grid, at the
    /// block's centre, folded through the net ([`RasterRead`] on a cube grid).
    pub(crate) cube: bool,
}

/// A drawing's detail term: the amplitude layer, the spec and the noise.
pub(crate) type DrawnDetail<'a> = Option<(
    &'a LoadedLayer,
    deeptime_format::DetailSpec,
    deeptime_format::Noise,
)>;

/// The detail term of a drawing from its amplitude layer and its noise (either may be absent).
pub(crate) fn drawn_detail(
    amp: Option<&LoadedLayer>,
    n: Option<(deeptime_format::DetailSpec, deeptime_format::Noise)>,
) -> DrawnDetail<'_> {
    match (amp, n) {
        (Some(a), Some((s, n))) => Some((a, s, n)),
        _ => None,
    }
}

impl Drawn<'_> {
    pub(crate) fn height(&self, x: f64, z: f64) -> f64 {
        if self.cube {
            let (bx, bz) = (x.floor(), z.floor());
            let e = self.elev.bilinear(bx, bz);
            return match &self.detail {
                Some((amp, spec, noise)) => e + amp.bilinear(bx, bz) * spec.noise(noise, bx, bz),
                None => e,
            };
        }
        let (x, z) = (x.round(), z.round());
        let wx = x - (x / self.c + 0.5).floor() * self.c;
        let e = self.elev.bilinear(wx, z);
        match &self.detail {
            Some((amp, spec, noise)) => e + amp.bilinear(wx, z) * spec.noise(noise, wx, z),
            None => e,
        }
    }
}

pub(crate) fn write_json(dir: &Path, rel: &str, v: &Value) -> Result<(u64, u32), CoreError> {
    let mut bytes =
        serde_json::to_vec_pretty(v).map_err(|e| CoreError::InvalidState(format!("{rel}: {e}")))?;
    bytes.push(b'\n');
    let crc = deeptime_format::fsutil::write_atomic(&dir.join(rel), &bytes)?;
    Ok((bytes.len() as u64, crc))
}

/// A structural event in the world: the window's centre through the projection, its axes and
/// sizes through the local Jacobian; a fault's slip and plane turned and stretched with them. An
/// intrusion's window is the exact image of the ellipse it holds (ledger: SCL-236), so a stock or
/// a plug drawn round on the planet (SCL-235) is round in the conformal cube world.
pub fn project_structure(
    map: &impl WorldMap,
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
        if matches!(e.shape, Shape::Intrusion { .. }) {
            // The ellipsoid's plan. Stretching each axis by its own image (the rule below) keeps
            // them perpendicular where the Lambert grid's shear turns their images off it, which
            // misdraws a yawed ellipse (a 1.6 : 1 stock yawed 35° at 55°, 16 % in radius).
            let (first, second, yaw) = conjugate_axes(
                (a.0 * w.half_along, a.1 * w.half_along),
                (c.0 * w.half_across, c.1 * w.half_across),
            );
            if first.is_finite() && second.is_finite() && first > 0.0 && second > 0.0 {
                let d = (yaw.cos(), yaw.sin());
                // Across on the side the old across axis went, as the rule below keeps it.
                let s = if c.1 * d.0 - c.0 * d.1 < 0.0 {
                    -1.0
                } else {
                    1.0
                };
                w.centre = [wx, wz];
                w.along = [d.0, d.1];
                w.across = [-s * d.1, s * d.0];
                w.half_along = first;
                w.half_across = second;
                continue;
            }
        }
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

/// Of (site id, deposit type) pairs, the sites of an in-stratum type (`hosts.in_stratum`, the
/// built-in types): their bodies lie in a host horizon and move through the projection only with
/// it ([`body_y`]).
pub(crate) fn stratabound_sites<'a>(
    sites: impl IntoIterator<Item = (deeptime_format::sites::SiteId, &'a str)>,
) -> Result<std::collections::HashSet<deeptime_format::sites::SiteId>, CoreError> {
    let types = crate::deposits::types::builtin()?;
    Ok(sites
        .into_iter()
        .filter(|&(_, ty)| types.iter().any(|t| t.id == ty && t.hosts.in_stratum))
        .map(|(id, _)| id)
        .collect())
}

/// What [`body_y`] did to a body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BodyY {
    /// Moved by this many blocks with the drawn ground.
    Moved(f64),
    /// A stratabound body, moved by `dy` with its host horizon (0 where the world's raise left
    /// the horizon where it was) and, with `clipped`, clipped under the world's ground.
    InHost { dy: f64, clipped: bool },
    /// Left as it was.
    Kept,
}

/// Moves a body's centre, clip and box up by `dy` blocks.
fn lift_body(nb: &mut deeptime_format::Body, dy: f64) {
    nb.center[1] += dy;
    if nb.clip_top_y != deeptime_format::bodies::NO_CLIP {
        nb.clip_top_y += dy.round() as i32;
    }
    nb.aabb = nb.computed_aabb();
}

/// A projected body's y. Most move with the drawn ground: by `dy`, the change of the drawn
/// height between the body's old column and its new one, with their clip. A stratabound one
/// (`host`: its site's type is in-stratum) moves only with its host horizon, by `host` blocks
/// (the world's top raise there, [`world_raise_shifts`]; the bake's own raise already moved it,
/// STR-056), and is clipped [`crate::deposits::SOIL_BLOCKS`] under `ground`, the world's drawn
/// ground over its centre: the young landscape and the amplifier ran after the deposit stage
/// placed it, and the drawing differs from the simulation's, so the ground over a horizon can lie
/// lower than the one the deposit stage kept the body under (ORE-271).
pub(crate) fn body_y(
    nb: &mut deeptime_format::Body,
    dy: f64,
    host: Option<f64>,
    ground: f64,
) -> BodyY {
    if let Some(host) = host {
        let host = if host.is_finite() { host } else { 0.0 };
        if host != 0.0 {
            lift_body(nb, host);
        }
        if !ground.is_finite() {
            return BodyY::InHost {
                dy: host,
                clipped: false,
            };
        }
        // ledger: ORE-271
        let clip = ground.floor() as i32 - crate::deposits::SOIL_BLOCKS;
        let clipped = clip < nb.aabb[1][1];
        if clipped {
            nb.clip_top_y = nb.clip_top_y.min(clip);
            nb.aabb = nb.computed_aabb();
        }
        return BodyY::InHost { dy: host, clipped };
    }
    if !dy.is_finite() {
        return BodyY::Kept;
    }
    lift_body(nb, dy);
    BodyY::Moved(dy)
}

/// A body in the world: its centre through the projection, its horizontal axes and yaw through
/// the local Jacobian, its box recomputed.
pub fn project_body(map: &impl WorldMap, b: &deeptime_format::Body) -> deeptime_format::Body {
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
    // a zero semi-axis and the body is a few blocks wide wherever it lands. ledger: SCL-091
    let stretch = |v: f32, s: f64| {
        let n = f64::from(v) * s;
        if n.is_finite() && n > 0.0 {
            n as f32
        } else {
            v
        }
    };
    out.axes[0] = stretch(b.axes[0], ua.0.hypot(ua.1));
    out.axes[2] = stretch(b.axes[2], uc.0.hypot(uc.1));
    out.aabb = out.computed_aabb();
    out
}

/// A site in the world: its centre through the projection, up by `dy` blocks (a stratabound
/// site's host horizon's move in the world's top raise, [`world_raise_shifts`]; 0 for the
/// others); its box the union of its bodies' (already projected) or, without bodies, its old box
/// moved with the centre.
pub fn project_site(
    map: &impl WorldMap,
    s: &deeptime_format::Site,
    bodies: &[deeptime_format::Body],
    dy: i32,
) -> deeptime_format::Site {
    let mut out = s.clone();
    // The centre block's centre through the projection, and the world block holding it (the
    // corner, rounded, could land a block off the column the body's centre is in).
    let (wx, wz) = map.to_world(f64::from(s.center[0]) + 0.5, f64::from(s.center[2]) + 0.5);
    let (cx, cz) = (wx.floor() as i32, wz.floor() as i32);
    let mut lo = [i32::MAX; 3];
    let mut hi = [i32::MIN; 3];
    let mut any = false;
    for b in bodies.iter().filter(|b| b.site == Some(s.id)) {
        // Boxes near the slide may sit a lap away from the centre: bring them next to it.
        let period = map.period() as i32;
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
        lo = [olo[0] + dx, olo[1] + dy, olo[2] + dz];
        hi = [ohi[0] + dx, ohi[1] + dy, ohi[2] + dz];
    }
    out.center = [cx, s.center[1] + dy, cz];
    out.aabb = [lo, hi];
    out
}

/// No spawn candidate within this many blocks of the cube world's open edges (the game's
/// `SpawnHandler.EDGE_MARGIN`, POL-024).
pub(crate) const SPAWN_EDGE_MARGIN: f64 = 64.0; // ledger: POL-024

/// The pass's log line (`projection …`).
pub fn summary_line(v: &Value) -> String {
    if v["kind"] == "cube" {
        return crate::bake::project_cube::summary_line(v);
    }
    format!("projection {}", v["kind"].as_str().unwrap_or("?"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::column::CLOSED_BASIN;

    /// MVT's bodies move through the projection only with their host horizon (by the world
    /// raise of the stacks, not the drawn ground); a porphyry's move with the drawn ground.
    #[test]
    fn only_in_stratum_sites_move_with_their_host() {
        let got = stratabound_sites([(1, "deeptime:mvt_zn_pb"), (2, "deeptime:porphyry_cu_mo_au")])
            .unwrap();
        assert_eq!(got.into_iter().collect::<Vec<_>>(), vec![1]);
    }

    fn lens(y: f64) -> deeptime_format::Body {
        deeptime_format::Body::new(
            1,
            Some(7),
            deeptime_format::bodies::Shape::Lens,
            "ore",
            [100.5, y, 200.5],
            [12.0, 3.0, 12.0],
            0.0,
            0.0,
            deeptime_format::bodies::NO_CLIP,
        )
    }

    /// A body moves with the drawn ground; a stratabound one moves only with its host (by the
    /// world's top raise there) and is clipped four blocks under the world's ground where that
    /// lies lower than its top.
    #[test]
    fn a_stratabound_body_moves_with_its_host_and_is_clipped_under_the_ground() {
        let mut b = lens(40.0);
        assert_eq!(body_y(&mut b, 5.0, None, 80.0), BodyY::Moved(5.0));
        assert_eq!(b.center[1], 45.0);
        // Kept: the ground (80) is well over its top (43), and its host did not move.
        let mut b = lens(40.0);
        let kept = BodyY::InHost {
            dy: 0.0,
            clipped: false,
        };
        assert_eq!(body_y(&mut b, 5.0, Some(0.0), 80.0), kept);
        assert_eq!(
            (b.center[1], b.clip_top_y),
            (40.0, deeptime_format::bodies::NO_CLIP)
        );
        // The ground at 44.7: nothing of it above block 40 (44, less the four of soil).
        let clipped = BodyY::InHost {
            dy: 0.0,
            clipped: true,
        };
        assert_eq!(body_y(&mut b, -30.0, Some(0.0), 44.7), clipped);
        assert_eq!((b.center[1], b.clip_top_y, b.aabb[1][1]), (40.0, 40, 40));
        // Never above a clip it had.
        let mut b = lens(40.0);
        b.clip_top_y = 38;
        b.aabb = b.computed_aabb();
        assert_eq!(body_y(&mut b, 0.0, Some(0.0), 44.7), kept);
        assert_eq!(b.clip_top_y, 38);
        // Its host rose 6 blocks: so does it, whatever the drawn ground did, clip and all.
        let mut b = lens(40.0);
        b.clip_top_y = 41;
        b.aabb = b.computed_aabb();
        let raised = BodyY::InHost {
            dy: 6.0,
            clipped: false,
        };
        assert_eq!(body_y(&mut b, -30.0, Some(6.0), 80.0), raised);
        assert_eq!((b.center[1], b.clip_top_y), (46.0, 47));
    }

    /// The column rule's undeformed column of `hz` at block (x, z).
    fn column(hz: &Horizons, x: i64, z: i64) -> Vec<deeptime_format::strata::ColumnHorizon> {
        let desc = hz.desc(deeptime_format::Codec::Raw);
        let (k, w) = (hz.k as usize, hz.grid.w as usize);
        let mut out = Vec::new();
        deeptime_format::strata::stack_at(
            &desc,
            hz.min_y,
            |i, j, h| {
                let n = (j as usize * w + i as usize) * k + h as usize;
                deeptime_format::strata::StoredHorizon {
                    top: hz.tops[n],
                    class: hz.classes[n],
                    key: hz.keys[n],
                }
            },
            x,
            z,
            &mut out,
        );
        out
    }

    /// The class of a drawn column at block y (`None` over its first top).
    fn class_at(col: &[deeptime_format::strata::ColumnHorizon], y: i32) -> Option<u8> {
        if col.is_empty() || y > col[0].0 {
            return None;
        }
        let mut h = 0;
        while h + 1 < col.len() && col[h + 1].0 >= y {
            h += 1;
        }
        Some(col[h].1)
    }

    /// An MVT body placed in a thin limestone stays in it through both of the steps that raise
    /// the rock after the deposit stage: the bake's lift and crest raise (`bake::strata`, the
    /// bodies risen by `key_shift`, STR-056) and the projection's raise of the world stacks to
    /// the drawn ground (`raise_world_tops`, the body moved by `world_raise_shifts` in `body_y`;
    /// ORE-271). The stacks are a soil, a 6 to 10 block limestone, a 2 to 5 block sandstone, a
    /// granite thick enough to take up both raises and gabbro, so each raise lifts the limestone
    /// whole. Keeping the body's y through the projection (the exemption before) leaves some of
    /// them out of it.
    #[test]
    fn a_stratabound_body_stays_in_its_host_through_the_bake_and_the_projection() {
        use crate::bake::strata::{Lift, raise_stack_unit, shifts};
        let class = |n: &str| crate::surface::lithology::index(n) as u8;
        let t = crate::structure::columns::Tables::get();
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let n = grid.cells();
        let (k, min_y, max_y) = (5usize, -64, 2000);
        let classes = [
            class("soil"),
            class("limestone"),
            class("sandstone"),
            class("granite"),
            class("gabbro"),
        ];
        let limestone = classes[1];
        let keys = [
            [0u16, 10],
            [5000, 5004],
            [5005, 5010],
            [32_768 + 64, 32_768 + 127],
            [33_100, 33_163],
        ];
        let mut rng = crate::sphere::SplitMix64::new(23);
        let mut tops = vec![i16::MIN; n * k];
        for c in 0..n {
            let s = 64 * 16 + rng.below(32 * 16) as i32;
            // Granite begins under 16 blocks of rock, so it takes up both raises.
            let lime = 6 * 16 + rng.below(4 * 16) as i32;
            let sand = 2 * 16 + rng.below(3 * 16) as i32;
            let granite = 60 * 16 + rng.below(40 * 16) as i32;
            let t0 = [
                s,
                s - 32,
                s - 32 - lime,
                s - 32 - lime - sand,
                s - 32 - lime - sand - granite,
            ];
            for (h, &v) in t0.iter().enumerate() {
                tops[c * k + h] = v as i16;
            }
        }
        let before = Horizons {
            grid: grid.clone(),
            k: k as u32,
            min_y,
            tops,
            classes: classes.repeat(n),
            keys: keys.repeat(n),
            tongues: (64, 0.15),
            min_blocks: Vec::new(),
        };
        // The deposit stage's bodies: mid-limestone, clear of its contacts.
        let mut bodies = Vec::new();
        while bodies.len() < 300 {
            let (x, z) = (rng.below(1024) as i64 - 512, rng.below(400) as i64 - 200);
            let col = column(&before, x, z);
            let Some(h) = col.iter().position(|c| c.1 == limestone) else {
                continue;
            };
            let bottom = col.get(h + 1).map_or(min_y, |c| c.0 + 1);
            if col[h].0 - bottom < 6 {
                continue;
            }
            let y = (col[h].0 + bottom) / 2;
            bodies.push(deeptime_format::Body::new(
                bodies.len() as u32,
                Some(bodies.len() as u32),
                deeptime_format::bodies::Shape::Lens,
                "ore",
                [x as f64 + 0.5, f64::from(y) + 0.5, z as f64 + 0.5],
                [6.0, 1.0, 6.0],
                0.0,
                0.0,
                deeptime_format::bodies::NO_CLIP,
            ));
        }
        // Step 1, the bake: a lift of 0 to 40 blocks on two cells in three and a crest raise of
        // up to 30 on every other cell; the bodies rise with the rock at their centres.
        let mut after = before.clone();
        let mut moved = vec![Moved::default(); n];
        for (c, m) in moved.iter_mut().enumerate() {
            let tops = &mut after.tops[c * k..(c + 1) * k];
            let lift = if rng.below(3) > 0 {
                rng.below(40 * 16) as i32
            } else {
                0
            };
            for t in tops.iter_mut() {
                *t = (i32::from(*t) + lift) as i16;
            }
            let need = i32::from(tops[0])
                + if rng.below(2) == 0 {
                    rng.below(30 * 16) as i32
                } else {
                    0
                };
            let (raise, upto) =
                raise_stack_unit(tops, &classes, &keys, need, min_y * 16, &t.unconsolidated);
            *m = Moved {
                lift,
                raise,
                upto: upto as u16,
            };
        }
        let (bodies_dy, _) = shifts(&after, &moved, &bodies, &[]);
        let lift = Lift {
            grid: grid.clone(),
            blocks: moved.iter().map(|m| m.lift as f32 / 16.0).collect(),
            bodies_dy,
            sites_dy: Vec::new(),
        };
        let risen = lift.bodies(&bodies);
        let host = |hz: &Horizons, b: &deeptime_format::Body| {
            let col = column(hz, b.center[0].floor() as i64, b.center[2].floor() as i64);
            class_at(&col, b.center[1].floor() as i32)
        };
        for b in &risen {
            assert_eq!(
                host(&after, b),
                Some(limestone),
                "body {} after the bake",
                b.id
            );
        }
        // Step 2, the projection (the world's stacks the bake's, the same grid): the drawn
        // ground over half the cells stands up to 24 blocks over the stack's top.
        let desc = after.desc(deeptime_format::Codec::Raw);
        let mut bytes = after.to_layer(deeptime_format::Codec::Raw).unwrap().bytes;
        let ground: Vec<u8> = (0..n)
            .flat_map(|c| {
                let top = f32::from(after.tops[c * k]) / 16.0;
                let over = if rng.below(2) == 0 {
                    rng.below(24) as f32
                } else {
                    -40.0
                };
                (top + over).to_le_bytes()
            })
            .collect();
        let elev = LoadedLayer::new(
            crate::bake::desc("terrain/elevation", DType::F32, &grid, 1.0, "blocks", None),
            ground,
        )
        .unwrap();
        let (raised, _, world_moved) =
            raise_world_tops(&desc, &mut bytes, &elev, None, min_y, max_y, true);
        assert!(raised > 0 && !world_moved.is_empty());
        let world = horizons_of(&desc, &bytes, min_y);
        let points: Vec<[f64; 3]> = risen.iter().map(|b| b.center).collect();
        let dy = world_raise_shifts(&desc, &bytes, min_y, &world_moved, &points);
        let (mut rose, mut kept_leaves) = (0, 0);
        for (b, &d) in risen.iter().zip(&dy) {
            let mut nb = b.clone();
            let got = body_y(&mut nb, f64::NAN, Some(d), f64::INFINITY);
            assert!(matches!(got, BodyY::InHost { clipped: false, .. }));
            assert_eq!(
                host(&world, &nb),
                Some(limestone),
                "body {} in the world",
                b.id
            );
            rose += usize::from(nb.center[1] > b.center[1]);
            kept_leaves += usize::from(host(&world, b) != Some(limestone));
        }
        assert!(
            rose > risen.len() / 4,
            "only {rose} bodies rose in the world"
        );
        assert!(
            kept_leaves > 0,
            "the test needs bodies the world's raise takes their host away from"
        );
    }

    /// The biome layers' land call is the column evaluator's rule on the world's ground
    /// (BIO-170, BAK-057, BAK-059, HYD-107): a node of the 35 m band under sea level, or on closed
    /// ground below it,
    /// is land and takes a land record; open ground under sea level is sea and takes the sea's;
    /// every node the rule agrees with is left as it was.
    #[test]
    fn the_biome_layers_settle_to_the_worlds_sea_rule() {
        use crate::climate::koppen;
        let grid = GridDesc::for_planet("L2", 1024, 8).unwrap();
        let (w, h) = (grid.w as usize, grid.h as usize);
        let n = w * h;
        let sea_level_y = 64;
        // Six bands of columns: dry land (the only land donor), the band H in (sea - 1, sea]
        // that the biome bake called ocean, open ground under the sea that it called land, closed
        // ground below sea level that it called ocean, deep sea, and shallow sea (both agree).
        let band = |k: usize| (k % w) * 6 / w;
        let elevation_of = |b: usize| [100.0f32, 63.5, 40.0, 40.0, -300.0, 40.0][b];
        let closed_of = |b: usize| u8::from(b == 3);
        let continents_of = |b: usize| [0.3f32, -0.25, 0.1, -0.3, -0.5, -0.3][b];
        let layer = |name: &str, dtype: DType, scale: f64, units: &str| {
            crate::bake::desc(name, dtype, &grid, scale, units, None)
        };
        let f32_bytes = |f: &dyn Fn(usize) -> f32| -> Vec<u8> {
            (0..n).flat_map(|k| f(k).to_le_bytes()).collect()
        };
        let elevation = LoadedLayer::new(
            layer("elevation", DType::F32, 1.0, "block"),
            f32_bytes(&|k| elevation_of(band(k))),
        )
        .unwrap();
        let closed = LoadedLayer::new(
            layer(CLOSED_BASIN, DType::U8, 1.0, "flag"),
            (0..n).map(|k| closed_of(band(k))).collect(),
        )
        .unwrap();
        // A sea node's record differs from a land node's in every layer.
        let i16_layer = |name: &str, land: f32, sea: f32| {
            let d = layer(name, DType::I16, 1e-4, "vanilla_param");
            let bytes = (0..n)
                .flat_map(|k| {
                    let v = if name == "climate/continents" {
                        continents_of(band(k))
                    } else if continents_of(band(k)) < -0.19 {
                        sea
                    } else {
                        land
                    };
                    ((f64::from(v) / 1e-4).round() as i16).to_le_bytes()
                })
                .collect();
            Held {
                entry: 0,
                desc: d,
                bytes,
            }
        };
        let u8_layer = |name: &str, land: u8, sea: u8| {
            let bytes = (0..n)
                .map(|k| {
                    if continents_of(band(k)) < -0.19 {
                        sea
                    } else {
                        land
                    }
                })
                .collect();
            Held {
                entry: 0,
                desc: layer(name, DType::U8, 1.0, ""),
                bytes,
            }
        };
        let mut held = vec![
            i16_layer("climate/temperature", 0.1, 0.3),
            i16_layer("climate/vegetation", -0.2, 0.0),
            i16_layer("climate/continents", 0.0, 0.0),
            i16_layer("climate/erosion", 0.25, 0.5),
            i16_layer("climate/ridges", 0.4, 0.0),
            u8_layer("climate/wetland", 1, 0),
            u8_layer("climate/koppen_l2", 5, koppen::SEA),
            // Six blocks of ice cover on the three mismatched bands, whatever their medium; the
            // dry land beside them, the land donor, is bare.
            Held {
                entry: 0,
                desc: layer(ICE_COVER, DType::U8, 1.0, "blocks"),
                bytes: (0..n)
                    .map(|k| if matches!(band(k), 1..=3) { 6 } else { 0 })
                    .collect(),
            },
        ];
        let before: Vec<Vec<u8>> = held.iter().map(|x| x.bytes.clone()).collect();
        let ground = BiomeGround {
            sea_level_y,
            m_per_block_v: 1.0,
            world_m_per_block_h: 8.0,
            bake_blocks_per_world_block: 1.0,
            elevation: &elevation,
            closed: Some(&closed),
            playa: None,
            delta: None,
            cube: None,
        };
        let note = settle_biome_layers(&mut held, &ground, &biomes::BiomeParams::default());
        let in_band = |b: usize| (0..n).filter(|&k| band(k) == b).count() as u64;
        // The record counts the nodes of the three mismatched bands, and nothing else.
        assert_eq!(
            note["land_biome_under_sea"]["nodes"].as_u64(),
            Some(in_band(2)),
            "{note}"
        );
        assert_eq!(
            note["ocean_biome_on_dry_ground"]["nodes"].as_u64(),
            Some(in_band(1) + in_band(3)),
            "{note}"
        );
        // The rule each node ends on is the format's own, on the world's elevation and flag.
        let continents = LoadedLayer::new(held[2].desc.clone(), held[2].bytes.clone()).unwrap();
        // (Rows 0 and h − 1 are glued onto themselves, which mixes this x-banded pattern: left out.)
        for k in w..n - w {
            let b = band(k);
            let sea = deeptime_format::water::sea_covers_height(
                f64::from(elevation_of(b)),
                sea_level_y,
                || closed_of(b) == 1,
            );
            let v = continents.raw_cell((k % w) as u32, (k / w) as u32) * 1e-4;
            assert_eq!(biomes::is_land_continentalness(v), !sea, "band {b}: {v}");
        }
        // Only the mismatched bands changed (the two edge rows are glued onto themselves, which
        // moves the cells of this x-banded pattern, so rows 1 to h − 2 are compared).
        for (li, (layer, was)) in held.iter().zip(&before).enumerate() {
            let cb = layer.desc.cell_bytes();
            for j in 1..h - 1 {
                for i in 0..w {
                    let k = j * w + i;
                    let (a, b) = (
                        &layer.bytes[k * cb..(k + 1) * cb],
                        &was[k * cb..(k + 1) * cb],
                    );
                    if matches!(band(k), 0 | 4 | 5) {
                        assert_eq!(a, b, "layer {li} band {} moved", band(k));
                    }
                }
            }
        }
        // The nodes that took the sea's record have the sea's class and no wetland; the nodes
        // that took the land's have the land donor's.
        let at = |name: &str, k: usize| -> f64 {
            let li = held.iter().position(|x| x.desc.name == name).unwrap();
            let l = &held[li];
            let cb = l.desc.cell_bytes();
            l.desc
                .physical(l.desc.dtype.decode_scalar(&l.bytes[k * cb..(k + 1) * cb]))
        };
        let k_of = |b: usize| (h / 2) * w + (0..w).find(|&i| band(i) == b).unwrap() + w / 24;
        assert_eq!(at("climate/koppen_l2", k_of(2)) as u8, koppen::SEA);
        assert_eq!(at("climate/wetland", k_of(2)) as u8, biomes::WETLAND_NONE);
        // The ice cover leaves the nodes the world's sea covers, and only those, and a covered
        // node that took a bare land donor's record is of the class EF (BIO-177): every node
        // still covered is EF.
        assert_eq!(
            note["ice_cover_off_the_sea"]["nodes"].as_u64(),
            Some(in_band(2)),
            "{note}"
        );
        assert_eq!(
            note["ice_cover_class_ef"]["nodes"].as_u64(),
            Some(in_band(1) + in_band(3)),
            "{note}"
        );
        assert_eq!(at(ICE_COVER, k_of(2)), 0.0);
        for b in [1, 3] {
            assert_eq!(at(ICE_COVER, k_of(b)), 6.0, "band {b}");
        }
        for k in w..n - w {
            if at(ICE_COVER, k) > 0.0 {
                assert_eq!(at("climate/koppen_l2", k) as u8, koppen::EF, "node {k}");
            }
        }
        assert_eq!(at("climate/koppen_l2", k_of(0)) as u8, 5);
        for b in [1, 3] {
            assert_eq!(
                at("climate/koppen_l2", k_of(b)) as u8,
                koppen::EF,
                "band {b}"
            );
            assert_eq!(at("climate/wetland", k_of(b)) as u8, 1, "band {b}");
            assert!(
                (at("climate/temperature", k_of(b)) - 0.1).abs() < 2e-4,
                "band {b}"
            );
        }
    }

    /// No swamp beside a world playa after the land call (BIO-178): on dry ground (no node
    /// settled), every node within the reach of the playa's node loses its wetland flag and has a
    /// swamp-band erosion held under E5, a node below the swamp band keeps its erosion, and every
    /// node beyond the reach keeps its record.
    #[test]
    fn a_world_playa_dries_the_swamp_beside_it() {
        let grid = GridDesc::for_planet("L2", 1024, 8).unwrap();
        let fine = GridDesc::for_planet("L3", 1024, 2).unwrap();
        let (w, h) = (grid.w as usize, grid.h as usize);
        let n = w * h;
        let layer = |g: &GridDesc, name: &str, dtype: DType, scale: f64, units: &str| {
            crate::bake::desc(name, dtype, g, scale, units, None)
        };
        let elevation = LoadedLayer::new(
            layer(&grid, "elevation", DType::F32, 1.0, "block"),
            (0..n).flat_map(|_| 100f32.to_le_bytes()).collect(),
        )
        .unwrap();
        // One cell of salt, the fine cell at the block of node (i0, j0).
        let (i0, j0) = (w / 2, h / 2);
        let (px, pz) = (grid.block_x(i0 as i64), grid.block_z(j0 as i64));
        let fw = fine.w as usize;
        let playa = LoadedLayer::new(
            layer(&fine, "playa", DType::U8, 1.0, "blocks"),
            (0..fine.cells())
                .map(|c| {
                    let (x, z) = (fine.block_x((c % fw) as i64), fine.block_z((c / fw) as i64));
                    if (x, z) == (px, pz) { 2 } else { 0 }
                })
                .collect(),
        )
        .unwrap();
        assert_eq!(playa.bytes.iter().filter(|&&b| b > 0).count(), 1);
        let quantum = |v: f64| ((v / 1e-4).round() as i16).to_le_bytes();
        let i16_layer = |name: &str, f: &dyn Fn(usize) -> f64| Held {
            entry: 0,
            desc: layer(&grid, name, DType::I16, 1e-4, "vanilla_param"),
            bytes: (0..n).flat_map(|k| quantum(f(k))).collect(),
        };
        // Even columns in the swamp band (E6), odd ones below it; even rows flagged swamp.
        let erosion_of = |k: usize| if (k % w).is_multiple_of(2) { 0.6 } else { 0.2 };
        let wet_of = |k: usize| u8::from((k / w).is_multiple_of(2));
        let mut held = vec![
            i16_layer("climate/temperature", &|_| 0.1),
            i16_layer("climate/vegetation", &|_| -0.2),
            i16_layer("climate/continents", &|_| 0.3),
            i16_layer("climate/erosion", &erosion_of),
            i16_layer("climate/ridges", &|_| 0.4),
            Held {
                entry: 0,
                desc: layer(&grid, "climate/wetland", DType::U8, 1.0, ""),
                bytes: (0..n).map(wet_of).collect(),
            },
        ];
        let before: Vec<Vec<u8>> = held.iter().map(|x| x.bytes.clone()).collect();
        let ground = BiomeGround {
            sea_level_y: 64,
            m_per_block_v: 1.0,
            world_m_per_block_h: 8.0,
            bake_blocks_per_world_block: 1.0,
            elevation: &elevation,
            closed: None,
            playa: Some(&playa),
            delta: None,
            cube: None,
        };
        let note = settle_biome_layers(&mut held, &ground, &biomes::BiomeParams::default());
        assert_eq!(note["ocean_biome_on_dry_ground"]["nodes"], 0, "{note}");
        assert_eq!(note["playa_dried"]["playa_nodes"], 1, "{note}");
        // The margin, two cells and a fine cell: 38 blocks, 4.75 cells.
        let reach = note["playa_dried"]["reach_blocks"].as_f64().unwrap();
        assert_eq!(reach, biomes::PLAYA_WETLAND_MARGIN_BLOCKS + 16.0 + 2.0);
        let held_e = ((biomes::reference().bands.erosion[5] - 1e-3) / 1e-4).round() as i16;
        let raw16 = |b: &[u8], k: usize| i16::from_le_bytes([b[2 * k], b[2 * k + 1]]);
        let mut expect = 0;
        // (Rows 0 and h − 1 are glued onto themselves: left out.)
        for k in w..n - w {
            let (di, dj) = ((k % w) as f64 - i0 as f64, (k / w) as f64 - j0 as f64);
            let near = di.hypot(dj) <= reach / 8.0;
            let (e0, f0) = (erosion_of(k), wet_of(k));
            let changes = near && (f0 == 1 || e0 > 0.55);
            let (e, f) = (raw16(&held[3].bytes, k), held[5].bytes[k]);
            let e_was = raw16(&before[3], k);
            if changes {
                expect += 1;
                assert_eq!(f, 0, "node {k}");
                assert_eq!(e, e_was.min(held_e), "node {k}");
            } else {
                assert_eq!((f, e), (f0, e_was), "node {k}");
            }
            // The other four inputs are as they were.
            for li in [0, 1, 2, 4] {
                assert_eq!(
                    raw16(&held[li].bytes, k),
                    raw16(&before[li], k),
                    "layer {li}"
                );
            }
        }
        assert!(expect > 20, "{expect}");
        assert_eq!(note["playa_dried"]["nodes"], expect, "{note}");
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
            "climate/koppen_l2",
            "meta/crust_type",
            "meta/province",
            "surface/class_top",
            "strata/lift",
        ] {
            assert_eq!(kind_of(&desc(name, "")), Kind::Categorical, "{name}");
        }
        assert_eq!(kind_of(&desc("some/mask", "flag")), Kind::Categorical);
        assert_eq!(kind_of(&desc("elevation", "block")), Kind::Continuous);
        // The ice cover is read by the cover rule, not averaged (BIO-159).
        assert_eq!(kind_of(&desc("climate/ice_cover", "blocks")), Kind::Cover);
        // The climate's band coordinates are read at the point, as the Köppen class beside them.
        assert_eq!(
            kind_of(&desc("climate/vegetation", "vanilla_param")),
            Kind::Point
        );
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! Tiled raster layers (plan §5.3): cell types, grid and layer descriptors, in-memory rasters and
//! the sampling rules every reader shares (wrap in x, clamp in z, bilinear in block space).

use serde::{Deserialize, Serialize};

use crate::error::{FormatError, Result};

/// The tile edge in cells (plan §5.3).
pub const TILE: u32 = 256;

/// A cell type. Scalars are little-endian; `horizon_stack` is the strata record of §5.7.4.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DType {
    U8,
    I8,
    U16,
    I16,
    U32,
    F32,
    /// `stack_depth` = K horizons per cell: K × i16 top elevation (1/16 block), then K × u8
    /// class index; 3K bytes. Unused horizons have top `i16::MIN` and class 255.
    HorizonStack,
}

impl DType {
    /// Bytes per cell of a scalar type; `None` for `horizon_stack` (it depends on K).
    pub fn scalar_bytes(self) -> Option<usize> {
        Some(match self {
            DType::U8 | DType::I8 => 1,
            DType::U16 | DType::I16 => 2,
            DType::U32 | DType::F32 => 4,
            DType::HorizonStack => return None,
        })
    }

    /// Decodes one little-endian scalar cell to f64 (the raw value, before scale and offset).
    pub fn decode_scalar(self, b: &[u8]) -> f64 {
        match self {
            DType::U8 => f64::from(b[0]),
            DType::I8 => f64::from(b[0] as i8),
            DType::U16 => f64::from(u16::from_le_bytes([b[0], b[1]])),
            DType::I16 => f64::from(i16::from_le_bytes([b[0], b[1]])),
            DType::U32 => f64::from(u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
            DType::F32 => f64::from(f32::from_le_bytes([b[0], b[1], b[2], b[3]])),
            DType::HorizonStack => f64::NAN,
        }
    }
}

/// How each tile is stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Codec {
    /// The tile's cells, little-endian, row-major.
    Raw,
    /// One LZ4 raw block (`lz4_flex::block`, no frame, no size prefix) per tile.
    Lz4Block,
}

/// An equirectangular raster in block space (plan §5.2). Cell (i, j) is the **point sample** at
/// block (origin_block_x + i·r, origin_block_z + j·r): cell 0 sits on the seam x = −C/2 and on the
/// north edge z = −C/4. Indices wrap in x (mod W) and clamp in z to [0, H−1].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridDesc {
    /// The grid's role: `L1`, `L2`, `L3` (plan §5.2).
    pub name: String,
    /// Cells east–west (W = C / r).
    pub w: u32,
    /// Cells north–south (H = W / 2).
    pub h: u32,
    /// Blocks per cell.
    pub r: u32,
    /// Always true in v0: x wraps with period W·r = C.
    pub wraps_x: bool,
    /// Block x of cell column 0 (−C/2).
    pub origin_block_x: i64,
    /// Block z of cell row 0 (−C/4).
    pub origin_block_z: i64,
}

impl GridDesc {
    /// The grid of a planet with circumference `c` blocks at `r` blocks per cell: W = C/r,
    /// H = W/2, origin (−C/2, −C/4).
    pub fn for_planet(name: &str, c: u32, r: u32) -> Result<GridDesc> {
        if r == 0 || !c.is_multiple_of(2 * r) {
            return Err(FormatError::Invalid(format!(
                "circumference {c} is not a multiple of 2·r = {}",
                2 * r
            )));
        }
        let w = c / r;
        Ok(GridDesc {
            name: name.to_owned(),
            w,
            h: w / 2,
            r,
            wraps_x: true,
            origin_block_x: -i64::from(c / 2),
            origin_block_z: -i64::from(c / 4),
        })
    }

    /// The circumference C = W·r in blocks.
    pub fn circumference(&self) -> u64 {
        u64::from(self.w) * u64::from(self.r)
    }

    /// Block x of cell column `i`.
    pub fn block_x(&self, i: i64) -> i64 {
        self.origin_block_x + i * i64::from(self.r)
    }

    /// Block z of cell row `j`.
    pub fn block_z(&self, j: i64) -> i64 {
        self.origin_block_z + j * i64::from(self.r)
    }

    /// Wraps a column index into [0, W).
    pub fn wrap_i(&self, i: i64) -> u32 {
        i.rem_euclid(i64::from(self.w)) as u32
    }

    /// Clamps a row index into [0, H − 1].
    pub fn clamp_j(&self, j: i64) -> u32 {
        j.clamp(0, i64::from(self.h) - 1) as u32
    }

    /// Cells in the raster.
    pub fn cells(&self) -> usize {
        self.w as usize * self.h as usize
    }
}

/// A layer's descriptor, stored as JSON in its tile pack (plan §5.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerDesc {
    /// `elevation`, `climate/temperature`, `strata/horizons`, …
    pub name: String,
    pub dtype: DType,
    /// physical = raw · scale + offset.
    pub scale: f64,
    pub offset: f64,
    /// `block/16`, `vanilla_param*10000`, …
    pub units: String,
    pub grid: GridDesc,
    /// Tile edge in cells (256).
    pub tile: u32,
    pub codec: Codec,
    /// The **raw** cell value meaning "no data" (e.g. −32768 for `water_level`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nodata: Option<f64>,
    /// K, for `horizon_stack` layers only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_depth: Option<u32>,
}

impl LayerDesc {
    /// A scalar layer with the default tile size.
    pub fn scalar(name: &str, dtype: DType, grid: GridDesc, codec: Codec) -> LayerDesc {
        LayerDesc {
            name: name.to_owned(),
            dtype,
            scale: 1.0,
            offset: 0.0,
            units: String::new(),
            grid,
            tile: TILE,
            codec,
            nodata: None,
            stack_depth: None,
        }
    }

    /// Bytes per cell.
    pub fn cell_bytes(&self) -> usize {
        match self.dtype.scalar_bytes() {
            Some(b) => b,
            None => 3 * self.stack_depth.unwrap_or(0) as usize,
        }
    }

    /// Tiles east–west.
    pub fn tiles_x(&self) -> u32 {
        self.grid.w.div_ceil(self.tile)
    }

    /// Tiles north–south.
    pub fn tiles_z(&self) -> u32 {
        self.grid.h.div_ceil(self.tile)
    }

    /// Tiles in the pack.
    pub fn tile_count(&self) -> usize {
        self.tiles_x() as usize * self.tiles_z() as usize
    }

    /// Tile `t` (row-major: t = tz · tiles_x + tx) as (first column, first row, width, height).
    /// Edge tiles are cropped to the raster.
    pub fn tile_rect(&self, t: usize) -> (u32, u32, u32, u32) {
        let tx = (t % self.tiles_x() as usize) as u32;
        let tz = (t / self.tiles_x() as usize) as u32;
        let x0 = tx * self.tile;
        let z0 = tz * self.tile;
        (
            x0,
            z0,
            self.tile.min(self.grid.w - x0),
            self.tile.min(self.grid.h - z0),
        )
    }

    /// Checks the descriptor is internally consistent.
    pub fn validate(&self) -> Result<()> {
        let bad = |m: String| Err(FormatError::Invalid(format!("layer {}: {m}", self.name)));
        if self.name.is_empty() {
            return bad("empty name".into());
        }
        if self.grid.w == 0 || self.grid.h == 0 || self.grid.r == 0 {
            return bad(format!("empty grid {:?}", self.grid));
        }
        if self.tile == 0 {
            return bad("tile edge 0".into());
        }
        match (self.dtype, self.stack_depth) {
            (DType::HorizonStack, Some(k)) if (1..=255).contains(&k) => {}
            (DType::HorizonStack, k) => {
                return bad(format!(
                    "horizon_stack needs stack_depth 1..=255, got {k:?}"
                ));
            }
            (_, Some(_)) => return bad("stack_depth on a scalar layer".into()),
            _ => {}
        }
        if !self.scale.is_finite() || !self.offset.is_finite() {
            return bad("non-finite scale or offset".into());
        }
        Ok(())
    }

    /// Raw value → physical value; NaN for nodata.
    pub fn physical(&self, raw: f64) -> f64 {
        if self.nodata == Some(raw) || raw.is_nan() {
            f64::NAN
        } else {
            raw * self.scale + self.offset
        }
    }
}

/// A scalar cell type that can be written to a tile pack.
pub trait Cell: Copy + Send + Sync + 'static {
    const DTYPE: DType;
    /// Appends the little-endian bytes of `self`.
    fn put_le(self, out: &mut Vec<u8>);
}

macro_rules! cell {
    ($t:ty, $d:ident) => {
        impl Cell for $t {
            const DTYPE: DType = DType::$d;
            fn put_le(self, out: &mut Vec<u8>) {
                out.extend_from_slice(&self.to_le_bytes());
            }
        }
    };
}
cell!(u8, U8);
cell!(i8, I8);
cell!(u16, U16);
cell!(i16, I16);
cell!(u32, U32);
cell!(f32, F32);

/// A whole raster in memory, row-major (`data[j · W + i]`).
#[derive(Clone, Debug, PartialEq)]
pub struct Raster<T> {
    pub grid: GridDesc,
    pub data: Vec<T>,
}

impl<T: Cell> Raster<T> {
    /// Fills every cell from `f(i, j)`.
    pub fn from_fn(grid: GridDesc, mut f: impl FnMut(u32, u32) -> T) -> Raster<T> {
        let mut data = Vec::with_capacity(grid.cells());
        for j in 0..grid.h {
            for i in 0..grid.w {
                data.push(f(i, j));
            }
        }
        Raster { grid, data }
    }

    /// The cell at (i, j) with x wrapped and z clamped.
    pub fn get(&self, i: i64, j: i64) -> T {
        let i = self.grid.wrap_i(i) as usize;
        let j = self.grid.clamp_j(j) as usize;
        self.data[j * self.grid.w as usize + i]
    }

    /// The raster's cells as little-endian bytes, row-major.
    pub fn to_le_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.data.len() * T::DTYPE.scalar_bytes().unwrap_or(1));
        for &v in &self.data {
            v.put_le(&mut out);
        }
        out
    }
}

/// Sampling shared by every reader. The Java `RasterSampler` implements the same arithmetic in
/// the same order, so results agree bit for bit (the parity test checks it).
pub trait RasterRead {
    fn desc(&self) -> &LayerDesc;

    /// The raw value of an in-range cell (0 ≤ i < W, 0 ≤ j < H).
    fn raw_cell(&self, i: u32, j: u32) -> f64;

    /// The raw value at (i, j), x wrapped, z clamped.
    fn raw(&self, i: i64, j: i64) -> f64 {
        let g = &self.desc().grid;
        self.raw_cell(g.wrap_i(i), g.clamp_j(j))
    }

    /// The physical value at cell (i, j), x wrapped, z clamped; NaN for nodata.
    fn get(&self, i: i64, j: i64) -> f64 {
        self.desc().physical(self.raw(i, j))
    }

    /// The physical value of the cell nearest block (bx, bz).
    fn nearest(&self, bx: f64, bz: f64) -> f64 {
        let g = &self.desc().grid;
        let u = (bx - g.origin_block_x as f64) / f64::from(g.r);
        let v = (bz - g.origin_block_z as f64) / f64::from(g.r);
        self.get((u + 0.5).floor() as i64, (v + 0.5).floor() as i64)
    }

    /// Bilinear interpolation of the physical values at block (bx, bz). NaN if any of the four
    /// cells is nodata.
    fn bilinear(&self, bx: f64, bz: f64) -> f64 {
        let g = &self.desc().grid;
        let u = (bx - g.origin_block_x as f64) / f64::from(g.r);
        let v = (bz - g.origin_block_z as f64) / f64::from(g.r);
        let fi = u.floor();
        let fj = v.floor();
        let fu = u - fi;
        let fv = v - fj;
        let (i0, j0) = (fi as i64, fj as i64);
        let v00 = self.get(i0, j0);
        let v10 = self.get(i0 + 1, j0);
        let v01 = self.get(i0, j0 + 1);
        let v11 = self.get(i0 + 1, j0 + 1);
        let a = v00 + (v10 - v00) * fu;
        let b = v01 + (v11 - v01) * fu;
        a + (b - a) * fv
    }
}

/// A whole layer's cells in memory (from [`crate::TilePackReader::load`] or built directly).
#[derive(Clone, Debug)]
pub struct LoadedLayer {
    pub desc: LayerDesc,
    /// Row-major cells, `desc.cell_bytes()` each.
    pub bytes: Vec<u8>,
}

impl LoadedLayer {
    /// Wraps row-major cell bytes; fails if the length does not match the descriptor.
    pub fn new(desc: LayerDesc, bytes: Vec<u8>) -> Result<LoadedLayer> {
        let want = desc.grid.cells() * desc.cell_bytes();
        if bytes.len() != want {
            return Err(FormatError::Invalid(format!(
                "layer {}: {} bytes of cells, expected {want}",
                desc.name,
                bytes.len()
            )));
        }
        Ok(LoadedLayer { desc, bytes })
    }

    /// Horizon `k` of the stack at in-range cell (i, j): (top in 1/16 block, class index).
    pub fn horizon(&self, i: u32, j: u32, k: u32) -> (i16, u8) {
        let depth = self.desc.stack_depth.unwrap_or(0) as usize;
        let cell = (j as usize * self.desc.grid.w as usize + i as usize) * 3 * depth;
        let t = cell + 2 * k as usize;
        let top = i16::from_le_bytes([self.bytes[t], self.bytes[t + 1]]);
        let class = self.bytes[cell + 2 * depth + k as usize];
        (top, class)
    }

    /// The horizon stack at block (x, z) exactly as the Java strata pass builds a column
    /// ([`stack_at`]).
    pub fn stack_at(&self, x: i64, z: i64, out: &mut Vec<(i32, u8)>) {
        stack_at(&self.desc, |i, j, k| self.horizon(i, j, k), x, z, out);
    }
}

/// The horizon stack at block (x, z) of a `horizon_stack` layer described by `d`, whose in-range
/// cells `horizon(i, j, k)` reads, exactly as the Java strata pass builds a column
/// (`StrataPass.column`, `worlddata/Columns`, plan §7.5): classes from the nearest cell, each top
/// bilinear between the four surrounding cells (the nearest cell's where a corner lacks the
/// horizon), floored to a block and never above the horizon over it; the walk stops at the first
/// unused horizon of the nearest cell. `out` gets (top block y, class index), top-down; horizon h
/// fills from its top down to the next horizon's top + 1, the last down to the world's floor.
pub fn stack_at(
    d: &LayerDesc,
    horizon: impl Fn(u32, u32, u32) -> (i16, u8),
    x: i64,
    z: i64,
    out: &mut Vec<(i32, u8)>,
) {
    out.clear();
    let g = &d.grid;
    let k = d.stack_depth.unwrap_or(0);
    let c = i64::from(g.w) * i64::from(g.r);
    let wx = (x + c / 2).rem_euclid(c) - c / 2;
    let u = (wx - g.origin_block_x) as f64 / f64::from(g.r);
    let v = (z - g.origin_block_z) as f64 / f64::from(g.r);
    let (i_n, j_n) = ((u + 0.5).floor() as i64, (v + 0.5).floor() as i64);
    let (i0, j0) = (u.floor() as i64, v.floor() as i64);
    let (fu, fv) = (u - i0 as f64, v - j0 as f64);
    let top_at = |i: i64, j: i64, h: u32| -> f64 {
        let (t, _) = horizon(g.wrap_i(i), g.clamp_j(j), h);
        if t == i16::MIN {
            f64::NAN
        } else {
            f64::from(t) * d.scale + d.offset
        }
    };
    let mut prev = i32::MAX;
    for h in 0..k {
        let (_, class) = horizon(g.wrap_i(i_n), g.clamp_j(j_n), h);
        let nearest = top_at(i_n, j_n, h);
        if class == 255 || nearest.is_nan() {
            break;
        }
        let (t00, t10) = (top_at(i0, j0, h), top_at(i0 + 1, j0, h));
        let (t01, t11) = (top_at(i0, j0 + 1, h), top_at(i0 + 1, j0 + 1, h));
        let top = if [t00, t10, t01, t11].iter().any(|t| t.is_nan()) {
            nearest
        } else {
            (t00 + (t10 - t00) * fu) * (1.0 - fv) + (t01 + (t11 - t01) * fu) * fv
        };
        let t = (top.floor() as i32).min(prev);
        out.push((t, class));
        prev = t;
    }
}

impl RasterRead for LoadedLayer {
    fn desc(&self) -> &LayerDesc {
        &self.desc
    }

    fn raw_cell(&self, i: u32, j: u32) -> f64 {
        let cb = self.desc.cell_bytes();
        let at = (j as usize * self.desc.grid.w as usize + i as usize) * cb;
        self.desc.dtype.decode_scalar(&self.bytes[at..at + cb])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(c: u32, r: u32) -> LoadedLayer {
        let grid = GridDesc::for_planet("L", c, r).unwrap();
        let ras = Raster::<i16>::from_fn(grid.clone(), |i, j| (i as i16) * 10 + j as i16);
        let mut d = LayerDesc::scalar("t", DType::I16, grid, Codec::Raw);
        d.scale = 0.5;
        d.nodata = Some(-32768.0);
        LoadedLayer::new(d, ras.to_le_bytes()).unwrap()
    }

    #[test]
    fn stack_at_follows_the_java_column_rule() {
        // Two horizons on a 2 × 1 grid (r 32, C 64): tops 10 and 0 blocks at cell 0, 20 and 5 at
        // cell 1; classes 3, 4 and 7, 8.
        let grid = GridDesc::for_planet("L1", 64, 32).unwrap();
        let mut d = LayerDesc::scalar("strata/horizons", DType::HorizonStack, grid, Codec::Raw);
        d.scale = 1.0 / 16.0;
        d.stack_depth = Some(2);
        let mut bytes = Vec::new();
        for (tops, classes) in [([160i16, 0], [3u8, 4]), ([320, 80], [7, 8])] {
            for t in tops {
                bytes.extend_from_slice(&t.to_le_bytes());
            }
            bytes.extend_from_slice(&classes);
        }
        let l = LoadedLayer::new(d, bytes).unwrap();
        let mut out = Vec::new();
        // Cell 0 sits at block x −32, cell 1 at 0; x −24 is a quarter of the way: tops 12.5 → 12
        // and 1.25 → 1, classes of the nearest (cell 0).
        l.stack_at(-24, -16, &mut out);
        assert_eq!(out, [(12, 3), (1, 4)]);
        // x −8: three quarters, nearest cell 1.
        l.stack_at(-8, -16, &mut out);
        assert_eq!(out, [(17, 7), (3, 8)]);
        // x wraps: 40 ≡ −24.
        l.stack_at(40, -16, &mut out);
        assert_eq!(out, [(12, 3), (1, 4)]);
    }

    #[test]
    fn grid_geometry() {
        let g = GridDesc::for_planet("L3", 16384, 2).unwrap();
        assert_eq!((g.w, g.h), (8192, 4096));
        assert_eq!((g.origin_block_x, g.origin_block_z), (-8192, -4096));
        assert_eq!(g.circumference(), 16384);
        assert!(GridDesc::for_planet("x", 1000, 16).is_err());
    }

    #[test]
    fn wrap_and_clamp() {
        let l = layer(64, 4); // 16 × 8
        assert_eq!(l.raw(-1, 0), l.raw(15, 0));
        assert_eq!(l.raw(16, 3), l.raw(0, 3));
        assert_eq!(l.raw(2, -5), l.raw(2, 0));
        assert_eq!(l.raw(2, 99), l.raw(2, 7));
        assert_eq!(l.get(3, 2), 0.5 * 32.0);
    }

    #[test]
    fn bilinear_hits_samples_and_midpoints() {
        let l = layer(64, 4);
        let g = &l.desc.grid;
        for (i, j) in [(0, 0), (5, 3), (15, 7)] {
            let bx = g.block_x(i) as f64;
            let bz = g.block_z(j) as f64;
            assert_eq!(l.bilinear(bx, bz), l.get(i, j));
            assert_eq!(l.nearest(bx + 1.0, bz - 1.0), l.get(i, j));
        }
        // Midway between columns 3 and 4 of row 2.
        let mid = l.bilinear(g.block_x(3) as f64 + 2.0, g.block_z(2) as f64);
        assert_eq!(mid, 0.5 * (l.get(3, 2) + l.get(4, 2)));
        // Across the seam: between the last column and column 0.
        let seam = l.bilinear(g.block_x(15) as f64 + 2.0, g.block_z(1) as f64);
        assert_eq!(seam, 0.5 * (l.get(15, 1) + l.get(0, 1)));
    }

    #[test]
    fn nodata_is_nan() {
        let grid = GridDesc::for_planet("L", 64, 4).unwrap();
        let ras = Raster::<i16>::from_fn(grid.clone(), |i, _| if i == 3 { i16::MIN } else { 7 });
        let mut d = LayerDesc::scalar("w", DType::I16, grid, Codec::Raw);
        d.nodata = Some(f64::from(i16::MIN));
        let l = LoadedLayer::new(d, ras.to_le_bytes()).unwrap();
        assert!(l.get(3, 0).is_nan());
        assert_eq!(l.get(2, 0), 7.0);
        assert!(
            l.bilinear(l.desc.grid.block_x(2) as f64 + 1.0, 0.0)
                .is_nan()
        );
    }

    #[test]
    fn desc_json_round_trips() {
        let d = layer(64, 4).desc;
        let s = serde_json::to_string(&d).unwrap();
        assert!(s.contains("\"dtype\":\"i16\""), "{s}");
        let back: LayerDesc = serde_json::from_str(&s).unwrap();
        assert_eq!(back, d);
    }

    #[test]
    fn tile_rects_crop_edges() {
        let grid = GridDesc::for_planet("L1", 16384 * 2, 64).unwrap(); // 512 × 256
        let mut d = LayerDesc::scalar("x", DType::U8, grid, Codec::Raw);
        d.tile = 200;
        assert_eq!((d.tiles_x(), d.tiles_z()), (3, 2));
        assert_eq!(d.tile_rect(0), (0, 0, 200, 200));
        assert_eq!(d.tile_rect(2), (400, 0, 112, 200));
        assert_eq!(d.tile_rect(5), (400, 200, 112, 56));
    }
}

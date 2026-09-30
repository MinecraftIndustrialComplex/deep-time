// SPDX-License-Identifier: LGPL-3.0-or-later
//! Tiled raster layers (plan §5.3): cell types, grid and layer descriptors, in-memory rasters and
//! the sampling rules every reader shares (wrap in x, clamp in z, bilinear in block space).

use serde::{Deserialize, Serialize};

use crate::error::{FormatError, Result};
use crate::strata::{ColumnHorizon, StoredHorizon};

/// Horizon `k` of one `horizon_stack` cell's bytes (K = `depth`, 7K bytes).
pub fn decode_horizon(cell: &[u8], depth: usize, k: usize) -> StoredHorizon {
    let t = 2 * k;
    let kt = 3 * depth + 2 * k;
    let kb = 5 * depth + 2 * k;
    StoredHorizon {
        top: i16::from_le_bytes([cell[t], cell[t + 1]]),
        class: cell[2 * depth + k],
        key: [
            u16::from_le_bytes([cell[kt], cell[kt + 1]]),
            u16::from_le_bytes([cell[kb], cell[kb + 1]]),
        ],
    }
}

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
    /// `stack_depth` = K horizons per cell: K × i16 top elevation (1/16 block), K × u8 class
    /// index, K × u16 stratigraphic key top, K × u16 key base; 7K bytes (format 0.2). Unused
    /// horizons have top `i16::MIN`, class 255 and keys `u16::MAX` (`crate::strata`).
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
            None => 7 * self.stack_depth.unwrap_or(0) as usize,
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

    /// Horizon `k` of the stack at in-range cell (i, j).
    pub fn horizon(&self, i: u32, j: u32, k: u32) -> StoredHorizon {
        let depth = self.desc.stack_depth.unwrap_or(0) as usize;
        let cell = (j as usize * self.desc.grid.w as usize + i as usize) * 7 * depth;
        decode_horizon(&self.bytes[cell..cell + 7 * depth], depth, k as usize)
    }

    /// The horizon stack at block (x, z) of a world whose floor is `min_y`, exactly as the Java
    /// strata pass builds a column ([`crate::strata::stack_at`]).
    pub fn stack_at(&self, x: i64, z: i64, min_y: i32, out: &mut Vec<ColumnHorizon>) {
        crate::strata::stack_at(
            &self.desc,
            min_y,
            |i, j, k| self.horizon(i, j, k),
            x,
            z,
            out,
        );
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

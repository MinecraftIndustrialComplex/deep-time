// SPDX-License-Identifier: LGPL-3.0-or-later
//! A tile pack read on demand (the Rust twin of Java's `RasterSampler` over a `TilePack`): tiles
//! decode on first touch into a bounded cache, so sampling a few columns of a 32k planet's L3 never
//! loads the whole layer. The sampling arithmetic is [`RasterRead`]'s, the column rule
//! [`crate::raster::stack_at`]'s, so a [`LazyLayer`] answers exactly what a
//! [`crate::LoadedLayer`] of the same file answers.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::error::Result;
use crate::raster::{LayerDesc, RasterRead};
use crate::tilepack::TilePackReader;

/// Decoded bytes kept per layer before the cache starts over.
pub const CACHE_BYTES: usize = 96 << 20;

/// A memory-mapped tile pack whose tiles decode lazily.
pub struct LazyLayer {
    reader: TilePackReader,
    cache: Mutex<Tiles>,
}

#[derive(Default)]
struct Tiles {
    map: HashMap<usize, Arc<Vec<u8>>>,
    bytes: usize,
}

impl LazyLayer {
    /// Maps `path` and checks its header and index (tiles are checked as they decode).
    pub fn open(path: &Path) -> Result<LazyLayer> {
        Ok(LazyLayer {
            reader: TilePackReader::open(path)?,
            cache: Mutex::new(Tiles::default()),
        })
    }

    pub fn reader(&self) -> &TilePackReader {
        &self.reader
    }

    /// Tile `t`, decoded. A tile that fails its CRC or does not decode panics: the pack's header
    /// and index were checked at open, so this is a file changed or damaged underneath us.
    fn tile(&self, t: usize) -> Arc<Vec<u8>> {
        let mut c = self.cache.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(hit) = c.map.get(&t) {
            return Arc::clone(hit);
        }
        let cells = Arc::new(
            self.reader
                .read_tile(t)
                .unwrap_or_else(|e| panic!("world data changed underneath the reader: {e}")),
        );
        if c.bytes + cells.len() > CACHE_BYTES {
            c.map.clear();
            c.bytes = 0;
        }
        c.bytes += cells.len();
        c.map.insert(t, Arc::clone(&cells));
        cells
    }

    /// The bytes of in-range cell (i, j).
    fn cell_bytes(&self, i: u32, j: u32) -> (Arc<Vec<u8>>, usize) {
        let d = self.reader.desc();
        let (tx, tz) = (i / d.tile, j / d.tile);
        let t = (tz * d.tiles_x() + tx) as usize;
        let (_, _, tw, _) = d.tile_rect(t);
        let (li, lj) = ((i % d.tile) as usize, (j % d.tile) as usize);
        let cb = d.cell_bytes();
        (self.tile(t), (lj * tw as usize + li) * cb)
    }

    /// Horizon `k` of the stack at in-range cell (i, j): (top in 1/16 block, class index).
    pub fn horizon(&self, i: u32, j: u32, k: u32) -> (i16, u8) {
        let depth = self.reader.desc().stack_depth.unwrap_or(0) as usize;
        let (tile, at) = self.cell_bytes(i, j);
        let t = at + 2 * k as usize;
        (
            i16::from_le_bytes([tile[t], tile[t + 1]]),
            tile[at + 2 * depth + k as usize],
        )
    }

    /// The horizon stack at block (x, z) as the Java strata pass builds a column (see
    /// [`crate::raster::stack_at`]).
    pub fn stack_at(&self, x: i64, z: i64, out: &mut Vec<(i32, u8)>) {
        crate::raster::stack_at(self.desc(), |i, j, k| self.horizon(i, j, k), x, z, out);
    }
}

impl RasterRead for LazyLayer {
    fn desc(&self) -> &LayerDesc {
        self.reader.desc()
    }

    fn raw_cell(&self, i: u32, j: u32) -> f64 {
        let (tile, at) = self.cell_bytes(i, j);
        let cb = self.reader.desc().cell_bytes();
        self.reader.desc().dtype.decode_scalar(&tile[at..at + cb])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raster::{Codec, DType, GridDesc, LoadedLayer, Raster};
    use crate::tilepack::write_tile_pack;

    #[test]
    fn a_lazy_layer_answers_what_the_loaded_one_does() {
        let dir = std::env::temp_dir().join(format!("dt-lazy-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // 600 × 300 cells: tiles of 256 with cropped edges.
        let grid = GridDesc::for_planet("L", 1200, 2).unwrap();
        let ras = Raster::<i16>::from_fn(grid.clone(), |i, j| {
            (i as i16).wrapping_mul(7) ^ (j as i16).wrapping_mul(13)
        });
        let mut d = LayerDesc::scalar("t", DType::I16, grid.clone(), Codec::Lz4Block);
        d.scale = 0.25;
        let path = dir.join("t.dtr");
        write_tile_pack(&path, &d, &ras.to_le_bytes()).unwrap();
        let lazy = LazyLayer::open(&path).unwrap();
        let loaded = LoadedLayer::new(d, ras.to_le_bytes()).unwrap();
        for (bx, bz) in [(-600.0, -300.0), (0.5, 3.25), (599.9, 299.0), (-1.3, 17.0)] {
            assert_eq!(lazy.bilinear(bx, bz), loaded.bilinear(bx, bz));
            assert_eq!(lazy.nearest(bx, bz), loaded.nearest(bx, bz));
        }
        for (i, j) in [(0, 0), (255, 255), (256, 0), (599, 299), (300, 150)] {
            assert_eq!(lazy.raw_cell(i, j), loaded.raw_cell(i, j));
        }

        // A horizon stack, K = 3, on the same grid.
        let k = 3usize;
        let mut bytes = Vec::new();
        for j in 0..grid.h {
            for i in 0..grid.w {
                for h in 0..k as i16 {
                    let top = 800 - 160 * h + (i % 17) as i16 + (j % 5) as i16;
                    bytes.extend_from_slice(&top.to_le_bytes());
                }
                bytes.extend_from_slice(&[(i % 7) as u8, 20, 255]);
            }
        }
        let mut hd = LayerDesc::scalar("strata/horizons", DType::HorizonStack, grid, Codec::Raw);
        hd.scale = 1.0 / 16.0;
        hd.stack_depth = Some(k as u32);
        let hpath = dir.join("h.dtr");
        write_tile_pack(&hpath, &hd, &bytes).unwrap();
        let lazy = LazyLayer::open(&hpath).unwrap();
        let loaded = LoadedLayer::new(hd, bytes).unwrap();
        let (mut a, mut b) = (Vec::new(), Vec::new());
        for (x, z) in [(-600, -300), (5, 7), (598, 299), (1201, -3)] {
            lazy.stack_at(x, z, &mut a);
            loaded.stack_at(x, z, &mut b);
            assert_eq!(a, b, "({x}, {z})");
            assert_eq!(a.len(), 2, "the unused third horizon stops the walk");
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

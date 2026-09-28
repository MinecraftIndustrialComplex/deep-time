// SPDX-License-Identifier: LGPL-3.0-or-later
//! Tile packs (`.dtr`, plan §5.7.3): header → JSON `LayerDesc` → tile index → tiles.
//! The byte layout is specified in `FORMAT.md`.

use std::fs::File;
use std::path::{Path, PathBuf};

use memmap2::Mmap;

use crate::error::{FormatError, Result};
use crate::fsutil::write_atomic;
use crate::raster::{Codec, LayerDesc, LoadedLayer};
use crate::{FORMAT_MINOR, FORMAT_VERSION};

/// Magic of a raster tile pack.
pub const MAGIC: [u8; 4] = *b"DTR1";
/// The fixed header's size.
pub const HEADER_BYTES: usize = 16;
/// Bytes per tile-index entry: u64 offset, u32 stored length, u32 CRC32.
pub const INDEX_ENTRY_BYTES: usize = 16;

fn align8(n: usize) -> usize {
    n.div_ceil(8) * 8
}

/// One tile's place in the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileEntry {
    /// Absolute byte offset from the start of the file.
    pub offset: u64,
    /// Stored (compressed, for LZ4) length in bytes.
    pub length: u32,
    /// CRC32 of the stored bytes.
    pub crc32: u32,
}

/// What [`write_tile_pack`] wrote.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Written {
    pub bytes: u64,
    pub crc32: u32,
}

/// Encodes a tile pack for `desc` from row-major `cells` (`desc.cell_bytes()` each).
pub fn encode_tile_pack(desc: &LayerDesc, cells: &[u8]) -> Result<Vec<u8>> {
    desc.validate()?;
    let cb = desc.cell_bytes();
    let want = desc.grid.cells() * cb;
    if cells.len() != want {
        return Err(FormatError::Invalid(format!(
            "layer {}: {} bytes of cells, expected {want}",
            desc.name,
            cells.len()
        )));
    }
    let json = serde_json::to_vec(desc).map_err(|source| FormatError::Json {
        what: format!("layer {}", desc.name),
        source,
    })?;
    let header_len = HEADER_BYTES + json.len();
    let header_len_u32 = u32::try_from(header_len)
        .map_err(|_| FormatError::Invalid("layer descriptor too large".into()))?;

    let mut out = Vec::with_capacity(align8(header_len) + cells.len() / 2);
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&header_len_u32.to_le_bytes());
    out.extend_from_slice(&FORMAT_MINOR.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // flags, reserved
    let mut h = crc32fast::Hasher::new();
    h.update(&out[..12]);
    h.update(&json);
    out.extend_from_slice(&h.finalize().to_le_bytes());
    out.extend_from_slice(&json);
    out.resize(align8(out.len()), 0);

    let n = desc.tile_count();
    let index_at = out.len();
    out.resize(align8(index_at + n * INDEX_ENTRY_BYTES), 0);

    let row_bytes = desc.grid.w as usize * cb;
    let mut tile_buf = Vec::with_capacity(desc.tile as usize * desc.tile as usize * cb);
    for t in 0..n {
        let (x0, z0, tw, th) = desc.tile_rect(t);
        tile_buf.clear();
        for z in z0..z0 + th {
            let start = z as usize * row_bytes + x0 as usize * cb;
            tile_buf.extend_from_slice(&cells[start..start + tw as usize * cb]);
        }
        let stored: Vec<u8>;
        let data: &[u8] = match desc.codec {
            Codec::Raw => &tile_buf,
            Codec::Lz4Block => {
                stored = lz4_flex::block::compress(&tile_buf);
                &stored
            }
        };
        let offset = out.len();
        let length = u32::try_from(data.len())
            .map_err(|_| FormatError::Invalid("tile larger than 4 GiB".into()))?;
        let crc = crc32fast::hash(data);
        out.extend_from_slice(data);
        out.resize(align8(out.len()), 0);
        let e = index_at + t * INDEX_ENTRY_BYTES;
        out[e..e + 8].copy_from_slice(&(offset as u64).to_le_bytes());
        out[e + 8..e + 12].copy_from_slice(&length.to_le_bytes());
        out[e + 12..e + 16].copy_from_slice(&crc.to_le_bytes());
    }
    Ok(out)
}

/// Compresses `data` into one LZ4 raw block, exactly as `lz4_block` tiles are stored.
pub fn lz4_block(data: &[u8]) -> Vec<u8> {
    lz4_flex::block::compress(data)
}

/// Writes a tile pack atomically (tmp + fsync + rename) and returns its size and CRC32.
pub fn write_tile_pack(path: &Path, desc: &LayerDesc, cells: &[u8]) -> Result<Written> {
    let bytes = encode_tile_pack(desc, cells)?;
    let crc32 = write_atomic(path, &bytes)?;
    Ok(Written {
        bytes: bytes.len() as u64,
        crc32,
    })
}

/// A memory-mapped tile pack.
pub struct TilePackReader {
    path: PathBuf,
    map: Mmap,
    desc: LayerDesc,
    index: Vec<TileEntry>,
}

impl TilePackReader {
    /// Maps `path` read-only and checks its header, descriptor and index.
    pub fn open(path: &Path) -> Result<TilePackReader> {
        let file = File::open(path).map_err(|e| FormatError::io(path, e))?;
        // SAFETY: world-data files are written once (tmp + rename) and never modified in place;
        // a file truncated underneath us is the one hazard, as for every mmap reader.
        let map = unsafe { Mmap::map(&file) }.map_err(|e| FormatError::io(path, e))?;
        let desc = parse_header(path, &map)?;
        let index = parse_index(path, &map, &desc)?;
        Ok(TilePackReader {
            path: path.to_owned(),
            map,
            desc,
            index,
        })
    }

    pub fn desc(&self) -> &LayerDesc {
        &self.desc
    }

    pub fn index(&self) -> &[TileEntry] {
        &self.index
    }

    /// Decodes tile `t`, checking its CRC32.
    pub fn read_tile(&self, t: usize) -> Result<Vec<u8>> {
        let e = self.index[t];
        let stored = &self.map[e.offset as usize..e.offset as usize + e.length as usize];
        let actual = crc32fast::hash(stored);
        if actual != e.crc32 {
            return Err(FormatError::Crc {
                path: self.path.clone(),
                what: format!("tile {t}"),
                expected: e.crc32,
                actual,
            });
        }
        let (_, _, tw, th) = self.desc.tile_rect(t);
        let size = tw as usize * th as usize * self.desc.cell_bytes();
        let cells = match self.desc.codec {
            Codec::Raw => stored.to_vec(),
            Codec::Lz4Block => lz4_flex::block::decompress(stored, size)
                .map_err(|err| FormatError::corrupt(&self.path, format!("tile {t}: LZ4: {err}")))?,
        };
        if cells.len() != size {
            return Err(FormatError::corrupt(
                &self.path,
                format!("tile {t}: {} bytes decoded, expected {size}", cells.len()),
            ));
        }
        Ok(cells)
    }

    /// Decodes every tile into one row-major raster.
    pub fn load(&self) -> Result<LoadedLayer> {
        let cb = self.desc.cell_bytes();
        let row_bytes = self.desc.grid.w as usize * cb;
        let mut all = vec![0u8; self.desc.grid.cells() * cb];
        for t in 0..self.index.len() {
            let tile = self.read_tile(t)?;
            let (x0, z0, tw, th) = self.desc.tile_rect(t);
            let tile_row = tw as usize * cb;
            for r in 0..th as usize {
                let dst = (z0 as usize + r) * row_bytes + x0 as usize * cb;
                all[dst..dst + tile_row].copy_from_slice(&tile[r * tile_row..(r + 1) * tile_row]);
            }
        }
        LoadedLayer::new(self.desc.clone(), all)
    }
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn parse_header(path: &Path, b: &[u8]) -> Result<LayerDesc> {
    if b.len() < HEADER_BYTES {
        return Err(FormatError::corrupt(
            path,
            "shorter than the 16-byte header",
        ));
    }
    if b[..4] != MAGIC {
        return Err(FormatError::corrupt(
            path,
            format!("magic {:?}, expected DTR1", &b[..4]),
        ));
    }
    let header_len = u32_at(b, 4) as usize;
    let minor = u16::from_le_bytes([b[8], b[9]]);
    if minor != FORMAT_MINOR {
        return Err(FormatError::Version {
            found: format!("0.{minor}"),
            supported: FORMAT_VERSION.to_owned(),
        });
    }
    if header_len < HEADER_BYTES || header_len > b.len() {
        return Err(FormatError::corrupt(
            path,
            format!("header length {header_len} out of range"),
        ));
    }
    let json = &b[HEADER_BYTES..header_len];
    let mut h = crc32fast::Hasher::new();
    h.update(&b[..12]);
    h.update(json);
    let actual = h.finalize();
    let expected = u32_at(b, 12);
    if actual != expected {
        return Err(FormatError::Crc {
            path: path.to_owned(),
            what: "header".into(),
            expected,
            actual,
        });
    }
    let desc: LayerDesc = serde_json::from_slice(json).map_err(|source| FormatError::Json {
        what: format!("{}: layer descriptor", path.display()),
        source,
    })?;
    desc.validate()?;
    Ok(desc)
}

fn parse_index(path: &Path, b: &[u8], desc: &LayerDesc) -> Result<Vec<TileEntry>> {
    let header_len = u32_at(b, 4) as usize;
    let at = align8(header_len);
    let n = desc.tile_count();
    if at + n * INDEX_ENTRY_BYTES > b.len() {
        return Err(FormatError::corrupt(path, "tile index past end of file"));
    }
    let mut index = Vec::with_capacity(n);
    for t in 0..n {
        let e = at + t * INDEX_ENTRY_BYTES;
        let offset = u64::from_le_bytes(b[e..e + 8].try_into().expect("8 bytes"));
        let entry = TileEntry {
            offset,
            length: u32_at(b, e + 8),
            crc32: u32_at(b, e + 12),
        };
        if offset.saturating_add(u64::from(entry.length)) > b.len() as u64 {
            return Err(FormatError::corrupt(
                path,
                format!("tile {t} past end of file"),
            ));
        }
        index.push(entry);
    }
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raster::{DType, GridDesc, Raster, RasterRead};

    fn tmp(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("dt-tilepack-{}", std::process::id()))
            .join(name)
    }

    /// splitmix64: a tiny deterministic generator, so the property tests need no crate.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
    }

    fn round_trip(desc: &LayerDesc, cells: &[u8], name: &str) {
        let p = tmp(name);
        let w = write_tile_pack(&p, desc, cells).unwrap();
        assert_eq!(w.bytes, std::fs::metadata(&p).unwrap().len());
        assert_eq!(w.crc32, crc32fast::hash(&std::fs::read(&p).unwrap()));
        let r = TilePackReader::open(&p).unwrap();
        assert_eq!(r.desc(), desc);
        let loaded = r.load().unwrap();
        assert_eq!(loaded.bytes, cells, "{name}");
        for e in r.index() {
            assert_eq!(e.offset % 8, 0, "tiles are 8-byte aligned");
        }
        std::fs::remove_file(&p).unwrap();
    }

    #[test]
    fn property_round_trip_all_dtypes_codecs_and_shapes() {
        let mut rng = Rng(42);
        let dtypes = [
            DType::U8,
            DType::I8,
            DType::U16,
            DType::I16,
            DType::U32,
            DType::F32,
            DType::HorizonStack,
        ];
        for case in 0..40 {
            let r = [1u32, 2, 4, 8][(rng.next() % 4) as usize];
            let w_cells = 2 * (1 + rng.next() % 300) as u32; // even, up to 600 (several tiles)
            let c = w_cells * r;
            let grid = GridDesc::for_planet("L", c, r).unwrap();
            let dtype = dtypes[(rng.next() % dtypes.len() as u64) as usize];
            let codec = if rng.next().is_multiple_of(2) {
                Codec::Raw
            } else {
                Codec::Lz4Block
            };
            let mut desc = LayerDesc::scalar("p", dtype, grid, codec);
            desc.tile = [256u32, 256, 64, 100][(rng.next() % 4) as usize];
            if dtype == DType::HorizonStack {
                desc.stack_depth = Some(1 + (rng.next() % 5) as u32);
            }
            let n = desc.grid.cells() * desc.cell_bytes();
            // Mix smooth runs (compressible) and noise.
            let mut cells = Vec::with_capacity(n);
            let mut v = 0u8;
            while cells.len() < n {
                if rng.next().is_multiple_of(3) {
                    v = rng.next() as u8;
                }
                cells.push(v);
            }
            round_trip(&desc, &cells, &format!("case{case}.dtr"));
        }
    }

    #[test]
    fn typed_values_survive() {
        let grid = GridDesc::for_planet("L3", 1024, 2).unwrap();
        let ras = Raster::<i16>::from_fn(grid.clone(), |i, j| {
            (i as i32 * 37 - j as i32 * 11 - 3000) as i16
        });
        let mut desc = LayerDesc::scalar("elevation", DType::I16, grid, Codec::Lz4Block);
        desc.scale = 1.0 / 16.0;
        let p = tmp("typed.dtr");
        write_tile_pack(&p, &desc, &ras.to_le_bytes()).unwrap();
        let l = TilePackReader::open(&p).unwrap().load().unwrap();
        for (i, j) in [(0, 0), (511, 255), (300, 17), (-1, 300)] {
            assert_eq!(l.raw(i, j), f64::from(ras.get(i, j)));
            assert_eq!(l.get(i, j), f64::from(ras.get(i, j)) / 16.0);
        }
        std::fs::remove_file(&p).unwrap();
    }

    #[test]
    fn detects_corruption_and_foreign_versions() {
        let grid = GridDesc::for_planet("L", 64, 1).unwrap();
        let desc = LayerDesc::scalar("c", DType::U8, grid, Codec::Lz4Block);
        let cells: Vec<u8> = (0..desc.grid.cells()).map(|i| (i % 7) as u8).collect();
        let good = encode_tile_pack(&desc, &cells).unwrap();
        let p = tmp("corrupt.dtr");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();

        // A flipped byte in the JSON: header CRC.
        let mut b = good.clone();
        b[HEADER_BYTES + 3] ^= 0x20;
        std::fs::write(&p, &b).unwrap();
        assert!(matches!(
            TilePackReader::open(&p),
            Err(FormatError::Crc { .. }) | Err(FormatError::Json { .. })
        ));

        // A flipped byte in the last tile: tile CRC.
        std::fs::write(&p, &good).unwrap();
        let e = *TilePackReader::open(&p).unwrap().index().last().unwrap();
        let mut b = good.clone();
        b[e.offset as usize + e.length as usize - 1] ^= 0xFF;
        std::fs::write(&p, &b).unwrap();
        let r = TilePackReader::open(&p).unwrap();
        assert!(matches!(r.load(), Err(FormatError::Crc { .. })));

        // Another minor version: a version error naming both.
        let mut b = good.clone();
        b[8] = 3;
        std::fs::write(&p, &b).unwrap();
        let err = TilePackReader::open(&p).err().unwrap().to_string();
        assert!(err.contains("0.3") && err.contains(FORMAT_VERSION), "{err}");

        // Bad magic.
        let mut b = good;
        b[0] = b'X';
        std::fs::write(&p, &b).unwrap();
        assert!(matches!(
            TilePackReader::open(&p),
            Err(FormatError::Corrupt { .. })
        ));
        std::fs::remove_file(&p).unwrap();
    }

    #[test]
    fn lz4_tile_is_a_plain_block() {
        // The Java side decodes with lz4-java's safe decompressor given the tile's cell count;
        // that only works if the stored bytes are a bare block with no size prefix.
        let grid = GridDesc::for_planet("L", 512, 2).unwrap();
        let desc = LayerDesc::scalar("z", DType::U8, grid, Codec::Lz4Block);
        let cells = vec![0u8; desc.grid.cells()];
        let packed = encode_tile_pack(&desc, &cells).unwrap();
        let p = tmp("block.dtr");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, &packed).unwrap();
        let r = TilePackReader::open(&p).unwrap();
        let e = r.index()[0];
        let stored = &packed[e.offset as usize..(e.offset + u64::from(e.length)) as usize];
        assert_eq!(stored, lz4_flex::block::compress(&cells).as_slice());
        std::fs::remove_file(&p).unwrap();
    }
}

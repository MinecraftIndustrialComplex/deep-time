// SPDX-License-Identifier: LGPL-3.0-or-later
//! Minecraft's region files (`region/r.<rx>.<rz>.mca`), as Java Edition 1.21.1 writes them.
//!
//! A region holds 32 × 32 chunks. The first 4 KiB are 1024 big-endian location entries
//! (3 bytes: offset in 4 KiB sectors, 1 byte: sector count), entry `(cx & 31) + (cz & 31) · 32`;
//! the next 4 KiB are timestamps. A chunk starts on its sector with a u32 length (bytes that
//! follow, compression byte included), a compression byte and the compressed NBT:
//! 1 gzip, 2 zlib (the default), 3 none, 4 LZ4 (`region-file-compression=lz4`, lz4-java's
//! `LZ4BlockOutputStream` stream). Bit 7 of the compression byte means the chunk was too large
//! and lives in `c.<cx>.<cz>.mcc` next to the region file.

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::bail;
use crate::error::{Error, Result};
use crate::nbt::{self, Tag};

pub const SECTOR: usize = 4096;

/// `r.<rx>.<rz>.mca` → (rx, rz).
pub fn parse_region_name(name: &str) -> Option<(i32, i32)> {
    let rest = name.strip_prefix("r.")?.strip_suffix(".mca")?;
    let (x, z) = rest.split_once('.')?;
    Some((x.parse().ok()?, z.parse().ok()?))
}

/// Every `r.X.Z.mca` in `region_dir`, sorted by (rx, rz).
pub fn list_regions(region_dir: &Path) -> Result<Vec<(i32, i32, PathBuf)>> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(region_dir).map_err(|e| Error::io(region_dir, e))?;
    for entry in rd {
        let entry = entry.map_err(|e| Error::io(region_dir, e))?;
        let name = entry.file_name();
        if let Some((rx, rz)) = name.to_str().and_then(parse_region_name) {
            out.push((rx, rz, entry.path()));
        }
    }
    out.sort();
    Ok(out)
}

/// One region file, read whole into memory.
pub struct RegionFile {
    pub path: PathBuf,
    pub rx: i32,
    pub rz: i32,
    data: Vec<u8>,
}

/// A chunk's stored bytes before decompression.
pub struct RawChunk {
    /// The compression byte with bit 7 (external) cleared.
    pub compression: u8,
    pub external: bool,
    pub payload: Vec<u8>,
    pub timestamp: u32,
}

impl RegionFile {
    pub fn open(path: &Path) -> Result<RegionFile> {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let Some((rx, rz)) = parse_region_name(name) else {
            bail!("{}: not a region file name (r.<x>.<z>.mca)", path.display());
        };
        let data = std::fs::read(path).map_err(|e| Error::io(path, e))?;
        if !data.is_empty() && data.len() < 2 * SECTOR {
            bail!(
                "{}: {} bytes, shorter than the 8 KiB header",
                path.display(),
                data.len()
            );
        }
        Ok(RegionFile {
            path: path.to_owned(),
            rx,
            rz,
            data,
        })
    }

    /// (sector offset, sector count) of local chunk (lx, lz), or None if it was never written.
    pub fn location(&self, lx: usize, lz: usize) -> Option<(usize, usize)> {
        if self.data.is_empty() {
            return None;
        }
        let i = 4 * ((lx & 31) + (lz & 31) * 32);
        let e = &self.data[i..i + 4];
        let offset = (usize::from(e[0]) << 16) | (usize::from(e[1]) << 8) | usize::from(e[2]);
        let count = usize::from(e[3]);
        (offset >= 2 && count > 0).then_some((offset, count))
    }

    /// The local coordinates of every chunk present, x inner.
    pub fn present(&self) -> Vec<(usize, usize)> {
        let mut v = Vec::new();
        for lz in 0..32 {
            for lx in 0..32 {
                if self.location(lx, lz).is_some() {
                    v.push((lx, lz));
                }
            }
        }
        v
    }

    /// The global chunk coordinates of local chunk (lx, lz).
    pub fn chunk_pos(&self, lx: usize, lz: usize) -> (i32, i32) {
        (self.rx * 32 + lx as i32, self.rz * 32 + lz as i32)
    }

    /// The stored bytes of local chunk (lx, lz).
    pub fn raw_chunk(&self, lx: usize, lz: usize) -> Result<Option<RawChunk>> {
        let Some((offset, count)) = self.location(lx, lz) else {
            return Ok(None);
        };
        let start = offset * SECTOR;
        let (cx, cz) = self.chunk_pos(lx, lz);
        if start + 5 > self.data.len() {
            bail!(
                "{}: chunk {cx},{cz} starts past the end of the file",
                self.path.display()
            );
        }
        let len = u32::from_be_bytes(self.data[start..start + 4].try_into().unwrap()) as usize;
        let kind = self.data[start + 4];
        let external = kind & 0x80 != 0;
        let ti = SECTOR + 4 * ((lx & 31) + (lz & 31) * 32);
        let timestamp = u32::from_be_bytes(self.data[ti..ti + 4].try_into().unwrap());
        let payload = if external {
            let mcc = self.path.with_file_name(format!("c.{cx}.{cz}.mcc"));
            std::fs::read(&mcc).map_err(|e| Error::io(&mcc, e))?
        } else {
            if len == 0 || start + 4 + len > self.data.len() || len > count * SECTOR {
                bail!(
                    "{}: chunk {cx},{cz} has length {len}, which does not fit its {count} sectors",
                    self.path.display()
                );
            }
            self.data[start + 5..start + 4 + len].to_vec()
        };
        Ok(Some(RawChunk {
            compression: kind & 0x7F,
            external,
            payload,
            timestamp,
        }))
    }

    /// The root compound of local chunk (lx, lz).
    pub fn chunk_nbt(&self, lx: usize, lz: usize) -> Result<Option<Tag>> {
        let Some(raw) = self.raw_chunk(lx, lz)? else {
            return Ok(None);
        };
        let (cx, cz) = self.chunk_pos(lx, lz);
        let bytes = decompress(raw.compression, &raw.payload)
            .map_err(|e| Error(format!("{}: chunk {cx},{cz}: {}", self.path.display(), e.0)))?;
        let (_, tag) = nbt::parse(&bytes)
            .map_err(|e| Error(format!("{}: chunk {cx},{cz}: {}", self.path.display(), e.0)))?;
        Ok(Some(tag))
    }
}

/// Decompresses a chunk payload of compression type `kind`.
pub fn decompress(kind: u8, payload: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    match kind {
        1 => {
            flate2::read::GzDecoder::new(payload)
                .read_to_end(&mut out)
                .map_err(|e| Error(format!("gzip: {e}")))?;
        }
        2 => {
            flate2::read::ZlibDecoder::new(payload)
                .read_to_end(&mut out)
                .map_err(|e| Error(format!("zlib: {e}")))?;
        }
        3 => out.extend_from_slice(payload),
        4 => out = lz4_block_stream(payload)?,
        127 => bail!("custom (namespaced) chunk compression is not supported"),
        k => bail!("unknown chunk compression type {k}"),
    }
    Ok(out)
}

/// lz4-java's `LZ4BlockOutputStream` framing: blocks of `"LZ4Block"`, a token (method 0x10 raw
/// or 0x20 LZ4, ORed with the level), then little-endian i32 compressed length, decompressed
/// length and checksum (XXHash32, not verified here), then the block. A block with decompressed
/// length 0 ends the stream.
pub fn lz4_block_stream(mut b: &[u8]) -> Result<Vec<u8>> {
    const MAGIC: &[u8] = b"LZ4Block";
    const HEADER: usize = MAGIC.len() + 1 + 4 + 4 + 4;
    let mut out = Vec::new();
    while !b.is_empty() {
        if b.len() < HEADER || &b[..MAGIC.len()] != MAGIC {
            bail!(
                "LZ4: bad block header at stream offset (remaining {} bytes)",
                b.len()
            );
        }
        let token = b[8];
        let le = |i: usize| i32::from_le_bytes(b[i..i + 4].try_into().unwrap());
        let (clen, dlen) = (le(9), le(13));
        if clen < 0 || dlen < 0 {
            bail!("LZ4: negative block length");
        }
        let (clen, dlen) = (clen as usize, dlen as usize);
        if b.len() < HEADER + clen {
            bail!("LZ4: block of {clen} bytes truncated");
        }
        let block = &b[HEADER..HEADER + clen];
        if dlen == 0 {
            break;
        }
        match token & 0xF0 {
            0x10 => {
                if clen != dlen {
                    bail!("LZ4: raw block with {clen} stored and {dlen} decoded bytes");
                }
                out.extend_from_slice(block);
            }
            0x20 => {
                let d = lz4_flex::block::decompress(block, dlen)
                    .map_err(|e| Error(format!("LZ4: {e}")))?;
                if d.len() != dlen {
                    bail!(
                        "LZ4: block decoded to {} bytes, header says {dlen}",
                        d.len()
                    );
                }
                out.extend_from_slice(&d);
            }
            m => bail!("LZ4: unknown block method 0x{m:02x}"),
        }
        b = &b[HEADER + clen..];
    }
    Ok(out)
}

/// Encodes `data` as an lz4-java block stream (for tests and fixtures). The checksum field is
/// written as 0: this reader does not check it.
pub fn encode_lz4_block_stream(data: &[u8], block_size: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let put = |out: &mut Vec<u8>, token: u8, stored: &[u8], dlen: usize| {
        out.extend_from_slice(b"LZ4Block");
        out.push(token);
        out.extend_from_slice(&(stored.len() as i32).to_le_bytes());
        out.extend_from_slice(&(dlen as i32).to_le_bytes());
        out.extend_from_slice(&0i32.to_le_bytes());
        out.extend_from_slice(stored);
    };
    for chunk in data.chunks(block_size.max(1)) {
        let c = lz4_flex::block::compress(chunk);
        if c.len() < chunk.len() {
            put(&mut out, 0x20 | 0x0A, &c, chunk.len());
        } else {
            put(&mut out, 0x10, chunk, chunk.len());
        }
    }
    put(&mut out, 0x10, &[], 0);
    out
}

/// Builds a region file from (local x, local z, compression byte, payload, timestamp) entries.
pub fn build_region(chunks: &[(usize, usize, u8, Vec<u8>, u32)]) -> Vec<u8> {
    let mut out = vec![0u8; 2 * SECTOR];
    for (lx, lz, kind, payload, ts) in chunks {
        let start = out.len();
        let len = payload.len() + 1;
        out.extend_from_slice(&(len as u32).to_be_bytes());
        out.push(*kind);
        out.extend_from_slice(payload);
        let sectors = (out.len() - start).div_ceil(SECTOR);
        out.resize(start + sectors * SECTOR, 0);
        let offset = start / SECTOR;
        let i = 4 * ((lx & 31) + (lz & 31) * 32);
        out[i] = (offset >> 16) as u8;
        out[i + 1] = (offset >> 8) as u8;
        out[i + 2] = offset as u8;
        out[i + 3] = sectors.min(255) as u8;
        out[SECTOR + i..SECTOR + i + 4].copy_from_slice(&ts.to_be_bytes());
    }
    out
}

/// Copies the chunks of `src` whose global chunk coordinates lie in `[cx0, cx1] × [cz0, cz1]`
/// into a new region file at `dst`, stored bytes unchanged (fixtures for tests).
pub fn extract(src: &Path, dst: &Path, cx0: i32, cz0: i32, cx1: i32, cz1: i32) -> Result<usize> {
    let r = RegionFile::open(src)?;
    let mut chunks = Vec::new();
    for (lx, lz) in r.present() {
        let (cx, cz) = r.chunk_pos(lx, lz);
        if cx < cx0 || cx > cx1 || cz < cz0 || cz > cz1 {
            continue;
        }
        let raw = r.raw_chunk(lx, lz)?.expect("present");
        if raw.external {
            bail!("chunk {cx},{cz} is stored externally; copy its .mcc file by hand");
        }
        chunks.push((lx, lz, raw.compression, raw.payload, raw.timestamp));
    }
    let n = chunks.len();
    std::fs::write(dst, build_region(&chunks)).map_err(|e| Error::io(dst, e))?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn doc() -> (Tag, Vec<u8>) {
        let tag = Tag::Compound(vec![
            ("DataVersion".into(), Tag::Int(3955)),
            ("Status".into(), Tag::String("minecraft:full".into())),
            (
                "pad".into(),
                Tag::LongArray((0..5000).map(|i| i * 7 % 13).collect()),
            ),
        ]);
        let bytes = nbt::write("", &tag);
        (tag, bytes)
    }

    fn compress(kind: u8, raw: &[u8]) -> Vec<u8> {
        match kind {
            1 => {
                let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
                e.write_all(raw).unwrap();
                e.finish().unwrap()
            }
            2 => {
                let mut e =
                    flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
                e.write_all(raw).unwrap();
                e.finish().unwrap()
            }
            3 => raw.to_vec(),
            4 => encode_lz4_block_stream(raw, 8192),
            _ => unreachable!(),
        }
    }

    #[test]
    fn names() {
        assert_eq!(parse_region_name("r.-16.3.mca"), Some((-16, 3)));
        assert_eq!(parse_region_name("r.0.0.mcc"), None);
        assert_eq!(parse_region_name("c.0.0.mca"), None);
    }

    #[test]
    fn every_compression_type_reads_back() {
        let (tag, raw) = doc();
        let dir = std::env::temp_dir().join(format!("dt-inspect-region-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let chunks: Vec<_> = [1u8, 2, 3, 4]
            .iter()
            .enumerate()
            .map(|(i, &k)| (i, 5, k, compress(k, &raw), 1000 + i as u32))
            .collect();
        // One more chunk stored externally (bit 7), its payload in c.<cx>.<cz>.mcc.
        let mut all = chunks.clone();
        all.push((31, 31, 0x82, Vec::new(), 7));
        let path = dir.join("r.-1.2.mca");
        std::fs::write(&path, build_region(&all)).unwrap();
        std::fs::write(dir.join("c.-1.95.mcc"), compress(2, &raw)).unwrap();

        let r = RegionFile::open(&path).unwrap();
        assert_eq!(r.present().len(), 5);
        assert_eq!(r.chunk_pos(0, 5), (-32, 69));
        for (lx, lz, ..) in &chunks {
            assert_eq!(r.chunk_nbt(*lx, *lz).unwrap().unwrap(), tag);
        }
        assert_eq!(r.raw_chunk(1, 5).unwrap().unwrap().timestamp, 1001);
        assert_eq!(r.chunk_nbt(31, 31).unwrap().unwrap(), tag);
        assert!(r.chunk_nbt(9, 9).unwrap().is_none());

        let out = dir.join("r.-1.2.copy.mca");
        // The copy's name need not parse; RegionFile::open is only used on the source.
        assert_eq!(extract(&path, &out, -32, 69, -31, 69).unwrap(), 2);
        let copied = std::fs::read(&out).unwrap();
        let back = dir.join("r.-1.2.mca");
        std::fs::write(&back, copied).unwrap();
        let r = RegionFile::open(&back).unwrap();
        assert_eq!(r.present(), vec![(0, 5), (1, 5)]);
        assert_eq!(r.chunk_nbt(1, 5).unwrap().unwrap(), tag);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn lz4_stream_with_raw_and_compressed_blocks() {
        let mut data: Vec<u8> = (0..20_000u32).map(|i| (i % 251) as u8).collect();
        // Incompressible tail: forces a raw (0x10) block.
        let mut x = 0x1234_5678u32;
        for _ in 0..3000 {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            data.push(x as u8);
        }
        let enc = encode_lz4_block_stream(&data, 4096);
        assert_eq!(lz4_block_stream(&enc).unwrap(), data);
        assert!(lz4_block_stream(&enc[..enc.len() - 30]).is_err());
    }
}

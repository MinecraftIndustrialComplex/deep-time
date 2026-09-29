// SPDX-License-Identifier: LGPL-3.0-or-later
//! A read-only zip (jar) reader: the central directory, stored and deflated entries, Zip64
//! sizes and offsets. Enough for mod jars and Minecraft's resource jar.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::bail;
use crate::error::{Error, Result};

enum Bytes {
    Mapped(memmap2::Mmap),
    Owned(Vec<u8>),
}

impl std::ops::Deref for Bytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self {
            Bytes::Mapped(m) => m,
            Bytes::Owned(v) => v,
        }
    }
}

#[derive(Clone, Copy)]
struct Entry {
    method: u16,
    compressed: u64,
    size: u64,
    local_header: u64,
}

pub struct Zip {
    pub name: String,
    bytes: Bytes,
    entries: HashMap<String, Entry>,
}

fn u16le(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([b[i], b[i + 1]])
}

fn u32le(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}

fn u64le(b: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}

impl Zip {
    /// Maps the file at `path` and reads its central directory.
    pub fn open(path: &Path) -> Result<Zip> {
        let file = std::fs::File::open(path).map_err(|e| Error::io(path, e))?;
        // SAFETY: jars are read, never written, while the tool runs.
        let map = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| Error::io(path, e))?;
        Zip::parse(path.display().to_string(), Bytes::Mapped(map))
    }

    /// Reads a zip held in memory (a jar nested in a jar).
    pub fn from_vec(name: String, v: Vec<u8>) -> Result<Zip> {
        Zip::parse(name, Bytes::Owned(v))
    }

    fn parse(name: String, bytes: Bytes) -> Result<Zip> {
        let b: &[u8] = &bytes;
        if b.len() < 22 {
            bail!("{name}: too short for a zip");
        }
        // End of central directory: the last "PK\5\6" within the last 64 KiB + 22 bytes.
        let lo = b.len().saturating_sub(22 + 65_535);
        let Some(eocd) = (lo..=b.len() - 22)
            .rev()
            .find(|&i| u32le(b, i) == 0x0605_4b50)
        else {
            bail!("{name}: no end of central directory (not a zip)");
        };
        let mut count = u64::from(u16le(b, eocd + 10));
        let mut cd_size = u64::from(u32le(b, eocd + 12));
        let mut cd_offset = u64::from(u32le(b, eocd + 16));
        if (count == 0xFFFF || cd_size == 0xFFFF_FFFF || cd_offset == 0xFFFF_FFFF)
            && eocd >= 20
            && u32le(b, eocd - 20) == 0x0706_4b50
        {
            let rec = u64le(b, eocd - 20 + 8) as usize;
            if rec + 56 > b.len() || u32le(b, rec) != 0x0606_4b50 {
                bail!("{name}: bad Zip64 end of central directory");
            }
            count = u64le(b, rec + 32);
            cd_size = u64le(b, rec + 40);
            cd_offset = u64le(b, rec + 48);
        }
        if cd_offset + cd_size > b.len() as u64 {
            bail!("{name}: central directory past the end of the file");
        }
        let mut entries = HashMap::with_capacity(count as usize);
        let mut at = cd_offset as usize;
        for _ in 0..count {
            if at + 46 > b.len() || u32le(b, at) != 0x0201_4b50 {
                bail!("{name}: bad central directory entry at {at}");
            }
            let method = u16le(b, at + 10);
            let mut compressed = u64::from(u32le(b, at + 20));
            let mut size = u64::from(u32le(b, at + 24));
            let n = usize::from(u16le(b, at + 28));
            let extra = usize::from(u16le(b, at + 30));
            let comment = usize::from(u16le(b, at + 32));
            let mut local_header = u64::from(u32le(b, at + 42));
            if at + 46 + n + extra > b.len() {
                bail!("{name}: truncated central directory");
            }
            let fname = String::from_utf8_lossy(&b[at + 46..at + 46 + n]).into_owned();
            // Zip64 extra field: the 0xFFFFFFFF fields, in order, as u64.
            let mut e = at + 46 + n;
            let end = e + extra;
            while e + 4 <= end {
                let id = u16le(b, e);
                let len = usize::from(u16le(b, e + 2));
                if id == 0x0001 {
                    let mut p = e + 4;
                    for field in [&mut size, &mut compressed, &mut local_header] {
                        if *field == 0xFFFF_FFFF && p + 8 <= e + 4 + len {
                            *field = u64le(b, p);
                            p += 8;
                        }
                    }
                }
                e += 4 + len;
            }
            entries.insert(
                fname,
                Entry {
                    method,
                    compressed,
                    size,
                    local_header,
                },
            );
            at += 46 + n + extra + comment;
        }
        Ok(Zip {
            name,
            bytes,
            entries,
        })
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    /// The uncompressed bytes of entry `name`.
    pub fn read(&self, name: &str) -> Result<Vec<u8>> {
        let Some(e) = self.entries.get(name).copied() else {
            bail!("{}: no entry {name}", self.name);
        };
        let b: &[u8] = &self.bytes;
        let lh = e.local_header as usize;
        if lh + 30 > b.len() || u32le(b, lh) != 0x0403_4b50 {
            bail!("{}: {name}: bad local header", self.name);
        }
        let start = lh + 30 + usize::from(u16le(b, lh + 26)) + usize::from(u16le(b, lh + 28));
        let end = start + e.compressed as usize;
        if end > b.len() {
            bail!("{}: {name}: data past the end of the file", self.name);
        }
        let data = &b[start..end];
        match e.method {
            0 => Ok(data.to_vec()),
            8 => {
                let mut out = Vec::with_capacity(e.size as usize);
                flate2::read::DeflateDecoder::new(data)
                    .read_to_end(&mut out)
                    .map_err(|err| Error(format!("{}: {name}: inflate: {err}", self.name)))?;
                Ok(out)
            }
            m => bail!(
                "{}: {name}: compression method {m} not supported",
                self.name
            ),
        }
    }
}

/// Writes a zip of stored (uncompressed) entries: test fixtures only.
pub fn write_stored(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in entries {
        let crc = {
            let mut h = flate2::Crc::new();
            h.update(data);
            h.sum()
        };
        let offset = out.len() as u32;
        let n = name.len() as u16;
        let size = data.len() as u32;
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&[20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(data);
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&[20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&n.to_le_bytes());
        central.extend_from_slice(&[0u8; 12]);
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name.as_bytes());
    }
    let cd_offset = out.len() as u32;
    let cd_size = central.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_round_trip() {
        let z = write_stored(&[("a/b.txt", &b"hello"[..]), ("c.json", &b"{}"[..])]);
        let zip = Zip::from_vec("t".into(), z).unwrap();
        assert_eq!(zip.read("a/b.txt").unwrap(), b"hello");
        assert_eq!(zip.read("c.json").unwrap(), b"{}");
        assert!(zip.read("missing").is_err());
        assert_eq!(zip.names().count(), 2);
    }
}

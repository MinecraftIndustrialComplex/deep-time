// SPDX-License-Identifier: LGPL-3.0-or-later
//! The binary container shared by event segments (`DTEV`), site records (`DTSR`), the site index
//! (`DTSI`), keyframes (`DTKF`) and checkpoints (`DTCP`): the 16-byte header of plan §5.7.2, a
//! fixed block of per-type fields, a section table and 8-byte-aligned sections, each with its own
//! CRC32. The byte layout is in `FORMAT.md` ("Sectioned files").
//!
//! ```text
//! 0      4     magic
//! 4      4     u32 header_len = 16 + F + 16·K
//! 8      2     u16 format_minor
//! 10     2     u16 flags, reserved, 0
//! 12     4     u32 CRC32 of bytes 0..12 followed by bytes 16..header_len
//! 16     F     fixed fields (per file type)
//! 16+F   16·K  section table: u64 offset, u32 length, u32 CRC32 of the section's bytes
//! …            sections, each starting on a multiple of 8, zero padding between
//! ```

use std::path::Path;

use crate::error::{FormatError, Result};
use crate::{FORMAT_MINOR, FORMAT_VERSION};

/// Size of the common header before the fixed fields.
pub const HEADER_BYTES: usize = 16;
/// Size of one section-table entry.
pub const SECTION_ENTRY_BYTES: usize = 16;

pub(crate) fn align8(n: usize) -> usize {
    n.div_ceil(8) * 8
}

/// Builds a sectioned file from its magic, fixed fields and sections.
pub fn encode(magic: [u8; 4], fixed: &[u8], sections: &[&[u8]]) -> Result<Vec<u8>> {
    let header_len = HEADER_BYTES + fixed.len() + SECTION_ENTRY_BYTES * sections.len();
    let header_len_u32 =
        u32::try_from(header_len).map_err(|_| FormatError::Invalid("header too large".into()))?;
    let total: usize = sections.iter().map(|s| align8(s.len())).sum();
    let mut out = Vec::with_capacity(align8(header_len) + total);
    out.extend_from_slice(&magic);
    out.extend_from_slice(&header_len_u32.to_le_bytes());
    out.extend_from_slice(&FORMAT_MINOR.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&[0; 4]); // CRC, filled below
    out.extend_from_slice(fixed);
    out.resize(header_len, 0);
    out.resize(align8(header_len), 0);
    for (k, s) in sections.iter().enumerate() {
        let offset = out.len() as u64;
        let length = u32::try_from(s.len())
            .map_err(|_| FormatError::Invalid(format!("section {k} larger than 4 GiB")))?;
        let e = HEADER_BYTES + fixed.len() + k * SECTION_ENTRY_BYTES;
        out[e..e + 8].copy_from_slice(&offset.to_le_bytes());
        out[e + 8..e + 12].copy_from_slice(&length.to_le_bytes());
        out[e + 12..e + 16].copy_from_slice(&crc32fast::hash(s).to_le_bytes());
        out.extend_from_slice(s);
        out.resize(align8(out.len()), 0);
    }
    let mut h = crc32fast::Hasher::new();
    h.update(&out[..12]);
    h.update(&out[HEADER_BYTES..header_len]);
    out[12..16].copy_from_slice(&h.finalize().to_le_bytes());
    Ok(out)
}

/// A parsed sectioned file borrowing its bytes.
pub struct Parsed<'a> {
    pub fixed: &'a [u8],
    pub sections: Vec<&'a [u8]>,
}

/// Checks magic, version, header CRC, section bounds and section CRCs. `fixed_len` is the size
/// of the type's fixed fields; `count_from_fixed` returns the type's section count K (a constant,
/// or read from the fixed fields when it varies, as for keyframes).
pub fn parse<'a>(
    path: &Path,
    b: &'a [u8],
    magic: [u8; 4],
    fixed_len: usize,
    count_from_fixed: impl FnOnce(&[u8]) -> usize,
) -> Result<Parsed<'a>> {
    if b.len() < HEADER_BYTES + fixed_len {
        return Err(FormatError::corrupt(path, "shorter than its header"));
    }
    if b[..4] != magic {
        return Err(FormatError::corrupt(
            path,
            format!(
                "magic {:?}, expected {}",
                &b[..4],
                String::from_utf8_lossy(&magic)
            ),
        ));
    }
    let minor = u16::from_le_bytes([b[8], b[9]]);
    if minor != FORMAT_MINOR {
        return Err(FormatError::Version {
            found: format!("0.{minor}"),
            supported: FORMAT_VERSION.to_owned(),
        });
    }
    let fixed = &b[HEADER_BYTES..HEADER_BYTES + fixed_len];
    let k = count_from_fixed(fixed);
    let header_len = u32_at(b, 4) as usize;
    let expected_len = HEADER_BYTES + fixed_len + k.saturating_mul(SECTION_ENTRY_BYTES);
    if header_len != expected_len || header_len > b.len() {
        return Err(FormatError::corrupt(
            path,
            format!("header length {header_len}, expected {expected_len}"),
        ));
    }
    let mut h = crc32fast::Hasher::new();
    h.update(&b[..12]);
    h.update(&b[HEADER_BYTES..header_len]);
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
    let mut sections = Vec::with_capacity(k);
    for s in 0..k {
        let e = HEADER_BYTES + fixed_len + s * SECTION_ENTRY_BYTES;
        let offset = u64_at(b, e);
        let length = u32_at(b, e + 8);
        let crc = u32_at(b, e + 12);
        let end = offset.saturating_add(u64::from(length));
        if end > b.len() as u64 || offset < header_len as u64 {
            return Err(FormatError::corrupt(
                path,
                format!("section {s} ({offset}+{length}) outside the file"),
            ));
        }
        let bytes = &b[offset as usize..end as usize];
        let actual = crc32fast::hash(bytes);
        if actual != crc {
            return Err(FormatError::Crc {
                path: path.to_owned(),
                what: format!("section {s}"),
                expected: crc,
                actual,
            });
        }
        sections.push(bytes);
    }
    Ok(Parsed { fixed, sections })
}

pub(crate) fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

pub(crate) fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().expect("4 bytes"))
}

pub(crate) fn i32_at(b: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(b[at..at + 4].try_into().expect("4 bytes"))
}

pub(crate) fn u64_at(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().expect("8 bytes"))
}

pub(crate) fn f32_at(b: &[u8], at: usize) -> f32 {
    f32::from_le_bytes(b[at..at + 4].try_into().expect("4 bytes"))
}

pub(crate) fn f64_at(b: &[u8], at: usize) -> f64 {
    f64::from_le_bytes(b[at..at + 8].try_into().expect("8 bytes"))
}

/// A string table: strings deduplicated in order of first use, stored as
/// `u32 byte length + UTF-8 bytes`, back to back.
#[derive(Default)]
pub(crate) struct StringTable {
    strings: Vec<String>,
    index: std::collections::HashMap<String, u32>,
}

impl StringTable {
    pub fn intern(&mut self, s: &str) -> u32 {
        if let Some(&i) = self.index.get(s) {
            return i;
        }
        let i = self.strings.len() as u32;
        self.strings.push(s.to_owned());
        self.index.insert(s.to_owned(), i);
        i
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for s in &self.strings {
            out.extend_from_slice(&(s.len() as u32).to_le_bytes());
            out.extend_from_slice(s.as_bytes());
        }
        out
    }
}

/// Decodes a string table of `count` strings.
pub(crate) fn decode_strings(path: &Path, b: &[u8], count: usize) -> Result<Vec<String>> {
    let mut out = Vec::with_capacity(count);
    let mut at = 0usize;
    for i in 0..count {
        if at + 4 > b.len() {
            return Err(FormatError::corrupt(
                path,
                format!("string {i} past the table"),
            ));
        }
        let len = u32_at(b, at) as usize;
        at += 4;
        if at + len > b.len() {
            return Err(FormatError::corrupt(
                path,
                format!("string {i} past the table"),
            ));
        }
        let s = std::str::from_utf8(&b[at..at + len])
            .map_err(|_| FormatError::corrupt(path, format!("string {i} is not UTF-8")))?;
        out.push(s.to_owned());
        at += len;
    }
    if at != b.len() {
        return Err(FormatError::corrupt(
            path,
            "trailing bytes after the string table",
        ));
    }
    Ok(out)
}

/// Looks a string index up in a decoded table.
pub(crate) fn string_at<'s>(
    path: &Path,
    strings: &'s [String],
    i: u32,
    what: &str,
) -> Result<&'s str> {
    strings.get(i as usize).map(String::as_str).ok_or_else(|| {
        FormatError::corrupt(path, format!("{what}: string {i} of {}", strings.len()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_crc_checks() {
        let fixed = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let a = b"hello".as_slice();
        let b = [9u8; 13];
        let bytes = encode(*b"TEST", &fixed, &[a, &b, &[]]).unwrap();
        let p = Path::new("t.bin");
        let parsed = parse(p, &bytes, *b"TEST", 8, |_| 3).unwrap();
        assert_eq!(parsed.fixed, fixed);
        assert_eq!(parsed.sections, vec![a, &b[..], &[][..]]);
        for k in 0..3 {
            let off = u64_at(&bytes, HEADER_BYTES + 8 + k * SECTION_ENTRY_BYTES);
            assert_eq!(off % 8, 0, "section {k} is 8-byte aligned");
        }

        let mut bad = bytes.clone();
        bad[HEADER_BYTES] ^= 1;
        assert!(matches!(
            parse(p, &bad, *b"TEST", 8, |_| 3),
            Err(FormatError::Crc { .. })
        ));
        let mut bad = bytes.clone();
        let off = u64_at(&bytes, HEADER_BYTES + 8) as usize;
        bad[off] ^= 1;
        assert!(matches!(
            parse(p, &bad, *b"TEST", 8, |_| 3),
            Err(FormatError::Crc { .. })
        ));
        let mut bad = bytes.clone();
        bad[8] = 7;
        assert!(matches!(
            parse(p, &bad, *b"TEST", 8, |_| 3),
            Err(FormatError::Version { .. })
        ));
        assert!(matches!(
            parse(p, &bytes, *b"XXXX", 8, |_| 3),
            Err(FormatError::Corrupt { .. })
        ));
    }

    #[test]
    fn strings() {
        let mut t = StringTable::default();
        assert_eq!(t.intern("a"), 0);
        assert_eq!(t.intern("bé"), 1);
        assert_eq!(t.intern("a"), 0);
        let b = t.encode();
        let back = decode_strings(Path::new("s"), &b, t.len()).unwrap();
        assert_eq!(back, vec!["a".to_string(), "bé".to_string()]);
        assert!(decode_strings(Path::new("s"), &b, 3).is_err());
    }
}

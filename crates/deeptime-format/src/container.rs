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

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::error::{FormatError, Result};
use crate::fsutil::read_exact_at;
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

/// A section's entry in the section table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectionRef {
    pub offset: u64,
    pub len: u32,
    pub crc32: u32,
}

/// Writes a sectioned file one section at a time into a seekable writer, for files too large to
/// build in memory (checkpoints carrying a recorded history, P1-17): a zeroed header first, each
/// section streamed through [`StreamWriter::section`], then [`StreamWriter::finish`] seeks back
/// and writes the header and section table. The bytes are exactly [`encode`]'s for the same
/// sections.
pub struct StreamWriter<W: Write + Seek> {
    w: W,
    path: PathBuf,
    magic: [u8; 4],
    fixed: Vec<u8>,
    count: usize,
    entries: Vec<SectionRef>,
    pos: u64,
}

/// What a section's writer writes into: counts the bytes and their CRC32.
pub struct SectionSink<'a, W: Write> {
    w: &'a mut W,
    crc: crc32fast::Hasher,
    len: u64,
}

impl<W: Write> Write for SectionSink<'_, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.w.write(buf)?;
        self.crc.update(&buf[..n]);
        self.len += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.w.flush()
    }
}

impl<W: Write + Seek> StreamWriter<W> {
    /// Starts a file of `count` sections (`path` only names it in errors).
    pub fn new(
        path: &Path,
        mut w: W,
        magic: [u8; 4],
        fixed: &[u8],
        count: usize,
    ) -> Result<StreamWriter<W>> {
        let header_len = HEADER_BYTES + fixed.len() + SECTION_ENTRY_BYTES * count;
        u32::try_from(header_len).map_err(|_| FormatError::Invalid("header too large".into()))?;
        w.seek(SeekFrom::Start(0))
            .and_then(|_| w.write_all(&vec![0u8; align8(header_len)]))
            .map_err(|e| FormatError::io(path, e))?;
        Ok(StreamWriter {
            w,
            path: path.to_owned(),
            magic,
            fixed: fixed.to_vec(),
            count,
            entries: Vec::with_capacity(count),
            pos: align8(header_len) as u64,
        })
    }

    /// Writes the next section with `fill`, then pads to 8 bytes; returns its table entry.
    pub fn section(
        &mut self,
        fill: impl FnOnce(&mut SectionSink<'_, W>) -> std::io::Result<()>,
    ) -> Result<SectionRef> {
        let k = self.entries.len();
        if k >= self.count {
            return Err(FormatError::Invalid(format!(
                "section {k} of a file of {}",
                self.count
            )));
        }
        let mut sink = SectionSink {
            w: &mut self.w,
            crc: crc32fast::Hasher::new(),
            len: 0,
        };
        fill(&mut sink).map_err(|e| FormatError::io(&self.path, e))?;
        let (len, crc32) = (sink.len, sink.crc.finalize());
        let len = u32::try_from(len)
            .map_err(|_| FormatError::Invalid(format!("section {k} larger than 4 GiB")))?;
        let entry = SectionRef {
            offset: self.pos,
            len,
            crc32,
        };
        let end = self.pos + u64::from(len);
        let pad = align8(end as usize) as u64 - end;
        self.w
            .write_all(&[0u8; 8][..pad as usize])
            .map_err(|e| FormatError::io(&self.path, e))?;
        self.pos = end + pad;
        self.entries.push(entry);
        Ok(entry)
    }

    /// Writes the header and the section table; every section must have been written.
    pub fn finish(mut self) -> Result<W> {
        if self.entries.len() != self.count {
            return Err(FormatError::Invalid(format!(
                "{} of {} sections written",
                self.entries.len(),
                self.count
            )));
        }
        let header_len = HEADER_BYTES + self.fixed.len() + SECTION_ENTRY_BYTES * self.count;
        let mut head = Vec::with_capacity(header_len);
        head.extend_from_slice(&self.magic);
        head.extend_from_slice(&(header_len as u32).to_le_bytes());
        head.extend_from_slice(&FORMAT_MINOR.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes());
        head.extend_from_slice(&[0; 4]);
        head.extend_from_slice(&self.fixed);
        for e in &self.entries {
            head.extend_from_slice(&e.offset.to_le_bytes());
            head.extend_from_slice(&e.len.to_le_bytes());
            head.extend_from_slice(&e.crc32.to_le_bytes());
        }
        let mut h = crc32fast::Hasher::new();
        h.update(&head[..12]);
        h.update(&head[HEADER_BYTES..header_len]);
        head[12..16].copy_from_slice(&h.finalize().to_le_bytes());
        self.w
            .flush()
            .and_then(|_| self.w.seek(SeekFrom::Start(0)))
            .and_then(|_| self.w.write_all(&head))
            .and_then(|_| self.w.flush())
            .map_err(|e| FormatError::io(&self.path, e))?;
        Ok(self.w)
    }
}

/// The header of a sectioned file on disk, read without its sections.
#[derive(Clone, Debug)]
pub struct FileTable {
    pub fixed: Vec<u8>,
    pub sections: Vec<SectionRef>,
}

/// Reads and checks a sectioned file's header and section table (magic, version, header CRC,
/// sections inside the file) without reading the sections; [`read_section`] and
/// [`check_section`] then read or verify one at a time.
pub fn read_table(
    path: &Path,
    file: &File,
    magic: [u8; 4],
    fixed_len: usize,
    count_from_fixed: impl FnOnce(&[u8]) -> usize,
) -> Result<FileTable> {
    let file_len = file.metadata().map_err(|e| FormatError::io(path, e))?.len();
    let start = HEADER_BYTES + fixed_len;
    if file_len < start as u64 {
        return Err(FormatError::corrupt(path, "shorter than its header"));
    }
    let mut b = vec![0u8; start];
    read_exact_at(file, &mut b, 0).map_err(|e| FormatError::io(path, e))?;
    let k = count_from_fixed(&b[HEADER_BYTES..]);
    let header_len = HEADER_BYTES + fixed_len + k.saturating_mul(SECTION_ENTRY_BYTES);
    if (header_len as u64) <= file_len {
        b.resize(header_len, 0);
        read_exact_at(file, &mut b[start..], start as u64).map_err(|e| FormatError::io(path, e))?;
    }
    // Everything but the section bytes checks exactly as `parse` does.
    let parsed = parse_header(path, &b, file_len, magic, fixed_len, k)?;
    Ok(FileTable {
        fixed: b[HEADER_BYTES..start].to_vec(),
        sections: parsed,
    })
}

/// Reads one section of a file whose table [`read_table`] returned, checking its CRC.
pub fn read_section(path: &Path, file: &File, s: &SectionRef, what: &str) -> Result<Vec<u8>> {
    let mut b = vec![0u8; s.len as usize];
    read_exact_at(file, &mut b, s.offset).map_err(|e| FormatError::io(path, e))?;
    let actual = crc32fast::hash(&b);
    if actual != s.crc32 {
        return Err(FormatError::Crc {
            path: path.to_owned(),
            what: what.to_owned(),
            expected: s.crc32,
            actual,
        });
    }
    Ok(b)
}

/// Checks one section's CRC by streaming it in 8 MiB pieces (no copy of the section is kept).
pub fn check_section(path: &Path, file: &File, s: &SectionRef, what: &str) -> Result<()> {
    const PIECE: usize = 8 << 20;
    let mut buf = vec![0u8; PIECE.min(s.len as usize)];
    let mut h = crc32fast::Hasher::new();
    let mut at = 0u64;
    while at < u64::from(s.len) {
        let n = (u64::from(s.len) - at).min(PIECE as u64) as usize;
        read_exact_at(file, &mut buf[..n], s.offset + at).map_err(|e| FormatError::io(path, e))?;
        h.update(&buf[..n]);
        at += n as u64;
    }
    let actual = h.finalize();
    if actual != s.crc32 {
        return Err(FormatError::Crc {
            path: path.to_owned(),
            what: what.to_owned(),
            expected: s.crc32,
            actual,
        });
    }
    Ok(())
}

/// Checks magic, version, header length and CRC of a header `b` (the header's bytes, or fewer
/// when the file is shorter than its declared header), and that every section lies inside a file
/// of `file_len` bytes; returns the section table.
fn parse_header(
    path: &Path,
    b: &[u8],
    file_len: u64,
    magic: [u8; 4],
    fixed_len: usize,
    k: usize,
) -> Result<Vec<SectionRef>> {
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
    let header_len = u32_at(b, 4) as usize;
    let expected_len = HEADER_BYTES + fixed_len + k.saturating_mul(SECTION_ENTRY_BYTES);
    if header_len != expected_len || header_len as u64 > file_len || header_len > b.len() {
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
        let len = u32_at(b, e + 8);
        let crc32 = u32_at(b, e + 12);
        let end = offset.saturating_add(u64::from(len));
        if end > file_len || offset < header_len as u64 {
            return Err(FormatError::corrupt(
                path,
                format!("section {s} ({offset}+{len}) outside the file"),
            ));
        }
        sections.push(SectionRef { offset, len, crc32 });
    }
    Ok(sections)
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
    let fixed = &b[HEADER_BYTES..HEADER_BYTES + fixed_len];
    let k = count_from_fixed(fixed);
    let table = parse_header(path, b, b.len() as u64, magic, fixed_len, k)?;
    let mut sections = Vec::with_capacity(k);
    for (s, e) in table.iter().enumerate() {
        let bytes = &b[e.offset as usize..(e.offset + u64::from(e.len)) as usize];
        let actual = crc32fast::hash(bytes);
        if actual != e.crc32 {
            return Err(FormatError::Crc {
                path: path.to_owned(),
                what: format!("section {s}"),
                expected: e.crc32,
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
    fn streamed_file_equals_encode_and_reads_back_by_section() {
        let fixed = [7u8, 0, 0, 0, 3, 0, 0, 0];
        let a = b"summary".as_slice();
        let b: Vec<u8> = (0..1000u32).map(|i| (i * 7) as u8).collect();
        let c = [0u8; 0];
        let d = [5u8; 17];
        let want = encode(*b"TEST", &fixed, &[a, &b, &c, &d]).unwrap();
        let p = Path::new("s.bin");
        let mut w =
            StreamWriter::new(p, std::io::Cursor::new(Vec::new()), *b"TEST", &fixed, 4).unwrap();
        w.section(|s| s.write_all(a)).unwrap();
        // A section written in pieces.
        let e = w
            .section(|s| {
                for piece in b.chunks(33) {
                    s.write_all(piece)?;
                }
                Ok(())
            })
            .unwrap();
        assert_eq!((e.len, e.crc32), (1000, crc32fast::hash(&b)));
        w.section(|_| Ok(())).unwrap();
        assert!(
            StreamWriter::new(p, std::io::Cursor::new(Vec::new()), *b"TEST", &fixed, 1)
                .unwrap()
                .finish()
                .is_err(),
            "a section missing"
        );
        w.section(|s| s.write_all(&d)).unwrap();
        assert!(w.section(|_| Ok(())).is_err(), "one section too many");
        let got = w.finish().unwrap().into_inner();
        assert_eq!(got, want, "the same bytes as encode");

        let dir = std::env::temp_dir().join(format!("dt-container-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("s.bin");
        std::fs::write(&path, &got).unwrap();
        let f = File::open(&path).unwrap();
        let t = read_table(&path, &f, *b"TEST", 8, |x| x[4] as usize + 1).unwrap();
        assert_eq!(t.fixed, fixed);
        assert_eq!(t.sections.len(), 4);
        assert_eq!(read_section(&path, &f, &t.sections[1], "b").unwrap(), b);
        check_section(&path, &f, &t.sections[3], "d").unwrap();
        // A flipped byte in a section fails its check; the table still reads.
        let mut bad = got.clone();
        bad[t.sections[1].offset as usize + 10] ^= 1;
        std::fs::write(&path, &bad).unwrap();
        let f = File::open(&path).unwrap();
        let t = read_table(&path, &f, *b"TEST", 8, |x| x[4] as usize + 1).unwrap();
        assert!(matches!(
            check_section(&path, &f, &t.sections[1], "b"),
            Err(FormatError::Crc { .. })
        ));
        // A truncated file: the table says a section lies past the end.
        std::fs::write(&path, &got[..got.len() - 24]).unwrap();
        let f = File::open(&path).unwrap();
        assert!(matches!(
            read_table(&path, &f, *b"TEST", 8, |x| x[4] as usize + 1),
            Err(FormatError::Corrupt { .. })
        ));
        std::fs::remove_dir_all(&dir).unwrap();
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

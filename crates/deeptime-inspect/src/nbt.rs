// SPDX-License-Identifier: LGPL-3.0-or-later
//! A small reader (and, for tests and fixtures, writer) of Minecraft's binary NBT, big-endian, as
//! stored in region-file chunks: a named root compound, thirteen tag types, strings in Java's
//! modified UTF-8.

use crate::bail;
use crate::error::Result;

/// One NBT value. Compounds keep their entries in file order.
#[derive(Clone, Debug, PartialEq)]
pub enum Tag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<Tag>),
    Compound(Vec<(String, Tag)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Tag {
    /// The entry `key` of a compound.
    pub fn get(&self, key: &str) -> Option<&Tag> {
        match self {
            Tag::Compound(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Tag::String(s) => Some(s),
            _ => None,
        }
    }

    /// Any integer tag widened to i64.
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Tag::Byte(v) => Some(i64::from(v)),
            Tag::Short(v) => Some(i64::from(v)),
            Tag::Int(v) => Some(i64::from(v)),
            Tag::Long(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Tag]> {
        match self {
            Tag::List(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_compound(&self) -> Option<&[(String, Tag)]> {
        match self {
            Tag::Compound(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_long_array(&self) -> Option<&[i64]> {
        match self {
            Tag::LongArray(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_int_array(&self) -> Option<&[i32]> {
        match self {
            Tag::IntArray(v) => Some(v),
            _ => None,
        }
    }

    fn id(&self) -> u8 {
        match self {
            Tag::Byte(_) => 1,
            Tag::Short(_) => 2,
            Tag::Int(_) => 3,
            Tag::Long(_) => 4,
            Tag::Float(_) => 5,
            Tag::Double(_) => 6,
            Tag::ByteArray(_) => 7,
            Tag::String(_) => 8,
            Tag::List(_) => 9,
            Tag::Compound(_) => 10,
            Tag::IntArray(_) => 11,
            Tag::LongArray(_) => 12,
        }
    }
}

/// Deeper nesting than this is refused rather than recursed into (Minecraft's own limit is 512).
const MAX_DEPTH: usize = 512;

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.b.len() - self.at < n {
            bail!("NBT truncated at byte {} (wanted {n} more)", self.at);
        }
        let s = &self.b[self.at..self.at + n];
        self.at += n;
        Ok(s)
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn i16(&mut self) -> Result<i16> {
        Ok(i16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn i64(&mut self) -> Result<i64> {
        Ok(i64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn len(&mut self) -> Result<usize> {
        let n = self.i32()?;
        if n < 0 {
            bail!("NBT negative length {n} at byte {}", self.at);
        }
        Ok(n as usize)
    }

    fn string(&mut self) -> Result<String> {
        let n = self.i16()? as u16 as usize;
        Ok(mutf8(self.take(n)?))
    }

    fn payload(&mut self, id: u8, depth: usize) -> Result<Tag> {
        if depth > MAX_DEPTH {
            bail!("NBT nested deeper than {MAX_DEPTH}");
        }
        Ok(match id {
            1 => Tag::Byte(self.u8()? as i8),
            2 => Tag::Short(self.i16()?),
            3 => Tag::Int(self.i32()?),
            4 => Tag::Long(self.i64()?),
            5 => Tag::Float(f32::from_bits(self.i32()? as u32)),
            6 => Tag::Double(f64::from_bits(self.i64()? as u64)),
            7 => {
                let n = self.len()?;
                Tag::ByteArray(self.take(n)?.iter().map(|&b| b as i8).collect())
            }
            8 => Tag::String(self.string()?),
            9 => {
                let inner = self.u8()?;
                let n = self.len()?;
                if inner == 0 && n > 0 {
                    bail!("NBT list of End tags with {n} entries");
                }
                // Each entry needs at least one byte (or none for empty compounds' End), so a
                // hostile length cannot make us allocate beyond the input.
                let mut v = Vec::with_capacity(n.min(self.b.len() - self.at));
                for _ in 0..n {
                    v.push(self.payload(inner, depth + 1)?);
                }
                Tag::List(v)
            }
            10 => {
                let mut v = Vec::new();
                loop {
                    let t = self.u8()?;
                    if t == 0 {
                        break;
                    }
                    let name = self.string()?;
                    v.push((name, self.payload(t, depth + 1)?));
                }
                Tag::Compound(v)
            }
            11 => {
                let n = self.len()?;
                let raw = self.take(n.saturating_mul(4))?;
                Tag::IntArray(
                    raw.as_chunks::<4>()
                        .0
                        .iter()
                        .map(|c| i32::from_be_bytes(*c))
                        .collect(),
                )
            }
            12 => {
                let n = self.len()?;
                let raw = self.take(n.saturating_mul(8))?;
                Tag::LongArray(
                    raw.as_chunks::<8>()
                        .0
                        .iter()
                        .map(|c| i64::from_be_bytes(*c))
                        .collect(),
                )
            }
            _ => bail!("NBT unknown tag type {id} at byte {}", self.at - 1),
        })
    }
}

/// Java's modified UTF-8 (NUL as C0 80, supplementary characters as surrogate pairs) to a
/// String; anything malformed becomes U+FFFD.
fn mutf8(b: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(b)
        && !b.iter().any(|&c| c == 0xC0 || c == 0xED)
    {
        return s.to_owned();
    }
    let mut units: Vec<u16> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c < 0x80 {
            units.push(u16::from(c));
            i += 1;
        } else if c & 0xE0 == 0xC0 && i + 1 < b.len() {
            units.push((u16::from(c & 0x1F) << 6) | u16::from(b[i + 1] & 0x3F));
            i += 2;
        } else if c & 0xF0 == 0xE0 && i + 2 < b.len() {
            units.push(
                (u16::from(c & 0x0F) << 12)
                    | (u16::from(b[i + 1] & 0x3F) << 6)
                    | u16::from(b[i + 2] & 0x3F),
            );
            i += 3;
        } else {
            units.push(0xFFFD);
            i += 1;
        }
    }
    String::from_utf16_lossy(&units)
}

/// Parses a whole NBT document: a root tag with its name.
pub fn parse(bytes: &[u8]) -> Result<(String, Tag)> {
    let mut r = Reader { b: bytes, at: 0 };
    let id = r.u8()?;
    if id == 0 {
        bail!("NBT document is a bare End tag");
    }
    let name = r.string()?;
    let tag = r.payload(id, 0)?;
    Ok((name, tag))
}

/// Writes a named root tag (for fixtures and tests). Strings are written as plain UTF-8, which
/// equals modified UTF-8 for text without NUL or supplementary characters.
pub fn write(name: &str, tag: &Tag) -> Vec<u8> {
    let mut out = vec![tag.id()];
    put_string(&mut out, name);
    put_payload(&mut out, tag);
    out
}

fn put_string(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u16).to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}

fn put_payload(out: &mut Vec<u8>, tag: &Tag) {
    match tag {
        Tag::Byte(v) => out.push(*v as u8),
        Tag::Short(v) => out.extend_from_slice(&v.to_be_bytes()),
        Tag::Int(v) => out.extend_from_slice(&v.to_be_bytes()),
        Tag::Long(v) => out.extend_from_slice(&v.to_be_bytes()),
        Tag::Float(v) => out.extend_from_slice(&v.to_bits().to_be_bytes()),
        Tag::Double(v) => out.extend_from_slice(&v.to_bits().to_be_bytes()),
        Tag::ByteArray(v) => {
            out.extend_from_slice(&(v.len() as i32).to_be_bytes());
            out.extend(v.iter().map(|&b| b as u8));
        }
        Tag::String(s) => put_string(out, s),
        Tag::List(v) => {
            out.push(v.first().map_or(0, Tag::id));
            out.extend_from_slice(&(v.len() as i32).to_be_bytes());
            for t in v {
                put_payload(out, t);
            }
        }
        Tag::Compound(v) => {
            for (k, t) in v {
                out.push(t.id());
                put_string(out, k);
                put_payload(out, t);
            }
            out.push(0);
        }
        Tag::IntArray(v) => {
            out.extend_from_slice(&(v.len() as i32).to_be_bytes());
            for x in v {
                out.extend_from_slice(&x.to_be_bytes());
            }
        }
        Tag::LongArray(v) => {
            out.extend_from_slice(&(v.len() as i32).to_be_bytes());
            for x in v {
                out.extend_from_slice(&x.to_be_bytes());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Tag {
        Tag::Compound(vec![
            ("b".into(), Tag::Byte(-3)),
            ("s".into(), Tag::Short(-300)),
            ("i".into(), Tag::Int(123_456)),
            ("l".into(), Tag::Long(-9_000_000_000)),
            ("f".into(), Tag::Float(1.5)),
            ("d".into(), Tag::Double(-2.25)),
            ("ba".into(), Tag::ByteArray(vec![1, -1, 7])),
            ("str".into(), Tag::String("minecraft:stone".into())),
            (
                "list".into(),
                Tag::List(vec![Tag::String("a".into()), Tag::String("b".into())]),
            ),
            ("empty".into(), Tag::List(vec![])),
            (
                "c".into(),
                Tag::Compound(vec![("Name".into(), Tag::String("x".into()))]),
            ),
            ("ia".into(), Tag::IntArray(vec![1, -2, 3])),
            ("la".into(), Tag::LongArray(vec![i64::MIN, 0, i64::MAX])),
        ])
    }

    #[test]
    fn round_trip() {
        let t = sample();
        let bytes = write("", &t);
        let (name, back) = parse(&bytes).unwrap();
        assert_eq!(name, "");
        assert_eq!(back, t);
        assert_eq!(
            back.get("str").and_then(Tag::as_str),
            Some("minecraft:stone")
        );
        assert_eq!(back.get("s").and_then(Tag::as_i64), Some(-300));
    }

    #[test]
    fn truncated_is_an_error() {
        let bytes = write("", &sample());
        for cut in [1, 5, bytes.len() / 2, bytes.len() - 1] {
            assert!(parse(&bytes[..cut]).is_err(), "cut at {cut}");
        }
    }

    #[test]
    fn modified_utf8() {
        // "a\0b" with NUL as C0 80, and U+1F600 as a surrogate pair (ED A0 BD ED B8 80).
        assert_eq!(mutf8(&[b'a', 0xC0, 0x80, b'b']), "a\0b");
        assert_eq!(mutf8(&[0xED, 0xA0, 0xBD, 0xED, 0xB8, 0x80]), "\u{1F600}");
        assert_eq!(mutf8("plain é".as_bytes()), "plain é");
    }
}

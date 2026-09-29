// SPDX-License-Identifier: LGPL-3.0-or-later
//! Preset share codes (plan §5.1): `DT1.` + base64url(deflate(canonical JSON)).
//!
//! - **Canonical JSON** is the preset parsed and written back compactly with every object's keys
//!   sorted (serde_json's default map), so one preset has one code.
//! - **deflate** is raw DEFLATE (RFC 1951, no zlib header), level 9, so Java decodes a code with
//!   `new java.util.zip.Inflater(true)` and no natives (a server's `config/deeptime/server.json`
//!   accepts share codes). The Java twin is `com.minecraftindustrialcomplex.deeptime.preset.ShareCode`;
//!   `tests/share-codes.json` holds codes written by each side, and both test suites decode both.
//! - **base64url** is RFC 4648 §5 without padding. Decoding also accepts padding and ignores ASCII
//!   whitespace (a code pasted across lines still works).
//!
//! Decoding inflates at most [`MAX_JSON_BYTES`] and requires the result to be JSON; it returns the
//! JSON text exactly as it was encoded.

use std::io::{Read, Write};

use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use serde_json::Value;

use crate::error::CoreError;

/// The prefix of a version-1 share code.
pub const PREFIX: &str = "DT1.";

/// The largest preset a share code may inflate to (a guard against deflate bombs).
pub const MAX_JSON_BYTES: usize = 1 << 20;

const WHAT: &str = "share code";

fn err(reason: impl Into<String>) -> CoreError {
    CoreError::preset(WHAT, reason)
}

/// True when `s` looks like a share code of any version (`DT<digits>.`), so a caller can route it
/// to [`decode`] and report an unsupported version instead of "file not found".
pub fn is_share_code(s: &str) -> bool {
    let s = s.trim_start();
    let Some(rest) = s.strip_prefix("DT") else {
        return false;
    };
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    digits > 0 && rest.as_bytes().get(digits) == Some(&b'.')
}

/// The canonical form of a JSON text: compact, object keys sorted.
pub fn canonical(json: &str) -> Result<String, CoreError> {
    let v: Value = serde_json::from_str(json)
        .map_err(|e| CoreError::preset("$", format!("not valid JSON: {e}")))?;
    Ok(v.to_string())
}

/// Encodes a preset (any JSON text; it is canonicalised first).
pub fn encode(json: &str) -> Result<String, CoreError> {
    let canon = canonical(json)?;
    let mut enc = DeflateEncoder::new(Vec::new(), Compression::best());
    enc.write_all(canon.as_bytes())
        .map_err(|e| err(format!("deflate: {e}")))?;
    let deflated = enc.finish().map_err(|e| err(format!("deflate: {e}")))?;
    Ok(format!("{PREFIX}{}", base64url_encode(&deflated)))
}

/// Decodes a share code into the JSON text it carries.
pub fn decode(code: &str) -> Result<String, CoreError> {
    let code = code.trim();
    let Some(body) = code.strip_prefix(PREFIX) else {
        if is_share_code(code) {
            let version = &code[..code.find('.').unwrap_or(code.len())];
            return Err(err(format!(
                "{version} codes are not supported by this Deep Time (it reads {})",
                &PREFIX[..PREFIX.len() - 1]
            )));
        }
        return Err(err(format!("a share code starts with {PREFIX}")));
    };
    let deflated = base64url_decode(body)?;
    let mut out = Vec::new();
    DeflateDecoder::new(deflated.as_slice())
        .take(MAX_JSON_BYTES as u64 + 1)
        .read_to_end(&mut out)
        .map_err(|e| err(format!("not a deflate stream: {e}")))?;
    if out.len() > MAX_JSON_BYTES {
        return Err(err(format!("inflates to more than {MAX_JSON_BYTES} bytes")));
    }
    let text = String::from_utf8(out).map_err(|_| err("does not hold UTF-8 text"))?;
    serde_json::from_str::<Value>(&text).map_err(|e| err(format!("does not hold JSON: {e}")))?;
    Ok(text)
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// RFC 4648 §5 base64url without padding.
pub fn base64url_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let chars = chunk.len() + 1;
        for k in 0..chars {
            out.push(ALPHABET[((n >> (18 - 6 * k)) & 63) as usize] as char);
        }
    }
    out
}

/// Decodes base64url; padding is optional and ASCII whitespace is skipped.
pub fn base64url_decode(text: &str) -> Result<Vec<u8>, CoreError> {
    let mut sextets: Vec<u8> = Vec::with_capacity(text.len());
    let mut padding = 0usize;
    for (pos, c) in text.bytes().enumerate() {
        if c.is_ascii_whitespace() {
            continue;
        }
        if c == b'=' {
            padding += 1;
            continue;
        }
        if padding > 0 {
            return Err(err(format!("base64url: data after padding at {pos}")));
        }
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => {
                return Err(err(format!(
                    "base64url: invalid character {:?} at {pos}",
                    c as char
                )));
            }
        };
        sextets.push(v);
    }
    if sextets.len() % 4 == 1 || padding > 2 {
        return Err(err("base64url: truncated"));
    }
    let mut out = Vec::with_capacity(sextets.len() * 3 / 4);
    for chunk in sextets.chunks(4) {
        let mut n = 0u32;
        for (k, s) in chunk.iter().enumerate() {
            n |= u32::from(*s) << (18 - 6 * k);
        }
        let bytes = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        out.extend_from_slice(&bytes[..chunk.len() - 1]);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `tests/share-codes.json`: canonical JSON with the code Rust writes and the code Java
    /// writes (`ShareCodeTest` in the mod decodes the same file).
    const FIXTURE: &str = include_str!("../tests/share-codes.json");

    #[test]
    fn base64url_matches_rfc_4648_vectors() {
        // RFC 4648 §10, re-alphabeted and unpadded.
        for (plain, enc) in [
            ("", ""),
            ("f", "Zg"),
            ("fo", "Zm8"),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg"),
            ("fooba", "Zm9vYmE"),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64url_encode(plain.as_bytes()), enc);
            assert_eq!(base64url_decode(enc).unwrap(), plain.as_bytes());
        }
        assert_eq!(base64url_decode("Zm8=").unwrap(), b"fo");
        assert_eq!(base64url_decode("Zm9v\n Yg").unwrap(), b"foob");
        assert_eq!(base64url_encode(&[0xfb, 0xff]), "-_8");
        assert!(base64url_decode("Zm9vY").is_err());
        assert!(base64url_decode("Zm+v").is_err());
        assert!(base64url_decode("Zg=g").is_err());
    }

    #[test]
    fn codes_round_trip_and_are_canonical() {
        let a = encode(r#"{ "b": 1, "a": {"y": [1, 2.5], "x": "é—" } }"#).unwrap();
        let b = encode(r#"{"a":{"x":"é—","y":[1,2.5]},"b":1}"#).unwrap();
        assert_eq!(a, b);
        assert!(a.starts_with(PREFIX));
        assert!(!a.contains('='));
        assert_eq!(decode(&a).unwrap(), r#"{"a":{"x":"é—","y":[1,2.5]},"b":1}"#);
        assert_eq!(decode(&format!("  {a}\n")).unwrap(), decode(&a).unwrap());
    }

    /// serde_json's number and string forms; the mod's `PresetsTest.canonicalJsonIsSerdes`
    /// expects the very same text from Java.
    #[test]
    fn canonical_numbers_follow_serde() {
        assert_eq!(
            canonical(concat!(
                r#"{"o":[true,false,null],"b":1.0E-7,"a":1e3,"c":0.001,"d":1.5e300,"e":123.456,"#,
                r#""f":18446744073709551615,"g":-9223372036854775808,"h":18446744073709551616,"i":1e16,"#,
                r#""j":123456789012345680.0,"k":0.00001,"l":0.000001,"m":-2.50,"n":0e5,"p":1e15}"#
            ))
            .unwrap(),
            concat!(
                r#"{"a":1000.0,"b":1e-7,"c":0.001,"d":1.5e+300,"e":123.456,"f":18446744073709551615,"#,
                r#""g":-9223372036854775808,"h":1.8446744073709552e+19,"i":1e+16,"j":1.2345678901234568e+17,"#,
                r#""k":0.00001,"l":1e-6,"m":-2.5,"n":0.0,"o":[true,false,null],"p":1000000000000000.0}"#
            )
        );
        assert_eq!(
            canonical(r#"{"s":"q\" b\\ \b\t\n\f\r\u0001 \u00e9 — \u2028"}"#).unwrap(),
            "{\"s\":\"q\\\" b\\\\ \\b\\t\\n\\f\\r\\u0001 é — \u{2028}\"}"
        );
    }

    #[test]
    fn bad_codes_say_why() {
        let reason = |code: &str| match decode(code) {
            Err(CoreError::PresetInvalid { path, reason }) => {
                assert_eq!(path, WHAT);
                reason
            }
            other => panic!("expected PresetInvalid, got {other:?}"),
        };
        assert!(reason("hello").contains("starts with DT1."));
        assert!(reason("DT2.abc").contains("DT2 codes are not supported"));
        assert!(reason("DT1.!!").contains("invalid character"));
        assert!(reason("DT1.Zm9vYmFy").contains("deflate"));
        // A deflated non-JSON text.
        let mut enc = DeflateEncoder::new(Vec::new(), Compression::best());
        enc.write_all(b"not json").unwrap();
        let code = format!("{PREFIX}{}", base64url_encode(&enc.finish().unwrap()));
        assert!(reason(&code).contains("does not hold JSON"));
        // A deflate bomb: 2 MiB of spaces.
        let mut enc = DeflateEncoder::new(Vec::new(), Compression::best());
        enc.write_all(&vec![b' '; 2 << 20]).unwrap();
        let code = format!("{PREFIX}{}", base64url_encode(&enc.finish().unwrap()));
        assert!(reason(&code).contains("more than"));
        assert!(is_share_code("DT1.x") && is_share_code("DT12.x"));
        assert!(!is_share_code("DT.x") && !is_share_code("earthlike_quick_16k"));
    }

    /// The cross-language check: Rust decodes the codes Java wrote and re-creates its own.
    #[test]
    fn fixture_codes_decode_on_both_sides() {
        let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
        let cases = fixture["cases"].as_array().unwrap();
        assert!(cases.len() >= 3);
        for case in cases {
            let name = case["name"].as_str().unwrap();
            let json = case["json"].as_str().unwrap();
            assert_eq!(
                canonical(json).unwrap(),
                json,
                "{name}: fixture JSON is canonical"
            );
            let rust = case["rust"].as_str().unwrap();
            assert_eq!(encode(json).unwrap(), rust, "{name}: Rust's code is stable");
            assert_eq!(decode(rust).unwrap(), json, "{name}: Rust's code decodes");
            let java = case["java"].as_str().unwrap();
            assert_eq!(
                decode(java).unwrap(),
                json,
                "{name}: Java's code decodes in Rust"
            );
        }
    }
}

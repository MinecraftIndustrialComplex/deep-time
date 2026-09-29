// SPDX-License-Identifier: LGPL-3.0-or-later
//! Timelapse keyframes on disk (plan §5.7.1, §6.1): `keyframes/kf-<n>.dtk`, one already-coloured
//! RGBA8 image per layer, each one LZ4 raw block, plus `keyframes/index.json`. The core produces
//! them while it runs (`deeptime_core::keyframes`); the viewer and, later, the in-game timelapse
//! read them. Byte layout in `FORMAT.md` ("Keyframes").

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::FORMAT_VERSION;
use crate::container::{self, f64_at, u32_at};
use crate::error::{FormatError, Result};
use crate::fsutil::write_atomic;

/// Magic of a keyframe file.
pub const MAGIC: [u8; 4] = *b"DTKF";
/// The keyframe folder inside a world-data folder.
pub const KEYFRAMES_DIR: &str = "keyframes";
/// The keyframe list, relative to the world-data folder.
pub const INDEX_PATH: &str = "keyframes/index.json";
const FIXED_BYTES: usize = 24;

/// `keyframes/kf-<n>.dtk`.
pub fn keyframe_path(n: u32) -> String {
    format!("{KEYFRAMES_DIR}/kf-{n}.dtk")
}

/// One timelapse frame: every layer as RGBA8, `w × h × 4` bytes, row-major, row 0 = north edge,
/// column 0 = the seam (longitude 0°, plan §5.2).
#[derive(Clone, Debug, PartialEq)]
pub struct KeyframeImage {
    /// Frame number within the run, from 0.
    pub n: u32,
    /// Simulated time, years relative to world creation.
    pub t: f64,
    /// The stage that produced it.
    pub stage: String,
    pub w: u32,
    pub h: u32,
    /// (layer name, RGBA8 bytes).
    pub layers: Vec<(String, Vec<u8>)>,
}

impl KeyframeImage {
    pub fn layer_names(&self) -> Vec<String> {
        self.layers.iter().map(|(n, _)| n.clone()).collect()
    }
}

/// Encodes a `.dtk` file.
pub fn encode_keyframe(k: &KeyframeImage) -> Result<Vec<u8>> {
    let size = k.w as usize * k.h as usize * 4;
    if k.w == 0 || k.h == 0 || k.layers.is_empty() {
        return Err(FormatError::Invalid(format!(
            "keyframe {}: empty ({}×{}, {} layers)",
            k.n,
            k.w,
            k.h,
            k.layers.len()
        )));
    }
    let mut blocks = Vec::with_capacity(k.layers.len());
    for (name, rgba) in &k.layers {
        if rgba.len() != size {
            return Err(FormatError::Invalid(format!(
                "keyframe {} layer {name}: {} bytes, expected {size}",
                k.n,
                rgba.len()
            )));
        }
        blocks.push(lz4_flex::block::compress(rgba));
    }
    // The string table: the stage, then the layer names in section order (not deduplicated).
    let mut table = Vec::new();
    for s in std::iter::once(&k.stage).chain(k.layers.iter().map(|(n, _)| n)) {
        table.extend_from_slice(&(s.len() as u32).to_le_bytes());
        table.extend_from_slice(s.as_bytes());
    }
    let mut fixed = Vec::with_capacity(FIXED_BYTES);
    fixed.extend_from_slice(&k.n.to_le_bytes());
    fixed.extend_from_slice(&k.w.to_le_bytes());
    fixed.extend_from_slice(&k.h.to_le_bytes());
    fixed.extend_from_slice(&(k.layers.len() as u32).to_le_bytes());
    fixed.extend_from_slice(&k.t.to_le_bytes());
    let mut sections: Vec<&[u8]> = vec![&table];
    sections.extend(blocks.iter().map(Vec::as_slice));
    container::encode(MAGIC, &fixed, &sections)
}

/// Decodes a `.dtk` file.
pub fn decode_keyframe(path: &Path, b: &[u8]) -> Result<KeyframeImage> {
    let p = container::parse(path, b, MAGIC, FIXED_BYTES, |f| 1 + u32_at(f, 12) as usize)?;
    let f = p.fixed;
    let (n, w, h, count) = (
        u32_at(f, 0),
        u32_at(f, 4),
        u32_at(f, 8),
        u32_at(f, 12) as usize,
    );
    let t = f64_at(f, 16);
    let strings = container::decode_strings(path, p.sections[0], 1 + count)?;
    let size = w as usize * h as usize * 4;
    let mut layers = Vec::with_capacity(count);
    for (k, name) in strings[1..].iter().enumerate() {
        let rgba = lz4_flex::block::decompress(p.sections[1 + k], size)
            .map_err(|e| FormatError::corrupt(path, format!("layer {name}: LZ4: {e}")))?;
        if rgba.len() != size {
            return Err(FormatError::corrupt(
                path,
                format!(
                    "layer {name}: {} bytes decoded, expected {size}",
                    rgba.len()
                ),
            ));
        }
        layers.push((name.clone(), rgba));
    }
    Ok(KeyframeImage {
        n,
        t,
        stage: strings[0].clone(),
        w,
        h,
        layers,
    })
}

/// One frame in `keyframes/index.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KeyframeEntry {
    pub n: u32,
    /// Years relative to world creation.
    pub t: f64,
    pub stage: String,
    pub w: u32,
    pub h: u32,
    pub layers: Vec<String>,
    /// Relative to the world-data folder.
    pub path: String,
    pub bytes: u64,
    pub crc32: u32,
}

/// `keyframes/index.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KeyframeIndex {
    pub format_version: String,
    pub frames: Vec<KeyframeEntry>,
}

impl Default for KeyframeIndex {
    fn default() -> Self {
        KeyframeIndex {
            format_version: FORMAT_VERSION.to_owned(),
            frames: Vec::new(),
        }
    }
}

impl KeyframeIndex {
    /// Reads `<world>/keyframes/index.json`; an absent file is an empty index.
    pub fn read(world: &Path) -> Result<KeyframeIndex> {
        let path = world.join(INDEX_PATH);
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(KeyframeIndex::default());
            }
            Err(e) => return Err(FormatError::io(&path, e)),
        };
        let index: KeyframeIndex =
            serde_json::from_slice(&bytes).map_err(|source| FormatError::Json {
                what: path.display().to_string(),
                source,
            })?;
        if index.format_version != FORMAT_VERSION {
            return Err(FormatError::Version {
                found: index.format_version,
                supported: FORMAT_VERSION.to_owned(),
            });
        }
        Ok(index)
    }

    /// Writes `<world>/keyframes/index.json` atomically; returns its size and CRC32.
    pub fn write(&self, world: &Path) -> Result<(u64, u32)> {
        let mut json = serde_json::to_vec_pretty(self).map_err(|source| FormatError::Json {
            what: INDEX_PATH.into(),
            source,
        })?;
        json.push(b'\n');
        let crc = write_atomic(&world.join(INDEX_PATH), &json)?;
        Ok((json.len() as u64, crc))
    }
}

/// Writes frame `k` as `keyframes/kf-<n>.dtk` under `world` and returns its index entry.
pub fn write_keyframe(world: &Path, k: &KeyframeImage) -> Result<KeyframeEntry> {
    let bytes = encode_keyframe(k)?;
    let rel = keyframe_path(k.n);
    let crc32 = write_atomic(&world.join(&rel), &bytes)?;
    Ok(KeyframeEntry {
        n: k.n,
        t: k.t,
        stage: k.stage.clone(),
        w: k.w,
        h: k.h,
        layers: k.layer_names(),
        path: rel,
        bytes: bytes.len() as u64,
        crc32,
    })
}

/// Reads frame `entry` from `world`, checking its size and CRC32 against the index.
pub fn read_keyframe(world: &Path, entry: &KeyframeEntry) -> Result<KeyframeImage> {
    let path = world.join(&entry.path);
    let bytes = std::fs::read(&path).map_err(|e| FormatError::io(&path, e))?;
    let crc = crc32fast::hash(&bytes);
    if bytes.len() as u64 != entry.bytes || crc != entry.crc32 {
        return Err(FormatError::Crc {
            path,
            what: "keyframe file".into(),
            expected: entry.crc32,
            actual: crc,
        });
    }
    decode_keyframe(&path, &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(n: u32) -> KeyframeImage {
        let (w, h) = (64u32, 32u32);
        let layer = |seed: u8| -> Vec<u8> {
            (0..w * h * 4)
                .map(|i| {
                    if i % 4 == 3 {
                        255
                    } else {
                        (i / 256) as u8 ^ seed
                    }
                })
                .collect()
        };
        KeyframeImage {
            n,
            t: -3.0e8 + f64::from(n) * 1.0e7,
            stage: "dummy_drift".into(),
            w,
            h,
            layers: vec![("elevation".into(), layer(1)), ("plates".into(), layer(9))],
        }
    }

    #[test]
    fn keyframes_round_trip() {
        let k = frame(3);
        let b = encode_keyframe(&k).unwrap();
        assert_eq!(decode_keyframe(Path::new("kf"), &b).unwrap(), k);
        let mut bad = k.clone();
        bad.layers[1].1.pop();
        assert!(encode_keyframe(&bad).is_err());

        let dir = std::env::temp_dir().join(format!("dt-kf-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(KeyframeIndex::read(&dir).unwrap().frames.is_empty());
        let mut index = KeyframeIndex::default();
        for n in 0..3 {
            index.frames.push(write_keyframe(&dir, &frame(n)).unwrap());
        }
        index.write(&dir).unwrap();
        let back = KeyframeIndex::read(&dir).unwrap();
        assert_eq!(back, index);
        assert_eq!(back.frames[2].path, "keyframes/kf-2.dtk");
        assert_eq!(read_keyframe(&dir, &back.frames[1]).unwrap(), frame(1));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

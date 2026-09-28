// SPDX-License-Identifier: LGPL-3.0-or-later
//! `manifest.json` (plan §5.7.1): what a world-data folder holds. Written last, atomically, so a
//! folder with a manifest is complete.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::FORMAT_VERSION;
use crate::error::{FormatError, Result};
use crate::fsutil::write_atomic;
use crate::raster::{Codec, DType, GridDesc};

/// The manifest's file name.
pub const MANIFEST: &str = "manifest.json";

/// Who wrote the world data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Writer {
    /// `deeptime-cli`, `deeptime-jni`, …
    pub name: String,
    /// The workspace version of the writer.
    pub version: String,
    /// `deeptime-format`'s crate version.
    pub format_crate: String,
}

impl Writer {
    /// A writer entry for `name` built from this workspace.
    pub fn this(name: &str) -> Writer {
        Writer {
            name: name.to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            format_crate: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}

/// The scale parameters the reader needs (plan §5.1 `ScaleParams`, the fixed subset in v0).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    pub circumference_blocks: u32,
    pub m_per_block_h: f64,
    pub m_per_block_v: f64,
    pub min_y: i32,
    pub max_y: i32,
    pub sea_level_y: i32,
}

impl Scale {
    /// The plan's defaults (§5.1) for circumference `c`.
    pub fn default_for(c: u32) -> Scale {
        Scale {
            circumference_blocks: c,
            m_per_block_h: 300.0,
            m_per_block_v: 20.0,
            min_y: -352,
            max_y: 639,
            sea_level_y: 63,
        }
    }
}

/// One tile pack of the folder.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerEntry {
    /// The layer name (`elevation`, `climate/temperature`, `strata/horizons`).
    pub name: String,
    /// Path relative to the world-data folder, `/`-separated.
    pub path: String,
    /// The key of its grid in [`Manifest::grids`].
    pub grid: String,
    pub dtype: DType,
    pub codec: Codec,
    pub bytes: u64,
    pub crc32: u32,
}

/// Any other file of the folder (JSON side files).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub bytes: u64,
    pub crc32: u32,
}

/// `manifest.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    /// `"0.<minor>"`; checked before anything else is parsed (plan §5.7.2).
    pub format_version: String,
    pub writers: Vec<Writer>,
    /// What produced the data: `synth:wrap-test`, `synth:lake-test`, `run:<preset>`.
    pub source: String,
    /// Hash of `preset.json`; none for synthetic worlds.
    #[serde(default)]
    pub preset_hash: Option<String>,
    /// Planet parameters (plan §5.1 `PlanetParams`); free-form in v0.
    #[serde(default)]
    pub planet: BTreeMap<String, serde_json::Value>,
    pub scale: Scale,
    /// Grid descriptors by name (`L1`, `L2`, `L3`).
    pub grids: BTreeMap<String, GridDesc>,
    pub layers: Vec<LayerEntry>,
    #[serde(default)]
    pub files: Vec<FileEntry>,
    /// Creation time, RFC 3339 UTC.
    pub created_utc: String,
    /// Source-specific notes (the synthetic worlds document their geometry here).
    #[serde(default)]
    pub notes: BTreeMap<String, serde_json::Value>,
}

impl Manifest {
    /// An empty manifest for this format version, stamped now.
    pub fn new(source: &str, writer: Writer, scale: Scale) -> Manifest {
        Manifest {
            format_version: FORMAT_VERSION.to_owned(),
            writers: vec![writer],
            source: source.to_owned(),
            preset_hash: None,
            planet: BTreeMap::new(),
            scale,
            grids: BTreeMap::new(),
            layers: Vec::new(),
            files: Vec::new(),
            created_utc: rfc3339_utc(SystemTime::now()),
            notes: BTreeMap::new(),
        }
    }

    /// The entry of layer `name`.
    pub fn layer(&self, name: &str) -> Option<&LayerEntry> {
        self.layers.iter().find(|l| l.name == name)
    }

    /// Reads `<dir>/manifest.json`, refusing another format version before parsing the rest.
    pub fn read(dir: &Path) -> Result<Manifest> {
        let path = dir.join(MANIFEST);
        let bytes = std::fs::read(&path).map_err(|e| FormatError::io(&path, e))?;
        let what = || path.display().to_string();
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|source| FormatError::Json {
                what: what(),
                source,
            })?;
        let found = value
            .get("format_version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FormatError::corrupt(&path, "no format_version"))?;
        if found != FORMAT_VERSION {
            return Err(FormatError::Version {
                found: found.to_owned(),
                supported: FORMAT_VERSION.to_owned(),
            });
        }
        serde_json::from_value(value).map_err(|source| FormatError::Json {
            what: what(),
            source,
        })
    }

    /// Writes `<dir>/manifest.json` atomically. Call it after every file it lists is written.
    pub fn write(&self, dir: &Path) -> Result<()> {
        let mut json = serde_json::to_vec_pretty(self).map_err(|source| FormatError::Json {
            what: "manifest".into(),
            source,
        })?;
        json.push(b'\n');
        write_atomic(&dir.join(MANIFEST), &json)?;
        Ok(())
    }

    /// Checks every listed file's size and CRC32 against the folder.
    pub fn verify(&self, dir: &Path) -> Result<()> {
        let listed = self
            .layers
            .iter()
            .map(|l| (&l.path, l.bytes, l.crc32))
            .chain(self.files.iter().map(|f| (&f.path, f.bytes, f.crc32)));
        for (rel, bytes, crc) in listed {
            let path = dir.join(rel);
            let data = std::fs::read(&path).map_err(|e| FormatError::io(&path, e))?;
            if data.len() as u64 != bytes {
                return Err(FormatError::corrupt(
                    &path,
                    format!("{} bytes, manifest says {bytes}", data.len()),
                ));
            }
            let actual = crc32fast::hash(&data);
            if actual != crc {
                return Err(FormatError::Crc {
                    path,
                    what: "file".into(),
                    expected: crc,
                    actual,
                });
            }
        }
        Ok(())
    }
}

/// Formats a time as RFC 3339 UTC with second precision (no date crate needed).
pub fn rfc3339_utc(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem / 60 % 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn rfc3339() {
        assert_eq!(rfc3339_utc(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        let t = UNIX_EPOCH + Duration::from_secs(1_790_000_000);
        assert_eq!(rfc3339_utc(t), "2026-09-21T14:13:20Z");
        let leap = UNIX_EPOCH + Duration::from_secs(951_782_400); // 2000-02-29
        assert_eq!(rfc3339_utc(leap), "2000-02-29T00:00:00Z");
    }

    #[test]
    fn round_trip_and_version_refusal() {
        let dir = std::env::temp_dir().join(format!("dt-manifest-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let crc = write_atomic(&dir.join("side.json"), b"{}").unwrap();
        let mut m = Manifest::new("test", Writer::this("unit"), Scale::default_for(1024));
        m.grids.insert(
            "L3".into(),
            GridDesc::for_planet("L3", 1024, 2).unwrap(),
        );
        m.files.push(FileEntry {
            path: "side.json".into(),
            bytes: 2,
            crc32: crc,
        });
        m.write(&dir).unwrap();
        let back = Manifest::read(&dir).unwrap();
        assert_eq!(back, m);
        back.verify(&dir).unwrap();

        let text = std::fs::read_to_string(dir.join(MANIFEST)).unwrap();
        std::fs::write(
            dir.join(MANIFEST),
            text.replace("\"format_version\": \"0.0\"", "\"format_version\": \"0.3\""),
        )
        .unwrap();
        let err = Manifest::read(&dir).unwrap_err().to_string();
        assert_eq!(
            err,
            "world data 0.3, this Deep Time reads 0.0; before 1.0 worlds are not migrated"
        );

        std::fs::write(dir.join("side.json"), b"{ }").unwrap();
        assert!(m.verify(&dir).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

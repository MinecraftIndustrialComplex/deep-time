// SPDX-License-Identifier: LGPL-3.0-or-later
//! `manifest.json` (plan §5.7.1): what a world-data folder holds. Written last, atomically, so a
//! folder with a manifest is complete.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{FormatError, Result};
use crate::fsutil::write_atomic;
use crate::projection::{ProjectionKind, ProjectionSpec, WorldProjection};
use crate::raster::{Codec, DType, GridDesc};
use crate::{CUBE_FORMAT_VERSION, FORMAT_VERSION, READS_VERSIONS};

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

/// The default vertical scale, metres of planet per block up (`scale.m_per_block_v`): 35 m, an
/// 8.6-fold vertical exaggeration against the 300 m horizontal block (Ben, decisions log
/// 2026-10-02, "35 m per block (Recommended)", after docs/research/notes/vertical-scale-study.md:
/// at 20 m, 42 % of the land was drawn steeper than 45° against vanilla's 15 %). A game choice.
pub const DEFAULT_M_PER_BLOCK_V: f64 = 35.0; // ledger: SCL-002 (game choice, Ben)

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
            m_per_block_v: DEFAULT_M_PER_BLOCK_V,
            min_y: -512,
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

/// Any other file of the folder (JSON side files, events, sites, keyframes).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub bytes: u64,
    pub crc32: u32,
}

impl FileEntry {
    /// The entry of `<dir>/<rel>` as it is on disk now.
    pub fn of(dir: &Path, rel: &str) -> Result<FileEntry> {
        let path = dir.join(rel);
        let data = std::fs::read(&path).map_err(|e| FormatError::io(&path, e))?;
        Ok(FileEntry {
            path: rel.to_owned(),
            bytes: data.len() as u64,
            crc32: crc32fast::hash(&data),
        })
    }
}

/// The format version of a world of projection `kind` (`None`: the Lambert frame):
/// [`CUBE_FORMAT_VERSION`] for a cube world, [`FORMAT_VERSION`] otherwise.
pub fn version_for(kind: Option<ProjectionKind>) -> &'static str {
    if kind == Some(ProjectionKind::Cube) {
        CUBE_FORMAT_VERSION
    } else {
        FORMAT_VERSION
    }
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
    /// The world's projection and layout (FORMAT.md "The world's layout"): the edges, their
    /// motions, the cone points and the poles. Absent: a Lambert world of `scale.circumference_blocks`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection: Option<ProjectionSpec>,
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
            projection: None,
            grids: BTreeMap::new(),
            layers: Vec::new(),
            files: Vec::new(),
            created_utc: rfc3339_utc(SystemTime::now()),
            notes: BTreeMap::new(),
        }
    }

    /// The world's projection ([`Manifest::projection`]; Lambert when absent).
    pub fn world_projection(&self) -> WorldProjection {
        WorldProjection::from_spec(self.projection.as_ref(), self.scale.circumference_blocks)
    }

    /// Sets the layout block and the format version that goes with it: [`CUBE_FORMAT_VERSION`]
    /// for a cube world, [`FORMAT_VERSION`] for the rest (FORMAT.md "Lambert-frame and cube worlds
    /// side by side").
    pub fn set_projection(&mut self, spec: Option<ProjectionSpec>) {
        self.format_version = version_for(spec.as_ref().map(|s| s.kind)).to_owned();
        self.projection = spec;
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
        if found != FORMAT_VERSION && found != CUBE_FORMAT_VERSION {
            return Err(FormatError::Version {
                found: found.to_owned(),
                supported: READS_VERSIONS.to_owned(),
            });
        }
        let found = found.to_owned();
        let m: Manifest = serde_json::from_value(value).map_err(|source| FormatError::Json {
            what: what(),
            source,
        })?;
        // A cube world is 0.7 and only a cube world is (FORMAT.md "Lambert-frame and cube worlds
        // side by side"); its map table and rotation must be ones this crate can draw.
        let kind = m.projection.as_ref().map(|s| s.kind);
        let want = version_for(kind);
        if found != want {
            return Err(FormatError::corrupt(
                &path,
                format!(
                    "format_version {found} on a {} world, which is {want}",
                    kind.unwrap_or(ProjectionKind::Lambert).name()
                ),
            ));
        }
        if kind == Some(ProjectionKind::Cube) {
            WorldProjection::try_from_spec(m.projection.as_ref(), m.scale.circumference_blocks)?;
        }
        Ok(m)
    }

    /// The exact bytes [`Manifest::write`] puts on disk: pretty JSON and a final newline.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>> {
        let mut json = serde_json::to_vec_pretty(self).map_err(|source| FormatError::Json {
            what: "manifest".into(),
            source,
        })?;
        json.push(b'\n');
        Ok(json)
    }

    /// Writes `<dir>/manifest.json` atomically. Call it after every file it lists is written.
    pub fn write(&self, dir: &Path) -> Result<()> {
        write_atomic(&dir.join(MANIFEST), &self.to_json_bytes()?)?;
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
    let secs = t
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
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
    fn the_layout_block_round_trips_and_names_the_projection() {
        use crate::cube::CubeTurn;
        use crate::cube_map::{CubeMap, WORLD_MAP};
        use crate::projection::{EdgeKind, ProjectionKind, WorldProjection};
        let map = CubeMap::builtin_static(WORLD_MAP).expect("the world map");
        let wp = WorldProjection::cube(1_024, map, CubeTurn::polar(31.25));
        let mut m = Manifest::new("test", Writer::this("unit"), Scale::default_for(4_096));
        assert_eq!(
            m.world_projection().kind(),
            ProjectionKind::Lambert,
            "absent: the Lambert frame"
        );
        m.projection = Some(wp.spec(None));
        let json = String::from_utf8(m.to_json_bytes().unwrap()).unwrap();
        assert!(json.contains("\"kind\": \"cube\""), "{json}");
        let back: Manifest = serde_json::from_str(&json).unwrap();
        // serde_json's default parser may land a float one ulp off (the corners' and faces'
        // latitudes and longitudes): everything else is compared exactly, those to 1e-12.
        let (a, b) = (
            back.projection.as_ref().unwrap(),
            m.projection.as_ref().unwrap(),
        );
        assert_eq!(
            (&a.edges, &a.poles, &a.domain, &a.border, a.kind),
            (&b.edges, &b.poles, &b.domain, &b.border, b.kind)
        );
        assert_eq!((a.face_blocks, &a.map), (b.face_blocks, &b.map));
        for (p, q) in a.cone_points.iter().zip(&b.cone_points) {
            assert_eq!((&p.name, &p.images), (&q.name, &q.images));
            assert!((p.lat_deg - q.lat_deg).abs() < 1e-12 && (p.lon_deg - q.lon_deg).abs() < 1e-12);
        }
        let spec = back.projection.as_ref().unwrap();
        assert_eq!(spec.edges.len(), 14);
        assert_eq!(
            spec.edges
                .iter()
                .filter(|e| e.kind == EdgeKind::Slide)
                .count(),
            2
        );
        match back.world_projection() {
            WorldProjection::Cube(v) => assert_eq!(v.cube.side(), 1_024),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn round_trip_and_version_refusal() {
        let dir = std::env::temp_dir().join(format!("dt-manifest-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let crc = write_atomic(&dir.join("side.json"), b"{}").unwrap();
        let mut m = Manifest::new("test", Writer::this("unit"), Scale::default_for(1024));
        m.grids
            .insert("L3".into(), GridDesc::for_planet("L3", 1024, 2).unwrap());
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
            text.replace(
                &format!("\"format_version\": \"{FORMAT_VERSION}\""),
                "\"format_version\": \"0.9\"",
            ),
        )
        .unwrap();
        let err = Manifest::read(&dir).unwrap_err().to_string();
        assert_eq!(
            err,
            format!(
                "world data 0.9, this Deep Time reads {READS_VERSIONS}; before 1.0 worlds are not migrated"
            )
        );

        std::fs::write(dir.join("side.json"), b"{ }").unwrap();
        assert!(m.verify(&dir).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

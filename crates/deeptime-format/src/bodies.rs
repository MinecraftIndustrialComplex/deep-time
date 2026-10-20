// SPDX-License-Identifier: LGPL-3.0-or-later
//! Bodies (plan §5.7.4): deposit bodies, and later intrusions and sky islands, as parametric
//! primitives with a transform, an AABB and zoning. On disk: `bodies/bodies.dtb`, a sectioned
//! file (`FORMAT.md`, "Bodies"). The Java adapter evaluates the same two metrics with the same
//! formulas ([`Body::locate`]); P1-15's parity test covers them.
//!
//! A body is a primitive in its own frame: semi-axes `a` (local x), `b` (local y, the vertical
//! half-height) and `c` (local z) in blocks, turned by `yaw` about the vertical and tilted by
//! `dip` about the local x axis. Two metrics cover every shape: the **ellipsoid** metric
//! `d = √((x/a)² + (y/b)² + (z/c)²)` and the **cylinder** metric
//! `d = max(√((x/a)² + (z/c)²), |y/b|)` (an elliptic cylinder: pipes, sheets, seams, blankets,
//! channels). A block is inside when `d ≤ 1` and its y is at or below `clip_top_y`. The vertical
//! position `v = (b − y)/(2b)` runs from 0 at the body's local top to 1 at its bottom. A zone
//! applies where `d0 ≤ d < d1` and `v0 ≤ v < v1` (an upper bound of 1 is inclusive); zones may
//! overlap (several products in one place).

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::container::{self, StringTable, f32_at, f64_at, i32_at, string_at, u32_at};
use crate::error::{FormatError, Result};
use crate::fsutil::write_atomic;
use crate::sites::{NONE_U32, SiteId};

/// Magic of the bodies file.
pub const MAGIC: [u8; 4] = *b"DTBD";
/// The bodies file, relative to the world-data folder.
pub const BODIES_PATH: &str = "bodies/bodies.dtb";
/// Bytes per body record.
pub const BODY_BYTES: usize = 112;
/// Bytes per zone record.
pub const ZONE_BYTES: usize = 40;
/// "No clip" for `clip_top_y`.
pub const NO_CLIP: i32 = i32::MAX;

const FIXED_BYTES: usize = 16;
const SECTIONS: usize = 3;

/// A body's id, stable for the life of the world; `u32::MAX` is reserved.
pub type BodyId = u32;

/// The primitive's shape. The name records what the body is; the metric decides the maths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shape {
    Ellipsoid,
    StockworkShell,
    Lens,
    Pod,
    Sheet,
    Dyke,
    Pipe,
    Blanket,
    Seam,
    Channel,
    Dome,
}

/// The distance function a shape uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metric {
    Ellipsoid,
    Cylinder,
}

impl Shape {
    pub const ALL: [Shape; 11] = [
        Shape::Ellipsoid,
        Shape::StockworkShell,
        Shape::Lens,
        Shape::Pod,
        Shape::Sheet,
        Shape::Dyke,
        Shape::Pipe,
        Shape::Blanket,
        Shape::Seam,
        Shape::Channel,
        Shape::Dome,
    ];

    pub fn code(self) -> u8 {
        Shape::ALL
            .iter()
            .position(|s| *s == self)
            .expect("every shape is in ALL") as u8
    }

    pub fn from_code(c: u8) -> Option<Shape> {
        Shape::ALL.get(usize::from(c)).copied()
    }

    /// The name used in deposit-type JSON (`geometry.shape`) and in the viewer.
    pub fn name(self) -> &'static str {
        match self {
            Shape::Ellipsoid => "ellipsoid",
            Shape::StockworkShell => "stockwork_shell",
            Shape::Lens => "lens",
            Shape::Pod => "pod",
            Shape::Sheet => "sheet",
            Shape::Dyke => "dyke",
            Shape::Pipe => "pipe",
            Shape::Blanket => "blanket",
            Shape::Seam => "seam",
            Shape::Channel => "channel",
            Shape::Dome => "dome",
        }
    }

    pub fn from_name(name: &str) -> Option<Shape> {
        Shape::ALL.iter().copied().find(|s| s.name() == name)
    }

    pub fn metric(self) -> Metric {
        match self {
            Shape::Ellipsoid | Shape::StockworkShell | Shape::Lens | Shape::Pod | Shape::Dome => {
                Metric::Ellipsoid
            }
            Shape::Sheet
            | Shape::Dyke
            | Shape::Pipe
            | Shape::Blanket
            | Shape::Seam
            | Shape::Channel => Metric::Cylinder,
        }
    }
}

/// One zone of a body.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    /// `potassic_core`, `enrichment_blanket`, …
    pub name: String,
    /// The product placed in this zone (`c:copper`), or none (an alteration or cap zone).
    pub material: Option<String>,
    /// A rock class the zone's host rock becomes (`ironstone` for a gossan), or none.
    pub rock_class: Option<String>,
    /// Normalised-distance range [d0, d1).
    pub d: [f32; 2],
    /// Vertical-position range [v0, v1) (0 = the body's local top, 1 = its bottom).
    pub v: [f32; 2],
    /// Mass fraction of `material` in ore blocks' rock (for prospecting and COE yields).
    pub grade: f32,
    /// Probability (0–1) that a host block in the zone becomes ore.
    pub density: f32,
    /// Of the ore blocks the zone places, the share (0–1) that are the material's raw-ore block
    /// (`raw_iron_block`, …: vanilla's ore veins, Ben 2026-10-02); 0 = none. A second draw,
    /// independent of the ore draw (FORMAT.md).
    #[serde(default)]
    pub raw: f32,
}

impl Zone {
    fn contains(&self, d: f64, v: f64) -> bool {
        let within = |x: f64, [lo, hi]: [f32; 2]| {
            let (lo, hi) = (f64::from(lo), f64::from(hi));
            x >= lo && (x < hi || (hi >= 1.0 && x <= hi))
        };
        within(d, self.d) && within(v, self.v)
    }
}

/// One body (plan §5.7.4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Body {
    pub id: BodyId,
    /// The site the body belongs to (a site's bodies are consecutive ids from `Site::body`).
    pub site: Option<SiteId>,
    pub shape: Shape,
    /// What the body is within its site: `ore`, `lithocap`, `gossan`, `reservoir`, …
    pub role: String,
    /// Block space; x in the canonical span [−C/2, C/2).
    pub center: [f64; 3],
    /// Semi-axes in blocks: a (local x), b (local y, vertical half-height), c (local z).
    pub axes: [f32; 3],
    /// Radians about the vertical, from +x towards +z.
    pub yaw: f32,
    /// Radians about the local x axis (a tilted sheet or dyke).
    pub dip: f32,
    /// Blocks above this y are outside the body (an erosion surface), or [`NO_CLIP`].
    pub clip_top_y: i32,
    /// Inclusive block bounds [min, max]; x may leave the canonical span (readers wrap).
    pub aabb: [[i32; 3]; 2],
    /// A rock class the whole body's host becomes (`banded_iron_formation`), or none.
    pub rock_class: Option<String>,
    pub zones: Vec<Zone>,
    /// When the body formed, Myr before the end of the world's history (a pluton's emplacement;
    /// decisions log 2026-10-19), or none.
    #[serde(default)]
    pub age_ma: Option<f32>,
}

/// Where a block centre sits in a body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    /// Normalised distance (≤ 1 inside).
    pub d: f64,
    /// Vertical position, 0 at the local top, 1 at the bottom.
    pub v: f64,
}

impl Body {
    /// A body with its AABB computed from the transform (clip applied).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: BodyId,
        site: Option<SiteId>,
        shape: Shape,
        role: &str,
        center: [f64; 3],
        axes: [f32; 3],
        yaw: f32,
        dip: f32,
        clip_top_y: i32,
    ) -> Body {
        let mut b = Body {
            id,
            site,
            shape,
            role: role.to_owned(),
            center,
            axes,
            yaw,
            dip,
            clip_top_y,
            aabb: [[0; 3]; 2],
            rock_class: None,
            zones: Vec::new(),
            age_ma: None,
        };
        b.aabb = b.computed_aabb();
        b
    }

    /// The rotation's rows: world offset → local (x, y, z).
    fn to_local(&self) -> [[f64; 3]; 3] {
        let (sy, cy) = f64::from(self.yaw).sin_cos();
        let (sd, cd) = f64::from(self.dip).sin_cos();
        // Yaw about +y: x' = cy·dx + sy·dz, z' = −sy·dx + cy·dz; then dip about local x:
        // y'' = cd·dy − sd·z', z'' = sd·dy + cd·z'.
        [
            [cy, 0.0, sy],
            [sd * sy, cd, -sd * cy],
            [-cd * sy, sd, cd * cy],
        ]
    }

    /// The conservative block AABB of the transformed primitive, clipped at `clip_top_y`.
    pub fn computed_aabb(&self) -> [[i32; 3]; 2] {
        let r = self.to_local();
        let h = self.axes.map(f64::from);
        // World extent along axis i is Σ_j |R⁻¹_ij| h_j; R⁻¹ = Rᵀ for a rotation.
        let ext: [f64; 3] = std::array::from_fn(|i| (0..3).map(|j| r[j][i].abs() * h[j]).sum());
        let mut lo: [i32; 3] = std::array::from_fn(|i| (self.center[i] - ext[i]).floor() as i32);
        let mut hi: [i32; 3] = std::array::from_fn(|i| (self.center[i] + ext[i]).ceil() as i32);
        if self.clip_top_y != NO_CLIP {
            hi[1] = hi[1].min(self.clip_top_y);
            lo[1] = lo[1].min(hi[1]);
        }
        for k in 0..3 {
            lo[k] = lo[k].min(hi[k]);
        }
        [lo, hi]
    }

    /// Where the centre of block (x, y, z) lies relative to the body on a planet of
    /// circumference `c` (x wraps), or `None` outside it. The sample point is the block centre
    /// (x + 0.5, y + 0.5, z + 0.5).
    pub fn locate(&self, c: u32, x: i32, y: i32, z: i32) -> Option<Location> {
        if self.clip_top_y != NO_CLIP && y > self.clip_top_y {
            return None;
        }
        let cf = f64::from(c);
        let mut dx = f64::from(x) + 0.5 - self.center[0];
        dx -= (dx / cf + 0.5).floor() * cf;
        let dy = f64::from(y) + 0.5 - self.center[1];
        let dz = f64::from(z) + 0.5 - self.center[2];
        let r = self.to_local();
        let l: [f64; 3] = std::array::from_fn(|i| r[i][0] * dx + r[i][1] * dy + r[i][2] * dz);
        let [a, b, cc] = self.axes.map(|v| f64::from(v).max(1e-6));
        let d = match self.shape.metric() {
            Metric::Ellipsoid => {
                ((l[0] / a).powi(2) + (l[1] / b).powi(2) + (l[2] / cc).powi(2)).sqrt()
            }
            Metric::Cylinder => ((l[0] / a).powi(2) + (l[2] / cc).powi(2))
                .sqrt()
                .max((l[1] / b).abs()),
        };
        if d > 1.0 {
            return None;
        }
        let v = ((b - l[1]) / (2.0 * b)).clamp(0.0, 1.0);
        Some(Location { d, v })
    }

    /// The zones that apply at a location.
    pub fn zones_at(&self, at: Location) -> impl Iterator<Item = &Zone> {
        self.zones.iter().filter(move |z| z.contains(at.d, at.v))
    }

    /// Volume in blocks³ before clipping.
    pub fn volume_blocks(&self) -> f64 {
        let [a, b, c] = self.axes.map(f64::from);
        match self.shape.metric() {
            Metric::Ellipsoid => 4.0 / 3.0 * std::f64::consts::PI * a * b * c,
            Metric::Cylinder => std::f64::consts::PI * a * c * 2.0 * b,
        }
    }

    pub fn validate(&self) -> Result<()> {
        let bad = |m: String| Err(FormatError::Invalid(format!("body {}: {m}", self.id)));
        if self.id == NONE_U32 {
            return bad("id u32::MAX is reserved".into());
        }
        if self.site == Some(NONE_U32) {
            return bad("site u32::MAX is reserved".into());
        }
        if !self.center.iter().all(|v| v.is_finite()) {
            return bad("centre is not finite".into());
        }
        if !self.axes.iter().all(|v| v.is_finite() && *v > 0.0) {
            return bad(format!("semi-axes {:?} must be finite and > 0", self.axes));
        }
        if !self.yaw.is_finite() || !self.dip.is_finite() {
            return bad("rotation is not finite".into());
        }
        if self.age_ma.is_some_and(|a| !a.is_finite()) {
            return bad("age is not finite".into());
        }
        let [lo, hi] = self.aabb;
        if (0..3).any(|k| lo[k] > hi[k]) {
            return bad(format!("empty AABB {:?}", self.aabb));
        }
        for z in &self.zones {
            let ok = |r: [f32; 2]| r[0].is_finite() && r[1].is_finite() && r[0] <= r[1];
            if !ok(z.d)
                || !ok(z.v)
                || !(0.0..=1.0).contains(&z.density)
                || !(0.0..=1.0).contains(&z.raw)
                || !z.grade.is_finite()
            {
                return bad(format!(
                    "zone {} has a bad range, grade, density or raw share",
                    z.name
                ));
            }
        }
        Ok(())
    }

    /// The JSON form the viewer reads (`FORMAT.md`, "Bodies").
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "site": self.site,
            "shape": self.shape.name(),
            "metric": match self.shape.metric() { Metric::Ellipsoid => "ellipsoid", Metric::Cylinder => "cylinder" },
            "role": self.role,
            "center": self.center,
            "axes": self.axes,
            "yaw": self.yaw,
            "dip": self.dip,
            "clip_top_y": (self.clip_top_y != NO_CLIP).then_some(self.clip_top_y),
            "aabb": self.aabb,
            "rock_class": self.rock_class,
            "age_ma": self.age_ma,
            "zones": self.zones.iter().map(|z| json!({
                "name": z.name, "material": z.material, "rock_class": z.rock_class,
                "d": z.d, "v": z.v, "grade": z.grade, "density": z.density, "raw": z.raw,
            })).collect::<Vec<_>>(),
        })
    }
}

fn opt_string(strings: &mut StringTable, s: &Option<String>) -> u32 {
    s.as_deref().map_or(NONE_U32, |s| strings.intern(s))
}

/// Encodes `bodies/bodies.dtb`: bodies sorted by id (ids unique).
pub fn encode_bodies(bodies: &[Body]) -> Result<Vec<u8>> {
    let mut sorted: Vec<&Body> = bodies.iter().collect();
    sorted.sort_by_key(|b| b.id);
    for w in sorted.windows(2) {
        if w[0].id == w[1].id {
            return Err(FormatError::Invalid(format!(
                "body {} appears twice",
                w[0].id
            )));
        }
    }
    let mut strings = StringTable::default();
    let mut records = Vec::with_capacity(sorted.len() * BODY_BYTES);
    let mut zones = Vec::new();
    for b in &sorted {
        b.validate()?;
        let first_zone = (zones.len() / ZONE_BYTES) as u32;
        for z in &b.zones {
            zones.extend_from_slice(&strings.intern(&z.name).to_le_bytes());
            zones.extend_from_slice(&opt_string(&mut strings, &z.material).to_le_bytes());
            zones.extend_from_slice(&opt_string(&mut strings, &z.rock_class).to_le_bytes());
            for v in [z.d[0], z.d[1], z.v[0], z.v[1], z.grade, z.density, z.raw] {
                zones.extend_from_slice(&v.to_le_bytes());
            }
        }
        let start = records.len();
        records.extend_from_slice(&b.id.to_le_bytes());
        records.extend_from_slice(&b.site.unwrap_or(NONE_U32).to_le_bytes());
        records.push(b.shape.code());
        records.push(u8::from(b.clip_top_y != NO_CLIP));
        records.extend_from_slice(&0u16.to_le_bytes());
        records.extend_from_slice(&strings.intern(&b.role).to_le_bytes());
        for v in b.center {
            records.extend_from_slice(&v.to_le_bytes());
        }
        for v in b.axes {
            records.extend_from_slice(&v.to_le_bytes());
        }
        records.extend_from_slice(&b.yaw.to_le_bytes());
        records.extend_from_slice(&b.dip.to_le_bytes());
        records.extend_from_slice(&b.clip_top_y.to_le_bytes());
        for corner in b.aabb {
            for v in corner {
                records.extend_from_slice(&v.to_le_bytes());
            }
        }
        records.extend_from_slice(&first_zone.to_le_bytes());
        records.extend_from_slice(&(b.zones.len() as u32).to_le_bytes());
        records.extend_from_slice(&opt_string(&mut strings, &b.rock_class).to_le_bytes());
        records.extend_from_slice(&b.age_ma.unwrap_or(f32::NAN).to_le_bytes());
        records.extend_from_slice(&0u64.to_le_bytes());
        debug_assert_eq!(records.len() - start, BODY_BYTES);
    }
    let strings_bytes = strings.encode();
    let mut fixed = Vec::with_capacity(FIXED_BYTES);
    fixed.extend_from_slice(&(sorted.len() as u32).to_le_bytes());
    fixed.extend_from_slice(&(strings.len() as u32).to_le_bytes());
    fixed.extend_from_slice(&((zones.len() / ZONE_BYTES) as u32).to_le_bytes());
    fixed.extend_from_slice(&0u32.to_le_bytes());
    container::encode(MAGIC, &fixed, &[&strings_bytes, &records, &zones])
}

/// Decodes `bodies/bodies.dtb`.
pub fn decode_bodies(path: &Path, b: &[u8]) -> Result<Vec<Body>> {
    let p = container::parse(path, b, MAGIC, FIXED_BYTES, |_| SECTIONS)?;
    let n = u32_at(p.fixed, 0) as usize;
    let s = u32_at(p.fixed, 4) as usize;
    let nz = u32_at(p.fixed, 8) as usize;
    let [strings_b, records, zones] = [p.sections[0], p.sections[1], p.sections[2]];
    let corrupt = |m: String| FormatError::corrupt(path, m);
    if records.len() != n * BODY_BYTES || zones.len() != nz * ZONE_BYTES {
        return Err(corrupt("body or zone section has the wrong length".into()));
    }
    let strings = container::decode_strings(path, strings_b, s)?;
    let opt_str = |v: u32, what: &str| -> Result<Option<String>> {
        match v {
            NONE_U32 => Ok(None),
            v => Ok(Some(string_at(path, &strings, v, what)?.to_owned())),
        }
    };
    let mut out = Vec::with_capacity(n);
    for r in 0..n {
        let rec = &records[r * BODY_BYTES..(r + 1) * BODY_BYTES];
        let id = u32_at(rec, 0);
        let shape = Shape::from_code(rec[8])
            .ok_or_else(|| corrupt(format!("body {id}: shape {}", rec[8])))?;
        let (zf, zn) = (u32_at(rec, 88) as usize, u32_at(rec, 92) as usize);
        if (zf + zn) * ZONE_BYTES > zones.len() {
            return Err(corrupt(format!("body {id}: zones past the section")));
        }
        let mut zs = Vec::with_capacity(zn);
        for i in zf..zf + zn {
            let e = &zones[i * ZONE_BYTES..(i + 1) * ZONE_BYTES];
            zs.push(Zone {
                name: string_at(path, &strings, u32_at(e, 0), "zone name")?.to_owned(),
                material: opt_str(u32_at(e, 4), "zone material")?,
                rock_class: opt_str(u32_at(e, 8), "zone rock class")?,
                d: [f32_at(e, 12), f32_at(e, 16)],
                v: [f32_at(e, 20), f32_at(e, 24)],
                grade: f32_at(e, 28),
                density: f32_at(e, 32),
                raw: f32_at(e, 36),
            });
        }
        let i3 = |at: usize| [i32_at(rec, at), i32_at(rec, at + 4), i32_at(rec, at + 8)];
        let site = u32_at(rec, 4);
        let body = Body {
            id,
            site: (site != NONE_U32).then_some(site),
            shape,
            role: string_at(path, &strings, u32_at(rec, 12), "role")?.to_owned(),
            center: [f64_at(rec, 16), f64_at(rec, 24), f64_at(rec, 32)],
            axes: [f32_at(rec, 40), f32_at(rec, 44), f32_at(rec, 48)],
            yaw: f32_at(rec, 52),
            dip: f32_at(rec, 56),
            clip_top_y: i32_at(rec, 60),
            aabb: [i3(64), i3(76)],
            rock_class: opt_str(u32_at(rec, 96), "body rock class")?,
            zones: zs,
            age_ma: Some(f32_at(rec, 100)).filter(|a| a.is_finite()),
        };
        body.validate()?;
        out.push(body);
    }
    Ok(out)
}

/// Writes `bodies/bodies.dtb` under `world` (atomically); returns (bytes, CRC32).
pub fn write_bodies(world: &Path, bodies: &[Body]) -> Result<(u64, u32)> {
    let bytes = encode_bodies(bodies)?;
    let crc = write_atomic(&world.join(BODIES_PATH), &bytes)?;
    Ok((bytes.len() as u64, crc))
}

/// Reads `bodies/bodies.dtb`; `None` when the folder has none.
pub fn read_bodies(world: &Path) -> Result<Option<Vec<Body>>> {
    let path = world.join(BODIES_PATH);
    match std::fs::read(&path) {
        Ok(b) => Ok(Some(decode_bodies(&path, &b)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(FormatError::io(&path, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn porphyry(id: u32) -> Body {
        let mut b = Body::new(
            id,
            Some(3),
            Shape::StockworkShell,
            "ore",
            [100.25, -40.0, -7.5],
            [10.0, 20.0, 8.0],
            0.3,
            0.0,
            NO_CLIP,
        );
        b.zones = vec![
            Zone {
                name: "potassic_core".into(),
                material: Some("c:copper".into()),
                rock_class: None,
                d: [0.0, 0.45],
                v: [0.0, 1.0],
                grade: 0.006,
                density: 0.5,
                raw: 0.04,
            },
            Zone {
                name: "propylitic_halo".into(),
                material: None,
                rock_class: Some("greenstone".into()),
                d: [0.7, 1.0],
                v: [0.0, 1.0],
                grade: 0.0,
                density: 0.0,
                raw: 0.0,
            },
        ];
        b
    }

    #[test]
    fn round_trip() {
        let mut sheet = Body::new(
            7,
            None,
            Shape::Sheet,
            "seam",
            [-2000.0, 12.0, 300.0],
            [40.0, 1.5, 25.0],
            1.0,
            0.4,
            30,
        );
        sheet.rock_class = Some("coal".into());
        sheet.age_ma = Some(412.5);
        let bodies = vec![sheet, porphyry(2)];
        let bytes = encode_bodies(&bodies).unwrap();
        let back = decode_bodies(Path::new("b.dtb"), &bytes).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back[0], bodies[1], "sorted by id");
        assert_eq!(back[1], bodies[0]);
        let dup = vec![porphyry(1), porphyry(1)];
        assert!(encode_bodies(&dup).is_err());
        let mut corrupt = bytes.clone();
        let last = corrupt.len() - 1;
        corrupt[last] ^= 1;
        assert!(decode_bodies(Path::new("b.dtb"), &corrupt).is_err());
    }

    /// Every block the metric puts inside lies in the AABB; the centre is inside; zones follow d.
    #[test]
    fn locate_agrees_with_aabb_and_zones() {
        let c = 4096;
        for (shape, dip) in [
            (Shape::StockworkShell, 0.0),
            (Shape::Dyke, 1.1),
            (Shape::Seam, 0.2),
        ] {
            let mut b = porphyry(1);
            b.shape = shape;
            b.dip = dip;
            b.aabb = b.computed_aabb();
            let [lo, hi] = b.aabb;
            let mut inside = 0;
            for x in lo[0] - 3..=hi[0] + 3 {
                for y in lo[1] - 3..=hi[1] + 3 {
                    for z in lo[2] - 3..=hi[2] + 3 {
                        if let Some(at) = b.locate(c, x, y, z) {
                            inside += 1;
                            assert!(
                                (lo[0]..=hi[0]).contains(&x)
                                    && (lo[1]..=hi[1]).contains(&y)
                                    && (lo[2]..=hi[2]).contains(&z),
                                "{shape:?} ({x},{y},{z}) outside the AABB"
                            );
                            assert!(at.d <= 1.0 && (0.0..=1.0).contains(&at.v));
                            let core = b.zones_at(at).any(|z| z.name == "potassic_core");
                            assert_eq!(core, at.d < 0.45);
                        }
                    }
                }
            }
            let ratio = f64::from(inside) / b.volume_blocks();
            assert!(
                (0.8..1.25).contains(&ratio),
                "{shape:?} volume ratio {ratio}"
            );
        }
        let b = porphyry(1);
        let at = b.locate(c, 100, -40, -8).unwrap();
        assert!(at.d < 0.1);
        assert!((at.v - 0.5).abs() < 0.05);
    }

    #[test]
    fn locate_wraps_the_seam_and_clips() {
        let c = 1024;
        let mut b = porphyry(1);
        b.center = [-511.0, 0.0, 0.0];
        b.aabb = b.computed_aabb();
        assert!(b.locate(c, 509, 0, 0).is_some(), "across the seam");
        assert!(b.aabb[0][0] < -512, "the AABB leaves the canonical span");
        b.clip_top_y = 5;
        b.aabb = b.computed_aabb();
        assert!(b.locate(c, -511, 6, 0).is_none());
        assert!(b.locate(c, -511, 5, 0).is_some());
        assert_eq!(b.aabb[1][1], 5);
    }

    #[test]
    fn write_and_read() {
        let dir = std::env::temp_dir().join(format!("dt-bodies-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(read_bodies(&dir).unwrap().is_none());
        write_bodies(&dir, &[porphyry(0)]).unwrap();
        assert_eq!(read_bodies(&dir).unwrap().unwrap(), vec![porphyry(0)]);
        write_bodies(&dir, &[]).unwrap();
        assert!(read_bodies(&dir).unwrap().unwrap().is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

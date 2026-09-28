// SPDX-License-Identifier: LGPL-3.0-or-later
//! The site registry (plan §5.6): every deposit, reservoir, placer, supergene blanket and fantasy
//! site of a world, with the event that formed it. On disk: `sites/sites.dtsr` (records) and
//! `sites/index.dtsi` (a 512-block grid → site ids), both sectioned files (`FORMAT.md`).

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::container::{self, StringTable, f32_at, f64_at, i32_at, string_at, u32_at, u64_at};
use crate::error::{FormatError, Result};
use crate::events::EventId;
use crate::fsutil::write_atomic;

/// Magic of the site records.
pub const MAGIC: [u8; 4] = *b"DTSR";
/// Magic of the site index.
pub const INDEX_MAGIC: [u8; 4] = *b"DTSI";
/// Records file, relative to the world-data folder.
pub const SITES_PATH: &str = "sites/sites.dtsr";
/// Index file, relative to the world-data folder.
pub const INDEX_PATH: &str = "sites/index.dtsi";
/// Edge of an index cell in blocks (plan §5.6).
pub const INDEX_CELL_BLOCKS: u32 = 512;
/// Bytes per site record.
pub const RECORD_BYTES: usize = 104;
/// Bytes per product entry.
pub const PRODUCT_BYTES: usize = 24;
/// "None" for u32 references (body, parent, strings).
pub const NONE_U32: u32 = u32::MAX;
/// "None" for `cdg_charge_mb`.
pub const NONE_U64: u64 = u64::MAX;

const FIXED_BYTES: usize = 16;
const SECTIONS: usize = 3;
const INDEX_FIXED_BYTES: usize = 32;
const INDEX_SECTIONS: usize = 2;

/// Stable for the life of the world; `u32::MAX` is reserved.
pub type SiteId = u32;

/// What a site is (plan §5.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiteKind {
    Deposit,
    Reservoir,
    Placer,
    Supergene,
    Fantasy,
}

impl SiteKind {
    pub fn code(self) -> u8 {
        match self {
            SiteKind::Deposit => 0,
            SiteKind::Reservoir => 1,
            SiteKind::Placer => 2,
            SiteKind::Supergene => 3,
            SiteKind::Fantasy => 4,
        }
    }

    pub fn from_code(c: u8) -> Option<SiteKind> {
        Some(match c {
            0 => SiteKind::Deposit,
            1 => SiteKind::Reservoir,
            2 => SiteKind::Placer,
            3 => SiteKind::Supergene,
            4 => SiteKind::Fantasy,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            SiteKind::Deposit => "deposit",
            SiteKind::Reservoir => "reservoir",
            SiteKind::Placer => "placer",
            SiteKind::Supergene => "supergene",
            SiteKind::Fantasy => "fantasy",
        }
    }
}

/// One product of a site.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Product {
    /// A namespaced material id (`minecraft:copper`, `deeptime:chalcopyrite`).
    pub material: String,
    /// Mass fraction (0–1) for ores; kind-defined for reservoirs.
    pub grade: f32,
    pub tonnage_t: f64,
    /// `primary`, `coproduct`, `byproduct`, …
    pub role: String,
}

/// How the site is mined (plan §5.6; the COE/CDG hooks of P1-16 read it).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Extraction {
    pub finite: bool,
    pub amount_base: f64,
    /// COE vein id, if the site feeds one.
    pub coe_vein: Option<String>,
    /// CDG charge in millibuckets, if the site feeds a Create Diesel Generators well.
    pub cdg_charge_mb: Option<u64>,
}

impl Default for Extraction {
    fn default() -> Self {
        Extraction {
            finite: true,
            amount_base: 0.0,
            coe_vein: None,
            cdg_charge_mb: None,
        }
    }
}

/// One site (plan §5.6).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Site {
    pub id: SiteId,
    pub kind: SiteKind,
    /// `deeptime:porphyry_cu_mo_au`, …
    pub deposit_type: String,
    /// Block space, canonical span (x ∈ [−C/2, C/2)).
    pub center: [i32; 3],
    /// Degrees, north and east positive.
    pub lat_lon: [f32; 2],
    /// Inclusive block-space bounds of every body of the site: [min, max]. x may leave the
    /// canonical span near the seam; readers wrap it.
    pub aabb: [[i32; 3]; 2],
    /// The geometry in `bodies/` (§5.7.4); `None` until bodies exist.
    pub body: Option<u32>,
    pub products: Vec<Product>,
    pub forming_event: EventId,
    /// Placer → its eroded source, blanket → its porphyry.
    pub parent: Option<SiteId>,
    pub extraction: Extraction,
}

impl Site {
    /// Checks what a reader relies on.
    pub fn validate(&self) -> Result<()> {
        let bad = |m: String| Err(FormatError::Invalid(format!("site {}: {m}", self.id)));
        if self.id == NONE_U32 {
            return bad("id u32::MAX is reserved".into());
        }
        if !crate::events::is_namespaced(&self.deposit_type) {
            return bad(format!(
                "deposit type {:?} is not namespaced",
                self.deposit_type
            ));
        }
        let [lo, hi] = self.aabb;
        if (0..3).any(|k| lo[k] > hi[k]) {
            return bad(format!("empty AABB {:?}", self.aabb));
        }
        if self.parent == Some(self.id) || self.parent == Some(NONE_U32) {
            return bad("bad parent".into());
        }
        if self.body == Some(NONE_U32) {
            return bad("body u32::MAX is reserved".into());
        }
        if self.extraction.cdg_charge_mb == Some(NONE_U64) {
            return bad("cdg_charge_mb u64::MAX is reserved".into());
        }
        Ok(())
    }

    /// True when block (x, z) lies in the AABB's footprint on a planet of circumference `c`.
    pub fn footprint_contains(&self, c: u32, x: i32, z: i32) -> bool {
        let [lo, hi] = self.aabb;
        if z < lo[2] || z > hi[2] {
            return false;
        }
        let c = i64::from(c);
        let span = i64::from(hi[0]) - i64::from(lo[0]);
        if span >= c - 1 {
            return true;
        }
        (i64::from(x) - i64::from(lo[0])).rem_euclid(c) <= span
    }

    /// The JSON form the viewer reads (`FORMAT.md`, "sites").
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "kind": self.kind.name(),
            "deposit_type": self.deposit_type,
            "center": self.center,
            "lat_lon": self.lat_lon,
            "aabb": self.aabb,
            "body": self.body,
            "products": self.products.iter().map(|p| json!({
                "material": p.material, "grade": p.grade, "tonnage_t": p.tonnage_t, "role": p.role,
            })).collect::<Vec<_>>(),
            "forming_event": self.forming_event.0,
            "parent": self.parent,
            "extraction": {
                "finite": self.extraction.finite,
                "amount_base": self.extraction.amount_base,
                "coe_vein": self.extraction.coe_vein,
                "cdg_charge_mb": self.extraction.cdg_charge_mb,
            },
        })
    }
}

/// Encodes `sites/sites.dtsr`: sites sorted by id (ids unique).
pub fn encode_sites(sites: &[Site]) -> Result<Vec<u8>> {
    let mut sorted: Vec<&Site> = sites.iter().collect();
    sorted.sort_by_key(|s| s.id);
    for w in sorted.windows(2) {
        if w[0].id == w[1].id {
            return Err(FormatError::Invalid(format!(
                "site {} appears twice",
                w[0].id
            )));
        }
    }
    let mut strings = StringTable::default();
    let mut records = Vec::with_capacity(sorted.len() * RECORD_BYTES);
    let mut products = Vec::new();
    for s in &sorted {
        s.validate()?;
        let ty = strings.intern(&s.deposit_type);
        let products_first = (products.len() / PRODUCT_BYTES) as u32;
        for p in &s.products {
            products.extend_from_slice(&strings.intern(&p.material).to_le_bytes());
            products.extend_from_slice(&p.grade.to_le_bytes());
            products.extend_from_slice(&p.tonnage_t.to_le_bytes());
            products.extend_from_slice(&strings.intern(&p.role).to_le_bytes());
            products.extend_from_slice(&0u32.to_le_bytes());
        }
        let vein = match &s.extraction.coe_vein {
            Some(v) => strings.intern(v),
            None => NONE_U32,
        };
        records.extend_from_slice(&s.id.to_le_bytes());
        records.push(s.kind.code());
        records.push(u8::from(s.extraction.finite));
        records.extend_from_slice(&0u16.to_le_bytes());
        records.extend_from_slice(&ty.to_le_bytes());
        records.extend_from_slice(&s.body.unwrap_or(NONE_U32).to_le_bytes());
        for v in s.center {
            records.extend_from_slice(&v.to_le_bytes());
        }
        for v in s.lat_lon {
            records.extend_from_slice(&v.to_le_bytes());
        }
        for corner in s.aabb {
            for v in corner {
                records.extend_from_slice(&v.to_le_bytes());
            }
        }
        records.extend_from_slice(&s.parent.unwrap_or(NONE_U32).to_le_bytes());
        records.extend_from_slice(&s.forming_event.0.to_le_bytes());
        records.extend_from_slice(&products_first.to_le_bytes());
        records.extend_from_slice(&(s.products.len() as u32).to_le_bytes());
        records.extend_from_slice(&s.extraction.amount_base.to_le_bytes());
        records.extend_from_slice(&vein.to_le_bytes());
        records.extend_from_slice(&0u32.to_le_bytes());
        records.extend_from_slice(&s.extraction.cdg_charge_mb.unwrap_or(NONE_U64).to_le_bytes());
    }
    let strings_bytes = strings.encode();
    let mut fixed = Vec::with_capacity(FIXED_BYTES);
    fixed.extend_from_slice(&(sorted.len() as u32).to_le_bytes());
    fixed.extend_from_slice(&(strings.len() as u32).to_le_bytes());
    fixed.extend_from_slice(&((products.len() / PRODUCT_BYTES) as u32).to_le_bytes());
    fixed.extend_from_slice(&0u32.to_le_bytes());
    container::encode(MAGIC, &fixed, &[&strings_bytes, &records, &products])
}

/// Decodes `sites/sites.dtsr`.
pub fn decode_sites(path: &Path, b: &[u8]) -> Result<Vec<Site>> {
    let p = container::parse(path, b, MAGIC, FIXED_BYTES, |_| SECTIONS)?;
    let n = u32_at(p.fixed, 0) as usize;
    let s = u32_at(p.fixed, 4) as usize;
    let np = u32_at(p.fixed, 8) as usize;
    let [strings_b, records, products] = [p.sections[0], p.sections[1], p.sections[2]];
    let corrupt = |m: String| FormatError::corrupt(path, m);
    if records.len() != n * RECORD_BYTES || products.len() != np * PRODUCT_BYTES {
        return Err(corrupt(
            "record or product section has the wrong length".into(),
        ));
    }
    let strings = container::decode_strings(path, strings_b, s)?;
    let opt = |v: u32| (v != NONE_U32).then_some(v);
    let mut out = Vec::with_capacity(n);
    for r in 0..n {
        let rec = &records[r * RECORD_BYTES..(r + 1) * RECORD_BYTES];
        let id = u32_at(rec, 0);
        let kind = SiteKind::from_code(rec[4])
            .ok_or_else(|| corrupt(format!("site {id}: kind {}", rec[4])))?;
        let (pf, pn) = (u32_at(rec, 72) as usize, u32_at(rec, 76) as usize);
        if (pf + pn) * PRODUCT_BYTES > products.len() {
            return Err(corrupt(format!("site {id}: products past the section")));
        }
        let mut prods = Vec::with_capacity(pn);
        for i in pf..pf + pn {
            let e = &products[i * PRODUCT_BYTES..(i + 1) * PRODUCT_BYTES];
            prods.push(Product {
                material: string_at(path, &strings, u32_at(e, 0), "material")?.to_owned(),
                grade: f32_at(e, 4),
                tonnage_t: f64_at(e, 8),
                role: string_at(path, &strings, u32_at(e, 16), "role")?.to_owned(),
            });
        }
        let i3 = |at: usize| [i32_at(rec, at), i32_at(rec, at + 4), i32_at(rec, at + 8)];
        let vein = u32_at(rec, 88);
        let charge = u64_at(rec, 96);
        let site = Site {
            id,
            kind,
            deposit_type: string_at(path, &strings, u32_at(rec, 8), "deposit type")?.to_owned(),
            body: opt(u32_at(rec, 12)),
            center: i3(16),
            lat_lon: [f32_at(rec, 28), f32_at(rec, 32)],
            aabb: [i3(36), i3(48)],
            parent: opt(u32_at(rec, 60)),
            forming_event: EventId(u64_at(rec, 64)),
            products: prods,
            extraction: Extraction {
                finite: rec[5] & 1 != 0,
                amount_base: f64_at(rec, 80),
                coe_vein: match vein {
                    NONE_U32 => None,
                    v => Some(string_at(path, &strings, v, "coe vein")?.to_owned()),
                },
                cdg_charge_mb: (charge != NONE_U64).then_some(charge),
            },
        };
        if let Some(prev) = out.last().map(|s: &Site| s.id)
            && prev >= id
        {
            return Err(corrupt(format!("site {id} out of order")));
        }
        out.push(site);
    }
    Ok(out)
}

/// The site index (`sites/index.dtsi`): a grid of 512 × 512-block cells over the planet, each
/// listing (ascending) the sites whose AABB footprint touches it. Columns wrap in x.
#[derive(Clone, Debug, PartialEq)]
pub struct SiteIndex {
    pub circumference: u32,
    pub cell_blocks: u32,
    pub cols: u32,
    pub rows: u32,
    /// Block x of column 0 (−C/2) and z of row 0 (−C/4).
    pub origin_x: i32,
    pub origin_z: i32,
    /// CSR: cell c's ids are `ids[offsets[c]..offsets[c + 1]]`, cell c = row · cols + col.
    offsets: Vec<u32>,
    ids: Vec<SiteId>,
}

impl SiteIndex {
    /// Builds the index of `sites` on a planet of circumference `c`.
    pub fn build(c: u32, sites: &[Site]) -> Result<SiteIndex> {
        if c == 0 || !c.is_multiple_of(4) {
            return Err(FormatError::Invalid(format!(
                "circumference {c} is not a positive multiple of 4"
            )));
        }
        let cb = INDEX_CELL_BLOCKS;
        let cols = c.div_ceil(cb);
        let rows = (c / 2).div_ceil(cb);
        let origin_x = -((c / 2) as i32);
        let origin_z = -((c / 4) as i32);
        let mut cells: Vec<Vec<SiteId>> = vec![Vec::new(); (cols * rows) as usize];
        let col_of = |x: i32| (i64::from(x) - i64::from(origin_x)).div_euclid(i64::from(cb));
        let row_of = |z: i32| {
            ((i64::from(z) - i64::from(origin_z)).div_euclid(i64::from(cb)))
                .clamp(0, i64::from(rows) - 1) as u32
        };
        for s in sites {
            s.validate()?;
            let [lo, hi] = s.aabb;
            let (c0, c1) = (col_of(lo[0]), col_of(hi[0]));
            let span = (c1 - c0 + 1).min(i64::from(cols));
            for r in row_of(lo[2])..=row_of(hi[2]) {
                for k in 0..span {
                    let col = (c0 + k).rem_euclid(i64::from(cols)) as u32;
                    cells[(r * cols + col) as usize].push(s.id);
                }
            }
        }
        let mut offsets = Vec::with_capacity(cells.len() + 1);
        let mut ids = Vec::new();
        offsets.push(0);
        for mut cell in cells {
            cell.sort_unstable();
            cell.dedup();
            ids.extend_from_slice(&cell);
            offsets.push(ids.len() as u32);
        }
        Ok(SiteIndex {
            circumference: c,
            cell_blocks: cb,
            cols,
            rows,
            origin_x,
            origin_z,
            offsets,
            ids,
        })
    }

    /// Sites whose footprint touches the index cell holding block (x, z); x wraps, z clamps.
    pub fn at_block(&self, x: i32, z: i32) -> &[SiteId] {
        let cb = i64::from(self.cell_blocks);
        let col = (i64::from(x) - i64::from(self.origin_x))
            .div_euclid(cb)
            .rem_euclid(i64::from(self.cols)) as u32;
        let row = (i64::from(z) - i64::from(self.origin_z))
            .div_euclid(cb)
            .clamp(0, i64::from(self.rows) - 1) as u32;
        self.cell(col, row)
    }

    /// The ids of cell (col, row).
    pub fn cell(&self, col: u32, row: u32) -> &[SiteId] {
        let c = (row * self.cols + col) as usize;
        &self.ids[self.offsets[c] as usize..self.offsets[c + 1] as usize]
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut fixed = Vec::with_capacity(INDEX_FIXED_BYTES);
        for v in [self.circumference, self.cell_blocks, self.cols, self.rows] {
            fixed.extend_from_slice(&v.to_le_bytes());
        }
        fixed.extend_from_slice(&self.origin_x.to_le_bytes());
        fixed.extend_from_slice(&self.origin_z.to_le_bytes());
        fixed.extend_from_slice(&(self.ids.len() as u32).to_le_bytes());
        fixed.extend_from_slice(&0u32.to_le_bytes());
        let offsets: Vec<u8> = self.offsets.iter().flat_map(|v| v.to_le_bytes()).collect();
        let ids: Vec<u8> = self.ids.iter().flat_map(|v| v.to_le_bytes()).collect();
        container::encode(INDEX_MAGIC, &fixed, &[&offsets, &ids])
    }

    pub fn decode(path: &Path, b: &[u8]) -> Result<SiteIndex> {
        let p = container::parse(path, b, INDEX_MAGIC, INDEX_FIXED_BYTES, |_| INDEX_SECTIONS)?;
        let f = p.fixed;
        let (c, cb, cols, rows) = (u32_at(f, 0), u32_at(f, 4), u32_at(f, 8), u32_at(f, 12));
        let entries = u32_at(f, 24) as usize;
        let cells = cols as usize * rows as usize;
        let corrupt = |m: &str| FormatError::corrupt(path, m);
        if cb == 0 || cols == 0 || rows == 0 {
            return Err(corrupt("empty index grid"));
        }
        if p.sections[0].len() != (cells + 1) * 4 || p.sections[1].len() != entries * 4 {
            return Err(corrupt("index sections have the wrong length"));
        }
        let offsets: Vec<u32> = (0..=cells).map(|i| u32_at(p.sections[0], i * 4)).collect();
        let ids: Vec<u32> = (0..entries).map(|i| u32_at(p.sections[1], i * 4)).collect();
        if offsets[0] != 0
            || offsets[cells] as usize != entries
            || offsets.windows(2).any(|w| w[0] > w[1])
        {
            return Err(corrupt("index offsets are not monotonic"));
        }
        Ok(SiteIndex {
            circumference: c,
            cell_blocks: cb,
            cols,
            rows,
            origin_x: i32_at(f, 16),
            origin_z: i32_at(f, 20),
            offsets,
            ids,
        })
    }
}

/// Sizes and CRC32s of the two files [`write_sites`] wrote, for the manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SitesWritten {
    pub sites_bytes: u64,
    pub sites_crc32: u32,
    pub index_bytes: u64,
    pub index_crc32: u32,
}

/// Writes `sites/sites.dtsr` and `sites/index.dtsi` under `world` (atomically, each).
pub fn write_sites(world: &Path, c: u32, sites: &[Site]) -> Result<SitesWritten> {
    let records = encode_sites(sites)?;
    let index = SiteIndex::build(c, sites)?.encode()?;
    let sites_crc32 = write_atomic(&world.join(SITES_PATH), &records)?;
    let index_crc32 = write_atomic(&world.join(INDEX_PATH), &index)?;
    Ok(SitesWritten {
        sites_bytes: records.len() as u64,
        sites_crc32,
        index_bytes: index.len() as u64,
        index_crc32,
    })
}

/// Reads both files; `None` when the folder has no `sites/sites.dtsr`.
pub fn read_sites(world: &Path) -> Result<Option<(Vec<Site>, SiteIndex)>> {
    let sp = world.join(SITES_PATH);
    let records = match std::fs::read(&sp) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(FormatError::io(&sp, e)),
    };
    let ip = world.join(INDEX_PATH);
    let index = std::fs::read(&ip).map_err(|e| FormatError::io(&ip, e))?;
    Ok(Some((
        decode_sites(&sp, &records)?,
        SiteIndex::decode(&ip, &index)?,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site(id: u32, x: i32, z: i32, half: i32) -> Site {
        Site {
            id,
            kind: SiteKind::from_code((id % 5) as u8).unwrap(),
            deposit_type: if id.is_multiple_of(2) {
                "deeptime:porphyry_cu_mo_au".into()
            } else {
                "deeptime:placer_au".into()
            },
            center: [x, -40, z],
            lat_lon: [z as f32 * -0.02, x as f32 * 0.02],
            aabb: [[x - half, -80, z - half], [x + half, 10, z + half]],
            body: id.is_multiple_of(3).then_some(id * 10),
            products: (0..id % 3)
                .map(|k| Product {
                    material: format!("minecraft:ore_{k}"),
                    grade: 0.01 * (k + 1) as f32,
                    tonnage_t: 1.0e6 * f64::from(id),
                    role: if k == 0 { "primary" } else { "byproduct" }.into(),
                })
                .collect(),
            forming_event: EventId::new(1, u64::from(id) * 7),
            parent: (id % 4 == 1).then_some(id + 100),
            extraction: Extraction {
                finite: id.is_multiple_of(2),
                amount_base: 12.5 * f64::from(id),
                coe_vein: (id % 2 == 1).then(|| "coe:copper_vein".to_string()),
                cdg_charge_mb: (id % 3 == 2).then_some(u64::from(id) * 1000),
            },
        }
    }

    #[test]
    fn records_round_trip() {
        let sites: Vec<Site> = [7u32, 1, 3, 2, 12, 40]
            .iter()
            .map(|&i| site(i, i as i32 * 100 - 2000, i as i32 * 37 - 900, 20))
            .collect();
        let bytes = encode_sites(&sites).unwrap();
        let back = decode_sites(Path::new("s.dtsr"), &bytes).unwrap();
        let mut expect = sites.clone();
        expect.sort_by_key(|s| s.id);
        assert_eq!(back, expect);
        assert!(
            decode_sites(Path::new("s"), &encode_sites(&[]).unwrap())
                .unwrap()
                .is_empty()
        );
        let mut dup = sites.clone();
        dup.push(sites[0].clone());
        assert!(encode_sites(&dup).is_err());
        let mut bad = sites[0].clone();
        bad.deposit_type = "nonamespace".into();
        assert!(encode_sites(&[bad]).is_err());
        let mut flipped = bytes.clone();
        let n = flipped.len();
        flipped[n - 3] ^= 1;
        assert!(decode_sites(Path::new("s"), &flipped).is_err());
    }

    #[test]
    fn index_finds_sites_and_wraps_the_seam() {
        let c = 4096; // 8 × 4 cells of 512 blocks
        let near_seam = Site {
            aabb: [[-2100, -10, -30], [-2030, 5, 30]], // x from −2100 (wrapped: 1996) to −2030
            ..site(5, -2048, 0, 1)
        };
        let big = Site {
            aabb: [[-3000, 0, -1024], [3000, 0, 1023]],
            ..site(6, 0, 0, 1)
        };
        let small = site(9, 700, -600, 10);
        let sites = vec![near_seam, big, small];
        let idx = SiteIndex::build(c, &sites).unwrap();
        assert_eq!(
            (idx.cols, idx.rows, idx.origin_x, idx.origin_z),
            (8, 4, -2048, -1024)
        );
        assert_eq!(idx.at_block(-2040, 0), &[5, 6]);
        assert_eq!(idx.at_block(2040, 0), &[5, 6], "the seam wraps");
        assert_eq!(idx.at_block(700, -600), &[6, 9]);
        assert_eq!(idx.at_block(700 + 4096, -600), &[6, 9]);
        assert_eq!(idx.at_block(-1000, 1000), &[6]);
        // Every cell of the big site's span holds it.
        for col in 0..8 {
            for row in 0..4 {
                assert!(idx.cell(col, row).contains(&6));
            }
        }
        let back = SiteIndex::decode(Path::new("i"), &idx.encode().unwrap()).unwrap();
        assert_eq!(back, idx);
        assert!(sites[0].footprint_contains(c, 2040, 0));
        assert!(!sites[0].footprint_contains(c, 0, 0));
    }

    #[test]
    fn write_and_read_folder() {
        let dir = std::env::temp_dir().join(format!("dt-sites-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(read_sites(&dir).unwrap().is_none());
        let sites = vec![site(1, 0, 0, 5), site(2, 1000, 500, 5)];
        let w = write_sites(&dir, 16384, &sites).unwrap();
        assert_eq!(
            w.sites_bytes,
            std::fs::metadata(dir.join(SITES_PATH)).unwrap().len()
        );
        let (back, idx) = read_sites(&dir).unwrap().unwrap();
        assert_eq!(back, sites);
        assert_eq!(idx.at_block(1000, 500), &[2]);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! Java Edition 1.21 chunk NBT: `Status`, `yPos`, `sections` (each with `Y` and a
//! `block_states` palette plus packed longs), `Heightmaps` and `structures`.
//!
//! Packed arrays (sections since 1.16, heightmaps): entries are `bits` wide, `64 / bits` of them
//! per long starting at the low bits, and never span two longs. Block states use
//! `bits = max(4, ⌈log₂ palette⌉)` and index `y·256 + z·16 + x`; a one-entry palette has no data.
//! Heightmaps hold 256 entries (`z·16 + x`) of `bits = ⌈log₂(height + 1)⌉`, each the first free y
//! above the column's top block minus the world's bottom (0: no block).

use crate::bail;
use crate::blocks::{self, Registry};
use crate::error::Result;
use crate::nbt::Tag;

/// One 16³ section: palette ids and packed indices.
pub struct Section {
    palette: Vec<u32>,
    data: Vec<u64>,
    bits: u32,
    /// OR of the palette's class flags ([`blocks`]), to skip sections quickly.
    pub flags: u16,
}

impl Section {
    /// The state id at local (x, y, z).
    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> u32 {
        if self.bits == 0 {
            return self.palette[0];
        }
        let i = (y << 8) | (z << 4) | x;
        let per = (64 / self.bits) as usize;
        let word = self.data[i / per];
        let shift = (i % per) as u32 * self.bits;
        let idx = ((word >> shift) & ((1u64 << self.bits) - 1)) as usize;
        // An index past the palette is corruption; read it as air rather than fail the column.
        self.palette.get(idx).copied().unwrap_or(0)
    }
}

/// One section's biomes: 4 × 4 × 4 cells of 4³ blocks (index `y·16 + z·4 + x`), a palette of
/// biome ids and `⌈log₂ palette⌉`-bit indices (no minimum width; a one-entry palette has no data).
pub struct BiomeSection {
    pub palette: Vec<String>,
    idx: Vec<u8>,
}

impl BiomeSection {
    /// The biome id at local block (x, y, z) of the section.
    pub fn get(&self, x: usize, y: usize, z: usize) -> &str {
        let i = ((y >> 2) << 4) | ((z >> 2) << 2) | (x >> 2);
        let k = self.idx.get(i).copied().unwrap_or(0) as usize;
        self.palette.get(k).map_or("?", String::as_str)
    }
}

/// One structure piece's bounding box (inclusive block coordinates) and its structure's id.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub structure: String,
    pub min: [i32; 3],
    pub max: [i32; 3],
}

/// A parsed chunk.
pub struct Chunk {
    pub cx: i32,
    pub cz: i32,
    /// `Status` without the `minecraft:` prefix (`full`, `features`, `structure_starts`, …).
    pub status: String,
    /// The lowest section's Y (block y = 16 · this).
    pub min_section: i32,
    /// Sections from `min_section` up; None where the file has no block states.
    pub sections: Vec<Option<Section>>,
    /// Heightmaps by name (`WORLD_SURFACE`, `OCEAN_FLOOR`, `MOTION_BLOCKING`, …), 256 values each.
    pub heightmaps: Vec<(String, Vec<u16>)>,
    /// Pieces of the structures that start in this chunk.
    pub pieces: Vec<Piece>,
    /// Biomes by section Y.
    pub biomes: Vec<(i32, BiomeSection)>,
}

/// ⌈log₂ n⌉ for n ≥ 1.
pub fn ceil_log2(n: usize) -> u32 {
    if n <= 1 {
        0
    } else {
        usize::BITS - (n - 1).leading_zeros()
    }
}

/// Unpacks `count` entries of `bits` from longs that never span entries across longs.
pub fn unpack(data: &[i64], bits: u32, count: usize) -> Option<Vec<u32>> {
    if bits == 0 || bits > 32 {
        return None;
    }
    let per = (64 / bits) as usize;
    if data.len() < count.div_ceil(per) {
        return None;
    }
    let mask = (1u64 << bits) - 1;
    Some(
        (0..count)
            .map(|i| ((data[i / per] as u64 >> ((i % per) as u32 * bits)) & mask) as u32)
            .collect(),
    )
}

/// Packs values as [`unpack`] reads them (tests and fixtures).
pub fn pack(values: &[u32], bits: u32) -> Vec<i64> {
    let per = (64 / bits) as usize;
    let mut out = vec![0u64; values.len().div_ceil(per)];
    for (i, &v) in values.iter().enumerate() {
        out[i / per] |= u64::from(v) << ((i % per) as u32 * bits);
    }
    out.into_iter().map(|w| w as i64).collect()
}

impl Chunk {
    /// Reads a chunk's root compound, interning its block states into `reg`.
    pub fn parse(root: &Tag, reg: &mut Registry) -> Result<Chunk> {
        let int = |k: &str| root.get(k).and_then(Tag::as_i64);
        let (Some(cx), Some(cz)) = (int("xPos"), int("zPos")) else {
            bail!("chunk NBT has no xPos/zPos (not a 1.18+ chunk?)");
        };
        let status = root
            .get("Status")
            .and_then(Tag::as_str)
            .unwrap_or("")
            .trim_start_matches("minecraft:")
            .to_owned();
        let sections_tag = root.get("sections").and_then(Tag::as_list).unwrap_or(&[]);
        let mut parsed: Vec<(i32, Section)> = Vec::new();
        let mut biomes: Vec<(i32, BiomeSection)> = Vec::new();
        for s in sections_tag {
            let Some(y) = s.get("Y").and_then(Tag::as_i64) else {
                continue;
            };
            if let Some(b) = s.get("biomes") {
                let palette: Vec<String> = b
                    .get("palette")
                    .and_then(Tag::as_list)
                    .unwrap_or(&[])
                    .iter()
                    .map(|t| t.as_str().unwrap_or("?").to_owned())
                    .collect();
                if !palette.is_empty() {
                    let idx = if palette.len() == 1 {
                        vec![0u8; 64]
                    } else {
                        let bits = ceil_log2(palette.len());
                        let data = b.get("data").and_then(Tag::as_long_array).unwrap_or(&[]);
                        unpack(data, bits, 64)
                            .map(|v| v.into_iter().map(|k| k as u8).collect())
                            .unwrap_or_else(|| vec![0u8; 64])
                    };
                    biomes.push((y as i32, BiomeSection { palette, idx }));
                }
            }
            let Some(bs) = s.get("block_states") else {
                continue;
            };
            let palette_tag = bs.get("palette").and_then(Tag::as_list).unwrap_or(&[]);
            if palette_tag.is_empty() {
                continue;
            }
            let mut palette = Vec::with_capacity(palette_tag.len());
            let mut flags = 0;
            for p in palette_tag {
                let name = p
                    .get("Name")
                    .and_then(Tag::as_str)
                    .unwrap_or("minecraft:air");
                let props: Vec<(String, String)> = p
                    .get("Properties")
                    .and_then(Tag::as_compound)
                    .map(|c| {
                        c.iter()
                            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_owned()))
                            .collect()
                    })
                    .unwrap_or_default();
                let id = reg.intern(name, &props);
                flags |= reg.flags(id);
                palette.push(id);
            }
            let (bits, data) = if palette.len() == 1 {
                (0, Vec::new())
            } else {
                let bits = ceil_log2(palette.len()).max(4);
                let data = bs.get("data").and_then(Tag::as_long_array).unwrap_or(&[]);
                let per = (64 / bits) as usize;
                if data.len() != 4096usize.div_ceil(per) {
                    bail!(
                        "chunk {cx},{cz} section {y}: {} longs for a {}-entry palette ({bits} bits), expected {}",
                        data.len(),
                        palette.len(),
                        4096usize.div_ceil(per)
                    );
                }
                (bits, data.iter().map(|&w| w as u64).collect())
            };
            parsed.push((
                y as i32,
                Section {
                    palette,
                    data,
                    bits,
                    flags,
                },
            ));
        }
        let min_section = int("yPos")
            .map(|v| v as i32)
            .or_else(|| parsed.iter().map(|(y, _)| *y).min())
            .unwrap_or(0);
        let top = parsed
            .iter()
            .map(|(y, _)| *y)
            .max()
            .unwrap_or(min_section - 1);
        let mut sections: Vec<Option<Section>> = (min_section..=top).map(|_| None).collect();
        for (y, s) in parsed {
            if y >= min_section {
                sections[(y - min_section) as usize] = Some(s);
            }
        }
        // World height from the sections the file holds block states for.
        let height = sections.len() * 16;
        let mut heightmaps = Vec::new();
        if let Some(Tag::Compound(hm)) = root.get("Heightmaps") {
            for (name, v) in hm {
                let Some(data) = v.as_long_array() else {
                    continue;
                };
                let mut bits = ceil_log2(height + 1);
                if data.len() != 256usize.div_ceil((64 / bits.max(1)) as usize) {
                    // Fall back to the narrowest width that matches the array's length.
                    if let Some(b) = (1..=16).find(|&b| data.len() == 256usize.div_ceil(64 / b)) {
                        bits = b as u32;
                    } else {
                        continue;
                    }
                }
                if let Some(vals) = unpack(data, bits, 256) {
                    heightmaps.push((name.clone(), vals.into_iter().map(|v| v as u16).collect()));
                }
            }
        }
        let mut pieces = Vec::new();
        if let Some(Tag::Compound(starts)) = root.get("structures").and_then(|s| s.get("starts")) {
            for (id, start) in starts {
                if start.get("id").and_then(Tag::as_str) == Some("INVALID") {
                    continue;
                }
                for child in start.get("Children").and_then(Tag::as_list).unwrap_or(&[]) {
                    if let Some(bb) = child.get("BB").and_then(Tag::as_int_array)
                        && bb.len() == 6
                    {
                        pieces.push(Piece {
                            structure: id.clone(),
                            min: [bb[0], bb[1], bb[2]],
                            max: [bb[3], bb[4], bb[5]],
                        });
                    }
                }
            }
        }
        Ok(Chunk {
            cx: cx as i32,
            cz: cz as i32,
            status,
            min_section,
            sections,
            heightmaps,
            pieces,
            biomes,
        })
    }

    /// The biome id at local (x, z) and world y (clamped into the stored sections).
    pub fn biome(&self, x: usize, y: i32, z: usize) -> Option<&str> {
        let lo = self.biomes.iter().map(|(y0, _)| *y0).min()?;
        let hi = self.biomes.iter().map(|(y0, _)| *y0).max()?;
        let y = y.clamp(lo * 16, hi * 16 + 15);
        let (_, s) = self.biomes.iter().find(|(y0, _)| *y0 == y >> 4)?;
        Some(s.get(x, (y & 15) as usize, z))
    }

    pub fn is_full(&self) -> bool {
        self.status == "full"
    }

    /// Whether the chunk's terrain is final: status `carvers` or later (carvers and noise caves
    /// have run; later steps only add features, light and entities).
    pub fn terrain_done(&self) -> bool {
        const DONE: &[&str] = &[
            "carvers",
            "features",
            "initialize_light",
            "light",
            "spawn",
            "full",
        ];
        DONE.contains(&self.status.as_str())
    }

    /// The world's lowest block y as this chunk sees it.
    pub fn min_y(&self) -> i32 {
        self.min_section * 16
    }

    /// One past the highest block y this chunk holds.
    pub fn max_y(&self) -> i32 {
        (self.min_section + self.sections.len() as i32) * 16
    }

    /// The state id at local (x, z) and world y; air outside the stored sections.
    #[inline]
    pub fn get(&self, x: usize, y: i32, z: usize) -> u32 {
        let si = (y >> 4) - self.min_section;
        if si < 0 || si as usize >= self.sections.len() {
            return 0;
        }
        match &self.sections[si as usize] {
            Some(s) => s.get(x, (y & 15) as usize, z),
            None => 0,
        }
    }

    /// The section holding world y, if stored.
    pub fn section_at(&self, y: i32) -> Option<&Section> {
        let si = (y >> 4) - self.min_section;
        if si < 0 {
            return None;
        }
        self.sections.get(si as usize)?.as_ref()
    }

    /// Heightmap `name` at local (x, z) as the y of the column's top block (None: empty column
    /// or no such heightmap).
    pub fn heightmap_top(&self, name: &str, x: usize, z: usize) -> Option<i32> {
        let (_, v) = self.heightmaps.iter().find(|(n, _)| n == name)?;
        let h = i32::from(v[z * 16 + x]);
        (h > 0).then(|| self.min_y() + h - 1)
    }

    /// The highest y at or below `from` in column (x, z) whose block has none of `skip`'s flags,
    /// scanning down to the bottom; sections whose every palette entry has a `skip` flag are
    /// passed over whole.
    pub fn scan_down(
        &self,
        reg: &Registry,
        x: usize,
        z: usize,
        from: i32,
        skip: u16,
    ) -> Option<i32> {
        let mut y = from.min(self.max_y() - 1);
        let bottom = self.min_y();
        while y >= bottom {
            if let Some(s) = self.section_at(y) {
                let local = (y & 15) as usize;
                // A uniform section of skipped blocks is passed over at once.
                if s.bits == 0 && reg.flags(s.palette[0]) & skip != 0 {
                    y = (y & !15) - 1;
                    continue;
                }
                let id = s.get(x, local, z);
                if reg.flags(id) & skip == 0 {
                    return Some(y);
                }
                y -= 1;
            } else {
                // No stored blocks: air, which every scan here skips.
                if blocks::AIR & skip == 0 {
                    return Some(y);
                }
                y = (y & !15) - 1;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(y: i8, palette: &[&str], index: impl Fn(usize) -> u32) -> Tag {
        let bits = ceil_log2(palette.len()).max(4);
        let values: Vec<u32> = (0..4096).map(index).collect();
        let mut bs = vec![(
            "palette".to_owned(),
            Tag::List(
                palette
                    .iter()
                    .map(|n| Tag::Compound(vec![("Name".into(), Tag::String((*n).into()))]))
                    .collect(),
            ),
        )];
        if palette.len() > 1 {
            bs.push(("data".into(), Tag::LongArray(pack(&values, bits))));
        }
        Tag::Compound(vec![
            ("Y".into(), Tag::Byte(y)),
            ("block_states".into(), Tag::Compound(bs)),
        ])
    }

    #[test]
    fn biomes_by_quart_cell() {
        // Section 0: three biomes (2 bits), cell index y·16 + z·4 + x → palette index % 3;
        // section 1: one biome, no data.
        let pal = ["minecraft:desert", "minecraft:plains", "terralith:steppe"];
        let idx: Vec<u32> = (0..64).map(|i| (i % 3) as u32).collect();
        let biomes = |y: i8, p: &[&str], data: Option<Vec<i64>>| {
            let mut b = vec![(
                "palette".to_owned(),
                Tag::List(p.iter().map(|n| Tag::String((*n).into())).collect()),
            )];
            if let Some(d) = data {
                b.push(("data".into(), Tag::LongArray(d)));
            }
            Tag::Compound(vec![
                ("Y".into(), Tag::Byte(y)),
                ("biomes".into(), Tag::Compound(b)),
            ])
        };
        let root = Tag::Compound(vec![
            ("xPos".into(), Tag::Int(0)),
            ("zPos".into(), Tag::Int(0)),
            ("yPos".into(), Tag::Int(0)),
            (
                "sections".into(),
                Tag::List(vec![
                    biomes(0, &pal, Some(pack(&idx, 2))),
                    biomes(1, &["minecraft:ocean"], None),
                ]),
            ),
        ]);
        let c = Chunk::parse(&root, &mut Registry::new()).unwrap();
        // Block (5, 9, 13): cell x 1, y 2, z 3 → 2·16 + 3·4 + 1 = 45 → 45 % 3 = 0.
        assert_eq!(c.biome(5, 9, 13), Some("minecraft:desert"));
        // Cell 46 → 1.
        assert_eq!(c.biome(9, 9, 13), Some("minecraft:plains"));
        assert_eq!(c.biome(0, 20, 0), Some("minecraft:ocean"));
        // Clamped into the stored sections.
        assert_eq!(c.biome(0, 400, 0), Some("minecraft:ocean"));
        assert_eq!(c.biome(0, -50, 0), Some("minecraft:desert"));
    }

    #[test]
    fn packing() {
        assert_eq!(ceil_log2(1), 0);
        assert_eq!(ceil_log2(2), 1);
        assert_eq!(ceil_log2(17), 5);
        assert_eq!(ceil_log2(993), 10);
        let v: Vec<u32> = (0..256).map(|i| (i * 37 % 993) as u32).collect();
        let p = pack(&v, 10);
        assert_eq!(p.len(), 43); // 6 entries per long, no spanning
        assert_eq!(unpack(&p, 10, 256).unwrap(), v);
    }

    #[test]
    fn sections_and_heightmaps() {
        // Two sections of a world starting at y = -32: a 17-entry palette (5 bits) and a
        // uniform one.
        let names: Vec<String> = (0..17).map(|i| format!("test:b{i}")).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let s0 = section(-2, &refs, |i| (i % 17) as u32);
        let s1 = section(-1, &["minecraft:stone"], |_| 0);
        let s2 = section(0, &["minecraft:air"], |_| 0);
        // Height 48 → 6 bits; top block of column (3, 4) at y = -17 → stored -17 + 32 + 1 = 16.
        let mut hm = vec![0u32; 256];
        hm[4 * 16 + 3] = 16;
        let root = Tag::Compound(vec![
            ("xPos".into(), Tag::Int(-1)),
            ("zPos".into(), Tag::Int(2)),
            ("yPos".into(), Tag::Int(-2)),
            ("Status".into(), Tag::String("minecraft:full".into())),
            ("sections".into(), Tag::List(vec![s0, s1, s2])),
            (
                "Heightmaps".into(),
                Tag::Compound(vec![("WORLD_SURFACE".into(), Tag::LongArray(pack(&hm, 6)))]),
            ),
            (
                "structures".into(),
                Tag::Compound(vec![(
                    "starts".into(),
                    Tag::Compound(vec![(
                        "minecraft:shipwreck".into(),
                        Tag::Compound(vec![
                            ("id".into(), Tag::String("minecraft:shipwreck".into())),
                            (
                                "Children".into(),
                                Tag::List(vec![Tag::Compound(vec![(
                                    "BB".into(),
                                    Tag::IntArray(vec![1, 2, 3, 4, 5, 6]),
                                )])]),
                            ),
                        ]),
                    )]),
                )]),
            ),
        ]);
        let mut reg = Registry::new();
        let c = Chunk::parse(&root, &mut reg).unwrap();
        assert!(c.is_full());
        assert_eq!((c.cx, c.cz, c.min_y(), c.max_y()), (-1, 2, -32, 16));
        for (x, y, z) in [(0, -32, 0), (5, -30, 7), (15, -17, 15)] {
            let i = (((y + 32) as usize) << 8) | (z << 4) | x;
            assert_eq!(reg.name(c.get(x, y, z)), format!("test:b{}", i % 17));
        }
        assert_eq!(reg.name(c.get(2, -5, 2)), "minecraft:stone");
        assert_eq!(reg.name(c.get(2, 40, 2)), "minecraft:air");
        assert_eq!(c.heightmap_top("WORLD_SURFACE", 3, 4), Some(-17));
        assert_eq!(c.heightmap_top("WORLD_SURFACE", 0, 0), None);
        assert_eq!(c.scan_down(&reg, 2, 2, 100, blocks::AIR), Some(-1));
        assert_eq!(
            c.pieces,
            vec![Piece {
                structure: "minecraft:shipwreck".into(),
                min: [1, 2, 3],
                max: [4, 5, 6]
            }]
        );
    }
}

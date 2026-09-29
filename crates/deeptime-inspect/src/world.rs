// SPDX-License-Identifier: LGPL-3.0-or-later
//! A world folder's (or dimension folder's) `region/` read into chunks, and the per-column
//! questions every render asks: the top block, the terrain surface, the water surface.
//!
//! **The terrain surface** (`terrain_top`) is the y of the highest block in the column that is
//! not air (air, cave air, void air), not a fluid (water, lava, bubble column) and not a feature
//! ([`crate::blocks::is_feature`]: vegetation, logs, wood, leaves, roots, huge mushrooms, coral,
//! sea pickles, snow layers, sheet ice, pumpkins, melons, bee nests, pointed dripstone). It is
//! the block the generator's density function made solid, before trees and plants were placed
//! on it: `OCEAN_FLOOR` minus logs, leaves and the other features.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::blocks::{self, Registry};
use crate::chunk::Chunk;
use crate::error::{Error, Result};
use crate::region::{self, RegionFile};

/// A rectangle of block columns, `x0 ≤ x < x1`, `z0 ≤ z < z1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area {
    pub x0: i32,
    pub z0: i32,
    pub x1: i32,
    pub z1: i32,
}

impl Area {
    pub fn width(&self) -> usize {
        (self.x1 - self.x0).max(0) as usize
    }

    pub fn height(&self) -> usize {
        (self.z1 - self.z0).max(0) as usize
    }

    pub fn contains_chunk(&self, cx: i32, cz: i32) -> bool {
        cx * 16 < self.x1 && cx * 16 + 16 > self.x0 && cz * 16 < self.z1 && cz * 16 + 16 > self.z0
    }

    /// Parses `x0,z0,x1,z1`.
    pub fn parse(s: &str) -> std::result::Result<Area, String> {
        let v: Vec<i32> = s
            .split(',')
            .map(|p| p.trim().parse::<i32>().map_err(|e| format!("'{p}': {e}")))
            .collect::<std::result::Result<_, _>>()?;
        if v.len() != 4 || v[2] <= v[0] || v[3] <= v[1] {
            return Err(format!(
                "area '{s}' must be x0,z0,x1,z1 with x1 > x0 and z1 > z0"
            ));
        }
        Ok(Area {
            x0: v[0],
            z0: v[1],
            x1: v[2],
            z1: v[3],
        })
    }

    pub fn to_json(self) -> serde_json::Value {
        serde_json::json!({"x0": self.x0, "z0": self.z0, "x1": self.x1, "z1": self.z1})
    }
}

/// The chunks of one world folder.
pub struct World {
    pub dir: PathBuf,
    pub reg: Registry,
    pub chunks: HashMap<(i32, i32), Chunk>,
    /// Chunks that could not be read, with the reason.
    pub errors: Vec<String>,
    /// Region files read.
    pub regions: usize,
}

impl World {
    /// Reads every chunk of `<dir>/region` for which `want(cx, cz)` holds.
    pub fn load(dir: &Path, want: impl Fn(i32, i32) -> bool) -> Result<World> {
        let region_dir = dir.join("region");
        if !region_dir.is_dir() {
            return Err(Error(format!(
                "{}: no region/ folder (pass the world or dimension folder)",
                dir.display()
            )));
        }
        let mut w = World {
            dir: dir.to_owned(),
            reg: Registry::new(),
            chunks: HashMap::new(),
            errors: Vec::new(),
            regions: 0,
        };
        for (rx, rz, path) in region::list_regions(&region_dir)? {
            // Skip regions none of whose chunks is wanted.
            let any = (0..32).any(|i| (0..32).any(|j| want(rx * 32 + i, rz * 32 + j)));
            if !any {
                continue;
            }
            let r = RegionFile::open(&path)?;
            w.regions += 1;
            for (lx, lz) in r.present() {
                let (cx, cz) = r.chunk_pos(lx, lz);
                if !want(cx, cz) {
                    continue;
                }
                let parsed = r
                    .chunk_nbt(lx, lz)
                    .and_then(|t| Chunk::parse(&t.expect("present"), &mut w.reg));
                match parsed {
                    Ok(c) => {
                        if (c.cx, c.cz) != (cx, cz) {
                            w.errors.push(format!(
                                "{}: slot {cx},{cz} holds chunk {},{}",
                                path.display(),
                                c.cx,
                                c.cz
                            ));
                        }
                        w.chunks.insert((cx, cz), c);
                    }
                    Err(e) => w.errors.push(e.0),
                }
            }
        }
        Ok(w)
    }

    /// Reads the chunks overlapping `area` (all chunks if None).
    pub fn load_area(dir: &Path, area: Option<Area>) -> Result<World> {
        World::load(dir, |cx, cz| area.is_none_or(|a| a.contains_chunk(cx, cz)))
    }

    pub fn chunk(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.chunks.get(&(x >> 4, z >> 4))
    }

    /// The chunk of column (x, z) if it is fully generated.
    pub fn full_chunk(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.chunk(x, z).filter(|c| c.is_full())
    }

    /// The state id at (x, y, z); air where no chunk is loaded.
    pub fn block(&self, x: i32, y: i32, z: i32) -> u32 {
        self.chunk(x, z)
            .map_or(0, |c| c.get((x & 15) as usize, y, (z & 15) as usize))
    }

    pub fn flags(&self, x: i32, y: i32, z: i32) -> u16 {
        self.reg.flags(self.block(x, y, z))
    }

    /// The highest non-air block of column (x, z): WORLD_SURFACE when the chunk has it, else a
    /// scan from the top.
    pub fn top(&self, x: i32, z: i32) -> Option<i32> {
        let c = self.chunk(x, z)?;
        let (lx, lz) = ((x & 15) as usize, (z & 15) as usize);
        let start = c
            .heightmap_top("WORLD_SURFACE", lx, lz)
            .map_or(c.max_y() - 1, |y| (y + 1).min(c.max_y() - 1));
        c.scan_down(&self.reg, lx, lz, start, blocks::AIR)
    }

    /// The terrain surface of column (x, z) (see the module docs).
    pub fn terrain_top(&self, x: i32, z: i32) -> Option<i32> {
        let c = self.chunk(x, z)?;
        let start = self.top(x, z)?;
        c.scan_down(
            &self.reg,
            (x & 15) as usize,
            (z & 15) as usize,
            start,
            blocks::NOT_TERRAIN,
        )
    }

    /// The y of the highest block holding water above the terrain surface of (x, z), if any.
    pub fn water_top(&self, x: i32, z: i32) -> Option<i32> {
        let top = self.top(x, z)?;
        let floor = self.terrain_top(x, z).unwrap_or(i32::MIN);
        (floor + 1..=top)
            .rev()
            .find(|&y| self.flags(x, y, z) & blocks::WATER != 0)
    }

    /// Fully generated chunks, sorted.
    pub fn full_chunks(&self) -> Vec<(i32, i32)> {
        let mut v: Vec<_> = self
            .chunks
            .values()
            .filter(|c| c.is_full())
            .map(|c| (c.cx, c.cz))
            .collect();
        v.sort();
        v
    }

    /// Groups of fully generated chunks that touch (8-neighbourhood), each as the area of its
    /// bounding box, largest first. A default render area per group keeps a world with
    /// generated spots far apart (spawn and the seam) from becoming one mostly empty image.
    pub fn full_areas(&self) -> Vec<Area> {
        let full: std::collections::HashSet<(i32, i32)> = self.full_chunks().into_iter().collect();
        let mut seen = std::collections::HashSet::new();
        let mut areas = Vec::new();
        let mut keys: Vec<_> = full.iter().copied().collect();
        keys.sort();
        for start in keys {
            if !seen.insert(start) {
                continue;
            }
            let (mut x0, mut z0, mut x1, mut z1) = (start.0, start.1, start.0, start.1);
            let mut stack = vec![start];
            let mut n = 0usize;
            while let Some((cx, cz)) = stack.pop() {
                n += 1;
                x0 = x0.min(cx);
                z0 = z0.min(cz);
                x1 = x1.max(cx);
                z1 = z1.max(cz);
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        let p = (cx + dx, cz + dz);
                        if full.contains(&p) && seen.insert(p) {
                            stack.push(p);
                        }
                    }
                }
            }
            areas.push((
                n,
                Area {
                    x0: x0 * 16,
                    z0: z0 * 16,
                    x1: x1 * 16 + 16,
                    z1: z1 * 16 + 16,
                },
            ));
        }
        areas.sort_by(|a, b| b.0.cmp(&a.0).then((a.1.x0, a.1.z0).cmp(&(b.1.x0, b.1.z0))));
        areas.into_iter().map(|(_, a)| a).collect()
    }
}

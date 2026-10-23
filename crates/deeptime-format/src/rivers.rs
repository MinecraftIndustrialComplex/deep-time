// SPDX-License-Identifier: LGPL-3.0-or-later
//! The river network (P1-22, plan §8.4; FORMAT.md, "Rivers"): `rivers/rivers.dtrv`, polylines of
//! river channels in block coordinates with, at every vertex, the water surface, the half-width,
//! the depth and the discharge, and a tile index so a chunk finds the segments near it without
//! reading the rest. The bake writes it (`deeptime_core::hydro`); the column evaluator
//! ([`crate::water`]) and Java's `worlddata/Rivers` read it and answer the same per column.
//!
//! **Chains.** The vertices of one chain are consecutive in the vertex array; segment `s` is the
//! pair of vertices (s, s + 1), which exists when both belong to the same chain. A chain runs
//! downstream from a head (a spring, or a lake's outlet) to its mouth: the sea, a lake, or a
//! vertex of a larger river (a confluence). x is continuous along a chain (it may leave the
//! canonical span [−C/2, C/2) by up to one lap near the seam); readers choose the image of a
//! column nearest to each segment.
//!
//! **Index.** Tiles of `tile` blocks in the canonical frame: tile (tx, tz) covers x in
//! [−C/2 + tx·tile, …) and z in [−C/4 + tz·tile, …), tx mod `tiles_x`, tz clamped. Each segment is
//! listed, ascending, in every tile its box, grown by [`Rivers::reach_of`], touches; so every
//! segment that can change a column is listed in the column's own tile.

use std::path::Path;

use crate::container::{self, f32_at, u32_at};
use crate::error::{FormatError, Result};
use crate::fsutil::write_atomic;
use crate::projection::IndexFrame;

/// Where the network lives in a world-data folder.
pub const RIVERS_PATH: &str = "rivers/rivers.dtrv";
/// The file's magic.
pub const MAGIC: [u8; 4] = *b"DTRV";
/// Fixed fields: C, tile, tiles_x, tiles_z, vertices, chains, index entries (u32 each); the biome
/// width, the bank zone and its ratio (f32); the index's origin x and z (i32; format 0.5, they
/// were reserved before).
pub const FIXED_BYTES: usize = 48;
/// Sections: vertices, chains, tile offsets, tile entries.
pub const SECTIONS: usize = 4;
/// x, z, w, hw, depth, q: six f32.
pub const VERTEX_BYTES: usize = 24;
/// first, count (u32), mouth (u8), 3 reserved bytes, into (u32).
pub const CHAIN_BYTES: usize = 16;
/// The smallest half-width a channel has (a one-block stream on a diagonal still joins up).
pub const MIN_HALF_WIDTH: f64 = 0.71;
/// The index's tile edge, blocks.
pub const TILE_BLOCKS: u32 = 32;

/// One vertex of a chain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiverVertex {
    /// Block coordinates (x continuous along the chain).
    pub x: f32,
    pub z: f32,
    /// The water surface (the same convention as `water_level`: water fills every block below it).
    pub w: f32,
    /// Half the channel's width, blocks.
    pub hw: f32,
    /// The channel's depth at its centre, blocks.
    pub depth: f32,
    /// Discharge, m³/s.
    pub q: f32,
}

/// Where a chain ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mouth {
    /// Its last vertex is in the sea (or on the first or last row, where water leaves the map).
    Sea,
    /// Its last vertex is in lake `into`.
    Lake,
    /// Its last vertex is a vertex of chain `into` (a confluence).
    River,
    /// Its last vertex lies on a turning seam of a cube world (format 0.7): the river goes on as
    /// chain `into`, whose first vertex is the same planet point on the seam's partner edge
    /// (FORMAT.md "Rivers", the cube's split).
    Seam,
}

impl Mouth {
    pub fn code(self) -> u8 {
        match self {
            Mouth::Sea => 0,
            Mouth::Lake => 1,
            Mouth::River => 2,
            Mouth::Seam => 3,
        }
    }

    pub fn from_code(c: u8) -> Option<Mouth> {
        [Mouth::Sea, Mouth::Lake, Mouth::River, Mouth::Seam]
            .into_iter()
            .find(|m| m.code() == c)
    }

    pub fn name(self) -> &'static str {
        match self {
            Mouth::Sea => "sea",
            Mouth::Lake => "lake",
            Mouth::River => "river",
            Mouth::Seam => "seam",
        }
    }
}

/// One chain: `count` vertices from `first`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chain {
    pub first: u32,
    pub count: u32,
    pub mouth: Mouth,
    /// The lake id (`Mouth::Lake`), the chain (`Mouth::River`, `Mouth::Seam`), else `u32::MAX`.
    pub into: u32,
}

/// The network and its index.
#[derive(Clone, Debug, PartialEq)]
pub struct Rivers {
    /// C, blocks.
    pub c: u32,
    pub tile: u32,
    pub tiles_x: u32,
    pub tiles_z: u32,
    /// Channels at least this wide (2·hw) make river biomes (`WorldDataTerrain`, FORMAT.md).
    pub biome_min_width: f32,
    /// The bank zone beyond a channel's edge: `bank_zone + bank_ratio · hw` blocks.
    pub bank_zone: f32,
    pub bank_ratio: f32,
    /// Block x and z of the index's first tile (the world projection's index frame).
    pub origin_x: i32,
    pub origin_z: i32,
    pub vertices: Vec<RiverVertex>,
    pub chains: Vec<Chain>,
    /// `tiles_x · tiles_z + 1` offsets into `entries`, tile-major (tz · tiles_x + tx).
    pub offsets: Vec<u32>,
    /// Segment ids per tile, ascending within a tile.
    pub entries: Vec<u32>,
}

/// What the channels decide at one column (the network's part of [`crate::water`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RiverSample {
    /// The covering segment with the lowest water surface (ties: the lowest segment id).
    pub hit: Option<RiverHit>,
    /// The covering segment whose channel floor under the column ([`RiverHit::floor`]) is lowest
    /// (ties: the lowest segment id): the channel the column lies deepest in. A wide, winding
    /// river's segments overlap a column many times over, and the lowest-surface one is mostly a
    /// neighbour whose edge (or end) only grazes it, one block deep there; the bed follows this one
    /// (ledger: HYD-120).
    pub deep: Option<RiverHit>,
    /// The lowest bank top of the segments whose bank zone holds the column but whose channel
    /// does not: ⌈w⌉ + ⌊d − hw⌋ (a 1:1 slope up from one block above the water).
    pub bank_cap: Option<i32>,
    /// A covering segment is at least `biome_min_width` wide.
    pub biome: bool,
}

/// A channel over one column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiverHit {
    pub seg: u32,
    /// Position along the segment, 0 … 1.
    pub t: f64,
    /// Distance from the centre line, blocks.
    pub d: f64,
    /// Water surface, half-width (≥ [`MIN_HALF_WIDTH`]), centre depth and discharge there.
    pub w: f64,
    pub hw: f64,
    pub depth: f64,
    pub q: f64,
}

impl RiverHit {
    /// The channel's floor under the column: w − max(1, depth · (1 − (d / hw)²)), its parabolic
    /// cross-section with at least one block of water (ledger: HYD-022).
    pub fn floor(&self) -> f64 {
        let r = self.d / self.hw;
        self.w - (self.depth * (1.0 - r * r)).max(1.0)
    }
}

/// The geometry of one segment at one column (both readers do this arithmetic in this order).
struct Along {
    t: f64,
    d: f64,
}

fn along(c: f64, a: &RiverVertex, b: &RiverVertex, wx: f64, z: f64) -> Along {
    let (ax, az) = (f64::from(a.x), f64::from(a.z));
    let (bx, bz) = (f64::from(b.x), f64::from(b.z));
    // The column's image nearest to the segment's first vertex.
    let k = ((ax - wx) / c + 0.5).floor();
    let px = wx + k * c;
    let ex = bx - ax;
    let ez = bz - az;
    let l2 = ex * ex + ez * ez;
    let mut t = if l2 > 0.0 {
        ((px - ax) * ex + (z - az) * ez) / l2
    } else {
        0.0
    };
    t = t.clamp(0.0, 1.0);
    let dx = px - (ax + t * ex);
    let dz = z - (az + t * ez);
    Along {
        t,
        d: (dx * dx + dz * dz).sqrt(),
    }
}

fn lerp(a: f32, b: f32, t: f64) -> f64 {
    f64::from(a) + (f64::from(b) - f64::from(a)) * t
}

impl Rivers {
    /// An empty network for a Lambert planet (or the simulation's frame) of circumference `c`.
    pub fn empty(c: u32) -> Rivers {
        Rivers::empty_in(c, IndexFrame::lambert(c))
    }

    /// An empty network of x period `c` whose tiles cover `frame` (the world projection's index
    /// frame, [`crate::projection::WorldProjection::index_frame`]).
    pub fn empty_in(c: u32, frame: IndexFrame) -> Rivers {
        let tile = TILE_BLOCKS;
        let (span_x, span_z) = frame.span;
        let tiles_x = (span_x as u32).div_ceil(tile);
        let tiles_z = (span_z as u32).div_ceil(tile);
        Rivers {
            c,
            tile,
            tiles_x,
            tiles_z,
            biome_min_width: 6.0,
            bank_zone: 1.0,
            bank_ratio: 0.5,
            origin_x: frame.origin.0 as i32,
            origin_z: frame.origin.1 as i32,
            vertices: Vec::new(),
            chains: Vec::new(),
            offsets: vec![0; (tiles_x * tiles_z) as usize + 1],
            entries: Vec::new(),
        }
    }

    /// How far beyond its centre line a segment with half-width `hw` can change a column: its
    /// channel and bank zone, plus a block.
    pub fn reach_of(&self, hw: f64) -> f64 {
        hw.max(MIN_HALF_WIDTH) + f64::from(self.bank_zone) + f64::from(self.bank_ratio) * hw + 1.0
    }

    /// Whether segment `s` exists (both vertices in one chain).
    fn is_segment(&self, chain_of: &[u32], s: usize) -> bool {
        s + 1 < self.vertices.len() && chain_of[s] == chain_of[s + 1]
    }

    /// Rebuilds the tile index from the vertices and chains.
    pub fn build_index(&mut self) {
        let n_tiles = (self.tiles_x * self.tiles_z) as usize;
        let mut chain_of = vec![u32::MAX; self.vertices.len()];
        for (k, ch) in self.chains.iter().enumerate() {
            for v in ch.first..ch.first + ch.count {
                chain_of[v as usize] = k as u32;
            }
        }
        let (x0, z0) = (f64::from(self.origin_x), f64::from(self.origin_z));
        let tile = f64::from(self.tile);
        let mut lists: Vec<Vec<u32>> = vec![Vec::new(); n_tiles];
        for s in 0..self.vertices.len() {
            if !self.is_segment(&chain_of, s) {
                continue;
            }
            let (a, b) = (&self.vertices[s], &self.vertices[s + 1]);
            let r = self.reach_of(f64::from(a.hw.max(b.hw)));
            let lo_x = f64::from(a.x.min(b.x)) - r;
            let hi_x = f64::from(a.x.max(b.x)) + r;
            let lo_z = f64::from(a.z.min(b.z)) - r;
            let hi_z = f64::from(a.z.max(b.z)) + r;
            let tx0 = ((lo_x - x0) / tile).floor() as i64;
            let tx1 = ((hi_x - x0) / tile).floor() as i64;
            let tz0 = (((lo_z - z0) / tile).floor() as i64).clamp(0, i64::from(self.tiles_z) - 1);
            let tz1 = (((hi_z - z0) / tile).floor() as i64).clamp(0, i64::from(self.tiles_z) - 1);
            let span = (tx1 - tx0).min(i64::from(self.tiles_x) - 1);
            for tz in tz0..=tz1 {
                for dx in 0..=span {
                    let tx = (tx0 + dx).rem_euclid(i64::from(self.tiles_x));
                    let t = (tz * i64::from(self.tiles_x) + tx) as usize;
                    if lists[t].last() != Some(&(s as u32)) {
                        lists[t].push(s as u32);
                    }
                }
            }
        }
        self.offsets.clear();
        self.entries.clear();
        self.offsets.push(0);
        for l in &lists {
            self.entries.extend_from_slice(l);
            self.offsets.push(self.entries.len() as u32);
        }
    }

    /// The tile of canonical column (wx, z).
    pub fn tile_of(&self, wx: i64, z: i64) -> usize {
        let (ox, oz) = (i64::from(self.origin_x), i64::from(self.origin_z));
        let t = i64::from(self.tile);
        let tx = (wx - ox).div_euclid(t).rem_euclid(i64::from(self.tiles_x));
        let tz = (z - oz).div_euclid(t).clamp(0, i64::from(self.tiles_z) - 1);
        (tz * i64::from(self.tiles_x) + tx) as usize
    }

    /// The segments listed in tile `t`.
    pub fn tile_entries(&self, t: usize) -> &[u32] {
        &self.entries[self.offsets[t] as usize..self.offsets[t + 1] as usize]
    }

    /// What the channels decide at column (x, z) (any lap), from the segments of its tile.
    pub fn sample(&self, x: i64, z: i64) -> RiverSample {
        let c = i64::from(self.c);
        let wx = (x + c / 2).rem_euclid(c) - c / 2;
        self.sample_among(self.tile_entries(self.tile_of(wx, z)), wx, z)
    }

    /// As [`Rivers::sample`] over any superset of the column's tile entries (a chunk gathers the
    /// entries of the tiles it touches once): the result does not depend on the order or on
    /// segments that cannot reach the column.
    pub fn sample_among(&self, segs: &[u32], wx: i64, z: i64) -> RiverSample {
        let c = f64::from(self.c);
        let (px, pz) = (wx as f64, z as f64);
        let mut out = RiverSample::default();
        for &s in segs {
            let (a, b) = (&self.vertices[s as usize], &self.vertices[s as usize + 1]);
            let g = along(c, a, b, px, pz);
            let hw = lerp(a.hw, b.hw, g.t);
            let hw_eff = hw.max(MIN_HALF_WIDTH);
            if g.d <= hw_eff {
                let h = RiverHit {
                    seg: s,
                    t: g.t,
                    d: g.d,
                    w: lerp(a.w, b.w, g.t),
                    hw: hw_eff,
                    depth: lerp(a.depth, b.depth, g.t),
                    q: lerp(a.q, b.q, g.t),
                };
                let better = match &out.hit {
                    None => true,
                    Some(o) => h.w < o.w || (h.w == o.w && s < o.seg),
                };
                if better {
                    out.hit = Some(h);
                }
                // The deepest channel (ledger: HYD-120).
                let f = h.floor();
                let deeper = match &out.deep {
                    None => true,
                    Some(o) => {
                        let of = o.floor();
                        f < of || (f == of && s < o.seg)
                    }
                };
                if deeper {
                    out.deep = Some(h);
                }
                if 2.0 * hw >= f64::from(self.biome_min_width) {
                    out.biome = true;
                }
            } else if g.d <= hw_eff + f64::from(self.bank_zone) + f64::from(self.bank_ratio) * hw {
                let w = lerp(a.w, b.w, g.t);
                let cap = w.ceil() as i32 + (g.d - hw_eff).floor() as i32;
                out.bank_cap = Some(out.bank_cap.map_or(cap, |b| b.min(cap)));
            }
        }
        out
    }

    /// The segment ids of every tile touching block box [x0, x1] × [z0, z1] (canonical or not),
    /// ascending and without repeats.
    pub fn entries_in(&self, x0: i64, z0: i64, x1: i64, z1: i64) -> Vec<u32> {
        let (ox, oz) = (i64::from(self.origin_x), i64::from(self.origin_z));
        let t = i64::from(self.tile);
        let tx0 = (x0 - ox).div_euclid(t);
        let tx1 = (x1 - ox).div_euclid(t);
        let tz0 = (z0 - oz)
            .div_euclid(t)
            .clamp(0, i64::from(self.tiles_z) - 1);
        let tz1 = (z1 - oz)
            .div_euclid(t)
            .clamp(0, i64::from(self.tiles_z) - 1);
        let span = (tx1 - tx0).min(i64::from(self.tiles_x) - 1);
        let mut out = Vec::new();
        for tz in tz0..=tz1 {
            for dx in 0..=span {
                let tx = (tx0 + dx).rem_euclid(i64::from(self.tiles_x));
                out.extend_from_slice(
                    self.tile_entries((tz * i64::from(self.tiles_x) + tx) as usize),
                );
            }
        }
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Checks counts, chains and the index against each other.
    pub fn validate(&self) -> Result<()> {
        let bad = |m: String| Err(FormatError::Invalid(format!("rivers: {m}")));
        if self.tile == 0 || self.tiles_x == 0 || self.tiles_z == 0 {
            return bad("empty tile grid".into());
        }
        if self.offsets.len() != (self.tiles_x * self.tiles_z) as usize + 1 {
            return bad("offsets do not match the tile grid".into());
        }
        if self.offsets.first() != Some(&0)
            || self.offsets.last().copied() != Some(self.entries.len() as u32)
            || self.offsets.windows(2).any(|w| w[0] > w[1])
        {
            return bad("offsets are not a partition of the entries".into());
        }
        let mut next = 0u32;
        for (k, ch) in self.chains.iter().enumerate() {
            if ch.first != next || ch.count < 2 {
                return bad(format!(
                    "chain {k} does not follow the previous one or is short"
                ));
            }
            next = ch.first + ch.count;
            if ch.mouth == Mouth::River && ch.into as usize >= self.chains.len() {
                return bad(format!("chain {k} flows into a missing chain"));
            }
        }
        if next as usize != self.vertices.len() {
            return bad("the chains do not cover the vertices".into());
        }
        for (i, v) in self.vertices.iter().enumerate() {
            let finite = [v.x, v.z, v.w, v.hw, v.depth, v.q]
                .iter()
                .all(|f| f.is_finite());
            if !finite || v.hw <= 0.0 || v.depth <= 0.0 {
                return bad(format!("vertex {i} is not finite or has no width or depth"));
            }
        }
        if self
            .entries
            .iter()
            .any(|&s| s as usize + 1 >= self.vertices.len())
        {
            return bad("an index entry is past the vertices".into());
        }
        Ok(())
    }
}

/// Encodes `rivers/rivers.dtrv`.
pub fn encode_rivers(r: &Rivers) -> Result<Vec<u8>> {
    r.validate()?;
    let mut fixed = Vec::with_capacity(FIXED_BYTES);
    for v in [
        r.c,
        r.tile,
        r.tiles_x,
        r.tiles_z,
        r.vertices.len() as u32,
        r.chains.len() as u32,
        r.entries.len() as u32,
    ] {
        fixed.extend_from_slice(&v.to_le_bytes());
    }
    for v in [r.biome_min_width, r.bank_zone, r.bank_ratio] {
        fixed.extend_from_slice(&v.to_le_bytes());
    }
    fixed.extend_from_slice(&r.origin_x.to_le_bytes());
    fixed.extend_from_slice(&r.origin_z.to_le_bytes());
    debug_assert_eq!(fixed.len(), FIXED_BYTES);
    let mut verts = Vec::with_capacity(r.vertices.len() * VERTEX_BYTES);
    for v in &r.vertices {
        for f in [v.x, v.z, v.w, v.hw, v.depth, v.q] {
            verts.extend_from_slice(&f.to_le_bytes());
        }
    }
    let mut chains = Vec::with_capacity(r.chains.len() * CHAIN_BYTES);
    for ch in &r.chains {
        chains.extend_from_slice(&ch.first.to_le_bytes());
        chains.extend_from_slice(&ch.count.to_le_bytes());
        chains.push(ch.mouth.code());
        chains.extend_from_slice(&[0; 3]);
        chains.extend_from_slice(&ch.into.to_le_bytes());
    }
    let offsets: Vec<u8> = r.offsets.iter().flat_map(|v| v.to_le_bytes()).collect();
    let entries: Vec<u8> = r.entries.iter().flat_map(|v| v.to_le_bytes()).collect();
    container::encode(MAGIC, &fixed, &[&verts, &chains, &offsets, &entries])
}

/// Decodes `rivers/rivers.dtrv`.
pub fn decode_rivers(path: &Path, b: &[u8]) -> Result<Rivers> {
    let p = container::parse(path, b, MAGIC, FIXED_BYTES, |_| SECTIONS)?;
    let f = p.fixed;
    let [c, tile, tiles_x, tiles_z, nv, nc, ne] = std::array::from_fn(|i| u32_at(f, 4 * i));
    let corrupt = |m: &str| FormatError::corrupt(path, m);
    let [verts, chains, offsets, entries] =
        [p.sections[0], p.sections[1], p.sections[2], p.sections[3]];
    let n_tiles = tiles_x as usize * tiles_z as usize;
    if verts.len() != nv as usize * VERTEX_BYTES
        || chains.len() != nc as usize * CHAIN_BYTES
        || offsets.len() != (n_tiles + 1) * 4
        || entries.len() != ne as usize * 4
    {
        return Err(corrupt("a section has the wrong length"));
    }
    let vertices = (0..nv as usize)
        .map(|i| {
            let a = i * VERTEX_BYTES;
            let g = |k: usize| f32_at(verts, a + 4 * k);
            RiverVertex {
                x: g(0),
                z: g(1),
                w: g(2),
                hw: g(3),
                depth: g(4),
                q: g(5),
            }
        })
        .collect();
    let mut out_chains = Vec::with_capacity(nc as usize);
    for k in 0..nc as usize {
        let a = k * CHAIN_BYTES;
        let mouth = Mouth::from_code(chains[a + 8]).ok_or_else(|| corrupt("unknown mouth"))?;
        out_chains.push(Chain {
            first: u32_at(chains, a),
            count: u32_at(chains, a + 4),
            mouth,
            into: u32_at(chains, a + 12),
        });
    }
    let r = Rivers {
        c,
        tile,
        tiles_x,
        tiles_z,
        biome_min_width: f32_at(f, 28),
        bank_zone: f32_at(f, 32),
        bank_ratio: f32_at(f, 36),
        origin_x: u32_at(f, 40) as i32,
        origin_z: u32_at(f, 44) as i32,
        vertices,
        chains: out_chains,
        offsets: (0..=n_tiles).map(|t| u32_at(offsets, 4 * t)).collect(),
        entries: (0..ne as usize).map(|i| u32_at(entries, 4 * i)).collect(),
    };
    r.validate()
        .map_err(|e| FormatError::corrupt(path, e.to_string()))?;
    Ok(r)
}

/// Writes the network into world folder `world`; returns (bytes, CRC32).
pub fn write_rivers(world: &Path, r: &Rivers) -> Result<(u64, u32)> {
    let bytes = encode_rivers(r)?;
    let crc = write_atomic(&world.join(RIVERS_PATH), &bytes)?;
    Ok((bytes.len() as u64, crc))
}

/// Reads the network of world folder `world`, None when it has none.
pub fn read_rivers(world: &Path) -> Result<Option<Rivers>> {
    let path = world.join(RIVERS_PATH);
    match std::fs::read(&path) {
        Ok(b) => Ok(Some(decode_rivers(&path, &b)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(FormatError::io(&path, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, z: f32, w: f32, hw: f32) -> RiverVertex {
        RiverVertex {
            x,
            z,
            w,
            hw,
            depth: 2.0,
            q: 10.0,
        }
    }

    /// A network of C = 1024: a chain along z = 0 across the seam (x continuous from 500 to 540)
    /// and a tributary joining its third vertex.
    fn network() -> Rivers {
        let mut r = Rivers::empty(1024);
        r.vertices = vec![
            v(500.0, 0.0, 80.0, 1.0),
            v(510.0, 0.0, 79.0, 1.5),
            v(520.0, 0.0, 78.0, 2.0),
            v(540.0, 0.0, 76.0, 4.0),
            v(520.0, -30.0, 90.0, 0.5),
            v(520.0, 0.0, 78.0, 0.5),
        ];
        r.chains = vec![
            Chain {
                first: 0,
                count: 4,
                mouth: Mouth::Sea,
                into: u32::MAX,
            },
            Chain {
                first: 4,
                count: 2,
                mouth: Mouth::River,
                into: 0,
            },
        ];
        r.build_index();
        r
    }

    #[test]
    fn round_trip_and_index() {
        let r = network();
        let bytes = encode_rivers(&r).unwrap();
        let back = decode_rivers(Path::new("rivers.dtrv"), &bytes).unwrap();
        assert_eq!(back, r);
        // Segment 3 (vertex 3 → 4) crosses chains: never listed.
        assert!(!r.entries.contains(&3));
        assert!(r.entries.contains(&4));
    }

    #[test]
    fn a_column_across_the_seam_sees_the_channel() {
        let r = network();
        // x = 530 is −494 in the canonical span: the column (−494, 0) is on segment 2.
        let s = r.sample(-494, 0);
        let h = s.hit.expect("on the channel");
        assert_eq!(h.seg, 2);
        assert!((h.t - 0.5).abs() < 1e-12);
        assert!((h.w - 77.0).abs() < 1e-12);
        assert_eq!(r.sample(530, 0), s, "any lap");
        assert_eq!(r.sample(530 + 1024, 0), s);
        // Beyond the channel: a bank cap, no hit.
        let bank = r.sample(-494, 4);
        assert!(bank.hit.is_none());
        assert!(bank.bank_cap.is_some());
        assert_eq!(r.sample(-494, 40), RiverSample::default());
    }

    #[test]
    fn the_lowest_surface_wins_and_order_does_not_matter() {
        let r = network();
        // (520, 0) is on the main stem (w 78) and at the tributary's mouth (w 78): tie → lower id.
        let s = r.sample(520, -1);
        assert_eq!(s.hit.unwrap().seg, 1);
        let mut all: Vec<u32> = (0..5).filter(|&s| s != 3).collect();
        let a = r.sample_among(&all, 520 - 1024, -1);
        all.reverse();
        let b = r.sample_among(&all, 520 - 1024, -1);
        assert_eq!(a, b);
        assert_eq!(a, s);
    }

    #[test]
    fn corrupt_files_are_refused() {
        let r = network();
        let mut bytes = encode_rivers(&r).unwrap();
        let n = bytes.len();
        bytes[n - 9] ^= 0xFF;
        assert!(decode_rivers(Path::new("x"), &bytes).is_err());
    }
}

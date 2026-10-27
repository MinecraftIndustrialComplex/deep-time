// SPDX-License-Identifier: LGPL-3.0-or-later
//! Where the water goes in one block column (P1-22; FORMAT.md, "Water columns"): the part of the
//! column evaluator that decides lakes, rivers, the sea and the banks that hold them. Java's
//! `worlddata/Water` is its twin, operation for operation, and the game's `worldgen/WaterBodies`
//! pass makes every generated column match it. Everything here is a function of (x, z) and the
//! world data alone — never of blocks already placed — so neighbouring chunks, and the two sides
//! of the seam, agree on every column without seeing each other.
//!
//! 1. **The ground.** The terrain density is interpolated over noise cells (`cell` = 4 blocks): the
//!    generated surface of column (x, z) is H bilinear between the four cell corners around it,
//!    `surface`, and its top block `ground` = ⌈surface⌉ − 1 (clamped like `top_y`).
//! 2. **Lake.** `level` is the `water_level` layer by the shore rule (its nearest cell, else the
//!    nearest wet cell within `reach` blocks; Java `WaterLevels.levelAt`). Where
//!    `ground` < `top` = ⌈level⌉ − 1 the column holds lake water from `ground` + 1 to `top`. Where
//!    a channel covers it and the lake is less than [`SHALLOW_LAKE`] deep there
//!    (level − surface), the bed is the channel's floor `channel` (step 4) when that is lower: a
//!    river crossing shallow lake water keeps its channel, one ending in a lake does not trench
//!    the lake's floor.
//! 3. **Sea.** Otherwise, where `ground` < sea level − 1 and the `closed_basin` layer's nearest
//!    cell is not 1, the sea covers it (`top` = sea level − 1): the aquifer's water, which the
//!    game's pass tops up where the aquifer left the column dry. Where `closed_basin` is 1 (a basin
//!    below sea level that the open sea does not reach, behind a rim above it: HYD-100) no sea
//!    stands; the column is a lake by step 2, a river, a bank or dry ground. A world without the
//!    layer has the sea on all ground below sea level (FORMAT.md 0.5 and before). Where a channel
//!    covers it, `bed` = min(`ground`, `channel`): a river's channel runs on across the shallows
//!    into the sea.
//! 4. **River.** Otherwise, where a channel covers the column ([`crate::rivers::Rivers::sample`]),
//!    the lowest covering water surface w gives t = ⌈w⌉ − 1 and the bed is cut (or filled) to
//!    `bed` = min(top − 1, `channel` − drop), `channel` = ⌊min over the covering segments of
//!    w − max(1, depth · (1 − (d / hw)²))⌋: each segment's parabolic cross-section, at least one
//!    block of water, and the deepest of them where several overlap (a wide, winding river's
//!    segments overlap a column many times over). `top` = t − drop: drop is [`INCISION`] where
//!    the ground a block beyond the deepest segment's edges ([`bank_ground`]) does not stand
//!    above t and t is above the sea's top, else 0, so a channel on flat ground is cut into it.
//! 5. **Bank.** A column holding none of these is a bank when a 4-neighbour holds water: its ground
//!    must reach `min_ground` = the highest water a neighbour holds (a lake's or the sea's `top`;
//!    beside a river the highest `top` of the river column and its own water neighbours,
//!    [`held`], so the one-block steps the pass puts in stay inside the banks), and air from
//!    `seal_from` (one above the lowest neighbour bed; beside the sea [`SEA_SEAL`] blocks under its
//!    top at most) up to `min_ground` is sealed, so no water finds a way out. Within a
//!    channel's bank zone the ground may be at most `max_ground`, a 1:1 slope up from a block above
//!    the water. A column may be a bank for either reason.

use serde_json::{Value, json};

use crate::rivers::{RiverHit, RiverSample, Rivers};

/// The column's water, as the pass builds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaterKind {
    None,
    Bank,
    River,
    Lake,
    Sea,
}

impl WaterKind {
    pub fn name(self) -> &'static str {
        match self {
            WaterKind::None => "none",
            WaterKind::Bank => "bank",
            WaterKind::River => "river",
            WaterKind::Lake => "lake",
            WaterKind::Sea => "sea",
        }
    }

    /// Whether the column holds water.
    pub fn is_water(self) -> bool {
        matches!(self, WaterKind::River | WaterKind::Lake | WaterKind::Sea)
    }
}

/// The constants the game's generator fixes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaterSpec {
    /// The noise cell's width, blocks (`noise.size_horizontal` · 4 = 4 in `deeptime:planet`).
    pub cell: i64,
    /// The shore rule's reach, blocks (one noise cell).
    pub reach: i64,
    pub sea_level_y: i32,
    pub min_y: i32,
    pub max_y: i32,
}

impl WaterSpec {
    /// The spec of a Deep Time world with `sea_level_y` and build range [min_y, max_y].
    pub fn planet(sea_level_y: i32, min_y: i32, max_y: i32) -> WaterSpec {
        WaterSpec {
            cell: 4,
            reach: 4,
            sea_level_y,
            min_y,
            max_y,
        }
    }
}

/// What a world gives the evaluator.
pub trait WaterSource {
    /// The terrain height H at block (x, z) (any lap).
    fn height(&self, x: i64, z: i64) -> f64;
    /// The `water_level` layer at block (x, z) by the shore rule; NaN where none.
    fn level(&self, x: i64, z: i64, reach: i64) -> f64;
    /// The river network, if the world has one.
    fn rivers(&self) -> Option<&Rivers>;
    /// Whether the `closed_basin` layer keeps the sea off block (x, z) (its nearest cell is 1).
    /// A world without the layer has none.
    fn closed(&self, _x: i64, _z: i64) -> bool {
        false
    }
}

/// One column's water (FORMAT.md, "Water columns").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterColumn {
    pub kind: WaterKind,
    /// The generated surface: H bilinear between the noise-cell corners.
    pub surface: f64,
    /// ⌈surface⌉ − 1, clamped to [min_y − 1, max_y].
    pub ground: i32,
    /// The lake level by the shore rule, where there is one.
    pub level: Option<f64>,
    /// The highest water block (river, lake, sea).
    pub top: Option<i32>,
    /// The solid block under the water (river, lake, sea).
    pub bed: Option<i32>,
    /// A bank's lowest allowed ground top.
    pub min_ground: Option<i32>,
    /// A bank's highest allowed ground top.
    pub max_ground: Option<i32>,
    /// The lowest y a bank seals (with `min_ground`).
    pub seal_from: Option<i32>,
    /// A channel wide enough for a river biome covers the column (whatever its kind).
    pub river_biome: bool,
    /// The channel over the column (river columns, and lake or sea columns a channel covers).
    pub river: Option<RiverHit>,
}

/// The kind, top and bed of a column before its neighbours are looked at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Core {
    pub kind: WaterKind,
    pub surface: f64,
    pub ground: i32,
    pub level: f64,
    pub top: i32,
    pub bed: i32,
    pub rivers: RiverSample,
}

/// The block y under a water surface or terrain height `h`: ⌈h⌉ − 1.
pub fn top_block(h: f64) -> i32 {
    (h.ceil() as i64 - 1).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

/// Step 3's test, the one place the sea's rule is written (ledger: HYD-106): the open sea stands on
/// a column whose top block `ground` is under sea level − 1 (a column whose top block is the
/// block under the sea's top is dry, so the water has at least one block to be: the 35 m band of
/// HYD-107) unless the `closed_basin` layer keeps it off (`closed`, read only where the first test
/// passes).
///
/// The game's column evaluator ([`core_with_surface`]), the bake's projection pass (the biome
/// layers' land call: `bake::project`, via [`sea_covers_height`]) and Java's `Water.seaCovers` all
/// read this one rule, so the sea, the far terrain and the biomes cannot name different columns.
pub fn sea_covers(ground: i32, sea_level_y: i32, closed: impl FnOnce() -> bool) -> bool {
    ground < sea_level_y - 1 && !closed()
}

/// [`sea_covers`] for a terrain height `h` (blocks) at a column: its top block is
/// [`top_block`]`(h)`.
pub fn sea_covers_height(h: f64, sea_level_y: i32, closed: impl FnOnce() -> bool) -> bool {
    sea_covers(top_block(h), sea_level_y, closed)
}

/// H bilinear between the corners of the noise cell holding column (wx, z).
pub fn cell_surface(src: &dyn WaterSource, spec: &WaterSpec, wx: i64, z: i64) -> f64 {
    let c = spec.cell;
    let x0 = wx.div_euclid(c) * c;
    let z0 = z.div_euclid(c) * c;
    let fx = (wx - x0) as f64 / c as f64;
    let fz = (z - z0) as f64 / c as f64;
    let h00 = src.height(x0, z0);
    let h10 = src.height(x0 + c, z0);
    let h01 = src.height(x0, z0 + c);
    let h11 = src.height(x0 + c, z0 + c);
    let a = h00 + (h10 - h00) * fx;
    let b = h01 + (h11 - h01) * fx;
    a + (b - a) * fz
}

/// Steps 1–4 for column (wx, z) (canonical x), given its generated surface.
pub fn core_with_surface(
    src: &dyn WaterSource,
    spec: &WaterSpec,
    wx: i64,
    z: i64,
    surface: f64,
    rivers: RiverSample,
) -> Core {
    let ground = ground_of(spec, surface);
    let level = src.level(wx, z, spec.reach);
    let mut out = Core {
        kind: WaterKind::None,
        surface,
        ground,
        level,
        top: i32::MIN,
        bed: i32::MIN,
        rivers,
    };
    // The channel's floor under the column, as a block: the lowest of its covering segments'
    // parabolas (ledger: HYD-022, HYD-120).
    let channel = rivers.deep.map(|d| d.floor().floor() as i32);
    if level.is_finite() {
        let t = top_block(level);
        if ground < t {
            out.kind = WaterKind::Lake;
            out.top = t;
            out.bed = ground;
            // A channel crossing lake water less than a block deep keeps its bed (ledger: HYD-122).
            if let Some(f) = channel
                && level - surface < SHALLOW_LAKE
            {
                out.bed = ground.min(f);
            }
            return out;
        }
    }
    if sea_covers(ground, spec.sea_level_y, || src.closed(wx, z)) {
        out.kind = WaterKind::Sea;
        out.top = spec.sea_level_y - 1;
        // A channel into the sea keeps its bed across the shallows (ledger: HYD-121).
        out.bed = channel.map_or(ground, |f| ground.min(f));
        return out;
    }
    if let (Some(h), Some(d), Some(f)) = (rivers.hit, rivers.deep, channel) {
        let t = top_block(h.w);
        // Incised, not perched: where the ground at the channel's banks does not stand above the
        // water's top block, the water and the bed go [`INCISION`] down, so the banks stand over
        // the water without being raised; never under the sea's top (ledger: HYD-123).
        let sea_top = spec.sea_level_y - 1;
        let drop = if t > sea_top && bank_ground(src, spec, ground, &d) <= t {
            INCISION.min(t - sea_top)
        } else {
            0
        };
        out.kind = WaterKind::River;
        out.top = t - drop;
        out.bed = (f - drop).min(out.top - 1);
    }
    out
}

/// How far a river column's water and bed go down where its banks do not stand above the water
/// (step 4), blocks: one, the freeboard the bake leaves under its filled surface (HYD-018), so a
/// channel on flat ground is cut a block into it rather than walled a block above it
/// (game-choice: Ben's curb report, 2026-10-22; ledger: HYD-123).
pub const INCISION: i32 = 1;

/// The top block of a generated surface, as step 1 clamps it.
fn ground_of(spec: &WaterSpec, surface: f64) -> i32 {
    if surface.is_finite() {
        top_block(surface).clamp(spec.min_y - 1, spec.max_y)
    } else {
        spec.min_y - 1
    }
}

/// The lower of the generated ground's top blocks at the two points a block beyond the edges of
/// channel `d` (on its segment's normal through the foot at `d.t`, `d.hw` + 1 out each side):
/// where the channel's banks stand before the pass touches them (ledger: HYD-123). A segment of
/// no length has no normal: the column's own `ground`.
pub fn bank_ground(src: &dyn WaterSource, spec: &WaterSpec, ground: i32, d: &RiverHit) -> i32 {
    let Some(r) = src.rivers() else {
        return ground;
    };
    let (a, b) = (&r.vertices[d.seg as usize], &r.vertices[d.seg as usize + 1]);
    let (ax, az) = (f64::from(a.x), f64::from(a.z));
    let (ex, ez) = (f64::from(b.x) - ax, f64::from(b.z) - az);
    let l = (ex * ex + ez * ez).sqrt();
    if l <= 0.0 {
        return ground;
    }
    let (cx, cz) = (ax + d.t * ex, az + d.t * ez);
    let (nx, nz) = (-ez / l, ex / l);
    let off = d.hw + 1.0;
    let c = i64::from(r.c);
    let at = |px: f64, pz: f64| -> i32 {
        let x = px.floor() as i64;
        let x = (x + c / 2).rem_euclid(c) - c / 2;
        ground_of(spec, cell_surface(src, spec, x, pz.floor() as i64))
    };
    at(cx + nx * off, cz + nz * off).min(at(cx - nx * off, cz - nz * off))
}

/// A lake column a channel covers keeps the channel's bed where the lake's water over the
/// generated surface is less than this deep, blocks: the bake's own line between lake water a
/// river crosses and lake water it ends in (HYD-020, a block), read at the column (ledger: HYD-122).
pub const SHALLOW_LAKE: f64 = 1.0;

/// Steps 1–4 for column (x, z), any lap.
pub fn core(src: &dyn WaterSource, spec: &WaterSpec, c: i64, x: i64, z: i64) -> Core {
    let wx = (x + c / 2).rem_euclid(c) - c / 2;
    let rivers = src.rivers().map(|r| r.sample(wx, z)).unwrap_or_default();
    let surface = cell_surface(src, spec, wx, z);
    core_with_surface(src, spec, wx, z, surface, rivers)
}

/// How far below the sea's top a coastal bank is sealed, blocks.
pub const SEA_SEAL: i32 = 3;

/// The 4-neighbour offsets, in the order both readers visit them.
pub const NEIGHBOURS: [(i64, i64); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// The highest water block a water column holds once the pass has put in its steps: its own
/// `top`, or a higher water 4-neighbour's, whose flowing water the pass spreads into it
/// (`neighbours` in [`NEIGHBOURS`] order; ledger: HYD-124). Not water: `i32::MIN`.
pub fn held(own: &Core, neighbours: &[Core; 4]) -> i32 {
    if !own.kind.is_water() {
        return i32::MIN;
    }
    neighbours
        .iter()
        .filter(|n| n.kind.is_water())
        .fold(own.top, |m, n| m.max(n.top))
}

/// Step 5: the column from its own core and its four neighbours' (in [`NEIGHBOURS`] order), with
/// what each neighbour holds ([`held`]).
pub fn finish(own: &Core, neighbours: &[Core; 4], holds: &[i32; 4]) -> WaterColumn {
    let mut out = WaterColumn {
        kind: own.kind,
        surface: own.surface,
        ground: own.ground,
        level: own.level.is_finite().then_some(own.level),
        top: None,
        bed: None,
        min_ground: None,
        max_ground: None,
        seal_from: None,
        river_biome: own.rivers.biome,
        river: own.rivers.hit,
    };
    if own.kind.is_water() {
        out.top = Some(own.top);
        out.bed = Some(own.bed);
        return out;
    }
    // A one-block seam between two river channels is channel (ledger: HYD-182): where two
    // segments' corridors meet edge to edge, a column between them can fall outside both, and as
    // a bank it kept its ground above the water, a dotted line of single dry blocks across the
    // river. Between river columns on opposite sides it holds the lower of their waters on the
    // shallower of their beds.
    for k in [0, 2] {
        let (a, b) = (&neighbours[k], &neighbours[k + 1]);
        if a.kind != WaterKind::River || b.kind != WaterKind::River {
            continue;
        }
        let w = if b.top < a.top { b } else { a };
        out.kind = WaterKind::River;
        out.top = Some(w.top);
        out.bed = Some(a.bed.max(b.bed).min(w.top - 1));
        out.river_biome = a.rivers.biome || b.rivers.biome;
        out.river = w.rivers.hit;
        return out;
    }
    let mut min_ground: Option<i32> = None;
    let mut seal: Option<i32> = None;
    for (n, &hold) in neighbours.iter().zip(holds) {
        if !n.kind.is_water() {
            continue;
        }
        // Beside a river, the water it holds with its steps (ledger: HYD-124); beside a lake or
        // the sea, its top (HYD-077).
        let need = if n.kind == WaterKind::River {
            hold
        } else {
            n.top
        };
        min_ground = Some(min_ground.map_or(need, |m| m.max(need)));
        // Beside the sea only the waterline is sealed (cave mouths at the coast), not the whole
        // depth of the ocean.
        let from = if n.kind == WaterKind::Sea {
            (n.bed + 1).max(n.top - SEA_SEAL)
        } else {
            n.bed + 1
        };
        seal = Some(seal.map_or(from, |s| s.min(from)));
    }
    out.min_ground = min_ground;
    out.seal_from = seal;
    out.max_ground = own.rivers.bank_cap;
    if min_ground.is_some() || out.max_ground.is_some() {
        out.kind = WaterKind::Bank;
    }
    out
}

/// The whole evaluation of column (x, z) (any lap) on a planet of circumference `c`.
pub fn column(src: &dyn WaterSource, spec: &WaterSpec, c: i64, x: i64, z: i64) -> WaterColumn {
    let own = core(src, spec, c, x, z);
    let n = NEIGHBOURS.map(|(dx, dz)| core(src, spec, c, x + dx, z + dz));
    let mut holds = [i32::MIN; 4];
    if !own.kind.is_water() {
        // A river neighbour's own neighbours: the ring two blocks out, and this column.
        for (k, &(dx, dz)) in NEIGHBOURS.iter().enumerate() {
            if n[k].kind != WaterKind::River {
                continue;
            }
            let (nx, nz) = (x + dx, z + dz);
            let nn = NEIGHBOURS.map(|(ex, ez)| {
                if (nx + ex, nz + ez) == (x, z) {
                    own
                } else {
                    core(src, spec, c, nx + ex, nz + ez)
                }
            });
            holds[k] = held(&n[k], &nn);
        }
    }
    finish(&own, &n, &holds)
}

impl WaterColumn {
    /// The column's `hydro` object in `parity/columns.json`.
    pub fn to_json(&self) -> Value {
        let f = |v: f64| if v.is_finite() { json!(v) } else { Value::Null };
        json!({
            "kind": self.kind.name(),
            "surface": f(self.surface),
            "ground": self.ground,
            "level": self.level,
            "top": self.top,
            "bed": self.bed,
            "min_ground": self.min_ground,
            "max_ground": self.max_ground,
            "seal_from": self.seal_from,
            "river_biome": self.river_biome,
            "river": self.river.map(|h| json!({
                "seg": h.seg, "t": h.t, "d": h.d, "w": h.w, "hw": h.hw, "depth": h.depth,
            })),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rivers::{Chain, Mouth, RiverVertex};

    /// A plain at y 80 with a basin (level 90 over a floor at 70 for |x| < 20, |z| < 20), a
    /// channel along z = 100 from x = −50 (w 79) to x = 50 (w 75), the sea from z = 200, and a
    /// closed basin below sea level, its floor at 50, round (300, 100) (|dx|, |dz| < 24) with a
    /// lake at 55 in its middle (|dx|, |dz| < 8).
    struct Toy {
        rivers: Rivers,
    }

    fn in_closed(x: i64, z: i64, half: i64) -> bool {
        (x - 300).abs() < half && (z - 100).abs() < half
    }

    impl WaterSource for Toy {
        fn height(&self, x: i64, z: i64) -> f64 {
            if x.abs() < 24 && z.abs() < 24 {
                70.0
            } else if in_closed(x, z, 24) {
                50.0
            } else if z > 200 {
                40.0
            } else {
                80.0
            }
        }
        fn closed(&self, x: i64, z: i64) -> bool {
            in_closed(x, z, 32)
        }
        fn level(&self, x: i64, z: i64, _reach: i64) -> f64 {
            if x.abs() < 20 && z.abs() < 20 {
                90.0
            } else if in_closed(x, z, 8) {
                55.0
            } else {
                f64::NAN
            }
        }
        fn rivers(&self) -> Option<&Rivers> {
            Some(&self.rivers)
        }
    }

    fn toy() -> Toy {
        let mut r = Rivers::empty(1024);
        let v = |x: f32, w: f32| RiverVertex {
            x,
            z: 100.0,
            w,
            hw: 3.0,
            depth: 3.0,
            q: 50.0,
        };
        r.vertices = vec![v(-50.0, 79.0), v(50.0, 75.0)];
        r.chains = vec![Chain {
            first: 0,
            count: 2,
            mouth: Mouth::Sea,
            into: u32::MAX,
            from: u32::MAX,
        }];
        r.build_index();
        Toy { rivers: r }
    }

    #[test]
    fn a_one_block_seam_between_two_channels_is_channel() {
        // ledger: HYD-182. East and west are river columns of two segments whose corridors left this
        // column out: it takes the lower water (62) on the shallower bed (60), not a bank's ground.
        let river = |top: i32, bed: i32| Core {
            kind: WaterKind::River,
            surface: 64.4,
            ground: 64,
            level: f64::NAN,
            top,
            bed,
            rivers: RiverSample::default(),
        };
        let none = Core {
            kind: WaterKind::None,
            surface: 64.4,
            ground: 64,
            level: f64::NAN,
            top: i32::MIN,
            bed: i32::MIN,
            rivers: RiverSample::default(),
        };
        let seam = finish(&none, &[river(63, 57), river(62, 60), none, none], &[63, 63, i32::MIN, i32::MIN]);
        assert_eq!(seam.kind, WaterKind::River);
        assert_eq!((seam.top, seam.bed), (Some(62), Some(60)));
        // North and south (the second pair) count the same; one river side alone stays a bank.
        let ns = finish(&none, &[none, none, river(63, 58), river(63, 59)], &[i32::MIN, i32::MIN, 63, 63]);
        assert_eq!((ns.kind, ns.top, ns.bed), (WaterKind::River, Some(63), Some(59)));
        let bank = finish(&none, &[river(63, 57), none, none, none], &[63, i32::MIN, i32::MIN, i32::MIN]);
        assert_eq!(bank.kind, WaterKind::Bank);
    }

    #[test]
    fn kinds_tops_and_beds() {
        let t = toy();
        let s = WaterSpec::planet(63, -352, 639);
        let lake = column(&t, &s, 1024, 0, 0);
        assert_eq!(lake.kind, WaterKind::Lake);
        assert_eq!((lake.top, lake.bed), (Some(89), Some(69)));
        let river = column(&t, &s, 1024, 0, 100);
        assert_eq!(river.kind, WaterKind::River);
        // w 77 at x 0: top 76, centre depth 3 → bed ⌊74⌋ = 74.
        assert_eq!((river.top, river.bed), (Some(76), Some(74)));
        let edge = column(&t, &s, 1024, 0, 103);
        assert_eq!(edge.kind, WaterKind::River);
        assert_eq!(edge.bed, Some(75), "one block of water at the edge");
        let bank = column(&t, &s, 1024, 0, 104);
        assert_eq!(bank.kind, WaterKind::Bank);
        assert_eq!(
            bank.min_ground,
            Some(77),
            "what the neighbour holds: its upstream neighbour's top, a block higher (HYD-124)"
        );
        assert_eq!(bank.seal_from, Some(76));
        assert_eq!(bank.max_ground, Some(77 + 1));
        let sea = column(&t, &s, 1024, 0, 300);
        assert_eq!(sea.kind, WaterKind::Sea);
        assert_eq!(sea.top, Some(62));
        let dry = column(&t, &s, 1024, 300, 0);
        assert_eq!(dry.kind, WaterKind::None);
    }

    #[test]
    fn no_sea_stands_in_a_closed_basin() {
        // HYD-100: the floor at 50 lies below sea level, but the closed basin layer keeps the sea
        // off it: dry ground, and a lake whose top (54) stands below the sea's (62).
        let t = toy();
        let s = WaterSpec::planet(63, -352, 639);
        let lake = column(&t, &s, 1024, 300, 100);
        assert_eq!(lake.kind, WaterKind::Lake);
        assert_eq!((lake.top, lake.bed), (Some(54), Some(49)));
        let dry = column(&t, &s, 1024, 316, 100);
        assert_eq!((dry.kind, dry.top), (WaterKind::None, None));
        // The lake's shore: a bank up to the lake's top only, not the sea's.
        let shore = column(&t, &s, 1024, 308, 100);
        assert_eq!(shore.kind, WaterKind::Bank);
        assert_eq!(shore.min_ground, Some(54));
        // Outside the layer the old rule holds: ground below sea level is sea.
        let sea = column(&t, &s, 1024, 300, 300);
        assert_eq!((sea.kind, sea.top), (WaterKind::Sea, Some(62)));
    }

    #[test]
    fn the_sea_rule_is_one_function_and_the_column_evaluator_reads_it() {
        // HYD-106: the top block under sea level - 1, not closed. Sea level 63: ground 61 is sea,
        // 62 (a column whose top block is the sea's top) is dry.
        assert!(sea_covers(61, 63, || false));
        assert!(!sea_covers(62, 63, || false));
        assert!(
            !sea_covers(61, 63, || true),
            "the closed basin keeps it off"
        );
        // The closed flag is read only where the ground passes the first test.
        assert!(!sea_covers(70, 63, || panic!(
            "read where the ground is high"
        )));
        // Heights: H <= sea - 1 is sea (top block ceil(H) - 1 <= sea - 2), the band above it is not.
        assert!(sea_covers_height(62.0, 63, || false));
        assert!(sea_covers_height(61.5, 63, || false));
        assert!(
            !sea_covers_height(62.01, 63, || false),
            "ceil(62.01) - 1 = 62"
        );
        assert!(!sea_covers_height(63.0, 63, || false));
        assert!(!sea_covers_height(62.0, 63, || true));
        // The evaluator and the rule name the same columns on the toy world, over every kind of
        // ground it has (plain, basin, closed bowl, sea), closed or not.
        let t = toy();
        let s = WaterSpec::planet(63, -352, 639);
        for (x, z) in [
            (300, 300),
            (0, 300),
            (300, 100),
            (316, 100),
            (-100, 0),
            (60, 150),
        ] {
            let c = core(&t, &s, 1024, x, z);
            let by_rule = sea_covers_height(t.height(x, z), 63, || t.closed(x, z));
            let lake = c.kind == WaterKind::Lake;
            assert!(
                lake || (c.kind == WaterKind::Sea) == by_rule,
                "({x}, {z}): {:?} vs the rule {by_rule}",
                c.kind
            );
        }
    }

    /// A wide river's lower reach (HYD-120 to HYD-122): one chain along z = 0 from x = −100 to 50,
    /// a vertex every 10 blocks, half-width 12 and depth 8, w falling from 64 to 63 at x = 0 and
    /// flat at 63 (the sea's level) beyond, on a plain at 64; the sea (ground 57) from x = 44, a
    /// lake less than a block deep (level 64.5 over 63.6) for x in [−60, −40) and one 4.5 blocks
    /// deep (over 60) for x in [−92, −72), both |z| < 32.
    struct Reach {
        rivers: Rivers,
    }

    impl WaterSource for Reach {
        fn height(&self, x: i64, z: i64) -> f64 {
            if x >= 44 {
                58.0
            } else if (-60..-40).contains(&x) && z.abs() < 32 {
                63.6
            } else if (-92..-72).contains(&x) && z.abs() < 32 {
                60.0
            } else {
                64.0
            }
        }
        fn level(&self, x: i64, z: i64, _reach: i64) -> f64 {
            if ((-60..-40).contains(&x) || (-92..-72).contains(&x)) && z.abs() < 32 {
                64.5
            } else {
                f64::NAN
            }
        }
        fn rivers(&self) -> Option<&Rivers> {
            Some(&self.rivers)
        }
    }

    fn reach() -> Reach {
        let mut r = Rivers::empty(1024);
        r.vertices = (0..16)
            .map(|i| {
                let x = -100.0 + 10.0 * i as f32;
                RiverVertex {
                    x,
                    z: 0.0,
                    w: if x < 0.0 { 63.0 - x / 100.0 } else { 63.0 },
                    hw: 12.0,
                    depth: 8.0,
                    q: 1000.0,
                }
            })
            .collect();
        r.chains = vec![Chain {
            first: 0,
            count: 16,
            mouth: Mouth::Sea,
            into: u32::MAX,
            from: u32::MAX,
        }];
        r.build_index();
        Reach { rivers: r }
    }

    #[test]
    fn a_channel_keeps_its_bed_where_segments_overlap_at_sea_level_and_into_the_sea() {
        let t = reach();
        let s = WaterSpec::planet(63, -352, 639);
        // (9, 0) lies under three segments, all at w 63: the tie goes to the lowest id, the one
        // ending at x 0, 9 blocks off its centre line (its parabola is 3.5 deep there); the
        // segment the column sits on is 8 deep. The bed is the deepest floor (HYD-120).
        let c = column(&t, &s, 1024, 9, 0);
        let h = c.river.expect("a channel covers it");
        assert_eq!(
            (h.seg, h.d),
            (9, 9.0),
            "the hit is still the lowest-w, lowest-id segment"
        );
        assert_eq!(c.kind, WaterKind::River);
        assert_eq!(
            (c.top, c.bed),
            (Some(62), Some(55)),
            "⌊63 − 8⌋, not ⌊63 − 3.5⌋"
        );
        // Upstream, the sloping reach: w 63.55 at x −55 would be lake (see below); at x −25,
        // w 63.25, top block 63, level with the plain's ground (63) at the banks: water and bed go
        // a block down (HYD-123), the bed to ⌊63.25 − 8⌋ − 1.
        let up = column(&t, &s, 1024, -25, 0);
        assert_eq!(
            (up.kind, up.top, up.bed),
            (WaterKind::River, Some(62), Some(54))
        );
        // Into the sea: the channel runs on under the sea's top across ground at 57 (HYD-121).
        let sea = column(&t, &s, 1024, 46, 0);
        assert_eq!(sea.kind, WaterKind::Sea);
        assert_eq!((sea.top, sea.bed), (Some(62), Some(55)));
        // The channel's edge in the sea: one block under the water surface at most, never above
        // the sea's own floor.
        let edge = column(&t, &s, 1024, 46, 12);
        assert_eq!((edge.kind, edge.bed), (WaterKind::Sea, Some(57)));
        // The sea off the channel keeps its floor.
        let open = column(&t, &s, 1024, 46, 40);
        assert_eq!((open.kind, open.bed), (WaterKind::Sea, Some(57)));
        // A lake less than a block deep the river crosses: the channel's bed (HYD-122).
        let shallow = column(&t, &s, 1024, -50, 0);
        assert_eq!(shallow.kind, WaterKind::Lake);
        assert_eq!((shallow.top, shallow.bed), (Some(64), Some(55)));
        // Off the channel the shallow lake keeps its floor.
        let beside = column(&t, &s, 1024, -50, 24);
        assert_eq!((beside.kind, beside.bed), (WaterKind::Lake, Some(63)));
        // A lake 4.5 blocks deep: the river ends in it and does not trench its floor.
        let deep = column(&t, &s, 1024, -80, 0);
        assert_eq!(deep.kind, WaterKind::Lake);
        assert_eq!((deep.top, deep.bed), (Some(64), Some(59)));
        assert!(deep.river.is_some(), "a channel does cover it");
    }

    /// A stream on a plain at 64 (ground 63): one chain along z = 0 from x = −40 to 40, a vertex
    /// every 8 blocks, half-width 1 and depth 1, w falling from 64.9 to 63.1 (its top block from
    /// 64 to 63, at or above the plain's ground: the bake's water flush with or over its banks),
    /// across a dip to 60 for |x| < 8.
    struct Stream {
        rivers: Rivers,
    }

    impl WaterSource for Stream {
        fn height(&self, x: i64, _z: i64) -> f64 {
            if x.abs() < 8 { 60.0 } else { 64.0 }
        }
        fn level(&self, _x: i64, _z: i64, _reach: i64) -> f64 {
            f64::NAN
        }
        fn rivers(&self) -> Option<&Rivers> {
            Some(&self.rivers)
        }
    }

    fn stream() -> Stream {
        let mut r = Rivers::empty(1024);
        r.vertices = (0..11)
            .map(|i| RiverVertex {
                x: -40.0 + 8.0 * i as f32,
                z: 0.0,
                w: 64.9 - 0.18 * i as f32,
                hw: 1.0,
                depth: 1.0,
                q: 1.0,
            })
            .collect();
        r.chains = vec![Chain {
            first: 0,
            count: 11,
            mouth: Mouth::Sea,
            into: u32::MAX,
            from: u32::MAX,
        }];
        r.build_index();
        Stream { rivers: r }
    }

    #[test]
    fn a_flush_stream_is_cut_in_and_gets_no_curb() {
        // HYD-123, HYD-124: a stream whose water the bake puts level with its banks or over them
        // is cut a block into the plain, and no bank column on the plain is raised above its
        // ground; where the stream crosses a real dip its banks are still built up (a levee).
        let t = stream();
        let s = WaterSpec::planet(63, -352, 639);
        let mut banks = 0;
        let mut old_raised = 0;
        for x in -36..=36 {
            // The water surface over column x (w falls linearly along the chain).
            let w = 64.9 - 0.18 * (x + 40) as f64 / 8.0;
            for z in -4..=4 {
                let c = column(&t, &s, 1024, x, z);
                if c.kind == WaterKind::River {
                    if x.abs() >= 12 {
                        assert_eq!(c.top, Some(top_block(w) - 1), "({x}, {z}): cut a block in");
                    }
                    continue;
                }
                let Some(m) = c.min_ground else { continue };
                if x.abs() >= 12 {
                    banks += 1;
                    assert!(
                        m <= c.ground,
                        "({x}, {z}): bank raised to {m} over its ground {}",
                        c.ground
                    );
                    // The old rule, the river's uncut top + 1: over the ground everywhere here.
                    old_raised += i32::from(top_block(w) + 1 > c.ground);
                } else if x.abs() <= 3 {
                    assert!(m > c.ground, "({x}, {z}): the dip's banks are built up");
                }
            }
        }
        assert_eq!(banks, 100, "both banks of 50 plain columns");
        assert_eq!(old_raised, banks, "the old rule raised every one");
        // Upstream of x 0 (where w crosses 64, in the dip) the water's top is the plain's ground
        // top, flush; downstream a block under it.
        let up = column(&t, &s, 1024, -30, 0);
        assert_eq!(
            (up.kind, up.top, up.bed),
            (WaterKind::River, Some(63), Some(62))
        );
        let down = column(&t, &s, 1024, 30, 0);
        assert_eq!(
            (down.kind, down.top, down.bed),
            (WaterKind::River, Some(62), Some(61))
        );
    }

    #[test]
    fn the_seam_and_laps_agree() {
        let t = toy();
        let s = WaterSpec::planet(63, -352, 639);
        for (x, z) in [(0, 0), (0, 100), (3, 104), (-511, 7), (511, 100)] {
            let a = column(&t, &s, 1024, x, z);
            assert_eq!(a, column(&t, &s, 1024, x + 1024, z));
            assert_eq!(a, column(&t, &s, 1024, x - 2048, z));
        }
    }

    #[test]
    fn the_surface_is_bilinear_between_cell_corners() {
        let t = toy();
        let s = WaterSpec::planet(63, -352, 639);
        // x 22 lies in the cell [20, 24): corners at 70 (x 20) and 80 (x 24).
        let h = cell_surface(&t, &s, 22, 0);
        assert!((h - 75.0).abs() < 1e-12);
    }
}

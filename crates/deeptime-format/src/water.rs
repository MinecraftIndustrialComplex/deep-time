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
//!    `ground` < `top` = ⌈level⌉ − 1 the column holds lake water from `ground` + 1 to `top`.
//! 3. **Sea.** Otherwise, where `ground` < sea level − 1, the sea covers it (`top` = sea level − 1):
//!    the aquifer's water, which the game's pass tops up where the aquifer left the column dry.
//! 4. **River.** Otherwise, where a channel covers the column ([`crate::rivers::Rivers::sample`]),
//!    the water surface w gives `top` = ⌈w⌉ − 1 and the bed is cut (or filled) to
//!    `bed` = min(top − 1, ⌊w − max(1, depth · (1 − (d / hw)²))⌋): a parabolic cross-section, at
//!    least one block of water.
//! 5. **Bank.** A column holding none of these is a bank when a 4-neighbour holds river or lake
//!    water (the sea is the aquifer's, which vanilla's own rules keep in place): its ground
//!    must reach `min_ground` = the highest neighbour `top` (+1 beside a river: the freeboard that
//!    keeps a river's one-block steps inside its banks), and air from `seal_from` (one above the
//!    lowest neighbour bed) up to `min_ground` is sealed, so no water finds a way out. Within a
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
    let ground = if surface.is_finite() {
        top_block(surface).clamp(spec.min_y - 1, spec.max_y)
    } else {
        spec.min_y - 1
    };
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
    if level.is_finite() {
        let t = top_block(level);
        if ground < t {
            out.kind = WaterKind::Lake;
            out.top = t;
            out.bed = ground;
            return out;
        }
    }
    let sea_top = spec.sea_level_y - 1;
    if ground < sea_top {
        out.kind = WaterKind::Sea;
        out.top = sea_top;
        out.bed = ground;
        return out;
    }
    if let Some(h) = rivers.hit {
        let t = top_block(h.w);
        let r = h.d / h.hw;
        let dp = (h.depth * (1.0 - r * r)).max(1.0);
        out.kind = WaterKind::River;
        out.top = t;
        out.bed = ((h.w - dp).floor() as i32).min(t - 1);
    }
    out
}

/// Steps 1–4 for column (x, z), any lap.
pub fn core(src: &dyn WaterSource, spec: &WaterSpec, c: i64, x: i64, z: i64) -> Core {
    let wx = (x + c / 2).rem_euclid(c) - c / 2;
    let rivers = src.rivers().map(|r| r.sample(wx, z)).unwrap_or_default();
    let surface = cell_surface(src, spec, wx, z);
    core_with_surface(src, spec, wx, z, surface, rivers)
}

/// The 4-neighbour offsets, in the order both readers visit them.
pub const NEIGHBOURS: [(i64, i64); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Step 5: the column from its own core and its four neighbours' (in [`NEIGHBOURS`] order).
pub fn finish(own: &Core, neighbours: &[Core; 4]) -> WaterColumn {
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
    let mut min_ground: Option<i32> = None;
    let mut seal: Option<i32> = None;
    for n in neighbours {
        if !matches!(n.kind, WaterKind::River | WaterKind::Lake) {
            continue;
        }
        let need = n.top + i32::from(n.kind == WaterKind::River);
        min_ground = Some(min_ground.map_or(need, |m| m.max(need)));
        let from = n.bed + 1;
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
    finish(&own, &n)
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

    /// A plain at y 80 with a basin (level 90 over a floor at 70 for |x| < 20, |z| < 20) and a
    /// channel along z = 100 from x = −50 (w 79) to x = 50 (w 75).
    struct Toy {
        rivers: Rivers,
    }

    impl WaterSource for Toy {
        fn height(&self, x: i64, z: i64) -> f64 {
            if x.abs() < 24 && z.abs() < 24 {
                70.0
            } else if z > 200 {
                40.0
            } else {
                80.0
            }
        }
        fn level(&self, x: i64, z: i64, _reach: i64) -> f64 {
            if x.abs() < 20 && z.abs() < 20 {
                90.0
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
        }];
        r.build_index();
        Toy { rivers: r }
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
        assert_eq!(bank.min_ground, Some(77), "the neighbour's top + 1");
        assert_eq!(bank.seal_from, Some(76));
        assert_eq!(bank.max_ground, Some(77 + 1));
        let sea = column(&t, &s, 1024, 0, 300);
        assert_eq!(sea.kind, WaterKind::Sea);
        assert_eq!(sea.top, Some(62));
        let dry = column(&t, &s, 1024, 300, 0);
        assert_eq!(dry.kind, WaterKind::None);
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

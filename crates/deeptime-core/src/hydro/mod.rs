// SPDX-License-Identifier: LGPL-3.0-or-later
//! Rivers and lakes (plan §8.4, work item P1-22, moved from P2-03 by Ben's decision of 2026-09-29,
//! "Rivers & lakes in v1"): the drainage of the finished terrain, the lakes it fills and the river
//! network it carries, in block space, for the bake to write and the game to carve.
//!
//! The bake ([`crate::bake::water`]) runs [`network::run`] on the L3 terrain (the amplifier's
//! `elevation`, 2 blocks a cell on every preset but 32k full, which is averaged to 4 blocks so the
//! working set stays under ~600 MB) with the climate's runoff:
//!
//! 1. **Drainage.** The open sea is every cell at least a block below sea level in a connected body of at least
//!    [`OPEN_SEA_KM2`]; a Priority-Flood (Barnes, Lehman & Mulla 2014, Algorithm 3 with the pit
//!    queue of Algorithm 4, implemented from the paper as `surface::pflood` is) from its shores
//!    (and the first and last rows, the poles past the hard cap, as outlets at their own height)
//!    gives every land cell its spill level F and the cell it was reached from. Each cell drains to
//!    its steepest strictly lower neighbour on F, else (flats and lakes) to the cell it was
//!    reached from, so every path descends to the sea, and the receivers form a forest.
//! 2. **Lakes: a water balance** (Ben, 2026-10-01, "freeze them and size them by climate"; the
//!    lake part of P2-03). A connected set of cells below their F is a basin; one of at least
//!    `lake_min_area_km2` and `lake_min_depth_blocks` is a candidate lake. Its level is where what
//!    flows in balances what its surface evaporates ([`HydroParams::budget`]):
//!    - *It fills.* If the basin, wet to its rim, still gains water (inflow + rain on it ≥ its
//!      evaporation), the lake is full to its lowest rim and spills the surplus through it, as
//!      every lake did before.
//!    - *It cannot fill.* The basin is terminal (endorheic). Its lowest cell becomes a sink, the
//!      basin's floor is drained to it (a Priority-Flood of the basin alone, so the streams on the
//!      exposed floor follow the real ground and the basins inside it are treated the same way,
//!      upstream first), and the lake grows from the sink over the basin's **hypsometry** (the
//!      loss a cell adds when it goes under, summed by height: no iteration over the grid) to the
//!      level where the losses take the whole inflow. A lake inside the basin that the terminal
//!      lake rises over becomes part of it.
//!    - *It is too thin.* A balance lake under the lake thresholds is a **playa** (a salt flat:
//!      water that reaches it evaporates off the floor); with no inflow at all the basin is dry.
//!    - *It lies below sea level.* A basin whose spill level is at most sea level is open to the
//!      sea: the sea reaches it at its own level ([`sea_reached`]), so a terminal one stands at
//!      sea level at least, as the column rule's sea does in the game. A closed basin, behind a
//!      rim above sea level, takes its balance like any other (HYD-100, 2026-10-20; before it
//!      every basin below sea level was raised to sea level, HYD-052): a lake whose surface may
//!      stand below sea level, a playa, or dry ground. The `closed_basin` layer tells the game where
//!      the sea does not stand ([`closed_basin_layer`]).
//!
//!    An upstream lake that does not spill passes nothing on, so a chain of lakes is solved in
//!    order. `lake_balance` false keeps every lake full, as before.
//! 3. **Discharge.** The liquid runoff (rain and melt less ET: Fu's Budyko curve as the surface
//!    stage, from the climate stage's temperature and precipitation) accumulated down the
//!    receivers: Q in m³/s. Where the warmest month stays below freezing nothing melts, so nothing
//!    runs off: frozen ground feeds no river and no lake.
//!
//! **Frozen lakes.** A lake cell whose mean annual temperature is below `lake_freeze_mean_c`
//! (0 °C: with a yearly cycle, ice for more than half the year) is ice-covered: the views draw it
//! as ice, and in the game it lies in the frozen biomes, whose `freeze_top_layer` ices it.
//! 4. **Rivers.** Cells carrying at least `river_min_discharge_m3s` from at least
//!    `river_min_area_km2` are river cells. They are cut into chains at confluences (a chain
//!    follows its largest donor), each ending in the sea, a lake or a larger river; the vertices
//!    are smoothed (a Gaussian of half-width `smoothing_blocks`) to take the grid's 45° steps out. The water
//!    surface at each vertex is F − `freeboard_blocks`, never below the level of the sea or lake
//!    downstream: it never rises downstream. Width and depth follow Leopold & Maddock's hydraulic
//!    geometry, W = a·Q^0.5 and D = c·Q^0.4, scaled for the block world (`width_coef`,
//!    `depth_coef`; × `scale.feature_scale.rivers`), at least one block each.
//!
//! What the game does with it is `deeptime_format::water` (and FORMAT.md, "Water columns").

pub mod network;

#[cfg(test)]
mod tests;

use deeptime_format::GridDesc;
use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// The smallest body of below-sea-level cells that counts as open sea, km² (smaller ones are
/// closed basins: lakes if they are deep enough).
pub const OPEN_SEA_KM2: f64 = 5_000.0;

/// The largest hydrology grid, cells; a larger L3 is averaged down by 2 in each direction.
pub const MAX_CELLS: usize = 40_000_000;

/// How far the `closed_basin` layer carries past a closed basin's low ground, blocks (ledger:
/// HYD-102, game-choice: agent, 2026-10-20). The game's ground is H = E + A·N, bilinear over 4-block
/// noise cells, so beside low ground it can dip under sea level where the bake's E stands above
/// it: such a column takes the side of the nearest ground at or below sea level within this
/// distance (two noise cells), the open sea's on a tie.
pub const CLOSED_REACH_BLOCKS: f64 = 8.0;

/// The 8 neighbours of cell `c` on a `w` × `h` grid that wraps in x (rows do not).
#[inline]
fn neighbour8(w: usize, h: usize, c: usize, k: usize) -> Option<usize> {
    const DX: [i64; 8] = [1, -1, 0, 0, 1, -1, 1, -1];
    const DZ: [i64; 8] = [0, 0, 1, -1, 1, 1, -1, -1];
    let (i, j) = ((c % w) as i64, (c / w) as i64);
    let nj = j + DZ[k];
    if nj < 0 || nj >= h as i64 {
        return None;
    }
    Some(nj as usize * w + (i + DX[k]).rem_euclid(w as i64) as usize)
}

/// The open sea of a `w` × `h` grid (x wraps): every cell where `deep` holds in an 8-connected body
/// of at least `min_cells` (the hydrology's own rule, [`OPEN_SEA_KM2`]).
pub fn open_sea_mask(
    w: usize,
    h: usize,
    deep: impl Fn(usize) -> bool,
    min_cells: usize,
) -> Vec<bool> {
    let n = w * h;
    let mut done = vec![false; n];
    let mut open = vec![false; n];
    let mut members: Vec<usize> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    for c0 in 0..n {
        if done[c0] || !deep(c0) {
            continue;
        }
        members.clear();
        stack.push(c0);
        done[c0] = true;
        while let Some(c) = stack.pop() {
            members.push(c);
            for k in 0..8 {
                if let Some(nb) = neighbour8(w, h, c, k)
                    && !done[nb]
                    && deep(nb)
                {
                    done[nb] = true;
                    stack.push(nb);
                }
            }
        }
        if members.len() >= min_cells {
            for &c in &members {
                open[c] = true;
            }
        }
    }
    open
}

/// The ground the open sea reaches at its own level (ledger: HYD-100): the `open` cells and every
/// cell joined to them, 8-connected (x wraps), through cells where `low` holds (ground at or below
/// sea level). It is the ground whose spill level towards the sea is at most sea level; a closed
/// basin's floor below sea level, behind a rim above it, is not reached, and no sea stands there.
pub fn sea_reached(w: usize, h: usize, open: &[bool], low: impl Fn(usize) -> bool) -> Vec<bool> {
    sea_reached_by(w * h, |c, k| neighbour8(w, h, c, k), open, low)
}

/// [`sea_reached`] on any grid of `n` cells whose neighbour `k` (0..8) of cell `c` is `nb(c, k)`
/// (a cube world's grid steps across its seams, SCL-122).
pub fn sea_reached_by(
    n: usize,
    nb: impl Fn(usize, usize) -> Option<usize>,
    open: &[bool],
    low: impl Fn(usize) -> bool,
) -> Vec<bool> {
    let mut reached = open.to_vec();
    // The open cells one at a time, each flooded to the end before the next: the same ground
    // (what any open cell reaches), and the stack holds only the low cells a flood found, not
    // every open cell at once (amp-est, 2026-10-23: the ocean's cells as usize, 134 MB on quick
    // 16k's world grid and twice that on an ocean past 2^24 cells).
    let mut stack: Vec<usize> = Vec::new();
    for c0 in (0..n).filter(|&c| open[c]) {
        stack.push(c0);
        while let Some(c) = stack.pop() {
            for k in 0..8 {
                if let Some(nb) = nb(c, k)
                    && !reached[nb]
                    && low(nb)
                {
                    reached[nb] = true;
                    stack.push(nb);
                }
            }
        }
    }
    reached
}

/// The `closed_basin` layer (ledger: HYD-100, HYD-102): 1 where the game's column rule puts no sea
/// (ground under sea level that the open sea does not reach, and ground within `radius` cells
/// nearer to it than to the open sea's), 0 elsewhere, where the old rule holds (the sea stands on
/// any ground under sea level). `low` is ground at or below sea level, `reached` what
/// [`sea_reached`] gave. A cell takes the side of the nearest low cell (8-connected steps), the
/// open sea's on a tie; a cell with none within `radius` gets 0.
pub fn closed_basin_layer(
    w: usize,
    h: usize,
    low: impl Fn(usize) -> bool,
    reached: &[bool],
    radius: usize,
) -> Vec<u8> {
    closed_basin_layer_by(w * h, |c, k| neighbour8(w, h, c, k), low, reached, radius)
}

/// [`closed_basin_layer`] on any grid of `n` cells with neighbours `nb` ([`sea_reached_by`]).
pub fn closed_basin_layer_by(
    n: usize,
    nb: impl Fn(usize, usize) -> Option<usize>,
    low: impl Fn(usize) -> bool,
    reached: &[bool],
    radius: usize,
) -> Vec<u8> {
    use std::collections::VecDeque;
    const OPEN: u8 = 1;
    const CLOSED: u8 = 2;
    let mut side = vec![0u8; n];
    for c in 0..n {
        if low(c) {
            side[c] = if reached[c] { OPEN } else { CLOSED };
        }
    }
    // From the edges of the low ground only, the sea's first: a breadth-first search keeps that
    // order within each distance, so a tie goes to the sea.
    let mut queue: VecDeque<(u32, u16)> = VecDeque::new();
    for want in [OPEN, CLOSED] {
        for c in 0..n {
            if side[c] == want && (0..8).any(|k| nb(c, k).is_some_and(|m| side[m] == 0)) {
                queue.push_back((c as u32, 0));
            }
        }
    }
    while let Some((c, d)) = queue.pop_front() {
        if usize::from(d) >= radius {
            continue;
        }
        let s = side[c as usize];
        for k in 0..8 {
            if let Some(m) = nb(c as usize, k)
                && side[m] == 0
            {
                side[m] = s;
                queue.push_back((m as u32, d + 1));
            }
        }
    }
    side.iter().map(|&s| u8::from(s == CLOSED)).collect()
}

/// The `closed_basin` layer on the world's own grid (ledger: HYD-105). The bake's layer comes from
/// the hydrology's grid (the simulation's Lambert grid) by the nearest cell, while the world's
/// ground is a footprint mean of the Lambert ground: a narrow sill that held a pocket of sea floor
/// apart from the ocean there can be gone here, and the game would leave the pocket a dry hole
/// beside the sea. Here a low cell (`low`: ground at or below sea level) keeps the bake's closed
/// flag only where the world's own open sea does not reach it. The open sea is every 8-connected
/// body of `deep` cells (x wraps) holding at least `min_open` cells the bake left open (`flag` 0),
/// so a closed basin's few edge cells that took an open neighbour's flag never open it, and the
/// ocean opens every pocket it joins. The halo is drawn again on this grid ([`closed_basin_layer`],
/// `radius` cells). Returns the layer and the number of low cells whose closed flag was cleared.
pub fn closed_basin_on_world(
    w: usize,
    h: usize,
    low: impl Fn(usize) -> bool,
    deep: impl Fn(usize) -> bool,
    flag: &[u8],
    min_open: usize,
    radius: usize,
) -> (Vec<u8>, u64) {
    closed_basin_on_world_by(
        w * h,
        |c, k| neighbour8(w, h, c, k),
        low,
        deep,
        flag,
        min_open,
        radius,
    )
}

/// [`closed_basin_on_world`] on any grid of `n` cells with neighbours `nb` ([`sea_reached_by`]):
/// a cube world's grid, whose open sea and basins run across the seams (SCL-122).
pub fn closed_basin_on_world_by(
    n: usize,
    nb: impl Fn(usize, usize) -> Option<usize>,
    low: impl Fn(usize) -> bool,
    deep: impl Fn(usize) -> bool,
    flag: &[u8],
    min_open: usize,
    radius: usize,
) -> (Vec<u8>, u64) {
    assert_eq!(
        flag.len(),
        n,
        "the closed_basin flags do not match the grid"
    );
    // A cell is new, met by the body being flooded, or done (its body decided).
    const NEW: u8 = 0;
    const MET: u8 = 1;
    const DONE: u8 = 2;
    assert!(
        u32::try_from(n).is_ok(),
        "a world grid of 2^32 cells or more"
    );
    let mut state = vec![NEW; n];
    let mut open = vec![false; n];
    // Breadth first, by u32 index: the queue holds a flood's front, not its body. Each body is
    // flooded twice, the second time through its own cells only, to mark it done and open or not;
    // the cells are the same, since each was met from one of them (amp-est, 2026-10-23: the body's
    // cells and a depth-first stack, as usize, held 380 MB on quick 16k's world grid of 22.2 M
    // cells and 648 MB where the ocean passed 2^24 cells; the bake's peak).
    let mut queue: std::collections::VecDeque<u32> = std::collections::VecDeque::new();
    for c0 in 0..n {
        if state[c0] != NEW || !deep(c0) {
            continue;
        }
        state[c0] = MET;
        queue.push_back(c0 as u32);
        let mut open_cells = 0usize;
        while let Some(c) = queue.pop_front() {
            let c = c as usize;
            open_cells += usize::from(flag[c] == 0);
            for k in 0..8 {
                if let Some(m) = nb(c, k)
                    && state[m] == NEW
                    && deep(m)
                {
                    state[m] = MET;
                    queue.push_back(m as u32);
                }
            }
        }
        let is_open = open_cells >= min_open;
        state[c0] = DONE;
        open[c0] = is_open;
        queue.push_back(c0 as u32);
        while let Some(c) = queue.pop_front() {
            for k in 0..8 {
                if let Some(m) = nb(c as usize, k)
                    && state[m] == MET
                {
                    state[m] = DONE;
                    open[m] = is_open;
                    queue.push_back(m as u32);
                }
            }
        }
    }
    drop(state);
    drop(queue);
    let reached = sea_reached_by(n, &nb, &open, &low);
    drop(open);
    let mut cleared = 0u64;
    // The side each low cell takes: closed only where the bake said so and this ground's sea does
    // not reach it.
    let not_closed: Vec<bool> = (0..n)
        .map(|c| {
            let closed = flag[c] == 1 && !reached[c];
            cleared += u64::from(low(c) && flag[c] == 1 && reached[c]);
            !closed
        })
        .collect();
    drop(reached);
    (
        closed_basin_layer_by(n, &nb, &low, &not_closed, radius),
        cleared,
    )
}

/// The 8 neighbours of a cube grid's stacked cell `c` (FORMAT.md, "Cube grids"): the same steps as
/// [`neighbour8`], taken in the cell's own face and resolved across its seams by
/// [`GridDesc::face_cell`] (the corner squares give the diagonal ghost). No step is ever off the
/// grid: the cube has no edge (SCL-122).
pub fn cube_neighbour8(grid: &GridDesc) -> impl Fn(usize, usize) -> Option<usize> + '_ {
    const DX: [i64; 8] = [1, -1, 0, 0, 1, -1, 1, -1];
    const DZ: [i64; 8] = [0, 0, 1, -1, 1, 1, -1, -1];
    let w = grid.w as usize;
    move |c, k| {
        let (face, a, b) = grid.face_of_cell((c % w) as u32, (c / w) as u32);
        let (i, j) = grid.face_cell(face, i64::from(a) + DX[k], i64::from(b) + DZ[k]);
        Some(j as usize * w + i as usize)
    }
}

/// The closed basins below sea level of an elevation field in metres (ledger: HYD-100): every
/// cell at or below `sea_m` that the open sea does not reach, by the hydrology's rules
/// ([`open_sea_mask`]: ground a block or more under sea level in a body of [`OPEN_SEA_KM2`];
/// [`sea_reached`]). The biome bake reads it on L2, where the water bake's own (on L3) is not
/// made yet: such ground is land to the biomes, as it is dry ground, a playa or a lake in the game.
pub fn closed_below_sea(
    grid: &GridDesc,
    elevation_m: &[f32],
    sea_m: f64,
    m_per_block_v: f64,
    m_per_block_h: f64,
) -> Vec<bool> {
    let (w, h) = (grid.w as usize, grid.h as usize);
    let low = |c: usize| f64::from(elevation_m[c]) <= sea_m;
    let deep = |c: usize| f64::from(elevation_m[c]) <= sea_m - m_per_block_v;
    let cell_km = f64::from(grid.r) * m_per_block_h / 1000.0;
    let min_cells = ((OPEN_SEA_KM2 / (cell_km * cell_km)).ceil() as usize).max(1);
    let open = open_sea_mask(w, h, deep, min_cells);
    let reached = sea_reached(w, h, &open, low);
    (0..w * h).map(|c| low(c) && !reached[c]).collect()
}

/// Potential evapotranspiration per degree of mean temperature above freezing, mm/yr/°C (the
/// surface stage's stand-in, `surface::climate`).
pub const PET_MM_PER_C: f64 = 60.0;

/// Days in a year (for the degree-day melt).
const YEAR_DAYS: f64 = 365.25;

/// The climate of a cell, as the hydrology reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Climate {
    /// Annual precipitation, mm/yr.
    pub precip_mm: f64,
    /// Mean annual temperature, °C.
    pub t_mean_c: f64,
    /// Mean temperature of the warmest and the coldest month, °C.
    pub t_warm_c: f64,
    pub t_cold_c: f64,
}

impl Climate {
    /// A climate with no seasons (tests, stand-ins).
    pub fn steady(precip_mm: f64, t_mean_c: f64) -> Climate {
        Climate {
            precip_mm,
            t_mean_c,
            t_warm_c: t_mean_c,
            t_cold_c: t_mean_c,
        }
    }

    /// From a mean and the warmest and coldest months, any of which may be missing (not finite):
    /// a missing mean is 0 °C, missing months are the mean ± 8 °C (as the biome mapping assumes).
    pub fn of(precip_mm: f64, t_mean_c: f64, t_warm_c: f64, t_cold_c: f64) -> Climate {
        let t = if t_mean_c.is_finite() { t_mean_c } else { 0.0 };
        Climate {
            precip_mm: if precip_mm.is_finite() {
                precip_mm.max(0.0)
            } else {
                0.0
            },
            t_mean_c: t,
            t_warm_c: if t_warm_c.is_finite() {
                t_warm_c.max(t)
            } else {
                t + 8.0
            },
            t_cold_c: if t_cold_c.is_finite() {
                t_cold_c.min(t)
            } else {
                t - 8.0
            },
        }
    }

    /// The year's mean of the temperature above freezing, °C, and the share of the year above
    /// freezing, for a sinusoidal year between the coldest and the warmest month (with no month
    /// below freezing: the mean annual temperature and 1).
    pub fn thaw(&self) -> (f64, f64) {
        let a = 0.5 * (self.t_warm_c - self.t_cold_c).max(0.0);
        let m = 0.5 * (self.t_warm_c + self.t_cold_c);
        if m - a >= 0.0 {
            (self.t_mean_c.max(0.0), 1.0)
        } else if m + a <= 0.0 {
            (0.0, 0.0)
        } else {
            // T = m + a·cos θ is above 0 for |θ| < θ0.
            let th = (-m / a).clamp(-1.0, 1.0).acos();
            (
                ((m * th + a * th.sin()) / std::f64::consts::PI).max(0.0),
                th / std::f64::consts::PI,
            )
        }
    }

    /// The aridity index P / PET (PET from the temperature above freezing), capped at 10.
    pub fn aridity(&self) -> f64 {
        let pet = PET_MM_PER_C * self.thaw().0;
        if pet <= 0.0 {
            10.0
        } else {
            (self.precip_mm / pet).min(10.0)
        }
    }
}

/// The water a cell gives and takes, mm/yr.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Budget {
    /// What runs off it as land.
    pub runoff_mm: f64,
    /// What falls on it as open water (the liquid part of the precipitation).
    pub rain_mm: f64,
    /// What evaporates from it as open water.
    pub evaporation_mm: f64,
}

/// Hydraulic geometry exponents (Leopold & Maddock 1953: width ∝ Q^0.5, depth ∝ Q^0.4).
pub const WIDTH_EXPONENT: f64 = 0.5;
pub const DEPTH_EXPONENT: f64 = 0.4;

/// The `hydro` section of a preset (every key optional).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct HydroParams {
    /// Rivers and lakes at all (false: `water_level` all nodata and no river network, as before
    /// P1-22).
    pub enabled: bool,
    /// Discharge from which a stream is drawn, m³/s.
    pub river_min_discharge_m3s: f64,
    /// Drainage area from which a stream is drawn, km².
    pub river_min_area_km2: f64,
    /// Channel width in blocks per (m³/s)^0.5, before `scale.feature_scale.rivers`.
    pub width_coef: f64,
    /// Channel depth in blocks per (m³/s)^0.4.
    pub depth_coef: f64,
    /// The widest channel, blocks.
    pub max_width_blocks: f64,
    /// The deepest channel, blocks.
    pub max_depth_blocks: f64,
    /// How far the water surface sits below the valley floor it follows, blocks.
    pub freeboard_blocks: f64,
    /// The half-width of the kernel that smooths river courses, blocks.
    pub smoothing_blocks: f64,
    /// The smallest lake, km².
    pub lake_min_area_km2: f64,
    /// The shallowest lake at its deepest point, blocks.
    pub lake_min_depth_blocks: f64,
    /// Whether evaporation from a lake's surface reduces what flows out of it.
    pub lake_evaporation: bool,
    /// Potential evapotranspiration per °C of biotemperature, mm/yr: Holdridge's 58.93 (L. R.
    /// Holdridge 1967, "Life Zone Ecology", Tropical Science Center: PET = 58.93 × the mean annual
    /// biotemperature, the year's mean of the temperature with the time below freezing counted as
    /// 0 °C), rounded to the surface stage's 60. The land's evapotranspiration (Fu's curve) and
    /// the lakes' evaporation both use it on the year's mean temperature above freezing, so a
    /// cold land with a warm summer evaporates in that summer.
    pub pet_mm_per_c: f64,
    /// Whether a lake's level follows its water balance (a basin that evaporates more than it
    /// gets holds a smaller lake below its rim, a salt flat or nothing). False: every lake is
    /// full to its rim, as before 2026-10-01.
    pub lake_balance: bool,
    /// What open water loses in a year at the freezing point, mm: the evaporation of the cool
    /// season and the sublimation of lake ice. In cold climates lakes lose more than the land's
    /// potential evapotranspiration (60 mm/°C) says. Calibration: Earth's 1.42 million lakes
    /// evaporate 1,500 ± 150 km³ a year without the Caspian (Zhao et al. 2022, "Evaporative water
    /// loss of 1.42 million global lakes", Nature Communications 13:3686), about 650 mm a year
    /// over their 2.3 million km², and most of that area is in cold climates (Canada, Siberia,
    /// Scandinavia: about 6–8 °C of mean temperature above freezing), which 200 + 60 · 7 meets.
    pub lake_evaporation_base_mm: f64,
    /// Scales open-water evaporation: `lake_evaporation_base_mm` plus 60 mm/yr per °C of the
    /// year's mean temperature above freezing, times this.
    pub lake_evaporation_factor: f64,
    /// Snow a degree-day above freezing melts, mm (the degree-day factor): where the year has too
    /// few of them the snow stays, and neither runs off nor feeds a lake.
    pub snow_melt_mm_per_degree_day: f64,
    /// A lake is ice-covered where the mean annual temperature is below this, °C. The views draw
    /// it so, and the game ices the lake's top block there and keeps it open elsewhere, whatever
    /// the biome (it reads this from `notes.bake.water.params`).
    pub lake_freeze_mean_c: f64,
    /// How many blocks of salt the game lays on a playa's floor (the `playa` layer; 0: none).
    pub playa_blocks: f64,
    /// Channels at least this wide are river biomes, blocks.
    pub biome_min_width_blocks: f64,
    /// Banks slope up 1:1 from a channel's edge over this many blocks …
    pub bank_zone_blocks: f64,
    /// … plus this share of its half-width.
    pub bank_zone_ratio: f64,
    /// Whether the game's carvers (caves, canyons) skip the water blocks Deep Time places in lakes
    /// and rivers. The ground under the water is still carved, so caves run beneath it and may
    /// open into a lake or river bed; only the water itself is never replaced (no air pockets in
    /// it, no holes in a lake's surface). Read by the game from `notes.bake.water.params`.
    pub protect_water_from_carvers: bool,
}

impl Default for HydroParams {
    fn default() -> Self {
        HydroParams {
            enabled: true,
            river_min_discharge_m3s: 2.0,
            river_min_area_km2: 50.0,
            width_coef: 1.0,
            depth_coef: 0.5,
            max_width_blocks: 64.0,
            max_depth_blocks: 12.0,
            freeboard_blocks: 1.0,
            smoothing_blocks: 8.0,
            lake_min_area_km2: 1.0,
            lake_min_depth_blocks: 1.0,
            lake_evaporation: true,
            pet_mm_per_c: PET_MM_PER_C,
            lake_balance: true,
            lake_evaporation_base_mm: 200.0,
            lake_evaporation_factor: 1.0,
            snow_melt_mm_per_degree_day: 4.0,
            lake_freeze_mean_c: 0.0,
            playa_blocks: 2.0,
            biome_min_width_blocks: 6.0,
            bank_zone_blocks: 1.0,
            bank_zone_ratio: 0.5,
            protect_water_from_carvers: true,
        }
    }
}

impl HydroParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let within = |v: f64, lo: f64, hi: f64, key: &str| {
            if v.is_finite() && (lo..=hi).contains(&v) {
                Ok(())
            } else {
                Err(CoreError::preset(
                    format!("$.hydro.{key}"),
                    format!("must be in [{lo}, {hi}]"),
                ))
            }
        };
        within(
            self.river_min_discharge_m3s,
            0.01,
            10_000.0,
            "river_min_discharge_m3s",
        )?;
        within(
            self.river_min_area_km2,
            0.0,
            100_000.0,
            "river_min_area_km2",
        )?;
        within(self.width_coef, 0.05, 20.0, "width_coef")?;
        within(self.depth_coef, 0.05, 20.0, "depth_coef")?;
        within(self.max_width_blocks, 1.0, 256.0, "max_width_blocks")?;
        within(self.max_depth_blocks, 1.0, 64.0, "max_depth_blocks")?;
        within(self.freeboard_blocks, 0.0, 8.0, "freeboard_blocks")?;
        within(self.smoothing_blocks, 0.0, 64.0, "smoothing_blocks")?;
        within(self.lake_min_area_km2, 0.0, 100_000.0, "lake_min_area_km2")?;
        within(
            self.lake_min_depth_blocks,
            0.0,
            64.0,
            "lake_min_depth_blocks",
        )?;
        within(self.pet_mm_per_c, 0.0, 200.0, "pet_mm_per_c")?;
        within(
            self.lake_evaporation_base_mm,
            0.0,
            1000.0,
            "lake_evaporation_base_mm",
        )?;
        within(
            self.lake_evaporation_factor,
            0.0,
            5.0,
            "lake_evaporation_factor",
        )?;
        within(
            self.snow_melt_mm_per_degree_day,
            0.0,
            20.0,
            "snow_melt_mm_per_degree_day",
        )?;
        within(self.lake_freeze_mean_c, -30.0, 10.0, "lake_freeze_mean_c")?;
        within(self.playa_blocks, 0.0, 8.0, "playa_blocks")?;
        within(
            self.biome_min_width_blocks,
            1.0,
            1000.0,
            "biome_min_width_blocks",
        )?;
        within(self.bank_zone_blocks, 0.0, 32.0, "bank_zone_blocks")?;
        within(self.bank_zone_ratio, 0.0, 4.0, "bank_zone_ratio")
    }

    /// The water budget of a cell under climate `c`.
    ///
    /// - **Liquid water.** Precipitation falls as rain in the share of the year above freezing and
    ///   as snow in the rest; the snow melts as far as the year's degree-days allow
    ///   (`snow_melt_mm_per_degree_day` × 365.25 × the mean temperature above freezing). What does
    ///   not melt stays as snow and ice (the climate stage's glaciers) and is not water here.
    /// - **Runoff** = liquid − ET, ET by Fu's Budyko curve on the liquid water with PET =
    ///   `pet_mm_per_c` × the year's mean temperature above freezing (Holdridge's biotemperature).
    ///   In a climate without frost that is the surface stage's 60 mm/°C × the mean temperature;
    ///   in a cold one the land still evaporates in its summer (before 2026-10-01 a land with a
    ///   mean below freezing evaporated nothing, and all its precipitation ran off).
    /// - **Open-water evaporation** = `lake_evaporation_factor` × (`lake_evaporation_base_mm` +
    ///   `pet_mm_per_c` × the year's mean temperature above freezing): 200 mm a year at the freezing
    ///   point (the cool season and the ice's sublimation), 1,100 at 15 °C, 2,000 at 30 °C (0
    ///   when `lake_evaporation` is false).
    pub fn budget(&self, c: &Climate) -> Budget {
        let (above, share) = c.thaw();
        let p = c.precip_mm.max(0.0);
        let melt = self.snow_melt_mm_per_degree_day * YEAR_DAYS * above;
        let liquid = p.min(p * share + melt);
        let pet = self.pet_mm_per_c * above;
        Budget {
            runoff_mm: (liquid - crate::surface::climate::fu_et(liquid, pet)).max(0.0),
            rain_mm: liquid,
            evaporation_mm: if self.lake_evaporation {
                self.lake_evaporation_factor
                    * (self.lake_evaporation_base_mm + self.pet_mm_per_c * above)
            } else {
                0.0
            },
        }
    }

    /// Whether a lake at mean annual temperature `t_mean_c` is ice-covered.
    pub fn frozen(&self, t_mean_c: f64) -> bool {
        t_mean_c < self.lake_freeze_mean_c
    }

    /// Channel width in blocks at discharge `q` (m³/s), with the rivers feature scale.
    pub fn width(&self, q: f64, scale: f64) -> f64 {
        (scale * self.width_coef * q.max(0.0).powf(WIDTH_EXPONENT))
            .clamp(1.0, self.max_width_blocks)
    }

    /// Channel depth in blocks at discharge `q`.
    pub fn depth(&self, q: f64) -> f64 {
        (self.depth_coef * q.max(0.0).powf(DEPTH_EXPONENT)).clamp(1.0, self.max_depth_blocks)
    }
}

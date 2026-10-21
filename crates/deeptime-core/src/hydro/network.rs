// SPDX-License-Identifier: LGPL-3.0-or-later
//! The drainage, lakes and river network of one terrain raster (see the module above for the
//! method). Everything but two pure maps (the receivers, the cells' own runoff) runs on one thread
//! in a fixed order, so the result is the same on any thread count.
//!
//! Heights are the elevation layer's raw values (1/16 block, i16) throughout, so every level is
//! exact: a full lake's level is its spill cell's height, a terminal lake's one of the 1/16-block
//! steps of its basin's hypsometry.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};

use deeptime_format::GridDesc;
use deeptime_format::rivers::{Chain, Mouth, RiverVertex, Rivers, TILE_BLOCKS};
use rayon::prelude::*;
use serde_json::{Value, json};

use super::{Climate, HydroParams, OPEN_SEA_KM2};

/// No water level (the `water_level` layer's nodata).
pub const NODATA: i16 = i16::MIN;
/// Seconds in a year (Julian).
const YEAR_S: f64 = 31_557_600.0;
/// No receiver (the sea, or a sink).
const NONE: u8 = 8;

/// Cell states.
const VISITED: u8 = 1;
const SEA: u8 = 2;
const LAKE: u8 = 4;
const RIVER: u8 = 8;
const DEPRESSION: u8 = 16;
const LABELLED: u8 = 32;
const FLAT: u8 = 64;
/// While the flats are found: a flat's cell already taken. After it: a cell of a basin whose
/// balance is still to be solved.
const FLAT_SEEN: u8 = 128;
const BASIN: u8 = FLAT_SEEN;

/// How deep basins inside terminal basins are solved; one nested deeper passes its water on as
/// filled ground.
const MAX_NESTING: u32 = 8;

/// A lake's climate classes in the statistics: arid below this aridity index (P / PET; UNEP's
/// arid and hyper-arid), polar where the warmest month is below this (Köppen's E), °C.
pub const ARID_INDEX: f64 = 0.2;
pub const POLAR_WARM_C: f64 = 10.0;

/// What a cell of [`Output::kind`] holds.
pub mod kind {
    pub const NONE: u8 = 0;
    /// Open lake water.
    pub const LAKE: u8 = 1;
    /// Lake water under ice (`HydroParams::frozen` at the cell).
    pub const FROZEN: u8 = 2;
    /// A playa: the floor of a terminal basin whose balance lake is under the lake thresholds.
    pub const PLAYA: u8 = 3;
}

/// The neighbours, in the order every pass visits them: E, W, S, N, SE, SW, NE, NW.
const DX: [i64; 8] = [1, -1, 0, 0, 1, -1, 1, -1];
const DZ: [i64; 8] = [0, 0, 1, -1, 1, 1, -1, -1];
/// The opposite of each direction.
const OPP: [u8; 8] = [1, 0, 3, 2, 7, 6, 5, 4];

/// What the network needs besides the parameters.
pub struct Input<'a> {
    /// The grid of `elev` (x wraps).
    pub grid: &'a GridDesc,
    /// Terrain height, 1/16 block, row-major.
    pub elev: &'a [i16],
    /// Sea level, 1/16 block: the sea's surface (its top water block is one below). A cell is
    /// sea when its height is at most one block below it (its top block under the sea's top
    /// block, `deeptime_format::water`); land up to sea level is beach, which rivers cross.
    pub sea_raw: i32,
    /// Metres per block horizontally.
    pub m_per_block_h: f64,
    /// The climate at block (x, z).
    pub climate: &'a (dyn Fn(i64, i64) -> Climate + Sync),
    pub params: &'a HydroParams,
    /// `scale.feature_scale.rivers`.
    pub width_scale: f64,
}

/// One lake.
#[derive(Clone, Debug, PartialEq)]
pub struct Lake {
    /// Its surface, 1/16 block: its basin's rim (a full lake), or the level of its water balance.
    pub level: i16,
    /// Its basin's lowest rim, 1/16 block (= `level` when it is full).
    pub rim: i16,
    pub cells: u32,
    /// Its deepest point below the surface, 1/16 block.
    pub depth: i32,
    /// The cell it spills into (`u32::MAX`: it does not spill).
    pub outlet: u32,
    /// Inflow (runoff from its catchment and rain on it) and evaporation, m³/s.
    pub inflow: f64,
    pub evaporation: f64,
    /// Nothing leaves it: it stands below its rim, or its evaporation takes the whole inflow.
    pub endorheic: bool,
    /// Ice-covered: the mean annual temperature over its surface is below
    /// `HydroParams::lake_freeze_mean_c`.
    pub frozen: bool,
    /// The climate over its surface (means over its cells): mean annual and warmest-month
    /// temperature, °C, and the aridity index P / PET (capped at 10).
    pub t_mean_c: f32,
    pub t_warm_c: f32,
    pub aridity: f32,
    /// One of its cells (the first in row order).
    pub cell: u32,
}

/// The network of one raster.
pub struct Output {
    /// The lake surface per cell, [`NODATA`] elsewhere.
    pub water_level: Vec<i16>,
    /// What stands on each cell ([`kind`]): open or frozen lake water, a playa, nothing.
    pub kind: Vec<u8>,
    /// The `closed_basin` layer ([`super::closed_basin_layer`]): 1 where the game puts no sea.
    pub closed_basin: Vec<u8>,
    pub lakes: Vec<Lake>,
    pub rivers: Rivers,
    /// Counts and timings for `notes.bake.water`.
    pub stats: Value,
}

struct Grid {
    w: usize,
    h: usize,
    r: f64,
}

impl Grid {
    #[inline]
    fn nb(&self, c: usize, k: usize) -> Option<usize> {
        let (i, j) = ((c % self.w) as i64, (c / self.w) as i64);
        let nj = j + DZ[k];
        if nj < 0 || nj >= self.h as i64 {
            return None;
        }
        let ni = (i + DX[k]).rem_euclid(self.w as i64);
        Some(nj as usize * self.w + ni as usize)
    }

    fn dist(k: usize) -> f64 {
        if k < 4 { 1.0 } else { std::f64::consts::SQRT_2 }
    }
}

fn ms(t: std::time::Instant) -> f64 {
    (t.elapsed().as_secs_f64() * 10_000.0).round() / 10.0
}

/// A basin whose balance is still to be solved.
struct Basin {
    /// Its lowest rim, 1/16 block.
    rim: i16,
    /// Its cells, every cell after the cell it drains to.
    members: Vec<u32>,
    /// Members the discharge pass has not reached yet: at 0 the basin's inflow is complete.
    pending: u32,
}

/// What became of the basins.
#[derive(Default)]
struct Tally {
    candidates: u32,
    full: u32,
    below_rim: u32,
    playa: u32,
    dry: u32,
    merged: u32,
    passed: u32,
}

/// The drainage being solved: the lakes' balance and the discharge (steps 4 and 5).
struct Net<'a> {
    g: &'a Grid,
    inp: &'a Input<'a>,
    cell_m2: f64,
    min_lake_cells: usize,
    min_lake_depth: i32,
    state: Vec<u8>,
    fill: Vec<i16>,
    rcv: Vec<u8>,
    /// Discharge, m³/s, and drainage area, km², accumulated down the receivers.
    q: Vec<f64>,
    area: Vec<f32>,
    basins: Vec<Basin>,
    basin_of: HashMap<u32, u32>,
    lakes: Vec<Lake>,
    /// Lakes a terminal lake rose over.
    merged: Vec<bool>,
    lake_of: HashMap<u32, u32>,
    playa: Vec<u32>,
    tally: Tally,
    /// Cells the open sea reaches at its own level ([`super::sea_reached`]): a basin among them
    /// is open to the sea.
    reached: Vec<bool>,
}

impl Net<'_> {
    #[inline]
    fn to(&self, c: usize) -> Option<usize> {
        let k = self.rcv[c];
        if k == NONE {
            None
        } else {
            self.g.nb(c, k as usize)
        }
    }

    fn climate(&self, c: usize) -> Climate {
        let grid = self.inp.grid;
        let ri = i64::from(grid.r);
        let (i, j) = ((c % self.g.w) as i64, (c / self.g.w) as i64);
        (self.inp.climate)(grid.origin_block_x + i * ri, grid.origin_block_z + j * ri)
    }

    /// (runoff as land, rain as open water, evaporation as open water) of cell `c`, m³/s.
    fn budget(&self, c: usize) -> (f64, f64, f64) {
        let b = self.inp.params.budget(&self.climate(c));
        let k = self.cell_m2 / 1000.0 / YEAR_S;
        (b.runoff_mm * k, b.rain_mm * k, b.evaporation_mm * k)
    }

    /// What each of `cells` loses when it goes under water (its runoff as land and its
    /// evaporation as water, less the rain on it) and its evaporation, m³/s.
    fn losses(&self, cells: &[u32]) -> Vec<(f64, f64)> {
        let one = |&c: &u32| {
            let (r, p, e) = self.budget(c as usize);
            (r + e - p, e)
        };
        if cells.len() >= 4096 {
            cells.par_iter().map(one).collect()
        } else {
            cells.iter().map(one).collect()
        }
    }

    /// Finds the basins among `order` (cells in drainage order): connected cells below their
    /// spill level, of at least the lake thresholds. Their cells get [`BASIN`].
    fn label(&mut self, order: &[u32]) {
        let elev = self.inp.elev;
        let first = self.basins.len() as u32;
        let mut stack: Vec<usize> = Vec::new();
        let mut members: Vec<usize> = Vec::new();
        for &c0 in order {
            let c0 = c0 as usize;
            if self.state[c0] & DEPRESSION == 0 || self.state[c0] & LABELLED != 0 {
                continue;
            }
            let level = self.fill[c0];
            members.clear();
            stack.clear();
            stack.push(c0);
            self.state[c0] |= LABELLED;
            while let Some(c) = stack.pop() {
                members.push(c);
                for k in 0..8 {
                    if let Some(nb) = self.g.nb(c, k)
                        && self.state[nb] & DEPRESSION != 0
                        && self.state[nb] & LABELLED == 0
                        && self.fill[nb] == level
                    {
                        self.state[nb] |= LABELLED;
                        stack.push(nb);
                    }
                }
            }
            let deepest = members
                .iter()
                .map(|&c| i32::from(level) - i32::from(elev[c]))
                .max()
                .unwrap_or(0);
            if members.len() < self.min_lake_cells || deepest < self.min_lake_depth {
                continue;
            }
            let id = self.basins.len() as u32;
            for &c in &members {
                self.state[c] |= BASIN;
                self.basin_of.insert(c as u32, id);
            }
            self.basins.push(Basin {
                rim: level,
                members: Vec::with_capacity(members.len()),
                pending: members.len() as u32,
            });
            self.tally.candidates += 1;
        }
        if self.basins.len() as u32 > first {
            for &c in order {
                if self.state[c as usize] & BASIN != 0 {
                    let b = self.basin_of[&c];
                    if b >= first {
                        self.basins[b as usize].members.push(c);
                    }
                }
            }
        }
    }

    /// Accumulates the discharge down `order` (cells in drainage order, so upstream cells come
    /// last). A basin's cells hold what reaches them until its last cell has been met: then every
    /// cell upstream of the basin has given, and [`Net::resolve`] decides what it passes on.
    fn accumulate(&mut self, order: &[u32], nesting: u32) {
        for &c in order.iter().rev() {
            let cu = c as usize;
            if self.state[cu] & BASIN != 0 {
                let b = self.basin_of[&c] as usize;
                self.basins[b].pending -= 1;
                if self.basins[b].pending == 0 {
                    self.resolve(b, cu, nesting);
                }
                continue;
            }
            if let Some(r) = self.to(cu) {
                self.q[r] += self.q[cu];
                self.area[r] += self.area[cu];
            }
        }
    }

    /// Carries the discharge through `members` (in drainage order) along their receivers, as far
    /// as `inside` holds for the receiver.
    fn through(&mut self, members: &[u32], inside: impl Fn(&Net<'_>, usize) -> bool) {
        for &m in members.iter().rev() {
            let m = m as usize;
            if let Some(r) = self.to(m)
                && inside(self, r)
            {
                self.q[r] += self.q[m];
                self.area[r] += self.area[m];
            }
        }
    }

    /// Solves basin `b`, whose inflow is complete; `release` is its cell furthest downstream.
    fn resolve(&mut self, b: usize, release: usize, nesting: u32) {
        let p = self.inp.params;
        let elev = self.inp.elev;
        let members = std::mem::take(&mut self.basins[b].members);
        let rim = self.basins[b].rim;
        for &m in &members {
            self.state[m as usize] &= !BASIN;
        }
        // Everything that reaches the basin and the runoff of its own cells as land.
        let inflow: f64 = members.iter().map(|&m| self.q[m as usize]).sum();
        let catchment: f32 = members.iter().map(|&m| self.area[m as usize]).sum();
        let losses = self.losses(&members);
        let loss: f64 = losses.iter().map(|l| l.0).sum();
        let evaporation: f64 = losses.iter().map(|l| l.1).sum();
        let surplus = inflow - loss;
        let out = self.to(release);

        // With no water at all (frozen ground, no rain) a basin is not a lake, however little it
        // would lose.
        let gains = surplus > 0.0 || (surplus == 0.0 && inflow > 0.0);
        if !p.lake_balance || gains {
            // Full to the rim: it spills what it does not evaporate.
            let id = self.lakes.len() as u32;
            let mut deepest = 0;
            for &m in &members {
                let m = m as usize;
                self.state[m] |= LAKE;
                self.lake_of.insert(m as u32, id);
                deepest = deepest.max(i32::from(rim) - i32::from(elev[m]));
            }
            self.through(&members, |n, r| n.state[r] & LAKE != 0);
            let spilt = surplus.max(0.0);
            if let Some(r) = out.filter(|&r| self.state[r] & LAKE == 0) {
                self.q[r] += spilt;
                self.area[r] += catchment;
            }
            self.lakes.push(Lake {
                level: rim,
                rim,
                cells: members.len() as u32,
                depth: deepest,
                outlet: out.map_or(u32::MAX, |r| r as u32),
                inflow: surplus + evaporation,
                evaporation,
                endorheic: surplus <= 0.0,
                frozen: false,
                t_mean_c: 0.0,
                t_warm_c: 0.0,
                aridity: 0.0,
                cell: 0,
            });
            self.merged.push(false);
            self.tally.full += 1;
            return;
        }
        // A basin the open sea reaches at its own level (its spill level is at most sea level)
        // holds the sea's water in the game wherever its ground is a block or more under sea
        // level (the column rule's sea), so it is never dry there (HYD-052, now for these basins
        // only). A closed basin, behind a rim above sea level, takes its balance however deep it
        // lies (ledger: HYD-100): a lake, a playa or dry ground, below sea level or not.
        let sea = self.inp.sea_raw;
        let open = self.reached[members[0] as usize];
        let lowest = members
            .iter()
            .map(|&m| i32::from(elev[m as usize]))
            .min()
            .unwrap_or(i32::MAX);
        let flooded = open && lowest <= sea - 16;
        if inflow <= 0.0 && !flooded {
            self.tally.dry += 1;
            return;
        }
        if nesting >= MAX_NESTING {
            // Nested too deep to solve: filled ground that passes its water on.
            self.through(&members, |_, _| true);
            self.tally.passed += 1;
            return;
        }

        // It cannot fill: a terminal basin. Drain its floor to its lowest cell, solve the basins
        // inside it, and let the lake grow from the sink over the basin's hypsometry.
        let local = self.reflood(&members);
        let spill: Vec<i16> = local.iter().map(|&c| self.fill[c as usize]).collect();
        self.label(&local);
        self.accumulate(&local, nesting + 1);
        let sink = local[0] as usize;
        let reached = self.q[sink];
        let floor = i32::from(elev[sink]);
        let bins = (i32::from(rim) - floor).max(1) as usize;
        let mut lost = vec![0f64; bins];
        let mut evap = vec![0f64; bins];
        let mut count = vec![0u32; bins];
        let losses = self.losses(&local);
        for (i, &c) in local.iter().enumerate() {
            let k = ((i32::from(spill[i]) - floor).max(0) as usize).min(bins - 1);
            // A lake inside the basin has paid its cells' losses already.
            if self.state[c as usize] & LAKE == 0 {
                lost[k] += losses[i].0;
            }
            evap[k] += losses[i].1;
            count[k] += 1;
        }
        // The level rises while the lake still gains: bins 0..k are under water. It stays below
        // the rim (the basin cannot fill).
        let (mut left, mut k, mut cells, mut evaporation) = (reached, 0usize, 0usize, 0.0);
        while k + 1 < bins && left - lost[k] >= 0.0 {
            left -= lost[k];
            cells += count[k] as usize;
            evaporation += evap[k];
            k += 1;
        }
        // Open to the sea and below sea level, the water stands at sea level at least (see
        // above): the lake is what the sea's level floods of the basin, however little reaches it.
        if open && floor <= sea - 16 {
            let sea_k = ((sea - floor) as usize).min(bins - 1);
            while k < sea_k {
                cells += count[k] as usize;
                evaporation += evap[k];
                left = 0.0;
                k += 1;
            }
        }
        let level = (floor + k as i32) as i16;
        let wet = |i: usize| i32::from(spill[i]) < i32::from(level);
        // What is left keeps a share of the next step's cells under a thin sheet of water.
        let sheet = if left > 0.0 && lost[k] > 0.0 {
            ((left / lost[k]).min(1.0) * f64::from(count[k])).floor() as usize
        } else {
            0
        };
        if cells >= self.min_lake_cells && k as i32 >= self.min_lake_depth && cells > 0 {
            let id = self.lakes.len() as u32;
            for (i, &c) in local.iter().enumerate() {
                if !wet(i) {
                    continue;
                }
                let cu = c as usize;
                if self.state[cu] & LAKE != 0 {
                    let old = self.lake_of[&c] as usize;
                    if !self.merged[old] {
                        self.merged[old] = true;
                        self.tally.merged += 1;
                    }
                }
                self.state[cu] = (self.state[cu] | LAKE) & !DEPRESSION;
                self.fill[cu] = level;
                self.lake_of.insert(c, id);
            }
            self.lakes.push(Lake {
                level,
                rim,
                cells: cells as u32,
                depth: k as i32,
                outlet: u32::MAX,
                // What reaches it and the rain on it: what it evaporates, and what is left
                // over for the step of the hypsometry it stopped below.
                inflow: evaporation + left,
                evaporation,
                endorheic: true,
                frozen: false,
                t_mean_c: 0.0,
                t_warm_c: 0.0,
                aridity: 0.0,
                cell: 0,
            });
            self.merged.push(false);
            self.tally.below_rim += 1;
        } else if reached > 0.0 && cells + sheet >= self.min_lake_cells {
            // Too thin or too small for a lake: a playa, the floor the water dries on (the cells
            // under the level and the sheet's, nearest the sink first).
            let mut sheet = sheet;
            for (i, &c) in local.iter().enumerate() {
                let under = wet(i)
                    || (i32::from(spill[i]) == i32::from(level) && sheet > 0 && {
                        sheet -= 1;
                        true
                    });
                if under && self.state[c as usize] & LAKE == 0 {
                    self.playa.push(c);
                }
            }
            self.tally.playa += 1;
        } else {
            self.tally.dry += 1;
        }
    }

    /// Drains a terminal basin's cells to its lowest one: a Priority-Flood of the basin alone
    /// from that cell (a sink), which gives every cell its spill level towards the sink and a
    /// receiver (the steepest strictly lower neighbour, else the cell it was reached from).
    /// Returns the cells in drainage order (the sink first).
    fn reflood(&mut self, members: &[u32]) -> Vec<u32> {
        let elev = self.inp.elev;
        let sink = members
            .iter()
            .copied()
            .min_by_key(|&m| (elev[m as usize], m))
            .expect("a basin has cells") as usize;
        for &m in members {
            self.state[m as usize] &= !(VISITED | LABELLED | FLAT | DEPRESSION);
        }
        // Every cell outside the basin is visited already (the first flood), so the flood stays
        // inside it.
        let mut heap: BinaryHeap<Reverse<(i16, u32)>> = BinaryHeap::new();
        let mut pit: VecDeque<u32> = VecDeque::new();
        let mut order: Vec<u32> = Vec::with_capacity(members.len());
        self.state[sink] |= VISITED;
        self.rcv[sink] = NONE;
        self.fill[sink] = elev[sink];
        heap.push(Reverse((elev[sink], sink as u32)));
        loop {
            let c = if let Some(c) = pit.pop_front() {
                c as usize
            } else if let Some(Reverse((_, c))) = heap.pop() {
                c as usize
            } else {
                break;
            };
            order.push(c as u32);
            let fc = self.fill[c];
            for (k, &opp) in OPP.iter().enumerate() {
                let Some(nb) = self.g.nb(c, k) else { continue };
                if self.state[nb] & VISITED != 0 {
                    continue;
                }
                self.state[nb] |= VISITED;
                self.rcv[nb] = opp;
                if elev[nb] <= fc {
                    self.fill[nb] = fc;
                    pit.push_back(nb as u32);
                } else {
                    self.fill[nb] = elev[nb];
                    heap.push(Reverse((elev[nb], nb as u32)));
                }
            }
        }
        debug_assert_eq!(order.len(), members.len(), "a basin is connected");
        for &c in &order[1..] {
            let c = c as usize;
            let fc = f64::from(self.fill[c]);
            let mut best = NONE;
            let mut best_s = 0.0;
            for k in 0..8 {
                if let Some(nb) = self.g.nb(c, k) {
                    let fnb = f64::from(self.fill[nb]);
                    if fnb < fc {
                        let s = (fc - fnb) / Grid::dist(k);
                        if best == NONE || s > best_s {
                            best = k as u8;
                            best_s = s;
                        }
                    }
                }
            }
            if best != NONE {
                self.rcv[c] = best;
            }
            if self.fill[c] > elev[c] {
                self.state[c] |= DEPRESSION;
            }
        }
        order
    }
}

/// Runs the whole network.
pub fn run(inp: &Input<'_>) -> Output {
    let g = Grid {
        w: inp.grid.w as usize,
        h: inp.grid.h as usize,
        r: f64::from(inp.grid.r),
    };
    let n = g.w * g.h;
    assert_eq!(inp.elev.len(), n, "elevation does not match its grid");
    let p = inp.params;
    let cell_m = g.r * inp.m_per_block_h;
    let cell_km2 = cell_m * cell_m / 1e6;
    let mut millis = serde_json::Map::new();

    // 1. The open sea.
    let t = std::time::Instant::now();
    let mut state = vec![0u8; n];
    let min_sea = ((OPEN_SEA_KM2 / cell_km2).ceil() as usize).max(1);
    let sea_floor = inp.sea_raw - 16;
    open_sea(&g, inp.elev, sea_floor, min_sea, &mut state);
    // The ground the open sea reaches at its own level (HYD-100).
    let low = |c: usize| i32::from(inp.elev[c]) <= inp.sea_raw;
    let open: Vec<bool> = state.iter().map(|&s| s & SEA != 0).collect();
    let reached = super::sea_reached(g.w, g.h, &open, low);
    drop(open);
    millis.insert("sea".into(), json!(ms(t)));

    // 2. Priority-Flood: spill levels, the cell each land cell was reached from, the order.
    let t = std::time::Instant::now();
    let sea_i16 = inp.sea_raw.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
    let floor_i16 = sea_floor.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
    let mut fill: Vec<i16> = inp.elev.to_vec();
    let mut parent = vec![NONE; n];
    let mut order: Vec<u32> = Vec::new();
    flood(
        &g,
        inp.elev,
        floor_i16,
        &mut state,
        &mut fill,
        &mut parent,
        &mut order,
    );
    millis.insert("flood".into(), json!(ms(t)));

    // 3. Receivers: the steepest strictly lower neighbour on the filled surface; on a flat (no lower
    // neighbour: quantised plains, lake floors) the flat's drainage mask, else the parent.
    let t = std::time::Instant::now();
    let mask = flat_masks(&g, &fill, &parent, &mut state);
    millis.insert("flats".into(), json!(ms(t)));
    let t = std::time::Instant::now();
    let rcv: Vec<u8> = (0..n)
        .into_par_iter()
        .map(|c| {
            if state[c] & SEA != 0 {
                return NONE;
            }
            if state[c] & FLAT != 0 && mask[c] != u32::MAX {
                // Across the flat: the neighbour of the same height with the lowest mask (an outlet
                // counts 0); the mask falls by at least one a step, so this ends at an outlet.
                let mut best = NONE;
                let mut best_m = mask[c];
                for k in 0..8 {
                    if let Some(nb) = g.nb(c, k)
                        && fill[nb] == fill[c]
                        && state[nb] & SEA == 0
                    {
                        let m = if state[nb] & FLAT != 0 { mask[nb] } else { 0 };
                        if m < best_m {
                            best = k as u8;
                            best_m = m;
                        }
                    }
                }
                if best != NONE {
                    return best;
                }
            }
            let fc = f64::from(fill[c]);
            let mut best = NONE;
            let mut best_s = 0.0;
            for k in 0..8 {
                if let Some(nb) = g.nb(c, k) {
                    let fnb = f64::from(fill[nb]);
                    if fnb < fc {
                        let s = (fc - fnb) / Grid::dist(k);
                        if best == NONE || s > best_s {
                            best = k as u8;
                            best_s = s;
                        }
                    }
                }
            }
            if best == NONE { parent[c] } else { best }
        })
        .collect();
    drop(parent);
    // Every cell after the cell it drains to: by height, then (on a flat) by mask.
    let key = |c: u32| -> (i16, u32, u32) {
        let cu = c as usize;
        let m = if state[cu] & FLAT != 0 && mask[cu] != u32::MAX {
            mask[cu]
        } else {
            0
        };
        (fill[cu], m, c)
    };
    order.par_sort_unstable_by_key(|&c| key(c));
    drop(mask);
    millis.insert("receivers".into(), json!(ms(t)));

    // 4. Basins, and every land cell's own runoff.
    let t = std::time::Instant::now();
    for &c in &order {
        let c = c as usize;
        if fill[c] > inp.elev[c] {
            state[c] |= DEPRESSION;
        }
    }
    let cell_m2 = cell_m * cell_m;
    let mut net = Net {
        g: &g,
        inp,
        cell_m2,
        min_lake_cells: (p.lake_min_area_km2 / cell_km2).ceil().max(1.0) as usize,
        min_lake_depth: (p.lake_min_depth_blocks * 16.0).round() as i32,
        state,
        fill,
        rcv,
        q: vec![0f64; n],
        area: vec![0f32; n],
        basins: Vec::new(),
        basin_of: HashMap::new(),
        lakes: Vec::new(),
        merged: Vec::new(),
        lake_of: HashMap::new(),
        playa: Vec::new(),
        tally: Tally::default(),
        reached,
    };
    net.label(&order);
    let own: Vec<f64> = order
        .par_iter()
        .map(|&c| net.budget(c as usize).0)
        .collect();
    for (&c, &r) in order.iter().zip(&own) {
        net.q[c as usize] = r;
        net.area[c as usize] = cell_km2 as f32;
    }
    drop(own);
    millis.insert("lakes".into(), json!(ms(t)));

    // 5. Discharge: runoff accumulated down the receivers, each basin solved as its inflow
    // completes (upstream lakes first, so one that does not spill feeds nothing downstream).
    let t = std::time::Instant::now();
    net.accumulate(&order, 0);
    let tally = std::mem::take(&mut net.tally);
    let Net {
        mut state,
        fill,
        rcv,
        q,
        area,
        lakes: all_lakes,
        merged,
        mut lake_of,
        playa,
        reached,
        ..
    } = net;
    // The lakes that stand (one a terminal lake rose over is part of it), numbered again.
    let mut renumber = vec![u32::MAX; all_lakes.len()];
    let mut lakes: Vec<Lake> = Vec::new();
    for (l, lake) in all_lakes.into_iter().enumerate() {
        if !merged[l] {
            renumber[l] = lakes.len() as u32;
            lakes.push(lake);
        }
    }
    lake_of.retain(|&c, l| {
        *l = renumber[*l as usize];
        if *l == u32::MAX {
            state[c as usize] &= !LAKE;
        }
        *l != u32::MAX
    });
    // Each lake's climate, and what stands on each cell.
    let (ox, oz) = (inp.grid.origin_block_x, inp.grid.origin_block_z);
    let ri = i64::from(inp.grid.r);
    let mut lake_cells: Vec<(u32, u32)> = lake_of.iter().map(|(&c, &l)| (c, l)).collect();
    lake_cells.par_sort_unstable();
    let climates: Vec<Climate> = lake_cells
        .par_iter()
        .map(|&(c, _)| {
            let cu = c as usize;
            (inp.climate)(ox + (cu % g.w) as i64 * ri, oz + (cu / g.w) as i64 * ri)
        })
        .collect();
    let mut water_level = vec![NODATA; n];
    let mut kinds = vec![kind::NONE; n];
    let mut sums = vec![(0f64, 0f64, 0f64, 0u32, u32::MAX); lakes.len()];
    for (&(c, l), cl) in lake_cells.iter().zip(&climates) {
        let s = &mut sums[l as usize];
        s.0 += cl.t_mean_c;
        s.1 += cl.t_warm_c;
        s.2 += cl.aridity();
        s.3 += 1;
        s.4 = s.4.min(c);
        water_level[c as usize] = lakes[l as usize].level;
        kinds[c as usize] = if p.frozen(cl.t_mean_c) {
            kind::FROZEN
        } else {
            kind::LAKE
        };
    }
    drop(climates);
    drop(lake_cells);
    for (lake, s) in lakes.iter_mut().zip(&sums) {
        let k = f64::from(s.3.max(1));
        lake.cells = s.3;
        lake.t_mean_c = (s.0 / k) as f32;
        lake.t_warm_c = (s.1 / k) as f32;
        lake.aridity = (s.2 / k) as f32;
        lake.frozen = p.frozen(s.0 / k);
        lake.cell = s.4;
    }
    let mut playa_cells = 0u64;
    for &c in &playa {
        if state[c as usize] & LAKE == 0 && kinds[c as usize] == kind::NONE {
            kinds[c as usize] = kind::PLAYA;
            playa_cells += 1;
        }
    }
    drop(playa);
    millis.insert("discharge".into(), json!(ms(t)));
    let to = |c: usize| -> Option<usize> {
        let k = rcv[c];
        if k == NONE { None } else { g.nb(c, k as usize) }
    };

    // 6. River cells, base levels, water surfaces. A lake cell less than a block deep is dry in
    // the game (its ground is in the same block as the surface), so a river runs on across it
    // until it meets water a block deep: `deep` lake cells end rivers, the others carry them.
    let t = std::time::Instant::now();
    let qmin = p.river_min_discharge_m3s;
    let amin = p.river_min_area_km2 as f32;
    let mut river_cells: Vec<u32> = Vec::new();
    for &c in &order {
        let cu = c as usize;
        if state[cu] & SEA == 0
            && !is_deep(&state, &fill, inp.elev, cu)
            && q[cu] >= qmin
            && area[cu] >= amin
        {
            state[cu] |= RIVER;
            river_cells.push(c);
        }
    }
    river_cells.sort_unstable();
    let land = order.len();
    drop(order);
    // The level of the sea or lake downstream of a cell, found by walking down from it (and kept
    // for every cell on the way): a lake's level, the sea's, or a sink's own height.
    let mut base = vec![NODATA; n];
    let mut path: Vec<usize> = Vec::new();
    let mut base_of = |c: usize| -> i16 {
        path.clear();
        let mut cur = c;
        let level = loop {
            if base[cur] != NODATA {
                break base[cur];
            }
            if state[cur] & SEA != 0 {
                break sea_i16;
            }
            if state[cur] & LAKE != 0 {
                break fill[cur];
            }
            path.push(cur);
            match to(cur) {
                Some(r) => cur = r,
                None => break fill[cur],
            }
        };
        for &c in &path {
            base[c] = level;
        }
        level
    };
    let fb = (p.freeboard_blocks * 16.0).round() as i32;
    let mut surface = |c: usize| -> f32 {
        let w = (i32::from(fill[c]) - fb).max(i32::from(base_of(c)));
        w as f32 / 16.0
    };

    let deep = |c: usize| is_deep(&state, &fill, inp.elev, c);

    // 7. Chains.
    let mut main_donor: HashMap<u32, u32> = HashMap::new();
    for &d in &river_cells {
        if let Some(r) = to(d as usize)
            && state[r] & RIVER != 0
        {
            let e = main_donor.entry(r as u32).or_insert(d);
            let (qd, qe) = (q[d as usize], q[*e as usize]);
            if qd > qe || (qd == qe && d < *e) {
                *e = d;
            }
        }
    }
    struct Walk {
        cells: Vec<u32>,
        mouth: Mouth,
        into: u32,
        lead: bool,
    }
    let mut walks: Vec<Walk> = Vec::new();
    for &head in &river_cells {
        if main_donor.contains_key(&head) {
            continue;
        }
        let mut cells = Vec::new();
        // A river leaving a lake starts in its water: the deep lake cell draining into its head
        // with the most discharge becomes the first vertex.
        let mut feeder: Option<usize> = None;
        for k in 0..8 {
            if let Some(nb) = g.nb(head as usize, k)
                && deep(nb)
                && to(nb) == Some(head as usize)
                && feeder.is_none_or(|f| q[nb] > q[f] || (q[nb] == q[f] && nb < f))
            {
                feeder = Some(nb);
            }
        }
        let lead = if let Some(f) = feeder {
            cells.push(f as u32);
            true
        } else {
            false
        };
        cells.push(head);
        let mut cur = head as usize;
        let (mouth, into) = loop {
            match to(cur) {
                // A sink: the lake it lies in, else the planet's edge or a dry basin's floor.
                None if state[cur] & LAKE != 0 => break (Mouth::Lake, lake_of[&(cur as u32)]),
                None => break (Mouth::Sea, u32::MAX),
                Some(r) if state[r] & SEA != 0 => {
                    cells.push(r as u32);
                    break (Mouth::Sea, u32::MAX);
                }
                Some(r) if deep(r) => {
                    cells.push(r as u32);
                    break (Mouth::Lake, lake_of[&(r as u32)]);
                }
                Some(r) if state[r] & RIVER != 0 => {
                    cells.push(r as u32);
                    if main_donor.get(&(r as u32)) == Some(&(cur as u32)) {
                        cur = r;
                        continue;
                    }
                    break (Mouth::River, r as u32);
                }
                // Below the threshold: only where a lake took the flow (the river ends in the
                // lake it was crossing, or in the shallows of the lake it reaches).
                Some(_) if state[cur] & LAKE != 0 => break (Mouth::Lake, lake_of[&(cur as u32)]),
                Some(r) if state[r] & LAKE != 0 => {
                    cells.push(r as u32);
                    break (Mouth::Lake, lake_of[&(r as u32)]);
                }
                Some(r) => {
                    cells.push(r as u32);
                    break (Mouth::Sea, u32::MAX);
                }
            }
        };
        // A chain needs a segment of its own (a river cell with no receiver, a sink, has none).
        if cells.len() < usize::from(lead) + 2 {
            continue;
        }
        walks.push(Walk {
            cells,
            mouth,
            into,
            lead,
        });
    }
    // Which chain runs through each river cell (not as its last vertex).
    let mut chain_of: HashMap<u32, u32> = HashMap::new();
    for (k, wk) in walks.iter().enumerate() {
        let own_end = wk.cells.len() - 1;
        for &c in &wk.cells[usize::from(wk.lead)..own_end] {
            chain_of.insert(c, k as u32);
        }
    }

    // 8. Geometry: unwrapped positions, attributes, smoothing (larger rivers first, so a
    // tributary's mouth takes the smoothed position of the vertex it joins).
    let c_blocks = inp.grid.circumference() as f64;
    let pos_of = |c: u32| -> (f64, f64) {
        let cu = c as usize;
        (
            (ox + (cu % g.w) as i64 * ri) as f64,
            (oz + (cu / g.w) as i64 * ri) as f64,
        )
    };
    let mut pos: Vec<Vec<(f64, f64)>> = walks
        .iter()
        .map(|wk| {
            let mut out: Vec<(f64, f64)> = Vec::with_capacity(wk.cells.len());
            for &c in &wk.cells {
                let (mut x, z) = pos_of(c);
                if let Some(&(px, _)) = out.last() {
                    x += ((px - x) / c_blocks + 0.5).floor() * c_blocks;
                }
                out.push((x, z));
            }
            out
        })
        .collect();
    let mut by_size: Vec<usize> = (0..walks.len()).collect();
    let own_max = |k: usize| -> f64 {
        let cells = &walks[k].cells;
        q[cells[cells.len().saturating_sub(2)] as usize]
    };
    by_size.sort_by(|&a, &b| own_max(b).total_cmp(&own_max(a)).then(a.cmp(&b)));
    let half = (p.smoothing_blocks / g.r).round() as usize;
    let mut smoothed_at: HashMap<u32, (f64, f64)> = HashMap::new();
    for &k in &by_size {
        let wk = &walks[k];
        let m = wk.cells.len();
        if wk.mouth == Mouth::River
            && let Some(&(sx, sz)) = smoothed_at.get(&wk.cells[m - 1])
        {
            let (ux, _) = pos[k][m - 1];
            let x = sx + ((ux - sx) / c_blocks + 0.5).floor() * c_blocks;
            pos[k][m - 1] = (x, sz);
        }
        if half > 0 && m > 2 {
            smooth(&mut pos[k], half);
        }
        for (v, &c) in wk.cells.iter().enumerate().take(m - 1) {
            if v >= usize::from(wk.lead) {
                smoothed_at.insert(c, pos[k][v]);
            }
        }
    }

    let mut rivers = Rivers::empty(inp.grid.circumference() as u32);
    rivers.tile = TILE_BLOCKS;
    rivers.biome_min_width = p.biome_min_width_blocks as f32;
    rivers.bank_zone = p.bank_zone_blocks as f32;
    rivers.bank_ratio = p.bank_zone_ratio as f32;
    let mut length_blocks = 0.0;
    let (mut q_max, mut w_max) = (0.0f64, 0.0f64);
    for (k, wk) in walks.iter().enumerate() {
        let first = rivers.vertices.len() as u32;
        let m = wk.cells.len();
        // Own vertices carry their cell's discharge; the lead (a lake cell) and the mouth copy
        // their neighbour's so a tributary does not swell at the confluence.
        let geom = |c: u32| {
            let qq = q[c as usize];
            (p.width(qq, inp.width_scale) / 2.0, p.depth(qq), qq)
        };
        let own_first = usize::from(wk.lead);
        for v in 0..m {
            let src = v.clamp(own_first, m - 2);
            let (hw, depth, qq) = geom(wk.cells[src]);
            let (x, z) = pos[k][v];
            q_max = q_max.max(qq);
            w_max = w_max.max(2.0 * hw);
            if v > 0 {
                let (px, pz) = pos[k][v - 1];
                length_blocks += ((x - px).powi(2) + (z - pz).powi(2)).sqrt();
            }
            rivers.vertices.push(RiverVertex {
                x: x as f32,
                z: z as f32,
                w: surface(wk.cells[v] as usize),
                hw: hw as f32,
                depth: depth as f32,
                q: qq as f32,
            });
        }
        let into = match wk.mouth {
            Mouth::River => chain_of.get(&wk.into).copied().unwrap_or(u32::MAX),
            Mouth::Lake => wk.into,
            Mouth::Sea => u32::MAX,
        };
        rivers.chains.push(Chain {
            first,
            count: m as u32,
            mouth: if wk.mouth == Mouth::River && into == u32::MAX {
                Mouth::Sea
            } else {
                wk.mouth
            },
            into,
        });
    }
    // Monotone surfaces along every chain: the vertices were taken from cells whose surface never
    // rises downstream, and a smoothed position does not change that; clamp the float rounding.
    for ch in &rivers.chains {
        let (a, b) = (ch.first as usize, (ch.first + ch.count) as usize);
        for v in a + 1..b {
            if rivers.vertices[v].w > rivers.vertices[v - 1].w {
                rivers.vertices[v].w = rivers.vertices[v - 1].w;
            }
        }
    }
    rivers.build_index();
    millis.insert("rivers".into(), json!(ms(t)));

    // 8. Where the game puts no sea (HYD-100, HYD-102), and what the ground a block or more below
    // sea level holds.
    let t = std::time::Instant::now();
    let radius = (super::CLOSED_REACH_BLOCKS / g.r).ceil().max(1.0) as usize;
    let closed_basin = super::closed_basin_layer(g.w, g.h, low, &reached, radius);
    let below = |c: usize| i32::from(inp.elev[c]) <= sea_floor;
    let (mut open_low, mut closed, mut wet, mut salt) = (0u64, 0u64, 0u64, 0u64);
    for c in 0..n {
        if state[c] & SEA != 0 || !below(c) {
            continue;
        }
        if reached[c] {
            open_low += 1;
            continue;
        }
        closed += 1;
        match kinds[c] {
            kind::LAKE | kind::FROZEN => wet += 1,
            kind::PLAYA => salt += 1,
            _ => {}
        }
    }
    drop(reached);
    let under_sea: Vec<&Lake> = lakes
        .iter()
        .filter(|l| i32::from(l.level) < inp.sea_raw)
        .collect();
    let share = |cells: u64| {
        if land > 0 {
            (cells as f64 / land as f64 * 1e5).round() / 1e5
        } else {
            0.0
        }
    };
    let km2 = |cells: u64| (cells as f64 * cell_km2).round();
    let below_sea = json!({
        "open_km2": km2(open_low),
        "closed_km2": km2(closed),
        "closed_share_of_land": share(closed),
        "lake_km2": km2(wet),
        "playa_km2": km2(salt),
        "dry_km2": km2(closed - wet - salt),
        "dry_share_of_land": share(closed - wet - salt),
        "lakes_below_sea_level": under_sea.len(),
        "lakes_below_sea_level_km2": km2(under_sea.iter().map(|l| u64::from(l.cells)).sum()),
        "closed_basin_radius_cells": radius,
    });
    millis.insert("closed_basin".into(), json!(ms(t)));

    let stats = statistics(
        inp,
        &g,
        &lakes,
        &tally,
        cell_km2,
        land,
        playa_cells,
        json!({
            "grid": inp.grid,
            "land_cells": land,
            "open_sea_min_cells": min_sea,
            "river_cells": river_cells.len(),
            "chains": rivers.chains.len(),
            "vertices": rivers.vertices.len(),
            "index_entries": rivers.entries.len(),
            "river_length_km": (length_blocks * inp.m_per_block_h / 1000.0).round(),
            "max_discharge_m3s": (q_max * 10.0).round() / 10.0,
            "widest_blocks": (w_max * 10.0).round() / 10.0,
            "below_sea": below_sea,
            "millis": Value::Object(millis),
        }),
    );
    Output {
        water_level,
        kind: kinds,
        closed_basin,
        lakes,
        rivers,
        stats,
    }
}

/// `notes.bake.water.stats`: `base` (the grid, the rivers, the timings) and the lakes: how many,
/// their area and its share of the land, what became of the basins, the lakes by climate, and the
/// largest ones.
#[allow(clippy::too_many_arguments)]
fn statistics(
    inp: &Input<'_>,
    g: &Grid,
    lakes: &[Lake],
    tally: &Tally,
    cell_km2: f64,
    land: usize,
    playa_cells: u64,
    base: Value,
) -> Value {
    let km2 = |cells: u64| (cells as f64 * cell_km2).round();
    let area = |f: &dyn Fn(&Lake) -> bool| -> (usize, u64) {
        let mut n = 0;
        let mut cells = 0u64;
        for l in lakes.iter().filter(|l| f(l)) {
            n += 1;
            cells += u64::from(l.cells);
        }
        (n, cells)
    };
    let (_, lake_cells) = area(&|_| true);
    let class = |f: &dyn Fn(&Lake) -> bool| -> Value {
        let (n, cells) = area(f);
        json!({
            "lakes": n,
            "km2": km2(cells),
            "share_of_lake_area": if lake_cells > 0 {
                (cells as f64 / lake_cells as f64 * 1e4).round() / 1e4
            } else {
                0.0
            },
        })
    };
    let arid = |l: &Lake| f64::from(l.aridity) < ARID_INDEX;
    let polar = |l: &Lake| f64::from(l.t_warm_c) < POLAR_WARM_C;
    let mut by_size: Vec<&Lake> = lakes.iter().collect();
    by_size.sort_by(|a, b| b.cells.cmp(&a.cells).then(a.cell.cmp(&b.cell)));
    let c = inp.grid.circumference() as f64;
    let largest: Vec<Value> = by_size
        .iter()
        .take(5)
        .map(|l| {
            let cu = l.cell as usize;
            let z = inp.grid.origin_block_z as f64 + (cu / g.w) as f64 * g.r;
            let round = |v: f32| (f64::from(v) * 100.0).round() / 100.0;
            json!({
                "km2": km2(u64::from(l.cells)),
                "latitude": (deeptime_format::Projection::of(c).lat_deg(z) * 10.0).round() / 10.0,
                "block_x": inp.grid.origin_block_x + (cu % g.w) as i64 * i64::from(inp.grid.r),
                "block_z": z,
                "level_blocks": f64::from(l.level) / 16.0,
                "below_rim_blocks": f64::from(l.rim - l.level) / 16.0,
                "depth_blocks": f64::from(l.depth) / 16.0,
                "endorheic": l.endorheic,
                "inflow_m3s": (l.inflow * 10.0).round() / 10.0,
                "evaporation_m3s": (l.evaporation * 10.0).round() / 10.0,
                "frozen": l.frozen,
                "t_mean_c": round(l.t_mean_c),
                "t_warm_c": round(l.t_warm_c),
                "aridity": round(l.aridity),
            })
        })
        .collect();
    let mut stats = json!({
        "lake_balance": inp.params.lake_balance,
        "lakes": lakes.len(),
        "lake_cells": lake_cells,
        "lake_area_km2": km2(lake_cells),
        "land_km2": km2(land as u64),
        "lake_share_of_land": if land > 0 {
            (lake_cells as f64 / land as f64 * 1e5).round() / 1e5
        } else {
            0.0
        },
        "largest_lake_km2": km2(by_size.first().map_or(0, |l| u64::from(l.cells))),
        "endorheic_lakes": lakes.iter().filter(|l| l.endorheic).count(),
        "basins": {
            "candidates": tally.candidates,
            "full": tally.full,
            "below_rim": tally.below_rim,
            "playa": tally.playa,
            "dry": tally.dry,
            "merged_into_a_terminal_lake": tally.merged,
            "nested_too_deep": tally.passed,
        },
        "playa_km2": km2(playa_cells),
        "below_rim": class(&|l| l.level < l.rim),
        "frozen": class(&|l| l.frozen),
        "arid": class(&arid),
        "polar": class(&polar),
        "arid_or_polar": class(&|l| arid(l) || polar(l)),
        "largest": largest,
    });
    if let (Value::Object(s), Value::Object(b)) = (&mut stats, base) {
        s.extend(b);
    }
    stats
}

/// A lake cell at least a block deep: water in the game (a shallower one is dry ground).
fn is_deep(state: &[u8], fill: &[i16], elev: &[i16], c: usize) -> bool {
    state[c] & LAKE != 0 && i32::from(fill[c]) - i32::from(elev[c]) >= 16
}

/// Drainage across flats (Garbrecht & Martz 1997, as improved by Barnes, Lehman & Mulla 2014, "An
/// efficient assignment of drainage direction over flat surfaces in raster digital elevation models",
/// Computers & Geosciences 62:128–135; implemented from the paper). A flat cell is a land cell with
/// no strictly lower neighbour on the filled surface that is not a sink; a flat is a connected set of
/// them at one height. Its outlets are neighbours of the same height that can drain. The mask is
/// 2 × (steps to the nearest outlet) + (the flat's largest distance from higher ground − the cell's
/// own), so water crosses a flat towards its outlets and away from the higher ground around it,
/// gathering in one channel instead of fanning out in parallel rays (what following the flood's
/// breadth-first parents does). Returns the mask per cell (`u32::MAX` off flats, or on a flat with
/// no outlet, where the parent stays) and marks [`FLAT`] in `state`.
fn flat_masks(g: &Grid, fill: &[i16], parent: &[u8], state: &mut [u8]) -> Vec<u32> {
    let n = g.w * g.h;
    let flat: Vec<bool> = (0..n)
        .into_par_iter()
        .map(|c| {
            state[c] & SEA == 0
                && parent[c] != NONE
                && !(0..8).any(|k| g.nb(c, k).is_some_and(|nb| fill[nb] < fill[c]))
        })
        .collect();
    for (c, f) in flat.iter().enumerate() {
        if *f {
            state[c] |= FLAT;
        }
    }
    drop(flat);
    let mut mask = vec![u32::MAX; n];
    let mut away = vec![0u32; n];
    let mut members: Vec<usize> = Vec::new();
    let mut queue: VecDeque<usize> = VecDeque::new();
    for c0 in 0..n {
        if state[c0] & FLAT == 0 || state[c0] & FLAT_SEEN != 0 {
            continue;
        }
        // The flat: 8-connected cells of one height.
        let h = fill[c0];
        members.clear();
        queue.clear();
        queue.push_back(c0);
        state[c0] |= FLAT_SEEN;
        while let Some(c) = queue.pop_front() {
            members.push(c);
            for k in 0..8 {
                if let Some(nb) = g.nb(c, k)
                    && state[nb] & FLAT != 0
                    && state[nb] & FLAT_SEEN == 0
                    && fill[nb] == h
                {
                    state[nb] |= FLAT_SEEN;
                    queue.push_back(nb);
                }
            }
        }
        let same_flat = |nb: usize| state[nb] & FLAT != 0 && fill[nb] == h;
        // Towards the outlets (the mask holds the distance for now).
        queue.clear();
        for &c in &members {
            let outlet = (0..8).any(|k| {
                g.nb(c, k)
                    .is_some_and(|nb| fill[nb] == h && state[nb] & (FLAT | SEA) == 0)
            });
            if outlet {
                mask[c] = 1;
                queue.push_back(c);
            }
        }
        if queue.is_empty() {
            continue; // no outlet: the parents stay
        }
        while let Some(c) = queue.pop_front() {
            for k in 0..8 {
                if let Some(nb) = g.nb(c, k)
                    && same_flat(nb)
                    && mask[nb] == u32::MAX
                {
                    mask[nb] = mask[c] + 1;
                    queue.push_back(nb);
                }
            }
        }
        // Away from higher ground.
        queue.clear();
        for &c in &members {
            let high = (0..8).any(|k| g.nb(c, k).is_some_and(|nb| fill[nb] > h));
            if high {
                away[c] = 1;
                queue.push_back(c);
            }
        }
        let mut max_away = 0;
        while let Some(c) = queue.pop_front() {
            max_away = max_away.max(away[c]);
            for k in 0..8 {
                if let Some(nb) = g.nb(c, k)
                    && same_flat(nb)
                    && away[nb] == 0
                {
                    away[nb] = away[c] + 1;
                    queue.push_back(nb);
                }
            }
        }
        for &c in &members {
            if mask[c] != u32::MAX {
                let a = if away[c] > 0 { max_away - away[c] } else { 0 };
                mask[c] = 2 * mask[c] + a;
            }
            away[c] = 0;
        }
    }
    for s in state.iter_mut() {
        *s &= !FLAT_SEEN;
    }
    mask
}

/// Marks the open sea: cells at or below `sea_raw` in an 8-connected body (x wrapping) of at
/// least `min_cells`. The others stay land.
fn open_sea(g: &Grid, elev: &[i16], sea_raw: i32, min_cells: usize, state: &mut [u8]) {
    let n = g.w * g.h;
    let mut done = vec![false; n];
    let mut members: Vec<usize> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    for c0 in 0..n {
        if done[c0] || i32::from(elev[c0]) > sea_raw {
            continue;
        }
        members.clear();
        stack.push(c0);
        done[c0] = true;
        while let Some(c) = stack.pop() {
            members.push(c);
            for k in 0..8 {
                if let Some(nb) = g.nb(c, k)
                    && !done[nb]
                    && i32::from(elev[nb]) <= sea_raw
                {
                    done[nb] = true;
                    stack.push(nb);
                }
            }
        }
        if members.len() >= min_cells {
            for &c in &members {
                state[c] |= SEA;
            }
        }
    }
}

/// Priority-Flood with the pit queue from the open sea's shores. Fills `fill` (the spill level
/// of every land cell; the sea at sea level), `parent` (the direction to the cell each land cell
/// was reached from) and `order` (land cells as they left the queues: every cell after the cell
/// it drains to). The first and last rows drain off the map (sinks at their own height); land that
/// neither reaches drains to its lowest cell, a sink.
fn flood(
    g: &Grid,
    elev: &[i16],
    sea: i16,
    state: &mut [u8],
    fill: &mut [i16],
    parent: &mut [u8],
    order: &mut Vec<u32>,
) {
    let n = g.w * g.h;
    let mut heap: BinaryHeap<Reverse<(i16, u32)>> = BinaryHeap::new();
    let mut pit: VecDeque<u32> = VecDeque::new();
    for c in 0..n {
        if state[c] & SEA != 0 {
            state[c] |= VISITED;
            fill[c] = sea;
        }
    }
    for c in 0..n {
        if state[c] & SEA != 0 && (0..8).any(|k| g.nb(c, k).is_some_and(|nb| state[nb] & SEA == 0))
        {
            heap.push(Reverse((sea, c as u32)));
        }
    }
    // The first and last rows (the poles, past the hard cap) are outlets too: water that reaches
    // them leaves the map there instead of running along the edge.
    for c in (0..g.w).chain((g.h - 1) * g.w..n) {
        if state[c] & VISITED == 0 {
            state[c] |= VISITED;
            parent[c] = NONE;
            heap.push(Reverse((elev[c], c as u32)));
        }
    }
    let mut next_unvisited = 0usize;
    loop {
        let c = if let Some(c) = pit.pop_front() {
            c as usize
        } else if let Some(Reverse((_, c))) = heap.pop() {
            c as usize
        } else {
            // Land no sea reaches: its lowest unvisited cell becomes a sink.
            let mut lowest: Option<usize> = None;
            while next_unvisited < n && state[next_unvisited] & VISITED != 0 {
                next_unvisited += 1;
            }
            for c in next_unvisited..n {
                if state[c] & VISITED == 0 && lowest.is_none_or(|l| elev[c] < elev[l]) {
                    lowest = Some(c);
                }
            }
            let Some(c) = lowest else { break };
            state[c] |= VISITED;
            parent[c] = NONE;
            heap.push(Reverse((elev[c], c as u32)));
            continue;
        };
        if state[c] & SEA == 0 {
            order.push(c as u32);
        }
        let fc = fill[c];
        for (k, &opp) in OPP.iter().enumerate() {
            let Some(nb) = g.nb(c, k) else { continue };
            if state[nb] & VISITED != 0 {
                continue;
            }
            state[nb] |= VISITED;
            parent[nb] = opp;
            if elev[nb] <= fc {
                fill[nb] = fc;
                pit.push_back(nb as u32);
            } else {
                fill[nb] = elev[nb];
                heap.push(Reverse((elev[nb], nb as u32)));
            }
        }
    }
}

/// Gaussian smoothing of a polyline's interior vertices over ±`half` vertices (σ = half / 2),
/// two passes; the first and last vertices stay.
fn smooth(pts: &mut [(f64, f64)], half: usize) {
    let sigma = (half as f64 / 2.0).max(0.5);
    let weights: Vec<f64> = (0..=half)
        .map(|j| (-(j as f64).powi(2) / (2.0 * sigma * sigma)).exp())
        .collect();
    let m = pts.len();
    for _ in 0..2 {
        let src = pts.to_vec();
        for v in 1..m - 1 {
            // A window that stays inside the polyline, symmetric so a straight run stays straight.
            let reach = half.min(v).min(m - 1 - v);
            let (mut sx, mut sz, mut sw) = (0.0, 0.0, 0.0);
            for j in 0..=reach {
                let wj = weights[j];
                if j == 0 {
                    sx += wj * src[v].0;
                    sz += wj * src[v].1;
                    sw += wj;
                } else {
                    sx += wj * (src[v - j].0 + src[v + j].0);
                    sz += wj * (src[v - j].1 + src[v + j].1);
                    sw += 2.0 * wj;
                }
            }
            pts[v] = (sx / sw, sz / sw);
        }
    }
}

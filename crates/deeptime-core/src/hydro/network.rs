// SPDX-License-Identifier: LGPL-3.0-or-later
//! The drainage, lakes and river network of one terrain raster (see the module above for the
//! method). Everything runs on one thread in a fixed order, so the result is the same on any
//! machine and any thread count; the only parallel pass (the receivers) is a pure map.
//!
//! Heights are the elevation layer's raw values (1/16 block, i16) throughout, so every level is
//! exact and a lake's level is a height the terrain actually has (its spill cell's).

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};

use deeptime_format::GridDesc;
use deeptime_format::rivers::{Chain, Mouth, RiverVertex, Rivers, TILE_BLOCKS};
use rayon::prelude::*;
use serde_json::{Value, json};

use super::{HydroParams, OPEN_SEA_KM2};
use crate::surface::climate::fu_et;

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
const FLAT_SEEN: u8 = 128;

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
    /// (precipitation, potential evapotranspiration), mm/yr, at block (x, z).
    pub climate: &'a (dyn Fn(i64, i64) -> (f64, f64) + Sync),
    pub params: &'a HydroParams,
    /// `scale.feature_scale.rivers`.
    pub width_scale: f64,
}

/// One lake.
#[derive(Clone, Debug, PartialEq)]
pub struct Lake {
    /// Its surface, 1/16 block (= its spill cell's height).
    pub level: i16,
    pub cells: u32,
    /// Its deepest point below the surface, 1/16 block.
    pub depth: i32,
    /// The cell it spills into.
    pub outlet: u32,
    /// Inflow (runoff from its catchment and rain on it) and evaporation, m³/s.
    pub inflow: f64,
    pub evaporation: f64,
    /// Nothing leaves it (its evaporation takes the whole inflow).
    pub endorheic: bool,
}

/// The network of one raster.
pub struct Output {
    /// The lake surface per cell, [`NODATA`] elsewhere.
    pub water_level: Vec<i16>,
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
    let to = |c: usize| -> Option<usize> {
        let k = rcv[c];
        if k == NONE { None } else { g.nb(c, k as usize) }
    };
    // Every cell after the cell it drains to: by height, then (on a flat) by mask.
    let key = |c: u32| -> (i16, u32, u32) {
        let cu = c as usize;
        let m = if state[cu] & FLAT != 0 && mask[cu] != u32::MAX { mask[cu] } else { 0 };
        (fill[cu], m, c)
    };
    order.par_sort_unstable_by_key(|&c| key(c));
    drop(mask);
    millis.insert("receivers".into(), json!(ms(t)));

    // 4. Depressions and lakes.
    let t = std::time::Instant::now();
    for &c in &order {
        let c = c as usize;
        if fill[c] > inp.elev[c] {
            state[c] |= DEPRESSION;
        }
    }
    let min_lake_cells = (p.lake_min_area_km2 / cell_km2).ceil().max(1.0) as usize;
    let min_lake_depth = (p.lake_min_depth_blocks * 16.0).round() as i32;
    let mut lake_of: HashMap<u32, u32> = HashMap::new();
    let mut lakes: Vec<Lake> = Vec::new();
    let mut queue: Vec<usize> = Vec::new();
    let mut members: Vec<usize> = Vec::new();
    for &c0 in &order {
        let c0 = c0 as usize;
        if state[c0] & DEPRESSION == 0 || state[c0] & LABELLED != 0 {
            continue;
        }
        members.clear();
        queue.clear();
        queue.push(c0);
        state[c0] |= LABELLED;
        while let Some(c) = queue.pop() {
            members.push(c);
            for k in 0..8 {
                if let Some(nb) = g.nb(c, k)
                    && state[nb] & DEPRESSION != 0
                    && state[nb] & LABELLED == 0
                {
                    state[nb] |= LABELLED;
                    queue.push(nb);
                }
            }
        }
        let level = fill[c0];
        let deepest = members
            .iter()
            .map(|&c| i32::from(level) - i32::from(inp.elev[c]))
            .max()
            .unwrap_or(0);
        if members.len() < min_lake_cells || deepest < min_lake_depth {
            continue;
        }
        let id = lakes.len() as u32;
        members.sort_unstable();
        for &c in &members {
            state[c] |= LAKE;
            lake_of.insert(c as u32, id);
        }
        lakes.push(Lake {
            level,
            cells: members.len() as u32,
            depth: deepest,
            outlet: u32::MAX,
            inflow: 0.0,
            evaporation: 0.0,
            endorheic: false,
        });
    }
    millis.insert("lakes".into(), json!(ms(t)));

    // 5. Discharge: runoff accumulated down the receivers; a lake loses its evaporation.
    let t = std::time::Instant::now();
    let cell_m2 = cell_m * cell_m;
    let mut q = vec![0f64; n];
    let mut area = vec![0f32; n];
    let (ox, oz) = (inp.grid.origin_block_x, inp.grid.origin_block_z);
    let ri = i64::from(inp.grid.r);
    let own: Vec<(u32, f64, f64)> = order
        .par_iter()
        .map(|&c| {
            let cu = c as usize;
            let (i, j) = ((cu % g.w) as i64, (cu / g.w) as i64);
            let (pr, pet) = (inp.climate)(ox + i * ri, oz + j * ri);
            let runoff = if state[cu] & LAKE != 0 {
                pr
            } else {
                (pr - fu_et(pr, pet)).max(0.0)
            };
            let evap = if state[cu] & LAKE != 0 && p.lake_evaporation {
                pet
            } else {
                0.0
            };
            (
                c,
                runoff / 1000.0 * cell_m2 / YEAR_S,
                evap / 1000.0 * cell_m2 / YEAR_S,
            )
        })
        .collect();
    for &(c, r, e) in &own {
        q[c as usize] = r;
        area[c as usize] = cell_km2 as f32;
        if let Some(&l) = lake_of.get(&c) {
            lakes[l as usize].evaporation += e;
        }
    }
    drop(own);
    let mut remaining: Vec<f64> = lakes.iter().map(|l| l.evaporation).collect();
    for &c in order.iter().rev() {
        let c = c as usize;
        let Some(r) = to(c) else { continue };
        let mut moved = q[c];
        if state[c] & LAKE != 0 {
            let l = lake_of[&(c as u32)] as usize;
            let leaves = lake_of.get(&(r as u32)).is_none_or(|&m| m as usize != l);
            if leaves {
                lakes[l].inflow += moved;
                let taken = moved.min(remaining[l]);
                remaining[l] -= taken;
                moved -= taken;
                if lakes[l].outlet == u32::MAX {
                    lakes[l].outlet = r as u32;
                }
            }
        }
        q[r] += moved;
        area[r] += area[c];
    }
    for (l, lake) in lakes.iter_mut().enumerate() {
        // Rain on the lake counts as inflow too (it went into q of the lake's cells).
        lake.endorheic = remaining[l] > 0.0 || lake.inflow <= lake.evaporation;
    }
    millis.insert("discharge".into(), json!(ms(t)));

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
    // The level of the sea or lake downstream of every cell (downstream cells come first in
    // `order`).
    let mut base = vec![sea_i16; n];
    for &c in &order {
        let cu = c as usize;
        base[cu] = if state[cu] & LAKE != 0 {
            fill[cu]
        } else {
            match to(cu) {
                Some(r) => base[r],
                // A sink (the planet's north or south edge): its own level.
                None => fill[cu],
            }
        };
    }
    let fb = (p.freeboard_blocks * 16.0).round() as i32;
    let surface = |c: usize| -> f32 {
        let w = (i32::from(fill[c]) - fb).max(i32::from(base[c]));
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
                // Below the threshold: only where a lake's evaporation took the flow (the river
                // ends in the lake it was crossing).
                Some(_) if state[cur] & LAKE != 0 => break (Mouth::Lake, lake_of[&(cur as u32)]),
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

    let mut water_level = vec![NODATA; n];
    for (&c, &l) in &lake_of {
        water_level[c as usize] = lakes[l as usize].level;
    }
    let lake_cells: u64 = lakes.iter().map(|l| u64::from(l.cells)).sum();
    let largest = lakes.iter().map(|l| l.cells).max().unwrap_or(0);
    let land = order.len();
    let stats = json!({
        "grid": inp.grid,
        "land_cells": land,
        "open_sea_min_cells": min_sea,
        "lakes": lakes.len(),
        "lake_cells": lake_cells,
        "lake_area_km2": (lake_cells as f64 * cell_km2).round(),
        "largest_lake_km2": (f64::from(largest) * cell_km2).round(),
        "endorheic_lakes": lakes.iter().filter(|l| l.endorheic).count(),
        "river_cells": river_cells.len(),
        "chains": rivers.chains.len(),
        "vertices": rivers.vertices.len(),
        "index_entries": rivers.entries.len(),
        "river_length_km": (length_blocks * inp.m_per_block_h / 1000.0).round(),
        "max_discharge_m3s": (q_max * 10.0).round() / 10.0,
        "widest_blocks": (w_max * 10.0).round() / 10.0,
        "millis": Value::Object(millis),
    });
    Output {
        water_level,
        lakes,
        rivers,
        stats,
    }
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
                g.nb(c, k).is_some_and(|nb| fill[nb] == h && state[nb] & (FLAT | SEA) == 0)
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

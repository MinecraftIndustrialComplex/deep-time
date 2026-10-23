// SPDX-License-Identifier: LGPL-3.0-or-later
//! Deltas (V1 gap 5, stage 1; docs/science/rivers-and-lakes.md, "Deltas and distributaries"):
//! the plain each sea-reaching river builds at its mouth, and the distributaries across it.
//!
//! The bake's water part ([`crate::bake::water`]) runs the network once ([`network::run`]), then
//! [`grow`] here: every river that reaches the open sea gets a sediment load from BQART, the plain
//! Earth's deltas build for that load, and that plain is laid over the sea and beach cells
//! beside its mouth as far as the volume lasts, one block above the sea's top water block. The
//! network runs again on the raised ground, and [`distributaries`] splits each plain's main stem
//! at its apex into channels that reach the plain's front. The single-receiver drainage is
//! untouched: the splits are new chains in the river file (`from`), and the game carves them as
//! it carves every chain.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

use deeptime_format::GridDesc;
use deeptime_format::rivers::{Chain, Mouth, RiverVertex};
use serde::Serialize;
use serde_json::{Value, json};

use super::network::{self, SeaMouth};
use super::{HydroParams, OPEN_SEA_KM2};
use crate::params::BuildFit;

/// Seconds in a year (Julian).
const YEAR_S: f64 = 31_557_600.0;

/// BQART (Syvitski & Milliman 2007, Eq. 3 and its cold branch): Qs = ω B Q^0.31 A^0.5 R T for
/// T ≥ 2 °C, 2 ω B Q^0.31 A^0.5 R below; ω = 0.02 for Qs in kg/s with Q in km³/yr, A in km², R
/// in km and T in °C. ledger: HYD-130
pub const BQART_OMEGA: f64 = 0.02; // ledger: HYD-130
pub const BQART_Q_EXP: f64 = 0.31; // ledger: HYD-130
pub const BQART_A_EXP: f64 = 0.5; // ledger: HYD-130
pub const BQART_T_MIN_C: f64 = 2.0; // ledger: HYD-130

/// Delta area from sediment load (Nienhuis et al., the 1,741 deltas of their data, least squares
/// in log space): log10 A*[m²] = a + b log10 Qs[kg/s]. ledger: HYD-132
pub const AREA_LOG_A: f64 = 5.463; // ledger: HYD-132
pub const AREA_LOG_B: f64 = 0.932; // ledger: HYD-132

/// The delta plain's thickness, the channel depth at the mouth: D = c Q^e m, at most `DEPTH_MAX_M`
/// (Nienhuis et al., `get_retention.m`). ledger: HYD-133
pub const DEPTH_C: f64 = 0.08; // ledger: HYD-133
pub const DEPTH_E: f64 = 0.65; // ledger: HYD-133
pub const DEPTH_MAX_M: f64 = 69.0; // ledger: HYD-133

/// The plain's highest step above the sea's surface at its mouth and its lowest at the front,
/// 1/16 block: the plain's top block is one above the sea's top water block (its height in
/// (sea level, sea level + 1]), graded so the plain drains outward. ledger: HYD-136
pub const PLAIN_TOP_STEPS: i32 = 15; // ledger: HYD-136
pub const PLAIN_FRONT_STEPS: i32 = 1; // ledger: HYD-136
/// The lowest the river's course across the plain lies, 1/16 block under the sea's surface: a
/// sixteenth above the open sea's depth (the course is dry ground of the beach band's height, the
/// column one block under the plain, and never open sea). ledger: HYD-136
pub const COURSE_FLOOR_STEPS: i32 = -15; // ledger: HYD-136
/// The most of a plain's volume its river's course across the shallows (with the banks beside
/// it) may take before the lobe grows at its end: half. ledger: HYD-134
pub const PLAIN_COURSE_SHARE: f64 = 0.5; // ledger: HYD-134

/// A plain of fewer cells is no delta (about 1 km² on the 600 m cells of L3). ledger: HYD-142
pub const MIN_PLAIN_CELLS: usize = 3; // ledger: HYD-142
/// Sea pockets a plain closes off from the open sea are filled up to this many cells; larger
/// ones stay as lagoons (closed basins). ledger: HYD-142
pub const POCKET_FILL_CELLS: usize = 200; // ledger: HYD-142

/// How far the main stem's slope is measured above the old mouth, m (the backwater length beside
/// the apex). ledger: HYD-137
pub const SLOPE_REACH_M: f64 = 30_000.0; // ledger: HYD-137

/// A distributary's path costs this much more on a cell another channel of the plain already
/// takes. ledger: HYD-144
pub const PATH_SHARED_COST: f64 = 4.0; // ledger: HYD-144

/// What the plains need besides the first network.
pub struct Input<'a> {
    /// The hydrology grid and its ground before any plain, 1/16 block.
    pub grid: &'a GridDesc,
    pub elev: &'a [i16],
    pub sea_raw: i32,
    pub m_per_block_h: f64,
    pub m_per_block_v: f64,
    /// The stored elevation's compression under the build limit (relief is measured on the
    /// simulated heights).
    pub fit: BuildFit,
    pub params: &'a HydroParams,
    pub width_scale: f64,
}

/// One delta plain.
#[derive(Clone, Debug, Serialize)]
pub struct Plain {
    /// The first network's chain and its mouth (sea cell).
    pub chain: u32,
    pub sea: u32,
    pub root: u32,
    pub q_m3s: f64,
    pub area_km2: f64,
    pub relief_km: f64,
    pub t_mean_c: f64,
    pub lake_share: f64,
    /// BQART's load, kg/s; Earth's plain for it (A*, km²) and its thickness (D, m).
    pub qs_kg_s: f64,
    pub a_star_km2: f64,
    pub d_m: f64,
    /// The main stem's water-surface slope over [`SLOPE_REACH_M`] above the old mouth (m/m).
    pub slope: f64,
    /// The plain's first cell on the stem (where the river comes off the land), the outflow
    /// direction its height falls along, and its highest step over the sea (HYD-136).
    pub entry: u32,
    #[serde(skip)]
    pub dir: (f64, f64),
    pub top_steps: i32,
    /// The river's course on the plain, entry first (the stem's crossing of the shallows, then on
    /// to the plain's tip), and the land cell it comes off (HYD-134, HYD-136).
    #[serde(skip)]
    pub course: Vec<u32>,
    #[serde(skip)]
    pub land: Option<u32>,
    /// The plain's cells (first taken first) and their height, 1/16 block.
    #[serde(skip)]
    pub cells: Vec<u32>,
    #[serde(skip)]
    pub height: Vec<i16>,
    /// Cells added from closed-off sea pockets.
    pub pocket_cells: u32,
    /// What the volume left over, m³ (0 when the plain used it all).
    pub left_m3: f64,
}

/// The plains of one raster.
#[derive(Default)]
pub struct Plains {
    pub plains: Vec<Plain>,
    /// The plain of each plain cell.
    pub owner: HashMap<u32, u32>,
    /// Sea pockets the plains closed off and left (lagoons), and their cells.
    pub lagoons: u32,
    pub lagoon_cells: u64,
    /// How many mouths were looked at and how many had a load.
    pub mouths: u32,
}

impl Plains {
    /// The ground with the plains on it.
    pub fn apply(&self, elev: &mut [i16]) {
        for p in &self.plains {
            for (&c, &h) in p.cells.iter().zip(&p.height) {
                let e = &mut elev[c as usize];
                *e = (*e).max(h);
            }
        }
    }
}

/// BQART's load, kg/s, of a river: Q m³/s, A km², relief R km, mean temperature T °C, the share
/// of its catchment behind lakes. ledger: HYD-130, HYD-131
pub fn bqart_kg_s(q_m3s: f64, area_km2: f64, relief_km: f64, t_c: f64, lake_share: f64) -> f64 {
    let q_km3 = q_m3s * YEAR_S / 1e9;
    // B = I L (1 − TE) Eh with I = L = Eh = 1; the trapping TE is the lake share (HYD-131).
    let b = (1.0 - lake_share).clamp(0.0, 1.0);
    let base = BQART_OMEGA
        * b
        * q_km3.max(0.0).powf(BQART_Q_EXP)
        * area_km2.max(0.0).powf(BQART_A_EXP)
        * relief_km.max(0.0);
    if t_c >= BQART_T_MIN_C {
        base * t_c
    } else {
        2.0 * base
    }
}

/// Earth's delta area for load `qs` kg/s, m². ledger: HYD-132
pub fn delta_area_m2(qs_kg_s: f64) -> f64 {
    if qs_kg_s <= 0.0 {
        0.0
    } else {
        10f64.powf(AREA_LOG_A + AREA_LOG_B * qs_kg_s.log10())
    }
}

/// The plain's thickness, m, at discharge `q` m³/s. ledger: HYD-133
pub fn plain_depth_m(q_m3s: f64) -> f64 {
    (DEPTH_C * q_m3s.max(0.0).powf(DEPTH_E)).min(DEPTH_MAX_M)
}

/// Positions and distances on the hydrology grid (x wraps).
struct Geo {
    w: usize,
    h: usize,
    r: f64,
    ox: f64,
    oz: f64,
    c: f64,
}

impl Geo {
    fn of(g: &GridDesc) -> Geo {
        Geo {
            w: g.w as usize,
            h: g.h as usize,
            r: f64::from(g.r),
            ox: g.origin_block_x as f64,
            oz: g.origin_block_z as f64,
            c: g.circumference() as f64,
        }
    }

    fn pos(&self, c: u32) -> (f64, f64) {
        let c = c as usize;
        (
            self.ox + (c % self.w) as f64 * self.r,
            self.oz + (c / self.w) as f64 * self.r,
        )
    }

    /// b − a, x wrapped into [−C/2, C/2).
    fn delta(&self, a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
        let mut dx = b.0 - a.0;
        dx -= self.c * (dx / self.c).round();
        (dx, b.1 - a.1)
    }

    fn nb(&self, c: usize, k: usize) -> Option<usize> {
        const DX: [i64; 8] = [1, -1, 0, 0, 1, -1, 1, -1];
        const DZ: [i64; 8] = [0, 0, 1, -1, 1, 1, -1, -1];
        let (i, j) = ((c % self.w) as i64, (c / self.w) as i64);
        let nj = j + DZ[k];
        if nj < 0 || nj >= self.h as i64 {
            return None;
        }
        Some(nj as usize * self.w + (i + DX[k]).rem_euclid(self.w as i64) as usize)
    }

    fn step(k: usize) -> f64 {
        if k < 4 { 1.0 } else { std::f64::consts::SQRT_2 }
    }
}

/// The direction a stem leaves the land in: from its vertex `back` cells above the sea cell to the
/// sea cell, unit length (east when the stem is too short to say).
fn outflow(geo: &Geo, stem: &[u32], back: usize) -> (f64, f64) {
    let m = stem.len();
    let from = stem[m.saturating_sub(1 + back)];
    let (dx, dz) = geo.delta(geo.pos(from), geo.pos(stem[m - 1]));
    let len = (dx * dx + dz * dz).sqrt();
    if len > 0.0 {
        (dx / len, dz / len)
    } else {
        (1.0, 0.0)
    }
}

/// The main stem's water-surface slope over [`SLOPE_REACH_M`] above its root (m/m; 0 where it is
/// level). ledger: HYD-137
fn stem_slope(geo: &Geo, mouth: &SeaMouth, m_h: f64, m_v: f64) -> f64 {
    let m = mouth.stem.len();
    if m < 3 {
        return 0.0;
    }
    let root = m - 2;
    let mut run = 0.0;
    let mut j = root;
    while j > 0 && run < SLOPE_REACH_M {
        let (dx, dz) = geo.delta(geo.pos(mouth.stem[j - 1]), geo.pos(mouth.stem[j]));
        run += (dx * dx + dz * dz).sqrt() * m_h;
        j -= 1;
    }
    if run <= 0.0 {
        return 0.0;
    }
    let rise = f64::from(i32::from(mouth.stem_fill[j]) - i32::from(mouth.stem_fill[root])) / 16.0;
    (rise * m_v / run).max(0.0)
}

/// Lays the delta plains on the ground (`inp.elev`, which this does not change): see the module.
/// `water_level` and `closed_basin` are the first network's; every plain cell is land after.
pub fn grow(inp: &Input<'_>, first: &network::Output) -> Plains {
    let geo = Geo::of(inp.grid);
    let p = inp.params;
    let n = geo.w * geo.h;
    let cell_m = geo.r * inp.m_per_block_h;
    let cell_m2 = cell_m * cell_m;
    let sea_y = f64::from(inp.sea_raw) / 16.0;
    let eligible = |c: usize| {
        i32::from(inp.elev[c]) <= inp.sea_raw
            && first.water_level[c] == network::NODATA
            && first.closed_basin[c] == 0
    };

    // Every mouth's load and plain, largest load first.
    struct Want {
        k: usize,
        qs: f64,
        relief_km: f64,
    }
    let mut wants: Vec<Want> = first
        .mouths
        .iter()
        .enumerate()
        .filter_map(|(k, m)| {
            let top = inp.fit.expand(f64::from(m.max_raw) / 16.0);
            let relief_km = (top - sea_y) * inp.m_per_block_v / 1000.0;
            let qs = bqart_kg_s(m.q, m.area_km2, relief_km, m.t_mean_c, m.lake_share);
            (qs > 0.0 && qs.is_finite()).then_some(Want { k, qs, relief_km })
        })
        .collect();
    wants.sort_by(|a, b| b.qs.total_cmp(&a.qs).then(a.k.cmp(&b.k)));

    let mut owner: HashMap<u32, u32> = HashMap::new();
    let mut plains: Vec<Plain> = Vec::new();
    let shape_k = p.delta_shape_k; // ledger: HYD-135
    for want in &wants {
        let m = &first.mouths[want.k];
        let sea = m.sea as usize;
        if owner.contains_key(&m.sea) || !eligible(sea) {
            continue;
        }
        let a_star = delta_area_m2(want.qs) * p.delta_area_mult; // ledger: HYD-132, HYD-141
        let d = plain_depth_m(m.q);
        if d <= 0.0 {
            continue;
        }
        let volume = a_star * d;
        let mut left = volume;
        // The cheapest cell, at half its volume, ends the growth (HYD-134).
        let cheapest = cell_m2 * d;
        let dir = outflow(&geo, &m.stem, 5);
        // The seeds: the stem's own way across the shallows to the sea cell (its trailing cells
        // at or below the sea that no plain holds yet), so the river runs on the plain from the
        // coast (HYD-134). The last cell before them is the land the river comes off.
        let mut from = m.stem.len() - 1;
        while from > 0 {
            let c = m.stem[from - 1];
            if !eligible(c as usize) || owner.contains_key(&c) {
                break;
            }
            from -= 1;
        }
        let seeds: Vec<u32> = m.stem[from..].to_vec();
        let seed_pos: Vec<(f64, f64)> = seeds.iter().map(|&c| geo.pos(c)).collect();
        // A cell's distance from the nearest seed, weighted by its bearing from the outflow.
        let prio = |c: usize| -> f64 {
            let pc = geo.pos(c as u32);
            seed_pos
                .iter()
                .map(|&ps| {
                    let (dx, dz) = geo.delta(ps, pc);
                    let dist = (dx * dx + dz * dz).sqrt();
                    if dist == 0.0 {
                        return 0.0;
                    }
                    let cos = (dx * dir.0 + dz * dir.1) / dist;
                    dist * (1.0 + shape_k * (1.0 - cos)) // ledger: HYD-135
                })
                .fold(f64::INFINITY, f64::min)
        };
        let key = |c: usize| (prio(c) * 16.0).round() as u64;
        // Each cell costs its area times the deeper of the plain's thickness and the water over
        // it (HYD-134).
        let cost_of = |c: usize| {
            let depth_m =
                f64::from((inp.sea_raw - i32::from(inp.elev[c])).max(0)) / 16.0 * inp.m_per_block_v;
            cell_m2 * d.max(depth_m) // ledger: HYD-134
        };
        // The river's course first, coast to sea, each cell with the shallows beside it (its
        // banks; the land cell's too beside the first), so the second network finds no lower
        // ground beside the course; it ends where the next cell and its banks would take the
        // course past its share of the volume, or past what is left (HYD-134).
        let seed_set: HashSet<u32> = seeds.iter().copied().collect();
        let land = (from > 0).then(|| m.stem[from - 1]);
        let mut taken: HashSet<u32> = HashSet::new();
        let mut cells: Vec<u32> = Vec::new();
        let mut course: Vec<u32> = Vec::new();
        let mut spent = 0.0;
        for (i, &s) in seeds.iter().enumerate() {
            let mut want = vec![s];
            let mut around = vec![s as usize];
            if i == 0
                && let Some(l) = land
            {
                around.push(l as usize);
            }
            for &a in &around {
                for k in 0..8 {
                    let Some(nb) = geo.nb(a, k) else { continue };
                    let nbu = nb as u32;
                    if eligible(nb)
                        && i32::from(inp.elev[nb]) > inp.sea_raw - 16
                        && !owner.contains_key(&nbu)
                        && !taken.contains(&nbu)
                        && !seed_set.contains(&nbu)
                        && !want.contains(&nbu)
                    {
                        want.push(nbu);
                    }
                }
            }
            let cost: f64 = want.iter().map(|&c| cost_of(c as usize)).sum();
            if left < cost / 2.0 || (i > 0 && spent + cost > volume * PLAIN_COURSE_SHARE) {
                break;
            }
            left -= cost;
            spent += cost;
            for &c in &want {
                taken.insert(c);
                cells.push(c);
            }
            course.push(s);
        }
        if course.is_empty() {
            continue;
        }
        // Then the lobe around the course and beyond its end, cell by cell in order of the
        // weighted distance (HYD-135).
        let mut heap: BinaryHeap<Reverse<(u64, u32)>> = BinaryHeap::new();
        let mut seen: HashSet<u32> = taken.clone();
        seen.extend(&course);
        let push =
            |c: usize, heap: &mut BinaryHeap<Reverse<(u64, u32)>>, seen: &mut HashSet<u32>| {
                for k in 0..8 {
                    if let Some(nb) = geo.nb(c, k)
                        && !owner.contains_key(&(nb as u32))
                        && eligible(nb)
                        && seen.insert(nb as u32)
                    {
                        heap.push(Reverse((key(nb), nb as u32)));
                    }
                }
            };
        for &c in &cells {
            push(c as usize, &mut heap, &mut seen);
        }
        while let Some(Reverse((_, c))) = heap.pop() {
            if left < cheapest / 2.0 {
                break;
            }
            let cost = cost_of(c as usize);
            if left < cost / 2.0 {
                continue;
            }
            left -= cost;
            cells.push(c);
            push(c as usize, &mut heap, &mut seen);
        }
        if cells.len() < MIN_PLAIN_CELLS {
            continue;
        }
        // The plain's top: one block over the sea's top water block, or the land it comes off
        // where that is lower (HYD-136).
        let land_raw = land.map_or(inp.sea_raw + PLAIN_TOP_STEPS, |l| {
            i32::from(inp.elev[l as usize])
        });
        let top = (land_raw - inp.sea_raw).clamp(PLAIN_FRONT_STEPS, PLAIN_TOP_STEPS);
        let id = plains.len() as u32;
        for &c in &cells {
            owner.insert(c, id);
        }
        plains.push(Plain {
            chain: m.chain,
            sea: m.sea,
            root: m.root,
            q_m3s: m.q,
            area_km2: m.area_km2,
            relief_km: want.relief_km,
            t_mean_c: m.t_mean_c,
            lake_share: m.lake_share,
            qs_kg_s: want.qs,
            a_star_km2: a_star / 1e6,
            d_m: d,
            slope: stem_slope(&geo, m, inp.m_per_block_h, inp.m_per_block_v),
            entry: seeds[0],
            dir,
            top_steps: top,
            course,
            land,
            cells,
            height: Vec::new(),
            pocket_cells: 0,
            left_m3: left.max(0.0),
        });
    }

    // Sea the plains closed off from the open sea, deep or shallow (the ground the open sea
    // reaches at its own level, before and after): small pockets join the plain beside them,
    // larger ones stay (lagoons). ledger: HYD-142
    let (mut lagoons, mut lagoon_cells) = (0u32, 0u64);
    let mut reach_after: Vec<bool> = Vec::new();
    if !plains.is_empty() {
        let cell_km2 = cell_m2 / 1e6;
        let min_sea = ((OPEN_SEA_KM2 / cell_km2).ceil() as usize).max(1);
        let deep_raw = inp.sea_raw - 16;
        let low = |c: usize| i32::from(inp.elev[c]) <= inp.sea_raw;
        let before = {
            let open = super::open_sea_mask(
                geo.w,
                geo.h,
                |c| i32::from(inp.elev[c]) <= deep_raw,
                min_sea,
            );
            super::sea_reached(geo.w, geo.h, &open, low)
        };
        let after = {
            let open = super::open_sea_mask(
                geo.w,
                geo.h,
                |c| i32::from(inp.elev[c]) <= deep_raw && !owner.contains_key(&(c as u32)),
                min_sea,
            );
            super::sea_reached(geo.w, geo.h, &open, |c| {
                low(c) && !owner.contains_key(&(c as u32))
            })
        };
        let mut done = vec![false; n];
        let mut stack: Vec<usize> = Vec::new();
        let mut members: Vec<usize> = Vec::new();
        for c0 in 0..n {
            if done[c0] || !before[c0] || after[c0] || owner.contains_key(&(c0 as u32)) {
                continue;
            }
            members.clear();
            stack.push(c0);
            done[c0] = true;
            let mut beside: Option<u32> = None;
            while let Some(c) = stack.pop() {
                members.push(c);
                for k in 0..8 {
                    let Some(nb) = geo.nb(c, k) else { continue };
                    if let Some(&o) = owner.get(&(nb as u32)) {
                        beside = Some(beside.map_or(o, |b: u32| b.min(o)));
                    }
                    if !done[nb] && before[nb] && !after[nb] && !owner.contains_key(&(nb as u32)) {
                        done[nb] = true;
                        stack.push(nb);
                    }
                }
            }
            match beside {
                Some(o) if members.len() <= POCKET_FILL_CELLS => {
                    members.sort_unstable();
                    let pl = &mut plains[o as usize];
                    for &c in &members {
                        pl.cells.push(c as u32);
                        pl.pocket_cells += 1;
                    }
                    for &c in &members {
                        owner.insert(c as u32, o);
                    }
                }
                _ => {
                    lagoons += 1;
                    lagoon_cells += members.len() as u64;
                }
            }
        }
        reach_after = after;
    }

    // Heights: a ramp along the outflow from the plain's entry, its top there and one sixteenth
    // over the sea at its farthest cell, level across the outflow. The river's course stays at
    // the beach band's height under the plain, never rising downstream, so the second network
    // keeps the river on it across the plain to the front (HYD-136).
    for (o, pl) in plains.iter_mut().enumerate() {
        let pe = geo.pos(pl.entry);
        let along: Vec<f64> = pl
            .cells
            .iter()
            .map(|&c| {
                let (dx, dz) = geo.delta(pe, geo.pos(c));
                (dx * pl.dir.0 + dz * pl.dir.1).max(0.0)
            })
            .collect();
        let far = along.iter().copied().fold(geo.r, f64::max);
        let span = f64::from(pl.top_steps - PLAIN_FRONT_STEPS);
        let mut steps: Vec<i32> = along
            .iter()
            .map(|&f| PLAIN_FRONT_STEPS + (span * (1.0 - f / far)).round() as i32)
            .collect();
        let index: HashMap<u32, usize> =
            pl.cells.iter().enumerate().map(|(i, &c)| (c, i)).collect();
        // The course on from the crossing's end to the plain's tip (its cell beside the open
        // sea's water farthest along the outflow) where the lobe closes the end off, the shortest
        // way through the plain that keeps off its edge where it can (a cell beside water the
        // plain does not hold costs as a shared one, HYD-144).
        let mine = |c: u32| owner.get(&c) == Some(&(o as u32));
        let beside_sea = |c: u32| {
            !reach_after.is_empty()
                && (0..8).any(|k| geo.nb(c as usize, k).is_some_and(|nb| reach_after[nb]))
        };
        if let Some(&end) = pl.course.last()
            && !beside_sea(end)
        {
            let on_course: HashSet<u32> = pl.course.iter().copied().collect();
            let tip = pl
                .cells
                .iter()
                .zip(&along)
                .filter(|&(&c, _)| beside_sea(c) && !on_course.contains(&c))
                .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(a.0)))
                .map(|(&c, _)| c);
            let edge: HashSet<u32> = pl
                .cells
                .iter()
                .copied()
                .filter(|&c| {
                    (0..8).any(|k| {
                        geo.nb(c as usize, k).is_some_and(|nb| {
                            i32::from(inp.elev[nb]) <= inp.sea_raw && !mine(nb as u32)
                        })
                    })
                })
                .collect();
            if let Some(tip) = tip
                && let Some(path) = shortest(
                    &geo,
                    end,
                    tip,
                    |c| mine(c) && (c == end || !on_course.contains(&c)),
                    &edge,
                )
            {
                pl.course.extend(&path[1..]);
            }
        }
        // The course: each cell at the highest ground the course has from it on (so it never
        // rises downstream and nothing is lowered), at most the sea's surface, and at least the
        // floor: a sixteenth over the open sea's depth, or the lowest sea-reached ground beside
        // its last cell where that is higher (so the river spills from it onto the beach band
        // rather than into a pit).
        let old = |c: u32| i32::from(inp.elev[c as usize]) - inp.sea_raw;
        let floor = pl
            .course
            .last()
            .and_then(|&end| {
                (0..8)
                    .filter_map(|k| geo.nb(end as usize, k))
                    .filter(|&nb| !reach_after.is_empty() && reach_after[nb])
                    .map(|nb| old(nb as u32))
                    .min()
            })
            .map_or(COURSE_FLOOR_STEPS, |m| m.max(COURSE_FLOOR_STEPS));
        let mut high = i32::MIN;
        for &c in pl.course.iter().rev() {
            high = high.max(old(c));
            steps[index[&c]] = high.clamp(floor.min(0), 0); // ledger: HYD-136
        }
        pl.height = steps.iter().map(|&s| (inp.sea_raw + s) as i16).collect();
    }
    Plains {
        plains,
        owner,
        lagoons,
        lagoon_cells,
        mouths: wants.len() as u32,
    }
}

/// One channel across a plain while the distributaries are laid out.
struct Branch {
    /// Its cells, from the cell it leaves its parent at (or the apex) to its sea cell.
    cells: Vec<u32>,
    /// The parent branch and the index in the parent's cells it leaves at (none: the main stem).
    parent: Option<(usize, usize)>,
    /// How many of its first cells lie on the plain (the rest cross the beach band to the sea):
    /// splits happen there only.
    on_plain: usize,
    /// The front segment its mouth lies in.
    seg: usize,
    /// From which of its cells on it carries which share of the apex discharge, in order.
    share: Vec<(usize, f64)>,
}

impl Branch {
    fn share_at(&self, v: usize) -> f64 {
        let mut s = 1.0;
        for &(from, f) in &self.share {
            if v >= from {
                s = f;
            }
        }
        s
    }
}

/// What [`distributaries`] did, per plain.
#[derive(Clone, Debug, Serialize)]
pub struct Split {
    pub plain: u32,
    /// The second network's main stem across the plain, the apex's distance along it from where
    /// it leaves the plain (blocks), the backwater length (m) and whether that length is the
    /// shorter (HYD-137).
    pub chain: u32,
    pub apex_blocks: f64,
    pub backwater_m: f64,
    pub backwater_inside: bool,
    /// The plain's front (cells beside the sea's water, the beach band or the open sea), the
    /// mouths that fit along it (HYD-138) and the mouths it holds.
    pub front_cells: u32,
    pub fit: u32,
    pub mouths: u32,
    /// Chains added.
    pub added: u32,
}

/// Splits each plain's main stem in the second network (`second`, run on the raised ground) into
/// distributaries (HYD-137 to HYD-140, HYD-144): new chains with `from`, and the main stem's
/// discharge, width and depth below each split cut to its share. `elev` is the raised ground.
pub fn distributaries(
    inp: &Input<'_>,
    elev: &[i16],
    plains: &Plains,
    second: &mut network::Output,
) -> Vec<Split> {
    let geo = Geo::of(inp.grid);
    let p = inp.params;
    let sea_w = inp.sea_raw as f32 / 16.0;
    let cell_km2 = (geo.r * inp.m_per_block_h).powi(2) / 1e6;
    let min_sea = ((OPEN_SEA_KM2 / cell_km2).ceil() as usize).max(1);
    let open = super::open_sea_mask(
        geo.w,
        geo.h,
        |c| i32::from(elev[c]) <= inp.sea_raw - 16,
        min_sea,
    );
    // The ground the open sea reaches at its own level: the open sea and the beach band joined
    // to it, none of a plain's cells (its river's course lies at the band's height). A plain's
    // front faces it.
    let reach = super::sea_reached(geo.w, geo.h, &open, |c| {
        i32::from(elev[c]) <= inp.sea_raw && !plains.owner.contains_key(&(c as u32))
    });
    // The second network's main stem across each plain: the sea mouth with the most discharge
    // whose stem crosses the plain (the last plain it crosses; it may leave the plain over the
    // beach band beyond it before it meets the open sea).
    let mut stem_of: HashMap<u32, usize> = HashMap::new();
    for (k, m) in second.mouths.iter().enumerate() {
        if let Some(&o) = m.stem.iter().rev().find_map(|c| plains.owner.get(c)) {
            let e = stem_of.entry(o).or_insert(k);
            let best = &second.mouths[*e];
            if m.q > best.q || (m.q == best.q && m.chain < best.chain) {
                *e = k;
            }
        }
    }
    let mut ids: Vec<u32> = stem_of.keys().copied().collect();
    ids.sort_unstable();
    // The steps from each sea-reached cell to the open sea, which the branches' tails descend.
    let to_sea = if ids.is_empty() {
        Vec::new()
    } else {
        sea_distance(&geo, &reach, &open)
    };
    let mut out = Vec::new();
    for o in ids {
        let plain = &plains.plains[o as usize];
        let mouth = second.mouths[stem_of[&o]].clone();
        let on = |c: u32| plains.owner.get(&c) == Some(&o);
        let m = mouth.stem.len();
        let Some(exit) = mouth.stem.iter().rposition(|&c| on(c)) else {
            continue;
        };
        if m < 3 || exit + 1 >= m {
            continue;
        }
        // The apex: where the stem comes onto the plain, the valley's mouth (HYD-137). The
        // backwater length up from where it leaves the plain is reported beside it: where it is
        // the shorter, the river is a steep one, whose fan delta spreads from the valley's mouth.
        let backwater_m = if plain.slope > 0.0 {
            plain.d_m / plain.slope
        } else {
            f64::INFINITY
        };
        let mut apex = exit;
        let mut run_m = 0.0;
        while apex > 0 && on(mouth.stem[apex - 1]) {
            let (dx, dz) = geo.delta(geo.pos(mouth.stem[apex - 1]), geo.pos(mouth.stem[apex]));
            run_m += (dx * dx + dz * dz).sqrt() * inp.m_per_block_h;
            apex -= 1;
        }
        // The front, by bearing from the apex: the plain's cells beside the sea's water ahead of
        // the apex, within a quarter turn of the outflow (HYD-138).
        let ap = geo.pos(mouth.stem[apex]);
        let dir = outflow(&geo, &mouth.stem[apex..], m - apex);
        let bearing = |c: u32| {
            let (dx, dz) = geo.delta(ap, geo.pos(c));
            (dir.0 * dz - dir.1 * dx).atan2(dir.0 * dx + dir.1 * dz)
        };
        let mut front: Vec<(f64, u32)> = plain
            .cells
            .iter()
            .filter(|&&c| (0..8).any(|k| geo.nb(c as usize, k).is_some_and(|nb| reach[nb])))
            .map(|&c| (bearing(c), c))
            .filter(|&(b, _)| b.abs() <= std::f64::consts::FRAC_PI_2) // ledger: HYD-138
            .collect();
        front.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let first_v = second.rivers.chains[mouth.chain as usize].first as usize;
        let q_apex = f64::from(second.rivers.vertices[first_v + apex].q);
        let front_blocks = front.len() as f64 * geo.r;
        // As many mouths as fit the front `delta_mouth_spacing` channel widths apart (HYD-138).
        let mut mouths_n = 1usize;
        for k in 2..=p.delta_max_mouths as usize {
            let need =
                k as f64 * p.delta_mouth_spacing * p.width(q_apex / k as f64, inp.width_scale);
            if need <= front_blocks {
                mouths_n = k;
            }
        }
        let mut split = Split {
            plain: o,
            chain: mouth.chain,
            apex_blocks: run_m / inp.m_per_block_h,
            backwater_m: if backwater_m.is_finite() {
                backwater_m.round()
            } else {
                -1.0
            },
            backwater_inside: backwater_m < run_m,
            front_cells: front.len() as u32,
            fit: mouths_n as u32,
            mouths: 1,
            added: 0,
        };
        if mouths_n < 2 || front.len() < mouths_n {
            out.push(split);
            continue;
        }
        let target = |s: usize| front[((2 * s + 1) * front.len()) / (2 * mouths_n)].1;
        let seg_of = |c: u32| {
            let b = bearing(c);
            let i = front.partition_point(|f| f.0 < b);
            (i * mouths_n / front.len()).min(mouths_n - 1)
        };
        let mut branches = vec![Branch {
            cells: mouth.stem[apex..].to_vec(),
            parent: None,
            on_plain: exit - apex + 1,
            seg: seg_of(mouth.stem[exit]),
            share: Vec::new(),
        }];
        let mut used: HashSet<u32> = branches[0].cells.iter().copied().collect();
        // Halve the front: the branch keeps the half its mouth is in, a new branch from the split
        // point takes the other half's middle (HYD-139, HYD-140).
        let mut work: Vec<(usize, usize, usize, usize)> = vec![(0, 0, mouths_n, 0)];
        while let Some((b, lo, hi, at)) = work.pop() {
            if hi - lo < 2 || at + 1 >= branches[b].on_plain {
                continue;
            }
            let mid = (lo + hi) / 2;
            let own = branches[b].seg.clamp(lo, hi - 1);
            let (keep, give) = if own < mid {
                ((lo, mid), (mid, hi))
            } else {
                ((mid, hi), (lo, mid))
            };
            let t_seg = (give.0 + give.1 - 1) / 2;
            let start = branches[b].cells[at];
            let goal = target(t_seg);
            let Some(mut path) = shortest(&geo, start, goal, |c| on(c) || c == start, &used) else {
                continue;
            };
            // From the front cell across the beach band (if any) to the first cell of the open
            // sea, as every sea mouth ends (HYD-144).
            let Some(tail) = to_open(&geo, goal, &to_sea, &used) else {
                continue;
            };
            let plain_len = path.len();
            path.extend(tail);
            if path.len() < 3 {
                continue;
            }
            let span = (hi - lo) as f64;
            let base = branches[b].share_at(at);
            branches[b]
                .share
                .push((at + 1, base * (keep.1 - keep.0) as f64 / span));
            for &c in &path {
                used.insert(c);
            }
            branches.push(Branch {
                cells: path,
                parent: Some((b, at)),
                on_plain: plain_len,
                seg: t_seg,
                share: vec![(0, base * (give.1 - give.0) as f64 / span)],
            });
            let nb = branches.len() - 1;
            let b_len = branches[b].on_plain;
            work.push((b, keep.0, keep.1, at + (b_len - at) / 2));
            work.push((nb, give.0, give.1, plain_len / 2));
        }
        // The main stem's own vertices below its splits carry their share.
        let main = &branches[0];
        for v in 0..main.cells.len() {
            let s = main.share_at(v);
            if s < 1.0 {
                let vx = &mut second.rivers.vertices[first_v + apex + v];
                let q = f64::from(vx.q) * s;
                vx.q = q as f32;
                vx.hw = (p.width(q, inp.width_scale) / 2.0) as f32;
                vx.depth = p.depth(q) as f32;
            }
        }
        // The new chains, each after its parent.
        let mut chain_id: Vec<u32> = vec![mouth.chain];
        let mut first_of: Vec<usize> = vec![first_v + apex];
        let half = (p.smoothing_blocks / geo.r).round() as usize;
        for br in branches.iter().skip(1) {
            let (pb, at) = br.parent.expect("a new branch has a parent");
            let pv = second.rivers.vertices[first_of[pb] + at];
            let mut pts: Vec<(f64, f64)> = Vec::with_capacity(br.cells.len());
            pts.push((f64::from(pv.x), f64::from(pv.z)));
            for &c in &br.cells[1..] {
                let prev = *pts.last().expect("a first point");
                let (dx, dz) = geo.delta(prev, geo.pos(c));
                pts.push((prev.0 + dx, prev.1 + dz));
            }
            if half > 0 && pts.len() > 2 {
                network::smooth(&mut pts, half);
            }
            let first = second.rivers.vertices.len() as u32;
            for (v, &(x, z)) in pts.iter().enumerate() {
                let q = q_apex * br.share_at(v);
                let (hw, depth) = (p.width(q, inp.width_scale) / 2.0, p.depth(q));
                second.rivers.vertices.push(RiverVertex {
                    x: if v == 0 { pv.x } else { x as f32 },
                    z: if v == 0 { pv.z } else { z as f32 },
                    w: if v == 0 { pv.w } else { sea_w.min(pv.w) },
                    hw: hw as f32,
                    depth: depth as f32,
                    q: q as f32,
                });
            }
            let mut ch = Chain::new(first, pts.len() as u32, Mouth::Sea, u32::MAX);
            ch.from = chain_id[pb];
            chain_id.push(second.rivers.chains.len() as u32);
            first_of.push(first as usize);
            second.rivers.chains.push(ch);
            split.added += 1;
        }
        split.mouths = 1 + split.added;
        out.push(split);
    }
    if out.iter().any(|s| s.added > 0) {
        second.rivers.build_index();
    }
    out
}

/// The shortest 8-connected path from `start` to `goal` over cells where `inside` holds, a step
/// costing [`PATH_SHARED_COST`] times more on a cell in `used` (HYD-144). Includes both ends.
fn shortest(
    geo: &Geo,
    start: u32,
    goal: u32,
    inside: impl Fn(u32) -> bool,
    used: &HashSet<u32>,
) -> Option<Vec<u32>> {
    let mut dist: HashMap<u32, u64> = HashMap::new();
    let mut prev: HashMap<u32, u32> = HashMap::new();
    let mut heap: BinaryHeap<Reverse<(u64, u32)>> = BinaryHeap::new();
    dist.insert(start, 0);
    heap.push(Reverse((0, start)));
    while let Some(Reverse((d, c))) = heap.pop() {
        if c == goal {
            break;
        }
        if dist.get(&c).is_some_and(|&b| d > b) {
            continue;
        }
        for k in 0..8 {
            let Some(nb) = geo.nb(c as usize, k) else {
                continue;
            };
            let nb = nb as u32;
            if !inside(nb) {
                continue;
            }
            let w = if used.contains(&nb) && nb != goal {
                PATH_SHARED_COST
            } else {
                1.0
            };
            let nd = d + (Geo::step(k) * w * 1000.0).round() as u64;
            if dist.get(&nb).is_none_or(|&b| nd < b) {
                dist.insert(nb, nd);
                prev.insert(nb, c);
                heap.push(Reverse((nd, nb)));
            }
        }
    }
    if !dist.contains_key(&goal) {
        return None;
    }
    let mut path = vec![goal];
    let mut c = goal;
    while c != start {
        c = prev[&c];
        path.push(c);
    }
    path.reverse();
    Some(path)
}

/// Each sea-reached cell's count of 8-connected steps over sea-reached ground (`reach`: the beach
/// band and the open sea) to the nearest `open` cell, `u32::MAX` on every other cell (HYD-144).
fn sea_distance(geo: &Geo, reach: &[bool], open: &[bool]) -> Vec<u32> {
    let n = geo.w * geo.h;
    let mut dist = vec![u32::MAX; n];
    let mut queue = std::collections::VecDeque::new();
    for c in 0..n {
        if open[c] {
            dist[c] = 0;
            // Only the open sea's edge starts the search.
            if (0..8).any(|k| geo.nb(c, k).is_some_and(|nb| reach[nb] && !open[nb])) {
                queue.push_back(c);
            }
        }
    }
    while let Some(c) = queue.pop_front() {
        for k in 0..8 {
            if let Some(nb) = geo.nb(c, k)
                && reach[nb]
                && dist[nb] == u32::MAX
            {
                dist[nb] = dist[c] + 1;
                queue.push_back(nb);
            }
        }
    }
    dist
}

/// The way from a plain's front cell `from` to the open sea: down the step counts `to_sea`
/// ([`sea_distance`]), one step nearer each cell, over cells no channel of the plain takes yet;
/// none where every way down is taken (HYD-144). Excludes `from`, includes the open cell.
fn to_open(geo: &Geo, from: u32, to_sea: &[u32], used: &HashSet<u32>) -> Option<Vec<u32>> {
    let mut tail = Vec::new();
    let mut c = from as usize;
    let mut here = u32::MAX;
    loop {
        let mut best: Option<(u32, usize)> = None;
        for k in 0..8 {
            let Some(nb) = geo.nb(c, k) else { continue };
            let d = to_sea[nb];
            if d < here && !used.contains(&(nb as u32)) && best.is_none_or(|b| (d, nb) < b) {
                best = Some((d, nb));
            }
        }
        let (d, nb) = best?;
        tail.push(nb as u32);
        if d == 0 {
            return Some(tail);
        }
        here = d;
        c = nb;
    }
}

/// `notes.bake.water.stats.deltas`: counts, areas, the largest plains, and Earth's figures beside
/// them.
pub fn stats(plains: &Plains, splits: &[Split], land_cells: u64, cell_km2: f64) -> Value {
    let cells: u64 = plains.plains.iter().map(|p| p.cells.len() as u64).sum();
    let area_km2 = cells as f64 * cell_km2;
    let added: u32 = splits.iter().map(|s| s.added).sum();
    let hist = |of: fn(&Split) -> u32| {
        let mut h = [0u32; 9];
        for s in splits {
            h[(of(s) as usize).min(8)] += 1;
        }
        h
    };
    let mouths_hist = hist(|s| s.mouths);
    let fit_hist = hist(|s| s.fit);
    let split_of: HashMap<u32, &Split> = splits.iter().map(|s| (s.plain, s)).collect();
    let mut by_q: Vec<usize> = (0..plains.plains.len()).collect();
    by_q.sort_by(|&a, &b| {
        plains.plains[b]
            .q_m3s
            .total_cmp(&plains.plains[a].q_m3s)
            .then(a.cmp(&b))
    });
    let largest: Vec<Value> = by_q
        .iter()
        .take(25)
        .map(|&id| {
            let p = &plains.plains[id];
            let s = split_of.get(&(id as u32));
            json!({
                "chain": p.chain,
                "q_m3s": (p.q_m3s * 10.0).round() / 10.0,
                "area_km2": p.area_km2.round(),
                "relief_km": (p.relief_km * 100.0).round() / 100.0,
                "t_mean_c": (p.t_mean_c * 10.0).round() / 10.0,
                "lake_share": (p.lake_share * 1000.0).round() / 1000.0,
                "qs_kg_s": (p.qs_kg_s * 10.0).round() / 10.0,
                "a_star_km2": (p.a_star_km2 * 10.0).round() / 10.0,
                "d_m": (p.d_m * 10.0).round() / 10.0,
                "slope_m_per_km": (p.slope * 1e4).round() / 10.0,
                "plain_cells": p.cells.len(),
                "plain_km2": (p.cells.len() as f64 * cell_km2 * 10.0).round() / 10.0,
                "pocket_cells": p.pocket_cells,
                "mouths": s.map_or(0, |s| s.mouths),
                "apex_blocks": s.map_or(0.0, |s| (s.apex_blocks * 10.0).round() / 10.0),
                "backwater_m": s.map_or(-1.0, |s| s.backwater_m),
                "front_cells": s.map_or(0, |s| s.front_cells),
                "fit": s.map_or(0, |s| s.fit),
            })
        })
        .collect();
    json!({
        "mouths_with_load": plains.mouths,
        "deltas": plains.plains.len(),
        "plain_cells": cells,
        "plain_km2": area_km2.round(),
        "share_of_land": if land_cells > 0 {
            json!(((cells as f64 / land_cells as f64) * 1e5).round() / 1e5)
        } else {
            Value::Null
        },
        "pocket_cells": plains.plains.iter().map(|p| u64::from(p.pocket_cells)).sum::<u64>(),
        "lagoons": plains.lagoons,
        "lagoon_cells": plains.lagoon_cells,
        "distributaries_added": added,
        "deltas_by_mouths": mouths_hist,
        "deltas_by_mouths_fit": fit_hist,
        "backwater_inside_plain": splits.iter().filter(|s| s.backwater_inside).count(),
        "earth": {
            "delta_share_of_land": 0.0057,
            "delta_area_km2": 847_936,
            "share_of_rivers_with_delta": 0.40,
            "sources": "Edmonds et al. 2020 (847,936 km², 0.57 % of land; HYD-143), Caldwell et al. 2019 (40 % of 5,399 rivers; HYD-143)"
        },
        "largest": largest,
    })
}

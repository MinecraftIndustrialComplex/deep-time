// SPDX-License-Identifier: LGPL-3.0-or-later
//! Alluvial fans (V1 gap 5, stage 4; docs/science/rivers-and-lakes.md, "Fans and braids"): the
//! cone a river builds where it leaves steep ground for gentler.
//!
//! "An alluvial fan is a stream deposit whose surface forms a segment of a cone that radiates
//! downslope from the point where the stream channel emerges from a mountainous area" (Bull 1964,
//! p. 93). Here, on the first network's chains ([`network::Output::walks`]):
//!
//! - **Size and slope** (Bull 1964, Fig. 54, the sandstone basins: HYD-165, HYD-166). A river
//!   draining A_D builds a fan of area A_F = 1.3 A_D^0.88 sloping S_F = 0.022 A_D^−0.32 (square
//!   miles). The fan is a cone of that slope about its apex.
//! - **Where** (HYD-167). The apex is a cell of the chain where the ground upstream is at least as
//!   steep as the fan (Bull 1964, p. 100: steep fans at the mouths of steep canyons) and the
//!   ground downstream gentler than it (the cone then stands on it; Ganti et al. 2014: fans grow
//!   at an abrupt decrease in bed slope), both measured over the fan's free length (below).
//!   Along a chain the apexes are taken where the cone would stand thickest, a fan's length apart.
//! - **Extent** (HYD-168, HYD-169). The cone is laid where it stands over the ground, on the cells
//!   joined to its apex across land that is no lake, playa, delta plain or channel of a larger
//!   river ("the radii of a fan are restricted by adjacent fans, and by stream deposits at the toe
//!   of the fan", Bull 1964, p. 94), and nowhere thicker than over its apex: where the ground falls
//!   away from the apex faster than the fan (off a plateau's edge, into a deeper valley) the fan
//!   ends, as the cone would otherwise bury lower ground far below it. Its height over the apex is
//!   found so that it covers A_F, but
//!   at most the height of a free fan: the cone of A_F spread over a quarter circle on flat ground
//!   (the median sector of Stock et al.'s 2008 four fans), so a fan hemmed in by higher ground is
//!   smaller than Bull's. A fan less than a block high at its apex is not drawn (HYD-170).
//!   Overlapping fans take the higher cone.
//! - **The planet's metric.** Distances are the planet's: on a Lambert frame a block east is cos φ
//!   of the planet's and a block south 1 / cos φ, so a fan is round on the planet
//!   (round-features.md, SCL-230). The frame's cells are of equal area, so a fan's area is its
//!   cell count.
//!
//! The water part raises the ground to the fans before it runs the network again, which then
//! runs its rivers down the cones; the fans are no floodplain.

use std::collections::{HashMap, VecDeque};

use deeptime_format::{GridDesc, Projection};
use rayon::prelude::*;
use serde_json::{Value, json};

use super::network;

/// Bull (1964, Fig. 54A; p. 126): fan area A_F = 1.3 A_D^0.88, both in square miles, for the
/// drainage basins of 58 to 68 % sandstone (0.5 to 296 mi² of drainage; the mudstone and shale
/// basins' 2.4 A_D^0.88 covers 0.2 to 29 mi² only).
pub const FAN_AREA_COEF: f64 = 1.3; // ledger: HYD-165
pub const FAN_AREA_EXP: f64 = 0.88; // ledger: HYD-165
/// Bull (1964, Fig. 54B; p. 126): the overall fan slope, apex to outer margin, S_F = 0.022
/// A_D^−0.32 (A_D in square miles), the same sandstone basins.
pub const FAN_SLOPE_COEF: f64 = 0.022; // ledger: HYD-166
pub const FAN_SLOPE_EXP: f64 = -0.32; // ledger: HYD-166
/// The sector a free fan spreads over, radians: the median of 2 A / L² over Stock et al.'s (2008,
/// Table 2) four fans' areas and arc lengths, 0.81, 1.33, 1.93 and 2.69 (a quarter circle).
pub const FREE_SECTOR_RAD: f64 = std::f64::consts::FRAC_PI_2; // ledger: HYD-168
/// The least height of a fan over the ground at its apex, blocks: thinner deposits are under the
/// world's resolution (game-choice: agent, 2026-10-24).
pub const MIN_THICKNESS_BLOCKS: f64 = 1.0; // ledger: HYD-170 (game-choice)

/// Square kilometres per square mile.
const KM2_PER_MI2: f64 = 2.589_988;
/// The least cos φ the Lambert metric is taken at (the pole rows).
const COS_MIN: f64 = 0.05;
/// Bisection steps for a fan's height.
const BISECT: usize = 30;

/// A fan's area, km², for a drainage area in km² (HYD-165).
pub fn fan_area_km2(drainage_km2: f64) -> f64 {
    FAN_AREA_COEF * (drainage_km2 / KM2_PER_MI2).max(0.0).powf(FAN_AREA_EXP) * KM2_PER_MI2
}

/// A fan's overall slope for a drainage area in km² (HYD-166).
pub fn fan_slope(drainage_km2: f64) -> f64 {
    FAN_SLOPE_COEF * (drainage_km2 / KM2_PER_MI2).max(1e-9).powf(FAN_SLOPE_EXP)
}

/// The free fan's length, m: the radius whose [`FREE_SECTOR_RAD`] sector covers the area.
pub fn free_length_m(drainage_km2: f64) -> f64 {
    (2.0 * fan_area_km2(drainage_km2) * 1e6 / FREE_SECTOR_RAD).sqrt()
}

/// What the fans read besides the first network.
pub struct Input<'a> {
    pub grid: &'a GridDesc,
    /// The ground the network ran on, 1/16 block.
    pub elev: &'a [i16],
    pub sea_raw: i32,
    pub m_per_block_h: f64,
    pub m_per_block_v: f64,
    /// The planet whose Lambert blocks the grid is in; None for a frame drawn true to shape.
    pub lambert: Option<Projection>,
    /// Cells no fan may cover besides lakes, playas and the sea (the delta plains).
    pub taken: &'a (dyn Fn(u32) -> bool + Sync),
}

/// One fan.
#[derive(Clone, Debug)]
pub struct Fan {
    /// The chain whose river built it, and its apex cell and block position.
    pub chain: u32,
    pub apex: u32,
    pub x: f64,
    pub z: f64,
    /// Drainage area at the apex, km²; Bull's fan area for it, km²; the fan's slope.
    pub drainage_km2: f64,
    pub bull_km2: f64,
    pub slope: f64,
    /// The ground's slopes over the free length above and below the apex.
    pub slope_up: f64,
    pub slope_down: f64,
    /// The cone's top over the apex's ground, m, and the free fan's (the bound).
    pub thickness_m: f64,
    pub free_thickness_m: f64,
    /// The cone's height at the apex, 1/16 block, and its fall per block of planet.
    pub top_raw: f64,
    pub fall_raw: f64,
    /// Its cells, and the cone's height on each, 1/16 block.
    pub cells: Vec<u32>,
    pub height: Vec<i16>,
    /// Its volume over the ground, km³.
    pub volume_km3: f64,
}

/// The fans of one raster.
#[derive(Default)]
pub struct Fans {
    pub fans: Vec<Fan>,
    /// Candidates looked at, and why the others were not drawn.
    pub candidates: u64,
    pub thin: u64,
    /// Fans whose cone met higher ground or a blocker before Bull's area.
    pub hemmed: u64,
    /// The grid's circumference and metric, for [`Fans::cone_raw`].
    c: f64,
    lambert: Option<Projection>,
}

impl Fans {
    /// The ground with the fans on it (each cell at least its highest cone).
    pub fn apply(&self, elev: &mut [i16]) {
        for f in &self.fans {
            for (&c, &h) in f.cells.iter().zip(&f.height) {
                let e = &mut elev[c as usize];
                *e = (*e).max(h);
            }
        }
    }

    /// Every fan cell with its highest cone (the ground there once applied), 1/16 block, sorted
    /// by cell.
    pub fn ground(&self) -> Vec<(u32, i16)> {
        let mut v: Vec<(u32, i16)> = self
            .fans
            .iter()
            .flat_map(|f| f.cells.iter().copied().zip(f.height.iter().copied()))
            .collect();
        v.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
        v.dedup_by_key(|e| e.0);
        v
    }

    /// Every fan cell, sorted.
    pub fn cells(&self) -> Vec<u32> {
        let mut v: Vec<u32> = self
            .fans
            .iter()
            .flat_map(|f| f.cells.iter().copied())
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    /// The fan's cone at block (x, z), 1/16 block.
    pub fn cone_raw(&self, f: &Fan, x: f64, z: f64) -> f64 {
        f.top_raw - f.fall_raw * planet_dist(self.c, self.lambert.as_ref(), f.x, f.z, x, z)
    }
}

/// The planet's distance between two points of a frame of circumference `c`, blocks of planet.
fn planet_dist(c: f64, lambert: Option<&Projection>, ax: f64, az: f64, bx: f64, bz: f64) -> f64 {
    let mut dx = bx - ax;
    dx -= c * (dx / c).round();
    let (ex, ez) = match lambert {
        Some(p) => {
            let cs = p.cos_lat(0.5 * (az + bz)).max(COS_MIN);
            (cs, 1.0 / cs)
        }
        None => (1.0, 1.0),
    };
    (dx * ex).hypot((bz - az) * ez)
}

/// A walk's profile: the cells and, per cell, its position along the course (blocks of planet)
/// and its filled surface (m).
struct Profile {
    cells: Vec<u32>,
    s: Vec<f64>,
    z_m: Vec<f64>,
}

/// The surface at distance `s` along a profile (linear between cells, held past its ends).
fn z_at(p: &Profile, s: f64) -> f64 {
    let n = p.s.len();
    if s <= p.s[0] {
        return p.z_m[0];
    }
    if s >= p.s[n - 1] {
        return p.z_m[n - 1];
    }
    let i = p.s.partition_point(|&x| x <= s).clamp(1, n - 1) - 1;
    let len = p.s[i + 1] - p.s[i];
    let t = if len > 0.0 { (s - p.s[i]) / len } else { 0.0 };
    p.z_m[i] + (p.z_m[i + 1] - p.z_m[i]) * t
}

/// The fans on the first network's chains.
pub fn find(inp: &Input<'_>, net: &network::Output) -> Fans {
    let g = inp.grid;
    let (w, h) = (g.w as usize, g.h as usize);
    let (ox, oz, r) = (
        g.origin_block_x as f64,
        g.origin_block_z as f64,
        f64::from(g.r),
    );
    // The planet's circumference (a window of a Lambert frame is narrower than it).
    let lambert = inp.lambert;
    let c = lambert.map_or(g.circumference() as f64, |p| p.circumference());
    let pos = |cell: u32| -> (f64, f64) {
        let cu = cell as usize;
        (ox + (cu % w) as f64 * r, oz + (cu / w) as f64 * r)
    };
    let dist = |a: (f64, f64), b: (f64, f64)| planet_dist(c, lambert.as_ref(), a.0, a.1, b.0, b.1);
    let (mh, mv) = (inp.m_per_block_h, inp.m_per_block_v);
    let walks = &net.walks;
    // Each river cell's chain and place on it (its own cells, not the end it runs into), and
    // the drainage area there.
    let mut on: HashMap<u32, (u32, u32, f32)> = HashMap::new();
    for (k, wk) in walks.iter().enumerate() {
        let m = wk.cells.len();
        for i in usize::from(wk.lead)..m.saturating_sub(1) {
            on.insert(wk.cells[i], (k as u32, i as u32, wk.area_km2[i]));
        }
    }
    let cell_km2 = (r * mh).powi(2) / 1e6;
    let free = |cu: usize| -> bool {
        net.kind.get(cu).is_none_or(|&k| k == network::kind::NONE)
            && (i32::from(inp.elev[cu]) >= inp.sea_raw
                || net.closed_basin.get(cu).is_some_and(|&b| b != 0))
            && !(inp.taken)(cu as u32)
    };

    // The profile of each chain, run on down its parents far enough for its largest fan.
    let profile = |k: usize| -> Profile {
        let wk = &walks[k];
        let mut cells: Vec<u32> = wk.cells[usize::from(wk.lead)..].to_vec();
        let mut fill: Vec<i16> = wk.fill[usize::from(wk.lead)..].to_vec();
        let reach = free_length_m(f64::from(*wk.area_km2.last().unwrap_or(&0.0))) / mh;
        let mut s = vec![0.0];
        for i in 1..cells.len() {
            s.push(s[i - 1] + dist(pos(cells[i - 1]), pos(cells[i])));
        }
        let own_end = *s.last().unwrap_or(&0.0);
        let mut guard = 0;
        while *s.last().unwrap_or(&0.0) - own_end < reach && guard < walks.len() {
            guard += 1;
            let Some(&end) = cells.last() else { break };
            let Some(&(p, i, _)) = on.get(&end) else {
                break;
            };
            let pw = &walks[p as usize];
            for j in i as usize + 1..pw.cells.len() {
                let last = *cells.last().unwrap();
                s.push(s.last().unwrap() + dist(pos(last), pos(pw.cells[j])));
                cells.push(pw.cells[j]);
                fill.push(pw.fill[j]);
            }
        }
        Profile {
            cells,
            s,
            z_m: fill.iter().map(|&f| f64::from(f) / 16.0 * mv).collect(),
        }
    };

    // Candidate apexes per chain, the thickest a free length apart.
    struct Cand {
        chain: u32,
        cell: u32,
        drainage: f64,
        slope: f64,
        up: f64,
        down: f64,
        top_raw: f64,
        free_m: f64,
    }
    let per_chain: Vec<(u64, Vec<Cand>)> = (0..walks.len())
        .into_par_iter()
        .map(|k| {
            let wk = &walks[k];
            let own = wk.cells.len() - usize::from(wk.lead) - 1;
            if own < 2 {
                return (0, Vec::new());
            }
            let p = profile(k);
            let mut cands: Vec<(f64, usize, Cand)> = Vec::new();
            let mut looked = 0u64;
            for i in 1..own {
                let cell = p.cells[i];
                let cu = cell as usize;
                let a = f64::from(wk.area_km2[i + usize::from(wk.lead)]);
                if a <= 0.0 || !free(cu) {
                    continue;
                }
                let (sf, len_m) = (fan_slope(a), free_length_m(a));
                let len = len_m / mh;
                let up_len = len.min(p.s[i]);
                if up_len <= 0.0 {
                    continue;
                }
                let up = (z_at(&p, p.s[i] - up_len) - p.z_m[i]) / (up_len * mh);
                let down_len = len.min(p.s[p.s.len() - 1] - p.s[i]);
                if down_len <= 0.0 {
                    continue;
                }
                let down = (p.z_m[i] - z_at(&p, p.s[i] + down_len)) / (down_len * mh);
                if up < sf || down >= sf {
                    continue;
                }
                looked += 1;
                let thick = (sf - down) * len_m;
                if thick < MIN_THICKNESS_BLOCKS * mv {
                    continue;
                }
                cands.push((
                    thick,
                    i,
                    Cand {
                        chain: k as u32,
                        cell,
                        drainage: a,
                        slope: sf,
                        up,
                        down,
                        top_raw: p.z_m[i] / mv * 16.0,
                        free_m: len_m,
                    },
                ));
            }
            cands.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
            let mut kept: Vec<(f64, f64, Cand)> = Vec::new();
            for (_, i, cd) in cands {
                let (si, li) = (p.s[i], cd.free_m / mh);
                if kept.iter().all(|(s, l, _)| (si - s).abs() >= li.max(*l)) {
                    kept.push((si, li, cd));
                }
            }
            (looked, kept.into_iter().map(|(_, _, cd)| cd).collect())
        })
        .collect();
    let candidates: u64 = per_chain.iter().map(|(n, _)| n).sum();
    let cands: Vec<Cand> = per_chain.into_iter().flat_map(|(_, v)| v).collect();

    // Each fan's cone: the height over its apex that covers Bull's area, at most the free fan's.
    let grow = |cd: &Cand, thick_m: f64, cells: &mut Vec<u32>, heights: &mut Vec<i16>| {
        cells.clear();
        heights.clear();
        let apex = pos(cd.cell);
        let top = cd.top_raw + thick_m / mv * 16.0;
        let fall = cd.slope * mh / mv * 16.0;
        // Nowhere thicker than over the apex: the ground no lower than a cone of the fan's slope
        // through the apex's ground.
        let cap = top - f64::from(inp.elev[cd.cell as usize]);
        let mut seen: HashMap<u32, ()> = HashMap::new();
        let mut queue = VecDeque::from([cd.cell]);
        seen.insert(cd.cell, ());
        while let Some(cell) = queue.pop_front() {
            let cu = cell as usize;
            let cone = top - fall * dist(apex, pos(cell));
            let e = f64::from(inp.elev[cu]);
            if cone <= e || cone - e > cap || !free(cu) {
                continue;
            }
            if let Some(&(k, _, a)) = on.get(&cell)
                && k != cd.chain
                && f64::from(a) > cd.drainage
            {
                continue;
            }
            cells.push(cell);
            heights.push(cone.floor().clamp(f64::from(i16::MIN), f64::from(i16::MAX)) as i16);
            let (i, j) = ((cu % w) as i64, (cu / w) as i64);
            for (di, dj) in [(1i64, 0i64), (-1, 0), (0, 1), (0, -1)] {
                let nj = j + dj;
                if nj < 0 || nj >= h as i64 {
                    continue;
                }
                let ni = if g.wraps_x {
                    (i + di).rem_euclid(w as i64)
                } else if (0..w as i64).contains(&(i + di)) {
                    i + di
                } else {
                    continue;
                };
                let nb = (nj as usize * w + ni as usize) as u32;
                if seen.insert(nb, ()).is_none() {
                    queue.push_back(nb);
                }
            }
        }
    };
    let built: Vec<Option<(Fan, bool)>> = cands
        .par_iter()
        .map(|cd| {
            let (mut cells, mut heights) = (Vec::new(), Vec::new());
            let want = (fan_area_km2(cd.drainage) / cell_km2).round().max(1.0) as usize;
            let t_max = cd.slope * cd.free_m;
            grow(cd, t_max, &mut cells, &mut heights);
            let hemmed = cells.len() < want;
            let thick = if hemmed {
                t_max
            } else {
                let (mut lo, mut hi) = (0.0, t_max);
                for _ in 0..BISECT {
                    let mid = 0.5 * (lo + hi);
                    grow(cd, mid, &mut cells, &mut heights);
                    if cells.len() < want {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                hi
            };
            if thick < MIN_THICKNESS_BLOCKS * mv {
                return None;
            }
            grow(cd, thick, &mut cells, &mut heights);
            let volume_km3 = cells
                .iter()
                .zip(&heights)
                .map(|(&cl, &ht)| f64::from(ht - inp.elev[cl as usize]).max(0.0) / 16.0 * mv)
                .sum::<f64>()
                * cell_km2
                / 1000.0;
            let (x, z) = pos(cd.cell);
            Some((
                Fan {
                    chain: cd.chain,
                    apex: cd.cell,
                    x,
                    z,
                    drainage_km2: cd.drainage,
                    bull_km2: fan_area_km2(cd.drainage),
                    slope: cd.slope,
                    slope_up: cd.up,
                    slope_down: cd.down,
                    thickness_m: thick,
                    free_thickness_m: t_max,
                    top_raw: cd.top_raw + thick / mv * 16.0,
                    fall_raw: cd.slope * mh / mv * 16.0,
                    cells,
                    height: heights,
                    volume_km3,
                },
                hemmed,
            ))
        })
        .collect();
    let mut fans = Fans {
        candidates,
        c,
        lambert,
        ..Fans::default()
    };
    for b in built {
        match b {
            Some((f, hemmed)) => {
                fans.hemmed += u64::from(hemmed);
                fans.fans.push(f);
            }
            None => fans.thin += 1,
        }
    }
    fans
}

/// `stats.fans`: the fans against Bull's laws and the land.
pub fn stats(fans: &Fans, land_cells: u64, cell_km2: f64, m_per_block_v: f64) -> Value {
    let n = fans.fans.len();
    let cells = fans.cells().len() as u64;
    let q = |mut v: Vec<f64>, p: f64| -> Value {
        if v.is_empty() {
            return Value::Null;
        }
        v.sort_by(f64::total_cmp);
        let i = ((v.len() - 1) as f64 * p).round() as usize;
        json!((v[i] * 1e3).round() / 1e3)
    };
    let ratio: Vec<f64> = fans
        .fans
        .iter()
        .map(|f| f.cells.len() as f64 * cell_km2 / f.bull_km2)
        .collect();
    let thick: Vec<f64> = fans
        .fans
        .iter()
        .map(|f| f.thickness_m / m_per_block_v)
        .collect();
    let area: Vec<f64> = fans
        .fans
        .iter()
        .map(|f| f.cells.len() as f64 * cell_km2)
        .collect();
    let drainage: Vec<f64> = fans.fans.iter().map(|f| f.drainage_km2).collect();
    let slope: Vec<f64> = fans.fans.iter().map(|f| f.slope).collect();
    // Bull's fitted range of drainage, 0.2 to 296 mi².
    let beyond = fans
        .fans
        .iter()
        .filter(|f| f.drainage_km2 > 296.0 * KM2_PER_MI2)
        .count();
    json!({
        "fans": n,
        "candidates": fans.candidates,
        "thin": fans.thin,
        "hemmed": fans.hemmed,
        "cells": cells,
        "area_km2": (cells as f64 * cell_km2).round(),
        "share_of_land": if land_cells > 0 { json!((cells as f64 / land_cells as f64 * 1e5).round() / 1e5) } else { Value::Null },
        "volume_km3": (fans.fans.iter().map(|f| f.volume_km3).sum::<f64>() * 10.0).round() / 10.0,
        "area_over_bull": {"p10": q(ratio.clone(), 0.1), "p50": q(ratio.clone(), 0.5), "p90": q(ratio, 0.9)},
        "thickness_blocks": {"p50": q(thick.clone(), 0.5), "p90": q(thick.clone(), 0.9), "max": q(thick, 1.0)},
        "fan_km2": {"p10": q(area.clone(), 0.1), "p50": q(area.clone(), 0.5), "p90": q(area.clone(), 0.9), "max": q(area, 1.0)},
        "drainage_km2": {"p10": q(drainage.clone(), 0.1), "p50": q(drainage.clone(), 0.5), "p90": q(drainage.clone(), 0.9), "max": q(drainage, 1.0)},
        "slope": {"p10": q(slope.clone(), 0.1), "p50": q(slope.clone(), 0.5), "p90": q(slope, 0.9)},
        "beyond_bull_range": beyond,
    })
}

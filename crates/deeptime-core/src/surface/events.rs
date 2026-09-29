// SPDX-License-Identifier: LGPL-3.0-or-later
//! The surface's events (plan §5.5; `EVENTS.md`, source `deeptime:surface`), decided once per
//! frame from the frame-mean rates:
//!
//! - `erosion_pulse`: a region of land eroding faster than `erosion_pulse_m_myr`, at least
//!   `event_min_area_km2`, tracked from frame to frame (a region whose centroid stays within
//!   [`MATCH_KM`] continues the pulse) and written when it ends, spanning its whole life;
//! - `basin_subsidence`: the same for regions aggrading faster than `basin_m_myr` (sea floor
//!   deeper than 2 km excluded), with the regime that dominates it as `setting`;
//! - `marine_transgression` / `marine_regression`: the share of continental crust under the sea,
//!   followed as a zigzag — a move of at least `transgression_frac` that then reverses by as much
//!   is written as one event from its start to its turning point.
//!
//! Every place is a point (the region's centroid at its peak, or the centroid of the cells the sea
//! gained or lost). Causes: the nearest `orogeny_phase`, `collision` or `subduction_onset` (pulses)
//! or `rift_onset`, `collision`, `orogeny_phase`, `subduction_onset`, `ridge_birth` (basins) active
//! within [`CAUSE_KM`] and 20 Myr before the region began.

use deeptime_format::{EventId, Place};
use serde::{Deserialize, Serialize};

use super::grid::Dims;
use crate::deposits::fields::{cell_area_km2, lat_of_row};
use crate::error::CoreError;
use crate::plates::state::regime;
use crate::world::WorldState;

/// The source of the surface's events.
pub const SOURCE: &str = "deeptime:surface";
/// A region continues a tracked one whose centroid is this close, km on the planet.
pub const MATCH_KM: f64 = 250.0;
/// Causes are looked for this far away, km on the planet.
pub const CAUSE_KM: f64 = 400.0;

/// A region tracked from frame to frame.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub lat: f64,
    pub lon: f64,
    pub t_start: f64,
    pub t_last: f64,
    pub peak_rate: f64,
    pub peak_area_km2: f64,
    pub volume_km3: f64,
    pub setting: String,
    pub cause: Option<EventId>,
    /// Matched in the current frame.
    pub seen: bool,
}

/// The sea's zigzag.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SeaTrack {
    pub started: bool,
    /// +1 rising, −1 falling, 0 undecided.
    pub dir: i8,
    pub pivot_t: f64,
    pub pivot_f: f64,
    pub pivot_sea: f64,
    pub pivot_mask: Vec<u8>,
    pub ext_t: f64,
    pub ext_f: f64,
    pub ext_sea: f64,
    pub ext_mask: Vec<u8>,
}

/// Everything the events keep between frames.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EventState {
    pub pulses: Vec<Track>,
    pub basins: Vec<Track>,
    pub sea: SeaTrack,
    pub counts: [u32; 4],
}

/// A region found in one frame.
struct Region {
    lat: f64,
    lon: f64,
    area_km2: f64,
    volume_km3: f64,
    peak: f64,
    setting: u8,
}

/// Great-circle distance on a planet of radius `r_km`.
pub fn distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64, r_km: f64) -> f64 {
    let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
    let dl = (lon2 - lon1).to_radians();
    let c = p1.sin() * p2.sin() + p1.cos() * p2.cos() * dl.cos();
    c.clamp(-1.0, 1.0).acos() * r_km
}

/// Connected regions (8-connected) of cells where `on` holds, with their true areas.
#[allow(clippy::too_many_arguments)]
fn regions(
    d: &Dims,
    grid: &deeptime_format::GridDesc,
    m_per_block: f64,
    on: &dyn Fn(usize) -> bool,
    rate: &[f32],
    span_myr: f64,
    reg: &[f32],
    min_km2: f64,
) -> Vec<Region> {
    let n = d.cells();
    let mut seen = vec![false; n];
    let mut out = Vec::new();
    let mut todo = Vec::new();
    let area_row: Vec<f64> = (0..d.h)
        .map(|j| cell_area_km2(grid, j as u32, m_per_block))
        .collect();
    for s in 0..n {
        if seen[s] || !on(s) {
            continue;
        }
        seen[s] = true;
        todo.clear();
        todo.push(s);
        let (mut area, mut vol, mut peak) = (0.0, 0.0, 0.0f64);
        let mut v = glam::DVec3::ZERO;
        let mut settings = [0.0f64; 10];
        while let Some(c) = todo.pop() {
            let j = c / d.w;
            let a = area_row[j];
            area += a;
            let r = f64::from(rate[c]);
            vol += r * span_myr * a / 1.0e3; // m/Myr · Myr · km² → km³ / 1000
            peak = peak.max(r);
            let lat = lat_of_row(grid, j as u32);
            let lon = 360.0 * (c % d.w) as f64 / d.w as f64;
            v += crate::sphere::unit_from_lat_lon_deg(lat, lon) * a;
            let rg = reg[c];
            if rg.is_finite() && (rg as usize) < 10 {
                settings[rg as usize] += a;
            }
            for (m, _) in d.neighbours(c) {
                if !seen[m] && on(m) {
                    seen[m] = true;
                    todo.push(m);
                }
            }
        }
        if area < min_km2 {
            continue;
        }
        let (lat, lon) = crate::sphere::lat_lon_deg(v);
        let setting = settings
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(&a.0)))
            .map_or(0, |(k, _)| k as u8);
        out.push(Region {
            lat,
            lon,
            area_km2: area,
            volume_km3: vol,
            peak,
            setting,
        });
    }
    out
}

/// The nearest event of one of `kinds` active near (lat, lon) between `t0` and `t1` (years).
fn cause(
    world: &WorldState,
    kinds: &[&str],
    lat: f64,
    lon: f64,
    t0: f64,
    t1: f64,
    r_km: f64,
) -> Option<EventId> {
    let mut best: Option<(f64, EventId)> = None;
    for e in &world.events {
        if !kinds.contains(&e.kind.as_str()) || e.t_end < t0 || e.t_start > t1 {
            continue;
        }
        if let Place::Point { lat: la, lon: lo } = e.place {
            let dk = distance_km(lat, lon, f64::from(la), f64::from(lo), r_km);
            if dk <= CAUSE_KM && best.is_none_or(|b| dk < b.0) {
                best = Some((dk, e.id));
            }
        }
    }
    best.map(|b| b.1)
}

/// What one frame's decision needs.
pub struct FrameView<'a> {
    pub d: &'a Dims,
    pub grid: &'a deeptime_format::GridDesc,
    pub m_per_block: f64,
    pub r_km: f64,
    pub t_start: f64,
    pub t_end: f64,
    pub span_myr: f64,
    pub erosion: &'a [f32],
    pub deposition: &'a [f32],
    /// Elevation relative to the sea of the time (frame mean), m.
    pub elevation: &'a [f32],
    pub regime: &'a [f32],
    pub crust_type: &'a [f32],
    pub sea_level_m: f64,
    pub erosion_pulse_m_myr: f64,
    pub basin_m_myr: f64,
    pub min_km2: f64,
    pub transgression_frac: f64,
}

fn close(
    world: &mut WorldState,
    kind: &str,
    t: &Track,
    ongoing: bool,
    threshold: f64,
) -> Result<(), CoreError> {
    let mut e = world
        .event(t.t_start, kind, SOURCE)
        .until(t.t_last.max(t.t_start))
        .at(Place::Point {
            lat: t.lat as f32,
            lon: t.lon as f32,
        })
        .magnitude((t.peak_rate / (4.0 * threshold)).min(1.0) as f32)
        .param("peak_rate_m_myr", t.peak_rate)
        .param("area_km2", t.peak_area_km2)
        .param("volume_km3", t.volume_km3)
        .param("ongoing", i64::from(ongoing));
    if !t.setting.is_empty() {
        e = e.param("setting", t.setting.as_str());
    }
    if let Some(c) = t.cause {
        e = e.caused_by(c);
    }
    world.emit(e)?;
    Ok(())
}

fn track(
    world: &mut WorldState,
    tracks: &mut Vec<Track>,
    found: Vec<Region>,
    v: &FrameView<'_>,
    kind: &str,
    cause_kinds: &[&str],
    threshold: f64,
) -> Result<u32, CoreError> {
    let mut emitted = 0;
    for t in tracks.iter_mut() {
        t.seen = false;
    }
    for r in found {
        let best = tracks
            .iter_mut()
            .filter(|t| !t.seen)
            .map(|t| (distance_km(t.lat, t.lon, r.lat, r.lon, v.r_km), t))
            .filter(|(dk, _)| *dk <= MATCH_KM)
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let setting = if kind == "deeptime:basin_subsidence" {
            regime::NAMES
                .get(usize::from(r.setting))
                .copied()
                .unwrap_or("")
                .to_owned()
        } else {
            String::new()
        };
        match best {
            Some((_, t)) => {
                t.seen = true;
                t.t_last = v.t_end;
                t.volume_km3 += r.volume_km3;
                if r.peak > t.peak_rate {
                    t.peak_rate = r.peak;
                    t.lat = r.lat;
                    t.lon = r.lon;
                }
                t.peak_area_km2 = t.peak_area_km2.max(r.area_km2);
            }
            None => {
                let c = cause(
                    world,
                    cause_kinds,
                    r.lat,
                    r.lon,
                    v.t_start - 2.0e7,
                    v.t_end,
                    v.r_km,
                );
                tracks.push(Track {
                    lat: r.lat,
                    lon: r.lon,
                    t_start: v.t_start,
                    t_last: v.t_end,
                    peak_rate: r.peak,
                    peak_area_km2: r.area_km2,
                    volume_km3: r.volume_km3,
                    setting,
                    cause: c,
                    seen: true,
                });
            }
        }
    }
    let mut keep = Vec::with_capacity(tracks.len());
    for t in tracks.drain(..) {
        if t.seen {
            keep.push(t);
        } else {
            close(world, kind, &t, false, threshold)?;
            emitted += 1;
        }
    }
    *tracks = keep;
    Ok(emitted)
}

fn mask_centroid(v: &FrameView<'_>, gained: &dyn Fn(usize) -> bool) -> Option<(f64, f64, f64)> {
    let mut s = glam::DVec3::ZERO;
    let mut area = 0.0;
    for c in 0..v.d.cells() {
        if gained(c) {
            let j = c / v.d.w;
            let a = cell_area_km2(v.grid, j as u32, v.m_per_block);
            let lat = lat_of_row(v.grid, j as u32);
            let lon = 360.0 * (c % v.d.w) as f64 / v.d.w as f64;
            s += crate::sphere::unit_from_lat_lon_deg(lat, lon) * a;
            area += a;
        }
    }
    (area > 0.0).then(|| {
        let (lat, lon) = crate::sphere::lat_lon_deg(s);
        (lat, lon, area)
    })
}

fn emit_sea(
    world: &mut WorldState,
    v: &FrameView<'_>,
    s: &SeaTrack,
    ongoing: bool,
) -> Result<(), CoreError> {
    let rising = s.ext_f > s.pivot_f;
    let kind = if rising {
        "deeptime:marine_transgression"
    } else {
        "deeptime:marine_regression"
    };
    let bit = |m: &[u8], c: usize| m.get(c / 8).is_some_and(|b| b & (1 << (c % 8)) != 0);
    let (pm, em) = (&s.pivot_mask, &s.ext_mask);
    let place = mask_centroid(v, &|c| {
        let (a, b) = (bit(pm, c), bit(em, c));
        if rising { b && !a } else { a && !b }
    });
    let (lat, lon, area) = place.unwrap_or((0.0, 0.0, 0.0));
    let e = world
        .event(s.pivot_t, kind, SOURCE)
        .until(s.ext_t)
        .at(if place.is_some() {
            Place::Point {
                lat: lat as f32,
                lon: lon as f32,
            }
        } else {
            Place::Global
        })
        .magnitude(((s.ext_f - s.pivot_f).abs() / 0.3).min(1.0) as f32)
        .param("flooded_from", s.pivot_f)
        .param("flooded_to", s.ext_f)
        .param("sea_level_from_m", s.pivot_sea)
        .param("sea_level_to_m", s.ext_sea)
        .param("area_km2", area)
        .param("ongoing", i64::from(ongoing));
    world.emit(e)?;
    Ok(())
}

/// Decides the frame's events; `last` closes everything still open (`ongoing` = 1).
pub fn frame(
    world: &mut WorldState,
    st: &mut EventState,
    v: &FrameView<'_>,
    last: bool,
) -> Result<(), CoreError> {
    // Erosion pulses.
    let on_pulse =
        |c: usize| v.elevation[c] > 0.0 && f64::from(v.erosion[c]) >= v.erosion_pulse_m_myr;
    let found = regions(
        v.d,
        v.grid,
        v.m_per_block,
        &on_pulse,
        v.erosion,
        v.span_myr,
        v.regime,
        v.min_km2,
    );
    let mut pulses = std::mem::take(&mut st.pulses);
    st.counts[0] += track(
        world,
        &mut pulses,
        found,
        v,
        "deeptime:erosion_pulse",
        &[
            "deeptime:orogeny_phase",
            "deeptime:collision",
            "deeptime:subduction_onset",
        ],
        v.erosion_pulse_m_myr,
    )?;
    st.pulses = pulses;
    // Subsiding basins.
    let on_basin =
        |c: usize| v.elevation[c] > -2000.0 && f64::from(v.deposition[c]) >= v.basin_m_myr;
    let found = regions(
        v.d,
        v.grid,
        v.m_per_block,
        &on_basin,
        v.deposition,
        v.span_myr,
        v.regime,
        v.min_km2,
    );
    let mut basins = std::mem::take(&mut st.basins);
    st.counts[1] += track(
        world,
        &mut basins,
        found,
        v,
        "deeptime:basin_subsidence",
        &[
            "deeptime:rift_onset",
            "deeptime:collision",
            "deeptime:orogeny_phase",
            "deeptime:subduction_onset",
            "deeptime:ridge_birth",
        ],
        v.basin_m_myr,
    )?;
    st.basins = basins;
    // The sea over the continents.
    let n = v.d.cells();
    let mut mask = vec![0u8; n.div_ceil(8)];
    let (mut flooded, mut cont) = (0.0, 0.0);
    for c in 0..n {
        if v.crust_type[c] as u8 == crate::plates::state::CONTINENTAL {
            let a = cell_area_km2(v.grid, (c / v.d.w) as u32, v.m_per_block);
            cont += a;
            if v.elevation[c] < 0.0 {
                flooded += a;
                mask[c / 8] |= 1 << (c % 8);
            }
        }
    }
    let f = if cont > 0.0 { flooded / cont } else { 0.0 };
    let s = &mut st.sea;
    if !s.started {
        *s = SeaTrack {
            started: true,
            dir: 0,
            pivot_t: v.t_end,
            pivot_f: f,
            pivot_sea: v.sea_level_m,
            pivot_mask: mask.clone(),
            ext_t: v.t_end,
            ext_f: f,
            ext_sea: v.sea_level_m,
            ext_mask: mask,
        };
    } else {
        let new_ext = match s.dir {
            1 => f > s.ext_f,
            -1 => f < s.ext_f,
            _ => (f - s.pivot_f).abs() > (s.ext_f - s.pivot_f).abs(),
        };
        let reversed = match s.dir {
            1 => s.ext_f - f >= v.transgression_frac,
            -1 => f - s.ext_f >= v.transgression_frac,
            _ => false,
        };
        if reversed {
            if (s.ext_f - s.pivot_f).abs() >= v.transgression_frac {
                emit_sea(world, v, s, false)?;
                st.counts[2] += 1;
            }
            s.pivot_t = s.ext_t;
            s.pivot_f = s.ext_f;
            s.pivot_sea = s.ext_sea;
            s.pivot_mask = s.ext_mask.clone();
            s.dir = -s.dir;
            s.ext_t = v.t_end;
            s.ext_f = f;
            s.ext_sea = v.sea_level_m;
            s.ext_mask = mask;
        } else if new_ext {
            if s.dir == 0 && (f - s.pivot_f).abs() > 1e-9 {
                s.dir = if f > s.pivot_f { 1 } else { -1 };
            }
            s.ext_t = v.t_end;
            s.ext_f = f;
            s.ext_sea = v.sea_level_m;
            s.ext_mask = mask;
        }
    }
    if last {
        for t in std::mem::take(&mut st.pulses) {
            close(
                world,
                "deeptime:erosion_pulse",
                &t,
                true,
                v.erosion_pulse_m_myr,
            )?;
            st.counts[0] += 1;
        }
        for t in std::mem::take(&mut st.basins) {
            close(world, "deeptime:basin_subsidence", &t, true, v.basin_m_myr)?;
            st.counts[1] += 1;
        }
        let s = &st.sea;
        if s.started && (s.ext_f - s.pivot_f).abs() >= v.transgression_frac {
            emit_sea(world, v, s, true)?;
            st.counts[2] += 1;
        }
    }
    Ok(())
}

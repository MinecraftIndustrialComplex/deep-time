// SPDX-License-Identifier: LGPL-3.0-or-later
//! Mantle-fixed hotspots and large igneous provinces — the extension Cortial et al. 2019 suggest
//! (§7.5: hot spots "could be easily implemented … as specific sample points drifting across
//! plates"). Implemented the other way round, as the geology has it: a hotspot is a fixed point of
//! the mantle frame and the plates drift over it, so each leaves a volcanic track.
//!
//! - New hotspots arrive as a Poisson process (`hotspot_births_per_100myr`), live for a time drawn
//!   from `hotspot_lifetime_myr` and lift the crust within `hotspot_radius_km` by up to
//!   `hotspot_uplift_mm_yr` with the kernel (1 − (d/r)²)², tapering to nothing at a volcanic
//!   ceiling (3 km on oceanic crust, 1.5 km swells on continents), adding basalt to the pools.
//! - With probability `lip_probability` a new hotspot arrives with a plume head: a large igneous
//!   province of radius `lip_radius_km` piles up to `lip_thickness_m` of basalt (its surface rises
//!   by a third of that) and boosts rifting of the plate above (`rifting.rs`), which it then causes.
//! - Events: large_igneous_province at the eruption; hotspot_track when the hotspot dies or the run
//!   ends, spanning its life, with the plates it crossed and its track length.

use glam::DVec3;

use super::Env;
use super::events::{self, HOTSPOT_TRACK, LARGE_IGNEOUS_PROVINCE};
use super::geometry::Tree;
use super::state::{CONTINENTAL, HOTSPOT, Hotspot, PlateState, pool};
use crate::error::CoreError;
use crate::world::WorldState;

/// Volcanic ceilings: hotspot uplift fades to zero at these heights.
const OCEANIC_CEILING_M: f64 = 3000.0;
const CONTINENTAL_CEILING_M: f64 = 1500.0;

/// World global: basalt the hotspots and their plume heads added to the pools over the run, km³
/// (Earth-normalized; the volcano stage's cones over them are counted inside, plate task 8).
pub const MAGMA_GLOBAL: &str = "plates/hotspot_magma_km3";
/// World globals (measurement, the crust budget's additions at hotspots against Stern & Scholl
/// 2010's Table 1; ledger: PLT-761): of [`MAGMA_GLOBAL`], the basalt added to continental crust,
/// km³; the volcano stage's cones a hotspot counted inside its rate (its shields and silicic
/// centres, PLT-286), km³, and of them those on continental crust.
pub const MAGMA_CONTINENTAL_GLOBAL: &str = "plates/hotspot_magma_continental_km3";
pub const CONES_GLOBAL: &str = "plates/hotspot_cones_km3";
pub const CONES_CONTINENTAL_GLOBAL: &str = "plates/hotspot_cones_continental_km3";

fn kernel(d: f64, r: f64) -> f64 {
    if d >= r {
        0.0
    } else {
        let x = d / r;
        (1.0 - x * x).powi(2)
    }
}

/// One step of every hotspot: births (with LIPs), volcanic uplift, tracks, deaths.
pub fn step(
    state: &mut PlateState,
    world: &mut WorldState,
    pos: &[DVec3],
    tree: &Tree,
    env: &Env,
    cones: &mut [[f64; 2]],
) -> Result<(), CoreError> {
    let k = env.k;
    let norm = env.norm;
    let t = env.t_myr;
    let mut magma_m = 0.0;
    // Measurement (ledger: PLT-761): the basalt on continental crust, the cones counted inside.
    let (mut magma_continental_m, mut cones_m, mut cones_continental_m) = (0.0, 0.0, 0.0);
    // Births.
    let mut expected = k.hotspot_births_per_100myr * env.dt_myr / 100.0;
    while expected > 0.0 {
        if state.rng.next_f64() < expected.min(1.0) {
            let id = state.hotspots.len() as u32;
            let at = state.rng.unit_vector();
            let life = state
                .rng
                .range(k.hotspot_lifetime_myr[0], k.hotspot_lifetime_myr[1]);
            let uplift = k.hotspot_uplift_mm_yr * state.rng.range(0.6, 1.4);
            let lip = state.rng.next_f64() < k.lip_probability;
            let mut h = Hotspot {
                id,
                pos: at.to_array(),
                born_myr: t,
                dies_myr: t + life,
                uplift_mm_yr: uplift,
                lip_event: None,
                plates: Vec::new(),
                track_km: 0.0,
                closed: false,
            };
            if lip {
                h.lip_event = Some(erupt(state, world, pos, tree, env, at)?);
            }
            state.hotspots.push(h);
            state.counters.hotspots += 1;
        }
        expected -= 1.0;
    }

    // Uplift, tracks and deaths.
    let dz_rate = 1.0e-3 * env.dt_yr;
    for h in 0..state.hotspots.len() {
        let hs = &state.hotspots[h];
        if hs.closed || hs.born_myr > t {
            continue;
        }
        let at = DVec3::from_array(hs.pos);
        if t >= hs.dies_myr {
            close(state, world, env, h, false)?;
            continue;
        }
        let r = norm.rad(k.hotspot_radius_km);
        let uplift = hs.uplift_mm_yr;
        let mut lifts: Vec<(usize, f64, f64)> = Vec::new();
        for (i, _) in tree.within(at, r) {
            let s = &state.samples[i as usize];
            if s.is_hidden() {
                continue;
            }
            let w = kernel(pos[i as usize].angle_between(at), r);
            let ceiling = if s.kind == CONTINENTAL {
                CONTINENTAL_CEILING_M
            } else {
                OCEANIC_CEILING_M
            };
            let z = f64::from(s.elevation_m);
            let room = ((ceiling - z) / ceiling).clamp(0.0, 1.0);
            lifts.push((i as usize, w, uplift * dz_rate * w * room));
        }
        // The cones the volcano stage built over the hotspot (its shields and silicic centres,
        // already in the pools as volcanic rock) are its erupted rock: the basalt it adds
        // itself is what they leave of its rate (plate task 8, counted once). ledger: PLT-286
        let rock: f64 = lifts.iter().map(|&(_, _, dz)| 3.0 * dz).sum();
        let mut built = 0.0;
        for &(i, _, _) in &lifts {
            if let Some(c) = cones.get_mut(i) {
                built += c[0] + c[1];
                if state.samples[i].kind == CONTINENTAL {
                    cones_continental_m += c[0] + c[1];
                }
                *c = [0.0; 2];
            }
        }
        cones_m += built;
        let q = if rock > 0.0 {
            (rock - built).max(0.0) / rock
        } else {
            1.0
        };
        for (i, w, dz) in lifts {
            let s = &mut state.samples[i];
            s.elevation_m = (f64::from(s.elevation_m) + dz) as f32;
            s.pools[pool::MAFIC_VOLCANIC] += (dz * 3.0 * q) as f32;
            s.thickness_m = s.pools.iter().sum();
            magma_m += dz * 3.0 * q;
            if s.kind == CONTINENTAL {
                magma_continental_m += dz * 3.0 * q;
            }
            if w > 0.2 {
                s.flags |= HOTSPOT;
            }
        }
        if let Some((j, _)) = tree.nearest(at) {
            let plate = state.plate_of(j as usize);
            let hs = &mut state.hotspots[h];
            if hs.plates.last() != Some(&plate) {
                hs.plates.push(plate);
            }
            let omega = DVec3::from_array(state.plates[plate as usize].omega);
            state.hotspots[h].track_km += norm.speed_mm_yr(omega, at) * env.dt_myr;
        }
    }
    *world.globals.entry(MAGMA_GLOBAL.to_owned()).or_insert(0.0) +=
        magma_m * norm.sample_area_km2() / 1000.0;
    for (name, m) in [
        (MAGMA_CONTINENTAL_GLOBAL, magma_continental_m),
        (CONES_GLOBAL, cones_m),
        (CONES_CONTINENTAL_GLOBAL, cones_continental_m),
    ] {
        *world.globals.entry(name.to_owned()).or_insert(0.0) += m * norm.sample_area_km2() / 1000.0;
    }
    Ok(())
}

/// A plume head's large igneous province at `at`; returns its event.
fn erupt(
    state: &mut PlateState,
    world: &mut WorldState,
    pos: &[DVec3],
    tree: &Tree,
    env: &Env,
    at: DVec3,
) -> Result<deeptime_format::EventId, CoreError> {
    let k = env.k;
    let norm = env.norm;
    let r = norm.rad(k.lip_radius_km);
    let mut volume_m3_model = 0.0;
    let mut continental_m3_model = 0.0;
    let mut count = 0usize;
    for (i, _) in tree.within(at, r) {
        let s = &mut state.samples[i as usize];
        if s.is_hidden() {
            continue;
        }
        let w = kernel(pos[i as usize].angle_between(at), r);
        let pile = k.lip_thickness_m * w;
        s.pools[pool::MAFIC_VOLCANIC] += pile as f32;
        s.thickness_m = s.pools.iter().sum();
        s.elevation_m = (f64::from(s.elevation_m) + pile / 3.0).min(k.max_elevation_m) as f32;
        volume_m3_model += pile * norm.sample_area_km2() * 1.0e6;
        if s.kind == CONTINENTAL {
            continental_m3_model += pile * norm.sample_area_km2() * 1.0e6;
        }
        count += 1;
    }
    let plate = tree.nearest(at).map(|(j, _)| state.plate_of(j as usize));
    let area_km2 = norm.planet_area_km2(count as f64 * norm.sample_area_km2());
    let f = norm.planet_km / norm.model_km;
    let volume_km3 = volume_m3_model * f * f / 1.0e9;
    *world.globals.entry(MAGMA_GLOBAL.to_owned()).or_insert(0.0) += volume_m3_model / 1.0e9;
    // ledger: PLT-761 (measurement)
    *world
        .globals
        .entry(MAGMA_CONTINENTAL_GLOBAL.to_owned())
        .or_insert(0.0) += continental_m3_model / 1.0e9;
    let mut e = events::event(world, env.t_myr, LARGE_IGNEOUS_PROVINCE, at)
        .magnitude((k.lip_thickness_m / 1000.0) as f32)
        .param("area_km2", area_km2)
        .param("volume_km3", volume_km3)
        .param("thickness_m", k.lip_thickness_m);
    if let Some(p) = plate {
        e = e.param("plate", i64::from(p));
    }
    let id = world.emit(e)?;
    if let Some(p) = plate {
        let plate = &mut state.plates[p as usize];
        plate.lip_until_myr = env.t_myr + k.lip_rift_boost_myr;
        plate.lip_event = Some(id);
    }
    state.counters.lips += 1;
    Ok(id)
}

/// Writes hotspot `h`'s track event and marks it closed (`ongoing` when the run ended first).
pub fn close(
    state: &mut PlateState,
    world: &mut WorldState,
    env: &Env,
    h: usize,
    ongoing: bool,
) -> Result<(), CoreError> {
    let hs = state.hotspots[h].clone();
    let start = hs.born_myr.max(state.start_myr);
    let end = if ongoing {
        env.t_myr
    } else {
        hs.dies_myr.min(env.t_myr)
    };
    let plates: Vec<String> = hs.plates.iter().map(u32::to_string).collect();
    let mut e = events::event(world, start, HOTSPOT_TRACK, DVec3::from_array(hs.pos))
        .until(end.max(start) * 1.0e6)
        .magnitude((hs.uplift_mm_yr / env.k.hotspot_uplift_mm_yr) as f32)
        .param("hotspot", i64::from(hs.id))
        .param("plates", plates.join(","))
        .param("track_km", env.norm.planet_len_km(hs.track_km))
        .param("uplift_mm_yr", hs.uplift_mm_yr)
        .param("ongoing", i64::from(ongoing));
    if let Some(lip) = hs.lip_event {
        e = e.caused_by(lip);
    }
    world.emit(e)?;
    state.hotspots[h].closed = true;
    Ok(())
}

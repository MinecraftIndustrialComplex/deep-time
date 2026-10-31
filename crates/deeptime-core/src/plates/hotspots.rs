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
use super::pools::isostatic_change_m;
use super::settle::WATER_LOADED;
use super::state::{CONTINENTAL, HOTSPOT, Hotspot, PlateState, Sample, pool};
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
/// World globals (measurement, the hotspot lane; ledger: PLT-795): the plume heads' large igneous
/// provinces whole (the plate model's dome and the volcano stage's plateau beyond it, VOL-035),
/// km³ Earth-normalized; of it on continental crust; the dome's part there (in
/// [`MAGMA_CONTINENTAL_GLOBAL`] too); and the provinces erupted.
pub const LIP_GLOBAL: &str = "plates/lip_km3";
pub const LIP_CONTINENTAL_GLOBAL: &str = "plates/lip_continental_km3";
pub const LIP_DOME_CONTINENTAL_GLOBAL: &str = "plates/lip_dome_continental_km3";
pub const LIP_COUNT_GLOBAL: &str = "plates/lips";

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
    let (births_per_myr, mut expected) = if k.hotspots_sourced {
        let b = sourced::births_per_myr(); // ledger: PLT-790
        (b, b * env.dt_myr)
    } else {
        (
            k.hotspot_births_per_100myr / 100.0,
            k.hotspot_births_per_100myr * env.dt_myr / 100.0,
        )
    };
    while expected > 0.0 {
        if state.rng.next_f64() < expected.min(1.0) {
            let id = state.hotspots.len() as u32;
            let at = state.rng.unit_vector();
            let (life, uplift, flux) = if k.hotspots_sourced {
                let life = sourced::lifetime_myr(&mut state.rng);
                let flux = sourced::flux_mg_s(&mut state.rng);
                (life, sourced::volcano_uplift_mm_yr(k, flux), flux)
            } else {
                let life = state
                    .rng
                    .range(k.hotspot_lifetime_myr[0], k.hotspot_lifetime_myr[1]);
                (life, k.hotspot_uplift_mm_yr * state.rng.range(0.6, 1.4), 0.0)
            };
            let head_share = if k.plume_heads_sourced {
                sourced::head_share(births_per_myr) // ledger: PLT-795
            } else {
                k.lip_probability
            };
            let lip = state.rng.next_f64() < head_share;
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
                flux_mg_s: flux,
                magma_km3_yr: flux * sourced::MAGMA_KM3_YR_PER_MG_S,
                head_radius_km: 0.0,
            };
            if lip {
                let (event, radius_km) = erupt(state, world, pos, tree, env, at)?;
                h.lip_event = Some(event);
                h.head_radius_km = radius_km;
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
        let tail_km3_yr = hs.magma_km3_yr;
        let mut lifts: Vec<(usize, f64, f64)> = Vec::new();
        for (i, _) in tree.within(at, r) {
            let s = &state.samples[i as usize];
            if s.is_hidden() {
                continue;
            }
            let w = kernel(pos[i as usize].angle_between(at), r);
            if tail_km3_yr > 0.0 {
                // A sourced tail (PLT-793): its magma is spread by the kernel below.
                lifts.push((i as usize, w, 0.0));
                continue;
            }
            let ceiling = if s.kind == CONTINENTAL {
                CONTINENTAL_CEILING_M
            } else {
                OCEANIC_CEILING_M
            };
            let z = f64::from(s.elevation_m);
            let room = ((ceiling - z) / ceiling).clamp(0.0, 1.0);
            lifts.push((i as usize, w, uplift * dz_rate * w * room));
        }
        if tail_km3_yr > 0.0 {
            // The tail's magma over the step, km³ (Earth-normalized), as metres of rock on the
            // samples in the kernel's proportion (ledger: PLT-793).
            let sum_w: f64 = lifts.iter().map(|&(_, w, _)| w).sum();
            let rock_m = if sum_w > 0.0 {
                tail_km3_yr * env.dt_yr * 1000.0 / norm.sample_area_km2()
            } else {
                0.0
            };
            for l in &mut lifts {
                l.2 = rock_m * l.1 / sum_w.max(1e-300);
            }
        }
        // The cones the volcano stage built over the hotspot (its shields and silicic centres,
        // already in the pools as volcanic rock) are its erupted rock: the basalt it adds
        // itself is what they leave of its rate (plate task 8, counted once). ledger: PLT-286
        let rock: f64 = if tail_km3_yr > 0.0 {
            lifts.iter().map(|&(_, _, m)| m).sum()
        } else {
            lifts.iter().map(|&(_, _, dz)| 3.0 * dz).sum()
        };
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
            let added = if tail_km3_yr > 0.0 {
                // Lavas over intrusions in the tail's share, the height by the model's isostasy
                // (ledger: PLT-794).
                let m = dz * q;
                add_magma(s, m, sourced::TAIL_EXTRUSIVE_SHARE, env);
                m
            } else {
                s.elevation_m = (f64::from(s.elevation_m) + dz) as f32;
                s.pools[pool::MAFIC_VOLCANIC] += (dz * 3.0 * q) as f32;
                s.thickness_m = s.pools.iter().sum();
                dz * 3.0 * q
            };
            magma_m += added;
            if s.kind == CONTINENTAL {
                magma_continental_m += added;
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

/// A plume head's large igneous province at `at`; returns its event and the radius its head
/// spread to, km on the model sphere.
fn erupt(
    state: &mut PlateState,
    world: &mut WorldState,
    pos: &[DVec3],
    tree: &Tree,
    env: &Env,
    at: DVec3,
) -> Result<(deeptime_format::EventId, f64), CoreError> {
    let k = env.k;
    let norm = env.norm;
    // With `plume_heads_sourced` the province's volume is one of the LIP record's and the
    // head's radius the flattened head's (ledger: PLT-796, PLT-797). The plate model's dome
    // holds the share of it that the dome is of the volcano stage's plateau (VOL-035), which
    // lays the rest on the same radius (`surface::volcanoes`), so the world's province holds
    // the record's volume. Off, the preset's radius and central pile.
    let sourced = k.plume_heads_sourced;
    let (radius_km, record_km3) = if sourced {
        (
            sourced::HEAD_RADIUS_KM,
            sourced::lip_volume_km3(&mut state.rng),
        )
    } else {
        (k.lip_radius_km, 0.0)
    };
    let r = norm.rad(radius_km);
    let area = norm.sample_area_km2();
    let within: Vec<(usize, f64, f64)> = tree
        .within(at, r)
        .into_iter()
        .filter(|&(i, _)| !state.samples[i as usize].is_hidden())
        .map(|(i, _)| {
            let d = pos[i as usize].angle_between(at);
            (i as usize, kernel(d, r), sourced::plateau(d / r))
        })
        .collect();
    let thickness_m = if sourced {
        let plateau_km2: f64 = within.iter().map(|&(_, _, p)| p).sum::<f64>() * area;
        if plateau_km2 > 0.0 {
            record_km3 * 1000.0 / plateau_km2
        } else {
            0.0
        }
    } else {
        k.lip_thickness_m
    };
    let mut volume_m3_model = 0.0;
    let mut continental_m3_model = 0.0;
    let (mut whole_m3, mut whole_c_m3) = (0.0, 0.0);
    for &(i, w, plateau) in &within {
        let s = &mut state.samples[i];
        let pile = thickness_m * w;
        if sourced {
            add_magma(s, pile, sourced::LIP_EXTRUSIVE_SHARE, env);
        } else {
            s.pools[pool::MAFIC_VOLCANIC] += pile as f32;
            s.thickness_m = s.pools.iter().sum();
            s.elevation_m = (f64::from(s.elevation_m) + pile / 3.0).min(k.max_elevation_m) as f32;
        }
        volume_m3_model += pile * area * 1.0e6;
        let whole = thickness_m * plateau * area * 1.0e6;
        whole_m3 += whole;
        if s.kind == CONTINENTAL {
            continental_m3_model += pile * area * 1.0e6;
            whole_c_m3 += whole;
        }
    }
    let count = within.len();
    let plate = tree.nearest(at).map(|(j, _)| state.plate_of(j as usize));
    let area_km2 = norm.planet_area_km2(count as f64 * area);
    let f = norm.planet_km / norm.model_km;
    let volume_km3 = if sourced { whole_m3 } else { volume_m3_model } * f * f / 1.0e9;
    *world.globals.entry(MAGMA_GLOBAL.to_owned()).or_insert(0.0) += volume_m3_model / 1.0e9;
    // ledger: PLT-761 (measurement)
    *world
        .globals
        .entry(MAGMA_CONTINENTAL_GLOBAL.to_owned())
        .or_insert(0.0) += continental_m3_model / 1.0e9;
    if sourced {
        // ledger: PLT-795 (measurement)
        for (name, v) in [
            (LIP_GLOBAL, whole_m3 / 1.0e9),
            (LIP_CONTINENTAL_GLOBAL, whole_c_m3 / 1.0e9),
            (LIP_DOME_CONTINENTAL_GLOBAL, continental_m3_model / 1.0e9),
            (LIP_COUNT_GLOBAL, 1.0),
        ] {
            *world.globals.entry(name.to_owned()).or_insert(0.0) += v;
        }
    }
    let mut e = events::event(world, env.t_myr, LARGE_IGNEOUS_PROVINCE, at)
        .magnitude((thickness_m / 1000.0) as f32)
        .param("area_km2", area_km2)
        .param("volume_km3", volume_km3)
        .param("thickness_m", thickness_m);
    if sourced {
        // The head's radius, for the volcano stage's plateau (ledger: PLT-797).
        e = e.param("radius_km", radius_km);
    }
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
    Ok((id, radius_km))
}

/// Adds `m` metres of mantle magma to `s` (the hotspot lane, PLT-794): lavas, `extrusive` of it,
/// over intrusions, the height by the model's Airy isostasy of the column's change
/// (`pools::isostatic_change_m`), water-loaded under the sea (`settle::WATER_LOADED`).
pub fn add_magma(s: &mut Sample, m: f64, extrusive: f64, env: &Env) {
    if !(m > 0.0) {
        return;
    }
    let before = s.pools;
    s.pools[pool::MAFIC_VOLCANIC] += (m * extrusive) as f32;
    s.pools[pool::MAFIC_PLUTONIC] += (m * (1.0 - extrusive)) as f32;
    s.thickness_m = s.pools.iter().sum();
    let dry = isostatic_change_m(&before, &s.pools, env.gravity);
    let wet = if s.elevation_m < 0.0 {
        WATER_LOADED
    } else {
        1.0
    };
    s.elevation_m = (f64::from(s.elevation_m) + dry * wet).min(env.k.max_elevation_m) as f32;
}

/// The plume heads' hot mantle, for the volcanic margins (PLT-798): every hotspot that rose with
/// a head, where and when, how far its head spread (its province's radius) and how hot it is.
pub fn hot_mantle(state: &PlateState, env: &Env) -> Vec<super::ridges::HotMantle> {
    state
        .hotspots
        .iter()
        .filter(|h| h.lip_event.is_some())
        .map(|h| super::ridges::HotMantle {
            at: DVec3::from_array(h.pos),
            from_myr: h.born_myr,
            radius_rad: env.norm.rad(if h.head_radius_km > 0.0 {
                h.head_radius_km
            } else {
                env.k.lip_radius_km
            }),
            excess_k: sourced::HEAD_EXCESS_K,
        })
        .collect()
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

/// The hotspot lane's sourced plumes, heads and margins (`tectonics.hotspots_sourced`,
/// `plume_heads_sourced`, `volcanic_margins`; docs/design/hotspots-lips-margins.md; the sources
/// and what was read of them in bibliography.md "The sources of the hotspot lane").
pub mod sourced {
    use crate::params::TectonicParams;
    use crate::sphere::SplitMix64;

    /// PLACEHOLDER (filled from the sources).
    pub const FLUX_MG_S: [f64; 1] = [f64::NAN];
    pub const COUNT: usize = FLUX_MG_S.len();
    pub const LIFETIME_MYR: [f64; 2] = [f64::NAN, f64::NAN];
    pub fn births_per_myr() -> f64 {
        COUNT as f64 / (0.5 * (LIFETIME_MYR[0] + LIFETIME_MYR[1]))
    }
    pub fn lifetime_myr(rng: &mut SplitMix64) -> f64 {
        rng.range(LIFETIME_MYR[0], LIFETIME_MYR[1])
    }
    pub fn flux_mg_s(rng: &mut SplitMix64) -> f64 {
        FLUX_MG_S[(rng.next_u64() % COUNT as u64) as usize]
    }
    pub const MAGMA_KM3_YR_PER_MG_S: f64 = f64::NAN;
    pub const TAIL_EXTRUSIVE_SHARE: f64 = f64::NAN;
    pub fn volcano_uplift_mm_yr(k: &TectonicParams, flux: f64) -> f64 {
        let below = FLUX_MG_S.iter().filter(|&&f| f < flux).count() as f64;
        let q = if COUNT > 1 {
            below / (COUNT - 1) as f64
        } else {
            0.5
        };
        k.hotspot_uplift_mm_yr * (0.6 + 0.8 * q)
    }
    pub const LIPS_PER_MYR: f64 = f64::NAN;
    pub fn head_share(births_per_myr: f64) -> f64 {
        if births_per_myr > 0.0 {
            (LIPS_PER_MYR / births_per_myr).min(1.0)
        } else {
            0.0
        }
    }
    pub const LIP_VOLUMES_KM3: [f64; 1] = [f64::NAN];
    pub fn lip_volume_km3(rng: &mut SplitMix64) -> f64 {
        LIP_VOLUMES_KM3[(rng.next_u64() % LIP_VOLUMES_KM3.len() as u64) as usize]
    }
    pub const LIP_EXTRUSIVE_SHARE: f64 = f64::NAN;
    pub const HEAD_RADIUS_KM: f64 = f64::NAN;
    /// The volcano stage's flood-basalt plateau at `x` of the radius (VOL-035).
    pub fn plateau(x: f64) -> f64 {
        let t = ((x - 0.6) / 0.4).clamp(0.0, 1.0);
        1.0 - t * t * (3.0 - 2.0 * t)
    }
    pub const HEAD_EXCESS_K: f64 = f64::NAN;
    pub const HEAD_HOT_MYR: f64 = f64::NAN;
    pub fn igneous_crust_m(excess_k: f64) -> f64 {
        7000.0 + excess_k * f64::NAN
    }
    pub const MARGIN_EXTRUSIVE_SHARE: f64 = f64::NAN;
    pub const MARGIN_WIDTH_KM: f64 = f64::NAN;
}

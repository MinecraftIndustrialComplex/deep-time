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
//!
//! From sources (the hotspot lane, 2026-10-31; [`sourced`], docs/design/hotspots-lips-margins.md;
//! each off by default):
//! - `hotspots_sourced`: Sleep's (1990) 37 tails, each with one of his buoyancy fluxes, living 100
//!   to 150 Myr (Courtillot et al. 2003), born at the steady state's rate, each making Hawaii's
//!   magma per unit of flux (Sleep: 4 m³/s at 8.7 Mg/s), a seventh of it lavas (White et al.
//!   2006), the heights by the model's isostasy instead of the uplift rule and its ceilings.
//! - `plume_heads_sourced`: a head with the share of births that gives one every 15 Myr (Ernst et
//!   al. 2021), spreading 1,000 km (Sleep 1990, White & McKenzie 1989, Campbell & Griffiths 1990),
//!   its province one of the record's for the crust it rises under (Clift et al. 2009), a quarter
//!   lavas.
//! - `volcanic_margins` (`ridges::resample`): a head's mantle is 150 K hot for 20 Myr after it
//!   rises ([`hot_mantle`]), and a continent's new sea floor that forms over it is a volcanic
//!   margin.

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
/// km³; without the cone budget, the volcano stage's cones a hotspot counted inside its rate by
/// place, on its kernel (any cone there the arcs had not counted: its shields and silicic
/// centres, PLT-286, and whatever else stood there), km³, and of them those on continental crust.
/// With `tectonics.hotspot_cone_budget` (PLT-803) the tails net their own cones by hotspot
/// instead ([`CONES_BY_ID_GLOBAL`]), the kernel counts none and these stay 0.
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
/// World globals (measurement, the cone budget; ledger: PLT-804), with
/// `tectonics.hotspot_cone_budget`: the cones the tails netted by their own hotspot, km³
/// (Earth-normalized), and of them on continental cells; the tails' rock before netting.
pub const CONES_BY_ID_GLOBAL: &str = "plates/hotspot_cones_by_id_km3";
pub const CONES_BY_ID_CONTINENTAL_GLOBAL: &str = "plates/hotspot_cones_by_id_continental_km3";
pub const TAIL_ROCK_GLOBAL: &str = "plates/hotspot_tail_rock_km3";
/// … and what the cones of dead tails handed (or their calderas took back) that no step netted,
/// km³ (Earth-normalized; either sign): a tail nets nothing once it has died.
pub const CONES_UNNETTED_GLOBAL: &str = "plates/hotspot_cones_unnetted_km3";

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
    // The cone budget (ledger: PLT-803, measurement PLT-804), km³.
    let budget = k.hotspot_cone_budget;
    let km3_per_m = norm.sample_area_km2() / 1000.0;
    let (mut by_id_km3, mut by_id_continental_km3, mut tail_rock_km3) = (0.0, 0.0, 0.0);
    let mut unnetted_km3 = 0.0;
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
                (
                    life,
                    k.hotspot_uplift_mm_yr * state.rng.range(0.6, 1.4),
                    0.0,
                )
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
                cones: Default::default(),
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
        if hs.born_myr > t {
            continue;
        }
        if hs.closed || t >= hs.dies_myr {
            if !hs.closed {
                close(state, world, env, h, false)?;
            }
            // The cone budget (ledger: PLT-803): a dead tail nets nothing, so what its cones
            // handed since its last step (and what their calderas take back later) leaves its
            // record as unnetted (measurement, PLT-804).
            if budget {
                let c = &mut state.hotspots[h].cones;
                unnetted_km3 += c.handed_km3;
                c.handed_km3 = 0.0;
                c.handed_continental_km3 = 0.0;
            }
            continue;
        }
        let hs = &state.hotspots[h];
        let at = DVec3::from_array(hs.pos);
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
                // With `hotspots_sourced`, the lid: the melt per unit of flux under each column's
                // lithosphere as a share of Hawaii's (ledger: PLT-805).
                let lid = if k.hotspots_sourced {
                    sourced::lid_factor(f64::from(state.samples[l.0].lithosphere_km))
                } else {
                    1.0
                };
                l.2 = rock_m * l.1 * lid / sum_w.max(1e-300);
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
        // Under the cone budget (ledger: PLT-803) the tail nets its own cones by hotspot below,
        // and the volcano stage hands them apart (none of them is in `cones`), so the kernel
        // counts nothing by place.
        let mut built = 0.0;
        if !budget {
            for &(i, _, _) in &lifts {
                if let Some(c) = cones.get_mut(i) {
                    built += c[0] + c[1];
                    if state.samples[i].kind == CONTINENTAL {
                        cones_continental_m += c[0] + c[1];
                    }
                    *c = [0.0; 2];
                }
            }
        }
        cones_m += built;
        // The lavas' share of the tail's rock (PLT-794; the preset's rock is all lava, PLT-115).
        let lavas_share = if tail_km3_yr > 0.0 {
            sourced::TAIL_EXTRUSIVE_SHARE
        } else {
            1.0
        };
        // The cone budget (ledger: PLT-803): the cones the hotspot's own volcanoes handed the pools
        // since the last step, wherever they stand, are its lavas, netted now up to the step's
        // rock. What is left carries to the next step, of either sign: past the step's rock, a
        // step with no rock (every kernel column at its ceiling or none visible), or below zero
        // (a caldera took back more than the cones handed since), so the tail never adds less
        // than nothing nor more than its rate.
        let netted_km3 = if budget {
            state.hotspots[h]
                .cones
                .handed_km3
                .clamp(0.0, (rock * km3_per_m).max(0.0))
        } else {
            0.0
        };
        let (q, extrusive) = if budget {
            // The tail adds the rest of its rock, its intrusions whole (a shortfall from the
            // step's lag comes off them).
            let handed = netted_km3 / km3_per_m;
            let left = (rock - handed).max(0.0);
            let lavas = (lavas_share * rock - handed).max(0.0);
            (
                if rock > 0.0 { left / rock } else { 1.0 },
                if left > 0.0 {
                    lavas / left
                } else {
                    lavas_share
                },
            )
        } else {
            let q = if rock > 0.0 {
                (rock - built).max(0.0) / rock
            } else {
                1.0
            };
            (q, sourced::TAIL_EXTRUSIVE_SHARE)
        };
        if budget {
            // The budget of the next frame: the step's lavas, none for a tail that dies before
            // the next step (nothing would net them).
            let lives_on = t + env.dt_myr < state.hotspots[h].dies_myr;
            let c = &mut state.hotspots[h].cones;
            let netted_c = if c.handed_km3 > 0.0 {
                (c.handed_continental_km3 / c.handed_km3).clamp(0.0, 1.0) * netted_km3
            } else {
                0.0
            };
            by_id_km3 += netted_km3;
            by_id_continental_km3 += netted_c;
            c.handed_km3 -= netted_km3;
            c.handed_continental_km3 -= netted_c;
            c.budget_km3 = if lives_on {
                lavas_share * rock * km3_per_m
            } else {
                0.0
            };
            tail_rock_km3 += rock * km3_per_m;
        }
        for (i, w, dz) in lifts {
            let s = &mut state.samples[i];
            let added = if tail_km3_yr > 0.0 {
                // Lavas over intrusions in the tail's share, the height by the model's isostasy
                // (ledger: PLT-794).
                let m = dz * q;
                add_magma(s, m, extrusive, env);
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
    if budget {
        // ledger: PLT-804 (measurement)
        for (name, v) in [
            (CONES_BY_ID_GLOBAL, by_id_km3),
            (CONES_BY_ID_CONTINENTAL_GLOBAL, by_id_continental_km3),
            (TAIL_ROCK_GLOBAL, tail_rock_km3),
            (CONES_UNNETTED_GLOBAL, unnetted_km3),
        ] {
            *world.globals.entry(name.to_owned()).or_insert(0.0) += v;
        }
    }
    Ok(())
}

/// A plume head's large igneous province at `at`; returns its event and the radius its head
/// spread to, km on the model sphere.
pub(super) fn erupt(
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
        let continental = tree
            .nearest(at)
            .is_some_and(|(j, _)| state.samples[j as usize].kind == CONTINENTAL);
        (
            sourced::HEAD_RADIUS_KM,
            sourced::lip_volume_km3(&mut state.rng, continental),
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
    if m.is_nan() || m <= 0.0 {
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

    /// Sleep (1990, Table 1, p. 6732; full text): the buoyancy fluxes of 37 hotspots, Mg/s, in
    /// his order (Afar to Yellowstone), "The total buoyancy flux is 54.9 Mg s-1" (p. 6730). A
    /// tail's flux is one of them, drawn evenly. ledger: PLT-791
    pub const FLUX_MG_S: [f64; 37] = [
        1.2, 0.9, 1.1, 0.3, 1.1, 0.4, 0.3, 1.0, 1.6, 1.6, 0.5, 0.5, 3.3, 0.5, 1.0, 0.5, 8.7, 0.9,
        1.4, 0.3, 1.6, 0.5, 0.9, 3.3, 3.3, 0.5, 0.5, 3.3, 1.9, 0.5, 1.6, 1.6, 3.3, 0.9, 0.9, 1.7,
        1.5,
    ];
    /// The tails a planet holds: Sleep's 37, the count of his catalogue (King & Adam 2014 list
    /// 26 to 69 in the catalogues, Courtillot et al. 2003 consider 49). ledger: PLT-790
    pub const COUNT: usize = FLUX_MG_S.len();
    /// A tail's life, Myr, drawn evenly: Courtillot et al. (2003, p. 303; full text), "The tails
    /// (stems) of primary plumes tend to last on the order of 130 Ma. Indeed, all plumes born as
    /// traps in the last 100 Ma ... are still quite active, whereas those born between 100 and
    /// 140 Ma may be failing ... and those older than 150 Ma do not in general have an active
    /// trace" (the range is the agent's reading; the only lifetime read, applied to every tail).
    /// ledger: PLT-792
    pub const LIFETIME_MYR: [f64; 2] = [100.0, 150.0];
    /// Births per Myr: the steady state of [`COUNT`] tails living [`LIFETIME_MYR`]'s mean
    /// (Little's law, the agent's arithmetic: 37 / 125). ledger: PLT-790
    pub fn births_per_myr() -> f64 {
        COUNT as f64 / (0.5 * (LIFETIME_MYR[0] + LIFETIME_MYR[1]))
    }
    pub fn lifetime_myr(rng: &mut SplitMix64) -> f64 {
        rng.range(LIFETIME_MYR[0], LIFETIME_MYR[1])
    }
    pub fn flux_mg_s(rng: &mut SplitMix64) -> f64 {
        FLUX_MG_S[(rng.next_u64() % COUNT as u64) as usize]
    }
    /// A tail's magma per unit of buoyancy flux, km³ a year per Mg/s: Hawaii's, Sleep (1990,
    /// p. 6719): "the flux Qv of magmas implaced in the crust or erupted is about 4 m3 s-1", at
    /// its 8.7 Mg/s (Table 1); 4 / 8.7 m³/s per Mg/s, × 3.156 × 10⁷ s a year (the agent's
    /// arithmetic). Hawaii's melt share of its plume applied to every tail (Sleep: Iceland's,
    /// on a ridge, is ten times it; the thick lid's lower melting is not modelled).
    /// ledger: PLT-793
    pub const MAGMA_KM3_YR_PER_MG_S: f64 = 4.0 / 8.7 * 3.155_76e7 / 1.0e9;
    /// Of a tail's magma, the share erupted: White, Crisp & Spera (2006, Table 3; full text) give
    /// the Hawaiian-Emperor chain an intrusive to extrusive ratio of 6:1 (5.9 × 10⁶ against
    /// 1.1 × 10⁶ km³ over 74 Myr, underplating included). ledger: PLT-794
    pub const TAIL_EXTRUSIVE_SHARE: f64 = 1.0 / 7.0;
    /// The volcano stage's strength for a tail (its shields' height, VOL-008): the preset's
    /// reference uplift times 0.6 to 1.4 by the flux's rank in Sleep's table, so the volcanoes
    /// keep the spread they had and the strongest plume builds the tallest shields (game-choice:
    /// the hotspot lane, 2026-10-31, a mapping for the volcano stage, not a rate). ledger: PLT-801
    pub fn volcano_uplift_mm_yr(k: &TectonicParams, flux: f64) -> f64 {
        let below = FLUX_MG_S.iter().filter(|&&f| f < flux).count() as f64;
        let q = below / (COUNT - 1) as f64;
        k.hotspot_uplift_mm_yr * (0.6 + 0.8 * q)
    }
    /// Plume heads (large igneous provinces) per Myr: Ernst et al. (2021, AGU Monograph 255
    /// ch. 1; full text), "Continental LIPs have occurred on average (but irregularly) every 30
    /// myr throughout the Phanerozoic and Proterozoic. The oceanic LIP record is similar to that
    /// of continental LIPs for the past 200 myr": one continental and one oceanic every 30 Myr
    /// (the agent's reading of "similar"), one every 15 Myr. ledger: PLT-795
    pub const LIPS_PER_MYR: f64 = 2.0 / 30.0;
    /// The share of new tails that rise with a head: the heads' rate over the births'
    /// (0.0667 / 0.296 = 0.225 with the sourced births; at most 1). ledger: PLT-795
    pub fn head_share(births_per_myr: f64) -> f64 {
        if births_per_myr > 0.0 {
            (LIPS_PER_MYR / births_per_myr).min(1.0)
        } else {
            0.0
        }
    }
    /// Continental large igneous provinces, km³ of magma with their intrusive roots: Clift,
    /// Vannucchi & Phipps Morgan (2009, Table 7, "Estimated total volume with intrusives", four
    /// times the original volcanic volume; the authors' revised manuscript, partly recovered by
    /// the research sub-agent): Deccan, Siberian, Ethiopia, Paraná, Emeishan, Etendeka, Columbia
    /// River, Karoo, North Atlantic (sum 30,463,152, "0.12 km3/a" over 250 Myr). ledger: PLT-796
    pub const CONTINENTAL_LIPS_KM3: [f64; 9] = [
        2_956_972.0,
        12_200_000.0,
        2_363_892.0,
        2_720_000.0,
        1_200_000.0,
        144_000.0,
        748_712.0,
        4_121_877.0,
        4_007_699.0,
    ];
    /// Oceanic plateaus, km³ of crust: Ontong Java "44.4x10⁶ km³ (Coffin and Eldholm, 1994)"
    /// and the Wallaby Plateau "4.2x10⁶ km³ (Schubert and Sandwell, 1989)" as Clift et al. 2009
    /// (§7.1, the manuscript) give them, the Caribbean "4.5 million km³" (Kasbohm, Schoene &
    /// Burgess 2021, full text). ledger: PLT-796
    pub const OCEANIC_LIPS_KM3: [f64; 3] = [44_400_000.0, 4_500_000.0, 4_200_000.0];
    /// A head's province: one of the record's for the crust it rises under, drawn evenly (the
    /// lid's control on how much a head melts, White & McKenzie 1989, read from the record
    /// rather than modelled). ledger: PLT-796
    pub fn lip_volume_km3(rng: &mut SplitMix64, continental: bool) -> f64 {
        let set: &[f64] = if continental {
            &CONTINENTAL_LIPS_KM3
        } else {
            &OCEANIC_LIPS_KM3
        };
        set[(rng.next_u64() % set.len() as u64) as usize]
    }
    /// Of a province's magma, the share erupted: Clift et al. (2009, §7.2, the manuscript) after
    /// White et al. 2006, "many of the extrusive:intrusive ratios lie in the range 1:2 to 1:5. In
    /// this study we therefore take a value of 1:3". ledger: PLT-796
    pub const LIP_EXTRUSIVE_SHARE: f64 = 0.25;
    /// How far a head spreads under the lithosphere, km: Sleep (1990, p. 6732), "The area
    /// strongly affected by modern plume heads is typically 1000 km in radius (data from White
    /// and McKenzie [1989])"; White & McKenzie (1989), "A 1000-km-radius circle, which is the
    /// typical extent of the thermal anomaly created by a plume"; Campbell & Griffiths (1990,
    /// p. 87), a head "forms a disk ~2000 km in diameter". ledger: PLT-797
    pub const HEAD_RADIUS_KM: f64 = 1000.0;
    /// The volcano stage's flood-basalt plateau at `x` of the radius, 1 − smoothstep(0.6, 1, x)
    /// (VOL-035), whose rock beyond the plate model's dome the volcano stage lays.
    pub fn plateau(x: f64) -> f64 {
        let t = ((x - 0.6) / 0.4).clamp(0.0, 1.0);
        1.0 - t * t * (3.0 - 2.0 * t)
    }
    /// A head's hot mantle above the normal mantle, K: White & McKenzie (1989; full text),
    /// "temperatures raised 100-200 °C above normal" (abstract), "of the order of 100-150 °C
    /// above normal" (conclusion); Franke (2013) quotes White: "an anomalously hot mantle
    /// (150-200° above normal) must be present under the rift shortly before continental
    /// breakup". 150 K is the value in all three ranges (the agent's reading). ledger: PLT-798
    pub const HEAD_EXCESS_K: f64 = 150.0;
    /// How long after its rise a head's hot mantle lies under the lithosphere, Myr: Campbell &
    /// Griffiths (1990; full text, abstract): the bulk of the magma within 2 to 3 Myr, smaller
    /// volumes over a further 5 to 10, and the activity decays "over a total time of order 20
    /// Ma". White & McKenzie's Table 2 puts the oceans' first spreading within that of their
    /// flood basalts (North Atlantic 58 to 56 Ma after 63 to 52, Deccan 64 within 67 to 60,
    /// Paraná 130 to 120 against 130 to 117). ledger: PLT-798
    pub const HEAD_HOT_MYR: f64 = 20.0;
    /// White & McKenzie (1989, Fig. 3, the research sub-agent's reading of the figure, ±1 km):
    /// the igneous crust mantle melts to at full breakup (β = 50, their stand-in for infinite
    /// stretching), m, at a potential temperature above the normal 1280 °C, K: 7 km at 1280 °C,
    /// 17 km at 1400 °C, 26 km at 1480 °C ("an increase of 200 °C above its normal value ... can
    /// more than triple the amount of melt"). ledger: PLT-799
    pub const IGNEOUS_CRUST_M: [(f64, f64); 3] =
        [(0.0, 7000.0), (120.0, 17_000.0), (200.0, 26_000.0)];
    /// [`IGNEOUS_CRUST_M`] interpolated linearly (the agent's), held at its ends.
    pub fn igneous_crust_m(excess_k: f64) -> f64 {
        let t = &IGNEOUS_CRUST_M;
        if excess_k.is_nan() || excess_k <= t[0].0 {
            return t[0].1;
        }
        for w in t.windows(2) {
            if excess_k <= w[1].0 {
                let f = (excess_k - w[0].0) / (w[1].0 - w[0].0);
                return w[0].1 + f * (w[1].1 - w[0].1);
            }
        }
        t[t.len() - 1].1
    }
    /// Of a volcanic margin's igneous crust, the share erupted (its seaward-dipping lavas over
    /// the high-velocity lower crust): the large igneous provinces' 1:3 (Clift et al. 2009);
    /// White & McKenzie (1989): "accompanying the extruded basalts ... there is an even greater
    /// volume of new underplated or intruded igneous rock in the lower crust"; the North
    /// Atlantic's "~1.8 million km³" extrusive of "between 5 and 10 million km³" (Kasbohm et al.
    /// 2021), 0.18 to 0.36. ledger: PLT-799
    pub const MARGIN_EXTRUSIVE_SHARE: f64 = 0.25;
    /// How far seaward of the continent the thick igneous crust reaches, km: Kelemen & Holbrook
    /// (1995, abstract), the US East Coast's "only for about 80–100 km seaward" (the middle of
    /// the range).
    pub const MARGIN_WIDTH_KM: f64 = 90.0; // ledger: PLT-802

    // The lid (docs/design/hotspot-lid.md; the sources in bibliography.md "The sources of the
    // hotspot lane's lid").

    /// A tail's plume's potential temperature, °C: Hawaii's, "Our calculated potential
    /// temperature below Hawaii is ∼1550°C" (Herzberg et al. 2007, para. 68, full text); Watson
    /// & McKenzie (1991, abstract): "a central potential temperature of 1558 °C". Applied to
    /// every tail. ledger: PLT-806
    pub const PLUME_TP_C: f64 = 1550.0;
    /// The anhydrous peridotite solidus, T = A1 + A2 P + A3 P² (°C, P in GPa): Katz, Spiegelman &
    /// Langmuir (2003, eq. 4 and Table 2, full text): 1085.7 °C, 132.9 °C/GPa, −5.1 °C/GPa².
    /// ledger: PLT-808
    pub const SOLIDUS_C: [f64; 3] = [1085.7, 132.9, -5.1];
    /// The plume's adiabat, °C per km (Fitton et al. 2020's melting model, read through the
    /// fetch tool's summary), and depth per pressure, km per GPa (Korenaga, Kelemen & Holbrook
    /// 2002: P_f = (H + b) / 30, full text). ledger: PLT-808
    pub const ADIABAT_C_PER_KM: f64 = 0.3;
    pub const KM_PER_GPA: f64 = 30.0;
    /// The mechanical boundary layer under the thermal lithosphere's base, km: White & McKenzie
    /// (1989, Fig. 3 caption, full text): "mechanical boundary layer thicknesses of 70, 100, and
    /// 130 km correspond to plate thicknesses of 87, 118, and 149 km" (17, 18, 19 km). ledger:
    /// PLT-807
    pub const PLATE_LESS_MBL_KM: f64 = 18.0;
    /// Hawaii's mechanical boundary layer, km, the lid Sleep's magma per flux (PLT-793) melts
    /// under: its plate "≥90 km thick" (Humphreys & Niu 2009, full text) less
    /// [`PLATE_LESS_MBL_KM`]; Watson & McKenzie (1991, abstract): "the mechanical boundary layer
    /// is 72 km thick". ledger: PLT-807
    pub const HAWAII_MBL_KM: f64 = 72.0;

    /// The depth where a plume `tp_c` hot begins to melt, km: where its adiabat crosses the
    /// solidus (the agent's construction from PLT-808; 56 km at 1300 °C and 120 km at 1500 °C,
    /// Fitton et al.'s 57 and ∼120). ledger: PLT-808
    pub fn melt_onset_km(tp_c: f64) -> f64 {
        let [a1, a2, a3] = SOLIDUS_C;
        // T_p + g z = a1 + a2 P + a3 P², z = k P: a3 P² + (a2 − g k) P + (a1 − T_p) = 0.
        let b = a2 - ADIABAT_C_PER_KM * KM_PER_GPA;
        let c = a1 - tp_c;
        let disc = b * b - 4.0 * a3 * c;
        if disc < 0.0 {
            return f64::INFINITY;
        }
        // The shallower root (a3 < 0: the solidus bends over at depth).
        let p = (-b + disc.sqrt()) / (2.0 * a3);
        (p * KM_PER_GPA).max(0.0)
    }

    /// A tail's melt per unit of flux under a thermal lithosphere `l_km` deep, as a share of
    /// Hawaii's (PLT-793's): the interval it rises through above the solidus to its lid, the
    /// mechanical boundary layer (L − [`PLATE_LESS_MBL_KM`], at least 0), over Hawaii's (Fitton et
    /// al.'s F_max at the lid's base; Niu 2021's linear scaling of the melting interval): 1 under
    /// Hawaii's lid, 0 under a lid at or below the onset (a craton's keel). 1 with no thermal
    /// state (`l_km` 0 or less). ledger: PLT-805
    pub fn lid_factor(l_km: f64) -> f64 {
        if l_km.is_nan() || l_km <= 0.0 {
            return 1.0;
        }
        let z0 = melt_onset_km(PLUME_TP_C);
        let lid = (l_km - PLATE_LESS_MBL_KM).max(0.0);
        ((z0 - lid) / (z0 - HAWAII_MBL_KM)).max(0.0)
    }
}

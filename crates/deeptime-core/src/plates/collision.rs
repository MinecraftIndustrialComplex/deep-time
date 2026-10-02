// SPDX-License-Identifier: LGPL-3.0-or-later
//! Continental collision and terrane transfer (Cortial et al. 2019, §4.2 and §6).
//!
//! A contact is in collision while the subducting plate's continental crust stands at the
//! overriding plate's edge, and counts its shortening: the convergence across that front
//! (`shortening.rs`). When the contact's count passes `collision_threshold_km` a collision event
//! fires (an interim trigger, and which count is `tectonics.collision_on_shortening`: the design's
//! task 4 welds plates when their convergence has stalled instead): the terrane R — the connected continental crust of
//! the subducting plate that reached the contact, "from small isolated islands to entire
//! continents" — detaches from its plate and sutures to the overriding one (here: its samples
//! move into a new rigid group of the receiving plate with the rotation they already had), and the
//! crust around the suture rises by the discrete surge
//!
//! Δz(p) = Δc · A · (1 − (d/r)²)², r = r_c · √(v(q)/v_0 · A/A_0),
//!
//! with A the terrane's area, A_0 the average initial plate area, v the convergence speed and the
//! fold direction set to f(p) = (n × (p − q)/‖p − q‖) × n (q the terrane's centroid). Two
//! deviations, both recorded in STATUS.md: d is measured from the contact (the forced-subducted
//! continental crust) rather than from the whole terrane, so a colliding continent rises along its
//! suture instead of everywhere at once (for a small terrane the two coincide), and Δc·A is capped
//! at `collision_uplift_max_m` (Δc·A of a continent would be tens of kilometres) and scaled by the
//! isostatic ceiling (`subduction::ceiling`). The receiving plate's motion becomes the
//! area-weighted mean of the two plates' (a collision slows it).
//!
//! Events by terrane size (A/A_0): below `terrane_event_min_frac` the terrane accretes silently
//! (island-arc slivers are frequent); below `collision_min_frac` a terrane_accretion; above it a
//! collision, its terrane_accretion and a Himalayan orogeny_phase.

use std::collections::BTreeMap;

use deeptime_format::EventId;
use glam::DVec3;

use super::Env;
use super::dynamics::{CollisionRec, Moments};
use super::events::{self, COLLISION, OROGENY_PHASE, PLATE_DEATH, TERRANE_ACCRETION};
use super::geometry::{Geo, coast_split, dijkstra};
use super::kinematics::{cap_speed, quat, tangent, to_group};
use super::state::{CONTINENTAL, Group, HIDDEN, OROGENY_HIMALAYAN, PlateState, pool};
use super::subduction::ceiling;
use crate::error::CoreError;
use crate::world::WorldState;

/// What a collision did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Collided {
    /// The collision (or terrane accretion) event, when one was written.
    pub event: Option<EventId>,
    /// The subducting plate sutured whole to the receiving one (it is dead now; the caller
    /// resamples the crust this step, so its subducted slab is consumed).
    pub sutured: bool,
}

/// Runs the collision of contact `k`, whose subducting plate's continental crust `seeds` has been
/// forced under the overriding plate.
pub fn collide(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    k: usize,
    seeds: &[u32],
) -> Result<Collided, CoreError> {
    let p = env.k;
    let t_myr = env.t_myr;
    let norm = env.norm;
    let (under, over) = (state.contacts[k].under, state.contacts[k].over);
    if under == over || !state.plates[under as usize].alive || !state.plates[over as usize].alive {
        return Ok(Collided::default());
    }
    let n = state.samples.len();
    let plate_of: Vec<u32> = (0..n).map(|i| state.plate_of(i)).collect();

    // The terrane: continental crust of the subducting plate connected to the seeds.
    let mut in_terrane = vec![false; n];
    let mut stack: Vec<u32> = Vec::new();
    for &s in seeds {
        if plate_of[s as usize] == under && state.samples[s as usize].kind == CONTINENTAL {
            in_terrane[s as usize] = true;
            stack.push(s);
        }
    }
    let mut terrane = Vec::new();
    while let Some(v) = stack.pop() {
        terrane.push(v);
        for (u, _) in geo.edges(v) {
            let ui = u as usize;
            if !in_terrane[ui] && plate_of[ui] == under && state.samples[ui].kind == CONTINENTAL {
                in_terrane[ui] = true;
                stack.push(u);
            }
        }
    }
    if terrane.is_empty() {
        super::shortening::end(state, k, super::dynamics::EMPTIED);
        state.contacts[k].speed_count_km = 0.0;
        return Ok(Collided::default());
    }
    terrane.sort_unstable();
    let area_km2 = terrane.len() as f64 * norm.sample_area_km2();
    let over_samples = f64::from(state.plates[over as usize].samples.max(1));
    let q = terrane
        .iter()
        .map(|&i| pos[i as usize])
        .sum::<DVec3>()
        .normalize_or(DVec3::from_array(state.contacts[k].centroid));

    let suture = area_km2 / state.a0_km2 >= p.collision_suture_frac;
    // The terrane's coasts whose ocean stays behind on the subducting plate (none with a suture).
    let split_coast = if suture {
        0
    } else {
        coast_split(
            geo,
            state,
            &terrane,
            |i| plate_of[i] == under,
            |i| u32::from(in_terrane[i]),
        )
    };
    // A continent too large to be carried under sutures its whole plate to the receiving one
    // (P1-27): the two plates become one, so the continent keeps the ocean on its other sides
    // and those margins stay passive. A smaller terrane moves alone to the receiving plate
    // (an island arc or a microcontinent accretes and subduction steps outboard of it).
    let under_samples = f64::from(state.plates[under as usize].samples.max(1));
    if suture {
        for g in state.groups.iter_mut().filter(|g| g.plate == under) {
            g.plate = over;
        }
        for &i in &terrane {
            state.samples[i as usize].flags &= !HIDDEN;
        }
    } else {
        // Transfer: each of the terrane's groups gets a twin in the receiving plate.
        let mut twin: BTreeMap<u32, u32> = BTreeMap::new();
        for &i in &terrane {
            let g = state.samples[i as usize].group;
            let new = *twin.entry(g).or_insert_with(|| {
                let rot = state.groups[g as usize].rot;
                state.groups.push(Group { plate: over, rot });
                (state.groups.len() - 1) as u32
            });
            let s = &mut state.samples[i as usize];
            s.group = new;
            s.flags &= !HIDDEN;
        }
    }

    // The surge around the suture.
    let omega_u = DVec3::from_array(state.plates[under as usize].omega);
    let omega_o = DVec3::from_array(state.plates[over as usize].omega);
    let centroid = DVec3::from_array(state.contacts[k].centroid);
    let conv = norm.speed_mm_yr(omega_u - omega_o, centroid);
    let ratio = ((conv / p.max_plate_speed_mm_yr).min(1.0) * area_km2 / state.a0_km2).max(0.0);
    let r_km = (p.collision_distance_km * ratio.sqrt())
        .clamp(2.0 * norm.km(norm.spacing_rad()), p.collision_distance_km);
    let dz_max = (p.collision_coefficient_per_km * area_km2 * 1000.0).min(p.collision_uplift_max_m);
    let over_plate = |i: u32| {
        let g = state.samples[i as usize].group as usize;
        state.groups[g].plate == over
    };
    let sources: Vec<(u32, f64, u32)> = seeds
        .iter()
        .filter(|&&s| over_plate(s))
        .map(|&s| (s, 0.0, 0))
        .collect();
    let (dist, _) = dijkstra(
        geo,
        &sources,
        |a, b| over_plate(a) && over_plate(b),
        norm.rad(r_km),
    );
    let r_rad = norm.rad(r_km);
    let mut uplifted = 0usize;
    // Measurement only (`dynamics.rs`): the ground the surge lifted, by more than 10 m and 100 m,
    // and from below the sea to above it; the overlap's continental crust.
    let recording = state.dynamics.is_some();
    let (mut m10, mut m100, mut memerged, mut mseeds) = (
        Moments::default(),
        Moments::default(),
        Moments::default(),
        Moments::default(),
    );
    if recording {
        for &s in seeds {
            mseeds.add(pos[s as usize]);
        }
    }
    for (i, &d) in dist.iter().enumerate() {
        if !d.is_finite() || d > r_rad {
            continue;
        }
        let x = d / r_rad;
        let w = (1.0 - x * x).powi(2);
        let rot = quat(state.groups[state.samples[i].group as usize].rot);
        let s = &mut state.samples[i];
        let z = f64::from(s.elevation_m);
        let dz = dz_max * w * ceiling(z, p.uplift_ceiling_m);
        s.elevation_m = (z + dz).min(p.max_elevation_m) as f32;
        if recording {
            if dz > 10.0 {
                m10.add(pos[i]);
            }
            if dz > 100.0 {
                m100.add(pos[i]);
            }
            if z <= 0.0 && z + dz > 0.0 {
                memerged.add(pos[i]);
            }
        }
        // The orogen proper is the core of the surge, where the crust thickened most (P1-27:
        // stamping everything the surge lifted by 10 m made belts 4,000 km wide, and most of
        // every continent a collision orogen); the rest only rises.
        if dz > 10.0 && w >= p.collision_orogen_core {
            uplifted += 1;
            s.orogeny = OROGENY_HIMALAYAN;
            s.orogeny_myr = t_myr as f32;
            let at = pos[i];
            let dir = tangent(at, at - q).normalize_or_zero();
            s.fold = to_group(rot, dir);
            // Regional metamorphism of the thickened crust.
            let m = s.pools[pool::SEDIMENTARY] * (0.3 * w) as f32;
            s.pools[pool::SEDIMENTARY] -= m;
            s.pools[pool::METAMORPHIC] += m;
        }
    }

    // The receiving plate carries the terrane's momentum (with a suture, the whole plate's).
    let t_samples = if suture {
        under_samples
    } else {
        terrane.len() as f64
    };
    let merged = (omega_o * over_samples + omega_u * t_samples) / (over_samples + t_samples);
    state.plates[over as usize].omega =
        cap_speed(merged, &norm, p.max_plate_speed_mm_yr).to_array();

    // The collision state ends here, with the transfer or weld. (While the pre-task-2 count
    // fires, a contact may fire with no collision state: its plates slid more than they
    // converged, or its crust at the front is freshly sutured. That is a collision of one step.)
    let (cont_in, contact_onset_myr) = (state.contacts[k].cont_in, state.contacts[k].onset_myr);
    let counted_km = state.contacts[k].speed_count_km;
    let col = state.contacts[k].collision.take();
    state.contacts[k].cont_in = 0;
    state.contacts[k].speed_count_km = 0.0;
    let collision_onset_myr = col.as_ref().map_or(t_myr, |c| c.onset_myr);
    let shortened_km = col.as_ref().map_or(0.0, |c| c.shortening_km);
    let crossed_km2 = col.as_ref().map_or(0.0, |c| c.area_km2);
    let planet_area = norm.planet_area_km2(area_km2);
    let onset = super::subduction::confirm(state, world, env, k)?;
    let frac = area_km2 / state.a0_km2;
    let coast_km = norm.planet_len_km(split_coast as f64 * norm.km(norm.spacing_rad()));
    let event = if frac < p.terrane_event_min_frac {
        state.counters.accretions += 1;
        None
    } else if frac < p.collision_min_frac {
        let e = events::event(world, t_myr, TERRANE_ACCRETION, q)
            .magnitude(frac as f32)
            .param("from_plate", i64::from(under))
            .param("to_plate", i64::from(over))
            .param("area_km2", planet_area)
            .param("samples", terrane.len() as i64)
            .param("uplift_m", dz_max)
            .param("coast_split_km", coast_km)
            .param("sutured", i64::from(suture))
            .caused_by(onset);
        let id = world.emit(e)?;
        state.counters.accretions += 1;
        Some(id)
    } else {
        let e = events::event(world, t_myr, COLLISION, centroid)
            .magnitude((dz_max / 1000.0) as f32)
            .param("plate_a", i64::from(under))
            .param("plate_b", i64::from(over))
            .param("terrane_area_km2", planet_area)
            .param("uplift_m", dz_max)
            .param("radius_km", norm.planet_len_km(r_km))
            .param("convergence_mm_yr", conv)
            .param("onset_myr", collision_onset_myr)
            .param("shortening_km", shortened_km)
            .param("coast_split_km", coast_km)
            .param("sutured", i64::from(suture))
            .caused_by(onset);
        let collision = world.emit(e)?;
        let e = events::event(world, t_myr, TERRANE_ACCRETION, q)
            .magnitude((area_km2 / state.a0_km2) as f32)
            .param("from_plate", i64::from(under))
            .param("to_plate", i64::from(over))
            .param("area_km2", planet_area)
            .param("samples", terrane.len() as i64)
            .caused_by(collision);
        world.emit(e)?;
        let e = events::event(world, t_myr, OROGENY_PHASE, centroid)
            .until((t_myr + p.collision_orogeny_myr) * 1.0e6)
            .magnitude((dz_max / 1000.0) as f32)
            .param("orogeny", "himalayan")
            .param("plate", i64::from(over))
            .param("uplift_m", dz_max)
            .param("uplifted_samples", uplifted as i64)
            .caused_by(collision);
        world.emit(e)?;
        state.counters.collisions += 1;
        state.counters.accretions += 1;
        Some(collision)
    };
    if suture {
        let at = DVec3::from_array(state.plates[under as usize].centroid);
        let e = events::event(world, t_myr, PLATE_DEATH, at)
            .param("plate", i64::from(under))
            .param("fate", "sutured")
            .param("absorbed_by", i64::from(over))
            .caused_by(event.unwrap_or(onset));
        world.emit(e)?;
        let dead = &mut state.plates[under as usize];
        dead.alive = false;
        dead.died_myr = Some(t_myr);
        dead.samples = 0;
        state.counters.plate_deaths += 1;
        state.counters.sutures += 1;
    }
    if recording {
        let seeds_fp = mseeds.footprint(norm.model_km);
        let under_plate = &state.plates[under as usize];
        let rec = CollisionRec {
            collision: frac >= p.collision_min_frac,
            sutured: suture,
            start_myr: collision_onset_myr,
            end_myr: t_myr,
            contact_onset_myr,
            under,
            over,
            under_samples: under_samples as u32,
            over_samples: over_samples as u32,
            terrane_samples: terrane.len() as u32,
            frac,
            convergence_mm_yr: conv,
            counted_km,
            front_shortening_km: shortened_km,
            area_km2: crossed_km2,
            onset_convergence_mm_yr: col.as_ref().map(|c| c.onset_convergence_mm_yr),
            cont_in,
            front_km: seeds_fp.length_km,
            shortening_km: if seeds_fp.length_km > 0.0 {
                f64::from(cont_in) * norm.sample_area_km2() / seeds_fp.length_km
            } else {
                0.0
            },
            peak_uplift_m: dz_max,
            radius_km: r_km,
            lifted10: m10.footprint(norm.model_km),
            lifted100: m100.footprint(norm.model_km),
            emerged: memerged.footprint(norm.model_km),
            under_born_myr: under_plate.born_myr,
            under_parent: under_plate.parent,
        };
        if let Some(d) = state.dynamics.as_mut() {
            // The belt record's convergence history (the wedge and foredeep come with task 3).
            if let (true, Some(c)) = (rec.collision, &col) {
                d.belts.push(super::dynamics::BeltRec {
                    start_myr: c.onset_myr,
                    under,
                    over,
                    convergence_onset_mm_yr: Some(c.onset_convergence_mm_yr),
                    convergence_20myr_mm_yr: c.later_convergence_mm_yr,
                    ..Default::default()
                });
            }
            d.collisions.push(rec);
        }
    }
    Ok(Collided {
        event,
        sutured: suture,
    })
}

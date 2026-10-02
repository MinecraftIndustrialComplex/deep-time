// SPDX-License-Identifier: LGPL-3.0-or-later
//! The end of a collision: terrane transfer (Cortial et al. 2019, §4.2 and §6) and the weld of two
//! plates on stalled convergence (task 4 of `docs/design/plate-force-balance.md`; option 2B of
//! `docs/design/plate-collisions-and-welding.md`).
//!
//! A contact is in collision while the subducting plate's continental crust stands at the
//! overriding plate's edge and converges on it (`shortening.rs`); every step of it stacks the crust
//! shortened into a belt and a wedge and resists the convergence (`belts.rs`). [`collide`] ends it
//! for a contact that `shortening::advance` hands over, with the terrane R — the connected
//! continental crust of the subducting plate that reached the front, "from small isolated islands
//! to entire continents":
//!
//! - a terrane of at least `collision_suture_frac` of an average plate (a continent) **welds** its
//!   plate to the other one, once the collision has stalled (its convergence below
//!   `weld_speed_mm_yr` for `weld_after_myr`): the two plates become one, the **larger keeps its
//!   id**, and the plate takes the two plates' area-weighted mean rotation. Before it has stalled
//!   nothing happens: the collision goes on converging, building its belt;
//! - a smaller terrane (an island arc, a microcontinent) **moves alone** to the overriding plate
//!   (its samples move into a new rigid group of that plate with the rotation they already had),
//!   once the collision has shortened by `collision_threshold_km` or stalled; the overriding plate
//!   takes the area-weighted mean of its rotation and the terrane's.
//!
//! No uplift is applied here: the belt is built while the collision lasts. (Until tasks 3 and 4
//! a collision raised everything within up to 2,000 km of the contact by one surge in the step it
//! fired, capped at `collision_uplift_max_m`.)
//!
//! Events by terrane size (A/A_0): below `terrane_event_min_frac` the terrane accretes silently
//! (island-arc slivers are frequent); below `collision_min_frac` a terrane_accretion; above it a
//! collision, its terrane_accretion and a Himalayan orogeny_phase from the collision's onset to
//! its end.

use std::collections::BTreeMap;

use deeptime_format::EventId;
use glam::DVec3;

use super::Env;
use super::dynamics::{CollisionRec, Footprint};
use super::events::{self, COLLISION, OROGENY_PHASE, PLATE_DEATH, TERRANE_ACCRETION};
use super::geometry::{Geo, coast_split};
use super::kinematics::cap_speed;
use super::state::{CONTINENTAL, Group, HIDDEN, PlateState};
use crate::error::CoreError;
use crate::world::WorldState;

/// What a collision did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Collided {
    /// The collision (or terrane accretion) event, when one was written.
    pub event: Option<EventId>,
    /// The two plates welded into one (one is dead now; the caller resamples the crust this
    /// step, so the subducted slab is consumed).
    pub sutured: bool,
    /// Nothing happened: the terrane is a continent and the collision has not stalled yet.
    pub deferred: bool,
}

/// Ends the collision of contact `k`, whose subducting plate's continental crust `seeds` stands at
/// or under the overriding plate, when it is due (see the module docs): `stalled` says whether
/// its convergence has stayed below `weld_speed_mm_yr` for `weld_after_myr`.
#[allow(clippy::too_many_arguments)]
pub fn collide(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    k: usize,
    seeds: &[u32],
    stalled: bool,
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
        return Ok(Collided::default());
    }
    terrane.sort_unstable();
    let area_km2 = terrane.len() as f64 * norm.sample_area_km2();
    // ledger: PLT-078 (a continent welds its plate; a smaller terrane moves alone).
    let suture = area_km2 / state.a0_km2 >= p.collision_suture_frac;
    if suture && !stalled {
        // A continent goes on converging until its collision stalls.
        return Ok(Collided {
            deferred: true,
            ..Collided::default()
        });
    }
    let over_samples = f64::from(state.plates[over as usize].samples.max(1));
    let under_samples = f64::from(state.plates[under as usize].samples.max(1));
    let q = terrane
        .iter()
        .map(|&i| pos[i as usize])
        .sum::<DVec3>()
        .normalize_or(DVec3::from_array(state.contacts[k].centroid));

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
    let omega_u = DVec3::from_array(state.plates[under as usize].omega);
    let omega_o = DVec3::from_array(state.plates[over as usize].omega);
    // A continent welds its whole plate to the other (P1-27: the continent keeps the ocean on its
    // other sides, and those margins stay passive): the two plates are one from now on, and the
    // larger keeps its id. A smaller terrane moves alone to the overriding plate (an island arc
    // or a microcontinent accretes and subduction steps outboard of it).
    // ledger: PLT-237 (the larger plate keeps its id).
    let (keep, die) = if suture && under_samples > over_samples {
        (under, over)
    } else {
        (over, under)
    };
    if suture {
        for g in state.groups.iter_mut().filter(|g| g.plate == die) {
            g.plate = keep;
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

    // The receiving plate carries the terrane's momentum (with a weld, the whole plate's).
    // ledger: PLT-080.
    let merged = if suture {
        (omega_o * over_samples + omega_u * under_samples) / (over_samples + under_samples)
    } else {
        let t = terrane.len() as f64;
        (omega_o * over_samples + omega_u * t) / (over_samples + t)
    };
    state.plates[keep as usize].omega =
        cap_speed(merged, &norm, p.max_plate_speed_mm_yr).to_array();

    // The collision state ends here, with the transfer or weld.
    let (cont_in, contact_onset_myr) = (state.contacts[k].cont_in, state.contacts[k].onset_myr);
    let col = state.contacts[k].collision.take();
    state.contacts[k].cont_in = 0;
    let collision_onset_myr = col.as_ref().map_or(t_myr, |c| c.onset_myr);
    let shortened_km = col.as_ref().map_or(0.0, |c| c.shortening_km);
    let crossed_km2 = col.as_ref().map_or(0.0, |c| c.area_km2);
    let conv = col.as_ref().map_or(0.0, |c| c.convergence_mm_yr);
    let belt = col.as_ref().map(|c| c.belt.clone()).unwrap_or_default();
    let centroid = col
        .as_ref()
        .map_or(DVec3::from_array(state.contacts[k].centroid), |c| {
            DVec3::from_array(c.centroid)
        });
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
            .param("uplift_m", belt.uplift_m)
            .param("coast_split_km", coast_km)
            .param("sutured", i64::from(suture))
            .caused_by(onset);
        let id = world.emit(e)?;
        state.counters.accretions += 1;
        Some(id)
    } else {
        let e = events::event(world, t_myr, COLLISION, centroid)
            .magnitude((belt.peak_height_m / 1000.0) as f32)
            .param("plate_a", i64::from(under))
            .param("plate_b", i64::from(over))
            .param("terrane_area_km2", planet_area)
            .param("uplift_m", belt.uplift_m)
            .param("belt_height_m", belt.peak_height_m)
            .param("convergence_mm_yr", conv)
            .param("onset_myr", collision_onset_myr)
            .param("shortening_km", shortened_km)
            .param("coast_split_km", coast_km)
            .param("sutured", i64::from(suture))
            .param("stalled", i64::from(stalled))
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
        // The orogeny lasted the collision: from its onset to now.
        let e = events::event(world, collision_onset_myr, OROGENY_PHASE, centroid)
            .until(t_myr * 1.0e6)
            .magnitude((belt.peak_height_m / 1000.0) as f32)
            .param("orogeny", "himalayan")
            .param("plate", i64::from(over))
            .param("uplift_m", belt.uplift_m)
            .param("belt_height_m", belt.peak_height_m)
            .caused_by(collision);
        world.emit(e)?;
        state.counters.collisions += 1;
        state.counters.accretions += 1;
        Some(collision)
    };
    if suture {
        let at = DVec3::from_array(state.plates[die as usize].centroid);
        let e = events::event(world, t_myr, PLATE_DEATH, at)
            .param("plate", i64::from(die))
            .param("fate", "sutured")
            .param("absorbed_by", i64::from(keep))
            .caused_by(event.unwrap_or(onset));
        world.emit(e)?;
        let dead = &mut state.plates[die as usize];
        dead.alive = false;
        dead.died_myr = Some(t_myr);
        dead.samples = 0;
        state.counters.plate_deaths += 1;
        state.counters.sutures += 1;
    }
    if state.dynamics.is_some() {
        let under_plate = &state.plates[under as usize];
        let most = |samples: u32| Footprint {
            samples,
            ..Footprint::default()
        };
        let rec = CollisionRec {
            collision: frac >= p.collision_min_frac,
            sutured: suture,
            died: suture.then_some(die),
            stalled,
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
            front_shortening_km: shortened_km,
            area_km2: crossed_km2,
            onset_convergence_mm_yr: col.as_ref().map(|c| c.onset_convergence_mm_yr),
            cont_in,
            front_km: col.as_ref().map_or(0.0, |c| c.peak_front_km),
            shortening_km: shortened_km,
            peak_uplift_m: belt.uplift_m,
            lifted10: most(belt.lifted10),
            lifted100: most(belt.lifted100),
            emerged: most(belt.emerged),
            belt: super::belts::footprint(&belt, norm.model_km),
            under_born_myr: under_plate.born_myr,
            under_parent: under_plate.parent,
        };
        if let Some(d) = state.dynamics.as_mut() {
            if let (true, Some(c)) = (rec.collision, &col) {
                let shaped = rec.belt.width_km > 0.0;
                // The design's foredeep is the one in front of a belt above 3 km.
                let high = belt.peak_height_m >= super::belts::FOREDEEP_BELT_M;
                d.belts.push(super::dynamics::BeltRec {
                    start_myr: c.onset_myr,
                    end_myr: Some(t_myr),
                    under,
                    over,
                    welded: suture,
                    convergence_onset_mm_yr: Some(c.onset_convergence_mm_yr),
                    convergence_20myr_mm_yr: c.later_convergence_mm_yr,
                    peak_height_m: Some(belt.peak_height_m),
                    belt_width_km: shaped.then_some(rec.belt.width_km),
                    belt_length_km: shaped.then_some(rec.belt.length_km),
                    wedge_width_km: Some(belt.peak_wedge_km),
                    foredeep_width_km: belt.foredeep_km.filter(|_| high),
                    foredeep_depth_m: belt.foredeep_m.filter(|_| high),
                });
            }
            d.collisions.push(rec);
        }
    }
    Ok(Collided {
        event,
        sutured: suture,
        deferred: false,
    })
}

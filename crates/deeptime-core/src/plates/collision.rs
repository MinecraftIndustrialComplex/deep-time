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
//!   takes the area-weighted mean of its rotation and the terrane's. A terrane its slab drags
//!   down whole (`tectonics.terrane_docking_cover`, ledger PLT-770) is not handed over on
//!   shortening (`shortening::advance` does not fire it): it goes down with the slab, its cover
//!   left on the overriding plate (`belts`), and only a stall hands it over;
//! - with `tectonics.transference` (Stern 2004's transference; ledger PLT-516), a continent whose
//!   collision has not stalled also moves alone, once its subduction has failed (the contact's
//!   slab is gone), its plate has gone on converging on it by the critical convergence since, and
//!   the collision's push on it over its margin against its own plate's ocean floor is enough to
//!   start a new subduction zone there ([`transference_due`]);
//! - with `tectonics.collision_backarc_reversal` (Stern 2004's polarity reversal; ledger
//!   PLT-737), in the same collision the overriding plate's continental block behind the front
//!   moves to the incoming plate instead, once the push carried to the block's back coast reaches
//!   that force there ([`backarc_block`]): a new subduction zone behind the block, where the
//!   overriding plate's own ocean floor meets it. Where both are due, the margin with the larger
//!   force per metre ruptures.
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
use super::forces::G_EARTH;
use super::geometry::{Geo, Tree, coast_split};
use super::kinematics::cap_speed;
use super::state::{CONTINENTAL, Group, HIDDEN, PlateState};
use crate::error::CoreError;
use crate::world::WorldState;

/// The force a plate must push with, per metre of a margin of oceanic lithosphere, to start a new
/// subduction zone there that goes on by itself, N/m at Earth's gravity: Gurnis et al. 2004 find
/// the resisting force "reaching a maximum value dependent on plate thickness, but typically
/// ranging from 2 to 3 × 10^12 N/m for cases that become self-sustaining"; the top of the range
/// is taken. Scaled with gravity as the slab pull is (FRC-002). ledger: PLT-517 (verified)
pub const TRANSFERENCE_FORCE_N_M: f64 = 3.0e12;

/// The convergence a new subduction zone needs before it goes on by itself, km: Gurnis et al.
/// 2004's critical convergence "only varies between 100 and 130 km (Table 3)" ("Cumulative
/// convergence, not the rate of convergence, is the dominant control"); Hall et al. 2003 give
/// "approximately 100 km of convergence". ledger: PLT-518 (verified)
pub const TRANSFERENCE_KM: f64 = 100.0;

/// Whether a continent pushed into a collision whose subduction has failed moves alone to the
/// overriding plate (Stern 2004's transference: "the oceanic lithosphere outboard of the
/// collision zone ultimately ruptures to become a new subduction zone"): its plate has converged
/// on it by [`TRANSFERENCE_KM`] since its slab was lost (`since_jam_km`), and the collision's push
/// on it (`push_n`, N) over the length of its margin against its own plate's ocean floor
/// (`margin_m`, m) reaches [`TRANSFERENCE_FORCE_N_M`] × `g_scale` (the planet's gravity over
/// Earth's). A continent with no ocean floor at its margin has nowhere to start the new zone.
/// ledger: PLT-516, PLT-517, PLT-518
pub fn transference_due(push_n: f64, margin_m: f64, since_jam_km: f64, g_scale: f64) -> bool {
    margin_m > 0.0
        && since_jam_km >= TRANSFERENCE_KM
        && push_n / margin_m >= TRANSFERENCE_FORCE_N_M * g_scale
}

/// The overriding block behind a collision front, its back coast and the collision's push there
/// ([`backarc_block`]; ledger PLT-737).
#[derive(Clone, Debug, PartialEq)]
pub struct Backarc {
    /// The block's samples, sorted.
    pub block: Vec<u32>,
    /// The push that reaches the back coast, N: the collision's, times the mean of e^(−x/L)
    /// over the coast.
    pub push_n: f64,
    /// The back coast's length, m.
    pub coast_m: f64,
}

/// The overriding block behind a collision front and the push that reaches its back coast, where
/// a new subduction zone would start ([`transference_due`] decides whether it does; ledger
/// PLT-737: the mirror of transference on the overriding side, Stern 2004's polarity reversal, "a
/// new subduction zone forming behind the magmatic arc"). None without a block or a back coast.
///
/// The block is plate `over`'s visible continental crust connected to its continental samples
/// on the front (`front`, [`super::shortening::Front::edge`]); its back coast the block's samples
/// next to visible ocean floor of their own plate and rigid group that lies farther from the front
/// than they do (x, the great-circle distance to the nearest front sample), so a forearc in front
/// of the block is not part of it. The collision's push (`push_n`, N) reaches the back coast as the
/// thin sheet's response does, by e^(−x/L) with L = 2W/π for a front `front_km` (W) long
/// (PLT-733's length), averaged over the coast.
#[allow(clippy::too_many_arguments)]
pub fn backarc_block(
    state: &PlateState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    over: u32,
    front: &[u32],
    front_km: f64,
    push_n: f64,
) -> Option<Backarc> {
    if front.is_empty() {
        return None;
    }
    let norm = env.norm;
    let plate_of = |i: usize| state.groups[state.samples[i].group as usize].plate;
    let crust = |i: usize| {
        let s = &state.samples[i];
        plate_of(i) == over && s.kind == CONTINENTAL && !s.is_hidden()
    };
    let n = state.samples.len();
    let mut in_block = vec![false; n];
    let mut stack: Vec<u32> = front
        .iter()
        .copied()
        .filter(|&i| crust(i as usize))
        .collect();
    for &i in &stack {
        in_block[i as usize] = true;
    }
    let mut block = Vec::new();
    while let Some(v) = stack.pop() {
        block.push(v);
        for (u, _) in geo.edges(v) {
            if !in_block[u as usize] && crust(u as usize) {
                in_block[u as usize] = true;
                stack.push(u);
            }
        }
    }
    if block.is_empty() {
        return None;
    }
    block.sort_unstable();
    let mut on_front = vec![false; n];
    for &i in front {
        on_front[i as usize] = true;
    }
    let edge = Tree::over(pos, |i| on_front[i]);
    // Great-circle distance behind the front, km (the tree's squared chord to an angle).
    let behind_km = |i: u32| {
        edge.nearest(pos[i as usize]).map_or(0.0, |(_, d2)| {
            norm.km(2.0 * (0.5 * d2.sqrt()).min(1.0).asin())
        })
    };
    let reach_km = super::belts::sheet_reach_km(front_km); // ledger: PLT-733
    let (mut coast, mut decay) = (0_usize, 0.0);
    for &i in &block {
        let g = state.samples[i as usize].group;
        let ocean: Vec<u32> = geo
            .edges(i)
            .map(|(u, _)| u)
            .filter(|&u| {
                let o = &state.samples[u as usize];
                plate_of(u as usize) == over
                    && o.group == g
                    && o.kind != CONTINENTAL
                    && !o.is_hidden()
            })
            .collect();
        if ocean.is_empty() {
            continue;
        }
        let x = behind_km(i);
        if ocean.iter().any(|&u| behind_km(u) > x) {
            coast += 1;
            decay += (-x / reach_km).exp();
        }
    }
    if coast == 0 {
        return None;
    }
    Some(Backarc {
        block,
        push_n: push_n * decay / coast as f64,
        coast_m: norm.km(coast as f64 * norm.spacing_rad()) * 1000.0,
    })
}

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
    /// A weld or a transfer moved the terrane and set both plates' motion this step (PLT-080).
    /// False when the collision was deferred and when nothing could happen: a plate already dead
    /// (welded earlier in the step) or a terrane gone (the collision ends emptied).
    pub moved: bool,
}

/// Ends the collision of contact `k`, whose subducting plate's continental crust `seeds` stands at
/// or under the overriding plate, when it is due (see the module docs): `stalled` says whether
/// its convergence has stayed below `weld_speed_mm_yr` for `weld_after_myr`; `push_n` is the
/// collision's plastic resistance this step, N (`forces::Answer::collision_n`), which
/// transference and the reversal behind the overriding block read; `front` is the overriding
/// plate's edge on this step's front, which the reversal reads ([`backarc_block`]).
#[allow(clippy::too_many_arguments)]
pub fn collide(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    k: usize,
    seeds: &[u32],
    front: &[u32],
    stalled: bool,
    push_n: Option<f64>,
) -> Result<Collided, CoreError> {
    let p = env.k;
    let t_myr = env.t_myr;
    let norm = env.norm;
    let (under, over) = (state.contacts[k].under, state.contacts[k].over);
    if under == over || !state.plates[under as usize].alive || !state.plates[over as usize].alive {
        return Ok(Collided::default());
    }
    let n = state.samples.len();
    // P1-19: the plate of a sample is read where it is needed (nothing below changes a plate
    // before the last read), not collected for the whole lattice at every contact.
    let plate_of = |i: usize| state.groups[state.samples[i].group as usize].plate;
    // ledger: PLT-078 (a continent welds its plate; a smaller terrane moves alone).
    let welds = |len: usize| {
        let area_km2 = len as f64 * norm.sample_area_km2();
        area_km2 / state.a0_km2 >= p.collision_suture_frac
    };
    // A continent goes on converging until its collision stalls.
    let deferred = Collided {
        deferred: true,
        ..Collided::default()
    };
    // Transference (ledger PLT-516): a pushed continent may go alone once its subduction has
    // failed and its plate has converged on it by the critical convergence since (the
    // shortening since the jam, km, and the push, N); the force is judged on the whole terrane.
    // The reversal behind the overriding block waits for the same (ledger PLT-737).
    let pushed = if (p.transference || p.collision_backarc_reversal) && !stalled {
        state.contacts[k]
            .collision
            .as_ref()
            .and_then(|c| c.jam_km.map(|j| c.shortening_km - j))
            .filter(|&km| km >= TRANSFERENCE_KM) // ledger: PLT-518
            .zip(push_n)
    } else {
        None
    };

    // The terrane: continental crust of the subducting plate connected to the seeds.
    let mut in_terrane = vec![false; n];
    let mut stack: Vec<u32> = Vec::new();
    for &s in seeds {
        if plate_of(s as usize) == under && state.samples[s as usize].kind == CONTINENTAL {
            in_terrane[s as usize] = true;
            stack.push(s);
        }
    }
    let mut terrane = Vec::new();
    while let Some(v) = stack.pop() {
        terrane.push(v);
        // P1-19: once the part found welds the plate (the test only grows with the terrane), an
        // unstalled collision is deferred whatever the rest of the terrane holds.
        if !stalled && pushed.is_none() && welds(terrane.len()) {
            return Ok(deferred);
        }
        for (u, _) in geo.edges(v) {
            let ui = u as usize;
            if !in_terrane[ui] && plate_of(ui) == under && state.samples[ui].kind == CONTINENTAL {
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
    let mut suture = welds(terrane.len());
    let mut transferred = false;
    // The overriding block, when it moves to the incoming plate (ledger PLT-737).
    let mut reversed: Option<Vec<u32>> = None;
    if suture && !stalled {
        let Some((since_km, push)) = pushed else {
            return Ok(deferred);
        };
        // The continent's margin against its own plate's ocean floor: where the new subduction
        // zone would start (the force per metre there, when it starts one).
        let ahead = p
            .transference
            .then(|| {
                let margin = terrane
                    .iter()
                    .filter(|&&i| {
                        !state.samples[i as usize].is_hidden()
                            && geo.edges(i).any(|(u, _)| {
                                let o = &state.samples[u as usize];
                                plate_of(u as usize) == under
                                    && o.kind != CONTINENTAL
                                    && !o.is_hidden()
                            })
                    })
                    .count();
                let margin_m = norm.km(margin as f64 * norm.spacing_rad()) * 1000.0;
                transference_due(push, margin_m, since_km, env.gravity / G_EARTH)
                    .then(|| push / margin_m)
            })
            .flatten();
        // Behind the overriding block: the other side's new subduction zone.
        let behind = p
            .collision_backarc_reversal
            .then(|| {
                let front_km = state.contacts[k]
                    .collision
                    .as_ref()
                    .map_or(0.0, |c| c.front_km);
                backarc_block(state, geo, pos, env, over, front, front_km, push)
            })
            .flatten()
            .filter(|b| transference_due(b.push_n, b.coast_m, since_km, env.gravity / G_EARTH))
            .map(|b| (b.block, b.push_n / b.coast_m));
        match (ahead, behind) {
            (None, None) => return Ok(deferred),
            (Some(a), Some((block, b))) if b > a => reversed = Some(block),
            (None, Some((block, _))) => reversed = Some(block),
            _ => transferred = true,
        }
        suture = false;
    }
    // What moves, from which plate to which: the terrane to the overriding plate, or with the
    // reversal the overriding block to the incoming plate.
    let (moving, from, to) = match &reversed {
        Some(block) => (block.as_slice(), over, under),
        None => (terrane.as_slice(), under, over),
    };
    let in_block: Vec<bool> = reversed.as_ref().map_or_else(Vec::new, |block| {
        let mut m = vec![false; n];
        for &i in block {
            m[i as usize] = true;
        }
        m
    });
    let moves = |i: usize| {
        if reversed.is_some() {
            in_block[i]
        } else {
            in_terrane[i]
        }
    };
    let over_samples = f64::from(state.plates[over as usize].samples.max(1));
    let under_samples = f64::from(state.plates[under as usize].samples.max(1));
    let q = moving
        .iter()
        .map(|&i| pos[i as usize])
        .sum::<DVec3>()
        .normalize_or(DVec3::from_array(state.contacts[k].centroid));

    // The coasts of what moves whose ocean stays behind on its plate (none with a suture).
    let split_coast = if suture {
        0
    } else {
        coast_split(
            geo,
            state,
            moving,
            |i| plate_of(i) == from,
            |i| u32::from(moves(i)),
        )
    };
    let omega_u = DVec3::from_array(state.plates[under as usize].omega);
    let omega_o = DVec3::from_array(state.plates[over as usize].omega);
    // A continent welds its whole plate to the other (P1-27: the continent keeps the ocean on its
    // other sides, and those margins stay passive): the two plates are one from now on, and the
    // larger keeps its id. A smaller terrane moves alone to the overriding plate (an island arc
    // or a microcontinent accretes and subduction steps outboard of it).
    // ledger: PLT-250 (the larger plate keeps its id).
    let (keep, die) = if suture && under_samples > over_samples {
        (under, over)
    } else {
        (to, from)
    };
    // The terrane's crust that crossed the front during the collision (`stacked`, under the
    // overriding plate) was stacked into the belt or went down with the slab when it crossed
    // (`belts.rs`): it stays under, and the next resampling consumes it (ledger: PLT-276,
    // PLT-291). Until 2026-10-18 the transfer and the weld brought it back up as crust of the
    // receiving plate, under that plate's own crust, and the resampling then added it to the
    // column on top: the crossed crust counted twice, in lumps of a whole continental column.
    if suture {
        for g in state.groups.iter_mut().filter(|g| g.plate == die) {
            g.plate = keep;
        }
        for &i in &terrane {
            let s = &mut state.samples[i as usize];
            if !s.stacked && s.is_hidden() {
                s.flags &= !HIDDEN;
                super::shown::collided(state, i as usize);
            }
        }
    } else {
        // Transfer: each of the moving crust's groups gets a twin in the receiving plate.
        let mut twin: BTreeMap<u32, u32> = BTreeMap::new();
        let moving: Vec<u32> = moving
            .iter()
            .copied()
            .filter(|&i| !state.samples[i as usize].stacked)
            .collect();
        for i in moving {
            let g = state.samples[i as usize].group;
            let new = *twin.entry(g).or_insert_with(|| {
                let rot = state.groups[g as usize].rot;
                state.groups.push(Group { plate: to, rot });
                (state.groups.len() - 1) as u32
            });
            let s = &mut state.samples[i as usize];
            s.group = new;
            if s.is_hidden() {
                s.flags &= !HIDDEN;
                super::shown::collided(state, i as usize);
            }
        }
    }

    // The receiving plate carries the moving crust's momentum (with a weld, the whole plate's).
    // ledger: PLT-080.
    let merged = if suture {
        (omega_o * over_samples + omega_u * under_samples) / (over_samples + under_samples)
    } else if reversed.is_some() {
        let t = moving.len() as f64;
        (omega_u * under_samples + omega_o * t) / (under_samples + t)
    } else {
        let t = moving.len() as f64;
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
        let moved_km2 = moving.len() as f64 * norm.sample_area_km2();
        let e = events::event(world, t_myr, TERRANE_ACCRETION, q)
            .magnitude((moved_km2 / state.a0_km2) as f32)
            .param("from_plate", i64::from(from))
            .param("to_plate", i64::from(to))
            .param("area_km2", norm.planet_area_km2(moved_km2))
            .param("samples", moving.len() as i64)
            .param("uplift_m", belt.uplift_m)
            .param("coast_split_km", coast_km)
            .param("sutured", i64::from(suture))
            .caused_by(onset);
        let e = if transferred {
            e.param("transference", 1_i64)
        } else if reversed.is_some() {
            e.param("backarc_reversal", 1_i64)
        } else {
            e
        };
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
        let e = if transferred {
            e.param("transference", 1_i64)
        } else if reversed.is_some() {
            e.param("backarc_reversal", 1_i64)
        } else {
            e
        };
        let collision = world.emit(e)?;
        // What joined another plate: the terrane, or with the reversal the overriding block.
        let moved_km2 = moving.len() as f64 * norm.sample_area_km2();
        let e = events::event(world, t_myr, TERRANE_ACCRETION, q)
            .magnitude((moved_km2 / state.a0_km2) as f32)
            .param("from_plate", i64::from(from))
            .param("to_plate", i64::from(to))
            .param("area_km2", norm.planet_area_km2(moved_km2))
            .param("samples", moving.len() as i64)
            .caused_by(collision);
        world.emit(e)?;
        // The orogeny lasted the collision: from its onset to now, on the plate its belt is on.
        let belt_plate = if reversed.is_some() { under } else { over };
        let e = events::event(world, collision_onset_myr, OROGENY_PHASE, centroid)
            .until(t_myr * 1.0e6)
            .magnitude((belt.peak_height_m / 1000.0) as f32)
            .param("orogeny", "himalayan")
            .param("plate", i64::from(belt_plate))
            .param("uplift_m", belt.uplift_m)
            .param("belt_height_m", belt.peak_height_m)
            .caused_by(collision);
        world.emit(e)?;
        state.counters.collisions += 1;
        state.counters.accretions += 1;
        Some(collision)
    };
    // In the dynamics record a transference is the collision of a continent (`frac` of
    // `collision_suture_frac` or more) that did not suture: without it every such one welds.
    if transferred {
        state.counters.transferences += 1;
    }
    if reversed.is_some() {
        state.counters.backarc_reversals += 1;
    }
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
            belt: super::belts::footprint(&belt, col.as_ref().map_or(0.0, |c| c.peak_front_km)),
            under_born_myr: under_plate.born_myr,
            under_parent: under_plate.parent,
            peak_height_m: belt.peak_height_m,
            stacked_km2_m: belt.stacked_km2_m,
            stripped_km2_m: belt.stripped_km2_m,
            centroid: col.as_ref().map_or([0.0; 3], |c| c.centroid),
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
        moved: true,
    })
}

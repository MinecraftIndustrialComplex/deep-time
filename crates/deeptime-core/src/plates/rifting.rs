// SPDX-License-Identifier: LGPL-3.0-or-later
//! Plate rifting (Cortial et al. 2019, §4.4).
//!
//! Rifting is a Poisson process per plate and step: P = λ·e^(−λ), λ = λ_0 · f(x_P) · A/A_0, with
//! x_P the plate's continental fraction and f(x) = w + (1 − w)·x² (oceanic plates rift at the
//! small rate w; design choice), A the plate's area and A_0 the average initial plate area. A large
//! igneous province under the plate multiplies λ for a while (a plume head weakens the
//! lithosphere) and becomes the rift's cause.
//!
//! A rift splits the plate into n ∈ `rift_fragments` (Cortial: 2–4) sub-plates: n random, spread
//! centroids in the plate, noise-warped Voronoi cells around them (irregular fracture lines), the
//! largest cell keeping the parent's id. Each fragment gets copies of the parent's rigid groups
//! (the samples stay where they are) and the parent's motion: no speed is handed out (Cortial's
//! "random diverging directions" are gone with the switch to the force balance,
//! docs/design/plate-force-balance.md task 7). Whether the rift opens is the balance's answer: a
//! fragment with a slab on its far side is pulled away, and the sides of a rift that nothing pulls
//! apart move together and heal ([`heal`], design §3.4). Sample membership is only relabelled, so
//! the parent's area equals the sum of the fragments' areas exactly.

use std::collections::BTreeMap;

use glam::DVec3;
use rayon::prelude::*;

use super::Env;
use super::events::{self, PLATE_BIRTH, PLATE_DEATH, RIFT_ONSET};
use super::geometry::Tree;
use super::geometry::{Geo, coast_split, connect_plates};
use super::noise::fbm;
use super::state::{
    BackArc, CONTINENTAL, Group, HIDDEN, Plate, PlateState, RiftLink, Sample, SlabTrack, TRENCH,
};
use crate::error::CoreError;
use crate::sphere::kdtree::angle_of_chord2;
use crate::world::WorldState;
use deeptime_format::EventId;

/// Plates below this many samples never rift.
const MIN_RIFT_SAMPLES: u32 = 64;

/// The thinnest lithosphere `rift_weakness` weighs, km: a young ridge's or a back-arc's is thinner,
/// and weighs as this (game-choice: agent 2026-10-23, so that one edge costs at most
/// (225 / 20)^m times its length). ledger: PLT-506
const WEAK_MIN_KM: f64 = 20.0;

/// The stress weight of V1 gap 2's stage C2 (ledger PLT-504, PLT-505): each plate's
/// e^(sign·σ/σ_s) over its mean across the plates `candidates` names, weighted by the hazard `w`
/// each has unweighted, so the hazard's expected count over them is unchanged (to first order in
/// the hazard) and only where it falls moves.
/// σ is the plate's mean in-plane stress from the step's force balance (FRC-022, MPa, tension
/// positive); a plate without one (not in the balance, or born this step) weighs 1. `None` when
/// the rule is off (`scale_mpa` 0) or no candidate has a stress.
fn stress_weights(
    stress: &[Option<f64>],
    candidates: &[(u32, f64)],
    scale_mpa: f64,
    sign: f64,
) -> Option<BTreeMap<u32, f64>> {
    if scale_mpa.is_nan() || scale_mpa <= 0.0 {
        return None;
    }
    // Clamped so that one plate's extreme stress cannot overflow the weights.
    let raw = |id: u32| {
        stress
            .get(id as usize)
            .copied()
            .flatten()
            .map(|s| (sign * s / scale_mpa).clamp(-20.0, 20.0).exp())
    };
    let (mut num, mut den) = (0.0, 0.0);
    for &(id, a) in candidates {
        if let Some(x) = raw(id) {
            num += a * x;
            den += a;
        }
    }
    if !(den > 0.0 && num > 0.0) {
        return None;
    }
    let mean = num / den;
    Some(
        candidates
            .iter()
            .map(|&(id, _)| (id, raw(id).map_or(1.0, |x| x / mean)))
            .collect(),
    )
}

/// Draws a rift for every plate (in id order) and runs the ones drawn. `stress` is each plate's
/// mean in-plane stress from the step's force balance (by plate id), for `rift_stress_mpa`.
pub fn maybe_rift(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    stress: &[Option<f64>],
) -> Result<(), CoreError> {
    let k = env.k;
    let area = env.norm.sample_area_km2();
    let min = (k.min_plate_area_frac * f64::from(state.n) * 8.0).max(f64::from(MIN_RIFT_SAMPLES));
    let ids: Vec<u32> = state.alive().map(|p| p.id).collect();
    let a0 = state.a0_km2;
    // A plate's rift hazard λ (Cortial's: continental weight, area, plume boost).
    let hazard = |p: &Plate| {
        let x = p.continental_frac;
        let f = k.rift_ocean_weight + (1.0 - k.rift_ocean_weight) * x * x;
        let boost = if env.t_myr < p.lip_until_myr {
            k.lip_rift_boost
        } else {
            1.0
        };
        // ledger: PLT-082, PLT-083 (per Myr, so the step's λ_0 scales with its length)
        let lambda0 = k.rift_per_myr * env.dt_myr;
        lambda0 * f * f64::from(p.samples) * area / a0 * boost
    };
    // Rifts under tension (V1 gap 2's stage C2): the plates the edge forces pull apart rift
    // more, the others less, at the same expected count (the weights' mean is taken over the
    // plates' hazards). ledger: PLT-504
    let weights = if k.rift_stress_mpa > 0.0 {
        let candidates: Vec<(u32, f64)> = ids
            .iter()
            .map(|&id| &state.plates[id as usize])
            .filter(|p| f64::from(p.samples) >= min)
            .map(|p| (p.id, hazard(p)))
            .collect();
        stress_weights(stress, &candidates, k.rift_stress_mpa, 1.0)
    } else {
        None
    };
    for id in ids {
        let alive = state.alive().count() as u32;
        if alive >= k.max_plates {
            break;
        }
        let p = &state.plates[id as usize];
        if f64::from(p.samples) < min {
            continue;
        }
        let mut lambda = hazard(p);
        if let Some(w) = weights.as_ref().and_then(|w| w.get(&id)) {
            lambda *= w;
        }
        let prob = lambda * (-lambda).exp();
        if state.rng.next_f64() < prob {
            let fragments = state
                .rng
                .below(u64::from(k.rift_fragments[1] - k.rift_fragments[0]) + 1)
                as u32
                + k.rift_fragments[0];
            let fragments = fragments.min(k.max_plates - alive + 1);
            if fragments >= 2 {
                rift(state, world, geo, pos, env, id, fragments)?;
            }
        }
    }
    Ok(())
}

/// Splits plate `id` into `fragments` sub-plates.
pub fn rift(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    id: u32,
    fragments: u32,
) -> Result<(), CoreError> {
    let k = env.k;
    let norm = env.norm;
    let t = env.t_myr;
    let n = state.samples.len();
    let members: Vec<u32> = (0..n as u32)
        .filter(|&i| state.plate_of(i as usize) == id)
        .collect();
    let visible: Vec<u32> = members
        .iter()
        .copied()
        .filter(|&i| !state.samples[i as usize].is_hidden())
        .collect();
    if visible.len() < fragments as usize * 8 {
        return Ok(());
    }
    // Spread centroids: each new one the best of 16 random candidates.
    let mut centres: Vec<DVec3> = Vec::with_capacity(fragments as usize);
    let mut centre_at: Vec<u32> = Vec::with_capacity(fragments as usize);
    let first = visible[state.rng.below(visible.len() as u64) as usize];
    centres.push(pos[first as usize]);
    centre_at.push(first);
    while centres.len() < fragments as usize {
        let mut best = (f64::NEG_INFINITY, DVec3::Z, 0u32);
        for _ in 0..16 {
            let at = visible[state.rng.below(visible.len() as u64) as usize];
            let c = pos[at as usize];
            let d = centres
                .iter()
                .map(|e| e.angle_between(c))
                .fold(f64::INFINITY, f64::min);
            if d > best.0 {
                best = (d, c, at);
            }
        }
        centres.push(best.1);
        centre_at.push(best.2);
    }
    let seed = state.rng.next_u64();
    let warp = k.rift_warp_rad;
    let member = membership(&members, n);
    // Rifts along weak lithosphere (V1 gap 2's stage C2): each fragment grows from its centre
    // through the plate with every edge's cost its length times (L_cold / L)^m, L the
    // lithosphere's depth (`thermal.rs`), so the fragments meet where it is thin. Off (m = 0, or
    // no thermal state), the distance is the great circle's. ledger: PLT-506
    let grown: Option<Vec<Vec<f64>>> = (k.rift_weakness > 0.0 && k.thermal_state).then(|| {
        let depth = |i: u32| {
            let l = f64::from(state.samples[i as usize].lithosphere_km);
            if l > 0.0 {
                l.clamp(WEAK_MIN_KM, super::thermal::L_COLD_KM)
            } else {
                super::thermal::L_COLD_KM
            }
        };
        let factor = |v: u32, u: u32| {
            (super::thermal::L_COLD_KM / (0.5 * (depth(v) + depth(u)))).powf(k.rift_weakness)
        };
        centre_at
            .iter()
            .map(|&c| super::geometry::dijkstra_weighted(geo, c, |i| member[i as usize], factor))
            .collect()
    });
    // Label = fragment index; non-members keep a label no fragment uses.
    let mut label = vec![u32::MAX; n];
    for &i in &members {
        let p = pos[i as usize];
        // A sample some fragment reached through the plate goes by the grown distances (one no
        // fragment reached is beyond them all); one none reached (a part of the plate cut off on
        // the lattice) by the great circle to every centre.
        let reached = grown
            .as_ref()
            .is_some_and(|g| g.iter().any(|d| d[i as usize].is_finite()));
        label[i as usize] = (0..fragments)
            .map(|f| {
                let noise = warp * fbm(p, 4.0, 3, seed.wrapping_add(u64::from(f) * 31));
                let d = match grown.as_ref() {
                    Some(g) if reached => g[f as usize][i as usize],
                    _ => p.angle_between(centres[f as usize]),
                };
                (f, d + noise)
            })
            .fold((0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a })
            .0;
    }
    connect_plates(geo, &mut label, |i| member[i]);
    let split_coast = coast_split(geo, state, &members, |i| member[i], |i| label[i]);
    let mut counts = vec![0usize; fragments as usize];
    for &i in &members {
        counts[label[i as usize] as usize] += 1;
    }
    let order: Vec<u32> = {
        let mut o: Vec<u32> = (0..fragments).filter(|&f| counts[f as usize] > 0).collect();
        o.sort_by_key(|&f| (std::cmp::Reverse(counts[f as usize]), f));
        o
    };
    if order.len() < 2 {
        return Ok(());
    }
    // Plate ids: the largest fragment keeps the parent's.
    let parent = state.plates[id as usize].clone();
    let mut plate_of_fragment: BTreeMap<u32, u32> = BTreeMap::new();
    for (rank, &f) in order.iter().enumerate() {
        if rank == 0 {
            plate_of_fragment.insert(f, id);
        } else {
            let new_id = state.plates.len() as u32;
            state.plates.push(Plate {
                id: new_id,
                alive: true,
                omega: parent.omega,
                born_myr: t,
                died_myr: None,
                birth_event: None,
                parent: Some(id),
                lip_until_myr: f64::NEG_INFINITY,
                lip_event: None,
                samples: 0,
                centroid: parent.centroid,
                continental_frac: 0.0,
            });
            plate_of_fragment.insert(f, new_id);
        }
    }
    // Groups: fragments other than the first get twins of the parent's groups.
    let mut twin: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for &i in &members {
        let f = label[i as usize];
        let pid = plate_of_fragment[&f];
        if pid == id {
            continue;
        }
        let g = state.samples[i as usize].group;
        let new = *twin.entry((g, pid)).or_insert_with(|| {
            let rot = state.groups[g as usize].rot;
            state.groups.push(Group { plate: pid, rot });
            (state.groups.len() - 1) as u32
        });
        state.samples[i as usize].group = new;
    }
    // Centroids, continental fractions, and the rift line.
    let mut sum: BTreeMap<u32, (DVec3, usize, usize)> = BTreeMap::new();
    let mut line = DVec3::ZERO;
    for &i in &members {
        let pid = plate_of_fragment[&label[i as usize]];
        let e = sum.entry(pid).or_insert((DVec3::ZERO, 0, 0));
        e.0 += pos[i as usize];
        e.1 += 1;
        if state.samples[i as usize].kind == CONTINENTAL {
            e.2 += 1;
        }
        if geo
            .edges(i)
            .any(|(u, _)| label[u as usize] != u32::MAX && label[u as usize] != label[i as usize])
        {
            line += pos[i as usize];
        }
    }
    let total: DVec3 = sum.values().map(|v| v.0).sum();
    let line = line.normalize_or(total.normalize_or(DVec3::Z));
    let cont_frac = sum.values().map(|v| v.2).sum::<usize>() as f64 / members.len() as f64;
    let area_planet = norm.planet_area_km2(members.len() as f64 * norm.sample_area_km2());
    let mut e = events::event(world, t, RIFT_ONSET, line)
        .magnitude(cont_frac as f32)
        .param("plate", i64::from(id))
        .param("fragments", order.len() as i64)
        .param("continental_frac", cont_frac)
        .param("area_km2", area_planet)
        .param(
            "coast_split_km",
            norm.planet_len_km(split_coast as f64 * norm.km(norm.spacing_rad())),
        );
    if let Some(lip) = parent.lip_event.filter(|_| t < parent.lip_until_myr) {
        e = e.caused_by(lip);
    }
    let rift_event = world.emit(e)?;
    state.counters.rifts += 1;

    let omega = DVec3::from_array(parent.omega);
    let pids: Vec<u32> = order.iter().map(|f| plate_of_fragment[f]).collect();
    for &pid in &pids {
        let (s, count, cont) = sum[&pid];
        let c = s.normalize_or(line);
        let plate = &mut state.plates[pid as usize];
        plate.omega = omega.to_array();
        plate.samples = count as u32;
        plate.centroid = c.to_array();
        plate.continental_frac = cont as f64 / count as f64;
        if pid != id {
            let e = events::event(world, t, PLATE_BIRTH, c)
                .magnitude((count as f64 * norm.sample_area_km2() / state.a0_km2) as f32)
                .param("plate", i64::from(pid))
                .param("parent", i64::from(id))
                .param(
                    "area_km2",
                    norm.planet_area_km2(count as f64 * norm.sample_area_km2()),
                )
                .param("continental_frac", cont as f64 / count as f64)
                .param("speed_mm_yr", norm.speed_mm_yr(omega, c))
                .param("origin", "rift")
                .caused_by(rift_event);
            let birth = world.emit(e)?;
            state.plates[pid as usize].birth_event = Some(birth);
            state.counters.plate_births += 1;
        }
    }
    for (a, &pa) in pids.iter().enumerate() {
        for &pb in &pids[a + 1..] {
            state.rift_links.push(RiftLink {
                a: pa.min(pb),
                b: pa.max(pb),
                event: rift_event,
                quiet_since_myr: None,
            });
        }
    }
    // The fragments take the boundaries they border (ledger: PLT-534).
    if k.split_inheritance {
        let born: Vec<u32> = pids.iter().copied().filter(|&p| p != id).collect();
        super::inherit::inherit(state, pos, env, &[(id, born)]).book(world);
    }
    Ok(())
}

/// The world global summing fix C's trench length × step over the run, km × Myr: the realized
/// initiations' expectation is `subduction_initiation_per_trench_km_myr` × this (measurement,
/// ledger PLT-692).
pub const INITIATION_TRENCH_GLOBAL: &str = "plates/initiation_trench_km_myr";

/// Spontaneous subduction initiation at old oceanic lithosphere (design choice, beyond Cortial:
/// without it, oceanic crust on a plate with no trench ages for ever, while Earth's ocean floor is
/// recycled within about 200 Myr; Stern 2004, "Subduction initiation: spontaneous and induced").
/// Crust older than `subduction_initiation_age_myr` can start sinking where no subduction zone is
/// near: farther than `subduction_initiation_clearance_km` from every trench (P1-27: before, only
/// plates that subducted nowhere qualified, so once collisions welded plates together the old
/// floor of any plate with a trench somewhere aged for ever). Per plate and step, with
/// probability 1 − e^(−λ), λ = rate · (such old oceanic area)/A_0, a fracture separates the
/// plate's old oceanic crust around a random old sample from the rest (a warped Voronoi split
/// between that sample and the centroid of the rest; continental crust stays on the parent, so
/// continents are never torn, except slivers stranded inside the old floor, which ride the new
/// plate) and the old part, a new plate with the parent's motion, gets a slab of
/// `initiation_slab_km` hanging under the fracture (`PlateState::slabs`): the force balance pulls
/// it towards the rest, and the convergence opens a subduction zone at the fracture on the
/// following steps, where the older crust goes down (`subduction.rs`) and the slab grows. Emits a
/// plate_birth with `origin` "subduction_initiation".
///
/// With `initiation_near_trenches` (fix C, ledger PLT-690 to PLT-692; Crameri et al. 2020) the
/// active trenches are this step's subduction zones, `fronts` (`subduction::Pull::
/// subducting_fronts`: the overriding side's samples of the oceanic fronts past incipient, the
/// boundary the plate check measures against Bird 2003's subduction zones); the floor that
/// qualifies is the visible ocean floor within `subduction_initiation_reach_km` of one (not a
/// front or TRENCH sample itself), of any age, and the hazard is the planet's: λ summed over the
/// plates is `subduction_initiation_per_trench_km_myr` × the fronts' length (one lattice spacing
/// per front sample, as the plate check counts it) × the step, each plate's share its qualifying
/// floor (times PLT-505's weight). No front, no initiation. The samples flagged TRENCH (the
/// subducting plates' rims beside their hidden floor) are no measure of that length: they also
/// mark collisions, incipient and slow fronts and the rims of floor held under after its contact
/// ended, and on base20 summed to about twice the plate check's subducting boundary. The fracture
/// is the bisector of the seed and its foot on the nearest trench
/// (`trench_foot`): parallel to that trench at half the seed's distance (Crameri et al. 2020's
/// Table 1: ten of thirteen new trenches at a normal offset parallel to the nearest older trench,
/// three oblique, none along its strike). A draw whose fracture leaves fewer than 32 samples on a
/// side is counted (`initiations_refused`) and dropped.
pub fn maybe_initiate_subduction(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    stress: &[Option<f64>],
    fronts: &[u32],
) -> Result<(), CoreError> {
    let k = env.k;
    let near_trenches = k.initiation_near_trenches;
    let rate = if near_trenches {
        k.subduction_initiation_per_trench_km_myr
    } else {
        k.subduction_initiation_per_myr
    };
    if rate <= 0.0 {
        return Ok(());
    }
    let t = env.t_myr;
    let area = env.norm.sample_area_km2();
    // The trenches: with fix C this step's subduction zones (Crameri et al. 2020's "active
    // subduction zones", ledger PLT-691); else the samples flagged TRENCH (PLT-064's clearance).
    let trenches: Vec<DVec3> = if near_trenches {
        fronts.iter().map(|&v| pos[v as usize]).collect()
    } else {
        state
            .samples
            .iter()
            .enumerate()
            .filter(|(_, s)| s.flags & TRENCH != 0)
            .map(|(i, _)| pos[i])
            .collect()
    };
    let mut on_front = vec![false; state.samples.len()];
    if near_trenches {
        for &v in fronts {
            on_front[v as usize] = true;
        }
    }
    // Subduction breeds subduction (Crameri et al. 2020): with no trench, nothing starts.
    if near_trenches && trenches.is_empty() {
        return Ok(());
    }
    let trench_tree = (!trenches.is_empty()).then(|| Tree::over(&trenches, |_| true));
    // The angle to the nearest trench sample (infinite with none).
    let to_trench = |i: usize| {
        trench_tree
            .as_ref()
            .and_then(|tr| tr.nearest(pos[i]))
            .map_or(f64::INFINITY, |(_, d2)| angle_of_chord2(d2))
    };
    // No initiation within one lattice spacing of a front where along-strike propagation
    // carries subduction to the margins (Crameri et al. 2020 source both); else the old 1,000 km.
    // ledger: PLT-064, PLT-656
    let old_clearance = env.norm.rad(k.subduction_initiation_clearance_km);
    let clearance = if k.margin_propagation {
        env.norm.spacing_rad()
    } else {
        old_clearance
    };
    // An initiation the old clearance would have refused (counted, PLT-064).
    let near_front = |i: usize| to_trench(i) <= old_clearance;
    // Within reach of an active trench (fix C). ledger: PLT-691
    let reach = env.norm.rad(k.subduction_initiation_reach_km);
    // The fracture's other centre with fix C: the seed's foot on the nearest trench, so the
    // fracture runs parallel to that trench halfway to the seed (PLT-690); else the young floor's
    // centroid. The trench's line is fitted over three lattice spacings (numerical: a few samples
    // of trench, about 210 km on the quick presets' 100,000 samples, not a physical length).
    let stencil = 3.0 * env.norm.spacing_rad();
    let toward = |seed: u32| {
        trench_tree
            .as_ref()
            .filter(|_| near_trenches)
            .and_then(|tr| trench_foot(pos[seed as usize], &trenches, tr, stencil))
    };
    // Off the passive margin (V1 gap 2's stage C1): the reach of the arc the new trench builds
    // on the plate, and a lattice spacing for where the front falls. ledger: PLT-503
    let margin = if k.subduction_initiation_offshore {
        env.norm.rad(k.arc_distance_km + k.arc_width_km) + env.norm.spacing_rad()
    } else {
        0.0
    };
    // The floor that may go: old and clear of trenches (PLT-062, PLT-064), or with fix C any
    // floor within reach of one (Crameri et al. 2020: the subducting plates' mean ages at
    // initiation 31.7 to 117.4 Myr, and none spontaneous). ledger: PLT-690
    // A front or trench sample itself is no new trench's place (and the split below needs the
    // seed off the front).
    let qualifies = |i: usize, s: &Sample| {
        if near_trenches {
            s.flags & TRENCH == 0 && !on_front[i] && to_trench(i) <= reach
        } else {
            s.age_myr(t) > k.subduction_initiation_age_myr && to_trench(i) > clearance
        }
    };
    // Which samples qualify, in parallel (a trench query each; the speed work of 2026-10-28),
    // then gathered per plate in sample order.
    let ok: Vec<bool> = state
        .samples
        .par_iter()
        .enumerate()
        .map(|(i, s)| !s.is_hidden() && s.kind != CONTINENTAL && qualifies(i, s))
        .collect();
    let mut old: Vec<Vec<u32>> = vec![Vec::new(); state.plates.len()];
    for (i, _) in ok.iter().enumerate().filter(|(_, q)| **q) {
        old[state.plate_of(i) as usize].push(i as u32);
    }
    let ids: Vec<u32> = state.alive().map(|p| p.id).collect();
    // The hazard per qualifying sample and step. Fix C: the planet's expected initiations, rate ×
    // the subduction zones' length (one lattice spacing per front sample, Earth-normalized km, as
    // the plate check's `boundary_subduction_km` counts it against Bird 2003's 51,310 km) × the
    // step, over the qualifying floor of the plates that can be drawn (ledger: PLT-692); else
    // PLT-063, per Myr and A_0 of old floor.
    let per_sample = if near_trenches {
        let trench_km = trenches.len() as f64 * env.norm.km(env.norm.spacing_rad());
        let floor: usize = ids
            .iter()
            .map(|&id| old[id as usize].len())
            .filter(|&n| n >= 16)
            .sum();
        if floor == 0 {
            return Ok(());
        }
        // Measurement: the length the hazard read × the step, summed over the run, so the
        // realized initiations read against the rate × this sum (PLT-692).
        *world
            .globals
            .entry(INITIATION_TRENCH_GLOBAL.to_owned())
            .or_insert(0.0) += trench_km * env.dt_myr;
        Some(near_trench_hazard(rate, trench_km, env.dt_myr, floor))
    } else {
        None
    };
    // Subduction starts under compression (V1 gap 2's stage C2): the same weight with −σ, over
    // the plates' old floor. ledger: PLT-505
    let weights = if k.initiation_stress_mpa > 0.0 {
        let candidates: Vec<(u32, f64)> = ids
            .iter()
            .filter(|&&id| old[id as usize].len() >= 16)
            .map(|&id| (id, old[id as usize].len() as f64))
            .collect();
        stress_weights(stress, &candidates, k.initiation_stress_mpa, -1.0)
    } else {
        None
    };
    for id in ids {
        if state.alive().count() as u32 >= k.max_plates {
            break;
        }
        let o = &old[id as usize];
        if o.len() < 16 {
            continue;
        }
        // ledger: PLT-063, PLT-692 (per Myr, so the step's rate scales with its length)
        let mut lambda = match per_sample {
            Some(h) => h * o.len() as f64,
            None => rate * env.dt_myr * o.len() as f64 * area / state.a0_km2,
        };
        if let Some(w) = weights.as_ref().and_then(|w| w.get(&id)) {
            lambda *= w;
        }
        if state.rng.next_f64() < 1.0 - (-lambda).exp() {
            let n = state.plates.len();
            let seed = if margin > 0.0 {
                // The old floor within the margin of the plate's continental crust stays on the
                // plate, and the fracture's seed is old floor beyond it; none, no initiation.
                let near = near_continent(state, pos, id, margin);
                let far: Vec<u32> = o.iter().copied().filter(|&i| !near[i as usize]).collect();
                if far.is_empty() {
                    state.counters.initiations_refused += 1;
                    continue;
                }
                let seed = far[state.rng.below(far.len() as u64) as usize];
                initiate(
                    state,
                    world,
                    geo,
                    pos,
                    env,
                    id,
                    seed,
                    o,
                    Some(near.as_slice()),
                    toward(seed),
                )?;
                seed
            } else {
                let seed = o[state.rng.below(o.len() as u64) as usize];
                initiate(state, world, geo, pos, env, id, seed, o, None, toward(seed))?;
                seed
            };
            if state.plates.len() == n {
                // Drawn, but the fracture left too little on a side (`initiate`). Counted, so the
                // realized rate can be read against the hazard (PLT-692).
                state.counters.initiations_refused += 1;
            } else {
                if near_front(seed as usize) {
                    state.counters.initiations_near_fronts += 1;
                }
                // Where the new trench falls from the old ones, in Crameri et al. 2020's
                // distance classes (measurement, PLT-690).
                let d = env
                    .norm
                    .km(fracture_reach(state, geo, n as u32, id, &to_trench));
                let c = &mut state.counters;
                for (within, count) in [
                    (250.0, &mut c.initiations_reach_250),
                    (500.0, &mut c.initiations_reach_500),
                    (1000.0, &mut c.initiations_reach_1000),
                    (1500.0, &mut c.initiations_reach_1500),
                ] {
                    if d <= within {
                        *count += 1;
                    }
                }
            }
        }
    }
    Ok(())
}

/// The foot of `q` on the trench nearest it (fix C, ledger PLT-690): the trench samples within
/// `stencil` radians of the trench sample nearest `q` (`tree` is over `trenches`), fitted by
/// their principal axis in the gnomonic projection about that sample (where a great circle is a
/// straight line), and `q` dropped onto that line. The nearest sample alone would tilt the
/// fracture by the lattice's offset along the trench, up to about atan(spacing / 2d) for a seed d
/// from it. With one trench sample in the stencil, that sample.
fn trench_foot(q: DVec3, trenches: &[DVec3], tree: &Tree, stencil: f64) -> Option<DVec3> {
    let (j, _) = tree.nearest(q)?;
    let p0 = trenches[j as usize];
    let near = tree.within(p0, stencil);
    if near.len() < 2 || q.dot(p0) <= 0.0 {
        return Some(p0);
    }
    let e1 = p0.any_orthogonal_vector().normalize();
    let e2 = p0.cross(e1);
    let plane = |p: DVec3| {
        let g = p / p.dot(p0) - p0;
        (g.dot(e1), g.dot(e2))
    };
    let pts: Vec<(f64, f64)> = near
        .iter()
        .map(|&(i, _)| plane(trenches[i as usize]))
        .collect();
    let n = pts.len() as f64;
    let (mx, my) = pts
        .iter()
        .fold((0.0, 0.0), |(a, b), &(x, y)| (a + x / n, b + y / n));
    let (sxx, sxy, syy) = pts.iter().fold((0.0, 0.0, 0.0), |(a, b, c), &(x, y)| {
        let (dx, dy) = (x - mx, y - my);
        (a + dx * dx, b + dx * dy, c + dy * dy)
    });
    let theta = 0.5 * (2.0 * sxy).atan2(sxx - syy);
    let (tx, ty) = (theta.cos(), theta.sin());
    let (sx, sy) = plane(q);
    let along = (sx - mx) * tx + (sy - my) * ty;
    Some((p0 + (mx + along * tx) * e1 + (my + along * ty) * e2).normalize())
}

/// The planet's subduction-initiation hazard per qualifying sample and step with fix C (ledger
/// PLT-692): `per_km_myr` × the active trench length (km) × the step (Myr) expected initiations,
/// spread evenly over the `floor` qualifying samples of the plates that can be drawn.
fn near_trench_hazard(per_km_myr: f64, trench_km: f64, dt_myr: f64, floor: usize) -> f64 {
    per_km_myr * trench_km * dt_myr / floor as f64
}

/// The angle from a new plate's fracture to the nearest trench that was there before it
/// (`to_trench`): the least over the new plate's visible samples with a visible lattice neighbour
/// on its parent. Infinite with no fracture or no trench.
fn fracture_reach(
    state: &PlateState,
    geo: &Geo,
    new_id: u32,
    parent: u32,
    to_trench: &impl Fn(usize) -> f64,
) -> f64 {
    (0..state.samples.len())
        .filter(|&i| {
            state.plate_of(i) == new_id
                && !state.samples[i].is_hidden()
                && geo.edges(i as u32).any(|(u, _)| {
                    state.plate_of(u as usize) == parent && !state.samples[u as usize].is_hidden()
                })
        })
        .map(to_trench)
        .fold(f64::INFINITY, f64::min)
}

/// The plate's visible oceanic samples within `margin` (radians, great circle) of its visible
/// continental crust: where a subduction initiation's fracture may not run (ledger PLT-503).
fn near_continent(state: &PlateState, pos: &[DVec3], id: u32, margin: f64) -> Vec<bool> {
    let n = state.samples.len();
    let on = |i: usize| state.plate_of(i) == id && !state.samples[i].is_hidden();
    let land = |i: usize| on(i) && state.samples[i].kind == CONTINENTAL;
    if !(0..n).any(land) {
        return vec![false; n];
    }
    let tree = Tree::over(pos, land);
    (0..n)
        .into_par_iter()
        .map(|i| {
            on(i)
                && state.samples[i].kind != CONTINENTAL
                && tree
                    .nearest(pos[i])
                    .is_some_and(|(_, d2)| angle_of_chord2(d2) <= margin)
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn initiate(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    id: u32,
    seed: u32,
    old: &[u32],
    keep: Option<&[bool]>,
    toward: Option<DVec3>,
) -> Result<(), CoreError> {
    let k = env.k;
    let norm = env.norm;
    let t = env.t_myr;
    let n = state.samples.len();
    let members: Vec<u32> = (0..n as u32)
        .filter(|&i| state.plate_of(i as usize) == id)
        .collect();
    let member = membership(&members, n);
    let c_old = pos[seed as usize];
    // Fix C (ledger PLT-690): the fracture is the bisector of the seed and its nearest trench,
    // parallel to that trench halfway to the seed (Crameri et al. 2020's Table 1: ten of
    // thirteen new trenches parallel to the old one at a normal offset, none along its strike).
    let c_young = if let Some(p) = toward {
        p
    } else {
        let mut is_old = vec![false; n];
        for &i in old {
            is_old[i as usize] = true;
        }
        let young: DVec3 = members
            .iter()
            .filter(|&&i| !is_old[i as usize] && !state.samples[i as usize].is_hidden())
            .map(|&i| pos[i as usize])
            .sum();
        if young.length_squared() > 1e-12 {
            young.normalize()
        } else {
            // All old: split off the half around the seed, away from the farthest point.
            members
                .iter()
                .map(|&i| pos[i as usize])
                .max_by(|a, b| c_old.angle_between(*a).total_cmp(&c_old.angle_between(*b)))
                .unwrap_or(-c_old)
        }
    };
    let noise_seed = state.rng.next_u64();
    let warp = k.rift_warp_rad;
    let mut label = vec![u32::MAX; n];
    for &i in &members {
        let p = pos[i as usize];
        let da = p.angle_between(c_old) + warp * fbm(p, 4.0, 3, noise_seed);
        let db = p.angle_between(c_young) + warp * fbm(p, 4.0, 3, noise_seed ^ 0x55);
        label[i as usize] = u32::from(
            da < db
                && state.samples[i as usize].kind != CONTINENTAL
                && !keep.is_some_and(|kp| kp[i as usize]),
        );
    }
    connect_plates(geo, &mut label, |i| member[i]);
    let split_coast = coast_split(geo, state, &members, |i| member[i], |i| label[i]);
    let split: Vec<u32> = members
        .iter()
        .copied()
        .filter(|&i| label[i as usize] == 1)
        .collect();
    if split.len() < 32 || members.len() - split.len() < 32 {
        return Ok(());
    }
    // Pieces of the young side cut off by the old floor ride with it: a sliver of continent may,
    // but a split that is mostly continent is no sinking ocean floor (2026-10-02: with the
    // resampling fix, a test history's initiation took a plate 0.89 continental, probably a
    // continent the old floor cut off, which `connect_plates` hands to the split). ledger: PLT-229
    let cont_split = split
        .iter()
        .filter(|&&i| state.samples[i as usize].kind == CONTINENTAL)
        .count();
    if 2 * cont_split >= split.len() {
        return Ok(());
    }
    // The older side of the fracture goes under (ledger: PLT-655); off, the split.
    let split_under = !k.initiation_polarity_by_age
        || split_goes_under(geo, state, &members, |i| member[i], |i| label[i], t);
    let new_id = state.plates.len() as u32;
    let parent = state.plates[id as usize].clone();
    let mut twin: BTreeMap<u32, u32> = BTreeMap::new();
    let (mut c, mut ages, mut oceanic) = (DVec3::ZERO, 0.0, 0usize);
    for &i in &split {
        let g = state.samples[i as usize].group;
        let new = *twin.entry(g).or_insert_with(|| {
            let rot = state.groups[g as usize].rot;
            state.groups.push(Group { plate: new_id, rot });
            (state.groups.len() - 1) as u32
        });
        let s = &mut state.samples[i as usize];
        s.group = new;
        c += pos[i as usize];
        if s.kind != CONTINENTAL {
            ages += s.age_myr(t);
            oceanic += 1;
        }
    }
    let c = c.normalize_or(c_old);
    let omega = DVec3::from_array(parent.omega);
    // The planted slab (design §3.3): the old floor starts to sink under the fracture, on its
    // older side (PLT-655). ledger: PLT-270
    let (under, over) = if split_under {
        (new_id, id)
    } else {
        (id, new_id)
    };
    state.slabs.push(SlabTrack {
        under,
        over,
        km: k.initiation_slab_km,
        ..SlabTrack::default()
    });
    let count = split.len();
    let cont = count - oceanic;
    state.plates.push(Plate {
        id: new_id,
        alive: true,
        omega: omega.to_array(),
        born_myr: t,
        died_myr: None,
        birth_event: None,
        parent: Some(id),
        lip_until_myr: f64::NEG_INFINITY,
        lip_event: None,
        samples: count as u32,
        centroid: c.to_array(),
        continental_frac: cont as f64 / count as f64,
    });
    // From this split's own membership: the plate's count may be stale (a suture this step).
    state.plates[id as usize].samples = (members.len() - count) as u32;
    let mean_age = if oceanic > 0 {
        ages / oceanic as f64
    } else {
        0.0
    };
    let e = events::event(world, t, PLATE_BIRTH, c)
        .magnitude((count as f64 * norm.sample_area_km2() / state.a0_km2) as f32)
        .param("plate", i64::from(new_id))
        .param("parent", i64::from(id))
        .param("origin", "subduction_initiation")
        .param(
            "area_km2",
            norm.planet_area_km2(count as f64 * norm.sample_area_km2()),
        )
        .param("mean_ocean_age_myr", mean_age)
        .param("continental_frac", cont as f64 / count as f64)
        .param("speed_mm_yr", norm.speed_mm_yr(omega, c))
        .param(
            "coast_split_km",
            norm.planet_len_km(split_coast as f64 * norm.km(norm.spacing_rad())),
        );
    let birth = world.emit(e)?;
    state.plates[new_id as usize].birth_event = Some(birth);
    state.counters.plate_births += 1;
    state.counters.initiations += 1;
    // The sinking plate takes the boundaries it borders (ledger: PLT-534); the slab planted
    // under the fracture is inside the split and stays.
    if k.split_inheritance {
        super::inherit::inherit(state, pos, env, &[(id, vec![new_id])]).book(world);
    }
    Ok(())
}

/// Whether the split's side of a subduction initiation's fracture goes under (ledger: PLT-655;
/// Gurnis, Hall & Lavier 2004 [60]: at a fracture zone "the older plate is thrust beneath the
/// younger"). The fracture's samples are the visible samples of `members` with a lattice
/// neighbour of the same rigid group, visible and a member, on the other side (`label`, 1 the
/// split, 0 the parent): the nearest floor the lattice resolves, one spacing each side (about
/// 180 km on the quick lattice; Gurnis et al.'s older plate underthrusts 100 to 150 km before
/// subduction sustains itself). As the rule of a new contact (`subduction::resolve`)
/// reads an overlap: a side whose fracture is mostly continental crust does not go under (both,
/// the split); otherwise the side with the older mean ocean floor does, the split on a tie or
/// with no fracture on either side.
fn split_goes_under(
    geo: &Geo,
    state: &PlateState,
    members: &[u32],
    member: impl Fn(usize) -> bool,
    label: impl Fn(usize) -> u32,
    t: f64,
) -> bool {
    let (mut seen, mut cont, mut age) = ([0u32; 2], [0u32; 2], [0.0f64; 2]);
    for &i in members {
        let s = &state.samples[i as usize];
        if s.is_hidden() {
            continue;
        }
        let own = label(i as usize);
        let on_fracture = geo.edges(i).any(|(u, _)| {
            let o = &state.samples[u as usize];
            member(u as usize) && !o.is_hidden() && o.group == s.group && label(u as usize) != own
        });
        if !on_fracture {
            continue;
        }
        let side = usize::from(own == 1);
        seen[side] += 1;
        if s.kind == CONTINENTAL {
            cont[side] += 1;
        } else {
            age[side] += s.age_myr(t);
        }
    }
    let continental = |s: usize| 2 * cont[s] >= seen[s].max(1);
    let mean_age = |s: usize| {
        let oceanic = seen[s] - cont[s];
        if oceanic == 0 {
            0.0
        } else {
            age[s] / f64::from(oceanic)
        }
    };
    match (continental(1), continental(0)) {
        (false, true) | (true, true) => true,
        (true, false) => false,
        (false, false) => seen[0] == 0 || seen[1] == 0 || mean_age(1) >= mean_age(0),
    }
}

fn membership(members: &[u32], n: usize) -> Vec<bool> {
    let mut m = vec![false; n];
    for &i in members {
        m[i as usize] = true;
    }
    m
}

/// The plate_birth and rift_onset `origin` of a back-arc rift (ledger PLT-710).
pub const BACKARC_ORIGIN: &str = "back_arc";

/// The youngest floor coming in at a front where a back-arc rifts, Myr (Sdrolias & Müller
/// 2006: "back-arc basins develop when the age of subducting normal oceanic lithosphere is
/// greater than 55 million years"). ledger: PLT-712
pub const BACKARC_SLAB_AGE_MYR: f64 = 55.0;

/// The shortest slab under a back-arc rift, km (Karig 1971: "Active extension is restricted to
/// arc systems with earthquakes deeper than about 350 km"). The slab's length stands for its
/// depth, as the slab pull reads it against PLT-184's 700 km. ledger: PLT-713
pub const BACKARC_SLAB_KM: f64 = 350.0;

/// The fewest samples a back-arc rift leaves on each side: a subduction initiation's 32
/// (`initiate`; the lattice's resolution, game-choice). ledger: PLT-714
const BACKARC_MIN_SAMPLES: usize = 32;

/// World globals of back-arc rifting and the trench's motion (measurement, written only with
/// `tectonics.backarc_rifting` on; ledger PLT-716). Over the step's subducting fronts
/// (`subduction::Pull::subducting_fronts`, one lattice spacing each, as PLT-692 counts them), km ×
/// Myr …
pub const TRENCH_FRONT_GLOBAL: &str = "plates/trench_front_km_myr";
/// … that length × the trench's velocity normal to the front, seaward positive (retreat, Heuret
/// & Lallemand 2005's sign; the trench moves with the plate that holds the front), mm/yr × km × Myr …
pub const TRENCH_VT_GLOBAL: &str = "plates/trench_vt_mm_yr_km_myr";
/// … that length × the convergence normal to the front (the subduction rate, Schellart et al.
/// 2007's vS + vT) …
pub const TRENCH_VC_GLOBAL: &str = "plates/trench_vc_mm_yr_km_myr";
/// … the length whose trench retreats (Vt > 0) …
pub const TRENCH_RETREAT_GLOBAL: &str = "plates/trench_retreat_km_myr";
/// … the length whose |Vt| is 50 mm/yr or less …
pub const TRENCH_WITHIN50_GLOBAL: &str = "plates/trench_within50_km_myr";
/// … the length back-arc slivers hold …
pub const BACKARC_FRONT_GLOBAL: &str = "plates/backarc_front_km_myr";
/// … the length that qualifies for a back-arc rift and no sliver holds …
pub const BACKARC_QUALIFYING_GLOBAL: &str = "plates/backarc_qualifying_km_myr";
/// … the slivers' fronts whose parent lives …
pub const BACKARC_VD_KM_GLOBAL: &str = "plates/backarc_vd_km_myr";
/// … and that length × the back-arc's opening normal to the front (the sliver's velocity less
/// its parent's, opening positive; Heuret & Lallemand's Vd with the sign turned).
pub const BACKARC_VD_GLOBAL: &str = "plates/backarc_vd_mm_yr_km_myr";

/// Back-arc rifting (`tectonics.backarc_rifting`, ledger PLT-710 to PLT-716): the moving
/// trench. In a model of rigid plates whose fronts are fixed in the overriding plate, a trench
/// moves only with that plate; Earth's trenches roll back where the overriding plate splits and a
/// back-arc basin opens behind its forearc (Heuret & Lallemand 2005: Vt = Vup + Vd).
///
/// `fronts` and `detail` are the step's subducting fronts (`subduction::Pull`): the overriding
/// side's samples past incipient, each with the subducting side's sample across it and the
/// boundary normal towards it. A front qualifies where the floor coming in is older than 55 Myr
/// (PLT-712), the plate holding the front moves away from it (its velocity along the normal
/// below 0: Sdrolias & Müller 2006, "The initiation of back-arc extension is preceded by the
/// landward migration of the overriding plate, in an absolute reference frame"; here the
/// mantle's frame), the pair's slab is 350 km or longer (PLT-713), and no back-arc sliver holds
/// it (once open, "back-arc formation will continue unabated even if the overriding plate begins
/// to move toward the trench"). Each pair of plates with a qualifying front draws once a step
/// with probability 1 − e^(−λ), λ = `backarc_rifting_per_myr` × dt (PLT-711); a draw splits off
/// the overriding plate's samples within `arc_distance_km` of those fronts, its forearc and arc,
/// the largest connected strip (PLT-714), as a new plate with the parent's motion. The sliver
/// takes the front, the slab and their suction (PLT-534's inheritance); the force balance decides
/// whether it opens (the rest of the plate no longer feels that suction), a rift link lets a
/// back-arc that does not open heal (PLT-272), and the sliver is kept below PLT-091's size while
/// it holds a subducting front (PLT-715). Emits a rift_onset and a plate_birth with `origin`
/// "back_arc". Also measures the trench's motion over every subducting front (PLT-716).
#[allow(clippy::too_many_arguments)]
pub fn maybe_rift_backarc(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    fronts: &[u32],
    detail: &[(u32, DVec3)],
) -> Result<(), CoreError> {
    let k = env.k;
    if !k.backarc_rifting {
        state.backarcs.clear();
        return Ok(());
    }
    let t = env.t_myr;
    let norm = env.norm;
    // One lattice spacing per front sample × the step (PLT-692's count).
    let km_myr = norm.km(norm.spacing_rad()) * env.dt_myr;
    // A sliver keeps its exemption while it holds a subducting front (PLT-715).
    let holders: std::collections::BTreeSet<u32> =
        fronts.iter().map(|&v| state.plate_of(v as usize)).collect();
    let plates = &state.plates;
    state.backarcs.retain(|b| {
        plates.get(b.plate as usize).is_some_and(|p| p.alive) && holders.contains(&b.plate)
    });
    let parent_of: BTreeMap<u32, u32> =
        state.backarcs.iter().map(|b| (b.plate, b.parent)).collect();
    let slab_km: BTreeMap<(u32, u32), f64> = state
        .slabs
        .iter()
        .filter(|s| !s.latent)
        .map(|s| ((s.under, s.over), s.km))
        .collect();
    let omega = |p: u32| DVec3::from_array(state.plates[p as usize].omega);
    let alive = |p: u32| state.plates.get(p as usize).is_some_and(|q| q.alive);
    let mut sums = [0.0_f64; 9];
    let mut qualifying: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (&v, &(u, n)) in fronts.iter().zip(detail) {
        let x = pos[v as usize];
        let o = state.plate_of(v as usize);
        let up = state.plate_of(u as usize);
        let v_o = norm.velocity_mm_yr(omega(o), x);
        // The trench moves with the plate that holds the front: Vt is its velocity along the
        // normal towards the subducting plate, seaward positive (Heuret & Lallemand 2005).
        let vt = v_o.dot(n);
        let vc = (v_o - norm.velocity_mm_yr(omega(up), x)).dot(n);
        sums[0] += km_myr;
        sums[1] += vt * km_myr;
        sums[2] += vc * km_myr;
        if vt > 0.0 {
            sums[3] += km_myr;
        }
        if vt.abs() <= 50.0 {
            sums[4] += km_myr;
        }
        if let Some(&parent) = parent_of.get(&o) {
            sums[5] += km_myr;
            if alive(parent) {
                sums[7] += km_myr;
                sums[8] += (v_o - norm.velocity_mm_yr(omega(parent), x)).dot(n) * km_myr;
            }
            continue;
        }
        // ledger: PLT-712, PLT-713
        let old = state.samples[u as usize].age_myr(t) > BACKARC_SLAB_AGE_MYR;
        let deep = slab_km
            .get(&(up, o))
            .is_some_and(|&km| km >= BACKARC_SLAB_KM);
        if up != o && old && deep && vt < 0.0 {
            sums[6] += km_myr;
            qualifying.entry((up, o)).or_default().push(v);
        }
    }
    for (key, x) in [
        TRENCH_FRONT_GLOBAL,
        TRENCH_VT_GLOBAL,
        TRENCH_VC_GLOBAL,
        TRENCH_RETREAT_GLOBAL,
        TRENCH_WITHIN50_GLOBAL,
        BACKARC_FRONT_GLOBAL,
        BACKARC_QUALIFYING_GLOBAL,
        BACKARC_VD_KM_GLOBAL,
        BACKARC_VD_GLOBAL,
    ]
    .into_iter()
    .zip(sums)
    {
        *world.globals.entry(key.to_owned()).or_insert(0.0) += x;
    }
    // ledger: PLT-711 (per Myr, so the step's hazard scales with its length)
    let lambda = k.backarc_rifting_per_myr * env.dt_myr;
    for ((_, o), at) in qualifying {
        if state.alive().count() as u32 >= k.max_plates {
            break;
        }
        if state.rng.next_f64() < 1.0 - (-lambda).exp() {
            rift_backarc(state, world, geo, pos, env, o, &at)?;
        }
    }
    Ok(())
}

/// Splits plate `id`'s samples within `arc_distance_km` of the fronts `at` off as a back-arc
/// sliver (ledger PLT-710, PLT-714): the forearc and the arc, where Taylor & Karner 1983 find
/// the arc rifting ("rifting of the volcanic arc may occur on either side of the line of active
/// volcanoes (±50 km)") and Sdrolias & Müller 2006 the locus "commonly in between the remnant and
/// active volcanic arc"; the arc stands `arc_distance_km` behind its trench (PLT-042). The
/// largest connected strip is the sliver; under 32 samples on either side, the draw is refused.
#[allow(clippy::too_many_arguments)]
pub(super) fn rift_backarc(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    id: u32,
    at: &[u32],
) -> Result<(), CoreError> {
    let k = env.k;
    let norm = env.norm;
    let t = env.t_myr;
    let n = state.samples.len();
    // A split earlier this step may have taken some of these fronts.
    let at: Vec<DVec3> = at
        .iter()
        .filter(|&&v| state.plate_of(v as usize) == id)
        .map(|&v| pos[v as usize])
        .collect();
    if at.is_empty() {
        return Ok(());
    }
    let members: Vec<u32> = (0..n as u32)
        .filter(|&i| state.plate_of(i as usize) == id)
        .collect();
    let member = membership(&members, n);
    let tree = Tree::over(&at, |_| true);
    let reach = norm.rad(k.arc_distance_km); // ledger: PLT-042, PLT-714
    let mut label = vec![u32::MAX; n];
    for &i in &members {
        let near = tree
            .nearest(pos[i as usize])
            .is_some_and(|(_, d2)| angle_of_chord2(d2) <= reach);
        label[i as usize] = u32::from(near);
    }
    connect_plates(geo, &mut label, |i| member[i]);
    let split: Vec<u32> = members
        .iter()
        .copied()
        .filter(|&i| label[i as usize] == 1)
        .collect();
    if split.len() < BACKARC_MIN_SAMPLES || members.len() - split.len() < BACKARC_MIN_SAMPLES {
        state.counters.backarc_refused += 1;
        return Ok(());
    }
    let split_coast = coast_split(geo, state, &members, |i| member[i], |i| label[i]);
    let new_id = state.plates.len() as u32;
    let parent = state.plates[id as usize].clone();
    let mut twin: BTreeMap<u32, u32> = BTreeMap::new();
    let (mut c, mut cont) = (DVec3::ZERO, 0usize);
    for &i in &split {
        let g = state.samples[i as usize].group;
        let new = *twin.entry(g).or_insert_with(|| {
            let rot = state.groups[g as usize].rot;
            state.groups.push(Group { plate: new_id, rot });
            (state.groups.len() - 1) as u32
        });
        let s = &mut state.samples[i as usize];
        s.group = new;
        c += pos[i as usize];
        cont += usize::from(s.kind == CONTINENTAL);
    }
    let line: DVec3 = at.iter().copied().sum();
    let c = c.normalize_or(line.normalize_or(DVec3::Z));
    let omega = DVec3::from_array(parent.omega);
    let count = split.len();
    state.plates.push(Plate {
        id: new_id,
        alive: true,
        omega: omega.to_array(),
        born_myr: t,
        died_myr: None,
        birth_event: None,
        parent: Some(id),
        lip_until_myr: f64::NEG_INFINITY,
        lip_event: None,
        samples: count as u32,
        centroid: c.to_array(),
        continental_frac: cont as f64 / count as f64,
    });
    state.plates[id as usize].samples = (members.len() - count) as u32;
    let area_km2 = norm.planet_area_km2(count as f64 * norm.sample_area_km2());
    let rift = events::event(world, t, RIFT_ONSET, c)
        .magnitude((cont as f64 / count as f64) as f32)
        .param("plate", i64::from(id))
        .param("fragments", 2_i64)
        .param("origin", BACKARC_ORIGIN)
        .param("continental_frac", cont as f64 / count as f64)
        .param("area_km2", area_km2)
        .param(
            "coast_split_km",
            norm.planet_len_km(split_coast as f64 * norm.km(norm.spacing_rad())),
        );
    let rift_event = world.emit(rift)?;
    let e = events::event(world, t, PLATE_BIRTH, c)
        .magnitude((count as f64 * norm.sample_area_km2() / state.a0_km2) as f32)
        .param("plate", i64::from(new_id))
        .param("parent", i64::from(id))
        .param("origin", BACKARC_ORIGIN)
        .param("area_km2", area_km2)
        .param("continental_frac", cont as f64 / count as f64)
        .param("speed_mm_yr", norm.speed_mm_yr(omega, c))
        .param(
            "front_km",
            norm.planet_len_km(at.len() as f64 * norm.km(norm.spacing_rad())),
        )
        .caused_by(rift_event);
    let birth = world.emit(e)?;
    state.plates[new_id as usize].birth_event = Some(birth);
    state.counters.plate_births += 1;
    state.counters.backarc_rifts += 1;
    // A back-arc that does not open heals (PLT-272).
    state.rift_links.push(RiftLink {
        a: id.min(new_id),
        b: id.max(new_id),
        event: rift_event,
        quiet_since_myr: None,
    });
    state.backarcs.push(BackArc {
        plate: new_id,
        parent: id,
        born_myr: t,
    });
    // The sliver takes the front, the contact and the slab it borders (ledger: PLT-534).
    if k.split_inheritance {
        super::inherit::inherit(state, pos, env, &[(id, vec![new_id])]).book(world);
    }
    Ok(())
}

/// Heals the rifts that did not open (design `docs/design/plate-force-balance.md` §3.4: two plates
/// that have stopped moving relative to each other are one plate). Two plates a rift separated
/// (`PlateState::rift_links`) whose relative speed has stayed under `weld_speed_mm_yr` all along
/// their shared boundary for `weld_after_myr` become one again: the larger keeps its id, the
/// motion is the area-weighted mean of the two, and the other dies (`plate_death`, fate "healed",
/// caused by the rift). Plates that overlap anywhere (a contact seen this step) or share no
/// boundary do not heal and their clock restarts. Returns the plates that healed and the plates
/// they healed into. Ledger PLT-272.
pub fn heal(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
) -> Result<Vec<u32>, CoreError> {
    let k = env.k;
    let t = env.t_myr;
    if state.rift_links.is_empty() {
        return Ok(Vec::new());
    }
    let alive = |st: &PlateState, p: u32| st.plates.get(p as usize).is_some_and(|q| q.alive);
    let living: Vec<bool> = state.plates.iter().map(|p| p.alive).collect();
    let lives = |p: u32| living.get(p as usize).copied().unwrap_or(false);
    state.rift_links.retain(|l| lives(l.a) && lives(l.b));
    if state.rift_links.is_empty() {
        return Ok(Vec::new());
    }
    // The fastest relative speed along each linked pair's shared boundary (P1-19: in parallel
    // over the samples; a maximum does not depend on the order it is taken in).
    let linked: std::collections::BTreeSet<(u32, u32)> =
        state.rift_links.iter().map(|l| (l.a, l.b)).collect();
    let (samples, groups, plates) = (&state.samples, &state.groups, &state.plates);
    let plate_of = |i: usize| groups[samples[i].group as usize].plate;
    let fastest: BTreeMap<(u32, u32), f64> = samples
        .par_iter()
        .zip(pos)
        .enumerate()
        .fold(
            BTreeMap::<(u32, u32), f64>::new,
            |mut fastest, (v, (sv, &x))| {
                if sv.flags & HIDDEN != 0 {
                    return fastest;
                }
                let pv = plate_of(v);
                for (u, _) in geo.edges(v as u32) {
                    let pu = plate_of(u as usize);
                    if pu <= pv || samples[u as usize].flags & HIDDEN != 0 {
                        continue;
                    }
                    let key = (pv, pu);
                    if !linked.contains(&key) {
                        continue;
                    }
                    let w = DVec3::from_array(plates[pu as usize].omega)
                        - DVec3::from_array(plates[pv as usize].omega);
                    let speed = env.norm.speed_mm_yr(w, x);
                    let e = fastest.entry(key).or_insert(0.0);
                    *e = e.max(speed);
                }
                fastest
            },
        )
        .reduce(BTreeMap::new, |mut a, b| {
            for (key, speed) in b {
                let e = a.entry(key).or_insert(0.0);
                *e = e.max(speed);
            }
            a
        });
    let touching: std::collections::BTreeSet<(u32, u32)> = state
        .contacts
        .iter()
        .filter(|c| c.carried || c.last_step + 1 >= env.step) // ledger: PLT-550
        .map(|c| (c.under.min(c.over), c.under.max(c.over)))
        .collect();
    let mut due: Vec<(u32, u32, EventId)> = Vec::new();
    for l in &mut state.rift_links {
        let key = (l.a, l.b);
        let quiet =
            !touching.contains(&key) && fastest.get(&key).is_some_and(|&v| v < k.weld_speed_mm_yr); // ledger: PLT-272, PLT-246
        if quiet {
            let since = *l.quiet_since_myr.get_or_insert(t - env.dt_myr);
            if t - since >= k.weld_after_myr - 1e-9 {
                due.push((l.a, l.b, l.event));
            }
        } else {
            l.quiet_since_myr = None;
        }
    }
    let mut healed = Vec::new();
    for (a, b, rift) in due {
        if !alive(state, a) || !alive(state, b) {
            continue;
        }
        let (na, nb) = (
            f64::from(state.plates[a as usize].samples),
            f64::from(state.plates[b as usize].samples),
        );
        let (keep, die) = if nb > na { (b, a) } else { (a, b) };
        for g in state.groups.iter_mut().filter(|g| g.plate == die) {
            g.plate = keep;
        }
        let (wk, wd) = (
            DVec3::from_array(state.plates[keep as usize].omega),
            DVec3::from_array(state.plates[die as usize].omega),
        );
        let (sk, sd) = (
            f64::from(state.plates[keep as usize].samples),
            f64::from(state.plates[die as usize].samples),
        );
        state.plates[keep as usize].omega = ((wk * sk + wd * sd) / (sk + sd).max(1.0)).to_array();
        let at = DVec3::from_array(state.plates[die as usize].centroid);
        let e = events::event(world, t, PLATE_DEATH, at)
            .param("plate", i64::from(die))
            .param("fate", "healed")
            .param("absorbed_by", i64::from(keep))
            .caused_by(rift);
        world.emit(e)?;
        let dead = &mut state.plates[die as usize];
        dead.alive = false;
        dead.died_myr = Some(t);
        dead.samples = 0;
        state.plates[keep as usize].samples = (sk + sd) as u32;
        state.counters.plate_deaths += 1;
        state.counters.healed += 1;
        state.rift_links.retain(|l| l.a != die && l.b != die);
        healed.extend([keep, die]);
    }
    if !healed.is_empty() {
        state.recount(pos);
    }
    Ok(healed)
}

#[cfg(test)]
pub(in crate::plates) mod tests {
    use super::*;
    use crate::params::TectonicParams;
    use crate::plates::geometry::positions;
    use crate::plates::normalise::Norm;
    use crate::plates::pools::{continental_column, oceanic_column};
    use crate::plates::state::Sample;

    #[test]
    fn stress_weights_keep_the_mean_and_favour_the_stress_asked_for() {
        let stress = vec![Some(40.0), Some(0.0), Some(-20.0), None];
        let cands = [(0, 2.0), (1, 1.0), (2, 1.0), (3, 5.0)];
        let w = stress_weights(&stress, &cands, 20.0, 1.0).unwrap();
        // The area-weighted mean over the plates with a stress is 1; one without weighs 1.
        let mean = (2.0 * w[&0] + w[&1] + w[&2]) / 4.0;
        assert!((mean - 1.0).abs() < 1e-12, "{mean}");
        assert_eq!(w[&3], 1.0);
        assert!(w[&0] > w[&1] && w[&1] > w[&2]);
        // Compression favoured with the sign turned.
        let c = stress_weights(&stress, &cands, 20.0, -1.0).unwrap();
        assert!(c[&2] > c[&1] && c[&1] > c[&0]);
        // Off, or no stress at all: no weights.
        assert!(stress_weights(&stress, &cands, 0.0, 1.0).is_none());
        assert!(stress_weights(&[None, None], &[(0, 1.0), (1, 1.0)], 20.0, 1.0).is_none());
    }

    pub(in crate::plates) const N: usize = 16_000;

    fn env(k: &TectonicParams) -> Env<'_> {
        Env {
            k,
            norm: Norm::new(6370.0, 6371.0, N),
            dt_myr: 2.0,
            dt_yr: 2.0e6,
            gravity: 9.81,
            step: 1,
            t_myr: 2.0,
        }
    }

    /// One plate over the whole sphere, each sample as `make` says from its position.
    pub(in crate::plates) fn one_plate(
        geo: &Geo,
        seed: u64,
        make: impl Fn(DVec3) -> Sample,
    ) -> PlateState {
        let ident = [0.0, 0.0, 0.0, 1.0];
        let mut st = PlateState {
            n: N as u32,
            lattice: ident,
            samples: geo.points().iter().map(|&p| make(p)).collect(),
            groups: vec![Group {
                plate: 0,
                rot: ident,
            }],
            plates: vec![Plate {
                id: 0,
                alive: true,
                omega: [0.0; 3],
                born_myr: 0.0,
                died_myr: None,
                birth_event: None,
                parent: None,
                lip_until_myr: f64::NEG_INFINITY,
                lip_event: None,
                samples: 0,
                centroid: [0.0, 0.0, 1.0],
                continental_frac: 0.0,
            }],
            contacts: Vec::new(),
            hotspots: Vec::new(),
            ridges: Vec::new(),
            rift_links: Vec::new(),
            rng: crate::sphere::SplitMix64::new(seed),
            step: 0,
            t_myr: 0.0,
            start_myr: 0.0,
            resample_step: 0,
            resample_myr: 0.0,
            a0_km2: 1.0,
            counters: Default::default(),
            slabs: Vec::new(),
            dynamics: None,
            gap_budget: None,
            limits: None,
            shown: None,
            one_way: None,
            propagations: Vec::new(),
            backarcs: Vec::new(),
            converging: Vec::new(),
            level_set: None,
        };
        let pos = positions(&st, geo);
        st.recount(&pos);
        st
    }

    pub(in crate::plates) fn continental() -> Sample {
        let mut s = Sample::blank(0);
        s.kind = CONTINENTAL;
        s.pools = continental_column();
        s.thickness_m = s.pools.iter().sum();
        s.elevation_m = 300.0;
        s
    }

    /// Ocean floor formed 400 Myr ago: old enough to start subducting anywhere (PLT-062).
    pub(in crate::plates) fn old_floor() -> Sample {
        let mut s = Sample::blank(0);
        s.pools = oceanic_column();
        s.thickness_m = s.pools.iter().sum();
        s.elevation_m = -5600.0;
        s.formed_myr = -400.0;
        s
    }

    /// Stern 2004 (§4.1): "There are no Cenozoic examples of a passive continental margin
    /// transforming into a convergent margin"; Crameri et al. 2020: passive margins are less
    /// favourable for subduction initiation than heterogeneous ocean interiors. A continent
    /// (a 60° cap) in an ocean of old floor: off, the initiation's fracture runs along the
    /// continent's edge on some draws and the old floor beside the coast goes to the sinking plate
    /// (the passive margin turned active, which PLT-067 meant to stop); on, no sample of the
    /// sinking plate lies within the arc's reach and a spacing of the continent, so the arc the
    /// new trench builds stands on ocean floor; with an arc reaching across the ocean there is no
    /// initiation. The planet has no trench, so this is PLT-062's siting (fix C off; with it on,
    /// nothing starts without a trench). ledger: PLT-503, PLT-067
    #[test]
    fn subduction_starts_off_the_passive_margin() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let cap = |p: DVec3| p.z > 0.5;
        let initiate_with = |offshore: bool, arc_km: f64, seed: u64| -> (PlateState, Vec<DVec3>) {
            let k = TectonicParams {
                initiation_near_trenches: false,
                subduction_initiation_per_myr: 100.0,
                subduction_initiation_offshore: offshore,
                arc_distance_km: arc_km,
                ..Default::default()
            };
            let mut state = one_plate(
                &geo,
                seed,
                |p| if cap(p) { continental() } else { old_floor() },
            );
            let pos = positions(&state, &geo);
            let mut world = WorldState::new(1, 16_384, 0.0);
            maybe_initiate_subduction(&mut state, &mut world, &geo, &pos, &env(&k), &[], &[])
                .unwrap();
            (state, pos)
        };
        let points = geo.points();
        let land = Tree::over(points, |i| cap(points[i]));
        let to_land = |p: DVec3| {
            land.nearest(p)
                .map_or(f64::INFINITY, |(_, d2)| angle_of_chord2(d2))
        };
        let k = TectonicParams::default();
        let norm = Norm::new(6370.0, 6371.0, N);
        let spacing = norm.spacing_rad();
        let margin = norm.rad(k.arc_distance_km + k.arc_width_km) + spacing;
        let (mut touched, mut started) = (0, 0);
        for seed in 1..=8 {
            let (base, pos) = initiate_with(false, k.arc_distance_km, seed);
            if base.plates.len() == 2
                && (0..N).any(|i| base.plate_of(i) == 1 && to_land(pos[i]) < 1.5 * spacing)
            {
                touched += 1;
            }
            let (kept, pos) = initiate_with(true, k.arc_distance_km, seed);
            if kept.plates.len() == 2 {
                started += 1;
                let nearest = (0..N)
                    .filter(|&i| kept.plate_of(i) == 1)
                    .map(|i| to_land(pos[i]))
                    .fold(f64::INFINITY, f64::min);
                assert!(
                    nearest > margin,
                    "seed {seed}: the sinking plate within {nearest} rad of the coast"
                );
            }
        }
        assert!(touched >= 1, "off, some fracture runs along the coast");
        assert!(started >= 1, "on, subduction still starts");
        let (none, _) = initiate_with(true, 20_000.0, 1);
        assert_eq!(
            none.plates.len(),
            1,
            "no old floor beyond the margin: no initiation"
        );
    }

    /// Audet & Bürgmann 2011 (abstract): "strain is concentrated at pre-existing zones of
    /// weakness"; the plate's effective elastic thickness is "low in the surrounding Phanerozoic
    /// belts". A plate of 225 km lithosphere with a thin belt (30 km) between 14.5° and 23.6° N,
    /// its centres drawn in the polar caps: the two fragments' fracture runs along the equator's
    /// side of the caps' bisector without the weakness, and inside the belt with it. ledger: PLT-506
    #[test]
    fn rifts_break_where_the_lithosphere_is_thin() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let belt = |p: DVec3| p.z > 0.25 && p.z < 0.4;
        let split = |weakness: f64| -> f64 {
            let k = TectonicParams {
                rift_warp_rad: 0.0,
                rift_weakness: weakness,
                ..Default::default()
            };
            let mut state = one_plate(&geo, 7, |p| {
                let mut s = continental();
                s.lithosphere_km = if belt(p) { 30.0 } else { 225.0 };
                // Only the caps are visible, so the fragments' centres are drawn there.
                if p.z.abs() < 0.6 {
                    s.flags |= HIDDEN;
                }
                s
            });
            let pos = positions(&state, &geo);
            let mut world = WorldState::new(1, 16_384, 0.0);
            rift(&mut state, &mut world, &geo, &pos, &env(&k), 0, 2).unwrap();
            assert_eq!(state.plates.len(), 2, "the plate rifted");
            let line: Vec<usize> = (0..N)
                .filter(|&i| {
                    geo.edges(i as u32)
                        .any(|(u, _)| state.plate_of(u as usize) != state.plate_of(i))
                })
                .collect();
            line.iter().filter(|&&i| belt(pos[i])).count() as f64 / line.len() as f64
        };
        let (plain, weak) = (split(0.0), split(2.0));
        assert!(plain < 0.3, "the bisector mostly misses the belt: {plain}");
        assert!(weak > 0.8, "the fracture runs in the thin belt: {weak}");
    }

    /// The plate under a trench, as `inherit`'s tests build it: plate 0 (old floor) subducts under
    /// plate 1 (z > 0.5) in a collision whose front is at +x, with a slab of 300 km.
    fn under_a_trench(geo: &Geo, seed: u64) -> (PlateState, DVec3) {
        use crate::plates::inherit::tests::{add_plate, collision_at, contact};
        let front = DVec3::new(0.87, 0.0, 0.5).normalize();
        let mut st = one_plate(geo, seed, |_| old_floor());
        add_plate(&mut st, geo, 1, |p| p.z > 0.5);
        let mut c = contact(0, 1, front);
        c.collision = Some(collision_at(front));
        st.contacts.push(c);
        st.slabs.push(SlabTrack {
            under: 0,
            over: 1,
            km: 300.0,
            ..SlabTrack::default()
        });
        (st, front)
    }

    /// A rift of a subducting plate (Stock & Lee 1994: "The fragmentation of a subducting plate
    /// creates a new triple junction, two limbs of which are along the trench"; the Farallon
    /// plate's fission into the Cocos and Nazca plates, whose trench both kept): with
    /// `split_inheritance` every fragment that borders the overriding plate keeps subducting
    /// under it with the slab, the collision goes with its front, and a fragment away from the
    /// trench gets nothing. Off, the parent keeps the one contact and slab. ledger: PLT-534
    #[test]
    fn the_fragments_of_a_rifted_plate_keep_the_trench_they_border() {
        use crate::plates::inherit::tests::assert_inherited;
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let run = |on: bool| {
            let k = TectonicParams {
                split_inheritance: on,
                ..Default::default()
            };
            let (mut st, front) = under_a_trench(&geo, 3);
            let pos = positions(&st, &geo);
            let mut world = WorldState::new(1, 16_384, 0.0);
            rift(&mut st, &mut world, &geo, &pos, &env(&k), 0, 3).unwrap();
            let pieces: Vec<u32> = std::iter::once(0)
                .chain(
                    st.plates
                        .iter()
                        .filter(|p| p.parent == Some(0))
                        .map(|p| p.id),
                )
                .collect();
            assert_eq!(pieces.len(), 3, "the plate rifted in three");
            (st, pos, front, pieces, world)
        };
        let k = TectonicParams::default();
        let (st, pos, front, pieces, world) = run(true);
        let bordering = assert_inherited(&st, &pos, &env(&k), &pieces, 1, front, 300.0);
        assert!(
            bordering.len() >= 2,
            "two fragments at the trench: {bordering:?}"
        );
        assert!(world.globals[crate::plates::inherit::COPIED_GLOBAL] >= 1.0);
        let (st, ..) = run(false);
        assert_eq!(st.contacts.len(), 1);
        assert_eq!((st.contacts[0].under, st.contacts[0].over), (0, 1));
        assert_eq!(st.slabs.len(), 1);
    }

    /// Gurnis, Hall & Lavier 2004 [60]: at a fracture zone in compression "the older plate is
    /// thrust beneath the younger"; [78]: with an asymmetry in age, "the weak zone overlies the
    /// thicker plate". A plate of old floor split at the plane x = 0 (no warp; the old list is
    /// x > 0, the seed at +x): on, the planted slab hangs from the older side of the fracture, the
    /// split's or the parent's, and from the split where the parent's side of the fracture is
    /// continental crust however old the floor beyond it, or where the ages tie; off, from the
    /// split whatever the ages. ledger: PLT-655, PLT-270
    #[test]
    fn the_older_side_of_the_fracture_goes_under() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let plant = |on: bool, split_age: f32, parent_age: f32, coast: bool| -> (u32, u32) {
            let k = TectonicParams {
                initiation_polarity_by_age: on,
                rift_warp_rad: 0.0,
                ..Default::default()
            };
            let mut st = one_plate(&geo, 3, |p| {
                if coast && p.x < 0.0 && p.x > -0.3 {
                    continental()
                } else {
                    let mut s = old_floor();
                    let age = if p.x > 0.0 { split_age } else { parent_age };
                    s.formed_myr = -age;
                    s
                }
            });
            let pos = positions(&st, &geo);
            let old: Vec<u32> = (0..N as u32).filter(|&i| pos[i as usize].x > 0.0).collect();
            let seed = *old
                .iter()
                .min_by(|&&a, &&b| {
                    DVec3::X
                        .angle_between(pos[a as usize])
                        .total_cmp(&DVec3::X.angle_between(pos[b as usize]))
                })
                .unwrap();
            let mut world = WorldState::new(1, 16_384, 0.0);
            initiate(
                &mut st,
                &mut world,
                &geo,
                &pos,
                &env(&k),
                0,
                seed,
                &old,
                None,
                None,
            )
            .unwrap();
            assert_eq!(st.plates.len(), 2, "an initiation");
            assert_eq!(st.slabs.len(), 1, "one planted slab");
            (st.slabs[0].under, st.slabs[0].over)
        };
        assert_eq!(
            plant(true, 200.0, 300.0, false),
            (0, 1),
            "the parent's is older"
        );
        assert_eq!(
            plant(true, 300.0, 200.0, false),
            (1, 0),
            "the split's is older"
        );
        assert_eq!(plant(true, 250.0, 250.0, false), (1, 0), "a tie: the split");
        assert_eq!(
            plant(true, 200.0, 300.0, true),
            (1, 0),
            "continental crust at the fracture does not go under"
        );
        assert_eq!(plant(false, 200.0, 300.0, false), (1, 0), "off: the split");
    }

    /// A subduction initiation's new plate is a split too: the old floor that sinks takes the
    /// part of the trench it borders, its slab and the collision at its edge, the parent keeps
    /// the part it still borders, and the slab planted under the fracture (PLT-270) stays.
    /// ledger: PLT-534
    #[test]
    fn an_initiation_hands_the_sinking_plate_the_trench_it_borders() {
        use crate::plates::inherit::tests::assert_inherited;
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = TectonicParams {
            split_inheritance: true,
            ..Default::default()
        };
        let (mut st, front) = under_a_trench(&geo, 3);
        let pos = positions(&st, &geo);
        let old: Vec<u32> = (0..N as u32)
            .filter(|&i| st.plate_of(i as usize) == 0 && pos[i as usize].z > 0.0)
            .collect();
        let at = DVec3::new(0.95, 0.0, 0.31).normalize();
        let seed = *old
            .iter()
            .min_by(|&&a, &&b| {
                at.angle_between(pos[a as usize])
                    .total_cmp(&at.angle_between(pos[b as usize]))
            })
            .unwrap();
        let mut world = WorldState::new(1, 16_384, 0.0);
        initiate(
            &mut st,
            &mut world,
            &geo,
            &pos,
            &env(&k),
            0,
            seed,
            &old,
            None,
            None,
        )
        .unwrap();
        assert_eq!(st.plates.len(), 3, "an initiation");
        let bordering = assert_inherited(&st, &pos, &env(&k), &[0, 2], 1, front, 300.0);
        assert_eq!(bordering, vec![0, 2], "both border the trench");
        assert!(
            crate::plates::inherit::tests::find(&st, 2, 1)
                .unwrap()
                .collision
                .is_some()
        );
        assert_eq!(
            crate::plates::inherit::tests::slab_of(&st, 2, 0).map(|s| s.km),
            Some(k.initiation_slab_km)
        );
    }

    /// The visible samples with a lattice neighbour on another plate.
    fn fracture(st: &PlateState, geo: &Geo) -> Vec<usize> {
        (0..N)
            .filter(|&i| {
                !st.samples[i].is_hidden()
                    && geo
                        .edges(i as u32)
                        .any(|(u, _)| st.plate_of(u as usize) != st.plate_of(i))
            })
            .collect()
    }

    /// Crameri et al. 2020: "All SZI events start less than 1500 km away from other active
    /// subduction zones and, in most cases, less than 500 km", and the subducting plates' mean
    /// ages at initiation run from 31.7 to 117.4 Myr. A plate of floor 10 Myr old with a
    /// subduction zone 2,200 km long on the equator: with fix C an initiation starts, its fracture
    /// within half the reach of the trench (the bisector of a seed within reach and its foot on
    /// the trench); off, floor that young never goes (PLT-062); with no subduction zone nothing
    /// starts on, and samples flagged TRENCH with no front this step (a collision, an incipient
    /// front, floor held under after its contact ended) are none. ledger: PLT-690, PLT-691,
    /// PLT-692
    #[test]
    fn subduction_starts_near_trenches_at_any_age() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let norm = Norm::new(6370.0, 6371.0, N);
        let spacing = norm.spacing_rad();
        let on_trench = |p: DVec3| p.z.asin().abs() < spacing && p.y.atan2(p.x).abs() < 0.17;
        let run = |near: bool, flagged: bool, front: bool, seed: u64| {
            let k = TectonicParams {
                initiation_near_trenches: near,
                subduction_initiation_per_trench_km_myr: 1.0,
                subduction_initiation_per_myr: 100.0,
                rift_warp_rad: 0.0,
                ..Default::default()
            };
            let mut st = one_plate(&geo, seed, |p| {
                let mut s = old_floor();
                s.formed_myr = -10.0;
                if flagged && on_trench(p) {
                    s.flags |= TRENCH;
                }
                s
            });
            let pos = positions(&st, &geo);
            let fronts: Vec<u32> = (0..N as u32)
                .filter(|&i| front && on_trench(pos[i as usize]))
                .collect();
            let mut world = WorldState::new(1, 16_384, 0.0);
            maybe_initiate_subduction(&mut st, &mut world, &geo, &pos, &env(&k), &[], &fronts)
                .unwrap();
            // The hazard's length, measured: one spacing per front sample × the step.
            let read = world.globals.get(INITIATION_TRENCH_GLOBAL).copied();
            let want = fronts.len() as f64 * norm.km(spacing) * env(&k).dt_myr;
            if near && front {
                let got = read.unwrap_or(0.0);
                assert!((got - want).abs() < 1e-6 * want, "{got} km Myr, not {want}");
            } else {
                assert_eq!(read, None, "no fix C hazard read");
            }
            (st, pos)
        };
        let reach = norm.rad(TectonicParams::default().subduction_initiation_reach_km);
        let points = geo.points();
        let line = Tree::over(points, |i| on_trench(points[i]));
        let to_trench = |p: DVec3| {
            line.nearest(p)
                .map_or(f64::INFINITY, |(_, d2)| angle_of_chord2(d2))
        };
        for seed in 1..=4 {
            let (st, pos) = run(true, true, true, seed);
            assert_eq!(st.plates.len(), 2, "seed {seed}: an initiation");
            assert_eq!(st.counters.initiations, 1);
            let closest = fracture(&st, &geo)
                .into_iter()
                .map(|i| to_trench(pos[i]))
                .fold(f64::INFINITY, f64::min);
            assert!(
                closest <= reach / 2.0 + spacing,
                "seed {seed}: the fracture {} km from the trench",
                norm.km(closest)
            );
            assert_eq!(st.counters.initiations_reach_1000, 1);
            assert_eq!(st.counters.initiations_refused, 0);
            let (off, _) = run(false, true, true, seed);
            assert_eq!(off.plates.len(), 1, "seed {seed}: off, young floor stays");
            let (none, _) = run(true, false, false, seed);
            assert_eq!(
                none.plates.len(),
                1,
                "seed {seed}: no trench, no initiation"
            );
            let (rim, _) = run(true, true, false, seed);
            assert_eq!(
                rim.plates.len(),
                1,
                "seed {seed}: a TRENCH rim with no front this step, no initiation"
            );
        }
    }

    /// Crameri et al. 2020, Table 1: ten of the thirteen initiations since about 105 Ma lie at "a
    /// normal offset, but otherwise parallel to the closest, pre-existing subduction trench", three
    /// at an oblique one, none along its strike. With fix C the fracture is the bisector of the
    /// seed and its foot on the nearest trench: a seed 6° north of a trench along the equator
    /// splits the plate along 3° N, level with the trench (the young floor's centroid, the south
    /// pole, would put it at 42° S). The trench sample nearest the seed alone tilts it by the
    /// lattice's offset along the trench (7.7° here, the first version). ledger: PLT-690
    #[test]
    fn the_fracture_runs_parallel_to_the_nearest_trench() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let spacing = Norm::new(6370.0, 6371.0, N).spacing_rad();
        let k = TectonicParams {
            rift_warp_rad: 0.0,
            ..Default::default()
        };
        let mut st = one_plate(&geo, 3, |_| old_floor());
        let pos = positions(&st, &geo);
        let lat = |p: DVec3| p.z.asin();
        let lon = |p: DVec3| p.y.atan2(p.x);
        let closest = |q: DVec3| {
            (0..N as u32)
                .min_by(|&a, &b| {
                    q.angle_between(pos[a as usize])
                        .total_cmp(&q.angle_between(pos[b as usize]))
                })
                .unwrap()
        };
        let at = |lat: f64| DVec3::new(lat.to_radians().cos(), 0.0, lat.to_radians().sin());
        let seed = closest(at(6.0));
        // A trench along the equator: the samples within half a spacing of it, over 60° of
        // longitude.
        let trenches: Vec<DVec3> = pos
            .iter()
            .copied()
            .filter(|&p| lat(p).abs() < 0.5 * spacing && p.x > 0.0 && lon(p).abs() < 0.52)
            .collect();
        let tree = Tree::over(&trenches, |_| true);
        let foot = trench_foot(pos[seed as usize], &trenches, &tree, 3.0 * spacing).unwrap();
        assert!(
            lat(foot).abs() < 0.25 * spacing
                && (lon(foot) - lon(pos[seed as usize])).abs() < 0.25 * spacing,
            "the foot ({:.2}, {:.2}) of the seed ({:.2}, {:.2})",
            lat(foot).to_degrees(),
            lon(foot).to_degrees(),
            lat(pos[seed as usize]).to_degrees(),
            lon(pos[seed as usize]).to_degrees()
        );
        let old: Vec<u32> = (0..N as u32).filter(|&i| pos[i as usize].z > 0.0).collect();
        let mut world = WorldState::new(1, 16_384, 0.0);
        initiate(
            &mut st,
            &mut world,
            &geo,
            &pos,
            &env(&k),
            0,
            seed,
            &old,
            None,
            Some(foot),
        )
        .unwrap();
        assert_eq!(st.plates.len(), 2, "an initiation");
        let mid = (lat(pos[seed as usize]) + lat(foot)) / 2.0;
        // The fracture's samples near the meridian (both sides of the split, each within an edge
        // or two of it, so the line through them is the bisector): a least-squares line in
        // latitude against longitude, level with the trench and halfway to the seed.
        let near_meridian: Vec<(f64, f64)> = fracture(&st, &geo)
            .into_iter()
            .filter(|&i| pos[i].x > 0.0 && lon(pos[i]).abs() < 10f64.to_radians())
            .map(|i| (lon(pos[i]), lat(pos[i])))
            .collect();
        assert!(near_meridian.len() >= 8, "{}", near_meridian.len());
        let n = near_meridian.len() as f64;
        let (mx, my) = near_meridian
            .iter()
            .fold((0.0, 0.0), |(a, b), &(x, y)| (a + x / n, b + y / n));
        let (sxx, sxy) = near_meridian.iter().fold((0.0, 0.0), |(a, b), &(x, y)| {
            (a + (x - mx) * (x - mx), b + (x - mx) * (y - my))
        });
        let slope = sxy / sxx;
        let at_meridian = my - slope * mx;
        assert!(
            slope.atan().abs() < 3f64.to_radians() && (at_meridian - mid).abs() < 0.5 * spacing,
            "the fracture at {:.2}° N on the meridian, tilted {:.2}°, against {:.2}° N: {:?}",
            at_meridian.to_degrees(),
            slope.atan().to_degrees(),
            mid.to_degrees(),
            near_meridian
                .iter()
                .map(|&(x, y)| format!("({:.2}, {:.2})", y.to_degrees(), x.to_degrees()))
                .collect::<Vec<_>>()
        );
    }

    /// Fix C's rate (ledger PLT-692): Crameri et al. 2020 count four newly destructive
    /// initiations since about 105 Ma, against Bird 2003's 51,310 km of subduction zones; with
    /// Earth's trench length a planet expects four in 105 Myr, however its qualifying floor is
    /// shared out.
    #[test]
    fn the_initiation_hazard_is_the_trench_length_rate() {
        let r = TectonicParams::default().subduction_initiation_per_trench_km_myr;
        for floor in [16, 1_000, 9_999] {
            let h = near_trench_hazard(r, 51_310.0, 105.0, floor);
            assert!((h * floor as f64 - 4.0).abs() < 1e-9, "{floor}: {h}");
        }
    }

    /// A back-arc test planet: the state, the positions, the subducting fronts and their detail.
    pub(in crate::plates) type BackarcPlanet =
        (PlateState, Vec<DVec3>, Vec<u32>, Vec<(u32, DVec3)>);

    /// Back-arc rifting's test planet: plate 0 (U) south of the equator, plate 1 (O) north of it
    /// on the hemisphere x > 0, plate 2 the rest, all of floor `age_myr` old. O turns about −Y
    /// at `mm_yr` (its front moves north, away from the trench, by up to that; negative, towards
    /// it), a slab `slab_km` long hangs from U under O with their contact. Returns the state, the
    /// positions and the step's subducting fronts as `subduction::Pull` gives them: O's samples
    /// with a lattice neighbour on U, each with that neighbour and the normal towards it (south).
    pub(in crate::plates) fn backarc_planet(
        geo: &Geo,
        age_myr: f64,
        mm_yr: f64,
        slab_km: f64,
    ) -> BackarcPlanet {
        use crate::plates::inherit::tests::{add_plate, contact};
        let mut st = one_plate(geo, 7, |_| {
            let mut s = old_floor();
            s.formed_myr = (2.0 - age_myr) as f32;
            s
        });
        add_plate(&mut st, geo, 1, |p| p.x > 0.0 && p.z > 0.0);
        add_plate(&mut st, geo, 2, |p| p.x <= 0.0);
        let norm = Norm::new(6370.0, 6371.0, N);
        st.plates[1].omega = (-DVec3::Y * norm.omega_of_speed(mm_yr)).to_array();
        st.contacts.push(contact(0, 1, DVec3::X));
        st.slabs.push(SlabTrack {
            under: 0,
            over: 1,
            km: slab_km,
            ..SlabTrack::default()
        });
        let pos = positions(&st, geo);
        let (mut fronts, mut detail) = (Vec::new(), Vec::new());
        for (v, &x) in pos.iter().enumerate() {
            if st.plate_of(v) != 1 {
                continue;
            }
            if let Some((u, _)) = geo
                .edges(v as u32)
                .find(|&(u, _)| st.plate_of(u as usize) == 0)
            {
                fronts.push(v as u32);
                detail.push((u, (-DVec3::Z - x * x.dot(-DVec3::Z)).normalize()));
            }
        }
        (st, pos, fronts, detail)
    }

    fn backarc_on(rate: f64) -> TectonicParams {
        TectonicParams {
            backarc_rifting: true,
            backarc_rifting_per_myr: rate,
            ..TectonicParams::default()
        }
    }

    /// Sdrolias & Müller 2006 (ledger PLT-710 to PLT-714): back-arc basins "develop when the age
    /// of subducting normal oceanic lithosphere is greater than 55 million years", their
    /// initiation "is preceded by the landward migration of the overriding plate", the locus is
    /// "commonly in between the remnant and active volcanic arc"; Karig 1971: extension only where
    /// the slab passes "about 350 km". A front with 80 Myr floor coming in, O moving away from it
    /// at up to 30 mm/yr and a 400 km slab qualifies: a certain draw splits off O's samples
    /// within `arc_distance_km` of its fronts (the forearc and the arc) as a plate with O's
    /// motion, which takes the contact and the slab, is linked to O to heal and is on the
    /// sliver list; one rift_onset and one plate_birth with `origin` "back_arc". A floor of 50
    /// Myr, O moving towards the trench, a 300 km slab, or the switch off: nothing, and off, not
    /// a draw of the generator nor a global.
    #[test]
    fn a_backarc_rifts_over_an_old_deep_slab_behind_a_retreating_upper_plate() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = backarc_on(1.0e9);
        let (mut st, pos, fronts, detail) = backarc_planet(&geo, 80.0, 30.0, 400.0);
        let before: Vec<u32> = (0..pos.len()).map(|i| st.plate_of(i)).collect();
        let mut world = WorldState::new(1, 16_384, 0.0);
        maybe_rift_backarc(&mut st, &mut world, &geo, &pos, &env(&k), &fronts, &detail).unwrap();
        assert_eq!(st.plates.len(), 4, "one sliver");
        let s = 3u32;
        assert_eq!(st.plates[3].parent, Some(1));
        assert_eq!(st.counters.backarc_rifts, 1);
        assert_eq!(
            st.backarcs,
            vec![BackArc {
                plate: s,
                parent: 1,
                born_myr: 2.0
            }]
        );
        assert!(st.rift_links.iter().any(|l| (l.a, l.b) == (1, s)));
        // Every sample of the sliver was O's and lies within the arc's distance of a front; every
        // front is on it.
        let reach = env(&k).norm.rad(k.arc_distance_km);
        let tree = Tree::over(&pos, |i| fronts.contains(&(i as u32)));
        let sliver: Vec<usize> = (0..pos.len()).filter(|&i| st.plate_of(i) == s).collect();
        assert!(sliver.len() >= BACKARC_MIN_SAMPLES, "{}", sliver.len());
        for &i in &sliver {
            assert_eq!(before[i], 1);
            let (_, d2) = tree.nearest(pos[i]).unwrap();
            assert!(
                angle_of_chord2(d2) <= reach + 1e-12,
                "sample {i} beyond the arc"
            );
        }
        assert!(fronts.iter().all(|&v| st.plate_of(v as usize) == s));
        assert_eq!(st.plates[3].omega, st.plates[1].omega);
        assert!(
            st.contacts.iter().any(|c| (c.under, c.over) == (0, s)),
            "the sliver holds the contact"
        );
        assert!(
            st.slabs
                .iter()
                .any(|sl| (sl.under, sl.over) == (0, s) && sl.km == 400.0),
            "the sliver holds the slab: {:?}",
            st.slabs
        );
        let origin = deeptime_format::ParamValue::Str(BACKARC_ORIGIN.into());
        let kinds: Vec<&str> = world.events.iter().map(|e| e.kind.as_str()).collect();
        assert_eq!(kinds, vec![RIFT_ONSET, PLATE_BIRTH], "{kinds:?}");
        assert!(
            world
                .events
                .iter()
                .all(|e| e.params.get("origin") == Some(&origin))
        );
        // The measures: every front qualified and no sliver held one this step.
        let g = |w: &WorldState, key: &str| w.globals.get(key).copied().unwrap_or(f64::NAN);
        assert!(
            (g(&world, BACKARC_QUALIFYING_GLOBAL) - g(&world, TRENCH_FRONT_GLOBAL)).abs() < 1e-9
        );
        assert_eq!(g(&world, BACKARC_FRONT_GLOBAL), 0.0);
        assert!(
            g(&world, TRENCH_VT_GLOBAL) < 0.0,
            "O moves away from its trench: Vt < 0"
        );
        // A step later the sliver holds the fronts and keeps its place on the list.
        let fronts2: Vec<u32> = fronts.clone();
        maybe_rift_backarc(&mut st, &mut world, &geo, &pos, &env(&k), &fronts2, &detail).unwrap();
        assert_eq!(st.plates.len(), 4, "a held front draws no second sliver");
        assert_eq!(st.backarcs.len(), 1);
        assert!(g(&world, BACKARC_FRONT_GLOBAL) > 0.0);
        // Without a front it leaves the list.
        maybe_rift_backarc(&mut st, &mut world, &geo, &pos, &env(&k), &[], &[]).unwrap();
        assert!(st.backarcs.is_empty());

        for (age, mm_yr, slab, why) in [
            (50.0, 30.0, 400.0, "floor of 50 Myr"),
            (80.0, -30.0, 400.0, "O moving towards the trench"),
            (80.0, 30.0, 300.0, "a 300 km slab"),
        ] {
            let (mut st, pos, fronts, detail) = backarc_planet(&geo, age, mm_yr, slab);
            let mut world = WorldState::new(1, 16_384, 0.0);
            maybe_rift_backarc(&mut st, &mut world, &geo, &pos, &env(&k), &fronts, &detail)
                .unwrap();
            assert_eq!(st.plates.len(), 3, "{why}: no sliver");
            assert_eq!(world.globals[BACKARC_QUALIFYING_GLOBAL], 0.0, "{why}");
        }
        let off = TectonicParams {
            backarc_rifting: false,
            ..backarc_on(1.0e9)
        };
        let (mut st, pos, fronts, detail) = backarc_planet(&geo, 80.0, 30.0, 400.0);
        let rng = st.rng.clone();
        let mut world = WorldState::new(1, 16_384, 0.0);
        maybe_rift_backarc(
            &mut st,
            &mut world,
            &geo,
            &pos,
            &env(&off),
            &fronts,
            &detail,
        )
        .unwrap();
        assert_eq!(st.plates.len(), 3);
        assert!(world.globals.is_empty() && world.events.is_empty());
        assert_eq!(st.rng, rng, "off draws nothing");
    }

    /// The hazard (ledger PLT-711): Heuret & Lallemand 2005's Table 1 has 16 of its 68 segments
    /// with a slab of 55 Myr or older under a landward upper plate in class E3 (back-arc
    /// spreading); Sdrolias & Müller 2006's four Cenozoic back-arc basins spread for 15.4 Myr on
    /// average. A qualifying front that converts at the rate h and spreads for D holds a spreading
    /// back-arc on the share h D / (1 + h D) of its length: Heuret & Lallemand's 16 / 68.
    #[test]
    fn the_backarc_hazard_gives_earth_s_spreading_share() {
        let d = (8.0 + 15.0 + 15.0 + 23.5) / 4.0;
        assert!((d - 15.375_f64).abs() < 1e-12);
        let exact = 16.0 / (52.0 * d);
        let share = exact * d / (1.0 + exact * d);
        assert!((share - 16.0 / 68.0).abs() < 1e-12, "{share}");
        // The default is the rate to three figures.
        let h = TectonicParams::default().backarc_rifting_per_myr;
        assert!((h / exact - 1.0).abs() < 1e-3, "{h} against {exact}");
        let share = h * d / (1.0 + h * d);
        assert!((share - 16.0 / 68.0).abs() < 1e-3, "{share}");
    }

    /// The sliver list goes into checkpoints (postcard, which is not self-describing): with no
    /// sliver and with one the plate state reads back as it was written. A field skipped when
    /// empty broke the read of a run that ended with no sliver (arm trench3, 2026-10-26).
    #[test]
    fn the_sliver_list_round_trips_a_checkpoint() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let (mut st, ..) = backarc_planet(&geo, 80.0, 30.0, 400.0);
        for _ in 0..2 {
            let bytes = postcard::to_allocvec(&st).unwrap();
            let back: PlateState = postcard::from_bytes(&bytes).unwrap();
            assert_eq!(back, st);
            st.backarcs.push(BackArc {
                plate: 1,
                parent: 0,
                born_myr: 2.0,
            });
        }
    }

    /// Bird 2003's back-arc microplates are 0.0002 to 0.016 sr (Table 1: Mariana 0.01037, Tonga
    /// 0.00625, South Sandwich 0.00317, Futuna 0.00079), below PLT-091's 0.003 of the sphere
    /// (0.0377 sr) but for the largest. A sliver on the list is kept below the size at a
    /// resampling's burial; off the list it is absorbed as any small plate (ledger PLT-715).
    #[test]
    fn a_sliver_holding_its_front_outlives_the_plate_size_rule() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = backarc_on(1.0e9);
        let (mut st, pos, fronts, detail) = backarc_planet(&geo, 80.0, 30.0, 400.0);
        let mut world = WorldState::new(1, 16_384, 0.0);
        maybe_rift_backarc(&mut st, &mut world, &geo, &pos, &env(&k), &fronts, &detail).unwrap();
        // Make the sliver small: the size rule a little above it.
        let small = TectonicParams {
            min_plate_area_frac: f64::from(st.plates[3].samples) / N as f64 * 1.5,
            ..k.clone()
        };
        let causes = BTreeMap::new();
        crate::plates::ridges::bury(&mut st, &mut world, Some(&geo), &pos, &env(&small), &causes)
            .unwrap();
        assert!(st.plates[3].alive, "on the list, the sliver stays");
        st.backarcs.clear();
        crate::plates::ridges::bury(&mut st, &mut world, Some(&geo), &pos, &env(&small), &causes)
            .unwrap();
        assert!(!st.plates[3].alive, "off the list, the size rule takes it");
    }
}

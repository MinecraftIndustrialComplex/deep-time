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
use super::state::{CONTINENTAL, Group, HIDDEN, Plate, PlateState, RiftLink, SlabTrack, TRENCH};
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
/// e^(sign·σ/σ_s) over its mean across the plates `candidates` names with their weights `w`
/// (area), so the hazard's expected count over them is unchanged and only where it falls moves.
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
    // Rifts under tension (V1 gap 2's stage C2): the plates the edge forces pull apart rift
    // more, the others less, at the same expected count. ledger: PLT-504
    let weights = if k.rift_stress_mpa > 0.0 {
        let candidates: Vec<(u32, f64)> = ids
            .iter()
            .map(|&id| &state.plates[id as usize])
            .filter(|p| f64::from(p.samples) >= min)
            .map(|p| (p.id, f64::from(p.samples)))
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
        let x = p.continental_frac;
        let f = k.rift_ocean_weight + (1.0 - k.rift_ocean_weight) * x * x;
        let boost = if env.t_myr < p.lip_until_myr {
            k.lip_rift_boost
        } else {
            1.0
        };
        // ledger: PLT-082, PLT-083 (per Myr, so the step's λ_0 scales with its length)
        let lambda0 = k.rift_per_myr * env.dt_myr;
        let mut lambda = lambda0 * f * f64::from(p.samples) * area / state.a0_km2 * boost;
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
    // lithosphere's depth (`thermal.rs`), so the fragments meet where it is thin. Off (m = 0),
    // the distance is the great circle's. ledger: PLT-506
    let grown: Option<Vec<Vec<f64>>> = (k.rift_weakness > 0.0).then(|| {
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
        label[i as usize] = (0..fragments)
            .map(|f| {
                let noise = warp * fbm(p, 4.0, 3, seed.wrapping_add(u64::from(f) * 31));
                let d = grown
                    .as_ref()
                    .map(|g| g[f as usize][i as usize])
                    .filter(|d| d.is_finite())
                    .unwrap_or_else(|| p.angle_between(centres[f as usize]));
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
    Ok(())
}

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
pub fn maybe_initiate_subduction(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    stress: &[Option<f64>],
) -> Result<(), CoreError> {
    let k = env.k;
    if k.subduction_initiation_per_myr <= 0.0 {
        return Ok(());
    }
    let t = env.t_myr;
    let area = env.norm.sample_area_km2();
    let trenches: Vec<DVec3> = state
        .samples
        .iter()
        .enumerate()
        .filter(|(_, s)| s.flags & TRENCH != 0)
        .map(|(i, _)| pos[i])
        .collect();
    let trench_tree = (!trenches.is_empty()).then(|| Tree::over(&trenches, |_| true));
    let clearance = env.norm.rad(k.subduction_initiation_clearance_km);
    let margin = env.norm.rad(k.subduction_initiation_margin_km);
    let clear = |i: usize| {
        trench_tree.as_ref().is_none_or(|tr| {
            tr.nearest(pos[i])
                .is_none_or(|(_, d2)| angle_of_chord2(d2) > clearance)
        })
    };
    let mut old: Vec<Vec<u32>> = vec![Vec::new(); state.plates.len()];
    for (i, s) in state.samples.iter().enumerate() {
        if !s.is_hidden()
            && s.kind != CONTINENTAL
            && s.age_myr(t) > k.subduction_initiation_age_myr
            && clear(i)
        {
            old[state.plate_of(i) as usize].push(i as u32);
        }
    }
    let ids: Vec<u32> = state.alive().map(|p| p.id).collect();
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
        // ledger: PLT-063 (per Myr, so the step's rate scales with its length)
        let rate = k.subduction_initiation_per_myr * env.dt_myr;
        let mut lambda = rate * o.len() as f64 * area / state.a0_km2;
        if let Some(w) = weights.as_ref().and_then(|w| w.get(&id)) {
            lambda *= w;
        }
        if state.rng.next_f64() < 1.0 - (-lambda).exp() {
            if margin > 0.0 {
                // Off the passive margin (V1 gap 2's stage C1): the old floor within the margin
                // of the plate's continental crust stays on the plate, and the fracture's seed
                // is old floor beyond it; none, no initiation. ledger: PLT-503
                let near = near_continent(state, pos, id, margin);
                let far: Vec<u32> = o.iter().copied().filter(|&i| !near[i as usize]).collect();
                if far.is_empty() {
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
                )?;
            } else {
                let seed = o[state.rng.below(o.len() as u64) as usize];
                initiate(state, world, geo, pos, env, id, seed, o, None)?;
            }
        }
    }
    Ok(())
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
) -> Result<(), CoreError> {
    let k = env.k;
    let norm = env.norm;
    let t = env.t_myr;
    let n = state.samples.len();
    let members: Vec<u32> = (0..n as u32)
        .filter(|&i| state.plate_of(i as usize) == id)
        .collect();
    let member = membership(&members, n);
    let mut is_old = vec![false; n];
    for &i in old {
        is_old[i as usize] = true;
    }
    let young: DVec3 = members
        .iter()
        .filter(|&&i| !is_old[i as usize] && !state.samples[i as usize].is_hidden())
        .map(|&i| pos[i as usize])
        .sum();
    let c_old = pos[seed as usize];
    let c_young = if young.length_squared() > 1e-12 {
        young.normalize()
    } else {
        // All old: split off the half around the seed, away from the farthest point.
        members
            .iter()
            .map(|&i| pos[i as usize])
            .max_by(|a, b| c_old.angle_between(*a).total_cmp(&c_old.angle_between(*b)))
            .unwrap_or(-c_old)
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
    // The planted slab (design §3.3): the old floor starts to sink under the fracture.
    // ledger: PLT-270
    state.slabs.push(SlabTrack {
        under: new_id,
        over: id,
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
    Ok(())
}

fn membership(members: &[u32], n: usize) -> Vec<bool> {
    let mut m = vec![false; n];
    for &i in members {
        m[i as usize] = true;
    }
    m
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
        .filter(|c| c.last_step + 1 >= env.step)
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
mod tests {
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

    const N: usize = 16_000;

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
    fn one_plate(geo: &Geo, seed: u64, make: impl Fn(DVec3) -> Sample) -> PlateState {
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
        };
        let pos = positions(&st, geo);
        st.recount(&pos);
        st
    }

    fn continental() -> Sample {
        let mut s = Sample::blank(0);
        s.kind = CONTINENTAL;
        s.pools = continental_column();
        s.thickness_m = s.pools.iter().sum();
        s.elevation_m = 300.0;
        s
    }

    /// Ocean floor formed 400 Myr ago: old enough to start subducting anywhere (PLT-062).
    fn old_floor() -> Sample {
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
    /// (a 60° cap) in an ocean of old floor: without the margin, the initiation's fracture runs
    /// along the continent's edge on some draws and the old floor beside the coast goes to the
    /// sinking plate (the passive margin turned active); with it, no sample of the sinking plate
    /// lies within the margin of the continent, and with a margin wider than the ocean there is
    /// no initiation. ledger: PLT-503, PLT-067
    #[test]
    fn subduction_starts_off_the_passive_margin() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let cap = |p: DVec3| p.z > 0.5;
        let initiate_with = |margin_km: f64, seed: u64| -> (PlateState, Vec<DVec3>) {
            let k = TectonicParams {
                subduction_initiation_per_myr: 100.0,
                subduction_initiation_margin_km: margin_km,
                ..Default::default()
            };
            let mut state = one_plate(
                &geo,
                seed,
                |p| if cap(p) { continental() } else { old_floor() },
            );
            let pos = positions(&state, &geo);
            let mut world = WorldState::new(1, 16_384, 0.0);
            maybe_initiate_subduction(&mut state, &mut world, &geo, &pos, &env(&k), &[]).unwrap();
            (state, pos)
        };
        let points = geo.points();
        let land = Tree::over(points, |i| cap(points[i]));
        let to_land = |p: DVec3| {
            land.nearest(p)
                .map_or(f64::INFINITY, |(_, d2)| angle_of_chord2(d2))
        };
        let spacing = Norm::new(6370.0, 6371.0, N).spacing_rad();
        let margin = Norm::new(6370.0, 6371.0, N).rad(500.0);
        let (mut touched, mut started) = (0, 0);
        for seed in 1..=8 {
            let (base, pos) = initiate_with(0.0, seed);
            if base.plates.len() == 2
                && (0..N).any(|i| base.plate_of(i) == 1 && to_land(pos[i]) < 1.5 * spacing)
            {
                touched += 1;
            }
            let (kept, pos) = initiate_with(500.0, seed);
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
        assert!(
            touched >= 1,
            "without the margin some fracture runs along the coast"
        );
        assert!(started >= 1, "with it, subduction still starts");
        let (none, _) = initiate_with(20_000.0, 1);
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
}

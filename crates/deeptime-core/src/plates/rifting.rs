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
//! (the samples stay where they are) and "random diverging directions": the parent's angular
//! velocity plus a rotation carrying the fragment away from the other fragments' centroid at a
//! speed drawn from `rift_speed_mm_yr`. Sample membership is only relabelled, so the parent's area
//! equals the sum of the fragments' areas exactly.

use std::collections::BTreeMap;

use glam::DVec3;

use super::Env;
use super::events::{self, PLATE_BIRTH, RIFT_ONSET};
use super::geometry::Tree;
use super::geometry::{Geo, coast_split, connect_plates};
use super::kinematics::cap_speed;
use super::noise::fbm;
use super::state::{CONTINENTAL, Group, Plate, PlateState, RiftLink, TRENCH};
use crate::error::CoreError;
use crate::sphere::kdtree::angle_of_chord2;
use crate::world::WorldState;

/// Plates below this many samples never rift.
const MIN_RIFT_SAMPLES: u32 = 64;

/// Draws a rift for every plate (in id order) and runs the ones drawn.
pub fn maybe_rift(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
) -> Result<(), CoreError> {
    let k = env.k;
    let area = env.norm.sample_area_km2();
    let min = (k.min_plate_area_frac * f64::from(state.n) * 8.0).max(f64::from(MIN_RIFT_SAMPLES));
    let ids: Vec<u32> = state.alive().map(|p| p.id).collect();
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
        let lambda = k.rift_lambda0 * f * f64::from(p.samples) * area / state.a0_km2 * boost;
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
    centres.push(pos[visible[state.rng.below(visible.len() as u64) as usize] as usize]);
    while centres.len() < fragments as usize {
        let mut best = (f64::NEG_INFINITY, DVec3::Z);
        for _ in 0..16 {
            let c = pos[visible[state.rng.below(visible.len() as u64) as usize] as usize];
            let d = centres
                .iter()
                .map(|e| e.angle_between(c))
                .fold(f64::INFINITY, f64::min);
            if d > best.0 {
                best = (d, c);
            }
        }
        centres.push(best.1);
    }
    let seed = state.rng.next_u64();
    let warp = k.rift_warp_rad;
    // Label = fragment index; non-members keep a label no fragment uses.
    let mut label = vec![u32::MAX; n];
    for &i in &members {
        let p = pos[i as usize];
        label[i as usize] = (0..fragments)
            .map(|f| {
                let noise = warp * fbm(p, 4.0, 3, seed.wrapping_add(u64::from(f) * 31));
                (f, p.angle_between(centres[f as usize]) + noise)
            })
            .fold((0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a })
            .0;
    }
    let member = membership(&members, n);
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

    let omega_parent = DVec3::from_array(parent.omega);
    let pids: Vec<u32> = order.iter().map(|f| plate_of_fragment[f]).collect();
    for &pid in &pids {
        let (s, count, cont) = sum[&pid];
        let c = s.normalize_or(line);
        let rest = (total - s).normalize_or(-c);
        let axis = rest.cross(c).normalize_or_zero();
        let v = state
            .rng
            .range(k.rift_speed_mm_yr[0], k.rift_speed_mm_yr[1]);
        let omega = cap_speed(
            omega_parent + axis * norm.omega_of_speed(v),
            &norm,
            k.max_plate_speed_mm_yr,
        );
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
/// plate) and the old part, a new plate, is set moving towards the rest at
/// `subduction_initiation_speed_mm_yr`. The convergence opens a subduction zone at the
/// fracture on the following steps, where the older crust goes down (`subduction.rs`). Emits a
/// plate_birth with `origin` "subduction_initiation".
pub fn maybe_initiate_subduction(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
) -> Result<(), CoreError> {
    let k = env.k;
    if k.subduction_initiation_rate <= 0.0 {
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
    for id in ids {
        if state.alive().count() as u32 >= k.max_plates {
            break;
        }
        let o = &old[id as usize];
        if o.len() < 16 {
            continue;
        }
        let lambda = k.subduction_initiation_rate * o.len() as f64 * area / state.a0_km2;
        if state.rng.next_f64() < 1.0 - (-lambda).exp() {
            let seed = o[state.rng.below(o.len() as u64) as usize];
            initiate(state, world, geo, pos, env, id, seed, o)?;
        }
    }
    Ok(())
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
        label[i as usize] = u32::from(da < db && state.samples[i as usize].kind != CONTINENTAL);
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
    // Old floor that encloses a continent (a terrane stranded in the ocean, which since plate
    // tasks 3 and 4 is no longer carried off at the first contact) carries it along whole; a
    // split that would be mostly continent is not old floor starting to sink.
    let continental = split
        .iter()
        .filter(|&&i| state.samples[i as usize].kind == CONTINENTAL)
        .count();
    if 2 * continental > split.len() {
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
    let axis = c.cross(c_young).normalize_or_zero();
    let v = state.rng.range(
        k.subduction_initiation_speed_mm_yr[0],
        k.subduction_initiation_speed_mm_yr[1],
    );
    let omega = cap_speed(
        DVec3::from_array(parent.omega) + axis * norm.omega_of_speed(v),
        &norm,
        k.max_plate_speed_mm_yr,
    );
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

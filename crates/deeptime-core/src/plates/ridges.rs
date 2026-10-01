// SPDX-License-Identifier: LGPL-3.0-or-later
//! Global resampling and oceanic crust generation (Cortial et al. 2019, §4.3 and §6), and the
//! bookkeeping of plates that die.
//!
//! Every `resample_min_steps`–`resample_max_steps` steps (Cortial: 10–60, "depending on the
//! observed maximum plate speed"; here when the fastest plate has moved `resample_drift_km`) the
//! crust is resampled onto a fresh lattice: the Fibonacci sphere under a new random rotation,
//! triangulated again by P1-02's hull.
//!
//! - A new sample inside a plate (a lattice triangle of one rigid group, located in the group's
//!   frame) takes that plate's crust by barycentric interpolation ("the parameters of the other
//!   sample points are computed using barycentric interpolation of crust data from the plate they
//!   intersect"). Where several plates cover it, the visible one wins (the subducting plate's
//!   hidden crust is consumed) and the conserved rock pools of the others are added to it —
//!   tectonics.js's `Crust.overlap` (see `pools.rs`): trench sediment and forced-subducted
//!   continental crust accrete to the overriding plate instead of vanishing.
//! - A new sample in a gap between diverging plates becomes fresh oceanic crust of the nearest
//!   plate. With d_P the distance to the nearest plate and d_Γ to the ridge (midway to the second
//!   nearest plate), α = d_Γ/(d_Γ + d_P) and the elevation is Cortial's
//!   z = α·z̄ + (1 − α)·z_Γ, z̄ interpolated between the two plates' edges and z_Γ the ridge
//!   profile at the sample's age (the dampening's own age–depth curve, `pools::ocean_elevation`).
//!   The crust's age is α times the time since the last resampling, and its ridge direction is
//!   r = p × (s_i − s_j) normalised (parallel to the ridge, perpendicular to spreading).
//! - The first crust made between two plates emits a ridge_birth event (caused by the rift that
//!   separated them, when one did) once max(3, n/2000) samples of new crust lie between them; a
//!   pair's ridge is born once. Plates left without samples die; plates smaller than
//!   `min_plate_area_frac` are absorbed by the neighbour they share most boundary with.

use std::collections::BTreeMap;

use deeptime_format::EventId;
use glam::{DQuat, DVec3};
use rayon::prelude::*;

use super::Env;
use super::events::{self, PLATE_DEATH, RIDGE_BIRTH};
use super::geometry::{Cover, Geo, Tree, cover, dijkstra_two};
use super::init::random_rotation;
use super::kinematics::{arr, quat, to_world};
use super::pools::{accrete, ocean_elevation, ocean_heat_flow, oceanic_column, thickness};
use super::state::{CONTINENTAL, Group, OCEANIC, PlateState, Ridge, Sample, pool, regime};
use crate::error::CoreError;
use crate::sphere::kdtree::angle_of_chord2;
use crate::world::WorldState;

/// New points within this many sample spacings of an old visible sample lie on a boundary strip,
/// not in a gap (the farthest a point of a lattice triangle can be from its nearest corner is
/// about 0.62 spacings).
const STRIP: f64 = 0.75;

/// What covers one new point: the group on top (if any) and the others.
type Found = (Option<(u32, Cover)>, Vec<(u32, Cover)>);

/// Whether a resampling is due after this step.
pub fn due(state: &PlateState, env: &Env) -> bool {
    let k = env.k;
    let since = state.step - state.resample_step;
    if since < u64::from(k.resample_min_steps) {
        return false;
    }
    if since >= u64::from(k.resample_max_steps) {
        return true;
    }
    let vmax = state
        .alive()
        .map(|p| DVec3::from_array(p.omega).length())
        .fold(0.0, f64::max);
    let drift_km = env
        .norm
        .km(vmax * (state.t_myr - state.resample_myr) * 1.0e6);
    drift_km >= k.resample_drift_km
}

/// Interpolates a sample from a cover of the old crust; vectors come out in the mantle frame,
/// which is the new lattice's frame (every group starts unrotated).
fn interpolate(src: &[Sample], c: &Cover, rot: DQuat) -> Sample {
    let m = c.major();
    let mut s = src[m as usize];
    let corners = c.tri.map(|v| &src[v as usize]);
    let w = c.w;
    let lerp = |f: &dyn Fn(&Sample) -> f32| -> f32 {
        (0..3).map(|k| w[k] * f64::from(f(corners[k]))).sum::<f64>() as f32
    };
    s.elevation_m = lerp(&|x| x.elevation_m);
    for p in 0..pool::COUNT {
        s.pools[p] = lerp(&|x| x.pools[p]);
    }
    s.arc_continental_m = lerp(&|x| x.arc_continental_m);
    s.arc_oceanic_m = lerp(&|x| x.arc_oceanic_m);
    s.oxidation = lerp(&|x| x.oxidation);
    s.heat_flow_mw_m2 = lerp(&|x| x.heat_flow_mw_m2);
    s.uplift_mm_yr = lerp(&|x| x.uplift_mm_yr);
    if s.kind == OCEANIC {
        let (mut sw, mut sf) = (0.0, 0.0);
        for k in 0..3 {
            if corners[k].kind == OCEANIC {
                sw += w[k];
                sf += w[k] * f64::from(corners[k].formed_myr);
            }
        }
        if sw > 0.0 {
            s.formed_myr = (sf / sw) as f32;
        }
    }
    let vec = |f: &dyn Fn(&Sample) -> [f32; 3]| -> [f32; 3] {
        let v: DVec3 = (0..3).map(|k| to_world(rot, f(corners[k])) * w[k]).sum();
        [v.x as f32, v.y as f32, v.z as f32]
    };
    s.ridge = vec(&|x| x.ridge);
    s.fold = vec(&|x| x.fold);
    s.flags = 0;
    s.thickness_m = thickness(&s.pools);
    s
}

/// Resamples the crust onto a fresh lattice and returns its geometry.
pub fn resample(
    state: &mut PlateState,
    world: &mut WorldState,
    pos: &[DVec3],
    old: &Geo,
    env: &Env,
) -> Result<Geo, CoreError> {
    let k = env.k;
    let norm = env.norm;
    let t = env.t_myr;
    let n = state.samples.len();
    let lattice = arr(random_rotation(&mut state.rng));
    let geo = Geo::build(n, lattice)?;
    let new_pts = geo.points();
    let rots: Vec<DQuat> = state.groups.iter().map(|g| quat(g.rot)).collect();
    let tree = Tree::over(pos, |_| true);
    let src = &state.samples;
    let st: &PlateState = state;

    // Which old groups cover each new point; the visible one on top. A point on a plate's
    // boundary strip (a lattice triangle spanning two groups) is not a gap: it copies its nearest
    // visible sample. Only points farther than `STRIP` spacings from every visible sample are gaps
    // that opened since the last resampling.
    let strip = STRIP * norm.spacing_rad();
    let found: Vec<Found> = new_pts
        .par_iter()
        .map(|&q| {
            let mut covers: Vec<(u32, Cover)> = Vec::new();
            let mut tried: Vec<u32> = Vec::with_capacity(8);
            let near = tree.nearest_n(q, 8);
            for &(j, _) in &near {
                let g = src[j as usize].group;
                if tried.contains(&g) {
                    continue;
                }
                tried.push(g);
                if let Some(c) = cover(old, st, g, rots[g as usize], q, j) {
                    covers.push((g, c));
                }
            }
            if covers.is_empty() {
                let copy = near
                    .iter()
                    .find(|(j, d2)| !src[*j as usize].is_hidden() && angle_of_chord2(*d2) <= strip);
                return match copy {
                    Some(&(j, _)) => {
                        let g = src[j as usize].group;
                        let t = old.mesh.vertex_triangle(j).unwrap_or(0);
                        let tri = old.mesh.triangles()[t as usize];
                        let w = tri.map(|v| if v == j { 1.0 } else { 0.0 });
                        (Some((g, Cover { t, tri, w })), covers)
                    }
                    None => (None, covers),
                };
            }
            // On top: visible, then continental, then higher, then the lower group id.
            let key = |i: usize| {
                let (g, c) = &covers[i];
                let m = &src[c.major() as usize];
                (!m.is_hidden(), m.kind == CONTINENTAL, m.elevation_m, *g)
            };
            let best = (0..covers.len())
                .max_by(|&a, &b| {
                    let (ka, kb) = (key(a), key(b));
                    ka.0.cmp(&kb.0)
                        .then(ka.1.cmp(&kb.1))
                        .then(ka.2.total_cmp(&kb.2))
                        .then(kb.3.cmp(&ka.3))
                })
                .unwrap_or(0);
            let top = covers.swap_remove(best);
            (Some(top), covers)
        })
        .collect();

    // Covered samples.
    let group_plate: Vec<u32> = state.groups.iter().map(|g| g.plate).collect();
    let mut plate_new: Vec<u32> = vec![u32::MAX; n];
    let mut samples: Vec<Sample> = found
        .par_iter()
        .zip(plate_new.par_iter_mut())
        .map(|((top, others), pl)| match top {
            None => Sample::blank(0),
            Some((g, c)) => {
                let mut s = interpolate(src, c, rots[*g as usize]);
                *pl = group_plate[*g as usize];
                for (g2, c2) in others {
                    let under = &src[c2.major() as usize];
                    if under.is_hidden() || group_plate[*g2 as usize] == *pl {
                        let u = interpolate(src, c2, rots[*g2 as usize]);
                        accrete(&mut s.pools, &u.pools);
                    }
                }
                s.thickness_m = thickness(&s.pools);
                s
            }
        })
        .collect();

    // Gaps: fresh oceanic crust between diverging plates.
    let gap: Vec<bool> = plate_new.iter().map(|&p| p == u32::MAX).collect();
    let mut sources = Vec::new();
    for v in 0..n as u32 {
        if !gap[v as usize] && geo.edges(v).any(|(u, _)| gap[u as usize]) {
            sources.push((v, 0.0, v));
        }
    }
    let best = dijkstra_two(
        &geo,
        &sources,
        |l| plate_new[l as usize],
        |_, b| gap[b as usize],
        std::f64::consts::PI,
    );
    let since = (t - state.resample_myr).max(env.dt_myr);
    let (zr, zf) = (k.ridge_elevation_m, k.oceanic_floor_elevation_m);
    let damping = k.oceanic_damping_mm_yr;
    let omega: Vec<DVec3> = state
        .plates
        .iter()
        .map(|p| DVec3::from_array(p.omega))
        .collect();
    let edge_z = |s: &Sample| -> f64 {
        if s.kind == CONTINENTAL {
            ocean_elevation(since, zr, zf, damping)
        } else {
            f64::from(s.elevation_m)
        }
    };
    let mut pairs: BTreeMap<(u32, u32), (usize, DVec3)> = BTreeMap::new();
    let mut orphans = Vec::new();
    for i in 0..n {
        if !gap[i] {
            continue;
        }
        let [(d1, l1), (d2, l2)] = best[i];
        if l1 == u32::MAX {
            orphans.push(i);
            continue;
        }
        let p1 = plate_new[l1 as usize];
        let z1 = edge_z(&samples[l1 as usize]);
        let (alpha, zbar, p2) = if l2 != u32::MAX {
            let z2 = edge_z(&samples[l2 as usize]);
            let s = (d1 + d2).max(1e-12);
            (
                ((d2 - d1) / s).clamp(0.0, 1.0),
                (d2 * z1 + d1 * z2) / s,
                Some(plate_new[l2 as usize]),
            )
        } else {
            (1.0, z1, None)
        };
        let age = alpha * since;
        let p = new_pts[i];
        let z = alpha * zbar + (1.0 - alpha) * ocean_elevation(age, zr, zf, damping);
        let mut s = Sample::blank(0);
        s.kind = OCEANIC;
        s.regime = regime::RIDGE;
        s.elevation_m = z as f32;
        s.formed_myr = (t - age) as f32;
        s.pools = oceanic_column();
        s.pools[pool::SEDIMENT] = (k.pelagic_m_per_myr * age) as f32;
        s.thickness_m = thickness(&s.pools);
        s.heat_flow_mw_m2 = ocean_heat_flow(age) as f32;
        if let Some(p2) = p2 {
            let spread = norm.velocity_mm_yr(omega[p1 as usize] - omega[p2 as usize], p);
            let r = p.cross(spread).normalize_or_zero();
            s.ridge = [r.x as f32, r.y as f32, r.z as f32];
            let e = pairs
                .entry((p1.min(p2), p1.max(p2)))
                .or_insert((0, DVec3::ZERO));
            e.0 += 1;
            e.1 += p;
        }
        samples[i] = s;
        plate_new[i] = p1;
    }
    for i in orphans {
        // Unreachable gap pockets: the nearest old sample's plate.
        let (j, _) = tree.nearest(new_pts[i]).unwrap_or((0, 0.0));
        let mut s = Sample::blank(0);
        s.pools = oceanic_column();
        s.thickness_m = thickness(&s.pools);
        s.elevation_m = zr as f32;
        s.formed_myr = t as f32;
        samples[i] = s;
        plate_new[i] = group_plate[state.samples[j as usize].group as usize];
    }

    // One unrotated group per plate.
    let mut group_of: BTreeMap<u32, u32> = BTreeMap::new();
    let mut groups: Vec<Group> = Vec::new();
    for p in state.alive() {
        group_of.insert(p.id, groups.len() as u32);
        groups.push(Group {
            plate: p.id,
            rot: [0.0, 0.0, 0.0, 1.0],
        });
    }
    for (s, &p) in samples.iter_mut().zip(&plate_new) {
        s.group = match group_of.get(&p) {
            Some(&g) => g,
            None => {
                // A plate that died this step still owned crust: keep it alive.
                let g = groups.len() as u32;
                group_of.insert(p, g);
                groups.push(Group {
                    plate: p,
                    rot: [0.0, 0.0, 0.0, 1.0],
                });
                g
            }
        };
    }
    state.samples = samples;
    state.groups = groups;
    state.lattice = lattice;
    state.resample_step = state.step;
    state.resample_myr = t;
    state.counters.resamplings += 1;
    let resampling = state.counters.resamplings;

    // Ridge births.
    let min_ridge = 3.max(n / 2000);
    for ((a, b), (count, sum)) in pairs {
        if count < min_ridge {
            continue;
        }
        let at = sum.normalize_or(DVec3::Z);
        if let Some(r) = state.ridges.iter_mut().find(|r| r.a == a && r.b == b) {
            r.last_resampling = resampling;
            continue;
        }
        let spread = norm.speed_mm_yr(omega[a as usize] - omega[b as usize], at);
        let mut e = events::event(world, t, RIDGE_BIRTH, at)
            .magnitude((spread / k.max_plate_speed_mm_yr) as f32)
            .param("plate_a", i64::from(a))
            .param("plate_b", i64::from(b))
            .param("spreading_mm_yr", spread)
            .param("new_samples", count as i64);
        if let Some(link) = state.rift_links.iter().find(|l| l.a == a && l.b == b) {
            e = e.caused_by(link.event);
        }
        let id = world.emit(e)?;
        state.counters.ridges += 1;
        state.ridges.push(Ridge {
            a,
            b,
            event: id,
            last_resampling: resampling,
        });
    }
    Ok(geo)
}

/// Marks plates without samples dead and, when `geo` is the lattice every group shares (right
/// after a resampling), absorbs plates below `min_plate_area_frac` into the neighbour they share
/// most lattice edges with. `pos` are current positions (for centroids). `causes` maps a plate to
/// the event that doomed it (its last subduction onset, a collision).
pub fn bury(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: Option<&Geo>,
    pos: &[DVec3],
    env: &Env,
    causes: &BTreeMap<u32, EventId>,
) -> Result<(), CoreError> {
    let t = env.t_myr;
    state.recount(pos);
    if let Some(geo) = geo {
        let min = (env.k.min_plate_area_frac * f64::from(state.n)).ceil() as u32;
        while let Some(small) = state
            .alive()
            .filter(|p| p.samples > 0 && p.samples < min)
            .min_by_key(|p| (p.samples, p.id))
            .map(|p| p.id)
        {
            let mut votes: BTreeMap<u32, usize> = BTreeMap::new();
            for i in 0..state.samples.len() {
                if state.plate_of(i) != small {
                    continue;
                }
                for (u, _) in geo.edges(i as u32) {
                    let pu = state.plate_of(u as usize);
                    if pu != small {
                        *votes.entry(pu).or_default() += 1;
                    }
                }
            }
            let Some((&into, _)) = votes
                .iter()
                .max_by_key(|(p, n)| (**n, std::cmp::Reverse(**p)))
            else {
                break;
            };
            let target_group = state
                .groups
                .iter()
                .position(|g| g.plate == into)
                .expect("a plate with samples has a group") as u32;
            for s in &mut state.samples {
                if state.groups[s.group as usize].plate == small {
                    s.group = target_group;
                }
            }
            let at = DVec3::from_array(state.plates[small as usize].centroid);
            let e = events::event(world, t, PLATE_DEATH, at)
                .param("plate", i64::from(small))
                .param("fate", "absorbed")
                .param("absorbed_by", i64::from(into));
            world.emit(e)?;
            let p = &mut state.plates[small as usize];
            p.alive = false;
            p.died_myr = Some(t);
            p.samples = 0;
            state.counters.plate_deaths += 1;
            state.recount(pos);
        }
        // Groups of plates absorbed above are now empty; drop them and renumber.
        let used: Vec<bool> = {
            let mut u = vec![false; state.groups.len()];
            for s in &state.samples {
                u[s.group as usize] = true;
            }
            u
        };
        if used.iter().any(|&u| !u) {
            let mut remap = vec![u32::MAX; state.groups.len()];
            let mut groups = Vec::new();
            for (g, grp) in state.groups.iter().enumerate() {
                if used[g] {
                    remap[g] = groups.len() as u32;
                    groups.push(*grp);
                }
            }
            for s in &mut state.samples {
                s.group = remap[s.group as usize];
            }
            state.groups = groups;
        }
    }
    let dead: Vec<u32> = state
        .alive()
        .filter(|p| p.samples == 0)
        .map(|p| p.id)
        .collect();
    for id in dead {
        let at = DVec3::from_array(state.plates[id as usize].centroid);
        let mut e = events::event(world, t, PLATE_DEATH, at)
            .param("plate", i64::from(id))
            .param("fate", "consumed");
        if let Some(&c) = causes.get(&id) {
            e = e.caused_by(c);
        }
        world.emit(e)?;
        let p = &mut state.plates[id as usize];
        p.alive = false;
        p.died_myr = Some(t);
        state.counters.plate_deaths += 1;
    }
    Ok(())
}

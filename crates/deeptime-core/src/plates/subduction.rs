// SPDX-License-Identifier: LGPL-3.0-or-later
//! Convergent boundaries: overlap detection, subduction polarity, trenches, subduction uplift and
//! arc magmatism (Cortial et al. 2019, §4.1 and §6 "Plate boundary tracking").
//!
//! **Overlaps.** Every step, each sample looks at its 8 nearest samples (all plates, current
//! positions); for each other plate among them it asks whether it lies inside one of that plate's
//! lattice triangles (three vertices of one rigid group, located in the group's frame). Such a
//! sample overlaps the other plate: the plates interpenetrate there.
//!
//! **Collisions.** `resolve` also lists, per contact, the subducting plate's continental crust in
//! the overlap (the seeds of a terrane). Whether a contact is in collision, how much it has
//! shortened and when its terrane is transferred is `shortening.rs`.
//!
//! **Polarity** is decided once per plate pair, when the first overlap of at least three samples
//! appears (Cortial: "Two converging oceanic plates always lead to the subduction of the older
//! plate. An oceanic plate always plunges beneath a continental plate … partial subduction when we
//! find a continental–continental configuration"; the smaller plate is forced under), and kept for
//! the contact's life, except that continental crust arriving on the subducting side of an oceanic
//! overriding plate reverses it. Samples of the subducting plate inside the overriding one are
//! hidden.
//!
//! **Fronts and uplift.** The front is the overriding plate's boundary where the subducting plate
//! converges on it (its lattice boundary samples, which stay fixed in its frame). A multi-source
//! Dijkstra inside each rigid group gives d(p), the distance to the front, up to r_s, and carries
//! the front's relative speed v and the subducting crust's elevation z_i. The uplift is Cortial's
//! ũ_j(p) = u_0 · f(d(p)) · g(v(p)) · h(z̃_i(p)), with f a piecewise cubic rising to 1 at
//! `subduction_peak_km` and falling to 0 at r_s, g(v) = v / v_0 and h(z̃) = z̃², z̃ = (z_i − z_t) /
//! (z_c − z_t). The fold direction follows the relative motion (f += β(s_i − s_j)δt), and uplifted
//! continental or emerging crust gets an Andean orogeny of age 0.
//!
//! **Extensions (design choices, recorded in STATUS.md).** An arc band (Gaussian, centred
//! `arc_distance_km` from the front) adds arc magma to the rock pools and the deposit
//! accumulators, and builds submarine overriding crust up at `arc_uplift_mm_yr` so island arcs
//! emerge (Cortial's h(z̃) makes abyssal slabs lift an oceanic overriding plate far slower than its
//! dampening); emerged arc crust becomes continental. A wider band around the arc
//! (`cordillera_width_km`) lifts continental crust at `cordillera_uplift_mm_yr`, slowing as it nears
//! `cordillera_height_m` (P1-27: the arc's own range, built by arc magmatism and crustal
//! thickening, which h(z̃) of an abyssal slab leaves at a few hundredths of a mm/yr), and adds the
//! Airy root that carries it to the crust. Trench samples (the subducting plate's
//! visible samples next to its own hidden ones) are pulled towards z_t and receive sediment
//! accretion (`pools.rs`). Slab pull turns the subducting plate's axis towards its fronts. Uplift
//! of crust above sea level is scaled by the isostatic ceiling (1 − z/z_c) ([`ceiling`]).

use std::collections::BTreeMap;

use deeptime_format::EventId;
use glam::{DQuat, DVec3};
use rayon::prelude::*;

use super::Env;
use super::events::{self, SUBDUCTION_ONSET};
use super::geometry::{Cover, Geo, Tree, cover, dijkstra};
use super::kinematics::{quat, tangent, to_group, to_world};
use super::pools::ISOSTATIC_FACTOR;
use super::state::{
    ARC, CONTINENTAL, Contact, DIVERGENT, EMERGED, HIDDEN, HOTSPOT, OROGENY_ANDEAN,
    OROGENY_HIMALAYAN, PlateState, SUBDUCTION_ZONE, TRENCH, pool,
};
use crate::error::CoreError;
use crate::world::WorldState;

/// Samples consulted around each sample.
const K_NEAR: usize = 8;
/// Overlapping samples needed to establish a contact; continental samples needed in the overlap
/// to seed a terrane, and on a front for a collision to begin (`shortening.rs`).
pub const MIN_CONTACT: u32 = 3;
/// Andean orogenies shorter than this are not written as events, Myr.
const OROGENY_MIN_MYR: f64 = 10.0;
/// Uplift below this (m/Myr) does not reset the orogeny age.
const OROGENY_MIN_M_PER_MYR: f64 = 5.0;

/// A sample lying inside another plate's surface.
#[derive(Clone, Copy, Debug)]
pub struct Overlap {
    pub i: u32,
    /// The group whose triangle holds it.
    pub group: u32,
    pub cover: Cover,
}

/// Finds every overlap (parallel over samples).
pub fn detect(state: &PlateState, geo: &Geo, pos: &[DVec3], tree: &Tree) -> Vec<Overlap> {
    let rots: Vec<DQuat> = state.groups.iter().map(|g| quat(g.rot)).collect();
    (0..state.samples.len())
        .into_par_iter()
        .filter_map(|i| {
            let pi = state.plate_of(i);
            let mut tried = [u32::MAX; K_NEAR];
            let mut nt = 0;
            for (j, _) in tree.nearest_n(pos[i], K_NEAR) {
                let g = state.samples[j as usize].group;
                if state.groups[g as usize].plate == pi || tried[..nt].contains(&g) {
                    continue;
                }
                tried[nt] = g;
                nt += 1;
                if let Some(c) = cover(geo, state, g, rots[g as usize], pos[i], j) {
                    return Some(Overlap {
                        i: i as u32,
                        group: g,
                        cover: c,
                    });
                }
            }
            None
        })
        .collect()
}

#[derive(Default)]
struct Agg {
    count: u32,
    sum: DVec3,
    /// Per side (0 = lower plate id, 1 = higher): samples seen, continental ones, freshly sutured
    /// ones, oceanic age sum.
    seen: [u32; 2],
    cont: [u32; 2],
    locked: [u32; 2],
    age: [f64; 2],
}

/// What `resolve` found this step.
pub struct Resolved {
    /// Per contact (by index): the subducting plate's continental samples in the overlap that are
    /// not freshly sutured, sorted, each once. They seed the terrane when the contact's collision
    /// fires (`shortening::advance`).
    pub seeds: BTreeMap<usize, Vec<u32>>,
}

/// Establishes contacts and their polarity, hides subducted samples, lists the continental crust
/// forced under, and emits subduction_onset events. `step` is the number of this step.
pub fn resolve(
    state: &mut PlateState,
    world: &mut WorldState,
    overlaps: &[Overlap],
    pos: &[DVec3],
    env: &Env,
) -> Result<Resolved, CoreError> {
    let (step, t_myr) = (env.step, env.t_myr);
    // Measurement only: which samples were already hidden last step (for `Contact::cont_in`).
    let was_hidden: Option<Vec<bool>> = state
        .dynamics
        .is_some()
        .then(|| state.samples.iter().map(|s| s.is_hidden()).collect());
    for s in &mut state.samples {
        s.flags &= !(HIDDEN | TRENCH | ARC | HOTSPOT | DIVERGENT | SUBDUCTION_ZONE | EMERGED);
    }
    let mut aggs: BTreeMap<(u32, u32), Agg> = BTreeMap::new();
    for o in overlaps {
        let a = state.plate_of(o.i as usize);
        let b = state.groups[o.group as usize].plate;
        let key = (a.min(b), a.max(b));
        let (sa, sb) = if a == key.0 { (0, 1) } else { (1, 0) };
        let e = aggs.entry(key).or_default();
        e.count += 1;
        e.sum += pos[o.i as usize];
        for (side, idx) in [(sa, o.i), (sb, o.cover.major())] {
            let s = &state.samples[idx as usize];
            e.seen[side] += 1;
            if s.kind == CONTINENTAL {
                e.cont[side] += 1;
                if locked(s, t_myr, env) {
                    e.locked[side] += 1;
                }
            } else {
                e.age[side] += s.age_myr(t_myr);
            }
        }
    }
    let confirm_samples = MIN_CONTACT.max(state.n / 10_000);
    let mut index: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for (k, c) in state.contacts.iter().enumerate() {
        index.insert((c.under.min(c.over), c.under.max(c.over)), k);
    }
    for (&(lo, hi), agg) in &aggs {
        let centroid = agg.sum.normalize_or(DVec3::Z);
        let frac = |s: usize| f64::from(agg.cont[s]) / f64::from(agg.seen[s].max(1));
        let lock = |s: usize| f64::from(agg.locked[s]) / f64::from(agg.seen[s].max(1));
        // Polarity reversal: continental crust arriving on the subducting side of an oceanic
        // overriding plate stops that subduction and starts the opposite one (Cortial: "an oceanic
        // plate always plunges beneath a continental plate"; without it a continent at a junction
        // of three trenches is handed round them for ever).
        if let Some(&k) = index.get(&(lo, hi)) {
            let c = &state.contacts[k];
            let under_side = usize::from(c.under != lo);
            let (u, o) = (under_side, 1 - under_side);
            if agg.count >= MIN_CONTACT
                && ((frac(u) >= REVERSAL_UNDER && frac(o) < REVERSAL_OVER)
                    || (lock(u) >= REVERSAL_UNDER && lock(o) < REVERSAL_OVER))
            {
                reverse(state, world, env, k)?;
            }
        }
        let k = match index.get(&(lo, hi)) {
            Some(&k) => k,
            None => {
                if agg.count < MIN_CONTACT {
                    continue;
                }
                let mean_age = |s: usize| {
                    let oceanic = agg.seen[s] - agg.cont[s];
                    if oceanic == 0 {
                        0.0
                    } else {
                        agg.age[s] / f64::from(oceanic)
                    }
                };
                let (c0, c1) = (frac(0) >= 0.5, frac(1) >= 0.5);
                let under_lo = match (c0, c1) {
                    (false, true) => true,
                    (true, false) => false,
                    (false, false) => mean_age(0) >= mean_age(1),
                    (true, true) => {
                        let (n0, n1) = (
                            state.plates[lo as usize].samples,
                            state.plates[hi as usize].samples,
                        );
                        n0 <= n1
                    }
                };
                let (under, over) = if under_lo { (lo, hi) } else { (hi, lo) };
                let setting = u8::from(c0) + u8::from(c1);
                let conv = env.norm.speed_mm_yr(
                    DVec3::from_array(state.plates[under as usize].omega)
                        - DVec3::from_array(state.plates[over as usize].omega),
                    centroid,
                );
                let slab_age = mean_age(if under_lo { 0 } else { 1 });
                state.contacts.push(Contact {
                    under,
                    over,
                    onset_myr: t_myr,
                    onset_step: step,
                    onset_event: None,
                    onset_slab_age_myr: slab_age,
                    onset_convergence_mm_yr: conv,
                    setting,
                    last_step: step,
                    samples: agg.count,
                    collision: None,
                    speed_count_km: 0.0,
                    max_uplift_mm_yr: 0.0,
                    andean: false,
                    centroid: centroid.to_array(),
                    cont_in: 0,
                });
                let k = state.contacts.len() - 1;
                index.insert((lo, hi), k);
                k
            }
        };
        let c = &mut state.contacts[k];
        c.last_step = step;
        c.samples = agg.count;
        c.centroid = centroid.to_array();
        if c.onset_event.is_none()
            && step - c.onset_step >= CONFIRM_STEPS
            && agg.count >= confirm_samples
        {
            let conv = env.norm.speed_mm_yr(
                DVec3::from_array(state.plates[c.under as usize].omega)
                    - DVec3::from_array(state.plates[c.over as usize].omega),
                centroid,
            );
            if conv >= CONFIRM_MM_YR {
                confirm(state, world, env, k)?;
            }
        }
    }

    let mut seeds: BTreeMap<usize, Vec<u32>> = BTreeMap::new();
    for o in overlaps {
        let a = state.plate_of(o.i as usize);
        let b = state.groups[o.group as usize].plate;
        let Some(&k) = index.get(&(a.min(b), a.max(b))) else {
            continue;
        };
        if state.contacts[k].last_step != step {
            continue;
        }
        if a == state.contacts[k].under {
            let s = &mut state.samples[o.i as usize];
            s.flags |= HIDDEN;
            if let Some(was) = &was_hidden
                && s.kind == CONTINENTAL
                && !was[o.i as usize]
            {
                state.contacts[k].cont_in += 1;
            }
            let s = &state.samples[o.i as usize];
            if s.kind == CONTINENTAL && !locked(s, t_myr, env) {
                seeds.entry(k).or_default().push(o.i);
            }
        } else {
            let m = o.cover.major();
            let s = &state.samples[m as usize];
            if s.kind == CONTINENTAL && !locked(s, t_myr, env) {
                seeds.entry(k).or_default().push(m);
            }
        }
    }
    for list in seeds.values_mut() {
        list.sort_unstable();
        list.dedup();
    }
    Ok(Resolved { seeds })
}

/// The isostatic ceiling on uplift (design choice): crust at height z above sea level rises by
/// (1 − z/z_c) of the tectonic forcing, so repeated uplift approaches z_c slowly instead of
/// saturating it (thickened crust spreads and collapses; Cortial's erosion alone is too slow to
/// balance repeated collisions). `tectonics.uplift_ceiling_m`; 0 (the pipeline presets) is no
/// ceiling: there the surface stage spreads over-thick crust itself (`surface::spread`), and the
/// height of a range is not a number in the plate model.
pub fn ceiling(z: f64, zc: f64) -> f64 {
    if z <= 0.0 || zc <= 0.0 {
        1.0
    } else {
        (1.0 - z / zc).max(0.0)
    }
}

/// Freshly sutured continental crust (a collision orogeny younger than `collision_orogeny_myr`)
/// cannot be forced under again: it never seeds a collision, and a contact whose subducting side
/// is mostly such crust reverses. Without this a continent at a junction of three converging
/// continental plates is handed round them every few Myr.
pub(super) fn locked(s: &super::state::Sample, t_myr: f64, env: &Env) -> bool {
    s.orogeny == OROGENY_HIMALAYAN && t_myr - f64::from(s.orogeny_myr) < env.k.collision_orogeny_myr
}

/// A contact reverses when at least this fraction of its subducting side's crust in the overlap
/// is continental (or freshly sutured) …
const REVERSAL_UNDER: f64 = 0.5;
/// … and less than this fraction of its overriding side's.
const REVERSAL_OVER: f64 = 0.25;

/// Reverses contact `k`: the subducting plate becomes the overriding one. The old subduction zone
/// closes (with its Andean orogeny, if any, and its collision state) and a fresh contact starts
/// with the opposite polarity.
fn reverse(
    state: &mut PlateState,
    world: &mut WorldState,
    env: &Env,
    k: usize,
) -> Result<(), CoreError> {
    let old = state.contacts[k].clone();
    if let Some(onset) = old.onset_event
        && old.andean
        && env.t_myr - old.onset_myr >= OROGENY_MIN_MYR
    {
        let e = events::event(
            world,
            old.onset_myr,
            events::OROGENY_PHASE,
            DVec3::from_array(old.centroid),
        )
        .until(env.t_myr * 1.0e6)
        .magnitude((old.max_uplift_mm_yr / env.k.subduction_uplift_mm_yr) as f32)
        .param("orogeny", "andean")
        .param("plate", i64::from(old.over))
        .param("subducting_plate", i64::from(old.under))
        .param("max_uplift_mm_yr", old.max_uplift_mm_yr)
        .param("ongoing", 0i64)
        .caused_by(onset);
        world.emit(e)?;
    }
    super::shortening::end(state, k, super::dynamics::REVERSED);
    let c = &mut state.contacts[k];
    std::mem::swap(&mut c.under, &mut c.over);
    c.onset_myr = env.t_myr;
    c.onset_step = env.step;
    c.onset_event = None;
    c.setting = 1;
    c.speed_count_km = 0.0;
    c.cont_in = 0;
    c.max_uplift_mm_yr = 0.0;
    c.andean = false;
    state.counters.reversals += 1;
    Ok(())
}

/// Steps a contact must last before it is a subduction zone with an onset event …
pub const CONFIRM_STEPS: u64 = 3;
/// … while converging at least this fast (mm/yr) over at least max(3, n/10,000) samples.
pub const CONFIRM_MM_YR: f64 = 10.0;

/// Writes contact `k`'s subduction_onset event (dated to when the contact began) if it has none;
/// returns it.
pub fn confirm(
    state: &mut PlateState,
    world: &mut WorldState,
    env: &Env,
    k: usize,
) -> Result<EventId, CoreError> {
    if let Some(id) = state.contacts[k].onset_event {
        return Ok(id);
    }
    let c = &state.contacts[k];
    let e = events::event(
        world,
        c.onset_myr,
        SUBDUCTION_ONSET,
        DVec3::from_array(c.centroid),
    )
    .magnitude((c.onset_convergence_mm_yr / env.k.max_plate_speed_mm_yr) as f32)
    .param("subducting_plate", i64::from(c.under))
    .param("overriding_plate", i64::from(c.over))
    .param(
        "setting",
        ["ocean_ocean", "ocean_continent", "continent_continent"][c.setting as usize],
    )
    .param("convergence_mm_yr", c.onset_convergence_mm_yr)
    .param("slab_age_myr", c.onset_slab_age_myr);
    let id = world.emit(e)?;
    state.contacts[k].onset_event = Some(id);
    state.counters.subduction_onsets += 1;
    Ok(id)
}

/// One front sample and what it carries into the Dijkstra.
struct Source {
    /// Contact index.
    contact: usize,
    /// Relative velocity of the subducting plate at the front, mm/yr (mantle frame).
    rel: DVec3,
    conv: f64,
    obliquity_deg: f64,
    z_sub: f64,
    slab_age: f64,
    sediment: f64,
    /// The subducting crust at the front is oceanic (a continental arc's cordillera needs an
    /// oceanic slab; continental crust forced under builds the collision's own relief).
    oceanic_slab: bool,
}

/// Slab pull per plate, from its subduction fronts this step.
pub struct Pull {
    /// Σ c × q / |c × q| over the plate's front samples, weighted by front length in 1,000 km.
    pub axis: Vec<DVec3>,
    /// Front length, km (Earth-normalized).
    pub front_km: Vec<f64>,
}

/// Divergence flags, trenches, fronts, subduction uplift, arcs and slab pull.
pub fn fronts_and_uplift(state: &mut PlateState, geo: &Geo, pos: &[DVec3], env: &Env) -> Pull {
    let (step, t_myr) = (env.step, env.t_myr);
    let k = env.k;
    let norm = env.norm;
    let n = state.samples.len();
    let plate_of: Vec<u32> = (0..n).map(|i| state.plate_of(i)).collect();
    let omega: Vec<DVec3> = state
        .plates
        .iter()
        .map(|p| DVec3::from_array(p.omega))
        .collect();
    let rots: Vec<DQuat> = state.groups.iter().map(|g| quat(g.rot)).collect();
    let pts = geo.points();
    let mut active: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for (i, c) in state.contacts.iter().enumerate() {
        if c.last_step == step {
            active.insert((c.under, c.over), i);
        }
    }

    // Per sample: divergence, trench, and at most one front (the fastest-converging neighbour).
    struct Boundary {
        divergent: bool,
        trench: bool,
        front: Option<(u32, usize, DVec3, DVec3)>,
    }
    let samples = &state.samples;
    let boundary: Vec<Boundary> = (0..n)
        .into_par_iter()
        .map(|v| {
            let sv = &samples[v];
            let mut b = Boundary {
                divergent: false,
                trench: false,
                front: None,
            };
            if sv.is_hidden() {
                return b;
            }
            let pv = plate_of[v];
            let x = pos[v];
            let rot = rots[sv.group as usize];
            let mut best = 0.0;
            for (u, _) in geo.edges(v as u32) {
                let su = &samples[u as usize];
                if su.group == sv.group && su.is_hidden() {
                    b.trench = true;
                }
                let pu = plate_of[u as usize];
                if pu == pv {
                    continue;
                }
                let out = tangent(x, rot * (pts[u as usize] - pts[v])).normalize_or_zero();
                let rel = norm.velocity_mm_yr(omega[pu as usize] - omega[pv as usize], x);
                let along = rel.dot(out);
                if along > 0.0 {
                    b.divergent = true;
                } else if let Some(&c) = active.get(&(pu, pv))
                    && -along > best
                {
                    best = -along;
                    b.front = Some((u, c, rel, out));
                }
            }
            b
        })
        .collect();

    let mut sources: Vec<(u32, f64, u32)> = Vec::new();
    let mut info: Vec<Source> = Vec::new();
    let mut pull = vec![DVec3::ZERO; state.plates.len()];
    let mut front_len = vec![0.0; state.plates.len()];
    let front_km = norm.km(norm.spacing_rad());
    for (v, b) in boundary.iter().enumerate() {
        let s = &mut state.samples[v];
        if b.divergent {
            s.flags |= DIVERGENT;
        }
        if b.trench {
            s.flags |= TRENCH;
        }
    }
    for (v, b) in boundary.iter().enumerate() {
        if let Some((u, c, rel, out)) = b.front {
            let su = state.samples[u as usize];
            let conv = rel.length();
            let obliquity = if conv > 0.0 {
                (rel.dot(-out) / conv).clamp(-1.0, 1.0).acos().to_degrees()
            } else {
                0.0
            };
            sources.push((v as u32, 0.0, info.len() as u32));
            info.push(Source {
                contact: c,
                rel,
                conv,
                obliquity_deg: obliquity,
                z_sub: f64::from(su.elevation_m),
                slab_age: su.age_myr(t_myr),
                sediment: f64::from(su.pools[pool::SEDIMENT]),
                oceanic_slab: su.kind != CONTINENTAL,
            });
            let under = state.contacts[c].under as usize;
            front_len[under] += front_km;
            let cu = DVec3::from_array(state.plates[under].centroid);
            let axis = cu.cross(pos[v]);
            if axis.length_squared() > 1e-18 {
                pull[under] += axis.normalize() * (front_km / 1000.0);
            }
        }
    }
    let rs = norm.rad(k.subduction_distance_km);
    let (dist, label) = if sources.is_empty() {
        (vec![f64::INFINITY; n], vec![u32::MAX; n])
    } else {
        let s = &state.samples;
        dijkstra(
            geo,
            &sources,
            |a, b| s[a as usize].group == s[b as usize].group && !s[b as usize].is_hidden(),
            rs,
        )
    };

    let dt_yr = env.dt_yr;
    let u0_m = k.subduction_uplift_mm_yr * 1.0e-3 * dt_yr;
    let arc_m = k.arc_uplift_mm_yr * 1.0e-3 * dt_yr;
    let cord_m = k.cordillera_uplift_mm_yr * 1.0e-3 * dt_yr;
    let (zt, zc) = (k.trench_elevation_m, k.uplift_reference_m);
    let z_ceiling = k.uplift_ceiling_m;
    let x0 = k.subduction_peak_km / k.subduction_distance_km;
    let v0 = k.max_plate_speed_mm_yr;
    let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
    let orogeny_m = OROGENY_MIN_M_PER_MYR * env.dt_myr;
    let collision_myr = k.collision_orogeny_myr;
    let info = &info;
    let effects: Vec<Option<(usize, f64, bool)>> = state
        .samples
        .par_iter_mut()
        .enumerate()
        .map(|(i, s)| {
            let d = dist[i];
            if !d.is_finite() || s.is_hidden() {
                return None;
            }
            let src = &info[label[i] as usize];
            s.flags |= SUBDUCTION_ZONE;
            let d_km = norm.km(d);
            let x = (d_km / k.subduction_distance_km).clamp(0.0, 1.0);
            let f = if x <= x0 {
                smooth(x / x0)
            } else {
                1.0 - smooth((x - x0) / (1.0 - x0))
            };
            let g = (src.conv / v0).min(1.0);
            let zn = ((src.z_sub - zt) / (zc - zt)).clamp(0.0, 1.0);
            let h = zn * zn;
            let z = f64::from(s.elevation_m);
            let mut dz = u0_m * f * g * h * ceiling(z, z_ceiling);
            let w_arc = (-((d_km - k.arc_distance_km) / k.arc_width_km).powi(2)).exp();
            if z < 0.0 {
                dz += arc_m * w_arc * g;
            }
            // The arc's own range (P1-27): continental crust over the arc thickens by arc
            // magmatism and shortening and stands up as a cordillera, so the cones rise from a
            // range (the Andes, the Cascades) rather than from a coastal plain; the root that
            // holds it up (Airy, `pools::ISOSTATIC_FACTOR` times the uplift) joins the crust.
            if cord_m > 0.0 && s.kind == CONTINENTAL && src.oceanic_slab {
                let w = (-((d_km - k.arc_distance_km) / k.cordillera_width_km).powi(2)).exp();
                // Thickened arc crust spreads under its own weight: the range builds towards
                // `cordillera_height_m` and no higher (the Altiplano and the Puna stop near 4 km).
                let room = if z <= 0.0 {
                    1.0
                } else {
                    (1.0 - z / k.cordillera_height_m).max(0.0)
                };
                let up = cord_m * w * g * room;
                dz += up;
                let root = (up * ISOSTATIC_FACTOR) as f32;
                s.pools[pool::FELSIC_PLUTONIC] += 0.6 * root;
                s.pools[pool::METAMORPHIC] += 0.4 * root;
            }
            s.elevation_m = (z + dz).min(zc) as f32;
            // Arc magma and the deposit accumulators.
            let m = k.arc_magma_m_per_myr * env.dt_myr * w_arc * g;
            if m > 0.0 {
                let before = s.arc_continental_m + s.arc_oceanic_m;
                if s.kind == CONTINENTAL {
                    s.pools[pool::FELSIC_PLUTONIC] += (0.7 * m) as f32;
                    s.pools[pool::FELSIC_VOLCANIC] += (0.3 * m) as f32;
                    s.arc_continental_m += m as f32;
                } else {
                    s.pools[pool::MAFIC_VOLCANIC] += (0.5 * m) as f32;
                    s.pools[pool::FELSIC_VOLCANIC] += (0.25 * m) as f32;
                    s.pools[pool::FELSIC_PLUTONIC] += (0.25 * m) as f32;
                    s.arc_oceanic_m += m as f32;
                }
                let ox = (0.25
                    + 0.5 * (src.slab_age / 150.0).min(1.0)
                    + 0.25 * (src.sediment / 500.0).min(1.0))
                .clamp(0.0, 1.0);
                let total = before + m as f32;
                s.oxidation = (s.oxidation * before + ox as f32 * m as f32) / total.max(1e-6);
                if w_arc > 0.3 {
                    s.flags |= ARC;
                }
            }
            s.convergence_mm_yr = src.conv as f32;
            s.obliquity_deg = src.obliquity_deg as f32;
            s.slab_age_myr = src.slab_age as f32;
            // Folds follow the relative motion, weighted by the deformation.
            let rot = rots[s.group as usize];
            let p = rot * pts[i];
            let fold = to_world(rot, s.fold) + tangent(p, src.rel).normalize_or_zero() * (f * g);
            s.fold = to_group(rot, tangent(p, fold));
            let andean = dz >= orogeny_m && (s.kind == CONTINENTAL || s.elevation_m > 0.0);
            if andean
                && (s.orogeny != OROGENY_HIMALAYAN
                    || t_myr - f64::from(s.orogeny_myr) > collision_myr)
            {
                s.orogeny = OROGENY_ANDEAN;
                s.orogeny_myr = t_myr as f32;
            }
            Some((
                src.contact,
                dz / dt_yr * 1.0e3,
                andean && s.kind == CONTINENTAL,
            ))
        })
        .collect();
    for (c, rate, andean) in effects.into_iter().flatten() {
        let contact = &mut state.contacts[c];
        contact.max_uplift_mm_yr = contact.max_uplift_mm_yr.max(rate);
        contact.andean |= andean;
    }
    Pull {
        axis: pull,
        front_km: front_len,
    }
}

/// Closes contacts unseen for `contact_timeout_steps` (or all of them when `all`), writing an
/// Andean orogeny_phase for those that uplifted continental crust and ending their collision
/// states (`dynamics::CLOSED`, or `OPEN` at the end of the run). Returns the closed contacts'
/// subducting plates with their onset events (the cause of a later plate death).
pub fn close_contacts(
    state: &mut PlateState,
    world: &mut WorldState,
    env: &Env,
    all: bool,
) -> Result<Vec<(u32, EventId)>, CoreError> {
    let (step, t_myr) = (env.step, env.t_myr);
    let timeout = u64::from(env.k.contact_timeout_steps);
    let mut closed = Vec::new();
    let mut keep = Vec::with_capacity(state.contacts.len());
    let alive: Vec<bool> = state.plates.iter().map(|p| p.alive).collect();
    for c in std::mem::take(&mut state.contacts) {
        let gone = !alive[c.under as usize] || !alive[c.over as usize];
        if !(all || gone || step.saturating_sub(c.last_step) > timeout) {
            keep.push(c);
            continue;
        }
        let last_myr = t_myr - env.dt_myr * step.saturating_sub(c.last_step) as f64;
        if let Some(col) = &c.collision {
            let reason = if all && !gone {
                super::dynamics::OPEN
            } else {
                super::dynamics::CLOSED
            };
            super::dynamics::record_parting(&mut state.dynamics, &c, col, reason);
        }
        let Some(onset) = c.onset_event else {
            continue;
        };
        if c.andean && last_myr - c.onset_myr >= OROGENY_MIN_MYR {
            let e = events::event(
                world,
                c.onset_myr,
                events::OROGENY_PHASE,
                DVec3::from_array(c.centroid),
            )
            .until(last_myr.max(c.onset_myr))
            .magnitude((c.max_uplift_mm_yr / env.k.subduction_uplift_mm_yr) as f32)
            .param("orogeny", "andean")
            .param("plate", i64::from(c.over))
            .param("subducting_plate", i64::from(c.under))
            .param("max_uplift_mm_yr", c.max_uplift_mm_yr)
            .param("ongoing", i64::from(all && !gone))
            .caused_by(onset);
            world.emit(e)?;
        }
        closed.push((c.under, onset));
    }
    state.contacts = keep;
    Ok(closed)
}

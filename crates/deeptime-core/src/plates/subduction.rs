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
//! `arc_distance_km` from the front) adds arc crust to the rock pools and the deposit
//! accumulators at the sourced rates (`island_arc_km3_per_km_myr` on oceanic crust,
//! `continental_arc_km3_per_km_myr` on continental crust; Stern & Scholl 2010), less the cones the
//! volcano stage built there (they reach the pools as volcanic rock first, so arc magma is counted
//! once), and the arc stands up by its thickness: an island arc on the ocean's height line
//! (`transition::held_floor_m`), a continental arc by its Airy change, towards and never past
//! the height its column holds (plate task 8; until then a height-only `arc_uplift_mm_yr`).
//! Island-arc crust becomes continental once it is `arc_mature_crust_km` thick
//! (`plates::finish_samples`). A wider band around the arc (`cordillera_width_km`) thickens
//! continental crust by the crust shortened into it at `cordillera_shortening_mm_yr`, taken from
//! the plate's continental crust behind the range (volume kept; until plate task 8 an uplift with
//! an Airy root made from nothing), slowing as the column's Airy height nears
//! `cordillera_height_m` (P1-27: the arc's own range, which h(z̃) of an abyssal slab leaves at a
//! few hundredths of a mm/yr). Trench samples (the subducting plate's
//! visible samples next to its own hidden ones) are pulled towards z_t and receive sediment
//! accretion (`pools.rs`). Slab pull turns the subducting plate's axis towards its fronts. Uplift
//! of crust above sea level is scaled by the isostatic ceiling (1 − z/z_c) ([`ceiling`]).

use std::collections::BTreeMap;

use deeptime_format::EventId;
use glam::{DQuat, DVec3};
use rayon::prelude::*;

use super::Env;
use super::collapse::{iso_height, toward};
use super::events::{self, SUBDUCTION_ONSET};
use super::geometry::{Cover, Geo, Tree, cover, dijkstra};
use super::kinematics::{quat, tangent, to_group, to_world};
use super::pools::isostatic_change_m;
use super::state::{
    ARC, CONTINENTAL, Contact, DIVERGENT, EMERGED, FORELAND, HIDDEN, HOTSPOT, OROGENY_ANDEAN,
    OROGENY_HIMALAYAN, PlateState, SUBDUCTION_ZONE, TRENCH, pool,
};
use super::transition::{continental_m, crust_per_metre, crystalline_m, held_floor_m};
use crate::error::CoreError;
use crate::world::WorldState;

/// Samples consulted around each sample.
const K_NEAR: usize = 8;
/// Overlapping samples needed to establish a contact; continental samples needed in the overlap
/// to seed a terrane, and on a front for a collision to begin (`shortening.rs`).
pub const MIN_CONTACT: u32 = 3;
/// Andean orogenies shorter than this are not written as events, Myr.
const OROGENY_MIN_MYR: f64 = 10.0;
/// Uplift below this (m/Myr) does not reset the orogeny age (here and in a collision belt,
/// `belts.rs`).
pub(super) const OROGENY_MIN_M_PER_MYR: f64 = 5.0;

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
        s.flags &=
            !(HIDDEN | TRENCH | ARC | HOTSPOT | DIVERGENT | SUBDUCTION_ZONE | EMERGED | FORELAND);
        // The collisions' footprint is stamped again each step they have a front (`belts.rs`).
        // `stacked` stays while the crust stays under (below).
        s.collision_mm_yr = 0.0;
        s.shortening_km = 0.0;
    }
    // Ocean floor at a plate contact with at least a normal continental column's crystalline
    // crust (T_n, 33 km, the transition's PLT-261) is continental crust, whatever stacked it there
    // (a resampling's accretion, a slab's accreted crust): it collides, stacks into a belt or
    // moves across as a terrane like any continental crust, instead of going under as ocean
    // floor and coming back in one lump on the column above it at the next resampling (ledger:
    // PLT-289; build log 2026-10-18 "Resampling lumps"). Crust that thick is far past the ~15 km
    // that jams a subduction zone (Cloos 1993, PLT-047); the threshold is T_n and not 15 km so
    // that the transition's ocean side (up to about 20 km, the thinned margin, ocean floor by
    // design) keeps its type: at 15 km two traced runs turned ten times as many samples, the
    // taper forming again behind each converted ring, and grew the continents (build log).
    let buoyant_m = continental_m();
    for o in overlaps {
        for idx in [o.i, o.cover.major()] {
            let s = &mut state.samples[idx as usize];
            if s.kind != CONTINENTAL && crystalline_m(&s.pools) >= buoyant_m {
                s.kind = CONTINENTAL;
                state.counters.buoyant += 1;
            }
        }
    }
    // A collision's own belt and wedge are stamped as a collision orogeny each step (`belts.rs`):
    // that crust is not locked against the collision that stamps it.
    let since: BTreeMap<(u32, u32), f64> = state
        .contacts
        .iter()
        .filter_map(|c| {
            let col = c.collision.as_ref()?;
            Some(((c.under.min(c.over), c.under.max(c.over)), col.onset_myr))
        })
        .collect();
    let mut aggs: BTreeMap<(u32, u32), Agg> = BTreeMap::new();
    for o in overlaps {
        let a = state.plate_of(o.i as usize);
        let b = state.groups[o.group as usize].plate;
        let key = (a.min(b), a.max(b));
        let (sa, sb) = if a == key.0 { (0, 1) } else { (1, 0) };
        let own = since.get(&key).copied();
        let e = aggs.entry(key).or_default();
        e.count += 1;
        e.sum += pos[o.i as usize];
        for (side, idx) in [(sa, o.i), (sb, o.cover.major())] {
            let s = &state.samples[idx as usize];
            e.seen[side] += 1;
            if s.kind == CONTINENTAL {
                e.cont[side] += 1;
                if locked_against(s, t_myr, env, own) {
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
        let own = state.contacts[k].collision.as_ref().map(|c| c.onset_myr);
        if a == state.contacts[k].under {
            let s = &mut state.samples[o.i as usize];
            s.flags |= HIDDEN;
            // ledger: PLT-276 (crust a collision stacks is not accreted again at a resampling).
            // Once stacked it stays so while it is under (ledger: PLT-291): the belt counted it
            // when it crossed the front, and a collision that ends (a parting, a transfer, a weld)
            // does not make it crust to accrete again.
            s.stacked |= own.is_some() && s.kind == CONTINENTAL;
            if let Some(was) = &was_hidden
                && s.kind == CONTINENTAL
                && !was[o.i as usize]
            {
                state.contacts[k].cont_in += 1;
            }
            let s = &state.samples[o.i as usize];
            if s.kind == CONTINENTAL && !locked_against(s, t_myr, env, own) {
                seeds.entry(k).or_default().push(o.i);
            }
        } else {
            let m = o.cover.major();
            let s = &state.samples[m as usize];
            if s.kind == CONTINENTAL && !locked_against(s, t_myr, env, own) {
                seeds.entry(k).or_default().push(m);
            }
        }
    }
    for s in &mut state.samples {
        s.stacked &= s.is_hidden();
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

/// [`locked`], except against the collision that stamped it: crust stamped since `own` (the onset
/// of the contact's own collision, if it is in one) is that collision's belt or wedge, which goes
/// on converging and being stamped (`belts.rs`).
pub(super) fn locked_against(
    s: &super::state::Sample,
    t_myr: f64,
    env: &Env,
    own: Option<f64>,
) -> bool {
    locked(s, t_myr, env) && own.is_none_or(|t0| f64::from(s.orogeny_myr) < t0 - 1e-6)
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
    .magnitude((c.onset_convergence_mm_yr / env.k.convergence_ref_mm_yr) as f32)
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

/// World global: arc crust added to continental crust over the run, km³ (Earth-normalized).
pub const ARC_CONTINENTAL_GLOBAL: &str = "plates/arc_continental_km3";
/// World global: arc crust added to oceanic samples (island arcs) over the run, km³.
pub const ARC_OCEANIC_GLOBAL: &str = "plates/arc_oceanic_km3";
/// World global: the visible continental crust's volume after the last step, km³ (Earth-normalized).
pub const CONTINENTAL_VOLUME_GLOBAL: &str = "plates/continental_volume_km3";
/// World global: oceanic subduction fronts over the run, km × Myr, where the overriding crust at
/// the front is continental (a continental arc; measurement: the arcs' rate per km of arc).
pub const ARC_FRONT_CONTINENTAL_GLOBAL: &str = "plates/arc_front_continental_km_myr";
/// … and where it is oceanic (an island arc), km × Myr.
pub const ARC_FRONT_OCEANIC_GLOBAL: &str = "plates/arc_front_oceanic_km_myr";
/// World global: crust shortened into the arcs' cordilleras over the run, km³ (moved, not made).
pub const CORDILLERA_GLOBAL: &str = "plates/cordillera_shortened_km3";

/// What `fronts_and_uplift` measured this step.
pub struct Pull {
    /// Each plate's oceanic subduction front length, km (Earth-normalized; the dynamics record's).
    pub front_km: Vec<f64>,
    /// Arc crust this step added to continental crust, km³ (Earth-normalized; measurement, for
    /// the crust budget against Stern & Scholl 2010).
    pub arc_continental_km3: f64,
    /// Arc crust this step added to oceanic samples (island arcs), km³ (Earth-normalized).
    pub arc_oceanic_km3: f64,
    /// Oceanic fronts this step under continental overriding crust, km × Myr …
    pub front_continental_km_myr: f64,
    /// … and under oceanic overriding crust.
    pub front_oceanic_km_myr: f64,
    /// Crust shortened into cordilleras this step, km³ (moved from behind the ranges).
    pub cordillera_km3: f64,
}

/// One subducting-side sample's effect on its contact and on the crust budget.
struct Effect {
    contact: usize,
    rate_mm_yr: f64,
    /// The sample is continental and rose fast enough for an Andean orogeny.
    andean: bool,
    /// Arc crust added to a continental sample, m.
    continental_m: f64,
    /// Arc crust added to an oceanic sample, m.
    oceanic_m: f64,
    /// Crust shortened into a cordillera sample, m.
    cordillera_m: f64,
}

/// The volcanic share of an arc's crust: continental arcs about 1 in 8 (an intrusive to extrusive
/// ratio of about 7:1, Stern & Scholl 2010 for Haschke & Günther's 35 km³/km/Myr), island arcs
/// 1 in 6 (White et al. 2006's median ratio, 5:1). ledger: PLT-046
const CONTINENTAL_VOLCANIC_SHARE: f64 = 1.0 / 8.0;
const ISLAND_VOLCANIC_SHARE: f64 = 1.0 / 6.0;
/// Of an island arc's volcanic rock, the mafic share (the rest felsic volcanic; the model's ratio
/// before plate task 8, kept). ledger: PLT-046
const ISLAND_MAFIC_SHARE: f64 = 2.0 / 3.0;

/// One sample's part in its contact's arc and cordillera this step (before any change).
#[derive(Clone, Copy, Default)]
struct Plan {
    /// Arc crust at full share, m (oceanic slabs only).
    magma_m: f64,
    /// Crust the cordillera would take in, m (range samples), or 0.
    demand_m: f64,
    /// The sample is continental crust behind the range that the shortening draws on.
    source: bool,
}

/// Per contact: what its arcs and cordillera add up to this step.
#[derive(Clone, Copy, Default)]
struct Budget {
    volcanic_m: f64,
    plutonic_m: f64,
    cones_m: f64,
    demand_m: f64,
    supply_m: f64,
    supply_pools: [f64; pool::COUNT],
}

/// Divergence flags, trenches, fronts, subduction uplift and arcs; each plate's oceanic front length.
///
/// `cones` (per sample, felsic and mafic volcanic rock, m) is the rock the volcano stage's cones
/// handed to the pools at this step's feedback: an arc counts its felsic cones inside its own
/// rate and clears them (`hotspots::step` counts what is left).
pub fn fronts_and_uplift(
    state: &mut PlateState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    cones: &mut [[f64; 2]],
) -> Pull {
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
    let mut front_len = vec![0.0; state.plates.len()];
    let (mut front_continental, mut front_oceanic) = (0.0_f64, 0.0_f64);
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
            // Only an oceanic slab counts as a front that pulls: continental crust at the front
            // is buoyant, and where it comes in the collision resists instead (`forces.rs`).
            // ledger: PLT-251
            if su.kind != CONTINENTAL {
                let under = state.contacts[c].under as usize;
                front_len[under] += front_km;
                if state.samples[v].kind == CONTINENTAL {
                    front_continental += front_km * env.dt_myr;
                } else {
                    front_oceanic += front_km * env.dt_myr;
                }
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
    let (zt, zc) = (k.trench_elevation_m, k.uplift_reference_m);
    let z_ceiling = k.uplift_ceiling_m;
    let x0 = k.subduction_peak_km / k.subduction_distance_km;
    let v0 = k.convergence_ref_mm_yr; // ledger: PLT-037
    let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
    let orogeny_m = OROGENY_MIN_M_PER_MYR * env.dt_myr;
    let collision_myr = k.collision_orogeny_myr;
    let gravity = env.gravity;
    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
    let (z_f, per_m) = (k.oceanic_floor_elevation_m, crust_per_metre(env));
    // The arcs' rates per km of arc, spread over the arc band (whose integral across the arc is
    // √π · `arc_width_km`): crust per step at the axis at v_0, m.
    let band_km = std::f64::consts::PI.sqrt() * k.arc_width_km;
    let island_axis_m = k.island_arc_km3_per_km_myr / band_km * 1000.0 * env.dt_myr; // ledger: PLT-045
    let continental_axis_m = k.continental_arc_km3_per_km_myr / band_km * 1000.0 * env.dt_myr; // ledger: PLT-285
    // The cordillera: pure shear of the range at `cordillera_shortening_mm_yr`, the strain spread
    // over its Gaussian band (integral √π · `cordillera_width_km`), fed by the crust behind the
    // range (from twice the band's half-width beyond the arc to r_s), which shortens at the same
    // rate and gives the volume. ledger: PLT-048
    let shorten_km = k.cordillera_shortening_mm_yr * 1.0e-3 * dt_yr / 1000.0;
    let strain_axis = shorten_km / (std::f64::consts::PI.sqrt() * k.cordillera_width_km);
    let range_edge_km = k.arc_distance_km + 2.0 * k.cordillera_width_km;
    let source_strain = shorten_km / (k.subduction_distance_km - range_edge_km).max(1.0);
    let info = &info;
    let contacts = &state.contacts;

    // What each sample would add or give, and each contact's totals.
    let plans: Vec<Plan> = state
        .samples
        .par_iter()
        .enumerate()
        .map(|(i, s)| {
            let d = dist[i];
            if !d.is_finite() || s.is_hidden() {
                return Plan::default();
            }
            let src = &info[label[i] as usize];
            // Arcs and their ranges are an oceanic slab's: continental crust forced under at a
            // collision front makes no arc (plate task 8).
            if !src.oceanic_slab {
                return Plan::default();
            }
            let d_km = norm.km(d);
            let g = (src.conv / v0).min(1.0);
            let continental = s.kind == CONTINENTAL;
            let axis = if continental {
                continental_axis_m
            } else {
                island_axis_m
            };
            let mut plan = Plan {
                magma_m: axis * w_arc_of(d_km, k) * g,
                ..Plan::default()
            };
            if shorten_km > 0.0 && continental && plate_of[i] == contacts[src.contact].over {
                if d_km < range_edge_km {
                    let w = (-((d_km - k.arc_distance_km) / k.cordillera_width_km).powi(2)).exp();
                    // Thickened arc crust spreads under its own weight: the range thickens until
                    // its column's Airy height reaches `cordillera_height_m` and no further (the
                    // Altiplano and the Puna stop near 4 km).
                    let iso = iso_height(&s.pools, z_0, gravity);
                    let room = (1.0 - iso.max(0.0) / k.cordillera_height_m).max(0.0);
                    plan.demand_m = f64::from(s.thickness_m) * strain_axis * w * g * room;
                } else {
                    plan.source = true;
                }
            }
            plan
        })
        .collect();
    let mut budget = vec![Budget::default(); contacts.len()];
    for (i, plan) in plans.iter().enumerate() {
        let s = &state.samples[i];
        if !dist[i].is_finite() || s.is_hidden() {
            continue;
        }
        let src = &info[label[i] as usize];
        if !src.oceanic_slab {
            continue;
        }
        let b = &mut budget[src.contact];
        let share = if s.kind == CONTINENTAL {
            CONTINENTAL_VOLCANIC_SHARE
        } else {
            ISLAND_VOLCANIC_SHARE
        };
        b.volcanic_m += share * plan.magma_m;
        b.plutonic_m += (1.0 - share) * plan.magma_m;
        if let Some(c) = cones.get_mut(i) {
            b.cones_m += c[0];
            c[0] = 0.0;
        }
        b.demand_m += plan.demand_m;
        if plan.source {
            b.supply_m += s.pools.iter().map(|&x| f64::from(x.max(0.0))).sum::<f64>();
            for (a, &x) in b.supply_pools.iter_mut().zip(&s.pools) {
                *a += f64::from(x.max(0.0));
            }
        }
    }
    // Per contact: the arc's own volcanic and plutonic factors once its felsic cones are counted
    // (the cones are its extrusive rock; beyond that share they count against the intrusive), the
    // strain of the crust behind the range and the share of the range's demand it meets.
    // ledger: PLT-286
    let factors: Vec<(f64, f64, f64, f64)> = budget
        .iter()
        .map(|b| {
            let qv = if b.volcanic_m > 0.0 {
                (b.volcanic_m - b.cones_m).max(0.0) / b.volcanic_m
            } else {
                1.0
            };
            let over = (b.cones_m - b.volcanic_m).max(0.0);
            let qp = if b.plutonic_m > 0.0 {
                (b.plutonic_m - over).max(0.0) / b.plutonic_m
            } else {
                1.0
            };
            let (phi, met) = if b.demand_m > 0.0 && b.supply_m > 0.0 {
                let phi = (b.demand_m / b.supply_m).min(source_strain);
                (phi, phi * b.supply_m / b.demand_m)
            } else {
                (0.0, 0.0)
            };
            (qv, qp, phi, met)
        })
        .collect();
    let budget = &budget;
    let factors = &factors;
    let plans = &plans;

    let effects: Vec<Option<Effect>> = state
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
            let continental = s.kind == CONTINENTAL;
            let (mut added_continental, mut added_oceanic, mut cordillera) =
                (0.0_f64, 0.0_f64, 0.0_f64);
            // Cortial's h(z̃) uplift is the oceanic slab's: where continental crust comes in, the
            // collision belt (`belts.rs`) builds the relief from the crust shortened.
            let lifted = if src.oceanic_slab {
                z + u0_m * f * g * h * ceiling(z, z_ceiling)
            } else {
                z
            };
            let before = s.pools;
            let plan = plans[i];
            let (qv, qp, phi, met) = factors[src.contact];
            // The arc's own range (P1-27): the crust shortened into it, with the mix of the crust
            // behind the range it came from (each pool kept), and that crust, thinned by the
            // same strain.
            if plan.demand_m > 0.0 && met > 0.0 {
                let b = &budget[src.contact];
                let gain = plan.demand_m * met;
                for (p, &x) in s.pools.iter_mut().zip(&b.supply_pools) {
                    *p += (gain * x / b.supply_m) as f32;
                }
                cordillera = gain;
            } else if plan.source && phi > 0.0 {
                for p in s.pools.iter_mut() {
                    *p -= (f64::from(p.max(0.0)) * phi) as f32;
                }
            }
            // Arc magma and the deposit accumulators.
            let m = plan.magma_m;
            if m > 0.0 {
                let before_m = s.arc_continental_m + s.arc_oceanic_m;
                if continental {
                    let v = m * CONTINENTAL_VOLCANIC_SHARE * qv;
                    let p = m * (1.0 - CONTINENTAL_VOLCANIC_SHARE) * qp;
                    s.pools[pool::FELSIC_PLUTONIC] += p as f32;
                    s.pools[pool::FELSIC_VOLCANIC] += v as f32;
                    s.arc_continental_m += m as f32;
                    added_continental += v + p;
                } else {
                    let v = m * ISLAND_VOLCANIC_SHARE * qv;
                    let p = m * (1.0 - ISLAND_VOLCANIC_SHARE) * qp;
                    s.pools[pool::MAFIC_VOLCANIC] += (v * ISLAND_MAFIC_SHARE) as f32;
                    s.pools[pool::FELSIC_VOLCANIC] += (v * (1.0 - ISLAND_MAFIC_SHARE)) as f32;
                    s.pools[pool::FELSIC_PLUTONIC] += p as f32;
                    // The arc's crust counts whole, its cones included, towards its maturity.
                    s.arc_oceanic_m += m as f32;
                    added_oceanic += v + p;
                }
                let ox = (0.25
                    + 0.5 * (src.slab_age / 150.0).min(1.0)
                    + 0.25 * (src.sediment / 500.0).min(1.0))
                .clamp(0.0, 1.0);
                let total = before_m + m as f32;
                s.oxidation = (s.oxidation * before_m + ox as f32 * m as f32) / total.max(1e-6);
                if w_arc_of(d_km, k) > 0.3 {
                    s.flags |= ARC;
                }
            }
            s.thickness_m = s.pools.iter().sum();
            // The arc and the range stand up by their thickness, towards and never past the
            // height their column holds: continental crust by its Airy change, an island arc on
            // the ocean's height line (`transition.rs`), the floor it cools towards
            // (`pools::surface_step`).
            let raised = if continental {
                let change = isostatic_change_m(&before, &s.pools, gravity);
                toward(lifted, change, iso_height(&s.pools, z_0, gravity))
            } else {
                let after = crystalline_m(&s.pools);
                let change = (after - crystalline_m(&before)) / per_m;
                toward(
                    lifted,
                    change,
                    held_floor_m(after, f64::INFINITY, z_f, per_m),
                )
            };
            s.elevation_m = raised.min(zc) as f32;
            let dz = f64::from(s.elevation_m) - z;
            s.convergence_mm_yr = src.conv as f32;
            s.obliquity_deg = src.obliquity_deg as f32;
            s.slab_age_myr = src.slab_age as f32;
            // Folds follow the relative motion, weighted by the deformation.
            let rot = rots[s.group as usize];
            let p = rot * pts[i];
            let fold = to_world(rot, s.fold) + tangent(p, src.rel).normalize_or_zero() * (f * g);
            s.fold = to_group(rot, tangent(p, fold));
            let andean = dz >= orogeny_m && (continental || s.elevation_m > 0.0);
            if andean
                && (s.orogeny != OROGENY_HIMALAYAN
                    || t_myr - f64::from(s.orogeny_myr) > collision_myr)
            {
                s.orogeny = OROGENY_ANDEAN;
                s.orogeny_myr = t_myr as f32;
            }
            Some(Effect {
                contact: src.contact,
                rate_mm_yr: dz / dt_yr * 1.0e3,
                andean: andean && continental,
                continental_m: added_continental,
                oceanic_m: added_oceanic,
                cordillera_m: cordillera,
            })
        })
        .collect();
    let (mut arc_continental_m, mut arc_oceanic_m, mut cordillera_m) = (0.0, 0.0, 0.0);
    for e in effects.into_iter().flatten() {
        let contact = &mut state.contacts[e.contact];
        contact.max_uplift_mm_yr = contact.max_uplift_mm_yr.max(e.rate_mm_yr);
        contact.andean |= e.andean;
        arc_continental_m += e.continental_m;
        arc_oceanic_m += e.oceanic_m;
        cordillera_m += e.cordillera_m;
    }
    let area = norm.sample_area_km2();
    Pull {
        front_km: front_len,
        arc_continental_km3: arc_continental_m * area / 1000.0,
        arc_oceanic_km3: arc_oceanic_m * area / 1000.0,
        front_continental_km_myr: front_continental,
        front_oceanic_km_myr: front_oceanic,
        cordillera_km3: cordillera_m * area / 1000.0,
    }
}

/// The arc band's weight at `d_km` from the front (Gaussian, centred `arc_distance_km` from it).
fn w_arc_of(d_km: f64, k: &crate::params::TectonicParams) -> f64 {
    (-((d_km - k.arc_distance_km) / k.arc_width_km).powi(2)).exp()
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
        // A collision keeps its contact while its front lives (`shortening::GRACE_STEPS`): two
        // plates that have stalled against each other overlap no more, and must still weld.
        let colliding = c.collision.as_ref().is_some_and(|col| {
            step.saturating_sub(col.last_step) <= super::shortening::GRACE_STEPS
        });
        if !(all || gone || (step.saturating_sub(c.last_step) > timeout && !colliding)) {
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
            // Years (the event's start is in years too: `events::event` converts); this was Myr
            // until 2026-10-05, which a run continued past the present exposed.
            .until(last_myr.max(c.onset_myr) * 1.0e6)
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

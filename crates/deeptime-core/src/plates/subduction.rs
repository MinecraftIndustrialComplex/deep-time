// SPDX-License-Identifier: LGPL-3.0-or-later
//! Convergent boundaries: overlap detection, subduction polarity, trenches, subduction uplift and
//! arc magmatism (Cortial et al. 2019, §4.1 and §6 "Plate boundary tracking").
//!
//! **Overlaps.** Every step, each sample looks at its 8 nearest samples (all plates, current
//! positions); for each other plate among them it asks whether it lies inside one of that plate's
//! lattice triangles (three vertices of one rigid group, located in the group's frame). Such a
//! sample overlaps the other plate: the plates interpenetrate there. With `outline_cover` (on by
//! default, PLT-553) it asks whether it lies inside that plate's outline instead, the zero line
//! of the interpolated distance the resampling gives the ground by (`level_set::cover`): a
//! triangle with corners of other groups holds it on the group's side of that line.
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
//! continental crust by the crust shortened into it at `cordillera_shortening_mm_yr` (Earth's
//! mean over its continental arcs, however fast the plates converge: PLT-048, PLT-320), taken from
//! the plate's continental crust behind the range (volume kept; until plate task 8 an uplift with
//! an Airy root made from nothing), slowing as the column's Airy height nears
//! `cordillera_height_m` (P1-27: the arc's own range, which h(z̃) of an abyssal slab leaves at a
//! few hundredths of a mm/yr). Trench samples (the subducting plate's
//! visible samples next to its own hidden ones) are pulled towards z_t and receive sediment
//! accretion (`pools.rs`). Slab pull turns the subducting plate's axis towards its fronts. Uplift
//! of crust above sea level is scaled by the isostatic ceiling (1 − z/z_c) ([`ceiling`]).
//!
//! **Subduction erosion** ([`erode_forearcs`], PLT-441, PLT-442; on by default since 2026-10-23,
//! off with `subduction_erosion_km3_per_km_myr` 0). Where an oceanic slab comes in fast across the
//! front (orthogonal convergence at least `subduction_erosion_min_mm_yr`) under little sediment
//! (its sediment and sedimentary rock under `subduction_erosion_max_sediment_m`), the margin is
//! erosive (Clift & Vannucchi 2004: erosion where convergence passes 6 cm/yr and the cover is under
//! 1 km): the front takes `subduction_erosion_km3_per_km_myr` of the overriding plate's crust per
//! km of front to the mantle. It takes it from the forearc, the samples the front reaches nearer
//! than the arc's axis, the nearest first and each down to an ocean's column: the plate model's
//! fronts stay where they are in their plate's frame, so the forearc the trench eats into is the
//! front's own row and then the row behind it, as the trench's retreat landward does (Clift &
//! Vannucchi's 1 to 5 km/Myr). The column loses each pool in proportion (the wedge goes whole) and
//! sinks by its Airy change (an island arc's forearc on the ocean's height line).
//!
//! **The trench's retreat** (`tectonics.erosion_retreat`, the docking lane's stage S1, PLT-522,
//! PLT-523; off by default). With it the erosive margin truncates its overriding plate instead
//! (Scholl & von Huene 2010: "Subduction erosion truncates the margin (migrates the trench inboard)
//! at ~2.5 km/Ma"): each erosive front sample goes whole, its column booked to the mantle, at the
//! chance that moves the trench inboard by [`RETREAT_KM_PER_MYR`] ([`retreat_chance`]), and the
//! sample leaves its plate for the slab's (a twin of its group in the subducting plate, so it
//! stays where it is this step), as the slab's own surface beside it with no sediment. The front,
//! the forearc and the arc then stand one row inboard. The volume is the retreat times the column
//! (on the normal continental column about Clift & Vannucchi's global 90 km³/km/Myr), and the area
//! goes with it; PLT-441's volume rate and its floor at an ocean's column are off while it is on.

use std::collections::BTreeMap;

use deeptime_format::EventId;
use glam::{DQuat, DVec3};
use rayon::prelude::*;

use super::Env;
use super::collapse::{iso_height, toward};
use super::events::{self, SUBDUCTION_ONSET};
use super::geometry::{Cover, Geo, Tree, cover, dijkstra_parts};
use super::kinematics::{quat, tangent, to_group, to_world};
use super::level_set;
use super::pools::isostatic_change_m;
use super::shortening::{boundary_normal, front_share_rad, is_transform};
use super::state::{
    ARC, CONTINENTAL, Contact, DIVERGENT, EMERGED, FORELAND, HIDDEN, HOTSPOT, OROGENY_ANDEAN,
    OROGENY_HIMALAYAN, OneWay, PlateState, SUBDUCTION_ZONE, TRENCH, pool,
};
use super::transition::{continental_m, crust_per_metre, crystalline_m, held_floor_m, oceanic_m};
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

/// The step's overlaps as `env`'s switches have them ([`detect`], with the outline when
/// `tectonics.outline_cover` is on; ledger PLT-553).
pub fn overlaps(
    state: &PlateState,
    geo: &Geo,
    pos: &[DVec3],
    tree: &Tree,
    env: &Env,
) -> Vec<Overlap> {
    let outline = env
        .k
        .outline_cover
        .then(|| level_set::distances_now(state, geo, env.norm.spacing_rad()));
    detect(state, geo, pos, tree, outline.as_deref())
}

/// Finds every overlap (parallel over samples). With `outline` (each sample's distance from its
/// group's outline, `level_set::distances_now`; `tectonics.outline_cover`, ledger PLT-553) a
/// sample overlaps another plate where it lies inside that plate's outline
/// (`level_set::cover`); without, only inside a lattice triangle whose corners are all one of
/// that plate's groups (`geometry::cover`).
pub fn detect(
    state: &PlateState,
    geo: &Geo,
    pos: &[DVec3],
    tree: &Tree,
    outline: Option<&[f64]>,
) -> Vec<Overlap> {
    let outline = outline.filter(|o| o.len() == state.samples.len());
    let rots: Vec<DQuat> = state.groups.iter().map(|g| quat(g.rot)).collect();
    let plate_of: Vec<u32> = (0..state.samples.len())
        .into_par_iter()
        .map(|i| state.plate_of(i))
        .collect();
    let inland = Voxels::new(pos, &plate_of);
    (0..state.samples.len())
        .into_par_iter()
        .filter_map(|i| {
            let pi = plate_of[i];
            // Every sample its nearest K_NEAR could hold is its own plate's: nothing to try
            // (P1-19; see `Voxels`).
            if inland.only(pos[i], pi) {
                return None;
            }
            let mut tried = [u32::MAX; K_NEAR];
            let mut nt = 0;
            for (j, _) in tree.nearest_n(pos[i], K_NEAR) {
                let g = state.samples[j as usize].group;
                if state.groups[g as usize].plate == pi || tried[..nt].contains(&g) {
                    continue;
                }
                tried[nt] = g;
                nt += 1;
                let rot = rots[g as usize];
                let found = match outline {
                    Some(phi) => level_set::cover(geo, state, phi, g, rot, pos[i], j),
                    None => cover(geo, state, g, rot, pos[i], j),
                };
                if let Some(c) = found {
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

/// A grid of cubes over the unit sphere, about four sample spacings a side: per cube, its
/// samples and the one plate that every sample of the 5 × 5 × 5 cubes around it belongs to (if
/// they share one). A sample whose own cube holds at least [`K_NEAR`] samples has its `K_NEAR`
/// nearest within the cube's diagonal, √3 sides, so inside those 125 cubes, whose faces lie at
/// least two sides away. When they are all its own plate's, so are its `K_NEAR` nearest, ties
/// and all, and [`detect`] would skip every one of them: it can skip the query (P1-19: every
/// sample asked the k-d tree, though only those near another plate find anything).
struct Voxels {
    /// Cubes per axis, and the inverse of a cube's side.
    m: usize,
    inv_h: f64,
    count: Vec<u32>,
    block: Vec<u32>,
}

/// [`Voxels`]' marks: no sample, and samples of two plates or more.
const VOXEL_EMPTY: u32 = u32::MAX;
const VOXEL_MIXED: u32 = u32::MAX - 1;

/// The plate two sets of samples share ([`VOXEL_EMPTY`] when both are empty, [`VOXEL_MIXED`]
/// when they hold two plates).
fn voxel_merge(a: u32, b: u32) -> u32 {
    if a == VOXEL_EMPTY {
        b
    } else if b == VOXEL_EMPTY || a == b {
        a
    } else {
        VOXEL_MIXED
    }
}

impl Voxels {
    fn new(pos: &[DVec3], plate_of: &[u32]) -> Voxels {
        let h = 4.0 * (4.0 * std::f64::consts::PI / pos.len().max(1) as f64).sqrt();
        let m = (2.0 / h).ceil() as usize + 1;
        let mut v = Voxels {
            m,
            inv_h: 1.0 / h,
            count: vec![0; m * m * m],
            block: Vec::new(),
        };
        let cells: Vec<usize> = pos.par_iter().map(|&p| v.cell(p)).collect();
        let mut own = vec![VOXEL_EMPTY; m * m * m];
        for (i, &c) in cells.iter().enumerate() {
            v.count[c] += 1;
            own[c] = voxel_merge(own[c], plate_of[i]);
        }
        // The 5-wide window along each axis in turn.
        for stride in [1, m, m * m] {
            own = (0..own.len())
                .into_par_iter()
                .map(|c| {
                    let a = (c / stride) % m;
                    let base = c - a * stride;
                    (a.saturating_sub(2)..=(a + 2).min(m - 1))
                        .fold(VOXEL_EMPTY, |x, b| voxel_merge(x, own[base + b * stride]))
                })
                .collect();
        }
        v.block = own;
        v
    }

    fn cell(&self, p: DVec3) -> usize {
        let f = |x: f64| (((x + 1.0) * self.inv_h) as usize).min(self.m - 1);
        (f(p.z) * self.m + f(p.y)) * self.m + f(p.x)
    }

    /// Whether the [`K_NEAR`] nearest samples to the sample at `q` are all of `plate`'s.
    fn only(&self, q: DVec3, plate: u32) -> bool {
        let c = self.cell(q);
        self.count[c] >= K_NEAR as u32 && self.block[c] == plate
    }
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
    /// … and ones with their plate's continental crust within the arc's distance
    /// ([`continent_within`]; counted only with `reversal_by_arc_crust`, PLT-371).
    near: [u32; 2],
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
    geo: &Geo,
    overlaps: &[Overlap],
    pos: &[DVec3],
    env: &Env,
) -> Result<Resolved, CoreError> {
    let (step, t_myr) = (env.step, env.t_myr);
    // Measurement only: which samples were already hidden last step (for `Contact::cont_in` and
    // the crust shown again, `shown.rs`).
    let was_hidden: Option<Vec<bool>> = super::shown::begin(state);
    // Measurement only: the pairs whose contact reverses this step (`shown.rs`).
    let mut reversed_now: Vec<(u32, u32)> = Vec::new();
    // ledger: PLT-371 (the reversal judged by the crust the arc stands on), PLT-370 (the reach):
    // the arc's footing is the crust visible as the step starts, since the flags are set again
    // below.
    let by_arc = env.k.reversal_by_arc_crust;
    let hidden_before: Vec<bool> = if by_arc {
        state.samples.iter().map(|s| s.is_hidden()).collect()
    } else {
        Vec::new()
    };
    // One-way subduction (ledger: PLT-542): ocean floor that went down a trench stays under until
    // the resampling consumes it (held, and still hidden: a collision that brought it up let it
    // go); off, the hiding is found again each step. The trench's reach is the forearc and arc
    // the last step's fronts left (`fronts_and_uplift`), read here before they are found again.
    let mut one_way: Option<OneWay> = if env.k.one_way_subduction {
        let n = state.samples.len();
        let mut ow = state.one_way.take().unwrap_or_default();
        if ow.held.len() != n {
            ow.held = vec![false; n];
        }
        if ow.forearc.len() != n {
            ow.forearc = vec![false; n];
        }
        Some(ow)
    } else {
        state.one_way = None;
        None
    };
    for (i, s) in state.samples.iter_mut().enumerate() {
        let keep = one_way.as_ref().is_some_and(|ow| ow.holds(i, s));
        s.flags &=
            !(HIDDEN | TRENCH | ARC | HOTSPOT | DIVERGENT | SUBDUCTION_ZONE | EMERGED | FORELAND);
        if let Some(ow) = one_way.as_mut() {
            ow.held[i] = keep;
            if keep {
                s.flags |= HIDDEN;
            }
        }
        // The collisions' footprint is stamped again each step they have a front (`belts.rs`).
        // `stacked` stays while the crust stays under (below).
        s.collision_mm_yr = 0.0;
        s.shortening_km = 0.0;
        s.carrier = 0;
        s.carrier_lithosphere_km = 0.0;
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
                // The floor's own crust stays under the crust on top (ledger: PLT-331).
                s.make_continental();
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
    let arc_steps = (env.k.arc_distance_km / env.norm.km(env.norm.spacing_rad())).floor() as u32;
    // Whether each overlap's two samples (its own, its cover's) would hold an arc on continental
    // crust: a search per sample that reads the state only, so all of them in parallel before the
    // tally (it took most of this function's time on one thread; the speed work of 2026-10-28).
    let near: Vec<[bool; 2]> = if by_arc {
        let st: &PlateState = state;
        overlaps
            .par_iter()
            .map(|o| {
                [o.i, o.cover.major()]
                    .map(|idx| continent_within(st, geo, idx, arc_steps, |v| hidden_before[v]))
            })
            .collect()
    } else {
        Vec::new()
    };
    for (k, o) in overlaps.iter().enumerate() {
        let a = state.plate_of(o.i as usize);
        let b = state.groups[o.group as usize].plate;
        let key = (a.min(b), a.max(b));
        let (sa, sb) = if a == key.0 { (0, 1) } else { (1, 0) };
        let own = since.get(&key).copied();
        let e = aggs.entry(key).or_default();
        e.count += 1;
        e.sum += pos[o.i as usize];
        for (t, (side, idx)) in [(sa, o.i), (sb, o.cover.major())].into_iter().enumerate() {
            let s = &state.samples[idx as usize];
            e.seen[side] += 1;
            if by_arc && near[k][t] {
                e.near[side] += 1;
            }
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
        // of three trenches is handed round them for ever). ledger: PLT-056. By default (since
        // 2026-10-23, V1 gap 2) the overriding side is judged by the crust its arc stands on
        // (Stern 2004, `reversal_by_arc_crust`, ledger: PLT-371, which has what it does to the
        // land); with the switch off, by its crust in the overlap (PLT-056's old judgement), often
        // an oceanic fringe in front of its continent, which fired about 17 times per 100 Myr
        // against Crameri et al. (2020)'s 4 in about 105 Myr.
        if let Some(&k) = index.get(&(lo, hi)) {
            let c = &state.contacts[k];
            let under_side = usize::from(c.under != lo);
            let (u, o) = (under_side, 1 - under_side);
            let near = |s: usize| f64::from(agg.near[s]) / f64::from(agg.seen[s].max(1));
            if agg.count >= MIN_CONTACT
                && reverses([frac(u), frac(o)], near(o), [lock(u), lock(o)], by_arc)
            {
                reverse(state, world, env, k)?;
                reversed_now.push((lo, hi));
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
                // A subduction initiation's contact takes the polarity of the slab planted under
                // its fracture, which the fracture's ages set (ledger: PLT-655).
                let planted = if env.k.initiation_polarity_by_age {
                    planted_polarity(state, lo, hi)
                } else {
                    None
                };
                let under_lo = planted.unwrap_or_else(|| match (c0, c1) {
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
                });
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
                    reversed: false,
                    carried: false,
                    arc_onset_myr: None,
                    arc_intra_oceanic: false,
                });
                let k = state.contacts.len() - 1;
                index.insert((lo, hi), k);
                k
            }
        };
        let c = &mut state.contacts[k];
        c.last_step = step;
        // Overlap seen again: the contact is active by it, not carried (ledger: PLT-550).
        c.carried = false;
        c.samples = agg.count;
        c.centroid = centroid.to_array();
        if c.onset_event.is_none()
            && step - c.onset_step >= confirm_steps(env.dt_myr)
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
            // Ocean floor under the overriding plate's forearc or arc went down its trench and
            // stays down (one-way subduction, ledger: PLT-542). Continental crust is too buoyant
            // to stay down (Cloos 1993, PLT-047): it collides, or shows again where the
            // overriding plate leaves it, as before.
            if let Some(ow) = one_way.as_mut()
                && s.kind != CONTINENTAL
                && ow.forearc[o.cover.major() as usize]
            {
                ow.held[o.i as usize] = true;
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
    // Held ocean floor that turned continental this step (`make_continental` above) is let go:
    // from the next step on it is judged as continental crust.
    if let Some(mut ow) = one_way {
        for (h, s) in ow.held.iter_mut().zip(&state.samples) {
            *h &= s.kind != CONTINENTAL;
        }
        state.one_way = Some(ow);
    }
    // Measurement only (`shown.rs`): why each sample hidden as the step began shows again.
    if let Some(was) = &was_hidden {
        let mut cover = vec![super::shown::LEFT as u8; state.samples.len()];
        for o in overlaps {
            let i = o.i as usize;
            if !was[i] || state.samples[i].is_hidden() {
                continue;
            }
            let a = state.plate_of(i);
            let b = state.groups[o.group as usize].plate;
            let key = (a.min(b), a.max(b));
            let c = match index.get(&key) {
                Some(&k) if state.contacts[k].last_step == step => {
                    if reversed_now.contains(&key) {
                        super::shown::REVERSAL
                    } else {
                        super::shown::ON_TOP
                    }
                }
                _ => super::shown::UNSEEN,
            };
            cover[i] = cover[i].min(c as u8);
        }
        super::shown::end_resolve(state, was, &cover);
    }
    for list in seeds.values_mut() {
        list.sort_unstable();
        list.dedup();
    }
    Ok(Resolved { seeds })
}

/// The isostatic ceiling on uplift (design choice): crust at height z above the sea (the callers
/// pass the elevation less the step's sea) rises by (1 − z/z_c) of the tectonic forcing, so
/// repeated uplift approaches z_c slowly instead of saturating it (thickened crust spreads and
/// collapses; Cortial's erosion alone is too slow to balance repeated collisions).
/// `tectonics.uplift_ceiling_m`; 0 (the pipeline presets) is no ceiling: there the surface stage
/// spreads over-thick crust itself (`surface::spread`), and the height of a range is not a number
/// in the plate model. ledger: PLT-041
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

/// Whether visible continental crust of sample `i`'s plate lies within `steps` lattice steps of it
/// through that plate's visible samples (itself included; `hidden` says which are not visible):
/// an arc behind a front there would stand on continental crust. The boundary mix measures the
/// same distance (`dynamics`, PLT-370).
/// The arc-front counters measure with it, and the polarity reversal judges the overriding side
/// by it with `reversal_by_arc_crust` (on by default since 2026-10-23; plates.md, "The plate
/// geometry" and "The crust flux into collision fronts", 2026-10-21; ledger: PLT-371).
fn continent_within(
    state: &PlateState,
    geo: &Geo,
    i: u32,
    steps: u32,
    hidden: impl Fn(usize) -> bool,
) -> bool {
    let p = state.plate_of(i as usize);
    let mut seen = vec![i];
    let mut frontier = vec![i];
    for depth in 0..=steps {
        let mut next = Vec::new();
        for &v in &frontier {
            let s = &state.samples[v as usize];
            if s.kind == CONTINENTAL && !hidden(v as usize) {
                return true;
            }
            if depth == steps {
                continue;
            }
            for (u, _) in geo.edges(v) {
                if !seen.contains(&u) && state.plate_of(u as usize) == p && !hidden(u as usize) {
                    seen.push(u);
                    next.push(u);
                }
            }
        }
        frontier = next;
    }
    false
}

/// The polarity of a new contact between a plate and its parent from the slab already planted
/// between them (a subduction initiation's, `rifting::initiate`, ledger PLT-655 and PLT-270):
/// whether `lo` goes under; `None` where neither plate is the other's parent, no slab or a slab
/// each way hangs between them.
fn planted_polarity(state: &PlateState, lo: u32, hi: u32) -> Option<bool> {
    let parent = |a: u32| state.plates.get(a as usize).and_then(|p| p.parent);
    if parent(lo) != Some(hi) && parent(hi) != Some(lo) {
        return None;
    }
    let hangs = |under: u32, over: u32| {
        state
            .slabs
            .iter()
            .any(|s| s.under == under && s.over == over)
    };
    match (hangs(lo, hi), hangs(hi, lo)) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        _ => None,
    }
}

/// A contact reverses when at least this fraction of its subducting side's crust in the overlap
/// is continental (or freshly sutured) …
const REVERSAL_UNDER: f64 = 0.5;
/// … and less than this fraction of its overriding side's.
const REVERSAL_OVER: f64 = 0.25;

/// Whether a contact reverses its polarity (ledger: PLT-056, PLT-371), from the shares of its
/// overlap samples on the subducting side `[0]` and the overriding side `[1]`: `cont` continental,
/// `lock` locked against it, `over_near` the overriding side's with their plate's continental
/// crust within the arc's distance ([`continent_within`]). Buoyant crust arriving on the
/// subducting side reverses an oceanic overriding plate; with `by_arc` the overriding side is
/// oceanic only where its arc would stand on ocean floor, so a continent behind an oceanic fringe
/// goes on as a collision ("polarity reversal ... is triggered by buoyant crust entering the
/// trench"; against a continent, "It takes less work to deform large parts of continental Asia
/// than it does to start a new subduction zone in the Indian Plate", Stern 2004).
fn reverses(cont: [f64; 2], over_near: f64, lock: [f64; 2], by_arc: bool) -> bool {
    let over_oceanic = if by_arc {
        over_near < REVERSAL_OVER
    } else {
        cont[1] < REVERSAL_OVER
    };
    (cont[0] >= REVERSAL_UNDER && over_oceanic)
        || (lock[0] >= REVERSAL_UNDER && lock[1] < REVERSAL_OVER)
}

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
    // The new polarity's slab is new: its arc begins when it passes incipient (PLT-669).
    c.arc_onset_myr = None;
    c.arc_intra_oceanic = false; // ledger: PLT-760
    // A reversal a geologist could map is one of a subduction zone that was confirmed, counted
    // when the new polarity is confirmed in turn (`confirm`; measurement, ledger PLT-514).
    c.reversed = old.onset_event.is_some();
    state.counters.reversals += 1;
    Ok(())
}

/// How long a contact must last before it is a subduction zone with an onset event, Myr (3 steps
/// at the 2-Myr step) …
// ledger: PLT-058 (game-choice, P1-03: long enough that a flicker at a transform boundary never
// gets one).
pub const CONFIRM_MYR: f64 = 6.0;

/// The steps a contact must last at a step of `dt_myr`: the fewest that span [`CONFIRM_MYR`].
pub fn confirm_steps(dt_myr: f64) -> u64 {
    (CONFIRM_MYR / dt_myr).ceil() as u64
}
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
    if std::mem::take(&mut state.contacts[k].reversed) {
        state.counters.reversals_confirmed += 1;
    }
    Ok(id)
}

/// One front sample and what it carries into the Dijkstra.
struct Source {
    /// Contact index.
    contact: usize,
    /// Relative velocity of the subducting plate at the front, mm/yr (mantle frame).
    rel: DVec3,
    /// The convergence the subduction uplift reads and the front stamps, mm/yr: the whole
    /// relative speed, or `normal` with `DEEPTIME_ARC_NORMAL=1` (PLT-312).
    conv: f64,
    /// The convergence normal to the front, mm/yr (PLT-312's component; 0 where the plates part
    /// along the normal): the arcs' rate reads it (PLT-665).
    normal: f64,
    obliquity_deg: f64,
    z_sub: f64,
    slab_age: f64,
    sediment: f64,
    /// The subducting crust at the front is oceanic (a continental arc's cordillera needs an
    /// oceanic slab; continental crust forced under builds the collision's own relief).
    oceanic_slab: bool,
    /// The front is incipient (PLT-610): its slab has not yet sunk far enough to melt an arc, or
    /// its oceanic slab comes in too slowly for a slab to form (PLT-681).
    incipient: bool,
}

/// World global: arc crust added to continental crust over the run, km³ (Earth-normalized).
pub const ARC_CONTINENTAL_GLOBAL: &str = "plates/arc_continental_km3";
/// World global: arc crust added to oceanic samples (island arcs) over the run, km³.
pub const ARC_OCEANIC_GLOBAL: &str = "plates/arc_oceanic_km3";
/// World global: the visible continental crust's volume after the last step, km³ (Earth-normalized).
pub const CONTINENTAL_VOLUME_GLOBAL: &str = "plates/continental_volume_km3";
/// World global: oceanic subduction fronts over the run, km × Myr, whose arc stands on continental
/// crust: the overriding plate's visible continental crust lies within `arc_distance_km` of the
/// front (a continental arc; measurement: the arcs' rate per km of arc). Until 2026-10-21 the
/// crust of the front sample itself, often an oceanic fringe in front of the continent.
/// ledger: PLT-370
pub const ARC_FRONT_CONTINENTAL_GLOBAL: &str = "plates/arc_front_continental_km_myr";
/// … and the rest, whose arc stands on ocean floor (an island arc), km × Myr.
pub const ARC_FRONT_OCEANIC_GLOBAL: &str = "plates/arc_front_oceanic_km_myr";
/// World globals: oceanic fronts over the run weighted by the arcs' convergence factor, km × Myr,
/// with the full relative speed and with the convergence normal to the front ([`Pull`]).
pub const FRONT_G_FULL_GLOBAL: &str = "plates/arc_front_g_full_km_myr";
pub const FRONT_G_NORMAL_GLOBAL: &str = "plates/arc_front_g_normal_km_myr";
/// The same two over the fronts under continental overriding crust, km × Myr.
pub const FRONT_G_CONTINENTAL_FULL_GLOBAL: &str = "plates/arc_front_g_full_continental_km_myr";
pub const FRONT_G_CONTINENTAL_NORMAL_GLOBAL: &str = "plates/arc_front_g_normal_continental_km_myr";
/// World globals: the continental and the island arcs' fronts over the run (as
/// [`ARC_FRONT_CONTINENTAL_GLOBAL`] and [`ARC_FRONT_OCEANIC_GLOBAL`]) weighted by the convergence
/// normal to the front, km × Myr × mm/yr; over those globals, the mean v_n the arcs read
/// (measurement; ledger: PLT-665).
pub const ARC_FRONT_NORMAL_CONTINENTAL_GLOBAL: &str = "plates/arc_front_normal_continental";
pub const ARC_FRONT_NORMAL_OCEANIC_GLOBAL: &str = "plates/arc_front_normal_oceanic";
/// World globals: the continental and the island arcs' fronts over the run (as
/// [`ARC_FRONT_CONTINENTAL_GLOBAL`] and [`ARC_FRONT_OCEANIC_GLOBAL`]) whose arc is younger than
/// [`ISLAND_ARC_BURST_MYR`] (its age from its contact's `arc_onset_myr`), km × Myr; whatever
/// `arc_front_loaded` (measurement: the length the early pulse acts on, against S&S's arcs).
/// ledger: PLT-670
pub const ARC_FRONT_YOUNG_CONTINENTAL_GLOBAL: &str = "plates/arc_front_young_continental_km_myr";
pub const ARC_FRONT_YOUNG_OCEANIC_GLOBAL: &str = "plates/arc_front_young_oceanic_km_myr";

/// Front-loaded island arcs (`tectonics.arc_front_loaded`): the early pulse, km³ of new crust per
/// km of arc per Myr, at any convergence … Stern & Scholl 2010, p. 11: intra-oceanic arc growth
/// is "approximately 100 km3/km-million years (500 km3/million years for first 10 million years
/// plus 30 km3/million years for 60 million years thereafter)"; the pulse is the arc's first
/// years of slab sinking and forearc spreading, not its convergence. ledger: PLT-668
pub const ISLAND_ARC_BURST_KM3_PER_KM_MYR: f64 = 500.0;
/// … for the arc's first this many Myr from its onset (Stern & Scholl's "first 10 million
/// years") …
pub const ISLAND_ARC_BURST_MYR: f64 = 10.0;
/// … and the mature arc's rate at Earth's mean orthogonal convergence of intra-oceanic arcs
/// (`island_arc_convergence_ref_mm_yr`, PLT-666), scaled by its own (S&S's "30 km3/million
/// years").
pub const ISLAND_ARC_MATURE_KM3_PER_KM_MYR: f64 = 30.0;

/// An island arc's crust per km of arc per Myr at its reference, and the factor on it, at arc age
/// `age_myr` (from its contact's `arc_onset_myr`; `None` before the onset) and orthogonal
/// convergence `v_n_mm_yr`: the rate is their product. Off (`arc_front_loaded`), the one rate
/// `island_arc_km3_per_km_myr` × v_n / `island_arc_convergence_ref_mm_yr` (PLT-045, PLT-665,
/// PLT-666) at every age; on, [`ISLAND_ARC_BURST_KM3_PER_KM_MYR`] with no factor while the arc
/// is younger than [`ISLAND_ARC_BURST_MYR`], then [`ISLAND_ARC_MATURE_KM3_PER_KM_MYR`] ×
/// v_n / the reference. The continental arcs keep their one rate either way: Stern & Scholl
/// measured the pulse on intra-oceanic arcs. ledger: PLT-668
pub fn island_arc_law(
    k: &crate::params::TectonicParams,
    age_myr: Option<f64>,
    v_n_mm_yr: f64,
) -> (f64, f64) {
    let g = v_n_mm_yr / k.island_arc_convergence_ref_mm_yr;
    if !k.arc_front_loaded {
        return (k.island_arc_km3_per_km_myr, g);
    }
    match age_myr {
        Some(a) if a < ISLAND_ARC_BURST_MYR => (ISLAND_ARC_BURST_KM3_PER_KM_MYR, 1.0),
        _ => (ISLAND_ARC_MATURE_KM3_PER_KM_MYR, g),
    }
}

/// World global: the oceanic fronts over the run by each front sample's share of the straight
/// front, km × Myr, whatever `front_share` (measurement: the `arc_front_*` globals over this are
/// the lattice's count of a front against its straight length while it is off). ledger: PLT-530
pub const ARC_FRONT_STRAIGHT_GLOBAL: &str = "plates/arc_front_straight_km_myr";
/// World global: the oceanic slabs' arc bands over the run by their length, km × Myr
/// ([`Pull::arc_band_km_myr`]; measurement: the arcs' crust is made over this length, against the
/// straight front above). ledger: PLT-554
pub const ARC_BAND_GLOBAL: &str = "plates/arc_band_km_myr";
/// World globals: the felsic cones the arcs counted inside their rates over the run, km³, on the
/// crust of the continental arcs' law and of the island arcs' ([`Pull::arc_cones_km3`]): with
/// [`ARC_CONTINENTAL_GLOBAL`] and [`ARC_OCEANIC_GLOBAL`], the arcs' whole rock (measurement;
/// ledger: PLT-761).
pub const ARC_CONES_CONTINENTAL_GLOBAL: &str = "plates/arc_cones_continental_km3";
pub const ARC_CONES_OCEANIC_GLOBAL: &str = "plates/arc_cones_oceanic_km3";
/// World global: the oceanic fronts past incipient of arcs that began intra-oceanic, km × Myr
/// ([`Pull::front_intra_oceanic_km_myr`]; measurement; ledger: PLT-760).
pub const ARC_FRONT_INTRA_OCEANIC_GLOBAL: &str = "plates/arc_front_intra_oceanic_km_myr";

/// Crust an island arc made: ocean floor, or continental crust made from ocean floor (an ocean
/// basement under it, PLT-331) whose own crystalline crust (its crystalline pools less the
/// basement) is at least half the island-arc magma it received (`arc_oceanic_m`): a matured
/// island arc (PLT-047), not a continent's own crust, a margin's thinned crust (PLT-661) or
/// a spread's receiver (PLT-662). ledger: PLT-760
pub fn island_arc_crust(s: &super::state::Sample) -> bool {
    if s.kind != CONTINENTAL {
        return true;
    }
    if s.basement() <= 0.0 {
        return false;
    }
    let own = s.own_pools();
    let crystalline: f32 = own[pool::METAMORPHIC..].iter().sum();
    // "At least half": the game-choice line between a matured arc's own crust and crust that
    // only took some island-arc magma (ledger: PLT-760).
    2.0 * s.arc_oceanic_m >= crystalline
}

/// Whether arc magma landing on `s` follows the island arcs' law (PLT-045, PLT-668) rather than
/// the continental arcs' (PLT-285): ocean floor always; with `island_arc_own_crust` also the
/// crust an intra-oceanic arc made ([`island_arc_crust`]) when the contact's arc began
/// intra-oceanic (`Contact::arc_intra_oceanic`). Stern & Scholl 2010 (p. 11) give the
/// front-loaded law for the intra-oceanic arcs they list, the Izu-Bonin-Mariana among them,
/// whose crust is far thicker than the 8 km that turns a column continental here (PLT-047).
/// ledger: PLT-760
pub fn arc_is_island(
    k: &crate::params::TectonicParams,
    intra_oceanic: bool,
    s: &super::state::Sample,
) -> bool {
    s.kind != CONTINENTAL || (k.island_arc_own_crust && intra_oceanic && island_arc_crust(s))
}
/// World global: crust shortened into the arcs' cordilleras over the run, km³ (moved, not made).
pub const CORDILLERA_GLOBAL: &str = "plates/cordillera_shortened_km3";

/// What `fronts_and_uplift` measured this step.
pub struct Pull {
    /// Each plate's oceanic subduction front length, km (Earth-normalized; the dynamics record's).
    pub front_km: Vec<f64>,
    /// This step's subduction zones: the overriding side's samples of the oceanic fronts past
    /// incipient (PLT-610, PLT-681), the fronts the plate check counts as subducting boundary
    /// against Bird 2003 (`boundary_subduction_km`). Fix C's active trenches: where subduction
    /// initiation may start and how long the planet's trench is (ledger: PLT-691, PLT-692).
    pub subducting_fronts: Vec<u32>,
    /// Each of `subducting_fronts` in order: the subducting side's sample across the front and the
    /// front's boundary normal (a unit tangent from the overriding plate towards it, PLT-310), for
    /// back-arc rifting and its trench-motion measures (ledger: PLT-710, PLT-716).
    pub subducting_detail: Vec<(u32, DVec3)>,
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
    /// Oceanic fronts this step weighted by the arcs' convergence factor min(v / v_0, 1), km × Myr,
    /// with v the full relative speed …
    pub front_g_full_km_myr: f64,
    /// … and with v the convergence normal to the front (measurement, either rule).
    pub front_g_normal_km_myr: f64,
    /// The same two over the fronts under continental overriding crust (the continental arcs and
    /// their cordilleras), km × Myr.
    pub front_g_continental_km_myr: [f64; 2],
    /// Oceanic fronts this step by their share of the straight front, km × Myr, whatever
    /// `front_share` (measurement; PLT-530).
    pub front_straight_km_myr: f64,
    /// `front_continental_km_myr` and `front_oceanic_km_myr` weighted by the convergence normal
    /// to the front, km × Myr × mm/yr: over them, the mean v_n of the continental and the island
    /// arcs' fronts (measurement; PLT-665).
    pub front_normal_km_myr: [f64; 2],
    /// `front_continental_km_myr` and `front_oceanic_km_myr` whose arc is younger than
    /// [`ISLAND_ARC_BURST_MYR`], km × Myr (measurement; PLT-670).
    pub front_young_km_myr: [f64; 2],
    /// The oceanic slabs' arc bands this step by their length, km × Myr: the band's weight summed
    /// over the samples its fronts reach, times a sample's area, over the band's integral across
    /// the arc (√π · `arc_width_km`); a straight front's band gives its length (measurement;
    /// ledger: PLT-554).
    pub arc_band_km_myr: f64,
    /// The volcano stage's felsic cones the arcs counted inside their rates this step (their
    /// extrusive rock, PLT-286), on the crust of the continental arcs' law and of the island
    /// arcs', km³ (Earth-normalized; measurement: `arc_*_km3` holds the arcs' rock less these).
    /// ledger: PLT-761
    pub arc_cones_km3: [f64; 2],
    /// Oceanic fronts past incipient whose contact's arc began intra-oceanic, km × Myr
    /// (measurement, the island arcs' front by their setting at onset; ledger: PLT-760).
    pub front_intra_oceanic_km_myr: f64,
    /// The step's erosive fronts and their forearcs ([`erode_forearcs`]); `None` while the rule is
    /// off.
    pub forearcs: Option<Forearcs>,
}

/// The step's erosive fronts (subduction erosion, [`erode_forearcs`]) and the distance field
/// their forearcs are found on.
pub struct Forearcs {
    /// Per front (the distance field's label): the crust its erosion takes this step, km³
    /// (Earth-normalized; 0 where the margin is not erosive).
    pub loss_km3: Vec<f64>,
    /// Per front: its overriding crust is continental (where the front sample's is).
    pub continental: Vec<bool>,
    /// Per front: the front sample (overriding plate) and the slab's sample across from it.
    pub front: Vec<[u32; 2]>,
    /// Per front: its length of front, km (a lattice spacing, or its share of the straight front
    /// with `front_share` on; PLT-530).
    pub front_km: Vec<f64>,
    /// The fronts' distance field ([`fronts_reach`]): distances (rad), labels, the reached samples.
    dist: Vec<f64>,
    label: Vec<u32>,
    reached: Vec<u32>,
    /// With `margin_slab_ends` (PLT-562): per sample, whether it stands over its front's slab
    /// (`over_the_slab`); a forearc is only there.
    over: Option<Vec<bool>>,
}

/// World global: the overriding plates' crust subduction erosion took to the mantle over the run,
/// km³ (Earth-normalized): continental crust (its basement left out) …
pub const FOREARC_CONTINENTAL_GLOBAL: &str = "plates/forearc_eroded_continental_km3";
/// … and the rest (ocean crust under it, island arcs' forearcs), km³.
pub const FOREARC_OTHER_GLOBAL: &str = "plates/forearc_eroded_other_km3";
/// World global: the erosive fronts over the run, km × Myr, under continental overriding crust …
pub const FOREARC_FRONT_CONTINENTAL_GLOBAL: &str = "plates/forearc_front_continental_km_myr";
/// … and under oceanic overriding crust.
pub const FOREARC_FRONT_OCEANIC_GLOBAL: &str = "plates/forearc_front_oceanic_km_myr";
/// World global: the erosion the forearcs had no crust above an ocean's column for, km³.
pub const FOREARC_SHORT_GLOBAL: &str = "plates/forearc_short_km3";
/// World global, with the trench's retreat on (PLT-523): the area its erosive margins consumed
/// over the run, km² (Earth-normalized), of continental crust …
pub const RETREAT_CONTINENTAL_GLOBAL: &str = "plates/forearc_retreat_continental_km2";
/// … and of other crust (island arcs' forearcs, ocean floor).
pub const RETREAT_OTHER_GLOBAL: &str = "plates/forearc_retreat_other_km2";

/// The trench's retreat under subduction erosion, km/Myr: Scholl & von Huene 2010, "Subduction
/// erosion truncates the margin (migrates the trench inboard) at ~2.5 km/Ma" (1 to 5); Clift &
/// Vannucchi 2004's Table 2, 0.9 to 4.7, median 3.0. ledger: PLT-522 (verified)
pub const RETREAT_KM_PER_MYR: f64 = 2.5;

/// The chance that an erosive front sample goes this step (PLT-523): the area the trench sweeps
/// over the sample's piece of front, `retreat_km` × `front_km`, over the sample's area, at most 1.
/// The expected area consumed per km of front is then the retreat (volume: the retreat times the
/// column).
pub fn retreat_chance(retreat_km: f64, front_km: f64, sample_area_km2: f64) -> f64 {
    (retreat_km * front_km / sample_area_km2).clamp(0.0, 1.0)
}

/// What [`erode_forearcs`] did this step, km³ (Earth-normalized), km × Myr and km².
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Eroded {
    pub continental_km3: f64,
    pub other_km3: f64,
    pub short_km3: f64,
    pub front_continental_km_myr: f64,
    pub front_oceanic_km_myr: f64,
    /// With the trench's retreat on: the area consumed, continental crust and other crust, km².
    pub retreat_continental_km2: f64,
    pub retreat_other_km2: f64,
}

/// One subducting-side sample's effect on its contact and on the crust budget.
struct Effect {
    contact: usize,
    rate_mm_yr: f64,
    /// The sample is continental and rose fast enough for an Andean orogeny.
    andean: bool,
    /// Arc crust added at the continental arcs' law, m: to a continental sample, or with
    /// `island_arc_own_crust` to continental crust no intra-oceanic arc made (ledger: PLT-760).
    continental_m: f64,
    /// Arc crust added at the island arcs' law, m: to an oceanic sample, or with
    /// `island_arc_own_crust` to the crust an intra-oceanic arc made.
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
    /// A source's share of the strain: 1, or with `tectonics.thermal_backstop` its lithosphere's
    /// weakness (`thermal::backstop_weight`, ledger: PLT-470).
    weight: f64,
    /// The arc magma follows the island arcs' law (ledger: PLT-760, [`arc_is_island`]).
    island: bool,
}

/// Per contact: what its arcs and cordillera add up to this step.
#[derive(Clone, Copy, Default)]
struct Budget {
    volcanic_m: f64,
    plutonic_m: f64,
    cones_m: f64,
    /// The felsic cones it counts inside its rate, on the crust of the continental arcs' law and
    /// of the island arcs' (measurement: `plates/arc_cones_*_km3`).
    cones_by_law_m: [f64; 2],
    demand_m: f64,
    supply_m: f64,
    supply_pools: [f64; pool::COUNT],
}

/// Divergence flags, trenches, fronts, subduction uplift and arcs; each plate's oceanic front length.
///
/// `cones` (per sample, felsic and mafic volcanic rock, m) is the rock the volcano stage's cones
/// handed to the pools at this step's feedback: an arc counts its felsic cones inside its own
/// rate and clears them (`hotspots::step` counts what is left). `sea_m` is the sea the step stands
/// under (the uplift's ceiling and an island arc's emergence are taken from it).
pub fn fronts_and_uplift(
    state: &mut PlateState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    cones: &mut [[f64; 2]],
    sea_m: f64,
) -> Pull {
    let tree = Tree::over(pos, |_| true);
    let begun = fronts_begin(state, geo, pos, &tree, env);
    let reach = fronts_reach(geo, &begun, env);
    fronts_finish(state, geo, env, cones, begun, reach, sea_m)
}

/// Per sample: divergence, trench, and at most one front (the fastest-converging neighbour).
struct Boundary {
    divergent: bool,
    trench: bool,
    front: Option<(u32, usize, DVec3, DVec3)>,
}

/// Set to 1 to scale the subduction uplift by the convergence normal to the front (the boundary's
/// normal, [`boundary_normal`]) instead of the full relative speed, and to stamp that component
/// as the samples' convergence (the cordillera takes no convergence since PLT-320). Off by
/// default: an experiment of the land-shape task (2026-10-20), which measured it, on the arcs as
/// well then, to bring the arcs' yearly crust near Earth's and to take a fifth of the land away
/// (`docs/science/plates.md`, "Land shape and the high ground"); the crust-budget task measured it
/// again with the cordillera at Earth's typical arc and the collisions still losing four times
/// Earth's ("The crust budget against Stern & Scholl"). Since 2026-10-25 the arcs read the normal
/// component whatever it says (PLT-665). ledger: PLT-312
pub const NORMAL_CONVERGENCE_ENV: &str = "DEEPTIME_ARC_NORMAL";

fn normal_convergence() -> bool {
    static M: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *M.get_or_init(|| std::env::var(NORMAL_CONVERGENCE_ENV).is_ok_and(|v| v == "1"))
}

/// The convergence normal to the front at `s` from what its front stamped, mm/yr: PLT-312's
/// component of the relative motion along the boundary's normal, Clift & Vannucchi (2004)'s
/// orthogonal rate. With `DEEPTIME_ARC_NORMAL=1` the stamped convergence is that component;
/// otherwise it is the whole relative speed, and the obliquity is measured from the normal, so
/// the component is the speed times its cosine (0 where the plates part along the normal). No
/// field of its own: a plate sample's 4 B more would take full 16k's memory estimate past its
/// 5 GiB cap. ledger: PLT-660
pub(super) fn orthogonal_convergence_mm_yr(s: &super::state::Sample) -> f64 {
    let v = f64::from(s.convergence_mm_yr);
    if normal_convergence() {
        v
    } else {
        (v * f64::from(s.obliquity_deg).to_radians().cos()).max(0.0)
    }
}

/// The first part of [`fronts_and_uplift`]: what it reads of the plates, their groups, the
/// hidden flags, the positions, the motions and the contacts, none of which the belts' stacking
/// (`belts::stack`) changes, so the arcs' distance field ([`fronts_reach`]) can run beside it
/// (P1-19).
pub struct FrontsBegun {
    plate_of: Vec<u32>,
    boundary: Vec<Boundary>,
    /// Per front sample: the boundary's unit normal towards the front's other plate
    /// ([`boundary_normal`]); zero elsewhere.
    normals: Vec<DVec3>,
    /// Per front sample: its share of the straight front, radians ([`front_share_rad`],
    /// PLT-530); zero elsewhere.
    shares: Vec<f64>,
    /// The fronts as the distance field's sources (sample, 0, the front's number).
    sources: Vec<(u32, f64, u32)>,
    /// Per sample: its group, with [`HIDDEN_KEY`] added when it is hidden.
    key: Vec<u32>,
}

impl FrontsBegun {
    /// Measurement only (`front_trace.rs`): each sample's front (the other-plate neighbour, the
    /// contact, the relative velocity, the lattice edge's direction) and its boundary normal.
    pub fn traced_fronts(&self) -> Vec<super::front_trace::FbFront> {
        self.boundary
            .iter()
            .zip(&self.normals)
            .map(|(b, &n)| b.front.map(|(u, c, rel, out)| (u, c, rel, out, n)))
            .collect()
    }
}

#[cfg(test)]
impl FrontsBegun {
    /// The step's front samples.
    pub(super) fn front_count(&self) -> usize {
        self.sources.len()
    }
}

/// [`FrontsBegun::key`]'s mark of a hidden sample.
const HIDDEN_KEY: u32 = 1 << 31;

/// How near a front sample a visible sample of the subducting plate must stand now for the front
/// to stand on the current geometry (`tectonics.front_current_geometry`), radians, from the lattice
/// `spacing` the run has: one spacing for the step to the neighbour across the boundary, one for
/// the trench's single hidden row, the subducting plate's edge sample that has just gone under the
/// forearc (the overlaps hide a sample once it crosses the overriding outline, PLT-553, so the
/// next row in stays visible). Not a tuned length: on a new lattice, where every front's
/// neighbour is beside it, 97 % of the ocean-continent fronts meet it (the front trace,
/// plates.md, "What the trace found"). ledger: PLT-582
pub fn trench_row_rad(spacing: f64) -> f64 {
    2.0 * spacing
}

/// Whether a sample's front towards a plate stands on the current geometry (PLT-582): a visible
/// sample of that plate within [`trench_row_rad`] of it now, asked once per plate met and kept.
/// With no row (the rule off) every plate counts.
pub(crate) struct SlabBeside {
    row: Option<f64>,
    met: [(u32, bool); 4],
}

impl SlabBeside {
    pub(crate) fn new(row: Option<f64>) -> SlabBeside {
        SlabBeside {
            row,
            met: [(u32::MAX, false); 4],
        }
    }

    /// Whether a visible sample of `plate` stands within the row of `x` (`tree` holds every
    /// sample; `plate_of` each sample's plate).
    pub(crate) fn has(
        &mut self,
        samples: &[super::state::Sample],
        plate_of: &[u32],
        tree: &Tree,
        x: DVec3,
        plate: u32,
    ) -> bool {
        let Some(row) = self.row else {
            return true;
        };
        if let Some(&(_, m)) = self.met.iter().find(|&&(p, _)| p == plate) {
            return m;
        }
        let m = tree
            .within(x, row)
            .iter()
            .any(|&(j, _)| plate_of[j as usize] == plate && !samples[j as usize].is_hidden());
        if let Some(slot) = self.met.iter_mut().find(|(p, _)| *p == u32::MAX) {
            *slot = (plate, m);
        }
        m
    }
}

/// Each sample's boundary and the step's fronts (see [`FrontsBegun`]); `tree` holds every
/// sample at `pos` (the step's), for the front on the current geometry (PLT-582).
pub fn fronts_begin(
    state: &PlateState,
    geo: &Geo,
    pos: &[DVec3],
    tree: &Tree,
    env: &Env,
) -> FrontsBegun {
    let step = env.step;
    let norm = env.norm;
    let n = state.samples.len();
    let plate_of: Vec<u32> = (0..n).into_par_iter().map(|i| state.plate_of(i)).collect();
    // ledger: PLT-582 (the front on the current geometry)
    let row = env
        .k
        .front_current_geometry
        .then(|| trench_row_rad(env.norm.spacing_rad()));
    let omega: Vec<DVec3> = state
        .plates
        .iter()
        .map(|p| DVec3::from_array(p.omega))
        .collect();
    let rots: Vec<DQuat> = state.groups.iter().map(|g| quat(g.rot)).collect();
    let pts = geo.points();
    // The working subduction zones: overlap seen this step, or carried through a resampling
    // (ledger: PLT-550).
    let mut active: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for (i, c) in state.contacts.iter().enumerate() {
        if c.active(step) {
            active.insert((c.under, c.over), i);
        }
    }

    let samples = &state.samples;
    let mut boundary: Vec<Boundary> = (0..n)
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
            let mut beside = SlabBeside::new(row);
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
                    && beside.has(samples, &plate_of, tree, x, pu)
                {
                    best = -along;
                    b.front = Some((u, c, rel, out));
                }
            }
            b
        })
        .collect();
    // Each front's boundary normal (zero elsewhere). ledger: PLT-310
    let mut normals: Vec<DVec3> = (0..n)
        .into_par_iter()
        .map(|v| match boundary[v].front {
            Some((u, ..)) => {
                boundary_normal(geo, pos, &rots, samples, &plate_of, v, plate_of[u as usize])
            }
            None => DVec3::ZERO,
        })
        .collect();
    // A front is where the plates converge across their boundary, beyond Bird's 20 degrees of a
    // transform, as the boundary mix judges it (`dynamics::classify_boundary`): a contact belongs
    // to a pair of plates, not to a stretch of their boundary, and one lattice edge's direction
    // is no azimuth (plates sliding past each other converge along half the edges of a zigzag
    // boundary). Until 2026-10-20 every lattice edge converging on an active contact was a
    // front, and the contacts' strike-slip stretches built arcs: 142,000 km of island-arc front
    // a step against the mix's 82,000 km of ocean-ocean subduction (build log 2026-10-20, "The
    // arc's basement and the arcs' fronts"). ledger: PLT-330
    for (b, nrm) in boundary.iter_mut().zip(normals.iter_mut()) {
        if let Some((_, _, rel, _)) = b.front
            && (is_transform(rel, *nrm) || rel.dot(*nrm) >= 0.0)
        {
            b.front = None;
            *nrm = DVec3::ZERO;
        }
    }
    // Each front sample's share of the straight front (measured whether or not `front_share`
    // counts by it: the `plates/arc_front_straight_km_myr` global). ledger: PLT-530
    let shares: Vec<f64> = (0..n)
        .into_par_iter()
        .map(|v| match boundary[v].front {
            Some((u, ..)) => front_share_rad(
                geo,
                pos,
                &rots,
                samples,
                &plate_of,
                v,
                plate_of[u as usize],
                normals[v],
            ),
            None => 0.0,
        })
        .collect();
    let mut sources: Vec<(u32, f64, u32)> = Vec::new();
    for (v, b) in boundary.iter().enumerate() {
        if b.front.is_some() {
            sources.push((v as u32, 0.0, sources.len() as u32));
        }
    }
    debug_assert!(state.groups.len() < HIDDEN_KEY as usize);
    let key: Vec<u32> = samples
        .par_iter()
        .map(|s| s.group | if s.is_hidden() { HIDDEN_KEY } else { 0 })
        .collect();
    FrontsBegun {
        plate_of,
        boundary,
        normals,
        shares,
        sources,
        key,
    }
}

/// The arcs' distance field: from the fronts over their own groups' visible samples, to
/// `subduction_distance_km` ([`geometry::dijkstra`]'s distances, labels and the reached samples,
/// ascending), each group's run on its own thread (no edge it may cross joins two groups).
pub fn fronts_reach(geo: &Geo, begun: &FrontsBegun, env: &Env) -> (Vec<f64>, Vec<u32>, Vec<u32>) {
    let rs = env.norm.rad(env.k.subduction_distance_km);
    let key = &begun.key;
    // The edge a → b is allowed when b is visible and of a's group (a, reached, is visible).
    dijkstra_parts(
        geo,
        &begun.sources,
        |v| key[v as usize] & !HIDDEN_KEY,
        |a, b| key[b as usize] == key[a as usize] & !HIDDEN_KEY,
        rs,
    )
}

/// The rest of [`fronts_and_uplift`], after the belts' stacking, with the fronts and the
/// distance field found before it.
pub fn fronts_finish(
    state: &mut PlateState,
    geo: &Geo,
    env: &Env,
    cones: &mut [[f64; 2]],
    begun: FrontsBegun,
    reach: (Vec<f64>, Vec<u32>, Vec<u32>),
    sea_m: f64,
) -> Pull {
    let t_myr = env.t_myr;
    let k = env.k;
    let norm = env.norm;
    let FrontsBegun {
        plate_of,
        boundary,
        normals,
        shares,
        sources,
        key,
    } = begun;
    let (dist, label, reached) = reach;
    // A contact carried through a resampling whose boundary has no front this step (its plates no
    // longer converge across it) is no working subduction zone (ledger: PLT-550).
    if state.contacts.iter().any(|c| c.carried) {
        let mut fronted = vec![false; state.contacts.len()];
        for b in &boundary {
            if let Some((_, c, ..)) = b.front {
                fronted[c] = true;
            }
        }
        for (c, f) in state.contacts.iter_mut().zip(fronted) {
            c.carried &= f;
        }
    }
    let rots: Vec<DQuat> = state.groups.iter().map(|g| quat(g.rot)).collect();
    let pts = geo.points();
    // Which reached samples stand over their front's slab, with `arc_slab_ends` (the arcs,
    // PLT-561) or `margin_slab_ends` (the uplift, the folds, the cordillera and the forearc,
    // PLT-562).
    let over_slab = (k.arc_slab_ends || k.margin_slab_ends).then(|| {
        over_the_slab(
            geo, &rots, &normals, &shares, &sources, &key, &dist, &label, &reached, env,
        )
    });
    let over = over_slab.as_deref();
    let slab_arc = |i: usize| !k.arc_slab_ends || over.is_none_or(|o| o[i]);
    let slab_margin = |i: usize| !k.margin_slab_ends || over.is_none_or(|o| o[i]);
    // One-way subduction: the forearc and arc that the next step's hiding reads, within the arc's
    // distance of a front and one lattice spacing for where the front falls, as PLT-503's reach
    // (ledger: PLT-542, PLT-042), over the slab with `margin_slab_ends` (PLT-562).
    if let Some(ow) = state.one_way.as_mut() {
        let reach_km = k.arc_distance_km + norm.km(norm.spacing_rad());
        ow.forearc = dist
            .iter()
            .zip(&state.samples)
            .enumerate()
            .map(|(i, (&d, s))| {
                d.is_finite() && !s.is_hidden() && norm.km(d) <= reach_km && slab_margin(i)
            })
            .collect();
    }
    // Incipient contacts (PLT-610): no arc until the slab, before this step's growth as the
    // force balance reads it, reaches the critical convergence.
    let young: Vec<bool> = if k.incipient_subduction {
        let km: BTreeMap<(u32, u32), f64> = state
            .slabs
            .iter()
            .map(|s| ((s.under, s.over), s.km))
            .collect();
        state
            .contacts
            .iter()
            .map(|c| {
                super::forces::incipient(k, km.get(&(c.under, c.over)).copied().unwrap_or(0.0))
            })
            .collect()
    } else {
        vec![false; state.contacts.len()]
    };
    let mut info: Vec<Source> = Vec::new();
    let mut front_len = vec![0.0; state.plates.len()];
    let mut subducting_fronts: Vec<u32> = Vec::new();
    let mut subducting_detail: Vec<(u32, DVec3)> = Vec::new();
    let (mut front_continental, mut front_oceanic) = (0.0_f64, 0.0_f64);
    // The same two weighted by the convergence normal to the front (measurement: the mean v_n at
    // the arcs, PLT-665).
    let mut front_normal = [0.0_f64; 2];
    // The same two whose arc is younger than the early pulse (measurement; PLT-670).
    let mut front_young = [0.0_f64; 2];
    // The arcs begun this step, whether one of their fronts had continental crust within the
    // arc's distance, and the fronts of intra-oceanic arcs past incipient, km × Myr (ledger:
    // PLT-760).
    let mut begun_now = vec![false; state.contacts.len()];
    let mut continent_at_onset = vec![false; state.contacts.len()];
    let mut front_intra_oceanic = 0.0_f64;
    let front_km = norm.km(norm.spacing_rad());
    let arc_steps = (k.arc_distance_km / front_km).floor() as u32;
    // A front sample's length of front, km: its share of the straight front with `front_share`
    // on, one lattice spacing off (which counts a straight front about 1.04 times). ledger: PLT-530
    let length_km = |v: usize| {
        if k.front_share {
            norm.km(shares[v])
        } else {
            front_km
        }
    };
    let mut straight_km_myr = 0.0_f64;
    for (v, b) in boundary.iter().enumerate() {
        let s = &mut state.samples[v];
        if b.divergent {
            s.flags |= DIVERGENT;
        }
        if b.trench {
            s.flags |= TRENCH;
        }
    }
    let by_normal = normal_convergence();
    let v0 = k.convergence_ref_mm_yr; // ledger: PLT-037
    let (mut g_full_km_myr, mut g_normal_km_myr) = (0.0_f64, 0.0_f64);
    let mut g_continental_km_myr = [0.0_f64; 2];
    // Subduction erosion's fronts (PLT-441, PLT-442), in the order of the distance field's labels.
    let erosion_rate = k.subduction_erosion_km3_per_km_myr; // ledger: PLT-441
    let (mut loss_km3, mut loss_continental, mut loss_front) = (Vec::new(), Vec::new(), Vec::new());
    let mut loss_km = Vec::new();
    for (v, b) in boundary.iter().enumerate() {
        if let Some((u, c, rel, _)) = b.front {
            let su = state.samples[u as usize];
            let speed = rel.length();
            // The convergence normal to the boundary, mm/yr: the other plate's motion towards
            // the sample's along the boundary's normal. ledger: PLT-312
            let normal = normals[v];
            let across = (-rel.dot(normal)).max(0.0);
            // Incipient: its contact's slab short of the critical convergence (PLT-610), or an
            // oceanic slab coming in too slowly for a slab to form (PLT-681).
            let incipient_here =
                young[c] || (su.kind != CONTINENTAL && super::forces::below_floor(k, across));
            let len_km = length_km(v);
            if erosion_rate > 0.0 {
                // An oceanic slab coming in fast under little sediment: an erosive margin.
                // ledger: PLT-442
                let cover = f64::from(su.pools[pool::SEDIMENT] + su.pools[pool::SEDIMENTARY]);
                let erosive = su.kind != CONTINENTAL
                    && across >= k.subduction_erosion_min_mm_yr
                    && cover < k.subduction_erosion_max_sediment_m;
                // At least a sliver of front where the share is 0, so the front is still
                // marked erosive (its retreat's chance and its volume are then 0).
                loss_km3.push(if erosive {
                    (erosion_rate * len_km * env.dt_myr).max(f64::MIN_POSITIVE)
                } else {
                    0.0
                });
                loss_continental.push(state.samples[v].kind == CONTINENTAL);
                loss_front.push([v as u32, u]);
                loss_km.push(len_km);
            }
            if su.kind != CONTINENTAL {
                straight_km_myr += norm.km(shares[v]) * env.dt_myr;
                g_full_km_myr += len_km * env.dt_myr * (speed / v0).min(1.0);
                g_normal_km_myr += len_km * env.dt_myr * (across / v0).min(1.0);
                if state.samples[v].kind == CONTINENTAL {
                    g_continental_km_myr[0] += len_km * env.dt_myr * (speed / v0).min(1.0);
                    g_continental_km_myr[1] += len_km * env.dt_myr * (across / v0).min(1.0);
                }
            }
            let conv = if by_normal { across } else { speed };
            // The obliquity is the angle between the convergence and the boundary's normal
            // (PLT-312), whichever rate the uplift reads, so the orthogonal rate is the stamped speed
            // times its cosine ([`orthogonal_convergence_mm_yr`], PLT-660). Until 2026-10-25 it
            // was measured from the lattice edge to the converging neighbour unless
            // `DEEPTIME_ARC_NORMAL=1`, an edge that zigzags about the normal.
            let obliquity = if speed > 0.0 {
                (rel.dot(-normal) / speed)
                    .clamp(-1.0, 1.0)
                    .acos()
                    .to_degrees()
            } else {
                0.0
            };
            info.push(Source {
                contact: c,
                rel,
                conv,
                normal: across,
                obliquity_deg: obliquity,
                z_sub: f64::from(su.elevation_m),
                slab_age: su.age_myr(t_myr),
                sediment: f64::from(su.pools[pool::SEDIMENT]),
                oceanic_slab: su.kind != CONTINENTAL,
                incipient: incipient_here,
            });
            // Only an oceanic slab counts as a front that pulls: continental crust at the front
            // is buoyant, and where it comes in the collision resists instead (`forces.rs`).
            // ledger: PLT-251
            if su.kind != CONTINENTAL {
                let under = state.contacts[c].under as usize;
                front_len[under] += front_km;
                if !incipient_here {
                    subducting_fronts.push(v as u32);
                    subducting_detail.push((u, normal));
                }
                // By the crust the arc stands on, as the boundary mix counts it (ledger: PLT-370).
                let continent_near = continent_within(state, geo, v as u32, arc_steps, |u| {
                    state.samples[u].is_hidden()
                });
                // The arc begins at its contact's first oceanic front past incipient (a sample
                // too slow for a slab is not, PLT-681), and is young until the early pulse ends
                // (ledger: PLT-669, PLT-670). Its setting is the crust at its fronts that step:
                // intra-oceanic where none of them has continental crust within the arc's
                // distance (ledger: PLT-760).
                let arc_young = !incipient_here && {
                    let con = &mut state.contacts[c];
                    if con.arc_onset_myr.is_none() {
                        con.arc_onset_myr = Some(t_myr);
                        begun_now[c] = true;
                    }
                    if begun_now[c] {
                        continent_at_onset[c] |= continent_near;
                    }
                    t_myr - con.arc_onset_myr.unwrap_or(t_myr) < ISLAND_ARC_BURST_MYR
                };
                let young_km_myr = if arc_young { len_km * env.dt_myr } else { 0.0 };
                // Measurement: the front of an intra-oceanic arc, by its setting at its onset
                // (ledger: PLT-760).
                if !incipient_here && state.contacts[c].arc_intra_oceanic {
                    front_intra_oceanic += len_km * env.dt_myr;
                }
                // Measurement only (ledger: PLT-370).
                if continent_near {
                    front_continental += len_km * env.dt_myr;
                    front_normal[0] += len_km * env.dt_myr * across;
                    front_young[0] += young_km_myr;
                } else {
                    front_oceanic += len_km * env.dt_myr;
                    front_normal[1] += len_km * env.dt_myr * across;
                    front_young[1] += young_km_myr;
                }
            }
        }
    }
    // The arcs begun this step take their setting (ledger: PLT-760).
    for (c, con) in state.contacts.iter_mut().enumerate() {
        if begun_now[c] {
            con.arc_intra_oceanic = !continent_at_onset[c];
        }
    }

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
    // √π · `arc_width_km`): crust per step at the axis at the type's reference convergence
    // (PLT-666), m.
    let band_km = std::f64::consts::PI.sqrt() * k.arc_width_km;
    let island_axis_m = k.island_arc_km3_per_km_myr / band_km * 1000.0 * env.dt_myr; // ledger: PLT-045
    let continental_axis_m = k.continental_arc_km3_per_km_myr / band_km * 1000.0 * env.dt_myr; // ledger: PLT-285
    // The cordillera: pure shear of the range at `cordillera_shortening_mm_yr` (Earth's typical
    // continental arc, at any convergence), the strain spread over its Gaussian band (integral
    // √π · `cordillera_width_km`), fed by the crust behind the range (from twice the band's
    // half-width beyond the arc to r_s), which shortens at the same rate and gives the volume.
    // ledger: PLT-048, PLT-320
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
            let continental = s.kind == CONTINENTAL;
            // The arc's type: the column's label, or with `island_arc_own_crust` the island law
            // on the crust an intra-oceanic arc made, whatever its label (ledger: PLT-760).
            let island = arc_is_island(k, contacts[src.contact].arc_intra_oceanic, s);
            // The arc's rate per km is its type's rate at Earth's mean orthogonal convergence of
            // that type's arcs, times v_n over that mean, with no cap: Stern & Scholl's rates are
            // averages over Earth's arcs at Earth's convergences, so scaling them by
            // min(v / 100 mm/yr, 1) counted the convergence twice. ledger: PLT-665, PLT-666
            let (axis, g) = if !island {
                (
                    continental_axis_m,
                    src.normal / k.continental_arc_convergence_ref_mm_yr,
                )
            } else if k.arc_front_loaded {
                // The island arc's early pulse, then its mature rate, by the arc's age (PLT-668).
                let age = contacts[src.contact].arc_onset_myr.map(|t0| t_myr - t0);
                let (rate, g) = island_arc_law(k, age, src.normal);
                (rate / band_km * 1000.0 * env.dt_myr, g)
            } else {
                (
                    island_axis_m,
                    src.normal / k.island_arc_convergence_ref_mm_yr,
                )
            };
            // Past the slab's ends no arc (`arc_slab_ends`, PLT-561), and none over an incipient
            // slab (PLT-610).
            let magma_m = if slab_arc(i) && !src.incipient {
                axis * w_arc_of(d_km, k) * g
            } else {
                0.0
            };
            let mut plan = Plan {
                magma_m,
                island,
                ..Plan::default()
            };
            // Past the slab's ends no range and no crust shortened into it (`margin_slab_ends`,
            // PLT-562).
            if shorten_km > 0.0
                && continental
                && plate_of[i] == contacts[src.contact].over
                && slab_margin(i)
            {
                if d_km < range_edge_km {
                    let w = (-((d_km - k.arc_distance_km) / k.cordillera_width_km).powi(2)).exp();
                    // Thickened arc crust spreads under its own weight: the range thickens until
                    // its column's Airy height reaches `cordillera_height_m` and no further (the
                    // Altiplano and the Puna stop near 4 km).
                    let iso = iso_height(&s.pools, z_0, gravity);
                    let room = (1.0 - iso.max(0.0) / k.cordillera_height_m).max(0.0);
                    // Not scaled by the convergence (g): back-arc deformation follows the upper
                    // plate's own motion, not the convergence (Heuret & Lallemand 2005; Jarrard
                    // 1986), and the rate is the length-weighted mean over Earth's continental
                    // arcs, whatever their convergence. ledger: PLT-320
                    plan.demand_m = f64::from(s.thickness_m) * strain_axis * w * room;
                } else {
                    plan.source = true;
                    plan.weight = super::thermal::backstop_weight(k, s.lithosphere_km);
                }
            }
            plan
        })
        .collect();
    // Per front sample (the distance field's label): how near its strip's closest sample with arc
    // magma lies to the arc's axis, km. Where the lattice is coarser than the arc band (5,000
    // samples are about 320 km apart, the band where its weight passes 0.3 is 132 km wide), no
    // sample may fall inside the band, and without this the planet has no arc at all; so each
    // front's strip flags its sample nearest the axis as well, within one spacing. On a lattice
    // finer than the band that sample lies inside the band already. ledger: PLT-043
    let mut axis_gap_km = vec![f64::INFINITY; info.len()];
    for &i in &reached {
        let i = i as usize;
        let d = dist[i];
        if !d.is_finite() || state.samples[i].is_hidden() || plans[i].magma_m <= 0.0 {
            continue;
        }
        let gap = &mut axis_gap_km[label[i] as usize];
        *gap = gap.min((norm.km(d) - k.arc_distance_km).abs());
    }
    let axis_gap_km = &axis_gap_km;
    let mut budget = vec![Budget::default(); contacts.len()];
    // Measurement only: the arc bands' weight over the samples they reach (ledger: PLT-554).
    let mut band_weight = 0.0_f64;
    // Over the reached samples, ascending (P1-19: over every sample before; the others add
    // nothing).
    for &i in &reached {
        let i = i as usize;
        let plan = &plans[i];
        let s = &state.samples[i];
        if !dist[i].is_finite() || s.is_hidden() {
            continue;
        }
        let src = &info[label[i] as usize];
        if !src.oceanic_slab {
            continue;
        }
        if slab_arc(i) && !src.incipient {
            band_weight += w_arc_of(norm.km(dist[i]), k);
        }
        let b = &mut budget[src.contact];
        // By the arc's law (the column's label unless `island_arc_own_crust`; PLT-760).
        let share = if plan.island {
            ISLAND_VOLCANIC_SHARE
        } else {
            CONTINENTAL_VOLCANIC_SHARE
        };
        b.volcanic_m += share * plan.magma_m;
        b.plutonic_m += (1.0 - share) * plan.magma_m;
        if let Some(c) = cones.get_mut(i) {
            b.cones_m += c[0];
            b.cones_by_law_m[usize::from(plan.island)] += c[0];
            c[0] = 0.0;
        }
        b.demand_m += plan.demand_m;
        if plan.source {
            // Weighted by the source's weakness (1 unless `thermal_backstop`; ledger: PLT-470).
            let w = plan.weight;
            b.supply_m += w * s.pools.iter().map(|&x| f64::from(x.max(0.0))).sum::<f64>();
            for (a, &x) in b.supply_pools.iter_mut().zip(&s.pools) {
                *a += w * f64::from(x.max(0.0));
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

    let effects: Vec<Effect> = state
        .samples
        .par_iter_mut()
        .enumerate()
        .filter_map(|(i, s)| {
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
            // Cortial's speed transfer, the uplift's alone since the arcs read PLT-665's.
            // ledger: PLT-037
            let g = (src.conv / v0).min(1.0);
            let zn = ((src.z_sub - zt) / (zc - zt)).clamp(0.0, 1.0);
            let h = zn * zn;
            let z = f64::from(s.elevation_m);
            let continental = s.kind == CONTINENTAL;
            let (mut added_continental, mut added_oceanic, mut cordillera) =
                (0.0_f64, 0.0_f64, 0.0_f64);
            // Cortial's h(z̃) uplift is the oceanic slab's: where continental crust comes in, the
            // collision belt (`belts.rs`) builds the relief from the crust shortened. Past the
            // slab's ends none, and no folds (`margin_slab_ends`, PLT-562).
            let margin = slab_margin(i);
            let lifted = if src.oceanic_slab && margin {
                z + u0_m * f * g * h * ceiling(z - sea_m, z_ceiling)
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
                // A cold, stiff source gives its weakness's share (ledger: PLT-470).
                let phi = phi * plan.weight;
                for p in s.pools.iter_mut() {
                    *p -= (f64::from(p.max(0.0)) * phi) as f32;
                }
                s.basement_m *= (1.0 - phi) as f32;
            }
            // Arc magma and the deposit accumulators.
            let m = plan.magma_m;
            if m > 0.0 {
                let before_m = s.arc_continental_m + s.arc_oceanic_m;
                // By the arc's law: the column's label, or with `island_arc_own_crust` the island
                // law on the crust an intra-oceanic arc made (ledger: PLT-760).
                if !plan.island {
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
                // The band where its weight passes 0.3, or the strip's sample nearest the axis
                // (above). ledger: PLT-043
                let gap = (d_km - k.arc_distance_km).abs();
                if w_arc_of(d_km, k) > 0.3
                    || (gap <= axis_gap_km[label[i] as usize] && gap < front_km)
                {
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
            if margin {
                let rot = rots[s.group as usize];
                let p = rot * pts[i];
                let fold =
                    to_world(rot, s.fold) + tangent(p, src.rel).normalize_or_zero() * (f * g);
                s.fold = to_group(rot, tangent(p, fold));
            }
            // An island arc's uplift marks an Andean orogeny where it stands above the sea.
            // ledger: PLT-060
            let andean = dz >= orogeny_m && (continental || f64::from(s.elevation_m) > sea_m);
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
    for e in effects {
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
        subducting_fronts,
        subducting_detail,
        arc_continental_km3: arc_continental_m * area / 1000.0,
        arc_oceanic_km3: arc_oceanic_m * area / 1000.0,
        front_continental_km_myr: front_continental,
        front_oceanic_km_myr: front_oceanic,
        cordillera_km3: cordillera_m * area / 1000.0,
        front_g_full_km_myr: g_full_km_myr,
        front_g_normal_km_myr: g_normal_km_myr,
        front_g_continental_km_myr: g_continental_km_myr,
        front_straight_km_myr: straight_km_myr,
        front_normal_km_myr: front_normal,
        front_young_km_myr: front_young,
        arc_band_km_myr: band_weight * area / band_km * env.dt_myr,
        arc_cones_km3: budget
            .iter()
            .fold([0.0; 2], |a, b| {
                [a[0] + b.cones_by_law_m[0], a[1] + b.cones_by_law_m[1]]
            })
            .map(|m| m * area / 1000.0),
        front_intra_oceanic_km_myr: front_intra_oceanic,
        forearcs: (erosion_rate > 0.0).then_some(Forearcs {
            loss_km3,
            continental: loss_continental,
            front: loss_front,
            front_km: loss_km,
            dist,
            label,
            reached,
            over: over_slab.filter(|_| k.margin_slab_ends),
        }),
    }
}

/// Subduction erosion (see the module docs; PLT-441, PLT-442): each erosive front takes its
/// step's crust from its forearc, the visible samples it reaches nearer than `arc_distance_km`,
/// the nearest first, each down to an ocean's column (`transition::oceanic_m`, 7 km). A column
/// loses its pools and its basement in proportion and sinks by the change, towards and never past
/// the height the column holds: continental crust by its Airy change, an island arc's forearc on
/// the ocean's height line (as the arcs raise them), never below z_t.
pub fn erode_forearcs(state: &mut PlateState, env: &Env, forearcs: &Forearcs) -> Eroded {
    let k = env.k;
    let norm = env.norm;
    let area = norm.sample_area_km2();
    let mut out = Eroded::default();
    for ((loss, &continental), km) in forearcs
        .loss_km3
        .iter()
        .zip(&forearcs.continental)
        .zip(&forearcs.front_km)
    {
        if *loss > 0.0 {
            if continental {
                out.front_continental_km_myr += km * env.dt_myr;
            } else {
                out.front_oceanic_km_myr += km * env.dt_myr;
            }
        }
    }
    if k.erosion_retreat {
        return retreat_forearcs(state, env, forearcs, out);
    }
    // The forearcs' samples, by front, the nearest first (ties by sample); over the slab with
    // `margin_slab_ends` (ledger: PLT-562).
    let width_rad = norm.rad(k.arc_distance_km); // ledger: PLT-441 (the forearc: to the arc's axis)
    let over = forearcs.over.as_deref();
    let mut band: Vec<(u32, f64, u32)> = forearcs
        .reached
        .iter()
        .filter_map(|&i| {
            let d = forearcs.dist[i as usize];
            let f = forearcs.label[i as usize];
            (d.is_finite()
                && d < width_rad
                && forearcs.loss_km3[f as usize] > 0.0
                && !state.samples[i as usize].is_hidden()
                && over.is_none_or(|o| o[i as usize]))
            .then_some((f, d, i))
        })
        .collect();
    band.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)).then(a.2.cmp(&b.2)));
    let floor_m = oceanic_m(); // ledger: PLT-441
    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
    let (z_f, per_m) = (k.oceanic_floor_elevation_m, crust_per_metre(env));
    let z_t = k.trench_elevation_m;
    let mut left_m = vec![0.0_f64; forearcs.loss_km3.len()];
    for (f, x) in forearcs.loss_km3.iter().enumerate() {
        left_m[f] = x / area * 1000.0;
    }
    for &(f, _, i) in &band {
        let left = &mut left_m[f as usize];
        if *left <= 0.0 {
            continue;
        }
        let s = &mut state.samples[i as usize];
        let t = f64::from(s.thickness_m);
        let take = (t - floor_m).max(0.0).min(*left);
        if take <= 0.0 {
            continue;
        }
        *left -= take;
        let q = take / t;
        let before = s.pools;
        let basement = f64::from(s.basement_m);
        for p in s.pools.iter_mut() {
            *p = (f64::from(*p) * (1.0 - q)) as f32;
        }
        s.basement_m = (basement * (1.0 - q)) as f32;
        s.thickness_m = s.pools.iter().sum();
        let gone = t - f64::from(s.thickness_m);
        let z = f64::from(s.elevation_m);
        let lowered = if s.kind == CONTINENTAL {
            let base_gone = basement * q;
            out.continental_km3 += (gone - base_gone) * area / 1000.0;
            out.other_km3 += base_gone * area / 1000.0;
            let change = isostatic_change_m(&before, &s.pools, env.gravity);
            toward(z, change, iso_height(&s.pools, z_0, env.gravity))
        } else {
            out.other_km3 += gone * area / 1000.0;
            let after = crystalline_m(&s.pools);
            let change = (after - crystalline_m(&before)) / per_m;
            toward(z, change, held_floor_m(after, f64::INFINITY, z_f, per_m))
        };
        s.elevation_m = lowered.max(z_t.min(z)) as f32;
    }
    out.short_km3 = left_m.iter().map(|m| m.max(0.0) * area / 1000.0).sum();
    out
}

/// The trench's retreat (PLT-522, PLT-523; the module docs): each erosive front sample goes whole
/// at [`retreat_chance`], its column booked to the mantle (continental crust, its basement as
/// other crust, as [`erode_forearcs`] books), and leaves its plate for the slab's: a twin of its
/// group in the subducting plate (the same rotation, so it stays where it is), holding the slab's
/// sample across the front with no sediment (that sample's own sediment is not counted twice).
/// The draws are in the fronts' order, from the plate state's generator.
fn retreat_forearcs(
    state: &mut PlateState,
    env: &Env,
    forearcs: &Forearcs,
    mut out: Eroded,
) -> Eroded {
    let norm = env.norm;
    let area = norm.sample_area_km2();
    let retreat_km = RETREAT_KM_PER_MYR * env.dt_myr; // ledger: PLT-522
    let mut twins: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for (f, &[v, u]) in forearcs.front.iter().enumerate() {
        if forearcs.loss_km3[f] <= 0.0 {
            continue;
        }
        // The sample's length of front: a spacing, or its straight share (PLT-530).
        let chance = retreat_chance(retreat_km, forearcs.front_km[f], area);
        let (vi, ui) = (v as usize, u as usize);
        let s = state.samples[vi];
        let under = state.plate_of(ui);
        if s.is_hidden() || state.plate_of(vi) == under || state.rng.next_f64() >= chance {
            continue;
        }
        let t = f64::from(s.thickness_m);
        if s.kind == CONTINENTAL {
            let basement = f64::from(s.basement_m);
            out.continental_km3 += (t - basement) * area / 1000.0;
            out.other_km3 += basement * area / 1000.0;
            out.retreat_continental_km2 += area;
        } else {
            out.other_km3 += t * area / 1000.0;
            out.retreat_other_km2 += area;
        }
        let twin = *twins.entry((s.group, under)).or_insert_with(|| {
            let rot = state.groups[s.group as usize].rot;
            state.groups.push(super::state::Group { plate: under, rot });
            (state.groups.len() - 1) as u32
        });
        let mut slab = state.samples[ui];
        slab.group = twin;
        slab.pools[pool::SEDIMENT] = 0.0;
        slab.pools[pool::SEDIMENTARY] = 0.0;
        slab.thickness_m = slab.pools.iter().sum();
        state.samples[vi] = slab;
    }
    out
}

/// Per sample, whether a sample the fronts' distance field reaches stands over its front's slab
/// (`tectonics.arc_slab_ends`, the front lane's step 10, plates.md "Step 10", ledger: PLT-561;
/// `tectonics.margin_slab_ends`, step 11, "Step 11", PLT-562): between the ends of the front its
/// foot stands on, along that front. Its foot is the front sample nearest it on the sphere, found
/// from its label (its nearest front sample by the lattice's distance) front sample by front
/// sample along the front: the lattice's distance runs up to 2/√3 longer than the straight line,
/// by the edges' direction, so the label can lie along the front by a good part of the distance
/// (some hundreds of km at the uplift's 1,800), and a front's line read there, on a curved front,
/// is not the sample's. The front is the foot and the front samples joined to it, one by one,
/// within two lattice edges in the same group (a zigzag boundary's second row, a corner), out to
/// a reach; its line runs through the foot square to the front's normals (PLT-310), each weighted
/// by its share of the straight front (PLT-530), and each front sample covers its share along the
/// line, centred on it. A sample whose distance along that line falls outside the front's extent
/// lies past the slab's end. The reach, the front the line is read along and that must run on
/// past the foot before its ends are read, is two lattice spacings more than the sample's tier:
/// the arc band's (`arc_distance_km` and two `arc_width_km`, past which its weight is under e^-4)
/// for a sample within that distance of the front (the arcs, the forearc, most of the range),
/// and the margin's (`subduction_distance_km`) for one further out. The lattice gives a front's
/// direction only to about its zigzag over the stretch it is read along (a few degrees over the
/// band's reach at a front's end, where the normals turn), which moves the end's meridian by the
/// sample's distance times that angle: read over the band's reach, the uplift and the range ran
/// on up to 2.1 spacings past a straight front's end, where the front itself reaches 1.25 (the
/// front lane's step 11). A curved front's far field reads the chord of its last
/// `subduction_distance_km`. The other samples are false.
#[allow(clippy::too_many_arguments)]
fn over_the_slab(
    geo: &Geo,
    rots: &[DQuat],
    normals: &[DVec3],
    shares: &[f64],
    sources: &[(u32, f64, u32)],
    key: &[u32],
    dist: &[f64],
    label: &[u32],
    reached: &[u32],
    env: &Env,
) -> Vec<bool> {
    let n = key.len();
    let pts = geo.points();
    let at = |i: usize| rots[(key[i] & !HIDDEN_KEY) as usize] * pts[i];
    let mut source_of = vec![u32::MAX; n];
    for (l, &(v, ..)) in sources.iter().enumerate() {
        source_of[v as usize] = l as u32;
    }
    // Each front sample's fronts within two lattice edges, in its own group.
    let near: Vec<Vec<u32>> = sources
        .par_iter()
        .map(|&(v, ..)| {
            let mut out: Vec<u32> = Vec::new();
            for (u, _) in geo.edges(v) {
                for w in std::iter::once(u).chain(geo.edges(u).map(|(w, _)| w)) {
                    let l = source_of[w as usize];
                    if w != v
                        && l != u32::MAX
                        && key[w as usize] == key[v as usize]
                        && !out.contains(&l)
                    {
                        out.push(l);
                    }
                }
            }
            out
        })
        .collect();
    let norm = env.norm;
    // The tiers: the arc band's distance and the margin's, rad.
    let band = norm.rad(env.k.arc_distance_km + 2.0 * env.k.arc_width_km);
    let margin = norm.rad(env.k.subduction_distance_km).max(band);
    // Each front sample's line (its position and the front's direction there) and the front's
    // extent along it, radians: [lo, hi], with the front read out to `reach`.
    let lines = |reach: f64| -> Vec<(DVec3, DVec3, f64, f64)> {
        sources
            .par_iter()
            .enumerate()
            .map(|(l, &(v, ..))| {
                let p = at(v as usize);
                // The front within the reach, front sample by front sample.
                let mut seen = std::collections::HashSet::from([l as u32]);
                let mut queue = vec![l as u32];
                let mut next = 0;
                while let Some(&a) = queue.get(next) {
                    next += 1;
                    for &b in &near[a as usize] {
                        let w = sources[b as usize].0 as usize;
                        if (at(w) - p).length() <= reach && seen.insert(b) {
                            queue.push(b);
                        }
                    }
                }
                // Its direction: square to its normals, each by its share of the front (one front
                // sample's own normal turns at a lattice corner, and most at a front's end).
                let mean: DVec3 = queue
                    .iter()
                    .map(|&b| {
                        let w = sources[b as usize].0 as usize;
                        normals[w] * shares[w]
                    })
                    .sum();
                let mut t = p.cross(mean).normalize_or_zero();
                if t == DVec3::ZERO {
                    t = p.cross(normals[v as usize]).normalize_or_zero();
                }
                let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
                for &b in &queue {
                    let w = sources[b as usize].0 as usize;
                    let tau = (at(w) - p).dot(t);
                    lo = lo.min(tau - 0.5 * shares[w]);
                    hi = hi.max(tau + 0.5 * shares[w]);
                }
                (p, t, lo, hi)
            })
            .collect()
    };
    let spacing = norm.spacing_rad();
    let near_lines = lines(band + 2.0 * spacing);
    let far = |i: usize| dist[i] > band;
    let far_lines = if margin > band && reached.iter().any(|&i| far(i as usize)) {
        lines(margin + 2.0 * spacing)
    } else {
        Vec::new()
    };
    // Each reached sample's foot on its front: from its label, front sample by front sample along
    // the front (within two lattice edges), to the nearest on the sphere.
    let foot = |i: usize| {
        let x = at(i);
        let mut a = label[i] as usize;
        let mut best = (at(sources[a].0 as usize) - x).length_squared();
        loop {
            let mut step = None;
            for &b in &near[a] {
                let e = (at(sources[b as usize].0 as usize) - x).length_squared();
                if e < best {
                    best = e;
                    step = Some(b as usize);
                }
            }
            match step {
                Some(b) => a = b,
                None => return a,
            }
        }
    };
    let found: Vec<bool> = reached
        .par_iter()
        .map(|&i| {
            let i = i as usize;
            let (p, t, lo, hi) = if far(i) && !far_lines.is_empty() {
                far_lines[foot(i)]
            } else {
                near_lines[foot(i)]
            };
            let tau = (at(i) - p).dot(t);
            t == DVec3::ZERO || (lo..=hi).contains(&tau)
        })
        .collect();
    let mut over = vec![false; n];
    for (&i, f) in reached.iter().zip(found) {
        over[i as usize] = f;
    }
    over
}

/// The arc band's weight at `d_km` from the front (Gaussian, centred `arc_distance_km` from it).
pub(super) fn w_arc_of(d_km: f64, k: &crate::params::TectonicParams) -> f64 {
    (-((d_km - k.arc_distance_km) / k.arc_width_km).powi(2)).exp()
}

/// At a resampling, with `on` (`tectonics.contact_carry`; ledger PLT-550): every contact active at
/// `step` is carried through it ([`super::state::Contact::carried`]), the subduction zone going
/// on while the new lattice's overlap forms again. Off, none is.
pub fn carry(state: &mut PlateState, step: u64, on: bool) {
    for c in &mut state.contacts {
        c.carried = on && c.active(step);
    }
}

/// Closes contacts unseen for longer than `contact_timeout_myr` (or all of them when `all`), writing an
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
    // ledger: PLT-059 (a time: the steps it spans follow the step's length)
    let timeout = (env.k.contact_timeout_myr / env.dt_myr).floor() as u64;
    let mut closed = Vec::new();
    let mut keep = Vec::with_capacity(state.contacts.len());
    let alive: Vec<bool> = state.plates.iter().map(|p| p.alive).collect();
    for c in std::mem::take(&mut state.contacts) {
        let gone = !alive[c.under as usize] || !alive[c.over as usize];
        // A collision keeps its contact while its front lives (`shortening::GRACE_MYR`): two
        // plates that have stalled against each other overlap no more, and must still weld.
        let colliding = c.collision.as_ref().is_some_and(|col| {
            step.saturating_sub(col.last_step) <= super::shortening::grace_steps(env.dt_myr)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sphere::{SplitMix64, fibonacci_sphere};

    /// The trench's retreat against its sources (PLT-522, PLT-523): Scholl & von Huene 2010's
    /// ~2.5 km/Myr, inside Clift & Vannucchi 2004's Table 2 range (0.9 to 4.7); on the normal
    /// continental column it takes about Clift & Vannucchi's global 90 km³/km/Myr (PLT-441; their
    /// margins 20.0 to 139.4); the chance is the swept area over the sample's, at most 1.
    #[test]
    fn the_retreat_is_scholl_and_von_huenes_and_takes_clift_and_vannucchis_volume() {
        assert_eq!(RETREAT_KM_PER_MYR, 2.5);
        assert!((0.9..=4.7).contains(&RETREAT_KM_PER_MYR));
        let per_km = RETREAT_KM_PER_MYR * continental_m() / 1000.0;
        assert!((per_km - 90.0).abs() < 0.1 * 90.0, "{per_km} km³/km/Myr");
        assert!((20.0..=139.4).contains(&per_km));
        let (front_km, area_km2) = (71.4, 5100.0);
        let p = retreat_chance(5.0, front_km, area_km2);
        assert!((p - 5.0 * front_km / area_km2).abs() < 1e-12);
        assert_eq!(retreat_chance(5_000.0, front_km, area_km2), 1.0);
        assert_eq!(retreat_chance(0.0, front_km, area_km2), 0.0);
    }

    /// Wherever the cubes say a sample's nearest are all its plate's, brute force agrees, on a
    /// lattice and on random samples, with plates cut along wavy bands and a gap left open.
    #[test]
    fn voxels_skip_only_samples_whose_nearest_share_their_plate() {
        let mut rng = SplitMix64::new(17);
        let random: Vec<DVec3> = (0..30_000)
            .map(|_| rng.unit_vector())
            .filter(|p| p.x.abs() > 0.05)
            .collect();
        for pts in [fibonacci_sphere(40_000), random] {
            let plate: Vec<u32> = pts
                .iter()
                .map(|p| ((p.z + 0.2 * (5.0 * p.x).sin() + 1.0) * 3.0) as u32)
                .collect();
            let vox = Voxels::new(&pts, &plate);
            let mut skipped = 0;
            for (i, &q) in pts.iter().enumerate() {
                if !vox.only(q, plate[i]) {
                    continue;
                }
                skipped += 1;
                let mut d: Vec<(f64, usize)> = pts
                    .iter()
                    .enumerate()
                    .map(|(j, p)| (p.distance_squared(q), j))
                    .collect();
                d.select_nth_unstable_by(K_NEAR - 1, |a, b| a.0.total_cmp(&b.0));
                let cut = d[K_NEAR - 1].0;
                // Every sample as near as the K_NEAR-th (ties included) is of the same plate.
                assert!(
                    d.iter()
                        .filter(|x| x.0 <= cut)
                        .all(|x| plate[x.1] == plate[i]),
                    "sample {i}"
                );
                if skipped >= 600 {
                    break;
                }
            }
            assert!(skipped >= 600, "{skipped}");
        }
    }

    /// Stern 2004: "polarity reversal ... is triggered by buoyant crust entering the trench", and
    /// against a continent "It takes less work to deform large parts of continental Asia than it
    /// does to start a new subduction zone in the Indian Plate". The reach first: on a cap of
    /// continent in old floor, a sample finds continental crust within `steps` lattice steps
    /// through its plate exactly when its lattice distance to the cap is at most `steps`, and
    /// hidden crust is no arc's footing. Then the judgement (ledger: PLT-056, PLT-371): off, an
    /// oceanic fringe in the overlap in front of a continent reverses; judged by the arc's
    /// crust it goes on as a collision; an intra-oceanic arc reverses either way, an overriding
    /// continent in the overlap never, and oceanic crust arriving never.
    #[test]
    fn reversal_judges_the_overriding_side_by_the_crust_its_arc_stands_on() {
        use crate::plates::rifting::tests::{N, continental, old_floor, one_plate};
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let cap = |p: DVec3| p.z > 0.5;
        let mut state = one_plate(
            &geo,
            3,
            |p| if cap(p) { continental() } else { old_floor() },
        );
        let hid = |st: &PlateState| {
            let h: Vec<bool> = st.samples.iter().map(|s| s.is_hidden()).collect();
            move |v: usize| h[v]
        };
        // Lattice distance to the cap by breadth first search (the edges are symmetric).
        let mut dist = vec![u32::MAX; N];
        let mut frontier: Vec<u32> = (0..N as u32)
            .filter(|&i| cap(geo.points()[i as usize]))
            .collect();
        for &i in &frontier {
            dist[i as usize] = 0;
        }
        let mut d = 0;
        while !frontier.is_empty() {
            d += 1;
            let mut next = Vec::new();
            for &v in &frontier {
                for (u, _) in geo.edges(v) {
                    if dist[u as usize] == u32::MAX {
                        dist[u as usize] = d;
                        next.push(u);
                    }
                }
            }
            frontier = next;
        }
        let vis = hid(&state);
        for steps in [0, 1, 3] {
            let mut near = 0;
            for i in (0..N as u32).step_by(7) {
                let want = dist[i as usize] <= steps;
                near += usize::from(want && dist[i as usize] > 0);
                assert_eq!(
                    continent_within(&state, &geo, i, steps, &vis),
                    want,
                    "sample {i}, {steps} steps, distance {}",
                    dist[i as usize]
                );
            }
            assert!(
                steps == 0 || near > 0,
                "{steps} steps: no ocean sample in reach"
            );
        }
        for s in state.samples.iter_mut().filter(|s| s.kind == CONTINENTAL) {
            s.flags |= crate::plates::state::HIDDEN;
        }
        let vis = hid(&state);
        assert!((0..N as u32).all(|i| !continent_within(&state, &geo, i, 3, &vis)));

        // A continent arriving under an oceanic fringe whose plate's continent is within the arc's
        // reach: the overlap's crust reverses it, the arc's does not.
        assert!(reverses([1.0, 0.0], 1.0, [0.0, 0.0], false));
        assert!(!reverses([1.0, 0.0], 1.0, [0.0, 0.0], true));
        // An intra-oceanic arc: buoyant crust arriving reverses it either way.
        assert!(reverses([1.0, 0.0], 0.0, [0.0, 0.0], false));
        assert!(reverses([1.0, 0.0], 0.0, [0.0, 0.0], true));
        // An overriding continent in the overlap: a collision either way.
        for by_arc in [false, true] {
            assert!(!reverses([1.0, 1.0], 1.0, [0.0, 0.0], by_arc));
            // Oceanic crust arriving never reverses.
            assert!(!reverses([0.0, 0.0], 0.0, [0.0, 0.0], by_arc));
        }
        // Locked (freshly sutured) crust arriving reverses an unlocked overriding continent,
        // whatever the arc stands on.
        for by_arc in [false, true] {
            assert!(reverses([1.0, 1.0], 1.0, [1.0, 0.0], by_arc));
        }
    }
}

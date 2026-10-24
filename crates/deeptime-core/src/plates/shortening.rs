// SPDX-License-Identifier: LGPL-3.0-or-later
//! Real shortening and the collision state (design `docs/design/plate-force-balance.md` §9, task 2;
//! diagnosis in `docs/design/plate-collisions-and-welding.md` §2.3).
//!
//! # What is counted
//! The **collision front** of a contact is the part of the overriding plate's edge that
//! continental crust of the subducting plate stands at now: the overriding plate's visible samples
//! with a lattice neighbour of another plate, whose nearest sample of another plate (within
//! [`ADJACENT_SPACINGS`] sample spacings) belongs to the contact's subducting plate, is
//! continental and is not freshly sutured crust (`subduction::locked`). The edge of a plate stays
//! fixed in its frame between resamplings, so the front does too; what stands at it is looked up
//! in this step's positions, so it does not depend on which plate the lattice put there at the
//! last resampling.
//!
//! The **convergence** across the front is the flux of the subducting plate's velocity relative
//! to the overriding plate through it, over its length. The edge of a plate is the chain of
//! Voronoi edges between its samples and their lattice neighbours of other plates; each such
//! lattice edge contributes its Voronoi edge's length ([`voronoi_edge_rad`]) times the component
//! of the relative velocity along it, inwards ([`normal_convergence`]). Summed along the front
//! that is exactly the crust crossing a straight line between the front's ends, however jagged the
//! lattice makes the edge: plates sliding past each other count nothing, an oblique contact counts
//! its normal part, and a parting one is negative. Divided by the front's length (the length of
//! that straight line, [`Front::length_km`]) it is the convergence in mm/yr. (A first version took,
//! per edge sample, the velocity along the mean direction to its foreign neighbours: on the
//! Fibonacci lattice those lean one way along a boundary, and pure strike-slip read 19 % of its
//! speed as convergence.) Shortening is the convergence integrated over time, parting steps adding
//! nothing (crust that went under does not come back).
//!
//! This replaces the count that `subduction::resolve` kept until 2026-10-02 (`depth_km`): the
//! magnitude of the relative speed at the overlap's centroid, times time, from the moment three
//! continental samples overlapped. It read 350 km when 87 km of crust had gone under.
//!
//! # The collision state
//! A contact is *in collision* ([`Collision`], `Contact::collision`) from the first step its front
//! has at least [`MIN_CONTACT`] samples and converges, while the contact is live (overlap seen
//! this step). The state then lives through steps with no front (a resampling consumes the
//! overlap; it takes a few steps to come back) for up to [`GRACE_MYR`]. It ends in one of:
//!
//! | End | Where | Record |
//! |---|---|---|
//! | the crust is transferred or the plates weld | `collision::collide` | a `CollisionRec` |
//! | no continental crust at the front for longer than [`GRACE_MYR`] (`parted`) | [`advance`] | a `PartingRec` |
//! | the contact reverses (`reversed`) | `subduction::reverse` | a `PartingRec` |
//! | the contact closes: no overlap for longer than `contact_timeout_myr`, or a plate died (`closed`) | `subduction::close_contacts` | a `PartingRec` |
//! | the run ends (`open`) | `subduction::close_contacts` | a `PartingRec` |
//!
//! # What ends it in a transfer or weld (tasks 3 and 4)
//! While the state lives, `belts.rs` stacks each step's shortened crust into a belt and a wedge,
//! and the belt resists the convergence through the force balance (`forces.rs`). [`advance`]
//! hands a contact to `collision::collide` when
//!
//! - its convergence across the front has stayed under `tectonics.weld_speed_mm_yr`, either way,
//!   for
//!   `tectonics.weld_after_myr` (the collision has **stalled**): a continent of
//!   `collision_suture_frac` or more welds its plate to the other one, the larger plate keeping
//!   its id; a smaller terrane moves alone;
//! - or its shortening has reached `tectonics.collision_threshold_km`: a terrane smaller than
//!   `collision_suture_frac` moves to the overriding plate (Cortial's transfer, on real
//!   shortening); a continent does not (it goes on converging until it stalls).
//!
//! The terrane's seeds are the continental samples of the subducting plate under the overriding
//! plate and those standing at the front ([`Front::crust`]); at least [`MIN_CONTACT`] are needed.
//! The count used before task 2 (relative speed × time from the first overlap) and the switch
//! between the two counts (`collision_on_shortening`) were removed with tasks 3 and 4.

use std::collections::BTreeMap;

use glam::{DQuat, DVec3};
use rayon::prelude::*;

use super::Env;
use super::dynamics;
use super::geometry::{Geo, Tree};
use super::kinematics::{quat, tangent};
use super::state::{CONTINENTAL, Collision, PlateState, Sample};
use super::subduction::{CONFIRM_MM_YR, MIN_CONTACT, locked_against};

/// A collision begins only where its front converges at least this fast, mm/yr: the rate a
/// contact needs to become a subduction zone (`subduction::CONFIRM_MM_YR`). Slower contact is a
/// front that only touches, or slides more than it closes; counted from there, the onset
/// convergence of the check's "convergence 20 Myr after onset ÷ at onset" was a few mm/yr and the
/// ratio a median 3-8 (2026-10-02). Once begun, every converging step counts, however slow.
// ledger: PLT-224 (game-choice: the existing subduction-confirmation rate).
pub const BEGIN_MM_YR: f64 = CONFIRM_MM_YR;
/// A collision lives this long without a front before it counts as parted, Myr (3 steps at the
/// 2-Myr step).
// ledger: PLT-225 (game-choice: long enough for an overlap to come back after a resampling).
pub const GRACE_MYR: f64 = 6.0;

/// The steps without a front a collision lives through at a step of `dt_myr`: as many as fit in
/// [`GRACE_MYR`] (3 at 2 Myr, 1 at 4 Myr: it parts after 8 Myr either way).
pub fn grace_steps(dt_myr: f64) -> u64 {
    (GRACE_MYR / dt_myr).floor() as u64
}
/// Samples looked at around a front sample for the crust that stands at it.
// ledger: PLT-223 (game-choice: a numerical rule).
const K_FRONT: usize = 12;
/// Crust of another plate stands at an edge sample when it is within this many sample spacings.
// ledger: PLT-223 (game-choice: a numerical rule).
pub const ADJACENT_SPACINGS: f64 = 1.75;
/// A collision records its convergence this long after its onset, Myr (the design's measure of
/// a collision's slowdown: convergence 20 Myr after the onset over convergence at the onset).
// ledger: PLT-226 (the measure's definition, design `plate-force-balance.md` §7).
pub const SLOWDOWN_AFTER_MYR: f64 = 20.0;

/// Convergence across a boundary, in the units of `rel`: the component of the incoming plate's
/// velocity relative to the overriding plate (`rel`) along the boundary's inward normal. `out` is
/// the unit normal pointing out of the overriding plate. Positive: converging; negative: parting;
/// zero for motion along the boundary.
// ledger: PLT-222 (with `fronts` and `voronoi_edge_rad`: geometry, no outside source).
pub fn normal_convergence(rel: DVec3, out: DVec3) -> f64 {
    -rel.dot(out)
}

/// Length, radians, of the Voronoi edge between lattice neighbours `v` and `u`: the stretch of
/// boundary a lattice edge stands for when its ends lie in different plates. From the cotangent
/// formula over the two triangles on the edge (the common neighbour with the widest angle on
/// each side, which is the Delaunay triangle's apex).
pub fn voronoi_edge_rad(geo: &Geo, v: u32, u: u32) -> f64 {
    let pts = geo.points();
    let (a, b) = (pts[v as usize], pts[u as usize]);
    let side = a.cross(b);
    // Per side of the edge: the smallest cotangent (the widest angle) among common neighbours.
    let mut cot = [f64::INFINITY; 2];
    for (w, _) in geo.edges(v) {
        if w == u || !geo.edges(u).any(|(x, _)| x == w) {
            continue;
        }
        let c = pts[w as usize];
        let (p, q) = (a - c, b - c);
        let sin = p.cross(q).length();
        if sin <= 0.0 {
            continue;
        }
        let k = usize::from(c.dot(side) < 0.0);
        cot[k] = cot[k].min(p.dot(q) / sin);
    }
    let sum: f64 = cot.iter().filter(|c| c.is_finite()).sum();
    (0.5 * (b - a).length() * sum).max(0.0)
}

/// Σ over the lattice edges from sample `v` (at `x`, its group turned by `rot`) to samples of
/// plate `pu`: the edge's Voronoi edge length ([`voronoi_edge_rad`]) times its direction. A
/// Voronoi edge is square to its lattice edge, so along a chain of boundary samples these sums
/// add up to the chord of that stretch of the boundary turned a right angle: its normal times its
/// straight length, however the lattice zigzags. Zero where `v` has no neighbour on `pu`.
pub(crate) fn crossing_sum(
    geo: &Geo,
    x: DVec3,
    rot: DQuat,
    plate_of: &[u32],
    v: u32,
    pu: u32,
) -> DVec3 {
    let pts = geo.points();
    let mut sum = DVec3::ZERO;
    for (u, _) in geo.edges(v) {
        if plate_of[u as usize] == pu {
            let out = tangent(x, rot * (pts[u as usize] - pts[v as usize])).normalize_or_zero();
            sum += voronoi_edge_rad(geo, v, u) * out;
        }
    }
    sum
}

/// The plate boundary's unit normal at sample `v`, pointing to plate `pu` (zero where `v` is not
/// on that boundary): the crossing sums ([`crossing_sum`]) of `v` and of the visible samples of
/// its own plate within two lattice steps, which give the chord of about five spacings of the
/// boundary. One sample's own edges give no usable normal: the lattice's boundary zigzags at its
/// spacing, and on the (near-hexagonal) Fibonacci lattice the edges that cross a straight
/// boundary lie about 30 degrees either side of its normal. ledger: PLT-310
pub fn boundary_normal(
    geo: &Geo,
    pos: &[DVec3],
    rots: &[DQuat],
    samples: &[Sample],
    plate_of: &[u32],
    v: usize,
    pu: u32,
) -> DVec3 {
    let pv = plate_of[v];
    let mut near: Vec<u32> = Vec::with_capacity(24);
    near.push(v as u32);
    let mut ring = 0..1;
    for _ in 0..2 {
        for i in ring.clone() {
            for (w, _) in geo.edges(near[i]) {
                let wi = w as usize;
                if plate_of[wi] == pv && !samples[wi].is_hidden() && !near.contains(&w) {
                    near.push(w);
                }
            }
        }
        ring = ring.end..near.len();
    }
    let mut n = DVec3::ZERO;
    for &w in &near {
        let wi = w as usize;
        let rot = rots[samples[wi].group as usize];
        n += crossing_sum(geo, pos[wi], rot, plate_of, w, pu);
    }
    tangent(pos[v], n).normalize_or_zero()
}

/// Sample `v`'s share of the straight boundary to plate `pu`, radians: its crossing sum
/// ([`crossing_sum`]) on the boundary's unit normal `normal` ([`boundary_normal`]), at least 0.
/// Along a boundary the crossing sums add up to its normal times its straight length, so the
/// shares of one side's samples add up to the boundary's length with the lattice's zigzag taken
/// out (where one sample spacing per sample counts a straight boundary about 1.04 times on the
/// Fibonacci lattice). ledger: PLT-530
#[allow(clippy::too_many_arguments)]
pub fn front_share_rad(
    geo: &Geo,
    pos: &[DVec3],
    rots: &[DQuat],
    samples: &[Sample],
    plate_of: &[u32],
    v: usize,
    pu: u32,
    normal: DVec3,
) -> f64 {
    let rot = rots[samples[v].group as usize];
    crossing_sum(geo, pos[v], rot, plate_of, v as u32, pu)
        .dot(normal)
        .max(0.0)
}

/// Bird's test of a transform: a boundary whose relative motion lies within 20 degrees of its
/// own direction is strike-slip. True when the normal part of `rel` (mm/yr, against the unit
/// normal `normal`) is under sin 20° of its speed, or the plates do not move. Bird calls a
/// converging step with a Benioff zone a subduction zone whatever its angle; a model contact
/// belongs to a pair of plates, not to a stretch of their boundary, so the test stands in for the
/// Benioff zone along a contact's strike-slip stretches. ledger: PLT-311
pub fn is_transform(rel: DVec3, normal: DVec3) -> bool {
    // ledger: PLT-311 (Bird 2003, paragraph [127]: within ±20 degrees of the step's azimuth)
    const TRANSFORM_DEG: f64 = 20.0;
    let speed = rel.length();
    speed <= 0.0 || rel.dot(normal).abs() < TRANSFORM_DEG.to_radians().sin() * speed
}

/// One contact's collision front in a step.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Front {
    /// Samples of the overriding plate's edge on the front, and those of them that are
    /// continental themselves.
    pub samples: u32,
    pub continental_over: u32,
    /// Flux of the relative velocity through the front, inwards: Σ over the front's lattice edges
    /// of Voronoi edge length × convergence along the edge, km × mm/yr (signed).
    pub flux: f64,
    /// Σ Voronoi edge length × outward edge direction, km: its length is the straight distance
    /// between the front's ends, its direction the front's mean outward normal.
    pub normal_sum: DVec3,
    /// Σ Voronoi edge length, km: the front's length along the jagged edge.
    pub edge_km: f64,
    /// Σ position over the samples.
    pub position_sum: DVec3,
    /// The subducting plate's continental samples standing at the front, sorted, each once.
    pub crust: Vec<u32>,
    /// The overriding plate's edge samples on the front, in sample order (the belt's sources).
    pub edge: Vec<u32>,
}

impl Front {
    /// The front's length, km: the edge's length with the lattice's jags taken out (for a front
    /// along a line, its length; for one that turns, the straight distance between its ends), and
    /// at least half its length along the edge (a front that wraps round a promontory has ends
    /// close together).
    pub fn length_km(&self) -> f64 {
        self.normal_sum.length().max(0.5 * self.edge_km)
    }

    /// Convergence across the front, mm/yr (signed): its flux over its length.
    pub fn convergence_mm_yr(&self) -> f64 {
        let length = self.length_km();
        if length > 0.0 {
            self.flux / length
        } else {
            0.0
        }
    }

    /// The front's unit centroid.
    pub fn centroid(&self) -> DVec3 {
        self.position_sum.normalize_or_zero()
    }

    /// The mean outward normal at the centroid, a unit tangent (zero for a closed front).
    pub fn normal(&self) -> DVec3 {
        tangent(self.centroid(), self.normal_sum).normalize_or_zero()
    }
}

/// One edge sample's part of a front.
struct Piece {
    contact: usize,
    flux: f64,
    normal: DVec3,
    edge_km: f64,
    continental: bool,
    crust: u32,
}

/// The collision front of every contact that has one this step, by contact index (see the module
/// docs). Reads the state; parallel over samples, summed in sample order.
pub fn fronts(
    state: &PlateState,
    geo: &Geo,
    pos: &[DVec3],
    tree: &Tree,
    env: &Env,
) -> BTreeMap<usize, Front> {
    let n = state.samples.len();
    let norm = env.norm;
    let plate_of: Vec<u32> = (0..n).map(|i| state.plate_of(i)).collect();
    let omega: Vec<DVec3> = state
        .plates
        .iter()
        .map(|p| DVec3::from_array(p.omega))
        .collect();
    let rots: Vec<DQuat> = state.groups.iter().map(|g| quat(g.rot)).collect();
    let pts = geo.points();
    let mut pair: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for (i, c) in state.contacts.iter().enumerate() {
        pair.insert((c.under, c.over), i);
    }
    if pair.is_empty() {
        return BTreeMap::new();
    }
    // A collision's own belt and wedge are stamped each step; they stay its front.
    let own: Vec<Option<f64>> = state
        .contacts
        .iter()
        .map(|c| c.collision.as_ref().map(|col| col.onset_myr))
        .collect();
    let reach = ADJACENT_SPACINGS * norm.spacing_rad();
    // Squared chord of the reach (the tree's distances are squared chords).
    let reach2 = (2.0 * (0.5 * reach).sin()).powi(2);
    let samples = &state.samples;
    let found: Vec<Option<Piece>> = (0..n)
        .into_par_iter()
        .map(|v| {
            let sv = &samples[v];
            let pv = plate_of[v];
            if sv.is_hidden() || !geo.edges(v as u32).any(|(u, _)| plate_of[u as usize] != pv) {
                return None;
            }
            let x = pos[v];
            // The crust that stands at this edge sample: the nearest sample of another plate.
            let (j, d2) = tree
                .nearest_n(x, K_FRONT)
                .into_iter()
                .find(|&(j, _)| plate_of[j as usize] != pv)?;
            if d2 > reach2 {
                return None;
            }
            let pu = plate_of[j as usize];
            let &contact = pair.get(&(pu, pv))?;
            let sj = &samples[j as usize];
            if sj.kind != CONTINENTAL || locked_against(sj, env.t_myr, env, own[contact]) {
                return None;
            }
            let rel = norm.velocity_mm_yr(omega[pu as usize] - omega[pv as usize], x);
            let rot = rots[sv.group as usize];
            let mut piece = Piece {
                contact,
                flux: 0.0,
                normal: DVec3::ZERO,
                edge_km: 0.0,
                continental: sv.kind == CONTINENTAL,
                crust: j,
            };
            for (u, _) in geo.edges(v as u32) {
                if plate_of[u as usize] == pv {
                    continue;
                }
                let out = tangent(x, rot * (pts[u as usize] - pts[v])).normalize_or_zero();
                let len = norm.km(voronoi_edge_rad(geo, v as u32, u));
                piece.flux += len * normal_convergence(rel, out);
                piece.normal += len * out;
                piece.edge_km += len;
            }
            Some(piece)
        })
        .collect();
    let mut fronts: BTreeMap<usize, Front> = BTreeMap::new();
    for (v, piece) in found.into_iter().enumerate() {
        if let Some(p) = piece {
            let e = fronts.entry(p.contact).or_default();
            e.samples += 1;
            e.continental_over += u32::from(p.continental);
            e.flux += p.flux;
            e.normal_sum += p.normal;
            e.edge_km += p.edge_km;
            e.position_sum += pos[v];
            e.crust.push(p.crust);
            e.edge.push(v as u32);
        }
    }
    for f in fronts.values_mut() {
        f.crust.sort_unstable();
        f.crust.dedup();
    }
    fronts
}

/// What a step did to a contact's collision state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// No collision before or after.
    None,
    /// A collision began this step.
    Began,
    /// The collision went on (with or without a front this step).
    Continued,
    /// No front for longer than [`GRACE_MYR`]: the caller records the parting and clears
    /// the state.
    Parted,
}

/// The step a collision is advanced by: its number, the time at its end and its length, and the
/// convergence below which the collision counts as stalled (`weld_speed_mm_yr`).
#[derive(Clone, Copy, Debug)]
pub struct StepClock {
    pub step: u64,
    pub t_myr: f64,
    pub dt_myr: f64,
    pub stall_mm_yr: f64,
}

/// Advances one contact's collision state by a step: `front` is its collision front this step (if
/// any), `live` whether the contact's overlap was seen this step. A collision begins only on a
/// live contact whose front has at least [`MIN_CONTACT`] samples and converges at
/// [`BEGIN_MM_YR`] or more; once begun it
/// accumulates the convergence of every step that has a front (parting steps add nothing).
pub fn step_collision(
    collision: &mut Option<Collision>,
    front: Option<&Front>,
    live: bool,
    clock: StepClock,
) -> Change {
    // ledger: PLT-224 (when a collision begins and what it accumulates).
    let front = front.filter(|f| f.samples >= MIN_CONTACT);
    let Some(f) = front else {
        return match collision {
            Some(c) if clock.step.saturating_sub(c.last_step) > grace_steps(clock.dt_myr) => {
                Change::Parted
            }
            Some(_) => Change::Continued,
            None => Change::None,
        };
    };
    let conv = f.convergence_mm_yr();
    let began = collision.is_none();
    if began {
        if !live || conv < BEGIN_MM_YR {
            return Change::None;
        }
        *collision = Some(Collision {
            onset_myr: clock.t_myr,
            onset_step: clock.step,
            last_step: clock.step,
            last_myr: clock.t_myr,
            shortening_km: 0.0,
            area_km2: 0.0,
            convergence_mm_yr: conv,
            onset_convergence_mm_yr: conv,
            peak_convergence_mm_yr: conv,
            later_convergence_mm_yr: None,
            front_km: 0.0,
            peak_front_km: 0.0,
            centroid: [0.0; 3],
            normal: [0.0; 3],
            continental_over: 0.0,
            stalled_since_myr: None,
            jam_km: None,
            belt: super::state::Belt::default(),
        });
    }
    let c = collision.as_mut().expect("set above");
    let front_km = f.length_km();
    // mm/yr × Myr = km.
    let shortened = conv.max(0.0) * clock.dt_myr;
    c.last_step = clock.step;
    c.last_myr = clock.t_myr;
    c.shortening_km += shortened;
    c.area_km2 += shortened * front_km;
    c.convergence_mm_yr = conv;
    c.peak_convergence_mm_yr = c.peak_convergence_mm_yr.max(conv);
    if c.later_convergence_mm_yr.is_none() && clock.t_myr - c.onset_myr >= SLOWDOWN_AFTER_MYR - 1e-9
    {
        c.later_convergence_mm_yr = Some(conv);
    }
    c.front_km = front_km;
    c.peak_front_km = c.peak_front_km.max(front_km);
    c.centroid = f.centroid().to_array();
    c.normal = f.normal().to_array();
    c.continental_over = f64::from(f.continental_over) / f64::from(f.samples);
    // ledger: PLT-246 (the stall that welds: under `weld_speed_mm_yr` either way, timed from its
    // first step). Since the switch to the force balance two plates in collision can also move
    // apart (a slab pulling one end swings the plate about the collision); that is no stall, and
    // the state ends as `parted` once the front is gone.
    if conv.abs() < clock.stall_mm_yr {
        c.stalled_since_myr.get_or_insert(clock.t_myr);
    } else {
        c.stalled_since_myr = None;
    }
    if began {
        Change::Began
    } else {
        Change::Continued
    }
}

/// A contact whose transfer or weld may fire this step ([`advance`]): its index, the terrane's
/// seeds, and whether the collision has stalled long enough to weld.
#[derive(Clone, Debug, PartialEq)]
pub struct Fired {
    pub contact: usize,
    pub seeds: Vec<u32>,
    pub stalled: bool,
}

/// Advances every contact's collision state by this step. Returns this step's fronts (by contact
/// index; the belts are stacked on them) and the contacts whose transfer or weld may fire now (see
/// the module docs), with the terrane's seeds: the subducting plate's continental samples in the
/// overlap (`seeds`, per contact, from `subduction::resolve`) and those at the front.
pub fn advance(
    state: &mut PlateState,
    geo: &Geo,
    pos: &[DVec3],
    tree: &Tree,
    env: &Env,
    mut seeds: BTreeMap<usize, Vec<u32>>,
) -> (BTreeMap<usize, Front>, Vec<Fired>) {
    let fronts = fronts(state, geo, pos, tree, env);
    let clock = StepClock {
        step: env.step,
        t_myr: env.t_myr,
        dt_myr: env.dt_myr,
        stall_mm_yr: env.k.weld_speed_mm_yr,
    };
    for k in 0..state.contacts.len() {
        let live = state.contacts[k].last_step == env.step;
        let change = step_collision(
            &mut state.contacts[k].collision,
            fronts.get(&k),
            live,
            clock,
        );
        match change {
            Change::Began => state.counters.collision_onsets += 1,
            Change::Parted => end(state, k, dynamics::PARTED),
            Change::None | Change::Continued => {}
        }
    }
    // Transference (ledger PLT-516): a collision's subduction has failed once its contact has no
    // slab left (it broke off, PLT-184, or never grew); from the step that is first seen the
    // plate pushes the continent, and the shortening since counts towards the critical
    // convergence (`collision::transference_due`). The slabs are those of the last balance.
    if env.k.transference {
        let live: std::collections::BTreeSet<(u32, u32)> = state
            .slabs
            .iter()
            .filter(|s| s.km > 0.0)
            .map(|s| (s.under, s.over))
            .collect();
        for c in &mut state.contacts {
            let key = (c.under, c.over);
            if let Some(col) = c.collision.as_mut()
                && col.jam_km.is_none()
                && !live.contains(&key)
            {
                col.jam_km = Some(col.shortening_km);
            }
        }
    }
    // ledger: PLT-068 (the threshold, for terrane transfers), PLT-246 (the weld on stall).
    let threshold = env.k.collision_threshold_km;
    let mut fired = Vec::new();
    for (k, c) in state.contacts.iter().enumerate() {
        let Some(col) = &c.collision else { continue };
        let stalled = col
            .stalled_since_myr
            .is_some_and(|t0| env.t_myr - t0 >= env.k.weld_after_myr - 1e-9);
        if !stalled && col.shortening_km < threshold {
            continue;
        }
        let mut list = seeds.remove(&k).unwrap_or_default();
        if let Some(f) = fronts.get(&k) {
            list.extend_from_slice(&f.crust);
        }
        list.sort_unstable();
        list.dedup();
        if list.len() as u32 >= MIN_CONTACT {
            fired.push(Fired {
                contact: k,
                seeds: list,
                stalled,
            });
        }
    }
    (fronts, fired)
}

/// Ends contact `k`'s collision (if it is in one) without a transfer or weld, recording why.
pub fn end(state: &mut PlateState, k: usize, reason: &str) {
    if let Some(col) = state.contacts[k].collision.take() {
        dynamics::record_parting(&mut state.dynamics, &state.contacts[k], &col, reason);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{PlanetPreset, plates_builtin};
    use crate::plates::normalise::Norm;
    use crate::plates::state::{Contact, Group, OROGENY_HIMALAYAN, Plate, Sample};
    use crate::preset::{Preset, PresetBody, with_timeline};

    const N: usize = 16_000;

    fn preset() -> PlanetPreset {
        let json = with_timeline(
            plates_builtin("plates_quick_16k").unwrap(),
            Some("supercontinent"),
            Some(100.0),
        )
        .unwrap();
        match Preset::from_json(&json).unwrap().body {
            PresetBody::Planet(p) => *p,
            other => panic!("not a planet preset: {other:?}"),
        }
    }

    fn plate(id: u32, omega: DVec3, centroid: DVec3) -> Plate {
        Plate {
            id,
            alive: true,
            omega: omega.to_array(),
            born_myr: 0.0,
            died_myr: None,
            birth_event: None,
            parent: None,
            lip_until_myr: 0.0,
            lip_event: None,
            samples: 0,
            centroid: centroid.to_array(),
            continental_frac: 1.0,
        }
    }

    /// Two plates split by the plane x = 0: plate 0 (x < 0) overrides, plate 1 (x ≥ 0) comes in
    /// with angular velocity `omega` (rad/yr). Crust is continental where `continental(p)`.
    fn two_plates(geo: &Geo, omega: DVec3, continental: impl Fn(DVec3) -> bool) -> PlateState {
        let samples: Vec<Sample> = geo
            .points()
            .iter()
            .map(|&p| {
                let mut s = Sample::blank(u32::from(p.x >= 0.0));
                if continental(p) {
                    s.kind = CONTINENTAL;
                }
                s
            })
            .collect();
        let ident = [0.0, 0.0, 0.0, 1.0];
        PlateState {
            n: N as u32,
            lattice: ident,
            samples,
            groups: vec![
                Group {
                    plate: 0,
                    rot: ident,
                },
                Group {
                    plate: 1,
                    rot: ident,
                },
            ],
            plates: vec![
                plate(0, DVec3::ZERO, DVec3::NEG_X),
                plate(1, omega, DVec3::X),
            ],
            contacts: vec![Contact {
                under: 1,
                over: 0,
                onset_myr: 0.0,
                onset_step: 1,
                onset_event: None,
                onset_slab_age_myr: 0.0,
                onset_convergence_mm_yr: 0.0,
                setting: 2,
                last_step: 1,
                samples: 3,
                collision: None,
                max_uplift_mm_yr: 0.0,
                andean: false,
                centroid: [0.0, 1.0, 0.0],
                cont_in: 0,
                reversed: false,
            }],
            hotspots: Vec::new(),
            ridges: Vec::new(),
            rift_links: Vec::new(),
            rng: crate::sphere::SplitMix64::new(1),
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
            level_set: None,
        }
    }

    struct Fixture {
        preset: PlanetPreset,
        geo: Geo,
        norm: Norm,
    }

    impl Fixture {
        fn new() -> Fixture {
            let preset = preset();
            let norm = Norm::new(preset.tectonics.model_radius_km, 6371.0, N);
            Fixture {
                geo: Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap(),
                preset,
                norm,
            }
        }

        fn env(&self, step: u64) -> Env<'_> {
            Env {
                k: &self.preset.tectonics,
                norm: self.norm,
                dt_myr: 2.0,
                dt_yr: 2.0e6,
                gravity: 9.81,
                step,
                t_myr: 2.0 * step as f64,
            }
        }

        /// Angular speed (rad/yr) that moves a point 90° from the pole at `mm_yr`.
        fn rate(&self, mm_yr: f64) -> f64 {
            mm_yr / self.norm.speed_mm_yr(DVec3::Z, DVec3::X)
        }

        fn front(&self, st: &PlateState, step: u64) -> Option<Front> {
            let pos = crate::plates::geometry::positions(st, &self.geo);
            let tree = Tree::over(&pos, |_| true);
            fronts(st, &self.geo, &pos, &tree, &self.env(step))
                .get(&0)
                .cloned()
        }
    }

    /// The continental part of the boundary in these tests: the arc of the great circle x = 0
    /// within 30° of +y, where a rotation about z is close to head-on.
    fn arc(p: DVec3) -> bool {
        p.y > 0.866
    }

    #[test]
    fn the_normal_component_is_what_converges() {
        let out = DVec3::X;
        assert_eq!(normal_convergence(DVec3::new(-50.0, 0.0, 0.0), out), 50.0);
        assert_eq!(normal_convergence(DVec3::new(0.0, 70.0, 0.0), out), 0.0);
        assert_eq!(normal_convergence(DVec3::new(30.0, 70.0, 0.0), out), -30.0);
        let oblique = DVec3::new(-40.0, 30.0, 0.0);
        assert!((normal_convergence(oblique, out) - 40.0).abs() < 1e-12);
        assert!((oblique.length() - 50.0).abs() < 1e-12);
    }

    /// Head-on: the plate x ≥ 0 turns about +z, so at (0, y, z) it moves along −x at `v·y`, straight
    /// into the other plate. The front's convergence is the mean of that over the arc, and a
    /// collision accumulates speed × time.
    #[test]
    fn head_on_convergence_accumulates_speed_times_time() {
        let mut fx = Fixture::new();
        fx.preset.tectonics.collision_threshold_km = 1.0e9;
        let v = 60.0;
        let mut st = two_plates(&fx.geo, DVec3::Z * fx.rate(v), arc);
        let f = fx.front(&st, 1).expect("a front");
        // The arc is 60° of a great circle, about 6,670 km. Along a great circle the edge's
        // normals are parallel, so the front's length is the arc's (the front reaches a sample or
        // two past each end, where the continent's corner is the nearest crust); the jagged edge
        // is up to half as long again.
        let length = f.length_km();
        let arc_km = fx.norm.km(std::f64::consts::FRAC_PI_3);
        assert!(
            (0.98..1.1).contains(&(length / arc_km)),
            "front {length:.0} km for an arc of {arc_km:.0} km"
        );
        assert!(
            f.edge_km > 0.95 * arc_km && f.edge_km < 1.5 * arc_km,
            "edge {:.0} km for an arc of {arc_km:.0} km",
            f.edge_km
        );
        assert!(f.samples >= 30 && f.continental_over + 2 >= f.samples);
        // The crust at the front is the incoming plate's, continental, a row of it.
        assert!(f.crust.len() >= 15 && f.crust.len() <= f.samples as usize);
        assert!(f.crust.windows(2).all(|w| w[0] < w[1]));
        assert!(
            f.crust
                .iter()
                .all(|&j| st.plate_of(j as usize) == 1 && st.samples[j as usize].is_continental())
        );
        // The convergence is the mean of v·y over the front: v sin(a)/a for a front reaching an
        // angle a to each side of +y (0.955 v for the arc's 30°).
        let a = 0.5 * length / fx.norm.km(1.0);
        let expect = v * a.sin() / a;
        let conv = f.convergence_mm_yr();
        assert!(
            (conv / expect - 1.0).abs() < 0.02,
            "head-on convergence {conv:.1} mm/yr, expected {expect:.1}"
        );
        // The mean normal points from the overriding plate (x < 0) to the incoming one.
        assert!(f.normal().dot(DVec3::X) > 0.99, "{:?}", f.normal());
        assert!(f.centroid().dot(DVec3::Y) > 0.99);

        // Ten steps of 2 Myr through `advance`: shortening is convergence × 20 Myr.
        let pos = crate::plates::geometry::positions(&st, &fx.geo);
        let tree = Tree::over(&pos, |_| true);
        for step in 1..=10 {
            st.contacts[0].last_step = step;
            let (_, fired) = advance(
                &mut st,
                &fx.geo,
                &pos,
                &tree,
                &fx.env(step),
                BTreeMap::new(),
            );
            assert!(fired.is_empty(), "nothing fires below the threshold");
        }
        let c = st.contacts[0].collision.as_ref().expect("in collision");
        assert_eq!((c.onset_step, c.last_step), (1, 10));
        assert!((c.shortening_km - conv * 20.0).abs() < 1e-6);
        assert!((c.shortening_km / (expect * 20.0) - 1.0).abs() < 0.02);
        assert!((c.area_km2 - c.shortening_km * length).abs() < 1e-3);
        assert_eq!(st.counters.collision_onsets, 1);
    }

    /// Strike-slip: the plate x ≥ 0 turns about +x, so every point of the boundary x = 0 moves
    /// along it. The relative speed is `v` everywhere, the convergence is none, and no collision
    /// begins (the old count read speed × time here).
    #[test]
    fn a_strike_slip_contact_accumulates_nothing() {
        let fx = Fixture::new();
        let v = 60.0;
        let mut st = two_plates(&fx.geo, DVec3::X * fx.rate(v), arc);
        let f = fx.front(&st, 1).expect("a front");
        let conv = f.convergence_mm_yr();
        assert!(
            conv.abs() < 0.02 * v,
            "strike-slip convergence {conv:.2} mm/yr at {v} mm/yr"
        );
        let pos = crate::plates::geometry::positions(&st, &fx.geo);
        let tree = Tree::over(&pos, |_| true);
        for step in 1..=10 {
            st.contacts[0].last_step = step;
            advance(
                &mut st,
                &fx.geo,
                &pos,
                &tree,
                &fx.env(step),
                BTreeMap::new(),
            );
        }
        let shortening = st.contacts[0]
            .collision
            .as_ref()
            .map_or(0.0, |c| c.shortening_km);
        assert!(
            shortening < 0.02 * v * 20.0,
            "strike-slip shortening {shortening:.1} km of {} km slid",
            v * 20.0
        );
    }

    /// Oblique: a rotation about an axis between +z (head-on) and +x (strike-slip) counts only its
    /// head-on part, whatever the strike-slip part is.
    #[test]
    fn an_oblique_contact_accumulates_its_normal_part() {
        let fx = Fixture::new();
        let (head_on, slide) = (40.0, 55.0);
        let pure = two_plates(&fx.geo, DVec3::Z * fx.rate(head_on), arc);
        let expect = fx.front(&pure, 1).unwrap().convergence_mm_yr();
        for sign in [1.0, -1.0] {
            let omega = DVec3::Z * fx.rate(head_on) + DVec3::X * fx.rate(slide) * sign;
            let st = two_plates(&fx.geo, omega, arc);
            let conv = fx.front(&st, 1).unwrap().convergence_mm_yr();
            assert!(
                (conv - expect).abs() < 0.02 * slide,
                "oblique convergence {conv:.1} mm/yr, head-on part {expect:.1}"
            );
            // The relative speed, which the old count used, is far larger.
            let speed = fx.norm.speed_mm_yr(omega, DVec3::Y);
            assert!(speed > 1.5 * conv, "{speed} vs {conv}");
        }
        // Parting plates: negative convergence, and no collision begins.
        let mut st = two_plates(&fx.geo, DVec3::Z * -fx.rate(head_on), arc);
        let f = fx.front(&st, 1).unwrap();
        assert!((f.convergence_mm_yr() + expect).abs() < 1e-9);
        let pos = crate::plates::geometry::positions(&st, &fx.geo);
        let tree = Tree::over(&pos, |_| true);
        advance(&mut st, &fx.geo, &pos, &tree, &fx.env(1), BTreeMap::new());
        assert!(st.contacts[0].collision.is_none());
    }

    /// The front is where continental crust of the subducting plate stands at the overriding
    /// plate's edge: an oceanic incoming plate, freshly sutured crust or the wrong polarity give
    /// none; the overriding plate may be oceanic (a terrane arriving at an arc).
    #[test]
    fn the_front_is_where_continental_crust_comes_in() {
        let fx = Fixture::new();
        let omega = DVec3::Z * fx.rate(50.0);
        // Continental crust only on the overriding plate.
        let st = two_plates(&fx.geo, omega, |p| arc(p) && p.x < 0.0);
        assert_eq!(fx.front(&st, 1), None);
        // Only on the incoming plate: a front, none of it continental on the overriding side.
        let st = two_plates(&fx.geo, omega, |p| arc(p) && p.x >= 0.0);
        let f = fx.front(&st, 1).expect("a front");
        assert!(f.samples >= 30 && f.continental_over == 0);
        // Freshly sutured crust on the incoming plate cannot be forced under again.
        let mut st = two_plates(&fx.geo, omega, arc);
        let whole = fx.front(&st, 1).unwrap().samples;
        for s in st.samples.iter_mut().filter(|s| s.group == 1) {
            s.orogeny = OROGENY_HIMALAYAN;
            s.orogeny_myr = 0.0;
        }
        assert_eq!(fx.front(&st, 1), None);
        // The other polarity: the front is on the other plate's edge, about as long.
        let mut st = two_plates(&fx.geo, omega, arc);
        st.contacts[0].under = 0;
        st.contacts[0].over = 1;
        let f = fx.front(&st, 1).expect("a front");
        assert!(f.normal().dot(DVec3::NEG_X) > 0.99);
        assert!((f64::from(f.samples) / f64::from(whole) - 1.0).abs() < 0.2);
        // No contact between the two plates: no front.
        st.contacts.clear();
        assert_eq!(fx.front(&st, 1), None);
    }

    /// A straight front of `samples` samples 70 km apart, converging at `conv` mm/yr.
    fn front(samples: u32, conv: f64) -> Front {
        let length = 70.0 * f64::from(samples);
        Front {
            samples,
            continental_over: samples,
            flux: conv * length,
            normal_sum: DVec3::X * length,
            edge_km: length,
            position_sum: DVec3::Y * f64::from(samples),
            crust: Vec::new(),
            edge: Vec::new(),
        }
    }

    /// The grace is a time (P1-19 #27): 3 steps at 2 Myr, 1 at 4 Myr, 2 at 3 Myr; a collision
    /// with no front parts once more than 6 Myr have gone, after 8 Myr at 2 and 4 Myr steps.
    #[test]
    fn the_grace_is_a_time() {
        assert_eq!(grace_steps(2.0), 3);
        assert_eq!(grace_steps(4.0), 1);
        assert_eq!(grace_steps(3.0), 2);
        for dt in [2.0, 4.0] {
            let parted_after = (grace_steps(dt) + 1) as f64 * dt;
            assert_eq!(parted_after, 8.0);
        }
        assert_eq!(super::super::subduction::confirm_steps(2.0), 3);
        assert_eq!(super::super::subduction::confirm_steps(4.0), 2);
        assert_eq!(super::super::subduction::confirm_steps(3.0), 2);
    }

    fn clock(step: u64) -> StepClock {
        StepClock {
            step,
            t_myr: 2.0 * step as f64,
            dt_myr: 2.0,
            stall_mm_yr: 5.0,
        }
    }

    /// The boundary x = 0 seen from plate 0's edge: the normal points along +x, a pure
    /// strike-slip motion reads as a transform by Bird's 20 degrees, and head-on motion does not.
    #[test]
    fn the_boundary_normal_sees_a_transform() {
        let fx = Fixture::new();
        let v = 60.0;
        let st = two_plates(&fx.geo, DVec3::X * fx.rate(v), |_| false);
        let pos = crate::plates::geometry::positions(&st, &fx.geo);
        let rots: Vec<DQuat> = st.groups.iter().map(|g| quat(g.rot)).collect();
        let plate_of: Vec<u32> = (0..st.samples.len()).map(|i| st.plate_of(i)).collect();
        let (mut total, mut worst) = (0usize, 0.0f64);
        let mut slid = 0usize;
        let mut head_on = 0usize;
        for (i, x) in pos.iter().enumerate() {
            if plate_of[i] != 0
                || !fx
                    .geo
                    .edges(i as u32)
                    .any(|(u, _)| plate_of[u as usize] == 1)
            {
                continue;
            }
            total += 1;
            let n = boundary_normal(&fx.geo, &pos, &rots, &st.samples, &plate_of, i, 1);
            let want = tangent(*x, DVec3::X).normalize();
            worst = worst.max(n.dot(want).clamp(-1.0, 1.0).acos().to_degrees());
            let slide = fx.norm.velocity_mm_yr(DVec3::X * fx.rate(v), *x);
            slid += usize::from(is_transform(slide, n));
            // Head-on where the rotation about z moves the boundary fast (|y| > 0.5).
            let push = fx.norm.velocity_mm_yr(DVec3::Z * fx.rate(v), *x);
            head_on += usize::from(x.y.abs() <= 0.5 || !is_transform(push, n));
        }
        assert!(total > 100, "{total} boundary samples");
        assert!(
            worst < 20.0,
            "the normal is {worst:.1} degrees off at worst"
        );
        assert!(slid as f64 > 0.97 * total as f64, "{slid} of {total} slide");
        assert_eq!(head_on, total, "head-on read as a transform");
    }

    /// The Voronoi edges round a sample close its cell: their lengths times their directions sum
    /// to nothing, and the cells tile the sphere.
    #[test]
    fn voronoi_edges_close_every_cell() {
        let geo = Geo::build(4000, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let pts = geo.points();
        let mut area = 0.0;
        for v in 0..pts.len() as u32 {
            let x = pts[v as usize];
            let (mut sum, mut len) = (DVec3::ZERO, 0.0);
            for (u, d) in geo.edges(v) {
                let l = voronoi_edge_rad(&geo, v, u);
                assert!(l >= 0.0 && l.is_finite());
                assert_eq!(l, voronoi_edge_rad(&geo, u, v), "the edge is one edge");
                sum += l * tangent(x, pts[u as usize] - x).normalize_or_zero();
                len += l;
                // A cell is the kites of its edges: half the edge to the neighbour, times the
                // Voronoi edge, halved.
                area += 0.25 * f64::from(d) * l;
            }
            assert!(
                sum.length() < 0.02 * len,
                "cell {v} is open: {sum:?} of {len}"
            );
        }
        let sphere = 4.0 * std::f64::consts::PI;
        assert!((area / sphere - 1.0).abs() < 0.01, "cells cover {area}");
    }

    /// A boundary's length by its samples' straight shares (PLT-530): on great circles cut
    /// across the lattice at random, the shares of either side's boundary samples add up to the
    /// circle's length within 1.5 %. On a small circle the concave side reads short and the convex
    /// side long (a crossing edge meets the boundary's normal at a wider angle at its inner end,
    /// where the radii converge; about ±1.5 % at 30 degrees' radius, ±3 % at 15), and their mean
    /// is within 1.5 %. One sample spacing per boundary sample (the count before) is the lattice's
    /// factor over the straight length: 1.04 on average (printed). So a per-km rate laid on the
    /// shares (subduction erosion's 90 km³/km/Myr, PLT-441; the retreat's 2.5 km/Myr, PLT-522)
    /// gives its source's rate per km of the boundary.
    #[test]
    fn the_straight_shares_add_up_to_the_boundarys_length() {
        let fx = Fixture::new();
        let mut st = two_plates(&fx.geo, DVec3::ZERO, |_| false);
        let rots: Vec<DQuat> = st.groups.iter().map(|g| quat(g.rot)).collect();
        let spacing = fx.norm.spacing_rad();
        let mut rng = crate::sphere::SplitMix64::new(7);
        let mut count_ratio = Vec::new();
        // Great circles (cos 0) and small circles of 60, 30 and 15 degrees' radius.
        for cos in [0.0_f64, 0.0, 0.0, 0.0, 0.5, 0.5, 0.866, 0.966] {
            let axis = rng.unit_vector();
            let length = 2.0 * std::f64::consts::PI * (1.0 - cos * cos).sqrt();
            for (i, p) in fx.geo.points().iter().enumerate() {
                st.samples[i].group = u32::from(p.dot(axis) >= cos);
            }
            let pos = crate::plates::geometry::positions(&st, &fx.geo);
            let plate_of: Vec<u32> = (0..st.samples.len()).map(|i| st.plate_of(i)).collect();
            let mut ratio = [0.0; 2];
            for side in 0..2u32 {
                let (mut share, mut count) = (0.0, 0usize);
                for i in 0..pos.len() {
                    let edge = fx
                        .geo
                        .edges(i as u32)
                        .any(|(u, _)| plate_of[u as usize] == 1 - side);
                    if plate_of[i] != side || !edge {
                        continue;
                    }
                    let other = 1 - side;
                    let n = boundary_normal(&fx.geo, &pos, &rots, &st.samples, &plate_of, i, other);
                    share +=
                        front_share_rad(&fx.geo, &pos, &rots, &st.samples, &plate_of, i, other, n);
                    count += 1;
                }
                ratio[side as usize] = share / length;
                count_ratio.push(count as f64 * spacing / length);
                println!(
                    "circle cos {cos} side {side}: shares / length {:.4}, count {:.4}",
                    share / length,
                    count as f64 * spacing / length
                );
            }
            let mean = 0.5 * (ratio[0] + ratio[1]);
            assert!((mean - 1.0).abs() < 0.015, "circle cos {cos}: {ratio:?}");
            let tol = if cos == 0.0 { 0.015 } else { 0.04 };
            assert!(
                ratio.iter().all(|r| (r - 1.0).abs() < tol),
                "circle cos {cos}: {ratio:?}"
            );
        }
        let mean = count_ratio.iter().sum::<f64>() / count_ratio.len() as f64;
        println!("one spacing per boundary sample over the straight length: mean {mean:.3}");
        assert!((1.0..1.1).contains(&mean), "the count's factor {mean:.3}");
    }

    /// The life cycle: nothing begins without a live contact, a long enough front and convergence;
    /// a collision carries its onset, shortening, rate and front through the steps; steps with no
    /// front (a resampling) and parting steps add nothing; it ends after the grace steps.
    #[test]
    fn a_collision_begins_persists_and_parts() {
        let mut col: Option<Collision> = None;
        // Too short a front, a contact not seen this step, plates parting: no collision.
        assert_eq!(
            step_collision(&mut col, Some(&front(2, 50.0)), true, clock(1)),
            Change::None
        );
        assert_eq!(
            step_collision(&mut col, Some(&front(10, 50.0)), false, clock(1)),
            Change::None
        );
        assert_eq!(
            step_collision(&mut col, Some(&front(10, -5.0)), true, clock(1)),
            Change::None
        );
        // Converging, but slower than a subduction zone needs: a front that only touches.
        assert_eq!(
            step_collision(
                &mut col,
                Some(&front(10, 0.9 * BEGIN_MM_YR)),
                true,
                clock(1)
            ),
            Change::None
        );
        assert_eq!(step_collision(&mut col, None, true, clock(1)), Change::None);
        assert!(col.is_none());

        // It begins at step 5 and shortens by 50 mm/yr × 2 Myr = 100 km a step.
        assert_eq!(
            step_collision(&mut col, Some(&front(10, 50.0)), true, clock(5)),
            Change::Began
        );
        assert_eq!(
            step_collision(&mut col, Some(&front(12, 50.0)), true, clock(6)),
            Change::Continued
        );
        let c = col.clone().unwrap();
        assert_eq!((c.onset_step, c.onset_myr), (5, 10.0));
        assert_eq!((c.last_step, c.last_myr), (6, 12.0));
        assert_eq!(c.shortening_km, 200.0);
        assert_eq!(c.area_km2, 100.0 * 700.0 + 100.0 * 840.0);
        assert_eq!(front(12, 50.0).convergence_mm_yr(), 50.0);
        assert_eq!((c.front_km, c.peak_front_km), (840.0, 840.0));
        assert_eq!(c.onset_convergence_mm_yr, 50.0);
        assert_eq!(c.later_convergence_mm_yr, None);
        assert_eq!(c.centroid, [0.0, 1.0, 0.0]);
        assert_eq!(c.normal, [1.0, 0.0, 0.0]);
        assert_eq!(c.continental_over, 1.0);

        // A slower step counts its own rate; the state goes on when the contact is not live.
        assert_eq!(
            step_collision(&mut col, Some(&front(8, 10.0)), false, clock(7)),
            Change::Continued
        );
        let c = col.clone().unwrap();
        assert_eq!(c.shortening_km, 220.0);
        assert_eq!(
            (c.convergence_mm_yr, c.peak_convergence_mm_yr),
            (10.0, 50.0)
        );
        assert_eq!((c.front_km, c.peak_front_km), (560.0, 840.0));

        // Parting steps add nothing, and shortening never falls.
        assert_eq!(
            step_collision(&mut col, Some(&front(8, -30.0)), true, clock(8)),
            Change::Continued
        );
        let c = col.clone().unwrap();
        assert_eq!(c.shortening_km, 220.0);
        assert_eq!(c.convergence_mm_yr, -30.0);
        assert_eq!(c.last_step, 8);

        // No front (or too short a one) for the grace steps: it lives; one more: parted.
        for step in 9..=8 + grace_steps(2.0) {
            let f = front(1, 80.0);
            let seen = if step % 2 == 0 { Some(&f) } else { None };
            assert_eq!(
                step_collision(&mut col, seen, true, clock(step)),
                Change::Continued
            );
        }
        assert_eq!(col.as_ref().unwrap().last_step, 8);
        assert_eq!(col.as_ref().unwrap().shortening_km, 220.0);
        assert_eq!(
            step_collision(&mut col, None, true, clock(9 + grace_steps(2.0))),
            Change::Parted
        );

        // A front that comes back within the grace steps carries the collision on.
        let mut col: Option<Collision> = None;
        step_collision(&mut col, Some(&front(10, 50.0)), true, clock(1));
        step_collision(&mut col, None, false, clock(2));
        step_collision(&mut col, None, false, clock(3));
        assert_eq!(
            step_collision(&mut col, Some(&front(10, 50.0)), true, clock(4)),
            Change::Continued
        );
        let c = col.clone().unwrap();
        assert_eq!((c.onset_step, c.last_step), (1, 4));
        assert_eq!(c.shortening_km, 200.0);

        // The convergence 20 Myr after the onset (2 Myr) is the first seen from 22 Myr on, once.
        assert_eq!(c.later_convergence_mm_yr, None);
        step_collision(&mut col, Some(&front(10, 40.0)), true, clock(10));
        assert_eq!(col.as_ref().unwrap().later_convergence_mm_yr, None);
        step_collision(&mut col, Some(&front(10, 30.0)), true, clock(11));
        step_collision(&mut col, Some(&front(10, 20.0)), true, clock(12));
        let c = col.unwrap();
        assert_eq!(c.later_convergence_mm_yr, Some(30.0));
        assert_eq!(c.onset_convergence_mm_yr, 50.0);
    }

    /// A collision stalls when its convergence stays under `weld_speed_mm_yr` either way: the
    /// clock starts at the first slow step, a faster step (or the plates moving apart faster)
    /// resets it, a step with no front leaves it; `advance`
    /// hands the contact over as stalled once it has lasted `weld_after_myr`, whatever its
    /// shortening, and a strike-slip contact neither shortens nor stalls into a weld.
    #[test]
    fn a_stalled_collision_fires_as_stalled() {
        let mut col: Option<Collision> = None;
        step_collision(&mut col, Some(&front(10, 50.0)), true, clock(1));
        assert_eq!(col.as_ref().unwrap().stalled_since_myr, None);
        step_collision(&mut col, Some(&front(10, 3.0)), true, clock(2));
        assert_eq!(col.as_ref().unwrap().stalled_since_myr, Some(4.0));
        step_collision(&mut col, None, true, clock(3));
        step_collision(&mut col, Some(&front(10, -1.0)), true, clock(4));
        assert_eq!(col.as_ref().unwrap().stalled_since_myr, Some(4.0));
        step_collision(&mut col, Some(&front(10, 12.0)), true, clock(5));
        assert_eq!(col.as_ref().unwrap().stalled_since_myr, None);
        // Moving apart faster than the weld speed is no stall.
        step_collision(&mut col, Some(&front(10, 3.0)), true, clock(6));
        assert!(col.as_ref().unwrap().stalled_since_myr.is_some());
        step_collision(&mut col, Some(&front(10, -20.0)), true, clock(7));
        assert_eq!(col.as_ref().unwrap().stalled_since_myr, None);

        let mut fx = Fixture::new();
        fx.preset.tectonics.collision_threshold_km = 1.0e9;
        fx.preset.tectonics.weld_speed_mm_yr = 5.0;
        fx.preset.tectonics.weld_after_myr = 10.0;
        // Head-on at 20 mm/yr: a collision begins; then the plates stop.
        let mut st = two_plates(&fx.geo, DVec3::Z * fx.rate(20.0), arc);
        let pos = crate::plates::geometry::positions(&st, &fx.geo);
        let tree = Tree::over(&pos, |_| true);
        st.contacts[0].last_step = 1;
        let (fronts, fired) = advance(&mut st, &fx.geo, &pos, &tree, &fx.env(1), BTreeMap::new());
        assert!(fronts.contains_key(&0) && fired.is_empty());
        st.plates[1].omega = [0.0; 3];
        // Stalled from step 2 (4 Myr): handed over at step 7 (14 Myr), 10 Myr later.
        for step in 2..=7 {
            let (_, fired) = advance(
                &mut st,
                &fx.geo,
                &pos,
                &tree,
                &fx.env(step),
                BTreeMap::new(),
            );
            let c = st.contacts[0].collision.as_ref().unwrap();
            assert_eq!(c.stalled_since_myr, Some(4.0), "step {step}");
            assert_eq!(fired.is_empty(), step < 7, "step {step}");
            if let Some(f) = fired.first() {
                assert!(f.stalled && f.contact == 0 && f.seeds.len() >= MIN_CONTACT as usize);
            }
        }
        // Strike-slip: no collision begins, so nothing stalls and nothing fires.
        let mut st = two_plates(&fx.geo, DVec3::X * fx.rate(60.0), arc);
        for step in 1..=10 {
            st.contacts[0].last_step = step;
            let (_, fired) = advance(
                &mut st,
                &fx.geo,
                &pos,
                &tree,
                &fx.env(step),
                BTreeMap::new(),
            );
            assert!(fired.is_empty(), "step {step}");
        }
        assert!(st.contacts[0].collision.is_none());
    }

    /// `advance` fires a contact once its shortening has reached the threshold and the terrane
    /// has seeds, and not before; a parted collision is recorded and cleared.
    #[test]
    fn the_threshold_fires_on_real_shortening_and_a_parting_is_recorded() {
        let mut fx = Fixture::new();
        fx.preset.tectonics.collision_threshold_km = 300.0;
        let mut st = two_plates(&fx.geo, DVec3::Z * fx.rate(60.0), arc);
        st.dynamics = Some(dynamics::Dynamics::default());
        let pos = crate::plates::geometry::positions(&st, &fx.geo);
        let tree = Tree::over(&pos, |_| true);
        let threshold = fx.preset.tectonics.collision_threshold_km;
        let per_step = fx.front(&st, 1).unwrap().convergence_mm_yr() * 2.0;
        let need = (threshold / per_step).ceil() as u64;
        assert!(need >= 2, "the threshold takes more than one step");
        let at_front = fx.front(&st, 1).unwrap().crust;
        // Seeds from the overlap (`subduction::resolve`): one sample at the front, one elsewhere.
        let extra = (0..N as u32).find(|j| !at_front.contains(j)).unwrap();
        let seeds = || BTreeMap::from([(0usize, vec![extra, at_front[0]])]);
        for step in 1..=need {
            st.contacts[0].last_step = step;
            let (_, fired) = advance(&mut st, &fx.geo, &pos, &tree, &fx.env(step), seeds());
            assert_eq!(!fired.is_empty(), step == need, "step {step} of {need}");
            if let Some(f) = fired.first() {
                // The terrane's seeds: the overlap's and the crust at the front, each once.
                assert_eq!(f.contact, 0);
                assert!(!f.stalled);
                assert_eq!(f.seeds.len(), at_front.len() + 1);
                assert!(f.seeds.contains(&extra) && f.seeds.windows(2).all(|w| w[0] < w[1]));
            }
        }
        // It fires again while the state stands (the stage's `collide` ends it), on the front's
        // crust alone.
        st.contacts[0].last_step = need + 1;
        let (_, fired) = advance(
            &mut st,
            &fx.geo,
            &pos,
            &tree,
            &fx.env(need + 1),
            BTreeMap::new(),
        );
        assert_eq!(
            fired,
            vec![Fired {
                contact: 0,
                seeds: at_front.clone(),
                stalled: false
            }]
        );
        let shortening = st.contacts[0].collision.as_ref().unwrap().shortening_km;
        assert!(shortening >= threshold);

        // The continent leaves the front (here: it becomes oceanic): after the grace steps the
        // collision is over, with one parting record.
        for s in &mut st.samples {
            s.kind = crate::plates::state::OCEANIC;
        }
        let last = need + 1;
        for step in last + 1..=last + grace_steps(2.0) + 1 {
            assert!(st.contacts[0].collision.is_some(), "step {step}");
            advance(
                &mut st,
                &fx.geo,
                &pos,
                &tree,
                &fx.env(step),
                BTreeMap::new(),
            );
        }
        assert!(st.contacts[0].collision.is_none());
        let d = st.dynamics.as_ref().unwrap();
        assert_eq!(d.partings.len(), 1);
        let p = &d.partings[0];
        assert_eq!(p.reason, dynamics::PARTED);
        assert_eq!((p.start_myr, p.end_myr), (2.0, 2.0 * last as f64));
        assert_eq!((p.under, p.over), (1, 0));
        assert!((p.shortening_km - shortening).abs() < 1e-9);
        assert!(p.front_km > 0.0 && p.area_km2 > 0.0);
        assert_eq!(st.counters.collision_onsets, 1);
    }
}

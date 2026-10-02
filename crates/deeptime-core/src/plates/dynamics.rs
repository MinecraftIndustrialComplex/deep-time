// SPDX-License-Identifier: LGPL-3.0-or-later
//! The plate dynamics record (design `docs/design/plate-collisions-and-welding.md`, §6 and task 1):
//! what `deeptime plate-check` judges a run by, written by the plates stage as it runs.
//!
//! It replaces the throwaway trace of the design's measurements (`DEEPTIME_PLATE_TRACE`), and it
//! is measurement only: nothing here is read by a plate rule, so a run with and without it ends in
//! the same world (the test `recording_does_not_change_the_world_and_matches_the_run` in
//! `plates/tests.rs`). It costs one pass over the samples per step (counts, plate boundaries) and
//! one more at intervals (the sea-floor ages). Setting [`OFF_ENV`] switches it off, to time that.
//!
//! # Where it lives
//! In [`PlateState::dynamics`] (so a checkpoint carries it and a resumed run goes on from it) and,
//! copied as JSON, in the run's note `notes.plates.dynamics` beside `notes.plates.margins`
//! (`record_margins` rewrites the note every [`super::MARGIN_EVERY_MYR`] and at the end, so the
//! manifest of a finished run holds the whole record). `plates::check` and `deeptime plate-check`
//! read it.
//!
//! # The record (`notes.plates.dynamics`)
//! | key | meaning |
//! |---|---|
//! | `version` | [`VERSION`] |
//! | `t0_myr`, `dt_myr` | time before the first step, step length |
//! | `samples` | L0 samples, `n`: every share below is of this unless it says "visible" |
//! | `steps` | one [`StepRec`] per step: the plates ([`PlateStep`]), land, crust, the sea floor's age percentiles, the largest continental pieces |
//! | `plates` | one [`PlateLife`] per plate that ever lived: parent, origin, death and its cause |
//! | `collisions` | one [`CollisionRec`] per terrane transfer, collisions included |
//! | `ages` | the sea-floor age distribution every [`AGE_EVERY_MYR`] ([`AgeRec`]) |
//! | `prev_omega` | each plate's rotation vector after the last step (for the next step's change) |
//! | `belts` | one [`BeltRec`] per collision belt: empty until the belt physics exists |
//!
//! Left empty by today's model, to be filled by the tasks that add the physics (the check reads
//! them and reports "no data" while they are empty): `StepRec::forces` (the force budget and mean
//! stress per plate, [`PlateForces`]), `StepRec::slabs_km` (slab length per contact) and
//! `Dynamics::belts` (convergence 20 Myr after onset, wedge and foredeep, [`BeltRec`]).
//!
//! Lengths are Earth-normalized km (the plate model's sphere, `model_radius_km`), speeds mm/yr,
//! times Myr relative to world creation. "Visible" samples are those not hidden under another
//! plate. The active margin share is not repeated here: `notes.plates.margins` has it.
//!
//! # What each collision must keep meaning
//! Tasks 2–4 of the design turn a collision from a one-step event into a state of a contact. The
//! record keeps its fields' meanings across that change: `start_myr` and `end_myr` are the **first
//! and last step** of the collision (equal today; the check's length is `end − start + dt`, so a
//! one-step collision is one step long, counted from the collision's first step and not from the
//! contact's onset, which is `contact_onset_myr`), `peak_uplift_m` the largest uplift it produced
//! anywhere, `shortening_km` the continental crust that went under over the front's length,
//! `lifted10`/`lifted100` the ground it raised by more than 10 and 100 m over its life, and
//! `emerged` the ground that rose from below the sea to above it.
//!
//! # Per plate and step (task 1b)
//! `rot_change` is the size of the change of the plate's rotation vector since the previous step
//! over the size of the previous one (`|Ω − Ω_prev| / |Ω_prev|`: direction and rate together); it
//! is known only when the plate existed at the previous step. `pole_inside` is whether the
//! plate's rotation pole (either end of its axis) lies on the plate: the visible sample nearest to
//! it belongs to the plate. `guard` is whether the speed guard (`max_plate_speed_mm_yr`) holds the
//! plate: its rotation is at the cap, which is where every clamp leaves it. A clamp that a later
//! one in the same step overwrote is not seen; the measure is the plate's state after the step.
//!
//! # Per step (task 1b)
//! The sea floor's median and 90th-percentile age and the counts older than 180 and 250 Myr are
//! recorded every step (one sort of the oceanic samples' ages). The histogram stays at the
//! [`AGE_EVERY_MYR`] cadence. `pieces` are the sizes, in visible samples, of the largest connected
//! pieces of continental crust, largest first (at most [`PIECES_KEPT`]); samples are joined along
//! lattice edges whose ends are less than [`PIECE_LINK_SPACINGS`] sample spacings apart now, so
//! two continents that a rift has pulled apart since the last resampling are not joined.

use std::collections::BTreeMap;

use deeptime_format::ParamValue;
use glam::DVec3;
use serde::{Deserialize, Serialize};

use super::Env;
use super::events::{PLATE_BIRTH, PLATE_DEATH};
use super::geometry::Geo;
use super::state::{CONTINENTAL, OCEANIC, PlateState};
use crate::world::WorldState;

/// Set this environment variable and the stage records nothing (to time what recording costs).
pub const OFF_ENV: &str = "DEEPTIME_NO_PLATE_DYNAMICS";
/// The record's format version (`notes.plates.dynamics.version`).
pub const VERSION: u32 = 2;
/// The sea-floor ages are recorded this often, Myr.
pub const AGE_EVERY_MYR: f64 = 10.0;
/// Width of an age bin, Myr.
pub const AGE_BIN_MYR: f64 = 10.0;
/// Age bins: 0–10, 10–20, … and the last one holds everything older than `(AGE_BINS − 1)` bins.
pub const AGE_BINS: usize = 41;
/// The two ages whose share of the sea floor is recorded every step, Myr.
pub const OLD_AGES_MYR: [f64; 2] = [180.0, 250.0];
/// How many of the largest continental pieces a step keeps.
pub const PIECES_KEPT: usize = 16;
/// Two continental samples joined by a lattice edge are one piece only while they are closer than
/// this many sample spacings.
pub const PIECE_LINK_SPACINGS: f64 = 2.0;
/// Bits of a [`PlateStep`]'s flags in its array form.
const POLE_INSIDE: u8 = 1;
const GUARD_BOUND: u8 = 2;
const ROT_KNOWN: u8 = 4;

/// The dynamics of a run, from the first step to the latest.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Dynamics {
    pub version: u32,
    /// Time before the first step, Myr.
    pub t0_myr: f64,
    /// Step length, Myr.
    pub dt_myr: f64,
    /// L0 samples.
    pub samples: u32,
    pub steps: Vec<StepRec>,
    pub plates: Vec<PlateLife>,
    pub collisions: Vec<CollisionRec>,
    pub ages: Vec<AgeRec>,
    /// How many of the world's events have been read (births and deaths come from them).
    pub events_seen: usize,
    /// Each plate's rotation vector after the last step, for the next step's `rot_change`.
    pub prev_omega: Vec<(u32, [f64; 3])>,
    /// Collision belts: convergence history, wedge and foredeep. Empty until the belt physics
    /// (design tasks 2-3) writes it.
    pub belts: Vec<BeltRec>,
}

/// One step's state of the planet and its plates.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StepRec {
    /// Time at the end of the step, Myr.
    pub t: f64,
    /// Visible samples.
    pub visible: u32,
    /// Visible samples above the plate model's sea datum (0 m): the land.
    pub land: u32,
    /// Visible continental samples: the continental crust.
    pub continental: u32,
    /// Collisions (terranes of at least `collision_min_frac` of an average plate) this step.
    pub collisions: u16,
    /// Other terrane transfers this step.
    pub transfers: u16,
    /// The plates alive at the end of the step.
    pub plates: Vec<PlateStep>,
    /// Visible oceanic samples, the median and 90th-percentile age of their crust (Myr) and how
    /// many are older than [`OLD_AGES_MYR`]`[0]` and `[1]`.
    pub ocean: u32,
    pub age_p50: f64,
    pub age_p90: f64,
    pub old180: u32,
    pub old250: u32,
    /// Sizes of the largest connected pieces of continental crust, in visible samples, largest
    /// first (see the module docs).
    pub pieces: Vec<u32>,
    /// Per-plate force budget and stress. Empty until the force balance (design task 5) writes it.
    pub forces: Vec<PlateForces>,
    /// Slab length of each contact, km. Empty until slab length is tracked (design task 5).
    pub slabs_km: Vec<f64>,
}

/// One plate's force budget in a step: the size of each torque term and the mean in-plane
/// stress. Nothing writes it yet (the force balance, design task 5); the check reads it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlateForces {
    pub id: u32,
    /// Torque magnitudes by term, in the balance's own unit (only ratios mean anything).
    pub slab_pull: f64,
    pub slab_suction: f64,
    pub edge_energy: f64,
    pub basal_drag: f64,
    pub slab_resistance: f64,
    pub boundary_resistance: f64,
    pub collision_resistance: f64,
    /// Mean in-plane stress from the edge forces, MPa: positive in tension, negative in
    /// compression.
    pub stress_mpa: f64,
}

/// A collision belt's later history and shape. Nothing writes it yet (design tasks 2-3); the
/// check reads it. `None` is "not measured".
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BeltRec {
    /// The collision's `start_myr` and its two plates, to find its `CollisionRec`.
    pub start_myr: f64,
    pub under: u32,
    pub over: u32,
    /// Convergence across the front at the collision's onset, and 20 Myr later (`None` while the
    /// collision has not lasted that long), mm/yr.
    pub convergence_onset_mm_yr: Option<f64>,
    pub convergence_20myr_mm_yr: Option<f64>,
    /// Width of the wedge on the incoming side, km.
    pub wedge_width_km: Option<f64>,
    /// Width, km, and depth of fill, m, of the foreland basin in front of the wedge.
    pub foredeep_width_km: Option<f64>,
    pub foredeep_depth_m: Option<f64>,
}

/// A plate in a step. Serialised as the array `[id, visible, continental, speed_mm_yr, front_km,
/// boundary_km, rot_change, flags]` (flags: 1 pole inside, 2 speed guard bound, 4 `rot_change`
/// known).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "[f64; 8]", into = "[f64; 8]")]
pub struct PlateStep {
    pub id: u32,
    /// Visible samples (its area, in samples), and of them the continental ones.
    pub visible: u32,
    pub continental: u32,
    /// Surface speed at the plate's centroid, mm/yr, after the step's motion update (the speed it
    /// moves at in the next step).
    pub speed_mm_yr: f64,
    /// Length of the subduction fronts this plate goes down at (the slab attached to it), km.
    pub front_km: f64,
    /// Length of its boundary, km: one sample spacing per visible sample that has a neighbour of
    /// another plate or one hidden under another plate (so a slab's trench side counts). An
    /// estimate, in the same unit as `front_km`.
    pub boundary_km: f64,
    /// `|Ω − Ω_prev| / |Ω_prev|` since the previous step (0 when `rot_known` is false).
    pub rot_change: f64,
    /// Whether the plate existed at the previous step, so `rot_change` is a measurement.
    pub rot_known: bool,
    /// Whether the plate's rotation pole lies on the plate.
    pub pole_inside: bool,
    /// Whether the speed guard holds the plate's rotation at its cap.
    pub guard: bool,
}

impl PlateStep {
    /// Share of its boundary attached to a subducting slab, 0 to 1.
    pub fn slab_share(&self) -> f64 {
        if self.boundary_km > 0.0 {
            (self.front_km / self.boundary_km).min(1.0)
        } else {
            0.0
        }
    }

    /// Share of its crust that is continental.
    pub fn continental_share(&self) -> f64 {
        f64::from(self.continental) / f64::from(self.visible.max(1))
    }
}

impl From<[f64; 8]> for PlateStep {
    fn from(a: [f64; 8]) -> Self {
        let flags = a[7] as u8;
        PlateStep {
            id: a[0] as u32,
            visible: a[1] as u32,
            continental: a[2] as u32,
            speed_mm_yr: a[3],
            front_km: a[4],
            boundary_km: a[5],
            rot_change: a[6],
            rot_known: flags & ROT_KNOWN != 0,
            pole_inside: flags & POLE_INSIDE != 0,
            guard: flags & GUARD_BOUND != 0,
        }
    }
}

impl From<PlateStep> for [f64; 8] {
    fn from(p: PlateStep) -> Self {
        let flag = |on: bool, bit: u8| if on { bit } else { 0 };
        let flags = flag(p.pole_inside, POLE_INSIDE)
            | flag(p.guard, GUARD_BOUND)
            | flag(p.rot_known, ROT_KNOWN);
        [
            f64::from(p.id),
            f64::from(p.visible),
            f64::from(p.continental),
            p.speed_mm_yr,
            p.front_km,
            p.boundary_km,
            p.rot_change,
            f64::from(flags),
        ]
    }
}

/// One plate's life.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PlateLife {
    pub id: u32,
    /// The plate it split from (a rift or a subduction initiation).
    pub parent: Option<u32>,
    /// `initial` (there at the start), `rift` or `subduction_initiation`.
    pub origin: String,
    pub born_myr: f64,
    pub died_myr: Option<f64>,
    /// How it died: `sutured` (welded whole to another plate by a continental collision),
    /// `absorbed` (too small, merged into a neighbour) or `consumed` (subducted away: no crust
    /// left).
    pub fate: Option<String>,
    /// The plate that took its crust (`sutured` and `absorbed`).
    pub into: Option<u32>,
    /// Its share of the visible planet in the last step before it died.
    pub share_at_death: Option<f64>,
}

/// The extent of a set of samples, from the second moments of their positions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Footprint {
    pub samples: u32,
    /// Length and width, km: `sqrt(12 λ) R` for the two largest eigenvalues λ of the set's
    /// covariance (exact for a uniform strip). 0 when the set has fewer than 6 samples.
    pub length_km: f64,
    pub width_km: f64,
}

impl Footprint {
    /// Length ÷ width (0 when the set has no extent).
    pub fn elongation(&self) -> f64 {
        if self.width_km > 0.0 {
            self.length_km / self.width_km
        } else {
            0.0
        }
    }
}

/// Count, sum and second moments of a set of unit vectors.
#[derive(Clone, Copy, Debug, Default)]
pub struct Moments {
    n: u32,
    s: DVec3,
    m: [f64; 6],
}

impl Moments {
    pub fn add(&mut self, p: DVec3) {
        self.n += 1;
        self.s += p;
        let q = [
            p.x * p.x,
            p.y * p.y,
            p.z * p.z,
            p.x * p.y,
            p.x * p.z,
            p.y * p.z,
        ];
        for (a, b) in self.m.iter_mut().zip(q) {
            *a += b;
        }
    }

    /// The set's extent on a sphere of `radius_km`.
    pub fn footprint(&self, radius_km: f64) -> Footprint {
        let mut f = Footprint {
            samples: self.n,
            ..Footprint::default()
        };
        if self.n < 6 {
            return f;
        }
        let n = f64::from(self.n);
        let mu = self.s / n;
        let c = [
            self.m[0] / n - mu.x * mu.x,
            self.m[1] / n - mu.y * mu.y,
            self.m[2] / n - mu.z * mu.z,
            self.m[3] / n - mu.x * mu.y,
            self.m[4] / n - mu.x * mu.z,
            self.m[5] / n - mu.y * mu.z,
        ];
        let [l1, l2, _] = eigenvalues(c);
        if l2 > 0.0 {
            f.length_km = (12.0 * l1).sqrt() * radius_km;
            f.width_km = (12.0 * l2).sqrt() * radius_km;
        }
        f
    }
}

/// Eigenvalues, largest first, of the symmetric matrix `[xx, yy, zz, xy, xz, yz]` (the closed
/// form for 3 × 3).
pub fn eigenvalues(m: [f64; 6]) -> [f64; 3] {
    let [a, b, c, d, e, f] = m;
    let p1 = d * d + e * e + f * f;
    let q = (a + b + c) / 3.0;
    let p2 = (a - q).powi(2) + (b - q).powi(2) + (c - q).powi(2) + 2.0 * p1;
    if p2 < 1e-300 {
        return [q, q, q];
    }
    let p = (p2 / 6.0).sqrt();
    let (ba, bb, bc, bd, be, bf) = ((a - q) / p, (b - q) / p, (c - q) / p, d / p, e / p, f / p);
    let det = ba * (bb * bc - bf * bf) - bd * (bd * bc - bf * be) + be * (bd * bf - bb * be);
    let phi = (det / 2.0).clamp(-1.0, 1.0).acos() / 3.0;
    let l1 = q + 2.0 * p * phi.cos();
    let l3 = q + 2.0 * p * (phi + 2.0 * std::f64::consts::FRAC_PI_3).cos();
    [l1, 3.0 * q - l1 - l3, l3]
}

/// One terrane transfer or collision (today a one-step event, see the module docs).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CollisionRec {
    /// Collision: the terrane is at least `collision_min_frac` of an average plate (a `collision`
    /// event is written); otherwise a terrane transfer.
    pub collision: bool,
    /// The plate welded whole to the receiving one.
    pub sutured: bool,
    /// First and last step of the collision, Myr.
    pub start_myr: f64,
    pub end_myr: f64,
    /// When the contact began, Myr.
    pub contact_onset_myr: f64,
    /// The subducting (terrane) and the receiving plate.
    pub under: u32,
    pub over: u32,
    /// Visible-and-hidden samples of each plate at the time.
    pub under_samples: u32,
    pub over_samples: u32,
    /// Samples in the terrane, and its share of an average plate's area.
    pub terrane_samples: u32,
    pub frac: f64,
    /// Convergence at the contact, mm/yr.
    pub convergence_mm_yr: f64,
    /// What the contact's counter read (relative speed × time, km): the quantity that fires the
    /// collision today, which is not shortening.
    pub counted_km: f64,
    /// Continental samples of the subducting plate that went under since the contact began (or
    /// since its last collision), each counted once.
    pub cont_in: u32,
    /// Length of the overlap's continental crust, km, and the real shortening: `cont_in` samples'
    /// area over that length.
    pub front_km: f64,
    pub shortening_km: f64,
    /// The largest uplift, m, and the radius of the surge, km.
    pub peak_uplift_m: f64,
    pub radius_km: f64,
    /// Ground raised by more than 10 m, by more than 100 m, and from below sea level to above it.
    pub lifted10: Footprint,
    pub lifted100: Footprint,
    pub emerged: Footprint,
    /// Birth of the terrane's plate and its parent.
    pub under_born_myr: f64,
    pub under_parent: Option<u32>,
}

/// The sea-floor age distribution at one moment (visible oceanic samples).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AgeRec {
    pub t: f64,
    pub samples: u32,
    pub p50_myr: f64,
    pub p90_myr: f64,
    pub max_myr: f64,
    /// Samples per [`AGE_BIN_MYR`] bin, [`AGE_BINS`] of them (the last holds all older crust).
    pub hist: Vec<u32>,
}

impl Dynamics {
    /// A record for a run that is about to take its first step. The plates alive now are its
    /// initial plates.
    pub fn start(st: &PlateState, world: &WorldState, env: &Env<'_>) -> Dynamics {
        Dynamics {
            version: VERSION,
            t0_myr: st.t_myr,
            dt_myr: env.dt_myr,
            samples: st.samples.len() as u32,
            plates: st
                .alive()
                .map(|p| PlateLife {
                    id: p.id,
                    origin: "initial".to_owned(),
                    born_myr: p.born_myr,
                    ..PlateLife::default()
                })
                .collect(),
            events_seen: world.events.len(),
            ..Dynamics::default()
        }
    }
}

fn int(e: &deeptime_format::Event, key: &str) -> Option<u32> {
    match e.params.get(key) {
        Some(ParamValue::Int(v)) => u32::try_from(*v).ok(),
        _ => None,
    }
}

fn text<'a>(e: &'a deeptime_format::Event, key: &str) -> Option<&'a str> {
    match e.params.get(key) {
        Some(ParamValue::Str(s)) => Some(s),
        _ => None,
    }
}

/// The sea-floor age bin of an age in Myr.
pub fn age_bin(age_myr: f64) -> usize {
    ((age_myr / AGE_BIN_MYR).floor().max(0.0) as usize).min(AGE_BINS - 1)
}

/// For each of some points, the plate of the nearest sample offered (a brute-force nearest
/// neighbour: a few dozen points over the samples costs less than one tree build).
#[derive(Clone, Debug)]
pub struct NearestPlate {
    points: Vec<DVec3>,
    best: Vec<(f64, u32)>,
}

impl NearestPlate {
    pub fn new(points: Vec<DVec3>) -> NearestPlate {
        let best = vec![(f64::NEG_INFINITY, u32::MAX); points.len()];
        NearestPlate { points, best }
    }

    /// Offers a sample at unit vector `p` of `plate`.
    pub fn offer(&mut self, p: DVec3, plate: u32) {
        for (q, b) in self.points.iter().zip(self.best.iter_mut()) {
            let d = p.dot(*q);
            if d > b.0 {
                *b = (d, plate);
            }
        }
    }

    /// The plate nearest to point `k` (`u32::MAX` when nothing was offered).
    pub fn plate(&self, k: usize) -> u32 {
        self.best[k].1
    }
}

/// The relative change of a rotation vector, `|now − prev| / |prev|` (`None` when `prev` is 0).
pub fn rotation_change(prev: DVec3, now: DVec3) -> Option<f64> {
    let n = prev.length();
    (n > 0.0).then(|| (now - prev).length() / n)
}

/// Sizes of the connected sets of members, largest first. `neighbours(i, buf)` pushes the samples
/// joined to member `i` into `buf`; only members are walked.
pub fn piece_sizes(
    n: usize,
    member: impl Fn(usize) -> bool,
    neighbours: impl Fn(usize, &mut Vec<u32>),
) -> Vec<u32> {
    let mut seen = vec![false; n];
    let (mut stack, mut buf, mut sizes) = (Vec::new(), Vec::new(), Vec::new());
    for s in 0..n {
        if seen[s] || !member(s) {
            continue;
        }
        seen[s] = true;
        stack.push(s as u32);
        let mut size = 0u32;
        while let Some(v) = stack.pop() {
            size += 1;
            buf.clear();
            neighbours(v as usize, &mut buf);
            for &u in &buf {
                if !seen[u as usize] && member(u as usize) {
                    seen[u as usize] = true;
                    stack.push(u);
                }
            }
        }
        sizes.push(size);
    }
    sizes.sort_unstable_by(|a, b| b.cmp(a));
    sizes
}

/// The sea-floor ages of the visible oceanic samples at `t`, sorted.
pub fn ocean_ages(st: &PlateState, t: f64) -> Vec<f64> {
    let mut ages: Vec<f64> = st
        .samples
        .iter()
        .filter(|s| !s.is_hidden() && s.kind == OCEANIC)
        .map(|s| s.age_myr(t))
        .collect();
    ages.sort_by(f64::total_cmp);
    ages
}

/// The `q` quantile of sorted ages (nearest rank; 0 of nothing).
fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        0.0
    } else {
        sorted[(((sorted.len() - 1) as f64 * q).round() as usize).min(sorted.len() - 1)]
    }
}

/// Appends this step's record. `front_km` is each plate's subduction front length this step
/// (`subduction::Pull::front_km`), `pos` the samples' current positions, `last` whether the run's
/// final step has been taken. Reads the state and the world's new events, changes only the record.
pub fn record_step(
    st: &mut PlateState,
    world: &WorldState,
    geo: &Geo,
    pos: &[DVec3],
    front_km: &[f64],
    env: &Env<'_>,
    last: bool,
) {
    let Some(mut d) = st.dynamics.take() else {
        return;
    };
    let np = st.plates.len();
    let (mut vis, mut cont, mut bnd) = (vec![0u32; np], vec![0u32; np], vec![0u32; np]);
    let (mut visible, mut land, mut continental) = (0u32, 0u32, 0u32);
    // Each plate's rotation pole, both ends of its axis: which plate holds the nearest sample.
    let mut pole_of: Vec<Option<usize>> = vec![None; np];
    let mut poles = Vec::new();
    for p in st.alive() {
        let w = DVec3::from_array(p.omega);
        if w.length() > 0.0 {
            pole_of[p.id as usize] = Some(poles.len() / 2);
            poles.push(w.normalize());
            poles.push(-w.normalize());
        }
    }
    let mut nearest = NearestPlate::new(poles);
    for (i, s) in st.samples.iter().enumerate() {
        if s.is_hidden() {
            continue;
        }
        let p = st.plate_of(i);
        nearest.offer(pos[i], p);
        vis[p as usize] += 1;
        visible += 1;
        if s.kind == CONTINENTAL {
            cont[p as usize] += 1;
            continental += 1;
        }
        if s.elevation_m > 0.0 {
            land += 1;
        }
        // On the plate's boundary: a neighbour of another plate, or one hidden under another
        // plate (the plate's own subducted crust: the trench side of a slab).
        let edge = geo
            .edges(i as u32)
            .any(|(u, _)| st.samples[u as usize].is_hidden() || st.plate_of(u as usize) != p);
        if edge {
            bnd[p as usize] += 1;
        }
    }
    let spacing_km = env.norm.km(env.norm.spacing_rad());
    let round = |x: f64, scale: f64| (x * scale).round() / scale;
    let cap = env.norm.omega_of_speed(env.k.max_plate_speed_mm_yr);
    let prev: BTreeMap<u32, DVec3> = d
        .prev_omega
        .iter()
        .map(|&(id, w)| (id, DVec3::from_array(w)))
        .collect();
    let plates: Vec<PlateStep> = st
        .alive()
        .map(|p| {
            let k = p.id as usize;
            let w = DVec3::from_array(p.omega);
            let change = prev.get(&p.id).and_then(|&w0| rotation_change(w0, w));
            PlateStep {
                id: p.id,
                visible: vis[k],
                continental: cont[k],
                speed_mm_yr: round(env.norm.speed_mm_yr(w, DVec3::from_array(p.centroid)), 10.0),
                front_km: round(front_km.get(k).copied().unwrap_or(0.0), 1.0),
                boundary_km: round(f64::from(bnd[k]) * spacing_km, 1.0),
                rot_change: round(change.unwrap_or(0.0), 1.0e4),
                rot_known: change.is_some(),
                pole_inside: pole_of[k].is_some_and(|j| {
                    nearest.plate(2 * j) == p.id || nearest.plate(2 * j + 1) == p.id
                }),
                guard: cap > 0.0 && w.length() >= cap * (1.0 - 1.0e-9),
            }
        })
        .collect();
    d.prev_omega = st.alive().map(|p| (p.id, p.omega)).collect();

    // Connected pieces of continental crust: continental samples joined along lattice edges
    // whose ends are still close (a rift since the last resampling pulls them apart).
    let link = (PIECE_LINK_SPACINGS * env.norm.spacing_rad()).cos();
    let mut pieces = piece_sizes(
        st.samples.len(),
        |i| {
            let s = &st.samples[i];
            !s.is_hidden() && s.kind == CONTINENTAL
        },
        |i, out| {
            out.extend(
                geo.edges(i as u32)
                    .filter(|&(u, _)| pos[i].dot(pos[u as usize]) >= link)
                    .map(|(u, _)| u),
            );
        },
    );
    pieces.truncate(PIECES_KEPT);

    // The sea floor's age, every step.
    let ages = ocean_ages(st, env.t_myr);
    let older = |limit: f64| ages.len() - ages.partition_point(|&a| a <= limit);

    // Births and deaths: from the events this step wrote.
    let last_shares: BTreeMap<u32, f64> = d
        .steps
        .last()
        .map(|s| {
            let total: u32 = s.plates.iter().map(|p| p.visible).sum();
            s.plates
                .iter()
                .map(|p| (p.id, f64::from(p.visible) / f64::from(total.max(1))))
                .collect()
        })
        .unwrap_or_default();
    for e in world.events.get(d.events_seen..).unwrap_or_default() {
        if e.kind == PLATE_BIRTH {
            if let Some(id) = int(e, "plate") {
                d.plates.push(PlateLife {
                    id,
                    parent: int(e, "parent"),
                    origin: text(e, "origin").unwrap_or("rift").to_owned(),
                    born_myr: e.t_start / 1.0e6,
                    ..PlateLife::default()
                });
            }
        } else if e.kind == PLATE_DEATH
            && let Some(id) = int(e, "plate")
            && let Some(life) = d.plates.iter_mut().find(|l| l.id == id)
        {
            life.died_myr = Some(e.t_start / 1.0e6);
            life.fate = text(e, "fate").map(str::to_owned);
            life.into = int(e, "absorbed_by");
            life.share_at_death = last_shares.get(&id).copied();
        }
    }
    d.events_seen = world.events.len();

    let t = env.t_myr;
    let (mut big, mut small) = (0u16, 0u16);
    for c in d
        .collisions
        .iter()
        .rev()
        .take_while(|c| c.end_myr >= t - 1e-9)
    {
        if c.collision {
            big += 1;
        } else {
            small += 1;
        }
    }
    d.steps.push(StepRec {
        t,
        visible,
        land,
        continental,
        collisions: big,
        transfers: small,
        plates,
        ocean: ages.len() as u32,
        age_p50: round(quantile(&ages, 0.5), 10.0),
        age_p90: round(quantile(&ages, 0.9), 10.0),
        old180: older(OLD_AGES_MYR[0]) as u32,
        old250: older(OLD_AGES_MYR[1]) as u32,
        pieces,
        ..StepRec::default()
    });

    let every = (AGE_EVERY_MYR / env.dt_myr).round().max(1.0) as u64;
    if last || env.step.is_multiple_of(every) {
        d.ages.push(age_record_of(&ages, t));
    }
    st.dynamics = Some(d);
}

/// The sea-floor age distribution of the visible oceanic samples at `t`.
pub fn age_record(st: &PlateState, t: f64) -> AgeRec {
    age_record_of(&ocean_ages(st, t), t)
}

/// The distribution of `ages` (sorted).
pub fn age_record_of(ages: &[f64], t: f64) -> AgeRec {
    let mut hist = vec![0u32; AGE_BINS];
    for &a in ages {
        hist[age_bin(a)] += 1;
    }
    AgeRec {
        t,
        samples: ages.len() as u32,
        p50_myr: quantile(ages, 0.5),
        p90_myr: quantile(ages, 0.9),
        max_myr: ages.last().copied().unwrap_or(0.0),
        hist,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on_sphere(lon_deg: f64, lat_deg: f64) -> DVec3 {
        let (lo, la) = (lon_deg.to_radians(), lat_deg.to_radians());
        DVec3::new(la.cos() * lo.cos(), la.cos() * lo.sin(), la.sin())
    }

    #[test]
    fn eigenvalues_of_known_matrices() {
        let e = eigenvalues([3.0, 1.0, 2.0, 0.0, 0.0, 0.0]);
        assert!(
            (e[0] - 3.0).abs() < 1e-12 && (e[1] - 2.0).abs() < 1e-12 && (e[2] - 1.0).abs() < 1e-12
        );
        // [[2, 1, 0], [1, 2, 0], [0, 0, 5]]: 5, 3, 1.
        let e = eigenvalues([2.0, 2.0, 5.0, 1.0, 0.0, 0.0]);
        assert!(
            (e[0] - 5.0).abs() < 1e-9 && (e[1] - 3.0).abs() < 1e-9 && (e[2] - 1.0).abs() < 1e-9
        );
        assert_eq!(eigenvalues([0.0; 6]), [0.0; 3]);
    }

    #[test]
    fn a_strip_is_long_and_a_disc_is_round() {
        let r = 6370.0;
        // A strip 20° long and 2° wide on the equator (about 2,200 by 220 km).
        let mut strip = Moments::default();
        for i in 0..=40 {
            for j in 0..=4 {
                strip.add(on_sphere(
                    -10.0 + 0.5 * f64::from(i),
                    -1.0 + 0.5 * f64::from(j),
                ));
            }
        }
        let f = strip.footprint(r);
        assert_eq!(f.samples, 41 * 5);
        assert!(f.length_km > f.width_km);
        assert!((f.length_km - 20.0_f64.to_radians() * r).abs() < 0.15 * 20.0_f64.to_radians() * r);
        assert!(f.elongation() > 6.0, "{}", f.elongation());
        // A disc 10° across.
        let mut disc = Moments::default();
        for i in -10..=10 {
            for j in -10..=10 {
                if i * i + j * j <= 100 {
                    disc.add(on_sphere(f64::from(i) * 0.5, f64::from(j) * 0.5));
                }
            }
        }
        let f = disc.footprint(r);
        assert!((0.9..1.1).contains(&f.elongation()), "{}", f.elongation());
        // The extent scales with the radius, and a handful of samples has none.
        let g = disc.footprint(2.0 * r);
        assert!((g.length_km / f.length_km - 2.0).abs() < 1e-9);
        let mut few = Moments::default();
        for i in 0..5 {
            few.add(on_sphere(f64::from(i), 0.0));
        }
        assert_eq!(few.footprint(r).length_km, 0.0);
        assert_eq!(few.footprint(r).elongation(), 0.0);
    }

    #[test]
    fn the_record_reads_back_from_json_and_a_plate_is_a_short_array() {
        let mut d = Dynamics {
            version: VERSION,
            dt_myr: 2.0,
            samples: 100,
            ..Dynamics::default()
        };
        d.steps.push(StepRec {
            t: 2.0,
            visible: 100,
            plates: vec![PlateStep {
                id: 3,
                visible: 60,
                continental: 20,
                speed_mm_yr: 41.5,
                front_km: 300.0,
                boundary_km: 900.0,
                rot_change: 0.0625,
                rot_known: true,
                pole_inside: true,
                guard: false,
            }],
            pieces: vec![40, 7],
            ..StepRec::default()
        });
        d.plates.push(PlateLife {
            id: 3,
            parent: Some(1),
            origin: "rift".into(),
            born_myr: 1.0,
            ..PlateLife::default()
        });
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(
            v["steps"][0]["plates"][0],
            serde_json::json!([3.0, 60.0, 20.0, 41.5, 300.0, 900.0, 0.0625, 5.0])
        );
        // The physics-to-come fields are there, empty (the checkpoint's postcard has no optional
        // fields).
        assert_eq!(v["steps"][0]["forces"], serde_json::json!([]));
        assert_eq!(v["belts"], serde_json::json!([]));
        let back: Dynamics = serde_json::from_value(v).unwrap();
        assert_eq!(back, d);
        let p = &d.steps[0].plates[0];
        assert!((p.slab_share() - 1.0 / 3.0).abs() < 1e-12);
        assert!((p.continental_share() - 1.0 / 3.0).abs() < 1e-12);
        // No boundary, no slab; a front longer than the boundary estimate saturates at 1.
        let lone = PlateStep {
            boundary_km: 0.0,
            ..p.clone()
        };
        assert_eq!(lone.slab_share(), 0.0);
        let over = PlateStep {
            front_km: 5000.0,
            ..p.clone()
        };
        assert_eq!(over.slab_share(), 1.0);
    }

    #[test]
    fn age_bins_cover_every_age() {
        assert_eq!(age_bin(0.0), 0);
        assert_eq!(age_bin(9.99), 0);
        assert_eq!(age_bin(10.0), 1);
        assert_eq!(age_bin(399.0), 39);
        assert_eq!(age_bin(400.0), AGE_BINS - 1);
        assert_eq!(age_bin(5000.0), AGE_BINS - 1);
        assert_eq!(age_bin(-3.0), 0);
    }

    #[test]
    fn a_rotation_changes_by_its_relative_difference() {
        let w = DVec3::new(0.0, 0.0, 2.0);
        assert_eq!(rotation_change(w, w), Some(0.0));
        // Twice as fast about the same axis: a change of 100 %.
        assert!((rotation_change(w, 2.0 * w).unwrap() - 1.0).abs() < 1e-12);
        // The same rate about a perpendicular axis: |Δ| = 2√2 over 2.
        let turned = DVec3::new(2.0, 0.0, 0.0);
        assert!((rotation_change(w, turned).unwrap() - 2.0_f64.sqrt()).abs() < 1e-12);
        // A plate at rest has no relative change.
        assert_eq!(rotation_change(DVec3::ZERO, w), None);
    }

    #[test]
    fn a_pole_belongs_to_the_plate_of_the_nearest_sample() {
        let poles = vec![on_sphere(0.0, 90.0), on_sphere(0.0, -90.0)];
        let mut n = NearestPlate::new(poles);
        assert_eq!(n.plate(0), u32::MAX);
        // Plate 1 holds the northern cap, plate 2 the rest.
        for (lon, lat, plate) in [(0.0, 80.0, 1), (90.0, 60.0, 2), (10.0, -30.0, 2)] {
            n.offer(on_sphere(lon, lat), plate);
        }
        assert_eq!(n.plate(0), 1);
        assert_eq!(n.plate(1), 2);
    }

    /// A ring of `n` samples (each joined to its two neighbours), `members` of them in the set.
    fn ring_pieces(n: usize, members: &[usize]) -> Vec<u32> {
        piece_sizes(
            n,
            |i| members.contains(&i),
            |i, out| {
                out.push(((i + 1) % n) as u32);
                out.push(((i + n - 1) % n) as u32);
            },
        )
    }

    #[test]
    fn connected_pieces_are_counted_and_sorted_largest_first() {
        // Samples 0-2 | 4 | 6-9 on a ring of 12: three pieces; 11 and 0 are neighbours, so 11 joins
        // the first.
        assert_eq!(
            ring_pieces(12, &[0, 1, 2, 4, 6, 7, 8, 9, 11]),
            vec![4, 4, 1]
        );
        assert_eq!(ring_pieces(12, &[3, 5]), vec![1, 1]);
        assert_eq!(ring_pieces(12, &(0..12).collect::<Vec<_>>()), vec![12]);
        assert!(ring_pieces(12, &[]).is_empty());
    }

    #[test]
    fn quantiles_of_sorted_ages_and_the_distribution_agree() {
        let ages: Vec<f64> = (0..=100).map(f64::from).collect();
        assert_eq!(quantile(&ages, 0.5), 50.0);
        assert_eq!(quantile(&ages, 0.9), 90.0);
        assert_eq!(quantile(&[], 0.5), 0.0);
        let rec = age_record_of(&ages, 7.0);
        assert_eq!((rec.p50_myr, rec.p90_myr, rec.max_myr), (50.0, 90.0, 100.0));
        assert_eq!(rec.samples, 101);
        assert_eq!(rec.hist.iter().sum::<u32>(), 101);
        // Samples older than a limit, as the step record counts them.
        let older = |limit: f64| ages.len() - ages.partition_point(|&a| a <= limit);
        assert_eq!(older(90.0), 10);
        assert_eq!(older(100.0), 0);
    }
}

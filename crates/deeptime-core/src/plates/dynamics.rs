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
//! | `partings` | one [`PartingRec`] per collision state that ended with no transfer or weld |
//! | `ages` | the sea-floor age distribution every [`AGE_EVERY_MYR`] ([`AgeRec`]) |
//! | `steps[].mix_km`, `arc_oceanic`, `arc_continental` | version 3: the boundary mix and the arc cells' heights (below) |
//! | `prev_omega` | each plate's rotation vector after the last step (for the next step's change) |
//! | `belts` | one [`BeltRec`] per collision belt: empty until the belt physics exists |
//! | `steps[].forces`, `slabs_km`, `breakoffs`; `slabs`, `prev_balance` | version 4: the force balance (below) |
//!
//! Left empty by today's model, to be filled by the task that adds the physics (the check reads
//! it and reports "no data" while it is empty): `Dynamics::belts` (convergence 20 Myr after
//! onset, wedge and foredeep, [`BeltRec`]).
//!
//! # The force balance (version 4)
//! `forces.rs` sums each plate's driving and resisting forces every step, solves the torque
//! balance for every plate's rotation and records it here without applying it (design
//! `docs/design/plate-force-balance.md`, task 5): `StepRec::forces` ([`PlateForces`]: the torque
//! of each term, the mean in-plane stress, the answer's speed and its change since the last
//! step), `StepRec::slabs_km` (the slab length of each subducting contact) and
//! `StepRec::breakoffs`; `Dynamics::slabs` ([`SlabTrack`]) and `prev_balance` carry the state
//! from step to step. It is taken right after the step's fronts are found, as the boundary mix.
//!
//! # Boundary mix and arc heights (version 3)
//! Two more measures for `plate-check`, measurement only (nothing a plate rule reads):
//! - `StepRec::mix_km`: the length of plate boundary by kind, km (one sample spacing per boundary
//!   sample, see [`classify_boundary`]): [`mix`]`::OCEAN_OCEAN`, `OCEAN_CONTINENT` (an oceanic
//!   slab going under an overriding plate's oceanic or continental crust), `COLLISION` (continental
//!   crust at the subducting side), `RIDGE`, `RIFT` and `OTHER` (transform, or converging with no
//!   contact yet). The subducting length is the first two. It is taken right after the step's
//!   fronts are found, with the same relative-motion test the fronts use
//!   (`subduction::fronts_and_uplift`), because the sample flags are cleared by a resampling.
//! - `StepRec::arc_oceanic` and `arc_continental` ([`ArcStat`]): the elevation, m above the plate
//!   model's sea datum, of the samples that received arc magma this step (flag `ARC`, the code's
//!   own arc cells: within `arc_width_km` of the arc distance from a front). An *oceanic arc* is
//!   one whose arc crust was oceanic when the magma came ([`is_island_arc`]); an island arc that
//!   emerged and matured into continental crust (`finish_samples`) is still an oceanic arc here.
//!   Taken after the step's surface and uplift are final.
//!
//! Lengths are Earth-normalized km (the plate model's sphere, `model_radius_km`), speeds mm/yr,
//! times Myr relative to world creation. "Visible" samples are those not hidden under another
//! plate. The active margin share is not repeated here: `notes.plates.margins` has it.
//!
//! # What each collision must keep meaning
//! Tasks 2–4 of the design turn a collision from a one-step event into a state of a contact
//! (`state::Collision`, `shortening.rs`; the state exists since task 2). The record keeps its
//! fields' meanings across that change: `start_myr` and `end_myr` are the **first and last step**
//! of the collision state (the step continental crust first converged at the front, and the step
//! of the transfer or weld; the check's length is `end − start + dt`, so a one-step collision is
//! one step long, counted from the collision's first step and not from the contact's onset, which
//! is `contact_onset_myr`), `peak_uplift_m` the largest uplift it produced anywhere,
//! `shortening_km` the continental crust that went under over the front's length,
//! `lifted10`/`lifted100` the ground it raised by more than 10 and 100 m over its life, and
//! `emerged` the ground that rose from below the sea to above it. Until task 3 the uplift is still
//! the one-step surge at the end, so the footprints are that step's. A collision state that ends
//! without a transfer or weld (the plates part, the contact reverses or closes, the run ends) is a
//! [`PartingRec`].
//!
//! # Per plate and step (task 1b)
//! `rot_change` is the size of the change of the plate's rotation vector since the previous step
//! over the size of the previous one (`|Ω − Ω_prev| / |Ω_prev|`: direction and rate together); it
//! is known only when the plate existed at the previous step. `pole_inside` is whether the
//! plate's rotation pole (either end of its axis) lies on the plate: the visible sample nearest to
//! it belongs to the plate. `spinning` is whether the pole is within [`SPIN_ANGLE_DEG`] of the
//! plate's centroid (either end of the axis): the plate then turns in place more than it travels
//! (the design's "turning about a pole near themselves"). `guard` is whether the speed guard (`max_plate_speed_mm_yr`) holds the
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

use std::collections::{BTreeMap, BTreeSet};

use deeptime_format::ParamValue;
use glam::DVec3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::Env;
use super::events::{PLATE_BIRTH, PLATE_DEATH};
use super::geometry::Geo;
use super::kinematics::{quat, tangent};
use super::state::{ARC, CONTINENTAL, OCEANIC, PlateState, Sample};
use crate::world::WorldState;

/// Set this environment variable and the stage records nothing (to time what recording costs).
pub const OFF_ENV: &str = "DEEPTIME_NO_PLATE_DYNAMICS";
/// The record's format version (`notes.plates.dynamics.version`).
pub const VERSION: u32 = 4;
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
const SPINNING: u8 = 8;
/// A plate whose rotation pole is within this many degrees of its own centroid (either end of the
/// axis) mostly spins in place.
pub const SPIN_ANGLE_DEG: f64 = 30.0;

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
    /// Collision states that ended with no transfer or weld.
    #[serde(default)]
    pub partings: Vec<PartingRec>,
    pub ages: Vec<AgeRec>,
    /// How many of the world's events have been read (births and deaths come from them).
    pub events_seen: usize,
    /// Each plate's rotation vector after the last step, for the next step's `rot_change`.
    pub prev_omega: Vec<(u32, [f64; 3])>,
    /// Collision belts: convergence history, wedge and foredeep. Empty until the belt physics
    /// (design tasks 2-3) writes it.
    pub belts: Vec<BeltRec>,
    /// Version 4 (`forces.rs`): the slab of every contact, and each plate's balance rotation
    /// after the last step (rad/yr, for the next step's `PlateForces::rot_change`).
    #[serde(default)]
    pub slabs: Vec<SlabTrack>,
    #[serde(default)]
    pub prev_balance: Vec<(u32, [f64; 3])>,
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
    /// Per-plate force budget, stress and balance answer (`forces.rs`; version 4), for the plates
    /// in the balance (alive, with visible samples, when the forces were summed).
    pub forces: Vec<PlateForces>,
    /// Slab length of each contact with an oceanic front this step, km (version 4).
    pub slabs_km: Vec<f64>,
    /// Slabs that broke off this step (version 4).
    #[serde(default)]
    pub breakoffs: u16,
    /// Length of plate boundary by kind, km, indexed by [`mix`] (all 0 in a record without it).
    #[serde(default)]
    pub mix_km: [f64; MIX_CLASSES],
    /// Elevation of the oceanic and continental arc cells of the step ([`ArcStat`]).
    #[serde(default)]
    pub arc_oceanic: ArcStat,
    #[serde(default)]
    pub arc_continental: ArcStat,
}

/// One plate's force budget in a step (`forces.rs`, version 4): the size of each torque term,
/// the mean in-plane stress, and the rotation at which the torques balance, which is recorded,
/// not applied. Serialised as the array `[id, slab_pull, slab_suction, edge_energy, mantle_flow,
/// basal_drag, slab_resistance, boundary_resistance, collision_resistance, stress_mpa,
/// speed_mm_yr, rms_mm_yr, rot_change, flags]` (flags: 1 `rot_change` known).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "[f64; 14]", into = "[f64; 14]")]
pub struct PlateForces {
    pub id: u32,
    /// Torque magnitudes by term, 10²⁴ N m (`forces::TORQUE_UNIT_N_M`), four significant
    /// digits. The driving terms (pull, suction, edge energy, the mantle flow's drag) do not
    /// depend on speed; the resisting ones are taken at the balance's answer. The edge energy
    /// drives at ridges and resists at high ground.
    pub slab_pull: f64,
    pub slab_suction: f64,
    pub edge_energy: f64,
    pub mantle_flow: f64,
    pub basal_drag: f64,
    pub slab_resistance: f64,
    pub boundary_resistance: f64,
    pub collision_resistance: f64,
    /// Mean in-plane stress from the edge forces and the basal drag at the answer, MPa: positive
    /// in tension, negative in compression.
    pub stress_mpa: f64,
    /// The balance's answer: its speed at the plate's centroid and its RMS over the plate, mm/yr.
    pub speed_mm_yr: f64,
    pub rms_mm_yr: f64,
    /// `|Ω − Ω_prev| / |Ω_prev|` of the answer since the previous step (0 when not known).
    pub rot_change: f64,
    pub rot_known: bool,
}

impl PlateForces {
    /// Share of the driving torque that is slab pull (by size: pull over the sum of the sizes of
    /// pull, suction, edge energy and the mantle flow; `None` with no driving torque).
    pub fn pull_share(&self) -> Option<f64> {
        let driving = self.slab_pull + self.slab_suction + self.edge_energy + self.mantle_flow; // ledger: FRC-023
        (driving > 0.0).then(|| self.slab_pull / driving)
    }
}

impl From<[f64; 14]> for PlateForces {
    fn from(a: [f64; 14]) -> Self {
        PlateForces {
            id: a[0] as u32,
            slab_pull: a[1],
            slab_suction: a[2],
            edge_energy: a[3],
            mantle_flow: a[4],
            basal_drag: a[5],
            slab_resistance: a[6],
            boundary_resistance: a[7],
            collision_resistance: a[8],
            stress_mpa: a[9],
            speed_mm_yr: a[10],
            rms_mm_yr: a[11],
            rot_change: a[12],
            rot_known: a[13] as u8 & 1 != 0,
        }
    }
}

impl From<PlateForces> for [f64; 14] {
    fn from(f: PlateForces) -> Self {
        [
            f64::from(f.id),
            f.slab_pull,
            f.slab_suction,
            f.edge_energy,
            f.mantle_flow,
            f.basal_drag,
            f.slab_resistance,
            f.boundary_resistance,
            f.collision_resistance,
            f.stress_mpa,
            f.speed_mm_yr,
            f.rms_mm_yr,
            f.rot_change,
            f64::from(u8::from(f.rot_known)),
        ]
    }
}

/// The slab hanging from one subducting contact (`forces.rs`, design §3.3): its length, km, and
/// how long continental crust has held at least half of its front, Myr. The contact is known by
/// its plates and the step it began (a reversed contact begins again, with a new slab).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SlabTrack {
    pub under: u32,
    pub over: u32,
    pub onset_step: u64,
    pub km: f64,
    pub blocked_myr: f64,
}

/// A collision belt's later history and shape; the check reads it. `None` is "not measured".
/// Since task 2 `collision::collide` writes one per collision that had a collision state, with
/// the convergence at its onset and 20 Myr later; the wedge and foredeep wait for task 3.
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
/// known, 8 spinning).
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
    /// Whether the pole is within [`SPIN_ANGLE_DEG`] of the plate's centroid: it turns in place.
    pub spinning: bool,
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
            spinning: flags & SPINNING != 0,
        }
    }
}

impl From<PlateStep> for [f64; 8] {
    fn from(p: PlateStep) -> Self {
        let flag = |on: bool, bit: u8| if on { bit } else { 0 };
        let flags = flag(p.pole_inside, POLE_INSIDE)
            | flag(p.guard, GUARD_BOUND)
            | flag(p.rot_known, ROT_KNOWN)
            | flag(p.spinning, SPINNING);
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

/// One terrane transfer or collision: a collision state that ended in a transfer or weld (see the
/// module docs).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CollisionRec {
    /// Collision: the terrane is at least `collision_min_frac` of an average plate (a `collision`
    /// event is written); otherwise a terrane transfer.
    pub collision: bool,
    /// The plate welded whole to the receiving one.
    pub sutured: bool,
    /// First and last step of the collision state, Myr: when continental crust first converged
    /// at the front, and when the terrane was transferred or the plates welded.
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
    /// What the contact's pre-task-2 count read (relative speed × time, km;
    /// `state::Contact::speed_count_km`): it fires the transfer or weld while
    /// `tectonics.collision_on_shortening` is off, and it is not shortening.
    pub counted_km: f64,
    /// The collision state's shortening, km: the convergence across the front integrated over
    /// the collision (`state::Collision::shortening_km`). It fires the transfer or weld when
    /// `tectonics.collision_on_shortening` is on. 0 when the contact fired with no collision state.
    #[serde(default)]
    pub front_shortening_km: f64,
    /// Crust that crossed the front over the collision state, km² (Earth-normalized): each
    /// step's shortening times its front length (`state::Collision::area_km2`).
    #[serde(default)]
    pub area_km2: f64,
    /// Convergence across the front when the collision state began, mm/yr; `None` when the
    /// contact fired with no collision state (possible while the pre-task-2 count fires).
    #[serde(default)]
    pub onset_convergence_mm_yr: Option<f64>,
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

/// Why a collision state ended with no transfer or weld ([`PartingRec::reason`]): no continental
/// crust at the front for more than `shortening::GRACE_STEPS` steps …
pub const PARTED: &str = "parted";
/// … the contact reversed its polarity …
pub const REVERSED: &str = "reversed";
/// … the contact closed (no overlap for `contact_timeout_steps`, or one of its plates died) …
pub const CLOSED: &str = "closed";
/// … its terrane was gone when the transfer fired (another collision of the step took it) …
pub const EMPTIED: &str = "emptied";
/// … or the run ended with the collision under way.
pub const OPEN: &str = "open";

/// A collision state that ended with no transfer or weld (`shortening.rs`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PartingRec {
    /// First step of the collision state and the last step continental crust stood at its
    /// front, Myr.
    pub start_myr: f64,
    pub end_myr: f64,
    /// When the contact began, Myr.
    pub contact_onset_myr: f64,
    /// The subducting and the overriding plate.
    pub under: u32,
    pub over: u32,
    /// Shortening it reached, km, and the crust that crossed the front, km².
    pub shortening_km: f64,
    pub area_km2: f64,
    /// The longest its front was, km, and its fastest convergence across it, mm/yr.
    pub front_km: f64,
    pub peak_convergence_mm_yr: f64,
    /// [`PARTED`], [`REVERSED`], [`CLOSED`], [`EMPTIED`] or [`OPEN`].
    pub reason: String,
}

/// Records the end of `contact`'s collision state `col` with no transfer or weld. Measurement
/// only: changes the record alone (nothing when recording is off).
pub fn record_parting(
    dynamics: &mut Option<Dynamics>,
    contact: &super::state::Contact,
    col: &super::state::Collision,
    reason: &str,
) {
    if let Some(d) = dynamics.as_mut() {
        d.partings.push(PartingRec {
            start_myr: col.onset_myr,
            end_myr: col.last_myr,
            contact_onset_myr: contact.onset_myr,
            under: contact.under,
            over: contact.over,
            shortening_km: col.shortening_km,
            area_km2: col.area_km2,
            front_km: col.peak_front_km,
            peak_convergence_mm_yr: col.peak_convergence_mm_yr,
            reason: reason.to_owned(),
        });
    }
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

/// Kinds of plate boundary in [`StepRec::mix_km`].
pub mod mix {
    /// An oceanic slab under an overriding plate's oceanic crust.
    pub const OCEAN_OCEAN: usize = 0;
    /// An oceanic slab under an overriding plate's continental crust.
    pub const OCEAN_CONTINENT: usize = 1;
    /// Continental crust at the subducting side of a front (a collision or its approach).
    pub const COLLISION: usize = 2;
    /// A spreading boundary between oceanic crust.
    pub const RIDGE: usize = 3;
    /// A spreading boundary with continental crust on a side (a rift).
    pub const RIFT: usize = 4;
    /// Transform, or converging with no contact established yet.
    pub const OTHER: usize = 5;
    pub const NAMES: [&str; 6] = [
        "ocean_ocean",
        "ocean_continent",
        "collision",
        "ridge",
        "rift",
        "other",
    ];
}
/// Number of boundary kinds ([`mix`]).
pub const MIX_CLASSES: usize = 6;

/// Which side of an active subduction contact a sample is on, towards one neighbour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// No active contact between the two plates.
    None,
    /// The sample is on the overriding plate and the neighbour on the subducting one.
    Over,
    /// The sample is on the subducting plate.
    Under,
}

/// One lattice neighbour of a boundary sample that belongs to another plate.
#[derive(Clone, Copy, Debug)]
pub struct Neighbour {
    /// The neighbour's velocity relative to the sample's plate, along the direction from the sample
    /// to the neighbour, mm/yr: positive is moving apart, negative converging.
    pub along: f64,
    /// The neighbour's crust ([`OCEANIC`] or [`CONTINENTAL`]).
    pub kind: u8,
    pub side: Side,
}

/// What kind of boundary a visible sample is on (ledger: PLT-221), from its other-plate neighbours, and how much of
/// the boundary it stands for in half sample spacings (2 or 1): `None` when it is on no boundary
/// or on the subducting side of a front (the overriding side counts that front). The tests are
/// those of `subduction::fronts_and_uplift`: a front is the fastest-converging neighbour of a
/// sample on the overriding side of an active contact; a sample with any neighbour moving apart is
/// divergent. A front counts one sample spacing (one plate's side of the boundary, as
/// `PlateStep::front_km` does); the others count half a spacing from each of their two sides.
pub fn classify_boundary(v_kind: u8, nbrs: &[Neighbour]) -> Option<(usize, u32)> {
    if nbrs.is_empty() {
        return None;
    }
    let mut best = 0.0;
    let mut front: Option<&Neighbour> = None;
    for n in nbrs {
        if n.along < 0.0 && n.side == Side::Over && -n.along > best {
            best = -n.along;
            front = Some(n);
        }
    }
    if let Some(n) = front {
        let class = if n.kind == CONTINENTAL {
            mix::COLLISION
        } else if v_kind == CONTINENTAL {
            mix::OCEAN_CONTINENT
        } else {
            mix::OCEAN_OCEAN
        };
        return Some((class, 2));
    }
    if nbrs.iter().any(|n| n.along > 0.0) {
        let continental =
            v_kind == CONTINENTAL || nbrs.iter().any(|n| n.along > 0.0 && n.kind == CONTINENTAL);
        let class = if continental { mix::RIFT } else { mix::RIDGE };
        return Some((class, 1));
    }
    if nbrs.iter().any(|n| n.along < 0.0 && n.side == Side::Under) {
        return None;
    }
    Some((mix::OTHER, 1))
}

/// The length of plate boundary by kind, km ([`mix`]), at the plates' positions `pos` and the
/// motion the step's fronts were found with (call it after `fronts_and_uplift`, before the
/// motion is updated). All zeros when the record is off.
pub fn boundary_mix(
    st: &PlateState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env<'_>,
) -> [f64; MIX_CLASSES] {
    if st.dynamics.is_none() {
        return [0.0; MIX_CLASSES];
    }
    let samples = &st.samples;
    let plate_of: Vec<u32> = (0..samples.len()).map(|i| st.plate_of(i)).collect();
    let omega: Vec<DVec3> = st
        .plates
        .iter()
        .map(|p| DVec3::from_array(p.omega))
        .collect();
    let rots: Vec<_> = st.groups.iter().map(|g| quat(g.rot)).collect();
    let pts = geo.points();
    let active: BTreeSet<(u32, u32)> = st
        .contacts
        .iter()
        .filter(|c| c.last_step == env.step)
        .map(|c| (c.under, c.over))
        .collect();
    let norm = env.norm;
    let units = (0..samples.len())
        .into_par_iter()
        .fold(
            || [0u64; MIX_CLASSES],
            |mut acc, v| {
                let sv = &samples[v];
                if sv.is_hidden() {
                    return acc;
                }
                let pv = plate_of[v];
                let x = pos[v];
                let rot = rots[sv.group as usize];
                let mut buf = [Neighbour {
                    along: 0.0,
                    kind: OCEANIC,
                    side: Side::None,
                }; 16];
                let mut n = 0;
                for (u, _) in geo.edges(v as u32) {
                    let pu = plate_of[u as usize];
                    if pu == pv || n == buf.len() {
                        continue;
                    }
                    let out = tangent(x, rot * (pts[u as usize] - pts[v])).normalize_or_zero();
                    let rel = norm.velocity_mm_yr(omega[pu as usize] - omega[pv as usize], x);
                    buf[n] = Neighbour {
                        along: rel.dot(out),
                        kind: samples[u as usize].kind,
                        side: if active.contains(&(pu, pv)) {
                            Side::Over
                        } else if active.contains(&(pv, pu)) {
                            Side::Under
                        } else {
                            Side::None
                        },
                    };
                    n += 1;
                }
                if let Some((class, half)) = classify_boundary(sv.kind, &buf[..n]) {
                    acc[class] += u64::from(half);
                }
                acc
            },
        )
        .reduce(
            || [0; MIX_CLASSES],
            |mut a, b| {
                for (x, y) in a.iter_mut().zip(b) {
                    *x += y;
                }
                a
            },
        );
    let spacing_km = norm.km(norm.spacing_rad());
    units.map(|u| (u as f64 * 0.5 * spacing_km).round())
}

/// The elevations of the arc cells of one kind in one step: how many, how many stand above the
/// model's sea datum, and the median, 90th percentile and highest, m. Serialised as the array
/// `[cells, emergent, p50, p90, max]`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "[f64; 5]", into = "[f64; 5]")]
pub struct ArcStat {
    pub cells: u32,
    pub emergent: u32,
    pub p50_m: f64,
    pub p90_m: f64,
    pub max_m: f64,
}

impl From<[f64; 5]> for ArcStat {
    fn from(a: [f64; 5]) -> Self {
        ArcStat {
            cells: a[0] as u32,
            emergent: a[1] as u32,
            p50_m: a[2],
            p90_m: a[3],
            max_m: a[4],
        }
    }
}

impl From<ArcStat> for [f64; 5] {
    fn from(a: ArcStat) -> Self {
        [
            f64::from(a.cells),
            f64::from(a.emergent),
            a.p50_m,
            a.p90_m,
            a.max_m,
        ]
    }
}

impl ArcStat {
    /// The statistics of a set of elevations (m).
    pub fn of(mut z: Vec<f64>) -> ArcStat {
        if z.is_empty() {
            return ArcStat::default();
        }
        z.sort_by(f64::total_cmp);
        ArcStat {
            cells: z.len() as u32,
            emergent: (z.len() - z.partition_point(|&e| e <= 0.0)) as u32,
            p50_m: quantile(&z, 0.5).round(),
            p90_m: quantile(&z, 0.9).round(),
            max_m: z[z.len() - 1].round(),
        }
    }

    /// Share of the cells above the sea datum (0 of none).
    pub fn emergent_share(&self) -> f64 {
        f64::from(self.emergent) / f64::from(self.cells.max(1))
    }
}

/// Whether an arc cell is part of an island arc (ledger: PLT-220) (its arc crust was oceanic when the magma came):
/// oceanic crust, or crust whose arc magma is mostly of the oceanic accumulator (an island arc that
/// emerged and matured into continental crust).
pub fn is_island_arc(s: &Sample) -> bool {
    s.kind == OCEANIC || s.arc_oceanic_m > s.arc_continental_m
}

/// The oceanic and the continental arcs' elevations ([`ArcStat`]) among `samples`.
pub fn arc_stats(samples: &[Sample]) -> [ArcStat; 2] {
    let (mut oc, mut cont) = (Vec::new(), Vec::new());
    for s in samples {
        if s.is_hidden() || s.flags & ARC == 0 {
            continue;
        }
        let z = f64::from(s.elevation_m);
        if is_island_arc(s) {
            oc.push(z);
        } else {
            cont.push(z);
        }
    }
    [ArcStat::of(oc), ArcStat::of(cont)]
}

/// The arcs of the step ([`arc_stats`]); empty when the record is off.
pub fn arc_heights(st: &PlateState) -> [ArcStat; 2] {
    if st.dynamics.is_none() {
        return [ArcStat::default(); 2];
    }
    arc_stats(&st.samples)
}

/// What a step measured before `record_step` sees its end state: the fronts' lengths, the boundary
/// mix and the arc heights, taken where the flags are fresh ([`boundary_mix`], [`arc_heights`]),
/// and the force balance (`forces::record`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Observed<'a> {
    /// Each plate's subduction front length this step (`subduction::Pull::front_km`).
    pub front_km: &'a [f64],
    pub mix_km: [f64; MIX_CLASSES],
    /// Oceanic, continental.
    pub arcs: [ArcStat; 2],
    /// The force balance (`None` when it was not taken).
    pub forces: Option<&'a super::forces::Recorded>,
}

/// Appends this step's record. `seen` is what the step measured (the fronts' lengths per plate,
/// the boundary mix, the arcs), `pos` the samples' current positions, `last` whether the run's
/// final step has been taken. Reads the state and the world's new events, changes only the record.
pub fn record_step(
    st: &mut PlateState,
    world: &WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env<'_>,
    last: bool,
    seen: Observed<'_>,
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
    let spin_cos = SPIN_ANGLE_DEG.to_radians().cos();
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
                front_km: round(seen.front_km.get(k).copied().unwrap_or(0.0), 1.0),
                boundary_km: round(f64::from(bnd[k]) * spacing_km, 1.0),
                rot_change: round(change.unwrap_or(0.0), 1.0e4),
                rot_known: change.is_some(),
                pole_inside: pole_of[k].is_some_and(|j| {
                    nearest.plate(2 * j) == p.id || nearest.plate(2 * j + 1) == p.id
                }),
                guard: cap > 0.0 && w.length() >= cap * (1.0 - 1.0e-9),
                spinning: w.length() > 0.0
                    && w.normalize().dot(DVec3::from_array(p.centroid)).abs() >= spin_cos,
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
        mix_km: seen.mix_km,
        arc_oceanic: seen.arcs[0],
        arc_continental: seen.arcs[1],
        forces: seen.forces.map(|f| f.forces.clone()).unwrap_or_default(),
        slabs_km: seen.forces.map(|f| f.slabs_km.clone()).unwrap_or_default(),
        breakoffs: seen.forces.map_or(0, |f| f.breakoffs),
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
                spinning: true,
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
            serde_json::json!([3.0, 60.0, 20.0, 41.5, 300.0, 900.0, 0.0625, 13.0])
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

    fn nb(along: f64, kind: u8, side: Side) -> Neighbour {
        Neighbour { along, kind, side }
    }

    #[test]
    fn boundary_samples_are_classified_by_their_neighbours() {
        use mix::*;
        // A front: the fastest-converging neighbour on an active contact decides, by crust.
        let slab_oc = [
            nb(-10.0, OCEANIC, Side::Over),
            nb(-30.0, OCEANIC, Side::Over),
        ];
        assert_eq!(classify_boundary(OCEANIC, &slab_oc), Some((OCEAN_OCEAN, 2)));
        assert_eq!(
            classify_boundary(CONTINENTAL, &slab_oc),
            Some((OCEAN_CONTINENT, 2))
        );
        // The fastest one is continental: a collision, whichever crust the sample is.
        let cont = [
            nb(-10.0, OCEANIC, Side::Over),
            nb(-30.0, CONTINENTAL, Side::Over),
        ];
        assert_eq!(classify_boundary(OCEANIC, &cont), Some((COLLISION, 2)));
        assert_eq!(classify_boundary(CONTINENTAL, &cont), Some((COLLISION, 2)));
        // A front beats a neighbour moving apart on the same sample.
        let both = [nb(5.0, OCEANIC, Side::None), nb(-1.0, OCEANIC, Side::Over)];
        assert_eq!(classify_boundary(OCEANIC, &both), Some((OCEAN_OCEAN, 2)));
        // Divergent: ridge between oceanic crust, rift when either side is continental; half a
        // spacing from this side.
        let apart = [nb(8.0, OCEANIC, Side::None), nb(-2.0, OCEANIC, Side::None)];
        assert_eq!(classify_boundary(OCEANIC, &apart), Some((RIDGE, 1)));
        assert_eq!(classify_boundary(CONTINENTAL, &apart), Some((RIFT, 1)));
        let apart_c = [nb(8.0, CONTINENTAL, Side::None)];
        assert_eq!(classify_boundary(OCEANIC, &apart_c), Some((RIFT, 1)));
        // Converging but not on a contact, and sliding past: other.
        assert_eq!(
            classify_boundary(OCEANIC, &[nb(-4.0, OCEANIC, Side::None)]),
            Some((OTHER, 1))
        );
        assert_eq!(
            classify_boundary(OCEANIC, &[nb(0.0, OCEANIC, Side::None)]),
            Some((OTHER, 1))
        );
        // The subducting side of a front is not counted (the overriding side counts it); a
        // converging neighbour on an active contact the other way round, and no neighbour at all.
        assert_eq!(
            classify_boundary(OCEANIC, &[nb(-4.0, OCEANIC, Side::Under)]),
            None
        );
        assert_eq!(classify_boundary(OCEANIC, &[]), None);
        // A converging contact the sample overrides at no speed is not a front.
        assert_eq!(
            classify_boundary(OCEANIC, &[nb(0.0, OCEANIC, Side::Over)]),
            Some((OTHER, 1))
        );
    }

    fn arc_cell(kind: u8, z: f32, oceanic_m: f32, continental_m: f32, flags: u8) -> Sample {
        Sample {
            kind,
            elevation_m: z,
            arc_oceanic_m: oceanic_m,
            arc_continental_m: continental_m,
            flags,
            ..Sample::blank(0)
        }
    }

    #[test]
    fn arc_cells_are_split_by_the_crust_the_magma_came_through_and_summarised() {
        use crate::plates::state::{HIDDEN, TRENCH};
        let mut cells = Vec::new();
        // Oceanic arc: ten cells at -1000 ... -100 m and three above the sea.
        for i in 0..10 {
            cells.push(arc_cell(
                OCEANIC,
                -1000.0 + 100.0 * i as f32,
                50.0,
                0.0,
                ARC,
            ));
        }
        for z in [50.0, 400.0, 900.0] {
            cells.push(arc_cell(OCEANIC, z, 80.0, 0.0, ARC));
        }
        // An island arc that matured into continental crust is still an oceanic arc.
        cells.push(arc_cell(CONTINENTAL, 1200.0, 300.0, 40.0, ARC));
        // Continental arc.
        for z in [1500.0, 2500.0, 3500.0, 4500.0, 5500.0] {
            cells.push(arc_cell(CONTINENTAL, z, 0.0, 200.0, ARC));
        }
        // Not arc cells: no flag, hidden, another flag.
        cells.push(arc_cell(OCEANIC, 3000.0, 0.0, 0.0, 0));
        cells.push(arc_cell(CONTINENTAL, 3000.0, 0.0, 9.0, ARC | HIDDEN));
        cells.push(arc_cell(CONTINENTAL, 3000.0, 0.0, 9.0, TRENCH));
        let [oc, cont] = arc_stats(&cells);
        assert_eq!((oc.cells, oc.emergent), (14, 4));
        assert!((oc.emergent_share() - 4.0 / 14.0).abs() < 1e-12);
        assert_eq!(oc.max_m, 1200.0);
        // Sorted: -1000 ... -100 (10), 50, 400, 900, 1200: the median is between -100 and 50 at
        // rank 6.5 -> nearest rank 7 (-300 + ...); just check the order and that it is below 0.
        assert!(oc.p50_m < 0.0 && oc.p90_m >= 400.0 && oc.p90_m <= oc.max_m);
        assert_eq!((cont.cells, cont.emergent), (5, 5));
        assert_eq!(
            (cont.p50_m, cont.p90_m, cont.max_m),
            (3500.0, 5500.0, 5500.0)
        );
        // No arc cells: an empty summary, not a NaN.
        let none = arc_stats(&[arc_cell(OCEANIC, 5.0, 0.0, 0.0, 0)]);
        assert_eq!(none[0], ArcStat::default());
        assert_eq!(none[0].emergent_share(), 0.0);
        // A sea-level cell is not emergent.
        assert_eq!(ArcStat::of(vec![0.0, 0.0, 1.0]).emergent, 1);
    }

    #[test]
    fn the_mix_and_the_arcs_travel_in_the_record() {
        let mut d = Dynamics {
            version: VERSION,
            dt_myr: 2.0,
            samples: 100,
            ..Dynamics::default()
        };
        d.steps.push(StepRec {
            t: 2.0,
            mix_km: [120.0, 80.0, 30.0, 400.0, 20.0, 150.0],
            arc_oceanic: ArcStat::of(vec![-500.0, -100.0, 300.0]),
            arc_continental: ArcStat::of(vec![2000.0, 3000.0]),
            ..StepRec::default()
        });
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(
            v["steps"][0]["mix_km"],
            serde_json::json!([120.0, 80.0, 30.0, 400.0, 20.0, 150.0])
        );
        // [cells, emergent, p50, p90, max]
        assert_eq!(
            v["steps"][0]["arc_continental"],
            serde_json::json!([2.0, 2.0, 3000.0, 3000.0, 3000.0])
        );
        let back: Dynamics = serde_json::from_value(v.clone()).unwrap();
        assert_eq!(back, d);
        // A record without them reads as zeros (no data), not as an error.
        let mut old = v;
        for k in ["mix_km", "arc_oceanic", "arc_continental"] {
            old["steps"][0].as_object_mut().unwrap().remove(k);
        }
        let back: Dynamics = serde_json::from_value(old).unwrap();
        assert_eq!(back.steps[0].mix_km, [0.0; MIX_CLASSES]);
        assert_eq!(back.steps[0].arc_oceanic, ArcStat::default());
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

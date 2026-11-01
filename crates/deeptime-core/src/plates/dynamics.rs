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
//! | `belts` | one [`BeltRec`] per collision (a terrane of `collision_min_frac` or more) that ended in a transfer or weld: its belt, wedge, foredeep and slowdown |
//! | `steps[].forces`, `slabs_km`, `breakoffs`; `prev_balance` | version 4: the force balance (below) |
//! | `steps[].shown` | the crust that went under and the crust shown again, by cause, and what each resampling consumed and what escaped it ([`super::shown::Step`], 2026-10-24) |
//!
//! # The force balance (version 4)
//! `forces.rs` sums each plate's driving and resisting forces every step and solves the torque
//! balance for every plate's rotation, towards which the plates' motion relaxes (design
//! `docs/design/plate-force-balance.md`, tasks 5 and 7); the record keeps `StepRec::forces`
//! ([`PlateForces`]: the torque of each term, the mean in-plane stress, the answer's speed and its
//! change since the last step), `StepRec::slabs_km` (the slab length of each subducting contact)
//! and `StepRec::breakoffs`; `prev_balance` carries the last answers from step to step (the slabs
//! themselves are plate state, `PlateState::slabs`). It is taken right after the step's fronts are found, as the boundary mix.
//!
//! # Boundary mix and arc heights (version 3)
//! Two more measures for `plate-check`, measurement only (nothing a plate rule reads):
//! - `StepRec::mix_km`: the length of plate boundary by kind, km (one sample spacing per boundary
//!   sample, see [`classify_boundary`]): [`mix`]`::OCEAN_OCEAN`, `OCEAN_CONTINENT` (an oceanic
//!   slab going under an overriding plate whose arc stands on oceanic or continental crust: the
//!   front's own crust, or the plate's continental crust within `arc_distance_km` of it, PLT-370),
//!   `COLLISION` (continental crust at the subducting side), `RIDGE`, `RIFT` and `OTHER`
//!   (transform, or converging with no contact yet). The subducting length is the first two. It is taken right after the step's
//!   fronts are found (`subduction::fronts_and_uplift`), because the sample flags are cleared by
//!   a resampling; the motion is judged across the boundary's normal, with Bird's 20 degrees for
//!   a transform (PLT-310, PLT-311).
//! - `StepRec::arc_oceanic` and `arc_continental` ([`ArcStat`]): the elevation, m above the sea
//!   the step stood under, of the samples that received arc magma this step (flag `ARC`, the code's
//!   own arc cells: within `arc_width_km` of the arc distance from a front). An *oceanic arc* is
//!   one whose arc crust was oceanic when the magma came ([`is_island_arc`]); an island arc that
//!   emerged and matured into continental crust (`finish_samples`) is still an oceanic arc here.
//!   Taken after the step's surface and uplift are final.
//!
//! Lengths are Earth-normalized km (the plate model's sphere, `model_radius_km`), speeds mm/yr,
//! times Myr relative to world creation. "Visible" samples are those not hidden under another
//! plate. The active margin share is not repeated here: `notes.plates.margins` has it.
//!
//! # What each collision means
//! A collision is a state of a contact (`state::Collision`, `shortening.rs`) that builds its belt
//! step by step (`belts.rs`, tasks 3 and 4). `start_myr` and `end_myr` are the **first and last
//! step** of the collision state (the step continental crust first converged at the front, and
//! the step of the transfer or weld; the check's length is `end − start + dt`, counted from the
//! collision's first step and not from the contact's onset, which is `contact_onset_myr`).
//! `peak_uplift_m` is the most any ground rose by it (the step's largest uplift, summed over its
//! steps: the uplift at the front), `shortening_km` the convergence across the front integrated,
//! `belt` the extent of everything it raised (its width from where the belt and wedge rose across
//! the front, weighted by the uplift; its length the front's: `belts::footprint`),
//! and `lifted10`, `lifted100` and `emerged` the most samples one step raised by more than 10 m,
//! by more than 100 m and from below the sea to above it (their extents are not kept). A
//! collision state that ends without a transfer or weld (the plates part, the contact reverses or
//! closes, the run ends) is a [`PartingRec`].
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
use super::geometry::{Geo, Tree};
use super::kinematics::{quat, tangent};
use super::shortening;
use super::state::{ARC, CONTINENTAL, OCEANIC, PlateState, Sample};
use super::subduction;
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
    /// Collision belts: convergence history, belt, wedge and foredeep (`belts.rs`).
    pub belts: Vec<BeltRec>,
    /// Version 4 (`forces.rs`): each plate's balance rotation after the last step (rad/yr, for
    /// the next step's `PlateForces::rot_change`).
    #[serde(default)]
    pub prev_balance: Vec<(u32, [f64; 3])>,
    /// Each pair of plates' front after the last step by setting and state ([`PairFront`]: under,
    /// over, setting, 1 if subducting, half sample spacings), for the next step's
    /// [`FrontBands::gain`] (measurement only, PLT-683).
    #[serde(default)]
    pub prev_pair_front: Vec<(u32, u32, u8, u8, u64)>,
    /// Measurement only (the onset trace, 2026-10-31): every subduction contact's onset with its
    /// boundary the step before ([`OnsetRec`]) …
    #[serde(default)]
    pub onsets: Vec<OnsetRec>,
    /// … each pair of plates' boundary after the last step ([`Mix::pair_bounds`]) …
    #[serde(default)]
    pub prev_pair_bound: Vec<PairBound>,
    /// … each pair of plates' boundary history ([`PairHist`]) …
    #[serde(default)]
    pub pair_hist: Vec<PairHist>,
    /// … and the active contacts after the last step (under, over), to tell a reversal.
    #[serde(default)]
    pub prev_contacts: Vec<(u32, u32)>,
    /// Measurement only (the resampling trace, 2026-10-31): each resampling ([`ResampleRec`]) …
    #[serde(default)]
    pub resamples: Vec<ResampleRec>,
    /// … and how the last one made each sample ([`PROV_GAP`] and the other bits; set by
    /// `ridges::resample`, one per sample, valid while the lattice stands). Saved with the state
    /// so a resumed run is the same run; emptied at the run's last step, so the record keeps none.
    #[serde(default)]
    pub prov: Vec<u8>,
    /// Measurement only (the overlap trace, 2026-10-31): the contacts begun this step, their
    /// plates' motion across their overlaps ([`onset_overlaps`], set after `subduction::resolve`);
    /// the step's record reads and empties it, so it is empty whenever a state or record is
    /// written, and neither carries it.
    #[serde(skip)]
    pub onset_overlap: Vec<OnsetOverlap>,
    /// Measurement only: the pairs (lower id first) whose contact `subduction::resolve` reversed
    /// this step (`Resolved::reversed`), the onset trace's route 1, emptied by the step's record
    /// as `onset_overlap` is.
    #[serde(skip)]
    pub onset_reversed: Vec<(u32, u32)>,
    /// Measurement only: the pairs whose new contact crossed the gate this step, with the steps
    /// that added to the sum and the sum, km (`Resolved::crossed`), emptied by the step's record
    /// as `onset_overlap` is.
    #[serde(skip)]
    pub onset_crossed: Vec<((u32, u32), u32, f64)>,
}

/// Measurement only (the resampling trace, 2026-10-31): how a resampling made each new sample
/// ([`Dynamics::prov`], bits): in no group's footprint, new sea floor of the gap fill (PLT-502) …
pub const PROV_GAP: u8 = 1;
/// … covered only by held crust, new sea floor (PLT-542's one-way gap) …
pub const PROV_SLAB_GAP: u8 = 2;
/// … a gap pocket the gap fill does not reach, given the nearest old sample's plate …
pub const PROV_ORPHAN: u8 = 4;
/// … in more than one group's footprint (the top one took it) …
pub const PROV_CONTESTED: u8 = 8;
/// … given another plate than the old sample nearest it.
pub const PROV_MOVED: u8 = 16;
/// The number of [`Dynamics::prov`]'s bits.
pub const PROV_BITS: usize = 5;

/// Measurement only (the resampling trace): one resampling, at the end of the step that ends at
/// `t` (Myr): the new samples and how many of them carry each of [`Dynamics::prov`]'s bits, in
/// their order ([`PROV_GAP`], [`PROV_SLAB_GAP`], [`PROV_ORPHAN`], [`PROV_CONTESTED`],
/// [`PROV_MOVED`]).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResampleRec {
    pub t: f64,
    pub step: u64,
    pub samples: u32,
    pub bits: [u32; PROV_BITS],
}

/// Measurement only (the resampling trace): one new contact's front samples in its first step
/// ([`Mix::onset_prov`]): how many, and how many of them carry each of [`Dynamics::prov`]'s bits on
/// the overriding side (the front sample) and on the subducting side (its lattice neighbour
/// across the front, which may have gone under: `under_hidden`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OnsetProv {
    pub under: u32,
    pub over: u32,
    pub n: u32,
    pub over_bits: [u32; PROV_BITS],
    pub under_bits: [u32; PROV_BITS],
    pub under_hidden: u32,
}

/// Measurement only (the overlap trace, 2026-10-31): a contact begun this step, the two plates'
/// motion across its overlap (`subduction::OverlapMotion`, read after `subduction::resolve`;
/// PLT-810's gate reads the same at a contact's start).
pub type OnsetOverlap = subduction::OverlapMotion;

/// Measurement only (the overlap trace, 2026-10-31): each contact begun this step (a new one or
/// a reversal's new onset), the two plates' motion across its overlapping samples
/// ([`OnsetOverlap`]), from the step's overlaps (`subduction::overlaps`), the samples' positions
/// and their tree; called after `subduction::resolve`, which reads nothing it writes.
pub fn onset_overlaps(
    st: &PlateState,
    overlaps: &[subduction::Overlap],
    pos: &[DVec3],
    tree: &Tree,
    env: &Env<'_>,
) -> Vec<OnsetOverlap> {
    let new: BTreeSet<(u32, u32)> = st
        .contacts
        .iter()
        .filter(|c| c.onset_step == env.step)
        .map(|c| (c.under.min(c.over), c.under.max(c.over)))
        .collect();
    if new.is_empty() {
        return Vec::new();
    }
    subduction::overlap_motion(st, overlaps, pos, tree, env, |key| new.contains(&key))
        .into_values()
        .collect()
}

/// One step's state of the planet and its plates.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StepRec {
    /// Time at the end of the step, Myr.
    pub t: f64,
    /// Visible samples.
    pub visible: u32,
    /// Visible samples above the sea the step stood under: the land.
    pub land: u32,
    /// Visible samples above the sea the step before stood under (`None` on a record's first
    /// step): the land with the sea held where it was, so the plate check's land gain counts what
    /// the ground's motion made and not what a falling sea laid dry.
    #[serde(default)]
    pub land_held: Option<u32>,
    /// The sea the step stood under, m (the plate model's frame).
    #[serde(default)]
    pub sea_m: f64,
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
    /// The contacts whose slab passed the critical convergence this step (PLT-610; measurement
    /// only, PLT-683): `[mean convergence of its oceanic front, mm/yr; the contact's age, Myr;
    /// its oceanic front samples]` (`forces::Recorded::matured`).
    #[serde(default)]
    pub matured: Vec<[f64; 3]>,
    /// Length of plate boundary by kind, km, indexed by [`mix`] (all 0 in a record without it).
    #[serde(default)]
    pub mix_km: [f64; MIX_CLASSES],
    /// Elevation of the oceanic and continental arc cells of the step ([`ArcStat`]).
    #[serde(default)]
    pub arc_oceanic: ArcStat,
    #[serde(default)]
    pub arc_continental: ArcStat,
    /// The subducting boundary by slab length, convergence and contact age ([`FrontBands`]).
    #[serde(default)]
    pub front_bands: FrontBands,
    /// The crust that went under and the crust shown again this step, by cause, and at a
    /// resampling what it consumed and what escaped it (`shown.rs`; the front lane, 2026-10-24).
    #[serde(default)]
    pub shown: super::shown::Step,
    /// The live plates the step's solved balance left out (no visible sample when it was solved:
    /// `forces::Answer::omega` `None`), which keep their rotation; empty in almost every step.
    /// Always written: the checkpoint's postcard is positional, so a field skipped when empty
    /// does not decode (`CHECKPOINT_SCHEMA` 14).
    #[serde(default)]
    pub out_of_balance: Vec<u32>,
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

/// A collision belt's history and shape (`belts.rs`); the check reads it. `None` is "not
/// measured". `collision::collide` writes one per collision (a terrane of `collision_min_frac` or
/// more) that ends in a transfer or weld.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BeltRec {
    /// The collision's `start_myr` and `end_myr` and its two plates, to find its `CollisionRec`.
    pub start_myr: f64,
    #[serde(default)]
    pub end_myr: Option<f64>,
    pub under: u32,
    pub over: u32,
    /// It ended in a weld (else a terrane transfer).
    #[serde(default)]
    pub welded: bool,
    /// Convergence across the front at the collision's onset, and 20 Myr later (`None` while the
    /// collision has not lasted that long), mm/yr.
    pub convergence_onset_mm_yr: Option<f64>,
    pub convergence_20myr_mm_yr: Option<f64>,
    /// The belt's highest mean height above the sea within half its width of the front, m.
    #[serde(default)]
    pub peak_height_m: Option<f64>,
    /// Width and length of everything the collision raised, km (`belts::footprint`: the spread of
    /// the belt and wedge's uplift across the front, and the front's longest length).
    #[serde(default)]
    pub belt_width_km: Option<f64>,
    #[serde(default)]
    pub belt_length_km: Option<f64>,
    /// Width of the wedge on the incoming side when the belt stood highest, km.
    pub wedge_width_km: Option<f64>,
    /// Width, km, and depth below local isostasy, m, of the foreland basin in front of the wedge
    /// when the belt stood highest (`surface/flexure_m`; `belts::Foredeep`), for a belt that
    /// reached `belts::FOREDEEP_BELT_M`.
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
    /// The two plates welded into one.
    pub sutured: bool,
    /// With a weld, the plate that died (the smaller: the larger keeps its id).
    #[serde(default)]
    pub died: Option<u32>,
    /// It ended because its convergence had stalled (else: a terrane transfer on shortening).
    #[serde(default)]
    pub stalled: bool,
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
    /// Convergence across the front in the collision's last step, mm/yr.
    pub convergence_mm_yr: f64,
    /// The collision state's shortening, km: the convergence across the front integrated over
    /// the collision (`state::Collision::shortening_km`).
    #[serde(default)]
    pub front_shortening_km: f64,
    /// Crust that crossed the front over the collision state, km² (Earth-normalized): each
    /// step's shortening times its front length (`state::Collision::area_km2`).
    #[serde(default)]
    pub area_km2: f64,
    /// Convergence across the front when the collision state began, mm/yr.
    #[serde(default)]
    pub onset_convergence_mm_yr: Option<f64>,
    /// Continental samples of the subducting plate that went under since the contact began (or
    /// since its last collision), each counted once.
    pub cont_in: u32,
    /// The front's longest length, km, and the shortening across it, km (as
    /// `front_shortening_km`).
    pub front_km: f64,
    pub shortening_km: f64,
    /// The most any ground rose by the collision, m (see the module docs).
    pub peak_uplift_m: f64,
    /// The most samples one step raised by more than 10 m, by more than 100 m, and from below sea
    /// level to above it (counts only).
    pub lifted10: Footprint,
    pub lifted100: Footprint,
    pub emerged: Footprint,
    /// The extent of everything the collision raised (see the module docs).
    #[serde(default)]
    pub belt: Footprint,
    /// Birth of the terrane's plate and its parent.
    pub under_born_myr: f64,
    pub under_parent: Option<u32>,
    /// The belt's highest mean height above the sea within half its width of the front, m, and
    /// the crust the belt and wedge stacked, km² × m (as [`PartingRec`]'s; measurement only, the
    /// docking lane, 2026-10-30: the record kept neither for a terrane transfer).
    #[serde(default)]
    pub peak_height_m: f64,
    #[serde(default)]
    pub stacked_km2_m: f64,
    /// The crust the slab took instead of the belt where the terrane docked stripped to its cover
    /// (`tectonics.terrane_docking_cover`, PLT-770), km² × m (measurement only).
    #[serde(default)]
    pub stripped_km2_m: f64,
    /// The front's unit centroid in the step it was last seen (zero when it never had one), so a
    /// measure can find the belt on a map.
    #[serde(default)]
    pub centroid: [f64; 3],
}

/// Why a collision state ended with no transfer or weld ([`PartingRec::reason`]): no continental
/// crust at the front for longer than `shortening::GRACE_MYR` …
pub const PARTED: &str = "parted";
/// … the contact reversed its polarity …
pub const REVERSED: &str = "reversed";
/// … the contact closed (no overlap for longer than `contact_timeout_myr`, or one of its plates died) …
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
    /// Its belt's highest mean height, m, and the crust it stacked, km² × m (`belts.rs`).
    #[serde(default)]
    pub peak_height_m: f64,
    #[serde(default)]
    pub stacked_km2_m: f64,
    /// [`PARTED`], [`REVERSED`], [`CLOSED`], [`EMPTIED`] or [`OPEN`].
    pub reason: String,
    /// As [`CollisionRec::stripped_km2_m`] and [`CollisionRec::centroid`].
    #[serde(default)]
    pub stripped_km2_m: f64,
    #[serde(default)]
    pub centroid: [f64; 3],
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
            peak_height_m: col.belt.peak_height_m,
            stacked_km2_m: col.belt.stacked_km2_m,
            reason: reason.to_owned(),
            stripped_km2_m: col.belt.stripped_km2_m,
            centroid: col.centroid,
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

    /// Takes what `later` saw, as if its samples had been offered after this one's (P1-19: the
    /// nearest of both, the earlier on a tie, as [`NearestPlate::offer`] keeps it).
    pub fn merge(&mut self, later: &NearestPlate) {
        for (b, l) in self.best.iter_mut().zip(&later.best) {
            if l.0 > b.0 {
                *b = *l;
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
    // P1-19: in parallel; `total_cmp` orders every value, so any sort gives the same list.
    let mut ages: Vec<f64> = st
        .samples
        .par_iter()
        .filter(|s| !s.is_hidden() && s.kind == OCEANIC)
        .map(|s| s.age_myr(t))
        .collect();
    ages.par_sort_unstable_by(f64::total_cmp);
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
    /// An oceanic slab under an overriding plate's oceanic crust, with none of that plate's
    /// continental crust within the arc's distance of the front: an intra-oceanic arc.
    pub const OCEAN_OCEAN: usize = 0;
    /// An oceanic slab under an overriding plate's continental crust, at the front or within the
    /// arc's distance behind it (PLT-370): a continental arc.
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
    /// The same velocity along the boundary's normal towards the neighbour's plate
    /// ([`shortening::boundary_normal`]), mm/yr, with the same sign; zero where the boundary is a
    /// transform ([`shortening::is_transform`]).
    pub normal: f64,
    /// The neighbour's crust ([`OCEANIC`] or [`CONTINENTAL`]).
    pub kind: u8,
    pub side: Side,
}

/// What kind of boundary a visible sample is on (ledger: PLT-221), from its other-plate neighbours, and how much of
/// the boundary it stands for in half sample spacings (2 or 1): `None` when it is on no boundary
/// or on the subducting side of a front (the overriding side counts that front). The motion is
/// judged across the boundary, not along single lattice edges ([`Neighbour::normal`]): Bird's
/// classes are by the relative velocity against the boundary's azimuth, and one lattice edge's
/// direction is no azimuth (plates sliding past each other converge along half the edges of a
/// zigzag boundary and part along the rest). A sample on the overriding side of an active contact
/// whose boundary converges is a front, its class named by the crust of its fastest-converging
/// neighbour there (and, for an oceanic slab, by the front sample's own crust; the caller names an
/// ocean-ocean front ocean-continent when its arc stands on continental crust, PLT-370); a sample
/// with a neighbour across a parting boundary is divergent; transforms and converging boundaries
/// with no contact are the rest. A front counts one sample spacing (one
/// plate's side of the boundary, as `PlateStep::front_km` does); the others count half a spacing
/// from each of their two sides.
pub fn classify_boundary(v_kind: u8, nbrs: &[Neighbour]) -> Option<(usize, u32)> {
    if nbrs.is_empty() {
        return None;
    }
    if let Some(n) = front_index(nbrs).map(|k| &nbrs[k]) {
        let class = if n.kind == CONTINENTAL {
            mix::COLLISION
        } else if v_kind == CONTINENTAL {
            mix::OCEAN_CONTINENT
        } else {
            mix::OCEAN_OCEAN
        };
        return Some((class, 2));
    }
    if nbrs.iter().any(|n| n.normal > 0.0) {
        let continental =
            v_kind == CONTINENTAL || nbrs.iter().any(|n| n.normal > 0.0 && n.kind == CONTINENTAL);
        let class = if continental { mix::RIFT } else { mix::RIDGE };
        return Some((class, 1));
    }
    if nbrs.iter().any(|n| n.normal < 0.0 && n.side == Side::Under) {
        return None;
    }
    Some((mix::OTHER, 1))
}

/// Which of a boundary sample's neighbours makes it a front ([`classify_boundary`]): the
/// fastest-converging one across the boundary on the subducting side of an active contact.
fn front_index(nbrs: &[Neighbour]) -> Option<usize> {
    let mut best = f64::NEG_INFINITY;
    let mut front = None;
    for (k, n) in nbrs.iter().enumerate() {
        if n.normal < 0.0 && n.side == Side::Over && -n.along > best {
            best = -n.along;
            front = Some(k);
        }
    }
    front
}

/// The lower edges of [`FrontBands::slab`]'s bands, km of slab.
pub const SLAB_BANDS_KM: [f64; BANDS] = [0.0, 50.0, 100.0, 150.0, 300.0, 600.0];
/// The lower edges of [`FrontBands::normal`]'s bands, mm/yr of convergence across the boundary.
pub const NORMAL_BANDS_MM_YR: [f64; BANDS] = [0.0, 5.0, 10.0, 20.0, 40.0, 80.0];
/// The lower edges of [`FrontBands::age`]'s bands, Myr since the contact's onset.
pub const AGE_BANDS_MYR: [f64; BANDS] = [0.0, 6.0, 12.0, 24.0, 50.0, 100.0];
/// The number of bands of each [`FrontBands`] measure.
pub const BANDS: usize = 6;

/// The band of `x` among the lower edges `edges`.
fn band_of(x: f64, edges: &[f64; BANDS]) -> usize {
    edges.iter().rposition(|&e| x >= e).unwrap_or(0)
}

/// Measurement only (the arc-front task, 2026-10-20): the subducting boundary of the mix
/// ([`mix`]`::OCEAN_OCEAN`, then `OCEAN_CONTINENT`) by the length of its contact's slab
/// (`PlateState::slabs`), by the convergence across it and by its contact's age, km in the bands
/// of [`SLAB_BANDS_KM`], [`NORMAL_BANDS_MM_YR`] and [`AGE_BANDS_MYR`]; and the boundary of class
/// `OTHER` that converges across it (no contact yet; the incipient fronts are `incipient`'s),
/// km.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FrontBands {
    pub slab: [[f64; BANDS]; 2],
    pub normal: [[f64; BANDS]; 2],
    pub age: [[f64; BANDS]; 2],
    pub other_converging: f64,
    /// Measurement only (the plate-geometry task, 2026-10-21): the ocean-ocean front, km, by the
    /// overriding plate's continental share and the subducting plate's (bands of
    /// [`CONT_SHARE_BANDS`]), by the distance from the front to the overriding plate's nearest
    /// continental crust through that plate (bands of [`COAST_BANDS_KM`], Earth-normalised; the
    /// last band also holds a plate with none), and by how the contact's two plates are related
    /// ([`PAIR_INITIATION`], [`PAIR_RIFT`], [`PAIR_OTHER`]).
    #[serde(default)]
    pub oo_over_cont: [f64; BANDS],
    #[serde(default)]
    pub oo_under_cont: [f64; BANDS],
    #[serde(default)]
    pub oo_coast: [f64; BANDS],
    #[serde(default)]
    pub oo_pair: [f64; 3],
    /// The ocean-ocean and ocean-continent fronts (by the crust the arc would stand on), km, that
    /// are incipient (PLT-610, PLT-614): counted as `OTHER` in the mix's lengths, as converging
    /// boundary with no subduction zone yet (Bird 2003's oceanic convergent boundary), and in
    /// none of the bands above.
    #[serde(default)]
    pub incipient: [f64; 2],
    /// Measurement only (the front lane, 2026-10-25): the subducting fronts above whose slab
    /// reaches [`BENIOFF_DEPTH_KM`] under their arc, km: a slab at least
    /// hypot(`arc_distance_km`, [`BENIOFF_DEPTH_KM`]) long, as a straight slab from the trench to
    /// that depth under the arc's axis (Bird 2003's subduction zone has "a Benioff zone of
    /// intermediate to deep earthquakes and/or a parallel volcanic arc"). ledger: PLT-615
    #[serde(default)]
    pub benioff: [f64; 2],
    /// Measurement only (the M5 lane's cross-tab, 2026-10-26; ledger: PLT-683), km, by setting
    /// (ocean-ocean, ocean-continent, as above). `slow`: the part of `incipient` that is
    /// incipient by the slab's thermal floor alone (its contact's slab past the critical
    /// convergence, the sample converging below `forces::SLAB_THERMAL_FLOOR_MM_YR`; PLT-681).
    #[serde(default)]
    pub slow: [f64; 2],
    /// … the incipient fronts (either cause) by their convergence across the boundary, in the
    /// bands of [`NORMAL_BANDS_MM_YR`] (they sum to `incipient`) …
    #[serde(default)]
    pub incipient_normal: [[f64; BANDS]; 2],
    /// … the subducting fronts by the sample's convergence band (first index after the
    /// setting) and by the mean convergence over its pair of plates' front samples this step,
    /// incipient ones included (second index; the mean the pair's slab is fed with, FRC-020),
    /// both in the bands of [`NORMAL_BANDS_MM_YR`] (summed over the second index, `normal`) …
    #[serde(default)]
    pub normal_pair: [[[f64; BANDS]; BANDS]; 2],
    /// … the ocean-ocean subducting front by how its plates are related ([`PAIR_INITIATION`],
    /// [`PAIR_RIFT`], [`PAIR_OTHER`]) and its convergence band (sums: `oo_pair`, `normal[0]`) …
    #[serde(default)]
    pub oo_normal_pair: [[f64; BANDS]; 3],
    /// … and the front's turnover since the step before, by setting and state (0 incipient, 1
    /// subducting): over every pair of plates, setting and state, the growth (`gain`) and the
    /// shrinking (`loss`) of its front's length; a pair's front that lengthens at one end and
    /// shortens at the other in the same step counts only the difference. Zero on the record's
    /// first step (filled by [`record_step`] from [`Dynamics::prev_pair_front`]).
    #[serde(default)]
    pub gain: [[f64; 2]; 2],
    #[serde(default)]
    pub loss: [[f64; 2]; 2],
    /// … the subducting fronts' `gain` by their pair's slab, in the bands of [`SLAB_BANDS_KM`]:
    /// front that joins a pair whose slab is already long subducts from its first step.
    #[serde(default)]
    pub gain_slab: [[f64; BANDS]; 2],
    /// Measurement only (the intra-oceanic fronts diagnosis, 2026-10-31), km, the subducting
    /// fronts by setting and by their arc's age (the contact's `arc_onset_myr`, which a reversal
    /// resets; every subducting front has one), in the bands of [`AGE_BANDS_MYR`] …
    #[serde(default)]
    pub arc_age: [[f64; BANDS]; 2],
    /// … the subducting fronts by setting and by their contact's slab age at its onset
    /// (`Contact::onset_slab_age_myr`, the mean age of the floor that first went under; a
    /// contact begun at a ridge starts on floor near 0 Myr), in the bands of [`AGE_BANDS_MYR`] …
    #[serde(default)]
    pub onset_slab: [[f64; BANDS]; 2],
    /// … and the ocean-ocean subducting front by its contact's age (rows) and the overriding
    /// plate's floor age at the front sample (columns), both in the bands of [`AGE_BANDS_MYR`]:
    /// floor younger than its contact was made after the subduction zone began.
    #[serde(default)]
    pub oo_over_floor: [[f64; BANDS]; BANDS],
    /// … and the subducting fronts by setting and by their contact's onset ([`OnsetRec`], the
    /// onset trace): by the boundary the step before (`onset_kind`, [`ONSET_KINDS`] then
    /// unknown: a contact begun before the record or not found), by the route (`onset_route`:
    /// plates that came to converge, a reversal, an initiation's pair, unknown) and by the two
    /// floors at that boundary against [`super::forces::SLAB_BUOYANT_MYR`] (`onset_buoyant`:
    /// both younger, the subducting side's only, the overriding side's only, neither, unknown).
    #[serde(default)]
    pub onset_kind: [[f64; ONSET_KINDS.len() + 1]; 2],
    #[serde(default)]
    pub onset_route: [[f64; 4]; 2],
    #[serde(default)]
    pub onset_buoyant: [[f64; 5]; 2],
}

/// [`OnsetRec::kind`]: the pair's boundary the step before a contact's onset, by its longest
/// class: spreading ([`mix::RIDGE`] or [`mix::RIFT`]), [`mix::OTHER`] converging across it on
/// the whole, [`mix::OTHER`] not converging (a transform), a front ([`mix::OCEAN_OCEAN`] or
/// [`mix::OCEAN_CONTINENT`]), a [`mix::COLLISION`], or no common boundary.
pub const ONSET_KINDS: [&str; 6] = [
    "spreading",
    "converging",
    "transform",
    "front",
    "collision",
    "none",
];

/// Measurement only (the onset trace, 2026-10-31): one pair of plates' common boundary in one
/// step, from [`boundary_mix`]'s boundary samples on both plates: its length by [`mix`] class
/// (half sample spacings; a sample counts towards every other plate it borders), the mean velocity
/// across it over the samples' neighbours on the other plate (mm/yr, negative converging), and for
/// each plate (`lo` first) its oceanic boundary samples, their mean floor age (Myr, -1 with none)
/// and its continental boundary samples. The subducting side of a front is no boundary sample
/// ([`classify_boundary`]), so a front's pair holds its overriding side only.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PairBound {
    pub lo: u32,
    pub hi: u32,
    pub class: [u64; MIX_CLASSES],
    pub normal_mm_yr: f64,
    pub ocean: [u32; 2],
    pub age_myr: [f64; 2],
    pub cont: [u32; 2],
}

impl PairBound {
    /// [`ONSET_KINDS`]' index of this boundary.
    pub fn kind(&self) -> u8 {
        let spread = self.class[mix::RIDGE] + self.class[mix::RIFT];
        let front = self.class[mix::OCEAN_OCEAN] + self.class[mix::OCEAN_CONTINENT];
        let parts = [
            (spread, 0u8),
            (self.class[mix::OTHER], 1),
            (front, 3),
            (self.class[mix::COLLISION], 4),
        ];
        let (n, k) = parts
            .iter()
            .copied()
            .fold((0u64, 5u8), |best, p| if p.0 > best.0 { p } else { best });
        if n == 0 {
            5
        } else if k == 1 && self.normal_mm_yr >= 0.0 {
            2
        } else {
            k
        }
    }
}

/// Measurement only (the onset trace): a pair of plates' boundary history ([`record_step`]):
/// when it was first seen, Myr; since when its mean velocity across has been converging at every
/// step (`None` while not); and the last step any of it spread (ridge or rift).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PairHist {
    pub lo: u32,
    pub hi: u32,
    pub first_myr: f64,
    pub converging_since_myr: Option<f64>,
    pub spread_last_myr: Option<f64>,
}

/// Measurement only (the onset trace, 2026-10-31): a subduction contact's onset (a new contact,
/// or a polarity reversal's new onset) with its pair's boundary the step before (`before`, the
/// default with none), its history then (the boundary's age since the pair first met, how long
/// it had converged, since it last spread; -1 for never), and the contact's own onset slab age
/// and convergence. `route`: 0 plates that came to converge, 1 a reversal (the pair's contact
/// the step before had the other polarity), 2 a plate and the plate an initiation split off it
/// (PLT-690). `kind`: [`ONSET_KINDS`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OnsetRec {
    pub t: f64,
    pub step: u64,
    pub under: u32,
    pub over: u32,
    pub setting: u8,
    pub route: u8,
    pub kind: u8,
    pub before: PairBound,
    pub boundary_age_myr: f64,
    pub converging_myr: f64,
    pub since_spread_myr: f64,
    pub slab_age_myr: f64,
    pub convergence_mm_yr: f64,
    /// The resampling trace (2026-10-31): Myr since the last resampling's step ended (-1 with
    /// none), the contact's front samples in its first step and their [`Dynamics::prov`] bits
    /// ([`OnsetProv`]) …
    #[serde(default)]
    pub since_resample_myr: f64,
    #[serde(default)]
    pub prov: OnsetProv,
    /// … and what became of it, with a split's copies: Myr from the onset to its first front past
    /// incipient (-1 never), its slab's longest (the cumulative oceanic convergence, FRC-020), km,
    /// its subducting front over the run by setting now (ocean-ocean, ocean-continent), km × Myr,
    /// and the last step it was active, Myr.
    #[serde(default)]
    pub subducting_after_myr: f64,
    #[serde(default)]
    pub max_slab_km: f64,
    #[serde(default)]
    pub front_km_myr: [f64; 2],
    #[serde(default)]
    pub last_myr: f64,
    /// The overlap trace (2026-10-31): its plates' motion across its overlap at the onset
    /// ([`OnsetOverlap`]; the default for a contact begun with no overlap).
    #[serde(default)]
    pub overlap: OnsetOverlap,
    /// The gate's crossing (PLT-810; PR #55's third review): the steps that added to the pair's
    /// cumulative convergence before its contact began and the sum then, km (Earth-normalized; its
    /// mean rate is the sum over the steps' Myr). 0 and 0 for a contact the gate did not hold (the
    /// switch off, a reversal, continental crust on both sides).
    #[serde(default)]
    pub cross_steps: u32,
    #[serde(default)]
    pub cross_km: f64,
}

impl OnsetRec {
    /// The floors at the boundary the step before against the age below which oceanic
    /// lithosphere is not denser than the mantle ([`super::forces::SLAB_BUOYANT_MYR`]): 0 both
    /// younger, 1 the subducting side's only, 2 the overriding side's only, 3 neither (or the
    /// overriding side continental), 4 unknown (no oceanic floor of the subducting plate there).
    pub fn buoyant(&self) -> u8 {
        let side = |p: u32| usize::from(p != self.before.lo);
        let (u, o) = (side(self.under), side(self.over));
        if self.before.ocean[u] == 0 {
            return 4;
        }
        let young = |i: usize| {
            self.before.ocean[i] > 0 && self.before.age_myr[i] < super::forces::SLAB_BUOYANT_MYR
        };
        match (young(u), young(o)) {
            (true, true) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (false, false) => 3,
        }
    }
}

/// The relation of two plates in the record's plate lives: [`PAIR_INITIATION`] (one split off
/// the other by a subduction initiation), [`PAIR_RIFT`] (one rifted off the other, or both off
/// the same parent) or [`PAIR_OTHER`].
fn pair_relation(lives: &BTreeMap<u32, (Option<u32>, &str)>, a: u32, b: u32) -> usize {
    let la = lives.get(&a).copied().unwrap_or((None, ""));
    let lb = lives.get(&b).copied().unwrap_or((None, ""));
    let child = |(parent, origin): (Option<u32>, &str), of: u32, how: &str| {
        parent == Some(of) && origin == how
    };
    if child(la, b, "subduction_initiation") || child(lb, a, "subduction_initiation") {
        PAIR_INITIATION
    } else if child(la, b, "rift")
        || child(lb, a, "rift")
        || (la.1 == "rift" && lb.1 == "rift" && la.0.is_some() && la.0 == lb.0)
    {
        PAIR_RIFT
    } else {
        PAIR_OTHER
    }
}

/// One pair of plates' front in one setting and state this step ([`Mix::pairs`]; measurement
/// only, PLT-683): the plate going under, the overriding plate, the setting (0 ocean-ocean, 1
/// ocean-continent), whether it subducts (else incipient), its length in half sample spacings
/// and its pair's slab, km.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PairFront {
    pub under: u32,
    pub over: u32,
    pub setting: u8,
    pub subducting: bool,
    pub units: u64,
    pub slab_km: f64,
}

/// The depth of the slab under the arc a subducting front's slab must reach for
/// [`FrontBands::benioff`], km: Crameri et al. 2020, "The progression from no subduction to
/// subduction, marked by a slab reaching around 100 km depth"; Syracuse & Abers 2006 (abstract)
/// find the slab under the arcs' volcanoes at a global average of 105 km. ledger: PLT-615
pub const BENIOFF_DEPTH_KM: f64 = 100.0;

/// The lower edges of [`FrontBands::oo_over_cont`]'s and `oo_under_cont`'s bands: a plate's
/// continental share. These bands and the two below are measurement bins, not physics
/// (game-choice, the plate-geometry task 2026-10-21; ledger: PLT-370).
pub const CONT_SHARE_BANDS: [f64; BANDS] = [0.0, 0.01, 0.05, 0.15, 0.3, 0.5];
/// The lower edges of [`FrontBands::oo_coast`]'s bands, km.
pub const COAST_BANDS_KM: [f64; BANDS] = [0.0, 150.0, 300.0, 600.0, 1200.0, 2400.0];
/// [`FrontBands::oo_pair`]: one plate was split off the other by a subduction initiation.
pub const PAIR_INITIATION: usize = 0;
/// … one rifted off the other, or both off the same parent.
pub const PAIR_RIFT: usize = 1;
/// … neither.
pub const PAIR_OTHER: usize = 2;

/// [`boundary_mix`]'s answer: the length by kind and the subducting length's bands.
#[derive(Clone, Debug, Default)]
pub struct Mix {
    pub km: [f64; MIX_CLASSES],
    pub bands: FrontBands,
    /// Each pair of plates' front by setting and state, ordered by (under, over, setting,
    /// subducting): what the front's turnover ([`FrontBands::gain`]) is measured from.
    pub pairs: Vec<PairFront>,
    /// Each pair of plates' common boundary, ordered by (lo, hi) (the onset trace).
    pub pair_bounds: Vec<PairBound>,
    /// The contacts begun this step, their front samples' provenance (the resampling trace).
    pub onset_prov: Vec<OnsetProv>,
}

/// One front sample of [`boundary_mix`], kept for the cross-tabs that need the whole step
/// (measurement only, PLT-683).
#[derive(Clone)]
struct FrontRec {
    v: u32,
    under: u32,
    over: u32,
    setting: usize,
    subducting: bool,
    /// Incipient by the slab's thermal floor alone (PLT-681).
    slow: bool,
    v_n: f64,
    half: u32,
    pair: usize,
    slab_km: f64,
    /// Measurement only (FrontBands::arc_age, onset_slab, oo_over_floor): the contact's age, its
    /// arc's age (NaN with no arc onset), its slab's age at its onset and the overriding sample's
    /// floor age, Myr.
    contact_age: f64,
    arc_age: f64,
    onset_slab_age: f64,
    over_floor_age: f64,
    /// The contact's onset (kind, route, buoyancy; FrontBands::onset_*).
    onset: [u8; 3],
    /// The resampling trace: the subducting side's lattice neighbour, and whether the contact
    /// began this step.
    u: u32,
    new: bool,
}

/// An active contact's measures for [`boundary_mix`]'s bands (measurement only): its slab, km,
/// its age, its arc's age (NaN with no arc onset) and its slab's age at its onset, Myr.
#[derive(Clone, Copy)]
struct ContactMeas {
    slab_km: f64,
    age: f64,
    arc_age: f64,
    onset_slab_age: f64,
    onset: [u8; 3],
    new: bool,
}

/// One pair of plates' boundary samples in a step, as integers so the sums do not depend on the
/// order the samples are taken in ([`PairBound`]): the velocity across in µm/yr, the floor age in
/// hundredths of a Myr.
#[derive(Clone, Default)]
struct PairAcc {
    class: [u64; MIX_CLASSES],
    normal_um_yr: i64,
    normal_n: u64,
    ocean: [u64; 2],
    age_centi: [u64; 2],
    cont: [u64; 2],
}

/// [`Mix`] in half sample spacings, summed over samples.
#[derive(Clone, Default)]
struct MixUnits {
    km: [u64; MIX_CLASSES],
    slab: [[u64; BANDS]; 2],
    normal: [[u64; BANDS]; 2],
    age: [[u64; BANDS]; 2],
    other_converging: u64,
    oo: [[u64; BANDS]; 3],
    oo_pair: [u64; 3],
    incipient: [u64; 2],
    benioff: [u64; 2],
    /// The front samples, for the cross-tabs that need the whole step (PLT-683).
    recs: Vec<FrontRec>,
    /// Each pair of plates' boundary samples (the onset trace).
    pair_bound: BTreeMap<(u32, u32), PairAcc>,
}

impl MixUnits {
    fn add(mut self, b: MixUnits) -> MixUnits {
        for (x, y) in self.km.iter_mut().zip(b.km) {
            *x += y;
        }
        for (row, other) in self.oo.iter_mut().zip(b.oo) {
            for (x, y) in row.iter_mut().zip(other) {
                *x += y;
            }
        }
        for (x, y) in self.oo_pair.iter_mut().zip(b.oo_pair) {
            *x += y;
        }
        for (mine, theirs) in [
            (&mut self.incipient, b.incipient),
            (&mut self.benioff, b.benioff),
        ] {
            for (x, y) in mine.iter_mut().zip(theirs) {
                *x += y;
            }
        }
        for (mine, theirs) in [
            (&mut self.slab, b.slab),
            (&mut self.normal, b.normal),
            (&mut self.age, b.age),
        ] {
            for (row, other) in mine.iter_mut().zip(theirs) {
                for (x, y) in row.iter_mut().zip(other) {
                    *x += y;
                }
            }
        }
        self.other_converging += b.other_converging;
        self.recs.extend(b.recs);
        for (key, y) in b.pair_bound {
            let x = self.pair_bound.entry(key).or_default();
            for (a, b) in x.class.iter_mut().zip(y.class) {
                *a += b;
            }
            x.normal_um_yr += y.normal_um_yr;
            x.normal_n += y.normal_n;
            for i in 0..2 {
                x.ocean[i] += y.ocean[i];
                x.age_centi[i] += y.age_centi[i];
                x.cont[i] += y.cont[i];
            }
        }
        self
    }
}

/// The length of plate boundary by kind, km ([`mix`]), at the plates' positions `pos` and the
/// motion the step's fronts were found with (call it after `fronts_and_uplift`, before the
/// motion is updated). All zeros when the record is off. With `classes` (one per sample), each
/// visible boundary sample's class is stored there as the [`mix`] class plus one (0: on no
/// boundary, or on the subducting side of a front), for the crust budget's maps (measurement
/// only). `tree` holds every sample at `pos`: with `front_current_geometry` a converging neighbour
/// of a plate subducting under this one is a boundary only while a visible sample of that plate
/// stands within `subduction::trench_row_rad` now, as the fronts take it (ledger: PLT-582).
pub fn boundary_mix(
    st: &PlateState,
    geo: &Geo,
    pos: &[DVec3],
    tree: &Tree,
    env: &Env<'_>,
    classes: Option<&[std::sync::atomic::AtomicU8]>,
) -> Mix {
    if st.dynamics.is_none() {
        return Mix::default();
    }
    let classes = classes.filter(|c| c.len() == st.samples.len());
    let samples = &st.samples;
    let plate_of: Vec<u32> = (0..samples.len()).map(|i| st.plate_of(i)).collect();
    let omega: Vec<DVec3> = st
        .plates
        .iter()
        .map(|p| DVec3::from_array(p.omega))
        .collect();
    let rots: Vec<_> = st.groups.iter().map(|g| quat(g.rot)).collect();
    let pts = geo.points();
    // The working subduction zones, as the fronts found them (ledger: PLT-550).
    let active: BTreeSet<(u32, u32)> = st
        .contacts
        .iter()
        .filter(|c| c.active(env.step))
        .map(|c| (c.under, c.over))
        .collect();
    // Each active contact's slab length, km, and age, Myr (the bands, measurement only).
    let slab_km: BTreeMap<(u32, u32), f64> =
        st.slabs.iter().map(|s| ((s.under, s.over), s.km)).collect();
    // The onset trace: each recorded onset's categories, by its step, slab age and convergence
    // (`onset_key`: a split's copies keep them; a reversal's onset has its own record).
    let onset_of: BTreeMap<(u64, u64, u64), [u8; 3]> = st
        .dynamics
        .as_ref()
        .map(|d| {
            d.onsets
                .iter()
                .map(|o| {
                    (
                        onset_key(o.step, o.slab_age_myr, o.convergence_mm_yr),
                        [o.kind, o.route, o.buoyant()],
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let contact_of: BTreeMap<(u32, u32), ContactMeas> = st
        .contacts
        .iter()
        .filter(|c| c.active(env.step))
        .map(|c| {
            let key = (c.under, c.over);
            let meas = ContactMeas {
                slab_km: slab_km.get(&key).copied().unwrap_or(0.0),
                age: env.t_myr - c.onset_myr,
                arc_age: c.arc_onset_myr.map_or(f64::NAN, |a| env.t_myr - a),
                onset_slab_age: c.onset_slab_age_myr,
                onset: onset_of
                    .get(&onset_key(
                        c.onset_step,
                        c.onset_slab_age_myr,
                        c.onset_convergence_mm_yr,
                    ))
                    .copied()
                    .unwrap_or([ONSET_KINDS.len() as u8, 3, 4]),
                new: c.onset_step == env.step,
            };
            (key, meas)
        })
        .collect();
    let norm = env.norm;
    // Measurement only (the ocean-ocean front's diagnostics, FrontBands::oo_*): how each active
    // contact's plates are related, and each visible sample's lattice steps to its plate's
    // nearest visible continental crust through that plate.
    let lives: BTreeMap<u32, (Option<u32>, &str)> = st
        .dynamics
        .as_ref()
        .map(|d| {
            d.plates
                .iter()
                .map(|p| (p.id, (p.parent, p.origin.as_str())))
                .collect()
        })
        .unwrap_or_default();
    let pair_of = |a: u32, b: u32| pair_relation(&lives, a, b);
    let pairs: BTreeMap<(u32, u32), usize> = active
        .iter()
        .map(|&(u, o)| ((u, o), pair_of(u, o)))
        .collect();
    let coast_steps: Vec<u32> = {
        let mut d = vec![u32::MAX; samples.len()];
        let mut queue = std::collections::VecDeque::new();
        for (i, s) in samples.iter().enumerate() {
            if !s.is_hidden() && s.kind == CONTINENTAL {
                d[i] = 0;
                queue.push_back(i as u32);
            }
        }
        while let Some(v) = queue.pop_front() {
            let pv = plate_of[v as usize];
            for (u, _) in geo.edges(v) {
                let ui = u as usize;
                if d[ui] == u32::MAX && !samples[ui].is_hidden() && plate_of[ui] == pv {
                    d[ui] = d[v as usize] + 1;
                    queue.push_back(u);
                }
            }
        }
        d
    };
    let spacing_km = norm.km(norm.spacing_rad());
    // ledger: PLT-582 (the front on the current geometry)
    let row = env
        .k
        .front_current_geometry
        .then(|| subduction::trench_row_rad(norm.spacing_rad()));
    // An ocean-ocean front whose overriding plate has continental crust within the arc's distance
    // of it, through that plate, builds its arc on that crust: a continental arc, the class
    // Stern & Scholl (2010) split the world's arcs by (the crust the arc stands on), so it counts
    // as ocean-continent. Lattice steps, so the distance is an upper bound. ledger: PLT-370
    // The slab below which a front is incipient (PLT-610; 0 with the switch off), and the slab
    // that reaches the depth under the arc (PLT-615).
    let young_km = super::forces::incipient_km(env.k);
    let benioff_km = env.k.arc_distance_km.hypot(BENIOFF_DEPTH_KM);
    let arc_on_continent = |v: usize| {
        coast_steps[v] != u32::MAX
            && f64::from(coast_steps[v]) * spacing_km <= env.k.arc_distance_km
    };
    let units = (0..samples.len())
        .into_par_iter()
        .fold(MixUnits::default, |mut acc, v| {
            let sv = &samples[v];
            if sv.is_hidden() {
                return acc;
            }
            let pv = plate_of[v];
            let x = pos[v];
            let rot = rots[sv.group as usize];
            let mut buf = [Neighbour {
                along: 0.0,
                normal: 0.0,
                kind: OCEANIC,
                side: Side::None,
            }; 16];
            let mut plate_buf = [0u32; 16];
            let mut idx_buf = [0u32; 16];
            let mut n = 0;
            // The boundary's normal towards each plate met, found once per plate.
            let mut normals: [(u32, DVec3); 4] = [(u32::MAX, DVec3::ZERO); 4];
            let mut beside = subduction::SlabBeside::new(row);
            for (u, _) in geo.edges(v as u32) {
                let pu = plate_of[u as usize];
                if pu == pv || n == buf.len() {
                    continue;
                }
                let out = tangent(x, rot * (pts[u as usize] - pts[v])).normalize_or_zero();
                let rel = norm.velocity_mm_yr(omega[pu as usize] - omega[pv as usize], x);
                let slot = normals
                    .iter()
                    .position(|&(p, _)| p == pu || p == u32::MAX)
                    .unwrap_or(normals.len() - 1);
                if normals[slot].0 != pu {
                    let nrm =
                        shortening::boundary_normal(geo, pos, &rots, samples, &plate_of, v, pu);
                    normals[slot] = (pu, nrm);
                }
                let nrm = normals[slot].1;
                // ledger: PLT-311 (Bird's transforms: within 20 degrees of the boundary)
                let across = if shortening::is_transform(rel, nrm) {
                    0.0
                } else {
                    rel.dot(nrm)
                };
                // A plate subducting here, converging, with none of its crust beside this sample
                // now: the frozen lattice's neighbour went under and on, and this is no boundary
                // (PLT-582). An opening or a transform of the same plate is asked nothing: its gap
                // is the opening's, which the next resampling fills.
                if across < 0.0
                    && active.contains(&(pu, pv))
                    && !beside.has(samples, &plate_of, tree, x, pu)
                {
                    continue;
                }
                buf[n] = Neighbour {
                    along: rel.dot(out),
                    normal: across,
                    kind: samples[u as usize].kind,
                    side: if active.contains(&(pu, pv)) {
                        Side::Over
                    } else if active.contains(&(pv, pu)) {
                        Side::Under
                    } else {
                        Side::None
                    },
                };
                plate_buf[n] = pu;
                idx_buf[n] = u;
                n += 1;
            }
            if let Some((class, half)) = classify_boundary(sv.kind, &buf[..n]) {
                let class = if class == mix::OCEAN_OCEAN && arc_on_continent(v) {
                    mix::OCEAN_CONTINENT
                } else {
                    class
                };
                let setting = match class {
                    mix::OCEAN_OCEAN => Some(0),
                    mix::OCEAN_CONTINENT => Some(1),
                    _ => None,
                };
                let front = setting.zip(front_index(&buf[..n])).map(|(k, f)| {
                    let (km, age) = contact_of
                        .get(&(plate_buf[f], pv))
                        .map_or((0.0, 0.0), |m| (m.slab_km, m.age));
                    (k, f, km, age)
                });
                // An incipient front (PLT-610) is no subduction zone yet: a converging boundary
                // with no slab sunk and no arc (ledger: PLT-614); nor is a front converging too
                // slowly for a slab to form, whatever its contact's slab (PLT-681).
                let young_slab = front.is_some_and(|(_, _, km, _)| km < young_km);
                let slow = front
                    .is_some_and(|(_, f, _, _)| super::forces::below_floor(env.k, -buf[f].normal));
                let young = young_slab || slow;
                let class = if young { mix::OTHER } else { class };
                if let Some(c) = classes {
                    c[v].store(class as u8 + 1, std::sync::atomic::Ordering::Relaxed);
                }
                acc.km[class] += u64::from(half);
                // Measurement only (the onset trace): this sample towards each other plate it
                // borders.
                for j in 0..n {
                    let pu = plate_buf[j];
                    if plate_buf[..j].contains(&pu) {
                        continue;
                    }
                    let key = (pv.min(pu), pv.max(pu));
                    let e = acc.pair_bound.entry(key).or_default();
                    e.class[class] += u64::from(half);
                    for (b, _) in buf[..n].iter().zip(&plate_buf[..n]).filter(|x| *x.1 == pu) {
                        e.normal_um_yr += (b.normal * 1000.0).round() as i64;
                        e.normal_n += 1;
                    }
                    let side = usize::from(pv != key.0);
                    if sv.kind == CONTINENTAL {
                        e.cont[side] += 1;
                    } else {
                        e.ocean[side] += 1;
                        e.age_centi[side] += (sv.age_myr(env.t_myr) * 100.0).round() as u64;
                    }
                }
                if let Some((k, f, km, age)) = front {
                    // Measurement only (PLT-683): the front sample for the step's cross-tabs.
                    let under = plate_buf[f];
                    let meas = contact_of.get(&(under, pv));
                    acc.recs.push(FrontRec {
                        v: v as u32,
                        under,
                        over: pv,
                        setting: k,
                        subducting: !young,
                        slow: slow && !young_slab,
                        v_n: -buf[f].normal,
                        half,
                        pair: pairs.get(&(under, pv)).copied().unwrap_or(PAIR_OTHER),
                        slab_km: km,
                        contact_age: age,
                        arc_age: meas.map_or(f64::NAN, |m| m.arc_age),
                        onset_slab_age: meas.map_or(f64::NAN, |m| m.onset_slab_age),
                        over_floor_age: sv.age_myr(env.t_myr),
                        onset: meas.map_or([ONSET_KINDS.len() as u8, 3, 4], |m| m.onset),
                        u: idx_buf[f],
                        new: meas.is_some_and(|m| m.new),
                    });
                    if young {
                        acc.incipient[k] += u64::from(half);
                        return acc;
                    }
                    // ledger: PLT-615 (measurement only)
                    if km >= benioff_km {
                        acc.benioff[k] += u64::from(half);
                    }
                    acc.slab[k][band_of(km, &SLAB_BANDS_KM)] += u64::from(half);
                    acc.normal[k][band_of(-buf[f].normal, &NORMAL_BANDS_MM_YR)] += u64::from(half);
                    acc.age[k][band_of(age, &AGE_BANDS_MYR)] += u64::from(half);
                    if k == 0 {
                        let under = plate_buf[f];
                        let share = |p: u32| st.plates[p as usize].continental_frac;
                        acc.oo[0][band_of(share(pv), &CONT_SHARE_BANDS)] += u64::from(half);
                        acc.oo[1][band_of(share(under), &CONT_SHARE_BANDS)] += u64::from(half);
                        let km = if coast_steps[v] == u32::MAX {
                            f64::INFINITY
                        } else {
                            f64::from(coast_steps[v]) * spacing_km
                        };
                        acc.oo[2][band_of(km, &COAST_BANDS_KM)] += u64::from(half);
                        let pair = pairs.get(&(under, pv)).copied().unwrap_or(PAIR_OTHER);
                        acc.oo_pair[pair] += u64::from(half);
                    }
                }
                if class == mix::OTHER && buf[..n].iter().any(|b| b.normal < 0.0) {
                    acc.other_converging += u64::from(half);
                }
            }
            acc
        })
        .reduce(MixUnits::default, MixUnits::add);
    let mut recs = units.recs;
    // The resampling trace: the contacts begun this step, their front samples' provenance.
    let prov: &[u8] = st
        .dynamics
        .as_ref()
        .map(|d| d.prov.as_slice())
        .filter(|p| p.len() == samples.len())
        .unwrap_or(&[]);
    let mut onset_prov: BTreeMap<(u32, u32), OnsetProv> = BTreeMap::new();
    for r in recs.iter().filter(|r| r.new) {
        let e = onset_prov.entry((r.under, r.over)).or_insert(OnsetProv {
            under: r.under,
            over: r.over,
            ..OnsetProv::default()
        });
        e.n += 1;
        e.under_hidden += u32::from(samples[r.u as usize].is_hidden());
        if prov.is_empty() {
            continue;
        }
        for b in 0..PROV_BITS {
            e.over_bits[b] += u32::from(prov[r.v as usize] & (1 << b) != 0);
            e.under_bits[b] += u32::from(prov[r.u as usize] & (1 << b) != 0);
        }
    }
    let km = |u: u64| (u as f64 * 0.5 * spacing_km).round();
    let bands = |b: [[u64; BANDS]; 2]| b.map(|row| row.map(km));
    // The cross-tabs over the whole step's front samples (PLT-683), in sample order so the
    // float sums are reproducible.
    recs.sort_unstable_by_key(|r| r.v);
    let mut pair_mean: BTreeMap<(u32, u32), (f64, u64)> = BTreeMap::new();
    for r in &recs {
        let m = pair_mean.entry((r.under, r.over)).or_default();
        m.0 += r.v_n * f64::from(r.half);
        m.1 += u64::from(r.half);
    }
    let mut slow = [0u64; 2];
    let mut incipient_normal = [[0u64; BANDS]; 2];
    let mut normal_pair = [[[0u64; BANDS]; BANDS]; 2];
    let mut oo_normal_pair = [[0u64; BANDS]; 3];
    let mut arc_age = [[0u64; BANDS]; 2];
    let mut onset_slab = [[0u64; BANDS]; 2];
    let mut oo_over_floor = [[0u64; BANDS]; BANDS];
    let mut onset_kind = [[0u64; ONSET_KINDS.len() + 1]; 2];
    let mut onset_route = [[0u64; 4]; 2];
    let mut onset_buoyant = [[0u64; 5]; 2];
    let mut by_pair: BTreeMap<(u32, u32, u8, bool), (u64, f64)> = BTreeMap::new();
    for r in &recs {
        let h = u64::from(r.half);
        let b = band_of(r.v_n, &NORMAL_BANDS_MM_YR);
        let e = by_pair
            .entry((r.under, r.over, r.setting as u8, r.subducting))
            .or_default();
        e.0 += h;
        e.1 = r.slab_km;
        if !r.subducting {
            incipient_normal[r.setting][b] += h;
            if r.slow {
                slow[r.setting] += h;
            }
            continue;
        }
        let (sum, w) = pair_mean[&(r.under, r.over)];
        let mean = sum / w.max(1) as f64;
        normal_pair[r.setting][b][band_of(mean, &NORMAL_BANDS_MM_YR)] += h;
        if r.setting == 0 {
            oo_normal_pair[r.pair][b] += h;
        }
        // Measurement only (the intra-oceanic fronts diagnosis).
        if !r.arc_age.is_nan() {
            arc_age[r.setting][band_of(r.arc_age, &AGE_BANDS_MYR)] += h;
        }
        if r.setting == 0 {
            let c = band_of(r.contact_age, &AGE_BANDS_MYR);
            oo_over_floor[c][band_of(r.over_floor_age, &AGE_BANDS_MYR)] += h;
        }
        if !r.onset_slab_age.is_nan() {
            onset_slab[r.setting][band_of(r.onset_slab_age, &AGE_BANDS_MYR)] += h;
        }
        onset_kind[r.setting][usize::from(r.onset[0])] += h;
        onset_route[r.setting][usize::from(r.onset[1])] += h;
        onset_buoyant[r.setting][usize::from(r.onset[2])] += h;
    }
    let pairs = by_pair
        .into_iter()
        .map(
            |((under, over, setting, subducting), (units, slab_km))| PairFront {
                under,
                over,
                setting,
                subducting,
                units,
                slab_km,
            },
        )
        .collect();
    let pair_bounds = units
        .pair_bound
        .into_iter()
        .map(|((lo, hi), a)| {
            let mean = |sum: u64, n: u64| {
                if n == 0 {
                    -1.0
                } else {
                    sum as f64 / 100.0 / n as f64
                }
            };
            PairBound {
                lo,
                hi,
                class: a.class,
                normal_mm_yr: if a.normal_n == 0 {
                    0.0
                } else {
                    a.normal_um_yr as f64 / 1000.0 / a.normal_n as f64
                },
                ocean: a.ocean.map(|x| x as u32),
                age_myr: [
                    mean(a.age_centi[0], a.ocean[0]),
                    mean(a.age_centi[1], a.ocean[1]),
                ],
                cont: a.cont.map(|x| x as u32),
            }
        })
        .collect();
    Mix {
        km: units.km.map(km),
        pairs,
        pair_bounds,
        onset_prov: onset_prov.into_values().collect(),
        bands: FrontBands {
            slab: bands(units.slab),
            normal: bands(units.normal),
            age: bands(units.age),
            other_converging: km(units.other_converging),
            oo_over_cont: units.oo[0].map(km),
            oo_under_cont: units.oo[1].map(km),
            oo_coast: units.oo[2].map(km),
            oo_pair: units.oo_pair.map(km),
            incipient: units.incipient.map(km),
            benioff: units.benioff.map(km),
            slow: slow.map(km),
            incipient_normal: bands(incipient_normal),
            normal_pair: normal_pair.map(|m| m.map(|row| row.map(km))),
            oo_normal_pair: oo_normal_pair.map(|row| row.map(km)),
            arc_age: bands(arc_age),
            onset_slab: bands(onset_slab),
            oo_over_floor: oo_over_floor.map(|row| row.map(km)),
            onset_kind: onset_kind.map(|row| row.map(km)),
            onset_route: onset_route.map(|row| row.map(km)),
            onset_buoyant: onset_buoyant.map(|row| row.map(km)),
            ..FrontBands::default()
        },
    }
}

/// The elevations of the arc cells of one kind in one step, m above the sea the step stood under:
/// how many, how many stand above it, and the median, 90th percentile and highest. Serialised as
/// the array `[cells, emergent, p50, p90, max]`.
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

    /// Share of the cells above the sea (0 of none).
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

/// The oceanic and the continental arcs' elevations above the sea at `sea_m` ([`ArcStat`]) among
/// `samples`.
pub fn arc_stats(samples: &[Sample], sea_m: f64) -> [ArcStat; 2] {
    let (mut oc, mut cont) = (Vec::new(), Vec::new());
    for s in samples {
        if s.is_hidden() || s.flags & ARC == 0 {
            continue;
        }
        let z = f64::from(s.elevation_m) - sea_m;
        if is_island_arc(s) {
            oc.push(z);
        } else {
            cont.push(z);
        }
    }
    [ArcStat::of(oc), ArcStat::of(cont)]
}

/// The arcs of the step above its sea ([`arc_stats`]); empty when the record is off.
pub fn arc_heights(st: &PlateState, sea_m: f64) -> [ArcStat; 2] {
    if st.dynamics.is_none() {
        return [ArcStat::default(); 2];
    }
    arc_stats(&st.samples, sea_m)
}

/// What a step measured before `record_step` sees its end state: the fronts' lengths, the boundary
/// mix and the arc heights, taken where the flags are fresh ([`boundary_mix`], [`arc_heights`]),
/// and the force balance (`forces::step`).
#[derive(Clone, Debug, Default)]
pub struct Observed<'a> {
    /// Each plate's subduction front length this step (`subduction::Pull::front_km`).
    pub front_km: &'a [f64],
    pub mix: Mix,
    /// Oceanic, continental.
    pub arcs: [ArcStat; 2],
    /// The force balance (`None` when it was not taken).
    pub forces: Option<&'a super::forces::Recorded>,
    /// The plates the speed guard held this step (`kinematics::guard_speed`).
    pub guarded: &'a [u32],
    /// The sea the step stood under, m (the land is the ground above it).
    pub sea_m: f64,
    /// The live plates the step's solved balance left out (`forces::Answer::omega` `None`: no
    /// visible sample), which keep their rotation.
    pub out_of_balance: &'a [u32],
}

/// What [`record_step`] counts over the samples (P1-19: per run of samples, then merged).
struct Tally {
    nearest: NearestPlate,
    vis: Vec<u32>,
    cont: Vec<u32>,
    bnd: Vec<u32>,
    visible: u32,
    land: u32,
    land_held: u32,
    continental: u32,
}

/// A recorded onset's key, which a contact and a split's copies of it keep: its step, its slab
/// age and its convergence at the onset (the slab age alone is 0 for every continental onset, so
/// two of one step shared a record; PR #55's review).
fn onset_key(step: u64, slab_age_myr: f64, convergence_mm_yr: f64) -> (u64, u64, u64) {
    (step, slab_age_myr.to_bits(), convergence_mm_yr.to_bits())
}

/// Measurement only (the onset trace, 2026-10-31): records each contact whose onset is this step
/// ([`OnsetRec`]) with its pair's boundary the step before, then updates the pairs' history and
/// keeps this step's boundaries and contacts for the next.
fn record_onsets(
    d: &mut Dynamics,
    st: &PlateState,
    env: &Env<'_>,
    bounds: &[PairBound],
    fronts: &[PairFront],
    prov: &[OnsetProv],
) {
    let t = env.t_myr;
    let since_resample = d.resamples.last().map_or(-1.0, |r| t - r.t);
    let prov_of: BTreeMap<(u32, u32), &OnsetProv> =
        prov.iter().map(|p| ((p.under, p.over), p)).collect();
    let lives: BTreeMap<u32, (Option<u32>, &str)> = d
        .plates
        .iter()
        .map(|p| (p.id, (p.parent, p.origin.as_str())))
        .collect();
    let prev: BTreeMap<(u32, u32), &PairBound> = d
        .prev_pair_bound
        .iter()
        .map(|b| ((b.lo, b.hi), b))
        .collect();
    let hist: BTreeMap<(u32, u32), &PairHist> =
        d.pair_hist.iter().map(|h| ((h.lo, h.hi), h)).collect();
    // A reversal is what `resolve` reversed this step, a dormant contact's too (PR #55's
    // re-review; before, the old polarity had to be active the step before).
    let reversed: BTreeSet<(u32, u32)> =
        std::mem::take(&mut d.onset_reversed).into_iter().collect();
    let overlap: BTreeMap<(u32, u32), OnsetOverlap> = std::mem::take(&mut d.onset_overlap)
        .into_iter()
        .map(|o| ((o.lo, o.hi), o))
        .collect();
    let crossed: BTreeMap<(u32, u32), (u32, f64)> = std::mem::take(&mut d.onset_crossed)
        .into_iter()
        .map(|(key, steps, km)| (key, (steps, km)))
        .collect();
    let mut new = Vec::new();
    for c in st.contacts.iter().filter(|c| c.onset_step == env.step) {
        let key = (c.under.min(c.over), c.under.max(c.over));
        let before = prev.get(&key).map(|b| (*b).clone()).unwrap_or(PairBound {
            lo: key.0,
            hi: key.1,
            age_myr: [-1.0, -1.0],
            ..PairBound::default()
        });
        let h = hist.get(&key);
        let route = if reversed.contains(&key) {
            1
        } else if pair_relation(&lives, c.under, c.over) == PAIR_INITIATION {
            2
        } else {
            0
        };
        new.push(OnsetRec {
            t,
            step: env.step,
            under: c.under,
            over: c.over,
            setting: c.setting,
            route,
            kind: before.kind(),
            boundary_age_myr: h.map_or(-1.0, |h| t - h.first_myr),
            converging_myr: h
                .and_then(|h| h.converging_since_myr)
                .map_or(0.0, |s| t - s),
            since_spread_myr: h.and_then(|h| h.spread_last_myr).map_or(-1.0, |s| t - s),
            slab_age_myr: c.onset_slab_age_myr,
            convergence_mm_yr: c.onset_convergence_mm_yr,
            before,
            since_resample_myr: since_resample,
            prov: prov_of
                .get(&(c.under, c.over))
                .map(|p| (*p).clone())
                .unwrap_or_default(),
            subducting_after_myr: -1.0,
            max_slab_km: 0.0,
            front_km_myr: [0.0; 2],
            overlap: overlap.get(&key).cloned().unwrap_or_default(),
            last_myr: t,
            cross_steps: crossed.get(&key).map_or(0, |c| c.0),
            cross_km: crossed.get(&key).map_or(0.0, |c| c.1),
        });
    }
    d.onsets.extend(new);
    // What became of each onset's contact and a split's copies of it (by `onset_key`, which the
    // copies keep), this step.
    let mut by_onset: BTreeMap<(u64, u64, u64), Vec<usize>> = BTreeMap::new();
    for (i, o) in d.onsets.iter().enumerate() {
        by_onset
            .entry(onset_key(o.step, o.slab_age_myr, o.convergence_mm_yr))
            .or_default()
            .push(i);
    }
    let slab_of: BTreeMap<(u32, u32), f64> =
        st.slabs.iter().map(|s| ((s.under, s.over), s.km)).collect();
    let spacing_km = env.norm.km(env.norm.spacing_rad());
    let mut contact_onset: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for c in st.contacts.iter().filter(|c| c.active(env.step)) {
        let Some(list) = by_onset.get(&onset_key(
            c.onset_step,
            c.onset_slab_age_myr,
            c.onset_convergence_mm_yr,
        )) else {
            continue;
        };
        let i = list
            .iter()
            .copied()
            .find(|&i| d.onsets[i].under == c.under || d.onsets[i].over == c.over)
            .unwrap_or(list[0]);
        contact_onset.insert((c.under, c.over), i);
        let o = &mut d.onsets[i];
        o.last_myr = t;
        let km = slab_of.get(&(c.under, c.over)).copied().unwrap_or(0.0);
        o.max_slab_km = o.max_slab_km.max(km);
    }
    for f in fronts.iter().filter(|f| f.subducting) {
        if let Some(&i) = contact_onset.get(&(f.under, f.over)) {
            let o = &mut d.onsets[i];
            o.front_km_myr[usize::from(f.setting.min(1))] +=
                f.units as f64 * 0.5 * spacing_km * env.dt_myr;
            if o.subducting_after_myr < 0.0 {
                o.subducting_after_myr = t - o.t;
            }
        }
    }
    let mut next: BTreeMap<(u32, u32), PairHist> =
        d.pair_hist.drain(..).map(|h| ((h.lo, h.hi), h)).collect();
    let now: BTreeSet<(u32, u32)> = bounds.iter().map(|b| (b.lo, b.hi)).collect();
    for (key, h) in &mut next {
        if !now.contains(key) {
            h.converging_since_myr = None;
        }
    }
    for b in bounds {
        let h = next.entry((b.lo, b.hi)).or_insert(PairHist {
            lo: b.lo,
            hi: b.hi,
            first_myr: t,
            ..PairHist::default()
        });
        if b.normal_mm_yr < 0.0 {
            h.converging_since_myr.get_or_insert(t);
        } else {
            h.converging_since_myr = None;
        }
        if b.class[mix::RIDGE] + b.class[mix::RIFT] > 0 {
            h.spread_last_myr = Some(t);
        }
    }
    d.pair_hist = next.into_values().collect();
    d.prev_pair_bound = bounds.to_vec();
    d.prev_contacts = st
        .contacts
        .iter()
        .filter(|c| c.active(env.step))
        .map(|c| (c.under, c.over))
        .collect();
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
    // P1-19: the counts and the poles' nearest samples are taken in parallel over runs of samples
    // (counts add up the same in any order; the runs' nearest samples are merged in sample
    // order), with the continental pieces and the sea floor's ages beside them.
    let (samples, groups) = (&st.samples, &st.groups);
    let plate_of = |i: usize| groups[samples[i].group as usize].plate;
    let sea = seen.sea_m;
    // The sea of the step before, for the land with the sea held (`StepRec::land_held`).
    let held = d.steps.last().map(|s| s.sea_m);
    let tally = || Tally {
        nearest: NearestPlate::new(poles.clone()),
        vis: vec![0; np],
        cont: vec![0; np],
        bnd: vec![0; np],
        visible: 0,
        land: 0,
        land_held: 0,
        continental: 0,
    };
    let count = || {
        (0..samples.len())
            .into_par_iter()
            .with_min_len(4096)
            .fold(tally, |mut t, i| {
                let s = &samples[i];
                if s.is_hidden() {
                    return t;
                }
                let p = plate_of(i);
                t.nearest.offer(pos[i], p);
                t.vis[p as usize] += 1;
                t.visible += 1;
                if s.kind == CONTINENTAL {
                    t.cont[p as usize] += 1;
                    t.continental += 1;
                }
                let z = f64::from(s.elevation_m);
                t.land += u32::from(z > sea);
                t.land_held += u32::from(held.is_some_and(|h| z > h));
                // On the plate's boundary: a neighbour of another plate, or one hidden under
                // another plate (the plate's own subducted crust: the trench side of a slab).
                let edge = geo
                    .edges(i as u32)
                    .any(|(u, _)| samples[u as usize].is_hidden() || plate_of(u as usize) != p);
                if edge {
                    t.bnd[p as usize] += 1;
                }
                t
            })
            .reduce(tally, |mut a, b| {
                a.nearest.merge(&b.nearest);
                for (x, y) in [
                    (&mut a.vis, &b.vis),
                    (&mut a.cont, &b.cont),
                    (&mut a.bnd, &b.bnd),
                ] {
                    x.iter_mut().zip(y).for_each(|(x, y)| *x += y);
                }
                a.visible += b.visible;
                a.land += b.land;
                a.land_held += b.land_held;
                a.continental += b.continental;
                a
            })
    };
    // Connected pieces of continental crust: continental samples joined along lattice edges
    // whose ends are still close (a rift since the last resampling pulls them apart). The
    // membership and each member's close edges (a bit per edge, the first 32) are found in
    // parallel first; the walk then reads them.
    let link = (PIECE_LINK_SPACINGS * env.norm.spacing_rad()).cos();
    let pieces = || {
        let member: Vec<bool> = samples
            .par_iter()
            .map(|s| !s.is_hidden() && s.kind == CONTINENTAL)
            .collect();
        let close: Vec<u32> = (0..samples.len())
            .into_par_iter()
            .map(|i| {
                if !member[i] {
                    return 0;
                }
                geo.edges(i as u32)
                    .take(32)
                    .enumerate()
                    .filter(|&(_, (u, _))| pos[i].dot(pos[u as usize]) >= link)
                    .fold(0u32, |m, (k, _)| m | (1 << k))
            })
            .collect();
        let mut pieces = piece_sizes(
            samples.len(),
            |i| member[i],
            |i, out| {
                out.extend(
                    geo.edges(i as u32)
                        .enumerate()
                        .filter(|&(k, (u, _))| {
                            if k < 32 {
                                close[i] & (1 << k) != 0
                            } else {
                                pos[i].dot(pos[u as usize]) >= link
                            }
                        })
                        .map(|(_, (u, _))| u),
                );
            },
        );
        pieces.truncate(PIECES_KEPT);
        pieces
    };
    // The sea floor's age, every step.
    let (tally, (pieces, ages)) =
        rayon::join(count, || rayon::join(pieces, || ocean_ages(st, env.t_myr)));
    let Tally {
        nearest,
        vis,
        cont,
        bnd,
        visible,
        land,
        land_held,
        continental,
    } = tally;
    let spacing_km = env.norm.km(env.norm.spacing_rad());
    let round = |x: f64, scale: f64| (x * scale).round() / scale;
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
                guard: seen.guarded.contains(&p.id),
                spinning: w.length() > 0.0
                    && w.normalize().dot(DVec3::from_array(p.centroid)).abs() >= spin_cos,
            }
        })
        .collect();
    d.prev_omega = st.alive().map(|p| (p.id, p.omega)).collect();

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
    // The front's turnover since the step before, by pair of plates, setting and state
    // (measurement only, PLT-683).
    let mut front_bands = seen.mix.bands;
    let now: Vec<(u32, u32, u8, u8, u64)> = seen
        .mix
        .pairs
        .iter()
        .map(|f| (f.under, f.over, f.setting, u8::from(f.subducting), f.units))
        .collect();
    if !d.steps.is_empty() {
        let units_km = |u: u64| (u as f64 * 0.5 * spacing_km).round();
        let slab_of: BTreeMap<(u32, u32, u8, u8), f64> = seen
            .mix
            .pairs
            .iter()
            .map(|f| {
                (
                    (f.under, f.over, f.setting, u8::from(f.subducting)),
                    f.slab_km,
                )
            })
            .collect();
        let mut change: BTreeMap<(u32, u32, u8, u8), i64> = BTreeMap::new();
        for &(u, o, s, m, n) in &now {
            *change.entry((u, o, s, m)).or_default() += n as i64;
        }
        for &(u, o, s, m, n) in &d.prev_pair_front {
            *change.entry((u, o, s, m)).or_default() -= n as i64;
        }
        let (mut gain, mut loss) = ([[0u64; 2]; 2], [[0u64; 2]; 2]);
        let mut gain_slab = [[0u64; BANDS]; 2];
        for (&(u, o, s, m), &dn) in &change {
            let (s, m) = (usize::from(s.min(1)), usize::from(m.min(1)));
            if dn > 0 {
                gain[s][m] += dn as u64;
                if m == 1 {
                    let km = slab_of.get(&(u, o, s as u8, 1)).copied().unwrap_or(0.0);
                    gain_slab[s][band_of(km, &SLAB_BANDS_KM)] += dn as u64;
                }
            } else {
                loss[s][m] += dn.unsigned_abs();
            }
        }
        front_bands.gain = gain.map(|r| r.map(units_km));
        front_bands.loss = loss.map(|r| r.map(units_km));
        front_bands.gain_slab = gain_slab.map(|r| r.map(units_km));
    }
    d.prev_pair_front = now;
    record_onsets(
        &mut d,
        st,
        env,
        &seen.mix.pair_bounds,
        &seen.mix.pairs,
        &seen.mix.onset_prov,
    );
    if last {
        d.prov.clear();
    }
    d.steps.push(StepRec {
        t,
        visible,
        land,
        land_held: held.map(|_| land_held),
        sea_m: sea,
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
        mix_km: seen.mix.km,
        front_bands,
        arc_oceanic: seen.arcs[0],
        arc_continental: seen.arcs[1],
        forces: seen.forces.map(|f| f.forces.clone()).unwrap_or_default(),
        slabs_km: seen.forces.map(|f| f.slabs_km.clone()).unwrap_or_default(),
        breakoffs: seen.forces.map_or(0, |f| f.breakoffs),
        matured: seen.forces.map(|f| f.matured.clone()).unwrap_or_default(),
        shown: super::shown::take_step(st),
        out_of_balance: seen.out_of_balance.to_vec(),
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

    /// The onset trace's classes (measurement only): a pair's boundary by its longest class, the
    /// "other" boundary split by the velocity across it, and the floors against 10 Myr.
    #[test]
    fn onset_trace_classes_a_boundary_and_its_floors() {
        let mut b = PairBound {
            lo: 1,
            hi: 2,
            ..PairBound::default()
        };
        assert_eq!(b.kind(), 5);
        b.class[mix::RIDGE] = 4;
        b.class[mix::OTHER] = 3;
        assert_eq!(b.kind(), 0);
        b.class[mix::OTHER] = 9;
        b.normal_mm_yr = -12.0;
        assert_eq!(b.kind(), 1);
        b.normal_mm_yr = 0.0;
        assert_eq!(b.kind(), 2);
        b.class[mix::OCEAN_OCEAN] = 20;
        assert_eq!(b.kind(), 3);
        b.ocean = [5, 7];
        b.age_myr = [4.0, 30.0];
        let o = |b: &PairBound, under, over| OnsetRec {
            under,
            over,
            before: b.clone(),
            ..OnsetRec::default()
        };
        // Plate 1 (lo) has 4 Myr floor at the boundary, plate 2 30 Myr.
        assert_eq!(o(&b, 1, 2).buoyant(), 1);
        assert_eq!(o(&b, 2, 1).buoyant(), 2);
        b.age_myr = [4.0, 8.0];
        assert_eq!(o(&b, 2, 1).buoyant(), 0);
        b.age_myr = [40.0, 80.0];
        assert_eq!(o(&b, 2, 1).buoyant(), 3);
        b.ocean = [5, 0];
        assert_eq!(o(&b, 2, 1).buoyant(), 4);
    }

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
        Neighbour {
            along,
            normal: along,
            kind,
            side,
        }
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
        // A transform judged across its boundary: the zigzag's edges converge and part, the
        // boundary's normal does neither; on an active contact it is still no front.
        let slide = |along, side| Neighbour {
            along,
            normal: 0.0,
            kind: OCEANIC,
            side,
        };
        let zigzag = [slide(-20.0, Side::Over), slide(20.0, Side::Over)];
        assert_eq!(classify_boundary(OCEANIC, &zigzag), Some((OTHER, 1)));
        let under = [slide(-20.0, Side::Under), slide(20.0, Side::None)];
        assert_eq!(classify_boundary(OCEANIC, &under), Some((OTHER, 1)));
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
        let [oc, cont] = arc_stats(&cells, 0.0);
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
        let none = arc_stats(&[arc_cell(OCEANIC, 5.0, 0.0, 0.0, 0)], 0.0);
        assert_eq!(none[0], ArcStat::default());
        assert_eq!(none[0].emergent_share(), 0.0);
        // A sea-level cell is not emergent.
        assert_eq!(ArcStat::of(vec![0.0, 0.0, 1.0]).emergent, 1);
        // Under a sea 450 m above the datum the heights are the sea's: the cells at 50 and 400 m
        // are under water, and every height is 450 m lower.
        let [high_sea, _] = arc_stats(&cells, 450.0);
        assert_eq!((high_sea.cells, high_sea.emergent), (14, 2));
        assert_eq!(high_sea.max_m, 750.0);
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

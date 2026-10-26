// SPDX-License-Identifier: LGPL-3.0-or-later
//! The force balance of the plates (docs/design/plate-force-balance.md, §3; its tasks 5 and 7):
//! every step, the driving and resisting forces on each plate, their torques, and the rotation at
//! which the torques balance. **This is what moves the plates** (since the switch, task 7): each
//! plate's rotation relaxes towards the answer over `motion_relax_myr` ([`relax`]; the plates step,
//! `plates/mod.rs`), under a speed guard that is reported whenever it binds. With the dynamics
//! record on, the budget of every plate also goes into the record (`dynamics.rs`,
//! `StepRec::forces`).
//!
//! # The balance
//! Mantle flow is viscous, so a plate has no inertia: the torques on it sum to zero (Forsyth &
//! Uyeda 1975). With Ω_p each plate's angular velocity, every resisting force here is linear in
//! the velocities (the plastic ones linearised about this step's motion, below), so the balance
//! is one linear system over all plates,
//!
//! ```text
//!     M Ω = T,
//! ```
//!
//! `T` the torques of the forces that do not depend on speed, `M` symmetric and positive definite
//! (basal drag makes each diagonal block definite; boundary and collision resistance couple
//! neighbours by blocks of the form [K −K; −K K]). It is solved by Cholesky factorisation: 3
//! unknowns per plate, at most a few hundred in all.
//!
//! # The terms (SI on the plate model's Earth-normalized sphere of radius R)
//! A force `f` (N) at the unit position `x` has the torque `R x × f`. Lengths are one sample
//! spacing per boundary sample (as the record's `boundary_km`), areas one sample's area, except
//! at a subduction front with `front_pull` (PLT-560): there the front is the arcs'
//! (`subduction::fronts_begin`: the plates converge across the boundary, beyond Bird's 20 degrees
//! of a transform), and the pull, the suction, the slab resistance and a collision's resistance
//! act along the boundary's normal over each sample's share of the straight front
//! (`shortening::front_share_rad`), the per-metre sizes' own count (a straight trench, the pull
//! normal to it: [`Coeffs::new`]). Without it every lattice edge converging on an active contact
//! was a front, each overriding sample pulling one spacing along that edge. With
//! `front_current_geometry` (PLT-582) a front, a hanging slab and a collision's incoming side
//! count a plate beside a sample only while a visible sample of that plate stands within
//! `subduction::trench_row_rad` of it now, as the arcs' fronts and the boundary mix do: between
//! resamplings the lattice is frozen, and the lattice neighbours a consumed plate had at the last
//! resampling have gone under or been carried on. Until 2026-10-26 the force balance read them,
//! so a consumed plate kept pulling along its trench as it stood at the last resampling while
//! its basal drag shrank with its area (plates.md, "The force balance's front on the current
//! geometry").
//! - **Slab pull** (drives the subducting plate, towards the trench): per metre of trench where
//!   the subducting crust is oceanic, `F_sp · slab_pull · s(age) · min(L, D)/D`, with `F_sp` the
//!   reference pull ([`SLAB_PULL_N_M`] at Earth's gravity, scaled with gravity), `s` the slab's
//!   thickness factor ([`slab_weight`]), `L` the contact's tracked slab length and `D`
//!   `slab_depth_km`. Continental crust in the trench pulls nothing.
//! - **Slab suction** (drives the overriding plate, towards the trench): `slab_suction_share` of
//!   that pull (Forsyth & Uyeda's 0.50 / 6.43 with `forsyth_uyeda_balance`, whatever the
//!   speed). With `suction_saturates` (PLT-540) and without that it is a traction towards the
//!   flow the sinking slab drives, `k (v_t − u)`: `u` the overriding plate's speed towards the
//!   trench, `v_t` the slab's terminal velocity (its pull over its slab resistance, Forsyth &
//!   Uyeda 1975), `k v_t` the share of the pull. At rest it is the share; it falls to nothing as the plate catches up
//!   with the slab, so no overriding plate is drawn faster than the slab sinks.
//! - **Edge energy** (ridge push and the weight of high ground): at every boundary sample of a
//!   plate, `(U(edge) − Ū) · edge_energy / EDGE_ENERGY_PHYSICAL` per metre, pushing the plate
//!   away from the edge when the edge stands high ([`column_energy`]: the gravitational
//!   potential energy of the column; `Ū` the plate's mean). On a rigid plate a constant `U`
//!   gives no torque, so only the differences matter; `Ū` keeps the sum's discretisation small.
//!   At a collision front the belt's weight acts as the collision resistance instead (below).
//! - **Basal drag** (resists): per m², `c_b (v − v_flow)`, `keel_drag` times more under
//!   continental crust. `c_b` is set by `plate_speed_scale_mm_yr` (see [`Coeffs::new`]); with
//!   `forsyth_uyeda_balance` (PLT-585, off by default) it and every other resistance's size are
//!   Forsyth & Uyeda's (1975, Table 6) against the reference pull, the continental factor theirs
//!   too, and the speed scale is not read.
//! - **Mantle flow** (drives, weakly): the drag of a slow degree-2 flow ([`flow_velocity`]) of
//!   peak `mantle_flow_mm_yr`, away from two antipodal upwellings: a stand-in for the
//!   convection the model does not have (Ben, 2026-10-02: weak mantle flow as a setting).
//! - **Slab resistance** (resists): per metre of trench, against the subducting plate's speed
//!   towards the trench (Forsyth & Uyeda's resistance to a slab's sinking).
//! - **Boundary resistance** (resists; only without `forsyth_uyeda_balance`): per metre of any
//!   plate boundary, against the relative velocity of the two plates (across and along), half
//!   from each side.
//! - **Transform and colliding resistance** (resist; with `forsyth_uyeda_balance`): Forsyth &
//!   Uyeda's friction, a force per metre whatever the speed, against the plates' relative motion,
//!   on both alike ([`Balance::add_friction`]): the transform resistance where the boundary's
//!   relative motion lies more than their 60 degrees from its normal ([`fu_transform`]), the
//!   colliding resistance where the plates converge across it otherwise (trenches and collision
//!   fronts both, as theirs), nothing at a ridge. A front's is counted once, at the front over
//!   its metres as the pull and the suction are; the rest of the boundary from both sides, half
//!   each. Linearised as the collision's resistance is (below).
//! - **Collision resistance** (resists): along a collision front (continental crust at the
//!   subducting side), against the convergence, with the force of the belt's edge energy
//!   `max(0, U(belt) − U(incoming))` plus the buoyancy of the continental crust the collision has
//!   dragged down (`collision_buoyancy` times [`collision_buoyancy_n_m`]: it grows with the
//!   collision's shortening, so a collision that nothing pulls on hard enough stalls). It is plastic in the design (a force whatever the speed),
//!   put into the linear solve as a drag whose coefficient is that force over this step's
//!   convergence, bounded below by [`COLLISION_FLOOR_MM_YR`] (design §3.4).
//!
//! # The mean in-plane stress
//! The mean of a plate's in-plane stress tensor is its edge forces' first moment over its area
//! (with the basal drag as a body force): the isotropic part is
//! `σ = (Σ r·f + Σ r·b dA) / (2 A h)`, `r` a point's offset from the plate's centroid in the
//! centroid's tangent plane, `h` [`LITHOSPHERE_KM`]. Positive is tension (edges pulled
//! outwards). Flat-plate geometry: for a plate covering a large part of the sphere it is an
//! index, not a stress field.
//!
//! # Slabs
//! A subduction zone keeps one slab length ([`SlabTrack`], in `PlateState::slabs`), known by its
//! two plates: it grows by the convergence at the contact's oceanic front each step (along the
//! boundary's normal with `front_pull`), up to
//! `slab_depth_km`, and breaks off (length 0) when continental crust has held at least half of
//! the contact's front for `slab_breakoff_myr` (design §3.3, option 5.3a). A contact that flickers
//! off and on keeps its slab (task 5 found the restarts to be the main source of noise); a
//! reversed contact starts a new one. A slab no oceanic front feeds this step hangs: it pulls at
//! the subducting plate's samples beside the overriding plate (with suction and slab resistance
//! as at a front) and is gone after `slab_breakoff_myr` without a front. A subduction initiation
//! plants a slab of `initiation_slab_km` (`rifting.rs`), which pulls the new plate towards its
//! parent until the trench forms. A copy of a slab that a split handed to a piece of a plate
//! (`inherit.rs`, `SlabTrack::latent`) pulls only at that piece's own fronts until its first one:
//! the pull of a split plate's pieces is then the parent's, front edge by front edge, and a piece
//! that borders the other plate across a ridge or a transform hangs none of it.

use std::collections::{BTreeMap, BTreeSet};

use glam::{DMat3, DVec3};
use rayon::prelude::*;

use super::Env;
use super::dynamics::PlateForces;
use super::geometry::{Geo, Tree};
use super::kinematics::{quat, tangent};
use super::normalise::Norm;
use super::state::{CONTINENTAL, PlateState, SlabTrack};
use super::{shortening, subduction};
use crate::params::TectonicParams;
use crate::sphere::SplitMix64;

/// World globals (measurement only): the oceanic fronts the slab pull acts along over the run,
/// km × Myr (Earth-normalized): one lattice spacing per overriding sample that pulls, as the pull
/// counts them; the part of each along the boundary's normal; and the samples' shares of the
/// straight front (PLT-530). ledger: PLT-554
pub const SLAB_FRONT_LATTICE_GLOBAL: &str = "plates/slab_front_lattice_km_myr";
pub const SLAB_FRONT_NORMAL_GLOBAL: &str = "plates/slab_front_normal_km_myr";
pub const SLAB_FRONT_STRAIGHT_GLOBAL: &str = "plates/slab_front_straight_km_myr";

/// Seconds in a year (Julian): the balance is solved in SI.
pub const YEAR_S: f64 = 3.155_76e7;
/// Earth's gravity, m/s²: the reference forces are Earth's and scale with the planet's gravity.
pub const G_EARTH: f64 = 9.81; // ledger: FRC-001
/// The reference slab pull, N per metre of trench, at Earth's gravity: a full-length slab of
/// 80-Myr floor. "About 1–3 × 10¹³ N/m gross" (design §3.2; textbook figure, not confirmed).
pub const SLAB_PULL_N_M: f64 = 3.0e13; // ledger: FRC-002
/// The reference trench of `plate_speed_scale_mm_yr`, km (the old speed law's full-speed front).
pub const REF_TRENCH_KM: f64 = 3000.0; // ledger: FRC-003
/// Slab thickness factor: √(age / this), so a slab of this age pulls with weight 1, Myr …
pub const SLAB_AGE_REF_MYR: f64 = 80.0; // ledger: FRC-004
/// … growing no more beyond this age, Myr (the factor's ceiling, 1.5) …
pub const SLAB_AGE_CAP_MYR: f64 = 180.0; // ledger: FRC-004
/// … and zero for crust younger than this, which is not yet denser than the mantle, Myr.
pub const SLAB_BUOYANT_MYR: f64 = 10.0; // ledger: FRC-005
/// Densities of the crust, the mantle and sea water, kg/m³, and the reference crustal thickness,
/// km, of the land column's gravitational potential energy.
pub const RHO_CRUST: f64 = 2750.0; // ledger: FRC-006
pub const RHO_MANTLE: f64 = 3300.0; // ledger: FRC-006
pub const RHO_WATER: f64 = 1000.0; // ledger: FRC-006
pub const CRUST_KM: f64 = 35.0; // ledger: FRC-007
/// Thermal expansion (1/K) and the temperature drop across the oceanic lithosphere (K), of the
/// sea floor's potential energy (half-space cooling).
pub const ALPHA_PER_K: f64 = 3.0e-5; // ledger: FRC-008
pub const DELTA_T_K: f64 = 1300.0; // ledger: FRC-008
/// The `edge_energy` at which the edge term is the potential energy as computed (a ridge's push
/// is then about 0.1 of the reference slab pull at the pipeline presets' ocean depths).
pub const EDGE_ENERGY_PHYSICAL: f64 = 0.1; // ledger: FRC-009
/// The thickness over which edge forces are spread to give a mean stress, km.
pub const LITHOSPHERE_KM: f64 = 100.0; // ledger: FRC-010
/// The collision drag's convergence floor, mm/yr: the plastic resistance enters the linear solve as
/// its force over max(convergence, floor). A collision whose resistance exceeds what drives it then
/// converges at no more than floor × drive / resistance, so the floor sits well under
/// `weld_speed_mm_yr` (5): with the floor at the weld speed (task 5's choice) a held collision
/// converges just under the weld speed at best, and welds were rare (plate task 7, 2026-10-05).
pub const COLLISION_FLOOR_MM_YR: f64 = 1.0; // ledger: FRC-011
/// Cloos 1993 (abstract): 80-Myr oceanic lithosphere is about 0.04 g/cm³ denser than the
/// asthenosphere, and 100 km of lithosphere with a 30 km granitic crust "resists subduction
/// because it is ∼0.09 gm/cm3 less dense": the buoyancy of continental lithosphere dragged down a
/// trench is this many times the negative buoyancy of the same length of 80-Myr slab.
pub const CONTINENT_BUOYANCY_RATIO: f64 = 0.09 / 0.04; // ledger: FRC-028
/// … for the crust thickness of that figure, km (thicker crust is more buoyant, in proportion).
pub const CONTINENT_BUOYANCY_CRUST_KM: f64 = 30.0; // ledger: FRC-029
/// Forsyth & Uyeda 1975, Table 6 (p. 190; their model X_s): the forces' relative strengths, one
/// scale for all. Per km of boundary: the slab pull, the suction, the transform resistance and
/// the colliding resistance (velocity-independent; the last two against the relative motion,
/// pp. 169-170); per km of trench per cm/yr of the plate's speed towards it: the slab resistance;
/// per km² per cm/yr of the plate's speed over the mantle: the mantle drag and the continental
/// drag beyond it. ledger: FRC-033
pub const FU_SLAB_PULL: f64 = 6.43;
pub const FU_SUCTION: f64 = 0.50; // ledger: FRC-033
pub const FU_TRANSFORM: f64 = 0.36; // ledger: FRC-033
pub const FU_COLLIDING: f64 = 0.16; // ledger: FRC-033
pub const FU_SLAB_RESISTANCE: f64 = 0.89; // ledger: FRC-033
pub const FU_MANTLE_DRAG: f64 = 0.82e-5; // ledger: FRC-033
pub const FU_CONTINENTAL_DRAG: f64 = 5.65e-5; // ledger: FRC-033
/// Forsyth & Uyeda's dividing line between a convergent boundary and a transform fault, the
/// rule their strengths were fitted with (1975, p. 183): a boundary segment whose relative
/// motion lies more than this from the boundary's normal (φ = tan⁻¹(V_PAR / V_PER)) is a
/// transform, so within 30 degrees of strike-slip, where V1 gap 2's boundary types take Bird's
/// 20 (PLT-311). ledger: FRC-035
pub const FU_TRANSFORM_PHI_DEG: f64 = 60.0;

/// Whether a boundary segment is a transform fault for Forsyth & Uyeda's resistances: the
/// relative motion `rel` more than [`FU_TRANSFORM_PHI_DEG`] from the boundary's normal `normal`
/// (a unit vector; either sense). No motion is no transform. ledger: FRC-035
pub fn fu_transform(rel: DVec3, normal: DVec3) -> bool {
    rel.dot(normal).abs() < FU_TRANSFORM_PHI_DEG.to_radians().cos() * rel.length()
}
/// Torques are recorded in this unit, N m.
pub const TORQUE_UNIT_N_M: f64 = 1.0e24;

/// The slab thickness factor of oceanic crust of `age_myr` at the trench (ledger: FRC-004,
/// FRC-005): √(age / 80 Myr), at most 1.5, and 0 below 10 Myr.
pub fn slab_weight(age_myr: f64) -> f64 {
    if age_myr < SLAB_BUOYANT_MYR {
        0.0
    } else {
        (age_myr.min(SLAB_AGE_CAP_MYR) / SLAB_AGE_REF_MYR).sqrt()
    }
}

/// Incipient subduction (`tectonics.incipient_subduction`, PLT-610): a front whose contact's slab
/// is shorter than this, km, is not yet self-sustaining. Gurnis, Hall & Lavier 2004: "Cumulative
/// convergence, not the rate of convergence, is the dominant control", the critical convergence
/// "only varies between 100 and 130 km (Table 3)"; Hall et al. 2003: "approximately 100 km of
/// convergence". The slab's length is its front's cumulative oceanic convergence (FRC-020), so
/// the bottom of the range is taken, as PLT-518's.
pub const INCIPIENT_SLAB_KM: f64 = 100.0; // ledger: PLT-611
/// The resistance of an incipient front to its convergence, N per metre of front at Earth's
/// gravity: the plate bending into the new trench. Gurnis et al. 2004: "rapid growth in resisting
/// force as the plate begins bending, reaching a maximum value dependent on plate thickness, but
/// typically ranging from 2 to 3 × 10^12 N/m for cases that become self-sustaining"; the middle
/// of the range. Plastic (a force whatever the speed), it enters the solve as a collision's
/// resistance does, over max(convergence, [`COLLISION_FLOOR_MM_YR`]).
pub const INCIPIENT_RESISTANCE_N_M: f64 = 2.5e12; // ledger: PLT-612
/// … where the floor going under is younger than [`SLAB_BUOYANT_MYR`], a ridge put under
/// compression: "The entire initial hump in resisting force is eliminated when lithosphere with a
/// typical thermal model of a spreading center is subject to compression", and the ridge's
/// "additional topography accounts for an additional 5 × 10^11 N/m which must be overcome to
/// compress the plates" (Gurnis et al. 2004).
pub const INCIPIENT_RIDGE_N_M: f64 = 5.0e11; // ledger: PLT-613

/// The slab length below which a front is incipient, km: [`INCIPIENT_SLAB_KM`] with
/// `incipient_subduction` on, else 0 (no front is). ledger: PLT-610
pub fn incipient_km(k: &TectonicParams) -> f64 {
    if k.incipient_subduction {
        INCIPIENT_SLAB_KM
    } else {
        0.0
    }
}

/// Whether a front whose contact's slab is `slab_km` long, before this step's growth, is
/// incipient (PLT-610): it resists its convergence ([`incipient_resistance_n_m`]), builds no arc
/// (`subduction::fronts_finish`), and the boundary mix counts it as converging, not subducting.
pub fn incipient(k: &TectonicParams, slab_km: f64) -> bool {
    slab_km < incipient_km(k)
}

/// The slab's thermal rate floor (`tectonics.slab_thermal_floor`, PLT-681), mm/yr of convergence
/// across the boundary: Toth & Gurnis 1998 (JGR 103, 18,053-18,067, p. 18,063, from their
/// models of a new subduction zone on a fault): "at subduction velocities of <10 mm/yr thermal
/// diffusion inhibits slab formation and no substantial slab pull force develops"; at 4 mm/yr
/// "the rate of convergence is so low that diffusive heating does not allow a slab to develop"
/// (p. 18,062). McKenzie 1977's 13 mm/yr ("required a convergence rate of 13 mm/yr for
/// subduction to be initiated", as Toth & Gurnis cite it, p. 18,064; Gurnis et al. 2004 [4] the
/// same; the paper itself not read, no open copy) is not used. Their criterion is for a slab
/// forming at a new trench (a model short of 10 mm/yr after 20 Myr has failed to initiate);
/// applied at once to a slow stretch of a mature front it takes the stretch's pull away before
/// an old slab would have warmed through (the agent's extension, an upper bound on the effect).
pub const SLAB_THERMAL_FLOOR_MM_YR: f64 = 10.0; // ledger: PLT-682

/// The convergence below which an oceanic front sample is incipient whatever its slab, mm/yr:
/// [`SLAB_THERMAL_FLOOR_MM_YR`] with `slab_thermal_floor` on, else 0 (none is). ledger: PLT-681
pub fn slab_floor_mm_yr(k: &TectonicParams) -> f64 {
    if k.slab_thermal_floor {
        SLAB_THERMAL_FLOOR_MM_YR
    } else {
        0.0
    }
}

/// Whether an oceanic front sample converging at `v_n_mm_yr` across its boundary is too slow for
/// a slab (PLT-681): incipient as a young contact's front is ([`incipient`]), whatever its
/// contact's slab. Every consumer asks this of its own v_n (the force balance, the arcs, the
/// boundary mix, the margins), each the other plate's motion towards the sample's along the
/// boundary's normal (PLT-310) at the step's motion.
pub fn below_floor(k: &TectonicParams, v_n_mm_yr: f64) -> bool {
    v_n_mm_yr < slab_floor_mm_yr(k)
}

/// Law 3 (`tectonics.incipient_resistance_by_thickness`, PLT-657): the incipient resistance of
/// floor of [`INCIPIENT_CASE1_AGE_MYR`], N per metre at Earth's gravity. Gurnis, Hall & Lavier
/// 2004 [59], their Case 1 (30 Myr homogeneous lithosphere): a peak of "3 × 10^12 N/m".
pub const INCIPIENT_CASE1_N_M: f64 = 3.0e12; // ledger: PLT-657
/// … the age of Case 1's floor, Myr ([46]: "Case 1, with an age of 30 Myr").
pub const INCIPIENT_CASE1_AGE_MYR: f64 = 30.0; // ledger: PLT-657
/// … and the growth of the force with the floor's thickness: [46], "the force required to cause
/// one plate to underthrust another increases by a factor of 2.2 as He increases" from 8.5 km (7
/// Myr floor) to 15 km (30 Myr floor), with both friction and cohesion. A power law through the
/// two points, F ∝ He^n with n = ln 2.2 / ln(15 / 8.5) ≈ 1.39; beyond 15 km it is extrapolated
/// (the agent's reading; [67] puts the initial bending force "between 10^12 and 10^13 N/m").
pub const INCIPIENT_THICKNESS_FACTOR: f64 = 2.2; // ledger: PLT-657
/// … the thin end of [46]'s pair, km …
pub const INCIPIENT_THIN_KM: f64 = 8.5; // ledger: PLT-657
/// … and the thick end, km.
pub const INCIPIENT_THICK_KM: f64 = 15.0; // ledger: PLT-657

/// The exponent n of law 3's power law in the elastic thickness (PLT-657), about 1.39.
pub fn incipient_thickness_exponent() -> f64 {
    INCIPIENT_THICKNESS_FACTOR.ln() / (INCIPIENT_THICK_KM / INCIPIENT_THIN_KM).ln()
}

/// Law 3's factor on Case 1's force for floor of `age_myr` (PLT-657): (Te(age) / Te(30 Myr))^n,
/// with Te the model's elastic thickness of ocean floor, 2.7 √age km and at most 40 km (ISO-048,
/// ISO-049, `surface::flexure`): 0.47 at 10 Myr, 1 at 30, 1.98 at 80, 3.46 at 180, and at most
/// 3.98 from 219 Myr on.
pub fn thickness_scale(age_myr: f64) -> f64 {
    let f = crate::surface::flexure::FlexureParams::default();
    let te = |a: f64| f.elastic_thickness_km(true, a, f64::INFINITY, false);
    (te(age_myr) / te(INCIPIENT_CASE1_AGE_MYR)).powf(incipient_thickness_exponent())
}

/// An incipient front's resistance, N per metre of front, where the floor going under is
/// `age_myr` old, at gravity `g`: scaled with gravity as the slab pull is (FRC-002). Floor
/// younger than [`SLAB_BUOYANT_MYR`] is a ridge (PLT-613); older floor resists with PLT-612's
/// single value, or with law 3 on, Case 1's force scaled with its elastic thickness
/// ([`thickness_scale`], PLT-657).
pub fn incipient_resistance_n_m(k: &TectonicParams, age_myr: f64, g: f64) -> f64 {
    let f = if age_myr < SLAB_BUOYANT_MYR {
        INCIPIENT_RIDGE_N_M // ledger: PLT-613
    } else if k.incipient_resistance_by_thickness {
        INCIPIENT_CASE1_N_M * thickness_scale(age_myr) // ledger: PLT-657
    } else {
        INCIPIENT_RESISTANCE_N_M // ledger: PLT-612
    };
    f * g / G_EARTH
}

/// Law 2's margin strength (PLT-658, the fallback): the integrated strength, N per metre of
/// margin, that a slab's pull must beat to cut a passive margin's floor off its continent, where
/// the floor at the cut is `age_myr` old, at gravity `g`. No source read gives a verified
/// integrated strength of a passive margin, so this is PLT-517's transference force (3 × 10¹²
/// N/m, Gurnis et al. 2004's Case 1 peak), scaled with the floor's thickness by law 3 when it is
/// on (and constant when it is off). It is the fallback the coordinator set (2026-10-25); the
/// age dependence of Zhou et al. 2020's propagation (old slabs propagate, young ones break off)
/// does not follow from it.
pub fn margin_strength_n_m(k: &TectonicParams, age_myr: f64, g: f64) -> f64 {
    let s = if k.incipient_resistance_by_thickness {
        thickness_scale(age_myr.max(SLAB_BUOYANT_MYR))
    } else {
        1.0
    };
    super::collision::TRANSFERENCE_FORCE_N_M * s * g / G_EARTH // ledger: PLT-658, PLT-517
}

/// Law 2's pull along a margin (PLT-656): the slab pull per metre (FRC-002, FRC-004) of a
/// slab `slab_km` long of floor `age_myr` old at gravity `g`, the share `cos_theta` of it
/// that acts across the margin (no pull for cos θ ≤ 0).
pub fn margin_pull_n_m(
    k: &TectonicParams,
    slab_km: f64,
    age_myr: f64,
    cos_theta: f64,
    g: f64,
) -> f64 {
    let depth = k.slab_depth_km.max(1.0);
    SLAB_PULL_N_M * g / G_EARTH
        * k.slab_pull
        * slab_weight(age_myr)
        * (slab_km.max(0.0).min(depth) / depth)
        * cos_theta.max(0.0)
}

/// The ridge-push coefficient `C` of the sea floor's potential energy, N/m³: a floor `d` metres
/// below the ridge crest has `C d²` less than the crest (Turcotte & Schubert's ridge push in
/// terms of the subsidence; ledger FRC-012). About 3.3 × 10⁵ at Earth's gravity.
pub fn ridge_coefficient(gravity: f64) -> f64 {
    let (rm, rw) = (RHO_MANTLE, RHO_WATER);
    let a = ALPHA_PER_K * DELTA_T_K;
    gravity * (rm - rw).powi(2) * std::f64::consts::PI / (4.0 * rm * a)
        * (1.0 + 2.0 * rm * a / (std::f64::consts::PI * (rm - rw)))
}

/// The gravitational potential energy of a column standing at `z_m` under a sea at `sea_m` (both
/// in the plate model's frame, m), N/m, relative to a ridge crest (`ridge_m`) and to continental
/// crust standing at the sea, which are taken to be equal (ledger FRC-013).
/// - Sea floor: `−C d²`, `d` its depth below the crest, at most the old floor's (`floor_m`):
///   deeper ground (a trench) is held down by the slab and by flexure, not by its own column.
/// - Continental crust (ledger FRC-014): `ρ g h (H + h (1 + ρ/(ρ_m − ρ_c))/2)` for an
///   Airy-compensated column `h = z − sea` above the sea on the crust that stands at the sea,
///   `H` thick; `ρ` is the crust's density above the sea and the crust's less the water's below
///   it. `H` is [`CRUST_KM`] at the datum and `sea · ρ_m / (ρ_m − ρ_c)` more under a sea off it
///   (the Airy thickness of a dry column standing there), so two dry columns differ by the same
///   energy wherever the sea stands. A 5 km belt on 35 km crust: 6.7 × 10¹² N/m (Tibet is quoted
///   at 6–7 × 10¹²). Under a sea at 0 it is the datum's formula, bit for bit.
pub fn column_energy(
    z_m: f64,
    continental: bool,
    ridge_m: f64,
    floor_m: f64,
    sea_m: f64,
    g: f64,
) -> f64 {
    if continental {
        // ledger: FRC-014 (the height and the water from the sea, the crust standing at the sea)
        let h = z_m - sea_m;
        let rho = if h >= 0.0 {
            // ledger: FRC-006, FRC-014
            RHO_CRUST
        } else {
            RHO_CRUST - RHO_WATER
        };
        let crust_m = CRUST_KM * 1000.0 + sea_m * RHO_MANTLE / (RHO_MANTLE - RHO_CRUST); // ledger: FRC-014
        let root = 1.0 + rho / (RHO_MANTLE - RHO_CRUST);
        rho * g * h * (crust_m + 0.5 * h * root)
    } else {
        let d = (ridge_m - z_m).clamp(0.0, (ridge_m - floor_m).max(0.0)); // ledger: FRC-012, FRC-021
        -ridge_coefficient(g) * d * d
    }
}

/// The mantle flow's upwelling axis for a run: drawn from the seed, by its own generator (the
/// plate model's is not touched).
pub fn flow_axis(seed: u64) -> DVec3 {
    SplitMix64::new(seed ^ 0x6d61_6e74_6c65_666c).unit_vector()
}

/// The mantle flow at the unit point `x`, in units of its peak speed: away from the upwellings
/// at ±`axis` towards the ring between them, `−2 (x·a) (a − (x·a) x)` (the surface gradient of a
/// degree-2 harmonic; its largest size, 1, is 45° from the axis). Ledger FRC-015.
pub fn flow_velocity(axis: DVec3, x: DVec3) -> DVec3 {
    let c = x.dot(axis);
    -2.0 * c * (axis - c * x) // ledger: FRC-015
}

/// The balance's coefficients in SI.
#[derive(Clone, Copy, Debug)]
pub struct Coeffs {
    /// R, m.
    pub radius_m: f64,
    /// One sample's area, m², and the length of boundary one boundary sample stands for, m.
    pub sample_m2: f64,
    pub spacing_m: f64,
    /// Slab pull per metre of trench of the reference slab, N/m (with `slab_pull` and gravity).
    pub pull_n_m: f64,
    pub suction_share: f64,
    /// The suction saturates at the slab's terminal velocity (`suction_saturates`, PLT-540; never
    /// with `forsyth_uyeda_balance`, whose suction does not depend on speed).
    pub suction_saturates: bool,
    /// The edge energy's multiplier (`edge_energy / EDGE_ENERGY_PHYSICAL`).
    pub edge_scale: f64,
    /// The multiplier of the collision's buoyancy (`collision_buoyancy`).
    pub buoyancy: f64,
    /// Basal drag under oceanic crust, Pa s/m, and the factor under continental crust.
    pub drag: f64,
    pub keel: f64,
    /// Slab resistance per metre of trench and boundary resistance per metre of boundary, N s/m².
    pub slab_res: f64,
    pub boundary_res: f64,
    /// The transform and colliding resistances, N per metre of boundary (friction:
    /// [`Balance::add_friction`]; 0 without `forsyth_uyeda_balance`), and the speed their links
    /// and the collision's divide by at least, mm/yr.
    pub transform_n_m: f64,
    pub colliding_n_m: f64,
    pub floor_mm_yr: f64,
    /// Slab length that pulls in full, km.
    pub slab_depth_km: f64,
    /// The mantle flow's peak speed, m/s, and its axis.
    pub flow_m_s: f64,
    pub flow_axis: DVec3,
    pub gravity: f64,
    /// The sea floor's crest and old-floor elevations, m (the column energy's reference).
    pub ridge_m: f64,
    pub floor_m: f64,
    /// The sea the step stands under, m: the land's column energy is taken from it.
    pub sea_m: f64,
}

impl Coeffs {
    /// From the settings. `a0_km2` is the average starting plate's area (the reference plate),
    /// `sea_m` the sea the step stands under.
    /// The basal drag coefficient follows from the reference plate: its area `A`, a straight
    /// trench of [`REF_TRENCH_KM`] of full-length 80-Myr slab, its neighbours at rest; then
    /// `(1 + slab_resistance + boundary_resistance) c_b A v = F_sp L` with `v` the speed scale
    /// (`slab_resistance` and `boundary_resistance` are multiples of `c_b A` per reference trench
    /// and per the reference plate's perimeter `2 √(π A)`).
    ///
    /// With `forsyth_uyeda_balance` (PLT-585) none of those settings is read: the resistances are
    /// Forsyth & Uyeda's (1975, Table 6) against the same reference pull. The slab resistance per
    /// metre of trench is the pull over their slab's terminal velocity, `F_SP / F_SR` (7.2 cm/yr;
    /// their text quotes 7.0 for the ratio of the drags, p. 189); the basal drag, the continental
    /// drag beyond it, the suction and the two friction resistances are their shares of it; and
    /// there is no viscous boundary resistance. The plates' speeds are then what these forces
    /// give, with no speed scale.
    pub fn new(
        k: &TectonicParams,
        norm: &Norm,
        gravity: f64,
        a0_km2: f64,
        flow_axis: DVec3,
        sea_m: f64,
    ) -> Coeffs {
        let radius_m = norm.model_km * 1000.0;
        let sample_m2 = norm.sample_area_km2() * 1.0e6;
        let spacing_m = norm.km(norm.spacing_rad()) * 1000.0;
        let g_scale = gravity / G_EARTH;
        let f_ref = SLAB_PULL_N_M * g_scale;
        let area = a0_km2.max(1.0) * 1.0e6;
        let trench = REF_TRENCH_KM * 1000.0;
        let perimeter = 2.0 * (std::f64::consts::PI * area).sqrt();
        let (drag, keel, slab_res, boundary_res, suction_share, transform_n_m, colliding_n_m) =
            if k.forsyth_uyeda_balance {
                // ledger: FRC-034 (the slab's terminal velocity, m/s: F_SP / F_SR in cm/yr)
                let v_t = FU_SLAB_PULL / FU_SLAB_RESISTANCE * 1.0e-2 / YEAR_S;
                let slab_res = f_ref / v_t; // ledger: FRC-034
                // F_DF per km² against F_SR per km of trench, both per cm/yr: per m², 1e-3 of it.
                let drag = slab_res * FU_MANTLE_DRAG / FU_SLAB_RESISTANCE * 1.0e-3; // ledger: FRC-034
                let keel = (FU_MANTLE_DRAG + FU_CONTINENTAL_DRAG) / FU_MANTLE_DRAG; // ledger: FRC-034
                (
                    drag,
                    keel,
                    slab_res,
                    0.0,
                    FU_SUCTION / FU_SLAB_PULL,           // ledger: FRC-034
                    f_ref * FU_TRANSFORM / FU_SLAB_PULL, // ledger: FRC-034, FRC-035
                    f_ref * FU_COLLIDING / FU_SLAB_PULL, // ledger: FRC-034, FRC-035
                )
            } else {
                let v = k.plate_speed_scale_mm_yr * 1.0e-3 / YEAR_S;
                let drag =
                    f_ref * trench / ((1.0 + k.slab_resistance + k.boundary_resistance) * area * v); // ledger: FRC-016
                (
                    drag,
                    k.keel_drag,
                    k.slab_resistance * drag * area / trench, // ledger: FRC-017
                    k.boundary_resistance * drag * area / perimeter, // ledger: FRC-017
                    k.slab_suction_share,
                    0.0,
                    0.0,
                )
            };
        Coeffs {
            radius_m,
            sample_m2,
            spacing_m,
            pull_n_m: f_ref * k.slab_pull,
            suction_share,
            suction_saturates: k.suction_saturates && !k.forsyth_uyeda_balance, // ledger: PLT-540, PLT-585
            edge_scale: k.edge_energy / EDGE_ENERGY_PHYSICAL,
            buoyancy: k.collision_buoyancy,
            drag,
            keel,
            slab_res,
            boundary_res,
            transform_n_m,
            colliding_n_m,
            floor_mm_yr: COLLISION_FLOOR_MM_YR.min(k.weld_speed_mm_yr), // ledger: FRC-011
            slab_depth_km: k.slab_depth_km,
            flow_m_s: k.mantle_flow_mm_yr * 1.0e-3 / YEAR_S,
            flow_axis,
            gravity,
            ridge_m: k.ridge_elevation_m,
            floor_m: k.oceanic_floor_elevation_m,
            sea_m,
        }
    }

    /// The column energy of a sample ([`column_energy`]).
    pub fn energy(&self, z_m: f64, continental: bool) -> f64 {
        column_energy(
            z_m,
            continental,
            self.ridge_m,
            self.floor_m,
            self.sea_m,
            self.gravity,
        )
    }
}

/// The driving terms, by which a force is booked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drive {
    SlabPull,
    Suction,
    Edge,
}

/// The resisting terms that act on plate boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resist {
    Slab,
    Boundary,
    Collision,
    /// The saturating suction's speed-dependent part (booked with the suction, PLT-540).
    Suction,
    /// The transform and colliding resistances' friction (booked with the boundary resistance,
    /// PLT-585).
    Friction,
}

/// The mantle as the other side of a link.
pub const MANTLE: u32 = u32::MAX;

/// A resistance at one boundary point to the velocity of plate `a` relative to plate `b` (or to
/// the mantle): along `n` only when `n` is set, else the whole tangent velocity. `k` is the
/// coefficient times the length it stands for, N s/m.
#[derive(Clone, Copy, Debug)]
pub struct Link {
    pub a: u32,
    pub b: u32,
    pub x: DVec3,
    pub n: Option<DVec3>,
    pub k: f64,
    pub term: Resist,
}

/// One plate's sums.
#[derive(Clone, Copy, Debug, Default)]
struct Acc {
    /// Visible samples, and their area, m².
    n: u32,
    area: f64,
    centroid: DVec3,
    /// Σ c_b A R² (I − x xᵀ): basal drag; Σ (I − x xᵀ): the same unweighted (for the RMS speed).
    drag: DMat3,
    geom: DMat3,
    /// Σ c_b A R (x × v_flow): the flow's torque.
    t_flow: DVec3,
    /// The basal drag's share of the stress moment: Σ c_b A R (x × r) (dotted with −Ω) and
    /// Σ c_b A r·v_flow.
    drag_moment: DVec3,
    flow_moment: f64,
    /// Torques of the driving forces, and their first moment Σ r·f.
    t_pull: DVec3,
    t_suck: DVec3,
    t_edge: DVec3,
    moment: f64,
}

/// The system for one step: plates, forces and links; [`Balance::solve`] gives the rotations.
#[derive(Clone, Debug)]
pub struct Balance {
    pub c: Coeffs,
    acc: Vec<Acc>,
    links: Vec<Link>,
}

/// A plate's answer and its budget.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlateAnswer {
    /// The balance's rotation, rad/yr (mantle frame, as `Plate::omega`).
    pub omega: DVec3,
    /// Torque magnitudes, N m, by term.
    pub slab_pull: f64,
    pub slab_suction: f64,
    pub edge_energy: f64,
    pub mantle_flow: f64,
    pub basal_drag: f64,
    pub slab_resistance: f64,
    pub boundary_resistance: f64,
    pub collision_resistance: f64,
    /// Mean in-plane stress, MPa (tension positive).
    pub stress_mpa: f64,
    /// Speed of the balance's rotation at the centroid and its RMS over the plate, mm/yr.
    pub speed_mm_yr: f64,
    pub rms_mm_yr: f64,
    /// |Σ torques| over the largest term at the answer: how well the solve balanced.
    pub residual: f64,
}

fn outer(a: DVec3, b: DVec3) -> DMat3 {
    DMat3::from_cols(a * b.x, a * b.y, a * b.z)
}

fn projector(x: DVec3) -> DMat3 {
    DMat3::IDENTITY - outer(x, x)
}

impl Balance {
    /// An empty system for plates `0..np` with these centroids (unit vectors).
    pub fn new(c: Coeffs, centroids: &[DVec3]) -> Balance {
        Balance {
            c,
            acc: centroids
                .iter()
                .map(|&centroid| Acc {
                    centroid,
                    ..Acc::default()
                })
                .collect(),
            links: Vec::new(),
        }
    }

    /// `x`'s offset from plate `p`'s centroid in the centroid's tangent plane, m.
    fn offset(&self, p: usize, x: DVec3) -> DVec3 {
        let c = self.acc[p].centroid;
        (x - c * c.dot(x)) * self.c.radius_m
    }

    /// One visible sample of plate `p` at `x`: its basal drag and the flow's.
    pub fn add_area(&mut self, p: u32, x: DVec3, continental: bool) {
        let mut one = Acc::default();
        self.area_into(&mut one, p as usize, x, continental);
        merge(&mut self.acc[p as usize], &one);
    }

    fn area_into(&self, a: &mut Acc, p: usize, x: DVec3, continental: bool) {
        let c = &self.c;
        let k = c.drag * if continental { c.keel } else { 1.0 } * c.sample_m2;
        let r2 = c.radius_m * c.radius_m;
        let proj = projector(x);
        let flow = flow_velocity(c.flow_axis, x) * c.flow_m_s;
        let r = self.offset(p, x);
        a.n += 1;
        a.area += c.sample_m2;
        a.drag += proj * (k * r2);
        a.geom += proj;
        a.t_flow += x.cross(flow) * (k * c.radius_m);
        a.drag_moment += x.cross(r) * (k * c.radius_m);
        a.flow_moment += k * r.dot(flow);
    }

    /// A driving force `f` (N) on plate `p` at `x`.
    pub fn add_force(&mut self, p: u32, x: DVec3, f: DVec3, term: Drive) {
        let t = x.cross(f) * self.c.radius_m;
        let m = self.offset(p as usize, x).dot(f);
        let a = &mut self.acc[p as usize];
        match term {
            Drive::SlabPull => a.t_pull += t,
            Drive::Suction => a.t_suck += t,
            Drive::Edge => a.t_edge += t,
        }
        a.moment += m;
    }

    pub fn add_link(&mut self, l: Link) {
        self.links.push(l);
    }

    /// The suction of a slab pulling with `pull` (N) on the overriding plate `p` at `x` over `len_m`
    /// of trench, `dir` pointing across the trench towards the subducting plate. A fixed share of
    /// the pull (PLT-186); with `suction_saturates` (PLT-540) that share is the traction at rest of
    /// `k (v_t − u)`, `u` the plate's speed along `dir`, `v_t = pull / (slab_res ℓ)` the slab's
    /// terminal velocity (`ℓ = len_m`), so `k = share × slab_res × ℓ`: a link to the mantle along
    /// `dir`.
    pub fn add_suction(&mut self, p: u32, x: DVec3, dir: DVec3, pull: f64, len_m: f64) {
        if pull <= 0.0 {
            return;
        }
        let c = self.c;
        self.add_force(p, x, dir * (pull * c.suction_share), Drive::Suction);
        if c.suction_saturates && c.slab_res > 0.0 {
            self.add_link(Link {
                a: p,
                b: MANTLE,
                x,
                n: Some(dir),
                k: c.suction_share * c.slab_res * len_m, // ledger: PLT-540
                term: Resist::Suction,
            });
        }
    }

    /// A friction of `force_n` (N) between plates `a` and `b` at the boundary point `x`, against
    /// their relative motion: Forsyth & Uyeda's transform and colliding resistances, "independent
    /// of velocity and parallel to the relative motion between the plates", on both plates alike
    /// (1975, pp. 169-170). Plastic, as the collision's resistance (design §3.4): in the linear
    /// solve a link against the whole relative velocity whose coefficient is the force over this
    /// step's relative speed `rel_mm_yr`, bounded below by `floor_mm_yr`, so where the plates
    /// keep their relative motion it resists with the force itself, and a boundary the drive
    /// cannot move against it slips at no more than the floor. ledger: FRC-035
    pub fn add_friction(&mut self, a: u32, b: u32, x: DVec3, force_n: f64, rel_mm_yr: f64) {
        if force_n <= 0.0 || self.c.floor_mm_yr <= 0.0 {
            return;
        }
        let speed = rel_mm_yr.max(self.c.floor_mm_yr) * 1.0e-3 / YEAR_S; // ledger: FRC-011
        self.add_link(Link {
            a,
            b,
            x,
            n: None,
            k: force_n / speed, // ledger: FRC-035
            term: Resist::Friction,
        });
    }

    /// Plates in the system: those with visible samples.
    fn index(&self) -> (Vec<usize>, Vec<Option<usize>>) {
        let mut idx = vec![None; self.acc.len()];
        let mut of = Vec::new();
        for (p, a) in self.acc.iter().enumerate() {
            if a.n > 0 {
                idx[p] = Some(of.len());
                of.push(p);
            }
        }
        (of, idx)
    }

    /// The link's torque matrix (on `a`, against `a`'s velocity relative to `b`), N m s.
    fn link_matrix(&self, l: &Link) -> DMat3 {
        let r2 = self.c.radius_m * self.c.radius_m;
        match l.n {
            Some(n) => {
                let arm = l.x.cross(n);
                outer(arm, arm) * (l.k * r2)
            }
            None => projector(l.x) * (l.k * r2),
        }
    }

    /// Solves `M Ω = T`; `None` when the system is empty or not positive definite.
    pub fn solve(&self) -> Option<Vec<DVec3>> {
        let (of, idx) = self.index();
        let m = of.len();
        if m == 0 {
            return None;
        }
        let dim = 3 * m;
        let mut mat = vec![0.0f64; dim * dim];
        let mut rhs = vec![0.0f64; dim];
        let add = |mat: &mut [f64], i: usize, j: usize, k: &DMat3, sign: f64| {
            for r in 0..3 {
                for c in 0..3 {
                    mat[(3 * i + r) * dim + 3 * j + c] += sign * k.col(c)[r];
                }
            }
        };
        for (i, &p) in of.iter().enumerate() {
            let a = &self.acc[p];
            add(&mut mat, i, i, &a.drag, 1.0);
            let t = a.t_pull + a.t_suck + a.t_edge + a.t_flow;
            rhs[3 * i] = t.x;
            rhs[3 * i + 1] = t.y;
            rhs[3 * i + 2] = t.z;
        }
        for l in &self.links {
            let Some(i) = idx.get(l.a as usize).copied().flatten() else {
                continue;
            };
            let k = self.link_matrix(l);
            add(&mut mat, i, i, &k, 1.0);
            if let Some(j) = idx.get(l.b as usize).copied().flatten() {
                add(&mut mat, j, j, &k, 1.0);
                add(&mut mat, i, j, &k, -1.0);
                add(&mut mat, j, i, &k, -1.0);
            }
        }
        // A tiny ridge on the diagonal: a plate of a few samples has almost no drag against
        // spinning about its own centroid.
        for i in 0..m {
            let tr = (0..3)
                .map(|r| mat[(3 * i + r) * dim + 3 * i + r])
                .sum::<f64>()
                / 3.0;
            for r in 0..3 {
                mat[(3 * i + r) * dim + 3 * i + r] += 1.0e-9 * tr;
            }
        }
        let x = cholesky_solve(&mut mat, &rhs, dim)?;
        let mut w = vec![DVec3::ZERO; self.acc.len()];
        for (i, &p) in of.iter().enumerate() {
            w[p] = DVec3::new(x[3 * i], x[3 * i + 1], x[3 * i + 2]);
        }
        Some(w)
    }

    /// The budget of every plate at the rotations `w` (rad/s, from [`Balance::solve`]).
    pub fn answers(&self, w: &[DVec3]) -> Vec<Option<PlateAnswer>> {
        let r = self.c.radius_m;
        let np = self.acc.len();
        let (mut t_slab, mut t_bnd, mut t_col, mut t_sat) = (
            vec![DVec3::ZERO; np],
            vec![DVec3::ZERO; np],
            vec![DVec3::ZERO; np],
            vec![DVec3::ZERO; np],
        );
        let mut moment: Vec<f64> = self.acc.iter().map(|a| a.moment).collect();
        let wof = |p: u32| -> DVec3 {
            if p == MANTLE {
                DVec3::ZERO
            } else {
                w.get(p as usize).copied().unwrap_or(DVec3::ZERO)
            }
        };
        for l in &self.links {
            if self.acc.get(l.a as usize).is_none_or(|a| a.n == 0) {
                continue;
            }
            let dw = wof(l.a) - wof(l.b);
            // The force on `a` (N); `b` gets its opposite.
            let f = match l.n {
                Some(n) => -n * (l.k * r * dw.dot(l.x.cross(n))),
                None => -(dw.cross(l.x)) * (l.k * r),
            };
            let t = l.x.cross(f) * r;
            let mut book = |p: u32, t: DVec3, f: DVec3| {
                let p = p as usize;
                match l.term {
                    Resist::Slab => t_slab[p] += t,
                    Resist::Boundary | Resist::Friction => t_bnd[p] += t,
                    Resist::Collision => t_col[p] += t,
                    Resist::Suction => t_sat[p] += t,
                }
                moment[p] += self.offset(p, l.x).dot(f);
            };
            book(l.a, t, f);
            if l.b != MANTLE && self.acc.get(l.b as usize).is_some_and(|a| a.n > 0) {
                book(l.b, -t, -f);
            }
        }
        let h = LITHOSPHERE_KM * 1000.0;
        (0..np)
            .map(|p| {
                let a = &self.acc[p];
                if a.n == 0 {
                    return None;
                }
                let wp = w[p];
                let t_drag = -(a.drag * wp);
                let body = -wp.dot(a.drag_moment) + a.flow_moment;
                // The suction net of its saturation (PLT-540; nothing without it).
                let t_suck = a.t_suck + t_sat[p];
                let terms = [
                    a.t_pull, t_suck, a.t_edge, a.t_flow, t_drag, t_slab[p], t_bnd[p], t_col[p],
                ];
                let sum: DVec3 = terms.iter().copied().sum();
                let largest = terms.iter().map(|t| t.length()).fold(0.0, f64::max);
                let w_yr = wp * YEAR_S;
                let rms = (wp.dot(a.geom * wp) / f64::from(a.n)).max(0.0).sqrt() * r;
                Some(PlateAnswer {
                    omega: w_yr,
                    slab_pull: a.t_pull.length(),
                    slab_suction: t_suck.length(),
                    edge_energy: a.t_edge.length(),
                    mantle_flow: a.t_flow.length(),
                    basal_drag: t_drag.length(),
                    slab_resistance: t_slab[p].length(),
                    boundary_resistance: t_bnd[p].length(),
                    collision_resistance: t_col[p].length(),
                    stress_mpa: (moment[p] + body) / (2.0 * a.area * h) * 1.0e-6, // ledger: FRC-010, FRC-022
                    speed_mm_yr: wp.cross(a.centroid).length() * r * 1.0e3 * YEAR_S,
                    rms_mm_yr: rms * 1.0e3 * YEAR_S,
                    residual: if largest > 0.0 {
                        sum.length() / largest
                    } else {
                        0.0
                    },
                })
            })
            .collect()
    }
}

fn merge(into: &mut Acc, a: &Acc) {
    into.n += a.n;
    into.area += a.area;
    into.drag += a.drag;
    into.geom += a.geom;
    into.t_flow += a.t_flow;
    into.drag_moment += a.drag_moment;
    into.flow_moment += a.flow_moment;
    into.t_pull += a.t_pull;
    into.t_suck += a.t_suck;
    into.t_edge += a.t_edge;
    into.moment += a.moment;
}

/// Solves the symmetric positive definite `mat x = rhs` (row-major, `dim` × `dim`) by Cholesky
/// factorisation in place; `None` when a pivot is not positive.
fn cholesky_solve(mat: &mut [f64], rhs: &[f64], dim: usize) -> Option<Vec<f64>> {
    for j in 0..dim {
        let mut d = mat[j * dim + j];
        for k in 0..j {
            d -= mat[j * dim + k] * mat[j * dim + k];
        }
        if d.is_nan() || d <= 0.0 {
            return None;
        }
        let d = d.sqrt();
        mat[j * dim + j] = d;
        for i in j + 1..dim {
            let mut s = mat[i * dim + j];
            for k in 0..j {
                s -= mat[i * dim + k] * mat[j * dim + k];
            }
            mat[i * dim + j] = s / d;
        }
    }
    let mut y = rhs.to_vec();
    for i in 0..dim {
        for k in 0..i {
            y[i] -= mat[i * dim + k] * y[k];
        }
        y[i] /= mat[i * dim + i];
    }
    for i in (0..dim).rev() {
        for k in i + 1..dim {
            y[i] -= mat[k * dim + i] * y[k];
        }
        y[i] /= mat[i * dim + i];
    }
    y.iter().all(|v| v.is_finite()).then_some(y)
}

/// What one visible boundary sample sees: its outward direction (towards other plates and its
/// own subducted crust), the other plates beside it, and at most one subduction front.
#[derive(Clone, Copy, Debug)]
struct Edge {
    v: u32,
    plate: u32,
    out: DVec3,
    others: [u32; 6],
    n_others: u8,
    /// With `forsyth_uyeda_balance` (PLT-585), each other plate's friction at this sample: the
    /// force, N (the transform's or the colliding resistance times the front's metres at the
    /// front this sample overrides; elsewhere times half the sample's share of the boundary, the
    /// other half from the other side), and the plates' relative speed, mm/yr.
    friction: [(f64, f64); 6],
    /// The front this sample overrides.
    front: Option<SlabFront>,
    /// A neighbour of a plate whose slab hangs under this sample's plate (a slab with no front
    /// this step, or one a subduction initiation planted): the neighbour, the direction towards
    /// it the slab pulls along and the length of trench the sample stands for, m (as at a
    /// front).
    hang: Option<(u32, DVec3, f64)>,
    /// The sample is continental on the subducting side of an active contact, converging: the
    /// incoming side of a collision.
    incoming: bool,
}

/// A subduction front at one overriding sample: where the slab pull, the suction, the slab
/// resistance and a collision's resistance act.
#[derive(Clone, Copy, Debug)]
struct SlabFront {
    /// The subducting neighbour and the contact.
    u: u32,
    contact: usize,
    /// The direction across the trench towards the subducting plate the forces act along, the
    /// convergence along it (mm/yr) and the length of trench the sample stands for (m): the
    /// boundary's normal, the convergence along it and the sample's share of the straight front
    /// with `front_pull` (PLT-560), else the converging lattice edge's direction, the
    /// convergence along it and one sample spacing.
    dir: DVec3,
    conv: f64,
    len_m: f64,
    /// The converging lattice edge's direction, the boundary's normal and the sample's share of
    /// the straight front, rad (the PLT-554 measure's).
    edge_dir: DVec3,
    normal: DVec3,
    share_rad: f64,
}

/// One contact's front this step: its front samples with oceanic and with continental crust on
/// the subducting side, and the oceanic ones' summed convergence, mm/yr.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrontSum {
    pub oceanic: u32,
    pub continental: u32,
    pub conv_sum: f64,
}

/// One step of a slab fed by a front (design §3.3; ledger PLT-184): it lengthens by the oceanic
/// front's mean convergence over `dt_myr` (mm/yr × Myr = km), up to `depth_km`; while
/// continental crust holds at least half of the front the block time runs, and after
/// `breakoff_myr` of it the slab is gone (length 0; returns true). An unblocked step resets the
/// block time.
pub fn grow_slab(
    s: &mut SlabTrack,
    f: &FrontSum,
    dt_myr: f64,
    depth_km: f64,
    breakoff_myr: f64,
) -> bool {
    if f.oceanic > 0 {
        let conv = f.conv_sum / f64::from(f.oceanic);
        s.km = (s.km + conv * dt_myr).min(depth_km); // ledger: FRC-020
    }
    if f.continental > 0 && f.continental >= f.oceanic {
        // ledger: FRC-019
        s.blocked_myr += dt_myr;
        if s.blocked_myr >= breakoff_myr - 1e-9 {
            s.km = 0.0;
            s.blocked_myr = 0.0;
            return true;
        }
    } else {
        s.blocked_myr = 0.0;
    }
    false
}

/// What the step's balance recorded.
#[derive(Clone, Debug, Default)]
pub struct Recorded {
    pub forces: Vec<PlateForces>,
    /// Slab length of each contact with an oceanic front this step, km.
    pub slabs_km: Vec<f64>,
    /// Slabs that broke off this step.
    pub breakoffs: u16,
    /// Measurement only (PLT-683): each contact whose slab passed the critical convergence this
    /// step (incipient before the step, PLT-610, its slab at least `INCIPIENT_SLAB_KM` after
    /// its growth): its oceanic front's mean convergence, mm/yr (the slab's feed, FRC-020), the
    /// contact's age, Myr since its onset, and its oceanic front samples. In contact order.
    pub matured: Vec<[f64; 3]>,
}

/// The step's balance: each plate's balance rotation, rad/yr (indexed by plate id; `None` for a
/// plate not in the balance, or for every plate when the solve failed), and what the dynamics
/// record keeps (`None` when the record is off).
#[derive(Clone, Debug, Default)]
pub struct Answer {
    pub omega: Vec<Option<DVec3>>,
    /// Each plate's mean in-plane stress, MPa, tension positive (indexed by plate id, `None` as
    /// for `omega`): handed on to where rifts and subduction start (ledger FRC-022, PLT-504,
    /// PLT-505). Never stored.
    pub stress_mpa: Vec<Option<f64>>,
    /// Each collision front's plastic resistance this step, N (by contact index): the force the
    /// collision pushes back on its plates with while it converges faster than the floor, summed
    /// over the front (design §3.4). Read by transference (`collision::transference_due`, ledger
    /// PLT-516). Never stored.
    pub collision_n: BTreeMap<usize, f64>,
    /// Each incipient front's resistance this step, N (by contact index): its plastic force
    /// before linearisation, summed over the front (PLT-610, PLT-612). Never stored.
    pub incipient_n: BTreeMap<usize, f64>,
    pub solved: bool,
    pub recorded: Option<Recorded>,
    /// Measurement only: the oceanic fronts the slab pull acts along this step, km
    /// (Earth-normalized): one spacing per overriding sample that pulls, the part of each along
    /// the boundary's normal, and the samples' shares of the straight front (PLT-530). ledger:
    /// PLT-554
    pub slab_front_km: [f64; 3],
}

/// Four significant digits (the record's precision for forces).
fn sig4(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return 0.0;
    }
    let e = x.abs().log10().floor() as i32 - 3;
    let s = 10f64.powi(e);
    (x / s).round() * s
}

/// The buoyancy of the continental lithosphere a collision has dragged down, N per metre of front
/// (design §3.4's collision resistance, its second part; ledger FRC-030). A collision that has
/// shortened by `s` km has dragged about `s` km of the incoming plate's lithosphere down the
/// trench; an 80-Myr slab that long would pull with `pull_n_m × min(s, D) / D` (`D` the slab
/// length that pulls in full, `slab_depth_km`, as for slab pull), and continental lithosphere
/// pushes back up with [`CONTINENT_BUOYANCY_RATIO`] times that, in proportion to its crust's
/// thickness `t_m` over [`CONTINENT_BUOYANCY_CRUST_KM`]. So continental crust "resists
/// subduction" (Cloos 1993): a collision slows as it shortens and stalls once nothing else pulls
/// hard enough.
pub fn collision_buoyancy_n_m(
    pull_n_m: f64,
    slab_depth_km: f64,
    t_m: f64,
    shortening_km: f64,
) -> f64 {
    let d = slab_depth_km.max(1.0);
    CONTINENT_BUOYANCY_RATIO // ledger: FRC-030
        * pull_n_m
        * (t_m.max(0.0) / (CONTINENT_BUOYANCY_CRUST_KM * 1000.0))
        * (shortening_km.max(0.0).min(d) / d)
}

/// The push per metre of a mountain belt on the lowland beside it: the belt's column energy over
/// the lowland's, `du_n_m` (at least 0), with the balance's edge scale, N/m. It is the edge term
/// (FRC-018) read across a belt: the force a range exerts on its lowlands (Molnar & Lyon-Caen 1988:
/// Tibet's 6–7 × 10¹² N/m; [`column_energy`] gives 6.7 × 10¹² for a 5 km belt). For the overriding
/// plate's share of a collision's convergence (`belts.rs`, PLT-652).
pub fn belt_push_n_m(k: &TectonicParams, du_n_m: f64) -> f64 {
    let edge_scale = k.edge_energy / EDGE_ENERGY_PHYSICAL; // ledger: FRC-009
    edge_scale * du_n_m.max(0.0) // ledger: FRC-018, PLT-652
}

/// The relaxation of a rotation towards the balance's over one step (design §5.1 a): the share
/// `1 − e^(−δt/τ)` of the way, τ = `motion_relax_myr` (ledger PLT-191).
pub fn relax(omega: DVec3, target: DVec3, dt_myr: f64, tau_myr: f64) -> DVec3 {
    let a = 1.0 - (-dt_myr / tau_myr.max(1e-9)).exp();
    omega + (target - omega) * a
}

/// The step's slab lengths and force balance, on the positions `pos` and the motion the step's
/// fronts were found with (call it after `subduction::fronts_and_uplift`, before the motion is
/// updated). Updates the slabs (`PlateState::slabs`) and, when the dynamics record is on, its
/// memory of the last answers; returns each plate's balance rotation, which the plates step
/// relaxes the plates' motion towards. `sea_m` is the sea the step stands under (the land's
/// column energy is taken from it, [`column_energy`]).
pub fn step(
    st: &mut PlateState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env<'_>,
    flow: DVec3,
    sea_m: f64,
) -> Answer {
    let mut slabs = std::mem::take(&mut st.slabs);
    let Balanced {
        omega,
        stress_mpa,
        collision_n,
        incipient_n,
        solved,
        out,
        slab_front_km,
        ..
    } = balance_step(st, geo, pos, env, flow, sea_m, &mut slabs);
    st.slabs = slabs;
    let recorded = st.dynamics.as_mut().map(|d| {
        let last: BTreeMap<u32, DVec3> = d
            .prev_balance
            .iter()
            .map(|&(id, w)| (id, DVec3::from_array(w)))
            .collect();
        d.prev_balance.clear();
        let mut out = out;
        for f in &mut out.forces {
            if let Some(w) = omega.get(f.id as usize).copied().flatten() {
                let change = last
                    .get(&f.id)
                    .and_then(|&w0| super::dynamics::rotation_change(w0, w));
                f.rot_change = change.map_or(0.0, |x| (x * 1.0e4).round() / 1.0e4);
                f.rot_known = change.is_some();
                d.prev_balance.push((f.id, w.to_array()));
            }
        }
        out
    });
    Answer {
        omega,
        stress_mpa,
        collision_n,
        incipient_n,
        solved,
        recorded,
        slab_front_km,
    }
}

/// What [`balance_step`] answers: each plate's balance rotation and mean stress (by plate id),
/// whether the solve succeeded, and the record's part.
struct Balanced {
    omega: Vec<Option<DVec3>>,
    stress_mpa: Vec<Option<f64>>,
    collision_n: BTreeMap<usize, f64>,
    incipient_n: BTreeMap<usize, f64>,
    solved: bool,
    out: Recorded,
    slab_front_km: [f64; 3],
    /// Each plate's slab pull and suction torques (N m, about the planet's centre, by plate id).
    #[cfg(test)]
    torques: Vec<(DVec3, DVec3)>,
    #[cfg(test)]
    terms: BoundaryTerms,
}

/// The trench resistances of one balance, for the tests: each (plate, other plate)'s Forsyth &
/// Uyeda friction summed over the plate's boundary samples, N, and the part of it at the fronts
/// the plate overrides; each front's oceanic metres, by (overriding, subducting) plate; each
/// incipient front's resistance, N, by contact; and the colliding and transform resistances, N/m.
#[cfg(test)]
#[derive(Clone, Debug, Default)]
pub(in crate::plates) struct BoundaryTerms {
    pub friction_n: BTreeMap<(u32, u32), f64>,
    pub front_friction_n: BTreeMap<(u32, u32), f64>,
    pub front_m: BTreeMap<(u32, u32), f64>,
    pub incipient_n: BTreeMap<usize, f64>,
    pub colliding_n_m: f64,
    pub transform_n_m: f64,
}

/// [`BoundaryTerms`] at a balance of `st` as it stands, its slabs left as they are.
#[cfg(test)]
pub(in crate::plates) fn boundary_terms(
    st: &PlateState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env<'_>,
) -> BoundaryTerms {
    let mut slabs = st.slabs.clone();
    balance_step(st, geo, pos, env, DVec3::Z, 0.0, &mut slabs).terms
}

/// The slab pull's and the suction's torque on each plate (N m, about the planet's centre, by
/// plate id) at a balance of `st` as it stands, its slabs left as they are.
#[cfg(test)]
pub(in crate::plates) fn slab_torques(
    st: &PlateState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env<'_>,
) -> Vec<(DVec3, DVec3)> {
    let mut slabs = st.slabs.clone();
    balance_step(st, geo, pos, env, DVec3::Z, 0.0, &mut slabs).torques
}

fn balance_step(
    st: &PlateState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env<'_>,
    flow: DVec3,
    sea_m: f64,
    slabs: &mut Vec<SlabTrack>,
) -> Balanced {
    let k = env.k;
    let norm = env.norm;
    let samples = &st.samples;
    let n = samples.len();
    let np = st.plates.len();
    let plate_of: Vec<u32> = (0..n).map(|i| st.plate_of(i)).collect();
    let omega: Vec<DVec3> = st
        .plates
        .iter()
        .map(|p| DVec3::from_array(p.omega))
        .collect();
    let rots: Vec<_> = st.groups.iter().map(|g| quat(g.rot)).collect();
    let pts = geo.points();
    let mut active: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for (i, c) in st.contacts.iter().enumerate() {
        // Overlap seen this step, or carried through a resampling (ledger: PLT-550).
        if c.active(env.step) {
            active.insert((c.under, c.over), i);
        }
    }
    // Slabs of living plates (before this step's growth): a sample of the overriding plate beside
    // the subducting one may hang one.
    slabs.retain(|s| {
        s.under != s.over
            && st.plates.get(s.under as usize).is_some_and(|p| p.alive)
            && st.plates.get(s.over as usize).is_some_and(|p| p.alive)
    });
    let hanging: BTreeSet<(u32, u32)> = slabs
        .iter()
        .filter(|s| s.km > 0.0 && !s.latent)
        .map(|s| (s.under, s.over))
        .collect();
    // The active contacts at their incipient stage (PLT-610), by their slab before this step's
    // growth (none without a slab yet), as the arcs and the boundary mix read it.
    let incipient_pairs: BTreeSet<(u32, u32)> = if k.incipient_subduction {
        let before: BTreeMap<(u32, u32), f64> =
            slabs.iter().map(|s| ((s.under, s.over), s.km)).collect();
        active
            .keys()
            .filter(|key| incipient(k, before.get(key).copied().unwrap_or(0.0)))
            .copied()
            .collect()
    } else {
        BTreeSet::new()
    };
    // The length of boundary one boundary sample stands for, m (`Coeffs::spacing_m`).
    let ell = norm.km(norm.spacing_rad()) * 1000.0;
    let c = Coeffs::new(k, &norm, env.gravity, st.a0_km2, flow, sea_m);
    let friction = c.transform_n_m > 0.0 || c.colliding_n_m > 0.0;
    // The front on the current geometry (PLT-582), as `subduction::fronts_begin` asks it: a plate
    // beside a sample counts for its front, its hanging slab and a collision's incoming side only
    // while a visible sample of it stands within the row of the sample now. The lattice is frozen
    // between resamplings, so its neighbours of a consumed plate have gone under or been carried
    // on. ledger: PLT-582
    let row = k
        .front_current_geometry
        .then(|| subduction::trench_row_rad(norm.spacing_rad()));
    let tree = row.map(|_| Tree::over(pos, |_| true));

    // The boundary, sample by sample (the tests of `subduction::fronts_and_uplift`).
    let edges: Vec<Edge> = (0..n)
        .into_par_iter()
        .filter_map(|v| {
            let sv = &samples[v];
            if sv.is_hidden() {
                return None;
            }
            let pv = plate_of[v];
            let x = pos[v];
            let rot = rots[sv.group as usize];
            let mut e = Edge {
                v: v as u32,
                plate: pv,
                out: DVec3::ZERO,
                others: [u32::MAX; 6],
                n_others: 0,
                friction: [(0.0, 0.0); 6],
                front: None,
                hang: None,
                incoming: false,
            };
            let mut any = false;
            let mut best = 0.0;
            // The other plates with a visible sample beside this one, by slot.
            let mut seen = 0u8;
            let mut beside = subduction::SlabBeside::new(row);
            let mut near = |q: u32| match &tree {
                Some(t) => beside.has(samples, &plate_of, t, x, q),
                None => true,
            };
            for (u, _) in geo.edges(v as u32) {
                let su = &samples[u as usize];
                let pu = plate_of[u as usize];
                let own_slab = pu == pv && su.is_hidden();
                if pu == pv && !own_slab {
                    continue;
                }
                any = true;
                let dir = tangent(x, rot * (pts[u as usize] - pts[v])).normalize_or_zero();
                e.out += dir;
                if own_slab {
                    continue;
                }
                let slot = match e.others[..usize::from(e.n_others)]
                    .iter()
                    .position(|&q| q == pu)
                {
                    Some(i) => Some(i),
                    None if usize::from(e.n_others) < e.others.len() => {
                        e.others[usize::from(e.n_others)] = pu;
                        e.n_others += 1;
                        Some(usize::from(e.n_others) - 1)
                    }
                    None => None,
                };
                if let Some(i) = slot
                    && !su.is_hidden()
                {
                    seen |= 1 << i;
                }
                if e.hang.is_none() && !su.is_hidden() && hanging.contains(&(pu, pv)) && near(pu) {
                    e.hang = Some((u, dir, ell));
                }
                let rel = norm.velocity_mm_yr(omega[pu as usize] - omega[pv as usize], x);
                let along = rel.dot(dir);
                if along < 0.0 {
                    if let Some(&c) = active.get(&(pu, pv))
                        && -along > best
                        && near(pu)
                    {
                        best = -along;
                        e.front = Some(SlabFront {
                            u,
                            contact: c,
                            dir,
                            conv: -along,
                            len_m: ell,
                            edge_dir: dir,
                            normal: DVec3::ZERO,
                            share_rad: 0.0,
                        });
                    } else if active.contains_key(&(pv, pu)) && sv.kind == CONTINENTAL && near(pu) {
                        e.incoming = true;
                    }
                }
            }
            // The boundary's normal (PLT-310) and the sample's share of the straight front
            // (PLT-530) at a front: the PLT-554 measure's, and with `front_pull` the front's.
            if let Some(f) = e.front.as_mut() {
                let pu = plate_of[f.u as usize];
                f.normal = shortening::boundary_normal(geo, pos, &rots, samples, &plate_of, v, pu);
                f.share_rad = shortening::front_share_rad(
                    geo, pos, &rots, samples, &plate_of, v, pu, f.normal,
                );
            }
            if k.front_pull {
                // The arcs' fronts (`subduction::fronts_begin`): the plates converge across the
                // boundary, beyond Bird's 20 degrees of a transform (PLT-330's rule); one
                // lattice edge's direction is no azimuth, and a contact's strike-slip stretches
                // converge along half the edges of their zigzag. At a front the forces act along
                // the boundary's normal over the sample's share of the straight front, the slab
                // fed by the convergence along that normal. ledger: PLT-560
                let front = e.front.and_then(|f| {
                    let pu = plate_of[f.u as usize];
                    let rel = norm.velocity_mm_yr(omega[pu as usize] - omega[pv as usize], x);
                    let conv = -rel.dot(f.normal);
                    (!shortening::is_transform(rel, f.normal) && conv > 0.0).then_some(SlabFront {
                        dir: f.normal,
                        conv,
                        len_m: norm.km(f.share_rad) * 1000.0,
                        ..f
                    })
                });
                e.front = front;
                // A hanging slab pulls the same way, along the whole shared boundary (no front
                // feeds it, so no relative motion selects its stretches).
                e.hang = e.hang.filter(|_| e.front.is_none()).map(|(u, _, _)| {
                    let pu = plate_of[u as usize];
                    let normal =
                        shortening::boundary_normal(geo, pos, &rots, samples, &plate_of, v, pu);
                    let share = shortening::front_share_rad(
                        geo, pos, &rots, samples, &plate_of, v, pu, normal,
                    );
                    (u, normal, norm.km(share) * 1000.0)
                });
            }
            // Forsyth & Uyeda's transform and colliding resistances (PLT-585), by their own
            // dividing rule, the one their strengths were fitted with (`fu_transform`): a
            // transform where the relative motion lies more than 60 degrees from the boundary's
            // normal, a colliding boundary where the plates converge across it otherwise, nothing
            // where they part (a ridge's resistance is not in their model). A trench's is counted
            // once, where its pull and suction are: at the front this sample overrides, over the
            // front's metres, against the plate going under (whose own samples beside the trench
            // have gone under it and see no other plate). Every other stretch of boundary with a
            // visible plate beyond it counts from both sides, half of its share each (PLT-530),
            // by its own normal (PLT-310), except the side going under in an active contact,
            // which its front counts. With incipient subduction (PLT-610), a contact at its
            // incipient stage has none from either side: its front's oceanic samples take the
            // incipient resistance alone, which stands for the trench's resistance until the
            // slab sinks by itself (a continental sample at its front is a collision's, as at a
            // mature contact). An oceanic front sample converging below the slab's thermal floor
            // (PLT-681) is incipient whatever its contact's slab, and takes none either.
            if friction {
                let front_q = e.front.map(|f| plate_of[f.u as usize]);
                if let Some(f) = e.front
                    && let Some(q) = front_q
                    && !((incipient_pairs.contains(&(q, pv)) || below_floor(k, f.conv)) && samples[f.u as usize].kind != CONTINENTAL) // ledger: PLT-610, PLT-681
                    && let Some(i) = e.others[..usize::from(e.n_others)]
                        .iter()
                        .position(|&o| o == q)
                {
                    let rel = norm.velocity_mm_yr(omega[q as usize] - omega[pv as usize], x);
                    let per_m = if fu_transform(rel, f.normal) {
                        c.transform_n_m
                    } else {
                        c.colliding_n_m
                    };
                    e.friction[i] = (per_m * f.len_m, rel.length()); // ledger: FRC-035
                }
                for i in 0..usize::from(e.n_others) {
                    let q = e.others[i];
                    // An incipient partner is skipped from either side: the side going under is an
                    // active contact's, the overriding side's here. ledger: PLT-610
                    if seen & (1 << i) == 0
                        || Some(q) == front_q
                        || active.contains_key(&(pv, q))
                        || incipient_pairs.contains(&(q, pv))
                    {
                        continue;
                    }
                    let normal =
                        shortening::boundary_normal(geo, pos, &rots, samples, &plate_of, v, q);
                    let rel = norm.velocity_mm_yr(omega[q as usize] - omega[pv as usize], x);
                    let per_m = if fu_transform(rel, normal) {
                        c.transform_n_m
                    } else if rel.dot(normal) < 0.0 {
                        c.colliding_n_m
                    } else {
                        0.0
                    };
                    if per_m > 0.0 {
                        let share = shortening::front_share_rad(
                            geo, pos, &rots, samples, &plate_of, v, q, normal,
                        );
                        let len_m = norm.km(share) * 1000.0;
                        e.friction[i] = (per_m * len_m * 0.5, rel.length()); // ledger: FRC-035
                    }
                }
            }
            any.then_some(e)
        })
        .collect();
    #[cfg(test)]
    let (friction_n, front_friction_n, front_m) = {
        let mut friction_n: BTreeMap<(u32, u32), f64> = BTreeMap::new();
        let mut front_friction_n: BTreeMap<(u32, u32), f64> = BTreeMap::new();
        let mut front_m: BTreeMap<(u32, u32), f64> = BTreeMap::new();
        for e in &edges {
            let front_q = e.front.map(|f| plate_of[f.u as usize]);
            for (&q, &(f, _)) in e.others[..usize::from(e.n_others)].iter().zip(&e.friction) {
                if f > 0.0 {
                    *friction_n.entry((e.plate, q)).or_default() += f;
                    if Some(q) == front_q {
                        *front_friction_n.entry((e.plate, q)).or_default() += f;
                    }
                }
            }
            if let Some(sf) = e.front
                && samples[sf.u as usize].kind != CONTINENTAL
            {
                *front_m
                    .entry((e.plate, plate_of[sf.u as usize]))
                    .or_default() += sf.len_m;
            }
        }
        (friction_n, front_friction_n, front_m)
    };

    // Slabs: grow by the oceanic front's convergence, break off when continental crust has held
    // the trench for `slab_breakoff_myr`; a slab no front feeds hangs and pulls along the shared
    // boundary for `slab_breakoff_myr` and is then gone.
    let mut fronts: BTreeMap<usize, FrontSum> = BTreeMap::new();
    for e in &edges {
        if let Some(sf) = e.front {
            let f = fronts.entry(sf.contact).or_default();
            if samples[sf.u as usize].kind == CONTINENTAL {
                f.continental += 1;
            } else {
                f.oceanic += 1;
                f.conv_sum += sf.conv;
            }
        }
    }
    let mut by_key: BTreeMap<(u32, u32), SlabTrack> = std::mem::take(slabs)
        .into_iter()
        .map(|s| ((s.under, s.over), s))
        .collect();
    let mut breakoffs = 0u16;
    let mut matured: Vec<[f64; 3]> = Vec::new();
    let mut slab_km: BTreeMap<usize, f64> = BTreeMap::new();
    let mut slabs_km = Vec::new();
    // Slabs with a front this step (fed when it is partly oceanic).
    let mut fronted: BTreeSet<(u32, u32)> = BTreeSet::new();
    let mut fed: BTreeSet<(u32, u32)> = BTreeSet::new();
    // Incipient fronts (PLT-610): the fronts of `incipient_pairs`.
    let mut young: BTreeSet<usize> = BTreeSet::new();
    for (&ci, f) in &fronts {
        let c = &st.contacts[ci];
        let key = (c.under, c.over);
        fronted.insert(key);
        let s = by_key.entry(key).or_insert(SlabTrack {
            under: c.under,
            over: c.over,
            ..SlabTrack::default()
        });
        // A split's copy holds the slab from its first front on (ledger: PLT-534).
        s.latent = false;
        if incipient_pairs.contains(&key) {
            young.insert(ci);
        }
        if grow_slab(s, f, env.dt_myr, k.slab_depth_km, k.slab_breakoff_myr) {
            breakoffs += 1;
        }
        // Measurement only (PLT-683): the contact's slab passes the critical convergence.
        if young.contains(&ci) && !incipient(k, s.km) && f.oceanic > 0 {
            let r1 = |x: f64| (x * 10.0).round() / 10.0;
            matured.push([
                r1(f.conv_sum / f64::from(f.oceanic)),
                r1(env.t_myr - c.onset_myr),
                f64::from(f.oceanic),
            ]);
        }
        if f.oceanic > 0 {
            s.idle_myr = 0.0;
            fed.insert(key);
            slabs_km.push((s.km * 10.0).round() / 10.0);
        }
        slab_km.insert(ci, s.km);
    }
    for (key, s) in &mut by_key {
        if !fronted.contains(key) {
            s.idle_myr += env.dt_myr; // ledger: PLT-271
        }
    }
    by_key.retain(|_, s| s.km > 0.0 && s.idle_myr < k.slab_breakoff_myr - 1e-9);
    // Slabs that pull along the boundary this step without a front (a split's copy that has had
    // no front yet does not, ledger: PLT-534).
    let hang_km: BTreeMap<(u32, u32), f64> = by_key
        .iter()
        .filter(|(key, s)| !fronted.contains(key) && !s.latent)
        .map(|(&key, s)| (key, s.km))
        .collect();
    *slabs = by_key.into_values().collect();

    // The system.
    let centroids: Vec<DVec3> = st
        .plates
        .iter()
        .map(|p| DVec3::from_array(p.centroid))
        .collect();
    let mut bal = Balance::new(c, &centroids);
    // Basal drag and the plates' mean column energy, in fixed chunks (a reproducible sum).
    const CHUNK: usize = 4096;
    // Per chunk: each plate's sums, and its column energies' sum and count.
    type Chunk = (Vec<Acc>, Vec<(f64, u32)>);
    let parts: Vec<Chunk> = (0..n.div_ceil(CHUNK))
        .into_par_iter()
        .map(|b| {
            let mut acc = vec![Acc::default(); np];
            let mut energy = vec![(0.0, 0u32); np];
            for i in b * CHUNK..((b + 1) * CHUNK).min(n) {
                let s = &samples[i];
                if s.is_hidden() {
                    continue;
                }
                let p = plate_of[i] as usize;
                let cont = s.kind == CONTINENTAL;
                bal.area_into(&mut acc[p], p, pos[i], cont);
                energy[p].0 += c.energy(f64::from(s.elevation_m), cont);
                energy[p].1 += 1;
            }
            (acc, energy)
        })
        .collect();
    let mut mean_u = vec![(0.0, 0u32); np];
    for (acc, energy) in &parts {
        for p in 0..np {
            merge(&mut bal.acc[p], &acc[p]);
            mean_u[p].0 += energy[p].0;
            mean_u[p].1 += energy[p].1;
        }
    }
    let mean_u: Vec<f64> = mean_u
        .iter()
        .map(|&(s, m)| if m > 0 { s / f64::from(m) } else { 0.0 })
        .collect();

    debug_assert_eq!(ell, c.spacing_m);
    // Measurement only: the oceanic fronts' length the slab pull acts along, km (ledger: PLT-554).
    let mut slab_front_km = [0.0_f64; 3];
    // The collision drag's convergence floor (design §3.4 linearises with the weld speed; it sits
    // under it, so a held collision stalls).
    let floor = c.floor_mm_yr; // ledger: FRC-011
    let mut collision_n: BTreeMap<usize, f64> = BTreeMap::new();
    let mut incipient_n: BTreeMap<usize, f64> = BTreeMap::new();
    for e in &edges {
        let v = e.v as usize;
        let sv = &samples[v];
        let x = pos[v];
        let p = e.plate;
        let mut collision = e.incoming;
        // The slab under this sample, at its front or hanging with no front: the subducting
        // plate, the neighbour, the direction towards it, the slab's length.
        let slab = match e.front {
            Some(sf) => {
                let (u, ci) = (sf.u, sf.contact);
                let su = &samples[u as usize];
                let under = st.contacts[ci].under;
                if su.kind == CONTINENTAL {
                    // The belt's weight against the convergence (design §3.4) …
                    collision = true;
                    let du = c.energy(f64::from(sv.elevation_m), sv.kind == CONTINENTAL)
                        - c.energy(f64::from(su.elevation_m), true);
                    let shortened = st.contacts[ci]
                        .collision
                        .as_ref()
                        .map_or(0.0, |col| col.shortening_km);
                    let force = (c.edge_scale * du.max(0.0) // ledger: FRC-018
                        + c.buoyancy
                            * collision_buoyancy_n_m(
                                c.pull_n_m,
                                c.slab_depth_km,
                                f64::from(su.thickness_m),
                                shortened,
                            ))
                        * sf.len_m;
                    let speed = sf.conv.max(floor) * 1.0e-3 / YEAR_S;
                    if force > 0.0 {
                        *collision_n.entry(ci).or_default() += force;
                        bal.add_link(Link {
                            a: under,
                            b: p,
                            x,
                            n: Some(-sf.dir),
                            k: force / speed,
                            term: Resist::Collision,
                        });
                    }
                    None
                } else {
                    // An incipient front resists its convergence with the bending's plastic
                    // force until its slab sinks by itself (PLT-610), linearised as the
                    // collision's; booked with the collision resistance. A front sample
                    // converging below the slab's thermal floor is incipient whatever its
                    // contact's slab (PLT-681).
                    let slow = below_floor(k, sf.conv);
                    let incipient_here = young.contains(&ci) || slow;
                    if incipient_here {
                        let force = incipient_resistance_n_m(k, su.age_myr(env.t_myr), env.gravity)
                            * sf.len_m;
                        let speed = sf.conv.max(floor) * 1.0e-3 / YEAR_S;
                        *incipient_n.entry(ci).or_default() += force;
                        bal.add_link(Link {
                            a: under,
                            b: p,
                            x,
                            n: Some(-sf.dir),
                            k: force / speed,
                            term: Resist::Collision,
                        });
                    }
                    // Gurnis et al. 2004's resisting force is net, the young slab's own weight
                    // in it: an incipient front has no pull, suction or slab resistance of its
                    // own (PLT-680), nor a sample too slow for a slab (PLT-681).
                    if slow || (young.contains(&ci) && k.incipient_net_resistance) {
                        None
                    } else {
                        let km = slab_km.get(&ci).copied().unwrap_or(0.0);
                        Some((under, u, sf.dir, sf.len_m, km))
                    }
                }
            }
            None => e.hang.and_then(|(u, dir, len_m)| {
                let under = plate_of[u as usize];
                (samples[u as usize].kind != CONTINENTAL)
                    .then(|| {
                        hang_km
                            .get(&(under, p))
                            .map(|&km| (under, u, dir, len_m, km))
                    })
                    .flatten()
            }),
        };
        if let Some((under, u, dir, len_m, len)) = slab {
            let su = &samples[u as usize];
            let towards = -dir;
            let per_m = c.pull_n_m
                * slab_weight(su.age_myr(env.t_myr))
                * (len / c.slab_depth_km).clamp(0.0, 1.0);
            let pull = per_m * len_m;
            if pull > 0.0 {
                bal.add_force(under, x, towards * pull, Drive::SlabPull);
                bal.add_suction(p, x, dir, pull, len_m);
            }
            // Measurement only (ledger: PLT-554): at a front that pulls, one spacing for the
            // sample, the part of it along the boundary's normal (by the converging lattice
            // edge's direction, the old law's), and its share of the straight front (PLT-530), so
            // the pull's count can be read against the front's length.
            if per_m > 0.0
                && let Some(sf) = e.front
            {
                let km = ell / 1000.0;
                slab_front_km[0] += km;
                slab_front_km[1] += km * sf.edge_dir.dot(sf.normal);
                slab_front_km[2] += norm.km(sf.share_rad);
            }
            if len_m > 0.0 {
                bal.add_link(Link {
                    a: under,
                    b: MANTLE,
                    x,
                    n: Some(towards),
                    k: c.slab_res * len_m,
                    term: Resist::Slab,
                });
            }
        }
        if !collision {
            let out = e.out.normalize_or_zero();
            if out != DVec3::ZERO {
                let du = c.energy(f64::from(sv.elevation_m), sv.kind == CONTINENTAL)
                    - mean_u[p as usize];
                let f = c.edge_scale * du * ell;
                if f != 0.0 {
                    bal.add_force(p, x, -out * f, Drive::Edge);
                }
            }
        }
        let m = usize::from(e.n_others);
        if c.boundary_res > 0.0 {
            for &q in &e.others[..m] {
                bal.add_link(Link {
                    a: p,
                    b: q,
                    x,
                    n: None,
                    k: c.boundary_res * ell * 0.5 / m as f64,
                    term: Resist::Boundary,
                });
            }
        }
        for (&q, &(force, rel)) in e.others[..m].iter().zip(&e.friction) {
            bal.add_friction(p, q, x, force, rel);
        }
    }

    let (answers, solved) = match bal.solve() {
        Some(w) => (bal.answers(&w), true),
        None => (vec![None; np], false),
    };
    let unit = TORQUE_UNIT_N_M;
    let mut forces = Vec::new();
    let mut out = vec![None; np];
    let mut stress = vec![None; np];
    for p in st.alive() {
        let Some(a) = answers.get(p.id as usize).copied().flatten() else {
            continue;
        };
        out[p.id as usize] = Some(a.omega);
        stress[p.id as usize] = Some(a.stress_mpa);
        forces.push(PlateForces {
            id: p.id,
            slab_pull: sig4(a.slab_pull / unit),
            slab_suction: sig4(a.slab_suction / unit),
            edge_energy: sig4(a.edge_energy / unit),
            mantle_flow: sig4(a.mantle_flow / unit),
            basal_drag: sig4(a.basal_drag / unit),
            slab_resistance: sig4(a.slab_resistance / unit),
            boundary_resistance: sig4(a.boundary_resistance / unit),
            collision_resistance: sig4(a.collision_resistance / unit),
            stress_mpa: (a.stress_mpa * 100.0).round() / 100.0,
            speed_mm_yr: (a.speed_mm_yr * 10.0).round() / 10.0,
            rms_mm_yr: (a.rms_mm_yr * 10.0).round() / 10.0,
            rot_change: 0.0,
            rot_known: false,
        });
    }
    #[cfg(test)]
    let terms = BoundaryTerms {
        friction_n,
        front_friction_n,
        front_m,
        incipient_n: incipient_n.clone(),
        colliding_n_m: bal.c.colliding_n_m,
        transform_n_m: bal.c.transform_n_m,
    };
    Balanced {
        omega: out,
        stress_mpa: stress,
        collision_n,
        incipient_n,
        solved,
        out: Recorded {
            forces,
            slabs_km,
            breakoffs,
            matured,
        },
        slab_front_km,
        #[cfg(test)]
        terms,
        #[cfg(test)]
        torques: bal.acc.iter().map(|a| (a.t_pull, a.t_suck)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on_sphere(lon_deg: f64, lat_deg: f64) -> DVec3 {
        let (lo, la) = (lon_deg.to_radians(), lat_deg.to_radians());
        DVec3::new(la.cos() * lo.cos(), la.cos() * lo.sin(), la.sin())
    }

    fn coeffs(k: &TectonicParams) -> Coeffs {
        let norm = Norm::new(6370.0, 782.0, 100_000);
        // 14 starting plates.
        let a0 = 4.0 * std::f64::consts::PI * 6370.0f64.powi(2) / 14.0;
        Coeffs::new(k, &norm, 9.81, a0, DVec3::Z, 0.0)
    }

    /// A spherical cap of angular radius `r_deg` about the north pole, sampled on a lattice of
    /// about one sample spacing: the cap's samples and its rim (point, outward direction).
    fn cap(c: &Coeffs, r_deg: f64) -> (Vec<DVec3>, Vec<(DVec3, DVec3)>) {
        let step = (c.spacing_m / c.radius_m).to_degrees();
        let mut pts = Vec::new();
        let mut colat = 0.5 * step;
        while colat < r_deg {
            let ring = (2.0 * std::f64::consts::PI * colat.to_radians().sin() / step.to_radians())
                .round()
                .max(1.0) as usize;
            for j in 0..ring {
                pts.push(on_sphere(360.0 * j as f64 / ring as f64, 90.0 - colat));
            }
            colat += step;
        }
        let ring = (2.0 * std::f64::consts::PI * r_deg.to_radians().sin() / step.to_radians())
            .round() as usize;
        let rim = (0..ring)
            .map(|j| {
                let lon = 360.0 * j as f64 / ring as f64;
                let x = on_sphere(lon, 90.0 - r_deg);
                // Outward: towards the south, the tangent of decreasing latitude.
                let south = tangent(x, -DVec3::Z).normalize();
                (x, south)
            })
            .collect();
        (pts, rim)
    }

    fn quiet() -> TectonicParams {
        TectonicParams {
            mantle_flow_mm_yr: 0.0,
            ..TectonicParams::default()
        }
    }

    /// Forsyth & Uyeda's balance (PLT-585, off by default), for its own tests.
    fn fu() -> TectonicParams {
        TectonicParams {
            forsyth_uyeda_balance: true,
            ..quiet()
        }
    }

    /// The old law, for its own tests: the speed scale and the viscous resistances (the
    /// 2026-10-02 calibration, FRC-016), the suction saturating (PLT-540).
    fn old_law() -> TectonicParams {
        TectonicParams {
            forsyth_uyeda_balance: false,
            suction_saturates: true,
            ..quiet()
        }
    }

    /// The reference plate of FRC-016: a cap of the 14 starting plates' mean area (its area, m²,
    /// its samples and its rim).
    #[allow(clippy::type_complexity)]
    fn reference_cap(c: &Coeffs) -> (f64, Vec<DVec3>, Vec<(DVec3, DVec3)>) {
        let area = 4.0 * std::f64::consts::PI * c.radius_m.powi(2) / 14.0;
        let r_deg = (1.0 - area / (2.0 * std::f64::consts::PI * c.radius_m.powi(2)))
            .acos()
            .to_degrees();
        let (pts, rim) = cap(c, r_deg);
        (area, pts, rim)
    }

    /// `base` (plate 0 and everything but friction) solved with friction against the mantle at
    /// the rim points `(x, outward normal, length m)`, classified as the balance step does: by
    /// Forsyth & Uyeda's rule on the plate's motion against the boundary's normal, a transform, a
    /// colliding boundary or, where the plate pulls away, nothing. The friction is linearised
    /// about the last answer until the answer stops changing: the fixed point the plates' motion
    /// relaxes to. The rotation (rad/s), the last system and the solves it took.
    fn plastic(base: &Balance, rim: &[(DVec3, DVec3, f64)]) -> (DVec3, Balance, usize) {
        let c = base.c;
        let mut w = base.solve().expect("solvable")[0];
        for it in 1..=200 {
            let mut bal = base.clone();
            for &(x, out, len) in rim {
                // The mantle side's motion relative to the plate, mm/yr.
                let rel = -(w.cross(x) * c.radius_m * 1.0e3 * YEAR_S);
                let per_m = if fu_transform(rel, out) {
                    c.transform_n_m
                } else if rel.dot(out) < 0.0 {
                    c.colliding_n_m
                } else {
                    0.0
                };
                bal.add_friction(0, MANTLE, x, per_m * len, rel.length());
            }
            let next = bal.solve().expect("solvable")[0];
            let change = (next - w).length() / next.length().max(1.0e-30);
            w = next;
            if change < 1.0e-10 {
                return (w, bal, it);
            }
        }
        panic!("the friction found no fixed point");
    }

    /// With `forsyth_uyeda_balance` (PLT-585) the resistances are Forsyth & Uyeda's (1975, Table
    /// 6) against the reference pull, whatever the old law's settings: the slab's terminal
    /// velocity F_SP / F_SR (7.2 cm/yr; "the terminal velocity computed to be 7.0 cm/yr", p. 189),
    /// the drag under continents "approximately eight times" the oceanic (ibid.), the suction and
    /// the two frictions their shares of the pull, no viscous boundary resistance; on FRC-016's
    /// reference plate the slab resistance is 8.9 times the basal drag.
    #[test]
    fn the_resistances_are_forsyth_and_uyedas_table_6() {
        let k = fu();
        let c = coeffs(&k);
        // One switch: the saturating suction (PLT-540, on by default) is not read with it.
        assert!(k.suction_saturates && !c.suction_saturates);
        let v_t = c.pull_n_m / c.slab_res * 1.0e3 * YEAR_S;
        assert!(
            (v_t / (FU_SLAB_PULL / FU_SLAB_RESISTANCE * 10.0) - 1.0).abs() < 1e-12,
            "{v_t}"
        );
        assert!(
            (v_t / 70.0 - 1.0).abs() < 0.05,
            "the paper's 7.0 cm/yr: {v_t}"
        );
        assert!((7.5..8.5).contains(&c.keel), "{}", c.keel);
        assert!((c.suction_share - 0.50 / 6.43).abs() < 1e-12);
        assert_eq!(c.boundary_res, 0.0);
        assert!((c.transform_n_m / c.pull_n_m - 0.36 / 6.43).abs() < 1e-12);
        assert!((c.colliding_n_m / c.pull_n_m - 0.16 / 6.43).abs() < 1e-12);
        // Per km of trench against per km² of base, both per cm/yr: 0.89 against 0.82e-5.
        let ratio = c.slab_res / (c.drag * 1.0e3);
        assert!((ratio / (0.89 / 0.82e-5) - 1.0).abs() < 1e-12, "{ratio}");
        let area = 4.0 * std::f64::consts::PI * c.radius_m.powi(2) / 14.0;
        let s = c.slab_res * REF_TRENCH_KM * 1000.0 / (c.drag * area);
        assert!((8.8..9.1).contains(&s), "{s}");
        // None of the old law's settings is read.
        let other = coeffs(&TectonicParams {
            plate_speed_scale_mm_yr: 90.0,
            slab_resistance: 10.0,
            boundary_resistance: 7.0,
            keel_drag: 1.0,
            slab_suction_share: 0.9,
            ..fu()
        });
        assert_eq!(
            (other.drag, other.keel, other.slab_res, other.boundary_res),
            (c.drag, c.keel, c.slab_res, c.boundary_res)
        );
        assert_eq!(
            (
                other.suction_share,
                other.transform_n_m,
                other.colliding_n_m
            ),
            (c.suction_share, c.transform_n_m, c.colliding_n_m)
        );
        // The old law has no friction.
        let old = coeffs(&old_law());
        assert_eq!((old.transform_n_m, old.colliding_n_m), (0.0, 0.0));
        assert!(old.boundary_res > 0.0 && old.suction_share == 0.3);
    }

    /// FRC-016's reference plate (the starting plates' mean area, 3,000 km of trench of a
    /// full-length 80-Myr slab, its neighbours at rest) under Forsyth & Uyeda's forces moves near
    /// its slab's terminal velocity, as their oceanic plates attached to a slab do ("roughly
    /// coincident with the average velocity of oceanic plates attached to a slab", p. 189): the
    /// basal drag takes 1 / (1 + 8.9) of it, 65 mm/yr, and the friction on its transforms and its
    /// colliding front (the trench's own included) about a sixth more (56 mm/yr).
    /// At the fixed point the friction is a force whatever the speed: the same plate with fixed
    /// forces in its place has the same answer.
    #[test]
    fn the_reference_plate_moves_near_its_slabs_terminal_velocity() {
        let c = coeffs(&fu());
        let (area, pts, rim) = reference_cap(&c);
        let mut base = Balance::new(c, &[DVec3::Z]);
        for &x in &pts {
            base.add_area(0, x, false);
        }
        let trench_m = REF_TRENCH_KM * 1000.0;
        let mut rim_sorted = rim.clone();
        rim_sorted.sort_by(|a, b| b.0.x.total_cmp(&a.0.x));
        let mut len = 0.0;
        for &(x, _) in &rim_sorted {
            if len >= trench_m {
                break;
            }
            let f = tangent(x, DVec3::X).normalize();
            base.add_force(0, x, f * c.pull_n_m * c.spacing_m, Drive::SlabPull);
            base.add_link(Link {
                a: 0,
                b: MANTLE,
                x,
                n: Some(f),
                k: c.slab_res * c.spacing_m,
                term: Resist::Slab,
            });
            len += c.spacing_m;
        }
        let v_t = c.pull_n_m / c.slab_res * 1.0e3 * YEAR_S;
        let s = c.slab_res * len / (c.drag * area);
        let speed = |w: DVec3| w.cross(DVec3::Z).length() * c.radius_m * 1.0e3 * YEAR_S;
        let free = speed(base.solve().unwrap()[0]);
        let expect = v_t * s / (1.0 + s);
        assert!(
            (free / expect - 1.0).abs() < 0.05,
            "{free} against {expect}"
        );
        // Friction all round the rim.
        let rim_m =
            2.0 * std::f64::consts::PI * c.radius_m * rim[0].0.z.acos().sin() / rim.len() as f64;
        let rim3: Vec<_> = rim.iter().map(|&(x, out)| (x, out, rim_m)).collect();
        let (w, bal, its) = plastic(&base, &rim3);
        let v = speed(w);
        eprintln!(
            "the reference plate: v_t {v_t:.1}, s {s:.2}, frictionless {free:.1} ({expect:.1} \
             flat), with friction {v:.1} mm/yr after {its} solves"
        );
        assert!((0.75..0.95).contains(&(v / free)), "{v} against {free}");
        assert!(v > 0.7 * v_t, "{v}");
        // Each link resists with its force: the friction as fixed forces against the motion, the
        // transform's or the colliding resistance times the length, gives the same answer.
        let mut fixed = base.clone();
        let (mut transform, mut colliding) = (0, 0);
        for &(x, out, len) in &rim3 {
            let dv = w.cross(x) * c.radius_m * 1.0e3 * YEAR_S;
            assert!(dv.length() > c.floor_mm_yr, "{dv}");
            let per_m = if fu_transform(-dv, out) {
                transform += 1;
                c.transform_n_m
            } else if dv.dot(out) > 0.0 {
                colliding += 1;
                c.colliding_n_m
            } else {
                0.0
            };
            fixed.add_force(0, x, -dv.normalize() * (per_m * len), Drive::Edge);
        }
        let links = bal.links.iter().filter(|l| l.term == Resist::Friction);
        assert_eq!(links.count(), transform + colliding);
        // Forsyth & Uyeda's 60 degrees: on a flat plate 120 degrees of the rim are transforms
        // and 120 the colliding front; on the cap the rim's motion leans towards its normal (84
        // and 103 of the rim's samples).
        let share = transform as f64 / (transform + colliding) as f64;
        assert!((share - 84.0 / 187.0).abs() < 0.03, "{share}");
        let wf = fixed.solve().unwrap()[0];
        assert!((wf - w).length() < 1e-6 * w.length(), "{wf} {w}");
        // The budget books the friction with the boundary resistance.
        let ans = bal.answers(&[w])[0].unwrap();
        assert!(
            ans.boundary_resistance > 0.0 && ans.residual < 1e-6,
            "{ans:?}"
        );
    }

    /// Friction is plastic (FRC-035): a drive that outweighs it moves the plate as a fixed force
    /// against the motion would, and one that does not leaves the boundary slipping under the
    /// floor speed.
    #[test]
    fn friction_resists_with_its_force_whatever_the_speed() {
        let c = coeffs(&fu());
        let (pts, _) = cap(&c, 25.0);
        let x0 = on_sphere(0.0, 65.0);
        let dir = tangent(x0, DVec3::X).normalize();
        let pull = c.pull_n_m * 1.0e6;
        let mut base = Balance::new(c, &[DVec3::Z]);
        for &x in &pts {
            base.add_area(0, x, false);
        }
        base.add_force(0, x0, dir * pull, Drive::SlabPull);
        let at = |w: DVec3| w.cross(x0) * c.radius_m * 1.0e3 * YEAR_S;
        let iterate = |force: f64| {
            let mut w = base.solve().unwrap()[0];
            for _ in 0..100 {
                let mut bal = base.clone();
                bal.add_friction(0, MANTLE, x0, force, at(w).length());
                w = bal.solve().unwrap()[0];
            }
            w
        };
        let free = at(base.solve().unwrap()[0]).length();
        let w = iterate(0.3 * pull);
        let v = at(w);
        let mut fixed = base.clone();
        fixed.add_force(0, x0, -v.normalize() * (0.3 * pull), Drive::Edge);
        let wf = fixed.solve().unwrap()[0];
        assert!((wf - w).length() < 1e-6 * w.length(), "{wf} {w}");
        assert!(
            (0.6..0.8).contains(&(v.length() / free)),
            "{} {free}",
            v.length()
        );
        let held = at(iterate(2.0 * pull)).length();
        assert!(held > 0.0 && held < c.floor_mm_yr, "{held}");
    }

    /// A plate with no slab, pushed by a ridge along the back of its rim. Without friction,
    /// Table 6's basal drag (half the old law's) and no boundary resistance let it run many times
    /// the old law's speed (the plate speed lane's diagnosis: the plates with no slab pull at 135
    /// and 95 mm/yr RMS under Table 6's resistances alone, 19.5 and 15.5 under the old law). The
    /// friction on its transforms and its colliding front takes most of that away: at Table 6's
    /// own ridge push (0.36 per km against the pull's 6.43) the friction about balances the push
    /// and the plate creeps at the floor speed (Forsyth & Uyeda's plates with no slab are slow:
    /// "continental drag is the primary regulator of the velocity of plates not attached to
    /// downgoing slabs", p. 190); at the model's (the column energy at the pool preset's ridge and old floor, about
    /// 0.1 of the pull) an oceanic plate moves at about a quarter of its frictionless speed,
    /// still faster than under the old law, and a continental one slower than under the old law.
    #[test]
    fn friction_holds_back_a_plate_with_no_slab() {
        let pool = |k: TectonicParams| TectonicParams {
            ridge_elevation_m: -2600.0,
            oceanic_floor_elevation_m: -5651.0,
            ..k
        };
        let c = coeffs(&pool(fu()));
        let old = coeffs(&pool(old_law()));
        let (_, pts, rim) = reference_cap(&c);
        let rim_m =
            2.0 * std::f64::consts::PI * c.radius_m * rim[0].0.z.acos().sin() / rim.len() as f64;
        let model_push = c.edge_scale * (c.energy(-2600.0, false) - c.energy(-5651.0, false));
        assert!(
            (0.08..0.12).contains(&(model_push / c.pull_n_m)),
            "{model_push:e}"
        );
        // Table 6's ridge push, 0.36 per km.
        let table_push = c.pull_n_m * 0.36 / FU_SLAB_PULL;
        let speed = |w: DVec3| w.cross(DVec3::Z).length() * c.radius_m * 1.0e3 * YEAR_S;
        // The ridge: the back of the rim (140 degrees about longitude 180), pushing towards +X.
        let build = |c: Coeffs, push: f64, continental: bool| {
            let mut bal = Balance::new(c, &[DVec3::Z]);
            for &x in &pts {
                bal.add_area(0, x, continental);
            }
            for &(x, out) in &rim {
                if x.y.atan2(x.x).to_degrees().abs() > 110.0 {
                    bal.add_force(0, x, -out * (push * rim_m), Drive::Edge);
                }
                if c.boundary_res > 0.0 {
                    bal.add_link(Link {
                        a: 0,
                        b: MANTLE,
                        x,
                        n: None,
                        k: c.boundary_res * rim_m,
                        term: Resist::Boundary,
                    });
                }
            }
            bal
        };
        let rim3: Vec<_> = rim.iter().map(|&(x, out)| (x, out, rim_m)).collect();
        let old_v = speed(build(old, model_push, false).solve().unwrap()[0]);
        let frictionless = speed(build(c, model_push, false).solve().unwrap()[0]);
        let (w, _, _) = plastic(&build(c, model_push, false), &rim3);
        let with = speed(w);
        let (w, _, _) = plastic(&build(c, table_push, false), &rim3);
        let held = speed(w);
        let old_cont = speed(build(old, model_push, true).solve().unwrap()[0]);
        let (w, _, _) = plastic(&build(c, model_push, true), &rim3);
        let with_cont = speed(w);
        eprintln!(
            "a plate with no slab: the old law {old_v:.1}, Table 6 without friction \
             {frictionless:.1}, with friction {with:.1}; at Table 6's ridge push {held:.2} mm/yr; \
             continental: the old law {old_cont:.1}, with friction {with_cont:.1}"
        );
        assert!(frictionless > 4.0 * old_v, "{frictionless} {old_v}");
        assert!(
            (0.2..0.35).contains(&(with / frictionless)),
            "{with} {frictionless}"
        );
        // Held at the floor: the friction is the push to within a few percent on this cap.
        assert!(
            held < 1.5 * c.floor_mm_yr && held < 0.02 * frictionless,
            "{held}"
        );
        assert!(with_cont < old_cont, "{with_cont} {old_cont}");
    }

    #[test]
    fn slab_weight_grows_with_the_root_of_age_and_young_crust_floats() {
        assert_eq!(slab_weight(5.0), 0.0);
        assert!((slab_weight(80.0) - 1.0).abs() < 1e-12);
        assert!((slab_weight(20.0) - 0.5).abs() < 1e-12);
        assert!((slab_weight(400.0) - 1.5).abs() < 1e-12);
    }

    /// Law 3 (PLT-657) against its source, Gurnis, Hall & Lavier 2004: the power law runs
    /// through [46]'s two points (a factor of 2.2 from an elastic thickness of 8.5 km to 15 km),
    /// it is Case 1's 3 × 10¹² N/m at 30 Myr ([59]), it grows with age to the model's 40 km cap
    /// (ISO-049, from 219 Myr) and stays inside [67]'s "between 10^12 and 10^13 N/m" for the
    /// floor that sinks, up to 180 Myr. The ridge (PLT-613) and gravity scaling are unchanged;
    /// law 2's margin strength (PLT-658) is PLT-517's force scaled the same way, constant with
    /// law 3 off.
    #[test]
    fn law_3_scales_the_incipient_resistance_with_the_elastic_thickness() {
        let n = incipient_thickness_exponent();
        assert!(
            ((INCIPIENT_THICK_KM / INCIPIENT_THIN_KM).powf(n) - INCIPIENT_THICKNESS_FACTOR).abs()
                < 1e-12
        );
        assert!((n - 1.388).abs() < 0.001, "{n}");
        assert!((thickness_scale(30.0) - 1.0).abs() < 1e-12);
        let mut last = 0.0;
        for age in [10.0, 20.0, 30.0, 80.0, 180.0, 219.0] {
            let s = thickness_scale(age);
            assert!(s > last, "not growing at {age} Myr: {s}");
            last = s;
        }
        // The model's Te at 10 Myr is √(10/30) of 30 Myr's.
        assert!((thickness_scale(10.0) - (10.0f64 / 30.0).sqrt().powf(n)).abs() < 1e-12);
        assert!((thickness_scale(400.0) - thickness_scale(250.0)).abs() < 1e-12);
        let on = TectonicParams {
            incipient_resistance_by_thickness: true,
            ..TectonicParams::default()
        };
        let off = TectonicParams {
            incipient_resistance_by_thickness: false,
            ..on.clone()
        };
        let g = G_EARTH;
        assert!((incipient_resistance_n_m(&on, 30.0, g) - 3.0e12).abs() < 1.0);
        for age in [10.0, 80.0, 180.0] {
            let f = incipient_resistance_n_m(&on, age, g);
            assert!((1.0e12..=1.1e13).contains(&f), "{age} Myr: {f:e}");
            assert_eq!(
                incipient_resistance_n_m(&off, age, g),
                INCIPIENT_RESISTANCE_N_M
            );
            assert!((incipient_resistance_n_m(&on, age, 0.5 * g) - 0.5 * f).abs() < 1.0);
        }
        assert_eq!(incipient_resistance_n_m(&on, 5.0, g), INCIPIENT_RIDGE_N_M);
        // Law 2's margin strength: PLT-517 at 30 Myr, Case 1's growth with age, constant off.
        let t = super::super::collision::TRANSFERENCE_FORCE_N_M;
        assert!((margin_strength_n_m(&on, 30.0, g) - t).abs() < 1.0);
        assert!(margin_strength_n_m(&on, 120.0, g) > margin_strength_n_m(&on, 40.0, g));
        assert_eq!(margin_strength_n_m(&off, 120.0, g), t);
        assert_eq!(
            margin_strength_n_m(&on, 2.0, g),
            margin_strength_n_m(&on, 10.0, g)
        );
        // Law 2's pull: FRC-002 at 80 Myr with a full slab, along the margin's normal only.
        let full = on.slab_depth_km;
        assert!((margin_pull_n_m(&on, full, 80.0, 1.0, g) - SLAB_PULL_N_M).abs() < 1.0);
        assert!((margin_pull_n_m(&on, full / 2.0, 80.0, 0.5, g) - SLAB_PULL_N_M / 4.0).abs() < 1.0);
        assert_eq!(margin_pull_n_m(&on, full, 80.0, -0.3, g), 0.0);
        assert_eq!(margin_pull_n_m(&on, full, 5.0, 1.0, g), 0.0);
    }

    #[test]
    fn the_column_energy_gives_ridge_push_and_plateau_weight_of_earths_size() {
        let g = 9.81;
        // Ridge push between a crest at -2.5 km and 100-Myr floor 3.5 km lower: about 4e12 N/m
        // (Turcotte & Schubert's formula at 100 Myr; 2-3.5e12 quoted, design §3.2).
        let push = column_energy(-2500.0, false, -2500.0, -6000.0, 0.0, g)
            - column_energy(-6000.0, false, -2500.0, -6000.0, 0.0, g);
        assert!((3.5e12..4.5e12).contains(&push), "{push:e}");
        // A trench is no deeper in energy than the old floor.
        assert_eq!(
            column_energy(-9000.0, false, -2500.0, -6000.0, 0.0, g),
            column_energy(-6000.0, false, -2500.0, -6000.0, 0.0, g)
        );
        // A 5 km belt: 6.7e12 N/m (design §3.2).
        let belt = column_energy(5000.0, true, -2500.0, -6000.0, 0.0, g);
        assert!((6.6e12..6.8e12).contains(&belt), "{belt:e}");
        // Sea level, the ridge crest: the shared reference.
        assert_eq!(column_energy(0.0, true, -2500.0, -6000.0, 0.0, g), 0.0);
        assert_eq!(column_energy(-2500.0, false, -2500.0, -6000.0, 0.0, g), 0.0);
        // Everything scales with gravity.
        let mars = column_energy(5000.0, true, -2500.0, -6000.0, 0.0, 3.71);
        assert!((mars / belt - 3.71 / 9.81).abs() < 1e-12);
    }

    #[test]
    fn the_land_s_column_energy_is_taken_from_the_sea() {
        let g = 9.81;
        let u = |z: f64, cont: bool, sea: f64| column_energy(z, cont, -2600.0, -5651.0, sea, g);
        for sea in [-800.0, -250.0, 300.0, 450.0] {
            // Crust standing at the sea is the reference, as the ridge crest is.
            assert_eq!(u(sea, true, sea), 0.0);
            // Two dry columns differ by the same energy wherever the sea stands (the crust that
            // stands at the sea is the Airy thickness of a column there).
            let (lo, hi) = (sea.max(0.0) + 200.0, 4000.0);
            let at_datum = u(hi, true, 0.0) - u(lo, true, 0.0);
            let here = u(hi, true, sea) - u(lo, true, sea);
            assert!(
                ((here - at_datum) / at_datum).abs() < 1e-9,
                "{sea}: {here:e} {at_datum:e}"
            );
            // The sea floor's energy is the ridge's, whatever the sea.
            assert_eq!(u(-4000.0, false, sea), u(-4000.0, false, 0.0));
        }
        // Ground between the datum and a higher sea is under water: it weighs less than the crust
        // standing at the sea, where the datum's formula had it above.
        assert!(u(100.0, true, 300.0) < 0.0);
        assert!(u(100.0, true, 0.0) > 0.0);
        // Ground between a lower sea and the datum is dry land above the sea.
        assert!(u(-100.0, true, -250.0) > 0.0);
        assert!(u(-100.0, true, 0.0) < 0.0);
    }

    #[test]
    fn the_flow_leaves_the_upwellings_and_peaks_at_45_degrees() {
        let a = DVec3::X;
        assert!(flow_velocity(a, DVec3::X).length() < 1e-12);
        assert!(flow_velocity(a, DVec3::Y).length() < 1e-12);
        let x = on_sphere(45.0, 0.0);
        let v = flow_velocity(a, x);
        assert!((v.length() - 1.0).abs() < 1e-12);
        assert!(v.dot(x).abs() < 1e-12, "tangent");
        // Away from the upwelling at +X: towards +Y.
        assert!(v.y > 0.0);
        assert!((flow_axis(7).length() - 1.0).abs() < 1e-12);
        assert_eq!(flow_axis(7), flow_axis(7));
        assert_ne!(flow_axis(7), flow_axis(8));
    }

    #[test]
    fn the_reference_plate_moves_at_the_speed_scale() {
        let k = old_law();
        let c = coeffs(&k);
        // A flat-ish cap of the reference area pulled by a straight trench across its rim: drag
        // (1 + slab resistance + boundary resistance) c_b A v = F L, so v is about the speed scale
        // (`plate_speed_scale_mm_yr`) when the trench is one slab of 80 Myr at full length.
        let area = 4.0 * std::f64::consts::PI * c.radius_m.powi(2) / 14.0;
        let r_deg = (1.0 - area / (2.0 * std::f64::consts::PI * c.radius_m.powi(2)))
            .acos()
            .to_degrees();
        let (pts, rim) = cap(&c, r_deg);
        let mut bal = Balance::new(c, &[DVec3::Z, -DVec3::Z]);
        for &x in &pts {
            bal.add_area(0, x, false);
        }
        // The trench: rim points within ±(half the trench) of longitude 0, pulled towards +X.
        let trench_m = REF_TRENCH_KM * 1000.0;
        let pull_dir = DVec3::X;
        let mut len = 0.0;
        let mut rim_sorted = rim.clone();
        rim_sorted.sort_by(|a, b| b.0.x.total_cmp(&a.0.x));
        for &(x, _) in &rim_sorted {
            if len >= trench_m {
                break;
            }
            let f = tangent(x, pull_dir).normalize();
            bal.add_force(0, x, f * c.pull_n_m * c.spacing_m, Drive::SlabPull);
            bal.add_link(Link {
                a: 0,
                b: MANTLE,
                x,
                n: Some(f),
                k: c.slab_res * c.spacing_m,
                term: Resist::Slab,
            });
            len += c.spacing_m;
        }
        // Boundary resistance against the mantle frame all round (neighbours at rest).
        for &(x, _) in &rim {
            bal.add_link(Link {
                a: 0,
                b: MANTLE,
                x,
                n: None,
                k: c.boundary_res * c.spacing_m,
                term: Resist::Boundary,
            });
        }
        let w = bal.solve().expect("solvable");
        let ans = bal.answers(&w)[0].expect("plate 0");
        eprintln!(
            "the reference plate on the ring moves at {:.1} mm/yr (the speed scale {:.0})",
            ans.speed_mm_yr, k.plate_speed_scale_mm_yr
        );
        // The sphere's cap is not flat, so the factor of the reference only holds roughly (49.5
        // mm/yr at 45, 82.6 at 75).
        let s = k.plate_speed_scale_mm_yr;
        assert!(
            (0.55 * s..1.55 * s).contains(&ans.speed_mm_yr),
            "speed {} at the scale {s}",
            ans.speed_mm_yr
        );
        // It moves towards the trench: the velocity at the centroid points along +X.
        let v = (w[0] * YEAR_S).cross(DVec3::Z);
        assert!(v.x > 0.0 && v.x.abs() > 5.0 * v.y.abs(), "{v:?}");
        // Torque direction: slab pull's torque is about the axis Z × X = +Y.
        let t = bal.acc[0].t_pull;
        assert!(
            t.y > 0.0 && t.y.abs() > 10.0 * t.x.abs().max(t.z.abs()),
            "{t:?}"
        );
        // Magnitude order: F L R, about 3e13 N/m × 3e6 m × 6.4e6 m = 5.7e26 N m.
        let expect = c.pull_n_m * trench_m * c.radius_m;
        assert!(
            (0.5 * expect..1.1 * expect).contains(&ans.slab_pull),
            "{:e} against {expect:e}",
            ans.slab_pull
        );
        // The balance holds.
        assert!(ans.residual < 1e-6, "{}", ans.residual);
        // No mantle flow, no ridge: the plate is pulled at one edge and dragged along its base:
        // a little tension.
        assert!(ans.stress_mpa > 0.0, "{}", ans.stress_mpa);
    }

    #[test]
    fn a_plate_ringed_by_trenches_is_in_tension_and_stands_still() {
        let k = old_law();
        let c = coeffs(&k);
        let (pts, rim) = cap(&c, 30.0);
        let mut bal = Balance::new(c, &[DVec3::Z]);
        for &x in &pts {
            bal.add_area(0, x, true);
        }
        // Suction of trenches all round: every rim point pulled outwards.
        for &(x, out) in &rim {
            bal.add_force(
                0,
                x,
                out * c.pull_n_m * c.suction_share * c.spacing_m,
                Drive::Suction,
            );
            bal.add_link(Link {
                a: 0,
                b: MANTLE,
                x,
                n: None,
                k: c.boundary_res * c.spacing_m,
                term: Resist::Boundary,
            });
        }
        let w = bal.solve().unwrap();
        let ans = bal.answers(&w)[0].unwrap();
        assert!(ans.speed_mm_yr < 0.5, "speed {}", ans.speed_mm_yr);
        assert!(ans.rms_mm_yr < 0.5, "rms {}", ans.rms_mm_yr);
        // Tension: about the suction over the lithosphere, 0.3 × 3e13 / 1e5 m = 90 MPa.
        assert!(
            (30.0..150.0).contains(&ans.stress_mpa),
            "{}",
            ans.stress_mpa
        );
        // The same ring pushing inwards (ridges all round) is compression.
        let mut push = Balance::new(c, &[DVec3::Z]);
        for &x in &pts {
            push.add_area(0, x, true);
        }
        for &(x, out) in &rim {
            push.add_force(0, x, -out * 3.0e12 * c.spacing_m, Drive::Edge);
        }
        let w = push.solve().unwrap();
        let a = push.answers(&w)[0].unwrap();
        assert!(a.stress_mpa < 0.0 && a.speed_mm_yr < 0.5, "{a:?}");
    }

    /// The saturating suction (PLT-540) against the fixed share (PLT-186). A plate drawn only by
    /// the suction of a straight trench, with nothing at its edges or, for the reference plate,
    /// boundary resistance all round. The fixed share's speed `u_f` (the share over the plate's
    /// resistance) becomes `u_f / (1 + u_f / v_t)`, `v_t` the slab's terminal velocity: its pull
    /// over its slab resistance (PLT-185; Forsyth & Uyeda 1975, p. 178: "effectively the terminal
    /// velocity of the dense slab falling in a viscous medium"). So a plate the share
    /// draws slowly keeps most of its speed, and a small plate (Earth's Tonga microplate, Bevis
    /// et al. 1995) that the share would draw several times faster than the slab sinks follows
    /// the slab at most. At rest the suction is the share.
    #[test]
    fn the_suction_draws_no_overriding_plate_faster_than_its_slab_sinks() {
        // (speed towards the trench at the centroid, mm/yr; the suction's torque at rest; the
        // fixed share's torque; v_t, mm/yr)
        let run = |r_deg: f64, trench_km: f64, edges: bool, saturates: bool| {
            let k = TectonicParams {
                suction_saturates: saturates,
                ..old_law()
            };
            let c = coeffs(&k);
            let (pts, rim) = cap(&c, r_deg);
            let mut bal = Balance::new(c, &[DVec3::Z]);
            for &x in &pts {
                bal.add_area(0, x, false);
            }
            // The trench: rim points nearest +X, `trench_km` of them, the slab across it towards +X.
            let pull = c.pull_n_m * c.spacing_m;
            let mut rim_sorted = rim.clone();
            rim_sorted.sort_by(|a, b| b.0.x.total_cmp(&a.0.x));
            let mut fixed = DVec3::ZERO;
            let mut len = 0.0;
            for &(x, _) in &rim_sorted {
                if len >= trench_km * 1000.0 {
                    break;
                }
                let dir = tangent(x, DVec3::X).normalize();
                bal.add_suction(0, x, dir, pull, c.spacing_m);
                fixed += x.cross(dir * pull * c.suction_share) * c.radius_m;
                len += c.spacing_m;
            }
            if edges {
                for &(x, _) in &rim {
                    bal.add_link(Link {
                        a: 0,
                        b: MANTLE,
                        x,
                        n: None,
                        k: c.boundary_res * c.spacing_m,
                        term: Resist::Boundary,
                    });
                }
            }
            let w = bal.solve().expect("solvable");
            let ans = bal.answers(&w)[0].expect("plate 0");
            assert!(ans.residual < 1e-6, "{}", ans.residual);
            let v = w[0].cross(DVec3::Z) * c.radius_m * 1.0e3 * YEAR_S;
            assert!(v.x > 0.0 && v.x > 5.0 * v.y.abs(), "{v:?}");
            let at_rest = bal.answers(&[DVec3::ZERO])[0].unwrap().slab_suction;
            let v_t = c.pull_n_m / c.slab_res * 1.0e3 * YEAR_S;
            (v.x, at_rest, fixed.length(), v_t)
        };
        // A small plate (about 70 samples of the 100k lattice, the size the speed guard held:
        // plates.md, "The speed guard") with a trench along a third of its edge.
        for (r_deg, trench_km, edges, tol) in [(3.0, 700.0, false, 0.03), (31.0, 3000.0, true, 0.1)]
        {
            let (u_f, rest_f, fixed, v_t) = run(r_deg, trench_km, edges, false);
            let (u_s, rest_s, _, v_t2) = run(r_deg, trench_km, edges, true);
            assert_eq!(v_t, v_t2);
            // The speed scale × (1 + 3 + 3) / 3 at Earth's settings: 105 mm/yr at 45, 175 at 75.
            let k = old_law();
            let want = k.plate_speed_scale_mm_yr
                * (1.0 + k.slab_resistance + k.boundary_resistance)
                / k.slab_resistance;
            assert!((v_t / want - 1.0).abs() < 0.05, "{v_t} {want}");
            assert!((rest_f / fixed - 1.0).abs() < 1e-9 && (rest_s / fixed - 1.0).abs() < 1e-9);
            let expect = u_f / (1.0 + u_f / v_t);
            eprintln!(
                "r {r_deg}: fixed {u_f:.1}, saturating {u_s:.1}, expected {expect:.1}, v_t {v_t:.1}"
            );
            assert!(
                (u_s / expect - 1.0).abs() < tol && u_s < v_t,
                "r {r_deg}: fixed {u_f:.1}, saturating {u_s:.1}, expected {expect:.1}, v_t {v_t:.1}"
            );
            if r_deg < 10.0 {
                assert!(
                    u_f > 2.0 * v_t,
                    "the share draws the small plate fast: {u_f}"
                );
            } else {
                assert!(u_f < 0.5 * v_t && u_s > 0.7 * u_f, "{u_f} {u_s}");
            }
        }
    }

    /// Two plates side by side, one pulled by a trench, the other free: with boundary resistance
    /// the free one is dragged along; the solve is finite and balanced from the weakest settings
    /// to the stiffest.
    #[test]
    fn the_solve_is_stable_from_the_weakest_to_the_stiffest_settings() {
        for (sr, br, keel, flow) in [
            (0.0, 0.0, 1.0, 0.0),
            (0.0, 0.0, 1.0, 20.0),
            (3.0, 3.0, 3.0, 5.0),
            (100.0, 1000.0, 20.0, 5.0),
        ] {
            let k = TectonicParams {
                slab_resistance: sr,
                boundary_resistance: br,
                keel_drag: keel,
                mantle_flow_mm_yr: flow,
                forsyth_uyeda_balance: false,
                ..TectonicParams::default()
            };
            let c = coeffs(&k);
            let (pts, rim) = cap(&c, 25.0);
            // Plate 1 is plate 0 turned to sit beside it, sharing the rim point nearest to it.
            let turn = glam::DQuat::from_rotation_y(50f64.to_radians());
            let mut bal = Balance::new(c, &[DVec3::Z, turn * DVec3::Z]);
            for &x in &pts {
                bal.add_area(0, x, false);
                bal.add_area(1, turn * x, true);
            }
            let mut pulled = 0.0;
            for &(x, out) in &rim {
                if x.x < -0.2 {
                    bal.add_force(0, x, out * c.pull_n_m * c.spacing_m, Drive::SlabPull);
                    bal.add_link(Link {
                        a: 0,
                        b: MANTLE,
                        x,
                        n: Some(out),
                        k: c.slab_res * c.spacing_m,
                        term: Resist::Slab,
                    });
                    pulled += 1.0;
                }
                if x.x > 0.3 {
                    bal.add_link(Link {
                        a: 0,
                        b: 1,
                        x,
                        n: None,
                        k: c.boundary_res * c.spacing_m,
                        term: Resist::Boundary,
                    });
                }
            }
            assert!(pulled > 0.0);
            let w = bal.solve().expect("positive definite");
            let ans = bal.answers(&w);
            let (a0, a1) = (ans[0].unwrap(), ans[1].unwrap());
            for a in [a0, a1] {
                assert!(
                    a.speed_mm_yr.is_finite() && a.stress_mpa.is_finite(),
                    "{a:?}"
                );
                assert!(a.residual < 1e-6, "residual {} at {sr} {br}", a.residual);
            }
            assert!(a0.speed_mm_yr > 1.0, "{a0:?}");
            if br > 100.0 {
                // Stiff coupling: the two move almost as one (a short shared edge leaves the
                // turn about an axis through it least constrained).
                let dw = (w[0] - w[1]).length() / w[0].length();
                assert!(dw < 0.15, "{dw}");
            }
            if br == 0.0 && flow == 0.0 {
                // Nothing drives the free plate.
                assert!(a1.speed_mm_yr < 1e-6, "{a1:?}");
            }
        }
    }

    #[test]
    fn a_slab_grows_with_convergence_and_breaks_off_when_a_continent_blocks_it() {
        let mut s = SlabTrack::default();
        let ocean = |n: u32, conv: f64| FrontSum {
            oceanic: n,
            continental: 0,
            conv_sum: conv * f64::from(n),
        };
        // 50 mm/yr for 2 Myr: 100 km a step, up to the depth of 700 km.
        assert!(!grow_slab(&mut s, &ocean(10, 50.0), 2.0, 700.0, 10.0));
        assert!((s.km - 100.0).abs() < 1e-9);
        for _ in 0..10 {
            grow_slab(&mut s, &ocean(10, 50.0), 2.0, 700.0, 10.0);
        }
        assert_eq!(s.km, 700.0);
        // A continent in a minority of the front: still growing, no block time.
        let some = FrontSum {
            oceanic: 8,
            continental: 2,
            conv_sum: 8.0 * 40.0,
        };
        s.km = 300.0;
        assert!(!grow_slab(&mut s, &some, 2.0, 700.0, 10.0));
        assert!((s.km - 380.0).abs() < 1e-9 && s.blocked_myr == 0.0);
        // Blocked for 10 Myr (5 steps of 2): it breaks off on the fifth and starts again.
        let blocked = FrontSum {
            oceanic: 2,
            continental: 8,
            conv_sum: 2.0 * 30.0,
        };
        for i in 0..4 {
            assert!(!grow_slab(&mut s, &blocked, 2.0, 700.0, 10.0), "step {i}");
        }
        assert!(s.km > 380.0 && (s.blocked_myr - 8.0).abs() < 1e-9);
        assert!(grow_slab(&mut s, &blocked, 2.0, 700.0, 10.0));
        assert_eq!((s.km, s.blocked_myr), (0.0, 0.0));
        // An unblocked step in between resets the clock.
        grow_slab(&mut s, &blocked, 2.0, 700.0, 10.0);
        grow_slab(&mut s, &ocean(5, 20.0), 2.0, 700.0, 10.0);
        assert_eq!(s.blocked_myr, 0.0);
        assert!((s.km - 100.0).abs() < 1e-9, "{}", s.km);
    }

    #[test]
    fn cholesky_solves_a_small_system_and_refuses_an_indefinite_one() {
        let mut m = vec![4.0, 2.0, 0.0, 2.0, 5.0, 1.0, 0.0, 1.0, 3.0];
        let x = cholesky_solve(&mut m, &[2.0, 3.0, 1.0], 3).unwrap();
        let a = [[4.0, 2.0, 0.0], [2.0, 5.0, 1.0], [0.0, 1.0, 3.0]];
        for r in 0..3 {
            let s: f64 = (0..3).map(|c| a[r][c] * x[c]).sum();
            assert!((s - [2.0, 3.0, 1.0][r]).abs() < 1e-12);
        }
        let mut bad = vec![1.0, 2.0, 2.0, 1.0];
        assert!(cholesky_solve(&mut bad, &[1.0, 1.0], 2).is_none());
    }

    /// The switch's relaxation (design §5.1 a): after one relaxation time a plate has made
    /// 1 − 1/e of the way to the balance's rotation, and it never overshoots.
    #[test]
    fn the_motion_relaxes_towards_the_balance_without_overshooting() {
        let (w0, target) = (DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, 2.0, 0.0));
        let mut w = w0;
        for _ in 0..3 {
            w = relax(w, target, 2.0, 6.0);
        }
        let gone = 1.0 - (w - target).length() / (w0 - target).length();
        assert!((gone - (1.0 - (-1.0f64).exp())).abs() < 1e-12, "{gone}");
        for _ in 0..200 {
            let next = relax(w, target, 2.0, 6.0);
            assert!((next - target).length() <= (w - target).length() + 1e-15);
            w = next;
        }
        assert!((w - target).length() < 1e-9);
        // At rest, a plate stays at rest until a force acts.
        assert_eq!(relax(DVec3::ZERO, DVec3::ZERO, 2.0, 6.0), DVec3::ZERO);
    }

    /// Two hemispheres: plate 0 pulled by a trench, plate 1 free, coupled by boundary
    /// resistance along their whole shared great circle. The rotations about the pull's axis
    /// are the known answer of the 2 × 2 system [D + K, −K; −K, D + K] w = [T, 0]: uncoupled the
    /// free plate stands still and the pulled one turns at T / D; stiffly coupled both turn at
    /// T / 2D.
    #[test]
    fn two_coupled_plates_move_as_the_algebra_says() {
        for br in [0.0, 3.0, 1.0e4] {
            let k = TectonicParams {
                boundary_resistance: br,
                slab_resistance: 0.0,
                ..old_law()
            };
            let c = coeffs(&k);
            let (pts, _) = cap(&c, 90.0);
            // Plate 0 the northern hemisphere, plate 1 the southern.
            let mut bal = Balance::new(c, &[DVec3::Z, -DVec3::Z]);
            for &x in &pts {
                bal.add_area(0, x, false);
                bal.add_area(1, DVec3::new(x.x, x.y, -x.z), false);
            }
            // A trench on plate 0 near its pole: pulls towards +X, a torque about +Y.
            let x0 = on_sphere(0.0, 60.0);
            let f = tangent(x0, DVec3::X).normalize() * (c.pull_n_m * 1.0e6);
            bal.add_force(0, x0, f, Drive::SlabPull);
            // The boundary: the equator, one link per spacing.
            let ring = (2.0 * std::f64::consts::PI * c.radius_m / c.spacing_m).round() as usize;
            for j in 0..ring {
                let x = on_sphere(360.0 * j as f64 / ring as f64, 0.0);
                bal.add_link(Link {
                    a: 0,
                    b: 1,
                    x,
                    n: None,
                    k: c.boundary_res * c.spacing_m,
                    term: Resist::Boundary,
                });
            }
            let w = bal.solve().unwrap();
            // The known answer, from the same sums about the y axis.
            let d = bal.acc[0].drag.col(1).y;
            let kk: f64 = bal.links.iter().map(|l| bal.link_matrix(l).col(1).y).sum();
            let t = bal.acc[0].t_pull.y;
            let det = d * (d + 2.0 * kk);
            let (w0, w1) = (t * (d + kk) / det, t * kk / det);
            // (The 2 × 2 answer leaves out the lattice's small cross terms about the other axes;
            // under stiff coupling they shift the answer by a few parts in a million.)
            assert!(
                (w[0].y - w0).abs() < 1e-4 * w0.abs(),
                "{br}: {} {w0}",
                w[0].y
            );
            assert!(
                (w[1].y - w1).abs() < 1e-4 * w0.abs(),
                "{br}: {} {w1}",
                w[1].y
            );
            if br == 0.0 {
                assert!(w[1].length() < 1e-12 * w[0].length());
            }
            if br > 1000.0 {
                assert!((w[0].y / (t / (2.0 * d)) - 1.0).abs() < 0.01, "{}", w[0].y);
            }
        }
    }

    /// The collision term is plastic: a collision front whose belt force exceeds the pull holds
    /// the convergence under the floor speed (`COLLISION_FLOOR_MM_YR`), and a weaker one only
    /// slows the plate.
    #[test]
    fn a_collision_front_stalls_a_plate_only_when_its_belt_outweighs_the_pull() {
        let k = quiet();
        let c = coeffs(&k);
        let (pts, _) = cap(&c, 25.0);
        let x0 = on_sphere(0.0, 65.0);
        let dir = tangent(x0, DVec3::X).normalize();
        let pull = c.pull_n_m * 1.0e6;
        let solve = |belt: Option<f64>| {
            let mut bal = Balance::new(c, &[DVec3::Z]);
            for &x in &pts {
                bal.add_area(0, x, false);
            }
            bal.add_force(0, x0, dir * pull, Drive::SlabPull);
            if let Some(force) = belt {
                let v = COLLISION_FLOOR_MM_YR * 1.0e-3 / YEAR_S;
                bal.add_link(Link {
                    a: 0,
                    b: MANTLE,
                    x: x0,
                    n: Some(dir),
                    k: force / v,
                    term: Resist::Collision,
                });
            }
            let w = bal.solve().unwrap();
            (w[0].cross(x0) * YEAR_S * c.radius_m * 1.0e3).dot(dir)
        };
        let free = solve(None);
        assert!(free > 20.0, "{free}");
        // A belt twice the pull: under the floor.
        let held = solve(Some(2.0 * pull));
        assert!(held > 0.0 && held < COLLISION_FLOOR_MM_YR, "{held}");
        // A belt of a small share of the pull: the plate still moves, slower.
        let slowed = solve(Some(0.001 * pull));
        assert!(slowed > held && slowed < free, "{slowed} {free}");
    }

    /// The collision's buoyancy (FRC-030): nothing before any crust has gone down; growing with
    /// the shortening as Cloos 1993's 0.09 / 0.04 times the pull of the same length of 80-Myr
    /// slab, in proportion to the crust's thickness; no more past the slab length that pulls in
    /// full. A 30 km crust 100 km into a collision pushes back with 0.32 of a full slab's pull.
    #[test]
    fn continental_lithosphere_dragged_down_pushes_back() {
        let (f, d) = (SLAB_PULL_N_M, 700.0);
        assert_eq!(collision_buoyancy_n_m(f, d, 30_000.0, 0.0), 0.0);
        let f100 = collision_buoyancy_n_m(f, d, 30_000.0, 100.0);
        assert!((f100 / collision_buoyancy_n_m(f, d, 30_000.0, 50.0) - 2.0).abs() < 1e-12);
        assert!(
            (f100 / (2.25 * f * 100.0 / 700.0) - 1.0).abs() < 1e-12,
            "{f100}"
        );
        assert!((f100 / f - 0.3214).abs() < 1e-3);
        // Thicker crust, more buoyancy.
        let thick = collision_buoyancy_n_m(f, d, 45_000.0, 100.0);
        assert!((thick / f100 - 1.5).abs() < 1e-12);
        // No more past the full slab length.
        assert_eq!(
            collision_buoyancy_n_m(f, d, 30_000.0, 2000.0),
            collision_buoyancy_n_m(f, d, 30_000.0, 700.0)
        );
        assert!((collision_buoyancy_n_m(f, d, 30_000.0, 700.0) / f - 2.25).abs() < 1e-12);
    }

    #[test]
    fn four_significant_digits() {
        assert_eq!(sig4(123456.0), 123500.0);
        assert!((sig4(0.000123456) - 0.0001235).abs() < 1e-15);
        assert_eq!(sig4(0.0), 0.0);
        assert_eq!(sig4(-98765.0), -98770.0);
    }
}

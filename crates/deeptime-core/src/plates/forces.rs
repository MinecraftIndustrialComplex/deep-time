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
//! the velocities, so the balance is one linear system over all plates,
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
//! spacing per boundary sample (as the record's `boundary_km`), areas one sample's area.
//! - **Slab pull** (drives the subducting plate, towards the trench): per metre of trench where
//!   the subducting crust is oceanic, `F_sp · slab_pull · s(age) · min(L, D)/D`, with `F_sp` the
//!   reference pull ([`SLAB_PULL_N_M`] at Earth's gravity, scaled with gravity), `s` the slab's
//!   thickness factor ([`slab_weight`]), `L` the contact's tracked slab length and `D`
//!   `slab_depth_km`. Continental crust in the trench pulls nothing.
//! - **Slab suction** (drives the overriding plate, towards the trench): `slab_suction_share` of
//!   that pull.
//! - **Edge energy** (ridge push and the weight of high ground): at every boundary sample of a
//!   plate, `(U(edge) − Ū) · edge_energy / EDGE_ENERGY_PHYSICAL` per metre, pushing the plate
//!   away from the edge when the edge stands high ([`column_energy`]: the gravitational
//!   potential energy of the column; `Ū` the plate's mean). On a rigid plate a constant `U`
//!   gives no torque, so only the differences matter; `Ū` keeps the sum's discretisation small.
//!   At a collision front the belt's weight acts as the collision resistance instead (below).
//! - **Basal drag** (resists): per m², `c_b (v − v_flow)`, `keel_drag` times more under
//!   continental crust. `c_b` is set by `plate_speed_scale_mm_yr` (see [`Coeffs::new`]).
//! - **Mantle flow** (drives, weakly): the drag of a slow degree-2 flow ([`flow_velocity`]) of
//!   peak `mantle_flow_mm_yr`, away from two antipodal upwellings: a stand-in for the
//!   convection the model does not have (Ben, 2026-10-02: weak mantle flow as a setting).
//! - **Slab resistance** (resists): per metre of trench, against the subducting plate's speed
//!   towards the trench (Forsyth & Uyeda's resistance to a slab's sinking).
//! - **Boundary resistance** (resists): per metre of any plate boundary, against the relative
//!   velocity of the two plates (across and along), half from each side.
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
//! two plates: it grows by the convergence at the contact's oceanic front each step, up to
//! `slab_depth_km`, and breaks off (length 0) when continental crust has held at least half of
//! the contact's front for `slab_breakoff_myr` (design §3.3, option 5.3a). A contact that flickers
//! off and on keeps its slab (task 5 found the restarts to be the main source of noise); a
//! reversed contact starts a new one. A slab no oceanic front feeds this step hangs: it pulls at
//! the subducting plate's samples beside the overriding plate (with suction and slab resistance
//! as at a front) and is gone after `slab_breakoff_myr` without a front. A subduction initiation
//! plants a slab of `initiation_slab_km` (`rifting.rs`), which pulls the new plate towards its
//! parent until the trench forms.

use std::collections::{BTreeMap, BTreeSet};

use glam::{DMat3, DVec3};
use rayon::prelude::*;

use super::Env;
use super::dynamics::PlateForces;
use super::geometry::Geo;
use super::kinematics::{quat, tangent};
use super::normalise::Norm;
use super::state::{CONTINENTAL, PlateState, SlabTrack};
use crate::params::TectonicParams;
use crate::sphere::SplitMix64;

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
        let v = k.plate_speed_scale_mm_yr * 1.0e-3 / YEAR_S;
        let drag = f_ref * trench / ((1.0 + k.slab_resistance + k.boundary_resistance) * area * v); // ledger: FRC-016
        Coeffs {
            radius_m,
            sample_m2,
            spacing_m,
            pull_n_m: f_ref * k.slab_pull,
            suction_share: k.slab_suction_share,
            edge_scale: k.edge_energy / EDGE_ENERGY_PHYSICAL,
            buoyancy: k.collision_buoyancy,
            drag,
            keel: k.keel_drag,
            slab_res: k.slab_resistance * drag * area / trench, // ledger: FRC-017
            boundary_res: k.boundary_resistance * drag * area / perimeter, // ledger: FRC-017
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
        let (mut t_slab, mut t_bnd, mut t_col) = (
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
                    Resist::Boundary => t_bnd[p] += t,
                    Resist::Collision => t_col[p] += t,
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
                let terms = [
                    a.t_pull, a.t_suck, a.t_edge, a.t_flow, t_drag, t_slab[p], t_bnd[p], t_col[p],
                ];
                let sum: DVec3 = terms.iter().copied().sum();
                let largest = terms.iter().map(|t| t.length()).fold(0.0, f64::max);
                let w_yr = wp * YEAR_S;
                let rms = (wp.dot(a.geom * wp) / f64::from(a.n)).max(0.0).sqrt() * r;
                Some(PlateAnswer {
                    omega: w_yr,
                    slab_pull: a.t_pull.length(),
                    slab_suction: a.t_suck.length(),
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
    /// The front this sample overrides: the subducting neighbour, the contact, the outward
    /// direction towards it and the convergence, mm/yr.
    front: Option<(u32, usize, DVec3, f64)>,
    /// A neighbour of a plate whose slab hangs under this sample's plate (a slab with no front
    /// this step, or one a subduction initiation planted): the neighbour and the direction
    /// towards it.
    hang: Option<(u32, DVec3)>,
    /// The sample is continental on the subducting side of an active contact, converging: the
    /// incoming side of a collision.
    incoming: bool,
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
}

/// The step's balance: each plate's balance rotation, rad/yr (indexed by plate id; `None` for a
/// plate not in the balance, or for every plate when the solve failed), and what the dynamics
/// record keeps (`None` when the record is off).
#[derive(Clone, Debug, Default)]
pub struct Answer {
    pub omega: Vec<Option<DVec3>>,
    pub solved: bool,
    pub recorded: Option<Recorded>,
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
    let (omega, solved, out) = balance_step(st, geo, pos, env, flow, sea_m, &mut slabs);
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
        solved,
        recorded,
    }
}

fn balance_step(
    st: &PlateState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env<'_>,
    flow: DVec3,
    sea_m: f64,
    slabs: &mut Vec<SlabTrack>,
) -> (Vec<Option<DVec3>>, bool, Recorded) {
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
        if c.last_step == env.step {
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
        .filter(|s| s.km > 0.0)
        .map(|s| (s.under, s.over))
        .collect();

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
                front: None,
                hang: None,
                incoming: false,
            };
            let mut any = false;
            let mut best = 0.0;
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
                if !e.others[..usize::from(e.n_others)].contains(&pu)
                    && usize::from(e.n_others) < e.others.len()
                {
                    e.others[usize::from(e.n_others)] = pu;
                    e.n_others += 1;
                }
                if e.hang.is_none() && !su.is_hidden() && hanging.contains(&(pu, pv)) {
                    e.hang = Some((u, dir));
                }
                let rel = norm.velocity_mm_yr(omega[pu as usize] - omega[pv as usize], x);
                let along = rel.dot(dir);
                if along < 0.0 {
                    if let Some(&c) = active.get(&(pu, pv))
                        && -along > best
                    {
                        best = -along;
                        e.front = Some((u, c, dir, -along));
                    } else if active.contains_key(&(pv, pu)) && sv.kind == CONTINENTAL {
                        e.incoming = true;
                    }
                }
            }
            any.then_some(e)
        })
        .collect();

    // Slabs: grow by the oceanic front's convergence, break off when continental crust has held
    // the trench for `slab_breakoff_myr`; a slab no front feeds hangs and pulls along the shared
    // boundary for `slab_breakoff_myr` and is then gone.
    let mut fronts: BTreeMap<usize, FrontSum> = BTreeMap::new();
    for e in &edges {
        if let Some((u, c, _, conv)) = e.front {
            let f = fronts.entry(c).or_default();
            if samples[u as usize].kind == CONTINENTAL {
                f.continental += 1;
            } else {
                f.oceanic += 1;
                f.conv_sum += conv;
            }
        }
    }
    let mut by_key: BTreeMap<(u32, u32), SlabTrack> = std::mem::take(slabs)
        .into_iter()
        .map(|s| ((s.under, s.over), s))
        .collect();
    let mut breakoffs = 0u16;
    let mut slab_km: BTreeMap<usize, f64> = BTreeMap::new();
    let mut slabs_km = Vec::new();
    // Slabs with a front this step (fed when it is partly oceanic).
    let mut fronted: BTreeSet<(u32, u32)> = BTreeSet::new();
    let mut fed: BTreeSet<(u32, u32)> = BTreeSet::new();
    for (&ci, f) in &fronts {
        let c = &st.contacts[ci];
        let key = (c.under, c.over);
        fronted.insert(key);
        let s = by_key.entry(key).or_insert(SlabTrack {
            under: c.under,
            over: c.over,
            ..SlabTrack::default()
        });
        if grow_slab(s, f, env.dt_myr, k.slab_depth_km, k.slab_breakoff_myr) {
            breakoffs += 1;
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
    // Slabs that pull along the boundary this step without a front.
    let hang_km: BTreeMap<(u32, u32), f64> = by_key
        .iter()
        .filter(|(key, _)| !fronted.contains(key))
        .map(|(&key, s)| (key, s.km))
        .collect();
    *slabs = by_key.into_values().collect();

    // The system.
    let a0 = st.a0_km2;
    let c = Coeffs::new(k, &norm, env.gravity, a0, flow, sea_m);
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

    let ell = c.spacing_m;
    // The collision drag's convergence floor (design §3.4 linearises with the weld speed; it sits
    // under it, so a held collision stalls).
    let floor = COLLISION_FLOOR_MM_YR.min(k.weld_speed_mm_yr); // ledger: FRC-011
    for e in &edges {
        let v = e.v as usize;
        let sv = &samples[v];
        let x = pos[v];
        let p = e.plate;
        let mut collision = e.incoming;
        // The slab under this sample, at its front or hanging with no front: the subducting
        // plate, the neighbour, the direction towards it, the slab's length.
        let slab = match e.front {
            Some((u, ci, dir, conv)) => {
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
                        * ell;
                    let speed = conv.max(floor) * 1.0e-3 / YEAR_S;
                    if force > 0.0 {
                        bal.add_link(Link {
                            a: under,
                            b: p,
                            x,
                            n: Some(-dir),
                            k: force / speed,
                            term: Resist::Collision,
                        });
                    }
                    None
                } else {
                    Some((under, u, dir, slab_km.get(&ci).copied().unwrap_or(0.0)))
                }
            }
            None => e.hang.and_then(|(u, dir)| {
                let under = plate_of[u as usize];
                (samples[u as usize].kind != CONTINENTAL)
                    .then(|| hang_km.get(&(under, p)).map(|&km| (under, u, dir, km)))
                    .flatten()
            }),
        };
        if let Some((under, u, dir, len)) = slab {
            let su = &samples[u as usize];
            let towards = -dir;
            let pull = c.pull_n_m
                * slab_weight(su.age_myr(env.t_myr))
                * (len / c.slab_depth_km).clamp(0.0, 1.0)
                * ell;
            if pull > 0.0 {
                bal.add_force(under, x, towards * pull, Drive::SlabPull);
                bal.add_force(p, x, dir * (pull * c.suction_share), Drive::Suction);
            }
            bal.add_link(Link {
                a: under,
                b: MANTLE,
                x,
                n: Some(towards),
                k: c.slab_res * ell,
                term: Resist::Slab,
            });
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

    let (answers, solved) = match bal.solve() {
        Some(w) => (bal.answers(&w), true),
        None => (vec![None; np], false),
    };
    let unit = TORQUE_UNIT_N_M;
    let mut forces = Vec::new();
    let mut out = vec![None; np];
    for p in st.alive() {
        let Some(a) = answers.get(p.id as usize).copied().flatten() else {
            continue;
        };
        out[p.id as usize] = Some(a.omega);
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
    (
        out,
        solved,
        Recorded {
            forces,
            slabs_km,
            breakoffs,
        },
    )
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

    #[test]
    fn slab_weight_grows_with_the_root_of_age_and_young_crust_floats() {
        assert_eq!(slab_weight(5.0), 0.0);
        assert!((slab_weight(80.0) - 1.0).abs() < 1e-12);
        assert!((slab_weight(20.0) - 0.5).abs() < 1e-12);
        assert!((slab_weight(400.0) - 1.5).abs() < 1e-12);
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
        let k = quiet();
        let c = coeffs(&k);
        // A flat-ish cap of the reference area pulled by a straight trench across its rim: drag
        // (1 + slab resistance + boundary resistance) c_b A v = F L, so v is about 45 mm/yr when
        // the trench is one slab of 80 Myr at full length.
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
        // The sphere's cap is not flat, so the factor of the reference only holds roughly.
        assert!(
            (25.0..70.0).contains(&ans.speed_mm_yr),
            "speed {}",
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
        let k = quiet();
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
                ..quiet()
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

// SPDX-License-Identifier: LGPL-3.0-or-later
//! Lithospheric flexure (Ben, decisions log 2026-10-02: "General flexure everywhere"; the design
//! is `docs/design/flexure.md`).
//!
//! # The model
//! The lithosphere is a thin elastic plate on a fluid mantle. Its deflection w (m, down positive)
//! under a load q (kg/m²) obeys
//!
//!   ∇²(D ∇²w) + ρ_m g w = g q,   D = E·Te³ / (12 (1 − ν²)),
//!
//! with Te the effective elastic thickness. With D = 0 it is Airy isostasy, w = q / ρ_m, which is
//! what the model did before; with D > 0 a load narrower than the flexural parameter
//! α = (4D / ρ_m g)^¼ is held up by the plate and pulls its surroundings down: a foreland basin in
//! front of a belt, a moat round a volcanic island, a low bulge beyond. Gravity enters through α
//! only (load and buoyancy both carry g).
//!
//! # Loads, and what is already compensated
//! The surface stage keeps, with the crust, the load `Q` since the start of the run (m of mantle
//! displaced under Airy: kg/m² ÷ ρ_m) and the deflection `w` that goes with it. Each step it adds
//! - the crust the plate model thickened or thinned (the Airy change u_c of the column's own crust,
//!   `plates/crust_uplift_mm_yr`: the thickening is u_c / (1 − k_a), k_a = ρ_c/ρ_m, and it raises
//!   the ground 1 − k_a of it in the air and 1 − k_w, k_w = (ρ_c − ρ_w)/(ρ_m − ρ_w), under water),
//!   and the crust the plateau spreading moved (`spread.rs`): for these the Airy root `a` is
//!   already in the elevation, and the spreading's stays there (it bends nothing, ISO-070). The
//!   rest of the plate model's uplift moved a height alone (its settling to the Airy height plus
//!   this flexure, its height rules, its rate limit's hold) and is no load (ISO-091). On ocean
//!   floor that builds nothing all of it is (its cooling, the pull to a trench and the release
//!   from it), and so is what the carry made of a gap's floor; the sea a height moved alone lets
//!   in or pushes out is no load either: the plate model's heights are heights under the sea, with
//!   the water's weight in them (ISO-096);
//! - the step's erosion and deposition, the rock other stages laid on the surface since the last
//!   step (the volcano stage's cones), the sea over the cell against the sea at the reference,
//!   and land ice: for these nothing has been applied;
//!
//! solves for the deflection δw of the step's loads, and changes the elevation by −(δw − a).
//! Every kilogram is in `Q` once and every metre of sinking is applied once, by the plate model
//! (`a`) or by the solve. With D = 0 the solve returns the Airy sinking of every load: erosion
//! rebounds as before (without the old blur) and plate uplift is unchanged.
//!
//! # Increments, not the whole load
//! The carried plate is taken to be in equilibrium under the carried load, as the real one is:
//! each step solves only for the loads of the step, what the last solve left unfinished (carried
//! with the crust), and the load a weakened plate stops holding up. Moving the plate, interpolating its
//! state on the raster and restarting new crust therefore move nothing. (The first build re-solved
//! the whole load every step; the interpolation error of the carried state then acted as a force
//! and moved the ground by up to several kilometres a step along margins and near the poles: the
//! cell-scale zigzag and the polar stripes of the build log, 2026-10-02.)
//!
//! # A plate that stiffens keeps its shape; one that weakens settles
//! Rigidity gained by cooling joins stress-free: nothing moves, and later loads meet the stiffer
//! plate. So an old belt's local root does not turn into relief as its crust cools, and a load
//! records the strength of the plate at the time of loading (Watts 1978). Rigidity lost
//! (reheating, a cell becoming a plate's broken edge) gives up the same share of what the plate
//! held: where D falls to r·D, the load the plate held away from local isostasy,
//! ρ_m (Q − w), less r of it, is a force. The ground settles towards local isostasy, which is
//! where a plate with no strength puts it, and no further. (Until 2026-10-18 the plate carried
//! its bending moment and released it as a moment: the curvature of a moment released along a
//! boundary one cell wide, the polar plate fields' edges stepping from row to row, made forces
//! of several times the load held, and holes and spikes of tens of kilometres near the poles,
//! build log 2026-10-18.)
//!
//! # Lengths
//! Earth-normalized, as the plate model's (decisions file, "Tectonics scaling"): the solve's cell
//! size is the L1 cell's share of the model sphere's circumference, so a basin is as wide against
//! its belt as on Earth. Heights are not scaled.
//!
//! # The solve
//! (L D L + C) δw = r on the raster in block metric (x wraps, rows closed), L the 5-point
//! Laplacian, C the restoring density (ρ_m, less ρ_w under the sea: a basin that sinks takes
//! water). Conjugate gradients, preconditioned by exact spectral solves at three uniform
//! rigidities (a Fourier transform in x, a cosine transform in y) blended by smooth weights in
//! ln α: symmetric, so it converges for any Te field, and the exact inverse where Te is uniform.
//! Presets with L1 cells finer than about 40 Earth-normalized km solve on L1 coarsened by a power
//! of two and interpolate the deflection.
//!
//! The solver is written from the equation; no flexure code was read or ported.

use std::sync::Arc;

use rayon::prelude::*;
use rustfft::num_complex::Complex64;
use rustfft::{Fft, FftPlanner};
use serde::{Deserialize, Serialize};

use super::advect::{self, Carry};
use super::grid::{Dims, box_blur};
use crate::error::CoreError;
use crate::plates::pools::RHO_CRUST_MEAN;
use crate::plates::state::{OCEANIC, regime};

/// Mantle, sea water and ice, kg/m³.
pub const RHO_MANTLE: f64 = 3300.0; // ledger: ISO-043
pub const RHO_WATER: f64 = 1030.0; // ledger: ISO-044
pub const RHO_ICE: f64 = 917.0; // ledger: ISO-045
/// Young's modulus, Pa, and Poisson's ratio (Turcotte & Schubert, *Geodynamics*).
pub const YOUNG_PA: f64 = 70.0e9; // ledger: ISO-041
pub const POISSON: f64 = 0.25; // ledger: ISO-042
/// The coarsest solve cell, Earth-normalized metres: finer L1 grids are coarsened towards it.
pub const SOLVE_CELL_M: f64 = 45_000.0; // ledger: ISO-058 (game-choice: numerical)

/// Fields the surface stage writes each step when flexure is on.
pub mod field {
    /// How far the ground stands above (+) or below (−) where local isostasy would put it, m:
    /// negative in foreland basins and moats, positive on bulges and on loads the plate holds up.
    pub const FLEXURE: &str = "surface/flexure_m";
    /// The whole sinking under the load since the start of the run, m (down positive).
    pub const DEFLECTION: &str = "surface/deflection_m";
    /// The effective elastic thickness the step was solved with, km.
    pub const ELASTIC_THICKNESS: &str = "surface/elastic_thickness_km";
}

/// `WorldState::globals`: solves, their conjugate-gradient iterations, and the solves that
/// stopped at `max_iterations`.
pub const SOLVES_GLOBAL: &str = "surface/flexure_solves";
pub const ITERATIONS_GLOBAL: &str = "surface/flexure_iterations";
pub const UNCONVERGED_GLOBAL: &str = "surface/flexure_unconverged";
/// `WorldState::globals`: the solve's cells whose plate yielded, summed over the steps (only
/// with `yield_anomaly_mgal`, ISO-120).
pub const YIELDED_GLOBAL: &str = "surface/flexure_yielded_cells";

/// The `surface.flexure` section: every key optional.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct FlexureParams {
    /// 0: local isostasy as before (`isostasy`, `isostasy_radius_km`); 1: the flexure solve;
    /// between, a blend of the two corrections. The struct's default is 0; the pipeline presets
    /// set 1.
    pub strength: f64,
    /// Multiplier on the elastic thickness everywhere (the editor's "Plate stiffness").
    pub elastic_thickness_scale: f64,
    /// Oceanic Te = this × √(age in Myr), km: 2.7 ± 0.15 measured under 38 seamounts (Calmant,
    /// Francheteau & Cazenave 1990, full text), near the 450 ± 150 °C isotherm of the cooling plate
    /// (Watts 1978, abstract).
    pub te_ocean_km_sqrt_myr: f64,
    /// … and at most this, km (Hawaii, on 80 Myr floor: 40 km, Watts & ten Brink 1989, abstract).
    pub te_ocean_max_km: f64,
    /// Continental Te long after the last orogeny, km (cratons: 70–90 km, Burov & Diament 1995).
    pub te_craton_km: f64,
    /// Continental Te recovers as 1 − e^(−t / this), t the time since the last orogeny, Myr.
    pub te_recovery_myr: f64,
    /// Te of arcs, rifts and everything else at least, km (the low end of continental Te: 5 km,
    /// Lu et al. 2024, abstract).
    pub te_min_km: f64,
    /// Where the law above lowers a plate's Te (an orogeny, an arc, a rift: heating), the plate
    /// weakens towards it as e^(−t / this), Myr: the time its geotherm takes to warm (the
    /// lithosphere's thermal time constant: 62.8 Myr, ISO-099). It stiffens at once. 0, the
    /// default: weakens at once too, the rule before 2026-10-23 (off until the pool says
    /// otherwise: on the pool's runs 62.8 left the criteria where they were and gave more closed
    /// basins and larger one-step falls, docs/science/flexure.md "The release's pace"). The break
    /// at a plate boundary is not the law's and stays at once.
    pub te_weakening_myr: f64,
    /// The plate's strength as the largest free-air gravity anomaly a plate holds, mGal: where
    /// the load a cell's plate holds away from local isostasy after the solve, ρ_m (Q + ice − w),
    /// passes Δg / 2πG, the plate there yields and the rest of the load sinks (or rises) under
    /// the cell alone, as on a plate with no strength. 380 is Earth's most negative free-air
    /// anomaly (the Puerto Rico Trench, ten Brink et al. 2004): 9.1 × 10⁶ kg/m², 2.75 km of
    /// mantle. 0, the default: off, the purely elastic plate (ISO-120).
    pub yield_anomaly_mgal: f64,
    /// The solve on the sphere's metric (ISO-127): on the equal-area raster a cell at latitude φ
    /// is cos φ as wide and 1 / cos φ as tall as at the equator, and the Laplacian in the raster's
    /// coordinates is ∂x(cos⁻²φ ∂x) + ∂y(cos²φ ∂y). False: every cell a square of the equator's
    /// width (the block metric, the rule before 2026-10-23), about 1 / cos⁴ φ too stiff north to
    /// south and cos⁴ φ too soft east to west.
    pub row_metric: bool,
    /// The water over a cell is the open sea's or its lake's, not every cell under sea level's
    /// (ISO-126): a closed basin carries its lake at the level its water balance holds, or no
    /// water when it is dry. False: every cell at or under sea level is under the sea (the rule
    /// before 2026-10-23).
    pub water_by_basin: bool,
    /// Solve every this many plate steps (loads in between wait for the next solve).
    pub every_steps: u32,
    /// The solve stops when its error estimate is under this, m.
    pub tolerance_m: f64,
    /// … or after this many iterations.
    pub max_iterations: u32,
}

impl Default for FlexureParams {
    fn default() -> Self {
        FlexureParams {
            strength: 0.0, // ledger: ISO-065 (game-choice: off unless a preset turns it on)
            elastic_thickness_scale: 1.0, // ledger: ISO-066
            te_ocean_km_sqrt_myr: 2.7, // ledger: ISO-048
            te_ocean_max_km: 40.0, // ledger: ISO-049
            te_craton_km: 80.0, // ledger: ISO-050
            te_recovery_myr: 300.0, // ledger: ISO-051 (uncited: this agent's choice)
            te_min_km: 5.0, // ledger: ISO-052
            te_weakening_myr: 0.0, // ledger: ISO-099 (game-choice: off by default)
            yield_anomaly_mgal: 0.0, // ledger: ISO-120 (off by default)
            row_metric: false, // ledger: ISO-127
            water_by_basin: false, // ledger: ISO-126
            every_steps: 1, // ledger: ISO-059 (game-choice: numerical)
            tolerance_m: 2.0, // ledger: ISO-059
            max_iterations: 60, // ledger: ISO-059
        }
    }
}

impl FlexureParams {
    /// Whether the solve runs at all.
    pub fn active(&self) -> bool {
        self.strength > 0.0
    }

    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let check = |ok: bool, key: &str, reason: &str| -> Result<(), CoreError> {
            if ok {
                Ok(())
            } else {
                Err(CoreError::preset(
                    format!("$.surface.flexure.{key}"),
                    reason,
                ))
            }
        };
        check(
            (0.0..=1.0).contains(&self.strength),
            "strength",
            "must be in [0, 1]",
        )?;
        check(
            self.elastic_thickness_scale.is_finite()
                && (0.01..=10.0).contains(&self.elastic_thickness_scale),
            "elastic_thickness_scale",
            "must be in [0.01, 10]",
        )?;
        for (v, key) in [
            (self.te_ocean_km_sqrt_myr, "te_ocean_km_sqrt_myr"),
            (self.te_ocean_max_km, "te_ocean_max_km"),
            (self.te_craton_km, "te_craton_km"),
            (self.te_recovery_myr, "te_recovery_myr"),
            (self.te_min_km, "te_min_km"),
            (self.tolerance_m, "tolerance_m"),
        ] {
            check(v.is_finite() && v > 0.0, key, "must be > 0")?;
        }
        check(
            self.te_weakening_myr.is_finite() && (0.0..=10_000.0).contains(&self.te_weakening_myr),
            "te_weakening_myr",
            "must be in [0, 10000]",
        )?;
        check(
            self.yield_anomaly_mgal.is_finite()
                && (0.0..=100_000.0).contains(&self.yield_anomaly_mgal),
            "yield_anomaly_mgal",
            "must be in [0, 100000]",
        )?;
        check(
            self.te_ocean_max_km <= 500.0 && self.te_craton_km <= 500.0,
            "te_craton_km",
            "elastic thicknesses must be at most 500 km",
        )?;
        check(
            (1..=1000).contains(&self.every_steps),
            "every_steps",
            "must be in [1, 1000]",
        )?;
        check(
            (1..=10_000).contains(&self.max_iterations),
            "max_iterations",
            "must be in [1, 10000]",
        )?;
        Ok(())
    }

    /// The effective elastic thickness of a cell, km, before smoothing: oceanic crust by its age
    /// (Myr), continental crust by the time since its last orogeny (Myr; NaN or infinite: never),
    /// arcs and rifts at the floor.
    pub fn elastic_thickness_km(
        &self,
        oceanic: bool,
        age_myr: f64,
        since_orogeny_myr: f64,
        weak: bool,
    ) -> f64 {
        let te = if weak {
            0.0
        } else if oceanic {
            (self.te_ocean_km_sqrt_myr * age_myr.max(0.0).sqrt()).min(self.te_ocean_max_km)
        } else if since_orogeny_myr.is_finite() {
            self.te_craton_km * (1.0 - (-since_orogeny_myr.max(0.0) / self.te_recovery_myr).exp())
        } else {
            self.te_craton_km
        };
        te.max(self.te_min_km) * self.elastic_thickness_scale
    }

    /// The law's Te a plate holds after a step of `dt_myr`, km: the law's value `target` where it
    /// is at or above the Te the plate held, `was` (NaN: crust new at the place); below it, `was`
    /// relaxes towards it as e^(−dt / `te_weakening_myr`).
    // ledger: ISO-099
    pub fn weakened_te_km(&self, target: f64, was: f64, dt_myr: f64) -> f64 {
        if was.is_nan() || was <= target || self.te_weakening_myr <= 0.0 {
            target
        } else {
            target + (was - target) * (-dt_myr / self.te_weakening_myr).exp()
        }
    }

    /// The most load a plate holds away from local isostasy, kg/m² (`yield_anomaly_mgal` as a
    /// mass, Δg / 2πG); `None` when off.
    // ledger: ISO-120
    pub fn yield_held_kg_m2(&self) -> Option<f64> {
        (self.yield_anomaly_mgal > 0.0)
            .then(|| self.yield_anomaly_mgal * 1.0e-5 / (std::f64::consts::TAU * G_NEWTON))
    }
}

/// The gravitational constant, m³/(kg s²) (CODATA 2022, NIST: 6.674 30(15) × 10⁻¹¹).
// ledger: ISO-120
pub const G_NEWTON: f64 = 6.674_30e-11;

/// Flexural rigidity over gravity, D / g, kg·m, of a plate `te_km` thick.
pub fn rigidity_per_g(te_km: f64, gravity_m_s2: f64) -> f64 {
    let te = te_km * 1000.0;
    YOUNG_PA * te * te * te / (12.0 * (1.0 - POISSON * POISSON)) / gravity_m_s2
}

/// The flexural parameter α = (4D / ρ_m g)^¼, m, of a rigidity over gravity.
pub fn flexural_parameter_m(d_per_g: f64) -> f64 {
    (4.0 * d_per_g / RHO_MANTLE).powf(0.25)
}

/// Airy share of crust under air and under water: a thickening T sinks k·T. The crust is the
/// plate model's whole column, at the continental crust's mean density (PLT-281, since
/// 2026-10-06; the upper crust's 2,700 before).
// ledger: ISO-046
fn airy_shares() -> (f64, f64) {
    let rho_c = RHO_CRUST_MEAN;
    (
        rho_c / RHO_MANTLE,
        (rho_c - RHO_WATER) / (RHO_MANTLE - RHO_WATER),
    )
}

/// The crust thickening T behind an Airy elevation change `u` (m, either sign) of ground that
/// stood at `z0` before it, the part of the change below the sea compensated under water; and
/// the root already in the elevation, a = T − u. Returns (load in m of mantle, a).
fn thickening(u: f64, z0: f64, sea: f64) -> (f64, f64) {
    if u == 0.0 {
        return (0.0, 0.0);
    }
    let (ka, kw) = airy_shares();
    let (lo, hi) = if u > 0.0 { (z0, z0 + u) } else { (z0 + u, z0) };
    let below = (sea.min(hi) - lo).max(0.0);
    let above = (hi - lo) - below;
    let t = (below / (1.0 - kw) + above / (1.0 - ka)) * u.signum();
    (t * ka, t - u)
}

/// The part of a change `u` (m, either sign) of ground standing at `z0` that lies below the sea at
/// `sea` (negative for a fall): the sea water the change moved.
fn under_sea(u: f64, z0: f64, sea: f64) -> f64 {
    let (lo, hi) = if u > 0.0 { (z0, z0 + u) } else { (z0 + u, z0) };
    (sea.min(hi) - lo).max(0.0) * u.signum()
}

/// The crust T behind a change `uc` (m, either sign) of a column's dry Airy height, and the
/// elevation change e it makes at ground standing at `z0`: a metre of crust raises the ground
/// 1 − k_w under the sea and 1 − k_a in the air ([`airy_shares`]). Returns (load in m of mantle
/// k_a·T, the root already in the elevation a = T − e, e). The inverse of [`thickening`].
fn crust_thickening(uc: f64, z0: f64, sea: f64) -> (f64, f64, f64) {
    if uc == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let (ka, kw) = airy_shares();
    let t = uc / (1.0 - ka);
    let e = if t > 0.0 {
        // Under the sea first, up to the sea level.
        let room = (sea - z0).max(0.0);
        let filling = room / (1.0 - kw);
        if t <= filling {
            t * (1.0 - kw)
        } else {
            room + (t - filling) * (1.0 - ka)
        }
    } else {
        // In the air first, down to the sea level.
        let room = (z0 - sea).max(0.0);
        let emptying = room / (1.0 - ka);
        if -t <= emptying {
            t * (1.0 - ka)
        } else {
            -room + (t + emptying) * (1.0 - kw)
        }
    };
    (t * ka, t - e, e)
}

/// The plate model's uplift at a cell as the flexure books it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Booked {
    /// The load of the crust the column gained or lost, m of mantle …
    load: f64,
    /// … the root already in the elevation …
    root: f64,
    /// … and the sea water that crust's own rise or fall pushed aside, m (its compensation under
    /// the sea, with the roots).
    crust_water: f64,
    /// The sea water the rest of the uplift moved, m (negative for a fall): a height moved alone
    /// (the plate model's settling, a height rule, its rate limit's hold) is no load, and the sea
    /// it lets in or pushes out is none either.
    rest_water: f64,
}

// ledger: ISO-061, ISO-091, ISO-096
/// The plate model's move of the ground at cell `c` this step, as the carry made it: the step's
/// start less the surface's elevation at the crust's source before the step (`from`,
/// [`FlexState::from_m`]; the plate model's uplift where that is unknown). Where the crust builds
/// (continental crust, an arc or a hotspot; not new at the place), the crust it gained or lost is
/// a thickening or a thinning ([`crust_thickening`] of `plates/crust_uplift_mm_yr`), and the rest
/// of the move is a height moved alone, no load. On ocean floor that builds nothing the whole move
/// is a height moved alone: its cooling (GDH1), the pull to a trench and the release from it, the
/// carry onto the trench's bent plate (ISO-097), a gap's floor deepening and the plate that covers
/// it. The sea such a move lets in or pushes out is no load either (`rest_water`): the plate
/// model's height of the ocean floor is its depth under the sea, the water's weight in it (GDH1's
/// fit; Cortial's trench depth; Turcotte & Schubert §3.13: the water fills the room the bending
/// makes, and the plate's restoring force is (ρ_m − ρ_w) g w). Until 2026-10-23 only GDH1's
/// cooling left its sea out: a trench row's pull of 2.4 km in a step sank it by up to
/// ρ_w/(ρ_m − ρ_w) = 0.45 of that more, and a gap's floor that a plate 6 km deeper covered sank
/// 2.4 km more; the plate model took little of either back (its feedback reads sample means) and
/// the deep floor stood 11 to 16 km down (`docs/science/flexure.md`, "The trench rows"). Until
/// 2026-10-22 every rise of a building column was taken as thickening, 1 / (1 − k) = 7 times it,
/// whose root a stiff plate held up: the plate model's settling (ISO-078) moved a column back to
/// its Airy height plus this flexure, the flexure took the move as new crust with a root it held,
/// and the two raised each other without bound (`docs/science/flexure.md`, "The settling's
/// feedback").
fn booked(x: &StepIn<'_>, c: usize, fresh: bool, from: f64) -> Booked {
    if fresh {
        return Booked::default();
    }
    let reg = x.regime[c] as u8;
    let builds = x.crust_type[c] as u8 != OCEANIC || reg == regime::ARC || reg == regime::HOTSPOT;
    let lift = x.dt_myr * 1.0e3; // mm/yr over the step, in m
    let sea = x.sea_at(c);
    let u = if from.is_finite() {
        x.h_start[c] - from
    } else {
        f64::from(x.uplift_mm_yr[c]) * lift
    };
    let uc = if builds {
        f64::from(x.crust_uplift_mm_yr[c]) * lift
    } else {
        0.0
    };
    let z0 = x.h_start[c] - u;
    let (load, root, e) = crust_thickening(uc, z0, sea);
    Booked {
        load,
        root,
        crust_water: under_sea(e, z0, sea),
        rest_water: under_sea(u - e, z0 + e, sea),
    }
}

/// The rise of the surface below the sea (m, negative for a fall) that the plate model's crust
/// and the plateau spreading brought at cell `c` this step, `h_now` the elevation after the
/// spreading: the sea water they pushed aside, which the step's change of sea water counts.
/// `from` as for [`booked`].
/// Measurement only: the gap budget books it with the roots, whose compensation under the sea it
/// is ([`StepOut::roots_m`], [`local_answers`]).
fn built_under_sea(x: &StepIn<'_>, c: usize, fresh: bool, from: f64, h_now: f64) -> f64 {
    let sea = x.sea_at(c);
    // As in `step`: crust the plate model thickened or thinned, and crust the spreading moved.
    let lifted = booked(x, c, fresh, from).crust_water;
    let ds = h_now - x.h_before_spread[c];
    lifted
        + if ds != 0.0 {
            under_sea(ds, x.h_before_spread[c], sea)
        } else {
            0.0
        }
}

/// The surface's flexure state, kept with the crust (`SurfaceState::flexure`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FlexState {
    /// The load since the start of the run, rock and sea water, m of mantle (kg/m² ÷ ρ_m).
    pub load_m: Vec<f32>,
    /// The sinking that goes with it, m (down positive).
    pub deflection_m: Vec<f32>,
    /// The depth of sea counted in `load_m`, m; NaN where the crust is new at the place (its
    /// reference is taken at the next step).
    pub water_m: Vec<f32>,
    /// The rigidity over gravity the cell was last solved with, D / g, kg·m (carried as its
    /// logarithm: ISO-075).
    pub rigidity: Vec<f32>,
    /// The law's elastic thickness the plate held at the last step, km, before the smoothing and
    /// the break at plate boundaries ([`FlexureParams::weakened_te_km`], ISO-099; carried as its
    /// logarithm, as the rigidity is); NaN where the crust is new at the place.
    #[serde(default)]
    pub te_law_km: Vec<f32>,
    /// The land ice the plate carried at the last step, m.
    pub ice_m: Vec<f32>,
    /// The solve's unfinished residual, m of mantle: what the last solve left (under its
    /// tolerance or at its iteration limit), and loads not yet answered (a cone laid on the
    /// surface by a later stage), solved for with the next step's loads.
    pub unsolved_m: Vec<f32>,
    /// The surface's elevation when this state was last brought up to date, m: what later stages
    /// add to it is a load.
    pub seen_elevation_m: Vec<f32>,
    /// Plate steps since the last solve.
    pub since_solve: u32,
    /// The surface's elevation before the step at the crust's source, m (the carry's corners,
    /// [`FlexState::carry`]); NaN where the crust is new at the place. The step's start less this
    /// is the plate model's move of the ground as the carry made it ([`booked`]). Set by the
    /// carry and taken by the next step.
    #[serde(skip)]
    pub from_m: Vec<f32>,
}

impl FlexState {
    fn fresh(n: usize) -> FlexState {
        FlexState {
            load_m: vec![0.0; n],
            deflection_m: vec![0.0; n],
            water_m: vec![f32::NAN; n],
            rigidity: vec![f32::NAN; n],
            te_law_km: vec![f32::NAN; n],
            ice_m: vec![0.0; n],
            unsolved_m: vec![0.0; n],
            seen_elevation_m: Vec::new(),
            since_solve: 0,
            from_m: Vec::new(),
        }
    }

    /// Carries the state with the crust over a plate step (`advect::carried_elevation`'s rule:
    /// only source cells that held the same crust count). `elevation_m` is the surface's elevation
    /// before the step, on the old positions: what other stages added to it since this state last
    /// saw it (the volcano stage's cones) joins the load as rock with no root.
    ///
    /// The carried plate is taken to be in equilibrium, as the real one is: moving it changes no
    /// load. The step solves only for the loads of the step (`step`), so the interpolation of
    /// the carried fields, and crust that starts again where another plate's crust arrived, move
    /// nothing. (Until 2026-10-02 each step solved for the whole load again; the carried state's
    /// interpolation error and the resets then acted as forces and moved the ground by up to
    /// several kilometres a step along margins and near the poles: build log 2026-10-02.)
    pub fn carry(&mut self, x: &Carry<'_>, elevation_m: &[f32]) {
        let n = elevation_m.len();
        if self.load_m.len() != n {
            *self = FlexState::fresh(n);
            return;
        }
        // Each loop here is cell by cell, so it runs in parallel with the same values (P1-19).
        if self.seen_elevation_m.len() == n {
            let (ka, _) = airy_shares();
            self.load_m
                .par_iter_mut()
                .zip(self.unsolved_m.par_iter_mut())
                .zip(elevation_m.par_iter().zip(self.seen_elevation_m.par_iter()))
                .for_each(|((load, unsolved), (&z, &seen))| {
                    let added = z - seen;
                    if added.is_finite() && added.abs() > 1.0e-3 {
                        // A load the plate has not answered yet: it waits in the unsolved
                        // residual.
                        let q = added * ka as f32;
                        *load += q;
                        *unsolved += q;
                    }
                });
        }
        // ledger: ISO-099, ISO-075
        // The law's Te the plate holds goes with its crust, blended as a logarithm as the
        // rigidity is: a blend errs towards the weaker corner, and the next step's law then
        // stiffens it at once, which moves nothing. NaN where no corner held the same crust.
        if self.te_law_km.len() == n {
            let ln_te: Vec<f32> = self.te_law_km.par_iter().map(|&t| t.ln()).collect();
            self.te_law_km = advect::carried_fields(x, [&ln_te])
                .into_par_iter()
                .map(|m| m.map_or(f32::NAN, |[l]| l.exp()))
                .collect();
        } else {
            self.te_law_km = vec![f32::NAN; n];
        }
        // Crust that is new at a place starts unloaded and flat (its water is NaN until the
        // next step takes its reference).
        // ledger: ISO-075
        // The rigidity is carried as its logarithm (game-choice: numerical, agent 2026-10-18):
        // it spans three orders of magnitude between a plate's broken edge and a craton, and
        // a linear blend of the corners reads a cell beside a stiff one as nearly as stiff as
        // that, so a weak cell carried a fraction of a cell would seem to have lost almost all
        // of its rigidity.
        let ln_rigidity: Vec<f32> = self.rigidity.par_iter().map(|&d| d.ln()).collect();
        let moved = advect::carried_fields(
            x,
            [
                &self.water_m,
                &self.load_m,
                &self.deflection_m,
                &ln_rigidity,
                &self.ice_m,
                &self.unsolved_m,
                elevation_m,
            ],
        );
        self.water_m
            .par_iter_mut()
            .zip(self.load_m.par_iter_mut())
            .zip(self.deflection_m.par_iter_mut())
            .zip(self.rigidity.par_iter_mut())
            .zip(self.ice_m.par_iter_mut())
            .zip(self.unsolved_m.par_iter_mut())
            .zip(moved.par_iter())
            .for_each(|((((((w, l), s), r), i), u), m)| {
                let [water, load, sunk, ln_d, ice, unsolved, _] = m
                    .filter(|m| m[..3].iter().chain(&m[4..]).all(|y| y.is_finite()))
                    .unwrap_or([f32::NAN, 0.0, 0.0, f32::NAN, 0.0, 0.0, f32::NAN]);
                *w = water;
                *l = load;
                *s = sunk;
                *r = ln_d.exp();
                *i = ice;
                *u = unsolved;
            });
        // ledger: ISO-096
        self.from_m = moved
            .into_par_iter()
            .map(|m| {
                m.filter(|m| m[..3].iter().chain(&m[4..]).all(|y| y.is_finite()))
                    .map_or(f32::NAN, |m| m[6])
            })
            .collect();
    }
}

/// The share of cells where `v` zigzags at the cell scale: the cell and its next neighbour along
/// x (or along y) are both strict extrema, of opposite kind, each by more than `threshold` on both
/// sides (… low, high, low, high …). A field that is smooth at the cell scale scores 0 whatever
/// its hills and hollows; a checkerboard scores about 1. Over the cells `sel` picks.
pub fn cell_scale_zigzag(
    v: &[f64],
    w: usize,
    h: usize,
    threshold: f64,
    sel: impl Fn(usize) -> bool,
) -> f64 {
    let ext = |l: f64, m: f64, r: f64| {
        (l - m) * (m - r) < 0.0 && (l - m).abs() > threshold && (m - r).abs() > threshold
    };
    let (mut hits, mut count) = (0usize, 0usize);
    for c in 0..w * h {
        if !sel(c) {
            continue;
        }
        let (i, j) = (c % w, c / w);
        let at = |ii: usize, jj: usize| v[jj * w + ii % w];
        let x = ext(at(i + w - 1, j), at(i, j), at(i + 1, j))
            && ext(at(i, j), at(i + 1, j), at(i + 2, j));
        let y = j > 0
            && j + 2 < h
            && ext(at(i, j - 1), at(i, j), at(i, j + 1))
            && ext(at(i, j), at(i, j + 1), at(i, j + 2));
        if x || y {
            hits += 1;
        }
        count += 1;
    }
    hits as f64 / count.max(1) as f64
}

/// What a solve reports.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SolveReport {
    pub iterations: u32,
    /// The last error estimate (the preconditioned residual's largest entry), m.
    pub error_m: f64,
    pub converged: bool,
}

/// The solve raster's metric on the sphere (`row_metric`, ISO-127). The L1 raster is Lambert's
/// cylindrical equal-area projection (x = R λ, y = R sin φ): a cell at latitude φ is cos φ as
/// wide and 1 / cos φ as tall on the sphere as at the equator, its area the same, and the
/// Laplacian in the raster's coordinates is ∂x(cos⁻²φ ∂x) + ∂y(cos²φ ∂y). In finite volumes on
/// the raster: per row the east-west coupling `a` = 1 / cos²φ at the row's middle, and per
/// boundary between rows the north-south coupling `b` = cos²φ there (`b[j]` is the boundary above
/// row j; `b[0]` and `b[h]`, the raster's first and last edge, are 0: closed, as the block
/// metric's rows are). The operator is symmetric, as the block metric's is (the area element is
/// uniform), and at the equator a = b = 1, the block metric.
#[derive(Clone, Debug, PartialEq)]
pub struct RowMetric {
    pub a: Vec<f64>,
    pub b: Vec<f64>,
}

impl RowMetric {
    /// From the sine of the latitude at each boundary between the fine raster's rows (h + 1
    /// values; past a pole they clamp to ±1), `f` fine rows to a row.
    // ledger: ISO-127
    pub fn from_sin_faces(sin_faces: &[f64], f: usize) -> RowMetric {
        let f = f.max(1);
        let h = sin_faces.len().saturating_sub(1) / f;
        let s = |k: usize| sin_faces[k * f].clamp(-1.0, 1.0);
        // A row wholly past a pole has no middle on the planet: its cos²φ is taken as half a
        // full row's height in sin φ, under that of a row that reaches the pole (game-choice:
        // numerical; such a row covers nothing, and its coupling only keeps the operator finite).
        let full = (0..h).map(|j| (s(j + 1) - s(j)).abs()).fold(0.0, f64::max);
        let least = (0.5 * full).max(1e-12);
        let a = (0..h)
            .map(|j| {
                let m = 0.5 * (s(j) + s(j + 1));
                1.0 / (1.0 - m * m).max(least)
            })
            .collect();
        let b = (0..=h)
            .map(|k| {
                if k == 0 || k == h {
                    0.0
                } else {
                    1.0 - s(k) * s(k)
                }
            })
            .collect();
        RowMetric { a, b }
    }

    /// The block metric's couplings (every row a = 1, every inner boundary b = 1).
    #[cfg(test)]
    fn block(h: usize) -> RowMetric {
        RowMetric {
            a: vec![1.0; h],
            b: (0..=h)
                .map(|k| if k == 0 || k == h { 0.0 } else { 1.0 })
                .collect(),
        }
    }
}

/// The exact solve of (d L² + c) x = r for uniform d and c on the metric's raster (ISO-127):
/// after a Fourier transform along x, L is per wavenumber k a symmetric tridiagonal matrix in y
/// (diagonal a_j λ_k − b_j − b_(j+1), off the diagonal b_(j+1), over dx²), so d L² + c is a
/// symmetric positive definite pentadiagonal one, factored as L D Lᵀ and solved per column.
struct RowSolve {
    a: Vec<f64>,
    b: Vec<f64>,
    /// The second difference's Fourier symbol along x per wavenumber, 2 cos(2πk / w) − 2.
    lx: Vec<f64>,
    /// 1 / dx².
    inv: f64,
}

impl RowSolve {
    /// `x` ← (d L_k² + c)⁻¹ `x` for wavenumber `k`; `dd`, `l1`, `l2` are scratch of the column's
    /// length.
    #[allow(clippy::too_many_arguments)]
    fn solve(
        &self,
        k: usize,
        d: f64,
        c: f64,
        x: &mut [Complex64],
        dd: &mut [f64],
        l1: &mut [f64],
        l2: &mut [f64],
    ) {
        let h = x.len();
        let (a, b, inv, lx) = (&self.a, &self.b, self.inv, self.lx[k]);
        // L_k: diagonal t(j), off the diagonal e(j) between j and j + 1.
        let t = |j: usize| (a[j] * lx - b[j] - b[j + 1]) * inv;
        let e = |j: usize| if j + 1 < h { b[j + 1] * inv } else { 0.0 };
        // d L_k² + c, by band: (j, j), (j, j + 1), (j, j + 2).
        let m0 = |j: usize| {
            let up = if j > 0 { e(j - 1) } else { 0.0 };
            d * (t(j) * t(j) + up * up + e(j) * e(j)) + c
        };
        let m1 = |j: usize| d * e(j) * (t(j) + t(j + 1));
        let m2 = |j: usize| d * e(j) * e(j + 1);
        for j in 0..h {
            let (mut p1, mut p2) = (0.0, 0.0);
            if j >= 2 {
                p2 = m2(j - 2) / dd[j - 2];
            }
            if j >= 1 {
                let carried = if j >= 2 {
                    p2 * dd[j - 2] * l1[j - 1]
                } else {
                    0.0
                };
                p1 = (m1(j - 1) - carried) / dd[j - 1];
            }
            let mut dj = m0(j);
            if j >= 1 {
                dj -= p1 * p1 * dd[j - 1];
            }
            if j >= 2 {
                dj -= p2 * p2 * dd[j - 2];
            }
            dd[j] = dj;
            l1[j] = p1;
            l2[j] = p2;
        }
        for j in 0..h {
            let mut v = x[j];
            if j >= 1 {
                v -= x[j - 1] * l1[j];
            }
            if j >= 2 {
                v -= x[j - 2] * l2[j];
            }
            x[j] = v;
        }
        for j in 0..h {
            x[j] /= dd[j];
        }
        for j in (0..h).rev() {
            let mut v = x[j];
            if j + 1 < h {
                v -= x[j + 1] * l1[j + 1];
            }
            if j + 2 < h {
                v -= x[j + 2] * l2[j + 2];
            }
            x[j] = v;
        }
    }
}

/// The transforms that diagonalise the 5-point Laplacian on a raster that wraps in x and is
/// closed at its first and last row: a Fourier transform along x (two real rows at a time, as
/// the real and imaginary part of one complex row), a cosine transform (DCT-II) along y by one
/// complex transform of the column's length (Makhoul's reordering: even entries first, odd ones
/// reversed after them). On the sphere's metric (`rows`, ISO-127) the operator is not the same
/// on every row: there is no transform along y, and the division by d·λ² + c is
/// [`RowSolve::solve`] per wavenumber.
struct Spectral {
    w: usize,
    h: usize,
    nk: usize,
    fx: Arc<dyn Fft<f64>>,
    ix: Arc<dyn Fft<f64>>,
    fy: Arc<dyn Fft<f64>>,
    iy: Arc<dyn Fft<f64>>,
    /// e^(−iπm / 2h).
    twiddle: Vec<Complex64>,
    /// The Laplacian's eigenvalue squared, column-major (k·h + m), 1/m⁴.
    lam2: Vec<f64>,
    /// The sphere's metric (ISO-127): no transform along y, a solve per wavenumber instead.
    rows: Option<RowSolve>,
}

impl Spectral {
    /// [`Self::new`], on the sphere's metric when given.
    fn with_rows(w: usize, h: usize, dx: f64, metric: Option<&RowMetric>) -> Spectral {
        let mut s = Spectral::new(w, h, dx);
        s.rows = metric.map(|m| RowSolve {
            a: m.a.clone(),
            b: m.b.clone(),
            lx: (0..s.nk)
                .map(|k| 2.0 * (std::f64::consts::TAU * k as f64 / w as f64).cos() - 2.0)
                .collect(),
            inv: 1.0 / (dx * dx),
        });
        s
    }

    fn new(w: usize, h: usize, dx: f64) -> Spectral {
        let mut planner = FftPlanner::<f64>::new();
        let nk = w / 2 + 1;
        let twiddle: Vec<Complex64> = (0..h)
            .map(|m| Complex64::from_polar(1.0, -std::f64::consts::PI * m as f64 / (2 * h) as f64))
            .collect();
        let lx: Vec<f64> = (0..nk)
            .map(|k| 2.0 * (std::f64::consts::TAU * k as f64 / w as f64).cos() - 2.0)
            .collect();
        let ly: Vec<f64> = (0..h)
            .map(|m| 2.0 * (std::f64::consts::PI * m as f64 / h as f64).cos() - 2.0)
            .collect();
        let inv = 1.0 / (dx * dx);
        let mut lam2 = vec![0.0; nk * h];
        for k in 0..nk {
            for m in 0..h {
                let l = (lx[k] + ly[m]) * inv;
                lam2[k * h + m] = l * l;
            }
        }
        Spectral {
            w,
            h,
            nk,
            fx: planner.plan_fft_forward(w),
            ix: planner.plan_fft_inverse(w),
            fy: planner.plan_fft_forward(h),
            iy: planner.plan_fft_inverse(h),
            twiddle,
            lam2,
            rows: None,
        }
    }

    /// `spec` (nk × h, column-major) ← the transform of `v`: Σ_i Σ_j v e^(−2πi·ik/w)
    /// cos(π(j + ½)m/h). `half` is scratch (h × nk).
    #[cfg(test)]
    fn forward(&self, v: &[f64], half: &mut [Complex64], spec: &mut [Complex64]) {
        self.forward_with(v, None, None, half, spec);
    }

    /// [`Self::forward`] of `v` × `weight` (cell by cell) when given, each coefficient then divided
    /// by `d`·λ² + `c` when `divide` = (d, c) is given: the same values as weighting, transforming
    /// and dividing in three passes, in one (P1-17).
    fn forward_with(
        &self,
        v: &[f64],
        weight: Option<&[f64]>,
        divide: Option<(f64, f64)>,
        half: &mut [Complex64],
        spec: &mut [Complex64],
    ) {
        let (w, h, nk) = (self.w, self.h, self.nk);
        let zero = Complex64::new(0.0, 0.0);
        // Rows a and b as a + i·b: A[k] = (Z[k] + conj Z[−k]) / 2, B[k] = (Z[k] − conj Z[−k]) / 2i.
        half.par_chunks_mut(2 * nk)
            .zip(v.par_chunks(2 * w))
            .enumerate()
            .for_each_init(
                || (vec![zero; w], vec![zero; self.fx.get_inplace_scratch_len()]),
                |(buf, scratch), (pair, (out, rows))| {
                    let (a, b) = rows.split_at(w);
                    match weight {
                        Some(wt) => {
                            let wr = &wt[2 * pair * w..(2 * pair * w + rows.len())];
                            let (wa, wb) = wr.split_at(w);
                            for (i, z) in buf.iter_mut().enumerate() {
                                *z = Complex64::new(
                                    a[i] * wa[i],
                                    b.get(i).map_or(0.0, |&x| x * wb[i]),
                                );
                            }
                        }
                        None => {
                            for (i, z) in buf.iter_mut().enumerate() {
                                *z = Complex64::new(a[i], b.get(i).copied().unwrap_or(0.0));
                            }
                        }
                    }
                    self.fx.process_with_scratch(buf, scratch);
                    let (oa, ob) = out.split_at_mut(nk);
                    for k in 0..nk {
                        let (z, zc) = (buf[k], buf[(w - k) % w].conj());
                        oa[k] = (z + zc) * 0.5;
                        if let Some(o) = ob.get_mut(k) {
                            let d = z - zc;
                            *o = Complex64::new(d.im, -d.re) * 0.5;
                        }
                    }
                },
            );
        let half = &*half;
        if let Some(rows) = &self.rows {
            // The sphere's metric: the columns as they are, solved per wavenumber (ISO-127).
            spec.par_chunks_mut(h).enumerate().for_each_init(
                || (vec![0.0; h], vec![0.0; h], vec![0.0; h]),
                |(dd, l1, l2), (k, col)| {
                    for (j, x) in col.iter_mut().enumerate() {
                        *x = half[j * nk + k];
                    }
                    if let Some((d, c)) = divide {
                        rows.solve(k, d, c, col, dd, l1, l2);
                    }
                },
            );
            return;
        }
        spec.par_chunks_mut(h).enumerate().for_each_init(
            || (vec![zero; h], vec![zero; self.fy.get_inplace_scratch_len()]),
            |(buf, scratch), (k, col)| {
                for j in 0..h {
                    let x = half[j * nk + k];
                    if j % 2 == 0 {
                        buf[j / 2] = x;
                    } else {
                        buf[h - 1 - j / 2] = x;
                    }
                }
                self.fy.process_with_scratch(buf, scratch);
                // ω_m V[m] = C[m] − i C[h − m].
                col[0] = buf[0];
                for m in 1..h {
                    let p = self.twiddle[m] * buf[m];
                    let q = self.twiddle[h - m] * buf[h - m];
                    col[m] = (p + Complex64::new(-q.im, q.re)) * 0.5;
                }
                if let Some((d, c)) = divide {
                    for (s, &l) in col.iter_mut().zip(&self.lam2[k * h..(k + 1) * h]) {
                        *s /= d * l + c;
                    }
                }
            },
        );
    }

    /// `out` ← the inverse transform of `spec` (overwritten).
    fn inverse(&self, spec: &mut [Complex64], out: &mut [f64]) {
        self.inverse_with(spec, None, out);
    }

    /// [`Self::inverse`], or with `weight` given, `out` += the inverse transform × `weight` (cell
    /// by cell): the same values as transforming into a buffer and adding it, in one pass.
    fn inverse_with(&self, spec: &mut [Complex64], weight: Option<&[f64]>, out: &mut [f64]) {
        let (w, h, nk) = (self.w, self.h, self.nk);
        let zero = Complex64::new(0.0, 0.0);
        let norm_y = 1.0 / h as f64;
        // (on the sphere's metric there is no transform along y, ISO-127)
        if self.rows.is_none() {
            spec.par_chunks_mut(h).for_each_init(
                || (vec![zero; h], vec![zero; self.iy.get_inplace_scratch_len()]),
                |(buf, scratch), col| {
                    buf[0] = col[0];
                    for m in 1..h {
                        let c = col[h - m];
                        buf[m] = self.twiddle[m].conj() * (col[m] + Complex64::new(c.im, -c.re));
                    }
                    self.iy.process_with_scratch(buf, scratch);
                    for j in 0..h {
                        col[j] = if j % 2 == 0 {
                            buf[j / 2]
                        } else {
                            buf[h - 1 - j / 2]
                        } * norm_y;
                    }
                },
            );
        }
        let spec = &*spec;
        let norm_x = 1.0 / w as f64;
        out.par_chunks_mut(2 * w).enumerate().for_each_init(
            || (vec![zero; w], vec![zero; self.ix.get_inplace_scratch_len()]),
            |(buf, scratch), (pair, rows)| {
                let (ja, jb) = (2 * pair, 2 * pair + 1);
                let i1 = Complex64::new(0.0, 1.0);
                for k in 0..nk {
                    let a = spec[k * h + ja];
                    let b = if jb < h { spec[k * h + jb] } else { zero };
                    buf[k] = a + i1 * b;
                    if k > 0 && k < w - k {
                        buf[w - k] = a.conj() + i1 * b.conj();
                    }
                }
                self.ix.process_with_scratch(buf, scratch);
                let (ra, rb) = rows.split_at_mut(w);
                match weight {
                    Some(wt) => {
                        let wr = &wt[ja * w..(ja * w + w + rb.len())];
                        let (wa, wb) = wr.split_at(w);
                        for (i, z) in buf.iter().enumerate() {
                            let p = z.re * norm_x;
                            ra[i] += p * wa[i];
                            if let Some(x) = rb.get_mut(i) {
                                let p = z.im * norm_x;
                                *x += p * wb[i];
                            }
                        }
                    }
                    None => {
                        for (i, z) in buf.iter().enumerate() {
                            ra[i] = z.re * norm_x;
                            if let Some(x) = rb.get_mut(i) {
                                *x = z.im * norm_x;
                            }
                        }
                    }
                }
            },
        );
    }
}

/// The preconditioner: exact solves at up to three uniform rigidities, blended by weights whose
/// squares sum to one.
struct Bank {
    /// D / g of each level, kg·m.
    d: Vec<f64>,
    /// The weight of each level per cell; empty for a single level (weight 1 everywhere).
    weights: Vec<Vec<f64>>,
    c_ref: f64,
}

impl Bank {
    fn new(d: &[f64], c: &[f64], dx: f64) -> Bank {
        let n = d.len();
        let ln_alpha: Vec<f64> = d
            .par_iter()
            .map(|&d| (flexural_parameter_m(d.max(1e-30)) / dx).ln())
            .collect();
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for &a in &ln_alpha {
            lo = lo.min(a);
            hi = hi.max(a);
        }
        let c_ref = c.iter().sum::<f64>() / n.max(1) as f64;
        let node_d = |ln_a: f64| RHO_MANTLE * (ln_a.exp() * dx).powi(4) / 4.0;
        // A plate softer than this is local at the grid's scale whatever its rigidity.
        let floor = 0.4f64.ln(); // ledger: ISO-060 (game-choice: numerical)
        if hi - lo < 0.05 || hi <= floor {
            return Bank {
                d: vec![node_d(0.5 * (lo + hi))],
                weights: Vec::new(),
                c_ref,
            };
        }
        let lo = lo.max(floor);
        const K: usize = 3;
        let nodes: Vec<f64> = (0..K)
            .map(|k| lo + (hi - lo) * k as f64 / (K - 1) as f64)
            .collect();
        let mut weights = vec![vec![0.0; n]; K];
        for (c, &a) in ln_alpha.iter().enumerate() {
            let t = ((a - lo) / (hi - lo)).clamp(0.0, 1.0) * (K - 1) as f64;
            let k0 = (t.floor() as usize).min(K - 2);
            let f = t - k0 as f64;
            weights[k0][c] = (1.0 - f).sqrt();
            weights[k0 + 1][c] = f.sqrt();
        }
        Bank {
            d: nodes.into_iter().map(node_d).collect(),
            weights,
            c_ref,
        }
    }
}

/// The preconditioner of one solve: the bank, the damped Jacobi weights and their scratch.
struct Preconditioner<'a> {
    bank: Bank,
    d: &'a [f64],
    c: &'a [f64],
    /// ω / the operator's diagonal, ω = ½ (the operator's largest eigenvalue over its diagonal
    /// is under 3.2, so the sweep contracts).
    jacobi: Vec<f64>,
    left: Vec<f64>,
    part: Vec<f64>,
}

impl<'a> Preconditioner<'a> {
    fn new(
        d: &'a [f64],
        c: &'a [f64],
        w: usize,
        h: usize,
        dx: f64,
        metric: Option<&RowMetric>,
    ) -> Preconditioner<'a> {
        let bank = Bank::new(d, c, dx);
        let n = w * h;
        let inv = 1.0 / (dx * dx * dx * dx);
        let jacobi = if bank.weights.is_empty() {
            Vec::new()
        } else {
            (0..n)
                .into_par_iter()
                .map(|k| {
                    let (i, j) = (k % w, k / w);
                    let (east, west) = (d[j * w + (i + 1) % w], d[j * w + (i + w - 1) % w]);
                    let (north, south) = (
                        d[j.saturating_sub(1) * w + i],
                        d[(j + 1).min(h - 1) * w + i],
                    );
                    match metric {
                        None => {
                            let around = east + west + north + south;
                            0.5 / (c[k] + (16.0 * d[k] + around) * inv)
                        }
                        // (L D L)'s diagonal on the sphere's metric (ISO-127).
                        Some(m) => {
                            let (a, bn, bs) = (m.a[j], m.b[j], m.b[j + 1]);
                            let centre = 2.0 * a + bn + bs;
                            let diag = d[k] * centre * centre
                                + (east + west) * a * a
                                + north * bn * bn
                                + south * bs * bs;
                            0.5 / (c[k] + diag * inv)
                        }
                    }
                })
                .collect()
        };
        Preconditioner {
            bank,
            d,
            c,
            jacobi,
            left: vec![0.0; n],
            part: vec![0.0; n],
        }
    }
}

/// The flexure solver of one raster: the transforms and the working arrays.
pub struct Solver {
    pub w: usize,
    pub h: usize,
    /// Cell side, m (Earth-normalized).
    pub dx: f64,
    /// The sphere's metric (ISO-127); None: the block metric.
    pub metric: Option<RowMetric>,
    spectral: Spectral,
    half: Vec<Complex64>,
    spec: Vec<Complex64>,
    tmp: Vec<f64>,
}

/// What [`Solver::laplacian_then`] does to each value after the Laplacian.
#[derive(Clone, Copy)]
enum Then<'a> {
    Nothing,
    /// × the value per cell.
    Scale(&'a [f64]),
    /// + c·u per cell, then (with `from`) `from` − that.
    Add(&'a [f64], &'a [f64], Option<&'a [f64]>),
}

/// A sum whose value does not depend on the thread count: per row, then over the rows in order.
fn dot(w: usize, a: &[f64], b: &[f64]) -> f64 {
    let rows: Vec<f64> = a
        .par_chunks(w)
        .zip(b.par_chunks(w))
        .map(|(x, y)| x.iter().zip(y).map(|(p, q)| p * q).sum::<f64>())
        .collect();
    rows.iter().sum()
}

fn max_abs(v: &[f64]) -> f64 {
    v.par_iter().map(|x| x.abs()).reduce(|| 0.0, f64::max)
}

/// [`max_abs`] of `b` and [`dot`] of `a` and `b` in one pass (P1-19): the same values.
fn max_abs_and_dot(w: usize, a: &[f64], b: &[f64]) -> (f64, f64) {
    let rows: Vec<(f64, f64)> = a
        .par_chunks(w)
        .zip(b.par_chunks(w))
        .map(|(x, y)| {
            let peak = y.iter().map(|q| q.abs()).fold(0.0, f64::max);
            (peak, x.iter().zip(y).map(|(p, q)| p * q).sum::<f64>())
        })
        .collect();
    let peak = rows.iter().map(|r| r.0).fold(0.0, f64::max);
    (peak, rows.iter().map(|r| r.1).sum())
}

impl Solver {
    pub fn new(w: usize, h: usize, dx: f64) -> Solver {
        Solver::with_metric(w, h, dx, None)
    }

    /// [`Self::new`], on the sphere's metric when given (ISO-127).
    pub fn with_metric(w: usize, h: usize, dx: f64, metric: Option<RowMetric>) -> Solver {
        let nk = w / 2 + 1;
        let zero = Complex64::new(0.0, 0.0);
        let metric = metric.filter(|m| m.a.len() == h && m.b.len() == h + 1);
        Solver {
            w,
            h,
            dx,
            spectral: Spectral::with_rows(w, h, dx, metric.as_ref()),
            metric,
            half: vec![zero; h * nk],
            spec: vec![zero; nk * h],
            tmp: vec![0.0; w * h],
        }
    }

    /// out ← the 5-point Laplacian of `v` (x wraps, the first and last row are closed), 1/m²: in
    /// the block metric, or on the sphere's ([`RowMetric`], ISO-127).
    pub fn laplacian(&self, v: &[f64], out: &mut [f64]) {
        self.laplacian_then(v, Then::Nothing, out);
    }

    /// [`Self::laplacian`], each value then put through `then` in the same pass (P1-19: a pass
    /// of its own before): the same operations in the same order.
    fn laplacian_then(&self, v: &[f64], then: Then<'_>, out: &mut [f64]) {
        let (w, h) = (self.w, self.h);
        let inv = 1.0 / (self.dx * self.dx);
        let metric = self.metric.as_ref();
        out.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
            let at = |jj: usize| &v[jj * w..(jj + 1) * w];
            let (mid, north, south) = (at(j), at(j.saturating_sub(1)), at((j + 1).min(h - 1)));
            if let Some(m) = metric {
                // ledger: ISO-127
                let (a, bn, bs) = (m.a[j], m.b[j], m.b[j + 1]);
                for i in 0..w {
                    let west = mid[if i == 0 { w - 1 } else { i - 1 }];
                    let east = mid[if i + 1 == w { 0 } else { i + 1 }];
                    let x = mid[i];
                    row[i] =
                        (a * (west + east - 2.0 * x) + bn * (north[i] - x) + bs * (south[i] - x))
                            * inv;
                }
            } else {
                for i in 0..w {
                    let west = mid[if i == 0 { w - 1 } else { i - 1 }];
                    let east = mid[if i + 1 == w { 0 } else { i + 1 }];
                    row[i] = (west + east + north[i] + south[i] - 4.0 * mid[i]) * inv;
                }
            }
            let cells = j * w..(j + 1) * w;
            match then {
                Then::Nothing => {}
                Then::Scale(d) => {
                    for (x, &d) in row.iter_mut().zip(&d[cells]) {
                        *x *= d;
                    }
                }
                Then::Add(c, u, from) => {
                    for (k, x) in cells.zip(row.iter_mut()) {
                        *x += c[k] * u[k];
                        if let Some(r) = from {
                            *x = r[k] - *x;
                        }
                    }
                }
            }
        });
    }

    /// out ← L(D·L v): the plate's elastic restoring force over gravity, kg/m².
    pub fn bend(&mut self, d: &[f64], v: &[f64], out: &mut [f64]) {
        let mut m = std::mem::take(&mut self.tmp);
        self.laplacian_then(v, Then::Scale(d), &mut m);
        self.laplacian(&m, out);
        self.tmp = m;
    }

    /// out ← (L D L + C) v.
    fn operator(&mut self, d: &[f64], c: &[f64], v: &[f64], out: &mut [f64]) {
        self.operator_from(d, c, v, None, out);
    }

    /// [`Self::operator`], or with `from` given, out ← `from` − (L D L + C) v: in two passes
    /// (P1-19: four, five with `from`).
    fn operator_from(
        &mut self,
        d: &[f64],
        c: &[f64],
        v: &[f64],
        from: Option<&[f64]>,
        out: &mut [f64],
    ) {
        let mut m = std::mem::take(&mut self.tmp);
        self.laplacian_then(v, Then::Scale(d), &mut m);
        self.laplacian_then(&m, Then::Add(c, v, from), out);
        self.tmp = m;
    }

    /// z ← the preconditioner applied to `r`: the blended spectral solves between two damped
    /// Jacobi sweeps (a symmetric product, so conjugate gradients still applies). The sweeps take
    /// the cell-scale error that the blend leaves along sharp changes of stiffness (plate
    /// boundaries): measured on a run's own stiffness field, 13 iterations instead of 27.
    fn precondition(&mut self, pre: &mut Preconditioner<'_>, r: &[f64], z: &mut [f64]) {
        if pre.bank.weights.is_empty() {
            self.spectral_solve(&pre.bank, r, z);
            return;
        }
        let mut left = std::mem::take(&mut pre.left);
        let mut part = std::mem::take(&mut pre.part);
        z.par_iter_mut()
            .zip(r.par_iter().zip(pre.jacobi.par_iter()))
            .for_each(|(z, (&r, &j))| *z = r * j);
        self.operator_from(pre.d, pre.c, z, Some(r), &mut left);
        self.spectral_solve(&pre.bank, &left, &mut part);
        z.par_iter_mut()
            .zip(part.par_iter())
            .for_each(|(z, &p)| *z += p);
        self.operator(pre.d, pre.c, z, &mut left);
        z.par_iter_mut()
            .zip(left.par_iter().zip(r.par_iter().zip(pre.jacobi.par_iter())))
            .for_each(|(z, (&l, (&r, &j)))| *z += (r - l) * j);
        pre.left = left;
        pre.part = part;
    }

    /// z ← the blend of the bank's exact solves applied to `r`.
    fn spectral_solve(&mut self, bank: &Bank, r: &[f64], z: &mut [f64]) {
        let mut half = std::mem::take(&mut self.half);
        let mut spec = std::mem::take(&mut self.spec);
        // Each level is weighted, transformed, divided by d·λ² + c, transformed back and added
        // in two passes (P1-17: six before), the same values operation for operation.
        if bank.weights.is_empty() {
            let divide = Some((bank.d[0], bank.c_ref));
            self.spectral
                .forward_with(r, None, divide, &mut half, &mut spec);
            self.spectral.inverse(&mut spec, z);
        } else {
            z.par_iter_mut().for_each(|x| *x = 0.0);
            for (k, wk) in bank.weights.iter().enumerate() {
                let divide = Some((bank.d[k], bank.c_ref));
                self.spectral
                    .forward_with(r, Some(wk), divide, &mut half, &mut spec);
                self.spectral.inverse_with(&mut spec, Some(wk), z);
            }
        }
        self.half = half;
        self.spec = spec;
    }

    /// Solves (L D L + C) x = b from x = 0 by preconditioned conjugate gradients: `d` is D / g
    /// per cell (kg·m), `c` the restoring density (kg/m³), `b` the load (kg/m²); x is in metres.
    /// It stops when the error estimate is under `tolerance_m` or after `max_iterations`.
    pub fn solve(
        &mut self,
        d: &[f64],
        c: &[f64],
        b: &[f64],
        tolerance_m: f64,
        max_iterations: u32,
    ) -> (Vec<f64>, SolveReport) {
        let n = self.w * self.h;
        assert!(d.len() == n && c.len() == n && b.len() == n, "raster size");
        let w = self.w;
        let mut pre = Preconditioner::new(d, c, w, self.h, self.dx, self.metric.as_ref());
        let mut x = vec![0.0; n];
        let mut r = b.to_vec();
        let mut z = vec![0.0; n];
        self.precondition(&mut pre, &r, &mut z);
        let mut report = SolveReport {
            iterations: 0,
            error_m: max_abs(&z),
            converged: true,
        };
        if report.error_m < tolerance_m {
            // Nothing worth a search direction: the preconditioner's answer is the correction.
            return (z, report);
        }
        let mut p = z.clone();
        let mut ap = vec![0.0; n];
        let mut rz = dot(w, &r, &z);
        report.converged = false;
        for it in 1..=max_iterations {
            self.operator(d, c, &p, &mut ap);
            let pap = dot(w, &p, &ap);
            if !(pap.is_finite() && pap > 0.0) {
                break;
            }
            let alpha = rz / pap;
            // (x and r in one pass, P1-19)
            x.par_iter_mut()
                .zip(r.par_iter_mut())
                .zip(p.par_iter().zip(ap.par_iter()))
                .for_each(|((x, r), (&p, &a))| {
                    *x += alpha * p;
                    *r -= alpha * a;
                });
            self.precondition(&mut pre, &r, &mut z);
            report.iterations = it;
            // (the error and the next r·z in one pass, P1-19: r·z unused when this was the last)
            let (error, rz_new) = max_abs_and_dot(w, &r, &z);
            report.error_m = error;
            if report.error_m < tolerance_m {
                report.converged = true;
                break;
            }
            let beta = rz_new / rz;
            rz = rz_new;
            p.par_iter_mut()
                .zip(z.par_iter())
                .for_each(|(p, &z)| *p = z + beta * *p);
        }
        (x, report)
    }
}

/// `passes` passes of the binomial filter (1, 2, 1)/4 along x (wrapping) and y (the edge rows
/// mirrored), in place.
fn binomial(v: &mut [f64], w: usize, h: usize, passes: usize) {
    let mut tmp = vec![0.0; v.len()];
    for _ in 0..passes {
        tmp.par_chunks_mut(w)
            .zip(v.par_chunks(w))
            .for_each(|(out, row)| {
                for i in 0..w {
                    out[i] = 0.25 * row[(i + w - 1) % w] + 0.5 * row[i] + 0.25 * row[(i + 1) % w];
                }
            });
        v.par_chunks_mut(w).enumerate().for_each(|(j, out)| {
            let (n, s) = (j.saturating_sub(1), (j + 1).min(h - 1));
            for i in 0..w {
                out[i] = 0.25 * tmp[n * w + i] + 0.5 * tmp[j * w + i] + 0.25 * tmp[s * w + i];
            }
        });
    }
}

/// The mean of each `f` × `f` block of `v` (w × h; the tests' reference for [`restrict_by`]).
#[cfg(test)]
fn restrict(v: &[f64], w: usize, f: usize) -> Vec<f64> {
    restrict_by(w, v.len() / w, f, |c| v[c])
}

/// [`restrict`] of the values `get` gives the cells of a w × h raster, without the raster: the
/// same sums in the same order, in one pass (P1-19: the step built each fine raster, then
/// restricted it, or copied it where `f` = 1).
fn restrict_by(w: usize, h: usize, f: usize, get: impl Fn(usize) -> f64 + Send + Sync) -> Vec<f64> {
    if f == 1 {
        return (0..w * h).into_par_iter().map(get).collect();
    }
    let ws = w / f;
    let hs = h / f;
    let norm = 1.0 / (f * f) as f64;
    (0..ws * hs)
        .into_par_iter()
        .map(|c| {
            let (i, j) = (c % ws, c / ws);
            let mut s = 0.0;
            for dj in 0..f {
                for di in 0..f {
                    s += get((j * f + dj) * w + i * f + di);
                }
            }
            s * norm
        })
        .collect()
}

/// The coarse cell (`ws` wide, `f` × `f` fine cells each) over fine cell (i, j).
#[inline]
fn coarse_cell(ws: usize, f: usize, i: usize, j: usize) -> usize {
    (j / f) * ws + i / f
}

/// Each coarse value on its `f` × `f` cells (the tests; the step reads [`coarse_cell`]).
#[cfg(test)]
fn prolong_const(v: &[f64], ws: usize, f: usize) -> Vec<f64> {
    if f == 1 {
        return v.to_vec();
    }
    let hs = v.len() / ws;
    let w = ws * f;
    (0..w * hs * f)
        .into_par_iter()
        .map(|c| v[coarse_cell(ws, f, c % w, c / w)])
        .collect()
}

/// Bilinear interpolation from the coarse cells' centres (x wraps, rows clamp).
fn prolong_linear(v: &[f64], ws: usize, f: usize) -> Vec<f64> {
    if f == 1 {
        return v.to_vec();
    }
    let hs = v.len() / ws;
    let w = ws * f;
    (0..w * hs * f)
        .into_par_iter()
        .map(|c| linear_at(v, ws, hs, f, c % w, c / w))
        .collect()
}

/// [`prolong_linear`]'s value at fine cell (i, j) (`v` itself where `f` = 1).
#[inline]
fn linear_at(v: &[f64], ws: usize, hs: usize, f: usize, i: usize, j: usize) -> f64 {
    if f == 1 {
        return v[j * ws + i];
    }
    let x = (i as f64 + 0.5) / f as f64 - 0.5;
    let y = ((j as f64 + 0.5) / f as f64 - 0.5).clamp(0.0, (hs - 1) as f64);
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = (x - x0, y - y0);
    let i0 = (x0 as i64).rem_euclid(ws as i64) as usize;
    let i1 = (i0 + 1) % ws;
    let j0 = y0 as usize;
    let j1 = (j0 + 1).min(hs - 1);
    (v[j0 * ws + i0] * (1.0 - tx) + v[j0 * ws + i1] * tx) * (1.0 - ty)
        + (v[j1 * ws + i0] * (1.0 - tx) + v[j1 * ws + i1] * tx) * ty
}

/// What [`step`] reads besides the state. Slices are per L1 cell.
pub struct StepIn<'a> {
    /// The L1 raster (block metric).
    pub dims: &'a Dims,
    /// The L1 cell's side on the Earth-normalized sphere, m.
    pub norm_dx_m: f64,
    /// The sine of the latitude at each boundary between the raster's rows, h + 1 values (past a
    /// pole: ±1), for the solve on the sphere's metric (`row_metric`, ISO-127); None: the block
    /// metric.
    pub sin_faces: Option<&'a [f64]>,
    /// The plate model's sample spacing on the same sphere, m (0: no filter). The plate model's
    /// uplift carries no information on shorter scales; only the part of its root it resolves
    /// bends the plate (`ledger: ISO-069`).
    pub plate_spacing_m: f64,
    pub gravity_m_s2: f64,
    pub sea_level_m: f64,
    /// The level of the water standing over each cell, m (ISO-126, `water_by_basin`): the sea
    /// level over the open sea, a closed basin's lake level over its lake, −∞ where no water
    /// stands (dry ground, a dry basin under sea level). None: every cell at or under
    /// `sea_level_m` is under the sea.
    pub water_z: Option<&'a [f64]>,
    pub dt_myr: f64,
    /// Elevation at the start of the step: carried with the crust, plus the plate model's uplift.
    pub h_start: &'a [f64],
    /// … after erosion and deposition, before any isostasy.
    pub h_eroded: &'a [f64],
    /// … after the local isostasy's share (`1 − strength`), before the plateau spreading.
    pub h_before_spread: &'a [f64],
    /// `plates/uplift_mm_yr`, `plates/crust_type`, `crust_age` (Myr; negative on continental
    /// crust), `plates/orogeny_age_myr` (NaN: none), `regime` (the regime, not `plates/flags`:
    /// the flags are zero on the steps the plate model resamples its crust).
    pub uplift_mm_yr: &'a [f32],
    /// `plates/crust_uplift_mm_yr`: the part of the uplift that is the column's own crust gained
    /// or lost, its pools' Airy change. Only this is crust thickened or thinned; the rest moved a
    /// height alone and is no load (`ledger: ISO-091`).
    pub crust_uplift_mm_yr: &'a [f32],
    pub crust_type: &'a [f32],
    pub crust_age_myr: &'a [f32],
    pub orogeny_age_myr: &'a [f32],
    pub regime: &'a [f32],
    /// `plates/plate_id`: plates are separate plates, with no strength across their boundaries.
    pub plate_id: &'a [f32],
    /// Land ice, m (`climate/ice_thickness_m`), when a climate stage publishes it.
    pub ice_m: Option<&'a [f32]>,
    /// The `isostasy` setting: the density of eroded and deposited rock over the mantle's.
    pub rock_share: f64,
    /// Also solve, apart, for the step's loads, for the roots already in the elevation and for
    /// the load a weakened plate let go ([`StepOut::loads_m`], [`StepOut::roots_m`],
    /// [`StepOut::released_m`]). Measurement only (the gap budget,
    /// `plates::gap_budget`): the step's own solve and result do not change.
    pub parts: bool,
}

impl StepIn<'_> {
    // ledger: ISO-126
    /// The level of the water over cell `c`, m: the open sea's or its lake's with `water_z`, else
    /// the sea level (−∞: no water).
    fn sea_at(&self, c: usize) -> f64 {
        self.water_z
            .and_then(|w| w.get(c))
            .map_or(self.sea_level_m, |&z| z)
    }
}

/// The surface's elevation before the step at the crust's source at cell `c`
/// ([`FlexState::from_m`]), NaN where it is not known (crust new at the place, no carry).
fn carried_from(from_m: &[f32], c: usize) -> f64 {
    from_m.get(c).map_or(f64::NAN, |&v| f64::from(v))
}

/// What a step did.
#[derive(Clone, Debug, Default)]
pub struct StepOut {
    /// None when the step was not a solve step (`every_steps`).
    pub solve: Option<SolveReport>,
    /// Σ of the elevation changes, m (× the cell area: the volume).
    pub sum_change_m: f64,
    /// The largest elevation change, m.
    pub max_change_m: f64,
    /// `field::FLEXURE`, `field::DEFLECTION`, `field::ELASTIC_THICKNESS`.
    pub flexure_m: Vec<f32>,
    pub deflection_m: Vec<f32>,
    pub elastic_thickness_km: Vec<f32>,
    /// With [`StepIn::parts`], on a solve step: the elevation change, m per cell, the solve gives
    /// the step's loads (rock, sea water, ice) alone, and the one it gives the roots the plate
    /// model and the spreading put in the elevation alone (their load and the sea water they
    /// pushed aside, less the root, and the plate's bending of the root: on land and under the
    /// sea alike only the bending), and the one it gives the load a weakened plate let go.
    /// The step's change less the three is what the loads carried in the unsolved residual (other
    /// stages' loads, the last solve's unfinished part) moved, less what this solve left
    /// unfinished. Empty otherwise.
    pub loads_m: Vec<f32>,
    pub roots_m: Vec<f32>,
    pub released_m: Vec<f32>,
    /// The solve's cells whose plate yielded this step (`yield_anomaly_mgal`, ISO-120).
    pub yielded_cells: usize,
}

/// The load share of the step's eroded and deposited rock, on land and under the sea (m of mantle
/// per m of rock), for the `isostasy` setting `rock_share`.
fn rock_shares(rock_share: f64) -> (f64, f64) {
    let water_share = RHO_WATER / RHO_MANTLE;
    // Sediment under water: the density today's 0.7 factor implies (docs/design/flexure.md 3.2).
    (
        rock_share,
        water_share + 0.7 * rock_share * (1.0 - water_share), // ledger: ISO-047
    )
}

/// The local (Airy) answers to the loads [`step`] adds, per cell, m: what a plate with no
/// strength would give them. `.0` for the step's erosion and deposition (the rock, and the sea
/// water that takes or gives up its room as the surface moves); `.1` for the rest of the change
/// of sea water and land ice over the cell (the sea rising or falling, the surface carried under
/// it), less the water the plate model's thickened crust and the spreading pushed aside, which
/// goes with their roots. Measurement only (the gap budget, `plates::gap_budget`); the solve spreads
/// the same loads over the plate. `state` is the state before the step, `h` the elevation the
/// solve starts from (after the spreading).
pub fn local_answers(
    p: &FlexureParams,
    state: Option<&FlexState>,
    x: &StepIn<'_>,
    h: &[f64],
) -> Vec<(f64, f64)> {
    let n = h.len();
    let water_share = RHO_WATER / RHO_MANTLE;
    let ice_share = RHO_ICE / RHO_MANTLE;
    let (rock_land, rock_sea) = rock_shares(x.rock_share);
    let st = state.filter(|st| st.water_m.len() == n);
    (0..n)
        .into_par_iter()
        .map(|c| {
            let sea = x.sea_at(c);
            // As in `step`: crust new at the place starts from the sea over it, with no ice.
            let age = x.crust_age_myr[c];
            let fresh = st.is_none_or(|st| st.water_m[c].is_nan())
                || (age >= 0.0 && f64::from(age) < x.dt_myr);
            let from = st
                .filter(|st| st.from_m.len() == n)
                .map_or(f64::NAN, |st| carried_from(&st.from_m, c));
            let (water_ref, ice_prev) = match st {
                Some(st) if !fresh => (
                    f64::from(st.water_m[c]) - booked(x, c, fresh, from).rest_water,
                    f64::from(st.ice_m[c]),
                ),
                _ => ((sea - x.h_start[c]).max(0.0), 0.0),
            };
            let (h0, h1) = (x.h_start[c], x.h_eroded[c]);
            let rock = if h1 <= sea { rock_sea } else { rock_land };
            let eroded_water = (sea - h1).max(0.0) - (sea - h0).max(0.0);
            let q_rock = (h1 - h0) * rock + water_share * eroded_water;
            let ice = x.ice_m.map_or(0.0, |v| f64::from(v[c]).max(0.0));
            // The water the plate model's thickened crust and the spreading pushed aside goes with
            // their roots ([`StepOut::roots_m`]).
            let other_water = (sea - h[c]).max(0.0) - water_ref - eroded_water
                + built_under_sea(x, c, fresh, from, h[c]);
            let q_water = water_share * other_water + ice_share * (ice - ice_prev);
            let wet = if h[c] <= sea { 1.0 } else { 0.0 };
            let answer = -p.strength * RHO_MANTLE / (RHO_MANTLE - RHO_WATER * p.strength * wet);
            (answer * q_rock, answer * q_water)
        })
        .collect()
}

/// The metric trace: measurement only, off by default (ISO-127). With it set, every solve is
/// solved again on the other metric (the sphere's for a run in the block metric, the block
/// metric for a run on the sphere's) and one line on stderr compares the two by band of latitude:
/// the rms of the step's deflection on the sphere's metric, the rms and the largest difference of
/// the block metric's from it, m. No rule reads it, and the world it makes is the same.
pub const METRIC_TRACE_ENV: &str = "DEEPTIME_FLEX_METRIC_TRACE";

/// One solve's [`METRIC_TRACE_ENV`] line; `system` is the solve's (d, c, r).
fn metric_trace(
    sv: &Solver,
    faces: &[f64],
    f: usize,
    system: (&[f64], &[f64], &[f64]),
    dw: &[f64],
    p: &FlexureParams,
) {
    let (d, c, r) = system;
    let on_sphere = sv.metric.is_some();
    let other_metric = (!on_sphere).then(|| RowMetric::from_sin_faces(faces, f));
    let mut other = Solver::with_metric(sv.w, sv.h, sv.dx, other_metric);
    let (dw2, rep) = other.solve(d, c, r, p.tolerance_m, p.max_iterations);
    let (sphere, block) = if on_sphere {
        (dw, &dw2[..])
    } else {
        (&dw2[..], dw)
    };
    const BANDS: [f64; 7] = [0.0, 30.0, 50.0, 60.0, 70.0, 80.0, 90.0];
    // Per band: cells, Σ sphere², Σ (block − sphere)², max |block − sphere|.
    let mut acc = [[0.0f64; 4]; 6];
    for j in 0..sv.h {
        let s = 0.5 * (faces[j * f].clamp(-1.0, 1.0) + faces[(j + 1) * f].clamp(-1.0, 1.0));
        let lat = s.asin().to_degrees().abs();
        let band = (0..6).find(|&b| lat < BANDS[b + 1]).unwrap_or(5);
        let a = &mut acc[band];
        for i in 0..sv.w {
            let k = j * sv.w + i;
            let diff = block[k] - sphere[k];
            a[0] += 1.0;
            a[1] += sphere[k] * sphere[k];
            a[2] += diff * diff;
            a[3] = a[3].max(diff.abs());
        }
    }
    let mut line = format!(
        "flexmetric solve={} other_iterations={} other_converged={}",
        if on_sphere { "sphere" } else { "block" },
        rep.iterations,
        u8::from(rep.converged)
    );
    for (b, a) in acc.iter().enumerate() {
        if a[0] > 0.0 {
            line.push_str(&format!(
                " lat{:.0}-{:.0}={:.1}/{:.1}/{:.1}",
                BANDS[b],
                BANDS[b + 1],
                (a[1] / a[0]).sqrt(),
                (a[2] / a[0]).sqrt(),
                a[3]
            ));
        }
    }
    eprintln!("{line}");
}

/// The largest power of two that keeps the solve cell at or under [`SOLVE_CELL_M`] and divides
/// the raster.
pub fn coarsening(w: usize, h: usize, norm_dx_m: f64) -> usize {
    let mut f = 1;
    while norm_dx_m * (2 * f) as f64 <= SOLVE_CELL_M
        && w.is_multiple_of(2 * f)
        && h.is_multiple_of(2 * f)
        && w / (2 * f) >= 16
        && h / (2 * f) >= 8
    {
        f *= 2;
    }
    f
}

/// One step of flexure on the elevations `h` (m; after erosion, the local isostasy's share and
/// the plateau spreading): the step's loads join the state, the plate is solved for the whole
/// load, and `h` changes by `strength` × the sinking not yet in it.
pub fn step(
    p: &FlexureParams,
    solver: &mut Option<Solver>,
    state: &mut Option<FlexState>,
    x: &StepIn<'_>,
    h: &mut [f64],
) -> Result<StepOut, CoreError> {
    let (w, hgt) = (x.dims.w, x.dims.h);
    let n = w * hgt;
    let st = state.get_or_insert_with(|| FlexState::fresh(n));
    if st.load_m.len() != n {
        *st = FlexState::fresh(n);
    }
    // The carry's sources, for this step only ([`FlexState::from_m`]).
    let from_m = std::mem::take(&mut st.from_m);
    let from_m: &[f32] = if from_m.len() == n { &from_m } else { &[] };
    let water_share = RHO_WATER / RHO_MANTLE;
    let ice_share = RHO_ICE / RHO_MANTLE;
    let (rock_land, rock_sea) = rock_shares(x.rock_share);

    // 1. The loads of the step and the roots already in the elevation. Increments only: the
    // carried plate is in equilibrium under the carried load.
    struct Cell {
        /// Crust new at this place: it starts unloaded and flat.
        fresh: bool,
        /// Rock and water load after the step's rock (the water change not yet in), m of mantle.
        load: f64,
        /// The Airy root the plate model or the spreading already put in the elevation, m.
        root: f64,
        /// The plate model's share of it.
        plate_root: f64,
        /// The load of the crust the plate model and the spreading thickened, m of mantle, less
        /// (with `StepIn::parts`) the sea water it pushed aside.
        root_load: f64,
        /// The step's load increment (rock, water, ice), m of mantle.
        dq: f64,
        /// The water reference (the depth counted in `load`), m.
        water: f64,
    }
    let cells: Vec<Cell> = (0..n)
        .into_par_iter()
        .map(|c| {
            // Crust that is new at this place (born this step, or another plate's edge arriving)
            // starts from its state at the start of the step: no load, no sinking, the sea over
            // it as its reference; the plate model's uplift was not added to it.
            let age = x.crust_age_myr[c];
            let fresh = st.water_m[c].is_nan() || (age >= 0.0 && f64::from(age) < x.dt_myr);
            // ledger: ISO-126
            let sea = x.sea_at(c);
            let depth = (sea - h[c]).max(0.0);
            let (prev, water, ice_prev) = if fresh {
                (0.0, (sea - x.h_start[c]).max(0.0), 0.0)
            } else {
                (
                    f64::from(st.load_m[c]),
                    f64::from(st.water_m[c]),
                    f64::from(st.ice_m[c]),
                )
            };
            // ledger: ISO-061, ISO-091, ISO-096
            // Crust the plate model thickened or thinned: orogens, arcs, hotspot volcanism,
            // rifts. The rest of its move of the ground (all of it on ocean floor that builds
            // nothing: its cooling, a trench's pull, a gap's floor) moved a height alone, and the
            // sea it let in or pushed out is no load either ([`booked`]).
            let from = carried_from(from_m, c);
            let b = booked(x, c, fresh, from);
            let water = water - b.rest_water;
            let mut load = prev + b.load;
            let mut root = b.root;
            let plate_root = b.root;
            let mut root_load = b.load;
            // Erosion and deposition.
            let dm = x.h_eroded[c] - x.h_start[c];
            load += dm
                * if x.h_eroded[c] <= sea {
                    rock_sea
                } else {
                    rock_land
                };
            // Crust moved by the plateau spreading.
            let ds = h[c] - x.h_before_spread[c];
            if ds != 0.0 {
                let (q, a) = thickening(ds, x.h_before_spread[c], sea);
                load += q;
                root += a;
                root_load += q;
            }
            if x.parts {
                // The sea water the roots' crust pushed aside goes with the roots (it is in `dq`
                // as a change of water): under the sea their load less their root is then nil
                // too, and only their bending is left.
                root_load -= water_share * built_under_sea(x, c, fresh, from, h[c]);
            }
            let ice = x.ice_m.map_or(0.0, |v| f64::from(v[c]).max(0.0));
            Cell {
                fresh,
                load,
                root,
                plate_root,
                root_load,
                dq: load - prev + water_share * (depth - water) + ice_share * (ice - ice_prev),
                water,
            }
        })
        .collect();

    // 2. The plate's stiffness now.
    let f = coarsening(w, hgt, x.norm_dx_m);
    let (ws, hs) = (w / f, hgt / f);
    let dxs = x.norm_dx_m * f as f64;
    // ledger: ISO-099
    // The law is the plate's thermal state: Te follows the depth of an isotherm, set by the
    // floor's age or the time since the last orogeny, and an arc's or a rift's heat. A plate the
    // law weakens has to warm first, and it does so by conduction over the lithosphere's thermal
    // time; only then does it let go of the load it held (ISO-056), and the ground settles as
    // fast as the geotherm warms, not in the step the law changed (the savanna pit: a craton the
    // Andean arc's orogeny reached gave up 4.3 km of held load in three steps, build log
    // 2026-10-23). A plate the law stiffens joins stress-free at once, as before.
    let law: Vec<f64> = (0..n)
        .into_par_iter()
        .map(|c| {
            let reg = x.regime[c] as u8;
            let target = p.elastic_thickness_km(
                x.crust_type[c] as u8 == OCEANIC,
                f64::from(x.crust_age_myr[c]),
                f64::from(x.orogeny_age_myr[c]),
                reg == regime::ARC || reg == regime::RIFT,
            );
            let was = if cells[c].fresh {
                f64::NAN
            } else {
                st.te_law_km.get(c).map_or(f64::NAN, |&t| f64::from(t))
            };
            p.weakened_te_km(target, was, x.dt_myr)
        })
        .collect();
    st.te_law_km = law.par_iter().map(|&t| t as f32).collect();
    let mut te = restrict_by(w, hgt, f, |c| law[c]);
    let ds = Dims::new(ws, hs, dxs, x.dims.wrap_x);
    box_blur(&ds, &mut te, 2, 2); // ledger: ISO-054 (game-choice: numerical)
    // ledger: ISO-055
    // A plate boundary is a break: a trench's subducting plate does not hold up the arc beside
    // it. The cells along a boundary are at the floor, the next row half way to it.
    let neighbours = |c: usize| {
        let (i, j) = (c % w, c / w);
        [
            j * w + (i + 1) % w,
            j * w + (i + w - 1) % w,
            j.saturating_sub(1) * w + i,
            (j + 1).min(hgt - 1) * w + i,
        ]
    };
    let edge = |c: usize| {
        neighbours(c)
            .iter()
            .any(|&m| x.plate_id[m] != x.plate_id[c])
    };
    let floor = p.te_min_km * p.elastic_thickness_scale;
    // The break as the grid can carry it. A boundary steps from row to row on the raster; a
    // break drawn cell by cell (boundary cells at the floor, their neighbours half way) is that
    // staircase, and it gave the deflection a zigzag along every margin (build log 2026-10-02,
    // the synthetic margin). The boundary cells are blurred instead and the share is taken from
    // that: on a straight boundary the two boundary rows stay at the floor and the next rows
    // carry 0.17 and 0.72 of their stiffness, and along a stepped boundary it varies smoothly.
    let mut near_edge = restrict_by(w, hgt, f, |c| if edge(c) { 1.0 } else { 0.0 });
    box_blur(&ds, &mut near_edge, 1, 2);
    let whole: Vec<f64> = near_edge
        .par_iter()
        .map(|&b| (1.0 - 2.5 * b).clamp(0.0, 1.0)) // ledger: ISO-055
        .collect();
    te.par_iter_mut()
        .zip(whole.par_iter())
        .for_each(|(t, &share)| *t = floor + (*t - floor).max(0.0) * share);
    let te_out: Vec<f32> = (0..n)
        .into_par_iter()
        .map(|c| te[coarse_cell(ws, f, c % w, c / w)] as f32)
        .collect();

    // ledger: ISO-068
    // 3. The residual of the step: ρ_m (ΔQ − a) − L(D L a) for the loads and the roots already
    // in the elevation, the last solve's unfinished part, and the load a weakened plate lets
    // go.
    // ledger: ISO-127
    // The solve on the sphere's metric, or in the block metric.
    let metric = x
        .sin_faces
        .filter(|s| p.row_metric && s.len() == hgt + 1)
        .map(|s| RowMetric::from_sin_faces(s, f));
    if solver
        .as_ref()
        .is_none_or(|s| s.w != ws || s.h != hs || s.dx != dxs || s.metric != metric)
    {
        *solver = Some(Solver::with_metric(ws, hs, dxs, metric));
    }
    let sv = solver.as_mut().expect("built above");
    let pick = |get: &(dyn Fn(usize) -> f64 + Sync)| -> Vec<f64> { restrict_by(w, hgt, f, get) };
    let d_now: Vec<f64> = te
        .par_iter()
        .map(|&t| rigidity_per_g(t, x.gravity_m_s2))
        .collect();
    let carried = |c: usize, v: &[f32]| {
        if cells[c].fresh { 0.0 } else { f64::from(v[c]) }
    };
    let dq = pick(&|c| cells[c].dq + carried(c, &st.unsolved_m));
    let root = pick(&|c| cells[c].root);
    // ledger: ISO-069
    // The root that bends the plate: the plate model's part only as far as the plate model
    // resolves it. Its uplift is interpolated from samples `plate_spacing_m` apart; what varies
    // from cell to cell below that is the sample lattice, not a load (measured: the uplift field
    // zigzags at the cell scale in about 5 % of margin cells), and a stiff plate passed the whole
    // thickening behind it, 7.0 times the uplift, where local isostasy had passed the uplift. The
    // rest stays locally compensated, as the plate model made it.
    // ledger: ISO-070
    // The spreading's root does not bend the plate at all (game-choice: numerical, agent
    // 2026-10-18): its diffusion of the surface is the locally compensated answer of the channel
    // flow (Clark & Royden's law is derived with local isostasy; from-memory, the paper is not
    // read), and its load and root cancel in the residual. A stiff plate that took its change as
    // crust to be held up passed up to 1 / (1 − k) = 7 times it at the cell scale after an
    // implicit step. That split grows wherever the spreading's step is strong at the cell scale
    // (D·Δt·k² above about 0.4), which the cells of large planets reach: a checkerboard of ±50 km
    // a step on the Earth-sized planet (build log 2026-10-18, the surface divergence).
    let bend_part = {
        let mut pf = pick(&|c| cells[c].plate_root);
        if x.plate_spacing_m > dxs {
            // Binomial passes (1, 2, 1)/4: nothing of the two-cell wave survives; their
            // variance (half a cell² each) reaches that of half the sample spacing.
            let half = 0.5 * x.plate_spacing_m / dxs;
            let passes = (2.0 * half * half).ceil().max(1.0) as usize;
            binomial(&mut pf, ws, hs, passes);
        }
        pf
    };
    let wet = pick(&|c| if h[c] <= x.sea_at(c) { 1.0 } else { 0.0 });
    let d_prev = pick(&|c| {
        if cells[c].fresh {
            f64::NAN
        } else {
            f64::from(st.rigidity[c])
        }
    });
    // ledger: ISO-056
    // Rigidity gained as crust cools joins stress-free: nothing moves, and later loads meet the
    // stiffer plate. Rigidity lost (reheating, a cell becoming a plate's broken edge) gives up
    // the same share of the stress the plate held: where D falls to r·D the plate stops holding
    // up (1 − r) of the load it kept away from local isostasy, ρ_m (Q + ice − w − unsolved),
    // and that is a force. It moves the ground towards local isostasy, where a plate with no
    // strength puts it, and no further, whatever the cell's size or latitude. The law's part of
    // a loss comes as the plate warms (ISO-099), a boundary's break at once. (The moment it
    // replaces, L((1 − r)·M), is the same force where r is uniform; where r changes from cell to
    // cell, along a plate boundary or the edge of a reheated zone, its curvature made forces of
    // several times the load held: the polar holes and spikes, build log 2026-10-18.)
    let held = pick(&|c| {
        if cells[c].fresh {
            0.0
        } else {
            RHO_MANTLE
                * (f64::from(st.load_m[c]) + ice_share * f64::from(st.ice_m[c])
                    - f64::from(st.deflection_m[c])
                    - f64::from(st.unsolved_m[c]))
        }
    });
    let let_go: Vec<f64> = (0..ws * hs)
        .into_par_iter()
        .map(|c| {
            let ratio = d_now[c] / d_prev[c];
            if ratio.is_finite() && ratio < 1.0 {
                (1.0 - ratio) * held[c]
            } else {
                0.0
            }
        })
        .collect();
    let mut bend_root = vec![0.0; ws * hs];
    sv.bend(&d_now, &bend_part, &mut bend_root);
    let r: Vec<f64> = (0..ws * hs)
        .into_par_iter()
        .map(|c| RHO_MANTLE * (dq[c] - root[c]) - bend_root[c] + let_go[c])
        .collect();
    let c_rest: Vec<f64> = wet
        .par_iter()
        .map(|&s| RHO_MANTLE - RHO_WATER * p.strength * s)
        .collect();

    st.since_solve += 1;
    let solve_now = st.since_solve >= p.every_steps;
    let mut out = StepOut {
        elastic_thickness_km: te_out,
        ..StepOut::default()
    };
    let (dw, left) = if solve_now {
        st.since_solve = 0;
        let (dw, report) = sv.solve(&d_now, &c_rest, &r, p.tolerance_m, p.max_iterations);
        if dw.iter().any(|v| !v.is_finite()) {
            return Err(CoreError::InvalidState(
                "the flexure solve produced a non-finite deflection".into(),
            ));
        }
        out.solve = Some(report);
        if let Some(faces) = x.sin_faces.filter(|s| s.len() == hgt + 1)
            && std::env::var_os(METRIC_TRACE_ENV).is_some()
        {
            metric_trace(sv, faces, f, (&d_now, &c_rest, &r), &dw, p);
        }
        if x.parts {
            // The step's loads, the roots and the load let go, each solved alone, more
            // tightly than the step (the solve is linear; the rest of the step's change is the
            // loads carried in the unsolved residual less this solve's unfinished part).
            let rl = pick(&|c| cells[c].root_load);
            let dq_step = pick(&|c| cells[c].dq - cells[c].root_load);
            let r_roots: Vec<f64> = (0..ws * hs)
                .into_par_iter()
                .map(|c| RHO_MANTLE * (rl[c] - root[c]) - bend_root[c])
                .collect();
            let r_loads: Vec<f64> = dq_step.par_iter().map(|&q| RHO_MANTLE * q).collect();
            let (tol, iters) = (0.1 * p.tolerance_m, 4 * p.max_iterations);
            let mut answer = |r: &[f64]| -> Vec<f32> {
                let (dw, _) = sv.solve(&d_now, &c_rest, r, tol, iters);
                prolong_linear(&dw, ws, f)
                    .iter()
                    .map(|&x| (-p.strength * x) as f32)
                    .collect()
            };
            out.loads_m = answer(&r_loads);
            out.roots_m = answer(&r_roots);
            out.released_m = answer(&let_go);
        }
        let mut adw = vec![0.0; ws * hs];
        sv.operator(&d_now, &c_rest, &dw, &mut adw);
        let left: Vec<f64> = r
            .iter()
            .zip(&adw)
            .map(|(a, b)| (a - b) / RHO_MANTLE)
            .collect();
        (dw, left)
    } else {
        // Between solves the step's loads wait in the unsolved residual.
        (
            vec![0.0; ws * hs],
            r.iter().map(|&v| v / RHO_MANTLE).collect(),
        )
    };
    // (the coarse rigidity, residual and deflection read per cell below: `prolong_const` and
    // `prolong_linear` without their fine rasters, P1-19)

    // 4. The elevation takes the sinking that is not yet in it; the state takes the step. Cell
    // by cell, rows in parallel; the sum of the changes is then taken in cell order, as before
    // (P1-17).
    let mut flex = vec![0.0f32; n];
    let mut changes = vec![0.0f64; n];
    // ledger: ISO-120
    // The plate's strength (off unless `yield_anomaly_mgal` is set). An elastic plate holds any
    // load; a real one yields where it is bent hard: its elastic thickness falls to about half at
    // a curvature of 1e-6 per m (Watts & Burov 2003), and no plate on Earth holds more than about
    // 3 km of rock away from local isostasy (the largest free-air anomalies over held loads,
    // ISO-093). Where the load a cell's plate holds after the solve, ρ_m (Q + ice − w), passes
    // that bound, the plate there has yielded: the rest of the load sinks (or rises) under the
    // cell alone, as on a plate with no strength, and the carried plate takes the result as its
    // state (ISO-068). Without it a narrow column the collapse fed crust step after step stood up
    // to 28 km over its Airy height on a stiff plate, and the plate model's settling (ISO-078)
    // followed it (docs/science/plates.md, "The polar peaks").
    let yield_bound = p.yield_held_kg_m2();
    let mut yielded = vec![0.0f32; if yield_bound.is_some() { n } else { 0 }];
    let mut yielded_rows: Vec<&mut [f32]> = if yield_bound.is_some() {
        yielded.chunks_mut(w).collect()
    } else {
        (0..hgt).map(|_| <&mut [f32]>::default()).collect()
    };
    (
        h.par_chunks_mut(w),
        st.load_m.par_chunks_mut(w),
        st.deflection_m.par_chunks_mut(w),
        st.water_m.par_chunks_mut(w),
        st.ice_m.par_chunks_mut(w),
        st.rigidity.par_chunks_mut(w),
        st.unsolved_m.par_chunks_mut(w),
        flex.par_chunks_mut(w),
        changes.par_chunks_mut(w),
        yielded_rows.par_iter_mut(),
    )
        .into_par_iter()
        .enumerate()
        .for_each(|(j, (hr, lr, dr, wr, ir, rr, ur, fr, cr, yr))| {
            for i in 0..w {
                let c = j * w + i;
                let cell = &cells[c];
                let coarse = coarse_cell(ws, f, i, j);
                let delta = linear_at(&dw, ws, hs, f, i, j);
                let change = -p.strength * delta;
                hr[i] += change;
                cr[i] = change;
                let sea = x.sea_at(c);
                let depth = (sea - hr[i]).max(0.0);
                let w_prev = if cell.fresh { 0.0 } else { f64::from(dr[i]) };
                let mut w_new = w_prev + cell.root + delta;
                let load = cell.load + water_share * (depth - cell.water);
                let ice = x.ice_m.map_or(0.0, |v| f64::from(v[c]).max(0.0));
                let mut holds = RHO_MANTLE * (load + ice_share * ice - w_new);
                if let Some(bound) = yield_bound
                    && holds.abs() > bound
                {
                    // ledger: ISO-120
                    let sink = (holds - bound * holds.signum()) / RHO_MANTLE;
                    hr[i] -= p.strength * sink;
                    cr[i] -= p.strength * sink;
                    w_new += sink;
                    holds -= RHO_MANTLE * sink;
                    yr[i] = sink as f32;
                }
                lr[i] = load as f32;
                dr[i] = w_new as f32;
                wr[i] = depth as f32;
                ir[i] = ice as f32;
                rr[i] = d_now[coarse] as f32;
                ur[i] = left[coarse] as f32;
                let restoring = if hr[i] <= sea {
                    RHO_MANTLE - RHO_WATER
                } else {
                    RHO_MANTLE
                };
                fr[i] = (holds / restoring) as f32;
            }
        });
    // The sum in cell order on this thread, beside the copies (P1-19). The seen elevation is
    // written over the last one where it has the size, so the copy of the deflection is the only
    // new grid here (two side by side went over the memory cap).
    let hh: &[f64] = h;
    let mut seen = std::mem::take(&mut st.seen_elevation_m);
    let deflection_now: &[f32] = &st.deflection_m;
    let ((sum, peak), ((), deflection)) = rayon::join(
        || {
            let (mut sum, mut peak) = (0.0, 0.0f64);
            for &change in &changes {
                sum += change;
                peak = peak.max(change.abs());
            }
            (sum, peak)
        },
        || {
            rayon::join(
                || {
                    if seen.len() == hh.len() {
                        seen.par_iter_mut()
                            .zip(hh.par_iter())
                            .for_each(|(s, &z)| *s = z as f32);
                    } else {
                        seen = hh.par_iter().map(|&z| z as f32).collect();
                    }
                },
                || crate::alloc::copied(deflection_now),
            )
        },
    );
    st.seen_elevation_m = seen;
    if !yielded.is_empty() {
        out.yielded_cells = yielded.iter().filter(|&&v| v != 0.0).count();
        // The gap budget's split: what the plate let go as it yielded is load let go.
        if out.released_m.len() == n {
            for (r, &v) in out.released_m.iter_mut().zip(&yielded) {
                *r -= (p.strength as f32) * v;
            }
        }
    }
    out.sum_change_m = sum;
    out.max_change_m = peak;
    out.deflection_m = deflection;
    out.flexure_m = flex;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const G: f64 = 9.81;

    fn uniform(n: usize, te_km: f64, g: f64) -> (Vec<f64>, Vec<f64>) {
        (vec![rigidity_per_g(te_km, g); n], vec![RHO_MANTLE; n])
    }

    /// Kelvin's kei(0) = −π/4: under a point load P the plate sinks P·l² / (8D), with
    /// l = (D / ρ_m g)^¼; in the solve's units a mass m sinks m / (8 ρ_m l²). The transforms
    /// invert each other.
    #[test]
    fn a_point_load_sinks_by_the_analytic_depth() {
        let (w, h, dx) = (256usize, 128usize, 10_000.0);
        // The transforms: forward then inverse is the identity, on even and on odd rasters.
        for (tw, th) in [(w, h), (15, 9), (12, 7)] {
            let ts = Solver::new(tw, th, dx);
            let v: Vec<f64> = (0..tw * th)
                .map(|c| ((c * 7919) % 101) as f64 - 50.0)
                .collect();
            let mut back = vec![0.0; tw * th];
            let (mut half, mut spec) = (ts.half.clone(), ts.spec.clone());
            ts.spectral.forward(&v, &mut half, &mut spec);
            ts.spectral.inverse(&mut spec, &mut back);
            let worst = v
                .iter()
                .zip(&back)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f64::max);
            assert!(worst < 1e-9, "transform round trip {tw} × {th}: {worst}");
        }

        let mut sv = Solver::new(w, h, dx);
        let te = 30.0;
        let (d, c) = uniform(w * h, te, G);
        let l = (d[0] / RHO_MANTLE).powf(0.25);
        assert!(l / dx > 4.0, "the load is a point: l = {l}");
        let mass = 1.0e15; // kg
        let mut b = vec![0.0; w * h];
        let centre = (h / 2) * w + w / 2;
        b[centre] = mass / (dx * dx);
        let (x, rep) = sv.solve(&d, &c, &b, 1e-9, 50);
        assert!(rep.converged && rep.iterations <= 1, "{rep:?}");
        let analytic = mass / (8.0 * RHO_MANTLE * l * l);
        assert!(
            (x[centre] / analytic - 1.0).abs() < 0.03,
            "depth under the load {} against {analytic}",
            x[centre]
        );
        // Compensation: the mantle pushed aside weighs what the load does.
        let displaced: f64 = x.iter().sum::<f64>() * RHO_MANTLE * dx * dx;
        assert!((displaced / mass - 1.0).abs() < 1e-9, "{displaced}");
        // A bulge rings the hollow.
        let lowest = x.iter().cloned().fold(f64::MAX, f64::min);
        assert!(lowest < 0.0 && lowest > -0.05 * analytic, "bulge {lowest}");
    }

    /// The depth under a line load and the place of its first zero crossing, by linear
    /// interpolation, m.
    fn line_load(te_km: f64, g: f64, dx: f64) -> (f64, f64, f64, f64, f64) {
        let (w, h) = (16usize, 1024usize);
        let mut sv = Solver::new(w, h, dx);
        let (d, c) = uniform(w * h, te_km, g);
        let alpha = flexural_parameter_m(d[0]);
        let per_m = 1.0e10; // kg per metre of line
        let row = h / 2;
        let mut b = vec![0.0; w * h];
        for i in 0..w {
            b[row * w + i] = per_m / dx;
        }
        let (x, rep) = sv.solve(&d, &c, &b, 1e-9, 50);
        assert!(rep.converged, "{rep:?}");
        let profile: Vec<f64> = (row..h).map(|j| x[j * w]).collect();
        let w0 = profile[0];
        let k = profile.iter().position(|&v| v < 0.0).expect("a bulge");
        let zero = (k as f64 - 1.0 + profile[k - 1] / (profile[k - 1] - profile[k])) * dx;
        let (kb, wb) = profile
            .iter()
            .enumerate()
            .fold((0, 0.0), |a, (i, &v)| if v < a.1 { (i, v) } else { a });
        (
            w0,
            zero,
            kb as f64 * dx,
            wb,
            per_m / (2.0 * RHO_MANTLE * alpha),
        )
    }

    /// Turcotte & Schubert's line load: w₀ = V α³ / 8D, zero at 3πα/4, the bulge at πα with
    /// −e^(−π) = −4.3 % of the depth; and the width goes as Te^¾ and as g^(−¼).
    #[test]
    fn a_line_load_gives_the_analytic_profile() {
        let dx = 5000.0;
        let te = 30.0;
        let alpha = flexural_parameter_m(rigidity_per_g(te, G));
        assert!((alpha / 1000.0 - 67.0).abs() < 2.0, "α = {alpha}");
        let (w0, zero, bulge_at, bulge, analytic) = line_load(te, G, dx);
        assert!(
            (w0 / analytic - 1.0).abs() < 0.01,
            "{w0} against {analytic}"
        );
        let pi = std::f64::consts::PI;
        assert!(
            (zero / (0.75 * pi * alpha) - 1.0).abs() < 0.02,
            "zero crossing at {zero}, 3πα/4 = {}",
            0.75 * pi * alpha
        );
        assert!(
            (bulge_at - pi * alpha).abs() <= 1.5 * dx,
            "bulge at {bulge_at}, πα = {}",
            pi * alpha
        );
        assert!(
            (bulge / w0 + (-pi).exp()).abs() < 0.003,
            "bulge height {} of the depth",
            bulge / w0
        );
        // Twice the elastic thickness: 2^¾ = 1.68 times the width.
        let (_, zero2, ..) = line_load(2.0 * te, G, dx);
        assert!(
            (zero2 / zero - 2.0f64.powf(0.75)).abs() < 0.02,
            "width ratio {}",
            zero2 / zero
        );
        // Mars's gravity: wider by (9.81 / 3.71)^¼ = 1.275, and so shallower under the line.
        let (w0m, zerom, ..) = line_load(te, 3.71, dx);
        assert!(
            (zerom / zero - (G / 3.71f64).powf(0.25)).abs() < 0.02,
            "{}",
            zerom / zero
        );
        assert!(w0m < w0);
    }

    fn patchy(w: usize, h: usize, lo: f64, hi: f64) -> Vec<f64> {
        // Stiff "cratons" in a soft plate, smoothed as the stage smooths Te.
        let mut te: Vec<f64> = (0..w * h)
            .map(|c| {
                let (i, j) = (c % w, c / w);
                if ((i / 24) + (j / 24)).is_multiple_of(3) {
                    hi
                } else {
                    lo
                }
            })
            .collect();
        box_blur(&Dims::new(w, h, 1.0, true), &mut te, 2, 2);
        te
    }

    /// With a stiffness that varies a thousandfold the solve converges, the answer satisfies the
    /// equation, and the buoyancy of the deflection balances the load, under water too.
    #[test]
    fn a_patchy_plate_converges_and_compensates() {
        let (w, h, dx) = (192usize, 96usize, 39_000.0);
        let mut sv = Solver::new(w, h, dx);
        let te = patchy(w, h, 6.0, 70.0);
        let d: Vec<f64> = te.iter().map(|&t| rigidity_per_g(t, G)).collect();
        let c: Vec<f64> = (0..w * h)
            .map(|k| {
                if (k % w) < w / 3 {
                    RHO_MANTLE - RHO_WATER
                } else {
                    RHO_MANTLE
                }
            })
            .collect();
        let mut b = vec![0.0; w * h];
        for (k, load) in b.iter_mut().enumerate() {
            let (i, j) = (k % w, k / w);
            if (i % 37 < 3 && j % 29 < 9) || (i + 2 * j) % 61 == 0 {
                *load = RHO_MANTLE * (500.0 + ((k * 31) % 17) as f64 * 250.0);
            }
        }
        let (x, rep) = sv.solve(&d, &c, &b, 0.01, 400);
        assert!(rep.converged && rep.iterations < 150, "{rep:?}");
        // The answer satisfies the equation: what is left of the load asks for no more sinking.
        let mut ax = vec![0.0; w * h];
        sv.operator(&d, &c, &x, &mut ax);
        let left: Vec<f64> = b.iter().zip(&ax).map(|(b, a)| b - a).collect();
        let mut more = vec![0.0; w * h];
        sv.spectral_solve(&Bank::new(&d, &c, dx), &left, &mut more);
        let worst = max_abs(&more);
        assert!(worst < 0.05, "sinking still asked for, m: {worst}");
        let load: f64 = b.iter().sum();
        let buoyancy: f64 = x.iter().zip(&c).map(|(x, c)| x * c).sum();
        assert!(
            (buoyancy / load - 1.0).abs() < 1e-3,
            "compensation {}",
            buoyancy / load
        );
        // A load on the soft plate sinks nearly where it lies; on a craton far less.
        let airy_wet = RHO_MANTLE / (RHO_MANTLE - RHO_WATER);
        assert!(x.iter().cloned().fold(f64::MIN, f64::max) < 4500.0 * airy_wet);
    }

    /// The fused blend of the spectral solves (P1-17) is the six-pass one it replaced, bit for bit,
    /// on a patchy plate (three levels) and a uniform one (one level), rows odd and even in
    /// number, at 1, 3 and 8 threads.
    #[test]
    fn the_fused_spectral_blend_is_the_six_pass_one_bit_for_bit() {
        let old = |sp: &Spectral, bank: &Bank, r: &[f64]| -> Vec<f64> {
            let n = r.len();
            let zero = Complex64::new(0.0, 0.0);
            let mut half = vec![zero; sp.h * sp.nk];
            let mut spec = vec![zero; sp.nk * sp.h];
            let divide = |spec: &mut [Complex64], d: f64| {
                for (s, &l) in spec.iter_mut().zip(&sp.lam2) {
                    *s /= d * l + bank.c_ref;
                }
            };
            let mut z = vec![0.0; n];
            if bank.weights.is_empty() {
                sp.forward(r, &mut half, &mut spec);
                divide(&mut spec, bank.d[0]);
                sp.inverse(&mut spec, &mut z);
                return z;
            }
            let mut part = vec![0.0; n];
            for (k, wk) in bank.weights.iter().enumerate() {
                for c in 0..n {
                    part[c] = r[c] * wk[c];
                }
                sp.forward(&part, &mut half, &mut spec);
                divide(&mut spec, bank.d[k]);
                sp.inverse(&mut spec, &mut part);
                for c in 0..n {
                    z[c] += part[c] * wk[c];
                }
            }
            z
        };
        let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<u64>>();
        for (w, h) in [(96usize, 48usize), (64, 37)] {
            let dx = 39_000.0;
            for te in [patchy(w, h, 6.0, 70.0), vec![30.0; w * h]] {
                let d: Vec<f64> = te.iter().map(|&t| rigidity_per_g(t, G)).collect();
                let c = vec![RHO_MANTLE; w * h];
                let r: Vec<f64> = (0..w * h)
                    .map(|k| ((k * 37) % 101) as f64 - 50.0 + (k as f64 * 0.01).sin())
                    .collect();
                let bank = Bank::new(&d, &c, dx);
                let mut sv = Solver::new(w, h, dx);
                let want = bits(&old(&sv.spectral, &bank, &r));
                for threads in [1, 3, 8] {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(threads)
                        .build()
                        .unwrap();
                    let mut z = vec![0.0; w * h];
                    pool.install(|| sv.spectral_solve(&bank, &r, &mut z));
                    assert!(bits(&z) == want, "{w}×{h}, {threads} threads");
                }
            }
        }
    }

    /// The weakest and the stiffest plates the editor allows (scale 0.25 and 3 on 5–80 km), and
    /// beyond: finite, converged, compensated.
    #[test]
    fn the_solve_is_stable_at_the_editors_extremes() {
        let (w, h, dx) = (128usize, 64usize, 39_000.0);
        let mut sv = Solver::new(w, h, dx);
        for (lo, hi) in [(1.25, 20.0), (15.0, 240.0), (0.1, 1.0), (300.0, 500.0)] {
            let te = patchy(w, h, lo, hi);
            let d: Vec<f64> = te.iter().map(|&t| rigidity_per_g(t, G)).collect();
            let c = vec![RHO_MANTLE; w * h];
            let mut b = vec![0.0; w * h];
            for load in b.iter_mut().step_by(53) {
                *load = RHO_MANTLE * 3000.0;
            }
            let (x, rep) = sv.solve(&d, &c, &b, 0.1, 400);
            assert!(rep.converged, "Te {lo}–{hi}: {rep:?}");
            assert!(x.iter().all(|v| v.is_finite()));
            let ratio = x.iter().sum::<f64>() * RHO_MANTLE / b.iter().sum::<f64>();
            assert!((ratio - 1.0).abs() < 1e-3, "Te {lo}–{hi}: {ratio}");
            assert!(x.iter().cloned().fold(f64::MIN, f64::max) <= 3000.0 * 1.001);
        }
    }

    #[test]
    fn the_elastic_thickness_follows_the_crusts_thermal_state() {
        let p = FlexureParams::default();
        assert_eq!(p.elastic_thickness_km(true, 0.0, f64::NAN, false), 5.0);
        assert!((p.elastic_thickness_km(true, 100.0, f64::NAN, false) - 27.0).abs() < 1e-9);
        assert_eq!(p.elastic_thickness_km(true, 400.0, f64::NAN, false), 40.0);
        assert_eq!(p.elastic_thickness_km(false, -1.0, f64::NAN, false), 80.0);
        assert_eq!(p.elastic_thickness_km(false, -1.0, 0.0, false), 5.0);
        let mid = p.elastic_thickness_km(false, -1.0, 300.0, false);
        assert!((mid - 80.0 * (1.0 - (-1.0f64).exp())).abs() < 1e-9);
        assert_eq!(p.elastic_thickness_km(false, -1.0, f64::NAN, true), 5.0);
        let stiff = FlexureParams {
            elastic_thickness_scale: 2.0,
            ..p.clone()
        };
        assert_eq!(
            stiff.elastic_thickness_km(false, -1.0, f64::NAN, false),
            160.0
        );
        p.validate().unwrap();
        assert!(FlexureParams { strength: 1.5, ..p }.validate().is_err());
        // The Airy shares are the plate model's: 1 km of uplift on land is 7.0 km of crust.
        let (q, a) = thickening(1000.0, 500.0, 0.0);
        let t = 1000.0 * 3300.0 / (3300.0 - 2830.0);
        assert!((a - (t - 1000.0)).abs() < 1e-6 && (q - t * 2830.0 / 3300.0).abs() < 1e-6);
        // Under water the root is smaller, and a change that crosses the sea is split.
        let (_, under) = thickening(1000.0, -3000.0, 0.0);
        assert!(under < a && under > 2000.0);
        let (_, across) = thickening(1000.0, -500.0, 0.0);
        assert!((across - 0.5 * (a + under)).abs() < 1e-6);
        let (qn, an) = thickening(-1000.0, 1500.0, 0.0);
        assert!((qn + q).abs() < 1e-6 && (an + a).abs() < 1e-6);
        // The crust behind a dry Airy change is the inverse: the same load and root, and the
        // elevation change that made them, on land, under the sea and across it either way.
        let (ka, _) = airy_shares();
        for (u, z0) in [
            (1000.0, 500.0),
            (1000.0, -3000.0),
            (1000.0, -500.0),
            (-1000.0, 1500.0),
            (-1000.0, 300.0),
            (-1000.0, -2000.0),
        ] {
            let (q, a) = thickening(u, z0, 0.0);
            let (qc, ac, e) = crust_thickening(q / ka * (1.0 - ka), z0, 0.0);
            assert!(
                (qc - q).abs() < 1e-6 && (ac - a).abs() < 1e-6 && (e - u).abs() < 1e-6,
                "{u} at {z0}: ({qc}, {ac}, {e}) against ({q}, {a}, {u})"
            );
        }
    }

    struct World {
        d: Dims,
        norm_dx: f64,
        plate_spacing: f64,
        h: Vec<f64>,
        uplift: Vec<f32>,
        crust: Vec<f32>,
        age: Vec<f32>,
        orogeny: Vec<f32>,
        regime: Vec<f32>,
        plate: Vec<f32>,
        ice: Vec<f32>,
        /// The share of the uplift that is crust (1: all of it; 0: a height moved alone).
        crust_share: f64,
        /// The sines at the rows' boundaries (`StepIn::sin_faces`), None: the block metric only.
        sin_faces: Option<Vec<f64>>,
        /// The water over each cell (`StepIn::water_z`), None: the sea over every cell under it.
        water_z: Option<Vec<f64>>,
    }

    impl World {
        /// A continent of cratons at `z` m everywhere.
        fn new(w: usize, h: usize, z: f64) -> World {
            let n = w * h;
            World {
                d: Dims::new(w, h, 4800.0, true),
                norm_dx: 39_000.0,
                plate_spacing: 0.0,
                h: vec![z; n],
                uplift: vec![0.0; n],
                crust: vec![1.0; n],
                age: vec![-1.0; n],
                orogeny: vec![f32::NAN; n],
                regime: vec![f32::from(regime::CRATON); n],
                plate: vec![0.0; n],
                ice: vec![0.0; n],
                crust_share: 1.0,
                sin_faces: None,
                water_z: None,
            }
        }

        /// One step: `eroded` is the surface's net change.
        fn step(
            &mut self,
            p: &FlexureParams,
            sv: &mut Option<Solver>,
            st: &mut Option<FlexState>,
            eroded: &[f64],
            sea: f64,
        ) -> StepOut {
            self.step_spreading(p, sv, st, eroded, sea, None)
        }

        /// One step with the plateau spreading after the erosion, `spreading` its strength
        /// height (m) and D₀ (km²/Myr), on the cells of `d` (`spread::step`).
        fn step_spreading(
            &mut self,
            p: &FlexureParams,
            sv: &mut Option<Solver>,
            st: &mut Option<FlexState>,
            eroded: &[f64],
            sea: f64,
            spreading: Option<(f64, f64)>,
        ) -> StepOut {
            let dt_myr = 2.0;
            let h_start: Vec<f64> = self
                .h
                .iter()
                .zip(&self.uplift)
                .map(|(&z, &u)| z + f64::from(u) * dt_myr * 1.0e3)
                .collect();
            let h_eroded: Vec<f64> = h_start.iter().zip(eroded).map(|(a, b)| a + b).collect();
            // The uplift is crust thickened at its Airy height: its crust's dry Airy change.
            let crust_uplift: Vec<f32> = self
                .h
                .iter()
                .zip(&self.uplift)
                .map(|(&z, &u)| crust_equivalent(u, z, sea, dt_myr) * self.crust_share as f32)
                .collect();
            let mut h = h_eroded.clone();
            if let Some((hs, d0)) = spreading {
                super::super::spread::step(&self.d, &mut h, sea, hs, d0, dt_myr * 1.0e6);
            }
            let out = step(
                p,
                sv,
                st,
                &StepIn {
                    dims: &self.d,
                    norm_dx_m: self.norm_dx,
                    sin_faces: self.sin_faces.as_deref(),
                    plate_spacing_m: self.plate_spacing,
                    gravity_m_s2: G,
                    sea_level_m: sea,
                    water_z: self.water_z.as_deref(),
                    dt_myr,
                    h_start: &h_start,
                    h_eroded: &h_eroded,
                    h_before_spread: &h_eroded,
                    uplift_mm_yr: &self.uplift,
                    crust_uplift_mm_yr: &crust_uplift,
                    crust_type: &self.crust,
                    crust_age_myr: &self.age,
                    orogeny_age_myr: &self.orogeny,
                    regime: &self.regime,
                    plate_id: &self.plate,
                    ice_m: Some(&self.ice),
                    rock_share: 0.8,
                    parts: true,
                },
                &mut h,
            )
            .unwrap();
            self.h = h;
            out
        }
    }

    /// The dry Airy change of the crust behind an elevation change of `u_mm_yr` over `dt_myr` of
    /// ground at `z0` (the plate model's `crust_uplift_mm_yr` for an uplift that is all crust).
    fn crust_equivalent(u_mm_yr: f32, z0: f64, sea: f64, dt_myr: f64) -> f32 {
        let (ka, _) = airy_shares();
        let (q, _) = thickening(f64::from(u_mm_yr) * dt_myr * 1.0e3, z0, sea);
        (q / ka * (1.0 - ka) / (dt_myr * 1.0e3)) as f32
    }

    fn on() -> FlexureParams {
        FlexureParams {
            strength: 1.0,
            tolerance_m: 0.01,
            max_iterations: 200,
            ..FlexureParams::default()
        }
    }

    /// The first step only takes the reference; the stage's own first step has no uplift in it.
    fn started(wd: &mut World, p: &FlexureParams, sea: f64) -> (Option<Solver>, Option<FlexState>) {
        let (mut sv, mut st) = (None, None);
        let n = wd.h.len();
        let before = wd.h.clone();
        let out = wd.step(p, &mut sv, &mut st, &vec![0.0; n], sea);
        assert_eq!(wd.h, before, "an unloaded plate does not move");
        assert_eq!(out.max_change_m, 0.0);
        (sv, st)
    }

    /// On a plate with no strength the stage is local isostasy: erosion rebounds by `isostasy`
    /// (0.7 of it under water), the plate model's uplift is untouched, and ice sinks the ground
    /// by ρ_i/ρ_m of its thickness and lets it back up when it melts.
    #[test]
    fn a_plate_without_strength_is_local_isostasy() {
        let p = FlexureParams {
            te_craton_km: 0.001,
            te_min_km: 0.001,
            te_ocean_max_km: 0.001,
            ..on()
        };
        let (w, h) = (64usize, 32usize);
        let n = w * h;
        let mut wd = World::new(w, h, 1000.0);
        for c in 0..n {
            if c % w < 16 {
                wd.h[c] = -2000.0;
            }
        }
        let (mut sv, mut st) = started(&mut wd, &p, 0.0);
        let before = wd.h.clone();
        let land = 10 * w + 40;
        let sea_cell = 10 * w + 5;
        let lifted = 20 * w + 50;
        let mut eroded = vec![0.0; n];
        eroded[land] = -900.0;
        eroded[sea_cell] = 500.0;
        wd.uplift[lifted] = 0.25; // 500 m over the step
        wd.step(&p, &mut sv, &mut st, &eroded, 0.0);
        let change = |c: usize| wd.h[c] - before[c];
        assert!(
            (change(land) - (-900.0 + 720.0)).abs() < 0.5,
            "{}",
            change(land)
        );
        assert!(
            (change(sea_cell) - (500.0 - 0.7 * 0.8 * 500.0)).abs() < 0.5,
            "{}",
            change(sea_cell)
        );
        assert!((change(lifted) - 500.0).abs() < 0.5, "{}", change(lifted));
        assert!(change(0).abs() < 1e-6 && change(n - 1).abs() < 1e-6);
        // Ice: 3 km of it over a wide sheet.
        wd.uplift[lifted] = 0.0;
        let bare = wd.h.clone();
        for c in 0..n {
            if c % w >= 32 {
                wd.ice[c] = 3000.0;
            }
        }
        wd.step(&p, &mut sv, &mut st, &vec![0.0; n], 0.0);
        let under = 16 * w + 48;
        assert!(
            (bare[under] - wd.h[under] - 3000.0 * RHO_ICE / RHO_MANTLE).abs() < 1.0,
            "{}",
            bare[under] - wd.h[under]
        );
        wd.ice = vec![0.0; n];
        wd.step(&p, &mut sv, &mut st, &vec![0.0; n], 0.0);
        assert!((wd.h[under] - bare[under]).abs() < 0.5, "rebound");
    }

    /// The plate model's moves of ocean floor that builds nothing are heights under the sea, the
    /// water's weight in them (ISO-096): GDH1's cooling and a trench's pull (its uplift), and a
    /// gap's floor that a plate 6.5 km deeper covers (the carry's move, [`FlexState::from_m`]),
    /// sink the floor by themselves alone. Read as sea let in, the cover sank the floor
    /// ρ_w/(ρ_m − ρ_w) of the move more. The sea rising over the floor is a load.
    #[test]
    fn the_sea_over_the_ocean_floors_own_moves_is_no_load() {
        let p = FlexureParams {
            te_craton_km: 0.001,
            te_min_km: 0.001,
            te_ocean_max_km: 0.001,
            ..on()
        };
        let (w, h) = (32usize, 16usize);
        let n = w * h;
        let probe = 8 * w + 16;
        let ocean = |wd: &mut World, reg: u8| {
            wd.crust = vec![f32::from(OCEANIC); n];
            wd.age = vec![60.0; n];
            wd.regime = vec![f32::from(reg); n];
        };
        let sink =
            crate::plates::pools::gdh1_depth_m(60.0) - crate::plates::pools::gdh1_depth_m(58.0);
        for (reg, fall) in [(regime::OCEAN_BASIN, sink), (regime::TRENCH, 2400.0)] {
            let mut wd = World::new(w, h, -5000.0);
            ocean(&mut wd, reg);
            let (mut sv, mut st) = started(&mut wd, &p, 0.0);
            let before = wd.h[probe];
            wd.uplift = vec![(-fall / 2.0e3) as f32; n];
            wd.step(&p, &mut sv, &mut st, &vec![0.0; n], 0.0);
            let sank = before - wd.h[probe];
            assert!((sank - fall).abs() < 0.05, "regime {reg}: {sank} vs {fall}");
        }
        let cover = |carried: bool| -> f64 {
            let mut wd = World::new(w, h, -3500.0);
            ocean(&mut wd, regime::BACK_ARC);
            let (mut sv, mut st) = started(&mut wd, &p, 0.0);
            if carried {
                st.as_mut().unwrap().from_m = wd.h.iter().map(|&z| z as f32).collect();
            }
            wd.h = vec![-10_000.0; n];
            wd.step(&p, &mut sv, &mut st, &vec![0.0; n], 0.0);
            -10_000.0 - wd.h[probe]
        };
        let moved = cover(true);
        assert!(
            moved.abs() < 0.05,
            "the cover sank the floor {moved} m more"
        );
        let read = cover(false);
        let loaded = 6500.0 * RHO_WATER / (RHO_MANTLE - RHO_WATER);
        assert!((read - loaded).abs() < 1.0, "{read} vs {loaded}");
        let mut wd = World::new(w, h, -5000.0);
        ocean(&mut wd, regime::OCEAN_BASIN);
        let (mut sv, mut st) = started(&mut wd, &p, 0.0);
        let before = wd.h[probe];
        wd.step(&p, &mut sv, &mut st, &vec![0.0; n], 100.0);
        let sank = before - wd.h[probe];
        let want = 100.0 * RHO_WATER / (RHO_MANTLE - RHO_WATER);
        assert!(
            (sank - want).abs() < 0.05,
            "the sea's rise: {sank} vs {want}"
        );
    }

    /// A collision as the plate model builds it (`plates/belts.rs`, tasks 3 and 4): a belt on the
    /// overriding plate and a wedge on the incoming one, both young orogens (weak), with the plate
    /// boundary between them a break. The incoming plate is a stiff craton beyond the wedge: it
    /// bends down in front of the wedge into a foreland basin deeper than its surroundings, with a
    /// bulge beyond, and nothing of the load reaches across the boundary.
    #[test]
    fn a_collision_wedge_bends_a_foreland_basin_on_the_incoming_plate() {
        let p = on();
        let (w, h) = (32usize, 256usize);
        let n = w * h;
        let mut wd = World::new(w, h, 300.0);
        let sea = -1.0e6;
        // Rows below 128: the overriding plate; from row 128 on: the incoming plate.
        for c in 0..n {
            if c / w >= 128 {
                wd.plate[c] = 1.0;
            }
        }
        let (mut sv, mut st) = started(&mut wd, &p, sea);
        // The belt: 10 rows (390 km) behind the front with the profile 1 − (d/W)², 2 km at the
        // front in the step; the wedge: 5 rows (195 km) in front, rising straight to the front.
        for c in 0..n {
            let j = c / w;
            let u = if (118..128).contains(&j) {
                let x = (127 - j) as f64 / 10.0;
                1.0 - x * x
            } else if (128..133).contains(&j) {
                1.0 - (j - 128) as f64 / 5.0
            } else {
                0.0
            };
            if u > 0.0 {
                wd.uplift[c] = u as f32;
                wd.orogeny[c] = 0.0;
                wd.regime[c] = f32::from(regime::COLLISION);
            }
        }
        let out = wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea);
        assert!(out.solve.as_ref().unwrap().converged, "{:?}", out.solve);
        let flex = |j: usize| f64::from(out.flexure_m[j * w + 3]);
        // In front of the wedge's toe (row 133 on): the foreland basin.
        let basin = (133..160).map(flex).fold(f64::MAX, f64::min);
        assert!(basin < -100.0, "no foreland basin: {basin}");
        let far = (210..250).map(|j| flex(j).abs()).fold(0.0f64, f64::max);
        assert!(
            basin < -far - 100.0,
            "the basin {basin} against {far} far away"
        );
        let bulge = (133..220).map(flex).fold(f64::MIN, f64::max);
        assert!(bulge > 0.0, "no bulge beyond: {bulge}");
        // The overriding plate behind its belt does not feel the incoming plate's bending.
        let behind = (60..110).map(|j| flex(j).abs()).fold(0.0f64, f64::max);
        assert!(behind < 0.5 * basin.abs(), "{behind} behind the belt");
    }

    /// Under the sea the roots' part of the split is their bending only, as on land: crust the
    /// plate model raised everywhere alike (nothing to bend) leaves the roots' and the loads'
    /// answers nil, the sea water it pushed aside going with the roots.
    #[test]
    fn under_the_sea_a_root_with_nothing_to_bend_answers_nothing() {
        let p = on();
        let (w, h) = (32usize, 256usize);
        let n = w * h;
        let mut wd = World::new(w, h, 300.0);
        let sea = 5000.0;
        let (mut sv, mut st) = started(&mut wd, &p, sea);
        wd.uplift = vec![1.0; n]; // 2 km of Airy elevation in the step, all of it under the sea
        let before = wd.h.clone();
        let out = wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea);
        for j in [5usize, 128, 250] {
            let c = j * w + 3;
            let change = wd.h[c] - before[c] - 2000.0;
            assert!(
                change.abs() < 1.0,
                "row {j}: the flexure moved it by {change}"
            );
            assert!(
                out.roots_m[c].abs() < 1.0,
                "row {j}: roots {}",
                out.roots_m[c]
            );
            assert!(
                out.loads_m[c].abs() < 1.0,
                "row {j}: loads {}",
                out.loads_m[c]
            );
        }
    }

    /// A height the plate model moved alone (its settling, a height rule; ISO-091) is no load, on
    /// land and under the sea: a stiff plate holds nothing up for it, the ground moves by the
    /// uplift and no more, and the load the plate carries does not change. Booked as crust (until
    /// 2026-10-22) the same rise stood hundreds of metres above it on the stiff plate.
    #[test]
    fn a_height_moved_alone_is_no_load() {
        let p = on();
        let (w, h) = (32usize, 256usize);
        let n = w * h;
        for (z, sea) in [(300.0, -1.0e6), (-4000.0, 0.0)] {
            let mut wd = World::new(w, h, z);
            let (mut sv, mut st) = started(&mut wd, &p, sea);
            let load0: f64 = st
                .as_ref()
                .unwrap()
                .load_m
                .iter()
                .map(|&q| f64::from(q))
                .sum();
            wd.crust_share = 0.0;
            for c in 0..n {
                if (124..132).contains(&(c / w)) {
                    wd.uplift[c] = 1.0; // 2 km in the step
                }
            }
            let before = wd.h.clone();
            let out = wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea);
            for j in [127usize, 135, 160, 250] {
                let c = j * w + 3;
                let change = wd.h[c] - before[c] - f64::from(wd.uplift[c]) * 2000.0;
                assert!(change.abs() < 1.0, "at {z}, row {j}: moved {change}");
                assert!(
                    out.flexure_m[c].abs() < 1.0,
                    "at {z}, row {j}: {}",
                    out.flexure_m[c]
                );
            }
            let load: f64 = st
                .as_ref()
                .unwrap()
                .load_m
                .iter()
                .map(|&q| f64::from(q))
                .sum();
            assert!((load - load0).abs() < 1.0, "at {z}: load {load0} to {load}");
        }
    }

    /// Crust the plate model thinned (a rift, the collapse of a plateau) is a load taken away: a
    /// stiff plate holds the thinned belt below its Airy height and lifts its flanks, the mirror
    /// of a belt raised.
    #[test]
    fn a_belt_thinned_on_a_stiff_plate_stands_below_its_airy_height() {
        let p = on();
        let (w, h) = (32usize, 256usize);
        let n = w * h;
        let mut wd = World::new(w, h, 3000.0);
        let sea = -1.0e6;
        let (mut sv, mut st) = started(&mut wd, &p, sea);
        for c in 0..n {
            if (124..132).contains(&(c / w)) {
                wd.uplift[c] = -1.0; // 2 km of Airy fall in the step
            }
        }
        let out = wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea);
        assert!(out.solve.as_ref().unwrap().converged, "{:?}", out.solve);
        assert!(
            out.flexure_m[127 * w + 3] < -500.0,
            "{}",
            out.flexure_m[127 * w + 3]
        );
        assert!(
            out.flexure_m[135 * w + 3] > 100.0,
            "{}",
            out.flexure_m[135 * w + 3]
        );
        assert!(
            wd.h[127 * w + 3] < 3000.0 - 2000.0 - 500.0,
            "{}",
            wd.h[127 * w + 3]
        );
    }

    /// A belt raised by the plate model on a craton: the plate holds part of its root, so the
    /// belt stands higher than its Airy height, a basin lies in front of it and a bulge beyond;
    /// the buoyancy of all the sinking is the belt's weight. When the belt is eroded away the
    /// basin comes back.
    #[test]
    fn a_belt_on_a_stiff_plate_gets_a_foreland_basin() {
        let p = on();
        let (w, h) = (32usize, 256usize);
        let n = w * h;
        let mut wd = World::new(w, h, 300.0);
        // No sea in this world: every load is rock on land.
        let sea = -1.0e6;
        let (mut sv, mut st) = started(&mut wd, &p, sea);
        let belt = |c: usize| (124..132).contains(&(c / w));
        for c in 0..n {
            if belt(c) {
                wd.uplift[c] = 1.0; // 2 km of Airy elevation in the step
            }
        }
        let before = wd.h.clone();
        let out = wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea);
        let rep = out.solve.unwrap();
        assert!(rep.converged, "{rep:?}");
        // The split the gap budget reads: the flexure's change is all the roots' answer here (the
        // belt held up, the basin pulled down), none of it the loads' or a weakened plate's.
        for j in [127usize, 135, 160, 250] {
            let c = j * w + 3;
            let change = wd.h[c] - before[c] - f64::from(wd.uplift[c]) * 2000.0;
            assert!(
                (f64::from(out.roots_m[c]) - change).abs() < 1.0,
                "row {j}: roots {} against {change}",
                out.roots_m[c]
            );
            assert!(out.loads_m[c].abs() < 1.0, "row {j}: {}", out.loads_m[c]);
            // The plate kept its stiffness: it let nothing go.
            assert!(
                out.released_m[c].abs() < 1.0,
                "row {j}: {}",
                out.released_m[c]
            );
        }
        let col = |j: usize| wd.h[j * w + 3];
        assert!(
            col(127) > 2300.0 + 500.0,
            "the belt stands above Airy: {}",
            col(127)
        );
        let basin = (132..150).map(col).fold(f64::MAX, f64::min);
        assert!(basin < 300.0 - 300.0, "a basin in front: {basin}");
        let bulge = (132..200).map(col).fold(f64::MIN, f64::max);
        assert!(bulge > 300.0 + 5.0, "a bulge beyond: {bulge}");
        assert!((col(250) - 300.0).abs() < 1.0, "far away nothing moved");
        // The basin is `surface/flexure_m` below zero, the belt above.
        assert!(out.flexure_m[135 * w + 3] < -100.0 && out.flexure_m[127 * w + 3] > 500.0);
        // Compensation: Σ sinking = Σ load (m of mantle), all on land.
        let load: f64 = st
            .as_ref()
            .unwrap()
            .load_m
            .iter()
            .map(|&q| f64::from(q))
            .sum();
        let sunk: f64 = out.deflection_m.iter().map(|&x| f64::from(x)).sum();
        assert!((sunk / load - 1.0).abs() < 1e-3, "{sunk} against {load}");
        // Erosion takes the belt away again (the crust thickened, 7.0 × the uplift): no load is
        // left and the basin is gone.
        wd.uplift = vec![0.0; n];
        let mut eroded = vec![0.0; n];
        for (c, dm) in eroded.iter_mut().enumerate() {
            if belt(c) {
                *dm = -2000.0 * (3300.0 / 470.0) * (2830.0 / 3300.0) / 0.8;
            }
        }
        let before = wd.h.clone();
        let out = wd.step(&p, &mut sv, &mut st, &eroded, sea);
        // … and the erosion's answer is all the loads'.
        for j in [127usize, 137, 160] {
            let c = j * w + 3;
            let change = wd.h[c] - before[c] - eroded[c];
            assert!(
                (f64::from(out.loads_m[c]) - change).abs() < 1.0,
                "row {j}: loads {} against {change}",
                out.loads_m[c]
            );
            assert!(out.roots_m[c].abs() < 1.0, "row {j}: {}", out.roots_m[c]);
            assert!(
                out.released_m[c].abs() < 1.0,
                "row {j}: {}",
                out.released_m[c]
            );
        }
        let left = st
            .as_ref()
            .unwrap()
            .load_m
            .iter()
            .fold(0.0f32, |a, &q| a.max(q.abs()));
        assert!(left < 1.0, "load left: {left}");
        let back = wd.h[137 * w + 3];
        assert!((back - 300.0).abs() < 2.0, "the basin rebounded: {back}");
    }

    /// A plate boundary is a break: the same belt beside a boundary pulls its own plate down and
    /// leaves the other plate where it was.
    #[test]
    fn a_plate_boundary_is_a_break() {
        let p = on();
        let (w, h) = (32usize, 128usize);
        let n = w * h;
        let sea = -1.0e6;
        let basin = |two_plates: bool| {
            let mut wd = World::new(w, h, 300.0);
            if two_plates {
                for c in 0..n {
                    if c / w >= 66 {
                        wd.plate[c] = 1.0;
                    }
                }
            }
            let (mut sv, mut st) = started(&mut wd, &p, sea);
            for c in 0..n {
                if (56..64).contains(&(c / w)) {
                    wd.uplift[c] = 1.0;
                }
            }
            wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea);
            // How far the ground sank five cells in front of the belt, on either side.
            (300.0 - wd.h[51 * w], 300.0 - wd.h[68 * w])
        };
        let (own, other) = basin(false);
        assert!(
            own > 100.0 && (own - other).abs() < 1.0,
            "one plate: {own}, {other}"
        );
        let (own2, other2) = basin(true);
        assert!(own2 > 100.0, "its own plate still bends: {own2}");
        assert!(
            other2.abs() < 0.2 * other,
            "the other plate is left alone: {other2} (one plate: {other})"
        );
    }

    /// A belt built while its crust was weak has a local root. When the crust stiffens nothing
    /// moves; a load added afterwards meets the stiff plate.
    #[test]
    fn a_plate_that_stiffens_keeps_its_shape() {
        let p = on();
        let (w, h) = (32usize, 128usize);
        let n = w * h;
        let mut wd = World::new(w, h, 300.0);
        wd.orogeny = vec![0.0; n]; // a young orogen everywhere: Te at its floor
        let (mut sv, mut st) = started(&mut wd, &p, 0.0);
        let belt = |c: usize| (60..64).contains(&(c / w));
        for c in 0..n {
            if belt(c) {
                wd.uplift[c] = 1.0;
            }
        }
        wd.step(&p, &mut sv, &mut st, &vec![0.0; n], 0.0);
        wd.uplift = vec![0.0; n];
        let weak = wd.h.clone();
        assert!(
            weak[62 * w] < 2300.0 + 400.0,
            "nearly Airy on the weak plate: {}",
            weak[62 * w]
        );
        // A long time later the crust is a craton.
        wd.orogeny = vec![f32::NAN; n];
        let out = wd.step(&p, &mut sv, &mut st, &vec![0.0; n], 0.0);
        assert!(
            out.max_change_m < 1.0,
            "stiffening moved the ground by {}",
            out.max_change_m
        );
        assert!(out.elastic_thickness_km[0] > 79.0);
        // The same load again, now on the stiff plate: it is held up.
        let before = wd.h.clone();
        for c in 0..n {
            if belt(c) {
                wd.uplift[c] = 1.0;
            }
        }
        wd.step(&p, &mut sv, &mut st, &vec![0.0; n], 0.0);
        let rise = wd.h[62 * w] - before[62 * w];
        assert!(rise > 2000.0 + 3000.0, "held up by the craton: {rise}");
    }

    /// A belt held up by a craton, then an orogeny reaches the craton (the law's Te falls to its
    /// floor and stays there). Weakening at once, the belt sinks most of the way to local
    /// isostasy in the step; weakening over the thermal time (ISO-099), a step lets go of a few
    /// per cent of that, and after eight time constants the plate stands where weakening at once
    /// put it. The plate stands on dry ground, the sea far below: with the sea at 0 the loading
    /// took ground under it (the lowest cell 4.3 km down without water), the flexure's water came
    /// in, whose depth has a kink at the sea, and the two paths ended 172 m apart.
    #[test]
    fn a_plate_the_law_weakens_lets_go_as_it_warms() {
        let (w, h) = (32usize, 128usize);
        let n = w * h;
        let sea = -20_000.0;
        let belt = |c: usize| (60..64).contains(&(c / w));
        let run = |p: &FlexureParams, steps: usize| -> (Vec<f64>, f64, Vec<f64>) {
            let mut wd = World::new(w, h, 300.0);
            let (mut sv, mut st) = started(&mut wd, p, sea);
            for c in 0..n {
                if belt(c) {
                    wd.uplift[c] = 1.0;
                }
            }
            wd.step(p, &mut sv, &mut st, &vec![0.0; n], sea);
            wd.uplift = vec![0.0; n];
            wd.orogeny = vec![0.0; n]; // the orogeny's clock restarts every step
            let first = wd.step(p, &mut sv, &mut st, &vec![0.0; n], sea);
            let te = f64::from(first.elastic_thickness_km[62 * w]);
            for _ in 1..steps {
                wd.step(p, &mut sv, &mut st, &vec![0.0; n], sea);
            }
            let released = first.released_m.iter().map(|&r| f64::from(r)).collect();
            (wd.h, te, released)
        };
        let at_once = FlexureParams {
            te_weakening_myr: 0.0,
            ..on()
        };
        let p = FlexureParams {
            te_weakening_myr: 62.8, // ledger: ISO-099
            ..on()
        };
        let steps = (8.0 * p.te_weakening_myr / 2.0).ceil() as usize;
        let (h_now, te_now, rel_now) = run(&at_once, steps);
        let (h_slow, te_slow, rel_slow) = run(&p, steps);
        let c = 62 * w;
        assert!(te_now < 5.5, "at once: Te {te_now}");
        let expect = 5.0 + 75.0 * (-2.0 / p.te_weakening_myr).exp();
        assert!(
            (te_slow - expect).abs() < 0.5,
            "one step of warming: Te {te_slow}, {expect} expected"
        );
        assert!(
            rel_now[c] < -500.0,
            "the belt sinks in the step: {}",
            rel_now[c]
        );
        assert!(
            rel_slow[c] < 0.0 && rel_slow[c] > 0.15 * rel_now[c],
            "one step lets go of a few per cent: {} against {}",
            rel_slow[c],
            rel_now[c]
        );
        let worst = h_now
            .iter()
            .zip(&h_slow)
            .fold(0.0f64, |a, (x, y)| a.max((x - y).abs()));
        assert!(
            worst < 0.02 * (h_now[c] - 300.0).abs().max(100.0),
            "the same plate in the end: {worst} m apart"
        );
    }

    /// The solve grid: one L1 cell on the quick presets, two on the full ones, and the same
    /// answer through a coarsened solve.
    #[test]
    fn fine_rasters_solve_on_a_coarser_grid() {
        assert_eq!(coarsening(1024, 512, 39_090.0), 1);
        assert_eq!(coarsening(2048, 1024, 19_545.0), 2);
        assert_eq!(coarsening(4096, 2048, 9_772.0), 4);
        assert_eq!(coarsening(256, 128, 156_000.0), 1);
        let v: Vec<f64> = (0..64).map(|c| c as f64).collect();
        let r = restrict(&v, 8, 2);
        assert_eq!(r.len(), 16);
        assert_eq!(r[0], (0.0 + 1.0 + 8.0 + 9.0) / 4.0);
        assert_eq!(restrict(&prolong_const(&r, 4, 2), 8, 2), r);
        let lin = prolong_linear(&r, 4, 2);
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        assert!((mean(&lin) - mean(&r)).abs() < 0.5);
        // A wide ice sheet through the coarsened solve sinks the ground as it does on L1.
        let p = FlexureParams {
            te_craton_km: 20.0,
            ..on()
        };
        let sink = |norm_dx: f64, w: usize, h: usize| {
            let mut wd = World::new(w, h, 500.0);
            wd.norm_dx = norm_dx;
            let (mut sv, mut st) = started(&mut wd, &p, 0.0);
            for c in 0..w * h {
                if (c / w) >= h / 4 && (c / w) < 3 * h / 4 {
                    wd.ice[c] = 2000.0;
                }
            }
            wd.step(&p, &mut sv, &mut st, &vec![0.0; w * h], 0.0);
            assert_eq!(sv.as_ref().unwrap().w, 32);
            500.0 - wd.h[(h / 2) * w]
        };
        let (a, b) = (sink(39_000.0, 32, 64), sink(19_500.0, 64, 128));
        assert!((a - 2000.0 * RHO_ICE / RHO_MANTLE).abs() < 5.0, "{a}");
        assert!((a - b).abs() < 5.0, "{a} and {b}");
    }

    /// The synthetic margin (the controlled case for the cell-scale alternation found along
    /// active margins in planet runs, build log 2026-10-02): a strip of three plates. Plate 2
    /// (columns 0–7) is fixed; plate 0 is old ocean floor moving east by 0.37 cells a step (its
    /// state is interpolated every step, new crust is born at the ridge at column 8); plate 1 is
    /// a continental craton whose edge advances west over plate 0 by 0.23 cells a step along a
    /// wavy boundary (two cells either way over 16 rows: smooth), so cells change plate, reset and
    /// change stiffness as at a real trench, and the break steps from row to row. A
    /// smooth sediment fan (50 m a step) rides on plate 0 and a smooth arc (0.02 mm/yr) rises
    /// behind the edge of plate 1. The deflection and the flexure field must not zigzag at the
    /// cell scale away from the plate boundaries.
    /// The margin after `steps` steps: its width and height, the deflection, the flexure field,
    /// the plates and the load (m of mantle).
    #[allow(clippy::type_complexity)]
    fn moving_margin(steps: usize) -> (usize, usize, Vec<f64>, Vec<f64>, Vec<f32>, Vec<f64>) {
        let p = on();
        let (w, h) = (96usize, 16usize);
        let n = w * h;
        let sea = 0.0;
        let dt_myr = 2.0;
        let (v0, v1) = (0.37, 0.23);
        let col = |c: usize| c % w;
        let edge_at = |t: usize, j: usize| {
            70.0 - v1 * t as f64 + 2.0 * (std::f64::consts::TAU * j as f64 / 16.0).sin()
        };
        let plate_at = |t: usize| -> Vec<f32> {
            (0..n)
                .map(|c| {
                    let i = col(c) as f64;
                    if i < 8.0 {
                        2.0
                    } else if i < edge_at(t, c / w) {
                        0.0
                    } else {
                        1.0
                    }
                })
                .collect()
        };
        let d = Dims::new(w, h, 4800.0, true);
        let parents = [None, None, None];
        let mut plate_prev = plate_at(0);
        let base = |c: usize, pl: f32| -> f32 {
            match pl as i32 {
                2 => -2500.0,
                0 => -2500.0 - 2500.0 * ((col(c) as f32 - 8.0).max(0.0) / 60.0).sqrt(),
                _ => 500.0,
            }
        };
        let mut elev: Vec<f32> = (0..n).map(|c| base(c, plate_prev[c])).collect();
        let (mut sv, mut st): (Option<Solver>, Option<FlexState>) = (None, None);
        let mut last = StepOut::default();
        for t in 1..=steps {
            let plate = plate_at(t);
            let crust: Vec<f32> = plate
                .iter()
                .map(|&q| if q == 1.0 { 1.0 } else { 0.0 })
                .collect();
            let edge = |c: usize| edge_at(t, c / w);
            let age: Vec<f32> = (0..n)
                .map(|c| match plate[c] as i32 {
                    2 => 20.0,
                    0 => ((col(c) as f64 - 8.0) / v0 * dt_myr) as f32,
                    _ => -1.0,
                })
                .collect();
            let reg: Vec<f32> = (0..n)
                .map(|c| {
                    let i = col(c) as f64;
                    f32::from(match plate[c] as i32 {
                        2 => regime::OCEAN_BASIN,
                        0 if i < 10.0 => regime::RIDGE,
                        0 if i > edge(c) - 2.0 => regime::TRENCH,
                        0 => regime::OCEAN_BASIN,
                        _ if i < edge(c) + 3.0 => regime::ARC,
                        _ => regime::CRATON,
                    })
                })
                .collect();
            let uplift: Vec<f32> = (0..n)
                .map(|c| {
                    let x = (col(c) as f64 - edge(c) - 4.0) as f32;
                    if plate[c] == 1.0 {
                        0.02 * (-(x / 3.0) * (x / 3.0)).exp()
                    } else {
                        0.0
                    }
                })
                .collect();
            let plate_elev: Vec<f32> = (0..n).map(|c| base(c, plate[c])).collect();
            let src: Vec<[f64; 2]> = (0..n)
                .map(|c| {
                    let (i, j) = (col(c) as f64, (c / w) as f64);
                    match plate[c] as i32 {
                        0 => [i - v0, j],
                        1 => [i + v1, j],
                        _ => [i, j],
                    }
                })
                .collect();
            let orogeny = vec![f32::NAN; n];
            let carry = Carry {
                w,
                h,
                src: &src,
                prev_elevation: &elev,
                prev_plate: Some(&plate_prev),
                prev_group: None,
                plate: &plate,
                group: None,
                plate_elevation: &plate_elev,
                uplift_mm_yr: &uplift,
                gap: None,
                prev_gap: None,
                prev_offset: None,
                prev_held_offset: None,
                flags: None,
                regime: None,
                parents: &parents,
                known_plates: 3,
                dt_yr: dt_myr * 1.0e6,
                blur_ref_yr: 2.0e6,
            };
            if let Some(s) = st.as_mut() {
                s.carry(&carry, &elev);
            }
            let (carried, _) = advect::carried_elevation(&carry);
            let h_start: Vec<f64> = carried.iter().map(|&z| f64::from(z)).collect();
            let h_eroded: Vec<f64> = (0..n)
                .map(|c| {
                    let x = col(c) as f64 - 45.0;
                    let fan = if plate[c] == 0.0 {
                        50.0 * (-(x / 5.0) * (x / 5.0)).exp()
                    } else {
                        0.0
                    };
                    h_start[c] + fan
                })
                .collect();
            let mut hh = h_eroded.clone();
            last = step(
                &p,
                &mut sv,
                &mut st,
                &StepIn {
                    dims: &d,
                    norm_dx_m: 39_000.0,
                    sin_faces: None,
                    plate_spacing_m: 71_000.0,
                    gravity_m_s2: G,
                    sea_level_m: sea,
                    water_z: None,
                    dt_myr,
                    h_start: &h_start,
                    h_eroded: &h_eroded,
                    h_before_spread: &h_eroded,
                    uplift_mm_yr: &uplift,
                    crust_uplift_mm_yr: &uplift,
                    crust_type: &crust,
                    crust_age_myr: &age,
                    orogeny_age_myr: &orogeny,
                    regime: &reg,
                    plate_id: &plate,
                    ice_m: None,
                    rock_share: 0.8,
                    parts: false,
                },
                &mut hh,
            )
            .unwrap();
            elev = hh.iter().map(|&z| z as f32).collect();
            plate_prev = plate;
        }
        let defl = last.deflection_m.iter().map(|&x| f64::from(x)).collect();
        let flex = last.flexure_m.iter().map(|&x| f64::from(x)).collect();
        let load = st
            .as_ref()
            .map(|s| s.load_m.iter().map(|&q| f64::from(q)).collect())
            .unwrap_or_default();
        (w, h, defl, flex, plate_prev, load)
    }

    #[test]
    fn a_moving_margin_bends_smoothly() {
        let (w, h, defl, flex, plate, load) = moving_margin(40);
        // Away from the plate boundaries (no other plate within two cells), where the plate is
        // broken on purpose.
        let away = |c: usize| {
            let (i, j) = (c % w, c / w);
            (-2i64..=2).all(|dj| {
                (-2i64..=2).all(|di| {
                    let ii = (i as i64 + di).rem_euclid(w as i64) as usize;
                    let jj = (j as i64 + dj).clamp(0, h as i64 - 1) as usize;
                    plate[jj * w + ii] == plate[c]
                })
            })
        };
        // The cells that zigzag, with their column (along the margin) and its plates.
        let spots = |v: &[f64], t: f64| {
            let mut out = Vec::new();
            for c in 0..w * h {
                if away(c) && cell_scale_zigzag(v, w, h, t, |k| k == c) > 0.0 {
                    let (i, j) = (c % w, c / w);
                    let col: Vec<String> =
                        (0..h).map(|jj| format!("{:.0}", v[jj * w + i])).collect();
                    let pl: String = (0..h)
                        .map(|jj| format!("{:.0}", plate[jj * w + i]))
                        .collect();
                    out.push(format!("({i},{j}) [{}] plates {pl}", col.join(" ")));
                }
            }
            out.join("\n")
        };
        // Half a block (10 m): the rows under the arc took on arc crust at different steps and
        // differ by a few metres in their load history. Where the load itself zigzags by more
        // (17 m at the trough of the margin's wave, where the arc's edge steps between ARC and
        // CRATON from row to row), a plate that weakened there settled towards local isostasy
        // and its deflection follows the load: that is the load's, and the flexure field below
        // shows the solve made none of it.
        let load_zig = |c: usize| cell_scale_zigzag(&load, w, h, 10.0, |k| k == c) > 0.0;
        let zig_d = cell_scale_zigzag(&defl, w, h, 10.0, |c| away(c) && !load_zig(c));
        // The flexure field (load less sinking) also carries each cell's load history: cells the
        // craton took over at different steps hold different loads, a few metres apart.
        let zig_f = cell_scale_zigzag(&flex, w, h, 10.0, away);
        assert!(
            zig_d == 0.0 && zig_f == 0.0,
            "cell-scale zigzag: deflection {zig_d:.4}, flexure {zig_f:.4}\ndeflection\n{}\nflexure\n{}",
            spots(&defl, 10.0),
            spots(&flex, 10.0)
        );
    }

    /// The polar stripes (build log 2026-10-02): near the pole a plate's rotation shifts each row
    /// of the block-metric raster by a different fraction of a cell per step, so the carried
    /// state is interpolated anew every step. A bent plate that is only carried must not move,
    /// and a ring load (a full row band) bends the plate the same at every longitude: the operator
    /// and the preconditioner have no term that depends on latitude.
    #[test]
    fn carrying_a_bent_plate_moves_nothing() {
        let p = on();
        let (w, h) = (64usize, 32usize);
        let n = w * h;
        let sea = -1.0e6;
        let mut wd = World::new(w, h, 300.0);
        let (mut sv, mut st) = started(&mut wd, &p, sea);
        // A ring near the "pole" (rows 3–5), raised in one step.
        for c in 0..n {
            if (3..6).contains(&(c / w)) {
                wd.uplift[c] = 0.5;
            }
        }
        wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea);
        let defl = st.as_ref().unwrap().deflection_m.clone();
        for j in 0..h {
            let row = &defl[j * w..(j + 1) * w];
            let (lo, hi) = row
                .iter()
                .fold((f32::MAX, f32::MIN), |m, &v| (m.0.min(v), m.1.max(v)));
            assert!(hi - lo < 1.0e-3 * hi.abs().max(1.0), "row {j}: {lo}..{hi}");
        }
        // Then a disc further south, so the plate is bent in both directions.
        wd.uplift = vec![0.0; n];
        for c in 0..n {
            let (i, j) = ((c % w) as f64, (c / w) as f64);
            if (i - 20.0).powi(2) + (j - 18.0).powi(2) < 9.0 {
                wd.uplift[c] = 0.5;
            }
        }
        wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea);
        wd.uplift = vec![0.0; n];
        // Carried for 20 steps, each row shifted by its own fraction of a cell (rotation about
        // the pole), with no new load: the ground stays where it is.
        let src: Vec<[f64; 2]> = (0..n)
            .map(|c| {
                let (i, j) = ((c % w) as f64, (c / w) as f64);
                [i - (0.13 + 0.71 / (j + 1.0)), j]
            })
            .collect();
        let ids = vec![0.0f32; n];
        let parents = [None];
        for _ in 0..20 {
            let elev: Vec<f32> = wd.h.iter().map(|&z| z as f32).collect();
            let carry = Carry {
                w,
                h,
                src: &src,
                prev_elevation: &elev,
                prev_plate: Some(&ids),
                prev_group: None,
                plate: &ids,
                group: None,
                plate_elevation: &elev,
                uplift_mm_yr: &wd.uplift,
                gap: None,
                prev_gap: None,
                prev_offset: None,
                prev_held_offset: None,
                flags: None,
                regime: None,
                parents: &parents,
                known_plates: 1,
                dt_yr: 2.0e6,
                blur_ref_yr: 2.0e6,
            };
            st.as_mut().unwrap().carry(&carry, &elev);
            let (carried, _) = advect::carried_elevation(&carry);
            wd.h = carried.iter().map(|&z| f64::from(z)).collect();
            let out = wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea);
            assert!(
                out.max_change_m < 0.01,
                "a carried plate moved by {} m",
                out.max_change_m
            );
        }
    }

    /// The plate model's uplift varies from cell to cell below its own sample spacing (the
    /// sample lattice, aliased onto L1). A stiff plate must not turn that into relief: with the
    /// plate model's spacing given, an uplift that alternates column by column raises the ground
    /// by about the uplift, as local isostasy does; without it, by the whole thickening behind it.
    #[test]
    fn the_plate_models_sample_lattice_is_not_a_load() {
        let p = on();
        let (w, h) = (32usize, 32usize);
        let n = w * h;
        let sea = -1.0e6;
        let zig = |spacing: f64| {
            let mut wd = World::new(w, h, 300.0);
            wd.plate_spacing = spacing;
            let (mut sv, mut st) = started(&mut wd, &p, sea);
            let before = wd.h.clone();
            for c in 0..n {
                if c % 2 == 0 {
                    wd.uplift[c] = 0.25; // 500 m of Airy uplift in the step, next to none
                }
            }
            wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea);
            let j = 16 * w;
            (wd.h[j + 10] - before[j + 10]) - (wd.h[j + 11] - before[j + 11])
        };
        let filtered = zig(71_000.0);
        assert!(
            (filtered - 500.0).abs() < 50.0,
            "column-to-column step {filtered} m, the uplift is 500 m"
        );
        let raw = zig(0.0);
        assert!(raw > 1500.0, "unfiltered the thickening shows: {raw} m");
    }

    /// The plateau spreading on a stiff plate, on cells as wide as the Earth-sized planet's
    /// (39 km), where its implicit step is strong at the cell scale: D·Δt·k² is about 2.3 for a
    /// checkerboard on a plateau three times the strength height. Its change is the locally
    /// compensated answer of the channel flow, and the plate leaves it so (ISO-070): the
    /// checkerboard flattens step by step and nothing leaves the plateau's range. When the plate
    /// held the spreading's change up as crust (up to 1 / (1 − k) = 7 times it at the cell
    /// scale) the checkerboard came back larger with its sign flipped, ±1.8 km after the first
    /// step: the ±50 km checkerboard of the build log 2026-10-18.
    #[test]
    fn the_spreading_on_a_stiff_plate_does_not_grow_a_checkerboard() {
        let p = on();
        let (w, h) = (32usize, 32usize);
        let n = w * h;
        let (sea, hs, top) = (0.0, 5100.0, 15_000.0);
        let mut wd = World::new(w, h, top);
        wd.d = Dims::new(w, h, 39_000.0, true);
        let (mut sv, mut st) = started(&mut wd, &p, sea);
        let patch = |c: usize| (8..24).contains(&(c % w)) && (8..24).contains(&(c / w));
        let sign = |c: usize| {
            if (c % w + c / w).is_multiple_of(2) {
                1.0
            } else {
                -1.0
            }
        };
        for c in (0..n).filter(|&c| patch(c)) {
            wd.h[c] += 300.0 * sign(c);
        }
        let amplitude = |h: &[f64]| {
            let cells: Vec<usize> = (0..n).filter(|&c| patch(c)).collect();
            cells.iter().map(|&c| sign(c) * (h[c] - top)).sum::<f64>() / cells.len() as f64
        };
        let start = amplitude(&wd.h);
        for k in 0..8 {
            wd.step_spreading(
                &p,
                &mut sv,
                &mut st,
                &vec![0.0; n],
                sea,
                Some((hs, 30_000.0)),
            );
            let (lo, hi) =
                wd.h.iter()
                    .fold((f64::MAX, f64::MIN), |m, &z| (m.0.min(z), m.1.max(z)));
            assert!(
                lo >= top - 300.0 - 1.0 && hi <= top + 300.0 + 1.0,
                "step {k}: the plateau left its range, {lo:.0} to {hi:.0} m"
            );
        }
        let end = amplitude(&wd.h);
        assert!(
            start == 300.0 && end.abs() < 0.01 * start,
            "the checkerboard went from {start:.1} to {end:.1} m"
        );
    }

    /// A plate boundary sweeping across a bent plate, stepping between rows as the plate model's
    /// boundaries do on the raster (near the poles its fields are sampled more than a cell apart
    /// north to south): each cell the boundary reaches loses its rigidity (ISO-055) and gives up
    /// the load it held, so it settles towards local isostasy and no further (ISO-056). No step
    /// moves the ground by more than the plate held it from local isostasy, and nothing is pushed
    /// far past it. While the plate released its bending moment as a moment, the curvature of a
    /// moment released along the boundary moved the ground by up to 1.46 times the load held and
    /// pushed it 4.2 km past local isostasy here (0.55 of the load held): the polar holes and
    /// spikes of the build log, 2026-10-18.
    #[test]
    fn a_plate_that_weakens_settles_towards_local_isostasy() {
        let p = on();
        let (w, h) = (64usize, 48usize);
        let n = w * h;
        let sea = -1.0e6;
        let mut wd = World::new(w, h, 300.0);
        let (mut sv, mut st) = started(&mut wd, &p, sea);
        // A disc raised in one step on the craton: the plate holds it up, with a basin and a
        // bulge round it.
        for c in 0..n {
            let (i, j) = ((c % w) as f64, (c / w) as f64);
            if (i - 26.0).powi(2) + (j - 24.0).powi(2) < 16.0 {
                wd.uplift[c] = 1.0;
            }
        }
        let mut held = wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea).flexure_m;
        wd.uplift = vec![0.0; n];
        let mut let_go = 0.0f64;
        for k in 0..10 {
            // A second plate south of a boundary along x that moves three rows a step and steps
            // two rows back and forth every two columns.
            for c in 0..n {
                let (i, j) = (c % w, c / w);
                let row = 10 + 3 * k + 2 * ((i / 2) % 2);
                wd.plate[c] = if (row..row + 40).contains(&j) {
                    1.0
                } else {
                    0.0
                };
            }
            let out = wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea);
            let most = held.iter().fold(0.0f64, |m, &v| m.max(f64::from(v).abs()));
            let past = held
                .iter()
                .zip(&out.flexure_m)
                .fold(0.0f64, |m, (&b, &a)| m.max(-f64::from(a * b.signum())));
            assert!(
                out.max_change_m <= 1.05 * most,
                "step {k}: the ground moved {:.0} m, the plate held {most:.0} m",
                out.max_change_m
            );
            assert!(
                past <= 0.25 * most,
                "step {k}: {past:.0} m past local isostasy, the plate held {most:.0} m"
            );
            let_go = let_go.max(out.max_change_m);
            held = out.flexure_m;
        }
        assert!(
            let_go > 5000.0,
            "the boundary crossed the disc: {let_go:.0} m"
        );
    }

    /// A narrow column on a craton fed crust step after step (the collapse's receiver in the
    /// polar peaks): the elastic plate holds its root up without bound; with the yield on
    /// (ISO-120) the plate holds at most its bound, the rest sinks under the column, and a load
    /// under the bound moves exactly as before.
    #[test]
    fn a_plate_that_yields_holds_no_more_than_its_strength() {
        let (w, h) = (64usize, 64usize);
        let n = w * h;
        let sea = -1.0e6;
        let column = |c: usize| (31..33).contains(&(c % w)) && (31..33).contains(&(c / w));
        // After each step: the most any cell's plate holds, the column's height, the cells that
        // yielded.
        let run = |p: &FlexureParams, mm_yr: f32, steps: usize| -> Vec<(f64, f64, usize)> {
            let mut wd = World::new(w, h, 300.0);
            let (mut sv, mut st) = started(&mut wd, p, sea);
            for c in 0..n {
                if column(c) {
                    wd.uplift[c] = mm_yr;
                }
            }
            (0..steps)
                .map(|_| {
                    let out = wd.step(p, &mut sv, &mut st, &vec![0.0; n], sea);
                    let most = out
                        .flexure_m
                        .iter()
                        .fold(0.0f64, |m, &v| m.max(f64::from(v).abs()));
                    let top = (0..n)
                        .filter(|&c| column(c))
                        .map(|c| wd.h[c])
                        .fold(f64::MIN, f64::max);
                    (most, top, out.yielded_cells)
                })
                .collect()
        };
        let elastic = on();
        let strong = FlexureParams {
            yield_anomaly_mgal: 380.0, // ledger: ISO-120
            ..on()
        };
        strong.validate().unwrap();
        let bound = strong.yield_held_kg_m2().unwrap() / RHO_MANTLE;
        assert!(
            (bound - 2745.0).abs() < 5.0,
            "380 mGal is {bound:.0} m of mantle"
        );
        // 1 mm/yr of Airy height a step, 14 km of crust: the elastic plate holds most of its root.
        let held = run(&elastic, 1.0, 12);
        assert!(
            held[11].0 > 4.0 * bound,
            "the elastic plate holds {:.0} m",
            held[11].0
        );
        let yielded = run(&strong, 1.0, 12);
        for (k, (&(most, top, cells), &(_, top_elastic, _))) in
            yielded.iter().zip(&held).enumerate()
        {
            assert!(
                most <= bound + 1.0,
                "step {k}: {most:.0} m held, the bound {bound:.0} m"
            );
            assert!(
                cells > 0 && top < top_elastic - 1000.0,
                "step {k}: {cells} cells yielded, the column {top:.0} m against {top_elastic:.0} m"
            );
        }
        // The column's Airy height is 2 km higher each step; the yielded plate holds it at most
        // the bound above that.
        let airy = 300.0 + 12.0 * 2000.0;
        assert!(
            yielded[11].1 <= airy + bound + 50.0,
            "{:.0} m against {airy:.0} + {bound:.0} m",
            yielded[11].1
        );
        // A load the plate can hold: nothing yields and nothing differs.
        let small = run(&elastic, 0.05, 3);
        assert!(small[2].0 < 0.9 * bound, "{small:?}");
        assert_eq!(small, run(&strong, 0.05, 3));
    }

    /// The sines at the `h + 1` row boundaries of a raster of `h` rows of side `dx` on Lambert's
    /// cylindrical equal-area projection of a sphere of radius `r_m` (y = R sin φ), centred on the
    /// equator, north first; past a pole they clamp to ±1.
    fn sphere_faces(h: usize, dx: f64, r_m: f64) -> Vec<f64> {
        (0..=h)
            .map(|k| ((0.5 * h as f64 - k as f64) * dx / r_m).clamp(-1.0, 1.0))
            .collect()
    }

    /// The sphere's metric (ISO-127): the block metric at the equator, 1 / cos²φ east to west and
    /// cos²φ north to south elsewhere, closed at the raster's first and last edge, and taken on
    /// every `f`-th boundary for a coarser solve raster.
    #[test]
    fn the_row_metric_is_the_block_metric_at_the_equator() {
        let (w, h) = (512usize, 164usize);
        let r_m = 6.371e6;
        let dx = std::f64::consts::TAU * r_m / w as f64;
        let faces = sphere_faces(h, dx, r_m);
        assert!(
            faces[0] == 1.0 && faces[h] == -1.0,
            "the raster covers the poles"
        );
        let m = RowMetric::from_sin_faces(&faces, 1);
        assert_eq!((m.a.len(), m.b.len()), (h, h + 1));
        assert_eq!((m.b[0], m.b[h]), (0.0, 0.0));
        assert_eq!(m.b[h / 2], 1.0, "the equator");
        assert!((m.a[h / 2] - 1.0).abs() < 1e-4, "{}", m.a[h / 2]);
        for j in 0..h {
            let s = 0.5 * (faces[j] + faces[j + 1]);
            if s.abs() < 0.99 {
                assert!((m.a[j] * (1.0 - s * s) - 1.0).abs() < 1e-12, "row {j}");
                assert!((m.b[j + 1] - (1.0 - faces[j + 1] * faces[j + 1])).abs() < 1e-15);
            }
            assert!(m.a[j].is_finite() && m.a[j] >= 1.0);
        }
        let m2 = RowMetric::from_sin_faces(&faces, 2);
        assert_eq!((m2.a.len(), m2.b.len()), (h / 2, h / 2 + 1));
        assert_eq!(m2.b[1], 1.0 - faces[2] * faces[2]);
        assert_eq!(RowMetric::block(4).b, vec![0.0, 1.0, 1.0, 1.0, 0.0]);
    }

    /// On the sphere's metric (ISO-127) the operator L D L + C is symmetric; the row solve
    /// inverts it exactly for a uniform plate (conjugate gradients stop after one iteration), and
    /// with the block metric's couplings it gives the cosine transform's answer; a plate whose
    /// rigidity varies a hundredfold converges.
    #[test]
    fn the_solve_on_the_sphere_inverts_its_operator() {
        let (w, h, dx) = (64usize, 32usize, 50_000.0);
        let faces = sphere_faces(h, dx, 0.49 * h as f64 * dx);
        let metric = RowMetric::from_sin_faces(&faces, 1);
        let n = w * h;
        let noise = |seed: usize| -> Vec<f64> {
            (0..n)
                .map(|c| (((c * 7919 + seed * 104_729) % 1009) as f64 - 504.0) * 10.0)
                .collect()
        };
        // Symmetric, with a plate twice as soft in a band and a wet zone.
        let mut sv = Solver::with_metric(w, h, dx, Some(metric.clone()));
        let d: Vec<f64> = (0..n)
            .map(|c| rigidity_per_g(if (c / w) % 9 < 4 { 40.0 } else { 8.0 }, G))
            .collect();
        let cr: Vec<f64> = (0..n)
            .map(|c| {
                if c % w < 20 {
                    RHO_MANTLE - RHO_WATER
                } else {
                    RHO_MANTLE
                }
            })
            .collect();
        let (u, v) = (noise(1), noise(2));
        let (mut au, mut av) = (vec![0.0; n], vec![0.0; n]);
        sv.operator(&d, &cr, &u, &mut au);
        sv.operator(&d, &cr, &v, &mut av);
        let (uav, vau) = (dot(w, &u, &av), dot(w, &v, &au));
        assert!(
            (uav - vau).abs() <= 1e-12 * uav.abs().max(vau.abs()),
            "{uav} vs {vau}"
        );

        // Exact for a uniform plate.
        let (du, cu) = uniform(n, 25.0, G);
        let b = noise(3);
        let (x, rep) = sv.solve(&du, &cu, &b, 1e-9, 20);
        assert_eq!(rep.iterations, 1, "{rep:?}");
        let mut ax = vec![0.0; n];
        sv.operator(&du, &cu, &x, &mut ax);
        let worst = ax
            .iter()
            .zip(&b)
            .map(|(p, q)| (p - q).abs())
            .fold(0.0, f64::max);
        let top = b.iter().map(|q| q.abs()).fold(0.0, f64::max);
        assert!(worst < 1e-9 * top, "{worst} of {top}");

        // The block metric's couplings through the row solve: the cosine transform's answer.
        let mut rows = Solver::with_metric(w, h, dx, Some(RowMetric::block(h)));
        let mut block = Solver::new(w, h, dx);
        let (xr, _) = rows.solve(&du, &cu, &b, 1e-9, 20);
        let (xb, _) = block.solve(&du, &cu, &b, 1e-9, 20);
        let peak = xb.iter().map(|v| v.abs()).fold(0.0, f64::max);
        let diff = xr
            .iter()
            .zip(&xb)
            .map(|(p, q)| (p - q).abs())
            .fold(0.0, f64::max);
        assert!(diff < 1e-9 * peak, "{diff} of {peak}");

        // A plate whose rigidity varies.
        let (x, rep) = sv.solve(&d, &cr, &b, 1e-6, 200);
        assert!(rep.converged, "{rep:?}");
        sv.operator(&d, &cr, &x, &mut ax);
        let worst = ax
            .iter()
            .zip(&b)
            .map(|(p, q)| (p - q).abs())
            .fold(0.0, f64::max);
        assert!(worst < 1e-3 * top, "{worst} of {top}");
    }

    /// The fault the sphere's metric mends (ISO-127): a load along a row (an east-west strip) at
    /// latitude φ on a plate of flexural parameter α, against the analytic strip on an elastic
    /// foundation (Hetényi: w = q/2k (2 − e^(−a/α) cos(a/α) − e^(−b/α) cos(b/α)) inside it,
    /// q/2k (e^(−a/α) cos(a/α) − e^(−b/α) cos(b/α)) outside, a and b the distances to its edges)
    /// in true distance, the strip as wide as the row is tall on the sphere. The block metric
    /// takes the row as tall as at the equator: its strip carries cos φ of the load and its
    /// profile is 1 / cos φ too wide. Prints the table (`--nocapture`).
    #[test]
    fn a_load_along_a_row_bends_the_plate_as_on_the_sphere() {
        let (w, h) = (1024usize, 328usize);
        let r_m = 6.371e6;
        let dx = std::f64::consts::TAU * r_m / w as f64;
        let faces = sphere_faces(h, dx, r_m);
        let alpha = 400_000.0;
        let c = RHO_MANTLE;
        let n = w * h;
        let d = vec![c * alpha.powi(4) / 4.0; n];
        let cr = vec![c; n];
        let q = 1.0e4; // kg/m²
        let mut sphere = Solver::with_metric(w, h, dx, Some(RowMetric::from_sin_faces(&faces, 1)));
        let mut block = Solver::new(w, h, dx);
        let lat = |k: usize| faces[k].asin();
        let mid = |j: usize| 0.5 * (lat(j) + lat(j + 1));
        let strip = |s: f64, half: f64| -> f64 {
            let f = |x: f64| (-x / alpha).exp() * (x / alpha).cos();
            let (a, b) = ((s.abs() - half).abs(), s.abs() + half);
            if s.abs() < half {
                q / (2.0 * c) * (2.0 - f(a) - f(b))
            } else {
                q / (2.0 * c) * (f(a) - f(b))
            }
        };
        eprintln!("lat   rows/α  w0_m    sphere_peak  block_peak  sphere_worst  block_worst");
        for (deg, sphere_most, block_least) in [
            (0.0, 0.02, 0.0),
            (45.0, 0.03, 0.2),
            (60.0, 0.05, 0.4),
            (70.0, 0.08, 0.5),
            (80.0, f64::INFINITY, 0.5),
        ] {
            let j0 = (0..h)
                .min_by(|&p, &k| {
                    (mid(p).to_degrees() - deg)
                        .abs()
                        .total_cmp(&(mid(k).to_degrees() - deg).abs())
                })
                .unwrap();
            let half = 0.5 * r_m * (lat(j0) - lat(j0 + 1));
            let b: Vec<f64> = (0..n).map(|k| if k / w == j0 { q } else { 0.0 }).collect();
            let w0 = strip(0.0, half);
            // Each solve's worst error, over the rows within 2α of the strip on the equator's
            // side, as a share of the peak.
            let worst = |x: &[f64]| -> (f64, f64) {
                let mut most = 0.0f64;
                for j in j0..h {
                    let s = r_m * (mid(j0) - mid(j));
                    if s > 2.0 * alpha {
                        break;
                    }
                    most = most.max((x[j * w] - strip(s, half)).abs() / w0);
                }
                (x[j0 * w], most)
            };
            let (xs, rs) = sphere.solve(&d, &cr, &b, 1e-9, 20);
            let (xb, rb) = block.solve(&d, &cr, &b, 1e-9, 20);
            assert!(rs.converged && rb.converged, "{rs:?} {rb:?}");
            let (ps, es) = worst(&xs);
            let (pb, eb) = worst(&xb);
            eprintln!(
                "{deg:>4.0}  {:>6.2}  {w0:>6.3}  {ps:>11.3}  {pb:>10.3}  {es:>12.3}  {eb:>11.3}",
                alpha / (2.0 * half)
            );
            assert!(es <= sphere_most, "{deg}°: the sphere's worst {es:.3}");
            assert!(
                eb >= block_least,
                "{deg}°: the block metric's worst {eb:.3}"
            );
            assert!(
                es <= eb + 1e-9,
                "{deg}°: {es:.3} against the block's {eb:.3}"
            );
        }
    }
}

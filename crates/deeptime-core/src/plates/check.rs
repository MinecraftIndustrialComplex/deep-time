// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime plate-check`: the plate dynamics of a finished run (`dynamics.rs`) against the target
//! ranges of `docs/design/plate-collisions-and-welding.md` §6.
//!
//! [`MEASURES`] is the one place the ranges live (the design's table is their source; a calibration
//! task moves them there, or in a file given to the check, [`Targets::apply_json`]). A measure is
//! computed per run ([`measures`]); the check takes the mean over the runs it is given (the design
//! asserts the plate targets on the mean of seeds 1–4) and passes it when it lies inside its range,
//! both ends included. A measure with no range is reported only. A measure the run cannot give
//! (no collision, no plate that died) is *no data*, which fails a ranged measure: a run too short
//! to measure something has not shown it holds.
//!
//! Every share is of the planet's L0 samples unless the measure says "plates" (then of the
//! visible planet, as the design measured them). The definitions follow the throwaway trace the
//! design's numbers came from (`tools/review/plate-dynamics-trace.patch`, now removed), so a
//! baseline on master reproduces the design's "Now" column.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use super::dynamics::{
    BeltRec, Dynamics, MIX_CLASSES, PlateForces, PlateLife, PlateStep, StepRec,
};
use super::stats::Margins;

/// One measure: its key, label, unit, range (inclusive; `None` is open) and how it is taken.
#[derive(Clone, Copy, Debug)]
pub struct MeasureDef {
    pub key: &'static str,
    pub label: &'static str,
    pub unit: &'static str,
    pub lo: Option<f64>,
    pub hi: Option<f64>,
    /// Set for a measure that needs physics not built yet: what it waits for. Without data it is
    /// reported as "no data" and does not fail the check.
    pub pending: Option<&'static str>,
}

const fn m(
    key: &'static str,
    label: &'static str,
    unit: &'static str,
    lo: Option<f64>,
    hi: Option<f64>,
) -> MeasureDef {
    MeasureDef {
        key,
        label,
        unit,
        lo,
        hi,
        pending: None,
    }
}

/// A measure that waits for physics not built yet (`why` names the design task).
const fn waits(
    key: &'static str,
    label: &'static str,
    unit: &'static str,
    lo: Option<f64>,
    hi: Option<f64>,
    why: &'static str,
) -> MeasureDef {
    MeasureDef {
        key,
        label,
        unit,
        lo,
        hi,
        pending: Some(why),
    }
}

const WAITS_COLLISION: &str = "the collision state and belts (design tasks 2-3)";
const WAITS_FORCES: &str = "a run recorded with the force balance (dynamics version 4)";
const WAITS_MIX: &str = "a run recorded with the boundary mix and arc heights (dynamics version 3)";

/// The measures in table order: first the ones with the design's §6 ranges, then the ones only
/// reported (the design's §2.2 figures and what a force-balance plate model will be judged on).
pub const MEASURES: &[MeasureDef] = &[
    m(
        "largest_share_p50",
        "Largest plate, median share of the planet",
        "share",
        Some(0.15),
        Some(0.35),
    ),
    m(
        "largest_over_50_pct",
        "Steps with one plate above 50 %",
        "%",
        None,
        Some(10.0),
    ),
    m(
        "plates_over_5pct_p50",
        "Plates above 5 % of the planet, median",
        "plates",
        Some(5.0),
        Some(9.0),
    ),
    m(
        "plate_life_p50_myr",
        "Median plate lifetime (plates that died)",
        "Myr",
        Some(80.0),
        None,
    ),
    m(
        "dead_within_20_pct",
        "Plates dead within 20 Myr",
        "%",
        None,
        Some(15.0),
    ),
    m(
        "rift_alive_50_pct",
        "Rift-born plates alive after 50 Myr",
        "%",
        Some(60.0),
        None,
    ),
    m(
        "rift_welded_back_50_pct",
        "Rift-born plates welded back to their own rift family within 50 Myr",
        "%",
        None,
        Some(10.0),
    ),
    m(
        "collision_land_max_pct",
        "Land raised by one collision in one step, the most",
        "% of planet",
        None,
        Some(0.5),
    ),
    m(
        "land_gain_max_pct",
        "Most land gained in any step (after the first 10 Myr)",
        "% of planet",
        None,
        Some(1.5),
    ),
    m(
        "belt_width_km_p50",
        "Collision belt width, median (ground raised 100 m or more)",
        "km",
        None,
        Some(800.0),
    ),
    m(
        "belt_elongation_p50",
        "Collision belt elongation, median",
        "ratio",
        Some(4.0),
        None,
    ),
    m(
        "collision_length_p50_myr",
        "Collision length, first to last step of the collision, median",
        "Myr",
        Some(15.0),
        Some(50.0),
    ),
    m(
        "collision_length_p90_myr",
        "Collision length, first to last step, 90th percentile",
        "Myr",
        None,
        Some(120.0),
    ),
    m(
        "uplift_spread",
        "Peak uplift per collision: 90th ÷ 10th percentile",
        "ratio",
        Some(2.0),
        None,
    ),
    m(
        "collisions_per_100myr",
        "Collisions per 100 Myr",
        "per 100 Myr",
        Some(2.0),
        Some(6.0),
    ),
    m(
        "continental_min",
        "Continental crust, share of the planet, lowest step (after the first 10 Myr)",
        "share",
        Some(0.37),
        None,
    ),
    m(
        "continental_max",
        "Continental crust, share of the planet, highest step (after the first 10 Myr)",
        "share",
        None,
        Some(0.45),
    ),
    m(
        "active_margin_mean",
        "Active share of the continental margin (after the first 100 Myr)",
        "share",
        Some(0.25),
        Some(0.40),
    ),
    m(
        "ocean_age_p90_mean_myr",
        "Ocean floor, 90th percentile age, mean over the run after the first 100 Myr",
        "Myr",
        None,
        Some(230.0),
    ),
    // Task 1b: the force-balance design's section 7 targets that today's model can be measured on.
    m(
        "speed_rms_mm_yr",
        "Plate speed, area-weighted RMS (mean over steps)",
        "mm/yr",
        Some(35.0),
        Some(55.0),
    ),
    m(
        "speed_big_p10_mm_yr",
        "Plate speed, 10th percentile of plate-steps (plates of 30 samples or more)",
        "mm/yr",
        None,
        Some(20.0),
    ),
    m(
        "speed_big_p50_mm_yr",
        "Plate speed, median of plate-steps (plates of 30 samples or more)",
        "mm/yr",
        Some(30.0),
        Some(60.0),
    ),
    m(
        "speed_big_p90_mm_yr",
        "Plate speed, 90th percentile of plate-steps (plates of 30 samples or more)",
        "mm/yr",
        Some(70.0),
        Some(110.0),
    ),
    m(
        "speed_oceanic_over_continental",
        "Median speed of plates under 20 % continental ÷ of plates over 50 %",
        "ratio",
        Some(2.0),
        None,
    ),
    m(
        "speed_slab_over_free",
        "Median speed of plates with 20 % or more of their edge on slab ÷ of plates under 5 %",
        "ratio",
        Some(2.5),
        None,
    ),
    m(
        "speed_slab_rank_corr",
        "Speed against slab share of the edge, rank correlation",
        "rho",
        Some(0.65),
        None,
    ),
    m(
        "speed_area_rank_corr",
        "Speed against plate area, rank correlation",
        "rho",
        Some(-0.4),
        Some(0.2),
    ),
    m(
        "speed_over_150_pct",
        "Plate-steps above 150 mm/yr",
        "%",
        None,
        Some(0.5),
    ),
    m(
        "guard_bind_steps",
        "Steps on which the speed guard binds",
        "steps",
        None,
        Some(0.0),
    ),
    m(
        "rot_change_p50_pct",
        "Change of a plate's rotation per step, median",
        "%",
        None,
        Some(8.0),
    ),
    m(
        "rot_change_p90_pct",
        "Change of a plate's rotation per step, 90th percentile",
        "%",
        None,
        Some(20.0),
    ),
    m(
        "ocean_age_p50_myr",
        "Ocean floor, median age (mean over the run after the first 100 Myr)",
        "Myr",
        Some(50.0),
        Some(80.0),
    ),
    m(
        "ocean_old180_pct",
        "Ocean floor older than 180 Myr (mean over the run after the first 100 Myr)",
        "% of sea floor",
        None,
        Some(10.0),
    ),
    m(
        "ocean_old250_pct",
        "Ocean floor older than 250 Myr (mean over the run after the first 100 Myr)",
        "% of sea floor",
        None,
        Some(3.0),
    ),
    m(
        "supercontinent_tenure_myr",
        "Supercontinent tenure (largest continent from assembled to dispersed), median",
        "Myr",
        Some(100.0),
        Some(300.0),
    ),
    m(
        "supercontinent_period_myr",
        "Assembly-to-assembly period, median",
        "Myr",
        Some(300.0),
        Some(600.0),
    ),
    // The boundary mix and arc heights (dynamics version 3), judged against Earth: each range
    // carries its ledger id (docs/science/ledger/PLT.md, PLT-213 to PLT-221). Runs recorded
    // without them say "no data".
    waits(
        // ledger: PLT-213 (Earth 19.7 %, Bird 2003), PLT-214 (the range)
        "boundary_subduction_pct",
        "Plate boundary that is subducting (ocean-ocean and ocean-continent), share of the total length, mean over the run after the first 100 Myr",
        "% of boundary",
        Some(15.0),
        Some(25.0),
        WAITS_MIX,
    ),
    waits(
        // ledger: PLT-215
        "arc_oceanic_emergent_pct",
        "Oceanic (island) arc cells above sea level, share of the arc cells, over the run after the first 100 Myr",
        "% of arc cells",
        Some(10.0),
        Some(35.0),
        WAITS_MIX,
    ),
    waits(
        // ledger: PLT-216 (lo, abstract-only), PLT-217 (hi, from-memory)
        "arc_oceanic_p90_m",
        "Oceanic arc cells, 90th percentile elevation (the crest; mean over the run after the first 100 Myr)",
        "m",
        Some(-1700.0),
        Some(1000.0),
        WAITS_MIX,
    ),
    waits(
        // ledger: PLT-218
        "arc_continental_p50_m",
        "Continental arc cells, median elevation (mean over the run after the first 100 Myr)",
        "m",
        Some(1200.0),
        Some(4500.0),
        WAITS_MIX,
    ),
    waits(
        // ledger: PLT-219 (from-memory)
        "arc_continental_p90_m",
        "Continental arc cells, 90th percentile elevation (the crest; mean over the run after the first 100 Myr)",
        "m",
        Some(2000.0),
        Some(6000.0),
        WAITS_MIX,
    ),
    // Waiting for physics: reported as "no data" until the record is filled (see `waits`).
    waits(
        "collision_convergence_ratio_20myr",
        "Convergence 20 Myr after a collision's onset ÷ at onset, median (needs the collision state)",
        "ratio",
        Some(0.2),
        Some(0.6),
        WAITS_COLLISION,
    ),
    waits(
        "wedge_width_km_p50",
        "Wedge on the incoming side, width, median (needs the belts)",
        "km",
        Some(100.0),
        Some(350.0),
        WAITS_COLLISION,
    ),
    waits(
        "foredeep_width_km_p50",
        "Foredeep width, median (needs the belts)",
        "km",
        Some(150.0),
        Some(450.0),
        WAITS_COLLISION,
    ),
    waits(
        "foredeep_depth_m_p50",
        "Foredeep depth of fill, median (needs the belts)",
        "m",
        Some(2000.0),
        Some(7000.0),
        WAITS_COLLISION,
    ),
    // Reported only: the design's §2.2 figures.
    m(
        "largest_over_30_pct",
        "Steps with one plate above 30 %",
        "%",
        None,
        None,
    ),
    m(
        "largest_share_max",
        "Largest plate, worst step",
        "share",
        None,
        None,
    ),
    m(
        "plates_alive_p50",
        "Plates alive, median",
        "plates",
        None,
        None,
    ),
    m(
        "megaplate_regrow_myr_p50",
        "After a rift splits a plate above half the planet: until one is above half again, median",
        "Myr",
        None,
        None,
    ),
    m(
        "rift_life_p50_myr",
        "Rift-born plates that died: median lifetime",
        "Myr",
        None,
        None,
    ),
    m(
        "deaths_sutured",
        "Plate deaths by welding",
        "per run",
        None,
        None,
    ),
    m(
        "deaths_absorbed",
        "Plate deaths by absorption of a tiny plate",
        "per run",
        None,
        None,
    ),
    m(
        "deaths_consumed",
        "Plate deaths subducted away",
        "per run",
        None,
        None,
    ),
    m(
        "welded_dying_larger_pct",
        "Welds in which the dying plate is the larger one",
        "%",
        None,
        None,
    ),
    m(
        "land_share_p50",
        "Land, median share of the planet",
        "share",
        None,
        None,
    ),
    m(
        "continental_end",
        "Continental crust at the end, share of the planet",
        "share",
        None,
        None,
    ),
    m(
        "collision_lifted10_pct_p50",
        "Ground one collision lifts by more than 10 m: median",
        "% of planet",
        None,
        None,
    ),
    m(
        "collision_lifted10_pct_max",
        "Ground one collision lifts by more than 10 m: the most",
        "% of planet",
        None,
        None,
    ),
    m(
        "belt_length_km_p50",
        "Collision belt length, median (ground raised 100 m or more)",
        "km",
        None,
        None,
    ),
    m(
        "belt_elongation_over_3_pct",
        "Collisions whose belt is at least 3 times longer than wide",
        "%",
        None,
        None,
    ),
    m(
        "contact_age_p50_myr",
        "Contact age at the collision, median",
        "Myr",
        None,
        None,
    ),
    m(
        "contact_age_p90_myr",
        "Contact age at the collision, 90th percentile",
        "Myr",
        None,
        None,
    ),
    m(
        "counted_km_p50",
        "What the collision counter read, median",
        "km",
        None,
        None,
    ),
    m(
        "shortening_km_p50",
        "Real shortening at the collision, median",
        "km",
        None,
        None,
    ),
    m(
        "shortening_km_p90",
        "Real shortening at the collision, 90th percentile",
        "km",
        None,
        None,
    ),
    m(
        "ocean_age_p90_myr",
        "Ocean floor, 90th percentile age, last step",
        "Myr",
        None,
        None,
    ),
    m(
        "continental_min_all",
        "Continental crust, share of the planet, lowest step (all steps)",
        "share",
        None,
        None,
    ),
    m(
        "continental_max_all",
        "Continental crust, share of the planet, highest step (all steps)",
        "share",
        None,
        None,
    ),
    // Reported only: what a force-balance plate model is judged on.
    m(
        "speed_p50_mm_yr",
        "Plate speed, median over plates and steps",
        "mm/yr",
        None,
        None,
    ),
    m(
        "speed_slab_mm_yr",
        "Speed of plates with a slab on 20 % of their boundary or more",
        "mm/yr",
        None,
        None,
    ),
    m(
        "speed_noslab_mm_yr",
        "Speed of plates with a slab on less than 20 % of their boundary",
        "mm/yr",
        None,
        None,
    ),
    m(
        "speed_slab_ratio",
        "Slab-attached ÷ other plate speed (Earth: 3 to 4)",
        "ratio",
        None,
        None,
    ),
    m(
        "speed_slab_corr",
        "Correlation of plate speed with slab share of the boundary",
        "r",
        None,
        None,
    ),
    m(
        "slab_share_mean",
        "Slab share of the boundary, mean over plates and steps",
        "share",
        None,
        None,
    ),
    m(
        "speed_oceanic_mm_yr",
        "Speed of mostly oceanic plates (under half continental)",
        "mm/yr",
        None,
        None,
    ),
    m(
        "speed_continental_mm_yr",
        "Speed of mostly continental plates",
        "mm/yr",
        None,
        None,
    ),
    // Task 1b, reported only: what the force-balance design asks to see.
    m(
        "speed_over_150_plate_steps",
        "Plate-steps above 150 mm/yr, count",
        "plate-steps",
        None,
        None,
    ),
    m(
        "guard_bind_plate_steps",
        "Plate-steps on which the speed guard binds",
        "plate-steps",
        None,
        None,
    ),
    m(
        "pole_inside_pct",
        "Plate-steps turning about a pole on the plate itself",
        "%",
        None,
        None,
    ),
    m(
        "spinning_pct",
        "Plate-steps turning about a pole within 30° of their own centroid (design: near themselves)",
        "%",
        None,
        None,
    ),
    m(
        "continent_pieces_p50",
        "Connected pieces of continental crust (of 1 % or more of it), median over steps",
        "pieces",
        None,
        None,
    ),
    m(
        "continent_largest_share_p50",
        "Largest continental piece, share of the continental crust, median over steps",
        "share",
        None,
        None,
    ),
    m(
        "continent_largest_share_min",
        "Largest continental piece, share of the continental crust, lowest step",
        "share",
        None,
        None,
    ),
    m(
        "supercontinent_episodes",
        "Supercontinent assemblies in the run (one at the start counts)",
        "episodes",
        None,
        None,
    ),
    m(
        "supercontinent_time_pct",
        "Steps with a supercontinent assembled",
        "%",
        None,
        None,
    ),
    m(
        "megaplate_rifts",
        "Rifts of a plate above 40 % of the planet, per run",
        "rifts",
        None,
        None,
    ),
    m(
        "megaplate_tenure_p50_myr",
        "A plate above 40 % of the planet: time above it until a rift splits it, median",
        "Myr",
        None,
        None,
    ),
    m(
        "megaplate_rift_apart_50_pct",
        "Rifts of such a plate whose fragments are all still separate 50 Myr later (strict)",
        "%",
        None,
        None,
    ),
    m(
        "megaplate_rift_split_50_pct",
        "Rifts of such a plate not healed whole: two plates or more still hold its crust 50 Myr later",
        "%",
        None,
        None,
    ),
    // Reported only: the boundary mix beyond its judged share (Earth, Bird 2003: spreading 36 %,
    // transform 28 %, continental convergent 9 %, other convergent 7 %, subduction 20 %).
    m(
        "boundary_total_km",
        "Plate boundary, total length per step, mean (after the first 100 Myr; Earth: 260,487 km, Bird 2003)",
        "km",
        None,
        None,
    ),
    m(
        "boundary_subduction_km",
        "Subducting plate boundary, length per step, mean (after the first 100 Myr; Earth: 51,310 km, Bird 2003)",
        "km",
        None,
        None,
    ),
    m(
        "arc_oceanic_p50_m",
        "Oceanic arc cells, median elevation (mean over the run after the first 100 Myr; band cells include the flanks, not only summits)",
        "m",
        None,
        None,
    ),
    m(
        "boundary_oo_of_subduction_pct",
        "Subducting boundary that is ocean-ocean, share of the subducting length (after the first 100 Myr)",
        "% of subduction",
        None,
        None,
    ),
    m(
        "boundary_spreading_pct",
        "Plate boundary that is spreading (ridge and rift), share of the total length (after the first 100 Myr)",
        "% of boundary",
        None,
        None,
    ),
    m(
        "boundary_collision_pct",
        "Plate boundary at a collision (continental crust at the subducting side), share of the total length (after the first 100 Myr)",
        "% of boundary",
        None,
        None,
    ),
    m(
        "boundary_other_pct",
        "Plate boundary that is transform or converging with no contact yet, share of the total length (after the first 100 Myr)",
        "% of boundary",
        None,
        None,
    ),
    m(
        "boundary_subduction_pct_early",
        "Plate boundary that is subducting, share of the total length, first 50 Myr",
        "% of boundary",
        None,
        None,
    ),
    m(
        "boundary_oo_of_subduction_pct_early",
        "Subducting boundary that is ocean-ocean, share of the subducting length, first 50 Myr",
        "% of subduction",
        None,
        None,
    ),
    m(
        "boundary_spreading_pct_early",
        "Plate boundary that is spreading, share of the total length, first 50 Myr",
        "% of boundary",
        None,
        None,
    ),
    m(
        "boundary_collision_pct_early",
        "Plate boundary at a collision, share of the total length, first 50 Myr",
        "% of boundary",
        None,
        None,
    ),
    m(
        "boundary_other_pct_early",
        "Plate boundary that is transform or converging with no contact yet, share of the total length, first 50 Myr",
        "% of boundary",
        None,
        None,
    ),
    m(
        "arc_oceanic_cells_mean",
        "Oceanic arc cells per step, mean (after the first 100 Myr)",
        "cells",
        None,
        None,
    ),
    m(
        "arc_continental_cells_mean",
        "Continental arc cells per step, mean (after the first 100 Myr)",
        "cells",
        None,
        None,
    ),
    m(
        "arc_oceanic_emergent_pct_early",
        "Oceanic arc cells above sea level, share, first 50 Myr",
        "% of arc cells",
        None,
        None,
    ),
    m(
        "arc_oceanic_p90_m_early",
        "Oceanic arc cells, 90th percentile elevation, first 50 Myr",
        "m",
        None,
        None,
    ),
    m(
        "arc_continental_p90_m_early",
        "Continental arc cells, 90th percentile elevation, first 50 Myr",
        "m",
        None,
        None,
    ),
    // The force balance (design task 5): recorded, not applied, so these say what the balance
    // would do on today's geometry (open loop), against today's motion. Reported only: the
    // design's section 7 targets judge the motion after the switch (task 7).
    waits(
        "force_slab_pull_share_p50",
        "Share of a plate's driving torque that is slab pull (by size, of pull, suction, edge energy and mantle flow), median over plates of 30+ samples",
        "share",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "megaplate_stress_mpa_p50",
        "Mean in-plane stress of a plate above 40 %, median; tension positive",
        "MPa",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "megaplate_tension_pct",
        "Steps in which a plate above 40 % is in tension",
        "%",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "slab_length_km_p50",
        "Slab length per subducting contact, median over contact-steps",
        "km",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "slab_breakoffs_per_100myr",
        "Slab break-offs (continental crust held the trench for slab_breakoff_myr), per 100 Myr",
        "per 100 Myr",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_rms_mm_yr",
        "Balance (not applied): plate speed, area-weighted RMS, mean over steps (design target 35-55)",
        "mm/yr",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_p10_mm_yr",
        "Balance: plate speed, 10th percentile of plate-steps (target at most 20)",
        "mm/yr",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_p50_mm_yr",
        "Balance: plate speed, median of plate-steps (target 30-60)",
        "mm/yr",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_p90_mm_yr",
        "Balance: plate speed, 90th percentile of plate-steps (target 70-110)",
        "mm/yr",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_norm_p10_mm_yr",
        "Balance scaled each step to an RMS of 45 mm/yr (the design's open-loop test): speed, 10th percentile (design: 27-31)",
        "mm/yr",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_norm_p50_mm_yr",
        "Balance scaled to an RMS of 45 mm/yr: speed, median (design: 62-68)",
        "mm/yr",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_norm_p90_mm_yr",
        "Balance scaled to an RMS of 45 mm/yr: speed, 90th percentile (design: 95-109)",
        "mm/yr",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_over_150_pct",
        "Balance: plate-steps above 150 mm/yr (target at most 0.5)",
        "%",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_over_250_pct",
        "Balance: plate-steps above 250 mm/yr, where the guard of the switch would bind (target 0)",
        "%",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_oceanic_over_continental",
        "Balance: median speed, plates under 20 % continental / plates over 50 % (target at least 2)",
        "ratio",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_slab_over_free",
        "Balance: median speed, plates with 20 % or more of the edge on slab / under 5 % (target at least 2.5)",
        "ratio",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_slab_rank_corr",
        "Balance: speed against slab share of the edge, rank correlation (target at least 0.65)",
        "rho",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_area_rank_corr",
        "Balance: speed against plate area, rank correlation (target -0.4 to 0.2)",
        "rho",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_rot_change_p50_pct",
        "Balance: change of a plate's rotation per step, median (target at most 8)",
        "%",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_rot_change_p90_pct",
        "Balance: change of a plate's rotation per step, 90th percentile (target at most 20)",
        "%",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_vs_motion_rank_corr",
        "Balance against today's motion: rank correlation of the two speeds over plate-steps",
        "rho",
        None,
        None,
        WAITS_FORCES,
    ),
    waits(
        "balance_over_motion_p50",
        "Balance against today's motion: median of balance speed / today's speed over plate-steps",
        "ratio",
        None,
        None,
        WAITS_FORCES,
    ),
];

/// What the measures take besides target ranges: definitions the design leaves to the check.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    /// A supercontinent is assembled when the largest connected piece of continental crust holds
    /// this share of the continental crust or more (the force-balance design's "70 % or more")...
    pub assembled_share: f64,
    /// ... and stays so until it holds less than this ("until it holds under 50 %"): a plain
    /// threshold would count every flicker around it as a break-up.
    pub dispersed_share: f64,
    /// A plate is a mega-plate while it holds more than this share of the visible planet.
    pub mega_plate_share: f64,
    /// Speeds, rotation changes and the pole test count plates of at least this many visible
    /// samples (the design's trace: "plates of at least 30 samples").
    pub min_plate_samples: f64,
    /// Plate speed above which a plate-step counts as fast, mm/yr.
    pub fast_mm_yr: f64,
    /// The sea-floor measures leave out this many Myr from the start of the run (the starting
    /// ocean's ages are drawn, not made by the model).
    pub warmup_myr: f64,
    /// The continental crust's lowest and highest step leave out this many Myr from the start
    /// (the same start-up as the land gain's).
    pub start_up_myr: f64,
    /// A piece of continental crust counts in the piece count when it holds at least this share
    /// of the continental crust.
    pub min_piece_share: f64,
    /// How long after a rift its fragments must still be separate, Myr.
    pub rift_window_myr: f64,
    /// The boundary mix and the arc heights are also reported for the first this many Myr.
    pub early_myr: f64,
    /// The RMS speed the balance's answers are scaled to, step by step, for the comparison with
    /// the design's open-loop test (its `plate_speed_scale`), mm/yr.
    pub balance_rms_mm_yr: f64,
    /// The speed guard of the switch (the design raises `max_plate_speed_mm_yr` to it), mm/yr.
    pub balance_guard_mm_yr: f64,
}

impl Default for Params {
    fn default() -> Params {
        Params {
            assembled_share: 0.70,
            dispersed_share: 0.50,
            mega_plate_share: 0.40,
            min_plate_samples: 30.0,
            fast_mm_yr: 150.0,
            warmup_myr: 100.0,
            start_up_myr: 10.0,
            min_piece_share: 0.01,
            rift_window_myr: 50.0,
            early_myr: 50.0,
            balance_rms_mm_yr: 45.0,    // ledger: CHK-058
            balance_guard_mm_yr: 250.0, // ledger: CHK-059
        }
    }
}

impl Params {
    /// The names `set` takes.
    pub const KEYS: &'static [&'static str] = &[
        "assembled_share",
        "dispersed_share",
        "mega_plate_share",
        "min_plate_samples",
        "fast_mm_yr",
        "warmup_myr",
        "start_up_myr",
        "min_piece_share",
        "rift_window_myr",
        "early_myr",
        "balance_rms_mm_yr",
        "balance_guard_mm_yr",
    ];

    /// Sets one parameter by name.
    pub fn set(&mut self, key: &str, value: f64) -> Result<(), String> {
        if !value.is_finite() {
            return Err(format!("parameter {key}: {value} is not a finite number"));
        }
        match key {
            "assembled_share" => self.assembled_share = value,
            "dispersed_share" => self.dispersed_share = value,
            "mega_plate_share" => self.mega_plate_share = value,
            "min_plate_samples" => self.min_plate_samples = value,
            "fast_mm_yr" => self.fast_mm_yr = value,
            "warmup_myr" => self.warmup_myr = value,
            "start_up_myr" => self.start_up_myr = value,
            "min_piece_share" => self.min_piece_share = value,
            "rift_window_myr" => self.rift_window_myr = value,
            "early_myr" => self.early_myr = value,
            "balance_rms_mm_yr" => self.balance_rms_mm_yr = value,
            "balance_guard_mm_yr" => self.balance_guard_mm_yr = value,
            _ => {
                return Err(format!(
                    "no parameter {key:?} (names: {})",
                    Self::KEYS.join(", ")
                ));
            }
        }
        Ok(())
    }

    /// `name=value` as the command line gives it.
    pub fn set_text(&mut self, text: &str) -> Result<(), String> {
        let (k, v) = text
            .split_once('=')
            .ok_or_else(|| format!("parameter {text:?}: expected name=value"))?;
        let v: f64 = v
            .trim()
            .parse()
            .map_err(|_| format!("parameter {k}: {v:?} is not a number"))?;
        self.set(k.trim(), v)
    }

    /// One line for the table's header.
    pub fn describe(&self) -> String {
        format!(
            "supercontinent assembled at {:.0} % of the continental crust in one piece, dispersed \
             under {:.0} %; mega-plate above {:.0} % of the planet; speeds of plates of {:.0}+ \
             samples; sea floor after the first {:.0} Myr; continental extremes after the first \
             {:.0} Myr; rift fragments apart after {:.0} Myr",
            100.0 * self.assembled_share,
            100.0 * self.dispersed_share,
            100.0 * self.mega_plate_share,
            self.min_plate_samples,
            self.warmup_myr,
            self.start_up_myr,
            self.rift_window_myr
        )
    }
}

/// Target ranges by measure key and the check's parameters: the built-in ones, with any overrides
/// applied.
#[derive(Clone, Debug)]
pub struct Targets {
    ranges: BTreeMap<&'static str, (Option<f64>, Option<f64>)>,
    pub params: Params,
}

impl Targets {
    /// The design's §6 ranges ([`MEASURES`]) and the default [`Params`].
    pub fn builtin() -> Targets {
        Targets {
            ranges: MEASURES.iter().map(|d| (d.key, (d.lo, d.hi))).collect(),
            params: Params::default(),
        }
    }

    /// The range of `key`.
    pub fn range(&self, key: &str) -> (Option<f64>, Option<f64>) {
        self.ranges.get(key).copied().unwrap_or((None, None))
    }

    /// Overrides from a JSON object `{"key": [lo, hi], …}`; `null` leaves an end open (and
    /// `"key": null` turns the measure into a reported one). The key `"params"` holds an object of
    /// [`Params`] values. Unknown keys are an error.
    pub fn apply_json(&mut self, text: &str) -> Result<(), String> {
        let v: Value = serde_json::from_str(text).map_err(|e| format!("targets file: {e}"))?;
        let obj = v
            .as_object()
            .ok_or("targets file: expected an object of measure -> [lo, hi]")?;
        for (key, range) in obj {
            if key == "params" {
                let o = range
                    .as_object()
                    .ok_or("targets file: params: expected an object of name -> number")?;
                for (name, v) in o {
                    let x = v
                        .as_f64()
                        .ok_or_else(|| format!("targets file: params.{name}: not a number"))?;
                    self.params.set(name, x)?;
                }
                continue;
            }
            let Some(slot) = self.ranges.get_mut(key.as_str()) else {
                return Err(format!(
                    "targets file: no measure {key:?} (keys: {})",
                    MEASURES
                        .iter()
                        .map(|d| d.key)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            };
            *slot = match range {
                Value::Null => (None, None),
                Value::Array(a) if a.len() == 2 => {
                    let end = |x: &Value| -> Result<Option<f64>, String> {
                        match x {
                            Value::Null => Ok(None),
                            _ => x
                                .as_f64()
                                .map(Some)
                                .ok_or_else(|| format!("targets file: {key}: {x} is not a number")),
                        }
                    };
                    (end(&a[0])?, end(&a[1])?)
                }
                _ => return Err(format!("targets file: {key}: expected [lo, hi] or null")),
            };
        }
        Ok(())
    }
}

fn median(xs: &[f64]) -> f64 {
    pct(xs, 0.5)
}

/// The `q` quantile (linear between the two nearest values), NaN of nothing.
fn pct(xs: &[f64], q: f64) -> f64 {
    let mut v: Vec<f64> = xs.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    let k = (v.len() - 1) as f64 * q;
    let (lo, hi) = (k.floor() as usize, k.ceil() as usize);
    v[lo] + (v[hi] - v[lo]) * (k - lo as f64)
}

fn mean(xs: impl IntoIterator<Item = f64>) -> f64 {
    let (s, n) = xs
        .into_iter()
        .filter(|x| x.is_finite())
        .fold((0.0, 0u32), |(s, n), x| (s + x, n + 1));
    if n == 0 { f64::NAN } else { s / f64::from(n) }
}

/// Shares of the visible planet of one step's plates, largest first.
fn shares(plates: &[PlateStep]) -> Vec<f64> {
    let total: u32 = plates.iter().map(|p| p.visible).sum();
    let mut s: Vec<f64> = plates
        .iter()
        .map(|p| f64::from(p.visible) / f64::from(total.max(1)))
        .collect();
    s.sort_by(|a, b| b.total_cmp(a));
    s
}

fn percent_of_steps(d: &Dynamics, f: impl Fn(&[f64]) -> bool) -> f64 {
    if d.steps.is_empty() {
        return f64::NAN;
    }
    let n = d.steps.iter().filter(|s| f(&shares(&s.plates))).count();
    100.0 * n as f64 / d.steps.len() as f64
}

/// The plate's life in Myr, for plates that died.
fn lives(d: &Dynamics) -> Vec<(&PlateLife, f64)> {
    d.plates
        .iter()
        .filter_map(|p| p.died_myr.map(|t| (p, t - p.born_myr)))
        .collect()
}

/// Rift-born plates welded back into the plate family they left (the plate holding their
/// parent's crust, or a sibling of the same rift) within `window_myr` of birth, among those born
/// at least `window_myr` before the end of the run (so none is cut off by it), as a percentage.
fn rift_welded_back(d: &Dynamics, window_myr: f64, end_myr: f64) -> f64 {
    // Plate ids change hands at every weld: follow them to the plate that holds the crust now.
    let mut succ: BTreeMap<u32, u32> = BTreeMap::new();
    fn holder(succ: &BTreeMap<u32, u32>, mut p: u32) -> u32 {
        let mut hops = 0;
        while let Some(&n) = succ.get(&p) {
            p = n;
            hops += 1;
            if hops > 100_000 {
                break;
            }
        }
        p
    }
    let mut dead: Vec<&PlateLife> = d.plates.iter().filter(|p| p.died_myr.is_some()).collect();
    dead.sort_by(|a, b| {
        a.died_myr
            .partial_cmp(&b.died_myr)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.id.cmp(&b.id))
    });
    let mut back: BTreeSet<u32> = BTreeSet::new();
    for p in dead {
        let died = p.died_myr.unwrap_or(f64::NAN);
        if p.origin == "rift" && p.fate.as_deref() == Some("sutured") {
            let siblings: BTreeSet<u32> = d
                .plates
                .iter()
                .filter(|q| {
                    q.id != p.id
                        && q.origin == "rift"
                        && q.born_myr == p.born_myr
                        && q.parent == p.parent
                })
                .map(|q| holder(&succ, q.id))
                .collect();
            if let (Some(into), Some(parent)) = (p.into, p.parent)
                && (into == holder(&succ, parent) || siblings.contains(&into))
                && died - p.born_myr <= window_myr
            {
                back.insert(p.id);
            }
        }
        if let Some(into) = p.into {
            succ.insert(p.id, into);
        }
    }
    let old: Vec<&PlateLife> = d
        .plates
        .iter()
        .filter(|p| p.origin == "rift" && p.born_myr <= end_myr - window_myr)
        .collect();
    if old.is_empty() {
        return f64::NAN;
    }
    100.0 * old.iter().filter(|p| back.contains(&p.id)).count() as f64 / old.len() as f64
}

/// The active share of the continental margin, mean over the records after the first 90 Myr of
/// the record (P1-27; all of them when the run is shorter): the margins line of `deeptime run`.
pub fn active_margin_mean(margins: &[Margins]) -> f64 {
    let Some(first) = margins.first() else {
        return f64::NAN;
    };
    let after: Vec<&Margins> = margins
        .iter()
        .filter(|m| m.t_myr - first.t_myr >= 90.0)
        .collect();
    let pool: Vec<&Margins> = if after.is_empty() {
        margins.iter().collect()
    } else {
        after
    };
    mean(pool.iter().map(|m| m.active_share()))
}

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len());
    if n < 3 {
        return f64::NAN;
    }
    let (mx, my) = (mean(x.iter().copied()), mean(y.iter().copied()));
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (a, b) in x.iter().zip(y) {
        sxy += (a - mx) * (b - my);
        sxx += (a - mx).powi(2);
        syy += (b - my).powi(2);
    }
    if sxx <= 0.0 || syy <= 0.0 {
        f64::NAN
    } else {
        sxy / (sxx * syy).sqrt()
    }
}

/// Every measure of one run, by key (NaN where the run cannot give it), with the default
/// [`Params`].
pub fn measures(d: &Dynamics, margins: &[Margins]) -> BTreeMap<&'static str, f64> {
    measures_with(d, margins, &Params::default())
}

/// Every measure of one run, by key (NaN where the run cannot give it).
pub fn measures_with(
    d: &Dynamics,
    margins: &[Margins],
    params: &Params,
) -> BTreeMap<&'static str, f64> {
    let mut out: BTreeMap<&'static str, f64> = BTreeMap::new();
    let mut put = |k: &'static str, v: f64| {
        out.insert(k, v);
    };
    let n = f64::from(d.samples.max(1));
    let first_t = d.steps.first().map_or(d.t0_myr, |s| s.t);
    let end_t = d.steps.last().map_or(d.t0_myr, |s| s.t);
    let span = end_t - d.t0_myr;

    // Plate sizes.
    let largest: Vec<f64> = d
        .steps
        .iter()
        .map(|s| shares(&s.plates).first().copied().unwrap_or(0.0))
        .collect();
    put("largest_share_p50", median(&largest));
    put(
        "largest_share_max",
        largest.iter().copied().fold(f64::NAN, f64::max),
    );
    put(
        "largest_over_50_pct",
        percent_of_steps(d, |s| s.first().is_some_and(|&x| x > 0.5)),
    );
    put(
        "largest_over_30_pct",
        percent_of_steps(d, |s| s.first().is_some_and(|&x| x > 0.3)),
    );
    let big: Vec<f64> = d
        .steps
        .iter()
        .map(|s| shares(&s.plates).iter().filter(|&&x| x >= 0.05).count() as f64)
        .collect();
    put("plates_over_5pct_p50", median(&big));
    // A rift that splits a plate above half the planet: how long until one plate is above half.
    let mut regrow = Vec::new();
    for i in 1..largest.len() {
        let rifted = d
            .plates
            .iter()
            .any(|p| p.origin == "rift" && (p.born_myr - d.steps[i].t).abs() < 1e-6);
        if rifted
            && largest[i - 1] > 0.5
            && largest[i] <= 0.5
            && let Some(j) = (i..largest.len()).find(|&j| largest[j] > 0.5)
        {
            regrow.push((j - i) as f64 * d.dt_myr);
        }
    }
    put("megaplate_regrow_myr_p50", median(&regrow));
    let alive: Vec<f64> = d.steps.iter().map(|s| s.plates.len() as f64).collect();
    put("plates_alive_p50", median(&alive));

    // Lifetimes and fates.
    let dead = lives(d);
    let life: Vec<f64> = dead.iter().map(|&(_, l)| l).collect();
    put("plate_life_p50_myr", median(&life));
    put(
        "dead_within_20_pct",
        if life.is_empty() {
            f64::NAN
        } else {
            100.0 * life.iter().filter(|&&l| l <= 20.0).count() as f64 / life.len() as f64
        },
    );
    let rift_life: Vec<f64> = dead
        .iter()
        .filter(|(p, _)| p.origin == "rift")
        .map(|&(_, l)| l)
        .collect();
    put("rift_life_p50_myr", median(&rift_life));
    let old_rift: Vec<&PlateLife> = d
        .plates
        .iter()
        .filter(|p| p.origin == "rift" && p.born_myr <= end_t - 50.0)
        .collect();
    put(
        "rift_alive_50_pct",
        if old_rift.is_empty() {
            f64::NAN
        } else {
            let alive_50 = old_rift
                .iter()
                .filter(|p| p.died_myr.is_none_or(|t| t - p.born_myr > 50.0))
                .count();
            100.0 * alive_50 as f64 / old_rift.len() as f64
        },
    );
    put("rift_welded_back_50_pct", rift_welded_back(d, 50.0, end_t));
    let fate = |f: &str| {
        d.plates
            .iter()
            .filter(|p| p.fate.as_deref() == Some(f))
            .count() as f64
    };
    put("deaths_sutured", fate("sutured"));
    put("deaths_absorbed", fate("absorbed"));
    put("deaths_consumed", fate("consumed"));
    // Of the collisions that welded a plate whole: how often the plate that died was the larger.
    let welds: Vec<&super::dynamics::CollisionRec> =
        d.collisions.iter().filter(|c| c.sutured).collect();
    put(
        "welded_dying_larger_pct",
        if welds.is_empty() {
            f64::NAN
        } else {
            let larger = welds
                .iter()
                .filter(|c| c.under_samples > c.over_samples)
                .count();
            100.0 * larger as f64 / welds.len() as f64
        },
    );

    // Land and continental crust.
    let land: Vec<f64> = d.steps.iter().map(|s| f64::from(s.land) / n).collect();
    put("land_share_p50", median(&land));
    let gains: Vec<f64> = d
        .steps
        .windows(2)
        .filter(|w| w[1].t >= first_t + 10.0)
        .map(|w| (f64::from(w[1].land) - f64::from(w[0].land)) / n)
        .collect();
    put(
        "land_gain_max_pct",
        100.0 * gains.iter().copied().fold(f64::NAN, f64::max),
    );
    let cont: Vec<f64> = d
        .steps
        .iter()
        .map(|s| f64::from(s.continental) / n)
        .collect();
    // The judged extremes leave out the start-up (the first `start_up_myr`), which is not the
    // model's doing; the extremes over every step are reported beside them.
    let settled: Vec<f64> = d
        .steps
        .iter()
        .filter(|s| s.t - d.t0_myr > params.start_up_myr)
        .map(|s| f64::from(s.continental) / n)
        .collect();
    let settled = if settled.is_empty() { &cont } else { &settled };
    put(
        "continental_min",
        settled.iter().copied().fold(f64::NAN, f64::min),
    );
    put(
        "continental_max",
        settled.iter().copied().fold(f64::NAN, f64::max),
    );
    put(
        "continental_min_all",
        cont.iter().copied().fold(f64::NAN, f64::min),
    );
    put(
        "continental_max_all",
        cont.iter().copied().fold(f64::NAN, f64::max),
    );
    put("continental_end", cont.last().copied().unwrap_or(f64::NAN));

    // Collisions: terranes of at least `collision_min_frac` of an average plate.
    let col: Vec<&super::dynamics::CollisionRec> =
        d.collisions.iter().filter(|c| c.collision).collect();
    let pick = |f: &dyn Fn(&super::dynamics::CollisionRec) -> f64| -> Vec<f64> {
        col.iter().map(|c| f(c)).collect()
    };
    let emerged = pick(&|c| 100.0 * f64::from(c.emerged.samples) / n);
    put(
        "collision_land_max_pct",
        emerged.iter().copied().fold(f64::NAN, f64::max),
    );
    let lifted = pick(&|c| 100.0 * f64::from(c.lifted10.samples) / n);
    put("collision_lifted10_pct_p50", median(&lifted));
    put(
        "collision_lifted10_pct_max",
        lifted.iter().copied().fold(f64::NAN, f64::max),
    );
    let shaped: Vec<&super::dynamics::CollisionRec> = col
        .iter()
        .copied()
        .filter(|c| c.lifted100.width_km > 0.0)
        .collect();
    put(
        "belt_width_km_p50",
        median(
            &shaped
                .iter()
                .map(|c| c.lifted100.width_km)
                .collect::<Vec<_>>(),
        ),
    );
    put(
        "belt_length_km_p50",
        median(
            &shaped
                .iter()
                .map(|c| c.lifted100.length_km)
                .collect::<Vec<_>>(),
        ),
    );
    let elong: Vec<f64> = shaped.iter().map(|c| c.lifted100.elongation()).collect();
    put("belt_elongation_p50", median(&elong));
    put(
        "belt_elongation_over_3_pct",
        if elong.is_empty() {
            f64::NAN
        } else {
            100.0 * elong.iter().filter(|&&e| e >= 3.0).count() as f64 / elong.len() as f64
        },
    );
    let length = pick(&|c| c.end_myr - c.start_myr + d.dt_myr);
    put("collision_length_p50_myr", median(&length));
    put("collision_length_p90_myr", pct(&length, 0.9));
    let peak = pick(&|c| c.peak_uplift_m);
    let (p10, p90) = (pct(&peak, 0.1), pct(&peak, 0.9));
    put(
        "uplift_spread",
        if p10 > 0.0 { p90 / p10 } else { f64::NAN },
    );
    put(
        "collisions_per_100myr",
        if span > 0.0 {
            100.0 * col.len() as f64 / span
        } else {
            f64::NAN
        },
    );
    let age = pick(&|c| c.start_myr - c.contact_onset_myr);
    put("contact_age_p50_myr", median(&age));
    put("contact_age_p90_myr", pct(&age, 0.9));
    put("counted_km_p50", median(&pick(&|c| c.counted_km)));
    let short: Vec<f64> = col
        .iter()
        .filter(|c| c.front_km > 0.0)
        .map(|c| c.shortening_km)
        .collect();
    put("shortening_km_p50", median(&short));
    put("shortening_km_p90", pct(&short, 0.9));

    // Margins and the sea floor.
    put("active_margin_mean", active_margin_mean(margins));
    put(
        "ocean_age_p90_myr",
        d.ages.last().map_or(f64::NAN, |a| a.p90_myr),
    );
    // The sea floor over the run, from the per-step record, after the warm-up (all steps when the
    // run is shorter than it).
    let late: Vec<&StepRec> = d
        .steps
        .iter()
        .filter(|s| s.t - d.t0_myr >= params.warmup_myr && s.ocean > 0)
        .collect();
    let late = if late.is_empty() {
        d.steps.iter().filter(|s| s.ocean > 0).collect()
    } else {
        late
    };
    put(
        "ocean_age_p90_mean_myr",
        mean(late.iter().map(|s| s.age_p90)),
    );
    put("ocean_age_p50_myr", mean(late.iter().map(|s| s.age_p50)));
    let share = |f: fn(&StepRec) -> u32| {
        mean(
            late.iter()
                .map(|s| 100.0 * f64::from(f(s)) / f64::from(s.ocean)),
        )
    };
    put("ocean_old180_pct", share(|s| s.old180));
    put("ocean_old250_pct", share(|s| s.old250));

    // What a force-balance plate model is judged on: speeds against slabs and crust type.
    let all: Vec<&PlateStep> = d.steps.iter().flat_map(|s| s.plates.iter()).collect();
    let speed = |sel: &dyn Fn(&PlateStep) -> bool| -> f64 {
        mean(all.iter().filter(|p| sel(p)).map(|p| p.speed_mm_yr))
    };
    let slab = speed(&|p| p.slab_share() >= 0.2);
    let noslab = speed(&|p| p.slab_share() < 0.2);
    put(
        "speed_p50_mm_yr",
        median(&all.iter().map(|p| p.speed_mm_yr).collect::<Vec<_>>()),
    );
    put("speed_slab_mm_yr", slab);
    put("speed_noslab_mm_yr", noslab);
    put(
        "speed_slab_ratio",
        if noslab > 0.0 {
            slab / noslab
        } else {
            f64::NAN
        },
    );
    let xs: Vec<f64> = all.iter().map(|p| p.slab_share()).collect();
    let ys: Vec<f64> = all.iter().map(|p| p.speed_mm_yr).collect();
    put("speed_slab_corr", pearson(&xs, &ys));
    put("slab_share_mean", mean(xs.iter().copied()));
    put(
        "speed_oceanic_mm_yr",
        speed(&|p| p.continental_share() < 0.5),
    );
    put(
        "speed_continental_mm_yr",
        speed(&|p| p.continental_share() >= 0.5),
    );

    // Task 1b: speeds, rotation changes, the speed guard, continents and mega-plates.
    plate_motion(d, params, &mut put);
    continents(d, params, &mut put);
    megaplates(d, params, &mut put);
    boundary_and_arcs(d, params, &mut put);
    pending_physics(d, params, &mut put);
    out
}

/// The plates the speed measures count: those of `params.min_plate_samples` visible samples or
/// more (a handful of samples is a sliver, and its speed says nothing about the balance).
fn big_plates<'a>(d: &'a Dynamics, params: &Params) -> Vec<&'a PlateStep> {
    d.steps
        .iter()
        .flat_map(|s| s.plates.iter())
        .filter(|p| f64::from(p.visible) >= params.min_plate_samples)
        .collect()
}

fn ratio(a: f64, b: f64) -> f64 {
    if a.is_finite() && b.is_finite() && b > 0.0 {
        a / b
    } else {
        f64::NAN
    }
}

fn median_of(
    plates: &[&PlateStep],
    sel: impl Fn(&PlateStep) -> bool,
    val: impl Fn(&PlateStep) -> f64,
) -> f64 {
    median(
        &plates
            .iter()
            .filter(|p| sel(p))
            .map(|p| val(p))
            .collect::<Vec<_>>(),
    )
}

/// Ranks from 1, ties sharing the mean of their ranks.
fn ranks(x: &[f64]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..x.len()).collect();
    order.sort_by(|&a, &b| x[a].total_cmp(&x[b]));
    let mut r = vec![0.0; x.len()];
    let mut i = 0;
    while i < order.len() {
        let mut j = i;
        while j + 1 < order.len() && x[order[j + 1]] == x[order[i]] {
            j += 1;
        }
        let mid = 0.5 * (i + j) as f64 + 1.0;
        for &k in &order[i..=j] {
            r[k] = mid;
        }
        i = j + 1;
    }
    r
}

/// Spearman's rank correlation (NaN when either series is constant or has under 3 points).
fn spearman(x: &[f64], y: &[f64]) -> f64 {
    pearson(&ranks(x), &ranks(y))
}

/// Speeds of plates and how they move from step to step.
fn plate_motion(d: &Dynamics, params: &Params, put: &mut impl FnMut(&'static str, f64)) {
    // Area-weighted RMS speed of each step's plates, averaged over the steps.
    let rms: Vec<f64> = d
        .steps
        .iter()
        .filter_map(|s| {
            let area: f64 = s.plates.iter().map(|p| f64::from(p.visible)).sum();
            (area > 0.0).then(|| {
                let sq: f64 = s
                    .plates
                    .iter()
                    .map(|p| f64::from(p.visible) * p.speed_mm_yr * p.speed_mm_yr)
                    .sum();
                (sq / area).sqrt()
            })
        })
        .collect();
    put("speed_rms_mm_yr", mean(rms));

    let big = big_plates(d, params);
    let speeds: Vec<f64> = big.iter().map(|p| p.speed_mm_yr).collect();
    put("speed_big_p10_mm_yr", pct(&speeds, 0.1));
    put("speed_big_p50_mm_yr", pct(&speeds, 0.5));
    put("speed_big_p90_mm_yr", pct(&speeds, 0.9));
    put(
        "speed_oceanic_over_continental",
        ratio(
            median_of(&big, |p| p.continental_share() < 0.2, |p| p.speed_mm_yr),
            median_of(&big, |p| p.continental_share() > 0.5, |p| p.speed_mm_yr),
        ),
    );
    put(
        "speed_slab_over_free",
        ratio(
            median_of(&big, |p| p.slab_share() >= 0.2, |p| p.speed_mm_yr),
            median_of(&big, |p| p.slab_share() < 0.05, |p| p.speed_mm_yr),
        ),
    );
    let slab: Vec<f64> = big.iter().map(|p| p.slab_share()).collect();
    put("speed_slab_rank_corr", spearman(&slab, &speeds));
    let area: Vec<f64> = big.iter().map(|p| f64::from(p.visible)).collect();
    put("speed_area_rank_corr", spearman(&area, &speeds));
    let fast = big
        .iter()
        .filter(|p| p.speed_mm_yr > params.fast_mm_yr)
        .count();
    put(
        "speed_over_150_pct",
        if big.is_empty() {
            f64::NAN
        } else {
            100.0 * fast as f64 / big.len() as f64
        },
    );
    put("speed_over_150_plate_steps", fast as f64);

    // The speed guard counts every plate, however small: it is the guard of all of them.
    let guard_steps = d
        .steps
        .iter()
        .filter(|s| s.plates.iter().any(|p| p.guard))
        .count();
    let guard_plate_steps: usize = d
        .steps
        .iter()
        .map(|s| s.plates.iter().filter(|p| p.guard).count())
        .sum();
    put(
        "guard_bind_steps",
        if d.steps.is_empty() {
            f64::NAN
        } else {
            guard_steps as f64
        },
    );
    put("guard_bind_plate_steps", guard_plate_steps as f64);

    let change: Vec<f64> = big
        .iter()
        .filter(|p| p.rot_known)
        .map(|p| 100.0 * p.rot_change)
        .collect();
    put("rot_change_p50_pct", pct(&change, 0.5));
    put("rot_change_p90_pct", pct(&change, 0.9));
    put(
        "spinning_pct",
        if big.is_empty() {
            f64::NAN
        } else {
            100.0 * big.iter().filter(|p| p.spinning).count() as f64 / big.len() as f64
        },
    );
    put(
        "pole_inside_pct",
        if big.is_empty() {
            f64::NAN
        } else {
            100.0 * big.iter().filter(|p| p.pole_inside).count() as f64 / big.len() as f64
        },
    );
}

/// A supercontinent's stay: from the step the largest piece first holds `assemble` of the
/// continental crust to the first step it holds under `disperse` (hysteresis: in between it keeps
/// whatever state it had).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Assembly {
    pub start_myr: f64,
    /// `None` while it still holds at the end of the run (cut off by it).
    pub end_myr: Option<f64>,
}

/// The assemblies in a series of `(time at the end of a step, largest piece's share)`. A run that
/// begins assembled has an assembly starting at `t0_myr`, the time before its first step.
pub fn assemblies(
    shares: &[(f64, f64)],
    t0_myr: f64,
    assemble: f64,
    disperse: f64,
) -> Vec<Assembly> {
    let mut out: Vec<Assembly> = Vec::new();
    let mut open = false;
    for (i, &(t, share)) in shares.iter().enumerate() {
        let share = if share.is_finite() { share } else { 0.0 };
        if !open && share >= assemble {
            out.push(Assembly {
                start_myr: if i == 0 { t0_myr } else { t },
                end_myr: None,
            });
            open = true;
        } else if open && share < disperse {
            if let Some(a) = out.last_mut() {
                a.end_myr = Some(t);
            }
            open = false;
        }
    }
    out
}

/// Connected pieces of continental crust and the supercontinent measures.
fn continents(d: &Dynamics, params: &Params, put: &mut impl FnMut(&'static str, f64)) {
    // Per step: the largest piece's share of the continental crust, and how many pieces hold at
    // least `min_piece_share` of it (of the largest kept: a step keeps `PIECES_KEPT`).
    let series: Vec<(f64, f64, f64)> = d
        .steps
        .iter()
        .filter(|s| s.continental > 0)
        .map(|s| {
            let total = f64::from(s.continental);
            let largest = s.pieces.first().map_or(0.0, |&x| f64::from(x)) / total;
            let count = s
                .pieces
                .iter()
                .filter(|&&x| f64::from(x) >= params.min_piece_share * total)
                .count();
            (s.t, largest, count as f64)
        })
        .collect();
    let shares: Vec<f64> = series.iter().map(|x| x.1).collect();
    put(
        "continent_pieces_p50",
        median(&series.iter().map(|x| x.2).collect::<Vec<_>>()),
    );
    put("continent_largest_share_p50", median(&shares));
    put(
        "continent_largest_share_min",
        shares.iter().copied().fold(f64::NAN, f64::min),
    );

    // A step with no continental crust has a largest share of 0 (nothing is assembled).
    let all: Vec<(f64, f64)> = d
        .steps
        .iter()
        .map(|s| {
            let share = if s.continental > 0 {
                s.pieces.first().map_or(0.0, |&x| f64::from(x)) / f64::from(s.continental)
            } else {
                0.0
            };
            (s.t, share)
        })
        .collect();
    let eps = assemblies(
        &all,
        d.t0_myr,
        params.assembled_share,
        params.dispersed_share,
    );
    if d.steps.is_empty() || d.steps.iter().all(|s| s.pieces.is_empty()) {
        // Not recorded: no data, not "no supercontinent".
        for k in [
            "supercontinent_tenure_myr",
            "supercontinent_period_myr",
            "supercontinent_episodes",
            "supercontinent_time_pct",
        ] {
            put(k, f64::NAN);
        }
        return;
    }
    let tenures: Vec<f64> = eps
        .iter()
        .filter_map(|a| a.end_myr.map(|e| e - a.start_myr))
        .collect();
    put("supercontinent_tenure_myr", median(&tenures));
    let gaps: Vec<f64> = eps
        .windows(2)
        .map(|w| w[1].start_myr - w[0].start_myr)
        .collect();
    put("supercontinent_period_myr", median(&gaps));
    put("supercontinent_episodes", eps.len() as f64);
    let held = all
        .iter()
        .filter(|&&(t, _)| {
            eps.iter()
                .any(|a| t >= a.start_myr && a.end_myr.is_none_or(|e| t < e))
        })
        .count();
    put(
        "supercontinent_time_pct",
        100.0 * held as f64 / all.len() as f64,
    );
}

/// A rift of a plate that was above the mega-plate share.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MegaRift {
    /// The plate that split.
    pub parent: u32,
    /// When it split, Myr.
    pub t_myr: f64,
    /// How long it had been above the share, Myr (from the start of the first step above it, or
    /// of the run, to the rift).
    pub held_myr: f64,
    /// Whether its fragments were all still separate `window_myr` later: none had welded into
    /// another (`None`: the run ends before that). The strict reading.
    pub apart: Option<bool>,
    /// Whether at least two plates still held its crust `window_myr` later: the rift was not
    /// healed whole, though some fragments may have rejoined. The lenient reading.
    pub split: Option<bool>,
}

/// The rifts of plates above `mega_share` of the visible planet, with how long each plate had
/// been above it and whether the rift held. The plate that split is the plate the new plates'
/// `parent` names, above the share at the step before.
pub fn mega_rifts(d: &Dynamics, mega_share: f64, window_myr: f64) -> Vec<MegaRift> {
    let share_at = |i: usize, id: u32| -> f64 {
        let s = &d.steps[i];
        let total: u32 = s.plates.iter().map(|p| p.visible).sum();
        s.plates
            .iter()
            .find(|p| p.id == id)
            .map_or(0.0, |p| f64::from(p.visible) / f64::from(total.max(1)))
    };
    let lives: BTreeMap<u32, &PlateLife> = d.plates.iter().map(|p| (p.id, p)).collect();
    // The plate that holds `id`'s crust at time `t`: follow the welds that had happened by then.
    let holder_at = |mut id: u32, t: f64| -> u32 {
        for _ in 0..10_000 {
            match lives.get(&id) {
                Some(l) if l.died_myr.is_some_and(|died| died <= t) && l.into.is_some() => {
                    id = l.into.unwrap_or(id);
                }
                _ => break,
            }
        }
        id
    };
    let end_t = d.steps.last().map_or(d.t0_myr, |s| s.t);
    let mut out = Vec::new();
    for i in 1..d.steps.len() {
        let t = d.steps[i].t;
        let mut parents: Vec<u32> = d
            .plates
            .iter()
            .filter(|p| p.origin == "rift" && (p.born_myr - t).abs() < 1e-6)
            .filter_map(|p| p.parent)
            .collect();
        parents.sort_unstable();
        parents.dedup();
        for x in parents {
            if share_at(i - 1, x) <= mega_share {
                continue;
            }
            let mut first = i - 1;
            while first > 0 && share_at(first - 1, x) > mega_share {
                first -= 1;
            }
            let held = t - (d.steps[first].t - d.dt_myr);
            let family: Vec<u32> = std::iter::once(x)
                .chain(
                    d.plates
                        .iter()
                        .filter(|p| {
                            p.origin == "rift"
                                && p.parent == Some(x)
                                && (p.born_myr - t).abs() < 1e-6
                        })
                        .map(|p| p.id),
                )
                .collect();
            let apart = (t + window_myr <= end_t + 1e-9).then(|| {
                // Rejoined: a member died within the window into a plate that holds another
                // member's crust.
                !family.iter().any(|&m| {
                    lives.get(&m).is_some_and(|l| {
                        l.died_myr.is_some_and(|died| died <= t + window_myr)
                            && l.into.is_some_and(|into| {
                                let died = l.died_myr.unwrap_or(t);
                                let to = holder_at(into, died);
                                family.iter().any(|&o| o != m && holder_at(o, died) == to)
                            })
                    })
                })
            });
            // Plates that hold the family's crust at the end of the window; a fragment that was
            // subducted away holds nothing.
            let split = (t + window_myr <= end_t + 1e-9).then(|| {
                let at = t + window_myr;
                let holders: BTreeSet<u32> = family
                    .iter()
                    .filter(|&&m| {
                        lives.get(&m).is_none_or(|l| {
                            l.died_myr.is_none_or(|died| died > at) || l.into.is_some()
                        })
                    })
                    .map(|&m| holder_at(m, at))
                    .collect();
                holders.len() >= 2
            });
            out.push(MegaRift {
                parent: x,
                t_myr: t,
                held_myr: held,
                apart,
                split,
            });
        }
    }
    out
}

fn megaplates(d: &Dynamics, params: &Params, put: &mut impl FnMut(&'static str, f64)) {
    if d.steps.is_empty() {
        for k in [
            "megaplate_rifts",
            "megaplate_tenure_p50_myr",
            "megaplate_rift_apart_50_pct",
            "megaplate_rift_split_50_pct",
        ] {
            put(k, f64::NAN);
        }
        return;
    }
    let rifts = mega_rifts(d, params.mega_plate_share, params.rift_window_myr);
    put("megaplate_rifts", rifts.len() as f64);
    put(
        "megaplate_tenure_p50_myr",
        median(&rifts.iter().map(|r| r.held_myr).collect::<Vec<_>>()),
    );
    let share_of = |verdicts: Vec<bool>| {
        if verdicts.is_empty() {
            f64::NAN
        } else {
            100.0 * verdicts.iter().filter(|&&a| a).count() as f64 / verdicts.len() as f64
        }
    };
    put(
        "megaplate_rift_apart_50_pct",
        share_of(rifts.iter().filter_map(|r| r.apart).collect()),
    );
    put(
        "megaplate_rift_split_50_pct",
        share_of(rifts.iter().filter_map(|r| r.split).collect()),
    );
}

/// The steps a boundary or arc measure covers: those after `params.warmup_myr` of the run (all
/// when the run is shorter), and those of its first `params.early_myr`.
fn windows<'a>(d: &'a Dynamics, params: &Params) -> (Vec<&'a StepRec>, Vec<&'a StepRec>) {
    let late: Vec<&StepRec> = d
        .steps
        .iter()
        .filter(|s| s.t - d.t0_myr >= params.warmup_myr)
        .collect();
    let late = if late.is_empty() {
        d.steps.iter().collect()
    } else {
        late
    };
    let early = d
        .steps
        .iter()
        .filter(|s| s.t - d.t0_myr <= params.early_myr + 1e-9)
        .collect();
    (late, early)
}

/// The boundary mix of `steps` (steps with no boundary recorded are skipped): the mean over the
/// steps of the subducting, spreading, collision and other shares of the boundary's length, and
/// the ocean-ocean share of the subducting length summed over the steps (NaN where there is none).
/// Returns `[subduction %, spreading %, collision %, other %, ocean-ocean % of subduction]`.
pub fn boundary_shares(steps: &[&StepRec]) -> [f64; 5] {
    use super::dynamics::mix::*;
    let mut shares: [Vec<f64>; 4] = Default::default();
    let (mut oo, mut sub) = (0.0, 0.0);
    for s in steps {
        let m = &s.mix_km;
        let total: f64 = m.iter().sum();
        if total <= 0.0 {
            continue;
        }
        let pct = |x: f64| 100.0 * x / total;
        shares[0].push(pct(m[OCEAN_OCEAN] + m[OCEAN_CONTINENT]));
        shares[1].push(pct(m[RIDGE] + m[RIFT]));
        shares[2].push(pct(m[COLLISION]));
        shares[3].push(pct(m[OTHER]));
        oo += m[OCEAN_OCEAN];
        sub += m[OCEAN_OCEAN] + m[OCEAN_CONTINENT];
    }
    let [a, b, c, e] = shares.map(mean);
    [
        a,
        b,
        c,
        e,
        if sub > 0.0 {
            100.0 * oo / sub
        } else {
            f64::NAN
        },
    ]
}

/// The arcs of `steps`: for the oceanic or continental arc, the share of its cells above the sea
/// datum (cells summed over the steps), and the cell-weighted mean over the steps of its median
/// and 90th percentile elevation (m), and its mean cells per step. NaN where no step has a cell.
/// Returns `[emergent %, p50, p90, cells per step]`.
pub fn arc_summary(
    steps: &[&StepRec],
    pick: impl Fn(&StepRec) -> super::dynamics::ArcStat,
) -> [f64; 4] {
    let (mut cells, mut emergent, mut p50, mut p90) = (0.0, 0.0, 0.0, 0.0);
    for s in steps {
        let a = pick(s);
        let n = f64::from(a.cells);
        cells += n;
        emergent += f64::from(a.emergent);
        p50 += n * a.p50_m;
        p90 += n * a.p90_m;
    }
    if cells <= 0.0 {
        return [f64::NAN; 4];
    }
    [
        100.0 * emergent / cells,
        p50 / cells,
        p90 / cells,
        cells / steps.len() as f64,
    ]
}

/// The boundary mix and the arc heights (dynamics version 3).
fn boundary_and_arcs(d: &Dynamics, params: &Params, put: &mut impl FnMut(&'static str, f64)) {
    let (late, early) = windows(d, params);
    let [sub, spread, collision, other, oo] = boundary_shares(&late);
    put("boundary_subduction_pct", sub);
    put("boundary_spreading_pct", spread);
    put("boundary_collision_pct", collision);
    put("boundary_other_pct", other);
    put("boundary_oo_of_subduction_pct", oo);
    // Mean per step over the steps that have a boundary recorded.
    let km = |f: fn(&[f64; MIX_CLASSES]) -> f64| {
        mean(
            late.iter()
                .filter(|s| s.mix_km.iter().sum::<f64>() > 0.0)
                .map(|s| f(&s.mix_km)),
        )
    };
    put("boundary_total_km", km(|m| m.iter().sum()));
    put(
        "boundary_subduction_km",
        km(|m| m[super::dynamics::mix::OCEAN_OCEAN] + m[super::dynamics::mix::OCEAN_CONTINENT]),
    );
    let [sub, spread, collision, other, oo] = boundary_shares(&early);
    put("boundary_subduction_pct_early", sub);
    put("boundary_spreading_pct_early", spread);
    put("boundary_collision_pct_early", collision);
    put("boundary_other_pct_early", other);
    put("boundary_oo_of_subduction_pct_early", oo);

    let [emergent, p50, p90, cells] = arc_summary(&late, |s| s.arc_oceanic);
    put("arc_oceanic_emergent_pct", emergent);
    put("arc_oceanic_p50_m", p50);
    put("arc_oceanic_p90_m", p90);
    put("arc_oceanic_cells_mean", cells);
    let [_, p50, p90, cells] = arc_summary(&late, |s| s.arc_continental);
    put("arc_continental_p50_m", p50);
    put("arc_continental_p90_m", p90);
    put("arc_continental_cells_mean", cells);
    let [emergent, _, p90, _] = arc_summary(&early, |s| s.arc_oceanic);
    put("arc_oceanic_emergent_pct_early", emergent);
    put("arc_oceanic_p90_m_early", p90);
    let [_, _, p90, _] = arc_summary(&early, |s| s.arc_continental);
    put("arc_continental_p90_m_early", p90);
}

/// The measures that wait for physics: read from the record's empty-for-now fields, so the task
/// that fills them lights the rows up.
fn pending_physics(d: &Dynamics, params: &Params, put: &mut impl FnMut(&'static str, f64)) {
    let belts: &[BeltRec] = &d.belts;
    let col =
        |f: &dyn Fn(&BeltRec) -> Option<f64>| -> Vec<f64> { belts.iter().filter_map(f).collect() };
    let ratios: Vec<f64> = belts
        .iter()
        .filter_map(
            |b| match (b.convergence_onset_mm_yr, b.convergence_20myr_mm_yr) {
                (Some(a), Some(c)) if a > 0.0 => Some(c / a),
                _ => None,
            },
        )
        .collect();
    put("collision_convergence_ratio_20myr", median(&ratios));
    put("wedge_width_km_p50", median(&col(&|b| b.wedge_width_km)));
    put(
        "foredeep_width_km_p50",
        median(&col(&|b| b.foredeep_width_km)),
    );
    put(
        "foredeep_depth_m_p50",
        median(&col(&|b| b.foredeep_depth_m)),
    );

    let pull_share: Vec<f64> = d
        .steps
        .iter()
        .flat_map(|s| {
            s.forces.iter().filter(|f| {
                s.plates
                    .iter()
                    .find(|p| p.id == f.id)
                    .is_some_and(|p| f64::from(p.visible) >= params.min_plate_samples)
            })
        })
        .filter_map(PlateForces::pull_share)
        .collect();
    put("force_slab_pull_share_p50", median(&pull_share));
    // The stress of a plate above the mega-plate share: the plates' forces are matched to their
    // size in the same step.
    let mut stress = Vec::new();
    for s in &d.steps {
        let total: u32 = s.plates.iter().map(|p| p.visible).sum();
        for f in &s.forces {
            let big = s.plates.iter().find(|p| p.id == f.id).is_some_and(|p| {
                f64::from(p.visible) / f64::from(total.max(1)) > params.mega_plate_share
            });
            if big {
                stress.push(f.stress_mpa);
            }
        }
    }
    put("megaplate_stress_mpa_p50", median(&stress));
    put(
        "megaplate_tension_pct",
        if stress.is_empty() {
            f64::NAN
        } else {
            100.0 * stress.iter().filter(|&&x| x > 0.0).count() as f64 / stress.len() as f64
        },
    );
    let slabs: Vec<f64> = d
        .steps
        .iter()
        .flat_map(|s| s.slabs_km.iter().copied())
        .collect();
    put("slab_length_km_p50", median(&slabs));
    let span = d.steps.last().map_or(0.0, |s| s.t) - d.t0_myr;
    let breaks: f64 = d.steps.iter().map(|s| f64::from(s.breakoffs)).sum();
    let recorded = d.steps.iter().any(|s| !s.forces.is_empty());
    put(
        "slab_breakoffs_per_100myr",
        if recorded && span > 0.0 {
            100.0 * breaks / span
        } else {
            f64::NAN
        },
    );
    balance(d, params, put);
}

/// The force balance's answer (recorded, not applied) against today's motion: the plates of
/// `params.min_plate_samples` or more, each balance answer matched with the same plate's record
/// in the same step.
fn balance(d: &Dynamics, params: &Params, put: &mut impl FnMut(&'static str, f64)) {
    let mut pairs: Vec<(&PlateStep, &PlateForces)> = Vec::new();
    let mut rms = Vec::new();
    let mut norm_speeds = Vec::new();
    for s in &d.steps {
        if s.forces.is_empty() {
            continue;
        }
        let (mut sq, mut area) = (0.0, 0.0);
        let mut step_pairs = Vec::new();
        for f in &s.forces {
            let Some(p) = s.plates.iter().find(|p| p.id == f.id) else {
                continue;
            };
            let a = f64::from(p.visible);
            sq += a * f.rms_mm_yr * f.rms_mm_yr;
            area += a;
            if a >= params.min_plate_samples {
                step_pairs.push((p, f));
            }
        }
        if area > 0.0 && sq > 0.0 {
            let r = (sq / area).sqrt();
            rms.push(r);
            let scale = params.balance_rms_mm_yr / r;
            norm_speeds.extend(step_pairs.iter().map(|(_, f)| f.speed_mm_yr * scale));
        }
        pairs.extend(step_pairs);
    }
    put("balance_rms_mm_yr", mean(rms));
    let speeds: Vec<f64> = pairs.iter().map(|(_, f)| f.speed_mm_yr).collect();
    put("balance_p10_mm_yr", pct(&speeds, 0.1));
    put("balance_p50_mm_yr", pct(&speeds, 0.5));
    put("balance_p90_mm_yr", pct(&speeds, 0.9));
    put("balance_norm_p10_mm_yr", pct(&norm_speeds, 0.1));
    put("balance_norm_p50_mm_yr", pct(&norm_speeds, 0.5));
    put("balance_norm_p90_mm_yr", pct(&norm_speeds, 0.9));
    let share_over = |limit: f64| {
        if speeds.is_empty() {
            f64::NAN
        } else {
            100.0 * speeds.iter().filter(|&&v| v > limit).count() as f64 / speeds.len() as f64
        }
    };
    put("balance_over_150_pct", share_over(params.fast_mm_yr));
    put("balance_over_250_pct", share_over(params.balance_guard_mm_yr));
    let med = |sel: &dyn Fn(&PlateStep) -> bool| {
        median(
            &pairs
                .iter()
                .filter(|(p, _)| sel(p))
                .map(|(_, f)| f.speed_mm_yr)
                .collect::<Vec<_>>(),
        )
    };
    put(
        "balance_oceanic_over_continental",
        ratio(
            med(&|p| p.continental_share() < 0.2),
            med(&|p| p.continental_share() > 0.5),
        ),
    );
    put(
        "balance_slab_over_free",
        ratio(
            med(&|p| p.slab_share() >= 0.2),
            med(&|p| p.slab_share() < 0.05),
        ),
    );
    let slab: Vec<f64> = pairs.iter().map(|(p, _)| p.slab_share()).collect();
    put("balance_slab_rank_corr", spearman(&slab, &speeds));
    let area: Vec<f64> = pairs.iter().map(|(p, _)| f64::from(p.visible)).collect();
    put("balance_area_rank_corr", spearman(&area, &speeds));
    let change: Vec<f64> = pairs
        .iter()
        .filter(|(_, f)| f.rot_known)
        .map(|(_, f)| 100.0 * f.rot_change)
        .collect();
    put("balance_rot_change_p50_pct", pct(&change, 0.5));
    put("balance_rot_change_p90_pct", pct(&change, 0.9));
    let today: Vec<f64> = pairs.iter().map(|(p, _)| p.speed_mm_yr).collect();
    put("balance_vs_motion_rank_corr", spearman(&today, &speeds));
    let over: Vec<f64> = pairs
        .iter()
        .filter(|(p, _)| p.speed_mm_yr > 0.0)
        .map(|(p, f)| f.speed_mm_yr / p.speed_mm_yr)
        .collect();
    put("balance_over_motion_p50", median(&over));
}

/// A run's data for the check.
#[derive(Clone, Debug)]
pub struct RunData {
    pub name: String,
    pub dynamics: Dynamics,
    pub margins: Vec<Margins>,
}

/// The verdict on one measure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    /// The runs could not give the measure, which has a range: counts as a failure.
    NoData,
    /// No range: reported only.
    Reported,
    /// The measure waits for physics not built yet and the record that feeds it is empty: it
    /// reports "no data" and does not fail.
    Pending,
}

impl Status {
    pub fn word(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Fail => "FAIL",
            Status::NoData => "NO DATA",
            Status::Reported => "-",
            Status::Pending => "no data",
        }
    }

    /// Whether this verdict fails the check.
    pub fn failed(self) -> bool {
        matches!(self, Status::Fail | Status::NoData)
    }
}

/// One line of the check's table.
#[derive(Clone, Debug)]
pub struct Row {
    pub def: MeasureDef,
    pub lo: Option<f64>,
    pub hi: Option<f64>,
    /// The mean over the runs that gave a value (NaN when none did).
    pub value: f64,
    /// Each run's value.
    pub runs: Vec<f64>,
    pub status: Status,
}

/// Judges `value` against a range (both ends included).
pub fn judge(value: f64, lo: Option<f64>, hi: Option<f64>) -> Status {
    if lo.is_none() && hi.is_none() {
        return Status::Reported;
    }
    if !value.is_finite() {
        return Status::NoData;
    }
    if lo.is_some_and(|l| value < l) || hi.is_some_and(|h| value > h) {
        Status::Fail
    } else {
        Status::Pass
    }
}

/// The check: every measure's mean over `runs` against `targets`.
pub fn evaluate(runs: &[RunData], targets: &Targets) -> Vec<Row> {
    let per_run: Vec<BTreeMap<&'static str, f64>> = runs
        .iter()
        .map(|r| measures_with(&r.dynamics, &r.margins, &targets.params))
        .collect();
    MEASURES
        .iter()
        .map(|def| {
            let values: Vec<f64> = per_run
                .iter()
                .map(|m| m.get(def.key).copied().unwrap_or(f64::NAN))
                .collect();
            let value = mean(values.iter().copied());
            let (lo, hi) = targets.range(def.key);
            Row {
                def: *def,
                lo,
                hi,
                value,
                runs: values,
                status: if def.pending.is_some() && !value.is_finite() {
                    Status::Pending
                } else {
                    judge(value, lo, hi)
                },
            }
        })
        .collect()
}

/// Whether any row fails.
pub fn failed(rows: &[Row]) -> bool {
    rows.iter().any(|r| r.status.failed())
}

fn num(x: f64) -> String {
    if !x.is_finite() {
        "n/a".to_owned()
    } else if x != 0.0 && x.abs() < 0.1 {
        format!("{x:.3}")
    } else if x.abs() < 10.0 {
        format!("{x:.2}")
    } else if x.abs() < 100.0 {
        format!("{x:.1}")
    } else {
        format!("{x:.0}")
    }
}

/// The range as the table prints it.
pub fn range_text(lo: Option<f64>, hi: Option<f64>) -> String {
    match (lo, hi) {
        (Some(l), Some(h)) => format!("{}-{}", num(l), num(h)),
        (Some(l), None) => format!("at least {}", num(l)),
        (None, Some(h)) => format!("at most {}", num(h)),
        (None, None) => "-".to_owned(),
    }
}

/// The table: measure, value, target range, pass or fail. With several runs the value is the mean
/// and a column lists the runs.
pub fn render(rows: &[Row]) -> String {
    let many = rows.first().is_some_and(|r| r.runs.len() > 1);
    let w0 = rows
        .iter()
        .map(|r| r.def.label.chars().count())
        .max()
        .unwrap_or(0);
    let mut s = String::new();
    let mut head = format!(
        "{:<w0$}  {:>9}  {:<14}  {:<8}",
        "measure", "value", "target", "result"
    );
    if many {
        head.push_str("  runs");
    }
    s.push_str(&head);
    s.push('\n');
    for r in rows {
        let label = format!("{} ({})", r.def.label, r.def.unit);
        let mut line = format!(
            "{:<w0$}  {:>9}  {:<14}  {:<8}",
            label,
            num(r.value),
            range_text(r.lo, r.hi),
            r.status.word()
        );
        if many {
            line.push_str("  ");
            line.push_str(&r.runs.iter().map(|&x| num(x)).collect::<Vec<_>>().join(" "));
        }
        s.push_str(&line);
        s.push('\n');
    }
    s
}

/// The rows as JSON (the check's `--json` file).
pub fn to_json(rows: &[Row], names: &[String], params: &Params) -> Value {
    let any_failed = failed(rows);
    let measures: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "key": r.def.key,
                "label": r.def.label,
                "unit": r.def.unit,
                "value": r.value.is_finite().then_some(r.value),
                "lo": r.lo,
                "hi": r.hi,
                "status": r.status.word(),
                "runs": r.runs.iter().map(|&x| x.is_finite().then_some(x)).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({
        "runs": names,
        "failed": any_failed,
        "params": params.describe(),
        "measures": measures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plates::dynamics::{AgeRec, CollisionRec, Footprint, PlateForces};

    const N: u32 = 1000;

    fn plate(
        id: u32,
        visible: u32,
        continental: u32,
        speed: f64,
        front: f64,
        bnd: f64,
    ) -> PlateStep {
        PlateStep {
            id,
            visible,
            continental,
            speed_mm_yr: speed,
            front_km: front,
            boundary_km: bnd,
            ..PlateStep::default()
        }
    }

    /// A step at `t` with plates of the given visible sample counts.
    fn step(t: f64, land: u32, continental: u32, sizes: &[u32]) -> StepRec {
        let plates: Vec<PlateStep> = sizes
            .iter()
            .enumerate()
            .map(|(i, &v)| plate(i as u32, v, 0, 40.0, 0.0, 1000.0))
            .collect();
        StepRec {
            t,
            visible: sizes.iter().sum(),
            land,
            continental,
            plates,
            ..StepRec::default()
        }
    }

    fn dynamics(steps: Vec<StepRec>) -> Dynamics {
        Dynamics {
            version: 1,
            t0_myr: steps.first().map_or(0.0, |s| s.t - 2.0),
            dt_myr: 2.0,
            samples: N,
            steps,
            ..Dynamics::default()
        }
    }

    fn life(id: u32, parent: Option<u32>, origin: &str, born: f64) -> PlateLife {
        PlateLife {
            id,
            parent,
            origin: origin.to_owned(),
            born_myr: born,
            ..PlateLife::default()
        }
    }

    fn dies(mut l: PlateLife, t: f64, fate: &str, into: Option<u32>) -> PlateLife {
        l.died_myr = Some(t);
        l.fate = Some(fate.to_owned());
        l.into = into;
        l
    }

    fn fp(samples: u32, length: f64, width: f64) -> Footprint {
        Footprint {
            samples,
            length_km: length,
            width_km: width,
        }
    }

    fn collision(start: f64, end: f64, peak: f64) -> CollisionRec {
        CollisionRec {
            collision: true,
            start_myr: start,
            end_myr: end,
            contact_onset_myr: start - 8.0,
            peak_uplift_m: peak,
            frac: 0.3,
            terrane_samples: 30,
            under_samples: 100,
            over_samples: 200,
            ..CollisionRec::default()
        }
    }

    fn get(m: &BTreeMap<&'static str, f64>, k: &str) -> f64 {
        *m.get(k).unwrap_or_else(|| panic!("no measure {k}"))
    }

    #[test]
    fn every_measure_is_computed_and_an_empty_run_fails() {
        let keys: BTreeSet<&str> = MEASURES.iter().map(|d| d.key).collect();
        assert_eq!(keys.len(), MEASURES.len(), "duplicate keys");
        let m = measures(&Dynamics::default(), &[]);
        for k in m.keys() {
            assert!(keys.contains(k), "{k} is computed but not in MEASURES");
        }
        for d in MEASURES {
            assert!(m.contains_key(d.key), "{} is never computed", d.key);
        }
        // Nothing recorded: a ranged measure has no data and fails; counts are zero.
        let rows = evaluate(
            &[RunData {
                name: "empty".into(),
                dynamics: Dynamics::default(),
                margins: vec![],
            }],
            &Targets::builtin(),
        );
        assert!(failed(&rows));
        assert!(rows.iter().all(|r| r.status != Status::Pass));
    }

    #[test]
    fn plate_sizes() {
        let d = dynamics(vec![
            step(2.0, 0, 0, &[600, 300, 100]),
            step(4.0, 0, 0, &[400, 400, 200]),
            step(6.0, 0, 0, &[560, 440]),
            step(8.0, 0, 0, &[800, 100, 50, 50]),
        ]);
        let m = measures(&d, &[]);
        // Largest shares 0.6, 0.4, 0.56, 0.8: median 0.58; three of four steps above 50 %, all
        // above 30 %.
        assert!((get(&m, "largest_share_p50") - 0.58).abs() < 1e-9);
        assert!((get(&m, "largest_share_max") - 0.8).abs() < 1e-9);
        assert_eq!(get(&m, "largest_over_50_pct"), 75.0);
        assert_eq!(get(&m, "largest_over_30_pct"), 100.0);
        // Plates of at least 5 %: 3, 3, 2, 4 (50 of 1000 is 5 %).
        assert_eq!(get(&m, "plates_over_5pct_p50"), 3.0);
        assert_eq!(get(&m, "plates_alive_p50"), 3.0);
    }

    #[test]
    fn a_split_megaplate_grows_back() {
        // Above half until 6, a rift at 8 splits it, above half again at 14 (6 Myr later) and
        // split once more at 20 (never above half again, which is not counted).
        let mut d = dynamics(vec![
            step(2.0, 0, 0, &[800, 200]),
            step(4.0, 0, 0, &[600, 400]),
            step(6.0, 0, 0, &[600, 400]),
            step(8.0, 0, 0, &[400, 300, 300]),
            step(10.0, 0, 0, &[450, 300, 250]),
            step(12.0, 0, 0, &[480, 300, 220]),
            step(14.0, 0, 0, &[520, 300, 180]),
            step(16.0, 0, 0, &[600, 400]),
            step(18.0, 0, 0, &[600, 400]),
            step(20.0, 0, 0, &[400, 300, 300]),
        ]);
        d.plates = vec![
            life(2, Some(0), "rift", 8.0),
            life(3, Some(0), "rift", 20.0),
            // A weld that happens to drop the largest share is not a rift.
            life(4, Some(0), "subduction_initiation", 12.0),
        ];
        let m = measures(&d, &[]);
        assert_eq!(get(&m, "megaplate_regrow_myr_p50"), 6.0);
    }

    #[test]
    fn lifetimes_and_fates() {
        let mut d = dynamics(vec![step(2.0, 0, 0, &[500, 500])]);
        d.plates = vec![
            dies(life(0, None, "initial", 0.0), 100.0, "sutured", Some(1)),
            dies(life(2, Some(1), "rift", 10.0), 20.0, "consumed", None),
            dies(life(3, Some(1), "rift", 30.0), 40.0, "absorbed", Some(1)),
            dies(life(4, Some(1), "rift", 10.0), 200.0, "sutured", Some(1)),
            life(1, None, "initial", 0.0),
        ];
        let m = measures(&d, &[]);
        // Lives of the four that died: 100, 10, 10, 190.
        assert_eq!(get(&m, "plate_life_p50_myr"), 55.0);
        assert_eq!(get(&m, "dead_within_20_pct"), 50.0);
        assert_eq!(get(&m, "rift_life_p50_myr"), 10.0);
        assert_eq!(get(&m, "deaths_sutured"), 2.0);
        assert_eq!(get(&m, "deaths_absorbed"), 1.0);
        assert_eq!(get(&m, "deaths_consumed"), 1.0);
    }

    #[test]
    fn rift_born_plates_that_survive_or_are_welded_back() {
        let mut d = dynamics(vec![
            step(2.0, 0, 0, &[500, 500]),
            step(300.0, 0, 0, &[500, 500]),
        ]);
        d.plates = vec![
            life(0, None, "initial", 0.0),
            // A rift of plate 0 at 10 Myr: 1 goes back to its parent (after 15 Myr), 2 to a
            // stranger.
            dies(life(1, Some(0), "rift", 10.0), 25.0, "sutured", Some(0)),
            dies(life(2, Some(0), "rift", 10.0), 30.0, "sutured", Some(9)),
            // A rift at 50 Myr: 5 is welded to its sibling 6 after 20 Myr.
            dies(life(5, Some(0), "rift", 50.0), 70.0, "sutured", Some(6)),
            life(6, Some(0), "rift", 50.0),
            // One that lives on; one too young to judge (born within 50 Myr of the end).
            life(4, Some(0), "rift", 20.0),
            life(7, Some(0), "rift", 290.0),
            // Welded back to its parent, but only after 80 Myr: not "within 50 Myr".
            dies(life(8, Some(0), "rift", 100.0), 180.0, "sutured", Some(0)),
            life(9, None, "initial", 0.0),
        ];
        let m = measures(&d, &[]);
        // Judged (born at least 50 Myr before 300): 1, 2, 4, 5, 6, 8. Alive after 50 Myr: 4, 6, 8
        // (8 lived 80).
        assert!((get(&m, "rift_alive_50_pct") - 50.0).abs() < 1e-9);
        // Welded back within 50 Myr: 1 (parent) and 5 (sibling).
        assert!((get(&m, "rift_welded_back_50_pct") - 100.0 * 2.0 / 6.0).abs() < 1e-9);
    }

    #[test]
    fn a_weld_follows_the_plate_that_holds_the_parents_crust() {
        // Plate 0 is welded into 3; plate 1, rifted from 0, is then welded into 3: back to the
        // plate that now holds its parent's crust.
        let mut d = dynamics(vec![step(300.0, 0, 0, &[500, 500])]);
        d.plates = vec![
            dies(life(0, None, "initial", 0.0), 5.0, "sutured", Some(3)),
            dies(life(1, Some(0), "rift", 0.0), 30.0, "sutured", Some(3)),
            life(3, None, "initial", 0.0),
        ];
        let m = measures(&d, &[]);
        assert_eq!(get(&m, "rift_welded_back_50_pct"), 100.0);
    }

    #[test]
    fn land_gain_leaves_out_the_start_up() {
        let d = dynamics(vec![
            step(2.0, 100, 400, &[1000]),
            // The start-up: +50 samples (5 % of the planet) in the first 10 Myr.
            step(4.0, 150, 400, &[1000]),
            step(14.0, 150, 380, &[1000]),
            // After it: +12 samples = 1.2 %, then a loss.
            step(16.0, 162, 390, &[1000]),
            step(18.0, 120, 440, &[1000]),
        ]);
        let m = measures(&d, &[]);
        assert!((get(&m, "land_gain_max_pct") - 1.2).abs() < 1e-9);
        assert!((get(&m, "continental_min") - 0.38).abs() < 1e-9);
        assert!((get(&m, "continental_max") - 0.44).abs() < 1e-9);
        assert!((get(&m, "continental_end") - 0.44).abs() < 1e-9);
        assert!((get(&m, "land_share_p50") - 0.15).abs() < 1e-9);
    }

    #[test]
    fn collisions() {
        let mut d = dynamics(vec![step(2.0, 0, 0, &[1000]), step(1000.0, 0, 0, &[1000])]);
        d.t0_myr = 0.0;
        let mut a = collision(100.0, 100.0, 1400.0);
        a.emerged = fp(20, 500.0, 300.0);
        a.lifted10 = fp(250, 4000.0, 2000.0);
        a.lifted100 = fp(100, 3000.0, 1000.0);
        a.counted_km = 350.0;
        a.front_km = 600.0;
        a.shortening_km = 90.0;
        let mut b = collision(300.0, 330.0, 2800.0);
        b.emerged = fp(3, 100.0, 50.0);
        b.lifted10 = fp(80, 4000.0, 800.0);
        b.lifted100 = fp(40, 4000.0, 500.0);
        b.front_km = 1000.0;
        b.shortening_km = 200.0;
        let mut c = collision(600.0, 680.0, 700.0);
        c.lifted100 = fp(60, 5000.0, 800.0);
        c.lifted10 = fp(90, 5000.0, 1000.0);
        let mut transfer = collision(700.0, 700.0, 1400.0);
        transfer.collision = false;
        transfer.emerged = fp(400, 1000.0, 1000.0);
        d.collisions = vec![a, b, c, transfer];
        let m = measures(&d, &[]);
        // Only collisions count, not terrane transfers: 20 samples of 1000 = 2 %.
        assert!((get(&m, "collision_land_max_pct") - 2.0).abs() < 1e-9);
        assert!((get(&m, "collision_lifted10_pct_max") - 25.0).abs() < 1e-9);
        assert!((get(&m, "collision_lifted10_pct_p50") - 9.0).abs() < 1e-9);
        assert_eq!(get(&m, "belt_width_km_p50"), 800.0);
        assert_eq!(get(&m, "belt_length_km_p50"), 4000.0);
        // Elongations 3, 8, 6.25.
        assert_eq!(get(&m, "belt_elongation_p50"), 6.25);
        assert!((get(&m, "belt_elongation_over_3_pct") - 100.0).abs() < 1e-9);
        // Lengths: a one-step collision is one step (2 Myr); 32 and 82.
        assert_eq!(get(&m, "collision_length_p50_myr"), 32.0);
        assert!((get(&m, "collision_length_p90_myr") - 72.0).abs() < 1e-9);
        // Peak uplifts 700, 1400, 2800: 90th ÷ 10th percentile.
        assert!((get(&m, "uplift_spread") - 2520.0 / 840.0).abs() < 1e-9);
        // Three collisions in 1000 Myr.
        assert!((get(&m, "collisions_per_100myr") - 0.3).abs() < 1e-9);
        // Contact age 8 for all; the shortening of the two that have a front.
        assert_eq!(get(&m, "contact_age_p50_myr"), 8.0);
        assert_eq!(get(&m, "shortening_km_p50"), 145.0);
        assert_eq!(get(&m, "counted_km_p50"), 0.0);
    }

    #[test]
    fn welds_report_which_plate_was_larger() {
        let mut d = dynamics(vec![step(2.0, 0, 0, &[1000])]);
        let mut big = collision(10.0, 10.0, 1.0);
        big.sutured = true;
        let small = big.clone();
        d.collisions = vec![big, small];
        // The plate that dies has 100 samples against the receiver's 200 in both.
        let m = measures(&d, &[]);
        assert_eq!(get(&m, "welded_dying_larger_pct"), 0.0);
        d.collisions[0].under_samples = 900;
        let m = measures(&d, &[]);
        assert_eq!(get(&m, "welded_dying_larger_pct"), 50.0);
    }

    #[test]
    fn the_sea_floor_and_the_margins() {
        let mut d = dynamics(vec![step(2.0, 0, 0, &[1000])]);
        let age = |t: f64, p90: f64| AgeRec {
            t,
            samples: 100,
            p90_myr: p90,
            ..AgeRec::default()
        };
        d.ages = vec![age(10.0, 100.0), age(120.0, 180.0), age(200.0, 220.0)];
        // The per-step record: the mean over the run leaves out the first 100 Myr.
        d.t0_myr = 0.0;
        d.steps = [(50.0, 300.0), (120.0, 180.0), (200.0, 220.0)]
            .iter()
            .map(|&(t, p90)| StepRec {
                ocean: 100,
                age_p90: p90,
                ..step(t, 0, 0, &[1000])
            })
            .collect();
        let margin = |t: f64, subduction: u32, collision: u32| Margins {
            t_myr: t,
            coast: 100,
            subduction,
            collision,
            ..Margins::default()
        };
        // The records less than 90 Myr after the first are left out: 10, 50 | 100, 150.
        let margins = [
            margin(10.0, 60, 0),
            margin(50.0, 50, 0),
            margin(100.0, 20, 5),
            margin(150.0, 30, 0),
        ];
        let m = measures(&d, &margins);
        assert_eq!(get(&m, "ocean_age_p90_myr"), 220.0);
        assert_eq!(get(&m, "ocean_age_p90_mean_myr"), 200.0);
        assert!((get(&m, "active_margin_mean") - 0.275).abs() < 1e-9);
        // A short run uses all its records.
        assert!((active_margin_mean(&margins[..2]) - 0.55).abs() < 1e-9);
        assert!(active_margin_mean(&[]).is_nan());
    }

    #[test]
    fn plate_motion_against_slabs_and_crust() {
        // Slab-attached plates fast, free plates slow, the continental ones slowest.
        let mk = |t: f64| StepRec {
            t,
            visible: 1000,
            plates: vec![
                plate(0, 400, 0, 80.0, 500.0, 1000.0),
                plate(1, 300, 0, 70.0, 400.0, 1000.0),
                plate(2, 200, 150, 30.0, 0.0, 1000.0),
                plate(3, 100, 80, 20.0, 0.0, 500.0),
            ],
            ..StepRec::default()
        };
        let d = dynamics(vec![mk(2.0), mk(4.0), mk(6.0)]);
        let m = measures(&d, &[]);
        assert_eq!(get(&m, "speed_slab_mm_yr"), 75.0);
        assert_eq!(get(&m, "speed_noslab_mm_yr"), 25.0);
        assert_eq!(get(&m, "speed_slab_ratio"), 3.0);
        assert!(get(&m, "speed_slab_corr") > 0.9);
        assert!((get(&m, "slab_share_mean") - 0.225).abs() < 1e-9);
        assert_eq!(get(&m, "speed_oceanic_mm_yr"), 75.0);
        assert_eq!(get(&m, "speed_continental_mm_yr"), 25.0);
        assert_eq!(get(&m, "speed_p50_mm_yr"), 50.0);
    }

    #[test]
    fn judging_is_inclusive_and_a_missing_value_fails() {
        assert_eq!(judge(5.0, Some(5.0), Some(9.0)), Status::Pass);
        assert_eq!(judge(9.0, Some(5.0), Some(9.0)), Status::Pass);
        assert_eq!(judge(4.99, Some(5.0), Some(9.0)), Status::Fail);
        assert_eq!(judge(9.01, Some(5.0), Some(9.0)), Status::Fail);
        assert_eq!(judge(100.0, Some(5.0), None), Status::Pass);
        assert_eq!(judge(100.0, None, Some(5.0)), Status::Fail);
        assert_eq!(judge(f64::NAN, Some(5.0), None), Status::NoData);
        assert!(Status::NoData.failed() && Status::Fail.failed());
        assert!(!Status::Pass.failed() && !Status::Reported.failed());
        assert_eq!(judge(f64::NAN, None, None), Status::Reported);
    }

    fn run_with(sizes: &[u32]) -> RunData {
        RunData {
            name: "r".into(),
            dynamics: dynamics(vec![step(2.0, 0, 0, sizes), step(4.0, 0, 0, sizes)]),
            margins: vec![],
        }
    }

    fn row<'a>(rows: &'a [Row], key: &str) -> &'a Row {
        rows.iter().find(|r| r.def.key == key).unwrap()
    }

    #[test]
    fn the_check_takes_the_mean_of_the_runs_and_fails_outside_the_range() {
        // Largest shares 0.2 and 0.4: mean 0.3.
        let rows = evaluate(
            &[
                run_with(&[200, 200, 200, 200, 200]),
                run_with(&[400, 300, 300]),
            ],
            &Targets::builtin(),
        );
        let r = row(&rows, "largest_share_p50");
        assert_eq!(r.runs.len(), 2);
        assert!((r.value - 0.3).abs() < 1e-9, "{}", r.value);
        assert_eq!(r.status, Status::Pass);
        assert!(failed(&rows), "the other measures have no data here");
        // 0.6 alone is outside.
        let rows = evaluate(&[run_with(&[600, 400])], &Targets::builtin());
        assert_eq!(row(&rows, "largest_share_p50").status, Status::Fail);
        let text = render(&rows);
        assert!(text.contains("FAIL") && text.contains("0.15-0.35"));
        assert_eq!(text.lines().count(), MEASURES.len() + 1);
        let v = to_json(&rows, &["r".to_owned()], &Params::default());
        assert_eq!(v["failed"], true);
        assert_eq!(v["measures"].as_array().unwrap().len(), MEASURES.len());
    }

    #[test]
    fn targets_can_be_moved_from_a_file() {
        let mut t = Targets::builtin();
        assert_eq!(t.range("largest_share_p50"), (Some(0.15), Some(0.35)));
        t.apply_json(r#"{"largest_share_p50": [0.2, null], "plate_life_p50_myr": null}"#)
            .unwrap();
        assert_eq!(t.range("largest_share_p50"), (Some(0.2), None));
        assert_eq!(t.range("plate_life_p50_myr"), (None, None));
        assert!(t.apply_json(r#"{"no_such_measure": [1, 2]}"#).is_err());
        assert!(t.apply_json(r#"{"largest_share_p50": [1]}"#).is_err());
        assert!(t.apply_json("[]").is_err());
        // The moved range is the one the check uses.
        let rows = evaluate(&[run_with(&[600, 400])], &t);
        let r = row(&rows, "largest_share_p50");
        assert_eq!((r.lo, r.hi), (Some(0.2), None));
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn percentiles_interpolate() {
        assert_eq!(pct(&[1.0, 2.0, 3.0, 4.0], 0.5), 2.5);
        assert_eq!(pct(&[1.0, 2.0, 3.0, 4.0], 0.0), 1.0);
        assert_eq!(pct(&[1.0, 2.0, 3.0, 4.0], 1.0), 4.0);
        assert!(pct(&[], 0.5).is_nan());
        assert!(pct(&[f64::NAN], 0.5).is_nan());
        assert!(pearson(&[1.0, 1.0, 1.0], &[1.0, 2.0, 3.0]).is_nan());
        assert!((pearson(&[1.0, 2.0, 3.0], &[2.0, 4.0, 6.0]) - 1.0).abs() < 1e-12);
    }

    // ---- Task 1b: speeds, rotation, the guard, the sea floor, continents, mega-plates ----

    /// A plate with its 1b fields.
    fn moving(id: u32, visible: u32, continental: u32, speed: f64, front: f64) -> PlateStep {
        plate(id, visible, continental, speed, front, 1000.0)
    }

    /// A step holding exactly these plates.
    fn step_of(t: f64, plates: Vec<PlateStep>) -> StepRec {
        StepRec {
            t,
            visible: plates.iter().map(|p| p.visible).sum(),
            plates,
            ..StepRec::default()
        }
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn plate_speeds_by_slab_crust_and_area() {
        // Plate 0 on slab and oceanic, 1 on slab, 2 continental and free, 3 a 5-sample sliver.
        let mk = |t: f64| {
            step_of(
                t,
                vec![
                    moving(0, 400, 0, 80.0, 500.0),
                    moving(1, 300, 0, 60.0, 300.0),
                    moving(2, 200, 150, 30.0, 0.0),
                    moving(3, 5, 0, 2000.0, 0.0),
                ],
            )
        };
        let d = dynamics(vec![mk(2.0), mk(4.0), mk(6.0)]);
        let m = measures(&d, &[]);
        // The RMS counts every plate by area, the sliver too.
        let sq: f64 = 400.0 * 6400.0 + 300.0 * 3600.0 + 200.0 * 900.0 + 5.0 * 4.0e6;
        assert!(near(get(&m, "speed_rms_mm_yr"), (sq / 905.0).sqrt()));
        // The percentiles do not: 30, 60 and 80, three steps each.
        assert_eq!(get(&m, "speed_big_p10_mm_yr"), 30.0);
        assert_eq!(get(&m, "speed_big_p50_mm_yr"), 60.0);
        assert_eq!(get(&m, "speed_big_p90_mm_yr"), 80.0);
        // Oceanic (0, 1: median 70) over continental (2: 30); on slab (0, 1) over free (2).
        assert!(near(get(&m, "speed_oceanic_over_continental"), 70.0 / 30.0));
        assert!(near(get(&m, "speed_slab_over_free"), 70.0 / 30.0));
        // Speed rises with slab share and with area, without exception.
        assert!(near(get(&m, "speed_slab_rank_corr"), 1.0));
        assert!(near(get(&m, "speed_area_rank_corr"), 1.0));
        assert_eq!(get(&m, "speed_over_150_pct"), 0.0);
        assert_eq!(get(&m, "speed_over_150_plate_steps"), 0.0);
    }

    #[test]
    fn fast_plate_steps_are_counted_among_plates_of_30_samples_or_more() {
        let mk = |t: f64, fast: f64| {
            step_of(
                t,
                vec![
                    moving(0, 500, 0, fast, 0.0),
                    moving(1, 400, 0, 50.0, 0.0),
                    moving(2, 29, 0, 900.0, 0.0),
                ],
            )
        };
        let d = dynamics(vec![
            mk(2.0, 200.0),
            mk(4.0, 60.0),
            mk(6.0, 150.0),
            mk(8.0, 70.0),
        ]);
        let m = measures(&d, &[]);
        // Of 8 plate-steps, one is above 150 (200); 150 itself is not, the 29-sample plate is
        // out.
        assert_eq!(get(&m, "speed_over_150_plate_steps"), 1.0);
        assert!(near(get(&m, "speed_over_150_pct"), 12.5));
        // The same with another threshold and another size limit.
        let p = Params {
            fast_mm_yr: 60.0,
            min_plate_samples: 10.0,
            ..Params::default()
        };
        let m = measures_with(&d, &[], &p);
        assert_eq!(get(&m, "speed_over_150_plate_steps"), 3.0 + 4.0);
    }

    #[test]
    fn rank_correlation_shares_ranks_between_ties() {
        assert_eq!(ranks(&[10.0, 30.0, 20.0, 20.0]), vec![1.0, 4.0, 2.5, 2.5]);
        assert!(near(
            spearman(&[1.0, 2.0, 3.0, 4.0], &[1.0, 8.0, 27.0, 64.0]),
            1.0
        ));
        assert!(near(
            spearman(&[1.0, 2.0, 3.0, 4.0], &[9.0, 7.0, 4.0, 1.0]),
            -1.0
        ));
        // A monotone but very nonlinear relation: rank 1, Pearson below it.
        let (x, y) = ([1.0, 2.0, 3.0, 4.0, 5.0], [1.0, 2.0, 3.0, 4.0, 1000.0]);
        assert!(near(spearman(&x, &y), 1.0) && pearson(&x, &y) < 0.9);
        assert!(spearman(&[1.0, 1.0, 1.0], &[1.0, 2.0, 3.0]).is_nan());
    }

    #[test]
    fn the_guard_counts_steps_and_plate_steps_and_fails_when_it_binds() {
        let mk = |t: f64, bound: &[bool]| {
            step_of(
                t,
                bound
                    .iter()
                    .enumerate()
                    .map(|(i, &g)| PlateStep {
                        guard: g,
                        ..moving(i as u32, 100, 0, 50.0, 0.0)
                    })
                    .collect(),
            )
        };
        let quiet = dynamics(vec![mk(2.0, &[false, false]), mk(4.0, &[false, false])]);
        let m = measures(&quiet, &[]);
        assert_eq!(get(&m, "guard_bind_steps"), 0.0);
        assert_eq!(get(&m, "guard_bind_plate_steps"), 0.0);
        let busy = dynamics(vec![
            mk(2.0, &[true, true]),
            mk(4.0, &[false, false]),
            mk(6.0, &[false, true]),
        ]);
        let m = measures(&busy, &[]);
        assert_eq!(get(&m, "guard_bind_steps"), 2.0);
        assert_eq!(get(&m, "guard_bind_plate_steps"), 3.0);
        // Target 0: one binding step in one run fails the check, and none passes it.
        let run = |d: Dynamics| {
            let rows = evaluate(
                &[RunData {
                    name: "r".into(),
                    dynamics: d,
                    margins: vec![],
                }],
                &Targets::builtin(),
            );
            row(&rows, "guard_bind_steps").status
        };
        assert_eq!(run(quiet), Status::Pass);
        assert_eq!(run(busy), Status::Fail);
    }

    #[test]
    fn rotation_change_and_pole_are_measured_on_plates_with_a_previous_step() {
        // Changes of 4, 8, 12 % (known) and one plate with no previous step (not counted).
        let mk = |t: f64, change: f64, known: bool, inside: bool| {
            step_of(
                t,
                vec![PlateStep {
                    rot_change: change,
                    rot_known: known,
                    pole_inside: inside,
                    spinning: change > 0.1,
                    ..moving(0, 100, 0, 50.0, 0.0)
                }],
            )
        };
        let d = dynamics(vec![
            mk(2.0, 0.0, false, true),
            mk(4.0, 0.04, true, false),
            mk(6.0, 0.08, true, false),
            mk(8.0, 0.12, true, true),
        ]);
        let m = measures(&d, &[]);
        assert!(near(get(&m, "rot_change_p50_pct"), 8.0));
        assert!(near(get(&m, "rot_change_p90_pct"), 11.2));
        // Two of the four plate-steps have the pole on the plate.
        assert!(near(get(&m, "pole_inside_pct"), 50.0));
        // One of the four (the 12 % change, standing for a pole at the centroid) is spinning.
        assert!(near(get(&m, "spinning_pct"), 25.0));
        // Nothing known: no data.
        let first = dynamics(vec![mk(2.0, 0.0, false, false)]);
        assert!(get(&measures(&first, &[]), "rot_change_p50_pct").is_nan());
    }

    #[test]
    fn the_sea_floor_is_judged_on_the_run_after_the_warm_up() {
        // t0 = 0: the step at 50 Myr is warm-up. After it: medians 60 and 40, 90th percentiles
        // 200 and 220, 12 % and 8 % older than 180 Myr, 4 % and 2 % older than 250.
        let mk = |t: f64, p50: f64, p90: f64, o180: u32, o250: u32| StepRec {
            ocean: 1000,
            age_p50: p50,
            age_p90: p90,
            old180: o180,
            old250: o250,
            ..step(t, 0, 0, &[1000])
        };
        let mut d = dynamics(vec![
            mk(50.0, 100.0, 300.0, 400, 300),
            mk(150.0, 60.0, 200.0, 120, 40),
            mk(250.0, 40.0, 220.0, 80, 20),
        ]);
        d.t0_myr = 0.0;
        let m = measures(&d, &[]);
        assert!(near(get(&m, "ocean_age_p50_myr"), 50.0));
        assert!(near(get(&m, "ocean_age_p90_mean_myr"), 210.0));
        assert!(near(get(&m, "ocean_old180_pct"), 10.0));
        assert!(near(get(&m, "ocean_old250_pct"), 3.0));
        // The warm-up is a parameter; a run shorter than it uses every step.
        let p = Params {
            warmup_myr: 0.0,
            ..Params::default()
        };
        assert!(near(
            get(&measures_with(&d, &[], &p), "ocean_age_p50_myr"),
            200.0 / 3.0
        ));
        let p = Params {
            warmup_myr: 1000.0,
            ..Params::default()
        };
        assert!(near(
            get(&measures_with(&d, &[], &p), "ocean_age_p50_myr"),
            200.0 / 3.0
        ));
        // No sea floor recorded: no data.
        let bare = dynamics(vec![step(2.0, 0, 0, &[1000])]);
        assert!(get(&measures(&bare, &[]), "ocean_old180_pct").is_nan());
    }

    #[test]
    fn continental_extremes_leave_out_the_start_up_and_all_steps_are_reported() {
        // t0 = 0 and a 10 Myr start-up: 0.49 at 2 Myr is the start-up.
        let d = dynamics(vec![
            step(2.0, 0, 490, &[1000]),
            step(12.0, 0, 400, &[1000]),
            step(14.0, 0, 430, &[1000]),
            step(16.0, 0, 380, &[1000]),
        ]);
        let m = measures(&d, &[]);
        assert!(near(get(&m, "continental_max"), 0.43));
        assert!(near(get(&m, "continental_min"), 0.38));
        assert!(near(get(&m, "continental_max_all"), 0.49));
        assert!(near(get(&m, "continental_min_all"), 0.38));
        let p = Params {
            start_up_myr: 0.0,
            ..Params::default()
        };
        assert!(near(
            get(&measures_with(&d, &[], &p), "continental_max"),
            0.49
        ));
    }

    /// A step with `continental` samples of continental crust in these pieces.
    fn with_pieces(t: f64, continental: u32, pieces: &[u32]) -> StepRec {
        StepRec {
            continental,
            pieces: pieces.to_vec(),
            ..step(t, 0, continental, &[1000])
        }
    }

    #[test]
    fn continental_pieces_and_their_largest_share() {
        // 400 continental samples: 360 + 20 + 10 + 2 (the 2 is under 1 % of 400); then one piece.
        let d = dynamics(vec![
            with_pieces(2.0, 400, &[360, 20, 10, 2]),
            with_pieces(4.0, 400, &[200, 200]),
            with_pieces(6.0, 400, &[400]),
        ]);
        let m = measures(&d, &[]);
        assert_eq!(get(&m, "continent_pieces_p50"), 2.0);
        assert!(near(get(&m, "continent_largest_share_p50"), 0.9));
        assert!(near(get(&m, "continent_largest_share_min"), 0.5));
        let p = Params {
            min_piece_share: 0.0,
            ..Params::default()
        };
        assert_eq!(
            get(&measures_with(&d, &[], &p), "continent_pieces_p50"),
            2.0
        );
    }

    #[test]
    fn supercontinent_assemblies_have_hysteresis() {
        let shares = [
            (2.0, 0.8),
            (4.0, 0.75),
            (6.0, 0.6),
            (8.0, 0.45),
            (10.0, 0.55),
            (12.0, 0.72),
            (14.0, 0.9),
            (16.0, 0.4),
            (18.0, 0.3),
            (20.0, 0.71),
        ];
        let eps = assemblies(&shares, 0.0, 0.7, 0.5);
        // Assembled at the start (time 0), dispersed at 8; 12 to 16; 20 and still.
        assert_eq!(
            eps,
            vec![
                Assembly {
                    start_myr: 0.0,
                    end_myr: Some(8.0)
                },
                Assembly {
                    start_myr: 12.0,
                    end_myr: Some(16.0)
                },
                Assembly {
                    start_myr: 20.0,
                    end_myr: None
                },
            ]
        );
        // No hysteresis: the first break-up comes at 6 Myr (60 %), not at 8 (45 %).
        let flat = assemblies(&shares, 0.0, 0.7, 0.7);
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].end_myr, Some(6.0));
        // Never assembled; a missing share is a zero.
        assert!(assemblies(&[(2.0, 0.6), (4.0, f64::NAN)], 0.0, 0.7, 0.5).is_empty());
        // A run that starts dispersed has its first assembly at the step it forms.
        let late = assemblies(&[(2.0, 0.3), (4.0, 0.8)], 0.0, 0.7, 0.5);
        assert_eq!(late[0].start_myr, 4.0);
    }

    #[test]
    fn supercontinent_tenure_period_and_time_come_from_the_largest_piece() {
        // 100 continental samples throughout; the largest piece is the series above.
        let largest = [80, 75, 60, 45, 55, 72, 90, 40, 30, 71];
        let steps: Vec<StepRec> = largest
            .iter()
            .enumerate()
            .map(|(i, &l)| with_pieces(2.0 * (i + 1) as f64, 100, &[l]))
            .collect();
        let d = dynamics(steps);
        let m = measures(&d, &[]);
        // Tenures 8 and 4 (the third assembly is cut off by the run's end): median 6. Starts at
        // 0, 12 and 20: gaps 12 and 8, median 10. Held at 2, 4, 6, 12, 14 and 20: 60 %.
        assert_eq!(get(&m, "supercontinent_tenure_myr"), 6.0);
        assert_eq!(get(&m, "supercontinent_period_myr"), 10.0);
        assert_eq!(get(&m, "supercontinent_episodes"), 3.0);
        assert!(near(get(&m, "supercontinent_time_pct"), 60.0));
        // A looser definition: assembled from 55 %, dispersed under 50 %.
        let p = Params {
            assembled_share: 0.55,
            ..Params::default()
        };
        let m = measures_with(&d, &[], &p);
        assert_eq!(get(&m, "supercontinent_episodes"), 3.0);
        assert_eq!(get(&m, "supercontinent_tenure_myr"), 7.0);
        // Not recorded (no pieces): no data, not "no supercontinent".
        let bare = dynamics(vec![step(2.0, 0, 100, &[1000]), step(4.0, 0, 100, &[1000])]);
        let m = measures(&bare, &[]);
        assert!(get(&m, "supercontinent_tenure_myr").is_nan());
        assert!(get(&m, "supercontinent_episodes").is_nan());
        // One assembly that never ends: a count, no tenure and no period.
        let held = dynamics(vec![
            with_pieces(2.0, 100, &[90]),
            with_pieces(4.0, 100, &[95]),
        ]);
        let m = measures(&held, &[]);
        assert_eq!(get(&m, "supercontinent_episodes"), 1.0);
        assert!(get(&m, "supercontinent_tenure_myr").is_nan());
        assert!(get(&m, "supercontinent_period_myr").is_nan());
    }

    /// Steps `t` = 2, 4, 6 with plates [600, 400], then a rift of plate 0 at 8 (fragment 2),
    /// and steps up to 100.
    fn megaplate_run(extra: Vec<PlateLife>) -> Dynamics {
        let mut steps = vec![
            step(2.0, 0, 0, &[600, 400]),
            step(4.0, 0, 0, &[600, 400]),
            step(6.0, 0, 0, &[600, 400]),
            step(8.0, 0, 0, &[300, 400, 300]),
        ];
        steps.push(step(100.0, 0, 0, &[300, 400, 300]));
        let mut d = dynamics(steps);
        d.plates = vec![
            life(0, None, "initial", 0.0),
            life(1, None, "initial", 0.0),
            life(2, Some(0), "rift", 8.0),
        ];
        d.plates.extend(extra);
        d
    }

    #[test]
    fn a_rift_of_a_megaplate_is_timed_and_judged_after_50_myr() {
        let d = megaplate_run(vec![]);
        let rifts = mega_rifts(&d, 0.4, 50.0);
        // Plate 0 stood above 40 % from the start (time 0) to the rift at 8: four steps. Plate 1
        // holds exactly 40 %, which is not above it.
        assert_eq!(
            rifts,
            vec![MegaRift {
                parent: 0,
                t_myr: 8.0,
                held_myr: 8.0,
                apart: Some(true),
                split: Some(true)
            }]
        );
        let m = measures(&d, &[]);
        assert_eq!(get(&m, "megaplate_rifts"), 1.0);
        assert_eq!(get(&m, "megaplate_tenure_p50_myr"), 8.0);
        assert_eq!(get(&m, "megaplate_rift_apart_50_pct"), 100.0);
        assert_eq!(get(&m, "megaplate_rift_split_50_pct"), 100.0);
        // The share is a parameter: above 35 % plate 1 counts too, but it did not rift.
        assert_eq!(mega_rifts(&d, 0.35, 50.0).len(), 1);
        // And above 70 % nothing is a mega-plate: no rifts, no data.
        let m = measures_with(
            &d,
            &[],
            &Params {
                mega_plate_share: 0.7,
                ..Params::default()
            },
        );
        assert_eq!(get(&m, "megaplate_rifts"), 0.0);
        assert!(get(&m, "megaplate_tenure_p50_myr").is_nan());
        assert!(get(&m, "megaplate_rift_apart_50_pct").is_nan());
    }

    #[test]
    fn a_rift_that_heals_is_not_apart() {
        // The fragment welds back into its parent after 20 Myr.
        let mut d = megaplate_run(vec![]);
        d.plates[2] = dies(life(2, Some(0), "rift", 8.0), 28.0, "sutured", Some(0));
        assert_eq!(mega_rifts(&d, 0.4, 50.0)[0].apart, Some(false));
        // The only fragment rejoined, so one plate holds it all: not split either.
        assert_eq!(mega_rifts(&d, 0.4, 50.0)[0].split, Some(false));
        // After the window it is apart.
        d.plates[2] = dies(life(2, Some(0), "rift", 8.0), 80.0, "sutured", Some(0));
        assert_eq!(mega_rifts(&d, 0.4, 50.0)[0].apart, Some(true));
        assert_eq!(mega_rifts(&d, 0.4, 50.0)[0].split, Some(true));
        // Into a stranger within the window: the rift still holds.
        d.plates[2] = dies(life(2, Some(0), "rift", 8.0), 28.0, "sutured", Some(1));
        assert_eq!(mega_rifts(&d, 0.4, 50.0)[0].apart, Some(true));
        // Into a plate that took the parent's crust earlier: healed through the chain.
        d.plates[0] = dies(life(0, None, "initial", 0.0), 20.0, "sutured", Some(5));
        d.plates[2] = dies(life(2, Some(0), "rift", 8.0), 30.0, "sutured", Some(5));
        d.plates.push(life(5, None, "initial", 0.0));
        assert_eq!(mega_rifts(&d, 0.4, 50.0)[0].apart, Some(false));
        // A rift into three: one fragment rejoins the parent, the other stays. Not "all apart",
        // but split.
        let mut three = megaplate_run(vec![life(3, Some(0), "rift", 8.0)]);
        three.plates[2] = dies(life(2, Some(0), "rift", 8.0), 28.0, "sutured", Some(0));
        let r = mega_rifts(&three, 0.4, 50.0);
        assert_eq!((r[0].apart, r[0].split), (Some(false), Some(true)));
        // A fragment subducted away holds nothing: the parent alone is not a split.
        let mut gone = megaplate_run(vec![]);
        gone.plates[2] = dies(life(2, Some(0), "rift", 8.0), 28.0, "consumed", None);
        let r = mega_rifts(&gone, 0.4, 50.0);
        assert_eq!((r[0].apart, r[0].split), (Some(true), Some(false)));
        // A rift too close to the end of the run to judge: no verdict, and no data.
        let mut short = megaplate_run(vec![]);
        short.steps.pop();
        assert_eq!(mega_rifts(&short, 0.4, 50.0)[0].apart, None);
        assert!(get(&measures(&short, &[]), "megaplate_rift_apart_50_pct").is_nan());
        assert!(get(&measures(&short, &[]), "megaplate_rift_split_50_pct").is_nan());
    }

    #[test]
    fn rifts_of_a_plate_that_was_not_a_megaplate_do_not_count() {
        // Plate 0 rifts while at 40 %, then again from above 40 % (a second rift).
        let mut d = dynamics(vec![
            step(2.0, 0, 0, &[400, 600]),
            step(4.0, 0, 0, &[200, 600, 200]),
            step(6.0, 0, 0, &[200, 600, 200]),
        ]);
        d.plates = vec![
            life(0, None, "initial", 0.0),
            life(1, None, "initial", 0.0),
            life(2, Some(0), "rift", 4.0),
            life(3, Some(1), "rift", 6.0),
        ];
        let rifts = mega_rifts(&d, 0.4, 50.0);
        // Only plate 1 (60 % before its rift); plate 0 stood at exactly 40 %.
        assert_eq!(rifts.len(), 1);
        assert_eq!(rifts[0].parent, 1);
        assert_eq!(rifts[0].t_myr, 6.0);
        // It had been above 40 % since the start (steps at 2, 4, 6: 6 Myr).
        assert_eq!(rifts[0].held_myr, 6.0);
    }

    #[test]
    fn measures_that_wait_for_physics_say_no_data_and_do_not_fail() {
        let pending: Vec<&MeasureDef> = MEASURES.iter().filter(|d| d.pending.is_some()).collect();
        assert!(pending.len() >= 8, "{}", pending.len());
        let run = RunData {
            name: "r".into(),
            dynamics: dynamics(vec![step(2.0, 0, 0, &[1000])]),
            margins: vec![],
        };
        let rows = evaluate(std::slice::from_ref(&run), &Targets::builtin());
        for d in &pending {
            let r = row(&rows, d.key);
            assert_eq!(r.status, Status::Pending, "{}", d.key);
            assert!(!r.status.failed());
        }
        let text = render(&rows);
        assert!(text.contains("no data"));
        // The ones with a range among them do not fail the check.
        let ranged_waiting = pending
            .iter()
            .filter(|d| d.lo.is_some() || d.hi.is_some())
            .count();
        assert!(ranged_waiting >= 4);
        assert!(
            !rows
                .iter()
                .any(|r| r.def.pending.is_some() && r.status.failed())
        );
        // A measure that is not waiting and has no data still fails.
        assert_eq!(
            row(&rows, "supercontinent_tenure_myr").status,
            Status::NoData
        );
    }

    #[test]
    fn the_balance_is_measured_against_todays_motion() {
        let answer = |id: u32, speed: f64, rot: Option<f64>| PlateForces {
            id,
            slab_pull: 1.0,
            speed_mm_yr: speed,
            rms_mm_yr: speed,
            rot_change: rot.unwrap_or(0.0),
            rot_known: rot.is_some(),
            ..PlateForces::default()
        };
        // Plate 0 oceanic on slab, 1 continental and free, 2 oceanic and nearly free, 3 a sliver.
        let plates = vec![
            moving(0, 400, 0, 80.0, 400.0),
            moving(1, 300, 200, 40.0, 0.0),
            moving(2, 300, 0, 60.0, 30.0),
            moving(3, 5, 0, 10.0, 0.0),
        ];
        let mut a = step_of(2.0, plates.clone());
        a.forces = vec![
            answer(0, 90.0, None),
            answer(1, 20.0, None),
            answer(2, 40.0, None),
            answer(3, 0.0, None),
        ];
        a.slabs_km = vec![100.0];
        a.breakoffs = 1;
        let mut b = step_of(4.0, plates);
        b.forces = vec![
            answer(0, 90.0, Some(0.1)),
            answer(1, 20.0, Some(0.2)),
            answer(2, 40.0, Some(0.3)),
            answer(3, 0.0, Some(0.9)),
        ];
        let d = dynamics(vec![a, b]);
        let m = measures(&d, &[]);
        // RMS over the plates' areas: √((400·90² + 300·20² + 300·40²) / 1005).
        let rms = (3_840_000.0f64 / 1005.0).sqrt();
        assert!(near(get(&m, "balance_rms_mm_yr"), rms));
        // Pooled over both steps, the sliver left out: 20, 20, 40, 40, 90, 90.
        assert!(near(get(&m, "balance_p50_mm_yr"), 40.0));
        assert!(near(get(&m, "balance_norm_p50_mm_yr"), 40.0 * 45.0 / rms));
        assert_eq!(get(&m, "balance_over_150_pct"), 0.0);
        // Oceanic (0, 2: median 65) over continental (1: 20); slab (0: 90) over free (1 and 2:
        // median 30).
        assert!(near(get(&m, "balance_oceanic_over_continental"), 3.25));
        assert!(near(get(&m, "balance_slab_over_free"), 3.0));
        assert!(near(get(&m, "balance_slab_rank_corr"), 1.0));
        assert!(near(get(&m, "balance_vs_motion_rank_corr"), 1.0));
        assert!(near(get(&m, "balance_over_motion_p50"), 40.0 / 60.0));
        assert!(near(get(&m, "balance_rot_change_p50_pct"), 20.0));
        // One break-off in 4 Myr.
        assert!(near(get(&m, "slab_breakoffs_per_100myr"), 25.0));
        assert_eq!(get(&m, "slab_length_km_p50"), 100.0);
        // A record without the balance has no data, and the rows wait.
        let none = measures(&dynamics(vec![step_of(2.0, vec![moving(0, 400, 0, 80.0, 0.0)])]), &[]);
        assert!(get(&none, "balance_p50_mm_yr").is_nan());
        assert!(get(&none, "slab_breakoffs_per_100myr").is_nan());
    }

    #[test]
    fn filling_the_record_lights_the_waiting_rows_up() {
        let mut d = dynamics(vec![
            step(2.0, 0, 0, &[600, 400]),
            step(4.0, 0, 0, &[600, 400]),
        ]);
        let belt = |onset: f64, later: Option<f64>| BeltRec {
            convergence_onset_mm_yr: Some(onset),
            convergence_20myr_mm_yr: later,
            wedge_width_km: Some(200.0),
            foredeep_width_km: Some(300.0),
            foredeep_depth_m: Some(4000.0),
            ..BeltRec::default()
        };
        // Ratios 0.3 and 0.5 (the third has not lasted 20 Myr).
        d.belts = vec![
            belt(100.0, Some(30.0)),
            belt(60.0, Some(30.0)),
            belt(80.0, None),
        ];
        d.steps[1].forces = vec![
            PlateForces {
                id: 0,
                slab_pull: 6.0,
                slab_suction: 1.0,
                edge_energy: 1.0,
                stress_mpa: -20.0,
                ..PlateForces::default()
            },
            PlateForces {
                id: 1,
                slab_pull: 1.0,
                slab_suction: 1.0,
                edge_energy: 0.0,
                stress_mpa: 30.0,
                ..PlateForces::default()
            },
        ];
        d.steps[1].slabs_km = vec![300.0, 500.0, 700.0];
        let m = measures(&d, &[]);
        assert!(near(get(&m, "collision_convergence_ratio_20myr"), 0.4));
        assert_eq!(get(&m, "wedge_width_km_p50"), 200.0);
        assert_eq!(get(&m, "foredeep_width_km_p50"), 300.0);
        assert_eq!(get(&m, "foredeep_depth_m_p50"), 4000.0);
        assert_eq!(get(&m, "slab_length_km_p50"), 500.0);
        // Pull shares 0.75 and 0.5.
        assert!(near(get(&m, "force_slab_pull_share_p50"), 0.625));
        // Only plate 0 (60 %) is a mega-plate: in compression.
        assert_eq!(get(&m, "megaplate_stress_mpa_p50"), -20.0);
        assert_eq!(get(&m, "megaplate_tension_pct"), 0.0);
        // They now have values, so the ranged ones are judged (ratio 0.4 is inside 0.2-0.6).
        let rows = evaluate(
            &[RunData {
                name: "r".into(),
                dynamics: d,
                margins: vec![],
            }],
            &Targets::builtin(),
        );
        assert_eq!(
            row(&rows, "collision_convergence_ratio_20myr").status,
            Status::Pass
        );
        assert_eq!(row(&rows, "wedge_width_km_p50").status, Status::Pass);
        assert_eq!(row(&rows, "foredeep_depth_m_p50").status, Status::Pass);
    }

    #[test]
    fn the_definitions_can_be_set_by_name_and_from_the_targets_file() {
        let mut p = Params::default();
        assert_eq!(p.assembled_share, 0.7);
        p.set_text("assembled_share=0.8").unwrap();
        p.set_text(" mega_plate_share = 0.35 ").unwrap();
        assert_eq!((p.assembled_share, p.mega_plate_share), (0.8, 0.35));
        assert!(p.set_text("assembled_share").is_err());
        assert!(p.set_text("assembled_share=much").is_err());
        assert!(p.set_text("no_such=1").is_err());
        assert!(p.set("warmup_myr", f64::NAN).is_err());
        for k in Params::KEYS {
            assert!(Params::default().set(k, 1.0).is_ok(), "{k}");
        }
        let mut t = Targets::builtin();
        t.apply_json(r#"{"params": {"dispersed_share": 0.4}, "speed_rms_mm_yr": [null, 50]}"#)
            .unwrap();
        assert_eq!(t.params.dispersed_share, 0.4);
        assert_eq!(t.range("speed_rms_mm_yr"), (None, Some(50.0)));
        assert!(t.apply_json(r#"{"params": {"nope": 1}}"#).is_err());
        assert!(t.apply_json(r#"{"params": {"warmup_myr": "x"}}"#).is_err());
        assert!(t.apply_json(r#"{"params": 3}"#).is_err());
        assert!(Params::default().describe().contains("70 %"));
    }

    #[test]
    fn the_boundary_mix_is_a_share_of_the_length_after_the_warm_up() {
        use crate::plates::dynamics::mix::*;
        let mk = |t: f64, m: [f64; MIX_CLASSES]| StepRec {
            mix_km: m,
            ..step(t, 0, 0, &[1000])
        };
        let mut v = [0.0; MIX_CLASSES];
        // The warm-up step (50 Myr): nothing but ridge. Two later steps: 20 % and 30 % subducting
        // (oo : oc = 3 : 1 and 1 : 1), the rest spreading, collision and other.
        v[RIDGE] = 100.0;
        let early = mk(50.0, v);
        let mut a = [0.0; MIX_CLASSES];
        a[OCEAN_OCEAN] = 150.0;
        a[OCEAN_CONTINENT] = 50.0;
        a[RIDGE] = 300.0;
        a[RIFT] = 100.0;
        a[COLLISION] = 100.0;
        a[OTHER] = 300.0;
        let mut b = [0.0; MIX_CLASSES];
        b[OCEAN_OCEAN] = 150.0;
        b[OCEAN_CONTINENT] = 150.0;
        b[RIDGE] = 400.0;
        b[OTHER] = 300.0;
        let mut d = dynamics(vec![early, mk(150.0, a), mk(250.0, b)]);
        d.t0_myr = 0.0;
        let m = measures(&d, &[]);
        // (20 + 30) / 2; spreading (40 + 40) / 2; collision (10 + 0) / 2; other (30 + 30) / 2.
        assert!(near(get(&m, "boundary_subduction_pct"), 25.0));
        assert!(near(get(&m, "boundary_spreading_pct"), 40.0));
        assert!(near(get(&m, "boundary_collision_pct"), 5.0));
        assert!(near(get(&m, "boundary_other_pct"), 30.0));
        // Lengths: 1,000 km a step; 250 km of it subducting on average.
        assert!(near(get(&m, "boundary_total_km"), 1000.0));
        assert!(near(get(&m, "boundary_subduction_km"), 250.0));
        // Ocean-ocean: 300 of 500 km of subduction, summed over the steps.
        assert!(near(get(&m, "boundary_oo_of_subduction_pct"), 60.0));
        // The first 50 Myr is the first step alone (t = 50): all ridge, nothing subducting.
        assert!(near(get(&m, "boundary_subduction_pct_early"), 0.0));
        assert!(near(get(&m, "boundary_spreading_pct_early"), 100.0));
        assert!(get(&m, "boundary_oo_of_subduction_pct_early").is_nan());
        // 25 % is inside 15-25 (both ends included); 26 would not be.
        let rows = evaluate(
            &[RunData {
                name: "r".into(),
                dynamics: d,
                margins: vec![],
            }],
            &Targets::builtin(),
        );
        assert_eq!(row(&rows, "boundary_subduction_pct").status, Status::Pass);
        // A boundary-less step is skipped, not counted as 0 %.
        let bare = dynamics(vec![step(150.0, 0, 0, &[1000])]);
        assert!(get(&measures(&bare, &[]), "boundary_subduction_pct").is_nan());
    }

    #[test]
    fn arc_heights_are_judged_per_kind_and_a_run_without_them_has_no_data() {
        use crate::plates::dynamics::ArcStat;
        // Oceanic arc: 10 cells (2 emergent), median -400, p90 300; then 30 cells (3 emergent),
        // median -200, p90 500. Continental arc: 20 cells, median 2000, p90 4000, in the second
        // step only.
        let mk = |t: f64, oc: ArcStat, cont: ArcStat| StepRec {
            arc_oceanic: oc,
            arc_continental: cont,
            ..step(t, 0, 0, &[1000])
        };
        let stat = |cells, emergent, p50, p90| ArcStat {
            cells,
            emergent,
            p50_m: p50,
            p90_m: p90,
            max_m: p90 + 100.0,
        };
        let mut d = dynamics(vec![
            // Warm-up: left out.
            mk(
                50.0,
                stat(100, 100, 3000.0, 4000.0),
                stat(100, 100, 9000.0, 9000.0),
            ),
            mk(150.0, stat(10, 2, -400.0, 300.0), ArcStat::default()),
            mk(
                250.0,
                stat(30, 3, -200.0, 500.0),
                stat(20, 20, 2000.0, 4000.0),
            ),
        ]);
        d.t0_myr = 0.0;
        let m = measures(&d, &[]);
        // Emergent: 5 of 40 cells. The medians and percentiles are weighted by cells.
        assert!(near(get(&m, "arc_oceanic_emergent_pct"), 12.5));
        assert!(near(
            get(&m, "arc_oceanic_p50_m"),
            (10.0 * -400.0 + 30.0 * -200.0) / 40.0
        ));
        assert!(near(
            get(&m, "arc_oceanic_p90_m"),
            (10.0 * 300.0 + 30.0 * 500.0) / 40.0
        ));
        assert!(near(get(&m, "arc_oceanic_cells_mean"), 20.0));
        assert!(near(get(&m, "arc_continental_p50_m"), 2000.0));
        assert!(near(get(&m, "arc_continental_p90_m"), 4000.0));
        assert!(near(get(&m, "arc_continental_cells_mean"), 10.0));
        // The first 50 Myr is the step at 50.
        assert!(near(get(&m, "arc_oceanic_emergent_pct_early"), 100.0));
        assert!(near(get(&m, "arc_continental_p90_m_early"), 9000.0));
        // Judged: crest 450 is inside the oceanic range (the median is only reported); 12.5 % is
        // inside 10-35.
        let rows = evaluate(
            &[RunData {
                name: "r".into(),
                dynamics: d,
                margins: vec![],
            }],
            &Targets::builtin(),
        );
        for k in [
            "arc_oceanic_emergent_pct",
            "arc_oceanic_p90_m",
            "arc_continental_p50_m",
            "arc_continental_p90_m",
        ] {
            assert_eq!(row(&rows, k).status, Status::Pass, "{k}");
        }
        // A high island arc fails its crest.
        let mut hi = dynamics(vec![mk(
            150.0,
            stat(10, 9, 600.0, 3000.0),
            ArcStat::default(),
        )]);
        hi.t0_myr = 0.0;
        let rows = evaluate(
            &[RunData {
                name: "hi".into(),
                dynamics: hi,
                margins: vec![],
            }],
            &Targets::builtin(),
        );
        assert_eq!(row(&rows, "arc_oceanic_p90_m").status, Status::Fail);
        assert_eq!(row(&rows, "arc_oceanic_p50_m").status, Status::Reported);
        assert_eq!(row(&rows, "arc_oceanic_emergent_pct").status, Status::Fail);
        // No continental arc recorded: no data, which does not fail (the run predates the record,
        // or the arcs never formed).
        assert_eq!(row(&rows, "arc_continental_p90_m").status, Status::Pending);
        assert!(!row(&rows, "arc_continental_p90_m").status.failed());
        // A record without any of it: every new judged row says "no data" and none fails.
        let old = RunData {
            name: "old".into(),
            dynamics: dynamics(vec![step(150.0, 0, 0, &[1000])]),
            margins: vec![],
        };
        let rows = evaluate(&[old], &Targets::builtin());
        for r in rows.iter().filter(|r| r.def.pending == Some(WAITS_MIX)) {
            assert_eq!(r.status, Status::Pending, "{}", r.def.key);
        }
        assert!(
            rows.iter()
                .filter(|r| r.def.pending == Some(WAITS_MIX))
                .count()
                >= 5
        );
    }
}

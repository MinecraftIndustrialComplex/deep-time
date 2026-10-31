// SPDX-License-Identifier: LGPL-3.0-or-later
//! A continental column settles towards the height its crust holds up (the continent-budget
//! task, 2026-10-19).
//!
//! # Why
//! The plate model moves a column's height only by what each process changes (`toward` in
//! `collapse.rs`, the belts', arcs' and transition's height lines), and nothing returns a column
//! to its isostatic height once some process has left it below. The gap budget (PLT-283,
//! plates.md "Which process holds the gap") found continental crust standing 1.1 to 1.9 km below
//! its Airy height in every thickness band, most of it made on the ocean floor: an island arc
//! turns continental at its height on the ocean's line, below the continent's Airy line, and
//! keeps the difference as it thickens. Normal continental crust (35 to 45 km) then lies under
//! the sea: at the end of a 1000 Myr supercontinent run (seed 2) the 35–40 km band's median is
//! −349 m against an Airy height of +868 m, and only 45 % of the continental crust is dry.
//!
//! # What happens on Earth
//! A column out of isostatic balance does not stay so: the mantle flows back within thousands
//! of years (Fennoscandia's rebound, relaxation times "close to 3-5 kyr": Wieczerkowski,
//! Mitrovica & Wolf 1999, abstract, ISO-064), except for what the lithosphere's flexural
//! strength holds up or down (the surface's flexure, below). What is thermal in a column's height
//! goes on the lithosphere's thermal time (McKenzie 1978's τ = 62.8 Myr): the model keeps that
//! part in its own rules (the ocean floor's age-depth law, the margins' transition, PLT-261 to
//! PLT-264), whose end states are the isostatic ones this rule settles to. The gaps the plate
//! rules leave are mechanical (a height moved by a change and not to an equilibrium; an arc's
//! crust on old ocean floor, whose cold lithosphere is about the continental reference column:
//! the water-loaded Airy height of 7 km of crust, −5.06 km, is the depth of 100 Myr old floor),
//! so they settle on the mantle's time, within the step.
//!
//! The settling time was 62.8 Myr (McKenzie's τ) at first: on `earthlike_quick_16k`
//! (supercontinent seed 2, 1000 Myr) it left the 35–45 km crust 0.6 to 0.7 km under its Airy
//! height and the continental crust 0.56 to 0.61 dry; 0.5 Myr and 10 Myr gave 0.68 and 0.71
//! (Earth 0.71, HYP-012) with every band at its Airy height (build log 2026-10-20).
//!
//! # The rule
//! Each plate step every visible continental sample (none at a trench) moves the share
//! 1 − e^(−δt/τ) of the way from its elevation z to
//!
//!   z* = z_eq + F,   z_eq = z_A if z_A ≥ s, else s + (z_A − s) · ρ_m / (ρ_m − ρ_w),
//!
//! with z_A its column's Airy height (`collapse::iso_height`), s the sea level, ρ_w the sea water
//! (ISO-044: a column below the sea carries the water above it) and F the surface's flexure
//! (`surface/flexure_m`, the flexure solve of ISO-040: how far the plate holds the ground above
//! or below local isostasy; 0 without a surface stage), τ = `isostasy_relax_myr` (4 kyr: the
//! whole way at any plate step, ISO-078). The change is the step's uplift; the step's rise guard
//! (PLT-288) runs after it. With `tectonics.thermal_isostasy_m` on, z_A gains the lithosphere's
//! thermal height (`thermal::uplift_m`, ISO-110): a hot lithosphere holds its column higher.
//!
//! The settling moves a height alone: it adds and removes no crust, so the surface's flexure
//! takes it as no load (ISO-091). The flexure books only `Sample::crust_uplift_mm_yr`, the change
//! of the column's own crust's Airy height ([`crust_uplift`]). Until 2026-10-22 it booked every
//! rise of a continental column as crust thickened, 7 times the rise, whose root a stiff plate
//! held up; the settling then moved the column to its Airy height plus that larger F, and the
//! two raised each other to the 30 km ceiling (`docs/science/flexure.md`, "The settling's
//! feedback").
//!
//! # Light rock off the top (`tectonics.settle_top_density`, off by default)
//! z_A is a whole column's height: crust of the column's mean density, 2,830 kg/m³, 7.0 m of
//! crust per m of height (PLT-281), right for crust stacked onto a column, spread from it or
//! foundering from its base. Rock the surface stage erodes is the top's, which PLT-281 keeps at
//! the upper crust's 2,700 kg/m³ (ISO-008: 5.5 m of rock per m of surface), and the surface's own
//! rebound gives back 0.8 of what it takes (ISO-001). Settling the eroded column to its
//! whole-column z_A lifts it further: by the uplift trace (docs/design/uplift-trace.md) the
//! response to erosion comes to 6/7 of the rock taken, against the upper crust's 1 − 2,700 /
//! 3,300 = 0.82. With the switch the column keeps a deficit D ([`TOP_DEFICIT_PER_M`] per metre
//! the surface took off its top, less as much per metre it laid down; `PlateState::top_deficit_m`,
//! carried through a resampling with the sample's weights) and settles at z_A − D: a column that
//! lost its light top is denser on average and stands lower. The column's other processes do not
//! move D (a simplification: the collapse's pure shear would thin it with the column, the belts'
//! stacking keeps it), except the rift's stretching (`extension.rs`), which thins it with the
//! crust. Deposition at 2,700 too (the surface's own rebound is one share for both, ISO-001).
// ledger: ISO-078 (the rule and τ), ISO-064 (the 3-5 kyr), ISO-044 (ρ_w), ISO-043 (ρ_m), PLT-780 (D).

use rayon::prelude::*;

use super::Env;
use super::collapse::iso_height;
use super::pools::{COLUMN_ISOSTATIC_FACTOR, ISOSTATIC_FACTOR};
use super::state::{CONTINENTAL, PlateState, TRENCH};
use super::thermal;
use crate::surface::flexure::{RHO_MANTLE, RHO_WATER};

/// `WorldState::globals`: the settling's rise and fall over the run, km³ of height × area.
pub const RISE_GLOBAL: &str = "plates/settle_rise_km3";
pub const FALL_GLOBAL: &str = "plates/settle_fall_km3";

/// How far below its whole-column Airy height a metre of rock taken off a column's top leaves it,
/// m: 1 / 5.5 − 1 / 7.0 = (ρ̄_c − ρ_u) / ρ_m = (2,830 − 2,700) / 3,300 = 0.039.
// ledger: PLT-780 (ISO-008's 2,700, PLT-281's 2,830, ISO-043's 3,300)
pub const TOP_DEFICIT_PER_M: f64 = 1.0 / ISOSTATIC_FACTOR - 1.0 / COLUMN_ISOSTATIC_FACTOR;

/// Books rock the surface took off a column's top (`taken_m` > 0) or laid on it (< 0) to its
/// deficit (see the module docs, ledger PLT-780).
pub fn take_from_top(deficit: &mut f32, taken_m: f64) {
    if taken_m.is_finite() {
        *deficit = (f64::from(*deficit) + taken_m * TOP_DEFICIT_PER_M) as f32;
    }
}

/// How much a metre of a submerged column's dry Airy height below the sea sinks it: its water.
pub const WATER_LOADED: f64 = RHO_MANTLE / (RHO_MANTLE - RHO_WATER);

/// What one step did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Settle {
    /// Σ of the rises and of the falls, m × km² (Earth-normalized).
    pub rise_km2_m: f64,
    pub fall_km2_m: f64,
}

/// The Airy height of each sample's own crust, m, over a fixed datum (`iso_height` at 0): the
/// step's change of it is the sample's crust uplift (`Sample::crust_uplift_mm_yr`).
pub fn crust_heights(state: &PlateState, gravity: f64) -> Vec<f64> {
    state
        .samples
        .par_iter()
        .map(|s| iso_height(&s.pools, 0.0, gravity))
        .collect()
}

/// Sets each sample's crust uplift to the change of its crust's Airy height since `before`
/// (from [`crust_heights`] on the same samples) over a step of `dt_yr` years, or with `add` adds
/// that change to it (a later pass of the same step).
// ledger: ISO-091
pub fn crust_uplift(state: &mut PlateState, before: &[f64], gravity: f64, dt_yr: f64, add: bool) {
    state
        .samples
        .par_iter_mut()
        .zip(before.par_iter())
        .for_each(|(s, &z)| {
            let rate = ((iso_height(&s.pools, 0.0, gravity) - z) / dt_yr * 1.0e3) as f32;
            s.crust_uplift_mm_yr = if add {
                s.crust_uplift_mm_yr + rate
            } else {
                rate
            };
        });
}

/// The height a column of dry Airy height `z_a` settles at under a sea at `sea`, m.
pub fn equilibrium_m(z_a: f64, sea: f64) -> f64 {
    if z_a >= sea {
        z_a
    } else {
        sea + (z_a - sea) * WATER_LOADED
    }
}

/// One plate step of the settling (see the module docs); `flexure` per sample, when the surface
/// stage keeps one.
pub fn step(state: &mut PlateState, env: &Env, flexure: Option<&[f64]>, sea: f64) -> Settle {
    let k = env.k;
    let tau = k.isostasy_relax_myr;
    if !(tau.is_finite() && tau > 0.0) || env.dt_myr <= 0.0 {
        return Settle::default();
    }
    let share = 1.0 - (-env.dt_myr / tau).exp();
    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
    let g = env.gravity;
    // The light rock the surface took off the top (ledger: PLT-780; none while off).
    let deficit: &[f32] = if k.settle_top_density {
        &state.top_deficit_m
    } else {
        &[]
    };
    // The changes are collected and summed in sample order: a parallel sum's order, so its last
    // bits, would change from run to run.
    let changes: Vec<f64> = state
        .samples
        .par_iter_mut()
        .enumerate()
        .map(|(i, s)| {
            if s.kind != CONTINENTAL || s.is_hidden() || s.flags & TRENCH != 0 {
                return 0.0;
            }
            let f = flexure
                .and_then(|f| f.get(i).copied())
                .filter(|f| f.is_finite())
                .unwrap_or(0.0);
            let d = deficit
                .get(i)
                .map(|&d| f64::from(d))
                .filter(|d| d.is_finite())
                .unwrap_or(0.0);
            // The lithosphere's thermal height joins the crust's (ledger: ISO-110; 0 when off).
            let z_a = iso_height(&s.pools, z_0, g) + thermal::uplift_m(k, s.lithosphere_km) - d;
            let target = equilibrium_m(z_a, sea) + f;
            let z = f64::from(s.elevation_m);
            let dz = (target - z) * share;
            s.elevation_m = (z + dz) as f32;
            dz
        })
        .collect();
    let (rise, fall) = changes.iter().fold((0.0, 0.0), |(r, f), &dz| {
        (r + dz.max(0.0), f + (-dz).max(0.0))
    });
    let area = env.norm.sample_area_km2();
    Settle {
        rise_km2_m: rise * area,
        fall_km2_m: fall * area,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// With the deficit a metre of rock off the top lowers the column's height by the upper
    /// crust's Airy share, 1 − 2,700 / 3,300 of it coming back, and a metre laid down undoes it.
    #[test]
    fn rock_off_the_top_lowers_the_column_by_the_upper_crusts_airy_share() {
        let mut d = 0.0f32;
        take_from_top(&mut d, 1000.0);
        let lowered = 1000.0 / COLUMN_ISOSTATIC_FACTOR + f64::from(d);
        assert!(
            (lowered - 1000.0 * (1.0 - 2700.0 / 3300.0)).abs() < 1e-3,
            "{lowered}"
        );
        assert!((f64::from(d) - 1000.0 * 130.0 / 3300.0).abs() < 1e-3, "{d}");
        take_from_top(&mut d, -1000.0);
        assert!(d.abs() < 1e-4);
        take_from_top(&mut d, f64::NAN);
        assert!(d.abs() < 1e-4);
    }

    #[test]
    fn a_dry_column_settles_at_its_airy_height_and_a_drowned_one_under_its_water() {
        assert_eq!(equilibrium_m(500.0, 0.0), 500.0);
        // 1 km of dry Airy height below the sea: 1.454 km under the water's load.
        assert!((equilibrium_m(-1000.0, 0.0) + 3300.0 / 2270.0 * 1000.0).abs() < 1e-9);
        assert!((equilibrium_m(-60.0, -60.0) + 60.0).abs() < 1e-9);
    }
}

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
//! −380 m against an Airy height of +855 m, and only 45 % of the continental crust is dry.
//!
//! # What happens on Earth
//! A column out of isostatic balance does not stay so: the mantle flows back within thousands
//! of years (the relaxation times of glacial rebound, 4 to 7 kyr: Mitrovica, Forte & Simons
//! 2000; Roy & Peltier 2015), except for what the lithosphere's flexural strength holds up or
//! down. What a cooled lithosphere adds to an ocean floor's depth goes on the lithosphere's
//! thermal time (McKenzie 1978's τ = 62.8 Myr): an arc built on old ocean floor stands as low
//! as its cold lithosphere makes it until that lithosphere warms or is replaced.
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
//! or below local isostasy; 0 without a surface stage), τ = `isostasy_relax_myr`. The model cannot tell which
//! part of a gap is mechanical (gone in kyr) and which is thermal, so it settles on the slower,
//! thermal time (ISO-078). The change is the step's uplift, which the surface stage takes as it
//! takes any uplift; the step's rise guard (PLT-288) runs after it.
// ledger: ISO-078 (the rule and τ), ISO-044 (ρ_w), ISO-043 (ρ_m).

use rayon::prelude::*;

use super::Env;
use super::collapse::iso_height;
use super::state::{CONTINENTAL, PlateState, TRENCH};
use crate::surface::flexure::{RHO_MANTLE, RHO_WATER};

/// `WorldState::globals`: the settling's rise and fall over the run, km³ of height × area.
pub const RISE_GLOBAL: &str = "plates/settle_rise_km3";
pub const FALL_GLOBAL: &str = "plates/settle_fall_km3";

/// How much a metre of a submerged column's dry Airy height below the sea sinks it: its water.
pub const WATER_LOADED: f64 = RHO_MANTLE / (RHO_MANTLE - RHO_WATER);

/// What one step did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Settle {
    /// Σ of the rises and of the falls, m × km² (Earth-normalized).
    pub rise_km2_m: f64,
    pub fall_km2_m: f64,
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
    let (rise, fall) = state
        .samples
        .par_iter_mut()
        .enumerate()
        .map(|(i, s)| {
            if s.kind != CONTINENTAL || s.is_hidden() || s.flags & TRENCH != 0 {
                return (0.0, 0.0);
            }
            let f = flexure
                .and_then(|f| f.get(i).copied())
                .filter(|f| f.is_finite())
                .unwrap_or(0.0);
            let target = equilibrium_m(iso_height(&s.pools, z_0, g), sea) + f;
            let z = f64::from(s.elevation_m);
            let dz = (target - z) * share;
            s.elevation_m = (z + dz) as f32;
            (dz.max(0.0), (-dz).max(0.0))
        })
        .reduce(|| (0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));
    let area = env.norm.sample_area_km2();
    Settle {
        rise_km2_m: rise * area,
        fall_km2_m: fall * area,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dry_column_settles_at_its_airy_height_and_a_drowned_one_under_its_water() {
        assert_eq!(equilibrium_m(500.0, 0.0), 500.0);
        // 1 km of dry Airy height below the sea: 1.454 km under the water's load.
        assert!((equilibrium_m(-1000.0, 0.0) + 3300.0 / 2270.0 * 1000.0).abs() < 1e-9);
        assert!((equilibrium_m(-60.0, -60.0) + 60.0).abs() < 1e-9);
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The fastest a column's crust thickens and its surface rises (ledger PLT-287, PLT-288).
//!
//! # Why
//! Each process of a plate step adds crust to columns by its own rule, and nothing bounded how
//! fast a column could gain it. A resampling's accretion (`ridges.rs`, PLT-284) gave the column on
//! top the crust of every footprint under it at once: up to 566 km of crust in one 2 Myr step,
//! and a surface rise of 25 to 36 km that stopped only at `max_elevation_m` (30 km), on every
//! resampling and on the forced one that ends a run (spike trace, build log 2026-10-18 "Plate
//! spikes"). The surface's flexure turned those single-step lifts into peaks of 50 to 100 km. The
//! collision wedge (`belts.rs`) filled thin crust in front of a front by 35 to 94 km in a step,
//! and its belt raised columns by up to 13 km.
//!
//! # What happens on Earth
//! Crust thickens where it shortens. Across a region of width W shortening at u, crust of
//! thickness H thickens at dH/dt = u·H / W (Garzione et al. 2006, eq. 2): with the Altiplano's
//! fastest long-term shortening, 10 mm/yr over 300 km of 50 km crust, about 1.7 mm/yr, and 8 km
//! in about 4 Myr (2 mm/yr). Southern Tibet thickened by about 40 km at about 1.3 mm/yr from 60
//! to 30 Ma (Sundell et al. 2021). These two, the thickest crust on Earth, were built at 1 to 2
//! mm/yr over their whole width; under Airy isostasy that raises the surface by a seventh of it
//! (PLT-281), a few tenths of a mm/yr. Garzione et al. (2006) found 2.5 to 3.5 km of surface
//! uplift of the Altiplano in 1 to 4 Myr, 0.6 to 3.5 mm/yr, too fast for crustal thickening, and
//! needed the removal of dense lower crust and mantle lithosphere to explain it.
//!
//! # The rule
//! When a step begins, every sample's thickness T₀ is kept; its ceiling is T₀ + ṫ·δt
//! (`thickening_max_mm_yr`, 5 mm/yr: about three times the fastest regional rate above, for one
//! column of the lattice over one step rather than a plateau over millions of years). After the
//! step's thickening processes (belts, arcs, the founder and the collapse, the transition, the
//! collisions, the hotspots) and again after a resampling (the ceilings blended with the
//! column's own weights), every visible column above its ceiling releases the excess: it moves
//! to the nearest room on the same plate within `collapse_reach_km`, as the collapse moves crust
//! (`collapse::spread`): crust up to `crust_max_km` and ocean floor up to the normal column, each
//! no higher than its own ceiling, so the orogen widens instead of piling up. What finds no room
//! goes down from the bottom of its column, as beyond `crust_max_km` (continental subduction,
//! the sediment and crust that subduction zones take down). The collapse and the founder fill
//! their receivers only up to the ceilings as well.
//!
//! A column that releases crust sinks by the Airy change of what it lost, but not below where it
//! stood when the step began (or, after a resampling, below the blended surface before the
//! accretion): the surface had not risen for crust the column never kept.
//!
//! No visible column ends either pass higher than its top, z₀ + ż·δt (`uplift_max_mm_yr`,
//! 3.5 mm/yr: the fastest of Garzione et al.'s (2006) Altiplano rates, which needed the loss of
//! the lithosphere's dense root): a process that lifts a column faster (a belt filling a
//! foreland, a column brought up to its Airy height at once) leaves it there, below its Airy
//! height, and later steps let it rise the rest. The surface stage's own changes (erosion,
//! rebound, flexure) are applied before the step begins and are not limited here.
//!
//! # What it is not
//! A strain-rate law: the excess is moved, not deformed, and the rate is one number for every
//! column, whatever its thickness or heat. Nor a limit on how fast a column sinks: crust that
//! founders or spreads away lowers its surface by its Airy change at once.

use std::collections::BTreeMap;

use super::Env;
use super::collapse::{self, Collapse};
use super::geometry::Geo;
use super::pools::isostatic_change_m;
use super::state::{PlateState, Sample};

/// `WorldState::globals`: crust beyond the ceilings over the run, km³ (Earth-normalized) …
pub const CAPPED_GLOBAL: &str = "plates/rate_capped_km3";
/// … and the part of it that found no room and went to the mantle.
pub const LOST_GLOBAL: &str = "plates/rate_lost_km3";
/// … and the surface held below the uplift's ceilings, km² × m / 1000.
pub const HELD_GLOBAL: &str = "plates/rate_held_km3";

/// A ceiling is passed when a column is thicker than it by more than this, m (`f32` sums).
const SLACK_M: f64 = 1.0;

/// The step's limits, by sample.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Limits {
    /// The thickest each column may be this step, m.
    pub ceiling_m: Vec<f32>,
    /// Where each column stood when the step began, m: a column releasing crust sinks no lower.
    pub floor_m: Vec<f32>,
    /// The highest each column may stand this step, m.
    pub top_m: Vec<f32>,
}

/// What one pass of [`limit`] did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Limited {
    /// Columns above their ceiling.
    pub columns: u32,
    /// Crust beyond the ceilings, km² × m …
    pub capped_km2_m: f64,
    /// … and what of it went to the mantle.
    pub lost_km2_m: f64,
    /// Columns that rose above their top …
    pub held: u32,
    /// … and by how much, km² × m.
    pub held_km2_m: f64,
}

/// The thickening one step allows a column, m; `None` when there is no limit.
pub fn allowance_m(env: &Env) -> Option<f64> {
    // ledger: PLT-287 (mm/yr × Myr = km).
    let a = env.k.thickening_max_mm_yr * env.dt_myr * 1000.0;
    (a.is_finite() && a > 0.0).then_some(a)
}

/// The rise one step allows a column's surface, m; `None` when there is no limit.
pub fn rise_m(env: &Env) -> Option<f64> {
    // ledger: PLT-288 (mm/yr × Myr = km).
    let r = env.k.uplift_max_mm_yr * env.dt_myr * 1000.0;
    (r.is_finite() && r > 0.0).then_some(r)
}

/// Sets the step's limits from the samples as the step begins (their elevations `z0`).
pub fn begin(state: &mut PlateState, env: &Env, z0: &[f32]) {
    let (a, r) = (allowance_m(env), rise_m(env));
    if a.is_none() && r.is_none() {
        state.limits = None;
        return;
    }
    let (a, r) = (a.unwrap_or(f64::INFINITY), r.unwrap_or(f64::INFINITY));
    state.limits = Some(Limits {
        ceiling_m: state
            .samples
            .iter()
            .map(|s| (f64::from(s.thickness_m.max(0.0)) + a) as f32)
            .collect(),
        floor_m: z0.to_vec(),
        top_m: z0.iter().map(|&z| (f64::from(z) + r) as f32).collect(),
    });
}

/// Brings every visible column back under its ceiling (see the module docs).
pub fn limit(state: &mut PlateState, geo: &Geo, env: &Env) -> Limited {
    let Some((ceiling, floor, top)) = state
        .limits
        .as_ref()
        .map(|l| (l.ceiling_m.clone(), l.floor_m.clone(), l.top_m.clone()))
    else {
        return Limited::default();
    };
    let n = state.samples.len();
    if ceiling.len() != n || floor.len() != n || top.len() != n {
        return Limited::default();
    }
    let over = |s: &Sample, i: usize| {
        let x = f64::from(s.thickness_m) - f64::from(ceiling[i]);
        (!s.is_hidden() && x > SLACK_M).then_some(x)
    };
    let mut out = Limited::default();
    let mut by_plate: BTreeMap<u32, Vec<(usize, f64)>> = BTreeMap::new();
    for i in 0..n {
        if let Some(x) = over(&state.samples[i], i) {
            out.columns += 1;
            out.capped_km2_m += x;
            by_plate.entry(state.plate_of(i)).or_default().push((i, x));
        }
    }
    if !by_plate.is_empty() {
        release(state, geo, env, by_plate, &floor, &mut out, over);
    }
    // No surface above its top: the column stands below its Airy height until later steps let
    // it rise (the gap budget counts it).
    for (i, s) in state.samples.iter_mut().enumerate() {
        let x = f64::from(s.elevation_m) - f64::from(top[i]);
        if !s.is_hidden() && x > SLACK_M {
            s.elevation_m = top[i];
            out.held += 1;
            out.held_km2_m += x;
        }
    }
    let area = env.norm.sample_area_km2();
    out.capped_km2_m *= area;
    out.lost_km2_m *= area;
    out.held_km2_m *= area;
    out
}

/// Moves the crust above the ceilings to room, or down to the mantle.
fn release(
    state: &mut PlateState,
    geo: &Geo,
    env: &Env,
    by_plate: BTreeMap<u32, Vec<(usize, f64)>>,
    floor: &[f32],
    out: &mut Limited,
    over: impl Fn(&Sample, usize) -> Option<f64>,
) {
    // The room: crust up to the thickest column, ocean floor up to the normal one, each under
    // its own ceiling (`collapse::spread` reads them from the state).
    let k = env.k;
    let to = if k.crust_max_km.is_finite() && k.crust_max_km > 0.0 {
        (k.crust_max_km * 1000.0).max(collapse::normal_thickness_m())
    } else {
        f64::INFINITY
    };
    let mut moved = Collapse::default();
    collapse::spread(
        state,
        geo,
        env,
        by_plate,
        collapse::RoomTo {
            continental_m: to,
            ocean_m: collapse::normal_thickness_m(),
        },
        &mut moved,
        Some(floor),
    );
    // What found no room goes down from the bottom of its column.
    for (i, s) in state.samples.iter_mut().enumerate() {
        let Some(x) = over(s, i) else { continue };
        let before = s.pools;
        out.lost_km2_m += collapse::sink(s, x);
        let z = f64::from(s.elevation_m);
        let change = isostatic_change_m(&before, &s.pools, env.gravity);
        s.elevation_m = collapse::held(z, change, f64::from(floor[i])) as f32;
    }
}

/// Clears the step's limits (the step is over).
pub fn end(state: &mut PlateState) {
    state.limits = None;
}

#[cfg(test)]
mod tests {
    use glam::DVec3;

    use super::*;
    use crate::params::TectonicParams;
    use crate::plates::geometry::positions;
    use crate::plates::normalise::Norm;
    use crate::plates::pools::{COLUMN_ISOSTATIC_FACTOR, continental_column};
    use crate::plates::state::{CONTINENTAL, Group, Plate, pool};

    const N: usize = 16_000;

    fn env(k: &TectonicParams) -> Env<'_> {
        Env {
            k,
            norm: Norm::new(6370.0, 6371.0, N),
            dt_myr: 2.0,
            dt_yr: 2.0e6,
            gravity: 9.81,
            step: 1,
            t_myr: 2.0,
        }
    }

    /// One plate, all continent (35.1 km at 300 m).
    fn planet(geo: &Geo) -> PlateState {
        let samples: Vec<Sample> = geo
            .points()
            .iter()
            .map(|_| {
                let mut s = Sample::blank(0);
                s.kind = CONTINENTAL;
                s.pools = continental_column();
                s.elevation_m = 300.0;
                s.thickness_m = s.pools.iter().sum();
                s
            })
            .collect();
        let ident = [0.0, 0.0, 0.0, 1.0];
        let mut st = PlateState {
            n: N as u32,
            lattice: ident,
            samples,
            groups: vec![Group {
                plate: 0,
                rot: ident,
            }],
            plates: vec![Plate {
                id: 0,
                alive: true,
                omega: [0.0; 3],
                born_myr: 0.0,
                died_myr: None,
                birth_event: None,
                parent: None,
                lip_until_myr: 0.0,
                lip_event: None,
                samples: 0,
                centroid: [0.0, 1.0, 0.0],
                continental_frac: 1.0,
            }],
            contacts: Vec::new(),
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
            shown: None,
            one_way: None,
            propagations: Vec::new(),
            level_set: None,
        };
        let pos = positions(&st, geo);
        st.recount(&pos);
        st
    }

    /// A process stacks 20 km on a small patch in one step (and raises it by its Airy change): the
    /// patch keeps one step's allowance, the rest widens it, and the surface rises by about the
    /// allowance's Airy height.
    #[test]
    fn no_column_ends_a_step_thicker_than_its_ceiling() {
        let k = TectonicParams::default();
        let env = env(&k);
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let mut state = planet(&geo);
        let patch: Vec<bool> = geo
            .points()
            .iter()
            .map(|p| p.dot(DVec3::Y) > 0.999)
            .collect();
        assert!(patch.iter().filter(|&&p| p).count() >= 3);
        let z0: Vec<f32> = state.samples.iter().map(|s| s.elevation_m).collect();
        begin(&mut state, &env, &z0);
        let a = allowance_m(&env).expect("on by default");
        assert!(
            (a - 10_000.0).abs() < 1e-6,
            "5 mm/yr over 2 Myr is 10 km, not {a}"
        );
        let added = 20_000.0f32;
        for (s, &p) in state.samples.iter_mut().zip(&patch) {
            if p {
                let before = s.pools;
                s.pools[pool::METAMORPHIC] += added;
                s.thickness_m = s.pools.iter().sum();
                s.elevation_m += isostatic_change_m(&before, &s.pools, env.gravity) as f32;
            }
        }
        let total =
            |st: &PlateState| -> f64 { st.samples.iter().map(|s| f64::from(s.thickness_m)).sum() };
        let before = total(&state);
        let done = limit(&mut state, &geo, &env);
        assert!(done.columns > 0);
        let limits = state.limits.as_ref().expect("kept");
        let mut highest = f64::MIN;
        for (i, s) in state.samples.iter().enumerate() {
            assert!(
                f64::from(s.thickness_m) <= f64::from(limits.ceiling_m[i]) + SLACK_M,
                "sample {i}: {} m over a ceiling of {} m",
                s.thickness_m,
                limits.ceiling_m[i]
            );
            highest = highest.max(f64::from(s.elevation_m) - 300.0);
        }
        let airy = a / COLUMN_ISOSTATIC_FACTOR;
        assert!(
            highest > 0.5 * airy && highest < 1.5 * airy,
            "the patch rose {highest} m, the allowance's Airy height is {airy} m"
        );
        // Conserved but for what went down; here every column had room.
        let lost = done.lost_km2_m / env.norm.sample_area_km2();
        assert!(lost < 1.0, "{lost} m went down with room all around");
        assert!((before - total(&state) - lost).abs() < 1e-6 * before);
        let widened = state
            .samples
            .iter()
            .zip(&patch)
            .filter(|(s, p)| !**p && s.thickness_m > 35_200.0)
            .count();
        assert!(widened > 0, "the excess spread to no neighbour");
        end(&mut state);
        assert!(state.limits.is_none());
    }

    /// A process lifts a column by 10 km without thickening it: it stands at one step's rise
    /// above where it began, and a column that rose less keeps its height.
    #[test]
    fn no_surface_rises_above_its_top() {
        let k = TectonicParams::default();
        let env = env(&k);
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let mut state = planet(&geo);
        let z0: Vec<f32> = state.samples.iter().map(|s| s.elevation_m).collect();
        begin(&mut state, &env, &z0);
        let r = rise_m(&env).expect("on by default");
        assert!(
            (r - 7_000.0).abs() < 1e-6,
            "3.5 mm/yr over 2 Myr is 7 km, not {r}"
        );
        state.samples[0].elevation_m += 10_000.0;
        state.samples[1].elevation_m += 5_000.0;
        let done = limit(&mut state, &geo, &env);
        assert_eq!((done.columns, done.held), (0, 1));
        assert!((f64::from(state.samples[0].elevation_m) - (300.0 + r)).abs() < 1.0);
        assert_eq!(state.samples[1].elevation_m, 5_300.0);
        let held = done.held_km2_m / env.norm.sample_area_km2();
        assert!((held - 3_000.0).abs() < 1.0, "{held} m held, not 3000");
    }

    /// With both rates at 0 nothing is limited.
    #[test]
    fn a_zero_rate_limits_nothing() {
        let k = TectonicParams {
            thickening_max_mm_yr: 0.0,
            uplift_max_mm_yr: 0.0,
            ..TectonicParams::default()
        };
        let env = env(&k);
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let mut state = planet(&geo);
        let z0: Vec<f32> = state.samples.iter().map(|s| s.elevation_m).collect();
        begin(&mut state, &env, &z0);
        assert!(state.limits.is_none());
        state.samples[0].pools[pool::METAMORPHIC] += 50_000.0;
        state.samples[0].thickness_m += 50_000.0;
        assert_eq!(limit(&mut state, &geo, &env), Limited::default());
        assert!(state.samples[0].thickness_m > 85_000.0);
    }
}

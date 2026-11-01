// SPDX-License-Identifier: LGPL-3.0-or-later
//! Plates and crust (plan §8.1, work item P1-03): the procedural tectonics of Cortial, Peytavie,
//! Galin & Guérin 2019, "Procedural Tectonic Planets" (Computer Graphics Forum 38(2);
//! https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf), implemented from the paper
//! (no licensed code exists, and the unlicensed community ports were not opened), with rock pools
//! ported from tectonics.js (CC BY 4.0, `pools.rs`) and mantle-fixed hotspots.
//!
//! # The model
//! The crust is point-sampled on L0, a Fibonacci sphere (P1-02), triangulated by P1-02's hull and
//! partitioned into plates. Plates move rigidly about Euler poles (`kinematics.rs`); each 2-Myr
//! step then
//! 1. finds where plates overlap and decides who subducts (`subduction.rs`), hiding the
//!    subducting plate's crust inside the overriding one;
//! 2. uplifts the overriding plate within r_s of its fronts (Cortial's u_0·f(d)·g(v)·h(z̃)), builds
//!    arcs, trenches and the deposit accumulators; then sums the forces on every plate (slab
//!    pull, suction, ridge push and the weight of high ground, drag, slab, boundary and collision
//!    resistance, a weak mantle flow) and solves their torque balance (`forces.rs`);
//! 3. keeps each contact's collision state and real shortening (`shortening.rs`), stacks the
//!    crust each collision shortened into a belt and a wedge (`belts.rs`), and welds two plates
//!    whose collision has stalled, or moves a small terrane to the other plate once it has
//!    shortened by the threshold (`collision.rs`); thickened crust that nothing raises any more
//!    collapses, spreading into the nearest thin crust of its plate and turning the plate's ocean
//!    floor beside the continent continental (`collapse.rs`);
//! 4. runs hotspots and large igneous provinces (`hotspots.rs`);
//! 5. applies continental erosion, oceanic dampening and trench accretion, and the rock-pool
//!    processes (`pools.rs`);
//!    rifts whose sides never moved apart heal (`rifting::heal`); the next step's motion relaxes
//!    towards the balance's rotation (`motion_relax_myr`), under a speed guard that is reported
//!    whenever it binds.
//! 6. draws Poisson rifts that split plates into 2–4 fragments, and subduction initiations that
//!    split old oceanic crust off to sink (`rifting.rs`);
//! 7. every 10–60 steps resamples the crust onto a fresh lattice, making oceanic crust in the gaps
//!    between diverging plates (`ridges.rs`);
//! 8. samples the crust onto L1 through the coupling for keyframes and later stages (`l1.rs`).
//!
//! Events (plan §5.5) go to the world's log with their causes (`events.rs`); lengths are
//! Earth-normalized (`normalise.rs`). Design choices beyond the paper are listed in each module's
//! docs and in STATUS.md's P1-03 entry.
//!
//! # Parallelism and determinism
//! Per-sample work (overlap detection, uplift, surface processes, resampling interpolation, L1
//! sampling) runs on the session's Rayon pool with disjoint writes; distance fields (Dijkstra),
//! contacts, collisions, rifts and events are serial. Every reduction is serial or index-ordered,
//! and randomness comes from one seeded generator in the state, so a run (and a run resumed from
//! a checkpoint) is reproducible.

pub mod belts;
pub mod check;
pub mod collapse;
pub mod collision;
pub mod crust_budget;
pub mod cut_trace;
pub mod dock_tally;
pub mod dynamics;
pub mod events;
pub mod flow;
pub mod forces;
pub mod front_trace;
pub mod gap_budget;
pub mod geometry;
pub mod hotspots;
pub mod inherit;
pub mod init;
pub mod kinematics;
pub mod l1;
pub mod level_set;
pub mod noise;
pub mod normalise;
pub mod peaks;
pub mod pools;
pub mod propagation;
pub mod rate;
pub mod ridges;
pub mod rifting;
pub mod settle;
pub mod shortening;
pub mod shown;
pub mod state;
pub mod stats;
pub mod subduction;
pub mod thermal;
pub mod transition;

use std::collections::BTreeMap;

use deeptime_format::{EventId, GridDesc};
use glam::DVec3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub use state::PlateState;

use crate::error::CoreError;
use crate::params::{PlanetPreset, TectonicParams};
use crate::sphere::{CellLink, Coupling, SampleTree};
use crate::stage::{Stage, StageContext, StageEstimate, from_state, to_state};
use crate::world::{Field, WorldState};
use geometry::{Geo, Tree};
use normalise::Norm;
use pools::{SurfaceRates, ocean_heat_flow, surface_step};
use state::{
    ARC, CONTINENTAL, DIVERGENT, EMERGED, HOTSPOT, OCEANIC, OROGENY_ANDEAN, OROGENY_HIMALAYAN,
    SUBDUCTION_ZONE, TRENCH, pool, regime,
};

/// The stage's name in a pipeline (`deeptime stage plates`).
pub const STAGE_NAME: &str = "plates";

/// The fields drawn into keyframes: elevation, plates and crust age (plan §8.1), and the regime.
pub const KEYFRAME_LAYERS: [&str; 4] = ["elevation", "plates", "crust_age", "regime"];

/// What every process of one step sees.
pub struct Env<'a> {
    pub k: &'a TectonicParams,
    pub norm: Norm,
    pub dt_myr: f64,
    pub dt_yr: f64,
    pub gravity: f64,
    /// The number of the step being taken (the first is 1).
    pub step: u64,
    /// The time at the end of this step, Myr.
    pub t_myr: f64,
}

struct Cache {
    geo: Geo,
    coupling: Option<Coupling>,
}

/// Wall time of the plate step's parts, milliseconds summed over the steps since the last
/// [`reset_timings`] (P1-17: `deeptime bench` prints them). A process-wide gauge rather than a
/// world note, so the world a run leaves is the same with or without it.
static TIMINGS: std::sync::Mutex<Vec<(&'static str, f64)>> = std::sync::Mutex::new(Vec::new());

/// The plate step's parts and their summed wall time (ms), in the order they first ran.
pub fn timings() -> Vec<(&'static str, f64)> {
    TIMINGS.lock().map(|t| t.clone()).unwrap_or_default()
}

/// Clears [`timings`] (a new run in the same process).
pub fn reset_timings() {
    if let Ok(mut t) = TIMINGS.lock() {
        t.clear();
    }
}

/// Adds the time since `*since` to `what` and restarts the clock.
fn lap(since: &mut std::time::Instant, what: &'static str) {
    let now = std::time::Instant::now();
    let ms = (now - *since).as_secs_f64() * 1000.0;
    *since = now;
    crate::compute::cpu_part(STAGE_NAME, what);
    if let Ok(mut t) = TIMINGS.lock() {
        match t.iter_mut().find(|e| e.0 == what) {
            Some(e) => e.1 += ms,
            None => t.push((what, ms)),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Saved {
    done: u64,
    next_output_years: f64,
}

/// The plates stage.
pub struct PlatesStage {
    preset: PlanetPreset,
    total: u64,
    done: u64,
    /// The next keyframe time, on the session's grid (plan §6.1).
    next_output_years: f64,
    cache: Option<Cache>,
    /// Whether to record the plate dynamics (`dynamics.rs`). On, unless `DEEPTIME_NO_PLATE_DYNAMICS`
    /// is set (to time what it costs); the tests that show recording changes nothing turn it off.
    record_dynamics: bool,
    /// Whether to keep the land's gap budget (`gap_budget.rs`, measurement only). Off, unless
    /// `DEEPTIME_GAP_BUDGET` is set.
    record_gap_budget: bool,
    /// The continental crust budget (`crust_budget.rs`, measurement only): kept while
    /// `DEEPTIME_CRUST_BUDGET` is set, started at the first step.
    record_crust_budget: bool,
    crust_budget: Option<crust_budget::CrustBudget>,
    /// The front trace (`front_trace.rs`, measurement only): kept while `DEEPTIME_FRONT_TRACE`
    /// names a folder.
    front_trace: Option<front_trace::FrontTrace>,
}

/// `WorldState::notes` key of the continental crust budget's lines (`crust_budget.rs`, when on).
pub const CRUST_BUDGET_NOTE: &str = "crust_budget";

impl PlatesStage {
    pub fn new(preset: PlanetPreset) -> PlatesStage {
        let total = preset.plate_steps();
        let next_output_years = preset.t_start_years();
        PlatesStage {
            preset,
            total,
            done: 0,
            next_output_years,
            cache: None,
            record_dynamics: std::env::var_os(dynamics::OFF_ENV).is_none(),
            record_gap_budget: gap_budget::enabled(),
            record_crust_budget: crust_budget::enabled(),
            crust_budget: None,
            front_trace: front_trace::env_dir().map(front_trace::FrontTrace::new),
        }
    }

    /// The same stage writing the front trace (`front_trace.rs`) into `dir`: the world it makes
    /// is the same.
    pub fn with_front_trace(mut self, dir: std::path::PathBuf) -> PlatesStage {
        self.front_trace = Some(front_trace::FrontTrace::new(dir));
        self
    }

    /// The same stage keeping the continental crust budget (`crust_budget.rs`): the world it
    /// makes is the same, but for the budget's note.
    pub fn with_crust_budget(mut self) -> PlatesStage {
        self.record_crust_budget = true;
        self
    }

    /// The same stage keeping the land's gap budget (`gap_budget.rs`): the world it makes is the
    /// same.
    pub fn with_gap_budget(mut self) -> PlatesStage {
        self.record_gap_budget = true;
        self
    }

    /// The same stage without the dynamics record (`dynamics.rs`): the world it makes is the same.
    pub fn without_dynamics(mut self) -> PlatesStage {
        self.record_dynamics = false;
        self
    }

    fn dt_myr(&self) -> f64 {
        self.preset.dt_myr()
    }

    fn norm(&self) -> Norm {
        Norm::new(
            self.preset.tectonics.model_radius_km,
            self.preset.scale.radius_m() / 1000.0,
            self.preset.resolution.l0_samples as usize,
        )
    }

    fn env<'a>(&self, k: &'a TectonicParams, step: u64) -> Env<'a> {
        let dt_myr = self.dt_myr();
        Env {
            k,
            norm: self.norm(),
            dt_myr,
            dt_yr: dt_myr * 1.0e6,
            gravity: self.preset.planet.gravity_m_s2,
            step,
            t_myr: self.preset.t_start_years() / 1.0e6 + step as f64 * dt_myr,
        }
    }

    fn l1_grid(&self) -> Result<GridDesc, CoreError> {
        Ok(GridDesc::for_planet(
            "L1",
            self.preset.scale.circumference_blocks,
            self.preset.resolution.l1_r,
        )?)
    }

    /// One plate step on `st` (taken out of the world while it runs).
    fn advance(&mut self, st: &mut PlateState, world: &mut WorldState) -> Result<(), CoreError> {
        let mut clock = std::time::Instant::now();
        crate::compute::cpu_mark(STAGE_NAME);
        if self
            .cache
            .as_ref()
            .is_none_or(|c| c.geo.lattice != st.lattice)
        {
            self.cache = Some(Cache {
                geo: Geo::build(st.n as usize, st.lattice)?,
                coupling: None,
            });
        }
        let step = st.step + 1;
        let tectonics = self.preset.tectonics.clone();
        let env = self.env(&tectonics, step);
        let k = env.k;
        // The surface's flexure under the samples, for the foreland basins of the collisions'
        // record (`belts.rs`); measurement only. Sampled beside the feedback where it can be.
        let collisions = st.contacts.iter().any(|c| c.collision.is_some());
        // The continental crust budget (`crust_budget.rs`, measurement only), taken out of the
        // stage for the step; its first snapshot is the run's start.
        let mut cb = self.crust_budget.take();
        // The sea the last step stood under: the climate stage's, else the one the surface
        // filled itself (`sea_level.rs`; without a climate stage), else the datum (the first
        // step, or the plates alone). One sea for both stages (ledger: ISO-086).
        let sea_m = |world: &WorldState| {
            world
                .globals
                .get(crate::climate::SEA_LEVEL_GLOBAL)
                .copied()
                .filter(|z| z.is_finite())
                .or_else(|| world.surface.as_ref().map(|s| s.sea_level_m))
                .filter(|z| z.is_finite())
                .unwrap_or(0.0)
        };
        if self.record_crust_budget && cb.is_none() {
            let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
            let mut b = crust_budget::CrustBudget::start(&st.samples, z_0, env.gravity);
            b.dock.start(st.t_myr);
            let geo = &self.cache.as_ref().expect("built above").geo;
            let pos0 = geometry::positions(st, geo);
            b.snapshot(
                &st.samples,
                &pos0,
                st.t_myr,
                env.norm.sample_area_km2(),
                sea_m(world),
                &world.globals,
                None,
            );
            b.write_plates(&st.samples, |i| st.plate_of(i), st.t_myr);
            b.write_ages(&st.samples, st.t_myr);
            cb = Some(b);
        }
        // The settling (`settle.rs`) reads the flexure too.
        let settles = k.isostasy_relax_myr.is_finite() && k.isostasy_relax_myr > 0.0;
        // The peak trace (`peaks.rs`, measurement only, off by default).
        let tracing = peaks::threshold().is_some();
        let mut traced = peaks::Begin::default();
        if tracing {
            traced.z_pre = st.samples.iter().map(|s| s.elevation_m).collect();
        }
        let (mut cones, flexure, held) =
            self.apply_surface_feedback(st, world, &env, collisions || settles)?;
        if tracing {
            traced.held = held.unwrap_or_default();
        }
        cones.resize(st.samples.len(), [0.0; 2]);
        if let Some(b) = cb.as_mut() {
            b.book(&st.samples, gap_budget::Part::Erosion);
        }
        // The front trace (`front_trace.rs`, measurement only), taken out of the stage for the
        // step as the crust budget is.
        let mut ft = self.front_trace.take();
        if let Some(f) = ft.as_mut() {
            f.book(&st.samples, gap_budget::Part::Erosion, env.t_myr);
        }
        let ft = std::cell::RefCell::new(ft);
        let cb = std::cell::RefCell::new(cb);
        lap(&mut clock, "feedback");
        // The land's gap budget (`gap_budget.rs`, measurement only): each process's change of the
        // gap between the samples' elevation and their columns' Airy height; the crust budget's
        // change of the continental stocks, on the same brackets.
        let book = |st: &mut PlateState, part: gap_budget::Part| {
            if let Some(gb) = st.gap_budget.as_mut() {
                gb.book(&st.samples, part);
            }
            if let Some(b) = cb.borrow_mut().as_mut() {
                b.book(&st.samples, part);
            }
            if let Some(f) = ft.borrow_mut().as_mut() {
                f.book(&st.samples, part, env.t_myr);
            }
        };
        let flexure = if collisions || settles {
            flexure.or_else(|| self.sample_flexure(world))
        } else {
            None
        };
        let sea = sea_m(world);
        let z0: Vec<f32> = st.samples.iter().map(|s| s.elevation_m).collect();
        if tracing {
            traced.z0 = z0.clone();
            traced.flexure = flexure
                .clone()
                .unwrap_or_else(|| vec![f64::NAN; st.samples.len()]);
        }
        // The crust's own Airy heights as the step begins (its crust uplift, ledger: ISO-091).
        let crust0 = settle::crust_heights(st, env.gravity);
        if let Some(gb) = st.gap_budget.as_mut() {
            gb.begin_step(&st.samples);
        }
        // The fastest a column may thicken this step (`rate.rs`).
        rate::begin(st, &env, &z0);
        lap(&mut clock, "flexure_sample");

        // Motion, then interactions at the new positions.
        kinematics::advance(st, env.dt_yr);
        book(st, gap_budget::Part::Other);
        let cache = self.cache.as_mut().expect("built above");
        let mut pos = geometry::positions(st, &cache.geo);
        st.recount(&pos);
        let tree = Tree::over(&pos, |_| true);
        lap(&mut clock, "motion");
        cut_trace::mark(st, &cache.geo, "motion");
        // ledger: PLT-553 (the overlaps read the outline the resampling gives the ground by)
        let overlaps = subduction::overlaps(st, &cache.geo, &pos, &tree, &env);
        book(st, gap_budget::Part::Other);
        lap(&mut clock, "detect");
        let resolved = subduction::resolve(st, world, &cache.geo, &overlaps, &pos, &env)?;
        book(st, gap_budget::Part::Subduction);
        lap(&mut clock, "resolve");
        cut_trace::mark(st, &cache.geo, "resolve");
        let (fronts, fired) =
            shortening::advance(st, &cache.geo, &pos, &tree, &env, resolved.seeds);
        book(st, gap_budget::Part::Shortening);
        lap(&mut clock, "shortening");
        cut_trace::mark(st, &cache.geo, "shortening");
        // The arcs' fronts and distance field read nothing the belts change: found beside them
        // (P1-19).
        let begun = subduction::fronts_begin(st, &cache.geo, &pos, &tree, &env);
        let traced_fronts = ft.borrow().as_ref().map(|_| begun.traced_fronts());
        let geo = &cache.geo;
        let (stacked, arcs) = rayon::join(
            || {
                belts::stack(
                    st,
                    geo,
                    &pos,
                    &env,
                    &fronts,
                    flexure.as_deref().filter(|_| collisions),
                    sea,
                )
            },
            || subduction::fronts_reach(geo, &begun, &env),
        );
        book(st, gap_budget::Part::Belts);
        // The fronts by what stands on either side (`dock_tally.rs`, measurement only).
        if let Some(b) = cb.borrow_mut().as_mut() {
            for s in stacked.values() {
                // The no-room split by the belt's class (PLT-701), over the whole run.
                b.dock.add_noroom(
                    s.belt_class.index(),
                    s.length_km * env.dt_myr,
                    [
                        s.crossed_km2 * s.thickness_m,
                        s.sliver_km2_m,
                        s.capped_km2_m,
                        s.stays_km2_m,
                    ]
                    .map(|x| x / 1000.0),
                );
            }
            for (&c, s) in &stacked {
                if let Some(col) = st.contacts[c].collision.as_ref() {
                    b.dock.add(
                        env.t_myr,
                        env.t_myr - col.onset_myr,
                        s.length_km * env.dt_myr,
                        col.convergence_mm_yr,
                        &s.sides,
                    );
                }
            }
        }
        // The collisions' crust budget: what crossed their fronts and what the belts and wedges
        // kept of it, km³ (Earth-normalized).
        let (crossed, kept) = stacked.values().fold((0.0, 0.0), |a, s| {
            (
                a.0 + s.crossed_km2 * s.thickness_m / 1000.0,
                a.1 + (s.belt_km2_m + s.wedge_km2_m + s.upper_km2_m) / 1000.0,
            )
        });
        *world
            .globals
            .entry(belts::CROSSED_GLOBAL.to_owned())
            .or_insert(0.0) += crossed;
        *world
            .globals
            .entry(belts::KEPT_GLOBAL.to_owned())
            .or_insert(0.0) += kept;
        // The crossing's parts (measurement only, CF-1).
        let dt = env.dt_myr;
        for (name, x) in [
            (
                belts::CROSSED_AREA_GLOBAL,
                stacked.values().map(|s| s.crossed_km2).sum::<f64>(),
            ),
            (
                belts::FRONT_GLOBAL,
                stacked.values().map(|s| s.length_km * dt).sum::<f64>(),
            ),
            (
                belts::ABOVE_GLOBAL,
                stacked
                    .values()
                    .map(|s| s.crossed_km2 * s.above_m / 1000.0)
                    .sum::<f64>(),
            ),
            (
                belts::RECENT_GLOBAL,
                stacked
                    .values()
                    .map(|s| s.crossed_km2 * s.recent_above_m / 1000.0)
                    .sum::<f64>(),
            ),
            (
                belts::WEDGE_GLOBAL,
                stacked
                    .values()
                    .map(|s| s.wedge_km2_m / 1000.0)
                    .sum::<f64>(),
            ),
            (
                belts::RESTACK_GLOBAL,
                stacked
                    .values()
                    .map(|s| s.crossed_km2 * s.restack_m / 1000.0)
                    .sum::<f64>(),
            ),
            (
                belts::SLIVER_GLOBAL,
                stacked
                    .values()
                    .map(|s| s.sliver_km2_m / 1000.0)
                    .sum::<f64>(),
            ),
            (
                belts::CAPPED_GLOBAL,
                stacked
                    .values()
                    .map(|s| s.capped_km2_m / 1000.0)
                    .sum::<f64>(),
            ),
            (
                belts::CLIPPED_GLOBAL,
                stacked
                    .values()
                    .map(|s| s.clipped_km2_m / 1000.0)
                    .sum::<f64>(),
            ),
            (
                belts::TOE_GLOBAL,
                stacked.values().map(|s| s.toe_km2_m / 1000.0).sum::<f64>(),
            ),
        ] {
            *world.globals.entry(name.to_owned()).or_insert(0.0) += x;
        }
        // The crust terranes docking stripped sent down (PLT-770, measurement only; written only
        // with the switch so a run without it keeps its globals list).
        if env.k.terrane_docking_cover {
            *world
                .globals
                .entry(belts::STRIPPED_GLOBAL.to_owned())
                .or_insert(0.0) += stacked
                .values()
                .map(|s| s.stripped_km2_m / 1000.0)
                .sum::<f64>();
        }
        // The crust the belts had no room for and kept (PLT-700, measurement only; written only
        // with the switch so a run without it keeps its globals list).
        if env.k.collision_noroom_stays {
            *world
                .globals
                .entry(belts::STAYS_GLOBAL.to_owned())
                .or_insert(0.0) += stacked
                .values()
                .map(|s| s.stays_km2_m / 1000.0)
                .sum::<f64>();
        }
        // The two wedges' retro plateau beyond the belt's reach (PLT-740, measurement only; written
        // only with the switch so a run without it keeps its globals list).
        if env.k.collision_retro_plateau {
            *world
                .globals
                .entry(belts::PLATEAU_GLOBAL.to_owned())
                .or_insert(0.0) += stacked
                .values()
                .map(|s| s.plateau_km2_m / 1000.0)
                .sum::<f64>();
        }
        // The overriding plate's share (PLT-652, measurement only; written only with the switch so
        // a run without it keeps its globals list).
        if env.k.collision_upper_plate_shortening {
            for (name, x) in [
                (
                    belts::UPPER_GLOBAL,
                    stacked
                        .values()
                        .map(|s| s.upper_km2_m / 1000.0)
                        .sum::<f64>(),
                ),
                (
                    belts::UPPER_AREA_GLOBAL,
                    stacked
                        .values()
                        .map(|s| s.crossed_km2 * s.upper_share)
                        .sum::<f64>(),
                ),
                (
                    belts::UPPER_MOST_GLOBAL,
                    stacked
                        .values()
                        .filter(|s| s.upper_share >= 0.5)
                        .map(|s| s.crossed_km2)
                        .sum::<f64>(),
                ),
            ] {
                *world.globals.entry(name.to_owned()).or_insert(0.0) += x;
            }
        }
        lap(&mut clock, "belts");
        cut_trace::mark(st, &cache.geo, "belts");
        let pull = subduction::fronts_finish(st, &cache.geo, &env, &mut cones, begun, arcs, sea);
        book(st, gap_budget::Part::Arcs);
        *world
            .globals
            .entry(subduction::ARC_CONTINENTAL_GLOBAL.to_owned())
            .or_insert(0.0) += pull.arc_continental_km3;
        *world
            .globals
            .entry(subduction::ARC_OCEANIC_GLOBAL.to_owned())
            .or_insert(0.0) += pull.arc_oceanic_km3;
        for (name, x) in [
            (
                subduction::ARC_FRONT_CONTINENTAL_GLOBAL,
                pull.front_continental_km_myr,
            ),
            (
                subduction::ARC_FRONT_OCEANIC_GLOBAL,
                pull.front_oceanic_km_myr,
            ),
            (subduction::CORDILLERA_GLOBAL, pull.cordillera_km3),
            (subduction::FRONT_G_FULL_GLOBAL, pull.front_g_full_km_myr),
            (
                subduction::FRONT_G_NORMAL_GLOBAL,
                pull.front_g_normal_km_myr,
            ),
            (
                subduction::FRONT_G_CONTINENTAL_FULL_GLOBAL,
                pull.front_g_continental_km_myr[0],
            ),
            (
                subduction::FRONT_G_CONTINENTAL_NORMAL_GLOBAL,
                pull.front_g_continental_km_myr[1],
            ),
            (
                subduction::ARC_FRONT_STRAIGHT_GLOBAL,
                pull.front_straight_km_myr,
            ),
            // ledger: PLT-665 (measurement only)
            (
                subduction::ARC_FRONT_NORMAL_CONTINENTAL_GLOBAL,
                pull.front_normal_km_myr[0],
            ),
            (
                subduction::ARC_FRONT_NORMAL_OCEANIC_GLOBAL,
                pull.front_normal_km_myr[1],
            ),
            // ledger: PLT-670 (measurement only)
            (
                subduction::ARC_FRONT_YOUNG_CONTINENTAL_GLOBAL,
                pull.front_young_km_myr[0],
            ),
            (
                subduction::ARC_FRONT_YOUNG_OCEANIC_GLOBAL,
                pull.front_young_km_myr[1],
            ),
            // ledger: PLT-554 (measurement only)
            (subduction::ARC_BAND_GLOBAL, pull.arc_band_km_myr),
            // ledger: PLT-761, PLT-760 (measurement only)
            (
                subduction::ARC_CONES_CONTINENTAL_GLOBAL,
                pull.arc_cones_km3[0],
            ),
            (subduction::ARC_CONES_OCEANIC_GLOBAL, pull.arc_cones_km3[1]),
            // ledger: PLT-763
            (subduction::ARC_SUTURE_GLOBAL, pull.arc_suture_km3),
            (
                subduction::ARC_FRONT_INTRA_OCEANIC_GLOBAL,
                pull.front_intra_oceanic_km_myr,
            ),
            (subduction::SUTURE_FRONT_GLOBAL, pull.suture_front_km_myr[0]),
            (
                subduction::SUTURE_FRONT_ACTIVE_GLOBAL,
                pull.suture_front_km_myr[1],
            ),
        ] {
            *world.globals.entry(name.to_owned()).or_insert(0.0) += x;
        }
        // Measurement only (the crust-budget lane's trace of PLT-760; a global whatever the crust
        // budget, so recording it changes nothing).
        for (name, x) in subduction::ARC_SPLIT_GLOBALS.iter().zip(pull.arc_split_km3) {
            *world.globals.entry((*name).to_owned()).or_insert(0.0) += x;
        }
        lap(&mut clock, "fronts");
        cut_trace::mark(st, &cache.geo, "fronts");
        // Subduction erosion (PLT-441): its globals only while the rule is on, so a run without
        // it keeps its record.
        if let Some(forearcs) = &pull.forearcs {
            let e = subduction::erode_forearcs(st, &env, forearcs);
            book(st, gap_budget::Part::Forearc);
            for (name, x) in [
                (subduction::FOREARC_CONTINENTAL_GLOBAL, e.continental_km3),
                (subduction::FOREARC_OTHER_GLOBAL, e.other_km3),
                (subduction::FOREARC_SHORT_GLOBAL, e.short_km3),
                (
                    subduction::FOREARC_FRONT_CONTINENTAL_GLOBAL,
                    e.front_continental_km_myr,
                ),
                (
                    subduction::FOREARC_FRONT_OCEANIC_GLOBAL,
                    e.front_oceanic_km_myr,
                ),
            ] {
                *world.globals.entry(name.to_owned()).or_insert(0.0) += x;
            }
            // The trench's retreat (PLT-523): its globals only while it is on.
            if env.k.erosion_retreat {
                for (name, x) in [
                    (
                        subduction::RETREAT_CONTINENTAL_GLOBAL,
                        e.retreat_continental_km2,
                    ),
                    (subduction::RETREAT_OTHER_GLOBAL, e.retreat_other_km2),
                ] {
                    *world.globals.entry(name.to_owned()).or_insert(0.0) += x;
                }
            }
            lap(&mut clock, "forearcs");
            cut_trace::mark(st, &cache.geo, "forearcs");
        }
        // The boundary mix (measurement only) on the same samples as the fronts, before the
        // collapse below turns any crust over.
        // At a step the crust budget maps, each sample's class too (the maps' front flags).
        let snapshot_due = step >= self.total
            || step.is_multiple_of((GAP_SNAPSHOT_MYR / env.dt_myr).round().max(1.0) as u64);
        let classes: Option<Vec<std::sync::atomic::AtomicU8>> =
            (snapshot_due && cb.borrow().as_ref().is_some_and(|b| b.maps_on())).then(|| {
                (0..st.samples.len())
                    .map(|_| std::sync::atomic::AtomicU8::new(0))
                    .collect()
            });
        // The front trace reads the classes at every step (measurement only).
        let traced_classes: Option<Vec<std::sync::atomic::AtomicU8>> =
            (classes.is_none() && traced_fronts.is_some()).then(|| {
                (0..st.samples.len())
                    .map(|_| std::sync::atomic::AtomicU8::new(0))
                    .collect()
            });
        let mix = dynamics::boundary_mix(
            st,
            &cache.geo,
            &pos,
            &tree,
            &env,
            classes.as_deref().or(traced_classes.as_deref()),
        );
        if let (Some(f), Some(fb)) = (ft.borrow_mut().as_mut(), traced_fronts.as_deref()) {
            f.dip = front_trace::dip_rows(
                st,
                &cache.geo,
                &pos,
                &env,
                fb,
                classes.as_deref().or(traced_classes.as_deref()),
            );
        }
        book(st, gap_budget::Part::Other);
        lap(&mut clock, "boundary_mix");
        // Crust beyond the thickest column goes down; thickened crust that nothing raises any
        // more spreads and gives its area back.
        let mut fell = collapse::Collapse::default();
        let foundered = collapse::founder(st, &cache.geo, &env, &mut fell, sea);
        book(st, gap_budget::Part::Founder);
        *world
            .globals
            .entry(collapse::FOUNDERED_GLOBAL.to_owned())
            .or_insert(0.0) += foundered / 1000.0;
        let quiet = collapse::step(st, &cache.geo, &env, sea);
        book(st, gap_budget::Part::Collapse);
        fell.sources += quiet.sources;
        fell.moved_km2_m += quiet.moved_km2_m;
        fell.converted += quiet.converted;
        *world
            .globals
            .entry(collapse::MOVED_GLOBAL.to_owned())
            .or_insert(0.0) += fell.moved_km2_m / 1000.0;
        *world
            .globals
            .entry(collapse::GAINED_GLOBAL.to_owned())
            .or_insert(0.0) += f64::from(fell.converted) * env.norm.sample_area_km2();
        *world
            .globals
            .entry(collapse::HELD_GLOBAL.to_owned())
            .or_insert(0.0) += quiet.held_km2_m / 1000.0;
        *world
            .globals
            .entry(collapse::UNPLACED_GLOBAL.to_owned())
            .or_insert(0.0) += quiet.unplaced_km2_m / 1000.0;
        lap(&mut clock, "collapse");
        cut_trace::mark(st, &cache.geo, "collapse");
        // Thickened crust spreads through its lower crust (`flow.rs`).
        let flowed = flow::step(st, &cache.geo, &env);
        book(st, gap_budget::Part::Flow);
        *world
            .globals
            .entry(flow::MOVED_GLOBAL.to_owned())
            .or_insert(0.0) += flowed.moved_km2_m / 1000.0;
        if flowed.substeps > 0 {
            for (key, v) in [
                (flow::STEPS_GLOBAL, 1.0),
                (flow::SUBSTEPS_GLOBAL, f64::from(flowed.substeps)),
                (flow::SHORT_GLOBAL, (env.dt_myr - flowed.time_myr).max(0.0)),
                (flow::EDGE_GLOBAL, f64::from(flowed.edge)),
            ] {
                *world.globals.entry(key.to_owned()).or_insert(0.0) += v;
            }
        }
        lap(&mut clock, "flow");
        cut_trace::mark(st, &cache.geo, "flow");
        // Rifted margins taper from continent to ocean over Earth's width (`transition.rs`).
        let taper = transition::step(st, &cache.geo, &pos, &env);
        book(st, gap_budget::Part::Transition);
        lap(&mut clock, "transition");
        *world
            .globals
            .entry(transition::MOVED_GLOBAL.to_owned())
            .or_insert(0.0) += taper.moved_km2_m / 1000.0;
        *world
            .globals
            .entry(transition::MARGINS_LEFT_GLOBAL.to_owned())
            .or_insert(0.0) += taper.margins_left_km2_m / 1000.0;
        // The visible continental crust's volume (measurement: the crust budget, km³).
        let volume_m: f64 = st
            .samples
            .iter()
            .filter(|s| s.is_continental() && !s.is_hidden())
            .map(|s| f64::from(s.thickness_m))
            .sum();
        world.globals.insert(
            subduction::CONTINENTAL_VOLUME_GLOBAL.to_owned(),
            volume_m * env.norm.sample_area_km2() / 1000.0,
        );
        // The force balance (`forces.rs`): the rotation each plate's motion relaxes towards. The
        // land's column energy is taken from the step's sea (ledger: FRC-014).
        let balance = forces::step(
            st,
            &cache.geo,
            &pos,
            &env,
            forces::flow_axis(self.preset.seed),
            sea,
        );
        book(st, gap_budget::Part::Other);
        // Measurement only: the slab pull's fronts (ledger: PLT-554).
        for (name, x) in [
            forces::SLAB_FRONT_LATTICE_GLOBAL,
            forces::SLAB_FRONT_NORMAL_GLOBAL,
            forces::SLAB_FRONT_STRAIGHT_GLOBAL,
        ]
        .into_iter()
        .zip(balance.slab_front_km)
        {
            *world.globals.entry(name.to_owned()).or_insert(0.0) += x * env.dt_myr;
        }
        if !balance.solved {
            *world
                .globals
                .entry(UNSOLVED_GLOBAL.to_owned())
                .or_insert(0.0) += 1.0;
        }
        // The live plates a solved balance left out (no visible sample: they keep their
        // rotation), for the dynamics record.
        let out_of_balance: Vec<u32> = if balance.solved {
            (0..st.plates.len())
                .filter(|&p| {
                    st.plates[p].alive && balance.omega.get(p).copied().flatten().is_none()
                })
                .map(|p| p as u32)
                .collect()
        } else {
            Vec::new()
        };
        lap(&mut clock, "forces");
        cut_trace::mark(st, &cache.geo, "forces");
        let mut causes: BTreeMap<u32, EventId> = BTreeMap::new();
        let mut sutured = false;
        // Plates whose motion a weld, a transfer or a healed rift set this step (the balance
        // takes over at the next step); a deferred collision sets none (`motion_set_by`).
        let mut merged: Vec<u32> = Vec::new();
        for f in &fired {
            let (under, over) = (st.contacts[f.contact].under, st.contacts[f.contact].over);
            let done = collision::collide(
                st,
                world,
                &cache.geo,
                &pos,
                &env,
                f.contact,
                &f.seeds,
                &f.front,
                f.stalled,
                balance.collision_n.get(&f.contact).copied(),
            )?;
            if let Some(ev) = done.event {
                causes.insert(under, ev);
            }
            if cut_trace::on() {
                let what = if done.sutured {
                    ["weld:under", "weld:over", "weld:other"]
                } else if done.deferred {
                    ["deferred:under", "deferred:over", "deferred:other"]
                } else {
                    ["transfer:under", "transfer:over", "transfer:other"]
                };
                cut_trace::mark_pair(st, &cache.geo, what, under, over);
            }
            sutured |= done.sutured;
            merged.extend(motion_set_by(&done, under, over).into_iter().flatten());
        }
        book(st, gap_budget::Part::Collision);
        cut_trace::mark(st, &cache.geo, "collision");
        // Rifts whose two sides never moved apart heal (design §3.4).
        let healed = rifting::heal(st, world, &cache.geo, &pos, &env)?;
        book(st, gap_budget::Part::Rifting);
        sutured |= !healed.is_empty();
        merged.extend(healed);
        lap(&mut clock, "collision");
        cut_trace::mark(st, &cache.geo, "heal");
        hotspots::step(st, world, &pos, &tree, &env, &mut cones)?;
        book(st, gap_budget::Part::Hotspots);
        lap(&mut clock, "hotspots");
        // The lithosphere's thermal state from the step's flags (`thermal.rs`; off by default).
        thermal::step(st, &env);
        // Continental columns settle towards the height their crust holds up (`settle.rs`).
        let settled = settle::step(st, &env, flexure.as_deref(), sea);
        book(st, gap_budget::Part::Settle);
        *world
            .globals
            .entry(settle::RISE_GLOBAL.to_owned())
            .or_insert(0.0) += settled.rise_km2_m / 1000.0;
        *world
            .globals
            .entry(settle::FALL_GLOBAL.to_owned())
            .or_insert(0.0) += settled.fall_km2_m / 1000.0;
        lap(&mut clock, "settle");
        // No column ends the step's processes thicker than its ceiling or higher than its top
        // (ledger: PLT-287, PLT-288).
        let limited = rate::limit(st, &cache.geo, &env);
        book(st, gap_budget::Part::Rate);
        add_limited(world, limited);
        lap(&mut clock, "rate");
        surface(st, &cache.geo, &env, &taper.floor_m, sea);
        book(st, gap_budget::Part::PlateSurface);
        finish_samples(st, &cache.geo, &z0, &env);
        settle::crust_uplift(st, &crust0, env.gravity, env.dt_yr, false);
        book(st, gap_budget::Part::Finish);
        let arcs = dynamics::arc_heights(st, sea);
        lap(&mut clock, "surface_finish");
        cut_trace::mark(st, &cache.geo, "surface_finish");

        // Motion for the next step: each plate's rotation relaxes towards the balance's
        // (`motion_relax_myr`), under the speed guard on the plate's own fastest point, which is
        // counted whenever it binds.
        let mut next: Vec<Option<DVec3>> = vec![None; st.plates.len()];
        for (p, w) in next.iter_mut().enumerate() {
            if !st.plates[p].alive || merged.contains(&(p as u32)) {
                continue;
            }
            if let Some(target) = balance.omega.get(p).copied().flatten() {
                *w = Some(forces::relax(
                    DVec3::from_array(st.plates[p].omega),
                    target,
                    env.dt_myr,
                    k.motion_relax_myr,
                ));
            }
        }
        let reach = kinematics::reach(st, &pos, &next);
        let mut guarded: Vec<u32> = Vec::new();
        for (p, w) in next.into_iter().enumerate() {
            let Some(w) = w else { continue };
            let (w, bound) =
                kinematics::guard_speed(w, reach[p], &env.norm, k.max_plate_speed_mm_yr);
            if bound {
                guarded.push(p as u32);
            }
            st.plates[p].omega = w.to_array();
        }
        if !guarded.is_empty() {
            *world
                .globals
                .entry(GUARD_BOUND_GLOBAL.to_owned())
                .or_insert(0.0) += guarded.len() as f64;
        }
        book(st, gap_budget::Part::Other);
        rifting::maybe_rift(st, world, &cache.geo, &pos, &env, &balance.stress_mpa)?;
        cut_trace::mark(st, &cache.geo, "rift");
        rifting::maybe_initiate_subduction(
            st,
            world,
            &cache.geo,
            &pos,
            &env,
            &balance.stress_mpa,
            &pull.subducting_fronts,
        )?;
        cut_trace::mark(st, &cache.geo, "initiate");
        // Back-arc rifting, the moving trench (ledger: PLT-710).
        rifting::maybe_rift_backarc(
            st,
            world,
            &cache.geo,
            &pos,
            &env,
            &pull.subducting_fronts,
            &pull.subducting_detail,
        )?;
        cut_trace::mark(st, &cache.geo, "backarc");
        // Along-strike propagation to a passive margin (law 2; ledger: PLT-656).
        propagation::step(st, world, &cache.geo, &pos, &env)?;
        cut_trace::mark(st, &cache.geo, "propagate");
        book(st, gap_budget::Part::Rifting);
        lap(&mut clock, "motion_rifting");

        st.t_myr = env.t_myr;
        st.step = step;
        world.t_years = env.t_myr * 1.0e6;
        for (under, ev) in subduction::close_contacts(st, world, &env, false)? {
            causes.entry(under).or_insert(ev);
        }
        cut_trace::mark(st, &cache.geo, "close_contacts");
        ridges::bury(st, world, None, &pos, &env, &causes)?;
        book(st, gap_budget::Part::Bury);
        lap(&mut clock, "close_bury");
        cut_trace::mark(st, &cache.geo, "bury");

        // Resample when due, and once at the end so the final crust has no gaps or hidden slabs.
        // A suture resamples at once, so the welded plate's subducted slab is consumed (hidden
        // crust of one plate under the same plate would surface).
        let final_step = step >= self.total;
        let resampling =
            ridges::due(st, &env) || ((final_step || sutured) && st.resample_step != st.step);
        if resampling {
            shown::resampling(st, env.t_myr);
            // The subduction zones go on through the new lattice (ledger: PLT-550).
            subduction::carry(st, step, env.k.contact_carry);
            let cycle = step.saturating_sub(st.resample_step);
            front_trace::want_resample(ft.borrow().is_some());
            let geo = ridges::resample(st, world, &pos, &tree, &cache.geo, &env)?;
            front_trace::want_resample(false);
            shown::resampled(st);
            if let Some(f) = ft.borrow_mut().as_mut() {
                f.resampled(&st.samples, env.t_myr, cycle);
            }
            if let Some(b) = cb.borrow_mut().as_mut() {
                b.book_lattice(&st.samples, gap_budget::Part::ResampleBlend);
            }
            // The accretion is thickening too (ledger: PLT-287); the surface's change counts as
            // the step's uplift, as the accretion's own rise does.
            let z: Vec<f32> = st.samples.iter().map(|s| s.elevation_m).collect();
            let crust = settle::crust_heights(st, env.gravity);
            let limited = rate::limit(st, &geo, &env);
            if limited.columns > 0 {
                for (s, &z) in st.samples.iter_mut().zip(&z) {
                    s.uplift_mm_yr +=
                        ((f64::from(s.elevation_m) - f64::from(z)) / env.dt_yr * 1.0e3) as f32;
                }
                settle::crust_uplift(st, &crust, env.gravity, env.dt_yr, true);
            }
            book(st, gap_budget::Part::Rate);
            add_limited(world, limited);
            pos = geo.points().to_vec();
            let doomed: BTreeMap<u32, EventId> = st
                .contacts
                .iter()
                .filter_map(|c| c.onset_event.map(|e| (c.under, e)))
                .collect();
            ridges::bury(st, world, Some(&geo), &pos, &env, &doomed)?;
            // Every sample's distance from its plate's outline as the resampling and the burial
            // left it (ledger: PLT-536).
            if env.k.level_set_membership {
                level_set::reinit(st, &geo, env.norm.spacing_rad());
            }
            book(st, gap_budget::Part::Bury);
            *cache = Cache {
                geo,
                coupling: None,
            };
            lap(&mut clock, "resample");
            cut_trace::mark(st, &cache.geo, "resample");
        }
        rate::end(st);
        // Crust made continental after `finish_samples` set the heat flow (ledger: PLT-469), and
        // how often the columns' ceiling binds (measurement, PLT-824).
        let clamped = thermal::refresh_heat_flow(st, env.k, env.t_myr) as f64;
        *world
            .globals
            .entry(thermal::CLAMPED_GLOBAL.to_owned())
            .or_insert(0.0) += clamped;
        world
            .globals
            .insert(thermal::CLAMPED_LAST_GLOBAL.to_owned(), clamped);

        book(st, gap_budget::Part::Other);
        if tracing {
            peaks::print(st, &cache.geo, &env, sea, &traced, resampling);
        }
        if let Some(gb) = st.gap_budget.as_mut() {
            gb.end_step(&st.samples, env.t_myr, env.dt_myr);
            let every = (GAP_SNAPSHOT_MYR / env.dt_myr).round().max(1.0) as u64;
            if step >= self.total || step.is_multiple_of(every) {
                gb.snapshot(&st.samples, env.t_myr, env.dt_myr);
            }
        }
        let mut cb = cb.into_inner();
        if let Some(b) = cb.as_mut() {
            if snapshot_due {
                // The fronts were found on the lattice a resampling has since replaced: none.
                let fronts: Option<Vec<u8>> = classes
                    .filter(|_| !resampling)
                    .map(|c| c.into_iter().map(|x| x.into_inner()).collect());
                b.snapshot(
                    &st.samples,
                    &pos,
                    env.t_myr,
                    env.norm.sample_area_km2(),
                    sea_m(world),
                    &world.globals,
                    fronts.as_deref(),
                );
                b.write_plates(&st.samples, |i| st.plate_of(i), env.t_myr);
                b.write_ages(&st.samples, env.t_myr);
            }
            b.end_step(env.t_myr, env.norm.sample_area_km2(), &world.globals);
            if step >= self.total {
                world.notes.insert(CRUST_BUDGET_NOTE.to_owned(), b.lines());
            }
        }
        self.crust_budget = cb;
        let mut ft = ft.into_inner();
        if let Some(f) = ft.as_mut() {
            f.end_step(
                &st.samples,
                &pos,
                |i| st.plate_of(i),
                step,
                env.t_myr,
                resampling,
                snapshot_due,
            );
        }
        self.front_trace = ft;
        dynamics::record_step(
            st,
            world,
            &cache.geo,
            &pos,
            &env,
            final_step,
            dynamics::Observed {
                front_km: &pull.front_km,
                mix,
                arcs,
                forces: balance.recorded.as_ref(),
                guarded: &guarded,
                sea_m: sea,
                out_of_balance: &out_of_balance,
            },
        );
        lap(&mut clock, "record");
        self.done = step;
        // How often the elevation cap binds: visible samples standing at `max_elevation_m` after
        // the step, and visible continental samples, summed over the steps (2026-10-01: the cap is
        // meant to be a guard that almost never binds; `run`/`stage` print the share).
        let cap = env.k.max_elevation_m as f32 - 0.5;
        let (at_cap, continental) = st
            .samples
            .par_iter()
            .filter(|s| !s.is_hidden() && s.kind == state::CONTINENTAL)
            .fold(
                || (0u64, 0u64),
                |(a, n), s| (a + u64::from(s.elevation_m >= cap), n + 1),
            )
            .reduce(|| (0, 0), |x, y| (x.0 + y.0, x.1 + y.1));
        *world
            .globals
            .entry(CAP_BOUND_GLOBAL.to_owned())
            .or_insert(0.0) += at_cap as f64;
        *world
            .globals
            .entry(CONTINENTAL_GLOBAL.to_owned())
            .or_insert(0.0) += continental as f64;
        let last = self.done >= self.total;
        let margin_every = (MARGIN_EVERY_MYR / env.dt_myr).round().max(1.0) as u64;
        if last || step.is_multiple_of(margin_every) {
            record_margins(st, world, &pos, &env);
        }
        lap(&mut clock, "stats_margins");
        let kf_every = self.preset.keyframes.every_myr * 1.0e6;
        let kf_due = world.t_years >= self.next_output_years;
        if kf_due {
            let origin = self.preset.t_start_years();
            let n = ((world.t_years - origin) / kf_every).floor() + 1.0;
            self.next_output_years = origin + n * kf_every;
        }
        let every = u64::from(k.l1_every_steps);
        if last || kf_due || (every > 0 && self.done.is_multiple_of(every)) {
            self.write_l1(st, world, &pos, &env)?;
        }
        lap(&mut clock, "write_l1");
        if last {
            subduction::close_contacts(st, world, &env, true)?;
            for h in 0..st.hotspots.len() {
                if !st.hotspots[h].closed {
                    hotspots::close(st, world, &env, h, true)?;
                }
            }
            // Closing the contacts ended the collisions still under way: the note gets them.
            note_dynamics(st, world);
        }
        Ok(())
    }

    /// The surface stage's state since the last step (P1-04, `surface/mod.rs`): where its surface
    /// stands over this model's elevation (`surface/feedback_m`, the whole offset since
    /// 2026-10-01, so a sample's elevation is the surface's at its scale) and the material it moved
    /// (`surface/material_m`) per L1 cell are spread onto the samples with the last L1 sampling's
    /// barycentric weights (each sample takes the weighted mean over the cells that see it), added
    /// to their elevation and rock pools (erosion strips the column from the top, deposits join
    /// the sediment pool); what was applied is sampled back onto L1
    /// (`surface/feedback_applied_m`, a diagnostic). Runs before the step's motion, on the
    /// positions of the last sampling.
    ///
    /// The volcano stage's cones (`surface/volcanic_felsic_m`, `surface/volcanic_mafic_m`) join
    /// the pools as felsic and mafic volcanic rock the same way, after the material (plate task
    /// 8: before, a cone was elevation only and its erosion stripped the crust under it).
    /// The hotspots' cones under the cone budget (`surface/volcanic_hotspot_felsic_m`,
    /// `surface/volcanic_hotspot_mafic_m`, ledger: PLT-803) join the pools the same way, after the
    /// rest, but are not returned: their tails net them by hotspot, so no arc nor kernel counts
    /// them again by place.
    /// Returns, per sample, the other cones' rock it received (felsic, mafic, m; empty without
    /// feedback), which the arcs and hotspots count inside their own rates, and, when `flexure`
    /// asks, the surface's flexure per sample ([`Self::sample_flexure`]), sampled beside the sums.
    fn apply_surface_feedback(
        &mut self,
        st: &mut PlateState,
        world: &mut WorldState,
        env: &Env<'_>,
        flexure: bool,
    ) -> Result<Feedback, CoreError> {
        let cone_fields = [
            world.fields.remove(crate::surface::field::VOLCANIC_FELSIC),
            world.fields.remove(crate::surface::field::VOLCANIC_MAFIC),
        ];
        let hot_fields = [
            world
                .fields
                .remove(crate::surface::field::VOLCANIC_HOTSPOT_FELSIC),
            world
                .fields
                .remove(crate::surface::field::VOLCANIC_HOTSPOT_MAFIC),
        ];
        let Some(fb) = world.fields.remove(crate::surface::field::FEEDBACK) else {
            return Ok((Vec::new(), None, None));
        };
        let material = world.fields.remove(crate::surface::field::MATERIAL);
        // The surface's parts of the feedback, for the gap budget (`gap_budget.rs`).
        const SP: usize = gap_budget::SURFACE_PARTS.len();
        let parts: Vec<Option<Field>> = gap_budget::SURFACE_PARTS
            .map(|(_, name)| world.fields.remove(name))
            .into_iter()
            .collect();
        let grid = self.l1_grid()?;
        if fb.data.len() != grid.cells() {
            return Ok((Vec::new(), None, None));
        }
        // The cells whose offset the surface holds (`surface/offset_held`, ISO-125), for the peak
        // trace only; the surface's own field, which its next step's carry reads (not taken).
        let held_field: Option<&[f32]> = peaks::threshold()
            .and(world.fields.get(crate::surface::field::OFFSET_HELD))
            .filter(|f| f.data.len() == grid.cells())
            .map(|f| f.data.as_slice());
        let cone_fields: [Option<Field>; 2] =
            cone_fields.map(|f| f.filter(|f| f.data.len() == grid.cells()));
        let hot_fields: [Option<Field>; 2] =
            hot_fields.map(|f| f.filter(|f| f.data.len() == grid.cells()));
        let hot_any = hot_fields.iter().any(Option::is_some);
        let parts: Option<Vec<Field>> = if st.gap_budget.is_some() {
            parts
                .into_iter()
                .map(|f| f.filter(|f| f.data.len() == grid.cells()))
                .collect()
        } else {
            None
        };
        let cache = self.cache.as_mut().expect("built before the step");
        if cache.coupling.is_none() {
            // After a resume: link the cells exactly as the last sampling did (history-free).
            let pos = geometry::positions(st, &cache.geo);
            let tree = SampleTree::new(cache.geo.points());
            let mut coupling = Coupling::build(&grid, &cache.geo.mesh, &tree);
            let _ = l1::sample(st, &cache.geo, &mut coupling, &pos, env, &self.preset.scale);
            cache.coupling = Some(coupling);
        }
        let coupling = cache.coupling.as_ref().expect("built above");
        let mesh = &cache.geo.mesh;
        let n = st.samples.len();
        let tris = mesh.triangles();
        let links = coupling.links();
        // Over every cell, by its weight: the offset `z`, the material and the parts the gap
        // budget books against it, the rock, the cones. A cell whose offset the surface holds
        // (NaN, `surface.held_offsets`, ISO-125) gives none and keeps its weight: a sample half on
        // held cells takes half the offset of the rest (renormalised over the rest, a sample 92 %
        // on held cells took 13 times its few other cells' offset and rose 7.6 km in a step). `wh`,
        // `zh`: the held cells' share and what they handed over (the peak trace).
        #[derive(Clone, Copy, Default)]
        struct Sums {
            w: f64,
            z: f64,
            m: f64,
            mz: f64,
            c: [f64; 2],
            h: [f64; 2],
            wh: f64,
            zh: f64,
        }
        // The flexure's sample (`sample_flexure`) beside the feedback's sums, on another thread:
        // both read the last sampling's links and neither writes what the other reads (P1-19:
        // one after the other before). Each stays one pass over the cells in order.
        let flex_field = world
            .fields
            .get(crate::surface::flexure::field::FLEXURE)
            .filter(|_| flexure);
        let points = cache.geo.points().len();
        let ((sums, psum), flexure) = rayon::join(
            || {
                let mut sums = vec![Sums::default(); n];
                let mut psum = vec![[0.0f64; SP]; if parts.is_some() { n } else { 0 }];
                for (c, l) in links.iter().enumerate() {
                    let t = tris[l.tri as usize];
                    let dz = f64::from(fb.data[c]);
                    // Without the material field the feedback is the rock (a held cell: none).
                    let dm = material
                        .as_ref()
                        .map_or(if dz.is_nan() { 0.0 } else { dz }, |m| f64::from(m.data[c]));
                    let cone = [0, 1].map(|k| {
                        cone_fields[k]
                            .as_ref()
                            .map_or(0.0, |f| f64::from(f.data[c]))
                    });
                    let hot = [0, 1]
                        .map(|k| hot_fields[k].as_ref().map_or(0.0, |f| f64::from(f.data[c])));
                    let pc = parts
                        .as_ref()
                        .map(|p| std::array::from_fn::<f64, SP, _>(|k| f64::from(p[k].data[c])));
                    let taken = !dz.is_nan();
                    let held = held_field.is_some_and(|h| h[c] > 0.5);
                    for (&v, &w) in t.iter().zip(l.w.iter()) {
                        let w = f64::from(w);
                        if w > 0.0 {
                            let a = &mut sums[v as usize];
                            a.w += w;
                            if taken {
                                a.z += w * dz;
                                a.mz += w * dm;
                            }
                            a.m += w * dm;
                            a.c[0] += w * cone[0];
                            a.c[1] += w * cone[1];
                            if hot_any {
                                a.h[0] += w * hot[0];
                                a.h[1] += w * hot[1];
                            }
                            if held {
                                a.wh += w;
                                if taken {
                                    a.zh += w * dz;
                                }
                            }
                            if let Some(pc) = pc.filter(|_| taken) {
                                for (a, x) in psum[v as usize].iter_mut().zip(pc) {
                                    *a += w * x;
                                }
                            }
                        }
                    }
                }
                (sums, psum)
            },
            || flex_field.and_then(|f| flexure_of(links, tris, points, f)),
        );
        let wsum = |v: usize| sums[v].w;
        let held = held_field.map(|_| {
            (0..n)
                .map(|v| {
                    if wsum(v) > 0.0 {
                        [(sums[v].wh / wsum(v)) as f32, (sums[v].zh / wsum(v)) as f32]
                    } else {
                        [f32::NAN; 2]
                    }
                })
                .collect()
        });
        let k = env.k;
        let dz: Vec<f64> = st
            .samples
            .par_iter_mut()
            .enumerate()
            .map(|(v, s)| {
                if sums[v].w <= 0.0 {
                    return 0.0;
                }
                let z0 = s.elevation_m;
                s.elevation_m = (f64::from(z0) + sums[v].z / sums[v].w)
                    .clamp(k.trench_elevation_m, k.max_elevation_m)
                    as f32;
                f64::from(s.elevation_m) - f64::from(z0)
            })
            .collect();
        let gravity = env.gravity;
        let applied: Vec<([f64; 2], [f64; 2], f64)> = st
            .samples
            .par_iter_mut()
            .enumerate()
            .map(|(v, s)| {
                if wsum(v) <= 0.0 {
                    return ([0.0; 2], [0.0; 2], 0.0);
                }
                let dm = sums[v].m / wsum(v);
                if dm < 0.0 {
                    pools::strip(&mut s.pools, -dm);
                } else {
                    s.pools[pool::SEDIMENT] += dm as f32;
                }
                // The cones' rock, on top (a caldera's collapse takes back what its summit had).
                // ledger: PLT-286
                let before = s.pools;
                let mut got = [0.0f64; 2];
                for (k, p) in [pool::FELSIC_VOLCANIC, pool::MAFIC_VOLCANIC]
                    .into_iter()
                    .enumerate()
                {
                    let x = (sums[v].c[k] / wsum(v)).max(-f64::from(s.pools[p]));
                    s.pools[p] += x as f32;
                    got[k] = x;
                }
                // The budgeted hotspots' cones, apart (ledger: PLT-803).
                let mut hot = [0.0f64; 2];
                if hot_any {
                    for (k, p) in [pool::FELSIC_VOLCANIC, pool::MAFIC_VOLCANIC]
                        .into_iter()
                        .enumerate()
                    {
                        let x = (sums[v].h[k] / wsum(v)).max(-f64::from(s.pools[p]));
                        s.pools[p] += x as f32;
                        hot[k] = x;
                    }
                }
                let cone_iso = pools::isostatic_change_m(&before, &s.pools, gravity);
                pools::diagenesis(&mut s.pools, gravity);
                s.thickness_m = s.pools.iter().sum();
                (got, hot, cone_iso)
            })
            .collect();
        let area = env.norm.sample_area_km2();
        for (k, name) in [CONE_FELSIC_GLOBAL, CONE_MAFIC_GLOBAL]
            .into_iter()
            .enumerate()
        {
            let km3: f64 = applied.iter().map(|(c, h, _)| c[k] + h[k]).sum::<f64>() * area / 1000.0;
            *world.globals.entry(name.to_owned()).or_insert(0.0) += km3;
        }
        // Measurement (ledger: PLT-804): of the cones' rock, what continental crust took, so the
        // crust budget's surface part (booked with erosion) can be read with the cones out.
        let continental: f64 = applied
            .iter()
            .zip(&st.samples)
            .filter(|(_, s)| s.kind == CONTINENTAL)
            .map(|((c, h, _), _)| c[0] + c[1] + h[0] + h[1])
            .sum();
        *world
            .globals
            .entry(CONE_CONTINENTAL_GLOBAL.to_owned())
            .or_insert(0.0) += continental * area / 1000.0;
        if let Some(gb) = st.gap_budget.as_mut() {
            // By the weight of every cell, as its elevation (ISO-125).
            let mean = |sum: &dyn Fn(usize) -> f64| -> Vec<f64> {
                (0..n)
                    .map(|v| {
                        if sums[v].w > 0.0 {
                            sum(v) / sums[v].w
                        } else {
                            0.0
                        }
                    })
                    .collect()
            };
            let m = mean(&|v| sums[v].mz);
            let part = |k: usize| {
                if psum.is_empty() {
                    vec![0.0; n]
                } else {
                    mean(&|v| psum[v][k])
                }
            };
            let surface: Vec<Vec<f64>> = (0..SP).map(part).collect();
            let cone_iso: Vec<f64> = applied.iter().map(|&(_, _, x)| x).collect();
            gb.book_surface(&st.samples, &m, &surface, &cone_iso);
        }
        let applied_dz = coupling.interpolate(mesh, |v| dz[v as usize]);
        world.fields.insert(
            crate::surface::field::FEEDBACK_APPLIED.to_owned(),
            Field {
                grid,
                data: applied_dz.into_iter().map(|x| x as f32).collect(),
            },
        );
        Ok((
            applied
                .into_iter()
                .map(|(c, _, _)| c.map(|x| x.max(0.0)))
                .collect(),
            flexure,
            held,
        ))
    }

    /// The surface's flexure (`surface/flexure_m`) per sample, through the last L1 sampling's
    /// links as [`Self::apply_surface_feedback`] spreads the feedback (NaN for a sample no cell
    /// sees); `None` without the field or the links.
    fn sample_flexure(&self, world: &WorldState) -> Option<Vec<f64>> {
        let field = world.fields.get(crate::surface::flexure::field::FLEXURE)?;
        let cache = self.cache.as_ref()?;
        let coupling = cache.coupling.as_ref()?;
        flexure_of(
            coupling.links(),
            cache.geo.mesh.triangles(),
            cache.geo.points().len(),
            field,
        )
    }

    fn write_l1(
        &mut self,
        st: &PlateState,
        world: &mut WorldState,
        pos: &[DVec3],
        env: &Env<'_>,
    ) -> Result<(), CoreError> {
        let grid = self.l1_grid()?;
        let cache = self.cache.as_mut().expect("built before sampling");
        if cache.coupling.is_none() {
            let tree = SampleTree::new(cache.geo.points());
            cache.coupling = Some(Coupling::build(&grid, &cache.geo.mesh, &tree));
        }
        let coupling = cache.coupling.as_mut().expect("built above");
        for (name, data) in l1::sample(st, &cache.geo, coupling, pos, env, &self.preset.scale) {
            world.fields.insert(
                name.to_owned(),
                Field {
                    grid: grid.clone(),
                    data,
                },
            );
        }
        Ok(())
    }
}

/// [`PlatesStage::apply_surface_feedback`]'s cone rock per sample, the flexure per sample, and
/// (for the peak trace) the share of each sample's cells whose offset the surface holds and what
/// they handed it, m.
type Feedback = (Vec<[f64; 2]>, Option<Vec<f64>>, Option<Vec<[f32; 2]>>);

/// The surface's flexure per sample through the links (`PlatesStage::sample_flexure`), for the
/// `n` samples of `tris`; `None` where the field is not the links' grid.
fn flexure_of(links: &[CellLink], tris: &[[u32; 3]], n: usize, field: &Field) -> Option<Vec<f64>> {
    if links.len() != field.data.len() {
        return None;
    }
    let mut wsum = vec![0.0f64; n];
    let mut sum = vec![0.0f64; n];
    for (c, l) in links.iter().enumerate() {
        let x = f64::from(field.data[c]);
        if !x.is_finite() {
            continue;
        }
        for (&v, &w) in tris[l.tri as usize].iter().zip(l.w.iter()) {
            let w = f64::from(w);
            if w > 0.0 && (v as usize) < n {
                wsum[v as usize] += w;
                sum[v as usize] += w * x;
            }
        }
    }
    Some(
        (0..n)
            .into_par_iter()
            .map(|v| {
                if wsum[v] > 0.0 {
                    sum[v] / wsum[v]
                } else {
                    f64::NAN
                }
            })
            .collect(),
    )
}

/// `WorldState::globals`: the volcano stage's cones handed to the pools as felsic volcanic rock
/// over the run, km³ (Earth-normalized; net of calderas' collapse).
pub const CONE_FELSIC_GLOBAL: &str = "plates/cone_felsic_km3";
/// … and as mafic volcanic rock (shields, the flood basalts' plateaus), km³.
pub const CONE_MAFIC_GLOBAL: &str = "plates/cone_mafic_km3";
/// … and of both, what continental crust took, km³ (measurement; ledger: PLT-804).
pub const CONE_CONTINENTAL_GLOBAL: &str = "plates/cone_continental_km3";

/// `WorldState::globals`: visible continental sample-steps standing at the elevation cap, and all
/// visible continental sample-steps.
pub const CAP_BOUND_GLOBAL: &str = "plates/cap_bound_sample_steps";
pub const CONTINENTAL_GLOBAL: &str = "plates/continental_sample_steps";
/// `WorldState::globals`: plate-steps on which the speed guard (`max_plate_speed_mm_yr`) bound,
/// and steps on which the force balance could not be solved (the plates kept their motion).
/// Both should stay 0 (design `docs/design/plate-force-balance.md` §7).
pub const GUARD_BOUND_GLOBAL: &str = "plates/guard_bound_plate_steps";
pub const UNSOLVED_GLOBAL: &str = "plates/balance_unsolved_steps";

/// The continental margins are measured this often, Myr (P1-27; `notes.plates.margins`).
pub const MARGIN_EVERY_MYR: f64 = 10.0;

/// The land's gap budget (`gap_budget.rs`, when on) is recorded this often, Myr (measurement
/// only).
// ledger: PLT-283 (game-choice: a reporting interval, read by no rule; agent, 2026-10-17).
pub const GAP_SNAPSHOT_MYR: f64 = 50.0;

/// Appends the continental margins of now ([`stats::margins`]) to the run's note
/// (`notes.plates.margins`, a series the CLI and the manifest report).
fn record_margins(st: &PlateState, world: &mut WorldState, pos: &[DVec3], env: &Env<'_>) {
    let initiated: std::collections::BTreeSet<u32> = world
        .events
        .iter()
        .filter(|e| {
            e.kind == events::PLATE_BIRTH
                && matches!(e.params.get("origin"),
                    Some(deeptime_format::ParamValue::Str(o)) if o == "subduction_initiation")
        })
        .filter_map(|e| match e.params.get("plate") {
            Some(deeptime_format::ParamValue::Int(p)) => u32::try_from(*p).ok(),
            _ => None,
        })
        .collect();
    let m = stats::margins(
        st,
        pos,
        &env.norm,
        &|p| initiated.contains(&p),
        forces::incipient_km(env.k),
        forces::slab_floor_mm_yr(env.k),
    );
    let mut note: serde_json::Value = world
        .notes
        .get(STAGE_NAME)
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    if !note["margins"].is_array() {
        note["margins"] = serde_json::json!([]);
    }
    if let (Some(list), Ok(v)) = (note["margins"].as_array_mut(), serde_json::to_value(&m)) {
        list.push(v);
    }
    world.notes.insert(STAGE_NAME.to_owned(), note.to_string());
    // The dynamics record goes into the same note (`dynamics.rs`), as of now.
    note_dynamics(st, world);
}

/// Writes the dynamics record (`dynamics.rs`), as of now, into the run's note
/// (`notes.plates.dynamics`). Nothing when recording is off.
fn note_dynamics(st: &PlateState, world: &mut WorldState) {
    let Some(d) = &st.dynamics else {
        return;
    };
    let mut note: serde_json::Value = world
        .notes
        .get(STAGE_NAME)
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    if let Ok(v) = serde_json::to_value(d) {
        note["dynamics"] = v;
    }
    world.notes.insert(STAGE_NAME.to_owned(), note.to_string());
}

/// Continental erosion, the ocean floor's subsidence (Cortial's dampening or GDH1's age law),
/// trench accretion and the rock pools (Cortial §4.5, `pools.rs`), then eroded rock moves one hop
/// downhill as sediment (to the lowest lattice neighbour of the same rigid group). `sea` is the
/// sea level the step starts under.
fn surface(st: &mut PlateState, geo: &Geo, env: &Env<'_>, held_floor: &[f32], sea: f64) {
    let k = env.k;
    let per_m = transition::crust_per_metre(env);
    let held_cap = 0.5 * (transition::continental_m() + transition::oceanic_m());
    let rates = SurfaceRates {
        erosion_m: k.continental_erosion_mm_yr * 1.0e-3 * env.dt_yr,
        damping_m: k.oceanic_damping_mm_yr * 1.0e-3 * env.dt_yr,
        accretion_m: k.sediment_accretion_mm_yr * 1.0e-3 * env.dt_yr,
        z_c: k.max_elevation_m,
        z_t: k.trench_elevation_m,
        z_floor: k.oceanic_floor_elevation_m,
        z_ridge: k.ridge_elevation_m,
        trench_relax: pools::trench_share(env.dt_myr, k.trench_half_life_myr),
        trench_release: if k.trench_release_half_life_myr > 0.0 {
            pools::trench_share(env.dt_myr, k.trench_release_half_life_myr) // ledger: PLT-443
        } else {
            0.0
        },
        gravity: env.gravity,
        law: k.ocean_depth,
        t_myr: env.t_myr,
        dt_myr: env.dt_myr,
        sea_m: sea,
    };
    let dampening = k.ocean_depth == crate::params::OceanDepth::Dampening;
    let eroded: Vec<f64> = st
        .samples
        .par_iter_mut()
        .enumerate()
        .map(|(i, s)| {
            // Thicker ocean crust than the ocean's own (an island arc, a plateau, stacked crust)
            // cools towards the floor its crust holds up, on the transition's height line,
            // never above the half-way column's (plate task 8; the gap budget's recommendation
            // 3: with arcs raised by their thickness, the bare floor would sink them).
            // ledger: PLT-264
            let mut held = held_floor.get(i).copied().unwrap_or(f32::NAN);
            if dampening && !held.is_finite() && s.kind == OCEANIC {
                held = transition::held_floor_m(
                    transition::crystalline_m(&s.pools),
                    held_cap,
                    k.oceanic_floor_elevation_m,
                    per_m,
                ) as f32;
            }
            surface_step(s, &rates, held)
        })
        .collect();
    let samples = &st.samples;
    let targets: Vec<(u32, f32)> = (0..samples.len())
        .into_par_iter()
        .filter(|&i| eroded[i] > 0.0)
        .map(|i| {
            let s = &samples[i];
            let lowest = geo
                .edges(i as u32)
                .map(|(u, _)| u)
                .filter(|&u| {
                    let o = &samples[u as usize];
                    o.group == s.group && !o.is_hidden() && o.elevation_m < s.elevation_m
                })
                .min_by(|&a, &b| {
                    samples[a as usize]
                        .elevation_m
                        .total_cmp(&samples[b as usize].elevation_m)
                        .then(a.cmp(&b))
                });
            (lowest.unwrap_or(i as u32), eroded[i] as f32)
        })
        .collect();
    for (to, m) in targets {
        let s = &mut st.samples[to as usize];
        s.pools[pool::SEDIMENT] += m;
        s.thickness_m += m;
    }
}

/// The two plates whose motion a collision's end set this step (`collision::collide`), which
/// skip the step's relaxation towards the balance (PLT-191) so the balance takes over at the
/// next step: a weld's (one rotation now, the area-weighted mean) and a transfer's (the
/// receiving plate took the terrane's momentum; ledger PLT-080). A deferred collision (a
/// continent converging that has not stalled) set nothing: its two plates go on relaxing
/// towards the balance, whose collision resistance is what slows and stalls them (force-balance
/// design, task 7). Until 2026-10-25 they were skipped too, so every continental collision past
/// `collision_threshold_km` held both plates' rotations until it stalled (the M5 lane's fix).
/// Nor does a collision that could do nothing (`Collided::moved` false): one whose plate a weld
/// killed earlier in the step, or whose terrane is gone so that it ends emptied. Until
/// 2026-10-26 those held both plates' rotations for the step too.
fn motion_set_by(done: &collision::Collided, under: u32, over: u32) -> Option<[u32; 2]> {
    done.moved.then_some([under, over])
}

/// Adds one pass of the thickening limit (`rate.rs`) to the run's globals, km³.
fn add_limited(world: &mut WorldState, limited: rate::Limited) {
    for (name, x) in [
        (rate::CAPPED_GLOBAL, limited.capped_km2_m),
        (rate::LOST_GLOBAL, limited.lost_km2_m),
        (rate::HELD_GLOBAL, limited.held_km2_m),
    ] {
        *world.globals.entry(name.to_owned()).or_insert(0.0) += x / 1000.0;
    }
}

/// Mature island arcs become continental; uplift rate, heat flow and regime for this step.
fn finish_samples(st: &mut PlateState, geo: &Geo, z0: &[f32], env: &Env<'_>) {
    let t = env.t_myr;
    // A belt is in the collision regime while its stamps go on: stamped this step (age 0) or the
    // last (age dt); half a step more absorbs the f32 stamp's rounding. ledger: PLT-734
    let active_myr = 1.5 * env.dt_myr;
    let samples = &st.samples;
    let margin: Vec<bool> = (0..samples.len())
        .into_par_iter()
        .map(|i| {
            let s = &samples[i];
            s.kind == CONTINENTAL
                && geo.edges(i as u32).any(|(u, _)| {
                    let o = &samples[u as usize];
                    o.group == s.group && o.kind == OCEANIC
                })
        })
        .collect();
    let dt_yr = env.dt_yr;
    // Island-arc crust becomes continental crust once it is thick enough, whatever its height
    // (2026-10-02; by thickness since plate task 8). Continental crust is defined by its
    // thickness and buoyancy: arc crust thicker than about 15 km is too buoyant to subduct and
    // jams a trench like a continent, which an arc built on ocean crust reaches after more than
    // about 20 Myr of activity (Cloos 1993). The arc's crust (`arc_oceanic_m`, at the sourced
    // rates, `subduction.rs`) on top of the ocean's own 7 km counts (ledger: PLT-047), not the
    // column's thickness: the plate model keeps crust stacked at resamplings under the oceanic
    // label (`transition.rs`), and that is no arc. Until 2026-10-02 a mature arc also had to
    // stand above sea level that step; most intra-oceanic arcs are submarine (Stern 2010).
    let mature_m = subduction::mature_arc_m(env.k);
    let thermal_heat = env.k.thermal_state;
    st.samples.par_iter_mut().enumerate().for_each(|(i, s)| {
        if s.kind == OCEANIC
            && !s.is_hidden()
            && s.arc_oceanic_m > 0.0
            && s.arc_oceanic_m >= mature_m
        {
            // The ocean crust under the arc stays under it, as its basement (ledger: PLT-331), and
            // the crust is an island arc's (ledger: PLT-760).
            s.make_continental();
            s.set_island_arc(true);
            s.flags |= EMERGED;
            if s.orogeny != OROGENY_HIMALAYAN {
                s.orogeny = OROGENY_ANDEAN;
                s.orogeny_myr = t as f32;
            }
        }
        s.uplift_mm_yr = ((f64::from(s.elevation_m) - f64::from(z0[i])) / dt_yr * 1.0e3) as f32;
        let f = s.flags;
        s.heat_flow_mw_m2 = if s.kind == OCEANIC {
            ocean_heat_flow(s.age_myr(t)) as f32
        } else {
            // An arc matured this step carries its floor's lithosphere.
            continental_heat_flow(s, t, thermal_heat)
        };
        s.regime = if s.is_hidden() {
            regime::TRENCH
        } else if f & HOTSPOT != 0 {
            regime::HOTSPOT
        } else if s.orogeny == OROGENY_HIMALAYAN && t - f64::from(s.orogeny_myr) < active_myr {
            // The active orogen: stamped this step or the last (ledger: PLT-734).
            regime::COLLISION
        } else if f & ARC != 0 {
            regime::ARC
        } else if f & TRENCH != 0 {
            regime::TRENCH
        } else if f & DIVERGENT != 0 {
            if s.kind == CONTINENTAL {
                regime::RIFT
            } else {
                regime::RIDGE
            }
        } else if f & SUBDUCTION_ZONE != 0 {
            regime::BACK_ARC
        } else if s.kind == OCEANIC {
            if s.age_myr(t) < 10.0 {
                regime::RIDGE
            } else {
                regime::OCEAN_BASIN
            }
        } else if margin[i] {
            regime::PASSIVE_MARGIN
        } else {
            regime::CRATON
        };
        s.thickness_m = s.pools.iter().sum();
        // No more basement than the column's mafic rock, none on ocean floor (what eroded,
        // foundered or flowed took it). ledger: PLT-331
        s.basement_m = s.basement();
    });
    st.counters.emerged += st.samples.iter().filter(|s| s.flags & EMERGED != 0).count() as u32;
}

/// A continental sample's heat flow, mW/m², by its law at `t_myr`: with `thermal` (the
/// lithosphere's state) from its lithosphere's depth, never above its lid's floor under the Moho
/// (PLT-820), its floor's where it has none (PLT-821), and its arc and hotspot terms (ledger:
/// PLT-467, PLT-468); else the old proxy, which decays after the last orogeny and adds an arc's, a
/// hotspot's and a divergent margin's terms (ledger: PLT-137). `finish_samples` sets it on every
/// continental sample; the heat flow's final pass ([`thermal::refresh_heat_flow`], PLT-469) on
/// crust made continental after that.
pub(crate) fn continental_heat_flow(s: &state::Sample, t_myr: f64, thermal: bool) -> f32 {
    let f = s.flags;
    if thermal {
        return thermal::sample_heat_flow(thermal::sample_l_km(s, t_myr) as f32, f);
    }
    let since = (t_myr - f64::from(s.orogeny_myr)).max(0.0);
    let mut q = 55.0 + 35.0 * (-since / 60.0).exp(); // ledger: PLT-137
    if f & ARC != 0 {
        q += 40.0;
    }
    if f & HOTSPOT != 0 {
        q += 40.0;
    }
    if f & DIVERGENT != 0 {
        q += 30.0;
    }
    q as f32
}

/// Table 2 of Cortial et al. 2019 (C++, i7-6700K): seconds per regular step, per oceanic crust
/// generation and per rifting at 60k, 100k, 250k and 500k samples.
const CORTIAL_TABLE2: [(f64, f64, f64, f64); 4] = [
    (60_000.0, 0.19, 0.58, 0.23),
    (100_000.0, 0.28, 1.22, 0.21),
    (250_000.0, 1.24, 3.58, 1.50),
    (500_000.0, 1.90, 13.10, 7.70),
];

/// Log–log interpolation of Cortial's Table 2 at `n` samples: (regular, generation, rifting).
pub fn cortial_cost(n: f64) -> (f64, f64, f64) {
    let t = &CORTIAL_TABLE2;
    let n = n.clamp(t[0].0, t[3].0);
    let i = (0..3).find(|&i| n <= t[i + 1].0).unwrap_or(2);
    let (a, b) = (t[i], t[i + 1]);
    let x = (n.ln() - a.0.ln()) / (b.0.ln() - a.0.ln());
    let lerp = |p: f64, q: f64| (p.ln() + x * (q.ln() - p.ln())).exp();
    (lerp(a.1, b.1), lerp(a.2, b.2), lerp(a.3, b.3))
}

/// The plan's cost model (§8.1) for a preset: steps × Cortial's regular step plus one crust
/// generation and one rift every 20 steps (40 Myr); memory from the state, mesh and L1 fields.
pub fn estimate(p: &PlanetPreset) -> StageEstimate {
    let n = f64::from(p.resolution.l0_samples);
    let steps = p.plate_steps() as f64;
    let (reg, gen_s, rift) = cortial_cost(n);
    let seconds = steps * reg + (steps / 20.0).ceil() * (gen_s + rift);
    let c = u64::from(p.scale.circumference_blocks);
    let r = u64::from(p.resolution.l1_r);
    let cells = (c / r) * (c / r / 2);
    let [kw, kh] = p.keyframes.size;
    let sample = std::mem::size_of::<state::Sample>() as u64;
    let n = u64::from(p.resolution.l0_samples);
    let bytes = n * (3 * sample + 400)
        + cells * (20 + 4 * 11)
        // The frames' layers, and the geologic provinces and the satellite view the session
        // derives from them.
        + 16 * (KEYFRAME_LAYERS.len() as u64 + 2) * u64::from(kw) * u64::from(kh) * 4;
    StageEstimate {
        bytes,
        seconds,
        history_bytes: 0,
    }
}

impl Stage for PlatesStage {
    fn name(&self) -> &str {
        STAGE_NAME
    }

    fn estimate(&self) -> StageEstimate {
        estimate(&self.preset)
    }

    fn total_steps(&self) -> u64 {
        self.total
    }

    fn steps_done(&self) -> u64 {
        self.done
    }

    fn step(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        if self.done >= self.total {
            return Ok(());
        }
        ctx.pool.install(|| {
            if world.plates.is_none() {
                ctx.reserve(self.estimate().bytes)?;
                let tectonics = self.preset.tectonics.clone();
                let env = self.env(&tectonics, 0);
                let (mut state, geo) = init::initial_state(&self.preset, &env)?;
                if self.record_dynamics {
                    state.dynamics = Some(dynamics::Dynamics::start(&state, world, &env));
                }
                if self.record_gap_budget {
                    let k = env.k;
                    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
                    state.gap_budget = Some(gap_budget::GapBudget::start(
                        &state.samples,
                        z_0,
                        env.gravity,
                    ));
                }
                world.plates = Some(state);
                self.cache = Some(Cache {
                    geo,
                    coupling: None,
                });
            }
            let mut st = world.plates.take().expect("initialised above");
            // A run resumed from a checkpoint (`deeptime stage --in`) starts its gap budget at the
            // resume: the parts count from there.
            if self.record_gap_budget && st.gap_budget.is_none() {
                let k = self.preset.tectonics.clone();
                let env = self.env(&k, st.step);
                let z_0 = 0.5 * (env.k.continent_elevation_m[0] + env.k.continent_elevation_m[1]);
                st.gap_budget = Some(gap_budget::GapBudget::start(&st.samples, z_0, env.gravity));
            }
            let result = self.advance(&mut st, world);
            geometry::forget_trees();
            world.plates = Some(st);
            result
        })
    }

    fn keyframe_layers(&self) -> &[&'static str] {
        &KEYFRAME_LAYERS
    }

    fn save_state(&self) -> Result<Vec<u8>, CoreError> {
        to_state(&Saved {
            done: self.done,
            next_output_years: self.next_output_years,
        })
    }

    fn load_state(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        let s: Saved = from_state(bytes)?;
        self.done = s.done;
        self.next_output_years = s.next_output_years;
        self.cache = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests;

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
pub mod dynamics;
pub mod events;
pub mod forces;
pub mod geometry;
pub mod hotspots;
pub mod init;
pub mod kinematics;
pub mod l1;
pub mod noise;
pub mod normalise;
pub mod pools;
pub mod ridges;
pub mod rifting;
pub mod shortening;
pub mod state;
pub mod stats;
pub mod subduction;
pub mod transition;

use std::collections::BTreeMap;

use deeptime_format::{EventId, GridDesc};
use glam::DVec3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub use state::PlateState;

use crate::error::CoreError;
use crate::params::{PlanetPreset, TectonicParams};
use crate::sphere::{Coupling, SampleTree};
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
}

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
        }
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
        self.apply_surface_feedback(st, world, &env)?;
        lap(&mut clock, "feedback");
        // The surface's flexure under the samples, for the foreland basins of the collisions'
        // record (`belts.rs`); measurement only.
        let flexure = if st.contacts.iter().any(|c| c.collision.is_some()) {
            self.sample_flexure(world)
        } else {
            None
        };
        let z0: Vec<f32> = st.samples.iter().map(|s| s.elevation_m).collect();
        lap(&mut clock, "flexure_sample");

        // Motion, then interactions at the new positions.
        kinematics::advance(st, env.dt_yr);
        let cache = self.cache.as_mut().expect("built above");
        let mut pos = geometry::positions(st, &cache.geo);
        st.recount(&pos);
        let tree = Tree::over(&pos, |_| true);
        lap(&mut clock, "motion");
        let overlaps = subduction::detect(st, &cache.geo, &pos, &tree);
        lap(&mut clock, "detect");
        let resolved = subduction::resolve(st, world, &overlaps, &pos, &env)?;
        lap(&mut clock, "resolve");
        let (fronts, fired) =
            shortening::advance(st, &cache.geo, &pos, &tree, &env, resolved.seeds);
        lap(&mut clock, "shortening");
        let stacked = belts::stack(st, &cache.geo, &pos, &env, &fronts, flexure.as_deref());
        // The collisions' crust budget: what crossed their fronts and what the belts and wedges
        // kept of it, km³ (Earth-normalized).
        let (crossed, kept) = stacked.values().fold((0.0, 0.0), |a, s| {
            (
                a.0 + s.crossed_km2 * s.thickness_m / 1000.0,
                a.1 + (s.belt_km2_m + s.wedge_km2_m) / 1000.0,
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
        lap(&mut clock, "belts");
        let pull = subduction::fronts_and_uplift(st, &cache.geo, &pos, &env);
        *world
            .globals
            .entry(subduction::ARC_CONTINENTAL_GLOBAL.to_owned())
            .or_insert(0.0) += pull.arc_continental_km3;
        *world
            .globals
            .entry(subduction::ARC_OCEANIC_GLOBAL.to_owned())
            .or_insert(0.0) += pull.arc_oceanic_km3;
        lap(&mut clock, "fronts");
        // The boundary mix (measurement only) on the same samples as the fronts, before the
        // collapse below turns any crust over.
        let mix_km = dynamics::boundary_mix(st, &cache.geo, &pos, &env);
        lap(&mut clock, "boundary_mix");
        // Crust beyond the thickest column goes down; thickened crust that nothing raises any
        // more spreads and gives its area back.
        let mut fell = collapse::Collapse::default();
        let foundered = collapse::founder(st, &cache.geo, &env, &mut fell);
        *world
            .globals
            .entry(collapse::FOUNDERED_GLOBAL.to_owned())
            .or_insert(0.0) += foundered / 1000.0;
        let quiet = collapse::step(st, &cache.geo, &env);
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
        lap(&mut clock, "collapse");
        // Rifted margins taper from continent to ocean over Earth's width (`transition.rs`).
        let taper = transition::step(st, &cache.geo, &pos, &env);
        lap(&mut clock, "transition");
        *world
            .globals
            .entry(transition::MOVED_GLOBAL.to_owned())
            .or_insert(0.0) += taper.moved_km2_m / 1000.0;
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
        // The force balance (`forces.rs`): the rotation each plate's motion relaxes towards.
        let balance = forces::step(
            st,
            &cache.geo,
            &pos,
            &env,
            forces::flow_axis(self.preset.seed),
        );
        if !balance.solved {
            *world
                .globals
                .entry(UNSOLVED_GLOBAL.to_owned())
                .or_insert(0.0) += 1.0;
        }
        lap(&mut clock, "forces");
        let mut causes: BTreeMap<u32, EventId> = BTreeMap::new();
        let mut sutured = false;
        // Plates whose motion a weld or a healed rift set this step (the balance takes over at
        // the next step).
        let mut merged: Vec<u32> = Vec::new();
        for f in &fired {
            let (under, over) = (st.contacts[f.contact].under, st.contacts[f.contact].over);
            let done = collision::collide(
                st, world, &cache.geo, &pos, &env, f.contact, &f.seeds, f.stalled,
            )?;
            if let Some(ev) = done.event {
                causes.insert(under, ev);
            }
            sutured |= done.sutured;
            merged.extend([under, over]);
        }
        // Rifts whose two sides never moved apart heal (design §3.4).
        let healed = rifting::heal(st, world, &cache.geo, &pos, &env)?;
        sutured |= !healed.is_empty();
        merged.extend(healed);
        lap(&mut clock, "collision");
        hotspots::step(st, world, &pos, &tree, &env)?;
        lap(&mut clock, "hotspots");
        surface(st, &cache.geo, &env, &taper.floor_m);
        finish_samples(st, &cache.geo, &z0, &env);
        let arcs = dynamics::arc_heights(st);
        lap(&mut clock, "surface_finish");

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
        rifting::maybe_rift(st, world, &cache.geo, &pos, &env)?;
        rifting::maybe_initiate_subduction(st, world, &cache.geo, &pos, &env)?;
        lap(&mut clock, "motion_rifting");

        st.t_myr = env.t_myr;
        st.step = step;
        world.t_years = env.t_myr * 1.0e6;
        for (under, ev) in subduction::close_contacts(st, world, &env, false)? {
            causes.entry(under).or_insert(ev);
        }
        ridges::bury(st, world, None, &pos, &env, &causes)?;
        lap(&mut clock, "close_bury");

        // Resample when due, and once at the end so the final crust has no gaps or hidden slabs.
        // A suture resamples at once, so the welded plate's subducted slab is consumed (hidden
        // crust of one plate under the same plate would surface).
        let final_step = step >= self.total;
        if ridges::due(st, &env) || ((final_step || sutured) && st.resample_step != st.step) {
            let geo = ridges::resample(st, world, &pos, &cache.geo, &env)?;
            pos = geo.points().to_vec();
            let doomed: BTreeMap<u32, EventId> = st
                .contacts
                .iter()
                .filter_map(|c| c.onset_event.map(|e| (c.under, e)))
                .collect();
            ridges::bury(st, world, Some(&geo), &pos, &env, &doomed)?;
            *cache = Cache {
                geo,
                coupling: None,
            };
            lap(&mut clock, "resample");
        }

        dynamics::record_step(
            st,
            world,
            &cache.geo,
            &pos,
            &env,
            final_step,
            dynamics::Observed {
                front_km: &pull.front_km,
                mix_km,
                arcs,
                forces: balance.recorded.as_ref(),
                guarded: &guarded,
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
    fn apply_surface_feedback(
        &mut self,
        st: &mut PlateState,
        world: &mut WorldState,
        env: &Env<'_>,
    ) -> Result<(), CoreError> {
        let Some(fb) = world.fields.remove(crate::surface::field::FEEDBACK) else {
            return Ok(());
        };
        let material = world.fields.remove(crate::surface::field::MATERIAL);
        let grid = self.l1_grid()?;
        if fb.data.len() != grid.cells() {
            return Ok(());
        }
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
        let tris = mesh.triangles();
        let n = st.samples.len();
        let mut wsum = vec![0.0f64; n];
        let mut zsum = vec![0.0f64; n];
        let mut msum = vec![0.0f64; n];
        for (c, l) in coupling.links().iter().enumerate() {
            let t = tris[l.tri as usize];
            let dz = f64::from(fb.data[c]);
            let dm = material.as_ref().map_or(dz, |m| f64::from(m.data[c]));
            for (&v, &w) in t.iter().zip(l.w.iter()) {
                let w = f64::from(w);
                if w > 0.0 {
                    let v = v as usize;
                    wsum[v] += w;
                    zsum[v] += w * dz;
                    msum[v] += w * dm;
                }
            }
        }
        let k = env.k;
        let dz: Vec<f64> = st
            .samples
            .par_iter_mut()
            .enumerate()
            .map(|(v, s)| {
                if wsum[v] <= 0.0 {
                    return 0.0;
                }
                let z0 = s.elevation_m;
                s.elevation_m = (f64::from(z0) + zsum[v] / wsum[v])
                    .clamp(k.trench_elevation_m, k.max_elevation_m)
                    as f32;
                f64::from(s.elevation_m) - f64::from(z0)
            })
            .collect();
        st.samples.par_iter_mut().enumerate().for_each(|(v, s)| {
            if wsum[v] <= 0.0 {
                return;
            }
            let dm = msum[v] / wsum[v];
            if dm < 0.0 {
                pools::strip(&mut s.pools, -dm);
            } else {
                s.pools[pool::SEDIMENT] += dm as f32;
            }
            pools::diagenesis(&mut s.pools, env.gravity);
            s.thickness_m = s.pools.iter().sum();
        });
        let applied = coupling.interpolate(mesh, |v| dz[v as usize]);
        world.fields.insert(
            crate::surface::field::FEEDBACK_APPLIED.to_owned(),
            Field {
                grid,
                data: applied.into_iter().map(|x| x as f32).collect(),
            },
        );
        Ok(())
    }

    /// The surface's flexure (`surface/flexure_m`) per sample, through the last L1 sampling's
    /// links as [`Self::apply_surface_feedback`] spreads the feedback (NaN for a sample no cell
    /// sees); `None` without the field or the links.
    fn sample_flexure(&self, world: &WorldState) -> Option<Vec<f64>> {
        let field = world.fields.get(crate::surface::flexure::field::FLEXURE)?;
        let cache = self.cache.as_ref()?;
        let coupling = cache.coupling.as_ref()?;
        let tris = cache.geo.mesh.triangles();
        let n = cache.geo.points().len();
        if coupling.links().len() != field.data.len() {
            return None;
        }
        let mut wsum = vec![0.0f64; n];
        let mut sum = vec![0.0f64; n];
        for (c, l) in coupling.links().iter().enumerate() {
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
    let m = stats::margins(st, pos, &env.norm, &|p| initiated.contains(&p));
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

/// Continental erosion, oceanic dampening, trench accretion and the rock pools (Cortial §4.5,
/// `pools.rs`), then eroded rock moves one hop downhill as sediment (to the lowest lattice
/// neighbour of the same rigid group).
fn surface(st: &mut PlateState, geo: &Geo, env: &Env<'_>, held_floor: &[f32]) {
    let k = env.k;
    let rates = SurfaceRates {
        erosion_m: k.continental_erosion_mm_yr * 1.0e-3 * env.dt_yr,
        damping_m: k.oceanic_damping_mm_yr * 1.0e-3 * env.dt_yr,
        accretion_m: k.sediment_accretion_mm_yr * 1.0e-3 * env.dt_yr,
        pelagic_m: k.pelagic_m_per_myr * env.dt_myr,
        z_c: k.max_elevation_m,
        z_t: k.trench_elevation_m,
        z_floor: k.oceanic_floor_elevation_m,
        trench_relax: k.trench_relax,
        gravity: env.gravity,
    };
    let eroded: Vec<f64> = st
        .samples
        .par_iter_mut()
        .enumerate()
        .map(|(i, s)| surface_step(s, &rates, held_floor.get(i).copied().unwrap_or(f32::NAN)))
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

/// Mature island arcs become continental; uplift rate, heat flow and regime for this step.
fn finish_samples(st: &mut PlateState, geo: &Geo, z0: &[f32], env: &Env<'_>) {
    let t = env.t_myr;
    let collision_myr = env.k.collision_orogeny_myr;
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
    // Island-arc crust becomes continental crust once the arc has built enough of it, whatever
    // its height (2026-10-02). Continental crust is defined by its thickness and buoyancy: arc
    // crust thicker than about 15 km is too buoyant to subduct and jams a trench like a
    // continent, which an arc built on ocean crust reaches after more than about 20 Myr of
    // activity (Cloos 1993). The model's arc magma (`arc_magma_m_per_myr`, 20 m/Myr at the axis
    // at v_0) is not a physical thickness (arcs thicken by several hundred m/Myr, Stern 2010), so
    // the rule counts Cloos's time in it: `arc_maturity_myr` of arc magma at the axis at full
    // rate (ledger: PLT-047). Until 2026-10-02 a mature arc also had to stand above sea level
    // that step; most intra-oceanic arcs are submarine (Stern 2010), and the height the plate
    // model sees is the surface's bent height, so flexure cut the continents' growth (build log
    // "Continental crust bisect").
    let mature_m = (env.k.arc_magma_m_per_myr * env.k.arc_maturity_myr) as f32;
    st.samples.par_iter_mut().enumerate().for_each(|(i, s)| {
        if s.kind == OCEANIC
            && !s.is_hidden()
            && s.arc_oceanic_m > 0.0
            && s.arc_oceanic_m >= mature_m
        {
            s.kind = CONTINENTAL;
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
            let since = (t - f64::from(s.orogeny_myr)).max(0.0);
            let mut q = 55.0 + 35.0 * (-since / 60.0).exp();
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
        };
        s.regime = if s.is_hidden() {
            regime::TRENCH
        } else if f & HOTSPOT != 0 {
            regime::HOTSPOT
        } else if s.orogeny == OROGENY_HIMALAYAN && t - f64::from(s.orogeny_myr) < collision_myr {
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
    });
    st.counters.emerged += st.samples.iter().filter(|s| s.flags & EMERGED != 0).count() as u32;
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
                world.plates = Some(state);
                self.cache = Some(Cache {
                    geo,
                    coupling: None,
                });
            }
            let mut st = world.plates.take().expect("initialised above");
            let result = self.advance(&mut st, world);
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

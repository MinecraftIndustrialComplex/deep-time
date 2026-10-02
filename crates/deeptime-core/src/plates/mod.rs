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
//!    arcs, trenches and the deposit accumulators, and turns subducting plates by slab pull;
//!    With the dynamics record on, it then sums the forces on every plate and solves their
//!    torque balance, which is recorded and not applied (`forces.rs`);
//! 3. keeps each contact's collision state and real shortening (`shortening.rs`) and fires a
//!    collision where the shortening has reached the threshold, moving the terrane to the other
//!    plate with a surge of uplift (`collision.rs`);
//! 4. runs hotspots and large igneous provinces (`hotspots.rs`);
//! 5. applies continental erosion, oceanic dampening and trench accretion, and the rock-pool
//!    processes (`pools.rs`);
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

pub mod check;
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
        self.preset.length_myr() / self.total.max(1) as f64
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
        let z0: Vec<f32> = st.samples.iter().map(|s| s.elevation_m).collect();

        // Motion, then interactions at the new positions.
        kinematics::advance(st, env.dt_yr);
        let cache = self.cache.as_mut().expect("built above");
        let mut pos = geometry::positions(st, &cache.geo);
        st.recount(&pos);
        let tree = Tree::over(&pos, |_| true);
        let overlaps = subduction::detect(st, &cache.geo, &pos, &tree);
        let resolved = subduction::resolve(st, world, &overlaps, &pos, &env)?;
        let fired = shortening::advance(st, &cache.geo, &pos, &tree, &env, resolved.seeds);
        let pull = subduction::fronts_and_uplift(st, &cache.geo, &pos, &env);
        let mix_km = dynamics::boundary_mix(st, &cache.geo, &pos, &env);
        // The force balance, recorded and not applied (`forces.rs`; nothing when the record is
        // off).
        let balance = forces::record(
            st,
            &cache.geo,
            &pos,
            &env,
            forces::flow_axis(self.preset.seed),
        );
        let mut causes: BTreeMap<u32, EventId> = BTreeMap::new();
        let mut sutured = false;
        for (c, seeds) in &fired {
            let under = st.contacts[*c].under;
            let done = collision::collide(st, world, &cache.geo, &pos, &env, *c, seeds)?;
            if let Some(ev) = done.event {
                causes.insert(under, ev);
            }
            sutured |= done.sutured;
        }
        hotspots::step(st, world, &pos, &tree, &env)?;
        surface(st, &cache.geo, &env);
        finish_samples(st, &cache.geo, &z0, &env);
        let arcs = dynamics::arc_heights(st);

        // Motion for the next step: slab pull (axis and speed), the random walk, the speed cap.
        let relax = 1.0 - (-env.dt_myr / k.slab_pull_relax_myr.max(1e-9)).exp();
        let base = 0.5 * (k.plate_speed_mm_yr[0] + k.plate_speed_mm_yr[1]);
        for p in 0..st.plates.len() {
            if !st.plates[p].alive {
                continue;
            }
            let mut w = DVec3::from_array(st.plates[p].omega);
            w = kinematics::pull_axis(
                w,
                pull.axis[p] * k.slab_pull_rad_per_1000km_myr * env.dt_myr,
            );
            let c = DVec3::from_array(st.plates[p].centroid);
            let v = env.norm.speed_mm_yr(w, c);
            if v > 1e-6 {
                let slab =
                    k.slab_pull_speed_mm_yr * (pull.front_km[p] / k.slab_pull_front_km).min(1.0);
                let target = slab.max(base)
                    * (1.0 - (1.0 - k.continental_speed_factor) * st.plates[p].continental_frac);
                w *= (v + (target - v) * relax) / v;
            }
            w = kinematics::jitter(&mut st.rng, w, k.motion_jitter_per_sqrt_myr, env.dt_myr);
            st.plates[p].omega =
                kinematics::cap_speed(w, &env.norm, k.max_plate_speed_mm_yr).to_array();
        }
        rifting::maybe_rift(st, world, &cache.geo, &pos, &env)?;
        rifting::maybe_initiate_subduction(st, world, &cache.geo, &pos, &env)?;

        st.t_myr = env.t_myr;
        st.step = step;
        world.t_years = env.t_myr * 1.0e6;
        for (under, ev) in subduction::close_contacts(st, world, &env, false)? {
            causes.entry(under).or_insert(ev);
        }
        ridges::bury(st, world, None, &pos, &env, &causes)?;

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
                forces: balance.as_ref(),
            },
        );
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
fn surface(st: &mut PlateState, geo: &Geo, env: &Env<'_>) {
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
        .map(|s| surface_step(s, &rates))
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

/// Emerged arcs become continental; uplift rate, heat flow and regime for this step.
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
    // An emerged island arc becomes continental crust once it is a mature arc (P1-27: volcano
    // cones lift young arcs above the sea long before their crust thickens, and with subduction
    // starting in the open ocean every such arc turned into continent).
    let mature_m = env.k.arc_continental_m as f32;
    st.samples.par_iter_mut().enumerate().for_each(|(i, s)| {
        if s.kind == OCEANIC
            && s.elevation_m > 0.0
            && s.flags & ARC != 0
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

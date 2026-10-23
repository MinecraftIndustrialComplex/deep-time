// SPDX-License-Identifier: LGPL-3.0-or-later
//! The ocean–continent transition of rifted (passive) margins: continental crust thins towards the
//! ocean over a width of Earth's, and the crust it loses thickens the ocean floor beside it (Ben,
//! decisions log 2026-10-03: "Merge both, then fix the cause").
//!
//! # Why
//! The plate model steps from a continental column (35 km of crust, a few hundred metres up) to
//! ocean floor (7 km, 2.5–5.6 km down) between two neighbouring samples. On the quick 16k planet a
//! sample is 71 km across on the Earth-sized model sphere but 8.7 km on the planet, and heights are
//! not scaled: the margin is drawn 8.1 times narrower than Earth's, and the surface stage inherits
//! a wall (build log 2026-10-03, "Margins and spires": the plate model's own L1 elevation dropped
//! from +715 m 20 km inland to −1,790 m 10 km out). The taper replaces the step with Earth's width
//! in the model's own frame (the next section); the planet draws it as narrow as it draws every
//! tectonic length (R/R_model of it), with the heights unscaled.
//!
//! # What happens on Earth
//! A passive margin is a rift that broke up. Before break-up the continent was stretched by a
//! factor β that grows towards the future ocean: the crust thins from its normal thickness at the
//! landward limit of thinning (the hinge) to a few kilometres at the continent–ocean boundary, over
//! tens to hundreds of kilometres. Thinned crust stands lower (Airy), and once the hot stretched
//! lithosphere cools the margin subsides with the new ocean floor beside it (McKenzie 1978). The
//! surface over the taper is the shelf (thinned crust under sediment that fills it to wave base),
//! the slope (where the crust thins fastest) and the rise (the distal margin under the sediment
//! apron at the slope's foot).
//!
//! # The rule
//! Each step, for every plate, the samples near the plate's own ocean–continent boundary (an edge
//! between a visible continental and a visible oceanic sample of the plate, neither at a trench,
//! in a subduction zone, on an arc or, for continental crust, raised by an orogeny within
//! `collapse_quiet_myr`; oceanic crust thicker than T_n, which the plate model stacks at
//! resamplings and accretes from slabs under the oceanic label, takes no part) get a signed
//! distance d from that boundary on the model sphere (positive inland;
//! the boundary lies half a sample spacing beyond each boundary sample), and a target crystalline
//! thickness (the crust less its sediment and sedimentary rock)
//!
//!   T*(d) = T_o + (T_n − T_o) · clamp(1/2 + d / W, 0, 1),
//!
//! T_n the normal continental column (33.0 km of crystalline crust in [`continental_column`]),
//! T_o the ocean's (7 km, [`oceanic_column`]) and W = `transition_width_km`, km on the model
//! sphere (Earth-normalized like the other plate lengths; Ben, decisions log 2026-10-22: "200 km
//! on sphere"). The taper is linear in thickness (β = T_n / T rises from 1 at the hinge, d = W/2,
//! to T_n / T_o at the toe, d = −W/2) and centred on the model's boundary: stretching a step from
//! T_n to T_o into this ramp keeps the crust's volume. Crust moves from the samples thicker than
//! T* to the samples thinner than T* within each stretch of margin (a catchment about W/2 long,
//! below), the share 1 − e^(−δt/τ) of what would bring the stretch to the profile
//! (τ = `transition_myr`). The sources thin in proportion (pure shear) and the sinks take their
//! rock. With `transition_ocean_crust` (PLT-294; on in every shipped preset since 2026-10-20, the
//! coordinator's call in the decisions log, off in `TectonicParams::default`) an oceanic sink
//! takes the same volume as ocean crust instead (lavas and gabbro in the ocean column's
//! proportion), as the
//! thick igneous crust of a volcanic margin (from-memory: White & McKenzie 1989, not read), the
//! commoner kind (Franke 2013, read; ledger PLT-294, a game-choice): the crust-template
//! review of 2026-10-19 (W then 200 km on the planet, 1,632 km on the model sphere of the quick
//! planet) found the sources' rock is the main source of granite and gneiss under
//! the sea on the quick planet, whose oceans lay mostly within W/2 of a margin, but the
//! continental rock the switch stops parking on the ocean side is then lost, and with it some land
//! (about 0.04 of the planet on supercontinent starts on master d54c7f0f; with the cordillera at
//! 1.7 mm/yr, 2026-10-20, the quick ensemble's land 0.27 on both starts, Earth's 0.29). Crust
//! types do not change: the thickened ocean-side crust stays oceanic, so the continental crust
//! share and land are not redefined by the rule.
//!
//! Heights follow the crust along the line through two reference columns: a continent of T_n at
//! z_0 (the middle of `continent_elevation_m`) and old ocean floor of T_o at z_f
//! (`oceanic_floor_elevation_m`). A metre of crust moved changes the height by 1/f,
//! f = (T_n − T_o)/(z_0 − z_f) (4.24 with the presets' 525 m and −5,600 m, between the model's dry
//! and water-loaded Airy factors, 5.5 and 3.78), on both sides, so the two sides of the boundary
//! meet. The ocean side then cools with its own age (the dampening, `pools::surface_step`), but
//! towards the floor its crust holds up, z_f + (min(T, T*) − T_o)/f, not the bare floor's (never
//! above what the taper's own ocean-side crust holds up, about ridge depth): a young margin
//! stands near ridge depth and the taper appears as the ocean beside it subsides (McKenzie's
//! thermal subsidence, carried by the ocean's age–depth law).
//!
//! The plate model's height change is its uplift; the surface stage takes it as crust thickened or
//! thinned under an Airy root and loads its flexure with it (`surface/flexure.rs`), so the shelf,
//! slope and rise come from the taper, the sediment the surface brings and its slope failure.
//!
//! # Stage 2: a stretching factor (`tectonics.transition_beta`)
//! With the switch on (off by default; lithosphere-thermal-state.md, "Stage 2") the margin is
//! drawn as a stretching factor β = T_n / T* across it rather than a linear ramp of thickness, on
//! both sides of the boundary (each conjugate margin is its own plate's boundary):
//!
//!   T*(d) = T_o + (T_n − T_o) · S(1/2 + d / W),  S(s) = 3s² − 2s³ on [0, 1],
//!
//! so β stays under about 1.15 over the inner quarter of the zone (the proximal domain, crust
//! 30 ± 5 km, "only a little thinned"), rises fastest at the boundary (the necking domain, where
//! "the crust is thinned from 30 ± 5 km to ca. 10 km") and is 3 or more over the outer quarter
//! (the hyperextended domain; Manatschal & Chenin 2023 after Chenin et al. 2017). S is symmetric
//! about s = 1/2 like the linear ramp, so it holds the step's crust; W, τ, the catchments and the
//! height line are unchanged (PLT-475, the form a game-choice).
//!
//! A continental source thinned to `keep` of its crystalline crust (β = 1 / `keep` for that step)
//! has its lithosphere and keel thinned by the same factor, L / β and K / β (McKenzie 1978's
//! uniform stretching; `thermal::stretch`, PLT-476). The state then cools it back over τ_c, and
//! with `thermal_isostasy_m` on its thermal height z_th falls by the same law: McKenzie's thermal
//! subsidence of the margin. Without `tectonics.thermal_state` there is no lithosphere to thin
//! and only the crust's profile changes; with the state but no thermal isostasy the stretched
//! margin is hotter and weaker (heat flow, `thermal_strength`) and its height is the crust's.
//!
//! # Catchments
//! Crust moves within a stretch of margin, not across the plate: boundary samples are grouped
//! round seeds picked along the boundary at least W/2 apart (in index order, so a run is
//! reproducible), every sample of the zone belongs to the seed of its nearest boundary sample (on
//! its own side, through its own plate), and a stretch balances its sources against its sinks
//! (a narrow peninsula has more ocean than continent: its continent thins to the profile and the
//! ocean beside it gets what there is).
// ledger: PLT-261 (the rule), PLT-262 (the width), PLT-263 (τ), PLT-264 (the height line),
// PLT-294 (the ocean side's rock).

use std::collections::HashMap;

use glam::DVec3;
use rayon::prelude::*;

use super::Env;
use super::geometry::{Geo, dijkstra_sparse};
use super::pools::{continental_column, oceanic_column};
use super::state::{ARC, CONTINENTAL, HIDDEN, OCEANIC, PlateState, SUBDUCTION_ZONE, TRENCH, pool};

/// `WorldState::globals`: crust moved by the transition over the run, km³.
pub const MOVED_GLOBAL: &str = "plates/transition_km3";

/// The pools that count as crystalline crust (not sediment or sedimentary rock).
const CRYSTALLINE: [usize; 5] = [
    pool::METAMORPHIC,
    pool::FELSIC_PLUTONIC,
    pool::FELSIC_VOLCANIC,
    pool::MAFIC_VOLCANIC,
    pool::MAFIC_PLUTONIC,
];

/// Crystalline thickness of a column, m.
pub fn crystalline_m(pools: &[f32; pool::COUNT]) -> f64 {
    CRYSTALLINE.iter().map(|&k| f64::from(pools[k])).sum()
}

/// The normal continental column's crystalline thickness T_n, m (33.0 km).
pub fn continental_m() -> f64 {
    crystalline_m(&continental_column())
}

/// The ocean's crystalline thickness T_o, m (7 km).
pub fn oceanic_m() -> f64 {
    crystalline_m(&oceanic_column())
}

/// The height line through the continent's and the old ocean floor's columns (see the module
/// docs): metres of crust per metre of height, f = (T_n − T_o)/(z_0 − z_f).
pub fn crust_per_metre(env: &Env) -> f64 {
    let k = env.k;
    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
    // ledger: PLT-264.
    (continental_m() - oceanic_m()) / (z_0 - k.oceanic_floor_elevation_m).max(1.0)
}

/// The floor oceanic crust of `crystalline` metres cools towards, m: the bare floor z_f, raised by
/// the crust beyond the ocean's own, but by no more than the taper's `target` holds up (see the
/// module docs: the plate model keeps stacked and accreted crust under the oceanic label, and
/// that crust is not a margin's).
pub fn held_floor_m(crystalline: f64, target: f64, z_f: f64, f: f64) -> f64 {
    z_f + (crystalline.min(target) - oceanic_m()).max(0.0) / f
}

/// The share of the way from the ocean's crust to the continent's that the target stands at, at
/// `s` = 1/2 + d/W in [0, 1] (0 the toe, 1 the hinge): `s` itself for the linear taper (PLT-261);
/// with `transition_beta` (stage 2) the smoothstep 3s² − 2s³, whose stretching factor
/// β = T_n / T* stays near 1 over the proximal margin, rises fastest at the boundary (the necking
/// zone) and stays high over the distal margin. Both are symmetric about s = 1/2, so both hold
/// the step's crust (see the module docs).
// ledger: PLT-261, PLT-475
pub fn shape(s: f64, beta: bool) -> f64 {
    let s = s.clamp(0.0, 1.0);
    if beta { s * s * (3.0 - 2.0 * s) } else { s }
}

/// What one step did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Transition {
    /// Samples in a transition zone (within W/2 of their plate's ocean–continent boundary).
    pub zone: u32,
    /// Crust moved, km² × m (Earth-normalized area).
    pub moved_km2_m: f64,
    /// Per sample: the floor an oceanic sample of the zone cools towards (`held_floor_m`), NaN
    /// elsewhere; `pools::surface_step` reads it.
    pub floor_m: Vec<f32>,
}

/// The transition's width W as an angle on the unit sphere: `transition_width_km` is a length on
/// the model sphere (R_model, `tectonics.model_radius_km`), so the planet's radius does not enter.
pub fn width_rad(env: &Env) -> f64 {
    // ledger: PLT-262.
    env.norm.rad(env.k.transition_width_km)
}

/// The seeds near a point (P1-19): seed numbers hashed by the cube of side `cell` they fall in,
/// `cell` no less than the chord of the seed spacing W/2, so every seed whose direction is within
/// W/2 of a point (dot product at least `cos_sep`) is in one of the 27 cubes round it. A filter
/// only: the caller makes the exact test on what it visits, so the seeds and catchments are the
/// ones a scan over every seed finds.
struct Near {
    cell: f64,
    cubes: HashMap<[i32; 3], Vec<u32>>,
}

impl Near {
    fn new(cos_sep: f64) -> Near {
        // The chord of the angle whose cosine is `cos_sep` on the unit sphere, with room for the
        // rounding of positions a few ulps off unit length.
        let cell = (2.0 * (1.0 - cos_sep) + 1e-9).max(0.0).sqrt() + 1e-9;
        Near {
            cell,
            cubes: HashMap::new(),
        }
    }

    fn cube(&self, p: DVec3) -> [i32; 3] {
        let c = |x: f64| ((x + 1.0) / self.cell).floor() as i32;
        [c(p.x), c(p.y), c(p.z)]
    }

    fn insert(&mut self, p: DVec3, seed: u32) {
        let key = self.cube(p);
        self.cubes.entry(key).or_default().push(seed);
    }

    /// Calls `f` with every seed in the 27 cubes round `p`.
    fn visit(&self, p: DVec3, mut f: impl FnMut(u32)) {
        let [x, y, z] = self.cube(p);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(list) = self.cubes.get(&[x + dx, y + dy, z + dz]) {
                        list.iter().for_each(|&i| f(i));
                    }
                }
            }
        }
    }
}

/// One step of the transition on `state` at positions `pos` (see the module docs). Runs after the
/// step's uplift and collapse, before the surface processes.
pub fn step(state: &mut PlateState, geo: &Geo, pos: &[DVec3], env: &Env) -> Transition {
    let n = state.samples.len();
    let mut out = Transition {
        floor_m: vec![f32::NAN; n],
        ..Transition::default()
    };
    let k = env.k;
    let w = width_rad(env);
    if w.is_nan() || w <= 0.0 || k.transition_myr.is_nan() || k.transition_myr <= 0.0 {
        return out;
    }
    let rate = 1.0 - (-env.dt_myr / k.transition_myr).exp();
    let half = 0.5 * env.norm.spacing_rad();
    let (t_n, t_o) = (continental_m(), oceanic_m());
    let f = crust_per_metre(env);
    let quiet_before = (env.t_myr - k.collapse_quiet_myr.max(0.5 * env.dt_myr)) as f32;
    let samples = &state.samples;
    let groups = &state.groups;
    // P1-19: the per-sample reads are parallel collects (each keeps the samples' order).
    let plate_of: Vec<u32> = samples
        .par_iter()
        .map(|s| groups[s.group as usize].plate)
        .collect();
    // Who takes part: visible, not at a trench, in a subduction zone or on an arc; continental
    // crust not raised by an orogeny lately (an active margin or a belt is not a rifted margin);
    // oceanic crust no thicker than the continent's (the plate model keeps crust stacked at
    // resamplings and accreted from slabs under the oceanic label, up to hundreds of km on
    // master: that is not the ocean floor beside a rifted margin, and it neither gives nor takes).
    let eligible: Vec<bool> = samples
        .par_iter()
        .map(|s| {
            s.flags & (HIDDEN | TRENCH | SUBDUCTION_ZONE | ARC) == 0
                && if s.kind == OCEANIC {
                    crystalline_m(&s.pools) <= t_n
                } else {
                    s.orogeny_myr < quiet_before
                }
        })
        .collect();
    let same = |a: u32, b: u32| {
        eligible[a as usize] && eligible[b as usize] && plate_of[a as usize] == plate_of[b as usize]
    };
    // Boundary samples: eligible, with an eligible neighbour of the other crust on the same plate.
    let boundary: Vec<u32> = (0..n as u32)
        .into_par_iter()
        .filter(|&v| {
            eligible[v as usize]
                && geo.edges(v).any(|(u, _)| {
                    same(v, u) && samples[u as usize].kind != samples[v as usize].kind
                })
        })
        .collect();
    if boundary.is_empty() {
        return out;
    }
    // Seeds along each plate's boundary, at least W/2 apart (index order), and each boundary
    // sample's seed (of its own plate: crust never moves between plates). P1-19: a seed only
    // looks at the seeds of the cubes round it (`Near`), which hold every seed within W/2.
    let cos_sep = (0.5 * w).min(std::f64::consts::PI).cos();
    let mut seeds: Vec<u32> = Vec::new();
    let mut near = Near::new(cos_sep);
    for &v in &boundary {
        let p = pos[v as usize];
        let pl = plate_of[v as usize];
        let mut far = true;
        near.visit(p, |i| {
            let s = seeds[i as usize];
            far &= plate_of[s as usize] != pl || pos[s as usize].dot(p) < cos_sep;
        });
        if far {
            near.insert(p, seeds.len() as u32);
            seeds.push(v);
        }
    }
    // The seed of `v`: the nearest seed of its plate, the first in index order on a tie. It is
    // never farther than W/2 (a boundary sample is a seed or has one that near), so the cubes
    // round `v` hold it.
    let seed_of = |v: u32| -> u32 {
        let p = pos[v as usize];
        let pl = plate_of[v as usize];
        let mut best = (f64::NEG_INFINITY, u32::MAX);
        near.visit(p, |i| {
            let s = seeds[i as usize];
            if plate_of[s as usize] != pl {
                return;
            }
            let c = pos[s as usize].dot(p);
            if c > best.0 || (c == best.0 && i < best.1) {
                best = (c, i);
            }
        });
        best.1
    };
    let seed_at: Vec<u32> = boundary.par_iter().map(|&v| seed_of(v)).collect();
    // Distances from the boundary on each side, through each side's own crust of the plate (the
    // two sides reach disjoint samples, so they run at once; P1-19).
    let reach = 0.5 * w + half;
    let side = |kind: u8| -> Vec<(u32, f64, u32)> {
        let src: Vec<(u32, f64, u32)> = boundary
            .iter()
            .zip(&seed_at)
            .filter(|&(&v, _)| samples[v as usize].kind == kind)
            .map(|(&v, &seed)| (v, 0.0, seed))
            .collect();
        dijkstra_sparse(
            geo,
            &src,
            |a, b| same(a, b) && samples[b as usize].kind == kind,
            reach,
        )
    };
    let (reach_c, reach_o) = rayon::join(|| side(CONTINENTAL), || side(OCEANIC));

    // The zone: signed distance and target, per sample, in sample order (a sample reached from
    // the continental side is continental, from the oceanic side oceanic).
    struct Member {
        i: usize,
        seed: u32,
        target: f64,
        excess: f64,
    }
    let member = |i: u32, d: f64, seed: u32| -> Option<Member> {
        if d.abs() >= 0.5 * w || seed == u32::MAX {
            return None;
        }
        let i = i as usize;
        // ledger: PLT-261, PLT-475.
        let target = t_o + (t_n - t_o) * shape(0.5 + d / w, k.transition_beta);
        Some(Member {
            i,
            seed,
            target,
            excess: crystalline_m(&samples[i].pools) - target,
        })
    };
    let mut members: Vec<Member> = reach_c
        .par_iter()
        .filter_map(|&(i, d, seed)| member(i, d + half, seed))
        .chain(
            reach_o
                .par_iter()
                .filter_map(|&(i, d, seed)| member(i, -(d + half), seed)),
        )
        .collect();
    members.par_sort_unstable_by_key(|m| m.i);
    out.zone = members.len() as u32;

    // Each stretch: sources against sinks (per seed, summed in the members' order).
    let mut stretch: Vec<(f64, f64)> = vec![(0.0, 0.0); seeds.len()];
    for m in &members {
        let e = &mut stretch[m.seed as usize];
        if m.excess > 0.0 {
            e.0 += m.excess;
        } else {
            e.1 -= m.excess;
        }
    }
    // The rock the sources give, per stretch, and the share each side moves this step.
    let mut mix: Vec<Option<[f64; pool::COUNT]>> = vec![None; seeds.len()];
    let lo = k.trench_elevation_m;
    let hi = k.max_elevation_m;
    for m in &members {
        let (give, take) = stretch[m.seed as usize];
        let moved = rate * give.min(take);
        if moved <= 0.0 || m.excess <= 0.0 {
            continue;
        }
        let g = m.excess * moved / give;
        let s = &mut state.samples[m.i];
        let cryst = crystalline_m(&s.pools);
        let keep = ((cryst - g) / cryst).clamp(0.0, 1.0);
        let acc = mix[m.seed as usize].get_or_insert([0.0; pool::COUNT]);
        for &p in &CRYSTALLINE {
            let before = f64::from(s.pools[p]);
            s.pools[p] = (before * keep) as f32;
            acc[p] += before - f64::from(s.pools[p]);
        }
        s.basement_m *= keep as f32;
        s.thickness_m = s.pools.iter().sum();
        s.elevation_m = (f64::from(s.elevation_m) - g / f).clamp(lo, hi) as f32;
        if k.transition_beta {
            // Stage 2: the lithosphere stretches with the crust (ledger: PLT-476).
            super::thermal::stretch(s, keep);
        }
        out.moved_km2_m += g * env.norm.sample_area_km2();
    }
    // What an oceanic sink takes: the ocean column's own rock in its own proportion.
    let ocean = oceanic_column().map(f64::from);
    let ocean_total: f64 = ocean.iter().sum();
    for m in &members {
        if m.excess >= 0.0 {
            continue;
        }
        let Some(acc) = &mix[m.seed as usize] else {
            continue;
        };
        let total: f64 = acc.iter().sum();
        if total <= 0.0 {
            continue;
        }
        let (give, take) = stretch[m.seed as usize];
        let moved = rate * give.min(take);
        let g = -m.excess * moved / take;
        let s = &mut state.samples[m.i];
        // With `transition_ocean_crust`, ocean floor thickens with ocean crust, the lavas and
        // gabbro of a volcanic margin, never with the continent's granite and gneiss; otherwise,
        // and on a continental sink, with the sources' rock. ledger: PLT-294.
        let (add, sum) = if k.transition_ocean_crust && s.kind == OCEANIC {
            (&ocean, ocean_total)
        } else {
            (acc, total)
        };
        for (x, a) in s.pools.iter_mut().zip(add.iter()) {
            *x += (g * a / sum) as f32;
        }
        s.thickness_m = s.pools.iter().sum();
        s.elevation_m = (f64::from(s.elevation_m) + g / f).clamp(lo, hi) as f32;
    }
    // The floor the zone's ocean crust cools towards.
    let z_f = k.oceanic_floor_elevation_m;
    for m in &members {
        let s = &state.samples[m.i];
        if s.kind == OCEANIC {
            out.floor_m[m.i] = held_floor_m(crystalline_m(&s.pools), m.target, z_f, f) as f32;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::TectonicParams;
    use crate::plates::geometry::positions;
    use crate::plates::normalise::Norm;
    use crate::plates::pools::{SurfaceRates, surface_step};
    use crate::plates::state::{Group, Plate, Sample};

    const N: usize = 16_000;
    /// The quick 16k planet's radius, km (16,384 blocks of 300 m round).
    const PLANET_KM: f64 = 16_384.0 * 0.3 / std::f64::consts::TAU;
    /// The test sphere's radius, km: the size of the quick 16k planet, so that the shipped 200 km
    /// margin spans the 9 samples it did when W was measured on the planet (N = 16,000). On
    /// Earth's 6,370 km sphere a sample is 178 km across at this N and a 200 km margin is a
    /// sample wide, too coarse for a test of the taper's shape.
    const MODEL_KM: f64 = PLANET_KM;
    /// cos 40°: the continent is the cap within 40° of +y.
    const COAST_Y: f64 = 0.766;

    fn plate(id: u32) -> Plate {
        Plate {
            id,
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
            continental_frac: 0.5,
        }
    }

    /// Plate 0 is x < 0, plate 1 x ≥ 0; on both a continent (35.1 km, 300 m) within 40° of +y and
    /// old ocean floor (7 km, −5,000 m) elsewhere, a step from one to the other between
    /// neighbouring samples, as the plate model has it. Plate 1's samples within `active` (rad) of
    /// the coast are in a subduction zone.
    fn planet(geo: &Geo, active: f64) -> PlateState {
        let coast = COAST_Y.acos();
        let samples: Vec<Sample> = geo
            .points()
            .iter()
            .map(|&p| {
                let mut s = Sample::blank(u32::from(p.x >= 0.0));
                if p.y > COAST_Y {
                    s.kind = CONTINENTAL;
                    s.pools = continental_column();
                    s.elevation_m = 300.0;
                } else {
                    s.pools = oceanic_column();
                    s.elevation_m = -5000.0;
                    s.formed_myr = -200.0;
                }
                if p.x >= 0.0 && (p.y.clamp(-1.0, 1.0).acos() - coast).abs() < active {
                    s.flags |= SUBDUCTION_ZONE;
                }
                s.thickness_m = s.pools.iter().sum();
                s
            })
            .collect();
        let ident = [0.0, 0.0, 0.0, 1.0];
        let mut st = PlateState {
            n: N as u32,
            lattice: ident,
            samples,
            groups: vec![
                Group {
                    plate: 0,
                    rot: ident,
                },
                Group {
                    plate: 1,
                    rot: ident,
                },
            ],
            plates: vec![plate(0), plate(1)],
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
        };
        let pos = positions(&st, geo);
        st.recount(&pos);
        st
    }

    struct Fx {
        k: TectonicParams,
        geo: Geo,
        norm: Norm,
    }

    impl Fx {
        fn new(model_km: f64) -> Fx {
            let k = TectonicParams {
                transition_width_km: 200.0,
                transition_myr: 2.0,
                collapse_quiet_myr: 0.0,
                // The pipeline presets' ocean floor (GDH1 fit, `pools.rs`).
                oceanic_floor_elevation_m: -5600.0,
                oceanic_damping_mm_yr: 0.12,
                ..TectonicParams::default()
            };
            Fx {
                k,
                geo: Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap(),
                norm: Norm::new(model_km, PLANET_KM, N),
            }
        }

        fn with(k: TectonicParams) -> Fx {
            Fx {
                k,
                ..Fx::new(MODEL_KM)
            }
        }

        fn env(&self, step: u64) -> Env<'_> {
            Env {
                k: &self.k,
                norm: self.norm,
                dt_myr: 2.0,
                dt_yr: 2.0e6,
                gravity: 9.81,
                step,
                t_myr: 2.0 * step as f64,
            }
        }

        /// `steps` steps on fixed positions; the last step's result.
        fn run(&self, st: &mut PlateState, steps: u64) -> Transition {
            let pos = positions(st, &self.geo);
            let mut last = Transition::default();
            for n in 1..=steps {
                last = step(st, &self.geo, &pos, &self.env(n));
            }
            last
        }
    }

    /// Signed distance from the coast on the test sphere (`MODEL_KM`), km (positive inland).
    fn inland_km(p: DVec3) -> f64 {
        (COAST_Y.acos() - p.y.clamp(-1.0, 1.0).acos()) * MODEL_KM
    }

    /// Mean of `f` over plate `plate`'s samples away from the plates' edges, in bins of `bin_km`
    /// by distance from the coast, from −`span` to +`span` km: (bin centre, mean).
    fn profile(
        st: &PlateState,
        geo: &Geo,
        plate: u32,
        bin_km: f64,
        span: f64,
        f: impl Fn(&Sample) -> f64,
    ) -> Vec<(f64, f64)> {
        let nb = (2.0 * span / bin_km).round() as usize;
        let mut sum = vec![(0.0, 0usize); nb];
        for (i, s) in st.samples.iter().enumerate() {
            let p = geo.points()[i];
            if st.plate_of(i) != plate || p.x.abs() < 0.2 {
                continue;
            }
            let b = ((inland_km(p) + span) / bin_km).floor();
            if b >= 0.0 && (b as usize) < nb {
                sum[b as usize].0 += f(s);
                sum[b as usize].1 += 1;
            }
        }
        sum.iter()
            .enumerate()
            .filter(|(_, (_, c))| *c > 0)
            .map(|(b, (v, c))| (-span + (b as f64 + 0.5) * bin_km, v / *c as f64))
            .collect()
    }

    fn crystalline_total(st: &PlateState, plate: u32) -> f64 {
        (0..st.samples.len())
            .filter(|&i| st.plate_of(i) == plate)
            .map(|i| crystalline_m(&st.samples[i].pools))
            .sum()
    }

    /// The model's step from continent to ocean becomes a taper W wide: the crystalline crust
    /// follows the taper across the zone (about half way at the old boundary), the ground falls
    /// with it with no drop the size of the old step, the volume is kept, crust types do not
    /// change, and nothing moves beyond W/2. With the rule off, the whole fall (5.3 km) comes
    /// between two neighbouring bins.
    #[test]
    fn a_rifted_margin_tapers_over_its_width_and_keeps_its_crust() {
        let fx = Fx::new(MODEL_KM);
        let mut st = planet(&fx.geo, 0.0);
        let before = st.samples.clone();
        let vol0 = crystalline_total(&st, 0);
        fx.run(&mut st, 30);
        let vol = crystalline_total(&st, 0);
        assert!((vol - vol0).abs() < 1e-6 * vol0, "volume {vol} vs {vol0}");
        let w = fx.k.transition_width_km;
        for (i, s) in st.samples.iter().enumerate() {
            assert_eq!(s.kind, before[i].kind, "crust types stay");
            let d = inland_km(fx.geo.points()[i]);
            if d.abs() > 0.5 * w + 15.0 {
                assert_eq!(s, &before[i], "beyond the zone, at {d:.0} km");
            }
        }
        let (t_n, t_o) = (continental_m(), oceanic_m());
        let thick = profile(&st, &fx.geo, 0, 20.0, 160.0, |s| crystalline_m(&s.pools));
        for &(d, t) in &thick {
            let want = t_o + (t_n - t_o) * (0.5 + d / w).clamp(0.0, 1.0);
            assert!(
                (t - want).abs() < 2500.0,
                "{d:.0} km: {t:.0} m against the taper's {want:.0}"
            );
        }
        let z = profile(&st, &fx.geo, 0, 20.0, 160.0, |s| f64::from(s.elevation_m));
        for pair in z.windows(2) {
            let rise = pair[1].1 - pair[0].1;
            assert!(rise > -50.0, "the ground rises inland: {z:?}");
            assert!(rise < 1500.0, "no step near the old 5.3 km: {z:?}");
        }
        let mut old = planet(&fx.geo, 0.0);
        Fx::with(TectonicParams {
            transition_width_km: 0.0,
            ..fx.k.clone()
        })
        .run(&mut old, 30);
        let z0 = profile(&old, &fx.geo, 0, 20.0, 160.0, |s| f64::from(s.elevation_m));
        let old_step = z0.windows(2).map(|p| p[1].1 - p[0].1).fold(0.0, f64::max);
        assert!(
            old_step > 3000.0,
            "the old model steps at the coast: {z0:?}"
        );
    }

    /// With `transition_ocean_crust` the ocean side thickens with ocean crust (PLT-294): no
    /// continental granite, gneiss or felsic lava reaches an oceanic sample, its lavas and gabbro
    /// grow in the ocean column's proportion, and the continent side still loses its own rock.
    #[test]
    fn the_ocean_side_thickens_with_ocean_crust_not_the_continents_rock() {
        let base = Fx::new(MODEL_KM);
        let fx = Fx::with(TectonicParams {
            transition_ocean_crust: true,
            ..base.k.clone()
        });
        let mut st = planet(&fx.geo, 0.0);
        let before = st.samples.clone();
        fx.run(&mut st, 30);
        let ocean = oceanic_column();
        let ratio = f64::from(ocean[pool::MAFIC_VOLCANIC]) / f64::from(ocean[pool::MAFIC_PLUTONIC]);
        let (mut thickened, mut thinned) = (0, 0);
        for (i, s) in st.samples.iter().enumerate() {
            let b = &before[i];
            if s.kind == OCEANIC {
                for p in [
                    pool::FELSIC_PLUTONIC,
                    pool::FELSIC_VOLCANIC,
                    pool::METAMORPHIC,
                ] {
                    assert_eq!(s.pools[p], b.pools[p], "sample {i}, pool {p}");
                }
                let mv = f64::from(s.pools[pool::MAFIC_VOLCANIC] - b.pools[pool::MAFIC_VOLCANIC]);
                let mp = f64::from(s.pools[pool::MAFIC_PLUTONIC] - b.pools[pool::MAFIC_PLUTONIC]);
                if mp > 100.0 {
                    thickened += 1;
                    assert!((mv / mp - ratio).abs() < 1e-3, "sample {i}: {mv} over {mp}");
                }
            } else if s.pools[pool::FELSIC_PLUTONIC] < b.pools[pool::FELSIC_PLUTONIC] - 100.0 {
                thinned += 1;
            }
        }
        assert!(
            thickened > 20 && thinned > 20,
            "{thickened} thickened, {thinned} thinned"
        );
        // Off (the default), the ocean side takes the continent's own rock.
        let mut st = planet(&base.geo, 0.0);
        base.run(&mut st, 30);
        let parked = (0..st.samples.len())
            .filter(|&i| {
                st.samples[i].kind == OCEANIC
                    && st.samples[i].pools[pool::FELSIC_PLUTONIC]
                        > before[i].pools[pool::FELSIC_PLUTONIC] + 100.0
            })
            .count();
        assert!(parked > 20, "{parked} ocean samples took granite");
    }

    /// The ocean side's thicker crust gives it a held floor, and cooling takes it towards that
    /// floor, not the bare one; ocean floor beyond the taper cools as before.
    #[test]
    fn the_ocean_side_cools_towards_the_floor_its_crust_holds_up() {
        let fx = Fx::new(MODEL_KM);
        let mut st = planet(&fx.geo, 0.0);
        let last = fx.run(&mut st, 30);
        let f = crust_per_metre(&fx.env(1));
        let rates = SurfaceRates {
            erosion_m: 0.0,
            damping_m: fx.k.oceanic_damping_mm_yr * 1.0e-3 * 2.0e6,
            accretion_m: 0.0,
            pelagic_m: 0.0,
            z_c: fx.k.max_elevation_m,
            z_t: fx.k.trench_elevation_m,
            z_floor: fx.k.oceanic_floor_elevation_m,
            z_ridge: fx.k.ridge_elevation_m,
            trench_relax: 0.0,
            trench_release: 0.0,
            gravity: 9.81,
            law: crate::params::OceanDepth::Dampening,
            t_myr: 0.0,
            dt_myr: 2.0,
            sea_m: 0.0,
        };
        let mut held = 0;
        for (i, s) in st.samples.iter().enumerate() {
            if s.kind != OCEANIC || st.plate_of(i) != 0 {
                continue;
            }
            let cryst = crystalline_m(&s.pools);
            if cryst < oceanic_m() + 2000.0 {
                continue;
            }
            held += 1;
            let floor = last.floor_m[i];
            let want = f64::from(floor);
            let unbounded = held_floor_m(cryst, cryst, fx.k.oceanic_floor_elevation_m, f);
            assert!(
                want.is_finite() && want <= unbounded + 1.0,
                "{floor} vs {unbounded}"
            );
            let (mut kept, mut bare) = (*s, *s);
            for _ in 0..250 {
                surface_step(&mut kept, &rates, floor);
                surface_step(&mut bare, &rates, f32::NAN);
            }
            assert!(
                (f64::from(kept.elevation_m) - want).abs() < 250.0,
                "{} vs {want}",
                kept.elevation_m
            );
            // The bare floor ends lower by about what the extra crust holds up.
            let held_up = want - fx.k.oceanic_floor_elevation_m;
            assert!(
                f64::from(kept.elevation_m - bare.elevation_m) > 0.8 * held_up,
                "{} vs {} ({held_up:.0} m held up)",
                kept.elevation_m,
                bare.elevation_m
            );
        }
        assert!(held > 20, "{held}");
        // Ocean floor with the ocean's own crust: the same as with no held floor.
        let mut a = Sample::blank(0);
        a.pools = oceanic_column();
        a.elevation_m = -3000.0;
        let mut b = a;
        let floor =
            held_floor_m(oceanic_m(), oceanic_m(), fx.k.oceanic_floor_elevation_m, f) as f32;
        surface_step(&mut a, &rates, floor);
        surface_step(&mut b, &rates, f32::NAN);
        assert!((a.elevation_m - b.elevation_m).abs() < 1e-3);
    }

    /// The plate model keeps crust stacked at resamplings under the oceanic label (up to hundreds of
    /// km on master). Such a column beside a margin is not the margin's ocean floor: it neither
    /// gives nor takes crust and holds no floor up, and no ocean floor of the zone is held above
    /// what the taper's thickest ocean-side crust holds up (ridge depth, near −2.5 km).
    #[test]
    fn stacked_crust_under_the_ocean_label_is_not_a_margin() {
        let fx = Fx::new(MODEL_KM);
        let mut st = planet(&fx.geo, 0.0);
        let mut stacked = Vec::new();
        for (i, s) in st.samples.iter_mut().enumerate() {
            let d = inland_km(fx.geo.points()[i]);
            if s.kind == OCEANIC && (-40.0..-20.0).contains(&d) && i % 3 == 0 {
                s.pools[pool::FELSIC_PLUTONIC] += 93_000.0;
                s.thickness_m = s.pools.iter().sum();
                stacked.push(i);
            }
        }
        assert!(stacked.len() > 10, "{}", stacked.len());
        let before = st.samples.clone();
        let last = fx.run(&mut st, 30);
        let f = crust_per_metre(&fx.env(1));
        let z_f = fx.k.oceanic_floor_elevation_m;
        let ceiling = held_floor_m(0.5 * (continental_m() + oceanic_m()), f64::INFINITY, z_f, f);
        for &i in &stacked {
            assert_eq!(st.samples[i].pools, before[i].pools, "stacked column {i}");
            assert!(last.floor_m[i].is_nan());
        }
        let held: Vec<f32> = last
            .floor_m
            .iter()
            .copied()
            .filter(|x| x.is_finite())
            .collect();
        assert!(held.len() > 20);
        for x in held {
            assert!(f64::from(x) <= ceiling + 1.0, "{x} over {ceiling:.0}");
        }
    }

    /// An active margin (in a subduction zone) keeps its step; with the width or the time at 0
    /// nothing moves; the width is on the model sphere (PLT-262), so the planet's size does not
    /// enter and a model sphere twice as large has a zone half as wide as an angle.
    #[test]
    fn active_margins_and_switched_off_rules_keep_their_step() {
        let fx = Fx::new(MODEL_KM);
        let mut st = planet(&fx.geo, 0.4);
        let before = st.samples.clone();
        fx.run(&mut st, 5);
        for (i, s) in st.samples.iter().enumerate() {
            if st.plate_of(i) == 1 {
                assert_eq!(s, &before[i]);
            }
        }
        assert!(st.samples != before, "plate 0's rifted margin tapers");
        for k in [
            TectonicParams {
                transition_width_km: 0.0,
                ..fx.k.clone()
            },
            TectonicParams {
                transition_myr: 0.0,
                ..fx.k.clone()
            },
        ] {
            let off = Fx::with(k);
            let mut st = planet(&off.geo, 0.0);
            let before = st.samples.clone();
            off.run(&mut st, 3);
            assert_eq!(st.samples, before);
        }
        let big = Fx::new(2.0 * MODEL_KM);
        assert!((width_rad(&fx.env(1)) - 2.0 * width_rad(&big.env(1))).abs() < 1e-12);
        // The planet's radius does not enter: the shipped 200 km on Earth's 6,370 km sphere is
        // 200/6370 of a radian on the 16k planet and on one twice its size.
        for planet_km in [PLANET_KM, 2.0 * PLANET_KM] {
            let env = Env {
                norm: Norm::new(6370.0, planet_km, N),
                ..fx.env(1)
            };
            assert_eq!(width_rad(&env), 200.0 / 6370.0);
        }
    }

    /// Stage 2's profile holds the step's crust as the linear taper does (both are symmetric about
    /// the boundary, so their mean is the mean of the two crusts), meets both crusts at the zone's
    /// ends, and is flatter than the taper near the hinge (the proximal margin, β near 1) and
    /// steeper at the boundary (the necking zone).
    #[test]
    fn the_stretching_profile_keeps_the_steps_crust() {
        let n = 10_000;
        let mean = |beta: bool| {
            (0..n)
                .map(|i| shape((i as f64 + 0.5) / n as f64, beta))
                .sum::<f64>()
                / n as f64
        };
        assert!((mean(true) - 0.5).abs() < 1e-9 && (mean(false) - 0.5).abs() < 1e-9);
        assert_eq!((shape(0.0, true), shape(1.0, true)), (0.0, 1.0));
        assert_eq!((shape(-0.3, true), shape(1.4, true)), (0.0, 1.0));
        for s in [0.1, 0.25, 0.4] {
            assert!((shape(s, true) + shape(1.0 - s, true) - 1.0).abs() < 1e-12);
            assert!(shape(1.0 - s, true) > shape(1.0 - s, false), "proximal");
        }
        let slope = |beta: bool| (shape(0.51, beta) - shape(0.49, beta)) / 0.02;
        assert!((slope(true) - 1.5).abs() < 1e-3 && (slope(false) - 1.0).abs() < 1e-9);
    }

    /// With `transition_beta` the continental side's lithosphere and keel thin by the factor its
    /// crystalline crust thins (L / β, K / β): every continental source keeps L / L₀ = T / T₀;
    /// a sample with no state, the ocean side and the margin with the switch off keep theirs. The
    /// crust's volume is kept and the crust follows the new profile.
    #[test]
    fn a_stretched_margins_lithosphere_thins_with_its_crust() {
        let base = Fx::new(MODEL_KM);
        let fx = Fx::with(TectonicParams {
            transition_beta: true,
            thermal_state: true,
            ..base.k.clone()
        });
        let mut st = planet(&fx.geo, 0.0);
        for s in st.samples.iter_mut() {
            if s.kind == CONTINENTAL {
                s.lithosphere_km = 225.0;
                s.keel_km = 200.0;
            }
        }
        let before = st.samples.clone();
        let vol0 = crystalline_total(&st, 0);
        fx.run(&mut st, 30);
        let vol = crystalline_total(&st, 0);
        assert!((vol - vol0).abs() < 1e-6 * vol0, "volume {vol} vs {vol0}");
        let mut thinned = 0;
        for (i, s) in st.samples.iter().enumerate() {
            let b = &before[i];
            if s.kind != CONTINENTAL {
                assert_eq!(s.lithosphere_km, b.lithosphere_km, "ocean side {i}");
                continue;
            }
            let ratio = crystalline_m(&s.pools) / crystalline_m(&b.pools);
            let l = f64::from(s.lithosphere_km) / f64::from(b.lithosphere_km);
            assert!(
                (l - ratio).abs() < 1e-4,
                "sample {i}: L by {l}, crust by {ratio}"
            );
            assert!(s.keel_km <= s.lithosphere_km);
            if ratio < 0.95 {
                thinned += 1;
                let k = f64::from(s.keel_km) / f64::from(b.keel_km);
                assert!((k - ratio).abs() < 1e-4, "keel {i}: {k} vs {ratio}");
            }
        }
        assert!(thinned > 20, "{thinned} samples stretched");
        let (t_n, t_o) = (continental_m(), oceanic_m());
        let w = fx.k.transition_width_km;
        let thick = profile(&st, &fx.geo, 0, 20.0, 160.0, |s| crystalline_m(&s.pools));
        for &(d, t) in &thick {
            let want = t_o + (t_n - t_o) * shape(0.5 + d / w, true);
            assert!(
                (t - want).abs() < 2500.0,
                "{d:.0} km: {t:.0} m against the profile's {want:.0}"
            );
        }
        // Off: the lithosphere is not touched by the taper.
        let mut off = before.clone();
        let mut st = planet(&base.geo, 0.0);
        st.samples.clone_from(&off);
        base.run(&mut st, 30);
        for (i, s) in st.samples.iter().enumerate() {
            assert_eq!(s.lithosphere_km, before[i].lithosphere_km);
        }
        // No state (L 0): nothing to stretch.
        for s in off.iter_mut() {
            s.lithosphere_km = 0.0;
            s.keel_km = 0.0;
        }
        let mut st = planet(&fx.geo, 0.0);
        st.samples.clone_from(&off);
        fx.run(&mut st, 30);
        assert!(st.samples.iter().all(|s| s.lithosphere_km == 0.0));
    }
}

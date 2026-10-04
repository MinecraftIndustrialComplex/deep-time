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
//! from +715 m 20 km inland to −1,790 m 10 km out).
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
//! `collapse_quiet_myr`) get a signed distance d from that boundary on the planet (positive inland;
//! the boundary lies half a sample spacing beyond each boundary sample), and a target crystalline
//! thickness (the crust less its sediment and sedimentary rock)
//!
//!   T*(d) = T_o + (T_n − T_o) · clamp(1/2 + d / W, 0, 1),
//!
//! T_n the normal continental column (33.0 km of crystalline crust in [`continental_column`]),
//! T_o the ocean's (7 km, [`oceanic_column`]) and W = `transition_width_km` on the planet. The
//! taper is linear in thickness (β = T_n / T rises from 1 at the hinge, d = W/2, to T_n / T_o at
//! the toe, d = −W/2) and centred on the model's boundary: stretching a step from T_n to T_o into
//! this ramp keeps the crust's volume. Crust moves from the samples thicker than T* to the samples
//! thinner than T* within each stretch of margin (a catchment about W/2 long, below), the share
//! 1 − e^(−δt/τ) of what would bring the stretch to the profile (τ = `transition_myr`), as the
//! sources' own rock (pure shear). Crust types do not change: the thinned ocean-side crust stays
//! oceanic, so the continental crust share and land are not redefined by the rule.
//!
//! Heights follow the crust along the line through two reference columns: a continent of T_n at
//! z_0 (the middle of `continent_elevation_m`) and old ocean floor of T_o at z_f
//! (`oceanic_floor_elevation_m`). A metre of crust moved changes the height by 1/f,
//! f = (T_n − T_o)/(z_0 − z_f) (4.24 with the presets' 525 m and −5,600 m, between the model's dry
//! and water-loaded Airy factors, 5.5 and 3.78), on both sides, so the two sides of the boundary
//! meet. The ocean side then cools with its own age (the dampening, `pools::surface_step`), but
//! towards the floor its crust holds up, z_f + (T − T_o)/f, not the bare floor's: a young margin
//! stands near ridge depth and the taper appears as the ocean beside it subsides (McKenzie's
//! thermal subsidence, carried by the ocean's age–depth law).
//!
//! The plate model's height change is its uplift; the surface stage takes it as crust thickened or
//! thinned under an Airy root and loads its flexure with it (`surface/flexure.rs`), so the shelf,
//! slope and rise come from the taper, the sediment the surface brings and its slope failure.
//!
//! # Catchments
//! Crust moves within a stretch of margin, not across the plate: boundary samples are grouped
//! round seeds picked along the boundary at least W/2 apart (in index order, so a run is
//! reproducible), every sample of the zone belongs to the seed of its nearest boundary sample (on
//! its own side, through its own plate), and a stretch balances its sources against its sinks
//! (a narrow peninsula has more ocean than continent: its continent thins to the profile and the
//! ocean beside it gets what there is).
// ledger: PLT-260 (the rule), PLT-261 (the width), PLT-262 (τ), PLT-263 (the height line).

use std::collections::BTreeMap;

use glam::DVec3;

use super::Env;
use super::geometry::{Geo, dijkstra};
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
    // ledger: PLT-263.
    (continental_m() - oceanic_m()) / (z_0 - k.oceanic_floor_elevation_m).max(1.0)
}

/// The floor oceanic crust of `crystalline` metres cools towards, m: the bare floor z_f, raised by
/// the crust beyond the ocean's own (see the module docs).
pub fn held_floor_m(crystalline: f64, z_f: f64, f: f64) -> f64 {
    z_f + (crystalline - oceanic_m()).max(0.0) / f
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

/// The transition's width W as an angle on the model sphere: `transition_width_km` is on the
/// planet, so it is scaled by R_model / R_planet.
pub fn width_rad(env: &Env) -> f64 {
    let n = env.norm;
    // ledger: PLT-261.
    env.k.transition_width_km / n.planet_km.max(1e-9)
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
    let plate_of: Vec<u32> = (0..n).map(|i| state.plate_of(i)).collect();
    let samples = &state.samples;
    // Who takes part: visible, not at a trench, in a subduction zone or on an arc; continental
    // crust not raised by an orogeny lately (an active margin or a belt is not a rifted margin).
    let eligible: Vec<bool> = samples
        .iter()
        .map(|s| {
            s.flags & (HIDDEN | TRENCH | SUBDUCTION_ZONE | ARC) == 0
                && (s.kind == OCEANIC || s.orogeny_myr < quiet_before)
        })
        .collect();
    let same = |a: u32, b: u32| {
        eligible[a as usize] && eligible[b as usize] && plate_of[a as usize] == plate_of[b as usize]
    };
    // Boundary samples: eligible, with an eligible neighbour of the other crust on the same plate.
    let boundary: Vec<u32> = (0..n as u32)
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
    // sample's seed (of its own plate: crust never moves between plates).
    let cos_sep = (0.5 * w).min(std::f64::consts::PI).cos();
    let mut seeds: Vec<u32> = Vec::new();
    for &v in &boundary {
        let p = pos[v as usize];
        let pl = plate_of[v as usize];
        if seeds
            .iter()
            .all(|&s| plate_of[s as usize] != pl || pos[s as usize].dot(p) < cos_sep)
        {
            seeds.push(v);
        }
    }
    let seed_of = |v: u32| -> u32 {
        let p = pos[v as usize];
        let pl = plate_of[v as usize];
        let mut best = (f64::NEG_INFINITY, u32::MAX);
        for (i, &s) in seeds.iter().enumerate() {
            if plate_of[s as usize] != pl {
                continue;
            }
            let c = pos[s as usize].dot(p);
            if c > best.0 {
                best = (c, i as u32);
            }
        }
        best.1
    };
    // Distances from the boundary on each side, through each side's own crust of the plate.
    let reach = 0.5 * w + half;
    let side = |kind: u8| -> (Vec<f64>, Vec<u32>) {
        let src: Vec<(u32, f64, u32)> = boundary
            .iter()
            .filter(|&&v| samples[v as usize].kind == kind)
            .map(|&v| (v, 0.0, seed_of(v)))
            .collect();
        dijkstra(
            geo,
            &src,
            |a, b| same(a, b) && samples[b as usize].kind == kind,
            reach,
        )
    };
    let (dist_c, label_c) = side(CONTINENTAL);
    let (dist_o, label_o) = side(OCEANIC);

    // The zone: signed distance and target, per sample.
    struct Member {
        i: usize,
        seed: u32,
        excess: f64,
    }
    let mut members: Vec<Member> = Vec::new();
    for (i, s) in samples.iter().enumerate() {
        let (d, seed) = if s.kind == CONTINENTAL && dist_c[i].is_finite() {
            (dist_c[i] + half, label_c[i])
        } else if s.kind == OCEANIC && dist_o[i].is_finite() {
            (-(dist_o[i] + half), label_o[i])
        } else {
            continue;
        };
        if d.abs() >= 0.5 * w || seed == u32::MAX {
            continue;
        }
        // ledger: PLT-260.
        let target = t_o + (t_n - t_o) * (0.5 + d / w).clamp(0.0, 1.0);
        members.push(Member {
            i,
            seed,
            excess: crystalline_m(&s.pools) - target,
        });
    }
    out.zone = members.len() as u32;

    // Each stretch: sources against sinks.
    let mut stretch: BTreeMap<u32, (f64, f64)> = BTreeMap::new();
    for m in &members {
        let e = stretch.entry(m.seed).or_default();
        if m.excess > 0.0 {
            e.0 += m.excess;
        } else {
            e.1 -= m.excess;
        }
    }
    // The rock the sources give, per stretch, and the share each side moves this step.
    let mut mix: BTreeMap<u32, [f64; pool::COUNT]> = BTreeMap::new();
    let lo = k.trench_elevation_m;
    let hi = k.max_elevation_m;
    for m in &members {
        let (give, take) = stretch[&m.seed];
        let moved = rate * give.min(take);
        if moved <= 0.0 || m.excess <= 0.0 {
            continue;
        }
        let g = m.excess * moved / give;
        let s = &mut state.samples[m.i];
        let cryst = crystalline_m(&s.pools);
        let keep = ((cryst - g) / cryst).clamp(0.0, 1.0);
        let acc = mix.entry(m.seed).or_insert([0.0; pool::COUNT]);
        for &p in &CRYSTALLINE {
            let before = f64::from(s.pools[p]);
            s.pools[p] = (before * keep) as f32;
            acc[p] += before - f64::from(s.pools[p]);
        }
        s.thickness_m = s.pools.iter().sum();
        s.elevation_m = (f64::from(s.elevation_m) - g / f).clamp(lo, hi) as f32;
        out.moved_km2_m += g * env.norm.sample_area_km2();
    }
    for m in &members {
        if m.excess >= 0.0 {
            continue;
        }
        let Some(acc) = mix.get(&m.seed) else {
            continue;
        };
        let total: f64 = acc.iter().sum();
        if total <= 0.0 {
            continue;
        }
        let (give, take) = stretch[&m.seed];
        let moved = rate * give.min(take);
        let g = -m.excess * moved / take;
        let s = &mut state.samples[m.i];
        for (x, a) in s.pools.iter_mut().zip(acc.iter()) {
            *x += (g * a / total) as f32;
        }
        s.thickness_m = s.pools.iter().sum();
        s.elevation_m = (f64::from(s.elevation_m) + g / f).clamp(lo, hi) as f32;
    }
    // The floor the zone's ocean crust cools towards.
    let z_f = k.oceanic_floor_elevation_m;
    for m in &members {
        let s = &state.samples[m.i];
        if s.kind == OCEANIC {
            out.floor_m[m.i] = held_floor_m(crystalline_m(&s.pools), z_f, f) as f32;
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
            dynamics: None,
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
        fn new(planet_km: f64) -> Fx {
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
                norm: Norm::new(6370.0, planet_km, N),
            }
        }

        fn with(k: TectonicParams) -> Fx {
            Fx {
                k,
                ..Fx::new(PLANET_KM)
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

    /// Signed distance from the coast on the planet, km (positive inland).
    fn inland_km(p: DVec3) -> f64 {
        (COAST_Y.acos() - p.y.clamp(-1.0, 1.0).acos()) * PLANET_KM
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
        let fx = Fx::new(PLANET_KM);
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
        let old_step = z0
            .windows(2)
            .map(|p| p[1].1 - p[0].1)
            .fold(0.0, f64::max);
        assert!(old_step > 3000.0, "the old model steps at the coast: {z0:?}");
    }

    /// The ocean side's thicker crust gives it a held floor, and cooling takes it towards that
    /// floor, not the bare one; ocean floor beyond the taper cools as before.
    #[test]
    fn the_ocean_side_cools_towards_the_floor_its_crust_holds_up() {
        let fx = Fx::new(PLANET_KM);
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
            trench_relax: 0.0,
            gravity: 9.81,
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
            let want = held_floor_m(cryst, fx.k.oceanic_floor_elevation_m, f);
            assert!((f64::from(floor) - want).abs() < 1.0, "{floor} vs {want}");
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
            assert!(bare.elevation_m < kept.elevation_m - 500.0);
        }
        assert!(held > 20, "{held}");
        // Ocean floor with the ocean's own crust: the same as with no held floor.
        let mut a = Sample::blank(0);
        a.pools = oceanic_column();
        a.elevation_m = -3000.0;
        let mut b = a;
        let floor = held_floor_m(oceanic_m(), fx.k.oceanic_floor_elevation_m, f) as f32;
        surface_step(&mut a, &rates, floor);
        surface_step(&mut b, &rates, f32::NAN);
        assert!((a.elevation_m - b.elevation_m).abs() < 1e-3);
    }

    /// An active margin (in a subduction zone) keeps its step; with the width or the time at 0
    /// nothing moves; the width is on the planet, so a planet twice as large has a zone half as
    /// wide on the model sphere.
    #[test]
    fn active_margins_and_switched_off_rules_keep_their_step() {
        let fx = Fx::new(PLANET_KM);
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
        let big = Fx::new(2.0 * PLANET_KM);
        assert!((width_rad(&fx.env(1)) - 2.0 * width_rad(&big.env(1))).abs() < 1e-12);
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The initial planet (Cortial §3, "Workflow"): plates as noise-warped spherical Voronoi cells of
//! random centroids, continents shaped by the start configuration the player chose, plates at
//! rest (the force balance sets them moving from the first step, docs/design/plate-force-balance.md
//! task 7), oceanic crust aged from the boundaries that diverge under a drawn earlier motion, the
//! slabs that motion was sinking where it converged ([`earlier_slabs`]), and the first hotspots.
//!
//! Start configurations (plan §5.1, decisions file "Timeline"):
//! - **supercontinent**: one continent of the whole continental area, centred on the largest
//!   plate, with a noise-warped outline;
//! - **scattered_continents**: `scattered_continents` continents of mixed sizes at spread-out
//!   centres;
//! - **random**: the lowest values of a fractal noise, so continents and islands of every size.
//!
//! In every case the dry continental area is (1 − ocean_fraction) of the planet, to one sample, and
//! every continent starts whole on the plate most of it lies on. With `continental_shelf_share`
//! s > 0 (P1-17) the continental crust is (1 − ocean_fraction)/(1 − s) and its part nearest the
//! ocean starts as a flooded shelf, so `ocean_fraction` stays the share of the surface under water
//! once erosion and the sea have drowned the continents' margins as on Earth; with s = 0 (the
//! default) all continental crust starts dry.

use std::collections::BTreeMap;

use glam::{DQuat, DVec3};
use rayon::prelude::*;

use super::Env;
use super::geometry::{Geo, connect_plates, dijkstra};
use super::kinematics::{arr, random_motion};
use super::noise::fbm;
use super::pools::{continental_column, floor_at, ocean_heat_flow, oceanic_column};
use super::state::{
    CONTINENTAL, Counters, Group, Hotspot, OCEANIC, Plate, PlateState, Sample, SlabTrack, pool,
    regime,
};
use crate::error::CoreError;
use crate::params::{OceanDepth, PlanetPreset};
use crate::preset::StartConfig;
use crate::sphere::SplitMix64;

/// Depth of the initial continental shelf at its seaward edge, m (Earth's shelf break averages
/// about 130 m: Shepard 1963, "Submarine Geology").
const SHELF_BREAK_M: f64 = -130.0;

/// A uniformly random rotation (Shoemake 1992, "Uniform random rotations").
pub fn random_rotation(rng: &mut SplitMix64) -> DQuat {
    let (u1, u2, u3) = (rng.next_f64(), rng.next_f64(), rng.next_f64());
    let (a, b) = ((1.0 - u1).sqrt(), u1.sqrt());
    let (t2, t3) = (std::f64::consts::TAU * u2, std::f64::consts::TAU * u3);
    DQuat::from_xyzw(a * t2.sin(), a * t2.cos(), b * t3.sin(), b * t3.cos()).normalize()
}

/// The starting ocean floor's oldest age, Myr: Earth's oldest ocean floor in place is about 180 Myr
/// (from memory), the cap the earlier motion's ages always had. ledger: PLT-356
const START_OCEAN_MAX_AGE_MYR: f64 = 180.0;

/// The starting ocean's ages spread over the steady-state age–area distribution of an ocean made
/// at a constant rate and consumed evenly at every age, dA/dt = C₀(1 − t/t_m) (Parsons 1982), in
/// the order the earlier motion gave them (`raw`: young where it diverged): the share F of the
/// floor younger than t is 1 − (1 − t/t_m)², so the sample of rank F (the samples are equal-area)
/// takes t_m(1 − √(1 − F)), a mean of t_m/3. NaN (no ocean floor) stays NaN. ledger: PLT-356
fn steady_state_ages(raw: &[f64]) -> Vec<f64> {
    let mut order: Vec<u32> = (0..raw.len() as u32)
        .filter(|&i| !raw[i as usize].is_nan())
        .collect();
    order
        .par_sort_unstable_by(|&a, &b| raw[a as usize].total_cmp(&raw[b as usize]).then(a.cmp(&b)));
    let m = order.len() as f64;
    let mut out = raw.to_vec();
    for (r, &i) in order.iter().enumerate() {
        let f = (r as f64 + 0.5) / m;
        out[i as usize] = START_OCEAN_MAX_AGE_MYR * (1.0 - (1.0 - f).sqrt());
    }
    out
}

/// Indices of the `count` smallest scores (ties by index).
fn lowest(score: &[f64], count: usize) -> Vec<u32> {
    let mut idx: Vec<u32> = (0..score.len() as u32).collect();
    idx.par_sort_unstable_by(|&a, &b| {
        score[a as usize]
            .total_cmp(&score[b as usize])
            .then(a.cmp(&b))
    });
    idx.truncate(count);
    idx
}

/// Builds the initial plate state and its lattice geometry.
pub fn initial_state(preset: &PlanetPreset, env: &Env) -> Result<(PlateState, Geo), CoreError> {
    let k = env.k;
    let start = preset.start()?;
    let t0 = preset.t_start_years() / 1.0e6;
    let n = preset.resolution.l0_samples as usize;
    let mut rng = SplitMix64::new(preset.seed ^ 0xDEE9_7143_0000_0001);
    let lattice = arr(random_rotation(&mut rng));
    let geo = Geo::build(n, lattice)?;
    let pts: Vec<DVec3> = geo.points().to_vec();
    let norm = env.norm;
    let seed = rng.next_u64();

    // Plates: noise-warped, additively weighted Voronoi cells (a few large, some small).
    let np = k.initial_plates as usize;
    let min_sep = 0.35 * (4.0 * std::f64::consts::PI / np as f64).sqrt();
    let mut centres: Vec<DVec3> = Vec::with_capacity(np);
    while centres.len() < np {
        let c = rng.unit_vector();
        if centres.iter().all(|d| d.angle_between(c) > min_sep) || rng.next_f64() < 0.02 {
            centres.push(c);
        }
    }
    let weights: Vec<f64> = (0..np)
        .map(|_| k.plate_size_variety_rad * rng.next_f64().powi(2))
        .collect();
    let mut plate: Vec<u32> = pts
        .par_iter()
        .map(|&p| {
            let warp = 0.2 * fbm(p, 2.5, 3, seed);
            (0..np)
                .map(|c| (c, p.angle_between(centres[c]) - weights[c] + warp))
                .fold((0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a })
                .0 as u32
        })
        .collect();
    connect_plates(&geo, &mut plate, |_| true);

    // Continents: (1 − ocean_fraction) of the planet dry, plus the share of continental crust
    // under the sea (`continental_shelf_share`, 0 by default: the continental crust is then
    // exactly the dry share, P1-03's rule).
    let dry = ((1.0 - preset.planet.ocean_fraction) * n as f64).round() as usize;
    let target = ((1.0 - preset.planet.ocean_fraction) / (1.0 - k.continental_shelf_share)
        * n as f64)
        .round()
        .min(n as f64) as usize;
    let warp = k.coast_warp_rad;
    let mut continental = vec![false; n];
    match start {
        StartConfig::Supercontinent => {
            let mut counts = vec![0usize; np];
            for &p in &plate {
                counts[p as usize] += 1;
            }
            let big = (0..np)
                .max_by_key(|&c| (counts[c], std::cmp::Reverse(c)))
                .unwrap_or(0);
            let c = plate
                .iter()
                .zip(&pts)
                .filter(|(p, _)| **p as usize == big)
                .map(|(_, q)| *q)
                .sum::<DVec3>()
                .normalize_or(centres[big]);
            let s2 = seed.wrapping_add(11);
            let score: Vec<f64> = pts
                .par_iter()
                .map(|&p| p.angle_between(c) + warp * fbm(p, 3.0, 4, s2))
                .collect();
            for i in lowest(&score, target) {
                continental[i as usize] = true;
            }
        }
        StartConfig::ScatteredContinents => {
            let m = k.scattered_continents.max(1) as usize;
            let mut sizes: Vec<f64> = (0..m).map(|_| rng.range(0.35, 1.6).powi(2)).collect();
            let total: f64 = sizes.iter().sum();
            for s in &mut sizes {
                *s = (*s / total * target as f64).round();
            }
            sizes.sort_by(|a, b| b.total_cmp(a));
            let sep = 0.9 * (4.0 * std::f64::consts::PI / m as f64).sqrt();
            let mut cs: Vec<DVec3> = Vec::new();
            let mut tries = 0;
            while cs.len() < m {
                let c = rng.unit_vector();
                tries += 1;
                if cs.iter().all(|d| d.angle_between(c) > sep) || tries > 2000 {
                    cs.push(c);
                }
            }
            let mut taken = 0usize;
            for (j, c) in cs.iter().enumerate() {
                let want = if j + 1 == m {
                    target.saturating_sub(taken)
                } else {
                    sizes[j] as usize
                };
                let sj = seed.wrapping_add(100 + j as u64);
                let score: Vec<f64> = pts
                    .par_iter()
                    .enumerate()
                    .map(|(i, &p)| {
                        if continental[i] {
                            f64::INFINITY
                        } else {
                            p.angle_between(*c) + warp * fbm(p, 3.5, 4, sj)
                        }
                    })
                    .collect();
                for i in lowest(&score, want) {
                    if !continental[i as usize] {
                        continental[i as usize] = true;
                        taken += 1;
                    }
                }
            }
        }
        StartConfig::Random => {
            let s3 = seed.wrapping_add(7);
            let score: Vec<f64> = pts.par_iter().map(|&p| -fbm(p, 1.6, 6, s3)).collect();
            for i in lowest(&score, target) {
                continental[i as usize] = true;
            }
        }
    }

    // Each continent rides whole on the plate most of it lies on, so plate boundaries start in
    // the oceans (a continent straddling a boundary would be torn or forced under at once).
    let mut blob = vec![u32::MAX; n];
    let mut stack = Vec::new();
    for s in 0..n {
        if !continental[s] || blob[s] != u32::MAX {
            continue;
        }
        let mut members = Vec::new();
        blob[s] = s as u32;
        stack.push(s as u32);
        while let Some(v) = stack.pop() {
            members.push(v);
            for (u, _) in geo.edges(v) {
                if continental[u as usize] && blob[u as usize] == u32::MAX {
                    blob[u as usize] = s as u32;
                    stack.push(u);
                }
            }
        }
        let mut votes = vec![0usize; np];
        for &v in &members {
            votes[plate[v as usize] as usize] += 1;
        }
        let major = (0..np)
            .max_by_key(|&c| (votes[c], std::cmp::Reverse(c)))
            .unwrap_or(0) as u32;
        for &v in &members {
            plate[v as usize] = major;
        }
    }
    connect_plates(&geo, &mut plate, |_| true);

    // Plates start at rest. The starting ocean floor still needs ages: they come from an earlier
    // motion drawn for the purpose (a random pole, a speed of 0.5-1.5 times the speed scale), which
    // is not kept. ledger: PLT-273
    let mut plates: Vec<Plate> = Vec::with_capacity(np);
    let mut prior: Vec<DVec3> = Vec::with_capacity(np);
    for (c, &centre) in centres.iter().enumerate() {
        let members: Vec<usize> = (0..n).filter(|&i| plate[i] as usize == c).collect();
        let centroid = members
            .iter()
            .map(|&i| pts[i])
            .sum::<DVec3>()
            .normalize_or(centre);
        let cont = if members.is_empty() {
            0.0
        } else {
            members.iter().filter(|&&i| continental[i]).count() as f64 / members.len() as f64
        };
        let v = rng.range(0.5, 1.5) * k.plate_speed_scale_mm_yr; // ledger: PLT-273
        prior.push(random_motion(&mut rng, centroid, v, &norm));
        plates.push(Plate {
            id: c as u32,
            alive: !members.is_empty(),
            omega: [0.0; 3],
            born_myr: t0,
            died_myr: None,
            birth_event: None,
            parent: None,
            lip_until_myr: f64::NEG_INFINITY,
            lip_event: None,
            samples: members.len() as u32,
            centroid: centroid.to_array(),
            continental_frac: cont,
        });
    }

    // Oceanic ages: distance from the boundaries that diverge, at each plate's own speed, under the
    // earlier motion.
    let omega = prior;
    let mut sources = Vec::new();
    let mut any_div = vec![false; np];
    for v in 0..n as u32 {
        let pv = plate[v as usize];
        for (u, _) in geo.edges(v) {
            let pu = plate[u as usize];
            if pu != pv {
                let x = pts[v as usize];
                let rel = norm.velocity_mm_yr(omega[pu as usize] - omega[pv as usize], x);
                if rel.dot(pts[u as usize] - x) > 0.0 {
                    sources.push((v, 0.0, pv));
                    any_div[pv as usize] = true;
                    break;
                }
            }
        }
    }
    for v in 0..n as u32 {
        let pv = plate[v as usize];
        if !any_div[pv as usize] && geo.edges(v).any(|(u, _)| plate[u as usize] != pv) {
            sources.push((v, 0.0, pv));
        }
    }
    let (dist, _) = dijkstra(
        &geo,
        &sources,
        |a, b| plate[a as usize] == plate[b as usize],
        std::f64::consts::PI,
    );
    let raw_age: Vec<f64> = (0..n)
        .into_par_iter()
        .map(|i| {
            if continental[i] {
                return f64::NAN;
            }
            let pl = plate[i] as usize;
            let v = norm
                .speed_mm_yr(omega[pl], DVec3::from_array(plates[pl].centroid))
                .max(10.0);
            let d_km = if dist[i].is_finite() {
                norm.km(dist[i])
            } else {
                0.0
            };
            (d_km / v).min(START_OCEAN_MAX_AGE_MYR)
        })
        .collect();
    // Under the earlier motion alone the floor is young (median about 20 Myr against Earth's
    // 50 to 60): with the sea from the ocean's volume such a floor has too little room and the
    // planet starts with its sea 300 to 600 m high. Under GDH1 (the volume sea's floor, PLT-350)
    // the ages keep the motion's order and take the steady state's spread; the dampening's
    // plate-only presets keep their start. ledger: PLT-356
    let start_age = match k.ocean_depth {
        OceanDepth::Gdh1 => steady_state_ages(&raw_age),
        OceanDepth::Dampening => raw_age,
    };

    // The continental shelf: the continental crust nearest the ocean beyond the dry share starts
    // under the sea, from 0 m at the coast to `SHELF_BREAK_M` at the ocean's edge (continents are
    // born with their passive margins flooded, as Earth's are).
    let mut shelf_z: Vec<Option<f32>> = vec![None; n];
    let shelf = target.saturating_sub(dry);
    if shelf > 0 {
        let coast: Vec<(u32, f64, u32)> = (0..n as u32)
            .filter(|&v| {
                continental[v as usize] && geo.edges(v).any(|(u, _)| !continental[u as usize])
            })
            .map(|v| (v, 0.0, 0))
            .collect();
        let (inland, _) = dijkstra(
            &geo,
            &coast,
            |a, b| continental[a as usize] && continental[b as usize],
            std::f64::consts::PI,
        );
        let score: Vec<f64> = (0..n)
            .map(|i| {
                if continental[i] {
                    inland[i]
                } else {
                    f64::INFINITY
                }
            })
            .collect();
        let outer = lowest(&score, shelf);
        let widest = outer
            .iter()
            .map(|&i| inland[i as usize])
            .fold(0.0, f64::max)
            .max(1e-12);
        for &i in &outer {
            shelf_z[i as usize] =
                Some((SHELF_BREAK_M * (1.0 - inland[i as usize] / widest)) as f32);
        }
    }

    let s4 = seed.wrapping_add(23);
    let samples: Vec<Sample> = (0..n)
        .into_par_iter()
        .map(|i| {
            let p = pts[i];
            let mut s = Sample::blank(plate[i]);
            if continental[i] {
                let f = 0.5 + 0.5 * fbm(p, 4.0, 4, s4);
                s.kind = CONTINENTAL;
                s.elevation_m = shelf_z[i].unwrap_or(
                    (k.continent_elevation_m[0]
                        + f * (k.continent_elevation_m[1] - k.continent_elevation_m[0]))
                        as f32,
                );
                s.pools = continental_column();
                s.regime = regime::CRATON;
                s.heat_flow_mw_m2 = 60.0;
            } else {
                let age = start_age[i];
                s.kind = OCEANIC;
                s.formed_myr = (t0 - age) as f32;
                s.elevation_m = floor_at(k, age) as f32;
                s.pools = oceanic_column();
                s.pools[pool::SEDIMENT] = (k.pelagic_m_per_myr * age) as f32;
                s.heat_flow_mw_m2 = ocean_heat_flow(age) as f32;
            }
            s.thickness_m = s.pools.iter().sum();
            s
        })
        .collect();

    let hotspots: Vec<Hotspot> = (0..k.hotspots_initial)
        .map(|id| {
            let born = t0 - rng.range(0.0, 50.0);
            let life = rng.range(k.hotspot_lifetime_myr[0], k.hotspot_lifetime_myr[1]);
            Hotspot {
                id,
                pos: rng.unit_vector().to_array(),
                born_myr: born,
                dies_myr: (born + life).max(t0 + 10.0),
                uplift_mm_yr: k.hotspot_uplift_mm_yr * rng.range(0.6, 1.4),
                lip_event: None,
                plates: Vec::new(),
                track_km: 0.0,
                closed: false,
            }
        })
        .collect();

    let groups: Vec<Group> = (0..np as u32)
        .map(|p| Group {
            plate: p,
            rot: [0.0, 0.0, 0.0, 1.0],
        })
        .collect();
    let model_area = 4.0 * std::f64::consts::PI * norm.model_km * norm.model_km;
    let state = PlateState {
        n: n as u32,
        lattice,
        samples,
        groups,
        plates,
        contacts: Vec::new(),
        hotspots,
        ridges: Vec::new(),
        rift_links: Vec::new(),
        rng,
        step: 0,
        t_myr: t0,
        start_myr: t0,
        resample_step: 0,
        resample_myr: t0,
        a0_km2: model_area / np as f64,
        counters: Counters {
            hotspots: k.hotspots_initial,
            ..Counters::default()
        },
        slabs: Vec::new(),
        dynamics: None,
        gap_budget: None,
        limits: None,
        shown: None,
        one_way: None,
        propagations: Vec::new(),
        level_set: None,
    };
    let mut state = state;
    state.slabs = earlier_slabs(&geo, &plate, &state.samples, &omega, k, t0, &norm);
    // The start's continents carry an old craton's lithosphere and keel (ledger: PLT-463).
    super::thermal::start(&mut state, k);
    Ok((state, geo))
}

/// The slabs of the earlier motion that aged the starting sea floor (ledger: PLT-274). Plates start
/// at rest, but the planet is a moment of a longer history: wherever the earlier motion converged
/// across a plate boundary with ocean floor on its sinking side, that floor was going down, and
/// its slab hangs from the boundary at full length (`slab_depth_km`). The sinking side follows the
/// rule of a new contact (`subduction::resolve`): ocean floor under continent, the older floor
/// where both sides are oceanic, and no slab where both are continental. A pair needs
/// `MIN_CONTACT` converging boundary samples. The force balance then sets the plates moving within
/// the first relaxation time; a slab whose trench never forms is gone after `slab_breakoff_myr`
/// (`forces.rs`, hanging slabs).
fn earlier_slabs(
    geo: &Geo,
    plate: &[u32],
    samples: &[Sample],
    omega: &[DVec3],
    k: &crate::params::TectonicParams,
    t0: f64,
    norm: &super::normalise::Norm,
) -> Vec<SlabTrack> {
    #[derive(Default)]
    struct Side {
        seen: u32,
        cont: u32,
        age: f64,
    }
    let pts = geo.points();
    let mut pairs: BTreeMap<(u32, u32), [Side; 2]> = BTreeMap::new();
    for v in 0..plate.len() as u32 {
        let pv = plate[v as usize];
        let x = pts[v as usize];
        // A sample counts once for its plate, if any neighbour of another plate converges on it.
        let mut hit: Option<u32> = None;
        for (u, _) in geo.edges(v) {
            let pu = plate[u as usize];
            if pu == pv {
                continue;
            }
            let rel = norm.velocity_mm_yr(omega[pu as usize] - omega[pv as usize], x);
            if rel.dot(pts[u as usize] - x) < 0.0 {
                hit = Some(pu);
                break;
            }
        }
        let Some(pu) = hit else { continue };
        let key = (pv.min(pu), pv.max(pu));
        let side = &mut pairs.entry(key).or_default()[usize::from(pv != key.0)];
        let s = &samples[v as usize];
        side.seen += 1;
        if s.kind == CONTINENTAL {
            side.cont += 1;
        } else {
            side.age += s.age_myr(t0);
        }
    }
    let mut out = Vec::new();
    for ((lo, hi), sides) in pairs {
        if sides[0].seen + sides[1].seen < super::subduction::MIN_CONTACT {
            continue;
        }
        let frac = |s: &Side| f64::from(s.cont) / f64::from(s.seen.max(1));
        let mean_age = |s: &Side| {
            let oceanic = s.seen - s.cont;
            if oceanic == 0 {
                0.0
            } else {
                s.age / f64::from(oceanic)
            }
        };
        let (c0, c1) = (frac(&sides[0]) >= 0.5, frac(&sides[1]) >= 0.5);
        let under_lo = match (c0, c1) {
            (false, true) => true,
            (true, false) => false,
            (false, false) => mean_age(&sides[0]) >= mean_age(&sides[1]),
            (true, true) => continue,
        };
        let (under, over) = if under_lo { (lo, hi) } else { (hi, lo) };
        out.push(SlabTrack {
            under,
            over,
            km: k.slab_depth_km, // ledger: PLT-274
            ..SlabTrack::default()
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_starting_ocean_takes_the_steady_state_ages() {
        // Ten thousand ocean samples, aged 0 to 40 Myr in a shuffled order, and a continent.
        let mut raw: Vec<f64> = (0..10_000u64)
            .map(|i| (i.wrapping_mul(7919) % 10_000) as f64 * 0.004)
            .collect();
        raw.push(f64::NAN);
        let a = steady_state_ages(&raw);
        assert!(a[10_000].is_nan());
        // The order is kept: an older raw age is never younger after.
        let mut by_raw: Vec<usize> = (0..10_000).collect();
        by_raw.sort_by(|&x, &y| raw[x].total_cmp(&raw[y]));
        assert!(by_raw.windows(2).all(|w| a[w[0]] <= a[w[1]]));
        // Parsons's triangle with t_m = 180 Myr: mean t_m/3 = 60 Myr, median t_m(1 − √½) = 52.7,
        // all within 0 to t_m.
        let mean = a[..10_000].iter().sum::<f64>() / 1e4;
        assert!((mean - 60.0).abs() < 0.05, "{mean}");
        let mut s = a[..10_000].to_vec();
        s.sort_by(f64::total_cmp);
        assert!(
            (s[5_000] - 180.0 * (1.0 - 0.5f64.sqrt())).abs() < 0.05,
            "{}",
            s[5_000]
        );
        assert!(s[0] >= 0.0 && s[9_999] < 180.0);
    }
}

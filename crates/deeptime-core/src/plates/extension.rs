// SPDX-License-Identifier: LGPL-3.0-or-later
//! A rift stretches the continental lithosphere on both its sides before it breaks up
//! (`tectonics.rift_stretching`, off by default; ledger PLT-781 to PLT-785). The design, the
//! sources and the arm are docs/design/continental-extension.md.
//!
//! # Why
//! The uplift trace (docs/design/uplift-trace.md) found that rifting thins nothing: a rift
//! (`rifting.rs`) only relabels a plate's samples, and when its two sides move apart the gap
//! between them becomes ocean floor at the next resampling (`ridges.rs`). Earth's continents
//! stretch first. Uniform stretching by a factor β thins the crust to 1/β of what it was, and the
//! area grows by β (McKenzie 1978); a rift through hot, thick crust spreads over a wide region
//! (the Basin and Range, the Aegean), one through cold lithosphere stays narrow, and the crust
//! breaks up only once it is hyper-extended, under 10 km (Brune et al. 2014). Thickened crust
//! near an opening boundary then leaves the land by stretching, not only by erosion: the
//! free-boundary collapse of Rey, Vanderhaeghe & Teyssier 2001, which the model's collapse
//! (`collapse.rs`, fixed boundaries, towards the lowland's column) does not have.
//!
//! # The rule
//! Each plate step, a boundary between two plates where both sides are visible continental
//! crust (no trench) and the relative motion opens it (along the boundary's normal,
//! `shortening::boundary_normal`, positive, and no transform, PLT-311) is a rift. Per pair of
//! plates, its length W (the boundary's straight length, `shortening::front_share_rad`, the mean
//! of its two sides) and its mean opening speed u. The opening's area grows at u W, and the two
//! sides' continental lithosphere takes it in pure shear, the column's area growing as its crust
//! thins:
//!
//!   Σ_i ε̇_i A = u W,   ε̇_i = u W w_i / (A Σ_j w_j),   w_i = e^(−x_i / L) · η_0 / η_i,
//!
//! over the visible continental columns of the two plates within reach of the rift along their
//! own plate's continental crust, x_i a column's distance from the rift, A a sample's area, L the
//! rift's half-width ([`half_width_km`], the rift's mode) and η_0 / η_i the column's weakness at
//! its lithosphere's thermal age (`thermal::sheet_weakness`, PLT-415's law with ISO-112's age; 1
//! everywhere with the thermal state off). The decay e^(−x / L) is the thin sheet's for a
//! boundary of length W (England, Houseman & Sonder 1985, as the collision's push, PLT-733); the
//! weakness puts the strain where the sheet is weakest, a game-choice. A column keeps
//! e^(−ε̇ δt) of its crust in its own mix, never less than 10 km of crystalline crust (Brune et
//! al. 2014's hyper-extended limit, PLT-661), its lithosphere and keel thinned by the same factor
//! (`thermal::stretch`, McKenzie's uniform stretching, PLT-476), its surface down by the Airy
//! change of what it lost (but not below the Airy height of what it keeps, `collapse::toward`).
//! A column at the limit takes no share: what the rift's speed asks beyond the columns that can
//! still thin is break-up, new sea floor.
//!
//! The crust a column gives moves into the opening, which is no sample yet: it waits on the pair
//! ([`PlateState::rift_fill`]) until the next resampling lays it in the gap between the two
//! plates ([`lay`]).
//!
//! # The resampling
//! The gap samples between the pair (their two nearest plates) take the fill evenly, a column of
//! its mix: the rift's floor of stretched continental crust, its lithosphere the thinned one it
//! came from, at its Airy height under the sea. If that column would hold less than 10 km of
//! crystalline crust the rift has broken up: the gap samples nearest the plates' edges take
//! 10 km columns of it, nearest first, until it is used, and the rest stays new sea floor (the
//! hyper-extended distal margins). A fill whose pair has no gap at a resampling (the rift opened
//! less than a sample, or healed) waits for the next one while both plates live; when one has
//! died it goes to the continental samples nearest where it came from (counted apart).
//!
//! # What it is not
//! A strain solve: the decay length and the weakness distribute the opening's area; the area
//! itself is the plates' motion. Nor the collapse: a rift needs an opening boundary, and a
//! collapse needs none.

use std::collections::BTreeMap;

use glam::{DQuat, DVec3};
use rayon::prelude::*;

use super::Env;
use super::collapse::{iso_height, toward};
use super::geometry::{Geo, dijkstra_sparse};
use super::kinematics::quat;
use super::pools::{isostatic_change_m, thickness};
use super::settle;
use super::shortening::{boundary_normal, front_share_rad, is_transform};
use super::state::{CONTINENTAL, PlateState, RiftFill, Sample, TRENCH, pool, regime};
use super::thermal;
use super::transition::{HYPER_EXTENDED_M, crystalline_m};

/// `WorldState::globals`: crust a rift stretched into its opening over the run, km³ …
pub const STRETCHED_GLOBAL: &str = "plates/rift_stretched_km3";
/// … laid as rifts' floors at resamplings …
pub const FLOOR_GLOBAL: &str = "plates/rift_floor_km3";
/// … and laid by the fallback, a pair with a dead plate (`lay`).
pub const FALLBACK_GLOBAL: &str = "plates/rift_fallback_km3";
/// Rift-steps (a pair of plates with a rift in a step) over the run.
pub const RIFT_STEPS_GLOBAL: &str = "plates/rift_steps";

/// How far a rift reaches, in half-widths: e^(−4) = 0.018 of the strain at the rift.
const REACH_HALF_WIDTHS: f64 = 4.0;

/// What one step did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Extension {
    /// Pairs of plates with a rift.
    pub rifts: u32,
    /// Columns that thinned.
    pub columns: u32,
    /// Crust moved into the rifts' fills, m summed over samples.
    pub stretched_m: f64,
}

/// What [`lay`] did at a resampling, m summed over samples.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Laid {
    pub floor_m: f64,
    pub fallback_m: f64,
    /// Gap samples made the rift's continental floor.
    pub samples: u32,
}

/// A rift's half-width L, km: the thin sheet's decay length for a boundary of length
/// `length_km`, 2 W / π (`belts::sheet_reach_km`, PLT-733's form with n = 1).
// ledger: PLT-782
pub fn half_width_km(length_km: f64) -> f64 {
    super::belts::sheet_reach_km(length_km)
}

/// One side of a rift: a sample of plate `plate` on its boundary with `other`, the opening speed
/// across the boundary (mm/yr) and the sample's share of the boundary's length (km).
#[derive(Clone, Copy, Debug)]
struct Edge {
    v: u32,
    plate: u32,
    other: u32,
    speed_mm_yr: f64,
    length_km: f64,
}

/// The rifts' edges this step (see the module docs).
fn edges(state: &PlateState, geo: &Geo, pos: &[DVec3], env: &Env, plate_of: &[u32]) -> Vec<Edge> {
    let samples = &state.samples;
    let rots: Vec<DQuat> = state.groups.iter().map(|g| quat(g.rot)).collect();
    let omega: Vec<DVec3> = state
        .plates
        .iter()
        .map(|p| DVec3::from_array(p.omega))
        .collect();
    let norm = env.norm;
    let continental = |s: &Sample| s.kind == CONTINENTAL && !s.is_hidden() && s.flags & TRENCH == 0;
    (0..samples.len())
        .into_par_iter()
        .flat_map_iter(|v| {
            let sv = &samples[v];
            let mut out: Vec<Edge> = Vec::new();
            if !continental(sv) {
                return out;
            }
            let pv = plate_of[v];
            let mut seen: Vec<u32> = Vec::new();
            for (u, _) in geo.edges(v as u32) {
                let pu = plate_of[u as usize];
                if pu == pv || seen.contains(&pu) || !continental(&samples[u as usize]) {
                    continue;
                }
                seen.push(pu);
                let x = pos[v];
                let rel = norm.velocity_mm_yr(omega[pu as usize] - omega[pv as usize], x);
                let n = boundary_normal(geo, pos, &rots, samples, plate_of, v, pu);
                // ledger: PLT-311 (no transform)
                if n == DVec3::ZERO || is_transform(rel, n) {
                    continue;
                }
                let speed = rel.dot(n);
                if speed <= 0.0 {
                    continue;
                }
                let share = front_share_rad(geo, pos, &rots, samples, plate_of, v, pu, n);
                out.push(Edge {
                    v: v as u32,
                    plate: pv,
                    other: pu,
                    speed_mm_yr: speed,
                    length_km: norm.km(share),
                });
            }
            out
        })
        .collect()
}

/// The pair key of two plates, the smaller first.
fn pair(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}

/// How much of a column's crust it may keep at least: its crystalline crust down to the
/// hyper-extended limit, as a share of the column (1 at or under the limit).
// ledger: PLT-661 (Brune et al. 2014's 10 km), PLT-783
pub fn least_keep(pools: &[f32; pool::COUNT]) -> f64 {
    let c = crystalline_m(pools);
    if c <= HYPER_EXTENDED_M {
        1.0
    } else {
        HYPER_EXTENDED_M / c
    }
}

/// The share of its crust a column keeps at strain rate `rate_per_myr` over `dt_myr`, no less
/// than [`least_keep`]: pure shear, e^(−ε̇ δt).
// ledger: PLT-781 (McKenzie 1978: crust thinned to 1/β), PLT-783
pub fn keep_share(rate_per_myr: f64, dt_myr: f64, pools: &[f32; pool::COUNT]) -> f64 {
    if !(rate_per_myr > 0.0 && dt_myr > 0.0) {
        return 1.0;
    }
    (-rate_per_myr * dt_myr).exp().max(least_keep(pools))
}

/// The strain rates of a rift's columns, per Myr, from their weights: the opening's area rate
/// `u W` (mm/yr × km = km²/Myr) over the sample area `area_km2` shared in proportion to the
/// weights (see the module docs). Σ rate × area = u W when any weight is positive.
// ledger: PLT-781 (the areas), PLT-782 (the weights)
pub fn strain_rates(speed_mm_yr: f64, length_km: f64, area_km2: f64, weights: &[f64]) -> Vec<f64> {
    let sum: f64 = weights.iter().filter(|w| **w > 0.0).sum();
    if !(sum > 0.0 && area_km2 > 0.0 && speed_mm_yr > 0.0 && length_km > 0.0) {
        return vec![0.0; weights.len()];
    }
    let scale = speed_mm_yr * length_km / (area_km2 * sum);
    weights.iter().map(|&w| w.max(0.0) * scale).collect()
}

/// One plate step of the rifts' stretching (see the module docs).
pub fn step(state: &mut PlateState, geo: &Geo, pos: &[DVec3], env: &Env) -> Extension {
    let k = env.k;
    if !k.rift_stretching || env.dt_myr <= 0.0 {
        return Extension::default();
    }
    let plate_of: Vec<u32> = (0..state.samples.len())
        .map(|i| state.plate_of(i))
        .collect();
    let found = edges(state, geo, pos, env, &plate_of);
    if found.is_empty() {
        return Extension::default();
    }
    // Per pair: the edges on each side.
    let mut rifts: BTreeMap<(u32, u32), Vec<Edge>> = BTreeMap::new();
    for e in found {
        rifts.entry(pair(e.plate, e.other)).or_default().push(e);
    }
    let area = env.norm.sample_area_km2();
    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
    let g = env.gravity;
    let tau = k.collapse_sheet_strength_myr;
    let thermal_on = k.thermal_state;
    let mut out = Extension::default();
    for (&(a, b), es) in &rifts {
        let len = |p: u32| -> f64 {
            es.iter()
                .filter(|e| e.plate == p)
                .map(|e| e.length_km)
                .sum()
        };
        let length_km = 0.5 * (len(a) + len(b));
        let speed = es.iter().map(|e| e.speed_mm_yr).sum::<f64>() / es.len() as f64;
        if !(length_km > 0.0 && speed > 0.0) {
            continue;
        }
        let l_km = half_width_km(length_km);
        let reach = env.norm.rad(REACH_HALF_WIDTHS * l_km);
        // The columns within reach on each side, along their own plate's continental crust.
        let mut cols: Vec<(u32, f64)> = Vec::new();
        for p in [a, b] {
            let src: Vec<(u32, f64, u32)> = es
                .iter()
                .filter(|e| e.plate == p)
                .map(|e| (e.v, 0.0, e.v))
                .collect();
            let samples = &state.samples;
            let on = |u: u32| {
                let s = &samples[u as usize];
                plate_of[u as usize] == p
                    && s.kind == CONTINENTAL
                    && !s.is_hidden()
                    && s.flags & TRENCH == 0
            };
            for (u, d, _) in dijkstra_sparse(geo, &src, |_, u| on(u), reach) {
                cols.push((u, env.norm.km(d)));
            }
        }
        cols.sort_by_key(|c| c.0);
        cols.dedup_by_key(|c| c.0);
        let weights: Vec<f64> = cols
            .iter()
            .map(|&(u, x)| {
                let s = &state.samples[u as usize];
                if least_keep(&s.pools) >= 1.0 {
                    return 0.0;
                }
                // ledger: PLT-415, ISO-112 (the weakness), PLT-782 (its use here)
                let weak = if thermal_on {
                    thermal::sheet_weakness(tau, s.lithosphere_km)
                } else {
                    1.0
                };
                (-x / l_km).exp() * weak
            })
            .collect();
        let rates = strain_rates(speed, length_km, area, &weights);
        let mut fill = RiftFill {
            a,
            b,
            ..RiftFill::default()
        };
        for (&(u, _), &rate) in cols.iter().zip(&rates) {
            let s = &mut state.samples[u as usize];
            let keep = keep_share(rate, env.dt_myr, &s.pools);
            if keep >= 1.0 {
                continue;
            }
            let before = s.pools;
            let mut given = 0.0;
            for (p, x) in s.pools.iter_mut().enumerate() {
                let r = f64::from(*x) * (1.0 - keep);
                *x = (f64::from(*x) - r) as f32;
                fill.pools[p] += r;
                given += r;
            }
            s.thickness_m = thickness(&s.pools);
            s.basement_m = (f64::from(s.basement_m) * keep) as f32;
            // ledger: ISO-091 (its height follows its crust), PLT-254 (not below what it keeps)
            let change = isostatic_change_m(&before, &s.pools, g);
            let iso = iso_height(&s.pools, z_0, g);
            s.elevation_m = toward(f64::from(s.elevation_m), change, iso) as f32;
            fill.lithosphere_km_m += given * f64::from(s.lithosphere_km.max(0.0)) * keep;
            thermal::stretch(s, keep);
            let p = pos[u as usize].normalize_or_zero();
            for (c, x) in fill.centroid.iter_mut().zip(p.to_array()) {
                *c += given * x;
            }
            if let Some(d) = state.top_deficit_m.get_mut(u as usize) {
                *d = (f64::from(*d) * keep) as f32;
            }
            out.columns += 1;
            out.stretched_m += given;
        }
        if fill.total_m() > 0.0 {
            out.rifts += 1;
            merge(&mut state.rift_fill, fill);
        }
    }
    out
}

/// Adds `fill` to its pair's fill on `fills` (or a new one).
fn merge(fills: &mut Vec<RiftFill>, fill: RiftFill) {
    if let Some(f) = fills.iter_mut().find(|f| f.a == fill.a && f.b == fill.b) {
        for (x, y) in f.pools.iter_mut().zip(fill.pools) {
            *x += y;
        }
        f.lithosphere_km_m += fill.lithosphere_km_m;
        for (x, y) in f.centroid.iter_mut().zip(fill.centroid) {
            *x += y;
        }
    } else {
        fills.push(fill);
    }
}

/// A gap sample of a resampling between two plates: its index, the pair, and its distance from
/// its own plate's edge (radians).
#[derive(Clone, Copy, Debug)]
pub struct GapPoint {
    pub i: u32,
    pub pair: (u32, u32),
    pub edge_rad: f64,
}

/// Lays the rifts' fills in a resampling's gaps (see the module docs, "The resampling"):
/// `samples` the new lattice's, `gaps` its gap samples between two plates, `points` the new
/// lattice's unit positions, `alive` whether a plate lives on, `sea_m` the sea over the datum.
/// Returns what it laid and the indices of the samples it made continental.
#[allow(clippy::too_many_arguments)]
pub fn lay(
    fills: &mut Vec<RiftFill>,
    samples: &mut [Sample],
    gaps: &[GapPoint],
    points: &[DVec3],
    alive: impl Fn(u32) -> bool,
    k: &crate::params::TectonicParams,
    gravity: f64,
    sea_m: f64,
) -> Laid {
    let mut laid = Laid::default();
    if fills.is_empty() {
        return laid;
    }
    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
    let mut by_pair: BTreeMap<(u32, u32), Vec<GapPoint>> = BTreeMap::new();
    for g in gaps {
        by_pair.entry(g.pair).or_default().push(*g);
    }
    let mut kept = Vec::new();
    for fill in std::mem::take(fills) {
        let total = fill.total_m();
        if !(total > 0.0) {
            continue;
        }
        let lith = fill.lithosphere_km_m / total;
        let members = by_pair.get(&(fill.a, fill.b)).cloned().unwrap_or_default();
        if members.is_empty() {
            if alive(fill.a) && alive(fill.b) {
                let mut f = fill;
                f.waited += 1;
                kept.push(f);
            } else {
                laid.fallback_m += fallback(&fill, samples, points, k, gravity, z_0, sea_m);
            }
            continue;
        }
        let mut members = members;
        members.sort_by(|x, y| x.edge_rad.total_cmp(&y.edge_rad).then(x.i.cmp(&y.i)));
        // The fill's column if spread evenly; break-up when it would be hyper-extended
        // (ledger: PLT-661, PLT-784).
        let mix: [f32; pool::COUNT] = std::array::from_fn(|p| (fill.pools[p] / total) as f32);
        let crystalline_share = crystalline_m(&mix);
        let crystalline_total = crystalline_share * total;
        let n = members.len();
        let m = if crystalline_total >= HYPER_EXTENDED_M * n as f64 || crystalline_share <= 0.0 {
            n
        } else {
            ((crystalline_total / HYPER_EXTENDED_M).floor() as usize).clamp(1, n)
        };
        let column = total / m as f64;
        for g in &members[..m] {
            let s = &mut samples[g.i as usize];
            *s = floor_sample(s.group, &mix, column, lith, k, gravity, z_0, sea_m);
            laid.samples += 1;
        }
        laid.floor_m += total;
    }
    *fills = kept;
    laid
}

/// The rift's floor at a gap sample: continental crust of `mix` (shares of 1), `column_m` thick,
/// its lithosphere `lith_km`, at its Airy height under a sea at `sea_m`.
#[allow(clippy::too_many_arguments)]
fn floor_sample(
    group: u32,
    mix: &[f32; pool::COUNT],
    column_m: f64,
    lith_km: f64,
    k: &crate::params::TectonicParams,
    gravity: f64,
    z_0: f64,
    sea_m: f64,
) -> Sample {
    let mut s = Sample::blank(group);
    s.kind = CONTINENTAL;
    s.regime = regime::RIFT;
    s.pools = std::array::from_fn(|p| (f64::from(mix[p]) * column_m) as f32);
    s.thickness_m = thickness(&s.pools);
    let z_a = iso_height(&s.pools, z_0, gravity) + thermal::uplift_m(k, lith_km as f32);
    s.elevation_m = settle::equilibrium_m(z_a, sea_m) as f32;
    if k.thermal_state && lith_km > 0.0 {
        s.lithosphere_km = lith_km as f32;
    }
    s.heat_flow_mw_m2 = thermal::sample_heat_flow(s.lithosphere_km, 0);
    s
}

/// A fill whose pair cannot take it (a plate died): onto the visible continental samples nearest
/// where it came from, each up to 10 km more, nearest first. Returns what it laid, m summed over
/// samples.
fn fallback(
    fill: &RiftFill,
    samples: &mut [Sample],
    points: &[DVec3],
    k: &crate::params::TectonicParams,
    gravity: f64,
    z_0: f64,
    sea_m: f64,
) -> f64 {
    let total = fill.total_m();
    let at = DVec3::from_array(fill.centroid).normalize_or_zero();
    let mut near: Vec<(f64, usize)> = samples
        .iter()
        .enumerate()
        .filter(|(_, s)| s.kind == CONTINENTAL && !s.is_hidden())
        .map(|(i, _)| (-points[i].dot(at), i))
        .collect();
    if near.is_empty() || at == DVec3::ZERO {
        return 0.0;
    }
    near.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));
    let m = ((total / HYPER_EXTENDED_M).ceil() as usize).clamp(1, near.len());
    let each = total / m as f64;
    for &(_, i) in &near[..m] {
        let s = &mut samples[i];
        for (p, x) in s.pools.iter_mut().enumerate() {
            *x = (f64::from(*x) + fill.pools[p] / total * each) as f32;
        }
        s.thickness_m = thickness(&s.pools);
        let z_a = iso_height(&s.pools, z_0, gravity) + thermal::uplift_m(k, s.lithosphere_km);
        s.elevation_m = settle::equilibrium_m(z_a, sea_m) as f32;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::TectonicParams;
    use crate::plates::geometry::positions;
    use crate::plates::normalise::Norm;
    use crate::plates::pools::{continental_column, oceanic_column};
    use crate::plates::state::{Group, OCEANIC, Plate};

    const N: usize = 16_000;
    /// Earth's sphere: a sample is about 178 km across at this N.
    const MODEL_KM: f64 = 6370.0;
    /// cos 40°: the continent is the cap within 40° of +y.
    const CAP_Y: f64 = 0.766;

    fn plate(id: u32, omega: [f64; 3]) -> Plate {
        Plate {
            id,
            alive: true,
            omega,
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

    /// Plate 0 is x < 0, plate 1 x ≥ 0; a continent (35.1 km) within 40° of +y across both, old
    /// ocean floor elsewhere. Plate 0 turns about +z and plate 1 about −z at `omega` rad/yr each,
    /// so at +y they move apart along x (`omega` > 0) or together.
    fn planet(geo: &Geo, omega: f64) -> PlateState {
        let samples: Vec<Sample> = geo
            .points()
            .iter()
            .map(|&p| {
                let mut s = Sample::blank(u32::from(p.x >= 0.0));
                if p.y > CAP_Y {
                    s.kind = CONTINENTAL;
                    s.pools = continental_column();
                    s.elevation_m = 300.0;
                    s.lithosphere_km = 150.0;
                } else {
                    s.pools = oceanic_column();
                    s.elevation_m = -5000.0;
                    s.formed_myr = -200.0;
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
            plates: vec![plate(0, [0.0, 0.0, omega]), plate(1, [0.0, 0.0, -omega])],
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
            top_deficit_m: Vec::new(),
            rift_fill: Vec::new(),
            shown: None,
            one_way: None,
            propagations: Vec::new(),
            backarcs: Vec::new(),
            level_set: None,
        };
        let pos = positions(&st, geo);
        st.recount(&pos);
        st
    }

    fn env(k: &TectonicParams) -> Env<'_> {
        Env {
            k,
            norm: Norm::new(MODEL_KM, MODEL_KM, N),
            dt_myr: 2.0,
            dt_yr: 2.0e6,
            gravity: 9.81,
            step: 1,
            t_myr: 2.0,
        }
    }

    /// 10 mm/yr at 90° from the pole on the model sphere, rad/yr.
    fn omega_10mm() -> f64 {
        10.0e-6 / MODEL_KM
    }

    fn crust(st: &PlateState) -> f64 {
        st.samples.iter().map(|s| f64::from(s.thickness_m)).sum()
    }

    fn rifting() -> TectonicParams {
        TectonicParams {
            rift_stretching: true,
            ..TectonicParams::default()
        }
    }

    /// The columns share the opening's area rate in proportion to their weights.
    #[test]
    fn the_columns_take_the_openings_area_in_proportion_to_their_weights() {
        let w = [1.0, 0.5, 0.0, 0.25, -1.0];
        let r = strain_rates(20.0, 500.0, 1000.0, &w);
        let area_rate: f64 = r.iter().map(|x| x * 1000.0).sum();
        assert!((area_rate - 20.0 * 500.0).abs() < 1e-9, "{area_rate}");
        assert!((r[0] / r[1] - 2.0).abs() < 1e-12);
        assert_eq!(r[2], 0.0);
        assert_eq!(r[4], 0.0);
        assert_eq!(
            strain_rates(20.0, 500.0, 1000.0, &[0.0, 0.0]),
            vec![0.0, 0.0]
        );
        assert_eq!(strain_rates(0.0, 500.0, 1000.0, &[1.0]), vec![0.0]);
    }

    /// Pure shear keeps e^(−ε̇ δt) of the crust, never less than 10 km of crystalline crust.
    #[test]
    fn a_column_thins_by_pure_shear_down_to_the_hyper_extended_limit() {
        let c = continental_column();
        let keep = keep_share(0.05, 2.0, &c);
        assert!((keep - (-0.1f64).exp()).abs() < 1e-12);
        let floor = keep_share(10.0, 2.0, &c);
        assert!(
            (floor - HYPER_EXTENDED_M / crystalline_m(&c)).abs() < 1e-12,
            "{floor}"
        );
        let mut thin = c;
        for x in thin.iter_mut() {
            *x *= 0.25;
        }
        assert_eq!(least_keep(&thin), 1.0);
        assert_eq!(keep_share(10.0, 2.0, &thin), 1.0);
        assert_eq!(keep_share(0.0, 2.0, &c), 1.0);
    }

    /// Two continental plates moving apart: the columns by the rift thin, the far ones less, the
    /// crust they give waits on the pair, and none is lost. Moving together, or with the switch
    /// off, nothing happens.
    #[test]
    fn a_rift_stretches_both_sides_and_keeps_their_crust_for_its_floor() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = rifting();
        let e = env(&k);
        let mut st = planet(&geo, omega_10mm());
        let pos = positions(&st, &geo);
        let before = crust(&st);
        let out = step(&mut st, &geo, &pos, &e);
        assert_eq!(out.rifts, 1, "{out:?}");
        assert!(out.columns > 4, "{out:?}");
        assert_eq!(st.rift_fill.len(), 1);
        let fill = &st.rift_fill[0];
        assert_eq!((fill.a, fill.b), (0, 1));
        assert!((fill.total_m() - out.stretched_m).abs() < 1e-6 * out.stretched_m);
        let after = crust(&st) + fill.total_m();
        assert!((after - before).abs() < 1e-6 * before, "{before} {after}");
        // The crust it gives is about the opening's area over the step times a column: the sides
        // move apart at 20 mm/yr (10 each way, along x at +y) over a rift of W km, u W δt T.
        let rift_km = out.stretched_m * e.norm.sample_area_km2() / (35_100.0 * 20.0 * 2.0);
        assert!(rift_km > 500.0 && rift_km < 9000.0, "{rift_km}");
        // Nearer the rift the crust thinned more.
        let thinned = |lo: f64, hi: f64| -> f64 {
            let v: Vec<f64> = (0..N)
                .filter(|&i| {
                    let p = geo.points()[i];
                    let x = p.x.abs() * MODEL_KM;
                    p.y > CAP_Y + 0.05 && x >= lo && x < hi
                })
                .map(|i| 35_100.0 - f64::from(st.samples[i].thickness_m))
                .collect();
            v.iter().sum::<f64>() / v.len().max(1) as f64
        };
        let (near, far) = (thinned(0.0, 300.0), thinned(1500.0, 2000.0));
        assert!(near > 2.0 * far && near > 0.0, "{near} {far}");
        // The ocean floor did not take part.
        assert!(
            st.samples
                .iter()
                .filter(|s| s.kind == OCEANIC)
                .all(|s| s.thickness_m == 7000.0)
        );

        let mut closing = planet(&geo, -omega_10mm());
        let pos = positions(&closing, &geo);
        assert_eq!(step(&mut closing, &geo, &pos, &e), Extension::default());
        assert!(closing.rift_fill.is_empty());

        let off = TectonicParams::default();
        let mut still = planet(&geo, omega_10mm());
        let pos = positions(&still, &geo);
        assert_eq!(
            step(&mut still, &geo, &pos, &env(&off)),
            Extension::default()
        );
        assert_eq!(still.samples, planet(&geo, omega_10mm()).samples);
    }

    /// A weaker side takes more of the opening: plate 1's lithosphere is hot, plate 0's cold.
    #[test]
    fn the_weaker_side_of_a_rift_takes_more_of_its_opening() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = rifting();
        let e = env(&k);
        let mut st = planet(&geo, omega_10mm());
        for (i, s) in st.samples.iter_mut().enumerate() {
            if s.kind == CONTINENTAL && geo.points()[i].x >= 0.0 {
                s.lithosphere_km = 60.0;
            }
        }
        let thick0: Vec<f32> = st.samples.iter().map(|s| s.thickness_m).collect();
        let pos = positions(&st, &geo);
        step(&mut st, &geo, &pos, &e);
        let lost = |plate: u32| -> f64 {
            (0..N)
                .filter(|&i| st.plate_of(i) == plate)
                .map(|i| f64::from(thick0[i] - st.samples[i].thickness_m))
                .sum()
        };
        assert!(lost(1) > 2.0 * lost(0), "{} {}", lost(0), lost(1));
    }

    fn gap(i: u32, d: f64) -> GapPoint {
        GapPoint {
            i,
            pair: (0, 1),
            edge_rad: d,
        }
    }

    /// At a resampling the fill becomes the rift's floor: even columns of its mix while they hold
    /// 10 km of crystalline crust or more, else 10 km columns nearest the plates' edges first and
    /// new sea floor beyond (break-up). A fill with no gap waits; one whose plate died goes to
    /// the continental crust nearest where it came from.
    #[test]
    fn a_resampling_lays_the_fill_as_the_rifts_floor_or_breaks_up() {
        let k = TectonicParams::default();
        let column = continental_column();
        let total_col: f64 = column.iter().map(|&x| f64::from(x)).sum();
        let fill_of = |columns: f64| RiftFill {
            a: 0,
            b: 1,
            pools: std::array::from_fn(|p| f64::from(column[p]) * columns),
            lithosphere_km_m: 80.0 * total_col * columns,
            centroid: [0.0, 1.0, 0.0],
            waited: 0,
        };
        let ocean = || {
            let mut s = Sample::blank(0);
            s.pools = oceanic_column();
            s.thickness_m = 7000.0;
            s
        };
        let points: Vec<DVec3> = (0..6)
            .map(|i| DVec3::new(0.0, 1.0, f64::from(i) * 0.01).normalize())
            .collect();
        let gaps: Vec<GapPoint> = (0..4).map(|i| gap(i, 0.01 * f64::from(4 - i))).collect();
        let n_cont = |s: &[Sample]| -> Vec<usize> {
            (0..s.len()).filter(|&i| s[i].kind == CONTINENTAL).collect()
        };

        // Two columns' crust over four gap samples: even, half a column each.
        let mut fills = vec![fill_of(2.0)];
        let mut samples: Vec<Sample> = (0..6).map(|_| ocean()).collect();
        let laid = lay(
            &mut fills,
            &mut samples,
            &gaps,
            &points,
            |_| true,
            &k,
            9.81,
            0.0,
        );
        assert!(fills.is_empty());
        assert_eq!(laid.samples, 4);
        assert_eq!(n_cont(&samples), vec![0, 1, 2, 3]);
        for s in &samples[..4] {
            assert!((f64::from(s.thickness_m) - total_col / 2.0).abs() < 1.0);
            assert!((s.lithosphere_km - 80.0).abs() < 1e-3);
            assert_eq!(s.regime, regime::RIFT);
        }
        assert!((laid.floor_m - 2.0 * total_col).abs() < 1.0);

        // Half a column over four (4.1 km of crystalline crust each): break-up, one column of
        // it nearest the edges (gap sample 3), sea floor beyond.
        let mut fills = vec![fill_of(0.5)];
        let mut samples: Vec<Sample> = (0..6).map(|_| ocean()).collect();
        lay(
            &mut fills,
            &mut samples,
            &gaps,
            &points,
            |_| true,
            &k,
            9.81,
            0.0,
        );
        assert_eq!(n_cont(&samples), vec![3]);
        assert!((f64::from(samples[3].thickness_m) - total_col / 2.0).abs() < 1.0);

        // No gap between the pair: it waits while both live …
        let mut fills = vec![fill_of(1.0)];
        let mut samples: Vec<Sample> = (0..6).map(|_| ocean()).collect();
        let laid = lay(
            &mut fills,
            &mut samples,
            &[],
            &points,
            |_| true,
            &k,
            9.81,
            0.0,
        );
        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].waited, 1);
        assert_eq!(laid, Laid::default());
        // … and with plate 1 dead goes to the continental crust nearest where it came from.
        let mut land = Sample::blank(0);
        land.kind = CONTINENTAL;
        land.pools = column;
        land.thickness_m = total_col as f32;
        samples[5] = land;
        let laid = lay(
            &mut fills,
            &mut samples,
            &[],
            &points,
            |p| p == 0,
            &k,
            9.81,
            0.0,
        );
        assert!(fills.is_empty());
        assert!((laid.fallback_m - total_col).abs() < 1.0);
        assert!((f64::from(samples[5].thickness_m) - 2.0 * total_col).abs() < 1.0);
    }
}

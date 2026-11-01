// SPDX-License-Identifier: LGPL-3.0-or-later
//! A rift stretches the continental lithosphere on both its sides before it breaks up
//! (`tectonics.rift_stretching`, off by default; ledger PLT-781 to PLT-787). The design, the
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
//!   Σ_i ε̇_i A = u W,   ε̇_i = u W w_i / (A Σ_j w_j),
//!
//! over the visible continental columns of the two plates that take part (below), A a sample's
//! area and w_i a column's weight by its mode.
//!
//! # Narrow and wide
//! Buck (1991) has three modes, "(1) core complex mode (concentrated upper crustal extension with
//! lower crustal thinning over a broad area ...), (2) wide rift mode (uniform crustal and mantle
//! lithospheric thinning over a width greater than the lithospheric thickness), and (3) narrow
//! rift mode (concentrated crustal and mantle lithospheric extension)", set by "crustal
//! thickness, heat flow, and strain rate" (abstract); "the wide rift and the 'core complex' modes
//! are only predicted for thick, hot crust" (Buck 2007, p. 28), and Brun (1999, abstract) has the
//! same split, necking where "the greatest strength [is] located in the sub-Moho mantle" against
//! spreading where the strength peaks "at the base of upper brittle crust". The model's test is
//! Buck's own map ([`in_wide_band`], PLT-786): his modes against crustal thickness and surface
//! heat flow (Buck 1991, as Buck 2007's Fig. 30 reprints it), whose wide-rift band runs from a
//! narrow edge ([`BAND_EDGE`]: about 41.5 km of crust at 74 mW/m², 34 at 83, 28 at 98, 24 at
//! 110, 20 at 136; read off the plot) to the core-complex field above it, and ends in a tip near
//! 56 km at about 70 mW/m², under which no wide rift is predicted. The heat flow is the plate
//! model's own, q = 26 + 3,375 / L (`thermal::conductive_mw_m2`). A side whose edge columns' mean
//! crust and lithosphere put it in the band or above it is wide, else narrow (the test
//! `bucks_band_puts_his_rifts_in_their_modes`). On the model's geotherm the band's edge lies at a
//! Moho of 810 to 910 °C: the fixed 700 °C test it replaced (Rey, Vanderhaeghe & Teyssier 2001,
//! p. 443) made sides with a Moho of 700 to 840 °C wide that Buck's map has narrow, and the rift
//! trace found those wide sides thinning the continents up to thousands of km from their rifts
//! (docs/design/continental-extension.md, "The rift trace").
//! - A narrow side takes the strain evenly over half the rift's width, the columns within
//!   L_lith / 2 of the rift (L_lith its edge columns' mean lithosphere): "narrow means about as
//!   wide as the lithosphere is thick" (Buck et al. 1999, p. 673); "on the order of 100 km wide"
//!   (Buck 2007, p. 25: the Rhinegraben, Suez, East Africa, Baikal, the Rio Grande).
//! - A wide side takes it evenly over its region in the band: the columns of its plate's
//!   continental crust that are each in the band, reached from the rift through the band
//!   (PLT-787). Buck's wide rift is "uniform crustal and mantle lithospheric thinning over a
//!   width greater than the lithospheric thickness" (1991, abstract), made by a zone of pure
//!   shear as wide as the lithosphere is thick that migrates: "If the force increased the
//!   extension was taken to migrate. Migration of extension would eventually produce a wide
//!   rift" (Buck 2007, p. 28). The sources give no speed for the migration, so a step spreads its
//!   strain over the whole region, which the band bounds: the Basin and Range's region of normal
//!   faulting is "as much as 800 km wide" (Buck 2007, p. 25). (Until 2026-10-31 the strain fell
//!   off as e^(−x / L) with L = 2 W / π, the collision's thin sheet, over 4 L.)
//!
//! With the thermal state off every side is narrow under `thermal_reference_km`.
//!
//! A column keeps e^(−ε̇ δt) of its crust in its own mix, never less than 10 km of crystalline
//! crust (Brune et al. 2014's hyper-extended limit, PLT-661), its lithosphere and keel thinned by
//! the same factor (`thermal::stretch`, McKenzie's uniform stretching, PLT-476), its surface down
//! by the Airy change of what it lost (but not below the Airy height of what it keeps,
//! `collapse::toward`). A column at the limit takes no share: what the rift's speed asks beyond
//! the columns that can still thin is break-up, new sea floor. As a wide side thins its crust and
//! lithosphere by the same share its heat flow rises, and it stays in the band until its crust is
//! about 20 km; as its lithosphere cools the band's edge rises, and under about 70 mW/m² it is
//! narrow whatever its crust (`a_wide_side_leaves_the_band_as_it_thins_and_cools`): Buck's
//! "temporal progression from core complex mode to wide rift mode to narrow rift mode" (abstract),
//! and Hopper & Buck 1996's "During rifting, the crust thins continuously and one would expect
//! from the mode maps that eventually a transition to narrow rift mode should occur" (p. 20,181).
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
//! A strain solve: the mode and the decay distribute the opening's area; the area itself is the
//! plates' motion. Nor the collapse: a rift needs an opening boundary (Rey et al. 2001's "free
//! boundary", or "a phase of tectonic divergence", p. 445), and the fixed-boundary collapse
//! (`collapse.rs`) needs none. Nor extension while the plates converge (Tibet's east-west
//! extension, which Molnar, England & Martinod 1993 put at about 2.8 km of crustal thinning since
//! 8 Ma, p. 374): a column needs a rift on its plate.

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

/// What one step did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Extension {
    /// Pairs of plates with a rift.
    pub rifts: u32,
    /// Columns that thinned.
    pub columns: u32,
    /// Crust moved into the rifts' fills, m summed over samples.
    pub stretched_m: f64,
    /// One record per pair with a rift, for the rift trace (`rift_trace.rs`; no rule reads it).
    pub records: Vec<RiftRecord>,
}

/// What [`lay`] did at a resampling, m summed over samples.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Laid {
    pub floor_m: f64,
    pub fallback_m: f64,
    /// Gap samples made the rift's continental floor.
    pub samples: u32,
    /// One record per fill, for the rift trace (`rift_trace.rs`; no rule reads it).
    pub records: Vec<LayRecord>,
}

/// One rift in one step (the rift trace's row; measurement only).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RiftRecord {
    pub a: u32,
    pub b: u32,
    /// The boundary's length W, km, and its mean opening speed u, mm/yr.
    pub length_km: f64,
    pub speed_mm_yr: f64,
    /// The opening's area over the step, u W δt, km² …
    pub asked_km2: f64,
    /// … and what the columns took of it, Σ A (−ln keep), km² (less than asked when the
    /// columns that take part reach the hyper-extended limit).
    pub taken_km2: f64,
    /// Crust moved into the fill, m summed over samples.
    pub stretched_m: f64,
    /// A sample's area, km².
    pub sample_km2: f64,
    /// Plate `a`'s side, then plate `b`'s.
    pub sides: [SideRecord; 2],
}

/// One side of a rift in one step (see [`RiftRecord`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SideRecord {
    /// 0 no edge on this side, 1 narrow, 2 wide.
    pub mode: u8,
    /// Its edge columns, and their mean crust (km), lithosphere (km) and Moho (°C), the means
    /// the mode test reads.
    pub edges: u32,
    pub crust_km: f64,
    pub lith_km: f64,
    pub moho_c: f64,
    /// How far the search reached, km.
    pub reach_km: f64,
    /// Columns within the reach, those of them with a positive weight, those that thinned, and
    /// those already at the hyper-extended limit.
    pub found: u32,
    pub weighted: u32,
    pub thinned: u32,
    pub at_limit: u32,
    /// The farthest column that thinned, km, and the columns' distance weighted by the crust
    /// they gave, km.
    pub far_km: f64,
    pub mean_km: f64,
    /// Crust this side gave, m summed over samples.
    pub given_m: f64,
    /// What Earth's rifts start on (docs/design/continental-extension.md, "The rifting rule
    /// against Earth's rifts"), for the trace: the share of the edge columns that a collision
    /// ever stamped (`Sample::collision_onset`, an old suture or belt: an inherited weakness),
    /// whether its plate's large igneous province still boosts its rifting (magma), and the
    /// plate's parent (-1: none) and birth, Myr (a split's two sides).
    pub belt: f64,
    pub lip: u8,
    pub parent: i64,
    pub born_myr: f64,
}

/// One fill at one resampling (the rift trace's row; measurement only).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayRecord {
    pub a: u32,
    pub b: u32,
    /// What the fill held, m summed over samples, and the mean thickness of the columns it came
    /// from, m.
    pub total_m: f64,
    pub source_m: f64,
    /// Gap samples between the pair, and how many took a column of the fill (fewer: break-up).
    pub gap: u32,
    pub columns: u32,
    /// The column each took, m, and the even spread over the columns, m.
    pub column_m: f64,
    pub even_m: f64,
    /// The crystalline crust of a column, m.
    pub crystalline_m: f64,
    /// Laid as the floor, left waiting, and sent to the fallback, m summed over samples.
    pub floor_m: f64,
    pub waits_m: f64,
    pub fallback_m: f64,
    /// Resamplings it had waited before this one.
    pub waited: u32,
}

/// The narrow edge of Buck's (1991) wide-rift band, as Buck 2007's Fig. 30 reprints his mode
/// map (crustal thickness against surface heat flow): (heat flow, mW/m²; the crust from which the
/// lithosphere is in the wide-rift mode, or the core-complex mode above it, km). Read off the plot
/// by the research sub-agent (2026-10-31; approximately, about ±1.5 km and ±3 mW/m²): the band
/// ends in a tip near 56 km at about 70 mW/m², under which no wide rift is predicted; at 136 mW/m²
/// its edge is at 20 km or less (the plot's edge, held beyond).
// ledger: PLT-786
pub const BAND_EDGE: [(f64, f64); 6] = [
    (70.0, 56.0),
    (74.0, 41.5),
    (83.0, 34.0),
    (98.0, 28.0),
    (110.0, 24.0),
    (136.0, 20.0),
];

/// The crust from which a lithosphere with surface heat flow `q_mw_m2` is in Buck's wide band
/// ([`BAND_EDGE`], linear between its points), km; None under the band's tip.
// ledger: PLT-786
pub fn band_edge_km(q_mw_m2: f64) -> Option<f64> {
    let (q0, _) = BAND_EDGE[0];
    if q_mw_m2.is_nan() || q_mw_m2 < q0 {
        return None;
    }
    for w in BAND_EDGE.windows(2) {
        let ((qa, ha), (qb, hb)) = (w[0], w[1]);
        if q_mw_m2 <= qb {
            return Some(ha + (hb - ha) * (q_mw_m2 - qa) / (qb - qa));
        }
    }
    Some(BAND_EDGE[BAND_EDGE.len() - 1].1)
}

/// Whether a crust `crust_km` thick over a lithosphere `lith_km` deep is in Buck's wide band (or
/// the core-complex field above it), its heat flow the plate model's own
/// (`thermal::conductive_mw_m2`, q = 26 + 3,375 / L).
// ledger: PLT-786, PLT-467
pub fn in_wide_band(crust_km: f64, lith_km: f64) -> bool {
    band_edge_km(thermal::conductive_mw_m2(lith_km)).is_some_and(|h| crust_km >= h)
}

/// The temperature at the base of a crust `crust_km` thick over a lithosphere `lith_km` deep,
/// °C, on the plate model's geotherm (`thermal::geotherm_share`, T_m at L).
// ledger: ISO-138, PLT-782
pub fn moho_c(crust_km: f64, lith_km: f64) -> f64 {
    thermal::MANTLE_C * thermal::geotherm_share(lith_km, crust_km.min(lith_km).max(0.0))
}

/// A sample's lithosphere, km; with no thermal state, `reference_km`.
fn sample_lith_km(s: &Sample, reference_km: f64) -> f64 {
    if s.lithosphere_km > 0.0 {
        f64::from(s.lithosphere_km)
    } else {
        reference_km
    }
}

/// A sample's Moho temperature, °C (the record's; no rule reads it).
fn sample_moho_c(s: &Sample, reference_km: f64) -> f64 {
    moho_c(
        f64::from(s.thickness_m) / 1000.0,
        sample_lith_km(s, reference_km),
    )
}

/// Whether a sample is in Buck's wide band ([`in_wide_band`]).
fn sample_in_band(s: &Sample, reference_km: f64) -> bool {
    in_wide_band(
        f64::from(s.thickness_m) / 1000.0,
        sample_lith_km(s, reference_km),
    )
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
    let thermal_on = k.thermal_state;
    let reference = k.thermal_reference_km;
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
        let mut record = RiftRecord {
            a,
            b,
            length_km,
            speed_mm_yr: speed,
            asked_km2: speed * length_km * env.dt_myr,
            sample_km2: area,
            ..RiftRecord::default()
        };
        // The columns that take part on each side, along their own plate's continental crust,
        // with their weights by the side's mode (ledger: PLT-782); each with its side and its
        // distance from the rift, km, for the record.
        let mut cols: Vec<(u32, f64, usize, f64)> = Vec::new();
        for (k_side, p) in [a, b].into_iter().enumerate() {
            let side: Vec<u32> = es.iter().filter(|e| e.plate == p).map(|e| e.v).collect();
            if side.is_empty() {
                continue;
            }
            let samples = &state.samples;
            let mean = |f: &dyn Fn(&Sample) -> f64| -> f64 {
                side.iter().map(|&v| f(&samples[v as usize])).sum::<f64>() / side.len() as f64
            };
            let edge_moho = mean(&|s| sample_moho_c(s, reference));
            let lith = mean(&|s| {
                if thermal_on && s.lithosphere_km > 0.0 {
                    f64::from(s.lithosphere_km)
                } else {
                    reference
                }
            });
            let crust = mean(&|s| f64::from(s.thickness_m) / 1000.0);
            let belt = mean(&|s| f64::from(u8::from(s.collision_onset != 0)));
            let wide = thermal_on && in_wide_band(crust, lith);
            // A narrow side reaches half its lithosphere from the rift; a wide one its whole
            // region in the band, through the band (PLT-786, PLT-787).
            let reach_km = if wide { f64::INFINITY } else { 0.5 * lith };
            let rec = &mut record.sides[k_side];
            rec.mode = if wide { 2 } else { 1 };
            rec.edges = side.len() as u32;
            rec.crust_km = crust;
            rec.lith_km = lith;
            rec.moho_c = edge_moho;
            rec.reach_km = reach_km;
            rec.belt = belt;
            if let Some(plate) = state.plates.get(p as usize) {
                rec.lip = u8::from(env.t_myr < plate.lip_until_myr);
                rec.parent = plate.parent.map_or(-1, i64::from);
                rec.born_myr = plate.born_myr;
            }
            let src: Vec<(u32, f64, u32)> = side.iter().map(|&v| (v, 0.0, v)).collect();
            let on = |u: u32| {
                let s = &samples[u as usize];
                plate_of[u as usize] == p
                    && s.kind == CONTINENTAL
                    && !s.is_hidden()
                    && s.flags & TRENCH == 0
                    && (!wide || sample_in_band(s, reference))
            };
            let max_rad = if wide {
                std::f64::consts::PI
            } else {
                env.norm.rad(reach_km)
            };
            for (u, d, _) in dijkstra_sparse(geo, &src, |_, u| on(u), max_rad) {
                let s = &samples[u as usize];
                let x = env.norm.km(d);
                let at_limit = least_keep(&s.pools) >= 1.0;
                // Uniform over the columns that take part (Buck 1991's wide rift: "uniform
                // crustal and mantle lithospheric thinning over a width greater than the
                // lithospheric thickness"); on a wide side a column out of the band (an edge
                // column below the side's mean) takes none.
                let w = if at_limit || (wide && !sample_in_band(s, reference)) {
                    0.0
                } else {
                    1.0
                };
                rec.found += 1;
                rec.weighted += u32::from(w > 0.0);
                rec.at_limit += u32::from(at_limit);
                cols.push((u, w, k_side, x));
            }
        }
        // A column on both sides' reach (none: each side is its own plate) counts once.
        cols.sort_by_key(|c| c.0);
        cols.dedup_by_key(|c| c.0);
        let weights: Vec<f64> = cols.iter().map(|c| c.1).collect();
        let rates = strain_rates(speed, length_km, area, &weights);
        let mut fill = RiftFill {
            a,
            b,
            ..RiftFill::default()
        };
        for (&(u, _, k_side, x_km), &rate) in cols.iter().zip(&rates) {
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
            let rec = &mut record.sides[k_side];
            rec.thinned += 1;
            rec.far_km = rec.far_km.max(x_km);
            rec.mean_km += given * x_km;
            rec.given_m += given;
            record.taken_km2 += -keep.ln() * area;
            record.stretched_m += given;
            fill.source_m += given * f64::from(thickness(&before));
            s.thickness_m = thickness(&s.pools);
            fill.basement_m += f64::from(s.basement_m) * (1.0 - keep);
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
        for rec in &mut record.sides {
            if rec.given_m > 0.0 {
                rec.mean_km /= rec.given_m;
            }
        }
        out.records.push(record);
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
        f.basement_m += fill.basement_m;
        f.source_m += fill.source_m;
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
        if total.is_nan() || total <= 0.0 {
            continue;
        }
        let lith = fill.lithosphere_km_m / total;
        let members = by_pair.get(&(fill.a, fill.b)).cloned().unwrap_or_default();
        let mut record = LayRecord {
            a: fill.a,
            b: fill.b,
            total_m: total,
            source_m: fill.source_m / total,
            gap: members.len() as u32,
            waited: fill.waited,
            ..LayRecord::default()
        };
        if members.is_empty() {
            if alive(fill.a) && alive(fill.b) {
                record.waits_m = total;
                let mut f = fill;
                f.waited += 1;
                kept.push(f);
            } else {
                let to = fallback(&fill, samples, points, k, gravity, z_0, sea_m);
                laid.fallback_m += to;
                record.fallback_m = to;
            }
            laid.records.push(record);
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
        // No gap sample takes a thicker column than the columns the crust came from had before
        // they stretched: the opening's crust is at most their own, never stacked (the pure shear
        // of PLT-781 spreads crust, it does not thicken it). A gap narrower than the fill (a
        // resampling that found few of its samples between the pair, or a fill that waited)
        // takes that much, and the rest waits for the next resampling while both plates live,
        // else goes to the fallback (ledger: PLT-784).
        let source = fill.source_m / total;
        let even = total / m as f64;
        let column = if source.is_finite() && source > 0.0 {
            even.min(source)
        } else {
            even
        };
        let placed = column * m as f64;
        let basement = (fill.basement_m / m as f64).min(column);
        for g in &members[..m] {
            let s = &mut samples[g.i as usize];
            *s = floor_sample(s.group, &mix, column, lith, k, gravity, z_0, sea_m);
            s.basement_m = basement as f32;
            laid.samples += 1;
        }
        laid.floor_m += placed;
        record.columns = m as u32;
        record.column_m = column;
        record.even_m = even;
        record.crystalline_m = crystalline_share * column;
        record.floor_m = placed;
        let rest = 1.0 - placed / total;
        if rest > 1.0e-9 {
            let mut f = fill;
            f.pools.iter_mut().for_each(|x| *x *= rest);
            f.lithosphere_km_m *= rest;
            f.basement_m = (f.basement_m - basement * m as f64).max(0.0);
            f.source_m *= rest;
            f.centroid.iter_mut().for_each(|x| *x *= rest);
            if alive(f.a) && alive(f.b) {
                record.waits_m = f.total_m();
                f.waited += 1;
                kept.push(f);
            } else {
                let to = fallback(&f, samples, points, k, gravity, z_0, sea_m);
                laid.fallback_m += to;
                record.fallback_m = to;
            }
        }
        laid.records.push(record);
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
        // The rift trace's record: the pair, its columns and its crust, and the opening's area
        // all taken (no column is near its limit).
        assert_eq!(out.records.len(), 1);
        let r = &out.records[0];
        assert_eq!((r.a, r.b), (0, 1));
        assert_eq!(r.sides.iter().map(|s| s.thinned).sum::<u32>(), out.columns);
        assert!((r.stretched_m - out.stretched_m).abs() < 1e-9 * out.stretched_m);
        assert!((r.taken_km2 / r.asked_km2 - 1.0).abs() < 1e-6, "{r:?}");
        for s in &r.sides {
            assert!(s.mode > 0 && s.edges > 0 && s.thinned > 0, "{s:?}");
            assert!(
                s.far_km <= s.reach_km + 1e-9 && s.mean_km <= s.far_km + 1e-9,
                "{s:?}"
            );
        }
        // The crust it gives is about the opening's area over the step times a column: the sides
        // move apart at 20 mm/yr (10 each way, along x at +y) over a rift of W km, u W δt T.
        let rift_km = out.stretched_m * e.norm.sample_area_km2() / (35_100.0 * 20.0 * 2.0);
        assert!(rift_km > 500.0 && rift_km < 12_000.0, "{rift_km}");
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

    /// Buck's band puts each of the rifts on his mode map (Buck 2007's Fig. 30, from Buck 1991;
    /// crustal thickness and surface heat flow read off the plot by the research sub-agent,
    /// approximately) in its mode, the lithosphere from the heat flow by the model's own law
    /// (`thermal::conductive_mw_m2`, q = 26 + 3,375 / L): the Altiplano and Tibet in the
    /// core-complex field above the band, a wide mode here.
    // ledger: PLT-786
    #[test]
    fn bucks_band_puts_his_rifts_in_their_modes() {
        let lith = |q: f64| thermal::MANTLE_HEAT_MW_M2_KM / (q - thermal::CRUST_HEAT_MW_M2);
        for (name, crust_km, q, wide) in [
            ("northern Basin and Range", 33.0, 90.0, true),
            ("Altiplano", 48.0, 90.0, true),
            ("Tibet", 64.0, 87.0, true),
            ("Rhinegraben", 36.0, 68.0, false),
            ("East Africa and Baikal", 40.0, 51.0, false),
            ("northern Red Sea", 30.0, 45.0, false),
            ("southern Rio Grande", 26.0, 87.0, false),
        ] {
            assert!((thermal::conductive_mw_m2(lith(q)) - q).abs() < 1e-9);
            assert_eq!(in_wide_band(crust_km, lith(q)), wide, "{name}");
        }
        // The band's edge at its points, and nothing under its tip.
        for (q, h) in BAND_EDGE {
            assert!((band_edge_km(q).unwrap() - h).abs() < 1e-9);
        }
        assert_eq!(band_edge_km(69.9), None);
        assert!((band_edge_km(90.5).unwrap() - 31.0).abs() < 1e-9);
        assert_eq!(band_edge_km(200.0), Some(20.0));
        // On the model's geotherm the band's narrow edge lies at a Moho of 810 to 910 °C from
        // 74 to 136 mW/m², not at a fixed 700 °C (the test this replaces; ext-cum2's sides with
        // a Moho of 700 to 840 °C were wide here and are narrow on Buck's map).
        for &(q, h) in &BAND_EDGE[1..] {
            let t = moho_c(h, lith(q));
            assert!(
                (800.0..=910.0).contains(&t),
                "edge at {q} mW/m²: Moho {t:.0} °C"
            );
        }
    }

    /// As a wide side thins by pure shear (crust and lithosphere by the same share, its heat flow
    /// rising, `thermal::stretch`) it stays in the band until its crust is about 20 km, and it
    /// leaves the band as its lithosphere cools: Hopper & Buck 1996's "During rifting, the crust
    /// thins continuously and one would expect from the mode maps that eventually a transition to
    /// narrow rift mode should occur" (p. 20,181).
    // ledger: PLT-786
    #[test]
    fn a_wide_side_leaves_the_band_as_it_thins_and_cools() {
        let (h, l) = (
            35.0,
            thermal::MANTLE_HEAT_MW_M2_KM / (90.0 - thermal::CRUST_HEAT_MW_M2),
        );
        assert!(in_wide_band(h, l));
        for keep in [0.9, 0.8, 0.7, 0.6] {
            assert!(in_wide_band(h * keep, l * keep), "keep {keep}");
        }
        assert!(!in_wide_band(h * 0.5, l * 0.5));
        // Cooled under 70 mW/m² (a lithosphere of 77 km or more) it is narrow whatever its crust.
        assert!(!in_wide_band(h, 80.0));
        assert!(!in_wide_band(60.0, 80.0));
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
            basement_m: 0.0,
            centroid: [0.0, 1.0, 0.0],
            waited: 0,
            source_m: total_col * total_col * columns,
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
        assert_eq!(n_cont(&samples), vec![3]);
        assert!((f64::from(samples[3].thickness_m) - total_col / 2.0).abs() < 1.0);
        // The trace's record: four gap samples, one column (break-up), none waits.
        let r = laid.records[0];
        assert_eq!((r.gap, r.columns), (4, 1), "{r:?}");
        assert!(
            (r.floor_m - total_col / 2.0).abs() < 1.0 && r.waits_m == 0.0,
            "{r:?}"
        );
        assert!(r.crystalline_m >= HYPER_EXTENDED_M, "{r:?}");

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
        assert_eq!((laid.floor_m, laid.fallback_m, laid.samples), (0.0, 0.0, 0));
        let r = laid.records[0];
        assert_eq!((r.gap, r.columns, r.waited), (0, 0, 0));
        assert!((r.waits_m - total_col).abs() < 1.0, "{r:?}");
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

        // Ten columns' crust over four gap samples (a gap narrower than the fill): each takes the
        // columns' own thickness, no more, and the other six columns' worth waits while both
        // plates live (the ext-cum arm's divergence, 2026-10-31: spread evenly, a few gap samples
        // took hundreds of km of crust and the surface ran away) …
        let mut fills = vec![fill_of(10.0)];
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
        assert_eq!(laid.samples, 4);
        for s in &samples[..4] {
            assert!(
                (f64::from(s.thickness_m) - total_col).abs() < 1.0,
                "{}",
                s.thickness_m
            );
        }
        assert!((laid.floor_m - 4.0 * total_col).abs() < 1.0);
        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].waited, 1);
        assert!((fills[0].total_m() - 6.0 * total_col).abs() < 1.0);
        assert!((fills[0].source_m / fills[0].total_m() - total_col).abs() < 1.0);
        let r = laid.records[0];
        assert_eq!((r.gap, r.columns), (4, 4), "{r:?}");
        assert!(
            (r.column_m - total_col).abs() < 1.0 && r.even_m > 2.0 * total_col,
            "{r:?}"
        );
        assert!((r.waits_m - 6.0 * total_col).abs() < 1.0, "{r:?}");
        // … and goes to the fallback when one of them has died.
        let mut fills = vec![fill_of(10.0)];
        let mut samples: Vec<Sample> = (0..6).map(|_| ocean()).collect();
        samples[5] = land;
        let laid = lay(
            &mut fills,
            &mut samples,
            &gaps,
            &points,
            |p| p == 0,
            &k,
            9.81,
            0.0,
        );
        assert!(fills.is_empty());
        assert!((laid.floor_m - 4.0 * total_col).abs() < 1.0);
        assert!((laid.fallback_m - 6.0 * total_col).abs() < 1.0);
    }
}

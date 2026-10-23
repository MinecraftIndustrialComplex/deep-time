// SPDX-License-Identifier: LGPL-3.0-or-later
//! Orogenic collapse: thickened continental crust spreads back out and gives its area back (Ben,
//! decisions log 2026-10-03: "Merge, then give area back").
//!
//! # Why
//! Continental crust that crosses a collision front is stacked into the belt and the wedge
//! (`belts.rs`): its volume stays, its area is gone. Area ≈ volume ÷ thickness, so unless the
//! thickened crust thins again a planet loses continental area with every collision (0.31 → 0.27
//! of the planet at 1000 Myr when the belts landed, build log "Plate tasks 3 and 4").
//!
//! # What happens on Earth
//! Thickened crust stands high, and its weight (its gravitational potential energy over the
//! lowland around it) pushes outwards. While convergence goes on, the push of the plates holds it
//! up; when the push weakens, the hot, weak orogen spreads under its own weight and thins:
//! orogenic or gravitational collapse (Dewey 1988; Rey, Vanderhaeghe & Teyssier 2001). The
//! Variscan orogen of Europe collapsed into a basin-and-range province within a few tens of Myr
//! of its thickening (Ménard & Molnar 1988); the North American Basin and Range was a thickened
//! plateau that extended by about a factor of two (Hamilton & Myers 1966 onwards). An orogen that
//! extends widens, and the plate it sits on carries more continent.
//!
//! # The rule
//! The driver is the crust's thickness. Under Airy isostasy the height of a column and the
//! weight that pushes it outwards both follow from its thickness, and in this model the
//! thickness is the better record of the two: the surface stage owns the elevation (its
//! feedback overwrites the samples' heights each step), and crust stacked where continents
//! overlap at a resampling (`ridges.rs`, `pools::accrete`) thickens a column without raising it.
//! Each step, every visible continental sample that no orogeny has raised for
//! `collapse_quiet_myr` (a belt being stacked or a margin being uplifted is held up by the
//! convergence; collapse follows the end of the thickening) and whose crust is thicker than
//! `collapse_thickness_km` (T_c) releases
//!
//!   q = (1 − e^(−δt/τ)) · (T − T_c)
//!
//! of its thickness (τ = `collapse_myr`, T its thickness). The crust released moves to the
//! nearest room on the same plate, within `collapse_reach_km`: the samples thinner than the
//! model's normal continental column T_n ([`normal_thickness_m`], 35.1 km), nearest first, are
//! filled up to T_n (continental crust that erosion or rifting thinned, then the plate's own
//! ocean floor at the continent's edge); the reach is at least 1.5 sample spacings, so a coarse
//! lattice still reaches a column's neighbours. An ocean sample filled to T_n becomes continental: the
//! continent's edge moves out. What finds no room within reach is not released this step.
//! Volume is conserved: the rock moved is the collapsing column's own mix, thinned in
//! proportion, as pure-shear extension does.
//!
//! The surface follows by Airy (`pools::COLUMN_ISOSTATIC_FACTOR`, 7.0 m of crust per m of height,
//! less the pull of an eclogite root, `pools::isostatic_change_m`; PLT-278 to PLT-281),
//! within the Airy height of the column, z_A(T) = z_0 + (T − T_n) / 7.0 (z_0 the middle of
//! `continent_elevation_m`): a source sinks by q / 7.0 but not below z_A of the crust it keeps,
//! so a root that never raised its surface (an overlap stack) does not drag it down as it
//! spreads; a receiver rises by g / 7.0 but not above z_A of its new column, so ocean floor
//! filled to T_n stands at a continent's height and thin crust that the surface already holds
//! high is not raised further.
//!
//! The area gained is the volume released over T_n: a column of thickness T gives back
//! (T − T_c) / T_n of a sample's area.
//!
//! # While the orogen is still hot (off by default)
//! T_c is what an old, cold root keeps (the Urals, 50-56 km some 300 Myr after their collision).
//! A young orogen thins further: the Variscan root of 55-70 km thinned from about 315 to 290 Ma,
//! as its thickening ended, to a Moho at 30-32 km where it flattened; the Basin and Range went
//! from 54 ± 6 km by 46 % extension (about 37 km); the Aegean is 26 km. Whether a thick crust
//! collapses that far is set by its heat when it was thickened (Wernicke et al. 1987). With
//! `collapse_hot_myr` W > 0 a column whose last orogeny is younger than W thins towards T_n
//! instead of T_c; older crust, and crust no orogeny raised, keeps T_c. W = 40 Myr is the 10 Myr
//! quiet time and the Variscan's 25 to 30 Myr of thinning (ledger PLT-390). Measured and not on
//! (2026-10-21): it lowers the land above 2 km from 0.29 to 0.18 of the land, but the crust it
//! spreads over the ocean floor lifts the sea (the planet's water is fixed), so the land falls by
//! 0.02 to 0.04 and the drowned continental share rises by 0.08 to 0.10.
//!
//! `collapse_continent_first` (PLT-391, off by default) fills the plate's thin continental
//! crust within reach, nearest first, before any ocean floor, so less of the collapse becomes new
//! continent below the sea.
//!
//! # Measured from the sea (off by default)
//! T_n stands at z_0 over the datum, a continent's height when the sea stands at the datum. The
//! sea is the planet's water filling the hypsometry (ISO-085) and stands 500 to 1,200 m over the
//! datum on the Earth-like presets, so T_n crust stands at or under it: a hot orogen thinned to
//! T_n, and ocean floor filled to it, are continent under the sea, and continental crust between
//! T_n and the old lowland's 40 to 42 km (Christensen & Mooney 1995: the continents' mean 41.1 km)
//! is no room for the collapse though it stands lower than the orogen. With
//! `collapse_sea_column` ([`room_column_m`], PLT-412) the collapse's normal column is the one
//! that stands as high over the step's sea as T_n over the datum, T_n + 7.0 S: what a hot orogen
//! thins towards, what its room is filled up to, and the column at which ocean floor turns
//! continental (also for the thickest crust's spreading). Heights still follow Airy in the
//! datum frame: only the column the collapse aims at moves with the sea.
//!
//! # As a thin viscous sheet (off by default)
//! The rule above is local: a column thins at one rate down to T_c, and its crust goes to room
//! within 200 km. On Earth a thickened region also spreads as a whole: the lithosphere is a thin
//! viscous sheet whose depth-integrated deviatoric stress balances the gradient of the
//! potential energy of its columns (England & McKenzie 1982; Houseman & England 1986), so a
//! plateau extends at a rate its potential energy over the lowland around it sets, interior and
//! edges alike (Tibet's east-west extension, England & Houseman 1989; the Basin and Range,
//! Sonder & Jones 1999). With `collapse_sheet_viscosity_pa_s` η_0 > 0 a quiet column an orogeny
//! raised thins in pure shear (plane strain) at
//!
//!   ε̇ = ΔU / (4 η L),   q = T (1 − e^(−ε̇ δt)),  at most T − T_col,
//!
//! where ΔU is the potential energy of the column at its Airy height over the step's sea less
//! that of the collapse's normal column T_col (`forces::column_energy`, FRC-014), L the
//! lithosphere's 100 km (FRC-010) and η = η_0 e^(a / τ_s), a the time since the column's last
//! orogeny and τ_s `collapse_sheet_strength_myr`: the lithosphere stiffens as it cools after the
//! orogeny (τ_s = 62.8 Myr is McKenzie 1978's thermal time; the exponential form is the model's).
//! η_0 = 10²² Pa s is the depth-averaged viscosity found for Tibet (England & Molnar 1997) and the
//! Basin and Range (Flesch et al. 2000). A column releases the larger of this and the threshold
//! law's q. Crust no orogeny raised (cratons, platforms) does not flow; nor, in effect, does an
//! orogen a few hundred Myr old (η 100 times η_0 at 290 Myr). A 4.2 km column under a sea at the
//! datum thins 3.9 % per Myr when its orogeny ends, a 2 km one 1.3 %: e-folds of 26 and 75 Myr. The
//! crust released goes where the rule sends it, to the nearest room on the plate within
//! `collapse_reach_km` (a measured arm sets the reach to the plate's size, so the continent's edge
//! moves out over its ocean floor). The quiet time still holds: an orogeny's push holds a belt up
//! (the force balance counts the belt's potential energy against the convergence, FRC-013).
//! ledger: PLT-414 (the law and η_0), PLT-415 (the strength's growth).
//!
//! # The thickest crust
//! No continental crust on Earth is much thicker than Tibet's or the Altiplano's. Continental
//! crust that a collision carries down further goes into the mantle with the slab (continental
//! subduction), and a root thickened past that limit founders (its lower crust turns to
//! eclogite, denser than the mantle). The model's stacking has no such limit of its own: the
//! belt's cap reads the surface's height (PLT-253), which the surface stage lowers by flexure and
//! erosion while the stacked thickness stays, and an overlap at a resampling adds a whole column
//! to the one on top. [`founder`] takes every visible continental column thicker than
//! `crust_max_km` back to it: the excess first spreads at once to the nearest room on its plate,
//! as the collapse moves crust but whether or not an orogeny holds the column up, filling
//! continental crust only up to `collapse_thickness_km` (lower crust flows out of the thickened
//! belt, but crust thicker than that does not stay, so it builds no second plateau around the
//! belt) and ocean floor up to the normal column; what finds no room goes down from the bottom of
//! the column and is gone (it gives no area back). Since 2026-10-05 (plate task 7, ledger
//! PLT-277); until then all of the excess foundered, and a collision's crust, also accreted a
//! second time at each resampling (PLT-276), foundered at three times the old motion law's rate
//! under the force balance.
//!
//! # What it is not
//! A lower-crustal flow solve. The surface stage's spreading (`surface::spread`) moves the surface
//! above the strength height; this moves crust, and only on the plate model's samples. Where the
//! orogen spreads is the plate's nearest room, not a flow field.
// ledger: PLT-254 (the rule), PLT-255 (τ), PLT-256 (the threshold thickness), PLT-257 (the
// reach), PLT-258 (the thickest crust), PLT-390 (the hot window), PLT-391 (continental room first),
// PLT-412 (the normal column from the sea), PLT-414 and PLT-415 (the thin sheet).

use std::collections::BTreeMap;

use rayon::prelude::*;

use super::Env;
use super::forces::{LITHOSPHERE_KM, YEAR_S, column_energy};
use super::geometry::{Geo, dijkstra_sparse};
use super::pools::{
    COLUMN_ISOSTATIC_FACTOR, continental_column, eclogite_m, isostatic_change_m, root_drop_m,
};
use super::state::{CONTINENTAL, OCEANIC, OROGENY_NONE, PlateState, Sample, TRENCH, pool};

/// `WorldState::globals`: crust moved by the collapse over the run, km³ …
pub const MOVED_GLOBAL: &str = "plates/collapse_km3";
/// … and the ocean floor it turned continental, km² (Earth-normalized).
pub const GAINED_GLOBAL: &str = "plates/collapse_gained_km2";
/// Measurement only (the crust budget): what the collapse held back over the run, km³, from
/// columns an orogeny still holds up …
pub const HELD_GLOBAL: &str = "plates/collapse_held_km3";
/// … and for want of room within reach.
pub const UNPLACED_GLOBAL: &str = "plates/collapse_unplaced_km3";
/// Continental crust lost to the mantle beyond `crust_max_km` over the run, km³.
pub const FOUNDERED_GLOBAL: &str = "plates/foundered_km3";

/// The shortest reach, in sample spacings (the nearest ring of neighbours, PLT-257).
const REACH_MIN_SPACINGS: f64 = 1.5;

/// Order in which crust beyond the thickest column goes down: the bottom of the column first.
const ROOT_ORDER: [usize; pool::COUNT] = [
    pool::MAFIC_PLUTONIC,
    pool::MAFIC_VOLCANIC,
    pool::FELSIC_PLUTONIC,
    pool::METAMORPHIC,
    pool::FELSIC_VOLCANIC,
    pool::SEDIMENTARY,
    pool::SEDIMENT,
];

/// Continental crust thicker than `crust_max_km` spreads the excess to the nearest room on its
/// plate at once (as the collapse moves crust, [`spread`], whether or not an orogeny holds it
/// up), and what finds no room goes to the mantle from the bottom of its column (see the module
/// docs, "The thickest crust"); returns the crust lost, km² × m, and adds what spread to `out`.
/// `sea_m` is the step's sea (m over the datum), read only with `collapse_sea_column` (the ocean
/// floor's fill, [`room_column_m`]). Runs every step, before [`step`].
pub fn founder(
    state: &mut PlateState,
    geo: &Geo,
    env: &Env,
    out: &mut Collapse,
    sea_m: f64,
) -> f64 {
    let k = env.k;
    if k.crust_max_km.is_nan() || k.crust_max_km <= 0.0 {
        return 0.0;
    }
    // ledger: PLT-258.
    let t_max = (k.crust_max_km * 1000.0).max(normal_thickness_m());
    if spreads(k) {
        // ledger: PLT-277 (the excess spreads before any of it founders).
        let mut by_plate: BTreeMap<u32, Vec<(usize, f64)>> = BTreeMap::new();
        // P1-19: found in parallel, kept in sample order.
        let over: Vec<usize> = (0..state.samples.len())
            .into_par_iter()
            .filter(|&i| {
                let s = &state.samples[i];
                s.kind == CONTINENTAL && !s.is_hidden() && f64::from(s.thickness_m) > t_max
            })
            .collect();
        for i in over {
            let s = &state.samples[i];
            by_plate
                .entry(state.plate_of(i))
                .or_default()
                .push((i, f64::from(s.thickness_m) - t_max));
        }
        // The excess goes to continental crust thinner than the collapse thickness (crust
        // thicker than that does not stay, PLT-256: lower crust flows out of the thickened belt
        // but does not build a second one around it) and to ocean floor up to the normal column
        // (from the datum or from the sea, PLT-412).
        let to = (k.collapse_thickness_km * 1000.0).clamp(normal_thickness_m(), t_max);
        if !by_plate.is_empty() {
            let ocean_to = room_column_m(k, sea_m).min(to);
            let to = RoomTo {
                continental_m: to,
                ocean_m: ocean_to,
            };
            spread(state, geo, env, by_plate, to, out, None);
        }
    }
    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
    let mut lost = 0.0;
    for s in state.samples.iter_mut() {
        if s.kind != CONTINENTAL || s.is_hidden() || f64::from(s.thickness_m) <= t_max {
            continue;
        }
        let left = f64::from(s.thickness_m) - t_max;
        let pools_before = s.pools;
        lost += sink(s, left);
        // Isostasy, as in the collapse: towards the height of the column kept, never past it (a
        // root that founders takes its eclogite with it, so the column may rise).
        let z = f64::from(s.elevation_m);
        let change = isostatic_change_m(&pools_before, &s.pools, env.gravity);
        s.elevation_m = toward(z, change, iso_height(&s.pools, z_0, env.gravity)) as f32;
    }
    lost * env.norm.sample_area_km2()
}

/// Takes `left` m of crust from the bottom of a column (mafic pools first, [`ROOT_ORDER`]) to the
/// mantle and returns what went, m; the caller moves the surface.
pub(super) fn sink(s: &mut Sample, mut left: f64) -> f64 {
    for &p in &ROOT_ORDER {
        let take = f64::from(s.pools[p]).min(left);
        s.pools[p] = (f64::from(s.pools[p]) - take) as f32;
        if p == pool::MAFIC_PLUTONIC || p == pool::MAFIC_VOLCANIC {
            // A basement (ocean crust under crust made continental) is the column's bottom.
            s.basement_m = (f64::from(s.basement_m) - take).max(0.0) as f32;
        }
        left -= take;
        if left <= 0.0 {
            break;
        }
    }
    let before = f64::from(s.thickness_m);
    s.thickness_m = s.pools.iter().sum();
    before - f64::from(s.thickness_m)
}

/// Moves a source's surface at `z` by `change` without raising it or taking it below
/// min(`z`, `floor`): crust taken off a column that this step thickened lowers it no further
/// than where the step began (`rate.rs`; a column the surface holds above its Airy height is
/// not dragged down to it).
pub(super) fn held(z: f64, change: f64, floor: f64) -> f64 {
    (z + change).clamp(z.min(floor), z)
}

/// A continental column's isostatic height, m: the middle of `continent_elevation_m` (`z_0`) for
/// the normal column, plus the Airy height of the thickness beyond it, less its eclogite root's
/// pull (PLT-281, PLT-278).
pub fn iso_height(pools: &[f32; pool::COUNT], z_0: f64, gravity: f64) -> f64 {
    let t: f64 = pools.iter().map(|&x| f64::from(x.max(0.0))).sum();
    z_0 + (t - normal_thickness_m()) / COLUMN_ISOSTATIC_FACTOR
        - root_drop_m(eclogite_m(pools, gravity))
}

/// Moves a surface at `z` by `change` towards the column's isostatic height `iso`, never past it:
/// a sinking column stops at it (a root that never raised its surface does not drag it down), a
/// rising one stops at it (thin crust the surface already holds high is not raised further).
pub(super) fn toward(z: f64, change: f64, iso: f64) -> f64 {
    if change < 0.0 {
        z + change.max(-(z - iso).max(0.0))
    } else {
        z + change.min((iso - z).max(0.0))
    }
}

/// The model's normal continental column, m: what a collapsing orogen thins towards and what it
/// fills its receivers up to (`pools::continental_column`, 35.1 km).
pub fn normal_thickness_m() -> f64 {
    continental_column().iter().map(|&x| f64::from(x)).sum()
}

/// The normal column the collapse uses under a sea `sea_m` over the datum, m: with
/// `collapse_sea_column` the column whose Airy height stands as high over the sea as the normal
/// column's stands over the datum, T_n + 7.0 S, kept within [T_n / 2, T_c] and under
/// `collapse_sea_column_max_km` when that is set; else T_n (see the module docs, "Measured from
/// the sea"). ledger: PLT-412, PLT-413
pub fn room_column_m(k: &crate::params::TectonicParams, sea_m: f64) -> f64 {
    let normal = normal_thickness_m();
    if !k.collapse_sea_column || !sea_m.is_finite() {
        return normal;
    }
    let mut top = (k.collapse_thickness_km * 1000.0).max(normal);
    if k.collapse_sea_column_max_km > 0.0 {
        // ledger: PLT-413 (the continents' mean crust, never under T_n).
        top = top.min((k.collapse_sea_column_max_km * 1000.0).max(normal));
    }
    // game-choice: the bounds are guards (a sea 2.1 km over the datum or 2.5 km under it), not
    // reached by the shipped presets' seas (about −300 to +1,200 m).
    (normal + COLUMN_ISOSTATIC_FACTOR * sea_m).clamp(0.5 * normal, top)
}

/// What one step did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Collapse {
    /// Samples that released crust.
    pub sources: u32,
    /// Crust moved, km² × m.
    pub moved_km2_m: f64,
    /// Ocean samples turned continental.
    pub converted: u32,
    /// Crust the collapse would have released from columns an orogeny still holds up (the quiet
    /// time), km² × m (measurement: the crust budget).
    pub held_km2_m: f64,
    /// Crust released that found no room within reach and stayed, km² × m (measurement).
    pub unplaced_km2_m: f64,
}

/// One step of the collapse on `state` (see the module docs) under the step's sea `sea_m` (m
/// over the datum; read only with `collapse_sea_column`). Runs after the step's uplift
/// (collision belts and subduction), so a sample raised this step is known.
pub fn step(state: &mut PlateState, geo: &Geo, env: &Env, sea_m: f64) -> Collapse {
    let k = env.k;
    if !spreads(k) {
        return Collapse::default();
    }
    let rate = 1.0 - (-env.dt_myr / k.collapse_myr).exp();
    // The normal column, from the datum or from the sea (ledger: PLT-412).
    let normal = room_column_m(k, sea_m);
    let t_c = (k.collapse_thickness_km * 1000.0).max(normal);
    // A sample raised by an orogeny carries the time of the step that raised it; it is held up
    // until no orogeny has raised it for `collapse_quiet_myr` (at least this step). ledger: PLT-259
    let quiet_before = (env.t_myr - k.collapse_quiet_myr.max(0.5 * env.dt_myr)) as f32;
    // A column an orogeny raised after this is still hot: it thins towards the normal column, not
    // only to T_c (post-orogenic collapse, ledger: PLT-390).
    let hot_after = if k.collapse_hot_myr > 0.0 {
        (env.t_myr - k.collapse_hot_myr) as f32
    } else {
        f32::INFINITY
    };
    // What a column thins towards: the normal column while it is hot, T_c otherwise.
    let to = |s: &Sample| {
        if s.orogeny != OROGENY_NONE && s.orogeny_myr > hot_after {
            normal
        } else {
            t_c
        }
    };
    // What a column releases: the threshold law's, or the thin sheet's where it is larger
    // (ledger: PLT-254, PLT-255, PLT-256, PLT-390; PLT-414, PLT-415).
    let sheet = Sheet::new(k, env, normal, sea_m);
    let release_of = |s: &Sample| {
        let q = rate * (f64::from(s.thickness_m) - to(s));
        sheet.as_ref().map_or(q, |sh| q.max(sh.release(s)))
    };

    // Each plate's sources and what they would release this step, m of thickness.
    let mut by_plate: BTreeMap<u32, Vec<(usize, f64)>> = BTreeMap::new();
    // P1-19: found in parallel, kept in sample order (and summed in it: a parallel sum's order,
    // so its last bits, would change from run to run).
    let held: f64 = state
        .samples
        .par_iter()
        .filter(|s| s.kind == CONTINENTAL && !s.is_hidden() && s.orogeny_myr >= quiet_before)
        .map(|s| release_of(s).max(0.0))
        .collect::<Vec<f64>>()
        .iter()
        .sum();
    let release: Vec<(usize, f64)> = state
        .samples
        .par_iter()
        .enumerate()
        .filter_map(|(i, s)| {
            if s.kind != CONTINENTAL || s.is_hidden() || s.orogeny_myr >= quiet_before {
                return None;
            }
            let q = release_of(s);
            (q > 0.0).then_some((i, q))
        })
        .collect();
    let mut budgets: BTreeMap<u32, f64> = BTreeMap::new();
    for (i, q) in release {
        let p = state.plate_of(i);
        *budgets.entry(p).or_insert(0.0) += q;
        by_plate.entry(p).or_default().push((i, q));
    }
    let area = env.norm.sample_area_km2();
    let mut out = Collapse {
        held_km2_m: held * area,
        ..Collapse::default()
    };
    let to = RoomTo {
        continental_m: normal,
        ocean_m: normal,
    };
    let shares = spread(state, geo, env, by_plate, to, &mut out, None);
    out.unplaced_km2_m = budgets
        .iter()
        .map(|(p, b)| b * (1.0 - shares.get(p).copied().unwrap_or(0.0)))
        .sum::<f64>()
        * area;
    out
}

/// The collapse as a thin viscous sheet for one step (see the module docs, "As a thin viscous
/// sheet"; ledger PLT-414, PLT-415).
#[derive(Clone, Copy, Debug)]
pub(super) struct Sheet {
    /// η_0, Pa s: the viscosity of a column whose orogeny is this step's …
    eta_pa_s: f64,
    /// … growing e-fold per this many Myr of the time since (0: no growth).
    strength_myr: f64,
    /// The step's time and length, Myr.
    t_myr: f64,
    dt_myr: f64,
    /// The column it thins towards, m, and that column's potential energy, N/m.
    normal_m: f64,
    u_normal: f64,
    /// The step's sea over the datum, m (the datum if unknown), the normal column's height there
    /// (`continent_elevation_m`'s middle) and gravity.
    sea_m: f64,
    z_0: f64,
    gravity: f64,
}

impl Sheet {
    /// The sheet for a step towards the normal column `normal_m` under a sea `sea_m`; `None` with
    /// `collapse_sheet_viscosity_pa_s` 0 (off).
    pub(super) fn new(
        k: &crate::params::TectonicParams,
        env: &Env,
        normal_m: f64,
        sea_m: f64,
    ) -> Option<Sheet> {
        let eta_pa_s = k.collapse_sheet_viscosity_pa_s; // ledger: PLT-414
        if !(eta_pa_s > 0.0 && eta_pa_s.is_finite()) {
            return None;
        }
        let sea_m = if sea_m.is_finite() { sea_m } else { 0.0 };
        let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
        let z_n = z_0 + (normal_m - normal_thickness_m()) / COLUMN_ISOSTATIC_FACTOR;
        Some(Sheet {
            eta_pa_s,
            strength_myr: k.collapse_sheet_strength_myr, // ledger: PLT-415
            t_myr: env.t_myr,
            dt_myr: env.dt_myr,
            normal_m,
            u_normal: column_energy(z_n, true, 0.0, 0.0, sea_m, env.gravity), // ledger: FRC-014
            sea_m,
            z_0,
            gravity: env.gravity,
        })
    }

    /// The column's thinning rate, per Myr: ε̇ = ΔU / (4 η L) for plane strain in a sheet L thick
    /// (FRC-010), ΔU its potential energy at its Airy height over the normal column's; 0 for
    /// crust no orogeny raised or no higher than the normal column.
    pub(super) fn rate_per_myr(&self, s: &Sample) -> f64 {
        if s.orogeny == OROGENY_NONE {
            return 0.0;
        }
        let z = iso_height(&s.pools, self.z_0, self.gravity);
        let du = column_energy(z, true, 0.0, 0.0, self.sea_m, self.gravity) - self.u_normal;
        if !(du > 0.0) {
            return 0.0;
        }
        let age = (self.t_myr - f64::from(s.orogeny_myr)).max(0.0);
        // ledger: PLT-415 (the lithosphere stiffens as it cools; the form is the model's)
        let eta = if self.strength_myr > 0.0 {
            self.eta_pa_s * (age / self.strength_myr).exp()
        } else {
            self.eta_pa_s
        };
        // ledger: PLT-414, FRC-010
        du / (4.0 * eta * LITHOSPHERE_KM * 1000.0) * YEAR_S * 1.0e6
    }

    /// What the column releases this step, m of thickness: T (1 − e^(−ε̇ δt)), at most down to
    /// the normal column.
    pub(super) fn release(&self, s: &Sample) -> f64 {
        let t = f64::from(s.thickness_m);
        if t <= self.normal_m {
            return 0.0;
        }
        let e = self.rate_per_myr(s);
        if e <= 0.0 {
            return 0.0;
        }
        (t * (1.0 - (-e * self.dt_myr).exp())).min(t - self.normal_m)
    }
}

/// One plate's release plan (`spread`): its budget, what is left of it and the receivers' shares.
type Plan = (f64, f64, Vec<(usize, f64)>);

/// Whether the collapse's settings let crust spread at all.
fn spreads(k: &crate::params::TectonicParams) -> bool {
    !(k.collapse_myr.is_nan()
        || k.collapse_myr <= 0.0
        || !k.collapse_reach_km.is_finite()
        || k.collapse_reach_km <= 0.0)
}

/// How thick [`spread`] fills its room, m: continental crust up to `continental_m`, ocean floor up
/// to `ocean_m` (continental once filled to it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RoomTo {
    pub continental_m: f64,
    pub ocean_m: f64,
}

/// Moves crust from each plate's `sources` (sample, m of thickness to release) to the nearest
/// room on the same plate within `collapse_reach_km` (see the module docs, "The rule": continental
/// crust thinner than `continental_to_m`, filled up to it, and ocean floor, filled up to
/// `ocean_to_m`, the normal column, and continental once filled; during a plate step never above
/// a receiver's thickening ceiling, `rate.rs`),
/// thinning the sources in proportion to what found room; returns the share of each plate's
/// release that was placed (sources of a plate whose release found no room keep their crust).
/// A source's surface sinks by Airy towards its column's height, or, given `floor`, as
/// [`held`] moves it.
pub(super) fn spread(
    state: &mut PlateState,
    geo: &Geo,
    env: &Env,
    by_plate: BTreeMap<u32, Vec<(usize, f64)>>,
    to: RoomTo,
    out: &mut Collapse,
    floor: Option<&[f32]>,
) -> BTreeMap<u32, f64> {
    let RoomTo {
        continental_m: continental_to_m,
        ocean_m: ocean_to_m,
    } = to;
    let k = env.k;
    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
    let mut shares = BTreeMap::new();
    // At least to the next ring of samples: on a coarse lattice the reach may be shorter than a
    // sample spacing, and a column always borders its neighbours.
    let reach = env
        .norm
        .rad(k.collapse_reach_km)
        .max(REACH_MIN_SPACINGS * env.norm.spacing_rad());
    // P1-19: where each plate's release goes is found for every plate at once (a plate's search
    // reads only its own samples, which no other plate's moves touch), then the moves are made
    // plate by plate in id order as before.
    let plates: Vec<(u32, Vec<(usize, f64)>)> = by_plate.into_iter().collect();
    let samples = &state.samples;
    let groups = &state.groups;
    let ceiling = state.limits.as_ref().map(|l| l.ceiling_m.as_slice());
    let plans: Vec<Option<Plan>> = plates
        .par_iter()
        .map(|(p, sources)| {
            let p = *p;
            let budget: f64 = sources.iter().map(|&(_, q)| q).sum();
            if budget <= 0.0 {
                return None;
            }
            let src: Vec<(u32, f64, u32)> =
                sources.iter().map(|&(i, _)| (i as u32, 0.0, 0)).collect();
            let on_plate = |v: u32| {
                let s = &samples[v as usize];
                groups[s.group as usize].plate == p && !s.is_hidden()
            };
            let reached = dijkstra_sparse(geo, &src, |_, b| on_plate(b), reach);
            // The room, nearest first, not at a trench: continental crust thinner than
            // `continental_to_m`, ocean floor up to `ocean_to_m`, neither above its ceiling.
            let mut room: Vec<(f64, usize, f64)> = reached
                .iter()
                .filter(|&&(j, _, _)| on_plate(j))
                .filter_map(|&(j, d, _)| {
                    let j = j as usize;
                    let s = &samples[j];
                    let to = if s.kind == CONTINENTAL {
                        continental_to_m
                    } else {
                        ocean_to_m
                    };
                    let cap = ceiling
                        .and_then(|c| c.get(j))
                        .map_or(f64::INFINITY, |&c| f64::from(c));
                    let r = to.min(cap) - f64::from(s.thickness_m);
                    (r > 0.0 && s.flags & TRENCH == 0).then_some((d, j, r))
                })
                .collect();
            // Continental room before ocean floor when the crust spreads within its continent
            // first (ledger: PLT-391), else the nearest first.
            if k.collapse_continent_first {
                room.sort_by(|a, b| {
                    (samples[a.1].kind != CONTINENTAL)
                        .cmp(&(samples[b.1].kind != CONTINENTAL))
                        .then(a.0.total_cmp(&b.0))
                        .then(a.1.cmp(&b.1))
                });
            } else {
                room.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            }
            let mut left = budget;
            let mut given: Vec<(usize, f64)> = Vec::new();
            for (_, j, r) in room {
                if left <= 0.0 {
                    break;
                }
                let g = r.min(left);
                given.push((j, g));
                left -= g;
            }
            Some((budget, left, given))
        })
        .collect();
    for ((p, sources), plan) in plates.into_iter().zip(plans) {
        let Some((budget, left, given)) = plan else {
            continue;
        };
        let placed = budget - left.max(0.0);
        if placed <= 0.0 {
            shares.insert(p, 0.0);
            continue;
        }
        let share = placed / budget;
        shares.insert(p, share);

        // Thin the sources in proportion (pure shear) and pool what they gave.
        let mut mix = [0.0f64; pool::COUNT];
        for &(i, q) in &sources {
            let s = &mut state.samples[i];
            let take = q * share;
            let t = f64::from(s.thickness_m);
            let keep = ((t - take) / t).max(0.0);
            let pools_before = s.pools;
            for (m, x) in mix.iter_mut().zip(s.pools.iter_mut()) {
                let before = f64::from(*x);
                *x = (before * keep) as f32;
                *m += before - f64::from(*x);
            }
            s.basement_m *= keep as f32;
            s.thickness_m = s.pools.iter().sum();
            // Isostasy (PLT-281, PLT-278): down by take / 7.0 less the root's change, not below
            // the height the crust kept holds up (or the floor).
            let z = f64::from(s.elevation_m);
            let change = isostatic_change_m(&pools_before, &s.pools, env.gravity);
            s.elevation_m = match floor {
                Some(f) => held(z, change, f64::from(f[i])),
                None => toward(z, change, iso_height(&s.pools, z_0, env.gravity)),
            } as f32;
            out.sources += 1;
        }
        let total: f64 = mix.iter().sum();
        if total <= 0.0 {
            continue;
        }
        for m in &mut mix {
            *m /= total;
        }
        // Fill the receivers; an ocean sample filled to the normal column (`ocean_to_m`) is
        // continent now.
        for (j, g) in given {
            let s = &mut state.samples[j];
            let pools_before = s.pools;
            for (x, m) in s.pools.iter_mut().zip(mix.iter()) {
                *x += (g * m) as f32;
            }
            s.thickness_m = s.pools.iter().sum();
            // Isostasy (PLT-281, PLT-278): up by g / 7.0 less the root's change, not above the
            // height the crust now holds up.
            let z = f64::from(s.elevation_m);
            let change = isostatic_change_m(&pools_before, &s.pools, env.gravity);
            s.elevation_m = toward(z, change, iso_height(&s.pools, z_0, env.gravity)) as f32;
            if s.kind == OCEANIC && f64::from(s.thickness_m) >= ocean_to_m - 1.0 {
                // The floor's own crust stays under the crust spread onto it (ledger: PLT-331).
                s.make_continental();
                out.converted += 1;
            }
        }
        out.moved_km2_m += placed * env.norm.sample_area_km2();
    }
    shares
}

#[cfg(test)]
pub(in crate::plates) mod tests {
    use glam::DVec3;

    use super::*;
    use crate::params::TectonicParams;
    use crate::plates::geometry::positions;
    use crate::plates::normalise::Norm;
    use crate::plates::state::{Group, OROGENY_HIMALAYAN, Plate, Sample};

    const N: usize = 16_000;

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

    /// Plate 0 is x < 0, plate 1 x ≥ 0. A continent (35.1 km, 300 m) within 40° of +y on both,
    /// ocean floor at −4.5 km elsewhere; on the continent a belt (`belt(p)`) thickened by
    /// `thick_m` and raised by `thick_m` / 7.0.
    pub(in crate::plates) fn planet(
        geo: &Geo,
        belt: impl Fn(DVec3) -> bool,
        thick_m: f32,
    ) -> PlateState {
        let samples: Vec<Sample> = geo
            .points()
            .iter()
            .map(|&p| {
                let mut s = Sample::blank(u32::from(p.x >= 0.0));
                if p.y > 0.766 {
                    s.kind = CONTINENTAL;
                    s.pools = continental_column();
                    s.elevation_m = 300.0;
                    if belt(p) {
                        s.pools[pool::METAMORPHIC] += thick_m;
                        s.elevation_m += thick_m / COLUMN_ISOSTATIC_FACTOR as f32;
                    }
                } else {
                    s.pools[pool::MAFIC_VOLCANIC] = 2000.0;
                    s.pools[pool::MAFIC_PLUTONIC] = 5000.0;
                    s.elevation_m = -4500.0;
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

    pub(in crate::plates) struct Fx {
        pub(in crate::plates) k: TectonicParams,
        pub(in crate::plates) geo: Geo,
        pub(in crate::plates) norm: Norm,
    }

    impl Fx {
        pub(in crate::plates) fn new() -> Fx {
            let k = TectonicParams {
                collapse_myr: 25.0,
                collapse_thickness_km: 45.0,
                collapse_quiet_myr: 0.0,
                collapse_reach_km: 1500.0,
                ..TectonicParams::default()
            };
            Fx {
                k,
                geo: Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap(),
                norm: Norm::new(6370.0, 6371.0, N),
            }
        }

        pub(in crate::plates) fn env(&self, step: u64) -> Env<'_> {
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
    }

    fn continental(st: &PlateState, plate: u32) -> usize {
        (0..st.samples.len())
            .filter(|&i| st.plate_of(i) == plate && st.samples[i].kind == CONTINENTAL)
            .count()
    }

    pub(in crate::plates) fn crust_m(st: &PlateState) -> f64 {
        st.samples.iter().map(|s| f64::from(s.thickness_m)).sum()
    }

    /// A quiet belt 5 km high on plate 0, at the continent's edge: it thins towards the collapse
    /// thickness and sinks towards that thickness's Airy height, the crust it loses goes to the
    /// plate's own ocean floor beside the continent, which turns continental, and the volume is
    /// kept. Plate 1, with no belt, does not change, and
    /// nothing turns continental beyond the reach.
    #[test]
    fn a_quiet_belt_collapses_and_its_continent_grows() {
        let fx = Fx::new();
        // A belt along the continent's edge on plate 0: 40°–50° from +y, x < 0.
        let belt = |p: DVec3| p.x < -0.1 && p.y > 0.766 && p.y < 0.85;
        let mut st = planet(&fx.geo, belt, 26_000.0);
        let z0: Vec<f32> = st.samples.iter().map(|s| s.elevation_m).collect();
        let kind0: Vec<u8> = st.samples.iter().map(|s| s.kind).collect();
        let (c0, c1) = (continental(&st, 0), continental(&st, 1));
        let crust0 = crust_m(&st);
        let mut converted = 0;
        for step in 1..=40 {
            let c = step_once(&fx, &mut st, step);
            converted += c.converted;
        }
        assert!(
            (crust_m(&st) - crust0).abs() < 1e-6 * crust0,
            "volume kept: {} vs {crust0}",
            crust_m(&st)
        );
        let grown = continental(&st, 0) - c0;
        assert!(
            grown > 20 && grown as u32 == converted,
            "{grown} {converted}"
        );
        assert_eq!(continental(&st, 1), c1, "plate 1 has no belt");
        // The belt thinned towards the collapse thickness, never below it, and sank towards its
        // Airy height.
        let t_c = fx.k.collapse_thickness_km * 1000.0;
        let airy = 0.5 * (fx.k.continent_elevation_m[0] + fx.k.continent_elevation_m[1])
            + (t_c - normal_thickness_m()) / COLUMN_ISOSTATIC_FACTOR;
        let in_belt: Vec<usize> = (0..st.samples.len())
            .filter(|&i| belt(fx.geo.points()[i]))
            .collect();
        let top = in_belt
            .iter()
            .map(|&i| f64::from(st.samples[i].elevation_m))
            .fold(f64::MIN, f64::max);
        assert!(top < airy + 300.0 && top >= airy - 1.0, "{top} vs {airy}");
        for &i in &in_belt {
            let t = f64::from(st.samples[i].thickness_m);
            assert!(t >= t_c - 1.0 && t < t_c + 1500.0, "{t}");
        }
        // New continent is on plate 0's ocean floor, within the reach of the belt.
        let reach = fx.norm.rad(fx.k.collapse_reach_km);
        for (i, s) in st.samples.iter().enumerate() {
            if kind0[i] == OCEANIC && s.kind == CONTINENTAL {
                assert_eq!(st.plate_of(i), 0);
                let p = fx.geo.points()[i];
                // Angular distance to the belt's band (y = 0.85 is 31.8° from +y).
                assert!(p.y.acos() - 0.766f64.acos() < reach, "{p}");
                // Raised to a continent's height, not above the Airy height of its column.
                let z_a = 0.5 * (fx.k.continent_elevation_m[0] + fx.k.continent_elevation_m[1])
                    + (f64::from(s.thickness_m) - normal_thickness_m()) / COLUMN_ISOSTATIC_FACTOR;
                assert!(s.elevation_m > z0[i] && f64::from(s.elevation_m) <= z_a + 1.0);
            }
        }
    }

    /// What a step moved, without its measurements of the crust it held or could not place.
    fn moves(c: Collapse) -> Collapse {
        Collapse {
            held_km2_m: 0.0,
            unplaced_km2_m: 0.0,
            ..c
        }
    }

    /// One collapse step on fixed positions, under a sea at the datum.
    fn step_once(fx: &Fx, st: &mut PlateState, n: u64) -> Collapse {
        step_under(fx, st, n, 0.0)
    }

    /// One collapse step on fixed positions under a sea `sea_m` over the datum.
    fn step_under(fx: &Fx, st: &mut PlateState, n: u64, sea_m: f64) -> Collapse {
        let env = fx.env(n);
        step(st, &fx.geo, &env, sea_m)
    }

    /// A belt raised by an orogeny this step is held up by the convergence; a root that never
    /// raised its surface (an overlap stack) spreads and leaves its surface where it was; a high
    /// swell with normal crust has nothing to give; with no room within reach nothing is
    /// released; with `collapse_myr` 0 nothing happens.
    #[test]
    fn only_quiet_thickened_crust_with_room_collapses() {
        let mut fx = Fx::new();
        let belt = |p: DVec3| p.x < -0.1 && p.y > 0.766 && p.y < 0.85;
        // Active: stamped this step.
        let mut st = planet(&fx.geo, belt, 26_000.0);
        for s in st.samples.iter_mut() {
            if s.thickness_m > 40_000.0 {
                s.orogeny = OROGENY_HIMALAYAN;
                s.orogeny_myr = 2.0;
            }
        }
        let before = st.samples.clone();
        let c = step_once(&fx, &mut st, 1);
        assert_eq!(moves(c), Collapse::default());
        assert!(c.held_km2_m > 0.0, "the held crust is counted: {c:?}");
        assert_eq!(st.samples, before);
        // The same belt stamped last step collapses now.
        assert!(step_once(&fx, &mut st.clone(), 2).sources > 0);
        // With a quiet time of 10 Myr it waits until the stamp is more than 10 Myr old.
        let mut quiet = Fx::new();
        quiet.k.collapse_quiet_myr = 10.0;
        for step in 2..=6 {
            assert_eq!(
                moves(step_once(&quiet, &mut st, step)),
                Collapse::default(),
                "{step}"
            );
        }
        assert!(step_once(&quiet, &mut st, 7).sources > 0);
        // A hidden root: 30 km more crust under ground at 300 m.
        let mut st = planet(&fx.geo, belt, 0.0);
        for (i, s) in st.samples.iter_mut().enumerate() {
            if belt(fx.geo.points()[i]) {
                s.pools[pool::FELSIC_PLUTONIC] += 30_000.0;
                s.thickness_m += 30_000.0;
            }
        }
        let before = st.samples.clone();
        let c = step_once(&fx, &mut st, 1);
        assert!(c.sources > 0 && c.moved_km2_m > 0.0, "{c:?}");
        for (i, s) in st.samples.iter().enumerate() {
            if belt(fx.geo.points()[i]) {
                assert!(s.thickness_m < before[i].thickness_m);
                assert_eq!(s.elevation_m, 300.0);
            }
        }
        // A swell: high, normal crust.
        let mut st = planet(&fx.geo, belt, 0.0);
        for (i, s) in st.samples.iter_mut().enumerate() {
            if belt(fx.geo.points()[i]) {
                s.elevation_m = 3000.0;
            }
        }
        assert_eq!(step_once(&fx, &mut st, 1), Collapse::default());
        // No room within reach: a belt in the middle of a continent, reach one sample.
        fx.k.collapse_reach_km = 50.0;
        let middle = |p: DVec3| p.x < -0.02 && p.y > 0.95;
        let mut st = planet(&fx.geo, middle, 26_000.0);
        let before = st.samples.clone();
        let c = step_once(&fx, &mut st, 1);
        assert_eq!(moves(c), Collapse::default());
        assert!(
            c.unplaced_km2_m > 0.0,
            "the crust with no room is counted: {c:?}"
        );
        assert_eq!(st.samples, before);
        // Off.
        fx.k.collapse_reach_km = 1500.0;
        fx.k.collapse_myr = 0.0;
        let mut st = planet(&fx.geo, belt, 26_000.0);
        assert_eq!(step_once(&fx, &mut st, 1), Collapse::default());
    }

    /// A belt whose last orogeny is younger than `collapse_hot_myr` thins past the collapse
    /// thickness towards the normal column; with no window, or with no orogeny stamp, it stops at
    /// the collapse thickness; once the window has passed, crust thinner than the collapse
    /// thickness keeps what it has (ledger: PLT-390).
    #[test]
    fn a_hot_belt_thins_to_the_normal_column() {
        let belt = |p: DVec3| p.x < -0.1 && p.y > 0.766 && p.y < 0.85;
        let t_c = Fx::new().k.collapse_thickness_km * 1000.0;
        // The belt's thinnest and thickest columns after `steps` steps from `first`.
        let span = |fx: &Fx, st: &mut PlateState, first: u64, last: u64| {
            let crust0 = crust_m(st);
            for step in first..=last {
                step_once(fx, st, step);
            }
            assert!((crust_m(st) - crust0).abs() < 1e-6 * crust0, "volume kept");
            let t: Vec<f64> = (0..st.samples.len())
                .filter(|&i| belt(fx.geo.points()[i]))
                .map(|i| f64::from(st.samples[i].thickness_m))
                .collect();
            let lo = t.iter().copied().fold(f64::MAX, f64::min);
            let hi = t.iter().copied().fold(f64::MIN, f64::max);
            (lo, hi)
        };
        let stamped = |fx: &Fx| {
            let mut st = planet(&fx.geo, belt, 26_000.0);
            for s in st.samples.iter_mut().filter(|s| s.thickness_m > 40_000.0) {
                s.orogeny = OROGENY_HIMALAYAN;
                s.orogeny_myr = 0.0;
            }
            st
        };
        // Hot: 19 steps of 2 Myr after an orogeny at 0 are all inside a 40 Myr window.
        let mut hot = Fx::new();
        hot.k.collapse_hot_myr = 40.0;
        let mut st = stamped(&hot);
        let (lo, hi) = span(&hot, &mut st, 1, 19);
        assert!(
            lo >= normal_thickness_m() - 1.0 && hi < t_c - 2000.0,
            "{lo} {hi}"
        );
        // The window has passed: nothing thinner than T_c moves.
        let before = st.samples.clone();
        let (lo2, hi2) = span(&hot, &mut st, 20, 30);
        assert!(
            (lo2 - lo).abs() < 1e-3 && (hi2 - hi).abs() < 1e-3,
            "{lo2} {hi2}"
        );
        assert_eq!(st.samples, before);
        // No window: the same belt stops at T_c.
        let cold = Fx::new();
        let (lo, _) = span(&cold, &mut stamped(&cold), 1, 19);
        assert!(lo >= t_c - 1.0, "{lo}");
        // A window but no orogeny stamp (a stack under ground no orogeny raised): T_c as well.
        let (lo, _) = span(&hot, &mut planet(&hot.geo, belt, 26_000.0), 1, 19);
        assert!(lo >= t_c - 1.0, "{lo}");
    }

    /// The normal column from the sea (PLT-412): off, T_n whatever the sea; on, T_n + 7.0 S, within
    /// [T_n / 2, T_c] and under the cap (PLT-413), and T_n when the sea is unknown.
    #[test]
    fn the_room_column_stands_over_the_sea_with_the_switch() {
        let mut k = TectonicParams::default();
        let t_n = normal_thickness_m();
        let t_c = k.collapse_thickness_km * 1000.0;
        for sea in [-300.0, 0.0, 956.0, 5000.0] {
            assert_eq!(room_column_m(&k, sea), t_n);
        }
        k.collapse_sea_column = true;
        assert_eq!(room_column_m(&k, 0.0), t_n);
        let f = COLUMN_ISOSTATIC_FACTOR;
        assert!((room_column_m(&k, 956.0) - (t_n + f * 956.0)).abs() < 1e-6);
        assert!((room_column_m(&k, 956.0) - 41_800.0).abs() < 100.0);
        assert!((room_column_m(&k, -300.0) - (t_n - f * 300.0)).abs() < 1e-6);
        assert_eq!(room_column_m(&k, 5000.0), t_c);
        assert_eq!(room_column_m(&k, -5000.0), 0.5 * t_n);
        assert_eq!(room_column_m(&k, f64::NAN), t_n);
        // The cap (PLT-413): the continents' mean crust bounds the column, never under T_n.
        k.collapse_sea_column_max_km = 41.1;
        assert!((room_column_m(&k, 1236.0) - 41_100.0).abs() < 1e-6);
        assert!((room_column_m(&k, 500.0) - (t_n + f * 500.0)).abs() < 1e-6);
        k.collapse_sea_column_max_km = 20.0;
        assert_eq!(room_column_m(&k, 956.0), t_n);
    }

    /// With `collapse_sea_column` under a sea 956 m over the datum, a hot belt thins towards the
    /// column that stands over the sea as T_n stands over the datum (41.8 km), not to T_n, and the
    /// ocean floor it fills turns continental only at that column; the volume is kept. Off, the
    /// sea is not read: the run is the datum's, bit for bit (ledger: PLT-412).
    #[test]
    fn a_hot_belt_thins_to_the_column_over_the_sea() {
        let belt = |p: DVec3| p.x < -0.1 && p.y > 0.766 && p.y < 0.85;
        let sea = 956.0;
        let run = |on: bool, sea_m: f64| {
            let mut fx = Fx::new();
            fx.k.collapse_hot_myr = 40.0;
            fx.k.collapse_sea_column = on;
            let mut st = planet(&fx.geo, belt, 26_000.0);
            for s in st.samples.iter_mut().filter(|s| s.thickness_m > 40_000.0) {
                s.orogeny = OROGENY_HIMALAYAN;
                s.orogeny_myr = 0.0;
            }
            let kind0: Vec<u8> = st.samples.iter().map(|s| s.kind).collect();
            let crust0 = crust_m(&st);
            let mut converted = 0;
            for step in 1..=19 {
                converted += step_under(&fx, &mut st, step, sea_m).converted;
            }
            assert!((crust_m(&st) - crust0).abs() < 1e-6 * crust0, "volume kept");
            let column = room_column_m(&fx.k, sea_m);
            let lo = (0..st.samples.len())
                .filter(|&i| belt(fx.geo.points()[i]))
                .map(|i| f64::from(st.samples[i].thickness_m))
                .fold(f64::MAX, f64::min);
            // Every ocean sample turned continental carries the column it was filled to.
            let thinnest_new = (0..st.samples.len())
                .filter(|&i| kind0[i] == OCEANIC && st.samples[i].kind == CONTINENTAL)
                .map(|i| f64::from(st.samples[i].thickness_m))
                .fold(f64::MAX, f64::min);
            (st, column, lo, thinnest_new, converted)
        };
        let (_, column, lo, thinnest_new, converted) = run(true, sea);
        let t_n = normal_thickness_m();
        assert!(column > t_n + 6000.0, "{column}");
        assert!(lo >= column - 1.0, "{lo} vs {column}");
        assert!(
            converted > 0 && thinnest_new >= column - 1.0,
            "{converted} {thinnest_new}"
        );
        // On, under a sea at the datum: T_n, as with the switch off. In 38 Myr (τ 25) the belt
        // loses 78 % of its excess either way, so it ends about 0.78 × 7.0 S thicker under the sea.
        let (_, column0, lo0, _, _) = run(true, 0.0);
        assert!((column0 - t_n).abs() < 1e-9 && lo0 >= t_n - 1.0, "{lo0}");
        assert!(lo - lo0 > 4000.0, "{lo} {lo0}");
        // Off: the sea is not read.
        let (a, ..) = run(false, sea);
        let (b, ..) = run(false, 0.0);
        assert_eq!(a.samples, b.samples);
    }

    /// The thin sheet's rate (PLT-414, PLT-415): a column 26 km over T_n (4.0 km high) under a sea
    /// at the datum, its orogeny this step, thins at ΔU / (4 η L) = 3.6 % per Myr with η 10²²
    /// Pa s; the rate falls e-fold per `collapse_sheet_strength_myr` of the time since its
    /// orogeny, and is 0 for crust no orogeny raised and for the normal column; the release stops
    /// at the normal column; off, there is no sheet.
    #[test]
    fn the_sheet_thins_young_high_crust_at_its_potential_energy() {
        let mut fx = Fx::new();
        assert!(Sheet::new(&fx.k, &fx.env(0), normal_thickness_m(), 0.0).is_none());
        fx.k.collapse_sheet_viscosity_pa_s = 1.0e22;
        let env = fx.env(0);
        let t_n = normal_thickness_m();
        let sheet = Sheet::new(&fx.k, &env, t_n, 0.0).unwrap();
        let mut s = Sample::blank(0);
        s.kind = CONTINENTAL;
        s.pools = continental_column();
        s.pools[pool::METAMORPHIC] += 26_000.0;
        s.thickness_m = s.pools.iter().sum();
        s.orogeny = OROGENY_HIMALAYAN;
        s.orogeny_myr = 0.0;
        // By hand: z = 525 + 26,000 / 7.02 = 4,229 m less its eclogite root's pull (PLT-281),
        // 255 m: 3,975 m; U = ρ g z (35 km + 3 z) = 5.03e12 N/m, the normal column's (525 m)
        // 5.18e11; ε̇ = 4.51e12 / (4 × 1e22 × 1e5 m) = 1.13e-15 / s.
        let z = iso_height(&s.pools, 525.0, 9.81);
        assert!((z - 3975.0).abs() < 5.0, "{z}");
        let e = sheet.rate_per_myr(&s);
        assert!((e - 0.0356).abs() < 0.0005, "{e}");
        let q = sheet.release(&s);
        let t = f64::from(s.thickness_m);
        assert!((q - t * (1.0 - (-2.0 * e).exp())).abs() < 1e-6, "{q}");
        // 62.8 Myr after its orogeny: e-fold slower.
        s.orogeny_myr = -62.8;
        let older = sheet.rate_per_myr(&s);
        assert!((older * std::f64::consts::E / e - 1.0).abs() < 1e-3, "{older}");
        // No orogeny: rigid. The normal column: nothing to give.
        s.orogeny = OROGENY_NONE;
        assert_eq!(sheet.release(&s), 0.0);
        s.orogeny = OROGENY_HIMALAYAN;
        let mut flat = s;
        flat.pools = continental_column();
        flat.thickness_m = flat.pools.iter().sum();
        assert_eq!(sheet.release(&flat), 0.0);
        // A long step stops at the normal column.
        let mut fx_long = Fx::new();
        fx_long.k.collapse_sheet_viscosity_pa_s = 1.0e18;
        let long = Sheet::new(&fx_long.k, &fx_long.env(0), t_n, 0.0).unwrap();
        s.orogeny_myr = 0.0;
        assert!((long.release(&s) - (t - t_n)).abs() < 1e-6);
    }

    /// With the sheet on, a quiet belt raised by an orogeny thins past T_c towards the normal
    /// column, keeping the volume; a belt no orogeny raised, or one whose orogeny is 600 Myr old,
    /// stops at T_c as the threshold law has it (ledger: PLT-414, PLT-415).
    #[test]
    fn with_the_sheet_a_young_belt_thins_past_the_collapse_thickness() {
        let belt = |p: DVec3| p.x < -0.1 && p.y > 0.766 && p.y < 0.85;
        let t_c = Fx::new().k.collapse_thickness_km * 1000.0;
        let run = |stamp: Option<f32>| {
            let mut fx = Fx::new();
            fx.k.collapse_sheet_viscosity_pa_s = 1.0e22;
            let mut st = planet(&fx.geo, belt, 26_000.0);
            if let Some(at) = stamp {
                for s in st.samples.iter_mut().filter(|s| s.thickness_m > 40_000.0) {
                    s.orogeny = OROGENY_HIMALAYAN;
                    s.orogeny_myr = at;
                }
            }
            let crust0 = crust_m(&st);
            for step in 1..=30 {
                step_once(&fx, &mut st, step);
            }
            assert!((crust_m(&st) - crust0).abs() < 1e-6 * crust0, "volume kept");
            (0..st.samples.len())
                .filter(|&i| belt(fx.geo.points()[i]))
                .map(|i| f64::from(st.samples[i].thickness_m))
                .fold(f64::MAX, f64::min)
        };
        let young = run(Some(0.0));
        assert!(
            young >= normal_thickness_m() - 1.0 && young < t_c - 3000.0,
            "{young}"
        );
        let none = run(None);
        assert!(none >= t_c - 1.0, "{none}");
        let old = run(Some(-600.0));
        assert!(old >= t_c - 50.0 && (old - none).abs() < 50.0, "{old} {none}");
    }

    /// With `collapse_continent_first` the crust a belt at the continent's edge releases goes to
    /// the plate's thinned continental crust within reach before the ocean floor beside it, so no
    /// ocean turns continental while that room lasts; nearest first, the ocean beside the belt
    /// takes it (ledger: PLT-391).
    #[test]
    fn continental_room_first_fills_the_continent_before_the_ocean() {
        let belt = |p: DVec3| p.x < -0.1 && p.y > 0.766 && p.y < 0.85;
        // Continental crust thinned to 30 km in the continent's interior on plate 0.
        let thin = |p: DVec3| p.x < 0.0 && p.y > 0.9;
        let run = |first: bool| {
            let mut fx = Fx::new();
            fx.k.collapse_continent_first = first;
            let mut st = planet(&fx.geo, belt, 26_000.0);
            for (i, s) in st.samples.iter_mut().enumerate() {
                if thin(fx.geo.points()[i]) {
                    let f = 30_000.0 / s.thickness_m;
                    for x in &mut s.pools {
                        *x *= f;
                    }
                    s.thickness_m = s.pools.iter().sum();
                }
            }
            let before = st.samples.clone();
            let c = step_once(&fx, &mut st, 1);
            let filled = (0..st.samples.len())
                .filter(|&i| {
                    thin(fx.geo.points()[i])
                        && st.samples[i].thickness_m > before[i].thickness_m + 1.0
                })
                .count();
            (c, filled)
        };
        let (c, filled) = run(true);
        assert!(
            c.sources > 0 && c.converted == 0 && filled > 20,
            "{c:?} {filled}"
        );
        let (c, filled) = run(false);
        assert!(c.converted > 0 && filled == 0, "{c:?} {filled}");
    }

    /// With nowhere to spread (the collapse off), a column thicker than `crust_max_km` loses the
    /// excess from its bottom (mafic root first) and keeps its surface rock; its surface stays
    /// where it stood when it is lower than the column it keeps would hold up. Columns at or
    /// under the limit, and ocean floor, keep all.
    #[test]
    fn crust_beyond_the_thickest_column_founders_from_the_bottom() {
        let mut fx = Fx::new();
        fx.k.collapse_myr = f64::NAN;
        let belt = |p: DVec3| p.x < -0.1 && p.y > 0.766 && p.y < 0.85;
        let mut st = planet(&fx.geo, belt, 0.0);
        for (i, s) in st.samples.iter_mut().enumerate() {
            if belt(fx.geo.points()[i]) {
                s.pools[pool::FELSIC_PLUTONIC] += 100_000.0;
                s.thickness_m += 100_000.0;
            }
        }
        let before = st.samples.clone();
        let lost = founder(&mut st, &fx.geo, &fx.env(1), &mut Collapse::default(), 0.0);
        let t_max = fx.k.crust_max_km * 1000.0;
        let mut n = 0;
        for (i, s) in st.samples.iter().enumerate() {
            let b = &before[i];
            if belt(fx.geo.points()[i]) {
                n += 1;
                assert!((f64::from(s.thickness_m) - t_max).abs() < 1.0);
                assert_eq!(s.pools[pool::MAFIC_PLUTONIC], 0.0);
                assert_eq!(s.pools[pool::SEDIMENT], b.pools[pool::SEDIMENT]);
                assert_eq!(s.pools[pool::METAMORPHIC], b.pools[pool::METAMORPHIC]);
                assert_eq!(s.elevation_m, b.elevation_m);
            } else {
                assert_eq!(s, b);
            }
        }
        let expect = n as f64 * (135_100.0 - t_max) * fx.norm.sample_area_km2();
        assert!((lost - expect).abs() < 1e-3 * expect, "{lost} vs {expect}");
    }

    /// The excess over `crust_max_km` spreads at once to the nearest room on the plate, even while
    /// an orogeny holds the belt up (PLT-277): ocean floor beside the continent fills and turns
    /// continental, and only what finds no room within reach founders. Volume is kept except
    /// what founders; the column ends at the limit either way.
    #[test]
    fn the_thickest_crust_spreads_before_it_founders() {
        let mut fx = Fx::new();
        // The shortest reach (1.5 spacings): only the ocean floor right beside the belt is room.
        fx.k.collapse_reach_km = 1.0;
        let belt = |p: DVec3| p.x < -0.1 && p.y > 0.766 && p.y < 0.85;
        let mut st = planet(&fx.geo, belt, 0.0);
        for (i, s) in st.samples.iter_mut().enumerate() {
            if belt(fx.geo.points()[i]) {
                s.pools[pool::FELSIC_PLUTONIC] += 100_000.0;
                s.thickness_m += 100_000.0;
                // Raised by an orogeny this step: the collapse would hold it, the limit does not.
                s.orogeny_myr = 2.0;
            }
        }
        let crust0 = crust_m(&st);
        let c0 = continental(&st, 0);
        let mut out = Collapse::default();
        let lost = founder(&mut st, &fx.geo, &fx.env(1), &mut out, 0.0);
        let area = fx.norm.sample_area_km2();
        let t_max = fx.k.crust_max_km * 1000.0;
        for (i, s) in st.samples.iter().enumerate() {
            if belt(fx.geo.points()[i]) {
                assert!(f64::from(s.thickness_m) <= t_max + 1.0);
            }
        }
        assert!(
            out.converted > 0,
            "ocean floor beside the belt turned continental"
        );
        assert_eq!(continental(&st, 0), c0 + out.converted as usize);
        assert_eq!(
            continental(&st, 1),
            continental(&planet(&fx.geo, belt, 0.0), 1)
        );
        let n_belt = (0..st.samples.len())
            .filter(|&i| belt(fx.geo.points()[i]))
            .count();
        let excess = n_belt as f64 * (135_100.0 - t_max) * area;
        assert!(lost > 0.0 && lost < excess, "{lost} of {excess} foundered");
        assert!(
            (crust_m(&st) * area + lost - crust0 * area).abs() < 1e-5 * crust0 * area,
            "volume kept but for what foundered"
        );
        assert!((out.moved_km2_m - (excess - lost)).abs() < 1e-3 * excess);
    }
}

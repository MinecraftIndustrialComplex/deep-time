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
// reach), PLT-258 (the thickest crust).

use std::collections::BTreeMap;

use super::Env;
use super::geometry::{Geo, dijkstra};
use super::pools::{
    COLUMN_ISOSTATIC_FACTOR, continental_column, eclogite_m, isostatic_change_m, root_drop_m,
};
use super::state::{CONTINENTAL, OCEANIC, PlateState, TRENCH, pool};

/// `WorldState::globals`: crust moved by the collapse over the run, km³ …
pub const MOVED_GLOBAL: &str = "plates/collapse_km3";
/// … and the ocean floor it turned continental, km² (Earth-normalized).
pub const GAINED_GLOBAL: &str = "plates/collapse_gained_km2";
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
/// Runs every step, before [`step`].
pub fn founder(state: &mut PlateState, geo: &Geo, env: &Env, out: &mut Collapse) -> f64 {
    let k = env.k;
    if k.crust_max_km.is_nan() || k.crust_max_km <= 0.0 {
        return 0.0;
    }
    // ledger: PLT-258.
    let t_max = (k.crust_max_km * 1000.0).max(normal_thickness_m());
    if spreads(k) {
        // ledger: PLT-277 (the excess spreads before any of it founders).
        let mut by_plate: BTreeMap<u32, Vec<(usize, f64)>> = BTreeMap::new();
        for (i, s) in state.samples.iter().enumerate() {
            if s.kind == CONTINENTAL && !s.is_hidden() && f64::from(s.thickness_m) > t_max {
                by_plate
                    .entry(state.plate_of(i))
                    .or_default()
                    .push((i, f64::from(s.thickness_m) - t_max));
            }
        }
        // The excess goes to continental crust thinner than the collapse thickness (crust
        // thicker than that does not stay, PLT-256: lower crust flows out of the thickened belt
        // but does not build a second one around it) and to ocean floor up to the normal column.
        let to = (k.collapse_thickness_km * 1000.0).clamp(normal_thickness_m(), t_max);
        if !by_plate.is_empty() {
            spread(state, geo, env, by_plate, to, out);
        }
    }
    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
    let mut lost = 0.0;
    for s in state.samples.iter_mut() {
        if s.kind != CONTINENTAL || s.is_hidden() || f64::from(s.thickness_m) <= t_max {
            continue;
        }
        let mut left = f64::from(s.thickness_m) - t_max;
        let pools_before = s.pools;
        for &p in &ROOT_ORDER {
            let take = f64::from(s.pools[p]).min(left);
            s.pools[p] = (f64::from(s.pools[p]) - take) as f32;
            left -= take;
            if left <= 0.0 {
                break;
            }
        }
        let before = f64::from(s.thickness_m);
        s.thickness_m = s.pools.iter().sum();
        let gone = before - f64::from(s.thickness_m);
        lost += gone;
        // Isostasy, as in the collapse: towards the height of the column kept, never past it (a
        // root that founders takes its eclogite with it, so the column may rise).
        let z = f64::from(s.elevation_m);
        let change = isostatic_change_m(&pools_before, &s.pools, env.gravity);
        s.elevation_m = toward(z, change, iso_height(&s.pools, z_0, env.gravity)) as f32;
    }
    lost * env.norm.sample_area_km2()
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
fn toward(z: f64, change: f64, iso: f64) -> f64 {
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

/// What one step did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Collapse {
    /// Samples that released crust.
    pub sources: u32,
    /// Crust moved, km² × m.
    pub moved_km2_m: f64,
    /// Ocean samples turned continental.
    pub converted: u32,
}

/// One step of the collapse on `state` (see the module docs). Runs after the step's uplift
/// (collision belts and subduction), so a sample raised this step is known.
pub fn step(state: &mut PlateState, geo: &Geo, env: &Env) -> Collapse {
    let k = env.k;
    if !spreads(k) {
        return Collapse::default();
    }
    let rate = 1.0 - (-env.dt_myr / k.collapse_myr).exp();
    let normal = normal_thickness_m();
    let t_c = (k.collapse_thickness_km * 1000.0).max(normal);
    // A sample raised by an orogeny carries the time of the step that raised it; it is held up
    // until no orogeny has raised it for `collapse_quiet_myr` (at least this step). ledger: PLT-259
    let quiet_before = (env.t_myr - k.collapse_quiet_myr.max(0.5 * env.dt_myr)) as f32;

    // Each plate's sources and what they would release this step, m of thickness.
    let mut by_plate: BTreeMap<u32, Vec<(usize, f64)>> = BTreeMap::new();
    for (i, s) in state.samples.iter().enumerate() {
        if s.kind != CONTINENTAL || s.is_hidden() || s.orogeny_myr >= quiet_before {
            continue;
        }
        // ledger: PLT-254, PLT-255, PLT-256.
        let q = rate * (f64::from(s.thickness_m) - t_c);
        if q > 0.0 {
            by_plate.entry(state.plate_of(i)).or_default().push((i, q));
        }
    }
    let mut out = Collapse::default();
    spread(state, geo, env, by_plate, normal, &mut out);
    out
}

/// Whether the collapse's settings let crust spread at all.
fn spreads(k: &crate::params::TectonicParams) -> bool {
    !(k.collapse_myr.is_nan()
        || k.collapse_myr <= 0.0
        || !k.collapse_reach_km.is_finite()
        || k.collapse_reach_km <= 0.0)
}

/// Moves crust from each plate's `sources` (sample, m of thickness to release) to the nearest
/// room on the same plate within `collapse_reach_km` (see the module docs, "The rule": continental
/// crust thinner than `continental_to_m`, filled up to it, and ocean floor, filled up to the
/// normal column), thinning
/// the sources in proportion to what found room; returns the share of each plate's release that
/// was placed (sources of a plate whose release found no room keep their crust).
fn spread(
    state: &mut PlateState,
    geo: &Geo,
    env: &Env,
    by_plate: BTreeMap<u32, Vec<(usize, f64)>>,
    continental_to_m: f64,
    out: &mut Collapse,
) -> BTreeMap<u32, f64> {
    let k = env.k;
    let normal = normal_thickness_m();
    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
    let n = state.samples.len();
    let plate_of: Vec<u32> = (0..n).map(|i| state.plate_of(i)).collect();
    let mut shares = BTreeMap::new();
    // At least to the next ring of samples: on a coarse lattice the reach may be shorter than a
    // sample spacing, and a column always borders its neighbours.
    let reach = env
        .norm
        .rad(k.collapse_reach_km)
        .max(REACH_MIN_SPACINGS * env.norm.spacing_rad());
    for (p, sources) in by_plate {
        let budget: f64 = sources.iter().map(|&(_, q)| q).sum();
        if budget <= 0.0 {
            continue;
        }
        let src: Vec<(u32, f64, u32)> = sources.iter().map(|&(i, _)| (i as u32, 0.0, 0)).collect();
        let samples = &state.samples;
        let on_plate = |v: u32| plate_of[v as usize] == p && !samples[v as usize].is_hidden();
        let (dist, _) = dijkstra(geo, &src, |_, b| on_plate(b), reach);
        // The room, nearest first, not at a trench: continental crust thinner than
        // `continental_to_m`, ocean floor up to the normal column.
        let mut room: Vec<(f64, usize, f64)> = (0..n)
            .filter(|&j| dist[j].is_finite() && on_plate(j as u32))
            .filter_map(|j| {
                let s = &samples[j];
                let to = if s.kind == CONTINENTAL {
                    continental_to_m
                } else {
                    normal
                };
                let r = to - f64::from(s.thickness_m);
                (r > 0.0 && s.flags & TRENCH == 0).then_some((dist[j], j, r))
            })
            .collect();
        room.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
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
            s.thickness_m = s.pools.iter().sum();
            // Isostasy (PLT-281, PLT-278): down by take / 7.0 less the root's change, not below
            // the height the crust kept holds up.
            let z = f64::from(s.elevation_m);
            let change = isostatic_change_m(&pools_before, &s.pools, env.gravity);
            s.elevation_m = toward(z, change, iso_height(&s.pools, z_0, env.gravity)) as f32;
            out.sources += 1;
        }
        let total: f64 = mix.iter().sum();
        if total <= 0.0 {
            continue;
        }
        for m in &mut mix {
            *m /= total;
        }
        // Fill the receivers; an ocean sample filled to the normal column is continent now.
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
            if s.kind == OCEANIC && f64::from(s.thickness_m) >= normal - 1.0 {
                s.kind = CONTINENTAL;
                out.converted += 1;
            }
        }
        out.moved_km2_m += placed * env.norm.sample_area_km2();
    }
    shares
}

#[cfg(test)]
mod tests {
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
    fn planet(geo: &Geo, belt: impl Fn(DVec3) -> bool, thick_m: f32) -> PlateState {
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
        fn new() -> Fx {
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
    }

    fn continental(st: &PlateState, plate: u32) -> usize {
        (0..st.samples.len())
            .filter(|&i| st.plate_of(i) == plate && st.samples[i].kind == CONTINENTAL)
            .count()
    }

    fn crust_m(st: &PlateState) -> f64 {
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

    /// One collapse step on fixed positions.
    fn step_once(fx: &Fx, st: &mut PlateState, n: u64) -> Collapse {
        let env = fx.env(n);
        step(st, &fx.geo, &env)
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
        assert_eq!(step_once(&fx, &mut st, 1), Collapse::default());
        assert_eq!(st.samples, before);
        // The same belt stamped last step collapses now.
        assert!(step_once(&fx, &mut st.clone(), 2).sources > 0);
        // With a quiet time of 10 Myr it waits until the stamp is more than 10 Myr old.
        let mut quiet = Fx::new();
        quiet.k.collapse_quiet_myr = 10.0;
        for step in 2..=6 {
            assert_eq!(
                step_once(&quiet, &mut st, step),
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
        assert_eq!(step_once(&fx, &mut st, 1), Collapse::default());
        assert_eq!(st.samples, before);
        // Off.
        fx.k.collapse_reach_km = 1500.0;
        fx.k.collapse_myr = 0.0;
        let mut st = planet(&fx.geo, belt, 26_000.0);
        assert_eq!(step_once(&fx, &mut st, 1), Collapse::default());
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
        let lost = founder(&mut st, &fx.geo, &fx.env(1), &mut Collapse::default());
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
        let lost = founder(&mut st, &fx.geo, &fx.env(1), &mut out);
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

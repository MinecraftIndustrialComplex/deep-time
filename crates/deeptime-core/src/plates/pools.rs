// SPDX-License-Identifier: LGPL-3.0-or-later
//! Rock pools per sample and the per-step surface processes of Cortial §4.5.
//!
//! # Rock pools — ported from tectonics.js
//! Ported from tectonics.js by Carl Davidson (https://github.com/davidson16807/tectonics.js,
//! `noncompiled/models/lithosphere/RockColumn.js` and `Crust.js`, commit ce371fa, CC BY 4.0,
//! https://creativecommons.org/licenses/by/4.0/). Changes: pools hold metres of rock instead of
//! kg/m²; they live on the plate model's L0 samples instead of tectonics.js's rasters; and the
//! rules below are rewritten for this model. What is kept is the pool set and its semantics:
//!
//! - seven pools: sediment, sedimentary, metamorphic, felsic plutonic, felsic volcanic, mafic
//!   volcanic, mafic plutonic (`RockColumn`);
//! - the felsic family (the first five) is **conserved**: "felsic type rock is never created or
//!   destroyed without our explicit say-so" (`Crust.js`) — erosion moves it, subduction does not
//!   destroy it but adds it to the column on top (`Crust.overlap`: conserved pools of an overridden
//!   column are added to the column above; mafic pools are replaced) — while mafic rock is made at
//!   ridges and destroyed by subduction;
//! - lithification turns sediment into sedimentary rock beyond an overburden of 2.2 MPa
//!   (`Crust.model_lithification`), metamorphosis turns sedimentary rock into metamorphic rock
//!   beyond 300 MPa (`Crust.model_metamorphosis`); both thresholds are converted to thickness with
//!   the planet's gravity;
//! - erosion strips the column from the top and moves the material downhill as sediment
//!   (`Crust.model_erosion`, here one hop per step to the lowest neighbour of the same group).
//!
//! Explicit creation happens at ridges (fresh oceanic crust), arcs and hotspots/LIPs.
//!
//! # Elevation processes (Cortial et al. 2019, §4.5)
//! Continental erosion z −= (z / z_c)·ε_c·δt (above sea level), oceanic dampening
//! z −= (1 − z / z_t)·ε_o·δt, and sediment accretion in trenches z += ε_t·δt. Continental erosion
//! removes rock ρ_m / (ρ_m − ρ_c) ≈ 5.5 times the surface lowering from the pools (Airy isostasy,
//! mantle 3,300 and crust 2,700 kg/m³), since Cortial's z is the surface.

use super::state::{OCEANIC, Sample, TRENCH, pool};

/// Density of sediment for lithification, kg/m³ (tectonics.js uses 1,500).
const RHO_SEDIMENT: f64 = 1500.0;
/// Density of sedimentary rock for metamorphosis, kg/m³ (tectonics.js: 2,700).
const RHO_ROCK: f64 = 2700.0;
/// Lithification overburden, Pa (tectonics.js: 2.2e6, "500ft of sediment").
const LITHIFY_PA: f64 = 2.2e6;
/// Metamorphosis overburden, Pa (tectonics.js: 300e6, "11km of sedimentary rock").
const METAMORPHOSE_PA: f64 = 300.0e6;
/// Rock removed per metre of surface lowering: ρ_m / (ρ_m − ρ_c) with 3,300 and 2,700 kg/m³.
pub const ISOSTATIC_FACTOR: f64 = 3300.0 / (3300.0 - 2700.0);

/// Order in which erosion strips conserved pools, top first.
const EROSION_ORDER: [usize; 5] = [
    pool::SEDIMENT,
    pool::SEDIMENTARY,
    pool::FELSIC_VOLCANIC,
    pool::METAMORPHIC,
    pool::FELSIC_PLUTONIC,
];

/// A continental column, metres (design choice: 35 km, granite-dominated upper crust over a
/// mafic lower crust).
pub fn continental_column() -> [f32; pool::COUNT] {
    let mut p = [0.0; pool::COUNT];
    p[pool::SEDIMENT] = 100.0;
    p[pool::SEDIMENTARY] = 2000.0;
    p[pool::METAMORPHIC] = 6000.0;
    p[pool::FELSIC_PLUTONIC] = 21000.0;
    p[pool::FELSIC_VOLCANIC] = 1500.0;
    p[pool::MAFIC_PLUTONIC] = 4500.0;
    p
}

/// Fresh oceanic crust at a ridge, metres: 2 km of basalt over 5 km of gabbro.
pub fn oceanic_column() -> [f32; pool::COUNT] {
    let mut p = [0.0; pool::COUNT];
    p[pool::MAFIC_VOLCANIC] = 2000.0;
    p[pool::MAFIC_PLUTONIC] = 5000.0;
    p
}

/// Removes `metres` of rock from the top of the conserved pools; returns what was removed.
pub fn strip(pools: &mut [f32; pool::COUNT], metres: f64) -> f64 {
    let mut left = metres.max(0.0) as f32;
    let mut removed = 0.0f32;
    for &k in &EROSION_ORDER {
        if left <= 0.0 {
            break;
        }
        let take = pools[k].min(left);
        pools[k] -= take;
        left -= take;
        removed += take;
    }
    f64::from(removed)
}

/// Lithification and metamorphosis by overburden (tectonics.js thresholds), with gravity `g`.
pub fn diagenesis(pools: &mut [f32; pool::COUNT], g: f64) {
    let lithify_m = (LITHIFY_PA / (g * RHO_SEDIMENT)) as f32;
    let excess = pools[pool::SEDIMENT] - lithify_m;
    if excess > 0.0 {
        pools[pool::SEDIMENT] -= excess;
        pools[pool::SEDIMENTARY] += excess;
    }
    let meta_m = (METAMORPHOSE_PA / (g * RHO_ROCK)) as f32;
    let over = pools[pool::SEDIMENT] + pools[pool::SEDIMENTARY] - meta_m;
    if over > 0.0 {
        let m = over.min(pools[pool::SEDIMENTARY]);
        pools[pool::SEDIMENTARY] -= m;
        pools[pool::METAMORPHIC] += m;
    }
}

/// `Crust.overlap`: the conserved pools of an overridden column join the column on top.
pub fn accrete(top: &mut [f32; pool::COUNT], under: &[f32; pool::COUNT]) {
    for k in 0..pool::CONSERVED {
        top[k] += under[k];
    }
}

/// Total thickness of a column, metres.
pub fn thickness(pools: &[f32; pool::COUNT]) -> f32 {
    pools.iter().sum()
}

/// The constants of Cortial §4.5 for one step, in metres.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceRates {
    /// ε_c·δt.
    pub erosion_m: f64,
    /// ε_o·δt.
    pub damping_m: f64,
    /// ε_t·δt.
    pub accretion_m: f64,
    /// Pelagic sediment per step, metres.
    pub pelagic_m: f64,
    pub z_c: f64,
    pub z_t: f64,
    pub trench_relax: f64,
    pub gravity: f64,
}

/// One sample's surface step; returns the rock eroded from it (metres), to be exported downhill.
pub fn surface_step(s: &mut Sample, r: &SurfaceRates) -> f64 {
    let mut z = f64::from(s.elevation_m);
    let mut eroded = 0.0;
    if s.kind == OCEANIC {
        z -= (1.0 - z / r.z_t) * r.damping_m;
        if !s.is_hidden() {
            s.pools[pool::SEDIMENT] += r.pelagic_m as f32;
        }
    } else if z > 0.0 {
        let lower = z / r.z_c * r.erosion_m;
        z -= lower;
        eroded = strip(&mut s.pools, lower * ISOSTATIC_FACTOR);
    }
    if s.flags & TRENCH != 0 {
        z += (r.z_t - z) * r.trench_relax;
        z += r.accretion_m;
    }
    s.elevation_m = z.clamp(r.z_t, r.z_c) as f32;
    diagenesis(&mut s.pools, r.gravity);
    s.thickness_m = thickness(&s.pools);
    eroded
}

/// Oceanic depth of crust `age_myr` old implied by Cortial's dampening from the ridge height:
/// z(a) = z_t + (z_r − z_t)·exp(−a·ε_o / |z_t|).
pub fn ocean_elevation(age_myr: f64, z_r: f64, z_t: f64, damping_mm_yr: f64) -> f64 {
    let tau_myr = z_t.abs() / (damping_mm_yr * 1.0e-3) / 1.0e6;
    z_t + (z_r - z_t) * (-age_myr.max(0.0) / tau_myr).exp()
}

/// Oceanic heat flow, mW/m²: q = 510 / √t (Stein & Stein 1992, GDH1), capped at 300.
pub fn ocean_heat_flow(age_myr: f64) -> f64 {
    (510.0 / age_myr.max(0.1).sqrt()).min(300.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stripping_and_diagenesis_conserve_felsic_rock() {
        let mut p = continental_column();
        let before: f32 = p[..pool::CONSERVED].iter().sum();
        let removed = strip(&mut p, 2500.0);
        assert!((removed - 2500.0).abs() < 1e-3);
        assert_eq!(p[pool::SEDIMENT], 0.0);
        assert_eq!(p[pool::SEDIMENTARY], 0.0);
        let after: f32 = p[..pool::CONSERVED].iter().sum();
        assert!((before - after - 2500.0).abs() < 1e-2);
        let mut q = [0.0; pool::COUNT];
        q[pool::SEDIMENT] = 20_000.0;
        let total = thickness(&q);
        diagenesis(&mut q, 9.81);
        assert!(
            (thickness(&q) - total).abs() < 1e-2,
            "diagenesis moves, never makes"
        );
        assert!(q[pool::SEDIMENT] < 160.0 && q[pool::METAMORPHIC] > 8000.0);
        let mut top = oceanic_column();
        accrete(&mut top, &continental_column());
        assert_eq!(
            top[pool::MAFIC_PLUTONIC],
            5000.0,
            "mafic rock is not accreted"
        );
        assert_eq!(top[pool::FELSIC_PLUTONIC], 21000.0);
    }

    #[test]
    fn dampening_matches_its_closed_form() {
        let r = SurfaceRates {
            erosion_m: 60.0,
            damping_m: 0.04e-3 * 1.0e6,
            accretion_m: 600.0,
            pelagic_m: 0.0,
            z_c: 10_000.0,
            z_t: -10_000.0,
            trench_relax: 0.5,
            gravity: 9.81,
        };
        let mut s = Sample::blank(0);
        s.elevation_m = -1000.0;
        for _ in 0..100 {
            surface_step(&mut s, &r);
        }
        let expect = ocean_elevation(100.0, -1000.0, -10_000.0, 0.04);
        assert!(
            (f64::from(s.elevation_m) - expect).abs() < 30.0,
            "{} vs {expect}",
            s.elevation_m
        );
        assert!(ocean_heat_flow(100.0) > 50.0 && ocean_heat_flow(0.0) == 300.0);
    }
}

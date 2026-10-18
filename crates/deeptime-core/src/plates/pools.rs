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

/// The mantle's density under the isostatic column, kg/m³ (the surface stage's flexure uses the
/// same, ISO-043) …
const RHO_MANTLE_ISO: f64 = 3300.0; // ledger: ISO-043
/// … and the continental crust's mean density, kg/m³: "The mean crustal density calculated from
/// our model is 2830 kg/m3" (Christensen & Mooney 1995, abstract). It sets how high a whole
/// column stands for its thickness when crust is stacked onto it or spread from it (belts,
/// collapse, foundering, cordillera roots): `ρ_m / (ρ_m − ρ̄_c)` = 7.0 m of crust per m of height, 142 m of height
/// per km of crust, where Hyndman & Currie (2011, full text) measure 131 ± 12 m/km (Cordillera)
/// and 141 ± 27 m/km (stable areas). [`ISOSTATIC_FACTOR`] (upper crust, 2,700 kg/m³) stays for
/// rock eroded from the top of a column.
pub const RHO_CRUST_MEAN: f64 = 2830.0; // ledger: PLT-281
/// Crust per metre of height for a whole column, ρ_m / (ρ_m − ρ̄_c) (see [`RHO_CRUST_MEAN`]).
pub const COLUMN_ISOSTATIC_FACTOR: f64 = RHO_MANTLE_ISO / (RHO_MANTLE_ISO - RHO_CRUST_MEAN);

/// The eclogite root (2026-10-06, Ben: "Fix isostasy first"): mafic lower crust carried deeper
/// than the pressure at which it turns to eclogite becomes denser than the mantle, so a thickened
/// column stands lower than Airy isostasy with one crustal density gives it. The pressure past
/// which mafic lower crust eclogitizes, Pa: "~15 to ~20 kbars" at "~60 km depth beneath southern
/// Tibet" (Wittlinger et al. 2009, full text), the lower end; "below ~60 km the lower crust may
/// have reached roughly 30% eclogitization" (Shi F. et al. 2018, full text) …
pub const ECLOGITE_PA: f64 = 1.5e9; // ledger: PLT-278
/// … the density of eclogitized mafic crust, kg/m³: "mafic eclogite (3,450 kg/m3)" (Shi D. et
/// al. 2020, full text, after Hetényi et al. 2007; Krystopowicz & Currie 2013 use 3,450) …
pub const RHO_ECLOGITE: f64 = 3450.0; // ledger: PLT-279
/// … and the share of the mafic crust beyond that pressure that is eclogite: the transformation
/// is kinetically delayed without fluids, and the Indian lower crust under southern Tibet is "only
/// ~30% eclogitized so gravitationally stable" (Shi D. et al. 2020, full text).
pub const ECLOGITE_SHARE: f64 = 0.3; // ledger: PLT-280

/// The thickness of eclogite in a continental column, m: the share [`ECLOGITE_SHARE`] of its mafic
/// pools (the bottom of the column) that lies deeper than the eclogite pressure (depth
/// `ECLOGITE_PA / (ρ̄_c g)`, 54 km at Earth's gravity).
pub fn eclogite_m(pools: &[f32; pool::COUNT], gravity: f64) -> f64 {
    let t: f64 = pools.iter().map(|&x| f64::from(x.max(0.0))).sum();
    let mafic = f64::from(pools[pool::MAFIC_PLUTONIC].max(0.0))
        + f64::from(pools[pool::MAFIC_VOLCANIC].max(0.0));
    let depth = ECLOGITE_PA / (RHO_CRUST_MEAN * gravity.max(1e-3));
    ECLOGITE_SHARE * mafic.min((t - depth).max(0.0))
}

/// How far an eclogite root of `eclogite_m` metres pulls the surface down below its height under
/// Airy isostasy with the crust's mean density, m: `E (ρ_e − ρ̄_c) / ρ_m`.
pub fn root_drop_m(eclogite_m: f64) -> f64 {
    eclogite_m * (RHO_ECLOGITE - RHO_CRUST_MEAN) / RHO_MANTLE_ISO
}

/// The change of a continental column's surface when its pools go from `before` to `after`, m:
/// the Airy change of its thickness (`ΔT / COLUMN_ISOSTATIC_FACTOR`) less the change of its
/// eclogite root's pull ([`root_drop_m`]).
pub fn isostatic_change_m(
    before: &[f32; pool::COUNT],
    after: &[f32; pool::COUNT],
    gravity: f64,
) -> f64 {
    let t = |p: &[f32; pool::COUNT]| -> f64 { p.iter().map(|&x| f64::from(x.max(0.0))).sum() };
    (t(after) - t(before)) / COLUMN_ISOSTATIC_FACTOR
        - (root_drop_m(eclogite_m(after, gravity)) - root_drop_m(eclogite_m(before, gravity)))
}

/// Order in which erosion strips the pools, top first: the conserved pools and the mafic volcanic
/// rock of lava fields, shields and ocean crust (plate task 8: the volcano stage's cones reach the
/// pools as volcanic rock, so eroding a basalt cone strips its basalt, not the crust under it;
/// ledger: PLT-286).
const EROSION_ORDER: [usize; 6] = [
    pool::SEDIMENT,
    pool::SEDIMENTARY,
    pool::FELSIC_VOLCANIC,
    pool::MAFIC_VOLCANIC,
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

/// Removes `metres` of rock from the top of the column (`EROSION_ORDER`); returns what was removed.
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
    /// The elevation oceanic crust subsides towards (`oceanic_floor_elevation_m`; Cortial: z_t).
    pub z_floor: f64,
    pub trench_relax: f64,
    pub gravity: f64,
}

/// One sample's surface step; returns the rock eroded from it (metres), to be exported downhill.
/// `held_floor` (finite) is the floor an oceanic sample of an ocean–continent transition cools
/// towards instead of `z_floor`: its thicker crust holds it up (`transition.rs`, ledger PLT-264).
pub fn surface_step(s: &mut Sample, r: &SurfaceRates, held_floor: f32) -> f64 {
    let mut z = f64::from(s.elevation_m);
    let mut eroded = 0.0;
    if s.kind == OCEANIC {
        // z −= (1 − z/z_f)·ε_o·δt, the relaxation towards z_f at the rate ε_o / |z_f|; towards
        // the held floor at the same rate where the crust holds one.
        if held_floor.is_finite() {
            let floor = f64::from(held_floor).max(r.z_floor);
            z -= (z - floor) / -r.z_floor * r.damping_m;
        } else {
            z -= (1.0 - z / r.z_floor) * r.damping_m;
        }
        if !s.is_hidden() {
            s.pools[pool::SEDIMENT] += r.pelagic_m as f32;
        }
    } else if z > 0.0 {
        let lower = z / r.z_c * r.erosion_m;
        z -= lower;
        eroded = strip(&mut s.pools, lower * ISOSTATIC_FACTOR);
    }
    // A trench is the oceanic slab's: continental crust at a collision front is not pulled down
    // (it builds the wedge, `belts.rs`; plate tasks 3 and 4). ledger: PLT-252
    if s.flags & TRENCH != 0 && s.kind == OCEANIC {
        z += (r.z_t - z) * r.trench_relax;
        z += r.accretion_m;
    }
    s.elevation_m = z.clamp(r.z_t, r.z_c) as f32;
    diagenesis(&mut s.pools, r.gravity);
    s.thickness_m = thickness(&s.pools);
    eroded
}

/// Oceanic depth of crust `age_myr` old implied by the dampening from the ridge height towards
/// the floor z_f: z(a) = z_f + (z_r − z_f)·exp(−a·ε_o / |z_f|). Cortial's z_f is the trench depth;
/// the pipeline presets' (`oceanic_floor_elevation_m`) follows GDH1.
pub fn ocean_elevation(age_myr: f64, z_r: f64, z_f: f64, damping_mm_yr: f64) -> f64 {
    let tau_myr = z_f.abs() / (damping_mm_yr * 1.0e-3) / 1.0e6;
    z_f + (z_r - z_f) * (-age_myr.max(0.0) / tau_myr).exp()
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
            z_floor: -10_000.0,
            trench_relax: 0.5,
            gravity: 9.81,
        };
        let mut s = Sample::blank(0);
        s.elevation_m = -1000.0;
        for _ in 0..100 {
            surface_step(&mut s, &r, f32::NAN);
        }
        let expect = ocean_elevation(100.0, -1000.0, -10_000.0, 0.04);
        assert!(
            (f64::from(s.elevation_m) - expect).abs() < 30.0,
            "{} vs {expect}",
            s.elevation_m
        );
        // A shallower floor (the pipeline presets' GDH1 fit): the steps follow the same closed form.
        let r = SurfaceRates {
            damping_m: 0.215e-3 * 1.0e6,
            z_floor: -5600.0,
            ..r
        };
        let mut s = Sample::blank(0);
        s.elevation_m = -2500.0;
        for _ in 0..60 {
            surface_step(&mut s, &r, f32::NAN);
        }
        let expect = ocean_elevation(60.0, -2500.0, -5600.0, 0.215);
        assert!(
            (f64::from(s.elevation_m) - expect).abs() < 60.0,
            "{} vs {expect}",
            s.elevation_m
        );
        assert!(ocean_heat_flow(100.0) > 50.0 && ocean_heat_flow(0.0) == 300.0);
    }

    /// The pipeline presets' floor (−5,600 m at 0.215 mm/yr from a −2,500 m ridge) against Stein &
    /// Stein's GDH1 (1992): d = 2,600 + 365·√t m up to 20 Myr, 5,651 − 2,473·e^(−0.0278·t) after.
    /// Cortial's law (towards the −10 km trench at 0.05 mm/yr) is 0.9 km too shallow at 10 Myr
    /// and 1.6 km too deep at 200.
    #[test]
    fn the_gdh1_floor_follows_stein_and_stein() {
        let gdh1 = |t: f64| {
            if t < 20.0 {
                2600.0 + 365.0 * t.sqrt()
            } else {
                5651.0 - 2473.0 * (-0.0278 * t).exp()
            }
        };
        for t in [5.0, 10.0, 20.0, 35.0, 50.0, 80.0, 100.0, 150.0, 200.0] {
            // The exponential cannot follow GDH1's √t near the ridge: within 0.4 km there, 0.12 km
            // from 20 Myr on.
            let tol = if t < 20.0 { 400.0 } else { 120.0 };
            let z = ocean_elevation(t, -2500.0, -5600.0, 0.215);
            assert!(
                (-z - gdh1(t)).abs() < tol,
                "{t} Myr: {z} vs GDH1 {}",
                gdh1(t)
            );
        }
        let cortial = |t: f64| -ocean_elevation(t, -2500.0, -10_000.0, 0.05);
        assert!(gdh1(10.0) - cortial(10.0) > 800.0 && cortial(200.0) - gdh1(200.0) > 1500.0);
    }

    /// The eclogite root (PLT-278 to PLT-280): the normal column and any column whose mafic base
    /// stays above the eclogite pressure stand at their Airy height; a column thickened past it
    /// keeps the eclogite share of its deep mafic crust as a dense root, which pulls the surface
    /// down by `E (ρ_e − ρ_c) / ρ_m`; and on a planet with lower gravity the eclogite depth lies
    /// deeper, so the same column has less root.
    #[test]
    fn a_thickened_column_grows_an_eclogite_root_below_its_pressure() {
        let normal = continental_column();
        assert_eq!(eclogite_m(&normal, 9.81), 0.0);
        let scaled = |f: f32| normal.map(|x| x * f);
        let depth = ECLOGITE_PA / (RHO_CRUST_MEAN * 9.81);
        // A column thickened in proportion until its base lies below the eclogite depth.
        let total: f32 = normal.iter().sum();
        let f = ((depth + 10_000.0) / f64::from(total)) as f32;
        let thick = scaled(f);
        let t: f64 = thick.iter().map(|&x| f64::from(x)).sum();
        let mafic = f64::from(thick[pool::MAFIC_PLUTONIC] + thick[pool::MAFIC_VOLCANIC]);
        let e = eclogite_m(&thick, 9.81);
        assert!(
            (e - ECLOGITE_SHARE * mafic.min(t - depth)).abs() < 1.0,
            "{e}"
        );
        assert!(e > 0.0);
        let change = isostatic_change_m(&normal, &thick, 9.81);
        let airy = (t - f64::from(total)) / COLUMN_ISOSTATIC_FACTOR;
        assert!(
            (airy - change - e * (RHO_ECLOGITE - RHO_CRUST_MEAN) / RHO_MANTLE_ISO).abs() < 1e-6,
            "{airy} {change}"
        );
        // Lower gravity: the eclogite depth is deeper, the root smaller.
        assert!(eclogite_m(&thick, 6.0) < e);
        // Thickening a column that stays above the eclogite depth is plain Airy.
        let a = scaled(1.1);
        let ta: f64 = a.iter().map(|&x| f64::from(x)).sum();
        assert!(ta < depth);
        let plain = isostatic_change_m(&normal, &a, 9.81);
        assert!((plain - (ta - f64::from(total)) / COLUMN_ISOSTATIC_FACTOR).abs() < 1e-6);
    }
}

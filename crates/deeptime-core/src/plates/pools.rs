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
//!
//! # The ocean floor's age–depth law (`tectonics.ocean_depth`)
//! Cortial's dampening relaxes every oceanic height towards one floor, so whatever stands on old
//! floor (the sediment the surface lays, a margin's thicker crust) sinks back to it at the same
//! rate as the floor's own cooling: on the quick planet's old enclosed floor about 48 m/Myr,
//! where Earth's cooling gives about 2 (rivers-and-lakes.md, "Basins that outlast their
//! sediment"). `ocean_depth: gdh1` (every shipped Earth-like preset) instead sinks each sample
//! by what its age sinks bare ocean floor under water, Stein & Stein's GDH1 (1992):
//! d(t) = 2,600 + 365·√t m up to 20 Myr and 5,651 − 2,473·e^(−0.0278·t) after, so the step
//! takes d(t) − d(t − δt) whatever stands on the sample. Its load stays: the sediment and its
//! water's flexure are the surface's (`surface/flexure.rs`), and the room they take raises the sea
//! (`sea_level.rs`). Above the sea the cooling column sinks the dry share, d′/W with W the
//! water-loaded factor ρ_m/(ρ_m − ρ_w) (`settle::WATER_LOADED`). New floor starts at the ridge's
//! 2,600 m. ledger: PLT-129 (the old rule), PLT-352 (the law), PLT-353 (the dry share).

use super::state::{OCEANIC, Sample, TRENCH, pool};
use crate::params::{OceanDepth, TectonicParams};

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

/// The pools of a column's supracrustal cover: the layered rock laid down on its basement
/// (sediment, sedimentary rock, and lavas and tuffs), above its plutonic and metamorphic crust.
/// Docking at a trench strips a terrane to it (`tectonics.terrane_docking_cover`): accreted
/// terranes are "thin crustal units sheared off" (Tetreault & Buiter 2014, p. 1243), "only the
/// upper 2–5 km of arc crust" where the arc rides the subducting plate (p. 1251); the Yakutat
/// terrane's "sedimentary cover ... has been scraped off basement rocks and accreted to the
/// growing fold-and-thrust belt" (Enkelmann et al. 2008, p. 916). Taking the cover's base as the
/// detachment is the project's reading ("rheologically weak crustal layers ... can be activated
/// as detachments", Tetreault & Buiter 2014, p. 1243). ledger: PLT-761
pub const COVER: [usize; 4] = [
    pool::SEDIMENT,
    pool::SEDIMENTARY,
    pool::FELSIC_VOLCANIC,
    pool::MAFIC_VOLCANIC,
];

/// The thickness of a column's cover ([`COVER`]), m.
pub fn cover_m(pools: &[f32; pool::COUNT]) -> f64 {
    COVER.iter().map(|&p| f64::from(pools[p].max(0.0))).sum()
}

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

/// Fresh oceanic crust at a ridge, metres: 2 km of seismic layer 2 (lavas over the sheeted dikes,
/// `structure::columns::OCEAN_LAVA_SHARE`) over 5 km of gabbro. The igneous crust averages
/// 7.1 ± 0.8 km (White et al. 1992), layer 2 1.84 km (Christeson et al. 2019).
// ledger: PLT-104 (abstract-only)
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
    pub z_c: f64,
    pub z_t: f64,
    /// The elevation oceanic crust subsides towards (`oceanic_floor_elevation_m`; Cortial: z_t).
    pub z_floor: f64,
    /// The ridge crest (`ridge_elevation_m`): with `z_floor`, the depths GDH1's shape is drawn
    /// between under [`OceanDepth::Gdh1`] ([`gdh1_scale`]).
    pub z_ridge: f64,
    pub trench_relax: f64,
    /// The share of the way back to its age's bare floor that ocean floor a trench left below
    /// it rises in a step (`trench_release_half_life_myr`; 0 = none). ledger: PLT-443
    pub trench_release: f64,
    pub gravity: f64,
    /// The ocean floor's law (`tectonics.ocean_depth`).
    pub law: OceanDepth,
    /// The step's end and length, Myr (the floor's age, under [`OceanDepth::Gdh1`]).
    pub t_myr: f64,
    pub dt_myr: f64,
    /// The sea level the step starts under, m (the dry share, under [`OceanDepth::Gdh1`], and the
    /// height continental erosion takes).
    pub sea_m: f64,
}

/// The share of the way to z_t a trench sample moves in a step of `dt_myr`: it closes half the
/// distance every `half_life_myr` (0.5 of it per step at the 2-Myr step and a 2-Myr half-life).
pub fn trench_share(dt_myr: f64, half_life_myr: f64) -> f64 {
    1.0 - 0.5f64.powf(dt_myr / half_life_myr) // ledger: PLT-053
}

/// One sample's surface step; returns the rock eroded from it (metres), to be exported downhill.
/// `held_floor` (finite) is the floor an oceanic sample of an ocean–continent transition cools
/// towards instead of `z_floor`: its thicker crust holds it up (`transition.rs`, ledger PLT-264).
pub fn surface_step(s: &mut Sample, r: &SurfaceRates, held_floor: f32) -> f64 {
    let mut z = f64::from(s.elevation_m);
    let mut eroded = 0.0;
    if s.kind == OCEANIC {
        match r.law {
            OceanDepth::Dampening => {
                // z −= (1 − z/z_f)·ε_o·δt, the relaxation towards z_f at the rate ε_o / |z_f|;
                // towards the held floor at the same rate where the crust holds one.
                if held_floor.is_finite() {
                    let floor = f64::from(held_floor).max(r.z_floor);
                    z -= (z - floor) / -r.z_floor * r.damping_m;
                } else {
                    z -= (1.0 - z / r.z_floor) * r.damping_m;
                }
            }
            OceanDepth::Gdh1 => {
                // What the step's cooling sinks bare floor of this age, whatever stands on it.
                let age = s.age_myr(r.t_myr);
                let sink = gdh1_scale(r.z_ridge, r.z_floor)
                    * (gdh1_depth_m(age) - gdh1_depth_m(age - r.dt_myr));
                z -= if z > r.sea_m {
                    sink / super::settle::WATER_LOADED // ledger: PLT-353
                } else {
                    sink
                };
            }
        }
    } else if z > r.sea_m {
        // Cortial's ε_c·(z/z_c)·δt on the height above the sea (ledger: PLT-127, PLT-128).
        let lower = (z - r.sea_m) / r.z_c * r.erosion_m;
        z -= lower;
        eroded = strip(&mut s.pools, lower * ISOSTATIC_FACTOR);
    }
    // A trench is the oceanic slab's: continental crust at a collision front is not pulled down
    // (it builds the wedge, `belts.rs`; plate tasks 3 and 4). ledger: PLT-252
    if s.flags & TRENCH != 0 && s.kind == OCEANIC {
        z += (r.z_t - z) * r.trench_relax;
        z += r.accretion_m;
    } else if r.trench_release > 0.0
        && r.law == OceanDepth::Gdh1
        && s.kind == OCEANIC
        && !s.is_hidden()
    {
        // The trench's depth is the slab's to hold: floor no trench holds this step and that
        // stands below its age's bare floor, which only the pull puts it under, rises back.
        // ledger: PLT-443
        let floor = gdh1_floor_m(r.z_ridge, r.z_floor, s.age_myr(r.t_myr));
        if z < floor {
            z += (floor - z) * r.trench_release;
        }
    }
    s.elevation_m = z.clamp(r.z_t, r.z_c) as f32;
    diagenesis(&mut s.pools, r.gravity);
    s.thickness_m = thickness(&s.pools);
    eroded
}

/// GDH1's depth of bare ocean floor `age_myr` old under water, m below the sea (Stein & Stein
/// 1992): 2,600 + 365·√t up to 20 Myr, 5,651 − 2,473·e^(−0.0278·t) after (the two meet within
/// 0.3 m). Ages below 0 are the ridge's. ledger: PLT-352
pub fn gdh1_depth_m(age_myr: f64) -> f64 {
    let t = age_myr.max(0.0);
    if t < GDH1_KNEE_MYR {
        GDH1_RIDGE_M + 365.0 * t.sqrt()
    } else {
        GDH1_FLOOR_M - 2473.0 * (-0.0278 * t).exp()
    }
}

/// GDH1's ridge depth and the depth old floor flattens towards, m (Stein & Stein 1992).
/// ledger: PLT-352
pub const GDH1_RIDGE_M: f64 = 2600.0;
pub const GDH1_FLOOR_M: f64 = 5651.0;
/// GDH1's age where the √t branch hands over to the plate's, Myr. ledger: PLT-352
const GDH1_KNEE_MYR: f64 = 20.0;

/// The share of GDH1's subsidence a floor drawn between the ridge crest `z_ridge` and the old floor
/// `z_floor` (elevations, m) takes: (z_r − z_f) / (5,651 − 2,600). GDH1's shape keeps its ages and
/// its depth range is the preset's, so a player's ridge or abyssal depth moves the curve's ends;
/// 1 at GDH1's own depths (every shipped preset), where the floor is GDH1 exactly.
/// ledger: PLT-355
pub fn gdh1_scale(z_ridge: f64, z_floor: f64) -> f64 {
    (z_ridge - z_floor) / (GDH1_FLOOR_M - GDH1_RIDGE_M)
}

/// The height of bare ocean floor `age_myr` old under GDH1 drawn between the ridge crest
/// `z_ridge` and the old floor `z_floor` (PLT-355). ledger: PLT-351
pub fn gdh1_floor_m(z_ridge: f64, z_floor: f64, age_myr: f64) -> f64 {
    // z_r − s·(d − 2,600), written so that s = 1 at GDH1's depths gives −d to the bit.
    let s = gdh1_scale(z_ridge, z_floor);
    -(s * gdh1_depth_m(age_myr) + (-z_ridge - s * GDH1_RIDGE_M))
}

/// The height of bare ocean floor `age_myr` old under the preset's law (`tectonics.ocean_depth`),
/// m: GDH1's shape from the preset's ridge crest (GDH1's depth below the datum, Earth's sea
/// today, at the shipped depths; PLT-355), or the dampening's closed form [`ocean_elevation`].
/// The floor a ridge, a gap or the first ocean is made at. ledger: PLT-351
pub fn floor_at(k: &TectonicParams, age_myr: f64) -> f64 {
    match k.ocean_depth {
        OceanDepth::Gdh1 => gdh1_floor_m(k.ridge_elevation_m, k.oceanic_floor_elevation_m, age_myr),
        OceanDepth::Dampening => ocean_elevation(
            age_myr,
            k.ridge_elevation_m,
            k.oceanic_floor_elevation_m,
            k.oceanic_damping_mm_yr,
        ),
    }
}

/// Oceanic depth of crust `age_myr` old implied by the dampening from the ridge height towards
/// the floor z_f: z(a) = z_f + (z_r − z_f)·exp(−a·ε_o / |z_f|). Cortial's z_f is the trench depth.
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

    /// A column's cover (ledger PLT-761) is its layered rock on the basement: the continental
    /// column's 0.1 km of sediment, 2 km of sedimentary and 1.5 km of volcanic rock; an island
    /// arc of 15 km on 7 km of ocean crust (its basement, PLT-331) keeps its own lavas and tuffs,
    /// a sixth of its magma (PLT-046), not the ocean floor's: 2.5 km, inside Tetreault & Buiter
    /// 2014's "upper 2–5 km of arc crust" (p. 1251) that accretes.
    #[test]
    fn the_cover_is_the_layered_rock_on_the_basement() {
        assert!((cover_m(&continental_column()) - 3600.0).abs() < 1e-3);
        let mut s = crate::plates::state::Sample::blank(0);
        // The arc turned continental (PLT-047), its ocean crust kept under it as its basement.
        s.kind = crate::plates::state::CONTINENTAL;
        s.pools = oceanic_column();
        s.basement_m = 7000.0;
        let arc = 15_000.0_f32;
        let volcanic = arc / 6.0;
        s.pools[pool::MAFIC_VOLCANIC] += volcanic * 2.0 / 3.0;
        s.pools[pool::FELSIC_VOLCANIC] += volcanic / 3.0;
        s.pools[pool::FELSIC_PLUTONIC] += arc - volcanic;
        s.thickness_m = s.pools.iter().sum();
        let own = cover_m(&s.own_pools());
        assert!((own - 2500.0).abs() < 1.0, "{own}");
        assert!((2000.0..=5000.0).contains(&own));
    }

    /// The trench's pull is a half-life (P1-19 #27): half the way per 2-Myr step, exactly as the
    /// old per-step share, and one 4-Myr step goes as far as two 2-Myr steps.
    #[test]
    fn the_trench_share_follows_the_step_length() {
        assert_eq!(trench_share(2.0, 2.0), 0.5);
        let two = 1.0 - (1.0 - trench_share(2.0, 2.0)).powi(2);
        assert!((trench_share(4.0, 2.0) - two).abs() < 1e-12);
        let thirds = 1.0 - (1.0 - trench_share(1.0, 2.0)).powi(3);
        assert!((trench_share(3.0, 2.0) - thirds).abs() < 1e-12);
    }

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
            z_c: 10_000.0,
            z_t: -10_000.0,
            z_floor: -10_000.0,
            z_ridge: -1000.0,
            trench_relax: 0.5,
            trench_release: 0.0,
            gravity: 9.81,
            law: OceanDepth::Dampening,
            t_myr: 0.0,
            dt_myr: 1.0,
            sea_m: 0.0,
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
        // A shallower floor (the old pipeline presets' fit to GDH1): the same closed form.
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

    /// Stein & Stein's GDH1 (1992): d = 2,600 + 365·√t m up to 20 Myr, 5,651 − 2,473·e^(−0.0278·t)
    /// after (their eqs. 2a and 2b). The rate form sinks a sample by what its age sinks bare
    /// floor, so a bare sample follows the law exactly, a load on it rides down with it (the
    /// dampening wore loads away, PLT-129), and ground above the sea sinks by 1/W of it.
    #[test]
    fn the_gdh1_floor_follows_stein_and_stein() {
        assert_eq!(gdh1_depth_m(0.0), 2600.0);
        assert_eq!(gdh1_depth_m(-3.0), 2600.0, "a ridge younger than 0");
        assert!((gdh1_depth_m(10.0) - (2600.0 + 365.0 * 10f64.sqrt())).abs() < 1e-9);
        // The two branches meet at 20 Myr within half a metre (4,232.30 and 4,232.74 m: the
        // published coefficients' own step).
        let below = 2600.0 + 365.0 * 20f64.sqrt();
        assert!(
            (gdh1_depth_m(20.0) - below).abs() < 0.5,
            "{}",
            gdh1_depth_m(20.0)
        );
        assert!((gdh1_depth_m(100.0) - (5651.0 - 2473.0 * (-2.78f64).exp())).abs() < 1e-9);
        assert!(gdh1_depth_m(1000.0) < GDH1_FLOOR_M && gdh1_depth_m(1000.0) > GDH1_FLOOR_M - 1.0);
        // Steps of a sample born at the ridge at 0 Myr.
        let rates = |t: f64| SurfaceRates {
            erosion_m: 0.0,
            damping_m: 0.0,
            accretion_m: 0.0,
            z_c: 10_000.0,
            z_t: -10_000.0,
            z_floor: -GDH1_FLOOR_M,
            z_ridge: -GDH1_RIDGE_M,
            trench_relax: 0.0,
            trench_release: 0.0,
            gravity: 9.81,
            law: OceanDepth::Gdh1,
            t_myr: t,
            dt_myr: 1.0,
            sea_m: 0.0,
        };
        let run = |z0: f32, sea: f64| {
            let mut s = Sample::blank(0);
            s.elevation_m = z0;
            for k in 1..=100 {
                let r = SurfaceRates {
                    sea_m: sea,
                    ..rates(f64::from(k))
                };
                surface_step(&mut s, &r, f32::NAN);
            }
            f64::from(s.elevation_m)
        };
        let bare = run(-2600.0, 0.0);
        assert!((bare + gdh1_depth_m(100.0)).abs() < 0.5, "{bare}");
        // A seamount 1,600 m high keeps its height over the floor.
        let mount = run(-1000.0, 0.0);
        assert!((mount - bare - 1600.0).abs() < 0.5, "{mount} vs {bare}");
        // The same seamount under a sea 6 km down stands dry the whole way and sinks by 1/W.
        let dry = run(-1000.0, -6000.0);
        let sink = gdh1_depth_m(100.0) - 2600.0;
        let expect = -1000.0 - sink / super::super::settle::WATER_LOADED;
        assert!((dry - expect).abs() < 0.5, "{dry} vs {expect}");
        // GDH1's shape between other depths (PLT-355): a ridge at -2,000 m and an old floor at
        // -6,000 m keep GDH1's ages; at GDH1's own depths the floor is GDH1 exactly.
        let k = TectonicParams {
            ocean_depth: OceanDepth::Gdh1,
            ridge_elevation_m: -GDH1_RIDGE_M,
            oceanic_floor_elevation_m: -GDH1_FLOOR_M,
            ..TectonicParams::default()
        };
        for t in [0.0, 10.0, 50.0, 300.0] {
            assert_eq!(floor_at(&k, t), -gdh1_depth_m(t));
        }
        let k = TectonicParams {
            ridge_elevation_m: -2000.0,
            oceanic_floor_elevation_m: -6000.0,
            ..k
        };
        assert_eq!(floor_at(&k, 0.0), -2000.0);
        assert!((floor_at(&k, 2000.0) + 6000.0).abs() < 1.0);
        let half = (gdh1_depth_m(30.0) - GDH1_RIDGE_M) / (GDH1_FLOOR_M - GDH1_RIDGE_M);
        assert!((floor_at(&k, 30.0) - (-2000.0 - 4000.0 * half)).abs() < 1e-9);
        let scaled = SurfaceRates {
            z_ridge: -2000.0,
            z_floor: -6000.0,
            ..rates(30.0)
        };
        let mut s = Sample::blank(0);
        s.elevation_m = floor_at(&k, 29.0) as f32;
        surface_step(&mut s, &scaled, f32::NAN);
        assert!((f64::from(s.elevation_m) - floor_at(&k, 30.0)).abs() < 0.01);
    }

    /// The trench's release (PLT-443): under GDH1 a trench pulls its row down towards z_t, and
    /// once no trench holds it the floor rises back to its age's bare floor with the release
    /// on, and stays at the trench's depth with it off (the incremental subsidence keeps any
    /// offset); floor standing above its age's (a seamount) is not touched.
    #[test]
    fn a_trench_gives_its_floor_back_when_it_has_gone() {
        let rates = |t: f64, release: f64| SurfaceRates {
            erosion_m: 0.0,
            damping_m: 0.0,
            accretion_m: 0.0,
            z_c: 10_000.0,
            z_t: -10_000.0,
            z_floor: -GDH1_FLOOR_M,
            z_ridge: -GDH1_RIDGE_M,
            trench_relax: trench_share(1.0, 2.0),
            trench_release: release,
            gravity: 9.81,
            law: OceanDepth::Gdh1,
            t_myr: t,
            dt_myr: 1.0,
            sea_m: 0.0,
        };
        let run = |release: f64, z0: f32, held_myr: u32| {
            let mut s = Sample::blank(0);
            s.elevation_m = z0;
            for k in 1..=80 {
                if k <= held_myr {
                    s.flags |= TRENCH;
                } else {
                    s.flags &= !TRENCH;
                }
                surface_step(&mut s, &rates(f64::from(k), release), f32::NAN);
            }
            f64::from(s.elevation_m)
        };
        let floor = -gdh1_depth_m(80.0);
        let at_trench = run(trench_share(1.0, 2.0), -2600.0, 80);
        assert!(at_trench < -9_900.0, "{at_trench}");
        let kept = run(0.0, -2600.0, 40);
        assert!(
            kept < floor - 3_000.0,
            "off: the depth stays, {kept} vs {floor}"
        );
        let released = run(trench_share(1.0, 2.0), -2600.0, 40);
        assert!((released - floor).abs() < 1.0, "{released} vs {floor}");
        let mount = run(trench_share(1.0, 2.0), -1000.0, 0);
        assert!((mount - floor - 1600.0).abs() < 0.5, "{mount} vs {floor}");
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

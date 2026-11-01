// SPDX-License-Identifier: LGPL-3.0-or-later
//! A continental column's height from its whole buoyancy (`tectonics.column_buoyancy`, on by
//! default since 2026-11-01; `docs/design/continental-buoyancy.md`,
//! `docs/science/continental-buoyancy.md`; ledger PLT-830 to PLT-836).
//!
//! # Why
//! Every continental column stood on PLT-281's one Airy line, a crust of one density from the
//! normal column's height, whatever its lithosphere. Earth's do not: at equal crust a hot
//! back-arc stands about 1,600 m over a cold craton (Hyndman & Currie 2011), and the low-continents
//! diagnosis (sea-level.md, "Why base35's continents stand low") found the model's thick crust
//! 400 to 1,320 m over Earth's at 36 to 44 km. ISO-110 adds a thermal term to the one line but needs
//! a frame for the line's depth, which no reading of the model's sea gives (an open fault,
//! lithosphere-thermal-state.md); this law needs none.
//!
//! # The law
//! A column floats on the asthenosphere, which has no strength (Lachenbruch & Morgan 1990, as
//! Zoback & Mooney 2003 print their eq. 1): its dry height over the datum is the height of a free
//! asthenosphere, H_f, plus Σ (ρ_a − ρ) dz / ρ_a down the column to the depth where every column is
//! asthenosphere.
//!
//! - **The crust**: each pool's rock at its depth (Christensen & Mooney 1995, Table 4: laboratory
//!   densities at room temperature with the compression of depth), expanded by α T on the
//!   column's geotherm; the pools lie cover first, then the felsic crust, then the mafic pools at
//!   the bottom (as `pools::eclogite_m` reads them), whose share past the eclogite pressure weighs
//!   as eclogite (PLT-278 to PLT-280).
//! - **The mantle lithosphere**, from the Moho to L: ρ_m0 (1 − α T), so Σ (ρ_a − ρ) dz =
//!   −ρ_m0 α Σ (T_m − T) dz, of which the depleted keel (K km under the Moho) compensates
//!   [`KEEL_COMPENSATION`] (Kaban et al. 2003: "partly (about 40%) compensated").
//! - **The asthenosphere**: ρ_a = ρ_m0 (1 − α T_m).
//! - **The geotherm**: the plate model's steady conductive one (ISO-138: T_m at L, the crust's
//!   heat decaying over h_r, the mantle's B / L), with L never above the lid's floor (PLT-820,
//!   `thermal::continental_l_km`) and from the floor's age for crust with no state (PLT-821,
//!   `thermal::sample_l_km`).
//! - **H_f**, from the ridge: GDH1's crest, 2,600 m of water (PLT-352), over the ocean's 7 km of
//!   crust (PLT-104) at the mantle's temperature (the cooling plate's t = 0).
//!
//! α, ρ_m0 and T_m are one set, the ocean's own: Parsons & Sclater's plate as Stein & Stein (1992)
//! print it, 125 km thick (the model's plate, PLT-465) at 1,350 °C (ISO-138). From the same
//! constants a half-space subsides 363 √t m against GDH1's 365, and the law's 7 km ocean column on
//! the model's ocean lithosphere stands within 100 m of GDH1 from 1 to 30 Myr (the tests): the
//! ocean keeps GDH1, its own fit to depth and heat flow, since the model's oceanic L (PLT-466)
//! grows as √t to its plate's 125 km while GDH1's plate is 95 km.
//!
//! # Against Earth
//! On CRUST1.0's own crusts (in situ, m4cold.py's p50 layers per region and band), with the
//! cratons' state (L 225 km, K 200 km) and a hot back-arc's (L 60 km, no keel), the law gives the
//! East European, Siberian, Canadian, Australian and Brazilian cratons' modal band (40 to 45 km)
//! and the Basin and Range and Cordillera's 30 to 40 km bands within 0.35 km of their rock height
//! (the tests). It misses the Canadian shield's 35 to 40 km band (815 m low) and the Cordillera
//! box's 40 to 45 km band (836 m high: the Colorado Plateau and the Rockies, not a hot back-arc).
//! Its slopes with a uniform 2,850 kg/m³ crust, 126 m/km on the craton at its mean 40 km and 124
//! on the back-arc at its 33 km, lie within Hyndman & Currie's 141 ± 27 and 131 ± 12 m/km; at equal
//! crust the two stand 1,846 m apart against their ~1,600.

use super::pools::{
    ECLOGITE_PA, ECLOGITE_SHARE, GDH1_RIDGE_M, RHO_CRUST_MEAN, RHO_ECLOGITE, oceanic_column,
};
use super::state::{Sample, pool};
use super::thermal::{self, CRUST_HEAT_MW_M2, MANTLE_C, MANTLE_HEAT_MW_M2_KM, RADIOGENIC_DEPTH_KM};
use crate::surface::flexure::RHO_WATER;

/// The mantle's thermal expansion, 1/K, and its density at 0 °C, kg/m³: Parsons & Sclater's
/// plate, "coefficient of thermal expansion 3.28 × 10−5 K−1 ... mantle density 3,330 kg m−3"
/// (Stein & Stein 1992, pp. 124 to 125), with the basal 1,350 °C of ISO-138.
pub const ALPHA_PER_K: f64 = 3.28e-5; // ledger: PLT-830
pub const RHO_MANTLE_0: f64 = 3330.0; // ledger: PLT-830

/// The share of the keel's thermal density excess its depletion compensates: "the density
/// variations due to temperature are partly (about 40%) compensated by density variations due to
/// compositional differences" (Kaban et al. 2003, conclusions, pp. 66 to 67).
pub const KEEL_COMPENSATION: f64 = 0.4; // ledger: PLT-831

/// Each pool's rock, its density at 5 km (kg/m³) and its increase per km deeper (Christensen &
/// Mooney 1995, Table 4, laboratory densities corrected for compression): metamorphic as biotite
/// (tonalite) gneiss (2,746 at 5 km, 2,772 at 25), felsic plutonic as granite gneiss (2,662,
/// 2,687), felsic volcanic as granite-granodiorite (2,654 at 5 km, the gneiss's increase), mafic
/// volcanic as basalt (2,878 at 5 km) and mafic plutonic as gabbro-norite-troctolite (2,966 at 5
/// km), both with mafic granulite's increase (2,977 at 5 km, 2,998 at 25). The sediment pools are
/// [`porous_density`]'s.
// ledger: PLT-832
const ROCK: [(f64, f64); pool::COUNT] = [
    (0.0, 0.0),                         // sediment: porous_density
    (0.0, 0.0),                         // sedimentary: porous_density
    (2746.0, (2772.0 - 2746.0) / 20.0), // metamorphic
    (2662.0, (2687.0 - 2662.0) / 20.0), // felsic plutonic
    (2654.0, (2687.0 - 2662.0) / 20.0), // felsic volcanic
    (2878.0, (2998.0 - 2977.0) / 20.0), // mafic volcanic
    (2966.0, (2998.0 - 2977.0) / 20.0), // mafic plutonic
];

/// Sediment and sedimentary rock: shale's porosity φ_0 e^(−c z) with sea water in the pores,
/// φ_0 0.5, c 5.0 × 10⁻⁴ per m, grain density 2.72 (Angevine et al. 1990, Fig. 3.11, "based on
/// studies by Sclater and Christie (1980)"; the primary not read).
// ledger: PLT-833
const SHALE_POROSITY: f64 = 0.5;
const SHALE_DECAY_PER_KM: f64 = 0.5;
const SHALE_GRAIN: f64 = 2720.0;

/// The pools top to bottom: the cover, the felsic crust, the mafic pools.
const LAYERS: [usize; pool::COUNT] = [
    pool::SEDIMENT,
    pool::SEDIMENTARY,
    pool::FELSIC_VOLCANIC,
    pool::METAMORPHIC,
    pool::FELSIC_PLUTONIC,
    pool::MAFIC_VOLCANIC,
    pool::MAFIC_PLUTONIC,
];

/// The asthenosphere's density, kg/m³: ρ_m0 (1 − α T_m), 3,183.
pub fn rho_asthenosphere() -> f64 {
    RHO_MANTLE_0 * (1.0 - ALPHA_PER_K * MANTLE_C)
}

/// Sediment's or sedimentary rock's density at depth `z_km`, kg/m³ (see [`SHALE_POROSITY`]).
fn porous_density(z_km: f64) -> f64 {
    let phi = SHALE_POROSITY * (-SHALE_DECAY_PER_KM * z_km.max(0.0)).exp();
    (1.0 - phi) * SHALE_GRAIN + phi * RHO_WATER
}

/// A pool's rock density at depth `z_km` at 0 °C, kg/m³.
pub fn rock_density(p: usize, z_km: f64) -> f64 {
    match p {
        pool::SEDIMENT | pool::SEDIMENTARY => porous_density(z_km),
        _ => ROCK[p].0 + ROCK[p].1 * (z_km - 5.0),
    }
}

/// The geotherm of a lithosphere `l_km` deep at depth `z_km`, °C (ISO-138; T_m below L).
fn temperature_c(z_km: f64, l_km: f64) -> f64 {
    if z_km >= l_km {
        MANTLE_C
    } else {
        MANTLE_C * thermal::geotherm_share(l_km, z_km)
    }
}

/// ∫ T dz over [a, b] within [0, L], °C km: ISO-138's geotherm (`thermal::geotherm_share`)
/// integrated in closed form.
fn temperature_integral(a: f64, b: f64, l_km: f64) -> f64 {
    let l = l_km.max(1.0);
    let qm = MANTLE_HEAT_MW_M2_KM / l;
    let (qc, hr) = (CRUST_HEAT_MW_M2, RADIOGENIC_DEPTH_KM);
    let heat_l = qm * l + qc * hr * (1.0 - (-l / hr).exp());
    let heat_ab =
        qm * (b * b - a * a) / 2.0 + qc * hr * ((b - a) + hr * ((-b / hr).exp() - (-a / hr).exp()));
    MANTLE_C * heat_ab / heat_l
}

/// Gauss-Legendre's eight nodes and weights on [−1, 1], to 15 digits.
const GAUSS: [(f64, f64); 8] = [
    (-0.960_289_856_497_536, 0.101_228_536_290_376),
    (-0.796_666_477_413_627, 0.222_381_034_453_375),
    (-0.525_532_409_916_329, 0.313_706_645_877_887),
    (-0.183_434_642_495_65, 0.362_683_783_378_362),
    (0.183_434_642_495_65, 0.362_683_783_378_362),
    (0.525_532_409_916_329, 0.313_706_645_877_887),
    (0.796_666_477_413_627, 0.222_381_034_453_375),
    (0.960_289_856_497_536, 0.101_228_536_290_376),
];

/// ∫ f over [a, b].
fn quad(f: impl Fn(f64) -> f64, a: f64, b: f64) -> f64 {
    if b <= a {
        return 0.0;
    }
    let (m, h) = (0.5 * (a + b), 0.5 * (b - a));
    GAUSS.iter().map(|&(x, w)| w * f(m + h * x)).sum::<f64>() * h
}

/// Σ (ρ_a − ρ) dz over a crust, kg/m³ × km: its pools at their depths, expanded on the geotherm
/// `temp` (°C at a depth in km), and the eclogite root (PLT-278 to PLT-280).
fn crust_sum(pools: &[f32; pool::COUNT], gravity: f64, temp: &dyn Fn(f64) -> f64) -> f64 {
    let ra = rho_asthenosphere();
    let mut z = 0.0;
    let mut sum = 0.0;
    for &p in &LAYERS {
        let th = f64::from(pools[p].max(0.0)) / 1000.0;
        if th > 0.0 {
            sum += quad(
                |d| ra - rock_density(p, d) * (1.0 - ALPHA_PER_K * temp(d)),
                z,
                z + th,
            );
            z += th;
        }
    }
    // Of the mafic rock deeper than the eclogite depth (the column's bottom), the share
    // ECLOGITE_SHARE weighs as eclogite (`pools::eclogite_m`'s reading).
    let mafic =
        f64::from(pools[pool::MAFIC_PLUTONIC].max(0.0) + pools[pool::MAFIC_VOLCANIC].max(0.0))
            / 1000.0;
    let depth = ECLOGITE_PA / (RHO_CRUST_MEAN * gravity.max(1e-3)) / 1000.0;
    let deep = mafic.min((z - depth).max(0.0));
    if deep > 0.0 {
        sum -= ECLOGITE_SHARE
            * quad(
                |d| {
                    (RHO_ECLOGITE - rock_density(pool::MAFIC_PLUTONIC, d))
                        * (1.0 - ALPHA_PER_K * temp(d))
                },
                z - deep,
                z,
            );
    }
    sum
}

/// Σ (ρ_a − ρ) dz over a mantle lithosphere from the Moho (`moho_km`) to L (`l_km`, at least the
/// Moho), with a keel `keel_km` thick under the Moho, kg/m³ × km.
fn mantle_sum(moho_km: f64, l_km: f64, keel_km: f64) -> f64 {
    let l = l_km.max(moho_km);
    let deficit = |a: f64, b: f64| MANTLE_C * (b - a) - temperature_integral(a, b, l);
    let lid = deficit(moho_km, l);
    let keel_base = l.min(moho_km + keel_km.max(0.0));
    let keel = if keel_base > moho_km {
        deficit(moho_km, keel_base)
    } else {
        0.0
    };
    -RHO_MANTLE_0 * ALPHA_PER_K * (lid - KEEL_COMPENSATION * keel)
}

/// The dry height of a free asthenosphere over the datum, m: the ridge's column balanced, GDH1's
/// 2,600 m of water over the ocean's crust at T_m (−2,580 m, so H_0 = 2.58 km against Zoback &
/// Mooney's 2.78 ± 0.35).
// ledger: PLT-830, PLT-352, PLT-104
pub fn free_asthenosphere_m(gravity: f64) -> f64 {
    let ra = rho_asthenosphere();
    let ridge = crust_sum(&oceanic_column(), gravity, &|_| MANTLE_C);
    -GDH1_RIDGE_M * (ra - RHO_WATER) / ra - ridge / ra * 1000.0
}

/// The law at one gravity, with its free asthenosphere's height computed once.
#[derive(Clone, Copy, Debug)]
pub struct Law {
    gravity: f64,
    free_m: f64,
}

impl Law {
    pub fn new(gravity: f64) -> Law {
        Law {
            gravity,
            free_m: free_asthenosphere_m(gravity),
        }
    }

    /// A continental column's dry isostatic height over the datum, m, from its pools, its
    /// lithosphere's depth (`l_km`, never above its lid's floor, PLT-820) and its keel.
    pub fn height_m(&self, pools: &[f32; pool::COUNT], l_km: f64, keel_km: f64) -> f64 {
        let moho = f64::from(pools.iter().map(|&x| x.max(0.0)).sum::<f32>()) / 1000.0;
        let l = thermal::continental_l_km(l_km, moho);
        let crust = crust_sum(pools, self.gravity, &|d| temperature_c(d, l));
        self.free_m + (crust + mantle_sum(moho, l, keel_km)) / rho_asthenosphere() * 1000.0
    }

    /// The same for crust given as in-situ layers (thickness km, density kg/m³, top first:
    /// CRUST1.0's own), with no thermal expansion of the crust (its densities are in situ).
    pub fn layers_height_m(&self, layers: &[(f64, f64)], l_km: f64, keel_km: f64) -> f64 {
        let ra = rho_asthenosphere();
        let crust: f64 = layers.iter().map(|&(th, rho)| (ra - rho) * th).sum();
        let moho: f64 = layers.iter().map(|&(th, _)| th).sum();
        let l = thermal::continental_l_km(l_km, moho);
        self.free_m + (crust + mantle_sum(moho, l, keel_km)) / ra * 1000.0
    }

    /// A sample's dry isostatic height over the datum, m: its pools on the lithosphere its laws
    /// read (`thermal::sample_l_km`: its own, floored, or its floor's for crust with no state)
    /// and its keel.
    pub fn sample_m(&self, s: &Sample, t_myr: f64) -> f64 {
        self.height_m(
            &s.pools,
            thermal::sample_l_km(s, t_myr),
            f64::from(s.keel_km),
        )
    }

    /// The thickness of the start's column ([`start_column`]) whose dry height is `dry_m` on a
    /// lithosphere `l_km` deep with a keel `keel_km`, m: bisection between 10 and 120 km to 0.1 m.
    // ledger: PLT-835
    pub fn start_thickness_m(&self, dry_m: f64, l_km: f64, keel_km: f64) -> f64 {
        let (mut lo, mut hi) = (10_000.0f64, 120_000.0f64);
        while hi - lo > 0.1 {
            let mid = 0.5 * (lo + hi);
            if self.height_m(&start_column(mid), l_km, keel_km) < dry_m {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }
}

/// [`Law::height_m`] at `gravity`.
pub fn height_m(pools: &[f32; pool::COUNT], l_km: f64, keel_km: f64, gravity: f64) -> f64 {
    Law::new(gravity).height_m(pools, l_km, keel_km)
}

/// The start's continental column at `thickness_m`: Rudnick & Gao's three layers as Hacker,
/// Kelemen & Behn (2015, Fig. 1) print them, upper crust 0 to 12 km, middle 12 to 23, lower 23 to
/// 40, the lower crust 80 % mafic, scaled: per 40 km, sediment 0.1 km and sedimentary rock 2.0 (the
/// cover PLT-131 gave the start), felsic plutonic 9.9 (the rest of the upper crust), metamorphic
/// 14.4 (the middle crust and the lower crust's felsic fifth) and mafic plutonic 13.6.
// ledger: PLT-834
pub fn start_column(thickness_m: f64) -> [f32; pool::COUNT] {
    let f = thickness_m.max(0.0) / 40_000.0;
    let mut p = [0.0f32; pool::COUNT];
    p[pool::SEDIMENT] = (100.0 * f) as f32;
    p[pool::SEDIMENTARY] = (2000.0 * f) as f32;
    p[pool::FELSIC_PLUTONIC] = (9900.0 * f) as f32;
    p[pool::METAMORPHIC] = (14_400.0 * f) as f32;
    p[pool::MAFIC_PLUTONIC] = (13_600.0 * f) as f32;
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plates::{pools, settle};

    const G: f64 = 9.81;
    /// Kaban et al. 2003's error of the cratons' residual topography, "about 0.35 km": the
    /// tolerance of a column's height against Earth's here.
    const KABAN_ERROR_M: f64 = 350.0;
    /// The cratons' state (PLT-462, PLT-463) and a hot back-arc's (PLT-461), L and K in km.
    const CRATON: (f64, f64) = (225.0, 200.0);
    const BACK_ARC: (f64, f64) = (60.0, 0.0);

    #[test]
    fn the_free_asthenosphere_stands_where_lachenbruch_and_morgan_put_it() {
        // H_0 = 2.78 ± 0.35 km (Zoback & Mooney 2003, eq. 3, the mean of their Table 1).
        let h0 = -free_asthenosphere_m(G) / 1000.0;
        assert!((h0 - 2.78).abs() <= 0.35, "{h0}");
        assert!((h0 - 2.580).abs() < 0.005, "law.py's 2.58: {h0}");
        assert!((rho_asthenosphere() - 3182.6).abs() < 0.1);
    }

    #[test]
    fn the_same_constants_subside_a_half_space_as_gdh1() {
        // 2 ρ_m α T_m / (ρ_m − ρ_w) √(κ t / π) (Stein & Stein 1992, p. 124) with the model's κ
        // (PLT-465), against GDH1's 365 √t m (PLT-352).
        let kappa =
            thermal::PLATE_KM.powi(2) / (std::f64::consts::PI.powi(2) * thermal::PLATE_TAU_MYR);
        let coef = 2.0 * RHO_MANTLE_0 * ALPHA_PER_K * MANTLE_C / (RHO_MANTLE_0 - RHO_WATER)
            * (kappa * 1.0e6 / std::f64::consts::PI).sqrt();
        assert!((coef / 365.0 - 1.0).abs() < 0.01, "{coef}");
    }

    #[test]
    fn the_ocean_column_meets_gdh1_on_young_floor() {
        // The law's 7 km of ocean crust on the model's ocean lithosphere (PLT-466) under the water
        // load, against GDH1 (PLT-352) from 1 to 30 Myr; older, the model's L outgrows GDH1's plate.
        for age in [1.0, 10.0, 30.0] {
            let l = thermal::ocean_km(age);
            let dry = height_m(&oceanic_column(), l, 0.0, G);
            let wet = settle::equilibrium_m(dry, 0.0);
            let gdh1 = -pools::gdh1_depth_m(age);
            assert!(
                (wet - gdh1).abs() < 100.0,
                "age {age}: {wet} against {gdh1}"
            );
        }
    }

    /// CRUST1.0's own crusts (in situ; m4cold.py's p50 sediment, upper, middle and lower crust per
    /// region and band, km and kg/m³) and their rock p50 over the sea, m.
    type Crust = (&'static str, [(f64, f64); 4], f64);
    const CRATONS: [Crust; 5] = [
        (
            "East European platform 40-45",
            [
                (1.5, 2283.0),
                (13.3, 2740.0),
                (14.5, 2830.0),
                (13.1, 2920.0),
            ],
            150.0,
        ),
        (
            "Siberian platform 40-45",
            [
                (2.8, 2400.0),
                (12.5, 2740.0),
                (13.9, 2830.0),
                (12.7, 2920.0),
            ],
            360.0,
        ),
        (
            "Canadian shield 40-45",
            [
                (0.0, 2110.0),
                (12.8, 2760.0),
                (14.9, 2810.0),
                (14.1, 2910.0),
            ],
            310.0,
        ),
        (
            "Australia 40-45",
            [
                (0.6, 2350.0),
                (13.2, 2740.0),
                (14.6, 2830.0),
                (13.3, 2920.0),
            ],
            340.0,
        ),
        (
            "Brazilian shield 40-45",
            [
                (0.7, 2226.0),
                (13.1, 2740.0),
                (14.1, 2830.0),
                (13.1, 2920.0),
            ],
            320.0,
        ),
    ];
    const BACK_ARCS: [Crust; 2] = [
        (
            "Basin and Range, Cordillera 30-35",
            [
                (0.1, 2110.0),
                (11.2, 2740.0),
                (11.0, 2780.0),
                (10.4, 2860.0),
            ],
            1407.0,
        ),
        (
            "Basin and Range, Cordillera 35-40",
            [(0.0, 2110.0), (14.7, 2740.0), (13.0, 2780.0), (8.1, 2950.0)],
            1606.0,
        ),
    ];

    #[test]
    fn earths_cratons_and_back_arc_stand_at_their_heights_on_crust1s_own_crusts() {
        let law = Law::new(G);
        for (name, layers, earth) in CRATONS {
            let z = law.layers_height_m(&layers, CRATON.0, CRATON.1);
            assert!(
                (z - earth).abs() <= KABAN_ERROR_M,
                "{name}: {z} against {earth}"
            );
        }
        for (name, layers, earth) in BACK_ARCS {
            let z = law.layers_height_m(&layers, BACK_ARC.0, BACK_ARC.1);
            assert!(
                (z - earth).abs() <= KABAN_ERROR_M,
                "{name}: {z} against {earth}"
            );
        }
    }

    #[test]
    fn the_slopes_and_the_gap_are_hyndman_and_curries() {
        // A uniform crust of Hasterok & Chapman's reference density, 2,850 kg/m³: elevation rises
        // 141 ± 27 m per km of crust on stable areas and 131 ± 12 on the Cordillera, whose crusts
        // average 40 ± 4 and 33 ± 5 km, and the two stand about 1,600 m apart at equal crust
        // (Hyndman & Currie 2011, pp. 6 to 8). Each slope is read at its setting's mean crust.
        let law = Law::new(G);
        let at = |t: f64, (l, k): (f64, f64)| law.layers_height_m(&[(t, 2850.0)], l, k);
        let slope = |s: (f64, f64), mid: f64| (at(mid + 2.5, s) - at(mid - 2.5, s)) / 5.0;
        let (craton, back_arc) = (slope(CRATON, 40.0), slope(BACK_ARC, 33.0));
        assert!((craton - 141.0).abs() <= 27.0, "{craton}");
        assert!((back_arc - 131.0).abs() <= 12.0, "{back_arc}");
        let gap = at(35.0, BACK_ARC) - at(35.0, CRATON);
        assert!(
            (gap - 1600.0).abs() <= KABAN_ERROR_M,
            "about 1,600 (the law gives 1,846): {gap}"
        );
    }

    #[test]
    fn the_start_column_has_earths_crustal_density() {
        // Its mean density at 0 °C between CRUST1.0's cratons' (2,776 to 2,832, m4cold.py) and
        // under Christensen & Mooney's whole-crust 2,830 (their abstract).
        let p = start_column(40_000.0);
        let mut z = 0.0;
        let mut mass = 0.0;
        for &k in &LAYERS {
            let th = f64::from(p[k]) / 1000.0;
            mass += quad(|d| rock_density(k, d), z, z + th);
            z += th;
        }
        let mean = mass / z;
        assert!((z - 40.0).abs() < 1e-3);
        assert!((2776.0..=2832.0).contains(&mean), "{mean}");
    }

    #[test]
    fn the_start_column_stands_where_earths_does() {
        // On a craton at Hacker et al.'s shields and platforms' 38.8 km (CRUST1.0, their Table 1)
        // within Kaban's error of the cratons' land p50 (150 to 360 m, the median 290); on a hot
        // back-arc at Hyndman & Currie's Cordillera 33 km within it of the 30 to 35 km band's 1,407.
        let law = Law::new(G);
        let craton = law.height_m(&start_column(38_800.0), CRATON.0, CRATON.1);
        assert!((craton - 290.0).abs() <= KABAN_ERROR_M, "{craton}");
        let back_arc = law.height_m(&start_column(33_000.0), BACK_ARC.0, BACK_ARC.1);
        assert!((back_arc - 1407.0).abs() <= KABAN_ERROR_M, "{back_arc}");
        // law.py's numbers (tools/review/buoyancy/law.py model).
        let craton_40 = law.height_m(&start_column(40_000.0), CRATON.0, CRATON.1);
        assert!((craton_40 - 628.0).abs() < 3.0, "{craton_40}");
        let back_arc_35 = law.height_m(&start_column(35_000.0), BACK_ARC.0, BACK_ARC.1);
        assert!((back_arc_35 - 2028.0).abs() < 3.0, "{back_arc_35}");
    }

    #[test]
    fn the_start_thickness_inverts_the_law() {
        let law = Law::new(G);
        for dry in [-89.0, 0.0, 150.0, 525.0, 900.0] {
            let t = law.start_thickness_m(dry, CRATON.0, CRATON.1);
            let z = law.height_m(&start_column(t), CRATON.0, CRATON.1);
            assert!((z - dry).abs() < 1.0, "{dry}: {z} at {t}");
        }
        // law.py: 35.7 km at 0 m, 39.3 km at 525 m.
        assert!((law.start_thickness_m(0.0, CRATON.0, CRATON.1) - 35_700.0).abs() < 100.0);
        assert!((law.start_thickness_m(525.0, CRATON.0, CRATON.1) - 39_300.0).abs() < 100.0);
    }

    #[test]
    fn a_hot_column_stands_higher_than_a_cold_one_and_a_keel_raises_it() {
        let law = Law::new(G);
        let p = start_column(38_000.0);
        let hot = law.height_m(&p, 60.0, 0.0);
        let cold = law.height_m(&p, 225.0, 0.0);
        let keeled = law.height_m(&p, 225.0, 200.0);
        assert!(hot > keeled && keeled > cold, "{hot} {keeled} {cold}");
        // The lid's floor: a lithosphere under 1.25 times the crust reads as the floor (PLT-820).
        assert_eq!(
            law.height_m(&p, 10.0, 0.0),
            law.height_m(&p, thermal::lid_floor_km(38.0), 0.0)
        );
    }

    #[test]
    fn the_closed_form_temperature_integral_matches_the_geotherm() {
        for l in [40.0, 60.0, 120.0, 225.0] {
            let (a, b) = (0.3 * l, 0.9 * l);
            let numeric = quad(|z| temperature_c(z, l), a, b);
            let closed = temperature_integral(a, b, l);
            assert!(
                (numeric - closed).abs() < 1e-6 * closed,
                "{l}: {numeric} {closed}"
            );
        }
    }
}

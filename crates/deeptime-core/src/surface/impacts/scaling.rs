// SPDX-License-Identifier: LGPL-3.0-or-later
//! Impact scaling laws (P2-08): how often impactors arrive, how big they are, and the crater each
//! makes. Everything is SI (metres, seconds, kilograms, joules) unless a name says otherwise.
//!
//! Every constant carries its ledger id (`docs/science/ledger/IMP.md`; page `docs/science/impacts.md`)
//! and the status it has there. Equation numbers are those of Collins, Melosh & Marcus 2005
//! (Meteoritics & Planet. Sci. 40(6):817–840, "CMM05"), read in full text in stage 2 (2026-10-02).
//!
//! # Sources
//! - **Rate.** Grieve & Shoemaker 1994 ("The record of past impacts on Earth", in Gehrels (ed.),
//!   *Hazards due to Comets and Asteroids*, Univ. Arizona Press, 417–462): (5.6 ± 2.8) × 10⁻¹⁵
//!   craters of D ≥ 20 km per km² per year on Earth, from the well-dated craters of the North
//!   American and European cratons. The chapter itself was not readable; the value was read as
//!   quoted by Hughes 2000 (MNRAS 317:429–437, p. 434). Above 20 km the cumulative size slope is
//!   1.8 (N(≥ D) ∝ D⁻¹·⁸: Grieve 1987, Ann. Rev. Earth Planet. Sci. 15:245–270). **Below
//!   20 km the record does not follow that power law** (Hughes 2000; Bland & Artemieva 2006,
//!   Meteoritics & Planet. Sci. 41:607–631, Fig. 1: "departure … for terrestrial craters
//!   < 10–20 km in diameter"), so the production function takes its shape there from Hughes
//!   2000's rates for D > 2.4, 5, 12 and 20 km ([`SMALL_CRATER_RATES`]).
//! - **Crater from impactor.** CMM05, which gathers the π-group transient-crater scaling of
//!   Holsapple & Schmidt 1982, Schmidt & Housen 1987 and Gault 1974 (eq. 21), the collapse to a
//!   simple crater and its breccia lens after Grieve & Garvin 1984 (eqs. 22–26), the complex-crater
//!   enlargement of McKinnon & Schenk 1985 (eq. 27), the depth of complex craters from Herrick et
//!   al. 1997's Venus data (eq. 28), the melt volume (eqs. 29–31) and the ejecta thickness (eq. 47,
//!   McGetchin et al. 1973's r⁻³; rim height eq. 48).
//! - **Simple-to-complex transition** 3.2 km on Earth (CMM05 p. 823, after Dence 1965; Grieve 1987
//!   gives about 2 km in sedimentary and 4 km in crystalline targets), ∝ 1/g (Pike 1980, not read).
//! - **Peak rings** from ~40 km on Venus (Alexopoulos & McKinnon 1994, not read), also ∝ 1/g; the
//!   ring's diameter is half the rim's and a central peak's base is 20–25 % of it (Pike 1985,
//!   Meteoritics 20:49–68, abstract and Table 1, read from a scan by OCR).
//! - **Oceans.** Davison & Collins 2007 (Meteoritics & Planet. Sci. 42(11):1915–1927), full text
//!   read: the final crater in the sea floor is 1 − 0.14 R of the dry one (their eq. 3,
//!   R = water depth / impactor diameter); no crater at R > 8; a dry-complex crater (D > 3.2 km)
//!   at R > 3.5 becomes an "impact-related resurge feature", not a crater ([`sea_floor_factor`]).

/// Earth's surface gravity, m/s².
pub const G_EARTH: f64 = 9.81; // ledger: IMP-025 (uncited; the standard value)
/// Venus's surface gravity, m/s².
pub const G_VENUS: f64 = 8.87; // ledger: IMP-026 (uncited; NASA Venus fact sheet value)
/// Craters of at least 20 km formed per km² per year on Earth (Grieve & Shoemaker 1994, as quoted
/// by Hughes 2000; Hughes's own estimate is 3.46 ± 0.30, the older ones 2.6 to 5.6).
pub const EARTH_RATE_20KM_PER_KM2_YR: f64 = 5.6e-15; // ledger: IMP-001
/// The diameter that rate is for, m.
pub const RATE_DIAMETER_M: f64 = 20_000.0; // ledger: IMP-002
/// Hughes 2000's rates of craters larger than D (D in m, rate in 10⁻¹⁵ km⁻² yr⁻¹; the abstract
/// and p. 434): they give the shape of the production function below 20 km, scaled to the rate at
/// 20 km above (Hughes's 3.46 ± 0.30 is lower than Grieve & Shoemaker's 5.6 ± 2.8 at every size).
const SMALL_CRATER_RATES: [(f64, f64); 4] = [
    (2_400.0, 12.0),
    (5_000.0, 9.5),
    (12_000.0, 6.5),
    (20_000.0, 3.46),
]; // ledger: IMP-004
/// Simple-to-complex transition on Earth, m (final rim diameter).
pub const TRANSITION_EARTH_M: f64 = 3200.0; // ledger: IMP-034
/// Peak-ring onset on Venus, m.
pub const PEAK_RING_VENUS_M: f64 = 40_000.0; // ledger: IMP-045
/// Impact angle from the horizontal the rate is turned into impactors with: the most probable
/// one (CMM05 p. 817: "The most likely impact angle is 45° (Shoemaker 1962)").
pub const ANGLE_RAD: f64 = std::f64::consts::FRAC_PI_4; // ledger: IMP-024
/// Slope of the final crater diameter against R = water depth / impactor diameter: the sea floor
/// crater is `1 − 0.14 R` of the dry one (Davison & Collins 2007, eq. 3).
pub const OCEAN_SLOPE: f64 = 0.14; // ledger: IMP-065
/// R above which a crater that would be complex on land is an "impact-related resurge feature"
/// and not a crater (Davison & Collins 2007, p. 1922: the model flags it at 3.5).
pub const RESURGE_RATIO: f64 = 3.5; // ledger: IMP-065
/// The water depth, in impactor diameters, beyond which no crater forms in the sea floor
/// (Davison & Collins 2007: "at water thicknesses in excess of ~8 L, no crater forms").
pub const SEA_CUTOFF_RATIO: f64 = 8.0; // ledger: IMP-066

/// What the scaling needs of the planet and the impactor population.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Setting {
    /// Surface gravity, m/s².
    pub gravity: f64,
    /// Impact speed, m/s.
    pub speed: f64,
    /// Impactor density, kg/m³.
    pub impactor_density: f64,
    /// Target density, kg/m³.
    pub target_density: f64,
    /// Water deeper than this many impactor diameters leaves no crater in the sea floor
    /// ([`sea_floor_factor`]).
    pub sea_cutoff: f64,
}

impl Setting {
    /// The same impactors on Earth.
    pub fn on_earth(&self) -> Setting {
        Setting {
            gravity: G_EARTH,
            ..*self
        }
    }

    /// Simple-to-complex transition diameter (final rim diameter), m.
    pub fn transition_m(&self) -> f64 {
        TRANSITION_EARTH_M * G_EARTH / self.gravity
    }

    /// Final diameter from which a crater has a peak ring instead of a central peak, m.
    pub fn peak_ring_m(&self) -> f64 {
        PEAK_RING_VENUS_M * G_VENUS / self.gravity
    }

    /// Transient crater diameter of an impactor `l` m across (CMM05 eq. 21, "applies … for all
    /// terrestrial impacts larger than a couple of hundred meters"; the smallest impactors here
    /// are about 80 m): D_tc = 1.161 (ρ_i/ρ_t)^⅓ L^0.78 v^0.44 g^−0.22 sin^⅓θ.
    pub fn transient_m(&self, l: f64) -> f64 {
        // ledger: IMP-030
        1.161
            * (self.impactor_density / self.target_density).cbrt()
            * l.powf(0.78)
            * self.speed.powf(0.44)
            * self.gravity.powf(-0.22)
            * ANGLE_RAD.sin().cbrt()
    }

    /// The impactor diameter that makes a transient crater `d_tc` m across (the inverse).
    pub fn impactor_m(&self, d_tc: f64) -> f64 {
        let k = self.transient_m(1.0);
        (d_tc / k).powf(1.0 / 0.78)
    }

    /// Final rim-to-rim diameter from the transient one (CMM05 eqs. 22 and 27): simple 1.25 D_tc;
    /// complex 1.17 D_tc^1.13 / D_c^0.13. CMM05 switches to the complex law at D_tc = 2.56 km,
    /// where it gives 2.9 km against the simple law's 3.2 km (a step down); this takes the larger
    /// of the two, which is continuous and increasing and agrees with the paper away from the
    /// transition (the two meet at D_tc ≈ 1.66 D_c). Game choice, ledger IMP-033.
    pub fn final_m(&self, d_tc: f64) -> f64 {
        let dc = self.transition_m();
        (1.25 * d_tc).max(1.17 * d_tc.powf(1.13) / dc.powf(0.13)) // ledger: IMP-031, IMP-032
    }

    /// The transient diameter of a final one (the inverse of [`Setting::final_m`]).
    pub fn transient_of_final_m(&self, d: f64) -> f64 {
        let dc = self.transition_m();
        (d / 1.25).min((d * dc.powf(0.13) / 1.17).powf(1.0 / 1.13))
    }

    /// Kinetic energy of an impactor `l` m across, J (CMM05 eq. 1: π/12 ρ_i L³ v²).
    pub fn energy_j(&self, l: f64) -> f64 {
        std::f64::consts::PI / 12.0 * self.impactor_density * l.powi(3) * self.speed * self.speed
    }

    /// Impact melt volume, m³ (CMM05 eq. 30: 8.9 × 10⁻¹² E sin θ; eq. 29 for granite at
    /// ε_m = 5.2 MJ/kg and densities of 2,700 kg/m³, vertical, scaled by sin θ).
    pub fn melt_m3(&self, l: f64) -> f64 {
        8.9e-12 * self.energy_j(l) * ANGLE_RAD.sin() // ledger: IMP-062
    }
}

/// How a crater's interior is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Morphology {
    /// A bowl.
    Simple,
    /// A flat floor, terraced walls and a central peak.
    Complex,
    /// A flat floor with a ring of peaks at half the rim's diameter.
    PeakRing,
}

impl Morphology {
    pub fn name(self) -> &'static str {
        match self {
            Morphology::Simple => "simple",
            Morphology::Complex => "complex",
            Morphology::PeakRing => "peak_ring",
        }
    }
}

/// The dimensions of a fresh crater, m (real metres, before any game scaling).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crater {
    pub morphology: Morphology,
    pub impactor_m: f64,
    pub transient_m: f64,
    /// Rim-to-rim diameter.
    pub diameter_m: f64,
    /// Rim crest to floor.
    pub depth_m: f64,
    /// Rim crest above the ground before the impact.
    pub rim_m: f64,
    /// Thickness of the breccia lens under the floor at the centre.
    pub breccia_m: f64,
    /// Thickness of the melt sheet on the floor (0 when it would be a film).
    pub melt_m: f64,
    pub melt_m3: f64,
    /// Depth of excavation at the centre (a third of the transient depth: Melosh 1989, not read;
    /// ledger IMP-053).
    pub excavation_m: f64,
    /// Central peak or peak ring height above the floor.
    pub peak_m: f64,
    pub energy_j: f64,
}

/// The crater an impactor `l` m across makes where the sea is `water_m` deep (0 on land), or
/// `None` when the sea leaves no crater (the impactor is stopped, or the impact makes a resurge
/// feature instead of a crater: [`sea_floor_factor`]).
pub fn crater(s: &Setting, l: f64, water_m: f64) -> Option<Crater> {
    // The crater on dry land, then the sea floor's: its final diameter is the dry one times the
    // factor of Davison & Collins 2007, and the transient crater is the one that makes it.
    let dry_tc = s.transient_m(l);
    let dry_d = s.final_m(dry_tc);
    let dc = s.transition_m();
    let f = sea_floor_factor(water_m, l, s.sea_cutoff, dry_d >= dc);
    if f <= 0.0 {
        return None;
    }
    let (d_tc, d) = if f < 1.0 {
        (s.transient_of_final_m(dry_d * f), dry_d * f)
    } else {
        (dry_tc, dry_d)
    };
    // Rim height (CMM05 eq. 48): 0.07 D_tc⁴ / D_fr³. The paper derives it for simple craters; the
    // rim of a complex crater is lower, because the thickest ejecta slumps back into the crater
    // (CMM05 p. 829), so this overstates it there.
    let rim = 0.07 * d_tc.powi(4) / d.powi(3); // ledger: IMP-038
    // Transient depth (CMM05 eq. 25, after Dence et al. 1977).
    let transient_depth = d_tc / (2.0 * std::f64::consts::SQRT_2); // ledger: IMP-037
    // Melt follows the transient crater's volume (CMM05 p. 825: "roughly proportional").
    let melt_m3 = s.melt_m3(l) * (d_tc / dry_tc).powi(3);
    // Melt sheet (CMM05 eq. 31): the melt spread over the transient crater's floor. Under ~10 m
    // it is pods and glass in the breccia, not a sheet (design value, ledger IMP-061).
    let sheet = 4.0 * melt_m3 / (std::f64::consts::PI * d_tc * d_tc); // ledger: IMP-063
    let melt = if sheet >= 10.0 { sheet } else { 0.0 };
    let (morphology, depth, breccia, peak) = if d < dc {
        // Breccia lens (CMM05 eqs. 23, 24 and 26): V_br = 0.032 D_fr³ as a lens,
        // t_br = 2.8 V_br (d_tc + h_fr) / (d_tc D_fr²); the final depth is the transient depth
        // plus the rim less the lens: about D / 4.7.
        let v_br = 0.032 * d.powi(3); // ledger: IMP-039
        let t_br = 2.8 * v_br * (transient_depth + rim) / (transient_depth * d * d); // IMP-040
        (Morphology::Simple, transient_depth + rim - t_br, t_br, 0.0) // ledger: IMP-041
    } else {
        // Complex depth (CMM05 eq. 28, from Herrick et al. 1997's Venus craters): d = 0.4 D^0.3,
        // both in km, at Earth's gravity (Venus's is 10 % lower). The scaling with gravity is
        // this code's own: D in Earth-gravity terms, depth ∝ 1/g (ledger IMP-044, uncited). The
        // cap at a fifth of the diameter (IMP-043) never binds at Earth's gravity for D ≥ 3.2 km.
        let km = d / 1000.0 * s.gravity / G_EARTH;
        let depth = (COMPLEX_DEPTH_M_AT_1_KM * km.powf(COMPLEX_DEPTH_EXPONENT) * G_EARTH
            / s.gravity)
            .min(0.2 * d);
        let ring = d >= s.peak_ring_m();
        // Peaks stand about half the floor's depth (design value, ledger IMP-049; Pike 1977's
        // lunar peaks, 0.0006 D^1.97 km, not read), a ring lower.
        let peak = if ring { 0.35 * depth } else { 0.5 * depth };
        // The allochthonous breccia and melt-bearing fill of complex craters is about as thick
        // as the crater is deep (Grieve, Dence & Robertson 1977, not read; ledger IMP-052).
        (
            if ring {
                Morphology::PeakRing
            } else {
                Morphology::Complex
            },
            depth,
            depth,
            peak,
        )
    };
    Some(Crater {
        morphology,
        impactor_m: l,
        transient_m: d_tc,
        diameter_m: d,
        depth_m: depth,
        rim_m: rim,
        breccia_m: breccia,
        melt_m: melt,
        melt_m3,
        excavation_m: transient_depth / 3.0,
        peak_m: peak,
        energy_j: s.energy_j(l),
    })
}

/// Depth of a complex crater of 1 km at Earth's gravity, m: CMM05 eq. 28, d = 0.4 D^0.3 (km).
pub const COMPLEX_DEPTH_M_AT_1_KM: f64 = 400.0; // ledger: IMP-042
/// The exponent of that law.
pub const COMPLEX_DEPTH_EXPONENT: f64 = 0.3; // ledger: IMP-042

/// The share of its dry diameter a crater keeps in the sea floor under `water_m` of water, for an
/// impactor `l` m across, from Davison & Collins 2007 (iSALE models of vertical 15 km/s impacts of
/// stony bodies into granite under water, 100 m, 500 m and 1 km across):
///
/// - R = water depth / impactor diameter; the final diameter is `1 − 0.14 R` of the dry one
///   (their eq. 3, a regression of the simple craters and of the complex ones at R < 3–4; R² 0.92),
///   reaching zero at R = 7.1;
/// - no crater at R ≥ `cutoff` (8: "at water thicknesses in excess of ~8 L, no crater forms");
/// - a crater that would be complex on land (`complex_on_land`) at R > 3.5 is not a crater: the
///   resurging water leaves a broad scoured "impact-related resurge feature" (their model flags it
///   there), which this stage does not draw. Returns 0 for these.
///
/// 1 on land. Their model is vertical and at 15 km/s; the paper says oblique impacts, which cross
/// more water, would filter more.
pub fn sea_floor_factor(water_m: f64, l: f64, cutoff: f64, complex_on_land: bool) -> f64 {
    if water_m <= 0.0 {
        return 1.0;
    }
    let r = water_m / l;
    if r >= cutoff || (complex_on_land && r > RESURGE_RATIO) {
        return 0.0;
    }
    (1.0 - OCEAN_SLOPE * r).max(0.0)
}

/// Ejecta thickness at `r` m from the centre, m (CMM05 eq. 47, after McGetchin et al. 1973's
/// r⁻³: D_tc⁴ / (112 r³)); it equals the rim's uplifted height at the rim.
pub fn ejecta_m(c: &Crater, r: f64) -> f64 {
    let rim = 0.5 * c.diameter_m;
    c.transient_m.powi(4) / (112.0 * r.max(rim).powi(3)) // ledger: IMP-054
}

/// The segments of the production function for a slope above 20 km: for each, its lower edge
/// (m), the rate there and its cumulative slope. Below 20 km the rate follows Hughes 2000's
/// shape (his 20-km rate taken as the reference, so that the rate at 20 km is the code's
/// [`EARTH_RATE_20KM_PER_KM2_YR`]); the first segment is extended down to any diameter.
fn production_segments(slope: f64) -> [(f64, f64, f64); 4] {
    let r20 = SMALL_CRATER_RATES[3].1;
    let at = |k: usize| EARTH_RATE_20KM_PER_KM2_YR * SMALL_CRATER_RATES[k].1 / r20;
    let b = |k: usize| {
        let (d0, f0) = SMALL_CRATER_RATES[k];
        let (d1, f1) = SMALL_CRATER_RATES[k + 1];
        (f0 / f1).ln() / (d1 / d0).ln()
    };
    [
        (SMALL_CRATER_RATES[0].0, at(0), b(0)),
        (SMALL_CRATER_RATES[1].0, at(1), b(1)),
        (SMALL_CRATER_RATES[2].0, at(2), b(2)),
        (RATE_DIAMETER_M, EARTH_RATE_20KM_PER_KM2_YR, slope),
    ]
}

/// Craters of at least `d` m (final diameter, as they would form on Earth's land) per km² per
/// year: 5.6 × 10⁻¹⁵ at 20 km and N(≥ D) ∝ D^−`slope` above it (the code's 1.8); below 20 km
/// the observed record's shallower shape (about D^−0.3 at 2 to 5 km), not the power law.
pub fn earth_rate_per_km2_yr(d: f64, slope: f64) -> f64 {
    let segs = production_segments(slope);
    let k = segs.iter().rposition(|&(d0, _, _)| d >= d0).unwrap_or(0);
    let (d0, f0, b) = segs[k];
    f0 * (d / d0).powf(-b) // ledger: IMP-001, IMP-003, IMP-004
}

/// An Earth-equivalent crater diameter drawn from the production function between `lo` and `hi`
/// m with `u` uniform in [0, 1): the inverse of the cumulative function of
/// [`earth_rate_per_km2_yr`], truncated to that range.
pub fn draw_diameter_m(u: f64, lo: f64, hi: f64, slope: f64) -> f64 {
    let (f_lo, f_hi) = (
        earth_rate_per_km2_yr(lo, slope),
        earth_rate_per_km2_yr(hi, slope),
    );
    let target = f_lo - u * (f_lo - f_hi);
    let segs = production_segments(slope);
    // The segment whose range of rates holds the target: the highest one starting at or below the
    // rate (rates fall with diameter), the first for anything above.
    let k = segs
        .iter()
        .rposition(|&(_, f0, _)| target <= f0)
        .unwrap_or(0);
    let (d0, f0, b) = segs[k];
    (d0 * (target / f0).powf(-1.0 / b)).clamp(lo, hi)
}

/// The impactor that makes a crater of final diameter `d` m on Earth's land.
pub fn impactor_of_earth_crater_m(s: &Setting, d: f64) -> f64 {
    let e = s.on_earth();
    e.impactor_m(e.transient_of_final_m(d))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn earth() -> Setting {
        Setting {
            gravity: G_EARTH,
            speed: 20_000.0,
            impactor_density: 3000.0,
            target_density: 2700.0,
            sea_cutoff: SEA_CUTOFF_RATIO,
        }
    }

    /// CMM05's three worked examples (Table 6, p. 834): a 40 m iron body (8,000 kg/m³, into
    /// sedimentary rock of 2,500 kg/m³, slowed from 20 to 10 km/s by the air) makes a 1.2 km simple
    /// crater; a 1.75 km stony body (2,700 kg/m³, into crystalline rock of 2,750) a 23.7 km complex
    /// one; an 18 km stony body a 186 km one. Ledger IMP-030 to IMP-032, IMP-064.
    #[test]
    fn the_papers_worked_examples_come_out() {
        let stony = |l: f64| {
            let s = Setting {
                impactor_density: 2700.0,
                target_density: 2750.0,
                ..earth()
            };
            crater(&s, l, 0.0).unwrap()
        };
        let ries = stony(1750.0);
        assert_eq!(ries.morphology, Morphology::Complex);
        assert!(
            (ries.diameter_m / 23_700.0 - 1.0).abs() < 0.01,
            "{}",
            ries.diameter_m
        );
        // Table 6's impact energy: 1.5 × 10²¹ J.
        assert!(
            (ries.energy_j / 1.5e21 - 1.0).abs() < 0.03,
            "{}",
            ries.energy_j
        );
        let chicxulub = stony(18_000.0);
        assert_eq!(chicxulub.morphology, Morphology::PeakRing);
        assert!(
            (chicxulub.diameter_m / 186_000.0 - 1.0).abs() < 0.01,
            "{}",
            chicxulub.diameter_m
        );
        assert!((chicxulub.energy_j / 1.65e24 - 1.0).abs() < 0.03);
        let iron = Setting {
            speed: 10_000.0,
            impactor_density: 8000.0,
            target_density: 2500.0,
            ..earth()
        };
        let barringer = crater(&iron, 40.0, 0.0).unwrap();
        assert_eq!(barringer.morphology, Morphology::Simple);
        assert!(
            (barringer.diameter_m / 1200.0 - 1.0).abs() < 0.05,
            "{}",
            barringer.diameter_m
        );
        // Ejecta at 200 km (Table 6: 0.09 m and 137 m; the table's own arithmetic differs a
        // little from eq. 47 as printed).
        assert!((ejecta_m(&ries, 200_000.0) / 0.09 - 1.0).abs() < 0.2);
        assert!((ejecta_m(&chicxulub, 200_000.0) / 137.0 - 1.0).abs() < 0.2);
    }

    /// CMM05 eq. 28: the depth of a complex crater is 0.4 D^0.3 km (D in km). The first coded
    /// version, 0.294 D^0.301, made every complex crater 27 % too shallow. Ledger IMP-042.
    #[test]
    fn complex_craters_are_as_deep_as_equation_28() {
        let s = earth();
        for d_km in [4.0, 10.0, 24.0, 60.0, 150.0, 300.0] {
            let l = impactor_of_earth_crater_m(&s, d_km * 1000.0);
            let c = crater(&s, l, 0.0).unwrap();
            assert_ne!(c.morphology, Morphology::Simple, "{d_km}");
            let d = c.diameter_m / 1000.0;
            let eq28 = 0.4 * d.powf(0.3) * 1000.0;
            assert!(
                (c.depth_m / eq28 - 1.0).abs() < 1e-9,
                "{d} km: {} m, eq. 28 {eq28} m",
                c.depth_m
            );
        }
        // The paper's own example: the 23.7 km crater is about a kilometre deep.
        let ries = {
            let s = Setting {
                impactor_density: 2700.0,
                target_density: 2750.0,
                ..s
            };
            crater(&s, 1750.0, 0.0).unwrap()
        };
        assert!((ries.depth_m - 1035.0).abs() < 25.0, "{}", ries.depth_m);
    }

    #[test]
    fn simple_craters_are_a_fifth_as_deep_as_wide_and_complex_ones_shallower() {
        let s = earth();
        let small = crater(&s, 60.0, 0.0).unwrap();
        assert_eq!(small.morphology, Morphology::Simple);
        let ratio = small.depth_m / small.diameter_m;
        assert!((0.17..0.25).contains(&ratio), "{ratio}");
        // Rim about 4 % of the diameter (Pike 1977: 0.036 D on the Moon).
        let rim = small.rim_m / small.diameter_m;
        assert!((0.02..0.05).contains(&rim), "{rim}");
        // The breccia lens is about half the apparent depth.
        assert!(small.breccia_m > 0.3 * small.depth_m && small.breccia_m < small.depth_m);
        let mut last = ratio;
        for l in [500.0, 2000.0, 8000.0] {
            let c = crater(&s, l, 0.0).unwrap();
            assert_ne!(c.morphology, Morphology::Simple, "{l}");
            let r = c.depth_m / c.diameter_m;
            assert!(r < last, "depth/diameter falls with size: {r} after {last}");
            assert!(c.depth_m < 3000.0);
            last = r;
        }
    }

    #[test]
    fn sizes_grow_with_the_impactor_and_invert() {
        let s = earth();
        let mut last = 0.0;
        for k in 0..60 {
            let l = 20.0 * 1.15f64.powi(k);
            let c = crater(&s, l, 0.0).unwrap();
            assert!(c.diameter_m > last, "monotone at {l}");
            assert!(c.depth_m > 0.0 && c.rim_m > 0.0 && c.breccia_m > 0.0);
            last = c.diameter_m;
            let back = impactor_of_earth_crater_m(&s, c.diameter_m);
            assert!((back / l - 1.0).abs() < 1e-9, "{l} -> {back}");
        }
    }

    #[test]
    fn the_transition_scales_inversely_with_gravity() {
        let e = earth();
        let moon = Setting { gravity: 1.62, ..e };
        let mars = Setting { gravity: 3.71, ..e };
        assert!((e.transition_m() - 3200.0).abs() < 1e-9);
        // The Moon's transition is 15–20 km, Mars's 6–9 km (Pike 1980).
        assert!((15_000.0..22_000.0).contains(&moon.transition_m()));
        assert!((6_000.0..10_000.0).contains(&mars.transition_m()));
        assert!((120_000.0..260_000.0).contains(&moon.peak_ring_m()));
        // The same impactor digs a wider crater in weaker gravity, and a 5-km crater is a bowl on
        // the Moon and a complex crater on Earth.
        let l = 300.0;
        let (a, b) = (crater(&e, l, 0.0).unwrap(), crater(&moon, l, 0.0).unwrap());
        assert!(b.transient_m > 1.3 * a.transient_m);
        assert_eq!(a.morphology, Morphology::Complex);
        assert_eq!(b.morphology, Morphology::Simple);
    }

    #[test]
    fn big_craters_have_peak_rings_and_melt_sheets() {
        let s = earth();
        // Chicxulub-sized: ~180 km, a peak ring, a melt sheet hundreds of metres thick.
        let l = impactor_of_earth_crater_m(&s, 180_000.0);
        assert!((8_000.0..20_000.0).contains(&l), "impactor {l}");
        let c = crater(&s, l, 0.0).unwrap();
        assert_eq!(c.morphology, Morphology::PeakRing);
        assert!((300.0..3000.0).contains(&c.melt_m), "melt {}", c.melt_m);
        assert!(c.melt_m3 > 1.0e12);
        // A small bowl has no sheet.
        assert_eq!(crater(&s, 50.0, 0.0).unwrap().melt_m, 0.0);
        // A 20-km crater has a central peak, not a ring.
        let l20 = impactor_of_earth_crater_m(&s, 20_000.0);
        assert_eq!(
            crater(&s, l20, 0.0).unwrap().morphology,
            Morphology::Complex
        );
    }

    #[test]
    fn ejecta_thin_with_the_cube_of_distance() {
        let c = crater(&earth(), 1000.0, 0.0).unwrap();
        let r = 0.5 * c.diameter_m;
        let at_rim = ejecta_m(&c, r);
        assert!(
            (at_rim / c.rim_m - 1.0).abs() < 0.05,
            "{at_rim} vs {}",
            c.rim_m
        );
        assert!((ejecta_m(&c, 2.0 * r) / at_rim - 0.125).abs() < 1e-9);
        assert!(ejecta_m(&c, 5.0 * r) < 0.01 * at_rim);
    }

    /// Davison & Collins 2007, Table 2: the final crater of an impactor 100 m across (simple) in
    /// water R impactor diameters deep, over the dry one, from their hydrocode runs; and eq. 3,
    /// 1 − 0.14 R, the regression through them.
    #[test]
    fn the_sea_follows_davison_and_collins() {
        let s = earth();
        let simple = 100.0;
        let dry = crater(&s, simple, 0.0).unwrap();
        assert_eq!(dry.morphology, Morphology::Simple);
        // (R, wet/dry final diameter) for the 100 m runs, Table 2.
        let table = [
            (1.0, 0.923),
            (1.5, 0.858),
            (2.0, 0.813),
            (2.5, 0.684),
            (3.0, 0.561),
            (3.5, 0.568),
            (4.0, 0.458),
            (5.0, 0.329),
        ];
        for (r, published) in table {
            let wet = crater(&s, simple, r * simple).unwrap();
            let ratio = wet.diameter_m / dry.diameter_m;
            // The regression is eq. 3 exactly …
            assert!((ratio - (1.0 - 0.14 * r)).abs() < 1e-9, "R = {r}: {ratio}");
            // … and is within the scatter of the runs it was fitted to.
            assert!(
                (ratio - published).abs() < 0.12,
                "R = {r}: {ratio} vs {published}"
            );
        }
        // The same law in the complex runs (500 m and 1 km impactors) for R below 3.
        for (l, r, published) in [
            (1000.0, 1.0, 0.948),
            (1000.0, 2.0, 0.694),
            (500.0, 2.5, 0.649),
        ] {
            let dry = crater(&s, l, 0.0).unwrap();
            assert_ne!(dry.morphology, Morphology::Simple);
            let wet = crater(&s, l, r * l).unwrap();
            let ratio = wet.diameter_m / dry.diameter_m;
            assert!(
                (ratio - (1.0 - 0.14 * r)).abs() < 1e-9,
                "{l} m, R = {r}: {ratio}"
            );
            assert!(
                (ratio - published).abs() < 0.12,
                "{l} m, R = {r}: {ratio} vs {published}"
            );
            // A smaller crater has less melt.
            assert!(wet.melt_m3 < dry.melt_m3);
        }
        // On land, and in no water at all, nothing changes.
        assert_eq!(sea_floor_factor(0.0, 10.0, 8.0, true), 1.0);
        assert_eq!(sea_floor_factor(-50.0, 10.0, 8.0, false), 1.0);
    }

    /// Davison & Collins 2007: no crater beyond R = 8; a crater that would be complex on land is a
    /// resurge feature, not a crater, beyond R = 3.5; so a body leaves a crater in the deep sea
    /// only when it is large. The first coded rule (a straight line to zero at R = 6.3) made a
    /// crater of 0.37 of the transient size from a 1 km body in 4 km of water.
    #[test]
    fn deep_water_stops_small_impactors_and_turns_mid_sized_ones_into_resurge_features() {
        let s = earth();
        // A 100 m body in 4 km of water (R = 40): nothing reaches the floor.
        assert!(crater(&s, 100.0, 4000.0).is_none());
        // A simple-crater body is stopped beyond R = 8 whatever the line says.
        assert_eq!(sea_floor_factor(8.5 * 50.0, 50.0, 8.0, false), 0.0);
        assert!(sea_floor_factor(6.0 * 50.0, 50.0, 8.0, false) > 0.1);
        // On a 100-m shelf it digs nearly the crater it would on land (R = 1: 0.86 of it).
        let shelf = crater(&s, 100.0, 100.0).unwrap();
        let land = crater(&s, 100.0, 0.0).unwrap();
        assert!((shelf.diameter_m / land.diameter_m - 0.86).abs() < 1e-9);
        // A 1 km body (a 15 km complex crater on land): a crater in 3 km of water (R = 3), a
        // resurge feature in 4 km (R = 4).
        assert!(crater(&s, 1000.0, 3000.0).is_some());
        assert!(crater(&s, 1000.0, 4000.0).is_none());
        // A 4.5 km body (a 100 km crater) hardly notices 4 km of water.
        assert!(sea_floor_factor(4000.0, 4500.0, 8.0, true) > 0.85);
        // The cutoff is the setting.
        assert!(
            crater(
                &Setting {
                    sea_cutoff: 2.0,
                    ..s
                },
                1000.0,
                2500.0
            )
            .is_none()
        );
    }

    /// The production function: Grieve & Shoemaker 1994's rate at 20 km and the 1.8 slope above;
    /// below 20 km Hughes 2000's shape (his abstract: 12.0, 9.5, 6.5 and 3.46 × 10⁻¹⁵ km⁻² yr⁻¹
    /// for D > 2.4, 5, 12 and 20 km). The first coded version carried the 1.8 slope down to 2 km,
    /// 63 times the rate at 20 km. Ledger IMP-001, IMP-003, IMP-004.
    #[test]
    fn the_production_function_follows_the_record() {
        let f = |d: f64| earth_rate_per_km2_yr(d, 1.8);
        assert!((f(20_000.0) - 5.6e-15).abs() < 1e-24);
        // Above 20 km: the power law.
        assert!((f(40_000.0) / f(20_000.0) - 2f64.powf(-1.8)).abs() < 1e-9);
        assert!((f(200_000.0) / f(20_000.0) - 10f64.powf(-1.8)).abs() < 1e-9);
        // Below: Hughes's ratios to his rate at 20 km.
        for (d, hughes) in [(2_400.0, 12.0), (5_000.0, 9.5), (12_000.0, 6.5)] {
            let want = hughes / 3.46;
            assert!(
                (f(d) / f(20_000.0) / want - 1.0).abs() < 1e-6,
                "{d}: {} against {want}",
                f(d) / f(20_000.0)
            );
        }
        // So 2-km craters are a few times the 20-km rate, not 63 times.
        let r2 = f(2_000.0) / f(20_000.0);
        assert!((3.0..5.0).contains(&r2), "{r2}");
        // Continuous and falling everywhere.
        let mut last = f64::INFINITY;
        for k in 0..400 {
            let d = 500.0 * 1.02f64.powi(k);
            let now = f(d);
            assert!(now < last, "falls at {d}");
            if k > 0 {
                assert!(now > 0.95 * last, "no jump at {d}: {now} after {last}");
            }
            last = now;
        }
        // The slope above 20 km is the setting's.
        assert!(
            (earth_rate_per_km2_yr(40_000.0, 2.5) / f(20_000.0) - 2f64.powf(-2.5)).abs() < 1e-9
        );
    }

    #[test]
    fn the_draws_follow_the_production_function() {
        let (lo, hi) = (2_000.0, 300_000.0);
        assert!((draw_diameter_m(0.0, lo, hi, 1.8) - lo).abs() < 1e-6);
        assert!(draw_diameter_m(1.0 - 1e-12, lo, hi, 1.8) <= hi);
        let mut last = 0.0;
        for k in 0..1000 {
            let d = draw_diameter_m(f64::from(k) / 1000.0, lo, hi, 1.8);
            assert!(d >= last && d >= lo && d <= hi);
            last = d;
        }
        // The share of draws below a diameter is the share of the cumulative rate: with u on a
        // fine grid, the fraction below 5, 12 and 20 km matches (F(lo) − F(d)) / (F(lo) − F(hi)).
        let f = |d: f64| earth_rate_per_km2_yr(d, 1.8);
        let n = 20_000;
        let draws: Vec<f64> = (0..n)
            .map(|k| draw_diameter_m((f64::from(k) + 0.5) / f64::from(n), lo, hi, 1.8))
            .collect();
        for d in [3_000.0, 5_000.0, 12_000.0, 20_000.0, 60_000.0] {
            let got = draws.iter().filter(|&&x| x < d).count() as f64 / f64::from(n);
            let want = (f(lo) - f(d)) / (f(lo) - f(hi));
            assert!((got - want).abs() < 0.002, "below {d}: {got} vs {want}");
        }
        // Craters of 20 km and more are about a quarter of those of 2 km and more.
        let big = draws.iter().filter(|&&x| x >= 20_000.0).count() as f64 / f64::from(n);
        assert!((0.2..0.35).contains(&big), "{big}");
    }
}

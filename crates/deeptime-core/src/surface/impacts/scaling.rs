// SPDX-License-Identifier: LGPL-3.0-or-later
//! Impact scaling laws (P2-08): how often impactors arrive, how big they are, and the crater each
//! makes. Everything is SI (metres, seconds, kilograms, joules) unless a name says otherwise.
//!
//! # Sources
//! - **Rate.** Grieve & Shoemaker 1994 ("The record of past impacts on Earth", in Gehrels (ed.),
//!   *Hazards due to Comets and Asteroids*, Univ. Arizona Press, 417–462): (5.6 ± 2.8) × 10⁻¹⁵
//!   craters of D ≥ 20 km per km² per year on Earth, from the well-dated craters of the North
//!   American and European cratons, and N(≥ D) ∝ D⁻¹·⁸ above ~20 km. The same slope is used below
//!   20 km, where erosion has removed the record; the rate has been constant within a factor of
//!   two for the last ~3 Gyr (Neukum, Ivanov & Hartmann 2001, Space Sci. Rev. 96:55–86), so it is
//!   constant over a run.
//! - **Crater from impactor.** Collins, Melosh & Marcus 2005 (Meteoritics & Planet. Sci.
//!   40(6):817–840, the Earth Impact Effects Program), which gathers the π-group transient-crater
//!   scaling of Schmidt & Housen 1987 (Int. J. Impact Eng. 5:543–560) with the oblique-impact
//!   factor of Gault & Wedekind 1978 and Chapman & McKinnon 1986; the collapse to a simple crater
//!   and its breccia lens (Grieve, Dence & Robertson 1977; Melosh 1989, *Impact Cratering*); the
//!   complex-crater enlargement of McKinnon & Schenk 1985 (as in Holsapple 1993); the complex
//!   depth law; the melt volume of Grieve & Cintala 1992 / Pierazzo, Vickery & Melosh 1997; and
//!   the ejecta thickness of McGetchin, Settle & Head 1973 (EPSL 20:226–236), ∝ r⁻³.
//! - **Simple-to-complex transition** 3.2 km on Earth (2 km in sedimentary, 4 km in crystalline
//!   targets: Grieve 1987, Ann. Rev. Earth Planet. Sci. 15:245–270), ∝ 1/g (Pike 1980, Proc. Lunar
//!   Planet. Sci. Conf. 11:2159–2189; Melosh 1989).
//! - **Peak rings** from ~40 km on Venus (Alexopoulos & McKinnon 1994, GSA Spec. Pap. 293:29–50),
//!   also ∝ 1/g; the ring's diameter is about half the rim's (Wood & Head 1976; Pike & Spudis
//!   1987).
//! - **Central peaks.** Peak diameter ≈ 0.22 D (Pike 1985, Meteoritics 20:49–68).
//! - **Oceans.** Davison & Collins 2007 (Meteoritics & Planet. Sci. 42(11):1915–1927): the sea
//!   removes about two thirds of the craters under 1 km, a third of those about 30 km, and hardly
//!   any over 100 km. [`sea_floor_factor`] is a linear fit to those three numbers in the ratio of
//!   water depth to impactor diameter (their own curves are not reproduced).
//!
//! The constants were written from the papers as remembered and checked for self-consistency by
//! the tests below (a 1-km stony asteroid at 20 km/s makes a ~16-km crater on Earth; a simple
//! crater is a fifth as deep as wide); STATUS.md lists what was not checked against the papers.

/// Earth's surface gravity, m/s².
pub const G_EARTH: f64 = 9.81;
/// Venus's surface gravity, m/s².
pub const G_VENUS: f64 = 8.87;
/// Craters of at least 20 km formed per km² per year on Earth (Grieve & Shoemaker 1994).
pub const EARTH_RATE_20KM_PER_KM2_YR: f64 = 5.6e-15;
/// The diameter that rate is for, m.
pub const RATE_DIAMETER_M: f64 = 20_000.0;
/// Simple-to-complex transition on Earth, m.
pub const TRANSITION_EARTH_M: f64 = 3200.0;
/// Peak-ring onset on Venus, m.
pub const PEAK_RING_VENUS_M: f64 = 40_000.0;
/// Impact angle from the horizontal the rate is turned into impactors with: the most probable
/// one (Gilbert 1893; Shoemaker 1962: dP = sin 2θ dθ).
pub const ANGLE_RAD: f64 = std::f64::consts::FRAC_PI_4;
/// The water depth, in impactor diameters, at which no crater forms in the sea floor
/// ([`sea_floor_factor`]).
pub const SEA_CUTOFF_RATIO: f64 = 6.3;

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
    /// Water deeper than this many impactor diameters stops the impactor ([`sea_floor_factor`]).
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

    /// Transient crater diameter of an impactor `l` m across (Collins et al. 2005, eq. 21):
    /// D_tc = 1.161 (ρ_i/ρ_t)^⅓ L^0.78 v^0.44 g^−0.22 sin^⅓θ.
    pub fn transient_m(&self, l: f64) -> f64 {
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

    /// Final rim-to-rim diameter from the transient one (Collins et al. 2005, eqs. 22 and 27):
    /// simple 1.25 D_tc; complex 1.17 D_tc^1.13 / D_c^0.13. The two meet at D_tc ≈ 1.66 D_c, so
    /// the larger is taken (a crater just past the transition is still bowl-sized).
    pub fn final_m(&self, d_tc: f64) -> f64 {
        let dc = self.transition_m();
        (1.25 * d_tc).max(1.17 * d_tc.powf(1.13) / dc.powf(0.13))
    }

    /// The transient diameter of a final one (the inverse of [`Setting::final_m`]).
    pub fn transient_of_final_m(&self, d: f64) -> f64 {
        let dc = self.transition_m();
        (d / 1.25).min((d * dc.powf(0.13) / 1.17).powf(1.0 / 1.13))
    }

    /// Kinetic energy of an impactor `l` m across, J.
    pub fn energy_j(&self, l: f64) -> f64 {
        std::f64::consts::PI / 12.0 * self.impactor_density * l.powi(3) * self.speed * self.speed
    }

    /// Impact melt volume, m³ (Collins et al. 2005, eq. 12: 8.9 × 10⁻¹² E sin θ).
    pub fn melt_m3(&self, l: f64) -> f64 {
        8.9e-12 * self.energy_j(l) * ANGLE_RAD.sin()
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
    /// Depth of excavation at the centre (a third of the transient depth: Melosh 1989).
    pub excavation_m: f64,
    /// Central peak or peak ring height above the floor.
    pub peak_m: f64,
    pub energy_j: f64,
}

/// The crater an impactor `l` m across makes where the sea is `water_m` deep (0 on land), or
/// `None` when the sea stops it.
pub fn crater(s: &Setting, l: f64, water_m: f64) -> Option<Crater> {
    let f = sea_floor_factor(water_m, l, s.sea_cutoff);
    if f <= 0.0 {
        return None;
    }
    let d_tc = s.transient_m(l) * f;
    let d = s.final_m(d_tc);
    let dc = s.transition_m();
    // Rim height (Collins et al. 2005, eq. 25): 0.07 D_tc⁴ / D_fr³.
    let rim = 0.07 * d_tc.powi(4) / d.powi(3);
    let transient_depth = d_tc / (2.0 * std::f64::consts::SQRT_2);
    let melt_m3 = s.melt_m3(l) * f.powi(3);
    // Melt sheet (eq. 31): the melt spread over the transient crater's floor. Under ~10 m it is
    // pods and glass in the breccia, not a sheet.
    let sheet = 4.0 * melt_m3 / (std::f64::consts::PI * d_tc * d_tc);
    let melt = if sheet >= 10.0 { sheet } else { 0.0 };
    let (morphology, depth, breccia, peak) = if d < dc {
        // Breccia lens (eqs. 24 and 26): V_br = 0.032 D_fr³ as a lens,
        // t_br = 2.8 V_br (d_tc + h_fr) / (d_tc D_fr²); the final depth is the transient depth
        // plus the rim less the lens: about D / 5.
        let v_br = 0.032 * d.powi(3);
        let t_br = 2.8 * v_br * (transient_depth + rim) / (transient_depth * d * d);
        (
            Morphology::Simple,
            transient_depth + rim - t_br,
            t_br,
            0.0,
        )
    } else {
        // Complex depth (eq. 28): 0.294 D^0.301 in km at Earth's gravity; at the transition it
        // is continued from the simple crater's D / 5, and it scales with the transition (∝ 1/g).
        let km = d / 1000.0 * s.gravity / G_EARTH;
        let depth = (294.0 * km.powf(0.301) * G_EARTH / s.gravity).min(0.2 * d);
        let ring = d >= s.peak_ring_m();
        // Peaks stand about half the floor's depth (design value within the observed range;
        // Pike 1977's lunar peaks, 0.0006 D^1.97 km, pass the floor depth near 50 km), a ring
        // lower.
        let peak = if ring { 0.35 * depth } else { 0.5 * depth };
        // The allochthonous breccia and melt-bearing fill of complex craters is about as thick
        // as the crater is deep (Grieve, Dence & Robertson 1977).
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

/// How much of its transient diameter a crater keeps in the sea floor under `water_m` of water,
/// for an impactor `l` m across: 1 − (H/L) / `cutoff` (6.3), never below 0 (no crater: the water
/// column takes the impact). Fitted to Davison & Collins 2007's summary: with N(≥ D) ∝ D⁻¹·⁸ and a
/// 3.8-km ocean over 71 % of the planet, it removes 2/3 of the craters under 1 km (every one in
/// the deep sea), a third at 30 km (a 2-km impactor: factor 0.70) and a tenth at 100 km.
pub fn sea_floor_factor(water_m: f64, l: f64, cutoff: f64) -> f64 {
    if water_m <= 0.0 {
        return 1.0;
    }
    (1.0 - water_m / l / cutoff).max(0.0)
}

/// Ejecta thickness at `r` m from the centre, m (McGetchin et al. 1973, as Collins et al. 2005,
/// eq. 32: D_tc⁴ / (112 r³)); it equals the rim's uplifted height at the rim.
pub fn ejecta_m(c: &Crater, r: f64) -> f64 {
    let rim = 0.5 * c.diameter_m;
    c.transient_m.powi(4) / (112.0 * r.max(rim).powi(3))
}

/// Craters of at least `d` m (final diameter, as they would form on Earth's land) per km² per
/// year, for a cumulative slope `slope` (1.8).
pub fn earth_rate_per_km2_yr(d: f64, slope: f64) -> f64 {
    EARTH_RATE_20KM_PER_KM2_YR * (d / RATE_DIAMETER_M).powf(-slope)
}

/// An Earth-equivalent crater diameter drawn from the production function between `lo` and `hi`
/// m with `u` uniform in [0, 1): the inverse of the truncated power law N(≥ D) ∝ D^−slope.
pub fn draw_diameter_m(u: f64, lo: f64, hi: f64, slope: f64) -> f64 {
    let a = lo.powf(-slope);
    let b = hi.powf(-slope);
    (a - u * (a - b)).powf(-1.0 / slope)
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

    #[test]
    fn a_kilometre_asteroid_makes_a_crater_about_fifteen_kilometres_across() {
        let c = crater(&earth(), 1000.0, 0.0).unwrap();
        assert!(
            (12_000.0..20_000.0).contains(&c.diameter_m),
            "{}",
            c.diameter_m
        );
        assert_eq!(c.morphology, Morphology::Complex);
        // Barringer: a ~40-m iron (7,800 kg/m³) at ~13 km/s made a 1.2-km bowl.
        let iron = Setting {
            speed: 13_000.0,
            impactor_density: 7800.0,
            ..earth()
        };
        let b = crater(&iron, 40.0, 0.0).unwrap();
        assert_eq!(b.morphology, Morphology::Simple);
        assert!(
            (800.0..1800.0).contains(&b.diameter_m),
            "{}",
            b.diameter_m
        );
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
        let moon = Setting {
            gravity: 1.62,
            ..e
        };
        let mars = Setting {
            gravity: 3.71,
            ..e
        };
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
        assert!((at_rim / c.rim_m - 1.0).abs() < 0.05, "{at_rim} vs {}", c.rim_m);
        assert!((ejecta_m(&c, 2.0 * r) / at_rim - 0.125).abs() < 1e-9);
        assert!(ejecta_m(&c, 5.0 * r) < 0.01 * at_rim);
    }

    #[test]
    fn deep_water_stops_small_impactors_and_not_large_ones() {
        let s = earth();
        // A 100-m body in 4 km of water: nothing reaches the floor.
        assert!(crater(&s, 100.0, 4000.0).is_none());
        // On a 100-m shelf it digs nearly the crater it would on land.
        let shelf = crater(&s, 100.0, 100.0).unwrap();
        let land = crater(&s, 100.0, 0.0).unwrap();
        assert!(shelf.diameter_m > 0.7 * land.diameter_m && shelf.diameter_m < land.diameter_m);
        // A 2-km body in the deep sea: a crater about 0.7 of the land one's transient size.
        assert!((sea_floor_factor(3800.0, 2000.0, 6.3) - 0.70).abs() < 0.02);
        // A 10-km body hardly notices 4 km of water.
        assert!(sea_floor_factor(4000.0, 10_000.0, 6.3) > 0.9);
        assert_eq!(sea_floor_factor(0.0, 10.0, 6.3), 1.0);
        assert_eq!(sea_floor_factor(-50.0, 10.0, 6.3), 1.0);
    }

    #[test]
    fn the_production_function_draws_many_small_and_few_large() {
        assert!((earth_rate_per_km2_yr(20_000.0, 1.8) - 5.6e-15).abs() < 1e-24);
        let ratio = earth_rate_per_km2_yr(2_000.0, 1.8) / earth_rate_per_km2_yr(20_000.0, 1.8);
        assert!((ratio - 10f64.powf(1.8)).abs() < 1e-6);
        let (lo, hi) = (2_000.0, 300_000.0);
        assert!((draw_diameter_m(0.0, lo, hi, 1.8) - lo).abs() < 1e-6);
        assert!(draw_diameter_m(1.0 - 1e-12, lo, hi, 1.8) <= hi);
        // Half of the draws are below lo · 2^(1/1.8) ≈ 1.47 lo.
        let median = draw_diameter_m(0.5, lo, hi, 1.8);
        assert!((median / lo - 2f64.powf(1.0 / 1.8)).abs() < 0.01, "{median}");
        let mut last = 0.0;
        for k in 0..100 {
            let d = draw_diameter_m(f64::from(k) / 100.0, lo, hi, 1.8);
            assert!(d >= last);
            last = d;
        }
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The preset's `structure.deformation` section (P2-01): how many folds and faults the tectonic
//! history leaves in the rock, how big they are and how strongly they deform it. Every key is
//! optional; the defaults are below with their sources.
//!
//! ```json
//! "structure": {"deformation": {"strength": 1.0, "density": 1.0, "fold_wavelength_km": [16, 8, 32]}}
//! ```
//!
//! **Sizes are true in block space** (ledger: DEF-011, a game choice open to Ben: STATUS.md
//! "Pending questions", docs/science/folds-faults-intrusions.md). A fold's wavelength and a
//! fault's length are given in km on the planet's horizontal block scale (300 m a block in the
//! built-in presets), and amplitudes and slips are that many *blocks* times the shape's own ratio
//! (amplitude over wavelength, slip over length), not metres at the vertical scale: heights are
//! stretched 15 times against widths (20 m against 300 m per block), and a fold drawn with its
//! true amplitude in metres would have near-vertical limbs. What is kept is the shape a geologist
//! would draw in a cross-section at 1:1. In true metres (the scale the strata, the plutons and
//! the terrain are drawn at) that makes every structure 15 times lower than it is wide: a 60°
//! fault in blocks is a 6.6° fault in metres, a 30° thrust a 2.2° one, and a fold's amplitude and
//! a fault's throw are a fifteenth of what the same shape has in metres. The horizontal part of
//! a slip (the heave) is true in metres.

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// The deformation section.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct DeformationParams {
    /// How strongly the rock is deformed: multiplies every fold amplitude and fault slip. 0 turns
    /// folds and faults off (the layers of P1-07, flat but for their gentle dips).
    pub strength: f64,
    /// How many structures form: multiplies every rate below.
    pub density: f64,
    /// Multiplies every length (wavelengths, fault lengths, and with them amplitudes and slips):
    /// the size structures are drawn at, like `scale.feature_scale`.
    pub size: f64,

    /// Fold wavelength, km: [median, min, max] of a lognormal with `fold_wavelength_sigma`.
    /// Median 16 km: the folds of the Zagros Folded Belt have "homogeneous fold wavelengths
    /// (15.8 ± 5.3 km)" (Mouthereau, Tensi, Bellahsen, Lacombe, De Boisgrollier & Kargar 2007,
    /// Tectonics 26:TC5006, abstract and fig. 5: 149 anticlines). The range is a design value
    /// about it; the same paper gives 11 km near the Surmeh fault to 30 km in the High Zagros. In
    /// buckling theory the dominant wavelength is proportional to the stiff layer's thickness,
    /// L = 2πH(R/6)^(1/3) for a viscosity ratio R (Biot 1957 and 1961, Ramberg 1962; Schmalholz
    /// & Mancktelow 2016, Solid Earth 7:1417–1465, eq. 13), and natural folds have wavelengths of
    /// 2 to 16 times their layer's thickness (same review, p. 1442); here it is drawn, not
    /// derived from the strata. ledger: DEF-001, DEF-002
    pub fold_wavelength_km: [f64; 3],
    pub fold_wavelength_sigma: f64,
    /// Wavelengths across one fold train, [min, max]: a train is a bundle of a few parallel
    /// anticlines and synclines that dies out sideways.
    pub fold_train_waves: [f64; 2],
    /// A fold train's length along its axis over its wavelength, [min, max]: the Zagros anticlines
    /// are "long (up to ~150–200 km)" and their axial length is typically over 10 half
    /// wavelengths, so over 5 wavelengths (Alavi 2007, Am. J. Sci. 307:1064–1095, pp. 1068 and
    /// 1077). ledger: DEF-004
    pub fold_length_ratio: [f64; 2],
    /// Shortening taken up by folding, [least, most], as a fraction: to first order a sine fold of
    /// amplitude A and wavelength λ has shortened by (πA/λ)², so A/λ = √e/π. The first-order
    /// form overstates it at steep limbs: by the sine's exact arc length the amplitudes drawn
    /// shorten the beds by 7 % (limbs 30°) to 20 % (limbs 48°), in block space. The Zagros has
    /// shortened by 16–19 % across the whole orogen (Mouthereau et al. 2007, abstract) and by 16 to
    /// 30 % across the fold-thrust belt's sections (Alavi 2007, table 1); McQuarrie 2004 (J.
    /// Struct. Geol. 26:519–535, abstract) gives about 20 %. The least (a design value) is used in
    /// forelands, the most (a design value: limbs at about 45°) where a collision has thickened
    /// the crust the most; 1 − `structure.reference_crust_m` / the plate model's thickness gives
    /// the local value in between. ledger: DEF-005, DEF-007
    pub fold_shortening: [f64; 2],
    /// Fold trains per km² of full-strength shortening per Myr. Design value: a train covers
    /// about 28 λ² (7,000 km² at 16 km), so 6e-6 covers a collision belt about 1.3 times over in
    /// 30 Myr.
    pub fold_rate_km2_myr: f64,

    /// Fault trace length, km: [median, min, max] of a lognormal with `fault_length_sigma`.
    /// Design values (segments of some tens of km); faults grow by the linkage of such segments
    /// (Gawthorpe & Leeder 2000, Basin Res. 12:195–218, not read: no open copy). ledger: DEF-013
    pub fault_length_km: [f64; 3],
    pub fault_length_sigma: f64,
    /// How far into the hanging wall a fault's displacement reaches, as a share of its length:
    /// the width of the tilted block (a half-graben). A design value.
    pub fault_reach_ratio: f64,
    /// Greatest slip over trace length. A fault's greatest displacement grows in proportion to
    /// its length, D = γ·L: for normal faults D = 0.03 L^1.06 (Schlische, Young, Ackermann &
    /// Gupta 1996, Geology 24:683–686), with γ "typically between 0.001 and 0.1" and strike-slip
    /// faults displaced more for their length than normal ones (both as reviewed by Lathrop,
    /// Jackson, Bell & Rotevatn 2022, Front. Earth Sci. 10:907543, pp. 2, 4 and 7); for thrusts
    /// about 7 % (Elliott 1976, Phil. Trans. R. Soc. A 283:289–312, p. 297, 20 thrusts of the
    /// Canadian Rockies). The values here are design values, slips along the dip in block space.
    /// Their heave (the horizontal part) is true in metres, so in metres a normal fault's
    /// displacement is 0.017 to 0.029 of its length and a thrust's 0.031 to 0.075, inside the
    /// ranges read; their throw (the vertical part) is a fifteenth of the same slip's in metres
    /// (see the module doc). ledger: DEF-015 to DEF-018
    pub normal_slip_ratio: f64,
    pub thrust_slip_ratio: f64,
    pub strike_slip_ratio: f64,
    /// Dip of normal faults, degrees [min, max] (about 60°), and of thrusts (about 30°): normal
    /// faults dip at more than 45° and thrusts at less, at 45° ∓ 15° for a friction coefficient of
    /// 0.577 (Anderson 1905, Trans. Edinb. Geol. Soc. 8:387–402, pp. 389 and 402; the theory of
    /// Anderson 1951, The Dynamics of Faulting, 2nd ed.); active normal faults rupture at 30° to
    /// 65° and reverse faults peak at 30° ± 5° (Collettini & Sibson 2001, Geology 29:927–930,
    /// abstract). Applied in block space (module doc): in metres these are 5.4° to 10.4° and 1.4°
    /// to 3.2°. ledger: DEF-019, DEF-020
    pub normal_dip_deg: [f64; 2],
    pub thrust_dip_deg: [f64; 2],
    /// Faults per km² of full-strength regime per Myr: thrusts where the crust shortens, normal
    /// faults where a continent rifts, strike-slip faults where convergence is oblique.
    pub thrust_rate_km2_myr: f64,
    pub normal_rate_km2_myr: f64,
    pub strike_slip_rate_km2_myr: f64,

    /// How far into the foreland of an active orogen the crust still shortens, km on the plate
    /// model's (Earth-normalized) sphere: the Zagros fold-thrust belt is "a ~200 to 300 km-wide
    /// zone" (Alavi 2007, abstract), its Fars sector 200 km (Mouthereau et al. 2007, §2); the
    /// Subandean belt of Bolivia is 60 to 140 km wide (Hérail et al., ORSTOM symposium paper,
    /// pp. 179–180, a poor scan). On a 16,384-block planet 250 km is about 100 blocks (30 km on
    /// the map, where the folds keep their Earth size). ledger: DEF-027
    pub foreland_km: f64,
    /// The plate model's uplift rate at which an Andean margin shortens at full strength, mm/yr
    /// (the amplifier's `active_uplift_mm_yr` is the same 0.1).
    pub arc_uplift_mm_yr: f64,
    /// Structures of one kind that may overlap at a place; a new one is not started where this
    /// many already are (strain goes on being taken up by the structures that exist).
    pub max_overlap: u32,
    /// At a place that is full, a new structure replaces the oldest one there when that one is at
    /// least this much older, Myr (a later orogenic pulse overprints an earlier one). Design
    /// value: the plate model's collision phase lasts 30 Myr.
    pub overprint_myr: f64,
    /// Scatter of a structure's trend about the regional trend, degrees (one standard deviation).
    pub trend_scatter_deg: f64,
    /// The most structures a world keeps (a guard, far above what the rates give).
    pub max_structures: u32,

    /// How much of their relief young structures put on the ground (fault scarps, rift valleys,
    /// fold ridges; `surface::amplify::structure`): 0 leaves the terrain as it was, 1 moves the
    /// ground of a structure made in the last plate step as far as its rock moved.
    pub relief: f64,
    /// A structure's relief fades to nothing over this long, Myr. Design value: metre-scale
    /// scarps soften within about 100 kyr and range fronts stand for millions of years; a
    /// structure here stands for a fault's whole active life.
    pub relief_age_myr: f64,
    /// … and as the deep-time surface erodes or buries the ground there by this many times the
    /// structure's own height (a fault that is still slipping renews its scarp, so more than
    /// once).
    pub relief_wear: f64,
    /// The share of a fold train's shift that shows as ridges and valleys.
    pub fold_relief: f64,
    /// The foot wall's movement the other way, as a share of the hanging wall's (the uplifted
    /// shoulders of a rift, the foredeep in front of a thrust). Rift flanks rise by flexure when
    /// the hanging wall unloads them (Weissel & Karner 1989, J. Geophys. Res. 94:13919-13950);
    /// the share is a design value.
    pub footwall_share: f64,

    /// How many small intrusions form (stocks, dykes, plugs: `intrude.rs`): multiplies their
    /// rates; 0 turns them off. They cut the layering and the folds older than themselves.
    pub intrusions: f64,
    /// Stocks per km² of collision or arc per Myr. A design value.
    pub stock_rate_km2_myr: f64,
    /// A stock's radius, km: [median, min, max] of a lognormal (sigma 0.4): its half-width at
    /// the world's floor, where the rooted ellipsoid is widest. A stock is a pluton exposed over
    /// less than 100 km² (Earle 2019, Physical Geology 2nd ed., §3.5); here a stock's top lies
    /// from 4 blocks above to 60 under the ground, so where one crops out it shows only its
    /// cupola, a few per cent of its section at depth, and the 100 km² holds at the surface while
    /// the body below widens to the size of a small batholith. ledger: DEF-042
    pub stock_radius_km: [f64; 3],
    /// How far a stock's top lies under the ground, blocks [min, max] (negative: it crops out).
    pub stock_top_depth_blocks: [f64; 2],
    /// Dykes per km² of continental rift per Myr. A design value.
    pub dyke_rate_km2_myr: f64,
    /// A dyke's length, km: [median, min, max] of a lognormal (sigma 0.5). Design values;
    /// single dykes run for kilometres (the Ship Rock dikes 3 to 9 km: Delaney & Pollard 1981,
    /// USGS PP 1202) to over 2,000 km in giant swarms (the Mackenzie swarm: Bunger, Menand,
    /// Cruden, Zhang & Halls 2013, Earth Planet. Sci. Lett. 375:270–279). ledger: DEF-047
    pub dyke_length_km: [f64; 3],
    /// A dyke's half-width at its middle, blocks [min, max]: 2 to 5 blocks (600 to 1,500 m)
    /// across. Real dykes are metres to tens of metres wide (Ship Rock's northeastern dike 2.3 m
    /// on average, Delaney & Pollard 1981; the Mackenzie dykes 1 to 150 m, mean 30 m, Bunger et
    /// al. 2013); a block (300 m) is the least that can be drawn. A game choice. ledger: DEF-048
    pub dyke_half_width_blocks: [f64; 2],
    /// A plug's radius as a share of its volcano's basal radius, and its range in blocks
    /// [min, max]: 1.5 to 4.8 km across. Design values: real necks are tens to hundreds of
    /// metres across (Ship Rock 500 m, New Mexico Bureau of Geology; the plugs beside it under
    /// 30 m, Delaney & Pollard 1981), so plugs are drawn 3 to 10 times wider than Ship Rock. A
    /// game choice. ledger: DEF-049
    pub plug_radius_share: f64,
    pub plug_radius_blocks: [f64; 2],
}

impl Default for DeformationParams {
    fn default() -> Self {
        DeformationParams {
            strength: 1.0,                        // ledger: DEF-012
            density: 1.0,                         // ledger: DEF-012
            size: 1.0,                            // ledger: DEF-012
            fold_wavelength_km: [16.0, 8.0, 32.0], // ledger: DEF-001
            fold_wavelength_sigma: 0.35,          // ledger: DEF-001
            fold_train_waves: [2.0, 5.0],         // ledger: DEF-003
            fold_length_ratio: [5.0, 12.0],       // ledger: DEF-004
            fold_shortening: [0.08, 0.30],        // ledger: DEF-005
            fold_rate_km2_myr: 6.0e-6,            // ledger: DEF-008
            fault_length_km: [40.0, 15.0, 150.0], // ledger: DEF-013
            fault_length_sigma: 0.5,              // ledger: DEF-013
            fault_reach_ratio: 0.4,               // ledger: DEF-014
            normal_slip_ratio: 0.05,              // ledger: DEF-016
            thrust_slip_ratio: 0.08,              // ledger: DEF-017
            strike_slip_ratio: 0.05,              // ledger: DEF-018
            normal_dip_deg: [55.0, 70.0],         // ledger: DEF-019
            thrust_dip_deg: [20.0, 40.0],         // ledger: DEF-020
            thrust_rate_km2_myr: 2.0e-5,          // ledger: DEF-022
            normal_rate_km2_myr: 4.0e-5,          // ledger: DEF-022
            strike_slip_rate_km2_myr: 4.0e-6,     // ledger: DEF-022
            foreland_km: 250.0,                   // ledger: DEF-027
            arc_uplift_mm_yr: 0.1,                // ledger: DEF-026
            max_overlap: 3,                       // ledger: DEF-032
            overprint_myr: 30.0,                  // ledger: DEF-033
            trend_scatter_deg: 12.0,              // ledger: DEF-009
            max_structures: 60_000,               // ledger: DEF-034
            relief: 1.0,                          // ledger: DEF-054
            relief_age_myr: 10.0,                 // ledger: DEF-055
            relief_wear: 5.0,                     // ledger: DEF-056
            fold_relief: 0.5,                     // ledger: DEF-058
            footwall_share: 0.2,                  // ledger: DEF-059
            intrusions: 1.0,                      // ledger: DEF-040
            stock_rate_km2_myr: 2.0e-5,           // ledger: DEF-041
            stock_radius_km: [4.0, 2.0, 10.0],    // ledger: DEF-042
            stock_top_depth_blocks: [-4.0, 60.0], // ledger: DEF-044
            dyke_rate_km2_myr: 8.0e-5,            // ledger: DEF-046
            dyke_length_km: [15.0, 5.0, 50.0],    // ledger: DEF-047
            dyke_half_width_blocks: [1.0, 2.5],   // ledger: DEF-048
            plug_radius_share: 0.08,              // ledger: DEF-049
            plug_radius_blocks: [2.5, 8.0],       // ledger: DEF-049
        }
    }
}

fn check(ok: bool, key: &str, reason: &str) -> Result<(), CoreError> {
    if ok {
        Ok(())
    } else {
        Err(CoreError::preset(
            format!("$.structure.deformation.{key}"),
            reason,
        ))
    }
}

impl DeformationParams {
    /// Whether any structure can form.
    pub fn enabled(&self) -> bool {
        self.strength > 0.0 && self.density > 0.0
    }

    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        check(
            (0.0..=4.0).contains(&self.strength),
            "strength",
            "must be in [0, 4]",
        )?;
        check(
            (0.0..=10.0).contains(&self.density),
            "density",
            "must be in [0, 10]",
        )?;
        check(
            (0.1..=10.0).contains(&self.size),
            "size",
            "must be in [0.1, 10]",
        )?;
        for (v, key) in [
            (self.fold_wavelength_km, "fold_wavelength_km"),
            (self.fault_length_km, "fault_length_km"),
            (self.stock_radius_km, "stock_radius_km"),
            (self.dyke_length_km, "dyke_length_km"),
        ] {
            let [m, lo, hi] = v;
            check(
                lo > 0.0 && lo <= m && m <= hi && hi.is_finite(),
                key,
                "[median, min, max] with 0 < min ≤ median ≤ max",
            )?;
        }
        for (v, key, most) in [
            (self.fold_train_waves, "fold_train_waves", 20.0),
            (self.fold_length_ratio, "fold_length_ratio", 100.0),
            (self.fold_shortening, "fold_shortening", 0.6),
            (self.normal_dip_deg, "normal_dip_deg", 89.0),
            (self.thrust_dip_deg, "thrust_dip_deg", 89.0),
            (self.dyke_half_width_blocks, "dyke_half_width_blocks", 16.0),
            (self.plug_radius_blocks, "plug_radius_blocks", 64.0),
        ] {
            let [a, b] = v;
            check(
                a > 0.0 && a <= b && b <= most,
                key,
                "[min, max] with 0 < min ≤ max, within the documented range",
            )?;
        }
        for (v, key) in [
            (self.fold_wavelength_sigma, "fold_wavelength_sigma"),
            (self.fault_length_sigma, "fault_length_sigma"),
            (self.fold_rate_km2_myr, "fold_rate_km2_myr"),
            (self.thrust_rate_km2_myr, "thrust_rate_km2_myr"),
            (self.normal_rate_km2_myr, "normal_rate_km2_myr"),
            (self.strike_slip_rate_km2_myr, "strike_slip_rate_km2_myr"),
            (self.trend_scatter_deg, "trend_scatter_deg"),
            (self.foreland_km, "foreland_km"),
            (self.overprint_myr, "overprint_myr"),
            (self.stock_rate_km2_myr, "stock_rate_km2_myr"),
            (self.dyke_rate_km2_myr, "dyke_rate_km2_myr"),
            (self.plug_radius_share, "plug_radius_share"),
        ] {
            check(v.is_finite() && v >= 0.0, key, "must be ≥ 0")?;
        }
        for (v, key) in [
            (self.normal_slip_ratio, "normal_slip_ratio"),
            (self.thrust_slip_ratio, "thrust_slip_ratio"),
            (self.strike_slip_ratio, "strike_slip_ratio"),
        ] {
            check((0.0..=0.3).contains(&v), key, "must be in [0, 0.3]")?;
        }
        check(
            self.fault_reach_ratio > 0.0 && self.fault_reach_ratio <= 2.0,
            "fault_reach_ratio",
            "must be in (0, 2]",
        )?;
        check(
            self.arc_uplift_mm_yr > 0.0 && self.arc_uplift_mm_yr.is_finite(),
            "arc_uplift_mm_yr",
            "must be > 0",
        )?;
        check(
            (0.0..=2.0).contains(&self.relief),
            "relief",
            "must be in [0, 2]",
        )?;
        check(
            self.relief_age_myr > 0.0 && self.relief_age_myr.is_finite(),
            "relief_age_myr",
            "must be > 0",
        )?;
        check(
            self.relief_wear > 0.0 && self.relief_wear.is_finite(),
            "relief_wear",
            "must be > 0",
        )?;
        check(
            (0.0..=1.0).contains(&self.fold_relief),
            "fold_relief",
            "must be in [0, 1]",
        )?;
        check(
            (0.0..=1.0).contains(&self.footwall_share),
            "footwall_share",
            "must be in [0, 1]",
        )?;
        check(
            (0.0..=10.0).contains(&self.intrusions),
            "intrusions",
            "must be in [0, 10]",
        )?;
        let [a, b] = self.stock_top_depth_blocks;
        check(
            a <= b && a.is_finite() && b.is_finite(),
            "stock_top_depth_blocks",
            "[min, max] with min ≤ max",
        )?;
        check(
            (1..=16).contains(&self.max_overlap),
            "max_overlap",
            "must be in [1, 16]",
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_and_ranges_are_checked() {
        let p = DeformationParams::default();
        p.validate().unwrap();
        assert!(p.enabled());
        let off = DeformationParams {
            strength: 0.0,
            ..p.clone()
        };
        off.validate().unwrap();
        assert!(!off.enabled());
        let bad = DeformationParams {
            fold_wavelength_km: [16.0, 20.0, 32.0],
            ..p.clone()
        };
        let e = bad.validate().unwrap_err().to_string();
        assert!(e.contains("fold_wavelength_km"), "{e}");
        let bad = DeformationParams {
            thrust_slip_ratio: 0.9,
            ..p
        };
        assert!(bad.validate().is_err());
        // Unknown keys are refused, known ones read.
        let read: DeformationParams = serde_json::from_str(r#"{"strength": 2.0}"#).unwrap();
        assert_eq!(read.strength, 2.0);
        assert!(serde_json::from_str::<DeformationParams>(r#"{"strenght": 2.0}"#).is_err());
    }
}

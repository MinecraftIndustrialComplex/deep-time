// SPDX-License-Identifier: LGPL-3.0-or-later
//! The preset's `structure.deformation` section (P2-01): how many folds and faults the tectonic
//! history leaves in the rock, how big they are and how strongly they deform it. Every key is
//! optional; the defaults are below with their sources.
//!
//! ```json
//! "structure": {"deformation": {"strength": 1.0, "density": 1.0, "fold_wavelength_km": [16, 8, 32]}}
//! ```
//!
//! **Sizes are true in block space.** A fold's wavelength and a fault's length are given in km on
//! the planet's horizontal block scale (300 m a block in the built-in presets), and amplitudes and
//! slips are that many *blocks* times the shape's own ratio (amplitude over wavelength, slip over
//! length), not metres at the vertical scale: the world is drawn 15 times taller than wide, and a
//! fold drawn with its true amplitude in metres would have vertical limbs. What is kept is the
//! shape a geologist would draw in a cross-section at 1:1.

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
    /// Tectonics 26:TC5006). The range is a design value about it. In buckling theory the
    /// wavelength follows the thickness of the stiff layers (Biot 1961, Geol. Soc. Am. Bull.
    /// 72:1595–1620, tested on natural folds by Sherwin & Chapple 1968, Am. J. Sci. 266:167–179);
    /// here it is drawn, not derived from the strata.
    pub fold_wavelength_km: [f64; 3],
    pub fold_wavelength_sigma: f64,
    /// Wavelengths across one fold train, [min, max]: a train is a bundle of a few parallel
    /// anticlines and synclines that dies out sideways.
    pub fold_train_waves: [f64; 2],
    /// A fold train's length along its axis over its wavelength, [min, max]. A design value after
    /// the long, doubly plunging anticlines of the Zagros Simple Folded Zone (its structural
    /// styles: Blanc, Allen, Inger & Hassani 2003, J. Geol. Soc. 160:401–412).
    pub fold_length_ratio: [f64; 2],
    /// Shortening taken up by folding, [least, most], as a fraction: a sine fold of amplitude A
    /// and wavelength λ has shortened by about (πA/λ)², so A/λ = √e/π. The Zagros Folded Belt
    /// has shortened by 16–19 % (Mouthereau et al. 2007) or about 20 % (McQuarrie 2004, J.
    /// Struct. Geol. 26:519–535). The least (a design value) is used in forelands, the most (a
    /// design value: limbs at about 45°) where a collision has thickened the crust the most;
    /// 1 − `structure.reference_crust_m` / the plate model's thickness gives the local value in
    /// between.
    pub fold_shortening: [f64; 2],
    /// Fold trains per km² of full-strength shortening per Myr. Design value: a train covers
    /// about 28 λ² (7,000 km² at 16 km), so 6e-6 covers a collision belt about 1.3 times over in
    /// 30 Myr.
    pub fold_rate_km2_myr: f64,

    /// Fault trace length, km: [median, min, max] of a lognormal with `fault_length_sigma`.
    /// Design values (segments of some tens of km); faults grow by the linkage of such segments
    /// (Gawthorpe & Leeder 2000, Basin Res. 12:195–218).
    pub fault_length_km: [f64; 3],
    pub fault_length_sigma: f64,
    /// How far into the hanging wall a fault's displacement reaches, as a share of its length:
    /// the width of the tilted block (a half-graben). A design value.
    pub fault_reach_ratio: f64,
    /// Greatest slip over trace length. A fault's greatest displacement grows in proportion to
    /// its length, D = γ·L (Cowie & Scholz 1992, J. Struct. Geol. 14:1149–1156), with γ about
    /// 0.03 for normal faults from millimetres to kilometres long (Schlische, Young, Ackermann &
    /// Gupta 1996, Geology 24:683–686) and a spread of an order of magnitude between data sets
    /// (reviewed by Kim & Sanderson 2005, Earth-Sci. Rev. 68:317–334). The values here are design
    /// values inside that spread, chosen so that offsets read at block scale.
    pub normal_slip_ratio: f64,
    pub thrust_slip_ratio: f64,
    pub strike_slip_ratio: f64,
    /// Dip of normal faults, degrees [min, max] (about 60°), and of thrusts (about 30°): the
    /// fault angles of Anderson 1951, The Dynamics of Faulting, 2nd ed.
    pub normal_dip_deg: [f64; 2],
    pub thrust_dip_deg: [f64; 2],
    /// Faults per km² of full-strength regime per Myr: thrusts where the crust shortens, normal
    /// faults where a continent rifts, strike-slip faults where convergence is oblique.
    pub thrust_rate_km2_myr: f64,
    pub normal_rate_km2_myr: f64,
    pub strike_slip_rate_km2_myr: f64,

    /// How far into the foreland of an active orogen the crust still shortens, km on the plate
    /// model's (Earth-normalized) sphere: the Zagros Simply Folded Belt is about 250 km wide, the
    /// Subandean belt 100–200 km. On a 16,384-block planet 250 km is about 100 blocks.
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
}

impl Default for DeformationParams {
    fn default() -> Self {
        DeformationParams {
            strength: 1.0,
            density: 1.0,
            size: 1.0,
            fold_wavelength_km: [16.0, 8.0, 32.0],
            fold_wavelength_sigma: 0.35,
            fold_train_waves: [2.0, 5.0],
            fold_length_ratio: [5.0, 12.0],
            fold_shortening: [0.08, 0.30],
            fold_rate_km2_myr: 6.0e-6,
            fault_length_km: [40.0, 15.0, 150.0],
            fault_length_sigma: 0.5,
            fault_reach_ratio: 0.4,
            normal_slip_ratio: 0.05,
            thrust_slip_ratio: 0.08,
            strike_slip_ratio: 0.05,
            normal_dip_deg: [55.0, 70.0],
            thrust_dip_deg: [20.0, 40.0],
            thrust_rate_km2_myr: 2.0e-5,
            normal_rate_km2_myr: 4.0e-5,
            strike_slip_rate_km2_myr: 4.0e-6,
            foreland_km: 250.0,
            arc_uplift_mm_yr: 0.1,
            max_overlap: 3,
            overprint_myr: 30.0,
            trend_scatter_deg: 12.0,
            max_structures: 60_000,
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

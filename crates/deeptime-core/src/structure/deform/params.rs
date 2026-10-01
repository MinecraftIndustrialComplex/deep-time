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
    /// Median 16 km: the Zagros Simply Folded Belt's folds are 15.8 ± 5.3 km apart (Mouthereau,
    /// Tensi, Bellahsen, Lacombe, De Boisgrollier & Kargar 2007, Tectonics 26:TC5006); the first
    /// -order folds of the Appalachian Valley and Ridge and of the Jura are 5–20 km. The wavelength
    /// follows the thickness of the stiff layers being buckled (Biot 1961, Geol. Soc. Am. Bull.
    /// 72:1595–1620; Sherwin & Chapple 1968, Am. J. Sci. 266:167–179).
    pub fold_wavelength_km: [f64; 3],
    pub fold_wavelength_sigma: f64,
    /// Wavelengths across one fold train, [min, max]: a train is a bundle of a few parallel
    /// anticlines and synclines that dies out sideways.
    pub fold_train_waves: [f64; 2],
    /// A fold train's length along its axis over its wavelength, [min, max]. Zagros anticlines are
    /// several times as long as they are wide and plunge out at both ends (Blanc, Allen, Inger &
    /// Hassani 2003, J. Geol. Soc. 160:401–412).
    pub fold_length_ratio: [f64; 2],
    /// Shortening taken up by folding, [least, most], as a fraction: a sine fold of amplitude A
    /// and wavelength λ has shortened by about (πA/λ)², so A/λ = √e/π. Fold-thrust belts shorten
    /// by roughly a tenth to a third (McQuarrie 2004, J. Struct. Geol. 26:519–535, for the
    /// Zagros); the least is used in forelands, the most where a collision has thickened the
    /// crust the most (1 − `structure.reference_crust_m` / the plate model's thickness gives the
    /// local value in between).
    pub fold_shortening: [f64; 2],
    /// Fold trains per km² of full-strength shortening per Myr. Design value: a train covers
    /// about 28 λ² (7,000 km² at 16 km), so 6e-6 covers a collision belt about 1.3 times over in
    /// 30 Myr.
    pub fold_rate_km2_myr: f64,

    /// Fault trace length, km: [median, min, max] of a lognormal with `fault_length_sigma`. Rift
    /// border-fault segments are some tens of km long before they link (Gawthorpe & Leeder 2000,
    /// Basin Res. 12:195–218).
    pub fault_length_km: [f64; 3],
    pub fault_length_sigma: f64,
    /// How far into the hanging wall a fault's displacement reaches, as a share of its length
    /// (the half-graben's width; rift half-grabens are about as wide as a third to a half of
    /// their border fault's length).
    pub fault_reach_ratio: f64,
    /// Greatest slip over trace length. Displacement scales with length, D = γ·L, with γ between
    /// about 0.01 and 0.1 (Cowie & Scholz 1992, J. Struct. Geol. 14:1149–1156; Schlische, Young,
    /// Ackermann & Gupta 1996, Geology 24:683–686: D = 0.03·L^1.06; Kim & Sanderson 2005, Earth
    /// -Sci. Rev. 68:317–334, who find thrusts at the high end).
    pub normal_slip_ratio: f64,
    pub thrust_slip_ratio: f64,
    pub strike_slip_ratio: f64,
    /// Dip of normal faults, degrees [min, max] (Andersonian ~60°), and of thrust ramps (~30°).
    pub normal_dip_deg: [f64; 2],
    pub thrust_dip_deg: [f64; 2],
    /// Faults per km² of full-strength regime per Myr: thrusts where the crust shortens, normal
    /// faults where a continent rifts, strike-slip faults where convergence is oblique.
    pub thrust_rate_km2_myr: f64,
    pub normal_rate_km2_myr: f64,
    pub strike_slip_rate_km2_myr: f64,

    /// Structures of one kind that may overlap at a place; a new one is not started where this
    /// many already are (strain goes on being taken up by the structures that exist).
    pub max_overlap: u32,
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
            max_overlap: 3,
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

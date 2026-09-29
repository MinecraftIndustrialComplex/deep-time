// SPDX-License-Identifier: LGPL-3.0-or-later
//! The preservation ledger (plan §8.6 step 6; Kesler & Wilkinson 2006, 2008;
//! `ore_genesis_modeling.md` Q5): formation depth and time, then burial B and erosion E at the
//! deposit's column since it formed, from the surface stage's rates. The body's centre now lies at
//! z_f + B − E below the surface:
//!
//! - its bottom above the surface → **eroded** (gone; resistates may shed placers);
//! - its top above the surface → **truncated** (cut at the surface; the tonnage left is the
//!   fraction of the primitive below the cut);
//! - otherwise **buried** at that depth (reachable or not is the engine's call).
//!
//! A type with `erosion_behavior: destroy` (a breached oil trap) counts any truncation as eroded.
//! The erosion history also gives the stability window a supergene blanket needs (`children.rs`).

use deeptime_format::bodies::Metric;

use super::types::ErosionBehavior;

/// The ledger of one deposit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ledger {
    /// Depth of the body's centre below the surface when it formed (m).
    pub formation_depth_m: f64,
    /// Half the body's vertical extent (m).
    pub half_height_m: f64,
    /// Erosion since formation (m).
    pub eroded_m: f64,
    /// Burial since formation (m).
    pub buried_m: f64,
}

/// What survives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fate {
    Buried {
        centre_depth_m: f64,
    },
    Truncated {
        centre_depth_m: f64,
        /// Fraction of the body (by volume) left below the surface.
        fraction_left: f64,
    },
    Eroded,
}

impl Fate {
    pub fn name(&self) -> &'static str {
        match self {
            Fate::Buried { .. } => "buried",
            Fate::Truncated { .. } => "truncated",
            Fate::Eroded => "eroded",
        }
    }

    pub fn fraction_left(&self) -> f64 {
        match self {
            Fate::Buried { .. } => 1.0,
            Fate::Truncated { fraction_left, .. } => *fraction_left,
            Fate::Eroded => 0.0,
        }
    }
}

/// Fraction of a primitive's volume below a horizontal cut at normalised height u ∈ [−1, 1]
/// (u = cut height above the centre / half-height): (2 + 3u − u³)/4 for an ellipsoid (a
/// spherical cap), (1 + u)/2 for a cylinder.
pub fn fraction_below(metric: Metric, u: f64) -> f64 {
    let u = u.clamp(-1.0, 1.0);
    match metric {
        Metric::Ellipsoid => (2.0 + 3.0 * u - u * u * u) / 4.0,
        Metric::Cylinder => (1.0 + u) / 2.0,
    }
}

impl Ledger {
    /// Present depth of the centre (m; negative = above the surface).
    pub fn centre_depth_m(&self) -> f64 {
        self.formation_depth_m + self.buried_m - self.eroded_m
    }

    pub fn apply(&self, behavior: ErosionBehavior, metric: Metric) -> Fate {
        let d = self.centre_depth_m();
        let h = self.half_height_m.max(1e-9);
        if d + h <= 0.0 {
            return Fate::Eroded;
        }
        if d - h >= 0.0 {
            return Fate::Buried { centre_depth_m: d };
        }
        if behavior == ErosionBehavior::Destroy {
            return Fate::Eroded;
        }
        // The surface cuts the body at height u = d / h above its centre.
        Fate::Truncated {
            centre_depth_m: d,
            fraction_left: fraction_below(metric, d / h),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ledger(eroded: f64, buried: f64) -> Ledger {
        Ledger {
            formation_depth_m: 2500.0,
            half_height_m: 750.0,
            eroded_m: eroded,
            buried_m: buried,
        }
    }

    /// Acceptance: the ledger buries, truncates and destroys in the documented cases.
    #[test]
    fn documented_cases() {
        // Untouched: the centre stays at 2.5 km.
        assert_eq!(
            ledger(0.0, 0.0).apply(ErosionBehavior::Truncate, Metric::Ellipsoid),
            Fate::Buried {
                centre_depth_m: 2500.0
            }
        );
        // Burial pushes it deeper.
        assert_eq!(
            ledger(100.0, 600.0).apply(ErosionBehavior::Truncate, Metric::Ellipsoid),
            Fate::Buried {
                centre_depth_m: 3000.0
            }
        );
        // 158 m/Myr (Kesler & Wilkinson 2006's porphyry exhumation rate) for 15 Myr removes
        // 2.37 km: the surface cuts the body just above its centre.
        let f = ledger(158.0 * 15.0, 0.0).apply(ErosionBehavior::Truncate, Metric::Ellipsoid);
        match f {
            Fate::Truncated {
                centre_depth_m,
                fraction_left,
            } => {
                assert!((centre_depth_m - 130.0).abs() < 1e-9);
                let u: f64 = 130.0 / 750.0;
                assert!((fraction_left - (2.0 + 3.0 * u - u.powi(3)) / 4.0).abs() < 1e-12);
                assert!(fraction_left > 0.5 && fraction_left < 0.7);
            }
            other => panic!("{other:?}"),
        }
        // …which a breached-trap type counts as destroyed.
        assert_eq!(
            ledger(2370.0, 0.0).apply(ErosionBehavior::Destroy, Metric::Ellipsoid),
            Fate::Eroded
        );
        // 3.25 km of erosion removes the whole body.
        assert_eq!(
            ledger(3250.0, 0.0).apply(ErosionBehavior::SpawnPlacer, Metric::Ellipsoid),
            Fate::Eroded
        );
        assert_eq!(fraction_below(Metric::Cylinder, 0.0), 0.5);
        assert_eq!(fraction_below(Metric::Ellipsoid, 1.0), 1.0);
        assert_eq!(fraction_below(Metric::Ellipsoid, -1.0), 0.0);
    }
}

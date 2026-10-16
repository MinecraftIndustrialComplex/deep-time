// SPDX-License-Identifier: LGPL-3.0-or-later
//! The preservation ledger (plan §8.6 step 6; Kesler & Wilkinson 2006, 2008;
//! `ore_genesis_modeling.md` Q5): formation depth and time, then burial B and erosion E at the
//! deposit's column since it formed, from the surface stage's rates. The body's centre now lies at
//! z_f + B − E below the surface. Its fate follows the deepest the surface ever cut towards it,
//! not the net: its centre was shallowest at z_f − X, X the most net erosion the column went
//! through at any time since formation (2026-10-05: before, a body exhumed through and buried
//! again counted as whole, the order of erosion and burial lost):
//!
//! - its bottom ever above the surface → **eroded** (gone; resistates may shed placers);
//! - its top ever above the surface → **truncated** (cut at that surface; the tonnage left is
//!   the fraction of the primitive below the cut, which later burial may have covered again);
//! - otherwise **buried** at its present depth (reachable or not is the engine's call).
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
    /// The most net erosion (erosion − burial since formation) at any time since formation (m,
    /// ≥ 0; at least `eroded_m − buried_m`).
    pub exhumed_max_m: f64,
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
        /// Depth below today's surface of the surface that cut it (m; 0 when that is today's,
        /// more where burial covered the cut again).
        cut_depth_m: f64,
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

    /// Depth below today's surface of the surface that cut the body (m; 0 unless truncated and
    /// buried again). The body is clipped there: an unconformity with the cover above it.
    pub fn cut_depth_m(&self) -> f64 {
        match self {
            Fate::Truncated { cut_depth_m, .. } => *cut_depth_m,
            _ => 0.0,
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

    /// Depth of the centre when the surface was nearest to it since formation (m).
    pub fn shallowest_centre_depth_m(&self) -> f64 {
        let net_exhumed = self.eroded_m - self.buried_m;
        self.formation_depth_m - self.exhumed_max_m.max(net_exhumed).max(0.0)
    }

    pub fn apply(&self, behavior: ErosionBehavior, metric: Metric) -> Fate {
        let d = self.centre_depth_m();
        let d_min = self.shallowest_centre_depth_m();
        let h = self.half_height_m.max(1e-9);
        if d_min + h <= 0.0 {
            return Fate::Eroded;
        }
        if d_min - h >= 0.0 {
            return Fate::Buried { centre_depth_m: d };
        }
        if behavior == ErosionBehavior::Destroy {
            return Fate::Eroded;
        }
        // The surface cut the body at height u = d_min / h above its centre.
        Fate::Truncated {
            centre_depth_m: d,
            fraction_left: fraction_below(metric, d_min / h),
            cut_depth_m: (d - d_min).max(0.0),
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
            exhumed_max_m: 0.0,
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
                cut_depth_m,
            } => {
                assert_eq!(cut_depth_m, 0.0);
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
        // The order counts (2026-10-05): 4 km of erosion and then 4 km of burial leave the net at
        // zero, but the surface went through the body: it is gone, not buried at 2.5 km.
        let through = Ledger {
            exhumed_max_m: 4000.0,
            ..ledger(4000.0, 4000.0)
        };
        assert_eq!(
            through.apply(ErosionBehavior::Truncate, Metric::Ellipsoid),
            Fate::Eroded
        );
        // 2.37 km of erosion, then 1 km of burial: cut where the 2.37 km left it, now 1 km under
        // the ground.
        let covered = Ledger {
            exhumed_max_m: 2370.0,
            ..ledger(2370.0, 1000.0)
        };
        match covered.apply(ErosionBehavior::Truncate, Metric::Ellipsoid) {
            Fate::Truncated {
                centre_depth_m,
                fraction_left,
                cut_depth_m,
            } => {
                assert!((centre_depth_m - 1130.0).abs() < 1e-9);
                assert!((cut_depth_m - 1000.0).abs() < 1e-9);
                let u: f64 = 130.0 / 750.0;
                assert!((fraction_left - (2.0 + 3.0 * u - u.powi(3)) / 4.0).abs() < 1e-12);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(fraction_below(Metric::Cylinder, 0.0), 0.5);
        assert_eq!(fraction_below(Metric::Ellipsoid, 1.0), 1.0);
        assert_eq!(fraction_below(Metric::Ellipsoid, -1.0), 0.0);
    }
}

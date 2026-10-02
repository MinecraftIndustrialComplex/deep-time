// SPDX-License-Identifier: LGPL-3.0-or-later
//! Crater shapes (P2-08): the radial profile of a crater in metres above the ground it struck, and
//! how the profile ages.
//!
//! Distances are block-metric (block offsets × `m_per_block_h`), so a crater is round in the world
//! whatever its latitude, like the volcanoes (`volcanoes/shape.rs`), whose rendering onto grids
//! (`cell_weight`, `blurred`, `Radial`) this module reuses. The profile is a function of
//! ρ = r / E, where E is the crater's **extent**: its rim radius R times `ejecta_radii`, where
//! the ejecta blanket ends. The rim is at ρ_r = R / E.
//!
//! **Fresh profile** (x = r / R; d the depth from rim crest to floor, h the rim's height, both
//! already scaled by `depth_scale`):
//! - simple: a parabolic bowl, h − d (1 − x²) (Melosh 1989, fig. 8.1: the apparent crater of a
//!   simple crater is close to a paraboloid);
//! - complex: a flat floor at h − d out to 0.55 R (a design value, IMP-051), then terraced walls
//!   rising to the rim (a smooth wall with the crest rounded; design, IMP-129), and a central peak
//!   0.22 R in radius (Pike 1985: the base of the central peak is 20–25 % of the rim diameter in
//!   small complex craters, read in its abstract);
//! - peak ring: the same floor and walls with a ring of peaks at 0.5 R (Pike 1985: the inner ring
//!   of a two-ring structure is 0.5 times the outer's diameter) instead of the central peak;
//! - outside the rim: the uplifted rim and ejecta, h x⁻³ (CMM05 eq. 43, after McGetchin, Settle &
//!   Head 1973: "a good approximation of data from explosion experiments … the power can vary
//!   between 2.5 and 3.5"), tapered to nothing between 0.7 E and E (design, IMP-135).
//!
//! **Ageing.** Hillslope diffusion of a landform for a time t is its convolution with a Gaussian
//! of variance 2 κ t (the Green's function of the linear diffusion equation; Culling 1960 and
//! Soderblom 1970, not read; crater degradation as diffusion: Fassett & Thomson 2014, abstract
//! read; ledger IMP-072). `aged` blurs the profile with a tent of the same variance (half-width
//! √(12 κ t); IMP-073): the rim rounds and lowers, the bowl shallows and widens, and a small crater
//! fades long before a large one. The diffusivity itself is unsourced (IMP-074).

use serde::{Deserialize, Serialize};

use super::scaling::Morphology;
use crate::surface::volcanoes::shape::{Radial, blurred};

/// Floor radius of a complex crater, as a share of the rim radius.
pub const FLOOR_RHO: f64 = 0.55; // ledger: IMP-051 (design value, uncited)
/// Central peak radius, as a share of the rim radius (Pike 1985: peak diameter 0.22 D).
pub const PEAK_RHO: f64 = 0.22; // ledger: IMP-048
/// Peak ring radius and half-width, as shares of the rim radius.
pub const RING_RHO: f64 = 0.5; // ledger: IMP-046
pub const RING_HALF: f64 = 0.1; // ledger: IMP-047 (design value, uncited)

/// A crater's fresh shape, in game metres.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub morphology: Morphology,
    /// Rim radius, m.
    pub radius_m: f64,
    /// Where the ejecta end, in rim radii.
    pub ejecta_radii: f64,
    /// Rim crest to floor, m.
    pub depth_m: f64,
    /// Rim crest above the ground struck, m.
    pub rim_m: f64,
    /// Central peak or peak ring above the floor, m.
    pub peak_m: f64,
}

fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl Profile {
    /// The extent E: where the ejecta end, m.
    pub fn extent_m(&self) -> f64 {
        self.radius_m * self.ejecta_radii
    }

    /// Height above the struck ground at `x` rim radii from the centre, m.
    pub fn at_rim_radii(&self, x: f64) -> f64 {
        let x = x.abs();
        let (d, h) = (self.depth_m, self.rim_m);
        if x >= 1.0 {
            if x >= self.ejecta_radii {
                return 0.0;
            }
            let taper = 1.0 - smooth(0.7 * self.ejecta_radii, self.ejecta_radii, x);
            return h * x.powi(-3) * taper;
        }
        match self.morphology {
            Morphology::Simple => h - d * (1.0 - x * x),
            Morphology::Complex | Morphology::PeakRing => {
                let t = ((x - FLOOR_RHO) / (1.0 - FLOOR_RHO)).clamp(0.0, 1.0);
                let wall = 0.6 * (t * t * (3.0 - 2.0 * t)) + 0.4 * t * t;
                let mut z = h - d + d * wall;
                if self.morphology == Morphology::Complex {
                    if x < PEAK_RHO {
                        z += self.peak_m * (1.0 - (x / PEAK_RHO).powi(2)).powi(2);
                    }
                } else {
                    let u = (x - RING_RHO) / RING_HALF;
                    if u.abs() < 1.0 {
                        z += self.peak_m * (1.0 - u * u).powi(2);
                    }
                }
                z
            }
        }
    }

    /// Height at ρ = r / E (0 beyond 1).
    pub fn at(&self, rho: f64) -> f64 {
        self.at_rim_radii(rho * self.ejecta_radii)
    }

    /// The profile after diffusion has spread it by a tent of half-width `spread_m` (0: fresh),
    /// as a table over ρ.
    pub fn aged(&self, spread_m: f64) -> Radial {
        let f = |rho: f64| self.at(rho);
        blurred(&f, (spread_m / self.extent_m()).max(0.0))
    }
}

/// The half-width of the tent that stands for diffusion with Σ κ·t = `kt_m2`: √(12 κ t), the tent
/// whose variance per axis (h²/6) is the Gaussian's 2 κ t.
pub fn spread_m(kt_m2: f64) -> f64 {
    (12.0 * kt_m2.max(0.0)).sqrt()
}

/// What is left of a crater's relief in a table: the highest point of the rim region less the
/// lowest point inside it, m.
pub fn relief_m(table: &Radial, rim_rho: f64) -> f64 {
    let n = 64;
    let (mut top, mut low) = (f64::NEG_INFINITY, f64::INFINITY);
    for k in 0..=n {
        let rho = 1.5 * rim_rho * f64::from(k) / f64::from(n);
        let z = table.at(rho);
        top = top.max(z);
        if rho <= rim_rho {
            low = low.min(z);
        }
    }
    (top - low).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple() -> Profile {
        Profile {
            morphology: Morphology::Simple,
            radius_m: 1000.0,
            ejecta_radii: 3.0,
            depth_m: 200.0,
            rim_m: 30.0,
            peak_m: 0.0,
        }
    }

    fn complex(m: Morphology) -> Profile {
        Profile {
            morphology: m,
            radius_m: 20_000.0,
            ejecta_radii: 3.0,
            depth_m: 800.0,
            rim_m: 200.0,
            peak_m: 400.0,
        }
    }

    #[test]
    fn a_bowl_has_a_floor_a_rim_and_thinning_ejecta() {
        let p = simple();
        assert!((p.at_rim_radii(0.0) - (30.0 - 200.0)).abs() < 1e-9, "floor");
        assert!((p.at_rim_radii(1.0) - 30.0).abs() < 1e-9, "rim crest");
        // Rising all the way from the centre to the rim, falling outside it to nothing.
        let mut last = f64::NEG_INFINITY;
        for k in 0..=100 {
            let z = p.at_rim_radii(f64::from(k) / 100.0);
            assert!(z >= last);
            last = z;
        }
        for k in 100..=300 {
            let z = p.at_rim_radii(f64::from(k) / 100.0);
            assert!(z <= last + 1e-12 && z >= 0.0);
            last = z;
        }
        assert!((p.at_rim_radii(2.0) - 30.0 / 8.0).abs() < 1e-9, "r^-3");
        assert_eq!(p.at_rim_radii(3.0), 0.0);
        assert_eq!(p.at(1.0), 0.0);
        assert!((p.at(1.0 / 3.0) - 30.0).abs() < 1e-9);
    }

    #[test]
    fn complex_craters_have_flat_floors_and_peaks_or_rings() {
        let c = complex(Morphology::Complex);
        let floor = 200.0 - 800.0;
        assert!((c.at_rim_radii(0.4) - floor).abs() < 1e-9, "flat floor");
        assert!((c.at_rim_radii(0.0) - (floor + 400.0)).abs() < 1e-9, "peak");
        assert!((c.at_rim_radii(1.0) - 200.0).abs() < 1e-9);
        let r = complex(Morphology::PeakRing);
        assert!(
            (r.at_rim_radii(0.0) - floor).abs() < 1e-9,
            "no central peak"
        );
        assert!((r.at_rim_radii(0.5) - (floor + 400.0)).abs() < 1e-9, "ring");
        assert!((r.at_rim_radii(0.3) - floor).abs() < 1.0);
    }

    #[test]
    fn diffusion_shallows_small_craters_long_before_large_ones() {
        let (small, big) = (simple(), complex(Morphology::Complex));
        let fresh = relief_m(&small.aged(0.0), 1.0 / 3.0);
        assert!((fresh - 200.0).abs() < 8.0, "{fresh}");
        // κ = 0.01 m²/yr for 10 Myr: a tent 1.1 km wide. The 2-km bowl loses a good part of its
        // relief; the 40-km crater hardly any.
        let s = spread_m(0.01 * 10.0e6);
        assert!((s - 1095.0).abs() < 1.0);
        let old = relief_m(&small.aged(s), 1.0 / 3.0);
        assert!(old < 0.7 * fresh && old > 0.0, "{old} of {fresh}");
        let big_fresh = relief_m(&big.aged(0.0), 1.0 / 3.0);
        let big_old = relief_m(&big.aged(s), 1.0 / 3.0);
        assert!(big_old > 0.9 * big_fresh, "{big_old} of {big_fresh}");
        // More time, less relief.
        let older = relief_m(&small.aged(spread_m(0.01 * 40.0e6)), 1.0 / 3.0);
        assert!(older < old);
        assert_eq!(spread_m(-1.0), 0.0);
    }
}

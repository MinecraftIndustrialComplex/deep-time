// SPDX-License-Identifier: LGPL-3.0-or-later
//! The spherical Fibonacci point set, L0's sample positions (plan §5.2).
//!
//! Point i of n sits at height z_i = 1 − (2i + 1)/n and longitude θ_i = 2π·frac(i/Φ), Φ the golden
//! ratio. Equal steps in z are equal areas on the sphere (Archimedes), so every point owns the same
//! area 4π/n, and the golden-ratio turn keeps neighbours apart; the set has no pole singularity and
//! no seam. Sources: Swinbank & Purser 2006 ("Fibonacci grids"), González 2010 ("Measurement of
//! areas on a sphere using Fibonacci and latitude–longitude lattices"), Keinert et al. 2015
//! ("Spherical Fibonacci mapping"); Cortial et al. 2019 sample their crust this way. Implemented
//! from the formula; no code was ported.
//!
//! The mean spacing is √(4π/n) radians ([`mean_spacing_rad`]): 0.0112 at 100k samples and 0.0071 at
//! 250k, i.e. 71 km and 45 km on Earth's 6,370 km radius, on which the plate model runs (§8.1).

use std::f64::consts::{PI, TAU};

use glam::DVec3;
use rayon::prelude::*;

/// 1/Φ = Φ − 1 = (√5 − 1)/2.
fn inv_phi() -> f64 {
    (5f64.sqrt() - 1.0) * 0.5
}

/// Point `i` of the `n`-point Fibonacci sphere, a unit vector (+Z is the north pole).
pub fn fibonacci_point(i: usize, n: usize) -> DVec3 {
    debug_assert!(i < n, "index {i} out of {n}");
    fibonacci_point_with(i, n, inv_phi())
}

fn fibonacci_point_with(i: usize, n: usize, inv_phi: f64) -> DVec3 {
    let z = 1.0 - (2 * i + 1) as f64 / n as f64;
    let r = (1.0 - z * z).max(0.0).sqrt();
    // frac(i/Φ) stays exact to ~1e-10 for i < 2^20: far below the spacing.
    let (s, c) = (TAU * (i as f64 * inv_phi).fract()).sin_cos();
    DVec3::new(r * c, r * s, z)
}

/// All `n` points of the Fibonacci sphere, north to south (z strictly decreasing).
pub fn fibonacci_sphere(n: usize) -> Vec<DVec3> {
    let k = inv_phi();
    (0..n)
        .into_par_iter()
        .map(|i| fibonacci_point_with(i, n, k))
        .collect()
}

/// The mean sample spacing √(4π/n) in radians (the side of a square of area 4π/n).
pub fn mean_spacing_rad(n: usize) -> f64 {
    (4.0 * PI / n as f64).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sphere::SampleTree;

    #[test]
    fn points_are_unit_ordered_and_deterministic() {
        let n = 10_000;
        let pts = fibonacci_sphere(n);
        assert_eq!(pts.len(), n);
        for (i, p) in pts.iter().enumerate() {
            assert!((p.length() - 1.0).abs() < 1e-14, "point {i} not unit");
            assert_eq!(*p, fibonacci_point(i, n));
        }
        assert!(pts.windows(2).all(|w| w[0].z > w[1].z));
        assert_eq!(pts, fibonacci_sphere(n));
    }

    /// Equal-area bands: any z-band holds a share of the points equal to its area share, to one
    /// point (z_i are the midpoints of n equal steps).
    #[test]
    fn bands_hold_their_area_share() {
        let n = 50_000;
        let pts = fibonacci_sphere(n);
        for k in 0..20 {
            let (lo, hi) = (-1.0 + k as f64 * 0.1, -1.0 + (k + 1) as f64 * 0.1);
            let count = pts.iter().filter(|p| p.z >= lo && p.z < hi).count() as f64;
            assert!((count - n as f64 * 0.05).abs() <= 1.0, "band {k}: {count}");
        }
    }

    /// Well spaced: every point's nearest neighbour lies between 0.6 and 1.6 mean spacings away.
    #[test]
    fn nearest_neighbour_spacing_is_uniform() {
        let n = 20_000;
        let pts = fibonacci_sphere(n);
        let tree = SampleTree::new(&pts);
        let spacing = mean_spacing_rad(n);
        let (mut lo, mut hi) = (f64::INFINITY, 0.0f64);
        for p in &pts {
            let nn = tree.nearest_n(*p, 2);
            let d = crate::sphere::kdtree::angle_of_chord2(nn[1].1) / spacing;
            lo = lo.min(d);
            hi = hi.max(d);
        }
        assert!(
            lo > 0.6 && hi < 1.6,
            "nearest-neighbour spacing ratio in [{lo}, {hi}]"
        );
    }
}

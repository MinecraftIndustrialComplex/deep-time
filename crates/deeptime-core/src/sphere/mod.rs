// SPDX-License-Identifier: LGPL-3.0-or-later
//! L0 geometry (plan §5.2): the Fibonacci sphere the plate model samples its crust on, the
//! spherical Delaunay triangulation of those samples, a k-d tree on their unit vectors, and the
//! coupling that samples the sphere onto the block-space rasters L1–L3.
//!
//! - [`fibonacci`] — the spherical Fibonacci point set (Cortial et al. 2019 sample their crust on
//!   it).
//! - [`hull`] — [`SphereMesh`], the spherical Delaunay triangulation as the convex hull of the unit
//!   vectors (randomized incremental hull with conflict buckets and exact predicates).
//! - [`kdtree`] — [`SampleTree`], nearest-sample queries (`kiddo`) on unit vectors.
//! - [`coupling`] — [`BlockFrame`] (block x/z ↔ latitude/longitude ↔ unit vector, with the seam
//!   wrap) and [`Coupling`] (raster cell → triangle, barycentric weights and nearest sample).
//! - [`around`]: [`Around`], the planet's metric about a point of the block frame (offsets east
//!   and south on the sphere), which draws features round on the planet.
//! - [`cube`] — [`CubeGrid`], the simulation's grid on Moser's equal-area cube (cube world plan
//!   §3, S1): six n × n faces, the D8 neighbours across the seams, the true metric, the ghost
//!   rings and Hilbert keys.
//!
//! Every function here is deterministic: parallel loops (Rayon, on the caller's pool) write
//! disjoint outputs, and the hull's insertion order comes from a seeded [`SplitMix64`].

pub mod around;
pub mod coupling;
pub mod cube;
pub mod fibonacci;
pub mod hull;
pub mod kdtree;

pub use around::{Around, conjugate_axes};
pub use coupling::{BlockFrame, CellLink, Coupling, lat_lon_deg, unit_from_lat_lon_deg};
pub use cube::{CubeGrid, CubeTopology, GhostTable};
pub use fibonacci::{fibonacci_point, fibonacci_sphere, mean_spacing_rad};
pub use hull::{Adjacency, HullError, SphereMesh};
pub use kdtree::SampleTree;

use glam::DVec3;

/// SplitMix64 (Steele, Lea & Flood 2014, "Fast splittable pseudorandom number generators"): the
/// small deterministic generator behind the hull's insertion order and the tests' random points.
/// Not for anything statistical beyond that (the plate model's Poisson draws and initial
/// shapes, P1-03, use it too; its state serialises into checkpoints).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SplitMix64(u64);

impl SplitMix64 {
    /// A generator seeded with `seed`.
    pub fn new(seed: u64) -> SplitMix64 {
        SplitMix64(seed)
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1), 53 bits.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Uniform in [0, n) for n > 0 (Lemire's multiply-shift; the bias is below 2⁻³² for n < 2³²).
    pub fn below(&mut self, n: u64) -> u64 {
        ((u128::from(self.next_u64()) * u128::from(n)) >> 64) as u64
    }

    /// Uniform in [lo, hi).
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next_f64()
    }

    /// A standard normal deviate (Box–Muller).
    pub fn normal(&mut self) -> f64 {
        let u = self.next_f64().max(f64::MIN_POSITIVE);
        let v = self.next_f64();
        (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
    }

    /// A uniformly distributed unit vector (Marsaglia 1972).
    pub fn unit_vector(&mut self) -> DVec3 {
        loop {
            let u = 2.0 * self.next_f64() - 1.0;
            let v = 2.0 * self.next_f64() - 1.0;
            let s = u * u + v * v;
            if s > 0.0 && s < 1.0 {
                let f = 2.0 * (1.0 - s).sqrt();
                return DVec3::new(u * f, v * f, 1.0 - 2.0 * s);
            }
        }
    }

    /// Shuffles `items` in place (Fisher–Yates).
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SplitMix64;

    #[test]
    fn splitmix_is_deterministic_and_in_range() {
        let mut a = SplitMix64::new(7);
        let mut b = SplitMix64::new(7);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
            let f = a.next_f64();
            assert!((0.0..1.0).contains(&f));
            b.next_f64();
            assert!(a.below(10) < 10);
            b.below(10);
            let v = a.unit_vector();
            assert!((v.length() - 1.0).abs() < 1e-12);
            b.unit_vector();
        }
        let mut v: Vec<u32> = (0..100).collect();
        SplitMix64::new(1).shuffle(&mut v);
        let mut sorted = v.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..100).collect::<Vec<_>>());
        assert_ne!(v, sorted);
    }
}

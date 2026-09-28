// SPDX-License-Identifier: LGPL-3.0-or-later
//! Nearest-sample queries on the unit sphere (plan §5.2): a `kiddo` k-d tree (MIT OR Apache-2.0)
//! on the samples' unit vectors.
//!
//! Distances are squared chords |p − q|² = 2 − 2 cos θ, which grow monotonically with the
//! great-circle angle θ, so the Euclidean nearest sample is the great-circle nearest one.
//! [`chord2_of_angle`] and [`angle_of_chord2`] convert.

use std::num::NonZeroUsize;

use glam::DVec3;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use rayon::prelude::*;

/// A static k-d tree over sample positions; the item of every result is the sample's index in
/// the slice the tree was built from. Rebuild it when samples move (a 250k build takes
/// milliseconds; see STATUS.md, P1-02).
pub struct SampleTree {
    tree: ImmutableKdTree<f64, 3>,
    len: usize,
}

impl SampleTree {
    /// Builds the tree over `points` (unit vectors), in parallel on the current Rayon pool.
    ///
    /// # Panics
    /// If there are 2³² or more points (sample indices are `u32`).
    pub fn new(points: &[DVec3]) -> SampleTree {
        let coords: Vec<[f64; 3]> = points.par_iter().map(|p| p.to_array()).collect();
        let tree =
            ImmutableKdTree::new_from_slice_parallel(&coords).expect("sample indices fit in u32");
        SampleTree {
            tree,
            len: points.len(),
        }
    }

    /// The number of samples.
    pub fn len(&self) -> usize {
        self.len
    }

    /// True when the tree holds no samples.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The nearest sample to `q` and its squared chord distance; `(u32::MAX, ∞)` for an empty
    /// tree. Ties go to whichever sample the tree meets first.
    pub fn nearest(&self, q: DVec3) -> (u32, f64) {
        if self.len == 0 {
            return (u32::MAX, f64::INFINITY);
        }
        let r = self
            .tree
            .query(&q.to_array())
            .nearest_one::<SquaredEuclidean<f64>>()
            .execute();
        (r.item, r.distance)
    }

    /// The `k` nearest samples to `q`, nearest first, with their squared chord distances.
    pub fn nearest_n(&self, q: DVec3, k: usize) -> Vec<(u32, f64)> {
        let Some(k) = NonZeroUsize::new(k.min(self.len)) else {
            return Vec::new();
        };
        self.tree
            .query(&q.to_array())
            .nearest_n::<SquaredEuclidean<f64>>(k)
            .execute()
            .into_iter()
            .map(|r| (r.item, r.distance))
            .collect()
    }

    /// Every sample within the great-circle angle `angle_rad` of `q`, nearest first, with their
    /// squared chord distances.
    pub fn within(&self, q: DVec3, angle_rad: f64) -> Vec<(u32, f64)> {
        if self.len == 0 {
            return Vec::new();
        }
        self.tree
            .query(&q.to_array())
            .within::<SquaredEuclidean<f64>>(chord2_of_angle(angle_rad))
            .execute()
            .into_iter()
            .map(|r| (r.item, r.distance))
            .collect()
    }

    /// The nearest sample of every query point, in parallel on the current Rayon pool.
    pub fn nearest_all(&self, queries: &[DVec3]) -> Vec<u32> {
        queries.par_iter().map(|&q| self.nearest(q).0).collect()
    }
}

/// The squared chord 4 sin²(θ/2) = 2 − 2 cos θ of the great-circle angle θ (for θ ≤ π).
pub fn chord2_of_angle(angle_rad: f64) -> f64 {
    let s = (0.5 * angle_rad.clamp(0.0, std::f64::consts::PI)).sin();
    4.0 * s * s
}

/// The great-circle angle of a squared chord, the inverse of [`chord2_of_angle`].
pub fn angle_of_chord2(chord2: f64) -> f64 {
    2.0 * (0.5 * chord2.max(0.0).sqrt()).min(1.0).asin()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sphere::{SplitMix64, fibonacci_sphere};

    /// The acceptance check of P1-02: nearest-sample queries agree with brute force on 10,000
    /// random points (on the 100k Fibonacci sphere and on 20k random samples).
    #[test]
    fn nearest_agrees_with_brute_force_on_10000_random_points() {
        let mut rng = SplitMix64::new(0xDEE9);
        let fib = fibonacci_sphere(100_000);
        let random: Vec<DVec3> = (0..20_000).map(|_| rng.unit_vector()).collect();
        for samples in [fib, random] {
            let tree = SampleTree::new(&samples);
            assert_eq!(tree.len(), samples.len());
            let queries: Vec<DVec3> = (0..10_000).map(|_| rng.unit_vector()).collect();
            let found = tree.nearest_all(&queries);
            let mismatches: Vec<usize> = (0..queries.len())
                .into_par_iter()
                .filter(|&k| {
                    let q = queries[k];
                    let (best, best_d) = samples
                        .iter()
                        .enumerate()
                        .map(|(i, p)| (i, p.distance_squared(q)))
                        .fold(
                            (usize::MAX, f64::INFINITY),
                            |a, b| if b.1 < a.1 { b } else { a },
                        );
                    let got = found[k] as usize;
                    // A tie (equal distance) may resolve to either sample.
                    got != best && (samples[got].distance_squared(q) - best_d).abs() > 1e-15
                })
                .collect();
            assert!(
                mismatches.is_empty(),
                "{} mismatches, first {:?}",
                mismatches.len(),
                mismatches.first()
            );
        }
    }

    #[test]
    fn nearest_n_and_within_match_brute_force() {
        let mut rng = SplitMix64::new(3);
        let samples: Vec<DVec3> = (0..5_000).map(|_| rng.unit_vector()).collect();
        let tree = SampleTree::new(&samples);
        for _ in 0..200 {
            let q = rng.unit_vector();
            let mut brute: Vec<(u32, f64)> = samples
                .iter()
                .enumerate()
                .map(|(i, p)| (i as u32, p.distance_squared(q)))
                .collect();
            brute.sort_by(|a, b| a.1.total_cmp(&b.1));
            let got = tree.nearest_n(q, 6);
            assert_eq!(got.len(), 6);
            for (g, b) in got.iter().zip(&brute) {
                assert!((g.1 - b.1).abs() < 1e-15);
            }
            let angle = 0.05;
            let inside = brute
                .iter()
                .filter(|b| b.1 <= chord2_of_angle(angle))
                .count();
            assert_eq!(tree.within(q, angle).len(), inside);
        }
        assert!(tree.nearest_n(DVec3::Z, 0).is_empty());
    }

    #[test]
    fn chord_and_angle_convert() {
        for k in 0..=100 {
            let a = std::f64::consts::PI * k as f64 / 100.0;
            assert!((angle_of_chord2(chord2_of_angle(a)) - a).abs() < 1e-7);
            let p = DVec3::X;
            let q = DVec3::new(a.cos(), a.sin(), 0.0);
            assert!((p.distance_squared(q) - chord2_of_angle(a)).abs() < 1e-14);
        }
    }

    #[test]
    fn empty_tree_answers_nothing() {
        let tree = SampleTree::new(&[]);
        assert!(tree.is_empty());
        assert_eq!(tree.nearest(DVec3::Z).0, u32::MAX);
        assert!(tree.within(DVec3::Z, 1.0).is_empty());
    }
}

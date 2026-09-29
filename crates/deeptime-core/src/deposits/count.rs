// SPDX-License-Identifier: LGPL-3.0-or-later
//! Counts (plan §8.6 step 2, the USGS three-part method's third part): per permissive tract
//! N ~ Poisson(λ), λ = ρ · A_tract · mean F · count_mult · E, with ρ the type's deposit density
//! (deposits per 100,000 km², its p50), A the tract's true area, mean F its area-weighted mean
//! favourability and E the tract's metallogenic-province endowment.
//!
//! **Provinces** (decisions file: regional specialisation configurable, default strong): the
//! planet is split into Voronoi provinces of about `province_km` across (seeds uniform on the
//! sphere); each province gets, per deposit type, an endowment E = exp(σz − σ²/2) with z standard
//! normal, so E has mean 1 and the planet's expected count stays the §8.6 formula's while some
//! provinces are rich in a type and others poor. σ is 0 (off), 0.35 (weak), 0.7 (medium) or 1.2
//! (strong) — design values, tuned by review.

use deeptime_format::GridDesc;
use glam::DVec3;
use rayon::prelude::*;

use super::random::{Rng, hash_str};
use super::types::Province;
use crate::sphere::{SplitMix64, unit_from_lat_lon_deg};

/// The §8.6 expectation λ = ρ/100,000 · A · F · count_mult · E.
pub fn expected(
    density_per_1e5km2: f64,
    area_km2: f64,
    mean_f: f64,
    count_mult: f64,
    endowment: f64,
) -> f64 {
    density_per_1e5km2 / 1.0e5 * area_km2 * mean_f * count_mult * endowment
}

/// σ of the log-endowment for a strength.
pub fn sigma(strength: Province) -> f64 {
    match strength {
        Province::Off => 0.0,
        Province::Weak => 0.35,
        Province::Medium => 0.7,
        Province::Strong => 1.2,
    }
}

/// The province map.
pub struct Provinces {
    /// Province index of every L1 cell.
    pub of_cell: Vec<u16>,
    pub count: usize,
    seed: u64,
}

impl Provinces {
    /// Voronoi provinces about `province_km` across on a planet of radius `radius_km`.
    pub fn new(grid: &GridDesc, seed: u64, province_km: f64, radius_km: f64) -> Provinces {
        let area = 4.0 * std::f64::consts::PI * radius_km * radius_km;
        let count = ((area / (province_km * province_km)).round() as usize).clamp(4, 4096);
        let mut sm = SplitMix64::new(seed ^ hash_str("provinces"));
        let seeds: Vec<DVec3> = (0..count).map(|_| sm.unit_vector()).collect();
        let c = grid.circumference() as f64;
        let of_cell = (0..grid.cells())
            .into_par_iter()
            .map(|cell| {
                let i = (cell % grid.w as usize) as i64;
                let j = (cell / grid.w as usize) as i64;
                let lon = 360.0 * (grid.block_x(i) as f64 + c / 2.0) / c;
                let lat = -(grid.block_z(j) as f64) / (c / 4.0) * 90.0;
                let u = unit_from_lat_lon_deg(lat, lon);
                let mut best = (0usize, f64::MIN);
                for (k, s) in seeds.iter().enumerate() {
                    let d = u.dot(*s);
                    if d > best.1 {
                        best = (k, d);
                    }
                }
                best.0 as u16
            })
            .collect();
        Provinces {
            of_cell,
            count,
            seed,
        }
    }

    /// Endowment of every province for `type_id` at `strength`.
    pub fn endowment(&self, type_id: &str, strength: Province) -> Vec<f64> {
        let s = sigma(strength);
        let mut rng = Rng::stream(self.seed, &format!("endowment/{type_id}"));
        (0..self.count)
            .map(|_| (s * rng.normal() - 0.5 * s * s).exp())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_is_the_formula() {
        // Plan §8.6: 29.7 M km² × 1 % permissive × 15 per 100,000 km² ≈ 45 (F = 1, E = 1).
        let l = expected(15.0, 29.7e6 * 0.01, 1.0, 1.0, 1.0);
        assert!((l - 44.55).abs() < 1e-9);
    }

    /// Acceptance: over 10,000 draws the Poisson counts average ρ·A·F within sampling error.
    #[test]
    fn poisson_counts_match_the_mean() {
        for (i, &(rho, area, f)) in [(15.0, 76_000.0, 0.42), (3.0, 20_000.0, 0.3), (60.0, 150_000.0, 0.8)]
            .iter()
            .enumerate()
        {
            let lambda = expected(rho, area, f, 1.0, 1.0);
            let draws = 10_000;
            let mut sum = 0.0;
            let mut sq = 0.0;
            for seed in 0..draws {
                let n = Rng::stream(seed, &format!("count-{i}")).poisson(lambda) as f64;
                sum += n;
                sq += n * n;
            }
            let mean = sum / draws as f64;
            let var = sq / draws as f64 - mean * mean;
            let se = (lambda / draws as f64).sqrt();
            assert!((mean - lambda).abs() < 4.0 * se, "λ={lambda}: mean {mean}");
            assert!((var / lambda - 1.0).abs() < 0.1, "λ={lambda}: variance {var}");
        }
    }

    #[test]
    fn provinces_cover_the_planet_and_endowments_average_one() {
        let grid = GridDesc::for_planet("L1", 4096, 16).unwrap();
        let p = Provinces::new(&grid, 3, 800.0, 4096.0 * 0.3 / (2.0 * std::f64::consts::PI));
        assert!(p.count >= 4);
        let used: std::collections::BTreeSet<u16> = p.of_cell.iter().copied().collect();
        assert!(used.len() > 2);
        let mut all = Vec::new();
        for k in 0..2000 {
            all.extend(p.endowment(&format!("t{k}"), Province::Strong));
        }
        let mean = all.iter().sum::<f64>() / all.len() as f64;
        assert!((mean - 1.0).abs() < 0.1, "{mean}");
        assert!(p.endowment("x", Province::Off).iter().all(|e| *e == 1.0));
    }
}

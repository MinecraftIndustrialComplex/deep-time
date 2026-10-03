// SPDX-License-Identifier: LGPL-3.0-or-later
//! The Earth-normalized mapping of plan §8.1 (decisions file, "Tectonics scaling").
//!
//! The plate model runs as if on a sphere of radius R_model = `tectonics.model_radius_km`
//! (6,370 km: Cortial's Earth, so plate count, speeds and ages are Earth's), on the unit sphere of
//! the L0 lattice. Its results map onto the real planet (radius R = C·300 m/2π) by latitude and
//! longitude, so every tectonic length shrinks by R/R_model (0.123 at 16k, 0.246 at 32k) while
//! elevations and uplift rates keep their physical values (they are vertical; the vertical scale,
//! `scale.m_per_block_v`, is the same everywhere). Events report areas and distances on the real planet (`*_km`,
//! `*_km2`) and speeds Earth-normalized (`*_mm_yr`), as the plates move in the model.
//!
//! Setting `model_radius_km` to the planet's own radius gives true-scale tectonics (the space-
//! planet parameterisation of the decisions file); nothing else changes.

use std::f64::consts::PI;

use glam::DVec3;

/// Lengths, areas and speeds between the model sphere, the unit lattice and the planet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Norm {
    /// R_model, km.
    pub model_km: f64,
    /// The planet's radius, km.
    pub planet_km: f64,
    /// L0 samples.
    pub n: usize,
}

impl Norm {
    pub fn new(model_km: f64, planet_km: f64, n: usize) -> Norm {
        Norm {
            model_km,
            planet_km,
            n,
        }
    }

    /// A model length (km) as an angle on the unit sphere.
    pub fn rad(&self, km: f64) -> f64 {
        km / self.model_km
    }

    /// An angle as a model length, km.
    pub fn km(&self, rad: f64) -> f64 {
        rad * self.model_km
    }

    /// The area one sample stands for on the model sphere, km² (Fibonacci points are equal-area).
    pub fn sample_area_km2(&self) -> f64 {
        4.0 * PI * self.model_km * self.model_km / self.n as f64
    }

    /// The mean sample spacing √(4π/n), radians.
    pub fn spacing_rad(&self) -> f64 {
        (4.0 * PI / self.n as f64).sqrt()
    }

    /// Model length → length on the planet, km.
    pub fn planet_len_km(&self, model_km: f64) -> f64 {
        model_km * self.planet_km / self.model_km
    }

    /// Model area → area on the planet, km².
    pub fn planet_area_km2(&self, model_km2: f64) -> f64 {
        let f = self.planet_km / self.model_km;
        model_km2 * f * f
    }

    /// |Ω × p| as a surface speed, mm/yr (1 km = 10⁶ mm).
    pub fn speed_mm_yr(&self, omega: DVec3, p: DVec3) -> f64 {
        omega.cross(p).length() * self.model_km * 1.0e6
    }

    /// The surface velocity Ω × p, mm/yr (a tangent vector).
    pub fn velocity_mm_yr(&self, omega: DVec3, p: DVec3) -> DVec3 {
        omega.cross(p) * (self.model_km * 1.0e6)
    }

    /// The angular speed (rad/yr) of a surface speed (mm/yr) at 90° from the pole.
    pub fn omega_of_speed(&self, mm_yr: f64) -> f64 {
        mm_yr / (self.model_km * 1.0e6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earth_normalized_scales() {
        // 16k singleplayer: R = 16,384 · 300 m / 2π = 782.3 km.
        let n = Norm::new(6370.0, 782.2784, 100_000);
        assert!((n.planet_len_km(6370.0) - 782.2784).abs() < 1e-9);
        assert!((n.planet_len_km(1800.0) / 1800.0 - 0.1228).abs() < 1e-3);
        assert!((n.sample_area_km2() * 100_000.0 - 4.0 * PI * 6370.0f64.powi(2)).abs() < 1.0);
        // 50 mm/yr at the equator of a pole on +Z.
        let w = DVec3::Z * n.omega_of_speed(50.0);
        assert!((n.speed_mm_yr(w, DVec3::X) - 50.0).abs() < 1e-9);
        assert!((n.km(n.rad(1234.0)) - 1234.0).abs() < 1e-9);
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! Insolation from latitude, the axial tilt and the star's flux (plan §8.3 item 1).
//!
//! Daily-mean insolation at the top of the atmosphere (Berger 1978; the form climlab's
//! `daily_insolation` evaluates, MIT) for a **circular orbit** (the preset has no eccentricity):
//!
//! Q(φ, λ) = S₀/π · (h₀ sin φ sin δ + cos φ cos δ sin h₀),  sin δ = sin ε · sin λ,
//! cos h₀ = −tan φ · tan δ (clamped: polar night h₀ = 0, polar day h₀ = π),
//!
//! with λ the solar longitude (0 at the northern spring equinox), ε the obliquity and h₀ the hour
//! angle of sunset. On a circular orbit λ advances uniformly, so the twelve "months" are equal 30°
//! steps of λ; month 0 starts at λ = 280°, about 1 January on Earth.

use std::f64::consts::PI;

/// Solar longitude at the start of month 0, radians (Earth's 1 January ≈ 280°).
pub const YEAR_START_RAD: f64 = 280.0 * PI / 180.0;

/// Daily-mean insolation, W/m², at latitude `lat` (rad) and solar longitude `lambda` (rad).
pub fn daily(s0: f64, lat: f64, lambda: f64, obliquity: f64) -> f64 {
    let sin_d = obliquity.sin() * lambda.sin();
    let d = sin_d.clamp(-1.0, 1.0).asin();
    let (sp, cp) = lat.sin_cos();
    // cos h₀ = −tan φ tan δ, written without the tangent so the poles need no special case.
    let h0 = if cp <= 1e-12 {
        if sp * sin_d > 0.0 { PI } else { 0.0 }
    } else {
        let c = -(sp / cp) * (sin_d / d.cos().max(1e-12));
        if c >= 1.0 {
            0.0
        } else if c <= -1.0 {
            PI
        } else {
            c.acos()
        }
    };
    (s0 / PI * (h0 * sp * sin_d + cp * d.cos() * h0.sin())).max(0.0)
}

/// Solar longitude at year fraction `f` (0 = the start of month 0).
pub fn lambda_at(f: f64) -> f64 {
    (YEAR_START_RAD + 2.0 * PI * f).rem_euclid(2.0 * PI)
}

/// Mean insolation over month `m` (0–11), `sub` samples per month.
pub fn monthly(s0: f64, lat: f64, m: usize, obliquity: f64, sub: usize) -> f64 {
    let sub = sub.max(1);
    (0..sub)
        .map(|k| {
            let f = (m as f64 + (k as f64 + 0.5) / sub as f64) / 12.0;
            daily(s0, lat, lambda_at(f), obliquity)
        })
        .sum::<f64>()
        / sub as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insolation_matches_the_known_limits() {
        let s0 = 1365.2;
        let eps = 23.4f64.to_radians();
        // At an equinox the equator gets S₀/π and the poles nothing.
        assert!((daily(s0, 0.0, 0.0, eps) - s0 / PI).abs() < 1e-9);
        assert!(daily(s0, PI / 2.0, 0.0, eps) < 1e-6);
        // At the June solstice the north pole gets S₀ sin ε (24 h of sun at elevation ε).
        let jun = PI / 2.0;
        assert!((daily(s0, PI / 2.0, jun, eps) - s0 * eps.sin()).abs() < 1e-6);
        assert_eq!(daily(s0, -PI / 2.0, jun, eps), 0.0, "polar night");
        // The global annual mean is S₀/4 (area-weighted over latitude, averaged over the year).
        let (mut sum, mut wsum) = (0.0, 0.0);
        for j in 0..180 {
            let lat = (-89.5 + j as f64).to_radians();
            let w = lat.cos();
            let q: f64 = (0..12).map(|m| monthly(s0, lat, m, eps, 30)).sum::<f64>() / 12.0;
            sum += w * q;
            wsum += w;
        }
        assert!((sum / wsum / (s0 / 4.0) - 1.0).abs() < 2e-3, "{}", sum / wsum);
        // No tilt: no seasons.
        let a = monthly(s0, 0.7, 0, 0.0, 10);
        let b = monthly(s0, 0.7, 6, 0.0, 10);
        assert!((a - b).abs() < 1e-9);
        // July is northern summer.
        assert!(monthly(s0, 0.8, 6, eps, 10) > 2.0 * monthly(s0, 0.8, 0, eps, 10));
    }
}

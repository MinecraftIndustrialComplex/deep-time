// SPDX-License-Identifier: LGPL-3.0-or-later
//! Orbital (Milankovitch) forcing for the recent ice-age window (deep-time climate, decisions log
//! 2026-09-29 "Recent ice-age cycles").
//!
//! Over the last `orbital_window_myr` of a run (3 Myr by default) the orbit varies as a short
//! quasi-periodic series with the leading periods of Berger (1978, "Long-term variations of daily
//! insolation and Quaternary climatic changes", J. Atmos. Sci. 35: 2362–2367) and Laskar et al.
//! (2004, A&A 428: 261–285) — a simple analytic stand-in, not the true Earth solution:
//!
//! - **obliquity** ε(t) = ε₀ + A_ε·[cos(2π(t − t_ε)/41 kyr) − cos(2π(−t_ε)/41 kyr)] +
//!   0.25·A_ε·[cos(2π(t − t_ε)/53.6 kyr) − cos(2π(−t_ε)/53.6 kyr)], A_ε = 1.2° (Earth's
//!   obliquity swings between about 22.1° and 24.5°), t_ε = −10 kyr (the last maximum, ~10 ka),
//!   so ε(0) is the preset's tilt;
//! - **eccentricity** e(t) = 0.028 − 0.011·cos(2π(t − t_e)/405 kyr) − 0.009·cos(2π(t − t_e)/95 kyr)
//!   − 0.007·cos(2π(t − t_e)/124 kyr), t_e = +27 kyr (Earth's next minimum), ≥ 0: between about 0
//!   and 0.055, e(0) ≈ 0.018 (Earth 0.0167);
//! - **precession**: the solar longitude of perihelion ϖ(t) = 283° + 360°·t / 23 kyr (perihelion in
//!   early January today, at the northern summer solstice ~11 ka).
//!
//! t is years relative to the end of the run (negative in the past). The obliquity's swing and the
//! eccentricity scale with `orbital_strength` (1 = Earth-like; 0 = a circular orbit at the
//! preset's tilt, no cycles).

use super::insolation::Orbit;

/// Obliquity swing at `orbital_strength` 1, degrees.
pub const OBLIQUITY_AMPLITUDE_DEG: f64 = 1.2;

/// The orbit at time `t_years` before (negative) the end of the run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitAt {
    pub obliquity_deg: f64,
    pub ecc: f64,
    pub long_peri_deg: f64,
}

impl OrbitAt {
    pub fn orbit(&self) -> Orbit {
        Orbit {
            ecc: self.ecc,
            long_peri_rad: self.long_peri_deg.to_radians(),
        }
    }
}

fn wave(t_kyr: f64, t0_kyr: f64, period_kyr: f64) -> f64 {
    (std::f64::consts::TAU * (t_kyr - t0_kyr) / period_kyr).cos()
}

/// The series at `t_years` (relative to the end of the run) for a planet whose present tilt is
/// `tilt_deg`, with the swings scaled by `strength`.
pub fn at(t_years: f64, tilt_deg: f64, strength: f64) -> OrbitAt {
    let t = t_years / 1000.0;
    let ob = |x: f64| {
        OBLIQUITY_AMPLITUDE_DEG
            * (wave(x, -10.0, 41.0) - wave(0.0, -10.0, 41.0)
                + 0.25 * (wave(x, -10.0, 53.6) - wave(0.0, -10.0, 53.6)))
    };
    let ecc_of = |x: f64| {
        0.028
            - 0.011 * wave(x, 27.0, 405.0)
            - 0.009 * wave(x, 27.0, 95.0)
            - 0.007 * wave(x, 27.0, 124.0)
    };
    OrbitAt {
        obliquity_deg: (tilt_deg + strength * ob(t)).clamp(0.0, 180.0),
        ecc: (strength * ecc_of(t)).clamp(0.0, 0.2),
        long_peri_deg: (283.0 + 360.0 * t / 23.0).rem_euclid(360.0),
    }
}

/// The orbit the run ends with (the present), used by snapshots outside the window too when the
/// window is on, so the present climate is the window's last state.
pub fn present(tilt_deg: f64, strength: f64) -> OrbitAt {
    at(0.0, tilt_deg, strength)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_series_has_earths_ranges_and_periods() {
        let p = present(23.44, 1.0);
        assert!((p.obliquity_deg - 23.44).abs() < 1e-12);
        assert!((p.ecc - 0.0167).abs() < 0.003, "{}", p.ecc);
        assert!((p.long_peri_deg - 283.0).abs() < 1e-9);
        let (mut lo, mut hi, mut emin, mut emax) = (90.0f64, 0.0f64, 1.0f64, 0.0f64);
        for k in 0..3000 {
            let o = at(-1000.0 * k as f64, 23.44, 1.0);
            lo = lo.min(o.obliquity_deg);
            hi = hi.max(o.obliquity_deg);
            emin = emin.min(o.ecc);
            emax = emax.max(o.ecc);
        }
        assert!(
            (21.5..22.6).contains(&lo) && (24.0..25.2).contains(&hi),
            "{lo}..{hi}"
        );
        assert!(
            emin < 0.01 && (0.035..0.06).contains(&emax),
            "{emin}..{emax}"
        );
        // Perihelion near the northern summer solstice (λ 90°) about 11 ka.
        let o = at(-11_500.0, 23.44, 1.0);
        assert!((o.long_peri_deg - 103.0).abs() < 1.0);
        // No swing at strength 0.
        let flat = at(-20_000.0, 23.44, 0.0);
        assert!((flat.obliquity_deg - 23.44).abs() < 1e-12 && flat.ecc == 0.0);
    }
}

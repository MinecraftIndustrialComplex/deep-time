// SPDX-License-Identifier: LGPL-3.0-or-later
//! Carbonate production: the light-limited growth law of Bosscher & Schlager 1992 ("Computer
//! simulation of reef growth", Sedimentology 39(3):503–512), G = G_m·tanh(I₀·e^(−kz) / I_k),
//! ported from Landlab's `CarbonateProducer` (https://github.com/landlab/landlab,
//! `src/landlab/components/carbonate/carbonate_producer.py`, MIT; the equations and defaults:
//! G_m 10 mm/yr, k 0.1 m⁻¹, I₀ 2,000 and I_k 400 µE/m²/s, the depth floored at −2·tidal range and a
//! logistic "smooth Heaviside" of width = tidal range across sea level).
//!
//! **Changes:** Landlab's rate applies everywhere the depth is positive; carbonate factories are
//! tropical and choke on mud, so here it is scaled by a water-temperature factor (0 below
//! `min_temp_c` − 6 °C, 1 above `min_temp_c`; the 18 °C of today's reef belt by default) and by
//! exp(−clastic / `clastic_m`) with the clastic sediment the cell received this step. Growth
//! stops at sea level: a step adds at most the water depth (Landlab's explicit rate times Δt would
//! overshoot at deep-time steps).

/// Parameters of the producer.
#[derive(Clone, Copy, Debug)]
pub struct CarbonateParams {
    /// G_m, m/yr.
    pub max_rate_m_yr: f64,
    /// k, 1/m.
    pub extinction_per_m: f64,
    pub surface_light: f64,
    pub saturating_light: f64,
    pub tidal_range_m: f64,
    pub min_temp_c: f64,
    pub clastic_m: f64,
}

/// Landlab's `smooth_heaviside`: 1 / (1 + e^(−2x/width)), a step when width is 0.
pub fn smooth_heaviside(x: f64, width: f64) -> f64 {
    if width > 0.0 {
        1.0 / (1.0 + (-2.0 / width * x).exp())
    } else if x > 0.0 {
        1.0
    } else if x < 0.0 {
        0.0
    } else {
        0.5
    }
}

/// Bosscher & Schlager's rate at water depth `z` (m, positive below sea level), m/yr.
pub fn rate(z: f64, p: &CarbonateParams) -> f64 {
    let z = z.max(-2.0 * p.tidal_range_m - 1.0e-6);
    let g = p.max_rate_m_yr
        * (p.surface_light * (-p.extinction_per_m * z).exp() / p.saturating_light).tanh();
    g * smooth_heaviside(z, p.tidal_range_m)
}

/// Thickness produced over `dt` years at depth `z` in water of `t_c` °C that received `clastic` m
/// of land-derived sediment this step, m (never more than the depth).
pub fn produce(z: f64, t_c: f64, clastic: f64, dt: f64, p: &CarbonateParams) -> f64 {
    if z <= 0.0 {
        return 0.0;
    }
    let warm = ((t_c - (p.min_temp_c - 6.0)) / 6.0).clamp(0.0, 1.0);
    if warm <= 0.0 {
        return 0.0;
    }
    let clear = if p.clastic_m > 0.0 {
        (-clastic.max(0.0) / p.clastic_m).exp()
    } else {
        1.0
    };
    (rate(z, p) * warm * clear * dt).min(z)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> CarbonateParams {
        CarbonateParams {
            max_rate_m_yr: 0.01,
            extinction_per_m: 0.1,
            surface_light: 2000.0,
            saturating_light: 400.0,
            tidal_range_m: 0.0,
            min_temp_c: 18.0,
            clastic_m: 20.0,
        }
    }

    #[test]
    fn bosscher_schlager_matches_its_formula() {
        let p = p();
        // tanh(2000 e^(−0.1·10) / 400) · 0.01
        let expect = 0.01 * (2000.0 * (-1.0f64).exp() / 400.0).tanh();
        assert!((rate(10.0, &p) - expect).abs() < 1e-12);
        assert!(rate(80.0, &p) < rate(10.0, &p));
        assert_eq!(rate(-5.0, &p), 0.0);
        // A warm clear lagoon 5 m deep fills to sea level in 2 Myr; cold or muddy water does not.
        assert!((produce(5.0, 26.0, 0.0, 2.0e6, &p) - 5.0).abs() < 1e-9);
        assert_eq!(produce(5.0, 5.0, 0.0, 2.0e6, &p), 0.0);
        assert!(produce(150.0, 26.0, 200.0, 1.0e4, &p) < 1e-3);
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! Weathering and regolith: soil production w = w₀·e^(−H/H*) (Ahnert 1976; Heimsath et al. 1997)
//! integrated over the step, ported from Landlab's `ExponentialWeathererIntegrated` and
//! `ExponentialWeatherer` (https://github.com/landlab/landlab, `src/landlab/components/weathering/`,
//! MIT; the equations): dH/dt = w₀·e^(−H/H*) gives H(t + Δt) = H*·ln(e^(H/H*) + w₀·Δt / H*).
//!
//! **Changes:** w₀ depends on climate (plan §8.2 couples the surface to the climate snapshot):
//! it scales with precipitation (P / 1,000 mm, 0.05–2) and doubles every 10 °C (an Arrhenius-like
//! factor, 2^((T − 15)/10)). The regolith's kind follows the research's duricrust rule
//! (`surface_processes_climate.md` §3–4: laterite where it is hot and wet with alternating dry
//! seasons, on gentle crests and plateaus, after long exposure): **laterite** when T > 20 °C,
//! P > 1,200 mm, slope < 1 % and H > 10 m, **bauxite** instead on felsic parents with P > 1,800 mm;
//! otherwise **saprolite** when H > 5 m and **soil** below. Thresholds are design values
//! (the literature gives conditions, not numbers; P1-07 tunes them).

/// Regolith kinds (the class it shows as).
pub const SOIL: u8 = 0;
pub const SAPROLITE: u8 = 1;
pub const LATERITE: u8 = 2;
pub const BAUXITE: u8 = 3;

/// H after `dt` years of production at the bare-rock rate `w0` (m/yr) with decay depth `h_star`.
pub fn integrate(h: f64, w0: f64, h_star: f64, dt: f64) -> f64 {
    if w0 <= 0.0 || h_star <= 0.0 {
        return h;
    }
    let x = h / h_star;
    if x > 50.0 {
        // e^x overflows long before production matters; H barely grows.
        return h + w0 * dt * (-x).exp();
    }
    h_star * (x.exp() + w0 * dt / h_star).ln()
}

/// The climate factor of w₀: precipitation and temperature.
pub fn climate_factor(precip_mm: f64, t_c: f64) -> f64 {
    (precip_mm / 1000.0).clamp(0.05, 2.0) * 2f64.powf((t_c - 15.0) / 10.0)
}

/// The regolith kind for thickness `h`, climate and slope; `felsic` for a felsic parent rock.
pub fn kind(h: f64, t_c: f64, precip_mm: f64, slope: f64, felsic: bool) -> u8 {
    if h > 10.0 && t_c > 20.0 && precip_mm > 1200.0 && slope < 0.01 {
        if felsic && precip_mm > 1800.0 {
            BAUXITE
        } else {
            LATERITE
        }
    } else if h > 5.0 {
        SAPROLITE
    } else {
        SOIL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration_matches_small_steps() {
        let (w0, hs) = (1.0e-4, 2.0);
        let big = integrate(0.5, w0, hs, 1.0e5);
        let mut h = 0.5;
        for _ in 0..100_000 {
            h += w0 * (-h / hs).exp();
        }
        assert!((big - h).abs() < 1e-3, "{big} vs {h}");
        assert_eq!(integrate(3.0, 0.0, hs, 1.0e6), 3.0);
        assert_eq!(kind(30.0, 26.0, 2000.0, 0.001, true), BAUXITE);
        assert_eq!(kind(30.0, 26.0, 1500.0, 0.001, true), LATERITE);
        assert_eq!(kind(6.0, 5.0, 500.0, 0.1, false), SAPROLITE);
    }
}

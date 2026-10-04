// SPDX-License-Identifier: LGPL-3.0-or-later
//! The CO₂ forcing at every concentration and surface temperature (P1-26 T6; Ben 2026-10-02,
//! "Research, then fix"; research note `docs/research/notes/high-co2-forcing-and-snowball-escape.md`):
//!
//! ```text
//! F(C, T) = m · [OLR_RC(T, 280 ppm) − OLR_RC(T, C)]
//! m = 5.35 ln 2 / [OLR_RC(288.15 K, 280 ppm) − OLR_RC(288.15 K, 560 ppm)]
//! ```
//!
//! OLR_RC is the clear-sky outgoing longwave of a radiative-convective model (Kopparapu et al.
//! 2013, ApJ 765: 131: 1 bar of N₂ plus CO₂, moist adiabat, saturated, CO₂ collision-induced
//! absorption) as tabulated by HEXTOR (Haqq-Misra & Hayworth 2022, PSJ; `radiation_N2_CO2_Sun.h5`,
//! commit f800e25f): T 190–370 K by 10 K, pCO₂ 10⁻⁴ to 10 bar. **The table (`co2_table.rs`) is
//! HEXTOR's data, MIT licence, Copyright (c) 2020 Blue Marble Space** (NOTICE.md). It is
//! interpolated linearly in T and in log₁₀ pCO₂ (HEXTOR's `getOLR` does the same), with pCO₂ =
//! C·10⁻⁶ bar: the model's ppm is a mole ratio, which the table's 1 bar of N₂ plus CO₂ reproduces
//! (its total pressure is a little below the true one, so pressure broadening is slightly
//! under-counted). m scales the clear-sky table so a doubling at 288.15 K gives Myhre et al.
//! 1998's all-sky 5.35 ln 2 = 3.71 W/m² (m = 0.903). F is zero at 280 ppm at every T, so Earth's
//! calibration is unchanged.
//!
//! Each tile's OLR is A + B·T − F(C, T) at its own temperature: the energy balances linearize F
//! about the tile's temperature at the step's start (it is piecewise linear in T, so exactly within
//! a 10-K segment). Beyond the table: T is clamped to 190–370 K for F; below its first column
//! (≈ 100 ppm) F continues as the logarithm with the table's own slope between ≈ 100 and ≈ 200 ppm;
//! above 10 bar it is clamped.

use std::sync::OnceLock;

/// The table's temperatures: 190 K and every 10 K to 370 K.
pub const T0_K: f64 = 190.0;
pub const DT_K: f64 = 10.0;
pub const NT: usize = 19;

include!("co2_table.rs");

/// 0 °C, K.
const KELVIN: f64 = 273.15;

fn log_levels() -> &'static [f64] {
    static L: OnceLock<Vec<f64>> = OnceLock::new();
    L.get_or_init(|| PCO2_BAR.iter().map(|p| p.log10()).collect())
}

/// OLR of table row `ti` at log₁₀ pCO₂ `lp` (clamped to the table), W/m².
fn olr_row(ti: usize, lp: f64) -> f64 {
    let l = log_levels();
    let n = l.len();
    let lp = lp.clamp(l[0], l[n - 1]);
    let k = match l.binary_search_by(|x| x.total_cmp(&lp)) {
        Ok(k) => k.min(n - 2),
        Err(k) => k.saturating_sub(1).min(n - 2),
    };
    let f = (lp - l[k]) / (l[k + 1] - l[k]);
    let row = &OLR_W_M2[ti];
    row[k] + f * (row[k + 1] - row[k])
}

/// The clear-sky forcing (before m) at every table temperature, of `ppm` inside the table.
fn raw_nodes(ppm: f64) -> [f64; NT] {
    let (l0, lc) = ((280.0e-6f64).log10(), (ppm * 1.0e-6).log10());
    std::array::from_fn(|ti| olr_row(ti, l0) - olr_row(ti, lc))
}

/// m: the clear-sky to all-sky scale (5.35 ln 2 over the table's doubling at 288.15 K).
pub fn scale_m() -> f64 {
    static M: OnceLock<f64> = OnceLock::new();
    // ledger: CLM-168
    *M.get_or_init(|| 5.35 * 2f64.ln() / Forcing::raw(560.0).at_k(288.15))
}

/// The forcing of one CO₂ level at every table temperature (W/m², × m); F(T) is linear between
/// them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Forcing {
    nodes: [f64; NT],
}

impl Forcing {
    fn raw(ppm: f64) -> Forcing {
        let c_min = PCO2_BAR[0] * 1.0e6;
        let nodes = if ppm >= c_min {
            raw_nodes(ppm)
        } else {
            // Below the table: the logarithm with the table's slope over its first doubling.
            // ledger: CLM-170
            let lo = raw_nodes(c_min);
            let c2 = PCO2_BAR[4] * 1.0e6; // ≈ 200 ppm
            let hi = raw_nodes(c2);
            let ln = (ppm.max(1e-3) / c_min).ln();
            std::array::from_fn(|i| lo[i] + (hi[i] - lo[i]) / (c2 / c_min).ln() * ln)
        };
        Forcing { nodes }
    }

    /// The forcing of `ppm` against 280 ppm (ledger: CLM-167).
    pub fn new(ppm: f64) -> Forcing {
        let m = scale_m();
        Forcing {
            nodes: Forcing::raw(ppm).nodes.map(|x| x * m),
        }
    }

    fn at_k(&self, t_k: f64) -> f64 {
        let x = ((t_k - T0_K) / DT_K).clamp(0.0, (NT - 1) as f64);
        let i = (x.floor() as usize).min(NT - 2);
        let f = x - i as f64;
        self.nodes[i] + f * (self.nodes[i + 1] - self.nodes[i])
    }

    /// F at surface temperature `t_c` °C, W/m².
    pub fn at(&self, t_c: f64) -> f64 {
        self.at_k(t_c + KELVIN)
    }

    /// F at `t_c` and its slope dF/dT there (0 beyond the table's temperatures), W/m² and
    /// W/m²/K.
    pub fn linear(&self, t_c: f64) -> (f64, f64) {
        let t_k = t_c + KELVIN;
        let x = (t_k - T0_K) / DT_K;
        let slope = if x <= 0.0 || x >= (NT - 1) as f64 {
            0.0
        } else {
            let i = (x.floor() as usize).min(NT - 2);
            (self.nodes[i + 1] - self.nodes[i]) / DT_K
        };
        (self.at_k(t_k), slope)
    }
}

/// A tile's longwave law about its step-start temperature `t0` °C: (A_eff, B_eff) with OLR ≈
/// A_eff + B_eff·T, the CO₂ forcing `co2` linearized about `t0` (B_eff kept ≥ 0.1 W/m²/K), and
/// the snowball cloud greenhouse `cloud` W/m² on the tile's `frozen` share subtracted
/// (`snowball_cloud_w_m2`, ledger: CLM-172).
pub fn longwave(
    a: f64,
    b: f64,
    co2: Option<&Forcing>,
    t0: f64,
    cloud: f64,
    frozen: f64,
) -> (f64, f64) {
    let (f0, df) = co2.map_or((0.0, 0.0), |c| c.linear(t0));
    let df = df.min((b - 0.1).max(0.0));
    (a - f0 + df * t0 - cloud * frozen, b - df)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_forcing_has_the_notes_anchors() {
        // Research note §3.2: m 0.903; F × m at (ppm, K).
        assert!((scale_m() - 0.903).abs() < 0.001, "{}", scale_m());
        let at = |ppm: f64, k: f64| Forcing::new(ppm).at(k - KELVIN);
        assert!((at(560.0, 288.15) - 3.708).abs() < 1e-3);
        for (ppm, k, want) in [
            (100.0, 230.0, -2.7),
            (1_000.0, 250.0, 4.8),
            (10_000.0, 288.15, 22.7),
            (50_000.0, 270.0, 34.4),
            (100_000.0, 250.0, 35.5),
            (300_000.0, 260.0, 62.7),
            (500_000.0, 270.0, 84.1),
            (1_000_000.0, 288.15, 123.4),
        ] {
            let f = at(ppm, k);
            assert!(
                (f - want).abs() < 0.15,
                "F({ppm}, {k}) = {f}, the note {want}"
            );
        }
        // Zero at 280 ppm at every temperature; growing with CO₂; continued below the table.
        for t in [-80.0, -20.0, 0.0, 15.0, 40.0, 120.0] {
            assert!(Forcing::new(280.0).at(t).abs() < 1e-12);
        }
        let mut prev = f64::MIN;
        // 10 ppm to 6·10⁶ ppm (6 bar: the table ends at 10).
        for k in 0..450 {
            let f = Forcing::new(10.0 * 1.03f64.powi(k)).at(5.0);
            assert!(f > prev, "monotone at {k}");
            prev = f;
        }
        assert!(Forcing::new(50.0).at(15.0) < Forcing::new(100.0).at(15.0));
        // The linearization reproduces the value and holds inside a segment.
        let f = Forcing::new(300_000.0);
        let (f0, s) = f.linear(-5.0);
        assert!((f0 - f.at(-5.0)).abs() < 1e-12);
        assert!((f.at(-7.0) - (f0 - 2.0 * s)).abs() < 1e-9);
    }

    #[test]
    fn the_longwave_law_keeps_a_positive_slope() {
        let f = Forcing::new(1_000_000.0);
        for t in [-60.0, -10.0, 0.0, 30.0] {
            let (a, b) = longwave(203.3, 2.09, Some(&f), t, 0.0, 0.0);
            assert!(b >= 0.1);
            // At t itself the linear law is the exact OLR.
            assert!((a + b * t - (203.3 + 2.09 * t - f.at(t))).abs() < 1e-9);
        }
        let (a, b) = longwave(203.3, 2.09, None, 0.0, 10.0, 0.5);
        assert_eq!((a, b), (198.3, 2.09));
    }
}

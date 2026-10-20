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
//!
//! **Pelagic carbonate** ([`pelagic`], 2026-10-19). Since the Mesozoic, planktic calcifiers
//! (coccolithophores, planktic foraminifera) rain calcareous ooze on the open sea floor, and about
//! as much carbonate accumulates there as on reefs, banks and shelves (Milliman and Droxler 1995,
//! after Milliman 1993: deep sea 11 of 32 × 10¹² mol/yr, slopes 6, the neritic rest 14.5). The ooze
//! survives only above the carbonate compensation depth (the CCD of the column builder,
//! `structure::columns::CCD_M`, RCK-015); between the lysocline and the CCD it dissolves more and
//! more. Before the calcifiers' clock (`deposits::clocks::Epoch::pelagic_calcifiers_at`) the deep
//! sea gets none (the Neritan ocean of Ridgwell 2005).

/// Pelagic carbonate above the lysocline once the calcifiers are in full swing, m/Myr of chalk:
/// the deep sea's 11 × 10¹² mol CaCO₃/yr over 290 × 10⁶ km² (Milliman and Droxler 1995, table 1)
/// is 1.4 m/Myr of calcite (2,710 kg/m³) over the whole deep floor, 3.1 m/Myr over the ~45 % of it
/// that is calcareous (Dutkiewicz et al. 2015, supplement table DR1; Diesing 2020: 37 %), so
/// 4.4 m/Myr of chalk at 30 % porosity (RCK-001: dry chalk 1,940 kg/m³, 28.8 %); rounded down
/// because the taper above the CCD thins it further (calcareous ooze 0.3–5 cm/kyr wet).
pub const PELAGIC_M_MYR: f64 = 4.0; // ledger: ERO-185
/// The lysocline lies this far above the CCD: the ooze's dissolution grows linearly from none at
/// the lysocline to all of it at the CCD.
pub const LYSOCLINE_ABOVE_CCD_M: f64 = 1000.0; // ledger: ERO-186

/// The share of the pelagic rain that survives at water depth `z` (m) over a CCD at `ccd_m`.
pub fn pelagic_share(z: f64, ccd_m: f64) -> f64 {
    if z <= 0.0 {
        return 0.0;
    }
    ((ccd_m - z) / LYSOCLINE_ABOVE_CCD_M).clamp(0.0, 1.0)
}

/// Chalk laid over `dt` years at water depth `z` (m) when the calcifiers are at `calcifiers`
/// (0–1) of their full rain, m (never more than the depth).
pub fn pelagic(z: f64, calcifiers: f64, dt: f64, ccd_m: f64) -> f64 {
    (PELAGIC_M_MYR * 1.0e-6 * dt * calcifiers.clamp(0.0, 1.0) * pelagic_share(z, ccd_m))
        .min(z.max(0.0))
}

/// The photic zone the factory's limits are measured over ([`limits`]), m: Bosscher and
/// Schlager's rate at 100 m is 2 × 10⁻⁴ of its surface value.
pub const PHOTIC_M: f64 = 100.0;

/// Whether a cell at depth `z` is in the photic zone, and its warmth and warmth × clarity factors
/// (`produce`'s), each 0 outside it (measurement only).
pub fn limits(z: f64, t_c: f64, clastic: f64, p: &CarbonateParams) -> [f64; 3] {
    if !(z > 0.0 && z <= PHOTIC_M) {
        return [0.0; 3];
    }
    let (warm, clear) = factors(t_c, clastic, p);
    [1.0, warm, warm * clear]
}

/// `produce`'s warmth and clarity factors.
fn factors(t_c: f64, clastic: f64, p: &CarbonateParams) -> (f64, f64) {
    let warm = ((t_c - (p.min_temp_c - 6.0)) / 6.0).clamp(0.0, 1.0);
    let clear = if p.clastic_m > 0.0 {
        (-clastic.max(0.0) / p.clastic_m).exp()
    } else {
        1.0
    };
    (warm, clear)
}

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
    let (warm, clear) = factors(t_c, clastic, p);
    if warm <= 0.0 {
        return 0.0;
    }
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

    #[test]
    fn pelagic_ooze_dissolves_between_the_lysocline_and_the_ccd() {
        let ccd = 4500.0;
        // Above the lysocline: the full rain, 4 m/Myr over 2 Myr.
        assert!((pelagic(3000.0, 1.0, 2.0e6, ccd) - 8.0).abs() < 1e-9);
        // Half way between the lysocline (3,500 m) and the CCD: half of it.
        assert!((pelagic(4000.0, 1.0, 2.0e6, ccd) - 4.0).abs() < 1e-9);
        // Below the CCD, on land, or before the calcifiers: none.
        assert_eq!(pelagic(5000.0, 1.0, 2.0e6, ccd), 0.0);
        assert_eq!(pelagic(-10.0, 1.0, 2.0e6, ccd), 0.0);
        assert_eq!(pelagic(3000.0, 0.0, 2.0e6, ccd), 0.0);
        // Never above the sea.
        assert!((pelagic(3.0, 1.0, 2.0e6, ccd) - 3.0).abs() < 1e-9);
    }
}

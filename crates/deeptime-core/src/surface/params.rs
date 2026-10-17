// SPDX-License-Identifier: LGPL-3.0-or-later
//! The preset's `surface` section (plan §5.1 `SurfaceParams`; P1-04): every key optional, the
//! defaults below. Rates are physical (m, yr); lengths are real metres at map scale (decisions
//! file, "Tectonics scaling": surface physics stays in real metres), so K and κ keep their usual
//! meaning on a 4.8 km (16k quick) or 2.4 km (16k full) cell.
//!
//! ```json
//! "surface": {"k_spl": 2e-9, "m_spl": 1.0, "n_spl": 2.0, "g_deposition": 1.0,
//!             "sea_level": {"base_m": 0, "cycles": [{"amplitude_m": 80, "period_myr": 250}]}}
//! ```

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// One sinusoidal sea-level cycle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeaCycle {
    pub amplitude_m: f64,
    pub period_myr: f64,
    #[serde(default)]
    pub phase_deg: f64,
}

/// Eustatic sea level (plan §8.3 item 7: "a curve from the preset"): `base_m` plus the cycles plus
/// the piecewise-linear `curve` of `[t_myr, m]` points (t relative to world creation, negative in
/// the past; held flat beyond its ends). Its cycles stand for the tectonic sea level (ridge volume,
/// the supercontinent cycle); with a climate stage the ice's glacio-eustasy is added to it
/// (`climate::ice`, `climate.glacio_eustasy`) and the surface takes the sum.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SeaLevel {
    pub base_m: f64,
    pub cycles: Vec<SeaCycle>,
    pub curve: Vec<[f64; 2]>,
}

impl SeaLevel {
    /// Sea level at `t_myr`, metres relative to the datum of the plate model's elevations.
    pub fn at(&self, t_myr: f64) -> f64 {
        let mut z = self.base_m;
        for c in &self.cycles {
            if c.period_myr > 0.0 {
                z += c.amplitude_m
                    * (std::f64::consts::TAU * t_myr / c.period_myr + c.phase_deg.to_radians())
                        .sin();
            }
        }
        if let (Some(first), Some(last)) = (self.curve.first(), self.curve.last()) {
            z += if t_myr <= first[0] {
                first[1]
            } else if t_myr >= last[0] {
                last[1]
            } else {
                self.curve
                    .windows(2)
                    .find(|w| t_myr <= w[1][0])
                    .map_or(last[1], |w| {
                        let f = (t_myr - w[0][0]) / (w[1][0] - w[0][0]).max(1e-12);
                        w[0][1] + f * (w[1][1] - w[0][1])
                    })
            };
        }
        z
    }
}

/// The surface section.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SurfaceParams {
    /// Stream-power K for crystalline basement (erodibility 1), m^(1−2m)/yr (Braun & Willett).
    /// The defaults are the nonlinear law n = 2, m = 1 (m/n = 0.5; worldwide ¹⁰Be denudation rates
    /// favour n > 1, Harel, Mudd & Attal 2016): with n = 1, m = 0.5 and any K that keeps orogens
    /// mountainous, continental rivers on gentle slopes planed whole continents to sea level
    /// within ~20 Myr in the 100–500-Myr tuning runs (STATUS.md, P1-04); with n = 2 steep orogens
    /// erode fast and low cratons slowly. K = 2·10⁻⁹ (m·yr)⁻¹ (P1-17 calibration with the real
    /// climate, Airy isostasy and the narrower Andean uplift of the pipeline presets: the land's
    /// median ends 0.6 km high and its 90th percentile ~2 km, Earth's 0.43 and 2.0 km; P1-04's
    /// 3·10⁻¹⁰ was set on the stand-in climate's wetter interiors and left 2.5-km plateaus).
    pub k_spl: f64,
    /// Area exponent m.
    pub m_spl: f64,
    /// Slope exponent n (Newton's method in the implicit solve when ≠ 1).
    pub n_spl: f64,
    /// Yuan et al. 2019 deposition coefficient G (0 = detachment-limited).
    pub g_deposition: f64,
    /// Runoff at which a cell counts its full area in the drainage area, m/yr.
    pub runoff_ref_m_yr: f64,
    /// Critical slope S_c (Landlab `ThresholdEroder`), m/m.
    pub threshold_slope: f64,
    /// Hillslope diffusivity κ for crystalline basement, m²/yr.
    pub kd_land_m2_yr: f64,
    /// Shallow-water marine diffusivity D₀ (Landlab `SimpleSubmarineDiffuser`), m²/yr: Landlab's
    /// default 100 (P1-17; P1-04 had 300). Faces take the mean of their cells' diffusivities, so
    /// the sea erodes a coastal land cell at about D₀/2: at 300 the shores retreated ~250 km in
    /// 500 Myr and drowned a third of the continental crust once the continents were Earth-low.
    pub kd_marine_m2_yr: f64,
    /// Wave base of the marine diffusivity, m (Landlab's default 60; P1-04 had 100).
    pub wave_base_m: f64,
    /// Tidal range (smooths the shoreline for diffusion and carbonates), m.
    pub tidal_range_m: f64,
    /// Depth below sea level river mouths fill to, m …
    pub marine_fill_depth_m: f64,
    /// … deepening with distance from the mouth by this gradient.
    pub shelf_slope: f64,
    /// The steepest slope sediment holds under the sea, degrees (`marine::slope_failure`): steeper
    /// sediment slumps and flows down to the slope's toe and the rise. 0 turns the failure off.
    /// Default 2.5°: the regional slope of the steepest clastic margin Pratson & Haxby (1996)
    /// measured (New Jersey–Maryland; the carbonate West Florida slope 4.4°, the others
    /// 0.5–2.0°); ETOPO 2022's continental slopes measured the same way as the planet's: median
    /// 2.2°, 90th percentile 5.2° (margins and spires task, 2026-10-03).
    pub submarine_slope_deg: f64, // ledger: ERO-168
    /// Bosscher–Schlager G_m, mm/yr (Landlab: 10).
    pub carbonate_max_mm_yr: f64,
    /// Light extinction k, 1/m (Landlab: 0.1).
    pub carbonate_extinction_per_m: f64,
    /// Surface light I₀ (Landlab: 2,000).
    pub carbonate_surface_light: f64,
    /// Saturating light I_k (Landlab: 400).
    pub carbonate_saturating_light: f64,
    /// Water temperature above which carbonate factories run at full rate, °C.
    pub carbonate_min_temp_c: f64,
    /// Clastic input that cuts carbonate production by e, m per step.
    pub carbonate_clastic_m: f64,
    /// Evaporite precipitation in restricted arid seas and closed lakes, m/Myr.
    pub evaporite_m_myr: f64,
    /// Bare-rock soil production w₀ at 1,000 mm/yr and 15 °C, m/Myr.
    pub weathering_max_m_myr: f64,
    /// Soil production decay depth H*, m.
    pub weathering_decay_depth_m: f64,
    /// Fraction of a load change compensated isostatically (the sea's water-loaded share is 0.7
    /// of it): Airy's ρ_crust / ρ_mantle ≈ 2.7 / 3.3 ≈ 0.8 (P1-17). P1-04 chose 0.6 because the
    /// plate model then uplifted whole continents (1,800-km Andean bands) and at steady state
    /// erosion = uplift / (1 − isostasy); with the pipeline presets' 800-km bands the uplift is
    /// confined to orogens, and 0.6 let erosion lower the cratons below the sea (a removed
    /// kilometre of crust lowers the ground 400 m at 0.6, 200 m at Airy's 0.8).
    pub isostasy: f64,
    /// Radius over which loads are spread before compensation (a flexural wavelength), km.
    pub isostasy_radius_km: f64,
    /// Eustatic sea level.
    pub sea_level: SeaLevel,
    /// K, layers per strata-log column (plan §5.7.4: 32).
    pub log_layers: u32,
    /// LEM steps per plate step.
    pub substeps: u32,
    /// Span of one frame of the deposit engine's field history, Myr.
    pub frame_every_myr: f64,
    /// Erosion rate that makes an erosion pulse, m/Myr (frame mean).
    pub erosion_pulse_m_myr: f64,
    /// Deposition rate that makes a subsiding basin, m/Myr (frame mean).
    pub basin_m_myr: f64,
    /// Smallest region of a pulse or basin, km².
    pub event_min_area_km2: f64,
    /// Change of the flooded share of continental crust that makes a transgression or regression.
    pub transgression_frac: f64,
    /// Iterations of Yuan et al.'s erosion–deposition solve (at most).
    pub spl_max_iter: u32,
    /// The drama slider (plan §8.2, P1-06; the editor's `surface.drama`): 0..=2, default 1. It
    /// scales the amplifier's thermal critical slope (`thermal::critical_slope`) and the
    /// amplitude of the sub-cell detail term (`detail.rs`).
    pub drama: f64,
    /// Glacial erosion, till and fjords (`glacial.rs`; Ben, 2026-09-29).
    pub glacial: super::glacial::GlacialParams,
    /// The plate's stiffness (`flexure.rs`; Ben, 2026-10-02: "General flexure everywhere").
    pub flexure: super::flexure::FlexureParams,
}

impl Default for SurfaceParams {
    fn default() -> Self {
        SurfaceParams {
            k_spl: 2.0e-9,
            m_spl: 1.0,
            n_spl: 2.0,
            g_deposition: 1.0,
            runoff_ref_m_yr: 0.5,
            threshold_slope: 0.35, // ledger: ERO-013 (uncited; see ERO-177)
            kd_land_m2_yr: 2.0, // ledger: ERO-023 (uncited; hillslope-scale 0.0036: ERO-177)
            kd_marine_m2_yr: 100.0,
            wave_base_m: 60.0,
            tidal_range_m: 2.0,
            marine_fill_depth_m: 0.0,
            shelf_slope: 0.0005,
            submarine_slope_deg: 2.5, // ledger: ERO-168
            carbonate_max_mm_yr: 10.0,
            carbonate_extinction_per_m: 0.1,
            carbonate_surface_light: 2000.0,
            carbonate_saturating_light: 400.0,
            carbonate_min_temp_c: 18.0,
            carbonate_clastic_m: 20.0,
            evaporite_m_myr: 100.0,
            weathering_max_m_myr: 50.0,
            weathering_decay_depth_m: 5.0,
            isostasy: 0.8,
            isostasy_radius_km: 50.0,
            sea_level: SeaLevel::default(),
            log_layers: 32,
            substeps: 1,
            frame_every_myr: 10.0,
            erosion_pulse_m_myr: 200.0,
            basin_m_myr: 80.0,
            event_min_area_km2: 3000.0,
            transgression_frac: 0.04,
            spl_max_iter: 6,
            drama: 1.0,
            glacial: super::glacial::GlacialParams::default(),
            flexure: super::flexure::FlexureParams::default(),
        }
    }
}

fn check(ok: bool, key: &str, reason: &str) -> Result<(), CoreError> {
    if ok {
        Ok(())
    } else {
        Err(CoreError::preset(format!("$.surface.{key}"), reason))
    }
}

impl SurfaceParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let pos = |v: f64, key: &str| check(v.is_finite() && v > 0.0, key, "must be > 0");
        let nonneg = |v: f64, key: &str| check(v.is_finite() && v >= 0.0, key, "must be ≥ 0");
        pos(self.k_spl, "k_spl")?;
        check(
            self.m_spl > 0.0 && self.m_spl < 1.5,
            "m_spl",
            "must be in (0, 1.5)",
        )?;
        check(
            self.n_spl >= 0.5 && self.n_spl <= 4.0,
            "n_spl",
            "must be in [0.5, 4]",
        )?;
        nonneg(self.g_deposition, "g_deposition")?;
        pos(self.runoff_ref_m_yr, "runoff_ref_m_yr")?;
        pos(self.threshold_slope, "threshold_slope")?;
        nonneg(self.kd_land_m2_yr, "kd_land_m2_yr")?;
        nonneg(self.kd_marine_m2_yr, "kd_marine_m2_yr")?;
        pos(self.wave_base_m, "wave_base_m")?;
        nonneg(self.tidal_range_m, "tidal_range_m")?;
        nonneg(self.marine_fill_depth_m, "marine_fill_depth_m")?;
        nonneg(self.shelf_slope, "shelf_slope")?;
        if !(self.submarine_slope_deg.is_finite()
            && (0.0..=30.0).contains(&self.submarine_slope_deg))
        {
            return Err(CoreError::preset(
                "$.surface.submarine_slope_deg",
                "must be in [0, 30] (0 = no slope failure)",
            ));
        }
        nonneg(self.carbonate_max_mm_yr, "carbonate_max_mm_yr")?;
        pos(
            self.carbonate_extinction_per_m,
            "carbonate_extinction_per_m",
        )?;
        nonneg(self.carbonate_surface_light, "carbonate_surface_light")?;
        pos(
            self.carbonate_saturating_light,
            "carbonate_saturating_light",
        )?;
        nonneg(self.carbonate_clastic_m, "carbonate_clastic_m")?;
        nonneg(self.evaporite_m_myr, "evaporite_m_myr")?;
        nonneg(self.weathering_max_m_myr, "weathering_max_m_myr")?;
        pos(self.weathering_decay_depth_m, "weathering_decay_depth_m")?;
        check(
            (0.0..=1.0).contains(&self.isostasy),
            "isostasy",
            "must be in [0, 1]",
        )?;
        nonneg(self.isostasy_radius_km, "isostasy_radius_km")?;
        check(
            (4..=255).contains(&self.log_layers),
            "log_layers",
            "must be in [4, 255]",
        )?;
        check(
            (1..=100).contains(&self.substeps),
            "substeps",
            "must be in [1, 100]",
        )?;
        pos(self.frame_every_myr, "frame_every_myr")?;
        pos(self.erosion_pulse_m_myr, "erosion_pulse_m_myr")?;
        pos(self.basin_m_myr, "basin_m_myr")?;
        nonneg(self.event_min_area_km2, "event_min_area_km2")?;
        pos(self.transgression_frac, "transgression_frac")?;
        check(
            (1..=50).contains(&self.spl_max_iter),
            "spl_max_iter",
            "must be in [1, 50]",
        )?;
        check(
            self.drama.is_finite() && (0.0..=2.0).contains(&self.drama),
            "drama",
            "must be in [0, 2]",
        )?;
        self.glacial.validate()?;
        self.flexure.validate()?;
        for c in &self.sea_level.cycles {
            check(
                c.amplitude_m.is_finite() && c.period_myr > 0.0,
                "sea_level.cycles",
                "amplitude finite, period > 0",
            )?;
        }
        check(
            self.sea_level
                .curve
                .windows(2)
                .all(|w| w[0][0] < w[1][0] && w[1][1].is_finite()),
            "sea_level.curve",
            "[t_myr, m] points with increasing t",
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sea_level_adds_cycles_and_the_curve() {
        let s = SeaLevel {
            base_m: 10.0,
            cycles: vec![SeaCycle {
                amplitude_m: 50.0,
                period_myr: 100.0,
                phase_deg: 0.0,
            }],
            curve: vec![[-100.0, 0.0], [0.0, 100.0]],
        };
        assert!((s.at(-100.0) - 10.0).abs() < 1e-9);
        // sin(2π·(−75)/100) = sin(−1.5π) = 1; the curve is a quarter of the way up.
        assert!((s.at(-75.0) - (10.0 + 50.0 + 25.0)).abs() < 1e-9);
        // Held at the last point beyond the curve; sin(π) = 0.
        assert!((s.at(50.0) - (10.0 + 100.0)).abs() < 1e-9);
        SurfaceParams::default().validate().unwrap();
        let bad = SurfaceParams {
            isostasy: 2.0,
            ..SurfaceParams::default()
        };
        assert!(bad.validate().is_err());
    }
}

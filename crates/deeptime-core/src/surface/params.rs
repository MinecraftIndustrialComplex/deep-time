// SPDX-License-Identifier: LGPL-3.0-or-later
//! The preset's `surface` section (plan §5.1 `SurfaceParams`; P1-04): every key optional, the
//! defaults below. Rates are physical (m, yr); lengths are real metres at map scale (decisions
//! file, "Tectonics scaling": surface physics stays in real metres), so K and κ keep their usual
//! meaning on a 4.8 km (16k quick) or 2.4 km (16k full) cell.
//!
//! ```json
//! "surface": {"k_spl": 1.5e-9, "m_spl": 1.0, "n_spl": 2.0, "g_deposition": 1.0}
//! ```

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// The surface section.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SurfaceParams {
    /// Stream-power K for crystalline basement (erodibility 1), m^(1−2m)/yr (Braun & Willett).
    /// Set against measured erosion rates (the erosion recalibration of 2026-10-18,
    /// erosion-and-sediment.md "The erosion law against measured rates"): with n = 2, m = 1 and
    /// S_c = 0.15 the planets' regions erode against their mean local relief as Montgomery &
    /// Brandon 2002 measured (their eq. 2 within 10 % below 1 km of relief, their power law within
    /// a factor 0.6 to 0.85 above; ERO-179, ERO-180). The hypsometry it was tuned to before
    /// (P1-17, 3·10⁻⁹) is a reference line now (HYP-001 to HYP-004). n = 2 lies inside the range
    /// fitted to the world's cosmogenic rates with hillslope diffusion (n ≈ 2.3, 1.59 to 2.70,
    /// Ruetenik et al. 2023, ERO-182), and n = 2.3 or 2.5 at the same erosion fitted no better;
    /// m = n / 2 keeps stream power the same at any drawn width. With n = 1, m = 0.5 gentle
    /// continental rivers planed whole continents in ~20 Myr (STATUS.md, P1-04).
    pub k_spl: f64,
    /// Area exponent m.
    pub m_spl: f64,
    /// Slope exponent n (Newton's method in the implicit solve when ≠ 1).
    pub n_spl: f64,
    /// Yuan et al. 2019 deposition coefficient G (0 = detachment-limited).
    pub g_deposition: f64,
    /// Runoff at which a cell counts its full area in the drainage area, m/yr.
    pub runoff_ref_m_yr: f64,
    /// Critical slope S_c (Landlab `ThresholdEroder`), m/m: Montgomery & Brandon's limiting local
    /// relief, 1,500 m over their 10-km circle (ERO-179), so no 10-km span stands more than Rc
    /// high at any cell size.
    pub threshold_slope: f64,
    /// Hillslope diffusivity κ for crystalline basement, m²/yr: at the map's scale, where it stands
    /// for every hillslope process below a cell; κ = 2 alone gives Montgomery & Brandon's
    /// quiet-land rate, 0.2 mm/yr per km of relief, on hills 9 km across (ERO-023, ERO-172).
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
    /// Clastic input that cuts carbonate production by e, m per Myr of the plate step so far (the
    /// clastic it is set against is what the cell received since the step began; 20 m over the
    /// 2-Myr step). Ledger ERO-065.
    pub carbonate_clastic_m_myr: f64,
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
    /// The surface holds the offset it carries over the plate model's height (in a gap between
    /// plates, on a trench row) instead of handing it to the plate model's feedback each step
    /// (`advect::held_cells`; flexure.md "The held offsets"). On by default since 2026-10-23.
    /// Off: the feedback takes the whole offset everywhere, as before. ledger: ISO-125
    pub held_offsets: bool,
    /// Lakes pass on what their water balance leaves (`closed.rs`): the stream power's drainage
    /// area leaves a lake's exit at the share of its inflow that flows out (Garcia-Castellanos
    /// 2006, eq. 2), none from a lake that cannot fill to its spill, an internally drained basin.
    /// Needs the hydrology's lake balance (`hydro.lake_balance`). Off by default (the plateau
    /// lane, 2026-10-31): every lake passes all of its inflow, as before. ledger: ERO-217, ERO-218
    pub closed_basins: bool,
    /// The stream power's runoff from the bake's water budget (`HydroParams::budget`, the lake
    /// balance's): rain only in the thawed share of the year, snow melted by its degree-days,
    /// evapotranspiration on the liquid water with PET from the biotemperature, about the
    /// climate stage's seasons. Off by default (the runoff lane, 2026-10-31): PET is 60 mm/°C x
    /// the mean annual temperature, 0 below 0 °C, and all of the precipitation of a land below
    /// freezing runs off, as before. ledger: ERO-221
    pub runoff_budget: bool,
    /// How the stage's solvers run (P1-19): numerics that change no law.
    pub numerics: Numerics,
}

/// The diffusion solve's preconditioner (`diffusion.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffusionSolver {
    /// Conjugate gradients preconditioned by one multigrid V-cycle (P1-19).
    Multigrid,
    /// Conjugate gradients preconditioned by the diagonal (P1-04 to P1-19).
    Jacobi,
}

/// The depression flood (`pflood.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloodMethod {
    /// The order-independent parallel flood (`pflood/parallel.rs`, P1-19).
    Parallel,
    /// The serial Priority-Flood+ε with its FIFO pit queue (P1-04 to P1-19).
    Serial,
}

/// The surface stage's numerical methods (P1-19, the 900-s gate run): each switch keeps the
/// method it replaced, so the two can be run side by side. None of them changes a law; they
/// change how closely, and how fast, the laws are solved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Numerics {
    /// The diffusion solve's preconditioner (`diffusion.rs`).
    pub diffusion_solver: DiffusionSolver,
    /// RMS residual of (I + Δt·L)h = h⁰ at which the multigrid diffusion solve stops, metres.
    /// The error of the solution is at most the residual (the matrix's eigenvalues are ≥ 1).
    /// 10⁻⁵ m leaves an RMS error of about 1.5·10⁻⁶ m and a largest of about 0.7 mm on the
    /// full 16k planet, against 3·10⁻⁷ m and 0.02 mm for the Jacobi solve at its 10⁻⁶ m
    /// (`CG_TOL_M`). The largest is about one step of the f32 the stage stores its elevations in
    /// (0.49 mm from 4 to 8 km), the RMS about a hundredth of one.
    pub diffusion_tol_m: f64, // ledger: ERO-183
    /// The depression flood (`pflood.rs`).
    pub flood: FloodMethod, // ledger: ERO-184
    /// The plate step whose carry blur the surface keeps per Myr at longer steps, Myr
    /// (`advect::blur_owed`; 0: none added, each carry blurs once whatever its length).
    pub carry_blur_ref_myr: f64, // ledger: ERO-185
    /// The longest step of the crust's spreading, Myr: a longer plate step solves it in as many
    /// equal steps as keep each this long or shorter (`spread.rs`; 0: one solve a plate step).
    pub spread_step_myr: f64, // ledger: ERO-186
}

impl Default for Numerics {
    fn default() -> Self {
        Numerics {
            diffusion_solver: DiffusionSolver::Multigrid,
            diffusion_tol_m: 1.0e-5,      // ledger: ERO-183
            flood: FloodMethod::Parallel, // ledger: ERO-184
            carry_blur_ref_myr: 2.0,      // ledger: ERO-185
            spread_step_myr: 2.0,         // ledger: ERO-186
        }
    }
}

impl Numerics {
    fn validate(&self) -> Result<(), CoreError> {
        check(
            self.diffusion_tol_m.is_finite()
                && self.diffusion_tol_m > 0.0
                && self.diffusion_tol_m <= 1.0,
            "numerics.diffusion_tol_m",
            "must be in (0, 1] m",
        )?;
        check(
            self.carry_blur_ref_myr.is_finite() && self.carry_blur_ref_myr >= 0.0,
            "numerics.carry_blur_ref_myr",
            "must be 0 or more Myr",
        )?;
        check(
            self.spread_step_myr.is_finite() && self.spread_step_myr >= 0.0,
            "numerics.spread_step_myr",
            "must be 0 or more Myr",
        )
    }

    /// Equal steps of the crust's spreading over a plate step of `dt_myr`.
    pub fn spread_steps(&self, dt_myr: f64) -> usize {
        if self.spread_step_myr > 0.0 {
            ((dt_myr / self.spread_step_myr) * (1.0 - 1.0e-9))
                .ceil()
                .max(1.0) as usize
        } else {
            1
        }
    }
}

impl Default for SurfaceParams {
    fn default() -> Self {
        SurfaceParams {
            k_spl: 1.5e-9, // ledger: ERO-004 (the presets' value, set against ERO-179 and ERO-180)
            m_spl: 1.0,    // ledger: ERO-005
            n_spl: 2.0,    // ledger: ERO-006
            g_deposition: 1.0,
            runoff_ref_m_yr: 0.5,
            threshold_slope: 0.15, // ledger: ERO-013 (Rc 1,500 m over 10 km, ERO-179)
            kd_land_m2_yr: 2.0,    // ledger: ERO-023 (map scale; hillslope-scale 0.0036: ERO-177)
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
            carbonate_clastic_m_myr: 10.0, // ledger: ERO-065
            evaporite_m_myr: 100.0,
            weathering_max_m_myr: 50.0,
            weathering_decay_depth_m: 5.0,
            isostasy: 0.8,
            isostasy_radius_km: 50.0,
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
            held_offsets: true,   // ledger: ISO-125
            closed_basins: false, // ledger: ERO-217 (off: the plateau lane, 2026-10-31)
            runoff_budget: false, // ledger: ERO-221 (off: the runoff lane, 2026-10-31)
            numerics: Numerics::default(),
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
        nonneg(self.carbonate_clastic_m_myr, "carbonate_clastic_m_myr")?;
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
        self.numerics.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_validate_and_a_bad_share_does_not() {
        SurfaceParams::default().validate().unwrap();
        let bad = SurfaceParams {
            isostasy: 2.0,
            ..SurfaceParams::default()
        };
        assert!(bad.validate().is_err());
    }

    /// ERO-186: the spreading takes one solve at a 2-Myr plate step (as before), two at 4, three
    /// at 5, and one whatever the step with the setting at 0.
    #[test]
    fn the_spreading_steps_at_most_the_setting() {
        let n = Numerics::default();
        assert!((n.spread_step_myr - 2.0).abs() < 1e-12);
        assert_eq!(n.spread_steps(2.0), 1);
        assert_eq!(n.spread_steps(1.0), 1);
        assert_eq!(n.spread_steps(1000.0 / 500.0), 1);
        assert_eq!(n.spread_steps(4.0), 2);
        assert_eq!(n.spread_steps(5.0), 3);
        let off = Numerics {
            spread_step_myr: 0.0,
            ..Numerics::default()
        };
        assert_eq!(off.spread_steps(4.0), 1);
    }
}

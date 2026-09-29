// SPDX-License-Identifier: LGPL-3.0-or-later
//! The preset's `climate` section (plan §5.1 `ClimateParams`; P1-05): every key optional, the
//! defaults below. The planet's `axial_tilt_deg`, `star_flux_rel` and `day_length_h` come from the
//! `planet` section.
//!
//! **Two length scales.** Planetary-scale climate — the energy balance by latitude, the wind belts,
//! how far moisture and maritime air reach inland, the ITCZ's smoothing — is Earth-normalized like
//! the plates (decisions file, "Tectonics scaling"): lengths ending in `_km` are Earth kilometres
//! and shrink by R/R⊕ on the small planet (0.123 at 16k), so a continent that spans 60° of longitude
//! has the dry interior an Earth continent of that angular size would. Orographic precipitation is
//! mesoscale physics on the terrain and runs in real metres at map scale, as the surface does.
//!
//! ```json
//! "climate": {"snapshot_every_myr": 10, "ocean_depth_m": 50, "moisture_decay_km": 2500}
//! ```

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// The climate section.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ClimateParams {
    /// A full climate snapshot every this many Myr (plan §8.3: 10), plus the first and the last
    /// plate step; between snapshots the climate's geographic part moves with the plates.
    pub snapshot_every_myr: f64,

    // --- The seasonal energy-balance model (climlab `EBM_seasonal`, MIT; `ebm.rs`).
    /// Solar constant at 1 AU, W/m² (climlab's `const.S0`); × `planet.star_flux_rel`.
    pub solar_constant_w_m2: f64,
    /// Outgoing longwave radiation A + B·T (North, Cahalan & Coakley 1981: 203.3 W/m²,
    /// 2.09 W/m²/K; climlab's defaults are 210 and 2).
    pub olr_a_w_m2: f64,
    pub olr_b_w_m2_k: f64,
    /// Meridional heat diffusivity D of the atmosphere, W/m²/K (North et al. 1981: 0.649;
    /// climlab 0.555).
    pub diffusivity_w_m2_k: f64,
    /// Ice-free planetary albedo a₀ + a₂·P₂(sin φ) (climlab `EBM_seasonal`: 0.33, 0.25).
    pub albedo_a0: f64,
    pub albedo_a2: f64,
    /// Albedo of snow, sea ice and ice sheets (climlab `ai` 0.62).
    pub albedo_ice: f64,
    /// Surface temperature at the middle of the ice-albedo step, °C (climlab's T_f −10 °C; smoothed
    /// over ±3 °C).
    pub ice_temp_c: f64,
    /// Heat capacity of the ocean mixed layer, metres of water (a slab ocean).
    pub ocean_depth_m: f64,
    /// Heat capacity of land with the air column above it, metres of water.
    pub land_depth_m: f64,
    /// Heat exchange between the land and the sea of one latitude band, W/m²/K (the zonal winds
    /// mixing continental and maritime air; the continentality of each cell decides how much of
    /// the land's own climate it gets, `lapse.rs`).
    pub land_ocean_exchange_w_m2_k: f64,
    /// Poleward heat transport of the slab ocean (diffusion through the sea), W/m²/K.
    pub ocean_diffusivity_w_m2_k: f64,
    /// How much the ice sheets of the previous snapshot brighten the EBM's land (0–1). 1 by
    /// default since the P1-17 calibration made the continents Earth-low (median 0.6–1.1 km): six
    /// 500-Myr pipeline runs end with 3–7 % of the land glaciated, 18 % on the coldest (Earth today
    /// ≈ 10 %, Antarctica and Greenland; on Earth's relief the model gives 9.7 %). While P1-04's
    /// continents were domed (3–4 km interiors) it froze 46 % of the land, so it was off (P1-05).
    /// Snow and sea ice keep their albedo feedback through the temperature either way.
    pub ice_sheet_feedback: f64,
    /// Latitude bands of the EBM (climlab `num_lat` 90).
    pub ebm_bands: u32,
    /// Time steps per model year (a multiple of 12).
    pub ebm_steps_per_year: u32,
    /// Years run at the first snapshot, and from the previous state at later ones (at most; the
    /// run stops early once the annual means change by less than 0.005 K).
    pub ebm_spinup_years: u32,
    pub ebm_warm_years: u32,
    /// Environmental lapse rate, °C/km.
    pub lapse_c_per_km: f64,

    // --- Circulation belts (`belts.rs`).
    pub trade_wind_m_s: f64,
    pub westerly_m_s: f64,
    pub polar_wind_m_s: f64,
    /// Largest seasonal shift of the ITCZ from the equator, degrees (it follows the thermal
    /// equator: ~5° over oceans, up to this over large continents — monsoons).
    pub itcz_max_shift_deg: f64,
    /// Smoothing of the ITCZ along longitude, Earth km.
    pub itcz_smooth_km: f64,
    /// Seasonal shift of the subtropical highs and the westerlies at 23.4° of tilt, degrees
    /// (scaled by sin tilt / sin 23.4°).
    pub subtropical_shift_deg: f64,

    // --- Moisture and continentality along the wind (`lapse.rs`), Earth km.
    /// Moisture over land decays toward `moisture_floor` over this distance downwind.
    pub moisture_decay_km: f64,
    /// Moisture left far inland (recycling by the land's own evaporation), 0–1.
    pub moisture_floor: f64,
    /// Where the belts are wet, evaporation from the land recycles the rain and moisture reaches
    /// further inland: the decay length × (1 + recycling × P_belt/P_belt(0)).
    pub recycling: f64,
    /// Air over the sea regains moisture over this distance.
    pub ocean_recharge_km: f64,
    /// Maritime (oceanic) air loses its influence on land temperatures over this distance.
    pub maritime_decay_km: f64,
    /// Proximity to the sea within this radius moderates land temperatures whatever the wind.
    pub coast_km: f64,
    /// The air's moisture and maritime influence are smoothed over this radius (winds are not
    /// purely zonal and mix across latitudes).
    pub air_mixing_km: f64,
    /// Rain shadow: air that crossed a barrier Δz higher than the ground keeps e^(−Δz/scale) of
    /// its moisture (the water-vapour scale height).
    pub vapour_scale_m: f64,
    /// A barrier's shadow fades over this distance downwind.
    pub barrier_recover_km: f64,
    /// Precipitation dries above this elevation with this e-folding height, m (high plateaus).
    pub plateau_m: f64,
    pub plateau_scale_m: f64,

    // --- Ocean gyres and the subtropical highs (`belts.rs`).
    /// Sea-surface anomalies at the coasts of wide ocean basins, °C: subtropics (15–35°) cold at
    /// the basin's eastern side (upwelling along continents' west coasts) and warm at its western
    /// side; mid latitudes (45–65°) warm at the eastern side (the North Atlantic Drift's role) and
    /// cold at the western side (Worldbuilding Pasta's rules of thumb, research note §5).
    pub sst_subtropical_cold_c: f64,
    pub sst_subtropical_warm_c: f64,
    pub sst_midlatitude_warm_c: f64,
    pub sst_midlatitude_cold_c: f64,
    /// Drying under the eastern side of the subtropical highs (west coasts), 0–1.
    pub subsidence: f64,
    /// Summer onshore (easterly) moisture on east coasts from 15° to 45° — the highs' western
    /// flanks and the monsoons that make Earth's humid subtropical east coasts, 0–1.
    pub onshore: f64,

    // --- Orographic precipitation (Smith & Barstad 2004; fastscape-lem/orographic-precipitation
    // defaults, MIT; `orographic.rs`).
    pub oro_conv_time_s: f64,
    pub oro_fall_time_s: f64,
    /// Moist stability (Brunt–Väisälä) frequency, 1/s.
    pub oro_nm: f64,
    /// Water-vapour scale height, m.
    pub oro_hw_m: f64,
    /// Uplift sensitivity c_w = ρ_s·Γ_m/Γ at `oro_ref_temp_c`, kg/m³ (7.4e-3 × 7/4).
    pub oro_cw_kg_m3: f64,
    /// Temperature c_w refers to; it scales with the local temperature by Clausius–Clapeyron.
    pub oro_ref_temp_c: f64,
    /// Hours per day the prevailing wind rains on the terrain (the orographic rate is an
    /// event rate; fastscape's `rainfall_frequency`).
    pub oro_hours_per_day: f64,
    /// The lee of a range keeps at least this share of its background precipitation (the
    /// linear theory's negative rates are applied proportionally and saturate here).
    pub oro_lee_floor: f64,
    /// Smallest precipitation anywhere, mm/yr.
    pub precip_min_mm: f64,

    // --- Glaciers (`glaciers.rs`).
    /// Added to the ELA temperature of Ohmura, Kasser & Funk 1992, °C.
    pub ela_offset_c: f64,
    /// A glaciation starts when glaciers cover this share of the land, and ends below the second.
    pub glaciation_on_frac: f64,
    pub glaciation_off_frac: f64,
}

impl Default for ClimateParams {
    fn default() -> Self {
        ClimateParams {
            snapshot_every_myr: 10.0,
            solar_constant_w_m2: 1365.2,
            olr_a_w_m2: 203.3,
            olr_b_w_m2_k: 2.09,
            diffusivity_w_m2_k: 0.649,
            albedo_a0: 0.33,
            albedo_a2: 0.25,
            albedo_ice: 0.62,
            ice_temp_c: -10.0,
            ocean_depth_m: 50.0,
            land_depth_m: 3.0,
            land_ocean_exchange_w_m2_k: 10.0,
            ocean_diffusivity_w_m2_k: 0.3,
            ice_sheet_feedback: 1.0,
            ebm_bands: 90,
            ebm_steps_per_year: 72,
            ebm_spinup_years: 40,
            ebm_warm_years: 6,
            lapse_c_per_km: 6.5,
            trade_wind_m_s: 7.0,
            westerly_m_s: 10.0,
            polar_wind_m_s: 5.0,
            itcz_max_shift_deg: 22.0,
            itcz_smooth_km: 1500.0,
            subtropical_shift_deg: 6.0,
            moisture_decay_km: 3500.0,
            moisture_floor: 0.3,
            recycling: 1.5,
            ocean_recharge_km: 600.0,
            maritime_decay_km: 1200.0,
            coast_km: 250.0,
            air_mixing_km: 300.0,
            vapour_scale_m: 2500.0,
            barrier_recover_km: 1500.0,
            plateau_m: 1500.0,
            plateau_scale_m: 2500.0,
            sst_subtropical_cold_c: 5.0,
            sst_subtropical_warm_c: 3.0,
            sst_midlatitude_warm_c: 6.0,
            sst_midlatitude_cold_c: 5.0,
            subsidence: 0.55,
            onshore: 1.0,
            oro_conv_time_s: 1000.0,
            oro_fall_time_s: 1000.0,
            oro_nm: 0.01,
            oro_hw_m: 3400.0,
            oro_cw_kg_m3: 0.01295, // 7.4e-3 × 7/4
            oro_ref_temp_c: 6.0,
            oro_hours_per_day: 0.5,
            oro_lee_floor: 0.3,
            precip_min_mm: 10.0,
            ela_offset_c: 0.0,
            glaciation_on_frac: 0.02,
            glaciation_off_frac: 0.01,
        }
    }
}

fn check(ok: bool, key: &str, reason: &str) -> Result<(), CoreError> {
    if ok {
        Ok(())
    } else {
        Err(CoreError::preset(format!("$.climate.{key}"), reason))
    }
}

impl ClimateParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let pos = |v: f64, key: &str| check(v.is_finite() && v > 0.0, key, "must be > 0");
        pos(self.snapshot_every_myr, "snapshot_every_myr")?;
        pos(self.solar_constant_w_m2, "solar_constant_w_m2")?;
        pos(self.olr_b_w_m2_k, "olr_b_w_m2_k")?;
        check(
            self.diffusivity_w_m2_k.is_finite() && self.diffusivity_w_m2_k >= 0.0,
            "diffusivity_w_m2_k",
            "must be ≥ 0",
        )?;
        for (v, key) in [
            (self.albedo_a0, "albedo_a0"),
            (self.albedo_ice, "albedo_ice"),
            (self.moisture_floor, "moisture_floor"),
            (self.subsidence, "subsidence"),
            (self.ice_sheet_feedback, "ice_sheet_feedback"),
            (self.oro_lee_floor, "oro_lee_floor"),
            (self.onshore, "onshore"),
        ] {
            check((0.0..=1.0).contains(&v), key, "must be in [0, 1]")?;
        }
        check(
            (0.0..1.0).contains(&(self.albedo_a0 + self.albedo_a2.max(0.0)))
                && self.albedo_a0 - 0.5 * self.albedo_a2.max(0.0) >= 0.0,
            "albedo_a2",
            "a₀ + a₂·P₂ must stay in [0, 1)",
        )?;
        pos(self.ocean_depth_m, "ocean_depth_m")?;
        pos(self.land_depth_m, "land_depth_m")?;
        check(
            self.land_ocean_exchange_w_m2_k >= 0.0,
            "land_ocean_exchange_w_m2_k",
            "must be ≥ 0",
        )?;
        check(
            self.ocean_diffusivity_w_m2_k.is_finite() && self.ocean_diffusivity_w_m2_k >= 0.0,
            "ocean_diffusivity_w_m2_k",
            "must be ≥ 0",
        )?;
        check(
            (6..=720).contains(&self.ebm_bands),
            "ebm_bands",
            "must be in [6, 720]",
        )?;
        check(
            self.ebm_steps_per_year >= 12
                && self.ebm_steps_per_year <= 8760
                && self.ebm_steps_per_year.is_multiple_of(12),
            "ebm_steps_per_year",
            "a multiple of 12 in [12, 8760]",
        )?;
        check(
            (1..=1000).contains(&self.ebm_spinup_years),
            "ebm_spinup_years",
            "must be in [1, 1000]",
        )?;
        check(
            (1..=1000).contains(&self.ebm_warm_years),
            "ebm_warm_years",
            "must be in [1, 1000]",
        )?;
        check(
            (0.0..=20.0).contains(&self.lapse_c_per_km),
            "lapse_c_per_km",
            "must be in [0, 20]",
        )?;
        pos(self.trade_wind_m_s, "trade_wind_m_s")?;
        pos(self.westerly_m_s, "westerly_m_s")?;
        pos(self.polar_wind_m_s, "polar_wind_m_s")?;
        check(
            (0.0..=40.0).contains(&self.itcz_max_shift_deg),
            "itcz_max_shift_deg",
            "must be in [0, 40]",
        )?;
        check(
            (0.0..=20.0).contains(&self.subtropical_shift_deg),
            "subtropical_shift_deg",
            "must be in [0, 20]",
        )?;
        for (v, key) in [
            (self.itcz_smooth_km, "itcz_smooth_km"),
            (self.moisture_decay_km, "moisture_decay_km"),
            (self.ocean_recharge_km, "ocean_recharge_km"),
            (self.maritime_decay_km, "maritime_decay_km"),
            (self.coast_km, "coast_km"),
            (self.air_mixing_km, "air_mixing_km"),
            (self.vapour_scale_m, "vapour_scale_m"),
            (self.barrier_recover_km, "barrier_recover_km"),
            (self.plateau_scale_m, "plateau_scale_m"),
            (self.oro_conv_time_s, "oro_conv_time_s"),
            (self.oro_fall_time_s, "oro_fall_time_s"),
            (self.oro_hw_m, "oro_hw_m"),
        ] {
            pos(v, key)?;
        }
        for (v, key) in [
            (self.recycling, "recycling"),
            (self.sst_subtropical_cold_c, "sst_subtropical_cold_c"),
            (self.sst_subtropical_warm_c, "sst_subtropical_warm_c"),
            (self.sst_midlatitude_warm_c, "sst_midlatitude_warm_c"),
            (self.sst_midlatitude_cold_c, "sst_midlatitude_cold_c"),
            (self.oro_nm, "oro_nm"),
            (self.oro_cw_kg_m3, "oro_cw_kg_m3"),
            (self.oro_hours_per_day, "oro_hours_per_day"),
            (self.precip_min_mm, "precip_min_mm"),
            (self.plateau_m, "plateau_m"),
        ] {
            check(v.is_finite() && v >= 0.0, key, "must be ≥ 0")?;
        }
        check(
            self.oro_hours_per_day <= 24.0,
            "oro_hours_per_day",
            "must be ≤ 24",
        )?;
        check(
            self.ela_offset_c.is_finite(),
            "ela_offset_c",
            "must be finite",
        )?;
        check(
            self.glaciation_off_frac > 0.0
                && self.glaciation_off_frac <= self.glaciation_on_frac
                && self.glaciation_on_frac < 1.0,
            "glaciation_on_frac",
            "need 0 < glaciation_off_frac ≤ glaciation_on_frac < 1",
        )?;
        Ok(())
    }
}

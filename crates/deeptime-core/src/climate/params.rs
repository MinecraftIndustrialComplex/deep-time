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

/// The climate the world is baked and played in (`present_climate`; decisions log 2026-10-01,
/// "prevent ice ages / snowball events for the final state"). It governs the run's last
/// `present_adjust_kyr` only: the history before it, its events and deposits stay as simulated.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentClimate {
    /// No ice age and no snowball at the present: an interglacial like Earth's today (ice sheets
    /// may remain; at most `present_interglacial_ice_frac` of the land under ice sheets, mountain
    /// ice not counted).
    #[default]
    Temperate,
    /// Whatever the simulation gives.
    Natural,
    /// A glacial maximum: at least `present_glacial_ice_frac` of the land under ice sheets, no
    /// snowball.
    IceAge,
    /// A snowball: the ocean frozen over.
    Snowball,
}

impl PresentClimate {
    pub fn name(self) -> &'static str {
        match self {
            PresentClimate::Temperate => "temperate",
            PresentClimate::Natural => "natural",
            PresentClimate::IceAge => "ice_age",
            PresentClimate::Snowball => "snowball",
        }
    }
}

/// Which climate model computes a snapshot (`climate.model`; P1-26).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClimateModel {
    /// Temperature and sea ice per cell of a 2-D coarse grid (`ebm2d.rs`, `ocean2d.rs`).
    #[default]
    PerCell,
    /// The zonal model: one temperature and sea ice per latitude band (kept for comparison).
    Banded,
}

/// Where a snapshot's rain comes from (`climate.rain_model`; P1-26 §3.8).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RainModel {
    /// The belt rain (`belts.rs`, `lapse.rs`), fed by the per-cell climate's ITCZ and sea.
    #[default]
    Belts,
    /// The monthly moisture budget on the coarse grid (`moisture.rs`).
    Budget,
    /// The belts' circulation with an upwind budget of vapour and cloud water for the drying
    /// (`upwind.rs`; Ben, decisions log 2026-10-04, "S&B + upwind moisture budget").
    Upwind,
}

/// The climate section.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ClimateParams {
    /// A full climate snapshot every this many Myr (plan §8.3: 10; the pipeline and earthlike
    /// presets 20 since P1-19 #27, climate.md "The climate snapshot every 20 Myr"), plus the first
    /// and the last plate step; between snapshots the climate's geographic part moves with the
    /// plates. ledger: SCL-038 (game-choice)
    pub snapshot_every_myr: f64,

    // --- The per-cell climate (P1-26; `docs/design/p1-26-per-cell-climate.md`).
    /// `per_cell` (the 2-D energy balance on a coarse grid) or `banded` (the zonal model).
    pub model: ClimateModel,
    /// Columns of the per-cell climate's coarse grid (rows: half); a coarse cell is a block of
    /// L1 cells, so the count is rounded to a divisor of L1's width.
    pub grid_columns: u32,
    /// Scales the surface-dependent atmospheric diffusivity (`ebm2d::D_*`, the sources' total
    /// transport) down to the atmosphere's own share (calibrated against its 5.0 PW at 43°N).
    pub atmosphere_diffusivity_scale: f64,
    /// East–west against north–south heat diffusion in the per-cell climate.
    pub zonal_diffusivity_ratio: f64,
    /// The share of the low-level wind's speed at which the boundary layer's heat moves along a
    /// row (coastal mildness carried inland; 0: none).
    pub wind_heat_transport: f64,
    /// Planetary albedo added over desert (Köppen B land at the previous snapshot).
    pub desert_albedo: f64,
    /// `belts` (the belt rain on the per-cell temperatures; the default) or `budget` (the monthly
    /// moisture budget, `moisture.rs`).
    pub rain_model: RainModel,
    /// The budget's monsoons: pressure per kelvin a cell departs from its row's mean temperature,
    /// Pa/K (hot continents draw sea air in, cold ones push it out).
    pub thermal_wind_pa_k: f64,
    /// The budget's rain: how sharply precipitation grows with the column's relative humidity
    /// (Bretherton, Peters & Back 2004's monthly fit: 11.4).
    pub precip_rh_slope: f64,
    /// The budget's soil water: what the land stores and evaporates again, mm.
    pub bucket_mm: f64,
    /// The upwind rain (`rain_model = upwind`): eddy diffusivity of the vapour and cloud water
    /// across the wind, m²/s (LFPM's transversal dispersion at the climate grid's scale).
    pub rain_diffusivity_m2_s: f64,
    /// The upwind rain: the share of the low-level wind's speed at which the vapour and cloud
    /// water move (the month's mean transport is slower than its mean wind).
    pub rain_wind_share: f64,
    /// The upwind rain: how much of the land's evapotranspiration (Fu's Budyko curve on its rain
    /// and potential evaporation) goes back into the air that carries the rain inland (1: all).
    pub rain_recycling: f64,
    /// The upwind rain: the near-surface relative humidity over the sea under a column holding
    /// the reference water (the bulk evaporation's humidity deficit).
    pub sea_surface_rh: f64,
    /// The upwind rain: how sharply condensation grows with the column's relative humidity (the
    /// e-folding a in e^(a (r − r_ref)); 0: linear, as LFPM). The budget's `precip_rh_slope`
    /// plays this part for `rain_model = budget`.
    pub rain_rh_slope: f64,
    /// How much heat the wind-driven gyres carry (`ocean2d.rs`; 0: none).
    pub gyre_heat_efficiency: f64,
    /// The overturning conveyor that sinks in the narrowest polar-connected basin, Sv (0: off;
    /// Earth's Atlantic overturning is about 17).
    pub overturning_sv: f64,
    /// How much colder snow-covered continental interiors get in the dark season than the air
    /// the energy balance carries, °C (winter surface inversions).
    pub winter_inversion_c: f64,
    /// How crisply sea-ice edges are drawn on L1: 1 soft (the coarse grid's), higher crisper.
    pub sea_ice_edge_sharpness: f64,
    /// Extra cloud greenhouse on frozen surfaces (sea ice, snow, ice sheets), W/m²: the cloudy
    /// models' snowball clouds beyond the ~20 W/m² the linear longwave law already holds (0: none;
    /// Ben 2026-10-02, "None by default, a setting").
    pub snowball_cloud_w_m2: f64,

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
    /// One factor on the belts' rain (`rain_model = belts`): calibrated so that Earth's relief and
    /// CO₂ give Earth's observed land rain (Ben, decisions log 2026-10-06, "Belts, scaled up to
    /// Earth's amount"). Scales the amount only; the pattern is the belts'.
    pub belts_rain_scale: f64,
    /// Wetting under the western side of the subtropical highs (east coasts): the circulation's
    /// rain there is up to 1 + this times the belt's.
    pub west_flank_wet: f64,
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

    // --- Greenhouse and sun over deep time (`carbon.rs`, `clock.rs`; decisions log 2026-09-29).
    /// Run the long-term carbon cycle: volcanic and metamorphic degassing from the plates against
    /// silicate weathering, CO₂ setting the EBM's longwave A (`olr_a_w_m2` is the 280 ppm value).
    /// Off: CO₂ stays at `initial_co2_ppm` (280 when that is 0).
    pub carbon_cycle: bool,
    /// Atmospheric CO₂ at the start of the run, ppm; 0 starts the carbon cycle in balance
    /// (degassing = weathering on the first step's geography and sun).
    pub initial_co2_ppm: f64,
    /// Scales the volcanic and metamorphic CO₂ degassing (1 = Earth's present rate for Earth's
    /// present ridge and subduction activity).
    pub degassing_scale: f64,
    /// Scales silicate weathering (1 = Earth's present relief under its present climate weathers
    /// exactly Earth's present degassing at 280 ppm).
    pub weathering_strength: f64,
    /// Temperature sensitivity of silicate weathering, 1/K (GEOCARB's e^(0.09·ΔT), an activation
    /// energy of ~63 kJ/mol; Berner 1994).
    pub weathering_temp_per_k: f64,
    /// Runoff sensitivity to the global temperature between snapshots, 1/K (GEOCARB's
    /// (1 + 0.038·ΔT)^0.65; Berner 1994).
    pub runoff_temp_per_k: f64,
    /// CO₂ fertilisation of weathering: W ∝ (CO₂/280 ppm)^this (Walker, Hays & Kasting 1981: 0.3).
    pub weathering_co2_exponent: f64,
    /// Limit weathering by the supply of fresh rock: each band's weathering cannot outrun what its
    /// erosion (the surface stage's, at least `weathering_min_denudation_m_myr`) exposes, times
    /// `weathering_rock_mol_kg` of Ca + Mg (West 2012; `carbon::supply_limited`).
    pub weathering_supply: bool,
    /// Ca + Mg of the eroded silicate rock, mol/kg (upper continental crust: CaO 3.6 %, MgO 2.5 %,
    /// Rudnick & Gao 2003 → 1.26 mol/kg).
    pub weathering_rock_mol_kg: f64,
    /// Denudation the supply assumes at least, m/Myr (soil production on flat, slowly eroding
    /// ground; cratons weather 2–10 m/Myr, Dixon & von Blanckenburg 2012).
    pub weathering_min_denudation_m_myr: f64,
    /// The L1 grid's erosion against real denudation: the coarse cells smooth the relief that
    /// erodes, so the surface stage's Earth-like planets carry ~9.5 Gt/yr of eroded rock where
    /// Earth's rivers carried ~20 Gt/yr before people (14 Gt suspended, Syvitski et al. 2005, plus
    /// dissolved and bed load): the supply is the grid's × this (2).
    pub weathering_supply_scale: f64,
    /// Chemical denudation: the land's silicate weathering takes its rock away in solution. The
    /// Ca + Mg the carbon cycle weathers over a step (band by band, as its reservoirs drew it) is
    /// placed on the land cells by each cell's own weathering and taken off the ground by the
    /// surface stage, `dissolved_kg_per_mol` of rock per mol, with the isostatic response of any
    /// other denudation (`carbon::place_dissolved`). Off: the weathered rock stays in place.
    pub chemical_denudation: bool,
    /// Rock that leaves in solution per mol of Ca + Mg the silicate weathering releases, kg/mol:
    /// the cations and the dissolved silica of silicate weathering against its Ca + Mg. 0.26 is
    /// the upper crust's Ca + Mg + Na + K weight fraction (0.1) × the cratons' median dissolved
    /// silicate load over its cations ((SCDR + SiO₂)/SCDR = 3.31, Grenville; 1.72 to 5.24 over
    /// the five cratons of West, Galy & Bickle 2005, Table 2) ÷ the 1.26 mol of Ca + Mg per kg
    /// (`weathering_rock_mol_kg`).
    pub dissolved_kg_per_mol: f64,
    /// Seafloor weathering's share of Earth's present silicate weathering (the rest is the land's;
    /// COPSE reloaded, Lenton et al. 2018: 3 of 15 Tmol/yr; 0: none). It needs no land, so a
    /// land-poor planet keeps a thermostat when its weathering is supply-limited.
    pub seafloor_weathering_share: f64,
    /// A brightening star: L(t) = L₀ / (1 + 0.4·(1 − age/`present_age_gyr`)) (Gough 1981), with the
    /// planet's age from the run's timeline (the run ends at `present_age_gyr`). Off: L₀ always.
    pub solar_brightening: bool,
    /// The star's age at the end of the run, Gyr (the Sun: 4.57).
    pub present_age_gyr: f64,
    /// Global precipitation changes by e^(this·ΔT) with the EBM's global temperature against
    /// `hydro_ref_c` (Held & Soden 2006: ~2 %/K).
    pub hydro_sensitivity_per_k: f64,
    pub hydro_ref_c: f64,
    /// The fine clock: the carbon cycle, the EBM and the ice advance in steps of this many kyr
    /// inside each plate step (200 by default; Ben 2026-09-30: "200 kyrs, but make sure it is
    /// configurable" — an editor setting), halved down to 1 kyr wherever CO₂ would move by more
    /// than half, with at most `ebm_clock_years` EBM years per step (the run stops early once
    /// settled).
    pub carbon_step_kyr: f64,
    pub ebm_clock_years: u32,
    /// A snowball (event `deeptime:snowball`) starts when sea ice covers this share of the ocean
    /// (annual mean) and ends below the second.
    pub snowball_on_frac: f64,
    pub snowball_off_frac: f64,
    /// A hothouse (event `deeptime:hothouse`) is a stretch with the EBM's global mean above this, °C.
    pub hothouse_c: f64,

    // --- Recent ice-age cycles (`orbit.rs`).
    /// The last this many Myr of the run have Milankovitch-style orbital forcing (0: none).
    pub orbital_window_myr: f64,
    /// Scales the obliquity and eccentricity swings (1: Earth-like, 0: none).
    pub orbital_strength: f64,
    /// Fine-clock step inside the window, kyr (numerics).
    pub orbital_step_kyr: f64,

    // --- Ice sheets and sea level (`ice.rs`).
    /// Basal yield stress of Nye's ice-sheet profile, kPa (thicker sheets with more).
    pub ice_yield_stress_kpa: f64,
    /// Thickest ice anywhere, m. 3,000 m: Antarctica's mean is ~2.1 km under domes of ~4 km, and
    /// snowball ice sheets, fed by a dry atmosphere, are thought to hold ~1 km of sea level
    /// (Hoffman et al. 2017, Sci. Adv. 3: e1600983); at 4,000 m this model's snowballs lowered
    /// the sea by ~2 km.
    pub ice_max_thickness_m: f64,
    /// Imbrie & Imbrie's τ_m, kyr (the ice's response inside the orbital window).
    pub ice_response_kyr: f64,
    /// Ice-sheet dynamics (`sheet.rs`): the shallow-ice flow with the elevation feedback, isostasy
    /// and calving on a coarse grid, in the ice-age window every clock step and in deep time for
    /// `ice_relax_kyr` per plate step. Off: Nye's equilibrium profile and Imbrie's lag, exactly
    /// the model before the sheets (the sheet is never stepped and adds no ice, so the flexure's
    /// ice load is the equilibrium glaciers'). **Off by default** (decisions log 2026-10-02,
    /// "Merge with ice sheets off": on the banded climate the sheets overgrow, 47–52 % of the land
    /// under ice; they are turned on once the per-cell climate, P1-26, fixes its cold bias).
    pub ice_dynamics: bool,
    /// The climate at the present, the state the world is baked and played in: `temperate` (no
    /// ice age, no snowball; the default), `natural`, `ice_age` or `snowball`. Over the run's
    /// last `present_adjust_kyr` the atmosphere's CO₂ is moved by the smallest factor that reaches
    /// the state (the ocean taking up or giving back carbon, as in Earth's glacial terminations),
    /// ramped in over `present_ramp_kyr`; nothing changes when the simulation already ends there.
    pub present_climate: PresentClimate,
    pub present_adjust_kyr: f64,
    pub present_ramp_kyr: f64,
    /// `temperate` allows at most this share of the land under ice sheets (Earth today: 0.10,
    /// Antarctica and Greenland); `ice_age` wants at least the second (a glacial maximum; NSIDC
    /// gives about 0.32 of the land under glacial ice at the last one). The share counts the ice sheets only, the continuous ice bodies of 50 000 km²
    /// or more (`climate::ICE_SHEET_MIN_KM2`); the glaciers, ice caps and icefields outside them,
    /// the mountain ice, are left out (decisions log 2026-10-18).
    pub present_interglacial_ice_frac: f64,
    pub present_glacial_ice_frac: f64,
    /// `temperate` raises the CO₂ by at most this factor: what the ocean gives back from a glacial
    /// to an interglacial (the late Quaternary's 172 to 300 ppm, Lüthi et al. 2008, ×1.74). Where
    /// the ice sheets stand on high ground that this does not melt, they stay and the present is
    /// marked not reached; before the cap (2026-10-19) such a planet ended a hothouse, the CO₂
    /// raised ×23 to ×47 (climate.md, "The present-day nudge's cap"). `ice_age` and `snowball`
    /// are not limited (the user forces them). 1 never warms; 10⁴ is the search's own limit.
    pub present_co2_factor_max: f64,
    /// Glen's flow factor A, Pa⁻³ yr⁻¹ (EISMINT: 10⁻¹⁶).
    pub ice_flow_a: f64,
    /// Mass-balance gradients above and below the equilibrium line, (m of ice/yr) per m of height
    /// (Oerlemans 1981: a few mm/yr per m).
    pub ice_accumulation_gradient: f64,
    pub ice_ablation_gradient: f64,
    /// The most accumulation at 1 m/yr of precipitation, m of ice/yr (scaled by the precipitation:
    /// a dry cold climate grows its sheets slowly).
    pub ice_accumulation_max_m_yr: f64,
    /// Isostatic e-folding time of the bed under the ice, kyr (Le Meur & Huybrechts 1996: ~3).
    pub ice_isostasy_kyr: f64,
    /// Calving at an ice margin whose depressed bed lies below the sea (the sea or a proglacial
    /// lake floods it), m of ice/yr (Pollard 1982: 20). With the bed's lagged rebound this is what
    /// lets a big sheet collapse when the line rises again; 0: sheets once grown stay.
    pub ice_calving_m_yr: f64,
    /// Columns of the sheet's coarse grid, about (numerics).
    pub ice_grid_cells: u32,
    /// Deep time: the sheet runs this long at every plate step's end climate from the ice the
    /// plates carried, kyr (numerics).
    pub ice_relax_kyr: f64,
    /// World time at the run's end, years: not a preset key, set by the stage from the timeline
    /// (`PlanetPreset::t_end_years`). The ice-age window and the orbit are measured from it
    /// (docs/design/playability-steering.md §2); 0 for a run to its planned length.
    #[serde(skip)]
    pub run_end_years: f64,
}

impl Default for ClimateParams {
    fn default() -> Self {
        ClimateParams {
            snapshot_every_myr: 10.0,
            model: ClimateModel::PerCell,      // ledger: CLM-175
            grid_columns: 128,                 // ledger: CLM-119
            atmosphere_diffusivity_scale: 1.3, // ledger: CLM-176
            zonal_diffusivity_ratio: 0.2,      // ledger: CLM-121
            wind_heat_transport: 0.2,          // ledger: CLM-125
            desert_albedo: 0.08,               // ledger: CLM-123
            rain_model: RainModel::Belts,      // ledger: CLM-178
            thermal_wind_pa_k: 60.0,           // ledger: CLM-141
            precip_rh_slope: 11.4,             // ledger: CLM-145
            bucket_mm: 150.0,                  // ledger: CLM-143
            rain_diffusivity_m2_s: 5.0e5,      // ledger: CLM-190
            rain_wind_share: 0.3,              // ledger: CLM-191
            rain_recycling: 1.0,               // ledger: CLM-195
            sea_surface_rh: 0.8,               // ledger: CLM-196
            rain_rh_slope: 0.0,                // ledger: CLM-200
            gyre_heat_efficiency: 1.0,         // ledger: CLM-130
            overturning_sv: 30.0,              // ledger: CLM-131
            winter_inversion_c: 15.0,          // ledger: CLM-134
            sea_ice_edge_sharpness: 4.0,       // ledger: CLM-135
            snowball_cloud_w_m2: 0.0,          // ledger: CLM-172
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
            ebm_warm_years: 12,
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
            west_flank_wet: 0.4,    // ledger: MOI-022
            belts_rain_scale: 1.42, // ledger: CLM-201
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
            carbon_cycle: true,
            initial_co2_ppm: 0.0,
            degassing_scale: 1.0,
            weathering_strength: 1.0,
            weathering_temp_per_k: 0.09,
            runoff_temp_per_k: 0.038,
            weathering_co2_exponent: 0.3,
            weathering_supply: true,              // ledger: CLM-152
            weathering_rock_mol_kg: 1.26,         // ledger: CLM-153
            weathering_min_denudation_m_myr: 2.0, // ledger: CLM-155
            weathering_supply_scale: 2.0,         // ledger: CLM-156
            chemical_denudation: true,            // ledger: ERO-205 (on since 2026-10-25)
            dissolved_kg_per_mol: 0.26,           // ledger: ERO-206
            seafloor_weathering_share: 0.2,       // ledger: CLM-158
            solar_brightening: true,
            present_age_gyr: 4.57,
            hydro_sensitivity_per_k: 0.02,
            hydro_ref_c: 15.0,
            carbon_step_kyr: 200.0, // ledger: CLM-082
            ebm_clock_years: 10,
            snowball_on_frac: 0.95,
            snowball_off_frac: 0.5,
            hothouse_c: 25.0,
            orbital_window_myr: 3.0,
            orbital_strength: 1.0,
            orbital_step_kyr: 2.0,
            ice_yield_stress_kpa: 50.0,
            ice_max_thickness_m: 3000.0,
            ice_response_kyr: 17.0,
            ice_dynamics: false,                        // ledger: CLM-147
            present_climate: PresentClimate::Temperate, // ledger: CLM-148
            present_adjust_kyr: 100.0,                  // ledger: CLM-149
            present_ramp_kyr: 10.0,                     // ledger: CLM-149
            present_interglacial_ice_frac: 0.15,        // ledger: CLM-150
            present_glacial_ice_frac: 0.25,             // ledger: CLM-150
            present_co2_factor_max: 1.75,               // ledger: CLM-206
            ice_flow_a: 1.0e-16,                        // ledger: GLA-068
            ice_accumulation_gradient: 0.005,           // ledger: GLA-069
            ice_ablation_gradient: 0.01,                // ledger: GLA-069
            ice_accumulation_max_m_yr: 0.5,             // ledger: GLA-070
            ice_isostasy_kyr: 3.0,                      // ledger: GLA-071
            ice_calving_m_yr: 20.0,                     // ledger: GLA-072
            ice_grid_cells: 256,                        // ledger: GLA-073
            ice_relax_kyr: 30.0,                        // ledger: GLA-074
            run_end_years: 0.0,
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
        check(
            (16..=720).contains(&self.grid_columns),
            "grid_columns",
            "must be in [16, 720]",
        )?;
        for (v, key, lo, hi) in [
            (
                self.atmosphere_diffusivity_scale,
                "atmosphere_diffusivity_scale",
                0.0,
                5.0,
            ),
            (
                self.zonal_diffusivity_ratio,
                "zonal_diffusivity_ratio",
                0.0,
                2.0,
            ),
            (self.wind_heat_transport, "wind_heat_transport", 0.0, 2.0),
            (self.desert_albedo, "desert_albedo", 0.0, 0.3),
            (self.thermal_wind_pa_k, "thermal_wind_pa_k", 0.0, 500.0),
            (self.precip_rh_slope, "precip_rh_slope", 1.0, 40.0),
            (self.bucket_mm, "bucket_mm", 1.0, 2000.0),
            (
                self.rain_diffusivity_m2_s,
                "rain_diffusivity_m2_s",
                0.0,
                1.0e7,
            ),
            (self.rain_recycling, "rain_recycling", 0.0, 1.0),
            (self.rain_wind_share, "rain_wind_share", 0.0, 2.0),
            (self.sea_surface_rh, "sea_surface_rh", 0.3, 0.99),
            (self.rain_rh_slope, "rain_rh_slope", 0.0, 40.0),
            (self.west_flank_wet, "west_flank_wet", 0.0, 5.0),
            (self.belts_rain_scale, "belts_rain_scale", 0.1, 5.0),
            (self.gyre_heat_efficiency, "gyre_heat_efficiency", 0.0, 5.0),
            (self.overturning_sv, "overturning_sv", 0.0, 100.0),
            (self.winter_inversion_c, "winter_inversion_c", 0.0, 40.0),
            (
                self.sea_ice_edge_sharpness,
                "sea_ice_edge_sharpness",
                1.0,
                20.0,
            ),
            (self.snowball_cloud_w_m2, "snowball_cloud_w_m2", 0.0, 20.0),
        ] {
            check(
                v.is_finite() && (lo..=hi).contains(&v),
                key,
                &format!("must be in [{lo}, {hi}]"),
            )?;
        }
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
        check(
            self.initial_co2_ppm == 0.0 || (1.0..=1.0e6).contains(&self.initial_co2_ppm),
            "initial_co2_ppm",
            "0 (in balance) or 1–1,000,000 ppm",
        )?;
        for (v, key, lo, hi) in [
            (self.degassing_scale, "degassing_scale", 0.0, 10.0),
            (self.weathering_strength, "weathering_strength", 0.01, 100.0),
            (
                self.weathering_temp_per_k,
                "weathering_temp_per_k",
                0.0,
                0.3,
            ),
            (self.runoff_temp_per_k, "runoff_temp_per_k", 0.0, 0.2),
            (
                self.weathering_co2_exponent,
                "weathering_co2_exponent",
                0.0,
                1.0,
            ),
            (
                self.weathering_rock_mol_kg,
                "weathering_rock_mol_kg",
                0.01,
                10.0,
            ),
            (
                self.weathering_supply_scale,
                "weathering_supply_scale",
                0.01,
                100.0,
            ),
            (self.dissolved_kg_per_mol, "dissolved_kg_per_mol", 0.0, 1.0),
            (
                self.seafloor_weathering_share,
                "seafloor_weathering_share",
                0.0,
                0.9,
            ),
            (
                self.weathering_min_denudation_m_myr,
                "weathering_min_denudation_m_myr",
                0.0,
                1000.0,
            ),
            (self.present_age_gyr, "present_age_gyr", 0.5, 12.0),
            (
                self.hydro_sensitivity_per_k,
                "hydro_sensitivity_per_k",
                0.0,
                0.1,
            ),
            (self.hydro_ref_c, "hydro_ref_c", -50.0, 50.0),
            (self.carbon_step_kyr, "carbon_step_kyr", 1.0, 2000.0),
            (self.hothouse_c, "hothouse_c", -50.0, 100.0),
            (self.orbital_window_myr, "orbital_window_myr", 0.0, 20.0),
            (self.orbital_strength, "orbital_strength", 0.0, 3.0),
            (self.orbital_step_kyr, "orbital_step_kyr", 0.25, 20.0),
            (self.present_adjust_kyr, "present_adjust_kyr", 1.0, 2000.0),
            (self.present_ramp_kyr, "present_ramp_kyr", 0.0, 2000.0),
            (
                self.present_interglacial_ice_frac,
                "present_interglacial_ice_frac",
                0.0,
                1.0,
            ),
            (
                self.present_glacial_ice_frac,
                "present_glacial_ice_frac",
                0.0,
                1.0,
            ),
            (
                self.present_co2_factor_max,
                "present_co2_factor_max",
                1.0,
                1.0e4,
            ),
            (
                self.ice_yield_stress_kpa,
                "ice_yield_stress_kpa",
                5.0,
                500.0,
            ),
            (
                self.ice_max_thickness_m,
                "ice_max_thickness_m",
                10.0,
                10_000.0,
            ),
            (self.ice_response_kyr, "ice_response_kyr", 0.1, 200.0),
            (self.ice_flow_a, "ice_flow_a", 1.0e-19, 1.0e-13),
            (
                self.ice_accumulation_gradient,
                "ice_accumulation_gradient",
                0.0,
                0.1,
            ),
            (
                self.ice_ablation_gradient,
                "ice_ablation_gradient",
                0.0,
                0.1,
            ),
            (
                self.ice_accumulation_max_m_yr,
                "ice_accumulation_max_m_yr",
                0.0,
                10.0,
            ),
            (self.ice_isostasy_kyr, "ice_isostasy_kyr", 0.1, 100.0),
            (self.ice_calving_m_yr, "ice_calving_m_yr", 0.0, 1000.0),
            (self.ice_relax_kyr, "ice_relax_kyr", 0.0, 1000.0),
        ] {
            check(
                v.is_finite() && (lo..=hi).contains(&v),
                key,
                &format!("must be in [{lo}, {hi}]"),
            )?;
        }
        check(
            (1..=1000).contains(&self.ebm_clock_years),
            "ebm_clock_years",
            "must be in [1, 1000]",
        )?;
        check(
            (8..=4096).contains(&self.ice_grid_cells),
            "ice_grid_cells",
            "must be in [8, 4096]",
        )?;
        check(
            0.0 < self.snowball_off_frac
                && self.snowball_off_frac <= self.snowball_on_frac
                && self.snowball_on_frac <= 1.0,
            "snowball_on_frac",
            "need 0 < snowball_off_frac ≤ snowball_on_frac ≤ 1",
        )?;
        Ok(())
    }
}

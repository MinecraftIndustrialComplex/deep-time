# Preset coverage: what the built-in Earth-like presets set, and which subsystem owns each key

Source files read: `crates/deeptime-core/presets/earthlike_quick_16k.json` (447 lines, read in full), the other three presets (diffed against it), `crates/deeptime-core/src/params.rs` (`PLANET_KEYS`, `SCALE_KEYS`, `PlanetPreset`, `PlanetPreset::validate`) and `crates/deeptime-core/src/preset.rs` (`HEADER_KEYS`, `Timeline`, `StartConfig`, `LENGTH_MYR_RANGE`), and `neoforge/src/main/resources/data/deeptime/worldgen/world_preset/planet.json`. The leaf list below was flattened mechanically from the JSON with `jq` (read-only, output only to this folder); an array of plain numbers is one leaf, an array of objects (`surface.sea_level.cycles`) is listed per element. Values longer than 120 characters are cut. `young` is an empty object `{}` and is listed as such.

The four presets are the "pipeline" presets under their game names (`earthlike_quick_16k` = `pipeline_quick_16k` and so on; only name and description differ; plan §5.1 "As built in P1-08"; `tools/dev/sync_builtin_presets.py` writes them; test `builtins_are_the_pipeline_presets` keeps them equal). Their `description` says "calibrated 2026-10-01, P1-17".

**Changed since this note was taken (plate tasks 3 and 4, 2026-10-02):** `tectonics.collision_distance_km`, `collision_coefficient_per_km`, `collision_uplift_max_m`, `collision_orogen_core` and `collision_on_shortening` are gone; `collision_belt_km` (400), `collision_stacking_share` (0.8), `wedge_slope_deg` (1.5), `collision_resistance_mm_yr` (200), `weld_speed_mm_yr` (5) and `weld_after_myr` (10) are new; `collision_threshold_km` is 100 (was 300) and `subduction_uplift_mm_yr` 0.55 on the quick presets and 0.45 on the full ones (were 0.42 and 0.34). The tables below are the snapshot as taken. Ledger: PLT-241 to PLT-252, PLT-039, PLT-068.

## Top-level sections and leaf counts (earthlike_quick_16k)

| section | leaves | subsystem | what it is |
|---|---|---|---|
| `schema` | 1 | other (header) | preset schema version, 0 |
| `name`, `description` | 2 | other (header) | labels |
| `seed` | 1 | other | random seed, 1 |
| `planet` | 6 | planet | gravity, day length, axial tilt, star flux, ocean fraction, regime |
| `scale` | 11 | scale | circumference, metres per block (horizontal and vertical), min_y, max_y, sea level, pole cap, build-limit band, three feature multipliers |
| `keyframes` | 2 | keyframes | timelapse interval (Myr) and picture size |
| `resolution` | 4 | resolution | L0 sample count; L1, L2, L3 blocks per cell |
| `tectonics` | 76 | plates | plate model constants (Cortial et al. 2019 Appendix A plus the implementation's own) |
| `surface` | 43 | surface | erosion, deposition, carbonate and evaporite growth, regolith, isostasy, sea level curve, glacial |
| `climate` | 78 | climate | energy balance, winds and moisture, orography, carbon cycle and star, ice sheets |
| `structure` | 31 | structure | plutons, strata smoothing, folds and faults (`structure.deformation`) |
| `deposits` | 8 | deposits | ore count, size, richness, province strength, essentials |
| `young` | 0 (empty object) | young | young landscape (L2), all defaults |
| `amplify` | 4 | amplify | incision cap, glacial shape, ruggedness |
| `volcanoes` | 24 | volcanoes | arc and hotspot volcano shapes, calderas, flood basalt, height ceiling |
| `impacts` | 11 | impacts | meteor crater flux and scaling |
| `hydro` | 23 | hydro | rivers, lakes, lake freezing, carver protection |
| `biomes` | 12 | other (bake: climate to biome inputs) | the knobs of the climate-to-biome mapping |
| `underground` | 6 | other (lava floor, lava lakes, magma chambers) | what the game puts in open space below ground |

`timeline` is deliberately absent from every built-in preset (no default story; decisions file "Timeline"); a run needs `timeline.start` (supercontinent, scattered_continents, random) and `timeline.length_myr` (1 to 4600). `PLANET_KEYS` lists the 16 planet-pipeline sections (`seed, planet, keyframes, resolution, tectonics, surface, climate, structure, deposits, young, amplify, biomes, hydro, volcanoes, underground, impacts`) plus the header keys `schema, name, description, placeholder, scale, timeline`; every one except `placeholder` and `timeline` is present in the preset.

Keys the code accepts but the preset leaves at their defaults (not exhaustive: I did not enumerate every serde-default field of every section): `amplify.detail.relief_fraction` 0.12, `max_blocks` 6.0, `channel_area_km2` [25, 250], `octaves` 3, `wavelength_blocks` 32.0 (the detail term the game adds on top of the baked terrain; `surface/detail.rs: DetailParams::default`); `amplify` solver constants (iterations, exponents, thermal slope and so on) and `amplify.character`; `placeholder`; `tectonics` fields with a default in `impl Default for TectonicParams` not listed above. The first ones are what make the 16-block detail headroom (`DETAIL_HEADROOM_BLOCKS`) larger than the default detail maximum (6 blocks, 12 at drama 2).

## Every leaf of earthlike_quick_16k

| key | value | subsystem |
|---|---|---|
| `schema` | `0` | other (header) |
| `name` | `"earthlike_quick_16k"` | other (header) |
| `description` | `"An Earth-like planet, quick (singleplayer size, C = 16384 blocks): plates and crust, climate, deep-time erosion and ...` | other (header) |
| `seed` | `1` | other (random seed) |
| `planet.gravity_m_s2` | `9.81` | planet |
| `planet.day_length_h` | `24.0` | planet |
| `planet.axial_tilt_deg` | `23.4` | planet |
| `planet.star_flux_rel` | `1.0` | planet |
| `planet.ocean_fraction` | `0.7` | planet |
| `planet.regime` | `"mobile_lid"` | planet |
| `scale.circumference_blocks` | `16384` | scale |
| `scale.m_per_block_h` | `300.0` | scale |
| `scale.m_per_block_v` | `20.0` | scale |
| `scale.min_y` | `-352` | scale |
| `scale.max_y` | `639` | scale |
| `scale.sea_level_y` | `63` | scale |
| `scale.pole_cap_deg` | `75.0` | scale |
| `scale.build_limit_band_blocks` | `48.0` | scale |
| `scale.feature_scale.deposits` | `1.0` | scale |
| `scale.feature_scale.strata` | `1.0` | scale |
| `scale.feature_scale.rivers` | `1.0` | scale |
| `keyframes.every_myr` | `50.0` | keyframes |
| `keyframes.size` | `[1024,512]` | keyframes |
| `young` | `{}` | young |
| `amplify.incision_cap` | `3.0` | amplify |
| `amplify.glacial.strength` | `1.0` | amplify |
| `amplify.glacial.ridge_share` | `0.5` | amplify |
| `amplify.ruggedness` | `1.0` | amplify |
| `resolution.l0_samples` | `100000` | resolution |
| `resolution.l1_r` | `16` | resolution |
| `resolution.l2_r` | `8` | resolution |
| `resolution.l3_r` | `2` | resolution |
| `deposits.count_mult` | `1.0` | deposits |
| `deposits.size_mult` | `1.0` | deposits |
| `deposits.richness_mult` | `1.0` | deposits |
| `deposits.province` | `"strong"` | deposits |
| `deposits.province_km` | `800.0` | deposits |
| `deposits.brine_flow_km` | `600.0` | deposits |
| `deposits.types` | `[]` | deposits |
| `deposits.essentials` | `[]` | deposits |
| `tectonics.time_step_myr` | `2.0` | plates |
| `tectonics.model_radius_km` | `6370.0` | plates |
| `tectonics.ridge_elevation_m` | `-2500.0` | plates |
| `tectonics.abyssal_elevation_m` | `-6000.0` | plates |
| `tectonics.trench_elevation_m` | `-10000.0` | plates |
| `tectonics.max_elevation_m` | `30000.0` | plates |
| `tectonics.uplift_reference_m` | `10000.0` | plates |
| `tectonics.uplift_ceiling_m` | `0.0` | plates |
| `tectonics.crust_strength_mpa` | `140.0` | plates |
| `tectonics.crust_flow_km2_myr` | `30000.0` | plates |
| `tectonics.subduction_distance_km` | `800.0` | plates |
| `tectonics.collision_distance_km` | `2000.0` | plates |
| `tectonics.collision_coefficient_per_km` | `0.000013` | plates |
| `tectonics.max_plate_speed_mm_yr` | `100.0` | plates |
| `tectonics.oceanic_damping_mm_yr` | `0.12` | plates |
| `tectonics.oceanic_floor_elevation_m` | `-5600.0` | plates |
| `tectonics.continental_erosion_mm_yr` | `0.0` | plates |
| `tectonics.sediment_accretion_mm_yr` | `0.3` | plates |
| `tectonics.subduction_uplift_mm_yr` | `0.42` | plates |
| `tectonics.collision_threshold_km` | `300.0` | plates |
| `tectonics.subduction_peak_km` | `300.0` | plates |
| `tectonics.collision_uplift_max_m` | `1400.0` | plates |
| `tectonics.collision_orogen_core` | `0.5` | plates |
| `tectonics.slab_pull_rad_per_1000km_myr` | `0.002` | plates |
| `tectonics.slab_pull_speed_mm_yr` | `80.0` | plates |
| `tectonics.slab_pull_front_km` | `3000.0` | plates |
| `tectonics.slab_pull_relax_myr` | `20.0` | plates |
| `tectonics.motion_jitter_per_sqrt_myr` | `0.02` | plates |
| `tectonics.initial_plates` | `14` | plates |
| `tectonics.plate_size_variety_rad` | `0.35` | plates |
| `tectonics.plate_speed_mm_yr` | `[20.0,70.0]` | plates |
| `tectonics.continental_speed_factor` | `0.6` | plates |
| `tectonics.continent_elevation_m` | `[150.0,900.0]` | plates |
| `tectonics.continental_shelf_share` | `0.29` | plates |
| `tectonics.scattered_continents` | `6` | plates |
| `tectonics.coast_warp_rad` | `0.25` | plates |
| `tectonics.rift_lambda0` | `0.012` | plates |
| `tectonics.rift_ocean_weight` | `0.08` | plates |
| `tectonics.rift_fragments` | `[2,4]` | plates |
| `tectonics.rift_speed_mm_yr` | `[20.0,50.0]` | plates |
| `tectonics.rift_warp_rad` | `0.2` | plates |
| `tectonics.min_plate_area_frac` | `0.003` | plates |
| `tectonics.max_plates` | `40` | plates |
| `tectonics.resample_min_steps` | `10` | plates |
| `tectonics.resample_max_steps` | `60` | plates |
| `tectonics.resample_drift_km` | `1500.0` | plates |
| `tectonics.arc_distance_km` | `200.0` | plates |
| `tectonics.arc_width_km` | `60.0` | plates |
| `tectonics.arc_uplift_mm_yr` | `0.6` | plates |
| `tectonics.arc_magma_m_per_myr` | `20.0` | plates |
| `tectonics.arc_continental_m` | `80.0` | plates |
| `tectonics.cordillera_uplift_mm_yr` | `0.3` | plates |
| `tectonics.cordillera_width_km` | `120.0` | plates |
| `tectonics.cordillera_height_m` | `3000.0` | plates |
| `tectonics.trench_relax` | `0.5` | plates |
| `tectonics.pelagic_m_per_myr` | `5.0` | plates |
| `tectonics.hotspots_initial` | `8` | plates |
| `tectonics.hotspot_births_per_100myr` | `3.0` | plates |
| `tectonics.hotspot_lifetime_myr` | `[40.0,150.0]` | plates |
| `tectonics.hotspot_radius_km` | `150.0` | plates |
| `tectonics.hotspot_uplift_mm_yr` | `0.3` | plates |
| `tectonics.lip_probability` | `0.5` | plates |
| `tectonics.lip_radius_km` | `900.0` | plates |
| `tectonics.lip_thickness_m` | `1500.0` | plates |
| `tectonics.lip_rift_boost` | `4.0` | plates |
| `tectonics.lip_rift_boost_myr` | `20.0` | plates |
| `tectonics.terrane_event_min_frac` | `0.005` | plates |
| `tectonics.collision_min_frac` | `0.1` | plates |
| `tectonics.collision_suture_frac` | `0.1` | plates |
| `tectonics.subduction_initiation_age_myr` | `150.0` | plates |
| `tectonics.subduction_initiation_rate` | `0.3` | plates |
| `tectonics.subduction_initiation_clearance_km` | `1000.0` | plates |
| `tectonics.subduction_initiation_speed_mm_yr` | `[30.0,60.0]` | plates |
| `tectonics.contact_timeout_steps` | `12` | plates |
| `tectonics.collision_orogeny_myr` | `30.0` | plates |
| `tectonics.l1_every_steps` | `1` | plates |
| `surface.k_spl` | `3E-9` | surface |
| `surface.m_spl` | `1.0` | surface |
| `surface.n_spl` | `2.0` | surface |
| `surface.g_deposition` | `1.0` | surface |
| `surface.runoff_ref_m_yr` | `0.5` | surface |
| `surface.threshold_slope` | `0.35` | surface |
| `surface.kd_land_m2_yr` | `2.0` | surface |
| `surface.kd_marine_m2_yr` | `100.0` | surface |
| `surface.wave_base_m` | `60.0` | surface |
| `surface.tidal_range_m` | `2.0` | surface |
| `surface.marine_fill_depth_m` | `0.0` | surface |
| `surface.shelf_slope` | `0.0005` | surface |
| `surface.carbonate_max_mm_yr` | `10.0` | surface |
| `surface.carbonate_extinction_per_m` | `0.1` | surface |
| `surface.carbonate_surface_light` | `2000.0` | surface |
| `surface.carbonate_saturating_light` | `400.0` | surface |
| `surface.carbonate_min_temp_c` | `18.0` | surface |
| `surface.carbonate_clastic_m` | `20.0` | surface |
| `surface.evaporite_m_myr` | `100.0` | surface |
| `surface.weathering_max_m_myr` | `50.0` | surface |
| `surface.weathering_decay_depth_m` | `5.0` | surface |
| `surface.isostasy` | `0.8` | surface |
| `surface.isostasy_radius_km` | `50.0` | surface |
| `surface.sea_level.base_m` | `0.0` | surface |
| `surface.sea_level.cycles.[0].amplitude_m` | `80.0` | surface |
| `surface.sea_level.cycles.[0].period_myr` | `250.0` | surface |
| `surface.sea_level.cycles.[0].phase_deg` | `0.0` | surface |
| `surface.sea_level.cycles.[1].amplitude_m` | `25.0` | surface |
| `surface.sea_level.cycles.[1].period_myr` | `36.0` | surface |
| `surface.sea_level.cycles.[1].phase_deg` | `90.0` | surface |
| `surface.sea_level.curve` | `[]` | surface |
| `surface.log_layers` | `32` | surface |
| `surface.drama` | `1.0` | surface |
| `surface.substeps` | `1` | surface |
| `surface.frame_every_myr` | `10.0` | surface |
| `surface.erosion_pulse_m_myr` | `200.0` | surface |
| `surface.basin_m_myr` | `80.0` | surface |
| `surface.event_min_area_km2` | `3000.0` | surface |
| `surface.transgression_frac` | `0.04` | surface |
| `surface.spl_max_iter` | `6` | surface |
| `surface.glacial.strength` | `1.0` | surface |
| `surface.glacial.young_ela_drop_m` | `500.0` | surface |
| `surface.glacial.valley_width` | `3.0` | surface |
| `climate.snapshot_every_myr` | `10.0` | climate |
| `climate.solar_constant_w_m2` | `1365.2` | climate |
| `climate.olr_a_w_m2` | `203.3` | climate |
| `climate.olr_b_w_m2_k` | `2.09` | climate |
| `climate.diffusivity_w_m2_k` | `0.649` | climate |
| `climate.albedo_a0` | `0.33` | climate |
| `climate.albedo_a2` | `0.25` | climate |
| `climate.albedo_ice` | `0.62` | climate |
| `climate.ice_temp_c` | `-10.0` | climate |
| `climate.ocean_depth_m` | `50.0` | climate |
| `climate.land_depth_m` | `3.0` | climate |
| `climate.land_ocean_exchange_w_m2_k` | `10.0` | climate |
| `climate.ocean_diffusivity_w_m2_k` | `0.3` | climate |
| `climate.ice_sheet_feedback` | `1.0` | climate |
| `climate.ebm_bands` | `90` | climate |
| `climate.ebm_steps_per_year` | `72` | climate |
| `climate.ebm_spinup_years` | `40` | climate |
| `climate.ebm_warm_years` | `6` | climate |
| `climate.lapse_c_per_km` | `6.5` | climate |
| `climate.trade_wind_m_s` | `7.0` | climate |
| `climate.westerly_m_s` | `10.0` | climate |
| `climate.polar_wind_m_s` | `5.0` | climate |
| `climate.itcz_max_shift_deg` | `22.0` | climate |
| `climate.itcz_smooth_km` | `1500.0` | climate |
| `climate.subtropical_shift_deg` | `6.0` | climate |
| `climate.moisture_decay_km` | `3500.0` | climate |
| `climate.moisture_floor` | `0.3` | climate |
| `climate.recycling` | `1.5` | climate |
| `climate.ocean_recharge_km` | `600.0` | climate |
| `climate.maritime_decay_km` | `1200.0` | climate |
| `climate.coast_km` | `250.0` | climate |
| `climate.air_mixing_km` | `300.0` | climate |
| `climate.vapour_scale_m` | `2500.0` | climate |
| `climate.barrier_recover_km` | `1500.0` | climate |
| `climate.plateau_m` | `1500.0` | climate |
| `climate.plateau_scale_m` | `2500.0` | climate |
| `climate.sst_subtropical_cold_c` | `5.0` | climate |
| `climate.sst_subtropical_warm_c` | `3.0` | climate |
| `climate.sst_midlatitude_warm_c` | `6.0` | climate |
| `climate.sst_midlatitude_cold_c` | `5.0` | climate |
| `climate.subsidence` | `0.55` | climate |
| `climate.onshore` | `1.0` | climate |
| `climate.oro_conv_time_s` | `1000.0` | climate |
| `climate.oro_fall_time_s` | `1000.0` | climate |
| `climate.oro_nm` | `0.01` | climate |
| `climate.oro_hw_m` | `3400.0` | climate |
| `climate.oro_cw_kg_m3` | `0.01295` | climate |
| `climate.oro_ref_temp_c` | `6.0` | climate |
| `climate.oro_hours_per_day` | `0.5` | climate |
| `climate.oro_lee_floor` | `0.3` | climate |
| `climate.precip_min_mm` | `10.0` | climate |
| `climate.ela_offset_c` | `0.0` | climate |
| `climate.glaciation_on_frac` | `0.02` | climate |
| `climate.glaciation_off_frac` | `0.01` | climate |
| `climate.carbon_cycle` | `true` | climate |
| `climate.initial_co2_ppm` | `0.0` | climate |
| `climate.degassing_scale` | `1.0` | climate |
| `climate.weathering_strength` | `1.0` | climate |
| `climate.weathering_temp_per_k` | `0.09` | climate |
| `climate.runoff_temp_per_k` | `0.038` | climate |
| `climate.weathering_co2_exponent` | `0.3` | climate |
| `climate.solar_brightening` | `true` | climate |
| `climate.present_age_gyr` | `4.57` | climate |
| `climate.hydro_sensitivity_per_k` | `0.02` | climate |
| `climate.hydro_ref_c` | `15.0` | climate |
| `climate.carbon_step_kyr` | `100.0` | climate |
| `climate.ebm_clock_years` | `10` | climate |
| `climate.snowball_on_frac` | `0.95` | climate |
| `climate.snowball_off_frac` | `0.5` | climate |
| `climate.hothouse_c` | `25.0` | climate |
| `climate.orbital_window_myr` | `3.0` | climate |
| `climate.orbital_strength` | `1.0` | climate |
| `climate.orbital_step_kyr` | `2.0` | climate |
| `climate.ice_yield_stress_kpa` | `50.0` | climate |
| `climate.ice_max_thickness_m` | `3000.0` | climate |
| `climate.ice_response_kyr` | `17.0` | climate |
| `climate.glacio_eustasy` | `1.0` | climate |
| `climate.ice_reference_km3` | `10000000.0` | climate |
| `structure.pluton_rate_km2_myr` | `0.00001` | structure |
| `structure.pluton_length_km` | `[25.0,8.0,80.0]` | structure |
| `structure.strata_smoothing_cells` | `10.0` | structure |
| `structure.strata_tongue_scale` | `1.0` | structure |
| `structure.strata_minor_share` | `0.15` | structure |
| `structure.deformation.strength` | `1.0` | structure |
| `structure.deformation.density` | `1.0` | structure |
| `structure.deformation.size` | `1.0` | structure |
| `structure.deformation.fold_wavelength_km` | `[16.0,8.0,32.0]` | structure |
| `structure.deformation.fold_shortening` | `[0.08,0.3]` | structure |
| `structure.deformation.fold_rate_km2_myr` | `0.000006` | structure |
| `structure.deformation.fault_length_km` | `[40.0,15.0,150.0]` | structure |
| `structure.deformation.fault_reach_ratio` | `0.4` | structure |
| `structure.deformation.normal_slip_ratio` | `0.05` | structure |
| `structure.deformation.thrust_slip_ratio` | `0.08` | structure |
| `structure.deformation.strike_slip_ratio` | `0.05` | structure |
| `structure.deformation.thrust_rate_km2_myr` | `0.00002` | structure |
| `structure.deformation.normal_rate_km2_myr` | `0.00004` | structure |
| `structure.deformation.strike_slip_rate_km2_myr` | `0.000004` | structure |
| `structure.deformation.foreland_km` | `250.0` | structure |
| `structure.deformation.max_overlap` | `3` | structure |
| `structure.deformation.relief` | `1.0` | structure |
| `structure.deformation.relief_age_myr` | `10.0` | structure |
| `structure.deformation.relief_wear` | `5.0` | structure |
| `structure.deformation.fold_relief` | `0.5` | structure |
| `structure.deformation.footwall_share` | `0.2` | structure |
| `structure.deformation.intrusions` | `1.0` | structure |
| `structure.deformation.stock_rate_km2_myr` | `0.00002` | structure |
| `structure.deformation.stock_radius_km` | `[4.0,2.0,10.0]` | structure |
| `structure.deformation.dyke_rate_km2_myr` | `0.00008` | structure |
| `structure.deformation.dyke_length_km` | `[15.0,5.0,50.0]` | structure |
| `volcanoes.density` | `1.0` | volcanoes |
| `volcanoes.size` | `1.0` | volcanoes |
| `volcanoes.flank_steepness` | `0.5` | volcanoes |
| `volcanoes.arc_spacing_km` | `60.0` | volcanoes |
| `volcanoes.arc_min_flux` | `0.25` | volcanoes |
| `volcanoes.arc_height_km` | `[0.6,2.0]` | volcanoes |
| `volcanoes.arc_radius_km` | `10.0` | volcanoes |
| `volcanoes.arc_lifetime_myr` | `[2.0,8.0]` | volcanoes |
| `volcanoes.hotspot_spacing_km` | `70.0` | volcanoes |
| `volcanoes.shield_height_km` | `[3.5,6.5]` | volcanoes |
| `volcanoes.shield_radius_km` | `40.0` | volcanoes |
| `volcanoes.silicic_height_km` | `0.9` | volcanoes |
| `volcanoes.silicic_radius_km` | `35.0` | volcanoes |
| `volcanoes.island_subsidence_m_myr` | `0.0` | volcanoes |
| `volcanoes.island_subsidence_max_m` | `1500.0` | volcanoes |
| `volcanoes.caldera_rate_per_myr` | `0.04` | volcanoes |
| `volcanoes.hotspot_caldera_rate_per_myr` | `1.0` | volcanoes |
| `volcanoes.caldera_radius_km` | `[3.0,9.0]` | volcanoes |
| `volcanoes.silicic_crust_km` | `42.0` | volcanoes |
| `volcanoes.flood_basalt_flows` | `8` | volcanoes |
| `volcanoes.flood_basalt_step_m` | `80.0` | volcanoes |
| `volcanoes.build_myr` | `1.0` | volcanoes |
| `volcanoes.fade_myr` | `5.0` | volcanoes |
| `volcanoes.max_elevation_km` | `6.8` | volcanoes |
| `biomes.temperature_shift` | `0.0` | other (bake: climate to biome inputs) |
| `biomes.humidity_shift` | `0.0` | other (bake: climate to biome inputs) |
| `biomes.beach_blocks` | `16.0` | other (bake: climate to biome inputs) |
| `biomes.terrain_window_blocks` | `64.0` | other (bake: climate to biome inputs) |
| `biomes.variant_blocks` | `1024.0` | other (bake: climate to biome inputs) |
| `biomes.wetlands` | `1.0` | other (bake: climate to biome inputs) |
| `biomes.inland_mid_blocks` | `200.0` | other (bake: climate to biome inputs) |
| `biomes.inland_far_blocks` | `1000.0` | other (bake: climate to biome inputs) |
| `biomes.inland_interior_blocks` | `5000.0` | other (bake: climate to biome inputs) |
| `biomes.wetland_humidity` | `2.5` | other (bake: climate to biome inputs) |
| `biomes.wetland_flat_share` | `0.35` | other (bake: climate to biome inputs) |
| `biomes.wetland_low_m` | `150.0` | other (bake: climate to biome inputs) |
| `hydro.enabled` | `true` | hydro |
| `hydro.river_min_discharge_m3s` | `2.0` | hydro |
| `hydro.river_min_area_km2` | `50.0` | hydro |
| `hydro.width_coef` | `1.0` | hydro |
| `hydro.depth_coef` | `0.5` | hydro |
| `hydro.max_width_blocks` | `64.0` | hydro |
| `hydro.max_depth_blocks` | `12.0` | hydro |
| `hydro.freeboard_blocks` | `1.0` | hydro |
| `hydro.smoothing_blocks` | `8.0` | hydro |
| `hydro.lake_min_area_km2` | `1.0` | hydro |
| `hydro.lake_min_depth_blocks` | `1.0` | hydro |
| `hydro.lake_evaporation` | `true` | hydro |
| `hydro.pet_mm_per_c` | `60.0` | hydro |
| `hydro.lake_balance` | `true` | hydro |
| `hydro.lake_evaporation_base_mm` | `200.0` | hydro |
| `hydro.lake_evaporation_factor` | `1.0` | hydro |
| `hydro.snow_melt_mm_per_degree_day` | `4.0` | hydro |
| `hydro.lake_freeze_mean_c` | `0.0` | hydro |
| `hydro.playa_blocks` | `2.0` | hydro |
| `hydro.biome_min_width_blocks` | `6.0` | hydro |
| `hydro.bank_zone_blocks` | `1.0` | hydro |
| `hydro.bank_zone_ratio` | `0.5` | hydro |
| `hydro.protect_water_from_carvers` | `true` | hydro |
| `underground.lava_floor_blocks` | `10` | other (underground: lava floor, lava lakes, magma chambers) |
| `underground.lava_lake_blocks` | `54` | other (underground: lava floor, lava lakes, magma chambers) |
| `underground.magma_chamber_depth_km` | `3.0` | other (underground: lava floor, lava lakes, magma chambers) |
| `underground.magma_chamber_size` | `1.0` | other (underground: lava floor, lava lakes, magma chambers) |
| `underground.crater_lake_share` | `0.25` | other (underground: lava floor, lava lakes, magma chambers) |
| `underground.remove_lava_features` | `true` | other (underground: lava floor, lava lakes, magma chambers) |
| `impacts.flux` | `1.0` | impacts |
| `impacts.size_slope` | `1.8` | impacts |
| `impacts.min_crater_km` | `2.0` | impacts |
| `impacts.max_crater_km` | `300.0` | impacts |
| `impacts.speed_km_s` | `20.0` | impacts |
| `impacts.impactor_density_kg_m3` | `3000.0` | impacts |
| `impacts.depth_scale` | `0.5` | impacts |
| `impacts.ejecta_radii` | `3.0` | impacts |
| `impacts.sea_cutoff_ratio` | `6.3` | impacts |
| `impacts.diffusivity_m2_yr` | `0.01` | impacts |
| `impacts.degradation` | `1.0` | impacts |

## Differences of the other three presets from earthlike_quick_16k

Found by `diff` of the JSON files (every other key, including all of `tectonics`, `surface`, `climate`, `structure`, `deposits`, `amplify`, `volcanoes`, `impacts`, `hydro`, `biomes`, `underground`, `planet`, `scale` except the circumference, `keyframes`, is identical in all four).

| key | quick_16k | full_16k | quick_32k | full_32k | subsystem |
|---|---|---|---|---|---|
| `name` | earthlike_quick_16k | earthlike_full_16k | earthlike_quick_32k | earthlike_full_32k | other (header) |
| `description` | quick, C = 16384, L0 100000, L1 1024x512, L2 2048x1024, L3 8192x4096 | full, C = 16384, L0 250000, L1 2048x1024, L2 4096x2048, L3 8192x4096 | quick, server size, C = 32768, same L0 and L1..L3 cell counts as quick_16k | full, server size, C = 32768, L0 250000, L1 2048x1024, L2 4096x2048, L3 16384x8192 | other (header) |
| `scale.circumference_blocks` | 16384 | 16384 | 32768 | 32768 | scale |
| `resolution.l0_samples` | 100000 | 250000 | 100000 | 250000 | resolution |
| `resolution.l1_r` | 16 | 8 | 32 | 16 | resolution |
| `resolution.l2_r` | 8 | 4 | 16 | 8 | resolution |
| `resolution.l3_r` | 2 | 2 | 4 | 2 | resolution |
| `tectonics.subduction_uplift_mm_yr` | 0.42 | 0.34 | 0.42 | 0.34 | plates |
| `tectonics.collision_uplift_max_m` | 1400.0 | 1100.0 | 1400.0 | 1100.0 | plates |

Observations: the two "full" presets carry lower plate-uplift rates (0.34 mm/yr and 1,100 m against 0.42 and 1,400) because they were calibrated separately (the descriptions say "calibrated 2026-10-01, P1-17"); the repo text read here does not say why finer grids need less uplift. The 32k presets keep the plate model unchanged (Earth-normalized, so the planet's size does not enter) and change only the circumference and the cell sizes in blocks. quick_32k keeps 100,000 L0 samples on a planet four times the area, so its L0 spacing on the map is 17.5 km (plan §5.2) against 8.8 km at 16k, and its L1 cell is 9.6 km against 4.8 km.

## planet.json: keys that carry physical meaning

File: `neoforge/src/main/resources/data/deeptime/worldgen/world_preset/planet.json` (`planet_32k.json` differs only in `circumference_blocks` 32768). It is a Minecraft world preset, not a Deep Time preset: it names the generator and a pointer to the world data. Physically meaningful keys:

| key path | value | meaning |
|---|---|---|
| `dimensions.minecraft:overworld.generator.world_data.circumference_blocks` | 16384 (32768 in planet_32k.json) | the planet's circumference in blocks (the east-west wrap period); duplicates preset `scale.circumference_blocks` |
| `dimensions.minecraft:overworld.generator.world_data.pole_cap_deg` | 75.0 | latitude where the pole storm and pushback start (Java only; the Rust `scale.pole_cap_deg` is not read by it) |
| `dimensions.minecraft:overworld.generator.world_data.pole_stop_deg` | 80.0 | latitude of the hard stop (no Rust equivalent) |
| `dimensions.minecraft:overworld.generator.settings` | `deeptime:planet` | points to `worldgen/noise_settings/planet.json`, which carries `sea_level` 63, `noise.min_y` -352, `noise.height` 992 |
| `dimensions.minecraft:overworld.type` | `deeptime:planet_overworld` | points to `dimension_type/planet_overworld.json`: `min_y` -352, `height` 992, `logical_height` 992 |
| `dimensions.minecraft:overworld.generator.biome_source` | `minecraft:multi_noise`, preset `minecraft:overworld` | vanilla's biome parameter space; the climate itself comes from the baked rasters (`deeptime:raster`), so this key carries no physics of its own |

No other key of the file is physical: `folder: "deeptime"` names the world-data folder; the End and Nether entries are vanilla (`minecraft:the_end`, `minecraft:the_nether`, with the Nether looped at 1:8 by `wrap/PlanetShape.java`, not by this file). The m per block, metres, gravity and so on are not in this file; they live in the baked world data's manifest.

// SPDX-License-Identifier: LGPL-3.0-or-later
//! Planet presets (plan §5.1): the planet, its scales, the timeline the player chose, the grid
//! ladder and the tectonic constants, as JSON.
//!
//! ```json
//! {"schema": 1, "name": "earthlike_quick_16k", "seed": 1,
//!  "planet": {"gravity_m_s2": 9.81, "day_length_h": 24, "axial_tilt_deg": 23.4,
//!             "star_flux_rel": 1.0, "ocean_fraction": 0.7, "regime": "mobile_lid"},
//!  "scale": {"circumference_blocks": 16384},
//!  "timeline": {"start": "scattered", "length_myr": 500, "keyframe_every_myr": 50},
//!  "resolution": {"l0_samples": 100000, "l1_r": 16},
//!  "tectonics": {…every key optional, Cortial et al. 2019 Appendix A by default…}}
//! ```
//!
//! **The timeline has no default story** (decisions file, "Timeline"): the built-in presets leave
//! `timeline.start` and `timeline.length_myr` empty and a run refuses to start until they are set
//! (the editor, the server config or the CLI's `--start`/`--timeline-myr` fill them in).
//!
//! Sections are parsed with serde, one at a time, so an error names the section
//! (`$.planet: missing field gravity_m_s2`); range checks name the exact key. Every section
//! rejects unknown keys. `surface`, `climate`, `deposits`, … arrive with their stages.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::error::CoreError;

/// The preset schema version (independent of the world-data format).
pub const PRESET_SCHEMA: u32 = 1;

/// The built-in presets (plan §5.1), shipped as JSON in the core.
pub const BUILTIN: [(&str, &str); 4] = [
    (
        "earthlike_quick_16k",
        include_str!("../presets/earthlike_quick_16k.json"),
    ),
    (
        "earthlike_full_16k",
        include_str!("../presets/earthlike_full_16k.json"),
    ),
    (
        "earthlike_quick_32k",
        include_str!("../presets/earthlike_quick_32k.json"),
    ),
    (
        "earthlike_full_32k",
        include_str!("../presets/earthlike_full_32k.json"),
    ),
];

/// The JSON of built-in preset `name`.
pub fn builtin(name: &str) -> Option<&'static str> {
    BUILTIN.iter().find(|(n, _)| *n == name).map(|(_, j)| *j)
}

/// The tectonic regime (plan §5.1). v1 simulates only a mobile lid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TectonicRegime {
    MobileLid,
}

/// The planet itself (plan §5.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanetParams {
    pub gravity_m_s2: f64,
    pub day_length_h: f64,
    pub axial_tilt_deg: f64,
    /// 1.0 = Earth's insolation.
    pub star_flux_rel: f64,
    /// Target ocean area fraction for the initial crust.
    pub ocean_fraction: f64,
    pub regime: TectonicRegime,
}

fn d_m_per_block_h() -> f64 {
    300.0
}
fn d_m_per_block_v() -> f64 {
    20.0
}
fn d_min_y() -> i32 {
    -352
}
fn d_max_y() -> i32 {
    639
}
fn d_sea_level_y() -> i32 {
    63
}
fn d_pole_cap_deg() -> f64 {
    75.0
}

/// Block-space scales (plan §5.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleParams {
    /// 16,384 (singleplayer) or 32,768 (server).
    pub circumference_blocks: u32,
    #[serde(default = "d_m_per_block_h")]
    pub m_per_block_h: f64,
    #[serde(default = "d_m_per_block_v")]
    pub m_per_block_v: f64,
    #[serde(default = "d_min_y")]
    pub min_y: i32,
    #[serde(default = "d_max_y")]
    pub max_y: i32,
    #[serde(default = "d_sea_level_y")]
    pub sea_level_y: i32,
    #[serde(default = "d_pole_cap_deg")]
    pub pole_cap_deg: f64,
}

impl ScaleParams {
    /// radius_m = C · m_per_block_h / 2π (plan §5.1).
    pub fn radius_m(&self) -> f64 {
        f64::from(self.circumference_blocks) * self.m_per_block_h / std::f64::consts::TAU
    }
}

/// The start configuration the player chooses (plan §5.1; decisions file, "Timeline").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartConfig {
    /// One large continent (a Pangaea) on one or two plates.
    Supercontinent,
    /// Several continents of mixed sizes.
    Scattered,
    /// Continents from a thresholded fractal noise.
    Random,
}

impl StartConfig {
    pub fn name(self) -> &'static str {
        match self {
            StartConfig::Supercontinent => "supercontinent",
            StartConfig::Scattered => "scattered",
            StartConfig::Random => "random",
        }
    }

    pub fn parse(s: &str) -> Option<StartConfig> {
        match s {
            "supercontinent" => Some(StartConfig::Supercontinent),
            "scattered" => Some(StartConfig::Scattered),
            "random" => Some(StartConfig::Random),
            _ => None,
        }
    }
}

fn d_keyframe_every_myr() -> f64 {
    50.0
}
fn d_keyframe_size() -> [u32; 2] {
    [1024, 512]
}

/// The timeline (plan §5.1): chosen by the player, no default story.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimelineParams {
    /// Required before a run: supercontinent, scattered or random.
    #[serde(default)]
    pub start: Option<StartConfig>,
    /// Required before a run: how many million years of history end at world creation.
    #[serde(default)]
    pub length_myr: Option<f64>,
    #[serde(default = "d_keyframe_every_myr")]
    pub keyframe_every_myr: f64,
    /// Keyframe width and height in pixels.
    #[serde(default = "d_keyframe_size")]
    pub keyframe_size: [u32; 2],
}

fn d_l2_r() -> u32 {
    8
}
fn d_l3_r() -> u32 {
    2
}

/// The grid ladder (plan §5.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionParams {
    /// L0: Fibonacci samples for plates and crust.
    pub l0_samples: u32,
    /// L1 blocks per cell.
    pub l1_r: u32,
    #[serde(default = "d_l2_r")]
    pub l2_r: u32,
    #[serde(default = "d_l3_r")]
    pub l3_r: u32,
}

/// Tectonic constants (plan §8.1). Cortial, Peytavie, Galin & Guérin 2019 ("Procedural Tectonic
/// Planets", Appendix A, Table 3) give the first group; everything after `collision_threshold_km`
/// is a design choice of this implementation (documented in `plates/mod.rs` and STATUS.md). The
/// model runs on a sphere of `model_radius_km` (6,370 km: Earth-normalized, decisions file
/// "Tectonics scaling") and maps onto the planet by latitude and longitude.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct TectonicParams {
    /// δt (Cortial: 2 My).
    pub time_step_myr: f64,
    /// R of the plate model (Cortial: 6,370 km). Earth-normalized by default; set it to the
    /// planet's own radius for true-scale tectonics.
    pub model_radius_km: f64,
    /// z_r, highest oceanic ridge (−1 km).
    pub ridge_elevation_m: f64,
    /// z_a, abyssal plains (−6 km).
    pub abyssal_elevation_m: f64,
    /// z_t, oceanic trench (−10 km).
    pub trench_elevation_m: f64,
    /// z_c, highest continental altitude (10 km).
    pub max_elevation_m: f64,
    /// r_s (1,800 km).
    pub subduction_distance_km: f64,
    /// r_c (4,200 km).
    pub collision_distance_km: f64,
    /// Δc (1.3e-5 km⁻¹).
    pub collision_coefficient_per_km: f64,
    /// v_0 (100 mm/yr).
    pub max_plate_speed_mm_yr: f64,
    /// ε_o (0.04 mm/yr).
    pub oceanic_damping_mm_yr: f64,
    /// ε_c (0.03 mm/yr).
    pub continental_erosion_mm_yr: f64,
    /// ε_t (0.3 mm/yr).
    pub sediment_accretion_mm_yr: f64,
    /// u_0 (0.6 mm/yr).
    pub subduction_uplift_mm_yr: f64,
    /// Interpenetration that triggers a collision (Cortial §6: 300 km).
    pub collision_threshold_km: f64,

    /// Where the subduction distance transfer f(d) peaks (design choice).
    pub subduction_peak_km: f64,
    /// Cap on Δc·A, the discrete collision uplift (design choice; Δc·A of a whole continent
    /// would be tens of kilometres).
    pub collision_uplift_max_m: f64,
    /// Axis change of a subducting plate towards its fronts per 1,000 km of front and Myr
    /// (slab pull, Cortial §4.1's ε scaled by front length).
    pub slab_pull_rad_per_1000km_myr: f64,
    /// Slab pull on speed: a plate with at least `slab_pull_front_km` of subduction front relaxes
    /// towards this speed at its centroid (less for shorter fronts; Forsyth & Uyeda 1975: plates
    /// attached to slabs move several times faster), mm/yr …
    pub slab_pull_speed_mm_yr: f64,
    /// … the front length that gives the full speed, km …
    pub slab_pull_front_km: f64,
    /// … with this relaxation time; plates without a front relax to the middle of
    /// `plate_speed_mm_yr`, Myr.
    pub slab_pull_relax_myr: f64,
    /// Random walk of the Euler poles, relative change per √Myr (mantle reorganisation).
    pub motion_jitter_per_sqrt_myr: f64,
    /// Plates at the start.
    pub initial_plates: u32,
    /// Spread of initial plate sizes (additive Voronoi weight, radians).
    pub plate_size_variety_rad: f64,
    /// Initial plate speeds at their centroids, mm/yr.
    pub plate_speed_mm_yr: [f64; 2],
    /// Speed factor of an all-continental plate (continents move slower).
    pub continental_speed_factor: f64,
    /// Initial continental elevation range, metres.
    pub continent_elevation_m: [f64; 2],
    /// Continents of the `scattered` start.
    pub scattered_continents: u32,
    /// Coastline irregularity (noise warp of the distance, radians).
    pub coast_warp_rad: f64,
    /// Rifting: λ_0, expected rifts per step of a plate with the average initial area and
    /// f(x_P) = 1 (Cortial §4.4).
    pub rift_lambda0: f64,
    /// f(x_P) = w + (1 − w)·x_P², x_P the plate's continental fraction: oceanic plates rift at w.
    pub rift_ocean_weight: f64,
    /// Fragments of one rift, inclusive (Cortial: 2–4).
    pub rift_fragments: [u32; 2],
    /// Divergence speed given to rift fragments, mm/yr.
    pub rift_speed_mm_yr: [f64; 2],
    /// Fracture-line irregularity (radians of noise warp).
    pub rift_warp_rad: f64,
    /// Plates smaller than this fraction of the planet are absorbed by a neighbour.
    pub min_plate_area_frac: f64,
    /// No rifting beyond this many plates.
    pub max_plates: u32,
    /// Global resampling: at least this many steps apart (Cortial: 10–60).
    pub resample_min_steps: u32,
    /// … at most this many steps apart.
    pub resample_max_steps: u32,
    /// … and when the fastest plate has moved this far since the last one.
    pub resample_drift_km: f64,
    /// Magmatic arc: distance from the front of its axis, km.
    pub arc_distance_km: f64,
    /// … and its half-width, km.
    pub arc_width_km: f64,
    /// Arc volcanic uplift of submarine overriding crust (island arcs), mm/yr at v = v_0.
    pub arc_uplift_mm_yr: f64,
    /// New arc crust per Myr at the arc axis at v = v_0, metres.
    pub arc_magma_m_per_myr: f64,
    /// Fraction of the way to z_t a trench sample moves per step.
    pub trench_relax: f64,
    /// Pelagic sedimentation on oceanic crust, m/Myr.
    pub pelagic_m_per_myr: f64,
    /// Hotspots at the start.
    pub hotspots_initial: u32,
    /// New hotspots per 100 Myr (Poisson).
    pub hotspot_births_per_100myr: f64,
    /// Hotspot lifetime range, Myr.
    pub hotspot_lifetime_myr: [f64; 2],
    /// Radius of a hotspot's volcanic swell, km.
    pub hotspot_radius_km: f64,
    /// Volcanic uplift at a hotspot's centre, mm/yr.
    pub hotspot_uplift_mm_yr: f64,
    /// Probability that a new hotspot arrives with a plume head (a large igneous province).
    pub lip_probability: f64,
    /// Radius of a LIP, km.
    pub lip_radius_km: f64,
    /// Lava pile of a LIP at its centre, metres of rock (its surface rises by a third).
    pub lip_thickness_m: f64,
    /// Rifting rate multiplier of the plate under a LIP …
    pub lip_rift_boost: f64,
    /// … for this long, Myr.
    pub lip_rift_boost_myr: f64,
    /// Terranes smaller than this fraction of A_0 accrete without an event.
    pub terrane_event_min_frac: f64,
    /// Terranes of at least this fraction of A_0 make a collision (and a collision orogeny);
    /// smaller ones are a terrane accretion only.
    pub collision_min_frac: f64,
    /// Oceanic crust older than this, on a plate that subducts nowhere, can start subducting on
    /// its own (Wilson cycle), Myr.
    pub subduction_initiation_age_myr: f64,
    /// Expected subduction initiations per step per A_0 of such old crust.
    pub subduction_initiation_rate: f64,
    /// Convergence given to the plate a subduction initiation splits off, mm/yr.
    pub subduction_initiation_speed_mm_yr: [f64; 2],
    /// Contacts unseen for this many steps are closed.
    pub contact_timeout_steps: u32,
    /// Duration recorded for a collision orogeny, Myr.
    pub collision_orogeny_myr: f64,
    /// Sample L1 every this many steps (0 = only for keyframes and at the end).
    pub l1_every_steps: u32,
}

impl Default for TectonicParams {
    fn default() -> Self {
        TectonicParams {
            time_step_myr: 2.0,
            model_radius_km: 6370.0,
            ridge_elevation_m: -1000.0,
            abyssal_elevation_m: -6000.0,
            trench_elevation_m: -10000.0,
            max_elevation_m: 10000.0,
            subduction_distance_km: 1800.0,
            collision_distance_km: 4200.0,
            collision_coefficient_per_km: 1.3e-5,
            max_plate_speed_mm_yr: 100.0,
            oceanic_damping_mm_yr: 0.04,
            continental_erosion_mm_yr: 0.03,
            sediment_accretion_mm_yr: 0.3,
            subduction_uplift_mm_yr: 0.6,
            collision_threshold_km: 300.0,
            subduction_peak_km: 300.0,
            collision_uplift_max_m: 4000.0,
            slab_pull_rad_per_1000km_myr: 0.002,
            slab_pull_speed_mm_yr: 80.0,
            slab_pull_front_km: 3000.0,
            slab_pull_relax_myr: 20.0,
            motion_jitter_per_sqrt_myr: 0.02,
            initial_plates: 14,
            plate_size_variety_rad: 0.35,
            plate_speed_mm_yr: [20.0, 70.0],
            continental_speed_factor: 0.6,
            continent_elevation_m: [150.0, 900.0],
            scattered_continents: 6,
            coast_warp_rad: 0.25,
            rift_lambda0: 0.012,
            rift_ocean_weight: 0.08,
            rift_fragments: [2, 4],
            rift_speed_mm_yr: [20.0, 50.0],
            rift_warp_rad: 0.2,
            min_plate_area_frac: 0.003,
            max_plates: 40,
            resample_min_steps: 10,
            resample_max_steps: 60,
            resample_drift_km: 1500.0,
            arc_distance_km: 200.0,
            arc_width_km: 60.0,
            arc_uplift_mm_yr: 0.1,
            arc_magma_m_per_myr: 20.0,
            trench_relax: 0.5,
            pelagic_m_per_myr: 5.0,
            hotspots_initial: 8,
            hotspot_births_per_100myr: 3.0,
            hotspot_lifetime_myr: [40.0, 150.0],
            hotspot_radius_km: 150.0,
            hotspot_uplift_mm_yr: 0.3,
            lip_probability: 0.5,
            lip_radius_km: 900.0,
            lip_thickness_m: 1500.0,
            lip_rift_boost: 4.0,
            lip_rift_boost_myr: 20.0,
            terrane_event_min_frac: 0.005,
            collision_min_frac: 0.1,
            subduction_initiation_age_myr: 150.0,
            subduction_initiation_rate: 0.1,
            subduction_initiation_speed_mm_yr: [30.0, 60.0],
            contact_timeout_steps: 12,
            collision_orogeny_myr: 30.0,
            l1_every_steps: 0,
        }
    }
}

/// A planet preset (plan §5.1): everything a run needs.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanetPreset {
    pub schema: u32,
    pub name: String,
    pub seed: u64,
    pub planet: PlanetParams,
    pub scale: ScaleParams,
    pub timeline: TimelineParams,
    pub resolution: ResolutionParams,
    pub tectonics: TectonicParams,
}

type Obj = Map<String, Value>;

const SECTIONS: [&str; 8] = [
    "schema",
    "name",
    "seed",
    "planet",
    "scale",
    "timeline",
    "resolution",
    "tectonics",
];

fn section<T: serde::de::DeserializeOwned>(obj: &Obj, key: &str) -> Result<Option<T>, CoreError> {
    match obj.get(key) {
        None => Ok(None),
        Some(v) => serde_json::from_value(v.clone())
            .map(Some)
            .map_err(|e| CoreError::preset(format!("$.{key}"), e.to_string())),
    }
}

fn check(ok: bool, path: &str, reason: &str) -> Result<(), CoreError> {
    if ok {
        Ok(())
    } else {
        Err(CoreError::preset(path, reason))
    }
}

impl PlanetPreset {
    /// Parses a planet preset from the top-level object. Sections present are parsed in
    /// document order first, so the first malformed section is reported before a missing one.
    pub fn from_obj(obj: &Obj) -> Result<PlanetPreset, CoreError> {
        if let Some(k) = obj.keys().find(|k| !SECTIONS.contains(&k.as_str())) {
            return Err(CoreError::preset(
                format!("$.{k}"),
                "unknown key (a planet preset has schema, name, seed, planet, scale, timeline, \
                 resolution and tectonics)",
            ));
        }
        let planet: Option<PlanetParams> = section(obj, "planet")?;
        let scale: Option<ScaleParams> = section(obj, "scale")?;
        let timeline: Option<TimelineParams> = section(obj, "timeline")?;
        let resolution: Option<ResolutionParams> = section(obj, "resolution")?;
        let tectonics: Option<TectonicParams> = section(obj, "tectonics")?;
        let missing = |what: &str| CoreError::preset("$", format!("missing section {what}"));
        let schema = match obj.get("schema") {
            None => PRESET_SCHEMA,
            Some(v) => v
                .as_u64()
                .filter(|&s| s == u64::from(PRESET_SCHEMA))
                .ok_or_else(|| {
                    CoreError::preset(
                        "$.schema",
                        format!("this build reads schema {PRESET_SCHEMA}"),
                    )
                })? as u32,
        };
        let name = match obj.get("name") {
            None => "planet".to_owned(),
            Some(v) => v
                .as_str()
                .filter(|s| {
                    !s.is_empty()
                        && s.len() <= 64
                        && s.chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                })
                .ok_or_else(|| CoreError::preset("$.name", "1–64 of [A-Za-z0-9_-]"))?
                .to_owned(),
        };
        let seed = match obj.get("seed") {
            None => 1,
            Some(v) => v
                .as_u64()
                .ok_or_else(|| CoreError::preset("$.seed", "expected an integer ≥ 0"))?,
        };
        let preset = PlanetPreset {
            schema,
            name,
            seed,
            planet: planet.ok_or_else(|| missing("planet"))?,
            scale: scale.ok_or_else(|| missing("scale"))?,
            timeline: timeline.ok_or_else(|| missing("timeline"))?,
            resolution: resolution.ok_or_else(|| missing("resolution"))?,
            tectonics: tectonics.unwrap_or_default(),
        };
        preset.validate()?;
        Ok(preset)
    }

    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let p = &self.planet;
        check(
            p.gravity_m_s2 > 0.0 && p.gravity_m_s2 < 100.0,
            "$.planet.gravity_m_s2",
            "must be in (0, 100)",
        )?;
        check(p.day_length_h > 0.0, "$.planet.day_length_h", "must be > 0")?;
        check(
            (0.0..=180.0).contains(&p.axial_tilt_deg),
            "$.planet.axial_tilt_deg",
            "must be in [0, 180]",
        )?;
        check(
            p.star_flux_rel > 0.0,
            "$.planet.star_flux_rel",
            "must be > 0",
        )?;
        check(
            (0.05..=0.95).contains(&p.ocean_fraction),
            "$.planet.ocean_fraction",
            "must be in [0.05, 0.95]",
        )?;
        let s = &self.scale;
        check(
            s.circumference_blocks >= 1024
                && s.circumference_blocks <= 1 << 17
                && s.circumference_blocks.is_multiple_of(64),
            "$.scale.circumference_blocks",
            "must be a multiple of 64 in [1024, 131072]",
        )?;
        check(
            s.m_per_block_h > 0.0,
            "$.scale.m_per_block_h",
            "must be > 0",
        )?;
        check(
            s.m_per_block_v > 0.0,
            "$.scale.m_per_block_v",
            "must be > 0",
        )?;
        check(
            s.min_y < s.sea_level_y && s.sea_level_y < s.max_y,
            "$.scale.sea_level_y",
            "must lie between min_y and max_y",
        )?;
        let t = &self.timeline;
        if let Some(len) = t.length_myr {
            check(
                len.is_finite() && (2.0..=5000.0).contains(&len),
                "$.timeline.length_myr",
                "must be in [2, 5000]",
            )?;
        }
        check(
            t.keyframe_every_myr > 0.0,
            "$.timeline.keyframe_every_myr",
            "must be > 0",
        )?;
        check(
            (16..=4096).contains(&t.keyframe_size[0]) && (8..=4096).contains(&t.keyframe_size[1]),
            "$.timeline.keyframe_size",
            "width in [16, 4096], height in [8, 4096]",
        )?;
        let r = &self.resolution;
        check(
            (2_000..=2_000_000).contains(&r.l0_samples),
            "$.resolution.l0_samples",
            "must be in [2000, 2000000]",
        )?;
        check(
            r.l1_r > 0 && s.circumference_blocks.is_multiple_of(2 * r.l1_r),
            "$.resolution.l1_r",
            "C must be a multiple of 2·l1_r",
        )?;
        check(
            s.circumference_blocks / r.l1_r <= 8192,
            "$.resolution.l1_r",
            "L1 would be more than 8192 cells wide",
        )?;
        let k = &self.tectonics;
        let pos = |v: f64, key: &str| check(v.is_finite() && v > 0.0, key, "must be > 0");
        pos(k.time_step_myr, "$.tectonics.time_step_myr")?;
        pos(k.model_radius_km, "$.tectonics.model_radius_km")?;
        pos(k.max_plate_speed_mm_yr, "$.tectonics.max_plate_speed_mm_yr")?;
        pos(
            k.subduction_distance_km,
            "$.tectonics.subduction_distance_km",
        )?;
        pos(k.collision_distance_km, "$.tectonics.collision_distance_km")?;
        pos(
            k.collision_threshold_km,
            "$.tectonics.collision_threshold_km",
        )?;
        check(
            k.subduction_peak_km > 0.0 && k.subduction_peak_km < k.subduction_distance_km,
            "$.tectonics.subduction_peak_km",
            "must be in (0, subduction_distance_km)",
        )?;
        check(
            k.trench_elevation_m < k.abyssal_elevation_m
                && k.abyssal_elevation_m < k.ridge_elevation_m
                && k.ridge_elevation_m < 0.0
                && k.max_elevation_m > 0.0,
            "$.tectonics.ridge_elevation_m",
            "need trench < abyssal < ridge < 0 < max_elevation",
        )?;
        check(
            (2..=200).contains(&k.initial_plates),
            "$.tectonics.initial_plates",
            "must be in [2, 200]",
        )?;
        check(
            k.max_plates >= k.initial_plates && k.max_plates <= 1000,
            "$.tectonics.max_plates",
            "must be in [initial_plates, 1000]",
        )?;
        check(
            k.rift_fragments[0] >= 2 && k.rift_fragments[0] <= k.rift_fragments[1],
            "$.tectonics.rift_fragments",
            "[min ≥ 2, max ≥ min]",
        )?;
        check(
            k.resample_min_steps >= 1 && k.resample_min_steps <= k.resample_max_steps,
            "$.tectonics.resample_min_steps",
            "must be in [1, resample_max_steps]",
        )?;
        for (v, key) in [
            (k.plate_speed_mm_yr, "$.tectonics.plate_speed_mm_yr"),
            (k.rift_speed_mm_yr, "$.tectonics.rift_speed_mm_yr"),
            (k.hotspot_lifetime_myr, "$.tectonics.hotspot_lifetime_myr"),
            (k.continent_elevation_m, "$.tectonics.continent_elevation_m"),
            (
                k.subduction_initiation_speed_mm_yr,
                "$.tectonics.subduction_initiation_speed_mm_yr",
            ),
        ] {
            check(
                v[0].is_finite() && v[1].is_finite() && v[0] <= v[1],
                key,
                "[min, max] with min ≤ max",
            )?;
        }
        check(
            (0.0..=1.0).contains(&k.min_plate_area_frac),
            "$.tectonics.min_plate_area_frac",
            "must be in [0, 1]",
        )?;
        check(
            (0.0..=1.0).contains(&k.trench_relax),
            "$.tectonics.trench_relax",
            "must be in [0, 1]",
        )?;
        check(
            (0.0..=1.0).contains(&k.lip_probability),
            "$.tectonics.lip_probability",
            "must be in [0, 1]",
        )?;
        Ok(())
    }

    /// The canonical JSON (what `preset.json` and checkpoints store).
    pub fn to_json(&self) -> Value {
        json!({
            "schema": self.schema,
            "name": self.name,
            "seed": self.seed,
            "planet": self.planet,
            "scale": self.scale,
            "timeline": self.timeline,
            "resolution": self.resolution,
            "tectonics": self.tectonics,
        })
    }

    /// The chosen start, or the error a run reports when the player has not chosen one.
    pub fn start(&self) -> Result<StartConfig, CoreError> {
        self.timeline.start.ok_or_else(|| {
            CoreError::preset(
                "$.timeline.start",
                "choose a start configuration: supercontinent, scattered or random \
                 (there is no default story)",
            )
        })
    }

    /// The chosen length in Myr, or the error a run reports when none was chosen.
    pub fn length_myr(&self) -> Result<f64, CoreError> {
        self.timeline.length_myr.ok_or_else(|| {
            CoreError::preset(
                "$.timeline.length_myr",
                "choose a timeline length in Myr (there is no default story)",
            )
        })
    }

    /// Checks that everything a run needs is chosen.
    pub fn check_runnable(&self) -> Result<(), CoreError> {
        self.start()?;
        self.length_myr()?;
        Ok(())
    }

    /// Plate steps of the run: length / δt, rounded (δt is stretched to fit).
    pub fn plate_steps(&self) -> u64 {
        let len = self.timeline.length_myr.unwrap_or(0.0);
        (len / self.tectonics.time_step_myr).round().max(1.0) as u64
    }

    /// Years at the start of the history (negative).
    pub fn t_start_years(&self) -> f64 {
        -self.timeline.length_myr.unwrap_or(0.0) * 1.0e6
    }
}

/// Applies the timeline choices a CLI or server passes beside a preset (`--start`,
/// `--timeline-myr`) to preset JSON, returning the edited JSON. A dummy preset is returned as is.
pub fn with_timeline(
    json: &str,
    start: Option<&str>,
    length_myr: Option<f64>,
) -> Result<String, CoreError> {
    if start.is_none() && length_myr.is_none() {
        return Ok(json.to_owned());
    }
    let mut v: Value = serde_json::from_str(json)
        .map_err(|e| CoreError::preset("$", format!("not valid JSON: {e}")))?;
    let obj = v
        .as_object_mut()
        .ok_or_else(|| CoreError::preset("$", "expected a JSON object"))?;
    if obj.contains_key("dummy") {
        return Err(CoreError::preset(
            "$.dummy",
            "the dummy preset has no timeline to choose",
        ));
    }
    let timeline = obj
        .entry("timeline")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| CoreError::preset("$.timeline", "expected a JSON object"))?;
    if let Some(s) = start {
        if StartConfig::parse(s).is_none() {
            return Err(CoreError::preset(
                "$.timeline.start",
                format!("{s:?} is not supercontinent, scattered or random"),
            ));
        }
        timeline.insert("start".into(), json!(s));
    }
    if let Some(l) = length_myr {
        timeline.insert("length_myr".into(), json!(l));
    }
    Ok(v.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Result<PlanetPreset, CoreError> {
        let v: Value = serde_json::from_str(json).unwrap();
        PlanetPreset::from_obj(v.as_object().unwrap())
    }

    fn path_of(json: &str) -> String {
        match parse(json) {
            Err(CoreError::PresetInvalid { path, .. }) => path,
            other => panic!("expected PresetInvalid, got {other:?}"),
        }
    }

    #[test]
    fn builtins_parse_and_need_a_timeline() {
        for (name, json) in BUILTIN {
            let p = parse(json).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(p.name, name);
            assert!(p.timeline.start.is_none() && p.timeline.length_myr.is_none());
            assert!(matches!(
                p.check_runnable(),
                Err(CoreError::PresetInvalid { ref path, .. }) if path == "$.timeline.start"
            ));
            // The canonical JSON parses back to the same preset.
            let again = parse(&p.to_json().to_string()).unwrap();
            assert_eq!(again, p);
        }
        let quick = parse(builtin("earthlike_quick_16k").unwrap()).unwrap();
        assert_eq!(quick.resolution.l0_samples, 100_000);
        assert_eq!(quick.resolution.l1_r, 16);
        assert_eq!(quick.tectonics, TectonicParams::default());
        assert!((quick.scale.radius_m() - 782_278.4).abs() < 1.0);
        let full = parse(builtin("earthlike_full_16k").unwrap()).unwrap();
        assert_eq!(full.resolution.l0_samples, 250_000);
    }

    #[test]
    fn timeline_choices_apply() {
        let json = with_timeline(
            builtin("earthlike_quick_16k").unwrap(),
            Some("scattered"),
            Some(500.0),
        )
        .unwrap();
        let p = parse(&json).unwrap();
        p.check_runnable().unwrap();
        assert_eq!(p.start().unwrap(), StartConfig::Scattered);
        assert_eq!(p.plate_steps(), 250);
        assert_eq!(p.t_start_years(), -5.0e8);
        assert!(with_timeline("{}", Some("pangaea"), None).is_err());
    }

    #[test]
    fn errors_name_the_key() {
        assert_eq!(path_of("{}"), "$");
        assert_eq!(path_of(r#"{"planet":{}}"#), "$.planet");
        assert_eq!(path_of(r#"{"x":1}"#), "$.x");
        let base = with_timeline(
            builtin("earthlike_quick_16k").unwrap(),
            Some("random"),
            Some(100.0),
        )
        .unwrap();
        let mut v: Value = serde_json::from_str(&base).unwrap();
        v["tectonics"]["nope"] = json!(1);
        assert_eq!(path_of(&v.to_string()), "$.tectonics");
        let mut v: Value = serde_json::from_str(&base).unwrap();
        v["planet"]["ocean_fraction"] = json!(1.5);
        assert_eq!(path_of(&v.to_string()), "$.planet.ocean_fraction");
        let mut v: Value = serde_json::from_str(&base).unwrap();
        v["resolution"]["l1_r"] = json!(7);
        assert_eq!(path_of(&v.to_string()), "$.resolution.l1_r");
    }
}

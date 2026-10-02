// SPDX-License-Identifier: LGPL-3.0-or-later
//! The planet preset (plan §5.1): the sections a preset running the real pipeline carries next to
//! the header every preset has (`preset.rs`: schema, name, description, placeholder, scale,
//! timeline).
//!
//! ```json
//! {"schema": 0, "name": "plates_quick_16k", "seed": 1,
//!  "planet": {"gravity_m_s2": 9.81, "day_length_h": 24, "axial_tilt_deg": 23.4,
//!             "star_flux_rel": 1.0, "ocean_fraction": 0.7, "regime": "mobile_lid"},
//!  "scale": {"circumference_blocks": 16384, "m_per_block_h": 300, …},
//!  "timeline": {"start": "scattered_continents", "length_myr": 500},
//!  "keyframes": {"every_myr": 50, "size": [1024, 512]},
//!  "resolution": {"l0_samples": 100000, "l1_r": 16},
//!  "tectonics": {…every key optional, Cortial et al. 2019 Appendix A by default…}}
//! ```
//!
//! A preset with a `planet` section runs the plates stage (P1-03); later stages add their sections
//! (`surface`, `climate`, …). **The timeline has no default story** (decisions file, "Timeline"):
//! the plate presets leave `timeline` out and a run refuses to start until it is set (the editor,
//! `config/deeptime/server.json` or the CLI's `--start`/`--length-myr`).
//!
//! The built-in `earthlike_*` presets (`preset::BUILTIN`) are the planet pipeline's presets under
//! their game names since P1-08 (`tools/dev/sync_builtin_presets.py`); the plate pipeline's
//! presets are `plates_*` (`presets/plates/`, [`PLATES_BUILTIN`]) and the full pipeline's
//! `pipeline_*` ([`PIPELINE_BUILTIN`], the files calibration edits), resolved by name like the
//! built-ins.
//!
//! Sections are parsed with serde, one at a time, so an error names the section
//! (`$.planet: missing field gravity_m_s2`); range checks name the exact key. Every section
//! rejects unknown keys.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::error::CoreError;
use crate::preset::{StartConfig, Timeline};

/// The plate pipeline's presets (`crates/deeptime-core/presets/plates/`): the Earth-like planets
/// of plan §5.1 with their tectonic section, not shipped as game built-ins until P1-08.
pub const PLATES_BUILTIN: [(&str, &str); 4] = [
    (
        "plates_quick_16k",
        include_str!("../presets/plates/plates_quick_16k.json"),
    ),
    (
        "plates_full_16k",
        include_str!("../presets/plates/plates_full_16k.json"),
    ),
    (
        "plates_quick_32k",
        include_str!("../presets/plates/plates_quick_32k.json"),
    ),
    (
        "plates_full_32k",
        include_str!("../presets/plates/plates_full_32k.json"),
    ),
];

/// The JSON of a plate preset.
pub fn plates_builtin(name: &str) -> Option<&'static str> {
    PLATES_BUILTIN
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, j)| *j)
}

/// The planet pipeline's presets (`crates/deeptime-core/presets/pipeline/`): every stage from the
/// plates to the amplifier. The game's `earthlike_*` built-ins are these under other names (P1-08;
/// `tools/dev/sync_builtin_presets.py` copies them, `preset::tests::builtins_are_the_pipeline_presets`
/// checks it).
pub const PIPELINE_BUILTIN: [(&str, &str); 4] = [
    (
        "pipeline_quick_16k",
        include_str!("../presets/pipeline/pipeline_quick_16k.json"),
    ),
    (
        "pipeline_full_16k",
        include_str!("../presets/pipeline/pipeline_full_16k.json"),
    ),
    (
        "pipeline_quick_32k",
        include_str!("../presets/pipeline/pipeline_quick_32k.json"),
    ),
    (
        "pipeline_full_32k",
        include_str!("../presets/pipeline/pipeline_full_32k.json"),
    ),
];

/// The JSON of a plate or pipeline preset.
pub fn planet_builtin(name: &str) -> Option<&'static str> {
    plates_builtin(name).or_else(|| {
        PIPELINE_BUILTIN
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, j)| *j)
    })
}

/// The top-level keys a planet preset adds to the header's.
pub const PLANET_KEYS: [&str; 17] = [
    "seed",
    "planet",
    "keyframes",
    "resolution",
    "tectonics",
    "surface",
    "climate",
    "structure",
    "deposits",
    "young",
    "amplify",
    "biomes",
    "hydro",
    "volcanoes",
    "underground",
    "provinces",
    "impacts",
];

/// The `scale` keys a planet preset may carry (the header reads `circumference_blocks`).
pub const SCALE_KEYS: [&str; 9] = [
    "circumference_blocks",
    "m_per_block_h",
    "m_per_block_v",
    "min_y",
    "max_y",
    "sea_level_y",
    "pole_cap_deg",
    "feature_scale",
    "build_limit_band_blocks",
];

/// The most of the planet the initial continental crust, (1 − ocean_fraction)/(1 − shelf share),
/// may cover: the rest is ocean floor. All continental crust (no ocean floor to subduct, no ridges)
/// gives NaN elevations from the surface stage; 0.986 was run and works.
pub const MAX_CONTINENTAL_SHARE: f64 = 0.98;

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
    -512
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
fn d_build_limit_band() -> f64 {
    48.0
}

/// The most the detail term adds to the baked elevation, blocks (`detail/amplitude` is a u8 of
/// sixteenths: at most 255/16).
pub const DETAIL_HEADROOM_BLOCKS: f64 = 16.0;

/// How the finished terrain is kept under the world's build limit (Ben, decisions log 2026-10-01:
/// Minecraft's height is a real limit; nothing is ever cut flat). Elevations up to `knee_y` are
/// stored as simulated; above it they are compressed smoothly towards `top_y`, which they never
/// reach: y ↦ knee + B·(1 − e^(−(y − knee)/B)), B = top − knee. The map is increasing and its slope
/// is 1 at the knee, so ridges stay ridges and drainage keeps its direction. `top_y` leaves room
/// for the detail term, so E + A·N stays below `max_y`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BuildFit {
    pub knee_y: f64,
    pub top_y: f64,
}

impl BuildFit {
    /// The stored elevation of simulated elevation `y`, blocks.
    pub fn compress(&self, y: f64) -> f64 {
        let band = self.top_y - self.knee_y;
        // (NaN stays NaN.)
        if y > self.knee_y && band > 0.0 {
            self.knee_y + band * (1.0 - (-(y - self.knee_y) / band).exp())
        } else {
            y
        }
    }
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
    /// How big the world's features are drawn in blocks, relative to their real size at the
    /// block scale (the editor's "feature scale" settings); 1 everywhere is the calibrated world.
    #[serde(default)]
    pub feature_scale: FeatureScale,
    /// The band under the build limit that terrain above its foot is compressed into, blocks
    /// ([`BuildFit`]): the foot (the knee) is at `max_y` − 16 − this. Terrain below the knee is
    /// as simulated.
    #[serde(default = "d_build_limit_band")]
    pub build_limit_band_blocks: f64,
}

/// Multipliers of feature sizes in blocks (plan §5.1 `feature_scale`; the editor's Multipliers
/// section). Only features the model has today are here: rivers since P1-22; canyons arrive with
/// Phase 2's cave stage and get their key then.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct FeatureScale {
    /// Ore bodies: their length, width and height in blocks (the tonnage, grade and site
    /// records stay as they are; `deposits.size_mult` is the amount of ore).
    pub deposits: f64,
    /// Rock layers: the thickness in blocks of every stratum of the baked columns (the history
    /// and the ore stay as they are; `scale.m_per_block_v` moves everything).
    pub strata: f64,
    /// Rivers: the width in blocks of every channel (P1-22; the discharge stays as it is).
    pub rivers: f64,
}

impl Default for FeatureScale {
    fn default() -> Self {
        FeatureScale {
            deposits: 1.0,
            strata: 1.0,
            rivers: 1.0,
        }
    }
}

/// The density of continental crust the strength comparison uses, kg/m³.
pub const CRUST_DENSITY_KG_M3: f64 = 2800.0;

impl TectonicParams {
    /// The height above the sea at which the crust's own weight reaches its strength,
    /// S / (ρ_c · g), m: where spreading starts. Gravity enters here and nowhere else.
    pub fn strength_height_m(&self, gravity_m_s2: f64) -> f64 {
        self.crust_strength_mpa * 1.0e6 / (CRUST_DENSITY_KG_M3 * gravity_m_s2)
    }
}

impl ScaleParams {
    /// Where the finished terrain's compression starts and what it never reaches ([`BuildFit`]).
    pub fn build_fit(&self) -> BuildFit {
        let top_y = f64::from(self.max_y) - DETAIL_HEADROOM_BLOCKS;
        BuildFit {
            knee_y: top_y - self.build_limit_band_blocks,
            top_y,
        }
    }

    /// The highest elevation above the sea the world shows as simulated (the knee), m.
    pub fn uncompressed_height_m(&self) -> f64 {
        (self.build_fit().knee_y - f64::from(self.sea_level_y)) * self.m_per_block_v
    }

    /// radius_m = C · m_per_block_h / 2π (plan §5.1).
    pub fn radius_m(&self) -> f64 {
        f64::from(self.circumference_blocks) * self.m_per_block_h / std::f64::consts::TAU
    }
}

fn d_every_myr() -> f64 {
    50.0
}
fn d_keyframe_size() -> [u32; 2] {
    [1024, 512]
}

/// Timelapse keyframes (plan §6.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyframeParams {
    #[serde(default = "d_every_myr")]
    pub every_myr: f64,
    /// Width and height in pixels.
    #[serde(default = "d_keyframe_size")]
    pub size: [u32; 2],
}

impl Default for KeyframeParams {
    fn default() -> Self {
        KeyframeParams {
            every_myr: d_every_myr(),
            size: d_keyframe_size(),
        }
    }
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
    /// A numerical guard on the plate model's elevation, not a limit that shapes the world: with
    /// the crust spreading (`crust_strength_mpa`) the pipeline presets put it at 30 km, where it
    /// never binds (`plate_cap_bound_share` on the `surface_final` line says how often it did).
    /// Cortial's z_c (10 km) in the plate-only presets, which have nothing else to stop a range.
    pub max_elevation_m: f64,
    /// Cortial's z_c as the scale of the uplift transfer function: the subducting crust's height
    /// is measured between the trench depth and this (10 km). A scale only; nothing is limited
    /// by it.
    pub uplift_reference_m: f64,
    /// The height at which tectonic uplift fades to nothing, uplift × (1 − z/this) (Cortial-style
    /// isostatic ceiling, `plates::subduction::ceiling`); 0 = no fading. The plate-only presets
    /// keep 10 km. The pipeline presets set 0: there the height of a range is what uplift,
    /// erosion and the spreading of over-thick crust leave (Ben, decisions log 2026-10-01: the
    /// limit must emerge, not be a number).
    pub uplift_ceiling_m: f64,
    /// The strength of the crust, MPa: where the weight of the land above sea level, ρ_c·g·h,
    /// passes it, the lower crust flows out from under the range and the surface spreads into a
    /// plateau (`surface::spread`). 140 MPa holds 5.1 km at Earth's gravity (Tibet stands near
    /// 5 km), 13.4 km at Mars's.
    pub crust_strength_mpa: f64,
    /// How fast over-thick crust spreads: the diffusivity of the surface at twice the strength
    /// height, km²/Myr (it grows with the cube of the excess, the channel-flow law); 0 = off.
    pub crust_flow_km2_myr: f64,
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
    /// The elevation ageing ocean floor subsides towards, m: z(a) = z_f + (z_r − z_f)·
    /// exp(−a·ε_o/|z_f|). Cortial's model subsides towards the trench depth (−10 km, the default
    /// and the plate-only presets'), which leaves floor older than ~120 Myr deeper than any abyssal
    /// plain. −5,600 m is GDH1's old-floor depth (Stein & Stein 1992, Nature 359:123–129); with
    /// ε_o 0.215 mm/yr from a −2,500 m ridge the law follows GDH1 within 0.12 km from 20 Myr on.
    /// The pipeline presets use −5,600 m with ε_o 0.12 mm/yr (3.6 km at 20 Myr, 4.5 at 50, 5.2 at
    /// 100, 5.6 beyond): at GDH1's full rate the young floor is deep enough that island arcs rarely
    /// emerge and the continental crust shrinks from 0.39 to 0.29 of the planet in 500 Myr (P1-17
    /// hypsometry recalibration, 2026-10-01; median sea depth 4.0 km, Earth 4.06).
    pub oceanic_floor_elevation_m: f64,
    /// ε_c (0.03 mm/yr).
    pub continental_erosion_mm_yr: f64,
    /// ε_t (0.3 mm/yr).
    pub sediment_accretion_mm_yr: f64,
    /// u_0 (0.6 mm/yr).
    pub subduction_uplift_mm_yr: f64,
    /// What a contact's collision count must reach to transfer its terrane or weld its plates
    /// (Cortial §6: 300 km of interpenetration). What is counted is `collision_on_shortening`.
    pub collision_threshold_km: f64,

    /// Where the subduction distance transfer f(d) peaks (design choice).
    pub subduction_peak_km: f64,
    /// Cap on Δc·A, the discrete collision uplift (design choice; Δc·A of a whole continent
    /// would be tens of kilometres).
    pub collision_uplift_max_m: f64,
    /// Crust lifted by at least this share of a collision's peak uplift becomes a collision
    /// orogen (Himalayan orogeny, folds, regional metamorphism); the rest of the surge only rises
    /// (P1-27: Earth's collision belts are a few hundred to ~1,500 km wide, not the whole surge).
    pub collision_orogen_core: f64,
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
    /// Share of the continental crust under the sea (shelves, slopes, epeiric seas): the initial
    /// continental area is (1 − ocean_fraction)/(1 − this), its part nearest the ocean born as a
    /// flooded shelf, so `planet.ocean_fraction` is the share of the surface under water rather
    /// than of oceanic crust. Earth: 29 % land on 41 % continental crust (Cogley 1984, Rev.
    /// Geophys. 22:101–122) → 0.29, the pipeline presets' value, whose surface stage floods the
    /// continents' margins. 0 (the default) makes all continental crust dry at the start (P1-03;
    /// the plate-only presets, whose continents never flood).
    pub continental_shelf_share: f64,
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
    /// Arc magma an island arc must have accumulated (`arc_oceanic_m`) before its emerged crust
    /// counts as continental, metres (P1-27; 0 converts any emerged arc at once, as P1-03 did;
    /// intra-oceanic arcs take tens of Myr to build 20–35 km of crust, Stern 2002).
    pub arc_continental_m: f64,
    /// Uplift of continental crust over a magmatic arc at v = v_0, mm/yr: the arc's own
    /// cordillera, built by arc magmatism and crustal thickening (P1-27; 0, the default and the
    /// plate-only presets' value, leaves only Cortial's subduction uplift; the planet pipeline
    /// presets use 0.2).
    pub cordillera_uplift_mm_yr: f64,
    /// … the half-width of its band, centred on the arc axis, km (Earth-normalized; the Andes
    /// and Cascades ranges are 100–300 km wide across the arc) …
    pub cordillera_width_km: f64,
    /// … and the height it builds towards, m: the uplift slows to nothing there (thickened arc
    /// crust spreads; the Central Andes' plateau stands near 4 km, the Cascades' base 1.2–2 km).
    pub cordillera_height_m: f64,
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
    /// A colliding continent of at least this fraction of A_0 sutures its whole plate to the
    /// receiving one, keeping the ocean on its other sides (P1-27: moving the continent alone
    /// left its passive margins facing an ocean that kept converging, so they all turned
    /// active); a smaller terrane moves alone (an island arc or microcontinent accretes).
    pub collision_suture_frac: f64,
    /// What `collision_threshold_km` is compared with (`plates/shortening.rs`). `true`: the
    /// collision's real shortening, the convergence across its front integrated since continental
    /// crust first converged there. `false`: the count used before plate-motion task 2, relative
    /// speed × time over the steps with three or more continental samples in the overlap, which
    /// also counts plates sliding past each other. An interim setting: `false` in every built-in
    /// preset until the belt and the weld on stalled convergence land (tasks 3 and 4 of
    /// `docs/design/plate-force-balance.md`): with it off the planets are master's, and with it on
    /// the plate-check measures move both ways (docs/design/plate-check-baseline.md, task 2).
    /// Task 4 removes it.
    // ledger: PLT-227 (game-choice, interim; the agent of plate-motion task 2, 2026-10-02).
    pub collision_on_shortening: bool,
    /// Oceanic crust older than this, on a plate that subducts nowhere, can start subducting on
    /// its own (Wilson cycle), Myr.
    pub subduction_initiation_age_myr: f64,
    /// Expected subduction initiations per step per A_0 of such old crust.
    pub subduction_initiation_rate: f64,
    /// Old crust can start subducting only this far from every trench, km (Earth-normalized;
    /// a new subduction zone does not open beside a working one).
    pub subduction_initiation_clearance_km: f64,
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
            uplift_reference_m: 10000.0,
            uplift_ceiling_m: 10000.0,
            crust_strength_mpa: 140.0,
            crust_flow_km2_myr: 0.0,
            subduction_distance_km: 1800.0,
            collision_distance_km: 4200.0,
            collision_coefficient_per_km: 1.3e-5,
            max_plate_speed_mm_yr: 100.0,
            oceanic_damping_mm_yr: 0.04,
            oceanic_floor_elevation_m: -10000.0,
            continental_erosion_mm_yr: 0.03,
            sediment_accretion_mm_yr: 0.3,
            subduction_uplift_mm_yr: 0.6,
            collision_threshold_km: 300.0,
            subduction_peak_km: 300.0,
            collision_uplift_max_m: 4000.0,
            collision_orogen_core: 0.5,
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
            continental_shelf_share: 0.0,
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
            arc_continental_m: 0.0,
            cordillera_uplift_mm_yr: 0.0,
            cordillera_width_km: 120.0,
            cordillera_height_m: 2500.0,
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
            collision_suture_frac: 0.1,
            collision_on_shortening: false,
            subduction_initiation_age_myr: 150.0,
            subduction_initiation_rate: 0.3,
            subduction_initiation_clearance_km: 1000.0,
            subduction_initiation_speed_mm_yr: [30.0, 60.0],
            contact_timeout_steps: 12,
            collision_orogeny_myr: 30.0,
            l1_every_steps: 0,
        }
    }
}

/// A planet preset's body (plan §5.1).
#[derive(Clone, Debug, PartialEq)]
pub struct PlanetPreset {
    pub name: String,
    pub seed: u64,
    pub planet: PlanetParams,
    pub scale: ScaleParams,
    /// The header's timeline: required before a run.
    pub timeline: Option<Timeline>,
    pub keyframes: KeyframeParams,
    pub resolution: ResolutionParams,
    pub tectonics: TectonicParams,
    /// The deep-time surface (P1-04); `None` runs the plates alone.
    pub surface: Option<crate::surface::SurfaceParams>,
    /// The climate (P1-05); `None` leaves the surface on its zonal stand-in.
    pub climate: Option<crate::climate::ClimateParams>,
    /// Column strata, metamorphism and plutons (P1-07); runs with the surface, with the
    /// defaults when the section is absent.
    pub structure: Option<crate::structure::StructureParams>,
    /// The deposit engine (P1-14) on the history the surface stage records; `None` stops after
    /// the surface. It needs a `surface` section.
    pub deposits: Option<crate::deposits::DepositParams>,
    /// The young landscape on L2 (P1-06); `None` leaves the surface on L1. It needs a `surface`
    /// section.
    pub young: Option<crate::surface::young::YoungParams>,
    /// Amplification to L3 (P1-06) and the detail layer; `None` writes no L3. It needs a
    /// `young` section.
    pub amplify: Option<crate::surface::amplify::AmplifyParams>,
    /// Volcanoes as landforms (P1-21); `None` makes none. It needs a `surface` section.
    pub volcanoes: Option<crate::surface::volcanoes::VolcanoParams>,
    /// Meteor impacts (P2-08); `None` makes none. It needs a `surface` section.
    pub impacts: Option<crate::surface::impacts::ImpactParams>,
    /// The knobs of the climate → biome-input mapping (P1-09, plan §7.2); the bake uses the
    /// defaults when the section is absent.
    pub biomes: Option<crate::bake::biomes::BiomeParams>,
    /// Rivers and lakes on the finished terrain (P1-22); the bake uses the defaults when the
    /// section is absent.
    pub hydro: Option<crate::hydro::HydroParams>,
    /// What the game puts in the open space below ground (the lava floor and lava lakes); the
    /// bake records the defaults when the section is absent.
    pub underground: Option<crate::underground::UndergroundParams>,
    /// What the bake's geologic provinces are (P1-10): they drive TerraBlender's regions; the bake
    /// uses the defaults when the section is absent.
    pub provinces: Option<crate::provinces::ProvinceParams>,
    /// The climate class placement table a datapack supplied through the content bundle
    /// (`climate_placement`, `bake::placement`); `None` uses the core's default table. Not part
    /// of the preset's JSON: it is pack data, recorded in the world data by the bake.
    pub placement: Option<crate::bake::placement::Placement>,
}

type Obj = Map<String, Value>;

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
    /// Parses the planet sections of a preset whose header is already parsed. Sections present
    /// are parsed in a fixed order before missing ones are reported, so a malformed `planet`
    /// section is named even when others are missing.
    pub fn from_obj(
        obj: &Obj,
        name: Option<&str>,
        timeline: Option<Timeline>,
    ) -> Result<PlanetPreset, CoreError> {
        let planet: Option<PlanetParams> = section(obj, "planet")?;
        let scale: Option<ScaleParams> = section(obj, "scale")?;
        let resolution: Option<ResolutionParams> = section(obj, "resolution")?;
        let tectonics: Option<TectonicParams> = section(obj, "tectonics")?;
        let keyframes: Option<KeyframeParams> = section(obj, "keyframes")?;
        let surface: Option<crate::surface::SurfaceParams> = section(obj, "surface")?;
        let climate: Option<crate::climate::ClimateParams> = section(obj, "climate")?;
        let structure: Option<crate::structure::StructureParams> = section(obj, "structure")?;
        let deposits: Option<crate::deposits::DepositParams> = section(obj, "deposits")?;
        let young: Option<crate::surface::young::YoungParams> = section(obj, "young")?;
        let amplify: Option<crate::surface::amplify::AmplifyParams> = section(obj, "amplify")?;
        let biomes: Option<crate::bake::biomes::BiomeParams> = section(obj, "biomes")?;
        let hydro: Option<crate::hydro::HydroParams> = section(obj, "hydro")?;
        let volcanoes: Option<crate::surface::volcanoes::VolcanoParams> =
            section(obj, "volcanoes")?;
        let underground: Option<crate::underground::UndergroundParams> =
            section(obj, "underground")?;
        let provinces: Option<crate::provinces::ProvinceParams> = section(obj, "provinces")?;
        let impacts: Option<crate::surface::impacts::ImpactParams> = section(obj, "impacts")?;
        let missing = |what: &str| CoreError::preset("$", format!("missing section {what}"));
        let seed = match obj.get("seed") {
            None => 1,
            Some(v) => v
                .as_u64()
                .ok_or_else(|| CoreError::preset("$.seed", "expected an integer ≥ 0"))?,
        };
        let preset = PlanetPreset {
            name: name.unwrap_or("planet").to_owned(),
            seed,
            planet: planet.ok_or_else(|| missing("planet"))?,
            scale: scale.ok_or_else(|| missing("scale"))?,
            timeline,
            keyframes: keyframes.unwrap_or_default(),
            resolution: resolution.ok_or_else(|| missing("resolution"))?,
            tectonics: tectonics.unwrap_or_default(),
            surface,
            climate,
            structure,
            deposits,
            young,
            amplify,
            volcanoes,
            impacts,
            biomes,
            hydro,
            underground,
            provinces,
            placement: None,
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
        for (v, key) in [
            (s.feature_scale.deposits, "$.scale.feature_scale.deposits"),
            (s.feature_scale.strata, "$.scale.feature_scale.strata"),
            (s.feature_scale.rivers, "$.scale.feature_scale.rivers"),
        ] {
            check(
                v.is_finite() && (0.1..=10.0).contains(&v),
                key,
                "must be in [0.1, 10]",
            )?;
        }
        let kf = &self.keyframes;
        check(kf.every_myr > 0.0, "$.keyframes.every_myr", "must be > 0")?;
        check(
            (16..=4096).contains(&kf.size[0]) && (8..=4096).contains(&kf.size[1]),
            "$.keyframes.size",
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
        check(
            s.build_limit_band_blocks.is_finite() && s.build_limit_band_blocks >= 8.0,
            "$.scale.build_limit_band_blocks",
            "must be at least 8",
        )?;
        check(
            s.build_fit().knee_y >= f64::from(s.sea_level_y) + 64.0,
            "$.scale.build_limit_band_blocks",
            "leaves less than 64 blocks between sea level and the compression band under max_y",
        )?;
        let k = &self.tectonics;
        // No check that the plate model's elevation guard fits under the knee: the editor's
        // sliders (vertical scale, this band) must be valid at both ends, and a world too short
        // for its mountains is squeezed, not refused. The bake says how much
        // (`notes.bake.terrain.build_limit.cells_compressed`).
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
        pos(k.slab_pull_relax_myr, "$.tectonics.slab_pull_relax_myr")?;
        pos(k.slab_pull_front_km, "$.tectonics.slab_pull_front_km")?;
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
            k.trench_elevation_m <= k.oceanic_floor_elevation_m
                && k.oceanic_floor_elevation_m < k.ridge_elevation_m,
            "$.tectonics.oceanic_floor_elevation_m",
            "must be in [trench_elevation_m, ridge_elevation_m)",
        )?;
        check(
            k.uplift_reference_m > 0.0 && k.uplift_ceiling_m >= 0.0,
            "$.tectonics.uplift_ceiling_m",
            "need uplift_reference_m > 0 and uplift_ceiling_m >= 0 (0 = no ceiling)",
        )?;
        check(
            k.crust_strength_mpa > 0.0
                && k.crust_flow_km2_myr >= 0.0
                && k.crust_flow_km2_myr.is_finite(),
            "$.tectonics.crust_strength_mpa",
            "need crust_strength_mpa > 0 and crust_flow_km2_myr >= 0",
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
        for (v, key) in [
            (k.collision_orogen_core, "$.tectonics.collision_orogen_core"),
            (k.min_plate_area_frac, "$.tectonics.min_plate_area_frac"),
            (k.trench_relax, "$.tectonics.trench_relax"),
            (k.lip_probability, "$.tectonics.lip_probability"),
        ] {
            check((0.0..=1.0).contains(&v), key, "must be in [0, 1]")?;
        }
        check(
            (0.0..0.9).contains(&k.continental_shelf_share),
            "$.tectonics.continental_shelf_share",
            "must be in [0, 0.9)",
        )?;
        for (v, key) in [
            (k.collision_suture_frac, "$.tectonics.collision_suture_frac"),
            (
                k.subduction_initiation_clearance_km,
                "$.tectonics.subduction_initiation_clearance_km",
            ),
            (
                k.cordillera_uplift_mm_yr,
                "$.tectonics.cordillera_uplift_mm_yr",
            ),
        ] {
            check(v.is_finite() && v >= 0.0, key, "must be ≥ 0")?;
        }
        pos(k.cordillera_width_km, "$.tectonics.cordillera_width_km")?;
        pos(k.cordillera_height_m, "$.tectonics.cordillera_height_m")?;
        // A planet without ocean floor has no ocean crust for the plate model and the surface to
        // work with: the surface stage's elevations come out NaN everywhere (found by the settings
        // audit's extremes: ocean_fraction 0.25 with the pipeline presets' shelf share 0.29).
        let continental = (1.0 - p.ocean_fraction) / (1.0 - k.continental_shelf_share);
        if continental > MAX_CONTINENTAL_SHARE {
            return Err(CoreError::preset(
                "$.tectonics.continental_shelf_share",
                format!(
                    "the continental crust, (1 − ocean_fraction)/(1 − continental_shelf_share) = \
                     {continental:.3} of the planet, must leave at least {:.0} % ocean floor: raise \
                     planet.ocean_fraction or lower the shelf share",
                    100.0 * (1.0 - MAX_CONTINENTAL_SHARE)
                ),
            ));
        }
        if let Some(s) = &self.structure {
            s.validate()?;
            check(
                self.surface.is_some(),
                "$.structure",
                "needs a surface section (the structure stage reads the strata log)",
            )?;
        }
        if let Some(s) = &self.surface {
            s.validate()?;
            check(
                k.l1_every_steps == 1,
                "$.tectonics.l1_every_steps",
                "must be 1 with a surface section (the surface stage reads the plates' L1 fields \
                 after every plate step)",
            )?;
        }
        if let Some(c) = &self.climate {
            c.validate()?;
            check(
                k.l1_every_steps == 1,
                "$.tectonics.l1_every_steps",
                "must be 1 with a climate section (the climate stage reads the plates' L1 fields \
                 after every plate step)",
            )?;
        }
        if let Some(d) = &self.deposits {
            d.validate("$.deposits")?;
            check(
                self.surface.is_some(),
                "$.deposits",
                "needs a surface section: the deposit stage reads the field history the surface \
                 stage records",
            )?;
        }
        if let Some(y) = &self.young {
            y.validate()?;
            check(
                self.surface.is_some(),
                "$.young",
                "needs a surface section: the young landscape starts from the deep-time surface",
            )?;
            check(
                crate::surface::amplify::doublings(r.l1_r, r.l2_r).is_some(),
                "$.resolution.l2_r",
                "l1_r must be a power of two times l2_r (L2 is L1 doubled once or more)",
            )?;
            check(
                s.circumference_blocks.is_multiple_of(2 * r.l2_r),
                "$.resolution.l2_r",
                "C must be a multiple of 2·l2_r",
            )?;
        }
        if let Some(a) = &self.amplify {
            a.validate()?;
            check(
                self.young.is_some(),
                "$.amplify",
                "needs a young section: the amplifier starts from the young landscape",
            )?;
            check(
                crate::surface::amplify::doublings(r.l2_r, r.l3_r).is_some_and(|n| n >= 1),
                "$.resolution.l3_r",
                "l2_r must be a power of two times l3_r, and larger (L3 is L2 doubled once or more)",
            )?;
            check(
                s.circumference_blocks.is_multiple_of(2 * r.l3_r),
                "$.resolution.l3_r",
                "C must be a multiple of 2·l3_r",
            )?;
        }
        if let Some(b) = &self.biomes {
            b.validate()?;
        }
        if let Some(h) = &self.hydro {
            h.validate()?;
        }
        if let Some(u) = &self.underground {
            u.validate()?;
        }
        if let Some(p) = &self.provinces {
            p.validate()?;
        }
        if let Some(v) = &self.volcanoes {
            v.validate()?;
            check(
                self.surface.is_some(),
                "$.volcanoes",
                "needs a surface section: volcanoes build on the deep-time surface",
            )?;
        }
        if let Some(i) = &self.impacts {
            i.validate()?;
            check(
                self.surface.is_some(),
                "$.impacts",
                "needs a surface section: craters are cut into the deep-time surface",
            )?;
        }
        Ok(())
    }

    /// The body's canonical JSON; `Preset::to_json` adds the header (schema, name, description,
    /// placeholder, timeline).
    pub fn to_json(&self) -> Value {
        let mut v = json!({
            "seed": self.seed,
            "planet": self.planet,
            "scale": self.scale,
            "keyframes": self.keyframes,
            "resolution": self.resolution,
            "tectonics": self.tectonics,
        });
        if let Some(s) = &self.surface {
            v["surface"] = json!(s);
        }
        if let Some(c) = &self.climate {
            v["climate"] = json!(c);
        }
        if let Some(s) = &self.structure {
            v["structure"] = json!(s);
        }
        if let Some(d) = &self.deposits {
            v["deposits"] = json!(d);
        }
        if let Some(y) = &self.young {
            v["young"] = json!(y);
        }
        if let Some(a) = &self.amplify {
            v["amplify"] = json!(a);
        }
        if let Some(b) = &self.biomes {
            v["biomes"] = json!(b);
        }
        if let Some(h) = &self.hydro {
            v["hydro"] = json!(h);
        }
        if let Some(x) = &self.volcanoes {
            v["volcanoes"] = json!(x);
        }
        if let Some(u) = &self.underground {
            v["underground"] = json!(u);
        }
        if let Some(p) = &self.provinces {
            v["provinces"] = json!(p);
        }
        if let Some(i) = &self.impacts {
            v["impacts"] = json!(i);
        }
        v
    }

    /// The chosen start, or the error a run reports when the player has not chosen one.
    pub fn start(&self) -> Result<StartConfig, CoreError> {
        self.timeline.map(|t| t.start).ok_or_else(|| {
            CoreError::preset(
                "$.timeline.start",
                "choose a start configuration: supercontinent, scattered_continents or random \
                 (there is no default story)",
            )
        })
    }

    /// Checks that everything a run needs is chosen.
    pub fn check_runnable(&self) -> Result<(), CoreError> {
        self.start().map(|_| ())
    }

    /// The history's length in Myr (0 before a timeline is chosen).
    pub fn length_myr(&self) -> f64 {
        self.timeline.map_or(0.0, |t| t.length_myr)
    }

    /// Plate steps of the run: length / δt, rounded (δt is stretched to fit).
    pub fn plate_steps(&self) -> u64 {
        (self.length_myr() / self.tectonics.time_step_myr)
            .round()
            .max(1.0) as u64
    }

    /// Years at the start of the history (negative).
    pub fn t_start_years(&self) -> f64 {
        -self.length_myr() * 1.0e6
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::{Preset, PresetBody, with_timeline};

    fn planet(json: &str) -> PlanetPreset {
        match Preset::from_json(json).unwrap().body {
            PresetBody::Planet(p) => *p,
            other => panic!("not a planet preset: {other:?}"),
        }
    }

    fn path_of(json: &str) -> String {
        match Preset::from_json(json) {
            Err(CoreError::PresetInvalid { path, .. }) => path,
            other => panic!("expected PresetInvalid, got {other:?}"),
        }
    }

    #[test]
    fn plate_presets_parse_and_need_a_timeline() {
        for (name, json) in PLATES_BUILTIN {
            let p = planet(json);
            assert_eq!(p.name, name);
            assert!(p.timeline.is_none(), "{name}: no default story");
            assert!(matches!(
                p.check_runnable(),
                Err(CoreError::PresetInvalid { ref path, .. }) if path == "$.timeline.start"
            ));
            // The canonical JSON parses back to the same preset.
            let pr = Preset::from_json(json).unwrap();
            assert_eq!(Preset::from_json(&pr.to_json().to_string()).unwrap(), pr);
            assert_eq!(pr.circumference(), p.scale.circumference_blocks);
        }
        let quick = planet(plates_builtin("plates_quick_16k").unwrap());
        assert_eq!(quick.resolution.l0_samples, 100_000);
        assert_eq!(quick.resolution.l1_r, 16);
        assert_eq!(quick.tectonics, TectonicParams::default());
        assert!((quick.scale.radius_m() - 782_278.4).abs() < 1.0);
        let full = planet(plates_builtin("plates_full_16k").unwrap());
        assert_eq!(full.resolution.l0_samples, 250_000);
    }

    /// The settings audit (decisions log 2026-09-29: tuning knobs that shape the world are settings): the
    /// editors change only keys the preset carries, so the pipeline presets write out the sections
    /// that used to be absent or empty (`structure`, `amplify`) at the values those sections take
    /// when the keys are missing, so a default run is unchanged (STATUS.md's settings audit entry compares
    /// the world files of a 100 Myr run byte for byte).
    #[test]
    fn the_pipeline_presets_write_out_the_tuning_knobs_at_their_defaults() {
        for (name, json) in PIPELINE_BUILTIN {
            let p = planet(json);
            assert_eq!(
                p.amplify,
                Some(crate::surface::amplify::AmplifyParams::default()),
                "{name}: the terrain carving is medium (Ben, 2026-09-29)"
            );
            assert_eq!(
                p.structure,
                Some(crate::structure::StructureParams::default()),
                "{name}: pluton rate and size"
            );
            assert_eq!(p.amplify.as_ref().unwrap().incision_cap, 3.0);
            assert_eq!(p.surface.as_ref().unwrap().drama, 1.0);
        }
    }

    #[test]
    fn timeline_choices_apply() {
        let json = with_timeline(
            plates_builtin("plates_quick_16k").unwrap(),
            Some("scattered_continents"),
            Some(500.0),
        )
        .unwrap();
        let p = planet(&json);
        p.check_runnable().unwrap();
        assert_eq!(p.start().unwrap(), StartConfig::ScatteredContinents);
        assert_eq!(p.plate_steps(), 250);
        assert_eq!(p.t_start_years(), -5.0e8);
    }

    #[test]
    fn errors_name_the_key() {
        assert_eq!(path_of(r#"{"planet":{}}"#), "$.planet");
        let base = with_timeline(
            plates_builtin("plates_quick_16k").unwrap(),
            Some("random"),
            Some(100.0),
        )
        .unwrap();
        let edit = |f: &dyn Fn(&mut Value)| {
            let mut v: Value = serde_json::from_str(&base).unwrap();
            f(&mut v);
            path_of(&v.to_string())
        };
        assert_eq!(edit(&|v| v["tectonics"]["nope"] = json!(1)), "$.tectonics");
        // No ocean floor at all: 60 % land on 100 % continental crust (0.6/0.4 of the planet).
        assert_eq!(
            edit(&|v| {
                v["planet"]["ocean_fraction"] = json!(0.4);
                v["tectonics"]["continental_shelf_share"] = json!(0.6);
            }),
            "$.tectonics.continental_shelf_share"
        );
        assert_eq!(
            edit(&|v| v["planet"]["ocean_fraction"] = json!(1.5)),
            "$.planet.ocean_fraction"
        );
        assert_eq!(
            edit(&|v| v["resolution"]["l1_r"] = json!(7)),
            "$.resolution.l1_r"
        );
        assert_eq!(edit(&|v| v["scale"]["radius"] = json!(1)), "$.scale.radius");
        assert_eq!(
            edit(&|v| v["keyframes"]["every_myr"] = json!(0)),
            "$.keyframes.every_myr"
        );
        assert_eq!(
            edit(&|v| v["dummy"] = json!({"steps": 1, "sleep_ms": 0})),
            "$"
        );
    }

    /// The world's floor (Ben, decisions log 2026-10-02, "Lower min_y to -512"): every built-in
    /// preset's deepest trench, plate model's trench elevation at the vertical scale, stands above
    /// the floor and clear of the router's bottom density slide (24 blocks), and the floor is the
    /// default of an absent key. With the old floor (−352) the trench of −10 km (y −437) was cut.
    #[test]
    fn the_floor_holds_the_deepest_trench() {
        assert_eq!(d_min_y(), -512);
        assert_eq!((d_min_y() + 1152 - 1, d_max_y()), (639, 639), "height 1152 ends at the top 639");
        for (name, json) in crate::preset::BUILTIN.iter().chain(PIPELINE_BUILTIN.iter()) {
            let p = planet(json);
            assert_eq!(p.scale.min_y, -512, "{name}");
            let trench_y = f64::from(p.scale.sea_level_y)
                + p.tectonics.trench_elevation_m / p.scale.m_per_block_v;
            assert!(
                trench_y > f64::from(p.scale.min_y) + 24.0,
                "{name}: the trench at y {trench_y} is not clear of the floor {}",
                p.scale.min_y
            );
        }
    }

    /// The build-limit compression (2026-10-01): as simulated below the knee, increasing and
    /// smooth through it, never at the top however high the terrain; and a band too thin is refused.
    #[test]
    fn the_top_band_is_compressed() {
        let p = planet(PIPELINE_BUILTIN[0].1);
        let fit = p.scale.build_fit();
        assert_eq!((fit.knee_y, fit.top_y), (575.0, 623.0));
        assert_eq!(p.scale.uncompressed_height_m(), 10_240.0);
        for y in [-512.0, 63.0, 574.9, 575.0] {
            assert_eq!(fit.compress(y), y);
        }
        let mut last = 575.0;
        for k in 1..=4000 {
            let y = 575.0 + f64::from(k) * 0.25;
            let c = fit.compress(y);
            assert!(c > last && c < fit.top_y && c <= y, "{y} -> {c}");
            last = c;
        }
        // Slope 1 at the knee; 20 km of mountain (y 1063) still under the top.
        assert!((fit.compress(575.25) - 575.25).abs() < 1e-3);
        assert!(fit.compress(1063.0) < 623.0 && fit.compress(1063.0) > 622.9);
        assert!(fit.compress(f64::NAN).is_nan());

        let edit = |f: &dyn Fn(&mut Value)| {
            let mut v: Value = serde_json::from_str(PIPELINE_BUILTIN[0].1).unwrap();
            f(&mut v);
            match Preset::from_json(&v.to_string()) {
                Err(CoreError::PresetInvalid { path, reason }) => format!("{path}: {reason}"),
                other => panic!("expected a preset error, got {other:?}"),
            }
        };
        assert_eq!(
            edit(&|v| v["scale"]["build_limit_band_blocks"] = json!(2.0)),
            "$.scale.build_limit_band_blocks: must be at least 8"
        );
        // A world too short for its mountains is squeezed, not refused (the sliders' ends are valid).
        let mut v: Value = serde_json::from_str(PIPELINE_BUILTIN[0].1).unwrap();
        v["scale"]["m_per_block_v"] = json!(1.0);
        Preset::from_json(&v.to_string()).unwrap();
    }
}

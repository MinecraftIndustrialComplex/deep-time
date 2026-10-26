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
//!  "keyframes": {"every_myr": 50, "size": [1024, 326]},
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
pub const PLANET_KEYS: [&str; 18] = [
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
    "steering",
];

/// The `scale` keys a planet preset may carry (the header reads `circumference_blocks`).
pub const SCALE_KEYS: [&str; 10] = [
    "circumference_blocks",
    "m_per_block_h",
    "m_per_block_v",
    "min_y",
    "max_y",
    "sea_level_y",
    "feature_scale",
    "build_limit_band_blocks",
    "lambda_c_deg",
    "edge_border",
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
    /// The planet's surface water, its oceans and land ice as water, as a layer over the whole
    /// planet, m: the sea stands where this water, less what the land ice holds, fills the open
    /// sea of the ground (`sea_level.rs`). Earth's: 2,664 m (ledger ISO-085).
    #[serde(default = "d_surface_water_m")]
    pub surface_water_m: f64,
}

fn d_surface_water_m() -> f64 {
    // Earth's ocean, 1.332·10⁹ km³ (Charette & Smith 2010), and its land ice as water: Antarctica's
    // grounded 26.5·10⁶ km³ (Fretwell et al. 2013) and Greenland's 2.99·10⁶ km³ (Morlighem et al.
    // 2017) at ρ_i/ρ_w, and the glaciers' 0.41 m of sea level (IPCC AR5 Table 4.1), 2.65·10⁷ km³
    // in all; over Earth's 5.10·10⁸ km². ledger: ISO-085
    2664.0
}

fn d_m_per_block_h() -> f64 {
    300.0
}
fn d_m_per_block_v() -> f64 {
    deeptime_format::DEFAULT_M_PER_BLOCK_V // ledger: SCL-002
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
fn d_build_limit_band() -> f64 {
    48.0
}
fn d_lambda_c_deg() -> f64 {
    // −1: the bake's turn search turns the cube for the planet (`bake::cube_turn`).
    LAMBDA_C_AUTO
}

/// `scale.lambda_c_deg`'s "turn the cube for this planet" value.
pub const LAMBDA_C_AUTO: f64 = -1.0;

/// What the game does at the cube world's open edges until the wrap engine joins them
/// (`scale.edge_border`; FORMAT.md "The cube world's layout", `border`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EdgeBorder {
    /// Players, their vehicles and Sable ships stop a block inside the edge (the default).
    #[default]
    Stop,
    /// No border: nothing stops a player at an open edge, and nothing joins it.
    None,
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

    /// The simulated elevation of stored elevation `s`, blocks: the inverse of
    /// [`BuildFit::compress`] (at or past `top_y`, infinite).
    pub fn expand(&self, s: f64) -> f64 {
        let band = self.top_y - self.knee_y;
        if s > self.knee_y && band > 0.0 {
            let f = (s - self.knee_y) / band;
            if f >= 1.0 {
                f64::INFINITY
            } else {
                self.knee_y - band * (1.0 - f).ln()
            }
        } else {
            s
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
    /// How big the world's features are drawn in blocks, relative to their real size at the
    /// block scale (the editor's "feature scale" settings); 1 everywhere is the calibrated world.
    #[serde(default)]
    pub feature_scale: FeatureScale,
    /// The band under the build limit that terrain above its foot is compressed into, blocks
    /// ([`BuildFit`]): the foot (the knee) is at `max_y` − 16 − this. Terrain below the knee is
    /// as simulated.
    #[serde(default = "d_build_limit_band")]
    pub build_limit_band_blocks: f64,
    /// The cube's polar turn, λ0 in degrees in [0, 360) (`CubeTurn::polar`,
    /// `bake::cube_turn::pinned`), or −1 ([`LAMBDA_C_AUTO`], the default): the bake's turn search
    /// turns the cube so its corners lie as far from land as it finds. A planet is always the cube
    /// world (Ben, decisions log 2026-10-24: "Cube only"); the simulation keeps its own grid and
    /// the bake projects the world ("Bake only", 2026-10-03).
    #[serde(default = "d_lambda_c_deg")]
    pub lambda_c_deg: f64,
    /// The temporary border at the cube world's open edges.
    #[serde(default)]
    pub edge_border: EdgeBorder,
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

    /// The world's east–west period in blocks, what the world data's
    /// `scale.circumference_blocks` and Toroidal World's loop hold: the cube's loop, four faces
    /// of side `face_for(C)` (FORMAT.md "The cube world's layout", Scale).
    pub fn world_circumference(&self) -> u32 {
        4 * deeptime_format::projection::face_for(self.circumference_blocks)
    }
}

fn d_every_myr() -> f64 {
    50.0
}
fn d_keyframe_size() -> [u32; 2] {
    // The map's aspect (the Lambert projection, π : 1: `Projection::picture_height`).
    [1024, 326]
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

/// The ocean floor's height law (`tectonics.ocean_depth`; `plates::pools`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OceanDepth {
    /// Cortial et al. 2019 §4.5: every oceanic height relaxes towards
    /// `oceanic_floor_elevation_m` at `oceanic_damping_mm_yr` (ledger PLT-129).
    #[default]
    Dampening,
    /// Stein & Stein 1992's GDH1 depth–age law as the floor's subsidence: a sample sinks by what
    /// its age sinks bare floor under water. GDH1's shape is drawn between `ridge_elevation_m`
    /// and `oceanic_floor_elevation_m`, GDH1's own −2,600 and −5,651 m in every shipped preset,
    /// where the floor is GDH1 exactly (ledger PLT-352; the depths, PLT-355; the choice of law,
    /// PLT-350).
    Gdh1, // ledger: PLT-350, PLT-352
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
    /// z_r, highest oceanic ridge (−1 km; GDH1's −2,600 m in the shipped presets, ledger ISO-014).
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
    /// The speed guard, mm/yr: no point of a plate moves faster. Plate motion comes from the force
    /// balance (`plates::forces`); the guard is reported every time it binds (the dynamics record
    /// and `plates/guard_bound_plate_steps`) and should never bind (design
    /// `docs/design/plate-force-balance.md` §6; ledger PLT-022).
    pub max_plate_speed_mm_yr: f64,
    /// v_0 of Cortial et al. 2019 (100 mm/yr): the convergence at which the subduction uplift runs
    /// at its full rate (g(v) = min(1, v / v_0)). Until 2026-10-25 it scaled the arcs' magma too,
    /// which counted Earth's convergence twice (their rates are averages over Earth's arcs at
    /// Earth's convergences): the arcs now scale by `island_arc_convergence_ref_mm_yr` and
    /// `continental_arc_convergence_ref_mm_yr` (PLT-665). Until the switch to the force balance it
    /// was also the speed cap. Ledger PLT-037.
    pub convergence_ref_mm_yr: f64,
    /// ε_o (0.04 mm/yr), the dampening law's rate (ledger ISO-013); unused under
    /// `ocean_depth: gdh1`, the shipped presets' law, which carry no rate.
    pub oceanic_damping_mm_yr: f64,
    /// The elevation ageing ocean floor subsides towards, m. Under the dampening law (the default
    /// and the plate-only presets') z(a) = z_f + (z_r − z_f)·exp(−a·ε_o/|z_f|) towards Cortial's
    /// trench depth (−10 km), which leaves floor older than ~120 Myr deeper than any abyssal
    /// plain. Under `ocean_depth: gdh1` it is the depth GDH1's curve flattens towards: GDH1's own
    /// −5,651 m (Stein & Stein 1992, Nature 359:123–129) in every shipped preset (ledger ISO-012,
    /// PLT-355). The shipped presets ran the dampening at −5,600 m and ε_o 0.12 mm/yr until
    /// 2026-10-21, when the sea came to follow the ocean's volume (ISO-085).
    pub oceanic_floor_elevation_m: f64,
    /// The ocean floor's height law (`plates::pools`): Cortial's dampening towards
    /// `oceanic_floor_elevation_m` (the default and the plate-only presets'), or GDH1's age–depth
    /// law as a subsidence rate (every shipped Earth-like preset since 2026-10-21: the floor sinks
    /// by what its age sinks bare floor, whatever its sediment; ledger PLT-352).
    pub ocean_depth: OceanDepth,
    /// ε_c (0.03 mm/yr).
    pub continental_erosion_mm_yr: f64,
    /// ε_t (0.3 mm/yr).
    pub sediment_accretion_mm_yr: f64,
    /// u_0 (0.6 mm/yr).
    pub subduction_uplift_mm_yr: f64,
    /// The real shortening (`plates/shortening.rs`) at which a terrane smaller than
    /// `collision_suture_frac` moves to the overriding plate (Cortial §6: 300 km of
    /// interpenetration). Continents larger than that weld on stalled convergence instead
    /// (`weld_speed_mm_yr`, `weld_after_myr`), whatever their shortening.
    pub collision_threshold_km: f64,

    /// Where the subduction distance transfer f(d) peaks (design choice).
    pub subduction_peak_km: f64,
    /// Width of the collision belt on the overriding plate behind the front, km
    /// (Earth-normalized): the crust shortened each step is stacked there (`plates/belts.rs`),
    /// highest at the front. Crust spreading (`crust_strength_mpa`) widens it into a plateau.
    pub collision_belt_km: f64,
    /// Share of the continental crust shortened across a collision front that stays in the
    /// crust, thickening the belt and its wedge; the rest goes down with the slab or is lost
    /// from the lower crust (`plates/belts.rs`).
    pub collision_stacking_share: f64,
    /// `true` (the default and every built-in preset since 2026-10-22, the coordinator's call):
    /// crust this collision stacked on the incoming plate (its wedge, which that plate carries to
    /// the front and under it) crosses the front again whole, as the orogen's own rock; only the
    /// crust the incoming plate brought takes `collision_stacking_share`. `false`: every
    /// crossing takes the share again (ledger PLT-400).
    pub collision_restack_whole: bool,
    /// Surface slope of the orogenic wedge on the incoming plate, degrees: the wedge rises
    /// towards the front at this slope up to the belt's height there (a critical wedge keeps
    /// its shape as it grows; the Lesser Himalayan wedge's surface slope is 1–1.6°).
    pub wedge_slope_deg: f64,
    /// `true` (the default since 2026-10-25): the collision belt's surface from critical taper
    /// (`plates/belts.rs`, `fill_taper`). The front is the toe of the orogenic wedge (the plate
    /// boundary surfaces at the deformation front), and the belt's surface rises from it at
    /// `wedge_slope_deg` up to the strength height, where it widens into a plateau. Each step's
    /// stacked crust fills the ground under that surface nearest the front first; once the belt
    /// is full to `collision_belt_km` the toe advances (the surface rises in place). How much
    /// the belt keeps is the old fill's (the sliver rule and the strength height's passes): only
    /// where the thickening lands moves. `false`: the profile `1 − (d/W)²`, highest at the front
    /// (ledger PLT-243). ledger: PLT-600, PLT-601
    pub collision_belt_taper: bool,
    /// Two plates in collision weld into one when the convergence across their front has stayed
    /// below this, mm/yr …
    pub weld_speed_mm_yr: f64,
    /// … for this long, Myr. The larger plate keeps its id.
    pub weld_after_myr: f64,
    /// Polarity reversal (`plates/subduction.rs`, PLT-056): `true` judges the overriding side by
    /// the crust its arc stands on (its plate's continental crust within `arc_distance_km` of the
    /// overlap, PLT-370; the default and every built-in preset since 2026-10-23, V1 gap 2) rather
    /// than by its crust in the overlap (`false`, PLT-056's old judgement; ledger PLT-371).
    pub reversal_by_arc_crust: bool,
    /// One-way subduction (the front lane, 2026-10-24): ocean floor hidden under an overriding
    /// plate's forearc (a cover within `arc_distance_km`, PLT-042, of a subduction front as the
    /// last step's fronts left it, plus one lattice spacing) is the slab: it stays hidden until
    /// the next resampling consumes it, whatever that plate does next, and a point of the new
    /// lattice that only such crust covers is a gap and gets new ocean floor
    /// (`subduction::resolve`, `ridges::resample`). Continental crust is never held (it is
    /// buoyant and comes back up, PLT-061), nor is crust a collision brings up. Off, a sample is
    /// hidden exactly while it lies under an overriding plate's cover, so old crust that went
    /// under shows again where that plate moves off it (plates.md, "The overlaps' hiding"). On
    /// by default since 2026-10-24 (plates.md, "Step 6"). ledger: PLT-542
    pub one_way_subduction: bool,
    /// A subduction zone goes on through a resampling (the front lane, 2026-10-24;
    /// `Contact::carried`, `subduction::carry`): a contact active when the lattice is rebuilt
    /// stays active, with its fronts, arcs and slab, until its overlap is seen again, its
    /// boundary stops converging (no front) or `contact_timeout_myr` passes. The rebuild is a
    /// numerical step, not a geological event, but it gives every plate fresh crust with no
    /// overlap, and the overlap that makes a contact active takes a few steps to form again. Off,
    /// a contact is active only at steps its overlap is seen, so every resampling switches off
    /// the subduction zones for 2 to 4 Myr (the record's subducting length falls from 99,000 to
    /// 63,000 km in the first 2 Myr after one, the converging boundary with no contact rises
    /// from 6,000 to 37,000; plates.md, "Step 7"). ledger: PLT-550
    pub contact_carry: bool,
    /// A front stands on the current geometry (the front lane, 2026-10-25;
    /// `subduction::fronts_begin`, `dynamics::boundary_mix`): an overriding sample is a front
    /// through a lattice neighbour of a subducting plate only while a visible sample of that plate
    /// stands within `subduction::trench_row_rad` of it now (two lattice spacings: the step to the
    /// neighbour across the boundary and the trench's one hidden row). Between resamplings the
    /// lattice is frozen, so the neighbour may have gone under and been carried far on (1,015 km,
    /// the median, for the fronts off the current boundary on the front trace): the overriding
    /// samples behind the real front kept a front with no slab beside them, up to a fifth of the
    /// ocean-continent front late in a cycle, and the new lattice dropped them at every
    /// resampling (−29.7 %; with this rule −3.4 %; plates.md, "What the trace found"). Off, any
    /// lattice neighbour of the plate counts, hidden or far. On by default since 2026-10-25.
    /// ledger: PLT-582
    pub front_current_geometry: bool,
    /// A sample goes under another plate where it crosses that plate's outline (the front lane's
    /// step 8, 2026-10-24; `subduction::detect`, `level_set::cover`): the zero line of the
    /// interpolated distance the resampling already gives the plates' ground by
    /// (`level_set_membership`, PLT-536; with that off, the midpoint of every edge between two
    /// plates), so the overlap and the resampling read one outline. Off, a sample overlaps only
    /// inside a lattice triangle whose three corners are the other plate's, half a spacing inside
    /// its outline on average, and the subducting plate's edge row reaches in among the
    /// overriding plate's before it goes under (43 % of the fronts' trench samples have another
    /// plate's sample within half a spacing; plates.md, "Step 7"). ledger: PLT-553
    pub outline_cover: bool,
    /// Orogenic collapse (`plates/collapse.rs`): continental crust thicker than
    /// `collapse_thickness_km` that no orogeny is raising any more thins by spreading, with this
    /// e-folding time of its excess thickness, Myr; the crust it loses fills the nearest thin
    /// crust of its plate, and the plate's ocean floor beside the continent turns continental,
    /// so collision belts give back the area they took. 0 = off.
    pub collapse_myr: f64,
    /// … the crust thickness it thins back to, km …
    pub collapse_thickness_km: f64,
    /// … once no orogeny has raised it for this long, Myr …
    pub collapse_quiet_myr: f64,
    /// … and how far from the collapsing crust the crust it loses may go, km (Earth-normalized).
    /// 5,000 (the default and every built-in preset since 2026-10-23) is a plate's size, so the
    /// crust a collapse releases reaches its continent's edge (ledger PLT-257).
    pub collapse_reach_km: f64,
    /// Post-orogenic collapse: for this long after its last orogeny a quiet column's crust is
    /// still hot and weak, and thins back towards the normal continental column (35.1 km,
    /// `collapse::normal_thickness_m`) rather than only to `collapse_thickness_km`, Myr; older
    /// crust, or crust no orogeny raised, keeps the threshold. 0 = never (the default and every
    /// built-in preset: measured and not on, it costs land while the sea stands off the datum;
    /// ledger PLT-390).
    pub collapse_hot_myr: f64,
    /// Where the crust a collapse (or the thickest crust's spreading, `crust_max_km`) releases
    /// goes first: `true` the thin continental crust within
    /// reach, nearest first, and the ocean floor only after it; `false` (the default and every
    /// built-in preset) the nearest room of either kind (ledger PLT-391).
    pub collapse_continent_first: bool,
    /// The normal column the collapse measures from the sea: `true` the column a hot orogen
    /// thins towards (`collapse_hot_myr`) and the collapse fills its room up to is the one that
    /// stands as high over the step's sea S as the normal continental column (35.1 km) stands
    /// over the datum, T_n + 7.0 S (41.8 km under a sea 956 m over the datum: the old lowland's
    /// crust, standing about 500 m over the sea), within [T_n / 2, `collapse_thickness_km`];
    /// `true` is the default and every built-in preset's since 2026-10-23; `false` T_n in the
    /// datum frame (ledger PLT-412).
    pub collapse_sea_column: bool,
    /// … and never thicker than this, km: the continents' mean crust (Christensen & Mooney 1995:
    /// 41.1 km, shields and platforms about the same), so a sea that rises past about 850 m over
    /// the datum does not lift the column with it. 41.1 is the default and every built-in
    /// preset's since 2026-10-23; 0 = no bound but `collapse_thickness_km` (ledger PLT-413).
    pub collapse_sea_column_max_km: f64,
    /// The collapse as a thin viscous sheet (`plates/collapse.rs`, "As a thin viscous sheet"):
    /// with this depth-averaged viscosity of a young orogen's lithosphere, Pa s, a quiet column
    /// raised by an orogeny thins at the strain rate its potential energy over the collapse's
    /// normal column drives, ΔU / (4 η L) (L the lithosphere's 100 km), down to that column,
    /// where it outpaces the threshold law (Tibet and the Basin and Range: about 10²² Pa s,
    /// England & Molnar 1997, Flesch et al. 2000). 10²² is the default and every built-in
    /// preset's since 2026-10-23; 0 = off (ledger PLT-414).
    pub collapse_sheet_viscosity_pa_s: f64,
    /// … with the viscosity growing e-fold per this many Myr since the column's last orogeny,
    /// as its lithosphere cools (McKenzie 1978's thermal time, 62.8 Myr); crust no orogeny raised
    /// does not flow. 0 = no growth (ledger PLT-415).
    pub collapse_sheet_strength_myr: f64,
    /// The sheet reads the column's thermal state, not its orogeny (the front lane, 2026-10-25):
    /// with `thermal_strength`, crust no orogeny raised flows too, at the viscosity its
    /// lithosphere's thermal age gives, so a hot column the founder or the lower-crustal flow
    /// filled to T_c thins on towards the normal column, while a craton at L_cold (an infinite
    /// thermal age) stays rigid: the lower crust "escapes from beneath thickened, elevated central
    /// plateau through regions where crust is weak" (Clark & Royden 2000, p. 706). On by default
    /// since 2026-10-25. Off, or without `thermal_strength`, only crust an orogeny raised flows
    /// (PLT-415's gate). ledger: PLT-581
    pub collapse_sheet_thermal: bool,
    /// The lithosphere's thermal state (`plates/thermal.rs`, gap 4): each sample carries the depth
    /// of its thermal lithosphere and its depleted keel; arcs, back-arcs, hotspots and rifts heat
    /// it, elsewhere it cools on McKenzie's time; continental heat flow comes from it (with the
    /// final pass of `heat_flow_refresh`). On by default and in every built-in preset since
    /// 2026-10-23 (decisions log, "Always, fix V1 elsewhere"); off = the old proxy, kept for tests
    /// that isolate one law (ledger PLT-137, PLT-460).
    pub thermal_state: bool,
    /// Thermal isostasy, m: the height between a hot back-arc's lithosphere and a cold craton's at
    /// equal crust, added to a continental column's Airy height in proportion to its
    /// lithosphere's depth (`thermal_reference_km` stands at 0). 0 = off, the default: no frame
    /// for the reference depth is right yet, as the model's sea is not (an open fault,
    /// lithosphere-thermal-state.md); needs `thermal_state` (ledger ISO-110).
    pub thermal_isostasy_m: f64,
    /// The lithosphere's depth, km, at which the model's Airy line stands (the thermal height is 0
    /// there), and the depth the start's continents begin at (ledger ISO-111).
    pub thermal_reference_km: f64,
    /// Strength from the state: the thin sheet's viscosity and the flexure's continental Te take
    /// the thermal age the lithosphere's depth implies in place of the orogeny's age; does
    /// nothing without `thermal_state`. On by default since 2026-10-23; off = the orogeny's age,
    /// kept for tests that isolate one law (ledger ISO-112, ISO-051).
    pub thermal_strength: bool,
    /// The collision's underthrust lithosphere (gap 6): in an active continental collision the
    /// incoming plate's lithosphere goes on under the front, so under the footprint (the wedge,
    /// and the belt on the overriding plate as far behind the front as the collision has
    /// shortened) the lithosphere that bends is the incoming plate's. The
    /// flexure then takes the footprint as part of the incoming plate (the break, ISO-055, moves
    /// from the front to the belt's back edge) and the incoming lithosphere's elastic thickness
    /// there, from its foreland's thermal age (with the belt's own as a decoupled layer above
    /// it): the range is a load on the edge of a stiff plate, which bends into the foreland basin
    /// in front of it (Beaumont 1981; Jordan 1981; Lyon-Caen & Molnar 1983). Off, the belt is a
    /// load on the overriding plate across a break from the incoming one, and the incoming plate
    /// feels only its wedge. Needs `thermal_state` for the incoming lithosphere's age (without
    /// it only the break moves). Off (2026-10-24): on the pool it brought no foredeep beyond the
    /// wedge, whose width the foreland's flexural length hardly passes, and the foreland's frame
    /// (ISO-133) doubled the largest change of a step (flexure.md "Foreland basins"). Still off
    /// since gap 6b (2026-10-25): with the belt's part of the deflection carried in its frame
    /// (ISO-136, which replaced ISO-133's column swap) the foredeep came, 143 to 179 km wide, but
    /// a cell's largest change in a step rose by 3 to 7 km and the guard's worst by 2 to 5 km
    /// (flexure.md "The superposition and the foreland's stiffness (gap 6b)").
    /// ledger: PLT-546, ISO-131, ISO-136
    pub collision_underthrust: bool,
    /// The overriding plate takes part of a collision's convergence as distributed shortening of
    /// its interior (England & Houseman 1986's thin sheet): behind the belt, along the strip
    /// behind the front, each continental column shortens at ε̇ = F / (4 η L) under the belt's
    /// push F (its column energy over the strip's, FRC-018's edge term read across the belt;
    /// Molnar & Lyon-Caen 1988), with the collapse's sheet viscosity at its thermal age
    /// (η = η_0 e^(t_eq / τ_s), PLT-414, PLT-415, ISO-112), up to the whole convergence. That
    /// share of the crust the step keeps thickens the interior in proportion to T ε̇ instead of
    /// stacking at the front; nothing is created. A cold, keeled craton takes almost none, a hot
    /// plate most. Does nothing without `thermal_state` or a sheet viscosity. On since 2026-10-25 (arm
    /// fl-upper-unfreeze, the guard clean). ledger: PLT-652
    pub collision_upper_plate_shortening: bool,
    /// The cordillera's back-arc source from the state: the crust behind an arc's range gives
    /// its share of the range's shortening in proportion to its weakness, the thin sheet's
    /// η_0 / η at its thermal age, so a cold, keeled craton is a backstop that hardly thins and a
    /// hot back-arc gives the crust; needs `thermal_state`. Off by default: a stand-in for the
    /// force balance's own strength, not physics of its own (ledger PLT-470).
    pub thermal_backstop: bool,
    /// The depleted keel under the start's continents, km (at most `thermal_reference_km`); 0 =
    /// no keels (ledger PLT-463).
    pub thermal_keel_km: f64,
    /// After the step's resampling, every continental sample gets at least its heat-flow law's
    /// conductive value: crust the rate guard spread onto old sea floor kept the floor's (ledger
    /// PLT-469). On by default since 2026-10-23 (the coordinator's call: a bookkeeping fix that
    /// moves no elevation); on with `thermal_state` whatever this says.
    pub heat_flow_refresh: bool,
    /// The thickest continental crust, km: a column thicker than this loses the excess from its
    /// bottom to the mantle (continental subduction, a foundering root; `plates/collapse.rs`).
    /// 0 = no limit.
    pub crust_max_km: f64,
    /// Lower-crustal flow (`plates/flow.rs`): continental crust thicker than
    /// `collapse_thickness_km` spreads into the crust around it on its plate, by diffusion of
    /// thickness with this diffusivity for a 15 km channel, km²/Myr (the cube of the channel's
    /// thickness above `collapse_thickness_km` scales it). 0 = off.
    pub crust_channel_km2_myr: f64,
    /// A continental column's surface settles towards the height its crust holds up (its Airy
    /// height, under its sea water when below the sea, plus the surface's flexure) with this
    /// e-folding time, Myr (`plates/settle.rs`). 0 = off.
    pub isostasy_relax_myr: f64,
    /// The fastest a column's crust thickens, mm/yr (km/Myr): no process of a plate step, the
    /// resampling's accretion included, leaves a visible column thicker than it was when the
    /// step began plus this rate times the step; the excess goes to the nearest room on its
    /// plate, and what finds none goes down to the mantle (`plates/rate.rs`). 0 = no limit.
    pub thickening_max_mm_yr: f64,
    /// The fastest a column's surface rises, mm/yr (km/Myr): no process of a plate step, the
    /// resampling's accretion included, leaves a visible column higher than it stood when the
    /// step began plus this rate times the step (`plates/rate.rs`). 0 = no limit.
    pub uplift_max_mm_yr: f64,
    /// The ocean–continent transition of rifted margins (`plates/transition.rs`): continental
    /// crust tapers into the ocean floor of its own plate over this width, km **on the model
    /// sphere** (`model_radius_km`; Earth-normalized like the other plate lengths, so the
    /// planet's size does not enter and the plates see Earth's margin width whatever the planet's
    /// size; Ben, decisions log 2026-10-22, ledger PLT-262), centred on the boundary between the
    /// crusts; the crust the continent's edge loses thickens the ocean floor beside it. 0 = off …
    pub transition_width_km: f64,
    /// … with this e-folding time towards the taper, Myr (0 = off) …
    pub transition_myr: f64,
    /// … and what the ocean side is made of (PLT-294): `true` ocean crust (lavas and gabbro), a
    /// volcanic margin's, so no granite or gneiss reaches the sea floor; `false` the continent's
    /// own rock, kept under the oceanic label. On in every shipped preset since 2026-10-20 (the
    /// coordinator's call, decisions log 2026-10-20: with it the land held at Earth's 0.29 and the
    /// ocean basins' felsic share fell from 0.76 to 0.46); `false` here, the struct's default,
    /// keeps the test presets and any preset without the key on the continent's rock.
    pub transition_ocean_crust: bool,
    /// Plates at the start.
    pub initial_plates: u32,
    /// Spread of initial plate sizes (additive Voronoi weight, radians).
    pub plate_size_variety_rad: f64,
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
    /// Rifting: expected rifts per Myr of a plate with the average initial area and f(x_P) = 1.
    /// A step's λ_0 (Cortial §4.4: per step) is this times the step's length: 0.012 at the
    /// 2-Myr step. Ledger PLT-082.
    pub rift_per_myr: f64,
    /// f(x_P) = w + (1 − w)·x_P², x_P the plate's continental fraction: oceanic plates rift at w.
    pub rift_ocean_weight: f64,
    /// Fragments of one rift, inclusive (Cortial: 2–4).
    pub rift_fragments: [u32; 2],
    /// Fracture-line irregularity (radians of noise warp).
    pub rift_warp_rad: f64,
    /// Rifts under tension (V1 gap 2's stage C2, `plates::rifting::maybe_rift`): a plate's rift
    /// hazard is weighted by e^(σ/σ_s) over its mean across the plates that may rift, weighted by
    /// their unweighted hazards, σ the plate's mean in-plane stress from the force balance
    /// (FRC-022, tension positive) and σ_s this, MPa; the expected count is unchanged, the rifts
    /// move to the plates pulled apart. 0 leaves the hazard as Cortial's; 20 is the default
    /// (since 2026-10-23). game-choice (agent 2026-10-23): about half the spread of the plates' σ
    /// on the base's records. ledger: PLT-504
    pub rift_stress_mpa: f64,
    /// Rifts along weak lithosphere (V1 gap 2's stage C2, `plates::rifting::rift`): a rift's
    /// fragments grow from their centres through the plate, each lattice edge costing its length
    /// times (225 km / L)^m, L the lithosphere's depth (`thermal_state`'s), m this; the fragments
    /// meet, and the plate breaks, where the lithosphere is thin. 0 splits by great-circle
    /// distance (Cortial), as does any m without `thermal_state`. The default is 2 (since
    /// 2026-10-23): a brittle strength that grows linearly with depth (Byerlee's friction, from
    /// memory) integrates to a plate strength ∝ L², and an edge costs its strength's inverse.
    /// Audet & Bürgmann 2011:
    /// strain concentrates at pre-existing weak zones, and the weak directions follow the
    /// gradients of the effective elastic thickness. ledger: PLT-506
    pub rift_weakness: f64,
    /// Plates smaller than this fraction of the planet are absorbed by a neighbour.
    pub min_plate_area_frac: f64,
    /// No rifting beyond this many plates.
    pub max_plates: u32,
    /// Global resampling: at least this long apart, Myr (Cortial: 10–60 steps, 20–120 Myr at
    /// the 2-Myr step). Ledger PLT-098.
    pub resample_min_myr: f64,
    /// … at most this long apart, Myr.
    pub resample_max_myr: f64,
    /// … and when the fastest plate has moved this far since the last one.
    pub resample_drift_km: f64,
    /// Ridge segments (V1 gap 2's stage A, `plates::ridges::segment_ridges`): at a resampling the
    /// new floor between two plates is cut into bands of this width along the meridians of their
    /// relative rotation's pole, and each band's ridge is a segment normal to the motion, the steps
    /// between them transforms. Km, Earth-normalized; 0 keeps Cortial's ridge at the gap's
    /// midline. The default (since 2026-10-23) is Earth's first-order segment, 600 km (Aghaei et
    /// al. 2014 after Macdonald 1998: "600 ± 300 km"); the second-order ones (140 ± 90 km) are under
    /// the quick lattice's two rings. ledger: PLT-502
    pub ridge_segment_km: f64,
    /// Magmatic arc: distance from the front of its axis, km.
    pub arc_distance_km: f64,
    /// … and its half-width, km.
    pub arc_width_km: f64,
    /// The arcs' band stops at the slab's ends (the front lane's step 10, 2026-10-24;
    /// `subduction::fronts_finish`): a sample the arcs' distance field reaches makes arc magma only
    /// where it lies over its front's slab, between the ends of the front its nearest front sample
    /// stands on, measured along that front's line (each front sample covering its share of the
    /// straight front, PLT-530, centred on it). An arc stands over its slab where the slab has sunk
    /// to about 100 km, and past the slab's edge there is no slab to melt above. On by default
    /// (2026-10-24). Off, the band runs round each end of a front as the distance field does, a
    /// quarter circle of `arc_distance_km` on the overriding side at each: a front of length L gets
    /// about L + π · `arc_distance_km` of band (1.32 times the arcs' straight front on arm
    /// fl-outline, PLT-554). ledger: PLT-561
    pub arc_slab_ends: bool,
    /// The overriding margin's deformation stops at the slab's ends too (the front lane's step
    /// 11, 2026-10-24; `subduction::fronts_finish`, `subduction::erode_forearcs`): the subduction
    /// uplift, the cordillera (its range and the crust behind it that shortens into it) and the
    /// forearc (subduction erosion's band and the one-way subduction's hold) act only on the
    /// samples over their front's slab, by the arcs' test (`arc_slab_ends`). With no slab there is
    /// no forearc, no arc crust thickening and no slab-driven shortening. Off, they run round each
    /// end of a front on the whole distance field. ledger: PLT-562
    pub margin_slab_ends: bool,
    /// New crust an island arc (an arc on oceanic crust) adds per km of arc per Myr at Earth's
    /// mean orthogonal convergence of intra-oceanic arcs (`island_arc_convergence_ref_mm_yr`),
    /// km³ (Earth-normalized): magma intruded and erupted, spread across the arc band
    /// (`arc_width_km`). Stern & Scholl (2010) put the long-term average of the intra-oceanic
    /// arcs at about 100 (90–180 for Izu-Bonin-Mariana and the Aleutians; 1.5 km³ a year over
    /// 14,500 km of arc). The arc stands up by its thickness (`plates::subduction`). ledger: PLT-045
    pub island_arc_km3_per_km_myr: f64,
    /// … and a continental arc (an arc on continental crust, including an island arc grown
    /// continental), km³ per km per Myr at Earth's mean orthogonal convergence of continental
    /// arcs (`continental_arc_convergence_ref_mm_yr`): about 35 (Stern & Scholl 2010 after
    /// Haschke & Günther 2003; 1.0 km³ a year over 27,500 km). ledger: PLT-285
    pub continental_arc_km3_per_km_myr: f64,
    /// The orthogonal convergence at which an island arc adds `island_arc_km3_per_km_myr`, mm/yr:
    /// an arc's rate is that times v_n / this, v_n the convergence normal to its front, no cap
    /// (PLT-665). Stern & Scholl's 100 is a mean over Earth's intra-oceanic arcs, so it holds at
    /// their mean orthogonal convergence: 86.6 mm/yr, the length-weighted mean of Clift &
    /// Vannucchi 2004's orthogonal convergence (Tables 1 and 2) over the margins of Stern &
    /// Scholl's list of intra-oceanic arcs (11,450 km). ledger: PLT-666
    pub island_arc_convergence_ref_mm_yr: f64,
    /// … and a continental arc's, mm/yr: 58.6, the same mean over the other 24 margins of Clift &
    /// Vannucchi's tables (28,775 km; Stern & Scholl's continental margin arcs are 27,500 km).
    /// ledger: PLT-666
    pub continental_arc_convergence_ref_mm_yr: f64,
    /// Island-arc crust counts as continental once it is this thick, km: an oceanic sample turns
    /// continental once the arc crust it has built (`arc_oceanic_m`) on top of normal ocean crust
    /// (7 km) reaches it, whatever its height. Cloos (1993): crust thicker than about 15 km is
    /// too buoyant to subduct, and arcs built on ocean crust need more than about 20 Myr of
    /// activity to reach it. 0 turns every sample that has received arc magma continental at
    /// once. ledger: PLT-047
    pub arc_mature_crust_km: f64,
    /// Shortening across a continental arc above an oceanic slab, mm/yr, at any convergence: the
    /// arc's own cordillera thickens by the crust shortened into it, which the model takes from
    /// the plate's continental crust behind the range (volume kept; `plates::subduction`). About
    /// 1.7 over Earth's continental arcs, length-weighted (Heuret & Lallemand 2005, Table 1: 21
    /// of 88 two-degree segments shorten across the whole lithosphere, at about 7 mm/yr; the
    /// Bolivian Andes, the chain's largest, about 6). 0, the default and the plate-only presets'
    /// value, leaves only Cortial's subduction uplift. ledger: PLT-048, PLT-320
    pub cordillera_shortening_mm_yr: f64,
    /// … the half-width of its band, centred on the arc axis, km (Earth-normalized; the Andes
    /// and Cascades ranges are 100–300 km wide across the arc) …
    pub cordillera_width_km: f64,
    /// … and the height it builds towards, m: the shortening thickens the crust under the arc
    /// less as the column's Airy height nears this and not at all above it (thickened arc crust
    /// spreads; the Central Andes' plateau stands near 4 km, the Cascades' base 1.2–2 km).
    pub cordillera_height_m: f64,
    /// A trench sample closes half its distance to z_t in this time, Myr (half of it per step
    /// at the 2-Myr step). Ledger PLT-053.
    pub trench_half_life_myr: f64,
    /// The trench's release under `ocean_depth: gdh1`: visible ocean floor that is not at a
    /// trench this step and stands below the bare floor of its age (`pools::floor_at`), where a
    /// trench's pull left it, rises back towards that floor, closing half the distance in this
    /// time, Myr. GDH1's subsidence moves the floor by its age's increment only, so without it
    /// a trench's depth stays wherever the trench has been. 2 (PLT-053's half-life) is the
    /// default and every built-in preset's since 2026-10-23; 0 releases nothing (the model before
    /// the trench budget, 2026-10-22). ledger: PLT-443
    pub trench_release_half_life_myr: f64,
    /// Pelagic sedimentation on oceanic crust, m/Myr.
    pub pelagic_m_per_myr: f64,
    /// Sediment subduction: the share of subducted ocean floor's sediment (its sediment and
    /// sedimentary pools) that goes down with the slab at a resampling instead of accreting to
    /// the column on top (`ridges::resample`), on every overriding plate. 0.12 is the default
    /// and every built-in preset's since 2026-10-23; 0 accretes all of it (the model before the
    /// trench budget, 2026-10-22). ledger: PLT-440
    pub sediment_subduction_share: f64,
    /// Sediment subduction by margin type (the M5 lane, 2026-10-25): `true` replaces
    /// `sediment_subduction_share` with Clift & Vannucchi's (2004) share for the margin the
    /// floor goes under (`ridges::sediment_subducted_share`): 0.83 of the incoming pile where it
    /// is over 1 km and the convergence under 76 mm/yr (an accretionary margin), all of it
    /// elsewhere (an erosive margin; von Huene & Scholl 1991's type 2), whatever the overriding
    /// plate is made of. Off by default. ledger: PLT-660
    pub sediment_subduction_sourced: bool,
    /// The rifted margin's sediment stays continental (the M5 lane, 2026-10-25;
    /// `transition::margin_continental`): ocean floor on the ocean side of a margin's taper
    /// (within W/2, `transition_width_km`, of its plate's continental crust that is not itself
    /// hyper-extended), not at a trench, in a subduction zone or on an arc, that holds rock from
    /// the continents (sediment beyond the pelagic rain for its age, or felsic or metamorphic
    /// rock) is the distal margin's thinned continental crust (Brune et al. 2014) and becomes
    /// continental crust over its own ocean crust (PLT-331); and ocean floor that crust spread
    /// from continental columns onto (`collapse::spread`) becomes continental at once rather
    /// than when filled to the normal column. Off by default. ledger: PLT-661, PLT-662
    pub margin_sediment_continental: bool,
    /// Subduction erosion: the crust a front's overriding plate loses from below, km³ per km of
    /// front and Myr (Earth-normalized), where the front erodes (the two keys below); taken
    /// from the front sample's crust beyond a normal ocean floor's (`subduction::erode_forearcs`).
    /// 90 is the default and every built-in preset's since 2026-10-23; 0 erodes nothing.
    /// ledger: PLT-441
    pub subduction_erosion_km3_per_km_myr: f64,
    /// … where the convergence normal to the front is at least this, mm/yr …
    /// ledger: PLT-442
    pub subduction_erosion_min_mm_yr: f64,
    /// … and the incoming plate's sediment (its sediment and sedimentary pools at the front)
    /// is thinner than this, m. ledger: PLT-442
    pub subduction_erosion_max_sediment_m: f64,
    /// Subduction erosion's trench retreat (the docking lane's stage S1, 2026-10-23; Scholl & von
    /// Huene 2010): with subduction erosion on, an erosive margin (the two keys above) truncates
    /// its overriding plate, its trench migrating inboard at `subduction::RETREAT_KM_PER_MYR`:
    /// each front sample goes whole, its column to the mantle, at the chance that gives that
    /// retreat, and leaves its plate for the slab's (`subduction::erode_forearcs`). The volume is
    /// the retreat times the column; PLT-441's volume rate and its floor at an ocean's column are
    /// off while it is on. Off, the forearc thins and the front stays. ledger: PLT-523
    pub erosion_retreat: bool,
    /// The subduction fronts' length by each front sample's share of the straight front (the
    /// front lane, 2026-10-24): its lattice edges' Voronoi crossings (`shortening::crossing_sum`)
    /// on the boundary's normal (PLT-310), which along a front sum to its length with the
    /// lattice's zigzag taken out, as a collision front's length is measured (PLT-222). The
    /// per-km rates of the fronts (subduction erosion, PLT-441; the trench's retreat, PLT-522)
    /// and the fronts' km × Myr globals then meet their sources per km. On by default since
    /// 2026-10-24 (arm fl-share against base3: no guard failure, the plate check 210 passes
    /// against 207). Off, each front sample counts one lattice spacing, which counts a straight
    /// front about 1.04 times and the model's own fronts 1.32 times. ledger: PLT-530
    pub front_share: bool,
    /// A plate is one rigid body (the front lane, 2026-10-24; Gordon 1998: "The central
    /// assumption of plate tectonics is that the plates are rigid"; Bird 2003 [123]: PB2002's
    /// plates file "contains 52 segments", each "a closed curve outlining that plate", and
    /// [112]: each of its 150 boundaries runs from a junction to a junction; Stock & Lee 1994, a
    /// fragment broken off a plate "may lose its slab-pull force and become driven entirely by
    /// different conditions", and microplates that "became attached to the Pacific plate";
    /// Lonsdale 2005, the Farallon plate's fission): right after a resampling
    /// each plate's visible crust cut into pieces that meet only through other plates
    /// (`geometry::pieces`) keeps its largest piece; every other piece of `min_plate_area_frac`
    /// (PLT-091) or more becomes a plate of its own with the parent's rotation (fission; the force
    /// balance moves it from the next step), and every smaller piece joins the plate most of its
    /// lattice neighbours belong to (capture), as a plate below that size is absorbed
    /// (`ridges::bury`). Off, a plate's pieces keep one rotation however far apart they lie.
    /// ledger: PLT-532
    pub plate_fission: bool,
    /// A boundary belongs to the lithosphere on its two sides (the front lane, 2026-10-24;
    /// `plates/inherit.rs`): when a plate splits (a fission, a rift, a subduction initiation's
    /// new plate), each contact, slab and collision state it had goes to the pieces that border
    /// the other plate, every bordering piece keeping the contact and the slab (Stock & Lee 1994:
    /// "The fragmentation of a subducting plate creates a new triple junction, two limbs of which
    /// are along the trench") and the collision going with its front. Off, every such state stays
    /// keyed to the parent, wherever its lithosphere went. ledger: PLT-534
    pub split_inheritance: bool,
    /// A plate's outline lies between the lattice's samples (the front lane, 2026-10-24;
    /// `plates/level_set.rs`): every sample carries its distance from its plate's outline, a
    /// resampling gives a new point the plate whose corners' distances, weighed with its
    /// barycentric weights, outweigh the others' (the zero level set of the interpolated signed
    /// distance; Osher & Sethian 1988, Sussman, Smereka & Osher 1994), and the distances are set
    /// again from the new outline after it. A great-circle outline lands where it was on any
    /// lattice. Off, a new point takes the plate of the old corner with the largest weight, which
    /// moves an outline up to half a spacing at random at every resampling, so a boundary that
    /// does not move roughens (`plates::tests::how_resamplings_roughen_a_boundary`). ledger:
    /// PLT-536
    pub level_set_membership: bool,
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
    /// Oceanic crust older than this, on a plate that subducts nowhere, can start subducting on
    /// its own (Wilson cycle), Myr.
    pub subduction_initiation_age_myr: f64,
    /// Expected subduction initiations per Myr per A_0 of such old crust. Ledger PLT-063.
    pub subduction_initiation_per_myr: f64,
    /// Old crust can start subducting only this far from every trench, km (Earth-normalized;
    /// a new subduction zone does not open beside a working one).
    pub subduction_initiation_clearance_km: f64,
    /// The slab a subduction initiation plants on the old floor it splits off, km: the plate
    /// moves only by the force balance, and the planted slab pulls it towards its parent until
    /// the trench forms and the slab grows by the convergence (design §3.3). Subduction becomes
    /// self-sustaining once some 100-150 km of slab has gone down (ledger PLT-270, from memory).
    pub initiation_slab_km: f64,
    /// Incipient subduction (the front lane, 2026-10-25; Gurnis, Hall & Lavier 2004): a
    /// subduction front whose contact's slab is shorter than the critical convergence
    /// (`forces::INCIPIENT_SLAB_KM`, 100 km) is not yet self-sustaining. Its plate must push
    /// against the resistance of bending into the new trench (`forces::INCIPIENT_RESISTANCE_N_M`,
    /// 2.5 × 10¹² N/m; a ridge's floor `forces::INCIPIENT_RIDGE_N_M`, 5 × 10¹¹), it builds no arc
    /// (`subduction::fronts_finish`), and the boundary mix and the margins count it as a
    /// converging boundary, not a subduction zone (Bird 2003's oceanic convergent boundary: no
    /// Benioff zone, no arc). A subduction initiation's planted slab (`initiation_slab_km`, 150
    /// km) starts past it, and a contact's slab survives a resampling by its plates, so only a
    /// new contact of plates that come to converge starts incipient. Off, every front is a working
    /// subduction zone from its first step. ledger: PLT-610
    pub incipient_subduction: bool,
    /// The older side of a subduction initiation's fracture goes under (the floor-age lane,
    /// 2026-10-25; Gurnis, Hall & Lavier 2004 [60]: at a fracture zone in compression "the older
    /// plate is thrust beneath the younger", and [78]: with an asymmetry in age, "the weak zone
    /// overlies the thicker plate"): the mean age of the visible ocean floor on each side within
    /// one lattice step of the other side sets which plate the planted slab (PLT-270) hangs from,
    /// the split's or the parent's; a side whose fracture is mostly continental crust never goes
    /// under; a tie, the split. The contact the two plates form later takes the planted slab's
    /// polarity, not the rule of a new contact (the overlap's crust and ages,
    /// `subduction::resolve`). Off, the slab is always the split's and the contact follows the
    /// overlap, which put the parent under in 1,494 of 1,998 such contacts on base15 (build log
    /// 2026-10-25). ledger: PLT-655
    pub initiation_polarity_by_age: bool,
    /// Along-strike propagation of subduction onto a passive margin (the floor-age lane's law 2,
    /// 2026-10-25; Zhou, Li, Gerya & Stern 2020: "With increasing slab pull, the subduction zone
    /// propagates laterally"; Govers & Wortel 2005's STEP tear): where a plate's trench reaches
    /// within reach of the plate's own continental margin and its slab is past the critical
    /// convergence, the margin's ocean floor is cut off the continent along the margin, with a
    /// tear from the margin to the plate's nearest other boundary, when the slab's pull along
    /// the cut beats the margin's strength (`forces::margin_strength_n_m`, the fallback of
    /// PLT-658). The continent and the floor beyond the cut become a new plate K' that the
    /// trench's plate U keeps converging on; each further critical convergence (100 km) past the
    /// cut, under the same force test, moves the next stretch of the margin's floor from K' to U.
    /// The rule replaces PLT-064's clearance: with it on, a new subduction initiation keeps one
    /// lattice spacing from a front, not `subduction_initiation_clearance_km` (Crameri et al.
    /// 2020: most initiations start within 500 km of another subduction zone). Off by default
    /// (parked 2026-10-26, the coordinator's call): under these starts and the 100 km gate it
    /// never advances in the sim, so it does not do the sourced process; off, PLT-064's 1,000 km
    /// holds. ledger: PLT-656
    pub margin_propagation: bool,
    /// Law 3: the resistance an incipient subduction zone pushes against scales with the elastic
    /// thickness of the floor going under (Gurnis, Hall & Lavier 2004 [46]: the force to start
    /// underthrusting "increases by a factor of 2.2" from 7 Myr to 30 Myr floor, elastic thickness
    /// 8.5 km to 15 km, anchored on their Case 1 peak of 3 × 10¹² N/m for 30 Myr floor, [59]),
    /// with the model's elastic thickness law (ISO-048, ISO-049). It replaces PLT-612's single
    /// 2.5 × 10¹² N/m above the ridge floor (PLT-613, kept) and scales law 2's margin strength.
    /// Off, PLT-612 as before. Off by default: built inside law 2's stage and parked with it
    /// (2026-10-26). ledger: PLT-657
    pub incipient_resistance_by_thickness: bool,
    /// Subduction initiation off the passive margin (V1 gap 2's stage C1): the old floor within
    /// the reach of the arc the new trench builds (`arc_distance_km` + `arc_width_km`, and one
    /// lattice spacing for where on the lattice the front falls; great circle) of the plate's
    /// continental crust stays on the plate, and the fracture's seed is old floor beyond it. The
    /// old floor then goes down at an intra-oceanic trench, its arc stands on ocean floor (an
    /// island arc), and the margin stays passive (Stern 2004, Crameri et al. 2020: no Cenozoic
    /// passive margin turned convergent). On by default since 2026-10-23; off lets the fracture
    /// run anywhere through the old floor, along the continent's edge included. ledger: PLT-503
    pub subduction_initiation_offshore: bool,
    /// Subduction starts under compression (V1 gap 2's stage C2): the initiation hazard weighted
    /// as `rift_stress_mpa`'s with −σ, over the plates' old floor, σ_s this, MPa (Crameri et al.
    /// 2020: horizontally forced initiation dominates the last 100 Myr). 0 leaves it unweighted;
    /// 20 is the default (since 2026-10-23). game-choice (agent 2026-10-23), as
    /// `rift_stress_mpa`. ledger: PLT-505
    pub initiation_stress_mpa: f64,
    /// Transference (V1 item 1, the docking lane's stage T; Stern 2004): a continent in a
    /// collision whose slab has broken off, and on which its plate has gone on converging for the
    /// critical convergence since, moves alone to the overriding plate once the collision's force
    /// over its margin against its own plate's ocean floor reaches the force that starts a new
    /// subduction zone there (Gurnis et al. 2004). Off, such a continent waits for its collision
    /// to stall and then welds its whole plate (PLT-078, PLT-246). `collision::transference_due`.
    /// ledger: PLT-516
    pub transference: bool,
    /// Contacts unseen for longer than this are closed, Myr (12 steps at the 2-Myr step).
    /// Ledger PLT-059.
    pub contact_timeout_myr: f64,
    /// Duration recorded for a collision orogeny, Myr.
    pub collision_orogeny_myr: f64,
    /// Sample L1 every this many steps (0 = only for keyframes and at the end).
    pub l1_every_steps: u32,
    // The force balance (`plates::forces`, docs/design/plate-force-balance.md §3 and §6): every
    // plate's rotation relaxes each step towards the one at which the forces on it balance.
    /// The force balance's resistances are Forsyth & Uyeda's (1975, Table 6; the plate speed lane,
    /// 2026-10-25; `forces::Coeffs::new`): the slab resistance is the reference pull over their
    /// slab's terminal velocity (6.43 / 0.89 cm/yr), the basal drag and the continental drag their
    /// shares of it, the suction their share of the pull, and their transform and colliding
    /// resistances act as friction (a force whatever the speed, against the relative motion) on
    /// the boundaries their own 60 degree rule calls transforms (`forces::fu_transform`) and on
    /// converging ones, in place of the viscous boundary resistance all round. The speed level
    /// then follows from the forces, so `plate_speed_scale_mm_yr`, `slab_resistance`,
    /// `slab_suction_share`, `keel_drag`, `boundary_resistance` and `suction_saturates` are read
    /// only with this off (the 2026-10-02 calibration, FRC-016). Off by default: built and tested
    /// against the paper, but on the pool (arm sp-fu, 2026-10-25) the speed guard (PLT-022) binds
    /// in every run, on small overriding oceanic plates that their speed-independent suction draws
    /// with nothing but the basal drag to hold them. ledger: PLT-585
    pub forsyth_uyeda_balance: bool,
    /// The overall speed scale of the balance, mm/yr: a reference oceanic plate (the average
    /// starting plate's area, a 3,000 km trench of a full-length 80-Myr slab, its neighbours at
    /// rest) moves at this speed. Sets the basal drag coefficient without
    /// `forsyth_uyeda_balance`, and the starting plates' speeds (PLT-273). Ledger PLT-181.
    pub plate_speed_scale_mm_yr: f64,
    /// Slab pull, relative strength (1 = the reference pull, `forces::SLAB_PULL_N_M`). PLT-182.
    pub slab_pull: f64,
    /// Slab length at which a slab pulls in full (the upper mantle), km. PLT-184.
    pub slab_depth_km: f64,
    /// A slab breaks off after continental crust has blocked its trench this long, Myr. PLT-184.
    pub slab_breakoff_myr: f64,
    /// Resistance of the mantle to a slab's sinking, against the basal drag of the reference
    /// plate (per 3,000 km of trench). PLT-185.
    pub slab_resistance: f64,
    /// Slab suction on the overriding plate, as a share of the slab's pull. PLT-186.
    pub slab_suction_share: f64,
    /// The suction saturates (the front lane, 2026-10-24): it is a traction towards the flow the
    /// sinking slab drives, the share of the pull at rest falling to nothing as the overriding
    /// plate's speed towards the trench reaches the slab's terminal velocity (its pull over its
    /// slab resistance, PLT-185; Forsyth & Uyeda 1975), so no overriding plate is drawn faster
    /// than its slab sinks (`forces::Balance::add_suction`). On by default (2026-10-24). Off, the
    /// suction is the fixed share however fast the plate already moves, and a small plate with
    /// trenches along much of its edge is drawn on until the speed guard (PLT-022) holds it. Not
    /// read with `forsyth_uyeda_balance`, whose suction does not depend on speed (Forsyth & Uyeda
    /// 1975, pp. 170-171). ledger: PLT-540
    pub suction_saturates: bool,
    /// The force balance's subduction fronts are the arcs' (the front lane's step 9, 2026-10-24;
    /// `forces::balance_step`): a front sample is one whose plates converge across the boundary
    /// beyond Bird's 20 degrees of a transform (PLT-330's rule, `shortening::is_transform`), and
    /// the slab pull, the suction, the slab resistance and the collision resistance act at it
    /// along the boundary's normal (PLT-310) over its share of the straight front (PLT-530), with
    /// the slab fed by the convergence along that normal; a hanging slab pulls the same way.
    /// Their per-metre sizes (FRC-002, FRC-017, FRC-018) then meet a front's length as the drag's
    /// definition counts it (FRC-016: a straight trench, the pull normal to it). Off, every
    /// lattice edge converging on an active contact is a front (the contacts' strike-slip
    /// stretches too), each overriding sample one lattice spacing along that edge: 1.71 times the
    /// arcs' straight front on arm fl-outline, 1.21 times it along the normal (PLT-554).
    /// ledger: PLT-560
    pub front_pull: bool,
    /// Edge energy (ridge push and the weight of high ground), relative to its physical size:
    /// 0.1 is the gravitational potential energy as computed (a ridge's push is then about 0.1 of
    /// the reference slab pull). PLT-187.
    pub edge_energy: f64,
    /// The buoyancy of continental crust dragged down a collision front, relative to its physical
    /// size (1): continental crust is lighter than the mantle, so the deeper a collision drags it
    /// the harder it pushes back against the convergence (Cloos 1993). A term of the collision
    /// resistance beside the belt's weight (`plates::forces`). PLT-275.
    pub collision_buoyancy: f64,
    /// Basal drag under continental crust, as a multiple of that under oceanic crust. PLT-188.
    pub keel_drag: f64,
    /// Resistance along plate boundaries to relative motion, against the basal drag of the
    /// reference plate (per its perimeter). PLT-189.
    pub boundary_resistance: f64,
    /// Peak speed of the weak, slow mantle flow that drags plates with no slab, mm/yr (a degree-2
    /// stand-in for convection). PLT-190.
    pub mantle_flow_mm_yr: f64,
    /// Relaxation time of a plate's rotation towards the balance, Myr. PLT-191.
    pub motion_relax_myr: f64,
}

impl Default for TectonicParams {
    fn default() -> Self {
        TectonicParams {
            time_step_myr: 2.0,
            model_radius_km: 6370.0,
            ridge_elevation_m: -1000.0, // ledger: ISO-014
            abyssal_elevation_m: -6000.0,
            trench_elevation_m: -10000.0,
            max_elevation_m: 10000.0,
            uplift_reference_m: 10000.0,
            uplift_ceiling_m: 10000.0,
            crust_strength_mpa: 140.0,
            crust_flow_km2_myr: 0.0,
            subduction_distance_km: 1800.0,
            max_plate_speed_mm_yr: 250.0,        // ledger: PLT-022
            convergence_ref_mm_yr: 100.0,        // ledger: PLT-037
            oceanic_damping_mm_yr: 0.04,         // ledger: ISO-013
            oceanic_floor_elevation_m: -10000.0, // ledger: ISO-012
            ocean_depth: OceanDepth::Dampening,  // ledger: PLT-350
            continental_erosion_mm_yr: 0.03,
            sediment_accretion_mm_yr: 0.3,
            subduction_uplift_mm_yr: 0.6,
            collision_threshold_km: 100.0, // ledger: PLT-068
            subduction_peak_km: 300.0,
            collision_belt_km: 400.0,         // ledger: PLT-241
            collision_stacking_share: 0.95,   // ledger: PLT-242
            collision_restack_whole: true,    // ledger: PLT-400 (game-choice; on since 2026-10-22)
            wedge_slope_deg: 1.5,             // ledger: PLT-192
            collision_belt_taper: true,       // ledger: PLT-600, PLT-601 (on since 2026-10-25)
            weld_speed_mm_yr: 5.0,            // ledger: PLT-246
            weld_after_myr: 10.0,             // ledger: PLT-246
            reversal_by_arc_crust: true,      // ledger: PLT-371 (on since 2026-10-23)
            one_way_subduction: true,         // ledger: PLT-542 (on since 2026-10-24)
            contact_carry: true,              // ledger: PLT-550 (on since 2026-10-24)
            front_current_geometry: true,     // ledger: PLT-582 (on since 2026-10-25)
            outline_cover: true,              // ledger: PLT-553 (on since 2026-10-24)
            collapse_myr: 15.0,               // ledger: PLT-255
            collapse_thickness_km: 50.0,      // ledger: PLT-256
            collapse_quiet_myr: 10.0,         // ledger: PLT-259
            collapse_reach_km: 5000.0,        // ledger: PLT-257 (since 2026-10-23)
            collapse_hot_myr: 0.0,            // ledger: PLT-390
            collapse_continent_first: false,  // ledger: PLT-391
            collapse_sea_column: true,        // ledger: PLT-412 (on since 2026-10-23)
            collapse_sea_column_max_km: 41.1, // ledger: PLT-413
            crust_max_km: 70.0,               // ledger: PLT-258
            crust_channel_km2_myr: 1500.0,    // ledger: PLT-300
            isostasy_relax_myr: 0.004,        // ledger: ISO-078
            thickening_max_mm_yr: 5.0,        // ledger: PLT-287
            uplift_max_mm_yr: 3.5,            // ledger: PLT-288
            transition_width_km: 200.0,       // ledger: PLT-262 (km on the model sphere)
            transition_myr: 20.0,             // ledger: PLT-263
            transition_ocean_crust: false,    // ledger: PLT-294 (game-choice; presets set true)
            // ledger: PLT-414 (on since 2026-10-23)
            collapse_sheet_viscosity_pa_s: 1.0e22,
            // ledger: PLT-415
            collapse_sheet_strength_myr: 62.8,
            collapse_sheet_thermal: true, // ledger: PLT-581 (on since 2026-10-25)
            thermal_state: true,          // ledger: PLT-460 (on since 2026-10-23)
            thermal_isostasy_m: 0.0,      // ledger: ISO-110 (off: an open fault; 1,600 m when on)
            thermal_reference_km: 225.0,  // ledger: ISO-111
            thermal_strength: true,       // ledger: ISO-112 (on since 2026-10-23)
            collision_underthrust: false, // ledger: PLT-546, ISO-131, ISO-136 (off: step noise on the pool)
            collision_upper_plate_shortening: true, // ledger: PLT-652 (on since 2026-10-25)
            thermal_backstop: false,      // ledger: PLT-470 (off: a stand-in, not physics)
            thermal_keel_km: crate::plates::thermal::KEEL_KM, // ledger: PLT-463
            heat_flow_refresh: true, // ledger: PLT-469 (game-choice: on since 2026-10-23, the coordinator's call)
            initial_plates: 14,
            plate_size_variety_rad: 0.35,
            continent_elevation_m: [150.0, 900.0],
            continental_shelf_share: 0.0,
            scattered_continents: 6,
            coast_warp_rad: 0.25,
            rift_per_myr: 0.006, // ledger: PLT-082
            rift_ocean_weight: 0.08,
            rift_fragments: [2, 4],
            rift_warp_rad: 0.2,
            rift_stress_mpa: 20.0, // ledger: PLT-504 (game-choice; on since 2026-10-23)
            rift_weakness: 2.0,    // ledger: PLT-506 (on since 2026-10-23)
            min_plate_area_frac: 0.003,
            max_plates: 40,
            resample_min_myr: 20.0,  // ledger: PLT-098
            resample_max_myr: 120.0, // ledger: PLT-098
            resample_drift_km: 1500.0,
            ridge_segment_km: 600.0, // ledger: PLT-502 (on since 2026-10-23)
            arc_distance_km: 200.0,
            arc_width_km: 60.0,
            arc_slab_ends: true,    // ledger: PLT-561 (on since 2026-10-24)
            margin_slab_ends: true, // ledger: PLT-562 (on since 2026-10-24)
            island_arc_km3_per_km_myr: 100.0, // ledger: PLT-045
            continental_arc_km3_per_km_myr: 35.0, // ledger: PLT-285
            island_arc_convergence_ref_mm_yr: 86.6, // ledger: PLT-666
            continental_arc_convergence_ref_mm_yr: 58.6, // ledger: PLT-666
            arc_mature_crust_km: 15.0, // ledger: PLT-047
            cordillera_shortening_mm_yr: 0.0, // ledger: PLT-048
            cordillera_width_km: 120.0,
            cordillera_height_m: 2500.0,       // ledger: PLT-050
            trench_half_life_myr: 2.0,         // ledger: PLT-053
            trench_release_half_life_myr: 2.0, // ledger: PLT-443 (on since 2026-10-23)
            pelagic_m_per_myr: 5.0,
            sediment_subduction_share: 0.12,    // ledger: PLT-440
            sediment_subduction_sourced: false, // ledger: PLT-660
            margin_sediment_continental: false, // ledger: PLT-661, PLT-662
            subduction_erosion_km3_per_km_myr: 90.0, // ledger: PLT-441
            subduction_erosion_min_mm_yr: 60.0, // ledger: PLT-442
            subduction_erosion_max_sediment_m: 1000.0, // ledger: PLT-442
            erosion_retreat: false,             // ledger: PLT-523
            front_share: true,                  // ledger: PLT-530
            plate_fission: true,                // ledger: PLT-532
            split_inheritance: true,            // ledger: PLT-534
            level_set_membership: true,         // ledger: PLT-536
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
            subduction_initiation_age_myr: 150.0,
            subduction_initiation_per_myr: 0.15, // ledger: PLT-063
            subduction_initiation_clearance_km: 1000.0,
            initiation_slab_km: 150.0,                // ledger: PLT-270
            incipient_subduction: true,               // ledger: PLT-610 (on since 2026-10-25)
            initiation_polarity_by_age: true,         // ledger: PLT-655
            margin_propagation: false, // ledger: PLT-656 (parked 2026-10-26: no advance)
            incipient_resistance_by_thickness: false, // ledger: PLT-657 (parked with law 2)
            subduction_initiation_offshore: true, // ledger: PLT-503 (on since 2026-10-23)
            initiation_stress_mpa: 20.0, // ledger: PLT-505 (game-choice; on since 2026-10-23)
            transference: false,       // ledger: PLT-516
            contact_timeout_myr: 24.0, // ledger: PLT-059
            collision_orogeny_myr: 30.0,
            l1_every_steps: 0,
            forsyth_uyeda_balance: false, // ledger: PLT-585 (built 2026-10-25, off)
            plate_speed_scale_mm_yr: 45.0, // ledger: PLT-181
            slab_pull: 1.0,               // ledger: PLT-182
            slab_depth_km: 700.0,         // ledger: PLT-184
            slab_breakoff_myr: 10.0,      // ledger: PLT-184
            slab_resistance: 3.0,         // ledger: PLT-185
            slab_suction_share: 0.3,      // ledger: PLT-186
            suction_saturates: true,      // ledger: PLT-540
            front_pull: true,             // ledger: PLT-560 (on since 2026-10-24)
            edge_energy: 0.1,             // ledger: PLT-187
            collision_buoyancy: 1.0,      // ledger: PLT-275
            keel_drag: 3.0,               // ledger: PLT-188
            boundary_resistance: 3.0,     // ledger: PLT-189
            mantle_flow_mm_yr: 5.0,       // ledger: PLT-190
            motion_relax_myr: 6.0,        // ledger: PLT-191
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
    /// End-weighted nudges toward a playable end (`steering`; docs/design/
    /// playability-steering.md §7); `None` (and `enabled: false`) runs unsteered. It needs a
    /// `climate` section.
    pub steering: Option<crate::steering::SteeringParams>,
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
        let steering: Option<crate::steering::SteeringParams> = section(obj, "steering")?;
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
            steering,
            placement: None,
        };
        preset.validate()?;
        Ok(preset)
    }

    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        if let Some(s) = &self.steering {
            s.validate()?;
            check(
                !s.enabled || self.climate.is_some(),
                "$.steering.enabled",
                "steering acts through the climate stage: it needs a climate section",
            )?;
        }
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
        check(
            p.surface_water_m.is_finite() && (0.0..=20_000.0).contains(&p.surface_water_m),
            "$.planet.surface_water_m",
            "must be in [0, 20000] m",
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
            s.lambda_c_deg == LAMBDA_C_AUTO
                || (s.lambda_c_deg.is_finite() && (0.0..360.0).contains(&s.lambda_c_deg)),
            "$.scale.lambda_c_deg",
            "must be -1 (turn the cube for the planet) or in [0, 360)",
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
        pos(k.convergence_ref_mm_yr, "$.tectonics.convergence_ref_mm_yr")?;
        pos(
            k.island_arc_convergence_ref_mm_yr,
            "$.tectonics.island_arc_convergence_ref_mm_yr",
        )?;
        pos(
            k.continental_arc_convergence_ref_mm_yr,
            "$.tectonics.continental_arc_convergence_ref_mm_yr",
        )?;
        pos(
            k.subduction_distance_km,
            "$.tectonics.subduction_distance_km",
        )?;
        pos(k.collision_belt_km, "$.tectonics.collision_belt_km")?;
        check(
            k.wedge_slope_deg > 0.0 && k.wedge_slope_deg < 45.0,
            "$.tectonics.wedge_slope_deg",
            "must be in (0, 45)",
        )?;
        for (v, key) in [
            (k.weld_speed_mm_yr, "$.tectonics.weld_speed_mm_yr"),
            (k.weld_after_myr, "$.tectonics.weld_after_myr"),
            (k.collapse_myr, "$.tectonics.collapse_myr"),
            (k.collapse_thickness_km, "$.tectonics.collapse_thickness_km"),
            (k.collapse_quiet_myr, "$.tectonics.collapse_quiet_myr"),
            (k.collapse_reach_km, "$.tectonics.collapse_reach_km"),
            (k.collapse_hot_myr, "$.tectonics.collapse_hot_myr"),
            (
                k.collapse_sea_column_max_km,
                "$.tectonics.collapse_sea_column_max_km",
            ),
            (
                k.collapse_sheet_viscosity_pa_s,
                "$.tectonics.collapse_sheet_viscosity_pa_s",
            ),
            (
                k.collapse_sheet_strength_myr,
                "$.tectonics.collapse_sheet_strength_myr",
            ),
            (k.crust_max_km, "$.tectonics.crust_max_km"),
            (k.crust_channel_km2_myr, "$.tectonics.crust_channel_km2_myr"),
            (k.isostasy_relax_myr, "$.tectonics.isostasy_relax_myr"),
            (k.thickening_max_mm_yr, "$.tectonics.thickening_max_mm_yr"),
            (k.uplift_max_mm_yr, "$.tectonics.uplift_max_mm_yr"),
            (k.transition_width_km, "$.tectonics.transition_width_km"),
            (k.transition_myr, "$.tectonics.transition_myr"),
            (
                k.subduction_erosion_km3_per_km_myr,
                "$.tectonics.subduction_erosion_km3_per_km_myr",
            ),
            (
                k.subduction_erosion_min_mm_yr,
                "$.tectonics.subduction_erosion_min_mm_yr",
            ),
            (
                k.subduction_erosion_max_sediment_m,
                "$.tectonics.subduction_erosion_max_sediment_m",
            ),
            (
                k.trench_release_half_life_myr,
                "$.tectonics.trench_release_half_life_myr",
            ),
        ] {
            check(v.is_finite() && v >= 0.0, key, "must be ≥ 0")?;
        }
        pos(
            k.collision_threshold_km,
            "$.tectonics.collision_threshold_km",
        )?;
        // The lithosphere's thermal state (`plates/thermal.rs`).
        check(
            k.thermal_isostasy_m.is_finite() && k.thermal_isostasy_m >= 0.0,
            "$.tectonics.thermal_isostasy_m",
            "must be ≥ 0",
        )?;
        check(
            k.thermal_reference_km >= crate::plates::thermal::L_HOT_KM
                && k.thermal_reference_km <= crate::plates::thermal::L_COLD_KM,
            "$.tectonics.thermal_reference_km",
            "must be in [60, 225] (the hot and the cold lithosphere)",
        )?;
        check(
            k.thermal_keel_km.is_finite() && k.thermal_keel_km >= 0.0,
            "$.tectonics.thermal_keel_km",
            "must be ≥ 0",
        )?;
        // `thermal_strength` and `rift_weakness` (on by default) do nothing without the state, so
        // turning the state off alone is allowed; the two opt-in terms are refused without it.
        check(
            k.thermal_state || (k.thermal_isostasy_m == 0.0 && !k.thermal_backstop),
            "$.tectonics.thermal_state",
            "thermal_isostasy_m and thermal_backstop need thermal_state",
        )?;
        // PLT-652 (`collision_upper_plate_shortening`, on by default) reads the sheet's strength
        // at the lithosphere's thermal age: without the state or a sheet viscosity it does nothing
        // (belts.rs `upper_applies`), so turning either off alone is allowed.
        // The force balance.
        pos(
            k.plate_speed_scale_mm_yr,
            "$.tectonics.plate_speed_scale_mm_yr",
        )?;
        pos(k.slab_depth_km, "$.tectonics.slab_depth_km")?;
        pos(k.slab_breakoff_myr, "$.tectonics.slab_breakoff_myr")?;
        pos(k.keel_drag, "$.tectonics.keel_drag")?;
        pos(k.motion_relax_myr, "$.tectonics.motion_relax_myr")?;
        for (v, key) in [
            (k.slab_pull, "$.tectonics.slab_pull"),
            (k.slab_resistance, "$.tectonics.slab_resistance"),
            (k.slab_suction_share, "$.tectonics.slab_suction_share"),
            (k.edge_energy, "$.tectonics.edge_energy"),
            (k.collision_buoyancy, "$.tectonics.collision_buoyancy"),
            (k.boundary_resistance, "$.tectonics.boundary_resistance"),
            (k.mantle_flow_mm_yr, "$.tectonics.mantle_flow_mm_yr"),
            (k.initiation_slab_km, "$.tectonics.initiation_slab_km"),
        ] {
            check(v.is_finite() && v >= 0.0, key, "must be >= 0")?;
        }
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
            k.resample_min_myr > 0.0 && k.resample_min_myr <= k.resample_max_myr,
            "$.tectonics.resample_min_myr",
            "must be in (0, resample_max_myr]",
        )?;
        for (v, key) in [
            (k.trench_half_life_myr, "$.tectonics.trench_half_life_myr"),
            (k.contact_timeout_myr, "$.tectonics.contact_timeout_myr"),
        ] {
            check(v.is_finite() && v > 0.0, key, "must be positive")?;
        }
        for (v, key) in [
            (k.hotspot_lifetime_myr, "$.tectonics.hotspot_lifetime_myr"),
            (k.continent_elevation_m, "$.tectonics.continent_elevation_m"),
        ] {
            check(
                v[0].is_finite() && v[1].is_finite() && v[0] <= v[1],
                key,
                "[min, max] with min ≤ max",
            )?;
        }
        for (v, key) in [
            (
                k.collision_stacking_share,
                "$.tectonics.collision_stacking_share",
            ),
            (k.min_plate_area_frac, "$.tectonics.min_plate_area_frac"),
            (k.lip_probability, "$.tectonics.lip_probability"),
            (
                k.sediment_subduction_share,
                "$.tectonics.sediment_subduction_share",
            ),
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
            (k.ridge_segment_km, "$.tectonics.ridge_segment_km"),
            (k.rift_stress_mpa, "$.tectonics.rift_stress_mpa"),
            (k.rift_weakness, "$.tectonics.rift_weakness"),
            (k.initiation_stress_mpa, "$.tectonics.initiation_stress_mpa"),
            (
                k.cordillera_shortening_mm_yr,
                "$.tectonics.cordillera_shortening_mm_yr",
            ),
            (k.arc_mature_crust_km, "$.tectonics.arc_mature_crust_km"),
            (
                k.island_arc_km3_per_km_myr,
                "$.tectonics.island_arc_km3_per_km_myr",
            ),
            (
                k.continental_arc_km3_per_km_myr,
                "$.tectonics.continental_arc_km3_per_km_myr",
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
        if let Some(s) = &self.steering {
            v["steering"] = json!(s);
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

    /// Plate steps of the planned length: length / δt, rounded (δt is stretched to fit).
    pub fn planned_steps(&self) -> u64 {
        (self.length_myr() / self.tectonics.time_step_myr)
            .round()
            .max(1.0) as u64
    }

    /// The plate step, Myr: the planned length over its steps, whatever the stop.
    pub fn dt_myr(&self) -> f64 {
        self.length_myr() / self.planned_steps() as f64
    }

    /// Myr from the start to the stop (`timeline.stop_myr`, else the planned length; 0 before a
    /// timeline is chosen).
    pub fn run_myr(&self) -> f64 {
        self.timeline.map_or(0.0, |t| t.run_myr())
    }

    /// Plate steps of the run: the planned ones, or the stop in steps of [`Self::dt_myr`].
    pub fn plate_steps(&self) -> u64 {
        match self.timeline.and_then(|t| t.stop_myr) {
            None => self.planned_steps(),
            Some(stop) => (stop / self.dt_myr()).round().max(1.0) as u64,
        }
    }

    /// Years at the start of the history (negative).
    pub fn t_start_years(&self) -> f64 {
        -self.length_myr() * 1.0e6
    }

    /// World time at the end of plate step `k`, years (as the plates stage computes it).
    pub fn t_at_step(&self, k: u64) -> f64 {
        (self.t_start_years() / 1.0e6 + k as f64 * self.dt_myr()) * 1.0e6
    }

    /// World time at the run's end, years: 0 for a run to its planned length (the present), else
    /// the end of its last step (negative: stopped early; positive: continued).
    pub fn t_end_years(&self) -> f64 {
        let steps = self.plate_steps();
        if steps == self.planned_steps() {
            0.0
        } else {
            self.t_at_step(steps)
        }
    }

    /// Years from the start to the end ([`Self::t_end_years`] − [`Self::t_start_years`]).
    pub fn run_years(&self) -> f64 {
        self.t_end_years() - self.t_start_years()
    }

    /// Whether plate step `k` (1-based) of this run depends on where the run ends: the last step,
    /// a step inside the climate's ice-age window, or one the steering weighs (docs/design/
    /// playability-steering.md §2).
    pub fn step_depends_on_end(&self, k: u64) -> bool {
        let total = self.plate_steps();
        if k >= total {
            return true;
        }
        let t = self.t_at_step(k);
        let end = self.t_end_years();
        if let Some(c) = &self.climate {
            let mut c = c.clone();
            c.run_end_years = end;
            if crate::climate::clock::in_window(&c, t) {
                return true;
            }
        }
        self.climate.is_some()
            && self
                .steering
                .is_some_and(|s| s.weight(t, end, self.run_years()) > 0.0)
    }

    /// The last plate step whose world does not depend on where the run ends (0: none, the
    /// start). A checkpoint after it serves this stop and every later one (§3 of the design).
    pub fn last_free_step(&self) -> u64 {
        let mut k = self.plate_steps().saturating_sub(1);
        while k > 0 && self.step_depends_on_end(k) {
            k -= 1;
        }
        k
    }

    /// The same preset with the stop at `stop_myr` (None: the planned length).
    pub fn with_stop(&self, stop_myr: Option<f64>) -> PlanetPreset {
        let mut p = self.clone();
        if let Some(t) = p.timeline.as_mut() {
            t.stop_myr = stop_myr;
        }
        p
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

    /// The plate rules that were counted per step are rates and times (P1-19 #27): at the 2-Myr
    /// step they give exactly the old per-step values, so a 2-Myr run is unchanged.
    #[test]
    fn the_per_myr_rules_give_the_old_per_step_values_at_two_myr() {
        let k = TectonicParams::default();
        assert_eq!(k.time_step_myr, 2.0);
        assert_eq!(k.rift_per_myr * k.time_step_myr, 0.012);
        assert_eq!(k.subduction_initiation_per_myr * k.time_step_myr, 0.3);
        assert_eq!(k.resample_min_myr / k.time_step_myr, 10.0);
        assert_eq!(k.resample_max_myr / k.time_step_myr, 60.0);
        assert_eq!(k.contact_timeout_myr / k.time_step_myr, 12.0);
        assert_eq!(
            crate::plates::pools::trench_share(k.time_step_myr, k.trench_half_life_myr),
            0.5
        );
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
        assert_eq!(
            (d_min_y() + 1152 - 1, d_max_y()),
            (639, 639),
            "height 1152 ends at the top 639"
        );
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
        // The knee is fixed in blocks: (575 − 63) blocks at 35 m a block.
        assert_eq!(p.scale.uncompressed_height_m(), 17_920.0);
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

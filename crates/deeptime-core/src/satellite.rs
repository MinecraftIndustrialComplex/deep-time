// SPDX-License-Identifier: LGPL-3.0-or-later
//! The "Satellite" view layer (Ben, 2026-09-30: "Default view on the viewer is currently an
//! elevation map, can you turn it into an actual map. (I.E. Approximately what you would see on
//! google earth zoomed out all the way / satellite imagery.) So I can see where glaciars / snow
//! is"): a true-colour picture of the planet, composited when a keyframe is drawn, and the first
//! layer of every frame that has it, so it is the default view of the workbench, the viewer
//! bundles and the in-game timelapse.
//!
//! **View only**, like `provinces`: it is composited from fields the stages already wrote into the
//! frame's RGBA image; nothing is written to `WorldState`, so a run's world files are unchanged.
//! Keyframe layers are RGBA already (`FORMAT.md` "Keyframes"), so the frame needs no new layer
//! kind and the game's timelapse shows it as it is.
//!
//! # Colour rules
//! The anchor colours are the classes of the `satellite` ramp in `ramps.json` ([`CLASSES`] are its
//! `labels`), so the legend is a key to exactly the colours drawn. Per pixel, from the nearest
//! cell of each field (the elevation averaged over the cells a pixel covers when its grid is finer);
//! the climate's fields are smoothed first over a few cells (`box_filter`: temperatures over ±3
//! rows, moisture over 3 × 3 cells, sea ice over 5 × 17 sea cells, the herb and tree cover and the
//! rock colour over 3 × 3 land cells), so the energy-balance model's latitude bands, the orographic
//! rain's cell streaks and the Köppen borders do not show as stripes and steps:
//!
//! - **Sea** (the elevation below sea level): coloured by depth, turquoise over the shallowest
//!   8 m, the shelf colour at 120 m, ocean blue at 1.5 km, deep navy from 5 km; annual sea-ice
//!   cover (`climate/sea_ice`) whitens it (fully from about 90 %).
//! - **Land with a climate** (`climate/t_mean_c` and `climate/aridity` or `climate/precip_mm`): a
//!   continuous land cover. Herb cover grows with the aridity index P/PET (none below 0.05, full
//!   from 0.45) and tree cover from 0.5 to 1.0; herbs need a warmest month (`climate/t_warm_c`) over
//!   about 0 °C and trees over about 10 °C (the treeline). The Köppen class (`climate/koppen`) keeps
//!   the picture on its map: BW desert holds at most 15 % herbs and no trees, BS steppe 35–75 %
//!   herbs and under 8 % trees, Af/Am full forest, Aw/As savanna 20–55 % trees, ET and EF no trees
//!   (EF at most 20 % herbs).
//!   Bare ground is desert sand where it is hot and grey-tan where it is cold, tinted by the
//!   surface rock (`surface/class`: sandstone and sand pale, red beds and laterite reddish, basalt
//!   dark, evaporites salt-white). Herbs are khaki steppe in drier land, olive savanna in wetter
//!   land, brown-grey tundra where the warmest month is under about 10 °C; forest is boreal
//!   (coldest month under −10 °C), temperate, or rainforest (coldest month over about 18 °C).
//! - **Mountains**: the ground turns to bare grey-brown rock from 2.5 km up (60 % at 5.5 km).
//! - **Snow**: perennial where the warmest month is below freezing or the ground stands above the
//!   snow line (`climate/ela_m`), 85 % snow colour; seasonal on cold land, a lighter tint growing
//!   with the coldest month from −2 to −22 °C (up to 35 %, half that under forest).
//! - **Glacier ice** (`climate/ice_thickness_m` of 1 m or more, else `climate/glacier`): white,
//!   85 % at the thinnest, 100 % from 200 m.
//! - **Land without a climate** (plate-only frames, presets without the climate stage): bare ground
//!   by geology, since nothing says where plants or ice would be: the surface rock's colour where
//!   it is known (`surface/class`, else `structure/rock`), else desert tan in the lowlands turning to
//!   grey-brown rock with height, darker on hotspots; no vegetation, snow or ice.
//! - **Relief**: a hill shade from the elevation (light from the north-west at 45°, heights
//!   exaggerated 14×, slopes in the model sphere's metres), applied softly: 60 % of its effect on
//!   land, 45 % on ice, 35 % on the sea floor (none under sea ice).
//!
//! This module draws the ground only. The rivers and lakes of the terrain are drawn over its
//! picture, after the shading, by `keyframes::render` (`crate::water_view`).

use deeptime_format::{Ramp, Ramps};

use crate::keyframes::ViewScale;
use crate::world::{Field, WorldState};

/// The keyframe layer's name.
pub const LAYER: &str = "satellite";
/// The ramp whose classes are the anchor colours and the legend.
pub const RAMP: &str = "satellite";

/// The legend classes, in the ramp's order (its `labels`).
pub const CLASSES: [&str; 18] = [
    "Tropical rainforest",
    "Temperate forest",
    "Boreal forest",
    "Savanna and shrubland",
    "Grassland and steppe",
    "Hot desert",
    "Cold desert",
    "Tundra",
    "Bare rock and high mountains",
    "Volcanic rock",
    "Salt flat",
    "Snow",
    "Glacier ice",
    "Sea ice",
    "Shallow sea",
    "Continental shelf",
    "Ocean",
    "Deep ocean",
];

/// Class indices into [`CLASSES`] and the ramp.
pub mod class {
    pub const RAINFOREST: usize = 0;
    pub const TEMPERATE: usize = 1;
    pub const BOREAL: usize = 2;
    pub const SAVANNA: usize = 3;
    pub const STEPPE: usize = 4;
    pub const HOT_DESERT: usize = 5;
    pub const COLD_DESERT: usize = 6;
    pub const TUNDRA: usize = 7;
    pub const BARE_ROCK: usize = 8;
    pub const VOLCANIC: usize = 9;
    pub const SALT: usize = 10;
    pub const SNOW: usize = 11;
    pub const GLACIER: usize = 12;
    pub const SEA_ICE: usize = 13;
    pub const SHALLOW: usize = 14;
    pub const SHELF: usize = 15;
    pub const OCEAN: usize = 16;
    pub const DEEP: usize = 17;
}
use class::*;

/// The elevation fields (block y) the picture is drawn on, finest first: the amplified terrain,
/// the young landscape, then the deep-time surface (or the plate model's).
pub const ELEVATION_FIELDS: [&str; 3] = ["amplify/elevation", "young/elevation", "elevation"];

/// The fields read besides the elevation.
pub mod input {
    pub const T_MEAN: &str = crate::climate::field::T_MEAN;
    pub const T_WARM: &str = crate::climate::field::T_WARM;
    pub const T_COLD: &str = crate::climate::field::T_COLD;
    pub const PRECIP: &str = crate::climate::field::PRECIP;
    pub const ARIDITY: &str = crate::climate::field::ARIDITY;
    pub const KOPPEN: &str = crate::climate::field::KOPPEN;
    pub const ICE: &str = crate::climate::field::ICE_THICKNESS;
    pub const GLACIER: &str = crate::climate::field::GLACIER;
    pub const SEA_ICE: &str = crate::climate::field::SEA_ICE;
    pub const ELA: &str = crate::climate::field::ELA;
    pub const SURFACE_ROCK: &str = crate::surface::field::CLASS;
    pub const STRUCTURE_ROCK: &str = "structure/rock";
    pub const REGIME: &str = "regime";
}

/// The look's constants (the module docs give them in words).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// Vertical exaggeration of the hill shade.
    pub exaggeration: f32,
    /// Light azimuth (clockwise from north) and altitude, degrees.
    pub light_azimuth_deg: f32,
    pub light_altitude_deg: f32,
    /// Share of the hill shade's effect on land, ice and the sea floor.
    pub shade_land: f32,
    pub shade_ice: f32,
    pub shade_sea: f32,
    /// Depths (m) of the shallow, shelf, ocean and deep colours, the first held from 0.
    pub depth_m: [f32; 4],
    /// Herb cover: 0 at this aridity index, full at the second; tree cover likewise.
    pub herb_ai: (f32, f32),
    pub tree_ai: (f32, f32),
    /// Warmest month (°C) over which herbs and trees grow in full: (none at, full at).
    pub herb_t_warm: (f32, f32),
    pub tree_t_warm: (f32, f32),
    /// Share of the surface rock's own colour in bare ground.
    pub rock_tint: f32,
    /// Bare rock on high ground: none at the first height (m), `alpine_rock` at the second.
    pub alpine_m: (f32, f32),
    pub alpine_rock: f32,
    /// Snow colour share of perennial snow, and the most seasonal snow gets.
    pub perennial_snow: f32,
    pub seasonal_snow: f32,
    /// Coldest month (°C) over which seasonal snow grows: (none at, full at).
    pub seasonal_t_cold: (f32, f32),
    /// Glacier ice share at 1 m of ice, full at `ice_full_m`.
    pub ice_min_share: f32,
    pub ice_full_m: f32,
}

impl Default for Look {
    fn default() -> Look {
        Look {
            exaggeration: 14.0,
            light_azimuth_deg: 315.0,
            light_altitude_deg: 45.0,
            shade_land: 0.6,
            shade_ice: 0.45,
            shade_sea: 0.35,
            depth_m: [8.0, 120.0, 1500.0, 5000.0],
            herb_ai: (0.05, 0.45),
            tree_ai: (0.5, 1.0),
            herb_t_warm: (-1.0, 8.0),
            tree_t_warm: (8.5, 12.5),
            rock_tint: 0.45,
            alpine_m: (2500.0, 5500.0),
            alpine_rock: 0.6,
            perennial_snow: 0.85,
            seasonal_snow: 0.35,
            seasonal_t_cold: (-2.0, -22.0),
            ice_min_share: 0.85,
            ice_full_m: 200.0,
        }
    }
}

/// The anchor colours, RGB 0–255.
#[derive(Clone, Debug, PartialEq)]
pub struct Palette(pub [[f32; 3]; CLASSES.len()]);

impl Palette {
    /// Class k's colour is the ramp's colour at k (a categorical ramp holds [k, c], [k + 0.999, c]).
    pub fn from_ramp(ramp: &Ramp) -> Palette {
        Palette(std::array::from_fn(|k| {
            let c = ramp.colour(k as f64);
            [f32::from(c[0]), f32::from(c[1]), f32::from(c[2])]
        }))
    }

    /// The built-in ramps' palette.
    pub fn builtin() -> Palette {
        let r = Ramps::builtin();
        Palette::from_ramp(
            r.ramps
                .get(RAMP)
                .expect("ramps.json has the satellite ramp"),
        )
    }

    fn get(&self, k: usize) -> [f32; 3] {
        self.0[k]
    }
}

/// What the compositor reads at one pixel. `NaN` = not known.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    /// Height above the sea, m (negative: the sea floor's depth).
    pub z_m: f32,
    pub t_mean_c: f32,
    pub t_warm_c: f32,
    pub t_cold_c: f32,
    pub precip_mm: f32,
    /// P / PET.
    pub aridity: f32,
    /// A Köppen class index (`climate::koppen::CODES`).
    pub koppen: f32,
    /// Land ice, m.
    pub ice_m: f32,
    /// 1 where land ice stands.
    pub glacier: f32,
    /// Annual-mean sea-ice cover, 0–1.
    pub sea_ice: f32,
    /// The snow line above sea level, m.
    pub ela_m: f32,
    /// The surface rock class (`synth::EVERY_CLASS` index).
    pub rock: f32,
    /// A plate-model regime code.
    pub regime: f32,
    /// Herb and tree cover, 0–1, when the caller derived them already (`render` derives them per
    /// cell and smooths them); `NaN`: derived here from the climate ([`cover`]).
    pub herb_cover: f32,
    pub tree_cover: f32,
    /// The bare ground's rock colour when the caller derived it (`render` smooths it over the
    /// neighbouring cells); `NaN`: from `rock`.
    pub rock_rgb: [f32; 3],
}

impl Cell {
    /// A cell at height `z_m` with nothing else known.
    pub fn at(z_m: f32) -> Cell {
        Cell {
            z_m,
            t_mean_c: f32::NAN,
            t_warm_c: f32::NAN,
            t_cold_c: f32::NAN,
            precip_mm: f32::NAN,
            aridity: f32::NAN,
            koppen: f32::NAN,
            ice_m: f32::NAN,
            glacier: f32::NAN,
            sea_ice: f32::NAN,
            ela_m: f32::NAN,
            rock: f32::NAN,
            regime: f32::NAN,
            herb_cover: f32::NAN,
            tree_cover: f32::NAN,
            rock_rgb: [f32::NAN; 3],
        }
    }

    /// Whether the cell has a climate (a temperature and an aridity or precipitation).
    pub fn has_climate(&self) -> bool {
        self.t_mean_c.is_finite() && (self.aridity.is_finite() || self.precip_mm.is_finite())
    }
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    if !x.is_finite() {
        return 0.0;
    }
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    std::array::from_fn(|k| a[k] + (b[k] - a[k]) * t)
}

/// The colour bare ground of rock class `k` shows from orbit (`None`: not known).
pub fn rock_tint(k: f32, pal: &Palette) -> Option<[f32; 3]> {
    if !k.is_finite() || k < 0.0 {
        return None;
    }
    let name = crate::synth::EVERY_CLASS.get(k as usize)?;
    Some(match *name {
        "granite" | "granodiorite" | "rhyolite" | "pumice" | "pegmatite" | "greisen"
        | "anorthosite" => [178.0, 160.0, 142.0],
        "diorite" | "andesite" | "tuff" | "volcanic_ash" => [140.0, 132.0, 122.0],
        "gabbro" | "basalt" | "scoria" | "obsidian" | "peridotite" | "kimberlite"
        | "serpentinite" | "greenstone" | "amphibolite" | "eclogite" | "blueschist"
        | "impact_melt" => pal.get(VOLCANIC),
        "conglomerate" | "sandstone" | "gravel" | "sand" | "quartzite" | "loess" => {
            [214.0, 186.0, 136.0]
        }
        "red_bed" | "laterite" | "bauxite" | "ironstone" | "banded_iron_formation" => {
            [178.0, 108.0, 66.0]
        }
        "limestone" | "dolostone" | "chalk" | "chert" | "travertine" | "marble" | "carbonatite"
        | "phosphorite" => [214.0, 204.0, 180.0],
        "halite" | "gypsum" | "anhydrite" => pal.get(SALT),
        "schist" | "gneiss" | "hornfels" | "skarn" | "diamictite" | "impact_breccia" => {
            [150.0, 138.0, 120.0]
        }
        // Mudrocks, slate, soils, till, coal.
        _ => [140.0, 124.0, 102.0],
    })
}

/// The sea's colour at depth `d` metres.
pub fn depth_colour(d: f32, pal: &Palette, look: &Look) -> [f32; 3] {
    let [d0, d1, d2, d3] = look.depth_m;
    let stops = [
        (d0, pal.get(SHALLOW)),
        (d1, pal.get(SHELF)),
        (d2, pal.get(OCEAN)),
        (d3, pal.get(DEEP)),
    ];
    if d <= stops[0].0 {
        return stops[0].1;
    }
    for w in stops.windows(2) {
        let ((da, ca), (db, cb)) = (w[0], w[1]);
        if d <= db {
            return mix(ca, cb, (d - da) / (db - da));
        }
    }
    stops[3].1
}

/// A pixel's colour before the hill shade, and the share of the hill shade it takes.
pub fn colour(c: &Cell, pal: &Palette, look: &Look) -> ([f32; 3], f32) {
    if c.z_m < 0.0 {
        let mut rgb = depth_colour(-c.z_m, pal, look);
        let ice = if c.sea_ice.is_finite() {
            0.97 * smoothstep(0.08, 0.9, c.sea_ice)
        } else {
            0.0
        };
        rgb = mix(rgb, pal.get(SEA_ICE), ice);
        return (rgb, look.shade_sea * (1.0 - ice));
    }
    let climate = c.has_climate();
    let (mut rgb, forest) = if climate {
        vegetated(c, pal, look)
    } else {
        (bare(c, pal), 0.0)
    };
    rgb = mix(
        rgb,
        pal.get(BARE_ROCK),
        look.alpine_rock * smoothstep(look.alpine_m.0, look.alpine_m.1, c.z_m),
    );
    if climate {
        let tw = warmest(c);
        let tc = coldest(c);
        let above_snowline = if c.ela_m.is_finite() {
            smoothstep(0.0, 400.0, c.z_m - c.ela_m)
        } else {
            0.0
        };
        let mut perennial = smoothstep(1.0, -3.0, tw).max(above_snowline);
        if koppen(c) == Some(30) {
            perennial = 1.0;
        }
        let seasonal = look.seasonal_snow
            * smoothstep(look.seasonal_t_cold.0, look.seasonal_t_cold.1, tc)
            * (1.0 - 0.5 * forest);
        rgb = mix(
            rgb,
            pal.get(SNOW),
            (perennial * look.perennial_snow).max(seasonal),
        );
    }
    let ice = if c.ice_m.is_finite() {
        if c.ice_m >= crate::climate::ICE_MIN_M {
            look.ice_min_share
                + (1.0 - look.ice_min_share) * smoothstep(1.0, look.ice_full_m, c.ice_m)
        } else {
            0.0
        }
    } else if c.glacier >= 0.5 {
        1.0
    } else {
        0.0
    };
    rgb = mix(rgb, pal.get(GLACIER), ice);
    let shade = look.shade_land + (look.shade_ice - look.shade_land) * ice;
    (rgb, shade)
}

fn warmest(c: &Cell) -> f32 {
    if c.t_warm_c.is_finite() {
        c.t_warm_c
    } else {
        c.t_mean_c + 10.0
    }
}

fn coldest(c: &Cell) -> f32 {
    if c.t_cold_c.is_finite() {
        c.t_cold_c
    } else {
        c.t_mean_c - 10.0
    }
}

fn koppen(c: &Cell) -> Option<u8> {
    (c.koppen.is_finite() && c.koppen >= 0.0).then(|| c.koppen.round() as u8)
}

/// P / PET; without it, Köppen's dryness threshold (20·T + 140 mm) gives a stand-in whose desert
/// and steppe limits (half and all of it) fall near UNEP's 0.2 and 0.5.
fn aridity_index(c: &Cell) -> f32 {
    if c.aridity.is_finite() {
        c.aridity
    } else {
        0.42 * c.precip_mm / (20.0 * c.t_mean_c.max(0.0) + 140.0)
    }
}

/// Herb and tree cover (0–1) the climate of `c` supports, the Köppen class holding it on its map.
pub fn cover(c: &Cell, look: &Look) -> (f32, f32) {
    let tw = warmest(c);
    let ai = aridity_index(c);
    let mut herbs = smoothstep(look.herb_ai.0, look.herb_ai.1, ai);
    let mut trees = smoothstep(look.tree_ai.0, look.tree_ai.1, ai);
    let mut grow = smoothstep(look.herb_t_warm.0, look.herb_t_warm.1, tw);
    let mut treeline = smoothstep(look.tree_t_warm.0, look.tree_t_warm.1, tw);
    match koppen(c) {
        Some(0 | 1) => {
            herbs = 1.0;
            trees = trees.max(0.92);
        }
        Some(2 | 3) => {
            herbs = herbs.max(0.85);
            trees = trees.clamp(0.2, 0.55);
        }
        Some(4 | 5) => {
            herbs = herbs.min(0.15);
            trees = 0.0;
        }
        Some(6 | 7) => {
            herbs = herbs.clamp(0.35, 0.75);
            trees = trees.min(0.08);
        }
        Some(29) => treeline = 0.0,
        Some(30) => {
            treeline = 0.0;
            grow = grow.min(0.2);
        }
        _ => {}
    }
    (herbs * grow, trees * treeline)
}

/// The bare ground's rock colour of `c`, if known.
fn ground_tint(c: &Cell, pal: &Palette) -> Option<[f32; 3]> {
    if c.rock_rgb.iter().all(|v| v.is_finite()) {
        Some(c.rock_rgb)
    } else {
        rock_tint(c.rock, pal)
    }
}

/// Land cover from the climate: (colour, tree cover).
fn vegetated(c: &Cell, pal: &Palette, look: &Look) -> ([f32; 3], f32) {
    let t = c.t_mean_c;
    let tw = warmest(c);
    let tc = coldest(c);
    let ai = aridity_index(c);
    let (herb_cover, tree_cover) = if c.herb_cover.is_finite() && c.tree_cover.is_finite() {
        (c.herb_cover, c.tree_cover)
    } else {
        cover(c, look)
    };
    let mut ground = mix(
        pal.get(COLD_DESERT),
        pal.get(HOT_DESERT),
        smoothstep(2.0, 16.0, t),
    );
    if let Some(tint) = ground_tint(c, pal) {
        ground = mix(ground, tint, look.rock_tint);
    }
    let grassland = mix(pal.get(STEPPE), pal.get(SAVANNA), smoothstep(0.35, 0.8, ai));
    let herb_colour = mix(pal.get(TUNDRA), grassland, smoothstep(6.0, 14.0, tw));
    let forest_colour = mix(
        mix(
            pal.get(BOREAL),
            pal.get(TEMPERATE),
            smoothstep(-10.0, 0.0, tc),
        ),
        pal.get(RAINFOREST),
        smoothstep(14.0, 19.0, tc),
    );
    let rgb = mix(
        mix(ground, herb_colour, herb_cover),
        forest_colour,
        tree_cover,
    );
    (rgb, tree_cover)
}

/// Bare ground by geology (no climate).
fn bare(c: &Cell, pal: &Palette) -> [f32; 3] {
    let lowland = mix(pal.get(HOT_DESERT), pal.get(COLD_DESERT), 0.5);
    let rgb = match ground_tint(c, pal) {
        Some(t) => mix(lowland, t, 0.7),
        None => mix(
            lowland,
            pal.get(BARE_ROCK),
            smoothstep(200.0, 2500.0, c.z_m),
        ),
    };
    if c.regime.is_finite() && c.regime.round() as u8 == crate::plates::state::regime::HOTSPOT {
        mix(rgb, pal.get(VOLCANIC), 0.5)
    } else {
        rgb
    }
}

/// The cell of `f` nearest pixel (px, py) of a `w × h` picture (keyframe pixel centres).
pub(crate) fn nearest(f: &Field, px: usize, py: usize, w: usize, h: usize) -> f32 {
    let (i, j) = f
        .grid
        .picture_cell(px as f64, py as f64, w as u32, h as u32);
    f.get(i, j)
}

/// A box-filtered copy of `f`: each cell the mean over ±`rx` columns (wrapping) and ±`ry` rows
/// (clamped), geometric when `log`, counting only the cells where `keep` holds (all when `None`).
/// The climate's fields carry steps a picture should not show as stripes: the orographic rain's
/// cell-to-cell streaks, and the energy-balance model's latitude bands in the temperatures and the
/// sea ice (whose edge would otherwise be a ruled line).
pub fn box_filter(
    f: &Field,
    (rx, ry): (usize, usize),
    log: bool,
    keep: Option<&dyn Fn(usize) -> bool>,
) -> Field {
    let (w, h) = (f.grid.w as usize, f.grid.h as usize);
    let n = w * h;
    let mut v = vec![0.0f64; n];
    let mut wt = vec![0.0f64; n];
    for k in 0..n.min(f.data.len()) {
        let x = f.data[k];
        if x.is_finite() && keep.is_none_or(|keep| keep(k)) {
            v[k] = if log {
                f64::from(x.max(1e-4)).ln()
            } else {
                f64::from(x)
            };
            wt[k] = 1.0;
        }
    }
    // Rows (x wraps), then columns (y clamps).
    let pass = |a: &[f64], horizontal: bool| -> Vec<f64> {
        let mut out = vec![0.0f64; n];
        for j in 0..h {
            for i in 0..w {
                let mut sum = 0.0;
                if horizontal {
                    for d in 0..=2 * rx {
                        sum += a[j * w + (i + w * (rx / w + 1) + d - rx) % w];
                    }
                } else {
                    for d in 0..=2 * ry {
                        let jj = (j + d).saturating_sub(ry).min(h - 1);
                        sum += a[jj * w + i];
                    }
                }
                out[j * w + i] = sum;
            }
        }
        out
    };
    let (v, wt) = if rx > 0 {
        (pass(&v, true), pass(&wt, true))
    } else {
        (v, wt)
    };
    let (v, wt) = if ry > 0 {
        (pass(&v, false), pass(&wt, false))
    } else {
        (v, wt)
    };
    let data = v
        .iter()
        .zip(&wt)
        .map(|(&s, &c)| match (c > 0.0, log) {
            (false, _) => f32::NAN,
            (true, true) => (s / c).exp() as f32,
            (true, false) => (s / c) as f32,
        })
        .collect();
    Field {
        grid: f.grid.clone(),
        data,
    }
}

/// The mean of `f` over the cells pixel (px, py) covers when `f` is finer than the picture, else
/// the nearest cell.
fn mean(f: &Field, px: usize, py: usize, w: usize, h: usize) -> f32 {
    let (kx, ky) = (f.grid.w as usize / w, f.grid.h as usize / h);
    if kx < 2 && ky < 2 {
        return nearest(f, px, py, w, h);
    }
    let (ci, cj) = f
        .grid
        .picture_cell(px as f64, py as f64, w as u32, h as u32);
    let (rx, ry) = ((kx / 2) as i64, (ky / 2) as i64);
    let (mut sum, mut n) = (0.0f64, 0u32);
    for j in cj - ry..=cj + ry {
        for i in ci - rx..=ci + rx {
            let v = f.get(i, j);
            if v.is_finite() {
                sum += f64::from(v);
                n += 1;
            }
        }
    }
    if n == 0 {
        f32::NAN
    } else {
        (sum / f64::from(n)) as f32
    }
}

/// `f` if it lies on grid `g`.
fn on_grid<'a>(f: Option<&'a Field>, g: &deeptime_format::GridDesc) -> Option<&'a Field> {
    f.filter(|f| f.grid == *g)
}

/// The lighting of the hill shade, worked out once from a [`Look`] and applied per pixel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    /// Towards the light in (east, south, up).
    dir: [f64; 3],
    exaggeration: f64,
}

impl Light {
    pub fn new(look: &Look) -> Light {
        let az = f64::from(look.light_azimuth_deg).to_radians();
        let alt = f64::from(look.light_altitude_deg).to_radians();
        Light {
            dir: [az.sin() * alt.cos(), -az.cos() * alt.cos(), alt.sin()],
            exaggeration: f64::from(look.exaggeration),
        }
    }

    /// The factor (1 on flat ground) of a surface whose height rises `dzdx` metres per metre to
    /// the east and `dzdy` per metre to the south. [`hillshade`] is this over a picture; the
    /// workbench's zoomable tiles ask it with slopes taken at their own scale, so every map of
    /// one world is lit the same way.
    #[inline]
    pub fn shade(&self, dzdx: f64, dzdy: f64) -> f32 {
        self.shade_exaggerated(dzdx, dzdy, 1.0)
    }

    /// [`Light::shade`] with the look's vertical exaggeration multiplied by `boost`.
    #[inline]
    pub fn shade_exaggerated(&self, dzdx: f64, dzdy: f64, boost: f64) -> f32 {
        let e = self.exaggeration * boost;
        let n = [-e * dzdx, -e * dzdy, 1.0];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        let lit = ((n[0] * self.dir[0] + n[1] * self.dir[1] + n[2] * self.dir[2]) / len).max(0.0);
        (lit / self.dir[2]) as f32
    }
}

/// A colour under the hill shade: `shade` (from [`Light::shade`] or [`hillshade`]) applied with
/// the `strength` [`colour`] returned (the share of its effect), as the satellite picture does.
pub fn shaded(rgb: [f32; 3], strength: f32, shade: f32) -> [u8; 3] {
    let f = (1.0 + strength * (shade - 1.0)).clamp(0.5, 1.35);
    rgb.map(|v| (v * f).round().clamp(0.0, 255.0) as u8)
}

/// The hill shade's factor per pixel (1 on flat ground) of heights `z` (m) on a `w × h`
/// equirectangular picture of a sphere of radius `radius_m`.
pub fn hillshade(z: &[f32], w: usize, h: usize, radius_m: f64, look: &Look) -> Vec<f32> {
    let light = Light::new(look);
    let dy = std::f64::consts::PI * radius_m / h as f64;
    let at = |i: usize, j: usize, c: f32| {
        let v = z[j * w + i];
        if v.is_finite() {
            f64::from(v)
        } else {
            f64::from(c)
        }
    };
    let mut out = vec![1.0f32; w * h];
    for j in 0..h {
        let lat = deeptime_format::Projection::picture_lat_deg(j as f64, h as u32).to_radians();
        let dx = 2.0 * std::f64::consts::PI * radius_m * lat.cos().max(0.15) / w as f64;
        let (jn, js) = (j.saturating_sub(1), (j + 1).min(h - 1));
        for i in 0..w {
            let c = z[j * w + i];
            if !c.is_finite() {
                continue;
            }
            let (iw, ie) = ((i + w - 1) % w, (i + 1) % w);
            let dzdx = (at(ie, j, c) - at(iw, j, c)) / (2.0 * dx);
            let dzdy = (at(i, js, c) - at(i, jn, c)) / ((js - jn).max(1) as f64 * dy);
            out[j * w + i] = light.shade(dzdx, dzdy);
        }
    }
    out
}

/// The finest elevation field of `world` (`ELEVATION_FIELDS`), if there is one.
pub fn elevation_field(world: &WorldState) -> Option<&Field> {
    ELEVATION_FIELDS
        .iter()
        .find_map(|&name| world.fields.get(name))
}

/// Height above sea level (m; NaN where the field has none) of each pixel of a `size` picture of
/// `elevation` (block y): the mean over the cells a pixel covers when its grid is finer, else the
/// nearest cell. The satellite view and the water view draw on the same pixels.
pub fn heights_m(elevation: &Field, view: &ViewScale, size: (u32, u32)) -> Vec<f32> {
    let (w, h) = (size.0 as usize, size.1 as usize);
    let mut z = Vec::with_capacity(w * h);
    for py in 0..h {
        for px in 0..w {
            let y = mean(elevation, px, py, w, h);
            z.push(((f64::from(y) - view.sea_level_y) * view.m_per_block_v) as f32);
        }
    }
    z
}

/// The satellite picture of `world`'s current state at `size`, RGBA8 (row 0 north, column 0 the
/// seam, as every keyframe layer); `None` without an elevation field.
pub fn render(
    world: &WorldState,
    ramps: &Ramps,
    view: &ViewScale,
    size: (u32, u32),
) -> Option<Vec<u8>> {
    let elevation = elevation_field(world)?;
    let pal = ramps
        .ramps
        .get(RAMP)
        .map(Palette::from_ramp)
        .unwrap_or_else(Palette::builtin);
    let look = Look::default();
    let (w, h) = (size.0 as usize, size.1 as usize);
    let z = heights_m(elevation, view, size);
    let shade = hillshade(&z, w, h, view.radius_m, &look);
    let field = |name: &str| world.fields.get(name);
    // The climate's fields, smoothed where their model draws steps (see `box_filter`).
    let smooth =
        |name: &str, r: (usize, usize), log: bool| field(name).map(|f| box_filter(f, r, log, None));
    let koppen = field(input::KOPPEN);
    let (t_mean, t_warm, t_cold) = (
        smooth(input::T_MEAN, (0, 3), false),
        smooth(input::T_WARM, (0, 3), false),
        smooth(input::T_COLD, (0, 3), false),
    );
    let (precip, aridity) = (
        smooth(input::PRECIP, (1, 1), true),
        smooth(input::ARIDITY, (1, 1), true),
    );
    // Sea ice over the sea's cells only (it is 0 on land).
    let sea_ice = field(input::SEA_ICE).map(|f| match koppen.filter(|k| k.grid == f.grid) {
        Some(k) => {
            let sea = |c: usize| k.data[c].round() as u8 == crate::climate::koppen::SEA;
            box_filter(f, (2, 8), false, Some(&sea))
        }
        None => box_filter(f, (2, 8), false, None),
    });
    let (ice, glacier, ela) = (field(input::ICE), field(input::GLACIER), field(input::ELA));
    let rock = field(input::SURFACE_ROCK).or_else(|| field(input::STRUCTURE_ROCK));
    let regime = field(input::REGIME);
    // Herb and tree cover per cell of the climate's grid, smoothed over the land's neighbouring
    // cells so a Köppen border is a gradient, not a step.
    let covers = t_mean.as_ref().map(|tm| {
        let g = &tm.grid;
        let (tw, tc) = (on_grid(t_warm.as_ref(), g), on_grid(t_cold.as_ref(), g));
        let (ar, pr, kp) = (
            on_grid(aridity.as_ref(), g),
            on_grid(precip.as_ref(), g),
            on_grid(koppen, g),
        );
        let get = |f: Option<&Field>, k: usize| f.map_or(f32::NAN, |f| f.data[k]);
        let n = tm.data.len();
        let (mut herb, mut tree) = (vec![f32::NAN; n], vec![f32::NAN; n]);
        for k in 0..n {
            let mut c = Cell::at(0.0);
            c.t_mean_c = tm.data[k];
            c.t_warm_c = get(tw, k);
            c.t_cold_c = get(tc, k);
            c.aridity = get(ar, k);
            c.precip_mm = get(pr, k);
            c.koppen = get(kp, k);
            if c.has_climate() {
                (herb[k], tree[k]) = cover(&c, &look);
            }
        }
        let land = |k: usize| get(kp, k).round() as u8 != crate::climate::koppen::SEA;
        let smooth = |data: Vec<f32>| {
            let f = Field {
                grid: tm.grid.clone(),
                data,
            };
            box_filter(&f, (1, 1), false, Some(&land))
        };
        (smooth(herb), smooth(tree))
    });
    // The rock colour of bare ground per cell, smoothed over its neighbours (a speckle of cells
    // with other rock is texture a satellite picture at this scale would not show).
    let rock_rgb = rock.map(|f| {
        let mut ch = [
            vec![f32::NAN; f.data.len()],
            vec![f32::NAN; f.data.len()],
            vec![f32::NAN; f.data.len()],
        ];
        for (k, &v) in f.data.iter().enumerate() {
            if let Some(t) = rock_tint(v, &pal) {
                for c in 0..3 {
                    ch[c][k] = t[c];
                }
            }
        }
        ch.map(|data| {
            let g = Field {
                grid: f.grid.clone(),
                data,
            };
            box_filter(&g, (1, 1), false, None)
        })
    });
    let mut rgba = Vec::with_capacity(w * h * 4);
    for py in 0..h {
        for px in 0..w {
            let k = py * w + px;
            if !z[k].is_finite() {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
                continue;
            }
            let at = |f: Option<&Field>| f.map_or(f32::NAN, |f| nearest(f, px, py, w, h));
            let cell = Cell {
                z_m: z[k],
                t_mean_c: at(t_mean.as_ref()),
                t_warm_c: at(t_warm.as_ref()),
                t_cold_c: at(t_cold.as_ref()),
                precip_mm: at(precip.as_ref()),
                aridity: at(aridity.as_ref()),
                koppen: at(koppen),
                ice_m: at(ice),
                glacier: at(glacier),
                sea_ice: at(sea_ice.as_ref()),
                ela_m: at(ela),
                rock: at(rock),
                regime: at(regime),
                herb_cover: at(covers.as_ref().map(|c| &c.0)),
                tree_cover: at(covers.as_ref().map(|c| &c.1)),
                rock_rgb: match &rock_rgb {
                    Some(ch) => [at(Some(&ch[0])), at(Some(&ch[1])), at(Some(&ch[2]))],
                    None => [f32::NAN; 3],
                },
            };
            let (rgb, strength) = colour(&cell, &pal, &look);
            rgba.extend_from_slice(&shaded(rgb, strength, shade[k]));
            rgba.push(255);
        }
    }
    Some(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::GridDesc;

    const W: usize = 32;
    const H: usize = 16;

    fn luma(p: &[u8]) -> f32 {
        0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2])
    }

    /// A synthetic frame: 8 regions of 4 columns each (flat inside, so the middle columns are
    /// unshaded), all latitudes alike.
    fn world(regions: &[(f32, [f32; 8])], climate: bool) -> WorldState {
        let grid = GridDesc::for_planet("L1", (W * 16) as u32, 16).unwrap();
        assert_eq!((grid.w as usize, grid.h as usize), (W, H));
        let mut world = WorldState::new(1, (W * 16) as u32, 0.0);
        let names = [
            input::T_MEAN,
            input::T_WARM,
            input::T_COLD,
            input::ARIDITY,
            input::KOPPEN,
            input::ICE,
            input::SEA_ICE,
            input::SURFACE_ROCK,
        ];
        let mut data = vec![vec![0.0f32; W * H]; names.len() + 1];
        for j in 0..H {
            for i in 0..W {
                let (z, v) = regions[i / 4];
                data[0][j * W + i] = (63.0 + f64::from(z) / 20.0) as f32;
                for (k, x) in v.iter().enumerate() {
                    data[k + 1][j * W + i] = *x;
                }
            }
        }
        world.fields.insert(
            "elevation".into(),
            Field {
                grid: grid.clone(),
                data: data[0].clone(),
            },
        );
        for (k, name) in names.iter().enumerate() {
            if climate || *name == input::SURFACE_ROCK {
                world.fields.insert(
                    (*name).into(),
                    Field {
                        grid: grid.clone(),
                        data: data[k + 1].clone(),
                    },
                );
            }
        }
        world
    }

    const NAN: f32 = f32::NAN;
    // [t_mean, t_warm, t_cold, aridity, köppen, ice m, sea ice, rock]
    fn regions() -> Vec<(f32, [f32; 8])> {
        let sand = crate::synth::EVERY_CLASS
            .iter()
            .position(|&c| c == "sand")
            .unwrap() as f32;
        let granite = 0.0;
        vec![
            // 0 deep ocean, ice-free
            (-6000.0, [4.0, 5.0, 3.0, NAN, 31.0, 0.0, 0.0, NAN]),
            // 1 a shallow tropical bank
            (-8.0, [26.0, 28.0, 24.0, NAN, 31.0, 0.0, 0.0, NAN]),
            // 2 hot sand desert (BWh)
            (400.0, [27.0, 35.0, 16.0, 0.02, 4.0, 0.0, 0.0, sand]),
            // 3 rainforest (Af)
            (200.0, [26.0, 27.0, 25.0, 2.5, 0.0, 0.0, 0.0, granite]),
            // 4 glacier ice (EF) 1.5 km thick
            (
                1500.0,
                [-25.0, -8.0, -40.0, 1.0, 30.0, 1500.0, 0.0, granite],
            ),
            // 5 frozen sea (sea ice cover 1)
            (-3000.0, [-20.0, -4.0, -32.0, NAN, 31.0, 0.0, 1.0, NAN]),
            // 6 steppe (BSk)
            (700.0, [8.0, 22.0, -8.0, 0.3, 7.0, 0.0, 0.0, sand]),
            // 7 tundra (ET)
            (150.0, [-12.0, 6.0, -30.0, 1.2, 29.0, 0.0, 0.0, granite]),
        ]
    }

    fn pixel(rgba: &[u8], region: usize) -> [u8; 4] {
        let k = ((H / 2) * W + region * 4 + 1) * 4;
        [rgba[k], rgba[k + 1], rgba[k + 2], rgba[k + 3]]
    }

    #[test]
    fn ramp_labels_are_the_classes() {
        let v: serde_json::Value =
            serde_json::from_str(deeptime_format::ramps::RAMPS_JSON).unwrap();
        let ramp = &v["ramps"][RAMP];
        let labels: Vec<&str> = ramp["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l.as_str().unwrap())
            .collect();
        assert_eq!(labels, CLASSES);
        // Categorical: [k, c], [k + 0.999, c] for every class.
        let stops = ramp["stops"].as_array().unwrap();
        assert_eq!(stops.len(), 2 * CLASSES.len());
        for (k, pair) in stops.chunks(2).enumerate() {
            assert_eq!(pair[0][0].as_f64().unwrap(), k as f64);
            assert!((pair[1][0].as_f64().unwrap() - k as f64 - 0.999).abs() < 1e-9);
            assert_eq!(pair[0][1], pair[1][1]);
        }
        assert_eq!(v["layers"][LAYER], RAMP);
    }

    #[test]
    fn synthetic_frame_reads_like_a_satellite_picture() {
        let w = world(&regions(), true);
        let ramps = Ramps::builtin();
        let rgba = render(&w, &ramps, &ViewScale::default(), (W as u32, H as u32)).unwrap();
        assert_eq!(rgba.len(), W * H * 4);
        let pal = Palette::builtin();
        let p = |r| pixel(&rgba, r);
        // Deep ocean: dark navy, the palette's deep colour (flat, unshaded).
        let deep = p(0);
        assert!(luma(&deep) < 40.0 && deep[2] > deep[0] + 30, "{deep:?}");
        assert_eq!(&deep[..3], &pal.0[DEEP].map(|v| v as u8)[..]);
        // A shallow bank: turquoise, far lighter than the deep sea, blue-green over red.
        let bank = p(1);
        assert!(bank[1] > 140 && bank[2] > 140 && bank[0] < 100, "{bank:?}");
        // Hot desert: tan (red > green > blue), bright.
        let desert = p(2);
        assert!(desert[0] > desert[1] && desert[1] > desert[2], "{desert:?}");
        assert!(desert[0] > 180 && luma(&desert) > 150.0, "{desert:?}");
        // Rainforest: deep green.
        let forest = p(3);
        assert!(forest[1] > forest[0] && forest[1] > forest[2], "{forest:?}");
        assert!(luma(&forest) < 80.0, "{forest:?}");
        // Glacier ice and the frozen sea: white.
        for r in [4, 5] {
            let ice = p(r);
            assert!(ice.iter().take(3).all(|&v| v > 210), "region {r}: {ice:?}");
        }
        // Steppe: khaki, between the desert and the forest in brightness, not green-dominated.
        let steppe = p(6);
        assert!(
            steppe[0] >= steppe[2] && steppe[1] > steppe[2],
            "{steppe:?}"
        );
        assert!(luma(&steppe) < luma(&desert) && luma(&steppe) > luma(&forest));
        // Tundra: brownish grey, darker than snow and than the desert, not green like the forest,
        // with a light seasonal-snow tint (coldest month −30 °C).
        let tundra = p(7);
        assert!(
            luma(&tundra) < 200.0 && luma(&tundra) > luma(&forest),
            "{tundra:?}"
        );
        // Everything is opaque.
        assert!((0..8).all(|r| p(r)[3] == 255));
    }

    #[test]
    fn snowball_is_white() {
        // Every sea frozen over, every land under ice.
        let mut rs = regions();
        for (z, v) in &mut rs {
            v[0] = -40.0;
            v[1] = -20.0;
            v[2] = -55.0;
            v[4] = if *z < 0.0 { 31.0 } else { 30.0 };
            v[5] = if *z < 0.0 { 0.0 } else { 800.0 };
            v[6] = if *z < 0.0 { 1.0 } else { 0.0 };
        }
        let w = world(&rs, true);
        let rgba = render(
            &w,
            &Ramps::builtin(),
            &ViewScale::default(),
            (W as u32, H as u32),
        )
        .unwrap();
        for px in rgba.chunks(4) {
            assert!(luma(px) > 200.0, "{px:?}");
        }
    }

    #[test]
    fn without_a_climate_the_land_is_bare_rock_and_the_sea_has_no_ice() {
        let w = world(&regions(), false);
        let rgba = render(
            &w,
            &Ramps::builtin(),
            &ViewScale::default(),
            (W as u32, H as u32),
        )
        .unwrap();
        let pal = Palette::builtin();
        // The ice cap and the frozen sea of the climate frame are rock and open sea here.
        let rock = pixel(&rgba, 4);
        assert!(luma(&rock) < 190.0, "{rock:?}");
        let sea = pixel(&rgba, 5);
        assert!(luma(&sea) < 60.0 && sea[2] > sea[0], "{sea:?}");
        // No green anywhere on land: bare ground is warm-toned (red ≥ green).
        for r in [2, 3, 4, 6, 7] {
            let p = pixel(&rgba, r);
            assert!(p[0] >= p[1] && p[1] >= p[2], "region {r}: {p:?}");
        }
        assert_eq!(&pixel(&rgba, 0)[..3], &pal.0[DEEP].map(|v| v as u8)[..]);
        // No elevation, no picture.
        let mut none = w.clone();
        none.fields.remove("elevation");
        assert!(render(&none, &Ramps::builtin(), &ViewScale::default(), (8, 4)).is_none());
    }

    #[test]
    fn hill_shade_lights_north_west_slopes() {
        // A ridge running north–south: its west flank faces the light, its east flank is shaded.
        let (w, h) = (16usize, 8usize);
        let z: Vec<f32> = (0..w * h)
            .map(|k| {
                let i = (k % w) as f32;
                (2000.0 - 400.0 * (i - 8.0).abs()).max(0.0)
            })
            .collect();
        let s = hillshade(&z, w, h, 6.37e6, &Look::default());
        let at = |i: usize| s[4 * w + i];
        assert!(at(6) > 1.0 && at(10) < 1.0, "{} {}", at(6), at(10));
        assert!((at(0) - 1.0).abs() < 1e-6, "flat ground is unchanged");
    }

    #[test]
    fn box_filter_smooths_bands_and_respects_the_mask() {
        let grid = GridDesc::for_planet("L1", 8 * 16, 16).unwrap();
        let (w, h) = (grid.w as usize, grid.h as usize);
        // A step between rows 1 and 2, and a land column (masked out) holding 0.
        let data: Vec<f32> = (0..w * h)
            .map(|k| if k % w == 3 || k / w >= 2 { 0.0 } else { 1.0 })
            .collect();
        let f = Field { grid, data };
        let keep = |k: usize| k % w != 3;
        let b = box_filter(&f, (1, 1), false, Some(&keep));
        // Row 1 now lies between the bands; the column beside the land keeps its value.
        let v = b.data[w + 1];
        assert!(v > 0.3 && v < 0.9, "{v}");
        assert_eq!(
            b.data[4], 1.0,
            "the masked land column does not pull the sea down"
        );
        // x wraps: column 0 of row 0 averages columns 7, 0 and 1 of rows 0 and 1.
        assert_eq!(b.data[0], 1.0);
        // Geometric: 1, 100, 1 average to 100^(1/3).
        let g = Field {
            grid: f.grid.clone(),
            data: (0..w * h)
                .map(|k| if k % 2 == 0 { 1.0 } else { 100.0 })
                .collect(),
        };
        let gb = box_filter(&g, (1, 0), true, None);
        assert!(
            (gb.data[1] - 100.0f32.powf(1.0 / 3.0)).abs() < 1e-3,
            "{}",
            gb.data[1]
        );
    }

    /// The hill shade is the arithmetic it was before `Light` was split out for the workbench's
    /// tiles (the frame maps' pixels must not move): the lighting formula written out, bit for bit.
    #[test]
    fn hillshade_is_the_lighting_formula_and_light_is_shared() {
        let (w, h) = (24usize, 12usize);
        let mut s = 0x9E37_79B9_7F4A_7C15u64;
        let z: Vec<f32> = (0..w * h)
            .map(|_| {
                s = s
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((s >> 40) as f32 / 16777216.0 - 0.5) * 6000.0
            })
            .collect();
        let look = Look::default();
        let got = hillshade(&z, w, h, 6.37e6, &look);
        let az = f64::from(look.light_azimuth_deg).to_radians();
        let alt = f64::from(look.light_altitude_deg).to_radians();
        let light = [az.sin() * alt.cos(), -az.cos() * alt.cos(), alt.sin()];
        let e = f64::from(look.exaggeration);
        let dy = std::f64::consts::PI * 6.37e6 / h as f64;
        for j in 0..h {
            let lat = (90.0 - 180.0 * j as f64 / h as f64).to_radians();
            let dx = 2.0 * std::f64::consts::PI * 6.37e6 * lat.cos().max(0.15) / w as f64;
            let (jn, js) = (j.saturating_sub(1), (j + 1).min(h - 1));
            for i in 0..w {
                let (iw, ie) = ((i + w - 1) % w, (i + 1) % w);
                let dzdx = (f64::from(z[j * w + ie]) - f64::from(z[j * w + iw])) / (2.0 * dx);
                let dzdy = (f64::from(z[js * w + i]) - f64::from(z[jn * w + i]))
                    / ((js - jn).max(1) as f64 * dy);
                let n = [-e * dzdx, -e * dzdy, 1.0];
                let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                let lit = ((n[0] * light[0] + n[1] * light[1] + n[2] * light[2]) / len).max(0.0);
                assert_eq!(got[j * w + i], (lit / light[2]) as f32, "({i}, {j})");
            }
        }
        // `Light` is that arithmetic for any slope; a boost of 1 changes nothing, and flat ground
        // is 1 at any exaggeration.
        let l = Light::new(&look);
        assert_eq!(l.shade(0.0, 0.0), 1.0);
        assert_eq!(l.shade_exaggerated(0.0, 0.0, 3.0), 1.0);
        assert_eq!(l.shade(0.01, -0.02), l.shade_exaggerated(0.01, -0.02, 1.0));
        assert!(l.shade_exaggerated(0.01, -0.02, 4.0) != l.shade(0.01, -0.02));
        // And the composition: no shade is the colour itself, a shaded one is clamped.
        assert_eq!(shaded([100.0, 150.0, 200.0], 0.6, 1.0), [100, 150, 200]);
        assert_eq!(shaded([200.0; 3], 1.0, 9.0), [255, 255, 255]);
        assert_eq!(shaded([200.0; 3], 1.0, 0.0), [100, 100, 100]);
    }
}

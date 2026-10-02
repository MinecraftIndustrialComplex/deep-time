// SPDX-License-Identifier: LGPL-3.0-or-later
//! The "Rivers & lakes" view (Ben, 2026-10-01: "Rivers & lakes on the maps"): the rivers, with
//! their width by flow, and the lakes of a frame's terrain, drawn over the satellite view of every
//! frame and as a map layer of their own (`rivers`).
//!
//! **View only**, like `satellite` and `provinces`: the water is derived when a frame is drawn,
//! from the fields the stages already wrote, and nothing is written to `WorldState`, so a run's
//! world files and the bake are what they were. The simulation and the bake do no extra work.
//!
//! # Where the water comes from
//! - **Every frame: the bake's own hydrology on the frame's terrain.** [`derive`] runs
//!   `hydro::network::run`, the code the bake's water part runs on the finished terrain, on the
//!   frame's elevation (`amplify/elevation`, else `young/elevation`, else `elevation`; a grid over
//!   [`MAX_CELLS`] is averaged down by 2 until it is under). That is a Priority-Flood of the frame
//!   grid from its open sea (a body of at least 5,000 km² below sea level), a lake wherever it
//!   fills a depression of at least `hydro.lake_min_area_km2` and `hydro.lake_min_depth_blocks`,
//!   the discharge of the climate's runoff (the frame's `climate/precip_mm` and `t_mean_c`, bilinear
//!   at each cell, as the bake does; the surface stage's zonal stand-in without a climate stage)
//!   accumulated down the steepest descent, and a river wherever it passes
//!   `hydro.river_min_discharge_m3s` (2 m³/s) from at least `hydro.river_min_area_km2` (50 km²).
//!   The thresholds are physical (a cell's area comes from the grid's blocks and the preset's
//!   metres per block), so they scale with the frame's resolution by themselves: on a 4.8 km cell
//!   a stream starts where about nine cells drain into it.
//! - **The final frame: the bake's files.** Once the bake has written `rivers/rivers.dtrv` and the
//!   `water_level` layer, [`from_bake`] draws exactly those, and the session draws the run's last
//!   frame again from them (`Session::finalize_write`, before the frame files are listed in the
//!   manifest). A run without a bake keeps the derived water.
//!
//! # How it is drawn
//! A [`WaterView`] holds the water in the picture's own pixels: the coverage of the rivers (a
//! thick line along each segment of the network, anti-aliased, the largest discharge where several
//! meet), the discharge under each river pixel, and the coverage of the lakes (the share of the
//! frame's cells under each pixel that hold lake water). A river is drawn at least
//! [`Look::width_px`] wide, thicker by [`Look::width_per_decade`] for every tenfold of flow past
//! the stream threshold, and at least its real width (`w` blocks across a `C`-block planet) so the
//! great rivers keep their true size. The colour is the `rivers` ramp of `ramps.json` at
//! log₁₀(m³/s): a clear water blue that deepens with flow. Lakes are one flat colour ([`LAKE`]).
//! Neither is drawn on the sea (a pixel at least a block below sea level).
//!
//! **Lakes follow the climate** (Ben, 2026-10-01, "freeze them and size them by climate"; the
//! hydrology's water balance, `hydro`): a basin that evaporates more than it gets holds a smaller
//! lake below its rim, or only a **playa** (its floor, where the water that reaches it dries:
//! drawn as a salt flat, [`PLAYA`]), or nothing. A lake cell whose mean annual temperature is
//! below `hydro.lake_freeze_mean_c` (0 °C: under ice for more than half the year, the frame's
//! `climate/t_mean_c` at the cell) is a **frozen lake**, drawn as lake ice ([`FROZEN`]).
//!
//! - **On the satellite view** ([`WaterView::overlay`]): rivers are blended over the land at 60 %
//!   (a stream) to 92 % (the largest river), lakes (open and frozen) at 95 % and playas at 80 %,
//!   after the hill shade, so they are flat. Under glacier ice (`climate/ice_thickness_m`, gone
//!   from 60 m of it) none show, as the ice is drawn white over the ground.
//! - **As a layer** ([`WaterView::render`]): a quiet base (land in a pale neutral with a soft hill
//!   shade, the sea in slate) with the lakes, playas and rivers hard-edged on it, so the workbench
//!   can read the colour back under the pointer: a pixel on the ramp is "river (~N m³/s)", one of
//!   [`LAKE`] is "lake", of [`FROZEN`] "frozen lake", of [`PLAYA`] "salt flat". On the satellite
//!   view the pointer reads the same three by their colours.

use std::path::Path;
use std::time::Instant;

use deeptime_format::rivers::{Rivers, read_rivers};
use deeptime_format::{GridDesc, Ramp, Ramps};

use crate::bake::water::{halve, load_i16};
use crate::error::CoreError;
use crate::hydro::Climate;
use crate::hydro::network::{self, NODATA, kind};
use crate::keyframes::ViewScale;
use crate::satellite;
use crate::surface::climate::{P_FIELD, T_FIELD, zonal_precip, zonal_temperature};
use crate::world::{Field, WorldState};

/// The keyframe layer's name.
pub const LAYER: &str = "rivers";
/// The ramp that colours the rivers: log₁₀ of the discharge, m³/s.
pub const RAMP: &str = "rivers";

/// The largest raster the water of a frame is derived on, cells (1024 × 512 is 0.52 M). A finer
/// grid (a 32k planet's L1, the young landscape's L2, the amplifier's preview) is averaged down.
pub const MAX_CELLS: usize = 600_000;

/// The layer's land, sea and lake colours (`ramps.json`: `rivers.keys`, so the legend and the
/// workbench's hover read the same).
pub const LAND: [u8; 3] = [0xdd, 0xd9, 0xcd];
pub const SEA: [u8; 3] = [0x2c, 0x40, 0x56];
pub const LAKE: [u8; 3] = [0x5c, 0xc0, 0xea];
/// A frozen lake (lake ice: a pale blue, greyer than open water and bluer than snow, glacier ice
/// and sea ice; on both views) and a playa (a salt flat) on the layer (`rivers.keys`:
/// `frozen_lake`, `salt_flat`).
pub const FROZEN: [u8; 3] = [0xa9, 0xd3, 0xe6];
pub const PLAYA: [u8; 3] = [0xf0, 0xd9, 0xa8];
/// A playa on the satellite view: its Salt flat class's colour (`satellite.keys.salt_flat`).
pub const PLAYA_SATELLITE: [u8; 3] = [0xe3, 0xdd, 0xd0];
/// A frozen lake on the satellite view: snow-covered lake ice, nearly white with a blue cast, so
/// it reads as ice beside open water and as a lake beside snow (`satellite.keys.frozen_lake`).
pub const FROZEN_SATELLITE: [u8; 3] = [0xd0, 0xe6, 0xf0];

/// The look's constants (the module docs give them in words).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// A river's width in pixels at the stream threshold …
    pub width_px: f32,
    /// … and what each tenfold of discharge adds.
    pub width_per_decade: f32,
    /// How opaque a stream and the largest river are on the satellite view, and the decades of
    /// discharge over which one grows into the other.
    pub alpha_min: f32,
    pub alpha_max: f32,
    pub alpha_decades: f32,
    /// How opaque lakes (open and frozen) and playas are on the satellite view.
    pub lake_alpha: f32,
    pub playa_alpha: f32,
    /// The thickness of land ice (m) under which the satellite view shows no water.
    pub ice_full_m: f32,
    /// The share of the hill shade's effect on the layer's land.
    pub relief: f32,
}

impl Default for Look {
    fn default() -> Look {
        Look {
            width_px: 1.0,
            width_per_decade: 0.5,
            alpha_min: 0.6,
            alpha_max: 0.92,
            alpha_decades: 2.5,
            lake_alpha: 0.95,
            playa_alpha: 0.8,
            ice_full_m: 60.0,
            relief: 0.5,
        }
    }
}

/// Where a view's water came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// Derived from the frame's terrain ([`derive`]).
    Derived,
    /// Read from the bake's files ([`from_bake`]).
    Bake,
}

/// What a view holds, for logs and tests.
#[derive(Clone, Debug, PartialEq)]
pub struct Info {
    pub source: Source,
    pub chains: usize,
    pub vertices: usize,
    /// The lakes' area in pixels (summed coverage), open and frozen together, and the frozen
    /// lakes' and the playas' alone.
    pub lake_px: f64,
    pub frozen_px: f64,
    pub playa_px: f64,
    /// Milliseconds it took to make.
    pub millis: f64,
    /// The hydrology's own milliseconds per phase (`hydro::network`'s `stats.millis`: sea, flood,
    /// flats, receivers, lakes, discharge, rivers); `Null` for water read from the bake.
    pub phases: serde_json::Value,
    /// The hydrology's statistics (`hydro::network`'s `stats`: the lakes, what became of the
    /// basins, the lakes by climate, the largest); `Null` for water read from the bake.
    pub stats: serde_json::Value,
}

/// The water of one frame in the picture's pixels (row 0 north, pixel (px, py) centred on the
/// point of the planet the keyframe layers sample, `keyframes::render`).
#[derive(Clone, Debug)]
pub struct WaterView {
    pub w: usize,
    pub h: usize,
    /// Height above sea level of each pixel, m (NaN: none).
    z: Vec<f32>,
    /// River coverage, 0–1.
    river: Vec<f32>,
    /// Discharge under each river pixel, m³/s (0 where there is none).
    q: Vec<f32>,
    /// Lake coverage, 0–1: open water, lake ice, and playas.
    lake: Vec<f32>,
    frozen: Vec<f32>,
    playa: Vec<f32>,
    /// How much of each pixel's water the land ice hides on the satellite view, 0–1: the
    /// glacier ice (`climate/ice_thickness_m`) is drawn white over rivers and lakes, so under
    /// [`Look::ice_full_m`] of it none show. The `rivers` layer shows the terrain's water as it is.
    ice: Vec<f32>,
    pub info: Info,
}

/// The raster the hydrology runs on: the elevation field in 1/16 blocks (NaN is deep sea),
/// averaged 2 × 2 until it is at most [`MAX_CELLS`].
fn work_terrain(field: &Field, sea_raw: i32) -> (GridDesc, Vec<i16>) {
    let deep = (sea_raw - 16 * 100).clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
    let mut elev: Vec<i16> = field
        .data
        .iter()
        .map(|&y| {
            if y.is_finite() {
                (f64::from(y) * 16.0)
                    .round()
                    .clamp(f64::from(i16::MIN), f64::from(i16::MAX)) as i16
            } else {
                deep
            }
        })
        .collect();
    let mut grid = field.grid.clone();
    while grid.cells() > MAX_CELLS && grid.w.is_multiple_of(2) && grid.h.is_multiple_of(2) {
        (grid, elev) = halve(&grid, &elev);
    }
    (grid, elev)
}

/// The climate at block (x, z), as the bake's water part reads it: precipitation, mean
/// temperature and the warmest and coldest month (`climate/t_warm_c`, `climate/t_cold_c`; the
/// mean ± 8 °C without them) bilinear from the frame's L1 climate, else the surface stage's zonal
/// stand-in.
fn climate_at<'w>(
    world: &'w WorldState,
    circumference: f64,
) -> impl Fn(i64, i64) -> Climate + Sync + 'w {
    let ok = |f: &&Field| f.data.len() == f.grid.cells() && !f.data.is_empty();
    let t = world.fields.get(T_FIELD).filter(ok);
    let p = world.fields.get(P_FIELD).filter(ok);
    let warm = world.fields.get(crate::climate::field::T_WARM).filter(ok);
    let cold = world.fields.get(crate::climate::field::T_COLD).filter(ok);
    move |bx, bz| {
        let (x, z) = (bx as f64, bz as f64);
        let at =
            |f: Option<&Field>| f.map_or(f64::NAN, |f| f64::from(crate::bake::bilinear(f, x, z)));
        match (t, p) {
            (Some(t), Some(p)) => Climate::of(
                f64::from(crate::bake::bilinear(p, x, z)),
                f64::from(crate::bake::bilinear(t, x, z)),
                at(warm),
                at(cold),
            ),
            _ => {
                let lat = -360.0 * z / circumference;
                Climate::of(
                    zonal_precip(lat),
                    zonal_temperature(lat),
                    f64::NAN,
                    f64::NAN,
                )
            }
        }
    }
}

/// The share of the cells under each pixel of a `(w, h)` map picture that `wet` marks, for a
/// raster on `grid` (the mean over the cells a pixel covers when the grid is finer, the nearest
/// cell when it is not; pixel ↔ cell by `GridDesc::picture_cell`).
fn lake_cover(grid: &GridDesc, wet: &dyn Fn(usize) -> bool, (w, h): (usize, usize)) -> Vec<f32> {
    let (gw, gh) = (grid.w as usize, grid.h as usize);
    let (rx, ry) = ((gw / w).max(1), (gh / h).max(1));
    let norm = 1.0 / (rx * ry) as f32;
    let mut out = vec![0.0f32; w * h];
    for py in 0..h {
        for px in 0..w {
            let (ci, cj) = grid.picture_cell(px as f64, py as f64, w as u32, h as u32);
            let j0 = cj - (ry / 2) as i64;
            let i0 = ci - (rx / 2) as i64;
            let mut n = 0u32;
            for dj in 0..ry as i64 {
                let j = (j0 + dj).clamp(0, gh as i64 - 1) as usize;
                for di in 0..rx as i64 {
                    let i = (i0 + di).rem_euclid(gw as i64) as usize;
                    n += u32::from(wet(j * gw + i));
                }
            }
            out[py * w + px] = n as f32 * norm;
        }
    }
    out
}

impl WaterView {
    fn blank(world: &WorldState, view: &ViewScale, size: (u32, u32)) -> Option<WaterView> {
        let elevation = satellite::elevation_field(world)?;
        if elevation.data.len() != elevation.grid.cells() || elevation.data.is_empty() {
            return None;
        }
        let (w, h) = (size.0 as usize, size.1 as usize);
        if w == 0 || h == 0 {
            return None;
        }
        let look = Look::default();
        let ice = match world.fields.get(satellite::input::ICE) {
            Some(f) if f.data.len() == f.grid.cells() && !f.data.is_empty() => (0..w * h)
                .map(|k| {
                    let m = satellite::nearest(f, k % w, k / w, w, h);
                    if m.is_finite() {
                        ((m - 1.0) / (look.ice_full_m - 1.0)).clamp(0.0, 1.0)
                    } else {
                        0.0
                    }
                })
                .collect(),
            _ => vec![0.0; w * h],
        };
        Some(WaterView {
            w,
            h,
            z: satellite::heights_m(elevation, view, size),
            river: vec![0.0; w * h],
            q: vec![0.0; w * h],
            lake: vec![0.0; w * h],
            frozen: vec![0.0; w * h],
            playa: vec![0.0; w * h],
            ice,
            info: Info {
                source: Source::Derived,
                chains: 0,
                vertices: 0,
                lake_px: 0.0,
                frozen_px: 0.0,
                playa_px: 0.0,
                millis: 0.0,
                phases: serde_json::Value::Null,
                stats: serde_json::Value::Null,
            },
        })
    }

    /// Draws `rivers` (block coordinates on a `C`-block planet) into the pixels the sea does not
    /// hold.
    fn draw_rivers(&mut self, rivers: &Rivers, view: &ViewScale, look: &Look) {
        let c = f64::from(rivers.c);
        if c <= 0.0 {
            return;
        }
        let (fw, ph) = (self.w as f64, self.h as u32);
        let proj = deeptime_format::Projection::of(c);
        let px = |x: f32, z: f32| -> (f32, f32) {
            (
                ((f64::from(x) + c / 2.0) * fw / c) as f32,
                proj.picture_y(f64::from(z), ph) as f32,
            )
        };
        let q_min = view.hydro.river_min_discharge_m3s.max(1e-3) as f32;
        let px_per_block = (fw / c) as f32;
        for ch in &rivers.chains {
            let (a, b) = (ch.first as usize, (ch.first + ch.count) as usize);
            let Some(vs) = rivers.vertices.get(a..b) else {
                continue;
            };
            for s in vs.windows(2) {
                let (v0, v1) = (&s[0], &s[1]);
                let q = 0.5 * (v0.q + v1.q);
                if !(q.is_finite() && q > 0.0) {
                    continue;
                }
                let (p0, p1) = (px(v0.x, v0.z), px(v1.x, v1.z));
                if (p1.0 - p0.0).abs() > 0.5 * self.w as f32 {
                    continue;
                }
                let cartographic =
                    look.width_px + look.width_per_decade * (q.max(q_min) / q_min).log10();
                let real = (v0.hw + v1.hw) * px_per_block;
                let radius = 0.5 * cartographic.max(real);
                self.stamp(p0, p1, radius, q);
            }
        }
        // Not on the sea: a river's last vertex is the sea cell it ends in.
        let floor = -view.m_per_block_v as f32;
        for (k, z) in self.z.iter().enumerate() {
            if z.is_finite() && *z < floor {
                self.river[k] = 0.0;
                self.q[k] = 0.0;
            }
        }
    }

    /// Lays one capsule (the segment p0 → p1 grown by `radius` pixels) into the coverage, keeping
    /// the larger coverage and the larger discharge: the same pixels are met again and again by a
    /// network sampled finer than the picture, which must not thicken it.
    fn stamp(&mut self, p0: (f32, f32), p1: (f32, f32), radius: f32, q: f32) {
        let (w, h) = (self.w as i64, self.h as i64);
        let reach = radius + 1.0;
        let y0 = (p0.1.min(p1.1) - reach).floor().max(0.0) as i64;
        let y1 = ((p0.1.max(p1.1) + reach).ceil() as i64).min(h - 1);
        let x0 = (p0.0.min(p1.0) - reach).floor() as i64;
        let x1 = (p0.0.max(p1.0) + reach).ceil() as i64;
        let (dx, dy) = (p1.0 - p0.0, p1.1 - p0.1);
        let len2 = dx * dx + dy * dy;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (fx, fy) = (x as f32 - p0.0, y as f32 - p0.1);
                let t = if len2 > 0.0 {
                    ((fx * dx + fy * dy) / len2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let d = (fx - t * dx).hypot(fy - t * dy);
                let cov = (radius + 0.5 - d).clamp(0.0, 1.0);
                if cov > 0.0 {
                    let k = (y * w + x.rem_euclid(w)) as usize;
                    if cov > self.river[k] {
                        self.river[k] = cov;
                    }
                    if q > self.q[k] {
                        self.q[k] = q;
                    }
                }
            }
        }
    }

    fn finish(&mut self, rivers: &Rivers, source: Source, started: Instant) {
        self.info = Info {
            source,
            chains: rivers.chains.len(),
            vertices: rivers.vertices.len(),
            lake_px: self
                .lake
                .iter()
                .zip(&self.frozen)
                .map(|(&a, &b)| f64::from(a) + f64::from(b))
                .sum(),
            frozen_px: self.frozen.iter().map(|&c| f64::from(c)).sum(),
            playa_px: self.playa.iter().map(|&c| f64::from(c)).sum(),
            millis: started.elapsed().as_secs_f64() * 1000.0,
            phases: serde_json::Value::Null,
            stats: serde_json::Value::Null,
        };
    }

    /// The river's colour at `q` m³/s on the `rivers` ramp.
    fn river_rgb(ramp: &Ramp, q: f32) -> [u8; 3] {
        ramp.colour(f64::from(q.max(1e-3)).log10())
    }

    /// Draws the water over a satellite picture of the same size (after its hill shade): rivers
    /// blended at [`Look::alpha_min`] to [`Look::alpha_max`] by flow, lakes at
    /// [`Look::lake_alpha`]. Transparent pixels (no data) stay as they are.
    pub fn overlay(&self, rgba: &mut [u8], ramps: &Ramps) {
        let look = Look::default();
        let ramp = ramps
            .ramps
            .get(RAMP)
            .unwrap_or_else(|| ramps.for_layer(LAYER).1);
        let blend = |px: &mut [u8; 4], rgb: [u8; 3], a: f32| {
            for (p, &c) in px.iter_mut().zip(&rgb) {
                *p = (f32::from(*p) * (1.0 - a) + f32::from(c) * a)
                    .round()
                    .clamp(0.0, 255.0) as u8;
            }
        };
        for (k, px) in rgba
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .enumerate()
            .take(self.w * self.h)
        {
            if px[3] == 0 {
                continue;
            }
            let shown = 1.0 - self.ice[k];
            if self.river[k] > 0.0 && shown > 0.0 {
                let q = self.q[k];
                let grow = ((q / 2.0).max(1.0).log10() / look.alpha_decades).clamp(0.0, 1.0);
                let a = self.river[k]
                    * (look.alpha_min + (look.alpha_max - look.alpha_min) * grow)
                    * shown;
                blend(px, Self::river_rgb(ramp, q), a);
            }
            if shown > 0.0 {
                if self.playa[k] > 0.0 {
                    blend(
                        px,
                        PLAYA_SATELLITE,
                        self.playa[k] * look.playa_alpha * shown,
                    );
                }
                if self.lake[k] > 0.0 {
                    blend(px, LAKE, self.lake[k] * look.lake_alpha * shown);
                }
                if self.frozen[k] > 0.0 {
                    blend(
                        px,
                        FROZEN_SATELLITE,
                        self.frozen[k] * look.lake_alpha * shown,
                    );
                }
            }
        }
    }

    /// The `rivers` layer: land (a soft hill shade), sea, the lakes and the rivers, hard-edged.
    /// `None` when the terrain has no height at all.
    pub fn render(&self, ramps: &Ramps, view: &ViewScale) -> Option<Vec<u8>> {
        if self.z.iter().all(|z| !z.is_finite()) {
            return None;
        }
        let look = Look::default();
        let ramp = ramps
            .ramps
            .get(RAMP)
            .unwrap_or_else(|| ramps.for_layer(LAYER).1);
        let shade = satellite::hillshade(
            &self.z,
            self.w,
            self.h,
            view.radius_m,
            &satellite::Look::default(),
        );
        let mut rgba = Vec::with_capacity(self.w * self.h * 4);
        for (k, (&z, &shade)) in self.z.iter().zip(&shade).enumerate() {
            if !z.is_finite() {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
                continue;
            }
            let rgb = if self.lake[k] + self.frozen[k] >= 0.5 {
                if self.frozen[k] > self.lake[k] {
                    FROZEN
                } else {
                    LAKE
                }
            } else if self.playa[k] >= 0.5 {
                PLAYA
            } else if self.river[k] >= 0.5 && self.q[k] > 0.0 {
                Self::river_rgb(ramp, self.q[k])
            } else if z < 0.0 {
                SEA
            } else {
                let f = (1.0 + look.relief * (shade - 1.0)).clamp(0.6, 1.2);
                LAND.map(|v| (f32::from(v) * f).round().clamp(0.0, 255.0) as u8)
            };
            rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        Some(rgba)
    }
}

/// The water of `world`'s current terrain at `size`, derived with the bake's hydrology (the module
/// docs). `None` when the preset's `hydro.enabled` is false, or the world has no elevation.
pub fn derive(world: &WorldState, view: &ViewScale, size: (u32, u32)) -> Option<WaterView> {
    derive_with_rivers(world, view, size).map(|(water, _)| water)
}

/// [`derive`], and the river network it drew (what the bake's file would hold for this terrain).
fn derive_with_rivers(
    world: &WorldState,
    view: &ViewScale,
    size: (u32, u32),
) -> Option<(WaterView, Rivers)> {
    let started = Instant::now();
    if !view.hydro.enabled {
        return None;
    }
    let mut out = WaterView::blank(world, view, size)?;
    let field = satellite::elevation_field(world)?;
    let sea_raw = (view.sea_level_y * 16.0).round() as i32;
    let (grid, elev) = work_terrain(field, sea_raw);
    let climate = climate_at(world, grid.circumference() as f64);
    let net = network::run(&network::Input {
        grid: &grid,
        elev: &elev,
        sea_raw,
        m_per_block_h: view.m_per_block_h,
        climate: &climate,
        params: &view.hydro,
        width_scale: view.rivers_scale,
    });
    // Lake cells a block deep or more: a shallower one is dry ground in the game (and, on a cell
    // of 4.8 km, a margin of the flat a lake fills, which draws as streaks).
    let deep = |c: usize| {
        let level = net.water_level[c];
        level != NODATA && i32::from(level) - i32::from(elev[c]) >= 16
    };
    let size_px = (out.w, out.h);
    out.lake = lake_cover(&grid, &|c| net.kind[c] == kind::LAKE && deep(c), size_px);
    out.frozen = lake_cover(&grid, &|c| net.kind[c] == kind::FROZEN && deep(c), size_px);
    out.playa = lake_cover(&grid, &|c| net.kind[c] == kind::PLAYA, size_px);
    let look = Look::default();
    out.draw_rivers(&net.rivers, view, &look);
    out.finish(&net.rivers, Source::Derived, started);
    out.info.phases = net.stats["millis"].clone();
    out.info.stats = net.stats;
    Some((out, net.rivers))
}

/// The water the bake wrote into `dir` (`rivers/rivers.dtrv`, and the `water_level` layer when the
/// manifest names its file, `water_level`), drawn at `size` over `world`'s terrain. `Ok(None)`
/// when the folder has no rivers file (a bake with `hydro.enabled` false, or none).
pub fn from_bake(
    world: &WorldState,
    view: &ViewScale,
    dir: &Path,
    water_level: Option<&str>,
    size: (u32, u32),
) -> Result<Option<WaterView>, CoreError> {
    let started = Instant::now();
    let Some(rivers) = read_rivers(dir)? else {
        return Ok(None);
    };
    let Some(mut out) = WaterView::blank(world, view, size) else {
        return Ok(None);
    };
    if let Some(path) = water_level {
        let (grid, levels) = load_i16(&dir.join(path))?;
        // Frozen where the cell's mean annual temperature says so, as the hydrology marks it.
        let climate = climate_at(world, grid.circumference() as f64);
        let gw = grid.w as usize;
        let (ox, oz, r) = (grid.origin_block_x, grid.origin_block_z, i64::from(grid.r));
        let frozen = |c: usize| {
            let cl = climate(ox + (c % gw) as i64 * r, oz + (c / gw) as i64 * r);
            view.hydro.frozen(cl.t_mean_c)
        };
        let size_px = (out.w, out.h);
        out.lake = lake_cover(&grid, &|c| levels[c] != NODATA && !frozen(c), size_px);
        out.frozen = lake_cover(&grid, &|c| levels[c] != NODATA && frozen(c), size_px);
        // The bake stores no playas (they hold no water): the frame's own.
        if let Some(derived) = derive(world, view, size) {
            out.playa = derived.playa;
            for (p, (&a, &b)) in out.playa.iter_mut().zip(out.lake.iter().zip(&out.frozen)) {
                *p = p.min(1.0 - a - b).max(0.0);
            }
        }
    }
    let look = Look::default();
    out.draw_rivers(&rivers, view, &look);
    out.finish(&rivers, Source::Bake, started);
    Ok(Some(out))
}

#[cfg(test)]
mod tests;

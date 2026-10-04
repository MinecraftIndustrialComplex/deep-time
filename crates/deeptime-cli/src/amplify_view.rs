// SPDX-License-Identifier: LGPL-3.0-or-later
//! What the viewer shows of the amplifier (P1-06): hill-shaded before/after views of a few
//! places, the same 512 × 512 cells of L3 (0.6 km at 16k quick) twice:
//!
//! - **before** is the young landscape's L2 terrain upsampled to L3 by the amplifier's own
//!   interpolation (the amplification with no erosion, thermal or deposition pass at all),
//! - **after** is what the amplifier wrote (`rasters/elevation.dtr`, as the game reads it,
//!   quantised to 1/16 block),
//!
//! each with and without the rivers a drainage analysis of that very terrain finds (D8 on the
//! view, areas of at least [`RIVER_KM2`]), plus the change in height between them. The places are
//! picked from the L3 layer itself: the roughest mountains and the second roughest, the coast
//! with the most shoreline, the lower course of the biggest river, the deepest lake, and the seam
//! (a window centred on x = ±C/2, where any wrap error would show).

use std::path::Path;

use deeptime_core::params::PlanetPreset;
use deeptime_core::preset::{Preset, PresetBody};
use deeptime_core::surface::amplify::{resample, tiles};
use deeptime_core::surface::grid::Dims;
use deeptime_core::surface::{pflood, routing, young};
use deeptime_core::world::WorldState;
use deeptime_format::{LoadedLayer, Ramp, Ramps, RasterRead, TilePackReader};
use serde_json::{Value, json};

use crate::run::CliResult;

/// Cells on a side of a view.
pub const SIDE: usize = 512;
/// Vertical exaggeration of the hill-shading (the game's own is 15).
pub const VE: f64 = 6.0;
/// Drainage area from which a cell is drawn as a river, km².
pub const RIVER_KM2: f64 = 80.0;
/// Height change at which the difference image saturates, m.
const DIFF_M: f64 = 60.0;

/// One view.
pub struct Region {
    pub id: &'static str,
    pub title: String,
    pub note: String,
    /// First column and row of the window on L3 (the column may be negative: across the seam).
    pub x0: i64,
    pub y0: i64,
    pub lat: f64,
    pub lon: f64,
    /// Ticks on the picture's edges where tile boundaries cross it: (vertical line?, pixel).
    pub ticks: Vec<(bool, usize)>,
}

/// The rendered pictures of one region (PNG bytes).
pub struct Rendered {
    pub region: Region,
    pub before: Vec<u8>,
    pub after: Vec<u8>,
    pub before_rivers: Vec<u8>,
    pub after_rivers: Vec<u8>,
    pub diff: Vec<u8>,
    /// Mean absolute height change, m; river cells before and after.
    pub mean_abs_change_m: f64,
    pub river_cells: (usize, usize),
}

pub struct View {
    pub regions: Vec<Rendered>,
    pub summary: String,
    pub stats: Vec<(String, String)>,
    pub cell_km: f64,
}

/// L3 height in blocks at (i, j), x wrapped, rows clamped.
struct L3<'a> {
    layer: &'a LoadedLayer,
    w: i64,
    h: i64,
}

impl L3<'_> {
    fn y(&self, i: i64, j: i64) -> f32 {
        (self.layer.raw(i.rem_euclid(self.w), j.clamp(0, self.h - 1)) / 16.0) as f32
    }
}

fn quantise(y: f32) -> f32 {
    (y * 16.0).round() / 16.0
}

/// Cell (i, j) of the L3 grid to a longitude and latitude (the planet's projection).
fn lonlat(grid: &deeptime_format::GridDesc, i: f64, j: f64) -> (f64, f64) {
    (
        deeptime_format::Projection::picture_lon_deg(i, grid.w).rem_euclid(360.0),
        grid.projection().lat_deg(grid.z_of_row(j)),
    )
}

struct Cfg {
    sea_y: f64,
    mv: f64,
    dx: f64,
}

/// Hill-shaded, hypsometrically tinted picture of `y` (block heights), with `rivers` (drainage
/// area in km² per cell) drawn over it when given.
fn render(
    y: &[f32],
    w: usize,
    h: usize,
    cfg: &Cfg,
    ramp: &Ramp,
    rivers: Option<&[f32]>,
    ticks: &[(bool, usize)],
) -> Vec<u8> {
    let light = {
        let (x, yy, z) = (-0.6f64, 0.6f64, 0.7f64);
        let n = (x * x + yy * yy + z * z).sqrt();
        (x / n, yy / n, z / n)
    };
    let flat = light.2;
    let mut out = vec![0u8; w * h * 4];
    let at = |i: usize, j: usize| f64::from(y[j.min(h - 1) * w + i.min(w - 1)]);
    for j in 0..h {
        for i in 0..w {
            let yc = at(i, j);
            let tint = ramp.colour(yc);
            let (mut r, mut g, mut b) =
                (f64::from(tint[0]), f64::from(tint[1]), f64::from(tint[2]));
            if yc > cfg.sea_y {
                let gx = (at(i + 1, j) - at(i.saturating_sub(1), j)) * cfg.mv / (2.0 * cfg.dx);
                let gs = (at(i, j + 1) - at(i, j.saturating_sub(1))) * cfg.mv / (2.0 * cfg.dx);
                let n = (-VE * gx, VE * gs, 1.0f64);
                let len = (n.0 * n.0 + n.1 * n.1 + n.2 * n.2).sqrt();
                let shade = ((n.0 * light.0 + n.1 * light.1 + n.2 * light.2) / len).max(0.0) / flat;
                let k = (0.3 + 0.7 * shade).clamp(0.2, 1.3);
                r = (r * k).min(255.0);
                g = (g * k).min(255.0);
                b = (b * k).min(255.0);
            }
            let c = (j * w + i) * 4;
            out[c] = r as u8;
            out[c + 1] = g as u8;
            out[c + 2] = b as u8;
            out[c + 3] = 255;
        }
    }
    // Red ticks on the edges where tile boundaries cross the picture.
    for &(vertical, at) in ticks {
        for d in 0..14 {
            for t in -1i64..=1 {
                let m = (at as i64 + t).clamp(0, if vertical { w } else { h } as i64 - 1) as usize;
                let cells = if vertical {
                    [(m, d), (m, h - 1 - d)]
                } else {
                    [(d, m), (w - 1 - d, m)]
                };
                for (i, j) in cells {
                    let c = (j * w + i) * 4;
                    out[c..c + 4].copy_from_slice(&[255, 40, 40, 255]);
                }
            }
        }
    }
    if let Some(area) = rivers {
        for c in 0..w * h {
            let a = f64::from(area[c]);
            if a >= RIVER_KM2 && f64::from(y[c]) > cfg.sea_y {
                let alpha = (0.45 + 0.2 * (a / RIVER_KM2).log10()).clamp(0.0, 0.95);
                let px = &mut out[c * 4..c * 4 + 3];
                let river = [34.0, 128.0, 255.0];
                for (p, r) in px.iter_mut().zip(river) {
                    *p = (f64::from(*p) * (1.0 - alpha) + r * alpha) as u8;
                }
            }
        }
    }
    out
}

/// Drainage area in km² of every cell of the view `y` (D8 on the priority-flooded terrain; the
/// window's border and the sea are outlets).
fn river_areas(y: &[f32], w: usize, h: usize, cfg: &Cfg) -> Vec<f32> {
    let n = w * h;
    let d = Dims::new(w, h, cfg.dx, false);
    let z: Vec<f64> = y
        .iter()
        .map(|&v| (f64::from(v) - cfg.sea_y) * cfg.mv)
        .collect();
    let mut kind = vec![pflood::LAND; n];
    let mut base = vec![0.0f64; n];
    for c in 0..n {
        let (i, j) = (c % w, c / w);
        let edge = i == 0 || j == 0 || i + 1 == w || j + 1 == h;
        if z[c] <= 0.0 {
            kind[c] = pflood::BASE;
        } else if edge {
            kind[c] = pflood::BASE;
            base[c] = z[c];
        }
    }
    let mut filled = vec![0.0; n];
    pflood::flood(&d, &z, &base, &mut kind, &mut filled);
    let mut rt = routing::Routing::default();
    routing::receivers(&d, &filled, &kind, &mut rt);
    routing::stack(&d, &kind, &mut rt);
    let ones = vec![1.0f32; n];
    let mut area = Vec::new();
    routing::accumulate(&d, &rt, &ones, &mut area);
    area.iter().map(|&a| (a / 1.0e6) as f32).collect()
}

/// The height change (m) as a diverging picture: blue where the amplifier cut, red where it
/// raised.
fn diff_image(before: &[f32], after: &[f32], w: usize, h: usize, cfg: &Cfg) -> (Vec<u8>, f64) {
    let mut out = vec![0u8; w * h * 4];
    let (mut sum, mut n) = (0.0f64, 0usize);
    for c in 0..w * h {
        let d = (f64::from(after[c]) - f64::from(before[c])) * cfg.mv;
        let land = f64::from(after[c]) > cfg.sea_y || f64::from(before[c]) > cfg.sea_y;
        let px = &mut out[c * 4..c * 4 + 4];
        if !land {
            px.copy_from_slice(&[24, 26, 32, 255]);
            continue;
        }
        sum += d.abs();
        n += 1;
        let t = (d / DIFF_M).clamp(-1.0, 1.0);
        let (from, to) = if t < 0.0 {
            ([247.0, 247.0, 247.0], [33.0, 102.0, 172.0])
        } else {
            ([247.0, 247.0, 247.0], [178.0, 24.0, 43.0])
        };
        for k in 0..3 {
            px[k] = (from[k] + (to[k] - from[k]) * t.abs()) as u8;
        }
        px[3] = 255;
    }
    (out, sum / n.max(1) as f64)
}

/// The block-height window of L2 `z` (metres) upsampled to L3 for the cells (x0.., y0..).
#[allow(clippy::too_many_arguments)]
fn before_window(
    z2: &[f32],
    w2: usize,
    h2: usize,
    scale: i64,
    x0: i64,
    y0: i64,
    cfg: &Cfg,
    sea: f64,
) -> Vec<f32> {
    // A margin of 4 L2 cells: the two interpolations wrap at the extracted window's own edges,
    // and their reach (a few L2 cells) stays inside the margin.
    let m = 4i64;
    let cx0 = x0.div_euclid(scale) - m;
    let cy0 = y0.div_euclid(scale) - m;
    let cw = (SIDE as i64 / scale + 2 * m + 2) as usize;
    let ch = cw;
    let mut win = vec![0.0f32; cw * ch];
    for j in 0..ch {
        let sj = (cy0 + j as i64).clamp(0, h2 as i64 - 1) as usize;
        for i in 0..cw {
            let si = (cx0 + i as i64).rem_euclid(w2 as i64) as usize;
            win[j * cw + i] = z2[sj * w2 + si];
        }
    }
    let (mut cur, mut w, mut h) = (win, cw, ch);
    let mut s = 1i64;
    while s < scale {
        cur = resample::upsample2_z(&cur, w, h);
        w *= 2;
        h *= 2;
        s *= 2;
    }
    let ox = (x0 - cx0 * scale) as usize;
    let oy = (y0 - cy0 * scale) as usize;
    let mut out = Vec::with_capacity(SIDE * SIDE);
    for j in 0..SIDE {
        for i in 0..SIDE {
            let zm = f64::from(cur[(oy + j) * w + ox + i]);
            out.push(quantise((cfg.sea_y + (zm - sea) / cfg.mv) as f32));
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn pick_regions(
    l3: &L3<'_>,
    gw: usize,
    gh: usize,
    z2: &[f32],
    lake2: &[f32],
    area2: &[f32],
    w2: usize,
    sea: f64,
    cfg: &Cfg,
    repair_at: Option<(i64, i64, f64)>,
    tile: Option<usize>,
) -> Vec<Region> {
    let (bw, bh) = (gw / SIDE, gh / SIDE);
    // Per block: land share, mean gradient over land (blocks per 4 cells), shoreline cells.
    struct Block {
        bi: usize,
        bj: usize,
        land: f64,
        grad: f64,
        coast: usize,
    }
    let mut blocks = Vec::new();
    for bj in 0..bh {
        for bi in 0..bw {
            let (mut land, mut n, mut grad, mut coast) = (0usize, 0usize, 0.0f64, 0usize);
            for j in (0..SIDE).step_by(4) {
                for i in (0..SIDE).step_by(4) {
                    let (gi, gj) = ((bi * SIDE + i) as i64, (bj * SIDE + j) as i64);
                    let y = f64::from(l3.y(gi, gj));
                    n += 1;
                    if y > cfg.sea_y {
                        land += 1;
                        grad += f64::from((l3.y(gi + 4, gj) - l3.y(gi, gj)).abs())
                            + f64::from((l3.y(gi, gj + 4) - l3.y(gi, gj)).abs());
                        if f64::from(l3.y(gi + 4, gj)) <= cfg.sea_y
                            || f64::from(l3.y(gi, gj + 4)) <= cfg.sea_y
                        {
                            coast += 1;
                        }
                    }
                }
            }
            blocks.push(Block {
                bi,
                bj,
                land: land as f64 / n as f64,
                grad: grad / land.max(1) as f64,
                coast,
            });
        }
    }
    let mut regions: Vec<Region> = Vec::new();
    let place = |id, title: &str, note: String, x0: i64, y0: i64| -> Region {
        let y0 = y0.clamp(0, gh as i64 - SIDE as i64);
        let (lon, lat) = lonlat(
            &l3.layer.desc.grid,
            (x0 + SIDE as i64 / 2) as f64,
            (y0 + SIDE as i64 / 2) as f64,
        );
        Region {
            id,
            title: title.to_owned(),
            note,
            x0,
            y0,
            lat,
            lon,
            ticks: Vec::new(),
        }
    };
    let far = |regions: &[Region], x0: i64, y0: i64| {
        regions.iter().all(|r| {
            let dx = (r.x0 - x0)
                .rem_euclid(gw as i64)
                .min((x0 - r.x0).rem_euclid(gw as i64));
            dx >= SIDE as i64 * 3 / 4 || (r.y0 - y0).abs() >= SIDE as i64 * 3 / 4
        })
    };
    // Mountains: the roughest ground of a mostly land block, and the next roughest one apart.
    let mut rough: Vec<&Block> = blocks.iter().filter(|b| b.land >= 0.6).collect();
    rough.sort_by(|a, b| {
        b.grad
            .total_cmp(&a.grad)
            .then(a.bi.cmp(&b.bi))
            .then(a.bj.cmp(&b.bj))
    });
    for (k, (id, title)) in [
        ("mountains", "The roughest mountains"),
        ("highlands", "More rough highlands"),
    ]
    .into_iter()
    .enumerate()
    {
        for b in &rough {
            let (x0, y0) = ((b.bi * SIDE) as i64, (b.bj * SIDE) as i64);
            if far(&regions, x0, y0) {
                regions.push(place(
                    id,
                    title,
                    format!(
                        "mean slope of the land {:.2} blocks per 4 cells",
                        b.grad / 2.0
                    ),
                    x0,
                    y0,
                ));
                break;
            }
        }
        let _ = k;
    }
    // Coast: the most shoreline.
    if let Some(b) = blocks
        .iter()
        .filter(|b| (0.3..=0.7).contains(&b.land))
        .max_by_key(|b| (b.coast, std::cmp::Reverse((b.bi, b.bj))))
    {
        let (x0, y0) = ((b.bi * SIDE) as i64, (b.bj * SIDE) as i64);
        if far(&regions, x0, y0) {
            regions.push(place(
                "coast",
                "A coast",
                format!("{} shoreline samples", b.coast),
                x0,
                y0,
            ));
        }
    }
    // The lower course of the biggest river: the largest area on land between 20 and 600 m.
    let centre = |c: usize| -> (i64, i64) {
        let (i, j) = ((c % w2) as i64, (c / w2) as i64);
        let s = (gw / w2) as i64;
        (
            i * s + s / 2 - SIDE as i64 / 2,
            j * s + s / 2 - SIDE as i64 / 2,
        )
    };
    let mut rivers: Vec<usize> = (0..z2.len())
        .filter(|&c| {
            let up = f64::from(z2[c]) - sea;
            (20.0..600.0).contains(&up)
        })
        .collect();
    rivers.sort_by(|&a, &b| area2[b].total_cmp(&area2[a]).then(a.cmp(&b)));
    for &c in rivers.iter().take(4000) {
        let (x0, y0) = centre(c);
        if far(&regions, x0, y0) {
            regions.push(place(
                "river",
                "The lower course of the biggest river",
                format!("{:.0} km² drain through the centre", area2[c] / 1.0e6),
                x0,
                y0,
            ));
            break;
        }
    }
    // The deepest lake.
    let lake = (0..lake2.len())
        .filter(|&c| f64::from(z2[c]) > sea + 1.0)
        .max_by(|&a, &b| lake2[a].total_cmp(&lake2[b]).then(b.cmp(&a)));
    if let Some(c) = lake.filter(|&c| lake2[c] > 0.05) {
        let (x0, y0) = centre(c);
        if far(&regions, x0, y0) {
            regions.push(place(
                "lake",
                "The deepest lake",
                format!("{:.0} m deep", lake2[c]),
                x0,
                y0,
            ));
        }
    }
    // Where the flood raised the ground most (the repair pass's largest act).
    if let Some((i, j, m)) = repair_at.filter(|r| r.2 > 5.0) {
        let (x0, y0) = (i - SIDE as i64 / 2, j - SIDE as i64 / 2);
        if far(&regions, x0, y0) {
            regions.push(place(
                "repair",
                "Where the flood raised the ground most",
                format!("{m:.0} m of fill, the largest of the repair pass"),
                x0,
                y0,
            ));
        }
    }
    // Where the tiles of a streamed run meet: the window round a tile boundary whose line runs
    // over the highest ground.
    if let Some(t) = tile.filter(|&t| t > 0 && t < gh && t < gw) {
        let mut best = (0.0f64, 0i64, 0i64);
        let half = SIDE as i64 / 2;
        let mut consider = |cx: i64, cy: i64, vertical: bool| {
            let mut score = 0.0f64;
            for d in (-half..half).step_by(4) {
                let y = if vertical {
                    l3.y(cx, cy + d)
                } else {
                    l3.y(cx + d, cy)
                };
                score += (f64::from(y) - cfg.sea_y).max(0.0);
            }
            if score > best.0 {
                best = (score, cx, cy);
            }
        };
        for k in 1..gw.div_ceil(t) {
            for j in (half..gh as i64 - half).step_by(64) {
                consider((k * t) as i64, j, true);
            }
        }
        for m in 1..gh.div_ceil(t) {
            for i in (0..gw as i64).step_by(64) {
                consider(i, (m * t) as i64, false);
            }
        }
        if best.0 > 0.0 {
            let (cx, cy) = (best.1, best.2);
            let (x0, y0) = (cx - half, (cy - half).clamp(0, gh as i64 - SIDE as i64));
            let mut r = place(
                "tiles",
                "Across the boundaries of the streamed run's tiles",
                "each tile is computed in its own window with a halo; the red ticks on the \
                 edges mark where tile boundaries cross the picture"
                    .into(),
                x0,
                y0,
            );
            // Boundaries inside the window: multiples of the tile size, in pixels.
            for k in 1..gw.div_ceil(t) {
                let px = (k * t) as i64 - x0;
                if (0..SIDE as i64).contains(&px) {
                    r.ticks.push((true, px as usize));
                }
            }
            for m in 1..gh.div_ceil(t) {
                let py = (m * t) as i64 - r.y0;
                if (0..SIDE as i64).contains(&py) {
                    r.ticks.push((false, py as usize));
                }
            }
            regions.push(r);
        }
    }
    // The seam: the window centred on x = ±C/2 with the most land.
    let mut best = (0usize, 0i64);
    for j0 in (0..=gh - SIDE).step_by(32) {
        // Within 65° of the equator (the polar caps are flat and D8 draws straight rivers there).
        let g = &l3.layer.desc.grid;
        let centre_lat = g.projection().lat_deg(g.z_of_row((j0 + SIDE / 2) as f64));
        if centre_lat.abs() > 65.0 {
            continue;
        }
        let mut land = 0usize;
        for j in (0..SIDE).step_by(8) {
            for i in (0..SIDE).step_by(8) {
                let gi = i as i64 - SIDE as i64 / 2;
                if f64::from(l3.y(gi, (j0 + j) as i64)) > cfg.sea_y {
                    land += 1;
                }
            }
        }
        if land > best.0 {
            best = (land, j0 as i64);
        }
    }
    regions.push(place(
        "seam",
        "Across the seam (x = ±C/2 in the middle)",
        "the wrap must leave no mark".into(),
        -(SIDE as i64) / 2,
        best.1,
    ));
    regions
}

fn field<'a>(world: &'a WorldState, name: &str) -> CliResult<&'a Vec<f32>> {
    world
        .fields
        .get(name)
        .map(|f| &f.data)
        .ok_or_else(|| format!("the world has no field {name}").into())
}

/// The view of a world whose pipeline ran the amplify stage, read from the layer in `work`.
pub fn build(world: &WorldState, preset_json: &str, work: &Path) -> CliResult<Option<View>> {
    let Some(note_text) = world.notes.get(deeptime_core::surface::amplify::STAGE_NAME) else {
        return Ok(None);
    };
    let note: Value = serde_json::from_str(note_text)?;
    let preset = Preset::from_json(preset_json)?;
    let PresetBody::Planet(p) = &preset.body else {
        return Ok(None);
    };
    let p: &PlanetPreset = p;
    let path = work.join(tiles::ELEVATION_PATH);
    if !path.exists() {
        return Ok(None);
    }
    let layer = TilePackReader::open(&path)?.load()?;
    let (gw, gh) = (layer.desc.grid.w as usize, layer.desc.grid.h as usize);
    let l3 = L3 {
        layer: &layer,
        w: gw as i64,
        h: gh as i64,
    };
    let s = &p.scale;
    let cfg = Cfg {
        sea_y: f64::from(s.sea_level_y),
        mv: s.m_per_block_v,
        dx: f64::from(layer.desc.grid.r) * s.m_per_block_h,
    };
    let sea = world.surface.as_ref().map_or(0.0, |st| st.sea_level_m);
    let z2 = field(world, young::field::ELEVATION_M)?;
    let lake2 = field(world, young::field::LAKE_M)?;
    let area2 = field(world, young::field::AREA_M2)?;
    let g2 = &world.fields[young::field::ELEVATION_M].grid;
    let (w2, h2) = (g2.w as usize, g2.h as usize);
    let scale = i64::from(g2.r / layer.desc.grid.r);
    if gw < SIDE * 2 || gh < SIDE * 2 {
        return Ok(None);
    }
    let repair_at = note["flood_max_raise_at"].as_array().and_then(|a| {
        Some((
            a.first()?.as_i64()?,
            a.get(1)?.as_i64()?,
            note["flood_max_raise_m"].as_f64()?,
        ))
    });
    let tile = (note["mode"] == "streamed")
        .then(|| note["tile_cells"].as_u64())
        .flatten()
        .map(|t| t as usize);
    let regions = pick_regions(
        &l3, gw, gh, z2, lake2, area2, w2, sea, &cfg, repair_at, tile,
    );
    let ramps = Ramps::builtin();
    let (_, ramp) = ramps.for_layer("elevation");
    let mut rendered = Vec::new();
    for region in regions {
        let mut after = Vec::with_capacity(SIDE * SIDE);
        for j in 0..SIDE {
            for i in 0..SIDE {
                after.push(l3.y(region.x0 + i as i64, region.y0 + j as i64));
            }
        }
        let before = before_window(z2, w2, h2, scale, region.x0, region.y0, &cfg, sea);
        let ra = river_areas(&after, SIDE, SIDE, &cfg);
        let rb = river_areas(&before, SIDE, SIDE, &cfg);
        let count = |a: &[f32], y: &[f32]| {
            a.iter()
                .zip(y)
                .filter(|&(&a, &y)| f64::from(a) >= RIVER_KM2 && f64::from(y) > cfg.sea_y)
                .count()
        };
        let (diff, mean_abs) = diff_image(&before, &after, SIDE, SIDE, &cfg);
        let png = |rgba: Vec<u8>| deeptime_viewer::encode_png(SIDE as u32, SIDE as u32, &rgba);
        rendered.push(Rendered {
            before: png(render(&before, SIDE, SIDE, &cfg, ramp, None, &region.ticks))?,
            after: png(render(&after, SIDE, SIDE, &cfg, ramp, None, &region.ticks))?,
            before_rivers: png(render(
                &before,
                SIDE,
                SIDE,
                &cfg,
                ramp,
                Some(&rb),
                &region.ticks,
            ))?,
            after_rivers: png(render(
                &after,
                SIDE,
                SIDE,
                &cfg,
                ramp,
                Some(&ra),
                &region.ticks,
            ))?,
            diff: png(diff)?,
            mean_abs_change_m: mean_abs,
            river_cells: (count(&rb, &before), count(&ra, &after)),
            region,
        });
    }
    let f = |v: &Value| v.as_f64().unwrap_or(0.0);
    let mut stats = vec![
        (
            "L3 grid".to_owned(),
            format!("{gw} × {gh} cells of {:.0} m", cfg.dx),
        ),
        (
            "last level".to_owned(),
            format!(
                "{}, {} tile(s), halo {} cells",
                note["mode"].as_str().unwrap_or("?").replace('_', " "),
                note["tiles"],
                note["halo"]
            ),
        ),
        (
            "new sinks outside lakes".to_owned(),
            format!("{}", note["sinks"]),
        ),
        (
            "flood repairs".to_owned(),
            format!(
                "{} cells raised, at most {:.2} m",
                note["flood_raised_cells"],
                f(&note["flood_max_raise_m"])
            ),
        ),
        (
            "eroded".to_owned(),
            format!(
                "{:.3e} km³ ({:.3e} km³ left in transit)",
                f(&note["eroded_km3"]),
                f(&note["lost_in_transit_km3"])
            ),
        ),
        ("drama".to_owned(), format!("{}", note["drama"])),
    ];
    if let Some(y) = world.notes.get(young::STAGE_NAME)
        && let Ok(v) = serde_json::from_str::<Value>(y)
    {
        stats.push(("young landscape".to_owned(), young::summary_line(&v)));
    }
    let summary = deeptime_core::surface::amplify::summary_line(&note);
    Ok(Some(View {
        regions: rendered,
        summary,
        stats,
        cell_km: cfg.dx / 1000.0,
    }))
}

/// Writes the pictures into `dir/amplify/` and returns the report's `amplify` object.
pub fn write(dir: &Path, v: &View) -> CliResult<Value> {
    let out = dir.join("amplify");
    std::fs::create_dir_all(&out)?;
    let mut regions = Vec::new();
    for r in &v.regions {
        let id = r.region.id;
        let mut files = serde_json::Map::new();
        for (key, bytes) in [
            ("before", &r.before),
            ("after", &r.after),
            ("before_rivers", &r.before_rivers),
            ("after_rivers", &r.after_rivers),
            ("diff", &r.diff),
        ] {
            let name = format!("amplify/{id}-{key}.png");
            std::fs::write(dir.join(&name), bytes)?;
            files.insert(key.to_owned(), json!(name));
        }
        regions.push(json!({
            "id": id, "title": r.region.title, "note": r.region.note,
            "lat": r.region.lat, "lon": r.region.lon, "w": SIDE, "h": SIDE,
            "km": SIDE as f64 * v.cell_km,
            "mean_abs_change_m": r.mean_abs_change_m,
            "river_cells": [r.river_cells.0, r.river_cells.1],
            "files": files,
        }));
    }
    Ok(json!({
        "summary": v.summary,
        "ve": VE,
        "river_km2": RIVER_KM2,
        "diff_m": DIFF_M,
        "cell_km": v.cell_km,
        "stats": v.stats.iter().map(|(a, b)| json!([a, b])).collect::<Vec<_>>(),
        "regions": regions,
    }))
}

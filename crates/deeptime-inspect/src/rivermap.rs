// SPDX-License-Identifier: LGPL-3.0-or-later
//! River and lake maps straight from the world data (P1-22), no region files needed:
//!
//! - [`planet`]: the whole planet (or a box) at N blocks a pixel: hill-shaded land tinted by
//!   height, the sea by depth, lakes from `water_level`, every river segment drawn at its width
//!   (at least a pixel) in a blue that darkens with discharge.
//! - [`columns`]: a box at one block a pixel (times `scale`) with what the column evaluator
//!   decides for each column (`deeptime_format::water`): river, lake, sea, bank, dry ground,
//!   over the hill-shaded generated surface; the exact picture of what the game's pass builds.

use std::path::Path;

use deeptime_format::rivers::MIN_HALF_WIDTH;
use deeptime_format::water::{self, WaterKind};
use deeptime_format::{ColumnWorld, RasterRead};
use serde_json::{Value, json};

use crate::error::{Error, Result};
use crate::image::{Image, Rgba, mix, shade};

const SEA_SHALLOW: Rgba = [70, 130, 190, 255];
const SEA_DEEP: Rgba = [18, 40, 90, 255];
const LAKE: Rgba = [40, 110, 210, 255];
const RIVER_SMALL: Rgba = [60, 150, 235, 255];
const RIVER_BIG: Rgba = [10, 60, 170, 255];
const BANK: Rgba = [205, 180, 120, 255];

fn land(h: f64, sea: f64) -> Rgba {
    // A hypsometric ramp in blocks above the sea (20 m a block).
    let stops: [(f64, Rgba); 6] = [
        (0.0, [96, 150, 80, 255]),
        (15.0, [140, 170, 95, 255]),
        (40.0, [190, 180, 120, 255]),
        (90.0, [160, 120, 80, 255]),
        (180.0, [130, 110, 100, 255]),
        (300.0, [240, 240, 245, 255]),
    ];
    let a = h - sea;
    for w in stops.windows(2) {
        let ((a0, c0), (a1, c1)) = (w[0], w[1]);
        if a <= a1 {
            let t = ((a - a0) / (a1 - a0)).clamp(0.0, 1.0);
            return mix(c1, c0, t);
        }
    }
    stops[5].1
}

fn hillshade(hx: f64, hz: f64, step: f64) -> f64 {
    // Light from the north-west; slopes in blocks per block (vertical exaggeration 3 for a map).
    let (dx, dz) = (hx * 3.0 / step, hz * 3.0 / step);
    let n = (dx * dx + dz * dz + 1.0).sqrt();
    let l = (-dx * -0.6 + -dz * -0.6 + 0.53) / n;
    (0.55 + 0.6 * l).clamp(0.35, 1.25)
}

/// The planet (or `bbox` = x0, z0, x1, z1) at `step` blocks a pixel.
pub fn planet(dir: &Path, out: &Path, step: usize, bbox: Option<[i64; 4]>) -> Result<Value> {
    let w = ColumnWorld::open(dir).map_err(|e| Error(e.to_string()))?;
    let c = w.c;
    // Pole to pole (the projection's z of the poles).
    let zp = w.manifest.world_projection().z_range().1 as i64;
    let [x0, z0, x1, z1] = bbox.unwrap_or([-c / 2, -zp, c / 2, zp]);
    let step = step.max(1) as i64;
    let (pw, ph) = (((x1 - x0) / step) as usize, ((z1 - z0) / step) as usize);
    if pw == 0 || ph == 0 || pw * ph > 40_000_000 {
        return Err(Error(format!(
            "a {pw} × {ph} image: pick another box or step"
        )));
    }
    let sea = f64::from(w.manifest.scale.sea_level_y);
    let levels = deeptime_format::LazyLayer::open(
        &dir.join(
            &w.manifest
                .layer("water_level")
                .ok_or_else(|| Error("no water_level layer".into()))?
                .path,
        ),
    )
    .map_err(|e| Error(e.to_string()))?;
    let mut hts = vec![0f64; (pw + 2) * (ph + 2)];
    for py in 0..ph + 2 {
        for px in 0..pw + 2 {
            let bx = x0 + (px as i64 - 1) * step + step / 2;
            let bz = (z0 + (py as i64 - 1) * step + step / 2).clamp(-zp, zp - 1);
            hts[py * (pw + 2) + px] = w.height(bx, bz);
        }
    }
    let mut img = Image::new(pw, ph, [0, 0, 0, 255]);
    let (mut lake_px, mut land_px) = (0u64, 0u64);
    for py in 0..ph {
        for px in 0..pw {
            let at = |dx: i64, dy: i64| {
                hts[(py as i64 + 1 + dy) as usize * (pw + 2) + (px as i64 + 1 + dx) as usize]
            };
            let h = at(0, 0);
            let bx = x0 + px as i64 * step + step / 2;
            let bz = z0 + py as i64 * step + step / 2;
            let wl = levels.nearest(w.wrap_x(bx) as f64, bz as f64);
            let col = if h <= sea - 1.0 {
                mix(SEA_DEEP, SEA_SHALLOW, ((sea - h) / 150.0).clamp(0.0, 1.0))
            } else if wl.is_finite() && wl > h {
                lake_px += 1;
                LAKE
            } else {
                land_px += 1;
                let s = hillshade(
                    (at(1, 0) - at(-1, 0)) / 2.0,
                    (at(0, 1) - at(0, -1)) / 2.0,
                    step as f64,
                );
                shade(land(h, sea), s)
            };
            img.set(px, py, col);
        }
    }
    // Rivers on top: each segment as a thick line.
    let mut segments = 0u64;
    if let Some(r) = &w.rivers {
        let qmax = r.vertices.iter().map(|v| v.q).fold(1.0f32, f32::max);
        for ch in &r.chains {
            for s in ch.first as usize..(ch.first + ch.count - 1) as usize {
                let (a, b) = (&r.vertices[s], &r.vertices[s + 1]);
                let hw = f64::from(a.hw.max(b.hw)).max(MIN_HALF_WIDTH);
                let t = (f64::from(a.q.max(b.q)).ln() / f64::from(qmax).ln()).clamp(0.0, 1.0);
                let col = mix(RIVER_BIG, RIVER_SMALL, t);
                let rad = (hw / step as f64).max(0.5);
                // Try the lap of the box.
                for lap in [-c, 0, c] {
                    let (ax, az) = (f64::from(a.x) + lap as f64, f64::from(a.z));
                    let (bx, bz) = (f64::from(b.x) + lap as f64, f64::from(b.z));
                    let to_px = |x: f64, z: f64| {
                        ((x - x0 as f64) / step as f64, (z - z0 as f64) / step as f64)
                    };
                    let (pa, pb) = (to_px(ax, az), to_px(bx, bz));
                    if pa.0.max(pb.0) < -rad - 1.0 || pa.0.min(pb.0) > pw as f64 + rad + 1.0 {
                        continue;
                    }
                    segments += 1;
                    stroke(&mut img, pa, pb, rad, col);
                }
            }
        }
    }
    img.save_png(out)?;
    Ok(json!({
        "png": out.display().to_string(),
        "box": [x0, z0, x1, z1],
        "step": step,
        "size": [pw, ph],
        "lake_px": lake_px,
        "land_px": land_px,
        "segments_drawn": segments,
        "chains": w.rivers.as_ref().map_or(0, |r| r.chains.len()),
    }))
}

/// A thick line from `a` to `b` (pixel coordinates) of radius `rad`.
fn stroke(img: &mut Image, a: (f64, f64), b: (f64, f64), rad: f64, col: Rgba) {
    let (x0, x1) = (a.0.min(b.0) - rad, a.0.max(b.0) + rad);
    let (y0, y1) = (a.1.min(b.1) - rad, a.1.max(b.1) + rad);
    let (ex, ey) = (b.0 - a.0, b.1 - a.1);
    let l2 = ex * ex + ey * ey;
    for py in (y0.floor().max(0.0) as i64)..=(y1.ceil().min(img.h as f64 - 1.0) as i64) {
        for px in (x0.floor().max(0.0) as i64)..=(x1.ceil().min(img.w as f64 - 1.0) as i64) {
            let (cx, cy) = (px as f64 + 0.5, py as f64 + 0.5);
            let t = if l2 > 0.0 {
                (((cx - a.0) * ex + (cy - a.1) * ey) / l2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let (dx, dy) = (cx - (a.0 + t * ex), cy - (a.1 + t * ey));
            if dx * dx + dy * dy <= rad * rad {
                img.set(px as usize, py as usize, col);
            }
        }
    }
}

/// Box [x0, x1) × [z0, z1) at one block a pixel (× `scale`): each column's water as the game
/// builds it. Returns counts per kind.
pub fn columns(dir: &Path, out: &Path, bbox: [i64; 4], scale: usize) -> Result<Value> {
    let w = ColumnWorld::open(dir).map_err(|e| Error(e.to_string()))?;
    let [x0, z0, x1, z1] = bbox;
    let (pw, ph) = ((x1 - x0) as usize, (z1 - z0) as usize);
    if pw == 0 || ph == 0 || pw * ph > 4_000_000 {
        return Err(Error(format!("a {pw} × {ph} box: at most 4 M columns")));
    }
    let spec = w.water_spec();
    let sea = f64::from(spec.sea_level_y);
    // Cores over the box and a two-column margin (what a river neighbour holds reads its own
    // neighbours), then each column from its neighbours.
    let (ew, eh) = (pw + 4, ph + 4);
    let cores: Vec<water::Core> = (0..ew * eh)
        .map(|k| {
            let (x, z) = (x0 - 2 + (k % ew) as i64, z0 - 2 + (k / ew) as i64);
            water::core(&w, &spec, w.c, x, z)
        })
        .collect();
    let at = |i: usize, j: usize| cores[j * ew + i];
    let around = |i: usize, j: usize| {
        water::NEIGHBOURS.map(|(dx, dz)| at((i as i64 + dx) as usize, (j as i64 + dz) as usize))
    };
    let mut img = Image::new(pw, ph, [0, 0, 0, 255]);
    let mut counts = std::collections::BTreeMap::<&str, u64>::new();
    let mut biome = 0u64;
    for pz in 0..ph {
        for px in 0..pw {
            let (i, j) = (px + 2, pz + 2);
            let k = j * ew + i;
            let n = around(i, j);
            let holds = water::NEIGHBOURS.map(|(dx, dz)| {
                let (ni, nj) = ((i as i64 + dx) as usize, (j as i64 + dz) as usize);
                water::held(&at(ni, nj), &around(ni, nj))
            });
            let col = water::finish(&cores[k], &n, &holds);
            *counts.entry(col.kind.name()).or_default() += 1;
            biome += u64::from(col.river_biome);
            let s = cores[k].surface;
            let hx = cores[k + 1].surface - cores[k - 1].surface;
            let hz = cores[k + ew].surface - cores[k - ew].surface;
            let shade_f = hillshade(hx / 2.0, hz / 2.0, 1.0);
            let c = match col.kind {
                WaterKind::River => {
                    let q = col.river.map_or(1.0, |h| h.q);
                    let t = (q.ln() / 1000f64.ln()).clamp(0.0, 1.0);
                    mix(RIVER_BIG, RIVER_SMALL, t)
                }
                WaterKind::Lake => LAKE,
                WaterKind::Sea => mix(SEA_DEEP, SEA_SHALLOW, ((sea - s) / 150.0).clamp(0.0, 1.0)),
                WaterKind::Bank => shade(BANK, shade_f),
                WaterKind::None => shade(land(s, sea), shade_f),
            };
            img.set(px, pz, c);
        }
    }
    img.scaled(scale.max(1)).save_png(out)?;
    Ok(json!({
        "png": out.display().to_string(),
        "box": bbox,
        "kinds": counts,
        "river_biome_columns": biome,
    }))
}

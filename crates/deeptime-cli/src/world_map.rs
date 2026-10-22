// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime world-map`: whole-world pictures of a baked world-data folder, in the world's own
//! block space (on a Petroff–Guyou world, format 0.5, the 2b × b rectangle itself), for reviewing a
//! bake without the workbench:
//!
//! - `satellite.png`: the workbench's "Final world" satellite look (`deeptime_tiles`), its tile
//!   pyramid at `--zoom` stitched (2^(z+1) × 2^z tiles of 256 pixels);
//! - `provinces.png`: the geologic provinces (`meta/province`, each id its own colour), the sea
//!   darkened, the cone points (red) and the poles (white) of the world's layout marked;
//! - `world-map.json`: the sizes, the projection block and the marks' pixels.

use std::path::Path;

use deeptime_format::{Manifest, RasterRead, TilePackReader};
use serde_json::{Value, json};

use deeptime_tiles::{BakeMap, Layer, TILE, TileGeom};

/// A colour of its own for each id (splitmix64 of the id).
fn colour(id: u64) -> [u8; 3] {
    let mut x = id.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    [
        80 + (x & 0x9f) as u8,
        80 + ((x >> 8) & 0x9f) as u8,
        80 + ((x >> 16) & 0x9f) as u8,
    ]
}

/// Writes the pictures of world folder `dir` into `out`; returns the summary.
pub fn run(dir: &Path, zoom: u32, out: &Path) -> Result<Value, String> {
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let manifest = Manifest::read(dir).map_err(|e| e.to_string())?;
    let c = manifest.scale.circumference_blocks;
    let map = BakeMap::open(dir)?;
    let (nx, ny) = (2usize << zoom, 1usize << zoom);
    let (w, h) = (nx * TILE, ny * TILE);
    let mut rgba = vec![0u8; w * h * 4];
    for ty in 0..ny {
        for tx in 0..nx {
            let g = TileGeom::new(c, zoom, tx as u32, ty as u32)
                .ok_or_else(|| format!("zoom {zoom} is past this world's pyramid"))?;
            let tile = map.render(Layer::Satellite, &g)?;
            for j in 0..TILE {
                let at = ((ty * TILE + j) * w + tx * TILE) * 4;
                rgba[at..at + TILE * 4].copy_from_slice(&tile[j * TILE * 4..(j + 1) * TILE * 4]);
            }
        }
    }
    let png = deeptime_viewer::encode_png(w as u32, h as u32, &rgba).map_err(|e| e.to_string())?;
    std::fs::write(out.join("satellite.png"), png).map_err(|e| e.to_string())?;

    // Provinces: one pixel per `s` blocks over the 2:1 frame from (−C/2, −C/4).
    let s = f64::from(c) / w as f64;
    let (x0, z0) = (-f64::from(c) / 2.0, -f64::from(c) / 4.0);
    let open = |name: &str| -> Result<Option<deeptime_format::LoadedLayer>, String> {
        match manifest.layer(name) {
            Some(e) => Ok(Some(
                TilePackReader::open(&dir.join(&e.path))
                    .and_then(|r| r.load())
                    .map_err(|e| e.to_string())?,
            )),
            None => Ok(None),
        }
    };
    let prov = open("meta/province")?.ok_or("the world has no meta/province layer")?;
    let elev = open("elevation")?.ok_or("the world has no elevation layer")?;
    let sea = f64::from(manifest.scale.sea_level_y);
    let wp = manifest.world_projection();
    let (zlo, zhi) = wp.z_range();
    let mut rgb = vec![0u8; w * h * 3];
    for j in 0..h {
        let z = z0 + (j as f64 + 0.5) * s;
        for i in 0..w {
            let x = x0 + (i as f64 + 0.5) * s;
            let at = (j * w + i) * 3;
            if z < zlo || z > zhi {
                rgb[at..at + 3].copy_from_slice(&[4, 6, 10]);
                continue;
            }
            let id = prov.nearest(x, z);
            let mut col = if id.is_finite() {
                colour(id as u64)
            } else {
                [60, 60, 60]
            };
            if elev.bilinear(x, z) < sea {
                col = col.map(|v| v / 3);
            }
            rgb[at..at + 3].copy_from_slice(&col);
        }
    }
    // The layout's marks: cone points red, poles white (a small square each).
    let mut marks = Vec::new();
    if let Some(spec) = &manifest.projection {
        let mut put = |bx: f64, bz: f64, col: [u8; 3], what: &str| {
            let pi = ((bx - x0) / s).round() as i64;
            let pj = ((bz - z0) / s).round() as i64;
            for dj in -3..=3 {
                for di in -3..=3 {
                    let (ii, jj) = (pi + di, pj + dj);
                    if ii >= 0 && jj >= 0 && (ii as usize) < w && (jj as usize) < h {
                        let at = (jj as usize * w + ii as usize) * 3;
                        rgb[at..at + 3].copy_from_slice(&col);
                    }
                }
            }
            // Block corners (the polar layouts) as integers, other net points as written.
            let num = |v: f64| {
                if v.fract() == 0.0 {
                    json!(v as i64)
                } else {
                    json!(v)
                }
            };
            marks.push(json!({"what": what, "x": num(bx), "z": num(bz), "px": [pi, pj]}));
        };
        for cp in &spec.cone_points {
            for im in &cp.images {
                put(im[0] as f64, im[1] as f64, [230, 40, 40], &cp.name);
            }
        }
        for p in &spec.poles {
            for im in &p.images {
                put(im[0], im[1], [255, 255, 255], &format!("{} pole", p.name));
            }
        }
    }
    let png =
        deeptime_viewer::encode_png_rgb(w as u32, h as u32, &rgb).map_err(|e| e.to_string())?;
    std::fs::write(out.join("provinces.png"), png).map_err(|e| e.to_string())?;
    let summary = json!({
        "world": dir.display().to_string(),
        "pixels": [w, h],
        "blocks_per_pixel": s,
        "circumference": c,
        "projection": manifest.projection,
        "marks": marks,
    });
    std::fs::write(
        out.join("world-map.json"),
        serde_json::to_vec_pretty(&summary).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(summary)
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! Resamples a baked world-data folder onto a quincuncial world (PROTOTYPE).
//!
//! The output is an ordinary world-data folder that today's Java readers open unchanged: every
//! raster keeps its name, dtype and cell size r, on a strip of circumference C′ (x wraps with period
//! C′, z runs over ±C′/4). Each output cell (a point sample at its block) is mapped to the planet
//! through the quincuncial world's forward map, then to the source's block through the source's own
//! projection (`deeptime_format::Projection`, read-only), and sampled there:
//!
//! - continuous fields bilinearly (the elevation with the source's detail term A·N added at the
//!   source point, so the sub-cell texture is the planet's, not the output grid's);
//! - the weirdness ("ridges") as an odd field, like Java's `bilinearOdd`;
//! - categories, stacks (strata horizons), water levels and ids by the nearest source cell.
//!
//! The forward map is defined on the whole plane, so cells outside the canonical domain get the
//! glued neighbour's planet point: the strip holds the p2 tiling, and the terrain beyond every seam
//! is the terrain the seam's portal shows. C′ must be a translation of the tiling's lattice
//! ([`strip_circumference`]).
//!
//! Dropped, because they are block-space vector data this prototype does not transform: rivers,
//! folds and faults, volcanoes, craters, deposit sites, bodies and events, keyframes. The spawn
//! candidates are carried over through the planet's latitude and longitude.

use std::path::Path;

use deeptime_format::detail::BAKE_NOTES;
use deeptime_format::{
    DType, DetailSpec, FileEntry, GridDesc, LayerDesc, LayerEntry, LoadedLayer, Manifest, Noise,
    Projection, RasterRead, TilePackReader, Writer, write_tile_pack,
};
use rayon::prelude::*;
use serde_json::{Value, json};

use crate::world::QWorld;

/// How a layer is resampled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Bilinear,
    Odd,
    Nearest,
    Elevation,
}

/// The method for a layer name: bilinear for continuous fields, nearest for anything else (safe
/// for categories, ids, stacks and lake surfaces).
pub fn method_of(name: &str) -> Method {
    match name {
        "elevation" => Method::Elevation,
        "climate/ridges" => Method::Odd,
        "climate/t_mean_c" | "climate/p_mm" | "climate/temperature" | "climate/vegetation"
        | "climate/continents" | "climate/erosion" | "climate/ice_cover" | "meta/crust_age_myr"
        | "meta/heat_flow_mw_m2" => {
            Method::Bilinear
        }
        n if n.starts_with("climate/month/") => Method::Bilinear,
        _ => Method::Nearest,
    }
}

/// The strip circumference C′ for a world: 4 × its domain height. A horizontal translation by C′ is a
/// lattice translation of both layouts (Peirce's lattice holds (2a, 0), Guyou's (2b, 0)), and the strip's
/// z range ±C′/4 holds the domain (±a/2 or ±b/2) with as much again of folded outside on each side.
/// That keeps today's latitude-from-z code (the pole cap, `BeyondPoleDensity`) away from the domain:
/// at |z| ≤ C′/8 it reads at most 45°.
pub fn strip_circumference(w: &QWorld) -> f64 {
    let (_, _, z0, z1) = w.extent();
    4.0 * (z1 - z0)
}

fn out_grid(src: &GridDesc, c_out: u32) -> Result<GridDesc, String> {
    if c_out % (2 * src.r) != 0 {
        return Err(format!("C′ {c_out} is not a multiple of 2·r = {}", 2 * src.r));
    }
    let w = c_out / src.r;
    Ok(GridDesc {
        name: src.name.clone(),
        w,
        h: w / 2,
        r: src.r,
        wraps_x: true,
        origin_block_x: -(i64::from(c_out) / 2),
        origin_block_z: -(i64::from(c_out) / 4),
    })
}

fn encode(dtype: DType, desc: &LayerDesc, v: f64, out: &mut [u8]) {
    let raw = if v.is_nan() {
        desc.nodata.unwrap_or(0.0)
    } else {
        ((v - desc.offset) / desc.scale).round()
    };
    match dtype {
        DType::U8 => out[0] = raw.clamp(0.0, 255.0) as u8,
        DType::I8 => out[0] = raw.clamp(-128.0, 127.0) as i8 as u8,
        DType::U16 => out.copy_from_slice(&(raw.clamp(0.0, 65535.0) as u16).to_le_bytes()),
        DType::I16 => out.copy_from_slice(&(raw.clamp(-32768.0, 32767.0) as i16).to_le_bytes()),
        DType::U32 => out.copy_from_slice(&(raw.clamp(0.0, 4.29e9) as u32).to_le_bytes()),
        DType::F32 => out.copy_from_slice(&(v as f32).to_le_bytes()),
        DType::HorizonStack => unreachable!("stacks are copied, not encoded"),
    }
}

fn odd(l: &LoadedLayer, bx: f64, bz: f64) -> f64 {
    let g = &l.desc.grid;
    let u = (bx - g.origin_block_x as f64) / f64::from(g.r);
    let v = (bz - g.origin_block_z as f64) / f64::from(g.r);
    let (fi, fj) = (u.floor(), v.floor());
    let (fu, fv) = (u - fi, v - fj);
    let (i0, j0) = (fi as i64, fj as i64);
    let (v00, v10, v01, v11) = (l.get(i0, j0), l.get(i0 + 1, j0), l.get(i0, j0 + 1), l.get(i0 + 1, j0 + 1));
    let a = v00.abs() + (v10.abs() - v00.abs()) * fu;
    let b = v01.abs() + (v11.abs() - v01.abs()) * fu;
    let m = a + (b - a) * fv;
    let near = if fv < 0.5 { if fu < 0.5 { v00 } else { v10 } } else if fu < 0.5 { v01 } else { v11 };
    m.copysign(near)
}

/// What one resample did.
#[derive(Debug, Default)]
pub struct Report {
    pub layers: Vec<(String, String, u64)>,
    pub dropped: Vec<String>,
    pub spawn_candidates: usize,
}

/// Resamples `src` onto `world` into `out` (created), on a strip of circumference `c_out`.
pub fn resample(src: &Path, out: &Path, world: &QWorld, c_out: u32) -> Result<Report, String> {
    let e = |x: deeptime_format::FormatError| x.to_string();
    let m = Manifest::read(src).map_err(e)?;
    let src_c = m.scale.circumference_blocks;
    let proj = Projection::new(src_c);
    let detail = DetailSpec::of(&m).map_err(e)?;
    let noise = match &detail {
        Some(d) => Some(Noise::new(d.seed().map_err(e)?, src_c)),
        None => None,
    };
    std::fs::create_dir_all(out).map_err(|x| x.to_string())?;
    let mut report = Report::default();

    // Source block of every output cell, per grid (f32 is ample for block coordinates).
    let mut grids = std::collections::BTreeMap::new();
    let mut maps = std::collections::BTreeMap::new();
    for (name, g) in &m.grids {
        let og = out_grid(g, c_out)?;
        let map: Vec<(f32, f32)> = (0..og.cells())
            .into_par_iter()
            .map(|c| {
                let (i, j) = ((c % og.w as usize) as i64, (c / og.w as usize) as i64);
                let (x, z) = (og.block_x(i) as f64, og.block_z(j) as f64);
                let (lat, lon) = world.lat_lon(x, z);
                let (sx, sz) = proj.block_of(lat, lon);
                (sx as f32, sz as f32)
            })
            .collect();
        grids.insert(name.clone(), og);
        maps.insert(name.clone(), map);
    }

    let amplitude = match &detail {
        Some(d) => {
            let le = m.layer(&d.layer).ok_or("the detail layer is not in the manifest")?;
            Some(TilePackReader::open(&src.join(&le.path)).and_then(|r| r.load()).map_err(e)?)
        }
        None => None,
    };

    let mut layers = Vec::new();
    for le in &m.layers {
        if detail.as_ref().is_some_and(|d| d.layer == le.name) {
            report.dropped.push(format!("layer {} (baked into elevation)", le.name));
            continue;
        }
        let layer = TilePackReader::open(&src.join(&le.path)).and_then(|r| r.load()).map_err(e)?;
        let og = grids[&le.grid].clone();
        let map = &maps[&le.grid];
        let mut desc = layer.desc.clone();
        desc.grid = og.clone();
        let cb = desc.cell_bytes();
        let method = if desc.dtype == DType::HorizonStack { Method::Nearest } else { method_of(&le.name) };
        let sg = layer.desc.grid.clone();
        let mut bytes = vec![0u8; og.cells() * cb];
        bytes.par_chunks_mut(cb).enumerate().for_each(|(c, cell)| {
            let (sx, sz) = (f64::from(map[c].0), f64::from(map[c].1));
            match method {
                Method::Nearest => {
                    let u = (sx - sg.origin_block_x as f64) / f64::from(sg.r);
                    let v = (sz - sg.origin_block_z as f64) / f64::from(sg.r);
                    let (i, j) = (sg.wrap_i((u + 0.5).floor() as i64), sg.clamp_j((v + 0.5).floor() as i64));
                    let at = (j as usize * sg.w as usize + i as usize) * cb;
                    cell.copy_from_slice(&layer.bytes[at..at + cb]);
                }
                Method::Bilinear => encode(desc.dtype, &desc, layer.bilinear(sx, sz), cell),
                Method::Odd => encode(desc.dtype, &desc, odd(&layer, sx, sz), cell),
                Method::Elevation => {
                    let mut h = layer.bilinear(sx, sz);
                    if let (Some(d), Some(n), Some(a)) = (&detail, &noise, &amplitude) {
                        h += a.bilinear(sx, sz) * d.noise(n, sx, sz);
                    }
                    encode(desc.dtype, &desc, h, cell);
                }
            }
        });
        if let Some(parent) = out.join(&le.path).parent() {
            std::fs::create_dir_all(parent).map_err(|x| x.to_string())?;
        }
        let written = write_tile_pack(&out.join(&le.path), &desc, &bytes).map_err(e)?;
        report.layers.push((le.name.clone(), format!("{method:?}"), written.bytes));
        layers.push(LayerEntry {
            bytes: written.bytes,
            crc32: written.crc32,
            ..le.clone()
        });
    }

    // Side files: keep the ones that do not hold block coordinates, carry the spawn candidates.
    let mut files = Vec::new();
    for f in &m.files {
        let keep = matches!(f.path.as_str(), "strata/classes.json" | "preset.json");
        if keep {
            let to = out.join(&f.path);
            if let Some(p) = to.parent() {
                std::fs::create_dir_all(p).map_err(|x| x.to_string())?;
            }
            std::fs::copy(src.join(&f.path), &to).map_err(|x| x.to_string())?;
            files.push(FileEntry::of(out, &f.path).map_err(e)?);
        } else if f.path == "spawn/candidates.json" {
            let text = std::fs::read_to_string(src.join(&f.path)).map_err(|x| x.to_string())?;
            let mut v: Value = serde_json::from_str(&text).map_err(|x| x.to_string())?;
            if let Some(list) = v.get_mut("candidates").and_then(Value::as_array_mut) {
                for c in list.iter_mut() {
                    let (Some(x), Some(z)) = (c["x"].as_f64(), c["z"].as_f64()) else { continue };
                    let (lat, lon) = proj.lat_lon(x, z);
                    let (nx, nz) = world.block_of(lat, lon);
                    c["x"] = json!(nx.round() as i64);
                    c["z"] = json!(nz.round() as i64);
                    report.spawn_candidates += 1;
                }
            }
            std::fs::create_dir_all(out.join("spawn")).map_err(|x| x.to_string())?;
            std::fs::write(out.join(&f.path), serde_json::to_vec_pretty(&v).map_err(|x| x.to_string())?)
                .map_err(|x| x.to_string())?;
            files.push(FileEntry::of(out, &f.path).map_err(e)?);
        } else {
            report.dropped.push(f.path.clone());
        }
    }

    let mut om = m.clone();
    om.source = format!("quincuncial-prototype:{}:{}", world.layout().name(), m.source);
    om.writers.push(Writer::this("deeptime-quincuncial"));
    om.scale.circumference_blocks = c_out;
    om.grids = grids;
    om.layers = layers;
    om.files = files;
    om.created_utc = deeptime_format::manifest::rfc3339_utc(std::time::SystemTime::now());
    if let Some(Value::Object(b)) = om.notes.get_mut(BAKE_NOTES) {
        b.remove("detail");
    }
    let (x0, x1, z0, z1) = world.extent();
    om.notes.insert(
        "quincuncial".into(),
        json!({
            "prototype": "docs/design/seamless-wrapping.md",
            "layout": world.layout().name(),
            "size": world.size(),
            "lon_deg": world.lon_deg(),
            "domain": [x0, x1, z0, z1],
            "singular_points": world.singular_points(),
            "seams": world.seams().iter().map(|s| s.portal(64.0, 1200.0)).collect::<Vec<_>>(),
            "source_circumference": src_c,
        }),
    );
    om.write(out).map_err(e)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::{Codec, Raster, Scale};

    /// A tiny source world whose elevation is a smooth function of the planet point; after the
    /// resample, each seam's two sides hold the same heights (the data-level seam check).
    #[test]
    fn a_resampled_world_matches_itself_across_every_seam() {
        let dir = std::env::temp_dir().join(format!("dtq-resample-{}", std::process::id()));
        let (src, out) = (dir.join("src"), dir.join("out"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&src).unwrap();
        let c = 2048u32;
        let proj = Projection::new(c);
        let g = GridDesc::for_planet("L3", c, 2).unwrap();
        let elev = Raster::<i16>::from_fn(g.clone(), |i, j| {
            let (lat, lon) = proj.lat_lon(g.block_x(i64::from(i)) as f64, g.block_z(i64::from(j)) as f64);
            let u = crate::world::unit_of_lat_lon(lat, lon);
            (1000.0 * (3.0 * u[0] + 2.0 * u[1] * u[2] + u[2])) as i16
        });
        let desc = LayerDesc::scalar("elevation", DType::I16, g.clone(), Codec::Lz4Block);
        let w = write_tile_pack(&src.join("elevation.dtr"), &desc, &elev.to_le_bytes()).unwrap();
        let mut m = Manifest::new("test", Writer::this("test"), Scale::default_for(c));
        m.grids.insert("L3".into(), g);
        m.layers.push(LayerEntry {
            name: "elevation".into(),
            path: "elevation.dtr".into(),
            grid: "L3".into(),
            dtype: DType::I16,
            codec: Codec::Lz4Block,
            bytes: w.bytes,
            crc32: w.crc32,
        });
        m.write(&src).unwrap();

        for world in [QWorld::guyou(512.0, 10.0), QWorld::peirce(512.0, 10.0)] {
            let c_out = strip_circumference(&world) as u32;
            let _ = std::fs::remove_dir_all(&out);
            resample(&src, &out, &world, c_out).unwrap();
            let om = Manifest::read(&out).unwrap();
            assert_eq!(om.scale.circumference_blocks, c_out);
            let l = TilePackReader::open(&out.join("elevation.dtr")).unwrap().load().unwrap();
            for seam in world.seams() {
                let mut worst = 0.0f64;
                for k in 0..200 {
                    let t = (k as f64 + 0.5) / 200.0 * seam.length - 0.5 * seam.length;
                    let (x, z) = seam.point(t);
                    // One block inside, and its glued partner one block inside on the other side.
                    let (ix, iz) = (x + seam.inward.0, z + seam.inward.1);
                    let (ox, oz) = (x - seam.inward.0, z - seam.inward.1);
                    let (gx, gz) = seam.apply(ox, oz);
                    // The outside block is the glued inside block: the strip holds the tiling.
                    let d_out = (l.bilinear(ox, oz) - l.bilinear(gx, gz)).abs();
                    worst = worst.max(d_out);
                    // Across the seam the terrain moves by about one block's slope, not by a jump.
                    let d_cross = (l.bilinear(ix, iz) - l.bilinear(gx, gz)).abs();
                    assert!(d_cross < 120.0, "{} t {t}: {d_cross}", seam.name);
                }
                assert!(worst < 2.5, "{:?} {}: outside differs by {worst}", world.layout(), seam.name);
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}

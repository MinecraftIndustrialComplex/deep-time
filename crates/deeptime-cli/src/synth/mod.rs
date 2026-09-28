// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime synth`: synthetic world data for Phase 0 and the GameTests (plan §9.3, P0-09).

pub mod terrain;

use std::path::{Path, PathBuf};

use deeptime_format::{
    Codec, DType, FileEntry, FormatError, GridDesc, LayerDesc, LayerEntry, LoadedLayer, Manifest,
    Raster, RasterRead, Scale, TILE, Writer, fsutil::write_atomic, manifest::MANIFEST,
    write_tile_pack,
};
use serde_json::{Value, json};

use terrain::{LakeTest, Terrain, WrapTest};

/// Blocks per cell of each grid in the synthetic worlds (the "16k quick" ladder of plan §5.2 at
/// every size: L3 r 2, L2 r 4 — the climate layers' vanilla quart grid — and L1 r 16).
pub const R_L3: u32 = 2;
pub const R_L2: u32 = 4;
pub const R_L1: u32 = 16;

/// Raw nodata of i16 layers.
pub const NODATA_I16: i16 = i16::MIN;

/// The climate layers, in vanilla parameter × 10,000 (the router's field names, plan §7.1).
pub const CLIMATE: [&str; 5] = [
    "climate/temperature",
    "climate/vegetation",
    "climate/continents",
    "climate/erosion",
    "climate/ridges",
];

/// The strata placeholder's classes (vanilla members in the rock-tag vocabulary: sandstone,
/// red_sandstone, granite).
pub const STRATA_CLASSES: [&str; 3] = ["sandstone", "red_bed", "granite"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    WrapTest,
    LakeTest,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::WrapTest => "wrap-test",
            Kind::LakeTest => "lake-test",
        }
    }
}

pub struct Options {
    pub kind: Kind,
    pub size: u32,
    pub out: PathBuf,
    pub viewer: Option<PathBuf>,
    pub dump_samples: Option<PathBuf>,
    pub codec: Codec,
}

/// What `run` produced, for the summary line.
pub struct Summary {
    pub layers: usize,
    pub bytes: u64,
}

/// Writes a synthetic world.
pub fn run(opts: &Options) -> Result<Summary, FormatError> {
    let c = opts.size;
    if c < 2 * TILE || !c.is_multiple_of(64) {
        return Err(FormatError::Invalid(format!(
            "--size {c}: the circumference must be a multiple of 64 and at least {}",
            2 * TILE
        )));
    }
    let terrain: Box<dyn Terrain> = match opts.kind {
        Kind::WrapTest => Box::new(WrapTest::new(c)),
        Kind::LakeTest => Box::new(LakeTest { c }),
    };
    let terrain = terrain.as_ref();

    // A folder with a manifest is complete: drop any old one before writing anything.
    let manifest_path = opts.out.join(MANIFEST);
    if manifest_path.exists() {
        std::fs::remove_file(&manifest_path).map_err(|e| io(&manifest_path, e))?;
    }

    let l3 = GridDesc::for_planet("L3", c, R_L3)?;
    let l2 = GridDesc::for_planet("L2", c, R_L2)?;
    let l1 = GridDesc::for_planet("L1", c, R_L1)?;

    let mut layers: Vec<LoadedLayer> = Vec::new();

    let elev = sample_i16(&l3, |x, z| quantise(terrain.height(x, z) * 16.0));
    layers.push(layer_i16(
        "elevation",
        "block",
        1.0 / 16.0,
        None,
        &l3,
        opts.codec,
        &elev,
    )?);

    let water = sample_i16(&l3, |x, z| match terrain.water_level(x, z) {
        Some(y) => quantise(y * 16.0),
        None => NODATA_I16,
    });
    layers.push(layer_i16(
        "water_level",
        "block",
        1.0 / 16.0,
        Some(NODATA_I16),
        &l3,
        opts.codec,
        &water,
    )?);

    for name in CLIMATE {
        let ras = sample_i16(&l2, |x, z| quantise(terrain.raster(name, x, z) * 10_000.0));
        layers.push(layer_i16(
            name,
            "vanilla_param",
            1e-4,
            None,
            &l2,
            opts.codec,
            &ras,
        )?);
    }

    let strata = strata_layer(&l1, &layers[0], opts.codec)?;

    // Write the tile packs, then the side files, then the manifest.
    let mut manifest = Manifest::new(
        &format!("synth:{}", opts.kind.name()),
        Writer::this("deeptime-cli"),
        Scale::default_for(c),
    );
    let radius_m = f64::from(c) * manifest.scale.m_per_block_h / (2.0 * std::f64::consts::PI);
    manifest.planet.insert("radius_m".into(), json!(radius_m));
    for g in [&l1, &l2, &l3] {
        manifest.grids.insert(g.name.clone(), g.clone());
    }
    let mut total = 0u64;
    for l in layers.iter().chain(std::iter::once(&strata)) {
        let rel = if l.desc.name.starts_with("strata/") {
            format!("{}.dtr", l.desc.name)
        } else {
            format!("rasters/{}.dtr", l.desc.name)
        };
        let w = write_tile_pack(&opts.out.join(&rel), &l.desc, &l.bytes)?;
        total += w.bytes;
        manifest.layers.push(LayerEntry {
            name: l.desc.name.clone(),
            path: rel,
            grid: l.desc.grid.name.clone(),
            dtype: l.desc.dtype,
            codec: l.desc.codec,
            bytes: w.bytes,
            crc32: w.crc32,
        });
    }

    let classes = json!({ "classes": STRATA_CLASSES });
    let spawn = spawn_candidates(&layers[0], &layers[1], manifest.scale.sea_level_y);
    for (rel, value) in [
        ("strata/classes.json", &classes),
        ("spawn/candidates.json", &spawn),
    ] {
        let mut bytes = serde_json::to_vec_pretty(value).expect("JSON");
        bytes.push(b'\n');
        let crc32 = write_atomic(&opts.out.join(rel), &bytes)?;
        total += bytes.len() as u64;
        manifest.files.push(FileEntry {
            path: rel.to_owned(),
            bytes: bytes.len() as u64,
            crc32,
        });
    }

    manifest.notes.insert(
        "synthetic".into(),
        json!({
            "kind": opts.kind.name(),
            "sampling": "cell (i, j) holds the value at block (origin_block_x + i·r, origin_block_z + j·r)",
            "elevation": "round(16·H(x, z)); H is AnalyticTestTerrain's closed form for wrap-test",
            "climate": "round(10000·p) of AnalyticTestTerrain.raster's formulas, with H from this kind's terrain"
        }),
    );
    manifest.notes.insert("strata".into(), strata_notes());
    if opts.kind == Kind::LakeTest {
        manifest.notes.insert("lake".into(), LakeTest::notes());
    }
    manifest.write(&opts.out)?;

    if let Some(path) = &opts.dump_samples {
        let dump = dump_samples(&manifest, &layers, &strata);
        let bytes = serde_json::to_vec(&dump).expect("JSON");
        write_atomic(path, &bytes)?;
    }

    if let Some(dir) = &opts.viewer {
        let mut refs: Vec<&LoadedLayer> = layers.iter().collect();
        refs.sort_by_key(|l| l.desc.name != "elevation"); // elevation first
        let info = deeptime_viewer::BundleInfo {
            title: format!("Deep Time — synthetic {} (C = {c})", opts.kind.name()),
            source: manifest.source.clone(),
            circumference: c,
            sea_level_y: manifest.scale.sea_level_y,
            format_version: manifest.format_version.clone(),
            created_utc: manifest.created_utc.clone(),
        };
        deeptime_viewer::write_bundle(
            dir,
            &info,
            &refs,
            Some(deeptime_viewer::Strata {
                layer: &strata,
                classes: STRATA_CLASSES.iter().map(|s| s.to_string()).collect(),
            }),
        )
        .map_err(|e| io(dir, e))?;
    }

    Ok(Summary {
        layers: manifest.layers.len(),
        bytes: total,
    })
}

fn io(path: &Path, e: std::io::Error) -> FormatError {
    FormatError::Io {
        path: path.to_owned(),
        source: e,
    }
}

/// Rounds half away from zero and saturates to i16 (NaN → nodata).
pub fn quantise(v: f64) -> i16 {
    if v.is_nan() {
        return NODATA_I16;
    }
    // Keep −32768 for nodata.
    v.round()
        .clamp(f64::from(i16::MIN + 1), f64::from(i16::MAX)) as i16
}

/// Samples `f(block_x, block_z)` at every cell, rows split across threads.
fn sample_i16(grid: &GridDesc, f: impl Fn(i64, i64) -> i16 + Sync) -> Raster<i16> {
    let w = grid.w as usize;
    let mut data = vec![0i16; grid.cells()];
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(8);
    let rows_per = (grid.h as usize).div_ceil(threads).max(1);
    std::thread::scope(|s| {
        for (chunk_no, chunk) in data.chunks_mut(rows_per * w).enumerate() {
            let f = &f;
            s.spawn(move || {
                for (r, row) in chunk.chunks_mut(w).enumerate() {
                    let j = (chunk_no * rows_per + r) as i64;
                    let z = grid.block_z(j);
                    for (i, cell) in row.iter_mut().enumerate() {
                        *cell = f(grid.block_x(i as i64), z);
                    }
                }
            });
        }
    });
    Raster {
        grid: grid.clone(),
        data,
    }
}

fn layer_i16(
    name: &str,
    units: &str,
    scale: f64,
    nodata: Option<i16>,
    grid: &GridDesc,
    codec: Codec,
    ras: &Raster<i16>,
) -> Result<LoadedLayer, FormatError> {
    let mut d = LayerDesc::scalar(name, DType::I16, grid.clone(), codec);
    d.units = units.to_owned();
    d.scale = scale;
    d.nodata = nodata.map(f64::from);
    LoadedLayer::new(d, ras.to_le_bytes())
}

/// The strata placeholder: per L1 cell three horizons below the surface sampled from the L3
/// elevation at the same block — sandstone from the surface, red beds from 6 blocks down, granite
/// from 40 blocks down to the bottom of the world.
fn strata_layer(
    l1: &GridDesc,
    elevation: &LoadedLayer,
    codec: Codec,
) -> Result<LoadedLayer, FormatError> {
    const K: usize = 3;
    let depths = [0i32, 6 * 16, 40 * 16];
    let e = &elevation.desc.grid;
    let mut bytes = Vec::with_capacity(l1.cells() * 3 * K);
    for j in 0..l1.h {
        for i in 0..l1.w {
            // Same block, L3 index: (block − origin) / r.
            let bx = l1.block_x(i64::from(i));
            let bz = l1.block_z(i64::from(j));
            let ei = (bx - e.origin_block_x) / i64::from(e.r);
            let ej = (bz - e.origin_block_z) / i64::from(e.r);
            let surface = elevation.raw(ei, ej) as i32;
            for d in depths {
                let top = (surface - d).max(i32::from(i16::MIN + 1)) as i16;
                bytes.extend_from_slice(&top.to_le_bytes());
            }
            bytes.extend_from_slice(&[0u8, 1, 2]);
        }
    }
    let mut d = LayerDesc::scalar("strata/horizons", DType::HorizonStack, l1.clone(), codec);
    d.units = "block/16 top, class index".into();
    d.scale = 1.0 / 16.0;
    d.stack_depth = Some(K as u32);
    LoadedLayer::new(d, bytes)
}

fn strata_notes() -> Value {
    json!({
        "placeholder": true,
        "grid": "L1",
        "horizons": [
            {"class": STRATA_CLASSES[0], "top": "surface (the L3 elevation at the cell's block)"},
            {"class": STRATA_CLASSES[1], "top": "surface − 6 blocks"},
            {"class": STRATA_CLASSES[2], "top": "surface − 40 blocks, down to min_y"}
        ]
    })
}

/// Land cells for spawn: dry, above sea level + 2, locally flat (≤ 1 block to each L3 neighbour),
/// at least 64 blocks apart, nearest to (0, 0) first; at most 8.
fn spawn_candidates(elevation: &LoadedLayer, water: &LoadedLayer, sea_level: i32) -> Value {
    let g = &elevation.desc.grid;
    let c = g.circumference() as i64;
    let stride = 4i64; // every 8 blocks at r 2
    let mut found: Vec<(i64, i64, i64, i64, f64)> = Vec::new(); // (dist², x, z, …)
    for j in (1..i64::from(g.h) - 1).step_by(stride as usize) {
        for i in (0..i64::from(g.w)).step_by(stride as usize) {
            let h = elevation.get(i, j);
            if h < f64::from(sea_level + 2) || !water.get(i, j).is_nan() {
                continue;
            }
            let flat = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .all(|&(di, dj)| (elevation.get(i + di, j + dj) - h).abs() <= 1.0);
            if !flat {
                continue;
            }
            let (x, z) = (g.block_x(i), g.block_z(j));
            found.push((x * x + z * z, x, z, i, h));
        }
    }
    found.sort_by_key(|f| (f.0, f.1, f.2));
    let mut picked: Vec<Value> = Vec::new();
    let mut taken: Vec<(i64, i64)> = Vec::new();
    for (_, x, z, i, h) in found {
        let far = taken.iter().all(|&(tx, tz)| {
            let mut dx = (x - tx).rem_euclid(c);
            dx = dx.min(c - dx);
            dx * dx + (z - tz) * (z - tz) >= 64 * 64
        });
        if !far {
            continue;
        }
        taken.push((x, z));
        let j = (z - g.origin_block_z) / i64::from(g.r);
        picked.push(json!({ "x": x, "z": z, "surface_y": h, "cell": [i, j] }));
        if picked.len() == 8 {
            break;
        }
    }
    json!({
        "rule": "L3 sample points on an 8-block lattice: elevation ≥ sea level + 2, no water, within 1 block of each L3 neighbour; nearest to (0, 0) first, at least 64 blocks apart, at most 8",
        "candidates": picked
    })
}

/// splitmix64, the deterministic generator behind the dumped random cells.
pub struct SplitMix(pub u64);

impl SplitMix {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

/// The layers the Java parity test compares.
pub const DUMPED: [&str; 3] = ["elevation", "water_level", "climate/temperature"];

/// Exact raw values at every cell of the first and last tile plus 1,000 deterministic random
/// cells of each dumped layer, bilinear samples at 200 random fractional block positions, and 200
/// random horizon stacks — the Java `TilePackParityTest` fixture.
fn dump_samples(manifest: &Manifest, layers: &[LoadedLayer], strata: &LoadedLayer) -> Value {
    let mut rng = SplitMix(0x0DEE_971A_E5EE_D000);
    let mut out_layers = serde_json::Map::new();
    let mut bilinear = serde_json::Map::new();
    for name in DUMPED {
        let l = layers
            .iter()
            .find(|l| l.desc.name == name)
            .expect("dumped layer");
        let d = &l.desc;
        let mut cells: Vec<Value> = Vec::new();
        let last = d.tile_count() - 1;
        let mut tiles = vec![0];
        if last != 0 {
            tiles.push(last);
        }
        for t in tiles {
            let (x0, z0, tw, th) = d.tile_rect(t);
            for j in z0..z0 + th {
                for i in x0..x0 + tw {
                    cells.push(json!([i, j, l.raw_cell(i, j) as i64]));
                }
            }
        }
        for _ in 0..1000 {
            let i = rng.below(u64::from(d.grid.w)) as u32;
            let j = rng.below(u64::from(d.grid.h)) as u32;
            cells.push(json!([i, j, l.raw_cell(i, j) as i64]));
        }
        out_layers.insert(
            name.into(),
            json!({
                "grid": d.grid, "scale": d.scale, "offset": d.offset, "nodata": d.nodata,
                "cells": cells
            }),
        );
        if d.nodata.is_none() {
            let c = d.grid.circumference() as i64;
            let r = i64::from(d.grid.r);
            let mut pts: Vec<Value> = Vec::new();
            for _ in 0..200 {
                // Thousandths of a block, a few cells past the seam and the poles.
                let bx = (-c / 2 - 3 * r) as f64
                    + rng.below(((c + 6 * r) * 1000) as u64) as f64 / 1000.0;
                let bz = (-c / 4 - 3 * r) as f64
                    + rng.below(((c / 2 + 6 * r) * 1000) as u64) as f64 / 1000.0;
                pts.push(json!([bx, bz, l.bilinear(bx, bz)]));
            }
            bilinear.insert(name.into(), Value::Array(pts));
        }
    }
    let k = strata.desc.stack_depth.unwrap_or(0);
    let mut stacks: Vec<Value> = Vec::new();
    for _ in 0..200 {
        let i = rng.below(u64::from(strata.desc.grid.w)) as u32;
        let j = rng.below(u64::from(strata.desc.grid.h)) as u32;
        let (tops, classes): (Vec<i16>, Vec<u8>) = (0..k).map(|h| strata.horizon(i, j, h)).unzip();
        stacks.push(json!([i, j, tops, classes]));
    }
    json!({
        "format_version": manifest.format_version,
        "source": manifest.source,
        "circumference": manifest.scale.circumference_blocks,
        "layers": out_layers,
        "bilinear": bilinear,
        "horizons": { "stack_depth": k, "cells": stacks },
        "lz4_vectors": lz4_vectors()
    })
}

/// Inputs and their `lz4_flex` blocks, for `Lz4RoundTripTest` (lz4-java must decode each block
/// back to its input).
fn lz4_vectors() -> Value {
    let mut rng = SplitMix(0x1234_5678);
    let text = "Deep Time world data: lz4_flex block -> lz4-java safe decompressor. ".repeat(20);
    let noise: Vec<u8> = (0..65_536).map(|_| rng.next_u64() as u8).collect();
    let smooth: Vec<u8> = (0..32_768)
        .flat_map(|i| (((f64::from(i) / 900.0).sin() * 4000.0) as i16).to_le_bytes())
        .collect();
    let zeros = vec![0u8; 131_072];
    let cases: [(&str, &[u8]); 5] = [
        ("tiny", b"a"),
        ("text", text.as_bytes()),
        ("noise", &noise),
        ("smooth_i16", &smooth),
        ("zeros", &zeros),
    ];
    Value::Array(
        cases
            .iter()
            .map(|(name, data)| {
                json!({
                    "name": name,
                    "input": deeptime_viewer::base64(data),
                    "block": deeptime_viewer::base64(&deeptime_format::tilepack::lz4_block(data))
                })
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::TilePackReader;

    #[test]
    fn quantise_saturates_and_keeps_nodata_free() {
        assert_eq!(quantise(1.5), 2);
        assert_eq!(quantise(-1.5), -2);
        assert_eq!(quantise(1e9), i16::MAX);
        assert_eq!(quantise(-1e9), i16::MIN + 1);
        assert_eq!(quantise(f64::NAN), NODATA_I16);
    }

    #[test]
    fn wrap_test_world_reads_back() {
        let dir = std::env::temp_dir().join(format!("dt-synth-{}", std::process::id()));
        let opts = Options {
            kind: Kind::WrapTest,
            size: 1024,
            out: dir.join("w"),
            viewer: Some(dir.join("v")),
            dump_samples: Some(dir.join("s.json")),
            codec: Codec::Lz4Block,
        };
        run(&opts).unwrap();
        let m = Manifest::read(&opts.out).unwrap();
        m.verify(&opts.out).unwrap();
        assert_eq!(m.format_version, "0.0");
        assert_eq!(m.layers.len(), 8);
        let e = TilePackReader::open(&opts.out.join("rasters/elevation.dtr"))
            .unwrap()
            .load()
            .unwrap();
        assert_eq!((e.desc.grid.w, e.desc.grid.h), (512, 256));
        let t = WrapTest::new(1024);
        for (x, z) in [(0i64, 0i64), (128, 32), (-512, -256), (510, 254)] {
            let h = e.bilinear(x as f64, z as f64);
            assert!((h - t.height(x, z)).abs() <= 1.0 / 32.0, "H({x},{z}) {h}");
        }
        for f in [
            "index.html",
            "viewer.js",
            "ramps.json",
            "layers/elevation.png",
        ] {
            assert!(dir.join("v").join(f).exists(), "{f}");
        }
        let dump: Value =
            serde_json::from_slice(&std::fs::read(dir.join("s.json")).unwrap()).unwrap();
        assert_eq!(
            dump["layers"]["elevation"]["cells"]
                .as_array()
                .unwrap()
                .len(),
            2 * 65536 + 1000
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn lake_test_holds_water_inside_the_rim() {
        let dir = std::env::temp_dir().join(format!("dt-lake-{}", std::process::id()));
        let opts = Options {
            kind: Kind::LakeTest,
            size: 1024,
            out: dir.clone(),
            viewer: None,
            dump_samples: None,
            codec: Codec::Raw,
        };
        run(&opts).unwrap();
        let load = |n: &str| TilePackReader::open(&dir.join(n)).unwrap().load().unwrap();
        let e = load("rasters/elevation.dtr");
        let w = load("rasters/water_level.dtr");
        assert_eq!(e.nearest(0.0, 0.0), 110.0);
        assert_eq!(w.nearest(0.0, 0.0), 120.0);
        assert_eq!(e.nearest(0.0, 48.0), 125.0);
        assert!(w.nearest(0.0, 48.0).is_nan());
        assert_eq!(e.nearest(100.0, 0.0), 110.0);
        assert_eq!(w.nearest(100.0, 0.0), 120.0);
        // Every water cell is ringed by terrain at least as high as the water before the nodata edge.
        let g = &e.desc.grid;
        for j in 0..i64::from(g.h) {
            for i in 0..i64::from(g.w) {
                if w.get(i, j).is_nan() {
                    continue;
                }
                for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if w.get(i + di, j + dj).is_nan() {
                        assert!(e.get(i + di, j + dj) >= 120.0, "leak at {i},{j}");
                    }
                }
            }
        }
        let m = Manifest::read(&dir).unwrap();
        assert_eq!(m.notes["lake"]["water_level_y"], 120.0);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

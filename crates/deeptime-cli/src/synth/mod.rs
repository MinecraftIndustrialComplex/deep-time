// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime synth`: synthetic world data for Phase 0 and the GameTests (plan §9.3, P0-09). The
//! terrains and the layer baking live in `deeptime_core::synth` (a session bakes them too, for the
//! Phase 1 placeholder presets); this module adds the folder of its own, the parity-test sample
//! dump and the viewer bundle.

use std::path::{Path, PathBuf};

use deeptime_core::synth::{self as core_synth, SplitMix};
use deeptime_format::{
    Codec, FormatError, LoadedLayer, Manifest, RasterRead, Writer, fsutil::write_atomic,
    manifest::MANIFEST,
};
use serde_json::{Value, json};

pub use deeptime_core::synth::{Kind, Strata};

pub struct Options {
    pub kind: Kind,
    pub strata: Strata,
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
    let world = core_synth::build(opts.kind, opts.strata, c, opts.codec)?;

    // A folder with a manifest is complete: drop any old one before writing anything.
    let manifest_path = opts.out.join(MANIFEST);
    if manifest_path.exists() {
        std::fs::remove_file(&manifest_path).map_err(|e| io(&manifest_path, e))?;
    }

    // Write the tile packs, then the side files, then the manifest.
    let mut manifest = Manifest::new(
        &format!("synth:{}", opts.kind.name()),
        Writer::this("deeptime-cli"),
        deeptime_format::Scale::default_for(c),
    );
    let total = world.write_into(&opts.out, &mut manifest)?;
    manifest.write(&opts.out)?;

    if let Some(path) = &opts.dump_samples {
        let dump = dump_samples(&manifest, &world.layers, &world.strata);
        let bytes = serde_json::to_vec(&dump).expect("JSON");
        write_atomic(path, &bytes)?;
    }

    if let Some(dir) = &opts.viewer {
        let mut refs: Vec<&LoadedLayer> = world.layers.iter().collect();
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
                layer: &world.strata,
                classes: opts
                    .strata
                    .classes()
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
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
                let zp = d.grid.projection().pole_z() as i64;
                let bz = (-zp - 3 * r) as f64
                    + rng.below(((2 * zp + 6 * r) * 1000) as u64) as f64 / 1000.0;
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
        let hz: Vec<_> = (0..k).map(|h| strata.horizon(i, j, h)).collect();
        let tops: Vec<i16> = hz.iter().map(|h| h.top).collect();
        let classes: Vec<u8> = hz.iter().map(|h| h.class).collect();
        let key_tops: Vec<u16> = hz.iter().map(|h| h.key[0]).collect();
        let key_bases: Vec<u16> = hz.iter().map(|h| h.key[1]).collect();
        stacks.push(json!([i, j, tops, classes, key_tops, key_bases]));
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
    use deeptime_core::synth::terrain::{Terrain, WrapTest};
    use deeptime_format::TilePackReader;

    #[test]
    fn wrap_test_world_reads_back() {
        let dir = std::env::temp_dir().join(format!("dt-synth-{}", std::process::id()));
        let opts = Options {
            kind: Kind::WrapTest,
            strata: Strata::Placeholder,
            size: 1024,
            out: dir.join("w"),
            viewer: Some(dir.join("v")),
            dump_samples: Some(dir.join("s.json")),
            codec: Codec::Lz4Block,
        };
        run(&opts).unwrap();
        let m = Manifest::read(&opts.out).unwrap();
        m.verify(&opts.out).unwrap();
        assert_eq!(m.format_version, "0.6");
        assert_eq!(m.layers.len(), 8);
        let e = TilePackReader::open(&opts.out.join("rasters/elevation.dtr"))
            .unwrap()
            .load()
            .unwrap();
        // The grid runs from z = −192 (the multiple of 64 past the Lambert pole at 163).
        assert_eq!((e.desc.grid.w, e.desc.grid.h), (512, 192));
        let t = WrapTest::new(1024);
        for (x, z) in [(0i64, 0i64), (128, 32), (-512, -150), (510, 150)] {
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
            512 * 192 + 1000
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn lake_test_holds_water_inside_the_rim() {
        let dir = std::env::temp_dir().join(format!("dt-lake-{}", std::process::id()));
        let opts = Options {
            kind: Kind::LakeTest,
            strata: Strata::Placeholder,
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
        // Every water cell is ringed by terrain at least as high as the water before the nodata edge
        // (the lake's 120, the closed bowl's 50).
        let g = &e.desc.grid;
        for j in 0..i64::from(g.h) {
            for i in 0..i64::from(g.w) {
                if w.get(i, j).is_nan() {
                    continue;
                }
                for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if w.get(i + di, j + dj).is_nan() {
                        assert!(e.get(i + di, j + dj) >= w.get(i, j), "leak at {i},{j}");
                    }
                }
            }
        }
        let m = Manifest::read(&dir).unwrap();
        assert_eq!(m.notes["lake"]["water_level_y"], 120.0);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn every_class_world_writes_its_classes() {
        let dir = std::env::temp_dir().join(format!("dt-every-{}", std::process::id()));
        let opts = Options {
            kind: Kind::WrapTest,
            strata: Strata::EveryClass,
            size: 1024,
            out: dir.clone(),
            viewer: None,
            dump_samples: None,
            codec: Codec::Lz4Block,
        };
        run(&opts).unwrap();
        let classes: Value =
            serde_json::from_slice(&std::fs::read(dir.join("strata/classes.json")).unwrap())
                .unwrap();
        assert_eq!(classes["classes"].as_array().unwrap().len(), 62);
        let m = Manifest::read(&dir).unwrap();
        assert_eq!(m.notes["strata"]["kind"], "every-class");
        let s = TilePackReader::open(&dir.join("strata/horizons.dtr"))
            .unwrap()
            .load()
            .unwrap();
        assert_eq!(s.desc.stack_depth, Some(8));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

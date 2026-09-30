// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime heightmap`: the terrain of a box of a world-data folder as a NumPy `.npy` array, for
//! looking at terrain without a Minecraft server (review renders, the mountain-character work).
//!
//! The array is float32 `[rows, cols, channels]` over blocks `x0..x1` × `z0..z1` (exclusive ends,
//! step `step`): channel 0 the terrain height H the game's density is zero at (the column
//! evaluator's `surface`), 1 the `elevation` layer E, 2 the detail term A·N, then one channel per
//! `--layers` name (the layer's nearest value). `<out>.json` says which channel is which.

use std::io::Write;
use std::path::Path;

use deeptime_format::{ColumnWorld, FormatError, LazyLayer, RasterRead};
use serde_json::json;

pub struct Box2 {
    pub x0: i64,
    pub z0: i64,
    pub x1: i64,
    pub z1: i64,
}

pub fn run(
    world: &Path,
    b: &Box2,
    step: i64,
    layers: &[String],
    out: &Path,
) -> Result<(usize, usize), FormatError> {
    let w = ColumnWorld::open(world)?;
    let mut extra = Vec::new();
    for name in layers {
        let e = w
            .manifest
            .layer(name)
            .ok_or_else(|| FormatError::Invalid(format!("no layer {name}")))?;
        extra.push(LazyLayer::open(&world.join(&e.path))?);
    }
    let step = step.max(1);
    let cols = ((b.x1 - b.x0) / step).max(0) as usize;
    let rows = ((b.z1 - b.z0) / step).max(0) as usize;
    let ch = 3 + extra.len();
    let mut data = vec![0f32; rows * cols * ch];
    for r in 0..rows {
        let z = b.z0 + r as i64 * step;
        for c in 0..cols {
            let x = b.x0 + c as i64 * step;
            let (e, d) = w.height_parts(x, z);
            let o = (r * cols + c) * ch;
            data[o] = w.height(x, z) as f32;
            data[o + 1] = e as f32;
            data[o + 2] = d as f32;
            let wx = w.wrap_x(x) as f64;
            for (k, l) in extra.iter().enumerate() {
                data[o + 3 + k] = l.nearest(wx, z as f64) as f32;
            }
        }
    }
    let io = |e: std::io::Error| FormatError::Io {
        path: out.to_owned(),
        source: e,
    };
    let mut f = std::io::BufWriter::new(std::fs::File::create(out).map_err(io)?);
    let mut header =
        format!("{{'descr': '<f4', 'fortran_order': False, 'shape': ({rows}, {cols}, {ch}), }}");
    while (10 + header.len() + 1) % 64 != 0 {
        header.push(' ');
    }
    header.push('\n');
    f.write_all(b"\x93NUMPY\x01\x00").map_err(io)?;
    f.write_all(&(header.len() as u16).to_le_bytes()).map_err(io)?;
    f.write_all(header.as_bytes()).map_err(io)?;
    for v in &data {
        f.write_all(&v.to_le_bytes()).map_err(io)?;
    }
    f.flush().map_err(io)?;
    let mut names = vec!["surface".to_owned(), "elevation".into(), "detail".into()];
    names.extend(layers.iter().cloned());
    let meta = json!({
        "box": [b.x0, b.z0, b.x1, b.z1], "step": step, "channels": names,
        "circumference": w.c, "source": w.manifest.source,
    });
    let mut mp = out.as_os_str().to_owned();
    mp.push(".json");
    std::fs::write(&mp, serde_json::to_vec_pretty(&meta).expect("json")).map_err(io)?;
    Ok((rows, cols))
}

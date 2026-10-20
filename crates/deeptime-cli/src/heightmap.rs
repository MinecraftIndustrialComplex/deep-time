// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime heightmap`: the terrain of a box of a world-data folder as a NumPy `.npy` array, for
//! looking at terrain without a Minecraft server (review renders, the mountain-character work).
//!
//! The array is float32 `[rows, cols, channels]` over blocks `x0..x1` × `z0..z1` (exclusive ends,
//! step `step`): channel 0 the terrain height H the game's density is zero at (the column
//! evaluator's `surface`), 1 the `elevation` layer E, 2 the detail term A·N, then one channel per
//! `--layers` name (the layer's nearest value), then with `--columns` what the column evaluator
//! decides there (the grid-artifact audit, P1-23): the rock class index at the top block and 8, 32
//! and 96 blocks under it (−1 where there is none), the number of horizons, the second horizon's
//! top, and the water kind (0 none, 1 bank, 2 river, 3 lake, 4 sea), then with `--scale` the
//! ground distance one block spans there in metres (`m_per_block`: the projection's local scale, so
//! real slopes can be read off a Petroff-Guyou world, whose scale varies; SCL-075). `<out>.json` says
//! which channel is which.

use std::io::Write;
use std::path::Path;

use deeptime_format::column::horizon_at;
use deeptime_format::water::WaterKind;
use deeptime_format::{ColumnWorld, FormatError, LazyLayer, RasterRead};
use serde_json::json;

/// The channels `--columns` adds.
const COLUMN_CHANNELS: [&str; 7] = [
    "rock_top",
    "rock_8",
    "rock_32",
    "rock_96",
    "horizons",
    "second_top",
    "water_kind",
];

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
    columns: bool,
    scale: bool,
    out: &Path,
) -> Result<(usize, usize), FormatError> {
    // Every thread opens the folder itself (below): a lazy layer's tile cache is behind one lock,
    // and threads reading stacks through one reader spend their time waiting for each other.
    let open = || -> Result<(ColumnWorld, Vec<LazyLayer>), FormatError> {
        let w = ColumnWorld::open(world)?;
        let mut extra = Vec::new();
        for name in layers {
            let e = w
                .manifest
                .layer(name)
                .ok_or_else(|| FormatError::Invalid(format!("no layer {name}")))?;
            extra.push(LazyLayer::open(&world.join(&e.path))?);
        }
        Ok((w, extra))
    };
    let (w, _) = open()?;
    let step = step.max(1);
    let cols = ((b.x1 - b.x0) / step).max(0) as usize;
    let rows = ((b.z1 - b.z0) / step).max(0) as usize;
    let base = 3 + layers.len();
    let cols_end = base + if columns { COLUMN_CHANNELS.len() } else { 0 };
    let ch = cols_end + usize::from(scale);
    // Metres of ground per radian of arc on the sphere the projection maps (its radius in blocks
    // times the nominal horizontal scale).
    let proj = w.manifest.world_projection();
    let m_per_rad = proj.radius() * w.manifest.scale.m_per_block_h;
    let proj = &proj;
    let mut data = vec![0f32; rows * cols * ch];
    // Rows in bands, one thread each (at most 8: the test machine is shared).
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get().min(8));
    let band = rows.div_ceil(threads).max(1);
    let open = &open;
    std::thread::scope(|scope| {
        for (n, part) in data.chunks_mut((band * cols * ch).max(1)).enumerate() {
            scope.spawn(move || {
                let (w, extra) = open().expect("the folder opened a moment ago");
                let mut horizons = Vec::new();
                for (k, row) in part.chunks_mut((cols * ch).max(1)).enumerate() {
                    let z = b.z0 + (n * band + k) as i64 * step;
                    for c in 0..cols {
                        let x = b.x0 + c as i64 * step;
                        let (e, d) = w.height_parts(x, z);
                        let o = c * ch;
                        row[o] = w.height(x, z) as f32;
                        row[o + 1] = e as f32;
                        row[o + 2] = d as f32;
                        let wx = w.wrap_x(x) as f64;
                        for (k, l) in extra.iter().enumerate() {
                            row[o + 3 + k] = l.nearest(wx, z as f64) as f32;
                        }
                        if columns {
                            horizons.clear();
                            if let Some(h) = w.strata() {
                                h.stack_at(x, z, w.min_y, &mut horizons);
                            }
                            let top = w.top_y(x, z);
                            for (k, depth) in [0, 8, 32, 96].into_iter().enumerate() {
                                row[o + base + k] = horizon_at(&horizons, top - depth)
                                    .map_or(-1.0, |h| f32::from(horizons[h].1));
                            }
                            row[o + base + 4] = horizons.len() as f32;
                            row[o + base + 5] = horizons.get(1).map_or(f32::NAN, |h| h.0 as f32);
                            row[o + base + 6] = match w.hydro(x, z).kind {
                                WaterKind::None => 0.0,
                                WaterKind::Bank => 1.0,
                                WaterKind::River => 2.0,
                                WaterKind::Lake => 3.0,
                                WaterKind::Sea => 4.0,
                            };
                        }
                        if scale {
                            // A conformal projection's scale is the same in every direction: the
                            // arc between the block's two x edges.
                            let (xf, zf) = (x as f64, z as f64);
                            let (a, c) = (proj.unit(xf - 0.5, zf), proj.unit(xf + 0.5, zf));
                            let d = [a[0] - c[0], a[1] - c[1], a[2] - c[2]];
                            let chord = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                            row[o + cols_end] = (2.0 * (0.5 * chord).asin() * m_per_rad) as f32;
                        }
                    }
                }
            });
        }
    });
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
    f.write_all(&(header.len() as u16).to_le_bytes())
        .map_err(io)?;
    f.write_all(header.as_bytes()).map_err(io)?;
    for v in &data {
        f.write_all(&v.to_le_bytes()).map_err(io)?;
    }
    f.flush().map_err(io)?;
    let mut names = vec!["surface".to_owned(), "elevation".into(), "detail".into()];
    names.extend(layers.iter().cloned());
    if columns {
        names.extend(COLUMN_CHANNELS.iter().map(|s| (*s).to_owned()));
    }
    if scale {
        names.push("m_per_block".to_owned());
    }
    let meta = json!({
        "box": [b.x0, b.z0, b.x1, b.z1], "step": step, "channels": names,
        "circumference": w.c, "source": w.manifest.source,
    });
    let mut mp = out.as_os_str().to_owned();
    mp.push(".json");
    std::fs::write(&mp, serde_json::to_vec_pretty(&meta).expect("json")).map_err(io)?;
    Ok((rows, cols))
}

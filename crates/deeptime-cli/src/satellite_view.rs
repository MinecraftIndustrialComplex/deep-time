// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime satellite`: the satellite view layer (`deeptime_core::satellite`), with the water of
//! the terrain drawn on it, and the "Rivers & lakes" layer (`deeptime_core::water_view`) of a
//! checkpoint's world as PNGs — for tuning the look against a finished run without rerunning the
//! simulation.

use std::path::{Path, PathBuf};
use std::time::Instant;

use deeptime_core::checkpoint::Checkpoint;
use deeptime_core::keyframes;
use deeptime_core::preset::Preset;
use deeptime_core::water_view;
use deeptime_format::Ramps;

/// What `run` drew.
pub struct Report {
    /// The world's time, years.
    pub t_years: f64,
    /// The elevation field it was drawn on.
    pub elevation: &'static str,
    /// The `rivers` layer's PNG, when the world's terrain has water to draw.
    pub rivers: Option<PathBuf>,
    /// Chains, lake pixels and milliseconds of the derived water (one more run of it, timed).
    pub water: Option<water_view::Info>,
    /// Milliseconds of the satellite picture alone, of deriving the water, and of drawing it (the
    /// overlay and the layer).
    pub satellite_ms: f64,
    pub derive_ms: f64,
    pub draw_ms: f64,
}

/// Draws the checkpoint's world `width` pixels wide (half as tall) into `out`, and its `rivers`
/// layer beside it (`<name>-rivers.png`). `on` draws it on that elevation field only (one of
/// `satellite::ELEVATION_FIELDS`, say `elevation` for the surface stage's L1 frame), else on the
/// finest the world has.
pub fn run(
    input: &Path,
    out: &Path,
    width: u32,
    on: Option<&str>,
    set: &[String],
) -> Result<Report, Box<dyn std::error::Error>> {
    let ck = Checkpoint::read(input)?;
    let preset_json = crate::run::apply_overrides(&ck.preset_json, set)?;
    let view = Preset::from_json(&preset_json)?.view_scale();
    let mut world = ck.world;
    if let Some(on) = on {
        if !deeptime_core::satellite::ELEVATION_FIELDS.contains(&on) {
            return Err(format!(
                "--on {on}: one of {:?}",
                deeptime_core::satellite::ELEVATION_FIELDS
            )
            .into());
        }
        for name in deeptime_core::satellite::ELEVATION_FIELDS {
            if name != on {
                world.fields.remove(name);
            }
        }
    }
    let elevation = deeptime_core::satellite::ELEVATION_FIELDS
        .into_iter()
        .find(|n| world.fields.contains_key(*n))
        .ok_or("the checkpoint's world has no elevation field")?;
    let size = (
        width.max(2),
        deeptime_format::Projection::picture_height(width.max(2)),
    );
    let ramps = Ramps::builtin();
    let frame = keyframes::render(
        &world,
        &ramps,
        &view,
        "satellite",
        &[
            deeptime_core::satellite::LAYER,
            deeptime_core::water_view::LAYER,
        ],
        0,
        size,
    )
    .ok_or("nothing to draw")?;
    let mut rivers = None;
    for (name, rgba) in &frame.layers {
        let path = if name == deeptime_core::satellite::LAYER {
            out.to_owned()
        } else {
            let stem = out
                .file_stem()
                .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
            let p = out.with_file_name(format!("{stem}-rivers.png"));
            rivers = Some(p.clone());
            p
        };
        std::fs::write(&path, deeptime_viewer::encode_png(size.0, size.1, rgba)?)?;
    }
    // What the water costs a frame, against the satellite picture it is drawn on (each once more,
    // timed; the first pass above has warmed the caches).
    let t = Instant::now();
    let _ = deeptime_core::satellite::render(&world, &ramps, &view, size);
    let satellite_ms = t.elapsed().as_secs_f64() * 1000.0;
    let t = Instant::now();
    let water = water_view::derive(&world, &view, size);
    let derive_ms = t.elapsed().as_secs_f64() * 1000.0;
    let t = Instant::now();
    if let Some(w) = &water {
        let mut rgba = vec![255u8; size.0 as usize * size.1 as usize * 4];
        w.overlay(&mut rgba, &ramps);
        let _ = w.render(&ramps, &view);
    }
    let draw_ms = t.elapsed().as_secs_f64() * 1000.0;
    Ok(Report {
        t_years: world.t_years,
        elevation,
        rivers,
        water: water.map(|w| w.info),
        satellite_ms,
        derive_ms,
        draw_ms,
    })
}

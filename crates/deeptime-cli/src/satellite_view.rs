// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime satellite`: the satellite view layer (`deeptime_core::satellite`) of a checkpoint's
//! world as a PNG — for tuning the look against a finished run without rerunning the simulation.

use std::path::Path;

use deeptime_core::checkpoint::Checkpoint;
use deeptime_core::preset::Preset;
use deeptime_format::Ramps;

/// Draws the checkpoint's world `width` pixels wide (half as tall) into `out`; returns its time
/// (years) and the elevation field it was drawn on.
pub fn run(
    input: &Path,
    out: &Path,
    width: u32,
) -> Result<(f64, &'static str), Box<dyn std::error::Error>> {
    let ck = Checkpoint::read(input)?;
    let view = Preset::from_json(&ck.preset_json)?.view_scale();
    let world = ck.world;
    let elevation = deeptime_core::satellite::ELEVATION_FIELDS
        .into_iter()
        .find(|n| world.fields.contains_key(*n))
        .ok_or("the checkpoint's world has no elevation field")?;
    let size = (width.max(2), (width / 2).max(1));
    let rgba = deeptime_core::satellite::render(&world, &Ramps::builtin(), &view, size)
        .ok_or("nothing to draw")?;
    std::fs::write(out, deeptime_viewer::encode_png(size.0, size.1, &rgba)?)?;
    Ok((world.t_years, elevation))
}

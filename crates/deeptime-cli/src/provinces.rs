// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime provinces`: the geologic-provinces view layer (`deeptime_core::provinces`) of a
//! checkpoint's world, as a PNG and the share of the planet each class covers — for checking the
//! rules against Earth's USGS map without rerunning the simulation.

use std::path::Path;

use deeptime_core::checkpoint::Checkpoint;
use deeptime_core::provinces::{self, CLASSES};
use deeptime_core::world::WorldState;
use deeptime_format::Ramps;

pub fn run(input: &Path, out: &Path) -> Result<Vec<(String, f64)>, Box<dyn std::error::Error>> {
    let ck = Checkpoint::read(input)?;
    let world = ck.world;
    let field = provinces::derive(&world).ok_or(
        "the checkpoint's world has no plate fields (regime, crust_age, plates/crust_type)",
    )?;
    if std::env::var_os("DEEPTIME_PROVINCE_STATS").is_some()
        || std::env::var_os("DEEPTIME_PROVINCE_DUMP").is_some()
    {
        stats(&world);
    }
    let mut counts = [0usize; CLASSES.len()];
    for &v in &field.data {
        if v.is_finite() {
            counts[v as usize] += 1;
        }
    }
    let total = field.data.len() as f64;
    let shares: Vec<(String, f64)> = CLASSES
        .iter()
        .zip(counts)
        .map(|(name, n)| ((*name).to_owned(), 100.0 * n as f64 / total))
        .collect();
    let frame = deeptime_core::keyframes::render(
        &world,
        &Ramps::builtin(),
        "provinces",
        &[provinces::LAYER],
        0,
        deeptime_core::keyframes::DEFAULT_SIZE,
    )
    .ok_or("nothing to draw")?;
    let png = deeptime_viewer::encode_png(frame.w, frame.h, &frame.layers[0].1)?;
    std::fs::write(out, png)?;
    Ok(shares)
}

/// Quantiles of the plate and surface fields over continental and oceanic cells (a calibration
/// aid for the rules: `DEEPTIME_PROVINCE_STATS=1`).
fn stats(world: &WorldState) {
    let Some(kind) = world.fields.get("plates/crust_type") else {
        return;
    };
    let names: Vec<&String> = world
        .fields
        .keys()
        .filter(|k| {
            ["plates/", "surface/", "structure/"]
                .iter()
                .any(|p| k.starts_with(p))
                || *k == "regime"
                || *k == "crust_age"
        })
        .collect();
    if let Some(dir) = std::env::var_os("DEEPTIME_PROVINCE_DUMP") {
        let dir = std::path::PathBuf::from(dir);
        let _ = std::fs::create_dir_all(&dir);
        for name in &names {
            let bytes: Vec<u8> = world.fields[*name]
                .data
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect();
            let _ = std::fs::write(dir.join(format!("{}.f32", name.replace('/', "__"))), bytes);
        }
    }
    for (label, continental) in [("continental", true), ("oceanic", false)] {
        println!("-- {label} cells");
        for name in &names {
            let f = &world.fields[*name];
            if f.data.len() != kind.data.len() {
                continue;
            }
            let mut v: Vec<f32> = f
                .data
                .iter()
                .zip(&kind.data)
                .filter(|(x, k)| x.is_finite() && (**k >= 0.5) == continental)
                .map(|(x, _)| *x)
                .collect();
            if v.is_empty() {
                continue;
            }
            v.sort_by(|a, b| a.total_cmp(b));
            let q = |p: f64| v[((v.len() - 1) as f64 * p) as usize];
            println!(
                "{name:36} n={:6} min {:10.2} p10 {:10.2} p50 {:10.2} p90 {:10.2} max {:10.2}",
                v.len(),
                q(0.0),
                q(0.1),
                q(0.5),
                q(0.9),
                q(1.0)
            );
        }
    }
}

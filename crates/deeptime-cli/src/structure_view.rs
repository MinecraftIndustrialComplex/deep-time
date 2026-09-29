// SPDX-License-Identifier: LGPL-3.0-or-later
//! What the viewer shows of a run's column strata (P1-07): the baked horizon stacks (the strata
//! inspector), the surface elevation and the rock at the surface as map layers, and meridian
//! cross-sections of the stacks with the plutons' bodies (`deeptime_viewer::section`).

use deeptime_core::params::PlanetPreset;
use deeptime_core::preset::{Preset, PresetBody};
use deeptime_core::structure::{self, CellSource, bake_horizons};
use deeptime_core::synth::EVERY_CLASS;
use deeptime_core::world::WorldState;
use deeptime_format::{Body, Codec, DType, LayerDesc, LoadedLayer, Ramps};
use deeptime_viewer::section::{self, Section};
use serde_json::{Value, json};

use crate::run::CliResult;

/// The viewer's share of a structure run.
pub struct StructureView {
    pub strata: LoadedLayer,
    pub classes: Vec<String>,
    /// `elevation` (block y) and `strata/outcrop` (class index), on L1.
    pub layers: Vec<LoadedLayer>,
    pub sections: Vec<Section>,
    pub bodies: Vec<Body>,
    pub summary: String,
    pub circumference: u32,
    pub sea_level_y: i32,
    pub m_per_block: (f64, f64),
}

fn f32_layer(name: &str, units: &str, grid: &deeptime_format::GridDesc, v: &[f32]) -> CliResult<LoadedLayer> {
    let mut d = LayerDesc::scalar(name, DType::F32, grid.clone(), Codec::Raw);
    d.units = units.to_owned();
    let bytes: Vec<u8> = v.iter().flat_map(|x| x.to_le_bytes()).collect();
    Ok(LoadedLayer::new(d, bytes)?)
}

/// The view of `world` when its pipeline ran the structure stage.
pub fn build(world: &WorldState, preset_json: &str) -> CliResult<Option<StructureView>> {
    let preset = Preset::from_json(preset_json)?;
    let PresetBody::Planet(p) = &preset.body else {
        return Ok(None);
    };
    let p: &PlanetPreset = p;
    if world.surface.is_none() || !world.fields.contains_key(structure::field::ROCK) {
        return Ok(None);
    }
    let h = bake_horizons::bake(world, p)?;
    h.check().map_err(|e| format!("horizon bake: {e}"))?;
    let strata = h.to_layer(Codec::Raw)?;
    let src = CellSource::new(world, p)?;
    let s = &p.scale;
    let grid = h.grid.clone();
    let (w, hgt) = (grid.w as usize, grid.h as usize);
    let n = grid.cells();
    let surface: Vec<f32> = (0..n).map(|c| src.surface_y(c, s) as f32).collect();
    let rock = world.fields[structure::field::ROCK].data.clone();
    let layers = vec![
        f32_layer("elevation", "block", &grid, &surface)?,
        f32_layer("strata/outcrop", "class index", &grid, &rock)?,
    ];
    let log_m: Vec<f32> = (0..n)
        .map(|c| world.surface.as_ref().map_or(0.0, |st| st.log.total(c) as f32))
        .collect();

    // Meridians: the column with the most land, and the one through the thickest strata log.
    let land = |i: usize| (0..hgt).filter(|&j| surface[j * w + i] > s.sea_level_y as f32).count();
    let i_a = (0..w).max_by_key(|&i| (land(i), std::cmp::Reverse(i))).unwrap_or(0);
    let thickest = (0..n)
        .max_by(|&a, &b| log_m[a].total_cmp(&log_m[b]))
        .unwrap_or(0);
    let mut i_b = thickest % w;
    if (i_b as i64 - i_a as i64).rem_euclid(w as i64).min((i_a as i64 - i_b as i64).rem_euclid(w as i64))
        < (w / 16) as i64
    {
        i_b = (i_a + w / 2) % w;
    }
    let c = i64::from(s.circumference_blocks);
    let full = |i: usize, title: &str| {
        let top = (0..hgt).map(|j| surface[j * w + i]).fold(f32::MIN, f32::max);
        let y1 = ((top + 20.0).ceil() as i32).clamp(s.sea_level_y + 20, s.max_y);
        Section {
            title: title.to_owned(),
            x: grid.block_x(i as i64),
            z0: -c / 4,
            z1: c / 4,
            y0: s.min_y,
            y1,
            w: (2 * hgt) as u32,
            h: (((y1 - s.min_y) as f64) / 1.5).round() as u32,
        }
    };
    let zoom = |x: i64, zc: i64, half: i64, title: &str| {
        let i = grid.wrap_i(((x - grid.origin_block_x) as f64 / f64::from(grid.r)).round() as i64) as usize;
        let rows = |z: i64| grid.clamp_j(((z - grid.origin_block_z) as f64 / f64::from(grid.r)).round() as i64) as usize;
        let (j0, j1) = (rows(zc - half), rows(zc + half));
        let (lo, hi) = (j0..=j1)
            .map(|j| surface[j * w + i])
            .fold((f32::MAX, f32::MIN), |(a, b), y| (a.min(y), b.max(y)));
        let y1 = ((hi.max(s.sea_level_y as f32) + 12.0).ceil() as i32).min(s.max_y);
        let y0 = ((lo - 220.0).floor() as i32).max(s.min_y);
        Section {
            title: title.to_owned(),
            x,
            z0: zc - half,
            z1: zc + half,
            y0,
            y1,
            w: 1000,
            h: ((y1 - y0) * 2).clamp(100, 900) as u32,
        }
    };
    let mut sections = vec![
        full(i_a, "Pole to pole along the meridian with the most land"),
        full(i_b, "Pole to pole through the thickest strata"),
    ];
    let x_a = grid.block_x(i_a as i64);
    let col_a: Vec<usize> = (0..hgt).map(|j| j * w + i_a).collect();
    if let Some(&cb) = col_a.iter().max_by(|&&a, &&b| log_m[a].total_cmp(&log_m[b])) {
        sections.push(zoom(x_a, grid.block_z((cb / w) as i64), 700, "The thickest basin on the first meridian (≈420 km)"));
    }
    if let Some(&cm) = col_a.iter().max_by(|&&a, &&b| surface[a].total_cmp(&surface[b])) {
        sections.push(zoom(x_a, grid.block_z((cm / w) as i64), 700, "The highest ground on the first meridian (≈420 km)"));
    }
    // Through the pluton whose roof is nearest the surface.
    let bodies: Vec<Body> = world.bodies.clone();
    let exposed = bodies
        .iter()
        .filter(|b| b.role == "pluton")
        .map(|b| {
            let ci = grid.wrap_i(((b.center[0] - grid.origin_block_x as f64) / f64::from(grid.r)).round() as i64) as usize;
            let cj = grid.clamp_j(((b.center[2] - grid.origin_block_z as f64) / f64::from(grid.r)).round() as i64) as usize;
            let top = b.center[1] + f64::from(b.axes[1]);
            (b, top - f64::from(surface[cj * w + ci]))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1));
    if let Some((b, _)) = exposed {
        let half = (f64::from(b.axes[0].max(b.axes[2])) * 4.0).max(250.0) as i64;
        let mut z = zoom(b.center[0].round() as i64, b.center[2].round() as i64, half, "Through the most exhumed pluton");
        let bottom = (b.center[1] - f64::from(b.axes[1]) - 30.0).floor() as i32;
        z.y0 = z.y0.min(bottom).max(s.min_y);
        z.h = ((z.y1 - z.y0) * 2).clamp(100, 900) as u32;
        sections.push(z);
    }
    let note = world
        .notes
        .get(structure::STAGE_NAME)
        .and_then(|t| serde_json::from_str::<Value>(t).ok())
        .map(|v| structure::summary_line(&v))
        .unwrap_or_default();
    let summary = format!(
        "{note}. The stacks are the baked horizons (K {}, {:.2} per cell on average) of L1 ({} × {} cells of {} blocks); \
         each section column is the stack at that block as the Java strata pass builds it, and bodies \
         (plutons and their aureoles) paint over the horizons.",
        h.k,
        h.mean_horizons(),
        grid.w,
        grid.h,
        grid.r
    );
    Ok(Some(StructureView {
        strata,
        classes: EVERY_CLASS.iter().map(|s| (*s).to_owned()).collect(),
        layers,
        sections,
        bodies,
        summary,
        circumference: s.circumference_blocks,
        sea_level_y: s.sea_level_y,
        m_per_block: (s.m_per_block_h, s.m_per_block_v),
    }))
}

/// Writes the sections into `dir` and returns the report's `structure` object.
pub fn write(dir: &std::path::Path, v: &StructureView) -> CliResult<Value> {
    let ramps = Ramps::builtin();
    let (_, ramp) = ramps.for_layer("structure/rock");
    let sections = section::write_sections(
        dir,
        &v.strata,
        &v.bodies,
        &v.classes,
        v.circumference,
        v.sea_level_y,
        &v.sections,
        ramp,
        v.m_per_block,
    )?;
    let mut used = vec![false; v.classes.len()];
    let k = v.strata.desc.stack_depth.unwrap_or(0) as usize;
    let cells = v.strata.desc.grid.cells();
    for c in 0..cells {
        for &b in &v.strata.bytes[c * 3 * k + 2 * k..c * 3 * k + 3 * k] {
            if usize::from(b) < used.len() {
                used[usize::from(b)] = true;
            }
        }
    }
    for b in &v.bodies {
        for name in b.rock_class.iter().chain(b.zones.iter().filter_map(|z| z.rock_class.as_ref())) {
            if let Some(k) = v.classes.iter().position(|c| c == name) {
                used[k] = true;
            }
        }
    }
    Ok(json!({
        "summary": v.summary,
        "sections": sections,
        "legend": section::legend(&v.classes, &used, ramp),
    }))
}

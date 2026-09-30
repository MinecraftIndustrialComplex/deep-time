// SPDX-License-Identifier: LGPL-3.0-or-later
//! The bake on a small planet run through the whole pipeline (plates, climate, surface, structure,
//! deposits, young, amplify) and `finalize_write`: every chunk-time layer is listed and checks out,
//! the column evaluator reads the folder, and the first horizon covers the amplified terrain.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use deeptime_format::{ColumnWorld, DetailSpec, FORMAT_VERSION, Manifest, Noise, RasterRead};
use serde_json::json;

use crate::params::planet_builtin;
use crate::preset::with_timeline;
use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};
use crate::synth::SplitMix;

/// A small planet with every stage: C 16,384 on coarse grids (L1 256 × 128, L2 512 × 256, L3
/// 1024 × 512), 60 Myr, plenty of plutons.
pub(crate) fn small_planet(myr: f64) -> String {
    let json = with_timeline(
        planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(myr),
    )
    .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = json!(3000);
    v["resolution"]["l1_r"] = json!(64);
    v["resolution"]["l2_r"] = json!(32);
    v["resolution"]["l3_r"] = json!(16);
    v["keyframes"]["size"] = json!([128, 64]);
    v["keyframes"]["every_myr"] = json!(20.0);
    v["seed"] = json!(5);
    v["structure"] = json!({"pluton_rate_km2_myr": 2e-3});
    v["young"] = json!({"myr": 1.0, "steps": 2});
    v["amplify"] = json!({
        "erosion_iterations": 4, "thermal_iterations": 3, "deposition_iterations": 2,
        "coarse_iteration_factor": 1.0, "repair_margin_cells": 8,
        "tile_cells": 256, "preview_width": 128
    });
    v.to_string()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-bake-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

pub(crate) fn run_to_folder(json: &str, dir: &Path) {
    let s = Session::create(
        json,
        "",
        SessionOptions {
            out_dir: dir.to_path_buf(),
            threads: 3,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap();
    loop {
        match s.advance(Duration::from_secs(60)).unwrap() {
            AdvanceStatus::Done => break,
            AdvanceStatus::Cancelled => panic!("cancelled"),
            _ => {}
        }
    }
    s.finalize_write().unwrap();
}

#[test]
fn a_planet_run_bakes_every_chunk_time_layer() {
    let dir = temp_dir("layers");
    run_to_folder(&small_planet(100.0), &dir);
    let m = Manifest::read(&dir).unwrap();
    m.verify(&dir).unwrap();
    assert_eq!(m.format_version, FORMAT_VERSION);
    assert_eq!(FORMAT_VERSION, "0.2");
    for name in [
        "elevation",
        "detail/amplitude",
        "water_level",
        "strata/horizons",
        "climate/temperature",
        "climate/vegetation",
        "climate/continents",
        "climate/erosion",
        "climate/ridges",
        "climate/t_mean_c",
        "climate/p_mm",
        "climate/koppen",
        "surface/regolith_blocks",
        "surface/class_top",
        "meta/plate_id",
        "meta/crust_age_myr",
        "meta/province",
    ] {
        assert!(m.layer(name).is_some(), "{name} is listed");
    }
    for f in [
        "strata/classes.json",
        "spawn/candidates.json",
        "preset.json",
    ] {
        assert!(m.files.iter().any(|e| e.path == f), "{f} is listed");
    }
    let bake = &m.notes["bake"];
    assert_eq!(bake["baked"], true);
    assert_eq!(bake["terrain"]["elevation"], "amplify");
    assert!(bake["meta"]["provinces"]["count"].as_u64().unwrap() >= 1);
    assert!(bake["terrain"]["spawn_candidates"].as_u64().unwrap() >= 1);
    let spec = DetailSpec::of(&m).unwrap().expect("a detail term");
    assert_eq!(spec.seed().unwrap(), 5);
    assert_eq!(spec.salt, super::DETAIL_SALT);
    assert_eq!(m.grids["L1"].w, 256);
    assert_eq!(m.grids["L2"].w, 512);
    assert_eq!(m.grids["L3"].w, 1024);
    // The climate layers carry the vanilla range.
    let t = deeptime_format::TilePackReader::open(&dir.join("rasters/climate/temperature.dtr"))
        .unwrap()
        .load()
        .unwrap();
    for j in (0..t.desc.grid.h).step_by(7) {
        for i in (0..t.desc.grid.w).step_by(11) {
            let v = t.get(i64::from(i), i64::from(j));
            assert!((-1.0..=1.0).contains(&v), "temperature {v}");
        }
    }

    // P1-25: the twelve months. At every L1 cell the mean of the twelve sea-level months, less the
    // lapse over the climate's own ground, is the annual mean the bake stores (to the quantum),
    // and the twelve precipitation shares sum to 12 (to the u8 quantum).
    let month = |name: String| {
        deeptime_format::TilePackReader::open(&dir.join(format!("rasters/{name}.dtr")))
            .unwrap()
            .load()
            .unwrap()
    };
    let t_sl: Vec<_> = (0..12)
        .map(|m| month(super::climate::t_sl_month_layer(m)))
        .collect();
    let share: Vec<_> = (0..12)
        .map(|m| month(super::climate::p_share_month_layer(m)))
        .collect();
    let t_mean = month("climate/t_mean_c".into());
    assert!(bake["climate"]["monthly"]["lapse_c_per_km"].is_number());
    let (mut sea_cells, mut worst_sea, mut worst_share) = (0, 0.0f64, 0.0f64);
    for j in (0..t_mean.desc.grid.h).step_by(3) {
        for i in (0..t_mean.desc.grid.w).step_by(5) {
            let (i, j) = (i64::from(i), i64::from(j));
            let mean_sl = t_sl.iter().map(|l| l.get(i, j)).sum::<f64>() / 12.0;
            let tm = t_mean.get(i, j);
            // Land cells are lapsed (tm ≤ mean_sl); sea cells are at sea level (equal).
            assert!(
                tm <= mean_sl + 0.011,
                "({i}, {j}) mean {tm} over sea-level {mean_sl}"
            );
            if (tm - mean_sl).abs() < 0.02 {
                sea_cells += 1;
                worst_sea = worst_sea.max((tm - mean_sl).abs());
            }
            let s: f64 = share.iter().map(|l| l.get(i, j)).sum();
            worst_share = worst_share.max((s - 12.0).abs());
        }
    }
    assert!(sea_cells > 0, "some cells sit at sea level");
    assert!(
        worst_share <= 12.0 * super::climate::P_SHARE_SCALE / 2.0 + 1e-9,
        "shares sum to 12 ± {worst_share}"
    );
    eprintln!(
        "bake test: monthly sea cells {sea_cells}, worst {worst_sea:.4} °C, share sum ± {worst_share:.4}"
    );
    let cw = deeptime_format::ClimateWorld::open(&dir)
        .unwrap()
        .expect("the climate API's layers");
    let s = cw.sample(123, 400, -456);
    let low = cw.sample(123, 63, -456);
    assert!(s.t_mean < low.t_mean, "colder higher up");
    assert!(
        ((low.t_mean - s.t_mean) - cw.lapse_c_per_km() * s.elevation_m / 1000.0).abs() < 1e-9,
        "the lapse over {} m",
        s.elevation_m
    );

    // The column evaluator reads the folder; the first horizon covers the terrain everywhere.
    let w = ColumnWorld::open(&dir).unwrap();
    let noise = Noise::new(5, 16_384);
    let c = 16_384i64;
    let mut rng = SplitMix(9);
    let (mut with_bodies, mut covered) = (0, 0);
    for _ in 0..400 {
        let x = rng.below(2 * c as u64) as i64 - c;
        let z = rng.below((c / 2) as u64) as i64 - c / 4;
        let col = w.sample(x, z);
        assert!(!col.horizons.is_empty(), "({x}, {z}) has horizons");
        assert!(
            col.horizons[0].0 >= col.top_y,
            "({x}, {z}): top horizon {} under the terrain's top block {} (surface {})",
            col.horizons[0].0,
            col.top_y,
            col.surface
        );
        covered += 1;
        // The detail term is the spec's noise times the amplitude layer.
        let (e, d) = w.height_parts(x, z);
        assert_eq!(e, col.elevation);
        assert_eq!(col.surface, e + d);
        let n = spec.noise(&noise, col.wx as f64, z as f64);
        assert!(d == 0.0 || (d / n).is_finite());
        // The rock runs cover the column from its top block to the floor without gaps.
        let mut y = col.top_y;
        for &(top, bottom, ref class) in &col.rock {
            assert_eq!(top, y);
            assert!(bottom <= top);
            y = bottom - 1;
            if top > col.horizons[0].0 {
                assert!(class.is_none());
            }
        }
        assert_eq!(y, -353, "down to min_y");
        if !col.bodies.is_empty() {
            with_bodies += 1;
        }
    }
    assert_eq!(covered, 400);
    // Columns through bodies exist (plutons): sample their centres.
    let mut hit = 0;
    for b in w.bodies.iter().take(40) {
        let col = w.sample(b.center[0].floor() as i64, b.center[2].floor() as i64);
        if col.bodies.iter().any(|&(id, _, _)| id == b.id) {
            hit += 1;
        }
    }
    assert!(
        w.bodies.is_empty() || hit > 0,
        "{} bodies, none hit at its centre column",
        w.bodies.len()
    );
    eprintln!(
        "bake test: {} bodies, {with_bodies} of 400 random columns through one, {hit} centre hits; {}; structure {}",
        w.bodies.len(),
        super::summary_line(bake),
        m.notes
            .get("structure")
            .map_or(String::new(), |v| v["plutons"].to_string())
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_run_without_amplify_gets_an_upsampled_terrain() {
    let dir = temp_dir("no-amplify");
    let mut v: serde_json::Value = serde_json::from_str(&small_planet(20.0)).unwrap();
    v.as_object_mut().unwrap().remove("amplify");
    v.as_object_mut().unwrap().remove("deposits");
    run_to_folder(&v.to_string(), &dir);
    let m = Manifest::read(&dir).unwrap();
    m.verify(&dir).unwrap();
    assert_eq!(
        m.notes["bake"]["terrain"]["elevation"],
        "young (L2, bilinear)"
    );
    assert!(m.layer("detail/amplitude").is_none());
    assert!(DetailSpec::of(&m).unwrap().is_none());
    let w = ColumnWorld::open(&dir).unwrap();
    let col = w.sample(100, -200);
    assert_eq!(col.detail, 0.0);
    assert_eq!(col.surface, col.elevation);
    assert!(col.horizons[0].0 >= col.top_y);
    std::fs::remove_dir_all(&dir).unwrap();
}

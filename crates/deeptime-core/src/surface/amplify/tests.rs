// SPDX-License-Identifier: LGPL-3.0-or-later
//! P1-06 acceptance: `cargo test -p deeptime-core surface::amplify`.

use std::path::PathBuf;
use std::sync::Arc;

use deeptime_format::{RasterRead, TilePackReader};

use super::kernels::{count_sinks, relief5};
use super::resample::Level;
use super::tiles::{MemSink, PackSink, TilePlan};
use super::window::Win;
use super::*;
use crate::params::planet_builtin;
use crate::preset::with_timeline;
use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};
use crate::surface::grid::Dims;
use crate::surface::{pflood, routing};

const W2: usize = 96;
const H2: usize = 48;
const R2: u32 = 8;
const C: u32 = W2 as u32 * R2;
const SEA_Y: f64 = 63.0;

/// A splitmix-style hash in [−1, 1] of a cell.
fn hash(i: usize, j: usize, salt: u64) -> f32 {
    let mut x = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (j as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ salt;
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    ((x >> 40) as f32 / (1u64 << 23) as f32) - 1.0
}

/// A wrapped planet: land across the seam, a range, a sea, a deep closed basin (a lake) and
/// noise, then the young stage's treatment of it (shallow depressions filled, exact areas).
fn l2_level() -> Level {
    let (w, h) = (W2, H2);
    let mut z = vec![0.0f32; w * h];
    for j in 0..h {
        for i in 0..w {
            let x = std::f32::consts::TAU * i as f32 / w as f32;
            let y = std::f32::consts::PI * (j as f32 + 0.5) / h as f32;
            let continent =
                1400.0 * (x + 0.4).cos() * y.sin() + 500.0 * (2.0 * x + 1.0).sin() * y.sin();
            let range = 900.0 * (-(((i as f32 - 12.0) / 5.0).powi(2))).exp() * y.sin();
            z[j * w + i] = continent + range - 250.0 + 3.0 * hash(i, j, 1);
        }
    }
    // A deep closed basin inland.
    for j in 0..h {
        for i in 0..w {
            let d2 = ((i as f32 - 6.0).powi(2) + (j as f32 - 22.0).powi(2)) / 9.0;
            z[j * w + i] -= 500.0 * (-d2).exp();
        }
    }
    let dx = f64::from(R2) * 300.0;
    let d = Dims::new(w, h, dx, true);
    let sea = 0.0;
    let hh: Vec<f64> = z.iter().map(|&v| f64::from(v)).collect();
    let mut kind = vec![pflood::LAND; w * h];
    pflood::open_sea(&d, &hh, sea, 8, &mut kind);
    let base = vec![sea; w * h];
    let mut filled = vec![0.0; w * h];
    pflood::flood(&d, &hh, &base, &mut kind, &mut filled);
    for c in 0..w * h {
        let depth = filled[c] - hh[c];
        if kind[c] != pflood::BASE && depth > 0.0 && depth < 5.0 {
            z[c] = filled[c] as f32;
        }
    }
    let hh: Vec<f64> = z.iter().map(|&v| f64::from(v)).collect();
    pflood::open_sea(&d, &hh, sea, 8, &mut kind);
    pflood::flood(&d, &hh, &base, &mut kind, &mut filled);
    let mut rt = routing::Routing::default();
    routing::receivers(&d, &filled, &kind, &mut rt);
    routing::stack(&d, &kind, &mut rt);
    let wgt = vec![1.0f32; w * h];
    let mut area = Vec::new();
    routing::accumulate(&d, &rt, &wgt, &mut area);
    Level {
        w,
        h,
        dx,
        z,
        area: area.iter().map(|&a| a as f32).collect(),
        wgt,
        kfac: vec![1.0; w * h],
        lake: (0..w * h)
            .map(|c| {
                u8::from(
                    kind[c] == pflood::SINK
                        || (kind[c] == pflood::LAND && filled[c] - hh[c] > pflood::LAKE_TOL_M),
                )
            })
            .collect(),
    }
}

/// A quick configuration: few iterations so windows stay small next to the test grids.
fn cfg(drama: f64) -> Config {
    Config {
        p: AmplifyParams {
            erosion_iterations: 4,
            thermal_iterations: 3,
            deposition_iterations: 2,
            coarse_iteration_factor: 2.0,
            repair_margin_cells: 8,
            incision_m: 8.0,
            ..AmplifyParams::default()
        },
        drama,
        seed: 7,
        circumference: C,
        sea: 0.0,
        sea_y: SEA_Y,
        mv: 20.0,
        mh: 300.0,
    }
}

fn never() -> bool {
    false
}

fn free(_: u64) -> Result<(), CoreError> {
    Ok(())
}

fn run_mem(level: &Level, cfg: &Config, plan: TilePlan, threads: usize) -> (MemSink, Outcome) {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap();
    let mut sink = MemSink::new(plan.gw, plan.gh);
    let out = pool
        .install(|| run(level.clone(), 2, cfg, plan, &mut sink, &never, &free))
        .unwrap();
    (sink, out)
}

/// The footprint of the lakes at L3: the L2 lakes grown by a cell at each of the two levels.
fn lake_l3(l2: &Level) -> Vec<u8> {
    let mid = l2.sample(&Win::full(2 * l2.w, 2 * l2.h));
    let l1 = Level {
        w: 2 * l2.w,
        h: 2 * l2.h,
        dx: l2.dx / 2.0,
        z: mid.z,
        area: mid.area,
        wgt: vec![1.0; 4 * l2.w * l2.h],
        kfac: mid.kfac,
        lake: mid.lake,
    };
    l1.sample(&Win::full(4 * l2.w, 4 * l2.h)).lake
}

fn plan_whole() -> TilePlan {
    TilePlan::whole(4 * W2, 4 * H2)
}

fn plan_tiled(c: &Config) -> TilePlan {
    TilePlan {
        gw: 4 * W2,
        gh: 4 * H2,
        tile: 64,
        halo: c.halo(),
    }
}

/// Land cells outside `mask` of `z` with no strictly lower neighbour, over the whole planet.
fn global_sinks(z: &[f32], mask: &[u8], sea: f32) -> u64 {
    let win = Win::full(4 * W2, 4 * H2);
    count_sinks(&win, z, mask, sea, (0, 0, win.w, win.h))
}

#[test]
fn amplification_creates_no_new_internal_sinks_outside_lakes() {
    let l2 = l2_level();
    assert!(l2.lake.contains(&1), "the test planet has a lake");
    assert!(
        l2.z.iter().any(|&z| z > 500.0) && l2.z.iter().any(|&z| z < -100.0),
        "and land and sea"
    );
    let c = cfg(1.0);
    let mask = lake_l3(&l2);
    // Doubling twice with no passes at all leaves pits (the interpolation makes them)…
    let plain = {
        let mid = resample::upsample2_z(&l2.z, l2.w, l2.h);
        resample::upsample2_z(&mid, 2 * l2.w, 2 * l2.h)
    };
    let before = global_sinks(&plain, &mask, 0.0);
    eprintln!("sinks of the plain upsampling: {before}");
    // … the amplifier leaves none.
    let (sink, out) = run_mem(&l2, &c, plan_whole(), 2);
    let after = global_sinks(&sink.z, &mask, 0.0);
    assert_eq!(after, 0, "sinks after amplification ({before} before)");
    assert_eq!(out.last.sinks, 0);
    assert_eq!(out.last.flood_unreached, 0);
    // It did something: relief changed, land and sea keep their places, the lake stays a basin.
    let changed = sink
        .z
        .iter()
        .zip(&plain)
        .filter(|(a, b)| (**a - **b).abs() > 0.5)
        .count();
    assert!(changed > sink.z.len() / 100, "only {changed} cells changed");
    let sea_flips = sink
        .z
        .iter()
        .zip(&plain)
        .filter(|(a, b)| (**a > 0.0) != (**b > 0.0))
        .count();
    assert!(
        sea_flips < sink.z.len() / 50,
        "{sea_flips} cells changed between land and sea"
    );
    assert!(sink.z.iter().all(|z| z.is_finite()));
}

#[test]
fn the_flood_repairs_a_pit_nobody_declared_a_lake() {
    // The young stage marks every depression it finds as a lake. A pit that reaches the amplifier
    // undeclared (here dug by hand into three slopes) is repaired by the flood, and the assembled
    // planet has no sink outside the declared lakes.
    let mut l2 = l2_level();
    let mut dug: Vec<(usize, usize)> = Vec::new();
    for c in 0..l2.z.len() {
        let (i, j) = (c % W2, c / W2);
        if dug.len() >= 3 || !(3..W2 - 3).contains(&i) || !(3..H2 - 3).contains(&j) {
            continue;
        }
        let apart = dug
            .iter()
            .all(|&(a, b)| a.abs_diff(i) > 8 || b.abs_diff(j) > 8);
        let ring: Vec<usize> = (0..9)
            .filter(|&k| k != 4)
            .map(|k| (j + k / 3 - 1) * W2 + i + k % 3 - 1)
            .collect();
        let low = ring.iter().map(|&n| l2.z[n]).fold(f32::MAX, f32::min);
        let all_land = ring.iter().all(|&n| l2.z[n] > 320.0 && l2.lake[n] == 0);
        if apart && all_land && l2.lake[c] == 0 {
            // A pit 200 m deeper than the lowest ground round it, still above the sea.
            l2.z[c] = low - 200.0;
            dug.push((i, j));
        }
    }
    let dug = dug.len();
    assert!(dug >= 1, "found nowhere to dig");
    let c = cfg(1.0);
    let (sink, out) = run_mem(&l2, &c, plan_whole(), 2);
    // The first level meets the pits (the last has nothing left to repair): the flood raised them.
    let first = &out.levels[0].stats;
    assert!(first.pre_flood_raised_cells > 0, "no pit to repair");
    assert!(
        first.pre_flood_max_raise_m > 10.0,
        "{}",
        first.pre_flood_max_raise_m
    );
    assert_eq!(global_sinks(&sink.z, &lake_l3(&l2), 0.0), 0);
    assert_eq!(out.last.sinks, 0);
}

#[test]
fn streamed_and_in_memory_l3_agree_bit_for_bit() {
    let l2 = l2_level();
    let c = cfg(1.0);
    let (whole, whole_out) = run_mem(&l2, &c, plan_whole(), 2);
    let plan = plan_tiled(&c);
    assert!(
        plan.halo * 2 + plan.tile < plan.gw,
        "real tiles, not one window"
    );
    let (tiled, tiled_out) = run_mem(&l2, &c, plan, 2);
    assert_eq!(tiled_out.last.tiles, plan.cores().len());
    assert!(tiled_out.last.tiles >= 8);
    let bad = whole
        .z
        .iter()
        .zip(&tiled.z)
        .enumerate()
        .filter(|(_, (a, b))| a.to_bits() != b.to_bits())
        .count();
    assert_eq!(
        bad,
        0,
        "{bad} of {} cells differ between the tilings",
        whole.z.len()
    );
    assert_eq!(
        whole.amp.iter().map(|a| a.to_bits()).collect::<Vec<_>>(),
        tiled.amp.iter().map(|a| a.to_bits()).collect::<Vec<_>>(),
        "the detail amplitude too"
    );
    assert_eq!(whole_out.last.sinks, 0);
    assert_eq!(tiled_out.last.sinks, 0);
    // The thinned copies and the statistics of the cores agree too.
    assert_eq!(whole_out.preview.z_m, tiled_out.preview.z_m);
    assert_eq!(
        whole_out.last.flood_raised_cells,
        tiled_out.last.flood_raised_cells
    );
    assert_eq!(
        whole_out.last.pre_flood_raised_cells,
        tiled_out.last.pre_flood_raised_cells
    );
    assert_eq!(
        whole_out.last.flood_max_raise_at,
        tiled_out.last.flood_max_raise_at
    );
    // (sums differ in the last bits only: tiles add up in another order)
    let (e0, e1) = (whole_out.last.eroded_m_cells, tiled_out.last.eroded_m_cells);
    assert!((e0 - e1).abs() <= 1e-9 * e0.abs(), "{e0} {e1}");
    assert_eq!(whole_out.last.cells, tiled_out.last.cells);
    // The levels before the last are the same in both.
    assert_eq!(whole_out.levels.len(), 1);
}

#[test]
fn the_levels_before_the_last_agree_tiled_and_whole() {
    // The middle level (192 × 96 cells here) in tiles of 64 with its halo, then the last level
    // in tiles: the terrain is what a run on whole rasters gives, to the bit.
    let l2 = l2_level();
    let whole_cfg = cfg(1.0);
    let (whole, _) = run_mem(&l2, &whole_cfg, plan_whole(), 2);
    let mut tiled_cfg = cfg(1.0);
    tiled_cfg.p.stream = StreamMode::Always;
    tiled_cfg.p.tile_cells = 64;
    let (tiled, out) = run_mem(&l2, &tiled_cfg, plan_tiled(&tiled_cfg), 2);
    assert!(out.levels[0].tiles >= 6, "{} tiles", out.levels[0].tiles);
    assert!(
        whole
            .z
            .iter()
            .zip(&tiled.z)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
        "tiled middle level and last level differ from the whole-raster run"
    );
    assert_eq!(out.levels[0].stats.sinks, 0);
}

#[test]
fn streamed_tile_packs_hold_what_the_memory_run_holds() {
    let l2 = l2_level();
    let c = cfg(1.0);
    let (mem, _) = run_mem(&l2, &c, plan_whole(), 2);
    let dir = std::env::temp_dir().join(format!("dt-amplify-packs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let l3 = GridDesc::for_planet("L3", C, 2).unwrap();
    assert_eq!((l3.w as usize, l3.h as usize), (4 * W2, 4 * H2));
    let mut sink = PackSink::create(&dir, &l3, 32, c.sea, c.sea_y, c.mv).unwrap();
    let plan = plan_tiled(&c);
    let out = run(l2.clone(), 2, &c, plan, &mut sink, &never, &free).unwrap();
    let files = sink.finish().unwrap();
    assert!(out.last.tiles >= 8);
    let elev = TilePackReader::open(&dir.join(tiles::ELEVATION_PATH))
        .unwrap()
        .load()
        .unwrap();
    let amp = TilePackReader::open(&dir.join(tiles::AMPLITUDE_PATH))
        .unwrap()
        .load()
        .unwrap();
    assert_eq!(elev.desc.name, "elevation");
    assert_eq!(
        files.elevation.2.bytes,
        std::fs::metadata(dir.join(tiles::ELEVATION_PATH))
            .unwrap()
            .len()
    );
    for j in [0usize, 1, 100, 191] {
        for i in [0usize, 5, 100, 383] {
            let z = mem.z[j * 4 * W2 + i];
            let y = SEA_Y + (f64::from(z) - c.sea) / c.mv;
            assert_eq!(
                elev.raw(i as i64, j as i64),
                f64::from(crate::synth::quantise(16.0 * y))
            );
            let a = crate::surface::detail::DetailParams::amplitude_raw(f64::from(
                mem.amp[j * 4 * W2 + i],
            ));
            assert_eq!(amp.raw(i as i64, j as i64), f64::from(a));
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn results_do_not_depend_on_the_thread_count() {
    let l2 = l2_level();
    let c = cfg(1.0);
    let (a, oa) = run_mem(&l2, &c, plan_tiled(&c), 1);
    let (b, ob) = run_mem(&l2, &c, plan_tiled(&c), 5);
    assert!(
        a.z.iter()
            .zip(&b.z)
            .all(|(x, y)| x.to_bits() == y.to_bits()),
        "terrain differs between 1 and 5 threads"
    );
    assert!(
        a.amp
            .iter()
            .zip(&b.amp)
            .all(|(x, y)| x.to_bits() == y.to_bits())
    );
    assert_eq!(oa.last.eroded_m_cells, ob.last.eroded_m_cells);
    assert_eq!(oa.last.sinks, ob.last.sinks);
}

/// Rotates a `w × h` array by `k` columns east.
fn rotate<T: Copy>(v: &[T], w: usize, k: usize) -> Vec<T> {
    let mut out = v.to_vec();
    for (c, o) in out.iter_mut().enumerate() {
        let (i, j) = (c % w, c / w);
        *o = v[j * w + (i + w - k % w) % w];
    }
    out
}

#[test]
fn the_seam_leaves_no_mark() {
    let l2 = l2_level();
    // Without noise nothing depends on where the seam is: shifting the L2 planet by 24 columns
    // (a whole number of L3 cells, 96 of them) shifts the amplified planet by 96 columns.
    let mut c = cfg(1.0);
    c.p.seed_relief = 0.0;
    c.p.thermal_noise = 0.0;
    let (base, _) = run_mem(&l2, &c, plan_tiled(&c), 2);
    let shift = 24usize;
    let shifted = Level {
        z: rotate(&l2.z, W2, shift),
        area: rotate(&l2.area, W2, shift),
        wgt: rotate(&l2.wgt, W2, shift),
        kfac: rotate(&l2.kfac, W2, shift),
        lake: rotate(&l2.lake, W2, shift),
        ..l2.clone()
    };
    let (rot, _) = run_mem(&shifted, &c, plan_tiled(&c), 2);
    let want = rotate(&base.z, 4 * W2, 4 * shift);
    let worst = want
        .iter()
        .zip(&rot.z)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst < 0.05,
        "a rotated planet amplifies differently by up to {worst} m"
    );
    // And the seam columns' steps are like any other column pair's.
    let gw = 4 * W2;
    let step = |i0: usize| -> f64 {
        (0..4 * H2)
            .map(|j| f64::from((base.z[j * gw + (i0 + 1) % gw] - base.z[j * gw + i0]).abs()))
            .sum::<f64>()
    };
    let seam = step(gw - 1);
    let typical: f64 = (0..gw - 1).map(step).sum::<f64>() / (gw - 1) as f64;
    assert!(
        seam < 1.6 * typical,
        "seam step {seam} against a typical {typical}"
    );
}

#[test]
fn drama_scales_the_slopes_and_the_detail() {
    let l2 = l2_level();
    let mut slopes = Vec::new();
    let mut amps = Vec::new();
    for drama in [0.0, 1.0, 2.0] {
        let mut c = cfg(drama);
        c.p.thermal_iterations = 8;
        c.p.thermal_slope = 0.08; // low enough that the test planet's slopes exceed it
        let (s, _) = run_mem(&l2, &c, plan_whole(), 2);
        let win = Win::full(4 * W2, 4 * H2);
        let mut relief = vec![0.0f32; win.cells()];
        relief5(&win, &s.z, &mut relief);
        let land: Vec<f32> = relief
            .iter()
            .zip(&s.z)
            .filter(|&(_, &z)| z > 0.0)
            .map(|(&r, _)| r)
            .collect();
        slopes.push(land.iter().map(|&r| f64::from(r)).sum::<f64>() / land.len() as f64);
        amps.push(s.amp.iter().map(|&a| f64::from(a)).sum::<f64>() / s.amp.len() as f64);
    }
    assert!(
        slopes[0] < slopes[1] && slopes[1] < slopes[2],
        "relief by drama {slopes:?}"
    );
    assert_eq!(amps[0], 0.0, "drama 0 has no detail");
    assert!(
        amps[1] > 0.0 && amps[2] > 1.3 * amps[1],
        "detail by drama {amps:?}"
    );
}

#[test]
fn the_detail_amplitude_fades_along_channels_and_stays_under_its_cap() {
    let l2 = l2_level();
    let c = cfg(1.0);
    let (s, out) = run_mem(&l2, &c, plan_whole(), 2);
    let max = s.amp.iter().copied().fold(0.0f32, f32::max);
    assert!(
        max > 0.0 && f64::from(max) <= c.p.detail.max_blocks + 1e-6,
        "{max}"
    );
    // Where the effective drainage area is past the second limit (250 km²) the amplitude is 0.
    let [_, a1] = c.p.detail.channel_area_km2;
    let (mut channels, mut rough) = (0usize, 0usize);
    for (k, &d) in out.preview.drainage.iter().enumerate() {
        if out.preview.z_m[k] <= 0.0 {
            continue;
        }
        if f64::from(d) >= a1.log10() {
            channels += 1;
            assert_eq!(
                out.preview.amp_blocks[k], 0.0,
                "cell {k} drains {d} (log km²)"
            );
        } else if out.preview.amp_blocks[k] > 0.0 {
            rough += 1;
        }
    }
    assert!(channels > 0, "the test planet has rivers past {a1} km²");
    assert!(rough > channels, "and rough ground away from them");
}

/// The coupled pipeline with the young and amplify stages on a small planet.
fn small_pipeline(myr: f64) -> String {
    let json = with_timeline(
        planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(myr),
    )
    .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = serde_json::json!(3000);
    v["resolution"]["l1_r"] = serde_json::json!(64);
    v["resolution"]["l2_r"] = serde_json::json!(32);
    v["resolution"]["l3_r"] = serde_json::json!(16);
    v["keyframes"]["size"] = serde_json::json!([128, 64]);
    v["keyframes"]["every_myr"] = serde_json::json!(20.0);
    v["seed"] = serde_json::json!(3);
    v.as_object_mut().unwrap().remove("deposits");
    v["young"] = serde_json::json!({"myr": 1.0, "steps": 2});
    v["amplify"] = serde_json::json!({
        "erosion_iterations": 4, "thermal_iterations": 3, "deposition_iterations": 2,
        "coarse_iteration_factor": 1.0, "repair_margin_cells": 8,
        "tile_cells": 256, "stream": "always", "preview_width": 128
    });
    v.to_string()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-amplify-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn finish(
    json: &str,
    dir: &std::path::Path,
    threads: usize,
) -> (Session, crate::world::WorldState) {
    let s = Session::create(
        json,
        "",
        SessionOptions {
            out_dir: dir.to_path_buf(),
            threads,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap();
    loop {
        match s.advance(std::time::Duration::from_secs(60)).unwrap() {
            AdvanceStatus::Done => break,
            AdvanceStatus::Cancelled => panic!("cancelled"),
            _ => {}
        }
    }
    let world = s.checkpoint().unwrap().world;
    (s, world)
}

#[test]
fn the_coupled_pipeline_amplifies_on_any_thread_count_and_registers_its_layers() {
    let json = small_pipeline(40.0);
    let p = crate::preset::Preset::from_json(&json).unwrap();
    let names: Vec<String> = p.stages().iter().map(|s| s.name().to_owned()).collect();
    assert_eq!(
        names,
        [
            "plates",
            "climate",
            "surface",
            "structure",
            "young",
            "amplify"
        ]
    );
    let d1 = temp_dir("pipe1");
    let d3 = temp_dir("pipe3");
    let (s1, w1) = finish(&json, &d1, 1);
    let (_s3, w3) = finish(&json, &d3, 3);
    // The young landscape and the amplifier ran, with no new sink.
    for f in [
        young::field::ELEVATION_M,
        young::field::AREA_M2,
        young::field::LAKE_M,
        field::ELEVATION,
        field::DETAIL,
        field::DRAINAGE,
    ] {
        assert!(w1.fields.contains_key(f), "{f}");
    }
    let note: Value = serde_json::from_str(&w1.notes["amplify"]).unwrap();
    assert_eq!(note["sinks"], 0, "{note}");
    assert_eq!(note["mode"], "streamed");
    assert_eq!(note["grid"]["w"], 1024);
    let young: Value = serde_json::from_str(&w1.notes["young"]).unwrap();
    assert!(young["after"]["land_z_p50_m"].as_f64().is_some(), "{young}");
    // Balanced uplift keeps the relief the deep-time run left (it neither planes it nor doubles it).
    let (before, after) = (
        young["before"]["land_z_p90_m"].as_f64().unwrap(),
        young["after"]["land_z_p90_m"].as_f64().unwrap(),
    );
    assert!(after > 0.6 * before && after < 1.5 * before, "{young}");
    assert_eq!(young["sinks"], 0, "{young}");
    // The same bytes on 1 and 3 threads, in the tile packs and in the published fields.
    for rel in [tiles::ELEVATION_PATH, tiles::AMPLITUDE_PATH] {
        assert_eq!(
            std::fs::read(d1.join(rel)).unwrap(),
            std::fs::read(d3.join(rel)).unwrap(),
            "{rel} differs between 1 and 3 threads"
        );
    }
    assert_eq!(w1.fields[field::ELEVATION], w3.fields[field::ELEVATION]);
    assert_eq!(
        w1.fields[young::field::ELEVATION_M],
        w3.fields[young::field::ELEVATION_M]
    );
    // The finished folder lists the layers, with the CRC of the file on disk.
    let manifest: Value = serde_json::from_str(&s1.finalize_write().unwrap()).unwrap();
    let layers = manifest["layers"].as_array().unwrap();
    let e = layers
        .iter()
        .find(|l| l["name"] == "elevation")
        .expect("elevation is listed");
    assert_eq!(e["grid"], "L3");
    let bytes = std::fs::read(d1.join(e["path"].as_str().unwrap())).unwrap();
    assert_eq!(e["crc32"], crc32fast::hash(&bytes));
    assert!(manifest["grids"]["L3"]["w"] == 1024);
    assert!(layers.iter().any(|l| l["name"] == "detail/amplitude"));
    let _ = std::fs::remove_dir_all(&d1);
    let _ = std::fs::remove_dir_all(&d3);
}

#[test]
fn a_checkpoint_before_the_young_stage_resumes_to_the_same_terrain() {
    let json = small_pipeline(30.0);
    let d = temp_dir("resume");
    let s = Session::create(
        &json,
        "",
        SessionOptions {
            out_dir: d.clone(),
            threads: 2,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap();
    // Run to the end of the coupled group (structure), checkpoint, then finish two ways.
    loop {
        let st = s.advance(std::time::Duration::from_secs(60)).unwrap();
        if st == AdvanceStatus::StageDone && s.progress().stage_index >= 4 {
            break;
        }
    }
    let ck = s.checkpoint().unwrap();
    assert_eq!(ck.next_stage_name(), Some("young"));
    let finish_from = |threads: usize, name: &str| {
        let dir = temp_dir(name);
        let s = Session::resume(
            ck.clone(),
            SessionOptions {
                out_dir: dir.clone(),
                threads,
                mem_cap: 0,
            },
            Arc::new(NoGauge),
        )
        .unwrap();
        loop {
            if s.advance(std::time::Duration::from_secs(60)).unwrap() == AdvanceStatus::Done {
                break;
            }
        }
        let bytes = std::fs::read(dir.join(tiles::ELEVATION_PATH)).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        bytes
    };
    assert_eq!(finish_from(1, "r1"), finish_from(4, "r4"));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn the_amplify_section_is_validated_and_needs_the_young_one() {
    let json = small_pipeline(10.0);
    let edit = |f: &dyn Fn(&mut serde_json::Value)| -> String {
        let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
        f(&mut v);
        match crate::preset::Preset::from_json(&v.to_string()) {
            Err(CoreError::PresetInvalid { path, .. }) => path,
            other => panic!("expected an invalid preset, got {:?}", other.map(|_| ())),
        }
    };
    assert_eq!(
        edit(&|v| v["amplify"]["thermal_rate"] = serde_json::json!(0.5)),
        "$.amplify.thermal_rate"
    );
    assert_eq!(
        edit(&|v| v["amplify"]["nope"] = serde_json::json!(1)),
        "$.amplify"
    );
    // The editor's "Terrain carving" (settings audit): the default is medium (3); the whole editor range
    // 0.25..=10 and the core's (0, 100] are accepted, nothing beyond.
    assert_eq!(AmplifyParams::default().incision_cap, 3.0);
    for ok in [0.25, 0.5, 3.0, 10.0, 100.0] {
        let p = AmplifyParams {
            incision_cap: ok,
            ..AmplifyParams::default()
        };
        p.validate().unwrap();
    }
    for bad in [0.0, -1.0, 100.5, f64::NAN] {
        let p = AmplifyParams {
            incision_cap: bad,
            ..AmplifyParams::default()
        };
        assert!(p.validate().is_err(), "{bad}");
    }
    assert_eq!(
        edit(&|v| v["amplify"]["incision_cap"] = serde_json::json!(101.0)),
        "$.amplify.incision_cap"
    );
    assert_eq!(
        edit(&|v| v["amplify"]["tile_cells"] = serde_json::json!(300)),
        "$.amplify.tile_cells"
    );
    assert_eq!(
        edit(&|v| {
            v.as_object_mut().unwrap().remove("young");
        }),
        "$.amplify"
    );
    assert_eq!(
        edit(&|v| v["resolution"]["l3_r"] = serde_json::json!(32)),
        "$.resolution.l3_r"
    );
    assert_eq!(
        edit(&|v| v["surface"]["drama"] = serde_json::json!(3.0)),
        "$.surface.drama"
    );
    assert_eq!(
        edit(&|v| v["young"]["steps"] = serde_json::json!(0)),
        "$.young.steps"
    );
    assert_eq!(
        edit(&|v| v["amplify"]["detail"]["max_blocks"] = serde_json::json!(9.0)),
        "$.amplify.detail.max_blocks"
    );
}

#[test]
fn doublings_and_strides() {
    assert_eq!(doublings(16, 2), Some(3));
    assert_eq!(doublings(8, 2), Some(2));
    assert_eq!(doublings(4, 2), Some(1));
    assert_eq!(doublings(2, 2), Some(0));
    assert_eq!(doublings(12, 2), None);
    assert_eq!(doublings(3, 2), None);
    assert_eq!(preview_stride(8192, 2048), 4);
    assert_eq!(preview_stride(16384, 2048), 8);
    assert_eq!(preview_stride(1024, 2048), 1);
    assert_eq!(preview_stride(1536, 512), 3);
}

#[test]
fn every_pipeline_preset_adds_little_by_streaming_when_it_must() {
    // Plan §5.7.5: "any preset whose estimate exceeds its cap streams L3 automatically": the
    // stage estimate assumes the streamed footprint exactly when the whole raster would not fit.
    for name in [
        "pipeline_quick_16k",
        "pipeline_full_16k",
        "pipeline_quick_32k",
        "pipeline_full_32k",
    ] {
        let json = with_timeline(
            planet_builtin(name).unwrap(),
            Some("scattered_continents"),
            Some(500.0),
        )
        .unwrap();
        let est: Value =
            serde_json::from_str(&crate::session::estimate(&json, "").unwrap()).unwrap();
        let stages = est["stages"].as_array().unwrap();
        let amp = stages.iter().find(|s| s["name"] == "amplify").unwrap();
        let young = stages.iter().find(|s| s["name"] == "young").unwrap();
        let preset = crate::preset::Preset::from_json(&json).unwrap();
        let crate::preset::PresetBody::Planet(p) = &preset.body else {
            panic!("a planet preset");
        };
        let world = crate::surface::estimate(p).bytes;
        let gib = |b: u64| b as f64 / (1u64 << 30) as f64;
        let own = amp["bytes"].as_u64().unwrap() - world;
        eprintln!(
            "{name}: young {:.2} GiB, amplify {:.2} GiB of which the stage's own {:.2} GiB, {:.0} s on one core",
            gib(young["bytes"].as_u64().unwrap()),
            gib(amp["bytes"].as_u64().unwrap()),
            gib(own),
            amp["seconds"].as_f64().unwrap(),
        );
        // What the amplifier adds to the world stays under 2.5 GiB on every preset: the whole
        // raster when it fits (67 B a cell: 2.1 GiB at 16k quick), tiles when it does not (32k
        // full would need 8 GiB whole).
        assert!(own < (5u64 << 29), "{name}: the stage adds {own} bytes");
    }
}

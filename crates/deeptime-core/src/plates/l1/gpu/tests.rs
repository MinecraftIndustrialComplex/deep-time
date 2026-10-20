// SPDX-License-Identifier: LGPL-3.0-or-later
//! The GPU relink against the CPU's, the reference (`docs/design/gpu-compute.md` §8): on a small
//! planet stepped through rifting, drift and subduction, at several steps, every link the device
//! settles equals the CPU's but at ties, every cell it leaves goes to the CPU unchanged, and the
//! layers agree to the f32 rounding of the weights. Skipped without an adapter (Mesa's lavapipe
//! counts: the test VM has no GPU).

use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use super::super::DeviceLinks;
use super::relink;
use crate::log::LogRing;
use crate::params::{PlanetPreset, plates_builtin};
use crate::plates::geometry;
use crate::plates::l1::sample_with;
use crate::plates::{PlateState, PlatesStage};
use crate::preset::{Preset, PresetBody, with_timeline};
use crate::session::NoGauge;
use crate::sphere::{Coupling, SampleTree};
use crate::stage::{Stage, StageContext};
use crate::world::WorldState;

/// A small planet: `n` samples, `myr` of history (as `plates/tests.rs` makes them).
fn preset(n: u32, myr: f64, start: &str, seed: u64) -> PlanetPreset {
    let json = with_timeline(
        plates_builtin("plates_quick_16k").unwrap(),
        Some(start),
        Some(myr),
    )
    .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = serde_json::json!(n);
    v["resolution"]["l1_r"] = serde_json::json!(64);
    v["keyframes"]["size"] = serde_json::json!([128, 64]);
    v["keyframes"]["every_myr"] = serde_json::json!(20.0);
    v["seed"] = serde_json::json!(seed);
    match Preset::from_json(&v.to_string()).unwrap().body {
        PresetBody::Planet(p) => *p,
        other => panic!("not a planet preset: {other:?}"),
    }
}

#[derive(Default, Debug)]
struct Tally {
    cells: usize,
    /// Cells the device left to the CPU.
    on_cpu: usize,
    /// Device-settled cells whose triangle or nearest sample differs from the CPU's (ties).
    links_differ: usize,
    /// The largest weight difference where the links agree.
    max_dw: f32,
    /// Layer values off by more than 1e-4 of the layer's largest magnitude.
    values_off: usize,
    values: usize,
    /// The layers holding values off.
    layers_off: Vec<&'static str>,
}

fn compare(gpu: &Arc<deeptime_gpu::Gpu>, stage: &PlatesStage, st: &PlateState, t: &mut Tally) {
    let cache = stage.cache.as_ref().expect("a step builds the cache");
    let geo = &cache.geo;
    let base = match &cache.coupling {
        Some(c) => c.clone(),
        None => Coupling::build(
            &stage.l1_grid().unwrap(),
            &geo.mesh,
            &SampleTree::new(geo.points()),
        ),
    };
    let pos = geometry::positions(st, geo);
    let env = stage.env(&stage.preset.tectonics, stage.done.max(1));
    let scale = &stage.preset.scale;

    let device = DeviceLinks(relink(gpu, st, geo, &base, &pos, &env).expect("the device relinks"));
    let mut on_cpu = base.clone();
    let cpu = sample_with(st, geo, &mut on_cpu, &pos, &env, scale, None);
    let mut on_gpu = base.clone();
    let dev = sample_with(st, geo, &mut on_gpu, &pos, &env, scale, Some(gpu));

    let (a, b) = (on_cpu.links(), on_gpu.links());
    assert_eq!(a.len(), device.len());
    t.cells += a.len();
    for c in 0..a.len() {
        match device.get(c) {
            None => {
                t.on_cpu += 1;
                assert_eq!(a[c], b[c], "cell {c}, left to the CPU, linked differently");
            }
            Some(d) => {
                assert_eq!(d, b[c], "cell {c}: the device's link was not the one used");
                if a[c].tri != d.tri || a[c].nearest != d.nearest {
                    t.links_differ += 1;
                } else {
                    for k in 0..3 {
                        t.max_dw = t.max_dw.max((a[c].w[k] - d.w[k]).abs());
                    }
                }
            }
        }
    }
    assert_eq!(cpu.len(), dev.len());
    for ((na, va), (nb, vb)) in cpu.iter().zip(&dev) {
        assert_eq!(na, nb);
        let big = va
            .iter()
            .filter(|x| x.is_finite())
            .fold(1.0f32, |m, x| m.max(x.abs()));
        t.values += va.len();
        // NaN marks "none" in several layers (orogeny age, fold trend, continental heat flow).
        let off = va
            .iter()
            .zip(vb)
            .filter(|(x, y)| {
                !(x == y || (x.is_nan() && y.is_nan()) || (*x - *y).abs() <= 1e-4 * big)
            })
            .count();
        if off > 0 && !t.layers_off.contains(na) {
            t.layers_off.push(*na);
        }
        t.values_off += off;
    }
}

#[test]
fn the_gpu_relink_matches_the_cpu_but_at_ties() {
    let gpu = match deeptime_gpu::shared(true) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("skipped: {e}");
            return;
        }
    };
    eprintln!("adapter: {}", gpu.describe());
    let p = preset(6000, 60.0, "supercontinent", 7);
    let mut stage = PlatesStage::new(p.clone());
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();
    let mut world = WorldState::new(p.seed, p.scale.circumference_blocks, p.t_start_years());
    let log = LogRing::default();
    let state = AtomicU8::new(0);
    let gauge = NoGauge;
    let mut t = Tally::default();
    let mut checked = 0;
    while !stage.is_done() {
        let ctx = StageContext {
            pool: &pool,
            log: &log,
            out_dir: std::path::Path::new("."),
            until: std::time::Instant::now(),
            gauge: &gauge,
            mem_cap: u64::MAX,
            state: &state,
            history: None,
            needed_peak: None,
            compute: crate::compute::ComputeMode::Cpu,
        };
        stage.step(&mut world, &ctx).unwrap();
        if stage.done.is_multiple_of(5) {
            let st = world.plates.as_ref().unwrap();
            compare(&gpu, &stage, st, &mut t);
            checked += 1;
        }
    }
    eprintln!("{checked} samplings: {t:?}");
    assert!(checked >= 3);
    // The device settles most cells (the boundary strips and gaps stay on the CPU).
    assert!(
        t.on_cpu * 5 < t.cells,
        "{} of {} cells left to the CPU",
        t.on_cpu,
        t.cells
    );
    // Ties: two samples equidistant, or a cell on a triangle's edge, to f32 rounding.
    assert!(
        t.links_differ * 1000 < t.cells,
        "{} links differ",
        t.links_differ
    );
    // Weights of f32 directions: about 1e-7 rad over a triangle a spacing (0.046 rad) across.
    assert!(t.max_dw < 1e-4, "weights differ by {}", t.max_dw);
    assert!(
        t.values_off * 1000 < t.values,
        "{} of {} values off, in {:?}",
        t.values_off,
        t.values,
        t.layers_off
    );
}

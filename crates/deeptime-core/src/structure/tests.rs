// SPDX-License-Identifier: LGPL-3.0-or-later
//! The P1-07 checks (plan §10.3 P1-07): on the coupled plates + surface + structure pipeline, the
//! baked horizon stacks are ordered top-down, every horizon is at least one block thick after
//! scaling and every class is valid; the deposit engine's history carries the trap structure and
//! the host fractions; plutons become bodies; a resumed run is the same run.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::*;
use crate::checkpoint::Checkpoint;
use crate::deposits::fields::FieldHistory;
use crate::params::planet_builtin;
use crate::preset::{Preset, with_timeline};
use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};

fn pipeline_json(n: u32, myr: f64, seed: u64) -> String {
    let json = with_timeline(
        planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(myr),
    )
    .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = serde_json::json!(n);
    v["resolution"]["l1_r"] = serde_json::json!(64);
    v["keyframes"]["size"] = serde_json::json!([128, 64]);
    v["keyframes"]["every_myr"] = serde_json::json!(20.0);
    v["seed"] = serde_json::json!(seed);
    // Plenty of plutons on the small test planet.
    v["structure"] = serde_json::json!({"pluton_rate_km2_myr": 2e-3});
    // These tests are about the structure stage; the deposit stage after it has its own
    // (`deposits::recorded_tests`).
    v.as_object_mut().unwrap().remove("deposits");
    // The young landscape and L3 amplification (P1-06) have their own tests.
    v.as_object_mut().unwrap().remove("young");
    v.as_object_mut().unwrap().remove("amplify");
    v.to_string()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-structure-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn session(json: &str, dir: &Path, threads: usize) -> Session {
    Session::create(
        json,
        "",
        SessionOptions {
            out_dir: dir.to_path_buf(),
            threads,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap()
}

fn finish(s: &Session) {
    loop {
        match s.advance(Duration::from_secs(60)).unwrap() {
            AdvanceStatus::Done => break,
            AdvanceStatus::Cancelled => panic!("cancelled"),
            _ => {}
        }
    }
}

#[test]
fn the_pipeline_bakes_ordered_block_thick_valid_stacks() {
    // 200 Myr: under the force balance the plates start at rest, and on this small planet the
    // first arcs and collisions (and so the first plutons) come later than under the old speed law.
    // Seed 8 since the erosion recalibration (2026-10-18): under the new law seed 7 emplaced no
    // pluton in 200 Myr (under the old law it still does), seeds 8 to 13 all do.
    let json = pipeline_json(3000, 200.0, 8);
    let dir = temp_dir("bake");
    let s = session(&json, &dir, 2);
    assert_eq!(
        s.stage_names(),
        [
            "plates",
            "climate",
            "surface",
            "volcanoes",
            "impacts",
            "structure"
        ]
    );
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let preset = match &Preset::from_json(&json).unwrap().body {
        crate::preset::PresetBody::Planet(p) => (**p).clone(),
        _ => unreachable!(),
    };

    // The acceptance: ordered top-down, ≥ one block thick after scaling, classes valid.
    let h = bake_horizons::bake(&world, &preset).unwrap();
    h.check().unwrap();
    assert_eq!(h.k, 32);
    // … and nothing a geologist would object to: no rock that was never as hot as the rock above
    // it, no sliver of metamorphic or plutonic rock, in the stored stacks and in the columns the
    // game builds from them (`realism`).
    h.check_bake(&Tables::get().min_blocks).unwrap();
    let cells = realism::census_of_cells(&h, &Rules::builtin(), &Limits::default(), 1);
    assert_eq!(cells.with_inversion, 0, "{:?}", cells.to_json());
    assert_eq!(cells.with_sliver, 0, "{:?}", cells.to_json());
    assert_eq!(cells.with_sheet, 0, "{:?}", cells.to_json());
    let columns = rule_columns(&h, 11, 3000);
    assert_eq!(columns.with_sliver, 0, "{:?}", columns.to_json());
    assert_eq!(columns.with_sheet, 0, "{:?}", columns.to_json());
    assert!(h.mean_horizons() > 2.0, "{}", h.mean_horizons());
    // The stack's top is the surface, and the layer round-trips through the format.
    let src = CellSource::new(&world, &preset).unwrap();
    for c in (0..h.cells()).step_by(97) {
        let want = (src.surface_y(c, &preset.scale).clamp(-511.0, 639.0) * 16.0).round();
        assert!((f64::from(h.cell(c).0[0]) - want).abs() <= 1.0, "cell {c}");
    }
    let layer = h.to_layer(deeptime_format::Codec::Raw).unwrap();
    let got = layer.horizon(5, 7, 1);
    let c = 7 * h.grid.w as usize + 5;
    assert_eq!(
        (got.top, got.class, got.key),
        (
            h.tops[c * 32 + 1],
            h.classes[c * 32 + 1],
            h.keys[c * 32 + 1]
        )
    );

    // Many classes appear: sediments, basement, metamorphic and mantle rocks.
    let mut seen = [false; crate::synth::EVERY_CLASS.len()];
    for &k in &h.classes {
        if k != bake_horizons::NO_CLASS {
            seen[usize::from(k)] = true;
        }
    }
    let used = seen.iter().filter(|&&x| x).count();
    assert!(used >= 12, "{used} classes in the stacks");

    // The deposit engine's history: every built-in type finds every field it reads.
    let hist = world.history.as_ref().expect("frames");
    let mut names = hist.names();
    // The regional fields the deposit engine derives on reading (`deposits::regional`).
    names.extend(crate::deposits::regional::derivable(&names));
    for t in crate::deposits::types::builtin().unwrap() {
        let missing: Vec<String> = t
            .fields()
            .into_iter()
            .filter(|f| !names.contains(f))
            .collect();
        assert!(missing.is_empty(), "{} lacks {missing:?}", t.id);
        for class in t.hosts.weights().unwrap().keys() {
            let f = format!("deeptime:host/{class}");
            assert!(names.contains(&f), "{f}");
        }
    }
    let frame = hist.frame(hist.frame_count() - 1).unwrap();
    let trap = &frame.fields[TRAP_FIELD];
    assert!(trap.iter().all(|&x| (0.0..=1.0 + 1e-6).contains(&x)));
    let hosts: Vec<&Vec<f32>> = frame
        .fields
        .iter()
        .filter(|(k, _)| k.starts_with(crate::deposits::fields::HOST_PREFIX))
        .map(|(_, v)| v)
        .collect();
    for c in 0..h.cells() {
        let sum: f32 = hosts.iter().map(|v| v[c]).sum();
        assert!(sum <= 1.0 + 0.02, "cell {c}: host fractions sum to {sum}");
    }

    // Plutons: emplaced, and the survivors are bodies of their class.
    let note: serde_json::Value = serde_json::from_str(&world.notes[STAGE_NAME]).unwrap();
    assert!(note["plutons"]["emplaced"].as_u64().unwrap() > 0, "{note}");
    let bodies = note["plutons"]["bodies"].as_u64().unwrap();
    // (The other bodies of the coupled group are the impact stage's breccia lenses, P2-08.)
    let plutons: Vec<&deeptime_format::Body> =
        world.bodies.iter().filter(|b| b.role == "pluton").collect();
    assert_eq!(plutons.len() as u64, bodies);
    assert!(
        world
            .bodies
            .iter()
            .all(|b| ["pluton", "aureole", "stock", "impact_breccia"].contains(&b.role.as_str()))
    );
    for b in plutons {
        let class = b.rock_class.as_deref().unwrap();
        assert!(
            ["granodiorite", "granite", "diorite"].contains(&class),
            "{class}"
        );
        assert!(b.age_ma.is_some_and(|a| a >= 0.0), "a pluton has its age");
        b.validate().unwrap();
    }
    // Their aureoles: bodies of hornfels and skarn zones, counted in the note, before the
    // plutons (a pluton paints over its aureole's inner part).
    let aureoles: Vec<&deeptime_format::Body> = world
        .bodies
        .iter()
        .filter(|b| b.role == "aureole")
        .collect();
    let a = &note["aureoles"];
    assert_eq!(
        aureoles.len() as u64,
        a["plutons"].as_u64().unwrap() + a["stocks"].as_u64().unwrap(),
        "{a}"
    );
    for b in &aureoles {
        assert!(b.rock_class.is_none() && !b.zones.is_empty() && b.age_ma.is_some());
        for z in &b.zones {
            let class = z.rock_class.as_deref().unwrap();
            assert!(["hornfels", "skarn"].contains(&class), "{class}");
        }
        b.validate().unwrap();
    }
    if let (Some(a), Some(p)) = (
        aureoles.last(),
        world.bodies.iter().find(|b| b.role == "pluton"),
    ) {
        assert!(a.id < p.id, "aureoles come before the plutons");
    }
    assert_eq!(note["horizons"]["check"], "ok");
    // Keyframe layers exist with valid values.
    let rock = &world.fields[field::ROCK].data;
    assert!(
        rock.iter()
            .all(|&k| k >= 0.0 && (k as usize) < crate::synth::EVERY_CLASS.len())
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_resumed_run_is_the_same_run() {
    let json = pipeline_json(2000, 30.0, 3);
    let dir = temp_dir("straight");
    let s = session(&json, &dir, 2);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let dir2 = temp_dir("resumed");
    let s2 = session(&json, &dir2, 2);
    for _ in 0..7 {
        assert_eq!(s2.advance(Duration::ZERO).unwrap(), AdvanceStatus::Running);
    }
    let ck = s2.checkpoint().unwrap();
    let bytes = ck.to_bytes().unwrap();
    let ck = Checkpoint::from_bytes(Path::new("ck"), &bytes).unwrap();
    let s3 = Session::resume(
        ck,
        SessionOptions {
            out_dir: dir2.clone(),
            threads: 3,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap();
    finish(&s3);
    let got = s3.checkpoint().unwrap().world;
    assert_eq!(
        got.surface.as_ref().unwrap().log,
        world.surface.as_ref().unwrap().log
    );
    assert_eq!(got.bodies, world.bodies);
    assert_eq!(got.history, world.history);
    assert_eq!(got.fields[field::ROCK].data, world.fields[field::ROCK].data);
    for d in [dir, dir2] {
        let _ = std::fs::remove_dir_all(d);
    }
}

// ---- What a geologist would object to (`realism`): property tests on random columns. They hold
// for any planet: no number of one run is in them.

use crate::plates::state::{CONTINENTAL, OCEANIC, OROGENY_ANDEAN, OROGENY_NONE, regime as reg};
use crate::structure::columns::{Unit, key};
use crate::structure::realism::{self, Findings, Limits, Rules};
use crate::surface::lithology::index;
use crate::surface::strata_log::{Layer, StrataLog};
use crate::synth::SplitMix;

/// One random cell: a strata log with a history (deposits, burial metamorphism, erosion), rock
/// pools, a geotherm.
struct RandomCell {
    log: Vec<Layer>,
    pools: [f32; pool::COUNT],
    crust_type: u8,
    regime: u8,
    orogeny: u8,
    orogeny_age: f32,
    regolith_m: f32,
    gradient_c_m: f64,
    top_active: bool,
}

impl RandomCell {
    fn new(rng: &mut SplitMix, paths: &Paths) -> RandomCell {
        const DEPOSITS: [&str; 13] = [
            "sandstone",
            "shale",
            "limestone",
            "mudstone",
            "basalt",
            "tuff",
            "conglomerate",
            "gypsum",
            "coal",
            "soil",
            "andesite",
            "till",
            "black_shale",
        ];
        let oceanic = rng.below(7) == 0;
        let mut log = StrataLog::new(1, 12);
        let mut age = -900.0f32;
        for _ in 0..rng.below(12) {
            match rng.below(10) {
                0..=5 => {
                    let class = index(DEPOSITS[rng.below(DEPOSITS.len() as u64) as usize]);
                    log.deposit(0, age, 30.0 + rng.below(2500) as f32, class, 0);
                    age = (age + 5.0 + rng.below(80) as f32).min(-1.0);
                }
                6 | 7 if !oceanic => {
                    let th = Thermal {
                        surface_c: 10.0,
                        gradient_c_m: (15 + rng.below(35)) as f64 / 1000.0,
                        burial: 1.0 + rng.below(2) as f64,
                    };
                    metamorphism::metamorphose(&mut log.column_mut(0), 0.0, &th, paths);
                }
                _ => {
                    log.erode(0, rng.below(3000) as f64);
                }
            }
        }
        let mut pools = if oceanic {
            crate::plates::pools::oceanic_column()
        } else {
            crate::plates::pools::continental_column()
        };
        for p in pools.iter_mut() {
            *p = match rng.below(10) {
                0..=2 => 0.0,
                // A thin leftover of a pool: what made the three-block gneiss.
                3 => 20.0 + rng.below(200) as f32,
                _ => *p * (rng.below(200) as f32 / 100.0),
            };
        }
        if !oceanic && rng.below(3) == 0 {
            pools[pool::MAFIC_VOLCANIC] = rng.below(3000) as f32;
        }
        RandomCell {
            log: log.column(0).to_vec(),
            pools,
            crust_type: if oceanic { OCEANIC } else { CONTINENTAL },
            regime: [
                reg::OCEAN_BASIN,
                reg::ARC,
                reg::COLLISION,
                reg::HOTSPOT,
                reg::BACK_ARC,
                reg::RIFT,
            ][rng.below(6) as usize],
            orogeny: [OROGENY_NONE, OROGENY_ANDEAN, OROGENY_HIMALAYAN][rng.below(3) as usize],
            orogeny_age: if rng.below(2) == 0 {
                f32::NAN
            } else {
                rng.below(200) as f32
            },
            regolith_m: [0.0, 0.5, 3.0, 40.0][rng.below(4) as usize],
            gradient_c_m: (15 + rng.below(34)) as f64 / 1000.0,
            top_active: rng.below(2) == 0,
        }
    }

    fn cell(&self) -> Cell<'_> {
        Cell {
            regolith_m: self.regolith_m,
            regolith_kind: crate::surface::weathering::SOIL,
            log: &self.log,
            pools: self.pools,
            crust_type: self.crust_type,
            regime: self.regime,
            orogeny: self.orogeny,
            orogeny_age: self.orogeny_age,
            depth_below_sea_m: if self.crust_type == OCEANIC {
                3000.0
            } else {
                -200.0
            },
            surface_c: 10.0,
            gradient_c_m: self.gradient_c_m,
            log_span_myr: 1000.0,
            top_active: self.top_active,
        }
    }
}

#[test]
fn no_column_holds_rock_that_was_never_as_hot_as_the_rock_above_it() {
    let tables = Tables::get();
    let rules = Rules::builtin();
    let p = StructureParams::default();
    let mut rng = SplitMix(0xDEE9_71AE);
    let (mut units, mut f) = (Vec::<Unit>::new(), Findings::default());
    let mut metamorphic = 0;
    for n in 0..4000 {
        let rc = RandomCell::new(&mut rng, &tables.paths);
        columns::build(&rc.cell(), &tables, &p, 0.0, 14_000.0, &mut units);
        // The keys: ordered, each span inside one key domain, the grade bands never decreasing.
        for (i, u) in units.iter().enumerate() {
            assert!(u.key[0] < u.key[1], "cell {n} unit {i}: {units:?}");
            assert_eq!(
                key::domain(u.key[0]),
                key::domain(u.key[1]),
                "cell {n} unit {i}: {units:?}"
            );
            if i > 0 {
                assert!(
                    units[i - 1].key[1] <= u.key[0],
                    "cell {n} unit {i}: {units:?}"
                );
            }
        }
        // The rock: every unit as a thick horizon, so only the order is judged.
        let col: Vec<realism::Visible> = units
            .iter()
            .enumerate()
            .map(|(i, u)| (-(i as i32) * 100, 100, u.class as u8))
            .collect();
        realism::check(&col, &rules, &Limits::default(), &mut f);
        assert!(
            f.inversions.is_empty(),
            "cell {n}: {:?} in {:?} (log {:?})",
            f.inversions,
            units
                .iter()
                .map(|u| (EVERY_CLASS[usize::from(u.class)], u.thickness_m))
                .collect::<Vec<_>>(),
            rc.log
        );
        metamorphic += usize::from(col.iter().any(|v| rules.metamorphic[usize::from(v.2)]));
    }
    assert!(
        metamorphic > 1000,
        "{metamorphic} of the cells hold metamorphic rock"
    );
}

/// P1-23: the smoothing's sample lattice does not show. The samples of a cell are the lattice
/// points in its window; whatever the cell's place between them, their weighted mean position is
/// the cell (so a tilted bed stays a plane instead of stepping with the lattice's period), their
/// spread is the same, and a sample at the window's edge weighs nothing (so one entering or
/// leaving changes nothing). The cut-off Gaussian this replaced fails all three: its mean swung
/// 0.23 cells at σ 10 and 0.68 at σ 20, and an edge sample held 2 % of the weight.
#[test]
fn the_smoothing_lattice_leaves_no_period() {
    use bake_horizons::{SMOOTH_RADIUS_SAMPLES, smooth_axis_weight, smooth_stride};
    for sigma in [2.5, 5.0, 10.0, 20.0, 37.0] {
        let step = smooth_stride(sigma);
        let reach = step * SMOOTH_RADIUS_SAMPLES;
        let mut widths = Vec::new();
        for a in 0..step {
            // The lattice points (multiples of step) in the window of the cell at `a`.
            let pts: Vec<f64> = (-2 * SMOOTH_RADIUS_SAMPLES..=2 * SMOOTH_RADIUS_SAMPLES)
                .map(|l| (l * step - a) as f64)
                .filter(|d| d.abs() <= reach as f64)
                .collect();
            let w: Vec<f64> = pts.iter().map(|&d| smooth_axis_weight(d, sigma)).collect();
            let sum: f64 = w.iter().sum();
            assert!(sum > 0.0);
            let mean: f64 = pts.iter().zip(&w).map(|(d, w)| d * w).sum::<f64>() / sum;
            let var: f64 = pts.iter().zip(&w).map(|(d, w)| d * d * w).sum::<f64>() / sum;
            assert!(
                mean.abs() < 0.01 * step as f64 / 4.0 + 0.005,
                "sigma {sigma}, cell {a}: the samples' mean lies {mean} cells off the cell"
            );
            widths.push(var.sqrt());
            for (d, w) in pts.iter().zip(&w) {
                if d.abs() >= reach as f64 {
                    assert_eq!(*w, 0.0, "sigma {sigma}: a sample at the edge has weight");
                }
                assert!(*w >= 0.0);
            }
            // Nothing outside the window, and the weight falls away from the cell.
            assert_eq!(smooth_axis_weight(reach as f64 + 1.0, sigma), 0.0);
            assert!(smooth_axis_weight(0.0, sigma) > smooth_axis_weight(step as f64, sigma));
        }
        let (lo, hi) = widths
            .iter()
            .fold((f64::MAX, f64::MIN), |(lo, hi), &x| (lo.min(x), hi.max(x)));
        assert!(
            hi / lo < 1.03,
            "sigma {sigma}: the width changes {lo}..{hi} with the cell"
        );
        assert!(
            (0.7..=1.0).contains(&(lo / sigma)),
            "sigma {sigma}: effective width {lo}"
        );
    }
}

/// P1-23, the same on stacks: a bed whose base is a plane tilted across the grid keeps a plane
/// base after smoothing, with no step every lattice period.
#[test]
fn a_tilted_bed_stays_a_plane_under_smoothing() {
    let tables = Tables::get();
    let p = StructureParams::default();
    let grid = deeptime_format::GridDesc::for_planet("L1", 4096, 16).unwrap();
    let (w, h) = (grid.w as usize, grid.h as usize);
    let k = 4usize;
    let n = w * h;
    let mut tops = vec![bake_horizons::NO_TOP; n * k];
    let mut classes = vec![bake_horizons::NO_CLASS; n * k];
    let mut keys = vec![[bake_horizons::NO_KEY; 2]; n * k];
    let (sandstone, granite) = (tables.cls.sandstone as u8, tables.cls.granite as u8);
    // The surface is level at y 200; the sandstone's base falls 2 blocks a cell eastwards over
    // the middle of the grid (far from the seam, where a plane cannot wrap).
    let base = |i: usize| 100 - 2 * (i as i32 - w as i32 / 2);
    for c in 0..n {
        let i = c % w;
        tops[c * k] = 200 * 16;
        classes[c * k] = sandstone;
        keys[c * k] = [5000, 6000];
        tops[c * k + 1] = (base(i).clamp(-300, 180) * 16) as i16;
        classes[c * k + 1] = granite;
        keys[c * k + 1] = [59_700, 59_900];
    }
    let hz = bake_horizons::Horizons {
        grid,
        k: k as u32,
        min_y: -352,
        tops,
        classes,
        keys,
        tongues: bake_horizons::tongue_params(&p),
        min_blocks: Vec::new(),
    }
    .smoothed(10.0, &tables);
    // Along a middle row, where the base is a plane within the window: the smoothed base's
    // second difference is rounding, at every phase of the four-cell lattice.
    let j = h / 2;
    let top = |i: usize| f64::from(hz.cell(j * w + i).0[1]) / 16.0;
    let mut worst = 0.0f64;
    for i in w / 2 - 20..w / 2 + 20 {
        assert_eq!(hz.cell(j * w + i).0.len(), 2, "two horizons at cell {i}");
        let d2 = top(i - 1) - 2.0 * top(i) + top(i + 1);
        worst = worst.max(d2.abs());
        assert!(
            (top(i) - f64::from(base(i))).abs() < 0.25,
            "cell {i}: {} {}",
            top(i),
            base(i)
        );
    }
    assert!(
        worst < 0.2,
        "the base bends by {worst} blocks between neighbours"
    );
}

/// A small planet's L1 grid of provinces, each a random cell (so a metamorphic belt lies beside
/// a basin, an ocean beside a craton), baked like the bake does.
fn random_horizons(seed: u64, sigma: f64) -> bake_horizons::Horizons {
    let tables = Tables::get();
    let p = StructureParams::default();
    let grid = deeptime_format::GridDesc::for_planet("L1", 1024, 16).unwrap();
    let (w, h) = (grid.w as usize, grid.h as usize);
    let mut rng = SplitMix(seed);
    let side = 8;
    let provinces: Vec<(RandomCell, f64)> = (0..(w / side) * (h / side))
        .map(|_| {
            (
                RandomCell::new(&mut rng, &tables.paths),
                40.0 + rng.below(300) as f64,
            )
        })
        .collect();
    let k = p.horizons as usize;
    let n = w * h;
    let mut tops = vec![bake_horizons::NO_TOP; n * k];
    let mut classes = vec![bake_horizons::NO_CLASS; n * k];
    let mut keys = vec![[bake_horizons::NO_KEY; 2]; n * k];
    let mut units = Vec::new();
    for c in 0..n {
        let (i, j) = (c % w, c / w);
        let (rc, y) = &provinces[(j / side) * (w / side) + i / side];
        // Relief inside a province too.
        let surface_y = y + 25.0 * ((i as f64 * 0.7).sin() + (j as f64 * 0.45).cos());
        columns::build(&rc.cell(), &tables, &p, 0.0, 14_000.0, &mut units);
        bake_horizons::bake_cell(
            &units,
            surface_y,
            -352,
            639,
            20.0,
            &tables.min_blocks,
            &mut tops[c * k..(c + 1) * k],
            &mut classes[c * k..(c + 1) * k],
            &mut keys[c * k..(c + 1) * k],
        );
    }
    bake_horizons::Horizons {
        grid,
        k: k as u32,
        min_y: -352,
        tops,
        classes,
        keys,
        tongues: bake_horizons::tongue_params(&p),
        min_blocks: tables.min_blocks.iter().map(|&b| b as u8).collect(),
    }
    .smoothed(sigma, &tables)
}

/// The columns the game builds from `hz` (the column rule at random blocks, each under its own
/// first top), every one asserted free of grade inversions; returns their census.
fn rule_columns(hz: &bake_horizons::Horizons, seed: u64, n: usize) -> realism::Census {
    let rules = Rules::builtin();
    let layer = hz.to_layer(deeptime_format::Codec::Raw).unwrap();
    let c = i64::from(hz.grid.w) * i64::from(hz.grid.r);
    let mut rng = SplitMix(seed);
    let (mut stack, mut raw, mut col, mut f) =
        (Vec::new(), Vec::new(), Vec::new(), Findings::default());
    let mut census = realism::Census::default();
    for _ in 0..n {
        let x = rng.below(c as u64) as i64 - c / 2;
        let zp = deeptime_format::Projection::of(c as f64).pole_z() as i64;
        let z = rng.below(2 * zp as u64) as i64 - zp;
        layer.stack_at(x, z, hz.min_y, &mut stack);
        raw.clear();
        raw.extend(stack.iter().map(|&(top, class, _)| (top, class)));
        realism::visible(&raw, raw[0].0, hz.min_y, &mut col);
        realism::check(&col, &rules, &Limits::default(), &mut f);
        assert!(
            f.inversions.is_empty(),
            "column ({x}, {z}): {:?} in {:?}",
            f.inversions,
            col.iter()
                .map(|v| (EVERY_CLASS[usize::from(v.2)], v.0, v.1))
                .collect::<Vec<_>>()
        );
        census.add(&col, &f, &rules);
    }
    census
}

#[test]
fn baked_stacks_and_blended_columns_have_no_grade_inversion_and_no_sliver() {
    let tables = Tables::get();
    let rules = Rules::builtin();
    for (seed, sigma) in [(1u64, 0.0), (2, 2.5), (3, 10.0), (4, 10.0)] {
        let hz = random_horizons(seed, sigma);
        hz.check().unwrap();
        hz.check_bake(&tables.min_blocks).unwrap();
        // The stored stacks: nothing to object to.
        let cells = realism::census_of_cells(&hz, &rules, &Limits::default(), 1);
        assert_eq!(
            cells.with_inversion,
            0,
            "seed {seed}: {:?}",
            cells.to_json()
        );
        assert_eq!(cells.with_sliver, 0, "seed {seed}: {:?}", cells.to_json());
        assert_eq!(cells.with_sheet, 0, "seed {seed}: {:?}", cells.to_json());
        assert!(
            cells.with_metamorphic * 10 > cells.columns,
            "seed {seed}: the test planet has metamorphic rock: {:?}",
            cells.to_json()
        );
        // The columns the game builds (sixteen stacks blended by key): no inversion (asserted
        // column by column in `rule_columns`).
        // … and no sliver: the rule drops the thin edge a metamorphic or plutonic unit would
        // have where it ends sideways (`deeptime_format::strata::drop_slivers`).
        let columns = rule_columns(&hz, seed ^ 0x5EED, 6000);
        assert!(columns.with_metamorphic > 0);
        assert_eq!(
            columns.with_sliver,
            0,
            "seed {seed}: {:?}",
            columns.to_json()
        );
        assert_eq!(
            columns.with_sheet,
            0,
            "seed {seed}: {:?}",
            columns.to_json()
        );
    }
}

/// Whatever the seed and the start, and however busy the belts, the stage ends without a panic
/// and its end-of-run check holds: histories longer than one Noddy history (127 events) are
/// undone as chains, and the Noddy port still agrees with the column rule on every sampled
/// point. (The settings here make far more structures than any preset: the longest history a
/// sampled column meets must pass one Noddy history in at least one of the runs, or the test
/// is not testing the case that used to panic. 80 Myr: on the Lambert planet after plate tasks 3
/// and 4, 50 Myr reached at most 122 events a column over six seeds; 80 reaches 270 and 135.
/// Since the force balance (plate task 7, 2026-10-05) the plates start at rest and collisions
/// fold less: 80 Myr reached 92 and 120 Myr 124, so the runs were 200 Myr. With the climate
/// snapshot every 20 Myr (P1-19 #27, 2026-10-18) 200 Myr reached 101 over the four runs (210 on
/// one run with 10-Myr snapshots); 300 Myr reaches 162 to 286 on every run, so the runs are
/// 300 Myr.)
#[test]
fn the_stage_never_panics_however_dense_the_structures() {
    let mut longest = 0u64;
    let mut long_columns = 0u64;
    for (seed, start) in [
        (1u64, "supercontinent"),
        (2, "supercontinent"),
        (3, "scattered_continents"),
        (4, "scattered_continents"),
    ] {
        let json = with_timeline(
            planet_builtin("pipeline_quick_16k").unwrap(),
            Some(start),
            Some(300.0),
        )
        .unwrap();
        let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
        v["resolution"]["l0_samples"] = serde_json::json!(3000);
        v["resolution"]["l1_r"] = serde_json::json!(64);
        v["keyframes"]["size"] = serde_json::json!([128, 64]);
        v["keyframes"]["every_myr"] = serde_json::json!(40.0);
        v["seed"] = serde_json::json!(seed);
        v["structure"] = serde_json::json!({"deformation": {
            "density": 10.0, "intrusions": 10.0, "max_overlap": 16, "overprint_myr": 1000.0,
            "fault_length_km": [150.0, 100.0, 150.0], "fold_wavelength_km": [32.0, 30.0, 32.0]
        }});
        for stage in ["deposits", "young", "amplify"] {
            v.as_object_mut().unwrap().remove(stage);
        }
        let dir = temp_dir(&format!("dense-{seed}"));
        let s = session(&v.to_string(), &dir, 2);
        finish(&s);
        let world = s.checkpoint().unwrap().world;
        let note: serde_json::Value =
            serde_json::from_str(world.notes.get(STAGE_NAME).expect("the stage's note")).unwrap();
        let check = &note["deformation"]["noddy_check"];
        assert!(
            check["points"].as_u64().unwrap() > 0,
            "{start} {seed}: {check}"
        );
        assert!(
            check["worst"].as_f64().unwrap() < 1e-4,
            "{start} {seed}: {check}"
        );
        let st = world.structure.as_ref().expect("the world's structure");
        assert!(
            st.events.windows(2).all(|w| w[0].key <= w[1].key),
            "youngest first"
        );
        longest = longest.max(check["max_history"].as_u64().unwrap());
        long_columns += check["long_histories"].as_u64().unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }
    assert!(
        longest > 127 && long_columns > 0,
        "the longest history met was {longest}: not past one Noddy history"
    );
}

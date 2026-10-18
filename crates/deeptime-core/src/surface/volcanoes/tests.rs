// SPDX-License-Identifier: LGPL-3.0-or-later
//! The P1-21 checks: arc volcanoes line up along the volcanic front at the arc spacing; on the
//! coupled pipeline, stratovolcanoes stand on arc crust, hotspot volcanoes form chains that age
//! away from their plume, every large igneous province becomes a flood-basalt province; the
//! events carry their causes; the strata log holds the volcanic rock; the catalogue is the same on
//! any thread count and after a resume.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::*;
use crate::checkpoint::Checkpoint;
use crate::params::planet_builtin;
use crate::preset::with_timeline;
use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};

#[test]
fn arcs_line_up_along_the_front_at_the_spacing() {
    let grid = GridDesc::for_planet("L1", 16_384, 16).unwrap();
    let (w, n) = (grid.w as usize, grid.cells());
    let mh = 300.0;
    // A 500-cell (2,400 km) arc three cells wide along row 200: full flux on the front, less on
    // its flanks; a weak arc elsewhere below the threshold.
    let mut flux = vec![0.0f64; n];
    for i in 100..600 {
        flux[199 * w + i] = 0.8;
        flux[200 * w + i] = 1.0;
        flux[201 * w + i] = 0.8;
        flux[300 * w + i] = 0.2;
    }
    let spacing = 60_000.0;
    let mut occupied = Vec::new();
    let mut rng = Rng::stream(5, "test");
    let born = place_arcs(&grid, mh, &flux, 0.25, spacing, &mut occupied, &mut rng);
    let km = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).hypot(a.1 - b.1) * mh / 1000.0;
    assert!(!born.is_empty());
    for &(x, z, c) in &born {
        assert!(flux[c] >= 0.25, "born on a cell of flux {}", flux[c]);
        let zr = grid.block_z(200) as f64;
        assert!((z - zr).abs() <= 1.5 * 16.0, "z {z} off the front");
        assert!((grid.block_x(100) as f64 - 8.0..=grid.block_x(599) as f64 + 8.0).contains(&x));
    }
    // Spaced: no pair nearer than 0.8 × the spacing; neighbours about the spacing apart.
    let pts: Vec<(f64, f64)> = born.iter().map(|&(x, z, _)| (x, z)).collect();
    let mut nn = Vec::new();
    for (a, &p) in pts.iter().enumerate() {
        let mut best = f64::INFINITY;
        for (b, &q) in pts.iter().enumerate() {
            if a != b {
                best = best.min(km(p, q));
            }
        }
        assert!(best >= 0.8 * 60.0 - 1e-6, "{best} km");
        nn.push(best);
    }
    let mean = nn.iter().sum::<f64>() / nn.len() as f64;
    assert!(
        (48.0..=80.0).contains(&mean),
        "mean neighbour distance {mean} km"
    );
    let count = born.len();
    assert!(
        (28..=52).contains(&count),
        "{count} volcanoes on 2,400 km of arc"
    );
    // Saturated: a second pass with the same occupancy adds almost nothing.
    let again = place_arcs(&grid, mh, &flux, 0.25, spacing, &mut occupied, &mut rng);
    assert!(again.len() <= count / 5, "{} more", again.len());
    // Density halves the spacing: about twice as many.
    let mut occ2 = Vec::new();
    let dense = place_arcs(&grid, mh, &flux, 0.25, spacing / 2.0, &mut occ2, &mut rng);
    assert!(
        dense.len() as f64 > 1.6 * count as f64,
        "{} vs {count}",
        dense.len()
    );
}

#[test]
fn the_front_is_the_middle_of_the_arc_band() {
    let grid = GridDesc::for_planet("L1", 16_384, 16).unwrap();
    let (w, n) = (grid.w as usize, grid.cells());
    let mut flux = vec![0.0f64; n];
    let mut elev = vec![0.0f32; n];
    // A band five cells wide along rows 100–104.
    for j in 100..105 {
        for i in 200..400 {
            flux[j * w + i] = 1.0;
        }
    }
    // A patch 40 × 40 cells striped every other row (the plate model's lattice stripes; the grid
    // is 1024 × 328).
    for j in (200..240).step_by(2) {
        for i in 500..540 {
            flux[j * w + i] = 1.0;
        }
    }
    // A trench: strong flux but 6 km deep.
    for i in 600..700 {
        flux[50 * w + i] = 1.0;
        elev[50 * w + i] = -6000.0;
    }
    let front = volcanic_front(&grid, &flux, &elev, 0.0, 0.25);
    let rows: Vec<usize> = (100..105)
        .filter(|&j| (200..400).any(|i| front[j * w + i]))
        .collect();
    assert_eq!(rows, vec![102], "the band's centre row");
    let patch = (200..240)
        .flat_map(|j| (500..540).map(move |i| j * w + i))
        .filter(|&c| front[c])
        .count();
    assert!(
        patch < 200,
        "{patch} of 800 striped cells on the patch's skeleton"
    );
    assert!(patch > 0);
    assert!(
        (600..700).all(|i| !front[50 * w + i]),
        "no front in a trench"
    );
}

/// `volcanic_front` as it was before P1-17 (serial maps, raster sweeps over every cell).
fn volcanic_front_reference(
    grid: &GridDesc,
    flux: &[f64],
    elev: &[f32],
    sea: f64,
    min_flux: f64,
) -> Vec<bool> {
    let (w, h) = (grid.w as usize, grid.h as usize);
    let n = w * h;
    let mask: Vec<bool> = (0..n)
        .map(|c| flux[c] >= min_flux && f64::from(elev[c]) >= sea - FRONT_MAX_DEPTH_M)
        .collect();
    let at = |i: i64, j: i64| -> Option<usize> {
        (j >= 0 && j < h as i64).then(|| j as usize * w + i.rem_euclid(w as i64) as usize)
    };
    let morph = |src: &[bool], any: bool| -> Vec<bool> {
        (0..n)
            .map(|c| {
                let (i, j) = ((c % w) as i64, (c / w) as i64);
                let mut hit = !any;
                for dj in -1..=1 {
                    for di in -1..=1 {
                        if let Some(m) = at(i + di, j + dj) {
                            if any && src[m] {
                                hit = true;
                            } else if !any && !src[m] {
                                hit = false;
                            }
                        }
                    }
                }
                hit
            })
            .collect()
    };
    let closed = morph(&morph(&mask, true), false);
    let inf = f32::INFINITY;
    let mut dist: Vec<f32> = closed.iter().map(|&m| if m { inf } else { 0.0 }).collect();
    let (a, b) = (1.0f32, std::f32::consts::SQRT_2);
    for _ in 0..2 {
        for j in 0..h as i64 {
            for i in 0..w as i64 {
                let c = j as usize * w + i as usize;
                for (di, dj, s) in [(-1, 0, a), (-1, -1, b), (0, -1, a), (1, -1, b)] {
                    if let Some(m) = at(i + di, j + dj) {
                        dist[c] = dist[c].min(dist[m] + s);
                    }
                }
            }
        }
        for j in (0..h as i64).rev() {
            for i in (0..w as i64).rev() {
                let c = j as usize * w + i as usize;
                for (di, dj, s) in [(1, 0, a), (1, 1, b), (0, 1, a), (-1, 1, b)] {
                    if let Some(m) = at(i + di, j + dj) {
                        dist[c] = dist[c].min(dist[m] + s);
                    }
                }
            }
        }
    }
    (0..n)
        .map(|c| {
            if !closed[c] {
                return false;
            }
            let (i, j) = ((c % w) as i64, (c / w) as i64);
            let d = dist[c];
            let near_mask =
                (-1..=1).any(|dj| (-1..=1).any(|di| at(i + di, j + dj).is_some_and(|m| mask[m])));
            near_mask
                && d.is_finite()
                && (-1..=1).all(|dj| {
                    (-1..=1).all(|di| at(i + di, j + dj).is_none_or(|m| dist[m] <= d + 1e-3))
                })
        })
        .collect()
}

/// P1-17: the parallel front with sweeps over the mask's cells only is the old front, cell for
/// cell, on random arc bands and patches (some across the x seam, some on the first and last
/// rows, some in trenches), at 1, 3 and 8 threads.
#[test]
fn the_front_matches_its_reference() {
    let grid = GridDesc::for_planet("L1", 4096, 16).unwrap();
    let (w, h, n) = (grid.w as usize, grid.h as usize, grid.cells());
    for seed in 1..=4u64 {
        let mut rng = crate::sphere::SplitMix64::new(seed);
        let mut flux = vec![0.0f64; n];
        let mut elev = vec![0.0f32; n];
        for _ in 0..12 {
            let (i0, j0) = (rng.below(w as u64) as usize, rng.below(h as u64) as usize);
            let (len, wid) = (5 + rng.below(60) as usize, 1 + rng.below(6) as usize);
            let striped = rng.below(3) == 0;
            let deep = rng.below(5) == 0;
            for dj in 0..wid {
                let j = (j0 + dj).min(h - 1);
                if striped && dj % 2 == 1 {
                    continue;
                }
                for di in 0..len {
                    let c = j * w + (i0 + di) % w;
                    flux[c] = 0.1 + 0.9 * rng.next_f64();
                    if deep {
                        elev[c] = -6000.0;
                    }
                }
            }
        }
        let want = volcanic_front_reference(&grid, &flux, &elev, 0.0, 0.25);
        assert!(want.iter().any(|&f| f), "seed {seed}: some front");
        for threads in [1, 3, 8] {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            let got = pool.install(|| volcanic_front(&grid, &flux, &elev, 0.0, 0.25));
            assert_eq!(got, want, "seed {seed}, {threads} threads");
        }
    }
}

/// A volcano of the given kind, active, 900 m tall, for the rate checks.
fn test_volcano(kind: Kind, oceanic: bool) -> Volcano {
    Volcano {
        id: 0,
        kind,
        x: 0.0,
        z: 0.0,
        plate: 0,
        oceanic,
        silicic: !oceanic,
        hotspot: Some(0),
        chain: 0,
        born_myr: 0.0,
        dies_myr: f64::INFINITY,
        extinct_myr: None,
        radius_m: 30_000.0,
        height_m: 900.0,
        peak_m: 900.0,
        built_m: 900.0,
        sunk_m: 0.0,
        kept_m: 900.0,
        target_m: 900.0,
        flux_last: 0.0,
        missed: 0,
        flux_myr: 0.0,
        active_myr: 0.0,
        depression: None,
        caldera: None,
        cause: None,
        segment: 0,
        gone: None,
    }
}

/// Ben, 2026-10-02 ("Lower to Earth's range"): stratovolcanoes stand 0.6-2.0 km (was 1.2-3.0),
/// with the basal radius 10 km (was 14) so the proportions stay. On a synthetic arc whose flux
/// rises from the threshold to the full flux, the cones the stage would place aim at heights
/// whose median lies within Earth's (Grosse et al. 2014, table 1: 512 stratovolcanoes, median
/// 1.02 km, mean 1.13 km, sd 0.55 km, range 0.17-4.57 km, height to basal width median 0.13,
/// range 0.02-0.30).
#[test]
fn stratovolcano_heights_fall_in_earths_range() {
    let p = VolcanoParams::default();
    assert_eq!(p.arc_height_km, [0.6, 2.0]);
    assert_eq!(p.arc_radius_km, 10.0);
    let grid = GridDesc::for_planet("L1", 16_384, 16).unwrap();
    let (w, n) = (grid.w as usize, grid.cells());
    let mh = 300.0;
    // A 600-cell arc, 3 cells wide, whose flux rises along it from the threshold to 1.
    let mut flux = vec![0.0f64; n];
    for i in 0..600 {
        let f = p.arc_min_flux + (1.0 - p.arc_min_flux) * i as f64 / 599.0;
        for j in 199..202 {
            flux[j * w + 100 + i] = f;
        }
    }
    let mut occupied = Vec::new();
    let mut rng = Rng::stream(7, "heights");
    let born = place_arcs(
        &grid,
        mh,
        &flux,
        p.arc_min_flux,
        p.arc_spacing_km * 1000.0,
        &mut occupied,
        &mut rng,
    );
    assert!(born.len() >= 30, "{} cones", born.len());
    let mut heights: Vec<f64> = born
        .iter()
        .map(|&(_, _, c)| arc_height_km(&p, flux[c]))
        .collect();
    heights.sort_by(f64::total_cmp);
    let median = heights[heights.len() / 2];
    // Earth's median 1.02 km; the mean 1.13 km plus or minus one standard deviation, 0.55 km, is
    // 0.58-1.68 km. The old range (1.2-3.0) put this median at 2.1 km.
    assert!((0.58..=1.68).contains(&median), "median {median} km");
    let (lo, hi) = (heights[0], heights[heights.len() - 1]);
    assert!(
        lo >= 0.17 && hi <= 4.57,
        "{lo}-{hi} km against Earth's 0.17-4.57"
    );
    // The tallest cone's height to basal width (the radius at the top of the range, doubled):
    // inside Earth's 0.02-0.30 and within 10 % of the proportion the old defaults had (3 km over
    // 28 km).
    let ratio = p.arc_height_km[1] / (2.0 * p.arc_radius_km);
    assert!((0.02..=0.30).contains(&ratio), "{ratio}");
    assert!((ratio / (3.0 / 28.0) - 1.0).abs() < 0.10, "{ratio}");
}

/// Ben, 2026-10-02 ("About 1 per Myr"): a continental hotspot's silicic centre collapses into a
/// caldera at about one per Myr while it is active (Christiansen 2001: three caldera-forming
/// eruptions at Yellowstone in about 1.5 Myr). A collapse ends the centre, and the hotspot
/// builds a new one, so a hotspot active for N Myr makes about N calderas: here one centre at a
/// time, renewed after each collapse, through the stage's own decision (`caldera_erupts`), in
/// steps of 0.05 Myr.
#[test]
fn a_continental_hotspot_makes_about_one_caldera_per_myr() {
    let p = VolcanoParams::default();
    assert_eq!(p.hotspot_caldera_rate_per_myr, 1.0);
    let (dt, myr) = (0.05, 300.0);
    let mut rng = Rng::stream(3, "hotspot-calderas");
    let mut centre = test_volcano(Kind::Silicic, false);
    let mut calderas = 0;
    for _ in 0..(myr / dt) as usize {
        if let Some(summit) = caldera_erupts(&centre, &p, dt, &mut rng) {
            assert!(
                !summit,
                "a continental centre collapses to a rim, not a summit pit"
            );
            calderas += 1;
            centre = test_volcano(Kind::Silicic, false);
        }
    }
    // Expected 300 x (1 - e^-0.05) / 0.05 = 293, standard deviation 17: 240-360 is three sigma.
    assert!(
        (240..=360).contains(&calderas),
        "{calderas} calderas in {myr} Myr"
    );
    // Arc calderas are unchanged at 0.04 per Myr: a silicic stratovolcano, same exposure.
    let mut arc = test_volcano(Kind::Strato, false);
    arc.silicic = true;
    let mut rng = Rng::stream(3, "arc-calderas");
    let n_arc = (0..(myr / dt) as usize)
        .filter(|_| caldera_erupts(&arc, &p, dt, &mut rng).is_some())
        .count();
    // Not renewed: 300 Myr at 0.04 per Myr is 12 eruptions' worth of exposure, the old
    // continental-hotspot rate (0.12) would give about 36.
    assert!((4..=24).contains(&n_arc), "{n_arc} arc eruptions");
    // A shield under 4 km never collapses, whatever the rates.
    let shield = test_volcano(Kind::Shield, true);
    let mut rng = Rng::stream(3, "shield");
    assert!((0..2000).all(|_| caldera_erupts(&shield, &p, dt, &mut rng).is_none()));
}

/// Ben, 2026-10-02 ("Set it to 0, keep the setting"): the default island subsidence is 0, so the
/// volcano stage leaves a dead oceanic volcano's height to the plate's flexure and the ageing
/// ocean floor; the setting still sinks it when asked, up to its maximum.
#[test]
fn island_subsidence_defaults_to_nothing_and_the_setting_still_works() {
    let mut p = VolcanoParams::default();
    assert_eq!(p.island_subsidence_m_myr, 0.0);
    let grid = GridDesc::for_planet("L1", 16_384, 16).unwrap();
    let n = grid.cells();
    let elev = vec![-3000.0f32; n];
    let island = |extinct: bool| {
        let mut v = test_volcano(Kind::Shield, true);
        v.radius_m = 20_000.0;
        v.extinct_myr = extinct.then_some(10.0);
        v
    };
    let run = |p: &VolcanoParams, steps: usize| {
        let mut l1 = L1 {
            grid: &grid,
            mh: 300.0,
            cell_m: 300.0 * 16.0,
            delta: vec![0.0; n],
            rock: vec![[0.0; 2]; n],
            elev: &elev,
        };
        let mut vs = vec![island(true), island(false)];
        for _ in 0..steps {
            sink_dead_islands(&mut vs, &mut l1, p, 2.0);
        }
        (vs, l1.delta)
    };
    let (vs, delta) = run(&p, 100);
    assert!(vs.iter().all(|v| v.sunk_m == 0.0));
    assert!(
        delta.iter().all(|&d| d == 0.0),
        "the default changes the surface"
    );
    // The setting: 25 m/Myr for 100 steps of 2 Myr would be 5 km; the maximum stops it at 1.5 km,
    // and only the dead volcano sinks.
    p.island_subsidence_m_myr = 25.0;
    let (vs, delta) = run(&p, 100);
    assert_eq!(vs[0].sunk_m, p.island_subsidence_max_m);
    assert_eq!(vs[1].sunk_m, 0.0);
    let deepest = delta.iter().fold(0.0f32, |m, &d| m.min(d));
    assert!((-1500.0..=-1000.0).contains(&deepest), "{deepest}");
}

fn pipeline_json(n: u32, myr: f64, seed: u64) -> String {
    let json = with_timeline(
        planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(myr),
    )
    .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = serde_json::json!(n);
    v["resolution"]["l1_r"] = serde_json::json!(32);
    v["keyframes"]["size"] = serde_json::json!([128, 64]);
    v["keyframes"]["every_myr"] = serde_json::json!(20.0);
    v["seed"] = serde_json::json!(seed);
    // Hotspots and plume heads often enough to see chains and provinces on a short run.
    v["tectonics"]["hotspot_births_per_100myr"] = serde_json::json!(12.0);
    v["tectonics"]["lip_probability"] = serde_json::json!(1.0);
    v["volcanoes"] = serde_json::json!({});
    // (The impact stage has its own tests; these are the volcanoes'.)
    for k in ["deposits", "young", "amplify", "impacts"] {
        v.as_object_mut().unwrap().remove(k);
    }
    v.to_string()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-volcanoes-{name}-{}-{:?}",
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
fn the_pipeline_puts_volcanoes_where_the_plates_say() {
    let json = pipeline_json(20_000, 120.0, 11);
    let dir = temp_dir("placement");
    let s = session(&json, &dir, 2);
    assert_eq!(
        s.stage_names(),
        ["plates", "climate", "surface", "volcanoes", "structure"]
    );
    let p = VolcanoParams::default();
    let grid = GridDesc::for_planet("L1", 16_384, 32).unwrap();
    // Step by step: every stratovolcano is born on an arc cell (or next to one: its point is
    // drawn inside its cell) whose flux, in the fields of that very step, is at least the
    // threshold.
    let mut known = 0u32;
    let mut births = 0;
    loop {
        let status = s.advance(Duration::ZERO).unwrap();
        let w_now = s.checkpoint().unwrap().world;
        if let Some(st) = &w_now.volcanoes {
            let f = |name: &str| &w_now.fields[name].data;
            let (flags, conv, crust) = (
                f("plates/flags"),
                f("plates/convergence_mm_yr"),
                f("plates/crust_type"),
            );
            let w = grid.w as usize;
            for v in st
                .volcanoes
                .iter()
                .filter(|v| v.id >= known && v.kind == Kind::Strato)
            {
                let c = cell_of(&grid, v.x, v.z);
                let (i, j) = ((c % w) as i64, (c / w) as i64);
                let mut best = 0.0f64;
                for dj in -1..=1 {
                    for di in -1..=1 {
                        let jj = (j + dj).clamp(0, i64::from(grid.h) - 1) as usize;
                        let cc = jj * w + grid.wrap_i(i + di) as usize;
                        best = best.max(arc_flux(flags[cc], conv[cc], crust[cc]));
                    }
                }
                assert!(
                    best >= p.arc_min_flux,
                    "volcano {} born on flux {best}",
                    v.id
                );
                births += 1;
            }
            known = st.next_id;
        }
        if status == AdvanceStatus::Done {
            break;
        }
    }
    assert!(births > 0);
    let world = s.checkpoint().unwrap().world;
    let st = world.volcanoes.as_ref().expect("the catalogue");
    let n = grid.cells();
    let frame = BlockFrame::new(16_384);
    assert!(st.counts.strato_born > 0, "{:?}", st.counts);
    assert!(
        st.counts.shield_born + st.counts.silicic_born > 0,
        "{:?}",
        st.counts
    );
    // Plate task 8: the cones' rock reached the plate model's pools, felsic and mafic.
    let global = |k: &str| world.globals.get(k).copied().unwrap_or(0.0);
    let (felsic, mafic) = (
        global(crate::plates::CONE_FELSIC_GLOBAL),
        global(crate::plates::CONE_MAFIC_GLOBAL),
    );
    assert!(felsic > 0.0 && mafic > 0.0, "cone rock {felsic} and {mafic} km³");

    // The default island subsidence is 0 (Ben, 2026-10-02): no volcano of the run has sunk.
    assert!(
        st.volcanoes.iter().chain(&st.gone).all(|v| v.sunk_m == 0.0),
        "the volcano stage sank an island under the defaults"
    );

    // Active stratovolcanoes last saw enough arc flux and have grown.
    let mut active = 0;
    for v in st
        .volcanoes
        .iter()
        .filter(|v| v.kind == Kind::Strato && v.active())
    {
        active += 1;
        assert!(
            v.flux_last >= p.arc_min_flux,
            "volcano {}: {}",
            v.id,
            v.flux_last
        );
        assert!(v.radius_m > 0.0 && v.missed < 2);
    }
    let stratos = st
        .volcanoes
        .iter()
        .filter(|v| v.kind == Kind::Strato)
        .count();
    assert!(
        active > 0,
        "no active arc volcano: {stratos} in the catalogue, {:?}",
        st.counts
    );
    // Active ones keep the spacing (they were placed apart and ride their plates together).
    let live: Vec<&Volcano> = st
        .volcanoes
        .iter()
        .filter(|v| v.kind == Kind::Strato && v.active())
        .collect();
    let mut close = 0;
    for (a, va) in live.iter().enumerate() {
        for vb in &live[a + 1..] {
            let d = (frame.wrap_x(va.x - vb.x) * 300.0).hypot((va.z - vb.z) * 300.0);
            if d < 0.5 * p.arc_spacing_km * 1000.0 {
                close += 1;
            }
        }
    }
    assert!(
        close * 20 <= live.len(),
        "{close} close pairs among {} live cones",
        live.len()
    );

    // Hotspot chains: in chain order, born later; every one but the newest two is dead; the live
    // ones lie within the spacing of their plume.
    let plates = world.plates.as_ref().unwrap();
    let mut chains: BTreeMap<u32, Vec<&Volcano>> = BTreeMap::new();
    for v in st.volcanoes.iter().chain(&st.gone) {
        if let Some(h) = v.hotspot {
            chains.entry(h).or_default().push(v);
        }
    }
    let mut long = 0;
    let (mut pairs, mut aged) = (0usize, 0usize);
    for (h, mut vs) in chains {
        vs.sort_by_key(|v| v.chain);
        for pair in vs.windows(2) {
            assert!(pair[0].born_myr < pair[1].born_myr, "hotspot {h}");
        }
        for v in &vs[..vs.len().saturating_sub(2)] {
            assert!(
                !v.active(),
                "hotspot {h}: volcano {} of {} still active",
                v.chain,
                vs.len()
            );
        }
        if vs.len() >= 3 {
            long += 1;
        }
        let hs = plates.hotspots.iter().find(|x| x.id == h).unwrap();
        let (px, pz) = frame.block(DVec3::from_array(hs.pos));
        for v in vs.iter().filter(|v| v.active() && v.gone.is_none()) {
            let d = (frame.wrap_x(v.x - px) * 300.0).hypot((v.z - pz) * 300.0);
            assert!(
                d <= p.hotspot_spacing_km * 1000.0 + 1.0,
                "{d} m from its plume"
            );
        }
        // Ageing along the track: of two consecutive volcanoes riding the same plate, the older
        // is mostly the farther from the plume (plate motions turn, so not always).
        for pair in vs.windows(2) {
            let (older, newer) = (pair[0], pair[1]);
            if older.gone.is_some() || newer.gone.is_some() || older.plate != newer.plate {
                continue;
            }
            let d = |v: &Volcano| (frame.wrap_x(v.x - px) * 300.0).hypot((v.z - pz) * 300.0);
            pairs += 1;
            if d(older) + 1.0 >= d(newer) {
                aged += 1;
            }
        }
    }
    assert!(
        pairs == 0 || aged * 10 >= pairs * 7,
        "{aged} of {pairs} consecutive pairs have the older farther from the plume"
    );
    assert!(long > 0, "no hotspot made a chain of three");

    // Every large igneous province became a flood-basalt province caused by it.
    let lips: Vec<EventId> = world
        .events
        .iter()
        .filter(|e| e.kind == LARGE_IGNEOUS_PROVINCE)
        .map(|e| e.id)
        .collect();
    assert!(!lips.is_empty());
    assert_eq!(st.basalts.len(), lips.len());
    let fb: Vec<_> = world
        .events
        .iter()
        .filter(|e| e.kind == kind::FLOOD_BASALT)
        .collect();
    assert_eq!(fb.len(), lips.len());
    for e in &fb {
        assert!(lips.contains(&e.causes[0]));
    }
    // Volcano events (grouped): arc segments caused by a subduction onset, hotspot chains by their
    // track (or plume head); the notable cones by their group; one arc event per segment.
    let kind_of = |id: EventId| {
        world
            .events
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.kind.clone())
    };
    let (mut arcs, mut chains_n, mut strato_events) = (0, 0, 0);
    for e in world.events.iter().filter(|e| e.source == SOURCE) {
        e.validate().unwrap();
        let cause = e.causes.first().and_then(|&c| kind_of(c));
        match e.kind.as_str() {
            kind::ARC => {
                arcs += 1;
                if let Some(c) = &cause {
                    assert_eq!(c, "deeptime:subduction_onset");
                }
                assert!(matches!(e.params.get("cones"), Some(ParamValue::Str(_))));
            }
            kind::HOTSPOT_CHAIN => {
                chains_n += 1;
                assert!(
                    matches!(
                        cause.as_deref(),
                        Some("deeptime:hotspot_track" | "deeptime:large_igneous_province")
                    ),
                    "{cause:?}"
                );
            }
            kind::STRATOVOLCANO => {
                strato_events += 1;
                assert!(
                    matches!(
                        cause.as_deref(),
                        Some(kind::ARC | "deeptime:subduction_onset")
                    ),
                    "{cause:?}"
                );
            }
            kind::SHIELD | kind::CALDERA_COMPLEX => {
                assert_eq!(cause.as_deref(), Some(kind::HOTSPOT_CHAIN));
            }
            kind::CALDERA | kind::FLOOD_BASALT => assert!(!e.causes.is_empty()),
            other => panic!("unexpected kind {other}"),
        }
    }
    assert!(arcs > 0 && chains_n > 0 && strato_events > 0);
    // One event per arc segment, but small segments with an onset are counts on it.
    let small = st
        .arcs
        .values()
        .filter(|g| {
            g.born < SMALL_SEGMENT
                && g.cause
                    .is_some_and(|c| world.events.iter().any(|e| e.id == c))
        })
        .count();
    assert_eq!(
        arcs + small,
        st.arcs.len(),
        "one event per segment or a count on its onset"
    );
    assert_eq!(st.counts.small_arc_segments as usize, small);
    assert!(
        strato_events <= st.arcs.len(),
        "at most one notable cone per segment"
    );
    if small > 0 {
        let counted = world
            .events
            .iter()
            .filter(|e| matches!(e.params.get("volcano_cones_built"), Some(ParamValue::Int(n)) if *n > 0))
            .count();
        assert!(counted > 0, "small segments counted on their onsets");
    }

    // The strata log holds the volcanic rock.
    let log = &world.surface.as_ref().unwrap().log;
    let (andesite, basalt, tuff) = (index("andesite"), index("basalt"), index("tuff"));
    let mut seen = [0usize; 3];
    for c in 0..n {
        for l in log.column(c) {
            if l.class == andesite {
                seen[0] += 1;
            } else if l.class == basalt {
                seen[1] += 1;
            } else if l.class == tuff {
                seen[2] += 1;
            }
        }
    }
    assert!(seen.iter().all(|&k| k > 0), "{seen:?}");
    for c in 0..n {
        log.check(c, crate::synth::EVERY_CLASS.len()).unwrap();
    }
    // The map layer marks them; the notes carry the summary.
    let map = &world.fields[field::MAP].data;
    assert!(map.contains(&code::STRATO_ACTIVE));
    let note: serde_json::Value = serde_json::from_str(&world.notes[STAGE_NAME]).unwrap();
    assert!(summary_line(&note).starts_with("volcanoes_final="));
    // P1-27: the ground under continental arc cones, every 10 Myr of the 120.
    let bases = note["arc_bases"].as_array().unwrap();
    assert_eq!(bases.len(), 12, "one record every 10 Myr");
    for b in bases {
        let cones = b["cones"].as_u64().unwrap();
        let low = b["low_share"].as_f64().unwrap();
        assert!((0.0..=1.0).contains(&low));
        if cones > 0 {
            let (p25, p50, p75) = (
                b["base_p25_m"].as_f64().unwrap(),
                b["base_p50_m"].as_f64().unwrap(),
                b["base_p75_m"].as_f64().unwrap(),
            );
            assert!(p25 <= p50 && p50 <= p75, "{b}");
        }
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn threads_and_a_resume_leave_the_volcanoes_unchanged() {
    let json = pipeline_json(12_000, 40.0, 4);
    let dir = temp_dir("one");
    let s = session(&json, &dir, 1);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let dir2 = temp_dir("resumed");
    let s2 = session(&json, &dir2, 3);
    for _ in 0..9 {
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
    // Compared as text: a record with no continental arc cones (plates start at rest since the
    // force balance, so the first 10 Myr can have none) keeps NaN percentiles, and NaN != NaN.
    assert_eq!(
        format!("{:?}", got.volcanoes),
        format!("{:?}", world.volcanoes)
    );
    assert!(!world.volcanoes.as_ref().unwrap().volcanoes.is_empty());
    let ours = |w: &WorldState| -> Vec<deeptime_format::Event> {
        w.events
            .iter()
            .filter(|e| e.source == SOURCE)
            .cloned()
            .collect()
    };
    assert_eq!(ours(&got), ours(&world));
    assert_eq!(
        got.surface.as_ref().unwrap().elevation_m,
        world.surface.as_ref().unwrap().elevation_m
    );
    let bits = |w: &WorldState| -> Vec<u32> {
        w.fields[field::MAP]
            .data
            .iter()
            .map(|x| x.to_bits())
            .collect()
    };
    assert_eq!(bits(&got), bits(&world));
    for d in [dir, dir2] {
        let _ = std::fs::remove_dir_all(d);
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The P1-22 invariants on a synthetic planet (C = 1,024, 2 blocks a cell): land sloping south to
//! a sea, two families of diagonal valleys that cross (confluences; some cross the seam), and a bowl
//! that must become a lake.

use std::collections::HashSet;

use deeptime_format::GridDesc;
use deeptime_format::rivers::Mouth;

use super::network::{self, NODATA, Output, kind};
use super::{Climate, HydroParams};

const C: u32 = 1024;
const SEA: f64 = 63.0;

/// Terrain height in blocks at cell (i, j) of the 512 × 256 grid.
fn height(i: usize, j: usize) -> f64 {
    let (fi, fj) = (i as f64, j as f64);
    if j >= 200 {
        return SEA - 2.0 - (fj - 200.0) * 0.5;
    }
    let slope = SEA + 1.0 + 0.25 * (200.0 - fj);
    // Two families of diagonal valleys, both draining south: where they cross, rivers join.
    let pi = std::f64::consts::PI;
    let a = (pi * (fi - 0.5 * fj) / 64.0).sin().abs();
    let b = (pi * (fi + 0.5 * fj) / 64.0).sin().abs();
    let valley = 8.0 * a.min(b);
    let d2 = (fi - 300.0).powi(2) + (fj - 100.0).powi(2);
    let bowl = if d2 < 144.0 {
        12.0 * (1.0 - d2 / 144.0)
    } else {
        0.0
    };
    slope + valley - bowl
}

fn grid() -> GridDesc {
    GridDesc::for_planet("L3", C, 2).unwrap()
}

fn elevation() -> Vec<i16> {
    let g = grid();
    let (w, h) = (g.w as usize, g.h as usize);
    (0..w * h)
        .map(|c| (height(c % w, c / w) * 16.0).round() as i16)
        .collect()
}

fn run_with(params: &HydroParams) -> Output {
    let g = grid();
    let elev = elevation();
    let climate = |_x: i64, _z: i64| Climate::steady(1500.0, 5.0);
    network::run(&network::Input {
        grid: &g,
        elev: &elev,
        sea_raw: (SEA * 16.0) as i32,
        m_per_block_h: 300.0,
        climate: &climate,
        params,
        width_scale: 1.0,
    })
}

fn run() -> Output {
    run_with(&HydroParams::default())
}

#[test]
fn every_river_descends_to_the_sea_or_a_lake() {
    let out = run();
    let r = &out.rivers;
    assert!(r.chains.len() >= 5, "{} chains", r.chains.len());
    for (k, ch) in r.chains.iter().enumerate() {
        let vs = &r.vertices[ch.first as usize..(ch.first + ch.count) as usize];
        for p in vs.windows(2) {
            assert!(
                p[1].w <= p[0].w,
                "chain {k}: the surface rises {} → {}",
                p[0].w,
                p[1].w
            );
            let step = (p[1].x - p[0].x).hypot(p[1].z - p[0].z);
            assert!(
                step < 6.0,
                "chain {k}: a {step}-block jump (the seam must not show)"
            );
        }
        let last = vs.last().unwrap();
        match ch.mouth {
            Mouth::Sea => assert_eq!(last.w, SEA as f32, "chain {k} reaches the sea at sea level"),
            Mouth::Lake => {
                let lake = &out.lakes[ch.into as usize];
                assert_eq!(
                    last.w,
                    f32::from(lake.level) / 16.0,
                    "chain {k} meets its lake's level"
                );
            }
            Mouth::River => {
                let into = &r.chains[ch.into as usize];
                let joins = r.vertices[into.first as usize..(into.first + into.count) as usize]
                    .iter()
                    .any(|v| {
                        let dx = (v.x - last.x).rem_euclid(C as f32);
                        dx.min(C as f32 - dx) < 1e-3 && (v.z - last.z).abs() < 1e-3 && v.w == last.w
                    });
                assert!(joins, "chain {k} ends on a vertex of chain {}", ch.into);
            }
        }
        // Following the mouths always ends in the sea or a lake.
        let mut at = k;
        let mut seen = HashSet::new();
        while r.chains[at].mouth == Mouth::River {
            assert!(seen.insert(at), "chain {k}: a cycle of confluences");
            at = r.chains[at].into as usize;
        }
    }
    assert!(r.chains.iter().any(|c| c.mouth == Mouth::Sea));
    assert!(r.chains.iter().any(|c| c.mouth == Mouth::River));
}

#[test]
fn every_lake_is_full_to_its_lowest_rim() {
    let out = run();
    assert!(!out.lakes.is_empty(), "the bowl is a lake");
    let g = grid();
    let (w, h) = (g.w as usize, g.h as usize);
    let elev = elevation();
    let wl = &out.water_level;
    // Lakes by level; each lake's rim is every cell next to it that holds no water.
    for (id, lake) in out.lakes.iter().enumerate() {
        let mut rim = i16::MAX;
        let mut cells = 0;
        for c in 0..w * h {
            if wl[c] != lake.level {
                continue;
            }
            cells += 1;
            assert!(
                elev[c] < lake.level,
                "lake {id}: a dry cell holds its level"
            );
            let (i, j) = (c % w, c / w);
            for (di, dj) in [
                (1i64, 0i64),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (-1, 1),
                (1, -1),
                (-1, -1),
            ] {
                let nj = j as i64 + dj;
                if nj < 0 || nj >= h as i64 {
                    continue;
                }
                let ni = (i as i64 + di).rem_euclid(w as i64) as usize;
                let n = nj as usize * w + ni;
                if wl[n] == NODATA {
                    rim = rim.min(elev[n]);
                }
            }
        }
        assert_eq!(cells as u32, lake.cells);
        assert!(
            (i32::from(rim) - i32::from(lake.level)).abs() <= 16,
            "lake {id}: level {} vs lowest rim {rim} (±1 block)",
            lake.level
        );
        assert_eq!(rim, lake.level, "lake {id}: exactly its spill height");
    }
}

#[test]
fn the_result_does_not_depend_on_threads_or_runs() {
    let p = HydroParams::default();
    let a = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap()
        .install(|| run_with(&p));
    let b = rayon::ThreadPoolBuilder::new()
        .num_threads(3)
        .build()
        .unwrap()
        .install(|| run_with(&p));
    assert_eq!(a.rivers, b.rivers);
    assert_eq!(a.water_level, b.water_level);
    assert_eq!(a.lakes, b.lakes);
    let enc = |o: &Output| deeptime_format::rivers::encode_rivers(&o.rivers).unwrap();
    assert_eq!(enc(&a), enc(&run_with(&p)));
}

#[test]
fn a_river_crosses_the_seam() {
    let out = run();
    let r = &out.rivers;
    let half = f64::from(C) / 2.0;
    let crosses = r.chains.iter().any(|ch| {
        let vs = &r.vertices[ch.first as usize..(ch.first + ch.count) as usize];
        let inside = vs.iter().any(|v| f64::from(v.x).abs() < half);
        let outside = vs.iter().any(|v| f64::from(v.x).abs() > half);
        inside && outside
    });
    assert!(
        crosses,
        "a valley crosses x = ±C/2: its river continues across"
    );
    // Every vertex's column finds its own channel, on any lap.
    for (s, v) in r.vertices.iter().enumerate().step_by(97) {
        let (x, z) = (f64::from(v.x).round() as i64, f64::from(v.z).round() as i64);
        let a = r.sample(x, z);
        assert_eq!(a, r.sample(x + i64::from(C), z), "vertex {s}");
        assert!(
            a.hit.is_some() || a.bank_cap.is_some(),
            "vertex {s} is on or beside a channel"
        );
    }
}

#[test]
fn width_and_depth_follow_the_discharge() {
    let out = run();
    let r = &out.rivers;
    let p = HydroParams::default();
    for v in &r.vertices {
        let q = f64::from(v.q);
        assert!((f64::from(v.hw) * 2.0 - p.width(q, 1.0)).abs() < 1e-3);
        assert!((f64::from(v.depth) - p.depth(q)).abs() < 1e-3);
        assert!(v.hw >= 0.5 && v.depth >= 1.0);
    }
    // Twice the rivers feature scale, twice the width (below the cap).
    let wide = {
        let g = grid();
        let elev = elevation();
        let climate = |_x: i64, _z: i64| Climate::steady(1500.0, 5.0);
        network::run(&network::Input {
            grid: &g,
            elev: &elev,
            sea_raw: (SEA * 16.0) as i32,
            m_per_block_h: 300.0,
            climate: &climate,
            params: &p,
            width_scale: 2.0,
        })
    };
    let big = |o: &Output| {
        o.rivers
            .vertices
            .iter()
            .map(|v| v.hw)
            .fold(0.0f32, f32::max)
    };
    assert!(big(&wide) > 1.9 * big(&out));
}

/// The synthetic planet under `climate`.
fn run_in(climate: &(dyn Fn(i64, i64) -> Climate + Sync), params: &HydroParams) -> Output {
    let g = grid();
    let elev = elevation();
    network::run(&network::Input {
        grid: &g,
        elev: &elev,
        sea_raw: (SEA * 16.0) as i32,
        m_per_block_h: 300.0,
        climate,
        params,
        width_scale: 1.0,
    })
}

fn lake_cells(o: &Output) -> usize {
    o.water_level.iter().filter(|&&l| l != NODATA).count()
}

#[test]
fn a_dry_climate_holds_a_smaller_lake_below_its_rim_or_none() {
    let p = HydroParams::default();
    let wet = run();
    assert!(wet.lakes.iter().all(|l| l.level == l.rim && !l.endorheic));
    // Little rain, strong evaporation (30 °C: 1,800 mm a year off open water).
    let dry = run_in(&|_, _| Climate::steady(200.0, 30.0), &p);
    assert!(
        dry.lakes
            .iter()
            .all(|l| l.level < l.rim && l.endorheic && l.outlet == u32::MAX),
        "no basin fills"
    );
    assert!(lake_cells(&dry) < lake_cells(&wet));
    assert!(dry.rivers.chains.len() < wet.rivers.chains.len());
    assert_eq!(dry.stats["basins"]["full"], 0);
    // The old rule: every lake full, flagged endorheic when evaporation takes its inflow.
    let old = HydroParams {
        lake_balance: false,
        ..p
    };
    let full = run_in(&|_, _| Climate::steady(200.0, 30.0), &old);
    assert_eq!(lake_cells(&full), lake_cells(&wet));
    assert!(full.lakes.iter().all(|l| l.level == l.rim && l.endorheic));
}

#[test]
fn a_lake_grows_with_the_rain_until_it_spills() {
    let p = HydroParams::default();
    let mut last = 0;
    let mut seen = Vec::new();
    for rain in [0.0, 60.0, 150.0, 300.0, 600.0, 1200.0, 2500.0] {
        let out = run_in(&|_, _| Climate::steady(rain, 25.0), &p);
        let cells = lake_cells(&out);
        assert!(cells >= last, "{rain} mm: {cells} cells after {last}");
        last = cells;
        seen.push((rain, cells, out.lakes.iter().any(|l| l.level == l.rim)));
        for l in &out.lakes {
            // At the balance the lake evaporates what reaches it, to within the step of the
            // hypsometry it stopped on; a full one spills the rest.
            if l.level < l.rim {
                assert!(l.evaporation <= l.inflow * 1.0001 + 1e-9, "{l:?}");
            } else {
                assert!(l.inflow >= l.evaporation, "{l:?}");
            }
        }
    }
    assert_eq!(seen[0].1, 0, "no rain, no lake: {seen:?}");
    assert!(seen.last().unwrap().2, "enough rain fills it: {seen:?}");
    assert!(
        seen.iter().any(|&(_, cells, full)| cells > 0 && !full),
        "in between it stands below its rim: {seen:?}"
    );
}

#[test]
fn a_cold_lake_is_frozen_and_ground_that_never_thaws_feeds_none() {
    let p = HydroParams::default();
    // Mean −5 °C with a +10 °C summer: the snow melts, the lake fills, and it is under ice for
    // more than half the year.
    let cold = run_in(&|_, _| Climate::of(600.0, -5.0, 10.0, -20.0), &p);
    assert!(!cold.lakes.is_empty());
    assert!(cold.lakes.iter().all(|l| l.frozen));
    assert!(cold.kind.contains(&kind::FROZEN));
    assert!(!cold.kind.contains(&kind::LAKE));
    assert!(!cold.rivers.chains.is_empty(), "meltwater runs off");
    // The warmest month below freezing: nothing melts, so no river and no lake.
    let never = run_in(&|_, _| Climate::of(600.0, -20.0, -4.0, -36.0), &p);
    assert!(never.lakes.is_empty() && never.rivers.chains.is_empty());
    assert!(never.water_level.iter().all(|&l| l == NODATA));
    // A warm lake is open water.
    let warm = run();
    assert!(warm.lakes.iter().all(|l| !l.frozen));
    assert!(!warm.kind.contains(&kind::FROZEN));
    // The freezing point is a setting.
    let hardy = HydroParams {
        lake_freeze_mean_c: -10.0,
        ..p
    };
    let open = run_in(&|_, _| Climate::of(600.0, -5.0, 10.0, -20.0), &hardy);
    assert!(open.lakes.iter().all(|l| !l.frozen));
}

#[test]
fn the_thaw_and_the_budget_follow_the_seasons() {
    let p = HydroParams::default();
    // No frost: the whole year counts, and the budget is the surface stage's.
    assert_eq!(Climate::steady(800.0, 12.0).thaw(), (12.0, 1.0));
    let b = p.budget(&Climate::steady(800.0, 12.0));
    assert_eq!(b.rain_mm, 800.0);
    assert_eq!(b.evaporation_mm, p.lake_evaporation_base_mm + 720.0);
    let et = crate::surface::climate::fu_et(800.0, 720.0);
    assert!((b.runoff_mm - (800.0 - et)).abs() < 1e-9);
    // Never above freezing: nothing.
    assert_eq!(Climate::of(300.0, -20.0, -2.0, -38.0).thaw(), (0.0, 0.0));
    let b = p.budget(&Climate::of(300.0, -20.0, -2.0, -38.0));
    // (Its ice still sublimates: the base loss.)
    assert_eq!(
        (b.runoff_mm, b.rain_mm, b.evaporation_mm),
        (0.0, 0.0, p.lake_evaporation_base_mm)
    );
    // A mean of 0 °C with ±10 °C: half the year above, 10/π °C on average.
    let (above, share) = Climate::of(500.0, 0.0, 10.0, -10.0).thaw();
    assert!((share - 0.5).abs() < 1e-12 && (above - 10.0 / std::f64::consts::PI).abs() < 1e-12);
    // A short cool summer melts only part of a heavy snowfall.
    let b = p.budget(&Climate::of(2000.0, -12.0, 1.0, -25.0));
    assert!(b.rain_mm > 0.0 && b.rain_mm < 500.0, "{b:?}");
    // A cold land with a warm summer evaporates in that summer (Holdridge's biotemperature): less
    // runs off than falls.
    let boreal = Climate::of(500.0, -2.0, 16.0, -20.0);
    let b = p.budget(&boreal);
    assert_eq!(b.rain_mm, 500.0);
    let et = crate::surface::climate::fu_et(500.0, 60.0 * boreal.thaw().0);
    assert!((b.runoff_mm - (500.0 - et)).abs() < 1e-9);
    assert!(b.runoff_mm > 150.0 && b.runoff_mm < 400.0, "{b:?}");
    // Evaporation off: a lake loses nothing.
    let off = HydroParams {
        lake_evaporation: false,
        ..p
    };
    assert_eq!(
        off.budget(&Climate::steady(800.0, 12.0)).evaporation_mm,
        0.0
    );
    // The aridity index is P / PET.
    assert!((Climate::steady(360.0, 30.0).aridity() - 0.2).abs() < 1e-12);
    assert_eq!(Climate::of(100.0, -20.0, -2.0, -38.0).aridity(), 10.0);
}

#[test]
fn a_basin_below_sea_level_stands_at_sea_level_however_dry() {
    // The bowl dug to 8 blocks below sea level (the game floods such ground with the sea's water
    // whatever the bake says), its rim well above.
    let g = grid();
    let (w, h) = (g.w as usize, g.h as usize);
    let elev: Vec<i16> = (0..w * h)
        .map(|c| {
            let (i, j) = ((c % w) as f64, (c / w) as f64);
            let d2 = (i - 300.0).powi(2) + (j - 100.0).powi(2);
            let y = if d2 < 144.0 {
                SEA - 8.0 + 20.0 * d2 / 144.0
            } else {
                height(c % w, c / w)
            };
            (y * 16.0).round() as i16
        })
        .collect();
    let sea_raw = (SEA * 16.0) as i32;
    let run = |climate: Climate| {
        network::run(&network::Input {
            grid: &g,
            elev: &elev,
            sea_raw,
            m_per_block_h: 300.0,
            climate: &move |_, _| climate,
            params: &HydroParams::default(),
            width_scale: 1.0,
        })
    };
    // No rain at all: the bowl is still a lake, at sea level, below its rim.
    let dry = run(Climate::steady(0.0, 30.0));
    check_holds(&dry, &elev, &g);
    assert_eq!(dry.lakes.len(), 1);
    let lake = &dry.lakes[0];
    assert_eq!(i32::from(lake.level), sea_raw);
    assert!(lake.level < lake.rim && lake.endorheic && lake.outlet == u32::MAX);
    assert_eq!(lake.depth, 8 * 16);
    assert!(dry.rivers.chains.is_empty());
    // A little rain: still sea level (the balance alone would stand lower).
    let arid = run(Climate::steady(150.0, 30.0));
    assert!(arid.lakes.iter().any(|l| i32::from(l.level) == sea_raw));
    // Wet: it fills to its rim, above sea level.
    let wet = run(Climate::steady(1500.0, 5.0));
    let full = wet.lakes.iter().max_by_key(|l| l.cells).unwrap();
    assert!(full.level == full.rim && i32::from(full.level) > sea_raw);
    assert!(full.cells > lake.cells);
}

/// A rough planet: hash noise in four octaves on a 512 × 256 grid of 2-block cells, sea beyond
/// the last rows, with basins of every size, some inside others.
fn rough() -> (GridDesc, Vec<i16>) {
    let g = grid();
    let (w, h) = (g.w as usize, g.h as usize);
    let hash = |i: i64, j: i64, o: u32| -> f64 {
        let mut x = (i as u64)
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add((j as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
            .wrapping_add(u64::from(o) << 32);
        x ^= x >> 31;
        x = x.wrapping_mul(0xD6E8_FEB8_6659_FD93);
        x ^= x >> 29;
        (x % 10_000) as f64 / 10_000.0
    };
    let noise = |fi: f64, fj: f64, step: f64, o: u32| -> f64 {
        let (u, v) = (fi / step, fj / step);
        let (i0, j0) = (u.floor(), v.floor());
        let (fu, fv) = (u - i0, v - j0);
        let wrap = (w as f64 / step) as i64;
        let at = |di: i64, dj: i64| hash((i0 as i64 + di).rem_euclid(wrap), j0 as i64 + dj, o);
        let a = at(0, 0) + (at(1, 0) - at(0, 0)) * fu;
        let b = at(0, 1) + (at(1, 1) - at(0, 1)) * fu;
        a + (b - a) * fv
    };
    let elev = (0..w * h)
        .map(|c| {
            let (i, j) = ((c % w) as f64, (c / w) as f64);
            if j >= 230.0 {
                return ((SEA - 6.0) * 16.0) as i16;
            }
            let y = SEA
                + 2.0
                + 0.05 * (230.0 - j)
                + 40.0 * noise(i, j, 64.0, 1)
                + 16.0 * noise(i, j, 16.0, 2)
                + 6.0 * noise(i, j, 4.0, 3)
                + 2.0 * noise(i, j, 1.0, 4);
            (y * 16.0).round() as i16
        })
        .collect();
    (g, elev)
}

fn run_rough(climate: &(dyn Fn(i64, i64) -> Climate + Sync), params: &HydroParams) -> Output {
    let (g, elev) = rough();
    network::run(&network::Input {
        grid: &g,
        elev: &elev,
        sea_raw: (SEA * 16.0) as i32,
        m_per_block_h: 300.0,
        climate,
        params,
        width_scale: 1.0,
    })
}

/// Every lake holds its water (no dry cell beside it lies below its level), every wet cell lies
/// below its level, and every river descends to the sea, a lake or a larger river.
fn check_holds(out: &Output, elev: &[i16], g: &GridDesc) {
    let (w, h) = (g.w as usize, g.h as usize);
    let wl = &out.water_level;
    let by_cell: std::collections::HashMap<u32, usize> = out
        .lakes
        .iter()
        .enumerate()
        .map(|(k, l)| (l.cell, k))
        .collect();
    for c in 0..w * h {
        if wl[c] == NODATA {
            assert!(out.kind[c] == kind::NONE || out.kind[c] == kind::PLAYA);
            continue;
        }
        assert!(out.kind[c] == kind::LAKE || out.kind[c] == kind::FROZEN);
        assert!(
            elev[c] < wl[c],
            "cell {c}: ground at or above its lake's level"
        );
        if let Some(&k) = by_cell.get(&(c as u32)) {
            assert_eq!(out.lakes[k].level, wl[c]);
        }
        let (i, j) = ((c % w) as i64, (c / w) as i64);
        for (di, dj) in [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (-1, 1),
            (1, -1),
            (-1, -1),
        ] {
            let nj = j + dj;
            if nj < 0 || nj >= h as i64 {
                continue;
            }
            let n = nj as usize * w + (i + di).rem_euclid(w as i64) as usize;
            if wl[n] == NODATA {
                assert!(elev[n] >= wl[c], "the lake at cell {c} leaks into cell {n}");
            } else {
                assert_eq!(
                    wl[n], wl[c],
                    "two lakes of different levels touch at {c}, {n}"
                );
            }
        }
    }
    for l in &out.lakes {
        assert!(l.cells > 0 && l.level <= l.rim && l.depth > 0, "{l:?}");
        assert!(wl[l.cell as usize] == l.level, "{l:?}");
        if l.level < l.rim {
            assert!(l.outlet == u32::MAX && l.endorheic, "{l:?}");
        }
    }
    assert_eq!(
        out.lakes.iter().map(|l| u64::from(l.cells)).sum::<u64>(),
        wl.iter().filter(|&&l| l != NODATA).count() as u64
    );
    let r = &out.rivers;
    for (k, ch) in r.chains.iter().enumerate() {
        let vs = &r.vertices[ch.first as usize..(ch.first + ch.count) as usize];
        assert!(vs.windows(2).all(|p| p[1].w <= p[0].w), "chain {k} rises");
        match ch.mouth {
            Mouth::Lake => {
                let lake = &out.lakes[ch.into as usize];
                assert_eq!(
                    vs.last().unwrap().w,
                    f32::from(lake.level) / 16.0,
                    "chain {k} meets its lake's level"
                );
            }
            Mouth::River => assert!((ch.into as usize) < r.chains.len()),
            Mouth::Sea => {}
        }
    }
}

#[test]
fn rough_ground_holds_its_lakes_in_every_climate() {
    let (g, elev) = rough();
    let p = HydroParams::default();
    let mut lakes = Vec::new();
    for (rain, t) in [(2000.0, 12.0), (500.0, 22.0), (250.0, 28.0), (80.0, 30.0)] {
        let out = run_rough(&|_, _| Climate::steady(rain, t), &p);
        check_holds(&out, &elev, &g);
        let b = &out.stats["basins"];
        let n = |k: &str| b[k].as_u64().unwrap();
        // Every basin is a full lake, a lake below its rim, a playa or dry; a lake a terminal
        // lake rose over is no longer one.
        assert_eq!(
            n("candidates"),
            n("full") + n("below_rim") + n("playa") + n("dry") + n("nested_too_deep"),
            "{b}"
        );
        assert_eq!(
            out.lakes.len() as u64 + n("merged_into_a_terminal_lake"),
            n("full") + n("below_rim"),
            "{b}"
        );
        lakes.push((
            lake_cells(&out),
            n("full"),
            n("below_rim"),
            n("playa") + n("dry"),
        ));
    }
    // Drier climates hold less lake, fewer full basins and more that stand below their rim or dry.
    assert!(lakes.windows(2).all(|p| p[1].0 <= p[0].0), "{lakes:?}");
    assert!(lakes[0].1 > lakes[3].1 && lakes[0].2 == 0, "{lakes:?}");
    assert!(lakes[2].2 > 0 && lakes[3].3 > lakes[0].3, "{lakes:?}");
    assert!(lakes[0].0 > 0);
}

#[test]
fn an_upstream_lake_that_does_not_spill_feeds_nothing_downstream() {
    // The same rough ground, wet in the south and either wet or desert in the north: with a desert
    // upstream the rivers of the south carry less, and nothing a northern terminal lake holds
    // reaches them.
    let p = HydroParams::default();
    let c = f64::from(C);
    let north = |z: i64| (z as f64) < -c / 4.0 + 200.0;
    let wet = run_rough(&|_, _| Climate::steady(1200.0, 15.0), &p);
    let split = run_rough(
        &|_, z| {
            if north(z) {
                Climate::steady(150.0, 30.0)
            } else {
                Climate::steady(1200.0, 15.0)
            }
        },
        &p,
    );
    let (g, elev) = rough();
    check_holds(&split, &elev, &g);
    let terminal = split.lakes.iter().filter(|l| l.level < l.rim).count();
    assert!(terminal > 0, "the desert's basins do not fill");
    assert!(
        split
            .lakes
            .iter()
            .filter(|l| l.level < l.rim)
            .all(|l| l.outlet == u32::MAX)
    );
    let q_max = |o: &Output| o.rivers.vertices.iter().map(|v| v.q).fold(0.0f32, f32::max);
    assert!(
        q_max(&split) < q_max(&wet),
        "{} {}",
        q_max(&split),
        q_max(&wet)
    );
    // No chain leaves a lake that stands below its rim: a chain's first vertex is never in one.
    let w = g.w as usize;
    let cell = |x: f32, z: f32| {
        let i = ((f64::from(x) - g.origin_block_x as f64) / f64::from(g.r)).round() as i64;
        let j = ((f64::from(z) - g.origin_block_z as f64) / f64::from(g.r)).round() as i64;
        j.clamp(0, g.h as i64 - 1) as usize * w + i.rem_euclid(w as i64) as usize
    };
    let below: std::collections::HashSet<i16> = split
        .lakes
        .iter()
        .filter(|l| l.level < l.rim)
        .map(|l| l.level)
        .collect();
    let full: std::collections::HashSet<i16> = split
        .lakes
        .iter()
        .filter(|l| l.level == l.rim)
        .map(|l| l.level)
        .collect();
    for ch in &split.rivers.chains {
        let v = &split.rivers.vertices[ch.first as usize];
        let c0 = cell(v.x, v.z);
        let l = split.water_level[c0];
        if l != NODATA && i32::from(l) - i32::from(elev[c0]) >= 16 {
            assert!(
                full.contains(&l) || !below.contains(&l),
                "a chain starts in a lake that does not spill"
            );
        }
    }
}

#[test]
fn rough_ground_is_the_same_on_any_thread_count() {
    let p = HydroParams::default();
    let climate = |_: i64, z: i64| Climate::of(400.0, 18.0 + z as f64 / 40.0, f64::NAN, f64::NAN);
    let on = |threads: usize| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| run_rough(&climate, &p))
    };
    let (a, b) = (on(1), on(4));
    assert_eq!(a.water_level, b.water_level);
    assert_eq!(a.kind, b.kind);
    assert_eq!(a.lakes, b.lakes);
    assert_eq!(a.rivers, b.rivers);
}

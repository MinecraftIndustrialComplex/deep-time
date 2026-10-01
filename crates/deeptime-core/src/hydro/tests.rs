// SPDX-License-Identifier: LGPL-3.0-or-later
//! The P1-22 invariants on a synthetic planet (C = 1,024, 2 blocks a cell): land sloping south to
//! a sea, two families of diagonal valleys that cross (confluences; some cross the seam), and a bowl
//! that must become a lake.

use std::collections::HashSet;

use deeptime_format::GridDesc;
use deeptime_format::rivers::Mouth;

use super::HydroParams;
use super::network::{self, NODATA, Output};

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
    let climate = |_x: i64, _z: i64| (1500.0, 300.0);
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
        let climate = |_x: i64, _z: i64| (1500.0, 300.0);
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

#[test]
fn a_dry_climate_leaves_the_lake_without_an_outlet() {
    let g = grid();
    let elev = elevation();
    // Little rain, strong evaporation: runoff a few mm, the lake evaporates more than it gets.
    let climate = |_x: i64, _z: i64| (200.0, 1800.0);
    let p = HydroParams::default();
    let out = network::run(&network::Input {
        grid: &g,
        elev: &elev,
        sea_raw: (SEA * 16.0) as i32,
        m_per_block_h: 300.0,
        climate: &climate,
        params: &p,
        width_scale: 1.0,
    });
    assert!(!out.lakes.is_empty());
    assert!(out.lakes.iter().all(|l| l.endorheic));
    assert!(out.rivers.chains.len() < run().rivers.chains.len());
}

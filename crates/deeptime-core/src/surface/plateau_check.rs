// SPDX-License-Identifier: LGPL-3.0-or-later
//! The controlled plateau (the plateau erosion task, 2026-10-05): a flat plateau 4 km above the
//! sea with a short escarpment, worn by the deep-time surface's own kernels (priority flood,
//! routing, stream power with deposition and the critical slope, then hillslope diffusion) for
//! 10 Myr, on the map's 4.8-km cells (16k quick) and on the same landform drawn at Earth's size
//! (cells 8.15 times wider: Earth's 40,075 km over the 16k planet's 4,915 km). Each process's
//! gross lowering of the plateau interior and of its edge is reported, so the run's numbers can
//! be read against what each law should do when the landform is drawn narrower:
//!
//! - stream power with m = 1, n = 2: F = K A^m Δt / L^n is unchanged when A scales as L², so
//!   the solve is the same in metres of height at either size (without the critical slope);
//! - linear diffusion: κ Δt / Δx² scales as 1/Δx², so the same relief diffuses 66 times faster
//!   on the 8.15-times narrower map;
//! - the critical slope S_c cuts at a fixed gradient: 8.15 times steeper ground meets it far
//!   more often.
//!
//! `cargo test -p deeptime-core plateau -- --nocapture` prints the table.

use super::grid::Dims;
use super::pflood::{BASE, LAND};
use super::*;
use crate::sphere::SplitMix64;

/// Earth's circumference over the 16k planet's (16,384 blocks of 300 m).
const EARTH_OVER_MAP: f64 = 40_075.0 / 4_915.2;

struct Case {
    dx: f64,
    kd: f64,
    sc: f64,
    weight: f32,
}

/// Mean gross lowering per process (m/Myr): [interior, edge] × [spl, threshold, diffusion, net].
fn run(case: &Case) -> [[f64; 4]; 2] {
    let (w, hh) = (56usize, 44usize);
    let d = Dims::new(w, hh, case.dx, false);
    let n = d.cells();
    let mut rng = SplitMix64::new(7);
    // Distance in cells from the land's edge; the sea is the outer 4 cells.
    let dist = |c: usize| -> i64 {
        let (i, j) = ((c % w) as i64, (c / w) as i64);
        (i - 4)
            .min(w as i64 - 5 - i)
            .min(j - 4)
            .min(hh as i64 - 5 - j)
    };
    const PLAIN: i64 = 2;
    const EDGE: i64 = 2;
    let mut h = vec![0.0; n];
    let mut kind0 = vec![LAND; n];
    for c in 0..n {
        let k = dist(c);
        h[c] = if k < 0 {
            kind0[c] = BASE;
            -1000.0
        } else if k < PLAIN {
            200.0
        } else if k < PLAIN + EDGE {
            200.0 + 3800.0 * (k - PLAIN + 1) as f64 / (EDGE + 1) as f64
        } else {
            4000.0 + rng.next_f64() * 30.0
        };
    }
    let zone = |c: usize| -> Option<usize> {
        let k = dist(c);
        if k >= PLAIN + EDGE + 2 {
            Some(0)
        } else if (PLAIN..PLAIN + EDGE + 2).contains(&k) {
            Some(1)
        } else {
            None
        }
    };
    let dt = 2.0e6;
    let steps = 5;
    let p = spl::SplParams {
        m: 1.0,
        n: 2.0,
        g: 1.0,
        dt,
        sc: case.sc,
        max_iter: 6,
        tol_m: 0.01,
    };
    let kf = vec![3.0e-9; n]; // the built-in presets' K (ERO-001)
    let kd = vec![case.kd; n];
    let base_z = vec![0.0; n];
    let weight = vec![case.weight; n];
    let mut sums = [[0.0f64; 4]; 2];
    let mut area_steps = [0.0f64; 2];
    let (mut filled, mut area, mut out, mut qin, mut cut) = (
        vec![0.0; n],
        Vec::new(),
        vec![0.0; n],
        vec![0.0; n],
        vec![0.0; n],
    );
    let mut r = routing::Routing::default();
    for _ in 0..steps {
        let start = h.clone();
        let mut kind = kind0.clone();
        pflood::flood(&d, &h, &base_z, &mut kind, &mut filled);
        routing::receivers(&d, &filled, &kind, &mut r);
        routing::stack(&d, &kind, &mut r);
        routing::accumulate(&d, &r, &weight, &mut area);
        let input = spl::SplInput {
            kind: &kind,
            h0: &h,
            filled: &filled,
            base_z: &base_z,
            area: &area,
            kf: &kf,
        };
        spl::solve_cut(&d, &r, &input, &p, &mut out, &mut qin, Some(&mut cut));
        for c in 0..n {
            if kind[c] == BASE {
                continue;
            }
            if let Some(z) = zone(c) {
                sums[z][0] += (h[c] - out[c]).max(0.0);
                sums[z][1] += cut[c];
            }
            h[c] = out[c];
        }
        let before = h.clone();
        diffusion::step(&d, &mut h, &kd, dt);
        for c in 0..n {
            if let Some(z) = zone(c) {
                sums[z][2] += (before[c] - h[c]).max(0.0);
                sums[z][3] += start[c] - h[c];
                area_steps[z] += 1.0;
            }
        }
    }
    let myr = dt / 1.0e6;
    for z in 0..2 {
        for v in &mut sums[z] {
            *v /= area_steps[z] * myr;
        }
    }
    sums
}

#[test]
fn plateau_erosion_by_process_and_scale() {
    let map = 4800.0;
    let earth = map * EARTH_OVER_MAP;
    let rows: [(&str, Case); 8] = [
        ("map, κ 2, S_c 0.35", Case { dx: map, kd: 2.0, sc: 0.35, weight: 1.0 }),
        ("Earth, κ 2, S_c 0.35", Case { dx: earth, kd: 2.0, sc: 0.35, weight: 1.0 }),
        ("map, κ 2, no S_c", Case { dx: map, kd: 2.0, sc: f64::INFINITY, weight: 1.0 }),
        ("Earth, κ 2, no S_c", Case { dx: earth, kd: 2.0, sc: f64::INFINITY, weight: 1.0 }),
        ("map, κ 0.01, S_c 0.35", Case { dx: map, kd: 0.01, sc: 0.35, weight: 1.0 }),
        ("map, κ 2, S_c 0.35, arid", Case { dx: map, kd: 2.0, sc: 0.35, weight: 0.1 }),
        ("map, no κ, no S_c", Case { dx: map, kd: 0.0, sc: f64::INFINITY, weight: 1.0 }),
        ("Earth, no κ, no S_c", Case { dx: earth, kd: 0.0, sc: f64::INFINITY, weight: 1.0 }),
    ];
    let mut got = Vec::new();
    eprintln!("m/Myr over 10 Myr: interior spl threshold diffusion net | edge spl threshold diffusion net");
    for (name, case) in &rows {
        let s = run(case);
        eprintln!(
            "{name:28} {:8.1} {:8.1} {:8.1} {:8.1} | {:8.1} {:8.1} {:8.1} {:8.1}",
            s[0][0], s[0][1], s[0][2], s[0][3], s[1][0], s[1][1], s[1][2], s[1][3]
        );
        got.push(s);
    }
    // Without the critical slope, stream power is the same at either size (m = 1, n = 2) ...
    let (sm, se) = (got[6][1][0], got[7][1][0]);
    assert!(
        sm > 0.0 && ((sm - se) / sm).abs() < 0.02,
        "edge stream power {sm} on the map, {se} at Earth's size"
    );
    // ... while diffusion is about Earth/map squared (66) times stronger on the map, until it
    // is limited by the relief it has to work on.
    let (dm, de) = (got[2][1][2], got[3][1][2]);
    assert!(dm > 10.0 * de, "edge diffusion {dm} on the map, {de} at Earth's size");
    // The critical slope only bites on the map's steeper ground.
    assert!(got[0][1][1] > got[1][1][1]);
}

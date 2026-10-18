// SPDX-License-Identifier: LGPL-3.0-or-later
//! The controlled plateau (the plateau erosion task, 2026-10-05 and 2026-10-16): a flat plateau
//! 4 km above the sea with a short escarpment, worn by the deep-time surface's own kernels
//! (priority flood, routing, stream power with deposition and the critical slope, then hillslope
//! diffusion) for 10 Myr, on the map's 4.8-km cells (16k quick) and on the same landform drawn at
//! Earth's size (cells 8.15 times wider: Earth's 40,075 km over the 16k planet's 4,915 km). Each
//! process's gross lowering of the plateau interior and of its edge is reported, so the numbers
//! can be read against what each law should do when the landform is drawn narrower:
//!
//! - stream power with m = 1, n = 2: F = K A^m Δt / L^n is unchanged when A scales as L², so
//!   the solve is the same in metres of height at either size (without the critical slope);
//! - linear diffusion: κ Δt / Δx² scales as 1/Δx², so the same relief diffuses 66 times faster
//!   on the 8.15-times narrower map;
//! - the critical slope S_c cuts at a fixed gradient: 8.15 times steeper ground meets it far
//!   more often.
//!
//! Further cases: a measured hillslope diffusivity, a dry plateau (runoff weight 0.1), a plateau
//! whose centre is a terminal lake at its own height (an internally drained plateau), and a plateau
//! ringed by a rim range 500 m higher, whose closed interior either spills over the rim (as the
//! deep-time stage's priority flood routes every closed basin) or ends in a terminal lake.
//!
//! `cargo test -p deeptime-core plateau -- --nocapture` prints the table.

use super::grid::Dims;
use super::pflood::{BASE, LAND};
use super::*;
use crate::sphere::SplitMix64;

/// Earth's circumference over the 16k planet's (16,384 blocks of 300 m).
const EARTH_OVER_MAP: f64 = 40_075.0 / 4_915.2; // ledger: ERO-176
/// The 16k quick planet's L1 cell, m.
const MAP: f64 = 4800.0; // ledger: ERO-176 (decision: surface physics in map metres)

struct Case {
    dx: f64,
    kd: f64,
    sc: f64,
    weight: f32,
    /// Cells of escarpment between the plain and the plateau.
    edge: i64,
    /// The plateau's centre is a terminal lake at the plateau's height (its own base level).
    lake: bool,
    /// A rim range 500 m above the plateau, `RIM` cells wide, between the escarpment's top and the
    /// interior (a plateau ringed by mountains, so the interior is a closed basin).
    rim: bool,
    /// Stream-power K, m^(1−2m)/yr, and slope exponent n (the area exponent is n / 2).
    k: f64,
    n: f64,
}

/// Width of the rim range, cells.
const RIM: i64 = 2; // ledger: ERO-176

fn case(dx: f64, kd: f64, sc: f64, weight: f32, edge: i64, lake: bool) -> Case {
    Case {
        dx,
        kd,
        sc,
        weight,
        edge,
        lake,
        rim: false,
        k: 3.0e-9, // the presets' K before the 2026-10-18 recalibration (ERO-004)
        n: 2.0,
    }
}

fn rimmed(lake: bool) -> Case {
    Case {
        rim: true,
        ..case(MAP, 2.0, 0.35, 1.0, 2, lake)
    }
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
    let edge = case.edge;
    let rim = if case.rim { RIM } else { 0 };
    let top = if case.rim { 4500.0 } else { 4000.0 };
    let mut h = vec![0.0; n];
    let mut kind0 = vec![LAND; n];
    let mut base_z = vec![0.0; n];
    for c in 0..n {
        let k = dist(c);
        h[c] = if k < 0 {
            kind0[c] = BASE;
            -1000.0
        } else if k < PLAIN {
            200.0
        } else if k < PLAIN + edge {
            200.0 + (top - 200.0) * (k - PLAIN + 1) as f64 / (edge + 1) as f64
        } else if k < PLAIN + edge + rim {
            top + rng.next_f64() * 30.0
        } else {
            4000.0 + rng.next_f64() * 30.0
        };
    }
    let centre = hh / 2 * w + w / 2;
    if case.lake {
        kind0[centre] = BASE;
        h[centre] = 4000.0;
        base_z[centre] = 4000.0;
    }
    let zone = |c: usize| -> Option<usize> {
        let k = dist(c);
        if case.lake && c == centre {
            None
        } else if k >= PLAIN + edge + rim + 2 {
            Some(0)
        } else if (PLAIN..PLAIN + edge + rim + 2).contains(&k) {
            Some(1)
        } else {
            None
        }
    };
    let dt = 2.0e6;
    let steps = 5;
    let p = spl::SplParams {
        m: case.n / 2.0,
        n: case.n,
        g: 1.0,
        dt,
        sc: case.sc,
        max_iter: 6,
        tol_m: 0.01,
    };
    let kf = vec![case.k; n];
    let kd = vec![case.kd; n];
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
    let earth = MAP * EARTH_OVER_MAP;
    let inf = f64::INFINITY;
    let rows = [
        (
            "map, κ 2, S_c 0.35 (presets)",
            case(MAP, 2.0, 0.35, 1.0, 2, false),
        ),
        (
            "Earth, κ 2, S_c 0.35",
            case(earth, 2.0, 0.35, 1.0, 2, false),
        ),
        ("map, no κ, no S_c", case(MAP, 0.0, inf, 1.0, 2, false)),
        ("Earth, no κ, no S_c", case(earth, 0.0, inf, 1.0, 2, false)),
        ("map, κ 2, no S_c", case(MAP, 2.0, inf, 1.0, 2, false)),
        ("Earth, κ 2, no S_c", case(earth, 2.0, inf, 1.0, 2, false)),
        (
            "map, κ 0.0036, S_c 0.35",
            case(MAP, 0.0036, 0.35, 1.0, 2, false),
        ),
        (
            "map, κ 2, S_c 0.35, dry 0.1",
            case(MAP, 2.0, 0.35, 0.1, 2, false),
        ),
        (
            "map, κ 2, S_c 0.35, lake",
            case(MAP, 2.0, 0.35, 1.0, 2, true),
        ),
        ("map, κ 0.0036, lake", case(MAP, 0.0036, 0.35, 1.0, 2, true)),
        (
            "map, 1-cell edge, no κ",
            case(MAP, 0.0, 0.35, 1.0, 1, false),
        ),
        (
            "Earth, 1-cell edge, no κ",
            case(earth, 0.0, 0.35, 1.0, 1, false),
        ),
        ("map, rim, drains over it", rimmed(false)),
        ("map, rim, terminal lake", rimmed(true)),
        (
            "map, the presets since 2026-10-18",
            Case {
                k: 1.5e-9,                             // ledger: ERO-004
                ..case(MAP, 2.0, 0.15, 1.0, 2, false)  // ledger: ERO-013, ERO-023
            },
        ),
    ];
    // Extra rows for screening a surface law, printed only: DEEPTIME_PLATEAU_LAWS="k:n:sc:kd/..."
    let extra: Vec<(String, Case)> = std::env::var("DEEPTIME_PLATEAU_LAWS")
        .unwrap_or_default()
        .split('/')
        .filter_map(|law| {
            let v: Vec<f64> = law.split(':').filter_map(|x| x.parse().ok()).collect();
            (v.len() == 4).then(|| {
                let c = Case {
                    k: v[0],
                    n: v[1],
                    ..case(MAP, v[3], v[2], 1.0, 2, false)
                };
                (
                    format!("map, K {:e} n {} S_c {} κ {}", v[0], v[1], v[2], v[3]),
                    c,
                )
            })
        })
        .collect();
    let mut got = Vec::new();
    eprintln!(
        "m/Myr over 10 Myr: interior spl threshold diffusion net | edge spl threshold diffusion net"
    );
    for (name, case) in &rows {
        let s = run(case);
        eprintln!(
            "{name:30} {:8.1} {:8.1} {:8.1} {:8.1} | {:8.1} {:8.1} {:8.1} {:8.1}",
            s[0][0], s[0][1], s[0][2], s[0][3], s[1][0], s[1][1], s[1][2], s[1][3]
        );
        got.push(s);
    }
    for (name, case) in &extra {
        let s = run(case);
        eprintln!(
            "{name:30} {:8.1} {:8.1} {:8.1} {:8.1} | {:8.1} {:8.1} {:8.1} {:8.1}",
            s[0][0], s[0][1], s[0][2], s[0][3], s[1][0], s[1][1], s[1][2], s[1][3]
        );
    }
    let rel = |a: f64, b: f64| ((a - b) / a).abs();
    // Without diffusion and the critical slope, stream power is the same at either size
    // (m = 1, n = 2), on the edge and in the interior ...
    assert!(got[2][1][0] > 0.0 && rel(got[2][1][0], got[3][1][0]) < 0.02);
    assert!(got[2][0][0] > 0.0 && rel(got[2][0][0], got[3][0][0]) < 0.02);
    // ... while diffusion is far stronger on the map (κ Δt / Δx², 66 times).
    assert!(
        got[4][1][2] > 10.0 * got[5][1][2],
        "edge diffusion map/Earth"
    );
    // Diffusion does not wear the plateau: it rounds the escarpment's lip, and the interior's
    // stream power falls (map, κ 2 against no κ).
    assert!(
        got[0][0][0] < 0.5 * got[2][0][0],
        "diffusion protects the interior"
    );
    // Runoff does: a plateau at a tenth of the runoff wears at a small part of the rate.
    assert!(
        got[7][0][3] < 0.2 * got[0][0][3],
        "a dry plateau wears slowly"
    );
    // A closed interior behind a rim range barely helps: ending in a terminal lake instead of
    // spilling over the rim, it still wears at most of the rate (the rim's own rivers cut through).
    assert!(
        got[13][0][3] > 0.5 * got[12][0][3],
        "a terminal lake behind a rim does not save the plateau"
    );
    // The recalibrated law (2026-10-18) still wears the plateau's interior by stream power, not
    // by the critical slope, which cuts nothing there at 0.15 either.
    assert!(
        got[14][0][0] > 10.0 * got[14][0][1].max(0.1),
        "the critical slope does not wear the interior"
    );
}

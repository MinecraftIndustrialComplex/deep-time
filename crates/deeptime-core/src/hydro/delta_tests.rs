// SPDX-License-Identifier: LGPL-3.0-or-later
//! The delta stage's laws and its plains and distributaries on a synthetic planet (C = 1,024, 2
//! blocks a cell): one V-shaped valley gathering all the land into one river that runs south across
//! a band of shallows to a shelf.

use deeptime_format::GridDesc;
use deeptime_format::rivers::{Mouth, NO_CHAIN};

use super::delta::{self, Plains};
use super::network::{self, Output};
use super::{Climate, HydroParams};
use crate::params::BuildFit;

const C: u32 = 1024;
const SEA: f64 = 63.0;
/// The coast's row: the beach band from here, the shelf from `SHELF`.
const COAST: usize = 110;
const SHELF: usize = 118;

fn close(a: f64, b: f64, rel: f64) -> bool {
    (a - b).abs() <= rel * b.abs()
}

#[test]
fn bqart_area_and_depth_follow_their_sources() {
    // Q 1,000 m³/s (31.56 km³/yr), A 10⁵ km², R 2 km, T 10 °C: 0.02 · 31.56^0.31 · 316.2 · 2 · 10.
    let q_km3 = 1000.0 * 31_557_600.0 / 1e9;
    let warm = 0.02 * f64::powf(q_km3, 0.31) * 1e5f64.sqrt() * 2.0 * 10.0;
    assert!(close(delta::bqart_kg_s(1000.0, 1e5, 2.0, 10.0, 0.0), warm, 1e-12));
    assert!(close(warm, 368.8, 0.002), "{warm}");
    // Below 2 °C the cold branch: 2 ω B Q^0.31 A^0.5 R, whatever T.
    let cold = delta::bqart_kg_s(1000.0, 1e5, 2.0, -5.0, 0.0);
    assert!(close(cold, warm / 10.0 * 2.0, 1e-12));
    // A lake on the way traps its share.
    assert!(close(
        delta::bqart_kg_s(1000.0, 1e5, 2.0, 10.0, 0.25),
        0.75 * warm,
        1e-12
    ));
    // log10 A* = 5.463 + 0.932 log10 Qs.
    assert!(close(delta::delta_area_m2(100.0), 10f64.powf(7.327), 1e-12));
    assert_eq!(delta::delta_area_m2(0.0), 0.0);
    // D = 0.08 Q^0.65, at most 69 m.
    assert!(close(delta::plain_depth_m(1000.0), 7.130, 0.001));
    assert_eq!(delta::plain_depth_m(1e7), 69.0);
}

#[test]
fn the_build_fit_expands_what_it_compressed() {
    let fit = BuildFit {
        knee_y: 250.0,
        top_y: 310.0,
    };
    for y in [-40.0, 63.0, 250.0, 260.0, 300.0, 420.0] {
        let s = fit.compress(y);
        assert!((fit.expand(s) - y).abs() < 1e-6 * y.abs().max(1.0), "{y}");
    }
    assert!(fit.expand(310.0).is_infinite());
}

/// Terrain height, blocks: land sloping gently south (0.02 a row) to the coast at row `COAST`, its
/// sides falling steeply (0.15 a column) to the valley's axis at column 256, so every cell drains
/// sideways to the axis (0.15 beats a diagonal's (0.15 + 0.02) / √2) and the axis carries the whole
/// land south; a band of shallows (half a block under the sea's surface) to row `SHELF`, then a
/// shelf deepening south.
fn height(i: usize, j: usize) -> f64 {
    let (fi, fj) = (i as f64, j as f64);
    if j >= SHELF {
        return SEA - 1.5 - (fj - SHELF as f64) * 0.1;
    }
    if j >= COAST {
        return SEA - 0.5;
    }
    let valley = 0.15 * (fi - 256.0).abs();
    SEA + 0.5 + 0.02 * (COAST as f64 - fj) + valley
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

const SEA_RAW: i32 = (SEA * 16.0) as i32;

fn net(elev: &[i16], params: &HydroParams) -> Output {
    let g = grid();
    let climate = |_x: i64, _z: i64| Climate::steady(2500.0, 20.0);
    network::run(&network::Input {
        grid: &g,
        elev,
        sea_raw: SEA_RAW,
        m_per_block_h: 300.0,
        climate: &climate,
        params,
        width_scale: 1.0,
        mouths: true,
    })
}

fn input<'a>(g: &'a GridDesc, elev: &'a [i16], params: &'a HydroParams) -> delta::Input<'a> {
    delta::Input {
        grid: g,
        elev,
        sea_raw: SEA_RAW,
        m_per_block_h: 300.0,
        m_per_block_v: 35.0,
        fit: BuildFit {
            knee_y: 250.0,
            top_y: 310.0,
        },
        params,
        width_scale: 1.0,
    }
}

/// The two passes as the bake's water part runs them.
fn deltas(params: &HydroParams) -> (Vec<i16>, Plains, Output, Vec<delta::Split>) {
    let g = grid();
    let mut elev = elevation();
    let first = net(&elev, params);
    assert!(!first.mouths.is_empty(), "no sea mouths");
    let plains = delta::grow(&input(&g, &elev, params), &first);
    plains.apply(&mut elev);
    let mut second = net(&elev, params);
    let splits = delta::distributaries(&input(&g, &elev, params), &elev, &plains, &mut second);
    (elev, plains, second, splits)
}

#[test]
fn a_large_river_builds_a_plain_above_the_sea_and_splits_across_it() {
    // A larger plain and closer mouths than the defaults, so this small planet's front holds
    // more than one of its wide channels (the defaults are measured on real worlds).
    let params = HydroParams {
        delta_area_mult: 16.0,
        delta_mouth_spacing: 1.0,
        ..HydroParams::default()
    };
    let base = elevation();
    let (elev, plains, second, splits) = deltas(&params);
    assert!(!plains.plains.is_empty(), "no delta");
    let biggest = plains
        .plains
        .iter()
        .max_by(|a, b| a.q_m3s.total_cmp(&b.q_m3s))
        .unwrap();
    assert!(biggest.qs_kg_s > 0.0 && biggest.a_star_km2 > 0.0 && biggest.d_m > 0.0);
    assert!(biggest.cells.len() >= delta::MIN_PLAIN_CELLS);
    // Every plain cell was sea or beach and is land a block above the sea's top water block now:
    // its height in (sea level, sea level + 1].
    for p in &plains.plains {
        for &c in &p.cells {
            let c = c as usize;
            assert!(i32::from(base[c]) <= SEA_RAW, "cell {c} was land");
            let e = i32::from(elev[c]);
            assert!(e > SEA_RAW && e <= SEA_RAW + 16, "cell {c} at {e}");
        }
    }
    // The river comes off the land onto the plain and runs across it to its front: the second
    // network's largest sea mouth ends on the plain, and the ground under its course never rises
    // downstream there (the ramp along the outflow, HYD-136).
    let main = second
        .mouths
        .iter()
        .max_by(|a, b| a.q.total_cmp(&b.q))
        .expect("a sea mouth");
    assert!(plains.owner.contains_key(&main.root), "the river leaves its plain");
    let on: Vec<i16> = main
        .stem
        .iter()
        .filter(|&&c| plains.owner.contains_key(&c))
        .map(|&c| elev[c as usize])
        .collect();
    assert!(on.len() >= 3, "{} stem cells on the plain", on.len());
    assert!(on.windows(2).all(|p| p[1] <= p[0]), "{on:?}");
    // The largest plain splits: more than one mouth, each new chain leaving its parent at one of
    // the parent's vertices and ending in the sea.
    let r = &second.rivers;
    r.validate().expect("the river file holds");
    let added: u32 = splits.iter().map(|s| s.added).sum();
    assert!(added >= 1, "no distributaries: {splits:?}");
    assert!(splits.iter().any(|s| s.mouths >= 2));
    let children: Vec<_> = r.chains.iter().filter(|c| c.from != NO_CHAIN).collect();
    assert_eq!(children.len() as u32, added);
    for ch in &children {
        assert_eq!(ch.mouth, Mouth::Sea);
        let v0 = r.vertices[ch.first as usize];
        let parent = &r.chains[ch.from as usize];
        assert!(
            (parent.first..parent.first + parent.count)
                .any(|i| r.vertices[i as usize].x == v0.x && r.vertices[i as usize].z == v0.z)
        );
        // A branch carries less than the apex and its water stands at the sea's surface.
        let last = r.vertices[(ch.first + ch.count - 1) as usize];
        assert!(last.w <= SEA as f32 + 1e-3);
        assert!(last.q > 0.0);
    }
}

#[test]
fn no_deltas_without_volume_and_the_same_on_a_rerun() {
    // A multiplier of 0 leaves no volume: no plain.
    let none = HydroParams {
        delta_area_mult: 0.0,
        ..HydroParams::default()
    };
    let (elev, plains, _, splits) = deltas(&none);
    assert!(plains.plains.is_empty() && splits.is_empty());
    assert_eq!(elev, elevation());
    // The stage is deterministic.
    let p = HydroParams::default();
    let (e1, p1, s1, _) = deltas(&p);
    let (e2, p2, s2, _) = deltas(&p);
    assert_eq!(e1, e2);
    assert_eq!(p1.owner, p2.owner);
    assert_eq!(s1.rivers, s2.rivers);
}

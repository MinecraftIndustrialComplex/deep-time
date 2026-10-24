// SPDX-License-Identifier: LGPL-3.0-or-later
//! The braids (HYD-171 to HYD-175) on straight fixture rivers: a steep river on a plain divides
//! around islands 5 widths long on alternate sides, the threads and the island as Leopold &
//! Wolman's divided channels are, the same on the planet at 0° and 55°; a gentle river keeps one
//! channel off a fan and braids on one; a narrow channel, a narrow plain and a neighbour stop the
//! islands.

use deeptime_format::Projection;
use deeptime_format::rivers::{Chain, Mouth, NO_CHAIN, RiverVertex, Rivers};
use serde_json::Value;

use super::braid::{self, Input};

const C: u32 = 16_384;
const M_H: f64 = 300.0;
const M_V: f64 = 35.0;

fn proj() -> Projection {
    Projection::of(f64::from(C))
}

/// The plain's level for water surface w: the bank, one block over the water (HYD-151).
fn bank(w: f64) -> i32 {
    ((w * 16.0).round() as i32 + 15).div_euclid(16) * 16 + 1
}

fn chain(r: &Rivers, k: usize) -> &[RiverVertex] {
    let ch = r.chains[k];
    &r.vertices[ch.first as usize..(ch.first + ch.count) as usize]
}

/// A river running east along block row z, `len` blocks of planet long, a vertex every 2 blocks
/// of planet, its surface falling at `slope` (m/m on the planet) from y 150; returns its id.
fn east(r: &mut Rivers, z: f64, len: f64, hw: f64, q: f64, slope: f64) -> u32 {
    let ex = proj().cos_lat(z);
    let n = (len / 2.0).round() as usize;
    let first = r.vertices.len() as u32;
    for i in 0..=n {
        let s = i as f64 * 2.0;
        r.vertices.push(RiverVertex {
            x: ((s - 0.5 * len) / ex) as f32,
            z: z as f32,
            w: (150.0 - slope * s * M_H / M_V) as f32,
            hw: hw as f32,
            depth: 2.0,
            q: q as f32,
        });
    }
    r.chains
        .push(Chain::new(first, n as u32 + 1, Mouth::Sea, u32::MAX));
    r.chains.len() as u32 - 1
}

/// The water surface of a river from [`east`] at block x on row z.
fn w_at(z: f64, len: f64, slope: f64, x: f64) -> f64 {
    let s = x * proj().cos_lat(z) + 0.5 * len;
    150.0 - slope * s.clamp(0.0, len) * M_H / M_V
}

fn run(
    r: &mut Rivers,
    plain: &(dyn Fn(f64, f64) -> Option<i32> + Sync),
    fan: &(dyn Fn(f64, f64) -> bool + Sync),
) -> Value {
    r.build_index();
    let s = braid::apply(
        r,
        &Input {
            plain,
            fan,
            lambert: Some(proj()),
            m_per_block_h: M_H,
            m_per_block_v: M_V,
        },
    );
    r.validate().unwrap();
    for k in 0..r.chains.len() {
        let v = chain(r, k);
        assert!(
            v.windows(2).all(|p| p[1].w <= p[0].w),
            "chain {k}: the surface rises"
        );
    }
    s
}

/// Each thread of a river from [`east`] on row z0: its largest offset across the course (blocks
/// of planet, + south) and its length on the planet between its ends.
fn threads(r: &Rivers, k: u32, z0: f64) -> Vec<(f64, f64)> {
    let p = proj();
    let mut out = Vec::new();
    for (t, ch) in r.chains.iter().enumerate() {
        if ch.from != k {
            continue;
        }
        assert_eq!((ch.mouth, ch.into), (Mouth::River, k), "thread {t}");
        let v = chain(r, t);
        let course = chain(r, k as usize);
        let (a, b) = (v[0], v[v.len() - 1]);
        for end in [a, b] {
            assert!(
                course.iter().any(|q| q.x == end.x && q.z == end.z),
                "thread {t} leaves or rejoins off its course"
            );
        }
        let off = v
            .iter()
            .map(|q| (f64::from(q.z) - z0) / p.cos_lat(f64::from(q.z)))
            .fold(0.0f64, |m, y| if y.abs() > m.abs() { y } else { m });
        let len = f64::from(b.x - a.x) * p.cos_lat(z0);
        out.push((off, len));
    }
    out
}

/// A steep river (0.0033 against the braided line's 0.0022 at 50 m³/s) 8 blocks wide on a plain
/// 60 blocks of planet either side, at latitude `lat`.
fn steep_on_a_plain(lat: f64) -> (Rivers, Value, f64) {
    let z0 = proj().z_of_lat(lat);
    let (len, slope) = (3000.0, 0.0033);
    let mut r = Rivers::empty(C);
    east(&mut r, z0, len, 4.0, 50.0, slope);
    let cos0 = proj().cos_lat(z0);
    let plain = move |x: f64, z: f64| {
        (((z - z0) / cos0).abs() <= 60.0).then(|| bank(w_at(z0, len, slope, x)))
    };
    let s = run(&mut r, &plain, &|_, _| false);
    (r, s, z0)
}

#[test]
fn a_steep_river_on_a_plain_divides_around_islands() {
    let (r, s, z0) = steep_on_a_plain(0.0);
    let th = threads(&r, 0, z0);
    assert!(th.len() >= 20, "{} threads: {s}", th.len());
    assert_eq!(s["threads_added"].as_u64(), Some(th.len() as u64));
    assert_eq!(r.chains.len(), 1 + th.len());
    // The thread's centre at the island's middle: the course's half-width 0.9 hw, the island
    // 2 hw, the thread's half-width 0.9 hw: 3.8 hw = 15.2 blocks.
    for &(off, len) in &th {
        assert!((off.abs() - 15.2).abs() < 0.3, "offset {off}");
        // 5 widths, to the course's vertex spacing (2 blocks).
        assert!((len - 40.0).abs() <= 2.5, "island {len} blocks long");
    }
    // On alternate sides.
    assert!(th.windows(2).all(|p| p[0].0.signum() != p[1].0.signum()));
    // Each thread 0.9 of the width and 0.75 of the depth at its middle, carrying half the flow;
    // the course too.
    let course = chain(&r, 0);
    let min_hw = course.iter().map(|v| v.hw).fold(f32::MAX, f32::min);
    let min_depth = course.iter().map(|v| v.depth).fold(f32::MAX, f32::min);
    assert!((min_hw - 3.6).abs() < 0.05 && (min_depth - 1.5).abs() < 0.02);
    for k in 1..r.chains.len() {
        let v = chain(&r, k);
        let mid = &v[v.len() / 2];
        assert!((mid.hw - 3.6).abs() < 0.05 && (mid.depth - 1.5).abs() < 0.02);
        assert!(v.iter().all(|q| (q.q - 25.0).abs() < 1e-3));
    }
    assert!(s["braided_km"].as_f64().unwrap() > 0.5 * s["steep_on_plain_km"].as_f64().unwrap());
}

#[test]
fn the_islands_are_the_same_on_the_planet_at_55_degrees() {
    let (r0, s0, z0) = steep_on_a_plain(0.0);
    let (r55, s55, z55) = steep_on_a_plain(55.0);
    let (a, b) = (threads(&r0, 0, z0), threads(&r55, 0, z55));
    assert_eq!(a.len(), b.len(), "{s0}\n{s55}");
    for (p, q) in a.iter().zip(&b) {
        assert!((p.0 - q.0).abs() < 0.3, "offsets {} and {}", p.0, q.0);
        assert!((p.1 - q.1).abs() < 2.5, "lengths {} and {}", p.1, q.1);
    }
    // On the grid the thread at 55° stands cos φ as far across (8.7 blocks for 15.2).
    let cos = proj().cos_lat(z55);
    let grid = chain(&r55, 1)
        .iter()
        .map(|v| (f64::from(v.z) - z55).abs())
        .fold(0.0, f64::max);
    assert!((grid - 15.2 * cos).abs() < 0.3, "{grid}");
}

#[test]
fn a_gentle_river_braids_on_a_fan_alone() {
    let z0 = 0.0;
    let (len, slope) = (3000.0, 0.001);
    for on_fan in [false, true] {
        let mut r = Rivers::empty(C);
        east(&mut r, z0, len, 4.0, 50.0, slope);
        let plain = move |x: f64, z: f64| (z.abs() <= 60.0).then(|| bank(w_at(z0, len, slope, x)));
        let fan = move |x: f64, _: f64| on_fan && x > 0.0;
        let s = run(&mut r, &plain, &fan);
        let th = threads(&r, 0, z0);
        if on_fan {
            assert!(th.len() >= 10, "{s}");
            assert!(s["braided_on_fans_km"].as_f64().unwrap() > 0.0);
            // Only on the fan's half.
            for k in 1..r.chains.len() {
                assert!(chain(&r, k)[0].x >= -2.0);
            }
        } else {
            assert!(th.is_empty(), "{s}");
            assert_eq!(s["steep_on_plain_km"].as_f64(), Some(0.0));
        }
    }
}

#[test]
fn a_narrow_channel_or_plain_keeps_one_channel() {
    let z0 = 0.0;
    let (len, slope) = (3000.0, 0.0033);
    // Half-width 1: under 3 blocks wide.
    let mut r = Rivers::empty(C);
    east(&mut r, z0, len, 1.0, 50.0, slope);
    let plain = move |x: f64, z: f64| (z.abs() <= 60.0).then(|| bank(w_at(z0, len, slope, x)));
    let s = run(&mut r, &plain, &|_, _| false);
    assert_eq!(r.chains.len(), 1, "{s}");
    assert!(s["too_narrow_km"].as_f64().unwrap() > 0.0);
    // A plain 10 blocks either side: no room for the thread and its bank.
    let mut r = Rivers::empty(C);
    east(&mut r, z0, len, 4.0, 50.0, slope);
    let plain = move |x: f64, z: f64| (z.abs() <= 10.0).then(|| bank(w_at(z0, len, slope, x)));
    let s = run(&mut r, &plain, &|_, _| false);
    assert_eq!(r.chains.len(), 1, "{s}");
    assert!(s["no_room"].as_u64().unwrap() > 0);
    assert_eq!(chain(&r, 0).iter().map(|v| v.hw).fold(0.0, f32::max), 4.0);
}

#[test]
fn a_neighbour_sends_the_islands_to_the_far_side() {
    let z0 = 0.0;
    let (len, slope) = (3000.0, 0.0033);
    let mut r = Rivers::empty(C);
    east(&mut r, z0, len, 4.0, 50.0, slope);
    east(&mut r, z0 + 30.0, len, 4.0, 50.0, slope);
    let plain = move |x: f64, z: f64| (z.abs() <= 100.0).then(|| bank(w_at(z0, len, slope, x)));
    let s = run(&mut r, &plain, &|_, _| false);
    let (a, b) = (threads(&r, 0, z0), threads(&r, 1, z0 + 30.0));
    assert!(a.len() >= 10 && b.len() >= 10, "{s}");
    // Each river's islands away from the other, and no thread crosses into the gap's far half.
    assert!(a.iter().all(|t| t.0 < 0.0) && b.iter().all(|t| t.0 > 0.0));
    for (k, ch) in r.chains.iter().enumerate().skip(2) {
        let v = chain(&r, k);
        let (lo, hi) = if ch.from == 0 {
            (f64::MIN, 0.0)
        } else {
            (30.0, f64::MAX)
        };
        assert!(
            v.iter().all(|q| (lo..=hi).contains(&f64::from(q.z))),
            "thread {k}"
        );
    }
    assert_eq!(r.chains[0].from, NO_CHAIN);
}

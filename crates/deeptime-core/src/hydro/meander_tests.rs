// SPDX-License-Identifier: LGPL-3.0-or-later
//! The meanders (HYD-155 to HYD-159): the sine-generated curve against Langbein & Leopold's
//! (1966) table, and straight fixture rivers on a plain: the wavelength, the amplitude the plain's
//! room allows, the braided line, the same belt on the planet at 0° and 55°, the joins, and the
//! chains that stay as they were.

use std::f64::consts::{FRAC_PI_2, PI};

use deeptime_format::Projection;
use deeptime_format::rivers::{Chain, Mouth, NO_CHAIN, RiverVertex, Rivers};
use serde_json::Value;

use super::meander::{self, Input};

const M_H: f64 = 300.0;
const M_V: f64 = 35.0;

/// The plain's level for water surface w: the bank, one block over the water (HYD-151).
fn bank(w: f64) -> i32 {
    ((w * 16.0).round() as i32 + 15).div_euclid(16) * 16 + 1
}

fn chain(r: &Rivers, k: usize) -> &[RiverVertex] {
    let ch = r.chains[k];
    &r.vertices[ch.first as usize..(ch.first + ch.count) as usize]
}

/// A straight chain from `a` to `b` (blocks), a vertex every `spacing` blocks, its surface falling
/// from w0 to w1; returns its id.
#[allow(clippy::too_many_arguments)]
fn line(
    r: &mut Rivers,
    a: (f64, f64),
    b: (f64, f64),
    spacing: f64,
    hw: f64,
    q: f64,
    w: (f64, f64),
    mouth: Mouth,
    into: u32,
) -> u32 {
    let len = (b.0 - a.0).hypot(b.1 - a.1);
    let n = (len / spacing).ceil().max(1.0) as usize;
    let first = r.vertices.len() as u32;
    for i in 0..=n {
        let t = i as f64 / n as f64;
        r.vertices.push(RiverVertex {
            x: (a.0 + (b.0 - a.0) * t) as f32,
            z: (a.1 + (b.1 - a.1) * t) as f32,
            w: (w.0 + (w.1 - w.0) * t) as f32,
            hw: hw as f32,
            depth: 2.0,
            q: q as f32,
        });
    }
    r.chains.push(Chain::new(first, n as u32 + 1, mouth, into));
    r.chains.len() as u32 - 1
}

fn run(r: &mut Rivers, plain: &(dyn Fn(f64, f64) -> Option<i32> + Sync)) -> Value {
    r.build_index();
    let c = f64::from(r.c);
    let s = meander::apply(
        r,
        &Input {
            plain,
            lambert: Some(Projection::of(c)),
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

/// The planet's metric at block z: blocks of planet per block east and south.
fn metric(c: f64, z: f64) -> (f64, f64) {
    let cp = Projection::of(c).cos_lat(z);
    (cp, 1.0 / cp)
}

/// A polyline's length on the planet, blocks.
fn planet_len(c: f64, v: &[RiverVertex]) -> f64 {
    v.windows(2)
        .map(|p| {
            let (ex, ez) = metric(c, 0.5 * f64::from(p[0].z + p[1].z));
            (f64::from(p[1].x - p[0].x) * ex).hypot(f64::from(p[1].z - p[0].z) * ez)
        })
        .sum()
}

/// A chain's shape on the planet about the straight axis it was drawn on, through `origin` along
/// `dir` (a unit grid vector, east or south): its largest offset across the axis (blocks of
/// planet, measured locally) and how many times it crosses the axis.
fn across(c: f64, v: &[RiverVertex], origin: (f64, f64), dir: (f64, f64)) -> (f64, usize) {
    let (mut off, mut cross, mut last) = (0.0f64, 0usize, 0.0);
    for p in v {
        let (ex, ez) = metric(c, f64::from(p.z));
        let (dx, dz) = (f64::from(p.x) - origin.0, f64::from(p.z) - origin.1);
        let y = -dir.1 * dx * ex + dir.0 * dz * ez;
        off = off.max(y.abs());
        if y.abs() > 2.0 {
            if last != 0.0 && y.signum() != last {
                cross += 1;
            }
            last = y.signum();
        }
    }
    (off, cross)
}

#[test]
fn the_curve_matches_langbein_and_leopolds_table() {
    // LL1966, p. H5, footnote 2 (k against ω, the maximum angle in radians). 1 / J₀ gives 1.066,
    // 1.307, 1.954 and 2.710: within 2 % of the paper's numbers (its 2.67 is 1.5 % under).
    for (omega, k) in [(0.5, 1.06), (1.0, 1.30), (1.5, 1.96), (1.75, 2.67)] {
        let got = meander::sinuosity(omega);
        assert!(
            (got / k - 1.0).abs() < 0.02,
            "ω {omega}: k {got}, the table {k}"
        );
    }
    // J₀'s first zero and Struve's H₀ against its series (2/π) Σ (−1)^n x^(2n+1) / (1·3···(2n+1))².
    assert!(meander::bessel_j0(2.404_825_557_7).abs() < 1e-9);
    for x in [0.3, 1.0, 1.5, 2.0] {
        let (mut sum, mut term) = (0.0, x);
        for n in 0..30 {
            sum += term;
            let d = (2 * n + 3) as f64;
            term *= -x * x / (d * d);
        }
        let h = 2.0 / PI * sum;
        assert!((meander::struve_h0(x) - h).abs() < 1e-9, "H₀({x})");
    }
    // The cap: sinuosity 2 at ω ≈ 1.52, half amplitude about 0.36 of the wavelength.
    let w = meander::omega_of_sinuosity(meander::MAX_SINUOSITY);
    assert!((w - 1.5208).abs() < 2e-3, "ω at k 2: {w}");
    let cap = meander::ratio_cap();
    assert!((0.34..0.38).contains(&cap), "a / λ at k 2: {cap}");
    assert!((meander::sinuosity_of_ratio(cap) - 2.0).abs() < 1e-6);
    // A gentle curve's sinuosity by the table's inverse.
    let r = meander::amplitude_ratio(1.0);
    assert!((meander::sinuosity_of_ratio(r) - 1.30).abs() < 0.01);
}

#[test]
fn the_shape_is_a_sine_when_gentle_and_odd_about_each_crossing() {
    for i in 0..32 {
        let th = i as f64 * PI / 16.0;
        assert!(
            (meander::y_hat(th, 1e-4) - th.sin()).abs() < 1e-3,
            "gentle at {th}"
        );
        let cap = meander::ratio_cap();
        let (a, b) = (meander::y_hat(th, cap), meander::y_hat(th + PI, cap));
        assert!((a + b).abs() < 2e-3, "odd at {th}: {a} {b}");
    }
    let cap = meander::ratio_cap();
    assert!(meander::y_hat(0.0, cap).abs() < 1e-3);
    assert!((meander::y_hat(FRAC_PI_2, cap) - 1.0).abs() < 1e-3);
    // Squarer than a sine at k 2: the bend's flank is steep, so a quarter of the way to the apex
    // the channel is already well across.
    assert!(meander::y_hat(PI / 8.0, cap) > (PI / 8.0).sin() + 0.1);
}

#[test]
fn a_straight_river_on_a_wide_plain_meanders_at_ten_point_nine_widths() {
    let c = 16_384;
    let mut r = Rivers::empty(c);
    line(
        &mut r,
        (-2000.0, 0.0),
        (2000.0, 0.0),
        2.0,
        5.0,
        100.0,
        (69.9, 69.1),
        Mouth::Sea,
        u32::MAX,
    );
    let before = chain(&r, 0).to_vec();
    let plain = |_: f64, _: f64| Some(bank(69.9));
    let s = run(&mut r, &plain);
    let v = chain(&r, 0);
    assert_eq!(v[0], before[0]);
    assert_eq!(v[v.len() - 1], before[before.len() - 1]);
    let k = s["reach_sinuosity"].as_f64().unwrap();
    assert!((1.8..=2.05).contains(&k), "sinuosity {k}: {s}");
    let (len, axis) = (
        planet_len(f64::from(c), v),
        planet_len(f64::from(c), &before),
    );
    let (off, crossings) = across(f64::from(c), v, (-2000.0, 0.0), (1.0, 0.0));
    // Two crossings a wavelength; λ = 10.9 × the width 10.
    let lambda = 2.0 * axis / crossings as f64;
    assert!(
        (lambda / 109.0 - 1.0).abs() < 0.1,
        "wavelength {lambda} (axis {axis}, length {len})"
    );
    let cap = 109.0 * meander::ratio_cap();
    assert!(
        off <= cap * 1.02 && off >= cap * 0.9,
        "offset {off}, the cap {cap}"
    );
    assert!(len / axis > 1.7, "length {len} over {axis}");
}

#[test]
fn the_plain_bounds_the_belt_and_a_narrow_one_keeps_the_course() {
    let c = 16_384;
    // Room 20 either side: the belt fills it less the half-width. Room 3.5 with a half-width of 3:
    // under a block of amplitude.
    for (half, moves) in [(20.0, true), (3.5, false)] {
        let mut r = Rivers::empty(c);
        line(
            &mut r,
            (-1500.0, 0.0),
            (1500.0, 0.0),
            2.0,
            3.0,
            50.0,
            (69.9, 69.1),
            Mouth::Sea,
            u32::MAX,
        );
        let before = r.vertices.clone();
        let plain = move |_: f64, z: f64| (z.abs() <= half).then_some(bank(69.9));
        let s = run(&mut r, &plain);
        if moves {
            let off = r
                .vertices
                .iter()
                .fold(0.0f64, |m, p| m.max(f64::from(p.z).abs()));
            assert!(off >= 10.0, "offset {off}: {s}");
            assert!(
                off + 3.0 <= half + 1.0,
                "the channel's edge {} past the plain's {half}",
                off + 3.0
            );
        } else {
            assert_eq!(r.vertices, before, "{s}");
            assert!(s["narrow_km"].as_f64().unwrap() > 0.0, "{s}");
        }
    }
}

#[test]
fn a_reach_steeper_than_the_braided_line_keeps_its_course() {
    let c = 16_384;
    let mut r = Rivers::empty(c);
    // 3,000 blocks falling 150: s = 150 × 35 / (3,000 × 300) = 0.0058, over 0.0125 × 100^-0.44 = 0.0016.
    line(
        &mut r,
        (-1500.0, 0.0),
        (1500.0, 0.0),
        2.0,
        5.0,
        100.0,
        (219.0, 69.0),
        Mouth::Sea,
        u32::MAX,
    );
    let before = r.vertices.clone();
    let w_at = |x: f64| 219.0 - 150.0 * (x + 1500.0) / 3000.0;
    let plain = move |x: f64, _: f64| Some(bank(w_at(x)));
    let s = run(&mut r, &plain);
    assert_eq!(r.vertices, before);
    assert!(
        s["steep_km"].as_f64().unwrap() > 0.8 * s["plain_km"].as_f64().unwrap(),
        "{s}"
    );
    // The same reach, gentle: it meanders.
    let mut r = Rivers::empty(c);
    line(
        &mut r,
        (-1500.0, 0.0),
        (1500.0, 0.0),
        2.0,
        5.0,
        100.0,
        (70.9, 69.1),
        Mouth::Sea,
        u32::MAX,
    );
    let w_at = |x: f64| 70.9 - 1.8 * (x + 1500.0) / 3000.0;
    let plain = move |x: f64, _: f64| Some(bank(w_at(x)));
    let s = run(&mut r, &plain);
    assert!(s["reaches"].as_u64().unwrap() >= 1, "{s}");
}

#[test]
fn the_belt_is_the_same_on_the_planet_at_the_equator_and_at_55_degrees() {
    let c = 65_536;
    let p = Projection::of(f64::from(c));
    let z55 = p.z_of_lat(55.0);
    let (ex, ez) = metric(f64::from(c), z55);
    let half = 900.0;
    let plain = |_: f64, _: f64| Some(bank(69.9));
    // (origin, direction, the far end, the vertex spacing; grid blocks): along the equator, along
    // the 55th parallel and along a meridian across 55°, each about 1,800 blocks of planet long
    // with a vertex every 2 blocks of planet.
    let cases = [
        ((-half, 0.0), (1.0, 0.0), (half, 0.0), 2.0),
        ((-half / ex, z55), (1.0, 0.0), (half / ex, z55), 2.0 / ex),
        (
            (0.0, z55 - half / ez),
            (0.0, 1.0),
            (0.0, z55 + half / ez),
            2.0 / ez,
        ),
    ];
    let mut got = Vec::new();
    for (a, dir, b, spacing) in cases {
        let mut r = Rivers::empty(c);
        line(
            &mut r,
            a,
            b,
            spacing,
            4.0,
            100.0,
            (69.9, 69.1),
            Mouth::Sea,
            u32::MAX,
        );
        let before = chain(&r, 0).to_vec();
        let s = run(&mut r, &plain);
        let v = chain(&r, 0);
        let (len, axis) = (
            planet_len(f64::from(c), v),
            planet_len(f64::from(c), &before),
        );
        let (off, crossings) = across(f64::from(c), v, a, dir);
        assert!(s["reaches"].as_u64().unwrap() >= 1, "{s}");
        got.push((len / axis, off, 2.0 * axis / crossings as f64, s));
    }
    let (k0, off0, l0, _) = &got[0];
    for (k, off, l, s) in &got[1..] {
        assert!(
            (k / k0 - 1.0).abs() < 0.02,
            "sinuosity {k} against {k0} at the equator: {s}"
        );
        assert!(
            (off / off0 - 1.0).abs() < 0.03,
            "offset {off} against {off0}: {s}"
        );
        assert!(
            (l / l0 - 1.0).abs() < 0.05,
            "wavelength {l} against {l0}: {s}"
        );
    }
    // In the grid's blocks the belt on the parallel is narrower north-south by cos 55°.
    assert!((ex - 55f64.to_radians().cos()).abs() < 1e-9);
}

#[test]
fn a_tributary_ends_on_a_vertex_of_its_parents_new_channel() {
    let c = 16_384;
    let mut r = Rivers::empty(c);
    let parent = line(
        &mut r,
        (-1500.0, 0.0),
        (1500.0, 0.0),
        2.0,
        5.0,
        200.0,
        (69.9, 69.1),
        Mouth::Sea,
        u32::MAX,
    );
    // From the north onto the parent's vertex at x 0 (vertex 750).
    let trib = line(
        &mut r,
        (0.0, -400.0),
        (0.0, 0.0),
        2.0,
        2.0,
        20.0,
        (69.95, 69.5),
        Mouth::River,
        parent,
    );
    let plain = |_: f64, _: f64| Some(bank(69.95));
    let s = run(&mut r, &plain);
    assert!(s["reaches"].as_u64().unwrap() >= 1, "{s}");
    let pv = chain(&r, parent as usize).to_vec();
    let tv = chain(&r, trib as usize);
    assert_eq!(r.chains[trib as usize].into, parent);
    let end = tv[tv.len() - 1];
    assert!(
        pv.iter().any(|q| q.x == end.x && q.z == end.z),
        "the tributary ends off its parent: {end:?} {s}"
    );
    // Up to its mouth the tributary stays out of the parent's channel.
    let dist = |p: &RiverVertex| {
        pv.windows(2)
            .map(|w| {
                let (ax, az, bx, bz) = (
                    f64::from(w[0].x),
                    f64::from(w[0].z),
                    f64::from(w[1].x),
                    f64::from(w[1].z),
                );
                let (px, pz) = (f64::from(p.x), f64::from(p.z));
                let (ex, ez) = (bx - ax, bz - az);
                let t = (((px - ax) * ex + (pz - az) * ez) / (ex * ex + ez * ez)).clamp(0.0, 1.0);
                (px - ax - t * ex).hypot(pz - az - t * ez)
            })
            .fold(f64::INFINITY, f64::min)
    };
    for p in &tv[..tv.len() - 1] {
        assert!(
            dist(p) > 4.0,
            "a tributary vertex {p:?} in the parent's channel"
        );
    }
    assert!(
        tv[0]
            == RiverVertex {
                x: 0.0,
                z: -400.0,
                ..tv[0]
            }
    );
}

#[test]
fn distributaries_and_chains_off_the_plain_stay_as_they_were() {
    let c = 16_384;
    let mut r = Rivers::empty(c);
    let stem = line(
        &mut r,
        (-1500.0, 0.0),
        (1500.0, 0.0),
        2.0,
        5.0,
        200.0,
        (69.9, 69.1),
        Mouth::Sea,
        u32::MAX,
    );
    // A distributary leaving the stem's vertex at x 600 (vertex 1050), and a river far away.
    let first = r.vertices.len() as u32;
    let apex = chain(&r, 0)[1050];
    let mut v = vec![apex];
    for i in 1..=100 {
        v.push(RiverVertex {
            x: apex.x + 2.0 * i as f32,
            z: 2.0 * i as f32,
            ..apex
        });
    }
    r.vertices.extend_from_slice(&v);
    let mut d = Chain::new(first, v.len() as u32, Mouth::Sea, u32::MAX);
    d.from = stem;
    r.chains.push(d);
    let far = line(
        &mut r,
        (-1500.0, 1500.0),
        (1500.0, 1500.0),
        2.0,
        5.0,
        200.0,
        (69.9, 69.1),
        Mouth::Sea,
        u32::MAX,
    );
    let dist_before = chain(&r, 1).to_vec();
    let far_before = chain(&r, far as usize).to_vec();
    let plain = |_: f64, z: f64| (z < 800.0).then_some(bank(69.9));
    let s = run(&mut r, &plain);
    assert!(s["reaches"].as_u64().unwrap() >= 1, "{s}");
    assert_eq!(chain(&r, 1), &dist_before[..]);
    assert_eq!(chain(&r, far as usize), &far_before[..]);
    assert_eq!(r.chains[1].from, stem);
    assert_eq!(r.chains[0].from, NO_CHAIN);
    // The stem still holds the apex.
    assert!(chain(&r, 0).contains(&apex));
}

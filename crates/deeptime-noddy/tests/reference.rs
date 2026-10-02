// SPDX-License-Identifier: LGPL-3.0-or-later
//! The port against the original C: `tests/data/reference.txt` holds what Noddy's own
//! `reverseEvents` and `taste` (Loop3D/noddyverse 6a21b23, compiled from `unEvents.c`, `taste.c`,
//! `utils.c`, `ellFault.c` and `proflib.c` by `tools/validation/noddy/`) did to random points
//! under random events and histories. The port must restore every point to the same place, stop
//! at the same events, write the same sequence codes and taste the same rock.

use deeptime_noddy::{
    Dyke, DykeKind, Event, Fault, FaultGeometry, FaultKind, Fold, History, Movement, Plug,
    PlugShape, Profile, Strain, Stratigraphy, Tilt, Unconformity, taste,
};

const REFERENCE: &str = include_str!("data/reference.txt");

/// Coordinates are a few thousand units and displacements tens to hundreds. On the machine that
/// wrote the fixture (x86_64 glibc) the port agrees with the C to the last bit (worst difference
/// 0). Another `libm` may differ in the last bit of a sine or cosine, and `rotset` takes `acos`
/// of values at or next to 1, where one bit turns a frame by about 1e-8 rad: 4e-5 units at 4,000
/// from the event. The tolerance leaves room for that and is still far below any real slip.
const TOLERANCE: f64 = 1e-3;

struct Point {
    input: [f64; 3],
    restored: [f64; 3],
    again: bool,
    sequence: [u8; 8],
    flavor: u8,
    index: usize,
}

struct Case {
    id: usize,
    kinds: Vec<String>,
    history: History,
    points: Vec<Point>,
}

fn numbers(fields: &[&str]) -> Vec<f64> {
    fields
        .iter()
        .map(|f| f.parse::<f64>().unwrap_or_else(|_| panic!("number {f}")))
        .collect()
}

fn event(fields: &[&str]) -> Event {
    let v = numbers(&fields[1..]);
    match fields[0] {
        "fold" => Event::Fold(Fold::new(
            [v[0], v[1], v[2]],
            v[3],
            v[4],
            v[5],
            v[6],
            v[7],
            v[8],
            v[9] != 0.0,
            Profile::sine_table(),
        )),
        "fault" => {
            let kind = [FaultKind::Fault, FaultKind::ShearZone, FaultKind::Kink][v[0] as usize];
            let geometry = [
                FaultGeometry::Translation,
                FaultGeometry::Rotation,
                FaultGeometry::Elliptical,
                FaultGeometry::Curved,
                FaultGeometry::Ring,
            ][v[1] as usize];
            let movement =
                [Movement::HangingWall, Movement::FootWall, Movement::Both][v[2] as usize];
            Event::Fault(
                Fault::new(
                    kind,
                    geometry,
                    movement,
                    [v[3], v[4], v[5]],
                    v[6],
                    v[7],
                    v[8],
                )
                .with_rotation(v[9])
                .with_slip(v[10])
                .with_radius(v[12])
                .with_width(v[13])
                .with_ellipsoid(v[14], v[15], v[16])
                .with_curve(v[11], v[17], v[18], Profile::sine_table()),
            )
        }
        "unc" => Event::Unconformity(Unconformity::new([v[0], v[1], v[2]], v[3], v[4])),
        "dyke" => Event::Dyke(Dyke::new(
            [DykeKind::Dilation, DykeKind::Stope][v[0] as usize],
            v[1] != 0.0,
            [v[2], v[3], v[4]],
            v[5],
            v[6],
            v[7],
            v[8],
            v[9],
        )),
        "plug" => {
            let shape = match v[0] as usize {
                0 => PlugShape::Cylinder { radius: v[8] },
                1 => PlugShape::Cone { apical_angle: v[9] },
                2 => PlugShape::Paraboloid { b: v[10] },
                _ => PlugShape::Ellipsoid {
                    a: v[11],
                    b: v[12],
                    c: v[13],
                },
            };
            Event::Plug(Plug::new(
                shape,
                v[1] != 0.0,
                [v[2], v[3], v[4]],
                v[5],
                v[6],
                v[7],
            ))
        }
        "tilt" => Event::Tilt(Tilt::new([v[0], v[1], v[2]], v[3], v[4], v[5])),
        "strain" => Event::Strain(Strain {
            inverse: [[v[0], v[1], v[2]], [v[3], v[4], v[5]], [v[6], v[7], v[8]]],
        }),
        other => panic!("unknown event {other}"),
    }
}

fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    let mut current: Option<Case> = None;
    for line in REFERENCE.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        match fields.first().copied() {
            None => {}
            Some(f) if f.starts_with('#') => {}
            Some("case") => {
                current = Some(Case {
                    id: fields[1].parse().unwrap(),
                    kinds: Vec::new(),
                    history: History::new(Stratigraphy::single()),
                    points: Vec::new(),
                });
            }
            Some("event") => {
                let c = current.as_mut().expect("event inside a case");
                c.kinds.push(fields[1..].join(" "));
                c.history.push(event(&fields[1..]));
            }
            Some("points") => {}
            Some("end") => out.push(current.take().expect("end inside a case")),
            Some(_) => {
                let v = numbers(&fields[..6]);
                let mut sequence = [0u8; 8];
                for (k, b) in sequence.iter_mut().enumerate() {
                    *b = u8::from_str_radix(&fields[7][2 * k..2 * k + 2], 16).unwrap();
                }
                current
                    .as_mut()
                    .expect("point inside a case")
                    .points
                    .push(Point {
                        input: [v[0], v[1], v[2]],
                        restored: [v[3], v[4], v[5]],
                        again: fields[6] == "1",
                        sequence,
                        flavor: fields[8].parse().unwrap(),
                        index: fields[9].parse().unwrap(),
                    });
            }
        }
    }
    out
}

#[test]
fn the_port_restores_points_as_the_original_does() {
    let cases = cases();
    assert!(cases.len() >= 100, "{} cases", cases.len());
    let mut points = 0usize;
    let mut moved = 0usize;
    let mut stopped = 0usize;
    let mut coded = 0usize;
    let mut worst = 0.0f64;
    let mut failures = Vec::new();
    for case in &cases {
        for (n, pt) in case.points.iter().enumerate() {
            points += 1;
            let r = case.history.restore(pt.input);
            let diff = (0..3)
                .map(|k| (r.point[k] - pt.restored[k]).abs())
                .fold(0.0f64, f64::max);
            let (flavor, index) = taste::taste(case.history.count(), &r.sequence);
            let same_codes = r.sequence.bytes()[..8] == pt.sequence;
            // NaN in the original (a degenerate random event) must be NaN here too.
            let same_place = diff <= TOLERANCE
                || (0..3).all(|k| {
                    (r.point[k].is_nan() && pt.restored[k].is_nan())
                        || (r.point[k] - pt.restored[k]).abs() <= TOLERANCE
                });
            if !same_place
                || !same_codes
                || r.stopped_at.is_none() != pt.again
                || flavor != pt.flavor
                || index != pt.index
            {
                if failures.len() < 12 {
                    failures.push(format!(
                        "case {} point {n} [{}]: in {:?}\n  C    {:?} again {} seq {:02x?} taste ({}, {})\n  Rust {:?} stopped {:?} seq {:02x?} taste ({flavor}, {index})",
                        case.id,
                        case.kinds.join(" | "),
                        pt.input,
                        pt.restored,
                        pt.again,
                        pt.sequence,
                        pt.flavor,
                        pt.index,
                        r.point,
                        r.stopped_at,
                        &r.sequence.bytes()[..8],
                    ));
                } else {
                    failures.push(String::new());
                }
                continue;
            }
            if diff.is_finite() {
                worst = worst.max(diff);
            }
            if (0..3).any(|k| (pt.restored[k] - pt.input[k]).abs() > 1e-9) {
                moved += 1;
            }
            if !pt.again {
                stopped += 1;
            }
            if pt.sequence.iter().any(|&b| b != 0) {
                coded += 1;
            }
        }
    }
    println!(
        "reference: {} cases, {points} points, {moved} moved, {stopped} stopped, {coded} coded, worst difference {worst:.3e}",
        cases.len()
    );
    assert!(
        failures.is_empty(),
        "{} of {points} points differ from the original:\n{}",
        failures.len(),
        failures
            .iter()
            .filter(|f| !f.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    // The fixture exercises what it claims to: most points move, some stop, many carry codes.
    assert!(moved * 2 > points, "{moved} of {points} moved");
    assert!(stopped * 20 > points, "{stopped} of {points} stopped");
    assert!(coded * 4 > points, "{coded} of {points} coded");
}

#[test]
fn every_kind_of_event_is_in_the_reference() {
    let cases = cases();
    let all: Vec<String> = cases.iter().flat_map(|c| c.kinds.iter().cloned()).collect();
    for kind in [
        "fold", "unc", "dyke 0", "dyke 1", "plug 0", "plug 1", "plug 2", "plug 3", "tilt", "strain",
    ] {
        assert!(
            all.iter().any(|k| k.starts_with(kind)),
            "no {kind} in the reference"
        );
    }
    for fault_kind in 0..3 {
        for geometry in 0..5 {
            // Shear zones and kinks have no elliptical or curved form in the harness.
            if fault_kind > 0 && (geometry == 2 || geometry == 3) {
                continue;
            }
            let prefix = format!("fault {fault_kind} {geometry} ");
            assert!(all.iter().any(|k| k.starts_with(&prefix)), "no {prefix}");
        }
    }
    assert!(cases.iter().any(|c| c.kinds.len() >= 5), "no long history");
}

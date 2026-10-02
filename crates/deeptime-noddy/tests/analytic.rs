// SPDX-License-Identifier: LGPL-3.0-or-later
//! Cases with known answers: a cylindrical fold of known wavelength and amplitude, a planar fault
//! of known slip, the order of composition, an angular unconformity, intrusions that cut across
//! the layering, a tilt. Each reads the model the way a geologist would: where is this horizon
//! now, and what rock is here.

use deeptime_noddy::{
    Dyke, DykeKind, Event, Fault, Fold, History, Movement, Plug, PlugShape, Profile, Rock,
    Stratigraphy, Tilt, Unconformity,
};

/// Layers 100 thick from −1000 to +1000.
fn layer_cake() -> Stratigraphy {
    let b: Vec<f64> = (-10..=10).map(|k| f64::from(k) * 100.0).collect();
    Stratigraphy::new(&b)
}

/// Index of the horizon z = 0 in [`layer_cake`]: layers 0..=10 are below it.
const BELOW_ZERO: usize = 10;

fn base_layer(h: &History, p: [f64; 3]) -> Option<usize> {
    match h.rock_at(p) {
        Rock::Layer { event: 0, layer } => Some(layer),
        _ => None,
    }
}

/// The height of the horizon that was z = 0 before any event, in the column at (x, y), by
/// bisection between `lo` and `hi` (the column must cross it once there).
fn horizon(h: &History, x: f64, y: f64, mut lo: f64, mut hi: f64) -> f64 {
    let below = |z: f64| base_layer(h, [x, y, z]).is_some_and(|l| l <= BELOW_ZERO);
    assert!(
        below(lo) && !below(hi),
        "the horizon is not between {lo} and {hi} at ({x}, {y})"
    );
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if below(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// An upright fold with a horizontal north–south axis through the origin.
fn upright_fold(wavelength: f64, amplitude: f64) -> Fold {
    Fold::new(
        [0.0, 0.0, 0.0],
        90.0,
        90.0,
        0.0,
        wavelength,
        amplitude,
        0.0,
        false,
        Profile::Sine,
    )
}

#[test]
fn a_cylindrical_fold_has_its_wavelength_and_amplitude() {
    let (wavelength, amplitude) = (800.0, 120.0);
    let h = History::new(layer_cake()).with(Event::Fold(upright_fold(wavelength, amplitude)));
    let mut top = f64::MIN;
    let mut bottom = f64::MAX;
    for i in 0..400 {
        let x = f64::from(i) * 4.0 - 800.0;
        let z = horizon(&h, x, 37.0, -300.0, 300.0);
        // The horizon is the sine itself: anticlines at λ/4 + nλ, synclines at 3λ/4 + nλ.
        let want = amplitude * (std::f64::consts::TAU * x / wavelength).sin();
        assert!((z - want).abs() < 1e-3, "x {x}: {z} vs {want}");
        top = top.max(z);
        bottom = bottom.min(z);
        // Cylindrical: the same anywhere along the axis.
        let z2 = horizon(&h, x, -5321.0, -300.0, 300.0);
        assert!((z - z2).abs() < 1e-6);
        // One wavelength on, the same height.
        let z3 = horizon(&h, x + wavelength, 37.0, -300.0, 300.0);
        assert!((z - z3).abs() < 1e-3);
    }
    assert!((top - amplitude).abs() < 0.1 && (bottom + amplitude).abs() < 0.1);
    // Simple shear along the axial plane: every bed keeps its thickness measured along it.
    for &x in &[0.0, 133.0, 200.0, 610.0] {
        let shift = amplitude * (std::f64::consts::TAU * x / wavelength).sin();
        for k in -5..5 {
            let z = f64::from(k) * 100.0 + 50.0 + shift;
            assert_eq!(
                base_layer(&h, [x, 0.0, z]),
                Some((k + 11) as usize),
                "x {x} k {k}"
            );
        }
    }
}

#[test]
fn a_fold_with_a_cycle_dies_out_along_its_axis() {
    let mut fold = upright_fold(800.0, 120.0);
    fold.cycle = 1000.0;
    let h = History::new(layer_cake()).with(Event::Fold(fold));
    let crest = |y: f64| horizon(&h, 200.0, y, -300.0, 300.0);
    assert!((crest(0.0) - 120.0).abs() < 1e-3);
    // Half the amplitude one cycle along the axis (0.693147 is the original's ln 2).
    assert!((crest(1000.0) - 60.0).abs() < 1e-3);
    assert!((crest(-1000.0) - 60.0).abs() < 1e-3);
    assert!(crest(5000.0).abs() < 1e-3);
}

#[test]
fn a_single_fold_is_one_wavelength() {
    let mut fold = upright_fold(800.0, 120.0);
    fold.single = true;
    let h = History::new(layer_cake()).with(Event::Fold(fold));
    assert!((horizon(&h, 200.0, 0.0, -300.0, 300.0) - 120.0).abs() < 1e-3);
    assert!(horizon(&h, -200.0, 0.0, -300.0, 300.0).abs() < 1e-6);
    assert!(horizon(&h, 1000.0, 0.0, -300.0, 300.0).abs() < 1e-6);
}

/// A fault through the origin striking north and dipping `dip`° to the east, pure dip-slip.
fn east_dipping_fault(dip: f64, slip: f64, movement: Movement) -> Fault {
    Fault::translation([0.0, 0.0, 0.0], 90.0, dip, 90.0, slip, movement)
}

#[test]
fn a_planar_fault_offsets_a_horizon_by_its_slip() {
    let (dip, slip) = (60.0f64, 200.0);
    let (throw, heave) = (slip * dip.to_radians().sin(), slip * dip.to_radians().cos());
    // Positive slip: the hanging wall goes down the dip (a normal fault).
    let h = History::new(layer_cake()).with(Event::Fault(east_dipping_fault(
        dip,
        slip,
        Movement::HangingWall,
    )));
    for &y in &[-900.0, 0.0, 444.0] {
        // Well into the foot wall (west) the horizon is where it was.
        assert!(horizon(&h, -2000.0, y, -500.0, 500.0).abs() < 1e-6);
        // Well into the hanging wall (east) it is down by the throw.
        assert!((horizon(&h, 2000.0, y, -500.0, 500.0) + throw).abs() < 1e-6);
    }
    // The hanging wall moved east by the heave: a marker bed's cut-off against the fault. The
    // horizon z = 0 meets the fault plane at x = 0 in the foot wall and, in the hanging wall, where
    // z = −throw on the plane: x = heave.
    let on_plane = |x: f64| -x * dip.to_radians().tan();
    assert!((on_plane(heave) + throw).abs() < 1e-9);
    // Just above the plane at that x the rock is the horizon's own two layers.
    let above = base_layer(&h, [heave + 1.0, 0.0, on_plane(heave + 1.0) + 2.0]).unwrap();
    assert!(above == BELOW_ZERO || above == BELOW_ZERO + 1);

    // Negative slip: a reverse fault, the hanging wall up by the throw: older rock over younger.
    let h = History::new(layer_cake()).with(Event::Fault(east_dipping_fault(
        dip,
        -slip,
        Movement::HangingWall,
    )));
    assert!((horizon(&h, 2000.0, 0.0, -500.0, 500.0) - throw).abs() < 1e-6);
    // A vertical hole through the fault meets the same bed twice: a repeated section.
    let x = 100.0;
    let z_fault = on_plane(x);
    let hw = base_layer(&h, [x, 0.0, z_fault + 1.0]).unwrap();
    let fw = base_layer(&h, [x, 0.0, z_fault - 1.0]).unwrap();
    assert!(
        hw < fw,
        "hanging wall {hw} must be older (lower) than foot wall {fw}"
    );

    // Both walls moving: each by half.
    let h = History::new(layer_cake()).with(Event::Fault(east_dipping_fault(
        dip,
        slip,
        Movement::Both,
    )));
    assert!((horizon(&h, -2000.0, 0.0, -500.0, 500.0) - throw / 2.0).abs() < 1e-6);
    assert!((horizon(&h, 2000.0, 0.0, -500.0, 500.0) + throw / 2.0).abs() < 1e-6);

    // Only the foot wall moving: up the dip by the whole slip.
    let h = History::new(layer_cake()).with(Event::Fault(east_dipping_fault(
        dip,
        slip,
        Movement::FootWall,
    )));
    assert!((horizon(&h, -2000.0, 0.0, -500.0, 500.0) - throw).abs() < 1e-6);
    assert!(horizon(&h, 2000.0, 0.0, -500.0, 500.0).abs() < 1e-6);
}

#[test]
fn a_strike_slip_fault_moves_rock_sideways_only() {
    // Pitch 0: slip along the strike. A flat horizon shows nothing …
    let fault = Fault::translation(
        [0.0, 0.0, 0.0],
        90.0,
        80.0,
        0.0,
        300.0,
        Movement::HangingWall,
    );
    let h = History::new(layer_cake()).with(Event::Fault(fault.clone()));
    assert!(horizon(&h, 2000.0, 0.0, -500.0, 500.0).abs() < 1e-6);
    // … but a fold older than the fault, with its axis across the fault, is offset by the slip.
    let fold = Fold::new(
        [0.0; 3],
        0.0,
        90.0,
        0.0,
        800.0,
        120.0,
        0.0,
        false,
        Profile::Sine,
    );
    let h = History::new(layer_cake())
        .with(Event::Fold(fold))
        .with(Event::Fault(fault));
    let profile = |x: f64, y: f64| horizon(&h, x, y, -400.0, 400.0);
    let mut worst = 0.0f64;
    for i in 0..80 {
        let y = f64::from(i) * 10.0;
        // East of the fault the fold's shape is the foot wall's, 300 along the strike.
        let a = profile(-2000.0, y);
        let ahead = profile(2000.0, y + 300.0);
        let behind = profile(2000.0, y - 300.0);
        worst = worst.max((a - ahead).abs().min((a - behind).abs()));
    }
    assert!(worst < 1e-6, "{worst}");
}

#[test]
fn events_compose_in_order() {
    let fold = upright_fold(800.0, 120.0);
    let fault = east_dipping_fault(45.0, 300.0, Movement::HangingWall);
    let fold_then_fault = History::new(layer_cake())
        .with(Event::Fold(fold.clone()))
        .with(Event::Fault(fault.clone()));
    let fault_then_fold = History::new(layer_cake())
        .with(Event::Fault(fault.clone()))
        .with(Event::Fold(fold.clone()));
    // Folded then faulted: the hanging wall carries the fold with it, shifted by the heave and
    // dropped by the throw.
    let (throw, heave) = (
        300.0 * 45.0f64.to_radians().sin(),
        300.0 * 45.0f64.to_radians().cos(),
    );
    for i in 0..50 {
        let x = 1500.0 + f64::from(i) * 16.0;
        let z = horizon(&fold_then_fault, x, 0.0, -700.0, 400.0);
        let want = 120.0 * (std::f64::consts::TAU * (x - heave) / 800.0).sin() - throw;
        assert!((z - want).abs() < 1e-3, "x {x}: {z} vs {want}");
        // Faulted then folded: the fold is not offset, only the step is folded.
        let z = horizon(&fault_then_fold, x, 0.0, -700.0, 400.0);
        let want = 120.0 * (std::f64::consts::TAU * x / 800.0).sin() - throw;
        assert!((z - want).abs() < 1e-3, "x {x}: {z} vs {want}");
    }
    // And the fault itself: straight when it is the younger event, folded when it is the older.
    let side = |h: &History, p: [f64; 3]| h.restore(p).sequence.get(h.events().len());
    assert_ne!(
        side(&fold_then_fault, [100.0, 0.0, -90.0]),
        side(&fold_then_fault, [100.0, 0.0, -110.0]),
        "a 45° plane through the origin passes z = −100 at x = 100"
    );
    // The two orders give different models.
    let differ = (0..200).any(|i| {
        let p = [
            f64::from(i) * 13.0 - 1300.0,
            0.0,
            f64::from(i % 17) * 40.0 - 340.0,
        ];
        fold_then_fault.rock_at(p) != fault_then_fold.rock_at(p)
    });
    assert!(differ);
    // Restoring is undoing the events one at a time, youngest first.
    let p = [321.0, -45.0, 67.0];
    let mut seq = deeptime_noddy::Sequence::new();
    let step1 = match fault.undo(p, 2, &mut seq) {
        deeptime_noddy::events::Step::Moved(q) => q,
        _ => unreachable!(),
    };
    let step2 = match fold.undo(step1) {
        deeptime_noddy::events::Step::Moved(q) => q,
        _ => unreachable!(),
    };
    assert_eq!(fold_then_fault.restore(p).point, step2);
}

#[test]
fn an_unconformity_cuts_folds_and_carries_flat_cover() {
    // Fold, erode flat at z = 50, lay three flat layers on top.
    let mut h = History::new(layer_cake()).with(Event::Fold(upright_fold(800.0, 120.0)));
    let unc = h.push_unconformity(
        Unconformity::new([0.0, 0.0, 50.0], 0.0, 0.0),
        Stratigraphy::new(&[150.0, 250.0]),
    );
    // Above the surface: cover, flat, whatever the fold does below.
    for &x in &[0.0, 200.0, 600.0, 777.0] {
        assert_eq!(
            h.rock_at([x, 0.0, 60.0]),
            Rock::Layer {
                event: unc,
                layer: 0
            }
        );
        assert_eq!(
            h.rock_at([x, 0.0, 200.0]),
            Rock::Layer {
                event: unc,
                layer: 1
            }
        );
        assert_eq!(
            h.rock_at([x, 0.0, 900.0]),
            Rock::Layer {
                event: unc,
                layer: 2
            }
        );
    }
    // Below it: the folded sequence, truncated. Under the anticline (x = 200) the surface cuts
    // older rock than under the syncline (x = 600): an angular unconformity.
    let under_anticline = base_layer(&h, [200.0, 0.0, 49.0]).unwrap();
    let under_syncline = base_layer(&h, [600.0, 0.0, 49.0]).unwrap();
    assert!(
        under_anticline < under_syncline,
        "{under_anticline} {under_syncline}"
    );
    assert_eq!(h.restore([0.0, 0.0, 60.0]).stopped_at, Some(unc));
    assert_eq!(h.restore([0.0, 0.0, 40.0]).stopped_at, None);

    // A later fault moves cover and basement together.
    h.push(Event::Fault(east_dipping_fault(
        60.0,
        200.0,
        Movement::HangingWall,
    )));
    let throw = 200.0 * 60.0f64.to_radians().sin();
    assert_eq!(
        h.rock_at([3000.0, 0.0, 60.0 - throw]),
        Rock::Layer {
            event: unc,
            layer: 0
        }
    );
    assert!(matches!(
        h.rock_at([3000.0, 0.0, 40.0 - throw]),
        Rock::Layer { event: 0, .. }
    ));
}

#[test]
fn intrusions_cut_across_the_layering() {
    // A vertical cylinder of radius 300 through folded layers: igneous at every height inside,
    // the folded layers outside, whatever the layer.
    let mut h = History::new(layer_cake()).with(Event::Fold(upright_fold(800.0, 120.0)));
    let plug = h.push(Event::Plug(Plug::new(
        PlugShape::Cylinder { radius: 300.0 },
        false,
        [1000.0, 0.0, 0.0],
        0.0,
        90.0,
        0.0,
    )));
    let mut layers_cut = std::collections::BTreeSet::new();
    for k in -9..9 {
        let z = f64::from(k) * 100.0 + 50.0;
        assert_eq!(h.rock_at([1000.0, 0.0, z]), Rock::Igneous { event: plug });
        assert_eq!(h.rock_at([1250.0, 100.0, z]), Rock::Igneous { event: plug });
        layers_cut.insert(base_layer(&h, [1000.0 + 301.0, 0.0, z]).unwrap());
    }
    assert!(
        layers_cut.len() >= 15,
        "the contact crosses {} layers",
        layers_cut.len()
    );

    // A dilation dyke 50 wide, vertical, striking north at x = 0: its walls match once closed.
    let mut h = History::new(layer_cake()).with(Event::Fold(Fold::new(
        [0.0; 3],
        0.0,
        90.0,
        0.0,
        800.0,
        120.0,
        0.0,
        false,
        Profile::Sine,
    )));
    let dyke = h.push(Event::Dyke(Dyke::new(
        DykeKind::Dilation,
        false,
        [0.0, 0.0, 0.0],
        90.0,
        90.0,
        0.0,
        0.0,
        50.0,
    )));
    let inside: Vec<f64> = (-200..200)
        .map(|i| f64::from(i) * 0.5)
        .filter(|&x| h.rock_at([x, 10.0, 10.0]) == Rock::Igneous { event: dyke })
        .collect();
    let width = inside.last().unwrap() - inside.first().unwrap();
    assert!((width - 50.0).abs() <= 1.0, "dyke width {width}");
    let (west, east) = (inside.first().unwrap() - 1.0, inside.last().unwrap() + 1.0);
    for i in 0..60 {
        let (y, z) = (f64::from(i) * 23.0 - 700.0, f64::from(i % 9) * 55.0 - 200.0);
        assert_eq!(
            base_layer(&h, [west, y, z]),
            base_layer(&h, [east, y, z]),
            "the walls of a dilation dyke carry the same rock"
        );
    }
    // A stope dyke replaces rock: nothing moves.
    let mut h = History::new(layer_cake());
    h.push(Event::Dyke(Dyke::new(
        DykeKind::Stope,
        false,
        [0.0, 0.0, 0.0],
        90.0,
        90.0,
        0.0,
        0.0,
        50.0,
    )));
    for &x in &[-500.0, -1.0, 80.0, 900.0] {
        let r = h.restore([x, 3.0, 4.0]);
        if r.stopped_at.is_none() {
            assert_eq!(r.point, [x, 3.0, 4.0]);
        }
    }
}

#[test]
fn merged_intrusions_share_one_rock() {
    let mut h = History::new(layer_cake());
    let first = h.push(Event::Plug(Plug::new(
        PlugShape::Cylinder { radius: 200.0 },
        false,
        [0.0, 0.0, 0.0],
        0.0,
        90.0,
        0.0,
    )));
    h.push(Event::Plug(Plug::new(
        PlugShape::Cylinder { radius: 200.0 },
        true,
        [300.0, 0.0, 0.0],
        0.0,
        90.0,
        0.0,
    )));
    assert_eq!(h.rock_at([0.0, 0.0, 0.0]), Rock::Igneous { event: first });
    assert_eq!(h.rock_at([400.0, 0.0, 0.0]), Rock::Igneous { event: first });
}

#[test]
fn a_tilt_dips_the_beds_by_its_angle() {
    // Rotate 20° about a horizontal north-pointing axis through the origin.
    let h = History::new(layer_cake()).with(Event::Tilt(Tilt::new([0.0; 3], 20.0, 0.0, 0.0)));
    let z1 = horizon(&h, 0.0, 0.0, -100.0, 100.0);
    let z2 = horizon(&h, 500.0, 0.0, -600.0, 600.0);
    let z3 = horizon(&h, 500.0, 900.0, -600.0, 600.0);
    assert!(z1.abs() < 1e-6);
    let dip = ((z2 - z1) / 500.0).atan().to_degrees().abs();
    // load_tilts turns degrees into radians with 0.0174532.
    assert!((dip - 20.0).abs() < 1e-3, "{dip}");
    assert!((z2 - z3).abs() < 1e-6, "the strike is along the axis");
}

#[test]
fn evaluation_is_deterministic() {
    let mut h = History::new(layer_cake())
        .with(Event::Fold(upright_fold(800.0, 120.0)))
        .with(Event::Fault(east_dipping_fault(
            60.0,
            200.0,
            Movement::Both,
        )));
    h.push_unconformity(
        Unconformity::new([0.0, 0.0, 300.0], 30.0, 5.0),
        Stratigraphy::new(&[400.0]),
    );
    h.push(Event::Plug(Plug::new(
        PlugShape::Ellipsoid {
            a: 400.0,
            b: 300.0,
            c: 500.0,
        },
        false,
        [-500.0, 200.0, -100.0],
        40.0,
        70.0,
        15.0,
    )));
    let a = h.block([-1000.0, -1000.0, -500.0], [24, 20, 16], [80.0, 90.0, 60.0]);
    let b = h.block([-1000.0, -1000.0, -500.0], [24, 20, 16], [80.0, 90.0, 60.0]);
    assert_eq!(a, b);
    assert_eq!(a.len(), 24 * 20 * 16);
    let kinds: std::collections::BTreeSet<String> = a.iter().map(|r| format!("{r:?}")).collect();
    assert!(kinds.len() > 8, "{} rocks in the block", kinds.len());
    // The lattice order: x fastest.
    assert_eq!(a[1], h.rock_at([-920.0, -1000.0, -500.0]));
    assert_eq!(a[24], h.rock_at([-1000.0, -910.0, -500.0]));
    assert_eq!(a[24 * 20], h.rock_at([-1000.0, -1000.0, -440.0]));
}

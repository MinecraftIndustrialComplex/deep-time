// SPDX-License-Identifier: LGPL-3.0-or-later
//! The structures as a whole: what the translation makes is the same thing to the Noddy port
//! (the reference kinematics) and to the world data's column rule (what the game runs).

use deeptime_format::GridDesc;
use deeptime_format::structure::{Shape, StructIndex};
use deeptime_noddy as noddy;

use super::emplace::tests::Planet;
use super::feature::{Feature, Kind};
use super::*;
use crate::deposits::random::Rng;
use crate::structure::params::StructureParams;

/// A key for a test structure's age: older is larger, as the bake's keys are.
fn test_key(f: &Feature) -> u16 {
    (4096.0 - 20.0 * f64::from(f.t_myr)).clamp(4096.0, 32767.0) as u16
}

/// Random structures of every kind scattered over a patch of a 4,096-block planet, some across
/// the seam.
fn scatter(n: usize, seed: u64) -> Vec<Feature> {
    let mut rng = Rng::stream(seed, "structure/deform/tests");
    let mut out = Vec::new();
    for id in 0..n {
        let kind = Kind::ALL[rng.below(4) as usize];
        let length = 80.0 + 300.0 * rng.uniform();
        let wavelength = 30.0 + 90.0 * rng.uniform();
        let mut f = Feature {
            id: id as u32,
            kind,
            x: if id % 5 == 0 {
                2048.0 - 600.0 * rng.uniform() - 1e-9
            } else {
                -2048.0 + 900.0 * rng.uniform()
            },
            z: -300.0 + 600.0 * rng.uniform(),
            plate: 0,
            t_myr: -(300.0 * rng.uniform()) as f32,
            azimuth_deg: (360.0 * rng.uniform()) as f32,
            half_along: 0.0,
            half_across: 0.0,
            wavelength: 0.0,
            amplitude: 0.0,
            phase: 0.0,
            dip_deg: 0.0,
            slip: 0.0,
            cause: 0,
            adrift: 0,
            wear_m: 0.0,
            rise: 0.0,
            depth: 0.0,
            base: 0.0,
            rock: 0,
        };
        match kind {
            Kind::FoldTrain => {
                f.wavelength = wavelength as f32;
                f.amplitude = (wavelength * (0.05 + 0.12 * rng.uniform())) as f32;
                f.half_across = (wavelength * (1.0 + 1.5 * rng.uniform())) as f32;
                f.half_along = (wavelength * (2.5 + 3.5 * rng.uniform())) as f32;
                f.phase = rng.uniform() as f32;
            }
            Kind::Thrust => {
                f.dip_deg = (20.0 + 20.0 * rng.uniform()) as f32;
                f.slip = -(0.08 * length) as f32;
                f.half_along = (0.5 * length) as f32;
                f.half_across = (0.4 * length) as f32;
            }
            Kind::Normal => {
                f.dip_deg = (55.0 + 15.0 * rng.uniform()) as f32;
                f.slip = (0.05 * length) as f32;
                f.half_along = (0.5 * length) as f32;
                f.half_across = (0.4 * length) as f32;
            }
            Kind::StrikeSlip => {
                f.dip_deg = 90.0;
                f.slip = (if rng.uniform() < 0.5 { -0.05 } else { 0.05 } * length) as f32;
                f.half_along = (0.5 * length) as f32;
                f.half_across = (0.4 * length) as f32;
            }
            Kind::Stock | Kind::Dyke | Kind::Plug => unreachable!("the first four kinds"),
        }
        out.push(f);
    }
    out
}

#[test]
fn the_column_rule_is_the_noddy_port() {
    let mut features = scatter(120, 11);
    undo_order(&mut features);
    for w in features.windows(2) {
        assert!(w[0].t_myr >= w[1].t_myr, "youngest first");
    }
    // Faults' traces at different heights, as the ground would put them.
    let st = finish(
        &features,
        |x, z| 60.0 + 0.02 * x - 0.03 * z,
        test_key,
        -64.0,
    );
    assert_eq!(st.events.len(), features.len());
    assert_eq!(st.profiles.len(), 1);
    let index = StructIndex::new(&st, 4096);
    // Columns on a lattice over the patch, the seam included.
    let mut sample = Vec::new();
    for j in 0..40 {
        for i in 0..70 {
            sample.push((
                -2048.0 + 21.3 * f64::from(i) - 400.0,
                -420.0 + 21.7 * f64::from(j),
            ));
        }
    }
    let check = lattice::noddy_check(&features, &st, &index, &sample, -352.0, 640.0, 7.0, 1e-6);
    println!(
        "noddy_check: {} columns ({} with events), {} points, {:.2} events per point, worst {:.3e}, \
         side flips {}, {:.0} ns/point Noddy, {:.0} ns/point columns",
        check.columns,
        check.columns_with_events,
        check.points,
        check.events_per_point,
        check.worst,
        check.side_flips,
        check.ns_per_point_noddy(),
        check.ns_per_point_columns()
    );
    assert!(
        check.columns_with_events * 2 > check.columns,
        "the patch is deformed"
    );
    assert!(check.points > 100_000);
    assert!(check.events_per_point > 2.0, "{}", check.events_per_point);
    // The same restored position, to well under a thousandth of a block.
    assert!(check.worst < 1e-5, "worst {}", check.worst);
    assert!(
        check.side_flips * 100_000 < check.points,
        "{} side flips",
        check.side_flips
    );
}

#[test]
fn each_kind_moves_rock_the_way_its_name_says() {
    let base = |kind: Kind| Feature {
        id: 0,
        kind,
        x: 0.0,
        z: 0.0,
        plate: 0,
        t_myr: -10.0,
        azimuth_deg: 0.0,
        half_along: 200.0,
        half_across: 100.0,
        wavelength: 50.0,
        amplitude: 6.0,
        phase: 0.0,
        dip_deg: 60.0,
        slip: 10.0,
        cause: 0,
        adrift: 0,
        wear_m: 0.0,
        rise: 0.0,
        depth: 0.0,
        base: 0.0,
        rock: 0,
    };
    let column = |f: &Feature, x: f64, z: f64| {
        let st = finish(std::slice::from_ref(f), |_, _| 64.0, test_key, -64.0);
        let mut segs = Vec::new();
        st.column(&[0], x, z, 4096.0, &mut segs);
        segs
    };
    // A fold train along north: crests and troughs alternate eastwards, a quarter and three
    // quarters of a wavelength from the centre.
    let f = base(Kind::FoldTrain);
    let crest = column(&f, 12.5, 0.0);
    let trough = column(&f, 37.5, 0.0);
    assert_eq!(crest.len(), 1);
    // A crest: the rock came from below (the source height is lower).
    assert!(
        crest[0].dy < -4.0 && trough[0].dy > 2.0,
        "{} {}",
        crest[0].dy,
        trough[0].dy
    );
    assert!(column(&f, 0.0, 0.0)[0].dy.abs() < 0.2);
    // A normal fault striking north dips east (azimuth + 90°): east of the trace the hanging
    // wall has gone down, so its rock came from above.
    let f = base(Kind::Normal);
    let segs = column(&f, 20.0, 0.0);
    assert_eq!(segs.len(), 2);
    assert!(segs[1].dy > 5.0, "{}", segs[1].dy);
    assert!(
        segs[1].x < 20.0,
        "and from nearer the trace: it moved down the dip"
    );
    assert!((segs[0].y_hi - (64.0 - 20.0 * 60.0f64.to_radians().tan())).abs() < 1e-6);
    // A thrust: the hanging wall has gone up and over: its rock came from below and behind.
    let mut f = base(Kind::Thrust);
    f.dip_deg = 30.0;
    f.slip = -10.0;
    let segs = column(&f, 20.0, 0.0);
    assert_eq!(segs.len(), 2);
    assert!(segs[1].dy < -3.0 && segs[1].x > 20.0, "{:?}", segs[1]);
    // A strike-slip fault: sideways along the strike only.
    let mut f = base(Kind::StrikeSlip);
    f.dip_deg = 90.0;
    let segs = column(&f, 20.0, 0.0);
    let hw = segs.last().unwrap();
    assert!(
        hw.dy.abs() < 1e-3 && (hw.x - 20.0).abs() < 1e-3 && (hw.z - 0.0).abs() > 5.0,
        "{hw:?}"
    );
    // Nothing is baked wrongly: poles point up, folds' phases are never negative in the window.
    for f in scatter(60, 5) {
        match f.to_baked(64.0, 5000, -64.0).shape {
            Shape::Fault { pole, .. } => {
                assert!(pole[1] > 0.0);
                let len = (pole[0] * pole[0] + pole[1] * pole[1] + pole[2] * pole[2]).sqrt();
                assert!((len - 1.0).abs() < 1e-9);
            }
            Shape::Fold { offset, .. } => assert!(offset <= -f64::from(f.half_across)),
            Shape::Intrusion { .. } => unreachable!("no intrusions in the scatter"),
        }
    }
}

#[test]
fn a_planet_of_structures_evaluates_on_a_lattice() {
    let p = Planet::new();
    let mut sp = StructureParams::default();
    sp.deformation.density = 8.0;
    let (mut features, counts) = p.run(&sp, 20, 21);
    assert!(counts.fold_trains > 0 && counts.thrusts > 0 && counts.normal_faults > 0);
    undo_order(&mut features);
    let mut st = finish(&features, |_, _| 70.0, test_key, -64.0);
    let l2 = GridDesc::for_planet("L2", 4096, 8).unwrap();
    // The margin holds the furthest sideways shift, with room to spare.
    let settled = settle_margin(&mut st, 4096, &l2, -352.0, 640.0);
    assert!(
        settled.max_heave * 1.25 <= st.margin,
        "{} vs {}",
        settled.max_heave,
        st.margin
    );
    assert!(st.margin >= deeptime_format::structure::REACH_MARGIN);
    let index = StructIndex::new(&st, 4096);
    let stats = lattice::columns(&st, &index, &l2, -352.0, 640.0);
    assert_eq!(stats.columns, l2.cells() as u64);
    // The belt and the rift are deformed, the craton between them is not: a minority of the
    // planet's columns, and every deformed one inside a structure's reach.
    assert!(
        stats.deformed > 1000 && stats.deformed * 4 < stats.columns,
        "{stats:?}"
    );
    assert!(stats.faulted > 100 && stats.faulted <= stats.deformed);
    assert!(stats.max_candidates >= 2 && stats.max_segments >= 2);
    // (Bounds at 1:1, times the vertical scale that heights are drawn at.)
    let x = sp.deformation.block_stretch(300.0, p.m_per_block_v);
    assert!(
        stats.max_shift > 2.0 && stats.max_shift < 80.0 * x,
        "{}",
        stats.max_shift
    );
    // The craton column between the belt and the rift: no candidates at all.
    let mut cand = Vec::new();
    index.candidates(
        &st,
        p.grid.block_x(140) as f64,
        p.grid.block_z(60) as f64,
        &mut cand,
    );
    assert!(cand.is_empty());
    // The answer does not depend on the thread count.
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    let mut one = pool.install(|| lattice::columns(&st, &index, &l2, -352.0, 640.0));
    one.seconds = stats.seconds;
    assert_eq!(one, stats);
    // And the Noddy port agrees on a sample of it.
    let sample: Vec<(f64, f64)> = (0..2000)
        .map(|k| {
            (
                p.grid.block_x(50 + (k % 140)) as f64 + 3.0,
                p.grid.block_z(42 + (k / 140) * 11) as f64 + 5.0,
            )
        })
        .collect();
    let check = lattice::noddy_check(&features, &st, &index, &sample, -352.0, 640.0, 16.0, 1e-6);
    // (Rounding only: at the default vertical scale faults dip 80° to 85° in blocks, and the two
    // frames' rounding differs by up to about 1 / cos(dip) times more than at 1:1.)
    assert!(check.worst < 1e-4, "{check:?}");
    // The history as a Noddy event list reads like any other.
    let profile = noddy::Profile::sine_table();
    let e = features[0].to_noddy(0.0, 0.0, 4096.0, 70.0, &profile);
    assert!(matches!(e, Some(noddy::Event::Windowed(_))));
}

#[test]
fn structures_are_grouped_by_what_made_them() {
    let mut features = scatter(120, 9);
    // Three causes and some structures with none (grouped by their plate).
    for (k, f) in features.iter_mut().enumerate() {
        f.cause = [0, 11, 12, 13][k % 4];
        f.plate = (k % 3) as u32;
    }
    let groups = groups(&features, 4096.0, 300.0, 20.0);
    // Every structure is in exactly one group of its own kind of setting.
    let count = |g: &Group| -> f64 {
        g.params
            .iter()
            .filter(|(k, _)| ["fold_trains", "thrusts", "stocks", "faults", "dykes"].contains(k))
            .map(|(_, v)| *v)
            .sum()
    };
    assert_eq!(
        groups.iter().map(count).sum::<f64>() as usize,
        features.len()
    );
    for g in &groups {
        assert!(g.kind == "deeptime:fold_belt" || g.kind == "deeptime:fault_system");
        assert_eq!(g.kind == "deeptime:fold_belt", g.setting == "orogen");
        assert!(g.t_start <= g.t_end && g.magnitude > 0.0 && g.magnitude <= 1.0);
        assert!((-90.0..=90.0).contains(&g.lat) && (0.0..360.0).contains(&g.lon));
        // Ungrouped ones say which plate; every group says where its strongest structure is.
        let has = |k: &str| g.params.iter().any(|(p, _)| *p == k);
        assert_eq!(g.cause == 0, has("plate"));
        assert!(has("strongest_x") && has("strongest_z") && has("strongest_azimuth_deg"));
        let x = g
            .params
            .iter()
            .find(|(p, _)| *p == "strongest_x")
            .unwrap()
            .1;
        assert!(
            features.iter().any(|f| f.x.round() == x),
            "a member is the strongest"
        );
    }
    // The same structures, the same groups.
    assert_eq!(groups, super::groups(&features, 4096.0, 300.0, 20.0));
    // Plugs belong to their volcanoes: no group.
    let mut plug = features[0].clone();
    plug.kind = Kind::Plug;
    assert!(super::groups(&[plug], 4096.0, 300.0, 20.0).is_empty());
}

/// The densest case: hundreds of folds and faults over one place (a belt that stayed active for
/// the whole history). One Noddy history holds 127 events; the check undoes such a column as a
/// chain of histories and still agrees with the column rule, and the column rule itself has no
/// limit.
#[test]
fn a_place_with_more_structures_than_one_noddy_history_holds() {
    let mut rng = Rng::stream(77, "structure/deform/tests/dense");
    let mut features: Vec<Feature> = (0..420u32)
        .map(|id| {
            let fold = id % 3 == 0;
            let length = 90.0 + 120.0 * rng.uniform();
            Feature {
                id,
                kind: if fold {
                    Kind::FoldTrain
                } else if id % 3 == 1 {
                    Kind::Thrust
                } else {
                    Kind::Normal
                },
                x: -30.0 + 60.0 * rng.uniform(),
                z: -30.0 + 60.0 * rng.uniform(),
                plate: 0,
                t_myr: -(900.0 * rng.uniform()) as f32,
                azimuth_deg: (360.0 * rng.uniform()) as f32,
                half_along: if fold { 260.0 } else { (0.5 * length) as f32 },
                half_across: if fold { 120.0 } else { (0.4 * length) as f32 },
                wavelength: 60.0,
                amplitude: if fold { 2.0 } else { 0.0 },
                phase: rng.uniform() as f32,
                dip_deg: if id % 3 == 1 { 30.0 } else { 60.0 },
                slip: if fold {
                    0.0
                } else if id % 3 == 1 {
                    -1.5
                } else {
                    1.5
                },
                cause: 0,
                adrift: 0,
                wear_m: 0.0,
                rise: 0.0,
                depth: 0.0,
                base: 0.0,
                rock: 0,
            }
        })
        .collect();
    undo_order(&mut features);
    let st = finish(&features, |_, _| 70.0, test_key, -64.0);
    let index = StructIndex::new(&st, 4096);
    let sample: Vec<(f64, f64)> = (0..25)
        .map(|k| (f64::from(k % 5) * 9.0 - 18.0, f64::from(k / 5) * 9.0 - 18.0))
        .collect();
    let check = lattice::noddy_check(&features, &st, &index, &sample, -352.0, 640.0, 16.0, 1e-6);
    assert_eq!(check.columns_with_events, 25);
    assert_eq!(check.max_history, 420, "every structure reaches the middle");
    assert_eq!(check.long_histories, 25);
    assert!(check.worst < 1e-4, "{check:?}");
    assert!(check.side_flips * 1000 < check.points, "{check:?}");
    // The column rule on the same columns: every structure a candidate, the column in pieces,
    // each piece's rock from somewhere finite.
    let (mut cand, mut segs) = (Vec::new(), Vec::new());
    for &(x, z) in &sample {
        index.candidates(&st, x, z, &mut cand);
        assert_eq!(cand.len(), 420);
        st.column(&cand, x, z, 4096.0, &mut segs);
        assert!(
            segs.len() > 1 && segs.len() < 2000,
            "{} segments",
            segs.len()
        );
        assert!(segs.windows(2).all(|w| w[0].y_hi == w[1].y_lo));
        assert!(
            segs.iter()
                .all(|g| g.dy.is_finite() && g.x.is_finite() && g.z.is_finite())
        );
    }
}

/// The column rule (what the strata pass, far terrain, inspect and sections all read) offsets the
/// beds by the stretch: a normal fault drawn 4 times taller restores the hanging wall's
/// rock from 4 times as far up and from as far to the side, and a fold 4 times taller shifts the
/// column 4 times as far.
#[test]
fn the_column_rule_offsets_beds_by_the_stretch() {
    let fault = |x: f64| {
        let (dip, slip) = super::emplace::stretch(60.0, 10.0, x);
        Feature {
            id: 0,
            kind: Kind::Normal,
            x: 0.0,
            z: 0.0,
            plate: 0,
            t_myr: -10.0,
            azimuth_deg: 0.0,
            half_along: 200.0,
            half_across: 100.0,
            wavelength: 50.0,
            amplitude: 6.0 * x as f32,
            phase: 0.0,
            dip_deg: dip as f32,
            slip: slip as f32,
            cause: 0,
            adrift: 0,
            wear_m: 0.0,
            rise: 0.0,
            depth: 0.0,
            base: 0.0,
            rock: 0,
        }
    };
    let column = |f: &Feature, x: f64| {
        let st = finish(std::slice::from_ref(f), |_, _| 64.0, test_key, -64.0);
        let mut segs = Vec::new();
        st.column(&[0], x, 0.0, 4096.0, &mut segs);
        segs
    };
    // East of the trace, past where either plane reaches, in the hanging wall.
    let (one, four) = (fault(1.0), fault(4.0));
    let (a, b) = (column(&one, 30.0), column(&four, 30.0));
    let (ha, hb) = (a.last().unwrap(), b.last().unwrap());
    assert!(ha.dy > 1.0, "{ha:?}");
    assert!(
        (hb.dy - 4.0 * ha.dy).abs() < 1e-3 * ha.dy,
        "{} vs 4 x {}",
        hb.dy,
        ha.dy
    );
    assert!(
        (hb.x - ha.x).abs() < 1e-3,
        "the heave is kept: {} vs {}",
        hb.x,
        ha.x
    );
    // The steeper plane (atan(4 tan 60°) = 81.8°) lies deeper under this column.
    assert!(b[0].y_hi < a[0].y_hi, "{} vs {}", b[0].y_hi, a[0].y_hi);
    // A fold.
    let fold = |x: f64| Feature {
        kind: Kind::FoldTrain,
        ..fault(x)
    };
    let (a, b) = (column(&fold(1.0), 12.5), column(&fold(4.0), 12.5));
    assert!(a[0].dy.abs() > 2.0);
    assert!((b[0].dy - 4.0 * a[0].dy).abs() < 1e-3 * a[0].dy.abs());
}

/// The ground keeps its relief at the default: a structure drawn 4 times taller with a quarter
/// of the relief share moves the ground as far, and wears off it as fast, as one at 1:1 with
/// the full share; and the default share on blocks 35 m high moves it as many metres.
#[test]
fn the_default_relief_keeps_the_ground_as_at_one_to_one() {
    let mut one = super::params::DeformationParams {
        true_height_share: 20.0 / 300.0,
        relief: 1.0,
        ..Default::default()
    };
    let four = super::params::DeformationParams::default();
    assert_eq!((four.block_stretch(300.0, 20.0), four.relief), (4.0, 0.25));
    let mut f1 = Feature {
        id: 0,
        kind: Kind::Normal,
        x: 0.0,
        z: 0.0,
        plate: 0,
        t_myr: -10.0,
        azimuth_deg: 0.0,
        half_along: 200.0,
        half_across: 100.0,
        wavelength: 50.0,
        amplitude: 6.0,
        phase: 0.0,
        dip_deg: 60.0,
        slip: 10.0,
        cause: 0,
        adrift: 0,
        wear_m: 0.0,
        rise: 0.0,
        depth: 0.0,
        base: 0.0,
        rock: 0,
    };
    let (dip, slip) = super::emplace::stretch(60.0, 10.0, 4.0);
    let mut f4 = Feature {
        dip_deg: dip as f32,
        slip: slip as f32,
        amplitude: 24.0,
        ..f1.clone()
    };
    // The default share on a 300 m by 35 m block.
    let x35 = four.block_stretch(300.0, 35.0);
    let (dip, slip) = super::emplace::stretch(60.0, 10.0, x35);
    let mut f35 = Feature {
        dip_deg: dip as f32,
        slip: slip as f32,
        amplitude: (6.0 * x35) as f32,
        ..f1.clone()
    };
    let throw = |f: &Feature| f64::from(f.slip) * f64::from(f.dip_deg).to_radians().sin();
    for (wear, now) in [(0.0, -9.0), (300.0, -8.0), (900.0, -5.0), (5_000.0, -9.5)] {
        f1.wear_m = wear;
        f4.wear_m = wear;
        f35.wear_m = wear;
        for kind in [Kind::Normal, Kind::FoldTrain] {
            f1.kind = kind;
            f4.kind = kind;
            f35.kind = kind;
            let height = |f: &Feature| match kind {
                Kind::FoldTrain => f64::from(f.amplitude),
                _ => throw(f),
            };
            let g1 = f64::from(relief_left(&f1, now, &one, 20.0)) * height(&f1);
            let g4 = f64::from(relief_left(&f4, now, &four, 20.0)) * height(&f4);
            assert!(
                (g1 - g4).abs() < 1e-4 * g1.abs().max(1e-3),
                "{kind:?} {wear}: {g1} vs {g4}"
            );
            // The same share on blocks 35 m high: the ground moves as many metres.
            let g35 = f64::from(relief_left(&f35, now, &four, 35.0)) * height(&f35) * 35.0;
            assert!(
                (g35 - 20.0 * g4).abs() < 1e-4 * (20.0 * g4).abs().max(1e-3),
                "{kind:?} {wear} at 35 m: {g35} m vs {} m",
                20.0 * g4
            );
        }
    }
    one.relief = 0.0;
    assert_eq!(relief_left(&f1, -9.0, &one, 20.0), 0.0);
}

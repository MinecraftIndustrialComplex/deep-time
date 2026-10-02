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
    let check = lattice::noddy_check(&features, &st, &index, &sample, -352.0, 640.0, 7.0);
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
    assert!(
        stats.max_shift > 2.0 && stats.max_shift < 80.0,
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
    let check = lattice::noddy_check(&features, &st, &index, &sample, -352.0, 640.0, 16.0);
    assert!(check.worst < 1e-5, "{check:?}");
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

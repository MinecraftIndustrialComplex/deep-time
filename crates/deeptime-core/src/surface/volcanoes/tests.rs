// SPDX-License-Identifier: LGPL-3.0-or-later
//! The P1-21 checks: arc volcanoes line up along the volcanic front at the arc spacing; on the
//! coupled pipeline, stratovolcanoes stand on arc crust, hotspot volcanoes form chains that age
//! away from their plume, every large igneous province becomes a flood-basalt province; the
//! events carry their causes; the strata log holds the volcanic rock; the catalogue is the same on
//! any thread count and after a resume.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::*;
use crate::checkpoint::Checkpoint;
use crate::params::planet_builtin;
use crate::preset::with_timeline;
use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};

#[test]
fn arcs_line_up_along_the_front_at_the_spacing() {
    let grid = GridDesc::for_planet("L1", 16_384, 16).unwrap();
    let (w, n) = (grid.w as usize, grid.cells());
    let mh = 300.0;
    // A 500-cell (2,400 km) arc three cells wide along row 200: full flux on the front, less on
    // its flanks; a weak arc elsewhere below the threshold.
    let mut flux = vec![0.0f64; n];
    for i in 100..600 {
        flux[199 * w + i] = 0.8;
        flux[200 * w + i] = 1.0;
        flux[201 * w + i] = 0.8;
        flux[300 * w + i] = 0.2;
    }
    let spacing = 60_000.0;
    let mut occupied = Vec::new();
    let mut rng = Rng::stream(5, "test");
    let born = place_arcs(&grid, mh, &flux, 0.25, spacing, &mut occupied, &mut rng);
    let km = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).hypot(a.1 - b.1) * mh / 1000.0;
    assert!(!born.is_empty());
    for &(x, z, c) in &born {
        assert!(flux[c] >= 0.25, "born on a cell of flux {}", flux[c]);
        let zr = grid.block_z(200) as f64;
        assert!((z - zr).abs() <= 1.5 * 16.0, "z {z} off the front");
        assert!((grid.block_x(100) as f64 - 8.0..=grid.block_x(599) as f64 + 8.0).contains(&x));
    }
    // Spaced: no pair nearer than 0.8 × the spacing; neighbours about the spacing apart.
    let pts: Vec<(f64, f64)> = born.iter().map(|&(x, z, _)| (x, z)).collect();
    let mut nn = Vec::new();
    for (a, &p) in pts.iter().enumerate() {
        let mut best = f64::INFINITY;
        for (b, &q) in pts.iter().enumerate() {
            if a != b {
                best = best.min(km(p, q));
            }
        }
        assert!(best >= 0.8 * 60.0 - 1e-6, "{best} km");
        nn.push(best);
    }
    let mean = nn.iter().sum::<f64>() / nn.len() as f64;
    assert!(
        (48.0..=80.0).contains(&mean),
        "mean neighbour distance {mean} km"
    );
    let count = born.len();
    assert!(
        (28..=52).contains(&count),
        "{count} volcanoes on 2,400 km of arc"
    );
    // Saturated: a second pass with the same occupancy adds almost nothing.
    let again = place_arcs(&grid, mh, &flux, 0.25, spacing, &mut occupied, &mut rng);
    assert!(again.len() <= count / 5, "{} more", again.len());
    // Density halves the spacing: about twice as many.
    let mut occ2 = Vec::new();
    let dense = place_arcs(&grid, mh, &flux, 0.25, spacing / 2.0, &mut occ2, &mut rng);
    assert!(
        dense.len() as f64 > 1.6 * count as f64,
        "{} vs {count}",
        dense.len()
    );
}

#[test]
fn the_front_is_the_middle_of_the_arc_band() {
    let grid = GridDesc::for_planet("L1", 16_384, 16).unwrap();
    let (w, n) = (grid.w as usize, grid.cells());
    let mut flux = vec![0.0f64; n];
    let mut elev = vec![0.0f32; n];
    // A band five cells wide along rows 100–104.
    for j in 100..105 {
        for i in 200..400 {
            flux[j * w + i] = 1.0;
        }
    }
    // A patch 40 × 40 cells striped every other row (the plate model's lattice stripes).
    for j in (300..340).step_by(2) {
        for i in 500..540 {
            flux[j * w + i] = 1.0;
        }
    }
    // A trench: strong flux but 6 km deep.
    for i in 600..700 {
        flux[50 * w + i] = 1.0;
        elev[50 * w + i] = -6000.0;
    }
    let front = volcanic_front(&grid, &flux, &elev, 0.0, 0.25);
    let rows: Vec<usize> = (100..105)
        .filter(|&j| (200..400).any(|i| front[j * w + i]))
        .collect();
    assert_eq!(rows, vec![102], "the band's centre row");
    let patch = (300..340)
        .flat_map(|j| (500..540).map(move |i| j * w + i))
        .filter(|&c| front[c])
        .count();
    assert!(
        patch < 200,
        "{patch} of 800 striped cells on the patch's skeleton"
    );
    assert!(patch > 0);
    assert!(
        (600..700).all(|i| !front[50 * w + i]),
        "no front in a trench"
    );
}

fn pipeline_json(n: u32, myr: f64, seed: u64) -> String {
    let json = with_timeline(
        planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(myr),
    )
    .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = serde_json::json!(n);
    v["resolution"]["l1_r"] = serde_json::json!(32);
    v["keyframes"]["size"] = serde_json::json!([128, 64]);
    v["keyframes"]["every_myr"] = serde_json::json!(20.0);
    v["seed"] = serde_json::json!(seed);
    // Hotspots and plume heads often enough to see chains and provinces on a short run.
    v["tectonics"]["hotspot_births_per_100myr"] = serde_json::json!(12.0);
    v["tectonics"]["lip_probability"] = serde_json::json!(1.0);
    v["volcanoes"] = serde_json::json!({});
    for k in ["deposits", "young", "amplify"] {
        v.as_object_mut().unwrap().remove(k);
    }
    v.to_string()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-volcanoes-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn session(json: &str, dir: &Path, threads: usize) -> Session {
    Session::create(
        json,
        "",
        SessionOptions {
            out_dir: dir.to_path_buf(),
            threads,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap()
}

fn finish(s: &Session) {
    loop {
        match s.advance(Duration::from_secs(60)).unwrap() {
            AdvanceStatus::Done => break,
            AdvanceStatus::Cancelled => panic!("cancelled"),
            _ => {}
        }
    }
}

#[test]
fn the_pipeline_puts_volcanoes_where_the_plates_say() {
    let json = pipeline_json(20_000, 120.0, 11);
    let dir = temp_dir("placement");
    let s = session(&json, &dir, 2);
    assert_eq!(
        s.stage_names(),
        ["plates", "climate", "surface", "volcanoes", "structure"]
    );
    let p = VolcanoParams::default();
    let grid = GridDesc::for_planet("L1", 16_384, 32).unwrap();
    // Step by step: every stratovolcano is born on an arc cell (or next to one: its point is
    // drawn inside its cell) whose flux, in the fields of that very step, is at least the
    // threshold.
    let mut known = 0u32;
    let mut births = 0;
    loop {
        let status = s.advance(Duration::ZERO).unwrap();
        let w_now = s.checkpoint().unwrap().world;
        if let Some(st) = &w_now.volcanoes {
            let f = |name: &str| &w_now.fields[name].data;
            let (flags, conv, crust) = (
                f("plates/flags"),
                f("plates/convergence_mm_yr"),
                f("plates/crust_type"),
            );
            let w = grid.w as usize;
            for v in st
                .volcanoes
                .iter()
                .filter(|v| v.id >= known && v.kind == Kind::Strato)
            {
                let c = cell_of(&grid, v.x, v.z);
                let (i, j) = ((c % w) as i64, (c / w) as i64);
                let mut best = 0.0f64;
                for dj in -1..=1 {
                    for di in -1..=1 {
                        let jj = (j + dj).clamp(0, i64::from(grid.h) - 1) as usize;
                        let cc = jj * w + grid.wrap_i(i + di) as usize;
                        best = best.max(arc_flux(flags[cc], conv[cc], crust[cc]));
                    }
                }
                assert!(
                    best >= p.arc_min_flux,
                    "volcano {} born on flux {best}",
                    v.id
                );
                births += 1;
            }
            known = st.next_id;
        }
        if status == AdvanceStatus::Done {
            break;
        }
    }
    assert!(births > 0);
    let world = s.checkpoint().unwrap().world;
    let st = world.volcanoes.as_ref().expect("the catalogue");
    let n = grid.cells();
    let frame = BlockFrame::new(16_384);
    assert!(st.counts.strato_born > 0, "{:?}", st.counts);
    assert!(
        st.counts.shield_born + st.counts.silicic_born > 0,
        "{:?}",
        st.counts
    );

    // Active stratovolcanoes last saw enough arc flux and have grown.
    let mut active = 0;
    for v in st
        .volcanoes
        .iter()
        .filter(|v| v.kind == Kind::Strato && v.active())
    {
        active += 1;
        assert!(
            v.flux_last >= p.arc_min_flux,
            "volcano {}: {}",
            v.id,
            v.flux_last
        );
        assert!(v.radius_m > 0.0 && v.missed < 2);
    }
    let stratos = st
        .volcanoes
        .iter()
        .filter(|v| v.kind == Kind::Strato)
        .count();
    assert!(
        active > 0,
        "no active arc volcano: {stratos} in the catalogue, {:?}",
        st.counts
    );
    // Active ones keep the spacing (they were placed apart and ride their plates together).
    let live: Vec<&Volcano> = st
        .volcanoes
        .iter()
        .filter(|v| v.kind == Kind::Strato && v.active())
        .collect();
    let mut close = 0;
    for (a, va) in live.iter().enumerate() {
        for vb in &live[a + 1..] {
            let d = (frame.wrap_x(va.x - vb.x) * 300.0).hypot((va.z - vb.z) * 300.0);
            if d < 0.5 * p.arc_spacing_km * 1000.0 {
                close += 1;
            }
        }
    }
    assert!(
        close * 20 <= live.len(),
        "{close} close pairs among {} live cones",
        live.len()
    );

    // Hotspot chains: in chain order, born later; every one but the newest two is dead; the live
    // ones lie within the spacing of their plume.
    let plates = world.plates.as_ref().unwrap();
    let mut chains: BTreeMap<u32, Vec<&Volcano>> = BTreeMap::new();
    for v in st.volcanoes.iter().chain(&st.gone) {
        if let Some(h) = v.hotspot {
            chains.entry(h).or_default().push(v);
        }
    }
    let mut long = 0;
    let (mut pairs, mut aged) = (0usize, 0usize);
    for (h, mut vs) in chains {
        vs.sort_by_key(|v| v.chain);
        for pair in vs.windows(2) {
            assert!(pair[0].born_myr < pair[1].born_myr, "hotspot {h}");
        }
        for v in &vs[..vs.len().saturating_sub(2)] {
            assert!(
                !v.active(),
                "hotspot {h}: volcano {} of {} still active",
                v.chain,
                vs.len()
            );
        }
        if vs.len() >= 3 {
            long += 1;
        }
        let hs = plates.hotspots.iter().find(|x| x.id == h).unwrap();
        let (px, pz) = frame.block(DVec3::from_array(hs.pos));
        for v in vs.iter().filter(|v| v.active() && v.gone.is_none()) {
            let d = (frame.wrap_x(v.x - px) * 300.0).hypot((v.z - pz) * 300.0);
            assert!(
                d <= p.hotspot_spacing_km * 1000.0 + 1.0,
                "{d} m from its plume"
            );
        }
        // Ageing along the track: of two consecutive volcanoes riding the same plate, the older
        // is mostly the farther from the plume (plate motions turn, so not always).
        for pair in vs.windows(2) {
            let (older, newer) = (pair[0], pair[1]);
            if older.gone.is_some() || newer.gone.is_some() || older.plate != newer.plate {
                continue;
            }
            let d = |v: &Volcano| (frame.wrap_x(v.x - px) * 300.0).hypot((v.z - pz) * 300.0);
            pairs += 1;
            if d(older) + 1.0 >= d(newer) {
                aged += 1;
            }
        }
    }
    assert!(
        pairs == 0 || aged * 10 >= pairs * 7,
        "{aged} of {pairs} consecutive pairs have the older farther from the plume"
    );
    assert!(long > 0, "no hotspot made a chain of three");

    // Every large igneous province became a flood-basalt province caused by it.
    let lips: Vec<EventId> = world
        .events
        .iter()
        .filter(|e| e.kind == LARGE_IGNEOUS_PROVINCE)
        .map(|e| e.id)
        .collect();
    assert!(!lips.is_empty());
    assert_eq!(st.basalts.len(), lips.len());
    let fb: Vec<_> = world
        .events
        .iter()
        .filter(|e| e.kind == kind::FLOOD_BASALT)
        .collect();
    assert_eq!(fb.len(), lips.len());
    for e in &fb {
        assert!(lips.contains(&e.causes[0]));
    }
    // Volcano events (grouped): arc segments caused by a subduction onset, hotspot chains by their
    // track (or plume head); the notable cones by their group; one arc event per segment.
    let kind_of = |id: EventId| {
        world
            .events
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.kind.clone())
    };
    let (mut arcs, mut chains_n, mut strato_events) = (0, 0, 0);
    for e in world.events.iter().filter(|e| e.source == SOURCE) {
        e.validate().unwrap();
        let cause = e.causes.first().and_then(|&c| kind_of(c));
        match e.kind.as_str() {
            kind::ARC => {
                arcs += 1;
                if let Some(c) = &cause {
                    assert_eq!(c, "deeptime:subduction_onset");
                }
                assert!(matches!(e.params.get("cones"), Some(ParamValue::Str(_))));
            }
            kind::HOTSPOT_CHAIN => {
                chains_n += 1;
                assert!(
                    matches!(
                        cause.as_deref(),
                        Some("deeptime:hotspot_track" | "deeptime:large_igneous_province")
                    ),
                    "{cause:?}"
                );
            }
            kind::STRATOVOLCANO => {
                strato_events += 1;
                assert!(
                    matches!(cause.as_deref(), Some(kind::ARC | "deeptime:subduction_onset")),
                    "{cause:?}"
                );
            }
            kind::SHIELD | kind::CALDERA_COMPLEX => {
                assert_eq!(cause.as_deref(), Some(kind::HOTSPOT_CHAIN));
            }
            kind::CALDERA | kind::FLOOD_BASALT => assert!(!e.causes.is_empty()),
            other => panic!("unexpected kind {other}"),
        }
    }
    assert!(arcs > 0 && chains_n > 0 && strato_events > 0);
    // One event per arc segment, but small segments with an onset are counts on it.
    let small = st
        .arcs
        .values()
        .filter(|g| {
            g.born < SMALL_SEGMENT && g.cause.is_some_and(|c| world.events.iter().any(|e| e.id == c))
        })
        .count();
    assert_eq!(arcs + small, st.arcs.len(), "one event per segment or a count on its onset");
    assert_eq!(st.counts.small_arc_segments as usize, small);
    assert!(
        strato_events <= st.arcs.len(),
        "at most one notable cone per segment"
    );
    if small > 0 {
        let counted = world
            .events
            .iter()
            .filter(|e| matches!(e.params.get("volcano_cones_built"), Some(ParamValue::Int(n)) if *n > 0))
            .count();
        assert!(counted > 0, "small segments counted on their onsets");
    }

    // The strata log holds the volcanic rock.
    let log = &world.surface.as_ref().unwrap().log;
    let (andesite, basalt, tuff) = (index("andesite"), index("basalt"), index("tuff"));
    let mut seen = [0usize; 3];
    for c in 0..n {
        for l in log.column(c) {
            if l.class == andesite {
                seen[0] += 1;
            } else if l.class == basalt {
                seen[1] += 1;
            } else if l.class == tuff {
                seen[2] += 1;
            }
        }
    }
    assert!(seen.iter().all(|&k| k > 0), "{seen:?}");
    for c in 0..n {
        log.check(c, crate::synth::EVERY_CLASS.len()).unwrap();
    }
    // The map layer marks them; the notes carry the summary.
    let map = &world.fields[field::MAP].data;
    assert!(map.contains(&code::STRATO_ACTIVE));
    let note: serde_json::Value = serde_json::from_str(&world.notes[STAGE_NAME]).unwrap();
    assert!(summary_line(&note).starts_with("volcanoes_final="));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn threads_and_a_resume_leave_the_volcanoes_unchanged() {
    let json = pipeline_json(12_000, 40.0, 4);
    let dir = temp_dir("one");
    let s = session(&json, &dir, 1);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let dir2 = temp_dir("resumed");
    let s2 = session(&json, &dir2, 3);
    for _ in 0..9 {
        assert_eq!(s2.advance(Duration::ZERO).unwrap(), AdvanceStatus::Running);
    }
    let ck = s2.checkpoint().unwrap();
    let bytes = ck.to_bytes().unwrap();
    let ck = Checkpoint::from_bytes(Path::new("ck"), &bytes).unwrap();
    let s3 = Session::resume(
        ck,
        SessionOptions {
            out_dir: dir2.clone(),
            threads: 3,
            mem_cap: 0,
        },
        Arc::new(NoGauge),
    )
    .unwrap();
    finish(&s3);
    let got = s3.checkpoint().unwrap().world;
    assert_eq!(got.volcanoes, world.volcanoes);
    assert!(!world.volcanoes.as_ref().unwrap().volcanoes.is_empty());
    let ours = |w: &WorldState| -> Vec<deeptime_format::Event> {
        w.events
            .iter()
            .filter(|e| e.source == SOURCE)
            .cloned()
            .collect()
    };
    assert_eq!(ours(&got), ours(&world));
    assert_eq!(
        got.surface.as_ref().unwrap().elevation_m,
        world.surface.as_ref().unwrap().elevation_m
    );
    let bits = |w: &WorldState| -> Vec<u32> {
        w.fields[field::MAP]
            .data
            .iter()
            .map(|x| x.to_bits())
            .collect()
    };
    assert_eq!(bits(&got), bits(&world));
    for d in [dir, dir2] {
        let _ = std::fs::remove_dir_all(d);
    }
}

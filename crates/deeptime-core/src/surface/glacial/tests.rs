// SPDX-License-Identifier: LGPL-3.0-or-later
//! Glacial erosion: fjord overdeepening, till mass balance, cold beds, thread counts.

use super::*;
use crate::surface::pflood::{self, LAND};
use crate::surface::routing;

/// A glaciated range draining to a sea: a ridge of `top` m at the west falling towards the east,
/// where the last `sea_cols` columns are a sea `depth` m deep, with a valley along the middle row
/// and a little noise.
fn range(w: usize, h: usize, top: f64, sea_cols: usize, depth: f64) -> Vec<f64> {
    let mut z = vec![0.0; w * h];
    let land = w - sea_cols;
    for j in 0..h {
        for i in 0..w {
            let c = j * w + i;
            if i >= land {
                z[c] = -depth;
                continue;
            }
            let x = i as f64 / (land - 1) as f64;
            let valley = (j as f64 - (h / 2) as f64).abs() * 60.0;
            let noise = ((i * 7919 + j * 104_729) % 97) as f64 * 0.05;
            z[c] = 20.0 + top * (1.0 - x) + valley + noise;
        }
    }
    z
}

/// The physics: glaciers as wide as they are (the valley width is a readability setting).
fn physical() -> GlacialParams {
    GlacialParams {
        valley_width: 1.0,
        ..GlacialParams::default()
    }
}

#[test]
fn the_valley_width_widens_the_footprint_not_the_ice() {
    let (w, h) = (48, 21);
    let d = Dims::new(w, h, 1000.0, false);
    // A broad valley (5 m a row across), so the footprint is not held in by steep walls.
    let mut z = range(w, h, 2500.0, 6, 150.0);
    for j in 0..h {
        for i in 0..w - 6 {
            z[j * w + i] -= (j as f64 - (h / 2) as f64).abs() * 55.0;
        }
    }
    // Ice only on the high west: a valley glacier flows down the axis.
    let cl = climate(w * h, 2000.0, -2.0);
    let rt = route(&d, &z, 0.0);
    let clim = IceClimate {
        ela_z: &cl.ela,
        precip_m_yr: &cl.p,
        t_c: &cl.t,
        allowed: None,
        time_share: None,
    };
    let solve = |p: &GlacialParams| {
        let mut st = IceState::default();
        ice(
            &d, &rt.r, &rt.kind, &z, &rt.filled, 0.0, &clim, p, 9.81, &mut st,
        );
        st
    };
    // Broad glaciers (a tenth as deep as wide), so they span cells either way.
    let one = solve(&GlacialParams {
        form_ratio: 0.1,
        ..physical()
    });
    let three = solve(&GlacialParams {
        form_ratio: 0.1,
        valley_width: 3.0,
        ..GlacialParams::default()
    });
    let widest = one.width.iter().copied().fold(0.0, f64::max);
    assert!(widest > 1000.0, "{widest}");
    // The same glaciers (the flow law does not see the setting) …
    assert_eq!(one.thick, three.thick);
    // … drawn over more ground.
    let covered = |s: &IceState| s.footprint.iter().filter(|x| x.is_finite()).count();
    assert!(
        covered(&three) > covered(&one),
        "{} vs {}",
        covered(&three),
        covered(&one)
    );
}

struct Routed {
    kind: Vec<u8>,
    filled: Vec<f64>,
    r: routing::Routing,
}

fn route(d: &Dims, z: &[f64], sea: f64) -> Routed {
    let n = d.cells();
    let mut kind = vec![LAND; n];
    pflood::open_sea(d, z, sea, 4, &mut kind);
    let base = vec![sea; n];
    let mut filled = vec![0.0; n];
    pflood::flood(d, z, &base, &mut kind, &mut filled);
    let mut r = routing::Routing::default();
    routing::receivers(d, &filled, &kind, &mut r);
    routing::stack(d, &kind, &mut r);
    Routed { kind, filled, r }
}

struct Climate {
    ela: Vec<f64>,
    p: Vec<f32>,
    t: Vec<f32>,
}

fn climate(n: usize, ela: f64, t: f32) -> Climate {
    Climate {
        ela: vec![ela; n],
        p: vec![2.0; n],
        t: vec![t; n],
    }
}

/// Runs `steps` steps of `dt` years, re-routing before each; returns the bed and the last output
/// with the volume totals of every step added up.
fn run(
    d: &Dims,
    z: &mut [f64],
    cl: &Climate,
    p: &GlacialParams,
    steps: usize,
    dt: f64,
) -> (GlacialOut, [f64; 5]) {
    let mut out = GlacialOut::default();
    let mut tot = [0.0; 5];
    for _ in 0..steps {
        let rt = route(d, z, 0.0);
        let clim = IceClimate {
            ela_z: &cl.ela,
            precip_m_yr: &cl.p,
            t_c: &cl.t,
            allowed: None,
            time_share: None,
        };
        step(
            d, &rt.r, &rt.kind, &rt.filled, 0.0, &clim, p, 9.81, dt, z, &mut out,
        );
        tot[0] += out.eroded_m3;
        tot[1] += out.till_m3;
        tot[2] += out.lake_m3;
        tot[3] += out.sink_m3;
        tot[4] += out.sea_m3;
    }
    (out, tot)
}

#[test]
fn a_glacier_reaching_the_sea_cuts_a_fjord_below_sea_level() {
    let (w, h) = (48, 9);
    let d = Dims::new(w, h, 1000.0, false);
    let mut z = range(w, h, 2500.0, 6, 150.0);
    let z0 = z.clone();
    let cl = climate(w * h, 600.0, -2.0);
    let p = physical();
    let (out, _) = run(&d, &mut z, &cl, &p, 8, 5.0e5);
    assert!(out.ice.cells > 0, "the range is glaciated");
    // The valley's last land cells before the old coast are now below sea level: a fjord, deeper
    // than the sea floor it opens into (an overdeepening behind a sill).
    let mid = h / 2;
    let land = w - 6;
    let floor: Vec<f64> = (land - 8..land).map(|i| z[mid * w + i]).collect();
    let deepest = floor.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        deepest < -150.0,
        "the valley's lower course is cut below the sea floor: {floor:?}"
    );
    // The glacier eroded most in the valley, and the ridge far above the ELA hardly at all.
    let cut = |i: usize, j: usize| z0[j * w + i] - z[j * w + i];
    assert!(cut(land - 3, mid) > cut(land - 3, 0));
    // Afloat ice stops cutting: the fjord is at most as deep as the ice can stay grounded.
    assert!(deepest > -3000.0, "{deepest}");
}

#[test]
fn eroded_rock_is_all_accounted_for() {
    let (w, h) = (40, 12);
    let d = Dims::new(w, h, 2000.0, true);
    let mut z = range(w, h, 3000.0, 5, 300.0);
    // A closed basin in the valley (a lake the meltwater crosses) and one without an outlet.
    for (ci, cj, depth) in [(20usize, 6usize, 400.0), (10, 2, 300.0)] {
        for j in 0..h {
            for i in 0..w {
                let d2 = ((i as f64 - ci as f64).powi(2) + (j as f64 - cj as f64).powi(2)) / 2.0;
                z[j * w + i] -= depth * (-d2).exp();
            }
        }
    }
    let cl = climate(w * h, 1200.0, 0.0);
    let p = GlacialParams {
        till_share: 0.4,
        max_till_m: 30.0,
        ..physical()
    };
    let before: f64 = z.iter().sum();
    let (_, tot) = run(&d, &mut z, &cl, &p, 6, 5.0e5);
    let [eroded, till, lake, sink, sea] = tot;
    assert!(eroded > 0.0 && till > 0.0 && sea > 0.0, "{tot:?}");
    let parts = till + lake + sink + sea;
    assert!(
        (eroded - parts).abs() <= 1e-9 * eroded,
        "eroded {eroded} vs till + lake + sink + sea {parts}"
    );
    // What stays on the grid is what did not leave for the sea.
    let after: f64 = z.iter().sum();
    let a = d.cell_area();
    let kept = (after - before) * a;
    assert!(
        (kept + sea).abs() <= 1e-6 * eroded,
        "grid change {kept} m³, sea {sea} m³"
    );
}

#[test]
fn a_cold_bed_does_not_erode_but_thick_ice_warms_it() {
    let (w, h) = (32, 7);
    let d = Dims::new(w, h, 2000.0, false);
    // A thin polar ice cap (little snow): frozen to its bed, it moves by creep alone and cuts
    // nothing.
    let thin = GlacialParams {
        max_accumulation_m_yr: 0.05,
        ..physical()
    };
    let mut z = range(w, h, 1200.0, 4, 100.0);
    let cl = climate(w * h, 200.0, -30.0);
    let (out, tot) = run(&d, &mut z, &cl, &thin, 2, 5.0e5);
    assert!(out.ice.cells > 0);
    let thickest = out.ice.thick.iter().copied().fold(0.0, f64::max);
    assert!(thickest < 800.0, "{thickest}");
    assert_eq!(tot[0], 0.0, "cold-based ice erodes nothing");
    // The same range at −2 °C slides and erodes.
    let mut z = range(w, h, 1200.0, 4, 100.0);
    let cl = climate(w * h, 200.0, -2.0);
    let (_, tot) = run(&d, &mut z, &cl, &thin, 2, 5.0e5);
    assert!(tot[0] > 0.0);
    // At −30 °C again, but with snow enough for a thick cap: its bed thaws under the ice and
    // the ice erodes.
    let mut z = range(w, h, 1200.0, 4, 100.0);
    let cl = climate(w * h, 200.0, -30.0);
    let (out, tot) = run(&d, &mut z, &cl, &physical(), 2, 5.0e5);
    let thickest = out.ice.thick.iter().copied().fold(0.0, f64::max);
    assert!(thickest > 800.0 && tot[0] > 0.0, "{thickest} {}", tot[0]);
}

#[test]
fn no_ice_below_the_equilibrium_line_and_none_where_not_allowed() {
    let (w, h) = (24, 5);
    let d = Dims::new(w, h, 1000.0, false);
    let mut z = range(w, h, 800.0, 3, 50.0);
    let z0 = z.clone();
    let cl = climate(w * h, 2000.0, 0.0);
    let (out, tot) = run(&d, &mut z, &cl, &physical(), 2, 5.0e5);
    assert_eq!(out.ice.cells, 0);
    assert_eq!(tot[0], 0.0);
    assert_eq!(z, z0);
    // Above the line, but the climate's own ice model says none.
    let cl = climate(w * h, 100.0, 0.0);
    let none = vec![0.0f32; w * h];
    let rt = route(&d, &z, 0.0);
    let clim = IceClimate {
        ela_z: &cl.ela,
        precip_m_yr: &cl.p,
        t_c: &cl.t,
        allowed: Some(&none),
        time_share: None,
    };
    let mut st = IceState::default();
    ice(
        &d,
        &rt.r,
        &rt.kind,
        &z,
        &rt.filled,
        0.0,
        &clim,
        &physical(),
        9.81,
        &mut st,
    );
    assert_eq!(st.cells, 0);
}

#[test]
fn the_same_bits_on_any_thread_count() {
    let (w, h) = (64, 32);
    let d = Dims::new(w, h, 1500.0, true);
    let base: Vec<f64> = (0..w * h)
        .map(|c| {
            let (i, j) = ((c % w) as f64, (c / w) as f64);
            1800.0 * (i / 64.0 * std::f64::consts::TAU).sin().max(0.0)
                + 400.0 * (j / 9.0).cos()
                + ((c * 2_654_435_761) % 1000) as f64 * 0.3
                - 300.0
        })
        .collect();
    let cl = climate(w * h, 700.0, -3.0);
    let p = physical();
    let go = |threads: usize| {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();
        pool.install(|| {
            let mut z = base.clone();
            let (out, tot) = run(&d, &mut z, &cl, &p, 3, 5.0e5);
            (z, out.till, tot)
        })
    };
    let (z1, t1, s1) = go(1);
    let (z4, t4, s4) = go(4);
    assert!(s1[0] > 0.0);
    let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&z1), bits(&z4));
    assert_eq!(bits(&t1), bits(&t4));
    assert_eq!(s1.map(f64::to_bits), s4.map(f64::to_bits));
}

/// A small coupled planet (plates, climate, surface, structure) with the snowline pushed down so
/// that it glaciates: 3,000 L0 samples, L1 256 × 128, `myr` of history.
fn glaciated_pipeline(myr: f64) -> String {
    let json = crate::preset::with_timeline(
        crate::params::planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(myr),
    )
    .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = serde_json::json!(3000);
    v["resolution"]["l1_r"] = serde_json::json!(64);
    v["keyframes"]["size"] = serde_json::json!([128, 64]);
    v["keyframes"]["every_myr"] = serde_json::json!(20.0);
    v["seed"] = serde_json::json!(11);
    v["climate"]["ela_offset_c"] = serde_json::json!(7.0);
    let o = v.as_object_mut().unwrap();
    for k in ["young", "amplify", "deposits"] {
        o.remove(k);
    }
    v.to_string()
}

fn session(json: &str, dir: &std::path::Path, threads: usize) -> crate::session::Session {
    crate::session::Session::create(
        json,
        "",
        crate::session::SessionOptions {
            out_dir: dir.to_path_buf(),
            threads,
            mem_cap: 0,
        },
        std::sync::Arc::new(crate::session::NoGauge),
    )
    .unwrap()
}

fn finish(s: &crate::session::Session) {
    use crate::session::AdvanceStatus;
    loop {
        match s.advance(std::time::Duration::from_secs(60)).unwrap() {
            AdvanceStatus::Done => break,
            AdvanceStatus::Cancelled => panic!("cancelled"),
            _ => {}
        }
    }
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-glacial-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn a_glaciated_planet_keeps_its_mass_logs_its_till_and_resumes() {
    use crate::session::AdvanceStatus;
    let json = glaciated_pipeline(40.0);
    let dir = temp_dir("straight");
    let s = session(&json, &dir, 2);
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let t = Totals::read(&world.notes);
    assert!(t.steps_with_ice > 0 && t.eroded_km3 > 0.0, "{t:?}");
    // Every eroded cubic kilometre went somewhere.
    let parts = t.till_km3 + t.lake_km3 + t.sink_km3 + t.sea_km3;
    assert!(
        (t.eroded_km3 - parts).abs() <= 1e-9 * t.eroded_km3,
        "{t:?}: eroded vs till + lake + sink + sea"
    );
    // The surface's own ledger still closes with the glaciers moving rock.
    let st = world.surface.as_ref().unwrap();
    assert!(
        st.totals.mass_residual_km3.abs() < 1e-6 * st.totals.eroded_km3,
        "residual {} of {} km³ eroded",
        st.totals.mass_residual_km3,
        st.totals.eroded_km3
    );
    // The strata log holds till, and no more of it than the glaciers ever eroded.
    let till = crate::surface::lithology::index("till");
    let a_km2 = (64.0 * 300.0f64).powi(2) / 1.0e6;
    let logged: f64 = (0..st.log.cells())
        .flat_map(|c| st.log.column(c).iter())
        .filter(|l| l.class == till)
        .map(|l| f64::from(l.thickness_m) / 1000.0 * a_km2)
        .sum();
    assert!(logged > 0.0, "the log records till");
    assert!(
        logged <= t.eroded_km3 * 1.01,
        "{logged} km³ of till logged, {t:?}"
    );
    let ice = &world.fields[crate::surface::field::ICE].data;
    assert!(ice.iter().any(|&x| x > 0.0));

    // Resume from a checkpoint inside the group on another thread count: the same world.
    let dir2 = temp_dir("resumed");
    let s2 = session(&json, &dir2, 1);
    for _ in 0..9 {
        assert_eq!(
            s2.advance(std::time::Duration::ZERO).unwrap(),
            AdvanceStatus::Running
        );
    }
    let ck = s2.checkpoint().unwrap();
    let bytes = ck.to_bytes().unwrap();
    let ck = crate::checkpoint::Checkpoint::from_bytes(std::path::Path::new("ck"), &bytes).unwrap();
    let s3 = crate::session::Session::resume(
        ck,
        crate::session::SessionOptions {
            out_dir: dir2.clone(),
            threads: 3,
            mem_cap: 0,
        },
        std::sync::Arc::new(crate::session::NoGauge),
    )
    .unwrap();
    finish(&s3);
    let got = s3.checkpoint().unwrap().world;
    let gs = got.surface.as_ref().unwrap();
    assert_eq!(gs.elevation_m, st.elevation_m, "elevation after resuming");
    assert_eq!(gs.log, st.log, "strata log after resuming");
    assert_eq!(got.notes[NOTES_KEY], world.notes[NOTES_KEY]);
    for name in [
        crate::surface::field::ICE,
        crate::surface::field::GLACIAL_EROSION,
    ] {
        assert_eq!(got.fields[name], world.fields[name], "{name}");
    }
    for d in [dir, dir2] {
        let _ = std::fs::remove_dir_all(d);
    }
}

#[test]
fn the_flow_law_is_the_shallow_ice_approximation() {
    let p = physical();
    let f = Flow::new(&p, 9.81, 1.0e5);
    // Pure creep of a 1 km sheet on a 0.2 % slope: q = (2A/5)(ρgS)³H⁵ per unit width.
    let a = 2.4e-24 * SECONDS_PER_YEAR;
    let want = 2.0 * a / 5.0 * (917.0 * 9.81 * 0.002f64).powi(3) * 1000.0f64.powi(5);
    let w = f.width(1000.0);
    let got = f.flux(1000.0, 0.002, 0.0) / w;
    // The thickness insulates a kilometre of ice a little (warm share 0.03): sliding adds some.
    assert!(got >= want && got < 1.6 * want, "{got} vs {want}");
    // 100 kPa of basal stress on a melting bed slides at `sliding_m_yr`.
    let h = 1.0e5 / (917.0 * 9.81 * 0.05);
    assert!((f.sliding(h, 0.05, 1.0) - p.sliding_m_yr).abs() < 1e-9);
    // The surface solve inverts the flux.
    let (zs, hh, s) = solve_surface(&f, 5.0e7, 100.0, 50.0, 2000.0, 1.0);
    assert!((f.flux(hh, s, 1.0) - 5.0e7).abs() < 1e-3 * 5.0e7);
    assert!((zs - 100.0 - hh).abs() < 1e-9);
}

/// The forest-parallel ice solve (P1-17) is the serial stack walk bit for bit, on a wrapping
/// range with closed basins, uneven climate and a mask of allowed ice, at 1, 3 and 8 threads.
#[test]
fn the_parallel_ice_solve_is_the_serial_walk_bit_for_bit() {
    let (w, h) = (96, 41);
    let d = Dims::new(w, h, 1000.0, true);
    let mut z = range(w, h, 3200.0, 10, 200.0);
    for (c, v) in z.iter_mut().enumerate() {
        let (i, j) = (c % w, c / w);
        // Dimples (closed basins) and a second, lower ridge.
        if (i * 13 + j * 7) % 29 == 0 {
            *v -= 300.0;
        }
        *v += 400.0 * ((i as f64 * 0.21).sin() * (j as f64 * 0.17).cos()).max(0.0);
    }
    let n = w * h;
    let rt = route(&d, &z, 0.0);
    let ela: Vec<f64> = (0..n).map(|c| 1500.0 + ((c * 31) % 400) as f64).collect();
    let p_m: Vec<f32> = (0..n).map(|c| 0.5 + ((c * 17) % 13) as f32 * 0.2).collect();
    let t: Vec<f32> = (0..n).map(|c| -15.0 + ((c * 11) % 19) as f32).collect();
    let allowed: Vec<f32> = (0..n)
        .map(|c| if c % 23 == 0 { 0.0 } else { 1.0 })
        .collect();
    let clim = IceClimate {
        ela_z: &ela,
        precip_m_yr: &p_m,
        t_c: &t,
        allowed: Some(&allowed),
        time_share: None,
    };
    let p = GlacialParams {
        form_ratio: 0.1,
        valley_width: 2.0,
        ..GlacialParams::default()
    };
    let bits = |s: &IceState| -> Vec<u64> {
        let mut v = Vec::new();
        for a in [
            &s.q,
            &s.melt_frac,
            &s.thick,
            &s.surface,
            &s.footprint,
            &s.cover,
            &s.width,
            &s.sliding,
            &s.slope,
        ] {
            v.extend(a.iter().map(|x| x.to_bits()));
        }
        v.push(s.cells as u64);
        v
    };
    let mut want = IceState::default();
    ice_serial(
        &d, &rt.r, &rt.kind, &z, &rt.filled, 0.0, &clim, &p, 9.81, &mut want,
    );
    assert!(want.cells > 50, "{} glacier cells", want.cells);
    for threads in [1, 3, 8] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();
        let mut got = IceState::default();
        pool.install(|| {
            ice(
                &d, &rt.r, &rt.kind, &z, &rt.filled, 0.0, &clim, &p, 9.81, &mut got,
            )
        });
        assert!(bits(&got) == bits(&want), "{threads} threads");
    }
}

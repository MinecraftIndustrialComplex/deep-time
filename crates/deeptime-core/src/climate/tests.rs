// SPDX-License-Identifier: LGPL-3.0-or-later
//! The climate on a synthetic planet and in the coupled pipeline.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use deeptime_format::GridDesc;

use super::snapshot::{self, Planet};
use super::*;
use crate::checkpoint::Checkpoint;
use crate::params::planet_builtin;
use crate::preset::with_timeline;
use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};

/// An idealised planet on 256 × 128 cells (16k, r 64): ocean at −4 km, a continent from 60° to
/// 150° E between 60° S and 70° N at 300 m with a 3-km north–south range along 70–74° E from 25°
/// to 60° N, and a 2-km polar continent south of 70° S.
#[allow(clippy::needless_range_loop)]
fn idealised() -> (GridDesc, Vec<f64>, Vec<f64>) {
    let grid = GridDesc::for_planet("L1", 16384, 64).unwrap();
    let (w, h) = (grid.w as usize, grid.h as usize);
    let lat: Vec<f64> = (0..h).map(|j| lat_of_row(&grid, j as u32)).collect();
    let mut z = vec![-4000.0; w * h];
    for j in 0..h {
        for i in 0..w {
            let lon = i as f64 / w as f64 * 360.0;
            let c = j * w + i;
            if lat[j] < -70.0 {
                z[c] = 2000.0;
            } else if (60.0..150.0).contains(&lon) && (-60.0..70.0).contains(&lat[j]) {
                z[c] = 300.0;
                if (70.0..74.0).contains(&lon) && (25.0..60.0).contains(&lat[j]) {
                    z[c] = 3000.0;
                }
            }
        }
    }
    (grid, lat, z)
}

fn planet() -> Planet {
    Planet {
        axial_tilt_deg: 23.4,
        star_flux_rel: 1.0,
        day_length_h: 24.0,
        earth_scale: 782.278 / 6370.0,
        m_per_block_h: 300.0,
    }
}

#[test]
fn an_idealised_planet_has_earthlike_zones() {
    let (grid, lat, z) = idealised();
    let (w, h) = (grid.w as usize, grid.h as usize);
    let p = ClimateParams::default();
    let d = Dims::new(w, h, 64.0 * 300.0, true);
    let inp = snapshot::Input {
        grid: &grid,
        d,
        lat: &lat,
        h: &z,
        sea: 0.0,
        p: &p,
        planet: planet(),
    };
    let mut ebm = EbmState::default();
    let s = snapshot::compute(&inp, &mut ebm, &[]);
    assert!(s.ebm_net_toa.abs() < 0.3, "EBM balance {}", s.ebm_net_toa);
    let row = |deg: f64| {
        (0..h)
            .min_by(|&a, &b| (lat[a] - deg).abs().total_cmp(&(lat[b] - deg).abs()))
            .unwrap()
    };
    let col = |deg: f64| ((deg / 360.0 * w as f64).round() as usize) % w;
    let at = |v: &[f32], la: f64, lo: f64| f64::from(v[row(la) * w + col(lo)]);
    // Temperature: warm equator, frozen polar continent, the lapse rate on the range.
    assert!(
        at(&s.t_mean, 0.0, 250.0) > 22.0,
        "{}",
        at(&s.t_mean, 0.0, 250.0)
    );
    assert!(
        at(&s.t_mean, -80.0, 100.0) < -15.0,
        "{}",
        at(&s.t_mean, -80.0, 100.0)
    );
    assert!(at(&s.t_mean, 40.0, 72.0) < at(&s.t_mean, 40.0, 85.0) - 10.0);
    // Seasons: the continental interior's range is larger than the ocean's at 50° N.
    let range = |la, lo| at(&s.t_warm, la, lo) - at(&s.t_cold, la, lo);
    assert!(range(50.0, 120.0) > range(50.0, 250.0) + 10.0);
    // Precipitation: the ITCZ over the ocean is wetter than the subtropical high.
    assert!(at(&s.precip, 2.0, 250.0) > 2.0 * at(&s.precip, 25.0, 250.0));
    // The westerlies' rain shadow: the range's windward (west) side is wetter than its lee.
    let windward = at(&s.precip, 45.0, 68.0);
    let lee = at(&s.precip, 45.0, 80.0);
    assert!(windward > 1.5 * lee, "windward {windward} lee {lee}");
    // The interior is drier than the coasts the wind comes from.
    assert!(at(&s.precip, 45.0, 140.0) < windward);
    // Köppen: every main group on land; the polar continent is EF or ET and glaciated.
    let groups: std::collections::BTreeSet<char> = (0..w * h)
        .filter(|&c| z[c] > 0.0)
        .map(|c| koppen::group(s.koppen[c]))
        .collect();
    for g in ['A', 'B', 'C', 'D', 'E'] {
        assert!(groups.contains(&g), "no {g} climate: {groups:?}");
    }
    assert_eq!(koppen::group(s.koppen[row(-80.0) * w + col(100.0)]), 'E');
    assert_eq!(s.glacier[row(-80.0) * w + col(100.0)], 1);
    assert_eq!(s.koppen[row(0.0) * w + col(250.0)], koppen::SEA);
    assert!(s.ice.frac() > 0.05 && s.ice.lat < -60.0);
}

/// A small coupled planet: `n` L0 samples, L1 r = 64 at 16k (256 × 128), `myr` of history.
fn pipeline_json(n: u32, myr: f64, seed: u64) -> String {
    let json = with_timeline(
        planet_builtin("pipeline_quick_16k").unwrap(),
        Some("scattered_continents"),
        Some(myr),
    )
    .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["resolution"]["l0_samples"] = serde_json::json!(n);
    v["resolution"]["l1_r"] = serde_json::json!(64);
    v["keyframes"]["size"] = serde_json::json!([128, 64]);
    v["keyframes"]["every_myr"] = serde_json::json!(20.0);
    v["seed"] = serde_json::json!(seed);
    // The young landscape and L3 amplification (P1-06) have their own tests
    // (`surface::amplify::tests`); their L2 and L3 are far finer than this L1.
    v.as_object_mut().unwrap().remove("young");
    v.as_object_mut().unwrap().remove("amplify");
    v.to_string()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "deeptime-climate-{name}-{}-{:?}",
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
fn the_climate_runs_in_the_coupled_pipeline_and_resumes() {
    let json = pipeline_json(3000, 60.0, 7);
    let dir = temp_dir("straight");
    let s = session(&json, &dir, 2);
    assert_eq!(
        s.stage_names(),
        ["plates", "climate", "surface", "volcanoes", "structure", "deposits"]
    );
    finish(&s);
    let world = s.checkpoint().unwrap().world;
    let n = 256 * 128;
    for name in [
        field::T_MEAN,
        field::PRECIP,
        field::KOPPEN,
        field::T_WARM,
        field::T_COLD,
        field::ARIDITY,
        field::GLACIER,
        field::ELA,
    ] {
        let f = world.fields.get(name).unwrap_or_else(|| panic!("{name}"));
        assert_eq!(f.data.len(), n, "{name}");
    }
    let t = &world.fields[field::T_MEAN].data;
    let p = &world.fields[field::PRECIP].data;
    assert!(
        t.iter().all(|x| (-80.0..50.0).contains(x)),
        "temperatures in range"
    );
    assert!(
        p.iter().all(|&x| (10.0..20_000.0).contains(&x)),
        "precipitation in range"
    );
    let k = &world.fields[field::KOPPEN].data;
    assert!(
        k.iter()
            .all(|&x| (0.0..=31.0).contains(&x) && x.fract() == 0.0)
    );
    assert!(k.iter().any(|&x| x < 31.0), "some land is classified");
    // The surface read the stage's climate, and the deposit frames carry it.
    let st = world.surface.as_ref().unwrap();
    let grid = st.grid.clone();
    let d = Dims::new(256, 128, 64.0 * 300.0, true);
    let hh: Vec<f64> = st.elevation_m.iter().map(|&z| f64::from(z)).collect();
    let snap = crate::surface::climate::snapshot(
        &world,
        &d,
        &|j| lat_of_row(&grid, j as u32),
        &hh,
        st.sea_level_m,
    );
    assert!(snap.from_stage);
    // The last frame's precipitation (8-bit packed, averaged over the frame's steps) follows the
    // climate's, not the stand-in's.
    let hist = world.history.as_ref().unwrap();
    use crate::deposits::fields::FieldHistory;
    let frame = hist.frame(hist.frame_count() - 1).unwrap();
    let fp = &frame.fields["deeptime:precip_mm"];
    let mean = |v: &[f32]| v.iter().map(|&x| f64::from(x)).sum::<f64>() / v.len() as f64;
    let (mf, mc) = (mean(fp), mean(p));
    assert!(
        (mf - mc).abs() < 0.2 * mc,
        "frame {mf} vs climate {mc} mm/yr"
    );
    let note: serde_json::Value = serde_json::from_str(&world.notes[STAGE_NAME]).unwrap();
    assert_eq!(note["snapshots"], 7, "t = 2, 10, …, 50 and 60 Myr: {note}");

    // Resume from a checkpoint inside the group on another thread count: the same world.
    let dir2 = temp_dir("resumed");
    let s2 = session(&json, &dir2, 1);
    for _ in 0..7 {
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
    for name in [
        field::T_MEAN,
        field::PRECIP,
        field::KOPPEN,
        field::D_LN_PRECIP,
    ] {
        assert_eq!(
            got.fields[name], world.fields[name],
            "{name} differs after resuming"
        );
    }
    assert_eq!(
        got.surface.as_ref().unwrap().elevation_m,
        world.surface.as_ref().unwrap().elevation_m
    );
    assert_eq!(got.history, world.history);
    assert_eq!(got.events.len(), world.events.len());
    for d in [dir, dir2] {
        let _ = std::fs::remove_dir_all(d);
    }
}

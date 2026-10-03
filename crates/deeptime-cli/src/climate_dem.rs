// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime climate-dem`: one climate snapshot (P1-05, `deeptime_core::climate`) on a given
//! elevation raster, for checking the climate model against a known planet — Earth's present
//! relief, whose Köppen map everyone knows — away from the simulated history.
//!
//! The raster is raw little-endian f32, `w` × `h` cells, row 0 at the north pole, column i at
//! longitude 360°·i/w, metres relative to sea level. The planet is taken as Earth-sized (the
//! Earth-normalized lengths apply 1:1) unless `--circumference-km` says otherwise.

use std::path::PathBuf;
use std::time::Instant;

use deeptime_core::climate::snapshot::{self, Planet};
use deeptime_core::climate::{ClimateParams, ebm::EbmState, koppen};
use deeptime_core::surface::grid::Dims;
use deeptime_core::world::{Field, WorldState};
use deeptime_format::{GridDesc, Ramps};
use serde_json::Value;

use crate::run::CliResult;

pub struct Options {
    pub dem: PathBuf,
    pub w: u32,
    pub h: u32,
    /// A preset whose `climate` and `planet` sections apply (default: the defaults, Earth).
    pub preset: Option<String>,
    pub circumference_km: f64,
    pub viewer: Option<PathBuf>,
    pub dump: Option<PathBuf>,
}

/// Snapshots repeated on the relief when the preset turns the ice-sheet feedback on.
const FEEDBACK_SNAPSHOTS: usize = 5;

/// Places whose Köppen class is well known (Peel et al. 2007's map), for the printed check.
const PLACES: [(&str, f64, f64, &str); 24] = [
    ("London", 51.5, -0.1, "Cfb"),
    ("Moscow", 55.8, 37.6, "Dfb"),
    ("Yakutsk", 62.0, 129.7, "Dfd"),
    ("Cairo", 30.0, 31.2, "BWh"),
    ("Riyadh", 24.7, 46.7, "BWh"),
    ("Manaus", -3.1, -60.0, "Af"),
    ("Kinshasa", -4.3, 15.3, "Aw"),
    ("Mumbai", 19.1, 72.9, "Am"),
    ("Beijing", 39.9, 116.4, "Dwa"),
    ("Tokyo", 35.7, 139.7, "Cfa"),
    ("Sydney", -33.9, 151.2, "Cfa"),
    ("Perth", -31.9, 115.9, "Csa"),
    ("Rome", 41.9, 12.5, "Csa"),
    ("Denver", 39.7, -105.0, "BSk"),
    ("Chicago", 41.9, -87.6, "Dfa"),
    ("Anchorage", 61.2, -149.9, "Dfc"),
    ("Lima", -12.0, -77.0, "BWh"),
    ("Buenos Aires", -34.6, -58.4, "Cfa"),
    ("Cape Town", -33.9, 18.4, "Csb"),
    ("Singapore", 1.35, 103.8, "Af"),
    ("Alice Springs", -23.7, 133.9, "BWh"),
    ("Ulaanbaatar", 47.9, 106.9, "Dwb"),
    ("Lhasa", 29.7, 91.1, "Dwb"),
    ("Vostok", -78.5, 106.8, "EF"),
];

#[allow(clippy::needless_range_loop)]
pub fn run(o: &Options) -> CliResult<()> {
    let started = Instant::now();
    let (w, h) = (o.w as usize, o.h as usize);
    let bytes = std::fs::read(&o.dem)?;
    if bytes.len() != w * h * 4 {
        return Err(format!(
            "{}: {} bytes, expected {} × {} × 4",
            o.dem.display(),
            bytes.len(),
            w,
            h
        )
        .into());
    }
    let z: Vec<f64> = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f64::from(f32::from_le_bytes(*b)))
        .collect();
    let (mut p, mut tilt, mut flux, mut day) = (ClimateParams::default(), 23.44, 1.0, 24.0);
    if let Some(arg) = &o.preset {
        let v: Value = serde_json::from_str(&crate::run::load_preset_with(arg, None, None)?)?;
        if let Some(c) = v.get("climate") {
            p = serde_json::from_value(c.clone())?;
        }
        if let Some(pl) = v.get("planet") {
            tilt = pl["axial_tilt_deg"].as_f64().unwrap_or(tilt);
            flux = pl["star_flux_rel"].as_f64().unwrap_or(flux);
            day = pl["day_length_h"].as_f64().unwrap_or(day);
        }
    }
    // A planet grid of w cells around (r = 1 block per cell; block = circumference / w).
    let grid = GridDesc::for_planet("L1", o.w, 1)?;
    let cell_m = o.circumference_km * 1000.0 / f64::from(o.w);
    let lat: Vec<f64> = (0..h).map(|j| grid.band_lat_deg(j as u32)).collect();
    let planet = Planet {
        axial_tilt_deg: tilt,
        star_flux_rel: flux,
        day_length_h: day,
        earth_scale: o.circumference_km / (std::f64::consts::TAU * 6370.0),
        m_per_block_h: cell_m,
        // The present: 280 ppm, today's sun, and the ice-age window's present orbit when the
        // preset runs the window (the stage's last snapshot sees the same).
        forcing_w_m2: 0.0,
        sun_rel: 1.0,
        orbit: (p.orbital_window_myr > 0.0)
            .then(|| deeptime_core::climate::orbit::present(tilt, p.orbital_strength)),
    };
    let inp = snapshot::Input {
        grid: &grid,
        d: Dims::new(w, h, cell_m, true),
        lat: &lat,
        h: &z,
        sea: 0.0,
        p: &p,
        planet,
        monthly: o.dump.is_some(),
        ice_prev: None,
        koppen_prev: None,
    };
    let mut ebm = EbmState::default();
    let mut cell = deeptime_core::climate::percell::PerCell::default();
    let mut s = snapshot::compute(&inp, &mut ebm, &[], &mut cell);
    // With the ice-sheet feedback on (P1-17), the stage feeds each snapshot's ice sheets to the
    // next one's EBM; repeat the snapshot on the same relief until the ice settles, as the stage
    // would over this many 10-Myr snapshots.
    if p.ice_sheet_feedback > 0.0 {
        for k in 1..=FEEDBACK_SNAPSHOTS {
            println!(
                "deeptime climate-dem: ice-sheet feedback {:.2}, snapshot {k}: glaciated land {:.3}",
                p.ice_sheet_feedback,
                s.ice.frac()
            );
            let sheet = s.band_sheet.clone();
            // The per-cell climate reads the previous snapshot's glaciers and deserts.
            let ice: Vec<f32> = s.glacier.iter().map(|&g| f32::from(g) * 1000.0).collect();
            let kop: Vec<f32> = s.koppen.iter().map(|&k| f32::from(k)).collect();
            let again = snapshot::Input {
                ice_prev: Some(&ice),
                koppen_prev: Some(&kop),
                ..inp
            };
            s = snapshot::compute(&again, &mut ebm, &sheet, &mut cell);
        }
    }
    println!(
        "deeptime climate-dem: {}x{} cells of {:.1} km, EBM {:.2} °C global, net TOA {:.3} W/m² \
         after {} years, ITCZ {:.1}..{:.1}°, glaciated land {:.3}, {:.2}s",
        w,
        h,
        cell_m / 1000.0,
        s.ebm_t_global,
        s.ebm_net_toa,
        s.ebm_years,
        s.itcz_range.0,
        s.itcz_range.1,
        s.ice.frac(),
        started.elapsed().as_secs_f64()
    );
    // Land means (cos φ-weighted) and the main groups' shares of the land.
    let (mut la, mut lt, mut lp) = (0.0, 0.0, 0.0);
    let mut groups = std::collections::BTreeMap::new();
    for j in 0..h {
        let wgt = lat[j].to_radians().cos();
        for i in 0..w {
            let c = j * w + i;
            if z[c] > 0.0 {
                la += wgt;
                lt += wgt * f64::from(s.t_mean[c]);
                lp += wgt * f64::from(s.precip[c]);
                *groups.entry(koppen::group(s.koppen[c])).or_insert(0.0) += wgt;
            }
        }
    }
    println!(
        "deeptime climate-dem: land T {:.1} °C, P {:.0} mm/yr, groups {}",
        lt / la,
        lp / la,
        groups
            .iter()
            .map(|(g, a)| format!("{g}{:.2}", a / la))
            .collect::<Vec<_>>()
            .join("/")
    );
    // The carbon cycle's and the sea level's Earth references (deep-time climate): the weathering
    // kernel Σ a·runoff·e^(k_T(T − 15 °C)) over the land (km²·m/yr, true Earth-normalized areas;
    // `climate::carbon::KERNEL_EARTH`) and the grounded ice of Nye's profile on the glaciated land
    // (km³; `climate.ice_reference_km3`).
    {
        use deeptime_core::climate::{carbon, ice};
        let es = planet.earth_scale;
        let area: Vec<f64> = lat
            .iter()
            .map(|l| (cell_m / 1000.0).powi(2) * l.to_radians().cos().max(0.0) / (es * es))
            .collect();
        let (mut kernel, mut land_a) = (0.0, 0.0);
        for j in 0..h {
            for i in 0..w {
                let c = j * w + i;
                if z[c] <= 0.0 {
                    continue;
                }
                let t = f64::from(s.t_mean[c]);
                let pr = f64::from(s.precip[c]);
                let pet = (60.0 * t).max(0.0);
                let runoff =
                    (pr - deeptime_core::surface::climate::fu_et(pr, pet)).max(0.0) / 1000.0;
                kernel += carbon::cell_kernel(area[j], runoff, t, p.weathering_temp_per_k);
                land_a += area[j];
            }
        }
        let mask: Vec<bool> = (0..w * h)
            .map(|c| z[c] > 0.0 && s.glacier[c] != 0)
            .collect();
        let sx: Vec<f64> = lat
            .iter()
            .map(|l| cell_m * l.to_radians().cos().max(0.0) / es)
            .collect();
        let thick = ice::thickness(
            w,
            h,
            &mask,
            &sx,
            cell_m / es,
            p.ice_yield_stress_kpa * 1000.0,
            p.ice_max_thickness_m,
        );
        let vol: f64 = (0..w * h)
            .map(|c| f64::from(thick[c]) * area[c / w] / 1000.0)
            .sum();
        let ocean: f64 = (0..w * h)
            .filter(|&c| z[c] <= 0.0)
            .map(|c| area[c / w])
            .sum();
        println!(
            "deeptime climate-dem: weathering kernel {kernel:.4e} km²·m/yr over {land_a:.4e} km² of \
             land (KERNEL_EARTH {:.4e}); grounded ice {vol:.4e} km³ (reference {:.4e}), {:.1} m of \
             sea level, peak {:.0} m",
            carbon::KERNEL_EARTH,
            p.ice_reference_km3,
            -ice::eustatic_m(vol, ocean),
            thick.iter().cloned().fold(0.0f32, f32::max)
        );
    }
    // The flowing ice sheets (`climate::sheet`) on this relief under this climate's equilibrium
    // line: grown from bare ground at the present line, through a glacial lowering of the line and
    // back — the sheet's Earth check (Earth: ~10 % of the land under ice today, ~25 % at the last
    // glacial maximum, and the northern sheets gone again 10 kyr after it).
    if p.ice_dynamics {
        use deeptime_core::climate::sheet;
        let es = planet.earth_scale;
        let sx: Vec<f64> = lat
            .iter()
            .map(|l| cell_m * l.to_radians().cos().max(0.0) / es)
            .collect();
        let land: Vec<bool> = z.iter().map(|&x| x > 0.0).collect();
        // Tuning overrides (dev): DT_ISO kyr, DT_CALV m/yr, DT_ABL and DT_ACC per m, DT_CELLS.
        let env = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<f64>().ok());
        let cells = env("DT_CELLS").map_or(p.ice_grid_cells as usize, |x| x as usize);
        let sg = sheet::Grid::new(w, h, &sx, cell_m / es, cells);
        let mut sp = sheet::Params::from(&p);
        if let Some(x) = env("DT_ISO") {
            sp.isostasy_years = x * 1000.0;
        }
        if let Some(x) = env("DT_CALV") {
            sp.calving = x;
        }
        if let Some(x) = env("DT_ABL") {
            sp.beta_abl = x;
        }
        if let Some(x) = env("DT_ACC") {
            sp.beta_acc = x;
        }
        println!("deeptime climate-dem: sheet parameters {sp:?}");
        let bed: Vec<f32> = z.iter().map(|&x| x as f32).collect();
        let ela: Vec<f32> = s
            .ela
            .iter()
            .map(|&e| if e.is_finite() { e } else { 1.0e6 })
            .collect();
        let precip: Vec<f32> = s.precip.iter().map(|&x| x / 1000.0).collect();
        let bed_c = sg.coarsen(&bed, &|_| true);
        let ela_c = sg.coarsen(&ela, &|c| land[c]);
        let precip_c = sg.coarsen(&precip, &|c| land[c]);
        let mut st = sheet::State::default();
        for (label, shift, kyr) in [
            ("the present line, from bare ground", 0.0f32, 100.0),
            ("the line 500 m lower", -500.0, 60.0),
            ("the line 1000 m lower", -1000.0, 60.0),
            (
                "the line 300 m higher (an early-interglacial summer), 10 kyr",
                300.0,
                10.0,
            ),
            ("the present line again, 30 kyr", 0.0, 30.0),
            ("a second glacial, the line 1000 m lower", -1000.0, 60.0),
            ("the present line at once, 10 kyr", 0.0, 10.0),
            ("the present line, 100 kyr more", 0.0, 100.0),
        ] {
            let t0 = Instant::now();
            let ela_now: Vec<f32> = ela_c.iter().map(|e| e + shift).collect();
            let steps = st.step(
                &sg,
                &sheet::Forcing {
                    bed: &bed_c,
                    ela: &ela_now,
                    precip: &precip_c,
                },
                &sp,
                kyr * 1000.0,
            );
            let fine = sg.refine(&st.thick);
            let (mut all, mut ice, mut north_all, mut north_ice) = (0.0, 0.0, 0.0, 0.0);
            for j in 0..h {
                let wgt = lat[j].to_radians().cos();
                for i in 0..w {
                    let c = j * w + i;
                    if !land[c] {
                        continue;
                    }
                    let on = if fine[c] >= 1.0 { wgt } else { 0.0 };
                    all += wgt;
                    ice += on;
                    if lat[j] > 0.0 {
                        north_all += wgt;
                        north_ice += on;
                    }
                }
            }
            println!(
                "deeptime climate-dem: sheet, {label}: {:.3} of the land under ice ({:.3} of the \
                 north's), {:.3e} km³, peak {:.0} m, {steps} steps, {:.1}s",
                ice / all,
                north_ice / north_all,
                st.volume_m3(&sg) / 1.0e9,
                st.thick.iter().cloned().fold(0.0f32, f32::max),
                t0.elapsed().as_secs_f64()
            );
        }
    }
    if let Some(dir) = &o.dump {
        std::fs::create_dir_all(dir)?;
        let write = |name: &str, v: &[f32]| -> std::io::Result<()> {
            let mut b = Vec::with_capacity(v.len() * 4);
            for x in v {
                b.extend_from_slice(&x.to_le_bytes());
            }
            std::fs::write(dir.join(format!("{name}.f32")), b)
        };
        for m in 0..12 {
            if let (Some(t), Some(pm)) = (s.t_sl_month.get(m), s.p_month.get(m)) {
                write(&format!("t_sl_{:02}", m + 1), t)?;
                write(&format!("p_{:02}", m + 1), pm)?;
            }
        }
        write("t_mean", &s.t_mean)?;
        write("t_warm", &s.t_warm)?;
        write("t_cold", &s.t_cold)?;
        write("precip", &s.precip)?;
        write("sea_ice", &s.sea_ice)?;
        write("sea_ice_max", &s.sea_ice_max)?;
        write("sea_ice_min", &s.sea_ice_min)?;
        write(
            "koppen",
            &s.koppen.iter().map(|&k| f32::from(k)).collect::<Vec<_>>(),
        )?;
        write(
            "glacier",
            &s.glacier.iter().map(|&g| f32::from(g)).collect::<Vec<_>>(),
        )?;
        let meta = serde_json::json!({
            "w": w, "h": h, "model": p.model, "rain_model": p.rain_model,
            "t_global_c": s.ebm_t_global, "net_toa_w_m2": s.ebm_net_toa, "years": s.ebm_years,
            "glaciated_land": s.ice.frac(), "per_cell": s.cell,
            "koppen_codes": koppen::CODES,
        });
        std::fs::write(dir.join("meta.json"), serde_json::to_string_pretty(&meta)?)?;
        println!("deeptime climate-dem: fields -> {}", dir.display());
    }
    // The printed check: the class at each known place, and whether its group matches.
    let (mut same, mut group) = (0, 0);
    for (name, la, lo, want) in PLACES {
        let j = (deeptime_format::Projection::picture_y_of_lat(la, h as u32).floor() as usize)
            .min(h - 1);
        let i = ((lo.rem_euclid(360.0) / 360.0 * w as f64).floor() as usize) % w;
        // Coastal cities can fall in a sea cell at this resolution: take the nearest land cell
        // within two cells.
        let mut c = j * w + i;
        if s.koppen[c] == koppen::SEA {
            let mut best = (i64::MAX, c);
            for dj in -2i64..=2 {
                for di in -2i64..=2 {
                    let jj = (j as i64 + dj).clamp(0, h as i64 - 1) as usize;
                    let ii = (i as i64 + di).rem_euclid(w as i64) as usize;
                    let cc = jj * w + ii;
                    let d2 = di * di + dj * dj;
                    if s.koppen[cc] != koppen::SEA && d2 < best.0 {
                        best = (d2, cc);
                    }
                }
            }
            c = best.1;
        }
        let got = koppen::CODES[s.koppen[c] as usize];
        let g = got.chars().next() == want.chars().next();
        same += usize::from(got == want);
        group += usize::from(g);
        println!(
            "deeptime climate-dem: {name:<14} {want:<4} → {got:<4} {} (T {:5.1} °C, {:6.0} mm, z {:5.0} m)",
            if got == want {
                "same"
            } else if g {
                "group"
            } else {
                "differs"
            },
            s.t_mean[c],
            s.precip[c],
            z[c]
        );
    }
    println!(
        "deeptime climate-dem: places {} of {} same class, {} same main group",
        same,
        PLACES.len(),
        group
    );
    if let Some(dir) = &o.viewer {
        let mut world = WorldState::new(1, o.w, 0.0);
        let put = |world: &mut WorldState, name: &str, data: Vec<f32>| {
            world.fields.insert(
                name.into(),
                Field {
                    grid: grid.clone(),
                    data,
                },
            );
        };
        // Elevation in block y at 20 m per block, sea level y 63 (the viewer's ramp).
        put(
            &mut world,
            "elevation",
            z.iter().map(|&m| (63.0 + m / 20.0) as f32).collect(),
        );
        put(&mut world, "climate/t_mean_c", s.t_mean.clone());
        put(&mut world, "climate/t_warm_c", s.t_warm.clone());
        put(&mut world, "climate/t_cold_c", s.t_cold.clone());
        put(&mut world, "climate/precip_mm", s.precip.clone());
        put(&mut world, "climate/aridity", s.aridity.clone());
        put(
            &mut world,
            "climate/koppen",
            s.koppen.iter().map(|&k| f32::from(k)).collect(),
        );
        put(
            &mut world,
            "climate/glacier",
            s.glacier.iter().map(|&g| f32::from(g)).collect(),
        );
        put(&mut world, "climate/ela_m", s.ela.clone());
        // The satellite view of the relief under this climate (its land cover, snow and glaciers).
        let layers = [
            "satellite",
            "climate/koppen",
            "climate/t_mean_c",
            "climate/precip_mm",
            "climate/t_warm_c",
            "climate/t_cold_c",
            "climate/aridity",
            "elevation",
        ];
        let frame = deeptime_core::keyframes::render(
            &world,
            &Ramps::builtin(),
            &deeptime_core::keyframes::ViewScale::default(),
            "climate",
            &layers,
            0,
            (o.w, o.h),
        )
        .expect("layers exist");
        let info = deeptime_viewer::BundleInfo {
            title: "Deep Time — climate on a given relief".into(),
            source: format!("climate-dem:{}", o.dem.display()),
            circumference: o.w,
            sea_level_y: 63,
            format_version: deeptime_format::FORMAT_VERSION.to_owned(),
            created_utc: deeptime_format::manifest::rfc3339_utc(std::time::SystemTime::now()),
        };
        deeptime_viewer::write_bundle_with(
            dir,
            &info,
            &[],
            None,
            &deeptime_viewer::Timeline {
                keyframes: vec![frame],
                events: Vec::new(),
                sites: Vec::new(),
            },
        )?;
        println!(
            "deeptime climate-dem: viewer -> {}",
            dir.join("index.html").display()
        );
    }
    Ok(())
}

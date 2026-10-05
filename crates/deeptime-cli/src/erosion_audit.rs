// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime erosion-audit`: how much rock the surface stage took off and laid on, per region type,
//! and where that left the porphyries (decisions log 2026-10-05, "Investigate and fix").
//!
//! From a checkpoint that carries the recorded history (`run --checkpoints`), every frame's
//! erosion and deposition rates as the deposit ledger reads them (`deposits::fields`):
//!
//! - **rates per region type**, area-weighted over every frame: land is an *orogen* where the
//!   plate model lifted continental crust that frame (`orogenic_uplift_m_myr` > 1 m/Myr), a
//!   *basin* where the frame laid down more than it took off, a *shield* where the crust is
//!   stable (`fault_density` < 0.15: no active regime and the last orogeny > ~70 Myr ago), else
//!   *other land*; sea floor over continental crust (> 25 km) is *shelf*;
//! - **totals per crust parcel**, followed with the crust through the frames exactly as the
//!   ledger follows a deposit (`origins`): the erosion and deposition the crust now at each cell
//!   saw over the whole run and over its last 500 Myr, grouped by the parcel's region type in the
//!   last frame;
//! - **hypsometry** of the deep-time surface at the end.
//!
//! From a world-data folder, every porphyry's ledger (`deposit_formation`: formation depth,
//! `eroded_m`, `buried_m`, `exhumed_max_m`, fate) and each surviving site's centre depth under
//! the ground.
//!
//! Earth's rates are reference lines to read the numbers against, never targets (decisions log
//! 2026-10-05): cosmogenic outcrops 12 m/Myr mean, 5.4 median; drainage basins 218 mean, 54
//! median, seismically active 367, inactive 182 (Portenga & Bierman 2011; ledger: ERO-171);
//! quiet land about 0.2 mm/yr per km of local relief, active ranges above it (Montgomery &
//! Brandon 2002; ledger: ERO-172).

use std::collections::BTreeMap;
use std::path::Path;

use deeptime_core::deposits::fields::{FieldHistory, NO_ORIGIN};
use deeptime_format::events::ParamValue;
use deeptime_format::sites::read_sites;
use deeptime_format::{ColumnWorld, EventLog, Manifest};
use serde_json::{Value, json};

/// Lower ends of the land elevation bands, m above the sea of the time.
const BANDS_M: [f64; 5] = [0.0, 1000.0, 2000.0, 3000.0, 4000.0];

/// Region types, in report order.
const CLASSES: [&str; 6] = ["orogen", "basin", "shield", "other_land", "shelf", "ocean"];

/// A frame's region type of one cell (see the module doc; diagnostic classes, ledger: ERO-173).
fn classify(elev_m: f32, thick_km: f32, uplift: f32, fault: f32, e: f32, d: f32) -> usize {
    let continental = thick_km > 25.0;
    if elev_m <= 0.0 {
        return if continental { 4 } else { 5 };
    }
    if uplift > 1.0 {
        0
    } else if d > e {
        1
    } else if fault < 0.15 {
        2
    } else {
        3
    }
}

/// An area-weighted histogram of non-negative values on log bins (1 cm … 1000 km), for
/// percentiles without keeping every value.
#[derive(Clone)]
struct Hist {
    zero: f64,
    bins: Vec<f64>,
    total: f64,
    sum: f64,
}

const BINS_PER_DECADE: f64 = 40.0;
const LO: f64 = 0.01;
const DECADES: f64 = 8.0;

impl Hist {
    fn new() -> Hist {
        Hist {
            zero: 0.0,
            bins: vec![0.0; (BINS_PER_DECADE * DECADES) as usize + 1],
            total: 0.0,
            sum: 0.0,
        }
    }

    fn add(&mut self, v: f64, w: f64) {
        if !v.is_finite() || w <= 0.0 {
            return;
        }
        self.total += w;
        self.sum += v * w;
        if v < LO {
            self.zero += w;
            return;
        }
        let b = ((v / LO).log10() * BINS_PER_DECADE).floor() as usize;
        let b = b.min(self.bins.len() - 1);
        self.bins[b] += w;
    }

    fn mean(&self) -> f64 {
        if self.total > 0.0 {
            self.sum / self.total
        } else {
            f64::NAN
        }
    }

    fn quantile(&self, q: f64) -> f64 {
        if self.total <= 0.0 {
            return f64::NAN;
        }
        let want = q * self.total;
        let mut acc = self.zero;
        if acc >= want {
            return 0.0;
        }
        for (b, w) in self.bins.iter().enumerate() {
            acc += w;
            if acc >= want {
                // The bin's geometric middle.
                return LO * 10f64.powf((b as f64 + 0.5) / BINS_PER_DECADE);
            }
        }
        LO * 10f64.powf(DECADES)
    }

    fn share_above(&self, v: f64) -> f64 {
        if self.total <= 0.0 {
            return f64::NAN;
        }
        let b0 = ((v / LO).log10() * BINS_PER_DECADE).floor().max(0.0) as usize;
        self.bins[b0.min(self.bins.len())..].iter().sum::<f64>() / self.total
    }

    fn json(&self, unit_scale: f64) -> Value {
        let r = |x: f64| (x * unit_scale * 10.0).round() / 10.0;
        json!({
            "area_share": self.total,
            "mean": r(self.mean()),
            "p50": r(self.quantile(0.5)),
            "p90": r(self.quantile(0.9)),
            "p99": r(self.quantile(0.99)),
            "max_bin": r(self.quantile(1.0)),
        })
    }
}

fn get<'a>(f: &'a deeptime_core::deposits::fields::FieldFrame, name: &str) -> Option<&'a [f32]> {
    f.fields.get(name).map(|v| v.as_slice())
}

/// The history part (see the module doc).
pub fn history(ckpt: &Path) -> Result<Value, String> {
    let ck = deeptime_core::checkpoint::Checkpoint::read(ckpt).map_err(|e| e.to_string())?;
    let preset: Value = serde_json::from_str(&ck.preset_json).map_err(|e| e.to_string())?;
    let m_per_block_h = preset["scale"]["m_per_block_h"].as_f64().unwrap_or(300.0);
    let hist =
        ck.world.history.as_ref().ok_or(
            "the checkpoint has no recorded history (a checkpoint after the surface stage)",
        )?;
    let grid = hist.grid().clone();
    let (w, h) = (grid.w as usize, grid.h as usize);
    let n = w * h;
    let area: Vec<f64> = (0..n)
        .map(|c| {
            deeptime_core::deposits::fields::cell_area_km2(&grid, (c / w) as u32, m_per_block_h)
        })
        .collect();
    let frames = hist.frame_count();
    let t_end = hist.frame(frames - 1).map_err(|e| e.to_string())?.t_years;

    // Per class: rates (m/Myr) of erosion and deposition, area × time weighted.
    let mut ero: Vec<Hist> = vec![Hist::new(); CLASSES.len()];
    // Land erosion by the frame's elevation band (how fast high ground, plateaus included, wears).
    let mut by_band: Vec<Hist> = vec![Hist::new(); BANDS_M.len()];
    let mut dep: Vec<Hist> = vec![Hist::new(); CLASSES.len()];
    // Per crust parcel, carried: totals over the run and over its last 500 Myr (m), frames as orogen.
    let mut cum_e = vec![0.0f64; n];
    let mut cum_d = vec![0.0f64; n];
    let mut cum_e500 = vec![0.0f64; n];
    let mut cum_d500 = vec![0.0f64; n];
    let mut last_class = vec![5usize; n];
    let mut last_elev = vec![f32::NAN; n];
    let mut peak = vec![0.0f32; n];
    let mut span_myr = 0.0;
    for k in 0..frames {
        let f = hist.frame(k).map_err(|e| e.to_string())?;
        if let Some(origin) = hist.origins(k).map_err(|e| e.to_string())? {
            for v in [&mut cum_e, &mut cum_d, &mut cum_e500, &mut cum_d500] {
                let old = std::mem::take(v);
                *v = origin
                    .iter()
                    .map(|&q| if q == NO_ORIGIN { 0.0 } else { old[q as usize] })
                    .collect();
            }
            let old = std::mem::take(&mut peak);
            peak = origin
                .iter()
                .map(|&q| if q == NO_ORIGIN { 0.0 } else { old[q as usize] })
                .collect();
        }
        let dt_myr = f.dt_years / 1.0e6;
        span_myr += dt_myr;
        let recent = f.t_years > t_end - 500.0e6;
        let (Some(e), Some(d), Some(z)) = (
            get(&f, "deeptime:erosion_rate_m_myr"),
            get(&f, "deeptime:deposition_rate_m_myr"),
            get(&f, "deeptime:elevation_m"),
        ) else {
            return Err(format!(
                "frame {k} lacks the erosion, deposition or elevation field"
            ));
        };
        let zero = vec![0.0f32; n];
        let thick = get(&f, "deeptime:crust_thickness_km").unwrap_or(&zero);
        let upl = get(&f, "deeptime:orogenic_uplift_m_myr").unwrap_or(&zero);
        let fault = get(&f, "deeptime:fault_density").unwrap_or(&zero);
        for c in 0..n {
            let (ec, dc) = (e[c].max(0.0), d[c].max(0.0));
            let cl = classify(z[c], thick[c], upl[c], fault[c], ec, dc);
            let wgt = area[c] * dt_myr;
            ero[cl].add(f64::from(ec), wgt);
            if z[c] > 0.0 {
                let b = BANDS_M
                    .iter()
                    .rposition(|&lo| f64::from(z[c]) >= lo)
                    .unwrap_or(0);
                by_band[b].add(f64::from(ec), wgt);
            }
            dep[cl].add(f64::from(dc), wgt);
            cum_e[c] += f64::from(ec) * dt_myr;
            cum_d[c] += f64::from(dc) * dt_myr;
            if recent {
                cum_e500[c] += f64::from(ec) * dt_myr;
                cum_d500[c] += f64::from(dc) * dt_myr;
            }
            peak[c] = peak[c].max(ec);
            if k + 1 == frames {
                last_class[c] = cl;
                last_elev[c] = z[c];
            }
        }
    }
    let total_w: f64 = area.iter().sum::<f64>() * span_myr;
    let mut rates = serde_json::Map::new();
    for (i, name) in CLASSES.iter().enumerate() {
        let mut e = ero[i].json(1.0);
        e["area_share"] = json!((ero[i].total / total_w * 1000.0).round() / 1000.0);
        rates.insert(
            (*name).into(),
            json!({
                "erosion_m_myr": e,
                "deposition_m_myr": dep[i].json(1.0),
                "erosion_share_above_100_m_myr": (ero[i].share_above(100.0) * 1000.0).round() / 1000.0,
                "erosion_share_above_500_m_myr": (ero[i].share_above(500.0) * 1000.0).round() / 1000.0,
            }),
        );
    }
    // Totals per parcel, by the parcel's region type in the last frame (km).
    let mut totals = serde_json::Map::new();
    for (i, name) in CLASSES.iter().enumerate() {
        let mut he = Hist::new();
        let mut hd = Hist::new();
        let mut he5 = Hist::new();
        let mut hd5 = Hist::new();
        let mut hp = Hist::new();
        let mut over10 = 0.0;
        let mut a = 0.0;
        for c in 0..n {
            if last_class[c] != i {
                continue;
            }
            a += area[c];
            he.add(cum_e[c], area[c]);
            hd.add(cum_d[c], area[c]);
            he5.add(cum_e500[c], area[c]);
            hd5.add(cum_d500[c], area[c]);
            hp.add(f64::from(peak[c]), area[c]);
            if cum_e[c] > 10_000.0 {
                over10 += area[c];
            }
        }
        totals.insert(
            (*name).into(),
            json!({
                "eroded_km": he.json(1.0e-3),
                "deposited_km": hd.json(1.0e-3),
                "eroded_last_500_myr_km": he5.json(1.0e-3),
                "deposited_last_500_myr_km": hd5.json(1.0e-3),
                "peak_frame_erosion_m_myr": hp.json(1.0),
                "share_eroded_over_10_km": if a > 0.0 { (over10 / a * 1000.0).round() / 1000.0 } else { f64::NAN },
            }),
        );
    }
    // Hypsometry of the deep-time surface at the end (the stage's `elevation` field, blocks, read
    // back to metres above the sea of the time), not a frame's time mean.
    let sea_y = preset["scale"]["sea_level_y"].as_f64().unwrap_or(63.0);
    let m_v = preset["scale"]["m_per_block_v"].as_f64().unwrap_or(35.0);
    let final_z: Vec<f32> = match ck
        .world
        .fields
        .get("elevation")
        .filter(|f| f.data.len() == n)
    {
        Some(f) => f
            .data
            .iter()
            .map(|&y| ((f64::from(y) - sea_y) * m_v) as f32)
            .collect(),
        None => last_elev.clone(),
    };
    let mut land = Hist::new();
    let mut land_a = 0.0;
    let mut all_a = 0.0;
    let mut above = [0.0f64; 5];
    for c in 0..n {
        all_a += area[c];
        let z = final_z[c];
        if z.is_finite() && z > 0.0 {
            land_a += area[c];
            land.add(f64::from(z), area[c]);
            for (k, lim) in [1000.0, 2000.0, 3000.0, 4000.0, 5000.0].iter().enumerate() {
                if f64::from(z) > *lim {
                    above[k] += area[c];
                }
            }
        }
    }
    let r3 = |x: f64| (x * 1000.0).round() / 1000.0;
    Ok(json!({
        "checkpoint": ckpt.display().to_string(),
        "frames": frames,
        "span_myr": span_myr,
        "rates_by_region": rates,
        "land_erosion_by_elevation_m": BANDS_M
            .iter()
            .enumerate()
            .map(|(i, lo)| {
                let mut v = by_band[i].json(1.0);
                v["area_share"] = json!((by_band[i].total / total_w * 1000.0).round() / 1000.0);
                (format!("from_{lo}"), v)
            })
            .collect::<serde_json::Map<String, Value>>(),
        "parcel_totals_by_final_region": totals,
        "hypsometry": {
            "land_share": r3(land_a / all_a),
            "land_elevation_m": land.json(1.0),
            "land_share_above_m": {
                "1000": r3(above[0] / land_a), "2000": r3(above[1] / land_a),
                "3000": r3(above[2] / land_a), "4000": r3(above[3] / land_a),
                "5000": r3(above[4] / land_a),
            },
        },
    }))
}

fn num(p: &BTreeMap<String, ParamValue>, k: &str) -> Option<f64> {
    match p.get(k)? {
        ParamValue::Float(v) => Some(*v),
        ParamValue::Int(v) => Some(*v as f64),
        _ => None,
    }
}

fn text<'a>(p: &'a BTreeMap<String, ParamValue>, k: &str) -> Option<&'a str> {
    match p.get(k)? {
        ParamValue::Str(s) => Some(s.as_str()),
        _ => None,
    }
}

/// The porphyry part (see the module doc).
pub fn porphyries(world: &Path) -> Result<Value, String> {
    let m = Manifest::read(world).map_err(|e| e.to_string())?;
    let m_v = m.scale.m_per_block_v;
    let log = EventLog::load(world).map_err(|e| e.to_string())?;
    let sites = read_sites(world)
        .map_err(|e| e.to_string())?
        .map(|(s, _)| s)
        .unwrap_or_default();
    let cols = ColumnWorld::open(world).map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    let mut eroded_km = Vec::new();
    let mut fates: BTreeMap<String, usize> = BTreeMap::new();
    for e in log.of_kind("deeptime:deposit_formation") {
        let ty = text(&e.params, "deposit_type").unwrap_or("");
        if !ty.contains("porphyry") {
            continue;
        }
        let fate = text(&e.params, "fate").unwrap_or("?").to_owned();
        *fates.entry(fate.clone()).or_default() += 1;
        let site = num(&e.params, "site").unwrap_or(-1.0) as i64;
        let er = num(&e.params, "eroded_m");
        let bu = num(&e.params, "buried_m");
        let zf = num(&e.params, "formation_depth_m").unwrap_or(f64::NAN);
        let mut depth_blocks = Value::Null;
        if site >= 0
            && let Some(s) = sites.iter().find(|s| i64::from(s.id) == site)
        {
            let col = cols.sample(i64::from(s.center[0]), i64::from(s.center[2]));
            depth_blocks = json!(((col.surface - f64::from(s.center[1])) * 10.0).round() / 10.0);
        }
        if let Some(x) = er {
            eroded_km.push(x / 1000.0);
        }
        rows.push(json!({
            "site": site,
            "formed_ma": (-e.t_start / 1.0e6 * 10.0).round() / 10.0,
            "fate": fate,
            "formation_depth_m": zf,
            "eroded_m": er.map(|x| x.round()),
            "buried_m": bu.map(|x| x.round()),
            "exhumed_max_m": num(&e.params, "exhumed_max_m").map(|x| x.round()),
            "ledger_centre_depth_m": match (er, bu) {
                (Some(er), Some(bu)) => json!((zf + bu - er).round()),
                _ => Value::Null,
            },
            "centre_depth_blocks": depth_blocks,
            "centre_depth_m_from_blocks": depth_blocks.as_f64().map(|b| (b * m_v).round()),
        }));
    }
    // Destroyed ones carry eroded_m on their exhumation event (also before formation events did).
    let mut exhumed_km: Vec<f64> = log
        .of_kind("deeptime:deposit_exhumation")
        .filter(|e| {
            text(&e.params, "deposit_type")
                .unwrap_or("")
                .contains("porphyry")
        })
        .filter_map(|e| num(&e.params, "eroded_m"))
        .map(|x| x / 1000.0)
        .collect();
    exhumed_km.sort_by(f64::total_cmp);
    eroded_km.sort_by(f64::total_cmp);
    let q = |v: &[f64], p: f64| -> Value {
        if v.is_empty() {
            Value::Null
        } else {
            json!((v[((v.len() - 1) as f64 * p).round() as usize] * 100.0).round() / 100.0)
        }
    };
    Ok(json!({
        "world_data": world.display().to_string(),
        "porphyries": rows.len(),
        "fates": fates,
        "eroded_km_p10_p50_p90": [q(&eroded_km, 0.1), q(&eroded_km, 0.5), q(&eroded_km, 0.9)],
        "exhumation_eroded_km_p10_p50_p90": [q(&exhumed_km, 0.1), q(&exhumed_km, 0.5), q(&exhumed_km, 0.9)],
        "rows": rows,
    }))
}

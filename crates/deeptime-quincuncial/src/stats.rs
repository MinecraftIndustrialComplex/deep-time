// SPDX-License-Identifier: LGPL-3.0-or-later
//! Measurements of a quincuncial world for the design doc (PROTOTYPE): where the singular points
//! and poles are, the spread of the area scale and where its extremes sit, blocks per degree, and
//! what an east–west walk along the equator meets.

use std::path::Path;

use deeptime_format::{Manifest, Projection, RasterRead, TilePackReader};
use serde_json::{Value, json};

use crate::world::{QWorld, unit_of_lat_lon};

fn quantile(sorted: &[(f64, f64)], total: f64, q: f64) -> f64 {
    let mut acc = 0.0;
    for &(v, w) in sorted {
        acc += w;
        if acc >= q * total {
            return v;
        }
    }
    sorted.last().map_or(f64::NAN, |p| p.0)
}

fn r3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

/// The measurements, as JSON. `n` is the sampling grid's cells along the domain's long side.
// ledger: SCL-075 (the numbers it prints)
pub fn measure(w: &QWorld, n: usize) -> Value {
    let (x0, x1, z0, z1) = w.extent();
    let step = (x1 - x0) / n as f64;
    let nz = ((z1 - z0) / step).round() as usize;
    // Magnification m = blocks per planet area relative to the planet's mean (1/area_scale_rel),
    // weighted by the planet area each sample holds.
    let mut samples = Vec::with_capacity(n * nz);
    let mut bands = [(0.0f64, 0.0f64); 6];
    let (mut min_m, mut min_at) = (f64::INFINITY, (0.0, 0.0));
    for i in 0..n {
        for j in 0..nz {
            let (x, z) = (x0 + (i as f64 + 0.5) * step, z0 + (j as f64 + 0.5) * step);
            let a = w.area_scale_rel(x, z);
            let m = 1.0 / a;
            samples.push((m, a));
            let (lat, _) = w.lat_lon(x, z);
            let b = ((lat.abs() / 15.0) as usize).min(5);
            bands[b].0 += m * a;
            bands[b].1 += a;
            if m < min_m {
                min_m = m;
                min_at = (x, z);
            }
        }
    }
    let total: f64 = samples.iter().map(|s| s.1).sum();
    samples.sort_by(|a, b| a.0.total_cmp(&b.0));
    let median = quantile(&samples, total, 0.5);
    let within2: f64 = samples
        .iter()
        .filter(|s| s.0 <= 2.0 * median && s.0 >= 0.5 * median)
        .map(|s| s.1)
        .sum();
    let (lat_min, lon_min) = w.lat_lon(min_at.0, min_at.1);

    // Blocks per degree at the named places.
    let bpd = |lat: f64, lon: f64| {
        let (x, z) = w.block_of(lat, lon);
        json!({"lat": lat, "lon": lon, "block": [r3(x), r3(z)], "blocks_per_degree": r3(w.blocks_per_degree(x, z))})
    };
    let singular: Vec<Value> = w
        .singular_points()
        .into_iter()
        .map(|(x, z)| {
            let (lat, lon) = w.lat_lon(x, z);
            // Blocks per degree on a ring 1° and 5° of planet away (it grows without bound inward).
            let ring = |deg: f64| {
                let (bx, bz) = w.block_of(lat + deg, lon);
                json!({"deg": deg, "blocks_from_point": r3((bx - x).hypot(bz - z)), "blocks_per_degree": r3(w.blocks_per_degree(bx, bz))})
            };
            json!({"block": [x, z], "lat": r3(lat), "lon": r3(lon), "rings": [ring(1.0), ring(5.0)]})
        })
        .collect();

    // The equator walked east: its length in blocks, the seams it crosses (consecutive points far
    // apart), the singular points it passes and the spread of its scale.
    let steps = 36_000;
    let mut length = 0.0;
    let mut crossings = 0;
    let mut eq_bpd: Vec<f64> = Vec::new();
    let mut prev: Option<(f64, f64)> = None;
    for k in 0..=steps {
        let lon = -180.0 + 360.0 * k as f64 / steps as f64;
        let (x, z) = w.block_of(0.0, lon);
        if let Some((px, pz)) = prev {
            let d = (x - px).hypot(z - pz);
            if d > 0.05 * w.size() {
                crossings += 1;
            } else {
                length += d;
            }
        }
        eq_bpd.push(w.blocks_per_degree(x, z));
        prev = Some((x, z));
    }
    eq_bpd.sort_by(f64::total_cmp);
    let eq_finite: Vec<f64> = eq_bpd.iter().copied().filter(|v| v.is_finite()).collect();
    // Is the equator one straight world line? Fit the deviation from the line z = const.
    let zs: Vec<f64> = (0..360)
        .map(|k| w.block_of(0.0, -179.5 + k as f64).1)
        .collect();
    let straight = zs.iter().all(|z| (z - zs[0]).abs() < 1e-6 * w.size());

    json!({
        "layout": w.layout().name(),
        "size": w.size(),
        "peirce_side": r3(w.peirce_side()),
        "lon_deg": w.lon_deg(),
        "domain": [x0, x1, z0, z1],
        "domain_blocks": r3(w.domain_area()),
        "singular_points": singular,
        "places": {
            "north_pole": bpd(90.0, 0.0),
            "south_pole": bpd(-90.0, 0.0),
            "equator_lon0": bpd(0.0, w.lon_deg()),
            "equator_lon0+45": bpd(0.0, w.lon_deg() + 45.0),
            "equator_lon0+90": bpd(0.0, w.lon_deg() + 90.0),
            "lat45_lon0+45": bpd(45.0, w.lon_deg() + 45.0),
            "lat60_lon0": bpd(60.0, w.lon_deg()),
        },
        "magnification": {
            "about": "blocks per unit planet area relative to the planet's mean over the domain, quantiles weighted by planet area; Inf at the singular points",
            "min": r3(min_m),
            "min_at": {"block": [r3(min_at.0), r3(min_at.1)], "lat": r3(lat_min), "lon": r3(lon_min)},
            "q01": r3(quantile(&samples, total, 0.01)),
            "q10": r3(quantile(&samples, total, 0.10)),
            "q50": r3(median),
            "q90": r3(quantile(&samples, total, 0.90)),
            "q99": r3(quantile(&samples, total, 0.99)),
            "q999": r3(quantile(&samples, total, 0.999)),
            "share_within_2x_of_median": r3(within2 / total),
            "by_abs_latitude_15deg_bands": bands.iter().map(|b| r3(b.0 / b.1)).collect::<Vec<_>>(),
        },
        "equator_walk": {
            "length_blocks": r3(length),
            "seam_crossings": crossings,
            "straight_line": straight,
            "blocks_per_degree_min": r3(eq_finite.first().copied().unwrap_or(f64::NAN)),
            "blocks_per_degree_median": r3(eq_finite[eq_finite.len() / 2]),
            "blocks_per_degree_max_finite": r3(eq_finite.last().copied().unwrap_or(f64::NAN)),
            "passes_singular_points": eq_bpd.iter().filter(|v| !v.is_finite() || **v > 1e6).count() > 0,
        },
        "seams": w.seams().iter().map(|s| s.portal(64.0, 1200.0)).collect::<Vec<_>>(),
        "portal_edge_length_blocks": w.seams().iter().map(|s| s.length).sum::<f64>(),
        "half_turn_edge_length_blocks": w.seams().iter().filter(|s| matches!(s.kind, crate::glue::SeamKind::HalfTurn{..})).map(|s| s.length).sum::<f64>(),
    })
}

/// Where the singular points fall on a baked planet, for every candidate longitude: the land share
/// in a disc of `radius_deg` around each and whether the point itself is land.
pub fn place(
    src: &Path,
    w_of: impl Fn(f64) -> QWorld,
    step_deg: f64,
    radius_deg: f64,
) -> Result<Value, String> {
    let m = Manifest::read(src).map_err(|e| e.to_string())?;
    let le = m.layer("elevation").ok_or("no elevation layer")?;
    let elev = TilePackReader::open(&src.join(&le.path))
        .and_then(|r| r.load())
        .map_err(|e| e.to_string())?;
    let proj = Projection::new(m.scale.circumference_blocks);
    let sea = f64::from(m.scale.sea_level_y);
    let land = |lat: f64, lon: f64| {
        let (x, z) = proj.block_of(lat, lon);
        elev.bilinear(x, z) > sea
    };
    // The planet's land share, area-weighted (a check that "land" is read right).
    let mut lsum = 0.0;
    let mut wsum = 0.0;
    for i in 0..180 {
        let lat = -89.5 + i as f64;
        for j in 0..360 {
            let wgt = lat.to_radians().cos();
            wsum += wgt;
            if land(lat, -179.5 + j as f64) {
                lsum += wgt;
            }
        }
    }
    let disc = |lat: f64, lon: f64| {
        let c = unit_of_lat_lon(lat, lon);
        let (mut n, mut l) = (0, 0);
        for i in 0..41 {
            for j in 0..41 {
                let (la, lo) = (
                    lat - radius_deg + 2.0 * radius_deg * i as f64 / 40.0,
                    lon - 3.0 * radius_deg + 6.0 * radius_deg * j as f64 / 40.0,
                );
                if la.abs() > 90.0 {
                    continue;
                }
                let u = unit_of_lat_lon(la, lo);
                let d = (c[0] * u[0] + c[1] * u[1] + c[2] * u[2])
                    .clamp(-1.0, 1.0)
                    .acos()
                    .to_degrees();
                if d <= radius_deg {
                    n += 1;
                    if land(la, lo) {
                        l += 1;
                    }
                }
            }
        }
        f64::from(l) / f64::from(n.max(1))
    };
    let mut rows = Vec::new();
    let mut best: Option<(f64, f64, usize)> = None;
    let mut lon = 0.0;
    while lon < 360.0 {
        let w = w_of(lon);
        let pts: Vec<(f64, f64)> = w
            .singular_points()
            .into_iter()
            .map(|(x, z)| w.lat_lon(x, z))
            .collect();
        let shares: Vec<f64> = pts.iter().map(|&(la, lo)| disc(la, lo)).collect();
        let on_land = pts.iter().filter(|&&(la, lo)| land(la, lo)).count();
        let worst = shares.iter().copied().fold(0.0, f64::max);
        let score = worst + shares.iter().sum::<f64>() / 4.0;
        if best.is_none_or(|b| (on_land, score) < (b.2, b.1)) {
            best = Some((lon, score, on_land));
        }
        rows.push(json!({"lon": lon, "points": pts.iter().map(|p| [r3(p.0), r3(p.1)]).collect::<Vec<_>>(),
                         "land_share_in_disc": shares.iter().map(|s| r3(*s)).collect::<Vec<_>>(), "points_on_land": on_land}));
        lon += step_deg;
    }
    let b = best.unwrap();
    let mut on: Vec<u64> = rows
        .iter()
        .map(|r| r["points_on_land"].as_u64().unwrap())
        .collect();
    on.sort_unstable();
    let median_on_land = on[on.len() / 2];
    Ok(
        json!({"planet_land_share": r3(lsum / wsum), "radius_deg": radius_deg, "best_lon": b.0,
              "best_points_on_land": b.2, "best_score": r3(b.1),
              "best": rows.iter().find(|r| r["lon"].as_f64() == Some(b.0)).cloned(),
              "median_points_on_land": median_on_land,
              "candidates": rows.len(),
              "candidates_all_points_at_sea": rows.iter().filter(|r| r["points_on_land"].as_u64() == Some(0)).count(),
              "candidates_all_discs_at_sea": rows.iter().filter(|r| r["land_share_in_disc"].as_array().unwrap().iter().all(|v| v.as_f64() == Some(0.0))).count(),
              "mean_disc_land_share": r3(rows.iter().map(|r| r["land_share_in_disc"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).sum::<f64>() / 4.0).sum::<f64>() / rows.len() as f64)}),
    )
}

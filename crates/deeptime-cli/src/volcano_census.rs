// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime volcano-census`: how the active volcanoes of a world-data folder stand on the ground
//! the game reads (the acceptance measure of VOL-080, the vent as the summit).
//!
//! Every volcano of `volcanoes/active.json` is measured in its own simulation frame: the simulation
//! draws a cone as a circle in its Lambert block metric (`vol::for_cells`: metres east–west are the
//! planet radius times the longitude difference, north–south the radius times the difference of
//! the unit vector's z), so a column's ρ (distance from the vent over the catalogue radius) is
//! computed exactly from the world projection's unit vector, whatever the projection stretches.
//! For each volcano:
//!
//! - **summit**: the highest column within ρ ≤ 0.35 (`SUMMIT_RHO`), its ρ, its distance from the
//!   vent in blocks and its height over the vent column; `centred` when it is on the crater's rim
//!   or inside it (ρ ≤ max(1.6 × the crater's ρ, 0.06)) or within 2 blocks of the vent;
//! - **ground tilt**: a plane fitted (least squares, metres) to the ring ρ 1.0 to 1.3 around the
//!   cone; its slope (m/m) and the tilt ratio, the fall of that plane across the cone's diameter
//!   over the cone's height (above 1 the ground falls more across the cone than the cone stands);
//! - **relief**: the summit and the vent column over that plane, as shares of the height;
//! - **crater**: the floor (lowest column within 0.6 × its ρ, else the vent column) and the rim
//!   (the highest column of each of 8 sectors between 0.6 and 1.6 × its ρ); `visible` when the
//!   lowest sector rim stands at least a block over the floor; the depth (mean rim over the floor)
//!   beside the catalogue's depth, both in blocks.
//!
//! Writes every row and a summary (all volcanoes and those whose vent is above sea level) as JSON.

use std::path::Path;

use deeptime_format::{ColumnWorld, FormatError};
use serde_json::{Value, json};

/// The summit search radius, in ρ: inside it a summit belongs to the cone (beyond it ground the
/// cone stands against may rise higher, as a range uphill does).
const SUMMIT_RHO: f64 = 0.35;
/// The ring the ground plane is fitted on, in ρ.
const RING: (f64, f64) = (1.0, 1.3);
/// Crater rim sectors.
const SECTORS: usize = 8;

/// One volcano of `active.json`.
struct Entry {
    id: u64,
    kind: String,
    x: f64,
    z: f64,
    radius_m: f64,
    height_m: f64,
    crater: Option<(f64, f64, bool)>,
}

/// Least-squares plane `h = a + b·x + c·z` through `pts` (x, z, h); `None` with fewer than 3
/// points or a degenerate spread.
fn plane(pts: &[(f64, f64, f64)]) -> Option<[f64; 3]> {
    if pts.len() < 3 {
        return None;
    }
    let n = pts.len() as f64;
    let (mx, mz, mh) = pts.iter().fold((0.0, 0.0, 0.0), |s, p| {
        (s.0 + p.0 / n, s.1 + p.1 / n, s.2 + p.2 / n)
    });
    let (mut sxx, mut sxz, mut szz, mut sxh, mut szh) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for p in pts {
        let (dx, dz, dh) = (p.0 - mx, p.1 - mz, p.2 - mh);
        sxx += dx * dx;
        sxz += dx * dz;
        szz += dz * dz;
        sxh += dx * dh;
        szh += dz * dh;
    }
    let det = sxx * szz - sxz * sxz;
    if det.abs() < 1e-9 * (sxx * szz).max(1e-30) {
        return None;
    }
    let b = (sxh * szz - szh * sxz) / det;
    let c = (szh * sxx - sxh * sxz) / det;
    Some([mh - b * mx - c * mz, b, c])
}

fn median(v: &mut [f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let n = v.len();
    Some(if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    })
}

fn quantile(v: &mut [f64], q: f64) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let k = ((v.len() - 1) as f64 * q).round() as usize;
    Some(v[k])
}

fn r2(v: f64) -> Value {
    if v.is_finite() {
        json!((v * 100.0).round() / 100.0)
    } else {
        Value::Null
    }
}

fn io(path: &Path, source: std::io::Error) -> FormatError {
    FormatError::Io {
        path: path.to_owned(),
        source,
    }
}

/// Measures one volcano (the row, or `None` when its box holds no column inside the cone).
fn measure(w: &ColumnWorld, v: &Entry, m_per_rad: f64, mv: f64, sea_y: f64) -> Option<Value> {
    let proj = w.manifest.world_projection();
    let u0 = proj.unit(v.x, v.z);
    let lon0 = u0[1].atan2(u0[0]);
    let tau = std::f64::consts::TAU;
    // Simulation-frame metres of world point (x, z) from the vent.
    let frame = |x: f64, z: f64| -> (f64, f64) {
        let u = proj.unit(x, z);
        let dl = (u[1].atan2(u[0]) - lon0 + std::f64::consts::PI).rem_euclid(tau)
            - std::f64::consts::PI;
        (m_per_rad * dl, -m_per_rad * (u[2] - u0[2]))
    };
    // The box: the smallest scale of the local Jacobian reaches past the ring.
    let (px, pz) = frame(v.x + 1.0, v.z);
    let (mx, mz) = frame(v.x - 1.0, v.z);
    let (qx, qz) = frame(v.x, v.z + 1.0);
    let (nx, nz) = frame(v.x, v.z - 1.0);
    let (a, b, c, d) = (
        0.5 * (px - mx),
        0.5 * (qx - nx),
        0.5 * (pz - mz),
        0.5 * (qz - nz),
    );
    // Singular values of [[a, b], [c, d]] (metres per block).
    let s1 = a * a + b * b + c * c + d * d;
    let s2 = ((a * a + b * b - c * c - d * d).powi(2) + 4.0 * (a * c + b * d).powi(2)).sqrt();
    let smin = (0.5 * (s1 - s2)).max(0.0).sqrt().max(1.0);
    let smax = (0.5 * (s1 + s2)).sqrt();
    let r = v.radius_m.max(1.0);
    let half = ((RING.1 + 0.1) * r / smin).ceil().min(800.0) as i64 + 2;
    let (cx, cz) = (v.x.round() as i64, v.z.round() as i64);
    let vent_y = w.height(cx, cz);
    let crater_rho = v.crater.map_or(0.0, |c| c.0 / r);
    let mut summit = (f64::NEG_INFINITY, 0.0, 0i64, 0i64);
    let mut ring = Vec::new();
    let mut floor = f64::INFINITY;
    let mut rims = [f64::NEG_INFINITY; SECTORS];
    for z in cz - half..=cz + half {
        for x in cx - half..=cx + half {
            let (ex, ez) = frame(x as f64, z as f64);
            let rho = ex.hypot(ez) / r;
            if rho > RING.1 {
                continue;
            }
            let y = w.height(x, z);
            if rho <= SUMMIT_RHO && y > summit.0 {
                summit = (y, rho, x, z);
            }
            if rho >= RING.0 {
                ring.push((ex, ez, y * mv));
            }
            if crater_rho > 0.0 {
                if rho <= 0.6 * crater_rho {
                    floor = floor.min(y);
                } else if rho <= 1.6 * crater_rho {
                    let s = ((ez.atan2(ex) + std::f64::consts::PI) / tau * SECTORS as f64) as usize;
                    let s = s.min(SECTORS - 1);
                    rims[s] = rims[s].max(y);
                }
            }
        }
    }
    if !summit.0.is_finite() {
        return None;
    }
    let pl = plane(&ring);
    let (slope, tilt, summit_relief, vent_relief) = match pl {
        Some([a0, b0, c0]) => {
            let slope = b0.hypot(c0);
            let (sx, sz) = frame(summit.2 as f64, summit.3 as f64);
            (
                slope,
                slope * 2.0 * r / v.height_m.max(1.0),
                (summit.0 * mv - (a0 + b0 * sx + c0 * sz)) / v.height_m.max(1.0),
                (vent_y * mv - a0) / v.height_m.max(1.0),
            )
        }
        None => (f64::NAN, f64::NAN, f64::NAN, f64::NAN),
    };
    let offset = ((summit.2 as f64 - v.x).hypot(summit.3 as f64 - v.z) * 10.0).round() / 10.0;
    let centred = summit.1 <= (1.6 * crater_rho).max(0.06) || offset <= 2.0;
    let crater = v.crater.map(|(_, depth_m, caldera)| {
        let floor = if floor.is_finite() { floor } else { vent_y };
        let got: Vec<f64> = rims.iter().copied().filter(|r| r.is_finite()).collect();
        let rim_min = got.iter().copied().fold(f64::INFINITY, f64::min);
        let rim_mean = got.iter().sum::<f64>() / got.len().max(1) as f64;
        json!({
            "rho": (crater_rho * 1000.0).round() / 1000.0,
            "caldera": caldera,
            "sectors": got.len(),
            "visible": got.len() >= SECTORS / 2 && rim_min - floor >= 1.0,
            "depth_blocks": r2(rim_mean - floor),
            "lowest_rim_over_floor_blocks": r2(rim_min - floor),
            "catalogue_depth_blocks": r2(depth_m / mv),
        })
    });
    Some(json!({
        "id": v.id,
        "kind": v.kind,
        "x": v.x,
        "z": v.z,
        "radius_m": v.radius_m,
        "height_m": v.height_m,
        "dry": vent_y > sea_y,
        "m_per_block": [r2(smin), r2(smax)],
        "vent_y": r2(vent_y),
        "summit": {
            "y": r2(summit.0),
            "x": summit.2,
            "z": summit.3,
            "rho": (summit.1 * 1000.0).round() / 1000.0,
            "offset_blocks": offset,
            "over_vent_blocks": r2(summit.0 - vent_y),
            "centred": centred,
        },
        "ground": {
            "ring_columns": ring.len(),
            "slope": pl.map(|_| (slope * 1000.0).round() / 1000.0),
            "tilt_ratio": r2(tilt),
            "summit_over_plane_share": r2(summit_relief),
            "vent_over_plane_share": r2(vent_relief),
        },
        "crater": crater,
    }))
}

/// Shares and medians of `rows`.
fn summary(rows: &[&Value]) -> Value {
    let f = |r: &Value, p: &str| r.pointer(p).and_then(Value::as_f64);
    let col = |p: &str| -> Vec<f64> { rows.iter().filter_map(|r| f(r, p)).collect() };
    let share = |p: &str| -> Value {
        if rows.is_empty() {
            return Value::Null;
        }
        let k = rows
            .iter()
            .filter(|r| r.pointer(p).and_then(Value::as_bool) == Some(true))
            .count();
        r2(k as f64 / rows.len() as f64)
    };
    let craters: Vec<&&Value> = rows.iter().filter(|r| !r["crater"].is_null()).collect();
    let visible = craters
        .iter()
        .filter(|r| r["crater"]["visible"].as_bool() == Some(true))
        .count();
    let tilted = col("/ground/tilt_ratio")
        .iter()
        .filter(|t| **t > 1.0)
        .count();
    json!({
        "volcanoes": rows.len(),
        "centred_share": share("/summit/centred"),
        "summit_offset_blocks": {
            "median": median(&mut col("/summit/offset_blocks")).map(r2),
            "p90": quantile(&mut col("/summit/offset_blocks"), 0.9).map(r2),
        },
        "summit_rho_median": median(&mut col("/summit/rho")).map(r2),
        "summit_over_vent_blocks_median": median(&mut col("/summit/over_vent_blocks")).map(r2),
        "ground_slope_median": median(&mut col("/ground/slope")).map(|v| (v * 1000.0).round() / 1000.0),
        "tilt_ratio": {
            "median": median(&mut col("/ground/tilt_ratio")).map(r2),
            "p90": quantile(&mut col("/ground/tilt_ratio"), 0.9).map(r2),
            "above_1_share": if rows.is_empty() { Value::Null } else { r2(tilted as f64 / rows.len() as f64) },
        },
        "summit_over_plane_share_median": median(&mut col("/ground/summit_over_plane_share")).map(r2),
        "vent_over_plane_share_median": median(&mut col("/ground/vent_over_plane_share")).map(r2),
        "craters": craters.len(),
        "crater_visible_share": if craters.is_empty() { Value::Null } else { r2(visible as f64 / craters.len() as f64) },
        "crater_depth_blocks_median": median(&mut col("/crater/depth_blocks")).map(r2),
        "crater_catalogue_depth_blocks_median": median(&mut col("/crater/catalogue_depth_blocks")).map(r2),
    })
}

/// Measures every active volcano of `world` and writes the report to `out`; returns the summary.
pub fn run(world: &Path, out: &Path) -> Result<Value, FormatError> {
    let path = world.join("volcanoes/active.json");
    let bytes = std::fs::read(&path).map_err(|e| io(&path, e))?;
    let doc: Value = serde_json::from_slice(&bytes)
        .map_err(|e| FormatError::Invalid(format!("{}: {e}", path.display())))?;
    let list: Vec<Entry> = doc["volcanoes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|o| {
            Some(Entry {
                id: o["id"].as_u64()?,
                kind: o["kind"].as_str().unwrap_or("").to_owned(),
                x: o["x"].as_f64()?,
                z: o["z"].as_f64()?,
                radius_m: o["radius_m"].as_f64()?,
                height_m: o["height_m"].as_f64()?,
                crater: o["crater"].as_object().map(|c| {
                    (
                        c.get("radius_m").and_then(Value::as_f64).unwrap_or(0.0),
                        c.get("depth_m").and_then(Value::as_f64).unwrap_or(0.0),
                        c.get("caldera").and_then(Value::as_bool).unwrap_or(false),
                    )
                }),
            })
        })
        .collect();
    let w = ColumnWorld::open(world)?;
    let proj = w.manifest.world_projection();
    let m_per_rad = proj.radius() * w.manifest.scale.m_per_block_h;
    let mv = w.manifest.scale.m_per_block_v;
    let sea_y = f64::from(w.manifest.scale.sea_level_y);
    // A few threads, each with its own reader (the lazy layers' caches are behind one lock).
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get().min(8));
    let chunk = list.len().div_ceil(threads).max(1);
    let mut rows: Vec<Value> = std::thread::scope(|scope| {
        let handles: Vec<_> = list
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || -> Result<Vec<Value>, FormatError> {
                    let w = ColumnWorld::open(world)?;
                    Ok(part
                        .iter()
                        .filter_map(|v| measure(&w, v, m_per_rad, mv, sea_y))
                        .collect())
                })
            })
            .collect();
        let mut all = Vec::new();
        for h in handles {
            all.extend(h.join().expect("census thread")?);
        }
        Ok::<_, FormatError>(all)
    })?;
    rows.sort_by_key(|r| r["id"].as_u64());
    let all: Vec<&Value> = rows.iter().collect();
    let dry: Vec<&Value> = rows.iter().filter(|r| r["dry"] == json!(true)).collect();
    let sum = json!({"all": summary(&all), "dry": summary(&dry)});
    let report = json!({
        "world": world.display().to_string(),
        "source": w.manifest.source,
        "m_per_rad": m_per_rad,
        "summit_rho": SUMMIT_RHO,
        "ring_rho": [RING.0, RING.1],
        "summary": sum,
        "volcanoes": rows,
    });
    std::fs::write(out, serde_json::to_vec_pretty(&report).expect("json"))
        .map_err(|e| io(out, e))?;
    Ok(report["summary"].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plane_fit_recovers_a_tilted_plane() {
        let pts: Vec<(f64, f64, f64)> = (0..30)
            .map(|k| {
                let (x, z) = ((k % 6) as f64 * 100.0, (k / 6) as f64 * 70.0 - 50.0);
                (x, z, 12.0 + 0.2 * x - 0.05 * z)
            })
            .collect();
        let [a, b, c] = plane(&pts).unwrap();
        assert!((a - 12.0).abs() < 1e-6 && (b - 0.2).abs() < 1e-9 && (c + 0.05).abs() < 1e-9);
        assert!(plane(&pts[..2]).is_none());
        let line: Vec<_> = (0..5).map(|k| (k as f64, 0.0, 1.0)).collect();
        assert!(plane(&line).is_none());
    }

    #[test]
    fn medians_and_quantiles() {
        assert_eq!(median(&mut [3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(median(&mut [4.0, 1.0, 2.0, 3.0]), Some(2.5));
        assert_eq!(quantile(&mut [1.0, 2.0, 3.0, 4.0, 5.0], 0.9), Some(5.0));
        assert_eq!(median(&mut []), None);
    }
}

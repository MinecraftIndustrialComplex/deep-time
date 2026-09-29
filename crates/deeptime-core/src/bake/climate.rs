// SPDX-License-Identifier: LGPL-3.0-or-later
//! The climate part of the bake.
//!
//! **Physical climate on L1** (plan §5.3; P1-09's inputs, the viewer's and the rules'):
//! `climate/t_mean_c` (i16, 0.01 °C), `climate/p_mm` (u16, mm/yr), `climate/koppen` (u8, the index
//! of `climate::koppen::CODES`, 31 = sea), from the climate stage's last snapshot.
//!
//! **The five biome inputs on L2** (`climate/{temperature, vegetation, continents, erosion,
//! ridges}`, i16, vanilla parameter × 10,000): what the noise router's climate functions read
//! (plan §7.1). Plan §7.2 has Java compute the mapping per world against the pack's own bands and
//! distributions (P1-09); until then this module writes a **provisional mapping** so a baked world
//! has plausible biomes. Each is piecewise linear between anchors placed on
//! `OverworldBiomeBuilder`'s band edges (1.21.1):
//!
//! - temperature from the mean annual temperature: −25 °C → −1, −10 → −0.45 (frozen | cold),
//!   0 → −0.15 (cold | temperate), 12 → 0.2 (temperate | warm), 22 → 0.55 (warm | hot), 32 → 1;
//! - vegetation (humidity) from the annual precipitation: 0 mm → −1, 250 → −0.35, 500 → −0.1,
//!   900 → 0.1, 1500 → 0.3, 3000 → 1;
//! - continentalness from the distance to the coast on land (0 km → −0.19, 5 → −0.11 (coast |
//!   near inland), 60 → 0.03, 300 → 0.3, 1500 → 1) and from the depth at sea (0 m → −0.19,
//!   200 → −0.3, 2500 → −0.455 (ocean | deep ocean), 6000 → −1.0);
//! - erosion from the local relief (the range of the L2 heights over 5 × 5 cells): 0 m → 0.8,
//!   100 → 0.5, 300 → 0.1, 800 → −0.3, 1500 → −0.6, 3000 → −0.9;
//! - ridges (weirdness) from the curvature: the L2 Laplacian over the 3 × 3 cells, −40 m (a
//!   valley) → peaks-and-valleys −0.8 … +40 m (a crest) → +1, turned into weirdness on the rising
//!   branch |w| = (PV + 1)/3 and signed by a slow noise (vanilla's biome variants). PV never goes
//!   under −0.8, so no river biome is placed: rivers are Phase 2.

use deeptime_format::{DType, Noise};
use rayon::prelude::*;
use serde_json::{Value, json};

use super::{Out, desc, field, grids, resample};
use crate::climate;
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::surface::SurfaceState;
use crate::surface::young;
use crate::world::{Field, WorldState};

/// The five biome inputs, in the router's names (plan §7.1).
pub const BIOME_INPUTS: [&str; 5] = crate::synth::CLIMATE;

/// Salt of the noise that signs the weirdness.
pub const WEIRDNESS_SALT: u32 = 1801;
/// Its wavelength, blocks.
pub const WEIRDNESS_WAVELENGTH_BLOCKS: f64 = 1500.0;

/// Piecewise linear through `anchors` (x ascending), clamped at the ends.
pub fn piecewise(anchors: &[(f64, f64)], x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    let (x0, y0) = anchors[0];
    if x <= x0 {
        return y0;
    }
    for w in anchors.windows(2) {
        let ((xa, ya), (xb, yb)) = (w[0], w[1]);
        if x == xb {
            return yb;
        }
        if x < xb {
            return ya + (yb - ya) * (x - xa) / (xb - xa);
        }
    }
    anchors[anchors.len() - 1].1
}

pub const TEMPERATURE: [(f64, f64); 6] = [
    (-25.0, -1.0),
    (-10.0, -0.45),
    (0.0, -0.15),
    (12.0, 0.2),
    (22.0, 0.55),
    (32.0, 1.0),
];
pub const HUMIDITY: [(f64, f64); 6] = [
    (0.0, -1.0),
    (250.0, -0.35),
    (500.0, -0.1),
    (900.0, 0.1),
    (1500.0, 0.3),
    (3000.0, 1.0),
];
pub const LAND_KM: [(f64, f64); 5] = [
    (0.0, -0.19),
    (5.0, -0.11),
    (60.0, 0.03),
    (300.0, 0.3),
    (1500.0, 1.0),
];
pub const SEA_DEPTH_M: [(f64, f64); 4] = [
    (0.0, -0.19),
    (200.0, -0.3),
    (2500.0, -0.455),
    (6000.0, -1.0),
];
pub const RELIEF_M: [(f64, f64); 6] = [
    (0.0, 0.8),
    (100.0, 0.5),
    (300.0, 0.1),
    (800.0, -0.3),
    (1500.0, -0.6),
    (3000.0, -0.9),
];
/// Laplacian (m, positive = convex crest) → peaks-and-valleys.
pub const CURVATURE_M: [(f64, f64); 2] = [(-40.0, -0.8), (40.0, 1.0)];

fn to_raw(v: f64) -> i16 {
    crate::synth::quantise(v * 10_000.0)
}

/// Distance to the other medium (land ↔ sea), km, from a two-pass chamfer transform on the grid
/// (x wraps: two sweeps with the wrap each way). Cells of `from` get 0.
fn distance_km(land: &[bool], w: usize, h: usize, cell_km: f64, to_sea: bool) -> Vec<f64> {
    let inf = f64::INFINITY;
    // Seeds: cells of the target medium.
    let mut d: Vec<f64> = land
        .iter()
        .map(|&l| if l == to_sea { inf } else { 0.0 })
        .collect();
    let (a, b) = (cell_km, cell_km * std::f64::consts::SQRT_2);
    for _ in 0..2 {
        for j in 0..h {
            for i in 0..w {
                let c = j * w + i;
                let mut v = d[c];
                let il = (i + w - 1) % w;
                v = v.min(d[j * w + il] + a);
                if j > 0 {
                    let up = (j - 1) * w;
                    v = v.min(d[up + i] + a);
                    v = v.min(d[up + il] + b);
                    v = v.min(d[up + (i + 1) % w] + b);
                }
                d[c] = v;
            }
        }
        for j in (0..h).rev() {
            for i in (0..w).rev() {
                let c = j * w + i;
                let mut v = d[c];
                let ir = (i + 1) % w;
                v = v.min(d[j * w + ir] + a);
                if j + 1 < h {
                    let dn = (j + 1) * w;
                    v = v.min(d[dn + i] + a);
                    v = v.min(d[dn + ir] + b);
                    v = v.min(d[dn + (i + w - 1) % w] + b);
                }
                d[c] = v;
            }
        }
    }
    d
}

pub fn bake(
    world: &WorldState,
    p: &PlanetPreset,
    surface: &SurfaceState,
    out: &mut Out<'_>,
) -> Result<Value, CoreError> {
    let (_l1, l2) = grids(p)?;
    let t = field(world, climate::field::T_MEAN);
    let pr = field(world, climate::field::PRECIP);
    let kp = field(world, climate::field::KOPPEN);

    // Physical climate on its own grid (L1).
    let mut physical = Vec::new();
    if let Some(f) = t {
        let raw: Vec<i16> = f
            .data
            .iter()
            .map(|&v| crate::synth::quantise(f64::from(v) * 100.0))
            .collect();
        out.layer(
            desc("climate/t_mean_c", DType::I16, &f.grid, 0.01, "degC", None),
            &raw,
        )?;
        physical.push("climate/t_mean_c");
    }
    if let Some(f) = pr {
        let raw: Vec<u16> = f
            .data
            .iter()
            .map(|&v| {
                if v.is_nan() {
                    0
                } else {
                    f64::from(v).round().clamp(0.0, 65_535.0) as u16
                }
            })
            .collect();
        out.layer(
            desc("climate/p_mm", DType::U16, &f.grid, 1.0, "mm/yr", None),
            &raw,
        )?;
        physical.push("climate/p_mm");
    }
    if let Some(f) = kp {
        let raw: Vec<u8> = f
            .data
            .iter()
            .map(|&v| {
                if v.is_nan() {
                    climate::koppen::SEA
                } else {
                    (v.round() as i64).clamp(0, 31) as u8
                }
            })
            .collect();
        out.layer(
            desc(
                "climate/koppen",
                DType::U8,
                &f.grid,
                1.0,
                "koppen code index (notes.bake.climate.koppen_codes)",
                Some(255.0),
            ),
            &raw,
        )?;
        physical.push("climate/koppen");
    }

    // The five biome inputs on L2.
    let (w, h) = (l2.w as usize, l2.h as usize);
    let sea = surface.sea_level_m;
    let elev: Vec<f32> = match field(world, young::field::ELEVATION_M) {
        Some(f) => resample(f, &l2, false),
        None => resample(
            &Field {
                grid: surface.grid.clone(),
                data: surface.elevation_m.clone(),
            },
            &l2,
            false,
        ),
    };
    let t2: Option<Vec<f32>> = t.map(|f| resample(f, &l2, false));
    let p2: Option<Vec<f32>> = pr.map(|f| resample(f, &l2, false));
    let land: Vec<bool> = elev.iter().map(|&z| f64::from(z) > sea).collect();
    let cell_km = f64::from(l2.r) * p.scale.m_per_block_h / 1000.0;
    let to_sea = distance_km(&land, w, h, cell_km, true);
    let c = f64::from(p.scale.circumference_blocks);
    let noise = Noise::new(p.seed, p.scale.circumference_blocks);
    let at = |i: i64, j: i64| -> f64 {
        let ii = i.rem_euclid(w as i64) as usize;
        let jj = j.clamp(0, h as i64 - 1) as usize;
        f64::from(elev[jj * w + ii])
    };
    let mut layers: [Vec<i16>; 5] = std::array::from_fn(|_| vec![0i16; w * h]);
    {
        let [temp, veg, cont, ero, ridge] = &mut layers;
        temp.par_chunks_mut(w)
            .zip(veg.par_chunks_mut(w))
            .zip(cont.par_chunks_mut(w))
            .zip(ero.par_chunks_mut(w))
            .zip(ridge.par_chunks_mut(w))
            .enumerate()
            .for_each(|(j, ((((tr, vr), cr), er), rr))| {
                let lat = 90.0 - 180.0 * (j as f64 + 0.5) / h as f64;
                for i in 0..w {
                    let k = j * w + i;
                    let z = f64::from(elev[k]);
                    // Without the climate stage: temperature by latitude, humidity middling.
                    let tc = t2
                        .as_ref()
                        .map_or(28.0 - 0.55 * lat.abs(), |v| f64::from(v[k]));
                    let pm = p2.as_ref().map_or(700.0, |v| f64::from(v[k]));
                    tr[i] = to_raw(piecewise(&TEMPERATURE, tc));
                    vr[i] = to_raw(piecewise(&HUMIDITY, pm));
                    cr[i] = to_raw(if land[k] {
                        piecewise(&LAND_KM, to_sea[k])
                    } else {
                        piecewise(&SEA_DEPTH_M, sea - z)
                    });
                    let (ii, jj) = (i as i64, j as i64);
                    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
                    for dj in -2..=2 {
                        for di in -2..=2 {
                            let v = at(ii + di, jj + dj);
                            lo = lo.min(v);
                            hi = hi.max(v);
                        }
                    }
                    er[i] = to_raw(piecewise(&RELIEF_M, hi - lo));
                    let mean8 = (at(ii - 1, jj - 1)
                        + at(ii, jj - 1)
                        + at(ii + 1, jj - 1)
                        + at(ii - 1, jj)
                        + at(ii + 1, jj)
                        + at(ii - 1, jj + 1)
                        + at(ii, jj + 1)
                        + at(ii + 1, jj + 1))
                        / 8.0;
                    // Positive on a crest (the cell stands above its neighbours).
                    let crest = z - mean8;
                    let pv = piecewise(&CURVATURE_M, crest);
                    let mag = (pv + 1.0) / 3.0;
                    let bx = -c / 2.0 + (i as f64) * f64::from(l2.r);
                    let bz = -c / 4.0 + (j as f64) * f64::from(l2.r);
                    let sign =
                        if noise.value(bx, bz, WEIRDNESS_WAVELENGTH_BLOCKS, WEIRDNESS_SALT) < 0.0 {
                            -1.0
                        } else {
                            1.0
                        };
                    rr[i] = to_raw(sign * mag);
                }
            });
    }
    for (name, data) in BIOME_INPUTS.iter().zip(&layers) {
        out.layer(
            desc(name, DType::I16, &l2, 1e-4, "vanilla_param", None),
            data,
        )?;
    }
    Ok(json!({
        "physical": physical,
        "koppen_codes": climate::koppen::CODES,
        "biome_inputs": BIOME_INPUTS,
        "mapping": "provisional (P1-08) until P1-09's per-world mapping: piecewise linear anchors on OverworldBiomeBuilder's band edges, see bake/climate.rs",
        "anchors": {
            "temperature_c": TEMPERATURE, "humidity_mm": HUMIDITY, "continents_land_km": LAND_KM,
            "continents_sea_depth_m": SEA_DEPTH_M, "erosion_relief_m": RELIEF_M,
            "ridges_crest_m": CURVATURE_M,
        },
        "grid": l2,
        "from_climate_stage": t.is_some(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piecewise_interpolates_and_clamps() {
        assert_eq!(piecewise(&TEMPERATURE, -40.0), -1.0);
        assert_eq!(piecewise(&TEMPERATURE, -10.0), -0.45);
        assert!((piecewise(&TEMPERATURE, 6.0) - 0.025).abs() < 1e-12);
        assert_eq!(piecewise(&TEMPERATURE, 99.0), 1.0);
        assert!(piecewise(&HUMIDITY, f64::NAN).is_nan());
    }

    #[test]
    fn distance_to_the_sea_wraps_in_x() {
        // One sea column at i = 0 of a 10 × 3 grid: land at i = 9 is one cell from it.
        let (w, h) = (10, 3);
        let land: Vec<bool> = (0..w * h).map(|c| c % w != 0).collect();
        let d = distance_km(&land, w, h, 2.0, true);
        assert_eq!(d[0], 0.0);
        assert_eq!(d[1], 2.0);
        assert_eq!(d[9], 2.0, "across the seam");
        assert_eq!(d[5], 10.0);
    }
}

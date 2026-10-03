// SPDX-License-Identifier: LGPL-3.0-or-later
//! Circulation belts: the three-cell wind belts, the ITCZ and its seasonal migration, and the
//! ocean gyres' imprint on coasts (plan §8.3 item 2; the Worldbuilding Pasta rules of thumb in
//! `surface_processes_climate.md` §5: wet belts near 0° and 60°, dry near 30° and the poles,
//! subtropical highs at 25–35°, the ITCZ following the thermal equator seasonally, gyre currents
//! warming and cooling coasts).
//!
//! **Belt coordinate.** Everything zonal is a function of φ′ = φ − shift(x, φ, month): near the
//! equator the shift is the ITCZ's latitude in that column and month (the thermal equator from the
//! EBM's land and sea temperatures and the local land share — so it swings far over continents,
//! which makes monsoons), blending by 30° into a hemispheric shift of the subtropical highs and
//! the westerlies (`subtropical_shift_deg` at Earth's tilt), and fading toward the poles.
//!
//! **Regimes.** |φ′| < 30°: trade winds (from the ENE in the north, ESE in the south); 30–60°:
//! westerlies (from the WSW / WNW); beyond 60°: polar easterlies; blended over ±3°. Where φ′ and φ
//! have opposite signs (between the equator and an ITCZ that moved across it) the trades cross
//! the equator and turn into monsoon westerlies (from the SW in the north, the NW in the south).
//!
//! **Basins.** On each row an ocean cell's position across its basin, a = (d_W − d_E)/(d_W + d_E)
//! from the distances to land westward and eastward, is +1 at the basin's eastern side (along a
//! continent's west coast) and −1 at its western side; wide basins only. It sets the gyres'
//! sea-surface anomalies and, in the subtropics, the drying under the highs' eastern flanks.

use rayon::prelude::*;

use super::ebm::smoothstep;
use super::params::ClimateParams;
use crate::surface::grid::{Dims, box_blur};

/// Regimes, in this order (their weights are `[f64; REGIMES]`).
pub const REGIMES: usize = 8;
pub const NE_TRADES: usize = 0;
pub const SE_TRADES: usize = 1;
pub const WESTERLIES_N: usize = 2;
pub const WESTERLIES_S: usize = 3;
pub const POLAR_N: usize = 4;
pub const POLAR_S: usize = 5;
pub const MONSOON_N: usize = 6;
pub const MONSOON_S: usize = 7;

/// Whether a regime blows from the west (so air travels east along a row).
pub const EASTWARD: [bool; REGIMES] = [false, false, true, true, false, false, true, true];

/// Names (for timings and tests).
pub const REGIME_NAMES: [&str; REGIMES] = [
    "ne_trades",
    "se_trades",
    "westerlies_n",
    "westerlies_s",
    "polar_n",
    "polar_s",
    "monsoon_n",
    "monsoon_s",
];

/// Wind of each regime in grid axes: (east, south) components, m/s.
pub fn winds(p: &ClimateParams) -> [(f64, f64); REGIMES] {
    let (t, w, q) = (p.trade_wind_m_s, p.westerly_m_s, p.polar_wind_m_s);
    let (c30, s30) = (0.866, 0.5);
    let (c20, s20) = (0.94, 0.342);
    let s45 = std::f64::consts::FRAC_1_SQRT_2;
    [
        (-c30 * t, s30 * t),  // NE trades blow to the WSW (south is +)
        (-c30 * t, -s30 * t), // SE trades blow to the WNW
        (c20 * w, -s20 * w),  // SW-erlies blow to the ENE
        (c20 * w, s20 * w),   // NW-erlies blow to the ESE
        (-c20 * q, s20 * q),  // polar NE-erlies
        (-c20 * q, -s20 * q), // polar SE-erlies
        (s45 * t, -s45 * t),  // SW monsoon blows to the NE
        (s45 * t, s45 * t),   // NW monsoon blows to the SE
    ]
}

/// The latitude (degrees, with its hemisphere) whose Coriolis parameter each regime's orographic
/// solve uses.
pub const REGIME_LAT_DEG: [f64; REGIMES] = [15.0, -15.0, 45.0, -45.0, 70.0, -70.0, 10.0, -10.0];

/// Regime weights at geographic latitude `phi` and belt latitude `phi_b` (degrees).
pub fn regime_weights(phi: f64, phi_b: f64) -> [f64; REGIMES] {
    let a = phi_b.abs();
    let trade = 1.0 - smoothstep(27.0, 33.0, a);
    let polar = smoothstep(57.0, 63.0, a);
    let west = (1.0 - trade - polar).max(0.0);
    let north = phi_b >= 0.0;
    let mut w = [0.0; REGIMES];
    // Trades that crossed the equator become monsoon westerlies (fully 3° past it).
    let crossed = if (phi >= 0.0) != north {
        smoothstep(0.0, 3.0, phi.abs())
    } else {
        0.0
    };
    if north {
        w[NE_TRADES] = trade * (1.0 - crossed);
        w[MONSOON_S] = trade * crossed;
        w[WESTERLIES_N] = west;
        w[POLAR_N] = polar;
    } else {
        w[SE_TRADES] = trade * (1.0 - crossed);
        w[MONSOON_N] = trade * crossed;
        w[WESTERLIES_S] = west;
        w[POLAR_S] = polar;
    }
    w
}

/// Zonal precipitation over open ocean at belt latitude φ′, mm/yr: the ITCZ, the subtropical dry
/// belts, the wet westerlies and the dry poles (Earth's zonal means in round numbers).
pub fn belt_precip(phi_b: f64) -> f64 {
    const P: [(f64, f64); 17] = [
        (0.0, 2400.0),
        (4.0, 2150.0),
        (8.0, 1600.0),
        (12.0, 1050.0),
        (16.0, 650.0),
        (20.0, 420.0),
        (24.0, 320.0),
        (28.0, 330.0),
        (32.0, 470.0),
        (36.0, 700.0),
        (42.0, 950.0),
        (48.0, 1100.0),
        (54.0, 1050.0),
        (60.0, 850.0),
        (70.0, 450.0),
        (80.0, 250.0),
        (90.0, 170.0),
    ];
    let a = phi_b.abs().min(90.0);
    for w in P.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if a <= x1 {
            return y0 + (y1 - y0) * (a - x0) / (x1 - x0);
        }
    }
    P[P.len() - 1].1
}

/// A smooth bump of half-width `hw` about `c`.
pub fn bump(x: f64, c: f64, hw: f64) -> f64 {
    let t = (x - c) / hw;
    if t.abs() >= 1.0 {
        0.0
    } else {
        let u = 1.0 - t * t;
        u * u
    }
}

/// Weight of the subtropical highs at |φ| (the basin effects of 15–35°).
pub fn subtropics(phi: f64) -> f64 {
    bump(phi.abs(), 25.0, 13.0)
}

/// Weight of the mid-latitude gyre effects at |φ| (45–65°).
pub fn midlatitudes(phi: f64) -> f64 {
    bump(phi.abs(), 55.0, 13.0)
}

/// The shift of the belts at latitude `phi` (degrees), given the column's ITCZ latitude and the
/// month's hemispheric shift of the subtropical highs.
pub fn shift(phi: f64, itcz: f64, hemi: f64) -> f64 {
    let t = smoothstep(10.0, 30.0, phi.abs());
    let s = itcz * (1.0 - t) + hemi * t;
    s * (1.0 - 0.6 * smoothstep(50.0, 85.0, phi.abs()))
}

/// The month's hemispheric shift of the subtropical highs, degrees (north in northern summer,
/// a month after the solstice).
pub fn hemispheric_shift(p: &ClimateParams, tilt_deg: f64, month: usize) -> f64 {
    let lam = super::insolation::lambda_at((month as f64 + 0.5) / 12.0);
    let k = tilt_deg.to_radians().sin() / 23.4f64.to_radians().sin();
    p.subtropical_shift_deg * k * (lam - 30f64.to_radians()).sin()
}

/// The ITCZ's latitude per column and month (`w × 12`, degrees): the thermal equator, i.e. the
/// area-weighted soft maximum (τ 1.5 °C; `area_row` the rows' cell areas, any unit: equal on the
/// Lambert grid) of the estimated sea-level temperature over |φ| ≤ 35°,
/// where the estimate mixes the EBM's ocean and land temperatures by the land share within
/// ~500 km; clamped to ±`itcz_max_shift_deg`, smoothed along the row over `itcz_smooth_km`
/// (Earth km).
#[allow(clippy::too_many_arguments)]
pub fn itcz(
    p: &ClimateParams,
    w: usize,
    lat_row: &[f64],
    area_row: &[f64],
    land_near: &[f64],
    t_ocean: &[[f64; 12]],
    t_land: &[[f64; 12]],
    smooth_cells: usize,
) -> Vec<[f64; 12]> {
    let rows: Vec<usize> = (0..lat_row.len())
        .filter(|&j| lat_row[j].abs() <= 35.0)
        .collect();
    let mut s: Vec<[f64; 12]> = (0..w)
        .into_par_iter()
        .map(|i| {
            let mut out = [0.0; 12];
            if rows.is_empty() {
                return out;
            }
            for (m, o) in out.iter_mut().enumerate() {
                let est = |j: usize| {
                    let l = land_near[j * w + i];
                    t_ocean[j][m] + l * (t_land[j][m] - t_ocean[j][m])
                };
                let tmax = rows.iter().map(|&j| est(j)).fold(f64::MIN, f64::max);
                let (mut num, mut den) = (0.0, 0.0);
                for &j in &rows {
                    // Each row's cells weigh their true area (equal on the Lambert grid).
                    let wt = ((est(j) - tmax) / 1.5).exp() * area_row[j];
                    num += wt * lat_row[j];
                    den += wt;
                }
                *o = (num / den.max(1e-300)).clamp(-p.itcz_max_shift_deg, p.itcz_max_shift_deg);
            }
            out
        })
        .collect();
    if smooth_cells > 0 && w > 1 {
        let r = smooth_cells.min(w / 2);
        for _ in 0..2 {
            let src = s.clone();
            let norm = 1.0 / (2 * r + 1) as f64;
            for (i, o) in s.iter_mut().enumerate() {
                for m in 0..12 {
                    let mut acc = 0.0;
                    for k in 0..=2 * r {
                        acc += src[(i + w + k - r) % w][m];
                    }
                    o[m] = acc * norm;
                }
            }
        }
    }
    s
}

/// Each ocean cell's position across its basin, in [−1, 1] (0 on land, in narrow seas and on rows
/// without land), smoothed over ocean cells with half-width `blur` cells.
/// `true_km(j)` is the true east–west width of a cell on row j in Earth km.
pub fn basin_position(
    d: &Dims,
    land: &[bool],
    true_km: &(dyn Fn(usize) -> f64 + Sync),
    blur: usize,
) -> Vec<f64> {
    let (w, h) = (d.w, d.h);
    let mut a = vec![0.0f64; w * h];
    a.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
        let l = &land[j * w..(j + 1) * w];
        if !l.iter().any(|&x| x) {
            return;
        }
        // Distance (cells) to the nearest land westward and eastward, wrapping.
        let mut dw = vec![0usize; w];
        let mut de = vec![0usize; w];
        let start = l.iter().position(|&x| x).expect("some land");
        let mut run = 0usize;
        for k in 1..=w {
            let i = (start + k) % w;
            if l[i] {
                run = 0;
            } else {
                run += 1;
            }
            dw[i] = run;
        }
        run = 0;
        for k in 1..=w {
            let i = (start + w - k) % w;
            if l[i] {
                run = 0;
            } else {
                run += 1;
            }
            de[i] = run;
        }
        let km = true_km(j);
        for i in 0..w {
            if l[i] {
                continue;
            }
            let (x, y) = (dw[i] as f64, de[i] as f64);
            let width = (x + y) * km;
            let wide = smoothstep(800.0, 3500.0, width);
            row[i] = wide * (x - y) / (x + y).max(1.0);
        }
    });
    if blur > 0 {
        // Normalised blur over ocean cells only.
        let mut wt: Vec<f64> = land.iter().map(|&x| if x { 0.0 } else { 1.0 }).collect();
        box_blur(d, &mut a, blur, 2);
        box_blur(d, &mut wt, blur, 2);
        for (c, v) in a.iter_mut().enumerate() {
            *v = if land[c] || wt[c] < 1e-6 {
                0.0
            } else {
                (*v / wt[c]).clamp(-1.0, 1.0)
            };
        }
    }
    a
}

/// The gyres' sea-surface anomaly at basin position `a` and latitude `phi`, °C.
pub fn sst_anomaly(p: &ClimateParams, a: f64, phi: f64) -> f64 {
    let (east, west) = (a.max(0.0), (-a).max(0.0));
    subtropics(phi) * (west * p.sst_subtropical_warm_c - east * p.sst_subtropical_cold_c)
        + midlatitudes(phi) * (east * p.sst_midlatitude_warm_c - west * p.sst_midlatitude_cold_c)
}

/// Precipitation factor of the subtropical highs at basin position `a` and belt latitude `phi_b`:
/// drier under their eastern flanks, wetter under their western (monsoonal) flanks.
pub fn subsidence_factor(p: &ClimateParams, a: f64, phi_b: f64) -> f64 {
    let s = subtropics(phi_b);
    (1.0 - p.subsidence * s * a.max(0.0) + 0.4 * s * (-a).max(0.0)).max(0.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn belts_have_their_wet_and_dry_latitudes() {
        assert!(belt_precip(0.0) > 3.0 * belt_precip(25.0));
        assert!(belt_precip(50.0) > 2.0 * belt_precip(25.0));
        assert!(belt_precip(50.0) > 3.0 * belt_precip(88.0));
        assert_eq!(belt_precip(-30.0), belt_precip(30.0));
        // Weights sum to one; trades in the tropics, westerlies at 45°, polar at 75°.
        for &(phi, pb) in &[
            (10.0, 10.0),
            (45.0, 45.0),
            (75.0, 75.0),
            (5.0, -3.0),
            (-40.0, -36.0),
        ] {
            let w = regime_weights(phi, pb);
            assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-12, "{phi} {pb}");
        }
        assert_eq!(regime_weights(10.0, 10.0)[NE_TRADES], 1.0);
        assert_eq!(regime_weights(45.0, 45.0)[WESTERLIES_N], 1.0);
        assert_eq!(regime_weights(-75.0, -75.0)[POLAR_S], 1.0);
        // North of the equator but south of a northward ITCZ: the SW monsoon.
        assert_eq!(regime_weights(12.0, -6.0)[MONSOON_N], 1.0);
        // Trade winds blow west, westerlies east.
        let p = ClimateParams::default();
        let wv = winds(&p);
        assert!(wv[NE_TRADES].0 < 0.0 && wv[WESTERLIES_N].0 > 0.0 && wv[MONSOON_N].0 > 0.0);
        for (r, &e) in EASTWARD.iter().enumerate() {
            assert_eq!(e, wv[r].0 > 0.0, "{}", REGIME_NAMES[r]);
        }
        // July: highs shifted north; January: south.
        assert!(hemispheric_shift(&p, 23.4, 6) > 4.0);
        assert!(hemispheric_shift(&p, 23.4, 0) < -4.0);
        assert!(hemispheric_shift(&p, 0.0, 6).abs() < 1e-12);
    }

    #[test]
    fn basins_know_their_sides() {
        // One row: land at 0..10, ocean 10..100 (a 90-cell basin).
        let d = Dims::new(100, 1, 1.0, true);
        let land: Vec<bool> = (0..100).map(|i| i < 10).collect();
        let a = basin_position(&d, &land, &|_| 100.0, 0);
        assert!(
            a[11] < -0.9,
            "west side of the basin (east coast): {}",
            a[11]
        );
        assert!(
            a[98] > 0.9,
            "east side of the basin (west coast): {}",
            a[98]
        );
        assert!(a[55].abs() < 0.05);
        assert_eq!(a[5], 0.0);
        let p = ClimateParams::default();
        assert!(
            sst_anomaly(&p, 1.0, 25.0) < -4.0,
            "cold upwelling off west coasts"
        );
        assert!(
            sst_anomaly(&p, 1.0, 55.0) > 5.0,
            "warm drift off west coasts"
        );
        assert!(sst_anomaly(&p, -1.0, 55.0) < -4.0);
        assert!(subsidence_factor(&p, 1.0, 25.0) < 0.5);
        assert!(subsidence_factor(&p, -1.0, 25.0) > 1.3);
    }
}

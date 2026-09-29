// SPDX-License-Identifier: LGPL-3.0-or-later
//! Orographic precipitation: Smith & Barstad 2004's linear theory (plan §8.3 item 4), ported from
//! `fastscape-lem/orographic-precipitation` (MIT, © 2020 Raphael Lange; commit baf8a0c,
//! `orographic_precipitation/orographic_precipitation.py`, `compute_orographic_precip`; NOTICE.md).
//!
//! In Fourier space the precipitation is one formula (Smith & Barstad 2004, eq. 49):
//!
//! P̂(k, l) = c_w · i σ ĥ / ((1 − i m H_w)(1 + i σ τ_c)(1 + i σ τ_f)),  σ = U k + V l,
//! m = sign(σ) · √(|(N_m² − σ²)/(σ² − f²)| · (k² + l²)),
//!
//! with the numerator clipped at 0 and the denominator kept at least machine epsilon away from 0,
//! as the original does; the inverse transform's real part × 3,600 is the orographic rate in mm/h
//! (negative in rain shadows; the caller adds the background rate and clips).
//!
//! **Differences from the Python module.** (1) The raster wraps in x (the planet), so only the
//! rows are zero-padded (`pad` rows above and below), not the columns. (2) The wavenumbers use the
//! true lengths of each axis: the Python module builds k from `x_len = nx·dx` with `nx` the row
//! count (its lengths are swapped on non-square rasters). (3) Grid axes here are x east and y
//! **south** (row index), and the wind is given as (U east, V south) in m/s instead of a speed and
//! an azimuth. (4) f comes from the planet's day length. The FFTs are rustfft's (MIT OR
//! Apache-2.0), run row by row and column by column in parallel; every 1-D transform is independent,
//! so the result does not depend on the thread count.

use rayon::prelude::*;
use rustfft::FftPlanner;
use rustfft::num_complex::Complex64;

/// The model's constants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OroParams {
    /// Cloud-water to hydrometeor conversion time τ_c, s.
    pub conv_time_s: f64,
    /// Hydrometeor fallout time τ_f, s.
    pub fall_time_s: f64,
    /// Moist stability frequency N_m, 1/s.
    pub nm: f64,
    /// Water-vapour scale height H_w, m.
    pub hw_m: f64,
    /// Uplift sensitivity c_w, kg/m³.
    pub cw: f64,
}

/// Orographic precipitation rate (mm/h, may be negative) over `rows` × `w` cells of terrain `h`
/// (m above sea level, 0 at sea; row-major), with cells `dx` × `dy` metres, wind (u east, v south)
/// m/s and Coriolis parameter `f` (1/s). x wraps; `pad` zero rows go above and below.
#[allow(clippy::too_many_arguments)]
pub fn precipitation(
    h: &[f64],
    w: usize,
    rows: usize,
    dx: f64,
    dy: f64,
    u: f64,
    v: f64,
    f: f64,
    p: &OroParams,
    pad: usize,
) -> Vec<f64> {
    assert_eq!(h.len(), w * rows, "terrain size");
    if w == 0 || rows == 0 {
        return Vec::new();
    }
    let ny = rows + 2 * pad;
    let nx = w;
    let mut planner = FftPlanner::<f64>::new();
    let fx = planner.plan_fft_forward(nx);
    let fy = planner.plan_fft_forward(ny);
    let ix = planner.plan_fft_inverse(nx);
    let iy = planner.plan_fft_inverse(ny);

    // Row-major ny × nx, then transposed nx × ny for the column transforms.
    let mut a = vec![Complex64::new(0.0, 0.0); ny * nx];
    for j in 0..rows {
        for i in 0..nx {
            a[(j + pad) * nx + i] = Complex64::new(h[j * w + i], 0.0);
        }
    }
    a.par_chunks_mut(nx).for_each(|row| fx.process(row));
    let mut t = transpose(&a, ny, nx);
    t.par_chunks_mut(ny).for_each(|col| fy.process(col));

    // The transfer function, on the transposed layout (column i of kx, entry j of ky).
    let freq = |k: usize, n: usize| -> f64 {
        if k < n.div_ceil(2) {
            k as f64
        } else {
            k as f64 - n as f64
        }
    };
    let two_pi = 2.0 * std::f64::consts::PI;
    let eps = f64::EPSILON;
    let one = Complex64::new(1.0, 0.0);
    let i1 = Complex64::new(0.0, 1.0);
    t.par_chunks_mut(ny).enumerate().for_each(|(ci, col)| {
        let kx = two_pi * freq(ci, nx) / (nx as f64 * dx);
        for (rj, z) in col.iter_mut().enumerate() {
            let ky = two_pi * freq(rj, ny) / (ny as f64 * dy);
            let sigma = kx * u + ky * v;
            let mut num = p.nm * p.nm - sigma * sigma;
            if num < 0.0 {
                num = 0.0;
            }
            let mut den = sigma * sigma - f * f;
            if (0.0..eps).contains(&den) {
                den = eps;
            } else if den > -eps && den < 0.0 {
                den = -eps;
            }
            let sign = if sigma >= 0.0 { 1.0 } else { -1.0 };
            let m = sign * (num / den * (kx * kx + ky * ky)).abs().sqrt();
            let top = p.cw * i1 * sigma * *z;
            let bottom = (one - i1 * (p.hw_m * m))
                * (one + i1 * (sigma * p.conv_time_s))
                * (one + i1 * (sigma * p.fall_time_s));
            *z = top / bottom;
        }
    });

    t.par_chunks_mut(ny).for_each(|col| iy.process(col));
    let mut b = transpose(&t, nx, ny);
    b.par_chunks_mut(nx).for_each(|row| ix.process(row));
    let norm = 3600.0 / (nx * ny) as f64;
    let mut out = vec![0.0; rows * w];
    for j in 0..rows {
        for i in 0..w {
            out[j * w + i] = b[(j + pad) * nx + i].re * norm;
        }
    }
    out
}

/// `a` is r × c row-major; returns c × r row-major.
fn transpose(a: &[Complex64], r: usize, c: usize) -> Vec<Complex64> {
    let mut t = vec![Complex64::new(0.0, 0.0); r * c];
    t.par_chunks_mut(r).enumerate().for_each(|(i, col)| {
        for (j, x) in col.iter_mut().enumerate() {
            *x = a[j * c + i];
        }
    });
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> OroParams {
        OroParams {
            conv_time_s: 1000.0,
            fall_time_s: 1000.0,
            nm: 0.01,
            hw_m: 3400.0,
            cw: 7.4e-3 * 7.0 / 4.0,
        }
    }

    /// A north–south ridge across a westerly wind: more rain on the windward (west) flank than on
    /// the lee (east) flank, which lies in a rain shadow (negative orographic rate).
    #[test]
    fn a_ridge_across_the_wind_is_wetter_windward() {
        let (w, rows, dx) = (128usize, 64usize, 4800.0);
        let x0 = 64.0;
        let h: Vec<f64> = (0..rows * w)
            .map(|c| {
                let i = (c % w) as f64;
                2000.0 * (-((i - x0) / 6.0).powi(2)).exp()
            })
            .collect();
        let f = 2.0 * 7.2921e-5 * 45f64.to_radians().sin();
        let p = precipitation(&h, w, rows, dx, dx, 10.0, 0.0, f, &params(), 16);
        let j = rows / 2;
        let windward: f64 = (50..64).map(|i| p[j * w + i]).sum::<f64>() / 14.0;
        let lee: f64 = (65..79).map(|i| p[j * w + i]).sum::<f64>() / 14.0;
        assert!(windward > 0.5, "windward {windward} mm/h");
        assert!(lee < windward / 3.0, "lee {lee} vs windward {windward}");
        assert!(p[j * w + 72] < 0.0, "rain shadow {}", p[j * w + 72]);
        // An easterly wind mirrors it.
        let q = precipitation(&h, w, rows, dx, dx, -10.0, 0.0, f, &params(), 16);
        for i in 40..88 {
            let mirror = (2.0 * x0) as usize - i;
            assert!(
                (q[j * w + i] - p[j * w + mirror]).abs() < 1e-6 * (1.0 + p[j * w + mirror].abs())
            );
        }
        // Flat terrain: no orographic rain.
        let flat = precipitation(
            &vec![0.0; rows * w],
            w,
            rows,
            dx,
            dx,
            10.0,
            0.0,
            f,
            &params(),
            8,
        );
        assert!(flat.iter().all(|x| x.abs() < 1e-12));
    }
}

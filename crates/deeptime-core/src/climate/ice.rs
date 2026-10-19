// SPDX-License-Identifier: LGPL-3.0-or-later
//! Land-ice thickness, its volume and the sea level it takes (deep-time climate, decisions log
//! 2026-09-29 "Recent ice-age cycles" and "Ice-linked sea level").
//!
//! - **Thickness.** Where the glacier model puts ice (land above its equilibrium line, `glaciers.rs`),
//!   the ice takes the perfectly plastic profile of Nye (1952, "The mechanics of glacier flow",
//!   J. Glaciol. 2: 82–93; Cuffey & Paterson 2010, *The Physics of Glaciers*, §8.7):
//!   H = √(2 τ₀ L / (ρ_i g)), L the distance to the ice margin, τ₀ the basal yield stress
//!   (`ice_yield_stress_kpa`, 50 kPa: Antarctica's ~3.3 km over ~1,500 km gives ~32 kPa,
//!   Greenland's ~3 km over ~500 km ~80 kPa), capped at `ice_max_thickness_m` (3 km: real sheets
//!   flatten where interior accumulation is low; Antarctica averages ~2.1 km under ~4-km domes). Distances are true
//!   Earth-normalized lengths (the climate's planetary scale: map metres ÷ R/R⊕, cos φ across a
//!   row), so ice sheets are as thick as an Earth sheet of that angular size and the eustatic sea
//!   level is Earth's for that ice. The distance is a separable exact Euclidean distance transform
//!   (Felzenszwalb & Huttenlocher 2012, "Distance transforms of sampled functions", Theory of
//!   Computing 8: 415–428; implemented from the paper), rows first (wrapping in x) then columns.
//! - **Ice sheets** (`ice_dynamics`, off by default: decisions log 2026-10-02): the flowing sheets of
//!   `sheet.rs` (shallow-ice flow, elevation feedback, isostasy, calving) on a coarse grid; the ice
//!   of a cell is the sheet's or this equilibrium profile's, whichever is thicker (the equilibrium
//!   keeps the valley and plateau glaciers the coarse grid cannot see).
//! - **Lag** (`ice_dynamics` off, the default; the recent ice-age window only): the ice relaxes toward that
//!   equilibrium as in
//!   Imbrie & Imbrie (1980, "Modeling the climatic response to orbital variations", Science 207:
//!   943–953): dH/dt = (H_eq − H)/τ with τ = τ_m/(1 − b) while the ice grows and τ_m/(1 + b) while
//!   it shrinks (τ_m = `ice_response_kyr`, 17 kyr; b = 0.6): slow build-up, fast terminations.
//! - **Glacio-eustasy.** Grounded ice lowers the sea by (ρ_i/ρ_w)·ΔV / A_ocean — about 2.5 m per
//!   10⁶ km³ of ice on Earth's 3.61·10⁸ km² of ocean — measured from `ice_reference_km3`, the ice
//!   the preset's sea-level curve already assumes (the present ice of the calibrated Earth-like
//!   runs, so an Earth-like present keeps its calibrated coastline).

use rayon::prelude::*;

/// Density of glacier ice and of sea water, kg/m³.
pub const RHO_ICE: f64 = 917.0;
pub const RHO_SEA: f64 = 1028.0;
/// Earth's standard gravity, m/s² (the planet's own, `planet.gravity_m_s2`, is what the ice
/// takes: P1-26 T6, science stage 1 fix 17).
pub const G: f64 = 9.81;
/// Imbrie & Imbrie's asymmetry b.
pub const IMBRIE_B: f64 = 0.6;

const INF: f64 = 1.0e30;

/// Felzenszwalb & Huttenlocher's 1-D squared distance transform of `f` sampled at spacing `s`:
/// d(p) = min_q (s·(p − q))² + f(q). `f` holds 0 at sites and `INF` elsewhere (or any sampled
/// function). `v`, `z` are scratch.
fn dt_1d(f: &[f64], s: f64, out: &mut [f64], v: &mut Vec<usize>, z: &mut Vec<f64>) {
    v.clear();
    z.clear();
    let s2 = s * s;
    // Parabola q: s²(p − q)² + f(q). Intersection of parabolas q and r (q > r):
    // ((f(q) + s²q²) − (f(r) + s²r²)) / (2 s² (q − r)).
    let inter = |q: usize, r: usize| -> f64 {
        ((f[q] + s2 * (q * q) as f64) - (f[r] + s2 * (r * r) as f64)) / (2.0 * s2 * (q - r) as f64)
    };
    for (q, &fq) in f.iter().enumerate() {
        if fq >= INF {
            continue;
        }
        loop {
            let Some(&r) = v.last() else {
                v.push(q);
                z.push(f64::NEG_INFINITY);
                break;
            };
            let x = inter(q, r);
            if x <= *z.last().expect("z parallels v") {
                v.pop();
                z.pop();
                continue;
            }
            v.push(q);
            z.push(x);
            break;
        }
    }
    if v.is_empty() {
        out.iter_mut().for_each(|o| *o = INF);
        return;
    }
    let mut k = 0;
    for (p, o) in out.iter_mut().enumerate() {
        while k + 1 < v.len() && z[k + 1] < p as f64 {
            k += 1;
        }
        let q = v[k];
        let dp = p as f64 - q as f64;
        *o = s2 * dp * dp + f[q];
    }
}

/// [`dt_1d`] for samples at increasing positions `pos` (m) instead of a fixed spacing:
/// d(p) = min_q (pos_p − pos_q)² + f(q). The column pass of the Lambert grid, whose rows are
/// unevenly spaced on the sphere (1 / cos φ).
fn dt_1d_at(f: &[f64], pos: &[f64], out: &mut [f64], v: &mut Vec<usize>, z: &mut Vec<f64>) {
    v.clear();
    z.clear();
    let inter = |q: usize, r: usize| -> f64 {
        ((f[q] + pos[q] * pos[q]) - (f[r] + pos[r] * pos[r])) / (2.0 * (pos[q] - pos[r]))
    };
    for (q, &fq) in f.iter().enumerate() {
        if fq >= INF {
            continue;
        }
        loop {
            let Some(&r) = v.last() else {
                v.push(q);
                z.push(f64::NEG_INFINITY);
                break;
            };
            let x = inter(q, r);
            if x <= *z.last().expect("z parallels v") {
                v.pop();
                z.pop();
                continue;
            }
            v.push(q);
            z.push(x);
            break;
        }
    }
    if v.is_empty() {
        out.iter_mut().for_each(|o| *o = INF);
        return;
    }
    let mut k = 0;
    for (p, o) in out.iter_mut().enumerate() {
        while k + 1 < v.len() && z[k + 1] < pos[p] {
            k += 1;
        }
        let q = v[k];
        let dp = pos[p] - pos[q];
        *o = dp * dp + f[q];
    }
}

/// Distance, m, from every ice cell to the nearest ice-free cell (0 on ice-free cells), on a
/// w × h grid whose row `j`'s cells are `sx[j]` metres wide and `sy[j]` metres high (rows are
/// (sy[j] + sy[j + 1]) / 2 apart along a column; the Lambert grid's rows are uneven on the
/// sphere); x wraps. Rows in parallel, then columns in parallel: deterministic.
pub fn distance_to_margin(w: usize, h: usize, ice: &[bool], sx: &[f64], sy: &[f64]) -> Vec<f64> {
    let mut rows = vec![0.0; w * h];
    rows.par_chunks_mut(w).enumerate().for_each(|(j, out)| {
        // Three copies of the row so the transform sees across the seam.
        let f: Vec<f64> = (0..3 * w)
            .map(|k| if ice[j * w + k % w] { INF } else { 0.0 })
            .collect();
        let mut d = vec![0.0; 3 * w];
        let (mut v, mut z) = (Vec::new(), Vec::new());
        dt_1d(&f, sx[j].max(1e-6), &mut d, &mut v, &mut z);
        out.copy_from_slice(&d[w..2 * w]);
    });
    // Row centres along a column, m (strictly increasing: a row wholly past a pole counts 1 µm).
    let mut pos = vec![0.0; h];
    for j in 1..h {
        pos[j] = pos[j - 1] + (0.5 * (sy[j - 1] + sy[j])).max(1e-6);
    }
    let cols: Vec<Vec<f64>> = (0..w)
        .into_par_iter()
        .map(|i| {
            let f: Vec<f64> = (0..h).map(|j| rows[j * w + i]).collect();
            let mut d = vec![0.0; h];
            let (mut v, mut z) = (Vec::new(), Vec::new());
            dt_1d_at(&f, &pos, &mut d, &mut v, &mut z);
            d
        })
        .collect();
    // Back into rows, rows in parallel (P1-17).
    let mut out = vec![0.0; w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
        for (x, col) in row.iter_mut().zip(&cols) {
            *x = col[j].min(INF).sqrt();
        }
    });
    out
}

/// Nye's equilibrium thickness at distance `l_m` from the margin under gravity `g` m/s², m.
pub fn nye(l_m: f64, tau_pa: f64, cap_m: f64, g: f64) -> f64 {
    (2.0 * tau_pa * l_m.max(0.0) / (RHO_ICE * g))
        .sqrt()
        .min(cap_m)
}

/// Thickness of every cell of an ice mask, m: Nye's profile at the distance to the margin (the
/// margin taken half a cell outside the last ice cell's centre), under gravity `g` m/s².
#[allow(clippy::too_many_arguments)]
pub fn thickness(
    w: usize,
    h: usize,
    ice: &[bool],
    sx: &[f64],
    sy: &[f64],
    tau_pa: f64,
    cap_m: f64,
    g: f64,
) -> Vec<f32> {
    if !ice.iter().any(|&x| x) {
        return crate::alloc::filled(w * h, 0.0);
    }
    let d = distance_to_margin(w, h, ice, sx, sy);
    d.par_iter()
        .zip(ice.par_iter())
        .enumerate()
        .map(|(c, (&dist, &on))| {
            if !on {
                return 0.0;
            }
            let half = 0.5 * sx[c / w].min(sy[c / w]);
            nye((dist - half).max(half), tau_pa, cap_m, g) as f32
        })
        .collect()
}

/// The surface mass balance of ice whose surface stands `above_m` over its equilibrium line, m of
/// ice/yr, under `precip_m_yr` of precipitation: β_a·above up to `acc_max`·P above the line,
/// β_m·above below it (Oerlemans 1981's linear balance gradients, GLA-069; the accumulation cap,
/// GLA-070). One law for the climate's balance field, the ice sheets' step (`sheet.rs`) and the
/// bake's cover (`bake/ice.rs`).
#[inline]
pub fn surface_balance(
    above_m: f64,
    precip_m_yr: f64,
    beta_acc: f64,
    beta_abl: f64,
    acc_max: f64,
) -> f64 {
    if above_m >= 0.0 {
        (beta_acc * above_m).min(acc_max * precip_m_yr.max(0.0))
    } else {
        beta_abl * above_m
    }
}

/// One Imbrie & Imbrie step of `dt_kyr` toward `eq` (m) with response time `tau_kyr`.
pub fn relax(h: f64, eq: f64, dt_kyr: f64, tau_kyr: f64) -> f64 {
    let tau = if eq > h {
        tau_kyr / (1.0 - IMBRIE_B)
    } else {
        tau_kyr / (1.0 + IMBRIE_B)
    };
    h + (eq - h) * (1.0 - (-dt_kyr / tau.max(1e-9)).exp())
}

/// Sea level change, m, of `dv_km3` more grounded ice than the reference on `ocean_km2` of sea
/// (both Earth-normalized): −(ρ_i/ρ_w)·ΔV/A.
pub fn eustatic_m(dv_km3: f64, ocean_km2: f64) -> f64 {
    if ocean_km2 <= 0.0 {
        return 0.0;
    }
    -(RHO_ICE / RHO_SEA) * dv_km3 / ocean_km2 * 1000.0
}

/// Each EBM band's land cells' margins μ = z − ELA (m above the line; sorted), so the share of a
/// band's land under ice for an ELA shift δ is the share with μ > δ.
#[derive(Clone, Debug, Default)]
pub struct BandMargins {
    pub band: Vec<Vec<f32>>,
}

impl BandMargins {
    /// From the margins `mu` of every cell (NaN at sea) and each cell's band.
    pub fn new(
        mu: &[f32],
        band_of_cell: &(dyn Fn(usize) -> usize + Sync),
        bands: usize,
    ) -> BandMargins {
        // P1-19 (#18): chunks of cells split their finite margins by band side by side, then each
        // band joins its chunks' and sorts, bands side by side. The sort's order is total, so the
        // lists are those of one serial pass (equal margins are the same bits).
        const CHUNK: usize = 1 << 16;
        let parts: Vec<Vec<Vec<f32>>> = mu
            .par_chunks(CHUNK)
            .enumerate()
            .map(|(k, cells)| {
                let mut band = vec![Vec::new(); bands];
                for (i, &m) in cells.iter().enumerate() {
                    if m.is_finite() {
                        band[band_of_cell(k * CHUNK + i)].push(m);
                    }
                }
                band
            })
            .collect();
        let band: Vec<Vec<f32>> = (0..bands)
            .into_par_iter()
            .map(|b| {
                let mut v = Vec::with_capacity(parts.iter().map(|p| p[b].len()).sum());
                for p in &parts {
                    v.extend_from_slice(&p[b]);
                }
                v.par_sort_unstable_by(f32::total_cmp);
                v
            })
            .collect();
        BandMargins { band }
    }

    /// Share of each band's land above its line after the shifts `delta` (0 where no land).
    pub fn shares(&self, delta: &[f64]) -> Vec<f64> {
        self.band
            .iter()
            .zip(delta)
            .map(|(v, &d)| {
                if v.is_empty() {
                    return 0.0;
                }
                let below = v.partition_point(|&m| f64::from(m) <= d);
                (v.len() - below) as f64 / v.len() as f64
            })
            .collect()
    }
}

/// The clock steps' shifts of one band, sorted, with the time of every shift at or above each:
/// the share of the plate step a cell of margin μ spent under ice is the time with δ < μ.
#[derive(Clone, Debug, Default)]
pub struct ShiftTimes {
    /// (δ, cumulative time of all shifts ≤ δ).
    pub sorted: Vec<(f64, f64)>,
    pub total: f64,
}

impl ShiftTimes {
    pub fn new(steps: &[(f64, f64)]) -> ShiftTimes {
        let mut v: Vec<(f64, f64)> = steps.to_vec();
        v.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut acc = 0.0;
        let sorted = v
            .into_iter()
            .map(|(d, dt)| {
                acc += dt;
                (d, acc)
            })
            .collect();
        ShiftTimes { sorted, total: acc }
    }

    /// Share of the time with δ < `mu`.
    pub fn share(&self, mu: f64) -> f64 {
        if self.total <= 0.0 {
            return 0.0;
        }
        let k = self.sorted.partition_point(|&(d, _)| d < mu);
        if k == 0 {
            0.0
        } else {
            self.sorted[k - 1].1 / self.total
        }
    }

    /// The lowest shift (the largest ice).
    pub fn min(&self) -> f64 {
        self.sorted.first().map_or(0.0, |x| x.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_balance_is_capped_above_the_line_and_steeper_below() {
        let law = |above, p| surface_balance(above, p, 0.005, 0.01, 0.5);
        assert_eq!(law(0.0, 1.0), 0.0);
        assert!((law(100.0, 1.0) - 0.5).abs() < 1e-12);
        // The cap: 0.5 m/yr per m/yr of precipitation, so a dry sheet gains little.
        assert!((law(1000.0, 0.2) - 0.1).abs() < 1e-12);
        assert_eq!(law(1000.0, -1.0), 0.0);
        // Below the line the melt grows twice as fast per metre.
        assert!((law(-100.0, 1.0) + 1.0).abs() < 1e-12);
        // Concave: lowering half of a body by d and raising the other half by d loses balance.
        assert!(law(50.0 + 200.0, 1.0) + law(50.0 - 200.0, 1.0) < 2.0 * law(50.0, 1.0));
    }

    #[test]
    fn band_margins_and_shift_times_count_ice() {
        let mu = [f32::NAN, -100.0, 50.0, 300.0, -20.0];
        let m = BandMargins::new(&mu, &|c| usize::from(c >= 3), 2);
        // Band 0: −100, 50; band 1: −20, 300.
        assert_eq!(m.shares(&[0.0, 0.0]), vec![0.5, 0.5]);
        assert_eq!(m.shares(&[-200.0, 400.0]), vec![1.0, 0.0]);
        let s = ShiftTimes::new(&[(100.0, 1.0), (-50.0, 3.0), (0.0, 1.0)]);
        assert_eq!(s.share(-60.0), 0.0);
        assert!((s.share(10.0) - 0.8).abs() < 1e-12);
        assert_eq!(s.share(1000.0), 1.0);
        assert_eq!(s.min(), -50.0);
    }

    #[test]
    fn the_distance_transform_is_exact_on_a_uniform_grid() {
        let (w, h) = (40, 20);
        let mut ice = vec![true; w * h];
        // Ice-free cells: one at (5, 5), and the whole column 30.
        ice[5 * w + 5] = false;
        for j in 0..h {
            ice[j * w + 30] = false;
        }
        let sx = vec![2.0; h];
        let d = distance_to_margin(w, h, &ice, &sx, &vec![3.0; h]);
        for j in 0..h {
            for i in 0..w {
                let dx5 = (i as i64 - 5)
                    .rem_euclid(w as i64)
                    .min((5 - i as i64).rem_euclid(w as i64));
                let a = ((dx5 as f64 * 2.0).powi(2) + ((j as f64 - 5.0) * 3.0).powi(2)).sqrt();
                let di = (i as i64 - 30)
                    .rem_euclid(w as i64)
                    .min((30 - i as i64).rem_euclid(w as i64));
                let b = di as f64 * 2.0;
                let want = a.min(b);
                assert!(
                    (d[j * w + i] - want).abs() < 1e-9,
                    "({i},{j}) {} vs {want}",
                    d[j * w + i]
                );
            }
        }
        // All ice: every distance is huge; no ice: zero.
        let none = distance_to_margin(4, 3, &[false; 12], &[1.0; 3], &[1.0; 3]);
        assert!(none.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn the_column_pass_measures_uneven_rows() {
        // One ice-free cell at row 0; rows 1, 2, 3 m high below it (the Lambert grid's rows
        // grow towards the poles): the distance down the column is the sum of half-heights.
        let (w, h) = (1, 4);
        let ice = [false, true, true, true];
        let sy = [1.0, 1.0, 2.0, 3.0];
        let d = distance_to_margin(w, h, &ice, &[1.0; 4], &sy);
        assert!(
            (d[1] - 1.0).abs() < 1e-12 && (d[2] - 2.5).abs() < 1e-12 && (d[3] - 5.0).abs() < 1e-12,
            "{d:?}"
        );
    }

    #[test]
    fn nye_and_eustasy_have_earths_numbers() {
        // 1,000 km from the margin at 50 kPa: ~3.3 km.
        let hh = nye(1.0e6, 5.0e4, 1.0e9, G);
        assert!((3200.0..3400.0).contains(&hh), "{hh}");
        assert_eq!(nye(1.0e7, 5.0e4, 4000.0, G), 4000.0);
        // On a Mars-gravity planet the same margin distance and stress hold √(9.81/3.71) = 1.63
        // times the ice (design §4.6).
        let mars = nye(1.0e6, 5.0e4, 1.0e9, 3.71);
        assert!((mars / hh - (9.81f64 / 3.71).sqrt()).abs() < 1e-12);
        // 10⁶ km³ on Earth's ocean: about 2.5 m.
        let dz = eustatic_m(1.0e6, 3.61e8);
        assert!((dz + 2.47).abs() < 0.03, "{dz}");
        // The Imbrie lag: growth slower than decay.
        let up = relax(0.0, 1000.0, 10.0, 17.0);
        let down = relax(1000.0, 0.0, 10.0, 17.0);
        assert!(up < 1000.0 - down, "grew {up}, shrank {}", 1000.0 - down);
    }

    #[test]
    fn a_round_cap_is_dome_shaped() {
        let (w, h) = (64, 32);
        let ice: Vec<bool> = (0..w * h)
            .map(|c| {
                let (i, j) = ((c % w) as f64 - 32.0, (c / w) as f64 - 16.0);
                i * i + j * j < 100.0
            })
            .collect();
        let t = thickness(
            w,
            h,
            &ice,
            &vec![10_000.0; h],
            &vec![10_000.0; h],
            5.0e4,
            4000.0,
            G,
        );
        let centre = t[16 * w + 32];
        let edge = t[16 * w + 41];
        assert!(centre > 2.0 * edge && edge > 0.0, "{centre} {edge}");
        assert_eq!(t[0], 0.0);
    }
}

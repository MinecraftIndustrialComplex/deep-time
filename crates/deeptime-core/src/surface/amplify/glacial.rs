// SPDX-License-Identifier: LGPL-3.0-or-later
//! Glacial valleys at block scale (Ben, 2026-09-29, "Glacial erosion landforms"): the amplifier
//! turns the valleys under the young landscape's glaciers (`young/ice_surface_m`, the glacial
//! state's ice, `surface/glacial.rs`) into U-shaped troughs, their heads into cirque-like bowls,
//! and gives the carved rock back to the ice-free ridges around, so that the coarse level's
//! mean height is kept and the ridges between troughs sharpen into arêtes and horns.
//!
//! # The trough
//! A glacier's bed evolves towards a cross-section that erodes evenly everywhere under the ice, a
//! parabola y = a·x^b with b ≈ 2 (Svensson 1959, J. Glaciol. 3:362–363, fitted b = 2.0–2.2;
//! Harbor 1992, GSA Bull. 104:1364–1375, showed V-shaped valleys evolving into it under
//! sliding-law erosion). The amplifier makes that shape pointwise from the ice depth
//! D = z_ice − z (the ice surface interpolated from the level below, smooth like a real glacier's)
//! and the valley's centre-line depth D_max (the largest D within `radius_cells`): with
//! t = D / D_max, a V (D falling linearly from the axis to the margin) has t = 1 − 2|x|/W and the
//! parabola with the same axis and margins has depth D_max·(1 − (1 − t)²), so each cell is
//! lowered by
//!
//! Δ = s · g(D_max) · D_max · t · (1 − t),
//!
//! (s the strength, g a ramp from `min_ice_m` to 3× it), times two shares that keep it to
//! what makes troughs:
//! - **confined** ice: ridges within `radius_cells` stand above the ice surface (by a quarter of
//!   D_max for the full effect). An ice sheet that drowns the relief scours its bed evenly
//!   instead of cutting troughs (Sugden 1978, "Glacial erosion by the Laurentide ice sheet",
//!   J. Glaciol. 20:367–391: areal scouring under the sheet, selective linear erosion in its
//!   outlet troughs), so it is left as the coarse level made it;
//! - **lateral** margins: the ice thins across the flow (a valley wall), not along it (a
//!   terminus or a tidewater front): |sin| of the angle between the gradients of D and of the
//!   ice surface.
//!
//! The axis (t = 1) and the ice margin (t = 0, the trimline) stay where they are; the valley
//! walls between them are cut back to the parabola, most at mid-height (D_max/4). The bowl at a
//! glacier's head, where the ice thins to nothing against its headwall across the flow, is the
//! same parabola: a cirque.
//!
//! # The ridges
//! The carved volume is spread (share `ridge_share`) over the ice-free cells within
//! `radius_cells` that stand above their surroundings (spurs and crests, not a valley floor
//! beyond a terminus: a box mean of Δ over one of that mask), raising them at most
//! `max_ridge_raise_m`: the level below modelled cell means, and relief it did not resolve is
//! trough below and arête above that mean (the "glacial buzzsaw": mean heights near the ELA,
//! peaks above it, Egholm et al. 2009, Nature 460:884–887; relief above cirque floors rarely over
//! 600 m, Mitchell & Humphries 2015, Geology 43:35–38).
//!
//! # Rivers under the ice
//! Fluvial incision is cut by `fluvial_damping` where the glaciers were (the valleys have had
//! ~10–20 kyr since the ice left: their floors are glacial, not river-cut).
//!
//! # Tiles
//! Every step is a pure function of a bounded neighbourhood (max and mean filters of radius R and
//! one-cell gradients, then two box means of radius R), summed cell by cell in a fixed order, so
//! a tile with a halo of
//! [`GlacialShape::radius`] more cells gives the whole raster's bits (tested in `amplify/tests`).

use serde::{Deserialize, Serialize};

use super::window::Win;
use crate::climate::ebm::smoothstep;
use crate::error::CoreError;

/// The `amplify.glacial` section: every key optional.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct GlacialShape {
    /// How far a cross-section moves towards the parabola: 0 turns block-scale glacial shaping
    /// off, 1 is the equilibrium trough (the editor's "Glacial valleys").
    pub strength: f64,
    /// Radius the valley's centre-line ice depth is looked for in, and the ridges within which
    /// get the carved rock back, cells.
    pub radius_cells: u32,
    /// Share of the carved rock the ice-free ridges around get back.
    pub ridge_share: f64,
    /// The most a ridge cell is raised, m.
    pub max_ridge_raise_m: f64,
    /// Share of the fluvial incision taken out under the ice.
    pub fluvial_damping: f64,
    /// Ice this shallow at the valley's axis shapes nothing, m (full effect at 3×).
    pub min_ice_m: f64,
}

impl Default for GlacialShape {
    fn default() -> Self {
        GlacialShape {
            strength: 1.0,
            radius_cells: 20,
            ridge_share: 0.5,
            max_ridge_raise_m: 150.0,
            fluvial_damping: 0.8,
            min_ice_m: 60.0,
        }
    }
}

impl GlacialShape {
    pub fn validate(&self, path: &str) -> Result<(), CoreError> {
        let bad = |k: &str, why: &str| CoreError::preset(format!("{path}.{k}"), why);
        if !(self.strength.is_finite() && (0.0..=2.0).contains(&self.strength)) {
            return Err(bad("strength", "must be in [0, 2]"));
        }
        if !(1..=64).contains(&self.radius_cells) {
            return Err(bad("radius_cells", "must be in [1, 64]"));
        }
        if !(0.0..=1.0).contains(&self.ridge_share) {
            return Err(bad("ridge_share", "must be in [0, 1]"));
        }
        if !(self.max_ridge_raise_m.is_finite() && self.max_ridge_raise_m >= 0.0) {
            return Err(bad("max_ridge_raise_m", "must be ≥ 0"));
        }
        if !(0.0..=1.0).contains(&self.fluvial_damping) {
            return Err(bad("fluvial_damping", "must be in [0, 1]"));
        }
        if !(self.min_ice_m.is_finite() && self.min_ice_m > 0.0) {
            return Err(bad("min_ice_m", "must be > 0"));
        }
        Ok(())
    }

    /// True when the pass changes anything.
    pub fn active(&self) -> bool {
        self.strength > 0.0
    }

    /// Dependency radius of [`shape`], cells (0 when off).
    pub fn radius(&self) -> usize {
        if self.active() {
            3 * self.radius_cells as usize + 1
        } else {
            0
        }
    }
}

/// Column of window cell `i + di` under the window's rules (wraps when the window holds every
/// column, else the edge is replicated).
#[inline]
fn col(win: &Win, i: usize, di: i64) -> usize {
    let x = i as i64 + di;
    if win.wraps() {
        x.rem_euclid(win.w as i64) as usize
    } else {
        x.clamp(0, win.w as i64 - 1) as usize
    }
}

/// Row `j + dj` clamped to the window (a planet edge has no rows beyond; an open edge is
/// replicated, which only the halo sees).
#[inline]
fn row(win: &Win, j: usize, dj: i64) -> usize {
    (j as i64 + dj).clamp(0, win.h as i64 - 1) as usize
}

/// Ice depth of every cell, m (0 outside the ice or without an ice surface).
fn depth(z: &[f32], zs: &[f32], out: &mut [f32]) {
    for ((o, &a), &b) in out.iter_mut().zip(zs).zip(z) {
        *o = if a.is_finite() { (a - b).max(0.0) } else { 0.0 };
    }
}

/// Separable filter of radius `r`: `max` (true) or the mean.
fn filter(win: &Win, v: &[f32], r: usize, max: bool, tmp: &mut [f32], out: &mut [f32]) {
    use rayon::prelude::*;
    let (w, r) = (win.w, r as i64);
    let n = (2 * r + 1) as f64;
    tmp.par_chunks_mut(w).enumerate().for_each(|(j, t)| {
        let src = &v[j * w..(j + 1) * w];
        for (i, o) in t.iter_mut().enumerate() {
            if max {
                let mut m = f32::MIN;
                for di in -r..=r {
                    m = m.max(src[col(win, i, di)]);
                }
                *o = m;
            } else {
                let mut s = 0.0f64;
                for di in -r..=r {
                    s += f64::from(src[col(win, i, di)]);
                }
                *o = (s / n) as f32;
            }
        }
    });
    out.par_chunks_mut(w).enumerate().for_each(|(j, o)| {
        for (i, x) in o.iter_mut().enumerate() {
            if max {
                let mut m = f32::MIN;
                for dj in -r..=r {
                    m = m.max(tmp[row(win, j, dj) * w + i]);
                }
                *x = m;
            } else {
                let mut s = 0.0f64;
                for dj in -r..=r {
                    s += f64::from(tmp[row(win, j, dj) * w + i]);
                }
                *x = (s / n) as f32;
            }
        }
    });
}

/// Glacier weight of an axis depth, 0–1.
#[inline]
fn ramp(p: &GlacialShape, dmax: f32) -> f32 {
    smoothstep(p.min_ice_m, 3.0 * p.min_ice_m, f64::from(dmax)) as f32
}

/// Multiplies `kfac` by the ice-free share (1 − `fluvial_damping` under deep ice); returns the
/// original to restore after the erosion passes, or None when there is no ice.
pub fn damp(p: &GlacialShape, z: &[f32], zs: &[f32], kfac: &mut [f32]) -> Option<Vec<f32>> {
    if !p.active() || zs.len() != z.len() || p.fluvial_damping <= 0.0 {
        return None;
    }
    let saved = kfac.to_vec();
    let damping = p.fluvial_damping as f32;
    for ((k, &a), &b) in kfac.iter_mut().zip(zs).zip(z) {
        if a.is_finite() && a > b {
            *k *= 1.0 - damping * ramp(p, a - b);
        }
    }
    Some(saved)
}

/// What one run of [`shape`] did over the window's core, m · cells.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShapeStats {
    pub carved_m_cells: f64,
    pub raised_m_cells: f64,
    pub glacier_cells: u64,
}

/// Six scratch arrays of a window's size (the amplifier lends its free buffers).
pub type Scratch<'a> = [&'a mut [f32]; 6];

/// Share of a margin that is lateral (a valley wall), not frontal (a terminus or a tidewater
/// front): |sin| of the angle between the gradient of the ice's depth over the smoothed bed
/// (`zs − zmean`: the valley's shape, not its gullies) and the ice-surface gradient (the flow).
/// 1 where either gradient vanishes.
#[inline]
fn lateral(win: &Win, zmean: &[f32], zs: &[f32], i: usize, j: usize) -> f32 {
    let w = win.w;
    let (ie, iw) = (col(win, i, 1), col(win, i, -1));
    let (js, jn) = (row(win, j, 1), row(win, j, -1));
    let dd = |c: usize| zs[c] - zmean[c];
    let gdx = dd(j * w + ie) - dd(j * w + iw);
    let gdy = dd(js * w + i) - dd(jn * w + i);
    let gzx = zs[j * w + ie] - zs[j * w + iw];
    let gzy = zs[js * w + i] - zs[jn * w + i];
    let nd = (gdx * gdx + gdy * gdy).sqrt();
    let nz = (gzx * gzx + gzy * gzy).sqrt();
    if nd < 1.0e-3 || nz < 1.0e-3 {
        return 1.0;
    }
    ((gdx * gzy - gdy * gzx) / (nd * nz)).abs().min(1.0)
}

/// The trough, cirque and ridge pass over a window (`z` in place; `zs` the ice surface; `scratch`
/// six arrays of the window's size whose contents are overwritten). Once the pass has looked at
/// the window, the last scratch array holds each cell's cut, m (0 where it cut nothing): the
/// amplifier hands it to the mountain character's fade (`tiles::run_tiles`).
pub fn shape(
    win: &Win,
    p: &GlacialShape,
    z: &mut [f32],
    zs: &[f32],
    core: (usize, usize, usize, usize),
    scratch: Scratch<'_>,
) -> ShapeStats {
    let mut st = ShapeStats::default();
    let n = win.cells();
    if !p.active() || zs.len() != n {
        return st;
    }
    let [d, tmp, dmax, zmax, zmean, cut] = scratch;
    let r = p.radius_cells as usize;
    depth(z, zs, d);
    if d.iter().all(|&x| x <= 0.0) {
        cut.iter_mut().for_each(|x| *x = 0.0);
        return st;
    }
    // The centre-line depth: the largest depth within reach, smoothed by a box mean of the same
    // radius (a raw maximum filter jumps where a deep cell enters its window, and those jumps
    // would print as streaks), never below the cell's own depth.
    filter(win, d, r, true, tmp, zmax);
    filter(win, zmax, r, false, tmp, dmax);
    for (m, &x) in dmax.iter_mut().zip(d.iter()) {
        *m = m.max(x);
    }
    // The highest ground within reach, smoothed the same way (`cut` holds the raw maximum until
    // the cut itself is made).
    filter(win, z, r, true, tmp, cut);
    filter(win, cut, r, false, tmp, zmax);
    filter(win, z, r, false, tmp, zmean);
    let s = p.strength as f32;
    // The cut under confined ice, and the ice-free ridges near it that take the rock back
    // (`free`, in `zmean`'s array once each cell has read its mean).
    // The lateral share of every cell first (it reads the neighbours' smoothed bed), kept in
    // `cut`'s array until each cell's cut replaces it.
    for c in 0..n {
        cut[c] = if d[c] > 0.0 {
            lateral(win, zmean, zs, c % win.w, c / win.w)
        } else {
            0.0
        };
    }
    let free = zmean;
    for c in 0..n {
        let dm = dmax[c];
        let mean_z = free[c];
        let lat = cut[c];
        cut[c] = 0.0;
        free[c] = 0.0;
        if dm <= 0.0 {
            continue;
        }
        let g = ramp(p, dm);
        if d[c] > 0.0 {
            // Confined: ridges within reach stand above this ice surface (an ice sheet that
            // drowns them scours its bed evenly instead).
            let confined = ((zmax[c] - zs[c]) / (0.25 * dm)).clamp(0.0, 1.0);
            if confined <= 0.0 {
                continue;
            }
            let t = (d[c] / dm).min(1.0);
            cut[c] = s * g * confined * lat * dm * t * (1.0 - t);
        } else {
            // Ice-free and standing above its surroundings: a crest or spur, not a slope or a
            // valley floor beyond a terminus (those lie at or below their neighbourhood's mean).
            free[c] = g * smoothstep(0.0, 200.0, f64::from(z[c] - mean_z)) as f32;
        }
    }
    // The depths are done with: their arrays take the box means.
    let (mean_cut, mean_free) = (d, dmax);
    let share = p.ridge_share as f32;
    if share > 0.0 {
        filter(win, cut, r, false, tmp, mean_cut);
        filter(win, free, r, false, tmp, mean_free);
    }
    let cap = p.max_ridge_raise_m as f32;
    let (x0, y0, cw, ch) = core;
    for c in 0..n {
        let mut dz = -cut[c];
        if share > 0.0 && free[c] > 0.0 {
            // Few ridge cells near a lot of carving do not get it all (no spikes): the share
            // of the neighbourhood that is ridge counts as at least a quarter.
            dz += (share * mean_cut[c] * free[c] / mean_free[c].max(0.25)).min(cap);
        }
        if dz != 0.0 {
            z[c] += dz;
            let (i, j) = (c % win.w, c / win.w);
            if i >= x0 && i < x0 + cw && j >= y0 && j < y0 + ch {
                if dz < 0.0 {
                    st.carved_m_cells += f64::from(-dz);
                    st.glacier_cells += 1;
                } else {
                    st.raised_m_cells += f64::from(dz);
                }
            }
        }
    }
    st
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A V-shaped valley along x, `w` cells across, filled with ice to 400 m above its floor.
    fn v_valley(w: usize, h: usize) -> (Win, Vec<f32>, Vec<f32>) {
        let win = Win::full(w, h);
        let mid = (h / 2) as f32;
        let mut z = vec![0.0f32; w * h];
        let mut zs = vec![0.0f32; w * h];
        for j in 0..h {
            for i in 0..w {
                let c = j * w + i;
                let x = (j as f32 - mid).abs();
                z[c] = 1000.0 + 100.0 * x - 2.0 * i as f32;
                zs[c] = 1400.0 - 2.0 * i as f32;
            }
        }
        (win, z, zs)
    }

    /// Runs [`shape`] with its own scratch.
    fn run(
        win: &Win,
        p: &GlacialShape,
        z: &mut [f32],
        zs: &[f32],
        core: (usize, usize, usize, usize),
    ) -> ShapeStats {
        let n = win.cells();
        let mut s: Vec<Vec<f32>> = (0..6).map(|_| vec![7.0f32; n]).collect();
        let [a, b, c, d, e, f] = &mut s[..] else {
            unreachable!()
        };
        shape(win, p, z, zs, core, [a, b, c, d, e, f])
    }

    /// Exponent b of y = a·|x|^b fitted to (|x|, depth above the floor) by least squares on logs.
    fn exponent(profile: &[(f32, f32)]) -> f64 {
        let pts: Vec<(f64, f64)> = profile
            .iter()
            .filter(|&&(x, y)| x > 0.0 && y > 1.0)
            .map(|&(x, y)| (f64::from(x).ln(), f64::from(y).ln()))
            .collect();
        let n = pts.len() as f64;
        let (sx, sy) = pts.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
        let (mx, my) = (sx / n, sy / n);
        let (mut num, mut den) = (0.0, 0.0);
        for &(x, y) in &pts {
            num += (x - mx) * (y - my);
            den += (x - mx) * (x - mx);
        }
        num / den
    }

    #[test]
    fn a_glaciated_v_valley_becomes_a_u_shaped_trough() {
        let (w, h) = (24, 21);
        let (win, mut z, zs) = v_valley(w, h);
        let before = z.clone();
        let p = GlacialShape {
            ridge_share: 0.0,
            radius_cells: 10,
            ..GlacialShape::default()
        };
        let st = run(&win, &p, &mut z, &zs, (0, 0, w, h));
        assert!(st.carved_m_cells > 0.0);
        // The cross-section at column 12 under the ice, depth above the floor against distance.
        let i = 12;
        let mid = h / 2;
        let floor = z[mid * w + i];
        assert_eq!(floor, before[mid * w + i], "the axis stays");
        let under = |zz: &[f32]| -> Vec<(f32, f32)> {
            (0..h)
                .filter(|&j| zz[j * w + i] < zs[j * w + i])
                .map(|j| ((j as f32 - mid as f32).abs(), zz[j * w + i] - floor))
                .collect()
        };
        let b0 = exponent(&under(&before));
        let b1 = exponent(&under(&z));
        assert!((b0 - 1.0).abs() < 0.05, "the V: b = {b0}");
        assert!(b1 > 1.6, "a U: y ∝ |x|^{b1}");
        // The walls are cut back most at mid-height, not at the margin or the axis.
        let cut = |j: usize| before[j * w + i] - z[j * w + i];
        assert!(cut(mid + 2) > cut(mid + 4) && cut(mid + 2) > 0.0 && cut(mid) == 0.0);
        // Above the ice nothing moves (no ridge share).
        for j in 0..h {
            if before[j * w + i] >= zs[j * w + i] {
                assert_eq!(z[j * w + i], before[j * w + i]);
            }
        }
    }

    #[test]
    fn the_ridges_get_the_rock_back_and_flat_ice_sheet_beds_stay() {
        // Parallel V valleys every 10 rows with sharp ridges between them standing 100 m above
        // the ice.
        let (w, h) = (24, 40);
        let win = Win::full(w, h);
        let mut z = vec![0.0f32; w * h];
        let mut zs = vec![0.0f32; w * h];
        for j in 0..h {
            for i in 0..w {
                let x = (j % 10) as f32 - 5.0;
                z[j * w + i] = 1400.0 - 80.0 * x.abs() - 2.0 * i as f32;
                zs[j * w + i] = 1300.0 - 2.0 * i as f32;
            }
        }
        let p = GlacialShape {
            ridge_share: 1.0,
            radius_cells: 10,
            max_ridge_raise_m: 1.0e6,
            ..GlacialShape::default()
        };
        let st = run(&win, &p, &mut z, &zs, (0, 0, w, h));
        // The crests get some of it back, never more than was carved.
        assert!(st.raised_m_cells > 0.1 * st.carved_m_cells, "{st:?}");
        assert!(st.raised_m_cells < 1.05 * st.carved_m_cells, "{st:?}");
        // A flat bed under 800 m of ice: t = 1 everywhere, nothing to shape.
        let win = Win::full(16, 16);
        let mut z = vec![100.0f32; 256];
        let zs = vec![900.0f32; 256];
        let st = run(&win, &p, &mut z, &zs, (0, 0, 16, 16));
        assert_eq!(st.carved_m_cells, 0.0);
        assert!(z.iter().all(|&x| x == 100.0));
    }

    #[test]
    fn a_window_gives_the_whole_rasters_bits() {
        // A wrapped planet of V valleys with ice of varying depth; a window with a halo of the
        // pass's radius gives the whole raster's values in its core.
        let (gw, gh) = (160, 96);
        let whole = Win::full(gw, gh);
        let mut z = vec![0.0f32; gw * gh];
        let mut zs = vec![f32::NAN; gw * gh];
        for j in 0..gh {
            for i in 0..gw {
                let c = j * gw + i;
                let x = ((i as f32 * 0.37).sin() * 6.0 + j as f32) % 9.0 - 4.5;
                z[c] = 500.0 + 80.0 * x.abs() + (i * 13 % 7) as f32;
                if (i / 8) % 3 != 0 {
                    zs[c] = 700.0 + (j as f32).sin() * 50.0;
                }
            }
        }
        let p = GlacialShape::default();
        let mut zw = z.clone();
        run(&whole, &p, &mut zw, &zs, (0, 0, gw, gh));
        let halo = p.radius();
        let (x0, y0, cw, ch) = (60usize, 30usize, 12usize, 8usize);
        let plan = super::super::tiles::TilePlan {
            gw,
            gh,
            tile: 16,
            halo,
        };
        let (win, cx, cy) = plan.window((x0, y0, cw, ch));
        assert!(!win.wraps());
        let pick = |v: &[f32]| -> Vec<f32> {
            (0..win.cells())
                .map(|k| {
                    let (gi, gj) = win.global(k % win.w, k / win.w);
                    v[gj * gw + gi]
                })
                .collect()
        };
        let mut zt = pick(&z);
        let zst = pick(&zs);
        run(&win, &p, &mut zt, &zst, (cx, cy, cw, ch));
        for j in 0..ch {
            for i in 0..cw {
                let a = zt[(j + cy) * win.w + i + cx];
                let b = zw[(y0 + j) * gw + x0 + i];
                assert_eq!(a.to_bits(), b.to_bits(), "({i},{j})");
            }
        }
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! Carrying the surface with the plates (plan §8.2: deep time on L1, coupled to the plate step).
//!
//! L1 is fixed on the planet while the crust moves over it, so each step the surface's own state
//! — the offset of its elevation from the plate model's, the regolith and the strata log — moves
//! with the plate under each cell: semi-Lagrangian advection. A cell on plate p after the step
//! takes what was, before the step, at its position turned back by p's rotation over the step,
//! R(−Ω_p·Δt)·q (the same finite rotation the plate model applied, `plates::kinematics`).
//! Continuous fields interpolate bilinearly there; the strata log and categorical fields take the
//! nearest cell (a column is copied whole, so the log keeps its invariants).
//!
//! The offset is small next to the elevation (valleys, basins, eroded orogens: what the plate
//! model's own elevation does not have), so the bilinear smoothing of repeated advection blurs
//! detail that erosion re-cuts every step rather than whole mountain ranges; the large-scale
//! elevation always comes from the plate model (`mod.rs`).

use glam::DVec3;
use rayon::prelude::*;

use deeptime_format::GridDesc;

use crate::plates::kinematics::rotation;
use crate::sphere::BlockFrame;

/// Where every cell's material was before the step: fractional (i, j) in the grid.
pub fn sources(grid: &GridDesc, plate_id: &[f32], omega: &[[f64; 3]], dt_yr: f64) -> Vec<[f64; 2]> {
    let frame = BlockFrame::new(grid.circumference() as u32);
    let w = grid.w as usize;
    let r = f64::from(grid.r);
    let (ox, oz) = (grid.origin_block_x as f64, grid.origin_block_z as f64);
    (0..grid.cells())
        .into_par_iter()
        .map(|c| {
            let (i, j) = ((c % w) as i64, (c / w) as i64);
            let p = plate_id[c];
            let om = if p.is_finite() && p >= 0.0 {
                omega.get(p as usize).copied().unwrap_or([0.0; 3])
            } else {
                [0.0; 3]
            };
            if om == [0.0; 3] {
                return [i as f64, j as f64];
            }
            let q = frame.unit(grid.block_x(i) as f64, grid.block_z(j) as f64);
            let back = rotation(DVec3::from_array(om), -dt_yr) * q;
            let (x, z) = frame.block(back);
            // x in [−C/2, C/2) maps to i = (x − origin_x) / r, wrapped by the samplers.
            [(x - ox) / r, (z - oz) / r]
        })
        .collect()
}

/// Bilinear sample of `v` at fractional (fi, fj): x wraps, rows clamp.
pub fn bilinear(w: usize, h: usize, v: &[f32], fi: f64, fj: f64) -> f32 {
    let fj = fj.clamp(0.0, (h - 1) as f64);
    let i0 = fi.floor();
    let j0 = fj.floor();
    let (ti, tj) = (fi - i0, fj - j0);
    let wi = |i: f64| (i as i64).rem_euclid(w as i64) as usize;
    let j0u = j0 as usize;
    let j1u = (j0u + 1).min(h - 1);
    let (a, b) = (wi(i0), wi(i0 + 1.0));
    let v00 = f64::from(v[j0u * w + a]);
    let v10 = f64::from(v[j0u * w + b]);
    let v01 = f64::from(v[j1u * w + a]);
    let v11 = f64::from(v[j1u * w + b]);
    ((v00 * (1.0 - ti) + v10 * ti) * (1.0 - tj) + (v01 * (1.0 - ti) + v11 * ti) * tj) as f32
}

/// [`bilinear`] over the corners that hold the same crust only: `same(cell)` says whether a
/// corner cell held, before the step, the crust that is now at the destination. The weights of
/// the corners that did are renormalised; `None` when none did (the crust is new at this place:
/// another plate's edge arrived, and what was here before belongs to the plate it overrode).
///
/// Without this the offset of one plate's surface is carried onto another plate's crust wherever
/// a boundary crosses a cell: a trench's sediment fill (plate elevation −6 km, offset +4 km)
/// lands on the overriding margin, and a range's offset on the next range. The 20 km "mountains"
/// of 2026-10-01 were built that way, a few km a step.
pub fn bilinear_same(
    w: usize,
    h: usize,
    v: &[f32],
    fi: f64,
    fj: f64,
    same: impl Fn(usize) -> bool,
) -> Option<f32> {
    let fj = fj.clamp(0.0, (h - 1) as f64);
    let i0 = fi.floor();
    let j0 = fj.floor();
    let (ti, tj) = (fi - i0, fj - j0);
    let wi = |i: f64| (i as i64).rem_euclid(w as i64) as usize;
    let j0u = j0 as usize;
    let j1u = (j0u + 1).min(h - 1);
    let (a, b) = (wi(i0), wi(i0 + 1.0));
    let corners = [
        (j0u * w + a, (1.0 - ti) * (1.0 - tj)),
        (j0u * w + b, ti * (1.0 - tj)),
        (j1u * w + a, (1.0 - ti) * tj),
        (j1u * w + b, ti * tj),
    ];
    let (mut sum, mut weight) = (0.0f64, 0.0f64);
    for (c, wt) in corners {
        if wt > 0.0 && same(c) {
            sum += wt * f64::from(v[c]);
            weight += wt;
        }
    }
    (weight > 1e-9).then(|| (sum / weight) as f32)
}

/// The nearest cell to fractional (fi, fj).
pub fn nearest(w: usize, h: usize, fi: f64, fj: f64) -> usize {
    let i = (fi.round() as i64).rem_euclid(w as i64) as usize;
    let j = (fj.round().clamp(0.0, (h - 1) as f64)) as usize;
    j * w + i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rotation_about_the_pole_shifts_columns() {
        // Ω along +Z (the pole): a turn by one cell's longitude per step moves material east.
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let w = grid.w as usize;
        let dlon = std::f64::consts::TAU / w as f64;
        let dt = 1.0e6;
        let omega = vec![[0.0, 0.0, dlon / dt]];
        let ids = vec![0.0f32; grid.cells()];
        let src = sources(&grid, &ids, &omega, dt);
        let c = 10 * w + 20;
        let [fi, fj] = src[c];
        assert!((fi - 19.0).abs() < 1e-6, "{fi}");
        assert!((fj - 10.0).abs() < 1e-6, "{fj}");
        let v: Vec<f32> = (0..grid.cells()).map(|k| (k % w) as f32).collect();
        assert!((bilinear(w, grid.h as usize, &v, 19.5, 10.0) - 19.5).abs() < 1e-5);
        assert_eq!(nearest(w, grid.h as usize, -0.4, 3.0), 3 * w);
        // Only the corners of the same crust count; none of them, no value.
        let h = grid.h as usize;
        assert_eq!(bilinear_same(w, h, &v, 19.5, 10.0, |_| true), Some(19.5));
        assert_eq!(bilinear_same(w, h, &v, 19.5, 10.0, |c| c % w == 19), Some(19.0));
        assert_eq!(bilinear_same(w, h, &v, 19.5, 10.0, |c| c % w == 20), Some(20.0));
        assert_eq!(bilinear_same(w, h, &v, 19.5, 10.0, |_| false), None);
        // A corner with no weight does not make the crust "the same".
        assert_eq!(bilinear_same(w, h, &v, 19.0, 10.0, |c| c % w == 20), None);
    }
}

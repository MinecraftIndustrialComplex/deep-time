// SPDX-License-Identifier: LGPL-3.0-or-later
//! Carrying the surface with the plates (plan §8.2: deep time on L1, coupled to the plate step).
//!
//! L1 is fixed on the planet while the crust moves over it, so each step the surface's own state
//! — its elevation, the regolith and the strata log — moves with the plate under each cell:
//! semi-Lagrangian advection. A cell on plate p after the step takes what was, before the step,
//! at its position turned back by p's rotation over the step, R(−Ω_p·Δt)·q (the same finite
//! rotation the plate model applied, `plates::kinematics`). Continuous fields interpolate
//! bilinearly there; the strata log and categorical fields take the nearest cell (a column is
//! copied whole, so the log keeps its invariants).
//!
//! The elevation moves with its own crust ([`carried_elevation`]): only source cells that held the
//! same crust count, the plate model's uplift of that crust is added, and where the crust is new
//! at a place the plate model's elevation is taken. (Until 2026-10-01 the state was the offset
//! from the plate model's elevation, advected whatever crust it came from and added to the plate
//! model's elevation under the cell. That elevation is sampled anew every step and can change by
//! kilometres under a cell with no uplift at all, when the cell reads the high side of a margin
//! instead of the low side; the offset built against the low side was added on top, 5 to 8 km in
//! a step.) The bilinear smoothing of repeated advection blurs detail that erosion re-cuts every
//! step.

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

/// What [`carried_elevation`] reads.
pub struct Carry<'a> {
    pub w: usize,
    pub h: usize,
    /// [`sources`] of the step.
    pub src: &'a [[f64; 2]],
    /// The surface's elevation after the last step, m.
    pub prev_elevation: &'a [f32],
    /// The plate and rigid group under each cell after the last step (`surface/crust_plate`,
    /// `surface/crust_group`); without the plates every source counts as the same crust.
    pub prev_plate: Option<&'a [f32]>,
    pub prev_group: Option<&'a [f32]>,
    /// `plates/plate_id`, `plates/group_id`, `plates/elevation_m`, `plates/uplift_mm_yr` now.
    pub plate: &'a [f32],
    pub group: Option<&'a [f32]>,
    pub plate_elevation: &'a [f32],
    pub uplift_mm_yr: &'a [f32],
    /// Each plate's parent, and how many plates existed before the step (a plate past that count
    /// was born in it, and its crust was its parent's).
    pub parents: &'a [Option<u32>],
    pub known_plates: usize,
    pub dt_yr: f64,
}

/// The surface's elevation at the start of a step, m, and the number of cells where another
/// plate's crust arrived: the last elevation of the same crust at its source (the corners that
/// held the same rigid group, the same plate, or the parent of a plate born this step), plus the
/// plate model's uplift of that crust over the step; where no corner held it (another plate's
/// edge arrived over a trench, a small plate was absorbed), the plate model's elevation: what the
/// surface had built there belongs to the crust that left. A group keeps its number when a
/// collision hands it to another plate, so a sutured terrane keeps its surface.
pub fn carried_elevation(x: &Carry<'_>) -> (Vec<f32>, u64) {
    let same = |c: usize, s: usize| -> bool {
        let Some(pp) = x.prev_plate else {
            return true;
        };
        let now = x.plate[c];
        if pp[s] == now {
            return true;
        }
        if let (Some(pg), Some(g)) = (x.prev_group, x.group)
            && pg[s] == g[c]
        {
            return true;
        }
        if !(now.is_finite() && now >= 0.0) {
            return false;
        }
        let mut k = now as usize;
        for _ in 0..64 {
            if k < x.known_plates {
                break;
            }
            match x.parents.get(k).copied().flatten() {
                Some(pk) => {
                    if pp[s] == pk as f32 {
                        return true;
                    }
                    k = pk as usize;
                }
                None => break,
            }
        }
        false
    };
    let lift = x.dt_yr * 1.0e-3;
    let foreign = std::sync::atomic::AtomicU64::new(0);
    let h = x
        .src
        .par_iter()
        .enumerate()
        .map(|(c, &[fi, fj])| {
            match bilinear_same(x.w, x.h, x.prev_elevation, fi, fj, |s| same(c, s)) {
                Some(h) => (f64::from(h) + f64::from(x.uplift_mm_yr[c]) * lift) as f32,
                None => {
                    foreign.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    x.plate_elevation[c]
                }
            }
        })
        .collect();
    (h, foreign.into_inner())
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
        assert_eq!(
            bilinear_same(w, h, &v, 19.5, 10.0, |c| c % w == 19),
            Some(19.0)
        );
        assert_eq!(
            bilinear_same(w, h, &v, 19.5, 10.0, |c| c % w == 20),
            Some(20.0)
        );
        assert_eq!(bilinear_same(w, h, &v, 19.5, 10.0, |_| false), None);
        // A corner with no weight does not make the crust "the same".
        assert_eq!(bilinear_same(w, h, &v, 19.0, 10.0, |c| c % w == 20), None);
    }

    /// The 2026-10-01 defect in small: plate 0 (low, a trench with 4 km of fill on it) west of
    /// column 8, plate 1 (a 9 km margin) from it east; nothing moves and nothing is uplifted, but
    /// the boundary steps one column west, so column 7 now reads plate 1's crust. The surface
    /// there is plate 1's elevation, not plate 1's elevation plus plate 0's fill; crust that
    /// stayed keeps its surface and takes the uplift; a sutured group keeps its surface under
    /// its new plate; a plate born this step carries its parent's.
    #[test]
    fn the_surface_moves_with_its_own_crust() {
        let (w, h) = (16usize, 4usize);
        let n = w * h;
        let src: Vec<[f64; 2]> = (0..n).map(|c| [(c % w) as f64, (c / w) as f64]).collect();
        let col = |c: usize| c % w;
        let prev_plate: Vec<f32> = (0..n).map(|c| if col(c) < 8 { 0.0 } else { 1.0 }).collect();
        let prev_group = prev_plate.clone();
        // Before: plate 0 at −6 km with 4 km of fill (surface −2 km), plate 1 at 9 km.
        let prev_elevation: Vec<f32> = (0..n)
            .map(|c| if col(c) < 8 { -2000.0 } else { 9000.0 })
            .collect();
        let plate: Vec<f32> = (0..n).map(|c| if col(c) < 7 { 0.0 } else { 1.0 }).collect();
        let plate_elevation: Vec<f32> = (0..n)
            .map(|c| if col(c) < 7 { -6000.0 } else { 9000.0 })
            .collect();
        let mut uplift = vec![0.0f32; n];
        uplift[3] = 0.5; // 0.5 mm/yr over 2 Myr: 1 km
        let carry = |plate: &[f32], group: &[f32], parents: &[Option<u32>]| {
            carried_elevation(&Carry {
                w,
                h,
                src: &src,
                prev_elevation: &prev_elevation,
                prev_plate: Some(&prev_plate),
                prev_group: Some(&prev_group),
                plate,
                group: Some(group),
                plate_elevation: &plate_elevation,
                uplift_mm_yr: &uplift,
                parents,
                known_plates: 2,
                dt_yr: 2.0e6,
            })
        };
        let (z, foreign) = carry(&plate, &plate, &[None, None]);
        assert_eq!(foreign, h as u64, "column 7 of every row");
        assert_eq!(
            z[7], 9000.0,
            "the margin's own elevation, not 9 km + 4 km of fill"
        );
        assert_eq!(z[6], -2000.0);
        assert_eq!(z[8], 9000.0);
        assert_eq!(z[3], -1000.0, "the same crust, lifted 1 km");
        // A suture: columns 4..8 keep their group (0) and now belong to plate 1.
        let sutured: Vec<f32> = (0..n).map(|c| if col(c) < 4 { 0.0 } else { 1.0 }).collect();
        let (z, foreign) = carry(&sutured, &prev_group, &[None, None]);
        assert_eq!((foreign, z[5]), (0, -2000.0));
        // A rift: columns 0..4 are plate 2, born this step from plate 0.
        let rifted: Vec<f32> = (0..n)
            .map(|c| match col(c) {
                0..4 => 2.0,
                4..8 => 0.0,
                _ => 1.0,
            })
            .collect();
        let (z, foreign) = carry(&rifted, &rifted, &[None, None, Some(0)]);
        assert_eq!((foreign, z[1], z[3]), (0, -2000.0, -1000.0));
        // An unrelated plate 2 there instead: its crust is new at the place.
        let (z, foreign) = carry(&rifted, &rifted, &[None, None, None]);
        assert_eq!((foreign, z[1]), (4 * h as u64, -6000.0));
    }
}

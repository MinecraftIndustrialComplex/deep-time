// SPDX-License-Identifier: LGPL-3.0-or-later
//! The crust's spreading (`surface/spread.rs`: the strength height, D = D₀ · min(x, X)³) on the
//! cube grid: the same law per cell, solved by the cube's Jacobi diffusion on the cells above the
//! strength height and their neighbours.

use rayon::prelude::*;

use super::diffusion::{self, Workspace};
use super::ordered_sum;
use crate::sphere::cube::CubeGrid;
use crate::surface::spread::{Spread, diffusivity};

/// One step of the spreading over `dt_yr` on elevations `h` (m). The [`Spread`]'s `moved_m` is
/// Σ of the rises over cells of the mean area (× `CubeGrid::mean_area`, the volume moved).
pub fn step_in(
    ws: &mut Workspace,
    g: &CubeGrid,
    h: &mut [f64],
    sea: f64,
    strength_height_m: f64,
    d0_km2_myr: f64,
    dt_yr: f64,
) -> Spread {
    if d0_km2_myr.is_nan() || d0_km2_myr <= 0.0 {
        return Spread::default();
    }
    let kd: Vec<f64> = h
        .par_iter()
        .map(|&z| diffusivity(z - sea, strength_height_m, d0_km2_myr))
        .collect();
    let cells = kd.par_iter().filter(|&&k| k > 0.0).count() as u64;
    if cells == 0 {
        return Spread::default();
    }
    let before = crate::alloc::copied(h);
    diffusion::step_in(ws, g, h, &kd, dt_yr);
    let (a, mean) = (g.areas(), g.mean_area());
    let moved_m = ordered_sum(h.len(), |r| {
        r.map(|c| (h[c] - before[c]).max(0.0) * a[c] / mean).sum()
    });
    Spread { cells, moved_m }
}

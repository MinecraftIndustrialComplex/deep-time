// SPDX-License-Identifier: LGPL-3.0-or-later
//! The surface solvers on the cube grid (cube world plan, phase 4, S2). Opt-in: nothing on the
//! Lambert path calls this module.
//!
//! **Which solvers run as they are.** The routing (`routing.rs`), both floods (`pflood.rs`, its
//! `parallel` module), the stream-power solve (`spl.rs`), the sea level (`sea_level.rs`,
//! [`crate::sea_level::fill_level_on`]) and the mouth fill (`marine::fill`) take any
//! [`SurfaceGrid`](super::grid::SurfaceGrid), and [`CubeGrid`] is one: its neighbour function
//! gives each cell's eight neighbours across faces, edges and the 24 corner cells' seven, with the
//! great-circle distances, and its cell areas are the true spherical areas. On the Lambert raster
//! (`Dims`) those solvers are what they were, bit for bit (the cell area is the one `dx²`, the
//! strips are the rows).
//!
//! **Which solvers are new here.** The implicit diffusion, its multigrid, the crust's spreading
//! and the submarine slope failure were written on the raster's index arithmetic (a five-point
//! stencil, rows and columns, one cell size), so the cube has its own:
//! - [`diffusion`]: backward Euler with the cube's coupling Laplacian (linear finite elements,
//!   `sphere/cube.rs`), the locally one-dimensional first guess per face, conjugate gradients
//!   with the Jacobi preconditioner or a multigrid V-cycle ([`multigrid`]), and the residual's
//!   volume put back, so Σ a·h is kept to round-off.
//! - [`spread`]: the strength-height spreading (`spread.rs`'s law, the cube's solve).
//! - [`failure`]: the submarine slope failure (`marine.rs`'s rule, pair volumes on true areas).
//!
//! Every sum in these solvers is taken over fixed chunks of cells in ascending order, and the
//! chunk sums then added in order, so results do not depend on the thread count. Cells are
//! stored face by face (F0, F1, F2, F3, N, S), so a chunk never mixes the order of the faces.

pub mod diffusion;
pub mod failure;
mod multigrid;
pub mod spread;

#[cfg(test)]
mod tests;

pub use crate::sphere::cube::CubeGrid;
pub use diffusion::{Workspace, solve};

use rayon::prelude::*;
use std::ops::Range;

/// Cells per task and per partial sum. It sets the grain of the work and the order of every sum;
/// no result depends on the thread count.
pub(crate) const CHUNK: usize = 4096;

/// Σ f(chunk) over the fixed chunks of `0..len`, the chunks in parallel and their sums added in
/// ascending order.
pub(crate) fn ordered_sum(len: usize, f: impl Fn(Range<usize>) -> f64 + Sync) -> f64 {
    let parts: Vec<f64> = (0..len.div_ceil(CHUNK))
        .into_par_iter()
        .map(|k| f(k * CHUNK..((k + 1) * CHUNK).min(len)))
        .collect();
    parts.iter().sum()
}

/// Σ a_c·v_c over the grid, m³ when `v` is in metres (the planet's volume of a layer).
pub fn volume(g: &CubeGrid, v: &[f64]) -> f64 {
    let a = g.areas();
    ordered_sum(v.len(), |r| r.map(|c| a[c] * v[c]).sum())
}

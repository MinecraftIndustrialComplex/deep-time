// SPDX-License-Identifier: LGPL-3.0-or-later
//! Marine sediment: delivery at the river mouths and depth-dependent diffusion.
//!
//! # Diffusivity (Landlab `SimpleSubmarineDiffuser`, MIT)
//! Ported from Landlab's `SimpleSubmarineDiffuser` (https://github.com/landlab/landlab,
//! `src/landlab/components/marine_sediment_transport/simple_submarine_diffuser.py`, MIT; the
//! equations, not the code): D(z) = D₀·f₁·f₂ with water depth z = sea level − h,
//! f₁ = (tanh(z / tidal range) + 1) / 2 (a smooth shoreline; a step without tidal range) and
//! f₂ = exp(−(z − wave base) / wave base) below the wave base (1 above it). Land takes the
//! hillslope diffusivity instead of Landlab's tiny value, since `diffusion.rs` solves land and sea
//! together. Landlab's defaults are D₀ = 100 m²/yr, wave base 60 m, tidal range 2 m.
//!
//! # Delivery at the mouths (Deep Time's rule)
//! Rivers deliver their load (`spl.rs`) to the sea cell they drain into. Dumping it there would
//! make towers on steep (active) margins, where the diffusivity below the wave base is nil, so each
//! mouth first **fills accommodation** outwards: a priority queue over sea cells from the mouth,
//! shallowest first, raises each cell to a clinoform fill level — `fill_depth_m` below sea level
//! at the mouth, deepening by `shelf_slope` with distance — until the load is spent. Deltas and
//! shelves prograde from the mouths this way; the diffusion then smooths them. Mass is exact: the
//! last cell a mouth reaches takes whatever is left.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::grid::Dims;

/// Landlab's `SimpleSubmarineDiffuser` diffusivity at water depth `z` (m, positive below sea
/// level), m²/yr; `land` is returned on land.
pub fn submarine_kd(z: f64, d0: f64, wave_base: f64, tidal_range: f64, land: f64) -> f64 {
    let f1 = if tidal_range > 0.0 {
        ((z / tidal_range).tanh() + 1.0) / 2.0
    } else if z >= 0.0 {
        1.0
    } else {
        0.0
    };
    let f2 = if z > wave_base && wave_base > 0.0 {
        (-(z - wave_base) / wave_base).exp()
    } else {
        1.0
    };
    // Across the shoreline blend towards the land value as f₁ falls.
    d0 * f1 * f2 + land * (1.0 - f1)
}

#[derive(Clone, Copy, PartialEq)]
struct Item {
    z: f64,
    c: u32,
    dist: f32,
}

impl Eq for Item {}

impl Ord for Item {
    fn cmp(&self, o: &Self) -> Ordering {
        o.z.total_cmp(&self.z).then_with(|| o.c.cmp(&self.c))
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// Scratch for [`fill`], reused between mouths and steps.
#[derive(Default)]
pub struct FillScratch {
    stamp: Vec<u32>,
    generation: u32,
    heap: BinaryHeap<Item>,
    /// The cells the last [`fill`] raised.
    pub touched: Vec<u32>,
}

/// Parameters of the mouth fill.
#[derive(Clone, Copy, Debug)]
pub struct FillParams {
    pub sea_level: f64,
    pub fill_depth_m: f64,
    pub shelf_slope: f64,
    /// Cells a mouth may visit before it dumps the rest where it stands.
    pub max_cells: usize,
}

/// Spreads `volume` m³ from the mouth `mouth` over sea cells (`sea[c]`), raising `h` and adding
/// the thickness to `dep`. Returns the number of cells that received sediment.
#[allow(clippy::too_many_arguments)]
pub fn fill(
    d: &Dims,
    sea: &[bool],
    mouth: usize,
    volume: f64,
    p: &FillParams,
    h: &mut [f64],
    dep: &mut [f32],
    s: &mut FillScratch,
) -> usize {
    let a = d.cell_area();
    if s.stamp.len() != d.cells() {
        s.stamp = vec![0; d.cells()];
        s.generation = 0;
    }
    s.generation = s.generation.wrapping_add(1);
    if s.generation == 0 {
        s.stamp.iter_mut().for_each(|x| *x = 0);
        s.generation = 1;
    }
    let stamp_id = s.generation;
    s.heap.clear();
    s.touched.clear();
    s.heap.push(Item {
        z: h[mouth],
        c: mouth as u32,
        dist: 0.0,
    });
    s.stamp[mouth] = stamp_id;
    let mut left = volume;
    let mut touched = 0;
    let mut visited = 0;
    let mut last = mouth;
    while left > 0.0 {
        let Some(it) = s.heap.pop() else { break };
        let c = it.c as usize;
        last = c;
        visited += 1;
        let level = p.sea_level - p.fill_depth_m - p.shelf_slope * f64::from(it.dist);
        if h[c] < level {
            let take = ((level - h[c]) * a).min(left);
            let dz = take / a;
            h[c] += dz;
            dep[c] += dz as f32;
            left -= take;
            touched += 1;
            s.touched.push(c as u32);
        }
        if left <= 0.0 || visited >= p.max_cells {
            break;
        }
        for (m, dl) in d.neighbours(c) {
            if sea[m] && s.stamp[m] != stamp_id {
                s.stamp[m] = stamp_id;
                s.heap.push(Item {
                    z: h[m],
                    c: m as u32,
                    dist: it.dist + dl as f32,
                });
            }
        }
    }
    if left > 0.0 {
        let dz = left / a;
        h[last] += dz;
        dep[last] += dz as f32;
        touched += 1;
        s.touched.push(last as u32);
    }
    touched
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kd_falls_below_the_wave_base_and_on_land() {
        let deep = submarine_kd(1000.0, 100.0, 60.0, 2.0, 1.0);
        let shallow = submarine_kd(30.0, 100.0, 60.0, 2.0, 1.0);
        let land = submarine_kd(-50.0, 100.0, 60.0, 2.0, 1.0);
        assert!((shallow - 100.0).abs() < 1e-6);
        assert!(deep < 1e-3 && deep > 0.0);
        assert!((land - 1.0).abs() < 1e-6);
    }

    #[test]
    fn a_mouth_fill_spends_exactly_its_volume() {
        let d = Dims::new(20, 20, 1000.0, true);
        let mut h = vec![-500.0; 400];
        let sea = vec![true; 400];
        let mut dep = vec![0.0f32; 400];
        let p = FillParams {
            sea_level: 0.0,
            fill_depth_m: 20.0,
            shelf_slope: 0.001,
            max_cells: 10_000,
        };
        let before: f64 = h.iter().sum();
        let v = 3.3e9;
        let mut s = FillScratch::default();
        let n = fill(&d, &sea, 210, v, &p, &mut h, &mut dep, &mut s);
        let after: f64 = h.iter().sum();
        assert!(n >= 7);
        assert!(
            ((after - before) * 1.0e6 - v).abs() < 1.0,
            "{} vs {v}",
            (after - before) * 1e6
        );
        assert!(h[210] <= -20.0 + 1e-9);
    }
}

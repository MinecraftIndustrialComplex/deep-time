// SPDX-License-Identifier: LGPL-3.0-or-later
//! Round shapes on the cube: the cube's counterparts of the volcano stage's footprint code
//! (`volcanoes::for_cells`, `volcanoes::shape::cell_weight`, and `L1::render`, `centre_weight`
//! and `base`), in the cube's metric (plan §7.3: a cone's radius in metres on the sphere, not
//! block metric).
//!
//! On Lambert a volcano is a block position, and a cell's offset from it is the block offset
//! times `m_per_block_h` (the game's flat world), so a cone is round in the world. On the cube the
//! simulation's metric is the sphere's:
//! - a volcano is a unit vector (cube frame), its basal radius metres on the sphere;
//! - a cell's distance from it is the great-circle arc R·θ between the volcano and the cell's
//!   centre, and its offset is that distance along the bearing: the azimuthal equidistant frame
//!   at the volcano, east and south (Lambert's dx and dz, +z south);
//! - a cell's weight is the mean of the profile over 4 × 4 points of the cell's own square on the
//!   map ([`CubeGrid::point_in`]). The simulation's map is close to equal-area (its cells are
//!   0.91 to 1.01 of the mean), so the plain mean is close to the area mean; the weight follows
//!   the cell's true shape and size on a face, on a seam or at a corner (a cone's volume is the
//!   same within 0.5 % at a corner, `a_cone_has_the_same_volume_anywhere`);
//! - a cell's size is the square root of its own area ([`cell_m`]), where Lambert takes
//!   `r · m_per_block_h` for every cell.
//!
//! The shapes themselves (`volcanoes::shape`: the profiles, the depressions, the blurs) are
//! functions of ρ = r / R and need no change. The footprint is found by a walk over the
//! neighbours from the cell holding the centre, so it crosses seams and corners like any other
//! cell edge; its cells come out in index order (each face's rows in order, faces in order), as
//! Lambert's come out in row order.

use std::collections::{HashSet, VecDeque};

use deeptime_format::projection::local_frame;

use crate::sphere::cube::{CubeGrid, NO_CELL};

/// How far past the reach the walk looks, in mean cell sides: more than the longest step between
/// two neighbours' centres (1.1 sides along a face's edge, 1.6 across a skewed corner cell's long
/// diagonal), so every cell within the reach is reached through cells within reach + margin.
/// game-choice: a search bound (no physics), Claude 2026-10-22.
const WALK_MARGIN: f64 = 2.5;

#[inline]
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The great-circle angle between two unit vectors (from the chord: exact near 0, symmetric).
#[inline]
fn arc(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    2.0 * (0.5 * dot(d, d).sqrt()).min(1.0).asin()
}

/// The great-circle distance between two unit vectors on the grid's sphere, m.
#[inline]
pub fn distance_m(g: &CubeGrid, a: [f64; 3], b: [f64; 3]) -> f64 {
    g.radius_m() * arc(a, b)
}

/// The size of cell c, m: the square root of its area (the cube's cells differ in size, from 0.91
/// to 1.01 of the mean on the simulation's map).
#[inline]
pub fn cell_m(g: &CubeGrid, c: usize) -> f64 {
    g.areas()[c].sqrt()
}

/// The offset of `p` from `centre` in the azimuthal equidistant frame at `centre`: (east, south),
/// m, at the great-circle distance along the bearing. (0, 0) at the centre itself.
pub fn offset(g: &CubeGrid, centre: [f64; 3], p: [f64; 3]) -> (f64, f64) {
    let (east, north) = local_frame(centre);
    let (x, y) = (dot(p, east), dot(p, north));
    let h = x.hypot(y);
    if h == 0.0 {
        return (0.0, 0.0);
    }
    let d = distance_m(g, centre, p);
    (d * x / h, -d * y / h)
}

/// Every cell whose centre is within `reach_m` (great-circle) of `centre`: calls
/// `f(cell, east_m, south_m)` in cell-index order, the offsets of [`offset`].
pub fn for_cells(g: &CubeGrid, centre: [f64; 3], reach_m: f64, mut f: impl FnMut(usize, f64, f64)) {
    for c in cells_within(g, centre, reach_m) {
        let (e, s) = offset(g, centre, g.unit(c));
        f(c, e, s);
    }
}

/// The cells whose centres are within `reach_m` of `centre`, in index order.
pub fn cells_within(g: &CubeGrid, centre: [f64; 3], reach_m: f64) -> Vec<usize> {
    let t = g.topology();
    let c0 = g.cell_of(centre);
    let walk = reach_m.max(0.0) + WALK_MARGIN * g.mean_area().sqrt();
    let mut seen: HashSet<u32> = HashSet::from([c0 as u32]);
    let mut queue = VecDeque::from([c0]);
    let mut hits = Vec::new();
    while let Some(c) = queue.pop_front() {
        if distance_m(g, centre, g.unit(c)) <= reach_m {
            hits.push(c);
        }
        for &m in t.slots(c) {
            if m == NO_CELL || seen.contains(&m) {
                continue;
            }
            if distance_m(g, centre, g.unit(m as usize)) <= walk {
                seen.insert(m);
                queue.push_back(m as usize);
            }
        }
    }
    hits.sort_unstable();
    hits
}

/// The cell-averaged weight of a normalised radial function `prof` (of ρ = r / `radius_m`) over
/// cell c for a shape centred on `centre`: the mean of 4 × 4 points of the cell's square on the
/// map, each at its great-circle distance (`shape::cell_weight`'s rule on the cube).
pub fn cell_weight(
    g: &CubeGrid,
    c: usize,
    centre: [f64; 3],
    radius_m: f64,
    prof: &dyn Fn(f64) -> f64,
) -> f64 {
    let mut s = 0.0;
    for b in 0..4 {
        for a in 0..4 {
            let p = g.point_in(
                c,
                (f64::from(a) + 0.5) / 4.0 - 0.5,
                (f64::from(b) + 0.5) / 4.0 - 0.5,
            );
            s += prof(distance_m(g, centre, p) / radius_m);
        }
    }
    s / 16.0
}

/// The rendered weight of `prof` at every cell of its reach (`reach` basal radii, plus one mean
/// cell for the cells the foot crosses, as Lambert adds one cell): (cell, weight > 0) in index
/// order (`L1::render` on the cube).
pub fn render(
    g: &CubeGrid,
    centre: [f64; 3],
    radius_m: f64,
    reach: f64,
    prof: &dyn Fn(f64) -> f64,
) -> Vec<(usize, f64)> {
    let mut out = Vec::new();
    let reach_m = radius_m * reach + g.mean_area().sqrt();
    for c in cells_within(g, centre, reach_m) {
        let w = cell_weight(g, c, centre, radius_m, prof);
        if w > 0.0 {
            out.push((c, w));
        }
    }
    out
}

/// The volume of a rendered shape of height `height_m`: Σ weight · area · height, m³ (Lambert's
/// `weight · cell_m²`).
pub fn volume_m3(g: &CubeGrid, cells: &[(usize, f64)], height_m: f64) -> f64 {
    let a = g.areas();
    cells.iter().map(|&(c, w)| w * a[c]).sum::<f64>() * height_m
}

/// The value of a field at a point: the cell holding it. (Lambert reads the four cells round it
/// bilinearly; on the cube the surface and a shape's weight are both read at the holding cell, so
/// [`centre_weight`] and a relief measured with this agree.)
pub fn sample(g: &CubeGrid, v: &[f32], p: [f64; 3]) -> f64 {
    f64::from(v[g.cell_of(p)])
}

/// The rendered weight of `prof` where [`sample`] reads the surface at the centre
/// (`L1::centre_weight` on the cube).
pub fn centre_weight(
    g: &CubeGrid,
    centre: [f64; 3],
    radius_m: f64,
    prof: &dyn Fn(f64) -> f64,
) -> f64 {
    cell_weight(g, g.cell_of(centre), centre, radius_m, prof)
}

/// The ground round a foot (`L1::base` on the cube): the lower quartile of `level(cell)` over the
/// ring from 1.05 R to 1.05 R + 1.5 cells of the holding cell's size (ledger VOL-033's ring, as on
/// Lambert), or `level` at the holding cell when the ring holds no cell.
pub fn ring_base(
    g: &CubeGrid,
    centre: [f64; 3],
    radius_m: f64,
    level: impl Fn(usize) -> f64,
) -> f64 {
    let c0 = g.cell_of(centre);
    let (inner, outer) = (1.05 * radius_m, 1.05 * radius_m + 1.5 * cell_m(g, c0));
    let mut ring: Vec<f64> = cells_within(g, centre, outer)
        .into_iter()
        .filter(|&c| distance_m(g, centre, g.unit(c)) >= inner)
        .map(&level)
        .collect();
    if ring.is_empty() {
        return level(c0);
    }
    ring.sort_by(f64::total_cmp);
    ring[ring.len() / 4]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::volcanoes::shape::{Kind, Shape};

    const R: f64 = 6.371e6;

    fn norm(v: [f64; 3]) -> [f64; 3] {
        let l = dot(v, v).sqrt();
        [v[0] / l, v[1] / l, v[2] / l]
    }

    /// The walk finds exactly the cells a scan of every cell finds, on a face, a seam and a
    /// corner, and the offsets give the great-circle distance.
    #[test]
    fn the_footprint_is_every_cell_within_reach() {
        let g = CubeGrid::new(48, R);
        for p in [
            [1.0, 0.2, 0.1],
            [1.0, 1.0, 0.05],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 1.0],
        ] {
            let p = norm(p);
            for reach in [50.0e3, 400.0e3, 1500.0e3] {
                let walked = cells_within(&g, p, reach);
                let scanned: Vec<usize> = (0..g.topology().cells())
                    .filter(|&c| distance_m(&g, p, g.unit(c)) <= reach)
                    .collect();
                assert_eq!(walked, scanned, "{p:?} {reach}");
                for_cells(&g, p, reach, |c, e, s| {
                    let d = distance_m(&g, p, g.unit(c));
                    assert!((e.hypot(s) - d).abs() < 1e-6 * d.max(1.0));
                });
            }
        }
    }

    /// A shield rendered at a face's centre, on a seam and on a corner has the analytic volume
    /// (∫ prof 2πR² sin θ dθ) within 1 %, and the three agree within 0.5 % (0.45 % measured
    /// 2026-10-22: the corner's sub-cell points are not quite equal areas).
    #[test]
    fn a_cone_has_the_same_volume_anywhere() {
        let g = CubeGrid::new(96, R);
        let shape = Shape {
            kind: Kind::Shield,
            depression: None,
        };
        let prof = |r: f64| shape.body(r);
        let radius = 600.0e3;
        // ∫₀¹ prof(ρ) 2πρ dρ · R², on the sphere's cap (sin θ for θ).
        let steps = 20_000;
        let exact: f64 = (0..steps)
            .map(|k| {
                let rho = (k as f64 + 0.5) / steps as f64;
                let theta = rho * radius / R;
                prof(rho) * 2.0 * std::f64::consts::PI * R * theta.sin() * radius / R / steps as f64
                    * R
            })
            .sum();
        let mut vols = Vec::new();
        for p in [[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0]] {
            let p = norm(p);
            let cells = render(&g, p, radius, 1.0, &prof);
            vols.push(volume_m3(&g, &cells, 1.0));
            let w = centre_weight(&g, p, radius, &prof);
            assert!(w > 0.9 && w <= 1.0, "centre weight {w}");
        }
        let dev = vols
            .iter()
            .map(|v| (v / vols[0] - 1.0).abs())
            .fold(0.0, f64::max);
        let err = (vols[0] / exact - 1.0).abs();
        eprintln!(
            "cube footprint: shield volume (km³ per m) {:.1} {:.1} {:.1}, exact {:.1}; error {:.3} \
             %, deviation {:.3} %",
            vols[0] / 1e6,
            vols[1] / 1e6,
            vols[2] / 1e6,
            exact / 1e6,
            err * 100.0,
            dev * 100.0
        );
        assert!(err < 0.01 && dev < 0.005, "error {err}, deviation {dev}");
    }

    /// The ring's ground is the level round the foot, whatever lies inside it.
    #[test]
    fn the_ring_reads_the_ground_round_the_foot() {
        let g = CubeGrid::new(64, R);
        let p = norm([1.0, 1.0, 1.0]);
        let radius = 300.0e3;
        let level = |c: usize| {
            if distance_m(&g, p, g.unit(c)) < radius {
                2000.0
            } else {
                -100.0
            }
        };
        assert_eq!(ring_base(&g, p, radius, level), -100.0);
    }
}

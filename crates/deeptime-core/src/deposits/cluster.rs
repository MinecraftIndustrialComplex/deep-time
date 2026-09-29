// SPDX-License-Identifier: LGPL-3.0-or-later
//! Positions (plan §8.6 step 3): a Thomas cluster process (Neyman–Scott parents with Gaussian
//! offspring; Thomas 1949) inside a permissive tract, weighted by favourability × province
//! endowment, with the USGS same-deposit separation as a hard minimum.
//!
//! Given the tract's count N (from `count.rs`): about N / `per_camp` camp centres are drawn from
//! the tract's cells in proportion to their weight, kept `camp_spacing` apart where possible
//! (Robert et al. 2005: orogenic-gold camps every 30–50 km); each deposit, largest first, joins
//! a random camp at a Gaussian offset of `sigma`, and is accepted only on a tract cell and at
//! least `max(min_sep, r_i + r_j)` from every deposit of its type already placed (so bodies of
//! one type never overlap). A deposit that cannot be placed in 64 tries near its camp tries 64
//! times anywhere in the tract, then is dropped (the census counts drops).
//!
//! Distances are in blocks in block metric (plan §5.2: local features are isotropic in block
//! space), x wrapping at the seam.

use deeptime_format::GridDesc;

use super::random::Rng;

/// A placed deposit: block x (canonical span), block z, its L1 cell and its horizontal radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub z: f64,
    pub cell: u32,
    pub radius: f64,
}

/// Block (x, z) of an L1 cell's sample point.
pub fn cell_xz(grid: &GridDesc, cell: u32) -> (f64, f64) {
    let i = i64::from(cell % grid.w);
    let j = i64::from(cell / grid.w);
    (grid.block_x(i) as f64, grid.block_z(j) as f64)
}

/// The L1 cell nearest block (x, z) (x wraps, z clamps).
pub fn cell_of(grid: &GridDesc, x: f64, z: f64) -> u32 {
    let r = f64::from(grid.r);
    let i = ((x - grid.origin_block_x as f64) / r).round() as i64;
    let j = ((z - grid.origin_block_z as f64) / r).round() as i64;
    grid.clamp_j(j) * grid.w + grid.wrap_i(i)
}

/// x wrapped into the canonical span [−C/2, C/2).
pub fn wrap_x(c: f64, x: f64) -> f64 {
    (x + c / 2.0).rem_euclid(c) - c / 2.0
}

/// Block-metric distance with the seam wrap.
pub fn distance(c: f64, a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = wrap_x(c, a.0 - b.0);
    (dx * dx + (a.1 - b.1).powi(2)).sqrt()
}

/// What `place` needs about a tract.
pub struct TractSpace<'a> {
    pub grid: &'a GridDesc,
    /// The tract's cells and their weights (same order; weights ≥ 0).
    pub cells: &'a [u32],
    pub weights: &'a [f64],
    /// Whether a cell may host a deposit (in this tract, weight > 0).
    pub allowed: &'a dyn Fn(u32) -> bool,
}

/// Placement parameters (blocks).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spacing {
    pub camp_spacing: f64,
    pub sigma: f64,
    pub min_sep: f64,
    /// Mean deposits per camp.
    pub per_camp: f64,
}

/// A position anywhere in the tract, drawn ∝ weight and jittered within its cell.
fn from_cells(rng: &mut Rng, t: &TractSpace<'_>) -> Option<(f64, f64)> {
    let k = rng.weighted(t.weights)?;
    let (x, z) = cell_xz(t.grid, t.cells[k]);
    let r = f64::from(t.grid.r);
    let c = t.grid.circumference() as f64;
    Some((
        wrap_x(c, x + (rng.uniform() - 0.5) * r),
        z + (rng.uniform() - 0.5) * r,
    ))
}

/// Places deposits of the given radii (blocks, any order; placed largest first) in a tract.
/// `placed` holds this type's deposits so far (all tracts) and receives the new ones. Returns
/// the placed deposits with their index into `radii`, and how many could not be placed.
pub fn place(
    rng: &mut Rng,
    t: &TractSpace<'_>,
    s: &Spacing,
    radii: &[f64],
    placed: &mut Vec<Point>,
) -> (Vec<(usize, Point)>, usize) {
    let c = t.grid.circumference() as f64;
    let mut order: Vec<usize> = (0..radii.len()).collect();
    order.sort_by(|a, b| radii[*b].total_cmp(&radii[*a]));
    let n_camps = ((radii.len() as f64 / s.per_camp.max(1.0)).round() as usize).max(1);
    let mut camps: Vec<(f64, f64)> = Vec::with_capacity(n_camps);
    for _ in 0..n_camps {
        let mut best = None;
        for _ in 0..50 {
            let Some(p) = from_cells(rng, t) else { break };
            let ok = camps.iter().all(|q| distance(c, p, *q) >= s.camp_spacing);
            best = Some(p);
            if ok {
                break;
            }
        }
        if let Some(p) = best {
            camps.push(p);
        }
    }
    let fits = |p: (f64, f64), r: f64, placed: &[Point]| -> Option<u32> {
        let (x, z) = (wrap_x(c, p.0), p.1);
        let half = t.grid.circumference() as f64 / 4.0;
        if !(-half..half).contains(&z) {
            return None;
        }
        let cell = cell_of(t.grid, x, z);
        if !(t.allowed)(cell) {
            return None;
        }
        placed
            .iter()
            .all(|q| distance(c, (x, z), (q.x, q.z)) >= s.min_sep.max(r + q.radius))
            .then_some(cell)
    };
    let mut out = Vec::new();
    let mut dropped = 0;
    for k in order {
        let r = radii[k];
        let mut done = None;
        if !camps.is_empty() {
            let camp = camps[rng.below(camps.len() as u64) as usize];
            for _ in 0..64 {
                let p = (camp.0 + s.sigma * rng.normal(), camp.1 + s.sigma * rng.normal());
                if let Some(cell) = fits(p, r, placed) {
                    done = Some((p, cell));
                    break;
                }
            }
        }
        if done.is_none() {
            for _ in 0..64 {
                let Some(p) = from_cells(rng, t) else { break };
                if let Some(cell) = fits(p, r, placed) {
                    done = Some((p, cell));
                    break;
                }
            }
        }
        match done {
            Some((p, cell)) => {
                let pt = Point {
                    x: wrap_x(c, p.0),
                    z: p.1,
                    cell,
                    radius: r,
                };
                placed.push(pt);
                out.push((k, pt));
            }
            None => dropped += 1,
        }
    }
    (out, dropped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_round_trip_and_wrap() {
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        for cell in [0u32, 5, 64 * 3 + 63, 64 * 31] {
            let (x, z) = cell_xz(&grid, cell);
            assert_eq!(cell_of(&grid, x + 3.0, z - 7.0), cell);
        }
        assert_eq!(cell_of(&grid, 511.0, -512.0), 0, "x = C/2 − 1 rounds to the seam cell");
        assert!((distance(1024.0, (-510.0, 0.0), (510.0, 0.0)) - 4.0).abs() < 1e-9);
        assert_eq!(wrap_x(1024.0, 600.0), -424.0);
    }

    /// Acceptance: the minimum separation (and body non-overlap) holds, over many seeds.
    #[test]
    fn min_separation_holds() {
        let grid = GridDesc::for_planet("L1", 2048, 16).unwrap(); // 128 × 64
        // A tract crossing the seam: columns 120..128 and 0..8 of rows 20..40.
        let cells: Vec<u32> = (20..40u32)
            .flat_map(|j| (120..128u32).chain(0..8).map(move |i| j * 128 + i))
            .collect();
        let weights: Vec<f64> = cells.iter().map(|c| 0.2 + f64::from(c % 7) / 7.0).collect();
        let set: std::collections::BTreeSet<u32> = cells.iter().copied().collect();
        let allowed = |c: u32| set.contains(&c);
        let t = TractSpace {
            grid: &grid,
            cells: &cells,
            weights: &weights,
            allowed: &allowed,
        };
        let s = Spacing {
            camp_spacing: 60.0,
            sigma: 20.0,
            min_sep: 7.0,
            per_camp: 4.0,
        };
        let mut total = 0;
        for seed in 0..200 {
            let mut rng = Rng::new(seed);
            let radii: Vec<f64> = (0..25).map(|_| 2.0 + 10.0 * rng.uniform()).collect();
            let mut placed = Vec::new();
            let (out, dropped) = place(&mut rng, &t, &s, &radii, &mut placed);
            assert_eq!(out.len() + dropped, radii.len());
            total += out.len();
            let out: Vec<Point> = out
                .iter()
                .map(|(k, p)| {
                    assert_eq!(radii[*k], p.radius);
                    *p
                })
                .collect();
            for (a, p) in out.iter().enumerate() {
                assert!(set.contains(&p.cell));
                assert_eq!(cell_of(&grid, p.x, p.z), p.cell);
                for q in &out[a + 1..] {
                    let d = distance(2048.0, (p.x, p.z), (q.x, q.z));
                    assert!(d >= s.min_sep.max(p.radius + q.radius) - 1e-9, "seed {seed}: {d}");
                }
            }
        }
        assert!(total > 200 * 15, "most deposits fit: {total}");
    }
}

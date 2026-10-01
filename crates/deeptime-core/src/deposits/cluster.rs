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
//! `adjoining` (coal seam fields, 2026-09-29): real coal basins are filled by fields that touch, so
//! there are no camps and no minimum spacing; each deposit, largest first, is drawn anywhere in
//! the tract (∝ weight, 400 tries) and accepted where its plan ellipse overlaps none placed
//! before ([`ellipses_overlap`], exact for convex shapes); the yaw is drawn here and kept.
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
    /// The plan footprint of an `adjoining` deposit: semi-axes `[a, c]` (blocks) and the yaw
    /// (radians, as `Body::yaw`); none for a circle of `radius`.
    pub ellipse: Option<[f64; 3]>,
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
    /// `adjoining`: no camps, no `min_sep`; footprints may touch but not overlap.
    pub adjoining: bool,
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
///
/// With `s.adjoining` the deposits are not clustered: each, largest first, is drawn anywhere in the
/// tract (∝ weight) up to 400 times and accepted where its plan ellipse (`footprints[k]` =
/// semi-axes `[a, c]` in blocks, a random yaw) overlaps no ellipse already placed
/// ([`ellipses_overlap`]); `min_sep` and the camps are not used.
pub fn place(
    rng: &mut Rng,
    t: &TractSpace<'_>,
    s: &Spacing,
    radii: &[f64],
    footprints: Option<&[[f64; 2]]>,
    placed: &mut Vec<Point>,
    claims: &mut dyn FnMut(usize, &Point) -> bool,
) -> (Vec<(usize, Point)>, usize) {
    let c = t.grid.circumference() as f64;
    if s.adjoining {
        let footprints = footprints.expect("adjoining placement needs the footprints");
        return place_adjoining(rng, t, radii, footprints, placed, claims);
    }
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
                let p = (
                    camp.0 + s.sigma * rng.normal(),
                    camp.1 + s.sigma * rng.normal(),
                );
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
                    ellipse: None,
                };
                placed.push(pt);
                out.push((k, pt));
            }
            None => dropped += 1,
        }
    }
    (out, dropped)
}

/// Whether the plan ellipses of two deposits overlap: centres `pa`, `pb` (blocks, seam wrap),
/// semi-axes `[a, c]` and yaw as [`Point::ellipse`]. Two convex shapes are disjoint when some
/// direction u separates them, u·(pb − pa) > h_a(u) + h_b(u), h the support function
/// √((a·u·e_x)² + (c·u·e_z)²); 360 directions are tried, and a pair must clear by a margin of
/// 0.1 % of the smaller semi-axis, which covers the gap between two sampled directions, so
/// "not overlapping" is exact for the placed bodies.
pub fn ellipses_overlap(
    c: f64,
    pa: (f64, f64),
    ea: [f64; 3],
    pb: (f64, f64),
    eb: [f64; 3],
) -> bool {
    let dx = wrap_x(c, pb.0 - pa.0);
    let dz = pb.1 - pa.1;
    let dist = (dx * dx + dz * dz).sqrt();
    let (ra, rb) = (ea[0].max(ea[1]), eb[0].max(eb[1]));
    if dist >= ra + rb {
        return false;
    }
    let (ia, ib) = (ea[0].min(ea[1]), eb[0].min(eb[1]));
    if dist < ia + ib {
        return true;
    }
    let margin = 1.0e-3 * ia.min(ib);
    let (sa, ca) = ea[2].sin_cos();
    let (sb, cb) = eb[2].sin_cos();
    const N: usize = 360;
    for k in 0..N {
        let th = std::f64::consts::TAU * k as f64 / N as f64;
        let (uz, ux) = th.sin_cos();
        // Local axes of a body (yaw ψ): e_x = (cos ψ, sin ψ), e_z = (−sin ψ, cos ψ) in (x, z).
        let h = |e: &[f64; 3], s: f64, cc: f64| {
            let px = ux * cc + uz * s;
            let pz = -ux * s + uz * cc;
            ((e[0] * px).powi(2) + (e[1] * pz).powi(2)).sqrt()
        };
        if ux * dx + uz * dz > h(&ea, sa, ca) + h(&eb, sb, cb) + margin {
            return false;
        }
    }
    true
}

/// [`place`] for `adjoining` deposits: each field, largest first, anywhere in the tract (∝ the weights) where
/// its plan ellipse crosses none of the fields that claim space. `claims(k, point)` says whether
/// field k, placed at `point`, reserves its footprint — a field that will not survive (eroded,
/// held by no cover) frees its space for later ones (Ben, 2026-09-30), so surviving fields never
/// overlap while the eroded ones do not crowd them out.
fn place_adjoining(
    rng: &mut Rng,
    t: &TractSpace<'_>,
    radii: &[f64],
    footprints: &[[f64; 2]],
    placed: &mut Vec<Point>,
    claims: &mut dyn FnMut(usize, &Point) -> bool,
) -> (Vec<(usize, Point)>, usize) {
    let c = t.grid.circumference() as f64;
    let mut order: Vec<usize> = (0..radii.len()).collect();
    order.sort_by(|a, b| radii[*b].total_cmp(&radii[*a]));
    let mut out = Vec::new();
    let mut dropped = 0;
    for k in order {
        let yaw = rng.uniform() * std::f64::consts::PI;
        let e = [footprints[k][0], footprints[k][1], yaw];
        let mut done = None;
        for _ in 0..400 {
            let Some(p) = from_cells(rng, t) else { break };
            let (x, z) = (wrap_x(c, p.0), p.1);
            let half = c / 4.0;
            if !(-half..half).contains(&z) {
                continue;
            }
            let cell = cell_of(t.grid, x, z);
            if !(t.allowed)(cell) {
                continue;
            }
            let clear = placed.iter().all(|q| match q.ellipse {
                Some(qe) => !ellipses_overlap(c, (x, z), e, (q.x, q.z), qe),
                None => distance(c, (x, z), (q.x, q.z)) >= radii[k] + q.radius,
            });
            if clear {
                done = Some((x, z, cell));
                break;
            }
        }
        match done {
            Some((x, z, cell)) => {
                let pt = Point {
                    x,
                    z,
                    cell,
                    radius: radii[k],
                    ellipse: Some(e),
                };
                if claims(k, &pt) {
                    placed.push(pt);
                }
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
        assert_eq!(
            cell_of(&grid, 511.0, -512.0),
            0,
            "x = C/2 − 1 rounds to the seam cell"
        );
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
            adjoining: false,
        };
        let mut total = 0;
        for seed in 0..200 {
            let mut rng = Rng::new(seed);
            let radii: Vec<f64> = (0..25).map(|_| 2.0 + 10.0 * rng.uniform()).collect();
            let mut placed = Vec::new();
            let (out, dropped) = place(&mut rng, &t, &s, &radii, None, &mut placed, &mut |_, _| {
                true
            });
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
                    assert!(
                        d >= s.min_sep.max(p.radius + q.radius) - 1e-9,
                        "seed {seed}: {d}"
                    );
                }
            }
        }
        assert!(total > 200 * 15, "most deposits fit: {total}");
    }

    #[test]
    fn ellipse_overlap_matches_the_geometry() {
        let c = 1000.0;
        let circle = |r: f64| [r, r, 0.0];
        assert!(ellipses_overlap(
            c,
            (0.0, 0.0),
            circle(10.0),
            (19.0, 0.0),
            circle(10.0)
        ));
        assert!(!ellipses_overlap(
            c,
            (0.0, 0.0),
            circle(10.0),
            (20.05, 0.0),
            circle(10.0)
        ));
        // Long along x (yaw 0): side by side along x they touch at 2a, along z at 2c.
        let e = [10.0, 3.0, 0.0];
        assert!(ellipses_overlap(c, (0.0, 0.0), e, (19.0, 0.0), e));
        assert!(!ellipses_overlap(c, (0.0, 0.0), e, (20.05, 0.0), e));
        assert!(ellipses_overlap(c, (0.0, 0.0), e, (0.0, 5.9), e));
        assert!(!ellipses_overlap(c, (0.0, 0.0), e, (0.0, 6.05), e));
        // Turned a quarter, the long axis lies along z.
        let q = [10.0, 3.0, std::f64::consts::FRAC_PI_2];
        assert!(ellipses_overlap(c, (0.0, 0.0), q, (0.0, 19.0), q));
        assert!(!ellipses_overlap(c, (0.0, 0.0), q, (6.05, 0.0), q));
        // Across the seam: x = 499 and −499 are 2 apart.
        assert!(ellipses_overlap(c, (499.0, 0.0), e, (-499.0, 0.0), e));
        // A ring around a small one: no touching boundaries, still overlapping.
        assert!(ellipses_overlap(
            c,
            (0.0, 0.0),
            [10.0, 10.0, 0.0],
            (1.0, 1.0),
            [2.0, 1.0, 0.3]
        ));
    }

    /// Adjoining placement: many large fields in a tract, no minimum spacing and no camps; no two
    /// footprints overlap (checked by sampling each ellipse's boundary against the other), and the
    /// tract fills far beyond what a circle-and-spacing rule allows.
    #[test]
    fn adjoining_fields_fill_a_tract_without_overlap() {
        let grid = GridDesc::for_planet("L1", 2048, 16).unwrap(); // 128 × 64
        let cells: Vec<u32> = (10..50u32)
            .flat_map(|j| (100..128u32).chain(0..30).map(move |i| j * 128 + i))
            .collect();
        let weights: Vec<f64> = cells.iter().map(|c| 0.3 + f64::from(c % 5) / 5.0).collect();
        let set: std::collections::BTreeSet<u32> = cells.iter().copied().collect();
        let allowed = |c: u32| set.contains(&c);
        let t = TractSpace {
            grid: &grid,
            cells: &cells,
            weights: &weights,
            allowed: &allowed,
        };
        let s = Spacing {
            camp_spacing: 500.0,
            sigma: 500.0,
            min_sep: 1.0e6,
            per_camp: 4.0,
            adjoining: true,
        };
        let inside = |p: (f64, f64), e: &[f64; 3], q: (f64, f64)| {
            let (sy, cy) = e[2].sin_cos();
            let (dx, dz) = (wrap_x(2048.0, q.0 - p.0), q.1 - p.1);
            let (lx, lz) = (cy * dx + sy * dz, -sy * dx + cy * dz);
            (lx / e[0]).powi(2) + (lz / e[1]).powi(2) < 1.0 - 1e-6
        };
        let boundary = |p: (f64, f64), e: &[f64; 3]| {
            let (sy, cy) = e[2].sin_cos();
            (0..720)
                .map(|k| {
                    let th = std::f64::consts::TAU * f64::from(k) / 720.0;
                    let (lz, lx) = (e[1] * th.sin(), e[0] * th.cos());
                    (p.0 + cy * lx - sy * lz, p.1 + sy * lx + cy * lz)
                })
                .collect::<Vec<_>>()
        };
        let mut total = 0;
        let mut requested = 0;
        for seed in 0..40 {
            let mut rng = Rng::new(seed);
            let semi: Vec<[f64; 2]> = (0..30)
                .map(|_| {
                    let a = 20.0 + 20.0 * rng.uniform();
                    [a, a * (1.0 + 0.5 * rng.uniform())]
                })
                .collect();
            let radii: Vec<f64> = semi.iter().map(|e| e[0].max(e[1])).collect();
            let mut placed = Vec::new();
            let (out, dropped) = place(
                &mut rng,
                &t,
                &s,
                &radii,
                Some(&semi),
                &mut placed,
                &mut |_, _| true,
            );
            assert_eq!(out.len() + dropped, radii.len());
            total += out.len();
            requested += radii.len();
            let pts: Vec<Point> = out.iter().map(|(_, p)| *p).collect();
            for (a, p) in pts.iter().enumerate() {
                assert!(set.contains(&p.cell));
                let (pe, pp) = (p.ellipse.unwrap(), (p.x, p.z));
                for q in &pts[a + 1..] {
                    let (qe, qp) = (q.ellipse.unwrap(), (q.x, q.z));
                    assert!(
                        !boundary(pp, &pe).iter().any(|b| inside(qp, &qe, *b))
                            && !boundary(qp, &qe).iter().any(|b| inside(pp, &pe, *b)),
                        "seed {seed}: fields at {pp:?} and {qp:?} overlap"
                    );
                }
            }
        }
        // min_sep 1e6 would forbid every pair under the Thomas rule; adjoining ignores it.
        assert!(total * 10 > requested * 7, "{total} of {requested} placed");
    }

    /// A field that does not claim its footprint (it will not survive) leaves the space to later
    /// fields: in a tract that holds one field, every field is placed when none claims, one when
    /// all do.
    #[test]
    fn only_claiming_fields_reserve_space() {
        let grid = GridDesc::for_planet("L1", 2048, 16).unwrap();
        let cells: Vec<u32> = vec![20 * 128 + 10];
        let weights = vec![1.0];
        let allowed = |c: u32| c == cells[0];
        let t = TractSpace {
            grid: &grid,
            cells: &cells,
            weights: &weights,
            allowed: &allowed,
        };
        let s = Spacing {
            camp_spacing: 500.0,
            sigma: 500.0,
            min_sep: 0.0,
            per_camp: 4.0,
            adjoining: true,
        };
        // Fields as large as the cell: two of them cannot sit side by side in it.
        let semi = vec![[20.0, 20.0]; 6];
        let radii = vec![20.0; 6];
        let run = |claim: &dyn Fn(usize) -> bool| {
            let mut rng = Rng::new(7);
            let mut placed = Vec::new();
            let (out, dropped) = place(
                &mut rng,
                &t,
                &s,
                &radii,
                Some(&semi),
                &mut placed,
                &mut |k, _| claim(k),
            );
            (out, dropped, placed)
        };
        let (out, dropped, placed) = run(&|_| false);
        assert_eq!((out.len(), dropped, placed.len()), (6, 0, 0));
        let (out, dropped, placed) = run(&|_| true);
        assert_eq!((out.len(), dropped, placed.len()), (1, 5, 1));
        // (Equal sizes keep their order.) The first three will not survive and all land; the
        // fourth survives and claims the cell, so the last two find no room.
        let (out, dropped, placed) = run(&|k| k >= 3);
        assert_eq!((out.len(), dropped, placed.len()), (4, 2, 1));
        assert_eq!(
            out.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
            [0, 1, 2, 3]
        );
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The S1 checks (cube world plan §3): the neighbour table against the format's `Across::cell`,
//! the areas against 4πR², the ghost fill against a direct fold of the cube's faces, the Hilbert
//! keys, and the metric's couplings.

use deeptime_format::cube::{Cube, CubeFold, Face, FaceSide};

use super::*;

const R: f64 = 6_371_000.0;

/// The face frame as integer axis vectors.
fn frame_i(face: Face) -> [[i64; 3]; 3] {
    face.frame().map(|v| v.map(|x| x.round() as i64))
}

fn dot_i(a: [i64; 3], b: [i64; 3]) -> i64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn lin(terms: &[(i64, [i64; 3])]) -> [i64; 3] {
    let mut p = [0i64; 3];
    for &(s, v) in terms {
        for k in 0..3 {
            p[k] += s * v[k];
        }
    }
    p
}

/// The point (scaled by n) of face `face`'s cell (i, j), at most one side out, folded onto the
/// cube's surface across that side: the face's plane turned about the cube edge onto the next
/// face, from the frames alone (no adjacency table).
fn fold_point3d(n: i64, face: Face, i: i64, j: i64) -> [i64; 3] {
    let [c, eu, ev] = frame_i(face);
    let (u, v) = (2 * i + 1 - n, 2 * j + 1 - n);
    if u > n {
        lin(&[(2 * n - u, c), (n, eu), (v, ev)])
    } else if u < -n {
        lin(&[(2 * n + u, c), (-n, eu), (v, ev)])
    } else if v > n {
        lin(&[(2 * n - v, c), (u, eu), (n, ev)])
    } else if v < -n {
        lin(&[(2 * n + v, c), (u, eu), (-n, ev)])
    } else {
        lin(&[(n, c), (u, eu), (v, ev)])
    }
}

/// The face and cell of a scaled surface point.
fn cell_of_point3d(n: i64, p: [i64; 3]) -> (Face, i64, i64) {
    for f in Face::ALL {
        let [c, eu, ev] = frame_i(f);
        if dot_i(p, c) == n {
            let (u, v) = (dot_i(p, eu), dot_i(p, ev));
            return (f, (u + n - 1) / 2, (v + n - 1) / 2);
        }
    }
    panic!("{p:?} is not on the cube's surface");
}

fn fold3d(n: i64, face: Face, i: i64, j: i64) -> (Face, i64, i64) {
    cell_of_point3d(n, fold_point3d(n, face, i, j))
}

#[test]
fn neighbours_match_the_formats_cells() {
    for n in [2usize, 3, 5, 8, 16, 48] {
        let t = CubeTopology::new(n);
        let ni = n as i64;
        let mut sevens = 0;
        for c in 0..t.cells() {
            let (face, i, j) = t.coords(c);
            assert_eq!(t.index(face, i, j), c);
            let s = t.slots(c);
            for (k, &(di, dj, _)) in OFFSETS.iter().enumerate() {
                let (a, b) = (i as i64 + di, j as i64 + dj);
                let out_i = !(0..ni).contains(&a);
                let out_j = !(0..ni).contains(&b);
                let want = match (out_i, out_j) {
                    (false, false) => t.index(face, a as usize, b as usize) as u32,
                    (true, true) => NO_CELL,
                    _ => {
                        let side = if a >= ni {
                            FaceSide::East
                        } else if a < 0 {
                            FaceSide::West
                        } else if b >= ni {
                            FaceSide::South
                        } else {
                            FaceSide::North
                        };
                        let x = face.across(side);
                        let (a2, b2) = x.cell(ni, a, b);
                        assert!((0..ni).contains(&a2) && (0..ni).contains(&b2));
                        t.index(x.face, a2 as usize, b2 as usize) as u32
                    }
                };
                assert_eq!(s[k], want, "n {n} cell {c} slot {k}");
            }
            let live: Vec<u32> = s.iter().copied().filter(|&m| m != NO_CELL).collect();
            if live.len() == 7 {
                sevens += 1;
            } else {
                assert_eq!(live.len(), 8, "n {n} cell {c}");
            }
            let mut sorted = live.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(
                sorted.len(),
                live.len(),
                "n {n} cell {c}: a neighbour twice"
            );
            for &m in &live {
                assert_ne!(m as usize, c);
                assert!(
                    t.slot_of(m as usize, c).is_some(),
                    "n {n}: {c} -> {m} not mutual"
                );
            }
        }
        assert_eq!(sevens, 24, "n {n}");
    }
}

#[test]
fn areas_sum_to_the_sphere() {
    let sphere = 4.0 * std::f64::consts::PI * R * R;
    for n in [16usize, 64, 240] {
        let g = CubeGrid::new(n, R);
        let rel = (g.total_area() - sphere).abs() / sphere;
        let (lo, hi) = g
            .areas()
            .iter()
            .fold((f64::INFINITY, 0.0f64), |(lo, hi), &a| {
                (lo.min(a), hi.max(a))
            });
        eprintln!(
            "cube n {n}: area sum rel error {rel:.3e}, cell area min/mean {:.6} max/mean {:.6}",
            lo / g.mean_area(),
            hi / g.mean_area()
        );
        assert!(rel < 1e-9, "n {n}: {rel:e}");
        // Moser's map is equal-area: every cell within a few per cent of the mean.
        assert!(
            lo / g.mean_area() > 0.9 && hi / g.mean_area() < 1.1,
            "n {n}"
        );
    }
}

#[test]
fn ghost_fill_is_the_direct_fold() {
    for (n, g) in [(4usize, 1usize), (4, 4), (6, 2), (16, 3), (16, 16), (7, 3)] {
        let t = CubeTopology::new(n);
        let gt = GhostTable::new(&t, g);
        let ni = n as i64;
        let gi = g as i64;
        let (mut edge, mut corner) = (0, 0);
        for face in Face::ALL {
            for j in -gi..ni + gi {
                for i in -gi..ni + gi {
                    let src = gt.source(face, i, j);
                    let out_i = !(0..ni).contains(&i);
                    let out_j = !(0..ni).contains(&j);
                    if out_i && out_j {
                        // Corner: the x-direction ghost turned about the corner, which keeps
                        // the distance from the cube's corner, on the x neighbour.
                        corner += 1;
                        let (si, sj) = corner_source(ni, i, j);
                        assert!(!(0..ni).contains(&si) && (0..ni).contains(&sj));
                        let side = if i < 0 {
                            FaceSide::West
                        } else {
                            FaceSide::East
                        };
                        let (sf, a, b) = fold3d(ni, face, si, sj);
                        assert_eq!(sf, face.across(side).face);
                        assert_eq!(src, t.index(sf, a as usize, b as usize));
                        let [c, eu, ev] = frame_i(face);
                        let k = lin(&[
                            (ni, c),
                            (if i < 0 { -ni } else { ni }, eu),
                            (if j < 0 { -ni } else { ni }, ev),
                        ]);
                        let ghost = lin(&[(ni, c), (2 * i + 1 - ni, eu), (2 * j + 1 - ni, ev)]);
                        let folded = fold_point3d(ni, face, si, sj);
                        let d2 = |p: [i64; 3]| {
                            let d = [p[0] - k[0], p[1] - k[1], p[2] - k[2]];
                            dot_i(d, d)
                        };
                        assert_eq!(d2(ghost), d2(folded), "n {n} {face:?} ({i}, {j})");
                    } else {
                        if out_i || out_j {
                            edge += 1;
                        }
                        let (f, a, b) = fold3d(ni, face, i, j);
                        assert_eq!(
                            src,
                            t.index(f, a as usize, b as usize),
                            "n {n} g {g} {face:?} ({i}, {j})"
                        );
                    }
                }
            }
        }
        assert_eq!(edge, 6 * 4 * n * g);
        assert_eq!(corner, 6 * 4 * g * g);
        // The fill is the gather of the table.
        let field: Vec<u64> = (0..t.cells() as u64).map(|c| c * 7 + 3).collect();
        let mut out = vec![0u64; 6 * gt.side() * gt.side()];
        gt.fill(&field, &mut out);
        for face in Face::ALL {
            for j in -gi..ni + gi {
                for i in -gi..ni + gi {
                    let q = face.index() * gt.side() * gt.side()
                        + ((j + gi) as usize) * gt.side()
                        + (i + gi) as usize;
                    assert_eq!(out[q], field[gt.source(face, i, j)]);
                }
            }
        }
    }
}

#[test]
fn edge_ghosts_match_the_nets_fold() {
    // The format's own fold of the net, block by block, on the first ghost ring away from the
    // face's corners (where two bands of the net meet, the fold splits the void square on its
    // diagonal, which is not a face's ghost).
    for n in [6usize, 16] {
        let t = CubeTopology::new(n);
        let gt = GhostTable::new(&t, 1);
        let cube = Cube::new(n as u32);
        let ni = n as i64;
        let mut checked = 0;
        for face in Face::ALL {
            let mut ghosts = Vec::new();
            for k in 1..ni - 1 {
                ghosts.extend([(-1, k), (ni, k), (k, -1), (k, ni)]);
            }
            for (i, j) in ghosts {
                let (x, z) = cube.block_of_cell(face, i, j);
                let (f, a, b) = match cube.fold(x, z) {
                    CubeFold::Face { face, i, j } => (face, i, j),
                    CubeFold::Outside { x, z, .. } => cube.face_of_block(x, z).unwrap(),
                    CubeFold::Ocean => panic!("{face:?} ({i}, {j}) folds to the ocean"),
                };
                assert_eq!(
                    gt.source(face, i, j),
                    t.index(f, a as usize, b as usize),
                    "n {n} {face:?} ({i}, {j})"
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 6 * 4 * (n - 2));
    }
}

#[test]
fn hilbert_keys_walk_each_face() {
    for n in [16usize, 64] {
        let order = hilbert_order(n);
        assert_eq!(order.len(), 6 * n * n);
        let t = CubeTopology::new(n);
        let mut seen = vec![false; 6 * n * n];
        for f in Face::ALL {
            let mut d: Vec<u64> = (0..n * n)
                .map(|r| hilbert_key(n, f, r % n, r / n) & ((1 << 60) - 1))
                .collect();
            d.sort_unstable();
            assert_eq!(d, (0..(n * n) as u64).collect::<Vec<_>>());
        }
        for w in order.windows(2) {
            let (fa, ia, ja) = t.coords(w[0] as usize);
            let (fb, ib, jb) = t.coords(w[1] as usize);
            if fa == fb {
                assert_eq!(ia.abs_diff(ib) + ja.abs_diff(jb), 1, "{w:?}");
            }
        }
        for &c in &order {
            assert!(!seen[c as usize]);
            seen[c as usize] = true;
        }
    }
    // Not a power of two: still one key per cell.
    let n = 48;
    let mut keys: Vec<u64> = (0..6 * n * n)
        .map(|c| hilbert_key(n, Face::ALL[c / (n * n)], c % n, c % (n * n) / n))
        .collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), 6 * n * n);
}

#[test]
fn couplings_are_mutual_and_sides_positive() {
    for n in [5usize, 16, 64] {
        let g = CubeGrid::new(n, R);
        let t = g.topology();
        let (mut min_side, mut max_diag, mut max_dist) = (f64::INFINITY, 0.0f64, 0.0f64);
        let mut min_dist = f64::INFINITY;
        for c in 0..t.cells() {
            for (k, &m) in t.slots(c).iter().enumerate() {
                if m == NO_CELL {
                    continue;
                }
                let m = m as usize;
                let km = t.slot_of(m, c).unwrap();
                assert_eq!(g.dists(c)[k].to_bits(), g.dists(m)[km].to_bits());
                assert_eq!(g.lap(c)[k].to_bits(), g.lap(m)[km].to_bits());
                let step = (std::f64::consts::PI / 2.0) * R / n as f64;
                let rel = g.dists(c)[k] / step;
                if k % 2 == 0 {
                    min_side = min_side.min(g.lap(c)[k]);
                    min_dist = min_dist.min(rel);
                    max_dist = max_dist.max(rel);
                } else {
                    max_diag = max_diag.max(g.lap(c)[k].abs());
                }
            }
        }
        eprintln!(
            "cube n {n}: side coupling min {min_side:.4}, |diagonal| max {max_diag:.4}, side \
             step {min_dist:.4}..{max_dist:.4} of the mean"
        );
        assert!(min_side > 0.0, "n {n}: a side coupling {min_side} ≤ 0");
    }
}

/// The discrete Laplacian of f against λ f (unit sphere): the relative RMS error, the relative
/// maximum error, and the relative maximum away from the 24 corner cells.
fn harmonic_error(g: &CubeGrid, f: impl Fn([f64; 3]) -> f64, lambda: f64) -> [f64; 3] {
    let t = g.topology();
    let r2 = g.radius_m() * g.radius_m();
    let (mut e2, mut f2, mut emax, mut fmax, mut emid) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for c in 0..t.cells() {
        let fc = f(g.unit(c));
        let mut s = 0.0;
        let mut corner = false;
        for (k, &m) in t.slots(c).iter().enumerate() {
            if m == NO_CELL {
                corner = true;
            } else {
                s += g.lap(c)[k] * (f(g.unit(m as usize)) - fc);
            }
        }
        let l = s / (g.areas()[c] / r2);
        let e = l - lambda * fc;
        e2 += e * e;
        f2 += (lambda * fc) * (lambda * fc);
        emax = emax.max(e.abs());
        if !corner {
            emid = emid.max(e.abs());
        }
        fmax = fmax.max((lambda * fc).abs());
    }
    [(e2 / f2).sqrt(), emax / fmax, emid / fmax]
}

/// The coupling Laplacian converges on smooth fields. Its pointwise error does not vanish at the
/// cube's corners (linear elements on a kinked grid; the solution still converges, which the
/// diffusion tests check), and those 24 cells carry the RMS: so first order, not second.
#[test]
fn laplacian_of_harmonics() {
    let mut last = [f64::INFINITY; 2];
    for n in [16usize, 32, 64] {
        let g = CubeGrid::new(n, R);
        let l1 = harmonic_error(&g, |p| p[2], -2.0);
        let l2 = harmonic_error(&g, |p| p[0] * p[1], -6.0);
        eprintln!(
            "cube n {n}: Laplacian of Y1 rms {:.3e} max {:.3e} (off corners {:.3e}); of Y2 rms \
             {:.3e} max {:.3e} (off corners {:.3e})",
            l1[0], l1[1], l1[2], l2[0], l2[1], l2[2]
        );
        assert!(
            l1[0] < 0.6 * last[0] && l2[0] < 0.6 * last[1],
            "n {n}: no first-order convergence"
        );
        last = [l1[0], l2[0]];
    }
    assert!(last[0] < 0.01 && last[1] < 0.01, "{last:?}");
}

#[test]
fn bearing_and_latitude() {
    let n = 5;
    let g = CubeGrid::new(n, R);
    let t = g.topology();
    for face in [Face::F0, Face::F1, Face::F2, Face::F3] {
        let c = t.index(face, 2, 2);
        assert!(g.bearing(c).abs() < 1e-12, "{face:?} {}", g.bearing(c));
        assert!(g.lat_deg(c).abs() < 1e-12);
    }
    assert!((g.lat_deg(t.index(Face::N, 2, 2)) - 90.0).abs() < 1e-9);
    assert!((g.lat_deg(t.index(Face::S, 2, 2)) + 90.0).abs() < 1e-9);
    // Up on F0's top row points north, at F0's centre column.
    let c = t.index(Face::F0, 2, 0);
    assert!(g.bearing(c).abs() < 1e-9 && g.lat_deg(c) > 20.0);
}

#[test]
fn face_sum_is_in_face_order() {
    let n = 4;
    let v: Vec<f64> = (0..6 * n * n).map(|c| 1.0 + c as f64 * 1e-3).collect();
    let mut want = 0.0;
    for f in 0..6 {
        let mut fs = 0.0;
        for j in 0..n {
            let mut r = 0.0;
            for i in 0..n {
                r += v[f * n * n + j * n + i];
            }
            fs += r;
        }
        want += fs;
    }
    assert_eq!(face_sum(n, &v).to_bits(), want.to_bits());
}

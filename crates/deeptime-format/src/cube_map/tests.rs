// SPDX-License-Identifier: LGPL-3.0-or-later
//! The checks of plan §1.6 M2 on both shipped tables.

use super::*;
use crate::cube::{ADJACENCY, FaceSide};

struct Rng(u64);
impl Rng {
    fn f(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
    }
    fn sym(&mut self) -> f64 {
        self.f() * 2.0 - 1.0
    }
}

fn maps() -> [Arc<CubeMap>; 2] {
    [CubeMap::world(), CubeMap::simulation()]
}

/// Half a face at 32k (s = 7,424), blocks per unit of u.
const H32K: f64 = 3_712.0;

fn sub(p: [f64; 3], q: [f64; 3]) -> [f64; 3] {
    [p[0] - q[0], p[1] - q[1], p[2] - q[2]]
}

fn norm(p: [f64; 3]) -> f64 {
    dot(p, p).sqrt()
}

/// The largest component of |p − q|.
fn max_diff(p: [f64; 3], q: [f64; 3]) -> f64 {
    let d = sub(p, q);
    d[0].abs().max(d[1].abs()).max(d[2].abs())
}

#[test]
fn the_shipped_tables_load_and_match_their_hashes() {
    let w = CubeMap::world();
    assert_eq!(w.name(), "conformal-n513");
    assert_eq!(w.tau(), 0.0);
    assert_eq!(w.flow_steps(), 0);
    assert_eq!((w.cells(), w.inverse_cells()), (256, 128));
    assert_eq!(w.corner_patch(), Some((4, 16)));
    assert_eq!(
        w.sha256_hex(),
        "c9dd9fd493ac905237678150c332fd580ed05bf0967af9519a85b889e279b9e1"
    );
    let s = CubeMap::simulation();
    assert_eq!(s.name(), "moser-t1-n513");
    assert_eq!(s.tau(), 1.0);
    assert_eq!(s.flow_steps(), 400);
    assert_eq!(s.corner_patch(), None);
    assert_eq!(
        s.sha256_hex(),
        "000e6532b89d84de287fbf9694fa080a3845261483e195b254d3b5a42a34f9d9"
    );
    assert!(CubeMap::builtin("lambert").is_none());
    assert_eq!(CubeMap::builtin_names(), [WORLD_MAP, SIMULATION_MAP]);
    // The generator's commit is in the header.
    assert_eq!(w.commit(), "6d28fa977d1d");
    assert_eq!(s.commit(), "0e645a133d08");
}

#[test]
fn bad_tables_are_refused() {
    let bytes = BUILTIN[0].1;
    assert!(CubeMap::load(&bytes[..bytes.len() - 16]).is_err());
    let mut wrong = bytes.to_vec();
    wrong[0] = b'X';
    assert!(CubeMap::load(&wrong).is_err());
    let mut version = bytes.to_vec();
    version[4] = 2;
    assert!(CubeMap::load(&version).is_err());
    let other = BUILTIN[1].2.split_whitespace().next().unwrap();
    let e = CubeMap::load_checked(bytes, other).unwrap_err().to_string();
    assert!(e.contains("sha256"), "{e}");
    // One flipped bit in a node changes the hash.
    let mut flipped = bytes.to_vec();
    flipped[HEADER_BYTES + 100] ^= 1;
    let own = BUILTIN[0].2.split_whitespace().next().unwrap();
    assert!(CubeMap::load_checked(&flipped, own).is_err());
}

#[test]
fn the_triangle_is_exact_on_its_lines() {
    for map in maps() {
        assert_eq!(map.triangle(0.0, 0.0), [0.0, 0.0]);
        assert_eq!(map.triangle(1.0, 1.0), [1.0, 1.0]);
        assert_eq!(map.triangle(1.0, 0.0), [1.0, 0.0]);
        let mut g = Rng(7);
        for _ in 0..2_000 {
            let t = g.f();
            assert_eq!(map.triangle(t, 0.0)[1], 0.0, "the axis");
            assert_eq!(map.triangle(1.0, t)[0], 1.0, "the edge");
        }
        // The face's centre and its edge midpoints.
        assert_eq!(map.forward(Face::F0, 0.0, 0.0), [1.0, 0.0, 0.0]);
        let mid = map.forward(Face::F0, 1.0, 0.0);
        let r = 0.5f64.sqrt();
        // 1/√2 by 1/r and by √0.5 differ in the last bit.
        assert!((mid[0] - r).abs() <= 1.2e-16 && mid[0] == mid[1] && mid[2] == 0.0);
    }
}

/// The readers' evaluator is the generator's: its inverse table (made by cube_map.py's bicubic)
/// starts within its own accuracy of where Newton on this bicubic ends.
#[test]
fn the_inverse_table_fits_this_bicubic() {
    for map in maps() {
        let mut g = Rng(11);
        let mut worst: f64 = 0.0;
        for _ in 0..20_000 {
            let (mut u, mut v) = (g.f(), g.f());
            if v > u {
                std::mem::swap(&mut u, &mut v);
            }
            let ab = map.triangle(u, v);
            let (su, sv) = map.inverse_start(ab[0], ab[1]);
            worst = worst.max((su - u).abs()).max((sv - v).abs());
        }
        // check.py: the start is within 8.2 blocks at 32k (2.2e-3 of u).
        assert!(worst < 2.5e-3, "{}: start off by {worst}", map.name());
    }
}

#[test]
fn forward_is_a_unit_vector_on_its_face() {
    for map in maps() {
        let mut g = Rng(3);
        for _ in 0..50_000 {
            let face = Face::ALL[(g.f() * 6.0) as usize];
            let (u, v) = (g.sym() * 0.999_999, g.sym() * 0.999_999);
            let p = map.forward(face, u, v);
            assert!((norm(p) - 1.0).abs() < 4e-16, "{p:?}");
            assert_eq!(Face::of_vector(p), face, "({u}, {v})");
        }
    }
}

/// inverse(forward(p)) within 1e-6 block at 32k: 10⁶ random points on each table.
#[test]
fn the_round_trip_is_within_a_micro_block() {
    for map in maps() {
        let mut g = Rng(0x0C0B_E001);
        let mut worst: f64 = 0.0;
        for k in 0..1_000_000 {
            let face = Face::ALL[k % 6];
            // A quarter of the points within 4 coarse cells of a corner (the conformal patch).
            let (u, v) = if k % 4 == 0 {
                let a = 1.0 - g.f() * 4.0 / 256.0;
                let b = 1.0 - g.f() * 4.0 / 256.0;
                (
                    if g.f() < 0.5 { -a } else { a },
                    if g.f() < 0.5 { -b } else { b },
                )
            } else {
                (g.sym(), g.sym())
            };
            let p = map.forward(face, u, v);
            let (f2, u2, v2) = map.inverse(p);
            assert_eq!(f2, face, "{} ({u}, {v})", map.name());
            let e = (u2 - u).abs().max((v2 - v).abs()) * H32K;
            worst = worst.max(e);
        }
        assert!(worst < 1e-6, "{}: {worst} block", map.name());
        eprintln!("{}: round trip worst {worst:.3e} block at 32k", map.name());
    }
}

/// The round trip on every face edge: the inverse may give the neighbour (ties go to x, y, z),
/// which must then be the same net point seen from across the edge.
#[test]
fn the_round_trip_holds_on_every_edge() {
    for map in maps() {
        let mut worst: f64 = 0.0;
        let mut g = Rng(5);
        for face in Face::ALL {
            for side in FaceSide::ALL {
                for k in 0..=2_000 {
                    // Exact grid points, random points and the corners' neighbourhood.
                    let t = match k % 3 {
                        0 => -1.0 + 2.0 * (k as f64) / 2_000.0,
                        1 => g.sym(),
                        _ => (1.0 - g.f() * 1e-3) * if g.f() < 0.5 { -1.0 } else { 1.0 },
                    };
                    let (u, v) = match side {
                        FaceSide::East => (1.0, t),
                        FaceSide::West => (-1.0, t),
                        FaceSide::South => (t, 1.0),
                        FaceSide::North => (t, -1.0),
                    };
                    let p = map.forward(face, u, v);
                    let (f2, u2, v2) = map.inverse(p);
                    let (eu, ev) = if f2 == face {
                        (u2 - u, v2 - v)
                    } else if t.abs() < 1.0 {
                        let a = ADJACENCY[face.index()][side.index()];
                        assert_eq!(a.face, f2, "{face:?} {side:?} t {t}");
                        let (nu, nv) = a.local(u, v);
                        (u2 - nu, v2 - nv)
                    } else {
                        // A cube corner: any of three faces, the same planet point.
                        let q = map.forward(f2, u2, v2);
                        (norm(sub(p, q)), 0.0)
                    };
                    worst = worst.max(eu.abs().max(ev.abs()) * H32K);
                }
            }
        }
        assert!(worst < 1e-6, "{}: {worst} block", map.name());
        eprintln!(
            "{}: edge round trip worst {worst:.3e} block at 32k",
            map.name()
        );
    }
}

/// The two faces' forward values at a shared edge agree to 1e-15.
#[test]
fn the_faces_agree_at_their_edges() {
    for map in maps() {
        let mut worst: f64 = 0.0;
        let mut g = Rng(13);
        for face in Face::ALL {
            for side in FaceSide::ALL {
                let a = ADJACENCY[face.index()][side.index()];
                for k in 0..=1_000 {
                    let t = if k % 2 == 0 {
                        -1.0 + 2.0 * (k as f64) / 1_000.0
                    } else {
                        g.sym()
                    };
                    let (u, v) = match side {
                        FaceSide::East => (1.0, t),
                        FaceSide::West => (-1.0, t),
                        FaceSide::South => (t, 1.0),
                        FaceSide::North => (t, -1.0),
                    };
                    let (nu, nv) = a.local(u, v);
                    let (p, q) = (map.forward(face, u, v), map.forward(a.face, nu, nv));
                    worst = worst.max(max_diff(p, q));
                }
            }
        }
        assert!(worst <= 1e-15, "{}: {worst}", map.name());
        eprintln!("{}: edge gap {worst:.3e}", map.name());
    }
}

/// `jacobian` against central differences of `forward`, to 1e-6 (per unit of u and v). The step
/// is small because the bicubic is only C¹: its second derivative jumps at knots, and the corner
/// patch's fine cells (1/4,096 of u) carry the conformal corner's large curvature.
#[test]
fn the_jacobian_matches_finite_differences() {
    let h = 1e-8;
    for map in maps() {
        let mut g = Rng(17);
        let mut worst: f64 = 0.0;
        for k in 0..100_000 {
            let face = Face::ALL[k % 6];
            let lim = 1.0 - 2.0 * h;
            let (u, v) = if k % 3 == 0 {
                (
                    (1.0 - g.f() * 0.02) * if g.f() < 0.5 { -lim } else { lim },
                    (1.0 - g.f() * 0.02) * if g.f() < 0.5 { -lim } else { lim },
                )
            } else {
                (g.sym() * lim, g.sym() * lim)
            };
            let j = map.jacobian(face, u, v);
            assert_eq!(j.p, map.forward(face, u, v));
            let fd = |a: [f64; 3], b: [f64; 3]| {
                let d = sub(a, b);
                [d[0] / (2.0 * h), d[1] / (2.0 * h), d[2] / (2.0 * h)]
            };
            let fu = fd(map.forward(face, u + h, v), map.forward(face, u - h, v));
            let fv = fd(map.forward(face, u, v + h), map.forward(face, u, v - h));
            worst = worst.max(max_diff(fu, j.du)).max(max_diff(fv, j.dv));
        }
        assert!(worst < 1e-6, "{}: {worst}", map.name());
        eprintln!("{}: jacobian vs differences {worst:.3e}", map.name());
    }
}

/// The derivative is positive (no fold-overs) on the face, out to the corner.
#[test]
fn the_face_never_folds_over() {
    for map in maps() {
        let mut g = Rng(19);
        let mut least = f64::INFINITY;
        for k in 0..200_000 {
            let (u, v) = if k % 2 == 0 {
                (1.0 - g.f() * 1e-3, 1.0 - g.f() * 1e-3)
            } else {
                (g.sym(), g.sym())
            };
            let j = map.jacobian(Face::F0, u, v);
            // The oriented area: (dp/du × dp/dv)·p, negative by the frames' e_u × e_v = −centre.
            let c = [
                j.du[1] * j.dv[2] - j.du[2] * j.dv[1],
                j.du[2] * j.dv[0] - j.du[0] * j.dv[2],
                j.du[0] * j.dv[1] - j.du[1] * j.dv[0],
            ];
            least = least.min(-dot(c, j.p));
        }
        assert!(least > 0.0, "{}: {least}", map.name());
    }
}

fn world(map: Arc<CubeMap>, turn: CubeTurn) -> CubeWorld {
    CubeWorld::new(Cube::new(7_424), map, turn, 300.0)
}

/// A turn that tilts the polar axis by `tilt` degrees towards `azimuth`, then spins about the cube's
/// own axis by `spin` (a free turn of the corner search).
fn free_turn(tilt: f64, azimuth: f64, spin: f64) -> CubeTurn {
    let (t, a, s) = (tilt.to_radians(), azimuth.to_radians(), spin.to_radians());
    // Rodrigues: rotate by t about the horizontal axis (sin a, −cos a, 0), then by s about z.
    let k = [a.sin(), -a.cos(), 0.0];
    let (st, ct) = t.sin_cos();
    let mut r = [[0.0; 3]; 3];
    for (i, row) in r.iter_mut().enumerate() {
        for (j, x) in row.iter_mut().enumerate() {
            let kk = [[0.0, -k[2], k[1]], [k[2], 0.0, -k[0]], [-k[1], k[0], 0.0]];
            let k2 = kk[i][0] * kk[0][j] + kk[i][1] * kk[1][j] + kk[i][2] * kk[2][j];
            *x = f64::from(u8::from(i == j)) + st * kk[i][j] + (1.0 - ct) * k2;
        }
    }
    let (ss, cs) = s.sin_cos();
    let z = [[cs, -ss, 0.0], [ss, cs, 0.0], [0.0, 0.0, 1.0]];
    let mut m = [[0.0; 3]; 3];
    for (i, row) in m.iter_mut().enumerate() {
        for (j, x) in row.iter_mut().enumerate() {
            *x = z[i][0] * r[0][j] + z[i][1] * r[1][j] + z[i][2] * r[2][j];
        }
    }
    CubeTurn::from_rows(m).unwrap()
}

/// The poles come from the turn, not from the faces: on the polar cube they are N's and S's
/// centres; turned freely (the study's median tilt 38°, up to 46°) they move off them, and the
/// world still puts latitude 90 there.
#[test]
fn the_poles_are_found_through_the_turn() {
    let polar = world(CubeMap::simulation(), CubeTurn::IDENTITY);
    let (f, u, v) = polar.north_pole();
    assert_eq!(f, Face::N);
    assert!(u.abs() < 1e-12 && v.abs() < 1e-12, "({u}, {v})");
    let (f, u, v) = polar.south_pole();
    assert_eq!(f, Face::S);
    assert!(u.abs() < 1e-12 && v.abs() < 1e-12, "({u}, {v})");
    let c = polar.corner_lat_lon([1, 1, 1]);
    assert!((c.0 - 35.264_389_682_754_654).abs() < 1e-9 && (c.1 - 45.0).abs() < 1e-9);

    for (tilt, az, spin) in [(38.0, 20.0, 10.0), (46.0, 200.0, -30.0), (12.0, 95.0, 44.0)] {
        let w = world(CubeMap::world(), free_turn(tilt, az, spin));
        for (north, (f, u, v)) in [(true, w.north_pole()), (false, w.south_pole())] {
            let p = w.unit_on_face(f, u, v);
            let want = if north { 1.0 } else { -1.0 };
            assert!((p[2] - want).abs() < 1e-12, "tilt {tilt}: {p:?}");
            assert!(
                u.abs().max(v.abs()) > 0.1,
                "tilt {tilt}: the pole left the centre"
            );
            let (x, z) = w.cube().point_of_local(f, u, v);
            let (lat, _) = w.lat_lon(x, z).unwrap();
            assert!((lat - 90.0 * want).abs() < 1e-5, "tilt {tilt}: lat {lat}");
            assert_eq!(w.point_of_unit([0.0, 0.0, want]), (x, z));
        }
    }
}

/// The world's 2 × 2 against differences of the planet point in metres, on the faces and beyond
/// net edges; sigma and the angle from it.
#[test]
fn the_world_jacobian_is_metres_per_block() {
    for (map, turn) in [
        (CubeMap::world(), free_turn(38.0, 20.0, 10.0)),
        (CubeMap::simulation(), CubeTurn::IDENTITY),
    ] {
        let w = world(map, turn);
        let s = 7_424.0;
        let mut g = Rng(23);
        let mut worst: f64 = 0.0;
        let mut n = 0;
        while n < 20_000 {
            let (x, z) = (g.sym() * 2.0 * s + 0.5 * s, g.sym() * 1.5 * s);
            let Some(j) = w.jacobian(x, z) else { continue };
            let Some(p0) = w.unit(x, z) else { continue };
            let h = 1e-3;
            let (Some(a), Some(b), Some(c), Some(d)) = (
                w.unit(x + h, z),
                w.unit(x - h, z),
                w.unit(x, z + h),
                w.unit(x, z - h),
            ) else {
                continue;
            };
            // Skip points whose stencil crosses a net corner's void.
            if norm(sub(a, b)) > 1e-5 || norm(sub(c, d)) > 1e-5 {
                continue;
            }
            n += 1;
            let (east, north) = local_frame(p0);
            let r = w.radius_m();
            let dx = sub(a, b);
            let dz = sub(c, d);
            let fd = [
                [
                    dot(dx, east) * r / (2.0 * h),
                    dot(dx, north) * r / (2.0 * h),
                ],
                [
                    dot(dz, east) * r / (2.0 * h),
                    dot(dz, north) * r / (2.0 * h),
                ],
            ];
            for (fr, jr) in fd.iter().zip(&j.m) {
                for (a, b) in fr.iter().zip(jr) {
                    worst = worst.max((a - b).abs() / j.sigma);
                }
            }
            let det = j.m[0][0] * j.m[1][1] - j.m[0][1] * j.m[1][0];
            assert!(det < 0.0, "net +z is south of net +x: det {det}");
            assert!((j.sigma - det.abs().sqrt()).abs() <= 1e-12 * j.sigma);
            assert!((pm::cos(j.angle) * j.m[0][0].hypot(j.m[0][1]) - j.m[0][0]).abs() < 1e-9);
        }
        assert!(worst < 1e-5, "{}: {worst}", w.map().name());
    }
    // The equal-area map's sigma is the nominal 300 m per block within its 0.6 % (outside the
    // corner cells).
    let m = world(CubeMap::simulation(), CubeTurn::IDENTITY);
    let mut g = Rng(29);
    for _ in 0..20_000 {
        let (u, v) = (g.sym() * 0.99, g.sym() * 0.99);
        let j = m.jacobian_on_face(Face::ALL[(g.f() * 6.0) as usize], u, v);
        assert!((j.sigma / 300.0 - 1.0).abs() < 0.006, "sigma {}", j.sigma);
    }
    // The conformal map's 2 × 2 is a scale times a rotation and a reflection.
    let c = world(CubeMap::world(), free_turn(38.0, 20.0, 10.0));
    for _ in 0..20_000 {
        let (x, z) = (g.sym() * 0.5 * 7_424.0, g.sym() * 0.5 * 7_424.0);
        let j = c.jacobian(x, z).unwrap().m;
        let (lx, lz) = (j[0][0].hypot(j[0][1]), j[1][0].hypot(j[1][1]));
        let cos = (j[0][0] * j[1][0] + j[0][1] * j[1][1]) / (lx * lz);
        assert!((lx / lz - 1.0).abs() < 2e-3 && cos.abs() < 2e-3, "{j:?}");
    }
}

/// The world's north in blocks points to rising latitude.
#[test]
fn north_points_up_the_latitude() {
    let w = world(CubeMap::world(), free_turn(38.0, 20.0, 10.0));
    let mut g = Rng(31);
    for _ in 0..5_000 {
        let (x, z) = (g.sym() * 0.49 * 7_424.0, g.sym() * 0.49 * 7_424.0);
        let (lat0, _) = w.lat_lon(x, z).unwrap();
        if lat0.abs() > 85.0 {
            continue;
        }
        let (dx, dz) = w.north(x, z).unwrap();
        let (lat1, _) = w.lat_lon(x + dx, z + dz).unwrap();
        assert!(lat1 > lat0, "({x}, {z}): {lat0} -> {lat1}");
    }
}

/// The fastest of seven timings of `f` over `n` calls, in ns per call (a shared VM: the fastest
/// run is the one least disturbed by other jobs).
fn best_ns(n: usize, mut f: impl FnMut() -> f64) -> f64 {
    let mut best = f64::INFINITY;
    let mut acc = 0.0;
    for _ in 0..7 {
        let t = std::time::Instant::now();
        acc += f();
        best = best.min(t.elapsed().as_nanos() as f64 / n as f64);
    }
    std::hint::black_box(acc);
    best
}

/// Speed: under 30 ns per forward on the VM (plan §1.6 M2). Run with
/// `cargo test --release -p deeptime-format forward_speed -- --ignored --nocapture`.
#[test]
#[ignore]
fn forward_speed() {
    for map in maps() {
        let mut g = Rng(37);
        let pts: Vec<(Face, f64, f64)> = (0..1 << 16)
            .map(|k| (Face::ALL[k % 6], g.sym(), g.sym()))
            .collect();
        let reps = 40;
        let fwd = best_ns(reps * pts.len(), || {
            let mut acc = 0.0;
            for _ in 0..reps {
                for &(f, u, v) in &pts {
                    acc += map.forward(std::hint::black_box(f), u, v)[0];
                }
            }
            acc
        });
        let units: Vec<[f64; 3]> = pts.iter().map(|&(f, u, v)| map.forward(f, u, v)).collect();
        let inv = best_ns(reps / 4 * units.len(), || {
            let mut acc = 0.0;
            for _ in 0..reps / 4 {
                for &p in &units {
                    acc += map.inverse(std::hint::black_box(p)).1;
                }
            }
            acc
        });
        let jac = best_ns(reps / 4 * pts.len(), || {
            let mut acc = 0.0;
            for _ in 0..reps / 4 {
                for &(f, u, v) in &pts {
                    acc += map.jacobian(std::hint::black_box(f), u, v).du[0];
                }
            }
            acc
        });
        // A raster's order: each face row by row.
        let scan: Vec<(Face, f64, f64)> = Face::ALL
            .iter()
            .flat_map(|&f| {
                (0..104).flat_map(move |j| {
                    (0..104).map(move |i| {
                        (
                            f,
                            (i as f64 + 0.5) / 52.0 - 1.0,
                            (j as f64 + 0.5) / 52.0 - 1.0,
                        )
                    })
                })
            })
            .collect();
        let fscan = best_ns(reps * scan.len(), || {
            let mut acc = 0.0;
            for _ in 0..reps {
                for &(f, u, v) in &scan {
                    acc += map.forward(std::hint::black_box(f), u, v)[0];
                }
            }
            acc
        });
        eprintln!(
            "{}: forward {fwd:.1} ns (random points), {fscan:.1} ns (raster order), inverse {inv:.1} ns, jacobian {jac:.1} ns (best of 7)",
            map.name()
        );
    }
}

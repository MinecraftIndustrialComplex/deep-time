// SPDX-License-Identifier: LGPL-3.0-or-later
//! Where the cube world sits on the planet (cube plan B3, as the projection study's §5.4 replaced
//! it: `docs/research/notes/mountains-and-the-projection.md`; science page
//! `docs/science/cube-world-bake.md`). The cube's eight corners are where the conformal map squeezes
//! the planet most (a corner's cell holds no planet at all), so the turn puts them as far from land
//! as it can:
//!
//! - **The score** of a turn Q (planet to cube): with c_k = (±1, ±1, ±1)/√3 in the corners' order,
//!   min_k d(Qᵀ c_k) + 0.001 · Σ_k d(Qᵀ c_k), d the signed great-circle distance to the coast
//!   (positive at sea, minus the distance to the sea on land, clamped to ±20°). The four reachable
//!   corners and the four on the border count alike.
//! - **The search**: a seeded generator (the planet's seed) draws 60,000 uniform random turns
//!   (Shoemake's quaternions); with the best turn about the polar axis (λ0 in 0.25° steps over 90°)
//!   the best 24 are refined over 120 rounds of 32 trials, small random turns whose size shrinks
//!   geometrically from 4° to 0.03°, keeping improvements. The distances are read from k-d trees
//!   of a Fibonacci sphere's land and sea points; the record's from rings round each corner.
//! - **The symmetry**: the cube's 24 rotations give the same corners. The one taken puts the north
//!   pole in face N (the largest cube z of the pole), then of N's four quarter turns the one with
//!   the most land on F0, ties by index.
//!
//! A preset that sets `scale.lambda_c_deg` gets the polar turn by that λ0 ([`CubeTurn::polar`]), so
//! A/B bakes of one planet keep one frame.

use glam::DVec3;
use rayon::prelude::*;
use serde_json::{Value, json};

use deeptime_format::cube::{Cube, CubeTurn, Face};
use deeptime_format::cube_map::CubeView;
use deeptime_format::projection::{lat_lon_of_unit, local_frame};

use crate::sphere::kdtree::angle_of_chord2;
use crate::sphere::{SampleTree, SplitMix64, fibonacci_sphere};

/// Signed distances to the coast are clamped to ± this, degrees: beyond it every corner is as
/// good, and the sum decides.
pub const CLAMP_DEG: f64 = 20.0; // ledger: SCL-120 (the λc search's LAND_SEARCH_DEG, SCL-087)

/// The weight of the corners' summed distance in the score (the least distance leads).
pub const SUM_WEIGHT: f64 = 0.001; // ledger: SCL-120 (game-choice, study §5.4 item 3)

/// The land the record counts as near a corner lies within this many degrees of it.
pub const CORNER_ZONE_DEG: f64 = 5.0; // ledger: SCL-121 (study §5.4 item 7)

/// The rings the record's distances are read on, degrees apart (360 directions each).
pub const RING_DEG: f64 = 0.05; // ledger: SCL-120 (game-choice: the record's resolution)

/// The size of a turn search.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TurnSearch {
    /// Fibonacci points the land and sea trees hold.
    pub samples: usize,
    /// Uniform random turns drawn.
    pub random: usize,
    /// The best random turns refined (the best polar turn is refined beside them).
    pub keep: usize,
    /// Refinement rounds, and trials in each.
    pub rounds: usize,
    pub trials: usize,
    /// The trials' largest turn, degrees: from `start_deg` in the first round to `end_deg` in the
    /// last, geometrically.
    pub start_deg: f64,
    pub end_deg: f64,
    /// The polar candidates' λ0 step, degrees (over 90°).
    pub polar_step_deg: f64,
}

/// The bake's search (study §5.4 item 4). ledger: SCL-120 (game-choice: the search's size)
pub const BAKE: TurnSearch = TurnSearch {
    samples: 400_000,
    random: 60_000,
    keep: 24,
    rounds: 120,
    trials: 32,
    start_deg: 4.0,
    end_deg: 0.03,
    polar_step_deg: 0.25,
};

/// The deposit census's forecast of the bake's turn (the coarser land of the forecast needs no
/// finer search). ledger: SCL-120 (game-choice)
pub const FORECAST: TurnSearch = TurnSearch {
    samples: 100_000,
    random: 20_000,
    keep: 8,
    rounds: 60,
    trials: 24,
    start_deg: 4.0,
    end_deg: 0.05,
    polar_step_deg: 0.5,
};

/// The generator's salts (the random stage, the refinement), so the two draw apart.
const SALT_RANDOM: u64 = 0xC0BE_7A11_0000_0001;
const SALT_REFINE: u64 = 0xC0BE_7A11_0000_0002;

/// The planet's land as the search reads it: a land test and the land and sea points of a
/// Fibonacci sphere in k-d trees.
pub struct Land<'a> {
    test: &'a (dyn Fn([f64; 3]) -> bool + Sync),
    points: Vec<DVec3>,
    is_land: Vec<bool>,
    land: SampleTree,
    sea: SampleTree,
}

impl<'a> Land<'a> {
    /// The land of `test` (whether a planet unit vector is land) on `samples` Fibonacci points.
    pub fn new(test: &'a (dyn Fn([f64; 3]) -> bool + Sync), samples: usize) -> Land<'a> {
        let points = fibonacci_sphere(samples.max(1));
        let is_land: Vec<bool> = points.par_iter().map(|p| test(p.to_array())).collect();
        let pick = |want: bool| -> Vec<DVec3> {
            points
                .iter()
                .zip(&is_land)
                .filter(|&(_, &l)| l == want)
                .map(|(p, _)| *p)
                .collect()
        };
        let (land, sea) = rayon::join(|| SampleTree::new(&pick(true)), || {
            SampleTree::new(&pick(false))
        });
        Land {
            test,
            points,
            is_land,
            land,
            sea,
        }
    }

    /// The share of the points that are land.
    pub fn land_share(&self) -> f64 {
        self.is_land.iter().filter(|&&l| l).count() as f64 / self.points.len().max(1) as f64
    }

    /// The signed distance to the coast at `p`, degrees, from the trees (positive at sea, clamped
    /// to ±[`CLAMP_DEG`]).
    pub fn signed(&self, p: [f64; 3]) -> f64 {
        let q = DVec3::from_array(p);
        let (tree, sign) = if (self.test)(p) {
            (&self.sea, -1.0)
        } else {
            (&self.land, 1.0)
        };
        let (_, c2) = tree.nearest(q);
        sign * angle_of_chord2(c2).to_degrees().min(CLAMP_DEG)
    }

    /// The signed distance to the coast at `p`, degrees, read on rings [`RING_DEG`] apart of 360
    /// directions (the record's).
    pub fn signed_rings(&self, p: [f64; 3]) -> f64 {
        let here = (self.test)(p);
        let (e, n) = local_frame(p);
        let rings = (CLAMP_DEG / RING_DEG).round() as usize;
        let sign = if here { -1.0 } else { 1.0 };
        for k in 1..=rings {
            let a = (k as f64 * RING_DEG).to_radians();
            let (sa, ca) = a.sin_cos();
            for d in 0..360 {
                let (st, ct) = f64::from(d).to_radians().sin_cos();
                let v = [
                    p[0] * ca + (e[0] * ct + n[0] * st) * sa,
                    p[1] * ca + (e[1] * ct + n[1] * st) * sa,
                    p[2] * ca + (e[2] * ct + n[2] * st) * sa,
                ];
                if (self.test)(v) != here {
                    return sign * k as f64 * RING_DEG;
                }
            }
        }
        sign * CLAMP_DEG
    }
}

/// A quaternion (w, x, y, z).
type Quat = [f64; 4];

fn quat_mul(a: Quat, b: Quat) -> Quat {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}

fn quat_norm(q: Quat) -> Quat {
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if n > 0.0 && n.is_finite() {
        [q[0] / n, q[1] / n, q[2] / n, q[3] / n]
    } else {
        [1.0, 0.0, 0.0, 0.0]
    }
}

/// The rotation matrix of a unit quaternion (rows: the turn's, planet to cube).
fn quat_matrix(q: Quat) -> [[f64; 3]; 3] {
    let [w, x, y, z] = q;
    [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y - w * z),
            2.0 * (x * z + w * y),
        ],
        [
            2.0 * (x * y + w * z),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z - w * x),
        ],
        [
            2.0 * (x * z - w * y),
            2.0 * (y * z + w * x),
            1.0 - 2.0 * (x * x + y * y),
        ],
    ]
}

/// A uniformly distributed rotation (Shoemake 1992, "Uniform random rotations", Graphics Gems III).
fn random_quat(rng: &mut SplitMix64) -> Quat {
    let (u1, u2, u3) = (rng.next_f64(), rng.next_f64(), rng.next_f64());
    let tau = std::f64::consts::TAU;
    let (a, b) = ((1.0 - u1).sqrt(), u1.sqrt());
    [
        b * (tau * u3).cos(),
        a * (tau * u2).sin(),
        a * (tau * u2).cos(),
        b * (tau * u3).sin(),
    ]
}

/// A random turn of at most `max_deg` about a uniformly random axis.
fn small_quat(rng: &mut SplitMix64, max_deg: f64) -> Quat {
    let axis = rng.unit_vector();
    let half = 0.5 * (2.0 * rng.next_f64() - 1.0) * max_deg.to_radians();
    let s = half.sin();
    [half.cos(), axis.x * s, axis.y * s, axis.z * s]
}

/// The polar turn by λ0 ([`CubeTurn::polar`]: about the z axis by −λ0) as a quaternion.
fn polar_quat(lambda0_deg: f64) -> Quat {
    let h = -0.5 * lambda0_deg.to_radians();
    [h.cos(), 0.0, 0.0, h.sin()]
}

fn turn_of(m: [[f64; 3]; 3]) -> CubeTurn {
    CubeTurn::from_rows(m).unwrap_or(CubeTurn::IDENTITY)
}

/// The eight corners' cube-frame unit vectors in [`Cube::corners`]' order.
pub fn corner_dirs() -> [[f64; 3]; 8] {
    let r = 3.0f64.sqrt();
    let cs = Cube::new(2).corners();
    std::array::from_fn(|k| {
        let c = cs[k].cube;
        [
            f64::from(c[0]) / r,
            f64::from(c[1]) / r,
            f64::from(c[2]) / r,
        ]
    })
}

/// The score of rotation `m` (rows planet to cube): (score, the eight signed distances).
fn score(land: &Land<'_>, corners: &[[f64; 3]; 8], m: &[[f64; 3]; 3]) -> (f64, [f64; 8]) {
    let to_planet = |c: [f64; 3]| {
        [
            m[0][0] * c[0] + m[1][0] * c[1] + m[2][0] * c[2],
            m[0][1] * c[0] + m[1][1] * c[1] + m[2][1] * c[2],
            m[0][2] * c[0] + m[1][2] * c[1] + m[2][2] * c[2],
        ]
    };
    let d: [f64; 8] = std::array::from_fn(|k| land.signed(to_planet(corners[k])));
    let min = d.iter().copied().fold(f64::INFINITY, f64::min);
    (min + SUM_WEIGHT * d.iter().sum::<f64>(), d)
}

/// The cube's 24 rotations (signed permutation matrices of determinant +1), in a fixed order.
fn symmetries() -> Vec<[[f64; 3]; 3]> {
    let perms = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let mut out = Vec::with_capacity(24);
    for p in perms {
        for signs in 0..8u32 {
            let mut m = [[0.0; 3]; 3];
            for (i, row) in m.iter_mut().enumerate() {
                row[p[i]] = if signs >> i & 1 == 1 { -1.0 } else { 1.0 };
            }
            let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
                - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
                + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
            if det > 0.0 {
                out.push(m);
            }
        }
    }
    out
}

fn mat_mul(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    std::array::from_fn(|i| std::array::from_fn(|j| a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j]))
}

/// The turn the search chose, and how good it is.
#[derive(Clone, Debug, PartialEq)]
pub struct TurnChoice {
    pub turn: CubeTurn,
    /// Searched (true) or set by the preset's λ0 (false).
    pub chosen: bool,
    /// The preset's λ0 when set.
    pub lambda0_deg: Option<f64>,
    /// The score the search ended on (trees).
    pub score: f64,
    /// Each corner's signed distance to the coast, degrees, read on rings ([`Land::signed_rings`]),
    /// in the corners' order.
    pub distances_deg: [f64; 8],
    /// The share of the random turns with all eight corners at sea, and all eight at least 5°
    /// out.
    pub share_all_at_sea: f64,
    pub share_all_5deg: f64,
    /// The best random turn's and the best polar turn's least distance (trees), degrees, and the
    /// polar turn's λ0.
    pub best_random_min_deg: f64,
    pub best_polar_min_deg: f64,
    pub best_polar_lambda0_deg: f64,
    /// Whether the refined winner started from the polar turn.
    pub from_polar: bool,
    /// The symmetry taken (an index of the cube's 24 rotations in this module's order).
    pub symmetry: usize,
    pub search: Option<TurnSearch>,
}

impl TurnChoice {
    /// The least of the eight distances, degrees.
    pub fn min_distance_deg(&self) -> f64 {
        self.distances_deg
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min)
    }
}

/// The polar turn by `lambda0_deg` (a preset's pinned λc), with its corners' distances.
pub fn pinned(land: &Land<'_>, lambda0_deg: f64) -> TurnChoice {
    let turn = CubeTurn::polar(lambda0_deg);
    let corners = corner_dirs();
    let (sc, _) = score(land, &corners, &turn.rows());
    TurnChoice {
        turn,
        chosen: false,
        lambda0_deg: Some(lambda0_deg),
        score: sc,
        distances_deg: std::array::from_fn(|k| land.signed_rings(turn.to_planet(corners[k]))),
        share_all_at_sea: f64::NAN,
        share_all_5deg: f64::NAN,
        best_random_min_deg: f64::NAN,
        best_polar_min_deg: f64::NAN,
        best_polar_lambda0_deg: lambda0_deg,
        from_polar: true,
        symmetry: 0,
        search: None,
    }
}

/// The search (module documentation), seeded by `seed`.
pub fn search(land: &Land<'_>, seed: u64, s: &TurnSearch) -> TurnChoice {
    let corners = corner_dirs();
    // The random stage: one stream, so the candidates do not depend on the threads.
    let mut rng = SplitMix64::new(seed ^ SALT_RANDOM);
    let cands: Vec<Quat> = (0..s.random.max(1)).map(|_| random_quat(&mut rng)).collect();
    let scored: Vec<(f64, f64)> = cands
        .par_iter()
        .map(|&q| {
            let (sc, d) = score(land, &corners, &quat_matrix(q));
            (sc, d.iter().copied().fold(f64::INFINITY, f64::min))
        })
        .collect();
    let n = scored.len() as f64;
    let share_all_at_sea = scored.iter().filter(|s| s.1 > 0.0).count() as f64 / n;
    let share_all_5deg = scored.iter().filter(|s| s.1 >= 5.0).count() as f64 / n;
    let mut order: Vec<usize> = (0..scored.len()).collect();
    order.sort_by(|&a, &b| scored[b].0.total_cmp(&scored[a].0).then(a.cmp(&b)));
    let best_random_min_deg = scored[order[0]].1;
    // The best turn about the polar axis.
    let steps = (90.0 / s.polar_step_deg).round().max(1.0) as usize;
    let polar: Vec<(f64, f64)> = (0..steps)
        .into_par_iter()
        .map(|k| {
            let l0 = k as f64 * s.polar_step_deg;
            let (sc, d) = score(land, &corners, &quat_matrix(polar_quat(l0)));
            (sc, d.iter().copied().fold(f64::INFINITY, f64::min))
        })
        .collect();
    let pk = (0..steps)
        .max_by(|&a, &b| polar[a].0.total_cmp(&polar[b].0).then(b.cmp(&a)))
        .unwrap_or(0);
    let best_polar_lambda0_deg = pk as f64 * s.polar_step_deg;
    // The pool: the best random turns, then the best polar one; each refined on its own stream.
    let mut pool: Vec<Quat> = order.iter().take(s.keep).map(|&k| cands[k]).collect();
    pool.push(polar_quat(best_polar_lambda0_deg));
    let rounds = s.rounds.max(1);
    let refined: Vec<(f64, Quat)> = pool
        .par_iter()
        .enumerate()
        .map(|(i, &q0)| {
            let mut rng = SplitMix64::new(
                seed ^ SALT_REFINE ^ (i as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15),
            );
            let mut q = q0;
            let mut best = score(land, &corners, &quat_matrix(q)).0;
            for r in 0..rounds {
                let t = if rounds == 1 {
                    0.0
                } else {
                    r as f64 / (rounds - 1) as f64
                };
                let size = s.start_deg * (s.end_deg / s.start_deg).powf(t);
                for _ in 0..s.trials {
                    let qn = quat_norm(quat_mul(small_quat(&mut rng, size), q));
                    let sc = score(land, &corners, &quat_matrix(qn)).0;
                    if sc > best {
                        best = sc;
                        q = qn;
                    }
                }
            }
            (best, q)
        })
        .collect();
    let win = (0..refined.len())
        .max_by(|&a, &b| refined[a].0.total_cmp(&refined[b].0).then(b.cmp(&a)))
        .unwrap_or(0);
    let (best_score, q) = refined[win];
    let m = quat_matrix(q);
    // The symmetry: the north pole in N (the largest cube z of the pole), then the most land on F0.
    let syms = symmetries();
    let pole_z: Vec<f64> = syms
        .iter()
        .map(|g| {
            let t = mat_mul(g, &m);
            t[2][2]
        })
        .collect();
    let top = pole_z.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let land_pts: Vec<DVec3> = land
        .points
        .iter()
        .zip(&land.is_land)
        .filter(|&(_, &l)| l)
        .map(|(p, _)| *p)
        .collect();
    let mut pick: Option<(usize, usize)> = None;
    for (k, g) in syms.iter().enumerate() {
        if pole_z[k] < top - 1e-12 {
            continue;
        }
        let t = mat_mul(g, &m);
        let on_f0 = land_pts
            .par_iter()
            .filter(|p| {
                let c = [
                    t[0][0] * p.x + t[0][1] * p.y + t[0][2] * p.z,
                    t[1][0] * p.x + t[1][1] * p.y + t[1][2] * p.z,
                    t[2][0] * p.x + t[2][1] * p.y + t[2][2] * p.z,
                ];
                Face::of_vector(c) == Face::F0
            })
            .count();
        if pick.is_none_or(|(_, best)| on_f0 > best) {
            pick = Some((k, on_f0));
        }
    }
    let symmetry = pick.map_or(0, |p| p.0);
    let turn = turn_of(mat_mul(&syms[symmetry], &m));
    TurnChoice {
        turn,
        chosen: true,
        lambda0_deg: None,
        score: best_score,
        distances_deg: std::array::from_fn(|k| land.signed_rings(turn.to_planet(corners[k]))),
        share_all_at_sea,
        share_all_5deg,
        best_random_min_deg,
        best_polar_min_deg: polar[pk].1,
        best_polar_lambda0_deg,
        from_polar: win + 1 == refined.len(),
        symmetry,
        search: Some(*s),
    }
}

/// The record of the land under the cube world `view` (`notes.bake.stats`, study §5.4 item 7):
/// the corners (latitude, longitude, signed distance), the land's blocks per planet area (the
/// nominal block's area over the world block's: 1 on average over the planet, above 1 where the
/// map draws the planet larger), its largest, 99.9th percentile, median and least, and the land
/// within [`CORNER_ZONE_DEG`] of a corner as a share of the land and of the planet.
pub fn land_stats(view: &CubeView<'_>, land: &Land<'_>, choice: &TurnChoice) -> Value {
    let corners = corner_dirs();
    let units: Vec<[f64; 3]> = corners.iter().map(|&c| view.turn.to_planet(c)).collect();
    let zone = CORNER_ZONE_DEG.to_radians().cos();
    let land_pts: Vec<DVec3> = land
        .points
        .iter()
        .zip(&land.is_land)
        .filter(|&(_, &l)| l)
        .map(|(p, _)| *p)
        .collect();
    let per: Vec<(f64, bool)> = land_pts
        .par_iter()
        .map(|p| {
            let v = p.to_array();
            let (face, u, w) = view.local_of_unit(v);
            let sigma = view.jacobian_on_face(face, u, w).sigma;
            let near = units
                .iter()
                .any(|c| c[0] * v[0] + c[1] * v[1] + c[2] * v[2] >= zone);
            (1.0 / (sigma * sigma).max(1e-300), near)
        })
        .collect();
    let mut bpa: Vec<f64> = per.iter().map(|p| p.0).collect();
    bpa.sort_by(f64::total_cmp);
    let q = |f: f64| -> f64 {
        if bpa.is_empty() {
            return f64::NAN;
        }
        bpa[((f * (bpa.len() - 1) as f64).round() as usize).min(bpa.len() - 1)]
    };
    let near = per.iter().filter(|p| p.1).count();
    let r4 = |x: f64| (x * 1e4).round() / 1e4;
    let corner_list: Vec<Value> = Cube::new(2)
        .corners()
        .iter()
        .enumerate()
        .map(|(k, c)| {
            let (lat, lon) = lat_lon_of_unit(units[k]);
            json!({
                "name": c.name,
                "lat_deg": r4(lat),
                "lon_deg": r4(lon),
                "land_distance_deg": r4(choice.distances_deg[k]),
            })
        })
        .collect();
    json!({
        "corners": corner_list,
        "corner_min_land_distance_deg": r4(choice.min_distance_deg()),
        "land_samples": land_pts.len(),
        "land_share": r4(land.land_share()),
        "land_blocks_per_planet_area": {
            "max": r4(q(1.0)),
            "p999": r4(q(0.999)),
            "median": r4(q(0.5)),
            "min": r4(q(0.0)),
        },
        "land_within_5deg_of_corner": {
            "share_of_land": near as f64 / land_pts.len().max(1) as f64,
            "share_of_planet": near as f64 / land.points.len().max(1) as f64,
        },
    })
}

/// The record of a choice (`notes.bake.projection.turn`, the deposit census's
/// `world_frame.turn`).
pub fn turn_note(choice: &TurnChoice) -> Value {
    let r4 = |x: f64| {
        if x.is_finite() {
            json!((x * 1e4).round() / 1e4)
        } else {
            Value::Null
        }
    };
    let mut v = json!({
        "chosen": choice.chosen,
        "rotation": choice.turn.rows(),
        "score": r4(choice.score),
        "min_distance_to_land_deg": r4(choice.min_distance_deg()),
        "corner_distances_deg": choice.distances_deg.iter().map(|&d| r4(d)).collect::<Vec<_>>(),
    });
    match (choice.chosen, choice.search) {
        (true, Some(s)) => {
            v["share_all_at_sea"] = r4(choice.share_all_at_sea);
            v["share_all_5deg_from_land"] = r4(choice.share_all_5deg);
            v["best_random_min_deg"] = r4(choice.best_random_min_deg);
            v["best_polar_min_deg"] = r4(choice.best_polar_min_deg);
            v["best_polar_lambda0_deg"] = r4(choice.best_polar_lambda0_deg);
            v["won_from_polar"] = json!(choice.from_polar);
            v["symmetry"] = json!(choice.symmetry);
            v["search"] = json!({
                "samples": s.samples, "random": s.random, "keep": s.keep, "rounds": s.rounds,
                "trials": s.trials, "start_deg": s.start_deg, "end_deg": s.end_deg,
                "polar_step_deg": s.polar_step_deg, "ring_deg": RING_DEG,
            });
        }
        _ => {
            v["set_by"] = json!("scale.lambda_c_deg");
            v["lambda0_deg"] = r4(choice.lambda0_deg.unwrap_or(f64::NAN));
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::projection::unit_of_lat_lon;

    /// A planet whose land is a cap of `deg` round (lat, lon).
    fn cap(lat: f64, lon: f64, deg: f64) -> impl Fn([f64; 3]) -> bool + Sync {
        let c = unit_of_lat_lon(lat, lon);
        let cos = deg.to_radians().cos();
        move |v: [f64; 3]| c[0] * v[0] + c[1] * v[1] + c[2] * v[2] >= cos
    }

    const SMALL: TurnSearch = TurnSearch {
        samples: 20_000,
        random: 2_000,
        keep: 4,
        rounds: 20,
        trials: 12,
        start_deg: 4.0,
        end_deg: 0.1,
        polar_step_deg: 1.0,
    };

    /// The search is the same on any number of threads, puts the north pole in N and the land on
    /// F0, and finds corners well out at sea round one continent.
    #[test]
    fn the_search_is_deterministic_and_keeps_the_symmetry_rule() {
        let test = cap(20.0, 40.0, 30.0);
        let land = Land::new(&test, SMALL.samples);
        let a = search(&land, 7, &SMALL);
        let b = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap()
            .install(|| search(&land, 7, &SMALL));
        assert_eq!(a, b);
        // The north pole's cube z is its largest component.
        let p = a.turn.to_cube([0.0, 0.0, 1.0]);
        assert_eq!(Face::of_vector(p), Face::N, "pole at {p:?}");
        // The land's centre lies on F0.
        let c = a.turn.to_cube(unit_of_lat_lon(20.0, 40.0));
        assert_eq!(Face::of_vector(c), Face::F0, "land centre at {c:?}");
        // A 30° cap leaves room for every corner well out at sea (a corner is 54.7° from the
        // nearest face centre, 35.3° from an edge's middle).
        assert!(a.min_distance_deg() > 5.0, "{:?}", a.distances_deg);
        // The rings and the trees agree to the trees' spacing (20,000 points: about 1.6°).
        for d in a.distances_deg {
            assert!(d > 0.0);
        }
    }

    /// A pinned λ0 gives the polar turn; the 24 symmetries are rotations.
    #[test]
    fn pinned_turns_and_symmetries() {
        let test = cap(0.0, 0.0, 10.0);
        let land = Land::new(&test, 5_000);
        let c = pinned(&land, 30.0);
        assert_eq!(c.turn, CubeTurn::polar(30.0));
        assert!(!c.chosen);
        let syms = symmetries();
        assert_eq!(syms.len(), 24);
        for g in &syms {
            assert!(CubeTurn::from_rows(*g).is_ok());
        }
        // The quaternion of the polar turn is the polar turn's matrix.
        let m = quat_matrix(polar_quat(30.0));
        let r = CubeTurn::polar(30.0).rows();
        for i in 0..3 {
            for j in 0..3 {
                assert!((m[i][j] - r[i][j]).abs() < 1e-12);
            }
        }
    }

    /// Ring distances read the cap's edge to the ring step.
    #[test]
    fn ring_distances_read_the_coast() {
        let test = cap(0.0, 0.0, 10.0);
        let land = Land::new(&test, 5_000);
        let at = |lat: f64, lon: f64| land.signed_rings(unit_of_lat_lon(lat, lon));
        assert!((at(0.0, 15.0) - 5.0).abs() <= RING_DEG + 1e-9);
        assert!((at(0.0, 0.0) + 10.0).abs() <= RING_DEG + 1e-9);
        assert_eq!(at(0.0, 90.0), CLAMP_DEG);
    }
}

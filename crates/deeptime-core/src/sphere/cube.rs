// SPDX-License-Identifier: LGPL-3.0-or-later
//! The simulation's grid on the cube (docs/plans/cube-world-plan.md §3, work item S1): six faces of
//! n × n cells on Moser's equal-area cube (`CubeMap::simulation()`, the slot `moser-t1-n513`), in
//! the polar-axis cube frame (+Z the north pole, +X the point (0°, 0°)).
//!
//! # Storage
//! Cells are stored face by face in the format's order (F0, F1, F2, F3, N, S), each face row by
//! row: cell (face, i, j) is index `face · n² + j · n + i`, i east and j south in the face's own
//! frame (the net's axes, `deeptime_format::cube`). This is the order the format's cube rasters
//! use, so a field is written as it is held.
//!
//! # Topology
//! [`CubeTopology`] holds every cell's eight neighbours in the D8 order of
//! [`crate::surface::grid::OFFSETS`] (E, NE, N, NW, W, SW, S, SE in the cell's own face frame;
//! "north" is j − 1, the face's up), found through the format's [`cell_on`]. Across a face edge the
//! neighbour lies on the next face; the diagonal step across a cube corner (two sides at once) has
//! no cell, so the 24 corner cells have seven neighbours and every other cell eight. Neighbours
//! are mutual: m is a neighbour of c exactly when c is one of m's.
//!
//! # Metric
//! [`CubeGrid`] adds the metric of a sphere of radius R. Every quantity comes from points of the
//! map, never from its Jacobian (which the table does not give reliably in the corner cells):
//! - the cell's centre, the map's image of the local centre ((2i + 1 − n)/n, (2j + 1 − n)/n);
//! - its corners, the images of ((2I − n)/n, (2J − n)/n), I, J ∈ [0, n] (the integer numerators
//!   keep the coordinates exactly antisymmetric, so a face's symmetries hold bit for bit);
//! - its area, R² times the solid angle of the geodesic quadrilateral of its corners, as two
//!   triangles by Van Oosterom & Strackee 1983 (the faces tile the sphere, so the areas sum to 4πR²
//!   to round-off);
//! - the distance to each neighbour, the great-circle arc between the centres;
//! - latitude and longitude of the centre, and the bearing of the face's up (−v) direction,
//!   radians clockwise from north (0 on the four equatorial faces' centres);
//! - the Laplacian's couplings ([`CubeGrid::lap`]): linear finite elements on the cells' centres,
//!   each quadrilateral of four centres around a grid vertex cut along both diagonals and the two
//!   cuts averaged (the triangle of three centres at a cube corner taken whole), the cotangent
//!   weights of each triangle (the standard P1 stiffness; for example Meyer, Desbrun, Schröder &
//!   Barr 2003). On a grid of parallelograms the side weight is |b|²/|a × b| and the diagonal one
//!   −½ cot φ, the nine-point stencil of a skewed lattice; on squares, the five-point one.
//!
//! Every pair's distance and coupling is computed once, from the lower-indexed cell, so both ends
//! hold the same bits and a flux leaves one cell exactly as it enters the other.
//!
//! # Ghosts
//! [`GhostTable`] maps a face padded by g ghost rings to the cells that fill them: a ghost beyond
//! one side is the neighbour face's cell ([`cell_on`]); a ghost beyond two sides (the corner
//! squares, where three faces meet and no single neighbour holds it) takes the x-direction
//! neighbour's cell turned about the corner, FV3's `fill_corners` rule (Putman & Lin 2007, the
//! AGRID x-direction fill). The fill is a gather in a fixed order, so it is deterministic and a
//! function of the field alone.
//!
//! # Hilbert keys
//! [`hilbert_key`] orders cells along a Hilbert curve within each face (the face in the top bits),
//! for cache-friendly orders and the GPU's tiles (S8). Storage stays row-major.

use std::ops::Range;

use deeptime_format::cube::{Face, cell_on};
use deeptime_format::cube_map::CubeMap;
use deeptime_format::projection::{lat_lon_of_unit, local_frame};
use rayon::prelude::*;

use crate::surface::grid::{OFFSETS, SurfaceGrid};

/// The empty neighbour slot (the diagonal across a cube corner).
pub const NO_CELL: u32 = u32::MAX;

/// Every cell's eight neighbours on an n × n cube (module notes, "Topology").
#[derive(Clone, Debug, PartialEq)]
pub struct CubeTopology {
    n: usize,
    nbr: Vec<[u32; 8]>,
}

impl CubeTopology {
    /// The topology of n × n faces (n ≥ 2).
    pub fn new(n: usize) -> CubeTopology {
        assert!(n >= 2, "a cube face needs at least 2 × 2 cells, not {n}");
        assert!(6 * n * n < NO_CELL as usize, "cube of side {n} too large");
        let nn = n * n;
        let mut nbr = vec![[NO_CELL; 8]; 6 * nn];
        nbr.par_iter_mut().enumerate().for_each(|(c, row)| {
            let face = Face::ALL[c / nn];
            let (i, j) = ((c % nn % n) as i64, (c % nn / n) as i64);
            for (k, &(di, dj, _)) in OFFSETS.iter().enumerate() {
                if let Some((f, a, b, _)) = cell_on(face, n as i64, i + di, j + dj) {
                    row[k] = (f.index() * nn + b as usize * n + a as usize) as u32;
                }
            }
        });
        CubeTopology { n, nbr }
    }

    /// Cells per face side.
    #[inline]
    pub fn n(&self) -> usize {
        self.n
    }

    /// 6n².
    #[inline]
    pub fn cells(&self) -> usize {
        self.nbr.len()
    }

    /// The index of cell (i, j) of `face`.
    #[inline]
    pub fn index(&self, face: Face, i: usize, j: usize) -> usize {
        face.index() * self.n * self.n + j * self.n + i
    }

    /// The face and cell of index `c`.
    #[inline]
    pub fn coords(&self, c: usize) -> (Face, usize, usize) {
        let nn = self.n * self.n;
        (Face::ALL[c / nn], c % nn % self.n, c % nn / self.n)
    }

    /// The eight neighbour slots of `c` ([`NO_CELL`] where there is none).
    #[inline]
    pub fn slots(&self, c: usize) -> &[u32; 8] {
        &self.nbr[c]
    }

    /// The slot of `c`'s table that holds `m`, if any.
    #[inline]
    pub fn slot_of(&self, c: usize, m: usize) -> Option<usize> {
        self.nbr[c].iter().position(|&x| x as usize == m)
    }
}

/// The cube's cells with the metric of a sphere (module notes, "Metric").
#[derive(Clone, Debug)]
pub struct CubeGrid {
    topo: CubeTopology,
    radius_m: f64,
    unit: Vec<[f64; 3]>,
    lat_deg: Vec<f64>,
    lon_deg: Vec<f64>,
    bearing: Vec<f64>,
    area: Vec<f64>,
    dist: Vec<[f64; 8]>,
    lap: Vec<[f64; 8]>,
    total_area: f64,
    mean_area: f64,
}

/// The unit vector of a face's local point, the cube frame.
#[inline]
fn point(map: &CubeMap, face: Face, n: usize, a2: i64, b2: i64) -> [f64; 3] {
    // a2, b2: twice the local coordinate times n (an integer), so (a2 / n) is exactly antisymmetric.
    let nf = n as f64;
    map.forward(face, a2 as f64 / nf, b2 as f64 / nf)
}

#[inline]
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

/// Solid angle of the geodesic triangle of unit vectors a, b, c (Van Oosterom & Strackee 1983:
/// tan(Ω/2) = |a·(b×c)| / (1 + a·b + b·c + c·a)).
#[inline]
fn solid_angle(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let num = dot(a, cross(b, c)).abs();
    let den = 1.0 + dot(a, b) + dot(b, c) + dot(c, a);
    2.0 * num.atan2(den)
}

/// The cotangent of the angle at `k` of the triangle (i, j, k).
#[inline]
fn cot_at(pi: [f64; 3], pj: [f64; 3], pk: [f64; 3]) -> f64 {
    let (e1, e2) = (sub(pi, pk), sub(pj, pk));
    dot(e1, e2) / norm(cross(e1, e2))
}

/// The great-circle angle between two unit vectors (from the chord, which is the same bits taken
/// either way round).
#[inline]
fn arc(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = sub(a, b);
    2.0 * (0.5 * dot(d, d).sqrt()).min(1.0).asin()
}

impl CubeGrid {
    /// The grid of n × n faces on the simulation's map, on a sphere of `radius_m`.
    pub fn new(n: usize, radius_m: f64) -> CubeGrid {
        CubeGrid::with_map(&CubeMap::simulation(), n, radius_m)
    }

    /// The grid on another map of the slot (the tests use the world's conformal one too).
    pub fn with_map(map: &CubeMap, n: usize, radius_m: f64) -> CubeGrid {
        let topo = CubeTopology::new(n);
        let nn = n * n;
        let cells = 6 * nn;
        let ni = n as i64;
        let unit: Vec<[f64; 3]> = (0..cells)
            .into_par_iter()
            .map(|c| {
                let (face, i, j) = topo.coords(c);
                point(map, face, n, 2 * i as i64 + 1 - ni, 2 * j as i64 + 1 - ni)
            })
            .collect();
        // The corners of each face, (n + 1)² points.
        let corners: Vec<Vec<[f64; 3]>> = Face::ALL
            .par_iter()
            .map(|&face| {
                (0..(n + 1) * (n + 1))
                    .map(|q| {
                        let (a, b) = ((q % (n + 1)) as i64, (q / (n + 1)) as i64);
                        point(map, face, n, 2 * a - ni, 2 * b - ni)
                    })
                    .collect()
            })
            .collect();
        let corner = |face: Face, a: usize, b: usize| corners[face.index()][b * (n + 1) + a];
        let r2 = radius_m * radius_m;
        let mut area = vec![0.0; cells];
        let mut bearing = vec![0.0; cells];
        let mut lat_deg = vec![0.0; cells];
        let mut lon_deg = vec![0.0; cells];
        area.par_iter_mut()
            .zip(bearing.par_iter_mut())
            .zip(lat_deg.par_iter_mut().zip(lon_deg.par_iter_mut()))
            .enumerate()
            .for_each(|(c, ((ar, br), (la, lo)))| {
                let (face, i, j) = topo.coords(c);
                let (q00, q10) = (corner(face, i, j), corner(face, i + 1, j));
                let (q01, q11) = (corner(face, i, j + 1), corner(face, i + 1, j + 1));
                *ar = r2 * (solid_angle(q00, q10, q11) + solid_angle(q00, q11, q01));
                let p = unit[c];
                let (lat, lon) = lat_lon_of_unit(p);
                *la = lat;
                *lo = lon;
                // The face's up: the top edge's midpoint minus the bottom's.
                let up = sub(
                    [q00[0] + q10[0], q00[1] + q10[1], q00[2] + q10[2]],
                    [q01[0] + q11[0], q01[1] + q11[1], q01[2] + q11[2]],
                );
                let (east, north) = local_frame(p);
                *br = dot(up, east).atan2(dot(up, north));
            });
        // Distances and couplings: computed from each cell, then every pair given the value its
        // lower-indexed cell computed.
        let raw: Vec<([f64; 8], [f64; 8])> = (0..cells)
            .into_par_iter()
            .map(|c| {
                let s = topo.slots(c);
                let p = |k: usize| unit[s[k] as usize];
                let mut dist = [0.0; 8];
                for k in 0..8 {
                    if s[k] != NO_CELL {
                        dist[k] = radius_m * arc(unit[c], p(k));
                    }
                }
                let mut lap = [0.0; 8];
                let pc = unit[c];
                for k in [1usize, 3, 5, 7] {
                    let (ka, kb) = (k - 1, (k + 1) % 8);
                    let (pa, pb) = (p(ka), p(kb));
                    if s[k] == NO_CELL {
                        // The triangle of three centres at a cube corner.
                        lap[ka] += 0.5 * cot_at(pc, pa, pb);
                        lap[kb] += 0.5 * cot_at(pc, pb, pa);
                    } else {
                        let pm = p(k);
                        // Both cuts of the quadrilateral (c, A, M, B), averaged.
                        lap[ka] += 0.25 * (cot_at(pc, pa, pm) + cot_at(pc, pa, pb));
                        lap[kb] += 0.25 * (cot_at(pc, pb, pm) + cot_at(pc, pb, pa));
                        lap[k] = 0.25 * (cot_at(pc, pm, pa) + cot_at(pc, pm, pb));
                    }
                }
                (dist, lap)
            })
            .collect();
        let (dist, lap): (Vec<[f64; 8]>, Vec<[f64; 8]>) = (0..cells)
            .into_par_iter()
            .map(|c| {
                let s = topo.slots(c);
                let (mut d, mut l) = raw[c];
                for k in 0..8 {
                    let m = s[k] as usize;
                    if s[k] != NO_CELL && m < c {
                        let km = topo.slot_of(m, c).expect("neighbours are mutual");
                        d[k] = raw[m].0[km];
                        l[k] = raw[m].1[km];
                    }
                }
                (d, l)
            })
            .unzip();
        // Per face row, rows in order, faces in order.
        let total_area = face_sum(n, &area);
        CubeGrid {
            topo,
            radius_m,
            unit,
            lat_deg,
            lon_deg,
            bearing,
            area,
            dist,
            lap,
            total_area,
            mean_area: total_area / cells as f64,
        }
    }

    /// The grid's topology.
    #[inline]
    pub fn topology(&self) -> &CubeTopology {
        &self.topo
    }

    /// Cells per face side.
    #[inline]
    pub fn n(&self) -> usize {
        self.topo.n
    }

    /// The sphere's radius, m.
    #[inline]
    pub fn radius_m(&self) -> f64 {
        self.radius_m
    }

    /// The centre's unit vector, cube frame.
    #[inline]
    pub fn unit(&self, c: usize) -> [f64; 3] {
        self.unit[c]
    }

    /// The centre's latitude, degrees.
    #[inline]
    pub fn lat_deg(&self, c: usize) -> f64 {
        self.lat_deg[c]
    }

    /// The centre's longitude, degrees.
    #[inline]
    pub fn lon_deg(&self, c: usize) -> f64 {
        self.lon_deg[c]
    }

    /// The bearing of the face's up (−v) at the centre, radians clockwise from north.
    #[inline]
    pub fn bearing(&self, c: usize) -> f64 {
        self.bearing[c]
    }

    /// Every cell's area, m².
    #[inline]
    pub fn areas(&self) -> &[f64] {
        &self.area
    }

    /// The great-circle distances to the eight neighbour slots, m (0 in an empty slot).
    #[inline]
    pub fn dists(&self, c: usize) -> &[f64; 8] {
        &self.dist[c]
    }

    /// The Laplacian's dimensionless couplings to the eight slots (module notes): a diffusivity
    /// K gives the volume flux K · w · (h_m − h_c) into c.
    #[inline]
    pub fn lap(&self, c: usize) -> &[f64; 8] {
        &self.lap[c]
    }

    /// The sum of the areas (per face row, rows then faces in order), m².
    #[inline]
    pub fn total_area(&self) -> f64 {
        self.total_area
    }

    /// The mean cell area, m².
    #[inline]
    pub fn mean_area(&self) -> f64 {
        self.mean_area
    }
}

/// The sum of a per-cell field in the grid's fixed order: per face row in ascending column order,
/// the rows of a face in order, then the six faces in the format's order (F0, F1, F2, F3, N, S).
/// The order the cube's reductions keep, so a sum does not depend on the thread count.
pub fn face_sum(n: usize, v: &[f64]) -> f64 {
    let rows: Vec<f64> = v.par_chunks(n).map(|r| r.iter().sum::<f64>()).collect();
    let mut total = 0.0;
    for face in rows.chunks(n) {
        let mut f = 0.0;
        for &r in face {
            f += r;
        }
        total += f;
    }
    total
}

impl SurfaceGrid for CubeGrid {
    #[inline]
    fn cells(&self) -> usize {
        self.topo.cells()
    }

    #[inline]
    fn neighbours(&self, c: usize) -> impl Iterator<Item = (usize, f64)> + '_ {
        let s = &self.topo.nbr[c];
        let d = &self.dist[c];
        (0..8).filter_map(move |k| (s[k] != NO_CELL).then(|| (s[k] as usize, d[k])))
    }

    #[inline]
    fn area(&self, c: usize) -> f64 {
        self.area[c]
    }

    #[inline]
    fn ref_area(&self) -> f64 {
        self.mean_area
    }

    #[inline]
    fn uniform_area(&self) -> Option<f64> {
        None
    }

    fn strips(&self, rows: usize) -> Vec<Range<usize>> {
        let n = self.topo.n;
        let rows = rows.max(1);
        let mut out = Vec::new();
        for f in 0..6 {
            for j0 in (0..n).step_by(rows) {
                let base = f * n * n;
                out.push(base + j0 * n..base + (j0 + rows).min(n) * n);
            }
        }
        out
    }

    fn strip_border(&self, s: Range<usize>, out: &mut Vec<usize>) {
        // A strip is whole rows of one face: its first and last rows, and the first and last
        // column of the rows between (across the face's east and west edges).
        let n = self.topo.n;
        out.clear();
        let (r0, r1) = (s.start / n, s.end / n);
        for r in r0..r1 {
            if r == r0 || r + 1 == r1 {
                out.extend(r * n..(r + 1) * n);
            } else {
                out.push(r * n);
                out.push(r * n + n - 1);
            }
        }
    }

    #[inline]
    fn neighbours_in(&self, c: usize, lo: usize, hi: usize, out: &mut [usize; 8]) -> usize {
        let mut k = 0;
        for &m in &self.topo.nbr[c] {
            let m = m as usize;
            if m >= lo && m < hi {
                out[k] = m;
                k += 1;
            }
        }
        k
    }
}

/// A face padded by `g` ghost rings: for each padded slot (i + g, j + g), i, j ∈ [−g, n + g), the
/// cell whose value fills it (module notes, "Ghosts").
#[derive(Clone, Debug, PartialEq)]
pub struct GhostTable {
    n: usize,
    g: usize,
    side: usize,
    src: Vec<u32>,
}

impl GhostTable {
    /// The table for g rings (1 ≤ g ≤ n).
    pub fn new(topo: &CubeTopology, g: usize) -> GhostTable {
        let n = topo.n;
        assert!(g >= 1 && g <= n, "ghost rings {g} must be in 1..={n}");
        let side = n + 2 * g;
        let ni = n as i64;
        let mut src = vec![NO_CELL; 6 * side * side];
        src.par_chunks_mut(side * side)
            .enumerate()
            .for_each(|(f, out)| {
                let face = Face::ALL[f];
                for (q, slot) in out.iter_mut().enumerate() {
                    let (i, j) = ((q % side) as i64 - g as i64, (q / side) as i64 - g as i64);
                    let (si, sj) = corner_source(ni, i, j);
                    let (sf, a, b, _) = cell_on(face, ni, si, sj)
                        .expect("a ghost's source is one side out at most");
                    *slot = topo.index(sf, a as usize, b as usize) as u32;
                }
            });
        GhostTable { n, g, side, src }
    }

    /// Ghost rings.
    #[inline]
    pub fn rings(&self) -> usize {
        self.g
    }

    /// The padded side n + 2g.
    #[inline]
    pub fn side(&self) -> usize {
        self.side
    }

    /// The cell that fills face `face`'s slot (i, j), i, j ∈ [−g, n + g).
    #[inline]
    pub fn source(&self, face: Face, i: i64, j: i64) -> usize {
        let g = self.g as i64;
        let q = ((j + g) as usize) * self.side + (i + g) as usize;
        self.src[face.index() * self.side * self.side + q] as usize
    }

    /// Fills `out` (6 · side² values, face by face, each padded face row by row) from `field`
    /// (6n², the grid's order): a gather, the same for any thread count.
    pub fn fill<T: Copy + Send + Sync>(&self, field: &[T], out: &mut [T]) {
        assert_eq!(field.len(), 6 * self.n * self.n);
        assert_eq!(out.len(), self.src.len());
        out.par_iter_mut()
            .zip(self.src.par_iter())
            .for_each(|(o, &s)| *o = field[s as usize]);
    }
}

/// Where the corner ghost (i, j) beyond two sides takes its value: the x-direction ghost turned
/// about the corner (FV3 `fill_corners`, AGRID, x direction; a, b ≥ 1 the depths in i and j). A
/// cell inside the face or one side out is its own source.
#[inline]
fn corner_source(n: i64, i: i64, j: i64) -> (i64, i64) {
    let lo_i = i < 0;
    let hi_i = i >= n;
    let lo_j = j < 0;
    let hi_j = j >= n;
    if lo_i && lo_j {
        let (a, b) = (-i, -j);
        (-b, a - 1)
    } else if lo_i && hi_j {
        let (a, b) = (-i, j - n + 1);
        (-b, n - a)
    } else if hi_i && lo_j {
        let (a, b) = (i - n + 1, -j);
        (n - 1 + b, a - 1)
    } else if hi_i && hi_j {
        let (a, b) = (i - n + 1, j - n + 1);
        (n - 1 + b, n - a)
    } else {
        (i, j)
    }
}

/// The Hilbert key of cell (i, j) of `face` on n × n faces: the face in the top bits, then the
/// cell's distance along the Hilbert curve of the face's square of side 2^⌈log₂ n⌉ (the curve's
/// standard construction: per bit level, the quadrant, then the turn that keeps the curve whole).
pub fn hilbert_key(n: usize, face: Face, i: usize, j: usize) -> u64 {
    let side = n.next_power_of_two() as u64;
    let (mut x, mut y) = (i as u64, j as u64);
    let mut d = 0u64;
    let mut s = side / 2;
    while s > 0 {
        let rx = u64::from(x & s > 0);
        let ry = u64::from(y & s > 0);
        d += s * s * ((3 * rx) ^ ry);
        if ry == 0 {
            if rx == 1 {
                x = side - 1 - x;
                y = side - 1 - y;
            }
            std::mem::swap(&mut x, &mut y);
        }
        s /= 2;
    }
    ((face.index() as u64) << 60) | d
}

/// The cells of n × n faces in Hilbert order (each face's cells along its curve, faces in order).
pub fn hilbert_order(n: usize) -> Vec<u32> {
    let topo_index = |f: usize, i: usize, j: usize| (f * n * n + j * n + i) as u32;
    let mut keyed: Vec<(u64, u32)> = (0..6 * n * n)
        .map(|c| {
            let (f, r) = (c / (n * n), c % (n * n));
            let (i, j) = (r % n, r / n);
            (hilbert_key(n, Face::ALL[f], i, j), topo_index(f, i, j))
        })
        .collect();
    keyed.sort_unstable();
    keyed.into_iter().map(|(_, c)| c).collect()
}

#[cfg(test)]
mod tests;

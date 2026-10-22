// SPDX-License-Identifier: LGPL-3.0-or-later
//! The cube world's geometry: six square faces of side s in a cross-shaped net
//! (docs/plans/cube-world-plan.md §1.2, work item W2; Ben, decisions log 2026-10-22: "Fuck it,
//! remove equal are restriction and move to a cube"). Pure integer and plane geometry: which face
//! a block is on, how the net's edges are glued, the corners, and the face adjacency that rasters
//! and solvers use. The map from a face to the sphere is [`crate::cube_map`]; the Java twin is
//! `worlddata/CubeLayout.java`, the pages' is `DTCube` in `deeptime-viewer/assets/projection.js`.
//!
//! # The net
//! +x is east and −z north, as everywhere in Minecraft. The window is x ∈ [−3s/2, 5s/2),
//! z ∈ [−3s/2, 3s/2); x wraps with period 4s (Toroidal World's slide).
//!
//! | Face | x range | z range | centre | centre in the cube frame |
//! |---|---|---|---|---|
//! | F3 | [−3s/2, −s/2) | [−s/2, s/2) | (−s, 0) | −Y |
//! | F0 | [−s/2, s/2) | [−s/2, s/2) | (0, 0) | +X |
//! | F1 | [s/2, 3s/2) | [−s/2, s/2) | (s, 0) | +Y |
//! | F2 | [3s/2, 5s/2) | [−s/2, s/2) | (2s, 0) | −X |
//! | N | [−s/2, s/2) | [−3s/2, −s/2) | (0, −s) | +Z |
//! | S | [−s/2, s/2) | [s/2, 3s/2) | (0, s) | −Z |
//!
//! N and S name the net's top and bottom faces, not the poles (see "Cube frame").
//!
//! # Local coordinates
//! Every face is drawn in the net without a turn, so its local axes are the net's: block cell
//! (i, j) ∈ [0, s)² is the net block (x0 + i, z0 + j) from the face's origin (its north-west
//! corner, [`Cube::face_origin`]), and the continuous local point (u, v) ∈ [−1, 1]² is
//! ((x − cx)/(s/2), (z − cz)/(s/2)) from its centre. u runs east in the net, v south.
//!
//! # Seams
//! Five cube edges are joined in the net (F3|F0, F0|F1, F1|F2, N|F0, S|F0). The other seven are
//! cut into pairs of net edges glued by a motion of the block grid ([`Iso`], the wrap engine's
//! `CellIsometry`): the east-west slide (F2 east and F3 west, 4s), four quarter turns about F0's
//! corners (N and S against F1 and F3) and two half turns (N's top and S's bottom against F2's).
//! [`Cube::edges`] lists the 14 net edges; [`Cube::fold`] takes any block to the planet block it
//! shows.
//!
//! # Cube frame
//! The faces' frames in the cube frame are [`Face::frame`]. Every frame has e_u × e_v = −centre,
//! so the turns between faces are rotations, never reflections. The cube frame is not the
//! planet's: a [`CubeTurn`] (the manifest's `projection.rotation`, 3 × 3, row-major, planet to
//! cube) places the cube on the planet. The world's turn is free (the corner search,
//! docs/research/notes/mountains-and-the-projection.md §5.4): its poles sit anywhere, usually
//! inside N and S but not at their centres, so nothing here knows latitude. Latitude comes from
//! the map ([`crate::cube_map`]) and the turn together ([`crate::cube_map::CubeWorld`]). The
//! identity turn is the polar-axis cube at λ0 = 0 (the simulation's cube): +X the planet point
//! (0°, 0°), +Z the north pole.

use crate::projection::{Axis, Domain, FACE_QUANTUM, Side, face_for};

/// A face of the cube, in the format's order (rasters are stored face by face in this order).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Face {
    F0 = 0,
    F1 = 1,
    F2 = 2,
    F3 = 3,
    N = 4,
    S = 5,
}

impl Face {
    /// The six faces in the format's order.
    pub const ALL: [Face; 6] = [Face::F0, Face::F1, Face::F2, Face::F3, Face::N, Face::S];

    /// The face's index in the format's order.
    #[inline]
    pub fn index(self) -> usize {
        self as usize
    }

    /// The face of index `i` in the format's order.
    pub fn from_index(i: usize) -> Option<Face> {
        Face::ALL.get(i).copied()
    }

    /// `F0` .. `F3`, `N`, `S`.
    pub fn name(self) -> &'static str {
        ["F0", "F1", "F2", "F3", "N", "S"][self.index()]
    }

    /// The face's centre in the net, in units of s.
    pub fn centre_units(self) -> (i64, i64) {
        [(0, 0), (1, 0), (2, 0), (-1, 0), (0, -1), (0, 1)][self.index()]
    }

    /// The face's frame in the cube frame: the unit vector of its centre, then e_u and e_v, the
    /// cube-frame directions of local +u (net +x) and +v (net +z) at the centre.
    pub const fn frame(self) -> [[f64; 3]; 3] {
        const X: [f64; 3] = [1.0, 0.0, 0.0];
        const Y: [f64; 3] = [0.0, 1.0, 0.0];
        const Z: [f64; 3] = [0.0, 0.0, 1.0];
        const MX: [f64; 3] = [-1.0, 0.0, 0.0];
        const MY: [f64; 3] = [0.0, -1.0, 0.0];
        const MZ: [f64; 3] = [0.0, 0.0, -1.0];
        match self {
            Face::F0 => [X, Y, MZ],
            Face::F1 => [Y, MX, MZ],
            Face::F2 => [MX, MY, MZ],
            Face::F3 => [MY, X, MZ],
            Face::N => [Z, Y, X],
            Face::S => [MZ, Y, MX],
        }
    }

    /// The face whose centre is the largest component of the cube-frame vector `p` (any length
    /// > 0): ties go to x, then y, then z, and a zero component counts as positive.
    pub fn of_vector(p: [f64; 3]) -> Face {
        let (ax, ay, az) = (p[0].abs(), p[1].abs(), p[2].abs());
        if ax >= ay && ax >= az {
            if p[0] >= 0.0 { Face::F0 } else { Face::F2 }
        } else if ay >= az {
            if p[1] >= 0.0 { Face::F1 } else { Face::F3 }
        } else if p[2] >= 0.0 {
            Face::N
        } else {
            Face::S
        }
    }

    /// What lies across side `side` of this face ([`ADJACENCY`]).
    #[inline]
    pub fn across(self, side: FaceSide) -> Across {
        ADJACENCY[self.index()][side.index()]
    }
}

/// A side of a face in its own local frame: east is local +u (i = n), south +v (j = n). The order
/// is clockwise on the map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum FaceSide {
    East = 0,
    South = 1,
    West = 2,
    North = 3,
}

impl FaceSide {
    pub const ALL: [FaceSide; 4] = [
        FaceSide::East,
        FaceSide::South,
        FaceSide::West,
        FaceSide::North,
    ];

    #[inline]
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        ["east", "south", "west", "north"][self.index()]
    }
}

/// A motion of the block grid that keeps blocks on blocks: `k` clockwise quarter turns (seen from
/// above with north up, Minecraft's sense) about the origin, then the translation (tx, tz). The
/// wrap engine's `CellIsometry`, the format's `iso` triple `[k, tx, tz]`. On block cells (the
/// block (x, z) spans [x, x+1) × [z, z+1)) the turns are k = 1: (x, z) → (−z − 1, x), k = 2:
/// (−x − 1, −z − 1), k = 3: (z, −x − 1); on points k = 1: (−z, x), k = 2: (−x, −z), k = 3: (z, −x).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Iso {
    /// Clockwise quarter turns, 0..=3.
    pub k: u8,
    pub tx: i64,
    pub tz: i64,
}

impl Iso {
    pub const IDENTITY: Iso = Iso { k: 0, tx: 0, tz: 0 };

    /// The motion `[k, tx, tz]` (k taken mod 4).
    pub fn new(k: i64, tx: i64, tz: i64) -> Iso {
        Iso {
            k: k.rem_euclid(4) as u8,
            tx,
            tz,
        }
    }

    pub fn translation(dx: i64, dz: i64) -> Iso {
        Iso::new(0, dx, dz)
    }

    /// The half turn about the block corner (a, b).
    pub fn half_turn_about(a: i64, b: i64) -> Iso {
        Iso::new(2, 2 * a, 2 * b)
    }

    /// The quarter turn (k = 1 clockwise, 3 counter-clockwise) about the block corner (a, b).
    pub fn quarter_turn_about(k: i64, a: i64, b: i64) -> Iso {
        let (rx, rz) = turn_i(k.rem_euclid(4) as u8, a, b);
        Iso::new(k, a - rx, b - rz)
    }

    /// The block cell (x, z) moved.
    #[inline]
    pub fn cell(&self, x: i64, z: i64) -> (i64, i64) {
        let (rx, rz) = match self.k {
            1 => (-z - 1, x),
            2 => (-x - 1, -z - 1),
            3 => (z, -x - 1),
            _ => (x, z),
        };
        (rx + self.tx, rz + self.tz)
    }

    /// The point (x, z) moved.
    #[inline]
    pub fn point(&self, x: f64, z: f64) -> (f64, f64) {
        let (rx, rz) = self.vector(x, z);
        (rx + self.tx as f64, rz + self.tz as f64)
    }

    /// A direction (dx, dz) turned (no translation).
    #[inline]
    pub fn vector(&self, dx: f64, dz: f64) -> (f64, f64) {
        match self.k {
            1 => (-dz, dx),
            2 => (-dx, -dz),
            3 => (dz, -dx),
            _ => (dx, dz),
        }
    }

    /// `self ∘ other`: first `other`, then `self`.
    pub fn compose(&self, other: &Iso) -> Iso {
        let (rx, rz) = turn_i(self.k, other.tx, other.tz);
        Iso::new(
            i64::from(self.k) + i64::from(other.k),
            rx + self.tx,
            rz + self.tz,
        )
    }

    /// The inverse motion.
    pub fn inverse(&self) -> Iso {
        let k = (4 - self.k) % 4;
        let (rx, rz) = turn_i(k, -self.tx, -self.tz);
        Iso::new(i64::from(k), rx, rz)
    }

    /// `[k, tx, tz]`, the format's `iso`.
    pub fn as_array(&self) -> [i64; 3] {
        [i64::from(self.k), self.tx, self.tz]
    }
}

/// The point turn of (x, z) by k clockwise quarter turns, on integers.
#[inline]
fn turn_i(k: u8, x: i64, z: i64) -> (i64, i64) {
    match k {
        1 => (-z, x),
        2 => (-x, -z),
        3 => (z, -x),
        _ => (x, z),
    }
}

/// What lies across one side of a face: the neighbour face, the side of it the step enters
/// through, and the motion of local cells. For an n × n grid on every face, the cell (i, j) of
/// this face that lies one step beyond the side (one of i, j in {−1, n}, or further out up to a
/// face width, the other in [0, n)) is the neighbour's cell
/// `(i', j') = R_k(i, j) + n · shift`, R_k the clockwise cell turn of [`Iso::cell`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Across {
    pub face: Face,
    pub side: FaceSide,
    /// Clockwise quarter turns from this face's local frame to the neighbour's.
    pub quarter_turns: u8,
    /// The translation in units of the grid's n.
    pub shift: [i8; 2],
}

impl Across {
    /// The neighbour's cell of this face's cell (i, j) of an n × n face grid.
    #[inline]
    pub fn cell(&self, n: i64, i: i64, j: i64) -> (i64, i64) {
        Iso::new(
            i64::from(self.quarter_turns),
            n * i64::from(self.shift[0]),
            n * i64::from(self.shift[1]),
        )
        .cell(i, j)
    }

    /// The neighbour's local point of this face's local point (u, v) (continuous, [−1, 1]² on
    /// each face, extended across the side).
    #[inline]
    pub fn local(&self, u: f64, v: f64) -> (f64, f64) {
        // On cells from the corner, p = (u + 1)/2 · n; p' = R p + n · shift.
        let r = Iso::new(i64::from(self.quarter_turns), 0, 0);
        let (pu, pv) = r.vector(u + 1.0, v + 1.0);
        (
            pu + 2.0 * f64::from(self.shift[0]) - 1.0,
            pv + 2.0 * f64::from(self.shift[1]) - 1.0,
        )
    }

    /// A direction (du, dv) in this face's local frame, in the neighbour's.
    #[inline]
    pub fn vector(&self, du: f64, dv: f64) -> (f64, f64) {
        Iso::new(i64::from(self.quarter_turns), 0, 0).vector(du, dv)
    }
}

const fn across(face: Face, side: FaceSide, quarter_turns: u8, shift: [i8; 2]) -> Across {
    Across {
        face,
        side,
        quarter_turns,
        shift,
    }
}

/// The face adjacency, `[face][side]` in the format's face order and [`FaceSide`] order (east,
/// south, west, north). Derived from the net's seams and checked against the fold and against the
/// cube frames (tests below).
pub const ADJACENCY: [[Across; 4]; 6] = {
    use Face::*;
    use FaceSide::{East as E, North as No, South as So, West as W};
    [
        // F0
        [
            across(F1, W, 0, [-1, 0]),
            across(S, No, 0, [0, -1]),
            across(F3, E, 0, [1, 0]),
            across(N, So, 0, [0, 1]),
        ],
        // F1
        [
            across(F2, W, 0, [-1, 0]),
            across(S, E, 1, [2, 0]),
            across(F0, E, 0, [1, 0]),
            across(N, E, 3, [1, 1]),
        ],
        // F2
        [
            across(F3, W, 0, [-1, 0]),
            across(S, So, 2, [1, 2]),
            across(F1, E, 0, [1, 0]),
            across(N, No, 2, [1, 0]),
        ],
        // F3
        [
            across(F0, W, 0, [-1, 0]),
            across(S, W, 3, [-1, 1]),
            across(F2, E, 0, [1, 0]),
            across(N, W, 1, [0, 0]),
        ],
        // N
        [
            across(F1, No, 1, [1, -1]),
            across(F0, No, 0, [0, -1]),
            across(F3, No, 3, [0, 0]),
            across(F2, No, 2, [1, 0]),
        ],
        // S
        [
            across(F1, So, 3, [0, 2]),
            across(F2, So, 2, [1, 2]),
            across(F3, So, 1, [1, 1]),
            across(F0, So, 0, [0, 1]),
        ],
    ]
};

/// The cell (i, j) of face `face`'s n × n grid, or the neighbour's cell it is when it lies beyond
/// one side (at most a face width out): `(face, i, j, quarter turns from face's frame)`. `None`
/// in the corner squares beyond two sides at once, where three faces meet at a cube corner and no
/// single neighbour holds the cell (a solver decides those itself).
pub fn cell_on(face: Face, n: i64, i: i64, j: i64) -> Option<(Face, i64, i64, u8)> {
    let out_i = i < 0 || i >= n;
    let out_j = j < 0 || j >= n;
    match (out_i, out_j) {
        (false, false) => Some((face, i, j, 0)),
        (true, true) => None,
        _ => {
            if i < -n || i >= 2 * n || j < -n || j >= 2 * n {
                return None;
            }
            let side = if i >= n {
                FaceSide::East
            } else if i < 0 {
                FaceSide::West
            } else if j >= n {
                FaceSide::South
            } else {
                FaceSide::North
            };
            let a = face.across(side);
            let (i2, j2) = a.cell(n, i, j);
            Some((a.face, i2, j2, a.quarter_turns))
        }
    }
}

/// Which pair of net edges a seam is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Seam {
    /// F2 east and F3 west, the slide (Toroidal World).
    EastWest,
    /// N east and F1 top, a quarter turn about F0's north-east corner.
    NorthEast,
    /// N west and F3 top, a quarter turn about F0's north-west corner.
    NorthWest,
    /// S east and F1 bottom, a quarter turn about F0's south-east corner.
    SouthEast,
    /// S west and F3 bottom, a quarter turn about F0's south-west corner.
    SouthWest,
    /// N top and F2 top, a half turn about (s, −s).
    NorthNorth,
    /// S bottom and F2 bottom, a half turn about (s, s).
    SouthSouth,
}

impl Seam {
    pub fn name(self) -> &'static str {
        match self {
            Seam::EastWest => "east_west",
            Seam::NorthEast => "ne",
            Seam::NorthWest => "nw",
            Seam::SouthEast => "se",
            Seam::SouthWest => "sw",
            Seam::NorthNorth => "nn",
            Seam::SouthSouth => "ss",
        }
    }
}

/// How a net edge is glued.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EdgeMotion {
    Slide,
    QuarterTurn,
    HalfTurn,
}

impl EdgeMotion {
    pub fn name(self) -> &'static str {
        match self {
            EdgeMotion::Slide => "slide",
            EdgeMotion::QuarterTurn => "quarter_turn",
            EdgeMotion::HalfTurn => "half_turn",
        }
    }
}

/// One of the 14 net edges that face the void or the window's ends, and the motion that brings
/// what lies beyond it back onto the planet (the wrap engine's `WrapEdge`, the 0.7 manifest's
/// `edges[]`).
#[derive(Clone, Debug, PartialEq)]
pub struct NetEdge {
    /// `F3-west`, `N-east`, ...: the face and its side.
    pub name: String,
    pub seam: Seam,
    pub motion: EdgeMotion,
    pub face: Face,
    pub face_side: FaceSide,
    /// The axis of the edge's normal (x for an east or west edge).
    pub axis: Axis,
    /// Which side of the line is beyond.
    pub beyond: Side,
    /// The edge line (x = coord or z = coord), a block boundary.
    pub coord: i64,
    /// [first, one past the last] block along the edge.
    pub span: [i64; 2],
    /// Beyond → planet.
    pub iso: Iso,
    /// The turn's centre (a block corner) for turning edges.
    pub pivot: Option<[i64; 2]>,
    /// The index in [`Cube::edges`] of the other net edge of the same seam.
    pub partner: usize,
    /// Who joins the edge in the game: `toroidal_world` (the slide) or `wrap_engine`.
    pub owner: &'static str,
}

impl NetEdge {
    /// How deep the block (x, z) lies beyond the edge, in its band (0 touching the line), or
    /// `None` when it is not beyond the edge within its span.
    pub fn depth(&self, x: i64, z: i64) -> Option<i64> {
        let (n, t) = match self.axis {
            Axis::X => (x, z),
            Axis::Z => (z, x),
        };
        if t < self.span[0] || t >= self.span[1] {
            return None;
        }
        let d = match self.beyond {
            Side::Min => self.coord - 1 - n,
            Side::Max => n - self.coord,
        };
        (d >= 0).then_some(d)
    }

    /// How far the point (x, z) lies beyond the edge line (0 on it), or `None` when it is on the
    /// planet's side or outside the span.
    pub fn depth_point(&self, x: f64, z: f64) -> Option<f64> {
        let (n, t) = match self.axis {
            Axis::X => (x, z),
            Axis::Z => (z, x),
        };
        if t < self.span[0] as f64 || t > self.span[1] as f64 {
            return None;
        }
        let d = match self.beyond {
            Side::Min => self.coord as f64 - n,
            Side::Max => n - self.coord as f64,
        };
        (d > 0.0).then_some(d)
    }
}

/// A cube corner: a cone point of 270° where three faces meet.
#[derive(Clone, Debug, PartialEq)]
pub struct Corner {
    /// `n0` .. `n3`, the corners of face N (cube +Z) counter-clockwise from (+1, +1, +1), then
    /// `s0` .. `s3`, face S's. Net names: where a corner lies on the planet is the turn's.
    pub name: String,
    /// The three faces that meet there.
    pub faces: [Face; 3],
    /// The corner's direction in the cube frame, (±1, ±1, ±1) unnormalised.
    pub cube: [i8; 3],
    /// Every block corner of the closed window that is this corner (the domain's closure:
    /// x ∈ [−3s/2, 5s/2], so the F2/F3 corners have a third image at x = 5s/2).
    pub images: Vec<[i64; 2]>,
}

/// Where [`Cube::fold`] takes a block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CubeFold {
    /// On the planet (after the slide): the face and the block's local cell in [0, s)².
    Face { face: Face, i: i64, j: i64 },
    /// Beyond a net edge, in its band: the edge (an index into [`Cube::edges`]), the planet block
    /// it shows (in the window) and the motion from the queried block to it (its `k` is the
    /// quarter turns a block state read there is turned back by).
    Outside {
        edge: usize,
        x: i64,
        z: i64,
        iso: Iso,
    },
    /// In no band, or more than a face width out: past the corners of the net (the missing
    /// quarter round an outer cube corner). The game draws deep ocean there (plan §1.4).
    Ocean,
}

/// A point brought onto the planet by [`Cube::fold_point`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointOnFace {
    pub face: Face,
    /// Local coordinates in [−1, 1]².
    pub u: f64,
    pub v: f64,
    /// The planet point in the window.
    pub x: f64,
    pub z: f64,
    /// The motion from the queried point (identity on the planet, the slide included).
    pub iso: Iso,
    /// The net edge crossed, if any.
    pub edge: Option<usize>,
}

/// The cube world's net for face side s (a multiple of [`FACE_QUANTUM`] in real worlds; any even
/// side in tests). Net geometry only: where the cube sits on the planet is a [`CubeTurn`].
#[derive(Clone, Debug, PartialEq)]
pub struct Cube {
    s: i64,
    edges: Vec<NetEdge>,
}

impl Cube {
    /// The net of face side `s` (even).
    pub fn new(s: u32) -> Cube {
        assert!(
            s >= 2 && s.is_multiple_of(2),
            "cube face side {s} must be even"
        );
        let s = i64::from(s);
        Cube {
            s,
            edges: net_edges(s),
        }
    }

    /// The net for a planet `circumference` blocks round ([`face_for`]).
    pub fn for_circumference(circumference: u32) -> Cube {
        Cube::new(face_for(circumference))
    }

    /// s, blocks.
    #[inline]
    pub fn side(&self) -> i64 {
        self.s
    }

    /// Whether s is a multiple of [`FACE_QUANTUM`] (a real world's face).
    pub fn quantised(&self) -> bool {
        self.s % i64::from(FACE_QUANTUM) == 0
    }

    /// The east-west period 4s, blocks (Toroidal World's loop).
    #[inline]
    pub fn period(&self) -> i64 {
        4 * self.s
    }

    /// The radius of the sphere the six faces stand for at equal area, blocks: 6s² = 4πR².
    pub fn radius(&self) -> f64 {
        self.s as f64 * (1.5 / std::f64::consts::PI).sqrt()
    }

    /// The window: x ∈ [−3s/2, 5s/2), z ∈ [−3s/2, 3s/2].
    pub fn window(&self) -> Domain {
        let h = self.s as f64 / 2.0;
        Domain {
            x_min: -3.0 * h,
            x_max: 5.0 * h,
            z_min: -3.0 * h,
            z_max: 3.0 * h,
        }
    }

    /// The net block of the face's local cell (0, 0) (its north-west corner).
    #[inline]
    pub fn face_origin(&self, face: Face) -> (i64, i64) {
        let (cx, cz) = face.centre_units();
        (cx * self.s - self.s / 2, cz * self.s - self.s / 2)
    }

    /// The face's centre, a block corner.
    #[inline]
    pub fn face_centre(&self, face: Face) -> (i64, i64) {
        let (cx, cz) = face.centre_units();
        (cx * self.s, cz * self.s)
    }

    /// The block x wrapped into the window [−3s/2, 5s/2).
    #[inline]
    pub fn wrap_x(&self, x: i64) -> i64 {
        let lo = -3 * self.s / 2;
        (x - lo).rem_euclid(4 * self.s) + lo
    }

    /// The point x wrapped into the window [−3s/2, 5s/2).
    #[inline]
    pub fn wrap_x_point(&self, x: f64) -> f64 {
        let p = (4 * self.s) as f64;
        let lo = -1.5 * self.s as f64;
        let t = (x - lo).rem_euclid(p);
        (if t >= p { 0.0 } else { t }) + lo
    }

    /// The face and local cell of the block (x, z) (x wraps), or `None` off the planet.
    pub fn face_of_block(&self, x: i64, z: i64) -> Option<(Face, i64, i64)> {
        let x = self.wrap_x(x);
        let h = self.s / 2;
        // Columns of s from x = −3s/2: 0 F3, 1 F0, 2 F1, 3 F2.
        let col = (x + 3 * h).div_euclid(self.s);
        let face = if (-h..h).contains(&z) {
            [Face::F3, Face::F0, Face::F1, Face::F2][col as usize]
        } else if col == 1 && (-3 * h..-h).contains(&z) {
            Face::N
        } else if col == 1 && (h..3 * h).contains(&z) {
            Face::S
        } else {
            return None;
        };
        let (ox, oz) = self.face_origin(face);
        Some((face, x - ox, z - oz))
    }

    /// The net block of a face's local cell (i, j) (in the window when (i, j) ∈ [0, s)²).
    #[inline]
    pub fn block_of_cell(&self, face: Face, i: i64, j: i64) -> (i64, i64) {
        let (ox, oz) = self.face_origin(face);
        (ox + i, oz + j)
    }

    /// The face and local point (u, v) ∈ [−1, 1]² of the point (x, z) (x wraps), or `None` off
    /// the planet. Faces are closed squares here: a point on a line two faces share is given to
    /// the first in the format's order (both faces map it to the same planet point).
    pub fn local_of_point(&self, x: f64, z: f64) -> Option<(Face, f64, f64)> {
        let x = self.wrap_x_point(x);
        let h = self.s as f64 / 2.0;
        for face in Face::ALL {
            let (cx, cz) = self.face_centre(face);
            let (u, v) = ((x - cx as f64) / h, (z - cz as f64) / h);
            if (-1.0..=1.0).contains(&u) && (-1.0..=1.0).contains(&v) {
                return Some((face, u, v));
            }
        }
        None
    }

    /// The net point of a face's local point (u, v).
    #[inline]
    pub fn point_of_local(&self, face: Face, u: f64, v: f64) -> (f64, f64) {
        let (cx, cz) = self.face_centre(face);
        let h = self.s as f64 / 2.0;
        (cx as f64 + u * h, cz as f64 + v * h)
    }

    /// The 14 net edges: the two slide edges, then the turning edges in the wrap engine's order
    /// (`WrapLayouts.cubeCross`): N-east, F1-top, N-west, F3-top, N-top, F2-top, S-east,
    /// F1-bottom, S-west, F3-bottom, S-bottom, F2-bottom.
    pub fn edges(&self) -> &[NetEdge] {
        &self.edges
    }

    /// The block (x, z) brought onto the planet ([`CubeFold`]). x wraps (the slide); a block off
    /// the planet takes the motion of the net edge whose band holds it, and where two bands meet
    /// (the void squares by F0's corners) the nearer edge's, ties to the earlier edge in
    /// [`Cube::edges`] (so the square splits on its diagonal through F0's corner, plan §1.4).
    /// Bands reach a face width out; a block in none is [`CubeFold::Ocean`].
    pub fn fold(&self, x: i64, z: i64) -> CubeFold {
        let xw = self.wrap_x(x);
        if let Some((face, i, j)) = self.face_of_block(xw, z) {
            return CubeFold::Face { face, i, j };
        }
        let mut best: Option<(usize, i64)> = None;
        for (k, e) in self.edges.iter().enumerate().skip(2) {
            if let Some(d) = e.depth(xw, z)
                && d < self.s
                && best.is_none_or(|(_, bd)| d < bd)
            {
                best = Some((k, d));
            }
        }
        let Some((k, _)) = best else {
            return CubeFold::Ocean;
        };
        let iso = self.edges[k].iso.compose(&Iso::translation(xw - x, 0));
        let (px, pz) = iso.cell(x, z);
        match self.face_of_block(px, pz) {
            Some(_) if px == self.wrap_x(px) => CubeFold::Outside {
                edge: k,
                x: px,
                z: pz,
                iso,
            },
            _ => CubeFold::Ocean,
        }
    }

    /// The point (x, z) brought onto the planet, as [`Cube::fold`] for blocks: x wraps; a point
    /// off the planet takes the motion of the nearest net edge whose band (a face width deep)
    /// holds it. `None` past the net's corners.
    pub fn fold_point(&self, x: f64, z: f64) -> Option<PointOnFace> {
        let xw = self.wrap_x_point(x);
        let slide = (xw - x).round() as i64;
        if let Some((face, u, v)) = self.local_of_point(xw, z) {
            return Some(PointOnFace {
                face,
                u,
                v,
                x: xw,
                z,
                iso: Iso::translation(slide, 0),
                edge: None,
            });
        }
        let mut best: Option<(usize, f64)> = None;
        for (k, e) in self.edges.iter().enumerate().skip(2) {
            if let Some(d) = e.depth_point(xw, z)
                && d <= self.s as f64
                && best.is_none_or(|(_, bd)| d < bd)
            {
                best = Some((k, d));
            }
        }
        let (k, _) = best?;
        let iso = self.edges[k].iso.compose(&Iso::translation(slide, 0));
        let (px, pz) = iso.point(x, z);
        let (face, u, v) = self.local_of_point(px, pz)?;
        Some(PointOnFace {
            face,
            u,
            v,
            x: self.wrap_x_point(px),
            z: pz,
            iso,
            edge: Some(k),
        })
    }

    /// The eight cube corners: `n0` .. `n3` round face N, then `s0` .. `s3` round face S, each
    /// with its cube-frame direction and its images in the net.
    pub fn corners(&self) -> Vec<Corner> {
        let h = self.s / 2;
        let c = |name: &str, faces: [Face; 3], cube: [i8; 3], images: Vec<[i64; 2]>| Corner {
            name: name.into(),
            faces,
            cube,
            images,
        };
        use Face::*;
        vec![
            c("n0", [F0, F1, N], [1, 1, 1], vec![[h, -h]]),
            c(
                "n1",
                [F1, F2, N],
                [-1, 1, 1],
                vec![[h, -3 * h], [3 * h, -h]],
            ),
            c(
                "n2",
                [F2, F3, N],
                [-1, -1, 1],
                vec![[-h, -3 * h], [-3 * h, -h], [5 * h, -h]],
            ),
            c("n3", [F3, F0, N], [1, -1, 1], vec![[-h, -h]]),
            c("s0", [F0, F1, S], [1, 1, -1], vec![[h, h]]),
            c("s1", [F1, F2, S], [-1, 1, -1], vec![[h, 3 * h], [3 * h, h]]),
            c(
                "s2",
                [F2, F3, S],
                [-1, -1, -1],
                vec![[-h, 3 * h], [-3 * h, h], [5 * h, h]],
            ),
            c("s3", [F3, F0, S], [1, -1, -1], vec![[-h, h]]),
        ]
    }
}

/// Where the cube sits on the planet: a rotation, planet frame to cube frame (the manifest's
/// `projection.rotation`, 3 × 3, row-major). The planet frame is the format's: +x the point
/// (0°, 0°), +y (0°, 90° E), +z the north pole ([`crate::projection::unit_of_lat_lon`]). The
/// world's turn is free (the corner search); the simulation's cube takes the identity (its poles
/// at the centres of N and S, λ0 = 0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubeTurn {
    m: [[f64; 3]; 3],
}

impl CubeTurn {
    /// The identity: the polar-axis cube.
    pub const IDENTITY: CubeTurn = CubeTurn {
        m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };

    /// A turn from its rows (planet to cube), checked to be a rotation: orthonormal to 1e-12 and
    /// of determinant +1.
    pub fn from_rows(m: [[f64; 3]; 3]) -> Result<CubeTurn, String> {
        for (i, a) in m.iter().enumerate() {
            for (j, b) in m.iter().enumerate() {
                let d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
                let want = if i == j { 1.0 } else { 0.0 };
                if (d - want).abs() > 1e-12 || !d.is_finite() {
                    return Err(format!("cube rotation rows {i}, {j}: dot {d}, want {want}"));
                }
            }
        }
        let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
        if det < 0.0 {
            return Err(format!("cube rotation has determinant {det}: a reflection"));
        }
        Ok(CubeTurn { m })
    }

    /// The polar-axis turn by λ0 (the old layout's): the planet point (0°, λ0) at F0's centre,
    /// the north pole at N's.
    pub fn polar(lambda0_deg: f64) -> CubeTurn {
        let l = crate::portable_math::to_radians(lambda0_deg);
        let (s, c) = (crate::portable_math::sin(l), crate::portable_math::cos(l));
        CubeTurn {
            m: [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// The rows, planet to cube.
    #[inline]
    pub fn rows(&self) -> [[f64; 3]; 3] {
        self.m
    }

    /// A planet-frame vector in the cube frame (Q·p).
    #[inline]
    pub fn to_cube(&self, p: [f64; 3]) -> [f64; 3] {
        let m = &self.m;
        [
            (m[0][0] * p[0] + m[0][1] * p[1]) + m[0][2] * p[2],
            (m[1][0] * p[0] + m[1][1] * p[1]) + m[1][2] * p[2],
            (m[2][0] * p[0] + m[2][1] * p[1]) + m[2][2] * p[2],
        ]
    }

    /// A cube-frame vector in the planet frame (Qᵀ·c).
    #[inline]
    pub fn to_planet(&self, c: [f64; 3]) -> [f64; 3] {
        let m = &self.m;
        [
            (m[0][0] * c[0] + m[1][0] * c[1]) + m[2][0] * c[2],
            (m[0][1] * c[0] + m[1][1] * c[1]) + m[2][1] * c[2],
            (m[0][2] * c[0] + m[1][2] * c[1]) + m[2][2] * c[2],
        ]
    }
}

/// The 14 net edges for face side s (see [`Cube::edges`]).
fn net_edges(s: i64) -> Vec<NetEdge> {
    let h = s / 2;
    use Face::*;
    use FaceSide::{East, North, South, West};
    #[allow(clippy::too_many_arguments)]
    let e = |face: Face,
             face_side: FaceSide,
             seam: Seam,
             motion: EdgeMotion,
             coord: i64,
             span: [i64; 2],
             iso: Iso,
             pivot: Option<[i64; 2]>,
             partner: usize| {
        let (axis, beyond) = match face_side {
            East => (Axis::X, Side::Max),
            West => (Axis::X, Side::Min),
            South => (Axis::Z, Side::Max),
            North => (Axis::Z, Side::Min),
        };
        let side_name = match (face, face_side) {
            (N | S | F1 | F2 | F3, North) => "top",
            (N | S | F1 | F2 | F3, South) => "bottom",
            _ => face_side.name(),
        };
        NetEdge {
            name: format!("{}-{side_name}", face.name()),
            seam,
            motion,
            face,
            face_side,
            axis,
            beyond,
            coord,
            span,
            iso,
            pivot,
            partner,
            owner: if motion == EdgeMotion::Slide {
                "toroidal_world"
            } else {
                "wrap_engine"
            },
        }
    };
    let q = EdgeMotion::QuarterTurn;
    let ht = EdgeMotion::HalfTurn;
    let half_n = Iso::half_turn_about(s, -s);
    let half_s = Iso::half_turn_about(s, s);
    vec![
        // 0, 1: the slide.
        e(
            F3,
            West,
            Seam::EastWest,
            EdgeMotion::Slide,
            -3 * h,
            [-h, h],
            Iso::translation(4 * s, 0),
            None,
            1,
        ),
        e(
            F2,
            East,
            Seam::EastWest,
            EdgeMotion::Slide,
            5 * h,
            [-h, h],
            Iso::translation(-4 * s, 0),
            None,
            0,
        ),
        // 2, 3: NE, a quarter turn about (s/2, −s/2), clockwise from N to F1.
        e(
            N,
            East,
            Seam::NorthEast,
            q,
            h,
            [-3 * h, -h],
            Iso::quarter_turn_about(1, h, -h),
            Some([h, -h]),
            3,
        ),
        e(
            F1,
            North,
            Seam::NorthEast,
            q,
            -h,
            [h, 3 * h],
            Iso::quarter_turn_about(3, h, -h),
            Some([h, -h]),
            2,
        ),
        // 4, 5: NW, about (−s/2, −s/2), counter-clockwise from N to F3.
        e(
            N,
            West,
            Seam::NorthWest,
            q,
            -h,
            [-3 * h, -h],
            Iso::quarter_turn_about(3, -h, -h),
            Some([-h, -h]),
            5,
        ),
        e(
            F3,
            North,
            Seam::NorthWest,
            q,
            -h,
            [-3 * h, -h],
            Iso::quarter_turn_about(1, -h, -h),
            Some([-h, -h]),
            4,
        ),
        // 6, 7: NN, the half turn about (s, −s).
        e(
            N,
            North,
            Seam::NorthNorth,
            ht,
            -3 * h,
            [-h, h],
            half_n,
            Some([s, -s]),
            7,
        ),
        e(
            F2,
            North,
            Seam::NorthNorth,
            ht,
            -h,
            [3 * h, 5 * h],
            half_n,
            Some([s, -s]),
            6,
        ),
        // 8, 9: SE, about (s/2, s/2), counter-clockwise from S to F1.
        e(
            S,
            East,
            Seam::SouthEast,
            q,
            h,
            [h, 3 * h],
            Iso::quarter_turn_about(3, h, h),
            Some([h, h]),
            9,
        ),
        e(
            F1,
            South,
            Seam::SouthEast,
            q,
            h,
            [h, 3 * h],
            Iso::quarter_turn_about(1, h, h),
            Some([h, h]),
            8,
        ),
        // 10, 11: SW, about (−s/2, s/2), clockwise from S to F3.
        e(
            S,
            West,
            Seam::SouthWest,
            q,
            -h,
            [h, 3 * h],
            Iso::quarter_turn_about(1, -h, h),
            Some([-h, h]),
            11,
        ),
        e(
            F3,
            South,
            Seam::SouthWest,
            q,
            h,
            [-3 * h, -h],
            Iso::quarter_turn_about(3, -h, h),
            Some([-h, h]),
            10,
        ),
        // 12, 13: SS, the half turn about (s, s).
        e(
            S,
            South,
            Seam::SouthSouth,
            ht,
            3 * h,
            [-h, h],
            half_s,
            Some([s, s]),
            13,
        ),
        e(
            F2,
            South,
            Seam::SouthSouth,
            ht,
            h,
            [3 * h, 5 * h],
            half_s,
            Some([s, s]),
            12,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIDES: [u32; 3] = [1_024, 3_840, 64];

    /// SplitMix64, as the parity fixture's.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
        fn range(&mut self, lo: i64, hi: i64) -> i64 {
            lo + (self.next() % (hi - lo) as u64) as i64
        }
    }

    #[test]
    fn iso_algebra_matches_the_wrap_engine() {
        // CellIsometry's examples: a half turn about (a, b) on cells is (2a − x − 1, 2b − z − 1).
        let h = Iso::half_turn_about(3, -5);
        assert_eq!(h.cell(1, 2), (2 * 3 - 1 - 1, -10 - 2 - 1));
        for k in 0..4 {
            for (a, b) in [(0, 0), (7, -3), (-128, 64)] {
                let q = Iso::quarter_turn_about(k, a, b);
                // The pivot is fixed as a point.
                assert_eq!(q.point(a as f64, b as f64), (a as f64, b as f64));
                let inv = q.inverse();
                for (x, z) in [(0, 0), (5, -9), (-200, 31)] {
                    let (x2, z2) = q.cell(x, z);
                    assert_eq!(inv.cell(x2, z2), (x, z));
                    assert_eq!(q.compose(&inv).cell(x, z), (x, z));
                }
            }
        }
    }

    #[test]
    fn faces_tile_the_net() {
        for s in SIDES {
            let cube = Cube::new(s);
            let s = i64::from(s);
            let mut count = [0i64; 6];
            let step = (s / 64).max(1);
            for z in (-3 * s / 2..3 * s / 2).step_by(step as usize) {
                for x in (-3 * s / 2..5 * s / 2).step_by(step as usize) {
                    if let Some((f, i, j)) = cube.face_of_block(x, z) {
                        assert!((0..s).contains(&i) && (0..s).contains(&j));
                        assert_eq!(cube.block_of_cell(f, i, j), (x, z));
                        count[f.index()] += 1;
                        // The slide: the same block a period away.
                        assert_eq!(cube.face_of_block(x + 4 * s, z), Some((f, i, j)));
                        assert_eq!(cube.face_of_block(x - 8 * s, z), Some((f, i, j)));
                    }
                }
            }
            let per = (s / step) * (s / step);
            assert_eq!(count, [per; 6], "s {s}");
        }
    }

    #[test]
    fn frames_are_right_handed_the_same_way() {
        for f in Face::ALL {
            let [c, eu, ev] = f.frame();
            let cross = [
                eu[1] * ev[2] - eu[2] * ev[1],
                eu[2] * ev[0] - eu[0] * ev[2],
                eu[0] * ev[1] - eu[1] * ev[0],
            ];
            assert_eq!(cross, [-c[0], -c[1], -c[2]], "{f:?}");
            assert_eq!(Face::of_vector(c), f);
        }
    }

    /// The cube-frame point of a gnomonic face point (A, B) (unnormalised).
    fn gnomonic(f: Face, a: f64, b: f64) -> [f64; 3] {
        let [c, eu, ev] = f.frame();
        [0, 1, 2].map(|k| c[k] + a * eu[k] + b * ev[k])
    }

    fn gnomonic_of(f: Face, p: [f64; 3]) -> (f64, f64) {
        let [c, eu, ev] = f.frame();
        let d = |q: [f64; 3]| p[0] * q[0] + p[1] * q[1] + p[2] * q[2];
        (d(eu) / d(c), d(ev) / d(c))
    }

    #[test]
    fn adjacency_matches_the_cube_frames() {
        // On an edge the map's local coordinate along the edge is the gnomonic one up to a fixed
        // odd function (the table's eightfold symmetry), so the gnomonic cube checks the edge
        // correspondence; the cell motion's direction across (into the neighbour) checks the rest.
        for f in Face::ALL {
            for side in FaceSide::ALL {
                let a = f.across(side);
                for t in [-0.75, -0.2, 0.0, 0.4, 1.0] {
                    let (u, v) = match side {
                        FaceSide::East => (1.0, t),
                        FaceSide::West => (-1.0, t),
                        FaceSide::South => (t, 1.0),
                        FaceSide::North => (t, -1.0),
                    };
                    let p = gnomonic(f, u, v);
                    let (u2, v2) = a.local(u, v);
                    let q = gnomonic(a.face, u2, v2);
                    for k in 0..3 {
                        assert!(
                            (p[k] - q[k]).abs() < 1e-12,
                            "{f:?} {side:?} t {t}: {p:?} vs {q:?}"
                        );
                    }
                    let (gu, gv) = gnomonic_of(a.face, p);
                    assert!((gu - u2).abs() < 1e-12 && (gv - v2).abs() < 1e-12);
                }
                // A step out of f lands just inside the neighbour, on the side it names.
                let (u, v) = match side {
                    FaceSide::East => (1.01, 0.3),
                    FaceSide::West => (-1.01, 0.3),
                    FaceSide::South => (0.3, 1.01),
                    FaceSide::North => (0.3, -1.01),
                };
                let (u2, v2) = a.local(u, v);
                assert!(
                    u2.abs() < 1.0 && v2.abs() < 1.0,
                    "{f:?} {side:?}: ({u2}, {v2})"
                );
                let near = match a.side {
                    FaceSide::East => u2 > 0.98,
                    FaceSide::West => u2 < -0.98,
                    FaceSide::South => v2 > 0.98,
                    FaceSide::North => v2 < -0.98,
                };
                assert!(near, "{f:?} {side:?} enters {:?} at ({u2}, {v2})", a.side);
                // And back.
                let back = a.face.across(a.side);
                assert_eq!(back.face, f);
                assert_eq!(back.side, side);
                let (u3, v3) = back.local(u2, v2);
                assert!((u3 - u).abs() < 1e-12 && (v3 - v).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn adjacency_matches_the_fold() {
        // With n = s the face grid is the net's blocks: a cell one step (or up to a face width)
        // beyond a side is the block the fold gives.
        for s in SIDES {
            let cube = Cube::new(s);
            let n = i64::from(s);
            for f in Face::ALL {
                for side in FaceSide::ALL {
                    for depth in [0, 1, n / 3, n - 1] {
                        for t in [0, 1, n / 2, n - 2, n - 1] {
                            let (i, j) = match side {
                                FaceSide::East => (n + depth, t),
                                FaceSide::West => (-1 - depth, t),
                                FaceSide::South => (t, n + depth),
                                FaceSide::North => (t, -1 - depth),
                            };
                            let (g, i2, j2, k) = cell_on(f, n, i, j).unwrap();
                            let (x, z) = cube.block_of_cell(f, i, j);
                            let (fx, fz, fk) = match cube.fold(x, z) {
                                CubeFold::Face { face, i, j } => {
                                    let (bx, bz) = cube.block_of_cell(face, i, j);
                                    (bx, bz, 0)
                                }
                                CubeFold::Outside { x, z, iso, .. } => (x, z, iso.k),
                                CubeFold::Ocean => panic!("{f:?} {side:?} ({i}, {j}) is ocean"),
                            };
                            // A block in a void square by F0's corner may belong to the other
                            // edge's half (the fold splits the square on its diagonal); the
                            // adjacency's answer is then the other half's.
                            let diag = depth >= t.min(n - 1 - t);
                            if !diag {
                                assert_eq!(
                                    (cube.block_of_cell(g, i2, j2), k),
                                    ((fx, fz), fk),
                                    "s {s} {f:?} {side:?} depth {depth} t {t}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn seams_map_edge_onto_edge_and_close() {
        for s in SIDES {
            let cube = Cube::new(s);
            let s = i64::from(s);
            let edges = cube.edges();
            assert_eq!(edges.len(), 14);
            for (k, e) in edges.iter().enumerate() {
                let p = &edges[e.partner];
                assert_eq!(p.partner, k);
                assert_eq!(p.seam, e.seam);
                assert_eq!(e.span[1] - e.span[0], s);
                // Every block of the band's first row maps onto the partner edge's inner row,
                // and every block of the partner's band onto this edge's inner row: the motion
                // takes beyond-this-edge onto inside-the-partner.
                for t in e.span[0]..e.span[1] {
                    let (x, z) = match (e.axis, e.beyond) {
                        (Axis::X, Side::Max) => (e.coord, t),
                        (Axis::X, Side::Min) => (e.coord - 1, t),
                        (Axis::Z, Side::Max) => (t, e.coord),
                        (Axis::Z, Side::Min) => (t, e.coord - 1),
                    };
                    let (px, pz) = e.iso.cell(x, z);
                    let inner = match (p.axis, p.beyond) {
                        (Axis::X, Side::Max) => px == p.coord - 1,
                        (Axis::X, Side::Min) => px == p.coord,
                        (Axis::Z, Side::Max) => pz == p.coord - 1,
                        (Axis::Z, Side::Min) => pz == p.coord,
                    };
                    assert!(
                        inner,
                        "{}: ({x}, {z}) -> ({px}, {pz}) not on {}",
                        e.name, p.name
                    );
                    let along = match p.axis {
                        Axis::X => pz,
                        Axis::Z => px,
                    };
                    assert!((p.span[0]..p.span[1]).contains(&along));
                    assert!(
                        cube.face_of_block(px, pz)
                            .is_some_and(|(f, _, _)| f == p.face)
                    );
                }
                // The partner's motion is this one's inverse; turns close up.
                assert_eq!(p.iso, e.iso.inverse(), "{} / {}", e.name, p.name);
                let order = match e.motion {
                    EdgeMotion::Slide => 1,
                    EdgeMotion::HalfTurn => 2,
                    EdgeMotion::QuarterTurn => 4,
                };
                let mut m = Iso::IDENTITY;
                for _ in 0..order {
                    m = e.iso.compose(&m);
                }
                if e.motion != EdgeMotion::Slide {
                    assert_eq!(m, Iso::IDENTITY, "{} to the power {order}", e.name);
                    let pivot = e.pivot.unwrap();
                    let pf = (pivot[0] as f64, pivot[1] as f64);
                    assert_eq!(e.iso.point(pf.0, pf.1), pf);
                    assert_eq!(pivot[0].rem_euclid(16), 0);
                    assert_eq!(pivot[1].rem_euclid(16), 0);
                }
            }
        }
    }

    #[test]
    fn fold_round_trips_outside_blocks() {
        // Plan W2: fold(partner) is the identity on 10,000 random outside blocks within s of the
        // planet: the partner is a planet block, and the motion's inverse brings it back.
        for s in SIDES {
            let cube = Cube::new(s);
            let s = i64::from(s);
            let mut rng = Rng(0x0DE5_7001 ^ s as u64);
            let (mut outside, mut ocean, mut tries) = (0, 0, 0);
            while outside < 10_000 {
                tries += 1;
                assert!(tries < 1_000_000);
                let x = rng.range(-5 * s / 2 - s, 7 * s / 2 + s);
                let z = rng.range(-5 * s / 2, 5 * s / 2);
                if cube.face_of_block(x, z).is_some() || !within_s(&cube, x, z) {
                    continue;
                }
                match cube.fold(x, z) {
                    CubeFold::Outside {
                        edge,
                        x: px,
                        z: pz,
                        iso,
                    } => {
                        outside += 1;
                        assert_eq!(iso.cell(x, z), (px, pz));
                        assert!(matches!(cube.fold(px, pz), CubeFold::Face { .. }));
                        assert_eq!(iso.inverse().cell(px, pz), (x, z));
                        let e = &cube.edges()[edge];
                        assert!(e.depth(cube.wrap_x(x), z).is_some());
                        assert_eq!(
                            cube.face_of_block(px, pz).unwrap().0,
                            cube.edges()[e.partner].face
                        );
                    }
                    CubeFold::Ocean => ocean += 1,
                    CubeFold::Face { .. } => panic!("({x}, {z}) is off the planet"),
                }
            }
            eprintln!(
                "s {s}: 10,000 outside blocks round-trip, {ocean} ocean (past the net's corners)"
            );
        }
    }

    /// Whether the block is within s (Chebyshev, the slide counted) of a planet block.
    fn within_s(cube: &Cube, x: i64, z: i64) -> bool {
        let s = cube.side();
        let x = cube.wrap_x(x);
        Face::ALL.iter().any(|&f| {
            let (ox, oz) = cube.face_origin(f);
            [-4 * s, 0, 4 * s].iter().any(|&o| {
                let dx = (ox + o - x).max(x - (ox + o + s - 1)).max(0);
                let dz = (oz - z).max(z - (oz + s - 1)).max(0);
                dx.max(dz) <= s
            })
        })
    }

    #[test]
    fn corners_and_their_images_agree() {
        for s in SIDES {
            let cube = Cube::new(s);
            let corners = cube.corners();
            assert_eq!(corners.len(), 8);
            let h = s as f64 / 2.0;
            for c in &corners {
                let want = c.cube.map(f64::from);
                for img in &c.images {
                    // The image is a corner of each of its faces that the net draws there; seen
                    // from every face square that touches it, it is that face's cube corner.
                    let mut seen = 0;
                    for f in Face::ALL {
                        let (cx, cz) = cube.face_centre(f);
                        for off in [0.0, 4.0 * s as f64, -4.0 * s as f64] {
                            let u = (img[0] as f64 + off - cx as f64) / h;
                            let v = (img[1] as f64 - cz as f64) / h;
                            if u.abs() == 1.0 && v.abs() == 1.0 {
                                assert!(c.faces.contains(&f), "{} image {img:?} on {f:?}", c.name);
                                let p = gnomonic(f, u, v);
                                assert_eq!(p, want, "{} via {f:?}", c.name);
                                seen += 1;
                            }
                        }
                    }
                    assert!(seen >= 1, "{} image {img:?} touches no face", c.name);
                }
                // The three faces' corners at this cube corner, all accounted for in the images.
                for &f in &c.faces {
                    let found = c.images.iter().any(|img| {
                        let (cx, cz) = cube.face_centre(f);
                        [0.0, 4.0 * s as f64, -4.0 * s as f64].iter().any(|off| {
                            let u = (img[0] as f64 + off - cx as f64) / h;
                            let v = (img[1] as f64 - cz as f64) / h;
                            u.abs() == 1.0 && v.abs() == 1.0 && gnomonic(f, u, v) == want
                        })
                    });
                    assert!(found, "{}: face {f:?} has no image", c.name);
                }
            }
            // The F0 corners are the quarter-turn pivots.
            let pivots: Vec<[i64; 2]> = cube.edges().iter().filter_map(|e| e.pivot).collect();
            for c in corners.iter().filter(|c| c.faces.contains(&Face::F0)) {
                assert_eq!(c.images.len(), 1);
                assert!(pivots.contains(&c.images[0]));
            }
        }
    }

    #[test]
    fn the_void_squares_split_on_the_diagonal() {
        // Plan §1.4: the void square above F1 splits on the diagonal through F0's corner, each
        // half folding through its own edge.
        let cube = Cube::new(1_024);
        let h = 512;
        for a in [0i64, 3, 100, 500] {
            for b in [0i64, 3, 100, 500] {
                let (x, z) = (h + a, -h - 1 - b);
                let CubeFold::Outside { edge, .. } = cube.fold(x, z) else {
                    panic!()
                };
                let name = &cube.edges()[edge].name;
                if a <= b {
                    assert_eq!(name, "N-east");
                } else {
                    assert_eq!(name, "F1-top");
                }
            }
        }
        // The void above F2 is N turned half way round.
        let CubeFold::Outside { x, z, iso, .. } = cube.fold(2 * 1_024, -1_024) else {
            panic!()
        };
        assert_eq!(iso.k, 2);
        assert!(matches!(
            cube.fold(x, z),
            CubeFold::Face { face: Face::N, .. }
        ));
    }

    #[test]
    fn points_fold_like_blocks() {
        let cube = Cube::new(1_024);
        let mut rng = Rng(7);
        for _ in 0..20_000 {
            let x = rng.range(-4_000, 4_000);
            let z = rng.range(-2_000, 2_000);
            let bf = cube.fold(x, z);
            let pf = cube.fold_point(x as f64 + 0.5, z as f64 + 0.5);
            match bf {
                CubeFold::Face { face, i, j } => {
                    let p = pf.unwrap();
                    assert_eq!(p.face, face);
                    assert_eq!(p.edge, None);
                    let (bx, bz) = cube.block_of_cell(face, i, j);
                    assert_eq!((p.x, p.z), (bx as f64 + 0.5, bz as f64 + 0.5));
                }
                CubeFold::Outside {
                    edge,
                    x: px,
                    z: pz,
                    iso,
                } => {
                    let p = pf.unwrap();
                    assert_eq!(p.iso, iso);
                    assert_eq!(p.edge, Some(edge));
                    assert_eq!((p.x, p.z), (px as f64 + 0.5, pz as f64 + 0.5));
                }
                CubeFold::Ocean => assert!(pf.is_none()),
            }
        }
    }

    #[test]
    fn turns_are_rotations_and_invert() {
        let polar = CubeTurn::polar(30.0);
        // the planet point (0°, 30°) at F0's centre, the north pole at N's
        let p = crate::projection::unit_of_lat_lon(0.0, 30.0);
        let c = polar.to_cube(p);
        assert!((c[0] - 1.0).abs() < 1e-15 && c[1].abs() < 1e-15 && c[2].abs() < 1e-15);
        assert_eq!(polar.to_cube([0.0, 0.0, 1.0]), [0.0, 0.0, 1.0]);
        assert!(CubeTurn::from_rows(polar.rows()).is_ok());
        // a free turn: the inverse is the transpose
        let r = 1.0 / 3.0_f64.sqrt();
        let s = 1.0 / 2.0_f64.sqrt();
        let t = 1.0 / 6.0_f64.sqrt();
        let free = CubeTurn::from_rows([[r, r, r], [s, -s, 0.0], [t, t, -2.0 * t]]).unwrap();
        let mut rng = Rng(3);
        for _ in 0..1_000 {
            let q = [
                rng.range(-1000, 1000) as f64,
                rng.range(-1000, 1000) as f64,
                rng.range(-1000, 1000) as f64,
            ];
            let back = free.to_planet(free.to_cube(q));
            for k in 0..3 {
                assert!((back[k] - q[k]).abs() < 1e-12, "{q:?} {back:?}");
            }
        }
        // reflections and non-orthonormal rows are refused
        assert!(CubeTurn::from_rows([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]]).is_err());
        assert!(CubeTurn::from_rows([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.1, 1.0]]).is_err());
    }
}

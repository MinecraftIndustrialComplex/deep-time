// SPDX-License-Identifier: LGPL-3.0-or-later
//! The map projection between block space and the sphere: the one place every conversion between
//! block coordinates, raster rows and latitude/longitude goes through
//! (`docs/design/equal-area-projection.md` §2; the Java twin is `worlddata/Projection.java`, the
//! pages' is `window.DTProj` in `deeptime-viewer/assets/projection.js`).
//!
//! The projection is **Lambert's cylindrical equal-area projection**, true on the equator (Ben,
//! decisions log 2026-10-02: "Lambert for now"): block x is linear in longitude and block z in
//! sin φ, so every block covers the same area of the planet. It replaced the equirectangular
//! layout (z linear in φ) the module was written with.
//!
//! # Conventions
//! - A planet of circumference C blocks has radius **R = C / 2π** blocks ([`Projection::radius`]).
//! - **Longitude** λ = 360°·(x + C/2)/C in [0°, 360°): x = −C/2 (the seam) is 0°, x wraps with
//!   period C.
//! - **Latitude** φ = asin(−z / R), north positive; north is −z, as in Minecraft. The poles are at
//!   z = ∓R ([`Projection::pole_z`]: 2,607.6 blocks at C = 16,384) and φ clamps to ±90° beyond.
//! - **Unit vector** (cos φ cos λ, cos φ sin λ, sin φ): +Z the north pole, +X the seam meridian.
//! - **Metric**: east–west a block is cos φ blocks of planet, north–south 1 / cos φ
//!   ([`Projection::scale_ew`], [`Projection::scale_ns`], [`Projection::metric`]); their product,
//!   the area a block stands for, is 1 everywhere ([`Projection::area_scale`]). North is −z
//!   everywhere ([`Projection::north`]). Shapes are true only on the equator.
//! - **Grids** (`GridDesc`) are square lattices in block space: W = C/r columns from the seam, and
//!   rows from z = −Zo to Zo, Zo the smallest multiple of the planet's quantum (64 blocks on every
//!   real planet) at or past the pole ([`Projection::grid_half_extent`]); so the grids of a planet
//!   share their origin and nest (L3 row 8j is L1 row j), and their rows are uniform in sin φ:
//!   every cell inside the sphere has the same area. The rows past a pole hold no planet.
//! - **Map pictures** of the whole planet run pole to pole: x linear in longitude from the seam,
//!   y linear in z (so in sin φ) from the north pole; aspect π : 1 ([`Projection::picture_height`]).
//! - **Topology**: a raster wraps in x and stops at its first and last rows
//!   ([`wrap_col`], [`clamp_row`], [`neighbour`]); block x wraps with period C and the planet
//!   ends at the poles ([`Projection::on_planet`]).
//!
//! The spatial indexes of the world data (the river tiles, the structure buckets, the site index,
//! the playability regions) are frames for lookups, not geography: lattices of their own cells
//! over the grids' extent, from [`Projection::index_origin`] over [`Projection::index_span`]
//! blocks.
//!
//! The projection is a game choice (ledger: SCL-049); that it keeps areas is a tested property of
//! the formulas (ledger: SCL-052).

use std::f64::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

use crate::cube::{Cube, CubeTurn, EdgeMotion, Face};
use crate::cube_map::{CubeMap, CubeView};
use crate::petroff::Petroff;
use crate::portable_math as pm;

/// The largest quantum of the grids' z extent, blocks ([`Projection::quantum`]).
pub const QUANTUM_MAX: u32 = 64;

/// Block space ↔ sphere for one planet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projection {
    c: f64,
    /// R = C / 2π, blocks.
    r: f64,
}

impl Projection {
    /// The projection of a planet `circumference` blocks round.
    pub fn new(circumference: u32) -> Projection {
        Projection::of(f64::from(circumference))
    }

    /// [`Projection::new`] for a circumference held as a float.
    pub fn of(circumference: f64) -> Projection {
        Projection {
            c: circumference,
            r: circumference / TAU,
        }
    }

    /// C, blocks.
    pub fn circumference(&self) -> f64 {
        self.c
    }

    /// The planet's radius R = C / 2π, blocks (the sphere the blocks stand for, at true scale on
    /// the equator).
    pub fn radius(&self) -> f64 {
        self.r
    }

    /// |z| of the poles, blocks: R. The planet's z range is [−pole_z, pole_z].
    pub fn pole_z(&self) -> f64 {
        self.r
    }

    /// The planet's extent in block space, blocks: (east–west, north–south) = (C, 2R).
    pub fn extent(&self) -> (f64, f64) {
        (self.c, 2.0 * self.pole_z())
    }

    /// Whether block (x, z) is on the planet: between the poles (x wraps, so every x is).
    pub fn on_planet(&self, _x: f64, z: f64) -> bool {
        z.abs() <= self.pole_z()
    }

    /// Whether block z lies strictly between the poles.
    pub fn inside_poles(&self, z: f64) -> bool {
        z.abs() < self.pole_z()
    }

    /// Clamps block z to the planet: [−pole_z, pole_z].
    pub fn clamp_z(&self, z: f64) -> f64 {
        z.clamp(-self.pole_z(), self.pole_z())
    }

    /// Wraps block x into [−C/2, C/2).
    pub fn wrap_x(&self, x: f64) -> f64 {
        let t = (x + 0.5 * self.c).rem_euclid(self.c);
        (if t >= self.c { 0.0 } else { t }) - 0.5 * self.c
    }

    /// Longitude of block x, degrees in [0, 360).
    pub fn lon_deg(&self, x: f64) -> f64 {
        wrap_deg(360.0 * (self.wrap_x(x) + 0.5 * self.c) / self.c)
    }

    /// Block x of a longitude, in [−C/2, C/2).
    pub fn x_of_lon(&self, lon_deg: f64) -> f64 {
        self.wrap_x(wrap_deg(lon_deg) / 360.0 * self.c - 0.5 * self.c)
    }

    /// sin φ of block z, clamped to [−1, 1] (north is −z).
    pub fn sin_lat(&self, z: f64) -> f64 {
        (-z / self.r).clamp(-1.0, 1.0)
    }

    /// Latitude of block z, degrees in [−90, 90] (north is −z; clamped beyond the poles).
    pub fn lat_deg(&self, z: f64) -> f64 {
        self.sin_lat(z).asin().to_degrees()
    }

    /// Block z of a latitude (clamped to ±90°), in [−R, R].
    pub fn z_of_lat(&self, lat_deg: f64) -> f64 {
        -self.r * lat_deg.clamp(-90.0, 90.0).to_radians().sin()
    }

    /// Block (x, z) of a latitude and longitude, x in [−C/2, C/2).
    pub fn block_of(&self, lat_deg: f64, lon_deg: f64) -> (f64, f64) {
        (self.x_of_lon(lon_deg), self.z_of_lat(lat_deg))
    }

    /// (latitude, longitude) of block (x, z), degrees.
    pub fn lat_lon(&self, x: f64, z: f64) -> (f64, f64) {
        (self.lat_deg(z), self.lon_deg(x))
    }

    /// The unit vector of block (x, z): (cos φ cos λ, cos φ sin λ, sin φ).
    pub fn unit(&self, x: f64, z: f64) -> [f64; 3] {
        let (sp, cp) = (self.sin_lat(z), self.cos_lat(z));
        let (sl, cl) = self.lon_deg(x).to_radians().sin_cos();
        [cp * cl, cp * sl, sp]
    }

    /// cos φ of block z (0 at and beyond the poles).
    pub fn cos_lat(&self, z: f64) -> f64 {
        let s = self.sin_lat(z);
        (1.0 - s * s).max(0.0).sqrt()
    }

    /// Blocks of planet per block east–west at block z: cos φ (the metric's x scale; 0 at the
    /// poles).
    pub fn scale_ew(&self, z: f64) -> f64 {
        self.cos_lat(z)
    }

    /// Blocks of planet per block north–south at block z: 1 / cos φ (infinite at the poles).
    pub fn scale_ns(&self, z: f64) -> f64 {
        1.0 / self.cos_lat(z)
    }

    /// The metric of block space at block (x, z): (g_xx, g_xz, g_zz), blocks of planet per block,
    /// squared. Diagonal for a cylindrical projection: (cos² φ, 0, 1 / cos² φ).
    pub fn metric(&self, _x: f64, z: f64) -> (f64, f64, f64) {
        let c = self.cos_lat(z);
        (c * c, 0.0, 1.0 / (c * c))
    }

    /// Area of planet per block of block space (√det g): 1 everywhere on the planet (the
    /// projection keeps areas), 0 past the poles.
    pub fn area_scale(&self, z: f64) -> f64 {
        if self.on_planet(0.0, z) { 1.0 } else { 0.0 }
    }

    /// The share of the z band [z_lo, z_hi] that lies on the planet ([−R, R]): 1 inside, less on a
    /// band reaching past a pole. With [`Projection::area_scale`] 1, it is the band's area weight.
    pub fn band_share(&self, z_lo: f64, z_hi: f64) -> f64 {
        let len = z_hi - z_lo;
        if len <= 0.0 {
            return 0.0;
        }
        let on = (z_hi.min(self.r) - z_lo.max(-self.r)).max(0.0);
        (on / len).clamp(0.0, 1.0)
    }

    /// The unit vector of north in block space at block (x, z), as (dx, dz): (0, −1) everywhere.
    pub fn north(&self, _x: f64, _z: f64) -> (f64, f64) {
        (0.0, -1.0)
    }

    /// The great-circle distance between blocks a and b, blocks of planet.
    pub fn great_circle_blocks(&self, a: (f64, f64), b: (f64, f64)) -> f64 {
        let (p1, p2) = (
            self.lat_deg(a.1).to_radians(),
            self.lat_deg(b.1).to_radians(),
        );
        let dl = (self.lon_deg(b.0) - self.lon_deg(a.0)).to_radians();
        // Haversine: well conditioned for short distances.
        let s = ((p2 - p1) / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
        2.0 * self.radius() * s.sqrt().clamp(0.0, 1.0).asin()
    }

    /// The quantum the grids' z extent is a multiple of: the largest power of two up to
    /// [`QUANTUM_MAX`] that divides C/2 and is at most R/2 (64 for every real planet).
    pub fn quantum(&self) -> u32 {
        let half = (self.c / 2.0).round() as u64;
        let mut q = QUANTUM_MAX;
        while q > 1 && (!half.is_multiple_of(u64::from(q)) || f64::from(q) > self.r / 2.0) {
            q /= 2;
        }
        q
    }

    /// The half-extent Zo of the planet's grids, blocks: the smallest multiple of the quantum at
    /// or past the pole. Every grid of a planet shares it (whatever its `r`), so the grids' rows
    /// nest (L3 row 8j is L1 row j).
    pub fn grid_half_extent(&self, _r: u32) -> i64 {
        let q = f64::from(self.quantum());
        ((self.r / q).ceil() * q) as i64
    }

    /// Block z of the first row of a grid of `r` blocks per cell: −Zo, at or just past the north
    /// pole.
    pub fn grid_origin_z(&self, r: u32) -> i64 {
        -self.grid_half_extent(r)
    }

    /// Rows of a grid of `r` blocks per cell: 2·Zo / r (rounded up when r does not divide it,
    /// which takes an r coarser than twice the quantum, 128 blocks on a real planet).
    pub fn grid_rows(&self, r: u32) -> u32 {
        let r = i64::from(r.max(1));
        ((2 * self.grid_half_extent(r as u32) + r - 1) / r) as u32
    }

    /// Block (x, z) of the first cell of the world data's spatial indexes (river tiles, structure
    /// buckets, site index, playability regions): (−C/2, −Zo), the seam and the grids' first row.
    pub fn index_origin(&self) -> (f64, f64) {
        let (x, z) = self.index_origin_blocks();
        (x as f64, z as f64)
    }

    /// Blocks the spatial indexes span: (east–west, north–south) = (C, 2·Zo).
    pub fn index_span(&self) -> (f64, f64) {
        let (x, z) = self.index_span_blocks();
        (x as f64, z as f64)
    }

    /// [`Projection::index_origin`] in whole blocks.
    pub fn index_origin_blocks(&self) -> (i64, i64) {
        let c = self.c as i64;
        (-(c / 2), -self.grid_half_extent(1))
    }

    /// [`Projection::index_span`] in whole blocks.
    pub fn index_span_blocks(&self) -> (i64, i64) {
        (self.c as i64, 2 * self.grid_half_extent(1))
    }

    /// Block z of the north edge of the workbench's tile pyramid: −C/4. The pyramid is a 2 : 1
    /// frame of power-of-two tiles centred on the equator, wider than the planet (pole to pole is
    /// C/π); its pixels past the poles hold no planet.
    pub fn tile_frame_z(&self) -> f64 {
        -self.c / 4.0
    }

    /// The longitude, degrees (not wrapped), at column coordinate `x` of a map picture or grid
    /// `w` columns wide (x = 0 the seam).
    pub fn picture_lon_deg(x: f64, w: u32) -> f64 {
        360.0 * x / f64::from(w.max(1))
    }

    /// The column coordinate, in [0, w), of longitude `lon_deg` in a map picture `w` columns wide.
    pub fn picture_x_of_lon(lon_deg: f64, w: u32) -> f64 {
        wrap_deg(lon_deg) / 360.0 * f64::from(w)
    }

    /// Block z at row coordinate `y` of a map picture `h` rows high (y = 0 the north pole, y = h
    /// the south pole; pixel `py` is centred on y = py, as keyframes are).
    pub fn picture_z(&self, y: f64, h: u32) -> f64 {
        -self.r + 2.0 * self.r * y / f64::from(h.max(1))
    }

    /// The row coordinate of block z in a map picture `h` rows high.
    pub fn picture_y(&self, z: f64, h: u32) -> f64 {
        (z + self.r) / (2.0 * self.r) * f64::from(h.max(1))
    }

    /// The latitude, degrees, at row coordinate `y` of a map picture `h` rows high: rows are
    /// uniform in sin φ.
    pub fn picture_lat_deg(y: f64, h: u32) -> f64 {
        (1.0 - 2.0 * y / f64::from(h.max(1)))
            .clamp(-1.0, 1.0)
            .asin()
            .to_degrees()
    }

    /// The row coordinate of latitude `lat_deg` in a map picture `h` rows high.
    pub fn picture_y_of_lat(lat_deg: f64, h: u32) -> f64 {
        (1.0 - lat_deg.clamp(-90.0, 90.0).to_radians().sin()) / 2.0 * f64::from(h)
    }

    /// The true size, metres (east–west, north–south), of a pixel at row coordinate `y` of a
    /// `w × h` map picture of a sphere of radius `radius_m`: 2πR·cos φ / w wide and
    /// 2R / (h·cos φ) high, cos φ held at `min_cos` or more near the poles.
    pub fn picture_pixel_m(y: f64, w: u32, h: u32, radius_m: f64, min_cos: f64) -> (f64, f64) {
        let s = (1.0 - 2.0 * y / f64::from(h.max(1))).clamp(-1.0, 1.0);
        let cos = (1.0 - s * s).sqrt().max(min_cos);
        (
            TAU * radius_m * cos / f64::from(w.max(1)),
            2.0 * radius_m / (f64::from(h.max(1)) * cos),
        )
    }

    /// The height of a map picture `w` pixels wide at the projection's aspect (π : 1).
    pub fn picture_height(w: u32) -> u32 {
        ((f64::from(w) / PI).round() as u32).max(1)
    }
}

/// The raster's topology in x: wraps a column index into [0, w).
#[inline]
pub fn wrap_col(i: i64, w: i64) -> i64 {
    i.rem_euclid(w)
}

/// The raster's topology in z: clamps a row index into [0, h − 1] (the rows stop at the poles).
#[inline]
pub fn clamp_row(j: i64, h: i64) -> i64 {
    j.clamp(0, h - 1)
}

/// The neighbour rule of a `w × h` raster: the cell at offset (di, dj) from cell (i, j), or
/// `None` when there is none. Columns wrap when `wraps_x` (the planet's grids do; small test
/// rasters may not), rows stop at the first and the last.
#[inline]
pub fn neighbour(
    w: usize,
    h: usize,
    wraps_x: bool,
    (i, j): (usize, usize),
    (di, dj): (i64, i64),
) -> Option<(usize, usize)> {
    let nj = j as i64 + dj;
    if nj < 0 || nj >= h as i64 {
        return None;
    }
    let mut ni = i as i64 + di;
    if ni < 0 || ni >= w as i64 {
        if !wraps_x {
            return None;
        }
        ni = wrap_col(ni, w as i64);
    }
    Some((ni as usize, nj as usize))
}

/// The unit vector at latitude `lat_deg` and longitude `lon_deg` (degrees).
pub fn unit_of_lat_lon(lat_deg: f64, lon_deg: f64) -> [f64; 3] {
    let (sp, cp) = lat_deg.to_radians().sin_cos();
    let (sl, cl) = lon_deg.to_radians().sin_cos();
    [cp * cl, cp * sl, sp]
}

/// Wraps an angle into [0°, 360°).
pub fn wrap_deg(a: f64) -> f64 {
    let w = a.rem_euclid(360.0);
    if w >= 360.0 { 0.0 } else { w }
}

// ------------------------------------------------------------------------------------------------
// The seamless world: Petroff's equal-area square in the Guyou 2:1 layout.
// ------------------------------------------------------------------------------------------------

/// Which projection a world uses (manifest `projection.kind`, FORMAT.md "The world's layout").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionKind {
    /// Lambert's cylindrical equal-area projection: one east–west slide, pole caps.
    Lambert,
    /// Petroff's equal-area square in the Guyou 2:1 layout: one east–west slide, two half-turn
    /// edges (Ben, decisions log 2026-10-04: "We'll proceed w/ Petroff–Guyou").
    #[default]
    PetroffGuyou,
    /// The cube world: six faces in a cross, drawn through a map table and turned freely onto the
    /// planet (Ben, decisions log 2026-10-22; format 0.7, FORMAT.md "The cube world's layout").
    Cube,
}

impl ProjectionKind {
    /// The setting's text (`projection.kind`).
    pub fn name(self) -> &'static str {
        match self {
            ProjectionKind::Lambert => "lambert",
            ProjectionKind::PetroffGuyou => "petroff_guyou",
            ProjectionKind::Cube => "cube",
        }
    }

    /// The kind named `s`.
    pub fn parse(s: &str) -> Option<ProjectionKind> {
        match s {
            "lambert" => Some(ProjectionKind::Lambert),
            "petroff_guyou" => Some(ProjectionKind::PetroffGuyou),
            "cube" => Some(ProjectionKind::Cube),
            _ => None,
        }
    }
}

/// The version of the layout block's schema (`projection.layout_version`).
pub const LAYOUT_VERSION: u32 = 1;

/// The cube world's layout block version (`projection.layout_version` of a `cube` world).
pub const CUBE_LAYOUT_VERSION: u32 = 2;

/// The hemisphere square's side is a multiple of this many blocks: the world's x period 2b is
/// then a multiple of 512 (a region file, what `WorldDataRef` and Toroidal World's loop take),
/// every grid's r (up to 256) divides b, and the half-turn pivots sit on chunk corners.
pub const HEMISPHERE_QUANTUM: u32 = 256; // ledger: SCL-084 (game-choice)

/// The cube world's face side is a multiple of this many blocks, for the same reasons as
/// [`HEMISPHERE_QUANTUM`]: every grid's r (up to 256) divides s, the east-west period 4s is a
/// multiple of 512, and every turn pivot (s/2 from a face centre) sits on a chunk corner
/// (docs/plans/cube-world-plan.md §1.1).
pub const FACE_QUANTUM: u32 = 256; // ledger: SCL-100 (game-choice)

/// The cube world's face side s for a planet `circumference` blocks round at today's scale: the
/// six faces' 6s² blocks cover the planet's 4πR² (R = C/2π, so s = C/√(6π)), rounded to the
/// nearest multiple of [`FACE_QUANTUM`] (game choice, ledger SCL-100). 4k: 1,024; 16k: 3,840 (the
/// planet drawn 1.8 % larger across, 294.8 m a block); 32k: 7,424 (1.6 % smaller, 305.0 m).
pub fn face_for(circumference: u32) -> u32 {
    let exact = f64::from(circumference) / (6.0 * PI).sqrt();
    let q = f64::from(FACE_QUANTUM);
    (((exact / q).round().max(1.0)) * q) as u32
}

/// One way a world edge is glued (`edges[].kind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// A translation (the east–west wrap).
    Slide,
    /// A half-turn about a point of the edge (the edge is glued onto itself), or on a cube world
    /// about the centre of a void square (the edge is glued onto its partner).
    HalfTurn,
    /// A quarter turn about one of F0's corners (cube worlds: k = 1 or 3).
    QuarterTurn,
}

/// The axis an edge's normal runs along (`edges[].axis`): `x` for an east or west edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    X,
    Z,
}

/// Which side of the edge line is beyond the world (`edges[].side`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// Beyond is v < coord (a west or north edge).
    Min,
    /// Beyond is v ≥ coord (an east or south edge).
    Max,
}

/// One edge of the world's fundamental domain and the motion that brings what lies beyond it
/// back inside (FORMAT.md "The world's layout"; the wrap engine's `WrapEdge`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EdgeSpec {
    /// `west`, `east`, `north`, `south`; a cube world's `F3-west`, `N-east`, ...
    pub name: String,
    /// The cube edge the net edge is half of (`east_west`, `ne`, `nw`, `se`, `sw`, `nn`, `ss`);
    /// cube worlds only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seam: Option<String>,
    pub kind: EdgeKind,
    pub axis: Axis,
    pub side: Side,
    /// The edge line, a block boundary (x = coord or z = coord).
    pub coord: i64,
    /// [first, one past the last] block along the edge.
    pub span: [i64; 2],
    /// The motion beyond → inside as `[quarter_turns, tx, tz]`, the wrap engine's
    /// `CellIsometry`: k clockwise quarter-turns seen from above, then the translation. On block
    /// cells k = 2 is (x, z) → (−x − 1 + tx, −z − 1 + tz); on points (−x + tx, −z + tz).
    pub iso: [i64; 3],
    /// The half-turn's centre (a block corner), for `half_turn` edges.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pivot: Option<[i64; 2]>,
    /// The other net edge of the same cube edge (cube worlds only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner: Option<String>,
    /// Who joins the edge in the game: `toroidal_world` (the slides, Ben 2026-10-03: "TW
    /// slides, engine turns") or `wrap_engine`.
    pub owner: String,
}

/// A cone point of the layout: a place on the planet whose surroundings the flat world covers
/// with less than a full turn (FORMAT.md "The world's layout").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConePoint {
    pub name: String,
    /// The world's angle round the point, degrees (180: half a turn missing; 270 at a cube's
    /// corner).
    pub cone_deg: f64,
    /// The three faces that meet there (cube worlds only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faces: Option<Vec<String>>,
    /// The corner's cube-frame direction, (±1, ±1, ±1) (cube worlds only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cube: Option<[i8; 3]>,
    pub lat_deg: f64,
    pub lon_deg: f64,
    /// The signed great-circle distance to the nearest land of the final keyframe, degrees
    /// (positive at sea, clamped to ±20°; the corner search's score). Cube worlds a bake wrote.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub land_distance_deg: Option<f64>,
    /// Whether a player can stand on it (cube worlds only: F0's four corners).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reachable: Option<bool>,
    /// Every block corner in the domain's closure that is this point.
    pub images: Vec<[i64; 2]>,
}

/// A pole's images in the world.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PoleSpec {
    /// `north` or `south`.
    pub name: String,
    /// The face the pole lies on and its local (u, v) (cube worlds only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub face: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub u: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v: Option<f64>,
    /// The points that are the pole: block corners on the polar layouts (written as integers),
    /// any net point on a cube world; empty when the pole is outside the world (Lambert).
    #[serde(serialize_with = "points_as_written")]
    pub images: Vec<[f64; 2]>,
}

impl PoleSpec {
    /// A pole of a polar layout: its images are block corners.
    pub fn corners(name: &str, images: &[[i64; 2]]) -> PoleSpec {
        PoleSpec {
            name: name.into(),
            face: None,
            u: None,
            v: None,
            images: images.iter().map(|p| [p[0] as f64, p[1] as f64]).collect(),
        }
    }
}

/// Writes whole coordinates as integers (the polar layouts' block corners, as format 0.5 wrote
/// them) and the rest as floats.
fn points_as_written<S: serde::Serializer>(
    points: &[[f64; 2]],
    ser: S,
) -> std::result::Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    #[derive(Serialize)]
    #[serde(untagged)]
    enum Coord {
        Int(i64),
        Float(f64),
    }
    let coord = |v: f64| {
        if v.is_finite() && v == v.trunc() && v.abs() < 9.0e15 {
            Coord::Int(v as i64)
        } else {
            Coord::Float(v)
        }
    };
    let mut seq = ser.serialize_seq(Some(points.len()))?;
    for p in points {
        seq.serialize_element(&[coord(p[0]), coord(p[1])])?;
    }
    seq.end()
}

/// The temporary safe border at edges nothing joins yet (FORMAT.md "The world's layout").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BorderSpec {
    /// `edge_stop`: players, their vehicles and Sable ships are stopped `inset_blocks` inside
    /// the named edges until the wrap engine joins them.
    pub kind: String,
    /// The polar layouts' bordered edges (empty on a cube world, which names `segments`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<String>,
    pub inset_blocks: f64,
    /// Always true: the border goes when the wrap engine installs the layout's topology.
    pub temporary: bool,
    /// A cube world's bordered net edges, by name: the 12 turning edges.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub segments: Vec<String>,
}

/// The fundamental domain, blocks: x in [x_min, x_max), z in [z_min, z_max].
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Domain {
    pub x_min: f64,
    pub x_max: f64,
    pub z_min: f64,
    pub z_max: f64,
    /// The east-west period, blocks (cube worlds only: 4s, Toroidal World's loop).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period_x: Option<i64>,
}

/// `manifest.json` `projection`: the world's projection and layout (FORMAT.md "The world's
/// layout"). The data the wrap engine reads to install its topology; stable from format 0.5.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionSpec {
    pub kind: ProjectionKind,
    pub layout_version: u32,
    /// b, the hemisphere square's side (petroff_guyou).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hemisphere_blocks: Option<u32>,
    /// λc, the longitude (degrees, the simulation's frame) of the western hemisphere's centre
    /// (petroff_guyou).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lambda_c_deg: Option<f64>,
    /// s, the face side in blocks (cube).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub face_blocks: Option<u32>,
    /// The map table that draws a face on the sphere, by name and sha256 (cube).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub map: Option<MapSpec>,
    /// The turn that places the cube on the planet, planet frame to cube frame, row-major (cube).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[[f64; 3]; 3]>,
    pub domain: Domain,
    /// The six faces in the format's order (cube).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faces: Option<Vec<FaceSpec>>,
    pub edges: Vec<EdgeSpec>,
    pub cone_points: Vec<ConePoint>,
    pub poles: Vec<PoleSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<BorderSpec>,
}

/// A cube world's map table (`projection.map`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapSpec {
    pub name: String,
    pub sha256: String,
}

/// One face of a cube world (`projection.faces[]`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FaceSpec {
    /// `F0` .. `F3`, `N`, `S`.
    pub id: String,
    /// The face's place in the format's order (rasters store the faces in this order).
    pub index: u32,
    /// The net block of the face's local cell (0, 0), its north-west corner.
    pub origin: [i64; 2],
    /// The cube-frame directions of the face's centre, local +u and local +v (`+X`, `-Z`, ...).
    pub frame: [String; 3],
    /// The planet point at the face's centre through the rotation, degrees.
    pub centre_lat_deg: f64,
    pub centre_lon_deg: f64,
}

/// Where a point landed after [`PetroffGuyou::fold`]: the canonical point and the number of
/// half-turns taken (odd: a heading there points the other way).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fold {
    pub x: f64,
    pub z: f64,
    pub half_turns: u32,
}

/// Petroff's equal-area square in the Guyou 2:1 layout: the seamless world (Ben, decisions log
/// 2026-10-04; `docs/design/seamless-wrapping.md`, `docs/research/notes/seamless-projection-survey.md`).
///
/// # The layout
/// Two hemisphere squares of side b side by side: block x in [−b, b), z in [−b/2, b/2]. The
/// western square (x < 0) holds the hemisphere centred on the equator at longitude λc, the eastern
/// one the other. The equator is the line z = 0; north is −z along it.
/// - **Edges.** West and east (x = ∓b) slide onto each other (period 2b, Toroidal World's wrap).
///   North (z = −b/2) and south (z = b/2) are each glued onto themselves by a half-turn about their
///   midpoint (0, ∓b/2): the point (x, −b/2 − d) beyond the north edge is (−x, −b/2 + d).
/// - **Poles** on the long edges at x = ∓b/2 (each appears twice there, the half-turn's images).
/// - **Cone points** (the four places where a turn of planet is drawn in half a turn of world):
///   the long edges' midpoints (0, ∓b/2) and the corners (±b, ∓b/2), at ±45° latitude on the
///   meridians λc ± 90°.
///
/// # The map
/// Tile coordinates X = 2x/b + 1 ∈ [−1, 3), Y = −2z/b ∈ [−1, 1] (Y up = north). The western tile
/// is Petroff's northern hemisphere in polar aspect, (u, v) = (X, Y) ↦ Petroff's square point
/// ((u + v)/2, (v − u)/2) (its diamond, so the tile edges are the polar-aspect equator); the
/// eastern tile is the mirrored southern one, (u, v) = (2 − X, Y) with φ ↦ −φ. A rotation of the
/// sphere then takes the polar-aspect pole (the western tile's centre) to (0°, λc) and the western
/// tile's top-edge midpoint to the north pole (the survey's `guyou`). Every edge is the
/// polar-aspect equator drawn the same way from both sides, so the glued edges map the same sphere
/// points exactly. Equal-area (Petroff), smooth across all three seams, slight creases
/// (≤ 5°) on the tiles' midlines and diagonals (survey §3.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PetroffGuyou {
    b: i64,
    lambda_c_deg: f64,
    pet: Petroff,
    /// The polar-aspect frame (a, e, f) and the world frame (a2, e2, f2): v_world =
    /// a2·(a·v) + e2·(e·v) + f2·(f·v).
    a: [f64; 3],
    e: [f64; 3],
    f: [f64; 3],
    a2: [f64; 3],
    e2: [f64; 3],
    f2: [f64; 3],
}

#[inline]
fn dot3(p: [f64; 3], q: [f64; 3]) -> f64 {
    p[0] * q[0] + p[1] * q[1] + p[2] * q[2]
}

/// The unit vector of (λ, φ) in radians, on the portable functions.
#[inline]
fn unit_pm(lambda: f64, phi: f64) -> [f64; 3] {
    let (sp, cp) = (pm::sin(phi), pm::cos(phi));
    [cp * pm::cos(lambda), cp * pm::sin(lambda), sp]
}

impl PetroffGuyou {
    /// The hemisphere side b for a planet `circumference` blocks round at today's scale: the
    /// world's 2b² blocks cover the planet's 4πR² (R = C/2π, so b = C/√(2π)), rounded to the
    /// nearest multiple of [`HEMISPHERE_QUANTUM`] (game choice, ledger SCL-084). 16k: 6,656 (the
    /// planet drawn 1.8 % larger across); 32k: 13,056 (0.1 % smaller).
    pub fn hemisphere_for(circumference: u32) -> u32 {
        let exact = f64::from(circumference) / (2.0 * PI).sqrt();
        let q = f64::from(HEMISPHERE_QUANTUM);
        (((exact / q).round().max(1.0)) * q) as u32
    }

    /// The world of hemisphere side `b` (a multiple of [`HEMISPHERE_QUANTUM`]) with the western
    /// hemisphere centred on longitude `lambda_c_deg`.
    pub fn new(b: u32, lambda_c_deg: f64) -> PetroffGuyou {
        let pet = Petroff::new();
        // The western tile's top-edge midpoint (u, v) = (0, 1) in polar aspect: on the equator.
        let (le, _) = pet.inverse(0.5, 0.5);
        let e = [pm::cos(le), pm::sin(le), 0.0];
        let a = [0.0, 0.0, 1.0];
        let f = [-e[1], e[0], 0.0]; // a × e
        let lc = pm::to_radians(lambda_c_deg);
        let a2 = [pm::cos(lc), pm::sin(lc), 0.0];
        let e2 = [0.0, 0.0, 1.0];
        let f2 = [a2[1], -a2[0], 0.0]; // a2 × e2
        PetroffGuyou {
            b: i64::from(b),
            lambda_c_deg,
            pet,
            a,
            e,
            f,
            a2,
            e2,
            f2,
        }
    }

    /// b, blocks.
    pub fn hemisphere(&self) -> i64 {
        self.b
    }

    /// λc, degrees.
    pub fn lambda_c_deg(&self) -> f64 {
        self.lambda_c_deg
    }

    /// The world's east–west period 2b, blocks (what `scale.circumference_blocks` holds for this
    /// world, and Toroidal World's loop).
    pub fn period(&self) -> i64 {
        2 * self.b
    }

    /// The radius of the sphere the world's blocks stand for at equal area, blocks: 2b² = 4πR².
    pub fn radius(&self) -> f64 {
        self.b as f64 / (2.0 * PI).sqrt()
    }

    /// The fundamental domain.
    pub fn domain(&self) -> Domain {
        let b = self.b as f64;
        Domain {
            x_min: -b,
            x_max: b,
            z_min: -b / 2.0,
            z_max: b / 2.0,
            period_x: None,
        }
    }

    /// Whether the point (x, z) lies in the domain (x wraps, so only z decides).
    pub fn on_world(&self, _x: f64, z: f64) -> bool {
        z.abs() <= self.b as f64 / 2.0
    }

    /// Wraps x into [−b, b).
    pub fn wrap_x(&self, x: f64) -> f64 {
        let p = self.period() as f64;
        let t = (x + 0.5 * p).rem_euclid(p);
        (if t >= p { 0.0 } else { t }) - 0.5 * p
    }

    /// Brings any point into the domain through the edges: x wraps; past the north or south
    /// edge the half-turn about its midpoint (repeated for points more than a world away).
    pub fn fold(&self, x: f64, z: f64) -> Fold {
        let (half, b) = (self.b as f64 / 2.0, self.b as f64);
        let (mut x, mut z, mut n) = (x, z, 0u32);
        for _ in 0..64 {
            if z < -half {
                x = -x;
                z = -b - z;
                n += 1;
            } else if z > half {
                x = -x;
                z = b - z;
                n += 1;
            } else {
                break;
            }
        }
        Fold {
            x: self.wrap_x(x),
            z: z.clamp(-half, half),
            half_turns: n,
        }
    }

    /// The block cell (bx, bz) brought into the domain, the wrap engine's cell arithmetic
    /// (`CellIsometry`): x wraps into [−b, b); a cell north of z = −b/2 is (−bx − 1, −b − bz − 1),
    /// south of z = b/2 (−bx − 1, b − bz − 1). Returns the cell and the half-turns taken.
    pub fn fold_cell(&self, bx: i64, bz: i64) -> (i64, i64, u32) {
        let (half, b) = (self.b / 2, self.b);
        let (mut x, mut z, mut n) = (bx, bz, 0u32);
        for _ in 0..64 {
            if z < -half {
                x = -x - 1;
                z = -b - z - 1;
                n += 1;
            } else if z >= half {
                x = -x - 1;
                z = b - z - 1;
                n += 1;
            } else {
                break;
            }
        }
        let p = 2 * b;
        let x = (x + b).rem_euclid(p) - b;
        (x, z, n)
    }

    /// Polar-aspect unit vector → world unit vector.
    #[inline]
    fn polar_to_world(&self, v: [f64; 3]) -> [f64; 3] {
        let (pa, pe, pf) = (dot3(self.a, v), dot3(self.e, v), dot3(self.f, v));
        [
            self.a2[0] * pa + self.e2[0] * pe + self.f2[0] * pf,
            self.a2[1] * pa + self.e2[1] * pe + self.f2[1] * pf,
            self.a2[2] * pa + self.e2[2] * pe + self.f2[2] * pf,
        ]
    }

    /// World unit vector → polar-aspect unit vector.
    #[inline]
    fn world_to_polar(&self, v: [f64; 3]) -> [f64; 3] {
        let (pa, pe, pf) = (dot3(self.a2, v), dot3(self.e2, v), dot3(self.f2, v));
        [
            self.a[0] * pa + self.e[0] * pe + self.f[0] * pf,
            self.a[1] * pa + self.e[1] * pe + self.f[1] * pf,
            self.a[2] * pa + self.e[2] * pe + self.f[2] * pf,
        ]
    }

    /// The unit vector of the point (x, z) (any point: it is folded into the domain first).
    pub fn unit(&self, x: f64, z: f64) -> [f64; 3] {
        let p = self.fold(x, z);
        self.unit_in_tile(p.x, p.z)
    }

    /// The unit vector of a point of the closed domain x ∈ [−b, b], z ∈ [−b/2, b/2] drawn from
    /// its own tile, without wrapping x (x ≤ 0 the western tile, x > 0 the eastern one, so x = b is
    /// drawn from the eastern tile's edge): the seam tests compare the two drawings of an edge.
    pub fn unit_unfolded(&self, x: f64, z: f64) -> [f64; 3] {
        let (b, h) = (self.b as f64, self.b as f64 / 2.0);
        self.unit_in_tile(x.clamp(-b, b), z.clamp(-h, h))
    }

    fn unit_in_tile(&self, x: f64, z: f64) -> [f64; 3] {
        let b = self.b as f64;
        let tx = 2.0 * x / b + 1.0;
        let ty = -2.0 * z / b;
        let (u, v, south) = if tx <= 1.0 {
            (tx, ty, false)
        } else {
            (2.0 - tx, ty, true)
        };
        let (lam, phi) = self.pet.inverse(0.5 * (u + v), 0.5 * (v - u));
        let phi = if south { -phi } else { phi };
        self.polar_to_world(unit_pm(lam, phi))
    }

    /// The canonical point (x, z) of a unit vector (any length > 0): x in [−b, b), z in
    /// [−b/2, b/2].
    pub fn point_of_unit(&self, v: [f64; 3]) -> (f64, f64) {
        let q = self.world_to_polar(v);
        let phi = pm::atan2(q[2], (q[0] * q[0] + q[1] * q[1]).sqrt());
        let lam = pm::atan2(q[1], q[0]);
        let south = phi < 0.0;
        let (xm, ym) = self.pet.forward(lam, if south { -phi } else { phi });
        let (u, v) = (xm - ym, xm + ym);
        let tx = if south { 2.0 - u } else { u };
        let b = self.b as f64;
        let x = (tx - 1.0) * b / 2.0;
        let z = -v * b / 2.0;
        (self.wrap_x(x), z.clamp(-b / 2.0, b / 2.0))
    }

    /// (latitude, longitude) of the point (x, z), degrees; longitude in [0, 360) in the
    /// simulation's frame (`unit_of_lat_lon`).
    pub fn lat_lon(&self, x: f64, z: f64) -> (f64, f64) {
        lat_lon_of_unit(self.unit(x, z))
    }

    /// The canonical point of a latitude and longitude (degrees).
    pub fn block_of(&self, lat_deg: f64, lon_deg: f64) -> (f64, f64) {
        let lat = pm::to_radians(lat_deg.clamp(-90.0, 90.0));
        let lon = pm::to_radians(lon_deg);
        self.point_of_unit(unit_pm(lon, lat))
    }

    /// The layout block for `manifest.json` (FORMAT.md "The world's layout"), with the border
    /// `border` (or none).
    pub fn spec(&self, border: Option<BorderSpec>) -> ProjectionSpec {
        let (b, h) = (self.b, self.b / 2);
        let edge = |name: &str,
                    kind: EdgeKind,
                    axis: Axis,
                    side: Side,
                    coord: i64,
                    span: [i64; 2],
                    iso: [i64; 3],
                    pivot: Option<[i64; 2]>,
                    owner: &str| EdgeSpec {
            name: name.into(),
            seam: None,
            kind,
            axis,
            side,
            coord,
            span,
            iso,
            pivot,
            partner: None,
            owner: owner.into(),
        };
        let edges = vec![
            edge(
                "west",
                EdgeKind::Slide,
                Axis::X,
                Side::Min,
                -b,
                [-h, h],
                [0, 2 * b, 0],
                None,
                "toroidal_world",
            ),
            edge(
                "east",
                EdgeKind::Slide,
                Axis::X,
                Side::Max,
                b,
                [-h, h],
                [0, -2 * b, 0],
                None,
                "toroidal_world",
            ),
            edge(
                "north",
                EdgeKind::HalfTurn,
                Axis::Z,
                Side::Min,
                -h,
                [-b, b],
                [2, 0, -b],
                Some([0, -h]),
                "wrap_engine",
            ),
            edge(
                "south",
                EdgeKind::HalfTurn,
                Axis::Z,
                Side::Max,
                h,
                [-b, b],
                [2, 0, b],
                Some([0, h]),
                "wrap_engine",
            ),
        ];
        let cone = |name: &str, images: Vec<[i64; 2]>| {
            let (lat, lon) = self.lat_lon(images[0][0] as f64, images[0][1] as f64);
            ConePoint {
                name: name.into(),
                cone_deg: 180.0,
                faces: None,
                cube: None,
                lat_deg: lat,
                lon_deg: lon,
                land_distance_deg: None,
                reachable: None,
                images,
            }
        };
        ProjectionSpec {
            kind: ProjectionKind::PetroffGuyou,
            layout_version: LAYOUT_VERSION,
            hemisphere_blocks: Some(b as u32),
            lambda_c_deg: Some(self.lambda_c_deg),
            face_blocks: None,
            map: None,
            rotation: None,
            domain: self.domain(),
            faces: None,
            edges,
            cone_points: vec![
                cone("north-midpoint", vec![[0, -h]]),
                cone("north-corner", vec![[-b, -h], [b, -h]]),
                cone("south-midpoint", vec![[0, h]]),
                cone("south-corner", vec![[-b, h], [b, h]]),
            ],
            poles: vec![
                PoleSpec::corners("north", &[[-h, -h], [h, -h]]),
                PoleSpec::corners("south", &[[-h, h], [h, h]]),
            ],
            border,
        }
    }

    /// The world's grid of `r` blocks a cell (`r` divides b): W = 2b/r columns from x = −b, H =
    /// b/r + 1 rows from the north edge to the south edge inclusive, rows past them folded by the
    /// half-turns ([`crate::GridDesc::cell`]).
    pub fn grid(&self, name: &str, r: u32) -> crate::Result<crate::GridDesc> {
        let r64 = i64::from(r.max(1));
        if self.b % r64 != 0 {
            return Err(crate::FormatError::Invalid(format!(
                "hemisphere {} is not a multiple of r = {r}",
                self.b
            )));
        }
        Ok(crate::GridDesc {
            name: name.to_owned(),
            w: (2 * self.b / r64) as u32,
            h: (self.b / r64 + 1) as u32,
            r,
            wraps_x: true,
            origin_block_x: -self.b,
            origin_block_z: -self.b / 2,
            z_edges: crate::raster::ZEdges::HalfTurn,
        })
    }

    /// The local Jacobian of the map at the point (x, z): the planet's displacement (blocks of
    /// the equal-area sphere of [`PetroffGuyou::radius`]) per block of x and per block of z, as
    /// [[east, north] per x, [east, north] per z] (central differences of `h` blocks, taken
    /// through the fold so they work at the edges).
    pub fn jacobian(&self, x: f64, z: f64, h: f64) -> [[f64; 2]; 2] {
        let v0 = self.unit(x, z);
        let (east, north) = local_frame(v0);
        let rr = self.radius();
        let d = |dx: f64, dz: f64| {
            let p = self.unit(x + dx, z + dz);
            let m = self.unit(x - dx, z - dz);
            let dv = [p[0] - m[0], p[1] - m[1], p[2] - m[2]];
            [
                dot3(dv, east) * rr / (2.0 * h),
                dot3(dv, north) * rr / (2.0 * h),
            ]
        };
        [d(h, 0.0), d(0.0, h)]
    }

    /// The unit vector of north in block space at the point (x, z), as (dx, dz).
    pub fn north(&self, x: f64, z: f64) -> (f64, f64) {
        let j = self.jacobian(x, z, 0.5);
        // Solve J·(dx, dz) = (0, 1) (north): dx·j[0] + dz·j[1] = (0, 1).
        let det = j[0][0] * j[1][1] - j[1][0] * j[0][1];
        if det == 0.0 || !det.is_finite() {
            return (0.0, -1.0);
        }
        let dx = -j[1][0] / det;
        let dz = j[0][0] / det;
        let n = dx.hypot(dz);
        (dx / n, dz / n)
    }
}

/// (latitude, longitude) of a unit vector, degrees, on the portable functions; longitude in
/// [0, 360), 0 at the poles.
pub fn lat_lon_of_unit(v: [f64; 3]) -> (f64, f64) {
    let h = (v[0] * v[0] + v[1] * v[1]).sqrt();
    let lat = pm::to_degrees(pm::atan2(v[2], h));
    let lon = if h == 0.0 {
        0.0
    } else {
        wrap_deg(pm::to_degrees(pm::atan2(v[1], v[0])))
    };
    (lat, lon)
}

/// The local east and north unit vectors at the unit vector `v` (east (0, 1, 0) at the poles).
pub fn local_frame(v: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let h = (v[0] * v[0] + v[1] * v[1]).sqrt();
    let east = if h > 1e-12 {
        [-v[1] / h, v[0] / h, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let north = [
        v[1] * east[2] - v[2] * east[1],
        v[2] * east[0] - v[0] * east[2],
        v[0] * east[1] - v[1] * east[0],
    ];
    // north = v × east (up × east): at (1, 0, 0) east is (0, 1, 0) and north (0, 0, 1).
    (east, north)
}

/// The frame of the world data's spatial indexes (river tiles, structure buckets, site index):
/// lattices of their own cells from `origin` over `span` blocks; x wraps with period `span.0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndexFrame {
    pub origin: (i64, i64),
    pub span: (i64, i64),
}

impl IndexFrame {
    /// The frame of a Lambert planet (and of the simulation's grids): (−C/2, −Zo) over (C, 2·Zo).
    pub fn lambert(c: u32) -> IndexFrame {
        let p = Projection::new(c);
        IndexFrame {
            origin: p.index_origin_blocks(),
            span: p.index_span_blocks(),
        }
    }
}

/// A world's projection: what its world data is laid out in (manifest `projection`).
// Copy on purpose (it is passed by value everywhere); the Petroff–Guyou variant's constants make it
// a few hundred bytes, which is fine for a value made once per world. The cube's net and map table
// are shared for the process's life ([`Cube::shared`], [`CubeMap::builtin_static`]), so its view is
// Copy too.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorldProjection {
    Lambert(Projection),
    PetroffGuyou(PetroffGuyou),
    /// A cube world (format 0.7). Its view's radius is in blocks of the equal-area sphere
    /// ([`Cube::radius`]), so its Jacobians are planet blocks per net block: times
    /// `scale.m_per_block_h` for metres.
    Cube(CubeView<'static>),
}

impl WorldProjection {
    /// The projection a layout block describes; `None` (a manifest without one) is Lambert on
    /// circumference `c`. Panics on a cube layout [`WorldProjection::try_from_spec`] refuses
    /// ([`crate::Manifest::read`] has refused those already).
    pub fn from_spec(spec: Option<&ProjectionSpec>, c: u32) -> WorldProjection {
        WorldProjection::try_from_spec(spec, c).unwrap_or_else(|e| panic!("{e}"))
    }

    /// [`WorldProjection::from_spec`], refusing a cube layout whose map this crate does not ship
    /// (by name and sha256), whose rotation is not one, or whose face side is not even.
    pub fn try_from_spec(spec: Option<&ProjectionSpec>, c: u32) -> crate::Result<WorldProjection> {
        Ok(match spec {
            Some(s) if s.kind == ProjectionKind::PetroffGuyou => {
                let b = s.hemisphere_blocks.unwrap_or(c / 2);
                WorldProjection::PetroffGuyou(PetroffGuyou::new(b, s.lambda_c_deg.unwrap_or(0.0)))
            }
            Some(s) if s.kind == ProjectionKind::Cube => {
                let bad = |m: String| crate::FormatError::Invalid(m);
                let face = s
                    .face_blocks
                    .ok_or_else(|| bad("cube layout without face_blocks".into()))?;
                if face < 2 || !face.is_multiple_of(2) {
                    return Err(bad(format!("cube face_blocks {face} is not even")));
                }
                let map = s
                    .map
                    .as_ref()
                    .ok_or_else(|| bad("cube layout without map".into()))?;
                let table = cube_map_named(&map.name, &map.sha256)?;
                let rows = s
                    .rotation
                    .ok_or_else(|| bad("cube layout without rotation".into()))?;
                let turn = CubeTurn::from_rows(rows).map_err(bad)?;
                WorldProjection::cube(face, table, turn)
            }
            _ => WorldProjection::Lambert(Projection::new(c)),
        })
    }

    /// The cube world of face side `s` drawn through `map` and placed by `turn`.
    pub fn cube(s: u32, map: &'static CubeMap, turn: CubeTurn) -> WorldProjection {
        let cube = Cube::shared(s);
        WorldProjection::Cube(CubeView {
            cube,
            map,
            turn,
            radius_m: cube.radius(),
        })
    }

    pub fn kind(&self) -> ProjectionKind {
        match self {
            WorldProjection::Lambert(_) => ProjectionKind::Lambert,
            WorldProjection::PetroffGuyou(_) => ProjectionKind::PetroffGuyou,
            WorldProjection::Cube(_) => ProjectionKind::Cube,
        }
    }

    /// The world's east–west period, blocks (`scale.circumference_blocks`).
    pub fn period(&self) -> f64 {
        match self {
            WorldProjection::Lambert(p) => p.circumference(),
            WorldProjection::PetroffGuyou(g) => g.period() as f64,
            WorldProjection::Cube(c) => c.cube.period() as f64,
        }
    }

    /// The equal-area sphere's radius, blocks.
    pub fn radius(&self) -> f64 {
        match self {
            WorldProjection::Lambert(p) => p.radius(),
            WorldProjection::PetroffGuyou(g) => g.radius(),
            WorldProjection::Cube(c) => c.cube.radius(),
        }
    }

    /// The unit vector of the point (x, z) (NaN past a cube's net corners).
    pub fn unit(&self, x: f64, z: f64) -> [f64; 3] {
        match self {
            WorldProjection::Lambert(p) => p.unit(x, z),
            WorldProjection::PetroffGuyou(g) => g.unit(x, z),
            WorldProjection::Cube(c) => c.unit(x, z).unwrap_or([f64::NAN; 3]),
        }
    }

    /// (latitude, longitude) of the point (x, z), degrees (NaN past a cube's net corners).
    pub fn lat_lon(&self, x: f64, z: f64) -> (f64, f64) {
        match self {
            WorldProjection::Lambert(p) => p.lat_lon(x, z),
            WorldProjection::PetroffGuyou(g) => g.lat_lon(x, z),
            WorldProjection::Cube(c) => c.lat_lon(x, z).unwrap_or((f64::NAN, f64::NAN)),
        }
    }

    /// The canonical point of a latitude and longitude, degrees.
    pub fn block_of(&self, lat_deg: f64, lon_deg: f64) -> (f64, f64) {
        match self {
            WorldProjection::Lambert(p) => p.block_of(lat_deg, lon_deg),
            WorldProjection::PetroffGuyou(g) => g.block_of(lat_deg, lon_deg),
            WorldProjection::Cube(c) => c.point_of_lat_lon(lat_deg, lon_deg),
        }
    }

    /// The canonical point of any point (x wrapped; Lambert clamps z to the poles; a cube folds
    /// through its net edges, NaN past the net's corners).
    pub fn canonical(&self, x: f64, z: f64) -> (f64, f64) {
        match self {
            WorldProjection::Lambert(p) => (p.wrap_x(x), p.clamp_z(z)),
            WorldProjection::PetroffGuyou(g) => {
                let f = g.fold(x, z);
                (f.x, f.z)
            }
            WorldProjection::Cube(c) => match c.cube.fold_point(x, z) {
                Some(p) => (p.x, p.z),
                None => (f64::NAN, f64::NAN),
            },
        }
    }

    /// The world's z range on the planet: [−R, R] (Lambert), [−b/2, b/2] (Petroff–Guyou) or
    /// the cube's window [−3s/2, 3s/2].
    pub fn z_range(&self) -> (f64, f64) {
        let d = match self {
            WorldProjection::Lambert(p) => return (-p.pole_z(), p.pole_z()),
            WorldProjection::PetroffGuyou(g) => g.domain(),
            WorldProjection::Cube(c) => c.cube.window(),
        };
        (d.z_min, d.z_max)
    }

    /// The world grid of `r` blocks a cell.
    pub fn grid(&self, name: &str, r: u32) -> crate::Result<crate::GridDesc> {
        match self {
            WorldProjection::Lambert(p) => {
                crate::GridDesc::for_planet(name, p.circumference() as u32, r)
            }
            WorldProjection::PetroffGuyou(g) => g.grid(name, r),
            WorldProjection::Cube(c) => crate::GridDesc::for_cube(name, c.cube.side() as u32, r),
        }
    }

    /// Block (x, z) of the first cell of the world data's spatial indexes (river tiles, structure
    /// buckets, site index): the grids' origin; a cube's window corner (−3s/2, −3s/2).
    pub fn index_origin_blocks(&self) -> (i64, i64) {
        match self {
            WorldProjection::Lambert(p) => p.index_origin_blocks(),
            WorldProjection::PetroffGuyou(g) => (-g.hemisphere(), -g.hemisphere() / 2),
            WorldProjection::Cube(c) => (-3 * c.cube.side() / 2, -3 * c.cube.side() / 2),
        }
    }

    /// Blocks the spatial indexes span: (east–west, north–south); a cube's window (4s, 3s).
    pub fn index_span_blocks(&self) -> (i64, i64) {
        match self {
            WorldProjection::Lambert(p) => p.index_span_blocks(),
            WorldProjection::PetroffGuyou(g) => (g.period(), g.hemisphere() + 1),
            WorldProjection::Cube(c) => (c.cube.period(), 3 * c.cube.side()),
        }
    }

    /// The spatial indexes' frame.
    pub fn index_frame(&self) -> IndexFrame {
        IndexFrame {
            origin: self.index_origin_blocks(),
            span: self.index_span_blocks(),
        }
    }

    /// The layout block (`manifest.json` `projection`); a cube's corners carry no land distance
    /// (the bake adds them, [`cube_spec`]).
    pub fn spec(&self, border: Option<BorderSpec>) -> ProjectionSpec {
        match self {
            WorldProjection::Lambert(p) => lambert_spec(p),
            WorldProjection::PetroffGuyou(g) => g.spec(border),
            WorldProjection::Cube(c) => cube_spec(c, border, None),
        }
    }
}

/// The shipped cube map table `name`, checked against `sha256` (lowercase hex). The refusal is
/// FORMAT.md's: `world data needs cube map <name> (sha256 <hash>); this Deep Time has <names>`.
pub fn cube_map_named(name: &str, sha256: &str) -> crate::Result<&'static CubeMap> {
    match CubeMap::builtin_static(name) {
        Some(m) if m.sha256_hex() == sha256.to_ascii_lowercase() => Ok(m),
        _ => Err(crate::FormatError::Invalid(format!(
            "world data needs cube map {name} (sha256 {sha256}); this Deep Time has {}",
            CubeMap::builtin_names()
                .iter()
                .map(|n| {
                    let m = CubeMap::builtin_static(n).expect("a builtin name");
                    format!("{n} (sha256 {})", m.sha256_hex())
                })
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

/// The border of a cube world: every turning net edge stopped `inset_blocks` inside its line
/// (FORMAT.md "The cube world's layout", `border`).
pub fn cube_border(cube: &Cube, inset_blocks: f64) -> BorderSpec {
    BorderSpec {
        kind: "edge_stop".into(),
        edges: Vec::new(),
        inset_blocks,
        temporary: true,
        segments: cube
            .edges()
            .iter()
            .filter(|e| e.motion != EdgeMotion::Slide)
            .map(|e| e.name.clone())
            .collect(),
    }
}

/// The cube-frame direction of a frame vector (`+X`, `-Z`, ...).
fn axis_name(v: [f64; 3]) -> String {
    let k = (0..3)
        .max_by(|&a, &b| v[a].abs().total_cmp(&v[b].abs()))
        .unwrap_or(0);
    format!(
        "{}{}",
        if v[k] < 0.0 { '-' } else { '+' },
        ['X', 'Y', 'Z'][k]
    )
}

/// The layout block of the cube world `view` (FORMAT.md "The cube world's layout"): the net, its
/// map and turn, the faces, the 14 edges, the 8 corners (with `land_distance_deg[k]` in corner
/// order when given) and the two poles, and `border`.
pub fn cube_spec(
    view: &CubeView,
    border: Option<BorderSpec>,
    land_distance_deg: Option<[f64; 8]>,
) -> ProjectionSpec {
    let cube = view.cube;
    let s = cube.side();
    let faces = Face::ALL
        .iter()
        .map(|&f| {
            let (ox, oz) = cube.face_origin(f);
            let (lat, lon) = lat_lon_of_unit(view.unit_on_face(f, 0.0, 0.0));
            let fr = f.frame();
            FaceSpec {
                id: f.name().into(),
                index: f.index() as u32,
                origin: [ox, oz],
                frame: [axis_name(fr[0]), axis_name(fr[1]), axis_name(fr[2])],
                centre_lat_deg: lat,
                centre_lon_deg: lon,
            }
        })
        .collect();
    let edges = cube
        .edges()
        .iter()
        .map(|e| EdgeSpec {
            name: e.name.clone(),
            seam: Some(e.seam.name().into()),
            kind: match e.motion {
                EdgeMotion::Slide => EdgeKind::Slide,
                EdgeMotion::QuarterTurn => EdgeKind::QuarterTurn,
                EdgeMotion::HalfTurn => EdgeKind::HalfTurn,
            },
            axis: e.axis,
            side: e.beyond,
            coord: e.coord,
            span: e.span,
            iso: e.iso.as_array(),
            pivot: e.pivot,
            partner: Some(cube.edges()[e.partner].name.clone()),
            owner: e.owner.into(),
        })
        .collect();
    let cone_points = cube
        .corners()
        .into_iter()
        .enumerate()
        .map(|(k, c)| {
            let (lat, lon) = view.corner_lat_lon(c.cube);
            ConePoint {
                name: c.name.clone(),
                cone_deg: 270.0,
                faces: Some(c.faces.iter().map(|f| f.name().to_owned()).collect()),
                cube: Some(c.cube),
                lat_deg: lat,
                lon_deg: lon,
                land_distance_deg: land_distance_deg.map(|d| d[k]),
                reachable: Some(c.images.len() == 1),
                images: c.images,
            }
        })
        .collect();
    let pole = |name: &str, (face, u, v): (Face, f64, f64)| {
        let mut images = vec![cube.point_of_local(face, u, v)];
        // A pole on a net edge has the partner edge's image too.
        let (x, z) = images[0];
        for e in cube.edges() {
            let on_line = match e.axis {
                Axis::X => {
                    x == e.coord as f64 && (e.span[0] as f64..=e.span[1] as f64).contains(&z)
                }
                Axis::Z => {
                    z == e.coord as f64 && (e.span[0] as f64..=e.span[1] as f64).contains(&x)
                }
            };
            if on_line {
                let p = cube.edges()[e.partner].iso.point(x, z);
                if !images.contains(&p) {
                    images.push(p);
                }
            }
        }
        PoleSpec {
            name: name.into(),
            face: Some(face.name().into()),
            u: Some(u),
            v: Some(v),
            images: images.into_iter().map(|(x, z)| [x, z]).collect(),
        }
    };
    ProjectionSpec {
        kind: ProjectionKind::Cube,
        layout_version: CUBE_LAYOUT_VERSION,
        hemisphere_blocks: None,
        lambda_c_deg: None,
        face_blocks: Some(s as u32),
        map: Some(MapSpec {
            name: view.map.name().into(),
            sha256: view.map.sha256_hex(),
        }),
        rotation: Some(view.turn.rows()),
        domain: cube.window(),
        faces: Some(faces),
        edges,
        cone_points,
        poles: vec![
            pole("north", view.north_pole()),
            pole("south", view.south_pole()),
        ],
        border,
    }
}

/// The layout block of a Lambert world: the slide, no cone points, the poles outside the
/// domain's z range (the pole cap, not a border, ends the world there).
pub fn lambert_spec(p: &Projection) -> ProjectionSpec {
    let c = p.circumference() as i64;
    let zo = p.grid_half_extent(1);
    let slide = |name: &str, side: Side, coord: i64, tx: i64| EdgeSpec {
        name: name.into(),
        seam: None,
        kind: EdgeKind::Slide,
        axis: Axis::X,
        side,
        coord,
        span: [-zo, zo],
        iso: [0, tx, 0],
        pivot: None,
        partner: None,
        owner: "toroidal_world".into(),
    };
    ProjectionSpec {
        kind: ProjectionKind::Lambert,
        layout_version: LAYOUT_VERSION,
        hemisphere_blocks: None,
        lambda_c_deg: None,
        face_blocks: None,
        map: None,
        rotation: None,
        domain: Domain {
            x_min: -(c / 2) as f64,
            x_max: (c / 2) as f64,
            z_min: -p.pole_z(),
            z_max: p.pole_z(),
            period_x: None,
        },
        faces: None,
        edges: vec![
            slide("west", Side::Min, -(c / 2), c),
            slide("east", Side::Max, c / 2, -c),
        ],
        cone_points: Vec::new(),
        poles: vec![
            PoleSpec::corners("north", &[]),
            PoleSpec::corners("south", &[]),
        ],
        border: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GridDesc;

    /// SplitMix64, for property tests without a dependency.
    struct Rng(u64);
    impl Rng {
        fn f(&mut self) -> f64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    const CS: [u32; 3] = [4_096, 16_384, 32_768];

    #[test]
    fn fixed_points() {
        for c in CS {
            let p = Projection::new(c);
            let r = f64::from(c) / TAU;
            assert_eq!(p.radius(), r);
            assert_eq!(p.pole_z(), r);
            assert_eq!(p.lat_deg(0.0), 0.0);
            assert_eq!(p.lat_deg(-r), 90.0, "north is -z");
            assert_eq!(p.lat_deg(r), -90.0);
            assert_eq!(p.lat_deg(-2.0 * r), 90.0, "latitude clamps past the pole");
            assert!((p.z_of_lat(90.0) + r).abs() < 1e-9);
            assert!((p.z_of_lat(30.0) + r / 2.0).abs() < 1e-9, "sin 30° = 1/2");
            assert_eq!(
                p.lon_deg(-f64::from(c) / 2.0),
                0.0,
                "the seam is longitude 0"
            );
            assert_eq!(p.lon_deg(0.0), 180.0);
            // True scale on the equator: one block of x and of z is one block of planet there.
            let ns = p.great_circle_blocks((0.0, 0.0), (0.0, 1.0));
            let ew = p.great_circle_blocks((0.0, 0.0), (1.0, 0.0));
            assert!(
                (ns - 1.0).abs() < 1e-6 && (ew - 1.0).abs() < 1e-6,
                "{ns} {ew}"
            );
        }
        // The decision's 16k numbers: 5,215 blocks pole to pole, 60° at z = −2,258, the grids to
        // ±2,624.
        let p = Projection::new(16_384);
        assert!((2.0 * p.radius() - 5_215.19).abs() < 0.01);
        assert!((p.z_of_lat(60.0) + 2_258.2).abs() < 0.1);
        assert_eq!(p.grid_half_extent(16), 2_624);
        assert_eq!(p.grid_half_extent(2), 2_624);
        assert_eq!(p.grid_rows(16), 328);
        assert_eq!(Projection::new(32_768).grid_half_extent(32), 5_248);
    }

    #[test]
    fn prop_block_lat_lon_round_trip() {
        let mut g = Rng(0xB10C);
        for c in CS {
            let p = Projection::new(c);
            let (h, r) = (f64::from(c) / 2.0, p.radius());
            for _ in 0..20_000 {
                let x = -h + g.f() * f64::from(c);
                let z = -r + g.f() * 2.0 * r;
                let (lat, lon) = p.lat_lon(x, z);
                let (x2, z2) = p.block_of(lat, lon);
                assert!(p.wrap_x(x2 - x).abs() < 1e-9, "x {x} -> {x2}");
                assert!((z2 - z).abs() < 1e-8, "z {z} -> {z2}");
                // And back from (lat, lon) through the blocks.
                let lat = -90.0 + 180.0 * g.f();
                let lon = 360.0 * g.f();
                let (x, z) = p.block_of(lat, lon);
                let (lat2, lon2) = p.lat_lon(x, z);
                assert!((lat2 - lat).abs() < 1e-7, "lat {lat} -> {lat2}");
                let d = (lon2 - lon).rem_euclid(360.0);
                assert!(d.min(360.0 - d) < 1e-9);
                // The unit vector is the one of (lat, lon).
                let v = p.unit(x, z);
                let want = unit_of_lat_lon(lat, lon);
                assert!(
                    (0..3).all(|k| (v[k] - want[k]).abs() < 1e-7),
                    "{v:?} {want:?}"
                );
            }
        }
    }

    #[test]
    fn prop_raster_row_round_trip() {
        // Block z ↔ grid row ↔ latitude.
        let mut g = Rng(0x2A57);
        for c in CS {
            for r in [2u32, 8, 16] {
                let grid = GridDesc::for_planet("L", c, r).unwrap();
                let p = grid.projection();
                for _ in 0..5_000 {
                    let z = -p.radius() + g.f() * 2.0 * p.radius();
                    let v = grid.row_of_z(z);
                    assert!((grid.z_of_row(v) - z).abs() < 1e-9);
                    let j = v.floor() as u32;
                    let zj = grid.block_z(i64::from(j)) as f64;
                    assert_eq!(grid.lat_of_row(j), p.lat_deg(zj));
                }
            }
        }
    }

    #[test]
    fn every_cell_has_the_same_area_and_they_sum_to_the_sphere() {
        for c in CS {
            for r in [2u32, 4, 8, 16, 32] {
                let grid = GridDesc::for_planet("L", c, r).unwrap();
                let p = grid.projection();
                let m = 300.0;
                // The true area of each row's cells on a sphere of R·m metres: 2πR²·Δ(sin φ)
                // over the row's band, divided among its W cells (exact, not the helper's).
                let rm = p.radius() * m;
                let mut total = 0.0;
                let mut interior = Vec::new();
                for j in 0..grid.h {
                    let z = grid.block_z(i64::from(j)) as f64;
                    let lo = if j == 0 {
                        f64::NEG_INFINITY
                    } else {
                        z - f64::from(r) / 2.0
                    };
                    let hi = if j + 1 == grid.h {
                        f64::INFINITY
                    } else {
                        z + f64::from(r) / 2.0
                    };
                    let exact = TAU * rm * rm * (p.sin_lat(lo) - p.sin_lat(hi)) / f64::from(grid.w);
                    let helper = grid.cell_area_km2(j, m) * 1e6;
                    assert!(
                        (helper - exact).abs() <= 1e-9 * exact.max(1.0),
                        "row {j}: {helper} {exact}"
                    );
                    total += helper * f64::from(grid.w);
                    if grid.area_share(j) == 1.0 {
                        interior.push(exact);
                    }
                }
                let sphere = 2.0 * TAU * rm * rm;
                assert!(
                    (total / sphere - 1.0).abs() < 1e-12,
                    "C {c} r {r}: {total} vs {sphere}"
                );
                // Any two cells inside the sphere are equal (within rounding), (r·m)² each.
                let (lo, hi) = interior
                    .iter()
                    .fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
                assert!(hi / lo - 1.0 < 1e-9, "C {c} r {r}: {lo} .. {hi}");
                assert!((lo / (f64::from(r) * m).powi(2) - 1.0).abs() < 1e-9);
                // The row weights are the same shares.
                let w: f64 = grid.row_weights().iter().sum();
                assert!((w - 2.0 * p.radius() / f64::from(r)).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn the_rasters_metric_is_the_spheres() {
        // A short step on the raster, measured with the projection's local metric (east–west
        // scale_ew, north–south scale_ns blocks of planet per block), against the great circle.
        let mut g = Rng(0x6EA7);
        for c in CS {
            let p = Projection::new(c);
            for _ in 0..20_000 {
                let lat = -75.0 + 150.0 * g.f();
                let (x, z) = p.block_of(lat, 360.0 * g.f());
                let (dx, dz) = (8.0 * (g.f() - 0.5), 8.0 * (g.f() - 0.5));
                let zm = z + dz / 2.0;
                let metric = (dx * p.scale_ew(zm)).hypot(dz * p.scale_ns(zm));
                let gc = p.great_circle_blocks((x, z), (x + dx, z + dz));
                assert!(
                    (metric - gc).abs() <= 3e-3 * gc + 1e-6,
                    "C {c} lat {lat:.1}: metric {metric} gc {gc}"
                );
                let (gxx, gxz, gzz) = p.metric(x, zm);
                assert!(((gxx * gzz - gxz * gxz).sqrt() - p.area_scale(zm)).abs() < 1e-9);
            }
        }
        // The equirectangular metric (north–south unscaled) is not the sphere's here.
        let p = Projection::new(16_384);
        let z = p.z_of_lat(60.0);
        let gc = p.great_circle_blocks((0.0, z), (0.0, z + 4.0));
        assert!(
            (gc - 8.0).abs() < 0.05,
            "4 blocks of z at 60° are 8 of planet: {gc}"
        );
    }

    #[test]
    fn the_seam_is_continuous() {
        for c in CS {
            let p = Projection::new(c);
            let h = f64::from(c) / 2.0;
            for lat in [0.0, 30.0, -60.0, 79.0] {
                let z = p.z_of_lat(lat);
                let one = p.great_circle_blocks((h - 1.0, z), (-h, z));
                let inside = p.great_circle_blocks((-1.0, z), (0.0, z));
                assert!((one - inside).abs() < 1e-9, "lat {lat}: {one} {inside}");
                assert!(p.great_circle_blocks((h, z), (-h, z)) < 1e-6);
            }
        }
    }

    #[test]
    fn pictures_span_pole_to_pole() {
        let p = Projection::new(16_384);
        assert_eq!(Projection::picture_height(1024), 326);
        assert_eq!(p.lat_deg(p.picture_z(0.0, 326)), 90.0);
        assert_eq!(p.lat_deg(p.picture_z(326.0, 326)), -90.0);
        assert!(p.lat_deg(p.picture_z(163.0, 326)).abs() < 1e-12);
        for y in [0.0, 17.5, 200.0] {
            assert!((p.picture_y(p.picture_z(y, 326), 326) - y).abs() < 1e-9);
            let lat = p.lat_deg(p.picture_z(y, 326));
            assert!((Projection::picture_lat_deg(y, 326) - lat).abs() < 1e-9);
            assert!((Projection::picture_y_of_lat(lat, 326) - y).abs() < 1e-9);
        }
        // Equal rows of a picture are equal bands of sin φ.
        let s = |y: f64| p.sin_lat(p.picture_z(y, 326));
        assert!(((s(10.0) - s(11.0)) - (s(300.0) - s(301.0))).abs() < 1e-12);
        assert_eq!(Projection::picture_lon_deg(256.0, 1024), 90.0);
        assert_eq!(Projection::picture_x_of_lon(450.0, 1024), 256.0);
    }

    #[test]
    fn quantum_and_extent() {
        assert_eq!(Projection::new(16_384).quantum(), 64);
        assert_eq!(Projection::new(32_768).quantum(), 64);
        // A tiny test planet: R = 10.2, so the quantum shrinks to 4.
        assert_eq!(Projection::new(64).quantum(), 4);
        let p = Projection::new(64);
        assert_eq!(p.grid_half_extent(4), 12);
        assert_eq!(p.grid_half_extent(2), 12);
        for c in CS {
            let p = Projection::new(c);
            for r in [1u32, 2, 4, 8, 16, 32, 64] {
                let zo = p.grid_half_extent(r);
                assert!(zo as f64 >= p.radius() && zo % i64::from(r) == 0);
                assert!((zo as f64) < p.radius() + 64.0f64.max(f64::from(r)));
            }
            let (ew, ns) = p.extent();
            assert_eq!(ew, f64::from(c));
            assert_eq!(ns, 2.0 * p.radius());
            assert!(p.on_planet(0.0, p.pole_z()) && !p.on_planet(0.0, p.pole_z() + 1.0));
            assert!(!p.inside_poles(-p.pole_z()) && p.inside_poles(0.0));
            assert_eq!(p.clamp_z(1e9), p.pole_z());
            // The grids and the spatial indexes share their origin, which covers the planet.
            let grid = GridDesc::for_planet("L", c, 16).unwrap();
            assert_eq!(
                p.index_origin_blocks(),
                (grid.origin_block_x, grid.origin_block_z)
            );
            assert_eq!(
                p.index_span_blocks(),
                (
                    i64::from(grid.w) * i64::from(grid.r),
                    i64::from(grid.h) * i64::from(grid.r)
                )
            );
            assert!(p.index_span().1 >= ns);
            assert_eq!(p.north(0.0, 0.0), (0.0, -1.0));
            assert_eq!(p.band_share(-1e9, 1e9) * 2e9, 2.0 * p.radius());
        }
    }

    #[test]
    fn the_neighbour_rule_wraps_columns_and_stops_at_the_rows() {
        assert_eq!(neighbour(8, 4, true, (7, 1), (1, 0)), Some((0, 1)));
        assert_eq!(neighbour(8, 4, true, (0, 1), (-1, -1)), Some((7, 0)));
        assert_eq!(neighbour(8, 4, false, (7, 1), (1, 0)), None);
        assert_eq!(neighbour(8, 4, true, (3, 0), (0, -1)), None);
        assert_eq!(neighbour(8, 4, true, (3, 3), (1, 1)), None);
        assert_eq!(wrap_col(-1, 8), 7);
        assert_eq!(clamp_row(9, 4), 3);
        let grid = GridDesc::for_planet("L", 4_096, 16).unwrap();
        assert_eq!(grid.wrap_i(-1), grid.w - 1);
        assert_eq!(grid.clamp_j(i64::from(grid.h) + 5), grid.h - 1);
    }
}

/// Petroff–Guyou: round trips, equal area, seams, folds and landmarks (ledger: SCL-086).
#[cfg(test)]
mod petroff_guyou_tests {
    use super::*;

    /// SplitMix64, for property tests without a dependency.
    struct Rng(u64);
    impl Rng {
        fn f(&mut self) -> f64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    fn angle(a: [f64; 3], b: [f64; 3]) -> f64 {
        let c = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ];
        let s = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        s.atan2(a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
    }

    fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
        [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
    }

    fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }

    fn norm(a: [f64; 3]) -> f64 {
        (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
    }

    /// The hemisphere sides of the 16k and 32k worlds and a small test world, with a λc each.
    const WORLDS: [(u32, f64); 3] = [(6_656, 31.0), (13_056, 200.25), (1_536, 0.0)];

    /// Exact inputs on the lines where Petroff's forward is degenerate (ledger: SCL-083), for the
    /// round trip below and the parity fixture: the equator every `step` degrees of longitude; the
    /// meridians λc, λc ± 90° and λc + 180° (the tiles' midlines and edges) every `step` degrees of
    /// latitude; the four cone points; the eight 67.5° vertices of Petroff's square (the equator at
    /// λc ± 22.5° and λc + 180° ± 22.5°, latitude ±22.5° on λc and λc + 180°); the two tile centres
    /// and the poles.
    pub(super) fn diagonal_inputs(lc: f64, step: f64) -> Vec<(f64, f64)> {
        let mut p = Vec::new();
        let n = (360.0 / step).round() as i32;
        for k in 0..n {
            p.push((0.0, f64::from(k) * step));
        }
        let m = (90.0 / step).round() as i32;
        for k in 1 - m..m {
            for d in [0.0, 90.0, 180.0, 270.0] {
                p.push((f64::from(k) * step, wrap_deg(lc + d)));
            }
        }
        for lat in [45.0, -45.0] {
            for d in [90.0, 270.0] {
                p.push((lat, wrap_deg(lc + d)));
            }
        }
        for d in [0.0, 180.0] {
            for o in [22.5, -22.5] {
                p.push((0.0, wrap_deg(lc + d + o)));
                p.push((o, wrap_deg(lc + d)));
            }
            p.push((0.0, wrap_deg(lc + d)));
        }
        p.push((90.0, 0.0));
        p.push((-90.0, 0.0));
        p
    }

    #[test]
    fn exact_inputs_on_the_diagonals_round_trip() {
        // Sphere → block → sphere on the forward's degenerate lines (SCL-083). Before 2026-10-20
        // the forward lost its last angle there: up to 1.6° off on the equator and 10.5° at the
        // cone points of the play world (6656, λc 146), whose cone points clamped to (616, −3328).
        for (b, lc) in WORLDS.into_iter().chain([(6_656, 146.0), (6_656, 31.25)]) {
            let g = PetroffGuyou::new(b, lc);
            let (bf, h) = (f64::from(b), f64::from(b) / 2.0);
            let pts = diagonal_inputs(lc, 0.1);
            let mut worst: f64 = 0.0;
            for &(lat, lon) in &pts {
                let (x, z) = g.block_of(lat, lon);
                assert!(
                    (-bf..bf).contains(&x) && z.abs() <= h,
                    "b {b} λc {lc}: ({lat}, {lon}) -> ({x}, {z})"
                );
                let d = angle(g.unit(x, z), unit_of_lat_lon(lat, lon)).to_degrees();
                worst = worst.max(d);
                assert!(
                    d < 1e-5,
                    "b {b} λc {lc}: ({lat}, {lon}) -> ({x}, {z}) is {d}° off"
                );
                if lat == 0.0 {
                    assert!(z.abs() < 1e-6, "b {b} λc {lc}: equator at {lon} -> z {z}");
                }
            }
            eprintln!(
                "b {b} λc {lc}: {} exact inputs, worst round trip {worst:e}°",
                pts.len()
            );
        }
    }

    #[test]
    fn the_sizes_follow_the_planet() {
        assert_eq!(PetroffGuyou::hemisphere_for(16_384), 6_656);
        assert_eq!(PetroffGuyou::hemisphere_for(32_768), 13_056);
        assert_eq!(PetroffGuyou::hemisphere_for(4_096), 1_536);
        for c in [1_024u32, 4_096, 16_384, 32_768] {
            let b = PetroffGuyou::hemisphere_for(c);
            assert_eq!(b % HEMISPHERE_QUANTUM, 0);
            assert_eq!((2 * b) % 512, 0, "the x period is whole region files");
            // Within half a quantum of the equal-area side C/√(2π).
            let exact = f64::from(c) / (2.0 * std::f64::consts::PI).sqrt();
            assert!(
                (f64::from(b) - exact).abs() <= 128.0,
                "C {c}: b {b} vs {exact}"
            );
        }
    }

    #[test]
    fn the_cube_face_follows_the_planet() {
        // docs/plans/cube-world-plan.md W1: 4,096, 16,384, 32,768 give 1,024, 3,840, 7,424.
        assert_eq!(face_for(4_096), 1_024);
        assert_eq!(face_for(16_384), 3_840);
        assert_eq!(face_for(32_768), 7_424);
        for c in [1_024u32, 4_096, 16_384, 32_768] {
            let s = face_for(c);
            assert_eq!(s % FACE_QUANTUM, 0);
            assert_eq!((4 * s) % 512, 0, "the x period is whole region files");
            assert_eq!((s / 2) % 16, 0, "the turn pivots sit on chunk corners");
            let exact = f64::from(c) / (6.0 * std::f64::consts::PI).sqrt();
            assert!(
                (f64::from(s) - exact).abs() <= 128.0,
                "C {c}: s {s} vs {exact}"
            );
        }
    }

    #[test]
    fn landmarks_and_orientation() {
        for (b, lc) in WORLDS {
            let g = PetroffGuyou::new(b, lc);
            let (bf, h) = (f64::from(b), f64::from(b) / 2.0);
            // The western hemisphere's centre is (0°, λc); the eastern one's (0°, λc + 180°).
            let (lat, lon) = g.lat_lon(-h, 0.0);
            assert!(lat.abs() < 1e-9, "{lat}");
            assert!(
                (wrap_deg(lon - lc + 180.0) - 180.0).abs() < 1e-9,
                "{lon} vs {lc}"
            );
            let (lat, lon) = g.lat_lon(h, 0.0);
            assert!(lat.abs() < 1e-9);
            assert!((wrap_deg(lon - lc) - 180.0).abs() < 1e-9, "{lon}");
            // The poles on the long edges at x = ∓b/2, each twice.
            for x in [-h, h] {
                assert!(
                    (g.lat_lon(x, -h).0 - 90.0).abs() < 1e-9,
                    "north pole at ({x}, {})",
                    -h
                );
                assert!(
                    (g.lat_lon(x, h).0 + 90.0).abs() < 1e-9,
                    "south pole at ({x}, {h})"
                );
            }
            // The equator is the line z = 0.
            for k in 0..100 {
                let x = -bf + 2.0 * bf * f64::from(k) / 100.0;
                assert!(g.lat_lon(x, 0.0).0.abs() < 1e-6, "x {x}");
            }
            // East is +x and north −z along the equator; not mirrored anywhere (the map keeps
            // orientation: (∂v/∂x × ∂v/∂z) points inwards, as east × south does).
            let (_, l0) = g.lat_lon(-h, 0.0);
            let (_, l1) = g.lat_lon(-h + 1.0, 0.0);
            assert!(wrap_deg(l1 - l0 + 180.0) - 180.0 > 0.0, "east is +x");
            assert!(g.lat_lon(-h, -1.0).0 > 0.0, "north is −z");
            let mut rng = Rng(u64::from(b));
            for _ in 0..2_000 {
                let (x, z) = ((rng.f() - 0.5) * 2.0 * bf, (rng.f() - 0.5) * bf);
                let v = g.unit(x, z);
                let vx = sub(g.unit(x + 0.01, z), g.unit(x - 0.01, z));
                let vz = sub(g.unit(x, z + 0.01), g.unit(x, z - 0.01));
                let n = cross(vx, vz);
                assert!(
                    n[0] * v[0] + n[1] * v[1] + n[2] * v[2] < 0.0,
                    "orientation at ({x}, {z})"
                );
            }
            // North along the equator is (0, −1).
            let (nx, nz) = g.north(-h, 0.0);
            assert!(nx.abs() < 1e-6 && (nz + 1.0).abs() < 1e-6, "{nx} {nz}");
        }
    }

    #[test]
    fn round_trips() {
        for (b, lc) in WORLDS {
            let g = PetroffGuyou::new(b, lc);
            let (bf, h) = (f64::from(b), f64::from(b) / 2.0);
            let mut rng = Rng(0xC0DE ^ u64::from(b));
            let mut worst_s: f64 = 0.0;
            let mut worst_b: f64 = 0.0;
            for _ in 0..100_000 {
                // Sphere → block → sphere.
                let lat = (2.0 * rng.f() - 1.0).asin().to_degrees();
                let lon = 360.0 * rng.f();
                let (x, z) = g.block_of(lat, lon);
                assert!(
                    (-bf..bf).contains(&x) && z.abs() <= h,
                    "({lat}, {lon}) -> ({x}, {z})"
                );
                let d = angle(g.unit(x, z), unit_of_lat_lon(lat, lon));
                worst_s = worst_s.max(d);
                assert!(d < 1e-8, "({lat}, {lon}) -> ({x}, {z}): {d} rad");
                // Block → sphere → block.
                let (x, z) = ((rng.f() - 0.5) * 2.0 * bf, (rng.f() - 0.5) * bf);
                let (x2, z2) = g.point_of_unit(g.unit(x, z));
                let e = g.wrap_x(x2 - x).abs().max((z2 - z).abs());
                worst_b = worst_b.max(e);
                assert!(e < 1e-6, "({x}, {z}) -> ({x2}, {z2})");
            }
            eprintln!("b {b}: worst sphere round trip {worst_s:e} rad, block {worst_b:e} blocks");
        }
    }

    #[test]
    fn equal_area() {
        // The area a block stands for, from the Jacobian at random points away from the four cone
        // points: second-order one-sided differences (step 1/20 block) on each side of the point, the
        // best of the four combinations (Petroff's creases, slope jumps of up to 9 % on the tiles'
        // midlines and diagonals, spoil a difference that straddles one), and the sum of the
        // spherical quads of a whole grid against 4π.
        let tri = |a: [f64; 3], b: [f64; 3], c: [f64; 3]| {
            // Van Oosterom–Strackee.
            let num = a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
                + a[2] * (b[0] * c[1] - b[1] * c[0]);
            let den = 1.0
                + (a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
                + (b[0] * c[0] + b[1] * c[1] + b[2] * c[2])
                + (c[0] * a[0] + c[1] * a[1] + c[2] * a[2]);
            2.0 * num.atan2(den)
        };
        for (b, lc) in WORLDS {
            let g = PetroffGuyou::new(b, lc);
            let (bf, h) = (f64::from(b), f64::from(b) / 2.0);
            let rr = g.radius();
            let mut rng = Rng(0xA4EA ^ u64::from(b));
            let (mut worst, mut n): (f64, usize) = (0.0, 0);
            for _ in 0..50_000 {
                let (x, z) = ((rng.f() - 0.5) * 2.0 * bf, (rng.f() - 0.5) * (bf - 1.0));
                let near_cone = [(0.0, -h), (0.0, h), (-bf, -h), (-bf, h), (bf, -h), (bf, h)]
                    .iter()
                    .any(|&(cx, cz)| (x - cx).hypot(z - cz) < 0.01 * bf);
                if near_cone {
                    continue;
                }
                let s = 0.05;
                let v0 = g.unit(x, z);
                let one_sided = |dx: f64, dz: f64| {
                    let v1 = g.unit(x + dx * s, z + dz * s);
                    let v2 = g.unit(x + 2.0 * dx * s, z + 2.0 * dz * s);
                    let k = (dx + dz) / (2.0 * s);
                    [
                        (-3.0 * v0[0] + 4.0 * v1[0] - v2[0]) * k,
                        (-3.0 * v0[1] + 4.0 * v1[1] - v2[1]) * k,
                        (-3.0 * v0[2] + 4.0 * v1[2] - v2[2]) * k,
                    ]
                };
                let (xp, xm) = (one_sided(1.0, 0.0), one_sided(-1.0, 0.0));
                let (zp, zm) = (one_sided(0.0, 1.0), one_sided(0.0, -1.0));
                let e = [(xp, zp), (xp, zm), (xm, zp), (xm, zm)]
                    .iter()
                    .map(|&(vx, vz)| (norm(cross(vx, vz)) * rr * rr - 1.0).abs())
                    .fold(f64::MAX, f64::min);
                worst = worst.max(e);
                n += 1;
            }
            eprintln!("b {b}: area per block within {worst:e} of 1 at {n} points");
            assert!(worst < 1e-5, "b {b}: area off by {worst}");
            // The whole world covers the sphere once: spherical quads of a 256 × 128 grid.
            let (nx, nz) = (256, 128);
            let mut total = 0.0;
            let mut cell_areas = Vec::new();
            for j in 0..nz {
                for i in 0..nx {
                    let x0 = -bf + 2.0 * bf * f64::from(i) / f64::from(nx);
                    let x1 = -bf + 2.0 * bf * f64::from(i + 1) / f64::from(nx);
                    let z0 = -h + bf * f64::from(j) / f64::from(nz);
                    let z1 = -h + bf * f64::from(j + 1) / f64::from(nz);
                    // Subdivide each cell 4 × 4 so the quads' straight edges follow the map.
                    let mut a = 0.0;
                    for sj in 0..4 {
                        for si in 0..4 {
                            let xa = x0 + (x1 - x0) * f64::from(si) / 4.0;
                            let xb = x0 + (x1 - x0) * f64::from(si + 1) / 4.0;
                            let za = z0 + (z1 - z0) * f64::from(sj) / 4.0;
                            let zb = z0 + (z1 - z0) * f64::from(sj + 1) / 4.0;
                            // Corners in the cell's own tile: nudge off the tile line x = 0.
                            let u = |x: f64, z: f64| g.unit_unfolded(x, z);
                            let (p00, p10, p11, p01) = (u(xa, za), u(xb, za), u(xb, zb), u(xa, zb));
                            a += -(tri(p00, p10, p11) + tri(p00, p11, p01));
                        }
                    }
                    // Leave out the cells next to a cone point (the quads' straight edges cannot
                    // follow the map there).
                    let (cx, cz) = ((x0 + x1) / 2.0, (z0 + z1) / 2.0);
                    let near = [(0.0, -h), (0.0, h), (-bf, -h), (-bf, h), (bf, -h), (bf, h)]
                        .iter()
                        .any(|&(px, pz)| {
                            (cx - px).abs() < 3.0 * (x1 - x0) && (cz - pz).abs() < 3.0 * (z1 - z0)
                        });
                    if !near {
                        cell_areas.push(a);
                    }
                    total += a;
                }
            }
            let sphere = 4.0 * std::f64::consts::PI;
            assert!((total / sphere - 1.0).abs() < 1e-9, "b {b}: {total} vs 4π");
            let mean = sphere / f64::from(nx * nz);
            let spread = cell_areas
                .iter()
                .map(|a| (a / mean - 1.0).abs())
                .fold(0.0f64, f64::max);
            eprintln!("b {b}: grid cells within {spread:e} of the mean area");
            assert!(spread < 5e-3, "b {b}: a cell off by {spread}");
        }
    }

    #[test]
    fn every_glued_edge_maps_the_same_sphere_points() {
        for (b, lc) in WORLDS {
            let g = PetroffGuyou::new(b, lc);
            let (bf, h) = (f64::from(b), f64::from(b) / 2.0);
            let mut worst: f64 = 0.0;
            for k in 0..=4_000 {
                let t = f64::from(k) / 4_000.0;
                // The slide: x = −b and x = b (drawn from the western and eastern tiles).
                let z = -h + bf * t;
                worst = worst.max(angle(g.unit_unfolded(-bf, z), g.unit_unfolded(bf, z)));
                // The half-turns: (x, ∓b/2) and (−x, ∓b/2).
                let x = -bf + 2.0 * bf * t;
                worst = worst.max(angle(g.unit_unfolded(x, -h), g.unit_unfolded(-x, -h)));
                worst = worst.max(angle(g.unit_unfolded(x, h), g.unit_unfolded(-x, h)));
                // The tile line x = 0 inside the world: both tiles agree on it.
                let z = -h + bf * t;
                worst = worst.max(angle(g.unit_unfolded(-1e-12, z), g.unit_unfolded(1e-12, z)));
            }
            assert!(worst <= 1e-8, "b {b}: seam closure {worst} rad");
            eprintln!("b {b}: seams close to {worst:e} rad");
        }
    }

    #[test]
    fn the_map_continues_smoothly_through_every_seam() {
        // Beyond an edge the fold gives the glued inside; the planet points there continue the
        // inside's run with no kink: v(p − d·n) − 2·v(p) + v(fold(p + d·n)) is second order in d.
        for (b, lc) in WORLDS {
            let g = PetroffGuyou::new(b, lc);
            let (bf, h) = (f64::from(b), f64::from(b) / 2.0);
            let d = 0.5;
            let mut worst: f64 = 0.0;
            for k in 1..400 {
                let t = f64::from(k) / 400.0;
                let x = -bf + 2.0 * bf * t;
                // Skip the cone points (x = 0, ±b) on the long edges.
                let cone = (x.abs() < 0.02 * bf) || (bf - x.abs() < 0.02 * bf);
                let z = -h + bf * t;
                let corner = h - z.abs() < 0.02 * bf;
                let mut pairs = Vec::new();
                if !corner {
                    pairs.push(((-bf + d, z), (-bf - d, z)));
                    pairs.push(((bf - d, z), (bf + d, z)));
                }
                if !cone {
                    pairs.push(((x, -h + d), (x, -h - d)));
                    pairs.push(((x, h - d), (x, h + d)));
                }
                for ((xi, zi), (xo, zo)) in pairs {
                    let on = g.unit((xi + xo) / 2.0, (zi + zo) / 2.0);
                    let inside = g.unit(xi, zi);
                    let outside = g.unit(xo, zo);
                    let second = [
                        inside[0] - 2.0 * on[0] + outside[0],
                        inside[1] - 2.0 * on[1] + outside[1],
                        inside[2] - 2.0 * on[2] + outside[2],
                    ];
                    let first = norm(sub(outside, inside));
                    worst = worst.max(norm(second) / first.max(1e-300));
                }
            }
            // A kink would make the second difference as large as the first (ratio ~ 1).
            assert!(worst < 1e-2, "b {b}: second/first difference {worst}");
            eprintln!("b {b}: second over first difference across seams ≤ {worst:e}");
        }
    }

    #[test]
    fn folds_match_the_wrap_engine() {
        let g = PetroffGuyou::new(6_656, 31.0);
        let (b, h) = (6_656i64, 3_328i64);
        // CellIsometry.halfTurnAbout(0, −b/2): (x, z) → (−x − 1, −b − z − 1).
        for (x, z) in [(10, -h - 1), (-6_000, -h - 40), (0, -h - 1), (-1, -h - 16)] {
            let (fx, fz, n) = g.fold_cell(x, z);
            assert_eq!(
                (fx, fz, n),
                (g_wrap(-x - 1, b), -b - z - 1, 1),
                "({x}, {z})"
            );
        }
        for (x, z) in [(10, h), (-6_000, h + 40)] {
            let (fx, fz, n) = g.fold_cell(x, z);
            assert_eq!((fx, fz, n), (g_wrap(-x - 1, b), b - z - 1, 1));
        }
        assert_eq!(g.fold_cell(b, 0), (-b, 0, 0), "the slide");
        assert_eq!(g.fold_cell(-b - 1, 5), (b - 1, 5, 0));
        // The point fold and the sphere agree: a point beyond is the planet point of its image.
        let mut rng = Rng(77);
        for _ in 0..2_000 {
            let x = (rng.f() - 0.5) * 2.0 * b as f64;
            let d = rng.f() * 3_000.0;
            let f = g.fold(x, -(h as f64) - d);
            assert!((f.x - g.wrap_x(-x)).abs() < 1e-9 && (f.z - (-(h as f64) + d)).abs() < 1e-9);
            assert_eq!(f.half_turns, 1);
        }
    }

    fn g_wrap(x: i64, b: i64) -> i64 {
        (x + b).rem_euclid(2 * b) - b
    }

    #[test]
    fn cone_points_sit_at_45_degrees_on_the_boundary_meridians() {
        for (b, lc) in WORLDS {
            let g = PetroffGuyou::new(b, lc);
            let spec = g.spec(None);
            assert_eq!(spec.cone_points.len(), 4);
            for c in &spec.cone_points {
                assert!(
                    (c.lat_deg.abs() - 45.0).abs() < 1e-6,
                    "{}: {} {}",
                    c.name,
                    c.lat_deg,
                    c.lon_deg
                );
                let dl = wrap_deg(c.lon_deg - lc);
                assert!(
                    (dl - 90.0).abs() < 1e-6 || (dl - 270.0).abs() < 1e-6,
                    "{}: lon {} vs λc {lc}",
                    c.name,
                    c.lon_deg
                );
                for im in &c.images[1..] {
                    let d = angle(
                        g.unit(im[0] as f64, im[1] as f64),
                        g.unit(c.images[0][0] as f64, c.images[0][1] as f64),
                    );
                    assert!(d < 1e-8);
                }
            }
            for p in &spec.poles {
                for im in &p.images {
                    let lat = g.lat_lon(im[0], im[1]).0;
                    assert!(
                        (lat.abs() - 90.0).abs() < 1e-9,
                        "{} at {im:?}: {lat}",
                        p.name
                    );
                }
            }
        }
    }
}

/// The cross-language parity fixture (`tests/data/projection_parity.json`): inputs and outputs of
/// the portable trigonometry, Petroff's square and two Petroff–Guyou worlds (random points and
/// exact inputs on the forward's degenerate lines) and the cube world (`"cube"`,
/// [`crate::cube_map::parity`]), every number as the hex of its bits. Java's `ProjectionParityTest` and the pages' `tools/viewer/projection-check.js`
/// read it and must give the same bits. `DEEPTIME_WRITE_PARITY=1 cargo test -p deeptime-format
/// parity_fixture` rewrites it after a deliberate change.
#[cfg(test)]
mod parity_fixture {
    use super::*;
    use crate::petroff::Petroff;

    fn hx(v: f64) -> String {
        format!("\"{:016x}\"", v.to_bits())
    }

    struct Rng(u64);
    impl Rng {
        fn f(&mut self) -> f64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    /// The fixture's text.
    pub(super) fn fixture() -> String {
        let mut g = Rng(0x0DE5_7001);
        let mut out = String::from(
            "{\n  \"about\": \"deeptime_format::projection parity fixture: inputs and outputs as f64 bit patterns (hex). Regenerate with DEEPTIME_WRITE_PARITY=1.\",\n",
        );
        // Portable trigonometry.
        out.push_str("  \"math\": [\n");
        let mut rows = Vec::new();
        for k in 0..240 {
            let x = (g.f() - 0.5) * if k % 3 == 0 { 40.0 } else { 4.0 };
            let y = (g.f() - 0.5) * 4.0;
            let u = g.f() * 2.0 - 1.0;
            rows.push(format!(
                "    {{\"x\": {}, \"y\": {}, \"u\": {}, \"sin\": {}, \"cos\": {}, \"tan\": {}, \"atan\": {}, \"atan2\": {}, \"asin\": {}, \"acos\": {}}}",
                hx(x),
                hx(y),
                hx(u),
                hx(pm::sin(x)),
                hx(pm::cos(x)),
                hx(pm::tan(x)),
                hx(pm::atan(x)),
                hx(pm::atan2(y, x)),
                hx(pm::asin(u)),
                hx(pm::acos(u))
            ));
        }
        out.push_str(&rows.join(",\n"));
        out.push_str("\n  ],\n");
        // Petroff's square.
        let p = Petroff::new();
        out.push_str("  \"petroff\": [\n");
        let mut rows = Vec::new();
        for _ in 0..200 {
            let (x, y) = (g.f() * 2.0 - 1.0, g.f() * 2.0 - 1.0);
            let (l, f) = p.inverse(x, y);
            let lam = g.f() * 2.0 * PI;
            let phi = pm::asin(g.f() * 2.0 - 1.0);
            let (fx, fy) = p.forward(lam, phi);
            rows.push(format!(
                "    {{\"x\": {}, \"y\": {}, \"lambda\": {}, \"phi\": {}, \"lam_in\": {}, \"phi_in\": {}, \"x_out\": {}, \"y_out\": {}}}",
                hx(x),
                hx(y),
                hx(l),
                hx(f),
                hx(lam),
                hx(phi),
                hx(fx),
                hx(fy)
            ));
        }
        out.push_str(&rows.join(",\n"));
        out.push_str("\n  ],\n");
        // Two worlds.
        out.push_str("  \"worlds\": [\n");
        let mut worlds = Vec::new();
        for (b, lc) in [(6_656u32, 31.25f64), (1_536, 0.0)] {
            let w = PetroffGuyou::new(b, lc);
            let (bf, h) = (f64::from(b), f64::from(b) / 2.0);
            let mut pts = Vec::new();
            for k in 0..200 {
                // Points inside, on the edges and a few past them (folded).
                let x = (g.f() - 0.5) * 2.0 * bf;
                let z = match k % 10 {
                    0 => -h,
                    1 => h,
                    2 => -h - g.f() * 50.0,
                    _ => (g.f() - 0.5) * bf,
                };
                let v = w.unit(x, z);
                let (lat, lon) = w.lat_lon(x, z);
                pts.push(format!(
                    "        {{\"x\": {}, \"z\": {}, \"v\": [{}, {}, {}], \"lat\": {}, \"lon\": {}}}",
                    hx(x),
                    hx(z),
                    hx(v[0]),
                    hx(v[1]),
                    hx(v[2]),
                    hx(lat),
                    hx(lon)
                ));
            }
            let mut inv = Vec::new();
            for _ in 0..200 {
                let lat = pm::to_degrees(pm::asin(g.f() * 2.0 - 1.0));
                let lon = g.f() * 360.0;
                let (x, z) = w.block_of(lat, lon);
                inv.push(format!(
                    "        {{\"lat\": {}, \"lon\": {}, \"x\": {}, \"z\": {}}}",
                    hx(lat),
                    hx(lon),
                    hx(x),
                    hx(z)
                ));
            }
            // Exact inputs on the forward's degenerate lines (SCL-083; no draws from `g`, so the
            // rows above keep their inputs).
            let mut exact = Vec::new();
            for (lat, lon) in super::petroff_guyou_tests::diagonal_inputs(lc, 2.5) {
                let (x, z) = w.block_of(lat, lon);
                exact.push(format!(
                    "        {{\"lat\": {}, \"lon\": {}, \"x\": {}, \"z\": {}}}",
                    hx(lat),
                    hx(lon),
                    hx(x),
                    hx(z)
                ));
            }
            worlds.push(format!(
                "    {{\"b\": {b}, \"lambda_c_deg\": {}, \"points\": [\n{}\n      ], \"inverse\": [\n{}\n      ], \"exact\": [\n{}\n      ]}}",
                hx(lc),
                pts.join(",\n"),
                inv.join(",\n"),
                exact.join(",\n")
            ));
        }
        out.push_str(&worlds.join(",\n"));
        // The cube world's geometry and maps (crate::cube_map::parity, plan M5).
        out.push_str("\n  ],\n");
        out.push_str(&crate::cube_map::parity::section());
        out.push_str("\n}\n");
        out
    }

    #[test]
    fn parity_fixture_is_current() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/projection_parity.json"
        );
        let text = fixture();
        if std::env::var_os("DEEPTIME_WRITE_PARITY").is_some() {
            std::fs::create_dir_all(std::path::Path::new(path).parent().unwrap()).unwrap();
            std::fs::write(path, &text).unwrap();
        }
        let on_disk = std::fs::read_to_string(path).expect("the parity fixture");
        assert!(
            on_disk == text,
            "{path} is stale: rerun with DEEPTIME_WRITE_PARITY=1 and commit it (Java and JS read it)"
        );
    }
}

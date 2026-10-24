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
// World layouts: the cube world, and the Lambert frame of the synthetic test worlds.
// ------------------------------------------------------------------------------------------------

/// Which projection a world uses (manifest `projection.kind`, FORMAT.md "The world's layout").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionKind {
    /// The simulation's own frame, Lambert's cylindrical equal-area grid, laid out as it is: the
    /// synthetic test worlds (`deeptime synth`, the deposit and river fixtures), which have one
    /// east–west slide and stop at their first and last rows. No planet is laid out this way: the
    /// cube is the only world (Ben, decisions log 2026-10-24, "Cube only"; cube plan §7.4).
    Lambert,
    /// The cube world: six faces in a cross, drawn through a map table and turned freely onto the
    /// planet (Ben, decisions log 2026-10-22; format 0.7, FORMAT.md "The cube world's layout").
    #[default]
    Cube,
}

impl ProjectionKind {
    /// The setting's text (`projection.kind`).
    pub fn name(self) -> &'static str {
        match self {
            ProjectionKind::Lambert => "lambert",
            ProjectionKind::Cube => "cube",
        }
    }

    /// The kind named `s`.
    pub fn parse(s: &str) -> Option<ProjectionKind> {
        match s {
            "lambert" => Some(ProjectionKind::Lambert),
            "cube" => Some(ProjectionKind::Cube),
            _ => None,
        }
    }
}

/// The version of the layout block's schema (`projection.layout_version`).
pub const LAYOUT_VERSION: u32 = 1;

/// The cube world's layout block version (`projection.layout_version` of a `cube` world).
pub const CUBE_LAYOUT_VERSION: u32 = 2;

/// The cube world's face side is a multiple of this many blocks: every grid's r (up to 256)
/// divides s, the east-west period 4s is a multiple of 512 (a region file, what `WorldDataRef` and
/// Toroidal World's loop take), and every turn pivot (s/2 from a face centre) sits on a chunk
/// corner (docs/plans/cube-world-plan.md §1.1).
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
    /// A half-turn about the centre of a void square of the cube's net (the edge is glued onto
    /// its partner).
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
    /// The world's angle round the point, degrees (270 at a cube's corner).
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
    /// The points that are the pole: any net point on a cube world; empty when the pole is
    /// outside the world (the Lambert frame).
    #[serde(serialize_with = "points_as_written")]
    pub images: Vec<[f64; 2]>,
}

impl PoleSpec {
    /// A pole whose images are block corners (the Lambert frame's: none).
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

/// Writes whole coordinates as integers (block corners) and the rest as floats.
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
// Copy on purpose (it is passed by value everywhere). The cube's net and map table are shared for
// the process's life ([`Cube::shared`], [`CubeMap::builtin_static`]), so its view is Copy too.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorldProjection {
    /// The simulation's Lambert frame as it is (the synthetic test worlds, [`ProjectionKind::Lambert`]).
    Lambert(Projection),
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
            WorldProjection::Cube(_) => ProjectionKind::Cube,
        }
    }

    /// The world's east–west period, blocks (`scale.circumference_blocks`).
    pub fn period(&self) -> f64 {
        match self {
            WorldProjection::Lambert(p) => p.circumference(),
            WorldProjection::Cube(c) => c.cube.period() as f64,
        }
    }

    /// The equal-area sphere's radius, blocks.
    pub fn radius(&self) -> f64 {
        match self {
            WorldProjection::Lambert(p) => p.radius(),
            WorldProjection::Cube(c) => c.cube.radius(),
        }
    }

    /// The unit vector of the point (x, z) (NaN past a cube's net corners).
    pub fn unit(&self, x: f64, z: f64) -> [f64; 3] {
        match self {
            WorldProjection::Lambert(p) => p.unit(x, z),
            WorldProjection::Cube(c) => c.unit(x, z).unwrap_or([f64::NAN; 3]),
        }
    }

    /// (latitude, longitude) of the point (x, z), degrees (NaN past a cube's net corners).
    pub fn lat_lon(&self, x: f64, z: f64) -> (f64, f64) {
        match self {
            WorldProjection::Lambert(p) => p.lat_lon(x, z),
            WorldProjection::Cube(c) => c.lat_lon(x, z).unwrap_or((f64::NAN, f64::NAN)),
        }
    }

    /// The canonical point of a latitude and longitude, degrees.
    pub fn block_of(&self, lat_deg: f64, lon_deg: f64) -> (f64, f64) {
        match self {
            WorldProjection::Lambert(p) => p.block_of(lat_deg, lon_deg),
            WorldProjection::Cube(c) => c.point_of_lat_lon(lat_deg, lon_deg),
        }
    }

    /// The canonical point of any point (x wrapped; Lambert clamps z to the poles; a cube folds
    /// through its net edges, NaN past the net's corners).
    pub fn canonical(&self, x: f64, z: f64) -> (f64, f64) {
        match self {
            WorldProjection::Lambert(p) => (p.wrap_x(x), p.clamp_z(z)),
            WorldProjection::Cube(c) => match c.cube.fold_point(x, z) {
                Some(p) => (p.x, p.z),
                None => (f64::NAN, f64::NAN),
            },
        }
    }

    /// The world's z range on the planet: [−R, R] (Lambert) or the cube's window [−3s/2, 3s/2].
    pub fn z_range(&self) -> (f64, f64) {
        match self {
            WorldProjection::Lambert(p) => (-p.pole_z(), p.pole_z()),
            WorldProjection::Cube(c) => {
                let d = c.cube.window();
                (d.z_min, d.z_max)
            }
        }
    }

    /// The world grid of `r` blocks a cell.
    pub fn grid(&self, name: &str, r: u32) -> crate::Result<crate::GridDesc> {
        match self {
            WorldProjection::Lambert(p) => {
                crate::GridDesc::for_planet(name, p.circumference() as u32, r)
            }
            WorldProjection::Cube(c) => crate::GridDesc::for_cube(name, c.cube.side() as u32, r),
        }
    }

    /// Block (x, z) of the first cell of the world data's spatial indexes (river tiles, structure
    /// buckets, site index): the grids' origin; a cube's window corner (−3s/2, −3s/2).
    pub fn index_origin_blocks(&self) -> (i64, i64) {
        match self {
            WorldProjection::Lambert(p) => p.index_origin_blocks(),
            WorldProjection::Cube(c) => (-3 * c.cube.side() / 2, -3 * c.cube.side() / 2),
        }
    }

    /// Blocks the spatial indexes span: (east–west, north–south); a cube's window (4s, 3s).
    pub fn index_span_blocks(&self) -> (i64, i64) {
        match self {
            WorldProjection::Lambert(p) => p.index_span_blocks(),
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

/// The layout block of the Lambert frame (the synthetic test worlds): the slide, no cone points,
/// the poles outside the domain's z range (the world's first and last rows end it there).
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
}

/// The cross-language parity fixture (`tests/data/projection_parity.json`): inputs and outputs of
/// the portable trigonometry and the cube world (`"cube"`, [`crate::cube_map::parity`]), every
/// number as the hex of its bits. Java's `ProjectionParityTest` and the pages' `tools/viewer/projection-check.js`
/// read it and must give the same bits. `DEEPTIME_WRITE_PARITY=1 cargo test -p deeptime-format
/// parity_fixture` rewrites it after a deliberate change.
#[cfg(test)]
mod parity_fixture {
    use super::*;

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
        // The cube world's geometry and maps (crate::cube_map::parity, plan M5).
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

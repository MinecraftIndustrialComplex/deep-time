// SPDX-License-Identifier: LGPL-3.0-or-later
//! Tiled raster layers (plan §5.3): cell types, grid and layer descriptors, in-memory rasters and
//! the sampling rules every reader shares (wrap in x, clamp in z, bilinear in block space).

use serde::{Deserialize, Serialize};

use crate::cube::{Cube, Face, cell_on};
use crate::error::{FormatError, Result};
use crate::projection::Projection;
use crate::strata::{ColumnHorizon, StoredHorizon};

/// Horizon `k` of one `horizon_stack` cell's bytes (K = `depth`, 7K bytes).
pub fn decode_horizon(cell: &[u8], depth: usize, k: usize) -> StoredHorizon {
    let t = 2 * k;
    let kt = 3 * depth + 2 * k;
    let kb = 5 * depth + 2 * k;
    StoredHorizon {
        top: i16::from_le_bytes([cell[t], cell[t + 1]]),
        class: cell[2 * depth + k],
        key: [
            u16::from_le_bytes([cell[kt], cell[kt + 1]]),
            u16::from_le_bytes([cell[kb], cell[kb + 1]]),
        ],
    }
}

/// The tile edge in cells (plan §5.3).
pub const TILE: u32 = 256;

/// A cell type. Scalars are little-endian; `horizon_stack` is the strata record of §5.7.4.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DType {
    U8,
    I8,
    U16,
    I16,
    U32,
    F32,
    /// `stack_depth` = K horizons per cell: K × i16 top elevation (1/16 block), K × u8 class
    /// index, K × u16 stratigraphic key top, K × u16 key base; 7K bytes (format 0.2). Unused
    /// horizons have top `i16::MIN`, class 255 and keys `u16::MAX` (`crate::strata`).
    HorizonStack,
}

impl DType {
    /// Bytes per cell of a scalar type; `None` for `horizon_stack` (it depends on K).
    pub fn scalar_bytes(self) -> Option<usize> {
        Some(match self {
            DType::U8 | DType::I8 => 1,
            DType::U16 | DType::I16 => 2,
            DType::U32 | DType::F32 => 4,
            DType::HorizonStack => return None,
        })
    }

    /// Decodes one little-endian scalar cell to f64 (the raw value, before scale and offset).
    pub fn decode_scalar(self, b: &[u8]) -> f64 {
        match self {
            DType::U8 => f64::from(b[0]),
            DType::I8 => f64::from(b[0] as i8),
            DType::U16 => f64::from(u16::from_le_bytes([b[0], b[1]])),
            DType::I16 => f64::from(i16::from_le_bytes([b[0], b[1]])),
            DType::U32 => f64::from(u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
            DType::F32 => f64::from(f32::from_le_bytes([b[0], b[1], b[2], b[3]])),
            DType::HorizonStack => f64::NAN,
        }
    }
}

/// How each tile is stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Codec {
    /// The tile's cells, little-endian, row-major.
    Raw,
    /// One LZ4 raw block (`lz4_flex::block`, no frame, no size prefix) per tile.
    Lz4Block,
}

/// What a row index past the first or the last row means (`z_edges`, FORMAT.md "Grids").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ZEdges {
    /// Rows clamp to [0, H − 1] (the simulation's grids and Lambert worlds: the rows stop at the
    /// poles).
    #[default]
    Clamp,
    /// Rows 0 and H − 1 lie on the world's north and south edges, each glued onto itself by a
    /// half-turn about its midpoint (block x = 0, which is column W/2): row −k is row k and
    /// row H − 1 + k is row H − 1 − k, both at column W − i (Petroff–Guyou worlds).
    HalfTurn,
    /// A cube world's six faces, n × n cells each, stacked F0 to S down the rows (W = n, H = 6n):
    /// an index off a face resolves through the face adjacency, and beyond two sides at once
    /// through the side it is further out of ([`GridDesc::face_cell`]; FORMAT.md "Cube grids").
    Cube,
}

/// A raster in block space (plan §5.2), laid out by the world's projection
/// ([`crate::projection`]). Cell (i, j) is the **point sample** at block
/// (origin_block_x + i·r, origin_block_z + j·r). On the simulation's grids and Lambert worlds
/// column 0 sits on the seam x = −C/2 and row 0 at z = −Zo, at or just past the north pole
/// ([`Projection::grid_half_extent`]); rows are uniform in sin φ; indices wrap in x (mod W) and
/// clamp in z to [0, H−1]. On a Petroff–Guyou world ([`crate::projection::PetroffGuyou::grid`])
/// column 0 sits on x = −b, row 0 on the north edge and row H − 1 on the south edge, and a row
/// past either folds through the edge's half-turn ([`ZEdges::HalfTurn`], [`GridDesc::cell`]).
///
/// A cube world's grid ([`GridDesc::for_cube`], [`ZEdges::Cube`]) is different: six planes of
/// n × n cells, one per face in the format's order, stacked down the rows; cell (i, j) of a face
/// is the **r × r square** of net blocks from the face's origin + (i·r, j·r), and its value sits
/// at the square's centre ([`GridDesc::cube_point`]). Its JSON form is
/// `{"name": "L3", "kind": "cube", "n": 1920, "r": 2}`; in memory W = n, H = 6n, no x wrap, and
/// the origin is F0's north-west corner (−s/2, −s/2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridDesc {
    /// The grid's role: `L1`, `L2`, `L3` (plan §5.2).
    pub name: String,
    /// Cells east–west (W = C / r).
    pub w: u32,
    /// Cells north–south (H = 2·Zo / r).
    pub h: u32,
    /// Blocks per cell.
    pub r: u32,
    /// Always true in v0: x wraps with period W·r = C.
    pub wraps_x: bool,
    /// Block x of cell column 0 (−C/2).
    pub origin_block_x: i64,
    /// Block z of cell row 0 (−Zo).
    pub origin_block_z: i64,
    /// What a row past the first or the last means (absent: clamp). Always written: the
    /// checkpoints encode grids with postcard, which cannot skip a field.
    pub z_edges: ZEdges,
}

/// A grid as every grid but a cube's is written (and as postcard writes every grid): the fields
/// in their order, `z_edges` always written.
#[derive(Serialize)]
struct PlainGridOut<'a> {
    name: &'a str,
    w: u32,
    h: u32,
    r: u32,
    wraps_x: bool,
    origin_block_x: i64,
    origin_block_z: i64,
    z_edges: ZEdges,
}

#[derive(Deserialize)]
struct PlainGridIn {
    name: String,
    w: u32,
    h: u32,
    r: u32,
    wraps_x: bool,
    origin_block_x: i64,
    origin_block_z: i64,
    #[serde(default)]
    z_edges: ZEdges,
}

/// A cube grid's JSON form.
#[derive(Serialize)]
struct CubeGridOut<'a> {
    name: &'a str,
    kind: &'a str,
    n: u32,
    r: u32,
}

/// Either JSON form, read.
#[derive(Deserialize)]
struct AnyGridIn {
    name: String,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    n: Option<u32>,
    #[serde(default)]
    w: Option<u32>,
    #[serde(default)]
    h: Option<u32>,
    r: u32,
    #[serde(default)]
    wraps_x: Option<bool>,
    #[serde(default)]
    origin_block_x: Option<i64>,
    #[serde(default)]
    origin_block_z: Option<i64>,
    #[serde(default)]
    z_edges: ZEdges,
}

impl Serialize for GridDesc {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> std::result::Result<S::Ok, S::Error> {
        if ser.is_human_readable() && self.z_edges == ZEdges::Cube {
            return CubeGridOut {
                name: &self.name,
                kind: "cube",
                n: self.w,
                r: self.r,
            }
            .serialize(ser);
        }
        PlainGridOut {
            name: &self.name,
            w: self.w,
            h: self.h,
            r: self.r,
            wraps_x: self.wraps_x,
            origin_block_x: self.origin_block_x,
            origin_block_z: self.origin_block_z,
            z_edges: self.z_edges,
        }
        .serialize(ser)
    }
}

impl<'de> Deserialize<'de> for GridDesc {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> std::result::Result<Self, D::Error> {
        use serde::de::Error;
        if !de.is_human_readable() {
            let g = PlainGridIn::deserialize(de)?;
            return Ok(GridDesc {
                name: g.name,
                w: g.w,
                h: g.h,
                r: g.r,
                wraps_x: g.wraps_x,
                origin_block_x: g.origin_block_x,
                origin_block_z: g.origin_block_z,
                z_edges: g.z_edges,
            });
        }
        let g = AnyGridIn::deserialize(de)?;
        match g.kind.as_deref() {
            Some("cube") => {
                let n = g.n.ok_or_else(|| D::Error::missing_field("n"))?;
                if n == 0 || g.r == 0 || !(u64::from(n) * u64::from(g.r)).is_multiple_of(2) {
                    return Err(D::Error::custom(format!(
                        "cube grid {}: n {n} × r {} is not an even face side",
                        g.name, g.r
                    )));
                }
                GridDesc::for_cube(&g.name, n * g.r, g.r).map_err(D::Error::custom)
            }
            Some(k) => Err(D::Error::custom(format!(
                "grid {}: unknown kind {k}",
                g.name
            ))),
            None => Ok(GridDesc {
                name: g.name,
                w: g.w.ok_or_else(|| D::Error::missing_field("w"))?,
                h: g.h.ok_or_else(|| D::Error::missing_field("h"))?,
                r: g.r,
                wraps_x: g
                    .wraps_x
                    .ok_or_else(|| D::Error::missing_field("wraps_x"))?,
                origin_block_x: g
                    .origin_block_x
                    .ok_or_else(|| D::Error::missing_field("origin_block_x"))?,
                origin_block_z: g
                    .origin_block_z
                    .ok_or_else(|| D::Error::missing_field("origin_block_z"))?,
                z_edges: g.z_edges,
            }),
        }
    }
}

/// Where a reader of a cube grid stands: the face, and the fractional local cell (a, b) of the
/// point (cell (i, j)'s centre is at (i, j)), or nothing past the net's corners.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubeSpot {
    pub face: Face,
    pub a: f64,
    pub b: f64,
}

/// The cell a face's local cell (i, j) of an n × n face grid is: itself on the face; the
/// neighbour's beyond one side ([`cell_on`]); beyond two sides at once (the corner squares where
/// three faces meet) the neighbour across the side it is further out of, ties east or west, with
/// the other index clamped onto the face, so the corner square repeats the strip along that
/// neighbour's edge (the diagonal ghost of FV3); more than a face out, clamped to a face out.
pub fn resolve_face_cell(face: Face, n: i64, i: i64, j: i64) -> (Face, i64, i64) {
    let (i, j) = (i.clamp(-n, 2 * n - 1), j.clamp(-n, 2 * n - 1));
    let out = |k: i64| {
        if k < 0 {
            -k
        } else if k >= n {
            k - n + 1
        } else {
            0
        }
    };
    let (oi, oj) = (out(i), out(j));
    let (i, j) = if oi > 0 && oj > 0 {
        if oi >= oj {
            (i, j.clamp(0, n - 1))
        } else {
            (i.clamp(0, n - 1), j)
        }
    } else {
        (i, j)
    };
    match cell_on(face, n, i, j) {
        Some((f, a, b, _)) => (f, a.clamp(0, n - 1), b.clamp(0, n - 1)),
        None => (face, i.clamp(0, n - 1), j.clamp(0, n - 1)),
    }
}

impl GridDesc {
    /// The grid of a cube world of face side `s` blocks at `r` blocks per cell (n = s / r cells
    /// a face side; [`ZEdges::Cube`]).
    pub fn for_cube(name: &str, s: u32, r: u32) -> Result<GridDesc> {
        if r == 0 || s < 2 || !s.is_multiple_of(2) || !s.is_multiple_of(r) {
            return Err(FormatError::Invalid(format!(
                "cube face side {s} is not even or not a multiple of r = {r}"
            )));
        }
        let n = s / r;
        Ok(GridDesc {
            name: name.to_owned(),
            w: n,
            h: 6 * n,
            r,
            wraps_x: false,
            origin_block_x: -i64::from(s / 2),
            origin_block_z: -i64::from(s / 2),
            z_edges: ZEdges::Cube,
        })
    }

    /// Whether this is a cube world's grid.
    #[inline]
    pub fn is_cube(&self) -> bool {
        self.z_edges == ZEdges::Cube
    }

    /// A cube grid's face side in blocks, s = n·r.
    pub fn face_blocks(&self) -> u32 {
        self.w * self.r
    }

    /// A cube grid's net ([`Cube::shared`]).
    pub fn cube(&self) -> &'static Cube {
        Cube::shared(self.face_blocks())
    }

    /// The stored cell of a cube grid's face `face`, local cell (i, j), resolved off the face
    /// ([`resolve_face_cell`]): (column, row) in the stacked planes.
    #[inline]
    pub fn face_cell(&self, face: Face, i: i64, j: i64) -> (u32, u32) {
        let n = i64::from(self.w);
        let (f, a, b) = resolve_face_cell(face, n, i, j);
        (a as u32, (f.index() as i64 * n + b) as u32)
    }

    /// The face and local cell of a cube grid's stacked cell (i, j) (in range).
    #[inline]
    pub fn face_of_cell(&self, i: u32, j: u32) -> (Face, u32, u32) {
        let k = j / self.w;
        (
            Face::from_index(k as usize).expect("a row of the six planes"),
            i,
            j - k * self.w,
        )
    }

    /// The net point at the centre of a cube grid's stacked cell (i, j) (in range): the face's
    /// origin + ((i + ½)·r, (j + ½)·r).
    pub fn cube_point(&self, i: u32, j: u32) -> (f64, f64) {
        let (face, a, b) = self.face_of_cell(i, j);
        let (ox, oz) = self.cube().face_origin(face);
        let r = f64::from(self.r);
        (
            ox as f64 + (f64::from(a) + 0.5) * r,
            oz as f64 + (f64::from(b) + 0.5) * r,
        )
    }

    /// Where the net point (x, z) (any point: folded onto the planet) lies on a cube grid: its face
    /// and fractional local cell; `None` past the net's corners.
    pub fn cube_spot(&self, x: f64, z: f64) -> Option<CubeSpot> {
        let cube = self.cube();
        let p = cube.fold_point(x, z)?;
        let (ox, oz) = cube.face_origin(p.face);
        let r = f64::from(self.r);
        Some(CubeSpot {
            face: p.face,
            a: (p.x - ox as f64) / r - 0.5,
            b: (p.z - oz as f64) / r - 0.5,
        })
    }

    /// Where block (bx, bz) of a reader lies on a cube grid: the block's centre (bx + ½, bz + ½)
    /// ([`GridDesc::cube_spot`]). Every [`RasterRead`] method takes blocks this way on a cube.
    #[inline]
    pub fn cube_spot_of_block(&self, bx: f64, bz: f64) -> Option<CubeSpot> {
        self.cube_spot(bx + 0.5, bz + 0.5)
    }

    /// The tile edge of a layer on this grid: [`TILE`], or on a cube grid the largest divisor of n
    /// that divides [`TILE`] too, so no tile holds two faces.
    pub fn tile(&self) -> u32 {
        if self.is_cube() {
            let (mut a, mut b) = (self.w, TILE);
            while b != 0 {
                (a, b) = (b, a % b);
            }
            a
        } else {
            TILE
        }
    }
    /// The grid of a planet with circumference `c` blocks at `r` blocks per cell: W = C/r,
    /// H = 2·Zo/r, origin (−C/2, −Zo), Zo = [`Projection::grid_half_extent`] (the smallest
    /// multiple of the planet's quantum, 64 blocks, at or past the pole R = C/2π).
    pub fn for_planet(name: &str, c: u32, r: u32) -> Result<GridDesc> {
        if r == 0 || !c.is_multiple_of(2 * r) {
            return Err(FormatError::Invalid(format!(
                "circumference {c} is not a multiple of 2·r = {}",
                2 * r
            )));
        }
        let p = Projection::new(c);
        Ok(GridDesc {
            name: name.to_owned(),
            w: c / r,
            h: p.grid_rows(r),
            r,
            wraps_x: true,
            origin_block_x: -i64::from(c / 2),
            origin_block_z: p.grid_origin_z(r),
            z_edges: ZEdges::Clamp,
        })
    }

    /// Every `s`-th cell of this grid as a grid of its own (`name`; W/s × H/s cells, rounded up,
    /// r·s blocks a cell, the same origin), as thinned copies and previews are taken.
    pub fn thinned(&self, name: &str, s: u32) -> GridDesc {
        let s = s.max(1);
        if self.is_cube() {
            // Whole faces: n/s cells a side, rounded up.
            let n = self.w.div_ceil(s);
            return GridDesc {
                name: name.to_owned(),
                w: n,
                h: 6 * n,
                r: self.r * s,
                ..self.clone()
            };
        }
        GridDesc {
            name: name.to_owned(),
            w: self.w.div_ceil(s),
            h: self.h.div_ceil(s),
            r: self.r * s,
            wraps_x: self.wraps_x,
            origin_block_x: self.origin_block_x,
            origin_block_z: self.origin_block_z,
            z_edges: self.z_edges,
        }
    }

    /// The planet's projection.
    pub fn projection(&self) -> Projection {
        Projection::of(self.circumference() as f64)
    }

    /// The (fractional) row of block z: (z − origin_block_z) / r.
    pub fn row_of_z(&self, z: f64) -> f64 {
        (z - self.origin_block_z as f64) / f64::from(self.r)
    }

    /// Block z of the (fractional) row `v`.
    pub fn z_of_row(&self, v: f64) -> f64 {
        self.origin_block_z as f64 + v * f64::from(self.r)
    }

    /// Latitude of row `j`, degrees (north positive; ±90 past the poles).
    pub fn lat_of_row(&self, j: u32) -> f64 {
        self.projection().lat_deg(self.block_z(i64::from(j)) as f64)
    }

    /// The share of an r × r square that row `j`'s cells cover of the planet: 1 inside, less on
    /// the rows that reach past a pole, 0 on rows wholly past it. The first and last row take
    /// everything beyond them (z clamps to them), so the shares sum to 2R / r.
    pub fn area_share(&self, j: u32) -> f64 {
        let (z, half) = (self.block_z(i64::from(j)) as f64, 0.5 * f64::from(self.r));
        let lo = if j == 0 { f64::NEG_INFINITY } else { z - half };
        let hi = if j + 1 >= self.h {
            f64::INFINITY
        } else {
            z + half
        };
        let r = self.projection().radius();
        ((hi.min(r) - lo.max(-r)).max(0.0) / f64::from(self.r)).min(1.0)
    }

    /// True area of one cell of row `j`, km², on a planet `m_per_block` metres per block: every
    /// cell inside the sphere is (r · m)² ([`GridDesc::area_share`] at the poles).
    pub fn cell_area_km2(&self, j: u32, m_per_block: f64) -> f64 {
        let side_km = f64::from(self.r) * m_per_block / 1000.0;
        side_km * side_km * self.area_share(j)
    }

    /// The area weight of the band from row `j` to row `j + 1`: the share of it on the planet.
    pub fn band_weight(&self, j: u32) -> f64 {
        let z = self.block_z(i64::from(j)) as f64;
        self.projection().band_share(z, z + f64::from(self.r))
    }

    /// The latitude, degrees, in the middle of the band from row `j` to row `j + 1`.
    pub fn band_lat_deg(&self, j: u32) -> f64 {
        let z = self.block_z(i64::from(j)) as f64;
        self.projection().lat_deg(z + 0.5 * f64::from(self.r))
    }

    /// [`GridDesc::area_share`] of every row: the rows' weights for true-area means and shares.
    pub fn row_weights(&self) -> Vec<f64> {
        (0..self.h).map(|j| self.area_share(j)).collect()
    }

    /// True size of a cell of row `j` on the planet, metres: (east–west, north–south). East–west
    /// r·m·cos φ at the row; north–south the arc of its band, R·m·(φ(z − r/2) − φ(z + r/2)),
    /// which is r·m / cos φ away from the poles (0 for a row wholly past a pole).
    pub fn true_cell_m(&self, j: u32, m_per_block: f64) -> (f64, f64) {
        let p = self.projection();
        let z = self.block_z(i64::from(j)) as f64;
        let half = 0.5 * f64::from(self.r);
        let ew = f64::from(self.r) * m_per_block * p.scale_ew(z);
        let arc = (p.lat_deg(z - half) - p.lat_deg(z + half)).to_radians();
        (ew, p.radius() * m_per_block * arc)
    }

    /// The (fractional) row under row coordinate `y` of a map picture `h` rows high that spans the
    /// planet pole to pole ([`Projection::picture_z`]; the grid reaches a little past the poles).
    pub fn picture_row(&self, y: f64, h: u32) -> f64 {
        self.row_of_z(self.projection().picture_z(y, h))
    }

    /// The cell nearest pixel (px, py) of a `w × h` map picture of the planet (pixel (px, py) is
    /// centred on column px·W/w and on the row of [`GridDesc::picture_row`]), not yet wrapped or
    /// clamped.
    pub fn picture_cell(&self, px: f64, py: f64, w: u32, h: u32) -> (i64, i64) {
        (
            (px * f64::from(self.w) / f64::from(w.max(1))).round() as i64,
            self.picture_row(py, h).round() as i64,
        )
    }

    /// The circumference C = W·r in blocks (a cube grid's loop, 4s).
    pub fn circumference(&self) -> u64 {
        if self.is_cube() {
            4 * u64::from(self.w) * u64::from(self.r)
        } else {
            u64::from(self.w) * u64::from(self.r)
        }
    }

    /// Block x of cell column `i`.
    pub fn block_x(&self, i: i64) -> i64 {
        self.origin_block_x + i * i64::from(self.r)
    }

    /// Block z of cell row `j`.
    pub fn block_z(&self, j: i64) -> i64 {
        self.origin_block_z + j * i64::from(self.r)
    }

    /// Wraps a column index into [0, W).
    pub fn wrap_i(&self, i: i64) -> u32 {
        crate::projection::wrap_col(i, i64::from(self.w)) as u32
    }

    /// Clamps a row index into [0, H − 1].
    pub fn clamp_j(&self, j: i64) -> u32 {
        crate::projection::clamp_row(j, i64::from(self.h)) as u32
    }

    /// The stored cell of any index pair: columns wrap; rows clamp, or fold through the edges'
    /// half-turns ([`ZEdges::HalfTurn`]: row −k is row k and row H − 1 + k row H − 1 − k, at
    /// column W − i). Every world-data reader samples through this.
    #[inline]
    pub fn cell(&self, i: i64, j: i64) -> (u32, u32) {
        match self.z_edges {
            ZEdges::Clamp => (self.wrap_i(i), self.clamp_j(j)),
            ZEdges::Cube => {
                // The row's plane (clamped to the six), then the face's local cell resolved off it.
                let n = i64::from(self.w);
                let k = j.div_euclid(n.max(1)).clamp(0, 5);
                let face = Face::from_index(k as usize).expect("a face index");
                self.face_cell(face, i, j - k * n)
            }
            ZEdges::HalfTurn => {
                let last = i64::from(self.h) - 1;
                let w = i64::from(self.w);
                let (mut i, mut j) = (i, j);
                if last <= 0 {
                    return (self.wrap_i(i), 0);
                }
                for _ in 0..64 {
                    if j < 0 {
                        j = -j;
                        i = w - i;
                    } else if j > last {
                        j = 2 * last - j;
                        i = w - i;
                    } else {
                        break;
                    }
                }
                (self.wrap_i(i), j.clamp(0, last) as u32)
            }
        }
    }

    /// Cells in the raster.
    pub fn cells(&self) -> usize {
        self.w as usize * self.h as usize
    }
}

/// A layer's descriptor, stored as JSON in its tile pack (plan §5.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerDesc {
    /// `elevation`, `climate/temperature`, `strata/horizons`, …
    pub name: String,
    pub dtype: DType,
    /// physical = raw · scale + offset.
    pub scale: f64,
    pub offset: f64,
    /// `block/16`, `vanilla_param*10000`, …
    pub units: String,
    pub grid: GridDesc,
    /// Tile edge in cells (256).
    pub tile: u32,
    pub codec: Codec,
    /// The **raw** cell value meaning "no data" (e.g. −32768 for `water_level`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nodata: Option<f64>,
    /// K, for `horizon_stack` layers only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_depth: Option<u32>,
    /// `horizon_stack` only: keys per band of the column rule's tongues (default
    /// `strata::BAND_KEYS`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tongue_band_keys: Option<u32>,
    /// `horizon_stack` only: the share below which a rock takes no part in the tongues (default
    /// `strata::MINOR_SHARE`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tongue_minor_share: Option<f64>,
    /// `horizon_stack` only: per class index, the least thickness (blocks) of a horizon of that
    /// class inside a column; the column rule drops a thinner one (`strata`, step 5). Absent, or
    /// 0 or 1 for a class: nothing is dropped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_blocks: Option<Vec<u8>>,
}

impl LayerDesc {
    /// A scalar layer with the grid's tile size ([`GridDesc::tile`]).
    pub fn scalar(name: &str, dtype: DType, grid: GridDesc, codec: Codec) -> LayerDesc {
        LayerDesc {
            name: name.to_owned(),
            dtype,
            scale: 1.0,
            offset: 0.0,
            units: String::new(),
            tile: grid.tile(),
            grid,
            codec,
            nodata: None,
            stack_depth: None,
            tongue_band_keys: None,
            tongue_minor_share: None,
            min_blocks: None,
        }
    }

    /// Bytes per cell.
    pub fn cell_bytes(&self) -> usize {
        match self.dtype.scalar_bytes() {
            Some(b) => b,
            None => 7 * self.stack_depth.unwrap_or(0) as usize,
        }
    }

    /// Tiles east–west.
    pub fn tiles_x(&self) -> u32 {
        self.grid.w.div_ceil(self.tile)
    }

    /// Tiles north–south.
    pub fn tiles_z(&self) -> u32 {
        self.grid.h.div_ceil(self.tile)
    }

    /// Tiles in the pack.
    pub fn tile_count(&self) -> usize {
        self.tiles_x() as usize * self.tiles_z() as usize
    }

    /// Tile `t` (row-major: t = tz · tiles_x + tx) as (first column, first row, width, height).
    /// Edge tiles are cropped to the raster.
    pub fn tile_rect(&self, t: usize) -> (u32, u32, u32, u32) {
        let tx = (t % self.tiles_x() as usize) as u32;
        let tz = (t / self.tiles_x() as usize) as u32;
        let x0 = tx * self.tile;
        let z0 = tz * self.tile;
        (
            x0,
            z0,
            self.tile.min(self.grid.w - x0),
            self.tile.min(self.grid.h - z0),
        )
    }

    /// Checks the descriptor is internally consistent.
    pub fn validate(&self) -> Result<()> {
        let bad = |m: String| Err(FormatError::Invalid(format!("layer {}: {m}", self.name)));
        if self.name.is_empty() {
            return bad("empty name".into());
        }
        if self.grid.w == 0 || self.grid.h == 0 || self.grid.r == 0 {
            return bad(format!("empty grid {:?}", self.grid));
        }
        if self.tile == 0 {
            return bad("tile edge 0".into());
        }
        if self.grid.is_cube()
            && (self.grid.h != 6 * self.grid.w
                || self.grid.wraps_x
                || !self.grid.w.is_multiple_of(self.tile))
        {
            return bad(format!(
                "cube grid {:?} needs H = 6W, no x wrap and a tile edge dividing W",
                self.grid
            ));
        }
        match (self.dtype, self.stack_depth) {
            (DType::HorizonStack, Some(k)) if (1..=255).contains(&k) => {}
            (DType::HorizonStack, k) => {
                return bad(format!(
                    "horizon_stack needs stack_depth 1..=255, got {k:?}"
                ));
            }
            (_, Some(_)) => return bad("stack_depth on a scalar layer".into()),
            _ => {}
        }
        if self.tongue_band_keys == Some(0) {
            return bad("tongue_band_keys 0".into());
        }
        if let Some(m) = self.tongue_minor_share
            && !(0.0..0.5).contains(&m)
        {
            return bad(format!("tongue_minor_share {m} outside [0, 0.5)"));
        }
        if !self.scale.is_finite() || !self.offset.is_finite() {
            return bad("non-finite scale or offset".into());
        }
        Ok(())
    }

    /// Raw value → physical value; NaN for nodata.
    pub fn physical(&self, raw: f64) -> f64 {
        if self.nodata == Some(raw) || raw.is_nan() {
            f64::NAN
        } else {
            raw * self.scale + self.offset
        }
    }
}

/// A scalar cell type that can be written to a tile pack.
pub trait Cell: Copy + Send + Sync + 'static {
    const DTYPE: DType;
    /// Appends the little-endian bytes of `self`.
    fn put_le(self, out: &mut Vec<u8>);
}

macro_rules! cell {
    ($t:ty, $d:ident) => {
        impl Cell for $t {
            const DTYPE: DType = DType::$d;
            fn put_le(self, out: &mut Vec<u8>) {
                out.extend_from_slice(&self.to_le_bytes());
            }
        }
    };
}
cell!(u8, U8);
cell!(i8, I8);
cell!(u16, U16);
cell!(i16, I16);
cell!(u32, U32);
cell!(f32, F32);

/// A whole raster in memory, row-major (`data[j · W + i]`).
#[derive(Clone, Debug, PartialEq)]
pub struct Raster<T> {
    pub grid: GridDesc,
    pub data: Vec<T>,
}

impl<T: Cell> Raster<T> {
    /// Fills every cell from `f(i, j)`.
    pub fn from_fn(grid: GridDesc, mut f: impl FnMut(u32, u32) -> T) -> Raster<T> {
        let mut data = Vec::with_capacity(grid.cells());
        for j in 0..grid.h {
            for i in 0..grid.w {
                data.push(f(i, j));
            }
        }
        Raster { grid, data }
    }

    /// The cell at (i, j) with x wrapped and z clamped.
    pub fn get(&self, i: i64, j: i64) -> T {
        let (i, j) = self.grid.cell(i, j);
        let (i, j) = (i as usize, j as usize);
        self.data[j * self.grid.w as usize + i]
    }

    /// The raster's cells as little-endian bytes, row-major.
    pub fn to_le_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.data.len() * T::DTYPE.scalar_bytes().unwrap_or(1));
        for &v in &self.data {
            v.put_le(&mut out);
        }
        out
    }
}

/// Sampling shared by every reader. The Java `RasterSampler` implements the same arithmetic in
/// the same order, so results agree bit for bit (the parity test checks it).
pub trait RasterRead {
    fn desc(&self) -> &LayerDesc;

    /// The raw value of an in-range cell (0 ≤ i < W, 0 ≤ j < H).
    fn raw_cell(&self, i: u32, j: u32) -> f64;

    /// The raw value at (i, j), x wrapped, z clamped.
    fn raw(&self, i: i64, j: i64) -> f64 {
        let g = &self.desc().grid;
        let (ci, cj) = g.cell(i, j);
        self.raw_cell(ci, cj)
    }

    /// The physical value at cell (i, j), x wrapped, z clamped; NaN for nodata.
    fn get(&self, i: i64, j: i64) -> f64 {
        self.desc().physical(self.raw(i, j))
    }

    /// The physical value of a cube grid's face `face`, local cell (i, j) resolved off the face.
    fn get_face(&self, face: Face, i: i64, j: i64) -> f64 {
        let (ci, cj) = self.desc().grid.face_cell(face, i, j);
        self.desc().physical(self.raw_cell(ci, cj))
    }

    /// The physical value of the cell nearest block (bx, bz). On a cube grid: the cell under the
    /// block's centre (NaN past the net's corners).
    fn nearest(&self, bx: f64, bz: f64) -> f64 {
        let g = &self.desc().grid;
        if g.is_cube() {
            return match g.cube_spot_of_block(bx, bz) {
                Some(s) => self.get_face(
                    s.face,
                    (s.a + 0.5).floor() as i64,
                    (s.b + 0.5).floor() as i64,
                ),
                None => f64::NAN,
            };
        }
        let u = (bx - g.origin_block_x as f64) / f64::from(g.r);
        let v = (bz - g.origin_block_z as f64) / f64::from(g.r);
        self.get((u + 0.5).floor() as i64, (v + 0.5).floor() as i64)
    }

    /// Bilinear interpolation of the physical values at block (bx, bz). NaN if any of the four
    /// cells is nodata. On a cube grid: between the centres of the four cells round the block's
    /// centre on its face, cells off the face read through [`GridDesc::face_cell`] (NaN past the
    /// net's corners).
    fn bilinear(&self, bx: f64, bz: f64) -> f64 {
        let g = &self.desc().grid;
        if g.is_cube() {
            let Some(s) = g.cube_spot_of_block(bx, bz) else {
                return f64::NAN;
            };
            let (fi, fj) = (s.a.floor(), s.b.floor());
            let (fu, fv) = (s.a - fi, s.b - fj);
            let (i0, j0) = (fi as i64, fj as i64);
            let v00 = self.get_face(s.face, i0, j0);
            let v10 = self.get_face(s.face, i0 + 1, j0);
            let v01 = self.get_face(s.face, i0, j0 + 1);
            let v11 = self.get_face(s.face, i0 + 1, j0 + 1);
            let a = v00 + (v10 - v00) * fu;
            let b = v01 + (v11 - v01) * fu;
            return a + (b - a) * fv;
        }
        let u = (bx - g.origin_block_x as f64) / f64::from(g.r);
        let v = (bz - g.origin_block_z as f64) / f64::from(g.r);
        let fi = u.floor();
        let fj = v.floor();
        let fu = u - fi;
        let fv = v - fj;
        let (i0, j0) = (fi as i64, fj as i64);
        let v00 = self.get(i0, j0);
        let v10 = self.get(i0 + 1, j0);
        let v01 = self.get(i0, j0 + 1);
        let v11 = self.get(i0 + 1, j0 + 1);
        let a = v00 + (v10 - v00) * fu;
        let b = v01 + (v11 - v01) * fu;
        a + (b - a) * fv
    }

    /// A cover layer's value at block (bx, bz) (`climate/ice_cover`, FORMAT.md "The ice cover"):
    /// [`cover_of`] with the nearest cell deciding whether the block is covered and the four cells
    /// of the bilinear blend its depth, so the cover's edge lies half-way between a covered cell and
    /// a bare one, where the nearest-cell Köppen class changes too. A plain bilinear read rounded to
    /// whole blocks covered every block where the blend passed half a block, which put a thick
    /// margin's cover most of a cell past it.
    fn cover(&self, bx: f64, bz: f64) -> f64 {
        let g = &self.desc().grid;
        if g.is_cube() {
            let Some(s) = g.cube_spot_of_block(bx, bz) else {
                return 0.0;
            };
            let (fi, fj) = (s.a.floor(), s.b.floor());
            let (i0, j0) = (fi as i64, fj as i64);
            let w = bilinear_weights(s.a - fi, s.b - fj);
            return cover_of(
                self.nearest(bx, bz),
                [
                    (self.get_face(s.face, i0, j0), w[0]),
                    (self.get_face(s.face, i0 + 1, j0), w[1]),
                    (self.get_face(s.face, i0, j0 + 1), w[2]),
                    (self.get_face(s.face, i0 + 1, j0 + 1), w[3]),
                ],
            );
        }
        let u = (bx - g.origin_block_x as f64) / f64::from(g.r);
        let v = (bz - g.origin_block_z as f64) / f64::from(g.r);
        let fi = u.floor();
        let fj = v.floor();
        let fu = u - fi;
        let fv = v - fj;
        let (i0, j0) = (fi as i64, fj as i64);
        let w = bilinear_weights(fu, fv);
        cover_of(
            self.nearest(bx, bz),
            [
                (self.get(i0, j0), w[0]),
                (self.get(i0 + 1, j0), w[1]),
                (self.get(i0, j0 + 1), w[2]),
                (self.get(i0 + 1, j0 + 1), w[3]),
            ],
        )
    }

    /// The column rule's cubic B-spline of the physical values at block column (x, z)
    /// (FORMAT.md, "The lift"): the 4 × 4 cells and weights [`crate::strata::stack_at`] reads a
    /// column's horizon tops with (x folded in whole blocks, z clamped), summed as an offset from
    /// cell (i0, j0)'s value as the tops are, so a uniform field gives exactly its value. A layer
    /// that moved every horizon of each cell by the cell's value (`strata/lift`) moved the
    /// interpolated tops of the column by this much.
    fn column_spline(&self, x: i64, z: i64) -> f64 {
        let g = &self.desc().grid;
        if g.is_cube() {
            let Some(s) = g.cube_spot_of_block(x as f64, z as f64) else {
                return f64::NAN;
            };
            let (i0, j0) = (s.a.floor() as i64, s.b.floor() as i64);
            let (bu, bv) = (
                crate::strata::bspline(s.a - i0 as f64),
                crate::strata::bspline(s.b - j0 as f64),
            );
            let base = self.get_face(s.face, i0, j0);
            let mut sum = 0.0f64;
            for (b, &wv) in bv.iter().enumerate() {
                for (a, &wu) in bu.iter().enumerate() {
                    let w = wu * wv;
                    sum += w * (self.get_face(s.face, i0 - 1 + a as i64, j0 - 1 + b as i64) - base);
                }
            }
            return base + sum;
        }
        let c = i64::from(g.w) * i64::from(g.r);
        let wx = (x + c / 2).rem_euclid(c) - c / 2;
        let u = (wx - g.origin_block_x) as f64 / f64::from(g.r);
        let v = (z - g.origin_block_z) as f64 / f64::from(g.r);
        let (i0, j0) = (u.floor() as i64, v.floor() as i64);
        let (bu, bv) = (
            crate::strata::bspline(u - i0 as f64),
            crate::strata::bspline(v - j0 as f64),
        );
        let base = self.get(i0, j0);
        let mut sum = 0.0f64;
        for (b, &wv) in bv.iter().enumerate() {
            for (a, &wu) in bu.iter().enumerate() {
                let w = wu * wv;
                sum += w * (self.get(i0 - 1 + a as i64, j0 - 1 + b as i64) - base);
            }
        }
        base + sum
    }
}

/// The weights of a bilinear blend's four cells at fractions (fu, fv) of the cell step: cells
/// (i0, j0), (i0 + 1, j0), (i0, j0 + 1), (i0 + 1, j0 + 1).
#[inline]
pub fn bilinear_weights(fu: f64, fv: f64) -> [f64; 4] {
    [
        (1.0 - fu) * (1.0 - fv),
        fu * (1.0 - fv),
        (1.0 - fu) * fv,
        fu * fv,
    ]
}

/// The cover rule, one for every reader of a cover layer (the game's `RasterSampler.cover`, the
/// bake's projection, the workbench): covered where `nearest`, the value of the nearest cell, is
/// above 0 (NaN is none), and then at the weighted mean of the samples (value, weight) above 0,
/// or at `nearest` where none is; 0 elsewhere. The nearest cell decides the extent as it decides a
/// block's Köppen class (read nearest), so every covered block is of the cover's class (EF,
/// BIO-158) and the cover's area is its cells' area; the blend of the covered cells around thins
/// its depth without spreading it past them. ledger: BIO-159 (game-choice)
pub fn cover_of(nearest: f64, samples: impl IntoIterator<Item = (f64, f64)>) -> f64 {
    if nearest.is_nan() || nearest <= 0.0 {
        return 0.0;
    }
    let (mut held, mut sum) = (0.0, 0.0);
    for (v, w) in samples {
        if v > 0.0 {
            held += w;
            sum += w * v;
        }
    }
    if held > 0.0 { sum / held } else { nearest }
}

/// A whole layer's cells in memory (from [`crate::TilePackReader::load`] or built directly).
#[derive(Clone, Debug)]
pub struct LoadedLayer {
    pub desc: LayerDesc,
    /// Row-major cells, `desc.cell_bytes()` each.
    pub bytes: Vec<u8>,
}

impl LoadedLayer {
    /// Wraps row-major cell bytes; fails if the length does not match the descriptor.
    pub fn new(desc: LayerDesc, bytes: Vec<u8>) -> Result<LoadedLayer> {
        let want = desc.grid.cells() * desc.cell_bytes();
        if bytes.len() != want {
            return Err(FormatError::Invalid(format!(
                "layer {}: {} bytes of cells, expected {want}",
                desc.name,
                bytes.len()
            )));
        }
        Ok(LoadedLayer { desc, bytes })
    }

    /// Horizon `k` of the stack at in-range cell (i, j).
    pub fn horizon(&self, i: u32, j: u32, k: u32) -> StoredHorizon {
        let depth = self.desc.stack_depth.unwrap_or(0) as usize;
        let cell = (j as usize * self.desc.grid.w as usize + i as usize) * 7 * depth;
        decode_horizon(&self.bytes[cell..cell + 7 * depth], depth, k as usize)
    }

    /// The horizon stack at block (x, z) of a world whose floor is `min_y`, exactly as the Java
    /// strata pass builds a column ([`crate::strata::stack_at`]).
    pub fn stack_at(&self, x: i64, z: i64, min_y: i32, out: &mut Vec<ColumnHorizon>) {
        crate::strata::stack_at(
            &self.desc,
            min_y,
            |i, j, k| self.horizon(i, j, k),
            x,
            z,
            out,
        );
    }

    /// [`LoadedLayer::stack_at`] with its working memory given: a reader of many columns keeps one
    /// [`StackScratch`](crate::strata::StackScratch) per thread. The same answer, bit for bit.
    pub fn stack_at_with(
        &self,
        scratch: &mut crate::strata::StackScratch,
        x: i64,
        z: i64,
        min_y: i32,
        out: &mut Vec<ColumnHorizon>,
    ) {
        crate::strata::stack_at_with(
            scratch,
            &self.desc,
            min_y,
            |i, j, k| self.horizon(i, j, k),
            x,
            z,
            out,
        );
    }
}

impl RasterRead for LoadedLayer {
    fn desc(&self) -> &LayerDesc {
        &self.desc
    }

    fn raw_cell(&self, i: u32, j: u32) -> f64 {
        let cb = self.desc.cell_bytes();
        let at = (j as usize * self.desc.grid.w as usize + i as usize) * cb;
        self.desc.dtype.decode_scalar(&self.bytes[at..at + cb])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(c: u32, r: u32) -> LoadedLayer {
        let grid = GridDesc::for_planet("L", c, r).unwrap();
        let ras = Raster::<i16>::from_fn(grid.clone(), |i, j| (i as i16) * 10 + j as i16);
        let mut d = LayerDesc::scalar("t", DType::I16, grid, Codec::Raw);
        d.scale = 0.5;
        d.nodata = Some(-32768.0);
        LoadedLayer::new(d, ras.to_le_bytes()).unwrap()
    }

    #[test]
    fn grid_geometry() {
        let g = GridDesc::for_planet("L3", 16384, 2).unwrap();
        // Lambert: pole to pole 2R = C/π = 5,215 blocks, the grid to ±2,624 (a multiple of 64).
        assert_eq!((g.w, g.h), (8192, 2624));
        assert_eq!((g.origin_block_x, g.origin_block_z), (-8192, -2624));
        let l1 = GridDesc::for_planet("L1", 16384, 16).unwrap();
        assert_eq!((l1.w, l1.h, l1.origin_block_z), (1024, 328, -2624));
        assert_eq!(l1.h * 8, g.h, "the ladder nests: L3 row 8j is L1 row j");
        assert_eq!(g.circumference(), 16384);
        assert!(GridDesc::for_planet("x", 1000, 16).is_err());
    }

    #[test]
    fn the_column_spline_is_exact_on_a_uniform_field_and_follows_a_ramp() {
        // A lift of 12.5 blocks everywhere (sixteenths, as `strata/lift` stores it).
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let mut d = LayerDesc::scalar("strata/lift", DType::I16, grid.clone(), Codec::Raw);
        d.scale = 1.0 / 16.0;
        let flat = Raster::<i16>::from_fn(grid.clone(), |_, _| 200);
        let flat = LoadedLayer::new(d.clone(), flat.to_le_bytes()).unwrap();
        for (x, z) in [(0, 0), (-511, 3), (300, -100), (511, 150), (1500, -9999)] {
            assert_eq!(flat.column_spline(x, z), 12.5, "({x}, {z})");
        }
        // A ramp east-west (away from the wrap's seam): the cubic B-spline reproduces it.
        let ramp = Raster::<i16>::from_fn(grid.clone(), |i, _| (i as i16) * 16);
        let ramp = LoadedLayer::new(d, ramp.to_le_bytes()).unwrap();
        for x in [-400i64, -100, -7, 0, 9, 250] {
            let u = (x - grid.origin_block_x) as f64 / f64::from(grid.r);
            assert!((ramp.column_spline(x, 40) - u).abs() < 1e-9, "{x}");
        }
    }

    #[test]
    fn wrap_and_clamp() {
        let l = layer(64, 4); // 16 × 8
        assert_eq!(l.raw(-1, 0), l.raw(15, 0));
        assert_eq!(l.raw(16, 3), l.raw(0, 3));
        assert_eq!(l.raw(2, -5), l.raw(2, 0));
        assert_eq!(l.raw(2, 99), l.raw(2, 7));
        assert_eq!(l.get(3, 2), 0.5 * 32.0);
    }

    #[test]
    fn bilinear_hits_samples_and_midpoints() {
        let l = layer(64, 4);
        let g = &l.desc.grid;
        for (i, j) in [(0, 0), (5, 3), (15, 7)] {
            let bx = g.block_x(i) as f64;
            let bz = g.block_z(j) as f64;
            assert_eq!(l.bilinear(bx, bz), l.get(i, j));
            assert_eq!(l.nearest(bx + 1.0, bz - 1.0), l.get(i, j));
        }
        // Midway between columns 3 and 4 of row 2.
        let mid = l.bilinear(g.block_x(3) as f64 + 2.0, g.block_z(2) as f64);
        assert_eq!(mid, 0.5 * (l.get(3, 2) + l.get(4, 2)));
        // Across the seam: between the last column and column 0.
        let seam = l.bilinear(g.block_x(15) as f64 + 2.0, g.block_z(1) as f64);
        assert_eq!(seam, 0.5 * (l.get(15, 1) + l.get(0, 1)));
    }

    /// The cover rule's vectors; neoforge's IceCoverTest holds the same ones for RasterSampler.
    #[test]
    fn the_nearest_cell_decides_and_the_covered_cells_blend() {
        // A covered nearest cell: covered, at the covered samples' weighted mean (not the blend
        // with the bare ones).
        assert_eq!(cover_of(8.0, [(8.0, 0.5), (0.0, 0.5)]), 8.0);
        assert_eq!(cover_of(4.0, [(4.0, 0.25), (8.0, 0.75), (0.0, 0.0)]), 7.0);
        // A bare nearest cell: bare, however thick and heavy the covered side is.
        assert_eq!(cover_of(0.0, [(12.0, 0.75), (0.0, 0.25)]), 0.0);
        // ... and a covered one is covered however light it is.
        assert_eq!(cover_of(12.0, [(12.0, 0.25), (0.0, 0.75)]), 12.0);
        // NaN (no data) counts as bare, never as cover.
        assert_eq!(cover_of(f64::NAN, [(6.0, 1.0)]), 0.0);
        assert_eq!(cover_of(6.0, [(f64::NAN, 0.75), (6.0, 0.25)]), 6.0);
        // No covered sample: the nearest cell's own value.
        assert_eq!(cover_of(5.0, [(0.0, 1.0)]), 5.0);
        assert_eq!(cover_of(5.0, std::iter::empty()), 5.0);
        assert_eq!(cover_of(0.0, std::iter::empty()), 0.0);
        let w = bilinear_weights(0.25, 0.5);
        assert_eq!(w, [0.375, 0.125, 0.375, 0.125]);
        assert_eq!(w.iter().sum::<f64>(), 1.0);
    }

    #[test]
    fn a_covers_edge_lies_half_way_between_cells() {
        // Cells 0..=3 of every row covered with 10 blocks, the rest bare.
        let grid = GridDesc::for_planet("L", 64, 4).unwrap();
        let ras = Raster::<u8>::from_fn(grid.clone(), |i, _| if i <= 3 { 10 } else { 0 });
        let l = LoadedLayer::new(
            LayerDesc::scalar("c", DType::U8, grid.clone(), Codec::Raw),
            ras.to_le_bytes(),
        )
        .unwrap();
        let z = grid.block_z(2) as f64;
        let x3 = grid.block_x(3) as f64;
        // On the last covered cell, and up to half a step past it: the full depth.
        assert_eq!(l.cover(x3, z), 10.0);
        assert_eq!(l.cover(x3 + 1.9, z), 10.0);
        // From half-way on (the nearest cell's tie goes to the next cell): bare, where the
        // bilinear blend still rounds to 4 blocks.
        assert_eq!(l.cover(x3 + 2.0, z), 0.0);
        assert_eq!(l.cover(x3 + 2.5, z), 0.0);
        assert!((l.bilinear(x3 + 2.5, z) - 3.75).abs() < 1e-12);
        // Across the seam: the last column is bare, column 0 covered.
        let x15 = grid.block_x(15) as f64;
        assert_eq!(l.cover(x15 + 1.0, z), 0.0);
        assert_eq!(l.cover(x15 + 2.0, z), 10.0);
        // At a lone covered cell's corner the nearest cell still decides: covered where it is the
        // nearest, as the block's class is (a share of the bilinear weight would leave it bare).
        let one = Raster::<u8>::from_fn(grid.clone(), |i, j| if (i, j) == (3, 2) { 6 } else { 0 });
        let one = LoadedLayer::new(
            LayerDesc::scalar("c", DType::U8, grid.clone(), Codec::Raw),
            one.to_le_bytes(),
        )
        .unwrap();
        assert_eq!(one.cover(x3 + 1.5, z + 1.5), 6.0);
        assert_eq!(one.cover(x3 + 2.5, z + 1.5), 0.0);
        for (dx, dz) in [(0.0, 0.0), (1.5, 1.5), (-1.9, 1.9), (2.0, 0.0), (0.5, -2.5)] {
            let near = one.nearest(x3 + dx, z + dz);
            assert_eq!(one.cover(x3 + dx, z + dz) > 0.0, near > 0.0, "({dx}, {dz})");
        }
    }

    #[test]
    fn nodata_is_nan() {
        let grid = GridDesc::for_planet("L", 64, 4).unwrap();
        let ras = Raster::<i16>::from_fn(grid.clone(), |i, _| if i == 3 { i16::MIN } else { 7 });
        let mut d = LayerDesc::scalar("w", DType::I16, grid, Codec::Raw);
        d.nodata = Some(f64::from(i16::MIN));
        let l = LoadedLayer::new(d, ras.to_le_bytes()).unwrap();
        assert!(l.get(3, 0).is_nan());
        assert_eq!(l.get(2, 0), 7.0);
        assert!(
            l.bilinear(l.desc.grid.block_x(2) as f64 + 1.0, 0.0)
                .is_nan()
        );
    }

    #[test]
    fn desc_json_round_trips() {
        let d = layer(64, 4).desc;
        let s = serde_json::to_string(&d).unwrap();
        assert!(s.contains("\"dtype\":\"i16\""), "{s}");
        let back: LayerDesc = serde_json::from_str(&s).unwrap();
        assert_eq!(back, d);
    }

    #[test]
    fn tile_rects_crop_edges() {
        let grid = GridDesc::for_planet("L1", 16384 * 2, 64).unwrap(); // 512 × 164
        let mut d = LayerDesc::scalar("x", DType::U8, grid, Codec::Raw);
        d.tile = 100;
        assert_eq!((d.tiles_x(), d.tiles_z()), (6, 2));
        assert_eq!(d.tile_rect(0), (0, 0, 100, 100));
        assert_eq!(d.tile_rect(5), (500, 0, 12, 100));
        assert_eq!(d.tile_rect(11), (500, 100, 12, 64));
    }

    #[test]
    fn a_cube_grid_is_written_by_its_side_and_read_back() {
        let g = GridDesc::for_cube("L3", 1024, 2).unwrap();
        assert_eq!((g.w, g.h, g.tile()), (512, 3072, 256));
        assert_eq!(GridDesc::for_cube("L1", 1024, 16).unwrap().tile(), 64);
        assert_eq!(GridDesc::for_cube("L3", 3840, 2).unwrap().tile(), 128);
        let s = serde_json::to_string(&g).unwrap();
        assert_eq!(s, r#"{"name":"L3","kind":"cube","n":512,"r":2}"#);
        let back: GridDesc = serde_json::from_str(&s).unwrap();
        assert_eq!(back, g);
        let odd = r#"{"name":"L3","kind":"cube","n":3,"r":1}"#;
        assert!(serde_json::from_str::<GridDesc>(odd).is_err());
        let hex = r#"{"name":"L3","kind":"hex","n":4,"r":1}"#;
        assert!(serde_json::from_str::<GridDesc>(hex).is_err());
        // A layer's descriptor carries it too.
        let d = LayerDesc::scalar("x", DType::F32, g, Codec::Raw);
        let back: LayerDesc = serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
        assert_eq!(back, d);
    }

    /// A field that is smooth on the planet, stored at the cube grid's cell centres, reads back
    /// bilinearly within a couple of cells' change of itself everywhere on the net, across every
    /// seam and fold: a wrong face or a wrong quarter turn would read cells far away.
    #[test]
    fn cube_bilinear_reads_are_continuous_across_every_seam() {
        use crate::cube::CubeTurn;
        use crate::cube_map::{CubeMap, CubeWorld};
        let s = 256u32;
        let world = CubeWorld::new(Cube::new(s), CubeMap::world(), CubeTurn::IDENTITY, 1.0);
        let view = world.view();
        let f = |p: [f64; 3]| 1000.0 * (0.6 * p[0] - 0.48 * p[1] + 0.64 * p[2]);
        let grid = GridDesc::for_cube("L", s, 4).unwrap();
        let mut bytes = Vec::with_capacity(grid.cells() * 4);
        for j in 0..grid.h {
            for i in 0..grid.w {
                let (x, z) = grid.cube_point(i, j);
                bytes.extend_from_slice(&(f(view.unit(x, z).unwrap()) as f32).to_le_bytes());
            }
        }
        let d = LayerDesc::scalar("f", DType::F32, grid, Codec::Raw);
        let l = LoadedLayer::new(d, bytes).unwrap();
        // The field changes at most 1000 / R per planet block, R = 2s/π blocks of the equal-area
        // sphere, and a block of the net spans at most about 1.3 of those.
        let per_block = 1000.0 / (2.0 * f64::from(s) / std::f64::consts::PI) * 1.3;
        let bound = 2.0 * 4.0 * per_block;
        let mut worst = 0.0f64;
        let h = i64::from(s);
        for bz in (-3 * h / 2..3 * h / 2).step_by(3) {
            for bx in (-3 * h / 2..5 * h / 2).step_by(3) {
                let read = l.bilinear(bx as f64, bz as f64);
                let Some(p) = view.unit(bx as f64 + 0.5, bz as f64 + 0.5) else {
                    assert!(read.is_nan(), "({bx}, {bz}) past the net read {read}");
                    continue;
                };
                let e = (read - f(p)).abs();
                worst = worst.max(e);
                assert!(e < bound, "({bx}, {bz}): read {read} for {}", f(p));
            }
        }
        assert!(worst > 0.0);
    }
}

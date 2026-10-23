// SPDX-License-Identifier: LGPL-3.0-or-later
//! The synthetic cube world (`deeptime synth --kind cube`, cube plan B10): format 0.7's first
//! world, for the game lane (phase 2) to read before any planet is baked to a cube. Its terrain is
//! defined on the sphere and sampled at the centres of the cube grids' cells through the world's
//! map and turn, so every net edge is seamless by construction, and its features sit where a
//! reader goes wrong first: a ridge across a quarter-turn seam, a cone on a reachable cube corner,
//! a lake across a half-turn seam, continents across the rest.

use deeptime_format::cube::CubeTurn;
use deeptime_format::cube_map::{CubeMap, CubeView};
use deeptime_format::projection::{WorldProjection, face_for};
use deeptime_format::{Codec, FormatError, GridDesc, LoadedLayer, Raster};
use serde_json::{Value, json};

use super::{
    CLIMATE, Kind, NODATA_I16, R_L1, R_L2, R_L3, Strata, SynthWorld, check_circumference,
    layer_i16, quantise, spawn_candidates, strata_layer,
};

/// The synthetic cube's turn: the parity fixture's free turn (azimuth −71.5°, tilt 38°, spin
/// 12°), so the poles lie off the face centres as on a baked world.
pub fn turn() -> CubeTurn {
    CubeTurn::from_angles(-71.5, 38.0, 12.0)
}

/// The cube world a synthetic planet of circumference `c` is drawn on: face side
/// [`face_for`]`(c)`, the world map, [`turn`].
pub fn projection(c: u32) -> WorldProjection {
    let map = CubeMap::builtin_static(deeptime_format::cube_map::WORLD_MAP).expect("the world map");
    WorldProjection::cube(face_for(c), map, turn())
}

/// One feature of the synthetic terrain, centred on a cube-frame direction.
struct Bump {
    /// Cube-frame centre (unit).
    centre: [f64; 3],
    /// Cube-frame unit tangent of the long axis at the centre (`None`: round).
    along: Option<[f64; 3]>,
    /// Height, blocks.
    height: f64,
    /// Standard deviations along and across, blocks of the sphere.
    sigma: (f64, f64),
}

fn norm(v: [f64; 3]) -> [f64; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / n, v[1] / n, v[2] / n]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// The synthetic cube's terrain, in the cube frame (so its features keep their places on the net
/// whatever the turn) except for latitude, which is the planet's.
pub struct CubeTerrain {
    view: CubeView<'static>,
    /// The sphere's radius, blocks.
    radius: f64,
    bumps: Vec<Bump>,
    /// The lake's centre (cube frame), radius and levels, blocks.
    lake: ([f64; 3], f64),
}

/// The lake's floor, its water surface and the plateau round it, blocks.
pub const LAKE_FLOOR_Y: f64 = 100.0;
pub const LAKE_WATER_Y: f64 = 108.0;
pub const LAKE_PLATEAU_Y: f64 = 116.0;

impl CubeTerrain {
    pub fn new(view: CubeView<'static>) -> CubeTerrain {
        let radius = view.cube.radius();
        // A ridge across the ne seam (N's east edge onto F1's top, a quarter turn): its long axis
        // runs across the seam line, which follows the cube's x axis.
        let ridge = Bump {
            centre: norm([0.0, 1.0, 1.0]),
            along: Some(norm([0.0, 1.0, -1.0])),
            height: 450.0,
            sigma: (radius * 0.25, radius * 0.06),
        };
        // A cone on corner n0, where F0, F1 and N meet (reachable: F0's north-east corner).
        let cone = Bump {
            centre: norm([1.0, 1.0, 1.0]),
            along: None,
            height: 300.0,
            sigma: (radius * 0.07, radius * 0.07),
        };
        // The plateau holding the lake across the nn seam (N's top onto F2's top, a half turn).
        let plateau = Bump {
            centre: norm([-1.0, 0.0, 1.0]),
            along: None,
            height: 0.0,
            sigma: (radius * 0.2, radius * 0.2),
        };
        CubeTerrain {
            view,
            radius,
            lake: (plateau.centre, radius * 0.06),
            bumps: vec![ridge, cone, plateau],
        }
    }

    /// The cube-frame unit vector of a planet unit vector.
    fn cube_vec(&self, p: [f64; 3]) -> [f64; 3] {
        self.view.turn.to_cube(p)
    }

    /// The great-circle distance from the lake's centre, blocks of the sphere.
    fn lake_distance(&self, q: [f64; 3]) -> f64 {
        self.radius * dot(q, self.lake.0).clamp(-1.0, 1.0).acos()
    }

    /// Terrain height at the planet unit vector `p`, blocks.
    pub fn height(&self, p: [f64; 3]) -> f64 {
        let q = self.cube_vec(p);
        // Two continents from the planet frame's x and a smaller wave, smooth everywhere.
        let mut h = 63.0 - 60.0 * p[0] + 25.0 * (4.0 * p[1] + 0.5).sin() * (3.0 * p[2]).cos();
        for b in &self.bumps[..2] {
            let c = dot(q, b.centre);
            if c <= 0.0 {
                continue;
            }
            let (da, db) = match b.along {
                Some(a) => {
                    let across = cross(b.centre, a);
                    (self.radius * dot(q, a), self.radius * dot(q, across))
                }
                None => {
                    let d = self.radius * c.clamp(-1.0, 1.0).acos();
                    (d, 0.0)
                }
            };
            h += b.height
                * (-(da * da) / (2.0 * b.sigma.0 * b.sigma.0)
                    - (db * db) / (2.0 * b.sigma.1 * b.sigma.1))
                    .exp();
        }
        // The plateau: flat at LAKE_PLATEAU_Y inside its sigma, blending out over the next one.
        let plateau = &self.bumps[2];
        let d = self.radius * dot(q, plateau.centre).clamp(-1.0, 1.0).acos();
        let s = plateau.sigma.0;
        let t = ((d - s) / s).clamp(0.0, 1.0);
        let blend = t * t * (3.0 - 2.0 * t);
        h = LAKE_PLATEAU_Y + (h - LAKE_PLATEAU_Y) * blend;
        // The lake's bowl: a flat floor inside the lake, a wall up to the plateau over 6 blocks.
        let dl = self.lake_distance(q);
        let r = self.lake.1;
        if dl < r + 6.0 {
            let w = ((dl - r) / 6.0).clamp(0.0, 1.0);
            h = h.min(LAKE_FLOOR_Y + (LAKE_PLATEAU_Y - LAKE_FLOOR_Y) * w);
        }
        h
    }

    /// The lake's water surface where it stands, blocks.
    pub fn water_level(&self, p: [f64; 3]) -> Option<f64> {
        let q = self.cube_vec(p);
        (self.lake_distance(q) < self.lake.1).then_some(LAKE_WATER_Y)
    }

    /// A climate layer's value at `p` (vanilla parameter units; the polar synthetic worlds'
    /// formulas, with the planet's latitude).
    pub fn raster(&self, layer: &str, p: [f64; 3], h: f64) -> f64 {
        let lat = p[2].clamp(-1.0, 1.0).asin();
        match layer {
            "climate/temperature" => 0.9 - 1.8 * lat.abs() / (std::f64::consts::PI / 2.0),
            "climate/continents" => (-1.0f64).max(1.0f64.min(-0.15 + (h - 63.0) / 80.0)),
            "climate/erosion" => 0.2,
            "climate/ridges" => 0.35,
            _ => 0.0,
        }
    }

    /// The notes `synthetic` of the manifest.
    pub fn notes(&self) -> Value {
        let (cube, turn) = (self.view.cube, self.view.turn);
        let at = |q: [f64; 3]| {
            let p = turn.to_planet(q);
            let (x, z) = self.view.point_of_unit(p);
            json!({ "x": x, "z": z })
        };
        json!({
            "kind": Kind::Cube.name(),
            "sampling": "a cube grid: cell (i, j) of a face holds the value at the centre of its r × r square, net point face origin + ((i + 0.5)·r, (j + 0.5)·r), drawn on the planet through the map and the rotation",
            "elevation": "round(16·H(p)) at the cell centre's planet unit vector p: 63 − 60·p_x + 25·sin(4·p_y + 0.5)·cos(3·p_z), a 450-block ridge across the ne seam, a 300-block cone on corner n0, a plateau at y 116 round the lake",
            "climate": "round(10000·v) of the polar synthetic worlds' formulas with the planet latitude asin(p_z)",
            "face_blocks": cube.side(),
            "ridge_ne_seam": at(norm([0.0, 1.0, 1.0])),
            "cone_n0": at(norm([1.0, 1.0, 1.0])),
            "lake_nn_seam": {
                "centre": at(self.lake.0),
                "radius_blocks": self.lake.1,
                "floor_y": LAKE_FLOOR_Y,
                "water_y": LAKE_WATER_Y,
                "plateau_y": LAKE_PLATEAU_Y
            }
        })
    }
}

/// Samples `f(planet unit vector)` at every cell centre of the cube grid `grid`, rows split across
/// threads.
fn sample_cube_i16(
    grid: &GridDesc,
    view: &CubeView<'static>,
    f: impl Fn([f64; 3]) -> i16 + Sync,
) -> Raster<i16> {
    let w = grid.w as usize;
    let mut data = vec![0i16; grid.cells()];
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(8);
    let rows_per = (grid.h as usize).div_ceil(threads).max(1);
    std::thread::scope(|s| {
        for (chunk_no, chunk) in data.chunks_mut(rows_per * w).enumerate() {
            let f = &f;
            s.spawn(move || {
                for (r, row) in chunk.chunks_mut(w).enumerate() {
                    let j = (chunk_no * rows_per + r) as u32;
                    for (i, cell) in row.iter_mut().enumerate() {
                        let (x, z) = grid.cube_point(i as u32, j);
                        let p = view.unit(x, z).expect("a cell centre lies on its face");
                        *cell = f(p);
                    }
                }
            });
        }
    });
    Raster {
        grid: grid.clone(),
        data,
    }
}

/// The synthetic cube world of a planet `c` blocks round (face side [`face_for`]`(c)`).
pub fn build(strata: Strata, c: u32, codec: Codec) -> Result<SynthWorld, FormatError> {
    check_circumference(c).map_err(|e| FormatError::Invalid(format!("--size {c}: {e}")))?;
    let proj = projection(c);
    let WorldProjection::Cube(view) = proj else {
        unreachable!("a cube projection")
    };
    let s = view.cube.side() as u32;
    let terrain = CubeTerrain::new(view);
    let l3 = GridDesc::for_cube("L3", s, R_L3)?;
    let l2 = GridDesc::for_cube("L2", s, R_L2)?;
    let l1 = GridDesc::for_cube("L1", s, R_L1)?;

    let mut layers: Vec<LoadedLayer> = Vec::new();
    let elev = sample_cube_i16(&l3, &view, |p| quantise(terrain.height(p) * 16.0));
    layers.push(layer_i16(
        "elevation",
        "block",
        1.0 / 16.0,
        None,
        &l3,
        codec,
        &elev,
    )?);
    let water = sample_cube_i16(&l3, &view, |p| match terrain.water_level(p) {
        Some(y) => quantise(y * 16.0),
        None => NODATA_I16,
    });
    layers.push(layer_i16(
        "water_level",
        "block",
        1.0 / 16.0,
        Some(NODATA_I16),
        &l3,
        codec,
        &water,
    )?);
    for name in CLIMATE {
        let ras = sample_cube_i16(&l2, &view, |p| {
            quantise(terrain.raster(name, p, terrain.height(p)) * 10_000.0)
        });
        layers.push(layer_i16(
            name,
            "vanilla_param",
            1e-4,
            None,
            &l2,
            codec,
            &ras,
        )?);
    }
    let strata_kind = strata;
    let strata = strata_layer(&l1, &layers[0], strata_kind, codec)?;
    let spawn = spawn_candidates(&layers[0], &layers[1], 63);
    Ok(SynthWorld {
        kind: Kind::Cube,
        strata_kind,
        circumference: c,
        grids: [l1, l2, l3],
        layers,
        strata,
        spawn,
        projection: Some(proj),
        notes: Some(terrain.notes()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::RasterRead;

    #[test]
    fn cube_synth_is_seamless_across_every_net_edge() {
        let w = build(Strata::Placeholder, 4096, Codec::Raw).unwrap();
        let elev = &w.layers[0];
        let g = &elev.desc.grid;
        assert!(g.is_cube());
        assert_eq!((g.w, g.h, g.r), (512, 6 * 512, 2));
        assert_eq!(elev.desc.tile, 256);
        let WorldProjection::Cube(view) = w.projection.unwrap() else {
            panic!("cube")
        };
        let terrain = CubeTerrain::new(view);
        // Bilinear reads just inside and just beyond every turning edge agree with the closed form
        // at the same planet point to well under a block (the grid's cells are 2 blocks).
        let cube = view.cube;
        let mut worst: f64 = 0.0;
        for e in cube.edges() {
            for t in 1..20 {
                let f = f64::from(t) / 20.0;
                let along = e.span[0] as f64 + f * (e.span[1] - e.span[0]) as f64;
                for depth in [-3.0, -0.5, 0.5, 3.0] {
                    let d = match e.beyond {
                        deeptime_format::projection::Side::Min => -depth,
                        deeptime_format::projection::Side::Max => depth,
                    };
                    let (bx, bz) = match e.axis {
                        deeptime_format::projection::Axis::X => (e.coord as f64 + d, along),
                        deeptime_format::projection::Axis::Z => (along, e.coord as f64 + d),
                    };
                    let read = elev.bilinear(bx, bz);
                    let p = view.unit(bx + 0.5, bz + 0.5).unwrap();
                    let want = quantise(terrain.height(p) * 16.0) as f64 / 16.0;
                    worst = worst.max((read - want).abs());
                }
            }
        }
        assert!(worst < 4.0, "worst bilinear error at the net edges {worst}");
        // The lake is where its notes say and holds water.
        let water = &w.layers[1];
        let notes = w.notes.as_ref().unwrap();
        let lx = notes["lake_nn_seam"]["centre"]["x"].as_f64().unwrap();
        let lz = notes["lake_nn_seam"]["centre"]["z"].as_f64().unwrap();
        assert_eq!(water.nearest(lx.floor(), lz.floor()), LAKE_WATER_Y);
    }
}

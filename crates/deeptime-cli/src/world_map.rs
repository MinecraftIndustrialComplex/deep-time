// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime world-map`: whole-world pictures of a baked world-data folder, in the world's own
//! block space (on a Petroff–Guyou world, format 0.5, the 2b × b rectangle itself), for reviewing a
//! bake without the workbench:
//!
//! - `satellite.png`: the workbench's "Final world" satellite look (`deeptime_tiles`), its tile
//!   pyramid at `--zoom` stitched (2^(z+1) × 2^z tiles of 256 pixels);
//! - `provinces.png`: the geologic provinces (`meta/province`, each id its own colour), the sea
//!   darkened, the cone points (red) and the poles (white) of the world's layout marked;
//! - `world-map.json`: the sizes, the projection block and the marks' pixels.
//!
//! A cube world (format 0.7) is drawn as its net instead: the window x in [−3s/2, 5s/2), z in
//! [−3s/2, 3s/2] (4s by 3s blocks, s the face side), one pixel per 4s / (2^(z+1) · 256) blocks.
//! Pixels off the six faces (half the window) are the dark void colour. Each pixel is read at its
//! centre (a block read is shifted by half a block to land there).
//!
//! - `provinces.png`: the same province colours (a world with no `meta/province` layer, such as
//!   the synthetic cube, gets one colour per face), the sea darkened; the six face outlines (grey,
//!   one pixel), the 12 turning edges the border names (yellow, three pixels), the eight cube
//!   corners (red squares, one per image) and both poles (white squares);
//! - `satellite.png`: a plain picture of the elevation layer alone (a blue ramp under the sea, a
//!   green to brown to white ramp above it) with a hillshade lit from the north west; the
//!   tile pyramid of the polar pictures is not used;
//! - `world-map.json`: also `kind`, `face_blocks` and `window` [x, z, width, height]; a mark per
//!   border edge (`border <name>`, its line and pixels), per corner image and per pole image.

use std::path::Path;

use deeptime_format::cube::{Cube, Face};
use deeptime_format::projection::{Axis, ProjectionKind, ProjectionSpec};
use deeptime_format::{LoadedLayer, Manifest, RasterRead, TilePackReader};
use serde_json::{Value, json};

use deeptime_tiles::{BakeMap, Layer, TILE, TileGeom};

/// A colour of its own for each id (splitmix64 of the id).
fn colour(id: u64) -> [u8; 3] {
    let mut x = id.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    [
        80 + (x & 0x9f) as u8,
        80 + ((x >> 8) & 0x9f) as u8,
        80 + ((x >> 16) & 0x9f) as u8,
    ]
}

/// The dark colour of what is not the planet.
const VOID: [u8; 3] = [4, 6, 10];
/// A cell with no value.
const NO_DATA: [u8; 3] = [60, 60, 60];

/// Opens a layer of the world by name; `None` when the world has none.
fn open_layer(manifest: &Manifest, dir: &Path, name: &str) -> Result<Option<LoadedLayer>, String> {
    match manifest.layer(name) {
        Some(e) => Ok(Some(
            TilePackReader::open(&dir.join(&e.path))
                .and_then(|r| r.load())
                .map_err(|e| e.to_string())?,
        )),
        None => Ok(None),
    }
}

/// A mark's square: 7 × 7 pixels centred on pixel (`pi`, `pj`), clipped to the `w` × `h` picture.
fn stamp(rgb: &mut [u8], w: usize, h: usize, pi: i64, pj: i64, col: [u8; 3]) {
    for dj in -3..=3 {
        for di in -3..=3 {
            let (ii, jj) = (pi + di, pj + dj);
            if ii >= 0 && jj >= 0 && (ii as usize) < w && (jj as usize) < h {
                let at = (jj as usize * w + ii as usize) * 3;
                rgb[at..at + 3].copy_from_slice(&col);
            }
        }
    }
}

/// A mark of the summary: block corners (the polar layouts) as integers, other net points as
/// written.
fn point_mark(what: &str, bx: f64, bz: f64, pi: i64, pj: i64) -> Value {
    let num = |v: f64| {
        if v.fract() == 0.0 {
            json!(v as i64)
        } else {
            json!(v)
        }
    };
    json!({"what": what, "x": num(bx), "z": num(bz), "px": [pi, pj]})
}

/// Writes the pictures of world folder `dir` into `out`; returns the summary.
pub fn run(dir: &Path, zoom: u32, out: &Path) -> Result<Value, String> {
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let manifest = Manifest::read(dir).map_err(|e| e.to_string())?;
    if let Some(spec) = &manifest.projection
        && spec.kind == ProjectionKind::Cube
    {
        let s = spec
            .face_blocks
            .ok_or("a cube world's projection has no face_blocks")?;
        return run_cube(dir, &manifest, spec, s, zoom, out);
    }
    let c = manifest.scale.circumference_blocks;
    let map = BakeMap::open(dir)?;
    let (nx, ny) = (2usize << zoom, 1usize << zoom);
    let (w, h) = (nx * TILE, ny * TILE);
    let mut rgba = vec![0u8; w * h * 4];
    for ty in 0..ny {
        for tx in 0..nx {
            let g = TileGeom::new(c, zoom, tx as u32, ty as u32)
                .ok_or_else(|| format!("zoom {zoom} is past this world's pyramid"))?;
            let tile = map.render(Layer::Satellite, &g)?;
            for j in 0..TILE {
                let at = ((ty * TILE + j) * w + tx * TILE) * 4;
                rgba[at..at + TILE * 4].copy_from_slice(&tile[j * TILE * 4..(j + 1) * TILE * 4]);
            }
        }
    }
    let png = deeptime_viewer::encode_png(w as u32, h as u32, &rgba).map_err(|e| e.to_string())?;
    std::fs::write(out.join("satellite.png"), png).map_err(|e| e.to_string())?;

    // Provinces: one pixel per `s` blocks over the 2:1 frame from (−C/2, −C/4).
    let s = f64::from(c) / w as f64;
    let (x0, z0) = (-f64::from(c) / 2.0, -f64::from(c) / 4.0);
    let prov = open_layer(&manifest, dir, "meta/province")?
        .ok_or("the world has no meta/province layer")?;
    let elev =
        open_layer(&manifest, dir, "elevation")?.ok_or("the world has no elevation layer")?;
    let sea = f64::from(manifest.scale.sea_level_y);
    let wp = manifest.world_projection();
    let (zlo, zhi) = wp.z_range();
    let mut rgb = vec![0u8; w * h * 3];
    for j in 0..h {
        let z = z0 + (j as f64 + 0.5) * s;
        for i in 0..w {
            let x = x0 + (i as f64 + 0.5) * s;
            let at = (j * w + i) * 3;
            if z < zlo || z > zhi {
                rgb[at..at + 3].copy_from_slice(&VOID);
                continue;
            }
            let id = prov.nearest(x, z);
            let mut col = if id.is_finite() {
                colour(id as u64)
            } else {
                NO_DATA
            };
            if elev.bilinear(x, z) < sea {
                col = col.map(|v| v / 3);
            }
            rgb[at..at + 3].copy_from_slice(&col);
        }
    }
    // The layout's marks: cone points red, poles white (a small square each).
    let mut marks = Vec::new();
    if let Some(spec) = &manifest.projection {
        let mut put = |bx: f64, bz: f64, col: [u8; 3], what: &str| {
            let pi = ((bx - x0) / s).round() as i64;
            let pj = ((bz - z0) / s).round() as i64;
            stamp(&mut rgb, w, h, pi, pj, col);
            marks.push(point_mark(what, bx, bz, pi, pj));
        };
        for cp in &spec.cone_points {
            for im in &cp.images {
                put(im[0] as f64, im[1] as f64, CORNER, &cp.name);
            }
        }
        for p in &spec.poles {
            for im in &p.images {
                put(im[0], im[1], POLE, &format!("{} pole", p.name));
            }
        }
    }
    let png =
        deeptime_viewer::encode_png_rgb(w as u32, h as u32, &rgb).map_err(|e| e.to_string())?;
    std::fs::write(out.join("provinces.png"), png).map_err(|e| e.to_string())?;
    let summary = json!({
        "world": dir.display().to_string(),
        "pixels": [w, h],
        "blocks_per_pixel": s,
        "circumference": c,
        "projection": manifest.projection,
        "marks": marks,
    });
    std::fs::write(
        out.join("world-map.json"),
        serde_json::to_vec_pretty(&summary).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(summary)
}

/// A cone point's square.
const CORNER: [u8; 3] = [230, 40, 40];
/// A pole's square.
const POLE: [u8; 3] = [255, 255, 255];
/// A face's outline in a cube world's picture.
const OUTLINE: [u8; 3] = [150, 150, 150];
/// A turning edge of the net (a border segment).
const BORDER: [u8; 3] = [240, 200, 40];
/// The border lines are this many pixels wide.
const BORDER_PIXELS: i64 = 3;

/// Writes the net pictures of cube world `dir` (side `s`) into `out`; returns the summary.
fn run_cube(
    dir: &Path,
    manifest: &Manifest,
    spec: &ProjectionSpec,
    s: u32,
    zoom: u32,
    out: &Path,
) -> Result<Value, String> {
    let pics = cube_pictures(dir, manifest, spec, s, zoom)?;
    let (w, h) = (pics.frame.w, pics.frame.h);
    for (name, rgb) in [
        ("satellite.png", &pics.satellite),
        ("provinces.png", &pics.provinces),
    ] {
        let png =
            deeptime_viewer::encode_png_rgb(w as u32, h as u32, rgb).map_err(|e| e.to_string())?;
        std::fs::write(out.join(name), png).map_err(|e| e.to_string())?;
    }
    let side = f64::from(s);
    let summary = json!({
        "world": dir.display().to_string(),
        "kind": "cube",
        "face_blocks": s,
        "window": [pics.frame.x0, pics.frame.z0, 4.0 * side, 3.0 * side],
        "pixels": [w, h],
        "blocks_per_pixel": pics.frame.bpp,
        "circumference": manifest.scale.circumference_blocks,
        "projection": manifest.projection,
        "marks": pics.marks,
    });
    std::fs::write(
        out.join("world-map.json"),
        serde_json::to_vec_pretty(&summary).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(summary)
}

/// The picture of a cube world's net window: pixel (i, j) is the point at its centre.
struct Frame {
    /// The window's north-west corner, blocks.
    x0: f64,
    z0: f64,
    /// Blocks per pixel.
    bpp: f64,
    w: usize,
    h: usize,
}

impl Frame {
    /// The window of a cube of face side `s`, as wide as the polar pictures at `zoom`.
    fn new(s: u32, zoom: u32) -> Frame {
        let side = f64::from(s);
        let w = (2usize << zoom) * TILE;
        let bpp = 4.0 * side / w as f64;
        Frame {
            x0: -1.5 * side,
            z0: -1.5 * side,
            bpp,
            w,
            h: (3.0 * side / bpp).round() as usize,
        }
    }

    /// The net point at the centre of pixel (i, j).
    fn point(&self, i: usize, j: usize) -> (f64, f64) {
        (
            self.x0 + (i as f64 + 0.5) * self.bpp,
            self.z0 + (j as f64 + 0.5) * self.bpp,
        )
    }

    /// The continuous pixel column of the block coordinate x (pixel i spans [i, i + 1)).
    fn col(&self, x: f64) -> f64 {
        (x - self.x0) / self.bpp
    }

    /// The continuous pixel row of the block coordinate z.
    fn row(&self, z: f64) -> f64 {
        (z - self.z0) / self.bpp
    }

    /// The pixel of a one-pixel line at pixel position `pos`: the pixel after it, kept inside the
    /// `n` pixels.
    fn thin(pos: f64, n: usize) -> i64 {
        (pos.round() as i64).clamp(0, n as i64 - 1)
    }

    /// The pixels of a [`BORDER_PIXELS`] wide line at pixel position `pos`, kept inside the `n`
    /// pixels: first and last.
    fn thick(pos: f64, n: usize) -> (i64, i64) {
        let lo =
            ((pos - BORDER_PIXELS as f64 / 2.0).ceil() as i64).clamp(0, n as i64 - BORDER_PIXELS);
        (lo, lo + BORDER_PIXELS - 1)
    }

    /// The pixels a span of pixel positions [a, b) covers, first and last, kept inside `n`.
    fn run(a: f64, b: f64, n: usize) -> (i64, i64) {
        (
            (a.floor() as i64).max(0),
            ((b.ceil() as i64) - 1).min(n as i64 - 1),
        )
    }

    /// Fills the pixels in the inclusive ranges, clipped to the picture.
    fn fill(&self, rgb: &mut [u8], cols: (i64, i64), rows: (i64, i64), col: [u8; 3]) {
        for j in rows.0.max(0)..=rows.1.min(self.h as i64 - 1) {
            for i in cols.0.max(0)..=cols.1.min(self.w as i64 - 1) {
                let at = (j as usize * self.w + i as usize) * 3;
                rgb[at..at + 3].copy_from_slice(&col);
            }
        }
    }
}

/// What the net pictures hold.
struct CubePictures {
    frame: Frame,
    /// RGB8, `frame.w · frame.h · 3` bytes each.
    provinces: Vec<u8>,
    satellite: Vec<u8>,
    marks: Vec<Value>,
}

/// Draws cube world `dir` (face side `s`) at `zoom`.
fn cube_pictures(
    dir: &Path,
    manifest: &Manifest,
    spec: &ProjectionSpec,
    s: u32,
    zoom: u32,
) -> Result<CubePictures, String> {
    let cube = Cube::shared(s);
    let fr = Frame::new(s, zoom);
    let (w, h) = (fr.w, fr.h);
    let prov = open_layer(manifest, dir, "meta/province")?;
    let elev = open_layer(manifest, dir, "elevation")?.ok_or("the world has no elevation layer")?;
    let sea = f64::from(manifest.scale.sea_level_y);

    // Which face each pixel is on (None: the void), its elevation and its province colour. A read
    // takes the block whose centre is the pixel's point, so the block is the point less half a
    // block. Reads in the void bands beside N and S fold onto real faces; they feed only the
    // hillshade's differences at the faces' edges.
    let mut face_of: Vec<Option<Face>> = vec![None; w * h];
    let mut height = vec![f32::NAN; w * h];
    let mut provinces = vec![0u8; w * h * 3];
    for j in 0..h {
        for i in 0..w {
            let at = j * w + i;
            let (x, z) = fr.point(i, j);
            let (bx, bz) = (x - 0.5, z - 0.5);
            let e = elev.bilinear(bx, bz);
            height[at] = e as f32;
            face_of[at] = cube.local_of_point(x, z).map(|(face, _, _)| face);
            let col = match face_of[at] {
                None => VOID,
                Some(face) => {
                    let mut col = match &prov {
                        Some(p) => {
                            let id = p.nearest(bx, bz);
                            if id.is_finite() {
                                colour(id as u64)
                            } else {
                                NO_DATA
                            }
                        }
                        None => colour(face as u64),
                    };
                    if e < sea {
                        col = col.map(|v| v / 3);
                    }
                    col
                }
            };
            provinces[at * 3..at * 3 + 3].copy_from_slice(&col);
        }
    }
    let satellite = satellite(&fr, &face_of, &height, sea);

    // The six faces' outlines, one pixel.
    let side = i64::from(s);
    for face in Face::ALL {
        let (ox, oz) = cube.face_origin(face);
        let (c0, c1) = (
            Frame::thin(fr.col(ox as f64), w),
            Frame::thin(fr.col((ox + side) as f64), w),
        );
        let (r0, r1) = (
            Frame::thin(fr.row(oz as f64), h),
            Frame::thin(fr.row((oz + side) as f64), h),
        );
        for row in [r0, r1] {
            fr.fill(&mut provinces, (c0, c1), (row, row), OUTLINE);
        }
        for col in [c0, c1] {
            fr.fill(&mut provinces, (col, col), (r0, r1), OUTLINE);
        }
    }

    // The border: the turning edges, three pixels wide.
    let mut marks = Vec::new();
    let segments = spec
        .border
        .as_ref()
        .map(|b| b.segments.as_slice())
        .unwrap_or_default();
    for name in segments {
        let e = cube
            .edges()
            .iter()
            .find(|e| e.name == *name)
            .ok_or_else(|| format!("the border names {name}, which is no net edge of side {s}"))?;
        let (band, span, axis) = match e.axis {
            Axis::X => (
                Frame::thick(fr.col(e.coord as f64), w),
                Frame::run(fr.row(e.span[0] as f64), fr.row(e.span[1] as f64), h),
                "x",
            ),
            Axis::Z => (
                Frame::thick(fr.row(e.coord as f64), h),
                Frame::run(fr.col(e.span[0] as f64), fr.col(e.span[1] as f64), w),
                "z",
            ),
        };
        match e.axis {
            Axis::X => fr.fill(&mut provinces, band, span, BORDER),
            Axis::Z => fr.fill(&mut provinces, span, band, BORDER),
        }
        marks.push(json!({
            "what": format!("border {name}"),
            "axis": axis,
            "coord": e.coord,
            "span": e.span,
            "band_px": [band.0, band.1],
            "span_px": [span.0, span.1],
        }));
    }

    // The eight cube corners (each image) and both poles.
    let mut put = |bx: f64, bz: f64, col: [u8; 3], what: &str| {
        let pi = fr.col(bx).round() as i64;
        let pj = fr.row(bz).round() as i64;
        stamp(&mut provinces, w, h, pi, pj, col);
        marks.push(point_mark(what, bx, bz, pi, pj));
    };
    for cp in &spec.cone_points {
        for im in &cp.images {
            put(im[0] as f64, im[1] as f64, CORNER, &cp.name);
        }
    }
    for p in &spec.poles {
        for im in &p.images {
            put(im[0], im[1], POLE, &format!("{} pole", p.name));
        }
    }
    Ok(CubePictures {
        frame: fr,
        provinces,
        satellite,
        marks,
    })
}

/// A piecewise linear colour ramp: the colour at `t` between the stops (value, colour), the end
/// colours beyond them.
fn ramp(stops: &[(f64, [u8; 3])], t: f64) -> [u8; 3] {
    let first = stops[0];
    if t <= first.0 {
        return first.1;
    }
    for pair in stops.windows(2) {
        let ((t0, c0), (t1, c1)) = (pair[0], pair[1]);
        if t <= t1 {
            let f = (t - t0) / (t1 - t0);
            return [0, 1, 2].map(|k| {
                (f64::from(c0[k]) + f * (f64::from(c1[k]) - f64::from(c0[k]))).round() as u8
            });
        }
    }
    stops[stops.len() - 1].1
}

/// Land above the sea by this many blocks: green on the shore, brown, grey rock, snow.
const LAND_RAMP: [(f64, [u8; 3]); 6] = [
    (0.0, [96, 140, 70]),
    (30.0, [68, 124, 54]),
    (80.0, [142, 126, 76]),
    (140.0, [122, 96, 76]),
    (200.0, [170, 165, 160]),
    (260.0, [250, 250, 250]),
];

/// Sea this many blocks deep: light blue on the shelf, dark blue in the deep.
const SEA_RAMP: [(f64, [u8; 3]); 4] = [
    (0.0, [110, 175, 215]),
    (10.0, [60, 130, 190]),
    (40.0, [25, 70, 140]),
    (100.0, [8, 22, 70]),
];

/// The satellite-like picture of the elevation `height` (blocks, NaN where unread) on the pixels
/// that lie on a face: a colour ramp about the sea level `sea`, with a hillshade lit from the north
/// west whose slopes are the differences between neighbouring pixels (a pixel off the picture or
/// with no value is replaced by the pixel itself, a one-sided difference).
fn satellite(fr: &Frame, face_of: &[Option<Face>], height: &[f32], sea: f64) -> Vec<u8> {
    let (w, h) = (fr.w, fr.h);
    // The direction to the sun in the net frame (x east, y up, z south), unit length.
    let sun = {
        let v = [-1.0f64, 1.2, -1.0];
        let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        v.map(|c| c / n)
    };
    let mut rgb = vec![0u8; w * h * 3];
    for j in 0..h {
        for i in 0..w {
            let at = j * w + i;
            let e = f64::from(height[at]);
            let col = if face_of[at].is_none() {
                VOID
            } else if !e.is_finite() {
                NO_DATA
            } else {
                let (ie, iw) = ((i + 1).min(w - 1), i.saturating_sub(1));
                let (js, jn) = ((j + 1).min(h - 1), j.saturating_sub(1));
                let near = |k: usize| {
                    let v = f64::from(height[k]);
                    if v.is_finite() { v } else { e }
                };
                let dx = (near(j * w + ie) - near(j * w + iw)) / ((ie - iw) as f64 * fr.bpp);
                let dz = (near(js * w + i) - near(jn * w + i)) / ((js - jn) as f64 * fr.bpp);
                // The surface normal (−dy/dx, 1, −dy/dz); the light on it, 1 on flat ground.
                let len = (dx * dx + dz * dz + 1.0).sqrt();
                let lit = (-dx * sun[0] + sun[1] - dz * sun[2]) / len / sun[1];
                let mut shade = lit.clamp(0.35, 1.5);
                let base = if e < sea {
                    // The sea floor shows through faintly.
                    shade = 1.0 + 0.4 * (shade - 1.0);
                    ramp(&SEA_RAMP, sea - e)
                } else {
                    ramp(&LAND_RAMP, e - sea)
                };
                base.map(|v| (f64::from(v) * shade).round().clamp(0.0, 255.0) as u8)
            };
            rgb[at * 3..at * 3 + 3].copy_from_slice(&col);
        }
    }
    rgb
}

#[cfg(test)]
mod tests {
    use deeptime_format::Codec;

    use super::*;
    use crate::synth::{Kind, Options, Strata};

    /// The PNG's width and height (the IHDR chunk's first eight data bytes).
    fn png_size(path: &Path) -> (u32, u32) {
        let b = std::fs::read(path).unwrap();
        assert_eq!(&b[1..4], b"PNG", "{}", path.display());
        let n = |at: usize| u32::from_be_bytes(b[at..at + 4].try_into().unwrap());
        (n(16), n(20))
    }

    fn px(rgb: &[u8], w: usize, i: usize, j: usize) -> [u8; 3] {
        let at = (j * w + i) * 3;
        [rgb[at], rgb[at + 1], rgb[at + 2]]
    }

    #[test]
    fn cube_world_is_drawn_as_its_net() {
        let dir = std::env::temp_dir().join(format!("dt-worldmap-{}", std::process::id()));
        let world = dir.join("w");
        let pics = dir.join("pics");
        crate::synth::run(&Options {
            kind: Kind::Cube,
            strata: Strata::Placeholder,
            size: 4096,
            out: world.clone(),
            viewer: None,
            dump_samples: None,
            codec: Codec::Lz4Block,
        })
        .unwrap();

        // Zoom 0: 512 pixels wide over 4s = 4096 blocks, 8 blocks a pixel, 384 high.
        let v = run(&world, 0, &pics).unwrap();
        assert_eq!(png_size(&pics.join("provinces.png")), (512, 384));
        assert_eq!(png_size(&pics.join("satellite.png")), (512, 384));
        let json: Value =
            serde_json::from_slice(&std::fs::read(pics.join("world-map.json")).unwrap()).unwrap();
        assert_eq!(json, v);
        assert_eq!(json["kind"], "cube");
        assert_eq!(json["face_blocks"], 1024);
        assert_eq!(json["window"], json!([-1536.0, -1536.0, 4096.0, 3072.0]));
        assert_eq!(json["pixels"], json!([512, 384]));
        assert_eq!(json["blocks_per_pixel"], 8.0);

        // The marks: the 12 border edges, one per image of each corner, one per image of a pole.
        let manifest = Manifest::read(&world).unwrap();
        let spec = manifest.projection.as_ref().unwrap();
        let marks = json["marks"].as_array().unwrap();
        let named = |f: &dyn Fn(&str) -> bool| {
            marks
                .iter()
                .filter(|m| f(m["what"].as_str().unwrap()))
                .count()
        };
        let corners: usize = spec.cone_points.iter().map(|c| c.images.len()).sum();
        let poles: usize = spec.poles.iter().map(|p| p.images.len()).sum();
        assert!(
            corners >= 8 && poles >= 2,
            "{corners} corners, {poles} poles"
        );
        assert_eq!(named(&|w| w.starts_with("border ")), 12);
        assert_eq!(named(&|w| w.ends_with(" pole")), poles);
        assert_eq!(marks.len(), 12 + corners + poles);
        let border_names: Vec<&str> = marks
            .iter()
            .filter_map(|m| m["what"].as_str()?.strip_prefix("border "))
            .collect();
        for seg in &spec.border.as_ref().unwrap().segments {
            assert!(border_names.contains(&seg.as_str()), "{seg}");
        }

        // The pixels, with the frame (x0 = z0 = −1536, 8 blocks a pixel): the window's corner and
        // the square beside N are void; F0's centre and N's centre are not; the line between F0
        // and F1 (x = 512, column 256) is grey; the edge N-east (the same column, rows 0 to 127)
        // is yellow.
        let pics = cube_pictures(&world, &manifest, spec, 1024, 0).unwrap();
        let (w, h) = (pics.frame.w, pics.frame.h);
        assert_eq!((w, h), (512, 384));
        for (name, rgb) in [
            ("provinces", &pics.provinces),
            ("satellite", &pics.satellite),
        ] {
            assert_eq!(px(rgb, w, 0, 0), VOID, "{name}: window corner");
            assert_eq!(px(rgb, w, 64, 64), VOID, "{name}: square beside N");
            assert_eq!(px(rgb, w, 448, 320), VOID, "{name}: square beside S");
            assert_ne!(px(rgb, w, 192, 192), VOID, "{name}: F0");
            assert_ne!(px(rgb, w, 192, 64), VOID, "{name}: N");
            assert_ne!(px(rgb, w, 192, 320), VOID, "{name}: S");
            assert_ne!(px(rgb, w, 448, 192), VOID, "{name}: F2");
            assert_ne!(px(rgb, w, 64, 192), VOID, "{name}: F3");
        }
        assert_eq!(px(&pics.provinces, w, 256, 192), OUTLINE);
        assert_eq!(px(&pics.provinces, w, 256, 64), BORDER);
        assert_ne!(px(&pics.satellite, w, 256, 64), BORDER);
        // The land and the sea differ in the satellite: its blue is deeper than its red under the
        // sea, and not on a green hill.
        let blue = |c: [u8; 3]| c[2] > c[0];
        let (mut sea_px, mut land_px) = (0, 0);
        for j in 0..h {
            for i in 0..w {
                if px(&pics.satellite, w, i, j) != VOID {
                    if blue(px(&pics.satellite, w, i, j)) {
                        sea_px += 1;
                    } else {
                        land_px += 1;
                    }
                }
            }
        }
        assert!(
            sea_px > 1000 && land_px > 1000,
            "{sea_px} sea, {land_px} land"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

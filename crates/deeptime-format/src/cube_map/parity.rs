// SPDX-License-Identifier: LGPL-3.0-or-later
//! The cube section of the cross-language parity fixture (`tests/data/projection_parity.json`,
//! written by `projection::parity_fixture`; plan M5). Java's `ProjectionParityTest` and the
//! pages' `tools/viewer/projection-check.js` read it and must give the same bits:
//!
//! - `face_for`: circumference → face side (W1);
//! - `net`: one net's edges, corners and the face adjacency, then [`Cube::fold`] on blocks and
//!   [`Cube::fold_point`] on points (W2);
//! - `maps`: each shipped table's name and sha256, then forward and jacobian at local points
//!   (every face's edges, within one block at 32k of every corner, the corners themselves, the
//!   diagonals, the axes and random points) and the inverse of their unit vectors and of the
//!   axes, the corners' directions and random directions (M2);
//! - `worlds`: a [`CubeWorld`] on each table with its turn's rows, and at net points (inside,
//!   on the face lines, round every corner image, beyond the net's edges and past its corners)
//!   the unit vector, latitude, longitude and the metres-per-block Jacobian; latitude and
//!   longitude back to net points; both poles and the eight corners found through the turn.
//!
//! Floats are the hex of their bits; integers plain numbers. A `null` is a `None`.

use super::*;
use crate::cube::{ADJACENCY, CubeFold};
use crate::projection::{Axis, Side, face_for};

fn hx(v: f64) -> String {
    format!("\"{:016x}\"", v.to_bits())
}

fn hx3(p: [f64; 3]) -> String {
    format!("[{}, {}, {}]", hx(p[0]), hx(p[1]), hx(p[2]))
}

/// The same SplitMix64 as the rest of the fixture, on its own seed (the cube rows draw nothing
/// from the older sections' stream).
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

    fn int(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.f() * (hi - lo) as f64) as i64
    }
}

/// One block of the 32k world's face (s = 7,424) in local units.
const BLOCK_32K: f64 = 2.0 / 7424.0;

/// The local points sampled on every face.
fn face_points(g: &mut Rng) -> Vec<(Face, f64, f64)> {
    let mut pts = Vec::new();
    for face in Face::ALL {
        pts.push((face, 0.0, 0.0));
        // On each side's edge.
        for t in [-0.75, -0.3, 0.0, 0.5, g.sym()] {
            pts.push((face, 1.0, t));
            pts.push((face, -1.0, t));
            pts.push((face, t, 1.0));
            pts.push((face, t, -1.0));
        }
        // Each corner, within one block of it, and on its edges within one block.
        for (su, sv) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
            pts.push((face, su, sv));
            for _ in 0..3 {
                let (du, dv) = (g.f() * BLOCK_32K, g.f() * BLOCK_32K);
                pts.push((face, su * (1.0 - du), sv * (1.0 - dv)));
            }
            pts.push((face, su, sv * (1.0 - g.f() * BLOCK_32K)));
            pts.push((face, su * (1.0 - g.f() * BLOCK_32K), sv));
        }
        // The diagonals and the axes (the triangle's swap and sign lines).
        let t = g.f();
        pts.push((face, t, t));
        pts.push((face, -t, t));
        pts.push((face, t, 0.0));
        pts.push((face, 0.0, -t));
        for _ in 0..10 {
            pts.push((face, g.sym(), g.sym()));
        }
    }
    pts
}

/// Directions whose face is decided by a tie or that sit on a line of the inverse.
fn special_directions() -> Vec<[f64; 3]> {
    let r2 = 0.5f64.sqrt();
    let r3 = 1.0 / 3.0f64.sqrt();
    let mut d = vec![
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
        [r2, r2, 0.0],
        [-r2, r2, 0.0],
        [r2, 0.0, -r2],
        [0.0, -r2, r2],
        [2.0, 0.0, 0.0],
        [0.3, -0.0, 0.0],
    ];
    for sx in [1.0, -1.0] {
        for sy in [1.0, -1.0] {
            for sz in [1.0, -1.0] {
                d.push([sx * r3, sy * r3, sz * r3]);
            }
        }
    }
    d
}

fn face_section(g: &mut Rng) -> String {
    let rows: Vec<String> = [1u32, 1000, 4096, 9000, 16384, 20000, 32768, 65536, 131072]
        .into_iter()
        .chain((0..6).map(|_| g.int(1, 200_000) as u32))
        .map(|c| format!("{{\"c\": {c}, \"s\": {}}}", face_for(c)))
        .collect();
    format!("    \"face_for\": [{}],\n", rows.join(", "))
}

fn axis_name(a: Axis) -> &'static str {
    match a {
        Axis::X => "x",
        Axis::Z => "z",
    }
}

fn side_name(s: Side) -> &'static str {
    match s {
        Side::Min => "min",
        Side::Max => "max",
    }
}

fn iso_json(iso: crate::cube::Iso) -> String {
    let [k, tx, tz] = iso.as_array();
    format!("[{k}, {tx}, {tz}]")
}

fn net_section(g: &mut Rng) -> String {
    let s: u32 = 1024;
    let cube = Cube::new(s);
    let si = i64::from(s);
    let h = si / 2;
    let mut out = format!("    \"net\": {{\"s\": {s},\n      \"edges\": [\n");
    let edges: Vec<String> = cube
        .edges()
        .iter()
        .map(|e| {
            format!(
                "        {{\"name\": \"{}\", \"seam\": \"{}\", \"motion\": \"{}\", \"face\": {}, \"face_side\": {}, \"axis\": \"{}\", \"beyond\": \"{}\", \"coord\": {}, \"span\": [{}, {}], \"iso\": {}, \"pivot\": {}, \"partner\": {}, \"owner\": \"{}\"}}",
                e.name,
                e.seam.name(),
                e.motion.name(),
                e.face.index(),
                e.face_side.index(),
                axis_name(e.axis),
                side_name(e.beyond),
                e.coord,
                e.span[0],
                e.span[1],
                iso_json(e.iso),
                e.pivot
                    .map_or("null".into(), |p| format!("[{}, {}]", p[0], p[1])),
                e.partner,
                e.owner
            )
        })
        .collect();
    out.push_str(&edges.join(",\n"));
    out.push_str("\n      ],\n      \"corners\": [\n");
    let corners: Vec<String> = cube
        .corners()
        .iter()
        .map(|c| {
            let images: Vec<String> = c
                .images
                .iter()
                .map(|p| format!("[{}, {}]", p[0], p[1]))
                .collect();
            format!(
                "        {{\"name\": \"{}\", \"faces\": [{}, {}, {}], \"cube\": [{}, {}, {}], \"images\": [{}]}}",
                c.name,
                c.faces[0].index(),
                c.faces[1].index(),
                c.faces[2].index(),
                c.cube[0],
                c.cube[1],
                c.cube[2],
                images.join(", ")
            )
        })
        .collect();
    out.push_str(&corners.join(",\n"));
    // [face][side]: [neighbour face, the side it enters through, quarter turns, shift u, shift v].
    out.push_str("\n      ],\n      \"adjacency\": [");
    let adj: Vec<String> = ADJACENCY
        .iter()
        .map(|row| {
            let cells: Vec<String> = row
                .iter()
                .map(|a| {
                    format!(
                        "[{}, {}, {}, {}, {}]",
                        a.face.index(),
                        a.side.index(),
                        a.quarter_turns,
                        a.shift[0],
                        a.shift[1]
                    )
                })
                .collect();
            format!("[{}]", cells.join(", "))
        })
        .collect();
    out.push_str(&adj.join(", "));
    // Blocks: the window and a face width round it, the lines either side of every net edge and
    // the void squares' diagonals.
    out.push_str("],\n      \"blocks\": [\n");
    let mut blocks: Vec<(i64, i64)> = (0..240)
        .map(|_| (g.int(-5 * h - 4, 9 * h + 4), g.int(-5 * h - 4, 5 * h + 4)))
        .collect();
    for e in cube.edges() {
        for _ in 0..3 {
            let t = g.int(e.span[0], e.span[1]);
            for d in [-1, 0, 1, si - 1, si] {
                let n = match e.beyond {
                    Side::Min => e.coord - 1 - d,
                    Side::Max => e.coord + d,
                };
                blocks.push(match e.axis {
                    Axis::X => (n, t),
                    Axis::Z => (t, n),
                });
            }
        }
    }
    for (cx, cz) in [(h, -h), (-h, -h), (h, h), (-h, h)] {
        for _ in 0..4 {
            let d = g.int(0, si);
            let (ox, oz) = (if cx > 0 { 1 } else { -1 }, if cz > 0 { 1 } else { -1 });
            let (bx, bz) = (
                cx + ox * d - i64::from(ox < 0),
                cz + oz * d - i64::from(oz < 0),
            );
            blocks.push((bx, bz));
            blocks.push((bx + ox, bz));
            blocks.push((bx, bz + oz));
        }
    }
    let rows: Vec<String> = blocks
        .iter()
        .map(|&(x, z)| {
            let f = match cube.fold(x, z) {
                CubeFold::Face { face, i, j } => {
                    format!("\"kind\": \"face\", \"face\": {}, \"i\": {i}, \"j\": {j}", face.index())
                }
                CubeFold::Outside { edge, x, z, iso } => format!(
                    "\"kind\": \"outside\", \"edge\": {edge}, \"px\": {x}, \"pz\": {z}, \"iso\": {}",
                    iso_json(iso)
                ),
                CubeFold::Ocean => "\"kind\": \"ocean\"".into(),
            };
            format!("        {{\"x\": {x}, \"z\": {z}, {f}}}")
        })
        .collect();
    out.push_str(&rows.join(",\n"));
    // Points: random over the window and round it, on the face lines, and on the edges' lines.
    out.push_str("\n      ],\n      \"points\": [\n");
    let hf = h as f64;
    let mut pts: Vec<(f64, f64)> = (0..160)
        .map(|_| (-5.0 * hf + g.f() * 14.0 * hf, -5.0 * hf + g.f() * 10.0 * hf))
        .collect();
    for k in [-3.0, -1.0, 1.0, 3.0, 5.0] {
        pts.push((k * hf, (g.sym()) * hf));
        pts.push((g.sym() * hf, k.min(3.0) * hf));
    }
    for _ in 0..8 {
        pts.push((g.sym() * hf + 17.0 * hf, g.sym() * hf));
    }
    let rows: Vec<String> = pts
        .iter()
        .map(|&(x, z)| {
            let f = match cube.fold_point(x, z) {
                Some(p) => format!(
                    "\"face\": {}, \"u\": {}, \"v\": {}, \"px\": {}, \"pz\": {}, \"iso\": {}, \"edge\": {}",
                    p.face.index(),
                    hx(p.u),
                    hx(p.v),
                    hx(p.x),
                    hx(p.z),
                    iso_json(p.iso),
                    p.edge.map_or("null".into(), |e| e.to_string())
                ),
                None => "\"face\": null".into(),
            };
            format!("        {{\"x\": {}, \"z\": {}, {f}}}", hx(x), hx(z))
        })
        .collect();
    out.push_str(&rows.join(",\n"));
    out.push_str("\n      ]},\n");
    out
}

fn map_section(g: &mut Rng, map: &CubeMap) -> String {
    let pts = face_points(g);
    let rows: Vec<String> = pts
        .iter()
        .map(|&(face, u, v)| {
            let p = map.forward(face, u, v);
            let j = map.jacobian(face, u, v);
            assert_eq!(j.p, p);
            format!(
                "          {{\"face\": {}, \"u\": {}, \"v\": {}, \"p\": {}, \"du\": {}, \"dv\": {}}}",
                face.index(),
                hx(u),
                hx(v),
                hx3(p),
                hx3(j.du),
                hx3(j.dv)
            )
        })
        .collect();
    let mut dirs: Vec<[f64; 3]> = pts
        .iter()
        .map(|&(face, u, v)| map.forward(face, u, v))
        .collect();
    dirs.extend(special_directions());
    for _ in 0..40 {
        dirs.push([g.sym(), g.sym(), g.sym()]);
    }
    let inv: Vec<String> = dirs
        .iter()
        .map(|&p| {
            let (face, u, v) = map.inverse(p);
            format!(
                "          {{\"p\": {}, \"face\": {}, \"u\": {}, \"v\": {}}}",
                hx3(p),
                face.index(),
                hx(u),
                hx(v)
            )
        })
        .collect();
    format!(
        "      {{\"name\": \"{}\", \"sha256\": \"{}\", \"points\": [\n{}\n        ], \"inverse\": [\n{}\n        ]}}",
        map.name(),
        map.sha256_hex(),
        rows.join(",\n"),
        inv.join(",\n")
    )
}

fn world_section(g: &mut Rng, w: &CubeWorld, m_per_block_h: f64) -> String {
    let s = w.cube().side();
    let h = s as f64 / 2.0;
    let rows = w.turn().rows();
    let mut pts: Vec<(f64, f64)> = (0..80)
        .map(|_| (-3.0 * h + g.f() * 8.0 * h, -3.0 * h + g.f() * 6.0 * h))
        .collect();
    // Beyond the window's ends (the slide) and beyond the net's edges, up to and past a face
    // width (past the corners: no point).
    for _ in 0..10 {
        let k = [-2.0, -1.0, 1.0, 3.0][g.int(0, 4) as usize];
        pts.push((-3.0 * h + g.f() * 8.0 * h + k * 8.0 * h, g.sym() * h));
    }
    for _ in 0..30 {
        pts.push((-5.0 * h + g.f() * 14.0 * h, -5.0 * h + g.f() * 10.0 * h));
    }
    // On the face lines.
    for k in [-3.0, -1.0, 1.0, 3.0, 5.0] {
        pts.push((k * h, g.sym() * h));
    }
    for k in [-3.0, -1.0, 1.0, 3.0] {
        pts.push((g.sym() * h, k * h));
    }
    // Round every corner image: on it and within a block of it.
    for c in w.cube().corners() {
        for im in &c.images {
            let (x, z) = (im[0] as f64, im[1] as f64);
            pts.push((x, z));
            for (dx, dz) in [(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
                pts.push((x + dx * g.f(), z + dz * g.f()));
            }
        }
    }
    let prow: Vec<String> = pts
        .iter()
        .map(|&(x, z)| {
            let head = format!("\"x\": {}, \"z\": {}", hx(x), hx(z));
            match (w.unit(x, z), w.lat_lon(x, z), w.jacobian(x, z)) {
                (Some(v), Some((lat, lon)), Some(j)) => format!(
                    "          {{{head}, \"v\": {}, \"lat\": {}, \"lon\": {}, \"m\": [{}, {}, {}, {}], \"sigma\": {}, \"angle\": {}}}",
                    hx3(v),
                    hx(lat),
                    hx(lon),
                    hx(j.m[0][0]),
                    hx(j.m[0][1]),
                    hx(j.m[1][0]),
                    hx(j.m[1][1]),
                    hx(j.sigma),
                    hx(j.angle)
                ),
                _ => format!("          {{{head}, \"v\": null}}"),
            }
        })
        .collect();
    let mut ll: Vec<(f64, f64)> = (0..60)
        .map(|_| (pm::to_degrees(pm::asin(g.sym())), g.f() * 360.0))
        .collect();
    ll.extend([
        (90.0, 0.0),
        (-90.0, 0.0),
        (0.0, 0.0),
        (0.0, 180.0),
        (45.0, -90.0),
    ]);
    for c in w.cube().corners() {
        ll.push(w.corner_lat_lon(c.cube));
    }
    let irow: Vec<String> = ll
        .iter()
        .map(|&(lat, lon)| {
            let (x, z) = w.point_of_lat_lon(lat, lon);
            format!(
                "          {{\"lat\": {}, \"lon\": {}, \"x\": {}, \"z\": {}}}",
                hx(lat),
                hx(lon),
                hx(x),
                hx(z)
            )
        })
        .collect();
    let pole = |(f, u, v): (Face, f64, f64)| {
        let (x, z) = w.cube().point_of_local(f, u, v);
        format!(
            "{{\"face\": {}, \"u\": {}, \"v\": {}, \"x\": {}, \"z\": {}}}",
            f.index(),
            hx(u),
            hx(v),
            hx(x),
            hx(z)
        )
    };
    let crow: Vec<String> = w
        .cube()
        .corners()
        .iter()
        .map(|c| {
            let (lat, lon) = w.corner_lat_lon(c.cube);
            format!(
                "          {{\"name\": \"{}\", \"unit\": {}, \"lat\": {}, \"lon\": {}}}",
                c.name,
                hx3(w.corner_unit(c.cube)),
                hx(lat),
                hx(lon)
            )
        })
        .collect();
    format!(
        "      {{\"s\": {s}, \"map\": \"{}\", \"rotation\": [{}], \"m_per_block_h\": {}, \"radius_m\": {},\n        \"north_pole\": {}, \"south_pole\": {},\n        \"corners\": [\n{}\n        ],\n        \"points\": [\n{}\n        ],\n        \"inverse\": [\n{}\n        ]}}",
        w.map().name(),
        rows.iter()
            .flatten()
            .map(|&x| hx(x))
            .collect::<Vec<_>>()
            .join(", "),
        hx(m_per_block_h),
        hx(w.radius_m()),
        pole(w.north_pole()),
        pole(w.south_pole()),
        crow.join(",\n"),
        prow.join(",\n"),
        irow.join(",\n")
    )
}

/// The fixture's `"cube"` member: `"cube": {...}` with no trailing comma or newline.
pub(crate) fn section() -> String {
    let mut g = Rng(0xC0BE_2026);
    let mut out = String::from("  \"cube\": {\n");
    out.push_str(&face_section(&mut g));
    out.push_str(&net_section(&mut g));
    out.push_str("    \"maps\": [\n");
    let maps = [CubeMap::world(), CubeMap::simulation()];
    let sections: Vec<String> = maps.iter().map(|m| map_section(&mut g, m)).collect();
    out.push_str(&sections.join(",\n"));
    out.push_str("\n    ],\n    \"worlds\": [\n");
    // The readers take the rows from the fixture, never recompute them.
    let free = CubeTurn::from_angles(-71.5, 38.0, 12.0);
    let worlds = [
        (
            CubeWorld::new(Cube::new(7424), CubeMap::world(), free, 300.0),
            300.0,
        ),
        (
            CubeWorld::new(
                Cube::new(1024),
                CubeMap::simulation(),
                CubeTurn::IDENTITY,
                1200.0,
            ),
            1200.0,
        ),
    ];
    let sections: Vec<String> = worlds
        .iter()
        .map(|(w, mpb)| world_section(&mut g, w, *mpb))
        .collect();
    out.push_str(&sections.join(",\n"));
    out.push_str("\n    ]\n  }");
    out
}

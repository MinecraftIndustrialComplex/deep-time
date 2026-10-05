// SPDX-License-Identifier: LGPL-3.0-or-later
//! Vertical cross-sections of a world's strata (P1-07's review picture): along a meridian (a line
//! of constant block x), each pixel column is the horizon stack at that block as the Java strata
//! pass builds it (`LoadedLayer::stack_at`, the rule of `StrataPass.column`), and bodies with a
//! rock class (plutons, their aureoles; deposit bodies later) paint over the horizons where their
//! SDF holds, the first body in id order winning as in `StrataPass.paintBodies`. Air is dark, the
//! sea (up to `sea_level_y`) blue.

use std::fs;
use std::io;
use std::path::Path;

use deeptime_format::{Body, LoadedLayer, Ramp};
use serde_json::{Value, json};

/// One section to draw.
#[derive(Clone, Debug)]
pub struct Section {
    pub title: String,
    /// The meridian, block x.
    pub x: i64,
    /// Block z at the left and right edges (north to south when z0 < z1).
    pub z0: i64,
    pub z1: i64,
    /// Block y at the bottom and top.
    pub y0: i32,
    pub y1: i32,
    /// The world's floor (the last horizon reaches down to it).
    pub min_y: i32,
    /// Pixels.
    pub w: u32,
    pub h: u32,
    /// Draw each strata cell's own stack (no interpolation between cells) instead of the Java
    /// strata pass's column.
    pub per_cell: bool,
}

/// The stack of the strata cell nearest block (x, z), without interpolation: (top block, class,
/// key top).
fn cell_stack(strata: &LoadedLayer, x: i64, z: i64, out: &mut Vec<(i32, u8, u16)>) {
    out.clear();
    let d = &strata.desc;
    let g = &d.grid;
    let c = i64::from(g.w) * i64::from(g.r);
    let wx = (x + c / 2).rem_euclid(c) - c / 2;
    let i = ((wx - g.origin_block_x) as f64 / f64::from(g.r) + 0.5).floor() as i64;
    let j = ((z - g.origin_block_z) as f64 / f64::from(g.r) + 0.5).floor() as i64;
    for h in 0..d.stack_depth.unwrap_or(0) {
        let (ci, cj) = g.cell(i, j);
        let s = strata.horizon(ci, cj, h);
        if !s.is_used() {
            break;
        }
        let top = (f64::from(s.top) * d.scale + d.offset).floor() as i32;
        out.push((top, s.class, s.key[0]));
    }
}

const AIR: [u8; 4] = [18, 20, 26, 255];
const SEA: [u8; 4] = [46, 104, 170, 255];
const SEA_LINE: [u8; 4] = [150, 200, 255, 255];

/// Whether (d, v) lies in a zone's ranges (`bodies::Zone`'s rule: an upper bound of 1 is
/// inclusive).
fn within(x: f64, r: [f32; 2]) -> bool {
    let (lo, hi) = (f64::from(r[0]), f64::from(r[1]));
    x >= lo && (x < hi || (hi >= 1.0 && x <= hi))
}

/// The class a body paints at (x, y, z), if any.
pub fn body_class(b: &Body, c: u32, x: i32, y: i32, z: i32) -> Option<&str> {
    let at = b.locate(c, x, y, z)?;
    let mut class = b.rock_class.as_deref();
    for zone in &b.zones {
        if zone.rock_class.is_some() && within(at.d, zone.d) && within(at.v, zone.v) {
            class = zone.rock_class.as_deref();
        }
    }
    class
}

/// Renders `s` from the horizon stacks `strata` (classes named by `classes`), read through the
/// world's folds and faults when `structure` is given, and `bodies`; RGBA, `s.w` × `s.h`.
#[allow(clippy::too_many_arguments)]
pub fn render(
    strata: &LoadedLayer,
    structure: Option<&deeptime_format::structure::Deformer>,
    bodies: &[Body],
    classes: &[String],
    circumference: u32,
    sea_level_y: i32,
    s: &Section,
    ramp: &Ramp,
) -> Vec<u8> {
    let c = i64::from(circumference);
    let on_plane: Vec<&Body> = bodies
        .iter()
        .filter(|b| b.rock_class.is_some() || b.zones.iter().any(|z| z.rock_class.is_some()))
        .filter(|b| {
            let [lo, hi] = b.aabb;
            [-c, 0, c]
                .iter()
                .any(|&k| s.x + k >= i64::from(lo[0]) && s.x + k <= i64::from(hi[0]))
        })
        .collect();
    let index = |name: &str| classes.iter().position(|c| c == name);
    let mut rgba = vec![0u8; (s.w * s.h * 4) as usize];
    let mut stack = Vec::new();
    let mut deform = deeptime_format::structure::DeformScratch::default();
    for px in 0..s.w {
        let z = s.z0 as f64 + (s.z1 - s.z0) as f64 * (f64::from(px) + 0.5) / f64::from(s.w);
        let z = z.floor() as i64;
        if s.per_cell {
            cell_stack(strata, s.x, z, &mut stack);
        } else if let Some(d) = structure {
            d.stack(
                s.x,
                z,
                s.min_y,
                &mut |sx, sz, o| strata.stack_at(sx, sz, s.min_y, o),
                &mut deform,
                &mut stack,
            );
        } else {
            strata.stack_at(s.x, z, s.min_y, &mut stack);
        }
        let near: Vec<&&Body> = on_plane
            .iter()
            .filter(|b| i64::from(b.aabb[0][2]) <= z && z <= i64::from(b.aabb[1][2]))
            .collect();
        for py in 0..s.h {
            let yf =
                f64::from(s.y1) - f64::from(s.y1 - s.y0) * (f64::from(py) + 0.5) / f64::from(s.h);
            let y = yf.floor() as i32;
            let top = stack.first().map_or(i32::MIN, |t| t.0);
            let colour = if y > top {
                if y == sea_level_y {
                    SEA_LINE
                } else if y < sea_level_y {
                    SEA
                } else {
                    AIR
                }
            } else {
                let mut h = 0;
                while h + 1 < stack.len() && y <= stack[h + 1].0 {
                    h += 1;
                }
                let mut class = f64::from(stack[h].1);
                for b in &near {
                    if let Some(name) = body_class(b, circumference, s.x as i32, y, z as i32)
                        && let Some(k) = index(name)
                    {
                        class = k as f64;
                        break;
                    }
                }
                ramp.rgba(class)
            };
            let at = ((py * s.w + px) * 4) as usize;
            rgba[at..at + 4].copy_from_slice(&colour);
        }
    }
    rgba
}

/// Writes each section as `sections/<n>.png` under `dir`; returns their descriptions for the
/// report (`window.DT_REPORT.structure.sections`).
#[allow(clippy::too_many_arguments)]
pub fn write_sections(
    dir: &Path,
    strata: &LoadedLayer,
    structure: Option<&deeptime_format::structure::Deformer>,
    bodies: &[Body],
    classes: &[String],
    circumference: u32,
    sea_level_y: i32,
    sections: &[Section],
    ramp: &Ramp,
    m_per_block: (f64, f64),
) -> io::Result<Value> {
    fs::create_dir_all(dir.join("sections"))?;
    let mut out = Vec::new();
    for (n, s) in sections.iter().enumerate() {
        let rgba = render(
            strata,
            structure,
            bodies,
            classes,
            circumference,
            sea_level_y,
            s,
            ramp,
        );
        let file = format!("sections/section-{n}.png");
        crate::write_rgba_png(&dir.join(&file), s.w, s.h, &rgba)?;
        let c = f64::from(circumference);
        let proj = deeptime_format::Projection::of(c);
        let lat = |z: i64| proj.lat_deg(z as f64);
        let lon = proj.lon_deg(s.x as f64);
        let m_per_px_h = (s.z1 - s.z0).abs() as f64 * m_per_block.0 / f64::from(s.w);
        let m_per_px_v = f64::from(s.y1 - s.y0) * m_per_block.1 / f64::from(s.h);
        out.push(json!({
            "title": s.title, "file": file, "w": s.w, "h": s.h, "per_cell": s.per_cell,
            "x": s.x, "z0": s.z0, "z1": s.z1, "y0": s.y0, "y1": s.y1,
            "lon": lon, "lat0": lat(s.z0), "lat1": lat(s.z1),
            "km": (s.z1 - s.z0).abs() as f64 * m_per_block.0 / 1000.0,
            "vertical_exaggeration": m_per_px_h / m_per_px_v,
        }));
    }
    Ok(Value::Array(out))
}

/// The legend: every class index in `used` with its colour.
pub fn legend(classes: &[String], used: &[bool], ramp: &Ramp) -> Value {
    Value::Array(
        classes
            .iter()
            .enumerate()
            .filter(|(k, _)| used.get(*k).copied().unwrap_or(false))
            .map(|(k, name)| {
                let [r, g, b, _] = ramp.rgba(k as f64);
                json!({ "index": k, "class": name, "colour": format!("#{r:02x}{g:02x}{b:02x}") })
            })
            .collect(),
    )
}

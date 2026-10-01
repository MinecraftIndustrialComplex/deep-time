// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime structure-sections`: cross-sections of a baked world through its folds and faults
//! (P2-01's review picture), each drawn twice: the layers as they were laid down, and as the
//! world's structures left them (the column rule the game runs, `ColumnWorld::stack`).
//!
//! Where to cut is read from the world's own event log: the largest `deeptime:fold_belt` a
//! collision made, the largest one a subduction zone made (an arc), and the largest rift's
//! `deeptime:fault_system`, each cut across its strongest structure (`strongest_x`,
//! `strongest_z`, `strongest_azimuth_deg`). `--line "title:x0,z0,x1,z1"` adds a line of your own.
//! Writes `<out>/<n>-plain.png`, `<n>-structure.png`, `sections.json` and `index.html`.

use std::path::Path;

use deeptime_format::column::ColumnWorld;
use deeptime_format::events::{EventLog, ParamValue, Place};
use deeptime_format::structure::DeformScratch;
use deeptime_format::{Event, Ramps};
use deeptime_viewer::section::body_class;
use serde_json::{Value, json};

use crate::run::CliResult;

const AIR: [u8; 4] = [18, 20, 26, 255];
const SEA: [u8; 4] = [46, 104, 170, 255];

/// One line to cut along.
struct Line {
    title: String,
    a: (f64, f64),
    b: (f64, f64),
    about: Value,
}

fn num(e: &Event, key: &str) -> Option<f64> {
    match e.params.get(key)? {
        ParamValue::Float(v) => Some(*v),
        ParamValue::Int(v) => Some(*v as f64),
        _ => None,
    }
}

fn text<'a>(e: &'a Event, key: &str) -> Option<&'a str> {
    match e.params.get(key)? {
        ParamValue::Str(s) => Some(s),
        _ => None,
    }
}

/// The line across group event `e`'s strongest structure, `half` blocks either side.
fn across(e: &Event, title: &str, half: f64, cause: Option<&Event>) -> Option<Line> {
    let (x, z) = (num(e, "strongest_x")?, num(e, "strongest_z")?);
    let az = (num(e, "strongest_azimuth_deg")? + 90.0).to_radians();
    let (dx, dz) = (az.sin(), -az.cos());
    let place = match e.place {
        Place::Point { lat, lon } => json!({"lat": lat, "lon": lon}),
        _ => Value::Null,
    };
    Some(Line {
        title: title.to_owned(),
        a: (x - half * dx, z - half * dz),
        b: (x + half * dx, z + half * dz),
        about: json!({
            "event": e.id.0,
            "kind": e.kind,
            "from_myr": e.t_start / 1.0e6,
            "to_myr": e.t_end / 1.0e6,
            "centroid": place,
            "cause": cause.map(|c| c.kind.clone()),
            "params": e.params.iter().filter_map(|(k, v)| match v {
                ParamValue::Float(f) => Some((k.clone(), json!(f))),
                ParamValue::Int(i) => Some((k.clone(), json!(i))),
                ParamValue::Str(s) => Some((k.clone(), json!(s))),
                ParamValue::Id(_) => None,
            }).collect::<serde_json::Map<_, _>>(),
        }),
    })
}

/// The lines the event log suggests: a collision belt, an arc and a rift.
fn suggested(log: &EventLog, half: f64) -> Vec<Line> {
    let size = |e: &Event| {
        ["fold_trains", "thrusts", "faults"]
            .iter()
            .filter_map(|k| num(e, k))
            .sum::<f64>()
    };
    let cause = |e: &Event| e.causes.first().and_then(|&c| log.get(c));
    let best = |pick: &dyn Fn(&Event) -> bool| {
        log.events()
            .iter()
            .filter(|e| pick(e) && num(e, "strongest_x").is_some())
            .max_by(|a, b| size(a).total_cmp(&size(b)).then(a.id.0.cmp(&b.id.0)))
    };
    let belt = |kinds: &'static [&'static str]| {
        move |e: &Event| {
            e.kind == "deeptime:fold_belt"
                && e.causes
                    .first()
                    .and_then(|&c| log.get(c))
                    .is_some_and(|c| kinds.contains(&c.kind.as_str()))
        }
    };
    let mut out = Vec::new();
    type Pick<'a> = Box<dyn Fn(&Event) -> bool + 'a>;
    let wanted: [(&str, Pick<'_>); 3] = [
        (
            "Across a collision belt",
            Box::new(belt(&[
                "deeptime:collision",
                "deeptime:orogeny_phase",
                "deeptime:terrane_accretion",
            ])),
        ),
        (
            "Across an arc (the fold belt of a subduction zone)",
            Box::new(belt(&["deeptime:subduction_onset"])),
        ),
        (
            "Across a rift",
            Box::new(|e: &Event| {
                e.kind == "deeptime:fault_system" && text(e, "setting") == Some("rift")
            }),
        ),
    ];
    for (title, pick) in &wanted {
        if let Some(e) = best(pick.as_ref())
            && let Some(l) = across(e, title, half, cause(e))
        {
            out.push(l);
        }
    }
    out
}

/// Draws the section along `line`: one pixel column per block, `px` pixels a block both ways.
/// Returns (width, height, y at the bottom, y at the top, RGBA).
fn draw(
    w: &ColumnWorld,
    line: &Line,
    deformed: bool,
    px: u32,
    ramp: &deeptime_format::Ramp,
) -> (u32, u32, i32, i32, Vec<u8>) {
    let n = (line.b.0 - line.a.0)
        .hypot(line.b.1 - line.a.1)
        .round()
        .max(2.0) as usize;
    let at = |k: usize| {
        let t = (k as f64 + 0.5) / n as f64;
        (
            (line.a.0 + (line.b.0 - line.a.0) * t).floor() as i64,
            (line.a.1 + (line.b.1 - line.a.1) * t).floor() as i64,
        )
    };
    let sea = w.manifest.scale.sea_level_y;
    let grounds: Vec<i32> = (0..n)
        .map(|k| {
            let (x, z) = at(k);
            w.top_y(x, z)
        })
        .collect();
    let hi = grounds.iter().copied().max().unwrap_or(sea).max(sea) + 12;
    let lo = (grounds.iter().copied().min().unwrap_or(sea).min(sea) - 220).max(w.min_y);
    let (y0, y1) = (lo, hi.min(w.max_y));
    let (width, height) = (n as u32 * px, (y1 - y0 + 1) as u32 * px);
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    let index = |name: &str| w.classes.iter().position(|c| c == name);
    let c = w.manifest.scale.circumference_blocks;
    let mut scratch = DeformScratch::default();
    let mut stack = Vec::new();
    for (k, &ground) in grounds.iter().enumerate() {
        let (x, z) = at(k);
        if deformed {
            w.stack(x, z, &mut scratch, &mut stack);
        } else {
            w.plain_stack(x, z, &mut stack);
        }
        let wx = w.wrap_x(x);
        let near: Vec<&deeptime_format::Body> = w
            .bodies
            .iter()
            .filter(|b| b.rock_class.is_some() || b.zones.iter().any(|q| q.rock_class.is_some()))
            .filter(|b| {
                let [l, h] = b.aabb;
                i64::from(l[2]) <= z
                    && z <= i64::from(h[2])
                    && [-i64::from(c), 0, i64::from(c)]
                        .iter()
                        .any(|&q| wx + q >= i64::from(l[0]) && wx + q <= i64::from(h[0]))
            })
            .collect();
        for y in y0..=y1 {
            let colour = if y > ground || stack.is_empty() {
                if y <= sea { SEA } else { AIR }
            } else {
                let mut h = 0;
                while h + 1 < stack.len() && y <= stack[h + 1].0 {
                    h += 1;
                }
                let mut class = f64::from(stack[h].1);
                for b in &near {
                    if let Some(name) = body_class(b, c, wx as i32, y, z as i32)
                        && let Some(i) = index(name)
                    {
                        class = i as f64;
                        break;
                    }
                }
                ramp.rgba(class)
            };
            let row = (y1 - y) as u32 * px;
            for dy in 0..px {
                for dx in 0..px {
                    let i = (((row + dy) * width + k as u32 * px + dx) * 4) as usize;
                    rgba[i..i + 4].copy_from_slice(&colour);
                }
            }
        }
    }
    (width, height, y0, y1, rgba)
}

fn parse_line(s: &str) -> Result<Line, String> {
    let bad = || format!("--line {s}: expected TITLE:x0,z0,x1,z1");
    let (title, rest) = s.rsplit_once(':').ok_or_else(bad)?;
    let v: Vec<f64> = rest
        .split(',')
        .map(|t| t.trim().parse::<f64>().map_err(|_| bad()))
        .collect::<Result<_, _>>()?;
    let [x0, z0, x1, z1] = v[..] else {
        return Err(bad());
    };
    Ok(Line {
        title: title.to_owned(),
        a: (x0, z0),
        b: (x1, z1),
        about: Value::Null,
    })
}

/// Writes the sections of `world` into `out`; returns `sections.json`'s value.
pub fn run(world: &Path, out: &Path, lines: &[String], half: f64, px: u32) -> CliResult<Value> {
    let w = ColumnWorld::open(world)?;
    if w.strata().is_none() {
        return Err(format!("{}: no strata/horizons layer", world.display()).into());
    }
    let log = EventLog::load(world)?;
    let mut all = suggested(&log, half);
    for l in lines {
        all.push(parse_line(l)?);
    }
    if all.is_empty() {
        return Err("no fold belt or fault system in the event log, and no --line given".into());
    }
    std::fs::create_dir_all(out)?;
    let ramps = Ramps::builtin();
    let (_, ramp) = ramps.for_layer("structure/rock");
    let s = &w.manifest.scale;
    let c = f64::from(s.circumference_blocks);
    let mut used = vec![false; w.classes.len()];
    let mut list = Vec::new();
    let mut html = String::from(
        "<!doctype html><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <meta name=\"description\" content=\"Cross-sections of a Deep Time planet through its folds and faults, before and after.\">\
         <title>Deep Time: sections through folds and faults</title>\
         <style>body{font:15px/1.5 system-ui,sans-serif;margin:16px;background:#14161b;color:#e6e6e6}\
         img{max-width:100%;image-rendering:pixelated;display:block;margin:4px 0 14px}h2{margin:28px 0 4px}\
         p{max-width:70em;color:#b9bec8}.k{display:inline-block;margin:2px 10px 2px 0}\
         .s{display:inline-block;width:12px;height:12px;vertical-align:-1px;margin-right:4px}</style>\
         <h1>Sections through folds and faults</h1>\
         <p>Each section is drawn twice in block space (one block is one square, so this is what a cliff or a mine shaft shows): \
         first the layers as they were laid down, then as the world's folds, faults and intrusions left them. \
         Rock younger than a structure lies undisturbed across it.</p>",
    );
    for (n, line) in all.iter().enumerate() {
        let mut files = Vec::new();
        let mut range = (0, 0);
        let mut size = (0, 0);
        for (deformed, tag) in [(false, "plain"), (true, "structure")] {
            let (width, height, y0, y1, rgba) = draw(&w, line, deformed, px, ramp);
            let file = format!("{n}-{tag}.png");
            std::fs::write(
                out.join(&file),
                deeptime_viewer::encode_png(width, height, &rgba)?,
            )?;
            files.push(file);
            range = (y0, y1);
            size = (width, height);
        }
        // The classes on the deformed section, for the legend.
        let mut scratch = DeformScratch::default();
        let mut stack = Vec::new();
        let blocks = (line.b.0 - line.a.0).hypot(line.b.1 - line.a.1);
        let steps = blocks.round().max(2.0) as usize;
        let mut changed = 0usize;
        let mut plain = Vec::new();
        for k in 0..steps {
            let t = (k as f64 + 0.5) / steps as f64;
            let x = (line.a.0 + (line.b.0 - line.a.0) * t).floor() as i64;
            let z = (line.a.1 + (line.b.1 - line.a.1) * t).floor() as i64;
            w.stack(x, z, &mut scratch, &mut stack);
            w.plain_stack(x, z, &mut plain);
            changed += usize::from(stack != plain);
            for h in &stack {
                if h.0 >= range.0
                    && let Some(u) = used.get_mut(usize::from(h.1))
                {
                    *u = true;
                }
            }
        }
        let lat = |z: f64| -360.0 * z / c;
        let lon = |x: f64| (360.0 * (x + c / 2.0).rem_euclid(c) / c).rem_euclid(360.0);
        let km = blocks * s.m_per_block_h / 1000.0;
        html.push_str(&format!(
            "<h2>{}</h2><p>From block ({:.0}, {:.0}) to ({:.0}, {:.0}): {:.0} blocks ({:.0} km), y {} to {}. \
             The structures change {} of its {} columns.</p>\
             <p>As laid down:</p><img src=\"{}\" alt=\"{} before\"><p>With the world's structures:</p><img src=\"{}\" alt=\"{} after\">",
            line.title, line.a.0, line.a.1, line.b.0, line.b.1, blocks, km, range.0, range.1,
            changed, steps, files[0], line.title, files[1], line.title
        ));
        list.push(json!({
            "title": line.title,
            "from": [line.a.0, line.a.1], "to": [line.b.0, line.b.1],
            "from_lat_lon": [lat(line.a.1), lon(line.a.0)], "to_lat_lon": [lat(line.b.1), lon(line.b.0)],
            "blocks": blocks, "km": km, "y0": range.0, "y1": range.1,
            "w": size.0, "h": size.1, "px_per_block": px,
            "columns_changed": changed, "columns": steps,
            "plain": files[0], "structure": files[1],
            "about": line.about,
        }));
    }
    let legend = deeptime_viewer::section::legend(&w.classes, &used, ramp);
    html.push_str("<h2>Rocks</h2><p>");
    for l in legend.as_array().into_iter().flatten() {
        html.push_str(&format!(
            "<span class=\"k\"><span class=\"s\" style=\"background:{}\"></span>{}</span>",
            l["colour"].as_str().unwrap_or("#000"),
            l["class"].as_str().unwrap_or("?")
        ));
    }
    html.push_str("</p>");
    let doc = json!({
        "world_data": world.display().to_string(),
        "events": w.structure().map_or(0, |d| d.structure.events.len()),
        "sections": list,
        "legend": legend,
    });
    std::fs::write(out.join("index.html"), html)?;
    std::fs::write(
        out.join("sections.json"),
        serde_json::to_vec_pretty(&doc).expect("sections serialise"),
    )?;
    Ok(doc)
}

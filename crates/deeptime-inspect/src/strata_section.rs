// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime-inspect wd-section`: a vertical cross-section of what the world data itself decides,
//! with no Minecraft world: each pixel column is Deep Time's column evaluator
//! (`deeptime_format::ColumnWorld::sample`, the twin of Java's `worlddata/Columns`) — the terrain
//! top, the horizon stack by the column rule and the bodies over it — coloured by rock class in a
//! geological palette, with a legend. It shows the strata exactly as the strata pass would paint
//! them before caves, carvers, surface rules and features, so layer geometry can be reviewed in
//! seconds from a bake (the in-game `section` shows the same planes through a generated world).

use std::collections::BTreeMap;
use std::path::Path;

use deeptime_format::{ColumnWorld, Ramps};
use serde_json::{Value, json};

use crate::error::{Error, Result};
use crate::image::{Image, Rgba, hex, shade, text_width};
use crate::render::{Axis, Out, SectionArgs};

const SKY: Rgba = [0xdf, 0xe9, 0xf3, 255];
const SEA: Rgba = [0x4a, 0x7f, 0xbf, 255];
const CRUST: Rgba = [0x80, 0x80, 0x80, 255];
const BG: Rgba = [0xfa, 0xf9, 0xf6, 255];
const INK: Rgba = [0x22, 0x22, 0x22, 255];

/// A geological colour for the common classes (loosely the conventions of geological maps:
/// sandstones yellow, shales grey-green, carbonates blue, granitoids pink-red, basic rocks green,
/// metamorphics violet, loose sediment pale); the rest come from the class ramp.
fn class_colour(name: &str, index: usize, ramps: &Ramps) -> Rgba {
    let c: Option<[u8; 3]> = match name {
        "sandstone" => Some([0xe6, 0xc8, 0x5a]),
        "red_bed" => Some([0xc0, 0x5a, 0x3c]),
        "conglomerate" => Some([0xd8, 0x9a, 0x3a]),
        // Diamictite (lithified till): plum, as in the viewer's rock_class ramp, apart from the tans.
        "diamictite" => Some([0x80, 0x40, 0x80]),
        // The impact rocks (P2-08): breccia a dusty rose, melt a dark teal, as in the viewer's ramp.
        "impact_breccia" => Some([0xb0, 0x70, 0x78]),
        "impact_melt" => Some([0x2e, 0x5a, 0x5e]),
        "mudstone" => Some([0x9a, 0x8c, 0x74]),
        "shale" => Some([0x6e, 0x78, 0x6a]),
        "black_shale" => Some([0x3a, 0x3e, 0x3c]),
        "limestone" => Some([0x6c, 0xa6, 0xd8]),
        "dolostone" => Some([0x8a, 0x9a, 0xd8]),
        "chalk" => Some([0xee, 0xf2, 0xf4]),
        "chert" => Some([0xa8, 0x6a, 0x6a]),
        "halite" | "gypsum" | "anhydrite" => Some([0xf0, 0xa0, 0xd8]),
        "coal" => Some([0x10, 0x10, 0x10]),
        "ironstone" | "banded_iron_formation" => Some([0x8a, 0x2a, 0x1a]),
        "granite" => Some([0xe8, 0x86, 0x9a]),
        "granodiorite" => Some([0xd8, 0x70, 0x70]),
        "diorite" => Some([0xb8, 0x9a, 0xa0]),
        "gabbro" => Some([0x2a, 0x6a, 0x4a]),
        "peridotite" => Some([0x5a, 0x8a, 0x2a]),
        "rhyolite" => Some([0xf0, 0xb0, 0x80]),
        "andesite" => Some([0xa0, 0x70, 0x50]),
        "basalt" => Some([0x3a, 0x4a, 0x3a]),
        "gneiss" => Some([0xa0, 0x70, 0xb8]),
        "schist" => Some([0x80, 0x58, 0xa8]),
        "slate" | "phyllite" => Some([0x5a, 0x5a, 0x80]),
        "quartzite" => Some([0xf0, 0xe0, 0xc0]),
        "marble" => Some([0xd8, 0xe8, 0xf8]),
        "amphibolite" | "greenstone" => Some([0x40, 0x7a, 0x60]),
        "hornfels" | "skarn" => Some([0x6a, 0x40, 0x40]),
        "soil" => Some([0x6a, 0x4a, 0x2a]),
        "saprolite" | "laterite" | "bauxite" => Some([0xb0, 0x60, 0x30]),
        "sand" => Some([0xf4, 0xe4, 0xa8]),
        "gravel" | "till" => Some([0xb8, 0xb0, 0xa0]),
        "mud" | "clay" => Some([0xb8, 0xa8, 0x90]),
        "loess" => Some([0xe0, 0xc8, 0x98]),
        _ => None,
    };
    match c {
        Some([r, g, b]) => [r, g, b, 255],
        None => ramps.for_layer("structure/rock").1.rgba(index as f64),
    }
}

/// Renders the section `args` of the world data at `dir` into `out`; returns its JSON.
pub fn wd_section(dir: &Path, args: &SectionArgs, out: &Out) -> Result<Value> {
    let wd = ColumnWorld::open(dir)?;
    let ramps = Ramps::builtin();
    let (lo, hi) = (args.from.min(args.to), args.from.max(args.to));
    let pos = |t: i32| match args.axis {
        Axis::X => (i64::from(args.at), i64::from(t)),
        Axis::Z => (i64::from(t), i64::from(args.at)),
    };
    let sea = wd.manifest.scale.sea_level_y;
    let cols: Vec<_> = (lo..=hi)
        .map(|t| {
            let (x, z) = pos(t);
            wd.sample(x, z)
        })
        .collect();
    let lowest = cols.iter().map(|c| c.top_y).min().unwrap_or(wd.min_y);
    let highest = cols
        .iter()
        .map(|c| c.top_y)
        .max()
        .unwrap_or(wd.max_y)
        .max(sea);
    let y0 = args.y_from.unwrap_or(lowest - 64).max(wd.min_y);
    let y1 = args.y_to.unwrap_or(highest + 12).min(wd.max_y);
    if y1 < y0 {
        return Err(Error(format!("empty y range {y0}..{y1}")));
    }
    let index: BTreeMap<&str, usize> = wd
        .classes
        .iter()
        .enumerate()
        .map(|(i, c)| (c.as_str(), i))
        .collect();
    let colour = |name: &str| class_colour(name, index.get(name).copied().unwrap_or(0), &ramps);
    let (w, h) = (cols.len(), (y1 - y0 + 1) as usize);
    let row = |y: i32| (y1 - y) as usize;
    let mut img = Image::new(w, h, SKY);
    let mut used: BTreeMap<String, usize> = BTreeMap::new();
    for (i, c) in cols.iter().enumerate() {
        for y in y0..=y1 {
            let px = if y > c.top_y {
                if y < sea { SEA } else { SKY }
            } else {
                match c.rock.iter().find(|r| r.0 >= y && y >= r.1) {
                    Some((_, _, Some(name))) => {
                        *used.entry(name.clone()).or_default() += 1;
                        colour(name)
                    }
                    _ => CRUST,
                }
            };
            img.set(i, row(y), px);
        }
        // Bodies (deposits, plutons): a darker rim where the column enters and leaves one.
        for &(_, top, bottom) in &c.bodies {
            for y in [top, bottom] {
                if (y0..=y1).contains(&y) && y <= c.top_y {
                    let p = img.get(i, row(y));
                    img.set(i, row(y), shade(p, 0.6));
                }
            }
        }
    }
    let s = out.scale.max(1);
    let body = img.scaled(s);
    // Margins: y labels left, x labels below, the legend on the right.
    let legend: Vec<(&String, Rgba)> = used.keys().map(|n| (n, colour(n))).collect();
    let lw = legend
        .iter()
        .map(|(n, _)| text_width(n, 1))
        .max()
        .unwrap_or(0)
        + 24;
    let (ml, mb) = (36usize, 18usize);
    let fh = body.h.max(legend.len() * 12 + 8) + mb;
    let mut fin = Image::new(body.w + ml + lw + 8, fh, BG);
    for y in 0..body.h {
        for x in 0..body.w {
            fin.set(x + ml, y, body.get(x, y));
        }
    }
    let first_tick = y0 + (16 - y0.rem_euclid(16)) % 16;
    for y in (first_tick..=y1).step_by(16) {
        let py = row(y) * s;
        fin.fill_rect(ml - 4, py, 4, 1, INK);
        let label = y.to_string();
        let tw = text_width(&label, 1);
        fin.text(
            ml.saturating_sub(6 + tw),
            py.saturating_sub(2),
            &label,
            1,
            INK,
        );
    }
    let first_t = lo + (32 - lo.rem_euclid(32)) % 32;
    for t in (first_t..=hi).step_by(32) {
        let px = ml + (t - lo) as usize * s;
        fin.fill_rect(px, body.h, 1, 4, INK);
        fin.text(px + 2, body.h + 6, &t.to_string(), 1, INK);
    }
    let lx = ml + body.w + 8;
    for (k, (name, c)) in legend.iter().enumerate() {
        let y = 4 + k * 12;
        fin.fill_rect(lx, y, 10, 9, *c);
        fin.text(lx + 14, y + 1, name, 1, INK);
    }
    let axis_name = match args.axis {
        Axis::X => "x",
        Axis::Z => "z",
    };
    let base = format!("wd-section-{axis_name}{}", args.at);
    let p = out.path(&format!("{base}.png"));
    fin.save_png(&p)?;
    let v = json!({
        "command": "wd-section",
        "world_data": dir.display().to_string(),
        "plane": format!("{axis_name} = {}", args.at),
        "from": lo, "to": hi, "y_from": y0, "y_to": y1,
        "definition": "one pixel per block (then scaled) of the world data's column evaluator (deeptime_format::ColumnWorld::sample, Java's worlddata/Columns): the rock class of every block from the terrain's top block down (horizons by the column rule, bodies over them), sky and sea above; a darker pixel where the column enters or leaves a body; grey = unclassified crust",
        "legend": legend.iter().map(|(n, c)| json!({"class": n, "colour": hex(*c), "blocks": used[*n]})).collect::<Vec<_>>(),
        "png": p.display().to_string(),
    });
    out.json(&v, &format!("{base}.json"))?;
    Ok(v)
}

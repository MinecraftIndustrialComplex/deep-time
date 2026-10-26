// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime margins`: the continental margins of a checkpoint's plate model by tectonic setting
//! (P1-27, `plates::stats::margin_classes`) as a map — passive, Andean (the continent overrides a
//! subduction front), subducting (the continent nears a trench), collision, another plate
//! boundary — over the sea floor by age, with the plate boundaries and the overriding side of every
//! live subduction front, and the margin shares printed.

use std::path::Path;

use deeptime_core::checkpoint::Checkpoint;
use deeptime_core::plates::geometry::{Geo, Tree, positions};
use deeptime_core::plates::normalise::Norm;
use deeptime_core::plates::stats::{Margins, margin_class as mc, margin_classes};
use deeptime_core::preset::{Preset, PresetBody};
use deeptime_core::sphere::BlockFrame;
use deeptime_format::GridDesc;

/// Width of the map, pixels (the height is half).
const W: u32 = 1024;

pub fn run(input: &Path, out: &Path) -> Result<Margins, Box<dyn std::error::Error>> {
    let ck = Checkpoint::read(input)?;
    let preset = Preset::from_json(&ck.preset_json)?;
    let PresetBody::Planet(p) = &preset.body else {
        return Err("the checkpoint's preset has no planet".into());
    };
    let st = ck
        .world
        .plates
        .as_ref()
        .ok_or("the checkpoint's world has no plate model")?;
    let geo = Geo::build(st.n as usize, st.lattice)?;
    let pos = positions(st, &geo);
    let norm = Norm::new(
        p.tectonics.model_radius_km,
        p.scale.radius_m() / 1000.0,
        st.n as usize,
    );
    let initiated: std::collections::BTreeSet<u32> = ck
        .world
        .events
        .iter()
        .filter(|e| {
            e.kind == deeptime_core::plates::events::PLATE_BIRTH
                && matches!(e.params.get("origin"),
                    Some(deeptime_format::ParamValue::Str(o)) if o == "subduction_initiation")
        })
        .filter_map(|e| match e.params.get("plate") {
            Some(deeptime_format::ParamValue::Int(v)) => u32::try_from(*v).ok(),
            _ => None,
        })
        .collect();
    let (m, class) = margin_classes(
        st,
        &pos,
        &norm,
        &|q| initiated.contains(&q),
        deeptime_core::plates::forces::incipient_km(&p.tectonics),
        deeptime_core::plates::forces::slab_floor_mm_yr(&p.tectonics),
    );

    let c = p.scale.circumference_blocks;
    let grid = GridDesc::for_planet("map", c, c / W)?;
    let frame = BlockFrame::new(c);
    let (w, h) = (grid.w as usize, grid.h as usize);
    let tree = Tree::over(&pos, |i| !st.samples[i].is_hidden());
    let t = st.t_myr;
    let mut nearest = vec![0u32; w * h];
    for j in 0..h {
        for i in 0..w {
            let q = frame.unit(grid.block_x(i as i64) as f64, grid.block_z(j as i64) as f64);
            nearest[j * w + i] = tree.nearest(q).map_or(0, |(s, _)| s);
        }
    }
    let mut rgba = vec![0u8; w * h * 4];
    for (px, &s) in nearest.iter().enumerate() {
        let sample = &st.samples[s as usize];
        let code = class[s as usize];
        let rgb: [u8; 3] = if code & mc::FRONT != 0 {
            [0, 0, 0]
        } else {
            match code {
                mc::INLAND => [184, 163, 122],
                mc::PASSIVE => [44, 160, 44],
                mc::ANDEAN => [214, 39, 40],
                mc::SUBDUCTING => [255, 127, 14],
                mc::COLLISION => [122, 1, 119],
                mc::BOUNDARY => [255, 215, 0],
                mc::FRAGMENT => [247, 182, 210],
                _ => {
                    // The sea floor: light when young, dark when old (0–200 Myr).
                    let a = (sample.age_myr(t) / 200.0).clamp(0.0, 1.0);
                    let mix = |young: f64, old: f64| (young + (old - young) * a) as u8;
                    [mix(140.0, 10.0), mix(190.0, 40.0), mix(230.0, 80.0)]
                }
            }
        };
        rgba[px * 4..px * 4 + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }
    // Plate boundaries: white where the next pixel east or south lies on another plate.
    for j in 0..h {
        for i in 0..w {
            let a = st.plate_of(nearest[j * w + i] as usize);
            let east = st.plate_of(nearest[j * w + (i + 1) % w] as usize);
            let south = st.plate_of(nearest[(j + 1).min(h - 1) * w + i] as usize);
            if a != east || a != south {
                let px = j * w + i;
                if class[nearest[px] as usize] & mc::FRONT == 0 {
                    rgba[px * 4..px * 4 + 3].copy_from_slice(&[255, 255, 255]);
                }
            }
        }
    }
    std::fs::write(out, deeptime_viewer::encode_png(w as u32, h as u32, &rgba)?)?;
    Ok(m)
}

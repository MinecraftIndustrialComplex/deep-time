// SPDX-License-Identifier: LGPL-3.0-or-later
//! The surface part of the bake, on L2 (plan §5.3): `surface/regolith_blocks` (u8, 1/16 block: the
//! deep-time surface's regolith thickness, bilinear from L1) and `surface/class_top` (u8, the rock
//! class index of `strata/classes.json` at the surface: the young landscape's `young/class`, else
//! the deep-time surface's `surface/class`, nearest). Nothing in the game reads them yet; they are
//! the surface lithology the plan lists for soils and surface rules (P1-09 on).

use deeptime_format::DType;
use serde_json::{Value, json};

use super::{Out, desc, field, grids, resample};
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::surface::{self, SurfaceState, young};
use crate::world::{Field, WorldState};

pub fn bake(
    world: &WorldState,
    p: &PlanetPreset,
    s: &SurfaceState,
    out: &mut Out<'_>,
) -> Result<Value, CoreError> {
    let (_l1, l2) = grids(p)?;
    let reg = resample(
        &Field {
            grid: s.grid.clone(),
            data: s.regolith_m.clone(),
        },
        &l2,
        false,
    );
    let mv = p.scale.m_per_block_v;
    let raw: Vec<u8> = reg
        .iter()
        .map(|&m| {
            if m.is_nan() {
                0
            } else {
                (f64::from(m) / mv * 16.0).round().clamp(0.0, 255.0) as u8
            }
        })
        .collect();
    out.layer(
        desc(
            "surface/regolith_blocks",
            DType::U8,
            &l2,
            1.0 / 16.0,
            "block",
            None,
        ),
        &raw,
    )?;
    let (class, from) = match field(world, young::field::CLASS) {
        Some(f) => (Some(resample(f, &l2, true)), young::field::CLASS),
        None => (
            field(world, surface::field::CLASS).map(|f| resample(f, &l2, true)),
            surface::field::CLASS,
        ),
    };
    let mut written = vec!["surface/regolith_blocks"];
    if let Some(class) = class {
        let raw: Vec<u8> = class
            .iter()
            .map(|&k| {
                if k.is_nan() {
                    255
                } else {
                    (k.round() as i64).clamp(0, 254) as u8
                }
            })
            .collect();
        out.layer(
            desc(
                "surface/class_top",
                DType::U8,
                &l2,
                1.0,
                "class index (strata/classes.json)",
                Some(255.0),
            ),
            &raw,
        )?;
        written.push("surface/class_top");
    }
    Ok(json!({ "layers": written, "class_top_from": from, "grid": l2 }))
}

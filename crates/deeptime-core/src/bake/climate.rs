// SPDX-License-Identifier: LGPL-3.0-or-later
//! The climate part of the bake.
//!
//! **Physical climate on L1** (plan §5.3; the viewer's and the rules'): `climate/t_mean_c` (i16,
//! 0.01 °C), `climate/p_mm` (u16, mm/yr), `climate/koppen` (u8, the index of
//! `climate::koppen::CODES`, 31 = sea), from the climate stage's last snapshot.
//!
//! **The five biome inputs on L2** (`climate/{temperature, vegetation, continents, erosion,
//! ridges}`, i16, vanilla parameter × 10,000): what the noise router's climate functions read
//! (plan §7.1), from the physical climate and the ground by the §7.2 mapping in [`super::biomes`]
//! (P1-09; the preset's `biomes` section holds its knobs). The game maps them once more per world
//! onto the pack's bands and distributions (`ClimateMappingBuilder`, `climate_mapping.json`).

use deeptime_format::DType;
use serde_json::{Value, json};

use super::biomes::{self, BiomeParams, Climate, Ground};
use super::{Out, desc, field, grids, resample};
use crate::climate;
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::surface::SurfaceState;
use crate::surface::young;
use crate::world::{Field, WorldState};

/// The five biome inputs, in the router's names (plan §7.1).
pub const BIOME_INPUTS: [&str; 5] = crate::synth::CLIMATE;

fn to_raw(v: f32) -> i16 {
    crate::synth::quantise(f64::from(v) * 10_000.0)
}

pub fn bake(
    world: &WorldState,
    p: &PlanetPreset,
    surface: &SurfaceState,
    out: &mut Out<'_>,
) -> Result<Value, CoreError> {
    let (l1, l2) = grids(p)?;
    let t = field(world, climate::field::T_MEAN);
    let pr = field(world, climate::field::PRECIP);
    let kp = field(world, climate::field::KOPPEN);

    // Physical climate on its own grid (L1).
    let mut physical = Vec::new();
    if let Some(f) = t {
        let raw: Vec<i16> = f
            .data
            .iter()
            .map(|&v| crate::synth::quantise(f64::from(v) * 100.0))
            .collect();
        out.layer(
            desc("climate/t_mean_c", DType::I16, &f.grid, 0.01, "degC", None),
            &raw,
        )?;
        physical.push("climate/t_mean_c");
    }
    if let Some(f) = pr {
        let raw: Vec<u16> = f
            .data
            .iter()
            .map(|&v| {
                if v.is_nan() {
                    0
                } else {
                    f64::from(v).round().clamp(0.0, 65_535.0) as u16
                }
            })
            .collect();
        out.layer(
            desc("climate/p_mm", DType::U16, &f.grid, 1.0, "mm/yr", None),
            &raw,
        )?;
        physical.push("climate/p_mm");
    }
    if let Some(f) = kp {
        let raw: Vec<u8> = f
            .data
            .iter()
            .map(|&v| {
                if v.is_nan() {
                    climate::koppen::SEA
                } else {
                    (v.round() as i64).clamp(0, 31) as u8
                }
            })
            .collect();
        out.layer(
            desc(
                "climate/koppen",
                DType::U8,
                &f.grid,
                1.0,
                "koppen code index (notes.bake.climate.koppen_codes)",
                Some(255.0),
            ),
            &raw,
        )?;
        physical.push("climate/koppen");
    }

    // The five biome inputs on L2.
    let sea = surface.sea_level_m;
    let elev: Vec<f32> = match field(world, young::field::ELEVATION_M) {
        Some(f) => resample(f, &l2, false),
        None => resample(
            &Field {
                grid: surface.grid.clone(),
                data: surface.elevation_m.clone(),
            },
            &l2,
            false,
        ),
    };
    let on_l1 = |name: &str| field(world, name).filter(|f| f.grid.w == l1.w && f.grid.h == l1.h);
    let (t1, k1) = (
        t.filter(|f| f.grid.w == l1.w),
        kp.filter(|f| f.grid.w == l1.w),
    );
    let climate_in = match (t1, k1) {
        (Some(t_mean), Some(koppen)) => Some(Climate {
            t_mean,
            t_warm: on_l1(climate::field::T_WARM),
            t_cold: on_l1(climate::field::T_COLD),
            aridity: on_l1(climate::field::ARIDITY),
            koppen,
            glacier: on_l1(climate::field::GLACIER),
        }),
        _ => None,
    };
    let params = p.biomes.clone().unwrap_or_default();
    let mapped = biomes::map(
        climate_in.as_ref(),
        &Ground {
            elevation_m: &elev,
            sea_level_m: sea,
            l1: &l1,
            l2: &l2,
            seed: p.seed,
            circumference: p.scale.circumference_blocks,
        },
        &params,
    );
    for (name, data) in BIOME_INPUTS.iter().zip(&mapped.layers) {
        let raw: Vec<i16> = data.iter().map(|&v| to_raw(v)).collect();
        out.layer(
            desc(name, DType::I16, &l2, 1e-4, "vanilla_param", None),
            &raw,
        )?;
    }
    let mut note = json!({
        "physical": physical,
        "koppen_codes": climate::koppen::CODES,
        "biome_inputs": BIOME_INPUTS,
        "grid": l2,
        "from_climate_stage": climate_in.is_some(),
        "default_params": params == BiomeParams::default(),
    });
    if let (Value::Object(n), Value::Object(m)) = (&mut note, mapped.note) {
        n.extend(m);
    }
    Ok(note)
}

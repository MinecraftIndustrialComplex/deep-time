// SPDX-License-Identifier: LGPL-3.0-or-later
//! The climate part of the bake.
//!
//! **Physical climate on L1** (plan §5.3; the viewer's and the rules'): `climate/t_mean_c` (i16,
//! 0.01 °C), `climate/p_mm` (u16, mm/yr), `climate/koppen` (u8, the index of
//! `climate::koppen::CODES`, 31 = sea), from the climate stage's last snapshot.
//!
//! **The twelve months on L1** (format 0.2, P1-25: what Deep Time's climate API reads):
//! `climate/month/t_sl_c_{01..12}` (i16, 0.01 °C, each month's temperature *at sea level*, so a
//! reader applies the lapse rate at its own block height; on land it holds the cell's plateau
//! warmth, `climate::lapse::mass_elevation_*`, MOI-085) and `climate/month/p_share_{01..12}`
//! (u8, 1/40: each month's precipitation rate over the annual, 0–6.375; a month's precipitation is
//! `p_mm` × share / 12 after the twelve are normalised to sum 12), from the climate stage's last
//! snapshot. Month 01 starts at solar longitude 280°, about 1 January on Earth.
//! `notes.bake.climate.monthly` records the layers, the lapse rate and the month convention.
//!
//! **The five biome inputs on L2** (`climate/{temperature, vegetation, continents, erosion,
//! ridges}`, i16, vanilla parameter × 10,000): what the noise router's climate functions read
//! (plan §7.1), from the physical climate and the ground by the §7.2 mapping in [`super::biomes`]
//! (P1-09; the preset's `biomes` section holds its knobs). The game maps them once more per world
//! onto the pack's bands and distributions (`ClimateMappingBuilder`, `climate_mapping.json`).
//!
//! **The wetland flag** (`climate/wetland`, L2, u8): 0 none, 1 temperate or warm wetland (vanilla's
//! swamp band), 2 cold wetland (boreal peatland: taiga in vanilla, the pack's peatland biome where it
//! has one; BIO-133).
//!
//! **The ice cover** (`climate/ice_cover`, L2, u8, blocks): how much snow and ice the game's surface
//! pass lays where the climate's land ice stands, whatever the biome ([`super::ice`]). It is
//! decided before the biome inputs, which take covered land as glaciated EF.

use deeptime_format::{DType, GridDesc, TilePackReader};
use serde_json::{Value, json};

use super::biomes::{self, BiomeParams, Climate, Ground};
use super::ice;
use super::placement;
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
    let monthly = bake_monthly(world, p, out)?;

    // The five biome inputs on L2, on the world's own ground (ledger: BIO-187): the terrain part's
    // `elevation` layer, which the game's terrain and its biome rules stand on, not the young
    // landscape the amplifier reshaped (its volcano shaper alone puts shields' young ground about
    // 0.8 km above the world's). Only a bake without that layer falls back to the young landscape
    // or the surface.
    let sea = surface.sea_level_m;
    let baked = match out.m.layer("elevation") {
        Some(e) => world_ground_l2(
            &out.dir.join(&e.path),
            &l2,
            sea,
            f64::from(p.scale.sea_level_y),
            p.scale.m_per_block_v,
        )?,
        None => None,
    };
    let (elev, ground): (Vec<f32>, &str) = match (baked, field(world, young::field::ELEVATION_M)) {
        (Some(g), _) => (g, "elevation (the world's L3 ground, L2 footprint mean)"),
        (None, Some(f)) => (resample(f, &l2, false), "young (L2, bilinear)"),
        (None, None) => (
            resample(
                &Field {
                    grid: surface.grid.clone(),
                    data: surface.elevation_m.clone(),
                },
                &l2,
                false,
            ),
            "surface (L1, bilinear)",
        ),
    };
    let on_l1 = |name: &str| field(world, name).filter(|f| f.grid.w == l1.w && f.grid.h == l1.h);
    let table = p.placement.as_ref().unwrap_or_else(|| placement::builtin());
    // The ice cover of the surface pass (`ice.rs`): blocks of snow and ice where the climate's
    // land ice stands, carried down to L2 by its extent and its bodies' balance. It comes first:
    // the biome inputs take the covered L2 cells as glaciated EF (BIO-158). A run without the
    // climate's ice thickness has none.
    let spec = table.ice_cover();
    let cover = on_l1(climate::field::ICE_THICKNESS).map(|thickness| {
        let cp = p.climate.clone().unwrap_or_default();
        ice::downscale(
            &ice::Inputs {
                thickness,
                ela: on_l1(climate::field::ELA),
                precip: on_l1(climate::field::PRECIP),
                elevation_m: &elev,
                l2: &l2,
                sea_m: sea,
                law: ice::Law {
                    beta_acc: cp.ice_accumulation_gradient,
                    beta_abl: cp.ice_ablation_gradient,
                    acc_max: cp.ice_accumulation_max_m_yr,
                },
            },
            spec,
        )
    });
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
            ice_cover: cover.as_ref().map(|c| c.blocks.as_slice()),
            months: {
                let t: Vec<&Field> = (0..12)
                    .filter_map(|m| on_l1(&climate::field::t_sl_month(m)))
                    .collect();
                let pr: Vec<&Field> = (0..12)
                    .filter_map(|m| on_l1(&climate::field::p_month(m)))
                    .collect();
                (t.len() == 12 && pr.len() == 12).then(|| biomes::Months {
                    t_sl: std::array::from_fn(|m| t[m]),
                    p: std::array::from_fn(|m| pr[m]),
                    lapse_c_per_km: p.climate.as_ref().map_or(6.5, |c| c.lapse_c_per_km),
                })
            },
        }),
        _ => None,
    };
    let params = p.biomes.clone().unwrap_or_default();
    // Closed basins below sea level are land to the biomes (HYD-100; ledger: HYD-104).
    let closed = crate::hydro::closed_below_sea(
        &l2,
        &elev,
        sea,
        p.scale.m_per_block_v,
        p.scale.m_per_block_h,
    );
    let mapped = biomes::map(
        climate_in.as_ref(),
        &Ground {
            elevation_m: &elev,
            sea_level_m: sea,
            l1: &l1,
            l2: &l2,
            seed: p.seed,
            circumference: p.scale.circumference_blocks,
            closed: Some(&closed),
        },
        &params,
        table,
    );
    for (name, data) in BIOME_INPUTS.iter().zip(&mapped.layers) {
        let raw: Vec<i16> = data.iter().map(|&v| to_raw(v)).collect();
        out.layer(
            desc(name, DType::I16, &l2, 1e-4, "vanilla_param", None),
            &raw,
        )?;
    }
    // Which land is wetland (BIO-133): 1 = swamp band (temperate and warm), 2 = cold wetland, where
    // the game puts the pack's peatland biome if it has one.
    out.layer(
        desc(
            "climate/wetland",
            DType::U8,
            &l2,
            1.0,
            "0 none, 1 swamp band (E6), 2 cold wetland (peatland)",
            None,
        ),
        &mapped.wetland,
    )?;
    // The Köppen class the biome inputs were placed by, when it was decided on L2 (each cell's
    // temperatures lapsed to its own height, P1-26): `climate/koppen` is the climate stage's L1
    // class, which differs from it wherever an L1 cell holds both high and low ground (a plateau's
    // edge, a mountain belt), so a check of biomes against classes reads this one.
    if !mapped.class_l2.is_empty() {
        out.layer(
            desc(
                "climate/koppen_l2",
                DType::U8,
                &l2,
                1.0,
                "koppen code index on L2, the biome inputs' class (notes.bake.climate.koppen_codes)",
                Some(255.0),
            ),
            &mapped.class_l2,
        )?;
    }
    let mut ice_note = Value::Null;
    if let Some(cover) = &cover {
        out.layer(
            desc("climate/ice_cover", DType::U8, &l2, 1.0, "blocks", None),
            &cover.blocks,
        )?;
        ice_note = spec.record();
        ice_note["source"] = json!(climate::field::ICE_THICKNESS);
        ice_note["rule"] = json!(
            "the climate's L1 extent (nearest L1 cell with at least 1 m of ice), its bodies \
             closed on their surface mass balance (BIO-082, BIO-157)"
        );
        ice_note["cells"] = json!(cover.blocks.iter().filter(|&&b| b > 0).count());
        ice_note["climate_cells"] = json!(cover.climate_cells);
        ice_note["trimmed_cells"] = json!(cover.trimmed_cells);
        ice_note["bodies"] = json!(cover.bodies);
        ice_note["trimmed_bodies"] = json!(cover.trimmed_bodies);
        ice_note["thickest_blocks"] = json!(cover.blocks.iter().copied().max().unwrap_or(0));
        physical.push("climate/ice_cover");
    }
    let mut note = json!({
        "ice_cover": ice_note,
        "physical": physical,
        "koppen_codes": climate::koppen::CODES,
        "biome_inputs": BIOME_INPUTS,
        "grid": l2,
        "ground": ground,
        "from_climate_stage": climate_in.is_some(),
        "monthly": monthly,
        "default_params": params == BiomeParams::default(),
    });
    if let (Value::Object(n), Value::Object(m)) = (&mut note, mapped.note) {
        n.extend(m);
    }
    Ok(note)
}

/// The nearest node of a grid axis to a block, as up to two (node, weight) pairs: one node of
/// weight 1, or half to each of the two nodes on an exact half-way (`block` and `origin` in blocks,
/// `r` blocks a node, `len` nodes; x wraps, z clamps to the grid's rows).
fn nearest_nodes(block: i64, origin: i64, r: u32, len: usize, wraps: bool) -> [(usize, f32); 2] {
    let (off, r) = (block - origin, i64::from(r));
    let (base, rem) = (off.div_euclid(r), off.rem_euclid(r));
    let n = len as i64;
    let fix = |i: i64| {
        if wraps {
            i.rem_euclid(n) as usize
        } else {
            i.clamp(0, n - 1) as usize
        }
    };
    match (2 * rem).cmp(&r) {
        std::cmp::Ordering::Less => [(fix(base), 1.0), (fix(base), 0.0)],
        std::cmp::Ordering::Greater => [(fix(base + 1), 1.0), (fix(base + 1), 0.0)],
        std::cmp::Ordering::Equal => [(fix(base), 0.5), (fix(base + 1), 0.5)],
    }
}

/// The world's ground on `l2`, m (ledger: BIO-187): the terrain part's `elevation` layer at
/// `path` (block y on L3, the height the game's terrain, its biome rules and its climate API stand
/// on, FORMAT.md "The terrain height", without the detail term, whose mean is 0) in metres,
/// `sea_m + (y − sea_y) × m_per_block_v`, the stored y as the game reads it (no build-fit expand),
/// averaged over each L2 node's footprint: every elevation node goes to its nearest L2 node, half
/// to each of two on an exact half-way ([`nearest_nodes`]). The tiles are read one at a time and
/// summed in their order: no L3-sized buffer, and the same sums whatever the thread count. `None`
/// when the layer is not on a grid this can average onto `l2` (a cube grid, no x wrap, cells
/// coarser than L2's, another circumference) or leaves an L2 node without a value.
pub(crate) fn world_ground_l2(
    path: &std::path::Path,
    l2: &GridDesc,
    sea_m: f64,
    sea_y: f64,
    m_per_block_v: f64,
) -> Result<Option<Vec<f32>>, CoreError> {
    let reader = TilePackReader::open(path)?;
    let d = reader.desc();
    let g = &d.grid;
    let cb = d.cell_bytes();
    if g.is_cube()
        || l2.is_cube()
        || !g.wraps_x
        || !l2.wraps_x
        || g.r > l2.r
        || u64::from(g.w) * u64::from(g.r) != u64::from(l2.w) * u64::from(l2.r)
        || d.dtype.scalar_bytes().is_none()
    {
        return Ok(None);
    }
    let (w2, h2) = (l2.w as usize, l2.h as usize);
    let cols: Vec<[(usize, f32); 2]> = (0..i64::from(g.w))
        .map(|i| nearest_nodes(g.block_x(i), l2.origin_block_x, l2.r, w2, true))
        .collect();
    let rows: Vec<[(usize, f32); 2]> = (0..i64::from(g.h))
        .map(|j| nearest_nodes(g.block_z(j), l2.origin_block_z, l2.r, h2, false))
        .collect();
    let mut sum = vec![0f32; w2 * h2];
    let mut weight = vec![0f32; w2 * h2];
    for t in 0..d.tile_count() {
        let cells = reader.read_tile(t)?;
        let (x0, z0, tw, _) = d.tile_rect(t);
        for (k, row) in cells.chunks_exact(tw as usize * cb).enumerate() {
            let zs = rows[z0 as usize + k];
            for (c, cell) in row.chunks_exact(cb).enumerate() {
                let y = d.physical(d.dtype.decode_scalar(cell));
                if y.is_nan() {
                    continue;
                }
                let xs = cols[x0 as usize + c];
                for &(j2, az) in zs.iter().filter(|z| z.1 > 0.0) {
                    for &(i2, ax) in xs.iter().filter(|x| x.1 > 0.0) {
                        let k2 = j2 * w2 + i2;
                        sum[k2] += (y - sea_y) as f32 * ax * az;
                        weight[k2] += ax * az;
                    }
                }
            }
        }
    }
    if weight.iter().any(|&w| w <= 0.0) {
        return Ok(None);
    }
    Ok(Some(
        sum.iter()
            .zip(&weight)
            .map(|(&s, &w)| (sea_m + f64::from(s / w) * m_per_block_v) as f32)
            .collect(),
    ))
}

/// Layer names of the twelve months (P1-25; FORMAT.md "The twelve months").
pub fn t_sl_month_layer(m: usize) -> String {
    format!("climate/month/t_sl_c_{:02}", m + 1)
}

/// See [`t_sl_month_layer`].
pub fn p_share_month_layer(m: usize) -> String {
    format!("climate/month/p_share_{:02}", m + 1)
}

/// The scale of `climate/month/p_share_*` (u8 × 1/40: 0–6.375 times the annual mean rate).
pub const P_SHARE_SCALE: f64 = 1.0 / 40.0;

/// Writes the twelve months when the climate stage left them (its last snapshot); returns the
/// note (`null` when there are none: a preset without a climate section).
fn bake_monthly(
    world: &WorldState,
    p: &PlanetPreset,
    out: &mut Out<'_>,
) -> Result<Value, CoreError> {
    let t: Vec<&Field> = (0..12)
        .filter_map(|m| field(world, &climate::field::t_sl_month(m)))
        .collect();
    let pr: Vec<&Field> = (0..12)
        .filter_map(|m| field(world, &climate::field::p_month(m)))
        .collect();
    if t.len() != 12 || pr.len() != 12 {
        return Ok(Value::Null);
    }
    let grid = t[0].grid.clone();
    let n = grid.cells();
    if t.iter().chain(&pr).any(|f| f.grid != grid) {
        return Err(CoreError::InvalidState(
            "the climate stage's twelve months are not on one grid".into(),
        ));
    }
    for (m, f) in t.iter().enumerate() {
        let raw: Vec<i16> = f
            .data
            .iter()
            .map(|&v| crate::synth::quantise(f64::from(v) * 100.0))
            .collect();
        out.layer(
            desc(
                &t_sl_month_layer(m),
                DType::I16,
                &grid,
                0.01,
                "degC at sea level",
                None,
            ),
            &raw,
        )?;
    }
    // Each month's rate over the mean of the twelve (the annual precipitation).
    let mean: Vec<f64> = (0..n)
        .map(|c| pr.iter().map(|f| f64::from(f.data[c])).sum::<f64>() / 12.0)
        .collect();
    for (m, f) in pr.iter().enumerate() {
        let raw: Vec<u8> = f
            .data
            .iter()
            .zip(&mean)
            .map(|(&v, &a)| {
                let share = if a > 0.0 && v.is_finite() {
                    f64::from(v) / a
                } else {
                    1.0
                };
                (share / P_SHARE_SCALE).round().clamp(0.0, 255.0) as u8
            })
            .collect();
        out.layer(
            desc(
                &p_share_month_layer(m),
                DType::U8,
                &grid,
                P_SHARE_SCALE,
                "month rate / annual mean rate",
                None,
            ),
            &raw,
        )?;
    }
    let lapse = p.climate.clone().unwrap_or_default().lapse_c_per_km;
    Ok(json!({
        "t_sl": (0..12).map(t_sl_month_layer).collect::<Vec<_>>(),
        "p_share": (0..12).map(p_share_month_layer).collect::<Vec<_>>(),
        "lapse_c_per_km": lapse,
        "months": "12 equal steps of solar longitude from 280 deg (month 01 about 1 January)",
    }))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::bake::write_layer;

    const SEA_M: f64 = 1136.5;
    const SEA_Y: f64 = 63.0;
    const MV: f64 = 35.0;

    /// A planet of circumference 256: L2 32 × 16 at r 8, the elevation on 128 × 64 at r 2.
    fn grids() -> (GridDesc, GridDesc) {
        (
            GridDesc::for_planet("L2", 256, 8).unwrap(),
            GridDesc::for_planet("L3", 256, 2).unwrap(),
        )
    }

    /// Writes `y(i, j)` (block y of elevation node i, j) as the terrain part does (i16, 1/16
    /// block), in tiles of `tile` cells, and returns its path.
    fn elevation(name: &str, l3: &GridDesc, tile: u32, y: impl Fn(i64, i64) -> f64) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "deeptime-ground-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut d = desc("elevation", DType::I16, l3, 1.0 / 16.0, "block", None);
        d.tile = tile;
        let mut bytes = Vec::new();
        for j in 0..i64::from(l3.h) {
            for i in 0..i64::from(l3.w) {
                bytes.extend_from_slice(&((y(i, j) * 16.0).round() as i16).to_le_bytes());
            }
        }
        let path = dir.join("elevation.dtr");
        write_layer(&path, &d, &bytes).unwrap();
        path
    }

    #[test]
    fn a_node_takes_the_nearest_node_or_half_of_two() {
        // L2 nodes every 8 blocks from -128: block -128 is node 0, -125 still node 0, -124 half
        // way, -123 node 1; x wraps (block 124 is half to node 31 and node 0), z clamps.
        assert_eq!(nearest_nodes(-128, -128, 8, 32, true), [(0, 1.0), (0, 0.0)]);
        assert_eq!(nearest_nodes(-125, -128, 8, 32, true), [(0, 1.0), (0, 0.0)]);
        assert_eq!(nearest_nodes(-124, -128, 8, 32, true), [(0, 0.5), (1, 0.5)]);
        assert_eq!(nearest_nodes(-123, -128, 8, 32, true), [(1, 1.0), (1, 0.0)]);
        assert_eq!(nearest_nodes(124, -128, 8, 32, true), [(31, 0.5), (0, 0.5)]);
        assert_eq!(nearest_nodes(126, -64, 8, 16, false), [(15, 1.0), (15, 0.0)]);
        assert_eq!(nearest_nodes(-70, -64, 8, 16, false), [(0, 1.0), (0, 0.0)]);
    }

    #[test]
    fn the_ground_is_the_baked_elevation_in_metres() {
        // Flat ground 10 blocks above sea level is 350 m above the model's sea everywhere, a
        // drowned floor 4 blocks under it 140 m below: the stored y, as the game reads it.
        let (l2, l3) = grids();
        for (name, dy, want) in [("land", 10.0, SEA_M + 350.0), ("sea", -4.0, SEA_M - 140.0)] {
            let path = elevation(name, &l3, 256, |_, _| SEA_Y + dy);
            let g = world_ground_l2(&path, &l2, SEA_M, SEA_Y, MV)
                .unwrap()
                .unwrap();
            assert_eq!(g.len(), l2.cells());
            for &v in &g {
                assert!((f64::from(v) - want).abs() < 1e-3, "{v} != {want}");
            }
        }
    }

    #[test]
    fn a_node_is_its_footprints_mean_and_the_tiles_do_not_matter() {
        // A node's footprint is the elevation nodes within half an L2 cell of it: offsets -2..=2
        // of r 2 (blocks -4..=4), the outer two at half weight each way (4 by 4 of weight in all).
        let (l2, l3) = grids();
        let y = |i: i64, j: i64| SEA_Y + ((i * 7 + j * 13) % 23) as f64 + 0.25 * (i % 4) as f64;
        let one = elevation("one", &l3, 256, y);
        let many = elevation("many", &l3, 16, y);
        let a = world_ground_l2(&one, &l2, SEA_M, SEA_Y, MV)
            .unwrap()
            .unwrap();
        let b = world_ground_l2(&many, &l2, SEA_M, SEA_Y, MV)
            .unwrap()
            .unwrap();
        assert_eq!(a, b, "the same sums whatever the tiling");
        let w = |o: i64| if o.abs() == 2 { 0.5 } else { 1.0 };
        let mean = |i2: i64, j2: i64| {
            let (mut s, mut n) = (0.0, 0.0);
            for oj in -2..=2i64 {
                for oi in -2..=2i64 {
                    let i3 = (4 * i2 + oi).rem_euclid(i64::from(l3.w));
                    let j3 = 4 * j2 + oj;
                    s += w(oi) * w(oj) * (y(i3, j3) - SEA_Y);
                    n += w(oi) * w(oj);
                }
            }
            SEA_M + s / n * MV
        };
        // An inner node, and the first and last nodes of a row, whose footprints wrap.
        for (i2, j2) in [(5, 7), (0, 3), (31, 8)] {
            let got = f64::from(a[j2 as usize * 32 + i2 as usize]);
            let want = mean(i2, j2);
            assert!((got - want).abs() < 1e-2, "node {i2},{j2}: {got} != {want}");
        }
    }

    #[test]
    fn a_coarser_or_foreign_grid_is_not_averaged() {
        let (l2, _) = grids();
        let coarse = GridDesc::for_planet("L1", 256, 16).unwrap();
        let path = elevation("coarse", &coarse, 256, |_, _| SEA_Y);
        assert!(
            world_ground_l2(&path, &l2, SEA_M, SEA_Y, MV)
                .unwrap()
                .is_none()
        );
        let other = GridDesc::for_planet("L3", 512, 2).unwrap();
        let path = elevation("other", &other, 256, |_, _| SEA_Y);
        assert!(
            world_ground_l2(&path, &l2, SEA_M, SEA_Y, MV)
                .unwrap()
                .is_none()
        );
    }
}

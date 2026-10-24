// SPDX-License-Identifier: LGPL-3.0-or-later
//! The floodplains (HYD-150 to HYD-152) on a synthetic planet (C = 1,024, 2 blocks a cell): one
//! valley running south to the sea, a flat floor 8 cells either side of its axis between steep
//! walls, every height a whole number of 1/16 blocks, so the height of each cell above the river
//! cell it drains to is known exactly.

use deeptime_format::GridDesc;

use super::network::{self, Floodplains, NODATA, Output};
use super::{Climate, HydroParams};

const C: u32 = 1024;
const SEA_RAW: i32 = 63 * 16;
/// The coast's row: land north of it, a shelf south.
const COAST: usize = 110;
/// The valley's axis (column) and its floor's half-width (cells).
const AXIS: i64 = 256;
const FLOOR: i64 = 8;
/// Rises, 1/16 block: down the valley a row, across the floor and across a wall a column. Across
/// the floor water takes the diagonal ((2 + 1)/√2 beats 2), so a floor cell d columns off the axis
/// reaches it d rows downstream, 3d above that river cell; across a wall it goes straight
/// (8 beats (8 + 1)/√2) to the floor's edge, so a wall cell d columns off stands
/// 3 FLOOR + 8 (d − FLOOR) above its river cell.
const DOWN: i32 = 1;
const ACROSS_FLOOR: i32 = 2;
const ACROSS_WALL: i32 = 8;
/// Metres per block vertically, chosen so the flood depth over the drains of rows 30 to 90 (7.7 to
/// 12.0 m for 4,200 to 18,200 km²) lies between the floor's highest cells (24/16) and the walls'
/// third column (48/16): 27/16 to 43/16.
const M_PER_BLOCK_V: f64 = 4.5;

fn grid() -> GridDesc {
    GridDesc::for_planet("L3", C, 2).unwrap()
}

fn off_axis(i: usize) -> i64 {
    (i as i64 - AXIS).abs()
}

fn raw(i: usize, j: usize) -> i32 {
    if j >= COAST {
        return SEA_RAW - 8 - 2 * (j - COAST) as i32;
    }
    let d = off_axis(i) as i32;
    let f = FLOOR as i32;
    let across = if d <= f {
        ACROSS_FLOOR * d
    } else {
        ACROSS_FLOOR * f + ACROSS_WALL * (d - f)
    };
    SEA_RAW + 8 + DOWN * (COAST - j) as i32 + across
}

fn elevation() -> Vec<i16> {
    let g = grid();
    let (w, h) = (g.w as usize, g.h as usize);
    (0..w * h).map(|c| raw(c % w, c / w) as i16).collect()
}

/// Only the axis is a river: the floor's cells gather a wall's row each (about 90 km²).
fn params() -> HydroParams {
    HydroParams {
        river_min_area_km2: 500.0,
        ..HydroParams::default()
    }
}

fn net(elev: &[i16], skip: &[u32]) -> Output {
    let g = grid();
    let climate = |_x: i64, _z: i64| Climate::steady(2500.0, 20.0);
    let params = params();
    network::run(&network::Input {
        grid: &g,
        elev,
        sea_raw: SEA_RAW,
        m_per_block_h: 300.0,
        climate: &climate,
        params: &params,
        width_scale: 1.0,
        mouths: false,
        floodplains: Some(Floodplains {
            m_per_block_v: M_PER_BLOCK_V,
            skip,
        }),
    })
}

/// The top block of ground or water whose surface is `h` (1/16 block): water or ground fills every
/// block under it.
fn top(h: i32) -> i32 {
    (h + 15).div_euclid(16) - 1
}

#[test]
fn gfplains_flood_depth_follows_its_code() {
    // h = 0.01 A^0.30, A in m².
    let close = |a: f64, b: f64| (a - b).abs() <= 1e-9 * b.abs();
    assert!(close(
        network::flood_depth_m(1_000.0),
        0.01 * 1e9f64.powf(0.3)
    ));
    assert!((network::flood_depth_m(1_000.0) - 5.012).abs() < 0.001);
    assert!((network::flood_depth_m(100_000.0) - 19.953).abs() < 0.001);
    assert_eq!(network::flood_depth_m(0.0), 0.0);
}

#[test]
fn the_bank_stands_a_block_over_the_water() {
    // A river a block under its valley floor (the freeboard): the plain is the floor itself.
    let (level, h) = network::bank_of(1100, 1084, 1_000.0, 35.0);
    assert_eq!(level, 1100);
    assert!((f64::from(h) - 5.012 / 35.0 * 16.0).abs() < 1e-3);
    // Water held up by a lake or the sea to the floor's height: the plain rises to the first
    // height whose top block is one over the water's.
    for w in [1008, 1009, 1015, 1016, 1030] {
        let (level, _) = network::bank_of(w as i16, w, 1_000.0, 35.0);
        assert_eq!(top(i32::from(level)), top(w) + 1, "water {w}");
        assert_eq!(
            top(i32::from(level) - 1),
            top(w),
            "water {w}: not the lowest such height"
        );
    }
    // A floor higher than that keeps its height.
    let (level, _) = network::bank_of(1040, 1008, 1_000.0, 35.0);
    assert_eq!(level, 1040);
}

#[test]
fn a_valley_floor_within_the_flood_depth_is_a_floodplain_at_its_rivers_bank() {
    let g = grid();
    let w = g.w as usize;
    let elev = elevation();
    let out = net(&elev, &[]);
    let fp = &out.floodplain;
    assert_eq!(fp.len(), elev.len());
    let (mut floor, mut walls_off) = (0, 0);
    for j in 30..=90 {
        for i in 0..w {
            let c = j * w + i;
            let d = off_axis(i);
            if d == 0 {
                // The river: in the plain, at its own bank, which is its ground (a block over its
                // water already).
                assert_eq!(fp[c], elev[c], "river cell ({i}, {j})");
            } else if d <= FLOOR {
                // The floor: lowered onto the river cell it drains to, d rows down the axis, its
                // top block a block over that cell's water (its ground less the freeboard).
                let drain = (j + d as usize) * w + AXIS as usize;
                assert_eq!(fp[c], elev[drain], "floor cell ({i}, {j})");
                assert_eq!(
                    i32::from(elev[c]) - i32::from(fp[c]),
                    3 * d as i32,
                    "floor cell ({i}, {j})"
                );
                let water = i32::from(elev[drain]) - 16;
                assert_eq!(top(i32::from(fp[c])), top(water) + 1);
                floor += 1;
            } else if d >= FLOOR + 3 {
                assert_eq!(fp[c], NODATA, "wall cell ({i}, {j}) on the plain");
                walls_off += 1;
            }
        }
    }
    assert_eq!(floor, 61 * 2 * FLOOR as usize);
    assert!(walls_off > 61 * 400);
    let s = &out.stats["floodplains"];
    assert!(s["cells"].as_u64().unwrap() >= floor as u64);
    assert_eq!(s["raised_cells"].as_u64(), Some(0));
    assert_eq!(s["basin_floor_cells"].as_u64(), Some(0));
}

#[test]
fn no_floodplain_on_skipped_cells_or_without_the_stage() {
    let g = grid();
    let w = g.w as usize;
    let elev = elevation();
    // Row 50's floor, as a delta plain's cells would be skipped.
    let mut skip: Vec<u32> = (AXIS - FLOOR..=AXIS + FLOOR)
        .filter(|&i| i != AXIS)
        .map(|i| (50 * w + i as usize) as u32)
        .collect();
    skip.sort_unstable();
    let out = net(&elev, &skip);
    for &c in &skip {
        assert_eq!(out.floodplain[c as usize], NODATA, "skipped cell {c}");
    }
    // Its neighbours up and down the valley are still on the plain.
    assert_ne!(out.floodplain[49 * w + AXIS as usize + 1], NODATA);
    assert_ne!(out.floodplain[51 * w + AXIS as usize + 1], NODATA);
    assert_eq!(
        out.stats["floodplains"]["skipped_cells"].as_u64(),
        Some(skip.len() as u64)
    );

    let params = params();
    let climate = |_x: i64, _z: i64| Climate::steady(2500.0, 20.0);
    let none = network::run(&network::Input {
        grid: &g,
        elev: &elev,
        sea_raw: SEA_RAW,
        m_per_block_h: 300.0,
        climate: &climate,
        params: &params,
        width_scale: 1.0,
        mouths: false,
        floodplains: None,
    });
    assert!(none.floodplain.is_empty());
    assert!(none.stats["floodplains"].is_null());
}

#[test]
fn a_dry_basins_floor_is_no_floodplain() {
    // A pit 2.5 blocks deep in the floor, 3 by 3 cells: filled, its cells either hold a lake (a
    // block deep or more) or, left dry by the balance, lie a block or more under the filled
    // surface. Neither is laid as a plain, which would fill the basin.
    let g = grid();
    let w = g.w as usize;
    let mut elev = elevation();
    let pit: Vec<usize> = (0..3)
        .flat_map(|dj| (0..3).map(move |di| (59 + dj) * w + (AXIS + 3) as usize + di))
        .collect();
    for &c in &pit {
        elev[c] -= 40;
    }
    let out = net(&elev, &[]);
    for &c in &pit {
        assert_eq!(out.floodplain[c], NODATA, "pit cell {c}");
    }
    // The floor beside the pit is still a plain.
    assert_ne!(out.floodplain[60 * w + AXIS as usize + 1], NODATA);
}

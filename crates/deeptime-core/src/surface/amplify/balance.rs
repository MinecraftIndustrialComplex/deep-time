// SPDX-License-Identifier: LGPL-3.0-or-later
//! The water the amplifier's first level holds: the bake's water balance on the young landscape
//! (ledger: AMP-095, AMP-096, AMP-097).
//!
//! The young landscape fills every closed basin to its spill point (`young::field::LAKE_M`).
//! The amplifier used to take that as the lakes it leaves alone, and the bake's hydrology later
//! found most of those basins too dry to hold a full lake: a terminal basin keeps the lake its
//! inflow and its evaporation balance, a playa, or nothing (HYD-050, HYD-053), whatever the
//! height of its floor when the open sea does not reach it (HYD-100). What the bake then drained
//! was ground the amplifier had never worked: the coarse level's flat floor, its straight walls
//! and the staircase of its cells, the "glitch" Ben saw in his savanna pit (2026-10-22).
//!
//! Here the bake's own hydrology ([`network::run`], the same parameters and climate) runs on the
//! young landscape before the ladder starts, and what it finds is what the amplifier works with:
//! - **Lakes**: the cells under a lake's surface (open or frozen) are the level's lakes, at that
//!   surface, a balance lake's level below its rim as well as a full one's.
//! - **Closed ground** (`resample::basin::CLOSED`): the `closed_basin` layer, ground below sea
//!   level the open sea does not reach (and its halo): never sea, so the passes work it as land,
//!   with no sea base level, no coast and no sea-level floor, however far down it lies.
//! - **Sinks** (`resample::basin::SINK`): the lowest cell of every basin the balance leaves dry or
//!   a playa. The flood drains the basin's floor to it instead of filling the basin to its rim,
//!   so the dry floor and the lower walls get the erosion, the slumping and the silt of any land,
//!   and the silt pass lays the catchment's sediment on the floor, as a closed basin collects
//!   its catchment's sediment (AMP-097).
//!
//! The bake runs the hydrology again on the finished L3 terrain; what it finds there is the same
//! basin, by then shaped like ground.

use deeptime_format::GridDesc;
use serde_json::{Value, json};

use super::resample::{Level, NO_LAKE, basin};
use crate::hydro::HydroParams;
use crate::hydro::network::{self, NODATA, kind};
use crate::params::ScaleParams;
use crate::world::WorldState;

/// Sets `level`'s lakes and basin classes from the water balance of its terrain (the module
/// docs). `grid` is the level's grid (L2), `s` and `params` the preset's scale and hydrology (the
/// bake's), `sea` the sea level, m. Returns the counts for `notes.amplify.balance`, or `None`,
/// with `level` untouched (the young landscape's spill-full lakes, every cell open), when the
/// hydrology is off.
pub fn apply(
    level: &mut Level,
    grid: &GridDesc,
    world: &WorldState,
    s: &ScaleParams,
    params: &HydroParams,
    sea: f64,
) -> Option<Value> {
    if !params.enabled {
        return None;
    }
    let started = std::time::Instant::now();
    let n = level.z.len();
    assert_eq!(
        (level.w, level.h),
        (grid.w as usize, grid.h as usize),
        "the level and its grid differ"
    );
    let (sea_y, mv) = (f64::from(s.sea_level_y), s.m_per_block_v);
    let sea_raw = s.sea_level_y * 16;
    // The terrain as the bake reads it, 1/16 block; deep sea where it has none (as
    // `water_view::work_terrain`).
    let deep = (sea_raw - 16 * 100).clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
    let elev: Vec<i16> = level
        .z
        .iter()
        .map(|&z| {
            if z.is_finite() {
                ((sea_y + (f64::from(z) - sea) / mv) * 16.0)
                    .round()
                    .clamp(f64::from(i16::MIN), f64::from(i16::MAX)) as i16
            } else {
                deep
            }
        })
        .collect();
    let climate = crate::water_view::climate_at(world, f64::from(s.circumference_blocks));
    let out = network::run(&network::Input {
        grid,
        elev: &elev,
        sea_raw,
        m_per_block_h: s.m_per_block_h,
        climate: &climate,
        params,
        width_scale: s.feature_scale.rivers,
    });
    let young_lake_cells = level.lake_z.iter().filter(|s| s.is_finite()).count();
    let to_m = |raw: i16| (sea + (f64::from(raw) / 16.0 - sea_y) * mv) as f32;
    let mut lake_cells = 0u64;
    for c in 0..n {
        let k = out.kind[c];
        let wl = out.water_level[c];
        level.lake_z[c] = if (k == kind::LAKE || k == kind::FROZEN) && wl != NODATA {
            lake_cells += 1;
            to_m(wl)
        } else {
            NO_LAKE
        };
    }
    level.closed = out
        .closed_basin
        .iter()
        .map(|&b| if b == 1 { basin::CLOSED } else { basin::OPEN })
        .collect();
    for &c in &out.sinks {
        level.closed[c as usize] = basin::SINK;
    }
    let low = |c: usize| i32::from(elev[c]) <= sea_raw;
    let closed_low = (0..n)
        .filter(|&c| level.closed[c] != basin::OPEN && low(c))
        .count();
    let playa_cells = out.kind.iter().filter(|&&k| k == kind::PLAYA).count();
    let lakes_below_sea = out
        .lakes
        .iter()
        .filter(|l| i32::from(l.level) < sea_raw)
        .count();
    Some(json!({
        "lakes": out.lakes.len(),
        "lakes_below_sea_level": lakes_below_sea,
        "lake_cells": lake_cells,
        "young_lake_cells": young_lake_cells,
        "playa_cells": playa_cells,
        "sinks": out.sinks.len(),
        "closed_cells_below_sea_level": closed_low,
        "millis": (started.elapsed().as_secs_f64() * 1000.0).round(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Field;

    /// Cells across the test planet and its rows (the grid runs past the Lambert poles: rows 4
    /// to 44 are on the planet).
    const W: usize = 128;
    const H: usize = 48;
    /// The cell size, blocks.
    const R: u32 = 8;

    fn scale() -> ScaleParams {
        serde_json::from_value(json!({"circumference_blocks": W as u32 * R})).expect("scale")
    }

    /// An L2 level at sea level 0 m: high ground at +700 m (y 83) with a cone-shaped bowl
    /// `depth` m deep at (64, 20), six cells in radius, and the open sea (−2,000 m) from row 36.
    fn bowl(depth: f32) -> (Level, GridDesc) {
        let s = scale();
        let grid = GridDesc::for_planet("L2", s.circumference_blocks, R).expect("grid");
        assert_eq!((grid.w as usize, grid.h as usize), (W, H));
        let mut z = vec![700.0f32; W * H];
        for j in 0..H {
            for i in 0..W {
                let r = ((i as f32 - 64.0).powi(2) + (j as f32 - 20.0).powi(2)).sqrt();
                if r < 6.0 {
                    z[j * W + i] = 700.0 - depth * (1.0 - r / 6.0);
                }
                if j >= 36 {
                    z[j * W + i] = -2000.0;
                }
            }
        }
        let n = W * H;
        let level = Level {
            w: W,
            h: H,
            dx: f64::from(R) * s.m_per_block_h,
            z,
            area: vec![1.0e6; n],
            wgt: vec![1.0; n],
            kfac: vec![1.0; n],
            lake_z: vec![NO_LAKE; n],
            zs: Vec::new(),
            closed: Vec::new(),
        };
        (level, grid)
    }

    /// A world whose climate is `rain` mm a year at a mean of `t` °C everywhere.
    fn world(grid: &GridDesc, t: f32, rain: f32) -> WorldState {
        let mut w = WorldState::new(1, W as u32 * R, 0.0);
        for (name, v) in [
            (crate::surface::climate::T_FIELD, t),
            (crate::surface::climate::P_FIELD, rain),
        ] {
            w.fields.insert(
                name.to_owned(),
                Field {
                    grid: grid.clone(),
                    data: vec![v; grid.cells()],
                },
            );
        }
        w
    }

    const CENTRE: usize = 20 * W + 64;

    #[test]
    fn a_dry_basin_below_sea_level_is_closed_ground_with_its_sink() {
        // 1,500 m deep: the floor 800 m below sea level. A hot desert: the balance leaves it dry.
        let (mut level, grid) = bowl(1500.0);
        let w = world(&grid, 28.0, 20.0);
        let stats = apply(
            &mut level,
            &grid,
            &w,
            &scale(),
            &HydroParams::default(),
            0.0,
        )
        .expect("hydrology on");
        assert_eq!(level.closed[CENTRE], basin::SINK, "{stats}");
        assert!(!level.is_lake(CENTRE), "{stats}");
        let below: Vec<usize> = (0..W * 36).filter(|&c| level.z[c] < 0.0).collect();
        assert!(below.len() > 10);
        for &c in &below {
            assert_ne!(
                level.closed[c],
                basin::OPEN,
                "bowl cell {c} is open: {stats}"
            );
        }
        // The sea stays open.
        assert_eq!(level.closed[40 * W + 5], basin::OPEN);
        assert_eq!(stats["sinks"], 1, "{stats}");
    }

    #[test]
    fn a_wet_basin_keeps_a_lake_and_no_sink() {
        let (mut level, grid) = bowl(300.0);
        let w = world(&grid, 8.0, 2500.0);
        let stats = apply(
            &mut level,
            &grid,
            &w,
            &scale(),
            &HydroParams::default(),
            0.0,
        )
        .expect("hydrology on");
        assert!(level.is_lake(CENTRE), "{stats}");
        assert!(level.lake_z[CENTRE] > level.z[CENTRE], "{stats}");
        assert_eq!(stats["sinks"], 0, "{stats}");
        assert!(level.closed.iter().all(|&k| k != basin::SINK));
    }

    #[test]
    fn no_hydrology_leaves_the_level_alone() {
        let (mut level, grid) = bowl(300.0);
        let off = HydroParams {
            enabled: false,
            ..HydroParams::default()
        };
        let w = world(&grid, 8.0, 2500.0);
        assert!(apply(&mut level, &grid, &w, &scale(), &off, 0.0).is_none());
        assert!(level.closed.is_empty());
        assert!(level.lake_z.iter().all(|s| !s.is_finite()));
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The meta part of the bake, on L1 (plan §5.3): `meta/plate_id` (u16), `meta/crust_age_myr`
//! (u16, Myr; nodata 65535 on continental crust, whose formation age the plate model does not
//! track) and `meta/province` (u16), for the viewer, "Why here?" and legends; `province` also
//! drives TerraBlender's regions (plan §7.3, P1-10).
//!
//! **Provinces.** Every L1 cell gets a geologic setting from the plate model's `regime` and the
//! surface: ocean (ocean basin, ridge, trench under water), island (the same above water),
//! arc (arc, back-arc), orogen (collision), rift, hotspot, craton, highland (a craton above
//! [`HIGHLAND_M`]) or margin (passive margin). A province is a connected region of one setting
//! (4-neighbours, x wraps); regions smaller than [`MIN_PROVINCE_KM2`] are absorbed by the nearest
//! larger one (a breadth-first flood from the larger regions), so provinces are contiguous and at
//! least that large. Ids count from 0 in the order of each province's first cell (row-major).

use std::collections::VecDeque;

use deeptime_format::DType;
use serde_json::{Value, json};

use super::{Out, desc, field, grids, nearest};
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::plates::state::regime;
use crate::surface::SurfaceState;
use crate::world::WorldState;

/// A province is at least this large, km² (a region the size of a small country).
pub const MIN_PROVINCE_KM2: f64 = 50_000.0;
/// Continental interior above this, m above the sea, is a highland rather than a craton.
pub const HIGHLAND_M: f64 = 1500.0;

/// Province settings, by code.
pub const SETTINGS: [&str; 10] = [
    "ocean", "island", "arc", "orogen", "rift", "hotspot", "craton", "highland", "margin", "other",
];

fn setting(reg: f32, above_sea_m: f64) -> u8 {
    let land = above_sea_m > 0.0;
    match reg.round() as i64 {
        r if r == i64::from(regime::OCEAN_BASIN)
            || r == i64::from(regime::RIDGE)
            || r == i64::from(regime::TRENCH) =>
        {
            if land {
                1
            } else {
                0
            }
        }
        r if r == i64::from(regime::ARC) || r == i64::from(regime::BACK_ARC) => 2,
        r if r == i64::from(regime::COLLISION) => 3,
        r if r == i64::from(regime::RIFT) => 4,
        r if r == i64::from(regime::HOTSPOT) => 5,
        r if r == i64::from(regime::CRATON) => {
            if above_sea_m > HIGHLAND_M {
                7
            } else {
                6
            }
        }
        r if r == i64::from(regime::PASSIVE_MARGIN) => 8,
        _ => 9,
    }
}

/// Province ids per cell (row-major, `w` × `h`, x wraps) of the settings `set`, with regions under
/// `min_cells` absorbed; returns (ids, setting of each province, cells of each province).
pub fn provinces(
    set: &[u8],
    w: usize,
    h: usize,
    min_cells: usize,
) -> (Vec<u16>, Vec<u8>, Vec<usize>) {
    let n = w * h;
    let neighbours = |c: usize| {
        let (i, j) = (c % w, c / w);
        let mut v = [usize::MAX; 4];
        v[0] = j * w + (i + w - 1) % w;
        v[1] = j * w + (i + 1) % w;
        if j > 0 {
            v[2] = c - w;
        }
        if j + 1 < h {
            v[3] = c + w;
        }
        v
    };
    // Connected regions of one setting.
    let mut region = vec![u32::MAX; n];
    let mut sizes: Vec<usize> = Vec::new();
    let mut queue = VecDeque::new();
    for start in 0..n {
        if region[start] != u32::MAX {
            continue;
        }
        let id = sizes.len() as u32;
        region[start] = id;
        queue.push_back(start);
        let mut size = 0;
        while let Some(c) = queue.pop_front() {
            size += 1;
            for nb in neighbours(c) {
                if nb != usize::MAX && region[nb] == u32::MAX && set[nb] == set[start] {
                    region[nb] = id;
                    queue.push_back(nb);
                }
            }
        }
        sizes.push(size);
    }
    // The large regions keep their cells; the small ones are flooded from them.
    let mut label = vec![u32::MAX; n];
    for c in 0..n {
        if sizes[region[c] as usize] >= min_cells {
            label[c] = region[c];
            queue.push_back(c);
        }
    }
    if queue.is_empty() {
        // No region is large enough (a tiny planet): one province.
        return (vec![0; n], vec![set.first().copied().unwrap_or(9)], vec![n]);
    }
    while let Some(c) = queue.pop_front() {
        for nb in neighbours(c) {
            if nb != usize::MAX && label[nb] == u32::MAX {
                label[nb] = label[c];
                queue.push_back(nb);
            }
        }
    }
    // Ids in order of first appearance.
    let mut remap = vec![u16::MAX; sizes.len()];
    let mut settings = Vec::new();
    let mut counts: Vec<usize> = Vec::new();
    let mut ids = vec![0u16; n];
    for c in 0..n {
        let r = label[c] as usize;
        if remap[r] == u16::MAX {
            remap[r] = settings.len().min(usize::from(u16::MAX - 1)) as u16;
            settings.push(set[c]);
            counts.push(0);
        }
        ids[c] = remap[r];
        counts[usize::from(remap[r])] += 1;
    }
    (ids, settings, counts)
}

pub fn bake(
    world: &WorldState,
    p: &PlanetPreset,
    s: &SurfaceState,
    out: &mut Out<'_>,
) -> Result<Value, CoreError> {
    let (l1, _l2) = grids(p)?;
    let mut written = Vec::new();
    let cell = |f: &crate::world::Field, i: usize, j: usize| {
        nearest(f, l1.block_x(i as i64) as f64, l1.block_z(j as i64) as f64)
    };
    let (w, h) = (l1.w as usize, l1.h as usize);
    let sample = |name: &str| -> Option<Vec<f32>> {
        field(world, name).map(|f| {
            (0..w * h)
                .map(|c| cell(f, c % w, c / w))
                .collect::<Vec<f32>>()
        })
    };
    if let Some(v) = sample("plates/plate_id") {
        let raw: Vec<u16> = v
            .iter()
            .map(|&x| {
                if x.is_nan() {
                    u16::MAX
                } else {
                    (x.round() as i64).clamp(0, 65_534) as u16
                }
            })
            .collect();
        out.layer(
            desc(
                "meta/plate_id",
                DType::U16,
                &l1,
                1.0,
                "plate id",
                Some(65_535.0),
            ),
            &raw,
        )?;
        written.push("meta/plate_id");
    }
    if let Some(v) = sample("crust_age") {
        let raw: Vec<u16> = v
            .iter()
            .map(|&x| {
                if x.is_nan() || x < 0.0 {
                    u16::MAX
                } else {
                    (f64::from(x).round() as i64).clamp(0, 65_534) as u16
                }
            })
            .collect();
        out.layer(
            desc(
                "meta/crust_age_myr",
                DType::U16,
                &l1,
                1.0,
                "Myr",
                Some(65_535.0),
            ),
            &raw,
        )?;
        written.push("meta/crust_age_myr");
    }
    let mut note = json!({ "layers": written, "grid": l1 });
    if let Some(reg) = sample("regime") {
        // The deep-time surface's height above the sea, on L1.
        let sea = s.sea_level_m;
        let surf = crate::world::Field {
            grid: s.grid.clone(),
            data: s.elevation_m.clone(),
        };
        let set: Vec<u8> = (0..w * h)
            .map(|c| setting(reg[c], f64::from(cell(&surf, c % w, c / w)) - sea))
            .collect();
        let cell_km2 = (f64::from(l1.r) * p.scale.m_per_block_h / 1000.0).powi(2);
        let min_cells = (MIN_PROVINCE_KM2 / cell_km2).ceil().max(1.0) as usize;
        let (ids, settings, counts) = provinces(&set, w, h, min_cells);
        out.layer(
            desc("meta/province", DType::U16, &l1, 1.0, "province id", None),
            &ids,
        )?;
        note["layers"]
            .as_array_mut()
            .expect("an array")
            .push(json!("meta/province"));
        note["provinces"] = json!({
            "count": settings.len(),
            "min_km2": MIN_PROVINCE_KM2,
            "settings": SETTINGS,
            "rule": "connected regions of one setting (regime + land/sea + highland), those under min_km2 absorbed by the nearest larger one; ids by first cell, row-major",
            "province": settings.iter().zip(&counts).enumerate().map(|(id, (&st, &n))| json!({
                "id": id, "setting": SETTINGS[usize::from(st)], "cells": n, "km2": (n as f64 * cell_km2).round(),
            })).collect::<Vec<_>>(),
        });
    }
    Ok(note)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_regions_join_their_neighbours_and_ids_are_contiguous() {
        // 8 × 4: an ocean (0) with a 2-cell island (1) and a large craton (6) across the seam.
        let (w, h) = (8, 4);
        let mut set = vec![0u8; w * h];
        for j in 0..h {
            for i in [0usize, 1, 7] {
                set[j * w + i] = 6;
            }
        }
        set[w + 4] = 1;
        set[2 * w + 4] = 1;
        let (ids, settings, counts) = provinces(&set, w, h, 5);
        assert_eq!(
            settings,
            vec![6, 0],
            "the craton wraps into one province; the island joins the ocean"
        );
        assert_eq!(counts.iter().sum::<usize>(), w * h);
        assert_eq!(ids[w + 4], ids[3]);
        assert_eq!(ids[0], ids[7]);
        // Nothing large enough: one province.
        let (ids, settings, _) = provinces(&set, w, h, 1000);
        assert!(ids.iter().all(|&i| i == 0));
        assert_eq!(settings.len(), 1);
    }
}

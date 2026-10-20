// SPDX-License-Identifier: LGPL-3.0-or-later
//! The meta part of the bake, on L1 (plan §5.3): `meta/plate_id` (u16), `meta/crust_type` (u8,
//! the plate model's crust codes `plates::state::OCEANIC` 0 and `CONTINENTAL` 1; nodata 255 where
//! it has none), `meta/crust_age_myr` (u16, Myr; nodata 65535 on continental crust, whose
//! formation age the plate model does not track), `meta/heat_flow_mw_m2` (u16, 0.1 mW/m²; the
//! plate model's continental heat flow, nodata on oceanic crust: the game's geothermal fields,
//! GEO-055) and `meta/province` (u16), for the viewer, "Why here?", the column inspectors and
//! legends; `province` also drives TerraBlender's regions (plan §7.3, P1-10).
//!
//! **Provinces** (P1-10). Every L1 cell gets its USGS class from [`crate::provinces`] (shield,
//! platform, orogen, basin, large igneous province, extended crust, oceanic crust in three age
//! classes; the preset's `provinces` section sets the thresholds). A province is a connected
//! region of one class (4-neighbours, x wraps); regions smaller than the section's `min_area_km2`
//! (50,000 km² by default) are absorbed by the nearest larger one (a breadth-first flood from the
//! larger regions), so provinces are contiguous and at least that large. Ids count from 0 in the
//! order of each province's first cell (row-major). Before P1-10 the classes were the plate
//! model's regime settings, which could not tell a shield from a basin.

use std::collections::VecDeque;

use deeptime_format::DType;
use serde_json::{Value, json};

use super::{Out, desc, field, grids, nearest};
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::provinces::{self, CLASSES};
use crate::surface::SurfaceState;
use crate::world::WorldState;

/// The class code of a cell the plate model has nothing for.
pub const UNCLASSIFIED: u8 = CLASSES.len() as u8; // ledger: PRV-031 (game-choice)

/// The layer holding the plate model's continental heat flow (L1, u16).
pub const HEAT_FLOW_LAYER: &str = "meta/heat_flow_mw_m2"; // ledger: GEO-055
/// Its quantum, mW/m² per count (a value is `raw × scale`; the plate model's largest continental
/// value is 200, far under the u16 range at 0.1).
pub const HEAT_FLOW_SCALE: f64 = 0.1;
/// Its nodata: oceanic crust, and a cell the plate model has nothing for.
pub const HEAT_FLOW_NODATA: u16 = u16::MAX;

/// The layer holding the plate model's crust type (L1, u8).
pub const CRUST_TYPE_LAYER: &str = "meta/crust_type";
/// Its nodata: a cell the plate model has nothing for (or a code outside u8).
pub const CRUST_TYPE_NODATA: u8 = u8::MAX;

/// The stored code of a cell's crust type `t` (the `plates/crust_type` field, whose values are
/// `plates::state`'s codes): nodata where the field has none.
fn crust_type_raw(t: f32) -> u8 {
    // NaN and anything that would round to the nodata code fall outside.
    if (0.0..f32::from(CRUST_TYPE_NODATA) - 0.5).contains(&t) {
        t.round() as u8
    } else {
        CRUST_TYPE_NODATA
    }
}

/// The stored count of a cell's continental heat flow `q` (mW/m²): nodata where the plate model
/// has none (oceanic crust, a gap).
fn heat_flow_raw(q: f32) -> u16 {
    if q.is_nan() || q < 0.0 {
        return HEAT_FLOW_NODATA;
    }
    ((f64::from(q) / HEAT_FLOW_SCALE).round() as i64).clamp(0, i64::from(HEAT_FLOW_NODATA) - 1)
        as u16
}

/// The name of a class code.
pub fn class_name(code: u8) -> &'static str {
    CLASSES
        .get(usize::from(code))
        .copied()
        .unwrap_or("Unclassified")
}

/// Province ids per cell (row-major, `w` × `h`, x wraps) of the classes `set`, with regions under
/// `min_cells` absorbed; returns (ids, class of each province, cells of each province).
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
    // Connected regions of one class.
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
        return (
            vec![0; n],
            vec![set.first().copied().unwrap_or(UNCLASSIFIED)],
            vec![n],
        );
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
    let mut classes = Vec::new();
    let mut counts: Vec<usize> = Vec::new();
    let mut ids = vec![0u16; n];
    for c in 0..n {
        let r = label[c] as usize;
        if remap[r] == u16::MAX {
            remap[r] = classes.len().min(usize::from(u16::MAX - 1)) as u16;
            classes.push(set[c]);
            counts.push(0);
        }
        ids[c] = remap[r];
        counts[usize::from(remap[r])] += 1;
    }
    (ids, classes, counts)
}

pub fn bake(
    world: &WorldState,
    p: &PlanetPreset,
    _s: &SurfaceState,
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
    // The crust type beside its age (the inspectors read both): the plate model's own codes, so a
    // reader names them with `plates::state` (FORMAT.md, "meta layers").
    if let Some(v) = sample("plates/crust_type") {
        let raw: Vec<u8> = v.iter().map(|&t| crust_type_raw(t)).collect();
        out.layer(
            desc(
                CRUST_TYPE_LAYER,
                DType::U8,
                &l1,
                1.0,
                "crust type (0 oceanic, 1 continental)",
                Some(f64::from(CRUST_TYPE_NODATA)),
            ),
            &raw,
        )?;
        written.push(CRUST_TYPE_LAYER);
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
    // The plate model's surface heat flow on continental crust (Ben, decisions log 2026-10-02,
    // "Export heat flow"): the game puts geothermal fields where it is high. The L1 field is
    // interpolated over the continental corners only and is NaN on oceanic crust, whose heat flow
    // is the sea floor's age law: a coastal land column must not read the young ridge crust
    // beside it through the raster's interpolation.
    if let Some(q) = sample("plates/continental_heat_flow_mw_m2") {
        let raw: Vec<u16> = q.iter().map(|&q| heat_flow_raw(q)).collect();
        out.layer(
            desc(
                HEAT_FLOW_LAYER,
                DType::U16,
                &l1,
                HEAT_FLOW_SCALE,
                "mW/m2",
                Some(f64::from(HEAT_FLOW_NODATA)),
            ),
            &raw,
        )?;
        written.push(HEAT_FLOW_LAYER);
    }
    let mut note = json!({ "layers": written, "grid": l1 });
    let params = p.provinces.unwrap_or_default();
    if let Some(classes) = provinces::derive_with(world, &params.rules()) {
        let set: Vec<u8> = (0..w * h)
            .map(|c| {
                let v = cell(&classes, c % w, c / w);
                if v.is_finite() { v as u8 } else { UNCLASSIFIED }
            })
            .collect();
        let cell_km2 = (f64::from(l1.r) * p.scale.m_per_block_h / 1000.0).powi(2);
        let min_cells = (params.min_area_km2 / cell_km2).ceil().max(1.0) as usize;
        let (ids, classes, counts) = provinces(&set, w, h, min_cells);
        out.layer(
            desc("meta/province", DType::U16, &l1, 1.0, "province id", None),
            &ids,
        )?;
        note["layers"]
            .as_array_mut()
            .expect("an array")
            .push(json!("meta/province"));
        note["provinces"] = json!({
            "count": classes.len(),
            "min_km2": params.min_area_km2,
            "params": params,
            "classes": CLASSES,
            "rule": "the USGS-style class of each cell (crate::provinces: shield, platform, orogen, basin, large igneous province, extended crust, oceanic crust by age), cut into connected regions of one class; regions under min_km2 are absorbed by the nearest larger one; ids by first cell, row-major",
            "shares": shares(&ids, &classes, &set, w, h, &l1.row_weights()),
            "shares_note": "`classified`: each class's share of the cells as the USGS-style rules classed them (the figure to set against Earth's: orogens and arcs about 0.17 of the continents); `provinces`: after regions under min_km2 were absorbed by larger neighbours, which thin belts (orogens, the young sea-floor bands) suffer most",
            "province": classes.iter().zip(&counts).enumerate().map(|(id, (&st, &n))| json!({
                "id": id, "class": class_name(st), "cells": n, "km2": (n as f64 * cell_km2).round(),
            })).collect::<Vec<_>>(),
        });
    }
    Ok(note)
}

/// The share of each class, by true area (rows weighted by `row_weight`, `GridDesc::row_weights`), of the
/// planet and of its continents (the classes up to extended crust), as the rules classed the cells
/// (`classified`: the numbers that tell how orogen-heavy the world is; Earth: orogens and arcs
/// about 0.17 of the continents) and as the baked provinces hold them (`provinces`, after the
/// small regions were absorbed), and how many cells changed class by it.
fn shares(
    ids: &[u16],
    province_class: &[u8],
    raw: &[u8],
    w: usize,
    h: usize,
    row_weight: &[f64],
) -> Value {
    let by_area = |class_of: &dyn Fn(usize) -> u8| -> Value {
        let mut area = vec![0.0f64; CLASSES.len() + 1];
        let mut total = 0.0;
        for j in 0..h {
            let wgt = row_weight[j];
            for i in 0..w {
                area[usize::from(class_of(j * w + i))] += wgt;
                total += wgt;
            }
        }
        let continental: f64 = area[..=usize::from(provinces::EXTENDED)].iter().sum();
        let mut planet = serde_json::Map::new();
        let mut cont = serde_json::Map::new();
        let round = |x: f64| (x * 1e4).round() / 1e4;
        for (k, a) in area.iter().enumerate() {
            planet.insert(
                class_name(k as u8).into(),
                json!(round(a / total.max(1e-12))),
            );
            if k <= usize::from(provinces::EXTENDED) {
                cont.insert(
                    class_name(k as u8).into(),
                    json!(round(a / continental.max(1e-12))),
                );
            }
        }
        json!({ "planet": planet, "continental": cont })
    };
    let changed = (0..w * h)
        .filter(|&c| province_class[usize::from(ids[c])] != raw[c])
        .count();
    json!({
        "classified": by_area(&|c| raw[c]),
        "provinces": by_area(&|c| province_class[usize::from(ids[c])]),
        "absorbed_cells": changed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provinces::ProvinceParams;

    /// The row weights of an `h`-row grid that spans the planet pole to pole: on the equal-area
    /// projection every row weighs the same (`GridDesc::row_weights`, inside the poles).
    fn band_weights(h: usize) -> Vec<f64> {
        let g = deeptime_format::GridDesc::for_planet("L1", 16_384, 16).unwrap();
        let inside: Vec<f64> = g.row_weights().into_iter().filter(|&w| w == 1.0).collect();
        assert!(inside.len() >= h);
        inside[..h].to_vec()
    }

    #[test]
    fn crust_type_codes_are_the_plate_models_and_gaps_are_nodata() {
        use crate::plates::state::{CONTINENTAL, OCEANIC};
        assert_eq!(crust_type_raw(f32::from(OCEANIC)), OCEANIC);
        assert_eq!(crust_type_raw(f32::from(CONTINENTAL)), CONTINENTAL);
        // A nearest sample of the f32 field is a whole code; a stray fraction rounds.
        assert_eq!(crust_type_raw(0.9999), CONTINENTAL);
        for gap in [f32::NAN, f32::INFINITY, -1.0, 255.0, 1.0e6] {
            assert_eq!(crust_type_raw(gap), CRUST_TYPE_NODATA, "{gap}");
        }
    }

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

    #[test]
    fn two_regions_of_one_class_are_two_provinces() {
        // 8 × 2: two shields (0) apart in an old ocean (8): same class, different provinces.
        let (w, h) = (8, 2);
        let mut set = vec![8u8; w * h];
        for j in 0..h {
            set[j * w + 1] = 0;
            set[j * w + 5] = 0;
        }
        let (ids, classes, counts) = provinces(&set, w, h, 2);
        assert_eq!(classes.iter().filter(|&&c| c == 0).count(), 2);
        assert_ne!(ids[1], ids[5]);
        assert_eq!(counts.iter().sum::<usize>(), w * h);
    }

    #[test]
    fn shares_are_by_area_and_continental_shares_add_to_one() {
        let (w, h) = (4, 4);
        // The top half shield (0), the bottom oldest ocean (8).
        let raw: Vec<u8> = (0..w * h).map(|c| if c < 8 { 0 } else { 8 }).collect();
        let (ids, classes, _) = provinces(&raw, w, h, 1);
        let s = shares(&ids, &classes, &raw, w, h, &band_weights(h));
        for view in ["classified", "provinces"] {
            let planet: f64 = s[view]["planet"]
                .as_object()
                .unwrap()
                .values()
                .map(|v| v.as_f64().unwrap())
                .sum();
            assert!(
                (planet - 1.0).abs() < 1e-3,
                "{view}: planet shares add to 1, got {planet}"
            );
            let cont = s[view]["continental"].as_object().unwrap();
            assert!((cont["Shield"].as_f64().unwrap() - 1.0).abs() < 1e-9);
        }
        assert_eq!(s["absorbed_cells"], 0);
        // A one-cell orogen in a shield is absorbed: classed (a share), not kept (no province).
        let mut raw2 = raw.clone();
        raw2[5] = 2;
        let (ids, classes, _) = provinces(&raw2, w, h, 2);
        let s = shares(&ids, &classes, &raw2, w, h, &band_weights(h));
        assert_eq!(s["absorbed_cells"], 1);
        assert!(s["classified"]["continental"]["Orogen"].as_f64().unwrap() > 0.0);
        assert_eq!(s["provinces"]["continental"]["Orogen"], 0.0);
        assert!(ProvinceParams::default().validate().is_ok());
    }
}

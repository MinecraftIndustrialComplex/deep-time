// SPDX-License-Identifier: LGPL-3.0-or-later
//! Spawn candidates ranked by the essentials (plan §7.8, decisions file "Playability overrides": "a
//! spawn chosen for its essentials"). The core ranks the candidates and stores the top ten in
//! `spawn/candidates.json`; the game takes the first one that is dry land once generated
//! (`SpawnHandler`).
//!
//! What a candidate is today: dry land at least two blocks above sea level, within one block of
//! its four L3 neighbours, on a 16-block lattice. The plan's other terms (temperate Köppen C or
//! mild D, fresh water within reach) need the climate and hydrology stages (P1-05, Phase 2); a
//! world without them is ranked on what exists. Ranking: fewest essentials out of reach, then the
//! smallest Σ min(d / max_blocks_from_spawn, 4) over the essentials that set a reach, then the
//! nearest to (0, 0); picks are at least 64 blocks apart.

use deeptime_format::{RasterRead, Site};
use rayon::prelude::*;
use serde_json::{Value, json};

use super::playability::Essential;

/// How many candidates are stored.
pub const TOP: usize = 10;
/// Blocks between lattice points.
pub const LATTICE_BLOCKS: i64 = 16;
/// Minimum distance between two stored candidates.
pub const SPACING_BLOCKS: i64 = 64;

/// East–west distance on a planet of circumference `c` (x wraps).
fn wrapped_dx(c: f64, dx: f64) -> f64 {
    let d = dx.rem_euclid(c);
    d.min(c - d)
}

struct Reach<'a> {
    material: &'a str,
    max: f64,
    /// (x, z, site id) of every site with the material among its products.
    sites: Vec<(f64, f64, u32)>,
}

impl Reach<'_> {
    /// (distance, site id) of the nearest site, or (∞, none).
    fn nearest(&self, c: f64, x: f64, z: f64) -> (f64, Option<u32>) {
        let mut best = (f64::INFINITY, None);
        for &(sx, sz, id) in &self.sites {
            let dx = wrapped_dx(c, sx - x);
            let d = (dx * dx + (sz - z) * (sz - z)).sqrt();
            if d < best.0 {
                best = (d, Some(id));
            }
        }
        best
    }
}

/// A lattice point that passed the land test, with its ranking keys.
struct Candidate {
    unmet: u32,
    score: f64,
    dist2: i64,
    x: i64,
    z: i64,
    i: i64,
    j: i64,
    surface: f64,
}

/// `spawn/candidates.json` ranked by the essentials that set `max_blocks_from_spawn`, or `None`
/// when none does (the caller keeps its unranked candidates).
pub fn ranked_candidates(
    elevation: &(impl RasterRead + Sync),
    water: &(impl RasterRead + Sync),
    sea_level: i32,
    sites: &[Site],
    essentials: &[Essential],
) -> Option<Value> {
    let g = &elevation.desc().grid;
    let c = g.circumference() as f64;
    let reaches: Vec<Reach> = essentials
        .iter()
        .filter_map(|e| {
            e.max_blocks_from_spawn.map(|max| Reach {
                material: &e.material,
                max: f64::from(max.max(1)),
                sites: sites
                    .iter()
                    .filter(|s| s.products.iter().any(|p| p.material == e.material))
                    .map(|s| (f64::from(s.center[0]), f64::from(s.center[2]), s.id))
                    .collect(),
            })
        })
        .collect();
    if reaches.is_empty() {
        return None;
    }
    let stride = (LATTICE_BLOCKS / i64::from(g.r)).max(1);
    let (w, h) = (i64::from(g.w), i64::from(g.h));
    let sea = f64::from(sea_level + 2);
    let rows: Vec<i64> = (1..h - 1).step_by(stride as usize).collect();
    let mut found: Vec<Candidate> = rows
        .into_par_iter()
        .flat_map_iter(|j| {
            let reaches = &reaches;
            (0..w).step_by(stride as usize).filter_map(move |i| {
                let e = elevation.get(i, j);
                if e.is_nan() || e < sea || !water.get(i, j).is_nan() {
                    return None;
                }
                let flat = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .all(|&(di, dj)| (elevation.get(i + di, j + dj) - e).abs() <= 1.0);
                if !flat {
                    return None;
                }
                let (x, z) = (g.block_x(i), g.block_z(j));
                let (mut unmet, mut score) = (0u32, 0.0);
                for r in reaches {
                    let (d, _) = r.nearest(c, x as f64, z as f64);
                    if d > r.max {
                        unmet += 1;
                    }
                    score += (d / r.max).min(4.0);
                }
                Some(Candidate {
                    unmet,
                    score,
                    dist2: x * x + z * z,
                    x,
                    z,
                    i,
                    j,
                    surface: e,
                })
            })
        })
        .collect();
    found.sort_by(|a, b| {
        a.unmet
            .cmp(&b.unmet)
            .then(a.score.total_cmp(&b.score))
            .then(a.dist2.cmp(&b.dist2))
            .then((a.x, a.z).cmp(&(b.x, b.z)))
    });
    let mut picked: Vec<Value> = Vec::new();
    let mut taken: Vec<(i64, i64)> = Vec::new();
    for Candidate {
        unmet,
        score,
        x,
        z,
        i,
        j,
        surface: e,
        ..
    } in found
    {
        let far = taken.iter().all(|&(tx, tz)| {
            let dx = wrapped_dx(c, (x - tx) as f64);
            dx * dx + ((z - tz) * (z - tz)) as f64 >= (SPACING_BLOCKS * SPACING_BLOCKS) as f64
        });
        if !far {
            continue;
        }
        taken.push((x, z));
        let ess: Vec<Value> = reaches
            .iter()
            .map(|r| {
                let (d, id) = r.nearest(c, x as f64, z as f64);
                json!({
                    "material": r.material,
                    "site": id,
                    "distance": if d.is_finite() { json!((d * 10.0).round() / 10.0) } else { Value::Null },
                    "max_blocks_from_spawn": r.max,
                })
            })
            .collect();
        picked.push(json!({
            "x": x, "z": z, "surface_y": e, "cell": [i, j],
            "unmet": unmet, "score": (score * 1000.0).round() / 1000.0,
            "essentials": ess,
        }));
        if picked.len() == TOP {
            break;
        }
    }
    Some(json!({
        "rule": "L3 sample points on a 16-block lattice: elevation ≥ sea level + 2, no water, within 1 block of each L3 neighbour; ranked by the number of essentials whose nearest site is farther than max_blocks_from_spawn, then Σ min(d / max, 4), then distance from (0, 0); at least 64 blocks apart, at most 10 (plan §7.8)",
        "ranked_by": reaches.iter().map(|r| json!({"material": r.material, "max_blocks_from_spawn": r.max, "sites": r.sites.len()})).collect::<Vec<_>>(),
        "candidates": picked,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::{Codec, SiteKind};

    fn site(id: u32, x: i32, z: i32, material: &str) -> Site {
        Site {
            id,
            kind: SiteKind::Deposit,
            deposit_type: "deeptime:test".into(),
            center: [x, 0, z],
            lat_lon: [0.0, 0.0],
            aabb: [[x - 2, -5, z - 2], [x + 2, 5, z + 2]],
            body: None,
            products: vec![deeptime_format::Product {
                material: material.into(),
                grade: 0.01,
                tonnage_t: 1.0e6,
                role: "primary".into(),
            }],
            forming_event: deeptime_format::EventId(1),
            parent: None,
            extraction: Default::default(),
        }
    }

    #[test]
    fn candidates_near_every_essential_come_first() {
        // A flat land planet (wrap-test is hilly; build a flat one through the lake test's
        // plain far from the basin).
        let w = crate::synth::build(
            crate::synth::Kind::LakeTest,
            crate::synth::Strata::Placeholder,
            1024,
            Codec::Raw,
        )
        .unwrap();
        let sites = vec![
            site(1, 300, 100, "c:iron"),
            site(2, 320, 120, "c:copper"),
            site(3, -300, -100, "c:iron"),
        ];
        let ess = |m: &str, max: Option<u32>| Essential {
            material: m.into(),
            min_sites_per_region: 1,
            region_blocks: 1024,
            exposed: false,
            max_blocks_from_spawn: max,
        };
        assert!(
            ranked_candidates(
                &w.layers[0],
                &w.layers[1],
                63,
                &sites,
                &[ess("c:iron", None)]
            )
            .is_none()
        );
        let v = ranked_candidates(
            &w.layers[0],
            &w.layers[1],
            63,
            &sites,
            &[ess("c:iron", Some(100)), ess("c:copper", Some(100))],
        )
        .unwrap();
        let cands = v["candidates"].as_array().unwrap();
        assert_eq!(cands.len(), TOP);
        let first = &cands[0];
        assert_eq!(first["unmet"], 0, "{first}");
        let (x, z) = (first["x"].as_i64().unwrap(), first["z"].as_i64().unwrap());
        assert!((x - 310).abs() < 100 && (z - 110).abs() < 100, "{first}");
        for e in first["essentials"].as_array().unwrap() {
            assert!(e["distance"].as_f64().unwrap() <= 100.0, "{e}");
        }
        // Stored candidates are spaced apart.
        for a in cands {
            for b in cands {
                if a == b {
                    continue;
                }
                let dx = (a["x"].as_i64().unwrap() - b["x"].as_i64().unwrap()) as f64;
                let dz = (a["z"].as_i64().unwrap() - b["z"].as_i64().unwrap()) as f64;
                assert!(wrapped_dx(1024.0, dx).hypot(dz) >= 64.0);
            }
        }
    }
}

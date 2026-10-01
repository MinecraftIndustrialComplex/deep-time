// SPDX-License-Identifier: LGPL-3.0-or-later
//! Structural closure for the petroleum systems (`deeptime:trap_structure`, 0–1; plan §8.6 item
//! 7: "traps from structure (anticlines, salt domes) and seals"), per L1 cell at each frame, the
//! largest of:
//!
//! - **The world's own folds and faults** (P2-01, [`structural`]): where a fold train, a thrust,
//!   a normal fault or a strike-slip fault lies over the cell and it holds at least
//!   `trap_min_sediment_m` of strata: min(1, sediment / (4 · minimum)) · the structure's share
//!   (an anticline up to 1 by how tight it is, a thrust's hanging wall 0.8, a tilted fault block
//!   0.6, a strike-slip fault 0.3). The structures ride their plates and stay in the rock, so a
//!   belt folded 200 Myr ago still holds traps.
//! - With folds and faults turned off (`deformation.strength` 0), the settings that make them
//!   stand in, as in Phase 1: **fold-thrust belts**, continental crust within `fold_belt_km` (on
//!   the plate model's Earth-sized sphere, like every tectonic width) of an active orogen but
//!   not in it: min(1, sediment / (4 · minimum)) · (1 − distance / belt width); and **rift fault
//!   blocks** (below).
//! - **Salt domes:** a halite layer at least `salt_min_m` thick under at least
//!   `salt_overburden_m` of strata: min(1, halite / (2 · minimum)) · min(1, overburden / minimum)
//!   (salt tectonics, Hudec & Jackson 2007).
//! - **Rift fault blocks:** continental crust in the rift regime with the minimum sediment:
//!   0.6 · min(1, sediment / (2 · minimum)) (tilted fault-block traps).
//! - **Growth faults:** sediment wedges thicker than `growth_fault_sediment_m`:
//!   0.8 · min(1, (sediment − g) / g) (rollover anticlines of deltas, Doust & Omatsola 1990).
//! - **Drape:** 0.1 · min(1, sediment / 1 km) everywhere (compaction over basement relief), below
//!   any sensible trap threshold on its own.
//!
//! The deposit engine records when a cell's closure first passed its type's threshold
//! (`petroleum.rs`), so a trap formed during an orogeny stays a trap after it.

use crate::plates::state::{ARC, CONTINENTAL, regime};
use crate::surface::fields::dist_to_edge;
use crate::surface::grid::Dims;
use crate::surface::strata_log::Layer;

use super::params::StructureParams;

/// Distance (km, block metric) from every cell to the nearest active orogen (collision regime, or
/// a continental arc); 0 inside one, 3,000 km where there is none.
pub fn orogen_distance_km(d: &Dims, regime_f: &[f32], crust: &[f32], flags: &[f32]) -> Vec<f32> {
    let n = d.cells();
    let mask: Vec<f32> = (0..n)
        .map(|c| {
            let reg = regime_f[c] as u8;
            let cont = crust[c] as u8 == CONTINENTAL;
            let arc = flags[c] as u32 as u8 & ARC != 0;
            if reg == regime::COLLISION || (cont && (arc || reg == regime::ARC)) {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let dist = dist_to_edge(d, &mask);
    dist.iter()
        .zip(&mask)
        .map(|(&x, &m)| if m >= 0.5 { 0.0 } else { x })
        .collect()
}

/// The share of structural closure the world's folds and faults give each cell of `grid` (L1),
/// 0–1: the largest over the structures whose window holds the cell's centre of
/// - a fold train: how tight it is, amplitude / wavelength over 0.1 (10 % of the wavelength is a
///   fold shortened by about a tenth), at most 1;
/// - a thrust: 0.8 over its hanging wall (the ramp anticline), 0.4 over its foot wall;
/// - a normal fault: 0.6 (a tilted block against the fault);
/// - a strike-slip fault: 0.3;
///
/// each times min(1, 2 · the window's weight), so closure fades out towards a structure's edge.
pub fn structural(
    features: &[super::deform::Feature],
    grid: &deeptime_format::GridDesc,
) -> Vec<f32> {
    use super::deform::Kind;
    let mut out = vec![0.0f32; grid.cells()];
    let c = grid.circumference() as f64;
    let r = f64::from(grid.r);
    let (ox, oz) = (grid.origin_block_x as f64, grid.origin_block_z as f64);
    let w = grid.w as i64;
    for f in features {
        let share = match f.kind {
            Kind::FoldTrain => {
                (f64::from(f.amplitude) / f64::from(f.wavelength).max(1e-9) / 0.1).min(1.0)
            }
            Kind::Thrust => 0.8,
            Kind::Normal => 0.6,
            Kind::StrikeSlip => 0.3,
            Kind::Stock | Kind::Dyke | Kind::Plug => continue,
        };
        let window = f.to_baked(0.0, 0).window;
        let reach = window.reach();
        let (i0, i1) = (
            ((f.x - reach - ox) / r).floor() as i64,
            ((f.x + reach - ox) / r).ceil() as i64,
        );
        let (j0, j1) = (
            (((f.z - reach - oz) / r).floor() as i64).max(0),
            (((f.z + reach - oz) / r).ceil() as i64).min(i64::from(grid.h) - 1),
        );
        for j in j0..=j1 {
            for i in i0..=i1 {
                let (x, z) = (ox + (i as f64 + 0.5) * r, oz + (j as f64 + 0.5) * r);
                let (a, b) = window.local(x, z, c);
                let wgt = window.weight(a, b);
                if wgt <= 0.0 {
                    continue;
                }
                let side = if f.kind == Kind::Thrust && b < 0.0 {
                    0.5
                } else {
                    1.0
                };
                let v = (share * side * (2.0 * wgt).min(1.0)) as f32;
                let k = (j * w + i.rem_euclid(w)) as usize;
                if v > out[k] {
                    out[k] = v;
                }
            }
        }
    }
    out
}

/// One cell's closure: `log` bottom first, `above_m` of regolith on top. `structural` is the
/// cell's share from the world's own folds and faults ([`structural`]), `None` when they are
/// turned off; `orogen_km` the distance to the nearest active orogen on the plate model's sphere.
#[allow(clippy::too_many_arguments)]
pub fn closure(
    log: &[Layer],
    above_m: f64,
    halite: u16,
    continental: bool,
    reg: u8,
    orogen_km: f64,
    structural: Option<f32>,
    p: &StructureParams,
) -> f32 {
    let sediment: f64 = log.iter().map(|l| f64::from(l.thickness_m)).sum();
    let min = p.trap_min_sediment_m.max(1.0);
    let mut best = 0.1 * (sediment / 1000.0).min(1.0);
    match structural {
        Some(share) => {
            if sediment >= min {
                best = best.max((sediment / (4.0 * min)).min(1.0) * f64::from(share));
            }
        }
        None => {
            if continental
                && sediment >= min
                && orogen_km > 0.0
                && orogen_km < p.fold_belt_km
                && reg != regime::COLLISION
            {
                best = best
                    .max((sediment / (4.0 * min)).min(1.0) * (1.0 - orogen_km / p.fold_belt_km));
            }
            if continental && reg == regime::RIFT && sediment >= min {
                best = best.max(0.6 * (sediment / (2.0 * min)).min(1.0));
            }
        }
    }
    let g = p.growth_fault_sediment_m.max(1.0);
    if sediment > g {
        best = best.max(0.8 * ((sediment - g) / g).min(1.0));
    }
    let mut overburden = above_m;
    for l in log.iter().rev() {
        let t = f64::from(l.thickness_m);
        if l.class == halite && t >= p.salt_min_m && overburden >= p.salt_overburden_m {
            let s = (t / (2.0 * p.salt_min_m.max(1.0))).min(1.0)
                * (overburden / p.salt_overburden_m.max(1.0)).min(1.0);
            best = best.max(s);
        }
        overburden += t;
    }
    best.clamp(0.0, 1.0) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::lithology::index;

    fn layer(th: f32, class: &str) -> Layer {
        Layer {
            age_myr: -10.0,
            thickness_m: th,
            class: index(class),
            provenance: 0,
        }
    }

    #[test]
    fn closure_comes_from_folds_salt_rifts_and_deltas() {
        let p = StructureParams::default();
        let halite = index("halite");
        let basin = [layer(1500.0, "sandstone"), layer(500.0, "shale")];
        // A foreland 60 km from the orogen: folded.
        let fold = closure(&basin, 0.0, halite, true, regime::BACK_ARC, 60.0, None, &p);
        assert!((fold - 0.8).abs() < 1e-6, "{fold}");
        // The same basin on a craton far from any orogen: drape plus a small growth-fault share.
        let calm = closure(&basin, 0.0, halite, true, regime::CRATON, 3000.0, None, &p);
        // With the world's own structures: the anticline over the cell decides, wherever it is;
        // the foreland without one is calm, and thin cover holds no trap.
        let tight = closure(
            &basin,
            0.0,
            halite,
            true,
            regime::CRATON,
            3000.0,
            Some(1.0),
            &p,
        );
        assert!((tight - 1.0).abs() < 1e-6, "{tight}");
        let none = closure(
            &basin,
            0.0,
            halite,
            true,
            regime::BACK_ARC,
            60.0,
            Some(0.0),
            &p,
        );
        assert!(none < 0.3, "{none}");
        let thin = [layer(200.0, "sandstone")];
        assert!(
            closure(
                &thin,
                0.0,
                halite,
                true,
                regime::CRATON,
                3000.0,
                Some(1.0),
                &p
            ) < 0.1
        );
        assert!(calm < 0.3, "{calm}");
        // Salt 300 m thick under 1.2 km of strata: a dome.
        let salt = [
            layer(300.0, "halite"),
            layer(700.0, "sandstone"),
            layer(500.0, "shale"),
        ];
        let dome = closure(&salt, 0.0, halite, true, regime::CRATON, 3000.0, None, &p);
        assert!((dome - 1.0).abs() < 1e-6, "{dome}");
        // Salt near the surface does not rise.
        let shallow = [layer(700.0, "sandstone"), layer(300.0, "halite")];
        assert!(
            closure(
                &shallow,
                0.0,
                halite,
                true,
                regime::CRATON,
                3000.0,
                None,
                &p
            ) < 0.3
        );
        // A rift basin with 1 km of fill.
        let rift = closure(
            &[layer(1000.0, "conglomerate")],
            0.0,
            halite,
            true,
            regime::RIFT,
            3000.0,
            None,
            &p,
        );
        assert!((rift - 0.6).abs() < 1e-6);
        // A 3-km delta wedge on oceanic crust.
        let delta = closure(
            &[layer(3000.0, "shale")],
            0.0,
            halite,
            false,
            regime::OCEAN_BASIN,
            3000.0,
            None,
            &p,
        );
        assert!((delta - 0.8).abs() < 1e-6);
    }

    #[test]
    fn the_worlds_structures_give_closure_where_they_lie() {
        use crate::structure::deform::{Feature, Kind};
        let grid = deeptime_format::GridDesc::for_planet("L1", 4096, 16).unwrap();
        let base = Feature {
            id: 0,
            kind: Kind::FoldTrain,
            x: 0.0,
            z: 0.0,
            plate: 0,
            t_myr: -50.0,
            azimuth_deg: 0.0,
            half_along: 300.0,
            half_across: 120.0,
            wavelength: 60.0,
            amplitude: 9.0,
            phase: 0.0,
            dip_deg: 30.0,
            slip: -10.0,
            cause: 0,
            adrift: 0,
            wear_m: 0.0,
            half_height: 0.0,
            depth: 0.0,
            rock: 0,
        };
        let at = |v: &[f32], x: f64, z: f64| v[crate::structure::plutons::cell_of(&grid, x, z)];
        // A tight fold train: full closure at its middle, less towards its edge, none outside.
        let fold = structural(std::slice::from_ref(&base), &grid);
        assert!((at(&fold, 0.0, 0.0) - 1.0).abs() < 1e-6);
        let edge = at(&fold, 100.0, 0.0);
        assert!(edge > 0.0 && edge < 0.5, "{edge}");
        assert_eq!(at(&fold, 200.0, 0.0), 0.0);
        assert_eq!(at(&fold, 0.0, 400.0), 0.0);
        // A gentle one: half.
        let gentle = Feature {
            amplitude: 3.0,
            ..base.clone()
        };
        let v = structural(std::slice::from_ref(&gentle), &grid);
        assert!((at(&v, 0.0, 0.0) - 0.5).abs() < 1e-6);
        // A thrust (dipping east: azimuth 0, the hanging wall to the east): more over the
        // hanging wall than the foot wall. A stock gives none. The largest wins.
        let thrust = Feature {
            kind: Kind::Thrust,
            ..base.clone()
        };
        let v = structural(std::slice::from_ref(&thrust), &grid);
        assert!((at(&v, 24.0, 0.0) - 0.8).abs() < 1e-6 && (at(&v, -24.0, 0.0) - 0.4).abs() < 1e-6);
        let stock = Feature {
            kind: Kind::Stock,
            ..base.clone()
        };
        assert!(
            structural(std::slice::from_ref(&stock), &grid)
                .iter()
                .all(|&x| x == 0.0)
        );
        let both = structural(&[thrust, gentle], &grid);
        assert!((at(&both, 24.0, 0.0) - 0.8).abs() < 1e-6);
    }
}

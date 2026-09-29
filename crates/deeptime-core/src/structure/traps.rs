// SPDX-License-Identifier: LGPL-3.0-or-later
//! Structural closure for the petroleum systems (`deeptime:trap_structure`, 0–1; plan §8.6 item
//! 7: "traps from structure (anticlines, salt domes) and seals"). Phase 1 has no folds or faults
//! of its own (they are Phase 2's kinematic history, §8.5), so closure is read from the settings
//! that make traps, per L1 cell at each frame, the largest of:
//!
//! - **Fold-thrust belts:** continental crust within `fold_belt_km` of an active orogen (a cell in
//!   the collision regime, or a continental arc) but not in it, with at least
//!   `trap_min_sediment_m` of strata: min(1, sediment / (4 · minimum)) · (1 − distance / belt width)
//!   — foreland anticlines (the Zagros, the Subandean belt).
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

/// One cell's closure: `log` bottom first, `above_m` of regolith on top.
#[allow(clippy::too_many_arguments)]
pub fn closure(
    log: &[Layer],
    above_m: f64,
    halite: u16,
    continental: bool,
    reg: u8,
    orogen_km: f64,
    p: &StructureParams,
) -> f32 {
    let sediment: f64 = log.iter().map(|l| f64::from(l.thickness_m)).sum();
    let min = p.trap_min_sediment_m.max(1.0);
    let mut best = 0.1 * (sediment / 1000.0).min(1.0);
    if continental
        && sediment >= min
        && orogen_km > 0.0
        && orogen_km < p.fold_belt_km
        && reg != regime::COLLISION
    {
        best = best.max((sediment / (4.0 * min)).min(1.0) * (1.0 - orogen_km / p.fold_belt_km));
    }
    if continental && reg == regime::RIFT && sediment >= min {
        best = best.max(0.6 * (sediment / (2.0 * min)).min(1.0));
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
        let fold = closure(&basin, 0.0, halite, true, regime::BACK_ARC, 60.0, &p);
        assert!((fold - 0.8).abs() < 1e-6, "{fold}");
        // The same basin on a craton far from any orogen: drape plus a small growth-fault share.
        let calm = closure(&basin, 0.0, halite, true, regime::CRATON, 3000.0, &p);
        assert!(calm < 0.3, "{calm}");
        // Salt 300 m thick under 1.2 km of strata: a dome.
        let salt = [
            layer(300.0, "halite"),
            layer(700.0, "sandstone"),
            layer(500.0, "shale"),
        ];
        let dome = closure(&salt, 0.0, halite, true, regime::CRATON, 3000.0, &p);
        assert!((dome - 1.0).abs() < 1e-6, "{dome}");
        // Salt near the surface does not rise.
        let shallow = [layer(700.0, "sandstone"), layer(300.0, "halite")];
        assert!(closure(&shallow, 0.0, halite, true, regime::CRATON, 3000.0, &p) < 0.3);
        // A rift basin with 1 km of fill.
        let rift = closure(
            &[layer(1000.0, "conglomerate")],
            0.0,
            halite,
            true,
            regime::RIFT,
            3000.0,
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
            &p,
        );
        assert!((delta - 0.8).abs() < 1e-6);
    }
}

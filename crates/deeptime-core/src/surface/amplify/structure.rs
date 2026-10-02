// SPDX-License-Identifier: LGPL-3.0-or-later
//! Fault scarps, rift valleys and fold ridges (P2-01; decisions log 2026-10-01, "Terrain too"):
//! the ground's share of the world's young structures.
//!
//! **Where in the pipeline, and why.** The deep-time surface (L1, 16 blocks a cell) cannot hold a
//! scarp, and changing it would move the calibrated hypsometry and the uplift work that reads it.
//! The game's terrain is the amplifier's, so the term is added there, on the young landscape's
//! grid (L2) *before* the amplifier's erosion passes, as the volcanoes' cones are (`volcano.rs`):
//! the stream-power, thermal and deposition passes then gully the scarps, slump them to the rock's
//! angle and floor the grabens, the flood makes every cell drain, and the bake's rivers and lakes
//! route on the result. One term, one switch (`structure.deformation.relief`, 0 = off), and
//! nothing downstream needs to know: the bake writes the amplifier's `elevation` as before.
//!
//! **What is added.** The surface moves as the rock under it did (`deeptime_format::structure`),
//! scaled by how much of each structure's relief is left:
//! - a **fault**: its hanging wall, the ground on the dip side of the trace, by the vertical part
//!   of `weight · slip` (down for a normal fault: with a facing fault, a graben; up for a thrust),
//!   and its foot wall the other way by `footwall_share` of that (a rift's shoulders, a thrust's
//!   foredeep). The step at the trace is the scarp.
//! - a **fold train**: `fold_relief` of its vertical shift (anticlines stand as ridges while
//!   they are young).
//!
//! **How much is left** is the structure stage's (`structure::deform::relief_left`): 1 for a
//! structure made this step, falling to 0 over `relief_age_myr` and as the deep-time surface
//! erodes or buries the ground there by `relief_wear` times the structure's own height. Old
//! faults keep their offset in the rock and none on the ground.
//!
//! **Land stays land.** Only land cells outside lakes change, and none is lowered below a metre
//! above the sea, so the land fraction is untouched; the notes carry the median before and after.

use deeptime_format::GridDesc;
use deeptime_format::structure::{Shape, StructEvent, profile_at};
use serde::Serialize;

use super::resample::Level;
use crate::params::PlanetPreset;
use crate::world::WorldState;

/// The structures that still show on the ground.
#[derive(Clone, Debug, Default)]
pub struct Relief {
    /// (event, the share of its relief left × the setting).
    items: Vec<(StructEvent, f64)>,
    profiles: Vec<Vec<f32>>,
    fold_share: f64,
    footwall_share: f64,
    /// Metres per block, vertical.
    mv: f64,
    circumference: f64,
}

/// What the term did to the young landscape.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ReliefStats {
    /// Structures with relief left.
    pub structures: usize,
    pub cells_changed: u64,
    pub mean_abs_m: f64,
    pub max_up_m: f64,
    pub max_down_m: f64,
    /// Land cells (unchanged by construction) and their median height above the sea, m.
    pub land_cells: u64,
    pub land_median_before_m: f64,
    pub land_median_after_m: f64,
    pub land_mean_before_m: f64,
    pub land_mean_after_m: f64,
}

impl Relief {
    /// The world's structures with relief left; `None` when there are none or the setting is 0.
    pub fn from_world(world: &WorldState, p: &PlanetPreset) -> Option<Relief> {
        let st = world.structure.as_ref()?;
        let d = p.structure.clone().unwrap_or_default().deformation;
        if d.relief <= 0.0 || world.structure_relief.len() != st.events.len() {
            return None;
        }
        let items: Vec<(StructEvent, f64)> = st
            .events
            .iter()
            .zip(&world.structure_relief)
            .filter(|(_, r)| **r > 0.01) // ledger: DEF-061
            .map(|(e, r)| (*e, f64::from(*r)))
            .collect();
        if items.is_empty() {
            return None;
        }
        Some(Relief {
            items,
            profiles: st.profiles.clone(),
            fold_share: d.fold_relief,
            footwall_share: d.footwall_share,
            mv: p.scale.m_per_block_v,
            circumference: f64::from(p.scale.circumference_blocks),
        })
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// How far one structure with all its relief moved the ground at block position (x, z),
    /// blocks (up positive).
    pub fn shift_blocks(&self, ev: &StructEvent, x: f64, z: f64) -> f64 {
        let (a, b) = ev.window.local(x, z, self.circumference);
        let w = ev.window.weight(a, b);
        if w == 0.0 {
            return 0.0;
        }
        match ev.shape {
            Shape::Fold {
                wavelength,
                amplitude,
                offset,
                profile,
            } => {
                let phase = (b - offset) * deeptime_format::structure::NODDY_TAU / wavelength;
                let v = profile_at(&self.profiles[usize::from(profile)], phase);
                self.fold_share * w * amplitude * v
            }
            Shape::Fault { slip, .. } => {
                // The trace lies on the ground; the dip side of it is the hanging wall.
                if b > 0.0 {
                    w * slip[1]
                } else {
                    -self.footwall_share * w * slip[1]
                }
            }
            Shape::Intrusion { .. } => 0.0,
        }
    }

    /// Adds the term to the ladder's first level (the L2 terrain, metres).
    pub fn apply(&self, level: &mut Level, grid: &GridDesc, sea_m: f64) -> ReliefStats {
        let n = level.z.len();
        let mut delta = vec![0.0f32; n];
        let r = f64::from(grid.r);
        let (ox, oz) = (grid.origin_block_x as f64, grid.origin_block_z as f64);
        let w = grid.w as i64;
        for (ev, left) in &self.items {
            let reach = ev.window.reach();
            let [cx, cz] = ev.window.centre;
            let (i0, i1) = (
                ((cx - reach - ox) / r).floor() as i64,
                ((cx + reach - ox) / r).ceil() as i64,
            );
            let (j0, j1) = (
                (((cz - reach - oz) / r).floor() as i64).max(0),
                (((cz + reach - oz) / r).ceil() as i64).min(i64::from(grid.h) - 1),
            );
            for j in j0..=j1 {
                let bz = oz + j as f64 * r;
                for i in i0..=i1 {
                    let bx = ox + i as f64 * r;
                    let v = self.shift_blocks(ev, bx, bz);
                    if v != 0.0 {
                        let c = (j * w + i.rem_euclid(w)) as usize;
                        delta[c] += (left * v * self.mv) as f32;
                    }
                }
            }
        }
        let sea = sea_m as f32;
        let floor = sea + 1.0; // ledger: DEF-061
        let mut stats = ReliefStats {
            structures: self.items.len(),
            ..ReliefStats::default()
        };
        let mut before: Vec<f32> = Vec::new();
        let mut after: Vec<f32> = Vec::new();
        let mut sum_abs = 0.0f64;
        for (c, &dz) in delta.iter().enumerate() {
            let z0 = level.z[c];
            if z0.is_nan() || z0 <= sea {
                continue;
            }
            let mut z1 = z0;
            if dz != 0.0 && level.lake.get(c).copied().unwrap_or(0) == 0 {
                z1 = (z0 + dz).max(floor.min(z0));
                if z1 != z0 {
                    let d = f64::from(z1 - z0);
                    stats.cells_changed += 1;
                    sum_abs += d.abs();
                    stats.max_up_m = stats.max_up_m.max(d);
                    stats.max_down_m = stats.max_down_m.max(-d);
                    level.z[c] = z1;
                    if !level.zs.is_empty() && level.zs[c] <= z0 {
                        // No glacier here: its "surface" stays a metre under the ground.
                        level.zs[c] = z1 - 1.0;
                    }
                }
            }
            before.push(z0 - sea);
            after.push(z1 - sea);
        }
        stats.land_cells = before.len() as u64;
        if stats.cells_changed > 0 {
            stats.mean_abs_m = sum_abs / stats.cells_changed as f64;
        }
        let mean = |v: &[f32]| v.iter().map(|&x| f64::from(x)).sum::<f64>() / v.len().max(1) as f64;
        stats.land_mean_before_m = mean(&before);
        stats.land_mean_after_m = mean(&after);
        let median = |v: &mut Vec<f32>| {
            if v.is_empty() {
                return 0.0;
            }
            let m = v.len() / 2;
            let (_, x, _) = v.select_nth_unstable_by(m, f32::total_cmp);
            f64::from(*x)
        };
        stats.land_median_before_m = median(&mut before);
        stats.land_median_after_m = median(&mut after);
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::structure::Window;

    fn level(grid: &GridDesc, z: f32) -> Level {
        let n = grid.cells();
        Level {
            w: grid.w as usize,
            h: grid.h as usize,
            dx: f64::from(grid.r) * 300.0,
            z: vec![z; n],
            area: vec![1.0; n],
            wgt: vec![1.0; n],
            kfac: vec![1.0; n],
            lake: vec![0; n],
            zs: Vec::new(),
        }
    }

    /// A normal fault through (0, 0) striking north, dipping east, 10 blocks of slip.
    fn fault(cx: f64) -> StructEvent {
        let (s, c) = (60.0f64.to_radians().sin(), 60.0f64.to_radians().cos());
        StructEvent {
            id: 0,
            key: 5000,
            t_years: -1.0e6,
            window: Window {
                centre: [cx, 0.0],
                along: [0.0, -1.0],
                across: [1.0, 0.0],
                half_along: 200.0,
                half_across: 80.0,
            },
            shape: Shape::Fault {
                y: 70.0,
                pole: [s, c, 0.0],
                slip: [10.0 * c, -10.0 * s, 0.0],
            },
        }
    }

    fn relief(items: Vec<(StructEvent, f64)>) -> Relief {
        Relief {
            items,
            profiles: vec![crate::structure::deform::sine_profile()],
            fold_share: 0.5,
            footwall_share: 0.2,
            mv: 20.0,
            circumference: 4096.0,
        }
    }

    #[test]
    fn a_young_normal_fault_makes_a_scarp_and_a_tilted_block() {
        let grid = GridDesc::for_planet("L2", 4096, 8).unwrap();
        let mut l = level(&grid, 500.0);
        let r = relief(vec![(fault(0.0), 1.0)]);
        let stats = r.apply(&mut l, &grid, 0.0);
        let at = |x: i64, z: i64| {
            let i = (x - grid.origin_block_x).div_euclid(8);
            let j = (z - grid.origin_block_z).div_euclid(8);
            f64::from(l.z[(j * i64::from(grid.w) + i) as usize])
        };
        let throw_m = 10.0 * 60.0f64.to_radians().sin() * 20.0;
        // Just east of the trace: down by nearly the whole throw. Just west: the shoulder, up.
        assert!(
            (at(8, 0) - (500.0 - throw_m * (1.0 - 0.01f64).powi(2))).abs() < 2.0,
            "{}",
            at(8, 0)
        );
        assert!(at(-8, 0) > 500.0 + 0.15 * throw_m && at(-8, 0) < 500.0 + 0.2 * throw_m + 1.0);
        // The block is tilted: the drop dies out away from the trace, and along it.
        assert!(at(40, 0) > at(8, 0) && at(40, 0) < 500.0);
        assert_eq!(at(88, 0), 500.0);
        assert!(at(8, 120) > at(8, 0));
        assert_eq!(at(8, 208), 500.0);
        assert!(stats.cells_changed > 100 && stats.max_down_m > 150.0 && stats.max_up_m > 20.0);
        // Half the relief left: half the scarp.
        let mut half = level(&grid, 500.0);
        relief(vec![(fault(0.0), 0.5)]).apply(&mut half, &grid, 0.0);
        let i = ((8 - grid.origin_block_x) / 8) as usize;
        let j = ((0 - grid.origin_block_z) / 8) as usize;
        let c = j * grid.w as usize + i;
        assert!(((500.0 - f64::from(half.z[c])) * 2.0 - (500.0 - at(8, 0))).abs() < 1.0);
    }

    #[test]
    fn two_facing_faults_make_a_graben() {
        let grid = GridDesc::for_planet("L2", 4096, 8).unwrap();
        let mut l = level(&grid, 500.0);
        // The second fault 96 blocks east, turned round: striking south, dipping west.
        let mut west_dipping = fault(96.0);
        west_dipping.window.along = [0.0, 1.0];
        west_dipping.window.across = [-1.0, 0.0];
        if let Shape::Fault { pole, slip, .. } = &mut west_dipping.shape {
            pole[0] = -pole[0];
            slip[0] = -slip[0];
        }
        relief(vec![(fault(0.0), 1.0), (west_dipping, 1.0)]).apply(&mut l, &grid, 0.0);
        let j = ((0 - grid.origin_block_z) / 8) as usize;
        let row: Vec<f32> = (0..grid.w as usize)
            .map(|i| l.z[j * grid.w as usize + i])
            .collect();
        let i_of = |x: i64| ((x - grid.origin_block_x) / 8) as usize;
        // The floor between the faults is lower than both shoulders.
        let floor = row[i_of(48)];
        assert!(floor < 400.0, "{floor}");
        assert!(row[i_of(-16)] > 500.0 && row[i_of(112)] > 500.0);
        assert!(row[i_of(8)] < 500.0 && row[i_of(88)] < 500.0);
    }

    #[test]
    fn land_stays_land_and_lakes_and_the_sea_are_left_alone() {
        let grid = GridDesc::for_planet("L2", 4096, 8).unwrap();
        // Low ground: the drop would take it under the sea.
        let mut l = level(&grid, 30.0);
        let stats = relief(vec![(fault(0.0), 1.0)]).apply(&mut l, &grid, 0.0);
        assert!(l.z.iter().all(|&z| z >= 1.0));
        assert_eq!(stats.land_cells, grid.cells() as u64);
        // The sea floor and a lake: untouched.
        let mut sea = level(&grid, -200.0);
        let s = relief(vec![(fault(0.0), 1.0)]).apply(&mut sea, &grid, 0.0);
        assert_eq!(s.cells_changed, 0);
        let mut lake = level(&grid, 500.0);
        lake.lake.fill(1);
        assert_eq!(
            relief(vec![(fault(0.0), 1.0)])
                .apply(&mut lake, &grid, 0.0)
                .cells_changed,
            0
        );
        // A thrust lifts its hanging wall.
        let mut thrust = fault(0.0);
        if let Shape::Fault { slip, .. } = &mut thrust.shape {
            slip[0] = -slip[0];
            slip[1] = -slip[1];
        }
        let mut up = level(&grid, 500.0);
        let s = relief(vec![(thrust, 1.0)]).apply(&mut up, &grid, 0.0);
        assert!(s.max_up_m > 150.0 && s.land_mean_after_m > s.land_mean_before_m);
    }
}

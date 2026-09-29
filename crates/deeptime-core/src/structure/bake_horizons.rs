// SPDX-License-Identifier: LGPL-3.0-or-later
//! The horizon bake (plan §5.7.4, §8.8): each L1 cell's column (`columns.rs`) as a horizon stack
//! in block space — the `strata/horizons` layer the Java strata pass reads (`StrataPass`), K
//! horizons per cell, each an i16 top (1/16 block) and a u8 class index into
//! `strata/classes.json` (`synth::EVERY_CLASS`), top-down; horizon h fills from its top down to
//! the next horizon's top, the last down to `min_y`. Unused horizons have top `i16::MIN` and class
//! 255 (`deeptime_format::raster`).
//!
//! Metres become blocks as the plan's bake does: an elevation of h metres above the sea of the
//! time is y = sea_level_y + h / m_per_block_v, and a unit of t metres spans t / m_per_block_v
//! blocks. The first horizon's top is the cell's surface (clamped to [min_y + 1, max_y]); units
//! wholly below `min_y` are dropped. Then **every horizon is at least one block thick after
//! scaling** (the last one down to `min_y` too) and **at most K are kept**: the thinnest horizon
//! under one block merges with its thinner neighbour, then, while more than K remain, the adjacent
//! pair with the smallest combined thickness merges (the strata log's cap rule); a merged horizon
//! takes the thicker member's class, and neighbours of one class merge.
//!
//! Plutons stay bodies (`plutons.rs`); P1-08 writes this layer and the bodies into the world data
//! (this task produces the stacks for the viewer and the tests).

use deeptime_format::{Codec, DType, FormatError, GridDesc, LayerDesc, LoadedLayer};
use rayon::prelude::*;
use serde_json::{Value, json};

use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::synth::EVERY_CLASS;
use crate::world::WorldState;

use super::columns::{self, Tables, Unit};
use super::{CellSource, StructureParams};

/// Unused horizon top.
pub const NO_TOP: i16 = i16::MIN;
/// Unused horizon class.
pub const NO_CLASS: u8 = 255;

/// The horizon stacks of a grid.
#[derive(Clone, Debug, PartialEq)]
pub struct Horizons {
    pub grid: GridDesc,
    pub k: u32,
    pub min_y: i32,
    /// `tops[c·K + h]`, 1/16 block.
    pub tops: Vec<i16>,
    /// `classes[c·K + h]`, index into `synth::EVERY_CLASS`.
    pub classes: Vec<u8>,
}

/// Bakes one column: `units` top-down under a surface at `surface_y` (blocks) into `tops` and
/// `classes` (length K, padded). Returns the number of horizons.
pub fn bake_cell(
    units: &[Unit],
    surface_y: f64,
    min_y: i32,
    max_y: i32,
    m_per_block_v: f64,
    tops: &mut [i16],
    classes: &mut [u8],
) -> usize {
    let k = tops.len();
    let floor = min_y * 16;
    let top0 = surface_y.clamp(f64::from(min_y) + 1.0, f64::from(max_y));
    let mut hz: Vec<(i32, u16)> = Vec::with_capacity(units.len());
    let mut y = top0;
    for u in units {
        if y <= f64::from(min_y) {
            break;
        }
        let q = (y * 16.0).round() as i32;
        match hz.last() {
            Some(&(_, c)) if c == u.class => {}
            _ => hz.push((q, u.class)),
        }
        y -= f64::from(u.thickness_m) / m_per_block_v;
    }
    if hz.is_empty() {
        hz.push(((top0 * 16.0).round() as i32, units.last().map_or(0, |u| u.class)));
    }
    let thick = |hz: &[(i32, u16)], i: usize| -> i32 {
        let below = if i + 1 < hz.len() { hz[i + 1].0 } else { floor };
        hz[i].0 - below
    };
    // Merge horizon i with i + 1: the upper top, the thicker member's class.
    let merge = |hz: &mut Vec<(i32, u16)>, i: usize| {
        let (ta, tb) = (thick(hz, i), thick(hz, i + 1));
        let class = if ta >= tb { hz[i].1 } else { hz[i + 1].1 };
        hz[i].1 = class;
        hz.remove(i + 1);
        // Same-class neighbours join.
        if i + 1 < hz.len() && hz[i + 1].1 == class {
            hz.remove(i + 1);
        }
        if i > 0 && hz[i - 1].1 == class {
            hz.remove(i);
        }
    };
    loop {
        if hz.len() < 2 {
            break;
        }
        // The thinnest horizon under one block, merged with its thinner neighbour.
        let thin = (0..hz.len())
            .filter(|&i| thick(&hz, i) < 16)
            .min_by_key(|&i| (thick(&hz, i), i));
        if let Some(i) = thin {
            let up = if i > 0 { Some(thick(&hz, i - 1)) } else { None };
            let down = if i + 1 < hz.len() {
                Some(thick(&hz, i + 1))
            } else {
                None
            };
            let with_up = match (up, down) {
                (Some(a), Some(b)) => a <= b,
                (Some(_), None) => true,
                _ => false,
            };
            merge(&mut hz, if with_up { i - 1 } else { i });
            continue;
        }
        if hz.len() > k {
            let i = (0..hz.len() - 1)
                .min_by_key(|&i| (thick(&hz, i) + thick(&hz, i + 1), i))
                .expect("two or more horizons");
            merge(&mut hz, i);
            continue;
        }
        break;
    }
    for h in 0..k {
        match hz.get(h) {
            Some(&(t, c)) => {
                tops[h] = t.clamp(i32::from(i16::MIN) + 1, i32::from(i16::MAX)) as i16;
                classes[h] = c as u8;
            }
            None => {
                tops[h] = NO_TOP;
                classes[h] = NO_CLASS;
            }
        }
    }
    hz.len()
}

impl Horizons {
    pub fn cells(&self) -> usize {
        self.grid.cells()
    }

    /// Cell `c`'s used horizons: (tops, classes).
    pub fn cell(&self, c: usize) -> (&[i16], &[u8]) {
        let k = self.k as usize;
        let n = self.classes[c * k..(c + 1) * k]
            .iter()
            .position(|&x| x == NO_CLASS)
            .unwrap_or(k);
        (&self.tops[c * k..c * k + n], &self.classes[c * k..c * k + n])
    }

    /// The plan's acceptance (P1-07): every stack ordered top-down, every horizon at least one
    /// block thick (the last down to `min_y`), every class a class of the vocabulary, at most K,
    /// padding only after the used horizons.
    pub fn check(&self) -> Result<(), String> {
        let k = self.k as usize;
        let floor = self.min_y * 16;
        for c in 0..self.cells() {
            let (tops, classes) = self.cell(c);
            if tops.is_empty() {
                return Err(format!("cell {c}: no horizons"));
            }
            for h in 0..tops.len() {
                if usize::from(classes[h]) >= EVERY_CLASS.len() {
                    return Err(format!("cell {c} horizon {h}: class {}", classes[h]));
                }
                let below = if h + 1 < tops.len() {
                    i32::from(tops[h + 1])
                } else {
                    floor
                };
                if i32::from(tops[h]) - below < 16 {
                    return Err(format!(
                        "cell {c} horizon {h}: top {} and {} below it: thinner than a block",
                        tops[h], below
                    ));
                }
            }
            for h in tops.len()..k {
                if self.tops[c * k + h] != NO_TOP || self.classes[c * k + h] != NO_CLASS {
                    return Err(format!("cell {c} horizon {h}: padding is not unused"));
                }
            }
        }
        Ok(())
    }

    /// The `strata/horizons` layer (per cell K i16 tops, then K u8 classes).
    pub fn to_layer(&self, codec: Codec) -> Result<LoadedLayer, FormatError> {
        let k = self.k as usize;
        let mut bytes = Vec::with_capacity(self.cells() * 3 * k);
        for c in 0..self.cells() {
            for &t in &self.tops[c * k..(c + 1) * k] {
                bytes.extend_from_slice(&t.to_le_bytes());
            }
            bytes.extend_from_slice(&self.classes[c * k..(c + 1) * k]);
        }
        let mut d = LayerDesc::scalar("strata/horizons", DType::HorizonStack, self.grid.clone(), codec);
        d.units = "block/16 top, class index".into();
        d.scale = 1.0 / 16.0;
        d.stack_depth = Some(self.k);
        LoadedLayer::new(d, bytes)
    }

    /// `strata/classes.json`.
    pub fn classes_json() -> Value {
        json!({ "classes": EVERY_CLASS.as_slice() })
    }

    /// The class of each cell's first horizon (the rock at its surface), as f32 for rasters.
    pub fn outcrop(&self) -> Vec<f32> {
        let k = self.k as usize;
        (0..self.cells())
            .map(|c| f32::from(self.classes[c * k]))
            .collect()
    }

    /// Mean horizons per cell.
    pub fn mean_horizons(&self) -> f64 {
        let used = self.classes.iter().filter(|&&x| x != NO_CLASS).count();
        used as f64 / self.cells().max(1) as f64
    }
}

/// Bakes every cell of the world's L1 grid at the end of a run (or at any step after the
/// structure stage ran).
pub fn bake(world: &WorldState, preset: &PlanetPreset) -> Result<Horizons, CoreError> {
    let p = preset.structure.clone().unwrap_or_default();
    let src = CellSource::new(world, preset)?;
    Ok(bake_with(&src, preset, &p, &Tables::get()))
}

/// [`bake`] from an assembled source.
pub fn bake_with(
    src: &CellSource<'_>,
    preset: &PlanetPreset,
    p: &StructureParams,
    t: &Tables,
) -> Horizons {
    let s = &preset.scale;
    let k = p.horizons as usize;
    let n = src.grid.cells();
    let mut tops = vec![NO_TOP; n * k];
    let mut classes = vec![NO_CLASS; n * k];
    tops.par_chunks_mut(k)
        .zip(classes.par_chunks_mut(k))
        .enumerate()
        .for_each_init(Vec::new, |units, (c, (tc, cc))| {
            let surface_y = src.surface_y(c, s);
            let depth = (surface_y - f64::from(s.min_y)).max(1.0) * s.m_per_block_v + 100.0;
            columns::build(&src.cell(c), t, p, src.now_myr, depth, units);
            bake_cell(units, surface_y, s.min_y, s.max_y, s.m_per_block_v, tc, cc);
        });
    Horizons {
        grid: src.grid.clone(),
        k: k as u32,
        min_y: s.min_y,
        tops,
        classes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::columns::Source;
    use crate::surface::lithology::index;

    fn unit(name: &str, th: f32) -> Unit {
        Unit {
            class: index(name),
            thickness_m: th,
            source: Source::Log,
        }
    }

    #[test]
    fn thin_units_merge_and_the_stack_fits_k() {
        let k = 4;
        let mut tops = vec![0i16; k];
        let mut classes = vec![0u8; k];
        // Surface at y 100 (2,000 m above y 0 at 20 m/block): a 5-m soil, 400 m sandstone, 10 m
        // coal, 600 m shale, 3 km limestone, then granite to depth.
        let units = [
            unit("soil", 5.0),
            unit("sandstone", 400.0),
            unit("coal", 10.0),
            unit("shale", 600.0),
            unit("limestone", 3000.0),
            unit("granite", 20_000.0),
        ];
        let n = bake_cell(&units, 100.0, -352, 639, 20.0, &mut tops, &mut classes);
        assert_eq!(n, 4);
        let h = Horizons {
            grid: GridDesc::for_planet("L1", 64, 32).unwrap(),
            k: k as u32,
            min_y: -352,
            tops: tops.repeat(2),
            classes: classes.repeat(2),
        };
        h.check().unwrap();
        // The soil and coal were thinner than a block and joined their neighbours.
        assert_eq!(
            &classes[..],
            &[
                index("sandstone") as u8,
                index("shale") as u8,
                index("limestone") as u8,
                index("granite") as u8
            ]
        );
        assert_eq!(tops[0], 1600, "the surface");
        // More horizons than K: the thinnest pairs merge.
        let many: Vec<Unit> = (0..20)
            .map(|i| unit(if i % 2 == 0 { "sandstone" } else { "shale" }, 100.0 + i as f32))
            .collect();
        let n = bake_cell(&many, 300.0, -352, 639, 20.0, &mut tops, &mut classes);
        assert!((1..=k).contains(&n), "{n}");
        let h = Horizons {
            grid: GridDesc::for_planet("L1", 64, 32).unwrap(),
            k: k as u32,
            min_y: -352,
            tops: tops.repeat(2),
            classes: classes.repeat(2),
        };
        h.check().unwrap();
        // A surface below the world's floor still gets one horizon a block thick.
        let n = bake_cell(&units, -500.0, -352, 639, 20.0, &mut tops, &mut classes);
        assert_eq!(n, 1);
        assert_eq!(tops[0], (-351 * 16) as i16);
        assert_eq!(tops[1], NO_TOP);
    }
}

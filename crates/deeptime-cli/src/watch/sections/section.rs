// SPDX-License-Identifier: LGPL-3.0-or-later
//! A vertical strata section along a line: each sample is one block column as the evaluator
//! decides it (the terrain's top block, the water over it, the rock from there down to the floor,
//! the bodies it passes through), so a section agrees with `deeptime inspect-column` and the game
//! at every column it samples.

use std::sync::Mutex;

use deeptime_format::strata::{ColumnHorizon, StackScratch};
use serde_json::{Value, json};

use super::world::{NONE, RockWorld};

/// Most columns a section samples (a longer line is sampled sparser).
pub const MAX_COLUMNS: usize = 2048;

/// One sampled column.
pub struct Column {
    pub x: i64,
    pub z: i64,
    /// The terrain's top block.
    pub ground: i32,
    /// The highest water block, and what holds it (`sea`, `lake`, `river`).
    pub water: Option<(i32, &'static str)>,
    /// (top y, class) top-down, from `ground` to the floor: a run fills from its top down to the
    /// next run's top + 1, the last down to the floor. [`NONE`] = no rock named.
    pub runs: Vec<(i32, u16)>,
    /// (body id, top y, bottom y) per maximal run of the column's blocks inside a body.
    pub bodies: Vec<(u32, i32, i32)>,
    /// The horizon stack the runs were painted from (top y, class, unit key), top-down.
    pub horizons: Vec<ColumnHorizon>,
    /// The crust: its `meta/crust_type` code and `meta/crust_age_myr` ([`ColumnWorld::crust_at`]).
    ///
    /// [`ColumnWorld::crust_at`]: deeptime_format::ColumnWorld::crust_at
    pub crust: (Option<i64>, Option<f64>),
    /// The structural events at the column ([`deeptime_format::column::events_at`]).
    pub events: Vec<Value>,
}

pub struct Section {
    /// Start and end blocks (the end continues past the seam if the short way crosses it).
    pub a: (i64, i64),
    pub b: (i64, i64),
    pub length_blocks: f64,
    pub columns: Vec<Column>,
}

/// The end block of a line from `a` to `b` along the shorter way round in x (so a line across the
/// seam does not circle the planet).
pub fn short_way(c: i64, a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    let dx = (b.0 - a.0 + c / 2).rem_euclid(c) - c / 2;
    (a.0 + dx, b.1)
}

impl RockWorld {
    /// The section from `a` to `b` (blocks; `b` as [`short_way`] gives it), `n` columns evenly
    /// spaced including both ends.
    pub fn section(&self, a: (i64, i64), b: (i64, i64), n: usize, threads: usize) -> Section {
        let n = n.clamp(2, MAX_COLUMNS);
        let (dx, dz) = ((b.0 - a.0) as f64, (b.1 - a.1) as f64);
        let at = |i: usize| {
            let t = i as f64 / (n - 1) as f64;
            (a.0 + (dx * t).round() as i64, a.1 + (dz * t).round() as i64)
        };
        let mut columns: Vec<Option<Column>> = (0..n).map(|_| None).collect();
        let queue = Mutex::new(columns.iter_mut().enumerate());
        std::thread::scope(|s| {
            for _ in 0..threads.max(1) {
                s.spawn(|| {
                    let (mut stack, mut scratch) = (Vec::new(), StackScratch::default());
                    loop {
                        let next = queue.lock().unwrap_or_else(|e| e.into_inner()).next();
                        let Some((i, slot)) = next else { break };
                        let (x, z) = at(i);
                        *slot = Some(self.column(x, z, &mut scratch, &mut stack));
                    }
                });
            }
        });
        Section {
            a,
            b,
            length_blocks: dx.hypot(dz),
            columns: columns.into_iter().flatten().collect(),
        }
    }

    /// One column of a section.
    pub fn column(
        &self,
        x: i64,
        z: i64,
        scratch: &mut StackScratch,
        stack: &mut Vec<ColumnHorizon>,
    ) -> Column {
        let (min_y, max_y) = (self.cw.min_y, self.cw.max_y);
        let hy = self.cw.hydro(x, z);
        let ground = hy.ground.clamp(min_y - 1, max_y);
        let water = match (hy.kind.is_water(), hy.top) {
            (true, Some(top)) => Some((top, hy.kind.name())),
            _ => None,
        };
        self.stack(scratch, x, z, stack);
        let crust = self.cw.crust_at(x, z);
        let crust = (crust["type"].as_i64(), crust["age_myr"].as_f64());
        let events = self.events_at(scratch, x, z);
        let mut runs: Vec<(i32, u16)> = Vec::new();
        let mut bodies = Vec::new();
        if ground >= min_y {
            // Block y = ground − i; a body's class overrides the horizon's where it names one.
            let mut painted = vec![NONE; (ground - min_y + 1) as usize];
            self.paint_column(x, z, min_y, ground, &mut painted);
            for (i, &p) in painted.iter().enumerate() {
                let y = ground - i as i32;
                let class = if p != NONE {
                    p
                } else {
                    deeptime_format::column::horizon_at(stack, y)
                        .map_or(NONE, |h| u16::from(stack[h].1))
                };
                if runs.last().is_none_or(|r| r.1 != class) {
                    runs.push((y, class));
                }
            }
            let wx = self.cw.wrap_x(x);
            for b in self.bodies_over(x, z) {
                let (lo, hi) = (b.aabb[0][1].max(min_y), b.aabb[1][1].min(ground));
                let mut run: Option<(i32, i32)> = None;
                for y in (lo..=hi).rev() {
                    let inside = self.inside(b, wx, y, z);
                    match (&mut run, inside) {
                        (Some((_, bottom)), true) => *bottom = y,
                        (None, true) => run = Some((y, y)),
                        (Some(r), false) => {
                            bodies.push((b.id, r.0, r.1));
                            run = None;
                        }
                        (None, false) => {}
                    }
                }
                if let Some(r) = run {
                    bodies.push((b.id, r.0, r.1));
                }
            }
        }
        Column {
            x,
            z,
            ground,
            water,
            runs,
            bodies,
            horizons: stack.clone(),
            crust,
            events,
        }
    }
}

impl Section {
    /// The distinct (unit key, class) pairs of the columns' horizons, sorted: the section's unit
    /// table, which [`Section::columns_json`]'s `units` index.
    pub fn units(&self) -> Vec<(u16, u8)> {
        let mut out: Vec<(u16, u8)> = self
            .columns
            .iter()
            .flat_map(|c| c.horizons.iter().map(|h| (h.2, h.1)))
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// The structural events at any of the columns, youngest first as the world lists them
    /// (each as [`deeptime_format::column::events_at`] gives it at the first column it holds,
    /// without the column's own numbers).
    pub fn events(&self) -> Vec<Value> {
        let mut seen = std::collections::BTreeMap::new();
        for e in self.columns.iter().flat_map(|c| &c.events) {
            if let Some(id) = e["id"].as_u64() {
                seen.entry(id).or_insert_with(|| {
                    let mut v = json!({});
                    for k in ["id", "kind", "t_years", "key", "class", "wavelength", "amplitude"] {
                        if let Some(x) = e.get(k) {
                            v[k] = x.clone();
                        }
                    }
                    v
                });
            }
        }
        let mut out: Vec<Value> = seen.into_values().collect();
        out.sort_by(|a, b| {
            b["t_years"]
                .as_f64()
                .unwrap_or(0.0)
                .total_cmp(&a["t_years"].as_f64().unwrap_or(0.0))
        });
        out
    }

    /// The columns as compact arrays (the page draws them): per column its block, top block,
    /// water, runs `[top, class, top, class, …]` (class −1 = none), body spans
    /// `[id, top, bottom, …]`, the horizon stack `units` as `[top, unit, …]` (an index into
    /// `units`, [`Section::units`]), the crust code and age, and the structural events
    /// `[id, changed, …]` (changed 1, 0, or −1 when not checked).
    pub fn columns_json(&self, units: &[(u16, u8)]) -> Value {
        let cols = &self.columns;
        let class = |k: u16| if k == NONE { -1 } else { i64::from(k) };
        let unit = |h: &ColumnHorizon| {
            units
                .binary_search(&(h.2, h.1))
                .map_or(-1, |i| i as i64)
        };
        json!({
            "x": cols.iter().map(|c| c.x).collect::<Vec<_>>(),
            "z": cols.iter().map(|c| c.z).collect::<Vec<_>>(),
            "ground": cols.iter().map(|c| c.ground).collect::<Vec<_>>(),
            "water": cols.iter().map(|c| c.water.map(|w| w.0)).collect::<Vec<_>>(),
            "water_kind": cols.iter().map(|c| c.water.map(|w| w.1)).collect::<Vec<_>>(),
            "runs": cols.iter().map(|c| c.runs.iter()
                .flat_map(|&(t, k)| [i64::from(t), class(k)]).collect::<Vec<_>>()).collect::<Vec<_>>(),
            "bodies": cols.iter().map(|c| c.bodies.iter()
                .flat_map(|&(id, t, b)| [i64::from(id), i64::from(t), i64::from(b)]).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            "units": cols.iter().map(|c| c.horizons.iter()
                .flat_map(|h| [i64::from(h.0), unit(h)]).collect::<Vec<_>>()).collect::<Vec<_>>(),
            "crust": cols.iter().map(|c| c.crust.0).collect::<Vec<_>>(),
            "crust_age": cols.iter().map(|c| c.crust.1).collect::<Vec<_>>(),
            "events": cols.iter().map(|c| c.events.iter()
                .flat_map(|e| [
                    e["id"].as_i64().unwrap_or(-1),
                    e["changed"].as_bool().map_or(-1, i64::from),
                ]).collect::<Vec<_>>()).collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_short_way_crosses_the_seam() {
        let c = 16384;
        // 8000 → −8000 is 384 blocks east across the seam, not 16,000 west.
        assert_eq!(short_way(c, (8000, 5), (-8000, 9)), (8384, 9));
        assert_eq!(short_way(c, (-8000, 5), (8000, 9)), (-8384, 9));
        assert_eq!(short_way(c, (10, 0), (500, 0)), (500, 0));
        assert_eq!(short_way(c, (500, 0), (10, 0)), (10, 0));
        // Exactly half way round goes west (x in [−C/2, C/2)).
        assert_eq!(short_way(c, (0, 0), (8192, 0)), (-8192, 0));
    }
}

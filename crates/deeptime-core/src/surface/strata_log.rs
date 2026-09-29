// SPDX-License-Identifier: LGPL-3.0-or-later
//! The strata log (plan §8.2, §5.7.5): per L1 cell, up to K layers of (age, thickness, class,
//! provenance) — 12 bytes each — recording every deposit on top of the crust. Its layer model
//! follows Landlab's `Lithology`/`MaterialLayers` (https://github.com/landlab/landlab,
//! `src/landlab/components/lithology/`, MIT; the design, not the code): layers are stored bottom to
//! top, deposition adds a layer on top (combined with the top layer when it has the same rock
//! type), erosion removes material from the top and drops emptied layers, and the surface's
//! properties are the top layer's.
//!
//! **Additions:** each layer keeps the age at its base (Myr, the time it started to be laid down)
//! and a provenance code; K caps the log — when a deposit would need layer K + 1, the adjacent pair
//! with the smallest combined thickness is merged into one (the thicker member's class and
//! provenance, the older base age), so thick layers and the most recent ones survive.
//!
//! Invariants (`check`): 0 ≤ count ≤ K; every used layer has a finite thickness > 0 and a class
//! inside the vocabulary; no two adjacent layers share a class; base ages never decrease upwards.

use serde::{Deserialize, Serialize};

/// Provenance of material formed in place (carbonate, evaporite, coal, regolith).
pub const IN_SITU: u16 = 0;

/// Provenance code of clastic sediment whose largest source family is `family` (0–3) with
/// `share` (0–1) of the flux: (family + 1) · 256 + share · 255.
pub fn provenance(family: usize, share: f64) -> u16 {
    ((family as u16 + 1) << 8) | (share.clamp(0.0, 1.0) * 255.0).round() as u16
}

/// One layer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    /// Base age, Myr (relative to world creation, negative in the past).
    pub age_myr: f32,
    pub thickness_m: f32,
    /// Index into `synth::EVERY_CLASS`.
    pub class: u16,
    /// See [`provenance`] and [`IN_SITU`].
    pub provenance: u16,
}

/// The log of every cell of a grid.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StrataLog {
    /// K, layers per cell.
    pub k: u32,
    /// Layers used per cell.
    pub count: Vec<u8>,
    /// `layers[c·K + i]`, i = 0 the bottom.
    pub layers: Vec<Layer>,
}

impl StrataLog {
    pub fn new(cells: usize, k: u32) -> StrataLog {
        let k = k.clamp(1, 255);
        StrataLog {
            k,
            count: vec![0; cells],
            layers: vec![Layer::default(); cells * k as usize],
        }
    }

    pub fn cells(&self) -> usize {
        self.count.len()
    }

    /// Bytes per cell (12·K plus the count).
    pub fn bytes_per_cell(k: u32) -> u64 {
        12 * u64::from(k) + 1
    }

    /// The used layers of cell `c`, bottom first.
    pub fn column(&self, c: usize) -> &[Layer] {
        let k = self.k as usize;
        &self.layers[c * k..c * k + usize::from(self.count[c])]
    }

    /// Total thickness of cell `c`'s log, m.
    pub fn total(&self, c: usize) -> f64 {
        self.column(c)
            .iter()
            .map(|l| f64::from(l.thickness_m))
            .sum()
    }

    /// The top layer of cell `c`.
    pub fn top(&self, c: usize) -> Option<&Layer> {
        self.column(c).last()
    }

    pub fn clear(&mut self, c: usize) {
        self.count[c] = 0;
    }

    /// Lays `thickness` m of `class` on cell `c` at `age_myr`.
    pub fn deposit(&mut self, c: usize, age_myr: f32, thickness: f32, class: u16, prov: u16) {
        self.column_mut(c).deposit(age_myr, thickness, class, prov);
    }

    /// Removes up to `amount` m from the top of cell `c`'s log; returns what it removed.
    pub fn erode(&mut self, c: usize, amount: f64) -> f64 {
        self.column_mut(c).erode(amount)
    }

    /// Cell `c`'s column, for editing.
    pub fn column_mut(&mut self, c: usize) -> ColumnMut<'_> {
        let k = self.k as usize;
        ColumnMut {
            layers: &mut self.layers[c * k..(c + 1) * k],
            count: &mut self.count[c],
        }
    }

    /// Checks the invariants of cell `c` (see the module docs); `classes` is the vocabulary size.
    pub fn check(&self, c: usize, classes: usize) -> Result<(), String> {
        let col = self.column(c);
        if col.len() > self.k as usize {
            return Err(format!("cell {c}: {} layers > K {}", col.len(), self.k));
        }
        for (i, l) in col.iter().enumerate() {
            if !(l.thickness_m.is_finite() && l.thickness_m > 0.0) {
                return Err(format!("cell {c} layer {i}: thickness {}", l.thickness_m));
            }
            if usize::from(l.class) >= classes {
                return Err(format!("cell {c} layer {i}: class {}", l.class));
            }
            if i > 0 {
                let below = &col[i - 1];
                if below.class == l.class {
                    return Err(format!("cell {c} layers {}–{i}: same class", i - 1));
                }
                if l.age_myr < below.age_myr {
                    return Err(format!(
                        "cell {c} layer {i}: base age {} below {}",
                        l.age_myr, below.age_myr
                    ));
                }
            }
        }
        Ok(())
    }
}

/// One cell's column: its K layer slots and its count (so a parallel loop can edit every column
/// of a log at once, `par_chunks_mut(K)` zipped with the counts).
pub struct ColumnMut<'a> {
    pub layers: &'a mut [Layer],
    pub count: &'a mut u8,
}

impl ColumnMut<'_> {
    /// The used layers, bottom first.
    pub fn used(&self) -> &[Layer] {
        &self.layers[..usize::from(*self.count)]
    }

    pub fn top(&self) -> Option<&Layer> {
        self.used().last()
    }

    pub fn total(&self) -> f64 {
        self.used().iter().map(|l| f64::from(l.thickness_m)).sum()
    }

    /// Lays `thickness` m of `class` at `age_myr` (see [`StrataLog`]'s docs for the merges).
    pub fn deposit(&mut self, age_myr: f32, thickness: f32, class: u16, prov: u16) {
        if thickness.is_nan() || thickness <= 0.0 {
            return;
        }
        let k = self.layers.len();
        let mut n = usize::from(*self.count);
        let col = &mut *self.layers;
        if n > 0 && col[n - 1].class == class {
            let top = &mut col[n - 1];
            // Provenance follows the larger contribution.
            if thickness > top.thickness_m {
                top.provenance = prov;
            }
            top.thickness_m += thickness;
            return;
        }
        if n == k {
            n = merge_thinnest(&mut col[..n]);
            // A merge can bring two same-class layers together at the top.
            if n > 0 && col[n - 1].class == class {
                col[n - 1].thickness_m += thickness;
                *self.count = n as u8;
                return;
            }
        }
        col[n] = Layer {
            age_myr,
            thickness_m: thickness,
            class,
            provenance: prov,
        };
        *self.count = (n + 1) as u8;
    }

    /// Removes up to `amount` m from the top; returns what it removed.
    pub fn erode(&mut self, amount: f64) -> f64 {
        let mut left = amount.max(0.0);
        let mut removed = 0.0;
        let mut n = usize::from(*self.count);
        while left > 0.0 && n > 0 {
            let top = &mut self.layers[n - 1];
            let t = f64::from(top.thickness_m);
            if t <= left {
                left -= t;
                removed += t;
                n -= 1;
            } else {
                top.thickness_m = (t - left) as f32;
                removed += left;
                left = 0.0;
                if top.thickness_m <= 0.0 {
                    n -= 1;
                }
            }
        }
        *self.count = n as u8;
        removed
    }
}

/// Merges the adjacent pair with the smallest combined thickness (the thicker one's class and
/// provenance, the older base age), then the result with a same-class neighbour; returns the new
/// number of layers.
fn merge_thinnest(col: &mut [Layer]) -> usize {
    let n = col.len();
    if n < 2 {
        return n;
    }
    let mut best = 0;
    let mut best_t = f32::INFINITY;
    for i in 0..n - 1 {
        let t = col[i].thickness_m + col[i + 1].thickness_m;
        if t < best_t {
            best_t = t;
            best = i;
        }
    }
    let (a, b) = (col[best], col[best + 1]);
    let keep = if a.thickness_m >= b.thickness_m { a } else { b };
    col[best] = Layer {
        age_myr: a.age_myr.min(b.age_myr),
        thickness_m: a.thickness_m + b.thickness_m,
        class: keep.class,
        provenance: keep.provenance,
    };
    col.copy_within(best + 2..n, best + 1);
    let mut m = n - 1;
    // The merged layer may now share a class with a neighbour.
    let mut i = best;
    if i > 0 && col[i - 1].class == col[i].class {
        col[i - 1].thickness_m += col[i].thickness_m;
        col.copy_within(i + 1..m, i);
        m -= 1;
        i -= 1;
    }
    if i + 1 < m && col[i + 1].class == col[i].class {
        col[i].thickness_m += col[i + 1].thickness_m;
        col.copy_within(i + 2..m, i + 1);
        m -= 1;
    }
    m
}

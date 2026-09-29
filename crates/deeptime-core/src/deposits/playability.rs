// SPDX-License-Identifier: LGPL-3.0-or-later
//! The playability pass (plan §7.8, §8.6 step 8; decisions file "Playability overrides"):
//! regional minimums for the essentials. The planet is cut into square regions of
//! `region_blocks`; in every region with land, each essential material needs
//! `min_sites_per_region` sites (exposed ones — top within [`EXPOSED_BLOCKS`] of the surface — when
//! `exposed`). Where natural placement fell short, the engine adds the smallest plausible deposit
//! (P90 tonnage, median grades) of the most favourable type producing the material, at that
//! type's most favourable land cell in the region, and records it as
//! `deeptime:guaranteed_occurrence` so "Why here?" stays honest.
//!
//! The essentials list is a pack's to define (`deeptime:essentials` registry, P1-15); until then
//! the preset carries it.

use serde::{Deserialize, Serialize};

/// A site counts as exposed when its top is at most this many blocks below the surface.
pub const EXPOSED_BLOCKS: f64 = 16.0;

/// One essential material.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Essential {
    /// `c:iron`.
    pub material: String,
    pub min_sites_per_region: u32,
    pub region_blocks: u32,
    #[serde(default)]
    pub exposed: bool,
}

/// The region grid of an essential.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Regions {
    pub c: f64,
    pub size: f64,
    pub cols: u32,
    pub rows: u32,
}

impl Regions {
    pub fn new(c: u32, region_blocks: u32) -> Regions {
        let rb = region_blocks.max(64);
        Regions {
            c: f64::from(c),
            size: f64::from(rb),
            cols: c.div_ceil(rb),
            rows: (c / 2).div_ceil(rb),
        }
    }

    /// The region (col, row) of block (x, z).
    pub fn of(&self, x: f64, z: f64) -> (u32, u32) {
        let col =
            (((x + self.c / 2.0) / self.size).floor() as i64).rem_euclid(i64::from(self.cols));
        let row = ((z + self.c / 4.0) / self.size)
            .floor()
            .clamp(0.0, f64::from(self.rows - 1));
        (col as u32, row as u32)
    }

    pub fn index(&self, x: f64, z: f64) -> usize {
        let (c, r) = self.of(x, z);
        (r * self.cols + c) as usize
    }

    pub fn count(&self) -> usize {
        (self.cols * self.rows) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regions_cover_the_planet() {
        let r = Regions::new(16_384, 4096);
        assert_eq!((r.cols, r.rows), (4, 2));
        assert_eq!(r.of(-8192.0, -4096.0), (0, 0));
        assert_eq!(r.of(8191.0, 4095.0), (3, 1));
        assert_eq!(r.of(8192.0 + 10.0, 0.0), (0, 1), "x wraps");
        assert_eq!(r.index(0.0, -1.0), 2);
        assert_eq!(r.count(), 8);
    }
}

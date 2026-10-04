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
//! A site counts toward an essential (here and in the spawn ranking) only when the material is one
//! of its **primary** products ([`Essential::counts`]; `primary_only`, default on). Counting
//! co-products and by-products let a 0.07 Mt impact Cu-Zn-Au deposit satisfy the copper essential of
//! a planet with two porphyries, and ranked the spawn beside it (P1-15 on real planets).
//!
//! The essentials list is a pack's to define: the game passes its `deeptime:essentials` registry
//! (P1-15) in the content bundle (`{"essentials": [...]}`), which replaces the preset's list
//! (`crate::preset::Preset::with_content`); runs without a bundle (the CLI) use the preset's.

use deeptime_format::Product;
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
    /// The spawn must lie within this many blocks of a site of the material (plan §7.8); the
    /// spawn ranking ([`super::spawn`]) scores candidates by it. `None`: no spawn requirement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_blocks_from_spawn: Option<u32>,
    /// Only a site whose product of the material has the role `primary` counts toward the regional
    /// minimum and the spawn's reach (default true); off, a co-product or by-product counts too.
    #[serde(default = "primary_only_default")] // ledger: DEP-206
    pub primary_only: bool,
}

fn primary_only_default() -> bool {
    true
}

impl Essential {
    /// Whether a site with these products counts toward this essential.
    pub fn counts(&self, products: &[Product]) -> bool {
        products
            .iter()
            .any(|p| p.material == self.material && (!self.primary_only || p.role == "primary"))
    }
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
        let (span_x, span_z) = deeptime_format::Projection::new(c).index_span_blocks();
        Regions {
            c: f64::from(c),
            size: f64::from(rb),
            // The regions' frame is the projection's index frame.
            cols: (span_x as u32).div_ceil(rb),
            rows: (span_z as u32).div_ceil(rb),
        }
    }

    /// The region (col, row) of block (x, z).
    pub fn of(&self, x: f64, z: f64) -> (u32, u32) {
        let (ox, oz) = deeptime_format::Projection::of(self.c).index_origin();
        let col = (((x - ox) / self.size).floor() as i64).rem_euclid(i64::from(self.cols));
        let row = ((z - oz) / self.size)
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

    fn product(material: &str, role: &str) -> Product {
        Product {
            material: material.into(),
            grade: 0.01,
            tonnage_t: 1.0e6,
            role: role.into(),
        }
    }

    #[test]
    fn only_primary_products_count_unless_the_essential_says_otherwise() {
        let json = r#"{"material": "c:copper", "min_sites_per_region": 1, "region_blocks": 8192}"#;
        let mut e: Essential = serde_json::from_str(json).unwrap();
        assert!(e.primary_only, "the default counts primary products only");
        let porphyry = [
            product("c:copper", "primary"),
            product("c:gold", "byproduct"),
        ];
        let impact = [
            product("c:zinc", "primary"),
            product("c:copper", "coproduct"),
        ];
        assert!(e.counts(&porphyry));
        assert!(!e.counts(&impact), "a co-product is not a copper source");
        assert!(!e.counts(&[]), "a site with no ore left counts for nothing");
        e.primary_only = false;
        assert!(e.counts(&impact));
        assert!(!e.counts(&[]));
        let off: Essential = serde_json::from_str(
            r#"{"material": "c:copper", "min_sites_per_region": 1, "region_blocks": 64, "primary_only": false}"#,
        )
        .unwrap();
        assert!(!off.primary_only);
    }

    #[test]
    fn regions_cover_the_planet() {
        let r = Regions::new(16_384, 4096);
        assert_eq!((r.cols, r.rows), (4, 2));
        assert_eq!(r.of(-8192.0, -4096.0), (0, 0));
        assert_eq!(r.of(8191.0, 4095.0), (3, 1));
        // Rows from z = −2,624 (the grids' extent past the Lambert pole at 2,607.6).
        assert_eq!(r.of(8192.0 + 10.0, 2000.0), (0, 1), "x wraps");
        assert_eq!(r.index(0.0, -1.0), 2);
        assert_eq!(r.count(), 8);
    }
}

// SPDX-License-Identifier: LGPL-3.0-or-later
//! The playability pass (plan §7.8, §8.6 step 8; decisions file "Playability overrides"):
//! regional minimums for the essentials. The world the game plays is cut into square regions of
//! `region_blocks` on its own index frame ([`WorldFrame`]: a cube world's, not the
//! simulation's Lambert one); in every region with land, each essential material needs
//! `min_sites_per_region` sites (exposed ones — top within [`EXPOSED_BLOCKS`] of the surface — when
//! `exposed`). Where natural placement fell short, the engine adds the smallest plausible deposit
//! (P90 tonnage, median grades) of the most favourable type producing the material, at that
//! type's most favourable land cell in the region, and records it as
//! `deeptime:guaranteed_occurrence` so "Why here?" stays honest.
//!
//! A site counts toward an essential only when the material is one of its **primary** products
//! ([`Essential::counts`]; `primary_only`, default on). Counting co-products and by-products let a
//! 0.07 Mt impact Cu-Zn-Au deposit satisfy the copper essential of a planet with two porphyries
//! (P1-15 on real planets).
//!
//! Essentials no longer pull the spawn toward them (Ben, 2026-10-05: "I don't believe we need spawn
//! selectoin since we have some passive iron gen like vanilla"): spawn is the standard dry, flat
//! land nearest (0, 0) (`synth::spawn_candidates`).
//!
//! The essentials list is a pack's to define: the game passes its `deeptime:essentials` registry
//! (P1-15) in the content bundle (`{"essentials": [...]}`), which replaces the preset's list
//! (`crate::preset::Preset::with_content`); runs without a bundle (the CLI) use the preset's.

use deeptime_format::cube_map::CubeView;
use deeptime_format::projection::{IndexFrame, WorldProjection};
use deeptime_format::{Product, Projection};
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
    /// Only a site whose product of the material has the role `primary` counts toward the regional
    /// minimum (default true); off, a co-product or by-product counts too.
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
            .any(|p| self.produced_as(&p.material, &p.role))
    }

    /// Whether a product of `material` in the role `role` counts toward this essential (the
    /// guarantee step's producers are the types with such a product, so the deposit it forces
    /// is one that counts).
    pub fn produced_as(&self, material: &str, role: &str) -> bool {
        material == self.material && (!self.primary_only || role == "primary")
    }
}

/// The region grid of an essential: squares of `size` blocks on the index frame of the world the
/// game plays (`deeptime_format::projection::IndexFrame`: its origin and span), x wrapping with
/// the frame's period. A cube world's frame is not the simulation's Lambert one, so the
/// playability pass counts its regions in the world's frame ([`WorldFrame`]), as the player meets
/// them (ledger: DEP-220).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Regions {
    /// The x period, blocks.
    pub c: f64,
    pub size: f64,
    pub cols: u32,
    pub rows: u32,
    /// The frame's origin, blocks.
    pub origin: (f64, f64),
}

impl Regions {
    /// The regions of a Lambert planet of circumference `c` (the simulation's frame).
    pub fn new(c: u32, region_blocks: u32) -> Regions {
        Regions::in_frame(IndexFrame::lambert(c), region_blocks)
    }

    /// The regions of a world laid out in index frame `frame` (`WorldProjection::index_frame`).
    pub fn in_frame(frame: IndexFrame, region_blocks: u32) -> Regions {
        let rb = region_blocks.max(64);
        let (span_x, span_z) = frame.span;
        Regions {
            c: span_x as f64,
            size: f64::from(rb),
            cols: (span_x.max(1) as u32).div_ceil(rb),
            rows: (span_z.max(1) as u32).div_ceil(rb),
            origin: (frame.origin.0 as f64, frame.origin.1 as f64),
        }
    }

    /// The region (col, row) of block (x, z).
    pub fn of(&self, x: f64, z: f64) -> (u32, u32) {
        let (ox, oz) = self.origin;
        // x wraps with the frame's period, which need not be a whole number of regions (the
        // last column is then narrower).
        let col = (((x - ox).rem_euclid(self.c) / self.size).floor() as i64)
            .clamp(0, i64::from(self.cols) - 1);
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

/// The world a planet's simulation becomes, as the playability pass needs it: where a simulation
/// block lands in the world the game plays, and that world's region grid. A Lambert world (the
/// synthetic worlds) is the simulation's own frame; a cube world is the conformal cube turned as
/// the turn search forecasts it (`bake::cube_turn`, SCL-129), so an essential's regions are the
/// ones the player walks, squares of the net's window (those in the void hold no land).
// Copy on purpose, as `WorldProjection`: made once per run.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Copy, Debug)]
pub enum WorldFrame {
    Lambert(Projection),
    Cube {
        view: CubeView<'static>,
        src: Projection,
    },
}

impl WorldFrame {
    /// The world block of simulation block (x, z).
    pub fn to_world(&self, x: f64, z: f64) -> (f64, f64) {
        match self {
            WorldFrame::Lambert(_) => (x, z),
            WorldFrame::Cube { view, src } => view.point_of_unit(src.unit(x, z)),
        }
    }

    /// The world's projection.
    pub fn projection(&self) -> WorldProjection {
        match self {
            WorldFrame::Lambert(p) => WorldProjection::Lambert(*p),
            WorldFrame::Cube { view, .. } => WorldProjection::Cube(*view),
        }
    }

    /// The world's region grid of `region_blocks`.
    pub fn regions(&self, region_blocks: u32) -> Regions {
        Regions::in_frame(self.projection().index_frame(), region_blocks)
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
        // A Lambert world is the simulation's own frame.
        assert_eq!(
            WorldFrame::Lambert(Projection::new(16_384)).to_world(5.0, 7.0),
            (5.0, 7.0)
        );
    }
}

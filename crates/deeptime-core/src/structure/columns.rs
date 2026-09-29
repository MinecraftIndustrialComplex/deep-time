// SPDX-License-Identifier: LGPL-3.0-or-later
//! Column strata (plan §8.5, Phase 1): one L1 cell's rock column from the surface down, as units
//! of (class, thickness). Everything below the surface reads it — the horizon bake
//! (`bake_horizons.rs`), the host-rock fractions and trap structure the deposit engine gets
//! (`hosts.rs`, `traps.rs`), the rock map and the cross-sections.
//!
//! Top-down:
//! 1. **Regolith** (the surface's soil, saprolite, laterite or bauxite), as thick as it is.
//! 2. **The strata log** (P1-04), youngest layer first, with the classes the metamorphism of
//!    `metamorphism.rs` left in it. **Lithification** is decided here: the part of a layer that
//!    lies within `loose_depth_m` of the surface and whose base is younger than `loose_age_myr` is
//!    shown loose (the class's `loose_equivalent`: sandstone → sand, shale → mud; a loose class such
//!    as a buried soil stays itself); the rest is rock (a loose class's `lithifies_to`: buried soil →
//!    mudstone, laterite → ironstone). The facies rules lay down rock classes, so the log itself is
//!    never rewritten for this.
//! 3. **The basement**: the plate model's rock pools under the cell (`plates/pool/*`, P1-03's
//!    tectonics.js pools), top-down in `plates::l1::TOP_ORDER`, less the strata log and regolith,
//!    which the pools already hold (the surface's deposits join the sediment pool, and its erosion
//!    strips the pools from the top). Each pool is one unit whose class follows the surface's
//!    basement rule (`surface::basement_class`, so a column shows at its top what the surface
//!    eroded): sediment → mudstone (on the ocean floor pelagic chalk above the carbonate
//!    compensation depth, pelagic clay as mudstone below it), sedimentary → sandstone (shale on
//!    oceanic crust), felsic volcanic → rhyolite (andesite in arcs), mafic volcanic → basalt,
//!    metamorphic → gneiss (schist in a Himalayan orogen younger than 100 Myr), felsic plutonic →
//!    granite (granodiorite under an Andean orogeny), mafic plutonic → gabbro. The first basement
//!    unit takes the surface's regime overrides (arc → andesite, hotspot → basalt).
//! 4. **The mantle** (peridotite) below the crust, down to the depth asked for.
//!
//! Neighbouring units of one class merge. Plutons are bodies, not units (`plutons.rs`).

use crate::plates::l1::TOP_ORDER;
use crate::plates::state::{OCEANIC, OROGENY_ANDEAN, OROGENY_HIMALAYAN, pool};
use crate::surface::lithology::{self, Classes, index};
use crate::surface::strata_log::Layer;
use crate::surface::{basement_class, weathering};

use super::params::StructureParams;

/// Depth of the carbonate compensation depth, m below sea level (4.5–5 km in today's oceans).
pub const CCD_M: f64 = 4500.0;

/// Where a unit comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Regolith,
    /// A strata-log layer (P1-04's deposits).
    Log,
    /// A rock pool of the plate model (`plates::state::pool`).
    Basement(u8),
    Mantle,
}

/// One unit of a column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Unit {
    /// Index into `synth::EVERY_CLASS`.
    pub class: u16,
    pub thickness_m: f32,
    pub source: Source,
}

/// The class tables the columns need, resolved once.
#[derive(Clone, Debug)]
pub struct Tables {
    pub cls: Classes,
    pub loose: Vec<Option<u16>>,
    pub lithified: Vec<Option<u16>>,
    pub peridotite: u16,
    pub chalk: u16,
    pub diorite: u16,
    pub hornfels: u16,
    pub skarn: u16,
    pub halite: u16,
    pub marble: u16,
    /// Per class: in the carbonate group.
    pub carbonate: Vec<bool>,
    /// Per class: crystalline (intrusive, special or regional-metamorphic above slate grade).
    pub crystalline: Vec<bool>,
}

impl Tables {
    pub fn get() -> Tables {
        let t = lithology::table();
        let group = |c: &lithology::RockClass, g: &str| c.groups.iter().any(|x| x.ends_with(g));
        Tables {
            cls: Classes::get(),
            loose: t.iter().map(|c| c.loose_equivalent).collect(),
            lithified: t.iter().map(|c| c.lithifies_to).collect(),
            peridotite: index("peridotite"),
            chalk: index("chalk"),
            diorite: index("diorite"),
            hornfels: index("hornfels"),
            skarn: index("skarn"),
            halite: index("halite"),
            marble: index("marble"),
            carbonate: t.iter().map(|c| group(c, "/carbonate")).collect(),
            crystalline: t
                .iter()
                .map(|c| {
                    group(c, "/intrusive")
                        || c.family == "special_host"
                        || (c.family == "metamorphic"
                            && !matches!(c.name, "slate" | "phyllite" | "serpentinite"))
                })
                .collect(),
        }
    }

    /// The regolith class of `weathering::{SOIL, SAPROLITE, LATERITE, BAUXITE}`.
    pub fn regolith_class(&self, kind: u8) -> u16 {
        match kind {
            weathering::SAPROLITE => self.cls.saprolite,
            weathering::LATERITE => self.cls.laterite,
            weathering::BAUXITE => self.cls.bauxite,
            _ => self.cls.soil,
        }
    }

    /// How `class` shows while young and shallow.
    pub fn loose_form(&self, class: u16) -> u16 {
        if self.lithified[usize::from(class)].is_some() {
            class
        } else {
            self.loose[usize::from(class)].unwrap_or(class)
        }
    }

    /// How `class` shows once buried or old.
    pub fn rock_form(&self, class: u16) -> u16 {
        self.lithified[usize::from(class)].unwrap_or(class)
    }

    /// Whether `class` is a loose (unconsolidated) class.
    pub fn is_loose(&self, class: u16) -> bool {
        self.lithified[usize::from(class)].is_some() || class == self.cls.bauxite
    }
}

/// What one cell's column is made of.
#[derive(Clone, Copy, Debug)]
pub struct Cell<'a> {
    pub regolith_m: f32,
    pub regolith_kind: u8,
    /// The strata log's layers, bottom first.
    pub log: &'a [Layer],
    /// The plate model's rock pools, metres, in `pool` order.
    pub pools: [f32; pool::COUNT],
    pub crust_type: u8,
    pub regime: u8,
    pub orogeny: u8,
    /// Myr since the last orogeny (NaN for none).
    pub orogeny_age: f32,
    /// Depth of the surface below sea level, m (negative on land).
    pub depth_below_sea_m: f64,
}

/// The class of pool `p` below the top of the basement.
pub fn pool_class(t: &Tables, cell: &Cell<'_>, p: usize) -> u16 {
    let c = &t.cls;
    let oceanic = cell.crust_type == OCEANIC;
    let young = |myr: f32| cell.orogeny_age.is_finite() && cell.orogeny_age < myr;
    match p {
        pool::SEDIMENT => {
            if oceanic && cell.depth_below_sea_m < CCD_M {
                t.chalk
            } else {
                c.mudstone
            }
        }
        pool::SEDIMENTARY => {
            if oceanic {
                c.shale
            } else {
                c.sandstone
            }
        }
        pool::FELSIC_VOLCANIC => {
            if cell.regime == crate::plates::state::regime::ARC
                || (cell.orogeny == OROGENY_ANDEAN && young(30.0))
            {
                c.andesite
            } else {
                c.rhyolite
            }
        }
        pool::MAFIC_VOLCANIC => c.basalt,
        pool::METAMORPHIC => {
            if cell.orogeny == OROGENY_HIMALAYAN && young(100.0) {
                c.schist
            } else {
                c.gneiss
            }
        }
        pool::FELSIC_PLUTONIC => {
            if cell.orogeny == OROGENY_ANDEAN {
                c.granodiorite
            } else {
                c.granite
            }
        }
        pool::MAFIC_PLUTONIC => c.gabbro,
        _ => t.peridotite,
    }
}

fn push(out: &mut Vec<Unit>, class: u16, thickness_m: f64, source: Source) {
    if !(thickness_m > 0.0) {
        return;
    }
    if let Some(last) = out.last_mut()
        && last.class == class
    {
        last.thickness_m += thickness_m as f32;
        return;
    }
    out.push(Unit {
        class,
        thickness_m: thickness_m as f32,
        source,
    });
}

/// Builds `cell`'s column into `out` (top-down) at time `now_myr`, down to at least `max_depth_m`
/// (the mantle fills whatever the crust leaves).
pub fn build(
    cell: &Cell<'_>,
    t: &Tables,
    p: &StructureParams,
    now_myr: f64,
    max_depth_m: f64,
    out: &mut Vec<Unit>,
) {
    out.clear();
    let mut depth = 0.0f64;
    let reg = f64::from(cell.regolith_m.max(0.0));
    if reg > 0.0 {
        push(out, t.regolith_class(cell.regolith_kind), reg, Source::Regolith);
        depth += reg;
    }
    let young_from = now_myr - p.loose_age_myr;
    let mut log_total = 0.0f64;
    for l in cell.log.iter().rev() {
        let th = f64::from(l.thickness_m);
        if !(th > 0.0) {
            continue;
        }
        log_total += th;
        if depth >= max_depth_m {
            depth += th;
            continue;
        }
        let young = f64::from(l.age_myr) >= young_from;
        let loose = if young {
            (p.loose_depth_m - depth).clamp(0.0, th)
        } else {
            0.0
        };
        push(out, t.loose_form(l.class), loose, Source::Log);
        push(out, t.rock_form(l.class), th - loose, Source::Log);
        depth += th;
    }

    // The basement: the pools less what the log and regolith already stand for.
    let mut pools = cell.pools.map(|x| f64::from(x.max(0.0)));
    let mut cover = log_total + reg;
    for k in [pool::SEDIMENT, pool::SEDIMENTARY] {
        let take = cover.min(pools[k]);
        pools[k] -= take;
        cover -= take;
    }
    let mut first = true;
    for &k in &TOP_ORDER {
        let th = pools[k];
        if th < 1.0 {
            continue;
        }
        if depth >= max_depth_m {
            break;
        }
        let class = if first && cell.crust_type != OCEANIC {
            basement_class(
                &t.cls,
                cell.crust_type,
                cell.regime,
                cell.orogeny,
                cell.orogeny_age,
                k as u8,
            )
        } else {
            pool_class(t, cell, k)
        };
        first = false;
        push(out, class, th, Source::Basement(k as u8));
        depth += th;
    }
    if depth < max_depth_m {
        push(out, t.peridotite, max_depth_m - depth, Source::Mantle);
    }
}

/// Total thickness of `units`, m.
pub fn total(units: &[Unit]) -> f64 {
    units.iter().map(|u| f64::from(u.thickness_m)).sum()
}

/// The class at depth `z` (m below the surface) of `units`, or the last unit's below them.
pub fn class_at(units: &[Unit], z: f64) -> Option<u16> {
    let mut top = 0.0;
    for u in units {
        let bottom = top + f64::from(u.thickness_m);
        if z < bottom {
            return Some(u.class);
        }
        top = bottom;
    }
    units.last().map(|u| u.class)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plates::state::{CONTINENTAL, regime};

    fn layer(age: f32, th: f32, class: &str) -> Layer {
        Layer {
            age_myr: age,
            thickness_m: th,
            class: index(class),
            provenance: 0,
        }
    }

    fn continental<'a>(log: &'a [Layer]) -> Cell<'a> {
        Cell {
            regolith_m: 0.0,
            regolith_kind: weathering::SOIL,
            log,
            pools: crate::plates::pools::continental_column(),
            crust_type: CONTINENTAL,
            regime: regime::CRATON,
            orogeny: 0,
            orogeny_age: f32::NAN,
            depth_below_sea_m: -200.0,
        }
    }

    #[test]
    fn young_shallow_sediment_is_loose_and_buried_regolith_lithifies() {
        let t = Tables::get();
        let p = StructureParams::default();
        // Bottom first: an old buried soil, old sandstone, then young sandstone 400 m thick.
        let log = [
            layer(-300.0, 50.0, "soil"),
            layer(-200.0, 300.0, "sandstone"),
            layer(-5.0, 400.0, "shale"),
        ];
        let mut out = Vec::new();
        build(&continental(&log), &t, &p, 0.0, 12_000.0, &mut out);
        // Young shale: its top 150 m show as mud, the rest as shale.
        assert_eq!(out[0].class, index("mud"));
        assert!((out[0].thickness_m - 150.0).abs() < 1e-3);
        assert_eq!(out[1].class, index("shale"));
        assert!((out[1].thickness_m - 250.0).abs() < 1e-3);
        // Old sandstone stays rock; the buried soil lithifies to mudstone.
        assert_eq!(out[2].class, index("sandstone"));
        assert_eq!(out[3].class, index("mudstone"));
        // The basement: the pools less the log (750 m) off the sediment pools, first unit by the
        // surface's rule (sedimentary → sandstone, merging with nothing: the soil sits between).
        assert_eq!(out[4].source, Source::Basement(pool::SEDIMENTARY as u8));
        let sed = crate::plates::pools::continental_column();
        let cover = f64::from(sed[pool::SEDIMENT] + sed[pool::SEDIMENTARY]) - 750.0;
        assert!((f64::from(out[4].thickness_m) - cover).abs() < 1e-2);
        assert!(total(&out) >= 12_000.0 - 1e-3);
        assert_eq!(class_at(&out, 10.0), Some(index("mud")));
        // An old log shows no loose top.
        let old = [layer(-100.0, 300.0, "sandstone")];
        build(&continental(&old), &t, &p, 0.0, 5000.0, &mut out);
        assert_eq!(out[0].class, index("sandstone"));
    }

    #[test]
    fn oceanic_columns_have_pelagic_sediment_basalt_gabbro_and_mantle() {
        let t = Tables::get();
        let p = StructureParams::default();
        let mut pools = crate::plates::pools::oceanic_column();
        pools[pool::SEDIMENT] = 300.0;
        let mut cell = continental(&[]);
        cell.pools = pools;
        cell.crust_type = OCEANIC;
        cell.regime = regime::OCEAN_BASIN;
        cell.depth_below_sea_m = 3000.0;
        let mut out = Vec::new();
        build(&cell, &t, &p, 0.0, 10_000.0, &mut out);
        let classes: Vec<u16> = out.iter().map(|u| u.class).collect();
        assert_eq!(
            classes,
            [
                index("chalk"),
                index("basalt"),
                index("gabbro"),
                index("peridotite")
            ]
        );
        cell.depth_below_sea_m = 5500.0;
        build(&cell, &t, &p, 0.0, 10_000.0, &mut out);
        assert_eq!(out[0].class, index("mudstone"), "below the CCD");
    }
}

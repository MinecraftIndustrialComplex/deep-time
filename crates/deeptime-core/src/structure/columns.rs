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
//!    eroded). Each crust kind has its own classes (ledger: RCK-096, from-memory):
//!    - **continental**: sediment → mudstone, sedimentary → sandstone, felsic volcanic →
//!      rhyolite (andesite in arcs), mafic volcanic → basalt, metamorphic → gneiss (schist in a
//!      Himalayan orogen younger than 100 Myr), felsic plutonic → granite (granodiorite under an
//!      Andean orogeny), mafic plutonic → gabbro; the first basement unit takes the surface's
//!      regime overrides (arc → andesite, hotspot → basalt);
//!    - **oceanic** (the Penrose sequence): sediment → pelagic chalk above the carbonate
//!      compensation depth, pelagic clay as mudstone below it; sedimentary → shale; the mafic
//!      volcanic pool (seismic layer 2) → basalt lavas over the sheeted dike complex, greenstone
//!      (greenschist-facies diabase; [`OCEAN_LAVA_SHARE`]); mafic plutonic → gabbro (layer 3);
//!      then the mantle. Felsic and metamorphic pools on ocean crust are local rock: an island
//!      arc's (andesite, its tonalitic middle crust as diorite under an active arc and as
//!      granodiorite where the arc has died), the thinned continental crust a margin's
//!      transition moved to its ocean side (granodiorite), or an accreted or deeply buried
//!      sediment pile's (schist). The plate model blends the rock pools only within one crust
//!      kind and does not blend ocean crust's felsic and metamorphic pools at all
//!      (`plates::ridges`, `plates::l1`), so a continent's granite does not spread into the sea
//!      floor beside it.
//!
//!    The pools have no burial history, so on continental crust a pool unit steps along its
//!    class's metamorphic path by the **present** geotherm (`metamorphism.rs`'s temperatures and
//!    geotherm, without the collision factor): the unit is split at the depths of the isotherms
//!    of its path, so a thick sedimentary pool is sandstone near the surface and quartzite below
//!    the 300 °C isotherm.
//! 4. **The mantle** (peridotite) below the crust, down to the depth asked for.
//!
//! Neighbouring units of one class merge. Plutons are bodies, not units (`plutons.rs`).
//!
//! **A column never gets cooler downwards.** The strata log keeps what its layers became at their
//! deepest burial, and the pools know only today's geotherm, so a pool unit under a metamorphosed
//! log layer would otherwise be unmetamorphosed rock under schist. The rock under a layer was at
//! least as hot as the layer whenever both were there, so every unit takes at least the grade of
//! the hottest formation temperature above it: a sedimentary or volcanic pool steps along its path
//! as far as that temperature (or the present geotherm, whichever is hotter) allows. A plutonic
//! pool is taken to be younger than the metamorphism of its roof and stays what it is: gabbro
//! under gneiss is gabbro, not greenstone, and only the step of the highest grade (granite to
//! gneiss at 650 °C) is taken, at its isotherm.
//!
//! **Stratigraphic keys** ([`key`]). Every unit carries the span of a key that orders units the
//! same way in every column, so the column evaluator can correlate one cell's horizons with its
//! neighbours' (FORMAT.md, "`horizon_stack` cells"): a key is a u16 that grows downwards. The key
//! space is ordered by **metamorphic grade first**: the regolith, then one band per formation
//! temperature ([`key::BAND_FROM_C`]: unmetamorphosed, then 250, 300, 350, 500, 525 and 650 °C),
//! then the mantle. A unit's band is that of the hottest formation temperature at or above it in
//! its own column, so bands never decrease downwards in a column; plutonic pool units are
//! crystalline basement and always lie in the last band, with the gneiss. Inside a band come the strata
//! log's layers by age (a layer spans the ages from the base of the layer above it to its own
//! base) and then the basement pools in erosion order (a slot per pool and metamorphic step).
//! Two cells' layers of one deposition time and one grade get the same keys wherever the log
//! holds them, so a bed stays one bed across cells and pinches out where a cell lacks it — and
//! whatever blend of neighbouring columns the bake's smoothing and the column rule make, ordered
//! by key, has the grade rising downwards: a metamorphic belt's rocks lie under its neighbours'
//! unmetamorphosed cover and over everyone's crystalline basement, never as schist on shale.
//! (Before, the log was keyed by age alone, and a belt's schist was correlated with the shale its
//! neighbours laid down at the same time: 57 % of a planet's columns had a grade inversion.)

use crate::plates::l1::TOP_ORDER;
use crate::plates::state::{OCEANIC, OROGENY_ANDEAN, OROGENY_HIMALAYAN, pool};
use crate::surface::lithology::{self, Classes, index};
use crate::surface::strata_log::Layer;
use crate::surface::{basement_class, weathering};

use super::metamorphism::Paths;
use super::params::StructureParams;

/// Depth of the carbonate compensation depth, m below sea level. Game choice: one value for every
/// ocean. Today's CCD is about 4.2–4.5 km in the Pacific (about 5 km under the equatorial
/// upwelling), about 4.3 km in the Indian Ocean and 5–6 km in the Atlantic (secondary sources:
/// Wikipedia, SEPM Strata, LibreTexts), so 4,500 m is the Pacific figure.
// ledger: RCK-015 (game choice)
pub const CCD_M: f64 = 4500.0;

/// The share of ocean crust's mafic volcanic pool (seismic layer 2) that is lavas, pillows and
/// flows; the rest is the sheeted dike complex under them. Drilled sections (Alt et al. 1996;
/// IODP Exp. 309/312): Hole 504B (Costa Rica Rift, intermediate rate) 571.5 m of lavas and a
/// 209 m transition over 1050 m of dikes, not bottomed (lavas 0.31 to 0.37 of the section); Hole
/// 1256D (superfast) 754 m of lavas and a 57 m transition over 346 m of dikes (0.65). 0.4 of a
/// fresh ridge's 2 km layer 2 (`plates::pools::oceanic_column`) gives 0.8 km of lavas over
/// 1.2 km of dikes; erosion takes the lavas first and later volcanism adds to them.
// ledger: RCK-095 (game-choice of the share inside the drilled range; the sections verified)
pub const OCEAN_LAVA_SHARE: f64 = 0.4;

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

/// Stratigraphic keys: where a unit sits in the order every column shares (the module docs).
pub mod key {
    /// The regolith's span.
    pub const REGOLITH: [u16; 2] = [0, 4095];
    /// Grade bands: band b holds the units whose hottest formation temperature (their own or of
    /// anything above them in their column) is at least `BAND_FROM_C[b]` and below the next
    /// (`metamorphism::formation_temp_c`: slate 250, quartzite, marble and greenstone 300,
    /// phyllite 350, amphibolite 500, schist 525, gneiss 650 °C).
    // ledger: RCK-052, STR-018, STR-019 (game choice)
    pub const BAND_FROM_C: [f64; BANDS] = [0.0, 250.0, 300.0, 350.0, 500.0, 525.0, 650.0];
    pub const BANDS: usize = 7;
    /// Band 0's strata log: key `LOG_TOP` is the present, `LOG_BASE` the oldest age of the run.
    pub const LOG_TOP: u16 = 4096;
    pub const LOG_BASE: u16 = 32767;
    /// Keys of one pool slot (one pool at one metamorphic step, in one band): one band of the
    /// column rule's tongues at its default (`deeptime_format::strata::BAND_KEYS`).
    pub const SLOT: u16 = 64;
    /// Slots a pool has in a band (metamorphic steps past the fourth share the last).
    pub const STEPS: u16 = 4;
    /// A pool's keys in a band; pool q of `plates::l1::TOP_ORDER` follows pool q − 1.
    pub const POOL_BLOCK: u16 = SLOT * STEPS;
    /// The pools' keys in a band (seven pools).
    pub const POOLS: u16 = 7 * POOL_BLOCK;
    /// Band 0's first pool key.
    pub const POOL0: u16 = 32768;
    /// The first metamorphic band's first key; band b ≥ 1 starts at `META0 + (b − 1)·META_BAND`:
    /// `META_LOG` keys of strata log by age, then the pools.
    pub const META0: u16 = POOL0 + POOLS;
    pub const META_LOG: u16 = 2688;
    pub const META_BAND: u16 = META_LOG + POOLS;
    /// The mantle's span.
    pub const MANTLE: [u16; 2] = [61440, 65535];

    /// The band of a formation temperature, °C.
    pub fn band_of(formed_at_c: f64) -> usize {
        BAND_FROM_C
            .iter()
            .rposition(|&t| formed_at_c >= t)
            .unwrap_or(0)
    }

    /// The strata log's keys in band `band`: [the present, the oldest age of the run].
    pub fn log_range(band: usize) -> [u16; 2] {
        if band == 0 {
            [LOG_TOP, LOG_BASE]
        } else {
            let lo = META0 + (band.min(BANDS - 1) as u16 - 1) * META_BAND;
            [lo, lo + META_LOG - 1]
        }
    }

    /// The key in band `band` of an age `delta_myr` before the present in a log whose oldest
    /// layer can be `span_myr` old: linear across the band's log range.
    pub fn of_age(band: usize, delta_myr: f64, span_myr: f64) -> u16 {
        let f = if span_myr > 0.0 {
            (delta_myr / span_myr).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let [lo, hi] = log_range(band);
        (f64::from(lo) + f * f64::from(hi - lo)).round() as u16
    }

    /// Pool `q`'s (position in `TOP_ORDER`) span at metamorphic step `m` in band `band`.
    pub fn pool(band: usize, q: usize, m: usize) -> [u16; 2] {
        let lo = log_range(band)[1]
            + 1
            + q.min(6) as u16 * POOL_BLOCK
            + m.min(usize::from(STEPS) - 1) as u16 * SLOT;
        [lo, lo + SLOT - 1]
    }

    /// The part of the key space `k` lies in: 0 the regolith, 1 + 2b band b's strata log,
    /// 2 + 2b band b's pools, 15 the mantle. Units of different domains are never one horizon:
    /// a horizon's span stays inside one domain, so its height is never spread over keys that
    /// other columns give to other rocks.
    pub fn domain(k: u16) -> u8 {
        if k < LOG_TOP {
            0
        } else if k >= MANTLE[0] {
            15
        } else if k < POOL0 {
            1
        } else if k < META0 {
            2
        } else {
            let b = (k - META0) / META_BAND;
            let log = (k - META0) % META_BAND < META_LOG;
            (1 + 2 * (b + 1) + u16::from(!log)) as u8
        }
    }

    /// Whether a unit spanning `b` may join the horizon spanning `a` above it: both in one
    /// domain.
    pub fn joins(a: [u16; 2], b: [u16; 2]) -> bool {
        domain(a[1]) == domain(b[0]) && domain(a[0]) == domain(b[1])
    }
}

/// One unit of a column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Unit {
    /// Index into `synth::EVERY_CLASS`.
    pub class: u16,
    pub thickness_m: f32,
    pub source: Source,
    /// Its stratigraphic key span [top, base] ([`key`]).
    pub key: [u16; 2],
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
    /// The sheeted dikes of ocean crust (greenschist-facies diabase, [`OCEAN_LAVA_SHARE`]).
    pub greenstone: u16,
    pub hornfels: u16,
    pub skarn: u16,
    pub halite: u16,
    pub marble: u16,
    /// Per class: in the carbonate group.
    pub carbonate: Vec<bool>,
    /// Per class: crystalline (intrusive, special or regional-metamorphic above slate grade).
    pub crystalline: Vec<bool>,
    /// Per class: the least temperature a regional-metamorphic class formed at, °C (0 for every
    /// other class; `realism::Rules` reads the same).
    pub formed_at_c: Vec<f64>,
    /// Per class: intrusive rock (the intrusive and special-host families), which may be younger
    /// than the metamorphism of the rock above it.
    pub intrusive: Vec<bool>,
    /// Per class: the least thickness of a horizon inside a stack, blocks
    /// (`bake_horizons`): [`METAMORPHIC_MIN_BLOCKS`] for regional-metamorphic rock,
    /// [`PLUTONIC_MIN_BLOCKS`] for plutonic rock, one block for the rest.
    pub min_blocks: Vec<i32>,
    /// Per class: unconsolidated (the regolith classes and the loose forms of young sediment).
    pub unconsolidated: Vec<bool>,
    pub paths: Paths,
}

/// The thinnest horizon of regional-metamorphic rock inside a stack, blocks: metamorphic rock
/// is a regional body, not a sheet between other rocks. Game choice. Real thin bodies exist
/// (an ophiolite's metamorphic sole is about 200 m thick, Hacker's lecture notes; skarn zones
/// run from 3 cm to 15 m, Nelson), and inverted sequences occur in thrust zones (Mottram et al.
/// 2014); eight blocks (160 m) keeps the look of a regional grade.
// ledger: STR-006, STR-045 (game choice)
pub const METAMORPHIC_MIN_BLOCKS: i32 = 8;
/// The thinnest horizon of plutonic rock inside a stack, blocks.
// ledger: STR-007, STR-046 (game choice)
pub const PLUTONIC_MIN_BLOCKS: i32 = 16;

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
            greenstone: index("greenstone"),
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
            formed_at_c: t
                .iter()
                .map(|c| {
                    super::metamorphism::formation_temp_c(c.name)
                        .filter(|&x| {
                            c.family == "metamorphic" && x >= super::realism::METAMORPHIC_FROM_C
                        })
                        .unwrap_or(0.0)
                })
                .collect(),
            intrusive: t
                .iter()
                .map(|c| c.family == "intrusive_igneous" || c.family == "special_host")
                .collect(),
            min_blocks: t
                .iter()
                .map(|c| {
                    if c.family == "metamorphic"
                        && super::metamorphism::formation_temp_c(c.name)
                            .is_some_and(|x| x >= super::realism::METAMORPHIC_FROM_C)
                    {
                        METAMORPHIC_MIN_BLOCKS
                    } else if c.family == "intrusive_igneous" && c.name != "peridotite" {
                        PLUTONIC_MIN_BLOCKS
                    } else {
                        1
                    }
                })
                .collect(),
            unconsolidated: t.iter().map(|c| c.family == "unconsolidated").collect(),
            paths: Paths::get(),
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
    /// The present geotherm: surface temperature, °C, and gradient, °C/m (0 turns the basement's
    /// metamorphism off).
    pub surface_c: f64,
    pub gradient_c_m: f64,
    /// How old the oldest log layer of the run can be, Myr (the log's key scale, [`key::of_age`]).
    pub log_span_myr: f64,
    /// Whether the cell's surface is still accumulating (net deposition at the last step, over
    /// the cell and its neighbours): its top log layer then reaches the present in key space;
    /// otherwise erosion has taken an unknown part of it and it keeps only its base age.
    pub top_active: bool,
}

/// The class of pool `p` below the top of the basement (ledger RCK-016 to RCK-019: the pool →
/// class table is a game choice; the age gates 30 and 100 Myr are the plate model's orogeny clocks).
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
        // On ocean crust the felsic and metamorphic pools are local rock (`plates::ridges`): an
        // island arc's andesite over its tonalitic middle crust (diorite under an active arc,
        // granodiorite where the arc has died or the rock is the thinned continental crust a
        // margin's transition moved to its ocean side), and schist, the metasediment of a thick
        // sediment pile or an accreted wedge: never granite, rhyolite or gneiss (ledger: RCK-096,
        // from-memory: arc middle crust as tonalitic, not read).
        pool::FELSIC_VOLCANIC => {
            if oceanic
                || cell.regime == crate::plates::state::regime::ARC
                || (cell.orogeny == OROGENY_ANDEAN && young(30.0))
            {
                c.andesite
            } else {
                c.rhyolite
            }
        }
        pool::MAFIC_VOLCANIC => c.basalt,
        pool::METAMORPHIC => {
            if oceanic || (cell.orogeny == OROGENY_HIMALAYAN && young(100.0)) {
                c.schist
            } else {
                c.gneiss
            }
        }
        pool::FELSIC_PLUTONIC => {
            if oceanic && cell.regime == crate::plates::state::regime::ARC {
                t.diorite
            } else if oceanic || cell.orogeny == OROGENY_ANDEAN {
                c.granodiorite
            } else {
                c.granite
            }
        }
        pool::MAFIC_PLUTONIC => c.gabbro,
        _ => t.peridotite,
    }
}

fn push(out: &mut Vec<Unit>, class: u16, thickness_m: f64, source: Source, key: [u16; 2]) {
    if thickness_m.is_nan() || thickness_m <= 0.0 {
        return;
    }
    if let Some(last) = out.last_mut()
        && last.class == class
        && key::joins(last.key, key)
    {
        last.thickness_m += thickness_m as f32;
        last.key[1] = last.key[1].max(key[1]);
        return;
    }
    out.push(Unit {
        class,
        thickness_m: thickness_m as f32,
        source,
        key,
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
        push(
            out,
            t.regolith_class(cell.regolith_kind),
            reg,
            Source::Regolith,
            key::REGOLITH,
        );
        depth += reg;
    }
    let young_from = now_myr - p.loose_age_myr;
    let mut log_total = 0.0f64;
    // The hottest formation temperature so far, from the top (the module docs).
    let mut hot = 0.0f64;
    // A layer spans the ages from the base of the layer above it to its own base, in its band;
    // the top layer reaches the present while the cell still accumulates, else it keeps only its
    // base age.
    let age_key =
        |band: usize, age: f32| key::of_age(band, now_myr - f64::from(age), cell.log_span_myr);
    // The layer above: (its band, its base key, its base age).
    let mut above: Option<(usize, u16, f32)> = None;
    for l in cell.log.iter().rev() {
        let th = f64::from(l.thickness_m);
        if th.is_nan() || th <= 0.0 {
            continue;
        }
        log_total += th;
        // The log's layers are already what their burial made them; a class without a path of
        // its own that shows as one with a path (a buried soil shows as mudstone) follows the
        // rock above it.
        let rock = t.paths.at(t.rock_form(l.class), hot);
        hot = hot.max(t.formed_at_c[usize::from(rock)]);
        let band = key::band_of(hot);
        let [lo, hi] = key::log_range(band);
        let base = age_key(band, l.age_myr);
        let kt = match above {
            Some((b, k, _)) if b == band => k,
            Some((_, _, age)) => age_key(band, age),
            None if cell.top_active => lo,
            None => base.saturating_sub(2).max(lo),
        }
        .min(hi - 2);
        let kb = base.max(kt + 2).min(hi);
        above = Some((band, kb, l.age_myr));
        if depth >= max_depth_m {
            depth += th;
            continue;
        }
        let km = kt + (kb - kt) / 2;
        let young = f64::from(l.age_myr) >= young_from;
        let loose = if young {
            (p.loose_depth_m - depth).clamp(0.0, th)
        } else {
            0.0
        };
        push(out, t.loose_form(l.class), loose, Source::Log, [kt, km]);
        push(out, rock, th - loose, Source::Log, [km, kb]);
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
    // The band of the last pool unit: bands never decrease downwards.
    let mut band = key::band_of(hot);
    for (q, &k) in TOP_ORDER.iter().enumerate() {
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
        // Split the pool where its class steps along its path: at the isotherms of the present
        // geotherm (continental crust only), and at once for every step the rock above was
        // already hot enough for; a plutonic pool passes over the steps cooler than its roof
        // (the module docs). Each step keeps its own slot of the pool's keys in its band.
        let geotherm = cell.crust_type != OCEANIC && cell.gradient_c_m > 0.0;
        let isotherm = |temp: f64| {
            if geotherm {
                (temp - cell.surface_c) / cell.gradient_c_m
            } else {
                f64::INFINITY
            }
        };
        let intrusive = t.intrusive[usize::from(class)];
        let src = Source::Basement(k as u8);
        let bottom = depth + th;
        let mut top = depth;
        let mut current = class;
        let mut step = 0;
        let mut emit = |out: &mut Vec<Unit>, class: u16, th: f64, step: usize, hot: &mut f64| {
            *hot = hot.max(t.formed_at_c[usize::from(class)]);
            // Plutonic rock is crystalline basement: the last band, whatever lies on it.
            band = band.max(if t.intrusive[usize::from(class)] {
                key::BANDS - 1
            } else {
                key::band_of(*hot)
            });
            push(out, class, th, src, key::pool(band, q, step));
        };
        // Ocean crust's layer 2: the lavas over the sheeted dikes, which the ridge's hydrothermal
        // circulation altered to greenschist facies as they formed (the dikes are the basalt
        // pool's first step, greenstone, so they share its keys with metamorphosed basalt
        // elsewhere). Either goes further along the path when the rock above was hotter.
        if cell.crust_type == OCEANIC && k == pool::MAFIC_VOLCANIC && class == t.cls.basalt {
            let steps = t.paths.steps(class);
            // How far along the path the roof's temperature takes the rock.
            let past = |hot: f64| steps.iter().take_while(|&&(_, temp)| temp <= hot).count();
            let class_of = |m: usize| if m == 0 { class } else { steps[m - 1].0 };
            // The dikes are the lower part of a fresh ridge's layer 2; erosion takes the lavas
            // first, and later volcanism (cones, flood basalts) adds lavas on top.
            let fresh = f64::from(crate::plates::pools::oceanic_column()[pool::MAFIC_VOLCANIC]);
            let dikes = th.min((1.0 - OCEAN_LAVA_SHARE) * fresh);
            let m = past(hot);
            emit(out, class_of(m), th - dikes, m, &mut hot);
            let m = past(hot).max(usize::from(!steps.is_empty()));
            emit(out, class_of(m), dikes, m, &mut hot);
            depth += th;
            continue;
        }
        for &(next, temp) in t.paths.steps(class) {
            let z = if temp < super::realism::METAMORPHIC_FROM_C {
                // Diagenesis: the present geotherm alone.
                isotherm(temp)
            } else if intrusive && temp < key::BAND_FROM_C[key::BANDS - 1] {
                continue;
            } else if !intrusive && temp <= hot {
                f64::NEG_INFINITY
            } else {
                isotherm(temp)
            };
            if z >= bottom {
                break;
            }
            if z > top {
                emit(out, current, z - top, step, &mut hot);
                top = z;
            }
            current = next;
            step += 1;
        }
        emit(out, current, bottom - top, step, &mut hot);
        depth += th;
    }
    if depth < max_depth_m {
        push(
            out,
            t.peridotite,
            max_depth_m - depth,
            Source::Mantle,
            key::MANTLE,
        );
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
            surface_c: 10.0,
            gradient_c_m: 0.0,
            log_span_myr: 500.0,
            top_active: true,
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
        // A hot continent (40 °C/km): the metamorphic pool is gneiss already; the 2-km
        // sedimentary pool stays sandstone above the 300 °C isotherm (7.25 km): nothing to split.
        // A thick sedimentary pool is split at 7.25 km into quartzite.
        let mut hot = continental(&[]);
        hot.gradient_c_m = 0.04;
        hot.pools[pool::SEDIMENTARY] = 10_000.0;
        build(&hot, &t, &p, 0.0, 12_000.0, &mut out);
        let classes: Vec<u16> = out.iter().map(|u| u.class).collect();
        assert_eq!(&classes[..2], &[index("mudstone"), index("sandstone")]);
        assert_eq!(classes[2], index("quartzite"));
        let above: f64 = out[..2].iter().map(|u| f64::from(u.thickness_m)).sum();
        assert!((above - 7250.0).abs() < 1e-2, "{above}");
    }

    #[test]
    fn oceanic_columns_are_the_penrose_sequence() {
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
        // Pelagic sediment, the lavas, the sheeted dikes, gabbro, the mantle.
        assert_eq!(
            classes,
            [
                index("chalk"),
                index("basalt"),
                index("greenstone"),
                index("gabbro"),
                index("peridotite")
            ]
        );
        let th: Vec<f32> = out.iter().map(|u| u.thickness_m).collect();
        assert!(
            (th[1] - 800.0).abs() < 1.0 && (th[2] - 1200.0).abs() < 1.0,
            "{th:?}"
        );
        assert!((th[3] - 5000.0).abs() < 1.0, "{th:?}");
        // Keys still grow downwards.
        for w in out.windows(2) {
            assert!(w[0].key[1] <= w[1].key[0], "{:?}", out);
        }
        cell.depth_below_sea_m = 5500.0;
        build(&cell, &t, &p, 0.0, 10_000.0, &mut out);
        assert_eq!(out[0].class, index("mudstone"), "below the CCD");

        // Erosion takes the lavas first: 1500 m of layer 2 is 300 m of lavas over the dikes.
        cell.pools[pool::MAFIC_VOLCANIC] = 1500.0;
        build(&cell, &t, &p, 0.0, 10_000.0, &mut out);
        let lava = out.iter().find(|u| u.class == index("basalt")).unwrap();
        let dikes = out.iter().find(|u| u.class == index("greenstone")).unwrap();
        assert!((lava.thickness_m - 300.0).abs() < 1.0 && (dikes.thickness_m - 1200.0).abs() < 1.0);
    }

    #[test]
    fn ocean_crust_felsic_rock_is_arc_rock_never_granite() {
        let t = Tables::get();
        let p = StructureParams::default();
        let mut cell = continental(&[]);
        cell.pools = crate::plates::pools::oceanic_column();
        cell.pools[pool::FELSIC_VOLCANIC] = 1500.0;
        cell.pools[pool::FELSIC_PLUTONIC] = 4000.0;
        cell.pools[pool::METAMORPHIC] = 300.0;
        cell.crust_type = OCEANIC;
        cell.regime = regime::OCEAN_BASIN;
        cell.depth_below_sea_m = 2000.0;
        let mut out = Vec::new();
        build(&cell, &t, &p, 0.0, 20_000.0, &mut out);
        let classes: Vec<u16> = out.iter().map(|u| u.class).collect();
        for name in ["granite", "diorite", "rhyolite", "gneiss"] {
            assert!(!classes.contains(&index(name)), "{name} in {classes:?}");
        }
        for name in ["andesite", "granodiorite", "schist", "gabbro"] {
            assert!(
                classes.contains(&index(name)),
                "{name} missing from {classes:?}"
            );
        }
        // Under an active island arc the plutonic rock is the arc's tonalitic middle crust.
        cell.regime = crate::plates::state::regime::ARC;
        build(&cell, &t, &p, 0.0, 20_000.0, &mut out);
        let classes: Vec<u16> = out.iter().map(|u| u.class).collect();
        assert!(classes.contains(&index("diorite")), "{classes:?}");
        assert!(!classes.contains(&index("granite")), "{classes:?}");
    }
}

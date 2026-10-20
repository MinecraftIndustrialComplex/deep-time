// SPDX-License-Identifier: LGPL-3.0-or-later
//! What the column inspectors say about a column (`deeptime inspect-column`, the workbench's
//! Inspect panel and its sections): each horizon's unit key decoded into the part of the column it
//! came from, its age and its metamorphic grade, the parent rock of a metamorphic unit, the age of
//! each structural event, and the scale a deposit body is drawn at. It reads the key layout of
//! [`crate::structure::columns::key`], the metamorphic paths and the world's own `preset.json`;
//! nothing here changes a world, and nothing here is a new physical value.
//!
//! **Unit keys** ([`decode`]). A horizon's unit key is the key top of the stored horizon it came
//! from (the least where the bake blended several), so it names the unit's youngest part:
//! - the regolith's keys: the loose cover at the surface;
//! - a band's strata log: a layer of the deep-time surface's log, keyed by age linearly across the
//!   band's log range from the present (its first key) to the run's start (its last), so the age
//!   is `(k - first) / (last - first)` of the log's span, `(now - t_start).max(1)` Myr, as
//!   `columns::key::of_age` writes it; the unit's top is that old, its base older;
//! - a band's pools: the plate model's rock pools (`plates::l1::TOP_ORDER`, a block of keys per
//!   pool and a slot per metamorphic step along its class's path);
//! - the mantle's keys.
//!
//! The band is the hottest formation temperature at or above the unit in its column
//! (`key::BAND_FROM_C`): band 0 is unmetamorphosed, band b formed at or above `BAND_FROM_C[b]`
//! (and below the next band's). Plutonic pool units are crystalline basement and always take the
//! last band, whatever their temperature. A horizon that an intrusion event made carries the
//! event's key (`StructEvent::key`: band 0's log key of the event's age), so [`annotate`] names it
//! from the event, not as a bed of that age.
//!
//! **Parent rock.** A basement pool unit's parent is its pool, and for a unit some steps along
//! its path also the pool's classes that reach its class at that step ([`pool_parents`], worked
//! out from the columns' own pool rule). The columns keep no protolith for a metamorphic unit of
//! the strata log (a log layer stores only what its deepest burial made of it,
//! `surface::strata_log::Layer::class`), so the parent of one is **inferred**: the log's classes
//! whose metamorphic path reaches its class ([`protoliths`]). Neither is a recorded fact.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::OnceLock;

use serde_json::{Value, json};

use crate::params::PlanetPreset;
use crate::plates::l1::TOP_ORDER;
use crate::plates::state::{CONTINENTAL, OCEANIC, pool, regime};
use crate::preset::{Preset, PresetBody};
use crate::structure::columns::{self, Cell, Tables, key};
use crate::structure::metamorphism::Paths;
use crate::structure::realism::METAMORPHIC_FROM_C;
use crate::surface::{basement_class, lithology};
use crate::synth::EVERY_CLASS;

/// The part of the key space a unit key lies in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Part {
    Regolith,
    /// A layer of the strata log: its age at the unit's top, Myr before the end of the history
    /// (None when the log's span is unknown), and how many Myr one key spans there.
    Log {
        age_ma: Option<f64>,
        key_myr: Option<f64>,
    },
    /// A basement pool: its position `q` in `TOP_ORDER`, its pool code (`plates::state::pool`) and
    /// its metamorphic step (0 the pool's own class, 3 the fourth step or later).
    Pool {
        q: usize,
        pool: usize,
        step: usize,
    },
    Mantle,
}

/// A decoded unit key.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitKey {
    pub key: u16,
    /// Its grade band (None for the regolith and the mantle).
    pub band: Option<usize>,
    pub part: Part,
}

/// Decodes unit key `k` of a world whose strata log spans `span_myr` (None: unknown).
pub fn decode(k: u16, span_myr: Option<f64>) -> UnitKey {
    let d = key::domain(k);
    let (band, part) = match d {
        0 => (None, Part::Regolith),
        15 => (None, Part::Mantle),
        _ => {
            let band = usize::from((d - 1) / 2);
            let [lo, hi] = key::log_range(band);
            if (d - 1).is_multiple_of(2) {
                let f = f64::from(k.saturating_sub(lo)) / f64::from(hi - lo);
                let span = span_myr.filter(|s| s.is_finite()).map(|s| s.max(1.0));
                (
                    Some(band),
                    Part::Log {
                        age_ma: span.map(|s| f.clamp(0.0, 1.0) * s),
                        key_myr: span.map(|s| s / f64::from(hi - lo)),
                    },
                )
            } else {
                let off = k - (hi + 1);
                let q = usize::from(off / key::POOL_BLOCK).min(TOP_ORDER.len() - 1);
                let step = usize::from((off % key::POOL_BLOCK) / key::SLOT);
                (
                    Some(band),
                    Part::Pool {
                        q,
                        pool: TOP_ORDER[q],
                        step,
                    },
                )
            }
        }
    };
    UnitKey { key: k, band, part }
}

/// Whether `class` is plutonic (the intrusive and special-host families, the columns'
/// `Tables::intrusive`): such a pool unit is keyed in the last band whatever its temperature.
fn plutonic(class: &str) -> bool {
    class_index(class).is_some_and(|k| {
        let f = &lithology::table()[usize::from(k)].family;
        f == "intrusive_igneous" || f == "special_host"
    })
}

/// The families a buried strata-log layer is (plutons are bodies, not log layers, and a loose
/// class is named by the rock it lithifies to): the possible parents of a metamorphic log unit.
const LOG_FAMILIES: [&str; 3] = ["clastic_sedimentary", "chemical_sedimentary", "volcanic"];

/// A pool's classes, each with the classes its steps take ([`pool_rocks`]).
type PoolRocks = Vec<(u16, Vec<u16>)>;

/// Per pool code, the classes its units can start as, each with the classes its steps take in
/// the columns' order ([`columns::build`]: a plutonic class passes over the metamorphic steps
/// below the last band). Worked out from [`columns::pool_class`] and the first unit's
/// [`basement_class`] over every crust type, regime, orogeny, orogeny age and sea depth, not
/// copied from them.
fn pool_rocks(p: usize) -> &'static [(u16, Vec<u16>)] {
    static ROCKS: OnceLock<Vec<PoolRocks>> = OnceLock::new();
    let all = ROCKS.get_or_init(|| {
        let t = Tables::get();
        let mut out = vec![BTreeSet::new(); pool::COUNT];
        for crust in [OCEANIC, CONTINENTAL] {
            for reg in 0..=regime::PASSIVE_MARGIN {
                for orogeny in 0..=2u8 {
                    for age in [f32::NAN, 10.0, 50.0, 500.0] {
                        for depth in [-1000.0, 1000.0, 6000.0] {
                            let cell = Cell {
                                regolith_m: 0.0,
                                regolith_kind: 0,
                                log: &[],
                                pools: [0.0; pool::COUNT],
                                crust_type: crust,
                                regime: reg,
                                orogeny,
                                orogeny_age: age,
                                depth_below_sea_m: depth,
                                surface_c: 10.0,
                                gradient_c_m: 0.025,
                                log_span_myr: 1000.0,
                                top_active: true,
                            };
                            for (k, set) in out.iter_mut().enumerate() {
                                set.insert(columns::pool_class(&t, &cell, k));
                                if crust != OCEANIC {
                                    set.insert(basement_class(
                                        &t.cls, crust, reg, orogeny, age, k as u8,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        let last_band_c = key::BAND_FROM_C[key::BANDS - 1];
        out.into_iter()
            .map(|set| {
                set.into_iter()
                    .map(|c| {
                        let intrusive = t.intrusive[usize::from(c)];
                        let steps = t
                            .paths
                            .steps(c)
                            .iter()
                            .filter(|&&(_, temp)| {
                                !(intrusive && temp >= METAMORPHIC_FROM_C && temp < last_band_c)
                            })
                            .map(|s| s.0)
                            .collect();
                        (c, steps)
                    })
                    .collect()
            })
            .collect()
    });
    all.get(p).map_or(&[], Vec::as_slice)
}

/// The rocks a unit of class `class` in pool `p` at step `step` (its key's slot) can have started
/// as: the pool's classes ([`pool_rocks`]) whose steps reach `class` at that step (the last slot
/// holds every later step too). At step 0 the unit is the pool's own rock: `class` itself.
pub fn pool_parents(p: usize, class: &str, step: usize) -> Vec<&'static str> {
    let Some(k) = class_index(class) else {
        return Vec::new();
    };
    let t = lithology::table();
    if step == 0 {
        return vec![t[usize::from(k)].name];
    }
    let last = usize::from(key::STEPS) - 1;
    pool_rocks(p)
        .iter()
        .filter(|(_, steps)| {
            if step < last {
                steps.get(step - 1) == Some(&k)
            } else {
                steps.iter().skip(step - 1).any(|&x| x == k)
            }
        })
        .map(|(c, _)| t[usize::from(*c)].name)
        .collect()
}

fn class_index(name: &str) -> Option<u16> {
    let name = name.strip_prefix("deeptime:").unwrap_or(name);
    EVERY_CLASS
        .iter()
        .position(|&c| c == name)
        .map(|k| k as u16)
}

/// The rock classes a strata-log layer of metamorphic class `class` can have been laid down as
/// (in the vocabulary's order): the log's classes ([`LOG_FAMILIES`]) whose metamorphic path
/// reaches `class`. Inferred: the world does not record which one it was. Empty for a class that
/// is not metamorphic (it is its own rock).
pub fn protoliths(class: &str) -> Vec<&'static str> {
    let Some(k) = class_index(class) else {
        return Vec::new();
    };
    let t = lithology::table();
    if t[usize::from(k)].family != "metamorphic" {
        return Vec::new();
    }
    let paths = Paths::get();
    (0..t.len() as u16)
        .filter(|&c| {
            LOG_FAMILIES.contains(&t[usize::from(c)].family.as_str())
                && paths.steps(c).iter().any(|s| s.0 == k)
        })
        .map(|c| t[usize::from(c)].name)
        .collect()
}

/// The pool code's name (`plates::state::pool::NAMES`), spaces for underscores.
pub fn pool_name(code: usize) -> String {
    pool::NAMES
        .get(code)
        .map_or_else(|| format!("pool {code}"), |n| n.replace('_', " "))
}

/// The peak-temperature words of a band, and the band's [lower, upper) bound in °C.
fn grade(band: usize) -> (String, f64, Option<f64>) {
    let lo = key::BAND_FROM_C[band.min(key::BANDS - 1)];
    let hi = key::BAND_FROM_C.get(band + 1).copied();
    let words = match (band, hi) {
        (0, Some(h)) => format!("unmetamorphosed (never at {h:.0} °C)"),
        (_, Some(h)) => format!("formed at or above {lo:.0} °C (below {h:.0} °C)"),
        (_, None) => format!("formed at or above {lo:.0} °C"),
    };
    (words, lo, hi)
}

fn round(x: f64, places: i32) -> f64 {
    let m = 10f64.powi(places);
    (x * m).round() / m
}

/// What a unit of class `class` (the world's class name) with key `u` is, as JSON: `part`
/// (regolith, log, pool, mantle), `band`, `peak_c` ([lower, upper or null] °C), `grade` (words),
/// `age_ma` and `key_myr` for a log unit, `pool` and `pool_step` for a pool unit, `parent`
/// (`{"kind": "pool" | "inferred" | "itself" | "none", "rocks": [...], "text"}`) and `text`, one
/// line of all of it.
pub fn describe(u: &UnitKey, class: &str) -> Value {
    let cls = class.strip_prefix("deeptime:").unwrap_or(class);
    let name = cls.replace('_', " ");
    let mut v = json!({ "key": u.key, "band": u.band });
    let (what, parent) = match u.part {
        Part::Regolith => {
            v["part"] = json!("regolith");
            (
                "regolith: the loose cover at the surface (not keyed by grade)".to_owned(),
                json!({ "kind": "none", "rocks": [], "text": "the rock beneath it, weathered" }),
            )
        }
        Part::Mantle => {
            v["part"] = json!("mantle");
            (
                "mantle below the crust (not keyed by grade)".to_owned(),
                json!({ "kind": "none", "rocks": [], "text": "none: mantle rock" }),
            )
        }
        Part::Log { age_ma, key_myr } => {
            v["part"] = json!("log");
            v["age_ma"] = json!(age_ma.map(|a| round(a, 2)));
            v["key_myr"] = json!(key_myr.map(|a| round(a, 4)));
            let when = match age_ma {
                Some(a) => format!("its top laid down {} Myr ago", round(a, 1)),
                None => "age unknown (the world has no preset to give the log's span)".to_owned(),
            };
            let parents = protoliths(cls);
            let parent = if !parents.is_empty() {
                json!({ "kind": "inferred", "rocks": parents,
                        "text": format!("inferred from the metamorphic paths: {}", parents.join(", ").replace('_', " ")) })
            } else {
                json!({ "kind": "itself", "rocks": [cls],
                        "text": format!("{name} as laid down (no metamorphic class)") })
            };
            (format!("strata log layer, {when}"), parent)
        }
        Part::Pool { q, pool, step } => {
            v["part"] = json!("pool");
            v["pool"] = json!(pool::NAMES.get(pool).copied());
            v["pool_order"] = json!(q);
            v["pool_step"] = json!(step);
            let pn = pool_name(pool);
            let stepped = if step == 0 {
                "the pool's own rock".to_owned()
            } else {
                format!(
                    "{step}{} step along its class's metamorphic path",
                    if step >= usize::from(key::STEPS) - 1 {
                        " (or later)"
                    } else {
                        ""
                    }
                )
            };
            let rocks = pool_parents(pool, cls, step);
            let text = if step == 0 || rocks.is_empty() {
                format!("the {pn} pool of the plate model")
            } else {
                format!(
                    "the {pn} pool of the plate model, {} before metamorphism (inferred from the pool's rules and the paths)",
                    rocks.join(" or ").replace('_', " ")
                )
            };
            (
                format!("basement: the {pn} pool, {stepped}"),
                json!({ "kind": "pool", "pool": pool::NAMES.get(pool).copied(), "rocks": rocks, "text": text }),
            )
        }
    };
    let peak = match (u.part, u.band) {
        (Part::Pool { .. }, Some(b)) if b == key::BANDS - 1 && plutonic(cls) => {
            v["peak_c"] = json!([key::BAND_FROM_C[b], null]);
            Some(
                "plutonic basement: keyed with the highest grade whatever its temperature"
                    .to_owned(),
            )
        }
        (_, Some(b)) => {
            let (words, lo, hi) = grade(b);
            v["peak_c"] = json!([lo, hi]);
            Some(words)
        }
        _ => {
            v["peak_c"] = Value::Null;
            None
        }
    };
    v["grade"] = json!(peak);
    v["parent"] = parent;
    v["text"] = json!(match &peak {
        Some(p) => format!("{what}; {p}"),
        None => what,
    });
    v
}

/// A number as the labels print it: whole numbers without decimals, else at most two.
fn num(x: f64) -> String {
    if (x - x.round()).abs() < 1e-9 {
        format!("{}", x.round() as i64)
    } else {
        let s = format!("{x:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_owned()
    }
}

/// The ledger row of a deposit type's playability enlargement (its `balance.size_mult_h` and
/// `size_mult_v`): coal seams DEP-127, porphyries DEP-195, the others ORE-055.
pub fn scale_ledger(type_id: &str) -> &'static str {
    match type_id {
        "deeptime:coal_seam" => "DEP-127",
        "deeptime:porphyry_cu_mo_au" => "DEP-195",
        _ => "ORE-055",
    }
}

/// How a body of deposit type `type_id` and `shape` is drawn against the deposit model, given the
/// type's horizontal and vertical multipliers `(h, v)` as the world used them: None when both
/// are 1. A game choice (the ledger row says why): "coal seam drawn 35 x its modelled thickness
/// (game choice, ledger DEP-127)".
pub fn scale_note(type_id: &str, shape: &str, h: f64, v: f64) -> Option<Value> {
    let one = |x: f64| (x - 1.0).abs() < 1e-9;
    if one(h) && one(v) {
        return None;
    }
    let short = type_id.strip_prefix("deeptime:").unwrap_or(type_id);
    let what = if short == "coal_seam" {
        "coal seam".to_owned()
    } else {
        format!("{} {shape}", short.replace('_', " "))
    };
    let tall = if matches!(shape, "seam" | "sheet" | "blanket" | "lens" | "channel") {
        "thickness"
    } else {
        "height"
    };
    let mut parts = Vec::new();
    if !one(v) {
        parts.push(format!("{} x its modelled {tall}", num(v)));
    }
    if !one(h) {
        parts.push(format!("{} x its modelled width", num(h)));
    }
    let ledger = scale_ledger(type_id);
    Some(json!({
        "size_mult_v": v, "size_mult_h": h, "ledger": ledger,
        "text": format!("{what} drawn {} (game choice, ledger {ledger})", parts.join(" and ")),
    }))
}

/// What the inspectors need of a world beyond its folder's layers: the history's clock and the
/// deposit types' drawn scales, from its `preset.json`.
#[derive(Clone, Debug, Default)]
pub struct WorldFacts {
    /// The end of the history and its start, Myr (world time: negative is the past).
    pub now_myr: Option<f64>,
    pub start_myr: Option<f64>,
    /// Deposit type id: its (size_mult_h, size_mult_v) as the run drew it.
    pub scales: BTreeMap<String, (f64, f64)>,
}

impl WorldFacts {
    /// The facts of a planet preset.
    pub fn of_preset(p: &PlanetPreset) -> WorldFacts {
        let scales = crate::deposits::types_of(p)
            .map(|types| {
                types
                    .iter()
                    .map(|t| (t.id.clone(), (t.balance.size_mult_h, t.balance.size_mult_v)))
                    .collect()
            })
            .unwrap_or_default();
        WorldFacts {
            now_myr: Some(p.t_end_years() / 1.0e6),
            start_myr: Some(p.t_start_years() / 1.0e6),
            scales,
        }
    }

    /// The facts of world-data folder `dir`, from its `preset.json`; empty (ages and scales
    /// unknown) when it has none or it is not a planet preset.
    pub fn of_world(dir: &Path) -> WorldFacts {
        let Ok(text) = std::fs::read_to_string(dir.join("preset.json")) else {
            return WorldFacts::default();
        };
        match Preset::from_json(&text).map(|p| p.body) {
            Ok(PresetBody::Planet(p)) => WorldFacts::of_preset(&p),
            _ => WorldFacts::default(),
        }
    }

    /// The strata log's span, Myr (`structure`'s `(now - t_start).max(1)`).
    pub fn span_myr(&self) -> Option<f64> {
        Some((self.now_myr? - self.start_myr?).max(1.0))
    }

    /// Myr before the end of the history of world time `t_years`.
    pub fn age_ma(&self, t_years: f64) -> Option<f64> {
        Some(self.now_myr? - t_years / 1.0e6)
    }

    /// The drawn-scale note of a body of deposit type `type_id` and `shape` ([`scale_note`]).
    pub fn scale_of(&self, type_id: &str, shape: &str) -> Option<Value> {
        let &(h, v) = self.scales.get(type_id)?;
        scale_note(type_id, shape, h, v)
    }
}

/// Adds what [`ColumnWorld::inspect`](deeptime_format::ColumnWorld::inspect) cannot know to its
/// answer `v`: each `strata` horizon's `unit_info` ([`describe`]; a horizon an intrusion event
/// made, its key and class those of a `structure_events` intrusion, is named from the event),
/// each structural event's `age_ma`, each body's `drawn_scale` ([`scale_note`]) and `units_note`,
/// how to read them.
pub fn annotate(v: &mut Value, facts: &WorldFacts) {
    let span = facts.span_myr();
    // The intrusions among the column's events: (key, class) → (id, age).
    let mut intrusions: Vec<(u64, String, Value, Value)> = Vec::new();
    if let Some(events) = v.get_mut("structure_events").and_then(Value::as_array_mut) {
        for e in events.iter_mut() {
            let age = e["t_years"].as_f64().and_then(|t| facts.age_ma(t));
            e["age_ma"] = json!(age.map(|a| round(a, 2)));
            if e["kind"] == "intrusion" {
                intrusions.push((
                    e["key"].as_u64().unwrap_or(u64::MAX),
                    e["class"].as_str().unwrap_or("").to_owned(),
                    e["id"].clone(),
                    e["age_ma"].clone(),
                ));
            }
        }
    }
    if let Some(strata) = v.get_mut("strata").and_then(Value::as_array_mut) {
        for h in strata.iter_mut() {
            let (Some(k), Some(class)) = (h["unit"].as_u64(), h["class"].as_str()) else {
                continue;
            };
            let class = class.to_owned();
            let k = k.min(u64::from(u16::MAX)) as u16;
            let info = match intrusions
                .iter()
                .find(|(ik, ic, _, _)| *ik == u64::from(k) && *ic == class)
            {
                Some((_, _, id, age)) => {
                    let when = age.as_f64().map_or("age unknown".to_owned(), |a| {
                        format!("{} Myr ago", round(a, 1))
                    });
                    json!({
                        "key": k, "part": "intrusion", "event": id, "age_ma": age,
                        "band": null, "peak_c": null, "grade": null,
                        "parent": { "kind": "none", "rocks": [], "text": "none: igneous rock of the intrusion" },
                        "text": format!("intrusion (structural event {id}), emplaced {when}; it replaced the rock older than itself"),
                    })
                }
                None => describe(&decode(k, span), &class),
            };
            h["unit_info"] = info;
        }
    }
    if let Some(bodies) = v.get_mut("body_details").and_then(Value::as_array_mut) {
        for b in bodies.iter_mut() {
            let (Some(t), Some(shape)) = (b["deposit_type"].as_str(), b["shape"].as_str()) else {
                continue;
            };
            if let Some(n) = facts.scale_of(t, shape) {
                b["drawn_scale"] = n;
            }
        }
    }
    v["units_note"] = json!(
        "unit_info decodes each horizon's unit key (deeptime_core::column_info): the part of the \
         column it came from, a log layer's age at its top (Myr before the end of the history), \
         the grade band (the hottest formation temperature at or above it) and its parent rock: \
         the pool for a basement unit, inferred from the metamorphic paths for a metamorphic log \
         unit (the world does not record which rock it was)"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPAN: Option<f64> = Some(1000.0);

    #[test]
    fn the_parts_of_the_key_space_decode() {
        assert_eq!(decode(0, SPAN).part, Part::Regolith);
        assert_eq!(decode(key::REGOLITH[1], SPAN).band, None);
        assert_eq!(decode(key::MANTLE[0], SPAN).part, Part::Mantle);
        assert_eq!(decode(u16::MAX, SPAN).band, None);
        // Band 0's log: the first key is the present, the last the run's start.
        let top = decode(key::LOG_TOP, SPAN);
        assert_eq!(top.band, Some(0));
        let Part::Log { age_ma, key_myr } = top.part else {
            panic!("{top:?}")
        };
        assert_eq!(age_ma, Some(0.0));
        assert!(
            (key_myr.unwrap() - 1000.0 / f64::from(key::LOG_BASE - key::LOG_TOP)).abs() < 1e-12
        );
        let Part::Log { age_ma, .. } = decode(key::LOG_BASE, SPAN).part else {
            panic!()
        };
        assert_eq!(age_ma, Some(1000.0));
        // An unknown span leaves the age unknown, not zero.
        assert_eq!(
            decode(key::LOG_TOP + 10, None).part,
            Part::Log {
                age_ma: None,
                key_myr: None
            }
        );
    }

    /// `decode` inverts `key::of_age` in every band, to within one key, and `key::pool` exactly.
    #[test]
    fn decoding_inverts_the_key_writers() {
        for band in 0..key::BANDS {
            for &(delta, span) in &[
                (0.0, 1000.0),
                (123.4, 1000.0),
                (499.0, 500.0),
                (3.0, 4000.0),
            ] {
                let k = key::of_age(band, delta, span);
                let u = decode(k, Some(span));
                assert_eq!(u.band, Some(band), "band {band}, key {k}");
                let Part::Log { age_ma, key_myr } = u.part else {
                    panic!("band {band}: {u:?}")
                };
                let (a, step) = (age_ma.unwrap(), key_myr.unwrap());
                assert!(
                    (a - delta).abs() <= step * 0.5 + 1e-9,
                    "band {band}: {delta} Myr read back as {a} (a key is {step} Myr)"
                );
            }
            for (q, &top) in TOP_ORDER.iter().enumerate() {
                for m in 0..usize::from(key::STEPS) {
                    let [lo, hi] = key::pool(band, q, m);
                    for k in [lo, hi] {
                        let u = decode(k, SPAN);
                        assert_eq!(u.band, Some(band));
                        assert_eq!(
                            u.part,
                            Part::Pool {
                                q,
                                pool: top,
                                step: m
                            },
                            "band {band}, pool {q}, step {m}, key {k}"
                        );
                    }
                }
            }
        }
        // A step past the last slot shares it.
        let [lo, _] = key::pool(2, 1, 9);
        let Part::Pool { step, .. } = decode(lo, SPAN).part else {
            panic!()
        };
        assert_eq!(step, usize::from(key::STEPS) - 1);
    }

    #[test]
    fn grades_and_parents_are_named() {
        // A log unit of phyllite in band 3 (formed at 350 °C or more, below 500).
        let k = key::of_age(3, 200.0, 1000.0);
        let d = describe(&decode(k, SPAN), "phyllite");
        assert_eq!(d["part"], "log");
        assert_eq!(d["peak_c"], json!([350.0, 500.0]));
        assert_eq!(d["parent"]["kind"], "inferred");
        let rocks: Vec<&str> = d["parent"]["rocks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.as_str().unwrap())
            .collect();
        assert!(
            rocks.contains(&"shale") && rocks.contains(&"mudstone"),
            "{rocks:?}"
        );
        assert!(
            d["text"]
                .as_str()
                .unwrap()
                .contains("formed at or above 350 °C")
        );
        // Band 0 is unmetamorphosed and its rock is its own.
        let d = describe(&decode(key::of_age(0, 50.0, 1000.0), SPAN), "sandstone");
        assert_eq!(d["parent"]["kind"], "itself");
        assert!(d["grade"].as_str().unwrap().starts_with("unmetamorphosed"));
        // A pool unit: the pool, and the step's parent; a plutonic one sits in the last band.
        let [lo, _] = key::pool(2, 1, 1);
        let d = describe(&decode(lo, SPAN), "quartzite");
        assert_eq!(d["part"], "pool");
        assert_eq!(d["pool"], "sedimentary");
        assert_eq!(d["parent"]["kind"], "pool");
        assert_eq!(d["parent"]["rocks"], json!(["sandstone"]));
        let q = TOP_ORDER
            .iter()
            .position(|&p| p == pool::FELSIC_PLUTONIC)
            .unwrap();
        let [lo, _] = key::pool(key::BANDS - 1, q, 0);
        let d = describe(&decode(lo, SPAN), "granite");
        assert_eq!(d["pool"], "felsic_plutonic");
        assert!(
            d["grade"]
                .as_str()
                .unwrap()
                .starts_with("plutonic basement")
        );
        // The regolith and the mantle have no grade.
        assert_eq!(describe(&decode(10, SPAN), "soil")["grade"], Value::Null);
        assert_eq!(
            describe(&decode(key::MANTLE[0], SPAN), "peridotite")["part"],
            "mantle"
        );
    }

    #[test]
    fn protoliths_follow_the_paths() {
        let gneiss = protoliths("gneiss");
        for p in ["shale", "mudstone", "rhyolite"] {
            assert!(gneiss.contains(&p), "{p} in {gneiss:?}");
        }
        // A metamorphic rock, a pluton or a loose class is not what a log layer was laid down as.
        for p in ["schist", "granite", "mud"] {
            assert!(!gneiss.contains(&p), "{p} in {gneiss:?}");
        }
        assert!(protoliths("deeptime:marble").contains(&"limestone"));
        assert!(protoliths("sandstone").is_empty());
        assert!(protoliths("no_such_rock").is_empty());
    }

    /// A pool's rocks are those of the columns' own pool rule, and a step's parents are among
    /// them, with a plutonic class passing over the steps below the last band.
    #[test]
    fn pool_parents_follow_the_pool_rule() {
        let names = |p: usize| -> Vec<&str> {
            pool_rocks(p)
                .iter()
                .map(|(c, _)| lithology::table()[usize::from(*c)].name)
                .collect()
        };
        let sedimentary = names(pool::SEDIMENTARY);
        assert!(
            sedimentary.contains(&"sandstone") && sedimentary.contains(&"shale"),
            "{sedimentary:?}"
        );
        assert!(names(pool::MAFIC_PLUTONIC).contains(&"gabbro"));
        assert_eq!(
            pool_parents(pool::SEDIMENTARY, "quartzite", 1),
            vec!["sandstone"]
        );
        let felsic = pool_parents(pool::FELSIC_PLUTONIC, "gneiss", 1);
        assert!(felsic.contains(&"granite"), "{felsic:?}");
        // Gabbro's greenstone and amphibolite steps are below the last band: a plutonic pool's
        // gabbro never takes them. Only the first basement unit of a continental hotspot or arc,
        // which `basement_class` makes basalt or andesite whatever its pool, can be greenstone
        // there.
        assert_eq!(
            pool_parents(pool::MAFIC_PLUTONIC, "greenstone", 1),
            vec!["andesite", "basalt"]
        );
        // The last slot holds every later step: shale's gneiss is its fourth.
        let last = usize::from(key::STEPS) - 1;
        assert!(pool_parents(pool::SEDIMENTARY, "gneiss", last).contains(&"shale"));
        assert_eq!(
            pool_parents(pool::SEDIMENTARY, "sandstone", 0),
            vec!["sandstone"]
        );
    }

    #[test]
    fn a_coal_seam_is_labelled_with_its_multiplier() {
        let n = scale_note("deeptime:coal_seam", "seam", 1.0, 35.0).unwrap();
        assert_eq!(
            n["text"],
            "coal seam drawn 35 x its modelled thickness (game choice, ledger DEP-127)"
        );
        let n = scale_note("deeptime:porphyry_cu_mo_au", "stockwork_shell", 12.0, 1.75).unwrap();
        assert_eq!(
            n["text"],
            "porphyry cu mo au stockwork_shell drawn 1.75 x its modelled height and 12 x its modelled width (game choice, ledger DEP-195)"
        );
        assert!(scale_note("deeptime:vms_cu_zn", "lens", 1.0, 1.0).is_none());
        // The built-in coal type's own multiplier, as a run on the default preset draws it.
        let types = crate::deposits::types::builtin().unwrap();
        let coal = types.iter().find(|t| t.id == "deeptime:coal_seam").unwrap();
        let n = scale_note(
            &coal.id,
            "seam",
            coal.balance.size_mult_h,
            coal.balance.size_mult_v,
        )
        .unwrap();
        assert!(n["text"].as_str().unwrap().starts_with("coal seam drawn"));
    }

    #[test]
    fn annotate_names_an_intrusion_from_its_event_and_ages_the_events() {
        let facts = WorldFacts {
            now_myr: Some(0.0),
            start_myr: Some(-1000.0),
            scales: BTreeMap::from([("deeptime:coal_seam".to_owned(), (1.0, 35.0))]),
        };
        let ik = key::of_age(0, 40.0, 1000.0);
        let mut v = json!({
            "strata": [
                { "top": 60, "bottom": 50, "class": "sandstone", "unit": key::of_age(0, 10.0, 1000.0) },
                { "top": 49, "bottom": 20, "class": "granite", "unit": ik },
                { "top": 19, "bottom": -64, "class": "peridotite", "unit": key::MANTLE[0] },
            ],
            "structure_events": [
                { "id": 7, "kind": "intrusion", "t_years": -40.0e6, "key": ik, "class": "granite" },
                { "id": 3, "kind": "fold", "t_years": -300.0e6, "key": 9000 },
            ],
            "body_details": [
                { "id": 1, "shape": "seam", "deposit_type": "deeptime:coal_seam" },
                { "id": 2, "shape": "ellipsoid", "role": "pluton", "deposit_type": null },
            ],
        });
        annotate(&mut v, &facts);
        let s = &v["strata"];
        assert_eq!(s[0]["unit_info"]["part"], "log");
        assert!((s[0]["unit_info"]["age_ma"].as_f64().unwrap() - 10.0).abs() < 0.05);
        assert_eq!(s[1]["unit_info"]["part"], "intrusion");
        assert_eq!(s[1]["unit_info"]["event"], 7);
        assert_eq!(s[2]["unit_info"]["part"], "mantle");
        assert_eq!(v["structure_events"][0]["age_ma"], 40.0);
        assert_eq!(v["structure_events"][1]["age_ma"], 300.0);
        assert_eq!(
            v["body_details"][0]["drawn_scale"]["text"],
            "coal seam drawn 35 x its modelled thickness (game choice, ledger DEP-127)"
        );
        assert!(v["body_details"][1].get("drawn_scale").is_none());
    }
}

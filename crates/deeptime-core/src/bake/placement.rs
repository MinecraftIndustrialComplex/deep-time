// SPDX-License-Identifier: LGPL-3.0-or-later
//! Where each Köppen class goes on vanilla's temperature and humidity bands (plan §7.2, P1-09).
//!
//! The table is data: `data/climate_mapping/placement.json` (FORMAT.md, "The climate mapping"),
//! compiled into the core as the bake's default and shipped in the jar as a datapack resource a
//! pack may override. The game reads the pack's copy at a dedicated server's first start and passes
//! it to the bake in the content bundle (`climate_placement`, [`crate::preset::Preset::with_content`]);
//! the bake records the table it used in `notes.bake.climate.placement`.
//!
//! A [`Placement`] is the parsed and checked table. The default reproduces the constants P1-09
//! first compiled in, operation for operation (the bake's rasters are bit-identical).
//!
//! **Pack placement tables** (decisions log 2026-10-04, "Deep Time-side fixes"): a biome mod whose
//! biomes sit in other parameter slots than vanilla's gets a table of its own, written as a JSON
//! merge patch on the base table (`data/climate_mapping/packs/<id>.json`, shipped in the jar beside
//! `placement.json`). The game applies the ones whose `requires_biomes` are all registered, in
//! order of `priority` then `id`, and passes them in the content bundle under [`PATCHES_KEY`];
//! [`Placement::with_packs`] merges them and the bake records their ids. Terralith's is the first.
//! A table may carry `class_rules`: per-class overrides that hold only where their conditions on
//! the cell (temperatures, aridity, latitude, height) do, such as Terralith's cloud forest on
//! tropical mountains.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde_json::{Map, Value};

use super::biomes::{humidity_of_aridity, thermal_sea, unit};
use crate::climate::koppen;
use crate::error::CoreError;

/// The default table, one file with the game.
pub const PLACEMENT_JSON: &str = include_str!("../../data/climate_mapping/placement.json");

/// The table's schema version.
pub const SCHEMA: u64 = 1;

/// The `content` key that carries a datapack's table to the bake.
pub const CONTENT_KEY: &str = "climate_placement";

/// The `content` key that carries the pack placement tables the game chose for the installed biome
/// mods (`data/deeptime/deeptime/climate_mapping/packs/*.json`, each a merge patch on the base
/// table; [`Placement::with_packs`]).
pub const PATCHES_KEY: &str = "climate_placement_patches";

/// A band coordinate is in [0, 5].
const TOP: f64 = 5.0;

/// What a ramp, a switch or a class rule's condition reads of a cell: the three temperatures (°C;
/// a ramp reads only these), and, for switches and class rules, the aridity index P/PET, the
/// absolute latitude (degrees) and the height of the ground above the sea (m).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Input {
    Mean,
    Warm,
    Cold,
    Aridity,
    AbsLat,
    Height,
}

impl Input {
    fn parse(s: &str) -> Option<Input> {
        match s {
            "t_mean" => Some(Input::Mean),
            "t_warm" => Some(Input::Warm),
            "t_cold" => Some(Input::Cold),
            "aridity" => Some(Input::Aridity),
            "abs_lat_deg" => Some(Input::AbsLat),
            "height_m" => Some(Input::Height),
            _ => None,
        }
    }

    /// The names a switch or a condition accepts.
    const NAMES: &'static str = "t_mean, t_warm, t_cold, aridity, abs_lat_deg or height_m";

    /// The range a threshold on this input must lie in.
    fn range(self) -> (f64, f64) {
        match self {
            Input::Mean | Input::Warm | Input::Cold => (-100.0, 100.0),
            Input::Aridity => (0.0, 100.0),
            Input::AbsLat => (0.0, 90.0),
            Input::Height => (-12_000.0, 12_000.0),
        }
    }

    fn is_temperature(self) -> bool {
        matches!(self, Input::Mean | Input::Warm | Input::Cold)
    }
}

/// What the table reads of one land cell. An input the caller does not know is NaN: a switch then
/// takes its `else` branch and a class rule's condition on it fails.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Site {
    /// Mean, warmest-month and coldest-month temperature, °C.
    pub t_mean: f64,
    pub t_warm: f64,
    pub t_cold: f64,
    /// The aridity index P/PET.
    pub aridity: f64,
    /// |latitude|, degrees.
    pub abs_lat_deg: f64,
    /// The ground's height above the sea, m (0 at or below it).
    pub height_m: f64,
    /// Above the equilibrium line (glaciated).
    pub glacier: bool,
}

impl Site {
    /// A site of these temperatures, aridity and glacier flag, its latitude and height unknown.
    pub fn of(t_mean: f64, t_warm: f64, t_cold: f64, aridity: f64, glacier: bool) -> Site {
        Site {
            t_mean,
            t_warm,
            t_cold,
            aridity,
            abs_lat_deg: f64::NAN,
            height_m: f64::NAN,
            glacier,
        }
    }

    fn input(&self, of: Input) -> f64 {
        match of {
            Input::Mean => self.t_mean,
            Input::Warm => self.t_warm,
            Input::Cold => self.t_cold,
            Input::Aridity => self.aridity,
            Input::AbsLat => self.abs_lat_deg,
            Input::Height => self.height_m,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Term {
    of: Input,
    from: f64,
    to: f64,
    weight: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct Ramp {
    base: f64,
    span: f64,
    terms: Vec<Term>,
}

/// A class's temperature: a ramp, or a switch on one input between two of these (nested: a
/// class can have a warm, a cool and a cold branch).
#[derive(Clone, Debug, PartialEq)]
enum Thermal {
    Ramp(usize),
    When {
        of: Input,
        above: f64,
        then: Box<Thermal>,
        otherwise: Box<Thermal>,
    },
}

/// A class's humidity: a rule, or a switch on a temperature input between two of these.
#[derive(Clone, Debug, PartialEq)]
enum Humid {
    Rule(usize),
    When {
        of: Input,
        above: f64,
        then: Box<Humid>,
        otherwise: Box<Humid>,
    },
}

/// The deepest a `when` may nest.
const MAX_NEST: usize = 4;

#[derive(Clone, Debug, PartialEq)]
struct Rule {
    squeeze: Option<(f64, f64)>,
    min: Option<f64>,
    max: Option<f64>,
}

/// What a class does where the glacier flag is set, instead of the table's `glacier_cap`: its own
/// temperature and/or humidity (each falls back to the class's ordinary one, uncapped).
#[derive(Clone, Debug, PartialEq)]
struct Glaciated {
    thermal: Option<Thermal>,
    humidity: Option<Humid>,
    erosion: Option<ErosionLimit>,
}

/// Limits (vanilla's units of the parameter: erosion, or continentalness) a class's land is held
/// to: clamped into `[min, max]`. A missing bound is the end of the parameter's range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limit {
    pub min: f64,
    pub max: f64,
}

/// Erosion limits (vanilla's erosion units, −1 … 1).
pub type ErosionLimit = Limit;

/// The continentalness parameter's range (vanilla's units: the mushroom-fields floor to the top).
const CONTINENTS_RANGE: (f64, f64) = (-1.2, 1.0);

/// One condition of a class rule: the input above or below a threshold (strictly; a NaN input
/// fails).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Cond {
    of: Input,
    above: Option<f64>,
    below: Option<f64>,
}

impl Cond {
    fn holds(&self, s: &Site) -> bool {
        let x = s.input(self.of);
        self.above.is_none_or(|a| x > a) && self.below.is_none_or(|b| x < b)
    }
}

/// A class rule (`class_rules`): where every condition holds on the unglaciated land of one of its
/// classes, its temperature, humidity, erosion and continentalness (each optional) replace the
/// class's own. The first rule that holds decides.
#[derive(Clone, Debug, PartialEq)]
struct ClassRule {
    name: String,
    /// Bit k set: Köppen class k.
    classes: u32,
    when: Vec<Cond>,
    thermal: Option<Thermal>,
    humidity: Option<Humid>,
    erosion: Option<Limit>,
    continentalness: Option<Limit>,
}

/// Where the table puts one land cell: its band coordinates, the limits its land is held to and its
/// branch of the variant noise.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub thermal: f64,
    pub humidity: f64,
    pub erosion: Option<Limit>,
    pub continentalness: Option<Limit>,
    pub weirdness: Weirdness,
    /// The index of the class rule that decided, if one did.
    pub rule: Option<usize>,
}

/// Which branch of the variant noise a class's land takes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Weirdness {
    /// The noise signs it (vanilla's variants on one branch, the other on the other).
    Noise,
    /// Always the positive branch.
    Positive,
    /// The positive branch where the cell's erosion is below `erosion` (high relief: frozen
    /// peaks); elsewhere (flats) the negative branch, except as `flats` says.
    PositiveBelowErosion { erosion: f64, flats: Flats },
}

/// What a class's flats take of the variant noise (the land its `positive_below_erosion` does not
/// keep positive).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flats {
    /// The flats are on the positive branch where the variant noise (−1 … 1) is above this, on the
    /// negative one elsewhere: 1 (the default) is never, 0 is the noise's own sign (about half).
    pub positive_noise_above: f64,
    /// The largest |weirdness| a flat takes (vanilla's units, 0.07 … 1), or none. The positive
    /// branch's variants at a high |weirdness| include Terralith's cherry grove.
    pub max_magnitude: Option<f64>,
}

impl Default for Flats {
    fn default() -> Self {
        Flats {
            positive_noise_above: 1.0,
            max_magnitude: None,
        }
    }
}

/// The ice cover the bake writes (`climate/ice_cover`) and the game's ice pass lays
/// (`placement.json`, `ice_cover`; FORMAT.md, "The ice cover").
#[derive(Clone, Debug, PartialEq)]
pub struct IceCoverSpec {
    /// Metres of land ice (or of height above the equilibrium line, without ice fields) per block
    /// of cover after the first.
    pub step_m: f64,
    /// The thickest cover, blocks.
    pub max_blocks: u8,
    /// Snow blocks on top of a cover.
    pub snow_blocks: u8,
    /// Covers of at least this many blocks have a blue-ice base.
    pub blue_ice_from_blocks: u8,
    /// Blocks of blue ice at the base of a thick cover.
    pub blue_ice_blocks: u8,
    /// Block ids: the snow on top, the ice between, the ice at the base.
    pub snow: String,
    pub ice: String,
    pub deep_ice: String,
}

impl Default for IceCoverSpec {
    fn default() -> Self {
        IceCoverSpec {
            step_m: 160.0,
            max_blocks: 12,
            snow_blocks: 3,
            blue_ice_from_blocks: 9,
            blue_ice_blocks: 2,
            snow: "minecraft:snow_block".to_owned(),
            ice: "minecraft:packed_ice".to_owned(),
            deep_ice: "minecraft:blue_ice".to_owned(),
        }
    }
}

impl IceCoverSpec {
    /// Blocks of cover for `above_m` metres of ice (or of ground above its equilibrium line): 0 at
    /// or below 0, 1 at its foot, one more per `step_m`, up to `max_blocks`.
    pub fn blocks(&self, above_m: f64) -> u8 {
        if above_m.is_nan() || above_m <= 0.0 {
            return 0;
        }
        let n = 1.0 + (above_m / self.step_m).floor();
        n.min(f64::from(self.max_blocks)) as u8
    }

    /// The `notes.bake.climate.ice_cover` record: what the game's ice pass reads.
    pub fn record(&self) -> Value {
        serde_json::json!({
            "step_m": self.step_m,
            "max_blocks": self.max_blocks,
            "snow_blocks": self.snow_blocks,
            "blue_ice_from_blocks": self.blue_ice_from_blocks,
            "blue_ice_blocks": self.blue_ice_blocks,
            "snow": self.snow,
            "ice": self.ice,
            "deep_ice": self.deep_ice,
        })
    }
}

/// The parsed, checked table.
#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    glacier_cap: f64,
    ramps: Vec<Ramp>,
    rules: Vec<Rule>,
    /// By Köppen index 0 … 30.
    thermal: Vec<Thermal>,
    humidity: Vec<Humid>,
    /// By Köppen index 0 … 30: `Some` for a class with a `glaciated` block.
    glaciated: Vec<Option<Glaciated>>,
    /// By Köppen index 0 … 30: the class's erosion limits.
    erosion: Vec<Option<ErosionLimit>>,
    /// By Köppen index 0 … 30: the class's continentalness limits.
    continentalness: Vec<Option<Limit>>,
    /// By Köppen index 0 … 30: the branch of the variant noise the class's land takes.
    weirdness: Vec<Weirdness>,
    /// The class rules, in order.
    class_rules: Vec<ClassRule>,
    ice_cover: IceCoverSpec,
    /// The table as written (without `_comment`): what `notes.bake.climate.placement` records.
    table: Value,
    /// The pack placement tables applied over the base, in order (their `id`s).
    patches: Vec<String>,
}

/// The default table, parsed once.
pub fn builtin() -> &'static Placement {
    static P: OnceLock<Placement> = OnceLock::new();
    P.get_or_init(|| Placement::parse(PLACEMENT_JSON).expect("placement.json parses"))
}

fn err(path: &str, reason: impl Into<String>) -> CoreError {
    let path = path.trim_start_matches('.');
    let at = if path.is_empty() {
        format!("$.{CONTENT_KEY}")
    } else {
        format!("$.{CONTENT_KEY}.{path}")
    };
    CoreError::preset(at, reason)
}

fn number(v: &Value, path: &str, lo: f64, hi: f64) -> Result<f64, CoreError> {
    let x = v
        .as_f64()
        .filter(|x| x.is_finite())
        .ok_or_else(|| err(path, "expected a number"))?;
    if (lo..=hi).contains(&x) {
        Ok(x)
    } else {
        Err(err(path, format!("must be in [{lo}, {hi}]")))
    }
}

fn object<'a>(v: &'a Value, path: &str) -> Result<&'a Map<String, Value>, CoreError> {
    v.as_object()
        .ok_or_else(|| err(path, "expected a JSON object"))
}

fn no_other_keys(o: &Map<String, Value>, allowed: &[&str], path: &str) -> Result<(), CoreError> {
    match o.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(k) => Err(err(&format!("{path}.{k}"), "unknown key")),
        None => Ok(()),
    }
}

fn string<'a>(v: &'a Value, path: &str) -> Result<&'a str, CoreError> {
    v.as_str().ok_or_else(|| err(path, "expected a string"))
}

/// The index of the temperature ramp a string names.
fn ramp_named(
    ramp_ix: &BTreeMap<&str, usize>,
    name: &Value,
    path: &str,
) -> Result<usize, CoreError> {
    let n = string(name, path)?;
    ramp_ix.get(n).copied().ok_or_else(|| {
        err(
            path,
            format!(
                "no temperature ramp named {n:?} (have {})",
                ramp_ix.keys().copied().collect::<Vec<_>>().join(", ")
            ),
        )
    })
}

/// The index of the humidity rule a string names.
fn rule_named(
    rule_ix: &BTreeMap<&str, usize>,
    name: &Value,
    path: &str,
) -> Result<usize, CoreError> {
    let n = string(name, path)?;
    rule_ix.get(n).copied().ok_or_else(|| {
        err(
            path,
            format!(
                "no humidity rule named {n:?} (have {})",
                rule_ix.keys().copied().collect::<Vec<_>>().join(", ")
            ),
        )
    })
}

/// `{min, max}` erosion limits (vanilla's erosion units, -1 .. 1; at least one key).
fn parse_erosion(v: &Value, path: &str) -> Result<ErosionLimit, CoreError> {
    parse_limit(v, path, (-1.0, 1.0))
}

/// `{min, max}` limits within `range` (at least one key; a missing one is the range's end).
fn parse_limit(v: &Value, path: &str, range: (f64, f64)) -> Result<Limit, CoreError> {
    let o = object(v, path)?;
    no_other_keys(o, &["min", "max"], path)?;
    if o.is_empty() {
        return Err(err(path, "needs at least one of min, max"));
    }
    let bound = |k: &str, default: f64| -> Result<f64, CoreError> {
        match o.get(k) {
            None => Ok(default),
            Some(x) => number(x, &format!("{path}.{k}"), range.0, range.1),
        }
    };
    let (min, max) = (bound("min", range.0)?, bound("max", range.1)?);
    if min > max {
        return Err(err(path, "min is above max"));
    }
    Ok(Limit { min, max })
}

/// `{"of": input, "above": x}` and/or `"below"`: one condition of a class rule.
fn parse_cond(v: &Value, path: &str) -> Result<Cond, CoreError> {
    let o = object(v, path)?;
    no_other_keys(o, &["of", "above", "below"], path)?;
    let of = o
        .get("of")
        .ok_or_else(|| err(&format!("{path}.of"), "missing"))?;
    let of = Input::parse(string(of, &format!("{path}.of"))?)
        .ok_or_else(|| err(&format!("{path}.of"), format!("must be {}", Input::NAMES)))?;
    let (lo, hi) = of.range();
    let bound = |k: &str| -> Result<Option<f64>, CoreError> {
        o.get(k)
            .map(|x| number(x, &format!("{path}.{k}"), lo, hi))
            .transpose()
    };
    let (above, below) = (bound("above")?, bound("below")?);
    if above.is_none() && below.is_none() {
        return Err(err(path, "needs at least one of above, below"));
    }
    if let (Some(a), Some(b)) = (above, below)
        && a >= b
    {
        return Err(err(path, "above must be less than below"));
    }
    Ok(Cond { of, above, below })
}

/// A JSON merge patch (RFC 7386): objects merge key by key, `null` removes a key, anything else
/// replaces the target.
pub fn merge_patch(target: &mut Value, patch: &Value) {
    let Value::Object(p) = patch else {
        *target = patch.clone();
        return;
    };
    if !target.is_object() {
        *target = Value::Object(Map::new());
    }
    let t = target.as_object_mut().expect("made an object");
    for (k, v) in p {
        if v.is_null() {
            t.remove(k);
        } else {
            merge_patch(t.entry(k.clone()).or_insert(Value::Null), v);
        }
    }
}

/// `"positive"`, or `{"positive_below_erosion": e}`.
fn parse_weirdness(v: &Value, path: &str) -> Result<Weirdness, CoreError> {
    match v {
        Value::String(s) if s == "positive" => Ok(Weirdness::Positive),
        Value::Object(o) => {
            no_other_keys(o, &["positive_below_erosion", "flats"], path)?;
            let k = format!("{path}.positive_below_erosion");
            let e = o
                .get("positive_below_erosion")
                .ok_or_else(|| err(&k, "missing"))?;
            let mut flats = Flats::default();
            if let Some(f) = o.get("flats") {
                let fp = format!("{path}.flats");
                let f = object(f, &fp)?;
                no_other_keys(f, &["positive_noise_above", "max_magnitude"], &fp)?;
                if f.is_empty() {
                    return Err(err(
                        &fp,
                        "needs at least one of positive_noise_above, max_magnitude",
                    ));
                }
                if let Some(x) = f.get("positive_noise_above") {
                    flats.positive_noise_above =
                        number(x, &format!("{fp}.positive_noise_above"), -1.0, 1.0)?;
                }
                if let Some(x) = f.get("max_magnitude") {
                    flats.max_magnitude =
                        Some(number(x, &format!("{fp}.max_magnitude"), 0.07, 1.0)?);
                }
            }
            Ok(Weirdness::PositiveBelowErosion {
                erosion: number(e, &k, -1.0, 1.0)?,
                flats,
            })
        }
        _ => Err(err(
            path,
            "must be \"positive\" or {\"positive_below_erosion\": erosion}",
        )),
    }
}

/// A block id such as `minecraft:snow_block`.
fn block_id(id: &str) -> bool {
    let part = |s: &str, extra: &str| {
        !s.is_empty()
            && s.chars().all(|c| {
                c.is_ascii_lowercase()
                    || c.is_ascii_digit()
                    || "_-.".contains(c)
                    || extra.contains(c)
            })
    };
    id.split_once(':')
        .is_some_and(|(ns, path)| part(ns, "") && part(path, "/"))
}

/// The table's `ice_cover` section (every key optional; the defaults are [`IceCoverSpec::default`]).
fn parse_ice_cover(v: Option<&Value>) -> Result<IceCoverSpec, CoreError> {
    let mut spec = IceCoverSpec::default();
    let Some(v) = v else {
        return Ok(spec);
    };
    let o = object(v, "ice_cover")?;
    no_other_keys(
        o,
        &[
            "step_m",
            "max_blocks",
            "snow_blocks",
            "blue_ice_from_blocks",
            "blue_ice_blocks",
            "snow",
            "ice",
            "deep_ice",
        ],
        "ice_cover",
    )?;
    let int = |k: &str, lo: f64, hi: f64, into: &mut u8| -> Result<(), CoreError> {
        if let Some(x) = o.get(k) {
            let n = number(x, &format!("ice_cover.{k}"), lo, hi)?;
            if n.fract() != 0.0 {
                return Err(err(&format!("ice_cover.{k}"), "expected a whole number"));
            }
            *into = n as u8;
        }
        Ok(())
    };
    if let Some(x) = o.get("step_m") {
        spec.step_m = number(x, "ice_cover.step_m", 10.0, 5000.0)?;
    }
    int("max_blocks", 1.0, 64.0, &mut spec.max_blocks)?;
    int("snow_blocks", 0.0, 64.0, &mut spec.snow_blocks)?;
    int(
        "blue_ice_from_blocks",
        1.0,
        65.0,
        &mut spec.blue_ice_from_blocks,
    )?;
    int("blue_ice_blocks", 0.0, 64.0, &mut spec.blue_ice_blocks)?;
    for (k, into) in [
        ("snow", &mut spec.snow),
        ("ice", &mut spec.ice),
        ("deep_ice", &mut spec.deep_ice),
    ] {
        if let Some(x) = o.get(k) {
            let p = format!("ice_cover.{k}");
            let id = string(x, &p)?;
            if !block_id(id) {
                return Err(err(&p, "expected a block id such as minecraft:snow_block"));
            }
            id.clone_into(into);
        }
    }
    if spec.snow_blocks > spec.max_blocks {
        return Err(err("ice_cover.snow_blocks", "is above max_blocks"));
    }
    if spec.blue_ice_blocks > spec.max_blocks {
        return Err(err("ice_cover.blue_ice_blocks", "is above max_blocks"));
    }
    Ok(spec)
}

/// The parts of a `{when, then, else}` switch: the input and threshold of `when`, and the `then` and
/// `else` values for the caller to read.
fn parse_switch<'a>(
    w: &'a Map<String, Value>,
    tp: &str,
) -> Result<(Input, f64, &'a Value, &'a Value), CoreError> {
    no_other_keys(w, &["when", "then", "else"], tp)?;
    let field = |key: &str| {
        w.get(key)
            .ok_or_else(|| err(&format!("{tp}.{key}"), "missing"))
    };
    let when = object(field("when")?, &format!("{tp}.when"))?;
    no_other_keys(when, &["of", "above"], &format!("{tp}.when"))?;
    let of = when
        .get("of")
        .ok_or_else(|| err(&format!("{tp}.when.of"), "missing"))?;
    let of = Input::parse(string(of, &format!("{tp}.when.of"))?).ok_or_else(|| {
        err(
            &format!("{tp}.when.of"),
            format!("must be {}", Input::NAMES),
        )
    })?;
    let above = when
        .get("above")
        .ok_or_else(|| err(&format!("{tp}.when.above"), "missing"))?;
    let (lo, hi) = of.range();
    let above = number(above, &format!("{tp}.when.above"), lo, hi)?;
    Ok((of, above, field("then")?, field("else")?))
}

/// A class's temperature: a ramp name, or a `{when, then, else}` switch whose branches are again
/// ramp names or switches (at most [`MAX_NEST`] deep).
fn parse_thermal(
    t: &Value,
    tp: &str,
    ramp_ix: &BTreeMap<&str, usize>,
    depth: usize,
) -> Result<Thermal, CoreError> {
    match t {
        Value::String(_) => Ok(Thermal::Ramp(ramp_named(ramp_ix, t, tp)?)),
        Value::Object(w) => {
            if depth >= MAX_NEST {
                return Err(err(tp, format!("switches nest at most {MAX_NEST} deep")));
            }
            let (of, above, then, otherwise) = parse_switch(w, tp)?;
            Ok(Thermal::When {
                of,
                above,
                then: Box::new(parse_thermal(
                    then,
                    &format!("{tp}.then"),
                    ramp_ix,
                    depth + 1,
                )?),
                otherwise: Box::new(parse_thermal(
                    otherwise,
                    &format!("{tp}.else"),
                    ramp_ix,
                    depth + 1,
                )?),
            })
        }
        _ => Err(err(
            tp,
            "expected a ramp name or a {when, then, else} switch",
        )),
    }
}

/// A class's humidity: a rule name, or a `{when, then, else}` switch on a temperature whose
/// branches are again rule names or switches (at most [`MAX_NEST`] deep).
fn parse_humid(
    h: &Value,
    hp: &str,
    rule_ix: &BTreeMap<&str, usize>,
    depth: usize,
) -> Result<Humid, CoreError> {
    match h {
        Value::String(_) => Ok(Humid::Rule(rule_named(rule_ix, h, hp)?)),
        Value::Object(w) => {
            if depth >= MAX_NEST {
                return Err(err(hp, format!("switches nest at most {MAX_NEST} deep")));
            }
            let (of, above, then, otherwise) = parse_switch(w, hp)?;
            Ok(Humid::When {
                of,
                above,
                then: Box::new(parse_humid(
                    then,
                    &format!("{hp}.then"),
                    rule_ix,
                    depth + 1,
                )?),
                otherwise: Box::new(parse_humid(
                    otherwise,
                    &format!("{hp}.else"),
                    rule_ix,
                    depth + 1,
                )?),
            })
        }
        _ => Err(err(
            hp,
            "expected a humidity rule name or a {when, then, else} switch",
        )),
    }
}

impl Placement {
    /// Parses and checks a table (the JSON of `placement.json`); every problem names its path.
    pub fn parse(json: &str) -> Result<Placement, CoreError> {
        let v: Value = serde_json::from_str(json).map_err(|e| {
            CoreError::preset(format!("$.{CONTENT_KEY}"), format!("not valid JSON: {e}"))
        })?;
        Placement::from_value(&v)
    }

    /// As [`Placement::parse`], from a parsed value.
    pub fn from_value(v: &Value) -> Result<Placement, CoreError> {
        let root = object(v, "")?;
        no_other_keys(
            root,
            &[
                "schema",
                "_comment",
                "glacier_cap",
                "temperature_ramps",
                "humidity_rules",
                "classes",
                "class_rules",
                "ice_cover",
            ],
            "",
        )?;
        let get = |k: &str| root.get(k).ok_or_else(|| err(k, "missing"));
        if get("schema")?.as_u64() != Some(SCHEMA) {
            return Err(err("schema", format!("must be {SCHEMA}")));
        }
        let glacier_cap = number(get("glacier_cap")?, "glacier_cap", 0.0, TOP)?;

        // Temperature ramps.
        let mut ramp_ix: BTreeMap<&str, usize> = BTreeMap::new();
        let mut ramps = Vec::new();
        for (name, r) in object(get("temperature_ramps")?, "temperature_ramps")? {
            let p = format!("temperature_ramps.{name}");
            let o = object(r, &p)?;
            no_other_keys(o, &["base", "span", "inputs"], &p)?;
            let field = |k: &str| o.get(k).ok_or_else(|| err(&format!("{p}.{k}"), "missing"));
            let base = number(field("base")?, &format!("{p}.base"), 0.0, TOP)?;
            let span = number(field("span")?, &format!("{p}.span"), -TOP, TOP)?;
            let (lo, hi) = (base.min(base + span), base.max(base + span));
            if lo < 0.0 || hi > TOP {
                return Err(err(
                    &p,
                    format!("base {base} + span {span} leaves the bands [0, {TOP}]"),
                ));
            }
            let inputs = field("inputs")?
                .as_array()
                .filter(|a| !a.is_empty())
                .ok_or_else(|| err(&format!("{p}.inputs"), "expected a non-empty list"))?;
            let mut terms = Vec::new();
            let mut total = 0.0;
            for (i, t) in inputs.iter().enumerate() {
                let tp = format!("{p}.inputs.{i}");
                let o = object(t, &tp)?;
                no_other_keys(o, &["of", "from", "to", "weight"], &tp)?;
                let f = |k: &str| o.get(k).ok_or_else(|| err(&format!("{tp}.{k}"), "missing"));
                let of = string(f("of")?, &format!("{tp}.of"))?;
                let of = Input::parse(of)
                    .filter(|i| i.is_temperature())
                    .ok_or_else(|| err(&format!("{tp}.of"), "must be t_mean, t_warm or t_cold"))?;
                let from = number(f("from")?, &format!("{tp}.from"), -100.0, 100.0)?;
                let to = number(f("to")?, &format!("{tp}.to"), -100.0, 100.0)?;
                if from == to {
                    return Err(err(&tp, "from and to must differ"));
                }
                let weight = match o.get("weight") {
                    None => 1.0,
                    Some(w) => number(w, &format!("{tp}.weight"), 0.0, 1.0)?,
                };
                total += weight;
                terms.push(Term {
                    of,
                    from,
                    to,
                    weight,
                });
            }
            if (total - 1.0).abs() > 1e-9 {
                return Err(err(
                    &format!("{p}.inputs"),
                    format!("the weights must add up to 1 (they add up to {total})"),
                ));
            }
            ramp_ix.insert(name, ramps.len());
            ramps.push(Ramp { base, span, terms });
        }

        // Humidity rules.
        let mut rule_ix: BTreeMap<&str, usize> = BTreeMap::new();
        let mut rules = Vec::new();
        for (name, r) in object(get("humidity_rules")?, "humidity_rules")? {
            let p = format!("humidity_rules.{name}");
            let o = object(r, &p)?;
            no_other_keys(o, &["squeeze", "min", "max"], &p)?;
            let opt = |k: &str| -> Result<Option<f64>, CoreError> {
                o.get(k)
                    .map(|x| number(x, &format!("{p}.{k}"), 0.0, TOP))
                    .transpose()
            };
            let (min, max) = (opt("min")?, opt("max")?);
            if let (Some(a), Some(b)) = (min, max)
                && a > b
            {
                return Err(err(&p, "min is above max"));
            }
            let squeeze = match o.get("squeeze") {
                None => None,
                Some(s) => {
                    let sp = format!("{p}.squeeze");
                    let a = s
                        .as_array()
                        .filter(|a| a.len() == 2)
                        .ok_or_else(|| err(&sp, "expected [lo, hi]"))?;
                    let (lo, hi) = (
                        number(&a[0], &format!("{sp}.0"), 0.0, TOP)?,
                        number(&a[1], &format!("{sp}.1"), 0.0, TOP)?,
                    );
                    if lo > hi {
                        return Err(err(&sp, "lo is above hi"));
                    }
                    Some((lo, hi))
                }
            };
            if min.is_none() && max.is_none() && squeeze.is_none() {
                return Err(err(&p, "needs at least one of squeeze, min, max"));
            }
            rule_ix.insert(name, rules.len());
            rules.push(Rule { squeeze, min, max });
        }

        // Classes.
        let classes = object(get("classes")?, "classes")?;
        let land = usize::from(koppen::SEA);
        let mut thermal: Vec<Option<Thermal>> = vec![None; land];
        let mut humidity: Vec<Option<Humid>> = vec![None; land];
        let mut glaciated: Vec<Option<Glaciated>> = vec![None; land];
        let mut erosion: Vec<Option<ErosionLimit>> = vec![None; land];
        let mut continentalness: Vec<Option<Limit>> = vec![None; land];
        let mut weirdness = vec![Weirdness::Noise; land];
        for (code, c) in classes {
            let p = format!("classes.{code}");
            let k = match koppen::index(code) {
                Some(k) if k < koppen::SEA => usize::from(k),
                _ => return Err(err(&p, "not a land Köppen class (Af … EF)")),
            };
            let o = object(c, &p)?;
            no_other_keys(
                o,
                &[
                    "temperature",
                    "humidity",
                    "glaciated",
                    "erosion",
                    "continentalness",
                    "weirdness",
                ],
                &p,
            )?;
            if let Some(w) = o.get("weirdness") {
                weirdness[k] = parse_weirdness(w, &format!("{p}.weirdness"))?;
            }
            if let Some(e) = o.get("erosion") {
                erosion[k] = Some(parse_erosion(e, &format!("{p}.erosion"))?);
            }
            if let Some(e) = o.get("continentalness") {
                continentalness[k] = Some(parse_limit(
                    e,
                    &format!("{p}.continentalness"),
                    CONTINENTS_RANGE,
                )?);
            }
            let tp = format!("{p}.temperature");
            let t = o.get("temperature").ok_or_else(|| err(&tp, "missing"))?;
            thermal[k] = Some(parse_thermal(t, &tp, &ramp_ix, 0)?);
            let hp = format!("{p}.humidity");
            let n = o.get("humidity").ok_or_else(|| err(&hp, "missing"))?;
            humidity[k] = Some(parse_humid(n, &hp, &rule_ix, 0)?);
            if let Some(g) = o.get("glaciated") {
                let gp = format!("{p}.glaciated");
                let g = object(g, &gp)?;
                no_other_keys(g, &["temperature", "humidity", "erosion"], &gp)?;
                if g.is_empty() {
                    return Err(err(
                        &gp,
                        "needs at least one of temperature, humidity, erosion",
                    ));
                }
                let thermal = match g.get("temperature") {
                    None => None,
                    Some(t) => Some(parse_thermal(t, &format!("{gp}.temperature"), &ramp_ix, 0)?),
                };
                let humidity = match g.get("humidity") {
                    None => None,
                    Some(n) => Some(parse_humid(n, &format!("{gp}.humidity"), &rule_ix, 0)?),
                };
                let erosion = match g.get("erosion") {
                    None => None,
                    Some(e) => Some(parse_erosion(e, &format!("{gp}.erosion"))?),
                };
                glaciated[k] = Some(Glaciated {
                    thermal,
                    humidity,
                    erosion,
                });
            }
        }
        let missing: Vec<&str> = (0..land)
            .filter(|&k| thermal[k].is_none())
            .map(|k| koppen::CODES[k])
            .collect();
        if !missing.is_empty() {
            return Err(err(
                "classes",
                format!(
                    "every land class needs an entry; missing {}",
                    missing.join(", ")
                ),
            ));
        }
        // Class rules.
        let mut class_rules = Vec::new();
        if let Some(rs) = root.get("class_rules") {
            let rs = rs
                .as_array()
                .ok_or_else(|| err("class_rules", "expected a list"))?;
            for (i, r) in rs.iter().enumerate() {
                let p = format!("class_rules.{i}");
                let o = object(r, &p)?;
                no_other_keys(
                    o,
                    &[
                        "name",
                        "_comment",
                        "classes",
                        "when",
                        "temperature",
                        "humidity",
                        "erosion",
                        "continentalness",
                    ],
                    &p,
                )?;
                let name = string(
                    o.get("name")
                        .ok_or_else(|| err(&format!("{p}.name"), "missing"))?,
                    &format!("{p}.name"),
                )?
                .to_owned();
                let cp = format!("{p}.classes");
                let codes = o
                    .get("classes")
                    .and_then(Value::as_array)
                    .filter(|a| !a.is_empty())
                    .ok_or_else(|| err(&cp, "expected a non-empty list of land classes"))?;
                let mut classes = 0u32;
                for (j, c) in codes.iter().enumerate() {
                    let jp = format!("{cp}.{j}");
                    match koppen::index(string(c, &jp)?) {
                        Some(k) if k < koppen::SEA => classes |= 1 << k,
                        _ => return Err(err(&jp, "not a land Köppen class (Af … EF)")),
                    }
                }
                let wp = format!("{p}.when");
                let when = o
                    .get("when")
                    .and_then(Value::as_array)
                    .filter(|a| !a.is_empty())
                    .ok_or_else(|| err(&wp, "expected a non-empty list of conditions"))?
                    .iter()
                    .enumerate()
                    .map(|(j, c)| parse_cond(c, &format!("{wp}.{j}")))
                    .collect::<Result<Vec<_>, _>>()?;
                let thermal = o
                    .get("temperature")
                    .map(|t| parse_thermal(t, &format!("{p}.temperature"), &ramp_ix, 0))
                    .transpose()?;
                let humidity = o
                    .get("humidity")
                    .map(|h| parse_humid(h, &format!("{p}.humidity"), &rule_ix, 0))
                    .transpose()?;
                let erosion = o
                    .get("erosion")
                    .map(|e| parse_erosion(e, &format!("{p}.erosion")))
                    .transpose()?;
                let continentalness = o
                    .get("continentalness")
                    .map(|e| parse_limit(e, &format!("{p}.continentalness"), CONTINENTS_RANGE))
                    .transpose()?;
                if thermal.is_none()
                    && humidity.is_none()
                    && erosion.is_none()
                    && continentalness.is_none()
                {
                    return Err(err(
                        &p,
                        "needs at least one of temperature, humidity, erosion, continentalness",
                    ));
                }
                class_rules.push(ClassRule {
                    name,
                    classes,
                    when,
                    thermal,
                    humidity,
                    erosion,
                    continentalness,
                });
            }
        }
        let ice_cover = parse_ice_cover(root.get("ice_cover"))?;
        let mut table = root.clone();
        table.remove("_comment");
        Ok(Placement {
            glacier_cap,
            ramps,
            rules,
            thermal: thermal.into_iter().flatten().collect(),
            humidity: humidity.into_iter().flatten().collect(),
            glaciated,
            erosion,
            continentalness,
            weirdness,
            class_rules,
            ice_cover,
            table: Value::Object(table),
            patches: Vec::new(),
        })
    }

    /// The table as written, without its comment.
    pub fn table(&self) -> &Value {
        &self.table
    }

    /// The pack placement tables applied over the base table, in order (their `id`s).
    pub fn patches(&self) -> &[String] {
        &self.patches
    }

    /// The class rules' names, in order (what [`Placed::rule`] indexes).
    pub fn rule_names(&self) -> Vec<&str> {
        self.class_rules.iter().map(|r| r.name.as_str()).collect()
    }

    /// The base table (`base`, the table as written; `None` for the default) with the pack placement
    /// tables `packs` applied in order (FORMAT.md, "Pack placement tables"): each is
    /// `{"schema": 1, "id": "...", "patch": {...}}` (plus the game's own `requires_biomes`, `priority`
    /// and `_comment`), its `patch` a JSON merge patch (RFC 7386) on the table so far. The result is
    /// parsed and checked whole; an error names the pack and, inside it, the path.
    pub fn with_packs(base: Option<&Value>, packs: &[Value]) -> Result<Placement, CoreError> {
        let mut table = match base {
            Some(b) => {
                let mut t = b.clone();
                if let Some(o) = t.as_object_mut() {
                    o.remove("_comment");
                }
                t
            }
            None => builtin().table().clone(),
        };
        let mut ids = Vec::new();
        for (i, pack) in packs.iter().enumerate() {
            let p = format!("$.{PATCHES_KEY}.{i}");
            let o = pack
                .as_object()
                .ok_or_else(|| CoreError::preset(&p, "expected a JSON object"))?;
            if let Some(k) = o.keys().find(|k| {
                ![
                    "schema",
                    "_comment",
                    "id",
                    "requires_biomes",
                    "priority",
                    "patch",
                ]
                .contains(&k.as_str())
            }) {
                return Err(CoreError::preset(format!("{p}.{k}"), "unknown key"));
            }
            if o.get("schema").and_then(Value::as_u64) != Some(SCHEMA) {
                return Err(CoreError::preset(
                    format!("{p}.schema"),
                    format!("must be {SCHEMA}"),
                ));
            }
            let id = o
                .get("id")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| CoreError::preset(format!("{p}.id"), "expected a name"))?;
            let patch = o
                .get("patch")
                .filter(|v| v.is_object())
                .ok_or_else(|| CoreError::preset(format!("{p}.patch"), "expected a JSON object"))?;
            merge_patch(&mut table, patch);
            if let Some(t) = table.as_object_mut() {
                t.remove("_comment");
            }
            ids.push(id.to_owned());
        }
        let mut placed = Placement::from_value(&table).map_err(|e| match e {
            CoreError::PresetInvalid { path, reason } if !ids.is_empty() => CoreError::preset(
                path,
                format!("{reason} (in the table after the packs {})", ids.join(", ")),
            ),
            e => e,
        })?;
        placed.patches = ids;
        Ok(placed)
    }

    fn ramp_value(&self, r: usize, s: &Site) -> f64 {
        let r = &self.ramps[r];
        let v = r.terms.iter().fold(0.0, |v, t| {
            let x = s.input(t.of);
            v + t.weight * unit((x - t.from) / (t.to - t.from))
        });
        r.base + r.span * v
    }

    fn thermal_value(&self, t: &Thermal, s: &Site) -> f64 {
        match t {
            Thermal::Ramp(r) => self.ramp_value(*r, s),
            Thermal::When {
                of,
                above,
                then,
                otherwise,
            } => {
                let branch = if s.input(*of) > *above {
                    then
                } else {
                    otherwise
                };
                self.thermal_value(branch, s)
            }
        }
    }

    /// The humidity rule a class's [`Humid`] picks at this site.
    fn humid_rule(h: &Humid, s: &Site) -> usize {
        match h {
            Humid::Rule(r) => *r,
            Humid::When {
                of,
                above,
                then,
                otherwise,
            } => {
                let branch = if s.input(*of) > *above {
                    then
                } else {
                    otherwise
                };
                Self::humid_rule(branch, s)
            }
        }
    }

    /// The humidity band coordinate rule `r` gives the aridity index's own band coordinate `h`.
    fn apply_rule(&self, r: usize, mut h: f64) -> f64 {
        let r = &self.rules[r];
        if let Some((lo, hi)) = r.squeeze {
            h = lo + (hi - lo) * unit((h - 1.0) / 3.0);
        }
        if let Some(m) = r.min {
            h = h.max(m);
        }
        if let Some(m) = r.max {
            h = h.min(m);
        }
        h
    }

    /// The first class rule of class `k` that holds at `s` (never on glaciated land).
    fn class_rule(&self, k: u8, s: &Site) -> Option<usize> {
        if s.glacier || k >= koppen::SEA {
            return None;
        }
        self.class_rules
            .iter()
            .position(|r| r.classes & (1 << k) != 0 && r.when.iter().all(|c| c.holds(s)))
    }

    /// Where the table puts a land cell of Köppen class `k` at `s`: [`Placement::thermal_land`],
    /// [`Placement::humidity_land`], [`Placement::erosion_limit`] and [`Placement::weirdness`] at
    /// once, with the class rules applied (the first rule of the class whose conditions all hold
    /// replaces what it names of the class's own; never on glaciated land).
    pub fn place(&self, k: u8, s: &Site) -> Placed {
        let rule = self.class_rule(k, s);
        let r = rule.map(|i| &self.class_rules[i]);
        let thermal = match r.and_then(|r| r.thermal.as_ref()) {
            Some(t) => self.thermal_value(t, s),
            None => self.thermal_at(k, s),
        };
        let humidity = match r.and_then(|r| r.humidity.as_ref()) {
            Some(h) => self.apply_rule(Self::humid_rule(h, s), humidity_of_aridity(s.aridity)),
            None => self.humidity_at(k, s),
        };
        let erosion = r
            .and_then(|r| r.erosion)
            .or_else(|| self.erosion_limit(k, s.glacier));
        let continentalness = r
            .and_then(|r| r.continentalness)
            .or_else(|| self.continentalness.get(usize::from(k)).copied().flatten());
        Placed {
            thermal,
            humidity,
            erosion,
            continentalness,
            weirdness: self.weirdness(k),
            rule,
        }
    }

    /// The branch of the variant noise class `k`'s land takes (the positive one is vanilla's
    /// rising-weirdness biomes: frozen peaks, never the falling branch's jagged peaks and
    /// Terralith's scarlet mountains in their place).
    pub fn weirdness(&self, k: u8) -> Weirdness {
        self.weirdness
            .get(usize::from(k))
            .copied()
            .unwrap_or(Weirdness::Noise)
    }

    /// The erosion limits of a land cell of class `k`: where the glacier flag is set the class's
    /// `glaciated` block's, when it has one, else the class's own; `None` for no limit.
    pub fn erosion_limit(&self, k: u8, glacier: bool) -> Option<ErosionLimit> {
        let own = self.erosion.get(usize::from(k)).copied().flatten();
        if glacier && let Some(e) = self.glaciated_block(k).and_then(|g| g.erosion) {
            return Some(e);
        }
        own
    }

    /// The ice cover the bake lays and the game's ice pass reads.
    pub fn ice_cover(&self) -> &IceCoverSpec {
        &self.ice_cover
    }

    /// The `glaciated` block of class `k`, if it has one.
    fn glaciated_block(&self, k: u8) -> Option<&Glaciated> {
        self.glaciated.get(usize::from(k)).and_then(Option::as_ref)
    }

    /// Temperature band coordinate of a land cell of Köppen class `k` (a sea class falls back to
    /// the sea's own curve). Where the glacier flag is set the class's `glaciated` block decides
    /// (its temperature, else the class's own, both uncapped); a class without one is held at or
    /// below the table's `glacier_cap`.
    ///
    /// The class rules are not applied (their latitude and height are unknown here): see
    /// [`Placement::place`].
    pub fn thermal_land(&self, k: u8, t_mean: f64, t_warm: f64, t_cold: f64, glacier: bool) -> f64 {
        self.thermal_at(k, &Site::of(t_mean, t_warm, t_cold, f64::NAN, glacier))
    }

    fn thermal_at(&self, k: u8, s: &Site) -> f64 {
        let own = || {
            self.thermal
                .get(usize::from(k))
                .map_or_else(|| thermal_sea(s.t_mean), |t| self.thermal_value(t, s))
        };
        if !s.glacier {
            return own();
        }
        match self.glaciated_block(k) {
            Some(g) => g
                .thermal
                .as_ref()
                .map_or_else(own, |t| self.thermal_value(t, s)),
            None => own().min(self.glacier_cap),
        }
    }

    /// Humidity band coordinate of a land cell of Köppen class `k` and aridity index `aridity`
    /// (the temperatures decide a class's humidity only when its entry is a `when` switch); where
    /// the glacier flag is set, the class's `glaciated` humidity rule when it has one.
    ///
    /// The class rules are not applied (their latitude and height are unknown here): see
    /// [`Placement::place`].
    pub fn humidity_land(
        &self,
        k: u8,
        aridity: f64,
        t_mean: f64,
        t_warm: f64,
        t_cold: f64,
        glacier: bool,
    ) -> f64 {
        self.humidity_at(k, &Site::of(t_mean, t_warm, t_cold, aridity, glacier))
    }

    fn humidity_at(&self, k: u8, s: &Site) -> f64 {
        let h = humidity_of_aridity(s.aridity);
        let rule = s
            .glacier
            .then(|| self.glaciated_block(k).and_then(|g| g.humidity.as_ref()))
            .flatten()
            .or_else(|| self.humidity.get(usize::from(k)))
            .map(|r| Self::humid_rule(r, s));
        match rule {
            Some(r) => self.apply_rule(r, h),
            None => h,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// P1-09's compiled-in constants, kept here as the oracle the table must reproduce.
    mod old {
        use crate::bake::biomes::{humidity_of_aridity, thermal_sea, unit};

        fn thermal_d(t_warm: f64, t_cold: f64) -> f64 {
            0.75 + 1.25
                * (0.65 * unit((t_warm - 10.0) / 12.0) + 0.35 * unit((t_cold + 30.0) / 30.0))
        }

        fn thermal_c(t_cold: f64) -> f64 {
            2.0 + 0.95 * unit(t_cold / 18.0)
        }

        pub fn thermal_land(k: u8, t_mean: f64, t_warm: f64, t_cold: f64, glacier: bool) -> f64 {
            let b = match k {
                29 | 30 => 0.75 * unit((t_warm + 20.0) / 30.0),
                17..=28 => thermal_d(t_warm, t_cold),
                8..=16 => thermal_c(t_cold),
                0..=3 => 3.0 + 0.9 * unit((t_cold - 18.0) / 10.0),
                4 | 5 => 4.05 + 0.9 * unit((t_mean - 5.0) / 25.0),
                6 => 3.0 + 0.9 * unit((t_mean - 18.0) / 12.0),
                7 => {
                    if t_cold > 0.0 {
                        thermal_c(t_cold)
                    } else {
                        thermal_d(t_warm, t_cold)
                    }
                }
                _ => thermal_sea(t_mean),
            };
            if glacier { b.min(0.35) } else { b }
        }

        pub fn humidity_land(k: u8, aridity: f64) -> f64 {
            let h = humidity_of_aridity(aridity);
            let squeeze = |lo: f64, hi: f64| lo + (hi - lo) * unit((h - 1.0) / 3.0);
            match k {
                0 => h.max(4.1),
                1 => h.max(3.2),
                2 | 3 => squeeze(1.05, 2.55),
                4 | 5 => h.clamp(0.0, 0.95),
                6 | 7 => h.clamp(1.05, 1.95),
                8..=10 => squeeze(1.05, 2.5),
                11..=16 => h.max(2.05),
                19 | 20 | 23 | 24 | 27 | 28 => h.max(3.05),
                17 | 18 | 21 | 22 | 25 | 26 => h.max(2.05),
                29 | 30 => h.min(1.95),
                _ => h,
            }
        }
    }

    /// EF (index 30) is the one class the default table no longer takes from P1-09's constants: it
    /// is ice cap where glaciated and barren frozen ground elsewhere (its own tests below).
    const EF: u8 = 30;

    /// BWk (index 5) is the other: a cold desert is not a hot one below a mean of 10 degC (its own
    /// tests below); at and above 10 degC it is P1-09's desert exactly.
    const BWK: u8 = 5;

    #[test]
    fn the_default_table_reproduces_the_compiled_in_constants_bit_for_bit() {
        let p = builtin();
        // A grid over every class but EF and BWk, the temperatures the classes live in and the
        // aridity's range.
        let mut n = 0u64;
        for k in (0..32u8).filter(|&k| k != EF && k != BWK) {
            for tm in (-60..=45).step_by(3) {
                let tm = f64::from(tm);
                for tw in [tm, tm + 5.0, tm + 12.0] {
                    for tc in [tm - 25.0, tm - 8.0, tm - 1.0, -0.4, 0.0, 0.4, 6.0] {
                        for glacier in [false, true] {
                            let (a, b) = (
                                p.thermal_land(k, tm, tw, tc, glacier),
                                old::thermal_land(k, tm, tw, tc, glacier),
                            );
                            assert_eq!(
                                a.to_bits(),
                                b.to_bits(),
                                "class {k} {tm} {tw} {tc} {glacier}"
                            );
                            n += 1;
                        }
                    }
                }
            }
            for i in 0..400 {
                let ai = 0.005 * 1.03f64.powi(i);
                assert_eq!(
                    p.humidity_land(k, ai, 10.0, 20.0, 0.0, false).to_bits(),
                    old::humidity_land(k, ai).to_bits(),
                    "class {k} aridity {ai}"
                );
                // No class but EF has a glaciated humidity rule.
                assert_eq!(
                    p.humidity_land(k, ai, 10.0, 20.0, 0.0, true).to_bits(),
                    old::humidity_land(k, ai).to_bits(),
                    "class {k} aridity {ai} glaciated"
                );
            }
        }
        assert!(n > 10_000);
    }

    /// Ben, 2026-10-02 (cold deserts): a BWk below a mean of 10 degC is not drawn as sand desert.
    /// Above 10 degC it keeps the desert ramp; between 0 and 10 it sits in the cold band with a dry
    /// humidity; at or below 0 it is barren frozen ground like EF's, fixed at humidity 1.5.
    #[test]
    fn a_cold_desert_is_placed_by_its_mean_temperature() {
        let p = builtin();
        let bwh = koppen::index("BWh").unwrap();
        assert_eq!(koppen::index("BWk"), Some(BWK));
        for tm in [10.5, 14.0, 18.0, 25.0] {
            for ai in [0.01, 0.1, 0.19] {
                assert_eq!(
                    p.thermal_land(BWK, tm, tm + 12.0, tm - 12.0, false)
                        .to_bits(),
                    p.thermal_land(bwh, tm, tm + 12.0, tm - 12.0, false)
                        .to_bits(),
                    "BWk at {tm} degC is the hot desert's"
                );
                assert_eq!(
                    p.humidity_land(BWK, ai, tm, tm + 12.0, tm - 12.0, false)
                        .to_bits(),
                    p.humidity_land(bwh, ai, tm, tm + 12.0, tm - 12.0, false)
                        .to_bits()
                );
            }
        }
        let mut last = 0.0;
        for tm in [0.5, 2.0, 5.0, 8.0, 10.0] {
            let t = p.thermal_land(BWK, tm, tm + 14.0, tm - 14.0, false);
            assert!((1.0..2.0).contains(&t), "BWk at {tm} degC: band {t}");
            assert!(t > last, "warmer is further up the band");
            last = t;
            for ai in [0.001, 0.05, 0.19, 0.4] {
                let h = p.humidity_land(BWK, ai, tm, tm + 14.0, tm - 14.0, false);
                assert!(h <= 0.6 + 1e-12, "BWk at {tm} degC, aridity {ai}: band {h}");
            }
        }
        for (tm, tw) in [(-0.1, 10.0), (-3.0, 12.0), (-9.0, 4.0), (-20.0, -5.0)] {
            let t = p.thermal_land(BWK, tm, tw, tm - 15.0, false);
            // The upper frozen band, where Terralith's gravel desert and cold shrubland are.
            assert!((0.5..1.0).contains(&t), "BWk at {tm} degC: band {t}");
            for ai in [0.001, 0.1, 0.4] {
                assert_eq!(p.humidity_land(BWK, ai, tm, tw, tm - 15.0, false), 1.5);
            }
        }
        // Vanilla's table has no cold desert: the cold branches fall on its dry cold land, not on
        // the hot band's desert (band 4).
        for tm in [-15.0, -3.0, 0.5, 4.0, 9.9] {
            assert!(p.thermal_land(BWK, tm, tm + 12.0, tm - 12.0, false) < 2.0);
        }
        // The hot desert class is as P1-09 had it.
        assert_eq!(
            p.thermal_land(bwh, 5.0, 30.0, -10.0, false).to_bits(),
            old::thermal_land(4, 5.0, 30.0, -10.0, false).to_bits()
        );
    }

    #[test]
    fn switches_nest_to_a_limit_and_name_the_path_of_a_bad_branch() {
        let leaf = |depth: usize| {
            let mut v = json!("desert");
            for _ in 0..depth {
                v = json!({"when": {"of": "t_mean", "above": 1.0}, "then": "desert", "else": v});
            }
            v
        };
        assert!(edited(|v| v["classes"]["BWk"]["temperature"] = leaf(MAX_NEST)).is_ok());
        assert!(edited(|v| v["classes"]["BWk"]["humidity"] = leaf(MAX_NEST)).is_ok());
        let temp = edited(|v| v["classes"]["BWk"]["temperature"] = leaf(MAX_NEST + 1));
        assert!(path_of(temp).starts_with("$.climate_placement.classes.BWk.temperature"));
        let hum = edited(|v| v["classes"]["BWk"]["humidity"] = leaf(MAX_NEST + 1));
        assert!(path_of(hum).starts_with("$.climate_placement.classes.BWk.humidity"));
        let bad = |key: &str, name: &str| {
            path_of(edited(|v| {
                v["classes"]["BWk"][key]["else"]["else"] = json!(name);
            }))
        };
        assert_eq!(
            bad("temperature", "no_such_ramp"),
            "$.climate_placement.classes.BWk.temperature.else.else"
        );
        assert_eq!(
            bad("humidity", "no_such_rule"),
            "$.climate_placement.classes.BWk.humidity.else.else"
        );
        // A humidity switch reads the temperatures it names.
        let q = edited(|v| {
            v["classes"]["Af"]["humidity"] = json!({
                "when": {"of": "t_cold", "above": 20.0}, "then": "rainforest", "else": "tundra"
            });
        })
        .unwrap();
        let af = koppen::index("Af").unwrap();
        assert!(q.humidity_land(af, 2.5, 26.0, 28.0, 22.0, false) >= 4.1);
        assert!(q.humidity_land(af, 2.5, 26.0, 28.0, 18.0, false) <= 1.95);
    }

    fn edited(f: impl FnOnce(&mut Value)) -> Result<Placement, CoreError> {
        let mut v: Value = serde_json::from_str(PLACEMENT_JSON).unwrap();
        f(&mut v);
        Placement::from_value(&v)
    }

    fn path_of(r: Result<Placement, CoreError>) -> String {
        match r.expect_err("must be refused") {
            CoreError::PresetInvalid { path, .. } => path,
            e => panic!("{e}"),
        }
    }

    #[test]
    fn a_changed_table_changes_the_answers() {
        let p = edited(|v| {
            v["classes"]["Af"]["temperature"] = json!("desert");
            v["classes"]["Af"]["humidity"] = json!("tundra");
            v["glacier_cap"] = json!(0.1);
        })
        .unwrap();
        let af = koppen::index("Af").unwrap();
        // Af: hot and wet by the default table, a desert temperature and a dry ceiling by this one.
        let (d, c) = (builtin(), &p);
        assert!(d.thermal_land(af, 27.0, 28.0, 26.0, false) < 4.0);
        assert!(c.thermal_land(af, 27.0, 28.0, 26.0, false) > 4.05);
        assert!(d.humidity_land(af, 2.5, 10.0, 20.0, 0.0, false) >= 4.1);
        assert!(c.humidity_land(af, 2.5, 10.0, 20.0, 0.0, false) <= 1.95);
        // The rest is untouched, and the glacier cap moved.
        let cfb = koppen::index("Cfb").unwrap();
        assert_eq!(
            d.thermal_land(cfb, 11.0, 18.7, 5.0, false),
            c.thermal_land(cfb, 11.0, 18.7, 5.0, false)
        );
        assert_eq!(c.thermal_land(cfb, 11.0, 18.7, 5.0, true), 0.1);
        assert_ne!(d, c);
        assert_eq!(c.table()["glacier_cap"], json!(0.1));
        assert!(c.table().get("_comment").is_none());
    }

    /// Ben, 2026-09-29, "Ice only where there's ice": glaciated EF is ice cap, the rest of EF is
    /// barren frozen ground, both in the upper half of the frozen band, and neither follows P/PET
    /// (which is pinned at 10 below freezing); ET is as it was.
    #[test]
    fn ef_is_ice_cap_where_glaciated_and_barren_frozen_ground_elsewhere() {
        let p = builtin();
        for (t_mean, t_warm, t_cold) in [
            (-45.0, -35.0, -55.0),
            (-12.0, -0.3, -25.0),
            (-30.0, -20.0, -40.0),
        ] {
            for glacier in [false, true] {
                let t = p.thermal_land(EF, t_mean, t_warm, t_cold, glacier);
                // Terralith's dry cold biomes start at band 0.4964 (temperature -0.7274); vanilla's
                // frozen band ends at 1.
                assert!(
                    (0.5..1.0).contains(&t),
                    "EF {t_warm} glaciated {glacier}: band {t}"
                );
            }
            for ai in [0.001, 0.4, 1.0, 10.0] {
                // Fixed rules: the aridity index does not matter to a freezing climate.
                assert_eq!(
                    p.humidity_land(EF, ai, 10.0, 20.0, 0.0, true),
                    0.5,
                    "ice cap"
                );
                assert_eq!(
                    p.humidity_land(EF, ai, 10.0, 20.0, 0.0, false),
                    1.5,
                    "barren"
                );
            }
        }
        // Humidity band 0 (below vegetation -0.35) is where vanilla puts ice spikes and Terralith
        // no forest; band 1 (-0.35 … -0.1) stays under the band whose variants are taiga and grove.
        assert!(p.humidity_land(EF, 10.0, 10.0, 20.0, 0.0, true) < 1.0);
        assert!((1.0..2.0).contains(&p.humidity_land(EF, 10.0, 10.0, 20.0, 0.0, false)));
        // The glacier cap no longer holds EF below band 0.35 (Terralith's snowy badlands).
        assert!(
            p.thermal_land(EF, -45.0, -35.0, -55.0, true)
                > p.table()["glacier_cap"].as_f64().unwrap()
        );
        // ET keeps its ramp and its humidity rule, and glaciated ET is still capped.
        let et = koppen::index("ET").unwrap();
        assert_eq!(
            p.thermal_land(et, -6.0, 6.0, -15.0, false),
            0.75 * (26.0f64 / 30.0)
        );
        assert_eq!(p.thermal_land(et, -6.0, 6.0, -15.0, true), 0.35);
        assert_eq!(p.humidity_land(et, 5.0, 10.0, 20.0, 0.0, false), 1.95);
        assert_eq!(p.humidity_land(et, 5.0, 10.0, 20.0, 0.0, true), 1.95);
    }

    #[test]
    fn ef_takes_its_peaks_positive_and_its_flats_mostly_negative_and_bad_values_name_their_path() {
        let p = builtin();
        for k in 0..32u8 {
            let want = if k == EF {
                Weirdness::PositiveBelowErosion {
                    erosion: -0.27,
                    flats: Flats {
                        positive_noise_above: 0.2,
                        max_magnitude: Some(0.25),
                    },
                }
            } else {
                Weirdness::Noise
            };
            assert_eq!(p.weirdness(k), want, "class {k}");
        }
        let et = koppen::index("ET").unwrap();
        let q = edited(|v| v["classes"]["ET"]["weirdness"] = json!("positive")).unwrap();
        assert_eq!(q.weirdness(et), Weirdness::Positive);
        assert_eq!(q.table()["classes"]["ET"]["weirdness"], json!("positive"));
        let path = |w: Value| path_of(edited(|v| v["classes"]["EF"]["weirdness"] = w));
        let at = "$.climate_placement.classes.EF.weirdness";
        for bad in [json!("negative"), json!(1)] {
            assert_eq!(path(bad), at);
        }
        assert_eq!(path(json!({})), format!("{at}.positive_below_erosion"));
        assert_eq!(
            path(json!({"positive_below_erosion": 3})),
            format!("{at}.positive_below_erosion")
        );
        assert_eq!(path(json!({"x": 1})), format!("{at}.x"));
        // Without `flats` the flats are all negative and unclamped.
        let q =
            edited(|v| v["classes"]["EF"]["weirdness"] = json!({"positive_below_erosion": -0.3}))
                .unwrap();
        let ef = Weirdness::PositiveBelowErosion {
            erosion: -0.3,
            flats: Flats::default(),
        };
        assert_eq!(q.weirdness(EF), ef);
        assert_eq!(Flats::default().positive_noise_above, 1.0);
        let flats = |f: Value| path(json!({"positive_below_erosion": -0.3, "flats": f}));
        assert_eq!(flats(json!({})), format!("{at}.flats"));
        assert_eq!(flats(json!(1)), format!("{at}.flats"));
        assert_eq!(flats(json!({"x": 1})), format!("{at}.flats.x"));
        assert_eq!(
            flats(json!({"positive_noise_above": 2})),
            format!("{at}.flats.positive_noise_above")
        );
        assert_eq!(
            flats(json!({"max_magnitude": 0.05})),
            format!("{at}.flats.max_magnitude"),
            "under the valley slice: rivers"
        );
    }

    #[test]
    fn erosion_limits_are_the_class_s_and_a_glaciated_block_s() {
        let p = builtin();
        for k in (0..32u8).filter(|&k| k != EF) {
            assert_eq!(p.erosion_limit(k, false), None, "class {k}");
            assert_eq!(p.erosion_limit(k, true), None, "class {k} glaciated");
        }
        // Barren EF is held below erosion 0.3 (off the glacial chasm); ice cap is free.
        let own = ErosionLimit {
            min: -1.0,
            max: 0.3,
        };
        assert_eq!(p.erosion_limit(EF, false), Some(own));
        assert_eq!(
            p.erosion_limit(EF, true),
            Some(ErosionLimit {
                min: -1.0,
                max: 1.0
            })
        );
        // A glaciated block without an erosion key falls back to the class's own.
        let q =
            edited(|v| v["classes"]["EF"]["glaciated"] = json!({"humidity": "ice_cap"})).unwrap();
        assert_eq!(q.erosion_limit(EF, true), Some(own));
        // Only a min is a floor.
        let q = edited(|v| v["classes"]["ET"]["erosion"] = json!({"min": -0.2})).unwrap();
        let et = koppen::index("ET").unwrap();
        assert_eq!(
            q.erosion_limit(et, false),
            Some(ErosionLimit {
                min: -0.2,
                max: 1.0
            })
        );
        let at = "$.climate_placement.classes.EF";
        for (bad, path) in [
            (json!({}), format!("{at}.erosion")),
            (json!({"min": 0.5, "max": 0.1}), format!("{at}.erosion")),
            (json!({"max": 2}), format!("{at}.erosion.max")),
            (json!({"mid": 0}), format!("{at}.erosion.mid")),
        ] {
            assert_eq!(
                path_of(edited(|v| v["classes"]["EF"]["erosion"] = bad.clone())),
                path
            );
        }
        assert_eq!(
            path_of(edited(
                |v| v["classes"]["EF"]["glaciated"]["erosion"] = json!({"max": "x"})
            )),
            format!("{at}.glaciated.erosion.max")
        );
    }

    #[test]
    fn the_ice_cover_section_has_defaults_and_names_its_bad_keys() {
        let d = builtin().ice_cover();
        // The jar's file says what the defaults say; a table without the section gets them.
        assert_eq!(*d, IceCoverSpec::default());
        let q = edited(|v| {
            v.as_object_mut().unwrap().remove("ice_cover");
        })
        .unwrap();
        assert_eq!(q.ice_cover(), d);
        let q = edited(|v| v["ice_cover"]["snow_blocks"] = json!(1)).unwrap();
        assert_eq!(q.ice_cover().snow_blocks, 1);
        assert_eq!(q.ice_cover().record()["snow_blocks"], json!(1));
        assert_eq!(
            q.ice_cover().record()["deep_ice"],
            json!("minecraft:blue_ice")
        );
        let at = "$.climate_placement.ice_cover";
        for (key, bad) in [
            ("step_m", json!(1.0)),
            ("max_blocks", json!(100)),
            ("max_blocks", json!(2.5)),
            ("snow", json!("Snow Block")),
            ("ice", json!(3)),
            ("colour", json!("white")),
        ] {
            assert_eq!(
                path_of(edited(|v| v["ice_cover"][key] = bad.clone())),
                format!("{at}.{key}"),
                "{key}"
            );
        }
        // More snow than cover.
        assert_eq!(
            path_of(edited(|v| {
                v["ice_cover"]["max_blocks"] = json!(2);
                v["ice_cover"]["snow_blocks"] = json!(3);
            })),
            format!("{at}.snow_blocks")
        );
        // The cover's thickness: 1 at the line's foot, one more per step, capped.
        assert_eq!(
            [-5.0, 0.0, 1.0, 159.9, 160.0, 479.0, 480.0, 1.0e9].map(|m| d.blocks(m)),
            [0, 0, 1, 1, 2, 3, 4, 12]
        );
    }

    #[test]
    fn a_glaciated_block_replaces_the_cap_for_its_class_only() {
        let p = edited(|v| {
            v["classes"]["Dfc"]["glaciated"] = json!({"humidity": "ice_cap"});
        })
        .unwrap();
        let d = builtin();
        let dfc = koppen::index("Dfc").unwrap();
        let dfd = koppen::index("Dfd").unwrap();
        // Dfc: temperature falls back to its own ramp, uncapped; humidity from the block.
        assert!(d.thermal_land(dfc, 1.0, 15.0, -25.0, true) <= 0.35);
        assert_eq!(
            p.thermal_land(dfc, 1.0, 15.0, -25.0, true),
            d.thermal_land(dfc, 1.0, 15.0, -25.0, false)
        );
        assert_eq!(p.humidity_land(dfc, 4.0, 10.0, 20.0, 0.0, true), 0.5);
        assert_eq!(
            p.humidity_land(dfc, 4.0, 10.0, 20.0, 0.0, false),
            d.humidity_land(dfc, 4.0, 10.0, 20.0, 0.0, false)
        );
        // Dfd has no block: as before.
        assert_eq!(
            p.thermal_land(dfd, 1.0, 15.0, -25.0, true),
            d.thermal_land(dfd, 1.0, 15.0, -25.0, true)
        );
        assert_ne!(p, *d);
        assert_eq!(
            p.table()["classes"]["Dfc"]["glaciated"],
            json!({"humidity": "ice_cap"})
        );
    }

    #[test]
    fn a_bad_glaciated_block_names_its_path() {
        let path = |g: Value| {
            path_of(edited(|v| {
                v["classes"]["EF"]["glaciated"] = g;
            }))
        };
        assert_eq!(path(json!(3)), "$.climate_placement.classes.EF.glaciated");
        assert_eq!(path(json!({})), "$.climate_placement.classes.EF.glaciated");
        assert_eq!(
            path(json!({"colour": "blue"})),
            "$.climate_placement.classes.EF.glaciated.colour"
        );
        assert_eq!(
            path(json!({"temperature": "lava"})),
            "$.climate_placement.classes.EF.glaciated.temperature"
        );
        assert_eq!(
            path(json!({"humidity": "lava"})),
            "$.climate_placement.classes.EF.glaciated.humidity"
        );
    }

    #[test]
    fn a_table_that_is_not_usable_names_its_path() {
        assert_eq!(
            path_of(edited(|v| v["schema"] = json!(2))),
            "$.climate_placement.schema"
        );
        assert_eq!(
            path_of(edited(|v| v["nope"] = json!(1))),
            "$.climate_placement.nope"
        );
        assert_eq!(
            path_of(edited(|v| {
                v["classes"].as_object_mut().unwrap().remove("Dfc");
            })),
            "$.climate_placement.classes"
        );
        assert_eq!(
            path_of(edited(|v| v["classes"]["Af"]["temperature"] = json!("lava"))),
            "$.climate_placement.classes.Af.temperature"
        );
        assert_eq!(
            path_of(edited(|v| v["classes"]["Af"]["humidity"] = json!("lava"))),
            "$.climate_placement.classes.Af.humidity"
        );
        assert_eq!(
            path_of(edited(|v| v["classes"]["Xx"] = v["classes"]["Af"].clone())),
            "$.climate_placement.classes.Xx"
        );
        assert_eq!(
            path_of(edited(|v| v["classes"]["sea"] = v["classes"]["Af"].clone())),
            "$.climate_placement.classes.sea"
        );
        assert_eq!(
            path_of(edited(
                |v| v["temperature_ramps"]["polar"]["span"] = json!(9.0)
            )),
            "$.climate_placement.temperature_ramps.polar.span"
        );
        assert_eq!(
            path_of(edited(
                |v| v["temperature_ramps"]["polar"]["base"] = json!(4.9)
            )),
            "$.climate_placement.temperature_ramps.polar"
        );
        assert_eq!(
            path_of(edited(
                |v| v["temperature_ramps"]["boreal"]["inputs"][0]["weight"] = json!(0.5)
            )),
            "$.climate_placement.temperature_ramps.boreal.inputs"
        );
        assert_eq!(
            path_of(edited(
                |v| v["temperature_ramps"]["polar"]["inputs"][0]["of"] = json!("t_max")
            )),
            "$.climate_placement.temperature_ramps.polar.inputs.0.of"
        );
        assert_eq!(
            path_of(edited(
                |v| v["temperature_ramps"]["polar"]["inputs"][0]["to"] = json!(-20.0)
            )),
            "$.climate_placement.temperature_ramps.polar.inputs.0"
        );
        assert_eq!(
            path_of(edited(|v| v["humidity_rules"]["taiga"] = json!({}))),
            "$.climate_placement.humidity_rules.taiga"
        );
        assert_eq!(
            path_of(edited(
                |v| v["humidity_rules"]["taiga"] = json!({"min": 4.0, "max": 3.0})
            )),
            "$.climate_placement.humidity_rules.taiga"
        );
        assert_eq!(
            path_of(edited(|v| v["glacier_cap"] = json!("cold"))),
            "$.climate_placement.glacier_cap"
        );
        assert!(Placement::parse("{").is_err());
    }

    /// RFC 7386: objects merge key by key, null removes, anything else replaces.
    #[test]
    fn a_merge_patch_merges_objects_removes_nulls_and_replaces_the_rest() {
        let mut t = json!({"a": 1, "b": {"c": 2, "d": [1, 2]}, "e": "x"});
        merge_patch(
            &mut t,
            &json!({"a": null, "b": {"c": 3, "d": [9], "n": {"m": 1}}, "f": true}),
        );
        assert_eq!(
            t,
            json!({"b": {"c": 3, "d": [9], "n": {"m": 1}}, "e": "x", "f": true})
        );
        let mut s = json!("scalar");
        merge_patch(&mut s, &json!({"k": 1}));
        assert_eq!(s, json!({"k": 1}));
    }

    fn pack(id: &str, patch: Value) -> Value {
        json!({"schema": 1, "id": id, "requires_biomes": ["some:biome"], "priority": 0, "patch": patch})
    }

    /// Decisions log 2026-10-04 ("Deep Time-side fixes"): a pack placement table is a merge patch on
    /// the base table; several apply in order, the result is checked whole, and the bake records
    /// their ids. A bad one names its index, or the path in the merged table and the packs.
    #[test]
    fn pack_tables_merge_over_the_base_in_order_and_record_their_ids() {
        let am = koppen::index("Am").unwrap();
        let a = pack(
            "a",
            json!({"humidity_rules": {"wetter": {"min": 4.05}}, "classes": {"Am": {"humidity": "wetter"}}}),
        );
        let b = pack("b", json!({"humidity_rules": {"wetter": {"min": 4.5}}}));
        let p = Placement::with_packs(None, &[a.clone(), b.clone()]).unwrap();
        assert_eq!(p.patches(), ["a", "b"]);
        assert!(builtin().humidity_land(am, 1.0, 26.0, 28.0, 24.0, false) < 4.0);
        assert_eq!(p.humidity_land(am, 1.0, 26.0, 28.0, 24.0, false), 4.5);
        // Everything the packs do not name is the base's.
        let af = koppen::index("Af").unwrap();
        assert_eq!(
            p.thermal_land(af, 26.0, 28.0, 24.0, false),
            builtin().thermal_land(af, 26.0, 28.0, 24.0, false)
        );
        assert_eq!(p.table()["classes"]["Am"]["temperature"], json!("tropical"));
        // No packs: the base itself.
        let none = Placement::with_packs(None, &[]).unwrap();
        assert_eq!(&none, builtin());
        // Over a pack's own base table.
        let mut base: Value = serde_json::from_str(PLACEMENT_JSON).unwrap();
        base["glacier_cap"] = json!(0.2);
        let q = Placement::with_packs(Some(&base), std::slice::from_ref(&a)).unwrap();
        assert_eq!(q.table()["glacier_cap"], json!(0.2));
        assert_eq!(q.humidity_land(am, 1.0, 26.0, 28.0, 24.0, false), 4.05);
        // Refusals.
        let refused = |packs: &[Value]| path_of(Placement::with_packs(None, packs));
        assert_eq!(
            refused(&[json!({"schema": 2, "id": "x", "patch": {}})]),
            "$.climate_placement_patches.0.schema"
        );
        assert_eq!(
            refused(&[a.clone(), json!({"schema": 1, "patch": {}})]),
            "$.climate_placement_patches.1.id"
        );
        assert_eq!(
            refused(&[json!({"schema": 1, "id": "x", "patch": {}, "colour": 1})]),
            "$.climate_placement_patches.0.colour"
        );
        assert_eq!(
            refused(&[json!({"schema": 1, "id": "x", "patch": []})]),
            "$.climate_placement_patches.0.patch"
        );
        let broken = pack(
            "broken",
            json!({"classes": {"Am": {"humidity": "nowhere"}}}),
        );
        match Placement::with_packs(None, &[broken]).expect_err("refused") {
            CoreError::PresetInvalid { path, reason } => {
                assert_eq!(path, "$.climate_placement.classes.Am.humidity");
                assert!(reason.contains("broken"), "{reason}");
            }
            e => panic!("{e}"),
        }
    }

    /// The class rules (decisions log 2026-10-04: Terralith's cloud forest): the first rule of a
    /// class whose conditions all hold replaces what it names; outside it, on glaciated land, or
    /// where an input it reads is unknown, the class's own placement stands.
    #[test]
    fn class_rules_hold_only_where_their_conditions_do() {
        let p = edited(|v| {
            v["temperature_ramps"]["montane"] =
                json!({"base": 1.9, "span": 0.0, "inputs": [{"of": "t_mean", "from": 0.0, "to": 1.0}]});
            v["humidity_rules"]["montane"] = json!({"min": 3.5});
            v["class_rules"] = json!([
                {"name": "cloud_forest", "classes": ["Cfb", "Cwb"],
                 "when": [{"of": "abs_lat_deg", "below": 25.0}, {"of": "height_m", "above": 1000.0},
                          {"of": "aridity", "above": 1.0}],
                 "temperature": "montane", "humidity": "montane",
                 "erosion": {"min": 0.6}, "continentalness": {"min": 0.35}},
                {"name": "second", "classes": ["Cfb"], "when": [{"of": "height_m", "above": 500.0}],
                 "erosion": {"max": -0.5}}
            ]);
        })
        .unwrap();
        assert_eq!(p.rule_names(), ["cloud_forest", "second"]);
        let cfb = koppen::index("Cfb").unwrap();
        let site = |lat: f64, h: f64, ai: f64, glacier: bool| Site {
            abs_lat_deg: lat,
            height_m: h,
            ..Site::of(15.0, 18.0, 12.0, ai, glacier)
        };
        let own = p.place(cfb, &site(40.0, 200.0, 2.0, false));
        assert_eq!(own.rule, None);
        assert_eq!(own.thermal, p.thermal_land(cfb, 15.0, 18.0, 12.0, false));
        assert_eq!(
            own.humidity,
            p.humidity_land(cfb, 2.0, 15.0, 18.0, 12.0, false)
        );
        assert_eq!((own.erosion, own.continentalness), (None, None));
        let cloud = p.place(cfb, &site(10.0, 2000.0, 2.0, false));
        assert_eq!(cloud.rule, Some(0));
        assert_eq!(cloud.thermal, 1.9);
        assert!(cloud.humidity >= 3.5);
        assert_eq!(cloud.erosion, Some(Limit { min: 0.6, max: 1.0 }));
        assert_eq!(
            cloud.continentalness,
            Some(Limit {
                min: 0.35,
                max: 1.0
            })
        );
        // Each condition matters; the next rule may then hold.
        assert_eq!(p.place(cfb, &site(30.0, 2000.0, 2.0, false)).rule, Some(1));
        assert_eq!(p.place(cfb, &site(10.0, 2000.0, 0.5, false)).rule, Some(1));
        let low = p.place(cfb, &site(10.0, 800.0, 2.0, false));
        assert_eq!(low.rule, Some(1));
        assert_eq!(
            low.erosion,
            Some(Limit {
                min: -1.0,
                max: -0.5
            })
        );
        assert_eq!(low.thermal, own.thermal);
        // Glaciated land, an unknown height, another class: no rule.
        assert_eq!(p.place(cfb, &site(10.0, 2000.0, 2.0, true)).rule, None);
        assert_eq!(p.place(cfb, &site(10.0, f64::NAN, 2.0, false)).rule, None);
        let cfa = koppen::index("Cfa").unwrap();
        assert_eq!(p.place(cfa, &site(10.0, 2000.0, 2.0, false)).rule, None);
        // Switches read the new inputs too.
        let q = edited(|v| {
            v["classes"]["Cfb"]["humidity"] = json!({"when": {"of": "abs_lat_deg", "above": 50.0},
                                                     "then": "tundra", "else": "humid"});
        })
        .unwrap();
        assert!(q.place(cfb, &site(60.0, 0.0, 3.0, false)).humidity <= 1.95);
        assert!(q.place(cfb, &site(40.0, 0.0, 3.0, false)).humidity >= 2.05);
        // Refusals name their path.
        let rule = |r: Value| path_of(edited(|v| v["class_rules"] = json!([r])));
        assert_eq!(
            rule(
                json!({"name": "x", "classes": ["Xx"], "when": [{"of": "t_mean", "above": 1.0}], "erosion": {"min": 0.0}})
            ),
            "$.climate_placement.class_rules.0.classes.0"
        );
        assert_eq!(
            rule(json!({"name": "x", "classes": ["Cfb"], "when": [], "erosion": {"min": 0.0}})),
            "$.climate_placement.class_rules.0.when"
        );
        assert_eq!(
            rule(
                json!({"name": "x", "classes": ["Cfb"], "when": [{"of": "height", "above": 1.0}], "erosion": {"min": 0.0}})
            ),
            "$.climate_placement.class_rules.0.when.0.of"
        );
        assert_eq!(
            rule(
                json!({"name": "x", "classes": ["Cfb"], "when": [{"of": "abs_lat_deg", "below": 95.0}], "erosion": {"min": 0.0}})
            ),
            "$.climate_placement.class_rules.0.when.0.below"
        );
        assert_eq!(
            rule(
                json!({"name": "x", "classes": ["Cfb"], "when": [{"of": "t_mean", "above": 1.0}]})
            ),
            "$.climate_placement.class_rules.0"
        );
        assert_eq!(
            rule(
                json!({"name": "x", "classes": ["Cfb"], "when": [{"of": "t_mean", "above": 1.0}], "continentalness": {"min": 1.5}})
            ),
            "$.climate_placement.class_rules.0.continentalness.min"
        );
        // A ramp still reads temperatures only.
        assert_eq!(
            path_of(edited(
                |v| v["temperature_ramps"]["polar"]["inputs"][0]["of"] = json!("height_m")
            )),
            "$.climate_placement.temperature_ramps.polar.inputs.0.of"
        );
    }

    /// Every pack placement table the game ships applies cleanly over the default table, and changes
    /// nothing a vanilla world reads (the default table stays the default).
    #[test]
    fn every_shipped_pack_table_applies_over_the_default_table() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/climate_mapping/packs");
        let mut n = 0;
        for e in std::fs::read_dir(&dir).unwrap() {
            let path = e.unwrap().path();
            if path.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            let id = v["id"].as_str().unwrap().to_owned();
            assert_eq!(
                path.file_stem().and_then(|s| s.to_str()),
                Some(id.as_str()),
                "a pack table's file is named by its id"
            );
            let reqs = v["requires_biomes"].as_array().expect("requires_biomes");
            assert!(!reqs.is_empty(), "{id}: requires_biomes is empty");
            let p = Placement::with_packs(None, std::slice::from_ref(&v))
                .unwrap_or_else(|e| panic!("{id}: {e}"));
            assert_eq!(p.patches(), [id.as_str()]);
            assert_ne!(&p, builtin());
            n += 1;
        }
        assert!(n >= 1, "no pack tables in {}", dir.display());
        assert!(builtin().rule_names().is_empty());
    }
}

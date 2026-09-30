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

/// A band coordinate is in [0, 5].
const TOP: f64 = 5.0;

/// The temperatures a ramp reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Input {
    Mean,
    Warm,
    Cold,
}

impl Input {
    fn parse(s: &str) -> Option<Input> {
        match s {
            "t_mean" => Some(Input::Mean),
            "t_warm" => Some(Input::Warm),
            "t_cold" => Some(Input::Cold),
            _ => None,
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

#[derive(Clone, Debug, PartialEq)]
enum Thermal {
    Ramp(usize),
    When {
        of: Input,
        above: f64,
        then: usize,
        otherwise: usize,
    },
}

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
    humidity: Option<usize>,
    erosion: Option<ErosionLimit>,
}

/// Erosion limits (vanilla's erosion units) a class's land is held to: clamped into
/// `[min, max]`. A missing bound is the end of the parameter's range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ErosionLimit {
    pub min: f64,
    pub max: f64,
}

/// Which branch of the variant noise a class's land takes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Weirdness {
    /// The noise signs it (vanilla's variants on one branch, the other on the other).
    Noise,
    /// Always the positive branch.
    Positive,
    /// The positive branch where the cell's erosion is below the value (high relief: frozen
    /// peaks), the negative branch elsewhere (flats).
    PositiveBelowErosion(f64),
}

/// The ice cover the bake writes (`climate/ice_cover`) and the game's ice pass lays
/// (`placement.json`, `ice_cover`; FORMAT.md, "The ice cover").
#[derive(Clone, Debug, PartialEq)]
pub struct IceCoverSpec {
    /// Height above the equilibrium line, m, per block of cover after the first.
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
    /// Blocks of cover for ground `above_m` metres above its equilibrium line: 0 at or below it,
    /// 1 at its foot, one more per `step_m`, up to `max_blocks`.
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
    humidity: Vec<usize>,
    /// By Köppen index 0 … 30: `Some` for a class with a `glaciated` block.
    glaciated: Vec<Option<Glaciated>>,
    /// By Köppen index 0 … 30: the class's erosion limits.
    erosion: Vec<Option<ErosionLimit>>,
    /// By Köppen index 0 … 30: the branch of the variant noise the class's land takes.
    weirdness: Vec<Weirdness>,
    ice_cover: IceCoverSpec,
    /// The table as written (without `_comment`): what `notes.bake.climate.placement` records.
    table: Value,
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
    let o = object(v, path)?;
    no_other_keys(o, &["min", "max"], path)?;
    if o.is_empty() {
        return Err(err(path, "needs at least one of min, max"));
    }
    let bound = |k: &str, default: f64| -> Result<f64, CoreError> {
        match o.get(k) {
            None => Ok(default),
            Some(x) => number(x, &format!("{path}.{k}"), -1.0, 1.0),
        }
    };
    let (min, max) = (bound("min", -1.0)?, bound("max", 1.0)?);
    if min > max {
        return Err(err(path, "min is above max"));
    }
    Ok(ErosionLimit { min, max })
}

/// `"positive"`, or `{"positive_below_erosion": e}`.
fn parse_weirdness(v: &Value, path: &str) -> Result<Weirdness, CoreError> {
    match v {
        Value::String(s) if s == "positive" => Ok(Weirdness::Positive),
        Value::Object(o) => {
            no_other_keys(o, &["positive_below_erosion"], path)?;
            let k = format!("{path}.positive_below_erosion");
            let e = o
                .get("positive_below_erosion")
                .ok_or_else(|| err(&k, "missing"))?;
            Ok(Weirdness::PositiveBelowErosion(number(e, &k, -1.0, 1.0)?))
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

/// A class's temperature: a ramp name, or a `{when, then, else}` switch.
fn parse_thermal(
    t: &Value,
    tp: &str,
    ramp_ix: &BTreeMap<&str, usize>,
) -> Result<Thermal, CoreError> {
    match t {
        Value::String(_) => Ok(Thermal::Ramp(ramp_named(ramp_ix, t, tp)?)),
        Value::Object(w) => {
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
            let of = Input::parse(string(of, &format!("{tp}.when.of"))?)
                .ok_or_else(|| err(&format!("{tp}.when.of"), "must be t_mean, t_warm or t_cold"))?;
            let above = when
                .get("above")
                .ok_or_else(|| err(&format!("{tp}.when.above"), "missing"))?;
            Ok(Thermal::When {
                of,
                above: number(above, &format!("{tp}.when.above"), -100.0, 100.0)?,
                then: ramp_named(ramp_ix, field("then")?, &format!("{tp}.then"))?,
                otherwise: ramp_named(ramp_ix, field("else")?, &format!("{tp}.else"))?,
            })
        }
        _ => Err(err(
            tp,
            "expected a ramp name or a {when, then, else} switch",
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
        let mut humidity: Vec<Option<usize>> = vec![None; land];
        let mut glaciated: Vec<Option<Glaciated>> = vec![None; land];
        let mut erosion: Vec<Option<ErosionLimit>> = vec![None; land];
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
            let tp = format!("{p}.temperature");
            let t = o.get("temperature").ok_or_else(|| err(&tp, "missing"))?;
            thermal[k] = Some(parse_thermal(t, &tp, &ramp_ix)?);
            let hp = format!("{p}.humidity");
            let n = o.get("humidity").ok_or_else(|| err(&hp, "missing"))?;
            humidity[k] = Some(rule_named(&rule_ix, n, &hp)?);
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
                    Some(t) => Some(parse_thermal(t, &format!("{gp}.temperature"), &ramp_ix)?),
                };
                let humidity = match g.get("humidity") {
                    None => None,
                    Some(n) => Some(rule_named(&rule_ix, n, &format!("{gp}.humidity"))?),
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
            weirdness,
            ice_cover,
            table: Value::Object(table),
        })
    }

    /// The table as written, without its comment.
    pub fn table(&self) -> &Value {
        &self.table
    }

    fn input(of: Input, t_mean: f64, t_warm: f64, t_cold: f64) -> f64 {
        match of {
            Input::Mean => t_mean,
            Input::Warm => t_warm,
            Input::Cold => t_cold,
        }
    }

    fn ramp_value(&self, r: usize, t_mean: f64, t_warm: f64, t_cold: f64) -> f64 {
        let r = &self.ramps[r];
        let s = r.terms.iter().fold(0.0, |s, t| {
            let x = Self::input(t.of, t_mean, t_warm, t_cold);
            s + t.weight * unit((x - t.from) / (t.to - t.from))
        });
        r.base + r.span * s
    }

    fn thermal_value(&self, t: &Thermal, t_mean: f64, t_warm: f64, t_cold: f64) -> f64 {
        match t {
            Thermal::Ramp(r) => self.ramp_value(*r, t_mean, t_warm, t_cold),
            Thermal::When {
                of,
                above,
                then,
                otherwise,
            } => {
                let r = if Self::input(*of, t_mean, t_warm, t_cold) > *above {
                    *then
                } else {
                    *otherwise
                };
                self.ramp_value(r, t_mean, t_warm, t_cold)
            }
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
    pub fn thermal_land(&self, k: u8, t_mean: f64, t_warm: f64, t_cold: f64, glacier: bool) -> f64 {
        let own = || {
            self.thermal.get(usize::from(k)).map_or_else(
                || thermal_sea(t_mean),
                |t| self.thermal_value(t, t_mean, t_warm, t_cold),
            )
        };
        if !glacier {
            return own();
        }
        match self.glaciated_block(k) {
            Some(g) => g
                .thermal
                .as_ref()
                .map_or_else(own, |t| self.thermal_value(t, t_mean, t_warm, t_cold)),
            None => own().min(self.glacier_cap),
        }
    }

    /// Humidity band coordinate of a land cell of Köppen class `k` and aridity index `aridity`;
    /// where the glacier flag is set, the class's `glaciated` humidity rule when it has one.
    pub fn humidity_land(&self, k: u8, aridity: f64, glacier: bool) -> f64 {
        let mut h = humidity_of_aridity(aridity);
        let rule = glacier
            .then(|| self.glaciated_block(k).and_then(|g| g.humidity))
            .flatten()
            .or_else(|| self.humidity.get(usize::from(k)).copied());
        let Some(r) = rule else {
            return h;
        };
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

    #[test]
    fn the_default_table_reproduces_the_compiled_in_constants_bit_for_bit() {
        let p = builtin();
        // A grid over every class but EF, the temperatures the classes live in and the aridity's range.
        let mut n = 0u64;
        for k in (0..32u8).filter(|&k| k != EF) {
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
                    p.humidity_land(k, ai, false).to_bits(),
                    old::humidity_land(k, ai).to_bits(),
                    "class {k} aridity {ai}"
                );
                // No class but EF has a glaciated humidity rule.
                assert_eq!(
                    p.humidity_land(k, ai, true).to_bits(),
                    old::humidity_land(k, ai).to_bits(),
                    "class {k} aridity {ai} glaciated"
                );
            }
        }
        assert!(n > 10_000);
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
        assert!(d.humidity_land(af, 2.5, false) >= 4.1);
        assert!(c.humidity_land(af, 2.5, false) <= 1.95);
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
                assert_eq!(p.humidity_land(EF, ai, true), 0.5, "ice cap");
                assert_eq!(p.humidity_land(EF, ai, false), 1.5, "barren");
            }
        }
        // Humidity band 0 (below vegetation -0.35) is where vanilla puts ice spikes and Terralith
        // no forest; band 1 (-0.35 … -0.1) stays under the band whose variants are taiga and grove.
        assert!(p.humidity_land(EF, 10.0, true) < 1.0);
        assert!((1.0..2.0).contains(&p.humidity_land(EF, 10.0, false)));
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
        assert_eq!(p.humidity_land(et, 5.0, false), 1.95);
        assert_eq!(p.humidity_land(et, 5.0, true), 1.95);
    }

    #[test]
    fn ef_takes_its_peaks_positive_and_its_flats_negative_and_bad_values_name_their_path() {
        let p = builtin();
        for k in 0..32u8 {
            let want = if k == EF {
                Weirdness::PositiveBelowErosion(-0.27)
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
        assert_eq!(p.humidity_land(dfc, 4.0, true), 0.5);
        assert_eq!(
            p.humidity_land(dfc, 4.0, false),
            d.humidity_land(dfc, 4.0, false)
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
}

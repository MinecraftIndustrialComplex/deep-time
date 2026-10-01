// SPDX-License-Identifier: LGPL-3.0-or-later
//! The workbench's editor (P1-20): the Create World setup page's rules (P1-11,
//! `screen/model/EditorModel`) over the definitions the game itself uses, so the browser and the
//! game build the same preset — and the same share code — from the same choices:
//!
//! - **Presets**: the core's built-ins (`preset::BUILTIN`, the game's list: since P1-08 the planet
//!   pipeline under its game names) plus the planet pipelines (`params::PLATES_BUILTIN`,
//!   `PIPELINE_BUILTIN`), grouped into families by their `_<n>k` size suffix as
//!   `EditorModel.familyOf` does.
//! - **Fields**: `editor_fields.json`, the catalogue the game's `EditorFields` reads from its jar.
//!   A field is editable exactly when the selected preset carries its path.
//! - **Timeline**: `StartConfig::ALL` and `LENGTH_MYR_RANGE`, applied with `preset::with_timeline`.
//! - **Numbers** as the game writes them (`EditorFields.Field.number`: integers for integer
//!   fields, doubles otherwise), canonical JSON and share codes by `share_code` (the Java twin
//!   `preset/ShareCode` is tested against it), validation and the estimate by the core.
//! - **Labels**: the game's lang file (`deeptime.editor.*`).
//!
//! - **Memory**: a run whose estimated peak is over its memory cap is refused, by the same rule
//!   in the form's answer ([`Catalog::effective`], the reason Run is off) and in the request that
//!   starts a run ([`Catalog::within_cap`]), so a stale page cannot start it either. The cap is the
//!   editor's field when set, else the workbench's default (`deeptime watch --mem-cap`, else the
//!   core's 4 GiB).
//!
//! The server computes the effective preset (not the page), so these rules live in one language
//! beside the game's.

use deeptime_core::preset::{self, LENGTH_MYR_RANGE, Preset, StartConfig};
use deeptime_core::session::DEFAULT_MEM_CAP;
use deeptime_core::{params, session, share_code};
use serde_json::{Map, Value, json};

/// The smallest memory cap the editor takes, in GiB (the game's `EditorModel.MIN_CAP_GIB`).
pub const MIN_CAP_GIB: f64 = 0.25;

const GIB: f64 = (1u64 << 30) as f64;

/// What the process holds beyond the stages' estimate (`session::estimate`): the server, the
/// preset, the frames of the run being watched. The memory cap counts the whole process (the
/// allocator's gauge), so a run needs its estimate plus this. Measured on the Mac (2026-09-29,
/// release build, private workbench): a finished run's peak minus its estimate was +0.40 GB for
/// `pipeline_quick_16k` at 100, 250 and 500 Myr and +0.49 GB for `pipeline_full_32k` at 30 Myr,
/// so the estimate alone runs low by about that much. A preset that estimates nothing (the
/// placeholders) needs nothing.
pub const SERVER_OVERHEAD_BYTES: u64 = 512 << 20;

/// One editable parameter (an entry of `editor_fields.json`).
#[derive(Debug, Clone)]
pub struct Field {
    pub id: String,
    pub kind: String,
    pub path: Vec<String>,
    pub min: f64,
    pub max: f64,
    pub integer: bool,
    pub phase3: bool,
    /// The values of a `choice` with a fixed list (empty: any string, the essentials lists).
    pub options: Vec<String>,
}

/// A preset the form offers.
#[derive(Debug, Clone)]
pub struct PresetEntry {
    pub name: String,
    pub family: String,
    pub circumference: u64,
    pub placeholder: bool,
    pub description: String,
    pub stages: Vec<String>,
    /// The preset without a timeline.
    pub base: Value,
    /// An internal or test preset (`pipeline_*`, `plates_*`): resolved by name, never offered by
    /// the page (Ben, 2026-09-29: the workbench lists only the game's Earth-like family).
    pub hidden: bool,
}

pub struct Catalog {
    fields: Vec<Field>,
    fields_json: Value,
    presets: Vec<PresetEntry>,
    labels: Value,
    /// The memory cap of a run whose editor field is 0 or unset, in bytes.
    default_cap: u64,
}

/// `pipeline_quick_16k` → `pipeline_quick` (`EditorModel.familyOf`).
pub fn family_of(name: &str) -> String {
    if let Some((head, tail)) = name.rsplit_once('_')
        && let Some(digits) = tail.strip_suffix('k')
        && !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
    {
        return head.to_owned();
    }
    name.to_owned()
}

/// One step down a path: an object's key, or an array's index (`structure.pluton_length_km.0`).
fn step<'a>(e: &'a Value, p: &str) -> Option<&'a Value> {
    match e {
        Value::Object(o) => o.get(p),
        Value::Array(a) => p.parse::<usize>().ok().and_then(|i| a.get(i)),
        _ => None,
    }
}

fn get<'a>(v: &'a Value, path: &[String]) -> Option<&'a Value> {
    let mut e = v;
    for p in path {
        e = step(e, p)?;
    }
    (!e.is_null()).then_some(e)
}

/// Sets `path` (the twin of `EditorFields.set`): objects on the way are created, an array's
/// element must exist (a field is edited only where the preset carries its path, so it does).
fn set(v: &mut Value, path: &[String], value: Value) {
    let mut e = v;
    for p in &path[..path.len() - 1] {
        let next = match e {
            Value::Object(o) => Some(o.entry(p.clone()).or_insert_with(|| json!({}))),
            Value::Array(a) => p.parse::<usize>().ok().and_then(|i| a.get_mut(i)),
            _ => None,
        };
        let Some(next) = next else { return };
        if !next.is_object() && !next.is_array() {
            *next = json!({});
        }
        e = next;
    }
    let last = &path[path.len() - 1];
    match e {
        Value::Object(o) => {
            o.insert(last.clone(), value);
        }
        Value::Array(a) => {
            if let Some(slot) = last.parse::<usize>().ok().and_then(|i| a.get_mut(i)) {
                *slot = value;
            }
        }
        _ => {}
    }
}

fn without_timeline(mut v: Value) -> Value {
    if let Some(o) = v.as_object_mut() {
        o.remove("timeline");
    }
    v
}

impl Catalog {
    /// The shared catalogue, the game's labels and every named preset.
    pub fn load() -> Result<Catalog, String> {
        let fields_json: Value = serde_json::from_str(deeptime_viewer::EDITOR_FIELDS_JSON)
            .map_err(|e| format!("editor_fields.json: {e}"))?;
        let mut fields = Vec::new();
        for f in fields_json["fields"]
            .as_array()
            .ok_or("editor_fields.json: no fields")?
        {
            let s = |k: &str| f[k].as_str().unwrap_or("").to_owned();
            fields.push(Field {
                id: s("id"),
                kind: s("kind"),
                path: s("path").split('.').map(str::to_owned).collect(),
                min: f["min"].as_f64().unwrap_or(0.0),
                max: f["max"].as_f64().unwrap_or(0.0),
                integer: f["integer"].as_bool().unwrap_or(false),
                phase3: f["phase3"].as_bool().unwrap_or(false),
                options: f["options"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|o| o.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default(),
            });
        }
        let lang: Value = serde_json::from_str(deeptime_viewer::LANG_EN_US_JSON)
            .map_err(|e| format!("en_us.json: {e}"))?;
        let labels: Map<String, Value> = lang
            .as_object()
            .map(|o| {
                o.iter()
                    .filter(|(k, _)| k.starts_with("deeptime.editor."))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let mut presets = Vec::new();
        let named = preset::BUILTIN
            .iter()
            .chain(params::PIPELINE_BUILTIN.iter())
            .chain(params::PLATES_BUILTIN.iter());
        for (name, text) in named {
            let base =
                without_timeline(serde_json::from_str(text).map_err(|e| format!("{name}: {e}"))?);
            let p = Preset::from_json(text).map_err(|e| format!("{name}: {e}"))?;
            presets.push(PresetEntry {
                name: (*name).to_owned(),
                family: family_of(name),
                circumference: u64::from(p.circumference()),
                placeholder: p.header.placeholder,
                description: p.header.description.clone().unwrap_or_default(),
                stages: p.stages().iter().map(|s| s.name().to_owned()).collect(),
                base,
                hidden: !preset::BUILTIN.iter().any(|(n, _)| n == name),
            });
        }
        Ok(Catalog {
            fields,
            fields_json,
            presets,
            labels: Value::Object(labels),
            default_cap: DEFAULT_MEM_CAP,
        })
    }

    /// The workbench's default memory cap (`--mem-cap`); 0 keeps the core's 4 GiB.
    pub fn with_default_cap(mut self, bytes: u64) -> Catalog {
        if bytes != 0 {
            self.default_cap = bytes;
        }
        self
    }

    /// The memory cap of a run whose editor field is 0 or unset, in bytes.
    pub fn default_cap(&self) -> u64 {
        self.default_cap
    }

    /// A request's memory cap in bytes: `cap_gib` when given (a number from [`MIN_CAP_GIB`] up),
    /// else the default (absent, null or 0), or why the field is refused.
    pub fn cap_bytes(&self, cap_gib: &Value) -> Result<u64, String> {
        match cap_gib {
            Value::Null => Ok(self.default_cap),
            v => match v.as_f64() {
                Some(0.0) => Ok(self.default_cap),
                Some(g) if g.is_finite() && g >= MIN_CAP_GIB => Ok((g * GIB) as u64),
                _ => Err(format!(
                    "the memory cap must be a number of at least {MIN_CAP_GIB} GiB"
                )),
            },
        }
    }

    #[cfg(test)]
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn preset(&self, name: &str) -> Option<&PresetEntry> {
        self.presets.iter().find(|p| p.name == name)
    }

    /// `GET /api/catalog`: what the form is built from.
    pub fn to_json(&self) -> Value {
        json!({
            "presets": self.presets.iter().filter(|p| !p.hidden).map(|p| json!({
                "name": p.name, "family": p.family, "circumference": p.circumference,
                "placeholder": p.placeholder, "description": p.description, "stages": p.stages,
            })).collect::<Vec<_>>(),
            "fields": self.fields_json["fields"],
            "sections": self.fields_json["sections"],
            "advanced_sections": self.fields_json["advanced_sections"],
            "labels": self.labels,
            "start_configs": StartConfig::ALL.iter().map(|c| c.name()).collect::<Vec<_>>(),
            "length_myr": { "min": LENGTH_MYR_RANGE.0, "max": LENGTH_MYR_RANGE.1 },
            "mem_cap_bytes": self.default_cap,
            "min_cap_gib": MIN_CAP_GIB,
            "server_overhead_bytes": SERVER_OVERHEAD_BYTES,
        })
    }

    /// `POST /api/preset`: the effective preset of a form state
    /// `{"preset": <built-in name>} | {"imported": <preset object>}`, `"start"`, `"length_myr"`,
    /// `"fields": {id: value}`, `"cap_gib"` (the memory cap field; 0 or absent = the default) →
    /// the canonical JSON, its share code, each field's availability and value, the problems, the
    /// core's estimate under the cap once the preset is complete (`session::estimate_for` with the
    /// cap less [`SERVER_OVERHEAD_BYTES`]: its `spill_bytes` is the recorded history that goes to
    /// the run folder's spill file, P1-17), what the run needs of the cap (`needed_bytes`: the
    /// estimate's peak plus [`SERVER_OVERHEAD_BYTES`]) and the cap it was held to (`cap_bytes`). A
    /// run that needs more than its cap — only a run whose base does not fit, since the history
    /// spills — is `can_run: false`, with the reason among the problems.
    pub fn effective(&self, req: &Value) -> Value {
        let mut problems: Vec<String> = Vec::new();
        let cap = self.cap_bytes(&req["cap_gib"]).unwrap_or_else(|e| {
            problems.push(e);
            self.default_cap
        });
        let mut field_problems = Map::new();
        let (mut base, name) = match (req["preset"].as_str(), req.get("imported")) {
            (Some(n), _) => match self.preset(n) {
                Some(p) => (p.base.clone(), p.name.clone()),
                None => return json!({ "error": format!("no preset '{n}'") }),
            },
            (None, Some(v)) if v.is_object() => (
                without_timeline(v.clone()),
                v["name"].as_str().unwrap_or("imported").to_owned(),
            ),
            _ => return json!({ "error": "give a preset name or an imported preset" }),
        };
        let original = base.clone();
        let edits = req["fields"].as_object().cloned().unwrap_or_default();
        let mut fields_out = Vec::new();
        for f in &self.fields {
            let available = !f.phase3 && get(&original, &f.path).is_some();
            if available && let Some(v) = edits.get(&f.id) {
                match self.check_value(f, v) {
                    Ok(value) => set(&mut base, &f.path, value),
                    Err(msg) => {
                        field_problems.insert(f.id.clone(), json!(msg));
                    }
                }
            }
            fields_out.push(json!({
                "id": f.id, "available": available,
                "value": get(&base, &f.path).cloned().unwrap_or(Value::Null),
            }));
        }
        if !field_problems.is_empty() {
            problems.push("some parameters are not valid (see the red fields)".into());
        }
        let start = req["start"].as_str().filter(|s| !s.is_empty());
        let length = req["length_myr"].as_f64();
        let start_ok = start.is_some_and(|s| StartConfig::from_name(s).is_some());
        let length_ok = length.is_some_and(|l| l >= LENGTH_MYR_RANGE.0 && l <= LENGTH_MYR_RANGE.1);
        if !start_ok {
            problems.push("choose how the continents start".into());
        }
        if !length_ok {
            problems.push(format!(
                "choose a length from {} to {} million years",
                LENGTH_MYR_RANGE.0, LENGTH_MYR_RANGE.1
            ));
        }
        let text = base.to_string();
        let json = if start_ok && length_ok {
            preset::with_timeline(&text, start, length)
        } else {
            share_code::canonical(&text)
        };
        let json = match json {
            Ok(j) => j,
            Err(e) => return json!({ "error": e.to_string() }),
        };
        let mut estimate = Value::Null;
        let mut needed = None;
        if problems.is_empty() {
            match Preset::from_json(&json).and_then(|p| p.check_runnable()) {
                Ok(()) => match stage_estimate(&json, cap) {
                    Ok(e) => {
                        estimate = serde_json::from_str(&e).unwrap_or(Value::Null);
                        needed = needed_bytes(&estimate);
                        problems.extend(over_cap(&estimate, cap));
                    }
                    Err(e) => problems.push(e.to_string()),
                },
                Err(e) => problems.push(e.to_string()),
            }
        }
        let code = share_code::encode(&json).unwrap_or_default();
        json!({
            "name": name,
            "json": json,
            "share_code": code,
            "circumference": get(&base, &["scale".into(), "circumference_blocks".into()]),
            "fields": fields_out,
            "field_problems": field_problems,
            "problems": problems,
            "estimate": estimate,
            "needed_bytes": needed,
            "cap_bytes": cap,
            "can_run": problems.is_empty(),
        })
    }

    /// A field's edit as the game writes it, or why it is refused.
    fn check_value(&self, f: &Field, v: &Value) -> Result<Value, String> {
        match f.kind.as_str() {
            "number" => {
                let x = v
                    .as_f64()
                    .or_else(|| {
                        v.as_str()
                            .and_then(|s| s.trim().replace(',', ".").parse().ok())
                    })
                    .filter(|x: &f64| x.is_finite())
                    .ok_or("not a number")?;
                if x < f.min || x > f.max {
                    return Err(format!("from {} to {}", f.min, f.max));
                }
                if f.integer {
                    if x != x.round() {
                        return Err("a whole number".into());
                    }
                    Ok(json!(x as u64))
                } else {
                    Ok(json!(x))
                }
            }
            "toggle" => v
                .as_bool()
                .map(Value::Bool)
                .ok_or_else(|| "on or off".into()),
            "choice" => {
                let s = v.as_str().ok_or("a choice")?;
                if f.options.is_empty() || f.options.iter().any(|o| o == s) {
                    Ok(json!(s))
                } else {
                    Err(format!("one of {}", f.options.join(", ")))
                }
            }
            k => Err(format!("unknown field kind {k}")),
        }
    }

    /// `POST /api/decode`: a share code's preset as a form state — the built-in it is (edits
    /// and all equal), else the imported preset — and its timeline (`EditorModel.importShareCode`).
    pub fn decode(&self, code: &str) -> Value {
        let text = match share_code::decode(code) {
            Ok(t) => t,
            Err(e) => return json!({ "error": e.to_string() }),
        };
        let v: Value = match serde_json::from_str(&text) {
            Ok(v @ Value::Object(_)) => v,
            _ => return json!({ "error": "the code does not hold a preset object" }),
        };
        let timeline = v.get("timeline").cloned().unwrap_or(Value::Null);
        let base = without_timeline(v);
        if get(&base, &["scale".into(), "circumference_blocks".into()]).is_none() {
            return json!({ "error": "the preset in this code names no size (scale.circumference_blocks)" });
        }
        let builtin = self
            .presets
            .iter()
            .find(|p| !p.hidden && p.base == base)
            .map(|p| p.name.clone());
        json!({
            "preset": builtin,
            "imported": if builtin.is_none() { base } else { Value::Null },
            "start": timeline["start"],
            "length_myr": timeline["length_myr"],
        })
    }

    /// A run request's preset: a complete, runnable preset, as canonical JSON.
    pub fn runnable(&self, json: &str) -> Result<String, String> {
        let p = Preset::from_json(json).map_err(|e| e.to_string())?;
        p.check_runnable().map_err(|e| e.to_string())?;
        share_code::canonical(json).map_err(|e| e.to_string())
    }

    /// Whether a run of `json` (complete and runnable) fits a memory cap of `cap` bytes: the rule
    /// of [`Catalog::effective`] applied again where the run is created, so a page that predates
    /// it, or never asked, cannot start a run that would only fail on the cap later.
    pub fn within_cap(&self, json: &str, cap: u64) -> Result<(), String> {
        let estimate = stage_estimate(json, cap).map_err(|e| e.to_string())?;
        let estimate: Value = serde_json::from_str(&estimate).map_err(|e| e.to_string())?;
        over_cap(&estimate, cap).map_or(Ok(()), Err)
    }
}

/// The core's estimate for a run under a cap of `cap` bytes: the stages get what the server leaves
/// of it, and the recorded history they cannot hold under that spills to disk (P1-17), so the
/// estimate's peak is over what they get only when the rest of the run (its base) is.
fn stage_estimate(json: &str, cap: u64) -> Result<String, deeptime_core::CoreError> {
    session::estimate_for(json, "", cap.saturating_sub(SERVER_OVERHEAD_BYTES).max(1))
}

/// What a run needs of the memory cap: the estimate's peak plus [`SERVER_OVERHEAD_BYTES`] (0 for
/// a preset that estimates nothing); `None` when there is no estimate.
fn needed_bytes(estimate: &Value) -> Option<u64> {
    let peak = estimate["peak_bytes"].as_u64()?;
    Some(if peak == 0 {
        0
    } else {
        peak.saturating_add(SERVER_OVERHEAD_BYTES)
    })
}

/// The reason a run does not fit `cap` bytes of memory, or `None` when it does: what it needs
/// against the cap the run gets.
fn over_cap(estimate: &Value, cap: u64) -> Option<String> {
    let needed = needed_bytes(estimate)?;
    let what = if estimate["history_bytes"].as_u64().unwrap_or(0) > 0 {
        "the estimate with its recorded history on disk"
    } else {
        "the estimate"
    };
    (needed > cap).then(|| {
        format!(
            "needs about {} ({what}, {}, and {} for the server itself); the memory cap is \
             {}: raise the cap in Memory, or pick a smaller preset",
            gib_up(needed),
            gib_up(needed - SERVER_OVERHEAD_BYTES),
            gib_up(SERVER_OVERHEAD_BYTES),
            gib(cap)
        )
    })
}

/// `bytes` in GiB to one decimal, rounded up (so a run just over the cap never reads as equal).
fn gib_up(bytes: u64) -> String {
    if bytes < 1 << 30 {
        return format!("{} MiB", bytes.div_ceil(1 << 20));
    }
    format!("{:.1} GiB", (bytes as f64 / GIB * 10.0).ceil() / 10.0)
}

/// A cap in GiB, without trailing zeros (`4 GiB`, `0.25 GiB`).
fn gib(bytes: u64) -> String {
    let text = format!("{:.2}", bytes as f64 / GIB);
    format!("{} GiB", text.trim_end_matches('0').trim_end_matches('.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cat() -> Catalog {
        Catalog::load().unwrap()
    }

    #[test]
    fn families_and_labels_come_from_the_game() {
        assert_eq!(family_of("pipeline_quick_16k"), "pipeline_quick");
        assert_eq!(family_of("earthlike_full_32k"), "earthlike_full");
        assert_eq!(family_of("custom"), "custom");
        assert_eq!(family_of("a_k"), "a_k");
        let c = cat();
        let j = c.to_json();
        assert!(j["presets"].as_array().unwrap().iter().any(|p| {
            p["name"] == "earthlike_quick_16k"
                && p["placeholder"] == false
                && p["stages"]
                    .as_array()
                    .is_some_and(|s| s.last() == Some(&json!("amplify")))
        }));
        // The page offers the Earth-like family only; the pipeline and plates presets stay
        // resolvable by name (tests, the CLI) but are not listed.
        let listed: Vec<&str> = j["presets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            listed,
            [
                "earthlike_quick_16k",
                "earthlike_full_16k",
                "earthlike_quick_32k",
                "earthlike_full_32k"
            ]
        );
        assert!(c.preset("pipeline_quick_16k").is_some_and(|p| p.hidden));
        assert!(c.preset("plates_quick_16k").is_some_and(|p| p.hidden));
        assert!(c.preset("earthlike_quick_16k").is_some_and(|p| !p.hidden));
        // A code of a hidden preset loads as an imported one, not as a name the page cannot show.
        let hidden = c.effective(
            &json!({ "preset": "plates_quick_16k", "start": "random", "length_myr": 100 }),
        );
        let d = c.decode(hidden["share_code"].as_str().unwrap());
        assert_eq!(d["preset"], Value::Null);
        assert_eq!(d["imported"]["name"], "plates_quick_16k");
        for f in c.fields() {
            assert!(
                j["labels"][format!("deeptime.editor.field.{}", f.id)].is_string(),
                "the game has a label for {}",
                f.id
            );
        }
        assert_eq!(j["start_configs"][1], "scattered_continents");
    }

    /// Every field a planet preset carries, edited within its range, gives a preset the core
    /// accepts (the catalogue's paths are the core's schema), with the numbers written as the game
    /// writes them.
    #[test]
    fn edits_land_where_the_core_reads_them() {
        let c = cat();
        let mut fields = Map::new();
        for f in c.fields() {
            if f.kind == "number" {
                // A short decimal, as a player types it (a 17-digit midpoint can come back one
                // ulp off from serde_json's parser).
                let mid: f64 = format!("{:.3e}", (f.min + f.max) / 2.0).parse().unwrap();
                let v = if f.id == "seed" {
                    json!(42)
                } else if f.integer {
                    json!(mid.round() as u64)
                } else {
                    json!(mid)
                };
                fields.insert(f.id.clone(), v);
            }
        }
        fields.insert("province_strength".into(), json!("weak"));
        fields.insert("gravity".into(), json!("3.71"));
        fields.insert("ocean_fraction".into(), json!(0.5));
        fields.insert("day_length".into(), json!(9));
        fields.insert("axial_tilt".into(), json!(23.4));
        fields.insert("star_flux".into(), json!(1.0));
        fields.insert("m_per_block_h".into(), json!(300));
        fields.insert("m_per_block_v".into(), json!(20));
        let r = c.effective(&json!({
            "preset": "pipeline_quick_16k", "start": "supercontinent", "length_myr": 300,
            "fields": fields,
        }));
        assert_eq!(r["problems"], json!([]), "{r:#}");
        assert_eq!(r["can_run"], true);
        let text = r["json"].as_str().unwrap();
        assert!(text.contains("\"seed\":42,"), "the seed stays an integer");
        assert!(text.contains("\"gravity_m_s2\":3.71"));
        assert!(
            text.contains("\"day_length_h\":9.0"),
            "numbers as the game writes them"
        );
        assert!(text.contains("\"timeline\":{\"length_myr\":300.0,\"start\":\"supercontinent\"}"));
        assert!(r["estimate"]["peak_bytes"].as_u64().unwrap() > 0);
        // The tuning knobs (settings audit): each lands at the key the core reads, whether an object's
        // (incision_cap), an integer (plate_count), an array's element (pluton_length_km.0, the
        // sea-level cycle) or a choice (province).
        assert!(text.contains("\"incision_cap\":5.125"), "{text}");
        assert!(
            text.contains("\"initial_plates\":22,"),
            "an integer knob stays an integer"
        );
        assert!(
            text.contains("\"pluton_length_km\":[44.0,8.0,80.0]"),
            "{text}"
        );
        assert!(
            text.contains(
                "\"cycles\":[{\"amplitude_m\":100.0,\"period_myr\":250.0,\"phase_deg\":0.0},"
            ),
            "{text}"
        );
        assert!(text.contains("\"province\":\"weak\""));
        assert!(
            text.contains("\"count_mult\":5.0"),
            "the deposit count multiplier"
        );
        // The Multipliers section is wired to the core (Ben, 2026-09-29): tonnage, grade, and the
        // feature scales of ore bodies, rock layers and (P1-22) rivers.
        assert!(text.contains("\"size_mult\":5.05"), "{text}");
        assert!(text.contains("\"richness_mult\":5.05"), "{text}");
        assert!(
            text.contains("\"feature_scale\":{\"deposits\":5.05,\"rivers\":5.05,\"strata\":5.05}"),
            "{text}"
        );
        let unavailable: Vec<&str> = r["fields"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["available"] == false)
            .map(|f| f["id"].as_str().unwrap())
            .collect();
        assert_eq!(
            unavailable,
            [
                // Canyons (Phase 2), Phase 3, and the pack-defined essentials lists.
                "feature_canyons",
                "fantasy_fields",
                "sky_islands",
                "exotic_landforms",
                "essentials"
            ],
            "every other field is editable in a pipeline preset"
        );
        // The share code round-trips to the same form state, as an imported preset (edited).
        let d = c.decode(r["share_code"].as_str().unwrap());
        assert_eq!(d["preset"], Value::Null);
        assert_eq!(d["start"], "supercontinent");
        assert_eq!(d["imported"]["planet"]["gravity_m_s2"], 3.71);
        assert_eq!(d["imported"]["amplify"]["incision_cap"], 5.125);
        assert_eq!(d["imported"]["structure"]["pluton_length_km"][0], 44.0);
        let again = c.effective(&json!({
            "imported": d["imported"], "start": d["start"], "length_myr": d["length_myr"],
        }));
        assert_eq!(again["json"], r["json"]);
        assert_eq!(again["share_code"], r["share_code"]);
        // The imported preset shows every edit as the value of its field.
        for f in again["fields"].as_array().unwrap() {
            let id = f["id"].as_str().unwrap();
            let sent = &r["fields"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == f["id"])
                .unwrap();
            assert_eq!(f["value"], sent["value"], "{id} survives the round trip");
        }
    }

    /// The presets a form starts from.
    const PIPELINES: [&str; 4] = [
        "pipeline_quick_16k",
        "pipeline_full_16k",
        "pipeline_quick_32k",
        "pipeline_full_32k",
    ];

    /// The tuning knobs (settings audit) are in every pipeline preset at a value the editor accepts, and
    /// the core accepts both ends of every range and every option: a knob never lets a player
    /// build a preset the core refuses. (`STATUS.md` records the same ends run on a small planet.)
    #[test]
    fn every_knob_is_in_every_pipeline_preset_and_both_ends_are_valid() {
        let c = cat();
        let timeline = |name: &str, fields: Value| {
            c.effective(&json!({
                "preset": name, "start": "scattered_continents", "length_myr": 100, "cap_gib": 64,
                "fields": fields,
            }))
        };
        for name in PIPELINES {
            let base = &c.preset(name).unwrap().base;
            let mut knobs = 0;
            for f in c.fields() {
                let Some(v) = get(base, &f.path) else {
                    continue;
                };
                knobs += 1;
                match f.kind.as_str() {
                    "number" => {
                        let x = v.as_f64().unwrap_or(f64::NAN);
                        assert!(
                            x >= f.min && x <= f.max,
                            "{name}: {} = {x} is outside its range {}..{}",
                            f.id,
                            f.min,
                            f.max
                        );
                        for end in [f.min, f.max] {
                            let value = if f.integer {
                                json!(end as u64)
                            } else {
                                json!(end)
                            };
                            let r = timeline(name, json!({ f.id.clone(): value }));
                            assert_eq!(r["problems"], json!([]), "{name}: {} = {end}: {r:#}", f.id);
                            assert_eq!(r["field_problems"], json!({}), "{name}: {}", f.id);
                        }
                        // One step outside is refused by the editor itself.
                        for out in [
                            f.min - 1.0e-3 * (f.max - f.min),
                            f.max + 1.0e-3 * (f.max - f.min),
                        ] {
                            let r = timeline(name, json!({ f.id.clone(): out }));
                            assert!(
                                r["field_problems"][&f.id].is_string(),
                                "{name}: {} = {out}",
                                f.id
                            );
                            assert_eq!(r["can_run"], false);
                        }
                    }
                    "choice" if !f.options.is_empty() => {
                        assert!(
                            f.options.iter().any(|o| v == o.as_str()),
                            "{name}: {}",
                            f.id
                        );
                        for o in &f.options {
                            let r = timeline(name, json!({ f.id.clone(): o }));
                            assert_eq!(r["problems"], json!([]), "{name}: {} = {o}: {r:#}", f.id);
                            assert_eq!(
                                get(
                                    &serde_json::from_str::<Value>(r["json"].as_str().unwrap())
                                        .unwrap(),
                                    &f.path
                                ),
                                Some(&json!(o))
                            );
                        }
                        let r = timeline(name, json!({ f.id.clone(): "everything" }));
                        assert!(r["field_problems"][&f.id].is_string());
                    }
                    _ => {}
                }
            }
            assert!(knobs >= 35, "{name} carries only {knobs} catalogue paths");
        }
    }

    /// The cross-language fixture (`deeptime-core/tests/share-codes.json`, also read by the game's
    /// `ShareCodeTest` and `AdvancedSettingsTest`).
    const SHARE_FIXTURE: &str = include_str!("../../../deeptime-core/tests/share-codes.json");

    /// The workbench's side of the game ↔ workbench round trip (settings audit): the tuned pipeline preset
    /// of the fixture, as the game encodes it (`java`) and as Rust does (`rust`), loads into the
    /// form with every tuned value in its field and comes back out as the very same JSON.
    #[test]
    fn a_tuned_code_from_the_game_loads_unchanged() {
        let c = cat();
        let fx: Value = serde_json::from_str(SHARE_FIXTURE).unwrap();
        let case = fx["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "pipeline_quick_16k_tuned")
            .expect("the fixture has the tuned pipeline case");
        let json = case["json"].as_str().unwrap();
        for who in ["rust", "java"] {
            let d = c.decode(case[who].as_str().unwrap());
            assert_eq!(
                d["preset"],
                Value::Null,
                "{who}: a tuned preset is a custom one"
            );
            assert_eq!(d["start"], "scattered_continents");
            assert_eq!(d["length_myr"], 100.0);
            let r = c.effective(&json!({
                "imported": d["imported"], "start": d["start"], "length_myr": d["length_myr"],
            }));
            assert_eq!(r["problems"], json!([]), "{who}: {r:#}");
            assert_eq!(
                r["json"].as_str().unwrap(),
                json,
                "{who}: nothing added, dropped or rounded"
            );
            assert_eq!(
                share_code::decode(r["share_code"].as_str().unwrap()).unwrap(),
                json
            );
            let value = |id: &str| {
                r["fields"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|f| f["id"] == id)
                    .unwrap()["value"]
                    .clone()
            };
            assert_eq!(value("carving"), 10.0);
            assert_eq!(value("plate_count"), 22);
            assert_eq!(value("erosion_strength"), 5e-9);
            assert_eq!(value("ridge_depth"), -3000.0);
            assert_eq!(value("pluton_size"), 40.0);
            assert_eq!(value("sea_level_swing"), 120.0);
            assert_eq!(value("deposit_size"), 2.5);
            assert_eq!(value("deposit_richness"), 1.5);
            assert_eq!(value("feature_deposits"), 2.0);
            assert_eq!(value("feature_strata"), 0.5);
            assert_eq!(value("province_strength"), "medium");
            // The biome placement settings (P1-09's remaining constants).
            assert_eq!(value("beach_width"), 40.0);
            assert_eq!(value("inland_mid"), 300.0);
            assert_eq!(value("inland_far"), 1500.0);
            assert_eq!(value("inland_interior"), 8000.0);
            assert_eq!(value("wetland_humidity"), 3.0);
            assert_eq!(value("wetland_flat_share"), 0.5);
            assert_eq!(value("wetland_lowland"), 250.0);
        }
    }

    /// The catalogue's structure: advanced sections exist, hold fields, and come last.
    #[test]
    fn the_catalogue_groups_the_advanced_settings() {
        let c = cat();
        let j = c.to_json();
        let adv: Vec<&str> = j["advanced_sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap())
            .collect();
        assert_eq!(adv.len(), 8);
        let fields = j["fields"].as_array().unwrap();
        let sections: Vec<&str> = fields
            .iter()
            .map(|f| f["section"].as_str().unwrap())
            .collect();
        for s in &adv {
            assert!(j["sections"][s].is_string(), "{s} has a description");
            assert!(
                j["labels"][format!("deeptime.editor.section.{s}")].is_string(),
                "the game has a title for {s}"
            );
            assert!(sections.contains(s), "{s} holds fields");
        }
        let first_adv = sections.iter().position(|s| adv.contains(s)).unwrap();
        assert!(
            sections[first_adv..].iter().all(|s| adv.contains(s)),
            "the advanced sections come last, so both editors can fold them"
        );
        for f in fields {
            let id = f["id"].as_str().unwrap();
            assert!(
                f["hint"].as_str().is_some_and(|h| h.len() > 20),
                "{id} has a hint"
            );
            assert!(
                j["sections"][f["section"].as_str().unwrap()].is_string(),
                "the section of {id} has a description"
            );
            for s in f["stops"].as_array().into_iter().flatten() {
                let v = s["value"].as_f64().unwrap();
                assert!(
                    v >= f["min"].as_f64().unwrap() && v <= f["max"].as_f64().unwrap(),
                    "{id}: the stop {s} is outside the range"
                );
                assert!(s["label"].as_str().is_some_and(|l| !l.is_empty()));
            }
            if f["kind"] == "choice" && f["options"].is_array() {
                for o in f["options"].as_array().unwrap() {
                    let key = format!("deeptime.editor.option.{id}.{}", o.as_str().unwrap());
                    assert!(j["labels"][key.as_str()].is_string(), "{key}");
                }
            }
        }
        assert_eq!(
            get(
                &json!({"a": [{"b": 1}, 2]}),
                &["a".into(), "0".into(), "b".into()]
            ),
            Some(&json!(1)),
            "a number in a path indexes an array"
        );
        assert_eq!(get(&json!({"a": [1]}), &["a".into(), "3".into()]), None);
        // Terrain carving (Ben, 2026-09-29): the default is the medium stop.
        let carving = fields.iter().find(|f| f["id"] == "carving").unwrap();
        let medium = carving["stops"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["label"] == "Medium")
            .unwrap();
        for name in PIPELINES {
            let base = &c.preset(name).unwrap().base;
            assert_eq!(
                base["amplify"]["incision_cap"].as_f64(),
                medium["value"].as_f64(),
                "{name}"
            );
        }
    }

    #[test]
    fn a_built_in_code_selects_the_built_in_and_bad_input_is_named() {
        let c = cat();
        let r = c.effective(
            &json!({ "preset": "earthlike_quick_16k", "start": "random", "length_myr": 100 }),
        );
        let d = c.decode(r["share_code"].as_str().unwrap());
        assert_eq!(d["preset"], "earthlike_quick_16k");
        assert_eq!(d["length_myr"], 100.0);
        // The built-ins are simulated planets since P1-08: their fields are editable, and they
        // need a timeline like the pipeline presets.
        let p =
            c.effective(&json!({ "preset": "earthlike_quick_16k", "fields": { "gravity": 3 } }));
        assert!(
            p["fields"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f["id"] == "gravity" && f["available"] == true)
        );
        assert_eq!(p["can_run"], false, "no timeline yet");
        assert!(
            p["problems"][0]
                .as_str()
                .unwrap()
                .contains("continents start")
        );
        let bad = c.effective(&json!({
            "preset": "pipeline_quick_16k", "start": "random", "length_myr": 50,
            "fields": { "seed": 4.5, "gravity": 500 },
        }));
        assert_eq!(bad["can_run"], false);
        assert_eq!(bad["field_problems"]["seed"], "a whole number");
        assert!(
            bad["field_problems"]["gravity"]
                .as_str()
                .unwrap()
                .starts_with("from 0.1")
        );
        assert!(c.decode("DT1.nope").get("error").is_some());
        assert!(c.decode("hello").get("error").is_some());
        assert!(c.runnable(r["json"].as_str().unwrap()).is_ok());
        assert!(
            c.runnable(p["json"].as_str().unwrap()).is_err(),
            "a built-in needs a timeline"
        );
        let no_timeline = c.effective(&json!({ "preset": "pipeline_quick_16k" }));
        assert!(c.runnable(no_timeline["json"].as_str().unwrap()).is_err());
    }

    /// P1-17: the recorded history goes to disk over the cap, so a run whose history does not fit
    /// is allowed (its estimate says how much spills) and only its base decides; 2000 Myr of
    /// `pipeline_full_32k` under the workbench's 8 GiB, refused before the spill (≈ 9 GiB).
    #[test]
    fn a_run_whose_history_spills_is_allowed() {
        let c = Catalog::load().unwrap().with_default_cap(8 << 30);
        let r = c.effective(&json!({
            "preset": "pipeline_full_32k", "start": "supercontinent", "length_myr": 2000,
        }));
        assert_eq!(r["can_run"], true, "{r:#}");
        let e = &r["estimate"];
        let (base, history, spill, peak) = (
            e["base_bytes"].as_u64().unwrap(),
            e["history_bytes"].as_u64().unwrap(),
            e["spill_bytes"].as_u64().unwrap(),
            e["peak_bytes"].as_u64().unwrap(),
        );
        assert!(
            base + history + SERVER_OVERHEAD_BYTES > 8 << 30,
            "all in memory it would not fit: {e}"
        );
        assert!(spill > 0 && peak + SERVER_OVERHEAD_BYTES <= 8 << 30, "{e}");
        assert_eq!(peak + spill, base + history);
        assert!(c.within_cap(r["json"].as_str().unwrap(), 8 << 30).is_ok());
    }

    /// The bug: `pipeline_full_32k`, supercontinent, 1000 Myr estimates about 5.5 GB, and the
    /// workbench answered `can_run: true` against a 4 GiB cap; the run failed 15 minutes in
    /// ("needed 4300143147 bytes, cap 4294967296 bytes"). The estimate is held to the run's cap now:
    /// the editor's field when set, else the workbench's default.
    #[test]
    fn a_run_over_its_memory_cap_is_refused_and_a_raised_cap_allows_it() {
        let c = cat();
        assert_eq!(c.default_cap(), DEFAULT_MEM_CAP);
        let req = json!({
            "preset": "pipeline_full_32k", "start": "supercontinent", "length_myr": 1000,
        });
        let r = c.effective(&req);
        let peak = r["estimate"]["peak_bytes"].as_u64().unwrap();
        assert!(
            peak > DEFAULT_MEM_CAP && peak + SERVER_OVERHEAD_BYTES < 8 << 30,
            "the scenario: over 4 GiB, within 8 GiB (peak {peak})"
        );
        assert_eq!(r["can_run"], false, "{r:#}");
        assert_eq!(r["cap_bytes"], DEFAULT_MEM_CAP);
        let reason = r["problems"][0].as_str().unwrap();
        assert!(reason.starts_with("needs about "), "{reason}");
        assert!(reason.contains("for the server itself"), "{reason}");
        assert!(reason.contains("the memory cap is 4 GiB"), "{reason}");
        assert!(reason.contains("raise the cap in Memory"), "{reason}");
        assert_eq!(r["problems"].as_array().unwrap().len(), 1);
        assert!(
            c.within_cap(r["json"].as_str().unwrap(), DEFAULT_MEM_CAP)
                .is_err()
        );

        // The editor's field raises the cap for this run; the base is the same, and more of the
        // recorded history stays in memory (P1-17).
        let mut raised = req.clone();
        raised["cap_gib"] = json!(8);
        let r8 = c.effective(&raised);
        assert_eq!(r8["problems"], json!([]), "{r8:#}");
        assert_eq!(r8["can_run"], true);
        assert_eq!(r8["cap_bytes"], 8u64 << 30);
        assert_eq!(r8["estimate"]["base_bytes"], r["estimate"]["base_bytes"]);
        assert!(r8["estimate"]["peak_bytes"].as_u64().unwrap() > peak);
        assert!(c.within_cap(r8["json"].as_str().unwrap(), 8 << 30).is_ok());

        // What the run needs is the estimate plus the server's own memory (the estimate alone ran
        // low by that much when measured). A cap just under it still refuses; just over allows.
        let needed = r["needed_bytes"].as_u64().unwrap();
        assert_eq!(needed, peak + SERVER_OVERHEAD_BYTES);
        let gib_of = |bytes: u64| bytes as f64 / (1u64 << 30) as f64;
        let mut edge = req.clone();
        edge["cap_gib"] = json!(gib_of(needed) - 0.01);
        assert_eq!(c.effective(&edge)["can_run"], false);
        edge["cap_gib"] = json!(gib_of(needed) + 0.01);
        assert_eq!(c.effective(&edge)["can_run"], true);
        edge["cap_gib"] = json!(gib_of(peak) + 0.01);
        assert_eq!(
            c.effective(&edge)["can_run"],
            false,
            "the estimate alone would have fitted"
        );

        // The workbench's default (`--mem-cap`) is what a run without a field gets; 0 is "unset";
        // a field still overrides it in either direction.
        let c8 = Catalog::load().unwrap().with_default_cap(8 << 30);
        assert_eq!(c8.to_json()["mem_cap_bytes"], 8u64 << 30);
        let d = c8.effective(&req);
        assert_eq!(d["can_run"], true, "{d:#}");
        assert_eq!(d["cap_bytes"], 8u64 << 30);
        let mut zero = req.clone();
        zero["cap_gib"] = json!(0);
        assert_eq!(c8.effective(&zero)["can_run"], true);
        let mut four = req.clone();
        four["cap_gib"] = json!(4);
        let f = c8.effective(&four);
        assert_eq!(f["can_run"], false);
        assert!(
            f["problems"][0]
                .as_str()
                .unwrap()
                .contains("the memory cap is 4 GiB")
        );
        assert_eq!(
            Catalog::load().unwrap().with_default_cap(0).default_cap(),
            DEFAULT_MEM_CAP,
            "0 keeps the core's default"
        );

        // A cap the editor cannot take is a problem of its own (the game's minimum, 0.25 GiB).
        for bad in [json!(0.1), json!(-3), json!("lots")] {
            let mut b = req.clone();
            b["cap_gib"] = bad;
            let r = c.effective(&b);
            assert_eq!(r["can_run"], false);
            assert!(
                r["problems"][0]
                    .as_str()
                    .unwrap()
                    .contains("at least 0.25 GiB"),
                "{r:#}"
            );
        }
        // Small runs are untouched by a small cap.
        let small = c.effective(&json!({
            "preset": "pipeline_quick_16k", "start": "random", "length_myr": 50, "cap_gib": 4,
        }));
        assert_eq!(small["can_run"], true, "{small:#}");
        assert_eq!(gib_up(5_532_931_904), "5.2 GiB");
        assert_eq!(gib_up(3 << 29), "1.5 GiB");
        assert_eq!(gib_up(100 << 20), "100 MiB");
        assert_eq!(gib(4 << 30), "4 GiB");
        assert_eq!(gib(1 << 28), "0.25 GiB");
    }
}

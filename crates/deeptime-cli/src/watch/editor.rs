// SPDX-License-Identifier: LGPL-3.0-or-later
//! The workbench's editor (P1-20): the Create World setup page's rules (P1-11,
//! `screen/model/EditorModel`) over the definitions the game itself uses, so the browser and the
//! game build the same preset — and the same share code — from the same choices:
//!
//! - **Presets**: the core's built-ins (`preset::BUILTIN`, the game's list) plus the planet
//!   pipelines (`params::PLATES_BUILTIN`, `PIPELINE_BUILTIN`: the real simulation until P1-08 makes
//!   them the built-ins), grouped into families by their `_<n>k` size suffix as
//!   `EditorModel.familyOf` does.
//! - **Fields**: `editor_fields.json`, the catalogue the game's `EditorFields` reads from its jar.
//!   A field is editable exactly when the selected preset carries its path.
//! - **Timeline**: `StartConfig::ALL` and `LENGTH_MYR_RANGE`, applied with `preset::with_timeline`.
//! - **Numbers** as the game writes them (`EditorFields.Field.number`: integers for integer
//!   fields, doubles otherwise), canonical JSON and share codes by `share_code` (the Java twin
//!   `preset/ShareCode` is tested against it), validation and the estimate by the core.
//! - **Labels**: the game's lang file (`deeptime.editor.*`).
//!
//! The server computes the effective preset (not the page), so these rules live in one language
//! beside the game's.

use deeptime_core::preset::{self, LENGTH_MYR_RANGE, Preset, StartConfig};
use deeptime_core::{params, session, share_code};
use serde_json::{Map, Value, json};

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
}

pub struct Catalog {
    fields: Vec<Field>,
    fields_json: Value,
    presets: Vec<PresetEntry>,
    labels: Value,
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

fn get<'a>(v: &'a Value, path: &[String]) -> Option<&'a Value> {
    let mut e = v;
    for p in path {
        e = e.as_object()?.get(p)?;
    }
    (!e.is_null()).then_some(e)
}

fn set(v: &mut Value, path: &[String], value: Value) {
    let mut e = v;
    for p in &path[..path.len() - 1] {
        let o = e.as_object_mut().expect("objects on the way");
        let next = o.entry(p.clone()).or_insert_with(|| json!({}));
        if !next.is_object() {
            *next = json!({});
        }
        e = next;
    }
    if let Some(o) = e.as_object_mut() {
        o.insert(path[path.len() - 1].clone(), value);
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
            });
        }
        Ok(Catalog {
            fields,
            fields_json,
            presets,
            labels: Value::Object(labels),
        })
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
            "presets": self.presets.iter().map(|p| json!({
                "name": p.name, "family": p.family, "circumference": p.circumference,
                "placeholder": p.placeholder, "description": p.description, "stages": p.stages,
            })).collect::<Vec<_>>(),
            "fields": self.fields_json["fields"],
            "labels": self.labels,
            "start_configs": StartConfig::ALL.iter().map(|c| c.name()).collect::<Vec<_>>(),
            "length_myr": { "min": LENGTH_MYR_RANGE.0, "max": LENGTH_MYR_RANGE.1 },
        })
    }

    /// `POST /api/preset`: the effective preset of a form state
    /// `{"preset": <built-in name>} | {"imported": <preset object>}`, `"start"`, `"length_myr"`,
    /// `"fields": {id: value}` → the canonical JSON, its share code, each field's availability and
    /// value, the problems, and the core's estimate once the preset is complete.
    pub fn effective(&self, req: &Value) -> Value {
        let mut problems: Vec<String> = Vec::new();
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
        if problems.is_empty() {
            match Preset::from_json(&json).and_then(|p| p.check_runnable()) {
                Ok(()) => match session::estimate(&json, "") {
                    Ok(e) => estimate = serde_json::from_str(&e).unwrap_or(Value::Null),
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
            "choice" => v
                .as_str()
                .map(|s| json!(s))
                .ok_or_else(|| "a choice".into()),
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
            .find(|p| p.base == base)
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
        assert!(
            j["presets"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["name"] == "earthlike_quick_16k" && p["placeholder"] == true)
        );
        assert!(
            j["presets"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["name"] == "pipeline_quick_16k"
                    && p["stages"]
                        .as_array()
                        .is_some_and(|s| s.first() == Some(&json!("plates"))
                            && s.contains(&json!("surface"))))
        );
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
                let v = if f.integer {
                    json!(42)
                } else {
                    json!((f.min + f.max) / 2.0)
                };
                fields.insert(f.id.clone(), v);
            }
        }
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
        let avail: Vec<&str> = r["fields"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["available"] == true)
            .map(|f| f["id"].as_str().unwrap())
            .collect();
        assert_eq!(
            avail,
            [
                "seed",
                "gravity",
                "day_length",
                "axial_tilt",
                "star_flux",
                "ocean_fraction",
                "m_per_block_h",
                "m_per_block_v",
                // The drama slider (P1-06): the pipeline presets carry `surface.drama`.
                "drama"
            ]
        );
        // The share code round-trips to the same form state, as an imported preset (edited).
        let d = c.decode(r["share_code"].as_str().unwrap());
        assert_eq!(d["preset"], Value::Null);
        assert_eq!(d["start"], "supercontinent");
        assert_eq!(d["imported"]["planet"]["gravity_m_s2"], 3.71);
        let again = c.effective(&json!({
            "imported": d["imported"], "start": d["start"], "length_myr": d["length_myr"],
        }));
        assert_eq!(again["json"], r["json"]);
        assert_eq!(again["share_code"], r["share_code"]);
    }

    #[test]
    fn a_built_in_code_selects_the_built_in_and_bad_input_is_named() {
        let c = cat();
        let r = c.effective(
            &json!({ "preset": "plates_quick_16k", "start": "random", "length_myr": 100 }),
        );
        let d = c.decode(r["share_code"].as_str().unwrap());
        assert_eq!(d["preset"], "plates_quick_16k");
        assert_eq!(d["length_myr"], 100.0);
        // Placeholders carry no parameter paths: every field unavailable, edits ignored.
        let p =
            c.effective(&json!({ "preset": "earthlike_quick_16k", "fields": { "gravity": 3 } }));
        assert!(
            p["fields"]
                .as_array()
                .unwrap()
                .iter()
                .all(|f| f["available"] == false)
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
            c.runnable(p["json"].as_str().unwrap()).is_ok(),
            "a placeholder needs no timeline"
        );
        let no_timeline = c.effective(&json!({ "preset": "pipeline_quick_16k" }));
        assert!(c.runnable(no_timeline["json"].as_str().unwrap()).is_err());
    }
}

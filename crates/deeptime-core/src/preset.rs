// SPDX-License-Identifier: LGPL-3.0-or-later
//! Presets (plan §5.1): parsing, the built-in presets, and how a preset argument is resolved.
//!
//! Until the real stages exist (P1-03 on) the only pipeline is the dummy one (`dummy.rs`). A
//! preset is `{"dummy": {…}}` plus optional header keys that every preset will carry:
//!
//! ```json
//! {"schema": 0, "name": "earthlike_quick_16k", "description": "…", "placeholder": true,
//!  "scale": {"circumference_blocks": 16384},
//!  "timeline": {"start": "supercontinent", "length_myr": 1000},
//!  "dummy": {…}}
//! ```
//!
//! - `scale.circumference_blocks` is C (plan §5.1). The game compares it with the world's
//!   `level-type` before running (P1-13); with a dummy `world` it must equal the dummy's
//!   circumference.
//! - `timeline` has no default (decisions file, "Timeline"): the built-in presets carry none, and
//!   the editor or `config/deeptime/server.json` supplies it. `start` is one of
//!   [`StartConfig`]; `length_myr` the simulated span in millions of years. On a dummy world it
//!   sets the clock to run from −length to 0 with eight keyframe intervals.
//! - `placeholder: true` marks the Phase 1 stand-ins shipped as the built-in presets before the real
//!   stages exist; the server logs a warning for them.
//!
//! Built-in presets are JSON files in `crates/deeptime-core/presets/`, compiled into the core and
//! copied into the mod jar (`deeptime/presets/<name>.json`, Gradle `processResources`).

use serde_json::{Map, Value, json};

use crate::dummy::DummyParams;
use crate::error::CoreError;
use crate::keyframes::KeyframePolicy;
use crate::share_code;
use crate::stage::Stage;
use crate::synth;
use crate::world::WorldState;

/// The built-in presets (plan §5.1), in the order the editor lists them.
pub const BUILTIN: [(&str, &str); 4] = [
    (
        "earthlike_quick_16k",
        include_str!("../presets/earthlike_quick_16k.json"),
    ),
    (
        "earthlike_full_16k",
        include_str!("../presets/earthlike_full_16k.json"),
    ),
    (
        "earthlike_quick_32k",
        include_str!("../presets/earthlike_quick_32k.json"),
    ),
    (
        "earthlike_full_32k",
        include_str!("../presets/earthlike_full_32k.json"),
    ),
];

/// The JSON of a built-in preset.
pub fn builtin(name: &str) -> Option<&'static str> {
    BUILTIN.iter().find(|(n, _)| *n == name).map(|(_, j)| *j)
}

/// Turns a preset argument into preset JSON: a share code (`DT1.…`), a built-in name, or a file.
pub fn resolve(arg: &str) -> Result<String, CoreError> {
    let arg = arg.trim();
    if share_code::is_share_code(arg) {
        return share_code::decode(arg);
    }
    if let Some(json) = builtin(arg) {
        return Ok(json.to_owned());
    }
    match std::fs::read_to_string(arg) {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && looks_like_a_name(arg) => {
            Err(CoreError::preset(
                "preset",
                format!(
                    "'{arg}' is not a built-in preset ({}), a file or a share code (DT1.…)",
                    BUILTIN.map(|(n, _)| n).join(", ")
                ),
            ))
        }
        Err(e) => Err(CoreError::io(arg, e)),
    }
}

fn looks_like_a_name(arg: &str) -> bool {
    !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Sets `timeline.start` and/or `timeline.length_myr` in a preset's JSON (the CLI's `--start` and
/// `--length-myr`; the server does the same in Java from `server.json`). Returns canonical JSON.
pub fn with_timeline(
    json: &str,
    start: Option<&str>,
    length_myr: Option<f64>,
) -> Result<String, CoreError> {
    let mut v: Value = serde_json::from_str(json)
        .map_err(|e| CoreError::preset("$", format!("not valid JSON: {e}")))?;
    let obj = v
        .as_object_mut()
        .ok_or_else(|| CoreError::preset("$", "expected a JSON object"))?;
    if start.is_none() && length_myr.is_none() {
        return Ok(v.to_string());
    }
    let t = obj
        .entry("timeline")
        .or_insert_with(|| Value::Object(Map::new()));
    let t = t
        .as_object_mut()
        .ok_or_else(|| CoreError::preset("$.timeline", "expected a JSON object"))?;
    if let Some(s) = start {
        t.insert("start".into(), json!(s));
    }
    if let Some(l) = length_myr {
        t.insert("length_myr".into(), json!(l));
    }
    Ok(v.to_string())
}

/// How the continents start (plan §5.1, the editor's "start configuration").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartConfig {
    Supercontinent,
    ScatteredContinents,
    Random,
}

impl StartConfig {
    pub const ALL: [StartConfig; 3] = [
        StartConfig::Supercontinent,
        StartConfig::ScatteredContinents,
        StartConfig::Random,
    ];

    pub fn name(self) -> &'static str {
        match self {
            StartConfig::Supercontinent => "supercontinent",
            StartConfig::ScatteredContinents => "scattered_continents",
            StartConfig::Random => "random",
        }
    }

    pub fn from_name(s: &str) -> Option<StartConfig> {
        StartConfig::ALL.into_iter().find(|c| c.name() == s)
    }
}

/// The shortest and longest history a timeline may ask for, in millions of years.
pub const LENGTH_MYR_RANGE: (f64, f64) = (1.0, 4600.0);

/// The player's timeline choice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timeline {
    pub start: StartConfig,
    pub length_myr: f64,
}

/// The keys every preset may carry next to its pipeline.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PresetHeader {
    pub schema: Option<u64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub placeholder: bool,
    /// `scale.circumference_blocks`.
    pub circumference: Option<u32>,
    pub timeline: Option<Timeline>,
}

/// The pipeline a preset runs.
#[derive(Debug, Clone, PartialEq)]
pub enum PresetBody {
    Dummy(DummyParams),
    /// P1-14: the deposit stage on a synthetic field history (`{"deposits_synthetic": {…}}`,
    /// `crate::deposits::SyntheticPreset`), until P1-03…P1-07 publish real fields.
    DepositsSynthetic(crate::deposits::SyntheticPreset),
}

/// A parsed preset.
#[derive(Debug, Clone, PartialEq)]
pub struct Preset {
    pub header: PresetHeader,
    pub body: PresetBody,
}

const HEADER_KEYS: [&str; 6] = [
    "schema",
    "name",
    "description",
    "placeholder",
    "scale",
    "timeline",
];

impl Preset {
    pub fn from_json(json: &str) -> Result<Preset, CoreError> {
        let value: Value = serde_json::from_str(json)
            .map_err(|e| CoreError::preset("$", format!("not valid JSON: {e}")))?;
        let obj = value
            .as_object()
            .ok_or_else(|| CoreError::preset("$", "expected a JSON object"))?;
        if let Some(key) = obj.keys().find(|k| {
            !["dummy", "deposits_synthetic"].contains(&k.as_str())
                && !HEADER_KEYS.contains(&k.as_str())
        }) {
            return Err(CoreError::preset(
                format!("$.{key}"),
                "unknown key (only the dummy and deposits_synthetic presets exist before P1-03)",
            ));
        }
        let header = parse_header(obj)?;
        if let Some(d) = obj.get("deposits_synthetic") {
            if obj.contains_key("dummy") {
                return Err(CoreError::preset(
                    "$",
                    "give one pipeline: dummy or deposits_synthetic",
                ));
            }
            let mut p = crate::deposits::SyntheticPreset::from_json(d)?;
            if let Some(c) = header.circumference
                && c != p.circumference
            {
                return Err(CoreError::preset(
                    "$.scale.circumference_blocks",
                    format!(
                        "{c} differs from deposits_synthetic's circumference {}",
                        p.circumference
                    ),
                ));
            }
            if let Some(t) = header.timeline {
                p.t_start_years = -t.length_myr * 1e6;
                p.t_end_years = 0.0;
            }
            p.stages()?;
            return Ok(Preset {
                header,
                body: PresetBody::DepositsSynthetic(p),
            });
        }
        let dummy = obj.get("dummy").ok_or_else(|| {
            CoreError::preset(
                "$",
                r#"only the dummy preset {"dummy":{"steps":N,"sleep_ms":M}} exists before P1-03"#,
            )
        })?;
        let mut params = DummyParams::from_json(dummy)?;
        let dummy_c = params
            .world
            .as_ref()
            .map_or(DEFAULT_DUMMY_CIRCUMFERENCE, |w| w.circumference);
        if let Some(c) = header.circumference
            && c != dummy_c
        {
            return Err(CoreError::preset(
                "$.scale.circumference_blocks",
                format!("{c} differs from the dummy world's circumference {dummy_c}"),
            ));
        }
        if let (Some(t), Some(w)) = (header.timeline, params.world.as_mut()) {
            w.t_start_years = -t.length_myr * 1e6;
            w.t_end_years = 0.0;
            w.keyframe_every_years = t.length_myr * 1e6 / 8.0;
        }
        Ok(Preset {
            header,
            body: PresetBody::Dummy(params),
        })
    }

    /// The canonical preset (what `preset.json` and checkpoints store).
    pub fn to_json(&self) -> Value {
        let mut v = match &self.body {
            PresetBody::Dummy(p) => p.to_json(),
            PresetBody::DepositsSynthetic(p) => json!({
                "deposits_synthetic": serde_json::to_value(p).expect("a preset serialises"),
            }),
        };
        let h = &self.header;
        let obj = v.as_object_mut().expect("a preset is an object");
        if let Some(s) = h.schema {
            obj.insert("schema".into(), json!(s));
        }
        if let Some(n) = &h.name {
            obj.insert("name".into(), json!(n));
        }
        if let Some(d) = &h.description {
            obj.insert("description".into(), json!(d));
        }
        if h.placeholder {
            obj.insert("placeholder".into(), json!(true));
        }
        if let Some(c) = h.circumference {
            obj.insert("scale".into(), json!({ "circumference_blocks": c }));
        }
        if let Some(t) = h.timeline {
            obj.insert(
                "timeline".into(),
                json!({ "start": t.start.name(), "length_myr": t.length_myr }),
            );
        }
        v
    }

    /// The preset's name, for `manifest.json` `source` (`run:<name>`): its `name`, else `dummy`.
    pub fn name(&self) -> &str {
        match (&self.header.name, &self.body) {
            (Some(n), _) => n,
            (None, PresetBody::Dummy(_)) => "dummy",
            (None, PresetBody::DepositsSynthetic(_)) => "deposits_synthetic",
        }
    }

    /// C in blocks: `scale.circumference_blocks`, else the pipeline's own.
    pub fn circumference(&self) -> u32 {
        self.header.circumference.unwrap_or(match &self.body {
            PresetBody::Dummy(p) => p
                .world
                .as_ref()
                .map_or(DEFAULT_DUMMY_CIRCUMFERENCE, |w| w.circumference),
            PresetBody::DepositsSynthetic(p) => p.circumference,
        })
    }

    /// The pipeline, in order.
    pub fn stages(&self) -> Vec<Box<dyn Stage>> {
        match &self.body {
            PresetBody::Dummy(p) => p.stages(),
            PresetBody::DepositsSynthetic(p) => {
                p.stages().expect("checked when the preset was parsed")
            }
        }
    }

    /// The world before the first stage.
    pub fn initial_world(&self) -> WorldState {
        match &self.body {
            PresetBody::Dummy(p) => p.initial_world(),
            PresetBody::DepositsSynthetic(p) => p.initial_world(),
        }
    }

    pub fn keyframe_policy(&self) -> KeyframePolicy {
        match &self.body {
            PresetBody::Dummy(p) => match &p.world {
                Some(w) => KeyframePolicy {
                    every_years: Some(w.keyframe_every_years),
                    size: w.keyframe_size,
                },
                None => KeyframePolicy::NONE,
            },
            PresetBody::DepositsSynthetic(p) => KeyframePolicy {
                every_years: Some(p.keyframe_every_years),
                size: (p.keyframe_size[0], p.keyframe_size[1]),
            },
        }
    }

    /// The synthetic terrain a placeholder preset bakes at `finalize_write`, if any.
    pub fn synth(&self) -> Option<synth::Kind> {
        match &self.body {
            PresetBody::Dummy(p) => p.synth,
            PresetBody::DepositsSynthetic(_) => None,
        }
    }
}

/// The circumference of a dummy preset without a `world` (`DummyParams::initial_world`).
const DEFAULT_DUMMY_CIRCUMFERENCE: u32 = 16_384;

fn parse_header(obj: &Map<String, Value>) -> Result<PresetHeader, CoreError> {
    let mut h = PresetHeader::default();
    if let Some(v) = obj.get("schema") {
        h.schema = Some(
            v.as_u64()
                .ok_or_else(|| CoreError::preset("$.schema", "expected an integer ≥ 0"))?,
        );
    }
    let text = |key: &str, max: usize| -> Result<Option<String>, CoreError> {
        let Some(v) = obj.get(key) else {
            return Ok(None);
        };
        let p = format!("$.{key}");
        let s = v
            .as_str()
            .ok_or_else(|| CoreError::preset(&p, "expected a string"))?;
        if s.chars().count() > max {
            return Err(CoreError::preset(
                &p,
                format!("longer than {max} characters"),
            ));
        }
        Ok(Some(s.to_owned()))
    };
    h.name = text("name", 64)?;
    h.description = text("description", 4000)?;
    if let Some(v) = obj.get("placeholder") {
        h.placeholder = v
            .as_bool()
            .ok_or_else(|| CoreError::preset("$.placeholder", "expected true or false"))?;
    }
    if let Some(v) = obj.get("scale") {
        let s = v
            .as_object()
            .ok_or_else(|| CoreError::preset("$.scale", "expected a JSON object"))?;
        if let Some(k) = s.keys().find(|k| k.as_str() != "circumference_blocks") {
            return Err(CoreError::preset(format!("$.scale.{k}"), "unknown key"));
        }
        if let Some(c) = s.get("circumference_blocks") {
            let p = "$.scale.circumference_blocks";
            let c = c
                .as_u64()
                .filter(|c| (1024..=1 << 20).contains(c) && c % 512 == 0)
                .ok_or_else(|| {
                    CoreError::preset(p, "expected a multiple of 512 from 1024 to 1048576")
                })?;
            h.circumference = Some(c as u32);
        }
    }
    if let Some(v) = obj.get("timeline") {
        h.timeline = Some(parse_timeline(v)?);
    }
    Ok(h)
}

fn parse_timeline(v: &Value) -> Result<Timeline, CoreError> {
    let path = "$.timeline";
    let t = v
        .as_object()
        .ok_or_else(|| CoreError::preset(path, "expected a JSON object"))?;
    if let Some(k) = t
        .keys()
        .find(|k| !matches!(k.as_str(), "start" | "length_myr"))
    {
        return Err(CoreError::preset(format!("{path}.{k}"), "unknown key"));
    }
    let names = StartConfig::ALL.map(StartConfig::name).join(", ");
    let start = t
        .get("start")
        .ok_or_else(|| {
            CoreError::preset(format!("{path}.start"), format!("missing (one of {names})"))
        })?
        .as_str()
        .and_then(StartConfig::from_name)
        .ok_or_else(|| {
            CoreError::preset(format!("{path}.start"), format!("expected one of {names}"))
        })?;
    let (lo, hi) = LENGTH_MYR_RANGE;
    let length_myr = t
        .get("length_myr")
        .ok_or_else(|| CoreError::preset(format!("{path}.length_myr"), "missing"))?
        .as_f64()
        .filter(|l| l.is_finite() && (lo..=hi).contains(l))
        .ok_or_else(|| {
            CoreError::preset(
                format!("{path}.length_myr"),
                format!("expected a number of million years from {lo} to {hi}"),
            )
        })?;
    Ok(Timeline { start, length_myr })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_of(json: &str) -> String {
        match Preset::from_json(json) {
            Err(CoreError::PresetInvalid { path, .. }) => path,
            other => panic!("expected PresetInvalid, got {other:?}"),
        }
    }

    #[test]
    fn builtins_parse_and_name_their_circumference() {
        for (name, json) in BUILTIN {
            let p = Preset::from_json(json).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(p.name(), name);
            assert!(p.header.placeholder, "{name} is a Phase 1 placeholder");
            assert!(p.header.timeline.is_none(), "{name}: no default story");
            let want = if name.ends_with("_32k") {
                32_768
            } else {
                16_384
            };
            assert_eq!(p.circumference(), want, "{name}");
            assert_eq!(p.synth(), Some(synth::Kind::WrapTest), "{name}");
            // Canonical and stable: parsing the canonical form gives the same preset.
            let again = Preset::from_json(&p.to_json().to_string()).unwrap();
            assert_eq!(again, p, "{name}");
        }
        assert_eq!(resolve("earthlike_quick_16k").unwrap(), BUILTIN[0].1);
    }

    #[test]
    fn resolve_takes_codes_names_and_files() {
        let code = share_code::encode(BUILTIN[2].1).unwrap();
        let decoded = resolve(&code).unwrap();
        assert_eq!(
            Preset::from_json(&decoded).unwrap(),
            Preset::from_json(BUILTIN[2].1).unwrap()
        );
        let dir = std::env::temp_dir().join(format!("dt-preset-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("p.json");
        std::fs::write(&file, BUILTIN[1].1).unwrap();
        assert_eq!(resolve(file.to_str().unwrap()).unwrap(), BUILTIN[1].1);
        match resolve("earthlike_huge_64k") {
            Err(CoreError::PresetInvalid { reason, .. }) => {
                assert!(reason.contains("earthlike_quick_16k"), "{reason}")
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            resolve(dir.join("missing.json").to_str().unwrap()),
            Err(CoreError::Io { .. })
        ));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn timeline_drives_the_dummy_clock() {
        let json = with_timeline(BUILTIN[0].1, Some("scattered_continents"), Some(800.0)).unwrap();
        let p = Preset::from_json(&json).unwrap();
        assert_eq!(
            p.header.timeline,
            Some(Timeline {
                start: StartConfig::ScatteredContinents,
                length_myr: 800.0
            })
        );
        let PresetBody::Dummy(d) = &p.body else {
            panic!("a dummy preset");
        };
        let w = d.world.as_ref().unwrap();
        assert_eq!((w.t_start_years, w.t_end_years), (-8.0e8, 0.0));
        assert_eq!(w.keyframe_every_years, 1.0e8);
        assert_eq!(p.to_json()["timeline"]["start"], "scattered_continents");
        assert_eq!(
            Preset::from_json(&p.to_json().to_string()).unwrap(),
            p,
            "idempotent"
        );
    }

    #[test]
    fn bad_headers_name_the_field() {
        let dummy = r#""dummy":{"steps":1,"sleep_ms":0}"#;
        assert_eq!(path_of(&format!(r#"{{"schema":-1,{dummy}}}"#)), "$.schema");
        assert_eq!(
            path_of(&format!(
                r#"{{"scale":{{"circumference_blocks":1000}},{dummy}}}"#
            )),
            "$.scale.circumference_blocks"
        );
        assert_eq!(
            path_of(&format!(
                r#"{{"scale":{{"circumference_blocks":32768}},{dummy}}}"#
            )),
            "$.scale.circumference_blocks"
        );
        assert_eq!(
            path_of(&format!(r#"{{"scale":{{"radius":1}},{dummy}}}"#)),
            "$.scale.radius"
        );
        assert_eq!(
            path_of(&format!(r#"{{"timeline":{{"length_myr":10}},{dummy}}}"#)),
            "$.timeline.start"
        );
        assert_eq!(
            path_of(&format!(
                r#"{{"timeline":{{"start":"pangaea","length_myr":10}},{dummy}}}"#
            )),
            "$.timeline.start"
        );
        assert_eq!(
            path_of(&format!(
                r#"{{"timeline":{{"start":"random","length_myr":0}},{dummy}}}"#
            )),
            "$.timeline.length_myr"
        );
        assert_eq!(
            path_of(&format!(r#"{{"placeholder":1,{dummy}}}"#)),
            "$.placeholder"
        );
        let ok = Preset::from_json(&format!(
            r#"{{"scale":{{"circumference_blocks":16384}},"timeline":{{"start":"random","length_myr":10}},{dummy}}}"#
        ))
        .unwrap();
        assert_eq!(ok.circumference(), 16_384);
        assert_eq!(ok.name(), "dummy");
    }
}

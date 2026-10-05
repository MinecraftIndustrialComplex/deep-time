// SPDX-License-Identifier: LGPL-3.0-or-later
//! Presets (plan §5.1): parsing, the built-in presets, and how a preset argument is resolved.
//!
//! The pipelines are the dummy one (`dummy.rs`), P1-14's deposit stage on a synthetic history
//! (`{"deposits_synthetic": {…}}`, `deposits::SyntheticPreset`) and, from P1-03, the planet
//! pipeline (a preset with a `planet` section, `params::PlanetPreset`: the plates stage first). A
//! preset is its pipeline's keys plus optional header keys that every preset carries:
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
//! - `placeholder: true` marks a stand-in that is not a simulated planet (the dummy pipeline with a
//!   synthetic terrain; P1-13's built-ins until P1-08, now only test presets); the server logs a
//!   warning for one and the workbench hides them behind its dev toggle.
//!
//! Built-in presets are JSON files in `crates/deeptime-core/presets/`, compiled into the core and
//! copied into the mod jar (`deeptime/presets/<name>.json`, Gradle `processResources`). Since P1-08
//! they are the planet pipeline presets under their game names (`earthlike_quick_16k` =
//! `pipeline_quick_16k`, … ; `tools/dev/sync_builtin_presets.py` writes them from the pipeline
//! files, `builtins_are_the_pipeline_presets` keeps the two equal).

use serde_json::{Map, Value, json};

use crate::dummy::DummyParams;
use crate::error::CoreError;
use crate::keyframes::KeyframePolicy;
use crate::params::{PLANET_KEYS, PlanetPreset, SCALE_KEYS};
use crate::plates::PlatesStage;
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

/// Turns a preset argument into preset JSON: a share code (`DT1.…`), a built-in name (or a plate
/// preset's, `params::PLATES_BUILTIN`), or a file.
pub fn resolve(arg: &str) -> Result<String, CoreError> {
    let arg = arg.trim();
    if share_code::is_share_code(arg) {
        return share_code::decode(arg);
    }
    if let Some(json) = builtin(arg).or_else(|| crate::params::planet_builtin(arg)) {
        return Ok(json.to_owned());
    }
    match std::fs::read_to_string(arg) {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && looks_like_a_name(arg) => {
            Err(CoreError::preset(
                "preset",
                format!(
                    "'{arg}' is not a built-in preset ({}; plate pipeline: {}; plates and \
                     surface: {}), a file or a share code (DT1.…)",
                    BUILTIN.map(|(n, _)| n).join(", "),
                    crate::params::PLATES_BUILTIN.map(|(n, _)| n).join(", "),
                    crate::params::PIPELINE_BUILTIN.map(|(n, _)| n).join(", ")
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

/// Sets `timeline.stop_myr` in a preset's JSON (the CLI's `--stop-myr`; the editor's Continue,
/// Stop here and Play this epoch); `None` removes it. The timeline must exist. Returns canonical
/// JSON.
pub fn with_stop(json: &str, stop_myr: Option<f64>) -> Result<String, CoreError> {
    let mut v: Value = serde_json::from_str(json)
        .map_err(|e| CoreError::preset("$", format!("not valid JSON: {e}")))?;
    let t = v
        .get_mut("timeline")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| CoreError::preset("$.timeline", "a stop needs a timeline"))?;
    match stop_myr {
        Some(s) => {
            t.insert("stop_myr".into(), json!(s));
        }
        None => {
            t.remove("stop_myr");
        }
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
    /// The planned length: the run starts `length_myr` before the present (world time 0).
    pub length_myr: f64,
    /// Where the run stops, Myr after its start (`timeline.stop_myr`; docs/design/
    /// playability-steering.md §1): `None` is the planned length. Less stops the run early (world
    /// time ends before the present), more continues it past the present.
    pub stop_myr: Option<f64>,
}

impl Timeline {
    /// Myr from the start to the stop.
    pub fn run_myr(&self) -> f64 {
        self.stop_myr.unwrap_or(self.length_myr)
    }

    /// The timeline's JSON.
    pub fn to_json(&self) -> Value {
        let mut v = json!({ "start": self.start.name(), "length_myr": self.length_myr });
        if let Some(s) = self.stop_myr {
            v["stop_myr"] = json!(s);
        }
        v
    }
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
    /// The planet pipeline (a `planet` section; `params.rs`): the plates stage (P1-03).
    Planet(Box<PlanetPreset>),
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
        let planet = obj.contains_key("planet");
        if let Some(key) = obj.keys().find(|k| {
            !["dummy", "deposits_synthetic"].contains(&k.as_str())
                && !HEADER_KEYS.contains(&k.as_str())
                && !(planet && PLANET_KEYS.contains(&k.as_str()))
        }) {
            return Err(CoreError::preset(
                format!("$.{key}"),
                "unknown key (a preset is a dummy, deposits_synthetic or planet pipeline plus \
                 its header)",
            ));
        }
        let header = parse_header(obj, planet)?;
        if planet {
            if obj.contains_key("dummy") || obj.contains_key("deposits_synthetic") {
                return Err(CoreError::preset(
                    "$",
                    "give one pipeline: dummy, deposits_synthetic or planet",
                ));
            }
            let p = PlanetPreset::from_obj(obj, header.name.as_deref(), header.timeline)?;
            return Ok(Preset {
                header,
                body: PresetBody::Planet(Box::new(p)),
            });
        }
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
                p.t_end_years = (t.run_myr() - t.length_myr) * 1e6;
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
            w.t_end_years = (t.run_myr() - t.length_myr) * 1e6;
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
            PresetBody::Planet(p) => p.to_json(),
        };
        let full_scale = matches!(self.body, PresetBody::Planet(_));
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
        if let Some(c) = h.circumference
            && !full_scale
        {
            obj.insert("scale".into(), json!({ "circumference_blocks": c }));
        }
        if let Some(t) = h.timeline {
            obj.insert("timeline".into(), t.to_json());
        }
        v
    }

    /// The preset's name, for `manifest.json` `source` (`run:<name>`): its `name`, else `dummy`.
    pub fn name(&self) -> &str {
        match (&self.header.name, &self.body) {
            (Some(n), _) => n,
            (None, PresetBody::Dummy(_)) => "dummy",
            (None, PresetBody::DepositsSynthetic(_)) => "deposits_synthetic",
            (None, PresetBody::Planet(p)) => &p.name,
        }
    }

    /// Checks that the preset can run: a planet preset needs its timeline chosen (there is no
    /// default story); the placeholder pipelines run without one.
    pub fn check_runnable(&self) -> Result<(), CoreError> {
        match &self.body {
            PresetBody::Planet(p) => p.check_runnable(),
            _ => Ok(()),
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
            PresetBody::Planet(p) => p.scale.circumference_blocks,
        })
    }

    /// The pipeline, in order.
    pub fn stages(&self) -> Vec<Box<dyn Stage>> {
        match &self.body {
            PresetBody::Dummy(p) => p.stages(),
            PresetBody::DepositsSynthetic(p) => {
                p.stages().expect("checked when the preset was parsed")
            }
            PresetBody::Planet(p) => {
                let mut stages: Vec<Box<dyn Stage>> =
                    vec![Box::new(PlatesStage::new((**p).clone()))];
                if p.climate.is_some() {
                    // Coupled to the plates, before the surface, which reads its fields (P1-05).
                    stages.push(Box::new(crate::climate::ClimateStage::new((**p).clone())));
                }
                if p.surface.is_some() {
                    // Coupled to the plates: one surface step after every plate step (plan §8).
                    stages.push(Box::new(crate::surface::SurfaceStage::new((**p).clone())));
                    if p.volcanoes.is_some() {
                        // Coupled too: builds its edifices on the surface each step, before the
                        // structure stage reads the strata log (P1-21).
                        stages.push(Box::new(crate::surface::volcanoes::VolcanoStage::new(
                            (**p).clone(),
                        )));
                    }
                    if p.impacts.is_some() {
                        // Coupled too: craters are cut into the surface and laid in the strata
                        // log before the structure stage reads it (P2-08).
                        stages.push(Box::new(crate::surface::impacts::ImpactStage::new(
                            (**p).clone(),
                        )));
                    }
                    // Column strata and metamorphism on the surface's strata log (P1-07).
                    stages.push(Box::new(crate::structure::StructureStage::new(
                        (**p).clone(),
                    )));
                }
                if p.deposits.is_some() {
                    // After the group: reads the history the plates and surface recorded (P1-14).
                    stages.push(Box::new(
                        crate::deposits::DepositStage::recorded(p)
                            .expect("checked when the preset was parsed"),
                    ));
                }
                if p.young.is_some() {
                    // The last Myr of erosion on L2, then amplification to L3 (P1-06). After the
                    // deposits: they read the L1 history and neither waits for L2 or L3.
                    stages.push(Box::new(crate::surface::young::YoungStage::new(
                        (**p).clone(),
                    )));
                    if p.amplify.is_some() {
                        stages.push(Box::new(crate::surface::amplify::AmplifyStage::new(
                            (**p).clone(),
                        )));
                    }
                }
                stages
            }
        }
    }

    /// The world before the first stage.
    pub fn initial_world(&self) -> WorldState {
        match &self.body {
            PresetBody::Dummy(p) => p.initial_world(),
            PresetBody::DepositsSynthetic(p) => p.initial_world(),
            PresetBody::Planet(p) => {
                WorldState::new(p.seed, p.scale.circumference_blocks, p.t_start_years())
            }
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
            PresetBody::Planet(p) => KeyframePolicy {
                every_years: Some(p.keyframes.every_myr * 1.0e6),
                size: (p.keyframes.size[0], p.keyframes.size[1]),
            },
        }
    }

    /// How the keyframes' view layers read heights (the satellite view): the planet's scale and
    /// model sphere, else the defaults.
    pub fn view_scale(&self) -> crate::keyframes::ViewScale {
        match &self.body {
            PresetBody::Planet(p) => crate::keyframes::ViewScale {
                sea_level_y: f64::from(p.scale.sea_level_y),
                m_per_block_v: p.scale.m_per_block_v,
                radius_m: p.tectonics.model_radius_km * 1000.0,
                m_per_block_h: p.scale.m_per_block_h,
                rivers_scale: p.scale.feature_scale.rivers,
                hydro: p.hydro.unwrap_or_default(),
            },
            _ => crate::keyframes::ViewScale::default(),
        }
    }

    /// The synthetic terrain a placeholder preset bakes at `finalize_write`, if any.
    pub fn synth(&self) -> Option<synth::Kind> {
        self.bake().map(|b| match b {
            Bake::Analytic(k) => k,
            Bake::DepositHistory { .. } => synth::Kind::DepositHistory,
        })
    }

    /// What `finalize_write` bakes into the world data besides the run's own files.
    pub fn bake(&self) -> Option<Bake> {
        match &self.body {
            PresetBody::Dummy(p) => p.synth.map(Bake::Analytic),
            PresetBody::DepositsSynthetic(p) => p.terrain.as_ref().map(|t| Bake::DepositHistory {
                params: p.synthetic(),
                strata: t.strata(),
            }),
            PresetBody::Planet(_) => None,
        }
    }

    /// This preset with the game's content bundle applied (plan §5.4): the bundle's
    /// `essentials` (the `deeptime:essentials` registry, P1-15) replace the preset's list, and its
    /// `climate_placement` (a datapack's copy of `data/deeptime/deeptime/climate_mapping/placement.json`,
    /// P1-09) replaces the bake's default class placement table. An empty bundle, or one without
    /// these keys, changes nothing; the dummy pipeline has neither.
    pub fn with_content(mut self, content_json: &str) -> Result<Preset, CoreError> {
        if content_json.trim().is_empty() {
            return Ok(self);
        }
        let v: Value = serde_json::from_str(content_json)
            .map_err(|e| CoreError::preset("content", format!("not valid JSON: {e}")))?;
        if let Some(t) = v.get(crate::bake::placement::CONTENT_KEY) {
            let table = crate::bake::placement::Placement::from_value(t)?;
            if let PresetBody::Planet(p) = &mut self.body {
                p.placement = Some(table);
            }
        }
        let Some(ess) = v.get("essentials") else {
            return Ok(self);
        };
        let list: Vec<crate::deposits::playability::Essential> =
            serde_json::from_value(ess.clone())
                .map_err(|e| CoreError::preset("content.essentials", e.to_string()))?;
        match &mut self.body {
            PresetBody::DepositsSynthetic(p) => p.deposits.essentials = list,
            PresetBody::Planet(p) => {
                if let Some(d) = &mut p.deposits {
                    d.essentials = list;
                }
            }
            PresetBody::Dummy(_) => {}
        }
        Ok(self)
    }
}

/// What a preset bakes at `finalize_write` (the stand-ins for P1-08's bake).
#[derive(Debug, Clone, PartialEq)]
pub enum Bake {
    /// A P0-09 synthetic terrain (`"synth": "wrap-test"` of a dummy preset, P1-13).
    Analytic(synth::Kind),
    /// A synthetic deposit world's terrain (`deposits_synthetic.terrain`, P1-15): the history's
    /// present elevation and the given horizons.
    DepositHistory {
        params: crate::deposits::synthetic::SyntheticParams,
        strata: synth::Strata,
    },
}

/// The circumference of a dummy preset without a `world` (`DummyParams::initial_world`).
const DEFAULT_DUMMY_CIRCUMFERENCE: u32 = 16_384;

fn parse_header(obj: &Map<String, Value>, planet: bool) -> Result<PresetHeader, CoreError> {
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
        // A planet preset's scale carries more keys (`params::ScaleParams` checks them). Any
        // preset may name its world's projection; a synthetic world (not a planet) is Lambert, and
        // says so for the game, which reads the key to know the world's east–west period.
        if let Some(k) = s.keys().find(|k| {
            k.as_str() != "circumference_blocks"
                && k.as_str() != "projection"
                && !(planet && SCALE_KEYS.contains(&k.as_str()))
        }) {
            return Err(CoreError::preset(format!("$.scale.{k}"), "unknown key"));
        }
        if !planet
            && let Some(v) = s.get("projection")
            && v.as_str() != Some("lambert")
        {
            return Err(CoreError::preset(
                "$.scale.projection",
                "a synthetic world is laid out in \"lambert\"",
            ));
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
        .find(|k| !matches!(k.as_str(), "start" | "length_myr" | "stop_myr"))
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
    let stop_myr = match t.get("stop_myr") {
        None => None,
        Some(v) => Some(
            v.as_f64()
                .filter(|s| s.is_finite() && *s > 0.0 && *s <= hi)
                .ok_or_else(|| {
                    CoreError::preset(
                        format!("{path}.stop_myr"),
                        format!("expected a number of million years above 0, at most {hi}"),
                    )
                })?,
        ),
    };
    Ok(Timeline {
        start,
        length_myr,
        stop_myr,
    })
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
            assert!(
                !p.header.placeholder,
                "{name} is a simulated planet since P1-08"
            );
            assert!(p.header.timeline.is_none(), "{name}: no default story");
            let want = if name.ends_with("_32k") {
                32_768
            } else {
                16_384
            };
            assert_eq!(p.circumference(), want, "{name}");
            assert_eq!(p.synth(), None, "{name}");
            let stages: Vec<String> = p.stages().iter().map(|s| s.name().to_owned()).collect();
            assert_eq!(
                stages,
                [
                    "plates",
                    "climate",
                    "surface",
                    "volcanoes",
                    "impacts",
                    "structure",
                    "deposits",
                    "young",
                    "amplify"
                ],
                "{name} runs the whole pipeline"
            );
            // Canonical and stable: parsing the canonical form gives the same preset.
            let again = Preset::from_json(&p.to_json().to_string()).unwrap();
            assert_eq!(again, p, "{name}");
        }
        assert_eq!(resolve("earthlike_quick_16k").unwrap(), BUILTIN[0].1);
    }

    /// The built-ins are the calibrated pipeline presets under their game names; only `name` and
    /// `description` differ (`tools/dev/sync_builtin_presets.py` writes them).
    #[test]
    fn builtins_are_the_pipeline_presets() {
        for (name, json) in BUILTIN {
            let pipeline = name.replace("earthlike_", "pipeline_");
            let mut ours: Value = serde_json::from_str(json).unwrap();
            let mut theirs: Value =
                serde_json::from_str(crate::params::planet_builtin(&pipeline).unwrap()).unwrap();
            for v in [&mut ours, &mut theirs] {
                let o = v.as_object_mut().unwrap();
                o.remove("name");
                o.remove("description");
            }
            assert!(
                ours == theirs,
                "{name} differs from {pipeline}: run tools/dev/sync_builtin_presets.py"
            );
        }
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

    /// A placeholder preset as P1-13 shipped them: the dummy pipeline plus a synthetic terrain.
    const PLACEHOLDER: &str = r#"{"schema":0,"name":"placeholder_16k","placeholder":true,
        "scale":{"circumference_blocks":16384},
        "dummy":{"steps":2,"sleep_ms":0,"stages":["dummy_drift","dummy_erode"],"synth":"wrap-test",
                 "world":{"circumference":16384,"grid_r":16,"t_start_years":-1.0e9,"t_end_years":0.0,
                          "keyframe_every_years":1.25e8,"keyframe_size":[256,128],
                          "event_every_steps":5,"sites":3,"seed":1}}}"#;

    #[test]
    fn placeholders_still_parse_and_bake_their_synthetic_terrain() {
        let p = Preset::from_json(PLACEHOLDER).unwrap();
        assert!(p.header.placeholder);
        assert_eq!(p.synth(), Some(synth::Kind::WrapTest));
        assert_eq!(p.circumference(), 16_384);
    }

    #[test]
    fn timeline_drives_the_dummy_clock() {
        let json = with_timeline(PLACEHOLDER, Some("scattered_continents"), Some(800.0)).unwrap();
        let p = Preset::from_json(&json).unwrap();
        assert_eq!(
            p.header.timeline,
            Some(Timeline {
                start: StartConfig::ScatteredContinents,
                length_myr: 800.0,
                stop_myr: None
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

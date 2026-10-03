// SPDX-License-Identifier: LGPL-3.0-or-later
//! The dummy preset and its stages: the pipeline's stand-in until the real stages exist (P0-07,
//! extended in P1-01).
//!
//! ```json
//! {"dummy": {"steps": 20, "sleep_ms": 0,
//!            "stages": ["dummy_drift", "dummy_erode"],
//!            "world": {"circumference": 4096, "grid_r": 16, "t_start_years": -4.0e8,
//!                      "t_end_years": 0.0, "keyframe_every_years": 5.0e7,
//!                      "keyframe_size": [256, 128], "event_every_steps": 4, "sites": 3,
//!                      "seed": 7}}}
//! ```
//!
//! `steps` and `sleep_ms` are required; everything else is optional. Without `world` the preset
//! behaves exactly as in Phase 0 (plan §6.1): one stage `dummy` per name in `stages` (default
//! `["dummy"]`) counting to `steps` and sleeping `sleep_ms` per step, with no fields, events,
//! sites or keyframes. With `world` the stages move a toy planet: a clock from `t_start_years` to
//! `t_end_years` split evenly over every stage's steps, an `elevation` field (block y) on the
//! L1-like grid C/grid_r × C/(2·grid_r) with a drifting continent, an event every
//! `event_every_steps` steps (a causal chain), `sites` deposit sites with their forming events at
//! the very end, and keyframes of `elevation` every `keyframe_every_years`. Two test hooks:
//! `reserve_bytes` asks the memory cap for that much before a stage's first step, and
//! `panic_at_step` panics at that step of every stage (the panic guard and session poisoning).
//!
//! `"synth": "wrap-test" | "lake-test"` (P1-13, needs `world`) makes the run's world data
//! playable: `finalize_write` bakes that synthetic terrain (`crate::synth`, P0-09's layers) at the
//! world's circumference next to the events, sites and keyframes. The built-in placeholder
//! presets use it until the real stages and the bake (P1-03…P1-08) exist.

use std::f64::consts::PI;
use std::time::Duration;

use deeptime_format::{Extraction, GridDesc, Place, Product, Site, SiteKind};
use serde_json::{Map, Value, json};

use crate::error::CoreError;
use crate::stage::{Stage, StageContext, StageEstimate, from_state, to_state};
use crate::synth;
use crate::world::{Field, WorldState};

const MAX_DUMMY_STEPS: u64 = 10_000_000;
const MAX_DUMMY_SLEEP_MS: u64 = 10_000;
const MAX_STAGES: usize = 16;
/// The source of every dummy event.
pub const SOURCE: &str = "deeptime:dummy";
/// The field the dummy world moves.
pub const ELEVATION: &str = "elevation";

/// The toy planet of a dummy run.
#[derive(Debug, Clone, PartialEq)]
pub struct DummyWorld {
    pub circumference: u32,
    pub grid_r: u32,
    pub t_start_years: f64,
    pub t_end_years: f64,
    pub keyframe_every_years: f64,
    /// Keyframe width and height in pixels.
    pub keyframe_size: (u32, u32),
    pub event_every_steps: u64,
    pub sites: u32,
    pub seed: u64,
}

impl DummyWorld {
    pub fn grid(&self) -> GridDesc {
        GridDesc::for_planet("L1", self.circumference, self.grid_r)
            .expect("validated when the preset was parsed")
    }
}

/// Parameters of the dummy preset.
#[derive(Debug, Clone, PartialEq)]
pub struct DummyParams {
    pub steps: u64,
    pub sleep_ms: u64,
    pub reserve_bytes: u64,
    pub panic_at_step: Option<u64>,
    /// Stage names, in order.
    pub stages: Vec<String>,
    pub world: Option<DummyWorld>,
    /// The synthetic terrain `finalize_write` bakes (needs `world`).
    pub synth: Option<synth::Kind>,
}

type Obj = Map<String, Value>;

fn preset_err(path: &str, reason: impl Into<String>) -> CoreError {
    CoreError::preset(path, reason)
}

fn check_keys(obj: &Obj, path: &str, allowed: &[&str]) -> Result<(), CoreError> {
    match obj.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(k) => Err(preset_err(&format!("{path}.{k}"), "unknown key")),
        None => Ok(()),
    }
}

fn uint(obj: &Obj, path: &str, name: &str, max: u64) -> Result<Option<u64>, CoreError> {
    let Some(v) = obj.get(name) else {
        return Ok(None);
    };
    let p = format!("{path}.{name}");
    let n = v
        .as_u64()
        .ok_or_else(|| preset_err(&p, "expected an integer ≥ 0"))?;
    if n > max {
        return Err(preset_err(&p, format!("{n} is above the maximum {max}")));
    }
    Ok(Some(n))
}

fn float(obj: &Obj, path: &str, name: &str) -> Result<Option<f64>, CoreError> {
    let Some(v) = obj.get(name) else {
        return Ok(None);
    };
    let n = v
        .as_f64()
        .filter(|f| f.is_finite())
        .ok_or_else(|| preset_err(&format!("{path}.{name}"), "expected a finite number"))?;
    Ok(Some(n))
}

impl DummyParams {
    /// Parses `$.dummy`.
    pub fn from_json(dummy: &Value) -> Result<DummyParams, CoreError> {
        let path = "$.dummy";
        let obj = dummy
            .as_object()
            .ok_or_else(|| preset_err(path, "expected a JSON object"))?;
        check_keys(
            obj,
            path,
            &[
                "steps",
                "sleep_ms",
                "reserve_bytes",
                "panic_at_step",
                "stages",
                "world",
                "synth",
            ],
        )?;
        let required = |name: &str, max: u64| -> Result<u64, CoreError> {
            uint(obj, path, name, max)?
                .ok_or_else(|| preset_err(&format!("{path}.{name}"), "missing"))
        };
        let steps = required("steps", MAX_DUMMY_STEPS)?;
        let sleep_ms = required("sleep_ms", MAX_DUMMY_SLEEP_MS)?;
        let stages = match obj.get("stages") {
            None => vec!["dummy".to_string()],
            Some(v) => parse_stage_names(v)?,
        };
        let world = match obj.get("world") {
            None => None,
            Some(v) => Some(parse_world(v)?),
        };
        let synth = match obj.get("synth") {
            None => None,
            Some(v) => Some(parse_synth(v, world.as_ref())?),
        };
        Ok(DummyParams {
            steps,
            sleep_ms,
            reserve_bytes: uint(obj, path, "reserve_bytes", u64::MAX)?.unwrap_or(0),
            panic_at_step: uint(obj, path, "panic_at_step", u64::MAX)?,
            stages,
            world,
            synth,
        })
    }

    /// The canonical JSON (what `preset.json` and checkpoints store).
    pub fn to_json(&self) -> Value {
        let mut dummy = json!({ "steps": self.steps, "sleep_ms": self.sleep_ms });
        if self.reserve_bytes > 0 {
            dummy["reserve_bytes"] = json!(self.reserve_bytes);
        }
        if let Some(at) = self.panic_at_step {
            dummy["panic_at_step"] = json!(at);
        }
        if self.stages != ["dummy"] {
            dummy["stages"] = json!(self.stages);
        }
        if let Some(w) = &self.world {
            dummy["world"] = json!({
                "circumference": w.circumference,
                "grid_r": w.grid_r,
                "t_start_years": w.t_start_years,
                "t_end_years": w.t_end_years,
                "keyframe_every_years": w.keyframe_every_years,
                "keyframe_size": [w.keyframe_size.0, w.keyframe_size.1],
                "event_every_steps": w.event_every_steps,
                "sites": w.sites,
                "seed": w.seed,
            });
        }
        if let Some(k) = self.synth {
            dummy["synth"] = json!(k.name());
        }
        json!({ "dummy": dummy })
    }

    /// The pipeline.
    pub fn stages(&self) -> Vec<Box<dyn Stage>> {
        (0..self.stages.len())
            .map(|i| Box::new(DummyStage::new(self.clone(), i)) as Box<dyn Stage>)
            .collect()
    }

    /// The world before the first stage.
    pub fn initial_world(&self) -> WorldState {
        match &self.world {
            Some(w) => WorldState::new(w.seed, w.circumference, w.t_start_years),
            None => WorldState::new(0, 16_384, 0.0),
        }
    }
}

fn parse_synth(v: &Value, world: Option<&DummyWorld>) -> Result<synth::Kind, CoreError> {
    let path = "$.dummy.synth";
    let names = synth::Kind::ALL.map(synth::Kind::name).join(", ");
    let kind = v
        .as_str()
        .and_then(synth::Kind::from_name)
        .ok_or_else(|| preset_err(path, format!("expected one of {names}")))?;
    let w = world.ok_or_else(|| preset_err(path, "needs $.dummy.world (its circumference)"))?;
    synth::check_circumference(w.circumference).map_err(|e| {
        preset_err(
            "$.dummy.world.circumference",
            format!("{} for synth: {e}", w.circumference),
        )
    })?;
    Ok(kind)
}

fn parse_stage_names(v: &Value) -> Result<Vec<String>, CoreError> {
    let path = "$.dummy.stages";
    let arr = v
        .as_array()
        .ok_or_else(|| preset_err(path, "expected an array of stage names"))?;
    if arr.is_empty() || arr.len() > MAX_STAGES {
        return Err(preset_err(
            path,
            format!("expected 1 to {MAX_STAGES} names"),
        ));
    }
    let mut names: Vec<String> = Vec::with_capacity(arr.len());
    for (k, n) in arr.iter().enumerate() {
        let p = format!("{path}[{k}]");
        let s = n
            .as_str()
            .ok_or_else(|| preset_err(&p, "expected a string"))?;
        let ok = !s.is_empty()
            && s.len() <= 32
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !ok {
            return Err(preset_err(&p, "a stage name is 1–32 of [a-z0-9_]"));
        }
        if names.iter().any(|x| x == s) {
            return Err(preset_err(&p, format!("stage {s} appears twice")));
        }
        names.push(s.to_owned());
    }
    Ok(names)
}

fn parse_world(v: &Value) -> Result<DummyWorld, CoreError> {
    let path = "$.dummy.world";
    let obj = v
        .as_object()
        .ok_or_else(|| preset_err(path, "expected a JSON object"))?;
    check_keys(
        obj,
        path,
        &[
            "circumference",
            "grid_r",
            "t_start_years",
            "t_end_years",
            "keyframe_every_years",
            "keyframe_size",
            "event_every_steps",
            "sites",
            "seed",
        ],
    )?;
    let circumference = uint(obj, path, "circumference", 1 << 16)?.unwrap_or(4096) as u32;
    let grid_r = uint(obj, path, "grid_r", 1024)?.unwrap_or(16) as u32;
    if grid_r == 0 || circumference < 64 || !circumference.is_multiple_of(2 * grid_r) {
        return Err(preset_err(
            &format!("{path}.circumference"),
            format!(
                "{circumference} must be ≥ 64 and a multiple of 2·grid_r = {}",
                2 * grid_r
            ),
        ));
    }
    let (gw, gh) = (circumference / grid_r, circumference / grid_r / 2);
    if gw > 8192 {
        return Err(preset_err(
            &format!("{path}.grid_r"),
            format!("the grid would be {gw} cells wide (at most 8192)"),
        ));
    }
    let t_start = float(obj, path, "t_start_years")?.unwrap_or(-5.0e8);
    let t_end = float(obj, path, "t_end_years")?.unwrap_or(0.0);
    if t_end <= t_start {
        return Err(preset_err(
            &format!("{path}.t_end_years"),
            "must be after t_start_years",
        ));
    }
    let every = float(obj, path, "keyframe_every_years")?.unwrap_or((t_end - t_start) / 8.0);
    if every <= 0.0 {
        return Err(preset_err(
            &format!("{path}.keyframe_every_years"),
            "must be > 0",
        ));
    }
    let keyframe_size = match obj.get("keyframe_size") {
        None => (gw, gh),
        Some(v) => {
            let p = format!("{path}.keyframe_size");
            let a = v
                .as_array()
                .filter(|a| a.len() == 2)
                .ok_or_else(|| preset_err(&p, "expected [width, height]"))?;
            let dim = |x: &Value| x.as_u64().filter(|n| (1..=4096).contains(n));
            match (dim(&a[0]), dim(&a[1])) {
                (Some(w), Some(h)) => (w as u32, h as u32),
                _ => return Err(preset_err(&p, "width and height are 1 to 4096")),
            }
        }
    };
    Ok(DummyWorld {
        circumference,
        grid_r,
        t_start_years: t_start,
        t_end_years: t_end,
        keyframe_every_years: every,
        keyframe_size,
        event_every_steps: uint(obj, path, "event_every_steps", u64::MAX)?.unwrap_or(5),
        sites: uint(obj, path, "sites", 1000)?.unwrap_or(3) as u32,
        seed: uint(obj, path, "seed", u64::MAX)?.unwrap_or(1),
    })
}

/// One dummy stage: step `k` of stage `index` is step `index · steps + k` of the whole run.
pub struct DummyStage {
    params: DummyParams,
    index: usize,
    done: u64,
}

/// The kinds the dummy world emits, in rotation (their params are in `EVENTS.md`).
pub const DUMMY_KINDS: [&str; 4] = [
    "deeptime:rift_onset",
    "deeptime:collision",
    "deeptime:orogeny_phase",
    "deeptime:erosion_pulse",
];

impl DummyStage {
    pub fn new(params: DummyParams, index: usize) -> Self {
        DummyStage {
            params,
            index,
            done: 0,
        }
    }

    fn total_run_steps(&self) -> u64 {
        self.params.steps * self.params.stages.len() as u64
    }

    /// Advances the toy planet to global step `g` (0-based, the step just finished).
    fn move_world(
        &self,
        w: &DummyWorld,
        world: &mut WorldState,
        g: u64,
        ctx: &StageContext<'_>,
    ) -> Result<(), CoreError> {
        let total = self.total_run_steps().max(1);
        let dt = (w.t_end_years - w.t_start_years) / total as f64;
        world.t_years = w.t_start_years + dt * (g + 1) as f64;
        let p = (g + 1) as f64 / total as f64;
        let continent = Continent::at(p);

        if !world.fields.contains_key(ELEVATION) {
            let grid = w.grid();
            ctx.reserve(Field::bytes_for(&grid))?;
            world
                .fields
                .insert(ELEVATION.to_owned(), Field::filled(grid, 0.0));
        }
        let field = world.fields.get_mut(ELEVATION).expect("inserted above");
        let grid = field.grid.clone();
        let (gw, gh) = (grid.w as usize, grid.h as usize);
        ctx.pool.install(|| {
            use rayon::prelude::*;
            field
                .data
                .par_chunks_mut(gw)
                .enumerate()
                .for_each(|(j, row)| {
                    let lat = 90.0 - 180.0 * j as f64 / gh as f64;
                    for (i, v) in row.iter_mut().enumerate() {
                        let lon = 360.0 * i as f64 / gw as f64; // plan §5.2: the seam is 0°
                        *v = continent.elevation(lat, lon) as f32;
                    }
                });
        });

        let every = w.event_every_steps;
        if every > 0 && (g + 1).is_multiple_of(every) {
            let k = ((g + 1) / every - 1) as usize;
            let kind = DUMMY_KINDS[k % DUMMY_KINDS.len()];
            let mut e = world
                .event(world.t_years, kind, SOURCE)
                .at(Place::Point {
                    lat: continent.lat as f32,
                    lon: continent.lon as f32,
                })
                .magnitude((continent.peak_m / 8000.0) as f32)
                .param("stage", self.name())
                .param("step", g as i64)
                .param("peak_y", 63.0 + continent.peak_m / deeptime_format::DEFAULT_M_PER_BLOCK_V);
            if kind == "deeptime:orogeny_phase" {
                e = e.until(world.t_years + 3.0 * dt);
            }
            if let Some(prev) = world.events.last() {
                e = e.caused_by(prev.id);
            }
            world.emit(e)?;
        }

        let last_stage = self.index + 1 == self.params.stages.len();
        if last_stage && g + 1 == total {
            self.form_sites(w, world, &continent)?;
        }
        Ok(())
    }

    /// The deposit sites of the dummy world, each with its forming event.
    fn form_sites(
        &self,
        w: &DummyWorld,
        world: &mut WorldState,
        c: &Continent,
    ) -> Result<(), CoreError> {
        let cause = world.events.last().map(|e| e.id);
        let mut first = None;
        for k in 0..w.sites {
            let id = world.next_site_id();
            let lat = (c.lat + 4.0 * (f64::from(k) - f64::from(w.sites) / 2.0)).clamp(-70.0, 70.0);
            let lon = c.lon + 6.0 * f64::from(k);
            let lon = lon.rem_euclid(360.0);
            let placer = k % 2 == 1;
            let mut e = world
                .event(world.t_years, "deeptime:deposit_formation", SOURCE)
                .at(Place::Site { site: id })
                .magnitude(1.0)
                .param("deposit_type", "deeptime:dummy_porphyry")
                .param("site", i64::from(id));
            if let Some(cause) = cause {
                e = e.caused_by(cause);
            }
            let event = world.emit(e)?;
            let (x, z) = world.block_of(lat, lon);
            let depth = if placer { 60 } else { -40 };
            world.add_site(Site {
                id,
                kind: if placer {
                    SiteKind::Placer
                } else {
                    SiteKind::Deposit
                },
                deposit_type: if placer {
                    "deeptime:dummy_placer".into()
                } else {
                    "deeptime:dummy_porphyry".into()
                },
                center: [x, depth, z],
                lat_lon: [lat as f32, lon as f32],
                aabb: [[x - 24, depth - 30, z - 24], [x + 24, depth + 10, z + 24]],
                body: None,
                products: vec![Product {
                    material: "minecraft:copper".into(),
                    grade: 0.006,
                    tonnage_t: 2.0e8 / f64::from(k + 1),
                    role: "primary".into(),
                }],
                forming_event: event,
                parent: if placer { first } else { None },
                extraction: Extraction {
                    amount_base: 1000.0,
                    ..Extraction::default()
                },
            })?;
            if !placer && first.is_none() {
                first = Some(id);
            }
        }
        Ok(())
    }
}

/// The dummy world's continent at run progress `p` ∈ [0, 1]: it drifts east, wobbles in latitude
/// and rises then wears down.
struct Continent {
    lat: f64,
    lon: f64,
    /// Peak height above sea level, metres.
    peak_m: f64,
}

impl Continent {
    fn at(p: f64) -> Continent {
        Continent {
            lat: 25.0 * (2.0 * PI * p).sin(),
            lon: 30.0 + 300.0 * p,
            peak_m: 1500.0 + 6000.0 * (PI * p).sin(),
        }
    }

    /// Block y at a latitude and longitude (sea level y 63, the default metres per block up).
    fn elevation(&self, lat: f64, lon: f64) -> f64 {
        let d = angular_distance(lat, lon, self.lat, self.lon);
        let arc = angular_distance(lat, lon, -30.0, self.lon + 90.0);
        let g = (-(d / 0.55).powi(2)).exp();
        let a = (-(arc / 0.25).powi(2)).exp();
        let metres = -4000.0 + (self.peak_m + 4000.0) * g + 0.4 * (self.peak_m + 4000.0) * a;
        63.0 + metres / deeptime_format::DEFAULT_M_PER_BLOCK_V
    }
}

/// Great-circle distance in radians.
fn angular_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
    let dl = (lon2 - lon1).to_radians();
    let c = p1.sin() * p2.sin() + p1.cos() * p2.cos() * dl.cos();
    c.clamp(-1.0, 1.0).acos()
}

impl Stage for DummyStage {
    fn name(&self) -> &str {
        &self.params.stages[self.index]
    }

    fn estimate(&self) -> StageEstimate {
        let field = self
            .params
            .world
            .as_ref()
            .map_or(0, |w| Field::bytes_for(&w.grid()));
        StageEstimate {
            bytes: field + self.params.reserve_bytes,
            seconds: self.params.steps as f64 * self.params.sleep_ms as f64 / 1000.0,
            history_bytes: 0,
        }
    }

    fn total_steps(&self) -> u64 {
        self.params.steps
    }

    fn steps_done(&self) -> u64 {
        self.done
    }

    fn step(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        if self.done == 0 && self.params.reserve_bytes > 0 {
            ctx.reserve(self.params.reserve_bytes)?;
        }
        if self.params.panic_at_step == Some(self.done) {
            panic!("dummy stage: panic_at_step {} reached", self.done);
        }
        if self.params.sleep_ms > 0 {
            std::thread::sleep(Duration::from_millis(self.params.sleep_ms));
        }
        if let Some(w) = self.params.world.clone() {
            let g = self.index as u64 * self.params.steps + self.done;
            self.move_world(&w, world, g, ctx)?;
        }
        self.done += 1;
        Ok(())
    }

    fn keyframe_layers(&self) -> &[&'static str] {
        if self.params.world.is_some() {
            &[ELEVATION]
        } else {
            &[]
        }
    }

    fn save_state(&self) -> Result<Vec<u8>, CoreError> {
        to_state(&self.done)
    }

    fn load_state(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        self.done = from_state(bytes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Result<DummyParams, CoreError> {
        let v: Value = serde_json::from_str(json).unwrap();
        DummyParams::from_json(&v["dummy"])
    }

    #[test]
    fn phase_zero_form_is_unchanged() {
        let p = parse(r#"{"dummy":{"steps":5,"sleep_ms":1}}"#).unwrap();
        assert_eq!(p.stages, vec!["dummy"]);
        assert!(p.world.is_none());
        assert_eq!(p.to_json(), json!({"dummy": {"steps": 5, "sleep_ms": 1}}));
    }

    #[test]
    fn world_form_round_trips_and_names_bad_fields() {
        let json = r#"{"dummy":{"steps":4,"sleep_ms":0,"stages":["a","b_2"],
            "world":{"circumference":2048,"grid_r":16,"keyframe_size":[64,32]}}}"#;
        let p = parse(json).unwrap();
        let w = p.world.clone().unwrap();
        assert_eq!((w.grid().w, w.grid().h), (128, 64));
        assert_eq!(w.keyframe_every_years, 5.0e8 / 8.0);
        let again = DummyParams::from_json(&p.to_json()["dummy"]).unwrap();
        assert_eq!(again, p);
        let path_of = |json: &str| match parse(json) {
            Err(CoreError::PresetInvalid { path, .. }) => path,
            other => panic!("expected PresetInvalid, got {other:?}"),
        };
        assert_eq!(
            path_of(r#"{"dummy":{"steps":1,"sleep_ms":0,"stages":["A"]}}"#),
            "$.dummy.stages[0]"
        );
        assert_eq!(
            path_of(r#"{"dummy":{"steps":1,"sleep_ms":0,"stages":["a","a"]}}"#),
            "$.dummy.stages[1]"
        );
        assert_eq!(
            path_of(r#"{"dummy":{"steps":1,"sleep_ms":0,"world":{"circumference":100}}}"#),
            "$.dummy.world.circumference"
        );
        assert_eq!(
            path_of(r#"{"dummy":{"steps":1,"sleep_ms":0,"world":{"t_end_years":-1e9}}}"#),
            "$.dummy.world.t_end_years"
        );
        assert_eq!(
            path_of(r#"{"dummy":{"steps":1,"sleep_ms":0,"world":{"x":1}}}"#),
            "$.dummy.world.x"
        );
    }

    #[test]
    fn continent_is_land_at_its_centre_and_ocean_far_away() {
        let c = Continent::at(0.5);
        assert!(c.elevation(c.lat, c.lon) > 63.0 + 150.0);
        assert!(c.elevation(-c.lat, c.lon + 180.0) < 63.0);
        assert!((angular_distance(0.0, 179.0, 0.0, -179.0) - 2f64.to_radians()).abs() < 1e-9);
    }
}

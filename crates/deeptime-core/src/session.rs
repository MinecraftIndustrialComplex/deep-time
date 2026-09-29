// SPDX-License-Identifier: LGPL-3.0-or-later
//! A simulation run: the session state machine over a list of stages (plan §6.1, §8), with the
//! persistence of P1-01 — the event log, the site registry, keyframes and checkpoints.
//!
//! Threading (plan §6.2): one caller steps a session (`advance`, `checkpoint` and
//! `finalize_write` take the step mutex); `progress`, `cancel`, `keyframe_count`,
//! `keyframe_info`, `copy_keyframe` and `events_since` never block on it (atomics, or short locks
//! the stepping thread holds only to publish). Data-parallel work runs on the session's private
//! Rayon pool, which is dropped with the session. No thread here ever calls back into the JVM.
//!
//! Files: keyframes are written to `<out>/keyframes/` as they are produced; `finalize_write`
//! writes `preset.json`, `events/`, `sites/`, `keyframes/index.json` and `manifest.json` last
//! (tmp, fsync, rename; plan §6.3), so a folder with a manifest is complete.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use deeptime_format::fsutil::write_atomic;
use deeptime_format::manifest::MANIFEST;
use deeptime_format::sites::{INDEX_PATH as SITE_INDEX_PATH, SITES_PATH, write_sites};
use deeptime_format::{
    Codec, Event, EventWriter, FileEntry, GEOLOGY_WRITER, Manifest, Ramps, Scale, Writer,
};
use serde_json::{Value, json};

use crate::checkpoint::Checkpoint;
use crate::error::CoreError;
use crate::keyframes::{self, KeyframePolicy, KeyframeStore, RING_CAPACITY};
use crate::log::{Level, LogRing};
use crate::world::WorldState;

pub use crate::dummy::{DummyParams, DummyStage};
pub use crate::preset::Preset;
pub use crate::stage::{Stage, StageContext, StageEstimate};

/// Number of `long`s `progress` fills (plan §6.1).
pub const PROGRESS_FIELDS: usize = 12;

/// The default native memory cap, 4 GiB (plan §5.7.5).
pub const DEFAULT_MEM_CAP: u64 = 4 << 30;

/// The event-log writer name of the geology core (writer 1, `EVENTS.md`).
pub const GEOLOGY_WRITER_NAME: &str = "deeptime:geology";

/// `preset.json`, relative to the world-data folder.
pub const PRESET_PATH: &str = "preset.json";

/// What one `advance` call ended with. The discriminants are the `int`s Java sees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum AdvanceStatus {
    /// The time budget ran out; call `advance` again.
    Running = 0,
    /// A stage finished and another follows.
    StageDone = 1,
    /// The last stage finished; call `finalize_write`.
    Done = 2,
    /// The session was cancelled.
    Cancelled = 3,
}

/// The session's lifecycle, as reported in the first progress field. `Running`, `Done` and
/// `Cancelled` share their codes with [`AdvanceStatus`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SessionState {
    Running = 0,
    Done = 2,
    Cancelled = 3,
    Failed = 4,
    Finalized = 5,
}

impl SessionState {
    fn from_u8(v: u8) -> SessionState {
        match v {
            0 => SessionState::Running,
            2 => SessionState::Done,
            3 => SessionState::Cancelled,
            4 => SessionState::Failed,
            _ => SessionState::Finalized,
        }
    }
}

/// Native memory as the counting global allocator sees it (`deeptime-jni`, `deeptime-cli`). Tests
/// and embedders without one use [`NoGauge`].
pub trait MemoryGauge: Send + Sync {
    /// Bytes currently allocated.
    fn in_use(&self) -> u64;
    /// The highest `in_use` seen.
    fn peak(&self) -> u64;
}

/// A gauge that reports nothing allocated.
pub struct NoGauge;

impl MemoryGauge for NoGauge {
    fn in_use(&self) -> u64 {
        0
    }
    fn peak(&self) -> u64 {
        0
    }
}

/// Validates the content bundle (rock classes, deposit types, fields; plan §5.4). No schema
/// yet: empty or any valid JSON is accepted.
pub fn check_content(json: &str) -> Result<(), CoreError> {
    if json.trim().is_empty() {
        return Ok(());
    }
    serde_json::from_str::<Value>(json)
        .map(|_| ())
        .map_err(|e| CoreError::preset("content", format!("not valid JSON: {e}")))
}

/// The memory and time estimate for a preset, as JSON (plan §5.7.5):
/// `{"peak_bytes": P, "stages": [{"bytes", "name", "seconds"}, …]}`. Each stage's bytes are its
/// peak working set, shared world state included, so the run's peak is their maximum.
pub fn estimate(preset_json: &str, content_json: &str) -> Result<String, CoreError> {
    let preset = Preset::from_json(preset_json)?;
    check_content(content_json)?;
    let mut peak = 0u64;
    let stages: Vec<Value> = preset
        .stages()
        .iter()
        .map(|s| {
            let e = s.estimate();
            peak = peak.max(e.bytes);
            json!({ "name": s.name(), "bytes": e.bytes, "seconds": e.seconds })
        })
        .collect();
    Ok(json!({ "peak_bytes": peak, "stages": stages }).to_string())
}

/// How a session is created.
#[derive(Debug, Clone)]
pub struct SessionOptions {
    /// Where keyframes go while running and `finalize_write` puts the world data.
    pub out_dir: PathBuf,
    /// Rayon threads; 0 means all available processors.
    pub threads: usize,
    /// Native memory cap in bytes; 0 means [`DEFAULT_MEM_CAP`].
    pub mem_cap: u64,
}

/// A snapshot of a session's progress; `to_array` is the `long[]` layout of plan §6.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub state: SessionState,
    pub stage_index: u32,
    pub stage_count: u32,
    pub stage_permille: u32,
    pub total_permille: u32,
    /// Simulated time in years × −1 (400,000,000 at 400 Myr ago).
    pub sim_time: i64,
    pub elapsed_ms: u64,
    /// −1 while unknown.
    pub eta_ms: i64,
    pub bytes_in_use: u64,
    pub bytes_peak: u64,
    pub keyframes_ready: u32,
    pub events_total: u64,
}

impl Progress {
    pub fn to_array(&self) -> [i64; PROGRESS_FIELDS] {
        [
            self.state as i64,
            self.stage_index as i64,
            self.stage_count as i64,
            self.stage_permille as i64,
            self.total_permille as i64,
            self.sim_time,
            self.elapsed_ms as i64,
            self.eta_ms,
            self.bytes_in_use as i64,
            self.bytes_peak as i64,
            self.keyframes_ready as i64,
            self.events_total as i64,
        ]
    }
}

/// Wall-clock time a stage spent stepping in this session.
#[derive(Debug, Clone, PartialEq)]
pub struct StageTiming {
    pub name: String,
    pub steps: u64,
    pub millis: f64,
}

struct Stepper {
    stages: Vec<Box<dyn Stage>>,
    current: usize,
    world: WorldState,
    /// Keyframe times are `kf_origin + k · every_years`.
    kf_origin: f64,
    next_kf: f64,
    last_kf_t: Option<f64>,
    timings: Vec<StageTiming>,
}

const NOT_FINISHED: u64 = u64::MAX;

/// One simulation run.
pub struct Session {
    preset: Preset,
    preset_json: String,
    content_json: String,
    stage_names: Vec<String>,
    out_dir: PathBuf,
    mem_cap: u64,
    threads: usize,
    pool: rayon::ThreadPool,
    gauge: Arc<dyn MemoryGauge>,
    log: LogRing,
    state: AtomicU8,
    stage_index: AtomicU32,
    stage_count: u32,
    stage_permille: AtomicU32,
    sim_time: AtomicI64,
    created: Instant,
    finished_ms: AtomicU64,
    stepper: Mutex<Stepper>,
    /// Events visible to `events_since`, in emission order.
    published: Mutex<Vec<Event>>,
    events_total: AtomicU64,
    keyframes: KeyframeStore,
    policy: KeyframePolicy,
    has_keyframes: bool,
    ramps: Ramps,
    checkpoint_dir: Option<PathBuf>,
}

impl Session {
    /// Parses the preset and content, builds the private Rayon pool and returns a session ready to
    /// `advance`.
    pub fn create(
        preset_json: &str,
        content_json: &str,
        options: SessionOptions,
        gauge: Arc<dyn MemoryGauge>,
    ) -> Result<Session, CoreError> {
        let preset = Preset::from_json(preset_json)?;
        check_content(content_json)?;
        let world = preset.initial_world();
        Session::build(preset, content_json, options, gauge, world, 0, &[])
    }

    /// Continues a run from a checkpoint (`deeptime stage`). The checkpoint's preset must still
    /// build the same pipeline.
    pub fn resume(
        checkpoint: Checkpoint,
        options: SessionOptions,
        gauge: Arc<dyn MemoryGauge>,
    ) -> Result<Session, CoreError> {
        let preset = Preset::from_json(&checkpoint.preset_json)?;
        check_content(&checkpoint.content_json)?;
        let names: Vec<String> = preset
            .stages()
            .iter()
            .map(|s| s.name().to_owned())
            .collect();
        if names != checkpoint.stage_names {
            return Err(CoreError::InvalidState(format!(
                "the checkpoint's pipeline {:?} is not the preset's {names:?}",
                checkpoint.stage_names
            )));
        }
        if checkpoint.next_stage as usize > names.len() {
            return Err(CoreError::InvalidState(format!(
                "the checkpoint's next stage {} is past the pipeline's {} stages",
                checkpoint.next_stage,
                names.len()
            )));
        }
        Session::build(
            preset,
            &checkpoint.content_json,
            options,
            gauge,
            checkpoint.world,
            checkpoint.next_stage as usize,
            &checkpoint.stage_state,
        )
    }

    fn build(
        preset: Preset,
        content_json: &str,
        options: SessionOptions,
        gauge: Arc<dyn MemoryGauge>,
        world: WorldState,
        next_stage: usize,
        stage_state: &[u8],
    ) -> Result<Session, CoreError> {
        let threads = if options.threads == 0 {
            std::thread::available_parallelism().map_or(1, |n| n.get())
        } else {
            options.threads
        };
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .thread_name(|i| format!("deeptime-rayon-{i}"))
            .build()
            .map_err(|e| CoreError::InvalidState(format!("cannot start the Rayon pool: {e}")))?;
        let mem_cap = if options.mem_cap == 0 {
            DEFAULT_MEM_CAP
        } else {
            options.mem_cap
        };
        let mut stages = preset.stages();
        if !stage_state.is_empty()
            && let Some(stage) = stages.get_mut(next_stage)
        {
            stage.load_state(stage_state)?;
        }
        let stage_names: Vec<String> = stages.iter().map(|s| s.name().to_owned()).collect();
        let timings = stage_names
            .iter()
            .map(|n| StageTiming {
                name: n.clone(),
                steps: 0,
                millis: 0.0,
            })
            .collect();
        let policy = preset.keyframe_policy();
        let has_keyframes =
            policy.every_years.is_some() && stages.iter().any(|s| !s.keyframe_layers().is_empty());
        let preset_json = serde_json::to_string(&preset.to_json())
            .map_err(|e| CoreError::InvalidState(format!("preset serialisation: {e}")))?;
        let events_total = world.events.len() as u64;
        let sim_time = -world.t_years.round() as i64;
        // Keyframe times count from the run's start, so a resumed run keeps the same grid: a
        // fresh run takes its first frame after its first step, a resumed one at the next
        // multiple of the interval.
        let kf_origin = preset.initial_world().t_years;
        let next_kf = match policy.every_years {
            Some(every) if world.t_years > kf_origin => {
                kf_origin + (((world.t_years - kf_origin) / every).floor() + 1.0) * every
            }
            _ => kf_origin,
        };
        let session = Session {
            stage_count: stages.len() as u32,
            stage_names,
            preset_json,
            content_json: content_json.to_owned(),
            keyframes: KeyframeStore::new(&options.out_dir, RING_CAPACITY),
            out_dir: options.out_dir,
            preset,
            mem_cap,
            threads,
            pool,
            gauge,
            log: LogRing::default(),
            state: AtomicU8::new(SessionState::Running as u8),
            stage_index: AtomicU32::new(next_stage as u32),
            stage_permille: AtomicU32::new(0),
            sim_time: AtomicI64::new(sim_time),
            created: Instant::now(),
            finished_ms: AtomicU64::new(NOT_FINISHED),
            published: Mutex::new(world.events.clone()),
            events_total: AtomicU64::new(events_total),
            stepper: Mutex::new(Stepper {
                stages,
                current: next_stage,
                world,
                kf_origin,
                next_kf,
                last_kf_t: None,
                timings,
            }),
            policy,
            has_keyframes,
            ramps: Ramps::builtin(),
            checkpoint_dir: None,
        };
        session.log.push(
            Level::Info,
            format!(
                "session created: preset {} threads={} mem_cap={} out_dir={} next_stage={}",
                session.preset_json,
                threads,
                mem_cap,
                session.out_dir.display(),
                next_stage
            ),
        );
        Ok(session)
    }

    /// Writes a checkpoint `<dir>/<index>-<stage>.dtcp` after every finished stage (dev and CLI;
    /// plan §5.7.1 `checkpoints/`).
    pub fn set_checkpoint_dir(&mut self, dir: Option<PathBuf>) {
        self.checkpoint_dir = dir;
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    pub fn log(&self) -> &LogRing {
        &self.log
    }

    pub fn preset(&self) -> &Preset {
        &self.preset
    }

    /// The stage names of the pipeline, in order.
    pub fn stage_names(&self) -> &[String] {
        &self.stage_names
    }

    pub fn out_dir(&self) -> &Path {
        &self.out_dir
    }

    pub fn state(&self) -> SessionState {
        SessionState::from_u8(self.state.load(Ordering::Acquire))
    }

    /// Moves `Running` to `to`; false if the session had already left `Running`.
    fn leave_running(&self, to: SessionState) -> bool {
        let moved = self
            .state
            .compare_exchange(
                SessionState::Running as u8,
                to as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok();
        if moved {
            self.finished_ms
                .store(self.created.elapsed().as_millis() as u64, Ordering::Release);
        }
        moved
    }

    fn lock_stepper(&self) -> Result<std::sync::MutexGuard<'_, Stepper>, CoreError> {
        self.stepper.lock().map_err(|_| CoreError::Poisoned)
    }

    fn fail(&self, what: &str, e: CoreError) -> CoreError {
        self.log.push(Level::Error, format!("{what} failed: {e}"));
        self.leave_running(SessionState::Failed);
        e
    }

    /// Runs stages for about `budget` (at least one step), then reports where the run stands.
    pub fn advance(&self, budget: Duration) -> Result<AdvanceStatus, CoreError> {
        let mut guard = self.lock_stepper()?;
        let s = &mut *guard;
        let start = Instant::now();
        let until = start
            .checked_add(budget)
            .unwrap_or_else(|| start + Duration::from_secs(365 * 86_400));
        loop {
            match self.state() {
                SessionState::Running => {}
                SessionState::Done | SessionState::Finalized => return Ok(AdvanceStatus::Done),
                SessionState::Cancelled => return Ok(AdvanceStatus::Cancelled),
                SessionState::Failed => {
                    return Err(CoreError::InvalidState(
                        "the session failed earlier; destroy it".into(),
                    ));
                }
            }
            let index = s.current;
            if index >= s.stages.len() {
                return Ok(self.complete());
            }
            if s.stages[index].is_done() {
                let name = s.stages[index].name().to_owned();
                self.finish_stage(s)
                    .map_err(|e| self.fail(&format!("finishing stage {name}"), e))?;
                self.stage_permille.store(0, Ordering::Release);
                if s.current >= s.stages.len() {
                    return Ok(self.complete());
                }
                self.stage_index.store(s.current as u32, Ordering::Release);
                return Ok(AdvanceStatus::StageDone);
            }
            let stage = &mut s.stages[index];
            if stage.steps_done() == 0 {
                self.log
                    .push(Level::Info, format!("stage {} started", stage.name()));
            }
            let ctx = StageContext {
                pool: &self.pool,
                log: &self.log,
                until,
                gauge: self.gauge.as_ref(),
                mem_cap: self.mem_cap,
                state: &self.state,
            };
            let stepped = Instant::now();
            if let Err(e) = stage.step(&mut s.world, &ctx) {
                let what = format!("stage {}", stage.name());
                return Err(self.fail(&what, e));
            }
            s.timings[index].steps = stage.steps_done();
            self.stage_permille.store(
                permille(stage.steps_done(), stage.total_steps()),
                Ordering::Release,
            );
            self.after_step(s)
                .map_err(|e| self.fail("keyframe or event publishing", e))?;
            // A stage's time includes publishing its events and drawing its keyframes.
            s.timings[index].millis += stepped.elapsed().as_secs_f64() * 1000.0;
            if start.elapsed() >= budget {
                return Ok(AdvanceStatus::Running);
            }
        }
    }

    /// Publishes new events and the clock; takes a keyframe when the clock passed the next one.
    fn after_step(&self, s: &mut Stepper) -> Result<(), CoreError> {
        self.publish(&s.world);
        if let Some(every) = self.policy.every_years
            && s.world.t_years >= s.next_kf
        {
            let layers = s.stages[s.current].keyframe_layers().to_vec();
            if !layers.is_empty() {
                let name = s.stages[s.current].name().to_owned();
                self.take_keyframe(s, &name, &layers)?;
                let k = ((s.world.t_years - s.kf_origin) / every).floor() + 1.0;
                s.next_kf = s.kf_origin + k * every;
            }
        }
        Ok(())
    }

    fn publish(&self, world: &WorldState) {
        self.sim_time
            .store(-world.t_years.round() as i64, Ordering::Release);
        let mut published = self.published.lock().unwrap_or_else(|e| e.into_inner());
        if world.events.len() > published.len() {
            let from = published.len();
            published.extend_from_slice(&world.events[from..]);
            self.events_total
                .store(published.len() as u64, Ordering::Release);
        }
    }

    fn take_keyframe(
        &self,
        s: &mut Stepper,
        stage: &str,
        layers: &[&str],
    ) -> Result<(), CoreError> {
        if s.last_kf_t == Some(s.world.t_years) {
            return Ok(());
        }
        let n = self.keyframes.next_n();
        if let Some(frame) =
            keyframes::render(&s.world, &self.ramps, stage, layers, n, self.policy.size)
        {
            self.keyframes.push(frame)?;
            s.last_kf_t = Some(s.world.t_years);
            self.log.push(
                Level::Debug,
                format!("keyframe {n} at t={} ({stage})", s.world.t_years),
            );
        }
        Ok(())
    }

    /// The current stage just finished: a closing keyframe, a checkpoint if asked for, next stage.
    fn finish_stage(&self, s: &mut Stepper) -> Result<(), CoreError> {
        let index = s.current;
        let name = s.stages[index].name().to_owned();
        self.log.push(Level::Info, format!("stage {name} finished"));
        let started = Instant::now();
        self.publish(&s.world);
        if self.policy.every_years.is_some() {
            let layers = s.stages[index].keyframe_layers().to_vec();
            if !layers.is_empty() {
                self.take_keyframe(s, &name, &layers)?;
            }
        }
        s.timings[index].millis += started.elapsed().as_secs_f64() * 1000.0;
        s.current += 1;
        if let Some(dir) = &self.checkpoint_dir {
            let path = dir.join(format!("{index}-{name}.dtcp"));
            self.checkpoint_of(s)?.write(&path)?;
            self.log.push(
                Level::Info,
                format!("checkpoint written to {}", path.display()),
            );
        }
        Ok(())
    }

    fn complete(&self) -> AdvanceStatus {
        self.stage_index.store(self.stage_count, Ordering::Release);
        if self.leave_running(SessionState::Done) {
            self.log.push(Level::Info, "run done");
            AdvanceStatus::Done
        } else if self.state() == SessionState::Cancelled {
            AdvanceStatus::Cancelled
        } else {
            AdvanceStatus::Done
        }
    }

    /// Requests cancellation. Callable from any thread; never blocks on a running `advance`.
    pub fn cancel(&self) {
        if self.leave_running(SessionState::Cancelled) {
            self.log.push(Level::Info, "session cancelled");
        }
    }

    pub fn progress(&self) -> Progress {
        let state = self.state();
        let stage_count = self.stage_count;
        let stage_index = self.stage_index.load(Ordering::Acquire).min(stage_count);
        let stage_permille = self.stage_permille.load(Ordering::Acquire);
        let total_permille = match state {
            SessionState::Done | SessionState::Finalized => 1000,
            _ if stage_count == 0 => 0,
            _ => (stage_index * 1000 + stage_permille) / stage_count,
        };
        let finished = self.finished_ms.load(Ordering::Acquire);
        let elapsed_ms = if finished == NOT_FINISHED {
            self.created.elapsed().as_millis() as u64
        } else {
            finished
        };
        let eta_ms = match state {
            SessionState::Running if total_permille > 0 => {
                (elapsed_ms as i64) * (1000 - total_permille as i64) / total_permille as i64
            }
            SessionState::Running => -1,
            _ => 0,
        };
        Progress {
            state,
            stage_index,
            stage_count,
            stage_permille,
            total_permille,
            sim_time: self.sim_time.load(Ordering::Acquire),
            elapsed_ms,
            eta_ms,
            bytes_in_use: self.gauge.in_use(),
            bytes_peak: self.gauge.peak(),
            keyframes_ready: self.keyframes.count(),
            events_total: self.events_total.load(Ordering::Acquire),
        }
    }

    /// Keyframes produced so far.
    pub fn keyframe_count(&self) -> u32 {
        self.keyframes.count()
    }

    fn require_keyframes(&self, what: &'static str) -> Result<(), CoreError> {
        if self.has_keyframes {
            Ok(())
        } else {
            Err(CoreError::Unsupported(what))
        }
    }

    /// `keyframeInfo`: JSON `{"n", "t", "stage", "w", "h", "layers"}` (plan §6.1).
    /// `Unsupported` when the preset produces no keyframes at all; `InvalidArgument` for an index
    /// not produced yet.
    pub fn keyframe_info(&self, index: u32) -> Result<String, CoreError> {
        self.require_keyframes("keyframeInfo (this preset produces no keyframes)")?;
        self.keyframes.info_json(index)
    }

    /// `copyKeyframe`: layer `layer` of keyframe `index` as RGBA8 into `dst` (≥ w·h·4 bytes).
    pub fn copy_keyframe(&self, index: u32, layer: u32, dst: &mut [u8]) -> Result<(), CoreError> {
        self.require_keyframes("copyKeyframe (this preset produces no keyframes)")?;
        self.keyframes.copy(index, layer, dst)
    }

    /// Events after `cursor` as a JSON array (`EVENTS.md`, "JSON"), at most `max`. The cursor
    /// counts events in emission order: pass the number already received.
    pub fn events_since(&self, cursor: u64, max: u32) -> String {
        let published = self.published.lock().unwrap_or_else(|e| e.into_inner());
        let from = usize::try_from(cursor)
            .unwrap_or(usize::MAX)
            .min(published.len());
        let to = from.saturating_add(max as usize).min(published.len());
        Value::Array(published[from..to].iter().map(Event::to_json).collect()).to_string()
    }

    /// How long each stage has stepped in this session.
    pub fn stage_timings(&self) -> Result<Vec<StageTiming>, CoreError> {
        Ok(self.lock_stepper()?.timings.clone())
    }

    fn checkpoint_of(&self, s: &Stepper) -> Result<Checkpoint, CoreError> {
        // A stage that has stepped keeps its state, even after its last step (the session moves
        // on to the next stage only on the following `advance`): a resume must not rerun it.
        let stage_state = match s.stages.get(s.current) {
            Some(stage) if stage.steps_done() > 0 => stage.save_state()?,
            _ => Vec::new(),
        };
        Ok(Checkpoint {
            preset_json: self.preset_json.clone(),
            content_json: self.content_json.clone(),
            stage_names: self.stage_names.clone(),
            next_stage: s.current as u32,
            stage_state,
            world: s.world.clone(),
        })
    }

    /// The run as it stands, for `deeptime stage` (between `advance` calls).
    pub fn checkpoint(&self) -> Result<Checkpoint, CoreError> {
        let s = self.lock_stepper()?;
        self.checkpoint_of(&s)
    }

    /// After `Done`: writes the world data and `manifest.json` last (tmp, fsync, rename; plan
    /// §6.3), and returns the manifest.
    pub fn finalize_write(&self) -> Result<String, CoreError> {
        let s = self.lock_stepper()?;
        match self.state() {
            SessionState::Done | SessionState::Finalized => {}
            SessionState::Cancelled => return Err(CoreError::Cancelled),
            SessionState::Running => {
                return Err(CoreError::InvalidState(
                    "finalizeWrite before the run is done".into(),
                ));
            }
            SessionState::Failed => {
                return Err(CoreError::InvalidState(
                    "finalizeWrite after the run failed".into(),
                ));
            }
        }
        let dir = &self.out_dir;
        let manifest_path = dir.join(MANIFEST);
        match std::fs::remove_file(&manifest_path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(CoreError::io(manifest_path, e)),
        }
        let world = &s.world;
        let c = world.circumference;
        let mut m = Manifest::new(
            &format!("run:{}", self.preset.name()),
            Writer::this("deeptime-core"),
            Scale::default_for(c),
        );
        m.preset_hash = Some(format!(
            "crc32:{:08x}",
            crc32fast::hash(self.preset_json.as_bytes())
        ));
        m.planet.insert(
            "radius_m".into(),
            json!(f64::from(c) * 300.0 / (2.0 * std::f64::consts::PI)),
        );

        // A placeholder preset's terrain (P1-13): the synthetic layers P0-09's `synth` writes,
        // until the bake of P1-08 exists.
        if let Some(kind) = self.preset.synth() {
            let started = Instant::now();
            let baked =
                crate::synth::build(kind, crate::synth::Strata::Placeholder, c, Codec::Lz4Block)?;
            let bytes = baked.write_into(dir, &mut m)?;
            self.log.push(
                Level::Info,
                format!(
                    "baked the synthetic terrain {} (C = {c}): {} layers, {:.1} MiB in {} ms",
                    kind.name(),
                    m.layers.len(),
                    bytes as f64 / 1_048_576.0,
                    started.elapsed().as_millis()
                ),
            );
        }
        let mut files = std::mem::take(&mut m.files);

        let mut preset = serde_json::to_vec_pretty(&self.preset.to_json())
            .map_err(|e| CoreError::InvalidState(format!("preset serialisation: {e}")))?;
        preset.push(b'\n');
        let crc32 = write_atomic(&dir.join(PRESET_PATH), &preset)?;
        files.push(FileEntry {
            path: PRESET_PATH.into(),
            bytes: preset.len() as u64,
            crc32,
        });

        let mut events = EventWriter::open(dir, GEOLOGY_WRITER, GEOLOGY_WRITER_NAME)?;
        events.clear()?;
        events.append(&world.events)?;
        for seg in &events.index().segments {
            files.push(FileEntry {
                path: seg.path.clone(),
                bytes: seg.bytes,
                crc32: seg.crc32,
            });
        }
        files.push(FileEntry::of(dir, deeptime_format::events::INDEX_PATH)?);

        let w = write_sites(dir, world.circumference, &world.sites)?;
        files.push(FileEntry {
            path: SITES_PATH.into(),
            bytes: w.sites_bytes,
            crc32: w.sites_crc32,
        });
        files.push(FileEntry {
            path: SITE_INDEX_PATH.into(),
            bytes: w.index_bytes,
            crc32: w.index_crc32,
        });

        let kf = self.keyframes.finish()?;
        files.push(FileEntry::of(dir, deeptime_format::keyframes::INDEX_PATH)?);
        for f in &kf.frames {
            files.push(FileEntry {
                path: f.path.clone(),
                bytes: f.bytes,
                crc32: f.crc32,
            });
        }

        m.files = files;
        let mut run = json!({
            "stages": s.timings.iter().map(|t| json!({
                "name": t.name, "steps": t.steps, "millis": t.millis,
            })).collect::<Vec<_>>(),
            "t_years": world.t_years,
            "events": world.events.len(),
            "sites": world.sites.len(),
            "keyframes": kf.frames.len(),
        });
        let header = &self.preset.header;
        if let Some(t) = header.timeline {
            run["timeline"] = json!({ "start": t.start.name(), "length_myr": t.length_myr });
        }
        if header.placeholder {
            run["placeholder"] = json!(true);
        }
        m.notes.insert("run".into(), run);
        let bytes = m.to_json_bytes()?;
        write_atomic(&dir.join(MANIFEST), &bytes)?;
        self.state
            .store(SessionState::Finalized as u8, Ordering::Release);
        self.log.push(
            Level::Info,
            format!("manifest written to {}", self.out_dir.display()),
        );
        String::from_utf8(bytes)
            .map_err(|e| CoreError::InvalidState(format!("manifest is not UTF-8: {e}")))
    }
}

fn permille(done: u64, total: u64) -> u32 {
    if total == 0 {
        1000
    } else {
        (done.min(total) as u128 * 1000 / total as u128) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::{EventLog, KeyframeIndex};

    /// The preset of the P1-01 acceptance run (`deeptime run --preset …/dummy.json`).
    const DUMMY_WORLD: &str = include_str!("../tests/presets/dummy.json");

    fn options(dir: &Path) -> SessionOptions {
        SessionOptions {
            out_dir: dir.to_path_buf(),
            threads: 2,
            mem_cap: 0,
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "deeptime-core-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn run_to_done(s: &Session) -> Vec<AdvanceStatus> {
        let mut seen = Vec::new();
        for _ in 0..10_000 {
            let st = s.advance(Duration::from_millis(2)).unwrap();
            seen.push(st);
            if st == AdvanceStatus::Done {
                return seen;
            }
        }
        panic!("the run never finished");
    }

    /// A placeholder preset (P1-13): the dummy run plus a baked synthetic terrain, with the
    /// header keys and a timeline, is one complete, verifiable world-data folder.
    #[test]
    fn placeholder_preset_bakes_terrain_next_to_the_run() {
        let dir = temp_dir("synth");
        let preset = r#"{"name":"tiny","placeholder":true,"scale":{"circumference_blocks":1024},
            "timeline":{"start":"supercontinent","length_myr":200},
            "dummy":{"steps":4,"sleep_ms":0,"stages":["dummy_drift","dummy_erode"],"synth":"wrap-test",
                     "world":{"circumference":1024,"grid_r":16,"keyframe_size":[32,16],"sites":2}}}"#;
        let s = Session::create(preset, "", options(&dir), Arc::new(NoGauge)).unwrap();
        run_to_done(&s);
        s.finalize_write().unwrap();
        let m = Manifest::read(&dir).unwrap();
        m.verify(&dir).unwrap();
        assert_eq!(m.source, "run:tiny");
        assert_eq!(m.scale.circumference_blocks, 1024);
        assert_eq!(
            m.layers.len(),
            8,
            "elevation, water_level, 5 climate, strata"
        );
        assert!(m.layer("elevation").is_some() && m.layer("strata/horizons").is_some());
        for f in [
            "spawn/candidates.json",
            "strata/classes.json",
            PRESET_PATH,
            SITES_PATH,
        ] {
            assert!(m.files.iter().any(|e| e.path == f), "{f} listed");
        }
        assert_eq!(m.notes["synthetic"]["kind"], "wrap-test");
        assert_eq!(m.notes["run"]["timeline"]["start"], "supercontinent");
        assert_eq!(m.notes["run"]["placeholder"], true);
        // The timeline set the clock: 200 Myr ago to now.
        let written: Value =
            serde_json::from_slice(&std::fs::read(dir.join(PRESET_PATH)).unwrap()).unwrap();
        assert_eq!(written["dummy"]["world"]["t_start_years"], -2.0e8);
        assert_eq!(written["timeline"]["length_myr"], 200.0);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn dummy_runs_to_done_and_writes_manifest() {
        let dir = temp_dir("done");
        let s = Session::create(
            r#"{"dummy":{"steps":5,"sleep_ms":1}}"#,
            "",
            options(&dir),
            Arc::new(NoGauge),
        )
        .unwrap();
        assert_eq!(s.threads(), 2);
        run_to_done(&s);
        let p = s.progress();
        assert_eq!(p.state, SessionState::Done);
        assert_eq!(p.total_permille, 1000);
        assert_eq!(p.stage_index, 1);
        assert_eq!(p.stage_count, 1);
        assert_eq!((p.keyframes_ready, p.events_total), (0, 0));
        assert_eq!(s.keyframe_count(), 0);
        assert_eq!(s.events_since(0, 10), "[]");
        assert!(matches!(s.keyframe_info(0), Err(CoreError::Unsupported(_))));
        let manifest = s.finalize_write().unwrap();
        let v: Value = serde_json::from_str(&manifest).unwrap();
        assert_eq!(v["format_version"], "0.0");
        assert_eq!(v["layers"], json!([]));
        assert!(manifest.contains("\"format_version\": \"0.0\""));
        assert!(manifest.contains("\"layers\": []"));
        let on_disk = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
        assert_eq!(on_disk, manifest);
        assert!(!dir.join("manifest.json.tmp").exists());
        assert_eq!(s.progress().state, SessionState::Finalized);
        // Even an empty run leaves a complete, verifiable folder.
        let m = Manifest::read(&dir).unwrap();
        m.verify(&dir).unwrap();
        assert!(EventLog::load(&dir).unwrap().is_empty());
        assert!(KeyframeIndex::read(&dir).unwrap().frames.is_empty());
        let lines = s.log().drain();
        assert!(lines.iter().any(|l| l.contains("stage dummy finished")));
        assert!(lines.iter().any(|l| l.contains("manifest written")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dummy_world_writes_events_sites_and_keyframes() {
        let dir = temp_dir("world");
        let s = Session::create(DUMMY_WORLD, "", options(&dir), Arc::new(NoGauge)).unwrap();
        assert_eq!(s.stage_names(), ["dummy_drift", "dummy_erode"]);
        assert!(matches!(
            s.keyframe_info(0),
            Err(CoreError::InvalidArgument(_))
        ));
        let statuses = run_to_done(&s);
        assert!(statuses.contains(&AdvanceStatus::StageDone));
        let p = s.progress();
        assert_eq!(p.sim_time, 0, "the clock ends at t_end_years = 0");
        assert_eq!(p.events_total, 13, "10 chain events + 3 deposit formations");
        assert_eq!(p.keyframes_ready, 9);
        let info: Value = serde_json::from_str(&s.keyframe_info(0).unwrap()).unwrap();
        assert_eq!(info["t"], -3.9e8);
        assert_eq!(
            (info["w"].as_u64(), info["h"].as_u64()),
            (Some(256), Some(128))
        );
        let mut rgba = vec![0u8; 256 * 128 * 4];
        s.copy_keyframe(8, 0, &mut rgba).unwrap();
        assert!(rgba.chunks(4).all(|px| px[3] == 255));

        // eventsSince pages through the log in emission order.
        let first: Vec<Value> = serde_json::from_str(&s.events_since(0, 4)).unwrap();
        let rest: Vec<Value> = serde_json::from_str(&s.events_since(4, 100)).unwrap();
        assert_eq!((first.len(), rest.len()), (4, 9));
        assert_eq!(first[1]["causes"], json!([first[0]["id"]]));
        assert_eq!(s.events_since(13, 5), "[]");
        assert_eq!(s.events_since(u64::MAX, 5), "[]");

        let manifest: Value = serde_json::from_str(&s.finalize_write().unwrap()).unwrap();
        assert_eq!(manifest["source"], "run:dummy");
        for f in [
            "events/1-0.dtev",
            "events/index.json",
            "keyframes/index.json",
        ] {
            assert!(dir.join(f).is_file(), "{f}");
        }
        Manifest::read(&dir).unwrap().verify(&dir).unwrap();
        let log = EventLog::load(&dir).unwrap();
        assert_eq!(log.len(), 13);
        let (sites, index) = deeptime_format::sites::read_sites(&dir).unwrap().unwrap();
        assert_eq!(sites.len(), 3);
        let site = &sites[1];
        assert_eq!(
            site.parent,
            Some(sites[0].id),
            "the placer names its source"
        );
        let forming = log.get(site.forming_event).unwrap();
        assert_eq!(forming.kind, "deeptime:deposit_formation");
        assert_eq!(
            log.cause_chain(forming.id, 3).len(),
            3,
            "Why here? follows the chain"
        );
        assert!(
            index
                .at_block(site.center[0], site.center[2])
                .contains(&site.id)
        );
        assert_eq!(KeyframeIndex::read(&dir).unwrap().frames.len(), 9);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A run stopped at every stage boundary (and once inside a stage), checkpointed through
    /// bytes and resumed, ends in exactly the state of a straight run.
    #[test]
    fn checkpoints_resume_to_the_same_world() {
        let straight = Session::create(
            DUMMY_WORLD,
            "",
            options(&temp_dir("straight")),
            Arc::new(NoGauge),
        )
        .unwrap();
        run_to_done(&straight);
        let expect = straight.checkpoint().unwrap().world;

        let dir = temp_dir("resumed");
        let mut s = Session::create(DUMMY_WORLD, "", options(&dir), Arc::new(NoGauge)).unwrap();
        // Inside the first stage: one step, then a mid-stage checkpoint.
        assert_eq!(s.advance(Duration::ZERO).unwrap(), AdvanceStatus::Running);
        let mut ck = s.checkpoint().unwrap();
        assert_eq!(ck.next_stage, 0);
        assert!(!ck.stage_state.is_empty(), "mid-stage state is saved");
        loop {
            let bytes = ck.to_bytes().unwrap();
            ck = Checkpoint::from_bytes(Path::new("ck"), &bytes).unwrap();
            s = Session::resume(ck, options(&dir), Arc::new(NoGauge)).unwrap();
            let status = loop {
                match s.advance(Duration::from_millis(50)).unwrap() {
                    AdvanceStatus::Running => continue,
                    other => break other,
                }
            };
            ck = s.checkpoint().unwrap();
            if status == AdvanceStatus::Done {
                break;
            }
            assert!(
                ck.stage_state.is_empty(),
                "a boundary checkpoint has no stage state"
            );
        }
        assert_eq!(ck.next_stage, 2);
        assert_eq!(ck.world, expect);

        // Right after a stage's last step, before the session moved on: the resume finishes that
        // stage instead of running it again.
        let edge = temp_dir("edge");
        let s = Session::create(DUMMY_WORLD, "", options(&edge), Arc::new(NoGauge)).unwrap();
        for _ in 0..20 {
            assert_eq!(s.advance(Duration::ZERO).unwrap(), AdvanceStatus::Running);
        }
        let ck = s.checkpoint().unwrap();
        assert_eq!(ck.next_stage, 0);
        assert!(!ck.stage_state.is_empty());
        let s = Session::resume(ck, options(&edge), Arc::new(NoGauge)).unwrap();
        assert_eq!(s.advance(Duration::ZERO).unwrap(), AdvanceStatus::StageDone);
        run_to_done(&s);
        assert_eq!(s.checkpoint().unwrap().world, expect);
        for d in [dir, edge] {
            let _ = std::fs::remove_dir_all(d);
        }
    }

    #[test]
    fn checkpoint_dir_gets_one_file_per_stage() {
        let dir = temp_dir("ckdir");
        let mut s = Session::create(DUMMY_WORLD, "", options(&dir), Arc::new(NoGauge)).unwrap();
        s.set_checkpoint_dir(Some(dir.join("checkpoints")));
        run_to_done(&s);
        let ck = Checkpoint::read(&dir.join("checkpoints/0-dummy_drift.dtcp")).unwrap();
        assert_eq!(ck.next_stage_name(), Some("dummy_erode"));
        assert_eq!(ck.world.t_years, -2.0e8);
        assert!(dir.join("checkpoints/1-dummy_erode.dtcp").is_file());
        let timings = s.stage_timings().unwrap();
        assert_eq!(
            timings.iter().map(|t| t.steps).collect::<Vec<_>>(),
            vec![20, 20]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cancel_stops_the_run() {
        let dir = temp_dir("cancel");
        let s = Session::create(
            r#"{"dummy":{"steps":1000,"sleep_ms":1}}"#,
            "{}",
            options(&dir),
            Arc::new(NoGauge),
        )
        .unwrap();
        assert_eq!(
            s.advance(Duration::from_millis(5)).unwrap(),
            AdvanceStatus::Running
        );
        s.cancel();
        assert_eq!(s.progress().state, SessionState::Cancelled);
        assert_eq!(
            s.advance(Duration::from_millis(5)).unwrap(),
            AdvanceStatus::Cancelled
        );
        assert!(matches!(s.finalize_write(), Err(CoreError::Cancelled)));
        assert!(!dir.join("manifest.json").exists());
    }

    #[test]
    fn invalid_presets_name_the_field() {
        let path_of = |json: &str| match Preset::from_json(json) {
            Err(CoreError::PresetInvalid { path, .. }) => path,
            other => panic!("expected PresetInvalid, got {other:?}"),
        };
        assert_eq!(path_of("not json"), "$");
        assert_eq!(path_of("{}"), "$");
        assert_eq!(path_of(r#"{"planet":{}}"#), "$.planet");
        assert_eq!(path_of(r#"{"dummy":{"sleep_ms":1}}"#), "$.dummy.steps");
        assert_eq!(
            path_of(r#"{"dummy":{"steps":-1,"sleep_ms":1}}"#),
            "$.dummy.steps"
        );
        assert_eq!(
            path_of(r#"{"dummy":{"steps":1,"sleep_ms":1,"x":2}}"#),
            "$.dummy.x"
        );
        assert!(matches!(
            check_content("{"),
            Err(CoreError::PresetInvalid { .. })
        ));
        assert_eq!(
            estimate(r#"{"dummy":{"steps":1,"sleep_ms":0}}"#, "").unwrap(),
            r#"{"peak_bytes":0,"stages":[{"bytes":0,"name":"dummy","seconds":0.0}]}"#
        );
        let world: Value = serde_json::from_str(&estimate(DUMMY_WORLD, "").unwrap()).unwrap();
        assert_eq!(world["peak_bytes"], 256 * 128 * 4);
        assert_eq!(world["stages"][1]["name"], "dummy_erode");
    }

    struct FixedGauge(u64);
    impl MemoryGauge for FixedGauge {
        fn in_use(&self) -> u64 {
            self.0
        }
        fn peak(&self) -> u64 {
            self.0
        }
    }

    #[test]
    fn memory_cap_fails_the_stage_before_allocating() {
        let s = Session::create(
            r#"{"dummy":{"steps":3,"sleep_ms":0,"reserve_bytes":600}}"#,
            "",
            SessionOptions {
                out_dir: temp_dir("cap"),
                threads: 1,
                mem_cap: 1000,
            },
            Arc::new(FixedGauge(500)),
        )
        .unwrap();
        match s.advance(Duration::from_millis(5)) {
            Err(CoreError::MemoryCap { needed, cap }) => {
                assert_eq!((needed, cap), (1100, 1000));
            }
            other => panic!("expected MemoryCap, got {other:?}"),
        }
        assert_eq!(s.progress().state, SessionState::Failed);
        assert!(matches!(
            s.advance(Duration::ZERO),
            Err(CoreError::InvalidState(_))
        ));
    }

    #[test]
    fn a_panicking_stage_poisons_the_session() {
        let s = Session::create(
            r#"{"dummy":{"steps":3,"sleep_ms":0,"panic_at_step":1}}"#,
            "",
            options(&temp_dir("panic")),
            Arc::new(NoGauge),
        )
        .unwrap();
        let caught =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| s.advance(Duration::MAX)));
        assert!(caught.is_err());
        assert!(matches!(
            s.advance(Duration::ZERO),
            Err(CoreError::Poisoned)
        ));
        // progress, cancel and the event/keyframe readers never take the step lock.
        let _ = s.progress();
        assert_eq!(s.events_since(0, 1), "[]");
        s.cancel();
    }

    #[test]
    fn zero_steps_is_done_at_once() {
        let s = Session::create(
            r#"{"dummy":{"steps":0,"sleep_ms":0}}"#,
            "",
            options(&temp_dir("zero")),
            Arc::new(NoGauge),
        )
        .unwrap();
        assert_eq!(s.advance(Duration::ZERO).unwrap(), AdvanceStatus::Done);
    }

    #[test]
    fn progress_array_has_the_documented_layout() {
        let p = Progress {
            state: SessionState::Running,
            stage_index: 1,
            stage_count: 2,
            stage_permille: 3,
            total_permille: 4,
            sim_time: -5,
            elapsed_ms: 6,
            eta_ms: 7,
            bytes_in_use: 8,
            bytes_peak: 9,
            keyframes_ready: 10,
            events_total: 11,
        };
        assert_eq!(p.to_array(), [0, 1, 2, 3, 4, -5, 6, 7, 8, 9, 10, 11]);
    }
}

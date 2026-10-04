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

use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use deeptime_format::fsutil::write_atomic;
use deeptime_format::manifest::MANIFEST;
use deeptime_format::sites::{INDEX_PATH as SITE_INDEX_PATH, SITES_PATH, write_sites};
use deeptime_format::{
    Codec, Event, EventWriter, FileEntry, GEOLOGY_WRITER, Manifest, Ramps, Scale, Writer,
};
use serde_json::{Value, json};

use crate::checkpoint::{Checkpoint, CheckpointRef};
use crate::deposits::recorded::Evicted;
use crate::deposits::spill::SpillStore;
use crate::error::CoreError;
use crate::keyframes::{self, KeyframePolicy, KeyframeStore, RING_CAPACITY, ViewScale};
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

/// The memory and time estimate for a preset under the default memory cap, as JSON (plan
/// §5.7.5); [`estimate_for`] with a cap of 0.
pub fn estimate(preset_json: &str, content_json: &str) -> Result<String, CoreError> {
    estimate_for(preset_json, content_json, 0)
}

/// Seconds to write and read back a byte of spilled history: written once, read by the deposit
/// engine's two passes (an SSD at ~1 GB/s; P1-17 measures it).
const SPILL_SECONDS_PER_BYTE: f64 = 3.0e-9;

/// The memory and time estimate for a preset run under a memory cap of `mem_cap` bytes (0: the
/// default), as JSON (plan §5.7.5; P1-17's history spill):
///
/// `{"peak_bytes", "base_bytes", "history_bytes", "spill_bytes", "mem_cap",
///   "stages": [{"name", "bytes", "seconds", "base_bytes", "history_bytes", "spill_bytes",
///   "coupled_to"}, …]}`
///
/// `coupled_to` is the stage a coupled stage steps with (`Stage::coupled_to`), or null: a
/// running session reports a group of coupled stages as its lead's index and permille, so a
/// caller weighting progress by the stages' seconds needs the groups.
///
/// A stage's `base_bytes` is its working set without the recorded history (what must fit under
/// the cap, or the run fails cleanly), `history_bytes` the history at the end of the stage (it
/// stays in memory while the cap allows and spills to the run's folder after that), `bytes`
/// = base + the history held in memory under this cap (its peak) and `spill_bytes` the rest (the
/// disk it uses). The run's figures are the stages' maxima: `peak_bytes` ≈ base + what of the
/// history fits, so a run whose `peak_bytes` is over its cap is one whose base does not fit.
/// Seconds include the spill's disk time.
pub fn estimate_for(
    preset_json: &str,
    content_json: &str,
    mem_cap: u64,
) -> Result<String, CoreError> {
    let preset = Preset::from_json(preset_json)?;
    preset.check_runnable()?;
    check_content(content_json)?;
    let preset = preset.with_content(content_json)?;
    let cap = if mem_cap == 0 {
        DEFAULT_MEM_CAP
    } else {
        mem_cap
    };
    let (mut peak, mut base, mut history, mut spill) = (0u64, 0u64, 0u64, 0u64);
    let stages: Vec<Value> = preset
        .stages()
        .iter()
        .map(|s| {
            let e = s.estimate();
            let hist = e.history_bytes.min(e.bytes);
            let b = e.bytes - hist;
            let held = hist.min(cap.saturating_sub(b));
            let spilled = hist - held;
            peak = peak.max(b + held);
            base = base.max(b);
            history = history.max(hist);
            spill = spill.max(spilled);
            json!({
                "name": s.name(), "bytes": b + held,
                "seconds": e.seconds + spilled as f64 * SPILL_SECONDS_PER_BYTE,
                "base_bytes": b, "history_bytes": hist, "spill_bytes": spilled,
                "coupled_to": s.coupled_to(),
            })
        })
        .collect();
    Ok(json!({
        "peak_bytes": peak, "base_bytes": base, "history_bytes": history,
        "spill_bytes": spill, "mem_cap": cap, "stages": stages,
    })
    .to_string())
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
    /// [`stage_groups`] of `stages`; `current` is always a group's lead.
    groups: Vec<Range<usize>>,
    current: usize,
    world: WorldState,
    /// Keyframe times are `kf_origin + k · every_years`.
    kf_origin: f64,
    next_kf: f64,
    /// Time and group (its lead's index) of the last keyframe: a group's closing keyframe is
    /// skipped when one was just drawn at the same instant, but a *later* group (the deposit
    /// stage runs at the end of time, where the plates and surface just drew theirs) still gets
    /// its own.
    last_kf_t: Option<(f64, usize)>,
    timings: Vec<StageTiming>,
}

impl Stepper {
    /// The group whose lead is `current` (empty past the last stage).
    fn group(&self) -> Range<usize> {
        self.groups
            .iter()
            .find(|g| g.start == self.current)
            .cloned()
            .unwrap_or(self.current..self.current)
    }
}

/// Saved states of a group's members (a checkpoint taken inside a group of more than one stage).
fn encode_group_state(states: &[Vec<u8>]) -> Result<Vec<u8>, CoreError> {
    crate::stage::to_state(&states.to_vec())
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
    /// The preset's scale as the keyframes' view layers read it (the satellite view).
    view: ViewScale,
    has_keyframes: bool,
    ramps: Ramps,
    checkpoint_dir: Option<PathBuf>,
    /// Where the recorded history's evicted frames go (P1-17): a spill file in `out_dir`.
    spill: Arc<SpillStore>,
    /// A fixed budget for the history's frames in memory, on top of the memory cap (tests and
    /// `--history-mem`); `None`: the cap alone decides.
    history_budget: Option<u64>,
    /// The first eviction was logged.
    spill_logged: AtomicBool,
    /// The most any stage's reservation asked for: bytes in use plus bytes reserved, with and
    /// without the history's frames then in memory.
    needed_peak: [AtomicU64; 2],
}

/// The share of the memory cap kept free after a step: the history is trimmed to
/// `cap - cap / TRIM_HEADROOM` so a stage's next allocations (reserved or not) have room.
const TRIM_HEADROOM: u64 = 16;

/// Logs an eviction of the recorded history's frames (`why`: what asked for the memory).
pub(crate) fn log_evicted(
    log: &LogRing,
    e: &Evicted,
    gauge: &dyn MemoryGauge,
    cap: u64,
    why: &str,
) {
    if e.frames == 0 {
        return;
    }
    let level = if e.created_file {
        Level::Info
    } else {
        Level::Debug
    };
    log.push(
        level,
        format!(
            "history: {} frame(s), {:.1} MiB, spilled to disk for {why} ({:.0} of {:.0} MiB in use){}",
            e.frames,
            e.bytes as f64 / 1_048_576.0,
            gauge.in_use() as f64 / 1_048_576.0,
            cap as f64 / 1_048_576.0,
            if e.created_file {
                "; the run continues from the spill file in its folder, slower but within the cap"
            } else {
                ""
            }
        ),
    );
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
        preset.check_runnable()?;
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
        preset.check_runnable()?;
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
        // The content bundle's essentials replace the preset's (P1-15); the canonical preset the
        // run records (preset.json, checkpoints) therefore carries the list it ran with.
        let preset = preset.with_content(content_json)?;
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
        let groups = stage_groups(&stages)?;
        if next_stage < stages.len() && !groups.iter().any(|g| g.start == next_stage) {
            return Err(CoreError::InvalidState(format!(
                "the checkpoint stops inside a group of coupled stages (at {})",
                stages[next_stage].name()
            )));
        }
        let group = groups
            .iter()
            .find(|g| g.start == next_stage)
            .cloned()
            .unwrap_or(next_stage..next_stage);
        if !stage_state.is_empty() {
            if group.len() > 1 {
                let states: Vec<Vec<u8>> = crate::stage::from_state(stage_state)?;
                if states.len() != group.len() {
                    return Err(CoreError::InvalidState(format!(
                        "the checkpoint holds {} states for a group of {} stages",
                        states.len(),
                        group.len()
                    )));
                }
                for (k, st) in group.clone().zip(states) {
                    if !st.is_empty() {
                        stages[k].load_state(&st)?;
                    }
                }
            } else if let Some(stage) = stages.get_mut(next_stage) {
                stage.load_state(stage_state)?;
            }
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
        let view = preset.view_scale();
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
        let spill = Arc::new(SpillStore::new(&options.out_dir));
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
                groups,
                current: next_stage,
                world,
                kf_origin,
                next_kf,
                last_kf_t: None,
                timings,
            }),
            policy,
            view,
            has_keyframes,
            ramps: Ramps::builtin(),
            checkpoint_dir: None,
            spill,
            history_budget: None,
            spill_logged: AtomicBool::new(false),
            needed_peak: [AtomicU64::new(0), AtomicU64::new(0)],
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

    /// Holds at most `bytes` of the recorded history's packed frames in memory, whatever the
    /// cap (`Some(0)`: every frame goes to disk as soon as its step is done); `None` leaves it to
    /// the cap. Results do not depend on it; tests and `--history-mem` use it.
    pub fn set_history_budget(&mut self, bytes: Option<u64>) {
        self.history_budget = bytes;
    }

    /// The spill store: the disk the recorded history used (bytes and frames written).
    pub fn spill(&self) -> &SpillStore {
        &self.spill
    }

    /// The most any stage's reservation asked for so far: bytes in use plus bytes reserved.
    pub fn needed_peak(&self) -> u64 {
        self.needed_peak[0].load(Ordering::Relaxed)
    }

    /// [`Session::needed_peak`] without the recorded history's frames then in memory: the smallest
    /// cap the run's reservations fit (the history spilling to make room), give or take the
    /// memory other threads hold at that moment.
    pub fn needed_base_peak(&self) -> u64 {
        self.needed_peak[1].load(Ordering::Relaxed)
    }

    /// Runs `f` on the world as it stands (between `advance` calls), without copying it.
    pub fn with_world<R>(&self, f: impl FnOnce(&WorldState) -> R) -> Result<R, CoreError> {
        let s = self.lock_stepper()?;
        Ok(f(&s.world))
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
            let group = s.group();
            if s.stages[group.clone()].iter().all(|st| st.is_done()) {
                let name = group_label(&s.stages, &group);
                self.finish_stage(s)
                    .map_err(|e| self.fail(&format!("finishing stage {name}"), e))?;
                self.stage_permille.store(0, Ordering::Release);
                if s.current >= s.stages.len() {
                    return Ok(self.complete());
                }
                self.stage_index.store(s.current as u32, Ordering::Release);
                return Ok(AdvanceStatus::StageDone);
            }
            // One step of the lead, then one of every follower (a group of one: just the stage).
            let mut last = index;
            for k in group.clone() {
                let stage = &mut s.stages[k];
                if stage.is_done() {
                    continue;
                }
                if stage.steps_done() == 0 {
                    self.log
                        .push(Level::Info, format!("stage {} started", stage.name()));
                }
                // The history's frames give way to a stage's reservations (P1-17).
                let history = s.world.history.as_ref().map(|h| {
                    h.attach(&self.spill);
                    h.handle()
                });
                let ctx = StageContext {
                    pool: &self.pool,
                    log: &self.log,
                    out_dir: &self.out_dir,
                    until,
                    gauge: self.gauge.as_ref(),
                    mem_cap: self.mem_cap,
                    state: &self.state,
                    history: history.as_ref(),
                    needed_peak: Some(&self.needed_peak),
                };
                let stepped = Instant::now();
                if let Err(e) = stage.step(&mut s.world, &ctx) {
                    let what = format!("stage {}", stage.name());
                    return Err(self.fail(&what, e));
                }
                if let Err(e) = self.trim_history(&s.world) {
                    return Err(self.fail("spilling the recorded history", e));
                }
                s.timings[k].steps = stage.steps_done();
                s.timings[k].millis += stepped.elapsed().as_secs_f64() * 1000.0;
                last = k;
            }
            let lead = &s.stages[index];
            self.stage_permille.store(
                permille(lead.steps_done(), lead.total_steps()),
                Ordering::Release,
            );
            let stepped = Instant::now();
            self.after_step(s)
                .map_err(|e| self.fail("keyframe or event publishing", e))?;
            // A stage's time includes publishing its events and drawing its keyframes (a group's
            // go to its last member that stepped).
            s.timings[last].millis += stepped.elapsed().as_secs_f64() * 1000.0;
            if start.elapsed() >= budget {
                return Ok(AdvanceStatus::Running);
            }
        }
    }

    /// After a step: the history's oldest frames go to disk while the gauge is over
    /// `cap - cap / TRIM_HEADROOM` (or the history over its fixed budget), so memory stays flat
    /// however long the run.
    fn trim_history(&self, world: &WorldState) -> Result<(), CoreError> {
        let Some(h) = &world.history else {
            return Ok(());
        };
        h.attach(&self.spill);
        let handle = h.handle();
        let mut out = Evicted::default();
        let mut why = "the memory cap";
        if let Some(budget) = self.history_budget {
            let resident = handle.resident_bytes();
            if resident > budget {
                out = handle.evict(resident - budget)?;
                why = "the history budget (--history-mem)";
            }
        }
        let target = self.mem_cap - self.mem_cap / TRIM_HEADROOM;
        let in_use = self.gauge.in_use();
        if in_use > target {
            let e = handle.evict(in_use - target)?;
            if e.frames > 0 {
                why = "the memory cap";
            }
            out.frames += e.frames;
            out.bytes += e.bytes;
        }
        if out.frames > 0 {
            out.created_file = !self.spill_logged.swap(true, Ordering::AcqRel);
            log_evicted(&self.log, &out, self.gauge.as_ref(), self.mem_cap, why);
        }
        Ok(())
    }

    /// The history's state for the log and the manifest.
    fn history_note(&self, world: &WorldState) -> Option<Value> {
        let h = world.history.as_ref()?;
        Some(json!({
            "frames": h.frame_count(),
            "bytes": h.bytes(),
            "resident_bytes": h.resident_bytes(),
            "frames_on_disk": h.frames_on_disk(),
            "spilled_bytes": self.spill.written_bytes(),
            "spilled_frames": self.spill.written_frames(),
            "spill_read_bytes": self.spill.io_stats().0,
            "spill_read_seconds": self.spill.io_stats().1,
            "spill_write_seconds": self.spill.io_stats().2,
            "reserved_peak_bytes": self.needed_peak(),
            "reserved_base_peak_bytes": self.needed_base_peak(),
        }))
    }

    /// Publishes new events and the clock; takes a keyframe when the clock passed the next one.
    fn after_step(&self, s: &mut Stepper) -> Result<(), CoreError> {
        self.publish(&s.world);
        if let Some(every) = self.policy.every_years
            && s.world.t_years >= s.next_kf
        {
            let group = s.group();
            let layers = group_layers(&s.stages, &group);
            if !layers.is_empty() {
                let name = group_label(&s.stages, &group);
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
        if s.last_kf_t == Some((s.world.t_years, s.current)) {
            return Ok(());
        }
        let n = self.keyframes.next_n();
        let drawn = Instant::now();
        // In the session's own pool: the water view's hydrology is parallel in places.
        let world = &s.world;
        if let Some(frame) = self.pool.install(|| {
            keyframes::render(
                world,
                &self.ramps,
                &self.view,
                stage,
                layers,
                n,
                self.policy.size,
            )
        }) {
            let millis = drawn.elapsed().as_secs_f64() * 1000.0;
            self.keyframes.push(frame)?;
            s.last_kf_t = Some((s.world.t_years, s.current));
            self.log.push(
                Level::Debug,
                format!(
                    "keyframe {n} at t={} ({stage}), drawn in {millis:.0} ms",
                    s.world.t_years
                ),
            );
        }
        Ok(())
    }

    /// The bake wrote `rivers/rivers.dtrv` and the `water_level` layer: the last frame that shows
    /// the water (`satellite`, `rivers`) is drawn again from them, so the final map has exactly
    /// the rivers and lakes of the bake, not the ones derived from the frame's terrain. Without a
    /// rivers file (no bake, or `hydro.enabled` false) the frame stays as it was; a failure is a
    /// warning, never the run's.
    fn bake_water_into_last_frame(
        &self,
        world: &WorldState,
        dir: &Path,
        m: &deeptime_format::Manifest,
    ) {
        let index = self.keyframes.index();
        let Some(entry) = index.frames.iter().rev().find(|f| {
            f.layers
                .iter()
                .any(|l| l == crate::satellite::LAYER || l == crate::water_view::LAYER)
        }) else {
            return;
        };
        let started = Instant::now();
        let result = (|| -> Result<Option<String>, CoreError> {
            let water = crate::water_view::from_bake(
                world,
                &self.view,
                dir,
                m.layer("water_level").map(|l| l.path.as_str()),
                (entry.w, entry.h),
            )?;
            let Some(water) = water else {
                return Ok(None);
            };
            let frame = self.keyframes.get(entry.n)?;
            let Some(redrawn) = self.pool.install(|| {
                keyframes::redraw_water(&frame, world, &self.ramps, &self.view, &water)
            }) else {
                return Ok(None);
            };
            self.keyframes.replace(redrawn)?;
            Ok(Some(format!(
                "keyframe {} drawn again with the bake's water ({} river chains, {:.0} px of lakes) in {:.0} ms",
                entry.n,
                water.info.chains,
                water.info.lake_px,
                started.elapsed().as_secs_f64() * 1000.0
            )))
        })();
        match result {
            Ok(Some(line)) => self.log.push(Level::Info, line),
            Ok(None) => {}
            Err(e) => self.log.push(
                Level::Warn,
                format!("could not draw the last keyframe with the bake's water: {e}"),
            ),
        }
    }

    /// The current stage just finished: a closing keyframe, a checkpoint if asked for, next stage.
    fn finish_stage(&self, s: &mut Stepper) -> Result<(), CoreError> {
        let index = s.current;
        let group = s.group();
        let name = group_label(&s.stages, &group);
        self.log.push(Level::Info, format!("stage {name} finished"));
        let started = Instant::now();
        self.publish(&s.world);
        if self.policy.every_years.is_some() {
            let layers = group_layers(&s.stages, &group);
            if !layers.is_empty() {
                self.take_keyframe(s, &name, &layers)?;
            }
        }
        let last = group.end.max(index + 1) - 1;
        s.timings[last].millis += started.elapsed().as_secs_f64() * 1000.0;
        s.current = group.end.max(index + 1);
        if let Some(note) = self.history_note(&s.world) {
            let mib = |k: &str| note[k].as_f64().unwrap_or(0.0) / 1_048_576.0;
            self.log.push(
                Level::Info,
                format!(
                    "history after {name}: {} frames, {:.1} MiB ({:.1} MiB in memory, {} frames on disk; {:.1} MiB written to the spill file); {:.0} MiB in use of the {:.0} MiB cap",
                    note["frames"],
                    mib("bytes"),
                    mib("resident_bytes"),
                    note["frames_on_disk"],
                    mib("spilled_bytes"),
                    self.gauge.in_use() as f64 / 1_048_576.0,
                    self.mem_cap as f64 / 1_048_576.0,
                ),
            );
        }
        if let Some(dir) = &self.checkpoint_dir {
            let path = dir.join(format!("{index}-{name}.dtcp"));
            self.write_checkpoint_of(s, &path)?;
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
        Ok(Checkpoint {
            preset_json: self.preset_json.clone(),
            content_json: self.content_json.clone(),
            stage_names: self.stage_names.clone(),
            next_stage: s.current as u32,
            stage_state: self.stage_state_of(s)?,
            world: s.world.clone(),
        })
    }

    /// Writes the run as it stands to `path`, streamed from the world (no copy of it is made).
    fn write_checkpoint_of(&self, s: &Stepper, path: &Path) -> Result<(), CoreError> {
        let stage_state = self.stage_state_of(s)?;
        CheckpointRef {
            preset_json: &self.preset_json,
            content_json: &self.content_json,
            stage_names: &self.stage_names,
            next_stage: s.current as u32,
            stage_state: &stage_state,
            world: &s.world,
        }
        .write(path)
    }

    /// The run as it stands, written to `path` (between `advance` calls; what
    /// [`Session::checkpoint`] returns, without copying the world).
    pub fn write_checkpoint(&self, path: &Path) -> Result<(), CoreError> {
        let s = self.lock_stepper()?;
        self.write_checkpoint_of(&s, path)
    }

    fn stage_state_of(&self, s: &Stepper) -> Result<Vec<u8>, CoreError> {
        // A stage that has stepped keeps its state, even after its last step (the session moves
        // on to the next stage only on the following `advance`): a resume must not rerun it.
        let group = s.group();
        let stage_state = if group.len() > 1 {
            let mut states = Vec::with_capacity(group.len());
            for st in &s.stages[group.clone()] {
                states.push(if st.steps_done() > 0 {
                    st.save_state()?
                } else {
                    Vec::new()
                });
            }
            if states.iter().all(Vec::is_empty) {
                Vec::new()
            } else {
                encode_group_state(&states)?
            }
        } else {
            match s.stages.get(s.current) {
                Some(stage) if stage.steps_done() > 0 => stage.save_state()?,
                _ => Vec::new(),
            }
        };
        Ok(stage_state)
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
        if let Some(bake) = self.preset.bake() {
            let started = Instant::now();
            let (kind, baked) = match bake {
                crate::preset::Bake::Analytic(kind) => (
                    kind,
                    crate::synth::build(
                        kind,
                        crate::synth::Strata::Placeholder,
                        c,
                        Codec::Lz4Block,
                    )?,
                ),
                // P1-15: a synthetic deposit world's terrain follows the history's present
                // elevation, and its spawn candidates are ranked by the essentials.
                crate::preset::Bake::DepositHistory {
                    params,
                    strata,
                    essentials,
                } => {
                    let history = crate::deposits::synthetic::SyntheticHistory::new(params)?;
                    let terrain = crate::deposits::synthetic::HistoryTerrain::new(&history);
                    let kind = crate::synth::Kind::DepositHistory;
                    let mut baked =
                        crate::synth::build_with(&terrain, kind, strata, c, Codec::Lz4Block)?;
                    if let Some(ranked) = crate::deposits::spawn::ranked_candidates(
                        &baked.layers[0],
                        &baked.layers[1],
                        m.scale.sea_level_y,
                        &world.sites,
                        &essentials,
                    ) {
                        baked.spawn = ranked;
                    }
                    (kind, baked)
                }
            };
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
        // The L3 layers the amplify stage spilled into the folder (P1-06).
        crate::surface::amplify::register(world, &mut m)?;
        // P1-08: a planet run's chunk-time layers (plan §5.3, §8.8).
        if let crate::preset::PresetBody::Planet(p) = &self.preset.body {
            let note = self
                .pool
                .install(|| crate::bake::bake(world, p, dir, &mut m))?;
            self.log.push(Level::Info, crate::bake::summary_line(&note));
            m.notes.insert(crate::bake::NOTES_KEY.into(), note);
            // The last frame shows the rivers and lakes the bake wrote (before the frame files
            // are listed in the manifest below).
            self.bake_water_into_last_frame(world, dir, &m);
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

        let (bytes, crc32) = deeptime_format::bodies::write_bodies(dir, &world.bodies)?;
        files.push(FileEntry {
            path: deeptime_format::bodies::BODIES_PATH.into(),
            bytes,
            crc32,
        });

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
            "bodies": world.bodies.len(),
            "keyframes": kf.frames.len(),
        });
        let header = &self.preset.header;
        if let Some(t) = header.timeline {
            run["timeline"] = json!({ "start": t.start.name(), "length_myr": t.length_myr });
        }
        if header.placeholder {
            run["placeholder"] = json!(true);
        }
        if let Some(h) = self.history_note(world) {
            run["history"] = h;
        }
        m.notes.insert("run".into(), run);
        // Stage summaries (the deposit census of P1-14).
        for (stage, text) in &world.notes {
            let v: Value = serde_json::from_str(text).unwrap_or_else(|_| json!(text));
            m.notes.insert(stage.clone(), v);
        }
        let mut bytes = m.to_json_bytes()?;
        write_atomic(&dir.join(MANIFEST), &bytes)?;
        // The impact craters' breccia lenses go under the floor the bake drew (the amplifier
        // changed the ground after the impact stage placed them): the pass reads the complete
        // folder as the game does, so the manifest is written again.
        if let crate::preset::PresetBody::Planet(p) = &self.preset.body
            && m.notes
                .get(crate::bake::NOTES_KEY)
                .is_some_and(|b| b["baked"] == json!(true))
            && world.impacts.is_some()
        {
            let note = crate::surface::impacts::settle_folder(world, dir, p, &mut m)?;
            if note["lenses"].as_u64().unwrap_or(0) > 0 {
                self.log.push(
                    Level::Info,
                    format!(
                        "impact lenses: {} seated under the drawn floor, {} moved by a block or more, {} dropped",
                        note["lenses"], note["moved_a_block_or_more"], note["dropped"]
                    ),
                );
                bytes = m.to_json_bytes()?;
                write_atomic(&dir.join(MANIFEST), &bytes)?;
            }
        }
        // Placers go where the finished world has their gravel: into the river channels the bake
        // made, or onto its ground (deposits::placer_channels). The pass reads the folder as the
        // game does, so it runs on the complete folder and the manifest is written again.
        if let crate::preset::PresetBody::Planet(p) = &self.preset.body
            && m.notes
                .get(crate::bake::NOTES_KEY)
                .is_some_and(|b| b["baked"] == json!(true))
        {
            let note = crate::deposits::placer_channels::settle_folder(dir, p, &mut m)?;
            if note["placers"].as_u64().unwrap_or(0) > 0 {
                self.log.push(
                    Level::Info,
                    format!(
                        "placers: {} in river channels, {} on the ground as benches",
                        note["in_channels"], note["benches"]
                    ),
                );
                bytes = m.to_json_bytes()?;
                write_atomic(&dir.join(MANIFEST), &bytes)?;
            }
        }
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

/// The pipeline's groups of coupled stages (`Stage::coupled_to`), in order: each range is a lead
/// and the followers interleaved with it; an uncoupled stage is a group of one. Fails when a
/// coupled stage does not follow a stage of the group it names.
pub fn stage_groups(stages: &[Box<dyn Stage>]) -> Result<Vec<Range<usize>>, CoreError> {
    let mut groups = Vec::new();
    let mut i = 0;
    while i < stages.len() {
        if let Some(to) = stages[i].coupled_to() {
            return Err(CoreError::InvalidState(format!(
                "stage {} is coupled to {to}, which does not come just before it",
                stages[i].name()
            )));
        }
        let mut j = i + 1;
        while j < stages.len()
            && stages[j]
                .coupled_to()
                .is_some_and(|to| stages[i..j].iter().any(|s| s.name() == to))
        {
            j += 1;
        }
        groups.push(i..j);
        i = j;
    }
    Ok(groups)
}

/// The group (range of stage indices) holding the stage called `name`.
pub fn group_of(stages: &[Box<dyn Stage>], name: &str) -> Option<Range<usize>> {
    let groups = stage_groups(stages).ok()?;
    let k = stages.iter().position(|s| s.name() == name)?;
    groups.into_iter().find(|g| g.contains(&k))
}

/// A group's label: its members' names joined with `+` (`plates+surface`).
fn group_label(stages: &[Box<dyn Stage>], g: &Range<usize>) -> String {
    stages[g.clone()]
        .iter()
        .map(|s| s.name())
        .collect::<Vec<_>>()
        .join("+")
}

/// The union of a group's keyframe layers, in member order.
fn group_layers(stages: &[Box<dyn Stage>], g: &Range<usize>) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for s in &stages[g.clone()] {
        for &l in s.keyframe_layers() {
            if !out.contains(&l) {
                out.push(l);
            }
        }
    }
    keyframes::add_derived(&mut out);
    out
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
        assert_eq!(v["format_version"], "0.4");
        assert_eq!(v["layers"], json!([]));
        assert!(manifest.contains("\"format_version\": \"0.4\""));
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
            r#"{"base_bytes":0,"history_bytes":0,"mem_cap":4294967296,"peak_bytes":0,"spill_bytes":0,"stages":[{"base_bytes":0,"bytes":0,"history_bytes":0,"name":"dummy","seconds":0.0,"spill_bytes":0}]}"#
        );
        let world: Value = serde_json::from_str(&estimate(DUMMY_WORLD, "").unwrap()).unwrap();
        // 256 × 88 cells (rows from z = −704, past the Lambert pole at 651.9).
        assert_eq!(world["peak_bytes"], 256 * 88 * 4);
        assert_eq!(world["stages"][1]["name"], "dummy_erode");
    }

    /// The estimate names each stage's coupling (P1-11's time left weights a running group of
    /// coupled stages by all of their seconds): followers name a stage of the group just before
    /// them, uncoupled stages null, the same groups the session steps.
    #[test]
    fn the_estimate_names_the_coupled_groups() {
        let json = crate::preset::with_timeline(
            crate::params::planet_builtin("earthlike_quick_16k").unwrap(),
            Some("supercontinent"),
            Some(100.0),
        )
        .unwrap();
        let est: Value = serde_json::from_str(&estimate(&json, "").unwrap()).unwrap();
        let stages = est["stages"].as_array().unwrap();
        let names: Vec<&str> = stages.iter().map(|s| s["name"].as_str().unwrap()).collect();
        assert!(stages[0]["coupled_to"].is_null(), "the first stage leads: {est}");
        let mut followers = 0;
        for (i, s) in stages.iter().enumerate() {
            if let Some(to) = s["coupled_to"].as_str() {
                followers += 1;
                assert!(
                    names[..i].contains(&to),
                    "{} is coupled to {to}, which comes before it",
                    names[i]
                );
            }
        }
        assert!(followers >= 1, "the real pipeline couples stages: {names:?}");
        let preset = Preset::from_json(&json).unwrap();
        let groups = stage_groups(&preset.stages()).unwrap();
        assert_eq!(
            groups.len(),
            stages.len() - followers,
            "one group per uncoupled stage"
        );
    }

    /// The estimate under a cap (P1-17): the recorded history is held while the cap allows and
    /// the rest is disk, so the peak is the base plus what fits, never over the cap unless the
    /// base itself is; a larger cap holds more and spills less; seconds grow with the spill.
    #[test]
    fn the_estimate_spills_the_history_over_the_cap() {
        // The run that failed at 4 GiB (1000 Myr of 32k full) up to its deposits: P1-06's young
        // landscape and amplifier come after and bring working sets of their own.
        let json = crate::preset::with_timeline(
            crate::params::planet_builtin("pipeline_full_32k").unwrap(),
            Some("supercontinent"),
            Some(1000.0),
        )
        .unwrap();
        let mut v: Value = serde_json::from_str(&json).unwrap();
        v.as_object_mut().unwrap().remove("young");
        v.as_object_mut().unwrap().remove("amplify");
        let json = v.to_string();
        let at = |cap: u64| -> Value {
            serde_json::from_str(&estimate_for(&json, "", cap).unwrap()).unwrap()
        };
        let big = at(64 << 30);
        let (base, history) = (
            big["base_bytes"].as_u64().unwrap(),
            big["history_bytes"].as_u64().unwrap(),
        );
        assert!(history > 2 << 30, "1000 Myr of 32k frames: {history} bytes");
        assert_eq!(big["spill_bytes"], 0);
        assert_eq!(big["peak_bytes"].as_u64().unwrap(), base + history);
        assert_eq!(big["mem_cap"], 64u64 << 30);
        let four = at(4 << 30);
        assert_eq!(four, at(0), "0 is the default cap, 4 GiB");
        let peak = four["peak_bytes"].as_u64().unwrap();
        let spill = four["spill_bytes"].as_u64().unwrap();
        assert!(base < 4 << 30, "the base fits 4 GiB: {base}");
        assert!(peak <= 4 << 30 && spill > 0, "{four}");
        assert_eq!(four["base_bytes"], big["base_bytes"]);
        let secs = |v: &Value| -> f64 {
            v["stages"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s["seconds"].as_f64().unwrap())
                .sum()
        };
        assert!(secs(&four) > secs(&big));
        // Every stage's own figures add up.
        for s in four["stages"].as_array().unwrap() {
            let (b, h, sp, bytes) = (
                s["base_bytes"].as_u64().unwrap(),
                s["history_bytes"].as_u64().unwrap(),
                s["spill_bytes"].as_u64().unwrap(),
                s["bytes"].as_u64().unwrap(),
            );
            assert_eq!(bytes + sp, b + h, "{s}");
        }
        // A cap under the base: the peak is the base, over the cap (the run would fail cleanly),
        // and the whole history would be disk.
        let tiny = at(1 << 30);
        assert_eq!(tiny["peak_bytes"], tiny["base_bytes"]);
        assert!(tiny["peak_bytes"].as_u64().unwrap() > 1 << 30);
        assert_eq!(tiny["spill_bytes"], tiny["history_bytes"]);
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

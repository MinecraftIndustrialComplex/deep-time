// SPDX-License-Identifier: LGPL-3.0-or-later
//! A simulation run: the session state machine, the `Stage` trait and, in Phase 0, `DummyStage`
//! (plan §6.1 "What each entry point does in Phase 0").
//!
//! Threading (plan §6.2): one caller steps a session (`advance` and `finalize_write` take the step
//! mutex); `progress` and `cancel` touch only atomics and never block on it. Data-parallel work runs
//! on the session's private Rayon pool, which is dropped with the session. No thread here ever
//! calls back into the JVM.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::error::CoreError;
use crate::log::{Level, LogRing};

/// Number of `long`s `progress` fills (plan §6.1).
pub const PROGRESS_FIELDS: usize = 12;

/// The default native memory cap, 4 GiB (plan §5.7.5).
pub const DEFAULT_MEM_CAP: u64 = 4 << 30;

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

/// What a stage sees while it steps.
pub struct StageContext<'a> {
    /// The session's private Rayon pool; run data-parallel kernels with `pool.install(..)`.
    pub pool: &'a rayon::ThreadPool,
    /// The session's log ring.
    pub log: &'a LogRing,
    gauge: &'a dyn MemoryGauge,
    mem_cap: u64,
    state: &'a AtomicU8,
}

impl StageContext<'_> {
    /// Checks, before allocating, that `bytes` more would stay within the memory cap.
    pub fn reserve(&self, bytes: u64) -> Result<(), CoreError> {
        let needed = self.gauge.in_use().saturating_add(bytes);
        if needed > self.mem_cap {
            return Err(CoreError::MemoryCap {
                needed,
                cap: self.mem_cap,
            });
        }
        Ok(())
    }

    /// True once the session was cancelled; long steps poll it and return early.
    pub fn cancelled(&self) -> bool {
        self.state.load(Ordering::Acquire) == SessionState::Cancelled as u8
    }
}

/// One stage of the pipeline (plates, surface, climate, … from Phase 1). A stage is stepped in
/// small increments so `advance` can honour its time budget and cancellation between steps.
pub trait Stage: Send {
    fn name(&self) -> &'static str;
    fn total_steps(&self) -> u64;
    fn steps_done(&self) -> u64;
    /// Runs one step. May allocate only after `ctx.reserve`.
    fn step(&mut self, ctx: &StageContext<'_>) -> Result<(), CoreError>;
    fn is_done(&self) -> bool {
        self.steps_done() >= self.total_steps()
    }
}

/// The Phase 0 stage: counts to `steps`, sleeping `sleep_ms` per step. Two optional test hooks:
/// `reserve_bytes` asks the memory cap for that much before the first step, and `panic_at_step`
/// panics at that step (to exercise the panic guard and session poisoning).
pub struct DummyStage {
    params: DummyParams,
    done: u64,
}

impl DummyStage {
    pub fn new(params: DummyParams) -> Self {
        DummyStage { params, done: 0 }
    }
}

impl Stage for DummyStage {
    fn name(&self) -> &'static str {
        "dummy"
    }

    fn total_steps(&self) -> u64 {
        self.params.steps
    }

    fn steps_done(&self) -> u64 {
        self.done
    }

    fn step(&mut self, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        if self.done == 0 && self.params.reserve_bytes > 0 {
            ctx.reserve(self.params.reserve_bytes)?;
        }
        if self.params.panic_at_step == Some(self.done) {
            panic!("dummy stage: panic_at_step {} reached", self.done);
        }
        if self.params.sleep_ms > 0 {
            std::thread::sleep(Duration::from_millis(self.params.sleep_ms));
        }
        self.done += 1;
        Ok(())
    }
}

/// Parameters of the dummy preset `{"dummy":{"steps":N,"sleep_ms":M}}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DummyParams {
    pub steps: u64,
    pub sleep_ms: u64,
    pub reserve_bytes: u64,
    pub panic_at_step: Option<u64>,
}

/// A parsed preset. Phase 0 knows only the dummy preset; anything else is `PresetInvalid`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Preset {
    Dummy(DummyParams),
}

const MAX_DUMMY_STEPS: u64 = 10_000_000;
const MAX_DUMMY_SLEEP_MS: u64 = 10_000;

impl Preset {
    pub fn from_json(json: &str) -> Result<Preset, CoreError> {
        let value: Value = serde_json::from_str(json)
            .map_err(|e| CoreError::preset("$", format!("not valid JSON: {e}")))?;
        let obj = value
            .as_object()
            .ok_or_else(|| CoreError::preset("$", "expected a JSON object"))?;
        if let Some(key) = obj.keys().find(|k| k.as_str() != "dummy") {
            return Err(CoreError::preset(
                format!("$.{key}"),
                "unknown key (Phase 0 accepts only the dummy preset)",
            ));
        }
        let dummy = obj.get("dummy").ok_or_else(|| {
            CoreError::preset(
                "$",
                r#"Phase 0 accepts only the dummy preset {"dummy":{"steps":N,"sleep_ms":M}}"#,
            )
        })?;
        let dummy = dummy
            .as_object()
            .ok_or_else(|| CoreError::preset("$.dummy", "expected a JSON object"))?;
        for key in dummy.keys() {
            if !matches!(
                key.as_str(),
                "steps" | "sleep_ms" | "reserve_bytes" | "panic_at_step"
            ) {
                return Err(CoreError::preset(format!("$.dummy.{key}"), "unknown key"));
            }
        }
        let field = |name: &str, required: bool, max: u64| -> Result<Option<u64>, CoreError> {
            match dummy.get(name) {
                None if required => Err(CoreError::preset(format!("$.dummy.{name}"), "missing")),
                None => Ok(None),
                Some(v) => {
                    let n = v.as_u64().ok_or_else(|| {
                        CoreError::preset(format!("$.dummy.{name}"), "expected an integer ≥ 0")
                    })?;
                    if n > max {
                        return Err(CoreError::preset(
                            format!("$.dummy.{name}"),
                            format!("{n} is above the maximum {max}"),
                        ));
                    }
                    Ok(Some(n))
                }
            }
        };
        Ok(Preset::Dummy(DummyParams {
            steps: field("steps", true, MAX_DUMMY_STEPS)?.unwrap_or(0),
            sleep_ms: field("sleep_ms", true, MAX_DUMMY_SLEEP_MS)?.unwrap_or(0),
            reserve_bytes: field("reserve_bytes", false, u64::MAX)?.unwrap_or(0),
            panic_at_step: field("panic_at_step", false, u64::MAX)?,
        }))
    }

    /// The preset as written into the manifest.
    pub fn to_json(&self) -> Value {
        match self {
            Preset::Dummy(p) => {
                let mut dummy = json!({ "steps": p.steps, "sleep_ms": p.sleep_ms });
                if p.reserve_bytes > 0 {
                    dummy["reserve_bytes"] = json!(p.reserve_bytes);
                }
                if let Some(at) = p.panic_at_step {
                    dummy["panic_at_step"] = json!(at);
                }
                json!({ "dummy": dummy })
            }
        }
    }

    fn stages(&self) -> Vec<Box<dyn Stage>> {
        match self {
            Preset::Dummy(p) => vec![Box::new(DummyStage::new(p.clone()))],
        }
    }
}

/// Validates the content bundle (rock classes, deposit types, fields; plan §5.4). Phase 0 has no
/// schema for it yet: empty or any valid JSON is accepted.
pub fn check_content(json: &str) -> Result<(), CoreError> {
    if json.trim().is_empty() {
        return Ok(());
    }
    serde_json::from_str::<Value>(json)
        .map(|_| ())
        .map_err(|e| CoreError::preset("content", format!("not valid JSON: {e}")))
}

/// The memory and time estimate for a preset, as JSON (plan §5.7.5). Phase 0: no stage allocates.
pub fn estimate(preset_json: &str, content_json: &str) -> Result<String, CoreError> {
    Preset::from_json(preset_json)?;
    check_content(content_json)?;
    Ok(json!({ "peak_bytes": 0, "stages": [] }).to_string())
}

/// How a session is created.
#[derive(Debug, Clone)]
pub struct SessionOptions {
    /// Where `finalize_write` puts the world data.
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
    /// Simulated time in years × −1 (0 until real stages exist).
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

struct Stepper {
    stages: Vec<Box<dyn Stage>>,
    current: usize,
}

const NOT_FINISHED: u64 = u64::MAX;

/// One simulation run.
pub struct Session {
    preset: Preset,
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
    created: Instant,
    finished_ms: AtomicU64,
    stepper: Mutex<Stepper>,
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
        let stages = preset.stages();
        let session = Session {
            stage_count: stages.len() as u32,
            preset,
            out_dir: options.out_dir,
            mem_cap,
            threads,
            pool,
            gauge,
            log: LogRing::default(),
            state: AtomicU8::new(SessionState::Running as u8),
            stage_index: AtomicU32::new(0),
            stage_permille: AtomicU32::new(0),
            created: Instant::now(),
            finished_ms: AtomicU64::new(NOT_FINISHED),
            stepper: Mutex::new(Stepper { stages, current: 0 }),
        };
        session.log.push(
            Level::Info,
            format!(
                "session created: preset {} threads={} mem_cap={} out_dir={}",
                session.preset.to_json(),
                threads,
                mem_cap,
                session.out_dir.display()
            ),
        );
        Ok(session)
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    pub fn log(&self) -> &LogRing {
        &self.log
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

    /// Runs stages for about `budget` (at least one step), then reports where the run stands.
    pub fn advance(&self, budget: Duration) -> Result<AdvanceStatus, CoreError> {
        let mut stepper = self.stepper.lock().map_err(|_| CoreError::Poisoned)?;
        let start = Instant::now();
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
            let index = stepper.current;
            if index >= stepper.stages.len() {
                return Ok(self.complete());
            }
            if stepper.stages[index].is_done() {
                self.log.push(
                    Level::Info,
                    format!("stage {} finished", stepper.stages[index].name()),
                );
                stepper.current += 1;
                self.stage_permille.store(0, Ordering::Release);
                if stepper.current >= stepper.stages.len() {
                    return Ok(self.complete());
                }
                self.stage_index
                    .store(stepper.current as u32, Ordering::Release);
                return Ok(AdvanceStatus::StageDone);
            }
            let stage = &mut stepper.stages[index];
            if stage.steps_done() == 0 {
                self.log
                    .push(Level::Info, format!("stage {} started", stage.name()));
            }
            let ctx = StageContext {
                pool: &self.pool,
                log: &self.log,
                gauge: self.gauge.as_ref(),
                mem_cap: self.mem_cap,
                state: &self.state,
            };
            if let Err(e) = stage.step(&ctx) {
                self.log
                    .push(Level::Error, format!("stage {} failed: {e}", stage.name()));
                self.leave_running(SessionState::Failed);
                return Err(e);
            }
            self.stage_permille.store(
                permille(stage.steps_done(), stage.total_steps()),
                Ordering::Release,
            );
            if start.elapsed() >= budget {
                return Ok(AdvanceStatus::Running);
            }
        }
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
            sim_time: 0,
            elapsed_ms,
            eta_ms,
            bytes_in_use: self.gauge.in_use(),
            bytes_peak: self.gauge.peak(),
            keyframes_ready: 0,
            events_total: 0,
        }
    }

    /// Keyframes produced so far. Phase 0: none.
    pub fn keyframe_count(&self) -> u32 {
        0
    }

    /// Geologic events after `cursor`, as a JSON array. Phase 0: none.
    pub fn events_since(&self, _cursor: u64, _max: u32) -> String {
        "[]".to_string()
    }

    /// After `Done`: writes `manifest.json` last (tmp, fsync, rename; plan §6.3) and returns it.
    pub fn finalize_write(&self) -> Result<String, CoreError> {
        let _stepper = self.stepper.lock().map_err(|_| CoreError::Poisoned)?;
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
        let created_unix_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as u64);
        let manifest = json!({
            "format_version": crate::FORMAT_VERSION,
            "writer": { "deeptime": crate::VERSION },
            "preset": self.preset.to_json(),
            "layers": [],
            "created_unix_ms": created_unix_ms,
        });
        let text = serde_json::to_string_pretty(&manifest)
            .map_err(|e| CoreError::InvalidState(format!("manifest serialisation: {e}")))?;
        write_atomically(&self.out_dir, "manifest.json", text.as_bytes())?;
        self.state
            .store(SessionState::Finalized as u8, Ordering::Release);
        self.log.push(
            Level::Info,
            format!("manifest written to {}", self.out_dir.display()),
        );
        Ok(text)
    }
}

fn permille(done: u64, total: u64) -> u32 {
    if total == 0 {
        1000
    } else {
        (done.min(total) as u128 * 1000 / total as u128) as u32
    }
}

/// Writes `dir/name` so readers see either nothing or the whole file: `name.tmp`, fsync, rename,
/// then (best effort) fsync the directory.
pub fn write_atomically(dir: &Path, name: &str, bytes: &[u8]) -> Result<(), CoreError> {
    use std::io::Write;
    std::fs::create_dir_all(dir).map_err(|e| CoreError::io(dir, e))?;
    let tmp = dir.join(format!("{name}.tmp"));
    let target = dir.join(name);
    {
        let mut file = std::fs::File::create(&tmp).map_err(|e| CoreError::io(&tmp, e))?;
        file.write_all(bytes).map_err(|e| CoreError::io(&tmp, e))?;
        file.sync_all().map_err(|e| CoreError::io(&tmp, e))?;
    }
    std::fs::rename(&tmp, &target).map_err(|e| CoreError::io(&target, e))?;
    if let Ok(d) = std::fs::File::open(dir) {
        let _ = d.sync_all();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut status = AdvanceStatus::Running;
        for _ in 0..100 {
            status = s.advance(Duration::from_millis(2)).unwrap();
            if status == AdvanceStatus::Done {
                break;
            }
        }
        assert_eq!(status, AdvanceStatus::Done);
        let p = s.progress();
        assert_eq!(p.state, SessionState::Done);
        assert_eq!(p.total_permille, 1000);
        assert_eq!(p.stage_index, 1);
        assert_eq!(p.stage_count, 1);
        let manifest = s.finalize_write().unwrap();
        let v: Value = serde_json::from_str(&manifest).unwrap();
        assert_eq!(v["format_version"], "0.0");
        assert_eq!(v["layers"], json!([]));
        let on_disk = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
        assert_eq!(on_disk, manifest);
        assert!(!dir.join("manifest.json.tmp").exists());
        assert_eq!(s.progress().state, SessionState::Finalized);
        let lines = s.log().drain();
        assert!(lines.iter().any(|l| l.contains("stage dummy finished")));
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
            r#"{"peak_bytes":0,"stages":[]}"#
        );
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
        // progress and cancel never take the step lock, so they still answer.
        let _ = s.progress();
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

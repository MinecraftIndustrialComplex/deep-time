// SPDX-License-Identifier: LGPL-3.0-or-later
//! Where the heavy kernels run (`docs/design/gpu-compute.md`): `auto` uses a hardware GPU when
//! the build has the `gpu` feature and the machine has a usable adapter, `cpu` never does, and
//! `gpu` insists (a software adapter such as lavapipe counts), still falling back to the CPU with
//! a warning when there is none. A kernel that fails on the device (an allocation refused, a
//! validation error, a lost device) reruns its work on the CPU; the CPU path is the reference.

use std::any::Any;
use std::cell::RefCell;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use crate::log::{Level, LogRing};

/// The session's compute mode (run config `compute`, CLI `--compute`, environment
/// [`ComputeMode::ENV`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComputeMode {
    #[default]
    Auto,
    Cpu,
    Gpu,
}

impl ComputeMode {
    /// The environment variable a session's default mode comes from (`auto`, `cpu` or `gpu`).
    pub const ENV: &'static str = "DEEPTIME_COMPUTE";

    pub fn parse(s: &str) -> Option<ComputeMode> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(ComputeMode::Auto),
            "cpu" => Some(ComputeMode::Cpu),
            "gpu" => Some(ComputeMode::Gpu),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ComputeMode::Auto => "auto",
            ComputeMode::Cpu => "cpu",
            ComputeMode::Gpu => "gpu",
        }
    }

    /// [`ComputeMode::ENV`], or `auto` when it is unset or not a mode.
    pub fn from_env() -> ComputeMode {
        std::env::var(Self::ENV)
            .ok()
            .and_then(|s| ComputeMode::parse(&s))
            .unwrap_or_default()
    }
}

/// The kernels the device may take, comma-separated names or prefixes (`surface:` takes both
/// surface kernels; the names are each kernel's log name: `plates:write_l1`, `climate:cells`,
/// `amplify:middle`, `surface:diffusion`, `surface:slope_failure`). Unset or empty: every kernel.
/// A diagnostic for bisecting a difference between the CPU and GPU worlds kernel by kernel
/// (`docs/design/gpu-compute.md` §10): a kernel left out runs on the CPU as under `cpu`.
pub const ENV_KERNELS: &str = "DEEPTIME_GPU_KERNELS";

/// Whether [`ENV_KERNELS`] lets the device take `kernel` (read once a process).
pub fn kernel_allowed(kernel: &str) -> bool {
    static LIST: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    let list = LIST.get_or_init(|| {
        std::env::var(ENV_KERNELS)
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    });
    list.is_empty() || list.iter().any(|p| kernel.starts_with(p.as_str()))
}

/// A session's compute mode and log, for kernels called deep inside a stage that has no
/// [`crate::stage::StageContext`] at hand (the plates' L1 sampling, `plates/l1.rs`, whose callers
/// in `plates/mod.rs` pass none). The session puts it on every worker of its Rayon pool when the
/// worker starts ([`enter`]), so [`ambient`] finds it on any thread of that session's stage
/// steps, and on no other.
pub struct Ambient {
    mode: AtomicU8,
    log: Arc<LogRing>,
    told: Mutex<Vec<&'static str>>,
    /// What kernels keep between calls (pipelines, buffers), by kernel: it goes when the session
    /// does, so a server that made its world holds no device memory for it afterwards.
    kept: Mutex<Vec<(&'static str, Arc<dyn Any + Send + Sync>)>>,
}

impl std::fmt::Debug for Ambient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ambient")
            .field("mode", &self.mode())
            .finish_non_exhaustive()
    }
}

impl Ambient {
    pub fn new(mode: ComputeMode, log: Arc<LogRing>) -> Ambient {
        Ambient {
            mode: AtomicU8::new(mode as u8),
            log,
            told: Mutex::new(Vec::new()),
            kept: Mutex::new(Vec::new()),
        }
    }

    /// The session's slot for what `kernel` keeps between calls (empty at first). One type per
    /// key: a second type under the same key gets a slot of its own that is not kept.
    pub fn kept<T: Send + 'static>(&self, kernel: &'static str) -> Arc<Mutex<Option<T>>> {
        let mut kept = self.kept.lock().unwrap_or_else(|p| p.into_inner());
        if let Some((_, slot)) = kept.iter().find(|(k, _)| *k == kernel)
            && let Ok(slot) = slot.clone().downcast::<Mutex<Option<T>>>()
        {
            return slot;
        }
        let slot = Arc::new(Mutex::new(None));
        if !kept.iter().any(|(k, _)| *k == kernel) {
            kept.push((kernel, slot.clone()));
        }
        slot
    }

    pub fn set_mode(&self, mode: ComputeMode) {
        self.mode.store(mode as u8, Ordering::Release);
    }

    pub fn mode(&self) -> ComputeMode {
        match self.mode.load(Ordering::Acquire) {
            1 => ComputeMode::Cpu,
            2 => ComputeMode::Gpu,
            _ => ComputeMode::Auto,
        }
    }

    pub fn log(&self) -> &LogRing {
        &self.log
    }

    /// Logs `message` the first time `key` is told in this session only (a kernel called every
    /// plate step says once that it runs on the CPU, not every step).
    pub fn tell_once(&self, key: &'static str, level: Level, message: impl AsRef<str>) {
        let mut told = self.told.lock().unwrap_or_else(|p| p.into_inner());
        if !told.contains(&key) {
            told.push(key);
            self.log.push(level, message);
        }
    }
}

thread_local! {
    static AMBIENT: RefCell<Option<Arc<Ambient>>> = const { RefCell::new(None) };
}

/// Makes `a` this thread's [`ambient`] (the session's pool calls it as each worker starts).
pub fn enter(a: Arc<Ambient>) {
    AMBIENT.with(|x| *x.borrow_mut() = Some(a));
}

/// The compute context of the session whose pool runs this thread; None elsewhere (tests and
/// tools that build their own pools run their kernels on the CPU).
pub fn ambient() -> Option<Arc<Ambient>> {
    AMBIENT.with(|x| x.borrow().clone())
}

/// The device a kernel without a stage context uses ([`ambient`]'s mode; None: the CPU). Says
/// once per session and kernel where it runs.
#[cfg(feature = "gpu")]
pub fn ambient_device(kernel: &'static str) -> Option<Arc<deeptime_gpu::Gpu>> {
    let a = ambient()?;
    let mode = a.mode();
    if mode == ComputeMode::Cpu {
        return None;
    }
    if !kernel_allowed(kernel) {
        a.tell_once(
            kernel,
            Level::Info,
            format!("compute: {kernel} runs on the CPU ({ENV_KERNELS} leaves it out)"),
        );
        return None;
    }
    match deeptime_gpu::shared(mode == ComputeMode::Gpu) {
        Ok(g) => {
            a.tell_once(
                kernel,
                Level::Info,
                format!("compute: {kernel} {} on {}", mode.as_str(), g.describe()),
            );
            Some(g)
        }
        Err(e) => {
            let level = if mode == ComputeMode::Gpu {
                Level::Warn
            } else {
                Level::Info
            };
            a.tell_once(
                kernel,
                level,
                format!("compute: {kernel} {} runs on the CPU ({e})", mode.as_str()),
            );
            None
        }
    }
}

/// Wall time of the GPU kernels' parts and their CPU twins, milliseconds summed since the last
/// [`reset_timings`], and counts (cells finished on the CPU, fallbacks) as plain sums: a
/// process-wide gauge like `plates::timings`, so a run's world is the same with or without it
/// (`deeptime bench` prints it).
static TIMINGS: Mutex<Vec<(&'static str, f64)>> = Mutex::new(Vec::new());

/// Adds `amount` to `what` (milliseconds, or a count).
pub fn add(what: &'static str, amount: f64) {
    let mut t = TIMINGS.lock().unwrap_or_else(|p| p.into_inner());
    match t.iter_mut().find(|e| e.0 == what) {
        Some(e) => e.1 += amount,
        None => t.push((what, amount)),
    }
}

/// Adds the milliseconds since `since` to `what`.
pub fn lap(what: &'static str, since: std::time::Instant) {
    add(what, since.elapsed().as_secs_f64() * 1000.0);
}

/// The speed work's diagnostic (2026-10-28): with `DEEPTIME_LAP_CPU` set, each timed part of the
/// plate, climate and surface steps also sums the process's CPU time (every thread) since the
/// part before it in that stage, or since the stage's step began, so a part's CPU over its wall
/// time is the cores it kept busy. Parts timed inside another part take the CPU since the last
/// part's end. Run it with `DEEPTIME_BENCH_NO_OVERLAP` so the structure stage's steps do not run
/// beside the plates'. Off (the default), nothing is read.
pub fn lap_cpu_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("DEEPTIME_LAP_CPU").is_some())
}

/// The process's CPU time over all its threads, ms (`CLOCK_PROCESS_CPUTIME_ID`; 0 off Linux).
pub fn process_cpu_ms() -> f64 {
    #[cfg(target_os = "linux")]
    {
        #[repr(C)]
        struct Timespec {
            tv_sec: i64,
            tv_nsec: i64,
        }
        unsafe extern "C" {
            fn clock_gettime(clock: i32, ts: *mut Timespec) -> i32;
        }
        const CLOCK_PROCESS_CPUTIME_ID: i32 = 2;
        let mut ts = Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: `ts` is a valid, writable timespec of the C layout for the call's duration.
        if unsafe { clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &mut ts) } == 0 {
            return ts.tv_sec as f64 * 1.0e3 + ts.tv_nsec as f64 * 1.0e-6;
        }
    }
    0.0
}

/// Each stage's last CPU reading, and the parts' CPU sums (`stage`, `part`, ms).
static CPU_LAST: Mutex<Vec<(&'static str, f64)>> = Mutex::new(Vec::new());
static CPU_TIMINGS: Mutex<Vec<(&'static str, &'static str, f64)>> = Mutex::new(Vec::new());

/// Starts `stage`'s CPU clock (its step begins) when [`lap_cpu_on`].
pub fn cpu_mark(stage: &'static str) {
    if !lap_cpu_on() {
        return;
    }
    let now = process_cpu_ms();
    if let Ok(mut last) = CPU_LAST.lock() {
        match last.iter_mut().find(|e| e.0 == stage) {
            Some(e) => e.1 = now,
            None => last.push((stage, now)),
        }
    }
}

/// Adds the CPU since `stage`'s last mark or part to `part` when [`lap_cpu_on`].
pub fn cpu_part(stage: &'static str, part: &'static str) {
    if !lap_cpu_on() {
        return;
    }
    let now = process_cpu_ms();
    let Ok(mut last) = CPU_LAST.lock() else {
        return;
    };
    let Some(e) = last.iter_mut().find(|e| e.0 == stage) else {
        last.push((stage, now));
        return;
    };
    let ms = now - e.1;
    e.1 = now;
    if let Ok(mut t) = CPU_TIMINGS.lock() {
        match t.iter_mut().find(|x| x.0 == stage && x.1 == part) {
            Some(x) => x.2 += ms,
            None => t.push((stage, part, ms)),
        }
    }
}

/// The parts' CPU sums (`stage`, `part`, ms), in the order they first ran.
pub fn cpu_timings() -> Vec<(&'static str, &'static str, f64)> {
    CPU_TIMINGS.lock().map(|t| t.clone()).unwrap_or_default()
}

/// The parts and their sums, in the order they first ran.
pub fn timings() -> Vec<(&'static str, f64)> {
    TIMINGS.lock().map(|t| t.clone()).unwrap_or_default()
}

/// Clears [`timings`] (a new run in the same process).
pub fn reset_timings() {
    if let Ok(mut t) = TIMINGS.lock() {
        t.clear();
    }
}

/// The device `kernel` uses under `mode`, or None for the CPU; logs the choice.
#[cfg(feature = "gpu")]
pub fn device(
    mode: ComputeMode,
    log: &LogRing,
    kernel: &'static str,
) -> Option<std::sync::Arc<deeptime_gpu::Gpu>> {
    if mode == ComputeMode::Cpu {
        return None;
    }
    if !kernel_allowed(kernel) {
        log.push(
            Level::Info,
            format!("compute: {kernel} runs on the CPU ({ENV_KERNELS} leaves it out)"),
        );
        return None;
    }
    match deeptime_gpu::shared(mode == ComputeMode::Gpu) {
        Ok(g) => {
            log.push(
                Level::Info,
                format!("compute: {} on {}", mode.as_str(), g.describe()),
            );
            Some(g)
        }
        Err(e) => {
            let level = if mode == ComputeMode::Gpu {
                Level::Warn
            } else {
                Level::Info
            };
            log.push(
                level,
                format!("compute: {} runs on the CPU ({e})", mode.as_str()),
            );
            None
        }
    }
}

/// Without the `gpu` feature everything runs on the CPU; a forced `gpu` is warned about.
#[cfg(not(feature = "gpu"))]
pub fn warn_without_gpu(mode: ComputeMode, log: &LogRing) {
    if mode == ComputeMode::Gpu {
        log.push(
            Level::Warn,
            "compute: gpu asked for, but this build has no GPU support; running on the CPU",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_parse_and_print() {
        for m in [ComputeMode::Auto, ComputeMode::Cpu, ComputeMode::Gpu] {
            assert_eq!(ComputeMode::parse(m.as_str()), Some(m));
        }
        assert_eq!(ComputeMode::parse(" GPU "), Some(ComputeMode::Gpu));
        assert_eq!(ComputeMode::parse("cuda"), None);
        assert_eq!(ComputeMode::default(), ComputeMode::Auto);
    }
}

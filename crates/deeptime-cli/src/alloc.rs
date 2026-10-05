// SPDX-License-Identifier: LGPL-3.0-or-later
//! The CLI's counting global allocator (plan §5.7.5; `deeptime_core::alloc`, as in `deeptime-jni`:
//! a global allocator must be declared by the final binary), so a run's memory cap works as in
//! the game and `deeptime bench` can report each run's peak.
//!
//! `DEEPTIME_MEM_TRACE=<file>` traces memory per phase of a run (each stage's step, keyframes,
//! the bake; [`MemoryGauge::phase`]): one JSON line per phase with the live bytes' peak in it, the
//! process's resident peak (Linux: `VmHWM`, reset at the phase's start), its resident size at the
//! end and, on glibc, what the allocator holds from the system and how much of that is free; `run`
//! prints a per-phase summary. `DEEPTIME_MALLOC_TRIM=1` also returns glibc's free memory to the
//! system after each phase (an experiment: it says how much of the resident size is free memory
//! the allocator kept).

use std::sync::atomic::{AtomicU64, Ordering};

use deeptime_core::alloc::{self as counts, CountingAlloc};
use deeptime_core::session::MemoryGauge;

pub use deeptime_core::alloc::{in_use, peak, reset_peak};

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// `DEEPTIME_LARGE_ALLOC=<bytes>|off` and `DEEPTIME_LARGE_CACHE=<bytes>` (experiments): the size
/// from which blocks are mapped on their own and how many bytes of freed ones are kept for reuse
/// (`deeptime_core::alloc::LARGE_BYTES`, `CACHE_BYTES` by default). Read before anything large is
/// allocated.
pub fn init_from_env() {
    if let Some(b) = std::env::var_os("DEEPTIME_LARGE_CACHE")
        .and_then(|v| v.to_string_lossy().parse::<usize>().ok())
    {
        counts::set_cache_limit(b);
    }
    if let Some(v) = std::env::var_os("DEEPTIME_LARGE_ALLOC") {
        let v = v.to_string_lossy();
        if v == "off" {
            counts::set_large_threshold(None);
        } else if let Ok(b) = v.parse::<usize>() {
            counts::set_large_threshold(Some(b));
        }
    }
}

/// The allocator as the core's [`MemoryGauge`].
pub struct AllocGauge;

impl MemoryGauge for AllocGauge {
    fn in_use(&self) -> u64 {
        in_use()
    }
    fn peak(&self) -> u64 {
        peak()
    }
    fn phase(&self, name: &str, begin: bool) {
        if begin {
            counts::start_window();
        }
        if let Some(t) = trace() {
            t.phase(name, begin);
        }
    }
    fn phase_peak(&self) -> u64 {
        counts::window_peak()
    }
    fn release(&self) -> u64 {
        counts::release_cache()
    }
    fn keep_below(&self, bytes: u64) {
        counts::keep_below(bytes);
    }
    fn reserved(&self, needed: u64, granted: bool) {
        if trace().is_some() {
            RESERVED.fetch_max(needed, Ordering::Relaxed);
            if !granted {
                REFUSED.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// The most a reservation asked for (in use + reserved) in the traced phase, and how many were
/// refused.
static RESERVED: AtomicU64 = AtomicU64::new(0);
static REFUSED: AtomicU64 = AtomicU64::new(0);

/// One phase's figures over all its occurrences.
#[derive(Default, Clone)]
pub struct PhaseAgg {
    pub count: u64,
    pub seconds: f64,
    /// The most live bytes during the phase.
    pub live_peak: u64,
    /// The process's resident peak during the phase (0 where not measurable).
    pub rss_peak: u64,
    /// The resident size at the phase's end.
    pub rss_end: u64,
    /// The most a reservation asked for during the phase (in use + reserved; 0: none).
    pub reserved: u64,
}

struct Tracer {
    out: std::sync::Mutex<std::fs::File>,
    started: std::time::Instant,
    phases: std::sync::Mutex<Vec<(String, PhaseAgg)>>,
    begun: std::sync::Mutex<Option<std::time::Instant>>,
    trim: bool,
}

fn trace() -> Option<&'static Tracer> {
    static TRACE: std::sync::OnceLock<Option<Tracer>> = std::sync::OnceLock::new();
    TRACE
        .get_or_init(|| {
            let path = std::env::var_os("DEEPTIME_MEM_TRACE")?;
            let file = std::fs::File::create(&path).ok()?;
            Some(Tracer {
                out: std::sync::Mutex::new(file),
                started: std::time::Instant::now(),
                phases: std::sync::Mutex::new(Vec::new()),
                begun: std::sync::Mutex::new(None),
                trim: std::env::var_os("DEEPTIME_MALLOC_TRIM").is_some_and(|v| v == "1"),
            })
        })
        .as_ref()
}

/// `(VmRSS, VmHWM)` in bytes, from `/proc/self/status` (0 where there is none).
fn rss() -> (u64, u64) {
    let Ok(s) = std::fs::read_to_string("/proc/self/status") else {
        return (0, 0);
    };
    let kb = |key: &str| {
        s.lines()
            .find_map(|l| l.strip_prefix(key))
            .and_then(|v| v.split_whitespace().next()?.parse::<u64>().ok())
            .map_or(0, |v| v * 1024)
    };
    (kb("VmRSS:"), kb("VmHWM:"))
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
mod glibc {
    #[repr(C)]
    pub struct MallInfo2 {
        pub arena: usize,
        pub ordblks: usize,
        pub smblks: usize,
        pub hblks: usize,
        pub hblkhd: usize,
        pub usmblks: usize,
        pub fsmblks: usize,
        pub uordblks: usize,
        pub fordblks: usize,
        pub keepcost: usize,
    }
    unsafe extern "C" {
        fn mallinfo2() -> MallInfo2;
        fn malloc_trim(pad: usize) -> i32;
    }
    /// `(bytes the allocator holds from the system, bytes of that free)`.
    pub fn held() -> (u64, u64) {
        // SAFETY: glibc's mallinfo2 takes no arguments and returns a plain struct.
        let m = unsafe { mallinfo2() };
        ((m.arena + m.hblkhd) as u64, m.fordblks as u64)
    }
    pub fn trim() {
        // SAFETY: malloc_trim only releases free memory to the system.
        unsafe { malloc_trim(0) };
    }
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
mod glibc {
    pub fn held() -> (u64, u64) {
        (0, 0)
    }
    pub fn trim() {}
}

impl Tracer {
    fn phase(&self, name: &str, begin: bool) {
        if begin {
            RESERVED.store(0, Ordering::Relaxed);
            REFUSED.store(0, Ordering::Relaxed);
            // Resets VmHWM to the current resident size (Linux 4.0+).
            let _ = std::fs::write("/proc/self/clear_refs", "5");
            if let Ok(mut b) = self.begun.lock() {
                *b = Some(std::time::Instant::now());
            }
            return;
        }
        let seconds = self
            .begun
            .lock()
            .ok()
            .and_then(|mut b| b.take())
            .map_or(0.0, |b| b.elapsed().as_secs_f64());
        let live_peak = counts::window_peak();
        let reserved = RESERVED.load(Ordering::Relaxed);
        let refused = REFUSED.load(Ordering::Relaxed);
        let (rss_end, rss_peak) = rss();
        let (held, free) = glibc::held();
        let trimmed_rss = if self.trim {
            glibc::trim();
            rss().0
        } else {
            0
        };
        let line = format!(
            "{{\"t\":{:.3},\"phase\":\"{name}\",\"s\":{seconds:.4},\"in_use\":{},\"live_peak\":{live_peak},\"rss_peak\":{rss_peak},\"rss\":{rss_end},\"malloc_held\":{held},\"malloc_free\":{free},\"rss_after_trim\":{trimmed_rss},\"reserved\":{reserved},\"refused\":{refused}}}\n",
            self.started.elapsed().as_secs_f64(),
            in_use(),
        );
        if let Ok(mut f) = self.out.lock() {
            use std::io::Write;
            let _ = f.write_all(line.as_bytes());
        }
        if let Ok(mut phases) = self.phases.lock() {
            let i = match phases.iter().position(|(n, _)| n == name) {
                Some(i) => i,
                None => {
                    phases.push((name.to_owned(), PhaseAgg::default()));
                    phases.len() - 1
                }
            };
            let a = &mut phases[i].1;
            a.count += 1;
            a.seconds += seconds;
            a.live_peak = a.live_peak.max(live_peak);
            a.rss_peak = a.rss_peak.max(rss_peak);
            a.rss_end = a.rss_end.max(rss_end);
            a.reserved = a.reserved.max(reserved);
        }
    }
}

/// The traced phases in the order they first ran (empty without `DEEPTIME_MEM_TRACE`).
pub fn trace_summary() -> Vec<(String, PhaseAgg)> {
    trace()
        .and_then(|t| t.phases.lock().ok().map(|p| p.clone()))
        .unwrap_or_default()
}

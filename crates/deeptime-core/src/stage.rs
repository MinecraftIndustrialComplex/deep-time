// SPDX-License-Identifier: LGPL-3.0-or-later
//! The `Stage` trait of plan §8 and what a stage sees while it steps.
//!
//! A stage is one module of the pipeline (plates, surface, climate, …; the dummy stages until
//! P1-03). The session steps it in small increments so `advance` honours its time budget and
//! cancellation between steps; the stage reads and writes the shared [`WorldState`], emits events
//! into it, and names the fields the session renders as keyframes. Its private state (counters,
//! solver scratch that must survive a resume) round-trips through `save_state`/`load_state`, which
//! checkpoints store with postcard (`checkpoint.rs`).
//!
//! Deviation from the plan's sketch (an implementation detail, plan §12): `checkpoint(&self, w)`
//! is split into `save_state`/`load_state` so the session builds the checkpoint (the world, the
//! position in the pipeline and the current stage's state) and a stage never serialises the
//! shared world itself; `step` takes a [`StageContext`] carrying the deadline, the Rayon pool,
//! the log and the memory cap instead of `(until, pool)`.

use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Instant;

use crate::deposits::recorded::HistoryHandle;
use crate::error::CoreError;
use crate::log::LogRing;
use crate::session::{MemoryGauge, SessionState};
use crate::world::WorldState;

/// A stage's memory and time estimate (plan §5.7.5; P1-17 calibrates the cost models).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StageEstimate {
    /// Peak native bytes while the stage runs with everything in memory, shared world state
    /// included.
    pub bytes: u64,
    /// Wall-clock seconds on one core (a design budget until measured).
    pub seconds: f64,
    /// The part of `bytes` that is the recorded history (P1-04's frames): it goes to disk when
    /// the memory cap needs the room (P1-17), so only `bytes - history_bytes` must fit the cap.
    pub history_bytes: u64,
}

/// What a stage sees while it steps.
pub struct StageContext<'a> {
    /// The session's private Rayon pool; run data-parallel kernels with `pool.install(..)`.
    pub pool: &'a rayon::ThreadPool,
    /// The session's log ring.
    pub log: &'a LogRing,
    /// The world-data folder the session writes (`SessionOptions::out_dir`): where a stage that
    /// spills tiles to disk puts them (the L3 amplifier, plan §5.7.5).
    pub out_dir: &'a std::path::Path,
    /// Return from `step` by about this instant (a stage doing one short step per call may ignore
    /// it: the session checks the budget between calls).
    pub until: Instant,
    pub(crate) gauge: &'a dyn MemoryGauge,
    pub(crate) mem_cap: u64,
    pub(crate) state: &'a AtomicU8,
    /// The world's recorded history, whose frames in memory give way when a reservation would
    /// not fit (P1-17; `None` before the history exists).
    pub(crate) history: Option<&'a HistoryHandle>,
    /// The most any reservation asked for, `[with, without]` the history's frames in memory
    /// (bytes in use + bytes reserved), for the session's report of what the run needed of its
    /// cap.
    pub(crate) needed_peak: Option<&'a [AtomicU64; 2]>,
}

impl StageContext<'_> {
    /// Checks, before allocating, that `bytes` more would stay within the memory cap. When they
    /// would not, the recorded history's oldest frames in memory go to the session's spill file
    /// first, if that makes room (decisions log 2026-09-29: a run over its cap slows down instead
    /// of failing); only what does not fit even then fails.
    pub fn reserve(&self, bytes: u64) -> Result<(), CoreError> {
        let needed = self.gauge.in_use().saturating_add(bytes);
        if let Some(p) = self.needed_peak {
            p[0].fetch_max(needed, Ordering::Relaxed);
            let history = self.history.map_or(0, HistoryHandle::resident_bytes);
            p[1].fetch_max(needed.saturating_sub(history), Ordering::Relaxed);
        }
        if needed <= self.mem_cap {
            return Ok(());
        }
        if let Some(h) = self.history {
            let over = needed - self.mem_cap;
            if h.resident_bytes() >= over {
                let e = h.evict(over)?;
                crate::session::log_evicted(
                    self.log,
                    &e,
                    self.gauge,
                    self.mem_cap,
                    "a reservation",
                );
                let needed = self.gauge.in_use().saturating_add(bytes);
                if needed <= self.mem_cap {
                    return Ok(());
                }
                return Err(CoreError::MemoryCap {
                    needed,
                    cap: self.mem_cap,
                });
            }
        }
        Err(CoreError::MemoryCap {
            needed,
            cap: self.mem_cap,
        })
    }

    /// True once the session was cancelled; long steps poll it and return early.
    pub fn cancelled(&self) -> bool {
        self.state.load(Ordering::Acquire) == SessionState::Cancelled as u8
    }

    /// True while the step's deadline has not passed.
    pub fn time_left(&self) -> bool {
        Instant::now() < self.until
    }
}

/// One stage of the pipeline (plan §8).
pub trait Stage: Send {
    /// Unique within a pipeline; `deeptime stage <name>` selects the stage by it.
    fn name(&self) -> &str;

    /// Memory and time for this stage, from the parameters it was built with.
    fn estimate(&self) -> StageEstimate;

    fn total_steps(&self) -> u64;

    fn steps_done(&self) -> u64;

    fn is_done(&self) -> bool {
        self.steps_done() >= self.total_steps()
    }

    /// Runs at least one step (more while `ctx.time_left()`). May allocate only after
    /// `ctx.reserve`. Advances `world.t_years` as simulated time passes.
    fn step(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError>;

    /// Names of `world.fields` the session renders into keyframes while this stage runs.
    fn keyframe_layers(&self) -> &[&'static str] {
        &[]
    }

    /// The stage this one is interleaved with (plan §8: "plates and deep-time surface processes
    /// interleaved per plate step"), if any. A stage that names the stage just before it in the
    /// pipeline (or a stage coupled to that one) joins its *group*: the session steps the group's
    /// lead once, then every follower once, in pipeline order, until all of them are done. The
    /// lead must take exactly one step per `step` call. Keyframes of a group draw the union of its
    /// members' layers; a checkpoint taken inside a group stores every member's state.
    fn coupled_to(&self) -> Option<&'static str> {
        None
    }

    /// The stage's private state (postcard bytes), for a checkpoint taken mid-stage.
    fn save_state(&self) -> Result<Vec<u8>, CoreError>;

    /// Restores what `save_state` produced.
    fn load_state(&mut self, bytes: &[u8]) -> Result<(), CoreError>;
}

/// Serialises a stage's private state with postcard.
pub fn to_state<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, CoreError> {
    postcard::to_stdvec(value)
        .map_err(|e| CoreError::InvalidState(format!("stage state serialisation: {e}")))
}

/// Deserialises what [`to_state`] produced.
pub fn from_state<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, CoreError> {
    postcard::from_bytes(bytes)
        .map_err(|e| CoreError::InvalidState(format!("stage state does not decode: {e}")))
}

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

use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Instant;

use crate::error::CoreError;
use crate::log::LogRing;
use crate::session::{MemoryGauge, SessionState};
use crate::world::WorldState;

/// A stage's memory and time estimate (plan §5.7.5; P1-17 calibrates the cost models).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StageEstimate {
    /// Peak native bytes while the stage runs, shared world state included.
    pub bytes: u64,
    /// Wall-clock seconds on one core (a design budget until measured).
    pub seconds: f64,
}

/// What a stage sees while it steps.
pub struct StageContext<'a> {
    /// The session's private Rayon pool; run data-parallel kernels with `pool.install(..)`.
    pub pool: &'a rayon::ThreadPool,
    /// The session's log ring.
    pub log: &'a LogRing,
    /// Return from `step` by about this instant (a stage doing one short step per call may ignore
    /// it: the session checks the budget between calls).
    pub until: Instant,
    pub(crate) gauge: &'a dyn MemoryGauge,
    pub(crate) mem_cap: u64,
    pub(crate) state: &'a AtomicU8,
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

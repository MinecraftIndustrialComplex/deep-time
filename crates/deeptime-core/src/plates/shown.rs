// SPDX-License-Identifier: LGPL-3.0-or-later
//! The crust shown again (measurement only; the front lane, 2026-10-24; `docs/science/plates.md`,
//! "The overlaps' hiding" and "Step 6: the crust shown again").
//!
//! `subduction::resolve` clears every sample's flags each step and hides exactly the subducting
//! samples that lie under an overriding plate's cover, so a sample that went down a trench shows
//! again whenever it leaves that cover before the next resampling consumes it. One-way subduction
//! (a sample once hidden stays hidden until the resampling) would consume that crust instead. This
//! counts what it would change: per step, the samples that went under and those shown again, by
//! cause ([`CAUSES`]); at each resampling, the hidden samples it consumes and the samples that
//! went under since the last resampling and stand visible (escaped), by the cause of their last
//! showing, with the ages of the escaped ocean floor against the visible ocean floor's.
//!
//! The causes, the first that holds:
//! - `plate`: the sample's plate is not the one it went under with (a rift split it off).
//! - `reversal`: it lies under a plate whose contact with its own reversed this step, so its
//!   plate now overrides.
//! - `on_top`: it lies under a plate whose contact with its own has its plate overriding (the
//!   contact reversed earlier, or the sample went under another contact).
//! - `unseen`: it lies under a plate with which its own has no contact this step (a new pair
//!   with fewer than `MIN_CONTACT` overlaps).
//! - `left`: it lies under no plate's cover (the overriding plate moved off it, or it lies under
//!   a seam).
//! - `collision`: a terrane transfer or a weld brought it up (`collision.rs`).
//!
//! Nothing a plate rule reads: it lives beside the dynamics record (on while that is on) and the
//! record test runs the planet with and without it.

use serde::{Deserialize, Serialize};

use super::state::{CONTINENTAL, OCEANIC, PlateState};

/// The causes of a showing, in the order they are judged.
pub const CAUSES: [&str; 6] = ["plate", "reversal", "on_top", "unseen", "left", "collision"];
pub const NC: usize = CAUSES.len();
pub const PLATE: usize = 0;
pub const REVERSAL: usize = 1;
pub const ON_TOP: usize = 2;
pub const UNSEEN: usize = 3;
pub const LEFT: usize = 4;
pub const COLLISION: usize = 5;

/// No plate: the sample has not gone under since the last resampling.
const NONE: u32 = u32::MAX;

/// One step's counts (`StepRec::shown`), in samples; ages summed in Myr.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Step {
    /// Samples visible as `resolve` began and hidden after it: the crust that went under.
    pub under: u32,
    /// Samples hidden as `resolve` began and visible after it, or brought up by a collision, by
    /// cause; and the continental ones.
    pub shown: [u32; NC],
    pub shown_cont: [u32; NC],
    /// At a resampling (0 otherwise): the hidden samples it consumes, the continental ones and
    /// the oceanic ones' Σ age.
    pub consumed: u32,
    pub consumed_cont: u32,
    pub consumed_age_myr: f64,
    /// … the visible samples that went under since the last resampling, by the cause of their
    /// last showing; the continental ones; the oceanic ones and their Σ age.
    pub escaped: [u32; NC],
    pub escaped_cont: [u32; NC],
    pub escaped_ocean: u32,
    pub escaped_age_myr: f64,
    /// … every visible oceanic sample and its Σ age.
    pub ocean: u32,
    pub ocean_age_myr: f64,
}

/// What the measure keeps between steps (`PlateState::shown`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Track {
    /// Per sample: the plate it was on when it was last hidden, [`NONE`] if it has not gone under
    /// since the last resampling.
    went: Vec<u32>,
    /// Per sample: 1 + the cause of its last showing, 0 while hidden or never shown.
    cause: Vec<u8>,
    /// This step's counts, taken by `dynamics::record_step`.
    pub step: Step,
}

impl Track {
    fn new(n: usize) -> Self {
        Self {
            went: vec![NONE; n],
            cause: vec![0; n],
            step: Step::default(),
        }
    }
}

/// The samples hidden as `resolve` begins, when the measure is on (with the dynamics record); it
/// starts the track if there is none for these samples.
pub fn begin(state: &mut PlateState) -> Option<Vec<bool>> {
    state.dynamics.as_ref()?;
    let n = state.samples.len();
    if state.shown.as_ref().is_none_or(|t| t.went.len() != n) {
        state.shown = Some(Track::new(n));
    }
    Some(state.samples.iter().map(|s| s.is_hidden()).collect())
}

/// After `resolve` has hidden this step's samples: counts what went under and what showed again.
/// `cover[i]` is the best cause sample `i`'s overlaps give it (`REVERSAL`, `ON_TOP`, `UNSEEN`), or
/// `LEFT` when it lies in none.
pub fn end_resolve(state: &mut PlateState, was: &[bool], cover: &[u8]) {
    let Some(mut t) = state.shown.take() else {
        return;
    };
    for (i, s) in state.samples.iter().enumerate() {
        let now = s.is_hidden();
        let p = state.groups[s.group as usize].plate;
        if now {
            if !was[i] {
                t.step.under += 1;
            }
            t.went[i] = p;
            t.cause[i] = 0;
        } else if was[i] {
            let c = if t.went[i] != NONE && t.went[i] != p {
                PLATE
            } else {
                usize::from(cover[i])
            };
            t.show(i, c, s.kind == CONTINENTAL);
        }
    }
    state.shown = Some(t);
}

impl Track {
    fn show(&mut self, i: usize, c: usize, continental: bool) {
        self.step.shown[c] += 1;
        if continental {
            self.step.shown_cont[c] += 1;
        }
        self.cause[i] = (c + 1) as u8;
    }
}

/// A collision brought hidden sample `i` up (`collision.rs`).
pub fn collided(state: &mut PlateState, i: usize) {
    let continental = state.samples[i].kind == CONTINENTAL;
    if let Some(t) = state.shown.as_mut()
        && t.went.len() == state.samples.len()
    {
        t.show(i, COLLISION, continental);
    }
}

/// Just before a resampling at `t_myr`: what it consumes and what escaped it.
pub fn resampling(state: &mut PlateState, t_myr: f64) {
    let Some(t) = state.shown.as_mut() else {
        return;
    };
    if t.went.len() != state.samples.len() {
        return;
    }
    let st = &mut t.step;
    for (i, s) in state.samples.iter().enumerate() {
        let ocean = s.kind == OCEANIC;
        let cont = s.kind == CONTINENTAL;
        let age = s.age_myr(t_myr);
        if s.is_hidden() {
            st.consumed += 1;
            st.consumed_cont += u32::from(cont);
            if ocean {
                st.consumed_age_myr += age;
            }
            continue;
        }
        if ocean {
            st.ocean += 1;
            st.ocean_age_myr += age;
        }
        if t.went[i] == NONE {
            continue;
        }
        debug_assert!(t.cause[i] > 0, "sample {i} visible with no showing");
        let c = usize::from(t.cause[i].max(1) - 1);
        st.escaped[c] += 1;
        st.escaped_cont[c] += u32::from(cont);
        if ocean {
            st.escaped_ocean += 1;
            st.escaped_age_myr += age;
        }
    }
}

/// After a resampling: the new samples have not gone under.
pub fn resampled(state: &mut PlateState) {
    if let Some(t) = state.shown.as_mut() {
        let step = t.step;
        *t = Track::new(state.samples.len());
        t.step = step;
    }
}

/// This step's counts, cleared for the next (`dynamics::record_step`).
pub fn take_step(state: &mut PlateState) -> Step {
    state
        .shown
        .as_mut()
        .map(|t| std::mem::take(&mut t.step))
        .unwrap_or_default()
}

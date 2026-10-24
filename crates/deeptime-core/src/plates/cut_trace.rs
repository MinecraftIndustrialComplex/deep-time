// SPDX-License-Identifier: LGPL-3.0-or-later
//! The cut trace (the front lane, 2026-10-24; measurement only, off by default): which part of
//! the plate step cuts a plate into pieces. A plate is one connected rigid body; the visible
//! samples of a plate that are outside its largest connected piece (over lattice edges between
//! visible samples of that plate, [`geometry::cut_samples`]) are its cut-off pieces. After each
//! part of the step [`mark`] counts them and books the rise or fall since the last mark to that
//! part, so over a run each part's total says how many samples it cut off their plates and how
//! many it joined back. A process-wide gauge, as the step's timings are: the world a run leaves
//! is the same with or without it. On with [`enable`] (a test) or `DEEPTIME_CUT_TRACE=1`.

use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use super::geometry::{self, Geo};
use super::state::PlateState;

static ENABLED: AtomicBool = AtomicBool::new(false);
static TRACE: Mutex<Trace> = Mutex::new(Trace {
    last: None,
    parts: Vec::new(),
});

struct Trace {
    last: Option<usize>,
    parts: Vec<Part>,
}

/// One part of the step's total over the marks since the last [`take`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Part {
    /// The part's name (the step's timing name, or finer).
    pub what: &'static str,
    /// Samples the part cut off their plate's largest piece, summed over its marks.
    pub cut: u64,
    /// Samples it joined back (or removed while cut off).
    pub joined: u64,
    /// Its marks.
    pub marks: u64,
}

/// Switches the trace on or off for the process.
pub fn enable(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

/// Whether the trace is on.
pub fn on() -> bool {
    static ENV: OnceLock<bool> = OnceLock::new();
    ENABLED.load(Ordering::Relaxed)
        || *ENV.get_or_init(|| std::env::var("DEEPTIME_CUT_TRACE").is_ok_and(|v| v == "1"))
}

/// The parts' totals since the last call, in the order they first marked, and a fresh start.
pub fn take() -> Vec<Part> {
    match TRACE.lock() {
        Ok(mut t) => {
            t.last = None;
            std::mem::take(&mut t.parts)
        }
        Err(_) => Vec::new(),
    }
}

/// The visible samples outside their plate's largest piece now.
pub fn count(st: &PlateState, geo: &Geo) -> usize {
    let plate: Vec<u32> = (0..st.samples.len()).map(|i| st.plate_of(i)).collect();
    geometry::cut_samples(geo, &plate, |i| !st.samples[i].is_hidden())
}

/// Books the change in [`count`] since the last mark to `what` (nothing when the trace is off).
pub fn mark(st: &PlateState, geo: &Geo, what: &'static str) {
    if !on() {
        return;
    }
    let now = count(st, geo);
    let Ok(mut t) = TRACE.lock() else { return };
    let before = t.last.replace(now).unwrap_or(now);
    let i = match t.parts.iter().position(|p| p.what == what) {
        Some(i) => i,
        None => {
            t.parts.push(Part {
                what,
                ..Part::default()
            });
            t.parts.len() - 1
        }
    };
    let p = &mut t.parts[i];
    p.marks += 1;
    if now >= before {
        p.cut += (now - before) as u64;
    } else {
        p.joined += (before - now) as u64;
    }
}

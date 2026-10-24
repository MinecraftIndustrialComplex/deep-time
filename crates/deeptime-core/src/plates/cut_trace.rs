// SPDX-License-Identifier: LGPL-3.0-or-later
//! The cut trace (the front lane, 2026-10-24; measurement only, off by default): which part of
//! the plate step cuts a plate into pieces. A plate is one connected rigid body; the visible
//! samples of a plate that are outside its largest connected piece (over lattice edges between
//! visible samples of that plate, [`geometry::cut_samples`]) are its cut-off pieces. After each
//! part of the step [`mark`] counts them and books the rise or fall since the last mark to that
//! part, so over a run each part's total says how many samples it cut off their plates and how
//! many it joined back. A process-wide gauge, as the step's timings are: the world a run leaves
//! is the same with or without it. On with [`enable`] (a test) or `DEEPTIME_CUT_TRACE=1`.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use super::geometry::{self, Geo};
use super::state::PlateState;

static ENABLED: AtomicBool = AtomicBool::new(false);
static TRACE: Mutex<Trace> = Mutex::new(Trace {
    last: None,
    by_plate: BTreeMap::new(),
    parts: Vec::new(),
});

struct Trace {
    last: Option<usize>,
    by_plate: BTreeMap<u32, usize>,
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
            t.by_plate.clear();
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

/// The visible samples outside their plate's largest piece now, per plate.
pub fn count_by_plate(st: &PlateState, geo: &Geo) -> BTreeMap<u32, usize> {
    let plate: Vec<u32> = (0..st.samples.len()).map(|i| st.plate_of(i)).collect();
    let (_, sizes, keep) = geometry::pieces(geo, &plate, |i| !st.samples[i].is_hidden());
    let mut out: BTreeMap<u32, usize> = BTreeMap::new();
    for (c, &(p, size)) in sizes.iter().enumerate() {
        if keep[&p].1 != c as u32 {
            *out.entry(p).or_default() += size;
        }
    }
    out
}

impl Trace {
    fn book(&mut self, what: &'static str, before: usize, now: usize) {
        let i = match self.parts.iter().position(|p| p.what == what) {
            Some(i) => i,
            None => {
                self.parts.push(Part {
                    what,
                    ..Part::default()
                });
                self.parts.len() - 1
            }
        };
        let p = &mut self.parts[i];
        p.marks += 1;
        if now >= before {
            p.cut += (now - before) as u64;
        } else {
            p.joined += (before - now) as u64;
        }
    }
}

/// Books the change in [`count`] since the last mark to `what` (nothing when the trace is off).
pub fn mark(st: &PlateState, geo: &Geo, what: &'static str) {
    if !on() {
        return;
    }
    let by = count_by_plate(st, geo);
    let now = by.values().sum();
    let Ok(mut t) = TRACE.lock() else { return };
    let before = t.last.replace(now).unwrap_or(now);
    t.by_plate = by;
    t.book(what, before, now);
}

/// As [`mark`], with the change split by plate: plate `a`'s to `what[0]`, plate `b`'s to
/// `what[1]` and every other plate's to `what[2]` (a collision's subducting and overriding plates).
pub fn mark_pair(st: &PlateState, geo: &Geo, what: [&'static str; 3], a: u32, b: u32) {
    if !on() {
        return;
    }
    let by = count_by_plate(st, geo);
    let now: usize = by.values().sum();
    let Ok(mut t) = TRACE.lock() else { return };
    let get = |m: &BTreeMap<u32, usize>, p: u32| m.get(&p).copied().unwrap_or(0);
    let before_all = t.last.unwrap_or(now);
    let (ba, bb) = (get(&t.by_plate, a), get(&t.by_plate, b));
    let (na, nb) = (get(&by, a), get(&by, b));
    t.book(what[0], ba, na);
    t.book(what[1], bb, nb);
    t.book(
        what[2],
        before_all.saturating_sub(ba + bb),
        now.saturating_sub(na + nb),
    );
    t.last = Some(now);
    t.by_plate = by;
}

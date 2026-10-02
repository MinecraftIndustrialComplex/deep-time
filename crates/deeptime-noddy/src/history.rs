// SPDX-License-Identifier: LGPL-3.0-or-later
//
// Ported from Noddy (Loop3D/noddyverse, commit 6a21b23, MIT License, Copyright (c) 2021 Loop3D):
// `src/unEvents.c` (`reverseEvents`) and `src/which.c` (`whichRock`, `whichLayer`). See NOTICE.md.
//! A history and its evaluation at a point.

use crate::V3;
use crate::events::{Event, Step, Unconformity};
use crate::taste::{MAX_EVENTS, Sequence, taste};

/// A layer-cake stratigraphy: flat layers numbered from the bottom, split at heights (z).
///
/// `bases[l]` is the base of layer `l` for l ≥ 1 (Noddy's `properties[l].height`); layer 0 has no
/// base and reaches down for ever, the top layer up for ever. A point exactly on a boundary
/// belongs to the layer below (`whichLayer`'s `z <= height`).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Stratigraphy {
    bases: Vec<f64>,
}

impl Stratigraphy {
    /// Layers split at `boundaries` (ascending): n boundaries make n + 1 layers.
    pub fn new(boundaries: &[f64]) -> Stratigraphy {
        debug_assert!(boundaries.windows(2).all(|w| w[0] <= w[1]));
        let mut bases = Vec::with_capacity(boundaries.len() + 1);
        bases.push(f64::NEG_INFINITY);
        bases.extend_from_slice(boundaries);
        Stratigraphy { bases }
    }

    /// One layer.
    pub fn single() -> Stratigraphy {
        Stratigraphy::new(&[])
    }

    pub fn layers(&self) -> usize {
        self.bases.len().max(1)
    }

    /// The layer holding height `z` (`whichLayer`).
    pub fn layer_at(&self, z: f64) -> usize {
        let n = self.layers();
        if n == 1 || z <= self.bases[1] {
            return 0;
        }
        if z > self.bases[n - 1] {
            return n - 1;
        }
        // The original scans layers 1 … n − 2 and keeps the last match; the boundaries ascend, so
        // there is one.
        let mut found = 0;
        for layer in 1..n - 1 {
            if z > self.bases[layer] && z <= self.bases[layer + 1] {
                found = layer;
            }
        }
        found
    }
}

/// The rock at a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rock {
    /// Layer `layer` (from the bottom) of the stratigraphy of event `event`: 0 is the base
    /// stratigraphy, any other an unconformity's cover.
    Layer { event: usize, layer: usize },
    /// The dyke or plug that is event `event`.
    Igneous { event: usize },
}

/// A point unravelled through a history.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Restored {
    /// Where the point was when its rock formed: in the base stratigraphy's frame, or, for a
    /// point that an event claimed, where it was just after that event.
    pub point: V3,
    pub sequence: Sequence,
    /// The event that claimed the point (above its unconformity, in its dyke or plug); `None`:
    /// the point went all the way back.
    pub stopped_at: Option<usize>,
}

/// A base stratigraphy and the events after it, oldest first. Event indices count the base
/// stratigraphy as 0, as Noddy's event list does.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct History {
    base: Stratigraphy,
    events: Vec<Event>,
    /// The cover of each unconformity (None for other events).
    covers: Vec<Option<Stratigraphy>>,
}

impl History {
    pub fn new(base: Stratigraphy) -> History {
        History {
            base,
            events: Vec::new(),
            covers: Vec::new(),
        }
    }

    /// Appends an event (the youngest so far) and returns its index. An unconformity pushed here
    /// gets a one-layer cover; [`History::push_unconformity`] gives it a layered one.
    ///
    /// # Panics
    /// With more than [`MAX_EVENTS`] − 1 events (the sequence code's size, the original's limit):
    /// ask [`History::is_full`] first, or use a [`Chain`], which has no limit.
    pub fn push(&mut self, event: Event) -> usize {
        assert!(
            !self.is_full(),
            "a history holds at most {MAX_EVENTS} events, the base stratigraphy included"
        );
        let cover = matches!(event, Event::Unconformity(_)).then(Stratigraphy::single);
        self.events.push(event);
        self.covers.push(cover);
        self.events.len()
    }

    /// Whether the history holds as many events as a sequence code has places for
    /// ([`MAX_EVENTS`] − 1, the base stratigraphy taking the first).
    pub fn is_full(&self) -> bool {
        self.events.len() + 1 >= MAX_EVENTS
    }

    /// Appends an unconformity with the stratigraphy laid on it (heights as they are now, after
    /// every later event is undone).
    pub fn push_unconformity(&mut self, unconformity: Unconformity, cover: Stratigraphy) -> usize {
        let i = self.push(Event::Unconformity(unconformity));
        self.covers[i - 1] = Some(cover);
        i
    }

    /// Builder form of [`History::push`].
    pub fn with(mut self, event: Event) -> History {
        self.push(event);
        self
    }

    /// Events after the base stratigraphy.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Events including the base stratigraphy (Noddy's `countObjects`).
    pub fn count(&self) -> usize {
        self.events.len() + 1
    }

    pub fn base(&self) -> &Stratigraphy {
        &self.base
    }

    /// The stratigraphy event `index` defines (the base for 0, an unconformity's cover).
    pub fn stratigraphy(&self, index: usize) -> Option<&Stratigraphy> {
        if index == 0 {
            Some(&self.base)
        } else {
            self.covers.get(index - 1).and_then(Option::as_ref)
        }
    }

    /// Unravels the history for one point, youngest event first (`reverseEvents`).
    pub fn restore(&self, p: V3) -> Restored {
        let mut point = p;
        let mut sequence = Sequence::new();
        let mut stopped_at = None;
        for num in (1..=self.events.len()).rev() {
            let event = &self.events[num - 1];
            if matches!(event, Event::Stop) {
                break;
            }
            let previous_igneous = num >= 2 && self.events[num - 2].is_igneous();
            match event.undo(point, num, previous_igneous, &mut sequence) {
                Step::Moved(q) => point = q,
                Step::Stop => {
                    stopped_at = Some(num);
                    break;
                }
            }
        }
        Restored {
            point,
            sequence,
            stopped_at,
        }
    }

    /// The rock of an unravelled point (`whichRock` after `reverseEvents`).
    pub fn rock_of(&self, r: &Restored) -> Rock {
        let (_, index) = taste(self.count(), &r.sequence);
        if index > 0 {
            match &self.events[index - 1] {
                e if e.is_igneous() => return Rock::Igneous { event: index },
                Event::Unconformity(_) => {
                    let layer = self.covers[index - 1]
                        .as_ref()
                        .map_or(0, |s| s.layer_at(r.point[2]));
                    return Rock::Layer {
                        event: index,
                        layer,
                    };
                }
                _ => {}
            }
        }
        Rock::Layer {
            event: 0,
            layer: self.base.layer_at(r.point[2]),
        }
    }

    /// The rock at a point of the present-day model.
    pub fn rock_at(&self, p: V3) -> Rock {
        self.rock_of(&self.restore(p))
    }

    /// The rock on a regular lattice: `counts` points along x, y and z from `origin`, `step`
    /// apart, x fastest, then y, then z.
    pub fn block(&self, origin: V3, counts: [usize; 3], step: V3) -> Vec<Rock> {
        let mut out = Vec::with_capacity(counts[0] * counts[1] * counts[2]);
        for k in 0..counts[2] {
            for j in 0..counts[1] {
                for i in 0..counts[0] {
                    out.push(self.rock_at([
                        origin[0] + i as f64 * step[0],
                        origin[1] + j as f64 * step[1],
                        origin[2] + k as f64 * step[2],
                    ]));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layers_split_as_noddy_splits_them() {
        let s = Stratigraphy::new(&[100.0, 200.0, 300.0]);
        assert_eq!(s.layers(), 4);
        assert_eq!(s.layer_at(-1.0e9), 0);
        assert_eq!(
            s.layer_at(100.0),
            0,
            "a boundary belongs to the layer below"
        );
        assert_eq!(s.layer_at(100.5), 1);
        assert_eq!(s.layer_at(200.0), 1);
        assert_eq!(s.layer_at(250.0), 2);
        assert_eq!(s.layer_at(300.0), 2);
        assert_eq!(s.layer_at(300.1), 3);
        assert_eq!(Stratigraphy::single().layer_at(5.0), 0);
        assert_eq!(Stratigraphy::new(&[10.0]).layer_at(10.0), 0);
        assert_eq!(Stratigraphy::new(&[10.0]).layer_at(10.1), 1);
    }
}

/// A history of any length, for restoring positions: histories of at most [`MAX_EVENTS`] − 1
/// events each, undone one after the other (Deep Time's addition; the original stops at its
/// sequence code's size).
///
/// Undoing a history is undoing its events youngest first, each from where the last left the
/// point, so cutting the list into runs and undoing the runs youngest first gives the same
/// position, exactly. What a chain does not give is one sequence code or one rock for the whole
/// list: each part has its own ([`Chain::parts`]).
#[derive(Clone, Debug, Default)]
pub struct Chain {
    /// Oldest first.
    parts: Vec<History>,
    events: usize,
}

impl Chain {
    pub fn new() -> Chain {
        Chain::default()
    }

    /// Appends an event (the youngest so far), starting a new part when the last is full.
    pub fn push(&mut self, event: Event) {
        if self.parts.last().is_none_or(History::is_full) {
            self.parts.push(History::new(Stratigraphy::single()));
        }
        if let Some(last) = self.parts.last_mut() {
            last.push(event);
        }
        self.events += 1;
    }

    /// Events in all.
    pub fn count(&self) -> usize {
        self.events
    }

    /// The parts, oldest first.
    pub fn parts(&self) -> &[History] {
        &self.parts
    }

    /// Where the point `p` was before every event.
    pub fn restore(&self, p: V3) -> V3 {
        let mut p = p;
        for part in self.parts.iter().rev() {
            p = part.restore(p).point;
        }
        p
    }
}

#[cfg(test)]
mod chain_tests {
    use super::*;
    use crate::events::{Fault, Fold, Movement};
    use crate::profile::Profile;

    fn event(k: usize) -> Event {
        let f = k as f64;
        if k.is_multiple_of(2) {
            Event::Fault(Fault::translation(
                [3.0 * f - 200.0, 7.0 * f - 300.0, 40.0 + f],
                (37.0 * f) % 360.0,
                30.0 + (f % 50.0),
                90.0,
                1.0 + (f % 7.0),
                Movement::HangingWall,
            ))
        } else {
            Event::Fold(Fold::new(
                [5.0 * f, -3.0 * f, 0.0],
                (53.0 * f) % 360.0,
                90.0,
                0.0,
                60.0 + f,
                2.0 + (f % 5.0),
                0.0,
                false,
                Profile::Sine,
            ))
        }
    }

    #[test]
    fn a_full_history_says_so_and_a_chain_takes_any_number() {
        let mut h = History::new(Stratigraphy::single());
        let mut chain = Chain::new();
        let mut n = 0;
        while !h.is_full() {
            h.push(event(n));
            chain.push(event(n));
            n += 1;
        }
        assert_eq!(n, MAX_EVENTS - 1);
        assert_eq!(chain.parts().len(), 1);
        // Up to the limit the chain is the history.
        for p in [
            [0.0, 0.0, 0.0],
            [120.0, -40.0, 33.0],
            [-300.0, 250.0, -90.0],
        ] {
            assert_eq!(chain.restore(p), h.restore(p).point);
        }
        // Past it: three and a half times as many, cut wherever the parts fill.
        let total = 3 * MAX_EVENTS + 61;
        let mut long = Chain::new();
        for k in 0..total {
            long.push(event(k));
        }
        assert_eq!(long.count(), total);
        assert_eq!(long.parts().len(), total.div_ceil(MAX_EVENTS - 1));
        // The same position as undoing the events one history each, youngest first.
        for p in [
            [0.0, 0.0, 0.0],
            [120.0, -40.0, 33.0],
            [-300.0, 250.0, -90.0],
        ] {
            let mut q = p;
            for k in (0..total).rev() {
                q = History::new(Stratigraphy::single())
                    .with(event(k))
                    .restore(q)
                    .point;
            }
            assert_eq!(long.restore(p), q);
            assert!(q.iter().all(|v| v.is_finite()));
        }
    }
}

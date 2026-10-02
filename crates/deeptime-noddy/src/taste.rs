// SPDX-License-Identifier: LGPL-3.0-or-later
//
// Ported from Noddy (Loop3D/noddyverse, commit 6a21b23, MIT License, Copyright (c) 2021 Loop3D):
// `src/taste.c` (`taste`, `S2Bits`, `G2Bits`, `coincide`, `lineCoincide`, `cypherDiff`,
// `lastdiff`, `izero`, `iequal`) and the stratigraphy codes of `src/nodStruc.h`. See NOTICE.md.
//! The sequence code of a point and what it says about the point's rock.
//!
//! While a point is unravelled, every event that says something discrete about it writes a
//! 4-bit [`Code`] at its own index: above an unconformity, inside a dyke or plug, on the moved
//! side of a fault. The codes of all events are the point's [`Sequence`] (the original's
//! `cypher`, 64 bytes of two codes each). Two points with the same sequence lie in the same
//! fault block and the same package of rock; [`taste`] reads from a sequence which event defines
//! the point's rock.

/// What an event recorded about a point (`*_STRAT` in `nodStruc.h`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Code {
    /// Nothing: the base stratigraphy as far as this event is concerned.
    Base = 0x0,
    /// Above this unconformity: its cover.
    Unconformity = 0x1,
    /// On the pole side of a planar fault, inside a ring fault, or on the side a dilation dyke
    /// pushed away.
    Fault1 = 0x2,
    /// Inside this dyke or plug.
    Igneous = 0x3,
    /// Below a curved or elliptical fault, inside its ellipsoid.
    Fault2 = 0x4,
    /// Above a curved or elliptical fault, inside its ellipsoid.
    Fault3 = 0x8,
}

/// `ARRAY_LENGTH_OF_STRAT_CODE` × `NUM_STRAT_CODE_IN_BYTE`: events a sequence can hold.
pub const MAX_EVENTS: usize = 128; // ledger: DEF-068

const FAULT1: u8 = Code::Fault1 as u8;
const FAULT2: u8 = Code::Fault2 as u8;
const FAULT3: u8 = Code::Fault3 as u8;

/// A point's codes, one nibble per event of the history (`STORY.sequence`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Sequence {
    words: [u64; MAX_EVENTS / 16],
}

impl Sequence {
    /// All zero (`izero`).
    pub const fn new() -> Sequence {
        Sequence {
            words: [0; MAX_EVENTS / 16],
        }
    }

    /// ORs `code` into event `place`'s nibble (`S2Bits`). Places past [`MAX_EVENTS`] are ignored.
    #[inline]
    pub fn set(&mut self, place: usize, code: Code) {
        if place < MAX_EVENTS {
            self.words[place / 16] |= u64::from(code as u8) << (4 * (place % 16));
        }
    }

    /// Event `place`'s raw nibble (`G2Bits`).
    #[inline]
    pub fn get(&self, place: usize) -> u8 {
        if place < MAX_EVENTS {
            ((self.words[place / 16] >> (4 * (place % 16))) & 0xf) as u8
        } else {
            0
        }
    }

    /// The sequence as the original's bytes (two codes a byte, the lower event in the low
    /// nibble), for comparing with the reference.
    pub fn bytes(&self) -> [u8; MAX_EVENTS / 2] {
        let mut out = [0u8; MAX_EVENTS / 2];
        for (i, b) in out.iter_mut().enumerate() {
            *b = (self.words[i / 8] >> (8 * (i % 8))) as u8;
        }
        out
    }

    /// `coincide`: the same sequence.
    pub fn coincide(&self, other: &Sequence) -> bool {
        self == other
    }

    /// `lineCoincide`: the same sequence as far as drawing a boundary between two points goes.
    /// Curved and elliptical faults mark both of their sides (`Fault2`, `Fault3`), so a point
    /// inside the fault's ellipsoid and one outside it differ in code without a discontinuity
    /// between them; only `Fault2` against `Fault3`, or a difference that involves neither,
    /// counts.
    pub fn line_coincide(&self, other: &Sequence) -> bool {
        !differs(self, other)
    }

    /// `lastdiff`: the youngest event (searching back from `events − 1`, never event 0) at which
    /// the two sequences differ in a way that matters (a `Fault2` or `Fault3` against nothing
    /// does not: see [`Sequence::line_coincide`]). 0 when there is none.
    pub fn last_diff(&self, other: &Sequence, events: usize) -> usize {
        for i in (1..events.min(MAX_EVENTS)).rev() {
            let (a, b) = (self.get(i), other.get(i));
            if a != b {
                let skip = (a == FAULT2 && b == 0)
                    || (b == FAULT2 && a == 0)
                    || (a == FAULT3 && b == 0)
                    || (b == FAULT3 && a == 0);
                if !skip {
                    return i;
                }
            }
        }
        0
    }
}

fn differs(a: &Sequence, b: &Sequence) -> bool {
    let mut count = false;
    for place in 0..MAX_EVENTS {
        let (v1, v2) = (a.get(place), b.get(place));
        if v1 == v2 {
            continue;
        }
        if (v1 | v2) == (FAULT2 | FAULT3)
            || (v1 != FAULT2 && v2 != FAULT3 && v2 != FAULT2 && v1 != FAULT3)
        {
            count = true;
        }
    }
    count
}

/// `cypherDiff`: whether sequences `s1` and `s2` of a table differ (as [`Sequence::line_coincide`]
/// decides); the same index never differs.
pub fn cypher_diff(s1: usize, s2: usize, cypher: &[Sequence]) -> bool {
    s1 != s2 && differs(&cypher[s1], &cypher[s2])
}

/// `taste`: which event defines the rock of a point with this sequence, and how.
///
/// Returns `(flavor, index)`: `index` is the youngest event with a code that is not a fault's
/// (faults only move rock, so the search passes over them), or 0, the base stratigraphy; `flavor`
/// is that event's raw code (0 base, 1 above an unconformity, 3 igneous; a fault code only when
/// event 0 itself carries one, which no history built here does).
pub fn taste(events: usize, cypher: &Sequence) -> (u8, usize) {
    let mut index = events as isize - 1;
    loop {
        if index < 0 {
            index = 0;
        }
        if cypher.get(index as usize) == 0 && index > 0 {
            loop {
                index -= 1;
                if !(cypher.get(index as usize) == 0 && index > 0) {
                    break;
                }
            }
        }
        let code = cypher.get(index as usize);
        if (code == FAULT1 || code == FAULT2 || code == FAULT3) && index > 0 {
            index -= 1;
            continue;
        }
        break;
    }
    (cypher.get(index as usize), index as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_round_trip_and_pack_like_the_original() {
        let mut s = Sequence::new();
        s.set(0, Code::Unconformity);
        s.set(1, Code::Igneous);
        s.set(5, Code::Fault3);
        s.set(127, Code::Fault1);
        assert_eq!(s.get(0), 1);
        assert_eq!(s.get(1), 3);
        assert_eq!(s.get(5), 8);
        assert_eq!(s.get(127), 2);
        assert_eq!(s.get(2), 0);
        let b = s.bytes();
        assert_eq!(b[0], 0x31);
        assert_eq!(b[2], 0x80);
        assert_eq!(b[63], 0x20);
        s.set(500, Code::Fault1);
        assert_eq!(s.get(500), 0);
    }

    #[test]
    fn taste_passes_over_faults() {
        // base, fault, unconformity, fault, fold: a point above the unconformity and on the moved
        // side of the last fault tastes of the unconformity.
        let mut s = Sequence::new();
        s.set(1, Code::Fault1);
        s.set(2, Code::Unconformity);
        s.set(3, Code::Fault1);
        assert_eq!(taste(5, &s), (1, 2));
        // Only faults: the base stratigraphy.
        let mut s = Sequence::new();
        s.set(1, Code::Fault1);
        s.set(3, Code::Fault3);
        assert_eq!(taste(5, &s), (0, 0));
        // Nothing at all, and a history of one event.
        assert_eq!(taste(5, &Sequence::new()), (0, 0));
        assert_eq!(taste(1, &Sequence::new()), (0, 0));
        assert_eq!(taste(0, &Sequence::new()), (0, 0));
        // A plug under a later fault.
        let mut s = Sequence::new();
        s.set(2, Code::Igneous);
        s.set(4, Code::Fault2);
        assert_eq!(taste(5, &s), (3, 2));
    }

    #[test]
    fn line_coincide_ignores_one_sided_ellipsoid_codes() {
        let mut a = Sequence::new();
        let mut b = Sequence::new();
        a.set(3, Code::Fault2);
        assert!(a.line_coincide(&b), "inside against outside the ellipsoid");
        assert!(!a.coincide(&b));
        b.set(3, Code::Fault3);
        assert!(!a.line_coincide(&b), "the two sides of the fault");
        assert_eq!(a.last_diff(&b, 6), 3);
        let mut c = Sequence::new();
        c.set(2, Code::Unconformity);
        assert!(!c.line_coincide(&Sequence::new()));
        assert_eq!(c.last_diff(&Sequence::new(), 6), 2);
        assert_eq!(a.last_diff(&Sequence::new(), 6), 0);
        assert!(cypher_diff(0, 1, &[a, b]));
        assert!(!cypher_diff(1, 1, &[a, b]));
    }
}

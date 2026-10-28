// SPDX-License-Identifier: LGPL-3.0-or-later
//! A monotone priority queue on `u64` keys for the floods (the speed work of 2026-10-28): the
//! radix heap of Ahuja, Mehlhorn, Orlin & Tarjan (1990, "Faster algorithms for the shortest path
//! problem", J. ACM 37(2):213-223), written from its description. A Priority-Flood pops cells in
//! non-decreasing order of height, and every cell it pushes stands above the one it came from, so
//! the keys it pushes are never below the last it popped; the queue keeps its items in 65
//! buckets by the highest bit in which a key differs from the last popped, and only moves a
//! bucket's items down when the buckets below it are empty. Each item moves a few times at most,
//! in sequential runs, where a binary heap's every push and pop walks a path of cache misses.
//!
//! With distinct keys, every exact priority queue pops the same sequence, so a flood keyed by
//! (height, cell) gives the binary heap's result bit for bit. A key below the last popped (a NaN
//! height, whose sign bit x86 sets) goes to a small binary heap that pops first, which keeps the
//! order exact for any keys.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// `f32` bits whose unsigned order is [`f32::total_cmp`]'s.
#[inline]
pub(crate) fn ordered(x: f32) -> u32 {
    let b = x.to_bits();
    if b >> 31 == 1 { !b } else { b | 0x8000_0000 }
}

/// [`ordered`] undone.
#[inline]
pub(crate) fn unordered(o: u32) -> f32 {
    f32::from_bits(if o >> 31 == 1 { o & 0x7fff_ffff } else { !o })
}

/// The least key first; ties in any order. See the module note.
pub(crate) struct RadixHeap<T> {
    last: u64,
    len: usize,
    buckets: Vec<Vec<(u64, T)>>,
    /// Keys pushed below `last` (never, in a flood over finite heights).
    below: BinaryHeap<Reverse<(u64, T)>>,
}

impl<T: Copy + Ord> RadixHeap<T> {
    pub(crate) fn new() -> RadixHeap<T> {
        RadixHeap {
            last: 0,
            len: 0,
            buckets: (0..65).map(|_| Vec::new()).collect(),
            below: BinaryHeap::new(),
        }
    }

    #[inline]
    fn bucket(&self, k: u64) -> usize {
        (64 - (k ^ self.last).leading_zeros()) as usize
    }

    #[inline]
    pub(crate) fn push(&mut self, k: u64, v: T) {
        if k < self.last {
            self.below.push(Reverse((k, v)));
            return;
        }
        let b = self.bucket(k);
        self.buckets[b].push((k, v));
        self.len += 1;
    }

    #[inline]
    pub(crate) fn pop(&mut self) -> Option<(u64, T)> {
        if let Some(Reverse(e)) = self.below.pop() {
            return Some(e);
        }
        if self.len == 0 {
            return None;
        }
        if self.buckets[0].is_empty() {
            let i = self.buckets.iter().position(|b| !b.is_empty())?;
            // Taken, not kept: a bucket emptied this way does not hold its largest size (the
            // floods run near the amplifier's memory cap).
            let items = std::mem::take(&mut self.buckets[i]);
            self.last = items.iter().map(|e| e.0).min().unwrap_or(self.last);
            // Every item goes to a bucket below `i`: its key agrees with the new least above
            // the bit that put them all in bucket `i`.
            for &e in &items {
                let b = self.bucket(e.0);
                self.buckets[b].push(e);
            }
        }
        self.len -= 1;
        self.buckets[0].pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small deterministic generator (the tests need no crate for it).
    fn next(s: &mut u64) -> u64 {
        *s ^= *s << 13;
        *s ^= *s >> 7;
        *s ^= *s << 17;
        *s
    }

    #[test]
    fn ordered_bits_sort_as_total_cmp() {
        let xs = [
            f32::NEG_INFINITY,
            -1.0e30,
            -2.5,
            -0.0,
            0.0,
            1.0e-40,
            3.0,
            f32::INFINITY,
            f32::NAN,
            -f32::NAN,
        ];
        for &a in &xs {
            assert_eq!(unordered(ordered(a)).to_bits(), a.to_bits());
            for &b in &xs {
                assert_eq!(ordered(a).cmp(&ordered(b)), a.total_cmp(&b), "{a} {b}");
            }
        }
    }

    /// A flood's pattern (pops interleaved with pushes at or above the last pop, distinct keys)
    /// and stray keys below the last pop: the radix heap pops what a binary heap pops.
    #[test]
    fn the_radix_heap_pops_what_a_binary_heap_pops() {
        let mut s = 0x9e37_79b9_7f4a_7c15u64;
        let mut radix = RadixHeap::new();
        let mut binary = BinaryHeap::new();
        let mut serial = 0u32;
        let mut push = |radix: &mut RadixHeap<u32>, binary: &mut BinaryHeap<_>, k: u64| {
            // Distinct keys, as a flood's (height, cell): the low bits are a serial number.
            let k = (k << 20) | u64::from(serial);
            serial += 1;
            radix.push(k, serial);
            binary.push(Reverse((k, serial)));
        };
        for _ in 0..2_000 {
            push(&mut radix, &mut binary, next(&mut s) >> 30);
        }
        let mut last = 0u64;
        let mut popped = 0;
        while let Some(Reverse(want)) = binary.pop() {
            assert_eq!(radix.pop(), Some(want));
            popped += 1;
            last = want.0;
            if popped < 50_000 {
                for _ in 0..(next(&mut s) % 3) {
                    let step = next(&mut s) >> 44;
                    // Now and then a key below the last pop.
                    let k = if next(&mut s) % 97 == 0 {
                        (last >> 20) / 2
                    } else {
                        (last >> 20) + step
                    };
                    push(&mut radix, &mut binary, k);
                }
            }
        }
        assert_eq!(radix.pop(), None);
        assert!(popped > 2_000);
    }
}

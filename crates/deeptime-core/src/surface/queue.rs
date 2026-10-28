// SPDX-License-Identifier: LGPL-3.0-or-later
//! A min-heap of packed keys for the surface's serial priority passes (the speed work of
//! 2026-10-28): the mouth fill (`marine::fill`, lowest ground first) and the periplatform export
//! (`carbonate::exported`, highest first). A key is a height's total-order bits, then a cell, then
//! 32 bits the pass carries; within one pass a cell goes in at most once, so no two keys share
//! height and cell, and every correct heap pops them in the one order the `BinaryHeap`s of items
//! the passes used before popped them. Four children a node halve the levels a sift walks, and a
//! node's children share a cache line.

/// The queue (see the module notes).
#[derive(Default)]
pub struct MinQueue(Vec<u128>);

/// `f64::total_cmp`'s order as an unsigned integer's.
fn order_bits(z: f64) -> u64 {
    let b = z.to_bits();
    if b >> 63 == 1 { !b } else { b | (1 << 63) }
}

impl MinQueue {
    /// The key that pops the lowest height first, ties by the lower cell, carrying `extra`.
    pub fn low_first(z: f64, c: u32, extra: f32) -> u128 {
        (u128::from(order_bits(z)) << 64) | (u128::from(c) << 32) | u128::from(extra.to_bits())
    }

    /// The key that pops the highest height first, ties by the lower cell.
    pub fn high_first(z: f64, c: u32) -> u128 {
        (u128::from(!order_bits(z)) << 64) | (u128::from(c) << 32)
    }

    /// The cell and the carried value of a key.
    pub fn cell_extra(k: u128) -> (usize, f32) {
        ((k >> 32) as u32 as usize, f32::from_bits(k as u32))
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn push(&mut self, k: u128) {
        let v = &mut self.0;
        let mut i = v.len();
        v.push(k);
        while i > 0 {
            let p = (i - 1) / 4;
            if v[p] <= k {
                break;
            }
            v[i] = v[p];
            i = p;
        }
        v[i] = k;
    }

    pub fn pop(&mut self) -> Option<u128> {
        let v = &mut self.0;
        let last = v.pop()?;
        let Some(&top) = v.first() else {
            return Some(last);
        };
        let n = v.len();
        let mut i = 0;
        loop {
            let first = 4 * i + 1;
            if first >= n {
                break;
            }
            let mut m = first;
            for c in first + 1..(first + 4).min(n) {
                if v[c] < v[m] {
                    m = c;
                }
            }
            if v[m] >= last {
                break;
            }
            v[i] = v[m];
            i = m;
        }
        v[i] = last;
        Some(top)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mouth fill's queue item as it was before the speed work of 2026-10-28, ordered for
    /// `BinaryHeap`: the lowest height first, ties by the lower cell.
    #[derive(Clone, Copy, PartialEq)]
    struct Item {
        z: f64,
        c: u32,
        dist: f32,
    }

    impl Eq for Item {}

    impl Ord for Item {
        fn cmp(&self, o: &Self) -> std::cmp::Ordering {
            o.z.total_cmp(&self.z).then_with(|| o.c.cmp(&self.c))
        }
    }

    impl PartialOrd for Item {
        fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(o))
        }
    }

    /// The four-way queue pops what the `BinaryHeap` of items popped, pushes and pops interleaved
    /// as a fill makes them: heights with ties (each cell once), both zeros, infinities and NaNs.
    #[test]
    fn the_queue_pops_what_the_binary_heap_popped_lowest_first() {
        let mut rng = crate::sphere::SplitMix64::new(4);
        let specials = [
            0.0,
            -0.0,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            -f64::NAN,
        ];
        let (mut want, mut got) = (std::collections::BinaryHeap::new(), MinQueue::default());
        let mut cells: Vec<u32> = (0..20_000).collect();
        for k in (1..cells.len()).rev() {
            cells.swap(k, (rng.next_f64() * (k + 1) as f64) as usize % (k + 1));
        }
        let mut next = cells.into_iter();
        let mut popped = 0;
        for round in 0..30_000 {
            if rng.next_f64() < 0.6 {
                let Some(c) = next.next() else { break };
                let r = rng.next_f64();
                let z = if r < 0.05 {
                    specials[(r * 120.0) as usize % specials.len()]
                } else {
                    // Few distinct heights, so many ties.
                    -((rng.next_f64() * 40.0).floor() * 25.0)
                };
                let dist = (round % 997) as f32 * 0.5;
                want.push(Item { z, c, dist });
                got.push(MinQueue::low_first(z, c, dist));
            } else {
                let (w, g) = (want.pop(), got.pop());
                assert_eq!(w.is_some(), g.is_some());
                if let (Some(w), Some(g)) = (w, g) {
                    assert_eq!(
                        (w.c as usize, w.dist),
                        MinQueue::cell_extra(g),
                        "pop {popped}"
                    );
                    popped += 1;
                }
            }
        }
        while let Some(w) = want.pop() {
            let g = got.pop().expect("the queue ran out first");
            assert_eq!((w.c as usize, w.dist), MinQueue::cell_extra(g));
            popped += 1;
        }
        assert!(got.pop().is_none());
        assert!(popped > 10_000, "{popped} pops");
    }

    /// The periplatform export's queue item as it was before the speed work of 2026-10-28.
    #[derive(Clone, Copy, PartialEq)]
    struct Higher {
        h: f64,
        c: u32,
    }

    impl Eq for Higher {}

    impl PartialOrd for Higher {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Higher {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.h.total_cmp(&other.h).then(other.c.cmp(&self.c))
        }
    }

    /// The highest-first keys pop what the `BinaryHeap` of the export's items popped, on heights
    /// with ties, both zeros, infinities and NaNs, pushes and pops interleaved.
    #[test]
    fn the_queue_pops_what_the_binary_heap_popped_highest_first() {
        let mut rng = crate::sphere::SplitMix64::new(5);
        let specials = [
            0.0,
            -0.0,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            -f64::NAN,
        ];
        let (mut want, mut got) = (std::collections::BinaryHeap::new(), MinQueue::default());
        let mut popped = 0;
        for c in 0..20_000u32 {
            let r = rng.next_f64();
            let h = if r < 0.05 {
                specials[(r * 120.0) as usize % specials.len()]
            } else {
                (rng.next_f64() * 40.0).floor() * 25.0 - 500.0
            };
            want.push(Higher { h, c });
            got.push(MinQueue::high_first(h, c));
            if rng.next_f64() < 0.4 {
                let (w, g) = (want.pop().unwrap(), got.pop().unwrap());
                assert_eq!(w.c as usize, MinQueue::cell_extra(g).0, "pop {popped}");
                popped += 1;
            }
        }
        while let Some(w) = want.pop() {
            assert_eq!(w.c as usize, MinQueue::cell_extra(got.pop().unwrap()).0);
            popped += 1;
        }
        assert!(got.pop().is_none() && popped == 20_000);
    }
}

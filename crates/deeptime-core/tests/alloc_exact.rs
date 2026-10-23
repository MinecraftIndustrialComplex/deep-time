// SPDX-License-Identifier: LGPL-3.0-or-later
//! The counting allocator's count is exact: after any mix of small allocations, reallocations
//! (across size classes, in place and moved), zeroed allocations and frees, on this thread and on
//! others, the count is back where it started (after a thread exits, within its park's bytes), with
//! the system allocator and with mimalloc under it (`--features mimalloc`). Its own binary without
//! the test harness (`harness = false` in Cargo.toml), so the allocator counts nothing else: libtest's
//! main thread files its bookkeeping for a test after it starts the test's thread, and under load
//! those blocks landed inside the one-thread check (the gate rerun on d74c74aa).

use std::alloc::{GlobalAlloc, Layout};

use deeptime_core::alloc::{self, CountingAlloc};

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// A small deterministic generator (xorshift), so a failure repeats.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// A churn of small blocks: allocated, grown and shrunk through many size classes, freed in an
/// order unlike the allocation order.
fn churn(seed: u64) {
    let mut rng = Rng(seed | 1);
    let mut live: Vec<(*mut u8, Layout)> = Vec::new();
    for _ in 0..20_000 {
        match rng.next() % 4 {
            0 | 1 => {
                let size = 1 + (rng.next() % 5000) as usize;
                let align = 1usize << (rng.next() % 7); // 1 to 64
                let layout = Layout::from_size_align(size, align).unwrap();
                // SAFETY: a non-zero size; freed below with the layout it was made with.
                let p = unsafe {
                    if rng.next().is_multiple_of(3) {
                        GLOBAL.alloc_zeroed(layout)
                    } else {
                        GLOBAL.alloc(layout)
                    }
                };
                assert!(!p.is_null());
                live.push((p, layout));
            }
            2 if !live.is_empty() => {
                let i = (rng.next() % live.len() as u64) as usize;
                let (p, layout) = live[i];
                let new_size = 1 + (rng.next() % 9000) as usize;
                // SAFETY: `p` is live with `layout`; the new size is non-zero.
                let q = unsafe { GLOBAL.realloc(p, layout, new_size) };
                assert!(!q.is_null());
                live[i] = (
                    q,
                    Layout::from_size_align(new_size, layout.align()).unwrap(),
                );
            }
            _ if !live.is_empty() => {
                let i = (rng.next() % live.len() as u64) as usize;
                let (p, layout) = live.swap_remove(i);
                // SAFETY: `p` is live with `layout`.
                unsafe { GLOBAL.dealloc(p, layout) };
            }
            _ => {}
        }
    }
    for (p, layout) in live {
        // SAFETY: as above.
        unsafe { GLOBAL.dealloc(p, layout) };
    }
}

fn main() {
    the_count_returns_to_where_it_started();
    println!("alloc_exact: the count returns to where it started");
}

fn the_count_returns_to_where_it_started() {
    // Parked blocks (Linux) stay counted until their thread gives them back.
    alloc::unpark_thread();
    let before = alloc::in_use();
    churn(7);
    alloc::unpark_thread();
    assert_eq!(
        alloc::in_use(),
        before,
        "one thread's churn leaves the count unchanged"
    );

    let handles: Vec<_> = (0..4u64)
        .map(|k| {
            std::thread::spawn(move || {
                churn(100 + k);
                alloc::unpark_thread();
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    alloc::release_cache();
    // The standard library frees a few blocks of its own after the closure's last `unpark_thread`,
    // as the thread exits; the park has no destructor (an allocator cannot allocate for one), so
    // up to its 4 slots of 4 KiB per thread stay counted. Anything a miscount of 20,000
    // operations left would be far above that.
    let left = alloc::in_use() as i64 - before as i64;
    let bound = 4 * (4 * 4096);
    assert!(
        left.abs() <= bound,
        "four threads' churn left {left} bytes counted (at most {bound} from thread exits)"
    );
}

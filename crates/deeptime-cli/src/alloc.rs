// SPDX-License-Identifier: LGPL-3.0-or-later
//! The CLI's counting global allocator (plan §5.7.5): `System` plus two atomics, so a run's memory
//! cap works as in the game and `deeptime bench` can report each run's peak. The same scheme as
//! `deeptime-jni`'s allocator (a global allocator must be declared by the final binary).

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

use deeptime_core::session::MemoryGauge;

static IN_USE: AtomicU64 = AtomicU64::new(0);
static PEAK: AtomicU64 = AtomicU64::new(0);

pub struct CountingAlloc;

#[inline]
fn added(bytes: usize) {
    let now = IN_USE.fetch_add(bytes as u64, Ordering::Relaxed) + bytes as u64;
    PEAK.fetch_max(now, Ordering::Relaxed);
}

#[inline]
fn removed(bytes: usize) {
    IN_USE.fetch_sub(bytes as u64, Ordering::Relaxed);
}

// SAFETY: every method forwards to `System` with the caller's layout and only updates counters.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwarded unchanged; the caller upholds `GlobalAlloc::alloc`'s contract.
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            added(layout.size());
        }
        p
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: as for `alloc`.
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            added(layout.size());
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` was allocated by this allocator (hence by `System`) with `layout`.
        unsafe { System.dealloc(ptr, layout) };
        removed(layout.size());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: as for `dealloc`; `new_size` is checked by the caller.
        let p = unsafe { System.realloc(ptr, layout, new_size) };
        if !p.is_null() {
            if new_size >= layout.size() {
                added(new_size - layout.size());
            } else {
                removed(layout.size() - new_size);
            }
        }
        p
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// Bytes allocated and not yet freed.
pub fn in_use() -> u64 {
    IN_USE.load(Ordering::Relaxed)
}

/// The highest `in_use` since the start or the last [`reset_peak`].
pub fn peak() -> u64 {
    PEAK.load(Ordering::Relaxed)
}

/// Starts a new peak measurement from what is in use now.
pub fn reset_peak() {
    PEAK.store(in_use(), Ordering::Relaxed);
}

/// The allocator as the core's [`MemoryGauge`].
pub struct AllocGauge;

impl MemoryGauge for AllocGauge {
    fn in_use(&self) -> u64 {
        in_use()
    }
    fn peak(&self) -> u64 {
        peak()
    }
}

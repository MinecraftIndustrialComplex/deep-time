// SPDX-License-Identifier: LGPL-3.0-or-later
//! The counting global allocator behind the memory cap (plan §5.7.5): the binaries that run the
//! simulation (`deeptime-jni`, `deeptime-cli`) declare `#[global_allocator] static _:
//! CountingAlloc`, and the session's [`MemoryGauge`] reads its counters.
//!
//! What it counts has to be what the process actually holds, or the cap, the estimate and the
//! editor's warnings are wrong (memacct, 2026-10-04: glibc kept 0.5–1.6 GiB of freed memory
//! that the live count did not see, so a quick run said 2.9 GiB at its peak while it held
//! 3.7 GiB). Two things make the count honest:
//!
//! - **Large blocks go straight to the OS** (Linux): a block of [`LARGE_BYTES`] or more is its
//!   own anonymous mapping, unmapped when freed and grown in place with `mremap`. glibc's malloc
//!   raises its mmap threshold every time a mapped block is freed (up to 32 MiB), after which
//!   every grid-sized array comes from an arena heap and stays there when freed, fragmented
//!   across the threads' arenas; that was where the missing memory was. Elsewhere the system
//!   allocator already returns large blocks (Windows' heap and macOS' large allocations).
//! - **Each block counts what it occupies**: a large block its whole pages, a small one its
//!   malloc chunk (the size plus glibc's 8-byte header, rounded up to 16, at least 32).
//!
//! The stages allocate and free the same grid-sized arrays every step (hundreds of GiB over a
//! quick run), so mapping each one afresh costs a page fault per page: a quick 16k run took 39 %
//! longer. A freed large block is therefore kept for the next block of exactly its size, up to
//! [`CACHE_BYTES`] of them; the kept blocks count as in use (the process holds them), and
//! [`release_cache`] gives them back when the memory cap needs the room.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use crate::session::MemoryGauge;

/// Blocks this large or larger are mapped from the OS on their own (Linux).
pub const LARGE_BYTES: usize = 1 << 20;

/// The page granule large blocks are counted in (the smallest page size of the platforms the
/// mod runs on; a mapping's real pages are at least this).
const PAGE: usize = 4096;

static IN_USE: AtomicU64 = AtomicU64::new(0);
/// The highest `IN_USE` since the start of the current window ([`start_window`]).
static PEAK: AtomicU64 = AtomicU64::new(0);
/// The highest `IN_USE` of the windows before the current one.
static EARLIER_PEAK: AtomicU64 = AtomicU64::new(0);
/// The current large-block threshold ([`set_large_threshold`]; `usize::MAX`: off).
static LARGE: AtomicUsize = AtomicUsize::new(LARGE_BYTES);

/// At most this many bytes of freed large blocks are kept for reuse.
pub const CACHE_BYTES: usize = 256 << 20;
/// The current limit of the kept blocks ([`set_cache_limit`]).
static CACHE_LIMIT: AtomicUsize = AtomicUsize::new(CACHE_BYTES);
/// At most this many kept blocks.
const SLOTS: usize = 64;

/// Freed large blocks kept for reuse, oldest first: `(address, page-rounded length)`.
struct Cache {
    blocks: [(usize, usize); SLOTS],
    n: usize,
    bytes: usize,
}

static CACHE: std::sync::Mutex<Cache> = std::sync::Mutex::new(Cache {
    blocks: [(0, 0); SLOTS],
    n: 0,
    bytes: 0,
});

fn cache() -> std::sync::MutexGuard<'static, Cache> {
    CACHE.lock().unwrap_or_else(|e| e.into_inner())
}

impl Cache {
    /// A kept block of exactly `len` bytes.
    fn take(&mut self, len: usize) -> Option<*mut u8> {
        let i = self.blocks[..self.n].iter().rposition(|b| b.1 == len)?;
        let (p, _) = self.blocks[i];
        self.blocks.copy_within(i + 1..self.n, i);
        self.n -= 1;
        self.bytes -= len;
        Some(p as *mut u8)
    }

    /// Unmaps the oldest kept block; its bytes leave the count.
    fn drop_oldest(&mut self) {
        let (p, len) = self.blocks[0];
        self.blocks.copy_within(1..self.n, 0);
        self.n -= 1;
        self.bytes -= len;
        // SAFETY: a kept block is a live mapping of `len` bytes that nothing else refers to.
        unsafe { os::unmap(p as *mut u8, len) };
        removed(len);
    }

    /// Keeps a freed block for reuse, making room by unmapping the oldest; false when it is
    /// bigger than the whole limit (the caller unmaps it).
    fn put(&mut self, p: *mut u8, len: usize) -> bool {
        let limit = CACHE_LIMIT.load(Ordering::Relaxed);
        if len > limit {
            return false;
        }
        while self.n > 0 && (self.n == SLOTS || self.bytes + len > limit) {
            self.drop_oldest();
        }
        self.blocks[self.n] = (p as usize, len);
        self.n += 1;
        self.bytes += len;
        true
    }
}

/// Gives every kept large block back to the OS; returns the bytes released.
pub fn release_cache() -> u64 {
    let mut c = cache();
    let bytes = c.bytes as u64;
    while c.n > 0 {
        c.drop_oldest();
    }
    bytes
}

/// Sets how many bytes of freed large blocks are kept for reuse (0: none). For experiments
/// (`DEEPTIME_LARGE_CACHE` in the CLI); the default is [`CACHE_BYTES`].
pub fn set_cache_limit(bytes: usize) {
    CACHE_LIMIT.store(bytes, Ordering::Relaxed);
    let mut c = cache();
    while c.n > 0 && c.bytes > bytes {
        c.drop_oldest();
    }
}

/// The counting allocator: [`System`] for small blocks, the OS's own mappings for large ones on
/// Linux, and two atomics.
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

/// What a small block occupies in the system allocator: glibc's chunk (size + 8 rounded up to
/// 16, at least 32 bytes); about the same in the other platforms' allocators.
#[inline]
fn small_footprint(size: usize) -> usize {
    (size.saturating_add(8 + 15) & !15).max(32)
}

#[inline]
fn pages(size: usize) -> usize {
    size.saturating_add(PAGE - 1) & !(PAGE - 1)
}

#[inline]
fn is_large(layout: &Layout) -> bool {
    cfg!(target_os = "linux")
        && layout.size() >= LARGE.load(Ordering::Relaxed)
        && layout.align() <= PAGE
}

#[cfg(target_os = "linux")]
mod os {
    use std::ffi::c_void;

    // The three calls large blocks need, from the C library every Linux Rust binary links.
    const PROT_READ: i32 = 1;
    const PROT_WRITE: i32 = 2;
    const MAP_PRIVATE: i32 = 0x02;
    const MAP_ANONYMOUS: i32 = 0x20;
    const MREMAP_MAYMOVE: i32 = 1;
    const MAP_FAILED: *mut c_void = !0usize as *mut c_void;

    unsafe extern "C" {
        fn mmap(addr: *mut c_void, len: usize, prot: i32, flags: i32, fd: i32, off: i64)
        -> *mut c_void;
        fn munmap(addr: *mut c_void, len: usize) -> i32;
        fn mremap(old: *mut c_void, old_len: usize, new_len: usize, flags: i32, ...)
        -> *mut c_void;
    }

    /// A fresh zeroed private mapping of `len` bytes, or null.
    pub fn map(len: usize) -> *mut u8 {
        // SAFETY: an anonymous private mapping at an address the kernel chooses touches no
        // existing memory.
        let p = unsafe {
            mmap(
                std::ptr::null_mut(),
                len,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        if p == MAP_FAILED {
            std::ptr::null_mut()
        } else {
            p.cast()
        }
    }

    /// Unmaps a block [`map`] or [`remap`] returned with this length.
    ///
    /// # Safety
    /// `p` and `len` must be a live mapping of this module's; it is gone afterwards.
    pub unsafe fn unmap(p: *mut u8, len: usize) {
        // SAFETY: the caller's contract.
        unsafe { munmap(p.cast(), len) };
    }

    /// Grows or shrinks a mapping, moving it when it must; null when it cannot (the old mapping
    /// is then untouched).
    ///
    /// # Safety
    /// `p` and `old` must be a live mapping of this module's.
    pub unsafe fn remap(p: *mut u8, old: usize, new: usize) -> *mut u8 {
        // SAFETY: the caller's contract; MREMAP_MAYMOVE leaves the old mapping intact on failure.
        let q = unsafe { mremap(p.cast(), old, new, MREMAP_MAYMOVE) };
        if q == MAP_FAILED {
            std::ptr::null_mut()
        } else {
            q.cast()
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod os {
    pub fn map(_len: usize) -> *mut u8 {
        unreachable!("large blocks are mapped only on Linux")
    }
    pub unsafe fn unmap(_p: *mut u8, _len: usize) {
        unreachable!("large blocks are mapped only on Linux")
    }
    pub unsafe fn remap(_p: *mut u8, _old: usize, _new: usize) -> *mut u8 {
        unreachable!("large blocks are mapped only on Linux")
    }
}

impl CountingAlloc {
    fn alloc_with(&self, layout: Layout, zeroed: bool) -> *mut u8 {
        if is_large(&layout) {
            let len = pages(layout.size());
            // A kept block is already counted.
            let kept = cache().take(len);
            if let Some(p) = kept {
                if zeroed {
                    // SAFETY: the kept block is `len` writable bytes nothing else refers to.
                    unsafe { std::ptr::write_bytes(p, 0, len) };
                }
                return p;
            }
            let p = os::map(len);
            if !p.is_null() {
                added(len);
            }
            return p;
        }
        // SAFETY: forwarded unchanged; the caller upholds `GlobalAlloc::alloc`'s contract.
        let p = unsafe {
            if zeroed {
                System.alloc_zeroed(layout)
            } else {
                System.alloc(layout)
            }
        };
        if !p.is_null() {
            added(small_footprint(layout.size()));
        }
        p
    }
}

// SAFETY: small blocks are forwarded to `System` with the caller's layout; large blocks are
// page-aligned private mappings of at least the layout's size, freed with the same length.
// Whether a block is large depends only on its layout and the threshold, which is fixed before
// the first large allocation (`set_large_threshold`).
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc_with(layout, false)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // A fresh mapping is zeroed by the kernel.
        self.alloc_with(layout, true)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if is_large(&layout) {
            let len = pages(layout.size());
            if !cache().put(ptr, len) {
                // SAFETY: `ptr` is the mapping `alloc` made for this layout.
                unsafe { os::unmap(ptr, len) };
                removed(len);
            }
            return;
        }
        // SAFETY: `ptr` was allocated by `System` with `layout`.
        unsafe { System.dealloc(ptr, layout) };
        removed(small_footprint(layout.size()));
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: the caller guarantees `new_size` with `layout.align()` is a valid layout.
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
        match (is_large(&layout), is_large(&new_layout)) {
            (false, false) => {
                // SAFETY: as for `dealloc`; `new_size` is checked by the caller.
                let p = unsafe { System.realloc(ptr, layout, new_size) };
                if !p.is_null() {
                    removed(small_footprint(layout.size()));
                    added(small_footprint(new_size));
                }
                p
            }
            (true, true) => {
                let (old, new) = (pages(layout.size()), pages(new_size));
                if old == new {
                    return ptr;
                }
                // SAFETY: `ptr` is the mapping of `old` bytes `alloc` made.
                let p = unsafe { os::remap(ptr, old, new) };
                if !p.is_null() {
                    if new > old {
                        added(new - old);
                    } else {
                        removed(old - new);
                    }
                }
                p
            }
            _ => {
                // Across the threshold: a new block, the bytes copied, the old block freed.
                let p = self.alloc_with(new_layout, false);
                if !p.is_null() {
                    // SAFETY: both blocks are live and at least `min` bytes long; they are
                    // distinct allocations, so they do not overlap.
                    unsafe {
                        std::ptr::copy_nonoverlapping(ptr, p, layout.size().min(new_size));
                        self.dealloc(ptr, layout);
                    }
                }
                p
            }
        }
    }
}

/// Bytes the process holds for allocations not yet freed (as the allocator occupies them).
pub fn in_use() -> u64 {
    IN_USE.load(Ordering::Relaxed)
}

/// The highest [`in_use`] since the start or the last [`reset_peak`].
pub fn peak() -> u64 {
    PEAK.load(Ordering::Relaxed)
        .max(EARLIER_PEAK.load(Ordering::Relaxed))
}

/// Starts a new [`peak`] measurement from what is in use now.
pub fn reset_peak() {
    PEAK.store(in_use(), Ordering::Relaxed);
    EARLIER_PEAK.store(0, Ordering::Relaxed);
}

/// Starts a new window for [`window_peak`] (a traced phase), keeping [`peak`] whole.
pub fn start_window() {
    EARLIER_PEAK.fetch_max(PEAK.load(Ordering::Relaxed), Ordering::Relaxed);
    PEAK.store(in_use(), Ordering::Relaxed);
}

/// The highest [`in_use`] since the last [`start_window`] (or [`reset_peak`]).
pub fn window_peak() -> u64 {
    PEAK.load(Ordering::Relaxed)
}

/// Sets the size from which blocks are mapped on their own (Linux; `None`: never). Call it
/// before anything allocates a block that large, and never again: a large block freed under a
/// different threshold would go to the wrong place. For experiments only (`DEEPTIME_LARGE_ALLOC`
/// in the CLI); the default is [`LARGE_BYTES`].
pub fn set_large_threshold(bytes: Option<usize>) {
    LARGE.store(bytes.unwrap_or(usize::MAX).max(PAGE), Ordering::Relaxed);
}

/// The allocator's counters as the session's [`MemoryGauge`].
pub struct AllocGauge;

impl MemoryGauge for AllocGauge {
    fn in_use(&self) -> u64 {
        in_use()
    }
    fn peak(&self) -> u64 {
        peak()
    }
    fn phase(&self, _name: &str, begin: bool) {
        if begin {
            start_window();
        }
    }
    fn phase_peak(&self) -> u64 {
        window_peak()
    }
    fn release(&self) -> u64 {
        release_cache()
    }
}

// The core's own tests run on this allocator, so every test exercises the large-block path.
#[cfg(test)]
#[global_allocator]
static TEST_GLOBAL: CountingAlloc = CountingAlloc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_a_large_allocation_by_its_pages() {
        let before = in_use();
        let v = vec![0u8; (64 << 20) + 1];
        assert!(v.iter().step_by(4096).all(|&b| b == 0), "not zeroed");
        // Other test threads allocate concurrently, so only check the order of magnitude.
        assert!(in_use() >= before + (64 << 20) - (8 << 20), "in_use did not grow");
        assert!(peak() >= 64 << 20);
        drop(v);
        // The freed block is kept for reuse, and counted, until the cache is released.
        release_cache();
        assert!(in_use() < before + (32 << 20));
    }

    #[test]
    fn growing_across_the_threshold_keeps_the_bytes() {
        let mut v: Vec<u32> = Vec::new();
        for i in 0..(3 << 20) {
            v.push(i);
        }
        assert!(v.iter().enumerate().all(|(i, &x)| x == i as u32));
        v.truncate(1000);
        v.shrink_to_fit();
        assert!(v.iter().enumerate().all(|(i, &x)| x == i as u32));
        let big = vec![7u64; LARGE_BYTES / 8 + 3];
        let mut b2 = big.clone();
        b2.resize(LARGE_BYTES, 9);
        assert_eq!(b2[LARGE_BYTES / 8 + 2], 7);
        assert_eq!(b2[LARGE_BYTES - 1], 9);
        b2.truncate(10);
        b2.shrink_to_fit();
        assert_eq!(b2, vec![7u64; 10]);
    }

    #[test]
    fn a_freed_large_block_leaves_the_process() {
        // The point of mapping large blocks: a large block is a mapping of its own, so freeing it
        // (munmap) gives its memory back at once and the count and the resident size agree.
        #[cfg(target_os = "linux")]
        {
            let len = 3 * LARGE_BYTES + 5;
            let v = vec![1u8; len];
            let start = v.as_ptr() as usize;
            let maps = std::fs::read_to_string("/proc/self/maps").unwrap();
            let own = maps.lines().any(|l| {
                let range = l.split_whitespace().next().unwrap();
                let (a, b) = range.split_once('-').unwrap();
                let a = usize::from_str_radix(a, 16).unwrap();
                let b = usize::from_str_radix(b, 16).unwrap();
                // The kernel may merge it with a neighbouring anonymous mapping, so it must
                // start or end exactly at the block's page-rounded bounds.
                a <= start && start + pages(len) <= b && (a == start || b == start + pages(len))
            });
            assert!(own, "a {len}-byte block is not a mapping of its own");
            drop(v);
        }
    }

    #[test]
    fn a_kept_block_is_reused_zeroed_and_released() {
        if !cfg!(target_os = "linux") {
            return;
        }
        // An odd size no other test allocates, so the kept block is this test's.
        let len = 5 * LARGE_BYTES + 3 * 4096 + 17;
        let mut v = vec![0u8; len];
        v.fill(7);
        let first = v.as_ptr() as usize;
        drop(v);
        let v = vec![0u8; len];
        assert_eq!(v.as_ptr() as usize, first, "the freed block was reused");
        assert!(v.iter().all(|&b| b == 0), "a reused block is zeroed when asked");
        drop(v);
        assert!(release_cache() >= pages(len) as u64);
    }

    #[test]
    fn small_blocks_count_their_chunk() {
        assert_eq!(small_footprint(1), 32);
        assert_eq!(small_footprint(24), 32);
        assert_eq!(small_footprint(25), 48);
        assert_eq!(small_footprint(1000), 1008);
        assert_eq!(pages(1), 4096);
        assert_eq!(pages(LARGE_BYTES + 1), LARGE_BYTES + 4096);
    }
}

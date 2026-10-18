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
//! [`CACHE_BYTES`] of them; the kept blocks count as in use (the process holds them), go back to
//! the OS oldest first when a new block would take the count past the ceiling the session sets
//! ([`keep_below`]), and all go back when a stage ends ([`release_cache`]).
//!
//! Small blocks stay with the system allocator, and glibc keeps the ones freed in each thread's
//! arena for reuse: that memory is the process's but not in the count. [`release_cache`] gives it
//! back too (`malloc_trim`), so between stages the process holds what the count says (memory
//! test, 2026-10-17: on the test planet the session's arenas kept 11 to 15 MiB of freed blocks
//! from the stages into the bake, more on a busy machine). Each thread also keeps its last few
//! freed small blocks for its own next block of the same size, counted while it keeps them (the
//! `park` module: a per-cell `Vec` freed and allocated again on one thread then touches neither
//! malloc nor the shared count).

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use crate::session::MemoryGauge;

/// Blocks this large or larger are mapped from the OS on their own (Linux).
pub const LARGE_BYTES: usize = 1 << 20;

/// The page granule large blocks are counted in (the smallest page size of the platforms the
/// mod runs on; a mapping's real pages are at least this).
const PAGE: usize = 4096;

/// A value alone on its cache lines (two 64-byte lines, which the adjacent-line prefetcher fetches
/// as a pair). Every allocation and free writes `IN_USE`, and a growing count `PEAK`; beside the
/// settings each allocation reads (`LARGE`, `KEPT`, `CEILING`), each of those writes would make
/// the next read of them on every other thread a miss (P1-19).
#[repr(align(128))]
struct Line<T>(T);

impl<T> std::ops::Deref for Line<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

static IN_USE: Line<AtomicU64> = Line(AtomicU64::new(0));
/// The highest `IN_USE` since the start of the current window ([`start_window`]).
static PEAK: Line<AtomicU64> = Line(AtomicU64::new(0));
/// The highest `IN_USE` of the windows before the current one.
static EARLIER_PEAK: AtomicU64 = AtomicU64::new(0);
/// The current large-block threshold ([`set_large_threshold`]; `usize::MAX`: off).
static LARGE: AtomicUsize = AtomicUsize::new(LARGE_BYTES);

/// At most this many bytes of freed large blocks are kept for reuse. Measured on quick 16k
/// (memacct, 2026-10-04, CPU time against glibc's malloc): none +27 %, 256 MiB +16 %, 1 GiB +5 %;
/// the coupled stages free and reallocate up to ~1 GiB of grids a step. Full 16k frees more
/// (P1-19, 30 Myr counted per phase): 1 GiB pushed out 2,730 kept blocks (33 GB) to make room
/// and the surface reused 62 % of its large blocks; 2 GiB pushed out 184 (2.8 GB), 4 GiB none,
/// with no further gain. Kept blocks count in the memory in use, so the cap bounds them as well.
pub const CACHE_BYTES: usize = 2 << 30;
/// The current limit of the kept blocks ([`set_cache_limit`]).
static CACHE_LIMIT: AtomicUsize = AtomicUsize::new(CACHE_BYTES);
/// At most this many kept blocks (2 GiB of the smallest large blocks).
const SLOTS: usize = 2048;
/// Kept blocks never hold the count above this ([`keep_below`]).
static CEILING: AtomicU64 = AtomicU64::new(u64::MAX);
/// The bytes of the kept blocks (`Cache::bytes`, readable without the lock).
static KEPT: AtomicUsize = AtomicUsize::new(0);

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
        KEPT.store(self.bytes, Ordering::Relaxed);
        Some(p as *mut u8)
    }

    /// Unmaps the oldest kept block; its bytes leave the count.
    fn drop_oldest(&mut self) {
        let (p, len) = self.blocks[0];
        self.blocks.copy_within(1..self.n, 0);
        self.n -= 1;
        self.bytes -= len;
        KEPT.store(self.bytes, Ordering::Relaxed);
        // SAFETY: a kept block is a live mapping of `len` bytes that nothing else refers to.
        unsafe { os::unmap(p as *mut u8, len) };
        removed(len);
    }

    /// Keeps a freed block for reuse, making room by unmapping the oldest; false when it is
    /// bigger than the whole limit (the caller unmaps it).
    fn put(&mut self, p: *mut u8, len: usize) -> bool {
        let limit = CACHE_LIMIT.load(Ordering::Relaxed);
        if len > limit || in_use() > CEILING.load(Ordering::Relaxed) {
            return false;
        }
        while self.n > 0 && (self.n == SLOTS || self.bytes + len > limit) {
            self.drop_oldest();
        }
        self.blocks[self.n] = (p as usize, len);
        self.n += 1;
        self.bytes += len;
        KEPT.store(self.bytes, Ordering::Relaxed);
        true
    }
}

/// Kept blocks never hold the count above `bytes`: the oldest go back to the OS now until it is
/// under, and a freed block is not kept while it is over (`MemoryGauge::keep_below`).
pub fn keep_below(bytes: u64) {
    CEILING.store(bytes, Ordering::Relaxed);
    let mut c = cache();
    while c.n > 0 && in_use() > bytes {
        c.drop_oldest();
    }
}

/// Gives every kept large block back to the OS; returns the bytes released (the count drops by
/// as much). On glibc it also returns the free memory of malloc's arenas ([`trim_system`]), which
/// the count never included.
pub fn release_cache() -> u64 {
    let bytes = {
        let mut c = cache();
        let bytes = c.bytes as u64;
        while c.n > 0 {
            c.drop_oldest();
        }
        bytes
    };
    trim_system();
    bytes
}

/// The bytes of the large blocks kept for reuse (in the count, [`in_use`]). They leave by
/// themselves, oldest first, when the count would pass the ceiling ([`keep_below`]), so the
/// session asks its history for room by the count without them (P1-19: giving them back whenever
/// a step might need room returned up to 3 GiB a few times a step on full 16k, and the next step
/// mapped and faulted it in again).
pub fn kept_bytes() -> u64 {
    KEPT.load(Ordering::Relaxed) as u64
}

/// Returns the free memory the system allocator keeps in its arenas to the OS (glibc's
/// `malloc_trim(0)`: every arena's top and the whole pages of its free chunks); a no-op where the
/// C library is not glibc. Small blocks freed by the stages' threads otherwise stay resident in
/// those threads' arenas, uncounted (module docs). The blocks in use are untouched, so the count
/// does not change.
pub fn trim_system() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        unsafe extern "C" {
            fn malloc_trim(pad: usize) -> i32;
        }
        // SAFETY: malloc_trim takes the arenas' own locks and only releases free memory.
        unsafe { malloc_trim(0) };
    }
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

/// Counts `bytes` more; returns the count with them.
#[inline]
fn added(bytes: usize) -> u64 {
    let now = IN_USE.fetch_add(bytes as u64, Ordering::Relaxed) + bytes as u64;
    // A read first: the count is mostly under its peak, and an atomic max takes the line for
    // writing even when it leaves the value as it is (P1-19). The peak is the same either way.
    if now > PEAK.load(Ordering::Relaxed) {
        PEAK.fetch_max(now, Ordering::Relaxed);
    }
    now
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
        fn mmap(
            addr: *mut c_void,
            len: usize,
            prot: i32,
            flags: i32,
            fd: i32,
            off: i64,
        ) -> *mut c_void;
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
                hold_ceiling(added(len));
            }
            return p;
        }
        if park::fits(&layout)
            && let Some(p) = park::take(layout.size())
        {
            // A parked block is already counted.
            if zeroed {
                // SAFETY: the parked block is at least `size` writable bytes nothing else
                // refers to.
                unsafe { std::ptr::write_bytes(p, 0, layout.size()) };
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
            hold_ceiling(added(small_footprint(layout.size())));
        }
        p
    }
}

/// A few freed small blocks each thread keeps for its own next block of the same size (Linux;
/// P1-19). The L1 sampling asks the k-d tree for the nearest samples of every cell of every plate
/// step, and each answer is a fresh `Vec` freed a moment later on the same thread: a malloc and a
/// free each time, and two writes of the process-wide count that every thread's allocations
/// share (2.8 % of a full-16k run's CPU in the counting allocator, two thirds of it from those
/// answers, P1-19 profile). A parked block stays counted, as the kept large blocks do, so the
/// count does not change when it is parked or taken back; a thread holds at most [`park::SLOTS`]
/// blocks of at most [`park::MAX`] bytes. The session's threads return theirs when they end
/// ([`unpark_thread`]); a thread that ends without it leaves them allocated and counted (at most
/// 16 KiB).
mod park {
    use std::alloc::Layout;
    use std::cell::Cell;

    /// Blocks parked per thread.
    pub const SLOTS: usize = 4;
    /// The largest block parked.
    pub const MAX: usize = 4096;

    pub struct Park {
        ptr: [Cell<usize>; SLOTS],
        /// The block's requested size; 0: the slot is empty (no block has size 0).
        size: [Cell<usize>; SLOTS],
        next: Cell<usize>,
    }

    // A constant initialiser and no destructor: a plain thread-local static, which allocates
    // nothing and is valid for the thread's whole life (an allocator must not allocate here).
    thread_local! {
        static PARK: Park = const {
            Park {
                ptr: [const { Cell::new(0) }; SLOTS],
                size: [const { Cell::new(0) }; SLOTS],
                next: Cell::new(0),
            }
        };
    }

    /// True for the blocks that can be parked: the system allocator's plain `malloc` blocks
    /// (alignment up to its 16 bytes and not above the size), small enough. Any two such blocks
    /// of one size are interchangeable.
    #[inline]
    pub fn fits(layout: &Layout) -> bool {
        cfg!(target_os = "linux")
            && layout.size() <= MAX
            && layout.align() <= 16
            && layout.align() <= layout.size()
    }

    /// This thread's parked block of exactly `size` bytes.
    #[inline]
    pub fn take(size: usize) -> Option<*mut u8> {
        PARK.with(|k| {
            for i in 0..SLOTS {
                if k.size[i].get() == size {
                    k.size[i].set(0);
                    return Some(k.ptr[i].get() as *mut u8);
                }
            }
            None
        })
    }

    /// Parks a freed block of `size` bytes; returns the block it pushed out, if any
    /// (`(address, size)`), which the caller frees.
    #[inline]
    pub fn put(p: *mut u8, size: usize) -> Option<(usize, usize)> {
        PARK.with(|k| {
            let i = (0..SLOTS)
                .find(|&i| k.size[i].get() == 0)
                .unwrap_or_else(|| {
                    let n = k.next.get();
                    k.next.set((n + 1) % SLOTS);
                    n
                });
            let out = (k.ptr[i].get(), k.size[i].get());
            k.ptr[i].set(p as usize);
            k.size[i].set(size);
            (out.1 != 0).then_some(out)
        })
    }

    /// Empties this thread's slots; returns the blocks that were in them.
    pub fn drain() -> [(usize, usize); SLOTS] {
        PARK.with(|k| {
            std::array::from_fn(|i| {
                let out = (k.ptr[i].get(), k.size[i].get());
                k.size[i].set(0);
                out
            })
        })
    }
}

/// Frees a block that was parked: the system allocator's, `size` bytes, as [`park::fits`] took.
fn free_parked(p: usize, size: usize) {
    // SAFETY: a parked block is a live `System` block of this size and an alignment the
    // system allocator gives every block (`park::fits`), and nothing else refers to it.
    unsafe { System.dealloc(p as *mut u8, Layout::from_size_align_unchecked(size, 1)) };
    removed(small_footprint(size));
}

/// Gives this thread's parked small blocks back to the system allocator (the session's threads
/// call it when they end).
pub fn unpark_thread() {
    for (p, size) in park::drain() {
        if size != 0 {
            free_parked(p, size);
        }
    }
}

/// After the count grew: while kept blocks hold it over the ceiling ([`keep_below`]), the oldest
/// go back to the OS. Small blocks and growth in place count too, not only fresh large blocks,
/// so a step that allocates many small blocks cannot pass the ceiling while kept blocks are still
/// held. `now` is the count the growth left ([`added`]), so the common case reads only the two
/// settings' lines, not the count's line other threads keep writing (P1-19).
#[inline]
fn hold_ceiling(now: u64) {
    if KEPT.load(Ordering::Relaxed) == 0 || now <= CEILING.load(Ordering::Relaxed) {
        return;
    }
    let mut c = cache();
    while c.n > 0 && in_use() > CEILING.load(Ordering::Relaxed) {
        c.drop_oldest();
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
        if park::fits(&layout) {
            // Parked, it stays counted; the block it pushed out is freed instead.
            if let Some((p, size)) = park::put(ptr, layout.size()) {
                free_parked(p, size);
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
                    hold_ceiling(added(small_footprint(new_size)));
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
                        hold_ceiling(added(new - old));
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

/// `n` copies of `v`, written in parallel. `vec![v; n]` writes them on the calling thread (a zero
/// `v` comes through `alloc_zeroed`, which clears a kept block there), and for a grid-sized array
/// that serial pass is what a step's parallel passes wait for (P1-19).
pub fn filled<T: Copy + Send + Sync>(n: usize, v: T) -> Vec<T> {
    let mut out = Vec::new();
    fill(&mut out, n, v);
    out
}

/// Makes `out` `n` copies of `v`, written in parallel, in the allocation it has when that is
/// large enough.
pub fn fill<T: Copy + Send + Sync>(out: &mut Vec<T>, n: usize, v: T) {
    use rayon::prelude::*;
    rayon::iter::repeat_n(v, n).collect_into_vec(out);
}

/// A copy of `src`, written in parallel: `to_vec` and `clone` copy a grid-sized array on the
/// calling thread, as [`filled`] says of `vec!` (P1-19).
pub fn copied<T: Copy + Send + Sync>(src: &[T]) -> Vec<T> {
    use rayon::prelude::*;
    let mut out = Vec::new();
    src.par_iter().copied().collect_into_vec(&mut out);
    out
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
    fn kept(&self) -> u64 {
        kept_bytes()
    }
    fn keep_below(&self, bytes: u64) {
        keep_below(bytes);
    }
}

// The core's own tests run on this allocator, so every test exercises the large-block path.
#[cfg(test)]
#[global_allocator]
static TEST_GLOBAL: CountingAlloc = CountingAlloc;

#[cfg(test)]
mod tests {
    use super::*;

    /// The tests that release or limit the kept blocks run one at a time.
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn serial() -> std::sync::MutexGuard<'static, ()> {
        SERIAL.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn counts_a_large_allocation_by_its_pages() {
        let _serial = serial();
        let before = in_use();
        let v = vec![0u8; (64 << 20) + 1];
        assert!(v.iter().step_by(4096).all(|&b| b == 0), "not zeroed");
        // Other test threads allocate concurrently, so only check the order of magnitude.
        assert!(
            in_use() >= before + (64 << 20) - (8 << 20),
            "in_use did not grow"
        );
        assert!(peak() >= 64 << 20);
        let addr = v.as_ptr() as usize;
        let kept_here = || {
            let c = cache();
            c.blocks[..c.n].iter().any(|b| b.0 == addr)
        };
        drop(v);
        // The freed block is kept for reuse, and counted, until the cache is released. Other
        // test threads allocate and free while release_cache trims the arenas, so follow this
        // block rather than the global count (the count check raced them).
        let kept = kept_here();
        let released = release_cache();
        if kept {
            assert!(released >= 64 << 20, "the kept block was not released");
        }
        assert!(!kept_here(), "the block is still kept after the release");
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
        let _serial = serial();
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
        assert!(
            v.iter().all(|&b| b == 0),
            "a reused block is zeroed when asked"
        );
        drop(v);
        assert!(release_cache() >= pages(len) as u64);
    }

    #[test]
    fn small_blocks_past_the_ceiling_give_back_kept_blocks() {
        let _serial = serial();
        if !cfg!(target_os = "linux") {
            return;
        }
        // An odd size no other test allocates, so the kept block is this test's.
        let len = 7 * LARGE_BYTES + 5 * 4096 + 3;
        let v = vec![1u8; len];
        let at = v.as_ptr() as usize;
        keep_below(u64::MAX);
        drop(v);
        let kept = |c: &Cache| c.blocks[..c.n].iter().any(|b| b.0 == at);
        assert!(kept(&cache()), "the freed block is kept");
        // A ceiling under the count, set without `keep_below`'s own release (other test threads
        // allocate and free concurrently, so a ceiling near the count would race them); then one
        // small block passes it. Whether the store alone releases nothing is not checked here: any
        // other test thread's small block past the ceiling gives the kept block back too, so that
        // check failed now and then on a busy machine (gate batch 2026-10-16).
        CEILING.store(0, Ordering::Relaxed);
        // Larger than a parked block, so it is a fresh one whatever this thread parked.
        let small = std::hint::black_box(vec![2u8; park::MAX + 1000]);
        let gone = !kept(&cache());
        keep_below(u64::MAX);
        drop(small);
        assert!(
            gone,
            "a small block past the ceiling gave the kept block back"
        );
    }

    #[test]
    fn a_freed_small_block_is_parked_counted_and_reused() {
        if !cfg!(target_os = "linux") {
            return;
        }
        // Each test runs on a thread of its own, so this thread's slots hold only its blocks.
        unpark_thread();
        let size = 777;
        let v = std::hint::black_box(vec![5u8; size]);
        let first = v.as_ptr() as usize;
        drop(v);
        let parked = park::drain();
        assert!(
            parked.contains(&(first, size)),
            "the freed block was parked"
        );
        for (p, n) in parked {
            if n != 0 {
                park::put(p as *mut u8, n);
            }
        }
        let v = std::hint::black_box(vec![0u8; size]);
        assert_eq!(v.as_ptr() as usize, first, "the parked block was reused");
        assert!(
            v.iter().all(|&b| b == 0),
            "a reused block is zeroed when asked"
        );
        drop(v);
        // A full thread's slots push out the oldest (round robin) and free it.
        let blocks: Vec<Vec<u8>> = (0..park::SLOTS + 2).map(|k| vec![1u8; 100 + k]).collect();
        drop(blocks);
        let held = park::drain();
        assert_eq!(held.iter().filter(|b| b.1 != 0).count(), park::SLOTS);
        for (p, n) in held {
            if n != 0 {
                free_parked(p, n);
            }
        }
        // Blocks that are not plain malloc blocks are never parked.
        assert!(!park::fits(&Layout::from_size_align(64, 32).unwrap()));
        assert!(!park::fits(&Layout::from_size_align(4, 8).unwrap()));
        assert!(!park::fits(
            &Layout::from_size_align(park::MAX + 1, 8).unwrap()
        ));
        assert!(park::fits(&Layout::from_size_align(96, 8).unwrap()));
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

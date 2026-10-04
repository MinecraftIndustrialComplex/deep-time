// SPDX-License-Identifier: LGPL-3.0-or-later
//! The counting global allocator (plan §5.7.5; `deeptime_core::alloc`): the memory cap is checked
//! before a stage allocates and the peak is reported in the progress block. It counts every
//! allocation of the library (all sessions together); the JVM heap is separate.

pub use deeptime_core::alloc::{AllocGauge, in_use, peak};
use deeptime_core::alloc::CountingAlloc;

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_a_large_allocation() {
        let before = in_use();
        let v = vec![0u8; 64 << 20];
        assert!(
            in_use() >= before + (64 << 20) - (8 << 20),
            "in_use did not grow"
        );
        assert!(peak() >= 64 << 20);
        drop(v);
        // The freed block is kept for reuse, and counted, until the cache is released.
        deeptime_core::alloc::release_cache();
        // Other test threads allocate concurrently, so only check the order of magnitude.
        assert!(in_use() < before + (32 << 20));
    }
}

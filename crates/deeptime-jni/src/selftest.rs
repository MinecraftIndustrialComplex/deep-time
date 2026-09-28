// SPDX-License-Identifier: LGPL-3.0-or-later
//! `selfTest` (plan §6.1): sums `(i × 2654435761) mod 2^64` for `i = 0 … 2^20 − 1` with a Rayon
//! parallel reduction. Wrapping integer arithmetic makes the sum identical on every platform and in
//! any reduction order, so the Java loader recomputes it in a plain loop and compares; no checksum
//! has to come from running a cross-built library. Its job is to prove the library links, runs
//! Rayon and returns.

use rayon::prelude::*;

use deeptime_core::CoreError;

/// Number of terms.
pub const TERMS: u64 = 1 << 20;
/// Knuth's multiplicative-hash constant.
pub const MULTIPLIER: u64 = 2_654_435_761;
/// The expected sum (also computed by `NativeLoader.expectedSelfTest()` in Java).
pub const EXPECTED: u64 = 1_997_318_690_096_807_936;

/// The parallel reduction, on a private pool that is dropped afterwards (the global Rayon pool is
/// never started inside the JVM).
pub fn run() -> Result<u64, CoreError> {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .thread_name(|i| format!("deeptime-rayon-selftest-{i}"))
        .build()
        .map_err(|e| CoreError::InvalidState(format!("cannot start the Rayon pool: {e}")))?;
    Ok(pool.install(|| {
        (0..TERMS)
            .into_par_iter()
            .map(|i| i.wrapping_mul(MULTIPLIER))
            .reduce(|| 0, u64::wrapping_add)
    }))
}

/// The same sum, sequentially.
pub fn sequential() -> u64 {
    (0..TERMS).fold(0u64, |acc, i| acc.wrapping_add(i.wrapping_mul(MULTIPLIER)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_equals_sequential_equals_constant() {
        assert_eq!(sequential(), EXPECTED);
        assert_eq!(run().unwrap(), EXPECTED);
    }
}

#!/usr/bin/env bash
# TEMP: force-balance task 7 test sweep (removed before landing).
cd "$(dirname "$0")/../.."
deeptime-admit run --kind cargo:test -- cargo test --release -p deeptime-core --lib -- --ignored temp_sweep_welds --nocapture 2>&1 | grep -E "SWEEP|test result|panicked|error"
echo SWEEP_DONE

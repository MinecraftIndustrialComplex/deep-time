#!/usr/bin/env bash
# TEMP: force-balance task 7 test sweeps (removed before landing).
cd "$(dirname "$0")/../.."
deeptime-admit run --kind cargo:test -- cargo test --release -p deeptime-core --lib -- --ignored temp_sweep_welds --nocapture 2>&1 | grep -E "SWEEP|test result|panicked|error" 
deeptime-admit run --kind cargo:test -- cargo test --release -p deeptime-core --lib -- structure::tests::the_pipeline_bakes_ordered structure::tests::the_stage_never_panics 2>&1 | grep -E "^test |test result|panicked|longest" 
echo SWEEP_DONE

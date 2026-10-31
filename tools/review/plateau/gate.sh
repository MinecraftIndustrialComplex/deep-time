#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The plateau lane's checks on the VM (run detached through mac.sh, inside the dev shell):
#   gate.sh quick   fmt, clippy on deeptime-core and deeptime-cli, the lane's tests
#   gate.sh full    fmt, clippy on the workspace, the workspace's tests (--no-fail-fast)
#   gate.sh bake    the bake test alone, with its print of the test planet's biomes and water
set -uo pipefail
cd "$(dirname "$0")/../../.."
mode=${1:-quick}
cargo fmt --all -- --check && echo "fmt-ok"
if [ "$mode" = bake ]; then
    deeptime-admit run --kind cargo:test -- cargo test -p deeptime-core --lib -- bake::tests::a_planet_run_bakes_every_chunk_time_layer --nocapture 2>&1 \
        | grep -E 'bake test|^test |^test result|FAILED|panicked|assertion' | head -40
elif [ "$mode" = full ]; then
    deeptime-admit run --kind cargo:clippy -- cargo clippy --workspace --all-targets -- -D warnings && echo "clippy-ok"
    deeptime-admit run --kind cargo:test -- cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test result|FAILED|panicked|failures:|^     Running' | head -80
else
    deeptime-admit run --kind cargo:clippy -- cargo clippy -p deeptime-core -p deeptime-cli --all-targets -- -D warnings && echo "clippy-ok"
    deeptime-admit run --kind cargo:test -- cargo test -p deeptime-core --lib -- closed plateau lakes erosion_bands routing young surface::climate runoff_budget hydro:: one_surface_step --nocapture 2>&1 \
        | grep -E 'm/Myr over|interior|spill|first step|weights|runoff|Tibet|Verkhoyansk|biotemperature|^test result|FAILED|panicked|assertion|left|right|one step' | head -400
fi

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The plateau lane's checks on the VM (run detached through mac.sh, inside the dev shell):
#   gate.sh quick   fmt, clippy on deeptime-core and deeptime-cli, the lane's tests
#   gate.sh full    fmt, clippy on the workspace, the whole deeptime-core test suite
set -uo pipefail
cd "$(dirname "$0")/../../.."
mode=${1:-quick}
cargo fmt --all -- --check && echo "fmt-ok"
if [ "$mode" = full ]; then
    deeptime-admit run --kind cargo:clippy -- cargo clippy --workspace --all-targets -- -D warnings && echo "clippy-ok"
    deeptime-admit run --kind cargo:test -- cargo test -p deeptime-core 2>&1 | grep -E '^test result|FAILED|panicked|failures:' | head -40
else
    deeptime-admit run --kind cargo:clippy -- cargo clippy -p deeptime-core -p deeptime-cli --all-targets -- -D warnings && echo "clippy-ok"
    deeptime-admit run --kind cargo:test -- cargo test -p deeptime-core --lib -- closed plateau lakes erosion_bands routing --nocapture 2>&1 \
        | grep -E 'm/Myr over|interior|^test result|FAILED|panicked|assertion' | head -60
fi

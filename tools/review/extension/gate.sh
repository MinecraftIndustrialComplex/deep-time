#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The continental-extension lane's checks on the VM (run detached through mac.sh, inside the dev shell):
#   gate.sh quick   fmt, clippy on deeptime-core and deeptime-cli, the tests of extension, settle
#                   and the gap budget
#   gate.sh full    fmt, clippy on the workspace, every test of the workspace (--no-fail-fast)
set -uo pipefail
cd "$(dirname "$0")/../../.."
mode=${1:-quick}
cargo fmt --all -- --check && echo "fmt-ok"
if [ "$mode" = full ]; then
    deeptime-admit run --kind cargo:clippy -- cargo clippy --workspace --all-targets -- -D warnings && echo "clippy-ok"
    deeptime-admit run --kind cargo:test -- cargo test --workspace --no-fail-fast 2>&1 \
        | grep -E '^test result|FAILED|panicked|failures:|^    [a-z_:]+$' | head -80
else
    deeptime-admit run --kind cargo:clippy -- cargo clippy -p deeptime-core -p deeptime-cli --all-targets -- -D warnings && echo "clippy-ok"
    for f in extension settle gap_budget rifts_stretching; do
        deeptime-admit run --kind cargo:test -- cargo test -p deeptime-core --lib -- "$f" --nocapture 2>&1 \
            | grep -E '^test |^test result|thickest|FAILED|panicked|assertion|left|right' | head -60
    done
fi

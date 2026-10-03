#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The cargo and Gradle gates of the cold-desert and schist branch, one log.
#   tools/remote/mac.sh -- tools/review/cold-cargo.sh
set -uo pipefail
cd "$(cd "$(dirname "$0")/../.." && pwd)"
export CARGO_BUILD_JOBS=8
echo "### fmt"; cargo fmt --check && echo FMT-OK
echo "### clippy"; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5
echo "### test"; cargo test --workspace 2>&1 | grep -E "^test result|FAILED|failed|panicked" | sort | uniq -c
echo "### deny"; cargo deny check licenses bans 2>&1 | tail -3
echo "### gradle"; (cd neoforge && ./gradlew build test 2>&1 | tail -15; python3 ../tools/ci/junit-summary.py build/test-results/test)
echo CARGO-GATES-DONE

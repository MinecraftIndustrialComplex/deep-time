#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The branch gates in one go (CLAUDE.md, "GameTests run once per merge batch": an agent on a branch
# runs cargo fmt, clippy, test, deny and `./gradlew build test`, not the GameTests). Meant for the
# test machine, detached, so a dropped laptop session does not lose the run:
#
#   tools/remote/mac.sh -- 'nohup setsid tools/dev/gates.sh <log> >/dev/null 2>&1 < /dev/null &'
#   tools/remote/mac.sh --no-sync -- 'tail -n 40 <log>'
#
# Every gate runs whatever the one before it did; the log ends with a line `GATES fmt=… clippy=…
# test=… deny=… gradle=…` (0 = passed) and the JUnit summary.
set -u
LOG="${1:?usage: tools/dev/gates.sh <log file>}"
cd "$(dirname "$0")/../.."
{
    echo "gates: $(date -u +%Y-%m-%dT%H:%M:%SZ) in $PWD"
    cargo fmt --all --check; FMT=$?
    cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1 | tail -n 60; CLIPPY=${PIPESTATUS[0]}
    cargo test --workspace --locked 2>&1 | grep -E 'test result|FAILED|panicked|failed|^error' ; TEST=${PIPESTATUS[0]}
    cargo deny check licenses bans 2>&1 | tail -n 3; DENY=${PIPESTATUS[0]}
    (cd neoforge && ./gradlew build test 2>&1 | tail -n 40; exit "${PIPESTATUS[0]}"); GRADLE=$?
    python3 tools/ci/junit-summary.py neoforge/build/test-results/test
    echo "GATES fmt=$FMT clippy=$CLIPPY test=$TEST deny=$DENY gradle=$GRADLE $(date -u +%H:%M:%SZ)"
} > "$LOG" 2>&1

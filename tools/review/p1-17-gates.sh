#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-17 gates, on the test VM (tools/remote/mac.sh --pack -- tools/remote/detach.sh start <name>
# --kind light tools/review/p1-17-gates.sh): cargo fmt, clippy, test, deny; Gradle build and test;
# the MIC server set; the GameTests standalone and with MIC. Each heavy step is admitted on its own.
# One summary line per step in review/out/p1-17/gates.txt, logs beside it.
#   tools/review/p1-17-gates.sh [cargo|gradle|gametest]...   (default: all)
set -uo pipefail
cd "$(dirname "$0")/../.."
dir=review/out/p1-17
mkdir -p "$dir"
out=$dir/gates.txt
step() {
    local name="$1"; shift
    "$@" > "$dir/gate-$name.log" 2>&1
    local rc=$?
    echo "$(date -u +%T) $name rc=$rc $(grep -v '^admit:' "$dir/gate-$name.log" | tail -1 | cut -c1-200)" | tee -a "$out"
}
for s in ${*:-cargo gradle gametest}; do
case "$s" in
cargo)
    step fmt cargo fmt --all --check
    step clippy deeptime-admit run --kind cargo:clippy -- cargo clippy --workspace --all-targets -- -D warnings
    step cargotest deeptime-admit run --kind cargo:test -- cargo test --workspace --no-fail-fast
    grep -E "^test result" "$dir/gate-cargotest.log" | awk '{p+=$4; f+=$6; i+=$8} END {print "cargo test: " p " passed, " f " failed, " i " ignored"}' | tee -a "$out"
    step deny deeptime-admit run --kind cargo:other -- cargo deny check licenses bans ;;
gradle)
    step gradle deeptime-admit run --kind gradle:build -- bash -c 'cd neoforge && ./gradlew build test && python3 ../tools/ci/junit-summary.py build/test-results/test' ;;
gametest)
    step install-mic deeptime-admit run --kind install -- tools/smoke/install.sh mic
    step gt-standalone ./gametest.sh
    step gt-mic ./gametest.sh -PwithMic ;;
esac
done
echo "$(date -u +%T) ALL DONE" | tee -a "$out"

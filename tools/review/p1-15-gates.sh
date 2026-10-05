#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-15 gates, on the test VM through tools/remote/mac.sh (detached): cargo fmt, clippy, test, deny,
# Gradle build and test, then the GameTests (standalone, MIC, EV). One summary line per step in
# review/out/p1-15-real/gates.txt.
#   tools/review/p1-15-gates.sh [cargo|gradle|gametest]...   (default: all)
set -uo pipefail
cd "$(dirname "$0")/../.."
mkdir -p review/out/p1-15-real
out=review/out/p1-15-real/gates.txt
step() {
    local name="$1"; shift
    "$@" > "review/out/p1-15-real/gate-$name.log" 2>&1
    echo "$name rc=$? $(tail -1 "review/out/p1-15-real/gate-$name.log" | cut -c1-200)" | tee -a "$out"
}
want() { [ $# -eq 0 ] || return 0; return 0; }
sel="${*:-cargo gradle gametest}"
for s in $sel; do
case "$s" in
cargo)
    step fmt cargo fmt --check
    step clippy deeptime-admit run --kind cargo:clippy -- cargo clippy --workspace --all-targets -- -D warnings
    step cargotest deeptime-admit run --kind cargo:test -- cargo test --workspace
    step deny cargo deny check licenses bans ;;
gradle)
    step gradle deeptime-admit run --kind gradle:build -- bash -c 'cd neoforge && ./gradlew build test && python3 ../tools/ci/junit-summary.py build/test-results/test' ;;
gametest)
    step gt-standalone ./gametest.sh
    step gt-mic ./gametest.sh -PwithMic
    step gt-ev ./gametest.sh -PwithEv ;;
esac
done

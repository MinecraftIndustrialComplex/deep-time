#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Force-balance tasks 7 and 8: the branch's gates in one detached job on the test VM, each heavy
# step admitted on its own. Start from the repo root:
#   tools/remote/mac.sh -- tools/remote/detach.sh start fb-gates --kind light tools/review/fb-gates.sh [rust|gradle|gt|all]
# Logs go to target/fb-gates-*.log; a summary to stdout.
set -u
cd "$(dirname "$0")/../.."
WHAT=${1:-all}
a() { deeptime-admit run --kind "$1" -- "${@:2}"; }
if [ "$WHAT" = all ] || [ "$WHAT" = rust ]; then
  echo "== fmt"; cargo fmt --all -- --check < /dev/null > target/fb-gates-fmt.log 2>&1; echo "fmt exit $?"; head -c 4000 target/fb-gates-fmt.log
  echo "== clippy"; a cargo:clippy cargo clippy --workspace --all-targets -- -D warnings < /dev/null > target/fb-gates-clippy.log 2>&1; echo "clippy exit $?"; grep -E "^(warning|error)" -A6 target/fb-gates-clippy.log | head -80
  echo "== deny"; cargo deny check licenses bans < /dev/null > target/fb-gates-deny.log 2>&1; echo "deny exit $?"; tail -3 target/fb-gates-deny.log
  echo "== test"; a cargo:test cargo test --workspace --no-fail-fast < /dev/null > target/fb-gates-test.log 2>&1; echo "test exit $?"
  grep -E "^test result|FAILED|panicked" target/fb-gates-test.log | head -80
  grep -A8 "panicked at" target/fb-gates-test.log | head -120
fi
if [ "$WHAT" = all ] || [ "$WHAT" = gradle ]; then
  echo "== gradle"; (cd neoforge && a gradle:build ./gradlew build test < /dev/null > ../target/fb-gates-gradle.log 2>&1); echo "gradle exit $?"
  tail -5 target/fb-gates-gradle.log; python3 tools/ci/junit-summary.py neoforge/build/test-results/test 2>&1 | tail -5
fi
summ() { grep -hE "required tests (passed|failed)|tests? failed|All [0-9]+ required|\[gametest-summary\]|FAILED|failed with|::.*fail" "$1" | tail -25; }
if [ "$WHAT" = all ] || [ "$WHAT" = gt ]; then
  echo "== gametest standalone"; ./gametest.sh < /dev/null > target/fb-gates-gt.log 2>&1; echo "exit $?"; summ target/fb-gates-gt.log
  echo "== install mic"; a install tools/smoke/install.sh mic < /dev/null > target/fb-gates-install-mic.log 2>&1; echo "exit $?"; tail -2 target/fb-gates-install-mic.log
  echo "== gametest mic"; ./gametest.sh -PwithMic < /dev/null > target/fb-gates-gt-mic.log 2>&1; echo "exit $?"; summ target/fb-gates-gt-mic.log
fi
echo GATES_DONE

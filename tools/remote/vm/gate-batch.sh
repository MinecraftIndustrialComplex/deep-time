#!/usr/bin/env zsh
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The coordinator's gate batch on master (CLAUDE.md: GameTests run once per merge batch). Runs on the
# test VM against the main checkout's copy (/data/ban/mic/mods/deep-time, synced by mac.sh). Every
# heavy step admits itself (deeptime-admit; Ben 2026-10-03), so the GameTest sets run side by side,
# each in its own server copy and ports. Start it detached, once (two batches double the load):
#
#   tools/remote/mac.sh --pack -- true    # sync master and the MIC pack first
#   ssh ban@<vm> 'pgrep -f "^zsh .*gate-batch.sh" >/dev/null && echo ALREADY || { setsid nohup \
#       /data/ban/mic/mods/deep-time/tools/remote/vm/gate-batch.sh > /data/ban/deeptime-cache/batch.log 2>&1 < /dev/null & }'
#
# The summary (grep -v '^admit:' batch.log) shows fmt, clippy, the test counts, JUnit and each
# GameTest set's "required tests" line.
cd /data/ban/mic/mods/deep-time
export GRADLE_USER_HOME=/data/ban/deeptime-cache/gradle CARGO_HOME=/data/ban/deeptime-cache/cargo XWIN_CACHE_DIR=/data/ban/deeptime-cache/xwin TMPDIR=/data/ban/deeptime-cache/tmp DEEPTIME_REMOTE=mac CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only
P=$(ls -dt /data/ban/deeptime-cache/devshell-*(N) | head -1)
[ -n "$P" ] || { echo "gate-batch: no dev shell profile yet; run tools/remote/mac.sh -- true first" >&2; exit 1; }
run(){ nix develop $P -c zsh -c "$1"; }
A(){ deeptime-admit run --kind "$1" -- "${@:2}"; }
echo "== batch $(git rev-parse --short HEAD 2>/dev/null) $(date +%H:%M)"
run 'cargo fmt --all --check && echo fmt-ok'
A cargo:clippy nix develop $P -c zsh -c 'cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1 | grep -E "^(error|warning)" -A7 | head -30'
A cargo:test nix develop $P -c zsh -c 'cargo test --workspace --locked 2>&1 | grep -E "^test result|FAILED|panicked|^error" | sort | uniq -c | sort -rn | head -15'
run 'cargo deny check licenses bans 2>&1 | tail -1'
# The pages' own checks (projection parity, world map, deposits overlay): light, so not admitted.
# Inside the dev shell, since worldmap-check.js builds its fixture with cargo.
for c in tools/viewer/*-check.js; do run "nix shell nixpkgs#nodejs -c node $c 2>&1 | tail -1 | cut -c1-200"; done
A gradle:build nix develop $P -c zsh -c 'cd neoforge && ./gradlew build test --console=plain 2>&1 | grep -E "BUILD|FAILED|error:" | head -8; python3 ../tools/ci/junit-summary.py build/test-results/test 2>&1 | tail -1'
A cargo:build nix develop $P -c zsh -c 'cargo build --release -p deeptime-cli 2>&1 | tail -1'
echo "== gametests $(date +%H:%M)"
for p in mic ev terrablender ee; do
    run "tools/smoke/install.sh $p > /tmp/dt_inst_$p.log 2>&1" || tail -5 /tmp/dt_inst_$p.log
done
for f in "" -PwithMic -PwithEv -PwithTerraBlender -PwithEe; do
  ( run "./gametest.sh $f > /tmp/dt_batch_gt$f.log 2>&1"; echo "-- ${f:-standalone}"; grep -E "required tests|gametest-result\].*FAIL" /tmp/dt_batch_gt$f.log | cut -c1-400 | head -10 ) &
done
wait
echo "== done $(date +%H:%M)"

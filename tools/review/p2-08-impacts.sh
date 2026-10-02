#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P2-08's evidence run (on the test machine, detached, from the repo root):
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start p2-08 tools/review/p2-08-impacts.sh
#   tools/remote/mac.sh --no-sync --pull review/out/p2-08 -- tools/remote/detach.sh wait p2-08
#
# Runs the quick 16k planet for 500 Myr from scattered continents (the invocation of STATUS.md's
# build log), prints the impact stage's summary line (impacts by size, what became of them, how
# many show at the present) and draws a young and an old crater and a few more
# (tools/review/p2-08-craters.py) into review/out/p2-08/. The run stays under
# /Volumes/Graphide/ban/deeptime-runs/p2-08/ until it is deleted by hand (`rm -rf`, when the task
# lands).
#
#   tools/review/p2-08-impacts.sh [<name> [<extra deeptime run arguments>...]]
#       <name>: the run's folder and the pictures' (default q16); e.g.
#       tools/review/p2-08-impacts.sh flux10 --set impacts.flux=10
set -uo pipefail
cd "$(dirname "$0")/../.."
NAME="${1:-q16}"
[ $# -gt 0 ] && shift
RUNS=/Volumes/Graphide/ban/deeptime-runs/p2-08
RUN="$RUNS/$NAME"
OUT="review/out/p2-08/$NAME"
mkdir -p "$RUNS" "$OUT"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-8}"
cargo build --release -q -p deeptime-cli || exit 1
rm -rf "$RUN"
/usr/bin/time -v target/release/deeptime run --preset earthlike_quick_16k --start scattered_continents \
    --length-myr 500 --threads 4 "$@" --out "$RUN" > "$OUT/run.log" 2>&1
status=$?
grep -a -E "impacts_final|volcanoes_final|amplify_final|Elapsed|Maximum resident" "$OUT/run.log"
[ "$status" = 0 ] || { tail -n 20 "$OUT/run.log"; exit "$status"; }
PY='(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.matplotlib])'
nix shell --impure --expr "$PY" -c python3 tools/review/p2-08-craters.py --world-data "$RUN" \
    --deeptime target/release/deeptime --out "$OUT"

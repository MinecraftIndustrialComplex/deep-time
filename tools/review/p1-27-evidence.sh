#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-27: the before/after evidence on the Mac (via tools/remote/mac.sh --pull review/out/p1-27/evidence):
# the coupled pipeline of earthlike_quick_16k for <length> Myr on both starts, with the "before"
# build (BEFORE, default the before-src tree) and this tree's, keyframes of the regime, elevation,
# provinces, satellite and volcano layers every 50 Myr (a viewer bundle per run: open its
# index.html), and each run's summary lines (the margins line carries the active / passive series
# every 10 Myr).
#
#   [LABELS="after"] tools/review/p1-27-evidence.sh <length Myr> "<starts>" <seed> [-- --set k=v ...]
set -euo pipefail
LEN=$1; STARTS=$2; SEED=$3; shift 3
[ "${1:-}" = "--" ] && shift
R=/Volumes/Graphide/ban/deeptime-runs/p1-27/evidence
OUT=review/out/p1-27/evidence
BEFORE=${BEFORE:-/Volumes/Graphide/ban/deeptime-runs/p1-27/before-src/target/release/deeptime}
mkdir -p "$R" "$OUT"
cargo build --release -q -p deeptime-cli
for start in $STARTS; do
    for label in ${LABELS:-before after}; do
        rm -rf "$R/view-$label-$start"
        if [ "$label" = before ]; then
            BIN="$BEFORE" VIEW="$R/view-$label-$start" tools/review/p1-27-pipeline.sh "$R" "$label" "$start" "$SEED" "$LEN"
        else
            VIEW="$R/view-$label-$start" tools/review/p1-27-pipeline.sh "$R" "$label" "$start" "$SEED" "$LEN" -- "$@"
        fi
        rm -rf "$OUT/$label-$start"
        cp -R "$R/view-$label-$start" "$OUT/$label-$start"
        cp "$R/$label-$start-$SEED.txt" "$OUT/$label-$start.txt"
        rm -rf "$R/view-$label-$start"
    done
done

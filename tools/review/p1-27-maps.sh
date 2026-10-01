#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-27: margin maps (`deeptime margins`) of a plates-only run at chosen steps (run on the Mac via
# tools/remote/mac.sh). Uses the plates-only preset tools/review/p1-27-margins.sh writes.
#
#   tools/review/p1-27-maps.sh <out dir> <label> <start> <seed> "<steps…>" [-- --set k=v ...]
#
# Writes <out dir>/<label>-<start>-<seed>-s<step>.png and prints each map's margin line.
set -euo pipefail
OUT=${1:?out dir}; LABEL=${2:?label}; START=${3:?start}; SEED=${4:?seed}; STEPS=${5:?steps}; shift 5
[ "${1:-}" = "--" ] && shift
BIN=target/release/deeptime
cargo build --release -q -p deeptime-cli
PRESET="$OUT/plates_only.json"
[ -f "$PRESET" ] || { echo "run tools/review/p1-27-margins.sh first (it writes $PRESET)" >&2; exit 1; }
W="$OUT/maps-$LABEL-$START-$SEED"
mkdir -p "$W"
prev=""; done_steps=0
for s in $STEPS; do
    n=$((s - done_steps))
    if [ -z "$prev" ]; then
        "$BIN" stage plates --preset "$PRESET" --start "$START" --length-myr 750 --set "seed=$SEED" "$@" \
            --max-steps "$n" --out "$W/ck$s" --work "$W/work" > "$W/log$s.txt" 2>&1
    else
        "$BIN" stage plates --in "$prev" --max-steps "$n" --out "$W/ck$s" --work "$W/work" > "$W/log$s.txt" 2>&1
    fi
    "$BIN" margins --in "$W/ck$s" --out "$OUT/$LABEL-$START-$SEED-s$s.png"
    [ -n "$prev" ] && rm -f "$prev"
    prev="$W/ck$s"; done_steps=$s
done
rm -rf "$W"

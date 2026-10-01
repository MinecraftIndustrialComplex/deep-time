#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-27: the coupled pipeline (plates, climate, surface, volcanoes, structure) of earthlike_quick_16k
# on the Mac (via tools/remote/mac.sh), printing the margin, arc-base, hypsometry, climate and plate
# lines; with VIEW=<dir> also a viewer bundle of the regime, elevation, provinces, satellite and
# volcano layers at every keyframe.
#
#   tools/review/p1-27-pipeline.sh <out dir> <label> <start> <seed> <length Myr> [-- --set k=v ...]
set -euo pipefail
OUT=${1:?out dir}; LABEL=${2:?label}; START=${3:?start}; SEED=${4:?seed}; LEN=${5:?length}; shift 5
[ "${1:-}" = "--" ] && shift
BIN=target/release/deeptime
cargo build --release -q -p deeptime-cli
mkdir -p "$OUT"
f="$OUT/$LABEL-$START-$SEED.txt"
"$BIN" stage volcanoes --preset earthlike_quick_16k --start "$START" --length-myr "$LEN" \
    --set "seed=$SEED" --work "$OUT/work-$LABEL-$START-$SEED" \
    ${VIEW:+--viewer "$VIEW" --viewer-layers regime,elevation,provinces,satellite,volcanoes/map} \
    ${CKPT:+--out "$CKPT"} "$@" > "$f" 2>&1 || { tail -20 "$f"; exit 1; }
rm -rf "$OUT/work-$LABEL-$START-$SEED"
echo "== $LABEL $START seed=$SEED ${LEN} Myr"
grep -E "margins records|volcanoes_final|hypsometry|climate_final|plates_final|seconds=" "$f" \
    | sed -E 's/series\(t:active\/passive\)=[^ ]*//' | cut -c1-1200

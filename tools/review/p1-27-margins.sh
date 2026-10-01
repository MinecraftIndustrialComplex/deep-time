#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-27: continental margins by tectonic setting over whole runs, plates only (run on the Mac via
# tools/remote/mac.sh). The preset is earthlike_quick_16k's tectonics with every later stage
# removed, so a 750-Myr run takes seconds; each run prints `margins …` (the shares measured every
# 10 Myr, `plates::stats::margins`) and the plate summary.
#
#   tools/review/p1-27-margins.sh <out dir> <label> [--seeds "1 2"] [--starts "a b"]
#                                 [--length-myr L] [--preset P] [-- --set k=v ...]
#
# Writes <out dir>/<label>-<start>-<seed>.txt. With EVENTS=1 also prints the Wilson-cycle event
# statistics of each run (tools/review/p1-27-events.py).
set -euo pipefail
OUT=${1:?out dir}; LABEL=${2:?label}; shift 2
SEEDS="1 2"; STARTS="supercontinent scattered_continents"; LEN=750
PRESET=crates/deeptime-core/presets/earthlike_quick_16k.json
while [ $# -gt 0 ]; do
    case "$1" in
        --seeds) SEEDS="$2"; shift 2 ;;
        --starts) STARTS="$2"; shift 2 ;;
        --length-myr) LEN="$2"; shift 2 ;;
        --preset) PRESET="$2"; shift 2 ;;
        --) shift; break ;;
        *) break ;;
    esac
done
mkdir -p "$OUT"
BIN=target/release/deeptime
cargo build --release -q -p deeptime-cli
python3 - "$PRESET" "$OUT/plates_only.json" <<'EOF'
import json, sys
d = json.load(open(sys.argv[1]))
for k in ['surface', 'climate', 'structure', 'volcanoes', 'deposits', 'young', 'amplify', 'biomes']:
    d.pop(k, None)
d['name'] = 'p1-27-plates-only'
d['tectonics']['l1_every_steps'] = 0
d['keyframes'] = {'every_myr': 250.0, 'size': [256, 128]}
json.dump(d, open(sys.argv[2], 'w'), indent=1)
EOF
for start in $STARTS; do
    for seed in $SEEDS; do
        f="$OUT/$LABEL-$start-$seed.txt"
        "$BIN" stage plates --preset "$OUT/plates_only.json" --start "$start" --length-myr "$LEN" \
            --work "$OUT/work-$LABEL-$start-$seed" --set "seed=$seed" \
            ${EVENTS:+--viewer "$OUT/view-$LABEL-$start-$seed" --viewer-layers regime} "$@" > "$f" 2>&1 \
            || { tail "$f"; exit 1; }
        rm -rf "$OUT/work-$LABEL-$start-$seed"
        if [ -n "${EVENTS:-}" ]; then
            python3 tools/review/p1-27-events.py "$OUT/view-$LABEL-$start-$seed/events.json"
            rm -rf "$OUT/view-$LABEL-$start-$seed"
        fi
        grep -o 'margins records.*final_coast_km=[0-9]*' "$f" | sed "s/^/$start seed=$seed /"
        grep -o 'plates_final=.*resamplings=[0-9]*' "$f" | head -1
    done
done

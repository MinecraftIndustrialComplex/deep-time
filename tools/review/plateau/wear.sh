#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The plateau lane (VM, cached pool folders only, no simulation): what wears the high ground.
#   wear.sh <tag>=<pool argkey folder> [...]   -> /data/ban/deeptime-runs/plateau/wear-<tag>.txt
# Run detached: tools/remote/mac.sh -- tools/remote/detach.sh start plateau-wear --kind light bash tools/review/plateau/wear.sh ...
set -uo pipefail
cd "$(dirname "$0")/../../.."
PY=/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3
OUT=/data/ban/deeptime-runs/plateau
mkdir -p "$OUT"
ONLY=${ONLY:-}
for pair in "$@"; do
    tag=${pair%%=*} folder=${pair#*=}
    if [ -n "$ONLY" ]; then
        nice -n 19 "$PY" -I "tools/review/plateau/$ONLY.py" "$folder" > "$OUT/$ONLY-$tag.txt" 2> "$OUT/$ONLY-$tag.log"
        echo "done $tag $ONLY"
        continue
    fi
    {
        nice -n 19 "$PY" -I tools/review/plateau/bands.py "$folder"
        HI=3000 VERBOSE=1 nice -n 19 "$PY" -I tools/review/plateau/cohort.py "$folder"
        HI=4000 nice -n 19 "$PY" -I tools/review/plateau/cohort.py "$folder"
        MODE=chain HI=3000 nice -n 19 "$PY" -I tools/review/plateau/cohort.py "$folder"
        nice -n 19 "$PY" -I tools/review/plateau/climate.py "$folder"
        nice -n 19 "$PY" -I tools/review/plateau/closed.py "$folder"
        nice -n 19 "$PY" -I tools/review/plateau/series.py "$folder"
        nice -n 19 "$PY" -I tools/review/orogen/hiseries.py "$folder"
    } > "$OUT/wear-$tag.txt" 2> "$OUT/wear-$tag.log"
    echo "done $tag"
done

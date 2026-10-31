#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The plateau lane (VM, cached pool folders only): the erosion recalibration's table (tools/review/erosion-recal.py,
# the land's erosion by relief and by region against Montgomery & Brandon 2002, ERO-172) for pool arms.
#   recal.sh <label>=<pool argkey folder> [...]   -> /data/ban/deeptime-runs/plateau/recal.txt
# The pool's runs are <start>-<seed>/summary.txt; erosion-recal.py reads <label>-<start>-<seed>-<length>.txt, so the
# summaries are linked under that name first.
set -uo pipefail
cd "$(dirname "$0")/../../.."
PY=/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3
OUT=/data/ban/deeptime-runs/plateau
LINKS=$OUT/recal-links
mkdir -p "$LINKS"
labels=()
for pair in "$@"; do
    label=${pair%%=*} folder=${pair#*=}
    labels+=("$label")
    for run in "$folder"/*/; do
        name=$(basename "$run")
        [ -f "$run/summary.txt" ] || continue
        ln -sf "$run/summary.txt" "$LINKS/$label-${name%-*}-${name##*-}-1000.txt"
    done
done
nice -n 19 "$PY" -I tools/review/erosion-recal.py "$LINKS" 1000 "1 2 3 4" "supercontinent scattered_continents" \
    "${labels[@]}" > "$OUT/recal.txt" 2> "$OUT/recal.log"
echo "done recal"

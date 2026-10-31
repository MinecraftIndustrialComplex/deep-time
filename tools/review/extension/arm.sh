#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The continental extension's arm read (VM, cached pool folders only, no simulation).
#   arm.sh <tag>=<pool argkey folder> [...]   -> /data/ban/deeptime-runs/extension/arm-<tag>.txt
# Per arm: the land ledger (whole run and the last 500 Myr), the rift totals, the crust budget against Stern & Scholl,
# then the plateau lane's wear read (wear.sh, its own file under /data/ban/deeptime-runs/plateau/).
# Run detached: tools/remote/mac.sh -- tools/remote/detach.sh start ext-arm --kind light tools/review/extension/arm.sh ...
set -uo pipefail
cd "$(dirname "$0")/../../.."
OUT=/data/ban/deeptime-runs/extension
mkdir -p "$OUT"
for pair in "$@"; do
    tag=${pair%%=*} folder=${pair#*=}
    runs=()
    for d in "$folder"/*/; do
        [ -f "$d/summary.txt" ] && runs+=("${d%/}")
    done
    sums=()
    for d in "${runs[@]}"; do sums+=("$d/summary.txt"); done
    {
        echo "== $tag: ${#runs[@]} runs in $folder"
        echo "== the land ledger, the whole run"
        python3 -I tools/review/uplift/ledger.py --all "${sums[@]}"
        echo "== the land ledger, the last 500 Myr"
        python3 -I tools/review/uplift/ledger.py --all --from -500 "${sums[@]}"
        echo "== the rift totals"
        python3 -I tools/review/extension/rift_globals.py "${runs[@]}"
        echo "== the crust budget"
        python3 -I tools/review/loss-budget.py 1000 "${sums[@]}" --label "$tag"
    } > "$OUT/arm-$tag.txt" 2> "$OUT/arm-$tag.log"
    echo "done $tag"
done
tools/review/plateau/wear.sh "$@"

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Placer sites and spawn, before and after the 2026-10-05 changes (partial placer shedding; no
# essentials-based spawn). Bakes earthlike_quick_16k (scattered continents, 500 Myr) for several
# seeds with the shipped essentials as the content bundle (what a first start passes), then reads
# each bake's sites and spawn candidates (tools/review/placer-spawn-compare.py). Runs ON the test VM,
# detached. TAG names the build ("before" from the old commit, "after" from the branch):
#
#   TAG=after tools/remote/mac.sh -- tools/review/placer-spawn-compare.sh bake 1 2 3 4
#   TAG=after tools/remote/mac.sh --no-sync -- tools/review/placer-spawn-compare.sh report 1 2 3 4
#   tools/remote/mac.sh --no-sync -- tools/review/placer-spawn-compare.sh clean
#
# Scratch: /Volumes/Graphide/ban/deeptime-runs/placer-spawn (deleted by `clean`). PRESET overrides
# the preset name, BAKE_EXTRA adds `deeptime run` arguments (e.g. a --set).
set -euo pipefail
RUNS=/Volumes/Graphide/ban/deeptime-runs/placer-spawn
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
mkdir -p "$RUNS"
PRESET="${PRESET:-earthlike_quick_16k}"
TAG="${TAG:-after}"
case "${1:-}" in
bake)
    cargo build --release -p deeptime-cli
    cp target/release/deeptime "$RUNS/deeptime-$TAG"
    python3 - "$RUNS/content-$TAG.json" <<'PY'
import glob, json, sys
ess = [json.load(open(f)) for f in sorted(glob.glob("neoforge/src/main/resources/data/deeptime/deeptime/essentials/*.json"))]
for e in ess:
    e.pop("_comment", None)
json.dump({"essentials": ess}, open(sys.argv[1], "w"))
PY
    for seed in "${@:2}"; do
        d="$RUNS/$TAG-s$seed"
        rm -rf "$d"
        setsid nohup sh -c "deeptime-admit run --kind sim:$PRESET:3t -- '$RUNS/deeptime-$TAG' run --preset $PRESET \
            --start scattered_continents --length-myr 500 --threads 3 --content '$RUNS/content-$TAG.json' \
            --out '$d' --set seed=$seed ${BAKE_EXTRA:-} > '$d.log' 2>&1; echo BAKE-DONE >> '$d.log'" > /dev/null 2>&1 < /dev/null &
    done
    echo "started: ${*:2}"
    ;;
report)
    shift
    args=()
    for seed in "$@"; do
        d="$RUNS/$TAG-s$seed"
        grep -q BAKE-DONE "$d.log" || { echo "seed $seed: not done"; continue; }
        "$RUNS/deeptime-$TAG" dump-deposits --world "$d" --out "$d-deposits.json" > /dev/null
        args+=("$seed=$d")
    done
    python3 tools/review/placer-spawn-compare.py "${args[@]}"
    ;;
clean)
    rm -rf "$RUNS"
    ;;
*) echo "usage: [TAG=<name>] placer-spawn-compare.sh bake <seeds> | report <seeds> | clean" >&2; exit 2 ;;
esac

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-15 spawn survey: does the spawn rule find a porphyry within the essentials' reach on real
# planets? Bakes earthlike_quick_16k (scattered continents, 500 Myr) for several seeds with the
# vanilla essentials as the content bundle (what a first start passes), then reads each bake's
# sites and spawn candidates (tools/review/p1-15-spawn-survey.py). Runs ON the test VM, detached:
#
#   tools/remote/mac.sh -- tools/review/p1-15-spawn-survey.sh bake 1 2 3 4 5 6
#   tools/remote/mac.sh --no-sync -- tools/review/p1-15-spawn-survey.sh report 1 2 3 4 5 6   (SURVEY_ARGS="--reach N --min-mt N")
#   tools/remote/mac.sh --no-sync -- tools/review/p1-15-spawn-survey.sh clean
#
# Scratch: /Volumes/Graphide/ban/deeptime-runs/p1-15-spawn (deleted by `clean`). PRESET overrides the
# preset name, TAG names the folders (default: the preset), BAKE_EXTRA adds `deeptime run` arguments
# (e.g. a --set).
set -euo pipefail
RUNS=/Volumes/Graphide/ban/deeptime-runs/p1-15-spawn
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
mkdir -p "$RUNS"
PRESET="${PRESET:-earthlike_quick_16k}"
TAG="${TAG:-$PRESET}"
case "${1:-}" in
bake)
    cargo build --release -p deeptime-cli
    cp target/release/deeptime "$RUNS/deeptime"
    python3 - "$RUNS/content.json" <<'PY'
import glob, json, sys
ess = [json.load(open(f)) for f in sorted(glob.glob("neoforge/src/main/resources/data/deeptime/deeptime/essentials/*.json"))]
for e in ess:
    e.pop("_comment", None)
json.dump({"essentials": ess}, open(sys.argv[1], "w"))
PY
    for seed in "${@:2}"; do
        d="$RUNS/$TAG-s$seed"
        rm -rf "$d"
        setsid nohup sh -c "deeptime-admit run --kind sim:$PRESET:3t -- '$RUNS/deeptime' run --preset $PRESET \
            --start scattered_continents --length-myr 500 --threads 3 --content '$RUNS/content.json' \
            --out '$d' --set seed=$seed ${BAKE_EXTRA:-} > '$d.log' 2>&1; echo BAKE-DONE >> '$d.log'" > /dev/null 2>&1 < /dev/null &
    done
    echo "started: ${*:2}"
    ;;
report)
    shift
    seeds="$*"
    args=()
    for seed in $seeds; do
        d="$RUNS/$TAG-s$seed"
        grep -q BAKE-DONE "$d.log" || { echo "seed $seed: not done"; continue; }
        "$RUNS/deeptime" dump-deposits --world "$d" --out "$d-deposits.json" > /dev/null
        args+=("$seed=$d")
    done
    python3 tools/review/p1-15-spawn-survey.py "${args[@]}" ${SURVEY_ARGS:-}
    ;;
clean)
    rm -rf "$RUNS"
    ;;
*) echo "usage: p1-15-spawn-survey.sh bake <seeds> | report <seeds> | clean" >&2; exit 2 ;;
esac

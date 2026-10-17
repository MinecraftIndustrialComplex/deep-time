#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Erosion and burial audit (decisions log 2026-10-05, "Investigate and fix"): bakes
# earthlike_quick_16k (scattered continents) for 500 and 1000 Myr with checkpoints, then reads each
# run's erosion and deposition per region type and its porphyry depths (`deeptime erosion-audit`).
# Runs ON the test VM, detached:
#
#   tools/remote/mac.sh -- tools/review/erosion-audit.sh bake <tag> [500] [1000]
#   tools/remote/mac.sh --kind light -- 'BIN=<tag> PRESET=<file> PRESET_KIND=<preset> tools/review/erosion-audit.sh bake <tag2> 500'
#       (a preset variant with an earlier bake's build: nothing to build, the run admits itself)
#   tools/remote/mac.sh --no-sync -- tools/review/erosion-audit.sh report <tag> [500] [1000]   (BIN=<tag> reads with that bake's build)
#   tools/remote/mac.sh --no-sync -- tools/review/erosion-audit.sh clean
#
# Scratch: /Volumes/Graphide/ban/deeptime-runs/erosion-audit (deleted by `clean`). BAKE_EXTRA adds
# `deeptime run` arguments, THREADS the run's threads (default 6), RUNS another scratch folder,
# PRESET a built-in name or a preset file (its admission kind: PRESET_KIND, default the file name).
set -euo pipefail
RUNS="${RUNS:-/Volumes/Graphide/ban/deeptime-runs/erosion-audit}"
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
mkdir -p "$RUNS"
PRESET="${PRESET:-earthlike_quick_16k}"
# The admission kind names the built-in preset a preset file was made from (PRESET_KIND).
PRESET_KIND="${PRESET_KIND:-$(basename "$PRESET" .json)}"
THREADS="${THREADS:-6}"
verb="${1:-}"
tag="${2:-}"
shift 2 || true
lengths="${*:-500 1000}"
case "$verb" in
bake)
    # BIN=<tag> bakes with that earlier bake's build (a preset variant needs no new build).
    if [ -n "${BIN:-}" ]; then
        cp "$RUNS/deeptime-$BIN" "$RUNS/deeptime-$tag"
    else
        cargo build --release -p deeptime-cli
        cp target/release/deeptime "$RUNS/deeptime-$tag"
    fi
    for len in $lengths; do
        d="$RUNS/$tag-$len"
        rm -rf "$d" "$d.log"
        mkdir -p "$d"
        setsid nohup sh -c "deeptime-admit run --kind sim:$PRESET_KIND:${THREADS}t -- '$RUNS/deeptime-$tag' run \
            --preset $PRESET --start scattered_continents --length-myr $len --threads $THREADS \
            --checkpoints --out '$d' ${BAKE_EXTRA:-} > '$d.log' 2>&1; echo BAKE-DONE \$? >> '$d.log'" \
            > /dev/null 2>&1 < /dev/null &
    done
    echo "started $tag: $lengths"
    ;;
report)
    # BIN names the build that reads the runs (default: the tag's own, saved by `bake`).
    bin="$RUNS/deeptime-${BIN:-$tag}"
    for len in $lengths; do
        d="$RUNS/$tag-$len"
        grep -q BAKE-DONE "$d.log" || { echo "$tag-$len: not done"; tail -3 "$d.log"; continue; }
        cp=$(command ls "$d"/checkpoints/0-*.dtcp 2>/dev/null | tail -1 || true)
        "$bin" erosion-audit --world-data "$d" ${cp:+--checkpoint "$cp"} \
            --json "$d-audit.json" > "$d-audit.txt" 2>&1 || true
        echo "== $tag-$len (checkpoint ${cp:-none})"
        cat "$d-audit.txt"
    done
    ;;
clean)
    rm -rf "$RUNS"
    ;;
*) echo "usage: erosion-audit.sh bake|report <tag> [lengths] | clean" >&2; exit 2 ;;
esac

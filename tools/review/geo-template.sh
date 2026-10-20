#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The crust-template and relief-lift review (2026-10-19): ocean-crust census, mountain fill and
# sections of earthlike_quick_16k planets (1000 Myr) baked by two builds, "before" (master) and
# "after" (the branch). Runs ON the test VM; each planet is its own admitted job.
#
#   tools/remote/mac.sh -- tools/review/geo-template.sh build before        # binary of this checkout
#   tools/remote/mac.sh --no-sync -- tools/review/geo-template.sh run before supercontinent 1 [--checkpoints]
#   tools/remote/mac.sh --no-sync -- tools/review/geo-template.sh census before supercontinent 1
#   tools/remote/mac.sh --no-sync -- tools/review/geo-template.sh report    # every censused run
#   tools/remote/mac.sh --no-sync -- tools/review/geo-template.sh status
#   tools/remote/mac.sh --no-sync -- tools/review/geo-template.sh clean
#
# `run` starts detached (setsid) and returns; `<dir>.log` ends with RUN-DONE. `census` reads the
# world with `inspect-column --points`: a 208-block grid (the triage's), and the two review lines
# (1.png and 2.png of the external review, seed 1 supercontinent). `report` writes
# $RUNS/report/{report.md,*.png} (tools/review/geo_template.py, numpy and matplotlib from nixpkgs).
# Scratch: /Volumes/Graphide/ban/deeptime-runs/geo-template (deleted by `clean`).
set -euo pipefail
RUNS=/Volumes/Graphide/ban/deeptime-runs/geo-template
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
mkdir -p "$RUNS"
PRESET=earthlike_quick_16k
# The review lines, block (x, z) end points (the external review's 1.png and 2.png).
LINE1="-44 -1366 -1819 -229 422"
LINE2="-6682 -482 -5633 -1374 460"

line_points() { # x0 z0 x1 z1 n
    python3 -c '
import sys
x0, z0, x1, z1, n = map(float, sys.argv[1:6]); n = int(n)
for i in range(n):
    t = i / (n - 1)
    print(round(x0 + (x1 - x0) * t), round(z0 + (z1 - z0) * t))
' "$@"
}

grid_points() { # step
    python3 -c '
import sys
s = int(sys.argv[1])
for z in range(-3328 + s // 2, 3328, s):
    for x in range(-6656 + s // 2, 6656, s):
        print(x, z)
' "$1"
}

case "${1:-}" in
build)
    tag="$2"
    cargo build --release -p deeptime-cli
    cp target/release/deeptime "$RUNS/deeptime-$tag"
    git log -1 --format='%H %s' > "$RUNS/deeptime-$tag.commit" 2>/dev/null || true
    echo "built $RUNS/deeptime-$tag"
    ;;
run)
    tag="$2" start="$3" seed="$4"; shift 4
    d="$RUNS/$tag-$start-s$seed"
    rm -rf "$d" "$d.log"
    setsid nohup sh -c "deeptime-admit run --kind sim:$PRESET:8t -- '$RUNS/deeptime-$tag' run \
        --preset $PRESET --start $start --length-myr 1000 --threads 8 --out '$d' \
        --set seed=$seed $* > '$d.log' 2>&1; echo RUN-DONE >> '$d.log'" > /dev/null 2>&1 < /dev/null &
    echo "started $d"
    ;;
census)
    tag="$2" start="$3" seed="$4"
    d="$RUNS/$tag-$start-s$seed"
    grep -q RUN-DONE "$d.log" || { echo "$d: not done"; exit 1; }
    bin="$RUNS/deeptime-$tag"
    grid_points 208 > "$d-grid.pts"
    "$bin" inspect-column --world-data "$d" --points "$d-grid.pts" > "$d-grid.jsonl"
    # shellcheck disable=SC2086
    line_points $LINE1 > "$d-line1.pts"
    "$bin" inspect-column --world-data "$d" --points "$d-line1.pts" > "$d-line1.jsonl"
    # shellcheck disable=SC2086
    line_points $LINE2 > "$d-line2.pts"
    "$bin" inspect-column --world-data "$d" --points "$d-line2.pts" > "$d-line2.jsonl"
    "$bin" dump-deposits --world "$d" --out "$d-deposits.json" > /dev/null
    echo "censused $d"
    ;;
report)
    mkdir -p "$RUNS/report"
    nix shell --impure --expr \
        '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.matplotlib])' \
        -c python3 "$REPO/tools/review/geo_template.py" "$RUNS" "$RUNS/report"
    ;;
status)
    for l in "$RUNS"/*.log; do
        [ -e "$l" ] || continue
        printf '%s: %s\n' "$(basename "$l" .log)" "$(tail -n 1 "$l" | cut -c1-120)"
    done
    deeptime-admit status 2>/dev/null | head -30 || true
    ;;
clean)
    rm -rf "$RUNS"
    ;;
*)
    echo "usage: geo-template.sh build TAG | run TAG START SEED [args] | census TAG START SEED | report | status | clean" >&2
    exit 2
    ;;
esac

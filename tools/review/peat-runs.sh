#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Cold wetland (2026-10-03 owner answer, taiga or peatland): one quick 16k planet run per build, detached,
# for before/after comparisons (biome shares by Koppen class through the game's biome source;
# rock-class shares in columns through `strata-check`). Scratch: /Volumes/Graphide/ban/deeptime-runs/peat/.
#
#   tools/remote/mac.sh -- tools/review/peat-runs.sh run <name> [length-myr] [start] [--set k=v]...
#       builds the release CLI, runs `deeptime run` (earthlike_quick_16k, 2 threads) into <name>-world/;
#       the log is <name>.log. `--set biomes.wetland_cold_below=1` is the old rule (swamp wherever thawed).
#   tools/remote/mac.sh --no-sync -- tools/review/peat-runs.sh status | show <name> | clean
set -euo pipefail

RUNS=/Volumes/Graphide/ban/deeptime-runs/peat
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$RUNS"
cd "$REPO"

case "${1:-}" in
    run)
        name="$2"; length="${3:-500}"; start="${4:-scattered_continents}"
        sets=""; shift 4 2>/dev/null || shift $#
        while [ $# -gt 0 ]; do [ "$1" = "--set" ] && sets="$sets --set $2"; shift 2; done
        setsid nohup sh -c "deeptime-admit run --kind cargo:build -- env CARGO_BUILD_JOBS=8 cargo build --release -p deeptime-cli \
            && cp target/release/deeptime '$RUNS/deeptime-$name' \
            && rm -rf '$RUNS/$name-world' \
            && deeptime-admit run --kind sim:earthlike_quick_16k:2t -- '$RUNS/deeptime-$name' run --preset earthlike_quick_16k \
               --start '$start' --length-myr '$length' --threads 2 $sets --out '$RUNS/$name-world'" \
            > "$RUNS/$name.log" 2>&1 < /dev/null &
        echo "started pid $! -> $RUNS/$name.log"
        ;;
    status)
        for f in "$RUNS"/*.log; do [ -e "$f" ] && { echo "== $f"; tail -n 4 "$f"; }; done
        ;;
    show)
        grep -aE "structure_final=|seconds=|Elapsed" "$RUNS/$2.log" || true
        [ -e "$RUNS/$2-strata.json" ] && cat "$RUNS/$2-strata.json"
        ;;
    clean)
        rm -rf "$RUNS"
        ;;
    *)
        sed -n '3,11p' "$0"; exit 2
        ;;
esac

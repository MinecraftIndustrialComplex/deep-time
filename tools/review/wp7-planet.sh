#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Science WP-7 (rock classes, metamorphism, plutons, strata): one quick 16k planet run and its
# `strata-check`, detached, for before/after comparisons. Scratch: /Volumes/Graphide/ban/deeptime-runs/wp7/.
#
#   tools/remote/mac.sh -- tools/review/wp7-planet.sh run <name> [length-myr] [start] [--set PATH=VALUE]...
#       builds the release CLI, runs `deeptime run` (earthlike_quick_16k, 2 threads) into <name>-world/,
#       then `strata-check` (20,000 columns) into <name>-strata.json; the log is <name>.log
#   tools/remote/mac.sh --no-sync -- tools/review/wp7-planet.sh status
#   tools/remote/mac.sh --no-sync -- tools/review/wp7-planet.sh show <name>
#   tools/remote/mac.sh --no-sync -- tools/review/wp7-planet.sh clean
set -euo pipefail

RUNS=/Volumes/Graphide/ban/deeptime-runs/wp7
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$RUNS"
cd "$REPO"

case "${1:-}" in
    run)
        name="$2"; length="${3:-500}"; start="${4:-scattered_continents}"
        shift $(( $# < 4 ? $# : 4 ))
        sets=""; for a in "$@"; do sets="$sets '$a'"; done
        setsid nohup sh -c "CARGO_BUILD_JOBS=8 cargo build --release -p deeptime-cli \
            && cp target/release/deeptime '$RUNS/deeptime-$name' \
            && rm -rf '$RUNS/$name-world' \
            && '$RUNS/deeptime-$name' run --preset earthlike_quick_16k --start '$start' --length-myr '$length' \
               --threads 2 --out '$RUNS/$name-world' $sets \
            && '$RUNS/deeptime-$name' strata-check --world-data '$RUNS/$name-world' --columns 20000 \
               > '$RUNS/$name-strata.json'" > "$RUNS/$name.log" 2>&1 < /dev/null &
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
        sed -n '3,12p' "$0"; exit 2
        ;;
esac

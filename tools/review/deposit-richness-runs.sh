#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Deposit richness, thinner scatter, raw-ore cores and the inherited iron-formation patch field
# (Ben, 2026-10-02): the measurement runs on the test machine, detached. Scratch in
# /Volumes/Graphide/ban/deeptime-runs/deposit-richness/ (deleted when the work lands).
#
#   tools/remote/mac.sh -- tools/review/deposit-richness-runs.sh world <name> [--set PATH=VALUE]...
#       builds the release CLI, copies it as deeptime-<name>, runs earthlike_quick_16k
#       (scattered_continents, 1000 Myr, 4 threads) into <name>-world/, then `deposit-audit`
#       (<name>-audit.md and .json, with the early-game access probe); log <name>.log.
#   tools/remote/mac.sh -- tools/review/deposit-richness-runs.sh audit <name> [deposit-audit args…]
#       the audit again on an existing <name>-world/ with this tree's CLI (no new run).
#   tools/remote/mac.sh --no-sync -- tools/review/deposit-richness-runs.sh status
#   tools/remote/mac.sh --no-sync -- tools/review/deposit-richness-runs.sh clean
set -euo pipefail

RUNS=/Volumes/Graphide/ban/deeptime-runs/deposit-richness
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$RUNS"
cd "$REPO"

case "${1:-}" in
    world)
        name="$2"; shift 2
        sets=""; for a in "$@"; do sets="$sets '$a'"; done
        setsid nohup sh -c "cargo build --release -p deeptime-cli \
            && cp target/release/deeptime '$RUNS/deeptime-$name' \
            && rm -rf '$RUNS/$name-world' \
            && /usr/bin/time -v '$RUNS/deeptime-$name' run --preset earthlike_quick_16k \
               --start scattered_continents --length-myr 1000 --threads 4 --out '$RUNS/$name-world' $sets \
            && '$RUNS/deeptime-$name' deposit-audit --world-data '$RUNS/$name-world' \
               --json '$RUNS/$name-audit.json' > '$RUNS/$name-audit.md' \
            && echo RICH-DONE" > "$RUNS/$name.log" 2>&1 < /dev/null &
        echo "started pid $! -> $RUNS/$name.log"
        ;;
    audit)
        name="$2"; shift 2
        cargo build -q --release -p deeptime-cli
        ./target/release/deeptime deposit-audit --world-data "$RUNS/$name-world" \
            --json "$RUNS/$name-audit.json" "$@" > "$RUNS/$name-audit.md"
        cat "$RUNS/$name-audit.md"
        ;;
    status)
        for l in "$RUNS"/*.log; do [ -e "$l" ] && { echo "== $l"; tail -3 "$l"; }; done
        ;;
    clean)
        rm -rf "$RUNS"
        ;;
    *) echo "usage: $0 world <name> [--set P=V]... | audit <name> [args…] | status | clean" >&2; exit 2 ;;
esac

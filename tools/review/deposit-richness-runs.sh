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
#       the audit again on an existing <name>-world/ with this tree's CLI (no new run), detached.
#   tools/remote/mac.sh -- tools/review/deposit-richness-runs.sh census <x0,z0,x1,z1> <name>...
#       real blocks of the box on the standalone set for each world (server lock; detached).
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
        cp target/release/deeptime "$RUNS/deeptime-audit-$name"
        args=""; for a in "$@"; do args="$args '$a'"; done
        setsid nohup sh -c "'$RUNS/deeptime-audit-$name' deposit-audit --world-data '$RUNS/$name-world' \
            --json '$RUNS/$name-audit.json' $args > '$RUNS/$name-audit.md' 2>&1; echo AUDIT-DONE >> '$RUNS/$name-audit.md'" \
            > /dev/null 2>&1 < /dev/null &
        echo "audit started pid $! -> $RUNS/$name-audit.md"
        ;;
    census)
        # Real blocks of one box on the standalone set, for each named world, one after the other
        # (deposit-audit-detached.sh takes the server lock for each); results box-<name>-standalone.json.
        box="$2"; shift 2
        cmd=""
        for name in "$@"; do
            cmd="$cmd BOX_TAG=$name SKIP_VANILLA=1 PROFILES=standalone tools/review/deposit-audit-detached.sh '$RUNS/$name-world' '$box';"
            cmd="$cmd cp review/out/deposit-audit/previews.log '$RUNS/previews-$name.log'; cp review/out/deposit-audit/box-$name-standalone.json '$RUNS/';"
        done
        setsid nohup sh -c "$cmd echo CENSUS-DONE > '$RUNS/census.done'" > /dev/null 2>&1 < /dev/null &
        echo "census started pid $!"
        ;;
    gametests)
        # ./gametest.sh, -PwithMic (the MIC set reinstalled first) and -PwithEv, detached, under the
        # server lock (mac.sh's protocol); logs gametest-{plain,mic,ev}.log, then gametests.done.
        # Run from mac.sh with --pack, without --server.
        LOCK="${DEEPTIME_CACHE:-/Volumes/Graphide/ban/deeptime-cache}/server.lock"
        rm -f "$RUNS/gametests.done"
        setsid nohup bash -c "
            while ! mkdir '$LOCK' 2>/dev/null; do
                age=\$(( \$(date +%s) - \$(stat -c %Y '$LOCK' 2>/dev/null || date +%s) ))
                [ \$age -gt 10800 ] && rm -rf '$LOCK' && continue
                sleep 30
            done
            basename '$REPO' > '$LOCK/owner'
            trap \"rm -rf '$LOCK'\" EXIT
            ./gametest.sh > '$RUNS/gametest-plain.log' 2>&1; a=\$?
            tools/smoke/install.sh mic > '$RUNS/install-mic.log' 2>&1
            ./gametest.sh -PwithMic > '$RUNS/gametest-mic.log' 2>&1; b=\$?
            ./gametest.sh -PwithEv > '$RUNS/gametest-ev.log' 2>&1; c=\$?
            echo \"plain=\$a mic=\$b ev=\$c\" > '$RUNS/gametests.done'
        " > /dev/null 2>&1 < /dev/null &
        echo "gametests started pid $!"
        ;;
    status)
        for l in "$RUNS"/*.log; do [ -e "$l" ] && { echo "== $l"; tail -3 "$l"; }; done
        ;;
    clean)
        rm -rf "$RUNS"
        ;;
    *) echo "usage: $0 world <name> [--set P=V]... | audit <name> [args…] | status | clean" >&2; exit 2 ;;
esac

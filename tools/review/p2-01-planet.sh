#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P2-01 (folds and faults): jobs for the test machine, run detached so a dropped laptop session
# does not kill them. Scratch and logs go to /Volumes/Graphide/ban/deeptime-runs/p2-01/.
#
#   tools/remote/mac.sh -- tools/review/p2-01-planet.sh start <name> <length-myr> [start-config] [preset]
#       builds the release CLI and runs `deeptime stage structure` (plates, surface, structure)
#       detached; the checkpoint is <name>.dtcp, the log <name>.log
#   tools/remote/mac.sh -- tools/review/p2-01-planet.sh world <name> <preset> [--set PATH=VALUE]...
#       builds the release CLI and runs the whole planet (`deeptime run`, 3 threads) detached into
#       <name>-world/, then `strata-check`, `structure-sections` (<name>-sections/) and the
#       parity fixture on it; the log is <name>.log
#   tools/remote/mac.sh -- tools/review/p2-01-planet.sh tests
#       runs `cargo test --workspace` detached (log: tests.log)
#   tools/remote/mac.sh --no-sync -- tools/review/p2-01-planet.sh status
#       the tail of every log, and what is still running
#   tools/remote/mac.sh --no-sync -- tools/review/p2-01-planet.sh summary <name>
#       the run's summary lines (structure_final=…) and its notes.structure.deformation
#   tools/remote/mac.sh --no-sync -- tools/review/p2-01-planet.sh stop <name>
#       stops that planet run
#   tools/remote/mac.sh --no-sync -- tools/review/p2-01-planet.sh clean
#       deletes the scratch folder (when the task has landed)
set -euo pipefail

RUNS=/Volumes/Graphide/ban/deeptime-runs/p2-01
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$RUNS"
cd "$REPO"

detach() { # <log> <command…>
    local log="$1"; shift
    setsid nohup "$@" > "$log" 2>&1 < /dev/null &
    echo "started pid $! → $log"
}

# The planet runs of this script, as "pid command". (The dev shell's pgrep does not start on the
# test machine: it loads the system libc under the Nix libsystemd.)
mine() {
    for d in /proc/[0-9]*; do
        c="$({ tr '\0' ' ' < "$d/cmdline"; } 2>/dev/null || true)"
        case "$c" in
            "$RUNS/deeptime-"*) echo "${d#/proc/} $c" ;;
        esac
    done
}

case "${1:-}" in
    start)
        name="$2"; length="$3"; start="${4:-scattered_continents}"; preset="${5:-earthlike_quick_16k}"
        detach "$RUNS/$name.log" sh -c "cargo build --release -p deeptime-cli \
            && cp target/release/deeptime '$RUNS/deeptime-$name' \
            && /usr/bin/time -v '$RUNS/deeptime-$name' stage structure --preset '$preset' \
               --start '$start' --length-myr '$length' --out '$RUNS/$name.dtcp'"
        ;;
    world)
        name="$2"; preset="$3"; shift 3
        sets=""; for a in "$@"; do sets="$sets '$a'"; done
        detach "$RUNS/$name.log" sh -c "cargo build --release -p deeptime-cli \
            && cp target/release/deeptime '$RUNS/deeptime-$name' \
            && rm -rf '$RUNS/$name-world' '$RUNS/$name-sections' \
            && /usr/bin/time -v '$RUNS/deeptime-$name' run --preset '$preset' --threads 3 \
               --out '$RUNS/$name-world' $sets \
            && '$RUNS/deeptime-$name' strata-check --world-data '$RUNS/$name-world' > '$RUNS/$name-strata.json' \
            && '$RUNS/deeptime-$name' structure-sections --world-data '$RUNS/$name-world' --out '$RUNS/$name-sections' \
            && '$RUNS/deeptime-$name' parity --world-data '$RUNS/$name-world' --columns 2000"
        ;;
    tests)
        detach "$RUNS/tests.log" cargo test --workspace --no-fail-fast
        ;;
    status)
        for f in "$RUNS"/*.log; do
            [ -e "$f" ] || continue
            echo "== $f"; tail -n "${2:-6}" "$f"
        done
        echo "== running"
        mine | grep . || echo "no planet run of this script (a detached test run is not listed)"
        ;;
    stop)
        # Stops one of this script's own planet runs, found by its binary's path (never by a
        # bare process name: the machine is shared).
        mine | grep -F "$RUNS/deeptime-$2 " | while read -r pid _; do
            kill -TERM "$pid" && echo "stopped $pid"
        done
        ;;
    summary)
        name="$2"
        grep -aE "structure_final=|structure deformation:|seconds=|Maximum resident|Elapsed" "$RUNS/$name.log" || true
        ;;
    failures)
        # What failed in a detached test log (default tests.log), with a little context.
        grep -aE "^test result|FAILED|panicked at|^---- |^error" -A"${3:-8}" "$RUNS/${2:-tests}.log" \
            | cut -c1-"${4:-500}" | head -n "${5:-150}" || echo "no failures in $RUNS/${2:-tests}.log"
        ;;
    clean)
        rm -rf "$RUNS"
        ;;
    *)
        sed -n '3,22p' "$0"; exit 2
        ;;
esac

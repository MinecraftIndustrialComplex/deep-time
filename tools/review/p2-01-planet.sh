#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P2-01 (folds and faults): jobs for the test machine, run detached so a dropped laptop session
# does not kill them. Scratch and logs go to /Volumes/Graphide/ban/deeptime-runs/p2-01/.
#
#   tools/remote/mac.sh -- tools/review/p2-01-planet.sh start <name> <length-myr> [start-config] [preset]
#       builds the release CLI and runs `deeptime stage structure` (plates, surface, structure)
#       detached; the checkpoint is <name>.dtcp, the log <name>.log
#   tools/remote/mac.sh -- tools/review/p2-01-planet.sh tests
#       runs `cargo test --workspace` detached (log: tests.log)
#   tools/remote/mac.sh --no-sync -- tools/review/p2-01-planet.sh status
#       the tail of every log, and what is still running
#   tools/remote/mac.sh --no-sync -- tools/review/p2-01-planet.sh summary <name>
#       the run's summary lines (structure_final=…) and its notes.structure.deformation
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

case "${1:-}" in
    start)
        name="$2"; length="$3"; start="${4:-scattered_continents}"; preset="${5:-earthlike_quick_16k}"
        detach "$RUNS/$name.log" sh -c "cargo build --release -p deeptime-cli \
            && cp target/release/deeptime '$RUNS/deeptime-$name' \
            && /usr/bin/time -v '$RUNS/deeptime-$name' stage structure --preset '$preset' \
               --start '$start' --length-myr '$length' --out '$RUNS/$name.dtcp'"
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
        /usr/bin/pgrep -af "$RUNS/deeptime-|cargo test --workspace" | grep -v pgrep || echo "nothing"
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
        sed -n '3,20p' "$0"; exit 2
        ;;
esac

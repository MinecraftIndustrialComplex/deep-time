#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The lakes' browser check on the test machine (task "climate-sized and frozen lakes", 2026-10-01):
# starts a PRIVATE `deeptime watch` on 127.0.0.1 and a free port (never the workbench on 8642: that
# is Ben's), queues one run, waits for it, and drives the page in headless Chromium
# (tools/workbench/lakes_shots.py): the Rivers & lakes legend, and the pointer's text over a lake, a
# frozen lake and a salt flat on that layer and on the satellite view. Run it through the wrapper,
# from the repo root:
#
#   tools/remote/mac.sh --pull review/out/lakes -- \
#       tools/workbench/lakes-check.sh --work /Volumes/Graphide/ban/deeptime-runs/lakes/wb
#
#   --work DIR       scratch folder (kept: delete it when done); the pictures go to review/out/lakes/
#   --length-myr N   length of the run (default 330: an ice age with frozen lakes on the built-in
#                    planet); --preset NAME (default earthlike_quick_16k); --start S (default random)
#   --bin PATH       the deeptime binary (default target/release/deeptime)
#
# Chromium: DEEPTIME_CHROME, else the one on PATH, else nixpkgs#chromium.

set -euo pipefail
cd "$(dirname "$0")/../.."
ARGS=("$@")
BIN=target/release/deeptime; PRESET=earthlike_quick_16k; LEN=330; START=random; WORK=""
while [ $# -gt 0 ]; do
    case "$1" in
        --bin) BIN="$2"; shift 2 ;;
        --preset) PRESET="$2"; shift 2 ;;
        --start) START="$2"; shift 2 ;;
        --length-myr) LEN="$2"; shift 2 ;;
        --work) WORK="$2"; shift 2 ;;
        *) echo "lakes-check: unknown argument $1" >&2; exit 2 ;;
    esac
done
[ -n "$WORK" ] || { echo "lakes-check: --work is required" >&2; exit 2; }
case "$WORK" in /Volumes/Graphide/ban/*) ;; *) echo "lakes-check: --work must be on the SSD (/Volumes/Graphide/ban/...)" >&2; exit 2 ;; esac
if [ -z "${DEEPTIME_CHROME:-}" ]; then
    command -v chromium > /dev/null 2>&1 || exec nix shell nixpkgs#chromium -c "$0" "${ARGS[@]}"
    DEEPTIME_CHROME="$(command -v chromium)"
    export DEEPTIME_CHROME
fi
mkdir -p "$WORK" review/out/lakes
RUNS="$WORK/runs"; rm -rf "$RUNS"
"$BIN" watch --preset "$PRESET" --start "$START" --length-myr "$LEN" --runs "$RUNS" --port 0 --bind 127.0.0.1 --keep 2 --threads 6 --mem-cap 8589934592 > "$WORK/watch.log" 2>&1 &
PID=$!
trap 'kill "$PID" 2>/dev/null || true' EXIT
PORT=""
for _ in $(seq 1 200); do
    PORT="$(sed -n 's#.*serving http://127.0.0.1:\([0-9]*\)/.*#\1#p' "$WORK/watch.log" | head -1)"
    [ -n "$PORT" ] && break
    kill -0 "$PID" 2>/dev/null || { echo "lakes-check: the server died:" >&2; tail -5 "$WORK/watch.log" >&2; exit 1; }
    sleep 0.3
done
[ -n "$PORT" ] || { echo "lakes-check: no port in the server's log" >&2; exit 1; }
echo "lakes-check: private workbench on 127.0.0.1:$PORT (pid $PID)"
URL="http://127.0.0.1:$PORT"
RUN=""
for _ in $(seq 1 100); do
    RUN="$(curl -s "$URL/api/runs" | python3 -c 'import json,sys; r=json.load(sys.stdin)["runs"]; print(r[0]["id"] if r else "")' || true)"
    [ -n "$RUN" ] && break
    sleep 0.3
done
[ -n "$RUN" ] || { echo "lakes-check: the run was not queued" >&2; exit 1; }
echo "lakes-check: run $RUN"
python3 tools/workbench/lakes_shots.py --url "$URL/" --run "$RUN" --out review/out/lakes --chromium "$DEEPTIME_CHROME" --size "${LAKES_SHOT_SIZE:-1600x1000}"

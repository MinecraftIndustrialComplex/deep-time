#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The deposits overlay's check on the Mac (task "Deposits layer", 2026-10-01): starts a PRIVATE
# `deeptime watch` on 127.0.0.1 and a free port (never the workbench on 8642: that is Ben's),
# queues one run, drives the page in headless Chromium (tools/viewer/deposits_shots.py)
# and stops its own server again. Run through the Mac wrapper, from the repo root:
#
#   tools/remote/mac.sh --pull review/out/wb-deposits -- \
#       tools/viewer/deposits-shots.sh --bin target/release/deeptime --preset earthlike_quick_16k \
#       --start scattered_continents --length-myr 500 --work /Volumes/Graphide/ban/deeptime-runs/wb-deposits
#
#   --bin      the deeptime binary to serve (default target/release/deeptime)
#   --preset   a built-in preset name, a preset file or a share code (what `deeptime watch --preset` takes)
#   --start / --length-myr   as for `deeptime watch`
#   --work     scratch folder for the run (kept: delete it when done); the screenshots go to
#              review/out/wb-deposits/ (not committed)
#   --reuse DIR  serve the finished runs a workbench kept in DIR instead of queueing one (the page
#              then opens on a finished run: the baked sites; the live path is not seen)
#   --keep-server   leave the server running when the checks end (prints its pid and port)
#
# Chromium is the playwright build in ~/Library/Caches/ms-playwright (override: DEEPTIME_CHROMIUM).

set -euo pipefail
cd "$(dirname "$0")/../.."

BIN=target/release/deeptime; PRESET=""; START=(); LEN=(); WORK=""; KEEP=0; REUSE=""
while [ $# -gt 0 ]; do
    case "$1" in
        --bin) BIN="$2"; shift 2 ;;
        --preset) PRESET="$2"; shift 2 ;;
        --start) START=(--start "$2"); shift 2 ;;
        --length-myr) LEN=(--length-myr "$2"); shift 2 ;;
        --work) WORK="$2"; shift 2 ;;
        --reuse) REUSE="$2"; shift 2 ;;
        --keep-server) KEEP=1; shift ;;
        *) echo "deposits-shots: unknown argument $1" >&2; exit 2 ;;
    esac
done
[ -n "$WORK" ] && { [ -n "$PRESET" ] || [ -n "$REUSE" ]; } || { echo "deposits-shots: --work and --preset (or --reuse RUNS_DIR) are required" >&2; exit 2; }
case "$WORK" in /Volumes/Graphide/ban/*) ;; *) echo "deposits-shots: --work must be on the SSD (/Volumes/Graphide/ban/...)" >&2; exit 2 ;; esac
CHROMIUM="${DEEPTIME_CHROMIUM:-$HOME/Library/Caches/ms-playwright/chromium-1243/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing}"
[ -x "$CHROMIUM" ] || { echo "deposits-shots: no Chromium at $CHROMIUM" >&2; exit 2; }

mkdir -p "$WORK" review/out/wb-deposits
if [ -n "$REUSE" ]; then
    # Serve the runs a finished workbench kept there (no new run): the newest is drawn.
    RUNS="$REUSE"; QUEUE=()
else
    RUNS="$WORK/runs"; rm -rf "$RUNS"
    QUEUE=(--preset "$PRESET" ${START[@]+"${START[@]}"} ${LEN[@]+"${LEN[@]}"})
fi
"$BIN" watch ${QUEUE[@]+"${QUEUE[@]}"} --runs "$RUNS" --port 0 --bind 127.0.0.1 --keep 2 --mem-cap 8589934592 > "$WORK/watch.log" 2>&1 &
PID=$!
cleanup() { if [ "$KEEP" = 0 ]; then kill "$PID" 2>/dev/null || true; fi; }
trap cleanup EXIT

PORT=""
for _ in $(seq 1 200); do
    PORT="$(sed -n 's#.*serving http://127.0.0.1:\([0-9]*\)/.*#\1#p' "$WORK/watch.log" | head -1)"
    [ -n "$PORT" ] && break
    kill -0 "$PID" 2>/dev/null || { echo "deposits-shots: the server died:" >&2; tail -5 "$WORK/watch.log" >&2; exit 1; }
    sleep 0.3
done
[ -n "$PORT" ] || { echo "deposits-shots: no port in the server's log" >&2; exit 1; }
echo "deposits-shots: private workbench on 127.0.0.1:$PORT (pid $PID)"

URL="http://127.0.0.1:$PORT"
RUN=""
for _ in $(seq 1 100); do
    RUN="$(curl -s "$URL/api/runs" | /usr/bin/python3 -c 'import json,sys; r=json.load(sys.stdin)["runs"]; print(r[0]["id"] if r else "")' || true)"
    [ -n "$RUN" ] && break
    sleep 0.3
done
[ -n "$RUN" ] || { echo "deposits-shots: the run was not queued" >&2; exit 1; }
echo "deposits-shots: run $RUN"

# The page opens now, while the run goes on: the driver waits for the sites to appear (the live
# path), then for the run to end and the legend to switch to the baked ones, then checks the rest.
status=0
/usr/bin/python3 tools/viewer/deposits_shots.py --url "$URL/" --run "$RUN" --out review/out/wb-deposits --chromium "$CHROMIUM" || status=$?
grep -E "timings|peak" "$WORK/watch.log" | cut -c1-400 || true
exit "$status"

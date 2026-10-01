#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# End-to-end check of the workbench's rock at depth and sections on a real baked planet, on the
# test machine (run it through tools/remote/mac.sh; never on the laptop):
#
#   tools/remote/mac.sh --pull review/out/wb-sections -- 'SCRATCH=/path/to/scratch/wb-sections tools/workbench/check-sections.sh'
#
# Makes (or reuses) a 500-Myr earthlike_quick_16k run in $SCRATCH/runs/bakeA/world, starts a private
# `deeptime watch` on a spare loopback port (never the live workbench's 8642), times the slice and
# section routes (cold, then cached), then drives the page in headless Chromium
# (tools/workbench/sections_shots.py) and leaves the pictures in review/out/wb-sections/. Needs
# python3 and a Chromium headless shell (DEEPTIME_CHROME). Exits non-zero on the first bad answer.

set -euo pipefail
cd "$(dirname "$0")/../.."
: "${SCRATCH:?set SCRATCH to a scratch folder on the test machine}"
# No Chromium given and none on PATH (the Linux test machine): run again inside `nix shell`.
if [ -z "${DEEPTIME_CHROME:-}" ] && ! ls ~/Library/Caches/ms-playwright/chromium_headless_shell-*/chrome-headless-shell-*/chrome-headless-shell > /dev/null 2>&1; then
    command -v chromium > /dev/null 2>&1 || exec nix shell nixpkgs#chromium -c "$0" "$@"
    DEEPTIME_CHROME="$(command -v chromium)"
    export DEEPTIME_CHROME
fi
mkdir -p "$SCRATCH"
PORT="${PORT:-8661}"
B="http://127.0.0.1:$PORT"
OUT="${OUT:-review/out/wb-sections}"
RUN="$SCRATCH/runs/bakeA"
mkdir -p "$RUN" "$OUT"

cargo build --release -q -p deeptime-cli
BIN="$PWD/target/release/deeptime"
if [ ! -f "$RUN/world/manifest.json" ]; then
    "$BIN" run --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --out "$RUN/world"
fi
python3 - "$RUN" <<'PY'
import json, sys
run = sys.argv[1]
preset = open(run + "/world/preset.json").read()
json.dump({"id": "bakeA", "created_utc": "2026-10-01T00:00:00Z", "label": "earthlike_quick_16k 500 Myr",
           "preset_json": preset, "share_code": "", "mem_cap": 0, "progress": {},
           "status": {"state": "done"}}, open(run + "/run.json", "w"))
PY
rm -rf "$RUN/rock"

"$BIN" watch --bind 127.0.0.1 --port "$PORT" --runs "$SCRATCH/runs" > "$SCRATCH/watch-$PORT.log" 2>&1 &
PID=$!
trap 'kill $PID 2>/dev/null || true' EXIT
for _ in $(seq 60); do curl -s -o /dev/null "$B/api/runs" && break; sleep 0.5; done

curl -s "$B/api/runs/bakeA/rock" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d["available"], d; print("rock info ok:", len(d["classes"]), "classes, depths", d["depths"])'
for d in 0 20 50 100 200 floor; do
    curl -s -o /dev/null -w "slice depth $d: %{http_code} %{size_download} bytes %{time_total}s (cold)\n" "$B/api/runs/bakeA/rock/slice.png?depth=$d"
done
curl -s -o /dev/null -w "slice depth 50 again: %{http_code} %{time_total}s (cached)\n" "$B/api/runs/bakeA/rock/slice.png?depth=50"
for q in 'x0=-2000&z0=-500&x1=3000&z1=800' 'x0=1237&z0=3100&x1=1237&z1=3650' 'x0=8000&z0=-300&x1=-7900&z1=200'; do
    curl -s -o /dev/null -w "section $q: %{http_code} %{size_download} bytes %{time_total}s\n" "$B/api/runs/bakeA/rock/section?$q"
done

python3 tools/workbench/sections_shots.py "$B" bakeA "$OUT" --depths 20,100,floor \
    --section dia=1237,3100,1237,3650 --section coal=3422,700,3422,1250 --section seam=8000,-300,-7900,200 --other-views
echo "watch log:"; cat "$SCRATCH/watch-$PORT.log"

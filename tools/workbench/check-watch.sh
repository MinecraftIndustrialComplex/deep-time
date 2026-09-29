#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# End-to-end check of the workbench's stage 2 through `deeptime watch --preview` (P1-20), on the
# Mac: starts a private workbench on a spare loopback port (never the live one on 8642), asks it
# for a 3D preview the way the page does (POST /api/preview with {run, frame, t_years, box,
# lat_lon}), follows status_url until the export is ready, then fetches the export, the viewer and
# a click-to-inspect of a column inside the box. The preview boots a Minecraft server, so the
# runner takes the Mac's server lock itself; run this without mac.sh --server:
#
#   tools/remote/mac.sh --pull smoke/workbench/jobs -- 'tools/workbench/check-watch.sh'
#
# Prints each answer; exits non-zero on the first thing that is not as expected.

set -euo pipefail
cd "$(dirname "$0")/../.."
PORT="${PORT:-8653}"
B="http://127.0.0.1:$PORT"
cargo build --release -q -p deeptime-cli
RUNS="$(mktemp -d)"
target/release/deeptime watch --bind 127.0.0.1 --port "$PORT" --runs "$RUNS" \
    --preview "$PWD/tools/workbench/preview.py" --python /usr/bin/python3 > "$RUNS/watch.log" 2>&1 &
PID=$!
trap 'kill $PID 2>/dev/null || true; rm -rf "$RUNS"' EXIT
for _ in $(seq 60); do curl -s -o /dev/null "$B/" && break; sleep 0.5; done
field() { python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('$1') or '')"; }

# The supergene blanket of the synthetic deposit world sits near x -5,563, z -1,002: lat 22.2..21.6,
# lon 57.1..58.5 on C = 16,384.
body='{"run":"check","frame":3,"t_years":-100000000,"box":{"x0":0,"z0":0,"x1":1,"z1":1},"lat_lon":{"north":22.2,"south":21.6,"west":57.1,"east":58.5}}'
t0=$(date +%s)
r="$(curl -s -H 'Content-Type: application/json' -d "$body" "$B/api/preview")"
echo "POST /api/preview → ${r:0:400}"
job="$(echo "$r" | field job)"
[ -n "$job" ] || { echo "no job in the answer"; exit 1; }
[ -n "$(echo "$r" | field note)" ] || { echo "no world note in the answer"; exit 1; }
status_url="$(echo "$r" | field status_url)"
url=""
while [ -z "$url" ]; do
    sleep 5
    s="$(curl -s "$B/$status_url")"
    state="$(echo "$s" | field state)"
    url="$(echo "$s" | field url)"
    err="$(echo "$s" | field error)"
    echo "  $(( $(date +%s) - t0 )) s: $state $(echo "$s" | field message | cut -c1-120)"
    [ -z "$err" ] || { echo "failed: $err"; exit 1; }
    [ $(( $(date +%s) - t0 )) -lt 1200 ] || { echo "timed out"; exit 1; }
done
echo "done after $(( $(date +%s) - t0 )) s; url = $url"
src="${url#area3d/?src=../}"; src="${src%%&*}"
bytes="$(curl -s "$B/$src" | wc -c | tr -d ' ')"
echo "GET /$src → $bytes bytes"
[ "$bytes" -gt 10000 ] || exit 1
viewer="$(curl -s -o /dev/null -w '%{http_code} %{content_type}' "$B/area3d/")"
echo "GET /area3d/ → $viewer"
js="$(curl -s -o /dev/null -w '%{http_code}' "$B/area3d/area3d.js")"
echo "GET /area3d/area3d.js → $js"
case "$viewer" in 200*text/html*) ;; *) exit 1 ;; esac
i="$(curl -s "$B/api/inspect?run=check&frame=3&x=0&z=0&lat=21.9&lon=57.6")"
echo "GET /api/inspect (a column inside the box) → url $(echo "$i" | field url)"
echo "$i" | field url | grep -q pickcol || { echo "inspect did not reuse the preview"; exit 1; }
echo "job $job: OK"

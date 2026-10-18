#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# One scale of the Phase 1 exit's scale comparison (plan §10.3 item 7), on the test VM through
# tools/remote/mac.sh --server: put the baked planet of that scale into the standalone server set, let Chunky
# generate the region around the chosen centre, render it with BlueMap (tools/review/render-bluemap.sh) and take
# the pictures (tools/review/phase1_shots.py). tools/review/phase1_scale.py drives the three runs one after the
# other: they share smoke/standalone, so they never overlap.
#
#   tools/review/phase1-scale-one.sh <m per block> <centre x> <centre z> [half-size blocks, default 1024]
#
# WORK (default /Volumes/Graphide/ban/deeptime-runs/phase1/scale) holds what the earlier steps wrote:
# preset-h<m>.json, h<m>/planet/deeptime (the bake). Writes $WORK/bm-h<m>/ (the BlueMap app) and
# review/out/phase1/scale/h<m>-*.png (the pictures; not committed).
set -euo pipefail
cd "$(dirname "$0")/../.."
H="${1:?m per block}"; CX="${2:?centre x}"; CZ="${3:?centre z}"; R="${4:-1024}"
WORK="${WORK:-/Volumes/Graphide/ban/deeptime-runs/phase1/scale}"
BAKE="$WORK/h$H/planet/deeptime"
[ -f "$BAKE/manifest.json" ] || { echo "phase1-scale-one: no bake at $BAKE" >&2; exit 1; }
SERVER=smoke/standalone/server

tools/smoke/install.sh standalone --size 32768 --preset "$WORK/preset-h$H.json" --start scattered_continents --length-myr 500 >/dev/null
rm -rf "$SERVER/planet"
mkdir -p "$SERVER/planet"
cp -r "$BAKE" "$SERVER/planet/deeptime"
# No --fresh: the world is the bake alone (its manifest is there, so the server does not generate).
python3 tools/smoke/run.py --profile standalone --timeout 3000 --chunky-at "$CX,$CZ,$R" > "$WORK/run-h$H.log" 2>&1 || true
grep -E '\[(PASS|FAIL)\]|^(PASS|FAIL)|world data present|chunky' "$WORK/run-h$H.log" | cut -c1-200
grep -q '^PASS' "$WORK/run-h$H.log" || { echo "phase1-scale-one: the server run failed; the log's tail:"; tail -n 60 "$WORK/run-h$H.log" | cut -c1-300; exit 1; }

X0=$((CX - R)); X1=$((CX + R - 1)); Z0=$((CZ - R)); Z1=$((CZ + R - 1))
BLUEMAP_BOX="$X0 $X1 $Z0 $Z1" BLUEMAP_MAP_LABEL="Deep Time 1:$H" \
    tools/review/render-bluemap.sh "$SERVER/planet" "$SERVER/mods" "$WORK/bm-h$H" 6 2>&1 \
    | grep -E -i 'render-bluemap|error|exception|finished' | grep -v -E 'Missing.*(sound|lang)' | tail -20

# Pictures: a tilted view from the south over the region's middle, and the top-down one. Same camera at every
# scale (the region is the same number of blocks), vertical scale unchanged. The ground sits around y 60..300.
mkdir -p review/out/phase1/scale
python3 tools/review/phase1_shots.py --web "$WORK/bm-h$H/web" --map planet --out "review/out/phase1/scale/h$H" \
    --view "tilt:$CX:120:$CZ:2600:1.0" --view "flat:$CX:120:$CZ:$R:0:flat" --wait "${SHOT_WAIT:-90}"
ls -la review/out/phase1/scale | grep "h$H" || true

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Assemble the world-shape page in review/out/world-shape-page/ (gitignored) from the page sources here,
# the pictures and data.js that run.sh made on the test VM, and a local copy of three.js r128 (MIT; the
# page loads nothing from the network). Then publish:
#   tools/review/publish.sh world-shape review/out/world-shape-page --description "..."
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
HOST="${DEEPTIME_MAC:-ban@20.120.160.3}"
RUNS="${RUNS:-/data/ban/deeptime-runs/world-shape}"
src="$repo/review/out/ws-src"
out="$repo/review/out/world-shape-page"
mkdir -p "$src" "$out"
[ "${NO_PULL:-0}" = 1 ] || rsync -az "$HOST:$RUNS/out/" "$src/"
[ -f "$src/three.min.js" ] || curl -fsSL -o "$src/three.min.js" https://cdnjs.cloudflare.com/ajax/libs/three.js/r128/three.min.js
find "$out" -mindepth 1 -delete
cp "$src"/*.webp "$src/data.js" "$src/three.min.js" "$out/"
cp "$here/page/index.html" "$here/page/app.js" "$here/page/globe.js" "$out/"
du -sh "$out"

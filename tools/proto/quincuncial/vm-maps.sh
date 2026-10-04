#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# PROTOTYPE (docs/design/seamless-wrapping.md). On the test VM, from the repo copy:
#   tools/remote/mac.sh -- 'bash tools/proto/quincuncial/vm-maps.sh'
# Renders the source planet (Lambert/equirect strip) and the resampled Guyou strip as PNGs into
# $RUNS (pull them back with scp). Expects $RUNS/deeptime, $RUNS/src-world and $RUNS/guyou-world.
set -euo pipefail
REPO="$(cd "$(dirname "$0")/../../.." && pwd)"
RUNS="${RUNS:-/data/ban/deeptime-runs/quincuncial}"
B="${B:-8192}"
cd "$RUNS"
PY() { nix shell nixpkgs#python3Packages.numpy -c python3 "$REPO/tools/proto/quincuncial/render.py" "$@"; }
./deeptime heightmap --world-data guyou-world --box="-$((2*B)),-$B,$((2*B)),$B" --step 32 --out guyou.npy
./deeptime heightmap --world-data src-world --box="-$B,-$((B/2)),$B,$((B/2))" --step 16 --out src.npy
H=$((B/2))
PY guyou.npy guyou-strip.png --box="-$((2*B)),-$B,$((2*B)),$B" --step 32 --domain="-$B,$B,-$H,$H" \
  --mark="0,-$H" --mark="0,$H" --mark="-$B,-$H" --mark="-$B,$H" --mark="$B,-$H" --mark="$B,$H" \
  --mark="-$H,-$H" --mark="$H,$H"
# The domain alone, at 16 blocks a pixel.
./deeptime heightmap --world-data guyou-world --box="-$B,-$H,$B,$H" --step 16 --out guyou-domain.npy
PY guyou-domain.npy guyou-domain.png --box="-$B,-$H,$B,$H" --step 16 \
  --mark="0,-$H" --mark="0,$H" --mark="-$B,-$H" --mark="-$B,$H" --mark="$B,-$H" --mark="$B,$H" \
  --mark="-$H,-$H" --mark="$H,$H"
PY src.npy src-equirect.png --box="-$B,-$H,$B,$H" --step 16
ls -la "$RUNS"/*.png

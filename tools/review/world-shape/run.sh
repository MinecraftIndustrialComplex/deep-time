#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Build the world-shape page's pictures and data on the test VM (never on the laptop):
#   tools/remote/mac.sh -- tools/remote/detach.sh start worldshape --kind shell:python \
#       bash tools/review/world-shape/run.sh
#   tools/remote/mac.sh --no-sync -- tools/remote/detach.sh wait worldshape
# then pull $RUNS/out into the page folder (tools/review/world-shape/pull.sh) and publish with
#   tools/review/publish.sh world-shape review/out/world-shape-page
# Inputs: a finished 16k Lambert bake (WORLD) and a deeptime binary that can read it (BIN).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
RUNS="${RUNS:-/data/ban/deeptime-runs/world-shape}"
WORLD="${WORLD:-/data/ban/deeptime-cache/workbench/runs/1a103bb2d2b/world}"
BIN="${BIN:-/data/ban/mic/mods/deep-time/target/release/deeptime}"
mkdir -p "$RUNS/out"
if [ ! -f "$RUNS/planet.npy" ]; then
    "$BIN" heightmap --world-data "$WORLD" --box=-8192,-2624,8192,2624 --step 4 \
        --layers climate/ice_cover,climate/p_mm --out "$RUNS/planet.npy"
fi
PY='(builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux.python3.withPackages (p: [p.numpy p.scipy p.mpmath p.pillow])'
nix shell --impure --expr "$PY" -c python3 "$here/compute.py" "$RUNS/planet.npy" "$RUNS/out" "$RUNS"
ls -la "$RUNS/out" | head -80
du -sh "$RUNS/out"

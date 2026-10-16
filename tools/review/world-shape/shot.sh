#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# On the test VM: screenshot the assembled page (rsynced to $RUNS/page) and cut it into pieces.
#   shot.sh [width] [height] [path]
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
RUNS="${RUNS:-/data/ban/deeptime-runs/world-shape-h2h}"
W="${1:-1280}"; H="${2:-11000}"; P="${3:-index.html}"
rm -f "$RUNS"/shot*.png
python3 "$here/../shot-page.py" "$RUNS/page" "$RUNS/shot.png" --width "$W" --height "$H" --budget 25000 --path "$P"
PY='(builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux.python3.withPackages (p: [p.pillow])'
nix shell --impure --expr "$PY" -c python3 "$here/crop.py" "$RUNS/shot.png" "$RUNS/shot" 1600

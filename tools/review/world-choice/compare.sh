#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Cube phase 2, Ben's world-shape decision: the two worlds worlds.sh kept (earthlike_quick_16k,
# seed 1, supercontinent, 1,000 Myr; pg16 on Petroff-Guyou, cube16 as the cube) drawn whole
# (world-map) and sampled (heightmap with lat/lon), then compare.py's distortion numbers, places and
# vista strings. Each heavy step admits itself.
#   tools/remote/mac.sh -- tools/remote/detach.sh start cube2-cmp bash tools/review/world-choice/compare.sh [plan]
# "plan" reruns only compare.py on the maps and samples already there.
set -euo pipefail
cd "$(dirname "$0")/../../.."
CMP=${CMP:-/Volumes/Graphide/ban/deeptime-runs/cube2-cmp}
BIN="$PWD/target/release/deeptime"
PY='(builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux.python3.withPackages (p: [p.numpy p.scipy])'
py() { nix shell --impure --expr "$PY" -c python3 -I tools/review/world-choice/compare.py "$@"; }
[ "${1:-}" = plan ] || deeptime-admit run --kind cargo:build -- nix develop -c cargo build --release -q -p deeptime-cli
for w in pg16 cube16; do
  [ "${1:-}" = plan ] && break
  [ -f "$CMP/$w/deeptime/manifest.json" ] || { echo "no $w world in $CMP"; exit 1; }
  deeptime-admit run --kind auto -- "$BIN" world-map --world-data "$CMP/$w/deeptime" --zoom 2 --out "$CMP/$w-map"
  B=$(py box "$CMP" "$w")
  echo "$w box $B"
  deeptime-admit run --kind auto -- "$BIN" heightmap --world-data "$CMP/$w/deeptime" "--box=$B" --step 16 --scale \
    --out "$CMP/$w-hm.npy"
done
deeptime-admit run --kind auto -- nix shell --impure --expr "$PY" -c python3 -I tools/review/world-choice/compare.py plan "$CMP"
echo "compare done"

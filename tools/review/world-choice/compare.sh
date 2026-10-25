#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Cube phase 2, Ben's world-shape decision: the two worlds worlds.sh kept (earthlike_quick_16k,
# seed 1, supercontinent, 1,000 Myr; pg16 on Petroff-Guyou, cube16 as the cube) drawn whole
# (world-map) and sampled (heightmap with lat/lon), then compare.py's distortion numbers, places and
# vista strings. Each heavy step admits itself.
#   tools/remote/mac.sh -- tools/remote/detach.sh start cube2-cmp bash tools/review/world-choice/compare.sh [plan]
# "plan" reruns only compare.py on the maps and samples already there.
# RETIRED in part (cube plan 7.4): the drawing and sampling step needs the kept pg16 world, which can
# no longer be baked (the Petroff-Guyou world is deleted), so only `plan` still runs, on the maps and
# samples already in $CMP. The finished page is the record of the comparison.
set -euo pipefail
cd "$(dirname "$0")/../../.."
CMP=${CMP:-/Volumes/Graphide/ban/deeptime-runs/cube2-cmp}
PY='(builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux.python3.withPackages (p: [p.numpy p.scipy])'
if [ "${1:-}" != plan ]; then
  echo "world-choice/compare.sh: only 'plan' still runs: drawing and sampling needs the pg16 world, and the Petroff-Guyou world is deleted (cube plan 7.4)." >&2
  exit 2
fi
deeptime-admit run --kind auto -- nix shell --impure --expr "$PY" -c python3 -I tools/review/world-choice/compare.py plan "$CMP"
echo "compare done"

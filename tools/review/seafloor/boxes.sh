#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Sea-floor spikes study (2026-10-20): close-ups of 384-block boxes of a baked world, at one block
# a sample, rendered by closeup.py (hillshade, the game-scale oblique view, transects, spectrum).
#   boxes.sh <deeptime binary> <world-data dir> <out dir> <name>=<x0>,<z0> ...
set -euo pipefail
B=$1 world=$2 out=$3
shift 3
R=$(cd "$(dirname "$0")/../../.." && pwd)
PY='(builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux.python3.withPackages (p: [p.numpy p.scipy p.matplotlib])'
mkdir -p "$out"
for spec in "$@"; do
    name=${spec%%=*} xz=${spec#*=}
    x0=${xz%,*} z0=${xz#*,}
    "$B" heightmap --world-data "$world" --box="$x0,$z0,$((x0 + 384)),$((z0 + 384))" --step 1 \
        --layers meta/crust_age_myr,meta/province --out "$out/$name.npy" 2>&1 | tail -1
    nix shell --impure --expr "$PY" -c python3 "$R/tools/review/seafloor/closeup.py" "$out/$name.npy" "$out/$name.png" \
        --title "$name box $x0,$z0 ($(basename "$(dirname "$out")")/$(basename "$out"))"
done

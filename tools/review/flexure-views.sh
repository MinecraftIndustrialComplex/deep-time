#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The flexure review pictures of a run's checkpoint (tools/review/flexure_views.py; the run is
# `deeptime stage volcanoes --preset … --out <checkpoint>`). On the test VM via tools/remote/mac.sh:
#
#   tools/remote/mac.sh --pull review/out/flexure -- \
#       'BIN=/path/to/deeptime tools/review/flexure-views.sh <checkpoint> <name> "<label>"'
#
# Writes review/out/flexure/<name>/{planet,moat,belt}.png and numbers.json. The sea level is read
# from the checkpoint's stage log beside it (<checkpoint without .dtcp>.log) when there is one,
# else SEA (default 0). BIN defaults to the release build of this checkout.
set -euo pipefail
CKPT=$1; NAME=$2; LABEL=${3:-}
BIN=${BIN:-target/release/deeptime}
LOG=${CKPT%.dtcp}.log
SEA=${SEA:-0}
if [ -f "$LOG" ]; then
    SEA=$(grep -o 'sea_level_m=[-0-9.]*' "$LOG" | tail -1 | cut -d= -f2)
fi
FIELDS=${CKPT%.dtcp}.fields
"$BIN" dump-fields --in "$CKPT" --out "$FIELDS" --fields \
    surface/elevation_m,surface/flexure_m,surface/deflection_m,surface/elastic_thickness_km,plates/crust_type,regime,plates/flags,plates/plate_id
nix shell --impure --expr \
    '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.matplotlib])' \
    -c python3 "$(dirname "$0")/flexure_views.py" --fields "$FIELDS" --sea "$SEA" \
    --out "review/out/flexure/$NAME" --label "$LABEL"
rm -rf "$FIELDS"

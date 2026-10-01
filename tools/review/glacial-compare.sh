#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The glacial landforms' review figures from a run's deposits checkpoint (run on the Mac through
# tools/remote/mac.sh): the young landscape and the amplifier with the glaciers on and off, then
# tools/review/glacial_views.py on the L3 terrains.
#
#   tools/review/glacial-compare.sh <deposits.dtcp> <work dir> <review dir> [--sea M] [-- extra --set …]
#
# Extra arguments after `--` go to every stage (e.g. --set amplify.glacial.ridge_share=0.3).
set -euo pipefail
CK="$1"; WORK="$2"; OUT="$3"; shift 3
SEA=0
if [ "${1:-}" = "--sea" ]; then SEA="$2"; shift 2; fi
[ "${1:-}" = "--" ] && shift
EXTRA=("$@")
B=target/release/deeptime
mkdir -p "$WORK" "$OUT"
if [ -z "${SKIP_YOUNG:-}" ] || [ ! -f "$WORK/y_on.dtcp" ]; then
    $B stage young --in "$CK" --out "$WORK/y_on.dtcp" "${EXTRA[@]}" | grep young_final
    $B stage young --in "$CK" --out "$WORK/y_off.dtcp" --set surface.glacial.strength=0 "${EXTRA[@]}" | grep young_final
    $B dump-fields --in "$WORK/y_on.dtcp" --out "$WORK/fields_on" \
        --fields young/ice_thickness_m,young/ice_surface_m,young/elevation_m,young/glacial_erosion_m,young/till_m
fi
$B stage amplify --in "$WORK/y_on.dtcp" --out "$WORK/a_on.dtcp" "${EXTRA[@]}" | grep amplify_final
$B stage amplify --in "$WORK/y_on.dtcp" --out "$WORK/a_noshape.dtcp" --set amplify.glacial.strength=0 "${EXTRA[@]}" | grep amplify_final
$B stage amplify --in "$WORK/y_off.dtcp" --out "$WORK/a_off.dtcp" --set amplify.glacial.strength=0 "${EXTRA[@]}" | grep amplify_final
PY=(nix shell --impure --expr
    '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.matplotlib p.lz4])'
    -c python3)
"${PY[@]}" tools/review/glacial_views.py --fields "$WORK/fields_on" \
    --l3 "no glaciers=$WORK/a_off.dtcp.work" \
    --l3 "glacial L2, no L3 shaping=$WORK/a_noshape.dtcp.work" \
    --l3 "glaciers=$WORK/a_on.dtcp.work" \
    --out "$OUT" --sea "$SEA"

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-15 / Phase 1 exit item 3 on a baked, pregenerated planet (run on the test VM through
# tools/remote/mac.sh): `deeptime-inspect` top-down, ore and section renders of the porphyries and
# the placer near the spawn, from the world a smoke run left in smoke/<profile>/server/planet.
#
#   tools/review/p1-15-geology.sh <standalone|mic> [out dir]
#
# The boxes and sections are those of the 2026-10-04 seed's world (sites 19, 20, 6 and the placer 4,
# see tools/review/p1-15-probes-4.txt); a different bake needs different numbers. Output:
# review/out/p1-15-real/<profile>/inspect/.
set -euo pipefail
cd "$(dirname "$0")/../.."
profile="${1:?profile}"
OUT="${2:-review/out/p1-15-real/$profile/inspect}"
W="smoke/$profile/server/planet"
mkdir -p "$OUT"
cargo build -q --release -p deeptime-inspect
INSPECT=target/release/deeptime-inspect
$INSPECT topdown --world "$W" --area=1850,-1950,2650,-1150 --scale 1 --out "$OUT/topdown" > "$OUT/topdown.json"
while read -r name x0 x1 z0 z1; do
    [ -n "$name" ] || continue
    $INSPECT ore --world "$W" --area="$x0,$z0,$((x1 + 1)),$((z1 + 1))" --scale 4 --out "$OUT/ore-$name" > "$OUT/ore-$name.json"
done <<'BOXES'
site19-20 1900 2200 -1470 -1270
site6-placer4 2150 2350 -1700 -1500
BOXES
while read -r name axis at from to y0 y1 scale; do
    [ -n "$name" ] || continue
    $INSPECT section --world "$W" --axis "$axis" --at="$at" --from="$from" --to="$to" --y-from="$y0" --y-to="$y1" \
        --scale "$scale" --out "$OUT/section-$name" > "$OUT/section-$name.txt"
done <<'SECTIONS'
site19 z -1366 1950 2060 -30 70 8
site20 z -1381 2080 2140 -45 90 8
site6 z -1553 2215 2295 -510 110 4
placer4 x 2215 -1705 -1595 55 115 8
SECTIONS
find "$OUT" -name '*.png' | sort

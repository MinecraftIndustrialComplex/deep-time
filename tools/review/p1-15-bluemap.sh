#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-15 / Phase 1 exit item 3: BlueMap of the arc range and the deposits near the 2026-10-04 spawn
# (boxes: the ranges' flank with the placers, sites 19 and 20 cut at y 25, site 6 cut at y -440; all
# inside the pregenerated square 1850..2650 x -1950..-1150). On the VM, after a smoke run with
# --chunky-at 2250,-1550,400:   tools/review/p1-15-bluemap.sh <standalone|mic>
set -euo pipefail
cd "$(dirname "$0")/../.."
p="${1:?profile}"
BLUEMAP_ONLY_PRESENT=1 BLUEMAP_BOXES="1850 2149 -1950 -1500 0; 1900 2200 -1470 -1270 25; 2150 2350 -1700 -1500 -440" \
    tools/review/render-bluemap.sh "smoke/$p/server/planet" "smoke/$p/server/mods" "review/out/p1-15-real/$p/bluemap" 4

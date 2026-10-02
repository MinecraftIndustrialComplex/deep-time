#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-10 acceptance renders (run on the test VM through tools/remote/mac.sh --server): a first start of a
# real planet (earthlike_quick_16k, scattered continents, 500 Myr by default) in the terrablender set
# (TerraBlender + Biomes O' Plenty), then `/deeptime probe regions 16`: every province's region and the
# biome the game places at the surface, over the whole planet. The grid lands in
# review/out/p1-10-data/ and tools/review/region_map.py turns it into the review page.
#
#   tools/review/p1-10-smoke.sh [length Myr]
set -euo pipefail
cd "$(dirname "$0")/../.."
LEN=${1:-500}
out=review/out/p1-10-data
rm -rf "$out"
mkdir -p "$out"
tools/smoke/install.sh terrablender --preset earthlike_quick_16k --start scattered_continents --length-myr "$LEN"
python3 tools/smoke/run.py --profile terrablender --fresh --rcon "deeptime probe regions 16" --timeout 3000
cp smoke/terrablender/server/deeptime-probe/regionmap.json "$out/"
cp smoke/terrablender/report.md "$out/" 2>/dev/null || true
cp smoke/terrablender/server/planet/deeptime/climate_mapping.json "$out/" 2>/dev/null || true
cp smoke/terrablender/server/planet/deeptime/manifest.json "$out/" 2>/dev/null || true
python3 tools/review/region_map.py --map "$out/regionmap.json" --out review/out/p1-10
ls -la "$out" review/out/p1-10

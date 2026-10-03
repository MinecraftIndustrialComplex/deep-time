#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Cold deserts: the biome census of two finished quick planets (the same world baked with the old and
# the new placement table: `deeptime run --content <bundle with climate_placement>`, or two runs of
# cold-schist-runs.sh) through the standalone, MIC and Biomes O' Plenty (terrablender) biome sources,
# under one hold of the server lock.
#
#   tools/remote/mac.sh --pack --server -- tools/review/cold-census.sh [<base> <after>]
set -uo pipefail
cd "$(cd "$(dirname "$0")/../.." && pwd)"
base="${1:-c-oldmap}"; after="${2:-c-after}"
for p in standalone mic terrablender; do
    for r in "$base" "$after"; do
        echo "#### census $p $r"
        tools/review/cold-desert-biomes.sh "$p" "$r" 2>&1 | tail -170
    done
done

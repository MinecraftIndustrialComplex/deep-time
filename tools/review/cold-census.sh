#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Cold deserts: the biome census of two finished quick planets (cold-schist-runs.sh run <base> and
# <after>) through the standalone and MIC biome sources, and of the after planet with Biomes O'
# Plenty (terrablender), under one hold of the server lock.
#
#   tools/remote/mac.sh --pack --server -- tools/review/cold-census.sh [<base> <after>]
set -uo pipefail
cd "$(cd "$(dirname "$0")/../.." && pwd)"
base="${1:-c-base}"; after="${2:-c-after}"
for p in standalone mic; do
    for r in "$base" "$after"; do
        echo "#### census $p $r"
        tools/review/cold-desert-biomes.sh "$p" "$r" 2>&1 | tail -170
    done
done
echo "#### census terrablender $after"
tools/review/cold-desert-biomes.sh terrablender "$after" 2>&1 | tail -170

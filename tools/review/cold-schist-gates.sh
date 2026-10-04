#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Cold deserts and schist: everything of the task that boots Minecraft, under one hold of the test
# VM's server lock (it is shared by about ten agents, so one wait instead of several):
#   census   the biome census of the quick planets of cold-schist-runs.sh (base and after) in the
#            standalone and MIC profiles and of the after planet with Biomes O' Plenty (terrablender);
#   gametest ./gametest.sh, -PwithMic and -PwithTerraBlender, their logs kept beside the census.
#
#   tools/remote/mac.sh --pack --server -- tools/review/cold-schist-gates.sh census|gametest
set -uo pipefail

RUNS=/Volumes/Graphide/ban/deeptime-runs/cold-desert
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"

case "${1:-}" in
    census)
        for p in standalone mic; do
            for r in base after; do
                echo "#### census $p $r"
                tools/review/cold-desert-biomes.sh "$p" "$r" 2>&1 | tail -170
            done
        done
        echo "#### census terrablender after"
        tools/review/cold-desert-biomes.sh terrablender after 2>&1 | tail -170
        ;;
    gametest)
        tools/smoke/install.sh mic > /dev/null 2>&1 || echo "mic install failed"
        tools/smoke/install.sh terrablender > /dev/null 2>&1 || echo "terrablender install failed"
        for flag in "" -PwithMic -PwithTerraBlender; do
            name="gametest${flag:-standalone}"
            echo "#### $name"
            ./gametest.sh $flag > "$RUNS/$name.out" 2>&1
            cp neoforge/build/gametest.log "$RUNS/$name.log" 2>/dev/null || true
            grep -E "required tests|tests failed|climateConsistent" "$RUNS/$name.log" | cut -c1-6000 | tail -40
        done
        ;;
    *)
        sed -n '3,10p' "$0"; exit 2
        ;;
esac

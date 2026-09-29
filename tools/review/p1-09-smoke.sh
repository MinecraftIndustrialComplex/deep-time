#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-09 acceptance renders (run on the Mac through tools/remote/mac.sh --pack --server): for each
# profile, a first start of a real planet (earthlike_quick_16k, scattered continents, 500 Myr),
# Chunky over one area per Köppen family Ben asked about, `/deeptime probe biomes 16` (the whole
# planet's surface biomes against its Köppen map), and `deeptime-inspect biomes` of the Chunky
# areas. Everything lands in review/out/p1-09-data/<profile>/; tools/review/biome_map.py turns it
# into the review page.
#
#   tools/review/p1-09-smoke.sh [standalone] [mic]
#
# The areas were picked on the Köppen map of the same preset's P1-08 bake (a 25 × 25-cell window
# mostly of the class): Af -3968,32; BW -4544,-1120; Dfc 7504,-2416; ET 2800,-3184; EF 1744,2912.
set -euo pipefail
cd "$(dirname "$0")/../.."
AREAS=(-3968,32,192 -4544,-1120,192 7504,-2416,192 2800,-3184,192 1744,2912,160)
NAMES=(af bw dfc et ef)
PROFILES=("$@")
[ ${#PROFILES[@]} -gt 0 ] || PROFILES=(standalone mic)
cargo build --release -p deeptime-inspect
for p in "${PROFILES[@]}"; do
    out="review/out/p1-09-data/$p"
    rm -rf "$out"
    mkdir -p "$out/inspect"
    tools/smoke/install.sh "$p" --preset earthlike_quick_16k --start scattered_continents --length-myr 500
    args=()
    for a in "${AREAS[@]}"; do args+=(--chunky-at "$a"); done
    python3 tools/smoke/run.py --profile "$p" --fresh "${args[@]}" --rcon "deeptime probe biomes 16" --timeout 3000
    cp "smoke/$p/server/deeptime-probe/biomemap.json" "$out/"
    cp "smoke/$p/report.md" "$out/" 2>/dev/null || true
    cp "smoke/$p/server/planet/deeptime/climate_mapping.json" "$out/" 2>/dev/null || true
    for i in "${!AREAS[@]}"; do
        IFS=, read -r x z r <<<"${AREAS[$i]}"
        target/release/deeptime-inspect biomes --world "smoke/$p/server/planet" --out "$out/inspect" \
            --area "$((x - r)),$((z - r)),$((x + r)),$((z + r))" --name "${NAMES[$i]}" >/dev/null
    done
    ls -la "$out" "$out/inspect"
done

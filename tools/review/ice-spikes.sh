#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Ice spikes report (Ben, 2026-10-20: "like 20x more ice spike biomes than I'd usually see"): a baked planet's
# rare-biome shares against a normal world of the same pack. Scratch: /Volumes/Graphide/ban/deeptime-runs/icespikes/.
#
#   tools/remote/mac.sh --no-sync -- tools/review/ice-spikes.sh dump <name> <world-data folder>
#       the baked biome inputs, Koppen classes and ice cover every 16 blocks on the census's grid
#       (deeptime heightmap; mac.sh admits it) into <name>-layers.npy
#   tools/remote/mac.sh [--pack] -- tools/review/ice-spikes.sh census <standalone|mic> <name> <world-data folder>
#       installs the profile's smoke set (16k), boots it on a copy of that world data and runs
#       `/deeptime probe biomes 16` and `/deeptime probe reference_biomes` (the pack's own router and parameter
#       list, 4 seeds, 400,000 blocks square every 512); writes <name>-<profile>-biomemap.json,
#       <profile>-reference.json, <profile>-reference-parameters.json and <name>-<profile>-climate_mapping.json
#   tools/remote/mac.sh --no-sync -- tools/review/ice-spikes.sh shares <name> <profile>
#       tools/review/ice_spikes.py over them, against both references when both exist
#   tools/remote/mac.sh --no-sync -- tools/review/ice-spikes.sh clean
set -euo pipefail

RUNS=/Volumes/Graphide/ban/deeptime-runs/icespikes
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$RUNS"
cd "$REPO"

case "${1:-}" in
    dump)
        name="${2:?name}"; world="${3:?world-data folder}"
        bin="$REPO/target/release/deeptime"
        [ -x "$bin" ] || CARGO_BUILD_JOBS=8 cargo build --release -q -p deeptime-cli
        # The census's grid (BiomeCensus: step 16, x0 = -C/2 + 8, z0 = -zp + 8; zp = C/4 on Petroff-Guyou).
        c=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["scale"]["circumference_blocks"])' "$world/manifest.json")
        zp=$(( c / 4 ))
        x0=$(( -c / 2 + 8 )); x1=$(( x0 + c ))
        z0=$(( -zp + 8 )); z1=$(( z0 + 2 * zp ))
        "$bin" heightmap --world-data "$world" --box="$x0,$z0,$x1,$z1" --step 16 \
            --layers climate/temperature,climate/vegetation,climate/continents,climate/erosion,climate/ridges,climate/koppen_l2,climate/koppen,climate/ice_cover,climate/wetland,climate/t_mean_c,climate/p_mm \
            --out "$RUNS/$name-layers.npy"
        ls -la "$RUNS/$name-layers.npy" "$RUNS/$name-layers.npy.json"
        ;;
    census)
        profile="${2:?profile}"; name="${3:?name}"; world="${4:?world-data folder}"
        [ -d "$world" ] || { echo "no world data at $world" >&2; exit 1; }
        tools/smoke/install.sh "$profile" --size 16384
        server="smoke/$profile/server"
        rm -rf "$server/planet"
        mkdir -p "$server/planet"
        cp -R "$world" "$server/planet/deeptime"
        rm -rf "$server/planet/deeptime/epochs"
        # The server runs in a private copy of its set: the probes write where the mod is told to.
        PROBE="$RUNS/probe-$profile-$name"; rm -rf "$PROBE"; mkdir -p "$PROBE"
        DEEPTIME_PROBE_DIR="$PROBE" python3 tools/smoke/run.py --profile "$profile" --timeout 3000 \
            --rcon "deeptime probe biomes 16" --rcon "deeptime probe reference_biomes" || true
        cp "$PROBE/biomemap.json" "$RUNS/$name-$profile-biomemap.json"
        cp "$PROBE/reference_biomes.json" "$RUNS/$profile-reference.json"
        cp "$PROBE/reference_biomes-parameters.json" "$RUNS/$profile-reference-parameters.json"
        cp "$server/planet/deeptime/climate_mapping.json" "$RUNS/$name-$profile-climate_mapping.json" 2>/dev/null || true
        ;;
    shares)
        name="${2:?name}"; profile="${3:?profile}"
        refs=()
        for p in standalone mic; do
            [ -e "$RUNS/$p-reference.json" ] && refs+=(--reference "$p=$RUNS/$p-reference.json")
        done
        census=()
        [ -e "$RUNS/$name-$profile-biomemap.json" ] && census=(--census "$RUNS/$name-$profile-biomemap.json")
        python3 tools/review/ice_spikes.py --layers "$RUNS/$name-layers.npy" \
            --mapping "$RUNS/$name-$profile-climate_mapping.json" --params "$RUNS/$profile-reference-parameters.json" \
            "${census[@]}" "${refs[@]}"
        ;;
    clean)
        rm -rf "$RUNS"
        ;;
    *)
        sed -n '3,19p' "$0"; exit 2
        ;;
esac

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Terralith-side biome fixes (Ben, decisions log 2026-10-04, "Deep Time-side fixes (Recommended)"): one quick
# 16k planet baked with and without a pack placement table, then the biome census of each world through a
# profile's biome source. Scratch: /Volumes/Graphide/ban/deeptime-runs/terralith-biomes/.
#
#   tools/remote/mac.sh -- tools/review/terralith-biomes.sh run <name> [placement-patch.json ...]
#       builds the release CLI and runs `deeptime run` (earthlike_quick_16k, scattered continents, 500 Myr,
#       seed 1, 3 threads, detached, through the admission tool) into <name>-world/, each patch given to
#       the bake with --placement-patch; the log is <name>.log
#   tools/remote/mac.sh --no-sync -- tools/review/terralith-biomes.sh dump <name>
#       the world's baked biome inputs, classes, surface rock and province every 16 blocks (deeptime
#       heightmap) into <name>-layers.npy (for tools/review/terralith_emulate.py)
#   tools/remote/mac.sh [--pack] -- tools/review/terralith-biomes.sh census <standalone|mic|terrablender> <name>
#       installs the profile's smoke server (16k), boots it on that world data and runs the probe; the grid
#       and census land in <name>-<profile>-biomemap.json, the world's climate mapping beside it
#   tools/remote/mac.sh --no-sync -- tools/review/terralith-biomes.sh status | clean
set -euo pipefail

RUNS=/Volumes/Graphide/ban/deeptime-runs/terralith-biomes
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$RUNS"
cd "$REPO"

case "${1:-}" in
    run)
        name="$2"; shift 2
        patches=""
        for p in "$@"; do patches="$patches --placement-patch $(realpath "$p")"; done
        CARGO_BUILD_JOBS=8 cargo build --release -p deeptime-cli
        cp target/release/deeptime "$RUNS/deeptime-$name"
        rm -rf "$RUNS/$name-world"
        setsid nohup deeptime-admit run --kind sim:earthlike_quick_16k:3t --label "tb-$name" -- \
            "$RUNS/deeptime-$name" run --preset earthlike_quick_16k --start scattered_continents --length-myr 500 \
            --threads 3 $patches --out "$RUNS/$name-world" > "$RUNS/$name.log" 2>&1 < /dev/null &
        echo "started pid $! -> $RUNS/$name.log"
        ;;
    dump)
        name="${2:?run name}"
        world="$RUNS/$name-world"
        # The census's grid (BiomeCensus: step 16, x0 = -C/2 + 8, z0 = -zp + 8, h = 2 zp / 16 rows).
        zp=$(python3 -c 'import math; print(int(16384/(2*math.pi)))')
        h=$(( 2 * zp / 16 ))
        z0=$(( -zp + 8 )); z1=$(( z0 + 16 * h ))
        "$RUNS/deeptime-$name" heightmap --world-data "$world" --box="-8184,$z0,8200,$z1" --step 16 \
            --layers climate/temperature,climate/vegetation,climate/continents,climate/erosion,climate/ridges,climate/koppen,surface/class_top,meta/province,climate/t_mean_c,climate/p_mm \
            --out "$RUNS/$name-layers.npy"
        ls -la "$RUNS/$name-layers.npy" "$RUNS/$name-layers.npy.json"
        ;;
    census)
        profile="${2:?profile}"; name="${3:?run name}"
        world="$RUNS/$name-world"
        [ -d "$world" ] || { echo "no world data at $world" >&2; exit 1; }
        tools/smoke/install.sh "$profile" --size 16384
        server="smoke/$profile/server"
        rm -rf "$server/planet"
        mkdir -p "$server/planet"
        cp -R "$world" "$server/planet/deeptime"
        # The server runs in a private copy of the set that goes with its job: the probe writes where the mod is told to.
        PROBE="$RUNS/probe-$profile-$name"; rm -rf "$PROBE"; mkdir -p "$PROBE"
        DEEPTIME_PROBE_DIR="$PROBE" python3 tools/smoke/run.py --profile "$profile" --rcon "deeptime probe biomes 16" --timeout 3000 || true
        cp "$PROBE/biomemap.json" "$RUNS/$name-$profile-biomemap.json"
        cp "$server/planet/deeptime/climate_mapping.json" "$RUNS/$name-$profile-climate_mapping.json" 2>/dev/null || true
        python3 tools/review/biomemap_shares.py "$RUNS/$name-$profile-biomemap.json" --classes Am,Csa,Csb,BSk,ET,Cfb,Cwb --top 8
        ;;
    status)
        for f in "$RUNS"/*.log; do [ -e "$f" ] && { echo "== $f"; tail -n 4 "$f"; }; done
        ;;
    clean)
        rm -rf "$RUNS"
        ;;
    *)
        sed -n '3,22p' "$0"; exit 2
        ;;
esac

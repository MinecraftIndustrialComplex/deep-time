#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Volcanic biomes tied to real volcanoes (Ben, decisions log 2026-10-03): one quick 16k planet, then the
# biome census of that one world through each profile's biome source. The census
# (`/deeptime probe biomes 16`) reports the volcanic biomes both ways in one boot: as the plain climate
# search places them (master's behaviour: wherever their parameters fall) and as the volcanic layer does,
# with every hot-spring pool's biome. Scratch: /Volumes/Graphide/ban/deeptime-runs/volcanic-biomes/.
#
#   tools/remote/mac.sh -- tools/review/volcanic-biomes.sh run <name> [length-myr] [start]
#       builds the release CLI and runs `deeptime run` (earthlike_quick_16k, 2 threads, detached) into
#       <name>-world/; the log is <name>.log
#   tools/remote/mac.sh [--pack] --server -- tools/review/volcanic-biomes.sh census <standalone|mic|terrablender> <name>
#       installs the profile's smoke server (16k), boots it on that world data and runs the probe; the grid and
#       census land in <name>-<profile>-biomemap.json
#   tools/remote/mac.sh --no-sync -- tools/review/volcanic-biomes.sh status | clean
set -euo pipefail

RUNS=/Volumes/Graphide/ban/deeptime-runs/volcanic-biomes
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$RUNS"
cd "$REPO"

case "${1:-}" in
    run)
        name="$2"; length="${3:-500}"; start="${4:-scattered_continents}"
        setsid nohup sh -c "CARGO_BUILD_JOBS=8 cargo build --release -p deeptime-cli \
            && cp target/release/deeptime '$RUNS/deeptime-$name' \
            && rm -rf '$RUNS/$name-world' \
            && '$RUNS/deeptime-$name' run --preset earthlike_quick_16k --start '$start' --length-myr '$length' \
               --threads 2 --out '$RUNS/$name-world'" > "$RUNS/$name.log" 2>&1 < /dev/null &
        echo "started pid $! -> $RUNS/$name.log"
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
        python3 tools/smoke/run.py --profile "$profile" --rcon "deeptime probe biomes 16" --timeout 3000 || true
        cp "$server/deeptime-probe/biomemap.json" "$RUNS/$name-$profile-biomemap.json"
        python3 -c 'import json,sys; m=json.load(open(sys.argv[1])); print(json.dumps(m["census"].get("volcanic"), indent=1))' \
            "$RUNS/$name-$profile-biomemap.json"
        ;;
    status)
        for f in "$RUNS"/*.log; do [ -e "$f" ] && { echo "== $f"; tail -n 4 "$f"; }; done
        ;;
    clean)
        rm -rf "$RUNS"
        ;;
    *)
        sed -n '3,17p' "$0"; exit 2
        ;;
esac

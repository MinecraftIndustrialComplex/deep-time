#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Expanded Ecosphere as a first-class target beside Terralith (Ben, decisions log 2026-10-04: "Build out
# support for both so if I install EE it works or if I go w/ terralith it also works"): one quick 16k
# planet, then the biome census of that one world through each profile's biome source (ee: Expanded
# Ecosphere; mic: Terralith; standalone: vanilla). The census (`/deeptime probe biomes 16`) includes the
# biome rules' section (which rule fired where, which mapped biome appeared). The server log's
# "[deeptime] overworld" lines say what Expanded Ecosphere's overworld override did to the world.
# Scratch: /Volumes/Graphide/ban/deeptime-runs/ee-compat/.
#
#   tools/remote/mac.sh -- tools/review/ee-biomes.sh run <name> [length-myr] [start]
#       builds the release CLI and runs `deeptime run` (earthlike_quick_16k, seed 1, 2 threads, detached)
#       into <name>-world/; the log is <name>.log
#   tools/remote/mac.sh [--pack] -- tools/review/ee-biomes.sh census <ee|mic|standalone> <name> [tag]
#       installs the profile's smoke server (16k), boots it on that world data and runs the probe; the grid
#       lands in <name>-<profile>[-tag]-biomemap.json, the server log in <name>-<profile>[-tag]-server.log
#   tools/remote/mac.sh --no-sync -- tools/review/ee-biomes.sh shares <name> [tag]
#       the land shares per class of the three profiles' grids (tools/review/biomemap_shares.py)
#   tools/remote/mac.sh --no-sync -- tools/review/ee-biomes.sh status | clean
set -euo pipefail

RUNS=/Volumes/Graphide/ban/deeptime-runs/ee-compat
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$RUNS"
cd "$REPO"

case "${1:-}" in
    run)
        name="$2"; length="${3:-500}"; start="${4:-scattered_continents}"
        setsid nohup sh -c "CARGO_BUILD_JOBS=8 cargo build --release -p deeptime-cli \
            && cp target/release/deeptime '$RUNS/deeptime-$name' \
            && rm -rf '$RUNS/$name-world' \
            && deeptime-admit run --kind sim:earthlike_quick_16k:2t -- '$RUNS/deeptime-$name' run --preset earthlike_quick_16k \
               --start '$start' --length-myr '$length' --set seed=1 --threads 2 --out '$RUNS/$name-world'" \
            > "$RUNS/$name.log" 2>&1 < /dev/null &
        echo "started pid $! -> $RUNS/$name.log"
        ;;
    census)
        profile="${2:?profile}"; name="${3:?run name}"; tag="${4:+-$4}"
        world="$RUNS/$name-world"
        [ -d "$world" ] || { echo "no world data at $world" >&2; exit 1; }
        tools/smoke/install.sh "$profile" --size 16384
        server="smoke/$profile/server"
        rm -rf "$server/planet"
        mkdir -p "$server/planet"
        if [ -d "$world/deeptime" ]; then cp -R "$world/deeptime" "$server/planet/deeptime"; else cp -R "$world" "$server/planet/deeptime"; fi
        probe="$RUNS/probe-$profile-$name$tag"; rm -rf "$probe"; mkdir -p "$probe"
        DEEPTIME_PROBE_DIR="$probe" python3 tools/smoke/run.py --profile "$profile" --rcon "deeptime probe biomes 16" --timeout 3000 || true
        cp "smoke/$profile/logs/server.log" "$RUNS/$name-$profile$tag-server.log" 2>/dev/null || true
        grep -E "\[deeptime\] (overworld|biome rules|volcanic biomes)|is not a Deep Time level" "$RUNS/$name-$profile$tag-server.log" | head -20 || true
        if [ -f "$probe/biomemap.json" ]; then
            cp "$probe/biomemap.json" "$RUNS/$name-$profile$tag-biomemap.json"
            python3 -c 'import json,sys; m=json.load(open(sys.argv[1])); print(json.dumps(m["census"].get("rules"), indent=1))' \
                "$RUNS/$name-$profile$tag-biomemap.json"
        else
            echo "no biome map: the probe did not run on a Deep Time overworld (see the server log)"
        fi
        ;;
    shares)
        name="${2:?run name}"; tag="${3:+-$3}"
        for p in ee mic standalone; do
            f="$RUNS/$name-$p$tag-biomemap.json"
            [ -f "$f" ] || { echo "== $p: no grid"; continue; }
            echo "== $p"
            python3 tools/review/biomemap_shares.py "$f" --top 8
        done
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

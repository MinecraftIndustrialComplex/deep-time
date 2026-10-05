#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Cold wetland (2026-10-03 owner answer, taiga or peatland): the surface biomes of a finished quick-planet run through a
# profile's biome source, by Koppen class, and the census of the bake's cold-wetland columns. Takes the world data `peat-runs.sh run <name>`
# wrote, puts it in the profile's smoke server (no new simulation), boots it and runs
# `/deeptime probe biomes 16`; the grid lands in <runs>/<name>-<profile>-biomemap.json and
# `tools/review/biomemap_shares.py` turns it into shares per class.
#
#   tools/remote/mac.sh [--pack] --server -- tools/review/peat-biomes.sh <standalone|mic|terrablender> <name>
set -euo pipefail

RUNS=/Volumes/Graphide/ban/deeptime-runs/peat
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
profile="${1:?profile}"; name="${2:?run name}"
world="$RUNS/$name-world"
[ -d "$world" ] || { echo "no world data at $world" >&2; exit 1; }

tools/smoke/install.sh "$profile" --size 16384
server="smoke/$profile/server"
rm -rf "$server/planet"
mkdir -p "$server/planet"
if [ -d "$world/deeptime" ]; then cp -R "$world/deeptime" "$server/planet/deeptime"; else cp -R "$world" "$server/planet/deeptime"; fi
PROBE="$RUNS/probe-$profile-$name"; rm -rf "$PROBE"; mkdir -p "$PROBE"
DEEPTIME_PROBE_DIR="$PROBE" python3 tools/smoke/run.py --profile "$profile" --rcon "deeptime probe biomes 16" --timeout 3000 || true
# The server runs in a private copy of the set that goes with its job: the probe writes where the mod is told to.
cp "$PROBE/biomemap.json" "$RUNS/$name-$profile-biomemap.json"
cp "$server/planet/deeptime/climate_mapping.json" "$RUNS/$name-$profile-climate_mapping.json" 2>/dev/null || true
python3 tools/review/biomemap_shares.py "$RUNS/$name-$profile-biomemap.json" --classes Dfc,Dfd,Dsc,Dwc,Dfb,Cfb --top 14
python3 - "$RUNS/$name-$profile-biomemap.json" <<'PY'
import json, sys
c = json.load(open(sys.argv[1]))["census"]
print("cold_wetland", json.dumps(c.get("cold_wetland")))
print("wetlands_set_aside", json.dumps(c.get("wetlands_set_aside")))
PY

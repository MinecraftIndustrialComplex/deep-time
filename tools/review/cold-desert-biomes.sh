#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Cold deserts (2026-10-02 owner answer): the surface biomes of a finished quick-planet run through a
# profile's biome source, by Koppen class. Takes the world data `cold-schist-runs.sh run <name>`
# wrote, puts it in the profile's smoke server (no new simulation), boots it and runs
# `/deeptime probe biomes 16`; the grid lands in <runs>/<name>-<profile>-biomemap.json and
# `tools/review/biomemap_shares.py` turns it into shares per class.
#
#   tools/remote/mac.sh [--pack] --server -- tools/review/cold-desert-biomes.sh <standalone|mic|terrablender> <name>
set -euo pipefail

RUNS=/Volumes/Graphide/ban/deeptime-runs/cold-desert
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
python3 tools/smoke/run.py --profile "$profile" --rcon "deeptime probe biomes 16" --timeout 3000 || true
cp "$server/deeptime-probe/biomemap.json" "$RUNS/$name-$profile-biomemap.json"
cp "$server/planet/deeptime/climate_mapping.json" "$RUNS/$name-$profile-climate_mapping.json" 2>/dev/null || true
python3 tools/review/biomemap_shares.py "$RUNS/$name-$profile-biomemap.json" --classes BWk,BSk,BWh,BSh,ET,EF --top 14

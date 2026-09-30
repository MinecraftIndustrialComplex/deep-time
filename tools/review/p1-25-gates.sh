#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-25 gates that boot Minecraft (run on the Mac through tools/remote/mac.sh --pack --server):
# Deep Time's GameTests, then mic-climate's own smoke (its smoke/install.sh + run.py, the real pack
# on a flat world, no Deep Time) with mic-climate's deep-time branch jar. The pack is the snapshot
# tools/smoke/install.sh mic leaves in smoke/cache/pack-snapshot (the pack's HEAD with stale index
# hashes fixed and its untracked jars copied in), because the pack's own HEAD may carry a stale
# hash that packwiz refuses. The mic-climate copy is the one tools/remote/mic-climate.sh synced to
# the Mac; its smoke/ folder must hold a copy of the smoke scripts that honours MIC_PACK.
#
#   tools/review/p1-25-gates.sh [--smoke-only]
set -uo pipefail
cd "$(dirname "$0")/../.."
REPO="$PWD"
MC=/Volumes/Graphide/ban/mic/mods/mic-climate
mkdir -p review/out/p1-25
if [ "${1:-}" != "--smoke-only" ]; then
    echo "=== DT gametest"
    ./gametest.sh > review/out/p1-25/dt-gametest-2.txt 2>&1
    grep -E "required tests|climateApi" review/out/p1-25/dt-gametest-2.txt | cut -c1-400
fi
SNAP="$REPO/smoke/cache/pack-snapshot"
[ -f "$SNAP/pack.toml" ] || tools/smoke/install.sh mic > /dev/null
echo "=== mic-climate smoke (pack snapshot: $(grep -o '"pack_ref": *"[^"]*"' smoke/mic/install.json 2>/dev/null))"
cd "$MC"
MIC_PACK="$SNAP" smoke/install.sh > build/smoke-install.txt 2>&1
tail -4 build/smoke-install.txt
python3 smoke/run.py > build/smoke-run.txt 2>&1
tail -14 build/smoke-run.txt
cp smoke/report.md "$REPO/review/out/p1-25/mic-climate-smoke-report.md" 2>/dev/null || true

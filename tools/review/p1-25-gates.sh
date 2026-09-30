#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-25 gates that boot Minecraft (run on the Mac through tools/remote/mac.sh --pack --server):
# Deep Time's GameTests, then mic-climate's own smoke (its smoke/install.sh + run.py, the real pack
# on a flat world, no Deep Time) against a snapshot of the pack's HEAD (the synced working tree may
# hold other sessions' unindexed edits, which packwiz refuses), with mic-climate's deep-time branch
# jar. The mic-climate copy is the one tools/remote/mic-climate.sh synced to the Mac; its smoke/
# folder must hold a copy of the smoke scripts that honours MIC_PACK.
set -uo pipefail
cd "$(dirname "$0")/../.."
C=/Volumes/Graphide/ban/deeptime-cache
MC=/Volumes/Graphide/ban/mic/mods/mic-climate
mkdir -p review/out/p1-25
echo "=== DT gametest"
./gametest.sh > review/out/p1-25/dt-gametest-2.txt 2>&1
grep -E "required tests|climateApi" review/out/p1-25/dt-gametest-2.txt | cut -c1-400
SNAP=$C/tmp/p125/pack-head
rm -rf "$SNAP"; mkdir -p "$SNAP"
"$C/git/bin/git" -C /Volumes/Graphide/ban/mic/pack archive HEAD | tar -xf - -C "$SNAP"
echo "pack HEAD $("$C/git/bin/git" -C /Volumes/Graphide/ban/mic/pack rev-parse --short HEAD)"
echo "=== mic-climate smoke"
cd "$MC"
MIC_PACK="$SNAP" smoke/install.sh > build/smoke-install.txt 2>&1
tail -4 build/smoke-install.txt
python3 smoke/run.py > build/smoke-run.txt 2>&1
tail -14 build/smoke-run.txt

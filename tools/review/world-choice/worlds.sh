#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Cube phase 2, Ben's world-shape decision: the two worlds compare.sh and shots.sh read. The dh set
# (standalone + Distant Horizons) on earthlike_quick_16k baked as a cube (install.sh --cube), its first
# start with the server-side DH checks (run.py --dh: the seam and the slide), then the real client
# (dh-client-check.sh): phase 2's Distant Horizons gate steps. The cube world is then kept, and the
# same preset and seed is baked on Petroff-Guyou in the same set and kept. Reports go to $OUT.
# Each step admits itself. SKIP_CUBE=1 runs only the Petroff-Guyou part.
#   tools/remote/mac.sh -- tools/remote/detach.sh start cube2-gdh bash tools/review/world-choice/worlds.sh
set -uo pipefail
cd "$(dirname "$0")/../../.."
OUT=${OUT:-/Volumes/Graphide/ban/deeptime-runs/cube2-gate}
CMP=${CMP:-/Volumes/Graphide/ban/deeptime-runs/cube2-cmp}
mkdir -p "$OUT" "$CMP"
keep() { # keep <name>: the set's world (level.dat and the world data) for the comparison shots
  rm -rf "${CMP:?}/$1"; mkdir -p "$CMP/$1"
  cp -a smoke/dh/server/planet/level.dat "$CMP/$1/" && cp -a smoke/dh/server/planet/deeptime "$CMP/$1/" \
    && echo "KEPT $1 ($(du -sh "$CMP/$1" | cut -f1))"
}
if [ "${SKIP_CUBE:-0}" != 1 ]; then
  deeptime-admit run --kind auto -- tools/smoke/install.sh dh --size 16384 --preset earthlike_quick_16k --cube \
      --start supercontinent --length-myr 1000 --clean || { echo "INSTALL-FAIL cube"; exit 1; }
  python3 tools/smoke/run.py --profile dh --fresh --dh
  echo "GATE dh run.py cube rc=$?"
  cp smoke/dh/report.md "$OUT/dh-16k-cube-report.md" 2>/dev/null
  tools/client/dh-client-check.sh dh
  echo "GATE dh-client-check cube rc=$?"
  cp smoke/dh/client-check.txt "$OUT/dh-16k-cube-client-check.txt" 2>/dev/null
  keep cube16
fi
deeptime-admit run --kind auto -- tools/smoke/install.sh dh --size 16384 --preset earthlike_quick_16k \
    --start supercontinent --length-myr 1000 --clean || { echo "INSTALL-FAIL pg"; exit 1; }
python3 tools/smoke/run.py --profile dh --fresh
echo "PG first start rc=$?"
cp smoke/dh/report.md "$OUT/dh-16k-pg-report.md" 2>/dev/null
keep pg16
echo "gate-dh done"

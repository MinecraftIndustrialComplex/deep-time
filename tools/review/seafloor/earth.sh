#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Sea-floor spikes study (2026-10-20): ETOPO 2022's deep sea floor by survey.py's measures, the
# 60 arc-second global grid (at 1.85 km and coarsened 3x, 5.6 km) and five 15 arc-second tiles
# (460 m) over rough sea floor: N00W120 (East Pacific Rise flank), N15E135 (Philippine Sea and the
# Mariana arc), N30W045 (Mid-Atlantic Ridge), N30W165 (Pacific fracture zones), S15W180 (Tonga
# and Fiji). The grids were downloaded by hand from NOAA NCEI into $S/etopo60 and $S/etopo15-<tile>
# (each its own folder).
#   earth.sh [study dir]
set -euo pipefail
S=${1:-/data/ban/deeptime-runs/seafloor}
R=$(cd "$(dirname "$0")/../../.." && pwd)
PY='(builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux.python3.withPackages (p: [p.numpy p.scipy p.h5py])'
run() {
    nix shell --impure --expr "$PY" -c python3 -I "$R/tools/review/seafloor/earth_stats.py" "$@" \
        --sigma-km 4.71 --tile-km 37.7 > /dev/null
    python3 -I -c "import json,sys; d=json.load(open(sys.argv[1])); print(sys.argv[1], json.dumps({k: d[k] for k in ('deg_pct','share_over','tile_rms_pct')}))" "$1"
}
run "$S/earth60.json" "$S/etopo60/ETOPO_2022_v1_60s_N90W180_surface.nc"
run "$S/earth60-c3.json" "$S/etopo60/ETOPO_2022_v1_60s_N90W180_surface.nc" --coarsen 3
for t in N00W120 N15E135 N30W045 N30W165 S15W180; do
    run "$S/earth15-$t.json" "$S/etopo15-$t/ETOPO_2022_v1_15s_${t}_surface.nc"
done

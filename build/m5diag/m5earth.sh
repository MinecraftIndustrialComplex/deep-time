#!/usr/bin/env bash
# M5 diagnosis: Earth's control (SRTM15+ 15') on the census grid and on Fibonacci points (VM).
set -uo pipefail
cd "$(dirname "$0")/../.."
mkdir -p /data/ban/deeptime-runs/m5diag
nix shell --impure --expr '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy p.h5py])' -c python3 -I build/m5diag/m5earth.py /data/ban/deeptime-runs/m5diag/earth/earth_relief_15m_p.grd /data/ban/deeptime-runs/m5diag/earth.json

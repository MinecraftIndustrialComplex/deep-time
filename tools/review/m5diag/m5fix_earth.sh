#!/usr/bin/env bash
# M5 measure fixes (RLC-052): Earth's present (SRTM15+) and the 109 PaleoDEMs on the new code (VM).
# Writes /data/ban/deeptime-runs/m5fix/earth-*.txt. Needs the data of tools/review/earth-paleodem.py's header.
set -uo pipefail
cd "$(dirname "$0")/../../.."
out=/data/ban/deeptime-runs/m5fix
mkdir -p "$out"
grd=/data/ban/deeptime-runs/m5diag/earth/earth_relief_15m_p.grd
maps=$(command ls -d "$out"/paleodem/*/ | head -1)
py() { nix shell --impure --expr '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy p.h5py])' -c python3 -I "$@"; }
for n in 40000 100000 332800; do
  py tools/review/earth-paleodem.py relief "$grd" "$n" > "$out/earth-relief-$n.txt" 2>&1
done
for n in 40000 332800; do
  py tools/review/earth-paleodem.py paleodem "$maps" "$n" > "$out/earth-paleodem-$n.txt" 2>&1
done
echo done

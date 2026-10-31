#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The low-continents reads on the VM (measurement only, cached pool runs): lowcont.sh <out dir> <name>=<arm folder>[@N] ...
# Earth (CRUST1.0, SRTM15+), then per arm: lowcont.py series and model, thinning.py classes and census, m5sea.py run;
# then m5sea.py table over the arms, m4cold.py and m5earthbands.py. Start it with
#   tools/remote/mac.sh -- tools/remote/detach.sh start lowcont --kind light bash tools/review/freeboard/lowcont.sh \
#     /data/ban/deeptime-runs/lowcont base35=<folder> base34=<folder>@4 ...
set -u
PY=/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3
OUT=$1
shift
HERE=$(cd "$(dirname "$0")" && pwd)
E=/data/ban/deeptime-runs/m5diag/earth
mkdir -p "$OUT"
# The Earth step needs h5py, which the pool's numpy environment lacks (m5earth.sh's recipe).
nix shell --impure --expr '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy p.h5py])' \
  -c python3 -I "$HERE/lowcont.py" earth "$E/crust1/crust1.bnds" "$E/earth_relief_15m_p.grd" > "$OUT/earth.txt" 2>&1
tail -n 1 "$OUT/earth.txt" > "$OUT/earth.json"
TABLE=()
for kv in "$@"; do
  name=${kv%%=*}
  arg=${kv#*=}
  dir=${arg%%@*}
  echo "== $name $arg"
  "$PY" -I "$HERE/lowcont.py" series "$arg" > "$OUT/series-$name.txt" 2>&1
  "$PY" -I "$HERE/lowcont.py" model "$OUT/earth.json" "$arg" > "$OUT/model-$name.txt" 2>&1
  n=${arg#*@}
  [ "$n" = "$arg" ] && n=12
  "$PY" -I "$HERE/thinning.py" "$dir@$n" > "$OUT/thin-$name.txt" 2>&1
  MODE=census CTIMES=-1000,-500,0 "$PY" -I "$HERE/thinning.py" "$dir@$n" > "$OUT/census-$name.txt" 2>&1
  if [ "$arg" = "$dir" ]; then  # m5sea.py reads every run in the folder: only for whole arms
    rm -f "$OUT/m5sea-$name.jsonl"
    "$PY" -I "$HERE/../m5diag/m5sea.py" run "$OUT/m5sea-$name.jsonl" "$dir" > "$OUT/m5sea-run-$name.txt" 2>&1
    TABLE+=("$name=$OUT/m5sea-$name.jsonl")
  fi
done
"$PY" -I "$HERE/../m5diag/m5sea.py" table "${TABLE[@]}" > "$OUT/m5sea-table.txt" 2>&1
"$PY" -I "$HERE/../m5diag/m4cold.py" "$E/crust1/crust1.bnds" "$E/crust1/crust1.rho" > "$OUT/m4cold.txt" 2>&1
"$PY" -I "$HERE/../m5diag/m5earthbands.py" "$E/crust1/crust1.bnds" > "$OUT/m5earthbands.txt" 2>&1
echo DONE

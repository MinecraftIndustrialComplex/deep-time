#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The coast-high diagnosis over an arm's cached pool folders (VM):
#   coasthi.sh model <out name> <workers> <arm folder>     coasthi.py over every run of the folder
#   coasthi.sh plates <out name> <workers> <arm folder>    coasthi_plates.py over every run of the folder
#   coasthi.sh earth <out name>                            coasthi.py on SRTM15+ 15' and PB2002
#   coasthi.sh paleodem <out name>                         coasthi.py on the PaleoDEMs of 0, 5, 10, 280 and 380 Ma
#   coasthi.sh etopo <out name> [block arc-min]            coasthi.py on ETOPO 2022 bedrock (by rock), block means (15')
# Writes /data/ban/deeptime-runs/coasthi/<out name>.jsonl (model, plates) or .json (Earth).
set -uo pipefail
cd "$(dirname "$0")/../../.."
OUT=/data/ban/deeptime-runs/coasthi
mkdir -p "$OUT"
PY=/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3
case "$1" in
  model|plates)
    name=$2 workers=$3 root=$4
    if [ "$1" = model ]; then script=(tools/review/coasthi/coasthi.py model); else script=(tools/review/coasthi/coasthi_plates.py); fi
    rm -f "$OUT/$name".part*.jsonl
    command ls -d "$root"/*/ | grep -v '\.lock' | sed 's:/$::' | awk -v n="$workers" '{print > ("/tmp/coasthi-'"$name"'-" (NR % n))}'
    for k in $(seq 0 $((workers - 1))); do
      [ -f "/tmp/coasthi-$name-$k" ] || continue
      xargs -a "/tmp/coasthi-$name-$k" "$PY" -I "${script[@]}" "$OUT/$name.part$k.jsonl" &
    done
    wait
    cat "$OUT/$name".part*.jsonl > "$OUT/$name.jsonl"
    rm -f "$OUT/$name".part*.jsonl /tmp/coasthi-"$name"-*
    wc -l "$OUT/$name.jsonl"
    ;;
  paleodem)
    name=$2
    D=/data/ban/deeptime-runs/m5fix/paleodem/Scotese_Wright_2018_Maps_1-88_1degX1deg_PaleoDEMS_nc_v2
    nix shell --impure --expr '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy p.h5py])' \
      -c python3 -I tools/review/coasthi/coasthi.py paleodem "$OUT/$name.json" "$D"/Map01_*_0Ma.nc "$D"/*_5Ma.nc "$D"/*_10Ma.nc \
      "$D"/*_280Ma.nc "$D"/*_380Ma.nc
    ;;
  etopo)
    # ETOPO 2022 bedrock (downloaded data, its own folder, not committed):
    #   curl -sSL -o ETOPO_2022_v1_60s_N90W180_bed.nc https://www.ngdc.noaa.gov/thredds/fileServer/global/ETOPO2022/60s/60s_bed_elev_netcdf/ETOPO_2022_v1_60s_N90W180_bed.nc
    name=$2
    nix shell --impure --expr '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy p.h5py])' \
      -c python3 -I tools/review/coasthi/coasthi.py etopo "$OUT/$name.json" "$OUT/etopo-bed/ETOPO_2022_v1_60s_N90W180_bed.nc" \
      /data/ban/deeptime-runs/m5diag/earth/PB2002_steps.dat.txt "${3:-15}"
    ;;
  earth)
    name=$2
    nix shell --impure --expr '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy p.h5py])' \
      -c python3 -I tools/review/coasthi/coasthi.py earth "$OUT/$name.json" /data/ban/deeptime-runs/m5diag/earth/earth_relief_15m_p.grd \
      /data/ban/deeptime-runs/m5diag/earth/PB2002_steps.dat.txt
    ;;
esac

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# V1 item 1's seven measures for an A/B of two builds (plates.md, "V1 item 1's seven measures";
# the high-ground lane's driver, 2026-10-22). Three steps, each runnable on its own on the test VM
# (start the long ones through tools/remote/detach.sh; every simulation and dump admits itself):
#
#   run    <runs dir> <arm> <binary> <seeds,...> [+ENV=v ...] [--set k=v ...]
#          force-balance-ensemble.sh in ck mode on both starts (supercontinent and
#          scattered_continents) with +DEEPTIME_CRUST_BUDGET=1, then `census` below.
#   census <runs dir> <arm> <seeds,...>
#          per run: dump-fields of the final checkpoint (<arm>/fields/<start>-<seed>/) and
#          continents_census.py fields on it (<arm>/census/<start>-<seed>.txt).
#   table  <runs dir> <base arm> <arm> [<lo>-<hi>]
#          v1-item1-table.py: per run the seven measures, the medians, the two-sided Mann-Whitney p
#          per measure and of the criteria met per run, and how many runs meet each criterion;
#          written to <runs dir>/v1-<arm>-vs-<base arm>.txt, the summary to stdout.
#
# Runs are <runs dir>/<arm>/<arm>-<start>-<seed>-<LEN>.*; a run whose summary exists is not run
# again, so a base arm is run once and shared (/data/ban/deeptime-runs/pool/BASES.md lists them).
# Environment: PRESET (earthlike_quick_16k), LEN (1000), T (threads per run, 2). Build base and arm
# from a `git archive` of their commits, each in its own folder, and run them side by side.
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start v1-sheet --kind shell:ensemble \
#       tools/review/v1-item1-table.sh run /data/ban/deeptime-runs/x sheet /data/ban/x/bin/sheet \
#       1,2,3,4,5,6 --set tectonics.collapse_sheet_viscosity_pa_s=1e22
#   tools/remote/mac.sh --kind shell:analysis -- tools/review/v1-item1-table.sh table \
#       /data/ban/deeptime-runs/x base sheet 1-6
set -uo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
PY='with import (builtins.getFlake "nixpkgs") {}; python3.withPackages (ps: [ps.numpy ps.scipy])'
LEN=${LEN:-1000}
STARTS="supercontinent scattered_continents"
FIELDS=elevation,surface/elevation_m,plates/elevation_m,plates/thickness_m,plates/crust_type,plates/orogeny_type,plates/orogeny_age_myr,climate/ice_thickness_m,climate/ela_m,climate/t_warm_c,climate/t_mean_c,climate/koppen,regime,surface/sediment_m,surface/class,crust_age,plates/flags,plates/top_pool
cmd=${1:-}; shift || true

census() {
  local R=$1 A=$2 seeds=${3//,/ } start seed f out c
  mkdir -p "$R/$A/fields" "$R/$A/census"
  for start in $STARTS; do
    for seed in $seeds; do
      f=$R/$A/$A-$start-$seed-$LEN
      out=$R/$A/fields/$start-$seed
      [ -s "$f.ck" ] || { echo "missing $f.ck"; continue; }
      if [ ! -s "$out/globals.json" ]; then
        env -u DEEPTIME_ADMIT_JOB -u DEEPTIME_ADMIT_PID deeptime-admit run --kind shell:dump \
          --label "v1-dump-$A-$start-$seed" -- \
          "$R/$A/deeptime.bin" dump-fields --in "$f.ck" --out "$out.part" --fields "$FIELDS" > "$out.log" 2>&1 \
          && rm -rf "$out" && mv "$out.part" "$out" || { echo "FAILED dump $f"; continue; }
      fi
      c=$R/$A/census/$start-$seed.txt
      [ -s "$c" ] || nix shell --impure --expr "$PY" -c python3 -I "$HERE/continents_census.py" fields "$out" > "$c" 2>&1
    done
  done
  echo "CENSUS_DONE $A"
}

case "$cmd" in
  run)
    R=$1; A=$2; B=$3; S=$4; shift 4
    mkdir -p "$R/$A"
    bash "$HERE/force-balance-ensemble.sh" "$R/$A" "$B" "$LEN" "$S" "${STARTS// /,}" ck "$A" \
      +DEEPTIME_CRUST_BUDGET=1 "$@"
    census "$R" "$A" "$S"
    echo "ARM_DONE $A"
    ;;
  census)
    census "$1" "$2" "$3"
    ;;
  table)
    R=$1; A0=$2; A1=$3; SEEDS=${4:-1-6}
    O="$R/v1-$A1-vs-$A0.txt"
    LEN=$LEN nix shell --impure --expr "$PY" -c python3 -I "$HERE/v1-item1-table.py" "$R" "$A0" "$A1" "$SEEDS" > "$O" 2>&1
    grep -E "^(SUMMARY|RUNS|COUNT)" "$O" || tail -5 "$O"
    ;;
  *)
    sed -n '3,30p' "$0"
    exit 2
    ;;
esac

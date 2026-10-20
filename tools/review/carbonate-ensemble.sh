#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The carbonate and ore-deposit task's runs (2026-10-19): whole planets of one build, each run,
# measured and deleted inside one admission entry, at most LANES at a time. Each run keeps its log
# (the climate_final, carbon_series and surface_final lines), `strata-check` (the land, sea and
# stored-cell censuses) and `deposit-audit` (every site's row: stratum, crust, height); the world
# folder is deleted after. Plain words for detach.sh:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start <job> --kind light \
#       tools/review/carbonate-ensemble.sh <out dir> <binary> <label> <lanes> <preset> \
#       <length Myr> <threads> <start:seed,...> [CLI args...]
#
# The binary is copied first, so a rebuild does not change a running ensemble. A run whose audit
# exists is not run again. Read the results with tools/review/carbonate-table.py.
set -uo pipefail
OUT=$1; BIN=$2; LABEL=$3; LANES=$4; PRESET=$5; LEN=$6; T=$7; RUNS=${8//,/ }; shift 8
mkdir -p "$OUT"
cmp -s "$BIN" "$OUT/$LABEL.bin" || cp "$BIN" "$OUT/$LABEL.bin" || exit 1
BIN=$OUT/$LABEL.bin
printf '%s\n' "$PRESET $LEN ${T}t [$RUNS] $*" > "$OUT/$LABEL.args"
one() {
  local start=$1 seed=$2 f=$3; shift 3
  local w="$f.world"
  rm -rf "$w"
  "$BIN" run --preset "$PRESET" --start "$start" --length-myr "$LEN" --threads "$T" \
    --set "seed=$seed" --out "$w" "$@" > "$f.part" 2>&1 || { echo "FAILED run $f" >&2; return 1; }
  mv "$f.part" "$f.txt"
  "$BIN" strata-check --world-data "$w" > "$f.strata.json" 2> "$f.strata.err" \
    || echo "FAILED strata-check $f" >&2
  "$BIN" deposit-audit --world-data "$w" --json "$f.audit.json" > "$f.audit.md" 2> "$f.audit.err" \
    || echo "FAILED deposit-audit $f" >&2
}
export -f one
export BIN PRESET LEN T
n=0
for r in $RUNS; do
  start=${r%%:*}; seed=${r##*:}
  f="$OUT/$LABEL-$start-$seed"
  if [ -s "$f.audit.json" ]; then continue; fi
  while [ "$(jobs -rp | wc -l)" -ge "$LANES" ]; do wait -n; done
  (
    env -u DEEPTIME_ADMIT_JOB -u DEEPTIME_ADMIT_PID deeptime-admit run \
      --kind "sim:$PRESET:${T}t" --label "carb-$LABEL-$start-$seed" -- \
      nice -n 10 bash -c 'one "$@"' _ "$start" "$seed" "$f" "$@"
    rm -rf "$f.world"
  ) &
  n=$((n + 1))
  sleep 5
done
wait
echo ALLDONE

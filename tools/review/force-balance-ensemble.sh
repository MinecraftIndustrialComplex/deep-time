#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Force-balance tasks 7 and 8 (docs/design/plate-force-balance.md): an ensemble of coupled-pipeline
# runs of one build on seeds x starts, every simulation admitted on its own by the test VM's
# admission tool, then `plate-check` per run, per start and pooled ("ck" mode), or only the runs'
# summaries for tools/review/p1-17-hyps.py ("hyps" mode). Written to start through detach.sh, whose
# command line is plain words, so seeds and starts are comma-separated:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start <job> tools/review/force-balance-ensemble.sh \
#       <out dir> <binary> <length Myr> <seeds,...> <starts,...> <ck|hyps> [<label>] [--set k=v ...]
#
# The binary is copied into <out dir> first, so a rebuild does not change a running ensemble.
# Environment: PRESET (default earthlike_quick_16k), T (threads per run, default 2). A run whose
# summary already exists is not run again. Checkpoints (.ck, about 0.8 GB each) stay in <out dir>
# for plate-check; delete the folder when the task lands.
set -uo pipefail
OUT=$1; BIN=$2; LEN=$3; SEEDS=${4//,/ }; STARTS=${5//,/ }; MODE=$6; shift 6
LABEL=r
if [ $# -gt 0 ] && [ "${1#--}" = "$1" ]; then LABEL=$1; shift; fi
SETS=("$@")
PRESET=${PRESET:-earthlike_quick_16k}
T=${T:-2}
mkdir -p "$OUT"
cp "$BIN" "$OUT/deeptime.bin" || exit 1
BIN=$OUT/deeptime.bin
printf '%s\n' "$PRESET $LEN [$SEEDS] [$STARTS] $MODE ${SETS[*]:-}" > "$OUT/$LABEL.args"
pids=()
for start in $STARTS; do
  for seed in $SEEDS; do
    f="$OUT/$LABEL-$start-$seed-$LEN"
    if [ -s "$f.txt" ]; then continue; fi
    (
      if [ "$MODE" = ck ]; then extra=(--out "$f.ck"); else extra=(); fi
      env -u DEEPTIME_ADMIT_JOB -u DEEPTIME_ADMIT_PID deeptime-admit run --kind "sim:$PRESET:${T}t" \
        --label "fb-$LABEL-$start-$seed" -- \
        nice -n 10 "$BIN" stage volcanoes --preset "$PRESET" --start "$start" --length-myr "$LEN" \
        --threads "$T" --set "seed=$seed" --work "$f.work" "${extra[@]}" "${SETS[@]}" > "$f.part" 2>&1 \
        && mv "$f.part" "$f.txt" || echo "FAILED $f" >&2
      rm -rf "$f.work"
    ) &
    pids+=($!)
    sleep 2
  done
done
for p in "${pids[@]}"; do wait "$p"; done
if [ "$MODE" = ck ]; then
  all=()
  for start in $STARTS; do
    runs=()
    for seed in $SEEDS; do
      f="$OUT/$LABEL-$start-$seed-$LEN"
      [ -s "$f.ck" ] || continue
      runs+=(--run "$f.ck")
      all+=(--run "$f.ck")
      "$BIN" plate-check --run "$f.ck" --json "$f.json" > "$f.table.txt" 2>&1 || true
    done
    [ ${#runs[@]} -gt 0 ] || continue
    "$BIN" plate-check "${runs[@]}" --json "$OUT/$LABEL-$start-$LEN.json" \
      > "$OUT/$LABEL-$start-$LEN.table.txt" 2>&1 || true
  done
  if [ ${#all[@]} -gt 0 ]; then
    "$BIN" plate-check "${all[@]}" --json "$OUT/$LABEL-all-$LEN.json" \
      > "$OUT/$LABEL-all-$LEN.table.txt" 2>&1 || true
  fi
fi
echo ALLDONE

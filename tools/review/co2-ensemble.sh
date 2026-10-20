#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# An ensemble of one build's runs through the volcano stage, each admitted on its own, for the
# end-of-run CO2 hunt (2026-10-19). Each run keeps its log (the climate_final, carbon_series and
# surface_final lines) and the climate stage's note (--climate-json: the clock's whole series and
# the present-day nudge), then deletes its work folder. Plain words for detach.sh:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start <job> tools/review/co2-ensemble.sh \
#       <out dir> <binary> <preset> <length Myr> <threads> <seeds,...> <starts,...> <label> [CLI args...]
#
# A start or length of "-" keeps the preset's own (a file preset such as planet_world_4k).
#
# The binary is copied first, so a rebuild does not change a running ensemble. A run whose log
# already exists is not run again. Each seed and start is admitted on its own; to keep the hunt
# to a few admission entries at once, drive one-seed calls through tools/review/co2-driver.sh.
set -uo pipefail
OUT=$1; BIN=$2; PRESET=$3; LEN=$4; T=$5; SEEDS=${6//,/ }; STARTS=${7//,/ }; LABEL=$8; shift 8
mkdir -p "$OUT"
# (Not over a copy that is running: a driver starts one seed at a time with the same label.)
cmp -s "$BIN" "$OUT/$LABEL.bin" || cp "$BIN" "$OUT/$LABEL.bin" || exit 1
BIN=$OUT/$LABEL.bin
printf '%s\n' "$PRESET $LEN ${T}t [$SEEDS] [$STARTS] $*" > "$OUT/$LABEL.args"
pids=()
for start in $STARTS; do
  for seed in $SEEDS; do
    f="$OUT/$LABEL-$start-$seed-$LEN-${T}t"
    # "-" keeps the preset's own start or length (a file preset such as planet_world_4k).
    timeline=()
    [ "$start" = - ] || timeline+=(--start "$start")
    [ "$LEN" = - ] || timeline+=(--length-myr "$LEN")
    if [ -s "$f.txt" ]; then continue; fi
    (
      env -u DEEPTIME_ADMIT_JOB -u DEEPTIME_ADMIT_PID deeptime-admit run --kind "sim:$PRESET:${T}t" \
        --label "co2-$LABEL-$start-$seed" -- \
        nice -n 10 "$BIN" stage volcanoes --preset "$PRESET" "${timeline[@]}" \
        --threads "$T" --set "seed=$seed" --work "$f.work" --climate-json "$f.climate.json" "$@" \
        > "$f.part" 2>&1 && mv "$f.part" "$f.txt" || echo "FAILED $f" >&2
      rm -rf "$f.work"
    ) &
    pids+=($!)
    sleep 2
  done
done
for p in "${pids[@]}"; do wait "$p"; done
echo ALLDONE

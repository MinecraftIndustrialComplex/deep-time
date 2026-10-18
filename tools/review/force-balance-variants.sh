#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Force-balance task 8: build this checkout's CLI once, then run one force-balance-ensemble.sh per
# variant side by side (each simulation and the build admitted on their own, so the wrapper itself is `--kind light`). Start through detach.sh from the
# repo root; the variants file holds one line per variant: `<label> [--set k=v ...]`.
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start <job> --kind light tools/review/force-balance-variants.sh \
#       <out dir> <variants file> <length Myr> <seeds,...> <starts,...> <ck|hyps>
set -uo pipefail
OUT=$1; VARIANTS=$2; LEN=$3; SEEDS=$4; STARTS=$5; MODE=$6
HERE=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$OUT"
BIN="$OUT/deeptime.build"
deeptime-admit run --kind cargo:build --label "fbv-build" -- bash "$HERE/force-balance-build.sh" "$BIN" || exit 1
pids=()
while read -r label rest; do
  [ -n "$label" ] && [ "${label#\#}" = "$label" ] || continue
  # shellcheck disable=SC2086
  bash "$HERE/force-balance-ensemble.sh" "$OUT/$label" "$BIN" "$LEN" "$SEEDS" "$STARTS" "$MODE" "$label" $rest \
    > "$OUT/$label.log" 2>&1 &
  pids+=($!)
  sleep 5
done < "$VARIANTS"
for p in "${pids[@]}"; do wait "$p"; done
echo VARIANTS_DONE

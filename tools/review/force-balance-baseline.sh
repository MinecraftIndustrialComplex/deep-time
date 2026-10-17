#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Force-balance tasks 7 and 8: the baseline to compare against. Builds the CLI from another source
# tree on the test VM (an extracted `git archive` of master), then runs one plate-check ensemble
# ("ck") and one hypsometry ensemble ("hyps") of it with force-balance-ensemble.sh. Start through
# detach.sh from the repo root:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start <job> --kind light tools/review/force-balance-baseline.sh \
#       <source dir> <out dir> <ck length> <ck seeds,...> <hyps length> <hyps seeds,...>
set -uo pipefail
SRC=$1; OUT=$2; CKLEN=$3; CKSEEDS=$4; HLEN=$5; HSEEDS=$6
HERE=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$OUT"
BIN="$OUT/deeptime.build"
deeptime-admit run --kind cargo:build --label "fb-baseline-build" -- \
  bash -c "cd '$SRC' && CARGO_TARGET_DIR='$OUT/target' cargo build --release -q -p deeptime-cli && cp '$OUT/target/release/deeptime' '$BIN'" || exit 1
rm -rf "$OUT/target"
STARTS=supercontinent,scattered_continents
bash "$HERE/force-balance-ensemble.sh" "$OUT/ck" "$BIN" "$CKLEN" "$CKSEEDS" "$STARTS" ck m > "$OUT/ck.log" 2>&1 &
p1=$!
sleep 5
bash "$HERE/force-balance-ensemble.sh" "$OUT/hyps" "$BIN" "$HLEN" "$HSEEDS" "$STARTS" hyps h > "$OUT/hyps.log" 2>&1 &
p2=$!
wait $p1 $p2
echo BASELINE_DONE

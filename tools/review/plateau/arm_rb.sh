#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The runoff lane's reads of plat-rb0 against plat-cb0 and orog-cum (VM, cached pool folders only, no simulation):
# wear.sh on plat-rb0, recal.sh on the three, closedshare.py on the two closed arms, the measures side by side.
#   tools/remote/mac.sh -- tools/remote/detach.sh start runoff-arm --kind light bash tools/review/plateau/arm_rb.sh
set -uo pipefail
cd "$(dirname "$0")/../../.."
PY=/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3
OUT=/data/ban/deeptime-runs/plateau
POOL=/data/ban/deeptime-runs/pool
RB=$POOL/a3b047bb6ddb/earthlike_quick_16k-1000/7591e701
CB=$POOL/e751b2344693/earthlike_quick_16k-1000/5e8a748c
OC=$POOL/f2d6412d801f/earthlike_quick_16k-1000/d3261b0a
bash tools/review/plateau/wear.sh platrb0="$RB"
bash tools/review/plateau/recal.sh platrb0="$RB" platcb0="$CB" orogcum="$OC"
{
    nice -n 19 "$PY" -I tools/review/plateau/closedshare.py "$RB" "$CB"
    nice -n 19 "$PY" -I tools/review/plateau/measures.py platrb0="$RB" platcb0="$CB" orogcum="$OC"
} > "$OUT/arm-rb.txt" 2> "$OUT/arm-rb.log"
echo "done arm-rb"

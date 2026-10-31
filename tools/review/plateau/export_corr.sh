#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The runoff lane: export_corr.py over the closed, runoff, orog-cum and base34 arms (VM, cached runs, no simulation)
#   -> /data/ban/deeptime-runs/plateau/export-corr.txt
#   tools/remote/mac.sh -- tools/remote/detach.sh start export-corr --kind light bash tools/review/plateau/export_corr.sh
set -uo pipefail
cd "$(dirname "$0")/../../.."
PY=/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3
POOL=/data/ban/deeptime-runs/pool
nice -n 19 "$PY" -I tools/review/plateau/export_corr.py \
    platrb0=$POOL/a3b047bb6ddb/earthlike_quick_16k-1000/7591e701 \
    platcb0=$POOL/e751b2344693/earthlike_quick_16k-1000/5e8a748c \
    orogcum=$POOL/f2d6412d801f/earthlike_quick_16k-1000/d3261b0a \
    base34=$POOL/122e4803cded/earthlike_quick_16k-1000/cd665cef \
    > /data/ban/deeptime-runs/plateau/export-corr.txt 2>&1
echo "done export-corr"

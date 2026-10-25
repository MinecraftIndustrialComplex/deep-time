#!/usr/bin/env bash
# M4 diagnosis (VM, cached pool folders only): m4split.sh <run folder> <name> [workers] [runs]
# Writes /data/ban/deeptime-runs/m5diag/m4split-<name>.jsonl (m4split.py) and m4table-<name>.txt (m4table.py).
set -uo pipefail
cd "$(dirname "$0")/../../.."
mkdir -p /data/ban/deeptime-runs/m5diag
export M5DIAG_RUNS="$1"
py=/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3
$py -I tools/review/m5diag/m4split.py "/data/ban/deeptime-runs/m5diag/m4split-$2.jsonl" "${3:-3}" ${4:-}
$py -I tools/review/m5diag/m4table.py "/data/ban/deeptime-runs/m5diag/m4split-$2.jsonl" > "/data/ban/deeptime-runs/m5diag/m4table-$2.txt"
cat "/data/ban/deeptime-runs/m5diag/m4table-$2.txt"
echo "M4SPLIT_SH_DONE $2"

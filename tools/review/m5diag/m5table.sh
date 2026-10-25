#!/usr/bin/env bash
# M5 diagnosis: the tables from the three outputs (VM): m5table.sh [fields] [maps] [out]; M5DIAG_RUNS names the arm's folder.
set -uo pipefail
cd "$(dirname "$0")/../../.."
D=/data/ban/deeptime-runs/m5diag
/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3 -I tools/review/m5diag/m5table.py $D/${1:-fields}.jsonl $D/${2:-maps}.jsonl $D/earth.json > $D/${3:-table}.txt 2>&1
cat $D/${3:-table}.txt

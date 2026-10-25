#!/usr/bin/env bash
# M5 diagnosis: M5 through time from an arm's crust-budget maps (VM, cached pool folders only): m5maps.sh [workers] [out] [runs];
# M5DIAG_RUNS names the arm's folder (default base12's).
set -uo pipefail
cd "$(dirname "$0")/../../.."
mkdir -p /data/ban/deeptime-runs/m5diag
/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3 -I tools/review/m5diag/m5maps.py /data/ban/deeptime-runs/m5diag/${2:-maps}.jsonl ${1:-3} ${3:-}

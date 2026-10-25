#!/usr/bin/env bash
# M5 diagnosis: the census-grid analysis of an arm's runs (VM, cached pool folders only): m5fields.sh [workers] [out] [runs];
# M5DIAG_RUNS names the arm's folder (default base12's).
set -uo pipefail
cd "$(dirname "$0")/../../.."
mkdir -p /data/ban/deeptime-runs/m5diag
/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3 -I tools/review/m5diag/m5fields.py /data/ban/deeptime-runs/m5diag/${2:-fields}.jsonl ${1:-3} ${3:-}

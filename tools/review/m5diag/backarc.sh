#!/usr/bin/env bash
# The back-arc and front-length diagnosis (VM, cached pool folders only): backarc.sh <pool key[,key]> <name> [runs]
# Writes /data/ban/deeptime-runs/m5diag/ba/ba-<name>.jsonl (backarc.py) and read-<name>.txt (backarc_read.py).
# base14: backarc.sh ac619d59fdab,a963c9ac56ee base14
set -uo pipefail
cd "$(dirname "$0")/../../.."
py=/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3
d=/data/ban/deeptime-runs/m5diag/ba
mkdir -p "$d"
if [ -n "${3:-}" ]; then
  deeptime-admit run --kind auto -- env BAKEY="$1" "$py" -I tools/review/m5diag/backarc.py "$d/ba-$2.jsonl" 1 "$3"
else
  deeptime-admit run --kind auto -- env BAKEY="$1" "$py" -I tools/review/m5diag/backarc.py "$d/ba-$2.jsonl" 4
fi
"$py" -I tools/review/m5diag/backarc_read.py "$d/ba-$2.jsonl" | tee "$d/read-$2.txt"
echo "BACKARC_SH_DONE $2"

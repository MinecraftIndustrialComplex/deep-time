#!/usr/bin/env bash
# The collision front's crust flux, rate and belt diagnosis (VM, cached pool folders only):
# collfront.sh <pool key[,key]> <name> [runs]
# Writes /data/ban/deeptime-runs/m5diag/cf/cf-<name>.jsonl (collfront.py) and read-<name>.txt (collfront_read.py).
# base14: collfront.sh ac619d59fdab,a963c9ac56ee base14
set -uo pipefail
cd "$(dirname "$0")/../../.."
py=/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3
d=/data/ban/deeptime-runs/m5diag/cf
mkdir -p "$d"
if [ -n "${3:-}" ]; then
  deeptime-admit run --kind auto -- env BAKEY="$1" "$py" -I tools/review/m5diag/collfront.py "$d/cf-$2.jsonl" 1 "$3"
else
  deeptime-admit run --kind auto -- env BAKEY="$1" "$py" -I tools/review/m5diag/collfront.py "$d/cf-$2.jsonl" 4
fi
"$py" -I tools/review/m5diag/collfront_read.py "$d/cf-$2.jsonl" | tee "$d/read-$2.txt"
echo "COLLFRONT_SH_DONE $2"

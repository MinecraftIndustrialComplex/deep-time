#!/usr/bin/env bash
# M4 diagnosis: Earth's control from CRUST1.0 (VM). Fetches the model into its own folder once
# (data, not code: the script reads it by path) and runs m4earth.py on its boundaries file.
set -uo pipefail
cd "$(dirname "$0")/../../.."
d=/data/ban/deeptime-runs/m5diag/earth/crust1
mkdir -p "$d"
if [ ! -f "$d/crust1.bnds" ]; then
  curl -sSfL -o "$d/crust1.0.tar.gz" "http://igppweb.ucsd.edu/%7Egabi/crust1/crust1.0.tar.gz"
  tar -xzf "$d/crust1.0.tar.gz" -C "$d"
  f=$(find "$d" -name crust1.bnds | head -1)
  [ -n "$f" ] && [ "$f" != "$d/crust1.bnds" ] && cp "$f" "$d/crust1.bnds"
fi
/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3 -I tools/review/m5diag/m4earth.py "$d/crust1.bnds"

#!/usr/bin/env bash
# Earth's collision belts per km of collision front (VM): earthcoll.sh
# Reads CRUST1.0 and PB2002's steps from /data/ban/deeptime-runs/m5diag/earth; prints earthcoll.py's tables.
set -uo pipefail
cd "$(dirname "$0")/../../.."
e=/data/ban/deeptime-runs/m5diag/earth
/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3 -I tools/review/m5diag/earthcoll.py "$e/crust1/crust1.bnds" "$e/PB2002_steps.dat.txt"

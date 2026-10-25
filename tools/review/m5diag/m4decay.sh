#!/usr/bin/env bash
# What slows the keel-less belts (VM, cached pool folders only): m4decay.sh <name> <run or arm folder> [more folders]
# Writes /data/ban/deeptime-runs/m5diag/m4decay-<name>.txt (m4decay.py) and prints it.
set -uo pipefail
cd "$(dirname "$0")/../../.."
mkdir -p /data/ban/deeptime-runs/m5diag
name="$1"
shift
py=/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3
nice -n 10 $py -I tools/review/m5diag/m4decay.py "$@" > "/data/ban/deeptime-runs/m5diag/m4decay-$name.txt" 2>&1
cat "/data/ban/deeptime-runs/m5diag/m4decay-$name.txt"
echo "M4DECAY_SH_DONE $name"

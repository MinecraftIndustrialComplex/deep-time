#!/usr/bin/env bash
# The drowning diagnosis (VM, cached pool folders only): m5sea.sh <name> <arm folder>[@N] ...
# Writes /data/ban/deeptime-runs/m5diag/m5sea-<name>.jsonl; read it with m5sea.py table.
set -uo pipefail
cd "$(dirname "$0")/../../.."
mkdir -p /data/ban/deeptime-runs/m5diag
name=$1
shift
/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3 -I tools/review/m5diag/m5sea.py run "/data/ban/deeptime-runs/m5diag/m5sea-$name.jsonl" "$@"

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The end-of-run CO2 hunt's runs (2026-10-19) through at most LANES admission entries, so other
# agents' jobs slot in between: a task starts only while fewer than LANES of this hunt's
# simulations (any `deeptime-admit run --kind sim:` whose command line names
# /data/ban/deeptime-runs/co2-hothouse/, waiting or running, from this driver or not) are on the
# box. Each line of the task file is one tools/review/co2-ensemble.sh call (one seed and one start
# keep it to one simulation); `#` starts a comment. A run whose log exists is skipped there.
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start co2-driver --kind light \
#       tools/review/co2-driver.sh <lanes> <task file>
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 1
LANES=$1; TASKS=$2
R=/data/ban/deeptime-runs/co2-hothouse/
log() { echo "$(date -u +%H:%M:%S) $*"; }
mine() {
  local n=0 d c
  for d in /proc/[0-9]*; do
    [ -O "$d" ] || continue
    c=$(tr '\0' ' ' < "$d/cmdline" 2>/dev/null) || continue
    case $c in *"admit run --kind sim:"*"$R"*|*"admit.py run --kind sim:"*"$R"*) n=$((n + 1)) ;; esac
  done
  echo "$n"
}
pids=()
while read -r line; do
  line=${line%%#*}; set -- $line; [ $# -gt 0 ] || continue
  while [ "$(mine)" -ge "$LANES" ]; do sleep 30; done
  log "start: $*"
  tools/review/co2-ensemble.sh "$@" > /dev/null 2>&1 &
  pids+=($!)
  # Let its admission entry appear before counting again.
  sleep 20
done < "$TASKS"
for p in "${pids[@]}"; do wait "$p"; done
log ALLDONE

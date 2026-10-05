#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Starts the after4 and before4 planets (the old rule is a content bundle without cold_wetland), waits, then runs peat-gates.sh.
set -uo pipefail
cd "$(dirname "$0")/../.."
RUNS=/Volumes/Graphide/ban/deeptime-runs/peat
bundle=$(tools/review/peat-runs.sh old-bundle)
tools/review/peat-runs.sh run after4 500 scattered_continents
sleep 5
tools/review/peat-runs.sh run before4 500 scattered_continents --content "$bundle"
for n in after4 before4; do
    until grep -aq "admit: sim.* done in" "$RUNS/$n.log" 2>/dev/null || grep -aqi "^error\|panicked" "$RUNS/$n.log" 2>/dev/null; do sleep 20; done
    tail -n 3 "$RUNS/$n.log"
done
tools/review/peat-gates.sh

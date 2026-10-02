#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Measures BiomeGameTests.climateConsistent's shares on several test planets in one profile (how its
# floors are set: the lowest share over several planets, less a margin). On the test VM:
#
#   tools/remote/mac.sh --server -- 'tools/review/climate-floors.sh "<seeds>" [gradle -P flags]'
#   e.g. tools/review/climate-floors.sh "11 12 13 14" -PwithTerraBlender
#
# Prints one line per planet: seed, then family:share(columns) for every family with columns.
set -uo pipefail
SEEDS=${1:?seeds}; shift
LOG=neoforge/build/gametest.log
for seed in $SEEDS; do
    DEEPTIME_TEST_PLANET_SET="seed=$seed" ./gametest.sh "$@" -PgametestOnly=climateConsistent > /dev/null 2>&1
    printf 'seed=%s [%s] ' "$seed" "$(grep -oE 'All [0-9]+ required tests passed|[0-9]+ required tests failed' "$LOG" | head -1)"
    grep 'climateConsistent {' "$LOG" | head -1 \
        | grep -oE '"[A-Za-z:]+":\{"classes":"[^"]*","columns":[0-9]+,"matching":[0-9]+,"share":[0-9.]+' \
        | sed -E 's/"([A-Za-z:]+)":\{"classes":"[^"]*","columns":([0-9]+),"matching":[0-9]+,"share":([0-9.]+)/\1:\3(\2)/' \
        | tr '\n' ' '
    echo
done

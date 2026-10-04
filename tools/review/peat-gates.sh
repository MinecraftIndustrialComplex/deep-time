#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Cold wetland gates, run on the test VM as one detached script (a dropped laptop session loses only the wait):
#   tools/remote/mac.sh -- tools/remote/detach.sh start peatgates tools/review/peat-gates.sh
#   tools/remote/mac.sh --no-sync -- tools/remote/detach.sh wait peatgates
# Gradle build + JUnit, ./gametest.sh on standalone, MIC and TerraBlender (with Biomes O' Plenty), then the TerraBlender
# biome shares of the two finished planets (peat-runs.sh run after3 / before3).
set -uo pipefail
cd "$(dirname "$0")/../.."
echo "=== gradle"
(cd neoforge && deeptime-admit run --kind gradle:build -- ./gradlew build test 2>&1 | grep -E "BUILD|FAILED|error:"; python3 ../tools/ci/junit-summary.py build/test-results/test)
for flag in "" "-PwithMic" "-PwithTerraBlender"; do
    echo "=== gametest $flag"
    [ "$flag" = "-PwithMic" ] && tools/smoke/install.sh mic > /dev/null 2>&1
    ./gametest.sh $flag 2>&1 | grep -E "passed|failed|FAILED|required|BUILD|climateConsistent|cold_wetland" | tail -n 25
done
for name in after3 before3; do
    echo "=== terrablender $name"
    tools/review/peat-biomes.sh terrablender "$name" 2>&1 | grep -v "^admit" | tail -n 90
done
echo PEATGATES_DONE

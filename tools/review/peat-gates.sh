#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Cold wetland gates, run on the test VM as one detached script (a dropped laptop session loses only the wait):
#   tools/remote/mac.sh -- tools/remote/detach.sh start peatgates tools/review/peat-gates.sh
#   tools/remote/mac.sh --no-sync -- tools/remote/detach.sh wait peatgates
# cargo (fmt, clippy, test, deny), Gradle build + JUnit, ./gametest.sh on standalone, MIC, TerraBlender and EE (with Biomes
# O' Plenty), then the biome shares of the two finished planets (peat-runs.sh run after4 / before4) per profile.
set -uo pipefail
cd "$(dirname "$0")/../.."
echo "=== cargo"
cargo fmt --check 2>&1 | tail -n 5; echo FMT_DONE
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -n 5; echo CLIPPY_DONE
cargo test --workspace 2>&1 | grep -E "^test result|FAILED|panicked|^error" | head -n 40; echo TEST_DONE
cargo deny check licenses bans 2>&1 | tail -n 2; echo DENY_DONE
echo "=== gradle"
(cd neoforge && ./gradlew build test 2>&1 | grep -E "BUILD|FAILED|error:"; python3 ../tools/ci/junit-summary.py build/test-results/test)
for flag in "" "-PwithMic" "-PwithTerraBlender" "-PwithEe"; do
    echo "=== gametest $flag"
    [ "$flag" = "-PwithMic" ] && tools/smoke/install.sh mic > /dev/null 2>&1
    [ "$flag" = "-PwithEe" ] && tools/smoke/install.sh ee > /dev/null 2>&1
    ./gametest.sh $flag 2>&1 | grep -E "passed|failed|FAILED|required|BUILD|climateConsistent|cold_wetland" | tail -n 25
done
for prof in standalone mic terrablender ee; do
    for name in after4 before4; do
        echo "=== $prof $name"
        tools/review/peat-biomes.sh "$prof" "$name" 2>&1 | grep -v "^admit" | tail -n 90
    done
done
echo PEATGATES_DONE

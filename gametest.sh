#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Deep Time's headless GameTest suite (plan §9.4), adapted from mic-climate's script.
#
#   ./gametest.sh                    Deep Time + Toroidal World only
#   ./gametest.sh -PwithMic          plus the current MIC server set (smoke/mic/server/mods)
#   ./gametest.sh -PwithTerraBlender plus TerraBlender, Biomes O' Plenty, GlitchCore
#   ./gametest.sh -PwithEv           plus Excavated Variants and Dynamic Asset Generator
#
# Boots NeoForge's GameTest server (namespace `deeptime`, -Ddeeptime.gametest=true), runs
# every required test and exits non-zero if anything failed or the run never reached the
# tests. Every assertion logs the numbers it compared on a "[gametest]" line. The log is
# left in neoforge/build/gametest.log.

set -uo pipefail
cd "$(dirname "$0")"

# Inside the dev shell already (tools/remote/mac.sh enters it once)? Then run directly.
dev() { if [ -n "${IN_NIX_SHELL:-}" ]; then "$@"; else nix develop -c "$@"; fi; }

LOG=neoforge/build/gametest.log
mkdir -p neoforge/build

# The GameTest server reuses its world folder; stale test levels and world data from an
# older build would be tested again, so they go first.
rm -rf run-gametest/world

# World data for the test_planet_data level (P0-10): the analytic terrain written as world data.
dev cargo run --release -q -p deeptime-cli -- synth --kind wrap-test --size 1024 --out run-gametest/world/deeptime-test \
    || { echo "FAILED: deeptime synth"; exit 1; }
# World data for the test_lake level (P0-13): a basin and a closed channel holding water at y 120.
dev cargo run --release -q -p deeptime-cli -- synth --kind lake-test --size 1024 --out run-gametest/world/deeptime-lake \
    || { echo "FAILED: deeptime synth lake-test"; exit 1; }

peak_rss_file=$(mktemp)
echo 0 > "$peak_rss_file"
(
    while true; do
        rss=$(ps -eo rss=,args= 2>/dev/null \
              | grep -F 'net.neoforged.devlaunch.Main' | grep -F 'run-gametest' \
              | grep -v grep \
              | awk '{ if ($1 > max) max = $1 } END { print max + 0 }')
        if [ -n "${rss:-}" ] && [ "$rss" -gt "$(cat "$peak_rss_file")" ]; then
            echo "$rss" > "$peak_rss_file"
        fi
        sleep 2
    done
) &
sampler=$!
trap 'kill "$sampler" 2>/dev/null; rm -f "$peak_rss_file"' EXIT

start=$(date +%s)
(cd neoforge && dev ./gradlew runGameTestServer --console=plain "$@") 2>&1 | tee "$LOG"
status=${PIPESTATUS[0]}
elapsed=$(( $(date +%s) - start ))
peak_kb=$(cat "$peak_rss_file")

echo
echo "--------------------------------------------------------------------"
if ! grep -q "GAME TESTS COMPLETE" "$LOG"; then
    echo "FAILED: the server never got as far as running the tests."
    grep -nE "Failed to start|MixinTransformerError|InjectionError|InvalidInjectionException|Mod loading has failed|Cannot find class|Missing test structure" "$LOG" | head -20
    echo "Full log: $LOG"
    exit 1
fi
grep -E "GAME TESTS COMPLETE|required tests (failed|passed)|GameTestServer\]:    - " "$LOG" | tail -20
echo
grep -E "\[gametest\]" "$LOG" | sed 's/^.*\[gametest\]/[gametest]/' | tail -40
echo
if grep -qE "InjectionError|MixinTransformerError|InvalidInjectionException|Critical injection failure" "$LOG"; then
    echo "Mixin PROBLEMS:"
    grep -nE "InjectionError|MixinTransformerError|InvalidInjectionException|Critical injection failure" "$LOG" | head -10
    status=1
else
    echo "Mixins: all Deep Time mixins bound (defaultRequire 1, and the server reached the tests)"
fi
echo "Wall time: ${elapsed}s   Peak server-JVM RSS: $(( peak_kb / 1024 )) MB"
echo "Full log:  $LOG"
exit "$status"

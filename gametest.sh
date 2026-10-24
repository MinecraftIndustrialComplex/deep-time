#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Deep Time's headless GameTest suite (plan §9.4), adapted from mic-climate's script.
#
#   ./gametest.sh                    Deep Time + Toroidal World only
#   ./gametest.sh -PwithMic          plus the current MIC server set (smoke/mic/server/mods) and the
#                                    MIC rock-tag datapack (-Ddeeptime.extraDatapack; see build.gradle)
#   ./gametest.sh -PextraDatapack=<folder>   one more datapack folder for any profile
#   ./gametest.sh -PwithTerraBlender plus TerraBlender, Biomes O' Plenty, GlitchCore
#   ./gametest.sh -PwithEv           plus Excavated Variants and Dynamic Asset Generator
#   ./gametest.sh -PwithEe           plus Expanded Ecosphere, Cristel Lib, Cloth Config (tools/smoke/profiles/ee.txt)
#   ./gametest.sh -PgametestOnly=<name>[,<name>…]   only those tests
#   DEEPTIME_TEST_PLANET_SET="seed=12" ./gametest.sh   the real-planet level on another planet (see below)
#
# Boots NeoForge's GameTest server (namespace `deeptime`, -Ddeeptime.gametest=true), runs
# every required test and exits non-zero if anything failed or the run never reached the
# tests. Every assertion logs the numbers it compared on a "[gametest]" line. The log is
# left in neoforge/build/gametest-<set>.log, and copied to neoforge/build/gametest.log.
#
# On the test VM it runs through the admission tool (tools/remote/vm/admit.py, kind
# gametest:<set>; Ben 2026-10-03: load-based admission, no server lock): it starts when its load
# fits, and runs in its own folder (the job's DEEPTIME_SERVER_DIR/run-gametest) with a copy of the
# set's jars and its own voice-chat port, so several GameTest runs, from one checkout too, run at once.

set -uo pipefail
SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
cd "$(dirname "$0")"

# The server set this run links (neoforge/build.gradle serverSetFor): its name is the job's kind.
SET=standalone
for a in "$@"; do
    case "$a" in
        -PwithMic) SET=mic ;;
        -PwithTerraBlender) [ "$SET" = standalone ] && SET=terrablender ;;
        -PwithEv) [ "$SET" = standalone ] && SET=ev ;;
        -PwithEe) [ "$SET" = standalone ] && SET=ee ;;
        -PrunSet=*) SET="${a#-PrunSet=}" ;;
    esac
done
ADMIT="${DEEPTIME_ADMIT:-/usr/local/lib/deeptime/admit.py}"
if [ "${DEEPTIME_ADMIT_PID:-}" != "$$" ] && [ -f "$ADMIT" ]; then
    exec python3 "$ADMIT" run --kind "gametest:$SET" --server --label "$(basename "$PWD")" -- "$SELF" "$@"
fi

# Inside the dev shell already (tools/remote/mac.sh enters it once)? Then run directly.
dev() { if [ -n "${IN_NIX_SHELL:-}" ]; then "$@"; else nix develop -c "$@"; fi; }

mkdir -p neoforge/build
LOG="neoforge/build/gametest-$SET.log"
RUN=run-gametest
if [ -n "${DEEPTIME_SERVER_DIR:-}" ]; then
    RUN="$DEEPTIME_SERVER_DIR/run-gametest"
    export DEEPTIME_GAMETEST_DIR="$RUN"
fi
mkdir -p "$RUN"

# The GameTest server reuses its world folder; stale test levels and world data from an
# older build would be tested again, so they go first.
rm -rf "$RUN/world"

# World data for the test_planet_data level (P0-10): the analytic terrain written as world data,
# with every rock class in its horizons (P1-12's StrataGameTests.everyClassInTheWorld).
dev cargo run --release -q -p deeptime-cli -- synth --kind wrap-test --strata every-class --size 1024 --out "$RUN"/world/deeptime-test \
    || { echo "FAILED: deeptime synth"; exit 1; }
# World data for the test_deposits levels (P1-15): the deposit engine's synthetic run (sites, bodies,
# events) with a terrain baked from the synthetic history's present elevation, C = 4096.
dev cargo run --release -q -p deeptime-cli -- run --preset crates/deeptime-core/tests/presets/deposits_world_4k.json --out "$RUN"/world/deeptime-deposits \
    || { echo "FAILED: deeptime run deposits_world_4k"; exit 1; }
# World data for the test_planet_real level (P1-08): a small real planet (every stage, baked, format
# 0.1) and Rust's column evaluator on 500 of its columns (parity/columns.json), which the real-planet
# GameTests compare placed blocks with.
# DEEPTIME_TEST_PLANET_SET="seed=12 planet.ocean_fraction=0.6" bakes another planet from the same
# preset (each word a `--set`): the real-planet tests assert properties, not one planet's numbers,
# so they must pass on any of them (the simulation is not bit-identical across platforms).
planet_sets=()
for kv in ${DEEPTIME_TEST_PLANET_SET:-}; do planet_sets+=(--set "$kv"); done
# The pack placement tables the set's biome mods get (decisions log 2026-10-04; a dedicated server chooses them
# from its biome registry at the first start, which the GameTest server does not run): Terralith's with MIC.
# PlacementPackGameTests checks that the bake used exactly the tables the server's registry chooses.
case "$SET" in
    mic) planet_sets+=(--placement-patch crates/deeptime-core/data/climate_mapping/packs/terralith.json) ;;
esac
dev cargo run --release -q -p deeptime-cli -- run --preset crates/deeptime-core/tests/presets/planet_world_4k.json "${planet_sets[@]}" --out "$RUN"/world/deeptime-real \
    || { echo "FAILED: deeptime run planet_world_4k"; exit 1; }
dev cargo run --release -q -p deeptime-cli -- parity --world-data "$RUN"/world/deeptime-real \
    || { echo "FAILED: deeptime parity"; exit 1; }
# World data for the test_planet_cube and test_cube_bare levels (cube plan G15): the same planet baked as a
# cube world (format 0.7; s = 1,024, period 4,096), for CubeWorldGameTests.
dev cargo run --release -q -p deeptime-cli -- run --preset crates/deeptime-core/tests/presets/planet_world_4k.json "${planet_sets[@]}" \
    --set scale.projection=cube --out "$RUN"/world/deeptime-cube \
    || { echo "FAILED: deeptime run planet_world_4k as a cube"; exit 1; }
# World data for the test_lake level (P0-13): a basin and a closed channel holding water at y 120.
dev cargo run --release -q -p deeptime-cli -- synth --kind lake-test --size 1024 --out "$RUN"/world/deeptime-lake \
    || { echo "FAILED: deeptime synth lake-test"; exit 1; }
# World data for the test_river_depth level (HYD-120 to HYD-124): a big river across sea-level flats into the
# sea and a flush stream on a plain, with their network (RiverDepthGameTests).
dev cargo run --release -q -p deeptime-cli -- synth --kind river-test --size 1024 --out "$RUN"/world/deeptime-river \
    || { echo "FAILED: deeptime synth river-test"; exit 1; }
# World data for the test_delta level (V1 gap 5, HYD-136 to HYD-140): a delta plain a block over the sea's water
# with a river and two distributaries leaving it at the apex (DeltaGameTests).
dev cargo run --release -q -p deeptime-cli -- synth --kind delta-test --size 1024 --out "$RUN"/world/deeptime-delta \
    || { echo "FAILED: deeptime synth delta-test"; exit 1; }
# World data for the test_meander level (V1 gap 5, HYD-155 to HYD-159): a straight river on a floodplain,
# meandered as the bake meanders it (MeanderGameTests).
dev cargo run --release -q -p deeptime-cli -- synth --kind meander-test --size 1024 --out "$RUN"/world/deeptime-meander \
    || { echo "FAILED: deeptime synth meander-test"; exit 1; }

# The set's jars, copied for this run (an install.sh refreshing smoke/<set> meanwhile cannot change
# them), and this job's voice-chat port (Simple Voice Chat binds one even in a GameTest server).
if [ "$SET" != standalone ]; then
    if [ -d "smoke/$SET/server/mods" ]; then
        python3 tools/smoke/instance.py snapshot-mods --set "smoke/$SET/server" --dest "$RUN/set-mods" >/dev/null \
            || { echo "FAILED: could not copy smoke/$SET/server/mods"; exit 1; }
        export DEEPTIME_RUN_MODS="$PWD/$RUN/set-mods"
        [ "${RUN#/}" != "$RUN" ] && export DEEPTIME_RUN_MODS="$RUN/set-mods"
    fi
fi
python3 tools/smoke/instance.py ports --dir "$RUN"

peak_rss_file=$(mktemp)
echo 0 > "$peak_rss_file"
sampler=""
tailer=""
trap 'kill $sampler $tailer 2>/dev/null; rm -f "$peak_rss_file"' EXIT

# Gradle writes the run's argument files (system properties: the datapack, the tests, DH expected)
# into neoforge/build/moddev just before it starts the game JVM: two runs of this checkout must not
# interleave there, so the launch holds the checkout's launch lock until this run's JVM is up.
exec 7>neoforge/build/.run-launch.lock
flock -n 7 || { echo "gametest.sh: waiting for this checkout's other Gradle run to start its JVM"; flock 7; }
start=$(date +%s)
: > "$LOG"
(cd neoforge && dev ./gradlew runGameTestServer --console=plain "$@") >> "$LOG" 2>&1 7>&- &
gradle_pid=$!
tail -n +1 -f --pid=$gradle_pid "$LOG" 7>&- &
tailer=$!
jvm=$(python3 tools/smoke/instance.py wait-jvm --dir "$RUN" --pid $gradle_pid --timeout 1800 7>&-)
flock -u 7
exec 7>&-
if [ -n "$jvm" ]; then
    (
        # This run's game JVM: the one whose working directory is this run's folder.
        while [ -r "/proc/$jvm/status" ]; do
            rss=$(awk '/^VmRSS:/ { print $2 }' "/proc/$jvm/status" 2>/dev/null)
            if [ -n "${rss:-}" ] && [ "$rss" -gt "$(cat "$peak_rss_file")" ]; then
                echo "$rss" > "$peak_rss_file"
            fi
            sleep 2
        done
    ) &
    sampler=$!
fi
wait $gradle_pid
status=$?
wait $tailer 2>/dev/null
elapsed=$(( $(date +%s) - start ))
peak_kb=$(cat "$peak_rss_file")
cp "$LOG" neoforge/build/gametest.log

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

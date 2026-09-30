#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Hemisphere seasons end to end (mic-climate's `seasons` branch; run on the Mac through
# tools/remote/mac.sh --pack --server): a MIC planet server (earthlike_quick_16k, scattered
# continents, 500 Myr) with this checkout's Deep Time (climate API 2: the planet's circumference on
# both sides) and mic-climate's seasons jar in place of the pack's, probing a grid of places with
# `/mic_climate probe` in northern midwinter and midsummer. The probe's `seasons` line is Serene
# Seasons' season as seen at the block (latitude, strength, the hemisphere's sub-season, the one its
# decisions use), its seasonal biome temperature beside the level's (snow and ice below 0.15), wheat
# fertility and the melt rate; `pa-season` is Project Atmosphere's regional season and sunlight;
# `pa-base` its snow/freeze and rain-or-snow temperatures. The southern row is probed once more with
# the hemisphere seasons switched off, for comparison. Results: smoke/mic/rcon.jsonl and
# smoke/mic/report.md, copied to review/out/seasons/.
#
#   MIC_CLIMATE_JAR=<jar on the Mac> tools/review/seasons-probe.sh
#
# The grid: 45 N (z = -2048), 15 N (z = -683), the equator and 45 S (z = 2048), eight longitudes
# 2048 blocks apart (C = 16384), so each latitude has some land whatever today's planet looks like.
set -euo pipefail
cd "$(dirname "$0")/../.."
MICJAR="${MIC_CLIMATE_JAR:-/Volumes/Graphide/ban/mic/mods/mic-climate-seasons/build/libs/mic_climate-1.21.1-0.1.0.jar}"
[ -f "$MICJAR" ] || { echo "seasons-probe: no mic-climate jar at $MICJAR" >&2; exit 1; }
ROWS=(n45:-2048 n15:-683 eq:0 s45:2048)
XS=(-7168 -5120 -3072 -1024 1024 3072 5120 7168)

tools/smoke/install.sh mic --preset earthlike_quick_16k --start scattered_continents --length-myr 500
mods=smoke/mic/server/mods
rm -f "$mods"/mic_climate-*.jar
cp "$MICJAR" "$mods/mic_climate-1.21.1-0.1.0.jar"
args=smoke/mic/server/user_jvm_args.txt
grep -q driveAtmosphere "$args" || echo "-Dmic_climate.driveAtmosphere=true" >> "$args"
# Mixin writes every class it transformed to .mixin.out/: the check (below) that each of mic-climate's
# optional (require = 0) injectors really landed in its target. (Not mixin.debug.countInjections: that
# turns every other mod's harmless zero-match injector into a fatal error.)
sed -i.bak '/countInjections/d' "$args" && rm -f "$args.bak"
grep -q "mixin.debug.export=" "$args" || echo "-Dmixin.debug.export=true" >> "$args"
rm -rf smoke/mic/server/.mixin.out

probe_row() {
    local z="$1"
    for x in "${XS[@]}"; do
        rc+=(--rcon "execute positioned $x 0 $z positioned over motion_blocking_no_leaves run mic_climate probe ~ ~ ~")
    done
}
probe_all() {
    for r in "${ROWS[@]}"; do probe_row "${r#*:}"; done
}

rc=()
chunky=()
for r in "${ROWS[@]}"; do
    z="${r#*:}"
    for x in "${XS[@]}"; do
        chunky+=(--chunky-at "$x,$z,16")
        rc+=(--rcon "forceload add $x $z")
    done
done
rc+=(--rcon "sleep 10" --rcon "gamerule doDaylightCycle false" --rcon "time set 6000" --rcon "mic_climate seasons")
rc+=(--rcon "season set mid_winter")
probe_all
rc+=(--rcon "mic_climate seasons off")
probe_row 2048
rc+=(--rcon "mic_climate seasons config")
rc+=(--rcon "season set mid_summer")
probe_all

python3 tools/smoke/run.py --profile mic --fresh "${chunky[@]}" "${rc[@]}" --timeout 3600 || true
out=review/out/seasons
mkdir -p "$out"
cp smoke/mic/rcon.jsonl smoke/mic/report.md "$out/" 2>/dev/null || true
grep -hE "Hemisphere seasons|hemisphere seasons|Applying .*Mixin|Skipping .*Mixin|micc[$]|mic_climate.*[Ii]njection" smoke/mic/server/logs/latest.log > "$out/mixins.txt" 2>/dev/null || true
# Which of mic-climate's handlers each patched class now calls (from the exported bytecode).
exp=smoke/mic/server/.mixin.out/class
: > "$out/injections.txt"
for c in sereneseasons/season/SeasonHooks sereneseasons/init/ModFertility sereneseasons/season/RandomUpdateHandler \
         sereneseasons/block/SeasonSensorBlock com/Gabou/sereneseasonsplus/features/logic/SnowAccumulationPolicy \
         net/Gabou/projectatmosphere/seasons/SereneSeasonsSeasonDelegate \
         net/Gabou/projectatmosphere/modules/atmosphere/AtmosphericUpdateScheduler; do
    f="$exp/$c.class"
    if [ -f "$f" ]; then
        # A call to a merged micc$ handler exists only where an injection point matched.
        echo "== $c" >> "$out/injections.txt"
        javap -c -p "$f" | grep -oE 'Method [A-Za-z0-9_$]*micc\$[A-Za-z]+' | sort | uniq -c >> "$out/injections.txt"
    else
        echo "== $c: not exported (not loaded, or no mixin applied)" >> "$out/injections.txt"
    fi
done
cat "$out/injections.txt"
command ls -la "$out"

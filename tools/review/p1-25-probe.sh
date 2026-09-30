#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-25 end to end (run on the Mac through tools/remote/mac.sh --pack --server): a MIC planet server
# (earthlike_quick_16k, scattered continents, 500 Myr) with mic-climate's deep-time branch jar in
# place of the pack's, probing a few places with Deep Time's `/deeptime climate` (the simulated
# climate, through the public API) beside `/mic_climate probe` (what Power Grid, Destroy, LSO and
# CROWNS read, and Project Atmosphere's own reading) in two Serene Seasons dates, then with Project
# Atmosphere's regional simulation running (driven, since no player is online) at noon and at
# midnight. Results: smoke/mic/rcon.jsonl and smoke/mic/report.md, copied to review/out/p1-25/.
#
#   MIC_CLIMATE_JAR=<jar on the Mac> tools/review/p1-25-probe.sh
#
# The places are P1-09's areas on the same preset (tools/review/p1-09-smoke.sh): Af (tropics),
# BW (desert), Dfc (subarctic), ET (tundra) and EF (the ice-cap plateau, southern hemisphere).
set -euo pipefail
cd "$(dirname "$0")/../.."
MICJAR="${MIC_CLIMATE_JAR:-/Volumes/Graphide/ban/mic/mods/mic-climate/build/libs/mic_climate-1.21.1-0.1.0.jar}"
[ -f "$MICJAR" ] || { echo "p1-25-probe: no mic-climate jar at $MICJAR" >&2; exit 1; }
SITES=(af:-3968,32 bw:-4544,-1120 dfc:7504,-2416 et:2800,-3184 ef:1744,2912)

tools/smoke/install.sh mic --preset earthlike_quick_16k --start scattered_continents --length-myr 500
mods=smoke/mic/server/mods
rm -f "$mods"/mic_climate-*.jar
cp "$MICJAR" "$mods/mic_climate-1.21.1-0.1.0.jar"
args=smoke/mic/server/user_jvm_args.txt
grep -q driveAtmosphere "$args" || echo "-Dmic_climate.driveAtmosphere=true" >> "$args"

probe_all() {
    for s in "${SITES[@]}"; do
        IFS=, read -r x z <<<"${s#*:}"
        rc+=(--rcon "execute positioned $x 0 $z positioned over motion_blocking_no_leaves run deeptime climate ~ ~ ~")
        rc+=(--rcon "execute positioned $x 0 $z positioned over motion_blocking_no_leaves run mic_climate probe ~ ~ ~")
    done
}
rc=()
chunky=()
for s in "${SITES[@]}"; do
    IFS=, read -r x z <<<"${s#*:}"
    chunky+=(--chunky-at "$x,$z,32")
    rc+=(--rcon "forceload add $x $z")
done
rc+=(--rcon "sleep 5" --rcon "gamerule doDaylightCycle false" --rcon "time set 6000")
# 1. The climate in two seasons, Project Atmosphere not simulating (no player online).
rc+=(--rcon "season set mid_summer")
probe_all
rc+=(--rcon "season set mid_winter")
probe_all
# 2. Project Atmosphere simulating (driven): noon, then midnight, in midwinter.
rc+=(--rcon "mic_climate atmosphere drive 3000" --rcon "sleep 150")
probe_all
rc+=(--rcon "time set 18000" --rcon "mic_climate atmosphere drive 3000" --rcon "sleep 150")
probe_all
rc+=(--rcon "weather rain" --rcon "mic_climate atmosphere drive 2000" --rcon "sleep 100")
probe_all

python3 tools/smoke/run.py --profile mic --fresh "${chunky[@]}" "${rc[@]}" --timeout 3600 || true
out=review/out/p1-25
mkdir -p "$out"
cp smoke/mic/rcon.jsonl smoke/mic/report.md "$out/" 2>/dev/null || true
ls -la "$out"

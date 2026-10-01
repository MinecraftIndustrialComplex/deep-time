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
# The places are one land site of each class, Af (tropics), BW* (desert), Dfc (subarctic), ET
# (tundra) and EF (ice cap), picked from the planet this run generates: a first server start scans
# `/deeptime climate` on a grid (every 512 blocks in x, 16 latitudes; it reads the baked climate, no
# chunk is generated) and takes, per class, the site nearest a reference latitude (Af the equator,
# Dfc z -2400, ET z -3200), the warmest desert and the coldest ice cap. Fixed coordinates went stale
# twice as the terrain changed: P1-09's areas, used until 2026-09-30, had become sea at four of five.
# SITES="name:x,z ..." skips the scan and probes those places instead. The scan is kept in
# review/out/p1-25/scan.jsonl and the chosen sites in sites.txt.
set -euo pipefail
cd "$(dirname "$0")/../.."
MICJAR="${MIC_CLIMATE_JAR:-/Volumes/Graphide/ban/mic/mods/mic-climate/build/libs/mic_climate-1.21.1-0.1.0.jar}"
[ -f "$MICJAR" ] || { echo "p1-25-probe: no mic-climate jar at $MICJAR" >&2; exit 1; }
out=review/out/p1-25
mkdir -p "$out"

tools/smoke/install.sh mic --preset earthlike_quick_16k --start scattered_continents --length-myr 500
mods=smoke/mic/server/mods
rm -f "$mods"/mic_climate-*.jar
cp "$MICJAR" "$mods/mic_climate-1.21.1-0.1.0.jar"
args=smoke/mic/server/user_jvm_args.txt
grep -q driveAtmosphere "$args" || echo "-Dmic_climate.driveAtmosphere=true" >> "$args"

fresh=--fresh
if [ -n "${SITES:-}" ]; then
    read -r -a SITES <<<"$SITES"
else
    scan=()
    for z in -3584 -3328 -3072 -2816 -2560 -2304 -1280 -1024 -128 0 128 1024 1280 3072 3328 3584; do
        for x in $(seq -8192 512 7680); do
            scan+=(--rcon "deeptime climate $x 64 $z")
        done
    done
    python3 tools/smoke/run.py --profile mic --fresh "${scan[@]}" --timeout 3600 > "$out/scan-run.log" 2>&1 || true
    cp smoke/mic/rcon.jsonl "$out/scan.jsonl"
    fresh=   # the probe below reuses the world the scan generated
    picked=$(python3 - "$out/scan.jsonl" <<'PY'
import json, sys
rows = [p for p in (json.loads(l).get("probe") or {} for l in open(sys.argv[1]))
        if p.get("probe") == "climate" and "koppen" in p]
def pick(name, ok, key):
    c = [r for r in rows if ok(r["koppen"])]
    return f'{name}:{min(c, key=key)["x"]},{min(c, key=key)["z"]}' if c else None
print(" ".join(p for p in [
    pick("af", lambda k: k == "Af", lambda r: abs(r["z"])),
    pick("bw", lambda k: k.startswith("BW"), lambda r: -r["mean_c"]),
    pick("dfc", lambda k: k == "Dfc", lambda r: abs(r["z"] + 2400)),
    pick("et", lambda k: k == "ET", lambda r: abs(r["z"] + 3200)),
    pick("ef", lambda k: k == "EF", lambda r: r["mean_c"]),
] if p))
PY
)
    read -r -a SITES <<<"$picked"
fi
echo "p1-25-probe: sites ${SITES[*]}" | tee "$out/sites.txt"
[ "${#SITES[@]}" -gt 0 ] || { echo "p1-25-probe: no sites (scan found no land of any class?)" >&2; exit 1; }

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

python3 tools/smoke/run.py --profile mic $fresh "${chunky[@]}" "${rc[@]}" --timeout 3600 || true
cp smoke/mic/rcon.jsonl smoke/mic/report.md "$out/" 2>/dev/null || true
ls -la "$out"

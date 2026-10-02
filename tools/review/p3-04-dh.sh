#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P3-04's server checks in one go (run on the test machine under one hold of the server lock:
#   tools/remote/mac.sh --pack --server -- tools/remote/detach.sh run p3-04 tools/review/p3-04-dh.sh):
#
#   1. smoke `dh` (standalone + Distant Horizons at the MIC pack's pin): a first start of a real
#      planet (earthlike_quick_16k, scattered continents, 200 Myr), then run.py --dh: the override
#      registers, DH's server-side /dh pregen is fed from the world data, what DH stores is far
#      terrain layer for layer and agrees with the chunks generated afterwards, section timings.
#   2. smoke `dh-next` (the newest Distant Horizons checked) on the synthetic wrap-test world:
#      the override registers through Toroidal World's guard (Deep Time's patch, TwDhGuard).
#   3. the far-terrain GameTests alone (far terrain against generated chunks on the real test planet).
#
#   tools/review/p3-04-dh.sh [dh] [dh-next] [gametest] [gametest-mic] [mic]     (default: dh dh-next gametest)
#
# `mic` runs step 1 on the whole MIC pack instead (Terralith's surface rules, C2ME, TerraBlender).
# Each step's result is printed at the end; the reports are smoke/<profile>/report.md and
# neoforge/build/gametest.log.
set -uo pipefail
cd "$(dirname "$0")/../.."
STEPS=("$@")
[ ${#STEPS[@]} -gt 0 ] || STEPS=(dh dh-next gametest)
declare -A RESULT
for step in "${STEPS[@]}"; do
    case "$step" in
        dh|mic)
            tools/smoke/install.sh "$step" --preset earthlike_quick_16k --start scattered_continents --length-myr 200 \
                && python3 tools/smoke/run.py --profile "$step" --fresh --dh --timeout 3000
            RESULT[$step]=$? ;;
        dh-next)
            tools/smoke/install.sh dh-next --synth wrap-test \
                && python3 tools/smoke/run.py --profile dh-next --fresh --dh --timeout 2400
            RESULT[$step]=$? ;;
        gametest|gametest-mic)
            # gametest-mic: the MIC set (re-run install.sh mic first); Distant Horizons is left out of
            # the GameTest server there (mic.gametest-excluded), which farterrainhasdistanthorizonswhenexpected logs.
            extra=(); [ "$step" = gametest-mic ] && extra=(-PwithMic)
            ./gametest.sh "${extra[@]}" -PgametestOnly=farterrainmatchesgeneratedchunks,farterrainmatchesgeneratedchunksacrosstheseam,farterrainwrapsandfillsthecolumn,farterrainsectionsarecheapandthreadsafe,farterrainhasdistanthorizonswhenexpected
            RESULT[$step]=$?
            grep -a -e "\[gametest\] farTerrain" -e "Distant Horizons expected" -e "left out of this dev run" neoforge/build/gametest.log || true ;;
        *) echo "p3-04-dh.sh: unknown step '$step'" >&2; exit 2 ;;
    esac
done
status=0
for step in "${STEPS[@]}"; do
    echo "p3-04-dh.sh: $step -> exit ${RESULT[$step]}"
    [ "${RESULT[$step]}" = 0 ] || status=1
done
for p in dh dh-next mic; do
    [ -f "smoke/$p/report.md" ] && { echo "== smoke/$p/report.md"; grep -a -e "^\*\*Result" -e "dh:" -e "far terrain" "smoke/$p/report.md"; }
done
exit $status

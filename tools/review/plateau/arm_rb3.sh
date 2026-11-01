#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The runoff law's arms after its second re-review (VM, cached pool folders only): rb3 (the branch's default, the law
# on, the climate stage's twelve months every step) and rb3-off (surface.runoff_budget=false: the months' and the thaw's
# corrections alone), both on the fixed head after origin/master e65259fe, against base35-s4 (base35's own runs of
# seeds 1 to 4). rb2 and rb2-off, on e01f6bfa's code, are read beside them when their folders are given.
#   arm_rb3.sh <rb3 argkey folder> <rb3-off argkey folder> [<rb2 folder> <rb2-off folder>]
#     -> /data/ban/deeptime-runs/plateau/recal.txt, wear-rb3.txt, wear-rb3off.txt, wear-base35s4.txt (and wear-rb2*.txt)
# Run detached: tools/remote/mac.sh -- tools/remote/detach.sh start rb3-read --kind light tools/review/plateau/arm_rb3.sh A B
set -uo pipefail
cd "$(dirname "$0")/../../.."
RB3=${1:?rb3 folder}
OFF=${2:?rb3-off folder}
BASE=/data/ban/deeptime-runs/pool/4dd28019a295/earthlike_quick_16k-1000/cd665cef
S4=/data/ban/deeptime-runs/plateau/base35-s4
mkdir -p "$S4"
for start in supercontinent scattered_continents; do
    for seed in 1 2 3 4; do
        ln -sfn "$BASE/$start-$seed" "$S4/$start-$seed"
    done
done
arms=(rb3="$RB3" rb3off="$OFF")
if [ $# -ge 4 ]; then
    arms+=(rb2="$3" rb2off="$4")
fi
bash tools/review/plateau/recal.sh "${arms[@]}" base35s4="$S4"
bash tools/review/plateau/wear.sh "${arms[@]}" base35s4="$S4"

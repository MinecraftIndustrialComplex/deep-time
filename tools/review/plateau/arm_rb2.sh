#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The runoff law's arms after its re-review (VM, cached pool folders only): rb2 (the branch's default, the law on) and
# rb2-off (surface.runoff_budget=false: the months' and the thaw's corrections alone), both on the merged head, against
# base35-s4: base35's own runs of seeds 1 to 4 only (the re-review: the earlier reads set 8-run arms against base35's 24).
#   arm_rb2.sh <rb2 argkey folder> <rb2-off argkey folder>
#     -> /data/ban/deeptime-runs/plateau/recal.txt, wear-rb2.txt, wear-rb2off.txt, wear-base35s4.txt
# Run detached: tools/remote/mac.sh -- tools/remote/detach.sh start rb2-read --kind light tools/review/plateau/arm_rb2.sh A B
set -uo pipefail
cd "$(dirname "$0")/../../.."
RB2=${1:?rb2 folder}
OFF=${2:?rb2-off folder}
BASE=/data/ban/deeptime-runs/pool/4dd28019a295/earthlike_quick_16k-1000/cd665cef
S4=/data/ban/deeptime-runs/plateau/base35-s4
mkdir -p "$S4"
for start in supercontinent scattered_continents; do
    for seed in 1 2 3 4; do
        ln -sfn "$BASE/$start-$seed" "$S4/$start-$seed"
    done
done
bash tools/review/plateau/recal.sh rb2="$RB2" rb2off="$OFF" base35s4="$S4"
bash tools/review/plateau/wear.sh rb2="$RB2" rb2off="$OFF" base35s4="$S4"

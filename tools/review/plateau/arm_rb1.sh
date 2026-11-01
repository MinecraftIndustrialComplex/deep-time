#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The runoff law's arms after its review fixes (VM, cached pool folders only): rb-months (the months' correction, the
# branch's default) and rb1 (surface.runoff_budget on) against base35 (key 4dd28019a295; its 24 runs, seeds 1 to 12).
#   arm_rb1.sh   -> /data/ban/deeptime-runs/plateau/recal.txt, wear-rbm.txt, wear-rb1.txt, wear-base35.txt
# Run detached: tools/remote/mac.sh -- tools/remote/detach.sh start rb1-read --kind light tools/review/plateau/arm_rb1.sh
set -uo pipefail
cd "$(dirname "$0")/../../.."
POOL=/data/ban/deeptime-runs/pool
BASE=$POOL/4dd28019a295/earthlike_quick_16k-1000/cd665cef
RBM=$POOL/c9be4b8a512c/earthlike_quick_16k-1000/cd665cef
RB1=$POOL/c9be4b8a512c/earthlike_quick_16k-1000/0f65c40e
bash tools/review/plateau/recal.sh rb1="$RB1" rbm="$RBM" base35="$BASE"
bash tools/review/plateau/wear.sh rb1="$RB1" rbm="$RBM" base35="$BASE"

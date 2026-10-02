#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# tools/review/flexure-compare.sh over seeds × starts, detached-friendly: one folder per seed and
# start under S, all output in S/ensemble.log. On the test VM:
#
#   S=/Volumes/Graphide/ban/deeptime-runs/<task>/ens LEN=200 SEEDS="1 2" \
#     STARTS="scattered_continents supercontinent" tools/review/flexure-ensemble.sh label=bin[:sets] ...
#
# Each seed and start runs its variants J (default 3) at once.
set -euo pipefail
S=${S:?scratch folder}; SEEDS=${SEEDS:-1 2}; STARTS=${STARTS:-scattered_continents supercontinent}
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$S"
for start in $STARTS; do
    for seed in $SEEDS; do
        echo "== $start $seed" >> "$S/ensemble.log"
        S="$S/$start-$seed" START=$start SEED=$seed "$here/flexure-compare.sh" "$@" >> "$S/ensemble.log" 2>&1 || true
        rm -f "$S/$start-$seed"/*.dtcp
    done
done
echo "== done" >> "$S/ensemble.log"

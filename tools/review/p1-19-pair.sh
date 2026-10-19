#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-19: two builds' `deeptime bench` runs side by side on the test VM's two CPU sets (set A
# 0,16,2,18,4,20,6,22 and set B 8,24,10,26,12,28,14,30, one logical CPU per physical core, each
# set across both L3 caches), then swapped, so each build runs once on each set; each pair is one
# admitted job of twice the threads, and each pair's worlds are compared (p1-17-cmp.py). Every run
# is a tools/review/bench-quiet.py run (log.txt, load.tsv, summary.txt, bench.json, work/).
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start <job> --kind light tools/review/p1-19-pair.sh \
#       <binA> <labelA> <binB> <labelB> <out root> <preset> <start> <myr> [threads] [pairs]
#
#   pairs   ab (A on set A, B on set B), ba (swapped) or ab,ba (the default)
#
# Writes <out root>/<pair>-<label>/ per run and <out root>/<pair>-cmp.txt per pair; the worlds stay
# for the caller to compare further or delete.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
[ $# -ge 8 ] || { sed -n 4,16p "$0"; exit 2; }
binA=$1; labA=$2; binB=$3; labB=$4; root=$5; preset=$6; start=$7; myr=$8
t=${9:-8}; pairs=${10:-ab,ba}
SETA=0,16,2,18,4,20,6,22
SETB=8,24,10,26,12,28,14,30
mkdir -p "$root"
for p in ${pairs//,/ }; do
    if [ "$p" = ab ]; then ca=$SETA; cb=$SETB; else ca=$SETB; cb=$SETA; fi
    oa="$root/$p-$labA"; ob="$root/$p-$labB"
    rm -rf "$oa" "$ob"
    deeptime-admit run --kind "sim:$preset:$((2 * t))t" --label "p119-pair-$p" -- bash -c "
        python3 '$HERE/bench-quiet.py' --cpus $ca --out '$oa' '$binA' $preset $start $myr $t &
        python3 '$HERE/bench-quiet.py' --cpus $cb --out '$ob' '$binB' $preset $start $myr $t &
        wait"
    python3 "$HERE/p1-17-cmp.py" "$oa/work/t$t" "$ob/work/t$t" > "$root/$p-cmp.txt" 2>&1
    echo "cmp exit $?" >> "$root/$p-cmp.txt"
    tail -n 3 "$root/$p-cmp.txt"
done
echo PAIRS_DONE

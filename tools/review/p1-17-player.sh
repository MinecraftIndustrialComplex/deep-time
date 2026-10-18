#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-17, the player's PC: two builds on P1-24's mid6 profile (6 cores / 12 CPUs, 11 Rayon threads,
# the client's 4 GiB cap; tools/review/p1-24-profile.sh) side by side, A pinned to the VM's CPUs
# 0-11 and B to 12-23, then their worlds compared (tools/review/p1-17-cmp.py).
#
#   tools/review/p1-17-player.sh <binA> <labelA> <binB> <labelB> <out root> <preset:start:length[:mem]>...
#
# mem is the scope's memory (default 6G; the full preset needs 16G). Each pair is one admitted job
# (sim:<preset>:22t) so both start together; start the script itself unadmitted (detach.sh start
# <name> --kind light ...). Writes <out root>/pp-<label>-<preset>-<start>-<length>/result.json.
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
if [ "${1:-}" = _pair ]; then
    shift
    binA=$1; labA=$2; binB=$3; labB=$4; root=$5; p=$6; s=$7; l=$8; m=$9
    P124_KEEP=1 P124_MEM=$m P124_BIN=$binA "$REPO/tools/review/p1-24-profile.sh" mid6 "$p" "$s" "$l" \
        "$root/pp-$labA-$p-$s-$l" 0-11 > "$root/pp-$labA-$p-$s-$l.txt" 2>&1 &
    P124_KEEP=1 P124_MEM=$m P124_BIN=$binB "$REPO/tools/review/p1-24-profile.sh" mid6 "$p" "$s" "$l" \
        "$root/pp-$labB-$p-$s-$l" 12-23 > "$root/pp-$labB-$p-$s-$l.txt" 2>&1 &
    wait
    exit 0
fi
[ $# -ge 6 ] || { sed -n 4,13p "$0"; exit 2; }
binA=$1; labA=$2; binB=$3; labB=$4; root=$5; shift 5
mkdir -p "$root"
for spec in "$@"; do
    IFS=: read -r p s l m <<< "$spec"
    m=${m:-6G}
    echo "== $(date -u +%T) $p $s $l $m"
    deeptime-admit run --kind "sim:$p:22t" --label "p1-17-pp-$labA-$labB-$p-$s-$l" -- \
        "$0" _pair "$binA" "$labA" "$binB" "$labB" "$root" "$p" "$s" "$l" "$m"
    for x in $labA $labB; do tail -1 "$root/pp-$x-$p-$s-$l.txt" | cut -c1-400; done
    python3 "$REPO/tools/review/p1-17-cmp.py" "$root/pp-$labA-$p-$s-$l/world" "$root/pp-$labB-$p-$s-$l/world" 2>&1 | tail -6
    rm -rf "$root/pp-$labA-$p-$s-$l/world" "$root/pp-$labB-$p-$s-$l/world"
done
echo "== $(date -u +%T) ALL DONE"

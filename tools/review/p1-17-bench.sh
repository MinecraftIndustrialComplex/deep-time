#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-17, the benchmark gate (V1): `deeptime bench` runs on the test VM, two binaries side by side so
# both see the same load (the VM is shared; the ratio is the reliable number), with /usr/bin/time -v
# and a byte-for-byte comparison of the two worlds (tools/review/p1-17-cmp.py).
#
#   tools/review/p1-17-bench.sh pair  <binA> <labelA> <binB> <labelB> <out root> <spec>...
#       for each spec in turn: A and B at the same time, then compare their worlds
#   tools/review/p1-17-bench.sh scale <bin> <label> <out root> <spec>...
#       every spec at the same time (thread-count scaling)
#
#   spec      <preset>:<start>:<length_myr>:<threads>, e.g. earthlike_quick_16k:supercontinent:1000:8
#
# Each run is one job of the VM's admission tool (sim:<preset>:<threads>t) and writes
# <out root>/<label>-<preset>-<start>-<len>-t<threads>/{log.txt,bench.json,manifest.json}; the world
# itself is deleted once compared. tools/review/p1-17-table.py turns the logs into the tables of
# docs/design/p1-17-benchmark-gate.md.
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"

one() { # one <bin> <label> <root> <spec>: one admitted bench run; prints its out dir
    local bin=$1 label=$2 root=$3 p s l t
    IFS=: read -r p s l t <<< "$4"
    local out="$root/$label-$p-$s-$l-t$t"
    rm -rf "$out"; mkdir -p "$out"
    deeptime-admit run --kind "sim:$p:${t}t" --label "p1-17-$label-$p-$s-$l-t$t" -- \
        env RAYON_NUM_THREADS="$t" /usr/bin/time -v "$bin" bench --preset "$p" --start "$s" \
        --length-myr "$l" --threads "$t" --out "$out/bench.json" --work "$out/work" > "$out/log.txt" 2>&1
    echo "exit $?" >> "$out/log.txt"
    cp "$out/work/t$t/manifest.json" "$out/manifest.json" 2>/dev/null
}
summary() { grep -h "total_ms\|Maximum resident\|^exit" "$1/log.txt" | cut -c1-220; }

verb=${1:-}
case "$verb" in
pair)
    [ $# -ge 7 ] || { sed -n 4,19p "$0"; exit 2; }
    binA=$2; labA=$3; binB=$4; labB=$5; root=$6; shift 6
    mkdir -p "$root"
    for spec in "$@"; do
        IFS=: read -r p s l t <<< "$spec"
        echo "== $(date -u +%T) $spec"
        one "$binA" "$labA" "$root" "$spec" &
        one "$binB" "$labB" "$root" "$spec" &
        wait
        a="$root/$labA-$p-$s-$l-t$t"; b="$root/$labB-$p-$s-$l-t$t"
        echo "== $(date -u +%T) done $spec"
        summary "$a"; summary "$b"
        python3 "$REPO/tools/review/p1-17-cmp.py" "$a/work/t$t" "$b/work/t$t" 2>&1 | tail -8 | tee "$b/cmp.txt"
        rm -rf "$a/work" "$b/work"
    done
    ;;
scale)
    [ $# -ge 5 ] || { sed -n 4,19p "$0"; exit 2; }
    bin=$2; lab=$3; root=$4; shift 4
    mkdir -p "$root"
    for spec in "$@"; do
        ( one "$bin" "$lab" "$root" "$spec"; IFS=: read -r p s l t <<< "$spec"
          rm -rf "$root/$lab-$p-$s-$l-t$t/work"; echo "== $(date -u +%T) done $spec"
          summary "$root/$lab-$p-$s-$l-t$t" ) &
    done
    wait
    ;;
*) sed -n 4,19p "$0"; exit 2 ;;
esac
echo "== $(date -u +%T) ALL DONE"

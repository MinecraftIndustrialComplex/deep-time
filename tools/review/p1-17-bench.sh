#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-17, the benchmark gate (V1): `deeptime bench` runs on the test VM, two binaries side by side so
# both see the same load (the VM is shared; the ratio is the reliable number), with /usr/bin/time -v
# and a byte-for-byte comparison of the two worlds (tools/review/p1-17-cmp.py).
#
#   tools/review/p1-17-bench.sh seq   <binA> <labelA> <binB> <labelB> <out root> <spec>...
#       for each spec in turn: A, then B, one run at a time, then compare their worlds (the
#       coordinator's rule of 2026-10-16: one bench at a time on the shared VM, A and B
#       interleaved, not side by side; list a spec twice to keep its second, warm-cache runs)
#   tools/review/p1-17-bench.sh pair  <binA> <labelA> <binB> <labelB> <out root> <spec>...
#       for each spec in turn: A and B at the same time, then compare their worlds (two runs at
#       once: not on the shared VM any more, see seq)
#   tools/review/p1-17-bench.sh scale <bin> <label> <out root> <spec>...
#       every spec at the same time (thread-count scaling)
#
#   spec      <preset>:<start>:<length_myr>:<threads>[:<mem cap bytes>], e.g.
#             earthlike_quick_16k:supercontinent:1000:8 (the cap defaults to bench's, 4 GiB)
#
# Each seq or scale run is one job of the VM's admission tool (sim:<preset>:<threads>t); each pair
# is one job of twice the threads, so both start together. Start the script itself unadmitted
# (`detach.sh start <name> --kind light ...`): inside an admitted job the runs would not queue. Each
# run writes
# <out root>/<label>-<preset>-<start>-<len>-t<threads>/{log.txt,bench.json,manifest.json}; the world
# itself is deleted once compared. tools/review/p1-17-table.py turns the logs into the tables of
# docs/design/p1-17-benchmark-gate.md.
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"

one() { # one <bin> <label> <root> <spec>: one bench run (admitted by the caller); its out dir
    local bin=$1 label=$2 root=$3 p s l t cap
    IFS=: read -r p s l t cap <<< "$4"
    local out="$root/$label-$p-$s-$l-t$t"
    rm -rf "$out"; mkdir -p "$out"
    env RAYON_NUM_THREADS="$t" /usr/bin/time -v "$bin" bench --preset "$p" --start "$s" \
        --length-myr "$l" --threads "$t" ${cap:+--mem-cap "$cap"} --out "$out/bench.json" \
        --work "$out/work" > "$out/log.txt" 2>&1
    echo "exit $?" >> "$out/log.txt"
    cp "$out/work/t$t/manifest.json" "$out/manifest.json" 2>/dev/null
}
summary() { grep -h "total_ms\|Maximum resident\|^exit" "$1/log.txt" | cut -c1-220; }

verb=${1:-}
case "$verb" in
_pair) # inside the admitted job: the two runs at once
    one "$2" "$3" "$6" "$7" &
    one "$4" "$5" "$6" "$7" &
    wait
    exit 0
    ;;
_one) # inside the admitted job: one run
    one "$2" "$3" "$4" "$5"
    exit 0
    ;;
seq)
    [ $# -ge 7 ] || { sed -n 4,26p "$0"; exit 2; }
    binA=$2; labA=$3; binB=$4; labB=$5; root=$6; shift 6
    mkdir -p "$root"
    for spec in "$@"; do
        IFS=: read -r p s l t _ <<< "$spec"
        for side in A B; do
            if [ "$side" = A ]; then bin=$binA; lab=$labA; else bin=$binB; lab=$labB; fi
            echo "== $(date -u +%T) $lab $spec"
            deeptime-admit run --kind "sim:$p:${t}t" --label "p1-17-$lab-$p-$s-$l-t$t" -- \
                "$0" _one "$bin" "$lab" "$root" "$spec"
        done
        a="$root/$labA-$p-$s-$l-t$t"; b="$root/$labB-$p-$s-$l-t$t"
        echo "== $(date -u +%T) done $spec"
        summary "$a"; summary "$b"
        python3 "$REPO/tools/review/p1-17-cmp.py" "$a/work/t$t" "$b/work/t$t" 2>&1 | tail -8 | tee "$b/cmp.txt"
        rm -rf "$a/work" "$b/work"
    done
    ;;
pair)
    [ $# -ge 7 ] || { sed -n 4,26p "$0"; exit 2; }
    binA=$2; labA=$3; binB=$4; labB=$5; root=$6; shift 6
    mkdir -p "$root"
    for spec in "$@"; do
        IFS=: read -r p s l t _ <<< "$spec"
        echo "== $(date -u +%T) $spec"
        # One admitted job for the pair (twice the threads), so A and B start together.
        deeptime-admit run --kind "sim:$p:$((2 * t))t" --label "p1-17-$labA-$labB-$p-$s-$l-t$t" -- \
            "$0" _pair "$binA" "$labA" "$binB" "$labB" "$root" "$spec"
        a="$root/$labA-$p-$s-$l-t$t"; b="$root/$labB-$p-$s-$l-t$t"
        echo "== $(date -u +%T) done $spec"
        summary "$a"; summary "$b"
        python3 "$REPO/tools/review/p1-17-cmp.py" "$a/work/t$t" "$b/work/t$t" 2>&1 | tail -8 | tee "$b/cmp.txt"
        rm -rf "$a/work" "$b/work"
    done
    ;;
scale)
    [ $# -ge 5 ] || { sed -n 4,26p "$0"; exit 2; }
    bin=$2; lab=$3; root=$4; shift 4
    mkdir -p "$root"
    for spec in "$@"; do
        ( IFS=: read -r p s l t _ <<< "$spec"
          deeptime-admit run --kind "sim:$p:${t}t" --label "p1-17-$lab-$p-$s-$l-t$t" -- \
              "$0" _one "$bin" "$lab" "$root" "$spec"
          rm -rf "$root/$lab-$p-$s-$l-t$t/work"; echo "== $(date -u +%T) done $spec"
          summary "$root/$lab-$p-$s-$l-t$t" ) &
    done
    wait
    ;;
*) sed -n 4,21p "$0"; exit 2 ;;
esac
echo "== $(date -u +%T) ALL DONE"

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Build and run the sphere-grid benchmark on the test VM (docs/research/notes/sphere-native-grids.md).
# Results go to review/out/sphere-grid-bench/ (never committed). From the laptop:
#   tools/remote/mac.sh --kind cargo:build -- tools/research/sphere-grid-bench/run.sh build
#   tools/remote/mac.sh --no-sync --kind sim:quick:8t -- tools/research/sphere-grid-bench/run.sh run
#   tools/remote/mac.sh --no-sync --pull review/out/sphere-grid-bench -- true
# `run` runs L1 (333,824 cells), L2 (1,335,296) and L3-like (5,341,184) sizes, generic x86-64 and
# target-cpu=native builds, at 1 and 8 threads.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
out="$repo/review/out/sphere-grid-bench"
mkdir -p "$out"
cd "$here"

build() {
    nix develop "$repo" -c cargo build --release --target-dir "$out/target-generic"
    RUSTFLAGS="-C target-cpu=native" nix develop "$repo" -c cargo build --release --target-dir "$out/target-native"
}

run() {
    lscpu > "$out/lscpu.txt" 2>&1 || true
    for n in 333824 1335296 5341184; do
        for flav in generic native; do
            echo "run $n $flav $(date +%T)"
            "$out/target-$flav/release/sphere-grid-bench" "$n" 8 40 > "$out/$flav-$n.md" 2> "$out/$flav-$n.log"
        done
    done
    echo "done $(date +%T)"
}

# repeat: the generic build again, several times per size (the VM is shared, so single runs are
# noisy); summarize.py takes the median over every run of a size.
repeat() {
    for k in 1 2 3; do
        for n in 333824 1335296; do
            echo "repeat $k $n $(date +%T)"
            "$out/target-generic/release/sphere-grid-bench" "$n" 8 40 > "$out/generic-$n-r$k.md" 2> /dev/null
        done
    done
    echo "repeat 5341184 $(date +%T)"
    "$out/target-generic/release/sphere-grid-bench" 5341184 8 40 > "$out/generic-5341184-r1.md" 2> /dev/null
    echo "done $(date +%T)"
}

case "${1:-all}" in
    build) build ;;
    run) run ;;
    repeat) repeat ;;
    all) build; run ;;
    *) echo "usage: run.sh build|run|all" >&2; exit 2 ;;
esac

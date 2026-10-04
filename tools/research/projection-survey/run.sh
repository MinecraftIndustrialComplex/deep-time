#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Run this folder's tools on the test VM (numpy/scipy/mpmath from nixpkgs; rustc from the dev shell).
# Results go to review/out/projection-survey/ (never synced over, never committed):
#   tools/remote/mac.sh -- tools/remote/detach.sh start projsurvey --kind shell:python \
#       tools/research/projection-survey/run.sh all
#   tools/remote/mac.sh --no-sync -- tools/remote/detach.sh wait projsurvey
#   tools/remote/mac.sh --no-sync --pull review/out/projection-survey -- true
# or one tool:  run.sh one <tag> <script.py> [args...]
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
out="$repo/review/out/projection-survey"
mkdir -p "$out"
PY='(builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux.python3.withPackages (p: [p.numpy p.scipy p.mpmath])'

one() {
    local tag="$1"; shift
    local script="$1"; shift
    echo "start $tag $(date +%T)"
    if nix shell --impure --expr "$PY" -c python3 "$here/$script" "$@" > "$out/$tag.json" 2> "$out/$tag.log"; then
        echo "done $tag $(date +%T)"
    else
        echo "FAILED $tag"; tail -20 "$out/$tag.log"
    fi
}

bench() {
    rustc -C opt-level=3 -C target-cpu=native -o "$out/projbench" "$here/bench.rs" && "$out/projbench" > "$out/bench.txt" 2>&1
    echo "bench: $(wc -l < "$out/bench.txt") lines"
}

case "${1:-all}" in
    one) shift; one "$@" ;;
    bench) bench ;;
    all)
        one conf conformal.py
        one survey survey.py n=300 lambert guyou collignon petroff gringorten healpix qsc rosca cobe gnomonic eac s2quad nowell
        one moser-cube moser.py cube 512
        one moser-peirce moser.py peirce 512
        bench
        ;;
    *) shift; for t in "$@"; do :; done; echo "unknown verb" ;;
esac

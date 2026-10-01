#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P2-01: the numbers and pictures of a `p2-01-planet.sh world` run, gathered into
# review/out/p2-01/ (gitignored) for pulling back to the laptop.
#
#   tools/remote/mac.sh --pull review/out/p2-01 -- tools/review/p2-01-report.sh <name> [<name-without-relief>]
set -euo pipefail

RUNS=/Volumes/Graphide/ban/deeptime-runs/p2-01
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
OUT=review/out/p2-01
mkdir -p "$OUT"
PY=(nix shell --impure --expr
    '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy])'
    -c python3)
"${PY[@]}" tools/review/p2-01-report.py "$RUNS" "$@" | tee "$OUT/report.txt"
rm -rf "$OUT/sections"
cp -r "$RUNS/$1-sections" "$OUT/sections"
cp "$RUNS/$1-strata.json" "$OUT/strata.json"

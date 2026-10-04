#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# P1-24: tools/review/p1-24-views.py over the matrix's kept worlds (tools/review/p1-24-matrix.sh),
# in the Python with numpy, matplotlib and lz4.
#   tools/review/p1-24-views.sh <runs dir> <out dir> label=run-name ...
set -euo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
runs="$1"; out="$2"; shift 2
args=()
for spec in "$@"; do args+=("${spec%%=*}=$runs/${spec#*=}/world"); done
nix shell --impure --expr '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.matplotlib p.lz4])' \
    -c python3 "$REPO/tools/review/p1-24-views.py" --bin "$REPO/target/release/deeptime" --out "$out" "${args[@]}"

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-10: how the orogen share of the continents (Earth ~0.17: orogens 12 % + arcs 5 %, USGS map) moves
# with the preset's `provinces` thresholds. Runs `deeptime provinces` over the last checkpoint of a run
# (`deeptime run --checkpoints`), once per setting, and prints the coherence line's class shares
# (continental area, before the bake's minimum-size absorption). On the test VM via tools/remote/mac.sh.
#
#   tools/review/p1-10-orogen.sh <checkpoint.dtcp>
set -euo pipefail
CK=${1:?checkpoint}
BIN=${BIN:-target/release/deeptime}
OUT=$(mktemp -d)
trap 'rm -rf "$OUT"' EXIT
run() {
    printf '%-34s ' "$*"
    "$BIN" provinces --in "$CK" --out "$OUT/p.png" "$@" \
        | sed -nE '1{s/^provinces //; s/ edge_density.*orogen_by/ orogen_by/; p}'
}
run
for age in 400 300 200 100 50; do run --orogen-max-age-myr "$age"; done
for h in 2000 2500 3000; do run --andean-min-elevation-m "$h"; done
for h in 25 10; do run --andean-max-age-myr "$h"; done
run --orogen-max-age-myr 200 --andean-max-age-myr 25 --andean-min-elevation-m 2500

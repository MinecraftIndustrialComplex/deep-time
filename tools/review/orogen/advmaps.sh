#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The orogen lane (VM, cached pool folders only): the coast-with-high-ground audit's map statistics
# (the advisor's adv_maps.py, /data/ban/deeptime-cache/advisor/, read-only: the hypsometry, the high
# ground's distance to the sea, what the high coasts stand on) over one arm's runs.
#   advmaps.sh <tag> <pool argkey folder>     -> /data/ban/deeptime-runs/orogen/adv_maps-<tag>.txt
set -uo pipefail
cd "$(dirname "$0")/../../.."
tag=$1 folder=$2
OUT=/data/ban/deeptime-runs/orogen
mkdir -p "$OUT"
nice -n 19 deeptime-admit run --kind light -- nix shell --impure --expr '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy p.h5py])' -c python3 -I /data/ban/deeptime-cache/advisor/adv_maps.py "$PWD/tools/review/m5diag" "$folder" "$OUT/adv_maps-$tag.jsonl" > "$OUT/adv_maps-$tag.txt" 2> "$OUT/adv_maps-$tag.log"
echo "exit $?" >> "$OUT/adv_maps-$tag.log"
sed -n '/Hypsometry/,/inland/p;/make-up/,/hi_inland/p' "$OUT/adv_maps-$tag.txt"
grep -E 'model +0\.0:' "$OUT/adv_maps-$tag.txt"

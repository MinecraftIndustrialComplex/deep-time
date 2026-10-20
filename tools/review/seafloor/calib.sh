#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Sea-floor spikes study (2026-10-20): the young and amplify stages again from run-s1's deposits
# checkpoint with a sea-floor critical slope, baked, then the whole world's heightmap and the
# survey at the L3 baseline and at ETOPO 60s's.
#   calib.sh <tag> <young.sea_floor_slope_deg> <amplify.submarine_slope>
set -euo pipefail
tag=$1 deg=$2 sub=$3
S=/data/ban/deeptime-runs/seafloor
R=/data/ban/mic/mods/agent-a178044f2a1370861
B=$S/deeptime-fix
W=$S/cal-$tag
PY='(builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux.python3.withPackages (p: [p.numpy p.scipy p.matplotlib])'
rm -rf "$W"
mkdir -p "$W"
"$B" stage young --threads 8 --in $S/run-s1/checkpoints/6-deposits.dtcp --out "$W/y.dtcp" --work "$W/yw" \
    --set young.sea_floor_slope_deg="$deg" 2>&1 | grep -E "young_final|error" || true
"$B" stage amplify --threads 8 --in "$W/y.dtcp" --out "$W/a.dtcp" --work "$W/aw" --bake \
    --set amplify.submarine_slope="$sub" 2>&1 | grep -E "amplify_final|bake|error" | cut -c1-400 || true
rm -f "$W/y.dtcp"
world=$(dirname "$(find "$W/aw" -name manifest.json | head -1)")
echo "world: $world"
"$B" heightmap --world-data "$world" --box=-6656,-3328,6656,3328 --step 2 \
    --layers meta/crust_age_myr,meta/province,meta/plate_id --scale --out "$W/hm.npy" 2>&1 | tail -2
for k in 1 3; do
    nix shell --impure --expr "$PY" -c python3 $R/tools/review/seafloor/survey.py "$W/hm.npy" "$W/survey-c$k" --coarsen $k \
        | python3 -c "import json,sys; t=sys.stdin.read(); i=t.index('{'); d=json.JSONDecoder().raw_decode(t[i:])[0]; print('c$k', json.dumps({k: d[k] for k in ('real_deg_pct_area','share_real_over_area','tile_rms_H_pct_area','game_deg_pct','share_game_over_45','share_game_over_45_area','sea_share','shelf_share_of_sea','mean_depth_m','hadal_share_of_sea','max_depth_m')}))"
done

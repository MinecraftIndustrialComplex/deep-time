#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# P1-27: margin-share experiments (plates only), one line per run: tools/review/p1-27-exp.sh <name>…
# Each name is a label from the table below.
set -e
R=/Volumes/Graphide/ban/deeptime-runs/p1-27/exp
mkdir -p $R
KEEP='^(supercontinent|scattered_continents|seed=|active_mean|passive_mean|regime_active_mean|passive_floor_age_p50_mean|fragment_coast_mean|final_coast_km|ocean_age_p90_myr|ocean_age_p50_myr|ocean_younger_250|plates_final|collisions|sutures|rifts|reversals|initiations)'
run() {
    label=$1; shift
    tools/review/p1-27-margins.sh $R $label -- "$@" | paste - - | while read -r line; do
        echo "$label $(echo "$line" | tr ' \t' '\n\n' | grep -E "$KEEP" | tr '\n' ' ')"
    done
}
for name in "$@"; do
    case "$name" in
        base) run base ;;
        init0) run init0 --set tectonics.subduction_initiation_rate=0 ;;
        jitter0) run jitter0 --set tectonics.motion_jitter_per_sqrt_myr=0 ;;
        rift_half) run rift_half --set tectonics.rift_lambda0=0.006 ;;
        noslab) run noslab --set tectonics.slab_pull_rad_per_1000km_myr=0 --set tectonics.slab_pull_speed_mm_yr=45 ;;
        slowbase) run slowbase --set 'tectonics.plate_speed_mm_yr=[5,25]' ;;
        contslow) run contslow --set tectonics.continental_speed_factor=0.2 ;;
        nosuture) run nosuture --set tectonics.collision_suture_frac=1000000 ;;
        fewplates) run fewplates --set tectonics.max_plates=20 --set tectonics.rift_ocean_weight=0.02 ;;
        # label:path=value,path=value — any other experiment.
        *:*) label=${name%%:*}; sets=(); IFS=',' read -ra kv <<< "${name#*:}"
             for x in "${kv[@]}"; do sets+=(--set "$x"); done
             run "$label" "${sets[@]}" ;;
        *) run "$name" ;;
    esac
done

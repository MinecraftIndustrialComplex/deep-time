#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-27: an ensemble of coupled-pipeline runs per variant (the plate model is chaotic, so one run
# says little), with the mean and range of the margin, arc, province, hypsometry and climate
# figures over the runs. On the Mac via tools/remote/mac.sh.
#
#   tools/review/p1-27-ensemble.sh <length Myr> "<seeds>" "<starts>" label[:path=value,...] ...
#
# A label `before` runs $BEFORE (another build, default the before-src tree) instead.
set -euo pipefail
LEN=$1; SEEDS=$2; STARTS=$3; shift 3
R=/Volumes/Graphide/ban/deeptime-runs/p1-27/ens
BEFORE=${BEFORE:-/Volumes/Graphide/ban/deeptime-runs/p1-27/before-src/target/release/deeptime}
mkdir -p "$R"
cargo build --release -q -p deeptime-cli
for v in "$@"; do
    label=${v%%:*}; sets=()
    if [ "$label" != "$v" ]; then
        IFS=',' read -ra kv <<< "${v#*:}"
        for x in "${kv[@]}"; do sets+=(--set "$x"); done
    fi
    for start in $STARTS; do
        for seed in $SEEDS; do
            if [ "$label" = before ]; then
                BIN="$BEFORE" tools/review/p1-27-pipeline.sh "$R" "$label" "$start" "$seed" "$LEN" > /dev/null
            else
                tools/review/p1-27-pipeline.sh "$R" "$label" "$start" "$seed" "$LEN" -- "${sets[@]}" > /dev/null
            fi
        done
    done
    python3 - "$R" "$label" "$SEEDS" "$STARTS" <<'EOF'
import re, sys
r, label, seeds, starts = sys.argv[1], sys.argv[2], sys.argv[3].split(), sys.argv[4].split()
keys = {
    'margins records': ['active_mean', 'passive_mean', 'regime_active_mean', 'passive_floor_age_p50_mean', 'fragment_coast_mean'],
    'provinces shield': ['shield', 'platform', 'orogen', 'basin', 'lip', 'extended', 'edge_density', 'speck_share'],
    'hypsometry': ['land_frac_area', 'land_z_p50_m', 'land_z_p90_m', 'land_above_2km', 'continental_frac', 'continental_dry'],
    'volcanoes_final': ['arc_base_p50_mean_m', 'arc_base_low_share_mean', 'active_strato', 'standing_strato'],
    'climate_final': ['glaciated_land_frac'],
    'plates_final': ['plates_final', 'ocean_age_p90_myr', 'ocean_younger_250', 'collisions', 'initiations'],
}
vals = {}
for start in starts:
    for seed in seeds:
        text = open(f"{r}/{label}-{start}-{seed}.txt").read()
        for tag, ks in keys.items():
            line = next((l for l in text.splitlines() if tag in l), '')
            for k in ks:
                m = re.search(r'(?:^|\s)' + re.escape(k) + r'=([-0-9.eE+]+)', line)
                if m:
                    vals.setdefault(k, []).append(float(m.group(1)))
        koppen = re.search(r'koppen_land=A([0-9.]+)/B([0-9.]+)/C([0-9.]+)/D([0-9.]+)/E([0-9.]+)', text)
        if koppen:
            for name, x in zip('ABCDE', koppen.groups()):
                vals.setdefault('koppen_' + name, []).append(float(x))
out = [f"{label} n={len(seeds) * len(starts)}"]
for k, xs in vals.items():
    out.append(f"{k}={sum(xs) / len(xs):.3g}[{min(xs):.3g},{max(xs):.3g}]")
print(' '.join(out), flush=True)
EOF
done

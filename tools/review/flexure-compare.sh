#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Runs one planet per binary-and-settings variant with a checkpoint, then measures the cell-scale
# artifacts of each (tools/review/flexure_artifacts.py). On the test VM via tools/remote/mac.sh:
#
#   S=/Volumes/Graphide/ban/deeptime-runs/<task> LEN=200 START=scattered_continents SEED=1 \
#     tools/review/flexure-compare.sh label=binary[:path=value;...] ...
#
# Runs at most J (default 3) at once, 2 threads each, niced. Keeps <S>/<label>.dtcp and .log for
# pictures (tools/review/flexure-views.sh); delete S when done.
set -euo pipefail
S=${S:?scratch folder}; LEN=${LEN:-200}; START=${START:-scattered_continents}; SEED=${SEED:-1}; J=${J:-3}
mkdir -p "$S"
run_one() {
    spec=$1
    label=${spec%%=*}; rest=${spec#*=}
    bin=${rest%%:*}; sets=""
    if [ "$bin" != "$rest" ]; then
        IFS=';' read -ra kv <<< "${rest#*:}"
        for x in "${kv[@]}"; do sets+=" --set $x"; done
    fi
    # shellcheck disable=SC2086
    nice -n 10 "$bin" stage volcanoes --preset earthlike_quick_16k --start "$START" --length-myr "$LEN" \
        --threads 2 --set "seed=$SEED" $sets --work "$S/$label.work" --out "$S/$label.dtcp" > "$S/$label.log" 2>&1 \
        || echo "FAILED $label" >&2
    rm -rf "$S/$label.work"
    "$bin" dump-fields --in "$S/$label.dtcp" --out "$S/$label.fields" \
        --fields surface/elevation_m,surface/flexure_m,surface/deflection_m,plates/plate_id > /dev/null
}
export -f run_one
export S LEN START SEED
printf '%s\n' "$@" | xargs -P "$J" -I{} bash -c 'run_one "$@"' _ {}
PY='(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy])'
for spec in "$@"; do
    label=${spec%%=*}
    sea=$(grep -o 'sea_level_m=[-0-9.]*' "$S/$label.log" | tail -1 | cut -d= -f2)
    secs=$(grep -o 'steps=[0-9]* seconds=[0-9.]*' "$S/$label.log" | head -1)
    echo "$label: $secs $(grep -o 'flexure_iterations_per_solve=[0-9.]*' "$S/$label.log" | head -1)"
    nix shell --impure --expr "$PY" -c python3 "$(dirname "$0")/flexure_artifacts.py" --label "$label" \
        --sea "${sea:-0}" "$S/$label.fields"
    rm -rf "$S/$label.fields"
done

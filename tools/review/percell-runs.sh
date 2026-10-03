#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-26 per-cell climate: climate-stage runs of a preset per variant (seeds × starts), their final
# fields dumped and measured (tools/review/percell_metrics.py, one JSON line each), then the work
# deleted. On the test VM via tools/remote/mac.sh, detached for long runs, e.g.
#
#   tools/review/percell-runs.sh 500 "1" "scattered_continents supercontinent" \
#       banded:climate.model=banded percell
#
# Environment: R (output folder, default /Volumes/Graphide/ban/deeptime-runs/p1-26/runs), PRESET
# (default earthlike_quick_16k), J (runs at once, default 2), T (threads per run, default 2),
# BIN (another build: skips the build), STAGE (default climate). A run whose .json exists is not
# run again.
set -euo pipefail
LEN=$1; SEEDS=$2; STARTS=$3; shift 3
R=${R:-/Volumes/Graphide/ban/deeptime-runs/p1-26/runs}
PRESET=${PRESET:-earthlike_quick_16k}
J=${J:-2}; T=${T:-2}; STAGE=${STAGE:-climate}
if [ -z "${BIN:-}" ]; then
    CARGO_BUILD_JOBS=8 cargo build --release -q -p deeptime-cli
    BIN=$PWD/target/release/deeptime
fi
METRICS=$PWD/tools/review/percell_metrics.py
PYENV='(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy])'
mkdir -p "$R"
jobs=()
for v in "$@"; do
    label=${v%%:*}
    sets=""
    if [ "$label" != "$v" ]; then
        IFS=';' read -ra kv <<< "${v#*:}"
        for x in "${kv[@]}"; do sets+=" --set $x"; done
    fi
    for start in $STARTS; do
        for seed in $SEEDS; do
            f="$R/$label-$start-$seed-$LEN"
            if [ -s "$f.json" ]; then continue; fi
            jobs+=("$f|$start|$seed|$sets|$label")
        done
    done
done
run_one() {
    IFS='|' read -r f start seed sets label <<< "$1"
    # shellcheck disable=SC2086
    if nice -n 10 "$BIN" stage "$STAGE" --preset "$PRESET" --start "$start" --length-myr "$LEN" \
        --threads "$T" --set "seed=$seed" --out "$f.dtcp" --work "$f.work" $sets > "$f.log" 2>&1; then
        "$BIN" dump-fields --in "$f.dtcp" --out "$f.dump" >> "$f.log" 2>&1
        nix shell --impure --expr "$PYENV" -c python3 "$METRICS" "$f.dump" "$label-$start-$seed" > "$f.json.part" && mv "$f.json.part" "$f.json"
    else
        echo "FAILED $f (see $f.log)" >&2
    fi
    if [ -z "${KEEP:-}" ]; then rm -rf "$f.work" "$f.dump" "$f.dtcp"; else rm -rf "$f.work"; fi
}
export -f run_one
export BIN PRESET LEN T STAGE METRICS KEEP PYENV
if [ ${#jobs[@]} -gt 0 ]; then
    printf '%s\n' "${jobs[@]}" | xargs -P "$J" -I{} bash -c 'run_one "$@"' _ {}
fi
cat "$R"/*-"$LEN".json 2>/dev/null || true

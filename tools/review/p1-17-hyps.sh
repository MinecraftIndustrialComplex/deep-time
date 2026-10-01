#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-17 hypsometry recalibration (2026-10-01): coupled-pipeline runs (plates, climate, surface,
# volcanoes, structure) of a preset per variant on seeds x starts, several at once, then one
# summary line per run and per variant (tools/review/p1-17-hyps.py). On the Mac via
# tools/remote/mac.sh, e.g.
#
#   tools/remote/mac.sh -- 'tools/review/p1-17-hyps.sh 500 "1 2 3" "supercontinent scattered_continents" \
#       base "k3:surface.k_spl=3e-9;surface.isostasy=0.85"'
#
#   tools/review/p1-17-hyps.sh <length Myr> "<seeds>" "<starts>" label[:path=value;path=value...] ...
#
# (sets are separated by ";" so a value may be a JSON array). Environment: R (output folder,
# default /Volumes/Graphide/ban/deeptime-runs/p1-17-hyps/runs),
# PRESET (default earthlike_quick_16k), J (runs at once, default 4), T (threads per run, default 2),
# STAGE (default volcanoes; an older build without the volcano stage: surface), BIN (another
# build: skips the build). A run whose output file already holds a finished summary
# is not run again (delete the file to redo it). Run folders are deleted after each run.
set -euo pipefail
LEN=$1; SEEDS=$2; STARTS=$3; shift 3
R=${R:-/Volumes/Graphide/ban/deeptime-runs/p1-17-hyps/runs}
PRESET=${PRESET:-earthlike_quick_16k}
J=${J:-4}; T=${T:-2}; STAGE=${STAGE:-volcanoes}
if [ -z "${BIN:-}" ]; then
    cargo build --release -q -p deeptime-cli
    BIN=$PWD/target/release/deeptime
fi
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
            f="$R/$label-$start-$seed-$LEN.txt"
            if [ -s "$f" ] && grep -q "hypsometry " "$f"; then continue; fi
            jobs+=("$f|$start|$seed|$sets")
        done
    done
done
run_one() {
    IFS='|' read -r f start seed sets <<< "$1"
    work="${f%.txt}.work"
    # shellcheck disable=SC2086
    "$BIN" stage "$STAGE" --preset "$PRESET" --start "$start" --length-myr "$LEN" --threads "$T" \
        --set "seed=$seed" --work "$work" $sets > "$f.part" 2>&1 \
        && mv "$f.part" "$f" || echo "FAILED $f (see $f.part)" >&2
    rm -rf "$work"
}
export -f run_one
export BIN PRESET LEN T STAGE
if [ ${#jobs[@]} -gt 0 ]; then
    printf '%s\n' "${jobs[@]}" | xargs -P "$J" -I{} bash -c 'run_one "$@"' _ {}
fi
labels=()
for v in "$@"; do labels+=("${v%%:*}"); done
python3 "$(dirname "$0")/p1-17-hyps.py" "$R" "$LEN" "$SEEDS" "$STARTS" "${labels[@]}"

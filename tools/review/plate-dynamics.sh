#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Plate dynamics of a run (docs/design/plate-collisions-and-welding.md): runs a preset per variant
# on seeds x starts with the measurement trace on, then prints tools/review/plate-dynamics.py's
# figures per run and per variant (largest-plate share, plate lifetimes and fates, land gained per
# step, the shape of what each collision lifted). On the test VM via tools/remote/mac.sh:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start pd tools/review/plate-dynamics.sh 1000 1 supercontinent base
#
#   tools/review/plate-dynamics.sh <length Myr> "<seeds>" "<starts>" label[:path=value;path=value...] ...
#
# (seeds and starts may also be separated by commas, which survive mac.sh's word joining.)
# The trace needs a binary built with tools/review/plate-dynamics-trace.patch applied (a throwaway
# instrument: `git apply` it, build, `git apply -R` it); the design proposes `deeptime plate-check`
# in its place. Environment: R (output folder), PRESET (default earthlike_quick_16k), STAGE
# (default volcanoes = the coupled pipeline up to the volcanoes; on a pipeline preset `plates`
# runs the same coupled group, so it is no faster), J (runs at once, default 4), T (threads per
# run, default 2), BIN (a built binary: skips the build). A 1000 Myr run takes about 18 minutes.
set -euo pipefail
LEN=$1; SEEDS=${2//,/ }; STARTS=${3//,/ }; shift 3
STAGE=${STAGE:-volcanoes}
R=${R:-/Volumes/Graphide/ban/deeptime-runs/plate-collisions-design/$STAGE}
PRESET=${PRESET:-earthlike_quick_16k}
J=${J:-4}; T=${T:-2}
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
            f="$R/$label-$start-$seed-$LEN"
            if [ -s "$f.txt" ] && [ -s "$f.events.json" ]; then continue; fi
            jobs+=("$f|$start|$seed|$sets")
        done
    done
done
run_one() {
    IFS='|' read -r f start seed sets <<< "$1"
    rm -rf "$f.trace" "$f.work" "$f.viewer"
    # shellcheck disable=SC2086
    DEEPTIME_PLATE_TRACE="$f.trace" nice -n 10 "$BIN" stage "$STAGE" --preset "$PRESET" --start "$start" \
        --length-myr "$LEN" --threads "$T" --set "seed=$seed" --work "$f.work" --viewer "$f.viewer" \
        --viewer-layers plates $sets > "$f.part" 2>&1 \
        && cp "$f.viewer/events.json" "$f.events.json" && mv "$f.part" "$f.txt" \
        || echo "FAILED $f (see $f.part)" >&2
    rm -rf "$f.work" "$f.viewer"
}
export -f run_one
export BIN PRESET LEN T STAGE
if [ ${#jobs[@]} -gt 0 ]; then
    printf '%s\n' "${jobs[@]}" | xargs -P "$J" -I{} bash -c 'run_one "$@"' _ {}
fi
for v in "$@"; do
    label=${v%%:*}
    files=()
    for start in $STARTS; do for seed in $SEEDS; do files+=("$R/$label-$start-$seed-$LEN"); done; done
    python3 "$(dirname "$0")/plate-dynamics.py" --label "$label" "${files[@]}"
done

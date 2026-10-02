#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The plate dynamics check on an ensemble (docs/design/plate-collisions-and-welding.md §6): runs a
# preset per variant on seeds x starts with the coupled pipeline up to the volcanoes, writes each
# run's checkpoint, and prints `deeptime plate-check`'s table for each run and for the mean of a
# variant's runs (the design asserts its plate targets on the mean of seeds 1-4). On the test VM via
# tools/remote/mac.sh; a 1000 Myr run takes about 18 minutes at 2 threads:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start pc tools/review/plate-check.sh 1000 1,2,3,4 supercontinent base
#   tools/remote/mac.sh --no-sync -- tools/remote/detach.sh wait pc
#
#   tools/review/plate-check.sh <length Myr> "<seeds>" "<starts>" label[:path=value;path=value...] ...
#
# (seeds and starts may also be separated by commas, which survive mac.sh's word joining; a variant
# is a label and `--set` overrides, as in tools/review/p1-27-ensemble.sh.) Output per run:
# $R/<label>-<start>-<seed>-<len>.{txt (the run's log), ck (checkpoint), json (the table)}, per
# variant $R/<label>-<start>-<len>.{table.txt,json}. The script's exit status is 0 when it ran;
# whether the targets hold is in the tables (plate-check exits non-zero when they do not, which is
# the expected state of a baseline). Environment: R (output folder), PRESET (default
# earthlike_quick_16k), STAGE (default volcanoes; on a pipeline preset `plates` runs the same coupled
# group, so it is no faster), J (runs at once, default 2), T (threads per run, default 2), BIN (a
# built binary: skips the build), TARGETS (a file moving target ranges, plate-check --targets),
# DEEPTIME_NO_PLATE_DYNAMICS (set: the runs record nothing, to time what recording costs).
# Delete $R when done: the checkpoints are large.
set -euo pipefail
LEN=$1; SEEDS=${2//,/ }; STARTS=${3//,/ }; shift 3
STAGE=${STAGE:-volcanoes}
R=${R:-/Volumes/Graphide/ban/deeptime-runs/plate-check/$STAGE}
PRESET=${PRESET:-earthlike_quick_16k}
J=${J:-2}; T=${T:-2}
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
            if [ -s "$f.txt" ] && [ -s "$f.ck" ]; then continue; fi
            jobs+=("$f|$start|$seed|$sets")
        done
    done
done
run_one() {
    IFS='|' read -r f start seed sets <<< "$1"
    rm -rf "$f.work" "$f.ck"
    # shellcheck disable=SC2086
    nice -n 10 "$BIN" stage "$STAGE" --preset "$PRESET" --start "$start" --length-myr "$LEN" \
        --threads "$T" --set "seed=$seed" --work "$f.work" --out "$f.ck" $sets > "$f.part" 2>&1 \
        && mv "$f.part" "$f.txt" || echo "FAILED $f (see $f.part)" >&2
    rm -rf "$f.work"
}
export -f run_one
export BIN PRESET LEN T STAGE
if [ ${#jobs[@]} -gt 0 ]; then
    printf '%s\n' "${jobs[@]}" | xargs -P "$J" -I{} bash -c 'run_one "$@"' _ {}
fi
tgt=()
if [ -n "${TARGETS:-}" ]; then tgt=(--targets "$TARGETS"); fi
for v in "$@"; do
    label=${v%%:*}
    for start in $STARTS; do
        runs=()
        for seed in $SEEDS; do
            f="$R/$label-$start-$seed-$LEN"
            [ -s "$f.ck" ] || continue
            runs+=(--run "$f.ck")
            echo "== $label $start seed $seed"
            "$BIN" plate-check --run "$f.ck" --json "$f.json" "${tgt[@]}" || true
        done
        [ ${#runs[@]} -gt 0 ] || continue
        echo "== $label $start, mean of seeds $SEEDS"
        "$BIN" plate-check "${runs[@]}" --json "$R/$label-$start-$LEN.json" "${tgt[@]}" \
            | tee "$R/$label-$start-$LEN.table.txt" || true
    done
done

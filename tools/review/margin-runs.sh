#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The ocean–continent transition's evidence (build log 2026-10-04, "An Earth-width ocean–continent
# transition"): coupled-pipeline runs of a preset on seeds x starts, each admitted on its own
# (`deeptime-admit run --kind sim:<preset>:<threads>t`, outside the script's own admission), with a checkpoint; then per run the L1
# fields the margin measure reads (`deeptime dump-fields`), `tools/review/margins.py` on them, and
# `deeptime plate-check` on the checkpoints. On the test VM through tools/remote/detach.sh:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start mt-base --kind light tools/review/margin-runs.sh base 1000 1,2,3,4 supercontinent
#   tools/remote/mac.sh -- tools/remote/detach.sh start mt-base5 --kind light tools/review/margin-runs.sh base 500 1,2,3 supercontinent,scattered_continents
#
#   tools/review/margin-runs.sh <tag> <length Myr> "<seeds>" "<starts>" [path=value ...]
#
# Output in $R/<tag>/ (R default /Volumes/Graphide/ban/deeptime-runs/ocean-margin): per run
# <start>-<seed>-<len>.{txt (log), ck, dump/, margins.json}, and the tables <len>.*. The binary is
# built once per tag into $R/<tag>/bin (BIN overrides). Seeds and starts may be separated by
# commas. Every run sets structure.deformation.density=0 (the structure stage writes nothing the
# plates or the surface read; plate-check-baseline.md). The hypsometry summary is
# tools/review/p1-17-hyps.py over the same logs (`<tag>-<start>-<seed>-<len>.txt` links).
set -euo pipefail
TAG=$1; LEN=$2; SEEDS=${3//,/ }; STARTS=${4//,/ }; shift 4
R=${R:-/Volumes/Graphide/ban/deeptime-runs/ocean-margin}
PRESET=${PRESET:-earthlike_quick_16k}
T=${T:-2}
O=$R/$TAG
mkdir -p "$O"
if [ -z "${BIN:-}" ]; then
    (
        flock 9
        if [ ! -x "$O/bin/deeptime" ]; then
            env -u DEEPTIME_ADMIT_JOB -u DEEPTIME_ADMIT_PID deeptime-admit run --kind cargo:build -- cargo build --release -q -p deeptime-cli
            mkdir -p "$O/bin"
            cp target/release/deeptime "$O/bin/deeptime"
        fi
    ) 9> "$O/.build.lock"
    BIN=$O/bin/deeptime
fi
sets=(--set structure.deformation.density=0)
for x in "$@"; do sets+=(--set "$x"); done
HERE=$(cd "$(dirname "$0")" && pwd)
PYENV='(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy])'
run_one() {
    local start=$1 seed=$2
    local f="$O/$start-$seed-$LEN"
    if [ ! -s "$f.txt" ] || [ ! -s "$f.ck" ]; then
        rm -rf "$f.work" "$f.ck"
        env -u DEEPTIME_ADMIT_JOB -u DEEPTIME_ADMIT_PID deeptime-admit run --kind "sim:$PRESET:${T}t" --label "mt-$TAG-$start-$seed-$LEN" -- \
            nice -n 10 "$BIN" stage volcanoes --preset "$PRESET" --start "$start" --length-myr "$LEN" \
            --threads "$T" --set "seed=$seed" --work "$f.work" --out "$f.ck" "${sets[@]}" > "$f.part" 2>&1 \
            && mv "$f.part" "$f.txt" || { echo "FAILED $f (see $f.part)" >&2; rm -rf "$f.work"; return 0; }
        rm -rf "$f.work"
    fi
    ln -sf "$f.txt" "$O/$TAG-$start-$seed-$LEN.txt"
    if [ ! -s "$f.margins.json" ]; then
        rm -rf "$f.dump"
        "$BIN" dump-fields --in "$f.ck" --out "$f.dump" \
            --fields surface/elevation_m,elevation,plates/crust_type,plates/thickness_m,plates/elevation_m,regime > /dev/null
        env -u DEEPTIME_ADMIT_JOB -u DEEPTIME_ADMIT_PID deeptime-admit run --kind shell:margins.py -- \
            nix shell --impure --expr "$PYENV" -c nice -n 10 python3 "$HERE/margins.py" planet "$f.dump" "$f.margins.json" \
            > "$f.margins.txt" 2>&1 || echo "margins FAILED $f" >&2
    fi
}
for start in $STARTS; do
    for seed in $SEEDS; do
        run_one "$start" "$seed" &
    done
done
wait
for start in $STARTS; do
    runs=()
    for seed in $SEEDS; do
        [ -s "$O/$start-$seed-$LEN.ck" ] && runs+=(--run "$O/$start-$seed-$LEN.ck")
    done
    [ ${#runs[@]} -gt 0 ] || continue
    "$BIN" plate-check "${runs[@]}" --json "$O/$start-$LEN.plate-check.json" > "$O/$start-$LEN.plate-check.txt" || true
done
python3 "$HERE/p1-17-hyps.py" "$O" "$LEN" "$SEEDS" "$STARTS" "$TAG" > "$O/$LEN.hyps.txt" || true
files=()
for start in $STARTS; do for seed in $SEEDS; do
    [ -s "$O/$start-$seed-$LEN.margins.json" ] && files+=("$O/$start-$seed-$LEN.margins.json")
done; done
nix shell --impure --expr "$PYENV" -c python3 "$HERE/margins.py" pool "$O/$LEN.margins-pooled.json" "${files[@]}" \
    > "$O/$LEN.margins.txt" 2>&1 || true
cat "$O/$LEN.hyps.txt" "$O/$LEN.margins.txt"
for start in $STARTS; do grep -E "met|Continental crust|Land" "$O/$start-$LEN.plate-check.txt" | head -20 || true; done
echo MARGIN-RUNS-DONE

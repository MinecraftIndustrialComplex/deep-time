#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The measurements of docs/design/playability-steering.md §8 on the test VM: quick 16k planets with
# the nudges off, at their defaults and strong, with epoch checkpoints on; then an epoch pick and a
# continue of one of them. Each run goes through the VM's admission tool; they run side by side.
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start ste-measure tools/review/steering-measure.sh <out> [seeds...]
#
# <out> gets one folder per run (world data with epochs/) and <name>.log (the run's report).
set -uo pipefail
OUT="${1:?usage: steering-measure.sh <out> [seeds...]}"; shift
SEEDS=("${@:-1 2}")
BIN="$(pwd)/target/release/deeptime"
ADMIT="${DEEPTIME_ADMIT:-/usr/local/lib/deeptime/admit.py}"
mkdir -p "$OUT"
run() { # name, then deeptime arguments
    local name="$1"; shift
    python3 "$ADMIT" run --kind sim:earthlike_quick_16k:4t --label "ste-$name" -- \
        "$BIN" "$@" --threads 4 > "$OUT/$name.log" 2>&1
    echo "$name EXIT=$?" >> "$OUT/status.txt"
}
pids=()
for seed in ${SEEDS[@]}; do
    common=(--preset earthlike_quick_16k --start scattered_continents --length-myr 500 --set seed=$seed)
    run "off-s$seed" run "${common[@]}" --set steering.enabled=false --epochs --out "$OUT/off-s$seed" &
    pids+=($!)
    run "default-s$seed" run "${common[@]}" --epochs --out "$OUT/default-s$seed" &
    pids+=($!)
    run "strong-s$seed" run "${common[@]}" --set steering.tail_fraction=0.2 --set steering.sea_rate_m_myr=10 \
        --set steering.sea_max_m=600 --set steering.co2_rate_per_myr=0.5 --set steering.co2_factor_max=4 \
        --out "$OUT/strong-s$seed" &
    pids+=($!)
done
wait "${pids[@]}"
# The epoch tools on the first seed's default run: copies, so each starts from the finished run.
s="${SEEDS%% *}"
for what in pick250 continue700 stop120; do
    rm -rf "$OUT/$what-s$s"; cp -a "$OUT/default-s$s" "$OUT/$what-s$s"
done
run "pick250-s$s" resume --out "$OUT/pick250-s$s" --stop-myr 250 &
run "continue700-s$s" resume --out "$OUT/continue700-s$s" --stop-myr 700 &
run "stop120-s$s" resume --out "$OUT/stop120-s$s" --stop-myr 120 &
wait
echo ALL-DONE >> "$OUT/status.txt"

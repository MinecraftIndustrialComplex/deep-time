#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The seamless world's evidence run (SW-1): one quick 16k planet baked twice, as the default
# Petroff-Guyou world and as the Lambert world (scale.projection lambert), the projection pass's
# record (land and ice shares on the map against the planet's, the pole anisotropy with and
# without the round footprint, the lambda_c search) and whole-world satellite and provinces pictures
# of both. Run on the test VM, detached:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start sw1-evidence --kind shell:seamless-evidence.sh \
#       bash tools/review/seamless-evidence.sh [out dir] [length myr] [seed]
#
# Each heavy step goes through the admission tool (deeptime-admit run). Output: <out>/{pg,lambert}/
# (world data), <out>/{pg,lambert}-map/ (pictures), <out>/summary.json.
set -euo pipefail
cd "$(dirname "$0")/../.."
OUT="${1:-/Volumes/Graphide/ban/deeptime-runs/sw1}"
MYR="${2:-500}"
SEED="${3:-1}"
mkdir -p "$OUT"
admit() { if command -v deeptime-admit >/dev/null; then deeptime-admit run --kind "$1" -- "${@:2}"; else "${@:2}"; fi; }
admit cargo:build cargo build --release -q -p deeptime-cli
BIN=target/release/deeptime
run() {
    local name="$1"; shift
    rm -rf "$OUT/$name"
    admit "sim:earthlike_quick_16k:6t" "$BIN" run --preset earthlike_quick_16k --start scattered_continents \
        --length-myr "$MYR" --set "seed=$SEED" --threads 6 "$@" --out "$OUT/$name" > "$OUT/$name.log" 2>&1
}
run pg &
P1=$!
run lambert --set scale.projection=lambert &
P2=$!
wait $P1; wait $P2
for w in pg lambert; do
    admit shell:world-map "$BIN" world-map --world-data "$OUT/$w" --zoom 2 --out "$OUT/$w-map" > "$OUT/$w-map.log" 2>&1
done
python3 - "$OUT" <<'PY'
import json, sys
out = sys.argv[1]
s = {}
for w in ("pg", "lambert"):
    m = json.load(open(f"{out}/{w}/manifest.json"))
    s[w] = {"scale": m["scale"], "projection_kind": (m.get("projection") or {}).get("kind"),
            "bake_projection": m["notes"]["bake"].get("projection"),
            "grids": {k: [g["w"], g["h"], g["r"]] for k, g in m["grids"].items()}}
json.dump(s, open(f"{out}/summary.json", "w"), indent=1)
print(json.dumps(s["pg"]["bake_projection"], indent=1)[:4000])
PY
grep -h "^projection\|projection petroff\|bake layers" "$OUT"/pg.log "$OUT"/lambert.log || true

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The vertical-scale study (docs/research/notes/vertical-scale-study.md): terrain statistics and
# comparison pictures of a baked planet at several vertical scales, beside vanilla's and the MIC
# pack's own terrain. On the test VM through tools/remote/mac.sh; nothing here boots Minecraft
# (the reference worlds are generated first with tools/smoke/run.py --level-type minecraft:normal
# --chunky R, under --server).
#
#   vertical-scale.sh planet <world-data> <name>      the planet's terrain height, every block column,
#                                                     as <work>/<name>.h16.npy (int16, 1/16 block)
#   vertical-scale.sh reference <world> <name> <r>    a generated world's terrain top, the square of
#                                                     radius <r> around 0,0, as <work>/<name>.ref.npy
#   vertical-scale.sh stats <args…>                   vertical_scale.py stats …
#   vertical-scale.sh py <args…>                      vertical_scale.py …
#
# <work> is $WORK (default /Volumes/Graphide/ban/deeptime-runs/vertical-scale). BIN and INSPECT
# default to this checkout's release builds.
set -euo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
WORK=${WORK:-/Volumes/Graphide/ban/deeptime-runs/vertical-scale}
BIN=${BIN:-$REPO/target/release/deeptime}
INSPECT=${INSPECT:-$REPO/target/release/deeptime-inspect}
PYENV='(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy p.matplotlib p.pillow])'
py() { nix shell --impure --expr "$PYENV" -c nice -n 10 python3 "$REPO/tools/review/vertical_scale.py" "$@"; }
mkdir -p "$WORK"

case "${1:-}" in
    planet)
        WD=$2; NAME=$3
        read -r C SEA < <(python3 -c "
import json, sys
m = json.load(open(sys.argv[1] + '/manifest.json'))
print(m['scale']['circumference_blocks'], m['scale']['sea_level_y'])" "$WD")
        H=$((C / 2)); ROWS=1024
        # Strips of 1024 rows: the heightmap command holds its box in memory.
        for ((z = -H / 2; z < H / 2; z += ROWS)); do
            f="$WORK/$NAME.strip.$((z + H / 2)).npy"
            [ -f "$f" ] || nice -n 10 "$BIN" heightmap --world-data "$WD" \
                "--box=$((-C / 2)),$z,$((C / 2)),$((z + ROWS))" --layers water_level --out "$f"
        done
        py join --work "$WORK" --name "$NAME" --world-data "$WD"
        rm -f "$WORK/$NAME".strip.*.npy "$WORK/$NAME".strip.*.npy.json
        ;;
    reference)
        WORLD=$2; NAME=$3; R=$4
        nice -n 10 "$INSPECT" heights --world "$WORLD" "--area=-$R,-$R,$R,$R" --out "$WORK/$NAME.ref.npy"
        ;;
    run)
        # A fresh planet at <m per block> (the check that a re-bake agrees with rescaling a bake):
        # the quick 16k preset, seed 2, supercontinent, 1000 Myr, only scale.m_per_block_v changed.
        MV=$2; OUT="$WORK/run$MV"
        mkdir -p "$OUT"
        python3 - "$REPO/crates/deeptime-core/presets/earthlike_quick_16k.json" "$OUT/preset.json" "$MV" <<'PY'
import json, sys
p = json.load(open(sys.argv[1]))
p["scale"]["m_per_block_v"] = float(sys.argv[3])
p["seed"] = 2
json.dump(p, open(sys.argv[2], "w"), indent=1)
PY
        RAYON_NUM_THREADS=${RAYON_NUM_THREADS:-4} nice -n 10 "$BIN" run --preset "$OUT/preset.json" \
            --start supercontinent --length-myr 1000 --out "$OUT/world"
        ;;
    stats) shift; py stats --work "$WORK" "$@" ;;
    py) shift; py "$@" ;;
    *) sed -n 3,18p "$0" >&2; exit 2 ;;
esac

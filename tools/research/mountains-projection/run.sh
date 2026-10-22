#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Mountains and the projection (docs/research/notes/mountains-and-the-projection.md): one quick 16k
# planet simulated once, baked as the Lambert world (the simulation's own frame) and projected to
# the Petroff-Guyou world the game plays (`deeptime project`), whole-world heightmaps of both, the
# cube maps' tables, then the measurements and the renders. Research tool, not shipped. Run on the
# test VM, detached; each heavy step goes through the admission tool:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start mp-planet --kind shell:mountains \
#       bash tools/research/mountains-projection/run.sh planet
#   tools/remote/mac.sh -- tools/remote/detach.sh start mp-analyse --kind shell:mountains \
#       bash tools/research/mountains-projection/run.sh analyse
#   tools/remote/mac.sh --no-sync --pull review/out/mountains-projection -- true
#
# Verbs: planet (build, run, project, heightmaps), cube (the cube maps' tables), analyse (measurements;
# MP_OTHERS=dir,dir adds other planets' land for the corner shares, MP_OLD_BIN a binary that reads
# older worlds, MP_PARTS picks parts), render
# (pictures; MP_TAUS=0.000,0.500,1.000 picks the cube maps), all. Pass the variables with env.
set -euo pipefail
cd "$(dirname "$0")/../../.."
REPO="$(pwd)"
HERE="$REPO/tools/research/mountains-projection"
WORK="${MP_WORK:-/Volumes/Graphide/ban/deeptime-runs/mountains-projection}"
OUT="$REPO/review/out/mountains-projection"
mkdir -p "$WORK" "$OUT"
admit() { if command -v deeptime-admit >/dev/null; then deeptime-admit run --kind "$1" -- "${@:2}"; else "${@:2}"; fi; }
PYENV='(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy p.matplotlib p.pillow p.mpmath])'
py() { admit "$1" nix shell --impure --expr "$PYENV" -c nice -n 10 python3 "${@:2}"; }
BIN="$REPO/target/release/deeptime"

planet() {
    admit cargo:build cargo build --release -q -p deeptime-cli
    rm -rf "$WORK/lambert" "$WORK/pg"
    admit "sim:earthlike_quick_16k:8t" "$BIN" run --preset earthlike_quick_16k --start supercontinent --length-myr 1000 --set scale.projection=lambert \
        --threads 8 --out "$WORK/lambert" > "$WORK/lambert.log" 2>&1
    admit shell:project "$BIN" project --world-data "$WORK/lambert" --out "$WORK/pg" > "$WORK/pg.log" 2>&1
    heightmaps
}

heightmaps() {
    # Whole worlds every 2 blocks (the L3 cell): H, E, A.N, meta/province, m_per_block, lat, lon.
    # Lambert: x -C/2..C/2, z over the grid rows (-2624..2624 at 16k); Petroff-Guyou: x -b..b,
    # z -b/2..b/2 (b 6,656).
    admit shell:heightmap "$BIN" heightmap --world-data "$WORK/lambert" --box=-8192,-2624,8192,2624 --step 2 \
        --layers meta/province --scale --out "$WORK/lambert-s2.npy"
    admit shell:heightmap "$BIN" heightmap --world-data "$WORK/pg" --box=-6656,-3328,6656,3328 --step 2 \
        --layers meta/province --scale --out "$WORK/pg-s2.npy"
}

cube() {
    # The conformal cube, the Moser cube and the maps between them (cube_table.py), 513^2 tables.
    py shell:python "$HERE/cube_table.py" "$WORK/cube" 512
}

analyse() {
    py shell:python "$HERE/analyse.py" --work "$WORK" --out "$OUT" --bin "$BIN" --others "${MP_OTHERS:-}" --old-bin "${MP_OLD_BIN:-}" ${MP_PARTS:+--parts "$MP_PARTS"}
}

render() {
    py shell:python "$HERE/render.py" --work "$WORK" --out "$OUT" --bin "$BIN" ${MP_TAUS:+--taus "$MP_TAUS"}
}

case "${1:-all}" in
    planet) planet ;;
    heightmaps) heightmaps ;;
    cube) cube ;;
    analyse) analyse ;;
    render) render ;;
    all) planet; cube; analyse; render ;;
    *) echo "unknown verb $1" >&2; exit 2 ;;
esac

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# RETIRED in part (cube plan 7.4): `planet`, `heightmaps` and `all` cannot run again. They baked the
# Lambert world and projected it with `deeptime project` to the Petroff-Guyou world, and both that
# command and that world are deleted (the cube is the only world type). `cube` still runs, and
# `analyse` and `render` still read a $MP_WORK that already holds lambert/ and pg/ bakes from before the
# deletion; the note keeps the numbers.
#
# Mountains and the projection (docs/research/notes/mountains-and-the-projection.md): one quick 16k
# planet simulated once, baked as the Lambert world (the simulation's own frame) and projected to
# the Petroff-Guyou world the game plays (`deeptime project`), whole-world heightmaps of both, the
# cube maps' tables, then the measurements and the renders. Research tool, not shipped. Run on the
# test VM, detached; each heavy step goes through the admission tool:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start mp-analyse --kind shell:mountains \
#       bash tools/research/mountains-projection/run.sh analyse
#   tools/remote/mac.sh --no-sync --pull review/out/mountains-projection -- true
#
# Verbs: cube (the cube maps' tables), analyse (measurements;
# MP_OTHERS=dir,dir adds other planets' land for the corner shares, MP_OLD_BIN a binary that reads
# older worlds, MP_PARTS picks parts), render
# (pictures; MP_TAUS=0.000,0.500,1.000 picks the cube maps). Pass the variables with env. (planet,
# heightmaps and all are the retired verbs.)
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

retired() {
    echo "mountains-projection: '$1' is retired: it needs 'deeptime project' and the Petroff-Guyou world, both deleted (cube plan 7.4)." >&2
    echo "  Still available: cube, analyse and render on a \$MP_WORK that already holds lambert/ and pg/ from before." >&2
    exit 2
}

case "${1:-all}" in
    planet|heightmaps|all) retired "${1:-all}" ;;
    cube) cube ;;
    analyse) analyse ;;
    render) render ;;
    *) echo "unknown verb $1" >&2; exit 2 ;;
esac

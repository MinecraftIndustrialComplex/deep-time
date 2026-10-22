#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The cube world's map table (cube_map.py) and its checks (check.py), on the test VM:
#
#   tools/remote/mac.sh --kind shell:python --pull crates/deeptime-format/data --pull review/out/cube-map -- \
#       env CM_COMMIT=$(git rev-parse --short=12 HEAD) CM_TAU=1 bash tools/cube-map/run.sh all
#
# Verbs: generate (the committed table, m 256 = n 513, into crates/deeptime-format/data/, and the
# n 257 and n 1,025 tables for the convergence check into the scratch folder), check (check.py over
# the three, review/out/cube-map/check-<name>.json), all. CM_TAU picks tau (default 1, the
# equal-area Moser cube); CM_COMMIT is the generator's commit written into the header.
set -euo pipefail
cd "$(dirname "$0")/../.."
REPO="$(pwd)"
HERE="$REPO/tools/cube-map"
TAU="${CM_TAU:-1}"
COMMIT="${CM_COMMIT:?CM_COMMIT: the generator commit for the header}"
WORK="${CM_WORK:-/Volumes/Graphide/ban/deeptime-runs/cube-map}"
OUT="$REPO/review/out/cube-map"
DATA="$REPO/crates/deeptime-format/data"
mkdir -p "$WORK" "$OUT" "$DATA"
admit() { if command -v deeptime-admit >/dev/null; then deeptime-admit run --kind "$1" -- "${@:2}"; else "${@:2}"; fi; }
PYENV='(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy])'
py() { admit "$1" nix shell --impure --expr "$PYENV" -c nice -n 10 python3 "${@:2}"; }
name() { printf 'moser-t%s-n%s' "$TAU" "$((2 * $1 + 1))"; }

generate() {
    py shell:python "$HERE/cube_map.py" generate --m 256 --q 128 --tau "$TAU" --name "$(name 256)" --commit "$COMMIT" --out "$DATA"
    py shell:python "$HERE/cube_map.py" generate --m 128 --q 64 --tau "$TAU" --name "$(name 128)" --commit "$COMMIT" --out "$WORK"
    py shell:python "$HERE/cube_map.py" generate --m 512 --q 256 --tau "$TAU" --name "$(name 512)" --commit "$COMMIT" --out "$WORK"
}

check() {
    py shell:python "$HERE/check.py" --tables "$WORK/cube-map-$(name 128).bin" "$DATA/cube-map-$(name 256).bin" \
        "$WORK/cube-map-$(name 512).bin" --committed "$DATA/cube-map-$(name 256).bin" --out "$OUT/check-$(name 256).json"
}

case "${1:-all}" in
    generate) generate ;;
    check) check ;;
    all) generate; check ;;
    *) echo "unknown verb $1" >&2; exit 2 ;;
esac

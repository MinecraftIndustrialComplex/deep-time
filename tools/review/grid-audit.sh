#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The grid-artifact audit (P1-23): does the finished world show the grids it was computed on?
# Bakes a planet, dumps what the game reads and measures the grid imprint of each product
# (tools/review/grid_audit.py; the method and the numbers are in
# docs/research/notes/grid-artifact-audit.md). Runs on the test machine, through mac.sh:
#
#   tools/remote/mac.sh -- tools/review/grid-audit.sh bake            # detached; about 15 min
#   tools/remote/mac.sh --no-sync -- tools/review/grid-audit.sh status
#   tools/remote/mac.sh -- tools/review/grid-audit.sh dump            # arrays for the analysis
#   tools/remote/mac.sh --pull review/out/grid-audit -- tools/review/grid-audit.sh analyse [part...]
#   tools/remote/mac.sh --no-sync -- tools/review/grid-audit.sh clean # delete the scratch folder
#
# GRID_AUDIT_DIR (default /Volumes/Graphide/ban/deeptime-runs/p1-23-grid) holds the planet and the
# arrays; GRID_AUDIT_WORLD names another world-data folder inside it (default `world`), so a second
# bake (`bake world-after`) can be measured beside the first. Pictures and numbers go to
# review/out/grid-audit/<world>/ (never committed).
set -euo pipefail
cd "$(dirname "$0")/../.."
S="${GRID_AUDIT_DIR:-/Volumes/Graphide/ban/deeptime-runs/p1-23-grid}"
W="${GRID_AUDIT_WORLD:-world}"
PRESET="${GRID_AUDIT_PRESET:-earthlike_quick_16k}"
START="${GRID_AUDIT_START:-scattered_continents}"
MYR="${GRID_AUDIT_MYR:-500}"
PY=(nix shell --impure --expr
    '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy p.pillow])'
    -c nice -n 10 python3)
verb="${1:-}"
[ -n "$verb" ] || { sed -n '3,17p' "$0"; exit 2; }
shift
mkdir -p "$S"

case "$verb" in
    bake)
        W="${1:-$W}"
        cargo build --release -q -p deeptime-cli
        # A copy, so a later build does not swap the binary under the run.
        cp target/release/deeptime "$S/deeptime-$W"
        rm -rf "${S:?}/$W" "${S:?}/$W-arrays"
        setsid nohup bash -c "/usr/bin/time -v '$S/deeptime-$W' run --preset '$PRESET' --start '$START' \
            --length-myr '$MYR' --out '$S/$W' --checkpoints; echo EXIT=\$?" > "$S/$W.log" 2>&1 < /dev/null &
        echo "grid-audit: bake of $W started (pid $!), log $S/$W.log"
        ;;
    status)
        W="${1:-$W}"
        tail -n "${2:-6}" "$S/$W.log"
        ;;
    dump)
        W="${1:-$W}"
        cargo build --release -q -p deeptime-cli
        B=target/release/deeptime
        D="$S/$W-arrays"
        mkdir -p "$D"
        C=$(python3 -c "import json;print(json.load(open('$S/$W/manifest.json'))['scale']['circumference_blocks'])")
        H=$((C / 2)); Q=$((C / 4))
        # The whole planet on the L3 points (every second block): E, A·N and the layers the game reads.
        [ "$D/planet.npy" -nt "$S/$W/manifest.json" ] || nice -n 10 $B heightmap --world-data "$S/$W" --box="-$H,-$Q,$H,$Q" --step 2 \
            --layers water_level,detail/amplitude,climate/ice_cover,climate/temperature,climate/vegetation,climate/continents,climate/erosion,climate/ridges,surface/class_top,meta/province,meta/plate_id,climate/koppen,climate/t_mean_c,climate/p_mm,playa \
            --out "$D/planet.npy"
        # The rock and the water the column evaluator decides, every eighth block.
        nice -n 10 $B heightmap --world-data "$S/$W" --box="-$H,-$Q,$H,$Q" --step 8 --columns \
            --out "$D/columns8.npy"
        # The simulation's own fields, for telling the stages apart.
        for ck in "$S/$W"/checkpoints/*.dtcp; do
            n=$(basename "$ck" .dtcp)
            case "$n" in *amplify*|*young*|*structure*|*climate*|*surface*|*plates*) ;; *) continue ;; esac
            nice -n 10 $B dump-fields --in "$ck" --out "$D/fields-$n" > /dev/null || true
        done
        nice -n 10 $B dump-deposits --world "$S/$W" --out "$D/deposits.json" || true
        du -sh "$D"
        ;;
    boxes)
        # Block-scale boxes named by the analysis (boxes.json: name → [x0, z0, x1, z1]).
        W="${1:-$W}"
        B=target/release/deeptime
        D="$S/$W-arrays"
        python3 - "$D/boxes.json" <<'EOF' | while read -r name box; do
import json, sys
for k, b in json.load(open(sys.argv[1])).items():
    print(k, ",".join(str(int(v)) for v in b))
EOF
            nice -n 10 $B heightmap --world-data "$S/$W" --box="$box" --step 1 --columns \
                --layers water_level,climate/ice_cover,climate/temperature,climate/vegetation,climate/continents,climate/erosion,climate/ridges \
                --out "$D/box-$name.npy"
        done
        ;;
    analyse)
        mkdir -p "review/out/grid-audit/$W"
        "${PY[@]}" tools/review/grid_audit.py --world "$S/$W" --arrays "$S/$W-arrays" \
            --out "review/out/grid-audit/$W" "$@"
        ;;
    py)
        # Any other analysis script with the same Python: grid-audit.sh py <script> [args...]
        "${PY[@]}" "$@"
        ;;
    gates)
        # The branch gates (CLAUDE.md): cargo fmt, clippy, test, deny; Gradle build and tests.
        cargo fmt --all -- --check && echo "FMT ok"
        cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -3
        cargo test --workspace 2>&1 | grep -E "^test result|FAILED|failed|panicked|^error" || true
        cargo deny check licenses bans 2>&1 | tail -2
        (cd neoforge && ./gradlew build test 2>&1 | tail -6 && python3 ../tools/ci/junit-summary.py build/test-results/test)
        ;;
    clean)
        rm -rf "$S"
        echo "grid-audit: removed $S"
        ;;
    *)
        echo "grid-audit.sh: unknown verb $verb" >&2; exit 2 ;;
esac

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The evidence for plan Q10 ("Excavated Variants in MIC?", decisions log 2026-09-28: decide after
# renders): the same Deep Time deposits in the current MIC pack, once without and once with
# Excavated Variants (EV) + Dynamic Asset Generator (DAG). Runs ON the Mac, one step per call:
#
#   tools/remote/mac.sh --pack   -- 'tools/review/q10-ev.sh install'   # smoke/mic and smoke/mic-ev
#   tools/remote/mac.sh --server -- 'tools/review/q10-ev.sh worlds'    # generate both worlds
#   tools/remote/mac.sh          -- 'tools/review/q10-ev.sh inspect'   # 2D renders, block counts
#   tools/remote/mac.sh --server -- 'tools/review/q10-ev.sh bluemap'   # BlueMap, both worlds
#   tools/remote/mac.sh --server -- 'tools/review/q10-ev.sh boots'     # more boot-time samples
#
# and on the laptop tools/client/ev-assets.sh (EV's generated textures, from a real client) before
# `inspect`/`bluemap`, then `python3 tools/review/q10_ev.py` (the page). Everything lands in
# review/out/q10/ (gitignored); pull it with mac.sh --pull review/out/q10.
#
# Both worlds come from one install recipe: the synthetic deposit world preset
# (deposits_world_16k.json: every-class strata, so many host rocks occur), the MIC rock-tag datapack,
# the same level seed, first-start generation. The EV world differs only by the two jars.
set -euo pipefail
cd "$(dirname "$0")/../.."
OUT=review/out/q10
PRESET="$PWD/crates/deeptime-core/tests/presets/deposits_world_16k.json"
TAGS=../../Docs/datapacks/deeptime-mic-rock-tags
SEED="${Q10_SEED:-7}"
PROFILES="mic mic-ev"
# Chunky squares "x,z,radius": the porphyry/supergene/placer cluster west of 0 and the coal/BIF
# guarantee next to spawn (P1-15's standalone run of the same preset).
CHUNKY="${Q10_CHUNKY:--5582,-1022,112 -16,-2144,80}"
# Render boxes "name minX maxX minZ maxZ cutY" (BlueMap's hires tiles start at x, z = 32k + 2, so
# boxes on that grid cost the fewest tiles). cutY: the cut map keeps blocks at or below it. Sites
# (both worlds, `deeptime sites near`): porphyry 2 (y 127-137) with its supergene blanket 3 (127-131),
# porphyry 10 (102-141) and blanket 1 (129-131) in the first box; the gold placer 8 (161-165) in the
# second; the coal seam 29 (66-68) and the BIF 37 (64-68) next to spawn in the third. Each cut is the
# layer with the most ore of its site (pairs-<box>.json "by_y"): 128 (156 ore blocks, 73 of them a
# different block with EV), 162 (64 gold in gravel, none different), 66 (213 coal, 1 different).
# tools/review/q10_ev.py reads this list.
BOXES="${Q10_BOXES:-porphyry -5630 -5535 -1054 -959 128
placer -5630 -5535 -1118 -1055 162
spawn -62 33 -2174 -2111 66}"
# Vertical sections "name axis at from to yfrom yto".
SECTIONS="${Q10_SECTIONS:-porphyry z -1002 -5630 -5535 90 180
porphyry10 x -5554 -1060 -960 90 180
placer z -1086 -5640 -5540 130 180
spawn x -16 -2174 -2111 20 90}"

step="${1:?usage: q10-ev.sh <install|worlds|inspect|bluemap|boots>}"
mkdir -p "$OUT"
inspect() { cargo run -q --release -p deeptime-inspect -- "$@"; }

case "$step" in
install)
    for p in $PROFILES; do
        tools/smoke/install.sh "$p" --preset "$PRESET" --length-myr 500 --datapack "$TAGS" --seed "$SEED" 2>&1 \
            | grep -v -E '^   (copied untracked|[A-Za-z0-9_.-]+ [A-Za-z0-9]{8} ->)' | tail -12
        python3 -c "import json,sys; d=json.load(open('smoke/$p/install.json')); print('$p:', d['pack'], 'seed', d['seed'], d['deeptime_jar'])"
    done ;;
worlds)
    args=()
    for c in $CHUNKY; do args+=(--chunky-at "$c"); done
    for p in $PROFILES; do
        mkdir -p "$OUT/$p"
        echo "== $p"
        set +e
        python3 tools/smoke/run.py --profile "$p" --fresh "${args[@]}" \
            --rcon "deeptime sites near -5582 -1022 400" --rcon "deeptime sites near -16 -2144 400" \
            --rcon "deeptime probe claims" --restart 2>&1 | grep -E '^\s*\[(PASS|FAIL| -- )\]|^(PASS|FAIL)|rcon deeptime sites' | cut -c1-400
        set -e
        cp smoke/"$p"/report.md smoke/"$p"/rcon.jsonl smoke/"$p"/install.json "$OUT/$p/" 2>/dev/null || true
        # EV's and DAG's own lines, and the boot's phases, for the cost table.
        grep -E -i 'excavated|dynamic.?asset|dynamicassetgenerator|Done \(|Loading [0-9]+ mods|Preparing (start region|level)' \
            smoke/"$p"/logs/server.log | cut -c1-400 > "$OUT/$p/server-lines.txt" || true
        find smoke/"$p"/server/mods -maxdepth 1 -name '*.jar' | wc -l | tr -d ' ' > "$OUT/$p/mod-count.txt"
    done ;;
inspect)
    # Colour tables for the sections: the set's jars, plus (with EV) the assets DAG made on a client
    # (tools/client/ev-assets.sh), zipped as one more "jar" so `colors` reads it like a mod's.
    vanilla="$(find neoforge/build/moddev/artifacts -name '*client-extra*.jar' | head -1)"
    for p in $PROFILES; do
        [ -n "$vanilla" ] || { echo "no client-extra jar: run a Gradle build first" >&2; break; }
        jars="$OUT/colors-jars-$p"; rm -rf "$jars"; mkdir -p "$jars"
        for j in smoke/"$p"/server/mods/*.jar; do ln -s "$PWD/$j" "$jars/"; done
        if [ "$p" = mic-ev ] && [ -d "$OUT/ev-assets/pack/assets" ]; then
            python3 -c 'import os,sys,zipfile
src, dst = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(dst, "w", zipfile.ZIP_DEFLATED) as z:
    for r, _, fs in os.walk(src):
        for f in fs: z.write(os.path.join(r, f), os.path.relpath(os.path.join(r, f), src))' "$OUT/ev-assets/pack" "$jars/zz-ev-assets.jar"
        fi
        inspect colors --jars "$jars" --vanilla "$vanilla" --out "$OUT/colors-$p.json"
    done
    # EV's stone and ore configs as the mic-ev set ships them (EV's jar and Deep Time's four stand-ins):
    # which host rocks and which ores can have a variant at all.
    python3 - smoke/mic-ev/server/mods "$OUT/ev-configs.json" <<'PY'
import glob, json, os, re, sys, zipfile
mods, out = sys.argv[1], sys.argv[2]
res = {"stones": {}, "ores": {}, "jars": []}
for jar in sorted(glob.glob(os.path.join(mods, "*.jar"))):
    with zipfile.ZipFile(jar) as z:
        hit = False
        for n in z.namelist():
            m = re.match(r"defaultresources/globaldata/([^/]+)/excavated_variants/(stone|ore)/(.+)\.json$", n)
            if not m:
                continue
            hit = True
            ns, kind, path = m.groups()
            d = json.loads(z.read(n))
            if kind == "stone":
                res["stones"][f"{ns}:{path}"] = d.get("block")
            else:
                res["ores"][f"{ns}:{path}"] = {"blocks": sorted(d.get("blocks", {})), "tags": d.get("tags", [])}
        if hit:
            res["jars"].append(os.path.basename(jar))
json.dump(res, open(out, "w"), indent=1)
print(f"ev-configs: {len(res['stones'])} stones, {len(res['ores'])} ores from {res['jars']}")
PY
    for p in $PROFILES; do
        w=smoke/$p/server/planet
        d="$OUT/$p/inspect"; rm -rf "$d"; mkdir -p "$d"
        while read -r name x0 x1 z0 z1 cut; do
            [ -n "$name" ] || continue
            area="$x0,$z0,$((x1 + 1)),$((z1 + 1))"
            inspect blocks --world "$w" --area="$area" > "$d/blocks-$name.json"
            inspect ore-hosts --world "$w" --area="$area" > "$d/ore-hosts-$name.json"
            inspect ore --world "$w" --area="$area" --scale 4 --out "$d/ore-$name" > "$d/ore-$name.json"
        done <<< "$BOXES"
        colors=""
        [ -f "$OUT/colors-$p.json" ] && colors="--colors $OUT/colors-$p.json"
        while read -r name axis at from to y0 y1; do
            [ -n "$name" ] || continue
            # shellcheck disable=SC2086
            inspect section --world "$w" --axis "$axis" --at="$at" --from="$from" --to="$to" --y-from="$y0" --y-to="$y1" \
                --scale 4 --out "$d/section-$name" $colors > "$d/section-$name.txt"
        done <<< "$SECTIONS"
        cp smoke/"$p"/palette.json "$OUT/$p/" 2>/dev/null || true
        echo "$p: $(find "$d" -name '*.png' | wc -l | tr -d ' ') PNGs in $d"
    done
    # The two worlds position by position: every ore position with the block each world has there
    # and the host rock beside it (EV's block names its stone exactly, which checks the estimate).
    while read -r name x0 x1 z0 z1 cut; do
        [ -n "$name" ] || continue
        inspect ore-hosts --world smoke/mic/server/planet --other smoke/mic-ev/server/planet \
            --area="$x0,$z0,$((x1 + 1)),$((z1 + 1))" > "$OUT/pairs-$name.json"
    done <<< "$BOXES"
    # A 16 x 16 texture for every block the pairs name (ores both ways, host rocks) for the page.
    if [ -n "$vanilla" ]; then
        rm -rf "$OUT/tiles"
        python3 tools/review/block-tiles.py --jars "$OUT/colors-jars-mic-ev" --vanilla "$vanilla" --out "$OUT/tiles" "$OUT"/pairs-*.json
    fi ;;
bluemap)
    for p in $PROFILES; do
        boxes="$(awk 'NF { printf "%s%s %s %s %s %s", (n++ ? "; " : ""), $2, $3, $4, $5, $6 }' <<< "$BOXES")"
        case "$p" in
            mic) prefix=plain-; label="MIC without EV"; sort=0; packs="" ;;
            mic-ev) prefix=ev-; label="MIC with EV"; sort=2; packs="$OUT/ev-assets/pack" ;;
        esac
        BLUEMAP_BOXES="$boxes" BLUEMAP_ONLY_PRESENT=1 BLUEMAP_MAP_PREFIX="$prefix" BLUEMAP_MAP_LABEL="$label" \
            BLUEMAP_MAP_SORT="$sort" BLUEMAP_PACKS="$packs" \
            tools/review/render-bluemap.sh "smoke/$p/server/planet" "smoke/$p/server/mods" "$OUT/bluemap-$p" 6 2>&1 \
            | grep -E -i 'render-bluemap|error|missing|exception|Rendering|finished' | grep -v -E 'Missing.*(sound|lang)' | tail -40
    done ;;
boots)
    # Boot-only runs on the existing worlds (no generation, no chunky): time to Done and RSS.
    for i in 1 2 3; do
        for p in $PROFILES; do
            python3 tools/smoke/run.py --profile "$p" 2>&1 | grep -E 'boot:|RSS|^(PASS|FAIL)' | sed "s/^/$p #$i /"
            cp smoke/"$p"/report.md "$OUT/$p/report-boot-$i.md"
        done
    done ;;
*) echo "q10-ev.sh: unknown step $step" >&2; exit 2 ;;
esac

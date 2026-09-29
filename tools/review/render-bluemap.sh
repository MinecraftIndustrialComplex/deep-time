#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Render a world with the BlueMap CLI into a static web app (plan §9.6, P0-14).
#
#   tools/review/render-bluemap.sh <world dir> <mods dir> <out dir> [threads]
#
# BLUEMAP_BOX="minX maxX minZ maxZ" limits the render (default: 256 blocks around 0,0). Hires tiles
# of a 992-block-tall world run to several MB before gzip, so a review render covers only the area
# a reviewer needs; tiles are stored gzipped and unzipped by the web app in the browser. The second
# map ("underground") keeps only what lies below y 0.
#
# BLUEMAP_BOXES="minX maxX minZ maxZ cutY; minX maxX minZ maxZ cutY; …" (Q10) instead renders
# several boxes, and the second map becomes a cut: in each box only the blocks at or below its
# cutY, seen from above, so ore bodies inside rock show. Boxes must not overlap.
# BLUEMAP_MAP_PREFIX="ev-" names the maps <prefix>planet and <prefix>underground (default: none),
# BLUEMAP_MAP_LABEL their titles (default "Deep Time planet"), so the maps of two worlds rendered
# into two out dirs can be merged into one web app (tools/review/q10_ev.py).
# BLUEMAP_PACKS="<folder or zip> …" adds resource packs (BlueMap reads <config>/packs/), e.g. the
# assets Dynamic Asset Generator made for Excavated Variants on a client (tools/client/ev-assets.sh):
# no jar holds them, so without the pack BlueMap has no model for EV's blocks.
#
# <mods dir> is a folder of mod jars whose block models and textures BlueMap loads (a server
# set's mods/, e.g. smoke/mic/server/mods). The web app lands in <out dir>/web; BlueMap's own
# data (resource cache, logs) in <out dir>/data. Run it on the Mac under the server lock:
#   tools/remote/mac.sh --server -- 'tools/review/render-bluemap.sh smoke/mic/server/planet smoke/mic/server/mods review/out/bluemap'
#
# BLUEMAP_ONLY_PRESENT=1 gives BlueMap only the jars whose blocks occur in the boxes (found with
# `deeptime-inspect blocks`): a map's textures.json holds every texture of every jar it is given,
# and with all ~160 MIC jars it is 10.4 MB gzipped, over the page host's 10 MB per-file limit.
set -euo pipefail
cd "$(dirname "$0")/../.."
WORLD="$(cd "$1" && pwd)"; MODS="$(cd "$2" && pwd)"; OUT="$3"; THREADS="${4:-4}"
mkdir -p "$OUT/config/maps" "$OUT/config/storages" "$OUT/web" "$OUT/data" libs/bluemap
OUT="$(cd "$OUT" && pwd)"

. <(grep -E '^(version|url|sha256)=' tools/bluemap/VERSION)
JAR="libs/bluemap/bluemap-$version-cli.jar"
if ! echo "$sha256  $JAR" | sha256sum -c --quiet 2>/dev/null; then
    curl -fsSL -o "$JAR.part" "$url" && mv "$JAR.part" "$JAR"
    echo "$sha256  $JAR" | sha256sum -c --quiet || { echo "render-bluemap: sha256 mismatch for $JAR" >&2; exit 1; }
fi

# Boxes: "minX maxX minZ maxZ cutY" per line.
if [ -n "${BLUEMAP_BOXES:-}" ]; then
    BOXES="$(printf '%s' "$BLUEMAP_BOXES" | tr ';' '\n' | sed -e 's/^ *//' -e 's/ *$//' -e '/^$/d')"
    CUTNAME="cut at the ore level"; CUTLIGHT=1.0
else
    BOXES="${BLUEMAP_BOX:--128 127 -128 127} 0"
    CUTNAME="below y 0 (strata)"; CUTLIGHT=0.6
fi
while read -r a b c d e rest; do
    [ -n "$e" ] && [ -z "$rest" ] || { echo "render-bluemap: a box is 'minX maxX minZ maxZ cutY': '$a $b $c $d $e $rest'" >&2; exit 2; }
done <<< "$BOXES"
read -r MINX MAXX MINZ MAXZ _ <<< "$(head -1 <<< "$BOXES")"
CX=$(( (MINX + MAXX) / 2 )); CZ=$(( (MINZ + MAXZ) / 2 ))
MASK="$(awk '{printf "%s{ min-x: %d, max-x: %d, min-z: %d, max-z: %d }", (NR > 1 ? ", " : ""), $1, $2, $3, $4}' <<< "$BOXES")"
CUTMASK="$(awk '{printf "%s{ min-x: %d, max-x: %d, min-z: %d, max-z: %d, max-y: %d }", (NR > 1 ? ", " : ""), $1, $2, $3, $4, $5}' <<< "$BOXES")"
PREFIX="${BLUEMAP_MAP_PREFIX:-}"; LABEL="${BLUEMAP_MAP_LABEL:-Deep Time planet}"; SORT="${BLUEMAP_MAP_SORT:-0}"

if [ -n "${BLUEMAP_ONLY_PRESENT:-}" ]; then
    NS=""
    while read -r a b c d _; do
        NS="$NS $(cargo run -q --release -p deeptime-inspect -- blocks --world "$WORLD" --area="$a,$c,$((b + 1)),$((d + 1))" \
                  | python3 -c 'import json,sys; print(" ".join(k for k in json.load(sys.stdin)["namespaces"] if k != "minecraft"))')"
    done <<< "$BOXES"
    rm -rf "$OUT/mods-present" && mkdir -p "$OUT/mods-present"
    python3 - "$MODS" "$OUT/mods-present" $NS <<'PY2'
import os, re, sys, zipfile
mods, dest, wanted = sys.argv[1], sys.argv[2], set(sys.argv[3:])
def ids(jar):
    try:
        with zipfile.ZipFile(jar) as z:
            text = z.read("META-INF/neoforge.mods.toml").decode("utf-8", "replace")
    except Exception:
        return set()
    out, in_mods = set(), False
    for line in text.splitlines():
        t = line.strip()
        if t.startswith("["):
            in_mods = t.startswith("[[mods]]")
        m = re.match(r'modId\s*=\s*"([^"]+)"', t)
        if in_mods and m:
            out.add(m.group(1))
    return out
found = set()
for f in sorted(os.listdir(mods)):
    if f.endswith(".jar") and ids(os.path.join(mods, f)) & wanted:
        os.symlink(os.path.realpath(os.path.join(mods, f)), os.path.join(dest, f))
        found |= ids(os.path.join(mods, f)) & wanted
print(f"render-bluemap: {len(found)} of {len(wanted)} block namespaces matched to jars; missing: {sorted(wanted - found)}")
PY2
    MODS="$OUT/mods-present"
fi
rm -rf "$OUT/config/packs" && mkdir -p "$OUT/config/packs"
for pack in ${BLUEMAP_PACKS:-}; do
    [ -e "$pack" ] || { echo "render-bluemap: no resource pack at $pack" >&2; exit 1; }
    ln -s "$(cd "$(dirname "$pack")" && pwd)/$(basename "$pack")" "$OUT/config/packs/$(basename "$pack")"
    echo "render-bluemap: resource pack $(basename "$pack")"
done
fill() {
    python3 - "$1" "$2" <<PY2
import sys
text = open(sys.argv[1]).read()
for k, v in {"CX": "$CX", "CZ": "$CZ", "MASK": """$MASK""", "CUTMASK": """$CUTMASK""", "WORLD": "$WORLD",
             "DATA": "$OUT/data", "WEBROOT": "$OUT/web", "THREADS": "$THREADS", "LABEL": """$LABEL""",
             "CUTNAME": "$CUTNAME", "CUTLIGHT": "$CUTLIGHT", "SORT0": str($SORT), "SORT1": str($SORT + 1)}.items():
    text = text.replace("@" + k + "@", v)
open(sys.argv[2], "w").write(text)
PY2
}
fill tools/bluemap/core.conf.tmpl "$OUT/config/core.conf"
fill tools/bluemap/webapp.conf.tmpl "$OUT/config/webapp.conf"
fill tools/bluemap/storages/file.conf.tmpl "$OUT/config/storages/file.conf"
rm -f "$OUT"/config/maps/*.conf
for m in tools/bluemap/maps/*.conf.tmpl; do fill "$m" "$OUT/config/maps/$PREFIX$(basename "$m" .conf.tmpl).conf"; done
# BlueMap writes defaults for anything missing (webserver.conf, plugin.conf); the web server is
# never started (-r renders, -g writes the web app).
rm -rf "$OUT/web/maps"   # a smaller box must not leave tiles of an earlier, larger render behind
nix shell nixpkgs#jdk25 -c java -Xmx4G -jar "$JAR" -c "$OUT/config" -n "$MODS" -v 1.21.1 -r -g
# Let the browser unzip the gzip tiles (see storages/file.conf.tmpl).
python3 - "$OUT/web/settings.json" <<'PY2'
import json, sys
p = sys.argv[1]; d = json.load(open(p)); d["clientDecompression"] = True
json.dump(d, open(p, "w"))
PY2
# A static review page needs neither the source map, the PHP backend nor 29 translations, and the
# page host allows 200 files per space; the render state (rstate/) is BlueMap's own bookkeeping.
rm -f "$OUT"/web/assets/*.map "$OUT/web/sql.php"
rm -rf "$OUT"/web/maps/*/rstate
find "$OUT/web/lang" -type f ! -name 'en.conf' ! -name 'settings.conf' -delete
echo "render-bluemap: web app in $OUT/web ($(du -sh "$OUT/web" | cut -f1), $(find "$OUT/web" -type f | wc -l) files), clientDecompression on"

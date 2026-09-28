#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Render a world with the BlueMap CLI into a static web app (plan §9.6, P0-14).
#
#   tools/review/render-bluemap.sh <world dir> <mods dir> <out dir> [threads]
#
# <mods dir> is a folder of mod jars whose block models and textures BlueMap loads (a server
# set's mods/, e.g. smoke/mic/server/mods). The web app lands in <out dir>/web; BlueMap's own
# data (resource cache, logs) in <out dir>/data. Run it on the Mac under the server lock:
#   tools/remote/mac.sh --server -- 'tools/review/render-bluemap.sh smoke/mic/server/planet smoke/mic/server/mods review/out/bluemap'
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

fill() { sed -e "s#@WORLD@#$WORLD#g" -e "s#@DATA@#$OUT/data#g" -e "s#@WEBROOT@#$OUT/web#g" -e "s#@THREADS@#$THREADS#g" "$1" > "$2"; }
fill tools/bluemap/core.conf.tmpl "$OUT/config/core.conf"
fill tools/bluemap/webapp.conf.tmpl "$OUT/config/webapp.conf"
fill tools/bluemap/storages/file.conf.tmpl "$OUT/config/storages/file.conf"
for m in tools/bluemap/maps/*.conf.tmpl; do fill "$m" "$OUT/config/maps/$(basename "$m" .tmpl)"; done
# BlueMap writes defaults for anything missing (webserver.conf, plugin.conf); the web server is
# never started (-r renders, -g writes the web app).
nix shell nixpkgs#jdk25 -c java -Xmx4G -jar "$JAR" -c "$OUT/config" -n "$MODS" -v 1.21.1 -r -g
echo "render-bluemap: web app in $OUT/web ($(du -sh "$OUT/web" | cut -f1))"

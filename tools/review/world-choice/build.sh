#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Cube phase 2, Ben's world-shape decision: build the page (page.py) from compare.sh's numbers and
# maps and shots.sh's client passes, then shoot it at a phone's and a desktop's width (shot-page.py)
# to look at before publishing. RETIRED in part (cube plan 7.4): the page is published; its Petroff-Guyou
# panels read the saved $CMP folder (compare.json, the pg16 maps and cmp-pg-* shots) and cannot be
# remade. Runs on the test VM; pull <out> and publish it from the laptop:
#   tools/remote/mac.sh --pull build/world-choice -- bash tools/review/world-choice/build.sh [gates.json]
#   tools/review/publish.sh world-choice build/world-choice/page
set -euo pipefail
cd "$(dirname "$0")/../../.."
CMP=${CMP:-/Volumes/Graphide/ban/deeptime-runs/cube2-cmp}
OUT=build/world-choice
PY='(builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux.python3.withPackages (p: [p.pillow])'
rm -rf "$OUT"; mkdir -p "$OUT/look"
G=(); [ -n "${1:-}" ] && G=(--gates "$1")
nix shell --impure --expr "$PY" -c python3 -I tools/review/world-choice/page.py --cmp "$CMP" --shots review/out \
  --out "$OUT/page" "${G[@]}"
du -sh "$OUT/page"; find "$OUT/page" -type f -size +9M
# Fonts for headless Chromium (the VM has none of its own).
FC=$(nix build --no-link --print-out-paths --impure --expr \
  'with (builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux; makeFontsConf { fontDirectories = [ dejavu_fonts noto-fonts ]; }')
# One shot per section: headless Chromium repeats the page past 8,192 pixels of window, and its
# --screenshot takes the document's top whatever a #fragment scrolled to, so each section is shot
# from a copy of the page that shows only that section (removed before publishing).
for sec in glance maps numbers places edges far gates cost decide; do
  sed "s|</head>|<style>main > *:not(#$sec) { display: none !important; }</style></head>|" \
    "$OUT/page/index.html" > "$OUT/page/look-$sec.html"
done
for w in 390 1280; do
  for sec in top glance maps numbers places edges far gates cost decide; do
    path="look-$sec.html"; [ "$sec" = top ] && path=index.html
    FONTCONFIG_FILE="$FC" deeptime-admit run --kind auto -- python3 -I tools/review/shot-page.py "$OUT/page" \
      "$OUT/look/w$w-$sec.png" --width "$w" --height 8000 --path "$path"
  done
done
rm -f "$OUT"/page/look-*.html
echo "build done"

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# sections_shots.py with a Chromium: playwright's headless shell if the machine has one, else nixpkgs'.
#   tools/workbench/sections-shots.sh http://127.0.0.1:8661 <run> <out dir> [sections_shots.py options]
set -euo pipefail
cd "$(dirname "$0")/../.."
if [ -z "${DEEPTIME_CHROME:-}" ] && ! ls ~/Library/Caches/ms-playwright/chromium_headless_shell-*/chrome-headless-shell-*/chrome-headless-shell > /dev/null 2>&1; then
    if ! command -v chromium > /dev/null 2>&1; then
        # nixpkgs' Chromium finds no fonts of its own: give it DejaVu through a fontconfig file.
        fonts="$(nix build nixpkgs#dejavu_fonts --no-link --print-out-paths | head -1)"
        conf="$(mktemp)"
        printf '<?xml version="1.0"?><fontconfig><dir>%s</dir><cachedir>/tmp/dt-fontcache</cachedir></fontconfig>\n' "$fonts" > "$conf"
        FONTCONFIG_FILE="$conf" exec nix shell nixpkgs#chromium -c "$0" "$@"
    fi
    DEEPTIME_CHROME="$(command -v chromium)"
    export DEEPTIME_CHROME
fi
exec python3 tools/workbench/sections_shots.py "$@"

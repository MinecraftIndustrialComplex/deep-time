#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The Mac half of tools/client/mac-screenshots.sh (run that, on the laptop; never this directly).
# Runs ON Ben's M6 Mac mini from the repo root, inside `tools/remote/mac.sh --no-sync --server`,
# so Deep Time's server lock is held for the whole run (the client starts an integrated server).
#
# Boots the real NeoForge 1.21.1 client -- ModDevGradle's own dev launch (build/moddev/runClient.sh,
# written by `./gradlew createClientLaunchScript`): Deep Time from this checkout's build output,
# Toroidal World's bundle from run/mods (linkRunModsClient) -- in the Mac's logged-in desktop
# session, where LWJGL gets a real window and the M6's GPU (no Xvfb, no HeadlessMC). With
# -Ddeeptime.dev.screenshots=true, ScreenshotDriver walks Create World -> editor -> run/timelapse
# -> review -> Accept -> world creation and writes the PNGs from the GL framebuffer itself, so no
# macOS screen-recording permission is involved (screencapture would need one; nothing grants it).
#
#   tools/client/mac-client.sh --name <name> --m6lock <path to graphide's m6lock.sh>
#                              [--no-guidance] [--no-create] [--size WxH] [--timeout S] [--lease-wait S]
#
# The launch itself -- the drive lease on graphide's M6 lock (taken before the first window opens,
# renewed every minute, released), the desktop session and `launchctl asuser`, the pid-marker
# cleanup on every failure, timeout or signal -- is tools/client/mac-client-lib.sh, which
# tools/client/ev-assets-mac.sh shares; read its header. --lease-wait is how long to wait for a busy
# desktop (default 600 s).
#
# Passes: `main` (create=true unless --no-create: 01..10, plus 02b and 02c, the Advanced settings) and, unless --no-guidance, `guidance`
# (-Ddeeptime.natives.forceFail=unsupported: its two shots land as 11-guidance-panel and
# 12-create-world-natives-missing). Output: review/out/<name>/ (PNGs, client-<pass>.log,
# shots.txt, runner.log, crash reports); tools/remote/mac.sh --pull brings it to the laptop.

set -euo pipefail
cd "$(dirname "$0")/../.."
REPO="$PWD"
source tools/client/mac-client-lib.sh

NAME=""; M6LOCK=""; GUIDANCE=1; CREATE=true; WINDOW_SIZE=""; TIMEOUT=1200; LEASE_WAIT=600
while [ $# -gt 0 ]; do
    case "$1" in
        --name) NAME="$2"; shift 2 ;;
        --m6lock) M6LOCK="$2"; shift 2 ;;
        --no-guidance) GUIDANCE=0; shift ;;
        --no-create) CREATE=false; shift ;;
        --size) WINDOW_SIZE="$2"; shift 2 ;;
        --timeout) TIMEOUT="$2"; shift 2 ;;
        --lease-wait) LEASE_WAIT="$2"; shift 2 ;;
        *) echo "mac-client: unknown argument $1" >&2; exit 2 ;;
    esac
done

OUT="$REPO/review/out/$NAME"
PURPOSE="Deep Time real-client screenshots ($NAME)"
mc_start

failed=0
set +e
# MC_SHOT_PROPS: more -D properties for the main pass (space-separated), e.g. the timeline and the
# vistas (ScreenshotDriver: deeptime.dev.screenshots.start/.length/.vista).
# shellcheck disable=SC2086
mc_run_pass main -Ddeeptime.dev.screenshots=true "-Ddeeptime.dev.screenshots.create=$CREATE" ${MC_SHOT_PROPS:-}; st=$?
set -e
[ "$st" = 0 ] || failed=1
cp "$GAMEDIR"/screenshots/*.png "$OUT/" 2>/dev/null || true
if [ "$GUIDANCE" = 1 ]; then
    set +e
    mc_run_pass guidance -Ddeeptime.dev.screenshots=true -Ddeeptime.dev.screenshots.create=false -Ddeeptime.natives.forceFail=unsupported; st=$?
    set -e
    [ "$st" = 0 ] || failed=1
    # Its 01 (Create World before the editor) repeats the main pass's; only the two new shots are kept.
    cp "$GAMEDIR/screenshots/02-guidance-panel.png" "$OUT/11-guidance-panel.png" 2>/dev/null || true
    cp "$GAMEDIR/screenshots/03-create-world-natives-missing.png" "$OUT/12-create-world-natives-missing.png" 2>/dev/null || true
fi

expected=(01-create-world-deeptime 02-setup-no-timeline 02b-setup-advanced 02c-setup-advanced-scrolled 03-setup-ready 04-run-map-live 05-run-globe-live
          06-run-done 07-run-scrubbed 08-review 09-create-world-accepted)
[ "$CREATE" = true ] && expected+=(10-in-world)
[ "$GUIDANCE" = 1 ] && expected+=(11-guidance-panel 12-create-world-natives-missing)
for e in "${expected[@]}"; do
    if [ ! -f "$OUT/$e.png" ]; then mc_log "missing: $e.png"; failed=1
    elif [ "$(wc -c < "$OUT/$e.png")" -lt 10000 ]; then mc_log "suspicious: $e.png is under 10 kB (a blank frame?)"; fi
done
grep -q "FAILED" "$OUT/shots.txt" 2>/dev/null && { mc_log "the driver reported FAILED (shots.txt)"; failed=1; }
mc_log "$(ls "$OUT"/*.png 2>/dev/null | wc -l | tr -d ' ') PNGs in review/out/$NAME"
exit "$failed"

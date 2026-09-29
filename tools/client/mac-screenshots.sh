#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Real-client screenshots of the Create World planet editor (P1-11's client check, plan §9.8) on
# Ben's M6 Mac mini's logged-in desktop session -- the way every real-client run goes (Ben,
# 2026-09-28; CLAUDE.md "Real-client runs"). Run from the laptop; nothing Minecraft runs here.
#
#   tools/client/mac-screenshots.sh [--name <name>] [--no-build] [--no-guidance] [--no-create]
#                                   [--size WxH] [--timeout S] [--lease-wait S]
#
#   --name        output folder review/out/<name>/ (default mac-client); replaced on every run
#   --no-build    skip the sync and the Gradle step (the Mac copy is already built)
#   --no-guidance skip the second pass (natives forced off: the guidance panel, Create blocked)
#   --no-create   stop after Accept (no world creation, no in-world shot)
#   --size WxH    window size (default the game's 854x480, the size of P1-11's laptop set)
#   --timeout S   per client pass, default 1200
#   --lease-wait  how long to wait for the Mac's desktop (graphide's M6 lock), default 600 s
#
# Steps: (1) tools/remote/mac.sh syncs this checkout and runs `./gradlew classes gametestClasses
# createClientLaunchScript linkRunModsClient` (Deep Time's classes and host natives, ModDevGradle's
# dev launch script, Toroidal World's bundle in run/mods); (2) copies graphide's m6lock.sh to the
# Mac (M6LOCK_SH overrides where it is read from); (3) `mac.sh --no-sync --server` runs
# tools/client/mac-client.sh there, under Deep Time's server lock: it takes graphide's `drive`
# lease, launches the client in the desktop session (sudo -n launchctl asuser), lets
# ScreenshotDriver walk the screens, renews the lease while the client runs, releases it, and kills
# only its own client on any failure; (4) mac.sh --pull brings review/out/<name>/ back: the twelve
# PNGs (01..10 from the main pass, 11-12 from the guidance pass), client-<pass>.log, shots.txt
# (the driver's [deeptime-shot] lines) and runner.log (session, lease and exit lines).
# Ctrl-C stops the Mac side too (the runner's pid file), which releases the lease.

set -euo pipefail
cd "$(dirname "$0")/../.."
REPO="$(git rev-parse --show-toplevel)"

NAME=mac-client; BUILD=1; PASS=()
while [ $# -gt 0 ]; do
    case "$1" in
        --name) NAME="$2"; shift 2 ;;
        --no-build) BUILD=0; shift ;;
        --no-guidance|--no-create) PASS+=("$1"); shift ;;
        --size|--timeout|--lease-wait) PASS+=("$1" "$2"); shift 2 ;;
        -h|--help) sed -n '2,/^set -euo/p' "$0" | sed 's/^# \{0,1\}//' | sed '$d'; exit 0 ;;
        *) echo "mac-screenshots: unknown argument $1 (see --help)" >&2; exit 2 ;;
    esac
done
[[ "$NAME" =~ ^[A-Za-z0-9._-]+$ ]] || { echo "mac-screenshots: --name must be one plain word" >&2; exit 2; }

# The same host and layout as tools/remote/mac.sh.
HOST="${DEEPTIME_MAC:-ban@m6-mac-mini}"
SSD=/Volumes/Graphide/ban
REMOTE="$SSD/mic/mods/$(basename "$REPO")"
CACHE="$SSD/deeptime-cache"
SSH=(ssh -o BatchMode=yes -o ConnectTimeout=15)

# graphide's lock script is the protocol's one source; it is copied fresh on every run.
M6LOCK_SH="${M6LOCK_SH:-/home/greencheetah/Projects/graphide/graphide/utilities/scripts/mactest/m6lock.sh}"
[ -f "$M6LOCK_SH" ] || { echo "mac-screenshots: no m6lock.sh at $M6LOCK_SH (graphide's utilities/scripts/mactest/; set M6LOCK_SH)" >&2; exit 2; }

if [ "$BUILD" = 1 ]; then
    tools/remote/mac.sh -- 'cd neoforge && ./gradlew --console=plain -q classes gametestClasses createClientLaunchScript linkRunModsClient'
fi
"${SSH[@]}" "$HOST" "mkdir -p '$CACHE' && cat > '$CACHE/m6lock.sh'" < "$M6LOCK_SH"

stop_mac_side() {
    trap - INT TERM
    echo "mac-screenshots: interrupted; stopping the Mac side" >&2
    "${SSH[@]}" -n "$HOST" "p=\$(cat '$REMOTE/neoforge/build/mac-client/runner.pid' 2>/dev/null) && kill -TERM \$p" || true
    exit 130
}
trap stop_mac_side INT TERM

rm -rf "review/out/$NAME"
set +e
tools/remote/mac.sh --no-sync --server --pull "review/out/$NAME" -- \
    "tools/client/mac-client.sh --name $NAME --m6lock '$CACHE/m6lock.sh' ${PASS[*]:-}"
status=$?
set -e
trap - INT TERM

echo "--------------------------------------------------------------------"
grep -v "^\[[a-z]*\] share code" "review/out/$NAME/shots.txt" 2>/dev/null | sed 's| -> /.*/| -> |' | cut -c1-200 || true
grep -hE "Moved staged|Copied staged|Bound Deep Time" "review/out/$NAME"/client-*.log 2>/dev/null | sed 's/^.*\]: //' | head -5 || true
echo
ls -1 "review/out/$NAME"/*.png 2>/dev/null || echo "no screenshots"
echo "exit $status; output review/out/$NAME/ (runner.log, client-<pass>.log, shots.txt)"
exit "$status"

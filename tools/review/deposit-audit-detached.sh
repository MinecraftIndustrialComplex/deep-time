#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# deposit-audit-previews.sh, detached: survives the laptop's SSH session (which has dropped
# several times). Takes Deep Time's server lock itself, by tools/remote/mac.sh's protocol (an
# atomic mkdir of the lock directory holding its owner; a lock older than three hours is stale),
# runs the previews and releases it. Start it from mac.sh WITHOUT --server, inside the dev shell:
#
#   tools/remote/mac.sh --pack -- 'nohup tools/review/deposit-audit-detached.sh <world-data> <box> >/dev/null 2>&1 &'
#
# Progress and results: review/out/deposit-audit/previews.log, then box-<profile>.json and
# vanilla.json there; the last log line is DONE.
set -uo pipefail
cd "$(dirname "$0")/../.."
OUT=review/out/deposit-audit
mkdir -p "$OUT"
LOG="$OUT/previews.log"
LOCK="${DEEPTIME_CACHE:-/Volumes/Graphide/ban/deeptime-cache}/server.lock"
: > "$LOG"
while ! mkdir "$LOCK" 2>/dev/null; do
    age=$(( $(date +%s) - $(stat -c %Y "$LOCK" 2>/dev/null || date +%s) ))
    if [ "$age" -gt 10800 ]; then echo "breaking a stale server lock" >> "$LOG"; rm -rf "$LOCK"; continue; fi
    sleep 20
done
basename "$PWD" > "$LOCK/owner"
trap 'rm -rf "$LOCK"' EXIT INT TERM
echo "lock taken $(date -u +%H:%M:%S)" >> "$LOG"
tools/review/deposit-audit-previews.sh "$@" >> "$LOG" 2>&1
echo "DONE $(date -u +%H:%M:%S)" >> "$LOG"

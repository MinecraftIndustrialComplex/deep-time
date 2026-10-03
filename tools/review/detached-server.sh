#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Run a command detached on the test machine under its server lock, so a dropped ssh connection while
# waiting for the lock (an hour or more when the machine is busy) does not lose the job. Start it with
#
#   tools/remote/mac.sh --pack -- tools/review/detached-server.sh <log> <command> [args...]
#
# (no --server: the lock is taken here, in the background). The log is the file to read afterwards;
# it ends with "DETACHED-EXIT <status>". Same lock directory and stale rule as mac.sh --server.
set -uo pipefail
log="$1"; shift
repo="$(cd "$(dirname "$0")/../.." && pwd)"
lock=/Volumes/Graphide/ban/deeptime-cache/server.lock
cd "$repo"
setsid nohup bash -c '
  lock="$1"; log="$2"; name="$3"; shift 3
  while ! mkdir "$lock" 2>/dev/null; do
    age=$(( $(date +%s) - $(stat -c %Y "$lock" 2>/dev/null || date +%s) ))
    if [ "$age" -gt 10800 ]; then echo "breaking a stale server lock"; rm -rf "$lock"; continue; fi
    echo "waiting for the server lock (held by $(cat "$lock/owner" 2>/dev/null))"; sleep 30
  done
  echo "$name" > "$lock/owner"; trap "rm -rf \"$lock\"" EXIT INT TERM
  "$@"; st=$?
  echo "DETACHED-EXIT $st"
' _ "$lock" "$log" "$(basename "$repo")" "$@" > "$log" 2>&1 < /dev/null &
echo "detached, log $log"

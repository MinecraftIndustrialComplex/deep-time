#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Run a command that boots a server detached on the test machine, so a dropped ssh connection while
# it waits for admission (tools/remote/vm/admit.py: it starts when its load fits) does not lose the
# job. Start it with
#
#   tools/remote/mac.sh --pack -- tools/review/detached-server.sh <log> <command> [args...]
#
# The job is admitted as a server job (its own ports and folder) of the kind its command classifies
# as. The log is the file to read afterwards; it ends with "DETACHED-EXIT <status>".
set -uo pipefail
log="$1"; shift
repo="$(cd "$(dirname "$0")/../.." && pwd)"
admit="${DEEPTIME_ADMIT:-/usr/local/lib/deeptime/admit.py}"
cd "$repo"
setsid nohup bash -c '
  admit="$1"; log="$2"; name="$3"; shift 3
  if [ -f "$admit" ]; then
    env -u DEEPTIME_ADMIT_JOB -u DEEPTIME_ADMIT_PID python3 "$admit" run --kind auto --server --label "$name" -- "$@"
  else
    "$@"
  fi
  st=$?
  echo "DETACHED-EXIT $st"
' _ "$admit" "$log" "$(basename "$repo")" "$@" > "$log" 2>&1 < /dev/null &
echo "detached, log $log"

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Runs a long job on the test machine detached from the ssh session that started it, so it
# survives the laptop session (which has crashed mid-run more than once). Use it through mac.sh:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start <name> [-C <dir>] [--kind K] <command words...>
#       starts the command (from the repo root, or <dir> under it, in the dev shell mac.sh
#       entered) in its own session; output goes to <log dir>/<name>.log, which ends with a line
#       EXIT=<status>. mac.sh joins its arguments into one line, so the command is plain words:
#       no quotes, no && and no globs (Gradle's --tests takes a full class name). The job runs
#       through the VM's admission tool (tools/remote/vm/admit.py; kind K, default classified from
#       the command): it starts when its load fits, and its log says why while it waits
#   tools/remote/mac.sh --no-sync -- tools/remote/detach.sh wait <name> [seconds] [lines]
#       waits (default 540 s, under a tool call's limit) for the job to end, then prints the log's
#       tail; exit 0 when the job ended with status 0, 1 when it failed, 3 when it is still running
#   tools/remote/mac.sh --no-sync -- tools/remote/detach.sh tail <name> [lines]
#
# The log dir is $DEEPTIME_DETACH_LOGS, default <repo>/target/detached (target/ is never synced
# or committed). `run` keeps the job in the foreground (mac.sh admits it); it writes the same log,
# so a dropped session loses the wait, not the output:
#   tools/remote/mac.sh -- tools/remote/detach.sh run <name> [-C <dir>] <command words...>
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
LOGS="${DEEPTIME_DETACH_LOGS:-$REPO/target/detached}"
mkdir -p "$LOGS"
usage() { echo "usage: detach.sh start|run <name> [-C <dir>] [--kind K] <command words...> | wait <name> [seconds] [lines] | tail <name> [lines]" >&2; exit 2; }
verb="${1:-}"; name="${2:-}"
[ -n "$verb" ] && [ -n "$name" ] || usage
log="$LOGS/$name.log"

dir="$REPO"
cmd=""
kind=auto
if [ "$verb" = start ] || [ "$verb" = run ]; then
    shift 2
    if [ "${1:-}" = -C ]; then dir="$REPO/$2"; shift 2; fi
    if [ "${1:-}" = --kind ]; then kind="$2"; shift 2; fi
    [ $# -gt 0 ] || usage
    cmd="$*"
fi
ADMIT="${DEEPTIME_ADMIT:-/usr/local/lib/deeptime/admit.py}"

case "$verb" in
    start)
        : > "$log"
        cd "$dir" || exit 2
        # Close every inherited descriptor above 2 (the nix sentry lock, a daemon socket): ssh
        # keeps the calling mac.sh waiting until nothing holds the session's descriptors, and the
        # job inherited those (2026-10-02: the call returned only when the job ended).
        job="$cmd"
        # Admitted on its own (not as part of whatever started it, which ends now).
        [ -f "$ADMIT" ] && job="env -u DEEPTIME_ADMIT_JOB -u DEEPTIME_ADMIT_PID python3 $ADMIT run --kind $(printf '%q' "$kind") --label $(printf '%q' "$name") -- bash -c $(printf '%q' "$cmd")"
        setsid nohup bash -c "for n in 3 4 5 6 7 8 9; do eval \"exec \$n>&-\"; done; $job; echo EXIT=\$?" >> "$log" 2>&1 < /dev/null &
        echo "detach.sh: started '$name' (pid $!), log $log"
        ;;
    run)
        : > "$log"
        cd "$dir" || exit 2
        bash -c "$cmd; echo EXIT=\$?" >> "$log" 2>&1 < /dev/null
        tail -n 40 "$log"
        grep -q '^EXIT=0$' "$log"
        ;;
    wait)
        limit="${3:-540}"
        end=$(( $(date +%s) + limit ))
        while ! grep -q '^EXIT=' "$log" 2>/dev/null; do
            if [ "$(date +%s)" -ge "$end" ]; then
                tail -n 15 "$log" 2>/dev/null
                echo "detach.sh: '$name' is still running"
                exit 3
            fi
            sleep 5
        done
        tail -n "${4:-60}" "$log"
        grep -q '^EXIT=0$' "$log"
        ;;
    tail)
        tail -n "${3:-60}" "$log"
        ;;
    *)
        usage ;;
esac

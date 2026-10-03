#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The laptop's two port forwards to the test VM, as transient systemd user units:
#   deeptime-workbench-tunnel   http://localhost:8642/   → the workbench on the VM
#   deeptime-play-tunnel        localhost:25765          → the play server on the VM
#
#   tools/remote/tunnels.sh [start]   (re)create both units (after a laptop restart or a desktop crash)
#   tools/remote/tunnels.sh status
#
# Since 2026-10-03 the VM deallocates itself after 30 idle minutes (tools/remote/vm/idle-watcher.py),
# so the tunnels must outlast a VM that is down for hours without spinning: ssh gives up a connect
# after 20 s, systemd restarts it after 15 s, backing off to one attempt every 2 minutes, and never
# stops retrying (no start limit). When the VM comes back the tunnels reconnect by themselves within
# about two minutes. They never start the VM: only the remote scripts (vm-access.sh) do, when an
# agent needs it. Their sessions run no command (`ssh -N`), so the idle watcher does not count them.
set -euo pipefail
HOST="${DEEPTIME_MAC:-ban@20.120.160.3}"
SSH_BIN=/run/current-system/sw/bin/ssh
[ -x "$SSH_BIN" ] || SSH_BIN="$(command -v ssh)"

tunnel() {   # <unit> <port>
    local unit="$1" port="$2"
    systemctl --user stop "$unit" 2>/dev/null || true
    systemctl --user reset-failed "$unit" 2>/dev/null || true
    systemd-run --user --quiet --unit "$unit" --description "Deep Time: localhost:$port to the test VM" \
        -p Restart=always -p RestartSec=15s -p RestartSteps=4 -p RestartMaxDelaySec=120s \
        -p StartLimitIntervalSec=0 \
        "$SSH_BIN" -N -o BatchMode=yes -o ExitOnForwardFailure=yes -o ConnectTimeout=20 \
        -o ServerAliveInterval=30 -o ServerAliveCountMax=3 \
        -L "127.0.0.1:$port:127.0.0.1:$port" "$HOST"
    echo "tunnels.sh: $unit (localhost:$port) $(systemctl --user is-active "$unit")"
}

case "${1:-start}" in
    start)
        tunnel deeptime-workbench-tunnel 8642
        tunnel deeptime-play-tunnel 25765 ;;
    status)
        for u in deeptime-workbench-tunnel deeptime-play-tunnel; do
            echo "$u: $(systemctl --user is-active "$u" 2>/dev/null || true), restarts: $(systemctl --user show -p NRestarts --value "$u" 2>/dev/null)"
        done ;;
    *) echo "usage: tunnels.sh [start|status]" >&2; exit 2 ;;
esac

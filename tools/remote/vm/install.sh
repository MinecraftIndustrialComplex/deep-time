#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Installs the test VM's services (Ben, 2026-10-03: auto-stop when idle). Run on the VM, from the
# laptop:
#
#   tools/remote/mac.sh -- tools/remote/vm/install.sh [--switch] [--idle-minutes N]
#
#   deeptime-idle.service       the idle watcher (idle-watcher.py): deallocates the VM after N idle
#                               minutes (default 30; /etc/default/deeptime-idle)
#   deeptime-workbench.service  `deeptime watch` on 127.0.0.1:8642, at boot
#   deeptime-play.service       the play server (port 25765; its folder in /etc/default/deeptime-play)
#   admit.py                    the admission tool every heavy job goes through (not a service)
#
# All three are enabled (they start at boot). The watcher is restarted on every install; the
# workbench and the play server are not touched unless --switch is given, which hands a hand-started
# workbench or play server over to its service, and refuses while either is in use (a run or preview
# in progress, a player online). Existing /etc/default files are kept; --idle-minutes rewrites that
# one setting (a short value is for the end-to-end test only).
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
# mac.sh runs this in the dev shell, whose LD_LIBRARY_PATH breaks /usr/bin/pgrep (nix libsystemd).
unset LD_LIBRARY_PATH
LIB=/usr/local/lib/deeptime
PLAY_DIR_DEFAULT=/data/ban/mic/mods/play-fa09e8b/smoke/mic/server
SWITCH=0; IDLE=""
while [ $# -gt 0 ]; do
    case "$1" in
        --switch) SWITCH=1; shift ;;
        --idle-minutes) IDLE="$2"; shift 2 ;;
        *) echo "usage: install.sh [--switch] [--idle-minutes N]" >&2; exit 2 ;;
    esac
done

sudo install -d -m 0755 "$LIB"
sudo install -m 0755 "$HERE/idle-watcher.py" "$LIB/idle-watcher.py"
# The admission tool (Ben, 2026-10-03: load-based admission): mac.sh, the server launchers and
# detach.sh call $LIB/admit.py; `deeptime-admit` is the same on PATH. Its state lives in
# deeptime-cache/admit/, the server jobs' folders in deeptime-cache/servers/. Installing a new
# version while jobs run is safe: each running admit.py keeps the code it started with, and the
# state file's format is shared.
sudo install -m 0755 "$HERE/admit.py" "$LIB/admit.py"
sudo ln -sfn "$LIB/admit.py" /usr/local/bin/deeptime-admit
install -d -m 0775 /data/ban/deeptime-cache/admit /data/ban/deeptime-cache/servers
if [ ! -e /etc/default/deeptime-admit ]; then
    sudo tee /etc/default/deeptime-admit >/dev/null <<'EOF'
# tools/remote/vm/admit.py (deep-time repo). Read by every admit.py call; no restart needed.
# OVERCOMMIT=0.25       start a job when its estimate fits in what is free plus this share
# RESERVE_CORES=4       headroom kept for the play server, a workbench run and ssh
# RESERVE_MEM_GIB=15
# STARVE_MIN=15         after this wait, nothing starts ahead of the oldest waiting job
EOF
fi
sudo install -m 0755 "$HERE/play-server.sh" "$LIB/play-server.sh"
for u in deeptime-idle deeptime-workbench deeptime-play; do
    sudo install -m 0644 "$HERE/$u.service" "/etc/systemd/system/$u.service"
done
if [ ! -e /etc/default/deeptime-idle ]; then
    sudo tee /etc/default/deeptime-idle >/dev/null <<EOF
# tools/remote/vm/idle-watcher.py (deep-time repo). Restart deeptime-idle after a change.
IDLE_MINUTES=30
INTERVAL=60
LOAD_BUSY=4.0
DRY_RUN=0
PLAY_DIR=$PLAY_DIR_DEFAULT
EOF
fi
if [ -n "$IDLE" ]; then
    sudo sed -i "s/^IDLE_MINUTES=.*/IDLE_MINUTES=$IDLE/" /etc/default/deeptime-idle
fi
[ -e /etc/default/deeptime-play ] || echo "PLAY_DIR=$PLAY_DIR_DEFAULT" | sudo tee /etc/default/deeptime-play >/dev/null
sudo systemctl daemon-reload
sudo systemctl enable deeptime-idle.service deeptime-workbench.service deeptime-play.service 2>&1 | grep -v '^Created symlink' || true
sudo systemctl restart deeptime-idle.service
echo "install.sh: deeptime-idle restarted ($(grep '^IDLE_MINUTES=' /etc/default/deeptime-idle))"

# shellcheck disable=SC1091
. /etc/default/deeptime-play
main_pid() { /usr/bin/systemctl show -p MainPID --value "$1"; }
hand_workbench() {
    local svc p; svc="$(main_pid deeptime-workbench)"
    for p in $(/usr/bin/pgrep -f '^/data/ban/deeptime-cache/workbench/bin/deeptime watch --bind 127.0.0.1 --port 8642' || true); do
        [ "$p" != "$svc" ] && echo "$p"
    done
}
hand_play() {
    local svc p; svc="$(main_pid deeptime-play)"
    for p in $(/usr/bin/pgrep -x java || true); do
        [ "$p" != "$svc" ] && [ "$(sudo readlink "/proc/$p/cwd" 2>/dev/null)" = "$PLAY_DIR" ] && echo "$p"
    done
}
wait_gone() { local _; for _ in $(seq 1 "$2"); do kill -0 "$1" 2>/dev/null || return 0; sleep 1; done; return 1; }

if [ "$SWITCH" = 1 ]; then
    if ! sudo env PLAY_DIR="$PLAY_DIR" python3 "$LIB/idle-watcher.py" --services; then
        echo "install.sh: not switching now; run --switch again when nothing is in progress" >&2
        exit 3
    fi
    for p in $(hand_workbench); do
        echo "install.sh: stopping the hand-started workbench (pid $p)"
        kill -TERM "$p"; wait_gone "$p" 30 || kill -KILL "$p"
    done
    for p in $(hand_play); do
        echo "install.sh: stopping the hand-started play server (pid $p; SIGTERM saves the world)"
        kill -TERM "$p"; wait_gone "$p" 180 || { echo "install.sh: play server $p did not stop" >&2; exit 1; }
    done
    sudo systemctl start deeptime-workbench.service deeptime-play.service
fi
for u in deeptime-idle deeptime-workbench deeptime-play; do
    printf '%-20s %s, %s (pid %s)\n' "$u" "$(systemctl is-enabled "$u")" "$(systemctl is-active "$u" || true)" "$(main_pid "$u")"
done
[ -z "$(hand_workbench)" ] || echo "note: a hand-started workbench runs outside its service (pid $(hand_workbench)); --switch hands it over"
[ -z "$(hand_play)" ] || echo "note: a hand-started play server runs outside its service (pid $(hand_play)); --switch hands it over"

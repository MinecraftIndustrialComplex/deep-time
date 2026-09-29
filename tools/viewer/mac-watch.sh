#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The Deep Time workbench on Ben's M6 Mac mini (P1-20; decisions log 2026-09-28): builds
# `deeptime` there, starts `deeptime watch` detached and puts it on the tailnet only. Run from
# the laptop.
#
#   tools/viewer/mac-watch.sh [start] [--preset P --start S --length-myr L] [--port N]
#                             [--keep N] [--threads N] [--every-myr M] [--no-build] [--tv]
#   tools/viewer/mac-watch.sh stop [--port N]      stop the server, its tailnet forward, the TV's caffeinate
#   tools/viewer/mac-watch.sh status [--port N]
#   tools/viewer/mac-watch.sh tv [--port N]        open the running workbench fullscreen on the Mac's screen
#
#   --preset …    also start a run right away (a built-in name, a file on the Mac or a share code);
#                 without it the page's editor starts runs
#   --port        default 8642 → http://mac-mini:8642/
#   --keep        finished runs kept on the SSD (default 8, ~1 GB each for a 500-Myr 16k run)
#   --threads     Rayon threads per run (default all 12; the Mac is shared)
#   --no-build    skip the sync and the release build (the Mac's binary is current)
#   --tv          also open the page fullscreen in Safari on the Mac's own screen (Ben's TV)
#
# How it is served. The server binds 127.0.0.1 on the Mac and `tailscale serve --bg --tcp <port>`
# forwards the tailnet (100.96.0.7 / mac-mini) to it: nothing listens on the LAN, and no
# freshly built binary needs a macOS firewall exception (binding the 100.x address directly raises
# the firewall's "accept incoming connections?" dialog on the shared screen and hangs until someone
# answers it; tailscaled is already allowed). The server refuses any peer that is neither loopback
# nor a Tailscale address anyway. The tailscale CLI matching the Mac's tailscaled is GC-rooted at
# deeptime-cache/tailscale. Nothing is published; there is no auth beyond the tailnet.
#
# It boots no Minecraft, so it does not take mac.sh's --server lock. The binary is copied to
# deeptime-cache/workbench/bin/ (later syncs and builds cannot pull it out from under the server);
# runs live in deeptime-cache/workbench/runs/ and survive restarts; the pid is in
# deeptime-cache/workbench/watch-<port>.pid, the log beside it.
#
# --tv / tv: graphide's vncdrive.sh (VNCDRIVE overrides the path) reports whether the screen is
# locked; if it is, this waits (up to 15 min) and says so — it never types a password. Then Safari
# opens http://localhost:<port>/?tv over ssh, vncdrive sends ctrl-cmd-F for fullscreen (that takes
# graphide's M6 "drive" lease for a moment; if the M6 is busy with tests it says so and leaves the
# window as it is) and `vncdrive.sh close` drops the tunnel and releases the lease. While the server
# runs, `caffeinate -d -w <server pid>` keeps the display awake; `stop` ends it.

set -euo pipefail
cd "$(dirname "$0")/../.."

HOST="${DEEPTIME_MAC:-ban@mac-mini}"
TAILNET_NAME="${DEEPTIME_MAC_TAILNET:-mac-mini}"
CACHE=/Volumes/Graphide/ban/deeptime-cache
WB="$CACHE/workbench"
REMOTE="/Volumes/Graphide/ban/mic/mods/$(basename "$(git rev-parse --show-toplevel)")"
VNCDRIVE="${VNCDRIVE:-/home/greencheetah/Projects/graphide/graphide-test-audit/utilities/scripts/macmini/vncdrive.sh}"
SSH=(command ssh -n -o BatchMode=yes -o ConnectTimeout=15 "$HOST")

CMD=start
case "${1:-}" in start|stop|status|tv) CMD="$1"; shift ;; esac
PORT=8642; BUILD=1; TV=0; KEEP=8; THREADS=0; EVERY=""; PRESET=();
while [ $# -gt 0 ]; do
    case "$1" in
        --port) PORT="$2"; shift 2 ;;
        --keep) KEEP="$2"; shift 2 ;;
        --threads) THREADS="$2"; shift 2 ;;
        --every-myr) EVERY="$2"; shift 2 ;;
        --preset|--start|--length-myr) PRESET+=("$1" "$2"); shift 2 ;;
        --no-build) BUILD=0; shift ;;
        --tv) TV=1; shift ;;
        -h|--help) sed -n '2,/^set -euo/p' "$0" | sed 's/^# \{0,1\}//' | sed '$d'; exit 0 ;;
        *) echo "mac-watch: unknown argument $1 (see --help)" >&2; exit 2 ;;
    esac
done
[[ "$PORT" =~ ^[0-9]+$ ]] || { echo "mac-watch: --port must be a number" >&2; exit 2; }
URL="http://$TAILNET_NAME:$PORT/"
PIDF="$WB/watch-$PORT.pid"
CAFF="$WB/caffeinate-$PORT.pid"
LOG="$WB/watch-$PORT.log"

# The tailscale CLI of the Mac's tailscaled, GC-rooted (the daemon at /usr/local/bin ships no CLI).
TS_SETUP='
TS='"$CACHE"'/tailscale/bin/tailscale
if [ ! -x "$TS" ]; then
  v=$(/usr/local/bin/tailscaled --version 2>/dev/null | head -1 | tr -d " ")
  p=$(ls -d /nix/store/*-tailscale-"$v" 2>/dev/null | head -1)
  if [ -n "$p" ]; then nix build "$p" --out-link '"$CACHE"'/tailscale >/dev/null
  else nix build nixpkgs#tailscale --out-link '"$CACHE"'/tailscale >/dev/null; fi
fi'

remote() { "${SSH[@]}" "exec zsh -lc $(printf '%q' "$1")"; }

stop_remote() {
    remote "$TS_SETUP
p=\$(cat '$PIDF' 2>/dev/null || true)
if [ -n \"\$p\" ] && ps -p \"\$p\" -o command= | grep -q 'deeptime watch'; then
  kill -TERM \"\$p\"; for i in 1 2 3 4 5 6 7 8 9 10; do ps -p \"\$p\" >/dev/null || break; sleep 0.5; done
  ps -p \"\$p\" >/dev/null && kill -KILL \"\$p\"
  echo \"stopped the workbench (pid \$p)\"
else
  echo 'no workbench running on port $PORT'
fi
rm -f '$PIDF'
c=\$(cat '$CAFF' 2>/dev/null || true)
[ -n \"\$c\" ] && ps -p \"\$c\" -o command= | grep -q caffeinate && kill \"\$c\" 2>/dev/null && echo 'display sleep allowed again'
rm -f '$CAFF'
\"\$TS\" serve --tcp=$PORT off >/dev/null 2>&1 && echo 'tailnet forward of port $PORT removed' || true"
}

tv_open() {
    [ -x "$VNCDRIVE" ] || { echo "mac-watch: no vncdrive.sh at $VNCDRIVE (set VNCDRIVE)" >&2; return 1; }
    local waited=0 state
    while :; do
        state="$("$VNCDRIVE" status 2>&1 | sed -n 's/^screen: //p')"
        case "$state" in
            unlocked*) break ;;
            locked*|login-window*)
                [ "$waited" = 0 ] && echo "mac-watch: the Mac's screen is ${state%% *}; waiting for Ben to unlock it (never typing a password)"
                [ "$waited" -ge 900 ] && { echo "mac-watch: still ${state%% *} after 15 min; the workbench keeps running at $URL" >&2; return 1; }
                sleep 10; waited=$((waited + 10)) ;;
            *) echo "mac-watch: cannot read the screen state (vncdrive.sh status: ${state:-no answer})" >&2; return 1 ;;
        esac
    done
    local pid
    pid="$(remote "cat '$PIDF' 2>/dev/null")" || true
    [ -n "$pid" ] || { echo "mac-watch: no workbench on port $PORT; start it first" >&2; return 1; }
    remote "c=\$(cat '$CAFF' 2>/dev/null); [ -n \"\$c\" ] && ps -p \"\$c\" >/dev/null || { nohup /usr/bin/caffeinate -d -w $pid >/dev/null 2>&1 < /dev/null & echo \$! > '$CAFF'; }"
    "${SSH[@]}" "/usr/bin/open -a Safari 'http://localhost:$PORT/?tv'"
    sleep 4
    if "$VNCDRIVE" key ctrl-super-f >/dev/null 2>&1; then echo "mac-watch: fullscreen on the Mac's screen"
    else echo "mac-watch: opened in Safari, but could not send ctrl-cmd-F (the M6 may be busy: vncdrive.sh status)" >&2; fi
    "$VNCDRIVE" close >/dev/null 2>&1 || true
}

case "$CMD" in
stop) stop_remote; exit 0 ;;
status)
    remote "$TS_SETUP
p=\$(cat '$PIDF' 2>/dev/null || true)
if [ -n \"\$p\" ] && ps -p \"\$p\" >/dev/null; then echo \"workbench: running (pid \$p)\"; else echo 'workbench: not running'; fi
\"\$TS\" serve status 2>&1 | sed 's/^/tailscale: /'
tail -3 '$LOG' 2>/dev/null | sed 's/^/log: /'"
    curl -sS -m 5 "${URL}api/runs" 2>/dev/null | python3 -c 'import json,sys; j=json.load(sys.stdin); print("runs:", len(j["runs"]), "kept; running:", j["running"], "; queue:", len(j["queue"]))' 2>/dev/null || echo "not answering at $URL"
    exit 0 ;;
tv) tv_open; exit $? ;;
esac

if [ "$BUILD" = 1 ]; then
    tools/remote/mac.sh -- 'cargo build --release -p deeptime-cli 2>&1 | tail -2'
fi
stop_remote >/dev/null
ARGS=(watch --bind 127.0.0.1 --port "$PORT" --runs "$WB/runs" --keep "$KEEP" --threads "$THREADS")
[ -n "$EVERY" ] && ARGS+=(--every-myr "$EVERY")
ARGS+=("${PRESET[@]}")
remote "$TS_SETUP
set -e
mkdir -p '$WB/bin' '$WB/runs'
cp '$REMOTE/target/release/deeptime' '$WB/bin/deeptime.new' && mv '$WB/bin/deeptime.new' '$WB/bin/deeptime'
cd '$WB'
nohup '$WB/bin/deeptime' $(printf '%q ' "${ARGS[@]}") > '$LOG' 2>&1 < /dev/null &
echo \$! > '$PIDF'
for i in 1 2 3 4 5 6 7 8 9 10; do /usr/bin/nc -z 127.0.0.1 $PORT 2>/dev/null && break; sleep 0.5; done
/usr/bin/nc -z 127.0.0.1 $PORT || { echo 'the workbench did not start:'; cat '$LOG'; exit 1; }
\"\$TS\" serve --bg --tcp $PORT tcp://127.0.0.1:$PORT >/dev/null
head -3 '$LOG'"
echo "mac-watch: the workbench is at $URL (tailnet only); stop it with tools/viewer/mac-watch.sh stop"
if [ "$TV" = 1 ]; then tv_open; fi

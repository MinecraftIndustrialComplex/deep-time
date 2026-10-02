#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Distant Horizons far terrain to a joined client (P3-04; Ben 2026-10-02, "Mixin around TW's
# guard"): run on the test VM, under the server lock, from the repo root:
#
#   tools/remote/mac.sh --server -- tools/remote/detach.sh run dh-client tools/client/dh-client-check.sh [profile]
#
# profile: an installed smoke set with Distant Horizons (default dh: standalone + DH at the MIC
# pack's pin; install it first with tools/smoke/install.sh dh --preset earthlike_quick_16k ...).
# It starts that server, then a real client (portablemc, NeoForge, the server's own mod jars,
# software GL on a private Xvfb display) that joins it, and records from the server's
# `/deeptime probe dh` how many sections Distant Horizons asked Deep Time's generator for, and over
# what x extent: with no /dh pregen, every section asked is a client's request. Then the player is
# moved beside the seam (x = C/2 - 100) and the extent must reach both ends of the lap (the
# client's requests past the seam reach the generator folded into the planet's lap). The
# client's own Distant Horizons database is sized at the end. Nothing is rendered or judged by eye.
#
# Output: smoke/<profile>/client-check.txt (the probe lines and the verdicts); exit 0 when the
# client's requests reached the generator on both sides of the seam.
set -uo pipefail
cd "$(dirname "$0")/../.."
REPO="$PWD"
P="${1:-dh}"
SRV="$REPO/smoke/$P/server"
CL="$REPO/smoke/$P/client"
OUT="$REPO/smoke/$P/client-check.txt"
[ -d "$SRV/mods" ] || { echo "dh-client-check: smoke/$P is not installed (tools/smoke/install.sh $P ...)" >&2; exit 2; }
. "$REPO/smoke/tools.env"
NEO="$(cat "$REPO/smoke/$P/neoforge.version")"
GAME=$(sed -n 's/^server-port=//p' "$SRV/server.properties")
RCONP=$(sed -n 's/^rcon.port=//p' "$SRV/server.properties")
PW=$(sed -n 's/^rcon.password=//p' "$SRV/server.properties")
DISP=:89
: > "$OUT"
say() { echo "$*" | tee -a "$OUT"; }
rcon() { "$MCRCON" -H 127.0.0.1 -P "$RCONP" -p "$PW" -c "$1" 2>&1; }
probe() { rcon "deeptime probe dh" >/dev/null; sleep 1; grep -a '\[deeptime-probe\] {"probe":"dh",' "$SRV/console.log" | tail -1 | sed 's/.*\[deeptime-probe\] //'; }
field() { python3 -c 'import json,sys; d=json.loads(sys.argv[1]); l=[l for l in d.get("levels",[]) if l["level"]=="minecraft:overworld"]; print((l[0] if l else {}).get(sys.argv[2],""))' "$1" "$2"; }

SPID=""; XPID=""; CPID=""
cleanup() {
    [ -n "$CPID" ] && kill "$CPID" 2>/dev/null
    pkill -f -- "--mc-dir $CL/game" 2>/dev/null
    [ -n "$SPID" ] && { rcon stop >/dev/null 2>&1; for i in $(seq 60); do kill -0 "$SPID" 2>/dev/null || break; sleep 1; done; kill "$SPID" 2>/dev/null; }
    [ -n "$XPID" ] && kill "$XPID" 2>/dev/null
}
trap cleanup EXIT

# 1. The server.
say "== server smoke/$P (NeoForge $NEO, port $GAME)"
( cd "$SRV" && exec setsid "$JAVA" @user_jvm_args.txt "@libraries/net/neoforged/neoforge/$NEO/unix_args.txt" --nogui ) > "$SRV/console.log" 2>&1 < /dev/null &
SPID=$!
for i in $(seq 360); do grep -a -q 'RCON running on' "$SRV/console.log" && break; kill -0 $SPID 2>/dev/null || { say "server died"; exit 1; }; sleep 5; done
grep -a -q 'RCON running on' "$SRV/console.log" || { say "server not up after 30 min"; exit 1; }
S0="$(probe)"; say "probe at start: $S0"
C=$(field "$S0" circumference); [ -n "$C" ] || { say "far terrain is not registered: $S0"; exit 1; }
rcon "gamerule doDaylightCycle false" >/dev/null

# 2. The client: the server's own mod jars, an offline profile, software GL.
mkdir -p "$CL/game/mods"
rm -f "$CL/game/mods/"*.jar
cp "$SRV/mods/"*.jar "$CL/game/mods/"
MESA=$(nix build --no-link --print-out-paths nixpkgs#mesa | head -1)
GLVND=$(nix build --no-link --print-out-paths nixpkgs#libglvnd | head -1)
XVFB=$(nix build --no-link --print-out-paths nixpkgs#xorg.xorgserver | head -1)/bin/Xvfb
PMC=$(nix build --no-link --print-out-paths nixpkgs#portablemc | head -1)/bin/portablemc
if ! [ -e /tmp/.X11-unix/X${DISP#:} ]; then
    setsid "$XVFB" $DISP -screen 0 1280x720x24 -nolisten tcp +extension GLX > "$CL/xvfb.log" 2>&1 < /dev/null &
    XPID=$!; sleep 2
fi
say "== client joins 127.0.0.1:$GAME"
DISPLAY=$DISP LD_LIBRARY_PATH="$MESA/lib:$GLVND/lib" __GLX_VENDOR_LIBRARY_NAME=mesa \
    __EGL_VENDOR_LIBRARY_DIRS="$MESA/share/glvnd/egl_vendor.d" LIBGL_DRIVERS_PATH="$MESA/lib/dri" \
    LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe LP_NUM_THREADS=4 \
    setsid "$PMC" --main-dir "$CL/pmc" start "neoforge::$NEO" --mc-dir "$CL/game" --jvm "$JAVA" \
    --jvm-arg=-Xmx6G --jvm-arg=-XX:ActiveProcessorCount=6 -u DeepTimeProbe --resolution 1280x720 \
    --join-server 127.0.0.1 --join-server-port "$GAME" > "$CL/client.log" 2>&1 < /dev/null &
CPID=$!
for i in $(seq 180); do grep -a -q 'DeepTimeProbe joined the game' "$SRV/console.log" && break; kill -0 $CPID 2>/dev/null || { say "client died (smoke/$P/client/client.log)"; exit 1; }; sleep 5; done
grep -a -q 'DeepTimeProbe joined the game' "$SRV/console.log" || { say "client did not join in 15 min"; exit 1; }
rcon "gamemode spectator DeepTimeProbe" >/dev/null
J="$(probe)"; say "probe at join: $J"

# 3. Beside the spawn, then beside the seam.
sleep 180
A="$(probe)"; say "probe after 3 min at the spawn: $A"
X=$(( C / 2 - 100 ))
say "== tp DeepTimeProbe $X 300 0 (seam at x = $(( C / 2 )))"
rcon "tp DeepTimeProbe $X 300 0" >/dev/null
sleep 300
B="$(probe)"; say "probe after 5 min beside the seam: $B"

# 4. Verdicts.
s0=$(field "$J" sections); s1=$(field "$A" sections); s2=$(field "$B" sections); ext=$(field "$B" x_extent)
say "sections asked by the client: at the spawn $(( s1 - s0 )), beside the seam $(( s2 - s1 )); x extent $ext (lap $(( -C / 2 ))..$(( C / 2 )))"
DB=$(find "$CL/game" -path '*Distant_Horizons*' -name '*.sqlite*' -printf '%s %p\n' 2>/dev/null | sort -n | tail -3)
say "client's Distant Horizons databases (bytes):"; say "${DB:-none}"
ok=0
[ $(( s1 - s0 )) -gt 0 ] || { say "FAIL: the client asked for nothing at the spawn"; ok=1; }
[ $(( s2 - s1 )) -gt 0 ] || { say "FAIL: the client asked for nothing beside the seam"; ok=1; }
lo=${ext%..*}; hi=${ext#*..}
if [ "$ext" = none ] || [ "$lo" -gt $(( -C / 2 + 2048 )) ] || [ "$hi" -lt $(( C / 2 - 64 )) ]; then
    say "FAIL: the sections asked do not reach both ends of the lap (east of the seam is x = $(( -C / 2 )) on)"; ok=1
fi
[ $ok = 0 ] && say "PASS: the client's requests reached Deep Time's generator on both sides of the seam"
exit $ok

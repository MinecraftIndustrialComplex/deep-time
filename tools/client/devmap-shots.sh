#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Real-client screenshots and checks of the internal developer map (devmap/, mod id deeptime_devmap)
# on the play world's data, on the test VM. From the laptop:
#
#   tools/remote/mac.sh -- 'cd neoforge && ./gradlew build -x test'        # the two jars (or the full gate)
#   tools/remote/mac.sh -- tools/remote/detach.sh start devmap-shots --kind client:devmap tools/client/devmap-shots.sh [options]
#   tools/remote/mac.sh --no-sync -- tools/remote/detach.sh wait devmap-shots
#   tools/remote/mac.sh --no-sync --pull review/out/devmap-shots -- true
#
# It runs through the admission tool as a server job (kind client:devmap): a dedicated server in the
# job's own folder and ports with three mods (this checkout's Deep Time and dev map jars, and the
# server set's Toroidal World) and a copy of the play world's world data (keyframes linked, not
# copied; nothing of the play folder is written), then a real client (portablemc, the same three jars,
# Mesa llvmpipe on the job's own Xvfb display) that joins it. The player is made an operator over
# RCON after joining (the dev map re-sends its info on the permission change, which turns the teleport
# on), and the client's -Ddeeptime_devmap.shots=true driver (devmap client/ShotDriver) waits for the
# server's two pictures (satellite, drawn by the natives, and elevation), visits the spawn, the wrap
# seam and the north pole by right-clicking the map, checks the readout against Deep Time's F3 line
# and the markers against the projection's inverse, and takes the HUD (with F3), the map and the
# globe at each in the satellite picture, plus the spawn's maps in elevation (devmap-1-spawn-map*-
# elevation.png). The server logs each teleport's surface against the world data's height and water
# (AGREE / DIFFER).
#
#   --name N      output folder review/out/<N>/ (default devmap-shots); replaced on every run
#   --world DIR   the world data (default: the play server's, <PLAY_DIR>/planet/deeptime from
#                 /etc/default/deeptime-play)
#   --set DIR     a server set (smoke/<profile>/server) whose NeoForge install and Toroidal World jar
#                 are used (default: this checkout's smoke/{standalone,terrablender}/server, then the
#                 main checkout's /data/ban/mic/mods/deep-time/smoke/terrablender/server)
#   --settle S    seconds the driver waits at each place for the terrain (default 60)
#   --places P    the driver's places, name:x:z;... (default: spawn, beside the seam, beside the north pole;
#                 on a cube world spawn, beside the slide, inside a quarter-turn border, the north pole,
#                 beside a reachable corner and under a border corner)
#   --level-type T  the server's level type (default: from the world data's manifest, its projection
#                 and circumference: deeptime:planet[_32k], deeptime:planet_cube[_32k] or
#                 deeptime:planet_lambert[_32k], the 32k types above a period of 20,000 blocks)
#
# Output: review/out/<N>/devmap-*.png, shots.txt (the driver's lines), server.txt (the server's
# [devmap] lines), index.html, client.log (the game's log), launcher.log, console.log (the server's). Exit 0
# when the walk finished ("done after" in shots.txt) with no FAIL.
set -uo pipefail
SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
cd "$(dirname "$0")/../.."
REPO="$PWD"
ADMIT="${DEEPTIME_ADMIT:-/usr/local/lib/deeptime/admit.py}"
ARGS=("$@")
NAME=devmap-shots; WORLD=""; SET=""; SETTLE=60; PLACES=""; LEVEL_TYPE=""
while [ $# -gt 0 ]; do
    case "$1" in
        --name) NAME="$2"; shift 2 ;;
        --world) WORLD="$2"; shift 2 ;;
        --set) SET="$2"; shift 2 ;;
        --settle) SETTLE="$2"; shift 2 ;;
        --places) PLACES="$2"; shift 2 ;;
        --level-type) LEVEL_TYPE="$2"; shift 2 ;;
        *) echo "usage: devmap-shots.sh [--name N] [--world DIR] [--set DIR] [--settle S] [--places name:x:z;...] [--level-type T]" >&2; exit 2 ;;
    esac
done
[[ "$NAME" =~ ^[0-9A-Za-z._-]+$ ]] || { echo "devmap-shots: odd --name" >&2; exit 2; }
[ "$(uname -s)" = Linux ] || { echo "devmap-shots: runs on the test VM" >&2; exit 2; }
if [ "${DEEPTIME_ADMIT_PID:-}" != "$$" ] && [ -f "$ADMIT" ]; then
    exec python3 "$ADMIT" run --kind client:devmap --server --label "$(basename "$REPO")" -- "$SELF" "${ARGS[@]}"
fi
# Admitted by someone else (detach.sh) as a job without its own ports and display: refuse, rather
# than boot a server on the set's ports beside other jobs.
if [ -f "$ADMIT" ] && [ -z "${DEEPTIME_SERVER_DIR:-}" ]; then
    echo "devmap-shots: admitted without a server slot; start it with detach.sh start <name> --kind client:devmap" >&2
    exit 2
fi

if [ -z "$WORLD" ]; then
    PLAY_DIR="$(sed -n 's/^PLAY_DIR=//p' /etc/default/deeptime-play 2>/dev/null)"
    WORLD="$PLAY_DIR/planet/deeptime"
fi
[ -f "$WORLD/manifest.json" ] || { echo "devmap-shots: no world data at $WORLD" >&2; exit 1; }
if [ -z "$LEVEL_TYPE" ]; then
    # The level type of the world data's map and size (the server refuses world data of another map).
    LEVEL_TYPE=$(python3 - "$WORLD/manifest.json" <<'PY'
import json, sys
m = json.load(open(sys.argv[1]))
proj = m.get("projection")
kind = proj.get("kind") if isinstance(proj, dict) else "lambert"
big = "_32k" if int((m.get("scale") or {}).get("circumference_blocks", 0)) > 20000 else ""
print({"cube": "deeptime:planet_cube", "lambert": "deeptime:planet_lambert"}.get(kind, "deeptime:planet") + big)
PY
)
fi
[[ "$LEVEL_TYPE" =~ ^[a-z0-9_]+:[a-z0-9_/]+$ ]] || { echo "devmap-shots: odd level type '$LEVEL_TYPE'" >&2; exit 2; }
if [ -z "$SET" ]; then
    for s in "$REPO/smoke/standalone/server" "$REPO/smoke/terrablender/server" /data/ban/mic/mods/deep-time/smoke/terrablender/server; do
        [ -d "$s/mods" ] && { SET="$s"; break; }
    done
fi
[ -d "$SET/mods" ] || { echo "devmap-shots: no server set (--set smoke/<profile>/server)" >&2; exit 1; }
SETDIR="$(dirname "$SET")"
NEO="$(cat "$SETDIR/neoforge.version")"
TOOLS="$SETDIR/../tools.env"; [ -f "$TOOLS" ] || TOOLS="$REPO/smoke/tools.env"
. "$TOOLS"
DT=""; DM=""
for j in "$REPO"/neoforge/build/libs/deeptime-*.jar; do case "$j" in *-sources.jar|*-slim.jar) ;; *) [ -f "$j" ] && DT="$j" ;; esac; done
for j in "$REPO"/devmap/build/libs/deeptime_devmap-*.jar; do case "$j" in *-sources.jar) ;; *) [ -f "$j" ] && DM="$j" ;; esac; done
TW=""; for j in "$SET"/mods/toroidal_world*.jar; do [ -f "$j" ] && TW="$j"; done
[ -n "$DT" ] && [ -n "$DM" ] && [ -n "$TW" ] || { echo "devmap-shots: missing a jar (deeptime '$DT', devmap '$DM', toroidal world '$TW'); cd neoforge && ./gradlew build -x test" >&2; exit 1; }

OUT="$REPO/review/out/$NAME"
WORK="${DEEPTIME_SERVER_DIR:-$REPO/target/devmap-shots}"
SRV="$WORK/server"
CL="$REPO/smoke/devmap-client"
PMCDIR="${DEEPTIME_PMC_DIR:-/data/ban/deeptime-cache/pmc-devmap}"
rm -rf "$OUT"; mkdir -p "$OUT" "$WORK"
LOG="$OUT/runner.log"
say() { echo "devmap-shots: $*" | tee -a "$LOG"; }
say "world $WORLD ($LEVEL_TYPE); set $SET (NeoForge $NEO); jars $(basename "$DT") $(basename "$DM") $(basename "$TW")"

# 1. The server: the set's NeoForge in the job's folder, three mods, the world data copied (keyframes linked).
rm -rf "$SRV"
python3 "$REPO/tools/smoke/instance.py" make --set "$SET" --dest "$SRV" >/dev/null || { say "cannot copy the set"; exit 1; }
rm -rf "$SRV/mods" "$SRV/planet" "$SRV/world" "$SRV/config/voicechat"; mkdir -p "$SRV/mods" "$SRV/planet"
cp "$DT" "$DM" "$TW" "$SRV/mods/"
rsync -a --exclude=/keyframes/ --exclude=/epochs/ "$WORLD/" "$SRV/planet/deeptime/"
if [ -d "$WORLD/keyframes" ]; then
    ln -s "$WORLD/keyframes" "$SRV/planet/deeptime/keyframes"
    # Minecraft refuses a world folder with a symlink unless the server's allowed_symlinks.txt lists
    # its target (a plain line is a prefix of the link's target).
    printf '%s\n' "$WORLD/keyframes" > "$SRV/allowed_symlinks.txt"
fi
GAME=$(sed -n 's/^server-port=//p' "$SRV/server.properties"); GAME=${GAME:-25565}
RCONP=$(sed -n 's/^rcon.port=//p' "$SRV/server.properties"); RCONP=${RCONP:-25575}
PW=$(python3 -c "import secrets; print(secrets.token_hex(12))")
cat > "$SRV/server.properties" <<EOF
server-ip=127.0.0.1
server-port=$GAME
enable-rcon=true
rcon.port=$RCONP
rcon.password=$PW
online-mode=false
level-type=$LEVEL_TYPE
level-name=planet
level-seed=54324317120568257
spawn-protection=0
max-tick-time=-1
view-distance=8
simulation-distance=6
gamemode=creative
force-gamemode=true
difficulty=peaceful
motd=Deep Time dev map shots
EOF
printf '%s\n' '-Xmx8G' '-XX:+UseG1GC' > "$SRV/user_jvm_args.txt"
echo 'eula=true' > "$SRV/eula.txt"
rcon() { "$MCRCON" -H 127.0.0.1 -P "$RCONP" -p "$PW" -c "$1" 2>&1; }

SPID=""; XPID=""; CPID=""; PAPID=""; PADIR=""
cleanup() {
    [ -n "$CPID" ] && kill "$CPID" 2>/dev/null
    pkill -f -- "deeptime_devmap.shots=true" 2>/dev/null
    [ -n "$SPID" ] && { rcon stop >/dev/null 2>&1; for i in $(seq 60); do kill -0 "$SPID" 2>/dev/null || break; sleep 1; done; kill "$SPID" 2>/dev/null; }
    [ -n "$XPID" ] && kill "$XPID" 2>/dev/null
    [ -n "$PAPID" ] && kill "$PAPID" 2>/dev/null; [ -n "$PADIR" ] && rm -rf "$PADIR"
    [ -f "$SRV/console.log" ] && cp "$SRV/console.log" "$OUT/console.log"
}
trap cleanup EXIT
trap 'exit 130' INT; trap 'exit 143' TERM; trap 'exit 129' HUP

say "server on 127.0.0.1:$GAME (RCON $RCONP)"
( cd "$SRV" && exec setsid "$JAVA" @user_jvm_args.txt "@libraries/net/neoforged/neoforge/$NEO/unix_args.txt" --nogui ) > "$SRV/console.log" 2>&1 < /dev/null &
SPID=$!
for i in $(seq 360); do grep -a -q 'RCON running on' "$SRV/console.log" && break; kill -0 $SPID 2>/dev/null || { say "server died ($(grep -a -m3 -e ERROR -e Exception "$SRV/console.log" | head -3))"; exit 1; }; sleep 5; done
grep -a -q 'RCON running on' "$SRV/console.log" || { say "server not up after 30 min"; exit 1; }
rcon "gamerule doDaylightCycle false" >/dev/null; rcon "time set noon" >/dev/null; rcon "gamerule doWeatherCycle false" >/dev/null
say "server up; $(grep -a '\[devmap\] [a-z]* picture' "$SRV/console.log" | sed 's/.*\[devmap\] //' | tr '\n' ';' || true)"

# 2. The client: the server's three jars, an offline profile, software GL on the job's display.
mkdir -p "$CL/game/mods" "$CL/game/config"
rm -rf "$CL/game/mods/"*.jar "$CL/game/screenshots" "$CL/game/crash-reports" "$CL/game/logs"
cp "$SRV/mods/"*.jar "$CL/game/mods/"
cat > "$CL/game/options.txt" <<'EOF'
onboardAccessibility:false
pauseOnLostFocus:false
guiScale:2
narrator:0
tutorialStep:none
joinedFirstServer:true
skipMultiplayerWarning:true
fullscreen:false
soundCategory_master:0.0
soundCategory_music:0.0
renderDistance:8
simulationDistance:6
EOF
printf 'earlyWindowControl = false\n' > "$CL/game/config/fml.toml"
MESA=$(nix build --no-link --print-out-paths nixpkgs#mesa | head -1)
GLVND=$(nix build --no-link --print-out-paths nixpkgs#libglvnd | head -1)
XVFB=$(nix build --no-link --print-out-paths nixpkgs#xorg.xorgserver | head -1)/bin/Xvfb
PMC=$(nix build --no-link --print-out-paths nixpkgs#portablemc | head -1)/bin/portablemc
PA=$(nix build --no-link --print-out-paths nixpkgs#pulseaudio | head -1)/bin/pulseaudio
DISP="${DEEPTIME_DISPLAY:-:93}"
export DISPLAY=$DISP LD_LIBRARY_PATH="$MESA/lib:$GLVND/lib" __GLX_VENDOR_LIBRARY_NAME=mesa \
    __EGL_VENDOR_LIBRARY_DIRS="$MESA/share/glvnd/egl_vendor.d" LIBGL_DRIVERS_PATH="$MESA/lib/dri" \
    LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe LP_NUM_THREADS=4
PADIR=$(mktemp -d /tmp/devmap-pulse.XXXXXX)
PULSE_RUNTIME_PATH="$PADIR" HOME="$PADIR" setsid "$PA" -n --daemonize=no --exit-idle-time=-1 --use-pid-file=no \
    --disable-shm=yes --load="module-native-protocol-unix socket=$PADIR/native auth-anonymous=1" \
    --load="module-null-sink" > "$WORK/pulse.log" 2>&1 < /dev/null &
PAPID=$!; sleep 2
export PULSE_SERVER="unix:$PADIR/native" ALSOFT_DRIVERS=pulse
rm -f "/tmp/.X${DISP#:}-lock" "/tmp/.X11-unix/X${DISP#:}" 2>/dev/null
setsid "$XVFB" "$DISP" -screen 0 1600x900x24 -nolisten tcp +extension GLX > "$WORK/xvfb.log" 2>&1 < /dev/null &
XPID=$!; sleep 3
kill -0 "$XPID" 2>/dev/null || { say "Xvfb $DISP did not start"; exit 1; }
# The launcher's output (until FML hands logging to log4j) and the game's own log, where the driver writes.
CLOG="$OUT/launcher.log"
GLOG="$CL/game/logs/latest.log"
say "client joins (display $DISP, 1600x900, GUI scale 2)"
setsid "$PMC" --main-dir "$PMCDIR" start "neoforge::$NEO" --mc-dir "$CL/game" --jvm "$JAVA" \
    --jvm-arg=-Xmx6G --jvm-arg=-XX:ActiveProcessorCount=6 \
    --jvm-arg=-Ddeeptime_devmap.shots=true "--jvm-arg=-Ddeeptime_devmap.shots.settle=$SETTLE" \
    ${PLACES:+"--jvm-arg=-Ddeeptime_devmap.shots.places=$PLACES"} \
    -u DevMapShot --resolution 1600x900 \
    --join-server 127.0.0.1 --join-server-port "$GAME" > "$CLOG" 2>&1 < /dev/null &
CPID=$!
joined=0
for i in $(seq 240); do
    grep -a -q 'DevMapShot joined the game' "$SRV/console.log" && { joined=1; break; }
    kill -0 $CPID 2>/dev/null || break
    grep -a -q -e 'Preparing crash report' -e 'Failed to find a valid GLFW profile' "$CLOG" && break
    sleep 5
done
[ $joined = 1 ] || { say "the client did not join ($(tail -3 "$CLOG" | tr '\n' ' '))"; exit 1; }
say "joined; op DevMapShot: $(rcon 'op DevMapShot')"

# 3. The walk: the driver quits the client when done.
start=$(date +%s); last=0; st=""
while kill -0 "$CPID" 2>/dev/null; do
    sleep 5
    now=$(date +%s)
    if [ $((now - start)) -gt 2400 ]; then say "timed out after 40 min"; st=124; break; fi
    if grep -a -q 'Preparing crash report' "$CLOG" "$GLOG" 2>/dev/null; then say "the client crashed"; sleep 10; st=1; break; fi
    if [ $((now - last)) -ge 60 ]; then
        say "$((now - start))s: $(grep -aF '[devmap-shot]' "$GLOG" 2>/dev/null | tail -1 | sed 's/^.*\[devmap-shot\] //' | cut -c1-200)"
        last=$now
    fi
done
[ -z "$st" ] && { wait "$CPID"; st=$?; }
CPID=""
grep -aF '[devmap-shot]' "$GLOG" | sed 's/^.*\[devmap-shot\] //' > "$OUT/shots.txt"
grep -aF '[devmap]' "$SRV/console.log" | sed 's/^.*\[devmap\] //' > "$OUT/server.txt"
grep -aF '[devmap]' "$GLOG" | sed 's/^.*\[devmap\] //' >> "$OUT/server.txt"
cp "$CL/game/screenshots/"devmap-*.png "$OUT/" 2>/dev/null
[ -f "$GLOG" ] && cp "$GLOG" "$OUT/client.log"
[ -d "$CL/game/crash-reports" ] && cp -R "$CL/game/crash-reports" "$OUT/crash-reports"

# 4. A contact sheet.
python3 - "$OUT" <<'PY'
import html, os, sys
out = sys.argv[1]
pngs = sorted(f for f in os.listdir(out) if f.endswith(".png"))
shots = open(os.path.join(out, "shots.txt")).read() if os.path.exists(os.path.join(out, "shots.txt")) else ""
server = open(os.path.join(out, "server.txt")).read() if os.path.exists(os.path.join(out, "server.txt")) else ""
with open(os.path.join(out, "index.html"), "w") as f:
    f.write("<!doctype html><meta charset=utf-8><title>Dev map shots</title>"
            "<style>body{font:14px system-ui;background:#111;color:#ddd;margin:16px}img{max-width:100%;border:1px solid #444}"
            "figure{margin:0 0 24px}pre{white-space:pre-wrap;background:#1a1a1a;padding:8px;font-size:12px}</style>"
            "<h1>Deep Time dev map: real-client shots</h1>")
    for p in pngs:
        f.write(f"<figure><img src='{html.escape(p)}' alt='{html.escape(p)}'><figcaption>{html.escape(p)}</figcaption></figure>")
    f.write("<h2>Driver</h2><pre>" + html.escape(shots) + "</pre><h2>Server</h2><pre>" + html.escape(server) + "</pre>")
PY
fails=$(grep -c 'FAIL' "$OUT/shots.txt")
done_line=$(grep -c '^done after' "$OUT/shots.txt")
n=$(ls "$OUT"/devmap-*.png 2>/dev/null | wc -l)
say "client exited $st; $n PNGs, $fails FAIL lines, walk finished: $([ "$done_line" -gt 0 ] && echo yes || echo NO); teleports: $(grep -c 'teleported' "$OUT/server.txt") ($(grep -c 'AGREE' "$OUT/server.txt") agree, $(grep -c 'DIFFER' "$OUT/server.txt") differ)"
[ "$st" = 0 ] && [ "$fails" = 0 ] && [ "$done_line" -gt 0 ] && [ "$n" -gt 0 ]

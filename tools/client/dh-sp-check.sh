#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Distant Horizons far terrain in SINGLEPLAYER (Ben's case, 2026-10-21: his client flew in spectator,
# saw almost no far terrain, and DH's generator threads used under 6 s of CPU in 7 minutes): run on
# the test VM from the repo root:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start dh-sp --kind shell:ensemble env DHSP_SPEED=0 tools/client/dh-sp-check.sh [profile]
#
# One client (portablemc, NeoForge, the server set's own mod jars, software GL on a private Xvfb
# display) opens a copy of the set's world (smoke/<profile>/server/planet; the set must have been
# started once, so that it has a level.dat) as a singleplayer world, in spectator mode. The world holds
# a datapack that moves the player along x at DHSP_SPEED blocks per second (a tick function; 0 = still),
# so no RCON is needed. Distant Horizons' integrated server runs Deep Time's world-generator override
# in the client's own JVM, next to the camera that its pauseIfMovingQuickly rule watches
# (ThreadPoolUtil.worldGenThreadsCanRun, 3.3.3: paused while the average camera speed is above 20).
#
# Every DHSP_SAMPLE seconds (default 20) for DHSP_SECS seconds (default 300) after the world is up it
# prints: the CPU seconds of the JVM's Distant Horizons threads (tools/client/dh-thread-cpu.py), the
# rows in the world's Distant Horizons database (a copy, so the write-ahead log is read) and the last
# progress line of the client log. Knobs, all env:
#   DHSP_SPEED=<b/s>    the player's speed (default 0); above 20 the camera counts as moving quickly
#   DHSP_NO_PAUSE=1     pauseIfMovingQuickly and limitIfServerUnhealthy = false in Distant Horizons' config
#   DHSP_PLAN=<plan>    generatorPlan (default SURFACE_THEN_CHUNKS; Ben's); chunkGeneratorMode is FEATURES
#   DHSP_SECS, DHSP_SAMPLE, DHSP_VIEW (render distance in chunks, default 12)
#
# Output: smoke/<profile>/sp-check.txt. Nothing is rendered or judged by eye; exit 0 when the world came up.
set -uo pipefail
SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
cd "$(dirname "$0")/../.."
REPO="$PWD"
P="${1:-mic}"
ADMIT="${DEEPTIME_ADMIT:-/usr/local/lib/deeptime/admit.py}"
if [ "${DEEPTIME_ADMIT_PID:-}" != "$$" ] && [ -f "$ADMIT" ]; then
    exec python3 "$ADMIT" run --kind "client:$P" --server --label "$(basename "$REPO")" -- "$SELF" "$@"
fi
SET_SRV="$REPO/smoke/$P/server"
CL="$REPO/smoke/$P/client-sp"
OUT="$REPO/smoke/$P/sp-check.txt"
[ -f "$SET_SRV/planet/level.dat" ] || { echo "dh-sp-check: smoke/$P/server/planet has no level.dat (start the set once: tools/smoke/run.py --profile $P, or dh-client-check.sh $P)" >&2; exit 2; }
exec 8>"$REPO/smoke/$P/.world.lock"
flock -n 8 || { echo "dh-sp-check: waiting for smoke/$P's world"; flock 8; }
. "$REPO/smoke/tools.env"
NEO="$(cat "$REPO/smoke/$P/neoforge.version")"
: > "$OUT"
say() { echo "$*" | tee -a "$OUT"; }
SPEED="${DHSP_SPEED:-0}"; SECS="${DHSP_SECS:-300}"; STEP="${DHSP_SAMPLE:-20}"; VIEW="${DHSP_VIEW:-12}"
DISP="${DEEPTIME_DISPLAY:-:89}"
MARK="$(basename "$REPO")/smoke/$P/client-sp/pmc/libraries"

CPID=""; XPID=""; PAPID=""; PADIR=""
cleanup() {
    [ -n "$CPID" ] && kill "$CPID" 2>/dev/null
    pkill -f -- "$MARK" 2>/dev/null
    [ -n "$XPID" ] && kill "$XPID" 2>/dev/null
    [ -n "$PAPID" ] && kill "$PAPID" 2>/dev/null; [ -n "$PADIR" ] && rm -rf "$PADIR"
}
trap cleanup EXIT

# The world: a fresh copy, no Distant Horizons database, a datapack that moves the player.
rm -rf "$CL/game/saves" "$CL/game/Distant_Horizons_server_data" "$CL/game/mods"
mkdir -p "$CL/game/saves" "$CL/game/mods" "$CL/game/config"
cp -r "$SET_SRV/planet" "$CL/game/saves/dhsp"
rm -f "$CL/game/saves/dhsp/session.lock"
find "$CL/game/saves/dhsp" -name 'DistantHorizons.sqlite*' -delete 2>/dev/null
DP="$CL/game/saves/dhsp/datapacks/dhmove"
mkdir -p "$DP/data/dhmove/function" "$DP/data/minecraft/tags/function"
printf '{"pack":{"pack_format":48,"description":"dh-sp-check: moves the player"}}\n' > "$DP/pack.mcmeta"
PER_TICK=$(python3 -c "print(f'{$SPEED/20:.3f}')")
{ echo "gamemode spectator @a"; [ "$SPEED" != 0 ] && echo "execute as @a at @s run tp @s ~$PER_TICK ~ ~"; } > "$DP/data/dhmove/function/tick.mcfunction"
printf '{"values":["dhmove:tick"]}\n' > "$DP/data/minecraft/tags/function/tick.json"
cp "$SET_SRV/mods/"*.jar "$CL/game/mods/"
rm -rf "$CL/game/kubejs"
if [ -d "$SET_SRV/kubejs" ]; then   # the set's KubeJS startup scripts register its items and blocks
    mkdir -p "$CL/game/kubejs"
    for d in startup_scripts client_scripts assets config; do [ -d "$SET_SRV/kubejs/$d" ] && cp -r "$SET_SRV/kubejs/$d" "$CL/game/kubejs/$d"; done
fi

# Distant Horizons' config (afresh): Ben's plan, progress to the log, updater off; the pause rules on
# (their default) unless DHSP_NO_PAUSE.
DHC="$CL/game/config/DistantHorizons.toml"
{
    printf '[client]\n\t[client.advanced]\n\t\t[client.advanced.autoUpdater]\n\t\t\tenableAutoUpdater = false\n'
    printf '[common]\n\t[common.worldGenerator]\n\t\tgeneratorPlan = "%s"\n\t\tchunkGeneratorMode = "FEATURES"\n' "${DHSP_PLAN:-SURFACE_THEN_CHUNKS}"
    printf '\t\tshowGenerationProgress = "LOG"\n\t\tgenerationProgressDisplayIntervalInSeconds = 5\n'
    [ -n "${DHSP_NO_PAUSE:-}" ] && printf '\t\tpauseIfMovingQuickly = false\n\t\tlimitIfServerUnhealthy = false\n'
} > "$DHC"
printf 'onboardAccessibility:false\nskipMultiplayerWarning:true\njoinedFirstServer:true\nsoundCategory_master:0.0\nrenderDistance:%s\n' "$VIEW" > "$CL/game/options.txt"
say "== singleplayer smoke/$P: speed $SPEED b/s, ${SECS} s, plan ${DHSP_PLAN:-SURFACE_THEN_CHUNKS}, pause rules ${DHSP_NO_PAUSE:+OFF}${DHSP_NO_PAUSE:-on}, render distance $VIEW"
say "load at start: $(cut -d' ' -f1-3 /proc/loadavg)"

pinned() { nix build --no-link --print-out-paths --inputs-from "$REPO" "nixpkgs#$1" | head -1; }
CJAVA=$(pinned jdk21)/bin/java
MESA=$(pinned mesa)
GLVND=$(pinned libglvnd)
XVFB=$(pinned xorg-server)/bin/Xvfb
PMC=$(nix build --no-link --print-out-paths nixpkgs#portablemc | head -1)/bin/portablemc
rm -f "$CL/xvfb.log"
export DISPLAY=$DISP LD_LIBRARY_PATH="$MESA/lib:$GLVND/lib" __GLX_VENDOR_LIBRARY_NAME=mesa \
    __EGL_VENDOR_LIBRARY_DIRS="$MESA/share/glvnd/egl_vendor.d" LIBGL_DRIVERS_PATH="$MESA/lib/dri" \
    LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe LP_NUM_THREADS=4
PA=$(pinned pulseaudio)/bin/pulseaudio
PADIR=$(mktemp -d /tmp/dhsp-pulse.XXXXXX)
PULSE_RUNTIME_PATH="$PADIR" HOME="$PADIR" setsid "$PA" -n --daemonize=no --exit-idle-time=-1 --use-pid-file=no \
    --disable-shm=yes --load="module-native-protocol-unix socket=$PADIR/native auth-anonymous=1" \
    --load="module-null-sink" > "$CL/pulse.log" 2>&1 < /dev/null &
PAPID=$!; sleep 2
export PULSE_SERVER="unix:$PADIR/native" ALSOFT_DRIVERS=pulse
pgrep -f "Xvfb $DISP " > /dev/null || {
    rm -f "/tmp/.X${DISP#:}-lock" "/tmp/.X11-unix/X${DISP#:}"
    setsid "$XVFB" $DISP -screen 0 1280x720x24 -nolisten tcp +extension GLX >> "$CL/xvfb.log" 2>&1 < /dev/null &
    XPID=$!; sleep 3
}
setsid "$PMC" --main-dir "$CL/pmc" start "neoforge::$NEO" --mc-dir "$CL/game" --jvm "$CJAVA" \
    --jvm-arg=-Xmx6G --jvm-arg=-XX:ActiveProcessorCount=6 \
    -u DeepTimeProbe --resolution 1280x720 --join-world dhsp > "$CL/client.log" 2>&1 < /dev/null &
CPID=$!
CLOG="$CL/game/logs/latest.log"
up=0
for i in $(seq 240); do
    grep -a -q 'DeepTimeProbe joined the game' "$CLOG" 2>/dev/null && { up=1; break; }
    kill -0 $CPID 2>/dev/null || { say "client exited before the world was up (smoke/$P/client-sp/client.log)"; break; }
    sleep 5
done
[ $up = 1 ] || { say "the world did not come up"; tail -5 "$CLOG" 2>/dev/null | tee -a "$OUT"; exit 1; }
T0=$SECONDS
say "world up; sampling for $SECS s"
cpu() { python3 "$REPO/tools/client/dh-thread-cpu.py" "$MARK" 'dh' 2>/dev/null | tr '\n' ';'; }
rows() {
    local db; db=$(find "$CL/game" -name 'DistantHorizons.sqlite' -printf '%s %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2-)
    [ -n "$db" ] || { echo "no database"; return; }
    rm -rf "$CL/dbcopy"; mkdir -p "$CL/dbcopy"; cp "$db"* "$CL/dbcopy/" 2>/dev/null
    python3 -c '
import sqlite3, sys
c = sqlite3.connect(sys.argv[1])
try:
    n = c.execute("select count(*) from FullData").fetchone()[0]
    by = c.execute("select DetailLevel, count(*) from FullData group by DetailLevel order by DetailLevel").fetchall()
    print(f"FullData rows {n} by detail {by}")
except Exception as e:
    print("db:", e)
' "$CL/dbcopy/$(basename "$db")"
}
t=0
while [ $t -lt "$SECS" ]; do
    sleep "$STEP"; t=$(( t + STEP ))
    say "t=${t}s load $(cut -d' ' -f1 /proc/loadavg): $(rows)"
    say "   threads: $(cpu)"
    say "   log: $(grep -a -i -E 'world ?gen|generation (progress|task)|chunks per second|import task' "$CLOG" 2>/dev/null | tail -1 | cut -c1-300)"
done
say "client log: 'Set world gen queue' $(grep -a -c 'Set world gen queue for level' "$CLOG" 2>/dev/null) time(s); Chunky rule line(s) $(grep -a -c 'Partially Incompatible Distant Horizons mod detected: \[Chunky\]' "$CLOG" 2>/dev/null)"
say "client log: DH progress lines (last 10): $(grep -a -i -E 'world ?gen|generation (progress|task)|chunks per second|import task' "$CLOG" 2>/dev/null | tail -10 | tr '\n' '|')"
exit 0

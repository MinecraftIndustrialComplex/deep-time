#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Distant Horizons far terrain to a joined client (P3-04; Ben 2026-10-02, "Mixin around TW's
# guard"): run on the test VM from the repo root:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh run dh-client tools/client/dh-client-check.sh [profile]
#
# It runs through the admission tool (tools/remote/vm/admit.py, kind client:<profile>; Ben 2026-10-03:
# load-based admission, no server lock): the server runs in a private copy of the set on the job's
# own ports, the client on the job's own X display; the set's world is linked in and held with the
# set's world lock, so another check of the same set waits while other servers run beside it.
#
# profile: an installed smoke set with Distant Horizons (default dh: standalone + DH at the MIC
# pack's pin; install it first with tools/smoke/install.sh dh --preset earthlike_quick_16k ...).
# It starts that server, then a real client (portablemc, NeoForge, the server's own mod jars,
# software GL on a private Xvfb display) that joins it, and records from the server's
# `/deeptime probe dh` how many sections Distant Horizons asked Deep Time's generator for, and over
# what x extent: with no /dh pregen, every section asked is a client's request. Then the player is
# moved beside the seam (x = C/2 - 100) and the extent must reach both ends of the lap (the
# client's requests past the seam reach the generator folded into the planet's lap). The
# client's own Distant Horizons database is then read: it must hold sections further from the
# client than any chunk it loaded, on both sides of the seam (they can only have been sent by the
# server). Nothing is rendered or judged by eye.
#
# Output: smoke/<profile>/client-check.txt (the probe lines and the verdicts); exit 0 when the
# client's requests reached the generator on both sides of the seam and the sections arrived.
#
# DHCC_CLIENT_JVM_ARGS: extra client JVM arguments, space-separated (a control run, e.g.
# -Ddeeptime.dhGenerationGate=false). DHCC_CLIENT_DROP: a jar glob left out of the client's mods
# (e.g. 'Chunky-*.jar'), the server keeps it. The client log's Distant Horizons facts are recorded:
# its Chunky rule, Deep Time's generation gate, and whether its request module set a world gen queue.
#
# Measurement knobs (all off by default; 2026-10-22, DH 3.3.3 fill-rate task):
#   DHCC_SERIES=<s>     probe every <s> seconds during the waits and print "series ..." lines (sections the
#                       generator made, real chunks the server built, the x extents), then the CPU seconds of
#                       the client's and the server's Distant Horizons threads (tools/client/dh-thread-cpu.py)
#   DHCC_PROGRESS=1     both sides' Distant Horizons config gets showGenerationProgress = "LOG" (its task
#                       counts reach the logs) and generatorPlan SURFACE_THEN_CHUNKS with chunkGeneratorMode
#                       FEATURES written explicitly (Ben's settings); the config files are rewritten every run
#   DHCC_NO_PAUSE=1     as DHCC_PROGRESS, plus pauseIfMovingQuickly and limitIfServerUnhealthy = false (the A/B)
#   DHCC_MOVE=<b/s>     after the two waits, a third phase of DHCC_MOVE_SECS (default 120) seconds in which the
#                       script teleports the spectating player along x at that many blocks per second (once a
#                       second): the "camera moving quickly" case of pauseIfMovingQuickly (it fires above 20)
#   DHCC_WAIT1, DHCC_WAIT2  the two waits in seconds (default 180 and 300)
#   DHCC_JOIN_SECS=<s>  how long one attempt waits for the client to join (default 900; the full MIC client
#                       on a loaded VM needs more)
set -uo pipefail
SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
cd "$(dirname "$0")/../.."
REPO="$PWD"
P="${1:-dh}"
ADMIT="${DEEPTIME_ADMIT:-/usr/local/lib/deeptime/admit.py}"
if [ "${DEEPTIME_ADMIT_PID:-}" != "$$" ] && [ -f "$ADMIT" ]; then
    exec python3 "$ADMIT" run --kind "client:$P" --server --label "$(basename "$REPO")" -- "$SELF" "$@"
fi
SET_SRV="$REPO/smoke/$P/server"
SRV="$SET_SRV"
CL="$REPO/smoke/$P/client"
OUT="$REPO/smoke/$P/client-check.txt"
[ -d "$SET_SRV/mods" ] || { echo "dh-client-check: smoke/$P is not installed (tools/smoke/install.sh $P ...)" >&2; exit 2; }
# The set's world (and this check's client folder) are this run's until it ends.
exec 8>"$REPO/smoke/$P/.world.lock"
flock -n 8 || { echo "dh-client-check: waiting for smoke/$P's world (another run of this set uses it)"; flock 8; }
# A fresh server-side Distant Horizons database: the world is linked, not copied, so sections an
# earlier check had generated are served from it without Deep Time's generator, and step 4 counts
# nothing asked (2026-10-20: 5 generator calls for 24,126 sections the client received).
find "$SET_SRV/planet" -path '*/data/DistantHorizons.sqlite*' -delete 2>/dev/null
if [ -n "${DEEPTIME_SERVER_DIR:-}" ]; then
    SRV="$DEEPTIME_SERVER_DIR/server"
    python3 "$REPO/tools/smoke/instance.py" make --set "$SET_SRV" --dest "$SRV" --world-link planet >/dev/null \
        || { echo "dh-client-check: could not copy smoke/$P/server" >&2; exit 1; }
fi
. "$REPO/smoke/tools.env"
NEO="$(cat "$REPO/smoke/$P/neoforge.version")"
GAME=$(sed -n 's/^server-port=//p' "$SRV/server.properties")
RCONP=$(sed -n 's/^rcon.port=//p' "$SRV/server.properties")
PW=$(sed -n 's/^rcon.password=//p' "$SRV/server.properties")
DISP="${DEEPTIME_DISPLAY:-:89}"
: > "$OUT"
say() { echo "$*" | tee -a "$OUT"; }
rcon() { "$MCRCON" -H 127.0.0.1 -P "$RCONP" -p "$PW" -c "$1" 2>&1; }
probe() { rcon "deeptime probe dh" >/dev/null; sleep 1; grep -a '\[deeptime-probe\] {"probe":"dh",' "$SRV/console.log" | tail -1 | sed 's/.*\[deeptime-probe\] //'; }
field() { python3 -c 'import json,sys; d=json.loads(sys.argv[1]); l=[l for l in d.get("levels",[]) if l["level"]=="minecraft:overworld"]; print((l[0] if l else {}).get(sys.argv[2],""))' "$1" "$2"; }

SPID=""; XPID=""; CPID=""
T0=$SECONDS
top() { python3 -c 'import json,sys; print(json.loads(sys.argv[1]).get(sys.argv[2],""))' "$1" "$2"; }
cpu_lines() {   # the DH threads' CPU seconds so far, client and server
    say "   client JVM, DH threads: $(python3 "$REPO/tools/client/dh-thread-cpu.py" "$(basename "$REPO")/smoke/$P/client/pmc/libraries" 'dh' 2>/dev/null | tr '\n' ';')"
    say "   server JVM, DH threads: $(python3 "$REPO/tools/client/dh-thread-cpu.py" "$SRV" 'dh' 2>/dev/null | tr '\n' ';')"
}
series() {   # $1: seconds to wait, $2: a label; with DHCC_SERIES a probe every that many seconds
    local secs=$1 step=${DHCC_SERIES:-0} t=0 q
    if [ "$step" -le 0 ]; then sleep "$secs"; return; fi
    while [ $t -lt "$secs" ]; do
        sleep "$step"; t=$(( t + step )); q="$(probe)"
        say "series $2 t=${t}s (run $(( SECONDS - T0 )) s): sections=$(field "$q" sections) chunks_surfaced=$(top "$q" chunks_surfaced) generator_x=$(field "$q" x_extent) chunk_x=$(top "$q" chunk_x_extent) by_detail=$(field "$q" by_detail)"
    done
}
cleanup() {
    [ -n "$CPID" ] && kill "$CPID" 2>/dev/null
    pkill -f -- "$(basename "$REPO")/smoke/$P/client/pmc/libraries" 2>/dev/null   # the client JVM (its classpath names this folder)
    [ -n "$SPID" ] && { rcon stop >/dev/null 2>&1; for i in $(seq 60); do kill -0 "$SPID" 2>/dev/null || break; sleep 1; done; kill "$SPID" 2>/dev/null; }
    [ -n "$XPID" ] && kill "$XPID" 2>/dev/null
    [ -n "${PAPID:-}" ] && kill "$PAPID" 2>/dev/null; [ -n "${PADIR:-}" ] && rm -rf "$PADIR"
    [ "$SRV" != "$SET_SRV" ] && [ -f "$SRV/console.log" ] && cp "$SRV/console.log" "$REPO/smoke/$P/console.log"
}
trap cleanup EXIT

# 1. The server, on the job's ports (instance.py wrote them into its server.properties). Off the test
# VM the set's fixed ones, which other sets share: wait while they are taken.
for i in $(seq 120); do
    ss -ltn 2>/dev/null | grep -q -e ":$GAME " -e ":$RCONP " || break
    [ $i = 1 ] && say "== port $GAME or $RCONP is taken; waiting (at most 20 min)"
    sleep 10
done
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
# An empty client-side Distant Horizons database every time: one kept from an earlier check is
# counted in step 5 as sent in this one (2026-10-20: a control whose client asked for nothing
# showed 43,175 far sections, all rebuilt on the client from the previous check's data).
rm -rf "$CL/game/Distant_Horizons_server_data" "$CL/dbcopy"
cp "$SRV/mods/"*.jar "$CL/game/mods/"
# KubeJS registers items, blocks and fluids in startup scripts: a client without the set's scripts is refused
# at login ("The server sent registries with unknown keys", 2026-10-22, the full MIC set).
if [ -d "$SRV/kubejs" ]; then
    rm -rf "$CL/game/kubejs"; mkdir -p "$CL/game/kubejs"
    for d in startup_scripts client_scripts assets config; do [ -d "$SRV/kubejs/$d" ] && cp -r "$SRV/kubejs/$d" "$CL/game/kubejs/$d"; done
fi
if [ -n "${DHCC_CLIENT_DROP:-}" ]; then
    for j in "$CL/game/mods/"$DHCC_CLIENT_DROP; do [ -f "$j" ] && { rm -f "$j"; say "client leaves out $(basename "$j")"; }; done
fi
JVMX=(); for a in ${DHCC_CLIENT_JVM_ARGS:-}; do JVMX+=("--jvm-arg=$a"); done
[ ${#JVMX[@]} -gt 0 ] && say "client JVM arguments: ${DHCC_CLIENT_JVM_ARGS}"
# Distant Horizons' update check opens an "update available" screen at the title screen, which
# holds the client there instead of joining (seen 2026-10-03 with 3.3.1; the pack pins 3.3.3 since pack 62dd14e).
# Minecraft's first-launch accessibility screen holds it the same way; the options file skips it.
[ -f "$CL/game/options.txt" ] || printf 'onboardAccessibility:false\nskipMultiplayerWarning:true\njoinedFirstServer:true\nsoundCategory_master:0.0\n' > "$CL/game/options.txt"
DHC="$CL/game/config/DistantHorizons.toml"
mkdir -p "$CL/game/config"
# The measurement knobs write both sides' config afresh (defaults but for what is named here).
dh_config() {   # $1: the file; the updater is off on the client only (a harmless key on the server)
    {
        printf '[client]\n\t[client.advanced]\n\t\t[client.advanced.autoUpdater]\n\t\t\tenableAutoUpdater = false\n'
        printf '[common]\n\t[common.worldGenerator]\n\t\tgeneratorPlan = "SURFACE_THEN_CHUNKS"\n\t\tchunkGeneratorMode = "FEATURES"\n'
        printf '\t\tshowGenerationProgress = "LOG"\n\t\tgenerationProgressDisplayIntervalInSeconds = 5\n'
        [ -n "${DHCC_NO_PAUSE:-}" ] && printf '\t\tpauseIfMovingQuickly = false\n\t\tlimitIfServerUnhealthy = false\n'
    } > "$1"
}
KNOBS=""
if [ -n "${DHCC_PROGRESS:-}${DHCC_NO_PAUSE:-}" ]; then
    KNOBS=1
    mkdir -p "$SRV/config"; dh_config "$SRV/config/DistantHorizons.toml"; dh_config "$DHC"
    say "DH config written on both sides: SURFACE_THEN_CHUNKS / FEATURES, progress LOG${DHCC_NO_PAUSE:+, pauseIfMovingQuickly and limitIfServerUnhealthy off}"
fi
if [ -n "$KNOBS" ]; then :
elif [ -f "$DHC" ]; then
    sed -i 's/^\(\s*enableAutoUpdater\s*=\s*\).*/\1false/' "$DHC"
else
    printf '[client]\n\t[client.advanced]\n\t\t[client.advanced.autoUpdater]\n\t\t\tenableAutoUpdater = false\n' > "$DHC"
fi
# The client's JVM, Mesa, libglvnd, Xvfb and PulseAudio come from the flake's own nixpkgs
# (flake.lock), as in tools/client/vm-client.sh. Mesa's driver loads inside the JVM, which has
# already loaded its own revision's libraries (libstdc++, zlib, ...) by soname; a Mesa from another
# revision (the registry's, or smoke/tools.env's JDK against the pinned Mesa) then fails to load
# there and GLFW sees "GLX: No GLXFBConfigs returned" while glxinfo on the same display works
# (2026-10-19). The server keeps smoke/tools.env's JDK: it loads no GL. portablemc only launches
# the JVM, so the registry's serves.
pinned() { nix build --no-link --print-out-paths --inputs-from "$REPO" "nixpkgs#$1" | head -1; }
CJAVA=$(pinned jdk21)/bin/java
MESA=$(pinned mesa)
GLVND=$(pinned libglvnd)
XVFB=$(pinned xorg-server)/bin/Xvfb
rm -f "$CL/xvfb.log"
PMC=$(nix build --no-link --print-out-paths nixpkgs#portablemc | head -1)/bin/portablemc
# The GL environment is exported before Xvfb starts: without it the X server has no GLX
# (the client then dies with "GLX extension not found").
export DISPLAY=$DISP LD_LIBRARY_PATH="$MESA/lib:$GLVND/lib" __GLX_VENDOR_LIBRARY_NAME=mesa \
    __EGL_VENDOR_LIBRARY_DIRS="$MESA/share/glvnd/egl_vendor.d" LIBGL_DRIVERS_PATH="$MESA/lib/dri" \
    LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe LP_NUM_THREADS=4
# Sound goes to a PulseAudio server of the check's own with a null sink: with the session's
# half-working PulseAudio the client JVM aborted while starting its sound engine (2026-10-03:
# "Assertion 'p' failed at ../src/pulse/simple.c:273"), and without OpenAL Minecraft does not start.
PA=$(pinned pulseaudio)/bin/pulseaudio
PADIR=$(mktemp -d /tmp/dhcc-pulse.XXXXXX)
PULSE_RUNTIME_PATH="$PADIR" HOME="$PADIR" setsid "$PA" -n --daemonize=no --exit-idle-time=-1 --use-pid-file=no \
    --disable-shm=yes --load="module-native-protocol-unix socket=$PADIR/native auth-anonymous=1" \
    --load="module-null-sink" > "$CL/pulse.log" 2>&1 < /dev/null &
PAPID=$!; sleep 2
export PULSE_SERVER="unix:$PADIR/native" ALSOFT_DRIVERS=pulse
start_xvfb() {
    if [ -n "$XPID" ] && kill -0 "$XPID" 2>/dev/null; then return; fi
    pgrep -f "Xvfb $DISP " > /dev/null && return
    rm -f "/tmp/.X${DISP#:}-lock" "/tmp/.X11-unix/X${DISP#:}"
    setsid "$XVFB" $DISP -screen 0 1280x720x24 -nolisten tcp +extension GLX >> "$CL/xvfb.log" 2>&1 < /dev/null &
    XPID=$!; sleep 3
}
# Up to three attempts: on 2026-10-03 the display went away 20 s after the client reached its
# title screen (Xlib: "X connection to :89 broken"), with nothing in Xvfb's log.
joined=0
for attempt in 1 2 3; do
    start_xvfb
    say "== client joins 127.0.0.1:$GAME (attempt $attempt)"
    setsid "$PMC" --main-dir "$CL/pmc" start "neoforge::$NEO" --mc-dir "$CL/game" --jvm "$CJAVA" \
        --jvm-arg=-Xmx6G --jvm-arg=-XX:ActiveProcessorCount=6 "${JVMX[@]}" \
        -u DeepTimeProbe --resolution 1280x720 \
        --join-server 127.0.0.1 --join-server-port "$GAME" > "$CL/client-$attempt.log" 2>&1 < /dev/null &
    CPID=$!
    why=""
    for i in $(seq $(( ${DHCC_JOIN_SECS:-900} / 5 ))); do
        grep -a -q 'DeepTimeProbe joined the game' "$SRV/console.log" && { joined=1; break; }
        kill -0 $CPID 2>/dev/null || { why="client exited"; break; }
        [ -n "$XPID" ] && ! kill -0 "$XPID" 2>/dev/null && { why="Xvfb $DISP went away"; break; }
        grep -a -q -e 'registries with unknown keys' -e 'Failed to connect to the server' "$CL/game/logs/latest.log" 2>/dev/null && {
            why="the server refused the client: $(grep -a -m1 -e 'registries with unknown keys' -e 'Failed to connect' "$CL/game/logs/latest.log" | cut -c1-200)"; break; }
        grep -a -q -e 'X connection to .* broken' "$CL/client-$attempt.log" && { why="the client lost the X display"; break; }
        grep -a -q -e 'Failed to find a valid GLFW profile' -e 'Preparing crash report' "$CL/client-$attempt.log" && {
            why="client failed to start: $(grep -a -m1 -e 'GLFW error' -e 'crash report' "$CL/client-$attempt.log")"; break; }
        sleep 5
    done
    [ $joined = 1 ] && break
    say "   attempt $attempt: ${why:-did not join in $(( ${DHCC_JOIN_SECS:-900} / 60 )) min} (smoke/$P/client/client-$attempt.log)"
    kill "$CPID" 2>/dev/null; pkill -f -- "$(basename "$REPO")/smoke/$P/client/pmc/libraries" 2>/dev/null; sleep 5
done
[ $joined = 1 ] || { say "the client did not join"; exit 1; }
rcon "gamemode spectator DeepTimeProbe" >/dev/null
J="$(probe)"; say "probe at join: $J"

# 3. Beside the spawn, then beside the seam.
series "${DHCC_WAIT1:-180}" spawn
A="$(probe)"; say "probe after ${DHCC_WAIT1:-180} s at the spawn: $A"
[ -n "${DHCC_SERIES:-}" ] && cpu_lines
X=$(( C / 2 - 100 ))
say "== tp DeepTimeProbe $X 300 0 (seam at x = $(( C / 2 )))"
rcon "tp DeepTimeProbe $X 300 0" >/dev/null
series "${DHCC_WAIT2:-300}" seam
B="$(probe)"; say "probe after ${DHCC_WAIT2:-300} s beside the seam: $B"
[ -n "${DHCC_SERIES:-}" ] && cpu_lines
if [ -n "${DHCC_MOVE:-}" ]; then
    say "== moving at ${DHCC_MOVE} blocks/s for ${DHCC_MOVE_SECS:-120} s (teleports once a second)"
    mx=$X
    for i in $(seq "${DHCC_MOVE_SECS:-120}"); do
        mx=$(( mx - DHCC_MOVE )); rcon "tp DeepTimeProbe $mx 300 0" >/dev/null; sleep 1
        [ -n "${DHCC_SERIES:-}" ] && [ $(( i % ${DHCC_SERIES} )) = 0 ] && { q="$(probe)"; say "series move t=${i}s (run $(( SECONDS - T0 )) s): sections=$(field "$q" sections) chunks_surfaced=$(top "$q" chunks_surfaced) generator_x=$(field "$q" x_extent)"; }
    done
    M="$(probe)"; say "probe after the move: $M"
    [ -n "${DHCC_SERIES:-}" ] && cpu_lines
fi

# 4. Verdicts. First what the client's own log says about Distant Horizons' generator.
CLOG="$CL/game/logs/latest.log"
say "client log: DH's Chunky rule $(grep -a -c 'Partially Incompatible Distant Horizons mod detected: \[Chunky\]' "$CLOG" 2>/dev/null) line(s);" \
    "Deep Time's gate: $(grep -a -o "\[deeptime\] Distant Horizons' world generator [a-z ]*" "$CLOG" 2>/dev/null | sort | uniq -c | tr -s ' ' | paste -sd ';' -)"
say "client log: 'Set world gen queue' $(grep -a -c 'Set world gen queue for level' "$CLOG" 2>/dev/null) time(s) (DH's request module started)"
say "client log: DH's progress lines (last 12): $(grep -a -i -E 'world ?gen|generation (progress|task)|chunks per second|import task' "$CLOG" 2>/dev/null | tail -12 | tr '\n' '|')"
say "server log: DH's progress lines (last 12): $(grep -a -i -E 'world ?gen|generation (progress|task)|chunks per second|import task' "$SRV/console.log" 2>/dev/null | tail -12 | tr '\n' '|')"
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

# 5. What arrived: the sections in the client's own database (a copy, so the running client's
# write-ahead log is read too). The client has loaded chunks only within the server's view distance
# of its two positions; a section further than 512 blocks from both can only have been sent by
# the server, and the server's only source is Deep Time's generator (no chunk exists out there).
MAIN=$(find "$CL/game" -path '*Distant_Horizons*' -name '*.sqlite' -printf '%s %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2-)
if [ -n "$MAIN" ]; then
    rm -rf "$CL/dbcopy"; mkdir -p "$CL/dbcopy"; cp "$MAIN"* "$CL/dbcopy/" 2>/dev/null
    SPAWN=$(python3 -c 'import json,sys; print(*json.loads(sys.argv[1]).get("spawn",[0,0]))' "$S0")
    python3 - "$CL/dbcopy/$(basename "$MAIN")" "$C" "$X" $SPAWN <<'PY' | tee -a "$OUT"
import sqlite3, sys
db, c, seam_post_x, spawn_x, spawn_z = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
con = sqlite3.connect(db)
cols = [r[1] for r in con.execute("pragma table_info(FullData)")]
if not {"DetailLevel", "PosX", "PosZ"} <= set(cols):
    print("client database: no FullData(DetailLevel, PosX, PosZ) table; tables:",
          [r[0] for r in con.execute("select name from sqlite_master where type='table'")])
    print("RECEIVED far=0 west=0 east=0")
    sys.exit(0)
rows = con.execute("select DetailLevel, PosX, PosZ from FullData").fetchall()
half = c // 2
base = 6 if rows and min(r[0] for r in rows) >= 6 else 0   # stored from 0 (a 64-block section) or as the section's own level
far = west = east = 0
by = {}
for d, px, pz in rows:
    d -= base
    w = 64 << d
    x0, z0 = px * w, pz * w
    xf = (x0 + half) % c - half   # folded into the lap, whichever way the client keys it
    lo, hi = by.get(d, (x0, x0 + w)); by[d] = (min(lo, x0), max(hi, x0 + w))
    def clear(cx, cz):   # further than 512 blocks from a position, measured through the seam
        dx = min(abs((x0 + w // 2 - cx) % c), c - abs((x0 + w // 2 - cx) % c))
        return max(dx, abs(z0 + w // 2 - cz)) - w // 2 > 512
    if clear(spawn_x, spawn_z) and clear(seam_post_x, 0):
        far += 1
        if d == 0 and half - 2048 <= xf < half: west += 1
        if d == 0 and -half <= xf < -half + 2048: east += 1
print(f"client database: {len(rows)} sections; x extent by detail level (blocks): "
      + "; ".join(f"{d}: {lo}..{hi}" for d, (lo, hi) in sorted(by.items())))
print(f"client database: {far} sections further than 512 blocks from both of the client's positions "
      f"(sent by the server); of the 64-block ones, {west} in the 2048 blocks west of the seam and {east} in the 2048 blocks east of it")
print(f"RECEIVED far={far} west={west} east={east}")
PY
    R=$(grep -a '^RECEIVED ' "$OUT" | tail -1)
    far=$(echo "$R" | sed -n 's/.*far=\([0-9]*\).*/\1/p'); west=$(echo "$R" | sed -n 's/.*west=\([0-9]*\).*/\1/p'); east=$(echo "$R" | sed -n 's/.*east=\([0-9]*\).*/\1/p')
    [ "${far:-0}" -gt 0 ] || { say "FAIL: the client's database holds no section beyond its own loaded chunks (nothing was sent)"; ok=1; }
    { [ "${west:-0}" -gt 0 ] && [ "${east:-0}" -gt 0 ]; } || { say "FAIL: the client's database lacks sent sections on one side of the seam (west $west, east $east)"; ok=1; }
else
    say "FAIL: the client has no Distant Horizons database"; ok=1
fi
[ $ok = 0 ] && say "PASS: the client's requests reached Deep Time's generator on both sides of the seam, and the sections arrived in the client's database"
exit $ok

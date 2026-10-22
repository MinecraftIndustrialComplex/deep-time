#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Real-client screenshots of the Create World planet editor (P1-11's client check, plan §9.8) on the
# test VM: the real NeoForge 1.21.1 client -- ModDevGradle's own dev launch (build/moddev/runClient.sh,
# Deep Time from this checkout's build output and host natives, Toroidal World's bundle from run/mods)
# -- on a private Xvfb display with Mesa's software GL (llvmpipe), as tools/client/dh-client-check.sh
# runs its client. With -Ddeeptime.dev.screenshots=true, ScreenshotDriver walks Create World ->
# editor -> run/timelapse -> layers -> review -> Accept -> world creation and writes the PNGs from the
# GL framebuffer. Nothing is judged by eye here: the PNGs, the driver's [deeptime-shot] lines and an
# HTML contact sheet land in review/out/<name>/ for a browser.
#
# Run it from the laptop, detached (a client walk on a real preset takes 10 to 30 minutes):
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start p1-11-client --kind client:editor tools/client/vm-client.sh --name p1-11 --prop length=250
#   tools/remote/mac.sh --no-sync -- tools/remote/detach.sh wait p1-11-client
#   tools/remote/mac.sh --no-sync --pull review/out/p1-11 -- true
#
#   --name N       output folder review/out/<N>/ (default vm-client); replaced on every run
#   --no-build     skip the Gradle step (classes, host natives, launch script, run/mods links)
#   --no-guidance  skip the second pass (natives forced off: the guidance panel, Create blocked)
#   --no-create    stop after Accept (no world creation, no in-world shot)
#   --size WxH     window size (default 1280x720)
#   --gui-scale N  Minecraft's GUI scale (default 2; 0 is the game's "Auto", the largest that fits)
#   --mods SET     link smoke/<SET>/server/mods into the client too (Gradle -PrunSet=SET; e.g. mic for
#                  the MIC pack's worldgen mods); DEEPTIME_RUN_MODS=<dir> points at another copy's set
#   --stage DIR    link the mods of a play-client-stage.sh stage (DIR is its minecraft/ folder, e.g.
#                  /data/ban/play/<hash>-client-lite/minecraft) and copy its kubejs/, config/ and
#                  defaultconfigs/ into the game dir (replacing the game dir's own), so the walk runs the
#                  player's set with the files the pack gives a new world (the lite set's rock tags come as
#                  kubejs/data/deeptime) and the pack's mod settings; implies --mods (the Gradle link step runs)
#   --rock-tags DIR  the MIC rock-tag datapack (e.g. /data/ban/mic/Docs/datapacks/deeptime-mic-rock-tags): its
#                  data/deeptime/ is copied into the game dir's kubejs/data/deeptime unless that already holds
#                  files (a stage's own), as tools/remote/play-client-apply.py does for Ben's full instance
#   --timeout S    per client pass (default 7200: a 1,000-Myr quick planet's ETA was ~2 h on the
#                  loaded VM; --prop length=300 takes a fraction of that)
#   --threads N    processors the client JVM sees (-XX:ActiveProcessorCount; default 8): the editor
#                  runs the simulation on N - 1 of them, as on a player's 8-core machine
#   --prop K=V     a ScreenshotDriver property for the main pass (repeatable), as
#                  -Ddeeptime.dev.screenshots.K=V: preset=<built-in family>, start=<start>,
#                  length=<Myr>, layers=<a,b,...>, vista=x,y,z,yaw,pitch[;...]
#
# The whole client run is one job of the VM's admission tool (kind client:editor, a server job: it
# gets its own X display; the integrated server binds no port). The Gradle step before it is its own
# job (gradle:build). Only this run's JVM, Xvfb and PulseAudio are ever stopped (their pids).
set -uo pipefail
SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
cd "$(dirname "$0")/../.."
REPO="$PWD"
ADMIT="${DEEPTIME_ADMIT:-/usr/local/lib/deeptime/admit.py}"

NAME=vm-client; BUILD=1; GUIDANCE=1; CREATE=true; WSIZE=1280x720; TIMEOUT=7200; THREADS=8; PROPS=()
GUISCALE=2; RUNSET=""; GRADLE_SET=(); STAGE_DIR=""; ROCK_TAGS=""
ARGS=("$@")
while [ $# -gt 0 ]; do
    case "$1" in
        --name) NAME="$2"; shift 2 ;;
        --no-build) BUILD=0; shift ;;
        --no-guidance) GUIDANCE=0; shift ;;
        --no-create) CREATE=false; shift ;;
        --size) WSIZE="$2"; shift 2 ;;
        --gui-scale) GUISCALE="$2"; shift 2 ;;
        --mods) RUNSET="$2"; shift 2 ;;
        --stage) STAGE_DIR="$2"; shift 2 ;;
        --rock-tags) ROCK_TAGS="$2"; shift 2 ;;
        --timeout) TIMEOUT="$2"; shift 2 ;;
        --threads) THREADS="$2"; shift 2 ;;
        --prop) [[ "$2" =~ ^[a-z]+=[-0-9.,\;a-z_/]+$ ]] || { echo "vm-client: bad --prop $2" >&2; exit 2; }
                PROPS+=("-Ddeeptime.dev.screenshots.$2"); shift 2 ;;
        -h|--help) sed -n '2,/^set -uo/p' "$0" | sed 's/^# \{0,1\}//' | sed '$d'; exit 0 ;;
        *) echo "vm-client: unknown argument $1 (see --help)" >&2; exit 2 ;;
    esac
done
[[ "$NAME" =~ ^[A-Za-z0-9._-]+$ ]] || { echo "vm-client: --name must be one plain word" >&2; exit 2; }
[[ "$WSIZE" =~ ^[0-9]+x[0-9]+$ ]] || { echo "vm-client: --size is WxH" >&2; exit 2; }
[[ "$GUISCALE" =~ ^[0-9]$ ]] || { echo "vm-client: --gui-scale is 0 (auto) to 9" >&2; exit 2; }
[ -z "$ROCK_TAGS" ] || [ -d "$ROCK_TAGS/data/deeptime" ] || { echo "vm-client: --rock-tags $ROCK_TAGS has no data/deeptime/" >&2; exit 2; }
if [ -n "$STAGE_DIR" ]; then
    [ -d "$STAGE_DIR/mods" ] || { echo "vm-client: --stage $STAGE_DIR has no mods/" >&2; exit 2; }
    # DEEPTIME_RUN_MODS replaces smoke/<set>/server/mods in the Gradle link step (neoforge/build.gradle);
    # the set's name only has to exist. Exported before the admitted re-run below, which inherits it.
    export DEEPTIME_RUN_MODS="$STAGE_DIR/mods"
    [ -n "$RUNSET" ] || RUNSET=stage
fi
if [ -n "$RUNSET" ]; then
    [[ "$RUNSET" =~ ^[a-z0-9-]+$ ]] || { echo "vm-client: --mods is a server set name" >&2; exit 2; }
    GRADLE_SET=("-PrunSet=$RUNSET")
fi
# A pack's mods need more heap (dh-client-check.sh gives MIC 6G).
XMX=4G; [ -n "$RUNSET" ] && XMX=6G
[ "$(uname -s)" = Linux ] || { echo "vm-client: runs on the test VM (Linux)" >&2; exit 2; }

# --- phase 1: the build (its own admitted job), then the client run as a server job ---------------
if [ "${DEEPTIME_ADMIT_PID:-}" != "$$" ] && [ -f "$ADMIT" ]; then
    if [ "$BUILD" = 1 ]; then
        python3 "$ADMIT" run --kind gradle:build --label "$(basename "$REPO")" -- \
            bash -c 'cd neoforge && ./gradlew --console=plain -q "$@" classes gametestClasses createClientLaunchScript linkRunModsClient' \
            vm-client-gradle ${GRADLE_SET[@]+"${GRADLE_SET[@]}"} \
            || { echo "vm-client: the Gradle step failed" >&2; exit 1; }
    fi
    exec python3 "$ADMIT" run --kind client:editor --server --label "$(basename "$REPO")" -- "$SELF" "${ARGS[@]}" --no-build
fi
if [ "$BUILD" = 1 ]; then
    (cd neoforge && ./gradlew --console=plain -q ${GRADLE_SET[@]+"${GRADLE_SET[@]}"} classes gametestClasses createClientLaunchScript linkRunModsClient) \
        || { echo "vm-client: the Gradle step failed" >&2; exit 1; }
fi

OUT="$REPO/review/out/$NAME"
WORK="$REPO/neoforge/build/vm-client"
LAUNCH_SCRIPT="$REPO/neoforge/build/moddev/runClient.sh"
rm -rf "$OUT" "$WORK"; mkdir -p "$OUT" "$WORK/tmp"
LOG="$OUT/runner.log"
# To stop a walk: kill $(cat neoforge/build/vm-client/runner.pid); the trap stops the client, Xvfb and PulseAudio.
echo $$ > "$WORK/runner.pid"
say() { echo "vm-client: $*" | tee -a "$LOG"; }
[ -f "$LAUNCH_SCRIPT" ] || { say "no $LAUNCH_SCRIPT (./gradlew createClientLaunchScript)"; exit 1; }

# runClient.sh is one line: (cd '<game dir>'; exec '<java>' '<arg>'...), each argument single-quoted
# by ModDevGradle; eval re-reads exactly those words (as tools/client/mac-client-lib.sh does).
line=$(grep -m1 "^(cd '" "$LAUNCH_SCRIPT")
GAMEDIR=$(printf '%s' "$line" | sed -n "s/^(cd '\([^']*\)'; exec .*/\1/p")
words=$(printf '%s' "$line" | sed -n "s/^(cd '[^']*'; exec \(.*\))$/\1/p")
[ -n "$GAMEDIR" ] && [ -n "$words" ] || { say "cannot read $LAUNCH_SCRIPT"; exit 1; }
eval "set -- $words"
JAVA="$1"; shift
DEV_ARGS=("$@")
[ -d "$GAMEDIR/mods" ] || { say "no $GAMEDIR/mods (./gradlew linkRunModsClient)"; exit 1; }
if [ -n "$STAGE_DIR" ]; then
    # The stage's kubejs/ (only data/deeptime in the lite set) is the pack's own way of giving a new
    # single-player world its rock tags: KubeJS adds kubejs/data as a global datapack.
    rm -rf "$GAMEDIR/kubejs"
    if [ -d "$STAGE_DIR/kubejs" ]; then cp -R "$STAGE_DIR/kubejs" "$GAMEDIR/kubejs"; say "kubejs/ from $STAGE_DIR: $(find "$GAMEDIR/kubejs" -type f | wc -l) files"; fi
    # The pack's config/ and defaultconfigs/ too, as a player's new instance has them (the mods write
    # the rest of config/ at launch). Without them every mod ran on its own defaults: Remove Reloading
    # Screen's hideOverlays = "ALL" (the pack sets "RELOADING") ended the start-up load early, Project
    # Atmosphere's client setup then failed on the already-loaded language map, mod loading stopped,
    # and Create World hung on its data reload (2026-10-22, the full MIC set).
    for d in config defaultconfigs; do
        rm -rf "${GAMEDIR:?}/$d"
        [ -d "$STAGE_DIR/$d" ] && cp -R "$STAGE_DIR/$d" "$GAMEDIR/$d"
    done
    say "config/ ($(find "$GAMEDIR/config" -type f 2>/dev/null | wc -l) files) and defaultconfigs/ ($(find "$GAMEDIR/defaultconfigs" -type f 2>/dev/null | wc -l)) from $STAGE_DIR"
fi
if [ -n "$ROCK_TAGS" ]; then
    if [ -n "$(find "$GAMEDIR/kubejs/data/deeptime" -name '*.json' 2>/dev/null | head -1)" ]; then
        say "rock tags: kubejs/data/deeptime already has files (the stage's own); $ROCK_TAGS not copied"
    else
        mkdir -p "$GAMEDIR/kubejs/data/deeptime"
        cp -R "$ROCK_TAGS/data/deeptime/." "$GAMEDIR/kubejs/data/deeptime/"
        say "rock tags: $(find "$GAMEDIR/kubejs/data/deeptime" -name '*.json' | wc -l) files from $ROCK_TAGS into kubejs/data/deeptime"
    fi
fi

# --- the display: Xvfb + Mesa llvmpipe, sound to a private null sink ---------------------------------
# Mesa, libglvnd and Xvfb come from the flake's own nixpkgs (flake.lock), the revision the client's
# JDK comes from. Mesa's driver loads inside the JVM, which has already loaded that revision's
# libraries (libstdc++, zlib, ...) by soname; the registry's newer Mesa then fails to load there and
# GLFW sees "GLX: No GLXFBConfigs returned" while glxinfo on the same display works (2026-10-19).
# Xvfb needs LIBGL_DRIVERS_PATH too: nixpkgs' xorg-server carries no DRI drivers of its own.
pinned() { nix build --no-link --print-out-paths --inputs-from "$REPO" "nixpkgs#$1" | head -1; }
MESA=$(pinned mesa)
GLVND=$(pinned libglvnd)
XVFB=$(pinned xorg-server)/bin/Xvfb
PA=$(pinned pulseaudio)/bin/pulseaudio
DISP="${DEEPTIME_DISPLAY:-:91}"
export DISPLAY=$DISP LD_LIBRARY_PATH="$MESA/lib:$GLVND/lib" __GLX_VENDOR_LIBRARY_NAME=mesa \
    __EGL_VENDOR_LIBRARY_DIRS="$MESA/share/glvnd/egl_vendor.d" LIBGL_DRIVERS_PATH="$MESA/lib/dri" \
    LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe LP_NUM_THREADS=4 LIBGL_DEBUG=verbose
XPID=""; PAPID=""; CPID=""
cleanup() {
    [ -n "$CPID" ] && kill "$CPID" 2>/dev/null && sleep 3 && kill -9 "$CPID" 2>/dev/null
    [ -n "$PAPID" ] && kill "$PAPID" 2>/dev/null
    [ -n "$XPID" ] && kill "$XPID" 2>/dev/null
    rm -rf "${PADIR:-/nonexistent-vm-client}"
}
trap cleanup EXIT
trap 'exit 130' INT; trap 'exit 143' TERM; trap 'exit 129' HUP
PADIR=$(mktemp -d /tmp/vmclient-pulse.XXXXXX)
PULSE_RUNTIME_PATH="$PADIR" HOME="$PADIR" setsid "$PA" -n --daemonize=no --exit-idle-time=-1 --use-pid-file=no \
    --disable-shm=yes --load="module-native-protocol-unix socket=$PADIR/native auth-anonymous=1" \
    --load="module-null-sink" > "$WORK/pulse.log" 2>&1 < /dev/null &
PAPID=$!
export PULSE_SERVER="unix:$PADIR/native" ALSOFT_DRIVERS=pulse
rm -f "/tmp/.X${DISP#:}-lock" "/tmp/.X11-unix/X${DISP#:}" 2>/dev/null
setsid "$XVFB" "$DISP" -screen 0 "${WSIZE}x24" -nolisten tcp +extension GLX > "$WORK/xvfb.log" 2>&1 < /dev/null &
XPID=$!
sleep 3
kill -0 "$XPID" 2>/dev/null || { say "Xvfb $DISP did not start ($(tail -3 "$WORK/xvfb.log"))"; exit 1; }
say "display $DISP (${WSIZE}, llvmpipe, GUI scale $GUISCALE${RUNSET:+, mods $RUNSET}), client JVM sees $THREADS processors"

write_options() {
    mkdir -p "$GAMEDIR/config"
    cat > "$GAMEDIR/options.txt" <<EOF
onboardAccessibility:false
pauseOnLostFocus:false
guiScale:$GUISCALE
narrator:0
tutorialStep:none
joinedFirstServer:true
skipMultiplayerWarning:true
fullscreen:false
soundCategory_master:0.0
soundCategory_music:0.0
renderDistance:8
EOF
    printf 'earlyWindowControl = false\n' > "$GAMEDIR/config/fml.toml"
    # Distant Horizons (in MIC) opens an "update available" screen over the title screen, which held
    # the walk there (2026-10-20, --mods mic); dh-client-check.sh turns its updater off the same way.
    local dhc="$GAMEDIR/config/DistantHorizons.toml"
    if [ -f "$dhc" ]; then
        sed -i 's/^\(\s*enableAutoUpdater\s*=\s*\).*/\1false/' "$dhc"
    else
        printf '[client]\n\t[client.advanced]\n\t\t[client.advanced.autoUpdater]\n\t\t\tenableAutoUpdater = false\n' > "$dhc"
    fi
    # KubeJS (in MIC) starts a local web server unless kubejs/config/web_server.json says otherwise
    # (Ben's instance has it off); it takes port 61423 or the next free one and the walk needs none.
    if [ -n "$RUNSET" ]; then
        local kws="$GAMEDIR/kubejs/config/web_server.json"
        mkdir -p "$GAMEDIR/kubejs/config"
        if [ -f "$kws" ]; then
            sed -i 's/"enabled"[[:space:]]*:[[:space:]]*true/"enabled": false/' "$kws"
        else
            printf '{\n  "enabled": false\n}\n' > "$kws"
        fi
    fi
}

# run_pass <pass> <-D...>: one client, start to quit; returns its exit status (124 on timeout).
run_pass() {
    local pass="$1"; shift
    local log="$OUT/client-$pass.log" start now last=0 st=""
    rm -rf "$GAMEDIR/screenshots" "$GAMEDIR/saves" "$GAMEDIR/deeptime/staging" "$GAMEDIR/crash-reports"
    write_options
    say "pass $pass: launching ($*)"
    (cd "$GAMEDIR" && exec "$JAVA" "-Xmx$XMX" "-XX:ActiveProcessorCount=$THREADS" "-Djava.io.tmpdir=$WORK/tmp" "$@" \
        "${DEV_ARGS[@]}" --width "${WSIZE%x*}" --height "${WSIZE#*x}") > "$log" 2>&1 < /dev/null &
    CPID=$!
    start=$(date +%s)
    while kill -0 "$CPID" 2>/dev/null; do
        sleep 5
        now=$(date +%s)
        if [ $((now - start)) -gt "$TIMEOUT" ]; then
            say "pass $pass: timed out after ${TIMEOUT}s; stopping the client"
            kill "$CPID" 2>/dev/null; sleep 10; kill -9 "$CPID" 2>/dev/null
            st=124; break
        fi
        # NeoForge's "Failed to wait for future" is a mod loading error: the client may go on (no
        # error screen when a mod ended the load early), but the walk would not test the player's game.
        if grep -a -q -e 'Preparing crash report' -e 'Failed to find a valid GLFW profile' -e 'Failed to wait for future' "$log" 2>/dev/null; then
            say "pass $pass: the client crashed or a mod failed to load ($(grep -a -m1 -e 'crash report' -e 'GLFW' -e 'Failed to wait for future' "$log" | cut -c1-200))"
            sleep 10; kill "$CPID" 2>/dev/null; st=1; break
        fi
        if [ $((now - last)) -ge 60 ]; then
            say "pass $pass: $((now - start))s, $(grep -aF '[deeptime-shot]' "$log" | tail -1 | sed 's/^.*\[deeptime-shot\] //' | cut -c1-200)"
            last=$now
        fi
    done
    if [ -z "$st" ]; then wait "$CPID"; st=$?; fi
    CPID=""
    say "pass $pass: client exited $st after $(( $(date +%s) - start ))s"
    grep -aF "[deeptime-shot]" "$log" | sed 's/^.*\[deeptime-shot\] //' | sed "s/^/[$pass] /" >> "$OUT/shots.txt"
    [ -d "$GAMEDIR/crash-reports" ] && cp -R "$GAMEDIR/crash-reports" "$OUT/crash-reports-$pass"
    return "$st"
}

failed=0
run_pass main -Ddeeptime.dev.screenshots=true "-Ddeeptime.dev.screenshots.create=$CREATE" ${PROPS[@]+"${PROPS[@]}"} || failed=1
cp "$GAMEDIR"/screenshots/*.png "$OUT/" 2>/dev/null
if [ "$GUIDANCE" = 1 ]; then
    run_pass guidance -Ddeeptime.dev.screenshots=true -Ddeeptime.dev.screenshots.create=false \
        -Ddeeptime.natives.forceFail=unsupported || failed=1
    cp "$GAMEDIR/screenshots/02-guidance-panel.png" "$OUT/20-guidance-panel.png" 2>/dev/null
    cp "$GAMEDIR/screenshots/03-create-world-natives-missing.png" "$OUT/21-create-world-natives-missing.png" 2>/dev/null
fi
grep -q "FAILED" "$OUT/shots.txt" 2>/dev/null && { say "the driver reported FAILED (shots.txt)"; failed=1; }
for p in "$OUT"/*.png; do
    [ -f "$p" ] && [ "$(wc -c < "$p")" -lt 10000 ] && say "suspicious: $(basename "$p") is under 10 kB (a blank frame?)"
done

# A contact sheet for the browser: every PNG with its name, and the driver's lines.
{
    echo '<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">'
    echo "<title>Client walk $NAME</title>"
    echo '<style>body{font:14px system-ui;background:#111;color:#ddd;margin:16px}figure{margin:0 0 24px}img{max-width:100%;border:1px solid #333}pre{white-space:pre-wrap;font-size:12px;background:#000;padding:8px}</style>'
    echo "<h1>Create World editor, real client ($NAME)</h1><p>$(date -u '+%Y-%m-%d %H:%M UTC'), $(git -C "$REPO" rev-parse --short HEAD 2>/dev/null)</p>"
    for p in "$OUT"/*.png; do
        [ -f "$p" ] || continue
        b=$(basename "$p")
        echo "<figure><img src=\"$b\" alt=\"$b\"><figcaption>$b</figcaption></figure>"
    done
    echo '<h2>Driver log</h2><pre>'
    sed 's/&/\&amp;/g; s/</\&lt;/g' "$OUT/shots.txt" 2>/dev/null
    echo '</pre>'
} > "$OUT/index.html"
say "$(ls "$OUT"/*.png 2>/dev/null | wc -l | tr -d ' ') PNGs in review/out/$NAME (index.html)"
exit "$failed"

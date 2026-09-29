#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Best-effort client check of the Create World screen's planet editor (P1-11; plan §9.8: the
# screens themselves are not reviewable otherwise). Boots the real NeoForge 1.21.1 client headlessly
# with HeadlessMC (MIT) under Xvfb + Mesa -- mic-climate's gametest-client.sh approach, adapted --
# with -Ddeeptime.dev.screenshots=true, so Deep Time's ScreenshotDriver walks the editor itself:
# Create World with the Deep Time preset, the setup page, the timelapse (map and globe), the review
# page, Accept, and (unless CREATE=0) the world's creation, which moves the staged data at server
# start. Screenshots land in $WORK/run/screenshots/, the log in $WORK/client.log.
#
#   tools/client/screenshots.sh <deeptime jar> [<libdeeptime_jni for this machine>]
#   GUIDANCE=1 tools/client/screenshots.sh <jar>      natives forced to fail: the guidance panel
#
# Builds happen on the Mac (tools/remote/mac.sh); pull the jar and, for a Linux client, a Linux
# library (tools/natives/build-natives.sh --only linux-x86_64-gnu): a dev jar carries only the
# build host's library, and -Ddeeptime.natives.path loads the one given here.
# Needs: ~/.minecraft (HMC_MC_DIR) with 1.21.1 and NeoForge 21.1.249 (installed on first use),
# network for the first download of HeadlessMC and Toroidal World, about 3 GB of RAM.

set -euo pipefail
cd "$(dirname "$0")/../.."
REPO="$PWD"

JAR="${1:?usage: tools/client/screenshots.sh <deeptime jar> [<native library>]}"
NATIVE="${2:-}"
JAR="$(realpath "$JAR")"
[ -n "$NATIVE" ] && NATIVE="$(realpath "$NATIVE")"

HMC_VERSION=2.10.0
MC_VERSION=1.21.1
TW_VERSION_ID=8YTSYi9y   # Toroidal World 0.19.3+mc1.21.1 (neoforge/gradle.properties tw_version)
WORK="${WORK:-$REPO/neoforge/build/client-shots}"
RUN="$WORK/run"
HMC_MC_DIR="${HMC_MC_DIR:-$HOME/.minecraft}"
LOG="$WORK/client.log"

mkdir -p "$WORK/HeadlessMC" "$RUN/mods" "$RUN/config" "$HMC_MC_DIR"
rm -rf "$RUN/screenshots" "$RUN/saves" "$RUN/deeptime/staging"

JAVA_BIN=$(nix develop -c sh -c 'command -v java')
echo "java: $JAVA_BIN"

cat > "$WORK/HeadlessMC/config.properties" <<EOF
hmc.java.versions=$JAVA_BIN
hmc.gamedir=$RUN
hmc.mcdir=$HMC_MC_DIR
hmc.offline=true
hmc.rethrow.launch.exceptions=true
hmc.exit.on.failed.command=true
hmc.assets.dummy=true
hmc.jline.enabled=false
EOF

LAUNCHER="$WORK/headlessmc-launcher-$HMC_VERSION.jar"
if [ ! -f "$LAUNCHER" ]; then
    echo "Downloading HeadlessMC $HMC_VERSION (MIT)..."
    curl -fsSL -o "$LAUNCHER" \
        "https://github.com/headlesshq/headlessmc/releases/download/$HMC_VERSION/headlessmc-launcher-$HMC_VERSION.jar"
fi

# Mods: Deep Time and Toroidal World only (mirror, never merge).
find "$RUN/mods" -name '*.jar' -delete
cp "$JAR" "$RUN/mods/"
TW=$(ls "$REPO"/smoke/cache/modrinth/toroidal_world-*.jar 2>/dev/null | head -1 || true)
if [ -z "$TW" ]; then
    TW="$WORK/toroidal_world.jar"
    if [ ! -f "$TW" ]; then
        url=$(curl -fsSL "https://api.modrinth.com/v2/version/$TW_VERSION_ID" | python3 -c 'import json,sys; print(json.load(sys.stdin)["files"][0]["url"])')
        curl -fsSL -o "$TW" "$url"
    fi
fi
cp "$TW" "$RUN/mods/"
echo "mods: $(ls "$RUN/mods")"

cat > "$RUN/options.txt" <<'EOF'
onboardAccessibility:false
pauseOnLostFocus:false
guiScale:2
narrator:0
tutorialStep:none
joinedFirstServer:true
EOF
# NeoForge's early loading window is a GL window of its own: off.
printf 'earlyWindowControl = false\n' > "$RUN/config/fml.toml"

hmc() { ( cd "$WORK" && "$JAVA_BIN" -jar "$LAUNCHER" --command "$@" ); }
[ -f "$HMC_MC_DIR/versions/$MC_VERSION/$MC_VERSION.json" ] || hmc download "$MC_VERSION"
ls -d "$HMC_MC_DIR"/versions/*neoforge* >/dev/null 2>&1 || hmc neoforge "$MC_VERSION" --java 21

# HeadlessMC 2.10.0's --jvm keeps only its first token (the game JVM got -Xmx3G and none of the
# -D properties: the open question in mic-climate's gametest-client.sh), so the properties travel
# in JAVA_TOOL_OPTIONS, which every JVM reads, the game's included (the launcher's own JVM ignores them).
create=true
[ "${CREATE:-1}" = 0 ] && create=false
PROPS="-Ddeeptime.dev.screenshots=true -Ddeeptime.dev.screenshots.create=$create"
[ -n "$NATIVE" ] && PROPS="$PROPS -Ddeeptime.natives.path=$NATIVE"
[ "${GUIDANCE:-0}" = 1 ] && PROPS="$PROPS -Ddeeptime.natives.forceFail=unsupported"
GAME_JVM="-Xmx3G"
echo "jvm: $GAME_JVM; JAVA_TOOL_OPTIONS=$PROPS"

# Real GL under Xvfb with Mesa: the prebuilt LWJGL natives dlopen libGL.so.1, which NixOS keeps in
# the store, so the store paths go on LD_LIBRARY_PATH by hand.
gl_libs=$(nix build --no-link --print-out-paths nixpkgs#libGL nixpkgs#libglvnd nixpkgs#mesa 2>/dev/null \
          | sed 's|$|/lib|' | paste -sd: -)
set +e
(
    cd "$WORK" && LD_LIBRARY_PATH="$gl_libs${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" JAVA_TOOL_OPTIONS="$PROPS" \
        timeout "${TIMEOUT:-1200}" nix shell nixpkgs#xvfb-run --command xvfb-run -a -s "-screen 0 1280x720x24" \
        "$JAVA_BIN" -jar "$LAUNCHER" --command launch '.*neoforge.*' -regex --jvm "$GAME_JVM"
) > "$LOG" 2>&1
status=$?
set -e

echo "--------------------------------------------------------------------"
grep -E "\[deeptime-shot\]|Moved staged|Copied staged|Bound Deep Time|accepted run" "$LOG" | sed 's/^.*\] //' | tail -40 || true
echo
ls -1 "$RUN/screenshots" 2>/dev/null || echo "no screenshots"
echo "exit $status; log $LOG"
grep -q "\[deeptime-shot\] FAILED" "$LOG" && exit 1
exit "$status"

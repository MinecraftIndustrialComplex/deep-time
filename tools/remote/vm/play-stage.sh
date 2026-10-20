#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Builds Ben's play server folder on the test VM from the MIC server set that `tools/smoke/install.sh mic`
# installed and from one workbench run's world data, in /data/ban/play/<hash>/: outside every repo copy,
# so tools/remote/mac-gc.sh never deletes it. Run it on the VM, from the laptop:
#
#   tools/remote/mac.sh -- tools/smoke/install.sh mic --preset earthlike_quick_16k --start supercontinent --length-myr 1000
#   tools/remote/mac.sh --no-sync -- tools/remote/vm/play-stage.sh --run <workbench run id> [--hash H]
#
# Then point /etc/default/deeptime-play at the folder and restart the unit (see STATUS.md, build log
# 2026-10-18):  PLAY_DIR=/data/ban/play/<hash>  PLAY_TOOLS_ENV=/data/ban/play/tools.env
#
#   --run ID    the workbench run whose world becomes planet/deeptime (<workbench runs>/<ID>/world)
#   --hash H    the folder's name (default: the first 8 characters of the commit in .sync-rev)
#   --label S   the motd's text after "Deep Time play test" (default "run <ID>")
#   --rock-tags DIR   the MIC rock-tag datapack copied into planet/datapacks/ (default: mic/Docs/datapacks/
#               deeptime-mic-rock-tags next to the checkout's pack, which `mac.sh --pack` syncs; override with
#               $DEEPTIME_ROCK_TAGS). The pack on master does not ship these tags (only its deep-time branch
#               does), so without them every rock class shows its vanilla fallback (ironstone as red
#               sandstone instead of Create's ochrum). Missing datapack = error, never a silent fallback.
#   --no-rock-tags    skip it (a pack that ships the tags itself)
#   --ops-from DIR    the folder whose ops.json is carried over (default: the folder /etc/default/deeptime-play
#               serves now), so the operators (Ben) stay operators across a rebuild; --ops-from none skips it
#   --devmap    also put the internal developer map (devmap/, mod id deeptime_devmap; never in the pack) in
#               mods/: the jar this checkout built, devmap/build/libs/deeptime_devmap-*.jar (build it first:
#               cd neoforge && ./gradlew :devmap:build). Opt-in, for dev play tests only.
#   --extra-mod JAR   also put JAR in mods/ (repeatable; --devmap is the same with the dev map's jar)
#   --mods-only only copy the --devmap / --extra-mod jars into the existing folder /data/ban/play/<--hash>/mods
#               (nothing else changes; then restart the play unit: sudo systemctl restart deeptime-play).
#               Pair it with play-client-stage.sh --mods-only --devmap so the client has the jar too.
#
# server.properties is Ben's (2026-10-18): port 25765 on 127.0.0.1 (the laptop's deeptime-play-tunnel
# forwards to it), RCON 25775, creative with force-gamemode, peaceful, no spawn protection, offline
# mode, view 10 / simulation 6, level-type deeptime:planet (a Petroff-Guyou world 13,312 blocks round), JVM -Xmx12G,
# Simple Voice Chat on 24464. The rcon password is generated once into <hash>/rcon.pw (mode 600, never
# committed). The JDK path is kept in /data/ban/play/tools.env and GC-rooted there.
set -euo pipefail
REPO="$(cd "$(dirname "$0")/../../.." && pwd)"
RUNS="${DEEPTIME_WORKBENCH_RUNS:-/data/ban/deeptime-cache/workbench/runs}"
PLAY=/data/ban/play
RUN=""; HASH=""; LABEL=""; TAGS="${DEEPTIME_ROCK_TAGS:-$(cd "$REPO/../.." && pwd)/Docs/datapacks/deeptime-mic-rock-tags}"; NO_TAGS=0
EXTRA=(); MODS_ONLY=0
OPS_FROM="$(sed -n 's/^PLAY_DIR=//p' /etc/default/deeptime-play 2>/dev/null || true)"
while [ $# -gt 0 ]; do
    case "$1" in
        --run) RUN="$2"; shift 2 ;;
        --hash) HASH="$2"; shift 2 ;;
        --label) LABEL="$2"; shift 2 ;;
        --rock-tags) TAGS="$2"; shift 2 ;;
        --no-rock-tags) NO_TAGS=1; shift ;;
        --ops-from) OPS_FROM="$2"; shift 2 ;;
        --devmap)
            DM=""
            for j in "$REPO"/devmap/build/libs/deeptime_devmap-*.jar; do
                case "$j" in *-sources.jar) ;; *) [ -f "$j" ] && DM="$j" ;; esac
            done
            [ -n "$DM" ] || { echo "play-stage.sh: no dev map jar in $REPO/devmap/build/libs (cd neoforge && ./gradlew :devmap:build)" >&2; exit 1; }
            EXTRA+=("$DM"); shift ;;
        --extra-mod) [ -f "$2" ] || { echo "play-stage.sh: no jar at $2" >&2; exit 1; }; EXTRA+=("$2"); shift 2 ;;
        --mods-only) MODS_ONLY=1; shift ;;
        *) echo "usage: play-stage.sh --run <id> [--hash H] [--label S] [--rock-tags DIR | --no-rock-tags] [--ops-from DIR|none] [--devmap] [--extra-mod JAR]..." >&2
           echo "       play-stage.sh --mods-only --hash H (--devmap | --extra-mod JAR)..." >&2; exit 2 ;;
    esac
done
# Adds the opt-in jars (the dev map) to a play folder's mods/, replacing an older build of the same mod.
add_extra_mods() {
    local dest="$1" j base stem
    for j in "${EXTRA[@]}"; do
        base="$(basename "$j")"
        stem="${base%%-[0-9]*}"   # deeptime_devmap-1.21.1-0.0.1.jar -> deeptime_devmap
        find "$dest/mods" -maxdepth 1 -name "$stem-*.jar" ! -name "$base" -print -delete
        cp -v "$j" "$dest/mods/"
    done
}
if [ "$MODS_ONLY" = 1 ]; then
    { [[ "$HASH" =~ ^[0-9A-Za-z._-]+$ ]] && [ -d "$PLAY/$HASH/mods" ]; } || { echo "play-stage.sh: --mods-only needs --hash <an existing play folder>" >&2; exit 2; }
    [ ${#EXTRA[@]} -gt 0 ] || { echo "play-stage.sh: --mods-only without --devmap or --extra-mod does nothing" >&2; exit 2; }
    echo "== extra mods into $PLAY/$HASH/mods"
    add_extra_mods "$PLAY/$HASH"
    echo "ready: $PLAY/$HASH (restart the play unit to load them: sudo systemctl restart deeptime-play)"
    exit 0
fi
[ -n "$RUN" ] || { echo "play-stage.sh: --run <workbench run id> is required" >&2; exit 2; }
[[ "$RUN" =~ ^[0-9a-f]+$ ]] || { echo "play-stage.sh: odd run id '$RUN'" >&2; exit 2; }
if [ -z "$HASH" ]; then
    HASH="$(sed -n 's/.*"commit": *"\([0-9a-f]\{8\}\).*/\1/p' "$REPO/.sync-rev")"
fi
[[ "$HASH" =~ ^[0-9A-Za-z._-]+$ ]] || { echo "play-stage.sh: cannot name the folder (no --hash, no .sync-rev commit)" >&2; exit 2; }
[ -n "$LABEL" ] || LABEL="run $RUN"
SET="$REPO/smoke/mic"
WORLD="$RUNS/$RUN/world"
DEST="$PLAY/$HASH"
[ -f "$SET/server/eula.txt" ] && [ -d "$SET/server/mods" ] || { echo "play-stage.sh: no server set at $SET/server (run tools/smoke/install.sh mic first)" >&2; exit 1; }
[ -f "$WORLD/deeptime/manifest.json" ] || [ -f "$WORLD/manifest.json" ] || { echo "play-stage.sh: no world data in $WORLD (is the run finished?)" >&2; exit 1; }
[ ! -e "$DEST" ] || { echo "play-stage.sh: $DEST exists; remove it or pick another --hash" >&2; exit 1; }
if [ "$NO_TAGS" = 0 ] && [ ! -f "$TAGS/pack.mcmeta" ]; then
    echo "play-stage.sh: no MIC rock-tag datapack at $TAGS (sync it with 'tools/remote/mac.sh --pack -- true', or pass --rock-tags DIR / --no-rock-tags)" >&2
    exit 1
fi

mkdir -p "$PLAY"
echo "== copying the server set to $DEST"
cp -a "$SET/server" "$DEST"
rm -rf "$DEST/mods-excluded" "$DEST/planet" "$DEST/logs" "$DEST/crash-reports" "$DEST/world" "$DEST/.fabric"
mkdir -p "$DEST/play-build"
cp -a "$SET/install.json" "$SET/excluded.txt" "$SET/excluded-why.txt" "$SET/neoforge.version" "$DEST/play-build/" 2>/dev/null || true
[ -f "$REPO/.sync-rev" ] && cp -a "$REPO/.sync-rev" "$DEST/play-build/"

echo "== tools.env (the JDK, GC-rooted) in $PLAY"
JAVA="$(sed -n 's/^JAVA=//p' "$SET/../tools.env")"
[ -x "$JAVA" ] || { echo "play-stage.sh: no JDK at '$JAVA'" >&2; exit 1; }
printf 'JAVA=%s\n' "$JAVA" > "$PLAY/tools.env"
JDK_STORE="$(dirname "$(dirname "$JAVA")")"
nix-store --add-root "$PLAY/jdk-root" --indirect -r "$JDK_STORE" >/dev/null 2>&1 || echo "   (could not GC-root $JDK_STORE; it stays as long as nothing collects it)"

echo "== server.properties, user_jvm_args.txt, voice chat"
[ -f "$DEST/rcon.pw" ] || { python3 -c "import secrets; print(secrets.token_hex(16))" > "$DEST/rcon.pw"; chmod 600 "$DEST/rcon.pw"; }
LEVEL_TYPE="$(sed -n 's/^level-type=//p' "$SET/server/server.properties")"
cat > "$DEST/server.properties" <<EOF
# Written by deep-time/tools/remote/vm/play-stage.sh -- Ben's play server, not a pack file.
server-ip=127.0.0.1
server-port=25765
enable-rcon=true
rcon.port=25775
rcon.password=$(cat "$DEST/rcon.pw")
broadcast-rcon-to-ops=false
online-mode=false
level-type=$LEVEL_TYPE
level-name=planet
spawn-protection=0
max-tick-time=-1
view-distance=10
simulation-distance=6
enable-command-block=true
sync-chunk-writes=false
gamemode=creative
force-gamemode=true
difficulty=peaceful
motd=Deep Time play test ($LABEL)
EOF
printf '%s\n' '-Xmx12G' '-XX:+UseG1GC' > "$DEST/user_jvm_args.txt"
if [ -f "$OPS_FROM/ops.json" ] && [ "$OPS_FROM" != "$DEST" ]; then
    cp -a "$OPS_FROM/ops.json" "$DEST/ops.json"
    echo "   operators carried over from $OPS_FROM/ops.json: $(grep -o '"name": *"[^"]*"' "$OPS_FROM/ops.json" | cut -d'"' -f4 | tr '\n' ' ')"
fi
# A pack mob's AI bug must not take the play server down (2026-10-20: Ice and Fire's hippocampus
# NPE'd in Toroidal World's seam distance check): NeoForge drops the erroring entity instead.
NF="$DEST/config/neoforge-server.toml"
if [ -f "$NF" ]; then
    sed -i 's/^removeErroringEntities = false/removeErroringEntities = true/; s/^removeErroringBlockEntities = false/removeErroringBlockEntities = true/' "$NF"
else
    printf '%s\n' 'removeErroringBlockEntities = true' 'removeErroringEntities = true' > "$NF"
fi
mkdir -p "$DEST/config/voicechat"
VC="$DEST/config/voicechat/voicechat-server.properties"
if [ -f "$VC" ] && grep -q '^port=' "$VC"; then sed -i 's/^port=.*/port=24464/' "$VC"; else echo 'port=24464' >> "$VC"; fi

echo "== world data: $WORLD -> planet/deeptime (without epochs/: StagingMover drops the epoch checkpoints too, a world never needs them)"
rm -rf "$DEST/planet" && mkdir "$DEST/planet"
SRC="$WORLD"; [ -f "$WORLD/manifest.json" ] || SRC="$WORLD/deeptime"
rsync -a --exclude=/epochs/ "$SRC/" "$DEST/planet/deeptime/"
echo "   $(du -sh "$DEST/planet/deeptime" | cut -f1) in planet/deeptime; level-type $LEVEL_TYPE"
if [ "$NO_TAGS" = 0 ]; then
    # A world datapack, as -PwithMic, the MIC smokes and the workbench preview add it (tools/smoke/install.sh --datapack).
    # Minecraft enables a datapack it finds in <world>/datapacks that level.dat does not list as disabled.
    mkdir -p "$DEST/planet/datapacks"
    cp -R "$TAGS" "$DEST/planet/datapacks/$(basename "$TAGS")"
    echo "== rock tags: $(find "$DEST/planet/datapacks/$(basename "$TAGS")/data" -name '*.json' | wc -l) JSON files from $TAGS in planet/datapacks/$(basename "$TAGS")"
else
    echo "== rock tags: skipped (--no-rock-tags)"
fi
if [ ${#EXTRA[@]} -gt 0 ]; then
    echo "== extra mods (opt-in): ${EXTRA[*]}"
    add_extra_mods "$DEST"
fi
echo "ready: $DEST (mods:$(find "$DEST/mods" -maxdepth 1 -name '*.jar' | wc -l))"

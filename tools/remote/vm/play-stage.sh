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
RUN=""; HASH=""; LABEL=""
while [ $# -gt 0 ]; do
    case "$1" in
        --run) RUN="$2"; shift 2 ;;
        --hash) HASH="$2"; shift 2 ;;
        --label) LABEL="$2"; shift 2 ;;
        *) echo "usage: play-stage.sh --run <id> [--hash H] [--label S]" >&2; exit 2 ;;
    esac
done
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
mkdir -p "$DEST/config/voicechat"
VC="$DEST/config/voicechat/voicechat-server.properties"
if [ -f "$VC" ] && grep -q '^port=' "$VC"; then sed -i 's/^port=.*/port=24464/' "$VC"; else echo 'port=24464' >> "$VC"; fi

echo "== world data: $WORLD -> planet/deeptime (without epochs/: StagingMover drops the epoch checkpoints too, a world never needs them)"
rm -rf "$DEST/planet" && mkdir "$DEST/planet"
SRC="$WORLD"; [ -f "$WORLD/manifest.json" ] || SRC="$WORLD/deeptime"
rsync -a --exclude=/epochs/ "$SRC/" "$DEST/planet/deeptime/"
echo "   $(du -sh "$DEST/planet/deeptime" | cut -f1) in planet/deeptime; level-type $LEVEL_TYPE"
echo "ready: $DEST (mods: $(find "$DEST/mods" -maxdepth 1 -name '*.jar' | wc -l))"

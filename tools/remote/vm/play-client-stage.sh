#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Builds the client's mod set and pack files for Ben's play server on the test VM, in
# /data/ban/play/<hash>-client/minecraft/ (outside every repo copy): the same MIC pack commit and the same
# Deep Time and Toroidal World jars as the play server folder /data/ban/play/<hash>/, so the client and
# the server run the same mod set. Run it on the VM, after tools/smoke/install.sh mic (and play-stage.sh, or
# before it: the install's server set is then the reference):
#
#   tools/remote/mac.sh --no-sync -- tools/remote/detach.sh start play-client-stage tools/remote/vm/play-client-stage.sh --hash <play folder name>
#
# then tools/remote/play-client-apply.py (on the laptop) carries it into the Prism instance. What it does:
#   1. packwiz-installer (-s client) from the install's pack snapshot (smoke/cache/pack-snapshot: the
#      pack's last commit, hashes refreshed), into an empty folder.
#   2. the same exclusions as the server set (tools/smoke/profiles/mic.excluded-mods: Cobblemon and
#      everything that requires it, tools/smoke/exclude_mods.py), because a client with a mod the
#      server lacks is refused at the handshake.
#   3. the Deep Time and Toroidal World jars of the play folder (a pack copy of either goes).
#   4. the client's mods compared with the server's: <hash>-client/client-vs-server.txt.
#
# --set lite (Ben, 2026-10-22: a small client for Deep Time tests): the same stage cut down to the mods
# Deep Time's own data names, their required dependencies and a fixed core (tools/smoke/lite_set.py, core
# list tools/smoke/profiles/mic-lite.core), in /data/ban/play/<hash>-client-lite/minecraft/. The server set
# it must match is the lite play folder <hash>-lite (play-stage.sh --set lite), or smoke/mic-lite/server
# (tools/smoke/install.sh mic-lite). The pack's kubejs/ (but kubejs/data/deeptime: the MIC rock tags, which
# KubeJS loads into a new single-player world; lite_set.py rule 3b), scripts/, resourcepacks/ and the config
# of the left-out mods are not staged; <hash>-client-lite/lite-why.txt says why each jar stays or goes. Without
# --set the stage is the full one, exactly as before.
#
# Opt-in extras (dev play tests only; never the pack's):
#   --devmap          also the internal developer map (mod id deeptime_devmap): the play folder's own
#                     deeptime_devmap-*.jar (play-stage.sh --devmap put it there), so both sides run one build
#   --extra-mod JAR   also JAR (repeatable)
#   --mods-only       the stage exists already: only copy the --devmap / --extra-mod jars into its mods/
#                     (then run play-client-apply.py as usual; its mods/ mirror carries them over)
set -euo pipefail
REPO="$(cd "$(dirname "$0")/../../.." && pwd)"
PLAY=/data/ban/play
HASH=""; DEVMAP=0; EXTRA=(); MODS_ONLY=0; SETNAME=full
while [ $# -gt 0 ]; do
    case "$1" in
        --hash) HASH="$2"; shift 2 ;;
        --devmap) DEVMAP=1; shift ;;
        --extra-mod) [ -f "$2" ] || { echo "play-client-stage.sh: no jar at $2" >&2; exit 1; }; EXTRA+=("$2"); shift 2 ;;
        --mods-only) MODS_ONLY=1; shift ;;
        --set) SETNAME="$2"; shift 2 ;;
        *) echo "usage: play-client-stage.sh --hash <play folder name> [--set full|lite] [--devmap] [--extra-mod JAR]... [--mods-only]" >&2; exit 2 ;;
    esac
done
[[ "$HASH" =~ ^[0-9A-Za-z._-]+$ ]] || { echo "play-client-stage.sh: --hash <play folder name> is required" >&2; exit 2; }
case "$SETNAME" in
    full) SUFFIX=""; SMOKE_SET=mic ;;
    lite) SUFFIX="-lite"; SMOKE_SET=mic-lite ;;
    *) echo "play-client-stage.sh: --set must be full or lite" >&2; exit 2 ;;
esac
SERVER="$PLAY/$HASH$SUFFIX"
OUT="$PLAY/$HASH-client$SUFFIX"
STAGE="$OUT/minecraft"
# The server's mods: the play folder's, or (when the world is not ready yet) the install's server set it is copied from.
[ -d "$SERVER/mods" ] || SERVER="$REPO/smoke/$SMOKE_SET/server"
[ -d "$SERVER/mods" ] || { echo "play-client-stage.sh: no server mods in $PLAY/$HASH$SUFFIX or $SERVER" >&2; exit 1; }
if [ "$DEVMAP" = 1 ]; then
    DM=""
    for j in "$SERVER"/mods/deeptime_devmap-*.jar; do [ -f "$j" ] && DM="$j"; done
    [ -n "$DM" ] || { echo "play-client-stage.sh: --devmap: no deeptime_devmap-*.jar in $SERVER/mods (play-stage.sh --devmap first)" >&2; exit 1; }
    EXTRA+=("$DM")
fi
# Adds the opt-in jars to the stage's mods/, replacing an older build of the same mod.
add_extra_mods() {
    local j base stem
    for j in "${EXTRA[@]}"; do
        base="$(basename "$j")"
        stem="${base%%-[0-9]*}"   # deeptime_devmap-1.21.1-0.0.1.jar -> deeptime_devmap
        find "$STAGE/mods" -maxdepth 1 -name "$stem-*.jar" ! -name "$base" -print -delete
        cp -v "$j" "$STAGE/mods/"
    done
}
if [ "$MODS_ONLY" = 1 ]; then
    [ -d "$STAGE/mods" ] || { echo "play-client-stage.sh: --mods-only: no stage at $STAGE" >&2; exit 2; }
    [ ${#EXTRA[@]} -gt 0 ] || { echo "play-client-stage.sh: --mods-only without --devmap or --extra-mod does nothing" >&2; exit 2; }
    echo "== extra mods into $STAGE/mods"
    add_extra_mods
    echo "ready: $STAGE (now tools/remote/play-client-apply.py --hash $HASH on the laptop)"
    exit 0
fi
[ ! -e "$STAGE" ] || { echo "play-client-stage.sh: $STAGE exists; remove it first" >&2; exit 1; }
SNAP="$REPO/smoke/cache/pack-snapshot"
BOOT="$REPO/smoke/cache/packwiz-installer-bootstrap.jar"
[ -f "$SNAP/pack.toml" ] && [ -f "$BOOT" ] || { echo "play-client-stage.sh: no pack snapshot or bootstrap jar under smoke/cache (run tools/smoke/install.sh mic)" >&2; exit 1; }
TOOLS_ENV="$PLAY/tools.env"; [ -f "$TOOLS_ENV" ] || TOOLS_ENV="$REPO/smoke/tools.env"
JAVA="$(sed -n 's/^JAVA=//p' "$TOOLS_ENV")"
[ -x "$JAVA" ] || { echo "play-client-stage.sh: no JDK in $TOOLS_ENV" >&2; exit 1; }
mkdir -p "$STAGE"

echo "== packwiz-installer -s client into $STAGE"
( cd "$STAGE" && "$JAVA" -jar "$BOOT" -g -s client "$SNAP/pack.toml" )

echo "== exclusions (mic.excluded-mods) and the Deep Time / Toroidal World jars"
mkdir -p "$OUT"
: > "$OUT/client-excluded.txt"; : > "$OUT/client-excluded-why.txt"
python3 "$REPO/tools/smoke/exclude_mods.py" "$STAGE" "$REPO/tools/smoke/profiles/mic.excluded-mods" "$OUT/client-excluded.txt" "$OUT/client-excluded-why.txt"
find "$STAGE/mods" -maxdepth 1 \( -iname 'deeptime-*.jar' -o -iname 'toroidal*world*.jar' \) -print -delete
for j in "$SERVER"/mods/deeptime-*.jar "$SERVER"/mods/toroidal_world*.jar; do cp -v "$j" "$STAGE/mods/"; done
if [ ${#EXTRA[@]} -gt 0 ]; then
    echo "== extra mods (opt-in): ${EXTRA[*]}"
    add_extra_mods
fi
if [ "$SETNAME" = lite ]; then
    # After the Deep Time, Toroidal World and dev map jars are in: the derivation keeps the core, the mods
    # Deep Time's data names and their required dependencies (client side: BOTH and CLIENT mods).
    echo "== the Deep Time lite set (tools/smoke/lite_set.py)"
    python3 -I "$REPO/tools/smoke/lite_set.py" apply "$STAGE" --side client --excluded "$OUT/client-excluded.txt" \
        --excluded-why "$OUT/client-excluded-why.txt" --prune-files
    cp "$STAGE/lite-why.txt" "$OUT/lite-why.txt"; rm -f "$STAGE/lite-why.txt"
fi

echo "== client vs server mods"
python3 - "$SERVER/mods" "$STAGE/mods" "$OUT/client-vs-server.txt" <<'PY'
import os, sys
server, client, report = sys.argv[1:]
s = {f for f in os.listdir(server) if f.endswith(".jar")}
c = {f for f in os.listdir(client) if f.endswith(".jar")}
lines = [f"server mods: {len(s)}", f"client mods: {len(c)}", f"in both: {len(s & c)}",
         f"client only ({len(c - s)}):"] + [f"  {f}" for f in sorted(c - s)] + \
        [f"server only ({len(s - c)}):"] + [f"  {f}" for f in sorted(s - c)]
open(report, "w").write("\n".join(lines) + "\n")
print("\n".join(lines))
PY
echo "ready: $STAGE (report $OUT/client-vs-server.txt)"

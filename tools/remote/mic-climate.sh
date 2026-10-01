#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Run a command in a mic-climate checkout on Ben's M6 Mac mini (P1-25), the way tools/remote/mac.sh
# runs Deep Time's: mic-climate's own scripts assume the laptop (its flake's dev shell pulls in
# Linux-only libraries), so this syncs the checkout and the local jars its build reads and runs the
# command inside Deep Time's dev shell (JDK 21) on the Mac.
#
#   tools/remote/mic-climate.sh [--src <mic-climate checkout>] [--deeptime-jar <jar>] [--server]
#                               [--pull <path>]... -- <command>
#
#   --src           the mic-climate checkout to sync (default: …/mic/mods/mic-climate, read-only here)
#   --deeptime-jar  copy this Deep Time jar to the Mac copy's libs/deeptime-1.21.1.jar (the
#                   compileOnly input of mic-climate's deep-time branch)
#   --server        hold Deep Time's Mac server lock while the command runs (anything that boots
#                   Minecraft: ./gametest.sh, the smoke server)
#   --pull <path>   copy <path> (relative to the checkout) back from the Mac afterwards; repeatable
#
# The Mac copy lives at /Volumes/Graphide/ban/mic/mods/<checkout name>. What mic-climate's build.gradle
# reads from outside its checkout is synced beside it, at the same relative paths:
#   ../Destroy/a/b/c/build/libs/destroy-1.21.1-0.4.1.jar, ../Destroy/_Migration-Toolkit-1.21/petrolpark_local_maven/
#   ../mic-destroy-electric/build/libs/mic_destroy_electric-1.21.1-1.0.0.jar
# and its gitignored libs/ (Project Atmosphere, Gabou's Libs) comes from the local mic-climate checkout.
# Project Atmosphere's licence forbids redistribution: the jar only ever goes to Ben's own Mac.

set -euo pipefail

HOST="${DEEPTIME_MAC:-ban@20.120.160.3}"
SSD=/Volumes/Graphide/ban
CACHE="$SSD/deeptime-cache"
MODS=/home/greencheetah/Projects/mic/mods
DT_REPO="$(git rev-parse --show-toplevel)"

SRC="$MODS/mic-climate"; DTJAR=""; SERVER=0; PULLS=()
while [ $# -gt 0 ]; do
    case "$1" in
        --src) SRC="$2"; shift 2 ;;
        --deeptime-jar) DTJAR="$2"; shift 2 ;;
        --server) SERVER=1; shift ;;
        --pull) PULLS+=("$2"); shift 2 ;;
        --) shift; break ;;
        *) break ;;
    esac
done
[ $# -gt 0 ] || { echo "usage: tools/remote/mic-climate.sh [--src DIR] [--deeptime-jar JAR] [--server] [--pull P]... -- <command>" >&2; exit 2; }
CMD="$*"
SRC="$(cd "$SRC" && pwd)"
NAME="$(basename "$SRC")"
REMOTE="$SSD/mic/mods/$NAME"

SSH=(ssh -n -o BatchMode=yes -o ConnectTimeout=15 -o ServerAliveInterval=30 "$HOST")
# The VM's firewall follows the laptop's address (vm-access.sh): repair it before the first call.
. "$(dirname "${BASH_SOURCE[0]}")/vm-access.sh"
vm_access_ensure "$HOST" || { echo "$(basename "$0"): cannot reach $HOST" >&2; exit 1; }
RSYNC=(rsync -a -e "ssh -o BatchMode=yes")

if "${SSH[@]}" 'test -e /Users/Shared/graphide-nix-maintenance -o -e /Users/Shared/graphide-nix-unhealthy'; then
    echo "mic-climate.sh: the Mac's Nix is under maintenance or unhealthy; not running" >&2
    exit 75
fi
"${SSH[@]}" "test -d $SSD && mkdir -p '$REMOTE/libs' '$SSD/mic/mods/Destroy/a/b/c/build/libs' \
    '$SSD/mic/mods/Destroy/_Migration-Toolkit-1.21' '$SSD/mic/mods/mic-destroy-electric/build/libs' '$CACHE/gradle' '$CACHE/tmp'" \
    || { echo "mic-climate.sh: $SSD is not mounted on $HOST" >&2; exit 1; }

"${RSYNC[@]}" --delete --exclude=/build/ --exclude=/.gradle/ --exclude=/run/ --exclude=/run-gametest/ \
    --exclude=/libs/ --exclude=/.graphide/ --exclude=/.git/ --exclude=/smoke/cache/ --exclude=/smoke/server/ \
    --exclude=/smoke/logs/ "$SRC/" "$HOST:$REMOTE/"
# The local jars mic-climate's build reads (never published; see the header).
"${RSYNC[@]}" "$MODS/mic-climate/libs/" "$HOST:$REMOTE/libs/"
[ -z "$DTJAR" ] || "${RSYNC[@]}" "$DTJAR" "$HOST:$REMOTE/libs/deeptime-1.21.1.jar"
"${RSYNC[@]}" "$MODS/Destroy/a/b/c/build/libs/destroy-1.21.1-0.4.1.jar" "$HOST:$SSD/mic/mods/Destroy/a/b/c/build/libs/"
"${RSYNC[@]}" "$MODS/Destroy/_Migration-Toolkit-1.21/petrolpark_local_maven" "$HOST:$SSD/mic/mods/Destroy/_Migration-Toolkit-1.21/"
"${RSYNC[@]}" "$MODS/mic-destroy-electric/build/libs/mic_destroy_electric-1.21.1-1.0.0.jar" "$HOST:$SSD/mic/mods/mic-destroy-electric/build/libs/"

if [ "$SERVER" = 1 ]; then
    LOCK="$CACHE/server.lock"
    CMD="while ! mkdir '$LOCK' 2>/dev/null; do
           age=\$(( \$(date +%s) - \$(stat -c %Y '$LOCK' 2>/dev/null || /usr/bin/stat -f %m '$LOCK' 2>/dev/null || date +%s) ))
           if [ \$age -gt 10800 ]; then echo 'mic-climate.sh: breaking a stale server lock'; rm -rf '$LOCK'; continue; fi
           echo \"mic-climate.sh: waiting for the server lock (held by \$(cat '$LOCK/owner' 2>/dev/null))\"; sleep 30
         done
         echo '$NAME' > '$LOCK/owner'; trap \"rm -rf '$LOCK'\" EXIT INT TERM
         $CMD"
fi
# Deep Time's dev shell (JDK 21), the GC-rooted profile mac.sh keeps; built on first use.
KEY="$(cat "$DT_REPO/flake.nix" "$DT_REPO/flake.lock" | sha256sum | cut -c1-16)"
PROFILE="$CACHE/devshell-$KEY"
DTREMOTE="$SSD/mic/mods/$(basename "$DT_REPO")"
INNER="export GRADLE_USER_HOME='$CACHE/gradle' TMPDIR='$CACHE/tmp' && { [ -e '$PROFILE' ] || (cd '$DTREMOTE' && nix develop --profile '$PROFILE' -c true); } && cd '$REMOTE' && exec nix develop '$PROFILE' -c zsh -c $(printf '%q' "$CMD")"
set +e
"${SSH[@]}" "exec zsh -lc $(printf '%q' "$INNER")"
status=$?
set -e
for p in "${PULLS[@]}"; do
    mkdir -p "$SRC/$(dirname "$p")"
    "${RSYNC[@]}" "$HOST:$REMOTE/$p" "$SRC/$(dirname "$p")/" || echo "mic-climate.sh: could not pull $p" >&2
done
exit "$status"

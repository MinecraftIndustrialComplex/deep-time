#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Starts the play server for deeptime-play.service: the JDK from the server set's smoke/tools.env,
# NeoForge's own argument files, the pid in deeptime-cache/play-server.pid (exec keeps the pid).
set -euo pipefail
: "${PLAY_DIR:?PLAY_DIR (the server folder) is not set}"
TOOLS_ENV="${PLAY_TOOLS_ENV:-$PLAY_DIR/../../tools.env}"
JAVA="$(sed -n 's/^JAVA=//p' "$TOOLS_ENV")"
[ -x "$JAVA" ] || { echo "play-server.sh: no JDK at '$JAVA' (from $TOOLS_ENV)" >&2; exit 1; }
cd "$PLAY_DIR"
echo $$ > /data/ban/deeptime-cache/play-server.pid
exec "$JAVA" @user_jvm_args.txt @libraries/net/neoforged/neoforge/21.1.249/unix_args.txt nogui

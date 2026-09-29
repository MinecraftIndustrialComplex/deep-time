#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The Mac side of tools/client/ev-assets.sh (read that first); runs inside the repo's dev shell on
# the Mac, under the Deep Time server lock, while the laptop holds the desktop's drive lease.
#
# 1. installs the ev-client set (tools/smoke/install.sh ev-client) if it is missing;
# 2. writes run/config/dynamic_asset_generator.json: full_cache (DAG writes every resource it makes
#    under run/.cache/dynamic_asset_generator/) and time_resources (times.log); other keys at their
#    defaults (DAG 6.1.2's ModConfig.getDefault);
# 3. starts `./gradlew runClient -PrunSet=ev-client` in ban's desktop session: `sudo launchctl asuser`
#    (macOS gives a GUI app spawned from sshd no window server) with this dev shell's environment;
# 4. waits for the block atlas ("Created: … minecraft:textures/atlas/blocks.png-atlas"), by which
#    time every block model and texture has been asked for, gives DAG 20 s more, stops the client;
# 5. writes summary.json, then turns the cache into a resource pack (tools/client/ev-assets-pack.py).
set -euo pipefail
cd "$(dirname "$0")/../.."
REPO="$PWD"
OUT="$REPO/review/out/q10/ev-assets"
RUN="$REPO/run"
LOG="$OUT/client.log"
TIMEOUT="${EV_ASSETS_TIMEOUT:-1500}"

rm -rf "$OUT" && mkdir -p "$OUT"
if [ ! -f smoke/ev-client/install.json ]; then
    tools/smoke/install.sh ev-client 2>&1 | tail -3
fi

mkdir -p "$RUN/config"
rm -rf "$RUN/.cache/dynamic_asset_generator" "$RUN/logs" "$RUN/crash-reports"
printf '%s\n' '{"full_cache": true, "palette_extraction_force_clustering_cutoff": 1000000, "time_resources": true, "keyed_cache": true}' \
    > "$RUN/config/dynamic_asset_generator.json"
cat > "$RUN/options.txt" <<'EOF'
onboardAccessibility:false
pauseOnLostFocus:false
narrator:0
tutorialStep:none
joinedFirstServer:true
soundCategory_master:0.0
EOF

# The launcher: this shell's environment (the dev shell: java, GRADLE_USER_HOME, …), then Gradle.
LAUNCH="$OUT/launch.sh"
{
    echo '#!/bin/bash'
    export -p | grep -v -E '^declare -[a-zA-Z-]* (BASHOPTS|SHELLOPTS|BASH_VERSINFO|EUID|PPID|UID|SHLVL|OLDPWD|PWD|_)='
    echo "cd '$REPO/neoforge'"
    echo "exec ./gradlew runClient --no-daemon --console=plain -PrunSet=ev-client"
} > "$LAUNCH"
chmod +x "$LAUNCH"

UID_NUM="$(id -u)"; USER_NAME="$(id -un)"
echo "ev-assets: starting the client in gui/$UID_NUM (log $LOG)"
T0=$(date +%s)
sudo -n launchctl asuser "$UID_NUM" sudo -n -u "$USER_NAME" -H /bin/bash "$LAUNCH" > "$LOG" 2>&1 &
LAUNCHER_PID=$!

# The client JVM and this run's Gradle wrapper: java processes whose command line names this
# checkout (other agents' worktrees have other paths; Gradle daemons name only the cache).
client_pids() {
    ps -axo pid=,command= | grep -F "$REPO/" | grep -E 'java' | grep -v -E 'GradleWrapperMain|GradleDaemon|grep' \
        | awk '{print $1}' || true
}
wrapper_pids() { ps -axo pid=,command= | grep -F "$REPO/neoforge/gradle/wrapper" | grep -v grep | awk '{print $1}' || true; }
stop_client() {
    local pids; pids="$(client_pids)"
    [ -n "$pids" ] && kill $pids 2>/dev/null || true
    sleep 5
    pids="$(client_pids) $(wrapper_pids)"; [ -n "${pids// /}" ] && kill -9 $pids 2>/dev/null || true
    wait "$LAUNCHER_PID" 2>/dev/null || true
}
trap stop_client EXIT

ATLAS=""; STATUS=timeout
while [ $(( $(date +%s) - T0 )) -lt "$TIMEOUT" ]; do
    if grep -q -E 'Created: .*minecraft:textures/atlas/blocks\.png-atlas' "$LOG" 2>/dev/null; then
        ATLAS=$(( $(date +%s) - T0 )); STATUS=atlas; break
    fi
    if grep -q -E 'Preparing crash report|BUILD FAILED|Crash report saved' "$LOG" 2>/dev/null; then STATUS=crash; break; fi
    if ! kill -0 "$LAUNCHER_PID" 2>/dev/null; then STATUS=exited; break; fi
    sleep 5
done
echo "ev-assets: $STATUS after $(( $(date +%s) - T0 )) s"
[ "$STATUS" = atlas ] && sleep 20
# RSS of the client JVM at the atlas, for the page's cost table.
RSS_KB="$(for p in $(client_pids); do ps -o rss= -p "$p"; done | sort -n | tail -1 | tr -d ' ')"
stop_client
trap - EXIT

cp "$RUN/logs/latest.log" "$OUT/latest.log" 2>/dev/null || true
cp -R "$RUN/crash-reports" "$OUT/" 2>/dev/null || true
CACHE="$RUN/.cache/dynamic_asset_generator"
[ -d "$CACHE" ] && cp -R "$CACHE" "$OUT/cache"
[ -f "$CACHE/times.log" ] && cp "$CACHE/times.log" "$OUT/times.log"


python3 - "$OUT" "$STATUS" "${ATLAS:-}" "${RSS_KB:-}" <<'PY'
import json, os, re, sys
out, status, atlas, rss = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
path = os.path.join(out, "client.log")
log = open(path, errors="replace").read() if os.path.exists(path) else ""
ev_lines = [l for l in log.splitlines() if re.search(r"(?i)excavated|dynamic.?asset|dynamicassetgenerator", l)][:60]
# The client's own timeline from its log: ModLauncher start, resource reload, block atlas.
stamps = {}
for key, rx in (("modlauncher", r"ModLauncher running"), ("reload", r"Reloading ResourceManager"),
                ("atlas", r"Created: .*minecraft:textures/atlas/blocks\.png-atlas")):
    for l in log.splitlines():
        m = re.search(r"\[(\d\d):(\d\d):(\d\d)", l)
        if m and re.search(rx, l):
            stamps[key] = int(m.group(1)) * 3600 + int(m.group(2)) * 60 + int(m.group(3))
            break
summary = {"status": status, "seconds_launch_to_block_atlas": int(atlas) if atlas else None,
           "client_rss_mb_at_atlas": int(rss) // 1024 if rss else None,
           "client_log_seconds": {k: v - stamps["modlauncher"] for k, v in stamps.items()} if "modlauncher" in stamps else {},
           "ev_and_dag_log_lines": ev_lines}
json.dump(summary, open(os.path.join(out, "summary.json"), "w"), indent=1)
PY
python3 tools/client/ev-assets-pack.py "$OUT"

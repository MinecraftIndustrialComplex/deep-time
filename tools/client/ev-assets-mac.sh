#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The Mac side of tools/client/ev-assets.sh (read that first); runs ON the Mac from the repo root,
# inside `tools/remote/mac.sh --no-sync --server` (Deep Time's server lock) and the repo's dev shell.
#
#   tools/client/ev-assets-mac.sh --m6lock <path to graphide's m6lock.sh> [--timeout S] [--lease-wait S]
#
# 1. checks the ev-client set is linked into the client's run/mods (the laptop side ran
#    `./gradlew -PrunSet=ev-client linkRunModsClient` after `tools/smoke/install.sh ev-client`);
# 2. writes run/config/dynamic_asset_generator.json: full_cache (DAG writes every resource it makes
#    under run/.cache/dynamic_asset_generator/) and time_resources (times.log); other keys at their
#    defaults (DAG 6.1.2's ModConfig.getDefault);
# 3. boots the client through tools/client/mac-client-lib.sh, the launch every real-client run
#    shares (the drive lease on graphide's M6 lock, renewed while it runs; ModDevGradle's dev launch
#    in ban's desktop session through `sudo launchctl asuser`; only this run's JVM is ever
#    stopped), without ScreenshotDriver: the client just loads. The pass ends 20 s after the block
#    atlas is created ("Created: … minecraft:textures/atlas/blocks.png-atlas"), by which time every
#    block model and texture has been asked for, and the client is stopped;
# 4. writes summary.json, then turns the cache into a resource pack (tools/client/ev-assets-pack.py).
set -euo pipefail
cd "$(dirname "$0")/../.."
REPO="$PWD"
source tools/client/mac-client-lib.sh

NAME=q10-ev-assets; M6LOCK=""; TIMEOUT="${EV_ASSETS_TIMEOUT:-1500}"; LEASE_WAIT=600
while [ $# -gt 0 ]; do
    case "$1" in
        --m6lock) M6LOCK="$2"; shift 2 ;;
        --timeout) TIMEOUT="$2"; shift 2 ;;
        --lease-wait) LEASE_WAIT="$2"; shift 2 ;;
        *) echo "ev-assets-mac: unknown argument $1" >&2; exit 2 ;;
    esac
done

OUT="$REPO/review/out/q10/ev-assets"
PURPOSE="Deep Time Q10: a Minecraft client makes Excavated Variants textures"
WINDOW_SIZE=""
XMX=4G   # what the run configuration gave the client before (a quarter of the 16 GB)
mc_start

# A screenshot run's `linkRunModsClient` (no -PrunSet) empties what an ev-client one linked.
compgen -G "$GAMEDIR/mods/*xcavated*" >/dev/null \
    || { mc_log "the ev-client set is not linked into $GAMEDIR/mods (tools/client/ev-assets.sh runs tools/smoke/install.sh ev-client and ./gradlew -PrunSet=ev-client linkRunModsClient)"; exit 1; }

mkdir -p "$GAMEDIR/config"
rm -rf "$GAMEDIR/.cache/dynamic_asset_generator" "$GAMEDIR/logs" "$GAMEDIR/crash-reports"
printf '%s\n' '{"full_cache": true, "palette_extraction_force_clustering_cutoff": 1000000, "time_resources": true, "keyed_cache": true}' \
    > "$GAMEDIR/config/dynamic_asset_generator.json"

MC_READY_REGEX='Created: .*minecraft:textures/atlas/blocks\.png-atlas'
MC_READY_GRACE=20
MC_FAIL_REGEX='Preparing crash report|Crash report saved'
set +e
mc_run_pass ev
st=$?
set -e
mc_release
case "$st" in 0) STATUS=atlas ;; 124) STATUS=timeout ;; *) STATUS=crash ;; esac
echo "ev-assets: $STATUS after ${MC_READY_AT:-?} s to the block atlas"
mv "$OUT/client-ev.log" "$OUT/client.log" 2>/dev/null || true
[ -s "$OUT/shots.txt" ] || rm -f "$OUT/shots.txt"   # the screenshot driver is off: nothing to list

cp "$GAMEDIR/logs/latest.log" "$OUT/latest.log" 2>/dev/null || true
cp -R "$GAMEDIR/crash-reports" "$OUT/" 2>/dev/null || true
CACHE="$GAMEDIR/.cache/dynamic_asset_generator"
[ -d "$CACHE" ] && cp -R "$CACHE" "$OUT/cache"
[ -f "$CACHE/times.log" ] && cp "$CACHE/times.log" "$OUT/times.log"

python3 - "$OUT" "$STATUS" "${MC_READY_AT:-}" "${MC_STOP_RSS_KB:-}" <<'PY'
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
[ "$STATUS" = atlas ]

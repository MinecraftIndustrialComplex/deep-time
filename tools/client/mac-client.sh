#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The Mac half of tools/client/mac-screenshots.sh (run that, on the laptop; never this directly).
# Runs ON Ben's M6 Mac mini from the repo root, inside `tools/remote/mac.sh --no-sync --server`,
# so Deep Time's server lock is held for the whole run (the client starts an integrated server).
#
# Boots the real NeoForge 1.21.1 client -- ModDevGradle's own dev launch (build/moddev/runClient.sh,
# written by `./gradlew createClientLaunchScript`): Deep Time from this checkout's build output,
# Toroidal World's bundle from run/mods (linkRunModsClient) -- in the Mac's logged-in desktop
# session, where LWJGL gets a real window and the M6's GPU (no Xvfb, no HeadlessMC). With
# -Ddeeptime.dev.screenshots=true, ScreenshotDriver walks Create World -> editor -> run/timelapse
# -> review -> Accept -> world creation and writes the PNGs from the GL framebuffer itself, so no
# macOS screen-recording permission is involved (screencapture would need one; nothing grants it).
#
#   tools/client/mac-client.sh --name <name> --m6lock <path to graphide's m6lock.sh>
#                              [--no-guidance] [--no-create] [--size WxH] [--timeout S] [--lease-wait S]
#
# The desktop is shared with graphide's agents: before the first window opens this takes a `drive`
# lease on graphide's M6 lock (utilities/scripts/mactest/m6lock.sh; excludes other desktop walks and
# the perf lane, never breaks anyone's lease; waits up to --lease-wait seconds, default 600), renews
# it every minute while the client runs (TTL 5 min, so a dead runner's lease lapses on its own) and
# releases it at the end. Any failure, timeout, lost lease or signal kills only this run's client
# (its pid, checked against a -Ddeeptime.client.run=<id> marker on its command line).
#
# Passes: `main` (create=true unless --no-create: 01..10) and, unless --no-guidance, `guidance`
# (-Ddeeptime.natives.forceFail=unsupported: its two shots land as 11-guidance-panel and
# 12-create-world-natives-missing). Output: review/out/<name>/ (PNGs, client-<pass>.log,
# shots.txt, runner.log, crash reports); tools/remote/mac.sh --pull brings it to the laptop.

set -euo pipefail
cd "$(dirname "$0")/../.."
REPO="$PWD"

NAME=""; M6LOCK=""; GUIDANCE=1; CREATE=true; SIZE=""; TIMEOUT=1200; LEASE_WAIT=600
while [ $# -gt 0 ]; do
    case "$1" in
        --name) NAME="$2"; shift 2 ;;
        --m6lock) M6LOCK="$2"; shift 2 ;;
        --no-guidance) GUIDANCE=0; shift ;;
        --no-create) CREATE=false; shift ;;
        --size) SIZE="$2"; shift 2 ;;
        --timeout) TIMEOUT="$2"; shift 2 ;;
        --lease-wait) LEASE_WAIT="$2"; shift 2 ;;
        *) echo "mac-client: unknown argument $1" >&2; exit 2 ;;
    esac
done
[[ "$NAME" =~ ^[A-Za-z0-9._-]+$ ]] || { echo "mac-client: --name must be one plain word (got '$NAME')" >&2; exit 2; }
[ -z "$SIZE" ] || [[ "$SIZE" =~ ^[0-9]+x[0-9]+$ ]] || { echo "mac-client: --size is WxH (got '$SIZE')" >&2; exit 2; }
[ "$(uname -s)" = Darwin ] || { echo "mac-client: runs only on the Mac (real-client runs never use the laptop)" >&2; exit 2; }
[ -f "$M6LOCK" ] || { echo "mac-client: no m6lock.sh at '$M6LOCK' (mac-screenshots.sh copies graphide's there)" >&2; exit 2; }

OUT="$REPO/review/out/$NAME"
WORK="$REPO/neoforge/build/mac-client"
LAUNCH_SCRIPT="$REPO/neoforge/build/moddev/runClient.sh"
RUNID="$NAME-$$-$(date +%s)"
HOLDER="deeptime-client:$(basename "$REPO")"
PURPOSE="Deep Time real-client screenshots ($NAME)"
TTL=300
ME="$(id -un)"
MY_UID="$(id -u)"
# System tools by absolute path: inside the dev shell, GNU coreutils come first on PATH.
SUDO=/usr/bin/sudo LAUNCHCTL=/bin/launchctl STAT=/usr/bin/stat

rm -rf "$OUT" "$WORK"
mkdir -p "$OUT" "$WORK/tmp"
RUNNER_LOG="$OUT/runner.log"
log() { echo "mac-client: $*" | tee -a "$RUNNER_LOG"; }
echo $$ > "$WORK/runner.pid"

[ -f "$LAUNCH_SCRIPT" ] || { log "no $LAUNCH_SCRIPT (./gradlew createClientLaunchScript first; mac-screenshots.sh does)"; exit 1; }

# --- the desktop session -------------------------------------------------------------------
# One account, ban, and one GUI session, ban's (graphide's mactest/remote/session.sh). A GUI
# process started straight from sshd has no WindowServer; `launchctl asuser` puts it in the
# session's bootstrap context (root only, hence sudo -n, then back to this account).
if ! "$LAUNCHCTL" print "gui/$MY_UID" >/dev/null 2>&1; then
    log "BLOCKED: $ME has no GUI session on this Mac (nobody logged in at the screen). Ben: log in as $ME at the Mac (or over Screen Sharing), then re-run."
    exit 4
fi
"$SUDO" -n true 2>/dev/null || { log "BLOCKED: 'sudo -n' needs a password here; launchctl asuser needs it (graphide's mactest provisioning sets it up)"; exit 4; }
case "$("$STAT" -f %Su /dev/console 2>/dev/null)" in
    root|loginwindow|"") log "warning: the screen is locked (untested case): check the PNGs for blank frames" ;;
esac

# --- the dev launch ------------------------------------------------------------------------------
# runClient.sh is one line: (cd '<game dir>'; exec '<java>' '<arg>'...), each argument single-quoted
# by ModDevGradle (CreateLaunchScriptTask.escapeShellArg); eval re-reads exactly those words.
line=$(grep -m1 "^(cd '" "$LAUNCH_SCRIPT")
GAMEDIR=$(printf '%s' "$line" | sed -n "s/^(cd '\([^']*\)'; exec .*/\1/p")
words=$(printf '%s' "$line" | sed -n "s/^(cd '[^']*'; exec \(.*\))$/\1/p")
[ -n "$GAMEDIR" ] && [ -n "$words" ] || { log "cannot read $LAUNCH_SCRIPT (ModDevGradle changed its format?)"; exit 1; }
eval "set -- $words"
JAVA="$1"; shift
DEV_ARGS=("$@")
[ -d "$GAMEDIR/mods" ] || { log "no $GAMEDIR/mods (./gradlew linkRunModsClient links Toroidal World there)"; exit 1; }

LEASED=0
lease() { /bin/bash "$M6LOCK" "$@"; }
# ps, not kill -0: the sudo in front of the GUI launch is root's, and kill -0 on it fails (EPERM).
alive() { [ -n "$1" ] && ps -p "$1" >/dev/null 2>&1; }
kill_client() {
    local pid
    pid=$(cat "$WORK/client.pid" 2>/dev/null) || return 0
    alive "$pid" || return 0
    # Only our own JVM: the pid must still carry this run's marker.
    ps -o command= -p "$pid" 2>/dev/null | grep -qF -- "-Ddeeptime.client.run=$RUNID" || return 0
    echo "mac-client: stopping the client (pid $pid)" >>"$RUNNER_LOG"
    kill -TERM "$pid" 2>/dev/null || true
    for _ in $(seq 1 15); do alive "$pid" || return 0; sleep 1; done
    kill -KILL "$pid" 2>/dev/null || true
}
cleanup() {
    trap - EXIT INT TERM HUP PIPE
    {
        kill_client
        if [ "$LEASED" = 1 ]; then lease release drive "$HOLDER" >/dev/null 2>&1 || true; echo "mac-client: drive lease released"; fi
        rm -f "$WORK/runner.pid"
    } >>"$RUNNER_LOG" 2>&1
}
# Set before the lease is taken, so every exit after it releases it (releasing a lease this holder
# does not hold is a no-op, so LEASED goes to 1 before the first attempt). mac-screenshots.sh TERMs
# this runner when the laptop side is interrupted; a merely dropped connection lets the pass run out.
trap cleanup EXIT
trap 'cleanup; exit 130' INT
trap 'cleanup; exit 143' TERM
trap 'cleanup; exit 129' HUP
trap 'cleanup; exit 141' PIPE

# --- the M6 lock's drive lease ---------------------------------------------------------------
LEASED=1
waited=0
until out=$(lease acquire drive "$HOLDER" "$PURPOSE" "$TTL" 2>&1); do
    rc=$?
    if [ "$rc" != 75 ]; then log "m6lock acquire failed (exit $rc): $out"; exit 1; fi
    if [ "$waited" -ge "$LEASE_WAIT" ]; then
        log "the desktop is busy after ${waited}s: $out. Not breaking anyone's lease; try again later (or --lease-wait)."
        exit 75
    fi
    log "waiting for the desktop: $out"
    sleep 30
    waited=$((waited + 30))
done
log "drive lease taken: $HOLDER ($(lease status))"

# The game's own options: no sound on Ben's speakers, no narrator, no pause when the window
# loses focus (the in-world shot), no onboarding screens. NeoForge's early window is off.
write_options() {
    mkdir -p "$GAMEDIR/config"
    cat > "$GAMEDIR/options.txt" <<'EOF'
onboardAccessibility:false
pauseOnLostFocus:false
guiScale:2
narrator:0
tutorialStep:none
joinedFirstServer:true
skipMultiplayerWarning:true
fullscreen:false
soundCategory_master:0.0
soundCategory_music:0.0
EOF
    printf 'earlyWindowControl = false\n' > "$GAMEDIR/config/fml.toml"
}

# run_pass <pass> <extra -D...>: one client, start to quit. Returns the client's exit status.
run_pass() {
    local pass="$1"; shift
    local log="$OUT/client-$pass.log" launch="$WORK/launch-$pass.sh"
    rm -rf "$GAMEDIR/screenshots" "$GAMEDIR/saves" "$GAMEDIR/deeptime/staging" "$GAMEDIR/crash-reports"
    rm -f "$WORK/client.pid" "$WORK/client.exit"
    write_options
    local jvm=("-Ddeeptime.client.run=$RUNID" -Xmx3G "-Djava.io.tmpdir=$WORK/tmp" -Ddeeptime.dev.screenshots=true "$@")
    local prog=()
    [ -z "$SIZE" ] || prog=(--width "${SIZE%x*}" --height "${SIZE#*x}")
    {
        echo '#!/bin/bash'
        echo "cd $(printf '%q' "$GAMEDIR") || exit 90"
        printf '%q ' "$JAVA" "${jvm[@]}" "${DEV_ARGS[@]}" ${prog[@]+"${prog[@]}"}
        echo "> $(printf '%q' "$log") 2>&1 &"
        echo "pid=\$!"
        echo "echo \$pid > $(printf '%q' "$WORK/client.pid")"
        echo "/usr/bin/caffeinate -d -w \$pid &"   # the display stays awake while the client runs
        echo "wait \$pid"
        echo "echo \$? > $(printf '%q' "$WORK/client.exit")"
    } > "$launch"
    log "pass $pass: launching in gui/$MY_UID ($*)"
    "$SUDO" -n "$LAUNCHCTL" asuser "$MY_UID" "$SUDO" -n -u "$ME" -H \
        env -i HOME="$HOME" USER="$ME" LOGNAME="$ME" LANG=en_US.UTF-8 PATH=/usr/bin:/bin:/usr/sbin:/sbin \
        TMPDIR="$WORK/tmp" /bin/bash "$launch" >>"$RUNNER_LOG" 2>&1 &
    local gui_job=$! start now last_refresh last_note pid status=""
    start=$(date +%s); last_refresh=$start; last_note=$start
    while [ ! -f "$WORK/client.exit" ]; do
        sleep 2
        now=$(date +%s)
        pid=$(cat "$WORK/client.pid" 2>/dev/null || true)
        if ! alive "$gui_job" && ! alive "$pid"; then
            sleep 2
            [ -f "$WORK/client.exit" ] && break
            log "pass $pass: the launch in the GUI session failed (see runner.log)"
            status=1; break
        fi
        if [ $((now - start)) -gt "$TIMEOUT" ]; then
            log "pass $pass: timed out after ${TIMEOUT}s; stopping the client"
            kill_client
            status=124; break
        fi
        if [ $((now - last_refresh)) -ge 60 ]; then
            # acquire renews our own lease; it fails only if the lease lapsed and someone else took the desktop.
            if ! out=$(lease acquire drive "$HOLDER" "$PURPOSE" "$TTL" 2>&1); then
                log "pass $pass: lost the drive lease ($out); stopping the client"
                kill_client
                status=1; break
            fi
            last_refresh=$now
        fi
        if [ $((now - last_note)) -ge 30 ]; then
            log "pass $pass: $((now - start))s, $(grep -F '[deeptime-shot]' "$log" 2>/dev/null | tail -1 | sed 's/^.*\[deeptime-shot\] //' || true)"
            last_note=$now
        fi
    done
    wait "$gui_job" 2>/dev/null || true
    [ -n "$status" ] || status=$(cat "$WORK/client.exit")
    log "pass $pass: client exited $status after $(( $(date +%s) - start ))s"
    grep -F "[deeptime-shot]" "$log" 2>/dev/null | sed 's/^.*\[deeptime-shot\] //' | sed "s/^/[$pass] /" >> "$OUT/shots.txt" || true
    [ -d "$GAMEDIR/crash-reports" ] && cp -R "$GAMEDIR/crash-reports" "$OUT/crash-reports-$pass"
    return "$status"
}

failed=0
set +e
run_pass main "-Ddeeptime.dev.screenshots.create=$CREATE"; st=$?
set -e
[ "$st" = 0 ] || failed=1
cp "$GAMEDIR"/screenshots/*.png "$OUT/" 2>/dev/null || true
if [ "$GUIDANCE" = 1 ]; then
    set +e
    run_pass guidance -Ddeeptime.dev.screenshots.create=false -Ddeeptime.natives.forceFail=unsupported; st=$?
    set -e
    [ "$st" = 0 ] || failed=1
    # Its 01 (Create World before the editor) repeats the main pass's; only the two new shots are kept.
    cp "$GAMEDIR/screenshots/02-guidance-panel.png" "$OUT/11-guidance-panel.png" 2>/dev/null || true
    cp "$GAMEDIR/screenshots/03-create-world-natives-missing.png" "$OUT/12-create-world-natives-missing.png" 2>/dev/null || true
fi

expected=(01-create-world-deeptime 02-setup-no-timeline 03-setup-ready 04-run-map-live 05-run-globe-live
          06-run-done 07-run-scrubbed 08-review 09-create-world-accepted)
[ "$CREATE" = true ] && expected+=(10-in-world)
[ "$GUIDANCE" = 1 ] && expected+=(11-guidance-panel 12-create-world-natives-missing)
for e in "${expected[@]}"; do
    if [ ! -f "$OUT/$e.png" ]; then log "missing: $e.png"; failed=1
    elif [ "$(wc -c < "$OUT/$e.png")" -lt 10000 ]; then log "suspicious: $e.png is under 10 kB (a blank frame?)"; fi
done
grep -q "FAILED" "$OUT/shots.txt" 2>/dev/null && { log "the driver reported FAILED (shots.txt)"; failed=1; }
log "$(ls "$OUT"/*.png 2>/dev/null | wc -l | tr -d ' ') PNGs in review/out/$NAME"
exit "$failed"

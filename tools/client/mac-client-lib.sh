# SPDX-License-Identifier: LGPL-3.0-or-later
# shellcheck shell=bash disable=SC2034
#
# The Mac-side launch path of every real-client run: sourced (never run) by
# tools/client/mac-client.sh (ScreenshotDriver's walk, from tools/client/mac-screenshots.sh) and
# tools/client/ev-assets-mac.sh (Excavated Variants' textures, from tools/client/ev-assets.sh).
# It runs ON Ben's M6 Mac mini from the repo root, inside `tools/remote/mac.sh --no-sync --server`,
# so Deep Time's server lock is held for the whole run (the client starts an integrated server).
#
# Boots the real NeoForge 1.21.1 client -- ModDevGradle's own dev launch (build/moddev/runClient.sh,
# written by `./gradlew createClientLaunchScript`): Deep Time from this checkout's build output,
# Toroidal World's bundle (and, with -PrunSet=<set>, that server set) from run/mods
# (linkRunModsClient) -- in the Mac's logged-in desktop session, where LWJGL gets a real window and
# the M6's GPU (no Xvfb, no HeadlessMC).
#
# The desktop is shared with graphide's agents: before the first window opens `mc_start` takes a
# `drive` lease on graphide's M6 lock (utilities/scripts/mactest/m6lock.sh; excludes other desktop
# walks and the perf lane, never breaks anyone's lease; waits up to LEASE_WAIT seconds), `mc_run_pass`
# renews it every minute while the client runs (TTL 5 min, so a dead runner's lease lapses on its
# own) and it is released at the end. Any failure, timeout, lost lease or signal kills only this
# run's client (its pid, checked against a -Ddeeptime.client.run=<id> marker on its command line).
#
# The caller sets, before mc_start:
#   NAME        one plain word: the run id and the lease holder's purpose
#   OUT         the output folder (emptied and created); the pass logs and runner.log go there
#   M6LOCK      graphide's m6lock.sh, copied to the Mac (the laptop side does that)
#   PURPOSE     the lease's purpose text
#   TIMEOUT     seconds per pass (default 1200)      LEASE_WAIT  seconds to wait for the desktop (600)
#   SIZE        optional WxH window size             XMX         the client's -Xmx (default 3G)
# then calls
#   mc_start                     the session and sudo checks, the launch line, the traps, the lease;
#                                exits 4 (no GUI session, no sudo, or the screen is locked), 75 (desktop busy), 1 (other);
#                                sets REPO, WORK, GAMEDIR, RUNNER_LOG
#   mc_run_pass <pass> [-D...]   one client, start to end, its JVM properties after the pass name;
#                                log $OUT/client-<pass>.log; returns the client's exit status, or
#                                124 on timeout, or 1 when the launch failed / the lease was lost /
#                                MC_FAIL_REGEX matched. With MC_READY_REGEX set the pass is over
#                                MC_READY_GRACE seconds after that line shows in the log (a client
#                                that never quits by itself: the client is stopped, status 0);
#                                MC_READY_AT is then the seconds from launch to the line and
#                                MC_STOP_RSS_KB the client's resident size when it was stopped.
#   mc_release                   stops a leftover client and releases the lease (idempotent; also
#                                runs on exit and on INT/TERM/HUP/PIPE)
#   mc_log <text>                a line to the console and $OUT/runner.log
# (MC_READY_REGEX, MC_READY_GRACE and MC_FAIL_REGEX are read at each mc_run_pass.)

# Where this file is, taken when it is sourced (the caller may change directory later).
MC_LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

mc_start() {
    cd "$MC_LIB_DIR/../.." || exit 1
    REPO="$PWD"
    TIMEOUT="${TIMEOUT:-1200}"; LEASE_WAIT="${LEASE_WAIT:-600}"; SIZE="${SIZE:-}"; XMX="${XMX:-3G}"
    [[ "$NAME" =~ ^[A-Za-z0-9._-]+$ ]] || { echo "mac-client: NAME must be one plain word (got '$NAME')" >&2; exit 2; }
    [ -z "$SIZE" ] || [[ "$SIZE" =~ ^[0-9]+x[0-9]+$ ]] || { echo "mac-client: --size is WxH (got '$SIZE')" >&2; exit 2; }
    [ "$(uname -s)" = Darwin ] || { echo "mac-client: runs only on the Mac (real-client runs never use the laptop)" >&2; exit 2; }
    [ -f "$M6LOCK" ] || { echo "mac-client: no m6lock.sh at '$M6LOCK' (the laptop side copies graphide's there)" >&2; exit 2; }

    WORK="$REPO/neoforge/build/mac-client"
    LAUNCH_SCRIPT="$REPO/neoforge/build/moddev/runClient.sh"
    RUNID="$NAME-$$-$(date +%s)"
    HOLDER="deeptime-client:$(basename "$REPO")"
    TTL=300
    ME="$(id -un)"
    MY_UID="$(id -u)"
    # System tools by absolute path: inside the dev shell, GNU coreutils come first on PATH.
    SUDO=/usr/bin/sudo LAUNCHCTL=/bin/launchctl

    rm -rf "$OUT" "$WORK"
    mkdir -p "$OUT" "$WORK/tmp"
    RUNNER_LOG="$OUT/runner.log"
    echo $$ > "$WORK/runner.pid"
    trap 'rm -f "$WORK/runner.pid"' EXIT   # early exits leave no stale pid for the laptop's interrupt to TERM

    [ -f "$LAUNCH_SCRIPT" ] || { mc_log "no $LAUNCH_SCRIPT (./gradlew createClientLaunchScript first; the laptop side does)"; exit 1; }

    # --- the desktop session ---------------------------------------------------------------------
    # One account, ban, and one GUI session, ban's (graphide's mactest/remote/session.sh). A GUI
    # process started straight from sshd has no WindowServer; `launchctl asuser` puts it in the
    # session's bootstrap context (root only, hence sudo -n, then back to this account).
    if ! "$LAUNCHCTL" print "gui/$MY_UID" >/dev/null 2>&1; then
        mc_log "BLOCKED: $ME has no GUI session on this Mac (nobody logged in at the screen). Ben: log in as $ME at the Mac (or over Screen Sharing), then re-run."
        exit 4
    fi
    "$SUDO" -n true 2>/dev/null || { mc_log "BLOCKED: 'sudo -n' needs a password here; launchctl asuser needs it (graphide's mactest provisioning sets it up)"; exit 4; }
    # A locked screen still has the account logged in, so the check above passes: ioreg's
    # IOConsoleLocked says. (/dev/console's owner does not: it is root here with ban at the desk.)
    # Nothing types a password; the run just stops, before anything is leased or started.
    if /usr/sbin/ioreg -n Root -d1 -w0 2>/dev/null | grep -q '"IOConsoleLocked" = Yes'; then
        mc_log "BLOCKED: the screen is locked. Ben: unlock the Mac (nothing here types a password), then re-run."
        exit 4
    fi

    # --- the dev launch --------------------------------------------------------------------------
    # runClient.sh is one line: (cd '<game dir>'; exec '<java>' '<arg>'...), each argument single-quoted
    # by ModDevGradle (CreateLaunchScriptTask.escapeShellArg); eval re-reads exactly those words.
    local line words
    line=$(grep -m1 "^(cd '" "$LAUNCH_SCRIPT")
    GAMEDIR=$(printf '%s' "$line" | sed -n "s/^(cd '\([^']*\)'; exec .*/\1/p")
    words=$(printf '%s' "$line" | sed -n "s/^(cd '[^']*'; exec \(.*\))$/\1/p")
    [ -n "$GAMEDIR" ] && [ -n "$words" ] || { mc_log "cannot read $LAUNCH_SCRIPT (ModDevGradle changed its format?)"; exit 1; }
    eval "set -- $words"
    JAVA="$1"; shift
    DEV_ARGS=("$@")
    [ -d "$GAMEDIR/mods" ] || { mc_log "no $GAMEDIR/mods (./gradlew linkRunModsClient links Toroidal World there)"; exit 1; }

    LEASED=0
    # Set before the lease is taken, so every exit after it releases it (releasing a lease this holder
    # does not hold is a no-op, so LEASED goes to 1 before the first attempt). The laptop side TERMs
    # this runner when it is interrupted; a merely dropped connection lets the pass run out.
    trap mc_release EXIT
    trap 'mc_release; exit 130' INT
    trap 'mc_release; exit 143' TERM
    trap 'mc_release; exit 129' HUP
    trap 'mc_release; exit 141' PIPE

    # --- the M6 lock's drive lease -----------------------------------------------------------------
    LEASED=1
    local waited=0 out rc
    until out=$(mc_lease acquire drive "$HOLDER" "$PURPOSE" "$TTL" 2>&1); do
        rc=$?
        if [ "$rc" != 75 ]; then mc_log "m6lock acquire failed (exit $rc): $out"; exit 1; fi
        if [ "$waited" -ge "$LEASE_WAIT" ]; then
            mc_log "the desktop is busy after ${waited}s: $out. Not breaking anyone's lease; try again later (or --lease-wait)."
            exit 75
        fi
        mc_log "waiting for the desktop: $out"
        sleep 30
        waited=$((waited + 30))
    done
    mc_log "drive lease taken: $HOLDER ($(mc_lease status))"
}

mc_log() { echo "mac-client: $*" | tee -a "$RUNNER_LOG"; }
mc_lease() { /bin/bash "$M6LOCK" "$@"; }
# ps, not kill -0: the sudo in front of the GUI launch is root's, and kill -0 on it fails (EPERM).
mc_alive() { [ -n "$1" ] && ps -p "$1" >/dev/null 2>&1; }

mc_kill_client() {
    local pid
    pid=$(cat "$WORK/client.pid" 2>/dev/null) || return 0
    mc_alive "$pid" || return 0
    # Only our own JVM: the pid must still carry this run's marker.
    ps -o command= -p "$pid" 2>/dev/null | grep -qF -- "-Ddeeptime.client.run=$RUNID" || return 0
    echo "mac-client: stopping the client (pid $pid)" >>"$RUNNER_LOG"
    kill -TERM "$pid" 2>/dev/null || true
    for _ in $(seq 1 15); do mc_alive "$pid" || return 0; sleep 1; done
    kill -KILL "$pid" 2>/dev/null || true
}

mc_release() {
    trap - EXIT INT TERM HUP PIPE
    {
        mc_kill_client
        if [ "${LEASED:-0}" = 1 ]; then
            mc_lease release drive "$HOLDER" >/dev/null 2>&1 || true
            LEASED=0
            echo "mac-client: drive lease released"
        fi
        rm -f "$WORK/runner.pid"
    } >>"$RUNNER_LOG" 2>&1
}

# The game's own options: no sound on Ben's speakers, no narrator, no pause when the window
# loses focus (the in-world shot), no onboarding screens. NeoForge's early window is off.
mc_write_options() {
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

# mc_run_pass <pass> <extra -D...>: one client, start to quit. Returns the client's exit status.
mc_run_pass() {
    local pass="$1"; shift
    local log="$OUT/client-$pass.log" launch="$WORK/launch-$pass.sh"
    rm -rf "$GAMEDIR/screenshots" "$GAMEDIR/saves" "$GAMEDIR/deeptime/staging" "$GAMEDIR/crash-reports"
    rm -f "$WORK/client.pid" "$WORK/client.exit"
    mc_write_options
    local jvm=("-Ddeeptime.client.run=$RUNID" "-Xmx$XMX" "-Djava.io.tmpdir=$WORK/tmp" "$@")
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
    mc_log "pass $pass: launching in gui/$MY_UID ($*)"
    "$SUDO" -n "$LAUNCHCTL" asuser "$MY_UID" "$SUDO" -n -u "$ME" -H \
        env -i HOME="$HOME" USER="$ME" LOGNAME="$ME" LANG=en_US.UTF-8 PATH=/usr/bin:/bin:/usr/sbin:/sbin \
        TMPDIR="$WORK/tmp" /bin/bash "$launch" >>"$RUNNER_LOG" 2>&1 &
    local gui_job=$! start now last_refresh last_note pid status="" out ready_at=""
    MC_READY_AT=""; MC_STOP_RSS_KB=""
    start=$(date +%s); last_refresh=$start; last_note=$start
    while [ ! -f "$WORK/client.exit" ]; do
        sleep 2
        now=$(date +%s)
        pid=$(cat "$WORK/client.pid" 2>/dev/null || true)
        if ! mc_alive "$gui_job" && ! mc_alive "$pid"; then
            sleep 2
            [ -f "$WORK/client.exit" ] && break
            mc_log "pass $pass: the launch in the GUI session failed (see runner.log)"
            status=1; break
        fi
        if [ $((now - start)) -gt "$TIMEOUT" ]; then
            mc_log "pass $pass: timed out after ${TIMEOUT}s; stopping the client"
            mc_kill_client
            status=124; break
        fi
        if [ -n "${MC_FAIL_REGEX:-}" ] && grep -qE -- "$MC_FAIL_REGEX" "$log" 2>/dev/null; then
            mc_log "pass $pass: the client log matches /$MC_FAIL_REGEX/ (a crash); stopping the client"
            mc_kill_client
            status=1; break
        fi
        if [ -n "${MC_READY_REGEX:-}" ]; then
            if [ -z "$ready_at" ] && grep -qE -- "$MC_READY_REGEX" "$log" 2>/dev/null; then
                ready_at=$((now - start)); MC_READY_AT=$ready_at
                mc_log "pass $pass: ready after ${ready_at}s; stopping the client in ${MC_READY_GRACE:-0}s"
            fi
            if [ -n "$ready_at" ] && [ $((now - start)) -ge $((ready_at + ${MC_READY_GRACE:-0})) ]; then
                MC_STOP_RSS_KB=$(ps -o rss= -p "$pid" 2>/dev/null | tr -d ' ' || true)
                mc_kill_client
                status=0; break
            fi
        fi
        if [ $((now - last_refresh)) -ge 60 ]; then
            # acquire renews our own lease; it fails only if the lease lapsed and someone else took the desktop.
            if ! out=$(mc_lease acquire drive "$HOLDER" "$PURPOSE" "$TTL" 2>&1); then
                mc_log "pass $pass: lost the drive lease ($out); stopping the client"
                mc_kill_client
                status=1; break
            fi
            last_refresh=$now
        fi
        if [ $((now - last_note)) -ge 30 ]; then
            mc_log "pass $pass: $((now - start))s, $(grep -F '[deeptime-shot]' "$log" 2>/dev/null | tail -1 | sed 's/^.*\[deeptime-shot\] //' || true)"
            last_note=$now
        fi
    done
    wait "$gui_job" 2>/dev/null || true
    [ -n "$status" ] || status=$(cat "$WORK/client.exit")
    mc_log "pass $pass: client exited $status after $(( $(date +%s) - start ))s"
    grep -F "[deeptime-shot]" "$log" 2>/dev/null | sed 's/^.*\[deeptime-shot\] //' | sed "s/^/[$pass] /" >> "$OUT/shots.txt" || true
    [ -d "$GAMEDIR/crash-reports" ] && cp -R "$GAMEDIR/crash-reports" "$OUT/crash-reports-$pass"
    return "$status"
}

# SPDX-License-Identifier: LGPL-3.0-or-later
# shellcheck shell=bash disable=SC2034
#
# The laptop-side plumbing every real-client run shares: sourced (never run) by
# tools/client/mac-screenshots.sh and tools/client/ev-assets.sh, from the repo root, after they have
# set REPO (the checkout's root) and TOOL (their own name, for messages). The Mac side of the same
# runs is tools/client/mac-client-lib.sh.
#
#   mcl_check_m6lock    graphide's m6lock.sh must exist here (M6LOCK_SH overrides where it is read
#                       from): the protocol has one source, so a run fails early, before any build
#   mcl_push_m6lock     copies it fresh to the Mac ($CACHE/m6lock.sh), where the runner takes the
#                       drive lease with it; nothing is leased from the laptop
#   mcl_stop_on_interrupt   Ctrl-C/TERM on the laptop TERMs the Mac runner through its pid file
#                       (neoforge/build/mac-client/runner.pid), which stops the client, releases
#                       the lease (and the admitted job ends with mac.sh)
#
# It also names the host and the Mac's layout, the same as tools/remote/mac.sh: HOST, REMOTE (this
# checkout's copy), CACHE (the caches on the SSD; the m6lock copy sits there), SSH.

HOST="${DEEPTIME_MAC:-ban@mac-mini}"
SSD=/Volumes/Graphide/ban
REMOTE="$SSD/mic/mods/$(basename "$REPO")"
CACHE="$SSD/deeptime-cache"
SSH=(ssh -o BatchMode=yes -o ConnectTimeout=15)
M6LOCK_SH="${M6LOCK_SH:-/home/greencheetah/Projects/graphide/graphide/utilities/scripts/mactest/m6lock.sh}"

mcl_check_m6lock() {
    [ -f "$M6LOCK_SH" ] || { echo "$TOOL: no m6lock.sh at $M6LOCK_SH (graphide's utilities/scripts/mactest/; set M6LOCK_SH)" >&2; exit 2; }
}

mcl_push_m6lock() {
    "${SSH[@]}" "$HOST" "mkdir -p '$CACHE' && cat > '$CACHE/m6lock.sh'" < "$M6LOCK_SH"
}

mcl_stop_mac_side() {
    trap - INT TERM
    echo "$TOOL: interrupted; stopping the Mac side" >&2
    "${SSH[@]}" -n "$HOST" "p=\$(cat '$REMOTE/neoforge/build/mac-client/runner.pid' 2>/dev/null) && kill -TERM \$p" || true
    exit 130
}

mcl_stop_on_interrupt() { trap mcl_stop_mac_side INT TERM; }

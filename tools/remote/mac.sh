#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Run a command on Ben's M6 Mac mini instead of the laptop (Ben's standing rule: cargo, Gradle,
# GameTests, smoke/planet servers and BlueMap run on the Mac; the laptop only edits and runs git).
#
#   tools/remote/mac.sh [--no-sync] [--pack] [--pull <path>]... -- <command>
#
#   --no-sync      skip the rsync of this checkout (the Mac copy is already current)
#   --pack         also sync the MIC pack (…/mic/pack, with its .git) for tools/smoke/install.sh mic
#   --server       hold the Mac's Deep Time server lock while the command runs: use it for every
#                  command that boots Minecraft (gametest.sh, gametest-planet.sh, smoke run.py,
#                  BlueMap). Several agents build at once, but only one test server runs at a time
#                  (fixed ports, 16 GB shared with graphide's agents). Waits for the lock.
#   --pull <path>  after the command, copy <path> (relative to the repo root) back from the Mac;
#                  repeatable (e.g. --pull smoke/mic/report.md --pull neoforge/build/gametest.log)
#   <command>      run from the repo root on the Mac, already inside the repo's dev shell, e.g.
#                  'cargo test --workspace'  or  './gametest.sh'
#
# Layout on the Mac, all on the external SSD (/Volumes/Graphide; the internal disk is small and
# /nix is already on the SSD):
#   /Volumes/Graphide/ban/mic/mods/<checkout name>   this checkout (a worktree gets its own copy)
#   /Volumes/Graphide/ban/mic/pack                   the MIC pack (--pack)
#   /Volumes/Graphide/ban/deeptime-cache/{gradle,cargo,xwin,tmp}   GRADLE_USER_HOME, CARGO_HOME, …
#
# The Mac has no GitHub access, so the checkout is rsynced (tracked + untracked, minus build
# outputs and runtimes) and a Mac-side git index is refreshed with `git add -A`: a Nix flake only
# sees files git tracks. That index is never committed or pushed; commits happen on the laptop.
# Output that matters (reports, logs) comes back with --pull. Exit status is the command's.

set -euo pipefail

HOST="${DEEPTIME_MAC:-ban@m6-mac-mini}"
SSD=/Volumes/Graphide/ban
REPO="$(git rev-parse --show-toplevel)"
NAME="$(basename "$REPO")"
REMOTE="$SSD/mic/mods/$NAME"
CACHE="$SSD/deeptime-cache"

SYNC=1; PACK=0; SERVER=0; PULLS=()
while [ $# -gt 0 ]; do
    case "$1" in
        --no-sync) SYNC=0; shift ;;
        --pack) PACK=1; shift ;;
        --server) SERVER=1; shift ;;
        --pull) PULLS+=("$2"); shift 2 ;;
        --) shift; break ;;
        *) break ;;
    esac
done
[ $# -gt 0 ] || { echo "usage: tools/remote/mac.sh [--no-sync] [--pack] [--pull <path>]... -- <command>" >&2; exit 2; }
CMD="$*"

# -n: never let ssh read (and drain) this script's stdin.
SSH=(ssh -n -o BatchMode=yes -o ConnectTimeout=15 -o ServerAliveInterval=30 "$HOST")

# The SSD must be mounted, or everything would silently land on the small internal disk.
# The Mac is shared with graphide's agents: respect their "keep off Nix" markers.
if "${SSH[@]}" 'test -e /Users/Shared/graphide-nix-maintenance -o -e /Users/Shared/graphide-nix-unhealthy'; then
    echo "mac.sh: the Mac's Nix is under maintenance or unhealthy (/Users/Shared/graphide-nix-*); not running" >&2
    exit 75
fi
"${SSH[@]}" "test -d /Volumes/Graphide/ban && mkdir -p '$REMOTE' '$CACHE/gradle' '$CACHE/cargo' '$CACHE/xwin' '$CACHE/tmp' '$SSD/mic/pack'" \
    || { echo "mac.sh: /Volumes/Graphide is not mounted on $HOST" >&2; exit 1; }

EXCLUDES=(--exclude=/target/ --exclude=/smoke/ --exclude=/run/ --exclude=/run-gametest/ --exclude=/run-planet/
          --exclude=/libs/ --exclude=/review/out/ --exclude=/neoforge/build/ --exclude=/neoforge/.gradle/
          --exclude=.gradle/ --exclude=/.graphide/ --exclude=/.claude/ --exclude=/.git)
# The Mac's /usr/bin/git is Apple's stub (no Xcode command-line tools), so a nixpkgs git is kept,
# GC-rooted, on the SSD. It matters: without a git index Nix would treat the synced tree as a path
# flake and copy all of it, build outputs included, into the store.
GIT="$CACHE/git/bin/git"
"${SSH[@]}" "exec zsh -lc $(printf '%q' "[ -x '$GIT' ] || nix build nixpkgs#git --out-link '$CACHE/git'")"
if [ "$SYNC" = 1 ]; then
    rsync -a --delete "${EXCLUDES[@]}" -e "ssh -o BatchMode=yes" "$REPO/" "$HOST:$REMOTE/"
    "${SSH[@]}" "cd '$REMOTE' && { [ -d .git ] || '$GIT' init -q -b master; } && '$GIT' add -A"
fi
if [ "$PACK" = 1 ]; then
    # The pack sits two levels above the main checkout; a worktree elsewhere uses the known path.
    PACKDIR=/home/greencheetah/Projects/mic/pack
    [ -f "$REPO/../../pack/pack.toml" ] && PACKDIR="$(cd "$REPO/../../pack" && pwd)"
    rsync -a --delete -e "ssh -o BatchMode=yes" "$PACKDIR/" "$HOST:$SSD/mic/pack/"
fi

# The command runs inside the repo's dev shell, entered through a GC-rooted profile keyed on
# flake.nix + flake.lock. Evaluating the flake afresh on every call would copy the synced tree
# into /nix/store each time (that pattern filled the Mac's SSD twice in September 2026); the
# repo's scripts see IN_NIX_SHELL and skip their own `nix develop`.
if [ "$SERVER" = 1 ]; then
    # mkdir is atomic: the lock is a directory holding who took it and when. A lock older than
    # three hours is taken to be stale (its holder died without releasing it).
    LOCK="$CACHE/server.lock"
    CMD="while ! mkdir '$LOCK' 2>/dev/null; do
           age=\$(( \$(date +%s) - \$(stat -f %m '$LOCK' 2>/dev/null || date +%s) ))
           if [ \$age -gt 10800 ]; then echo 'mac.sh: breaking a stale server lock'; rm -rf '$LOCK'; continue; fi
           echo \"mac.sh: waiting for the server lock (held by \$(cat '$LOCK/owner' 2>/dev/null))\"; sleep 30
         done
         echo '$NAME' > '$LOCK/owner'; trap \"rm -rf '$LOCK'\" EXIT INT TERM
         $CMD"
fi
KEY="$(cat "$REPO/flake.nix" "$REPO/flake.lock" | sha256sum | cut -c1-16)"
PROFILE="$CACHE/devshell-$KEY"
INNER="cd '$REMOTE' && export GRADLE_USER_HOME='$CACHE/gradle' CARGO_HOME='$CACHE/cargo' XWIN_CACHE_DIR='$CACHE/xwin' TMPDIR='$CACHE/tmp' DEEPTIME_REMOTE=mac && { [ -e '$PROFILE' ] || nix develop --profile '$PROFILE' -c true; } && exec nix develop '$PROFILE' -c zsh -c $(printf '%q' "$CMD")"
set +e
"${SSH[@]}" "exec zsh -lc $(printf '%q' "$INNER")"
status=$?
set -e

for p in "${PULLS[@]}"; do
    mkdir -p "$REPO/$(dirname "$p")"
    rsync -a -e "ssh -o BatchMode=yes" "$HOST:$REMOTE/$p" "$REPO/$(dirname "$p")/" || echo "mac.sh: could not pull $p" >&2
done
exit "$status"

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
#   --pull <path>  after the command, copy <path> (relative to the repo root) back from the Mac;
#                  repeatable (e.g. --pull smoke/mic/report.md --pull neoforge/build/gametest.log)
#   <command>      run from the repo root on the Mac inside a login shell, e.g.
#                  'nix develop -c cargo test --workspace'  or  './gametest.sh'
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

HOST="${DEEPTIME_MAC:-ban@mac-mini}"
SSD=/Volumes/Graphide/ban
REPO="$(git rev-parse --show-toplevel)"
NAME="$(basename "$REPO")"
REMOTE="$SSD/mic/mods/$NAME"
CACHE="$SSD/deeptime-cache"

SYNC=1; PACK=0; PULLS=()
while [ $# -gt 0 ]; do
    case "$1" in
        --no-sync) SYNC=0; shift ;;
        --pack) PACK=1; shift ;;
        --pull) PULLS+=("$2"); shift 2 ;;
        --) shift; break ;;
        *) break ;;
    esac
done
[ $# -gt 0 ] || { echo "usage: tools/remote/mac.sh [--no-sync] [--pack] [--pull <path>]... -- <command>" >&2; exit 2; }
CMD="$*"

SSH=(ssh -o BatchMode=yes -o ConnectTimeout=15 -o ServerAliveInterval=30 "$HOST")

# The SSD must be mounted, or everything would silently land on the small internal disk.
"${SSH[@]}" "test -d /Volumes/Graphide/ban && mkdir -p '$REMOTE' '$CACHE/gradle' '$CACHE/cargo' '$CACHE/xwin' '$CACHE/tmp' '$SSD/mic/pack'" \
    || { echo "mac.sh: /Volumes/Graphide is not mounted on $HOST" >&2; exit 1; }

EXCLUDES=(--exclude=/target/ --exclude=/smoke/ --exclude=/run/ --exclude=/run-gametest/ --exclude=/run-planet/
          --exclude=/libs/ --exclude=/review/out/ --exclude=/neoforge/build/ --exclude=/neoforge/.gradle/
          --exclude=.gradle/ --exclude=/.graphide/ --exclude=/.claude/ --exclude=/.git)
if [ "$SYNC" = 1 ]; then
    rsync -a --delete "${EXCLUDES[@]}" -e "ssh -o BatchMode=yes" "$REPO/" "$HOST:$REMOTE/"
    "${SSH[@]}" "cd '$REMOTE' && { [ -d .git ] || git init -q -b master; } && git add -A >/dev/null 2>&1; true"
fi
if [ "$PACK" = 1 ]; then
    # The pack sits two levels above the main checkout; a worktree elsewhere uses the known path.
    PACKDIR=/home/greencheetah/Projects/mic/pack
    [ -f "$REPO/../../pack/pack.toml" ] && PACKDIR="$(cd "$REPO/../../pack" && pwd)"
    rsync -a --delete -e "ssh -o BatchMode=yes" "$PACKDIR/" "$HOST:$SSD/mic/pack/"
fi

set +e
"${SSH[@]}" "cd '$REMOTE' && export GRADLE_USER_HOME='$CACHE/gradle' CARGO_HOME='$CACHE/cargo' XWIN_CACHE_DIR='$CACHE/xwin' TMPDIR='$CACHE/tmp' DEEPTIME_REMOTE=mac && exec zsh -lc $(printf '%q' "$CMD")"
status=$?
set -e

for p in "${PULLS[@]}"; do
    mkdir -p "$REPO/$(dirname "$p")"
    rsync -a -e "ssh -o BatchMode=yes" "$HOST:$REMOTE/$p" "$REPO/$(dirname "$p")/" || echo "mac.sh: could not pull $p" >&2
done
exit "$status"

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Run a command on the test VM (deeptime-test; "the Mac" in older notes) instead of the laptop (Ben's
# standing rule: cargo, Gradle, GameTests, smoke/planet servers and BlueMap run there; the laptop only
# edits and runs git).
#
#   tools/remote/mac.sh [--no-sync] [--pack] [--server] [--kind K] [--pull <path>]... -- <command>
#
#   --no-sync      skip the rsync of this checkout (the Mac copy is already current)
#   --pack         also sync the MIC pack (…/mic/pack, with its .git) for tools/smoke/install.sh mic,
#                  and the MIC rock-tag datapack (…/mic/Docs/datapacks/deeptime-mic-rock-tags) that
#                  ./gametest.sh -PwithMic adds
#   --server       the command boots Minecraft: its job gets a server slot (its own ports and folder;
#                  tools/remote/vm/admit.py). The repo's server launchers (gametest.sh,
#                  gametest-planet.sh, smoke run.py, preview.py, dh-client-check.sh) take a slot
#                  themselves, so --server is only needed for other scripts that boot a server
#   --kind K       the job's kind for the admission tool (default: classified from the command, e.g.
#                  cargo:test, gradle:build, gametest:mic, sim:<preset>:<threads>t; `admit.py classify`)
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
#
# Every command runs through the VM's admission tool (/usr/local/lib/deeptime/admit.py; Ben,
# 2026-10-03: "Guess load, run if load is less than what is available + 25%"): it starts when its
# kind's estimated load fits (CPU: 1.25 x the free cores; memory: estimate x 1.25 under the free memory), waits (saying why) otherwise, and
# its measured load refines the next estimate. Quick looks (cat, git, ls ...) and detached starts run
# at once. `tools/remote/mac.sh --no-sync -- deeptime-admit status` shows what runs and waits.

set -euo pipefail

HOST="${DEEPTIME_MAC:-ban@20.120.160.3}"
SSD=/Volumes/Graphide/ban
REPO="$(git rev-parse --show-toplevel)"
NAME="$(basename "$REPO")"
REMOTE="$SSD/mic/mods/$NAME"
CACHE="$SSD/deeptime-cache"

SYNC=1; PACK=0; SERVER=0; KIND=auto; PULLS=()
while [ $# -gt 0 ]; do
    case "$1" in
        --no-sync) SYNC=0; shift ;;
        --pack) PACK=1; shift ;;
        --server) SERVER=1; shift ;;
        --kind) KIND="$2"; shift 2 ;;
        --pull) PULLS+=("$2"); shift 2 ;;
        --) shift; break ;;
        *) break ;;
    esac
done
[ $# -gt 0 ] || { echo "usage: tools/remote/mac.sh [--no-sync] [--pack] [--server] [--kind K] [--pull <path>]... -- <command>" >&2; exit 2; }
CMD="$*"

# -n: never let ssh read (and drain) this script's stdin.
SSH=(ssh -n -o BatchMode=yes -o ConnectTimeout=15 -o ServerAliveInterval=30 "$HOST")
# The VM's firewall follows the laptop's address (vm-access.sh): repair it before the first call.
. "$(dirname "${BASH_SOURCE[0]}")/vm-access.sh"
vm_access_ensure "$HOST" || { echo "$(basename "$0"): cannot reach $HOST" >&2; exit 1; }

# The SSD must be mounted, or everything would silently land on the small internal disk.
# The Mac is shared with graphide's agents: respect their "keep off Nix" markers.
if "${SSH[@]}" 'test -e /Users/Shared/graphide-nix-maintenance -o -e /Users/Shared/graphide-nix-unhealthy'; then
    echo "mac.sh: the Mac's Nix is under maintenance or unhealthy (/Users/Shared/graphide-nix-*); not running" >&2
    exit 75
fi
# Ben paused Deep Time jobs on the M6 on 2026-10-01: our load made graphide's timing-sensitive tests
# flaky. The marker lives on the machine, so pointing DEEPTIME_MAC at the new test box is unaffected.
if "${SSH[@]}" "test -e '$CACHE/PAUSED'"; then
    echo "mac.sh: Deep Time jobs on $HOST are paused ($CACHE/PAUSED): $("${SSH[@]}" "cat '$CACHE/PAUSED'")" >&2
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
    # --checksum --no-times: copy a file only when its bytes differ, and give each copied file the
    # Mac's current time. With plain -a the laptop's mtimes came across, so a file whose content went
    # back to an older version (a merge, a branch switch) could be older than the build artifact made
    # from the newer version, and Cargo kept the stale artifact (2026-09-30: release deeptime-cli
    # failed with "no field min_y" while the sources were correct). Unchanged files keep their time.
    rsync -rlpD --checksum --no-times --delete "${EXCLUDES[@]}" -e "ssh -o BatchMode=yes" "$REPO/" "$HOST:$REMOTE/"
    "${SSH[@]}" "cd '$REMOTE' && { [ -d .git ] || '$GIT' init -q -b master; } && '$GIT' add -A"
    # Which laptop revision this copy is (the Mac's .git has no commits, and Apple's git there is a
    # stub): tools/smoke/jarstate.py reads it into smoke/<profile>/install.json. Written after
    # `git add -A`, so it stays untracked and invisible to Nix flakes; rsync --delete removes it on
    # the next sync before it is written again.
    SYNC_REV="$(printf '{"describe": "%s", "commit": "%s", "branch": "%s", "synced": "%s"}' \
        "$(git -C "$REPO" describe --always --dirty --abbrev=8 2>/dev/null | tr -cd 'A-Za-z0-9._/@+-')" \
        "$(git -C "$REPO" rev-parse HEAD 2>/dev/null | tr -cd '0-9a-f')" \
        "$(git -C "$REPO" rev-parse --abbrev-ref HEAD 2>/dev/null | tr -cd 'A-Za-z0-9._/@+-')" \
        "$(date -u +%Y-%m-%dT%H:%M:%SZ)")"
    "${SSH[@]}" "printf '%s\n' '$SYNC_REV' > '$REMOTE/.sync-rev'"
fi
if [ "$PACK" = 1 ]; then
    # The pack sits two levels above the main checkout; a worktree elsewhere uses the known path.
    PACKDIR=/home/greencheetah/Projects/mic/pack
    [ -f "$REPO/../../pack/pack.toml" ] && PACKDIR="$(cd "$REPO/../../pack" && pwd)"
    rsync -a --delete -e "ssh -o BatchMode=yes" "$PACKDIR/" "$HOST:$SSD/mic/pack/"
    # The MIC rock-tag datapack (read-only here), which -PwithMic adds to the dev servers through
    # -Ddeeptime.extraDatapack (neoforge/build.gradle, P1-12). It lives next to the pack in mic/Docs.
    TAGSDIR="$(dirname "$PACKDIR")/Docs/datapacks/deeptime-mic-rock-tags"
    if [ -f "$TAGSDIR/pack.mcmeta" ]; then
        "${SSH[@]}" "mkdir -p '$SSD/mic/Docs/datapacks'"
        rsync -a --delete -e "ssh -o BatchMode=yes" "$TAGSDIR/" "$HOST:$SSD/mic/Docs/datapacks/deeptime-mic-rock-tags/"
    fi
fi

# The command runs inside the repo's dev shell, entered through a GC-rooted profile keyed on
# flake.nix + flake.lock. Evaluating the flake afresh on every call would copy the synced tree
# into /nix/store each time (that pattern filled the Mac's SSD twice in September 2026); the
# repo's scripts see IN_NIX_SHELL and skip their own `nix develop`.
KEY="$(cat "$REPO/flake.nix" "$REPO/flake.lock" | sha256sum | cut -c1-16)"
PROFILE="$CACHE/devshell-$KEY"
# Every worktree has its own copy and target/ on the SSD, so target size is multiplied by the number
# of agents (2026-09-30: ~60 copies filled the SSD). Incremental caches were half of a debug target
# and full debuginfo most of the rest; without them a copy's target is a fraction of the size, at the
# cost of recompiling the workspace crates (not the dependencies) on each change. Line tables keep
# file:line in panics and backtraces.
# No fixed job sizes (the M6-era CARGO_BUILD_JOBS=8 and RAYON_NUM_THREADS=6 are gone): the admission
# tool starts a job when its measured load fits. A command may still set its own values.
# The admission tool runs outside the dev shell (its LD_LIBRARY_PATH breaks system tools); the dev
# shell is entered inside the admitted job.
ADMIT=/usr/local/lib/deeptime/admit.py
DEV="nix develop '$PROFILE' -c zsh -c $(printf '%q' "$CMD")"
ADMIT_RUN="/usr/bin/python3 '$ADMIT' run --kind $(printf '%q' "$KIND") --label '$NAME'$([ "$SERVER" = 1 ] && echo ' --server' || true) -- $DEV"
INNER="cd '$REMOTE' && export GRADLE_USER_HOME='$CACHE/gradle' CARGO_HOME='$CACHE/cargo' XWIN_CACHE_DIR='$CACHE/xwin' TMPDIR='$CACHE/tmp' DEEPTIME_REMOTE=mac CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only && { [ -e '$PROFILE' ] || nix develop --profile '$PROFILE' -c true; } && if [ -f '$ADMIT' ]; then exec $ADMIT_RUN; else exec $DEV; fi"
set +e
"${SSH[@]}" "exec zsh -lc $(printf '%q' "$INNER")"
status=$?
set -e

for p in "${PULLS[@]}"; do
    mkdir -p "$REPO/$(dirname "$p")"
    rsync -a -e "ssh -o BatchMode=yes" "$HOST:$REMOTE/$p" "$REPO/$(dirname "$p")/" || echo "mac.sh: could not pull $p" >&2
done
exit "$status"

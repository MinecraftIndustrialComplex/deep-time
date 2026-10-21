#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Free the Mac's SSD after merges (CLAUDE.md "Mac disk hygiene"). Two jobs:
#   - delete the Mac copy (/Volumes/Graphide/ban/mic/mods/<checkout name>) of every worktree of this
#     repo whose branch is merged into master;
#   - drop target/*/incremental from the copies that stay (mac.sh builds with CARGO_INCREMENTAL=0,
#     so those caches are dead weight left from older builds);
#   - drop the whole target/ of a copy that stays but was not synced for 7 days and is not in use
#     (an unmerged or abandoned branch, or a copy whose worktree is gone; Ben 2026-10-21: "get a
#     decent garbage collection system up and running"). Only build outputs go; mac.sh rebuilds.
#
#   tools/remote/mac-gc.sh [--dry-run]
#
# A copy is kept when any of these holds: it is the main checkout's; its branch is not merged into
# master, or its worktree is detached; its worktree has uncommitted changes (an agent is mid-task);
# it was synced in the last hour; a Mac process has it in its command line; or it belongs to no
# worktree or branch of this repo (another repo's checkout, such as mic-climate). A copy is derived
# from a laptop worktree, so deleting one loses no work, only build outputs; mac.sh recreates it.
#
# The remote side runs as `bash -s` scripts: the Mac's login shell is zsh, which does not split an
# unquoted $var into words, so an inline `for d in $LIST` there loops once over the whole list.

set -euo pipefail

HOST="${DEEPTIME_MAC:-ban@deeptime-test-ban.westus2.cloudapp.azure.com}"
MODS=/Volumes/Graphide/ban/mic/mods
DRY=0; [ "${1:-}" = --dry-run ] && DRY=1
IDLE_TARGET=$(( 7 * 86400 ))  # a kept copy idle this long loses its target/
SSH=(ssh -o BatchMode=yes -o ConnectTimeout=15 "$HOST")
# The VM's firewall follows the laptop's address (vm-access.sh): repair it before the first call.
. "$(dirname "${BASH_SOURCE[0]}")/vm-access.sh"
vm_access_ensure "$HOST" || { echo "$(basename "$0"): cannot reach $HOST" >&2; exit 1; }

MAIN_NAME="$(basename "$(dirname "$(cd "$(git rev-parse --git-common-dir)" && pwd)")")"

declare -A WT BR
path=""
while read -r key val; do
    case "$key" in
        worktree) path="$val"; WT["$(basename "$path")"]="$path" ;;
        branch) BR["$(basename "$path")"]="${val#refs/heads/}" ;;
    esac
done < <(git worktree list --porcelain)

listing="$("${SSH[@]}" bash -s -- "$MODS" <<'EOF'
cd "$1" || exit 1
now=$(date +%s)
for d in */; do
    d=${d%/}
    t=$(stat -c %Y "$d/.sync-rev" 2>/dev/null || stat -c %Y "$d" 2>/dev/null || /usr/bin/stat -f %m "$d/.sync-rev" 2>/dev/null || /usr/bin/stat -f %m "$d")
    echo "copy $d $(( now - t ))"
done
ps -axo command | sed 's/^/proc /'
EOF
)"
procs="$(sed -n 's/^proc //p' <<<"$listing")"

OPS=()
while read -r _ name age; do
    wt="${WT[$name]:-}"
    b="${BR[$name]:-}"
    [ -z "$b" ] && [ -z "$wt" ] && git show-ref -q --verify "refs/heads/worktree-$name" && b="worktree-$name"
    busy=0; grep -Eq "$MODS/$name(/|[[:space:]]|\$)" <<<"$procs" && busy=1
    why=""
    if [ "$name" = "$MAIN_NAME" ]; then why="main checkout"
    elif [ -z "$b" ] && [ -z "$wt" ]; then
        if [ "$age" -ge "$IDLE_TARGET" ] && [ "$busy" = 0 ]; then
            echo "keep $name: not a worktree or branch of this repo; its target/ goes (idle $(( age / 86400 )) d)"
            OPS+=("target:$name")
        else
            echo "skip $name: not a worktree or branch of this repo"
        fi
        continue
    elif [ -z "$b" ]; then why="detached worktree"
    elif ! git merge-base --is-ancestor "$b" master; then why="$b not merged"
    elif [ -n "$wt" ] && [ -n "$(git -C "$wt" status --porcelain 2>/dev/null)" ]; then why="uncommitted changes in $wt"
    elif [ "$age" -lt 3600 ]; then why="synced $(( age / 60 )) min ago"
    elif [ "$busy" = 1 ]; then why="in use on the Mac"
    fi
    if [ -z "$why" ]; then
        OPS+=("del:$name")
    else
        if [ "$busy" = 0 ] && [ "$name" != "$MAIN_NAME" ] && [ "$age" -ge "$IDLE_TARGET" ]; then
            echo "keep $name: $why; its target/ goes (idle $(( age / 86400 )) d)"
            OPS+=("target:$name")
        else
            echo "keep $name: $why"
            [ "$busy" = 0 ] && OPS+=("prune:$name")
        fi
    fi
done < <(grep '^copy ' <<<"$listing")

[ "${#OPS[@]}" -gt 0 ] || { echo "nothing to do"; exit 0; }

# Deletion re-checks the process list on the Mac, right before each rm.
"${SSH[@]}" bash -s -- "$MODS" "$DRY" "${OPS[@]}" <<'EOF'
mods=$1; dry=$2; shift 2
cd "$mods" || exit 1
verb=deleted; [ "$dry" = 1 ] && verb="would delete"
procs=$(ps -axo command)
for a in "$@"; do
    op=${a%%:*}; d=${a#*:}
    if printf '%s\n' "$procs" | grep -Eq "$mods/$d(/|[[:space:]]|\$)"; then echo "keep $d: in use on the Mac"; continue; fi
    if [ "$op" = del ]; then
        echo "$verb $d ($(du -sh "$d" | cut -f1))"
        [ "$dry" = 1 ] || rm -rf "$d"
    elif [ "$op" = target ]; then
        [ -d "$d/target" ] || continue
        echo "$verb $d/target ($(du -sh "$d/target" | cut -f1))"
        [ "$dry" = 1 ] || rm -rf "$d/target"
    else
        for i in "$d"/target/*/incremental; do
            [ -d "$i" ] || continue
            echo "$verb $i ($(du -sh "$i" | cut -f1))"
            [ "$dry" = 1 ] || rm -rf "$i"
        done
    fi
done
df -h "$mods" | tail -1
EOF

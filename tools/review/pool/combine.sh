#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Test two or more unmerged branches together in one set of runs (Ben, 2026-10-22: "If both deltas and
# continent spread need to do a world gen to test they should just merge their test."):
#
#   tools/review/pool/combine.sh [--no-build] <name> <branch> [<branch> ...]
#
# Makes a scratch worktree of origin/master under build/, merges each branch into it in turn
# (`git merge --no-commit --no-ff`, committed on the throwaway branch pool/<name>), builds the CLI on the
# VM through the pool, and prints the combined tree's source key. Run it with
# `tools/review/pool/pool.sh run <arm> --key <key> ...`. A merge conflict stops the whole thing and says
# which files: this script never resolves one (the lanes' owners do, or the lanes run apart). Nothing is
# pushed and master is untouched; the scratch worktree and its VM copy are removed after the build, the
# branch stays until `tools/review/pool/pool.sh forget <name>`. With --no-build it only merges, reports
# conflicts and prints the key the combination would have (nothing is built; the branch is removed too).
set -euo pipefail
REPO=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)
cd "$REPO"

die() { echo "combine.sh: $*" >&2; exit 2; }
no_build=""
if [ "${1:-}" = "--no-build" ]; then no_build=1; shift; fi
[ $# -ge 2 ] || die "usage: combine.sh [--no-build] <name> <branch> [<branch> ...]"
name=$1; shift
[[ "$name" =~ ^[A-Za-z0-9_.-]+$ ]] || die "the name '$name' may hold letters, digits, _ . - only"
branches=("$@")
git rev-parse --verify -q "refs/heads/pool/$name" > /dev/null && die "the branch pool/$name exists: tools/review/pool/pool.sh forget $name first"
for b in "${branches[@]}"; do
    git rev-parse --verify -q "$b^{commit}" > /dev/null || die "no such branch or commit: $b"
done

if [ -z "${POOL_NO_FETCH:-}" ]; then
    timeout 60 git fetch -q origin master || echo "combine.sh: could not fetch origin/master; using the one this clone has" >&2
fi
base=origin/master
git rev-parse --verify -q "$base^{commit}" > /dev/null || die "no $base in this clone"

scratch="$REPO/build/pool-$name"
[ ! -e "$scratch" ] || die "$scratch exists (an earlier combine?): remove it with git worktree remove --force"
mkdir -p "$REPO/build"
git worktree add -q --detach "$scratch" "$base"
cd "$scratch"
git switch -q -c "pool/$name"

abort() {
    cd "$REPO"
    git -C "$scratch" merge --abort 2> /dev/null || true
    git worktree remove --force "$scratch" 2> /dev/null || true
    git branch -D "pool/$name" > /dev/null 2>&1 || true
}

for b in "${branches[@]}"; do
    if git merge-base --is-ancestor "$b" HEAD; then
        echo "combine.sh: $b is already in $base (or an earlier branch); nothing to merge"
        continue
    fi
    if ! merge_out=$(git merge --no-commit --no-ff -q "$b" 2>&1); then
        echo "combine.sh: CONFLICT merging $b onto $base${branches[1]:+ and the branches before it}. Nothing resolved." >&2
        git diff --name-only --diff-filter=U | sed 's/^/  conflict: /' >&2
        echo "$merge_out" | tail -n 5 >&2
        abort
        exit 3
    fi
    git commit -q --no-verify -m "pool/$name: merge $b into $base"
    echo "combine.sh: merged $b ($(git rev-parse --short=10 "$b"))"
done

# The scratch tree may predate the pool (master without this folder): the pool's own scripts come from
# the checkout that was asked, copied in as untracked files, which the source key does not cover.
mkdir -p tools/review
rm -rf tools/review/pool
cp -r "$REPO/tools/review/pool" tools/review/pool
rm -rf tools/review/pool/__pycache__
joined=$(IFS=+; echo "${branches[*]}")
key=$(python3 -I tools/review/pool/srckey.py key --ref HEAD) || { echo "combine.sh: the combined tree has no usable source key; the scratch worktree stays at $scratch" >&2; exit 3; }
echo "combine.sh: source key $key for pool/$name = $base + ${branches[*]}"
if [ -n "$no_build" ]; then
    cd "$REPO"
    git worktree remove --force "$scratch"
    git branch -D "pool/$name" > /dev/null 2>&1 || true
    echo "combine.sh: --no-build: merged without conflicts; nothing built, scratch worktree and branch removed"
    exit 0
fi
if ! tools/review/pool/pool.sh build --key "$key" --note "combine:$name=$joined"; then
    echo "combine.sh: the build failed; the scratch worktree stays at $scratch and its VM copy at /data/ban/mic/mods/pool-$name" >&2
    exit 1
fi

# The binary is in the pool now: the scratch tree and its VM copy have done their job.
tools/remote/mac.sh --no-sync --kind light -- "cd .. && rm -rf pool-$name" > /dev/null 2>&1 || echo "combine.sh: could not remove the VM copy pool-$name (tools/remote/mac-gc.sh will not either: its branch is not merged)" >&2
cd "$REPO"
git worktree remove --force "$scratch"
cat <<EOF
combine.sh: done. Source key: $key  (branch pool/$name, $base + ${branches[*]})
  run it:   tools/review/pool/pool.sh run <arm> --key $key --preset earthlike_quick_16k --length 1000 --starts supercontinent,scattered_continents --seeds 1,2,3
  forget:   tools/review/pool/pool.sh forget $name   (the pool keeps the runs and the binary until pool.sh gc)
EOF

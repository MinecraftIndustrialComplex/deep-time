#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The coordinator's merge of a finished agent branch into master (run on master, in the main checkout):
#
#   tools/dev/merge-branch.sh <branch> ["one-line summary for the merge message"]
#
# `git merge --no-ff`, then the conflicts every batch has are settled the same way each time:
#   STATUS.md, docs/deeptime-design-decisions.md, docs/science/bibliography.md
#       both sides append rows and entries: a union of the hunks (git merge-file --union)
#   docs/science/claims.md
#       its table is counts over the ledgers: master's side, then tools/review/claims_counts.py --write
# Any other conflicted file stops the script with the merge left in progress (exit 3) for a person
# or agent to resolve; nothing is committed then.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
b="${1:?usage: merge-branch.sh <branch> [summary]}"
summary="${2:-}"
[ "$(git branch --show-current)" = master ] || { echo "merge-branch: not on master" >&2; exit 2; }
git merge --no-ff --no-commit "$b" >/dev/null 2>&1
union() {
    local f="$1" tmp; tmp="$(mktemp -d)"
    git show ":1:$f" > "$tmp/base" 2>/dev/null || : > "$tmp/base"
    git show ":2:$f" > "$tmp/ours"; git show ":3:$f" > "$tmp/theirs"
    git merge-file --union -p "$tmp/ours" "$tmp/base" "$tmp/theirs" > "$f"
    rm -rf "$tmp"; git add "$f"
}
for f in $(git diff --name-only --diff-filter=U); do
    case "$f" in
        STATUS.md|docs/deeptime-design-decisions.md|docs/science/bibliography.md) union "$f"; echo "merge-branch: $f: union" ;;
        docs/science/claims.md) git checkout --ours -- "$f" && git add "$f"; echo "merge-branch: $f: master's, recounted below" ;;
    esac
done
left="$(git diff --name-only --diff-filter=U)"
if [ -n "$left" ]; then
    echo "merge-branch: unresolved, merge left in progress:" >&2; echo "$left" >&2; exit 3
fi
if git diff --cached --name-only | grep -q '^docs/science/'; then
    python3 tools/review/claims_counts.py --write >/dev/null && git add docs/science/claims.md
fi
if git diff --cached --quiet && ! git rev-parse -q --verify MERGE_HEAD >/dev/null; then
    echo "merge-branch: $b has nothing to merge"; exit 0
fi
git commit -q -m "Merge $b${summary:+ ($summary)}

Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>" && git log --oneline -1

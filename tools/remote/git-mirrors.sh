#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Serves the workspace's PRIVATE git dependencies to the test VM's cargo (sourced by mac.sh;
# decisions log 2026-10-21, "vendor mic-viewer or depend on it?"). TODAY THERE IS NONE: mic-viewer,
# the one dependency this was written for, became public on 2026-10-21 (decisions log) and Cargo.toml
# fetches it over https, which the VM, the laptop and CI do with no credentials. The script finds no
# pins (it matches only ssh://git@github.com URLs), so a sync does nothing here. It stays for the next
# private git dependency: give that one an ssh://git@github.com/<org>/<repo>.git URL and a full
# commit hash as `rev`, and every mac.sh sync will mirror it to the VM as described below.
#
# The VM has no GitHub credentials, and none are given to it: no deploy key, no forwarded agent (the
# VM is shared by every agent, and a forwarded agent would expose Ben's GitHub key). Instead:
#  - The VM keeps one bare mirror per private repository,
#    <cache>/git-mirrors/<org>/<repo>.git (e.g. .../MinecraftIndustrialComplex/mic-viewer.git).
#  - Its git maps the GitHub URL to that mirror: `url.<mirror>.insteadOf <github url>` in ban's
#    ~/.gitconfig, so every git on the VM sees it, whatever started the cargo that runs it (mac.sh,
#    detach.sh jobs, gate-batch.sh over plain ssh, the workbench's preview.py, Gradle's cargo tasks,
#    tools/natives). Cargo fetches git dependencies with the git command
#    (.cargo/config.toml `net.git-fetch-with-cli`), which follows the rewrite.
#  - Cargo.lock keeps the GitHub URL, so it is the same file as on the laptop and `--locked` holds
#    (a `[patch]` to a path would rewrite it).
#  - The laptop fills the mirrors. On every sync mac.sh asks the VM, inside the ssh call it makes
#    anyway, which pinned commits its mirrors lack; when one does, the laptop fetches that commit from
#    GitHub into its own cache (${XDG_CACHE_HOME:-~/.cache}/deeptime/git-mirrors/<org>/<repo>.git)
#    and pushes it to the VM's mirror as refs/pins/<commit>. An up-to-date mirror costs no extra
#    connection. Every checkout (the main one, agent worktrees) reads its own Cargo.lock and
#    Cargo.toml files, so a branch that bumps the pin gets its commit pushed on its first sync.
#
#   git_mirror_pins <repo dir>                 "<url> <commit>" per pinned private commit: Cargo.lock's
#                                              ssh://git@github.com sources and the manifests'
#                                              `git = "ssh://git@github.com/...", rev = "<40 hex>"`
#                                              (so a bumped rev reaches the VM before its lock does)
#   git_mirror_vm_script <vm cache> <vm git> <pins>
#                                              shell for the VM: makes the mirrors and the insteadOf
#                                              entries, prints "missing <url> <commit>" per absent commit
#   git_mirror_push <user@host> <vm cache> <vm git> <missing lines>
#                                              fetches those commits on the laptop and pushes them
#
# By hand, e.g. after a VM rebuild with nothing to sync: any `tools/remote/mac.sh -- true`.

GIT_MIRROR_URL_RE='ssh://git@github\.com/[A-Za-z0-9._/-]*'

git_mirror_pins() {
    local repo="$1"
    # A missing file is no pin (callers run under `set -euo pipefail`).
    {
        sed -n "s|^source = \"git+\\($GIT_MIRROR_URL_RE\\)[^\"#]*#\\([0-9a-f]\\{40\\}\\)\"\$|\\1 \\2|p" \
            "$repo/Cargo.lock" 2>/dev/null || true
        { cat "$repo/Cargo.toml" "$repo"/crates/*/Cargo.toml 2>/dev/null || true; } \
            | sed -n "s|.*git *= *\"\\($GIT_MIRROR_URL_RE\\)\".*rev *= *\"\\([0-9a-f]\\{40\\}\\)\".*|\\1 \\2|p"
    } | sort -u
}

git_mirror_vm_script() {
    local cache="$1" vmgit="$2" pins="$3" url commit
    [ -n "$pins" ] || { echo true; return; }
    # mv -T: of two syncs making the same mirror at once, one wins and the other drops its copy.
    # allowAnySHA1InWant: cargo fetches the pinned commit by its hash.
    cat <<EOF
gm() {
  m="\$1"; u="\$2"; c="\$3"; G='$vmgit'
  if [ ! -d "\$m" ]; then
    mkdir -p "\${m%/*}" && t="\$m.new.\$\$" && "\$G" init -q --bare "\$t" \\
      && "\$G" -C "\$t" config uploadpack.allowAnySHA1InWant true \\
      && { mv -T "\$t" "\$m" 2>/dev/null || rm -rf "\$t"; }
  fi
  "\$G" config --global --get-all "url.\$m.insteadOf" 2>/dev/null | grep -qxF "\$u" \\
    || "\$G" config --global --add "url.\$m.insteadOf" "\$u" \\
    || echo "git-mirrors: could not set url.\$m.insteadOf in ~/.gitconfig" >&2
  "\$G" -C "\$m" cat-file -e "\$c^{commit}" 2>/dev/null || echo "missing \$u \$c"
}
EOF
    while read -r url commit; do
        [ -n "$commit" ] || continue
        printf "gm '%s' '%s' '%s'\n" "$cache/git-mirrors/${url#ssh://git@github.com/}" "$url" "$commit"
    done <<< "$pins"
    echo true
}

git_mirror_push() {
    local host="$1" cache="$2" vmgit="$3" lines="$4" word url commit path mine rc=0
    local base="${XDG_CACHE_HOME:-$HOME/.cache}/deeptime/git-mirrors"
    while read -r word url commit; do
        [ "$word" = missing ] && [ -n "$commit" ] || continue
        path="${url#ssh://git@github.com/}"
        mine="$base/$path"
        mkdir -p "$(dirname "$mine")"
        # Several agents' syncs share this cache: one fetch into it at a time.
        if ! (
            command -v flock >/dev/null && flock 9
            { [ -d "$mine" ] || git init -q --bare "$mine"; } || exit 1
            git -C "$mine" cat-file -e "$commit^{commit}" 2>/dev/null \
                || git -C "$mine" fetch -q --no-tags "$url" "+$commit:refs/pins/$commit"
        ) 9>"$mine.lock"; then
            echo "git-mirrors: could not fetch $commit from $url on the laptop; cargo on the VM will not find it" >&2
            rc=1; continue
        fi
        # A failed push may be another sync pushing the same ref at the same moment: look again.
        if GIT_SSH_COMMAND="ssh -o BatchMode=yes -o ConnectTimeout=15" git -C "$mine" push -q \
                --receive-pack="$vmgit receive-pack" "$host:$cache/git-mirrors/$path" \
                "$commit:refs/pins/$commit" 2>/dev/null \
            || ssh -n -o BatchMode=yes -o ConnectTimeout=15 "$host" \
                "'$vmgit' -C '$cache/git-mirrors/$path' cat-file -e '$commit^{commit}'"; then
            echo "git-mirrors: the VM's mirror has ${path%.git} $commit"
        else
            echo "git-mirrors: could not push $commit to $host:$cache/git-mirrors/$path; cargo on the VM will not find it" >&2
            rc=1
        fi
    done <<< "$lines"
    return "$rc"
}

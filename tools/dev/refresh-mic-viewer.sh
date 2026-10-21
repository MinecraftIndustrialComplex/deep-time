#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Refreshes vendor/mic-viewer, the shared viewer core (MinecraftIndustrialComplex/mic-viewer,
# MPL-2.0; decisions log 2026-10-21, VW3 and BR-VWD), from a checkout of that repository.
#
#   tools/dev/refresh-mic-viewer.sh [REV]     REV defaults to origin/main
#
# Why a copy and not a git dependency: mic-viewer is a private repository and the test VM, where
# every build and test of this repository runs, cannot fetch private repositories (mic-viewer's
# docs/choices-viewer.md VC4). The copy is the crate folder at one commit, named in
# vendor/mic-viewer/PINNED; the workspace excludes it (Cargo.toml `exclude`), so its own tests and
# lints stay in its repository. Edit it there, never here: this script overwrites the folder.
#
# MIC_VIEWER_REPO names the checkout (default: ../mic-viewer beside this repository's folder).

set -euo pipefail
cd "$(dirname "$0")/../.."

SRC="${MIC_VIEWER_REPO:-$(dirname "$(git rev-parse --show-toplevel)")/mic-viewer}"
REV="${1:-origin/main}"
[ -d "$SRC/.git" ] || { echo "refresh-mic-viewer: no mic-viewer checkout at $SRC (set MIC_VIEWER_REPO)" >&2; exit 2; }
git -C "$SRC" fetch -q origin
commit="$(git -C "$SRC" rev-parse --verify "$REV^{commit}")"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
git -C "$SRC" archive "$commit" crates/mic-viewer LICENSE | tar -x -C "$tmp"
rm -rf vendor/mic-viewer
mkdir -p vendor/mic-viewer
cp -R "$tmp/crates/mic-viewer/Cargo.toml" "$tmp/crates/mic-viewer/rustfmt.toml" \
      "$tmp/crates/mic-viewer/src" "$tmp/crates/mic-viewer/web" vendor/mic-viewer/
cp "$tmp/LICENSE" vendor/mic-viewer/LICENSE
cat > vendor/mic-viewer/PINNED <<PIN
# vendor/mic-viewer is MinecraftIndustrialComplex/mic-viewer's crates/mic-viewer at this commit
# (MPL-2.0, LICENSE beside this file). Refresh with tools/dev/refresh-mic-viewer.sh; never edit here.
commit = $commit
date = $(git -C "$SRC" show -s --format=%cs "$commit")
PIN
echo "refresh-mic-viewer: vendor/mic-viewer is now $commit"

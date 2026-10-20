#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The end-of-run CO2 hunt (2026-10-19): builds the `deeptime` CLI at several commits on the test
# VM, whose repo copies have no history, from `git archive` tarballs the laptop copied into
# <run dir>/src/<commit>.tar, one after another in one build folder so the dependencies stay
# built. Each build is admitted on its own (cargo:build); the binaries land in <run dir>/bins/.
# Start it detached through mac.sh (plain words only):
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start co2-build tools/review/co2-bisect.sh \
#       <run dir> <commit> [<commit> ...]
set -euo pipefail
R=$1; shift
CACHE=/data/ban/deeptime-cache
GIT=$CACHE/git/bin/git
export CARGO_HOME=$CACHE/cargo TMPDIR=$CACHE/tmp CARGO_INCREMENTAL=0
mkdir -p "$R/build" "$R/bins"
for c in "$@"; do
  if [ -x "$R/bins/deeptime-$c" ]; then continue; fi
  rm -rf "$R/x"; mkdir -p "$R/x"
  tar -xf "$R/src/$c.tar" -C "$R/x"
  # Copy only what changed and give it the current time (cargo fingerprints by mtime).
  rsync -rlpD --checksum --no-times --delete --exclude=/target/ --exclude=/.git "$R/x/" "$R/build/"
  cd "$R/build"
  [ -d .git ] || "$GIT" init -q -b master
  "$GIT" add -A
  KEY=$(cat flake.nix flake.lock | sha256sum | cut -c1-16)
  PROFILE=$CACHE/devshell-$KEY
  [ -e "$PROFILE" ] || nix develop --profile "$PROFILE" -c true
  env -u DEEPTIME_ADMIT_JOB -u DEEPTIME_ADMIT_PID /usr/bin/python3 /usr/local/lib/deeptime/admit.py \
    run --kind cargo:build --label "co2-bisect-$c" -- \
    nix develop "$PROFILE" -c cargo build --release -p deeptime-cli
  cp target/release/deeptime "$R/bins/deeptime-$c"
  echo "built $c"
done
rm -rf "$R/x"
echo BUILT

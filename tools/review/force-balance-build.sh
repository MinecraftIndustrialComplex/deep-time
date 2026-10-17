#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Force-balance tasks 7 and 8: build the release CLI and copy it to <dest> (for
# force-balance-ensemble.sh), from the repo root, inside the dev shell (detach.sh start).
set -euo pipefail
cargo build --release -q -p deeptime-cli
cp target/release/deeptime "$1"
echo "BUILD_OK $1"

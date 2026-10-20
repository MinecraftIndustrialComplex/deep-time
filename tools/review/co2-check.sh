#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The end-of-run CO2 fix's quick check (2026-10-19), from the repo root in the dev shell: the
# core's clippy, the present-day climate's tests, then the release CLI copied to <bin out>.
# Plain words for detach.sh:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start <job> --kind cargo:test \
#       tools/review/co2-check.sh <bin out> [test filter ...]
set -euo pipefail
OUT=$1; shift
cargo clippy -p deeptime-core -p deeptime-cli --all-targets -- -D warnings
for f in "$@"; do
  cargo test -p deeptime-core --lib -- "$f"
done
cargo build --release -p deeptime-cli
mkdir -p "$(dirname "$OUT")"
cp target/release/deeptime "$OUT"
echo "binary at $OUT"

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-15: dump a baked world's sites and list its porphyries (bodies, children, distance to spawn).
#   tools/review/p1-15-dump.sh <world-data folder> <out json> <spawn x> <spawn z> [type suffix]
set -euo pipefail
cd "$(dirname "$0")/../.."
target/release/deeptime dump-deposits --world "$1" --out "$2" > /dev/null
python3 tools/review/p1-15-bodies.py "$2" "$3" "$4" ${5:+"$5"}

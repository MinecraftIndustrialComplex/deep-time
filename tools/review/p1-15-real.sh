#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-15 / Phase 1 exit items 3 and 4 on a real baked planet (run on the test VM through
# tools/remote/mac.sh, detached): a first start of earthlike_quick_16k (scattered continents,
# 500 Myr) on the given profile, then the RCON probes listed one per line in a file.
#
#   tools/review/p1-15-real.sh boot <standalone|mic> <probes-file> [run.py args…]
#       install (if the profile has no server yet, or REINSTALL=1), run.py --fresh with every line
#       of <probes-file> as an --rcon command; the report, rcon.jsonl and the spawn files are
#       copied to review/out/p1-15-real/<profile>/.
#   tools/review/p1-15-real.sh again <standalone|mic> <probes-file> [run.py args…]
#       the same on the world the last boot generated (no --fresh).
set -euo pipefail
cd "$(dirname "$0")/../.."
verb="${1:?boot|again}"; profile="${2:?profile}"; probes="${3:?probes file}"; shift 3
out="review/out/p1-15-real/$profile"
mkdir -p "$out"
args=()
while IFS= read -r line; do
    [ -n "$line" ] && args+=(--rcon "$line")
done < "$probes"
if [ "$verb" = boot ]; then
    if [ -n "${REINSTALL:-}" ] || [ ! -d "smoke/$profile/server" ]; then
        tools/smoke/install.sh "$profile" --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --clean
    fi
    fresh=--fresh
else
    fresh=
fi
set +e
python3 tools/smoke/run.py --profile "$profile" $fresh "${args[@]}" --timeout 3000 "$@"
rc=$?
set -e
cp "smoke/$profile/report.md" "smoke/$profile/rcon.jsonl" "$out/" 2>/dev/null || true
world="smoke/$profile/server/planet/deeptime"
cp "$world/spawn/candidates.json" "$out/" 2>/dev/null || true
exit $rc

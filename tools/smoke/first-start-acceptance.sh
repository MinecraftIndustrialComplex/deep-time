#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-13 acceptance (Phase 1 exit test item 2): the dedicated server generates its world on the
# first start from a real built-in preset. Per profile: the three clean-stop scenarios, then a
# fresh first start with a restart (progress lines, no regeneration, mixins, probes against the
# world data, first-start time, peak native memory). Runs where install.sh and run.py run (the test
# VM); start it detached:
#   tools/remote/mac.sh [--pack] -- tools/remote/detach.sh start p113-<profile> bash tools/smoke/first-start-acceptance.sh <profile> [preset]
set -uo pipefail
cd "$(dirname "$0")/../.."
P="${1:?usage: first-start-acceptance.sh <standalone|mic> [preset]}"
PRESET="${2:-earthlike_quick_16k}"
tools/smoke/install.sh "$P" --preset "$PRESET" --start supercontinent --length-myr 1000 --clean || { echo "INSTALL-FAIL"; exit 1; }
for sc in first_start_no_config first_start_circumference_mismatch first_start_native_failure; do
    if python3 tools/smoke/run.py --profile "$P" --scenario "$sc"; then echo "SCENARIO-OK $sc"; else echo "SCENARIO-FAIL $sc"; fi
    cp "smoke/$P/report.md" "smoke/$P/report-$sc.md" 2>/dev/null || true
done
python3 tools/smoke/run.py --profile "$P" --fresh --restart
rc=$?
echo "FIRST-START-RUN rc=$rc"
exit $rc

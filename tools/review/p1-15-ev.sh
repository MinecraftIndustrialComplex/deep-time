#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# P1-15 gate: install the EV server set (a worktree copy has none), then ./gametest.sh -PwithEv.
set -uo pipefail
cd "$(dirname "$0")/../.."
tools/smoke/install.sh ev > review/out/p1-15-real/gate-ev-install.log 2>&1 || { echo "ev install rc=$?" | tee -a review/out/p1-15-real/gates.txt; exit 1; }
./gametest.sh -PwithEv > review/out/p1-15-real/gate-gt-ev2.log 2>&1
echo "gt-ev2 rc=$? $(tail -1 review/out/p1-15-real/gate-gt-ev2.log | cut -c1-160)" | tee -a review/out/p1-15-real/gates.txt

#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The Phase 1 exit (plan §10.3, P1-19): Phase 0's suite again plus items 1 to 8 of the "Phase 1 exit test", each a
# PASS/FAIL row with its evidence paths, and the draft of the private page "Deep Time: Phase 1 (MVP)".
#
#   tools/review/phase1.sh [--only <step>[,<step>...]] [--skip-phase0] [--publish] [--list]
#
# Runs on this laptop and drives the test VM through tools/remote/mac.sh (every heavy command is admitted by the
# VM's admission tool). The work is tools/review/phase1.py; read its docstring for the steps, the rows and where
# everything lands (review/out/phase1-src/: index.html, exit-table.md, rows.json, logs/; evidence in
# review/out/phase1/). `--only scale` re-runs one step and rebuilds the page from the others' last rows.
# The page is never published unless --publish is given and every row passes.
set -euo pipefail
exec python3 "$(dirname "$0")/phase1.py" "$@"

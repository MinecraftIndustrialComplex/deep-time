#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The command of a detached Phase 1 job, passed as one base64 word (tools/review/phase1_detach.py): detach.sh takes
# plain words only (no quotes, no pipes), the suite's commands have both. Runs from the repo root on the test VM.
set -uo pipefail
eval "$(printf %s "${1:?base64 command}" | base64 -d)"

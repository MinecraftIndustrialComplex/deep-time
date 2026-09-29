#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# From the laptop: generate one or more boxes of a Deep Time world on the Mac's headless server,
# export each for the 3D viewer, and pull the job folder back (P1-20 stage 2).
#
#   tools/workbench/preview-area.sh [--pack] [--name <job id>] [--install] \
#       --profile <standalone|mic|mic-ev|terrablender|ev> [--size 16384|32768] \
#       --box x0,z0,x1,z1 [--box …] [--y lo,hi] \
#       ( --synth <kind> [--strata <kind>] | --preset <name|file> [--start S] [--length-myr N] \
#         | --world-data <dir> | --test-terrain <kind> ) \
#       [--datapack <dir>]… [--seed N] [--no-mic-rock-tags] [--keep-world] [--max-side N]
#
#   --pack     also sync the MIC pack and its rock-tag datapack to the Mac (mic profiles)
#   --name     the job id (default: <date>-<time>-laptop)
#   --install  run tools/smoke/install.sh <profile> on the Mac first (always after Java changes:
#              the server set carries the Deep Time jar it was installed with)
#
# Everything else goes to tools/workbench/preview.py run (README.md next to this file). Paths are
# relative to the repo root. The run holds the Mac's server lock (mac.sh --server), waiting for it
# if another agent's server holds it. The job lands in smoke/workbench/jobs/<job>/ here:
# status.json, area-<i>.dtva.gz, probes.jsonl, server.log. Open an export with
#   tools/workbench/view.sh smoke/workbench/jobs/<job>/area-0.dtva.gz

set -euo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"

MAC=(tools/remote/mac.sh --server)
JOB=""; PASS=()
while [ $# -gt 0 ]; do
    case "$1" in
        --pack) MAC+=(--pack); shift ;;
        --name) JOB="$2"; shift 2 ;;
        --install|--no-mic-rock-tags|--keep-world) PASS+=("$1"); shift ;;
        --*=*) PASS+=("$1"); shift ;;
        --*) [ $# -ge 2 ] || { echo "preview-area.sh: $1 needs a value" >&2; exit 2; }
             # --opt=value, so argparse never reads a negative box (-5698,…) as an option.
             PASS+=("$1=$2"); shift 2 ;;
        *) echo "preview-area.sh: unexpected argument '$1'" >&2; exit 2 ;;
    esac
done
[ -n "$JOB" ] || JOB="$(date +%Y%m%d-%H%M%S)-laptop"
REL="smoke/workbench/jobs/$JOB"

QUOTED=""
for a in "${PASS[@]}"; do QUOTED+=" $(printf '%q' "$a")"; done
set +e
"${MAC[@]}" --pull "$REL" -- "python3 tools/workbench/preview.py run --lock-held --job $JOB$QUOTED"
status=$?
set -e
if [ -f "$REL/status.json" ]; then
    python3 - "$REL/status.json" <<'PY'
import json, sys
s = json.load(open(sys.argv[1]))
print(f"\njob {s['job']}: {s['state']} — {s.get('message', '')}")
print("timings:", json.dumps(s.get("timings", {})))
for e in s.get("exports", []):
    st = e["summary"]["stats"]
    print(f"  {e['file']}: box {e['box']}, {e['bytes'] / 1e6:.2f} MB, {st['non_air']:,} blocks, {st['runs']:,} runs, "
          f"{e['summary']['bodies']} bodies, {e['summary']['sites']} sites")
PY
fi
exit "$status"
